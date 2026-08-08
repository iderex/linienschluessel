//! What a refusal is, and why it always carries a place.
//!
//! `docs/decisions/input-contract.md` asks the validator to refuse "naming the
//! field and the line". A refusal that says only that a file is invalid sends
//! the producer back to read their own file against a document, which is the
//! work the validator exists to do for them. So the place is part of the type
//! rather than part of the message, and a refusal cannot be constructed without
//! deciding what it is about.

use std::fmt;

/// One reason one file was refused, with the place it was refused at.
///
/// `line` is absent only where the fault has no line: an absent header field
/// and an absent column are facts about the whole file. `field` is absent only
/// where the fault is the shape of a line rather than the value in a field,
/// because a row with the wrong number of cells has not omitted its last field,
/// it has moved all of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub line: Option<usize>,
    pub field: Option<String>,
    pub reason: String,
}

impl Refusal {
    /// A value in a named field on a numbered line.
    pub fn field_on_line(line: usize, field: &str, reason: impl Into<String>) -> Self {
        Refusal {
            line: Some(line),
            field: Some(field.to_owned()),
            reason: reason.into(),
        }
    }

    /// The shape of a numbered line.
    pub fn on_line(line: usize, reason: impl Into<String>) -> Self {
        Refusal {
            line: Some(line),
            field: None,
            reason: reason.into(),
        }
    }

    /// A named field the file never carried, so there is no line to name.
    pub fn absent_field(field: &str, reason: impl Into<String>) -> Self {
        Refusal {
            line: None,
            field: Some(field.to_owned()),
            reason: reason.into(),
        }
    }

    /// A fault of the file as a whole.
    pub fn in_file(reason: impl Into<String>) -> Self {
        Refusal {
            line: None,
            field: None,
            reason: reason.into(),
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, &self.field) {
            (Some(line), Some(field)) => write!(f, "line {line}, field `{field}`: {}", self.reason),
            (Some(line), None) => write!(f, "line {line}: {}", self.reason),
            (None, Some(field)) => write!(f, "field `{field}`: {}", self.reason),
            (None, None) => write!(f, "{}", self.reason),
        }
    }
}

/// Every reason one file was refused.
///
/// The validator collects rather than stopping at the first fault. A producer
/// fixing an emitter wants the list, and a validator that reports one fault per
/// run turns a file with eleven faults into eleven runs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Refusals(Vec<Refusal>);

impl Refusals {
    pub fn new() -> Self {
        Refusals(Vec::new())
    }

    pub fn push(&mut self, refusal: Refusal) {
        self.0.push(refusal);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Refusal> {
        self.0.iter()
    }

    /// Hand back `value` where nothing was refused, and the refusals otherwise.
    pub fn into_result<T>(self, value: T) -> Result<T, Refusals> {
        if self.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}

impl From<Refusal> for Refusals {
    fn from(refusal: Refusal) -> Self {
        Refusals(vec![refusal])
    }
}

impl<'a> IntoIterator for &'a Refusals {
    type Item = &'a Refusal;
    type IntoIter = std::slice::Iter<'a, Refusal>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl fmt::Display for Refusals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, refusal) in self.0.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "{refusal}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Refusals {}
