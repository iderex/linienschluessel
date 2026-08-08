//! The lexical layer every conforming file shares.
//!
//! `docs/decisions/input-contract.md` fixes the medium as "a UTF-8 text file
//! with a labelled header and a tab-separated body" and then fixes the fields.
//! Between those two there is a gap: a producer cannot emit a file from a field
//! list, and a validator cannot refuse an unlabelled field without a rule
//! saying where a label lives. This module closes that gap and the rules below
//! are the validator's, not the record's. Where the record is silent the choice
//! here is the one that refuses a producer's mistake rather than absorbing it,
//! and pulling these rules back into the record is owed.
//!
//! The rules.
//!
//! The file is UTF-8. A leading UTF-8 byte order mark is accepted and is not
//! part of the first field, because an export written by a spreadsheet carries
//! one and the contents of such a file are not wrong. Any other byte order mark
//! is refused.
//!
//! A line ends with LF or with CRLF and one file may not mix them. A carriage
//! return anywhere else is refused rather than treated as whitespace. The file
//! may end with one line terminator or with none. A line with no bytes in it is
//! refused, because a blank line and a row of one empty cell cannot be told
//! apart afterwards.
//!
//! The header block comes first. A header line begins with `#` in column zero,
//! immediately followed by the field name, then one tab, then the value, which
//! runs to the end of the line. The block ends at the first line that does not
//! begin with `#`, and that line is the column label row. A line beginning with
//! `#` below the label row is refused naming the line: a comment where a number
//! should be is a thing real exports carry, and the answer here is a refusal
//! rather than a skip, because a skipped row is a row missing from an answer
//! that does not say so.
//!
//! A header field appears once. A column label appears once. Neither an unknown
//! header field nor an unknown label is refused here, because the minor version
//! rule in the record requires both to be kept and reported.
//!
//! Every body row carries exactly as many tab-separated cells as the label row.
//! No cell is trimmed: a cell is the bytes between two tabs, so ` 1.0` is not
//! the number `1.0`. Padding a column to a width is a habit of exports rather
//! than of this contract, and stripping it is what an adapter is for.

use crate::refusal::{Refusal, Refusals};

/// One `#name<tab>value` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
    pub line: usize,
}

/// One body row, with its cells in label order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub line: usize,
    pub cells: Vec<String>,
}

/// A file that is lexically a conforming file, whatever its fields say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub headers: Vec<Header>,
    pub labels: Vec<String>,
    pub label_line: usize,
    pub rows: Vec<Row>,
}

/// How a file terminated its lines, which one file may not mix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ending {
    Lf,
    CrLf,
}

impl Document {
    /// Read the bytes of a file into headers, labels and rows, or refuse.
    ///
    /// This takes bytes rather than a `str` on purpose. A reader is a thing
    /// that meets byte order marks and carriage returns, and a signature taking
    /// `&str` would have had somebody else decide about both before the
    /// validator ever saw the file.
    pub fn lex(bytes: &[u8]) -> Result<Document, Refusals> {
        let body = strip_byte_order_mark(bytes)?;
        let text = match std::str::from_utf8(body) {
            Ok(text) => text,
            Err(error) => {
                return Err(Refusal::in_file(format!(
                    "the file is not UTF-8: the byte at offset {} does not begin a valid sequence",
                    error.valid_up_to()
                ))
                .into());
            }
        };

        let mut refusals = Refusals::new();
        let mut lines: Vec<(usize, &str)> = Vec::new();
        let mut ending: Option<Ending> = None;

        let pieces: Vec<&str> = text.split('\n').collect();
        let last_piece = pieces.len() - 1;
        for (index, raw) in pieces.iter().copied().enumerate() {
            let number = index + 1;
            let is_last = index == last_piece;
            if is_last && raw.is_empty() {
                // The terminator of the previous line, not a line of its own.
                break;
            }
            let (content, this_ending) = match raw.strip_suffix('\r') {
                Some(stripped) if !is_last => (stripped, Some(Ending::CrLf)),
                _ if !is_last => (raw, Some(Ending::Lf)),
                _ => (raw, None),
            };
            if let Some(this) = this_ending {
                match ending {
                    None => ending = Some(this),
                    Some(first) if first != this => {
                        refusals.push(Refusal::on_line(
                            number,
                            "the line terminator is not the one the file started with, and one file may not mix LF and CRLF",
                        ));
                    }
                    Some(_) => {}
                }
            }
            if content.contains('\r') {
                refusals.push(Refusal::on_line(
                    number,
                    "a carriage return appears somewhere other than at the end of the line",
                ));
            }
            if content.is_empty() {
                refusals.push(Refusal::on_line(
                    number,
                    "the line has no bytes in it, and a blank line cannot be told from a row of one empty cell",
                ));
                continue;
            }
            lines.push((number, content));
        }

        let mut headers: Vec<Header> = Vec::new();
        let mut cursor = 0;
        while let Some(&(number, content)) = lines.get(cursor) {
            let Some(rest) = content.strip_prefix('#') else {
                break;
            };
            cursor += 1;
            match rest.split_once('\t') {
                Some((name, value)) if !name.is_empty() => {
                    if headers.iter().any(|held| held.name == name) {
                        refusals.push(Refusal::field_on_line(
                            number,
                            name,
                            "the header field appears more than once",
                        ));
                    }
                    headers.push(Header {
                        name: name.to_owned(),
                        value: value.to_owned(),
                        line: number,
                    });
                }
                Some((_, _)) => refusals.push(Refusal::on_line(
                    number,
                    "the header line names no field before its tab",
                )),
                None => refusals.push(Refusal::on_line(
                    number,
                    "the header line carries no tab, so it states a name and no value",
                )),
            }
        }

        let Some(&(label_line, label_content)) = lines.get(cursor) else {
            refusals.push(Refusal::in_file(
                "the file carries no column label row, so no field in it is labelled",
            ));
            return Err(refusals);
        };
        cursor += 1;

        let labels: Vec<String> = label_content.split('\t').map(str::to_owned).collect();
        for (position, label) in labels.iter().enumerate() {
            if label.is_empty() {
                refusals.push(Refusal::on_line(
                    label_line,
                    format!("column {} carries no label", position + 1),
                ));
            } else if labels.iter().take(position).any(|earlier| earlier == label) {
                refusals.push(Refusal::field_on_line(
                    label_line,
                    label,
                    "the column label appears more than once",
                ));
            }
        }

        let mut rows: Vec<Row> = Vec::new();
        for &(number, content) in &lines[cursor..] {
            if content.starts_with('#') {
                refusals.push(Refusal::on_line(
                    number,
                    "a line below the label row begins with `#`, and a comment where a row should be is refused rather than skipped",
                ));
                continue;
            }
            let cells: Vec<String> = content.split('\t').map(str::to_owned).collect();
            if cells.len() != labels.len() {
                refusals.push(Refusal::on_line(
                    number,
                    format!(
                        "the row carries {} cells against {} labels, so it is refused as a row shape rather than as a missing field",
                        cells.len(),
                        labels.len()
                    ),
                ));
                continue;
            }
            rows.push(Row {
                line: number,
                cells,
            });
        }

        refusals.into_result(Document {
            headers,
            labels,
            label_line,
            rows,
        })
    }

    /// The header field of that name, where the file carried one.
    pub fn header(&self, name: &str) -> Option<&Header> {
        self.headers.iter().find(|held| held.name == name)
    }

    /// The position of the column of that label, where the file carried one.
    pub fn column(&self, label: &str) -> Option<usize> {
        self.labels.iter().position(|held| held == label)
    }

    /// The header fields this file carried that the caller did not name.
    ///
    /// Reported rather than dropped, which is what the record's minor version
    /// rule requires of a reader meeting a later minor than it was written for.
    pub fn headers_outside(&self, known: &[&str]) -> Vec<String> {
        self.headers
            .iter()
            .filter(|held| !known.contains(&held.name.as_str()))
            .map(|held| held.name.clone())
            .collect()
    }

    /// The column labels this file carried that the caller did not name.
    pub fn columns_outside(&self, known: &[&str]) -> Vec<String> {
        self.labels
            .iter()
            .filter(|held| !known.contains(&held.as_str()))
            .cloned()
            .collect()
    }
}

fn strip_byte_order_mark(bytes: &[u8]) -> Result<&[u8], Refusal> {
    const UTF8: &[u8] = &[0xEF, 0xBB, 0xBF];
    const UTF16_LE: &[u8] = &[0xFF, 0xFE];
    const UTF16_BE: &[u8] = &[0xFE, 0xFF];

    if let Some(rest) = bytes.strip_prefix(UTF8) {
        return Ok(rest);
    }
    if bytes.starts_with(UTF16_LE) || bytes.starts_with(UTF16_BE) {
        return Err(Refusal::in_file(
            "the file begins with a UTF-16 byte order mark, and a conforming file is UTF-8",
        ));
    }
    Ok(bytes)
}
