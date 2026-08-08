//! The line list of `docs/decisions/input-contract.md`.

use crate::document::{Document, Row};
use crate::reading::Reading;
use crate::refusal::{Refusal, Refusals};
use crate::value::{
    INTENSITY_SCALE, LINE_FLAG, POSITION_MEDIUM, UNCERTAINTY_KIND, Version, parse_number,
    parse_vocabulary,
};

const HEADER_FIELDS: &[&str] = &[
    "contract_version",
    "position_unit",
    "position_medium",
    "line_list_id",
];

const COLUMNS: &[&str] = &[
    "feature_id",
    "spectrum_id",
    "segment_id",
    "position",
    "position_medium",
    "position_uncertainty",
    "uncertainty_kind",
    "uncertainty_class",
    "intensity",
    "intensity_scale",
    "flags",
    "ritz_position",
];

/// One observed feature.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub line: usize,
    pub feature_id: String,
    pub spectrum_id: String,
    pub segment_id: Option<String>,
    pub position: f64,
    /// The medium this row is in, which is the header default unless the row
    /// overrode it. Resolved here so that nothing downstream has to remember
    /// which of the two it is holding.
    pub position_medium: String,
    pub position_uncertainty: Option<f64>,
    pub uncertainty_kind: &'static str,
    pub uncertainty_class: Option<String>,
    pub intensity: Option<f64>,
    pub intensity_scale: Option<&'static str>,
    pub flags: Vec<&'static str>,
    /// Read so that a producer holding one does not have to throw it away, and
    /// never read by the objective: a Ritz position was computed from a level
    /// set, so scoring it against a difference of that same set measures
    /// arithmetic rather than a spectrum.
    pub ritz_position: Option<f64>,
}

/// A line list that a reader accepted.
#[derive(Debug, Clone, PartialEq)]
pub struct LineList {
    pub contract_version: Version,
    pub position_unit: String,
    pub position_medium: String,
    pub line_list_id: String,
    pub lines: Vec<Line>,
    pub unknown_header_fields: Vec<String>,
    pub unknown_columns: Vec<String>,
}

/// Read a line list, or refuse naming every field and line that failed.
pub fn read(bytes: &[u8]) -> Result<LineList, Refusals> {
    let document = Document::lex(bytes)?;
    read_document(&document)
}

fn read_document(document: &Document) -> Result<LineList, Refusals> {
    let mut reading = Reading::new(document);

    let contract_version = reading.contract_version();
    let position_unit = reading.required_header("position_unit");
    let line_list_id = reading.required_header("line_list_id");
    let default_medium = read_header_medium(&mut reading);

    let feature_id = reading.required_column("feature_id");
    let spectrum_id = reading.required_column("spectrum_id");
    let position = reading.required_column("position");
    let position_uncertainty = reading.required_column("position_uncertainty");
    let uncertainty_kind = reading.required_column("uncertainty_kind");
    let uncertainty_class = reading.required_column("uncertainty_class");
    let flags = reading.required_column("flags");
    let segment_id = reading.optional_column("segment_id");
    let row_medium = reading.optional_column("position_medium");
    let intensity = reading.optional_column("intensity");
    let intensity_scale = reading.optional_column("intensity_scale");
    let ritz_position = reading.optional_column("ritz_position");

    let mut lines: Vec<Line> = Vec::new();
    let mut seen: Vec<String> = Vec::new();

    for row in &document.rows {
        let read_feature_id = feature_id.and_then(|column| {
            let value = reading.required_cell(row, column, "feature_id")?;
            if seen.contains(&value) {
                reading.refuse(Refusal::field_on_line(
                    row.line,
                    "feature_id",
                    format!("`{value}` names a feature this file already carried, and a feature identifier is unique in its file"),
                ));
                return None;
            }
            seen.push(value.clone());
            Some(value)
        });
        let read_spectrum_id =
            spectrum_id.and_then(|column| reading.required_cell(row, column, "spectrum_id"));
        let read_position =
            position.and_then(|column| reading.parsed_cell(row, column, "position", parse_number));
        let read_position_uncertainty = position_uncertainty.and_then(|column| {
            reading.parsed_cell(row, column, "position_uncertainty", |cell| {
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
        let read_segment_id = segment_id.map(|column| {
            let cell = reading.cell(row, column);
            if cell.is_empty() {
                None
            } else {
                Some(cell.to_owned())
            }
        });
        let read_medium = read_row_medium(&mut reading, row, row_medium, default_medium);
        let read_intensity = read_intensity(&mut reading, row, intensity, intensity_scale);
        let read_flags = flags.and_then(|column| read_flags(&mut reading, row, column));
        let read_ritz = ritz_position.and_then(|column| {
            let cell = reading.cell(row, column);
            if cell.is_empty() {
                Some(None)
            } else {
                reading
                    .parsed_cell(row, column, "ritz_position", parse_number)
                    .map(Some)
            }
        });

        if let (
            Some(feature_id),
            Some(spectrum_id),
            Some(position),
            Some(position_medium),
            Some(position_uncertainty),
            Some(uncertainty_kind),
            Some((intensity, intensity_scale)),
            Some(flags),
        ) = (
            read_feature_id,
            read_spectrum_id,
            read_position,
            read_medium,
            read_position_uncertainty,
            read_uncertainty_kind,
            read_intensity,
            read_flags,
        ) {
            lines.push(Line {
                line: row.line,
                feature_id,
                spectrum_id,
                segment_id: read_segment_id.flatten(),
                position,
                position_medium,
                position_uncertainty,
                uncertainty_kind,
                uncertainty_class: read_uncertainty_class.flatten(),
                intensity,
                intensity_scale,
                flags,
                ritz_position: read_ritz.flatten(),
            });
        }
    }

    let unknown_header_fields = document.headers_outside(HEADER_FIELDS);
    let unknown_columns = document.columns_outside(COLUMNS);

    let built = match (
        contract_version,
        position_unit,
        default_medium,
        line_list_id,
    ) {
        (
            Some(contract_version),
            Some(position_unit),
            Some(position_medium),
            Some(line_list_id),
        ) => Some(LineList {
            contract_version,
            position_unit: position_unit.to_owned(),
            position_medium: position_medium.to_owned(),
            line_list_id: line_list_id.to_owned(),
            lines,
            unknown_header_fields,
            unknown_columns,
        }),
        _ => None,
    };
    reading.finish(built)
}

/// The header default medium.
///
/// The record says the medium is a value and never a rule, so a header naming
/// some source's own boundary convention is refused here rather than resolved.
/// Resolving a convention into an explicit per-row medium is what an adapter is
/// for, and a contract that took the rule would have taken one upstream's
/// convention into itself.
fn read_header_medium(reading: &mut Reading<'_>) -> Option<&'static str> {
    let Some(header) = reading.document().header("position_medium").cloned() else {
        reading.refuse(Refusal::absent_field(
            "position_medium",
            "the file carries no such header field, so the rows that state no medium of their own have none",
        ));
        return None;
    };
    match parse_vocabulary(&header.value, POSITION_MEDIUM) {
        Ok(member) => Some(member),
        Err(reason) => {
            reading.refuse(Refusal::field_on_line(
                header.line,
                "position_medium",
                format!(
                    "{reason}, and a medium is a value rather than a rule: a header naming a source's own boundary convention is refused"
                ),
            ));
            None
        }
    }
}

fn read_row_medium(
    reading: &mut Reading<'_>,
    row: &Row,
    column: Option<usize>,
    default: Option<&'static str>,
) -> Option<String> {
    let Some(column) = column else {
        let Some(default) = default else {
            reading.refuse(Refusal::absent_field(
                "position_medium",
                "no row states a medium and the header states none either, so no position in this file has one",
            ));
            return None;
        };
        return Some(default.to_owned());
    };
    let cell = row.cells[column].as_str();
    if cell.is_empty() {
        let Some(default) = default else {
            reading.refuse(Refusal::field_on_line(
                row.line,
                "position_medium",
                "the row states no medium and the header states no default, so this position has none",
            ));
            return None;
        };
        return Some(default.to_owned());
    }
    reading
        .vocabulary_cell(row, column, "position_medium", POSITION_MEDIUM)
        .map(str::to_owned)
}

/// An intensity and the scale it is on, which the record requires together.
///
/// The branching constraint takes a different form on a photon scale than on an
/// energy scale, so an intensity with no scale is refused rather than guessed
/// at. An absent `intensity_scale` column is refused only where some row
/// actually carries an intensity, because a file with no intensities at all is
/// a conforming file.
fn read_intensity(
    reading: &mut Reading<'_>,
    row: &Row,
    intensity: Option<usize>,
    intensity_scale: Option<usize>,
) -> Option<(Option<f64>, Option<&'static str>)> {
    let Some(column) = intensity else {
        return Some((None, None));
    };
    let cell = row.cells[column].as_str();
    if cell.is_empty() {
        return Some((None, None));
    }
    let value = reading.parsed_cell(row, column, "intensity", parse_number);
    let Some(scale_column) = intensity_scale else {
        reading.refuse(Refusal::absent_field(
            "intensity_scale",
            "the file carries intensities and no column saying what scale they are on",
        ));
        return None;
    };
    let scale_cell = row.cells[scale_column].as_str();
    if scale_cell.is_empty() {
        reading.refuse(Refusal::field_on_line(
            row.line,
            "intensity_scale",
            "the row carries an intensity and states no scale for it",
        ));
        return None;
    }
    let scale = reading.vocabulary_cell(row, scale_column, "intensity_scale", INTENSITY_SCALE);
    match (value, scale) {
        (Some(value), Some(scale)) => Some((Some(value), Some(scale))),
        _ => None,
    }
}

fn read_flags(reading: &mut Reading<'_>, row: &Row, column: usize) -> Option<Vec<&'static str>> {
    let cell = row.cells[column].as_str();
    if cell.is_empty() {
        return Some(Vec::new());
    }
    let mut held: Vec<&'static str> = Vec::new();
    let mut refused = false;
    for part in cell.split(',') {
        match parse_vocabulary(part, LINE_FLAG) {
            Ok(flag) if held.contains(&flag) => {
                reading.refuse(Refusal::field_on_line(
                    row.line,
                    "flags",
                    format!("`{flag}` appears more than once"),
                ));
                refused = true;
            }
            Ok(flag) => held.push(flag),
            Err(reason) => {
                reading.refuse(Refusal::field_on_line(row.line, "flags", reason));
                refused = true;
            }
        }
    }
    if refused { None } else { Some(held) }
}
