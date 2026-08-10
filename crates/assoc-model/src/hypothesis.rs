//! A hypothesis, and the structural key computed from it.
//!
//! `docs/decisions/what-an-assignment-is.md` fixes the shape: a hypothesis for
//! an observation is a set of distinct items, size zero says no item in the
//! offered set produced the observation, size one is a single assignment, and
//! size two or more is a blend. One type, three sizes, no special case, and no
//! absent value standing in for the empty one.
//!
//! The key is computed on request and never stored. A key stored beside the
//! components is a second copy of the same fact, and the failure it invites is
//! silent: an edit to the components that forgets the copy leaves a hypothesis
//! selecting competitors by what it used to be.

use std::collections::{BTreeMap, BTreeSet};

use crate::id::SlotValue;
use crate::item::Item;
use crate::refusal::Refused;

/// Every item a hypothesis claims produced its observation.
///
/// Ordered underneath, so iteration is the same on every run and on every
/// platform. `docs/decisions/repeatable-runs.md` owns the wider rule; what this
/// type owes it is that the order comes from identifiers and never from the
/// order a caller happened to offer.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hypothesis {
    items: BTreeSet<Item>,
}

impl Hypothesis {
    /// Build a hypothesis from the items it claims.
    ///
    /// One item offered twice is refused rather than deduplicated. Deduplicating
    /// would accept a caller saying that one item produced part of an
    /// observation and then produced it again, which is not a statement about
    /// anything, and it would accept it silently.
    ///
    /// Items of differing arity are refused for the reason [`StructuralKey`]
    /// gives: two keys of different widths are not comparable, and a hypothesis
    /// nothing can be compared with is worse than one that was never built.
    pub fn new(items: impl IntoIterator<Item = Item>) -> Result<Self, Refused> {
        let mut held: BTreeSet<Item> = BTreeSet::new();
        let mut arity: Option<usize> = None;
        for item in items {
            match arity {
                None => arity = Some(item.arity()),
                Some(expected) if expected != item.arity() => {
                    return Err(Refused::SlotCountDiffers {
                        expected,
                        found: item.arity(),
                    });
                }
                Some(_) => {}
            }
            if !held.insert(item.clone()) {
                return Err(Refused::ItemRepeated {
                    item: Box::new(item),
                });
            }
        }
        Ok(Hypothesis { items: held })
    }

    /// The hypothesis that no offered item produced the observation.
    ///
    /// It is a hypothesis of size zero and not a flag, an option or a sentinel,
    /// so it is weighed against the others rather than beside them.
    pub fn none_of_these() -> Self {
        Hypothesis {
            items: BTreeSet::new(),
        }
    }

    /// How many items this hypothesis claims.
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// True for the size-zero hypothesis and for no other.
    pub fn is_none_of_these(&self) -> bool {
        self.items.is_empty()
    }

    /// How many slots the items of this hypothesis carry, and zero where it
    /// claims none.
    pub fn arity(&self) -> usize {
        self.items.first().map_or(0, Item::arity)
    }

    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.items.iter()
    }

    pub fn holds(&self, item: &Item) -> bool {
        self.items.contains(item)
    }

    /// True where both hypotheses claim at least one item in common.
    ///
    /// A whole item, never a slot value. Two hypotheses reaching one slot value
    /// from two different items are the ordinary case rather than a conflict.
    pub fn shares_an_item_with(&self, other: &Hypothesis) -> bool {
        self.items.intersection(&other.items).next().is_some()
    }

    /// Every item both hypotheses claim, in the canonical order.
    ///
    /// This is what a conflict is reported with. A run that says two
    /// observations cannot both hold what they hold has to name the item, or a
    /// reader is left to find it.
    pub fn shared_items(&self, other: &Hypothesis) -> Vec<Item> {
        self.items.intersection(&other.items).cloned().collect()
    }

    /// One multiset per slot, computed from the items now.
    pub fn key(&self) -> StructuralKey {
        let mut slots: Vec<BTreeMap<SlotValue, usize>> = vec![BTreeMap::new(); self.arity()];
        for item in &self.items {
            for (index, value) in item.slots().iter().enumerate() {
                *slots[index].entry(value.clone()).or_insert(0) += 1;
            }
        }
        StructuralKey { slots }
    }
}

/// What two hypotheses are compared by when the question is whether they are
/// the same claim.
///
/// One multiset per slot, in layout order. Multisets rather than sets, because
/// two distinct items may share a value in one slot: a hypothesis of two items
/// reaching one slot value from a second is a different claim from a hypothesis
/// of two items reaching two, and a set would collapse the two into one.
///
/// The key of the size-zero hypothesis carries no slot at all, because a
/// hypothesis claiming no item names no value in any slot. It cannot collide
/// with any other key: a hypothesis of `n` items puts `n` entries in every slot
/// it has, so every key with a slot has a non-empty multiset in it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructuralKey {
    slots: Vec<BTreeMap<SlotValue, usize>>,
}

impl StructuralKey {
    /// How many slots this key carries a multiset for.
    pub fn width(&self) -> usize {
        self.slots.len()
    }

    /// The multiset in one slot, or `None` for a slot this key does not carry.
    pub fn multiset(&self, slot: usize) -> Option<&BTreeMap<SlotValue, usize>> {
        self.slots.get(slot)
    }

    /// How many of the hypothesis's items put `value` in `slot`.
    pub fn count(&self, slot: usize, value: &SlotValue) -> usize {
        self.slots
            .get(slot)
            .and_then(|multiset| multiset.get(value))
            .copied()
            .unwrap_or(0)
    }
}
