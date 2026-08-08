//! The values a cell or a header may carry, and what each one refuses.
//!
//! Every parse here refuses rather than repairs. `docs/decisions/input-contract.md`
//! is built against the plausible wrong number, so a value that is nearly right
//! is refused with the same words as one that is nothing like it.

use std::fmt;

/// The contract major version this code was written for.
///
/// The record fixes that `contract_version` carries a major and a minor number
/// and the rule for changing each. It names no number, so the number is fixed
/// here and is owed back to the record.
pub const SUPPORTED_MAJOR: u32 = 1;

/// The `contract_version` of one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl Version {
    /// Read `major.minor`, refusing anything else.
    pub fn parse(text: &str) -> Result<Version, String> {
        let Some((major, minor)) = text.split_once('.') else {
            return Err(format!(
                "`{text}` is not a major and a minor number separated by a full stop"
            ));
        };
        let major = parse_component(major)?;
        let minor = parse_component(minor)?;
        Ok(Version { major, minor })
    }

    /// Whether a reader written for [`SUPPORTED_MAJOR`] may read this file.
    ///
    /// A later minor is readable, which is the whole point of the minor rule: a
    /// minor increment adds optional fields and vocabulary members, and both
    /// are reported rather than dropped. A different major is not, and the
    /// refusal names the version found and the versions known, because a best
    /// effort read of a file whose fields have changed meaning is the plausible
    /// wrong number this board is built to avoid.
    pub fn is_readable(&self) -> bool {
        self.major == SUPPORTED_MAJOR
    }
}

fn parse_component(text: &str) -> Result<u32, String> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("`{text}` is not a number"));
    }
    text.parse::<u32>()
        .map_err(|_| format!("`{text}` does not fit in a version number"))
}

/// A non-negative half-integer, held as twice its value so that comparison and
/// equality are exact. A J of 3/2 is `twice = 3`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HalfInteger {
    twice: u32,
}

impl HalfInteger {
    pub fn twice(&self) -> u32 {
        self.twice
    }

    pub fn is_half_odd(&self) -> bool {
        self.twice % 2 == 1
    }

    /// Read a non-negative half-integer in either spelling a spectroscopist
    /// writes: `3/2` or `1.5`, and `2` or `2.0` for a whole one.
    ///
    /// Both spellings are accepted because both are what upstreams emit, and
    /// refusing one would push the choice into every adapter. Anything that is
    /// not an exact multiple of one half is refused rather than rounded.
    pub fn parse(text: &str) -> Result<HalfInteger, String> {
        if let Some((numerator, denominator)) = text.split_once('/') {
            if denominator != "2" {
                return Err(format!(
                    "`{text}` is not a half-integer: the only denominator a half-integer has is 2"
                ));
            }
            let numerator = parse_component(numerator)?;
            if numerator % 2 == 0 {
                return Err(format!(
                    "`{text}` is a whole number written as a fraction, and a whole J is written without one"
                ));
            }
            return Ok(HalfInteger { twice: numerator });
        }
        if let Some((whole, fraction)) = text.split_once('.') {
            let whole = parse_component(whole)?;
            let twice = match fraction {
                "0" => whole.checked_mul(2),
                "5" => whole.checked_mul(2).and_then(|held| held.checked_add(1)),
                _ => {
                    return Err(format!(
                        "`{text}` is not a multiple of one half, and a J is never rounded to one"
                    ));
                }
            };
            return twice
                .map(|twice| HalfInteger { twice })
                .ok_or_else(|| format!("`{text}` is larger than a J this reader can hold"));
        }
        let whole = parse_component(text)?;
        whole
            .checked_mul(2)
            .map(|twice| HalfInteger { twice })
            .ok_or_else(|| format!("`{text}` is larger than a J this reader can hold"))
    }
}

impl fmt::Display for HalfInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_half_odd() {
            write!(f, "{}/2", self.twice)
        } else {
            write!(f, "{}", self.twice / 2)
        }
    }
}

/// Read a finite number, with no leniency about what surrounds it.
///
/// No cell is trimmed, so ` 1.0` is refused here rather than parsed, and the
/// message says which of the two happened. A non-finite value is refused
/// because there is no position, energy or rate that an infinity states.
pub fn parse_number(text: &str) -> Result<f64, String> {
    if text != text.trim() {
        return Err(format!(
            "`{text}` is padded with whitespace, and no cell is trimmed before it is read"
        ));
    }
    match text.parse::<f64>() {
        Ok(number) if number.is_finite() => Ok(number),
        Ok(_) => Err(format!("`{text}` is not a finite number")),
        Err(_) => Err(format!("`{text}` is not a number")),
    }
}

/// Read a value against a closed vocabulary, naming the vocabulary on refusal.
pub fn parse_vocabulary<'a>(text: &str, vocabulary: &[&'a str]) -> Result<&'a str, String> {
    vocabulary
        .iter()
        .copied()
        .find(|member| *member == text)
        .ok_or_else(|| {
            format!(
                "`{text}` is not one of {}",
                vocabulary
                    .iter()
                    .map(|member| format!("`{member}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}

/// Whether a level or a line carries a declared parity.
pub const PARITY: &[&str] = &["even", "odd", "unknown"];

/// How an uncertainty was stated, which decides the distribution it becomes.
pub const UNCERTAINTY_KIND: &[&str] = &["standard_deviation", "spread", "class"];

/// Whether a level was measured or predicted.
pub const ORIGIN: &[&str] = &["measured", "predicted"];

/// The medium a position is stated in.
pub const POSITION_MEDIUM: &[&str] = &["vacuum", "air_standard"];

/// The scale an intensity is on, which the branching constraint reads.
pub const INTENSITY_SCALE: &[&str] = &["arbitrary", "photographic", "photon", "energy"];

/// The conditions under which the branching constraint does not apply.
pub const LINE_FLAG: &[&str] = &["saturated", "self_absorbed", "blended"];
