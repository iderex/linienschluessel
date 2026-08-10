//! What an item is, and when two of them are the same item.
//!
//! `docs/decisions/what-an-assignment-is.md` fixes identity by listing every
//! part of it: an item is one value per slot together with the channel it was
//! generated under, and two items are the same item when all of those agree.
//! The source the values were drawn from is part of it as well, which the same
//! record makes load bearing where one item is seen in two groups.
//!
//! `docs/decisions/layout.md` puts the slot names outside this crate. A layout
//! here is a count and a list of names it was handed; nothing in this file
//! knows what any of the names mean or that there are two of them.

use std::fmt;

use crate::id::{ChannelId, SlotValue, SourceId};
use crate::refusal::Refused;

/// The fixed set of named slots every item of one run occupies.
///
/// Declared by the crate that instantiates this model, once, and handed to
/// every item built under it. The arity is what makes a structural key
/// comparable between two hypotheses, so an item built against a different
/// layout is refused rather than compared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotLayout {
    names: Vec<String>,
}

impl SlotLayout {
    /// Declare the slots, in the order every item will give its values in.
    ///
    /// A layout with no slot is refused. A structural key over no slot is the
    /// same key for every hypothesis, which would make every hypothesis a
    /// duplicate of every other in the one place that decides what a competitor
    /// is. A repeated name is refused for a smaller reason: two slots a caller
    /// cannot tell apart are two slots a caller will eventually swap.
    pub fn new(names: impl IntoIterator<Item = impl Into<String>>) -> Result<Self, Refused> {
        let names: Vec<String> = names.into_iter().map(Into::into).collect();
        if names.is_empty() {
            return Err(Refused::LayoutHasNoSlot);
        }
        for (index, name) in names.iter().enumerate() {
            if names[..index].contains(name) {
                return Err(Refused::SlotNameRepeated { name: name.clone() });
            }
        }
        Ok(SlotLayout { names })
    }

    /// How many values every item under this layout carries.
    pub fn arity(&self) -> usize {
        self.names.len()
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// The position a named slot occupies in a key, or `None` for a name this
    /// layout does not declare.
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|declared| declared == name)
    }
}

/// One thing that could have produced part of an observation.
///
/// Ordering is over the source, then the slot values in layout order, then the
/// channel, and it is derived from the identifiers rather than from anything
/// measured. That is what lets a hypothesis be an ordered set whose iteration
/// order is the same on every run.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item {
    source: SourceId,
    slots: Vec<SlotValue>,
    channel: ChannelId,
}

impl Item {
    /// Build an item against the layout its values are given in.
    ///
    /// A value count that is not the layout's arity is refused here rather than
    /// filled in or truncated. A missing value would otherwise become a slot
    /// with no entry in the structural key, and a key missing an entry compares
    /// equal to no hypothesis and unequal to the one it was meant to be.
    pub fn new(
        layout: &SlotLayout,
        source: SourceId,
        values: impl IntoIterator<Item = SlotValue>,
        channel: ChannelId,
    ) -> Result<Self, Refused> {
        let slots: Vec<SlotValue> = values.into_iter().collect();
        if slots.len() != layout.arity() {
            return Err(Refused::SlotCountDiffers {
                expected: layout.arity(),
                found: slots.len(),
            });
        }
        Ok(Item {
            source,
            slots,
            channel,
        })
    }

    pub fn source(&self) -> &SourceId {
        &self.source
    }

    pub fn slots(&self) -> &[SlotValue] {
        &self.slots
    }

    pub fn channel(&self) -> &ChannelId {
        &self.channel
    }

    /// The number of slots this item carries a value for.
    pub fn arity(&self) -> usize {
        self.slots.len()
    }

    /// The value in one slot, or `None` for a slot beyond this item's arity.
    pub fn value(&self, slot: usize) -> Option<&SlotValue> {
        self.slots.get(slot)
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:(", self.source)?;
        for (index, value) in self.slots.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{value}")?;
        }
        write!(f, ") via {}", self.channel)
    }
}
