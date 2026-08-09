//! The lines export of the NIST Atomic Spectra Database, issue #23.
//!
//! `docs/sources.md` carries the entry for this upstream, its terms and the
//! attribution an answer computed from it has to carry. This module carries the
//! parsing half and nothing else: it emits a conforming line list for
//! `spectro-contract` to read, and it decides nothing the records have not
//! already decided.
//!
//! What the export leaves implicit, and what is done about each.
//!
//! The medium is in the column label and the label changes inside one file. A
//! query spanning the regions emits its header row again where the medium
//! changes, and the second block is labelled `obs_wl_vac(nm)` where the first
//! was `obs_wl_air(nm)`. Retrieved on 2026-08-09:
//!
//! ```text
//! curl -sS -L -o a4.tsv 'https://physics.nist.gov/cgi-bin/ASD/lines1.pl?spectra=Ne+II&limits_type=0&low_w=50&upp_w=4000&unit=1&submit=Retrieve+Data&de=0&format=3&line_out=0&en_unit=0&output=0&bibrefs=1&page_size=15&show_obs_wl=1&show_calc_wl=1&unc_out=1&order_out=0&show_av=2&tsb_value=0&A_out=0&intens_out=on&allowed_out=1&forbid_out=1&conf_out=on&term_out=on&enrg_out=on&J_out=on'
//! grep -n 'obs_wl' a4.tsv | cut -d: -f1
//! 1
//! 230
//! 1616
//! ```
//!
//! Three header rows in one file, for one query. A reader that took the medium
//! from the first of them would label sixteen hundred rows `air_standard`, and
//! the ones it got wrong are the two ends of the spectrum rather than a
//! scattering. So every row this module emits states its own medium, taken from
//! the block it stood in, and the file default is never the thing a row falls
//! back on.
//!
//! The uncertainty is a standard deviation because the source says so, not
//! because it is the usual reading. From the help page, read 2026-08-09 with
//! `curl -sS -L https://physics.nist.gov/PhysRefData/ASD/Html/lineshelp.html`:
//! "All uncertainties given in ASD are meant to be on the level of one standard
//! deviation." An entry with no uncertainty gets `none` rather than a default,
//! which is `docs/decisions/uncertainty-model.md`'s rule.
//!
//! The accuracy grade does not grade the wavelength. The same page groups it
//! under "Transition Strengths, Accuracy" and offers it as an "Accuracy minimum
//! for Aki, fik, S, or log(gf)", so it rates the transition probability. A
//! conforming line list carries no transition probability, so the grade is
//! reported as not taken rather than written into `uncertainty_class`. Putting
//! it there would have attached a rating of one quantity to the uncertainty of
//! another, and every position it touched would have looked classed rather than
//! ungraded from that point on.
//!
//! A Ritz wavelength never becomes an observation. It is computed from the
//! level set this board is about to assign against, so an engine fed one is
//! measuring its own arithmetic. Where a row carries a Ritz wavelength and no
//! observed one, no line is emitted and the row is reported.
//!
//! Nothing here opens a socket or names a host. Retrieval is the operator's,
//! which `docs/data-on-the-host.md` requires and issue #25 turns into a check.

use spectro_contract::{Refusal, Refusals};

use crate::accuracy::AccuracyGrade;

/// The `contract_version` this module emits.
const CONTRACT_VERSION: &str = "1.0";

/// The column label row of a conforming line list, in the order emitted here.
const LABELS: &str = "feature_id\tspectrum_id\tsegment_id\tposition\tposition_medium\t\
                      position_uncertainty\tuncertainty_kind\tuncertainty_class\tintensity\t\
                      intensity_scale\tflags\tritz_position";

/// What the export does not say and the operator has to.
///
/// The export is one spectrum's worth of rows and names neither the spectrum
/// the operator recorded nor the file they are building, so both come from
/// outside rather than being invented from the query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Naming {
    pub spectrum_id: String,
    pub line_list_id: String,
}

/// One thing the export said that the conforming file does not carry.
///
/// Reported rather than dropped. A row silently missing from a line list is a
/// row missing from an answer that does not say so, which is the failure
/// `docs/decisions/input-contract.md` refuses a skipped comment row for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotTaken {
    /// The line of the export, counting from one.
    pub line: usize,
    /// The export's own label for the column this came out of.
    pub column: String,
    /// The cell, verbatim.
    pub value: String,
    pub reason: String,
}

impl std::fmt::Display for NotTaken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "line {}, `{}`: `{}` {}",
            self.line, self.column, self.value, self.reason
        )
    }
}

/// A conforming line list and everything the export held that it does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversion {
    pub line_list: String,
    pub not_taken: Vec<NotTaken>,
}

/// Convert an export into a conforming line list, or refuse naming the line.
pub fn convert(bytes: &[u8], naming: &Naming) -> Result<Conversion, Refusals> {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Err(Refusal::in_file("the export is not UTF-8").into());
    };

    let mut refusals = Refusals::new();
    check_naming(&mut refusals, "spectrum_id", &naming.spectrum_id);
    check_naming(&mut refusals, "line_list_id", &naming.line_list_id);

    let mut block: Option<Block> = None;
    let mut unit: Option<String> = None;
    let mut first_medium: Option<&'static str> = None;
    let mut rows: Vec<String> = Vec::new();
    let mut not_taken: Vec<NotTaken> = Vec::new();

    let pieces: Vec<&str> = text.split('\n').collect();
    let last = pieces.len() - 1;
    for (index, piece) in pieces.iter().copied().enumerate() {
        let line = index + 1;
        let content = piece.strip_suffix('\r').unwrap_or(piece);
        if content.is_empty() {
            if index != last {
                refusals.push(Refusal::on_line(
                    line,
                    "the export carries a line with no bytes, and a blank line and a row of empty cells cannot be told apart afterwards",
                ));
            }
            continue;
        }

        let cells: Vec<&str> = content.split('\t').collect();
        if is_header_row(&cells) {
            match Block::read(&cells, line) {
                Ok(read) => {
                    match &unit {
                        None => unit = Some(read.unit.clone()),
                        Some(held) if held != &read.unit => {
                            refusals.push(Refusal::field_on_line(
                                line,
                                "position_unit",
                                format!(
                                    "this block states positions in `{}` and an earlier one stated them in `{held}`, and a conforming line list has one unit for the file",
                                    read.unit
                                ),
                            ));
                        }
                        Some(_) => {}
                    }
                    first_medium.get_or_insert(read.medium);
                    block = Some(read);
                }
                Err(refusal) => {
                    refusals.push(refusal);
                    block = None;
                }
            }
            continue;
        }

        let Some(block) = block.as_ref() else {
            refusals.push(Refusal::on_line(
                line,
                "a data row stands before any header row, so nothing says which medium or unit its position is in",
            ));
            continue;
        };

        match block.row(&cells, line, naming, &mut not_taken) {
            Ok(Some(row)) => rows.push(row),
            Ok(None) => {}
            Err(found) => {
                for refusal in &found {
                    refusals.push(refusal.clone());
                }
            }
        }
    }

    let (Some(unit), Some(first_medium)) = (unit, first_medium) else {
        refusals.push(Refusal::in_file(
            "the export carries no header row naming an observed wavelength column, so no position in it has a medium or a unit",
        ));
        return refusals.into_result(Conversion {
            line_list: String::new(),
            not_taken,
        });
    };

    // The header default exists because the contract requires one. No row
    // relies on it: every row below states its own medium, which is what keeps
    // the region rule from collapsing into whichever block came first.
    let mut line_list = format!(
        "#contract_version\t{CONTRACT_VERSION}\n\
         #position_unit\t{unit}\n\
         #position_medium\t{first_medium}\n\
         #line_list_id\t{}\n{LABELS}\n",
        naming.line_list_id
    );
    for row in &rows {
        line_list.push_str(row);
        line_list.push('\n');
    }

    refusals.into_result(Conversion {
        line_list,
        not_taken,
    })
}

/// One header row and the columns it labels.
struct Block {
    medium: &'static str,
    unit: String,
    observed: usize,
    ritz: Option<usize>,
    uncertainty: Option<usize>,
    intensity: Option<usize>,
    accuracy: Option<(usize, String)>,
}

impl Block {
    fn read(cells: &[&str], line: usize) -> Result<Block, Refusal> {
        let Some(observed) = cells.iter().position(|cell| cell.starts_with("obs_wl_")) else {
            return Err(Refusal::field_on_line(
                line,
                "obs_wl",
                "this header row labels no observed wavelength column",
            ));
        };
        let (medium, unit) = medium_and_unit(cells[observed], "obs_wl_")
            .map_err(|reason| Refusal::field_on_line(line, "obs_wl", reason))?;

        let ritz = cells.iter().position(|cell| cell.starts_with("ritz_wl_"));
        if let Some(ritz) = ritz {
            let (ritz_medium, ritz_unit) = medium_and_unit(cells[ritz], "ritz_wl_")
                .map_err(|reason| Refusal::field_on_line(line, "ritz_wl", reason))?;
            if ritz_medium != medium || ritz_unit != unit {
                return Err(Refusal::field_on_line(
                    line,
                    "ritz_wl",
                    format!(
                        "the Ritz column of this block is `{ritz_medium}` in `{ritz_unit}` and the observed column is `{medium}` in `{unit}`"
                    ),
                ));
            }
        }

        Ok(Block {
            medium,
            unit,
            observed,
            ritz,
            uncertainty: cells.iter().position(|cell| *cell == "unc_obs_wl"),
            intensity: cells.iter().position(|cell| *cell == "intens"),
            accuracy: cells
                .iter()
                .position(|cell| *cell == "Acc")
                .map(|column| (column, cells[column].to_owned())),
        })
    }

    /// One data row, as a conforming line, or nothing where the export carried
    /// no observation to make one out of.
    fn row(
        &self,
        cells: &[&str],
        line: usize,
        naming: &Naming,
        not_taken: &mut Vec<NotTaken>,
    ) -> Result<Option<String>, Refusals> {
        let mut refusals = Refusals::new();

        let observed = unquote(self.cell(cells, Some(self.observed)));
        let ritz = self
            .ritz
            .map(|column| unquote(self.cell(cells, Some(column))));

        if observed.is_empty() {
            not_taken.push(NotTaken {
                line,
                column: "obs_wl".to_owned(),
                value: ritz.unwrap_or_default().to_owned(),
                reason:
                    "is a Ritz wavelength with no observation beside it, and a Ritz wavelength was computed from the level set this board assigns against"
                        .to_owned(),
            });
            return Ok(None);
        }
        if number(observed).is_none() {
            refusals.push(Refusal::field_on_line(
                line,
                "position",
                format!("`{observed}` is not a number this reader can carry as a position"),
            ));
        }

        // The source's own sentence decides the kind. An absent uncertainty is
        // an absence rather than a zero and rather than a default.
        let declared = unquote(self.cell(cells, self.uncertainty));
        let uncertainty = if declared.is_empty() {
            "none".to_owned()
        } else if number(declared).is_some() {
            declared.to_owned()
        } else {
            refusals.push(Refusal::field_on_line(
                line,
                "position_uncertainty",
                format!("`{declared}` is not a number, and an absent uncertainty is an empty cell"),
            ));
            "none".to_owned()
        };

        if let Some((column, label)) = &self.accuracy {
            let cell = unquote(self.cell(cells, Some(*column)));
            if !cell.is_empty() {
                let grade = AccuracyGrade::taken(cell);
                not_taken.push(NotTaken {
                    line,
                    column: label.clone(),
                    value: grade.as_str().to_owned(),
                    reason:
                        "grades a transition probability rather than a position, and a conforming line list carries neither the probability nor a grade"
                            .to_owned(),
                });
            }
        }

        let (intensity, scale) = self.intensity(cells, line, not_taken);

        let ritz_out = match ritz {
            Some("") => String::new(),
            Some(cell) if number(cell).is_some() => cell.to_owned(),
            Some(cell) => {
                refusals.push(Refusal::field_on_line(
                    line,
                    "ritz_position",
                    format!("`{cell}` is not a number this reader can carry as a Ritz position"),
                ));
                String::new()
            }
            None => String::new(),
        };

        if !refusals.is_empty() {
            return Err(refusals);
        }

        // The feature identifier is the export's own line number, so a row in
        // an answer can be taken back to the row it came from without a second
        // file mapping one to the other.
        Ok(Some(format!(
            "l{line}\t{spectrum}\t\t{observed}\t{medium}\t{uncertainty}\tstandard_deviation\t\t{intensity}\t{scale}\t\t{ritz_out}",
            spectrum = naming.spectrum_id,
            medium = self.medium,
        )))
    }

    /// The intensity, with the trailing character code the export appends taken
    /// off and reported rather than read.
    ///
    /// The split is lexical: the digits are the number and the letters are not.
    /// What the letters mean is not read here, and no flag is derived from one,
    /// because `docs/decisions/what-an-assignment-is.md` and the line flags of
    /// `docs/decisions/input-contract.md` are a vocabulary of three and mapping
    /// an upstream's character codes onto them is a modelling decision this
    /// module is not the place for.
    fn intensity(
        &self,
        cells: &[&str],
        line: usize,
        not_taken: &mut Vec<NotTaken>,
    ) -> (String, String) {
        let cell = unquote(self.cell(cells, self.intensity));
        if cell.is_empty() {
            return (String::new(), String::new());
        }
        let digits = cell
            .find(|character: char| !character.is_ascii_digit() && character != '.')
            .unwrap_or(cell.len());
        let (value, code) = cell.split_at(digits);
        if !code.is_empty() {
            not_taken.push(NotTaken {
                line,
                column: "intens".to_owned(),
                value: code.to_owned(),
                reason:
                    "is the character code the export appends to an intensity, and what it means is not read here"
                        .to_owned(),
            });
        }
        if value.is_empty() || number(value).is_none() {
            not_taken.push(NotTaken {
                line,
                column: "intens".to_owned(),
                value: cell.to_owned(),
                reason: "holds no number in front of its character code".to_owned(),
            });
            return (String::new(), String::new());
        }
        // The source labels no scale, so the scale is `arbitrary` and source
        // dependent. Saying nothing is what the reader refuses; saying more
        // than the source said is worse.
        (value.to_owned(), "arbitrary".to_owned())
    }

    fn cell<'a>(&self, cells: &[&'a str], column: Option<usize>) -> &'a str {
        column
            .and_then(|column| cells.get(column).copied())
            .unwrap_or("")
    }
}

/// `obs_wl_air(nm)` into the contract's medium and the file's unit.
fn medium_and_unit(label: &str, prefix: &str) -> Result<(&'static str, String), String> {
    let rest = label
        .strip_prefix(prefix)
        .ok_or_else(|| format!("`{label}` does not begin `{prefix}`"))?;
    let (tag, unit) = rest
        .split_once('(')
        .ok_or_else(|| format!("`{label}` names no unit in brackets"))?;
    let unit = unit
        .strip_suffix(')')
        .ok_or_else(|| format!("`{label}` does not close the bracket around its unit"))?;
    if unit.is_empty() {
        return Err(format!("`{label}` brackets an empty unit"));
    }
    let medium = match tag {
        "air" => "air_standard",
        "vac" => "vacuum",
        _ => {
            return Err(format!(
                "`{label}` names the medium `{tag}`, which is neither `air` nor `vac`"
            ));
        }
    };
    Ok((medium, unit.to_owned()))
}

/// The export wraps most cells in quotation marks and leaves some bare.
fn unquote(cell: &str) -> &str {
    cell.strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(cell)
}

/// Whether a cell holds a finite number, without saying what it is worth.
fn number(cell: &str) -> Option<f64> {
    cell.parse::<f64>().ok().filter(|value| value.is_finite())
}

fn is_header_row(cells: &[&str]) -> bool {
    cells.iter().any(|cell| cell.starts_with("obs_wl_"))
}

/// What the operator supplied has to survive being written into a tab-separated
/// file with a `#` header block, or the file it lands in is not the file they
/// described.
fn check_naming(refusals: &mut Refusals, field: &str, value: &str) {
    if value.is_empty() {
        refusals.push(Refusal::absent_field(field, "is empty"));
        return;
    }
    if value.contains('\t') || value.contains('\n') || value.contains('\r') {
        refusals.push(Refusal::absent_field(
            field,
            format!("`{value}` carries a tab or a line ending, which a cell may not"),
        ));
    }
}
