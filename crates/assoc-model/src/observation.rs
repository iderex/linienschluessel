//! An observation, the hypotheses it could have, and when two of them cannot
//! both hold.
//!
//! `docs/decisions/what-an-assignment-is.md` keeps two objects apart that loose
//! talk runs together. A configuration is a choice of exactly one hypothesis
//! for every observation in a run, and there are combinatorially many of them.
//! A hypothesis set is every hypothesis one observation could have, and it is
//! the object a run reports. This module carries the second and the exclusion
//! rules a configuration would have to satisfy; the search over configurations
//! is elsewhere.

use std::collections::BTreeSet;

use crate::hypothesis::Hypothesis;
use crate::id::{GroupId, ObservationId};
use crate::item::Item;
use crate::refusal::Refused;

/// One thing measured once, and the group it was measured in.
///
/// The group decides what competes with what. Identifiers are unique within a
/// run; two observations built with one identifier are one observation as far
/// as every rule below is concerned.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Observation {
    id: ObservationId,
    group: GroupId,
}

impl Observation {
    pub fn new(id: ObservationId, group: GroupId) -> Self {
        Observation { id, group }
    }

    pub fn id(&self) -> &ObservationId {
        &self.id
    }

    pub fn group(&self) -> &GroupId {
        &self.group
    }
}

/// Every hypothesis one observation could have.
///
/// The size-zero member is put in by the constructor and by nothing else, so
/// there is no path through this type that produces a set without it. That is
/// the whole reason the constructor exists rather than the field being public:
/// the record says the size-zero hypothesis is a member of every hypothesis
/// set, including the sets of observations with a dozen good candidates, and a
/// caller that assembles the set itself is a caller who can leave it out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HypothesisSet {
    observation: ObservationId,
    hypotheses: BTreeSet<Hypothesis>,
}

impl HypothesisSet {
    /// Build the set for one observation from the hypotheses offered for it.
    ///
    /// Offering none is legitimate and produces a set of one. Offering the
    /// size-zero hypothesis explicitly changes nothing, because it is already
    /// there.
    ///
    /// Hypotheses of differing arity are refused, since a set whose members
    /// carry keys of two widths cannot be compared member against member, which
    /// is the one thing this set is for.
    pub fn new(
        observation: ObservationId,
        offered: impl IntoIterator<Item = Hypothesis>,
    ) -> Result<Self, Refused> {
        let mut hypotheses: BTreeSet<Hypothesis> = BTreeSet::new();
        hypotheses.insert(Hypothesis::none_of_these());

        let mut arity: Option<usize> = None;
        for hypothesis in offered {
            if !hypothesis.is_none_of_these() {
                match arity {
                    None => arity = Some(hypothesis.arity()),
                    Some(expected) if expected != hypothesis.arity() => {
                        return Err(Refused::SlotCountDiffers {
                            expected,
                            found: hypothesis.arity(),
                        });
                    }
                    Some(_) => {}
                }
            }
            hypotheses.insert(hypothesis);
        }

        Ok(HypothesisSet {
            observation,
            hypotheses,
        })
    }

    pub fn observation(&self) -> &ObservationId {
        &self.observation
    }

    /// True always. It is a method rather than a comment so that a test can ask
    /// it of a set built any way a caller can build one.
    pub fn holds_none_of_these(&self) -> bool {
        self.hypotheses.contains(&Hypothesis::none_of_these())
    }

    /// How many hypotheses the observation has, the size-zero one included.
    pub fn len(&self) -> usize {
        self.hypotheses.len()
    }

    /// False always, since the size-zero member is never absent. Present
    /// because a type carrying `len` and no `is_empty` is one every caller
    /// writes `len() == 0` against instead.
    pub fn is_empty(&self) -> bool {
        self.hypotheses.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Hypothesis> {
        self.hypotheses.iter()
    }
}

/// One hypothesis together with the observation it is a hypothesis for.
///
/// Exclusion is a question about a pair of these rather than about a pair of
/// hypotheses. Two identical sets of items exclude each other when they belong
/// to two observations of one group and support each other when they belong to
/// two groups, so a hypothesis alone cannot answer it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Claim<'a> {
    observation: &'a Observation,
    hypothesis: &'a Hypothesis,
}

impl<'a> Claim<'a> {
    pub fn new(observation: &'a Observation, hypothesis: &'a Hypothesis) -> Self {
        Claim {
            observation,
            hypothesis,
        }
    }

    pub fn observation(&self) -> &Observation {
        self.observation
    }

    pub fn hypothesis(&self) -> &Hypothesis {
        self.hypothesis
    }

    /// True where no configuration may contain both.
    ///
    /// Within one observation, because a configuration chooses exactly one of
    /// its hypotheses. Across observations of one group, when the two claim an
    /// item in common, because one item produces one observation and a
    /// configuration letting two claim it says it was recorded twice. Across
    /// groups, never: see [`Claim::shares_support_with`].
    ///
    /// A hypothesis against itself is one hypothesis and not two, so it
    /// excludes nothing. A configuration holding it holds no second thing that
    /// the first would have to make room for.
    pub fn excludes(&self, other: &Claim<'_>) -> bool {
        if self.observation.id() == other.observation.id() {
            return self.hypothesis != other.hypothesis;
        }
        if self.observation.group() != other.observation.group() {
            return false;
        }
        self.hypothesis.shares_an_item_with(other.hypothesis)
    }

    /// True where the two are in different groups and claim an item in common.
    ///
    /// One item claimed in two groups is one item observed twice, which is the
    /// consistency evidence a model built one group at a time throws away. It
    /// is the opposite of a conflict and it is named here so that nothing has
    /// to infer it from [`Claim::excludes`] returning false, which it also does
    /// for two claims that have nothing to do with each other.
    pub fn shares_support_with(&self, other: &Claim<'_>) -> bool {
        self.observation.group() != other.observation.group()
            && self.hypothesis.shares_an_item_with(other.hypothesis)
    }

    /// Every item both claims hold, which is what a reported conflict names.
    pub fn shared_items(&self, other: &Claim<'_>) -> Vec<Item> {
        self.hypothesis.shared_items(other.hypothesis)
    }
}
