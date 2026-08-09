//! The accuracy grade an export prints, held as text and never as a number.
//!
//! `docs/decisions/uncertainty-model.md` refuses any mapping from a class to a
//! number inside the engine, and issue #23 refuses one inside this crate too.
//! So a grade is carried in a type with no arithmetic and no numeric
//! conversion, and this module is the only place one is held.
//!
//! The command that decides whether that still holds is in this crate's own
//! documentation rather than here. A grep whose pattern sits inside the file it
//! searches matches its own explanation, which is a check that can never come
//! back clean.

/// One accuracy grade, exactly as the export printed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccuracyGrade(String);

impl AccuracyGrade {
    /// Take the cell verbatim. Nothing is trimmed, folded or ranked here: two
    /// upstreams that both print `B` do not thereby mean the same thing, and a
    /// grade that has been tidied is no longer what the upstream said.
    pub fn taken(cell: &str) -> AccuracyGrade {
        AccuracyGrade(cell.to_owned())
    }

    /// The grade as the export wrote it.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AccuracyGrade {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
