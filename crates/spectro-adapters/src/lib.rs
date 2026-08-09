//! One module per upstream export, each emitting a conforming input.
//!
//! `docs/decisions/layout.md` puts the adapters here and gives them one job. An
//! adapter reads somebody else's file and emits a file
//! `docs/decisions/input-contract.md` describes. It scores nothing, converts no
//! medium, and turns no published grade into a number.
//!
//! Two of those are greppable over this crate rather than merely stated, and
//! the commands are in the change that landed this crate rather than here. A
//! grep whose pattern sits inside the file it searches matches its own
//! explanation, and a check that can never come back clean is worse than none.
//!
//! What the greps bound. They reach this crate and no further, so neither stops
//! the same mapping being written in another crate against
//! [`accuracy::AccuracyGrade::as_str`]. The invariant that reaches the whole
//! tree is issue #53's and is not landed.

pub mod accuracy;
pub mod nist_asd_levels;
pub mod nist_asd_lines;

pub use accuracy::AccuracyGrade;
pub use nist_asd_lines::{Conversion, Naming, NotTaken};

// The levels export reaches a caller as `nist_asd_levels::Conversion` and
// `nist_asd_levels::Naming` and is not re-exported here. Two upstream exports
// need the same two words for two different things, one carrying a spectrum and
// a line list identifier and the other a species and a level set identifier, and
// a root that holds one of each pair silently decides which upstream a bare
// `Naming` means.
