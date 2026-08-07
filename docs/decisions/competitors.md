# What a competing assignment is, and how many are reported

Decision record for issue #17. An assignment returned with no competitor has
hidden the information a spectroscopist needs to judge it. A winner at 0.55 with
its best rival at 0.44 and a winner at 0.55 with its best rival at 0.03 are
different situations, and printing only the winner makes them look identical.

Nothing in this repository refuses a violation of this record today. Issue #41
is where the competitor becomes a field in the output that cannot be left empty,
and issue #43 is where the schema that carries it is defined.

## Vocabulary

The unit is the feature: one thing measured in one spectrum, which may be a
single line or an unresolved blend. Issue #11 defines the hypothesis object; this
record uses it as follows.

A hypothesis for a feature is a multiset of transitions. A hypothesis of size
one is a single assignment. A hypothesis of size two or more is a blend. The
hypothesis of size zero is the none-of-these hypothesis, which says the offered
level set did not produce this feature.

Every hypothesis carries a probability, and the probabilities over the
hypotheses of one feature sum to one. The winner is the hypothesis with the
largest probability.

## What is reported alongside every assignment

Three things, always, for every feature the run reports.

The none-of-these probability. It is reported whether it wins, loses narrowly or
loses by a mile, and it is never omitted for being small. It is not selected by
any rule and is not subject to any floor, because it is the one alternative that
is always available and the one whose suppression would be most damaging. A
feature assigned at 0.72 with none-of-these at 0.26 is a different claim from
the same 0.72 with none-of-these at 0.01.

The best upper-different competitor and the best lower-different competitor.
These answer different questions. A reader who doubts the upper level wants the
best alternative that moves it; a reader who doubts the lower level wants the
best alternative that moves that. A single ranked list answers neither cleanly,
because the runner-up is usually whichever direction happened to be closer.

Any further alternative above the reporting floor, deduplicated by the
structural key below. The floor is `report.competitor_floor`, default 0.01, and
it is stated in every answer file rather than assumed by the reader. This list
is often empty and that is not a failure.

## Why not a fixed number of runners-up, and why not a floor alone

A fixed list of four is arbitrary in a way that hurts here. The second, third
and fourth candidates are frequently the same physical alternative expressed
against three nearby levels, so a list of four can hold one real competitor and
three copies of it, and the reader counts four and concludes the assignment is
badly contested when it is contested once.

A floor alone produces a variable-length list, which is honest about how much
competition exists and leaves every downstream reader handling a list that is
sometimes empty. The floor stays, as the third item above, but it is not the
whole rule, because it can return three near-duplicates and no alternative in
the direction the reader cares about.

The two structural competitors are the harder thing to define and the more
informative one, so they are defined here rather than left out for being
awkward.

## Structural difference

Write U(H) for the multiset of upper levels of the transitions in hypothesis H,
and L(H) for the multiset of lower levels. Multisets, not sets, so a blend that
uses the same upper level twice is distinguished from one that uses it once.

A hypothesis H' is upper-different from the winner H when U(H') is not equal to
U(H) as a multiset. It is lower-different when L(H') is not equal to L(H). A
hypothesis may be both, and the same hypothesis may therefore be reported in
both slots; where that happens the output says so rather than printing it twice
as though two rivals had been found.

For blends this is the whole definition and needs no special case. Swapping one
component of a two-transition blend changes one entry of the multiset, so the
result is upper-different, lower-different or both, according to which end of
that component moved. Dropping a component from a blend, or adding one, also
changes the multiset and is likewise a structural difference. This is deliberate:
"the same assignment plus a second weak component" is a genuinely different
claim about the feature and a reader is entitled to see it as a rival rather
than as a variant of the winner.

For the none-of-these hypothesis, U and L are both empty. So when the winner has
at least one transition, none-of-these is upper-different and lower-different at
once, by the definition and without an exception being written for it. It is
still reported in its own slot rather than in either competitor slot, because a
reader looking for the best rival assignment is not helped by being handed the
statement that there is no assignment, and because it is reported unconditionally
while the competitor slots are not.

When the winner is the none-of-these hypothesis, every rival has at least one
transition and is therefore both upper-different and lower-different, so the two
slots collapse to the same hypothesis. The output carries it once, in both
slots, with the collapse stated. The alternative, suppressing one slot, would
make a reader think no such rival existed.

Deduplication in the floor list uses the pair (U(H), L(H)) as the structural key.
Two hypotheses with the same key are the same structural claim, and only the
highest-probability one is listed, with the count of the ones it stands for.

## The fields a competitor carries

A competitor carries the same fields as the winner, with no reduced form. A
competitor printed as a bare identifier is a competitor nobody can weigh, and
sending the reader back to the input to weigh it defeats the point of reporting
it at all.

For each hypothesis, winner or competitor:

- the structural key, as the multiset of upper levels and the multiset of lower
  levels
- for each transition in it: the upper level and lower level identifiers, the
  energies and declared uncertainties as the run used them, the parity and J of
  both, and the multipole the candidate was generated under
- the predicted position, the observed position, and the residual between them
  in the run's internal representation
- the probability, and the score with its per-term breakdown, so the reader can
  see which term separated this hypothesis from the winner
- the branching form applied to it, or the reason none was
- every hard rule that could not be evaluated on it, and every weighted rule that
  fired with the weight it contributed
- which competitor slot it occupies, and for the floor list, how many
  structurally identical hypotheses it stands for

The score breakdown is the field that makes the report useful rather than
decorative. Two hypotheses at 0.51 and 0.47 separated entirely by the position
term, and two separated entirely by an intercombination prior, are different
situations, and the second is one a reader may reasonably overturn by hand.

## When there is no competitor at all

This is a real case. A feature may generate exactly one transition hypothesis,
so nothing is upper-different or lower-different except none-of-these, which has
its own slot. Or every alternative may fall below the floor.

It is never reported as an absent field and never as an empty list on its own.
The competitor slot carries an explicit statement with one of these reasons:

- `no_alternative_generated`, when candidate generation produced no other
  hypothesis differing in that direction
- `all_below_floor`, when alternatives exist but none reached
  `report.competitor_floor`
- `removed_by_hard_rule`, when alternatives existed before the hard rules of
  `docs/decisions/selection-rules.md` removed all of them in that direction

Each carries the count of hypotheses that were generated for the feature, the
count that survived the hard rules, and the floor in force. "No competitor" is
then a measured statement about the spectrum, which is what it should be, rather
than silence that a reader will read as either confidence or a bug.
