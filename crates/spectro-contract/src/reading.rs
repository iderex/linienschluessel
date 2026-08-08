//! The part of reading a file that is the same for all four of them.
//!
//! An absent required column and an empty value in a present one are different
//! statements and the record refuses them differently, so they are separate
//! calls here rather than one call with a flag. That distinction is the one the
//! record singles out as easy to leave out, and a helper that collapsed it
//! would put it back.

use crate::document::{Document, Row};
use crate::refusal::{Refusal, Refusals};
use crate::value::{SUPPORTED_MAJOR, Version, parse_vocabulary};

/// One file part-way through being read, carrying what has been refused so far.
pub(crate) struct Reading<'a> {
    document: &'a Document,
    refusals: Refusals,
}

impl<'a> Reading<'a> {
    pub(crate) fn new(document: &'a Document) -> Self {
        Reading {
            document,
            refusals: Refusals::new(),
        }
    }

    pub(crate) fn document(&self) -> &'a Document {
        self.document
    }

    pub(crate) fn refuse(&mut self, refusal: Refusal) {
        self.refusals.push(refusal);
    }

    /// Finish where the value could only be built if every required field was
    /// there, and refuse where it could not.
    ///
    /// This exists so that no reader has to invent a placeholder for a field
    /// the file never gave. A reader that filled in an empty unit in order to
    /// construct a value it was about to throw away would still be a reader
    /// with a default unit in it, and the invariant this crate owes issue #20
    /// is a search over its own source rather than a claim about which branch
    /// runs.
    pub(crate) fn finish<T>(mut self, value: Option<T>) -> Result<T, Refusals> {
        match value {
            Some(value) => self.refusals.into_result(value),
            None => {
                if self.refusals.is_empty() {
                    self.refuse(Refusal::in_file(
                        "the file could not be read and no reason was recorded",
                    ));
                }
                Err(self.refusals)
            }
        }
    }

    /// A header field the file has to carry.
    pub(crate) fn required_header(&mut self, name: &str) -> Option<&'a str> {
        match self.document.header(name) {
            Some(header) if header.value.is_empty() => {
                self.refuse(Refusal::field_on_line(
                    header.line,
                    name,
                    "the header field is present and states nothing, and an empty value is not a value here",
                ));
                None
            }
            Some(header) => Some(header.value.as_str()),
            None => {
                self.refuse(Refusal::absent_field(
                    name,
                    "the file carries no such header field",
                ));
                None
            }
        }
    }

    /// A header field the file may carry.
    pub(crate) fn optional_header(&self, name: &str) -> Option<&'a str> {
        self.document
            .header(name)
            .map(|header| header.value.as_str())
    }

    /// `contract_version`, and whether this reader may go on reading the file.
    ///
    /// An absent marker is refused rather than defaulted to the current one,
    /// which the record states in its own sentence.
    pub(crate) fn contract_version(&mut self) -> Option<Version> {
        let Some(header) = self.document.header("contract_version") else {
            self.refuse(Refusal::absent_field(
                "contract_version",
                "the file states no contract version, and an absent version is refused rather than read as the current one",
            ));
            return None;
        };
        match Version::parse(&header.value) {
            Ok(version) if version.is_readable() => Some(version),
            Ok(version) => {
                self.refuse(Refusal::field_on_line(
                    header.line,
                    "contract_version",
                    format!(
                        "the file states major version {} and this reader knows major version {SUPPORTED_MAJOR} only",
                        version.major
                    ),
                ));
                None
            }
            Err(reason) => {
                self.refuse(Refusal::field_on_line(
                    header.line,
                    "contract_version",
                    reason,
                ));
                None
            }
        }
    }

    /// A column the file has to label, refused as absent where it does not.
    pub(crate) fn required_column(&mut self, label: &str) -> Option<usize> {
        match self.document.column(label) {
            Some(index) => Some(index),
            None => {
                self.refuse(Refusal::absent_field(
                    label,
                    "the label row carries no such column, which is a fact about the file rather than a value that is missing from a row",
                ));
                None
            }
        }
    }

    /// A column the file may label.
    pub(crate) fn optional_column(&self, label: &str) -> Option<usize> {
        self.document.column(label)
    }

    /// The cell of a row under a column that was found.
    pub(crate) fn cell(&self, row: &'a Row, column: usize) -> &'a str {
        row.cells[column].as_str()
    }

    /// A cell that has to carry something.
    pub(crate) fn required_cell(
        &mut self,
        row: &Row,
        column: usize,
        label: &str,
    ) -> Option<String> {
        let cell = row.cells[column].as_str();
        if cell.is_empty() {
            self.refuse(Refusal::field_on_line(
                row.line,
                label,
                "the cell is empty, and this field has no empty value",
            ));
            return None;
        }
        Some(cell.to_owned())
    }

    /// A cell read against a closed vocabulary.
    pub(crate) fn vocabulary_cell(
        &mut self,
        row: &Row,
        column: usize,
        label: &str,
        vocabulary: &'static [&'static str],
    ) -> Option<&'static str> {
        let cell = row.cells[column].as_str();
        match parse_vocabulary(cell, vocabulary) {
            Ok(member) => Some(member),
            Err(reason) => {
                self.refuse(Refusal::field_on_line(row.line, label, reason));
                None
            }
        }
    }

    /// `uncertainty_class`, which both the level set and the line list carry
    /// under the same rule.
    ///
    /// It is required exactly where `uncertainty_kind` is `class`, and it is
    /// refused anywhere else, because a class label sitting beside a numeric
    /// kind describes nothing and would travel into an answer unchallenged. The
    /// outer `None` is a refusal; the inner one is a row that legitimately
    /// names no class.
    pub(crate) fn uncertainty_class(
        &mut self,
        row: &Row,
        column: usize,
        kind: Option<&'static str>,
    ) -> Option<Option<String>> {
        let cell = row.cells[column].as_str();
        match (kind, cell.is_empty()) {
            (Some("class"), true) => {
                self.refuse(Refusal::field_on_line(
                    row.line,
                    "uncertainty_class",
                    "the uncertainty kind is `class` and no class is named, so the flag the upstream stated cannot travel",
                ));
                None
            }
            (Some("class"), false) => Some(Some(cell.to_owned())),
            (Some(_), false) => {
                self.refuse(Refusal::field_on_line(
                    row.line,
                    "uncertainty_class",
                    "a class is named against an uncertainty kind that is not `class`",
                ));
                None
            }
            (_, true) => Some(None),
            (None, false) => Some(Some(cell.to_owned())),
        }
    }

    /// A cell read by a parser that refuses in its own words.
    pub(crate) fn parsed_cell<T>(
        &mut self,
        row: &Row,
        column: usize,
        label: &str,
        parse: impl Fn(&str) -> Result<T, String>,
    ) -> Option<T> {
        let cell = row.cells[column].as_str();
        match parse(cell) {
            Ok(value) => Some(value),
            Err(reason) => {
                self.refuse(Refusal::field_on_line(row.line, label, reason));
                None
            }
        }
    }
}
