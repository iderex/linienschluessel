//! The level set of `docs/decisions/input-contract.md`.

use crate::document::Document;
use crate::reading::Reading;
use crate::refusal::{Refusal, Refusals};
use crate::value::{HalfInteger, ORIGIN, PARITY, UNCERTAINTY_KIND, Version, parse_number};

const HEADER_FIELDS: &[&str] = &[
    "contract_version",
    "energy_unit",
    "energy_reference",
    "level_set_id",
    "covariance_file",
    "derived_from_line_lists",
];

const COLUMNS: &[&str] = &[
    "level_id",
    "species",
    "energy",
    "energy_uncertainty",
    "uncertainty_kind",
    "uncertainty_class",
    "parity",
    "j",
    "origin",
    "configuration",
    "term",
];

/// One level, with every field the record says the objective needs.
#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    /// The line of the file this level was read from, so that anything derived
    /// from it can say where it came from.
    pub line: usize,
    pub level_id: String,
    pub species: String,
    pub energy: f64,
    /// `None` where the file said `none`, which is a declared absence and is
    /// not the same as a level whose uncertainty nobody wrote down.
    pub energy_uncertainty: Option<f64>,
    pub uncertainty_kind: &'static str,
    pub uncertainty_class: Option<String>,
    pub parity: &'static str,
    /// `None` where the file said `unknown`. A file with no `j` column at all
    /// is refused, so this `None` is always a fact about the spectrum.
    pub j: Option<HalfInteger>,
    pub origin: &'static str,
    pub configuration: String,
    pub term: String,
}

/// A level set that a reader accepted.
#[derive(Debug, Clone, PartialEq)]
pub struct LevelSet {
    pub contract_version: Version,
    pub energy_unit: String,
    pub energy_reference: String,
    pub level_set_id: String,
    pub covariance_file: Option<String>,
    pub derived_from_line_lists: Option<String>,
    pub levels: Vec<Level>,
    /// Header fields and column labels this reader did not know, kept and
    /// reported rather than dropped, which is what the minor version rule asks
    /// of a reader meeting a later minor than it was written for.
    pub unknown_header_fields: Vec<String>,
    pub unknown_columns: Vec<String>,
}

/// Read a level set, or refuse naming every field and line that failed.
pub fn read(bytes: &[u8]) -> Result<LevelSet, Refusals> {
    let document = Document::lex(bytes)?;
    read_document(&document)
}

fn read_document(document: &Document) -> Result<LevelSet, Refusals> {
    let mut reading = Reading::new(document);

    let contract_version = reading.contract_version();
    let energy_unit = reading.required_header("energy_unit");
    let energy_reference = reading.required_header("energy_reference");
    let level_set_id = reading.required_header("level_set_id");
    let covariance_file = reading.optional_header("covariance_file");
    let derived_from_line_lists = reading.optional_header("derived_from_line_lists");

    let level_id = reading.required_column("level_id");
    let species = reading.required_column("species");
    let energy = reading.required_column("energy");
    let energy_uncertainty = reading.required_column("energy_uncertainty");
    let uncertainty_kind = reading.required_column("uncertainty_kind");
    let uncertainty_class = reading.required_column("uncertainty_class");
    let parity = reading.required_column("parity");
    let j = reading.required_column("j");
    let origin = reading.required_column("origin");
    let configuration = reading.required_column("configuration");
    let term = reading.required_column("term");

    let mut levels: Vec<Level> = Vec::new();
    let mut seen: Vec<String> = Vec::new();

    for row in &document.rows {
        let read_level_id = level_id.and_then(|column| {
            let value = reading.required_cell(row, column, "level_id")?;
            if seen.contains(&value) {
                reading.refuse(Refusal::field_on_line(
                    row.line,
                    "level_id",
                    format!("`{value}` names a level this file already carried, and a level identifier is unique in its file"),
                ));
                return None;
            }
            seen.push(value.clone());
            Some(value)
        });
        let read_species = species.and_then(|column| reading.required_cell(row, column, "species"));
        let read_energy =
            energy.and_then(|column| reading.parsed_cell(row, column, "energy", parse_number));
        let read_energy_uncertainty = energy_uncertainty.and_then(|column| {
            reading.parsed_cell(row, column, "energy_uncertainty", |cell| {
                if cell == "none" {
                    Ok(None)
                } else {
                    parse_number(cell).map(Some).map_err(|reason| {
                        format!("{reason}, and an absent uncertainty is written `none`")
                    })
                }
            })
        });
        let read_uncertainty_kind = uncertainty_kind.and_then(|column| {
            reading.vocabulary_cell(row, column, "uncertainty_kind", UNCERTAINTY_KIND)
        });
        let read_uncertainty_class = uncertainty_class
            .and_then(|column| reading.uncertainty_class(row, column, read_uncertainty_kind));
        let read_parity =
            parity.and_then(|column| reading.vocabulary_cell(row, column, "parity", PARITY));
        let read_j = j.and_then(|column| {
            reading.parsed_cell(row, column, "j", |cell| {
                if cell == "unknown" {
                    Ok(None)
                } else {
                    HalfInteger::parse(cell).map(Some).map_err(|reason| {
                        format!("{reason}, and a J that was never determined is written `unknown`")
                    })
                }
            })
        });
        let read_origin =
            origin.and_then(|column| reading.vocabulary_cell(row, column, "origin", ORIGIN));
        let read_configuration = configuration.map(|column| reading.cell(row, column).to_owned());
        let read_term = term.map(|column| reading.cell(row, column).to_owned());

        if let (
            Some(level_id),
            Some(species),
            Some(energy),
            Some(energy_uncertainty),
            Some(uncertainty_kind),
            Some(parity),
            Some(j),
            Some(origin),
            Some(configuration),
            Some(term),
        ) = (
            read_level_id,
            read_species,
            read_energy,
            read_energy_uncertainty,
            read_uncertainty_kind,
            read_parity,
            read_j,
            read_origin,
            read_configuration,
            read_term,
        ) {
            levels.push(Level {
                line: row.line,
                level_id,
                species,
                energy,
                energy_uncertainty,
                uncertainty_kind,
                uncertainty_class: read_uncertainty_class.flatten(),
                parity,
                j,
                origin,
                configuration,
                term,
            });
        }
    }

    let unknown_header_fields = document.headers_outside(HEADER_FIELDS);
    let unknown_columns = document.columns_outside(COLUMNS);

    let built = match (
        contract_version,
        energy_unit,
        energy_reference,
        level_set_id,
    ) {
        (Some(contract_version), Some(energy_unit), Some(energy_reference), Some(level_set_id)) => {
            Some(LevelSet {
                contract_version,
                energy_unit: energy_unit.to_owned(),
                energy_reference: energy_reference.to_owned(),
                level_set_id: level_set_id.to_owned(),
                covariance_file: covariance_file.map(str::to_owned),
                derived_from_line_lists: derived_from_line_lists.map(str::to_owned),
                levels,
                unknown_header_fields,
                unknown_columns,
            })
        }
        _ => None,
    };
    reading.finish(built)
}
