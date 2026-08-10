//! The identifiers this crate carries, and why each one is its own type.
//!
//! All five are strings underneath and none of them is interchangeable with
//! another. A slot value handed where a channel was meant is the mistake that
//! produces a hypothesis nobody can read and that no arithmetic catches, so the
//! distinction is made by the compiler rather than by a naming convention.
//!
//! An identifier is taken as the producer of the input wrote it and is never
//! parsed, trimmed or folded here. Two identifiers are the same identifier when
//! their bytes agree. `docs/decisions/repeatable-runs.md` derives a run's
//! canonical order from identifiers, and an order over values something else
//! had already normalised would move the day the normalisation changed.

use std::fmt;

macro_rules! identifier {
    ($(#[$note:meta])* $name:ident) => {
        $(#[$note])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Take the identifier exactly as it was given.
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

identifier! {
    /// The set an item's slot values were drawn from.
    ///
    /// It is part of an item's identity rather than context beside it. Two runs
    /// against different sets share no item identity even where the values they
    /// name are close, and the record leans on that: the same item seen in two
    /// groups is shared support only because both name one source.
    SourceId
}

identifier! {
    /// One value occupying one slot of one item.
    SlotValue
}

identifier! {
    /// The channel an item was generated under.
    ///
    /// Two items with the same slot values generated under different channels
    /// are different items, because they are different claims about what
    /// produced the observation rather than one claim written twice.
    ChannelId
}

identifier! {
    /// One thing measured once, which a hypothesis set is built for.
    ObservationId
}

identifier! {
    /// The set of observations that compete with one another.
    ///
    /// Two observations in one group may not both hold one item. Two
    /// observations in different groups may, and that is the strongest
    /// consistency evidence a model per group throws away.
    GroupId
}
