# What an assignment is

Decision record for issue #11. The tempting model is a function from observed
lines to level pairs, and it is wrong in three ways that each cost something
specific. A function must choose one owner for a feature that has two. A total
function must give an owner to a feature that has none, and it does that most
confidently in the crowded regions where an accidental match is likeliest. And a
function defined per spectrum throws away the strongest consistency evidence
available, which is the same transition seen twice.

So the object is not a function. This record says what it is instead.

Nothing in this repository refuses a violation of this record today, because
there is no source tree here yet:

    git ls-files 'crates' 'crates/*' '*.rs' 'Cargo.toml' ; echo "exit=$?"
    exit=0

That is also why issue #11 is not closed by this file. Its Done-when asks that
the type in the tree match the document and be greppable, and there is no type.
Issue #40 is where the probability becomes a field on every hypothesis, issue
#30 is where the destructive test below lands, and issue #43 is where the shapes
here become a schema.

## The two objects loose talk calls an assignment

They are different and the record keeps them apart, because almost every
confusion in this area is one being used where the other was meant.

A configuration is a choice of exactly one hypothesis for every feature in the
run, taken together, satisfying the exclusion rules below. It is the object the
solver searches over and the object the posterior of
`docs/decisions/probability-model.md` is defined on. There are combinatorially
many of them and the answer file contains none.

A feature's hypothesis set is every hypothesis that feature could have, each
carrying the marginal probability of the configurations in which it holds. It is
the object the answer file reports, one per feature, and it is what a reader
means by "the assignment of this line".

The winner of a feature is the highest-probability member of its hypothesis set.
A run does not report a configuration, and the winners of two features are not
guaranteed to be jointly consistent. That is not a defect being hidden: it is
what a marginal is, and where two winners conflict the run says so rather than
silently editing one of them. The rule is at the end of this record.

## The hypothesis

A transition is an ordered pair of levels from the offered level set together
with the multipole the candidate was generated under, so `(upper_level_id,
lower_level_id, multipole)`. Two transitions are the same transition when all
three agree.

A hypothesis for a feature is a set of distinct transitions. The set is what
holds: a hypothesis containing one transition twice would be claiming that one
transition produced part of a feature and then produced it again, which is not a
statement about anything, and it is refused rather than deduplicated silently.

The structural key of a hypothesis is a pair of multisets, the upper level
identifiers of its transitions and the lower level identifiers of its
transitions. Multisets rather than sets, because two distinct transitions can
share an end. Two lines from one upper level to two different lower levels give
a key whose upper multiset holds that identifier twice, and that is a different
claim from a blend of two transitions with two different upper levels. This is
the key `docs/decisions/competitors.md` selects competitors by and the key
`docs/validation-metrics.md` compares against a published assignment.

The size of a hypothesis is the number of transitions in it.

| Size | Name | What it says |
| --- | --- | --- |
| 0 | none-of-these | No transition in the offered level set produced this feature |
| 1 | a single assignment | Exactly one transition produced it |
| 2 or more | a blend | The feature is unresolved and these transitions together produced it |

The largest size generated is `prior.max_components`, default 2, which is the
cap in `docs/decisions/probability-model.md`. A feature that is really a blend of
three is therefore not offered its true hypothesis, and what wins for it is
either a two-component subset or none-of-these. The cap is in the run's
provenance, so a reader can see which it was, and the measurement that would
move it is the share of features a solved spectrum resolves into three or more.

## The none-of-these hypothesis

It exists for every feature, always. Not for features that failed to acquire a
candidate, not as a fallback the run inserts when the score is poor, and not
under a switch. It is a member of every hypothesis set the run builds, including
the sets of features with a dozen good candidates.

It cannot be removed. The hard rules of `docs/decisions/selection-rules.md`
remove candidates, and a hypothesis of size zero has nothing for a rule about
parity or J to be evaluated against, so no rule reaches it. The tolerance that
removes a candidate whose position lies too far from the feature likewise has
nothing to act on. It is not subject to the reporting floor of
`docs/decisions/competitors.md`, and its probability is reported whether it wins,
loses narrowly or loses by a mile.

It can win, and winning is a normal outcome rather than a failure of the run. It
wins whenever the posterior puts more mass on it than on any assignment, which
happens for impurity lines, for molecular bands, for other ionisation stages, for
instrumental ghosts, for genuinely unknown species, and for real transitions
whose multipole this release does not generate. `docs/decisions/selection-rules.md`
names that last case explicitly: everything above E1, M1 and E2 is out of the
first release, and a feature whose only explanation is a higher multipole is
reported unassigned rather than forced onto an E1 pair.

The property that keeps this from being prose is destructive and belongs to
issue #30. Remove from the level set a level the truth needs, and the features
that depended on it come back as none-of-these rather than reassigned to the
nearest surviving pair. A run that passes every other test and fails this one has
the failure this record was written against.

## The blend

A blend is a hypothesis of size two or more. It is one recorded feature with more
than one owner, and it is not a defect in the data.

Its components share the observed intensity by summing, and the sum is the whole
rule. `docs/decisions/intensities.md` predicts each transition's latent log
photon flux from the model side. For a blend, the feature's predicted flux is the
sum of the fluxes of its component transitions, and the order constraint that
recorded intensity supplies applies to that sum. No component of a blend is ever
handed the feature's recorded intensity, and no component is handed a share of it
either. There is no splitting rule, no equal division and no division by
predicted strength, because the recorded number belongs to the feature and any
division of it would be an inference presented as an input.

The branching constraint reaches a blended feature only through the same sum,
which usually means it is not applied at all: a sum of branches from different
upper levels constrains none of them individually.

Two blends are recorded differently and the difference is a field rather than a
footnote. A feature the source itself flagged `blended` in the line list was
known to be a blend before the run. A feature the run assigned a blend hypothesis
to was proposed as one by the run. Both are treated identically by the model and
they are different evidential situations, so `docs/decisions/intensities.md`
requires them to be distinguishable in the output and this record requires the
hypothesis to carry which it is.

The prior on the blend rate is Beta(1, 9), informative and deliberately so, and
it is in `docs/decisions/probability-model.md` rather than restated here. The
reason it is informative is this record's concern: a blend hypothesis is always
available and always able to absorb a residual, so an uninformative prior lets it
win everywhere and every crowded region fills with two-component explanations.

## Mutual exclusion

Two hypotheses are mutually exclusive when no configuration may contain both.
There are two sources of it and one case that looks like a third and is not.

Within one feature. The hypotheses of a single feature are mutually exclusive by
construction, since a configuration chooses exactly one of them. This is why the
probabilities over the hypotheses of one feature sum to one, which is the
statement `docs/decisions/competitors.md` already makes.

Across features in one spectrum. Two hypotheses belonging to different features
of the same spectrum are mutually exclusive when they share a transition. One
transition of one species in one spectrum produces one feature, so a
configuration in which two features both claim it is claiming that one transition
was recorded twice. A blend inherits this from each of its components: a
two-component hypothesis holding transitions t and u excludes every hypothesis of
every other feature in that spectrum that holds t, and every one that holds u,
separately.

This is a hard constraint on the configuration space rather than a penalty, and
it is not a rule about levels. Two features of one spectrum may share an upper
level, which is the ordinary case and the one the branching constraint exists
for, and they may share a lower level. What they may not share is a whole
transition.

Across spectra it is not exclusion at all. The same transition appearing in two
spectra is one transition observed twice, at two intensities, and that is the
consistency evidence a per-spectrum model throws away. It is carried by
transition identity: the transitions of two hypotheses in different spectra are
the same transition when the triple agrees against one `level_set_id`, and the
posterior treats that as shared support rather than as a conflict. The
identifier of the level set is load bearing here, because two runs against
different level sets share no transition identities even where the level
energies are close.

The case the exclusion rule gets wrong is stated rather than left to be
discovered. A transition resolved into hyperfine or Zeeman components appears in
the line list as several features, all genuinely produced by one transition, and
the rule above forces at most one of them to own it. The others are pushed toward
none-of-these, so the run under-assigns exactly where the data is best resolved.
The run reports every feature whose winner changed because of the exclusion rule
and the feature it lost to, so the cost is visible rather than absorbed. What
would replace the rule is an input contract able to say that two features are
components of one transition, which `docs/decisions/input-contract.md` does not
carry today, and that is where the repair belongs rather than here.

## Where two winners conflict

Marginals are reported per feature, so two features in one spectrum can each have
a winner and the two winners can share a transition, which no configuration
allows. The run does not resolve this by editing one of them.

It reports both winners with their probabilities, and beside each it names the
conflict, the other feature, and the shared transition. A reader is then looking
at the situation the data actually presents, which is two features competing for
one transition with the mass split between them, and that is more informative
than a run that silently demoted the second and printed a clean table.

## The shapes, written out

Every identifier and every number below is illustrative. No run produced them,
because there is nothing here to run. They are here because issue #11 asks for an
example of each shape and because a shape argued in prose alone is a shape two
readers will implement differently.

A single assignment.

    feature_id      F0117
    spectrum_id     S1
    winner
      size          1
      components    (L0042, L0009, E1)
      key           upper {L0042}  lower {L0009}
      probability   0.83
    none_of_these
      probability   0.04

A blend of two, proposed by the run rather than flagged by the source.

    feature_id      F0233
    spectrum_id     S1
    winner
      size          2
      components    (L0061, L0012, E1), (L0074, L0028, E1)
      key           upper {L0061, L0074}  lower {L0012, L0028}
      blend_origin  proposed_by_run
      intensity     the order constraint applies to the summed flux of both
                    components; neither component is given the recorded value
      probability   0.57
    none_of_these
      probability   0.11

A blend of two from one upper level, which is the case the multiset key exists
for. Its upper multiset holds one identifier twice, and it is a different
hypothesis from any blend with two different upper levels.

    feature_id      F0301
    spectrum_id     S1
    winner
      size          2
      components    (L0088, L0015, E1), (L0088, L0016, E1)
      key           upper {L0088, L0088}  lower {L0015, L0016}
      probability   0.44

None-of-these winning.

    feature_id      F0402
    spectrum_id     S1
    winner
      size          0
      components    none
      key           upper {}  lower {}
      probability   0.68
    best_alternative
      size          1
      components    (L0103, L0055, E2)
      probability   0.19

One transition in two spectra, which is not exclusion.

    feature_id      F0117   spectrum_id  S1   winner  (L0042, L0009, E1)  0.83
    feature_id      G0088   spectrum_id  S2   winner  (L0042, L0009, E1)  0.79
    relation        same transition identity against level_set_id LS-1;
                    shared support, no exclusion, the two intensities are
                    ordered within their own spectra and never against
                    each other

Two features of one spectrum claiming one transition, which is.

    feature_id      F0510   spectrum_id  S1   winner  (L0120, L0033, E1)  0.51
    feature_id      F0511   spectrum_id  S1   winner  (L0120, L0033, E1)  0.46
    relation        mutually exclusive; no configuration holds both
    reported        both winners, both probabilities, the conflict named on
                    each, and the shared transition

## What the type owes when it lands

Issue #11 asks that the type in the tree match this document and that the match
be greppable. What that means, so that whoever writes the type is not left to
infer it.

A hypothesis holds a set of transitions, not a list and not an optional single
transition, so that size zero, one and many are one type with no special case and
none-of-these cannot be represented by an absent value. The structural key is
computed from the components rather than stored beside them, so the two cannot
disagree. Mutual exclusion is a property of a pair of hypotheses that the type
can answer, rather than a rule the solver reimplements. And a feature's
hypothesis set includes the size-zero member at construction, so there is no
code path that builds a set without it.

Per `docs/decisions/layout.md`, all of that is `assoc-model`'s, in the generic
vocabulary of items and slots, and the words upper, lower and multipole belong to
the crate that instantiates it.
