//! What this crate refuses to build, and what it says when it does.
//!
//! Every variant below is a shape that has a plausible silent repair, and the
//! silent repair is why the refusal exists. A repeated item could be
//! deduplicated, a short value list could be padded, a layout with no slot
//! could be tolerated. Each of those produces an object that is well formed and
//! says something other than what its caller meant, and none of them is
//! visible afterwards.

use std::fmt;

use crate::item::Item;

/// One thing this crate would not build, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// A slot layout declaring no slot.
    LayoutHasNoSlot,
    /// Two slots of one layout under one name.
    SlotNameRepeated { name: String },
    /// A value count, or an item arity, that does not match what it is being
    /// put beside.
    SlotCountDiffers { expected: usize, found: usize },
    /// One item offered twice inside one hypothesis.
    ItemRepeated { item: Box<Item> },
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refused::LayoutHasNoSlot => f.write_str(
                "a slot layout declares no slot, so every hypothesis would carry the same key",
            ),
            Refused::SlotNameRepeated { name } => {
                write!(f, "two slots are declared under the name `{name}`")
            }
            Refused::SlotCountDiffers { expected, found } => {
                write!(f, "{expected} slot(s) were expected and {found} were given")
            }
            Refused::ItemRepeated { item } => write!(
                f,
                "the item {item} was offered twice inside one hypothesis, which claims it \
                 produced part of the observation and then produced it again"
            ),
        }
    }
}

impl std::error::Error for Refused {}
