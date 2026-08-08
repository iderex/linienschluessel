//! The readers and the validator for `docs/decisions/input-contract.md`.
//!
//! `docs/decisions/layout.md` puts the readers and the validator in this crate
//! and says that nothing else reads an input file.
//!
//! The validator is the readers. Every `validate_` function below reads the
//! file with the same code a run reads it with and then throws the result away,
//! so a file the validator accepts cannot be a file a reader afterwards
//! rejects. A validator written beside the readers would diverge from them, and
//! the day it did, a producer who had checked their emitter against it would be
//! worse off than one who had never run it.
//!
//! Nothing here opens a socket, reads an environment variable or names a host.
//! A conforming input is a file, which `docs/decisions/input-contract.md` fixes
//! and `docs/data-on-the-host.md` relies on.
//!
//! What the record leaves to this crate, and what is therefore owed back to it.
//! The record fixes the medium, every field of the level set and the line list,
//! and what the validator refuses. It does not fix the lexical form of a file,
//! the number a first `contract_version` carries, or the column labels of the
//! rate table and the covariance companion. Each of those is decided here,
//! beside the code that acts on it, and each is marked where it is decided:
//! [`document`] for the lexical form, [`value::SUPPORTED_MAJOR`] for the
//! version, and [`rates`] and [`covariance`] for their labels.

pub mod covariance;
pub mod document;
pub mod level_set;
pub mod line_list;
pub mod rates;
mod reading;
pub mod refusal;
pub mod value;

pub use covariance::Covariance;
pub use document::Document;
pub use level_set::{Level, LevelSet};
pub use line_list::{Line, LineList};
pub use rates::{Rate, RateTable};
pub use refusal::{Refusal, Refusals};
pub use value::{HalfInteger, SUPPORTED_MAJOR, Version};

/// Accept a conforming level set, or refuse naming the field and the line.
pub fn validate_level_set(bytes: &[u8]) -> Result<(), Refusals> {
    level_set::read(bytes).map(|_| ())
}

/// Accept a conforming line list, or refuse naming the field and the line.
pub fn validate_line_list(bytes: &[u8]) -> Result<(), Refusals> {
    line_list::read(bytes).map(|_| ())
}

/// Accept a conforming rate table, or refuse naming the field and the line.
pub fn validate_rate_table(bytes: &[u8]) -> Result<(), Refusals> {
    rates::read(bytes).map(|_| ())
}

/// Accept a conforming covariance companion, or refuse naming field and line.
pub fn validate_covariance(bytes: &[u8]) -> Result<(), Refusals> {
    covariance::read(bytes).map(|_| ())
}

/// Accept a level set together with the companions stated against it.
///
/// The cross-file checks live here rather than in either reader because
/// neither file can make them alone: a `level_id` in a companion is only wrong
/// against a level set that does not carry it.
pub fn validate_input(
    level_set_bytes: &[u8],
    covariance_bytes: Option<&[u8]>,
    rate_table_bytes: Option<&[u8]>,
) -> Result<(), Refusals> {
    let levels = level_set::read(level_set_bytes)?;
    let mut refusals = Refusals::new();

    match (levels.covariance_file.as_deref(), covariance_bytes) {
        (Some(named), None) => refusals.push(Refusal::absent_field(
            "covariance_file",
            format!("the level set names `{named}` and no companion was offered with it"),
        )),
        (None, Some(_)) => refusals.push(Refusal::absent_field(
            "covariance_file",
            "a covariance companion was offered and the level set names none",
        )),
        (Some(_), Some(bytes)) => match covariance::read(bytes) {
            Ok(companion) => {
                if let Err(found) = companion.check_against(&levels) {
                    for refusal in &found {
                        refusals.push(refusal.clone());
                    }
                }
            }
            Err(found) => {
                for refusal in &found {
                    refusals.push(refusal.clone());
                }
            }
        },
        (None, None) => {}
    }

    if let Some(bytes) = rate_table_bytes {
        match rates::read(bytes) {
            Ok(table) => {
                if let Err(found) = table.check_against(&levels) {
                    for refusal in &found {
                        refusals.push(refusal.clone());
                    }
                }
            }
            Err(found) => {
                for refusal in &found {
                    refusals.push(refusal.clone());
                }
            }
        }
    }

    refusals.into_result(())
}
