//! The covariance companion named by `covariance_file`.
//!
//! `docs/decisions/input-contract.md` states this as the one place the contract
//! is not a single file, and says why: a dense covariance over a few hundred
//! levels is tens of thousands of numbers and does not belong inline in a file
//! somebody reads. The record fixes that the companion carries the same
//! `contract_version` and `level_set_id`, that its body is `level_id` by
//! `level_id` by value over the upper triangle including the diagonal, that a
//! pair not listed is zero, and that a `level_id` in it which is not in the
//! level set is refused rather than ignored. It names no column labels, so the
//! labels below are fixed here and are owed back to the record.

use crate::document::Document;
use crate::level_set::LevelSet;
use crate::reading::Reading;
use crate::refusal::{Refusal, Refusals};
use crate::value::{Version, parse_number};

const HEADER_FIELDS: &[&str] = &["contract_version", "level_set_id"];

const COLUMNS: &[&str] = &["level_id_a", "level_id_b", "covariance"];

/// One entry of the upper triangle.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub line: usize,
    pub level_id_a: String,
    pub level_id_b: String,
    pub covariance: f64,
}

/// A covariance companion that a reader accepted.
#[derive(Debug, Clone, PartialEq)]
pub struct Covariance {
    pub contract_version: Version,
    pub level_set_id: String,
    pub entries: Vec<Entry>,
    pub unknown_header_fields: Vec<String>,
    pub unknown_columns: Vec<String>,
}

/// Read a covariance companion, or refuse naming the field and the line.
pub fn read(bytes: &[u8]) -> Result<Covariance, Refusals> {
    let document = Document::lex(bytes)?;
    let mut reading = Reading::new(&document);

    let contract_version = reading.contract_version();
    let level_set_id = reading.required_header("level_set_id");

    let first = reading.required_column("level_id_a");
    let second = reading.required_column("level_id_b");
    let value = reading.required_column("covariance");

    let mut entries: Vec<Entry> = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new();

    for row in &document.rows {
        let read_first = first.and_then(|column| reading.required_cell(row, column, "level_id_a"));
        let read_second =
            second.and_then(|column| reading.required_cell(row, column, "level_id_b"));
        let read_value =
            value.and_then(|column| reading.parsed_cell(row, column, "covariance", parse_number));

        if let (Some(level_id_a), Some(level_id_b), Some(covariance)) =
            (read_first, read_second, read_value)
        {
            let key = (level_id_a.clone(), level_id_b.clone());
            let mirrored = (level_id_b.clone(), level_id_a.clone());
            if seen.contains(&key) || seen.contains(&mirrored) {
                reading.refuse(Refusal::on_line(
                    row.line,
                    format!(
                        "the pair `{level_id_a}` and `{level_id_b}` appears more than once, and the body is the upper triangle rather than the whole matrix"
                    ),
                ));
                continue;
            }
            seen.push(key);
            entries.push(Entry {
                line: row.line,
                level_id_a,
                level_id_b,
                covariance,
            });
        }
    }

    let unknown_header_fields = document.headers_outside(HEADER_FIELDS);
    let unknown_columns = document.columns_outside(COLUMNS);

    let built = match (contract_version, level_set_id) {
        (Some(contract_version), Some(level_set_id)) => Some(Covariance {
            contract_version,
            level_set_id: level_set_id.to_owned(),
            entries,
            unknown_header_fields,
            unknown_columns,
        }),
        _ => None,
    };
    reading.finish(built)
}

impl Covariance {
    /// Refuse a companion that is not about this level set.
    pub fn check_against(&self, levels: &LevelSet) -> Result<(), Refusals> {
        let mut refusals = Refusals::new();
        if self.level_set_id != levels.level_set_id {
            refusals.push(Refusal::absent_field(
                "level_set_id",
                format!(
                    "the companion is stated against `{}` and the level set is `{}`",
                    self.level_set_id, levels.level_set_id
                ),
            ));
        }
        for entry in &self.entries {
            for (label, id) in [
                ("level_id_a", &entry.level_id_a),
                ("level_id_b", &entry.level_id_b),
            ] {
                if !levels.levels.iter().any(|level| &level.level_id == id) {
                    refusals.push(Refusal::field_on_line(
                        entry.line,
                        label,
                        format!("`{id}` names a level the level set does not carry"),
                    ));
                }
            }
        }
        refusals.into_result(())
    }
}
