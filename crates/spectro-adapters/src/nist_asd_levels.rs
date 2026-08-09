//! The levels export of the NIST Atomic Spectra Database, issue #22.
//!
//! `docs/sources.md` carries the entry for this upstream, its terms and the
//! attribution an answer computed from it has to carry. This module carries the
//! parsing half and nothing else: it emits a conforming level set for
//! `spectro-contract` to read, and it decides nothing the records have not
//! already decided.
//!
//! Every column is found by its label and never by its position, and that is a
//! measurement rather than a habit. One query string, changed only in the
//! species, does not answer with one column layout. Retrieved on 2026-08-09:
//!
//! ```text
//! curl -sS -L -o ne2.tsv 'https://physics.nist.gov/cgi-bin/ASD/energy1.pl?de=0&spectrum=Ne+II&units=0&format=3&output=0&page_size=15&multiplet_ordered=0&conf_out=on&term_out=on&level_out=on&unc_out=1&j_out=on&g_out=on&lande_out=on&perc_out=on&biblio=on&temp=&submit=Retrieve+Data'
//! head -1 ne2.tsv | tr '\t' '|'
//! Configuration|Term|J|g|Prefix|Level (cm-1)|Suffix|Uncertainty (cm-1)|Leading percentages|Reference
//! ```
//!
//! The same query for `Nd+II` answers with an eleventh column, `Lande`, sitting
//! after `Uncertainty (cm-1)`, so every column past that point moves one place
//! along. A reader holding column numbers would read a bibliography code as a
//! leading percentage for every spectrum that carries a measured Lande factor,
//! and it would do so without failing.
//!
//! ## What the export does not say, and who says it instead
//!
//! The species. A levels export is one spectrum's worth of rows, no row names
//! its species, and the form refuses to be asked for two at once. Retrieved on
//! 2026-08-09 with `spectrum=Ne+II;Ne+III`, the answer is a page carrying:
//!
//! ```text
//! Selection of multiple spectra is not allowed for the Energy Level Search.
//! ```
//!
//! So one export is one species, the species is what the operator asked for,
//! and it arrives beside the file rather than out of it. An adapter taking the
//! species from a file name would be guessing at the one field that decides
//! which levels may combine at all.
//!
//! The reference point. The source states it once, for every export, on the
//! help page read on 2026-08-09 with
//! `curl -sS -L https://physics.nist.gov/PhysRefData/ASD/Html/levelshelp.html`:
//! "The levels are normally given as they are stored in the database, in units
//! of cm-1, with respect to the ground level at zero cm-1." That sentence is
//! what `energy_reference` is emitted from, and the word "normally" in it is
//! the case handled below rather than ignored.
//!
//! ## What is skipped, and why each one is not a level of this species
//!
//! An ionisation limit. The same help page: the ionisation energy is followed
//! by "the configuration and term designations and the J value for the ground
//! level of this next ion. The word 'Limit' appears in the 'Term' column". A
//! limit row is a level of the next ion, so emitting it into a level set for
//! this species would hand the enumerator of issue #26 a level that cannot
//! produce any transition it is looking for, under this species' name.
//!
//! A level whose connection to the rest of the spectrum has not been made. The
//! help page: certain level values are followed by "+x", extended to "+y" and
//! "+z" for further such systems, and "no experimental connection between this
//! system and the other levels of the spectrum has been made". Those levels
//! carry a real position with respect to each other and an unknown offset with
//! respect to the ground level, so they are not on the reference the file
//! declares, and writing one in as though it were is the plausible wrong number
//! this board is built against. Every such row is reported instead. One was
//! found in the Nd I export retrieved on 2026-08-09, in the `Suffix` column.
//!
//! A row with no level value at all. The help page describes a blank level
//! beside a J as a missing level of a term. Nine of those stand in the Ce I
//! export and twenty-nine in Nd I, both retrieved on 2026-08-09, so this is
//! ordinary rather than a malformed file. There is no energy to emit and the
//! row is reported.
//!
//! ## What is read from what, and the bound on each reading
//!
//! Whether a level was measured or calculated is read from the `Prefix` column
//! and from nothing else. The help page: square brackets are "energies
//! determined by interpolation, extrapolation, or other semi-empirical
//! procedure relying on some known experimental values", and parentheses are
//! "energies determined from ab-initio calculation or by other means not
//! involving evaluated experimental data". Both are `predicted`; a bare value
//! is `measured`. A prefix this module does not know is refused rather than
//! read as a bare one, because the failure of a wrong guess here is a
//! calculated energy entering the objective as a measurement.
//!
//! Parity is read from the asterisk. The help page: odd parity is indicated
//! "in the ASCII output by an asterisk (*) at the end of the term label". A row
//! with no term label at all therefore states no parity and gets `unknown`,
//! which is the fact about that level rather than about the file, and the file
//! is refused outright where the export carries no term column, below.
//!
//! Every uncertainty is a standard deviation because the source says so, on the
//! same page: "All uncertainties given in ASD are meant to be on the level of
//! one standard deviation." An empty uncertainty cell becomes `none` and never
//! a default. The help page offers a rule of thumb for that case, that the
//! probable error is usually between 2.5 and 25 units in the last decimal
//! place, and turning a rule of thumb into a number in an input file is exactly
//! what `docs/decisions/uncertainty-model.md` refuses, so it is not used here.
//!
//! ## What the export carries that a conforming level set has no field for
//!
//! Reported rather than dropped, through the same type the line adapter
//! reports with. A J given as several possible values, which the help page
//! describes as ordinary for an uncertain assignment, cannot become one J, and
//! choosing one of them would be an invention, so the level is emitted with
//! `unknown` and the cell is reported verbatim. A suffix that is neither the
//! closing half of a bracket nor a plus notation is reported the same way: the
//! `a` of substantial autoionisation broadening and the `?` of a level that may
//! not be real are both statements a conforming level set has nowhere to put.
//!
//! Nothing here opens a socket or names a host. Retrieval is the operator's,
//! which `docs/data-on-the-host.md` requires and issue #25 turns into a check.

use spectro_contract::{HalfInteger, Refusal, Refusals};

use crate::nist_asd_lines::NotTaken;

/// The `contract_version` this module emits.
const CONTRACT_VERSION: &str = "1.0";

/// The column label row of a conforming level set, in the order emitted here.
const LABELS: &str = "level_id\tspecies\tenergy\tenergy_uncertainty\tuncertainty_kind\t\
                      uncertainty_class\tparity\tj\torigin\tconfiguration\tterm";

/// What `energy_reference` carries, from the sentence quoted above.
const ENERGY_REFERENCE: &str = "ground_level";

/// What the export does not say and the operator has to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Naming {
    pub species: String,
    pub level_set_id: String,
}

/// Where one field of the contract's level set comes from, for this upstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A column of the export, under the label given.
    Column(&'static str),
    /// The operator supplies it, because the export does not carry it.
    Operator,
    /// Fixed here, for the reason given, from something the source states about
    /// every export it emits rather than from a cell.
    Fixed(&'static str),
    /// The column is emitted because the contract requires the column, and this
    /// upstream never fills it.
    Empty(&'static str),
    /// This upstream does not give it and nothing here invents one, so the
    /// emitted file does not carry the field at all.
    Unavailable(&'static str),
}

/// Every field of `docs/decisions/input-contract.md`'s level set, and what this
/// upstream supplies for each one.
///
/// This is the "either mapped or explicitly reported as unavailable" half of
/// issue #22's Done-when, held as data so that a test can compare it against
/// the file this module emits rather than against a list somebody reads. What
/// the test can and cannot derive is written where the test makes the
/// comparison.
pub const FIELDS: &[(&str, Source)] = &[
    (
        "contract_version",
        Source::Fixed("the contract version this module emits"),
    ),
    ("energy_unit", Source::Column("Level (<unit>)")),
    (
        "energy_reference",
        Source::Fixed(
            "the source states that its levels stand with respect to the ground level at zero",
        ),
    ),
    ("level_set_id", Source::Operator),
    (
        "covariance_file",
        Source::Unavailable(
            "the export carries no covariance between level energies, so every run built on it falls to the declared correlation default of docs/decisions/uncertainty-model.md",
        ),
    ),
    (
        "derived_from_line_lists",
        Source::Unavailable(
            "the export does not say which line list the fit that produced these levels consumed, so the circularity disclosure cannot be made from it",
        ),
    ),
    (
        "level_id",
        Source::Fixed("the line of the export the level was read from"),
    ),
    ("species", Source::Operator),
    ("energy", Source::Column("Level (<unit>)")),
    ("energy_uncertainty", Source::Column("Uncertainty (<unit>)")),
    (
        "uncertainty_kind",
        Source::Fixed(
            "the source states that every uncertainty it gives is one standard deviation",
        ),
    ),
    (
        "uncertainty_class",
        Source::Empty(
            "this upstream states its level uncertainties as numbers and grades none of them, so no row carries a class",
        ),
    ),
    ("parity", Source::Column("Term")),
    ("j", Source::Column("J")),
    ("origin", Source::Column("Prefix")),
    ("configuration", Source::Column("Configuration")),
    ("term", Source::Column("Term")),
];

/// A conforming level set and everything the export held that it does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversion {
    pub level_set: String,
    pub not_taken: Vec<NotTaken>,
}

/// Convert an export into a conforming level set, or refuse naming the line.
pub fn convert(bytes: &[u8], naming: &Naming) -> Result<Conversion, Refusals> {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Err(Refusal::in_file("the export is not UTF-8").into());
    };

    let mut refusals = Refusals::new();
    check_naming(&mut refusals, "species", &naming.species);
    check_naming(&mut refusals, "level_set_id", &naming.level_set_id);

    let mut columns: Option<Columns> = None;
    // A header row that was refused is still a header row. Without this the one
    // fault in it is reported again by every data row standing under it, as a
    // row that arrived before any header, and a producer fixing one label reads
    // a page of refusals about something else.
    let mut header_seen = false;
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
            if header_seen {
                refusals.push(Refusal::on_line(
                    line,
                    "the export carries a second header row, and a levels export answers one query about one species",
                ));
                continue;
            }
            header_seen = true;
            match Columns::read(&cells, line) {
                Ok(read) => columns = Some(read),
                Err(found) => {
                    for refusal in &found {
                        refusals.push(refusal.clone());
                    }
                }
            }
            continue;
        }

        let Some(columns) = columns.as_ref() else {
            if !header_seen {
                refusals.push(Refusal::on_line(
                    line,
                    "a data row stands before any header row, so nothing says which column holds a level value",
                ));
            }
            continue;
        };

        match columns.row(&cells, line, naming, &mut not_taken) {
            Ok(Some(row)) => rows.push(row),
            Ok(None) => {}
            Err(found) => {
                for refusal in &found {
                    refusals.push(refusal.clone());
                }
            }
        }
    }

    let Some(columns) = columns else {
        if !header_seen {
            refusals.push(Refusal::in_file(
                "the export carries no header row naming a level column, so nothing in it has an energy or a unit",
            ));
        }
        return refusals.into_result(Conversion {
            level_set: String::new(),
            not_taken,
        });
    };

    let mut level_set = format!(
        "#contract_version\t{CONTRACT_VERSION}\n\
         #energy_unit\t{unit}\n\
         #energy_reference\t{ENERGY_REFERENCE}\n\
         #level_set_id\t{id}\n{LABELS}\n",
        unit = columns.unit,
        id = naming.level_set_id,
    );
    for row in &rows {
        level_set.push_str(row);
        level_set.push('\n');
    }

    refusals.into_result(Conversion {
        level_set,
        not_taken,
    })
}

/// The header row, read as a set of labelled columns.
struct Columns {
    unit: String,
    width: usize,
    configuration: usize,
    term: usize,
    j: usize,
    prefix: usize,
    suffix: usize,
    level: usize,
    uncertainty: Option<usize>,
}

impl Columns {
    fn read(cells: &[&str], line: usize) -> Result<Columns, Refusals> {
        let mut refusals = Refusals::new();

        let level = cells.iter().position(|cell| cell.starts_with("Level ("));
        let unit = match level {
            Some(column) => match unit_in_brackets(cells[column]) {
                Ok(unit) => Some(unit),
                Err(reason) => {
                    refusals.push(Refusal::field_on_line(line, "Level", reason));
                    None
                }
            },
            None => {
                refusals.push(Refusal::field_on_line(
                    line,
                    "Level",
                    "this header row labels no level column, so the export was retrieved without level values",
                ));
                None
            }
        };

        // Every one of these is refused rather than defaulted, and each refusal
        // says what the emitted file would otherwise have claimed. The contract
        // keeps an absent column and an `unknown` value apart on purpose, so an
        // export retrieved without one of these columns cannot become a level
        // set carrying `unknown` in its place: that would state a fact about
        // the spectrum out of a fact about the query.
        let term = required(
            &mut refusals,
            cells,
            line,
            "Term",
            "so no row would state a parity, and every level would arrive as though its parity had never been determined",
        );
        let j = required(
            &mut refusals,
            cells,
            line,
            "J",
            "and a level set with no J column is refused by the contract rather than read",
        );
        let prefix = required(
            &mut refusals,
            cells,
            line,
            "Prefix",
            "so nothing would separate a calculated energy from a measured one, and every level would arrive as measured",
        );
        let suffix = required(
            &mut refusals,
            cells,
            line,
            "Suffix",
            "so a level standing on an unestablished connection to the rest of the spectrum could not be told from one on the file's own reference",
        );
        let configuration = required(
            &mut refusals,
            cells,
            line,
            "Configuration",
            "and an empty configuration is a level the source gave no label for, which is not the same statement",
        );

        let uncertainty = cells
            .iter()
            .position(|cell| cell.starts_with("Uncertainty ("));
        if let (Some(column), Some(unit)) = (uncertainty, unit.as_ref()) {
            match unit_in_brackets(cells[column]) {
                Ok(found) if &found == unit => {}
                Ok(found) => refusals.push(Refusal::field_on_line(
                    line,
                    "Uncertainty",
                    format!(
                        "the uncertainty column is in `{found}` and the level column is in `{unit}`, and an uncertainty in another unit is not this level's uncertainty"
                    ),
                )),
                Err(reason) => refusals.push(Refusal::field_on_line(line, "Uncertainty", reason)),
            }
        }

        let (
            Some(unit),
            Some(level),
            Some(term),
            Some(j),
            Some(prefix),
            Some(suffix),
            Some(configuration),
        ) = (unit, level, term, j, prefix, suffix, configuration)
        else {
            return Err(refusals);
        };

        refusals.into_result(Columns {
            unit,
            width: cells.len(),
            configuration,
            term,
            j,
            prefix,
            suffix,
            level,
            uncertainty,
        })
    }

    /// One data row, as a conforming line, or nothing where the export carried
    /// no level of this species to make one out of.
    fn row(
        &self,
        cells: &[&str],
        line: usize,
        naming: &Naming,
        not_taken: &mut Vec<NotTaken>,
    ) -> Result<Option<String>, Refusals> {
        let mut refusals = Refusals::new();

        if cells.len() != self.width {
            return Err(Refusal::on_line(
                line,
                format!(
                    "this row carries {} cell(s) and the header row labels {}, so nothing says which column any of them is in",
                    cells.len(),
                    self.width
                ),
            )
            .into());
        }

        let term = self.cell(cells, Some(self.term));
        let suffix = self.cell(cells, Some(self.suffix));
        let level = self.cell(cells, Some(self.level));

        if term == "Limit" {
            not_taken.push(NotTaken {
                line,
                column: "Term".to_owned(),
                value: level.to_owned(),
                reason:
                    "is an ionisation limit, which the source gives as the ground level of the next ion rather than as a level of this spectrum"
                        .to_owned(),
            });
            return Ok(None);
        }

        if suffix.contains('+') {
            not_taken.push(NotTaken {
                line,
                column: "Suffix".to_owned(),
                value: format!("{level}{suffix}"),
                reason:
                    "stands on a system of levels the source states has no established connection to the rest of the spectrum, so its value is not on the reference this file declares"
                        .to_owned(),
            });
            return Ok(None);
        }

        if level.is_empty() {
            not_taken.push(NotTaken {
                line,
                column: "Level".to_owned(),
                value: term.to_owned(),
                reason: "is a term the source lists without a level value".to_owned(),
            });
            return Ok(None);
        }

        if number(level).is_none() {
            refusals.push(Refusal::field_on_line(
                line,
                "energy",
                format!("`{level}` is not a number this reader can carry as a level energy"),
            ));
        }

        let origin = match self.cell(cells, Some(self.prefix)) {
            "" => "measured",
            "[" | "(" => "predicted",
            other => {
                refusals.push(Refusal::field_on_line(
                    line,
                    "origin",
                    format!(
                        "`{other}` is a level prefix this reader does not know, and reading it as a bare value would put a calculated energy into the objective as a measurement"
                    ),
                ));
                "measured"
            }
        };

        // The closing half of a bracket says what the prefix already said. Any
        // other annotation is the source stating something a conforming level
        // set has no field for, so it is reported rather than interpreted.
        if !matches!(suffix, "" | "]" | ")") {
            not_taken.push(NotTaken {
                line,
                column: "Suffix".to_owned(),
                value: suffix.to_owned(),
                reason:
                    "is an annotation the source appends to a level value, and a conforming level set carries no field for one"
                        .to_owned(),
            });
        }

        let declared = self.cell(cells, self.uncertainty);
        let uncertainty = if declared.is_empty() {
            "none".to_owned()
        } else if number(declared).is_some() {
            declared.to_owned()
        } else {
            refusals.push(Refusal::field_on_line(
                line,
                "energy_uncertainty",
                format!("`{declared}` is not a number, and an absent uncertainty is an empty cell"),
            ));
            "none".to_owned()
        };

        let j = self.cell(cells, Some(self.j));
        let j = if j.is_empty() {
            "unknown".to_owned()
        } else if HalfInteger::parse(j).is_ok() {
            j.to_owned()
        } else {
            not_taken.push(NotTaken {
                line,
                column: "J".to_owned(),
                value: j.to_owned(),
                reason:
                    "names no single J, and choosing one of the values the source offers would be this module inventing the field"
                        .to_owned(),
            });
            "unknown".to_owned()
        };

        let parity = match term.trim_end_matches('?') {
            "" => "unknown",
            labelled if labelled.ends_with('*') => "odd",
            _ => "even",
        };

        let configuration = self.cell(cells, Some(self.configuration));
        for (field, value) in [("configuration", configuration), ("term", term)] {
            if value.contains('\t') || value.contains('\r') {
                refusals.push(Refusal::field_on_line(
                    line,
                    field,
                    format!("`{value}` carries a tab or a carriage return, which a cell may not"),
                ));
            }
        }

        if !refusals.is_empty() {
            return Err(refusals);
        }

        // The level identifier is the export's own line number, so a level in
        // an answer can be taken back to the row it came from without a second
        // file mapping one to the other.
        Ok(Some(format!(
            "l{line}\t{species}\t{level}\t{uncertainty}\tstandard_deviation\t\t{parity}\t{j}\t{origin}\t{configuration}\t{term}",
            species = naming.species,
        )))
    }

    /// A cell, unquoted and with the padding an export writes taken off.
    ///
    /// The contract refuses a padded number and says why: padding a column to a
    /// width is a habit of exports rather than of this format, and stripping it
    /// is what an adapter is for. This is that stripping, and it is the only
    /// place a cell of this export is altered on its way through.
    fn cell<'a>(&self, cells: &[&'a str], column: Option<usize>) -> &'a str {
        column
            .and_then(|column| cells.get(column).copied())
            .map(|cell| unquote(cell).trim())
            .unwrap_or("")
    }
}

/// One required column, or a refusal naming it and what its absence would cost.
fn required(
    refusals: &mut Refusals,
    cells: &[&str],
    line: usize,
    label: &str,
    cost: &str,
) -> Option<usize> {
    match cells.iter().position(|cell| *cell == label) {
        Some(column) => Some(column),
        None => {
            refusals.push(Refusal::field_on_line(
                line,
                label,
                format!("the export carries no `{label}` column, {cost}"),
            ));
            None
        }
    }
}

/// `Level (cm-1)` into the unit the file states its energies in.
fn unit_in_brackets(label: &str) -> Result<String, String> {
    let (_, unit) = label
        .split_once('(')
        .ok_or_else(|| format!("`{label}` names no unit in brackets"))?;
    let unit = unit
        .strip_suffix(')')
        .ok_or_else(|| format!("`{label}` does not close the bracket around its unit"))?;
    if unit.is_empty() {
        return Err(format!("`{label}` brackets an empty unit"));
    }
    Ok(unit.to_owned())
}

/// The export wraps most cells in quotation marks and leaves some bare.
///
/// Held here rather than shared with the line export module, so that a change
/// to either upstream's reader touches one file. Folding the two together is a
/// change to both and is not made by the change that landed this one.
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
    cells.iter().any(|cell| cell.starts_with("Level ("))
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
