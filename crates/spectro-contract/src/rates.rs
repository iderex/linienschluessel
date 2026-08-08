//! The transition rate table, which is the third file.
//!
//! `docs/decisions/input-contract.md` records why it cannot be a column in
//! either of the other two: a rate is a property of a pair of levels, so it is
//! a property of neither an observed feature nor a single level. The record
//! fixes the two keys, the identifier the table is stated against, and that the
//! table carries the rate, its unit and its uncertainty. It names no column
//! labels, so the labels below are fixed here and are owed back to the record.
//!
//! The table is optional and the board runs without it. A shared upper level
//! whose rates are missing is issue #34's case.

use crate::document::Document;
use crate::level_set::LevelSet;
use crate::reading::Reading;
use crate::refusal::{Refusal, Refusals};
use crate::value::{Version, parse_number};

const HEADER_FIELDS: &[&str] = &["contract_version", "level_set_id", "rate_unit"];

const COLUMNS: &[&str] = &[
    "upper_level_id",
    "lower_level_id",
    "rate",
    "rate_uncertainty",
];

/// One rate, for the transition between one pair of levels.
#[derive(Debug, Clone, PartialEq)]
pub struct Rate {
    pub line: usize,
    pub upper_level_id: String,
    pub lower_level_id: String,
    pub rate: f64,
    pub rate_uncertainty: Option<f64>,
}

/// A rate table that a reader accepted.
#[derive(Debug, Clone, PartialEq)]
pub struct RateTable {
    pub contract_version: Version,
    pub level_set_id: String,
    pub rate_unit: String,
    pub rates: Vec<Rate>,
    pub unknown_header_fields: Vec<String>,
    pub unknown_columns: Vec<String>,
}

/// Read a rate table, or refuse naming every field and line that failed.
pub fn read(bytes: &[u8]) -> Result<RateTable, Refusals> {
    let document = Document::lex(bytes)?;
    let mut reading = Reading::new(&document);

    let contract_version = reading.contract_version();
    let level_set_id = reading.required_header("level_set_id");
    let rate_unit = reading.required_header("rate_unit");

    let upper = reading.required_column("upper_level_id");
    let lower = reading.required_column("lower_level_id");
    let rate = reading.required_column("rate");
    let uncertainty = reading.required_column("rate_uncertainty");

    let mut rates: Vec<Rate> = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new();

    for row in &document.rows {
        let read_upper =
            upper.and_then(|column| reading.required_cell(row, column, "upper_level_id"));
        let read_lower =
            lower.and_then(|column| reading.required_cell(row, column, "lower_level_id"));
        let read_rate =
            rate.and_then(|column| reading.parsed_cell(row, column, "rate", parse_number));
        let read_uncertainty = uncertainty.and_then(|column| {
            reading.parsed_cell(row, column, "rate_uncertainty", |cell| {
                if cell == "none" {
                    Ok(None)
                } else {
                    parse_number(cell).map(Some).map_err(|reason| {
                        format!("{reason}, and an absent uncertainty is written `none`")
                    })
                }
            })
        });

        if let (Some(upper), Some(lower), Some(rate), Some(rate_uncertainty)) =
            (read_upper, read_lower, read_rate, read_uncertainty)
        {
            if upper == lower {
                reading.refuse(Refusal::field_on_line(
                    row.line,
                    "upper_level_id",
                    format!("`{upper}` is named as both ends of one transition"),
                ));
                continue;
            }
            let key = (upper.clone(), lower.clone());
            if seen.contains(&key) {
                reading.refuse(Refusal::on_line(
                    row.line,
                    format!("the pair `{upper}` to `{lower}` appears more than once, and a rate is one number per transition"),
                ));
                continue;
            }
            seen.push(key);
            rates.push(Rate {
                line: row.line,
                upper_level_id: upper,
                lower_level_id: lower,
                rate,
                rate_uncertainty,
            });
        }
    }

    let unknown_header_fields = document.headers_outside(HEADER_FIELDS);
    let unknown_columns = document.columns_outside(COLUMNS);

    let built = match (contract_version, level_set_id, rate_unit) {
        (Some(contract_version), Some(level_set_id), Some(rate_unit)) => Some(RateTable {
            contract_version,
            level_set_id: level_set_id.to_owned(),
            rate_unit: rate_unit.to_owned(),
            rates,
            unknown_header_fields,
            unknown_columns,
        }),
        _ => None,
    };
    reading.finish(built)
}

impl RateTable {
    /// Refuse a rate table that is not about this level set.
    ///
    /// A rate naming a level the set does not carry is refused rather than
    /// ignored, for the same reason the covariance companion refuses one: a
    /// silently dropped row is a constraint missing from an answer that does
    /// not say so.
    pub fn check_against(&self, levels: &LevelSet) -> Result<(), Refusals> {
        let mut refusals = Refusals::new();
        if self.level_set_id != levels.level_set_id {
            refusals.push(Refusal::absent_field(
                "level_set_id",
                format!(
                    "the rate table is stated against `{}` and the level set is `{}`",
                    self.level_set_id, levels.level_set_id
                ),
            ));
        }
        for rate in &self.rates {
            for (label, id) in [
                ("upper_level_id", &rate.upper_level_id),
                ("lower_level_id", &rate.lower_level_id),
            ] {
                if !levels.levels.iter().any(|level| &level.level_id == id) {
                    refusals.push(Refusal::field_on_line(
                        rate.line,
                        label,
                        format!("`{id}` names a level the level set does not carry"),
                    ));
                }
            }
        }
        refusals.into_result(())
    }
}
