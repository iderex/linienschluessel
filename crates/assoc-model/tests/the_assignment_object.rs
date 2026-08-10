//! The four things `docs/decisions/what-an-assignment-is.md` says the type owes
//! when it lands, and the near-misses each one exists against.
//!
//! Every case here is built from identifiers written in this file. There is no
//! reader on this side of the boundary and nothing here is drawn from an input.
//!
//! The slots are called `from` and `to` because this crate does not know what
//! they mean. The crate that instantiates the model names them, which is
//! `docs/decisions/layout.md`'s rule and the reason these names are deliberately
//! uninformative.

use assoc_model::{
    ChannelId, Claim, GroupId, Hypothesis, HypothesisSet, Item, Observation, ObservationId,
    Refused, SlotLayout, SlotValue, SourceId,
};

// ---------------------------------------------------------------- the fixtures

fn layout() -> SlotLayout {
    SlotLayout::new(["from", "to"]).expect("two distinct slot names are a layout")
}

/// An item under the default source and channel, from `from` to `to`.
fn item(from: &str, to: &str) -> Item {
    at("SET-1", from, to, "C1")
}

/// An item with every part of its identity given.
fn at(source: &str, from: &str, to: &str, channel: &str) -> Item {
    Item::new(
        &layout(),
        SourceId::new(source),
        [SlotValue::new(from), SlotValue::new(to)],
        ChannelId::new(channel),
    )
    .expect("two values under a two-slot layout")
}

fn hypothesis(items: impl IntoIterator<Item = Item>) -> Hypothesis {
    Hypothesis::new(items).expect("distinct items of one arity are a hypothesis")
}

fn observation(id: &str, group: &str) -> Observation {
    Observation::new(ObservationId::new(id), GroupId::new(group))
}

// ------------------------------------------------- a set, and never a list

#[test]
fn one_item_offered_twice_is_refused_rather_than_deduplicated() {
    let repeated = Hypothesis::new([item("A", "B"), item("A", "B")]);

    match repeated {
        Err(Refused::ItemRepeated { item: offered }) => {
            assert_eq!(*offered, item("A", "B"));
        }
        other => panic!("a repeated item is refused, and this was {other:?}"),
    }

    // The near-miss this guards: collecting into a set, which accepts the same
    // input and reports a hypothesis of one where the caller wrote two.
    assert_eq!(hypothesis([item("A", "B")]).size(), 1);
}

#[test]
fn size_zero_one_and_many_are_one_type_with_no_absent_value() {
    let none_of_these = Hypothesis::none_of_these();
    let single = hypothesis([item("A", "B")]);
    let blend = hypothesis([item("A", "B"), item("C", "D")]);

    assert_eq!(none_of_these.size(), 0);
    assert_eq!(single.size(), 1);
    assert_eq!(blend.size(), 2);

    assert!(none_of_these.is_none_of_these());
    assert!(!single.is_none_of_these());
    assert!(!blend.is_none_of_these());

    // All three are the same type, so all three can be held, compared and
    // weighed together. The size-zero one is not an `Option::None` standing
    // beside the others.
    let together = [&none_of_these, &single, &blend];
    assert_eq!(together.len(), 3);
    assert_ne!(none_of_these, single);
}

#[test]
fn items_of_two_arities_are_refused_in_one_hypothesis() {
    let one_slot = SlotLayout::new(["only"]).expect("one name is a layout");
    let narrow = Item::new(
        &one_slot,
        SourceId::new("SET-1"),
        [SlotValue::new("A")],
        ChannelId::new("C1"),
    )
    .expect("one value under a one-slot layout");

    assert_eq!(
        Hypothesis::new([item("A", "B"), narrow]),
        Err(Refused::SlotCountDiffers {
            expected: 2,
            found: 1
        })
    );
}

#[test]
fn a_value_count_that_is_not_the_layouts_arity_is_refused() {
    let short = Item::new(
        &layout(),
        SourceId::new("SET-1"),
        [SlotValue::new("A")],
        ChannelId::new("C1"),
    );

    assert_eq!(
        short,
        Err(Refused::SlotCountDiffers {
            expected: 2,
            found: 1
        })
    );
}

#[test]
fn a_layout_with_no_slot_and_a_layout_with_one_name_twice_are_both_refused() {
    assert_eq!(
        SlotLayout::new(Vec::<String>::new()),
        Err(Refused::LayoutHasNoSlot)
    );
    assert_eq!(
        SlotLayout::new(["from", "from"]),
        Err(Refused::SlotNameRepeated {
            name: "from".to_owned()
        })
    );
    assert_eq!(layout().index_of("to"), Some(1));
    assert_eq!(layout().index_of("sideways"), None);
}

// ------------------------------------ the key, computed from the components

#[test]
fn the_key_is_one_multiset_per_slot_and_a_shared_value_is_counted_twice() {
    // Two items leaving one `from` value. This is the case the multiset exists
    // for: the `from` multiset holds that value twice.
    let shared_origin = hypothesis([item("A", "B"), item("A", "C")]);
    let key = shared_origin.key();

    assert_eq!(key.width(), 2);
    assert_eq!(key.count(0, &SlotValue::new("A")), 2);
    assert_eq!(key.count(1, &SlotValue::new("B")), 1);
    assert_eq!(key.count(1, &SlotValue::new("C")), 1);

    // A blend of two items with two different `from` values is a different
    // claim, and a set rather than a multiset would have collapsed the two.
    let separate_origins = hypothesis([item("A", "B"), item("D", "C")]);
    assert_ne!(shared_origin.key(), separate_origins.key());
    assert_eq!(separate_origins.key().count(0, &SlotValue::new("A")), 1);
}

#[test]
fn the_key_comes_from_the_components_and_not_from_the_order_they_arrived_in() {
    let one_way = hypothesis([item("A", "B"), item("C", "D")]);
    let other_way = hypothesis([item("C", "D"), item("A", "B")]);

    assert_eq!(one_way, other_way);
    assert_eq!(one_way.key(), other_way.key());

    // And a hypothesis differing in one component has a different key, so the
    // key tracks the components rather than being a label beside them.
    let changed = hypothesis([item("A", "B"), item("C", "E")]);
    assert_ne!(one_way.key(), changed.key());
}

#[test]
fn the_size_zero_key_carries_no_slot_and_collides_with_nothing() {
    let empty = Hypothesis::none_of_these().key();
    assert_eq!(empty.width(), 0);
    assert_eq!(empty.multiset(0), None);

    // Every key with a slot has a non-empty multiset in it, so nothing else can
    // produce this one.
    let single = hypothesis([item("A", "B")]).key();
    assert_ne!(empty, single);
    assert_eq!(single.multiset(0).map(|slot| slot.len()), Some(1));
}

// ------------------------------- the size-zero member, from construction

#[test]
fn a_hypothesis_set_carries_the_size_zero_member_however_it_was_built() {
    let with_nothing_offered =
        HypothesisSet::new(ObservationId::new("O1"), []).expect("no hypothesis is a legal offer");
    assert_eq!(with_nothing_offered.len(), 1);
    assert!(with_nothing_offered.holds_none_of_these());
    assert!(!with_nothing_offered.is_empty());

    let with_three = HypothesisSet::new(
        ObservationId::new("O2"),
        [
            hypothesis([item("A", "B")]),
            hypothesis([item("C", "D")]),
            hypothesis([item("A", "B"), item("C", "D")]),
        ],
    )
    .expect("three hypotheses of one arity are a set");
    assert_eq!(with_three.len(), 4);
    assert!(with_three.holds_none_of_these());

    // Offering it explicitly changes nothing, because it was never absent.
    let offered_explicitly = HypothesisSet::new(
        ObservationId::new("O3"),
        [Hypothesis::none_of_these(), hypothesis([item("A", "B")])],
    )
    .expect("the size-zero hypothesis may be offered");
    assert_eq!(offered_explicitly.len(), 2);
    assert!(offered_explicitly.holds_none_of_these());
    assert_eq!(offered_explicitly.observation(), &ObservationId::new("O3"));
}

#[test]
fn a_set_whose_members_carry_two_arities_is_refused() {
    let one_slot = SlotLayout::new(["only"]).expect("one name is a layout");
    let narrow = Item::new(
        &one_slot,
        SourceId::new("SET-1"),
        [SlotValue::new("A")],
        ChannelId::new("C1"),
    )
    .expect("one value under a one-slot layout");

    assert_eq!(
        HypothesisSet::new(
            ObservationId::new("O1"),
            [hypothesis([item("A", "B")]), hypothesis([narrow])],
        ),
        Err(Refused::SlotCountDiffers {
            expected: 2,
            found: 1
        })
    );
}

// ------------------------------------- exclusion, answered by the type

#[test]
fn two_hypotheses_of_one_observation_exclude_each_other() {
    let one = observation("O1", "G1");
    let this = hypothesis([item("A", "B")]);
    let that = hypothesis([item("C", "D")]);

    assert!(Claim::new(&one, &this).excludes(&Claim::new(&one, &that)));
    assert!(Claim::new(&one, &that).excludes(&Claim::new(&one, &this)));

    // Including the size-zero one, which competes with the rest rather than
    // sitting outside the competition.
    let nothing = Hypothesis::none_of_these();
    assert!(Claim::new(&one, &this).excludes(&Claim::new(&one, &nothing)));
}

#[test]
fn a_hypothesis_against_itself_is_one_hypothesis_and_not_two() {
    let one = observation("O1", "G1");
    let this = hypothesis([item("A", "B")]);

    assert!(!Claim::new(&one, &this).excludes(&Claim::new(&one, &this)));
}

#[test]
fn two_observations_of_one_group_exclude_each_other_only_over_a_whole_item() {
    let first = observation("O1", "G1");
    let second = observation("O2", "G1");

    let shared = hypothesis([item("A", "B")]);
    assert!(Claim::new(&first, &shared).excludes(&Claim::new(&second, &shared)));
    assert_eq!(
        Claim::new(&first, &shared).shared_items(&Claim::new(&second, &shared)),
        vec![item("A", "B")]
    );

    // The near-miss: exclusion written over slot values rather than over items.
    // These two claims share the `from` value and no item, which is the
    // ordinary case and not a conflict.
    let same_origin = hypothesis([item("A", "Z")]);
    assert!(!Claim::new(&first, &shared).excludes(&Claim::new(&second, &same_origin)));
    assert!(
        Claim::new(&first, &shared)
            .shared_items(&Claim::new(&second, &same_origin))
            .is_empty()
    );

    // And a claim sharing the `to` value only is not a conflict either.
    let same_target = hypothesis([item("Z", "B")]);
    assert!(!Claim::new(&first, &shared).excludes(&Claim::new(&second, &same_target)));
}

#[test]
fn a_blend_inherits_exclusion_from_each_of_its_components_separately() {
    let first = observation("O1", "G1");
    let second = observation("O2", "G1");
    let third = observation("O3", "G1");

    let blend = hypothesis([item("A", "B"), item("C", "D")]);
    let holds_one = hypothesis([item("A", "B")]);
    let holds_the_other = hypothesis([item("C", "D")]);
    let holds_neither = hypothesis([item("E", "F")]);

    assert!(Claim::new(&first, &blend).excludes(&Claim::new(&second, &holds_one)));
    assert!(Claim::new(&first, &blend).excludes(&Claim::new(&third, &holds_the_other)));
    assert!(!Claim::new(&first, &blend).excludes(&Claim::new(&second, &holds_neither)));
}

#[test]
fn across_groups_a_shared_item_is_support_and_never_exclusion() {
    let here = observation("O1", "G1");
    let elsewhere = observation("G0088", "G2");
    let shared = hypothesis([item("A", "B")]);

    assert!(!Claim::new(&here, &shared).excludes(&Claim::new(&elsewhere, &shared)));
    assert!(Claim::new(&here, &shared).shares_support_with(&Claim::new(&elsewhere, &shared)));

    // Two claims in one group are never support, whatever they share, because
    // there the same item is a conflict instead.
    let second_here = observation("O2", "G1");
    assert!(!Claim::new(&here, &shared).shares_support_with(&Claim::new(&second_here, &shared)));

    // And two claims with nothing in common are neither.
    let unrelated = hypothesis([item("Y", "Z")]);
    assert!(!Claim::new(&here, &shared).excludes(&Claim::new(&elsewhere, &unrelated)));
    assert!(!Claim::new(&here, &shared).shares_support_with(&Claim::new(&elsewhere, &unrelated)));
}

#[test]
fn an_item_is_not_the_same_item_against_another_source_or_another_channel() {
    let here = observation("O1", "G1");
    let elsewhere = observation("O2", "G2");

    let one_set = hypothesis([at("SET-1", "A", "B", "C1")]);
    let other_set = hypothesis([at("SET-2", "A", "B", "C1")]);
    let other_channel = hypothesis([at("SET-1", "A", "B", "C2")]);

    // The same slot values against another source are not one item, so nothing
    // is shared and there is no support to claim.
    assert_ne!(one_set, other_set);
    assert!(!Claim::new(&here, &one_set).shares_support_with(&Claim::new(&elsewhere, &other_set)));

    // The same slot values under another channel are a different claim about
    // what produced the observation, so two observations of one group may hold
    // both.
    let second_here = observation("O3", "G1");
    assert_ne!(one_set, other_channel);
    assert!(!Claim::new(&here, &one_set).excludes(&Claim::new(&second_here, &other_channel)));

    // The item that agrees in every part is one item, and there it is shared.
    assert!(
        Claim::new(&here, &one_set)
            .shares_support_with(&Claim::new(&elsewhere, &hypothesis([item("A", "B")])))
    );
}

#[test]
fn a_hypothesis_says_which_items_it_holds() {
    let blend = hypothesis([item("A", "B"), item("C", "D")]);

    assert!(blend.holds(&item("A", "B")));
    assert!(blend.holds(&item("C", "D")));
    assert!(!blend.holds(&item("A", "D")));
    assert_eq!(blend.items().count(), 2);
    assert_eq!(blend.arity(), 2);
    assert_eq!(Hypothesis::none_of_these().arity(), 0);
}
