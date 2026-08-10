//! Observations, hypotheses over items, the structural key and mutual
//! exclusion, for `docs/decisions/what-an-assignment-is.md`.
//!
//! `docs/decisions/layout.md` puts these objects here and fixes the vocabulary
//! they are written in. This crate knows that an item occupies a fixed set of
//! named slots and that the structural key is one multiset per slot. It does
//! not know how many slots there are, what they are called, or what a slot
//! value stands for. The crate that instantiates this declares all three.
//!
//! What the record asks the type to carry, and where each half is:
//!
//! - a hypothesis holds a **set** of items rather than a list or an optional
//!   single item, so size zero, one and many are one type with no special case
//!   and the empty hypothesis is never an absent value. [`Hypothesis`].
//! - the structural key is **computed** from the components rather than stored
//!   beside them, so the two cannot disagree. [`Hypothesis::key`].
//! - mutual exclusion is a question the type answers rather than a rule a
//!   solver reimplements. [`Claim::excludes`].
//! - a hypothesis set carries its size-zero member **from construction**, so no
//!   code path builds one without it. [`HypothesisSet::new`].
//!
//! Nothing here carries a probability. `docs/decisions/probability-model.md`
//! puts one on every member of a hypothesis set and issue #40 is where it
//! lands, so a hypothesis built here states what could have produced an
//! observation and not yet how likely that is. Nothing here scores anything
//! either: `docs/decisions/layout.md` keeps every score term on the other side
//! of the boundary.

pub mod hypothesis;
pub mod id;
pub mod item;
pub mod observation;
pub mod refusal;

pub use hypothesis::{Hypothesis, StructuralKey};
pub use id::{ChannelId, GroupId, ObservationId, SlotValue, SourceId};
pub use item::{Item, SlotLayout};
pub use observation::{Claim, HypothesisSet, Observation};
pub use refusal::Refused;
