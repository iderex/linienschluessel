# The probability model, and what a number like 0.80 is a probability of

Decision record for issue #12. The sentence this board exists to be able to
print is that an assignment is eighty per cent likely to be the right one, and
that sentence means nothing until the sample space is written down. This record
writes it down, names the construction, lists every prior and nuisance parameter
the construction needs, and fixes the words an answer is allowed to use.

Nothing in this repository refuses a violation of this record today. Issue #40
is where the probability becomes a field on every hypothesis, issue #42 is where
it is measured against outcomes, and issue #43 is where the wording below
becomes the schema an answer is written to.

## The sample space, in one sentence

For one observed feature, the sample space is the finite set of hypotheses that
`docs/decisions/competitors.md` defines for it, which is every multiset of
transitions the offered level set generates and the hard rules of
`docs/decisions/selection-rules.md` keep, together with the none-of-these
hypothesis, and a probability on it is conditional on that level set, that line
list, the tolerance rule the run used and the priors this record names.

Three things in that sentence are load bearing. The space is per feature. It
contains the empty hypothesis, so it is never a space of assignments only. And
it is conditional on the inputs, which is why a probability from this board is
not a probability that a transition exists in nature.

## The construction

A joint Bayesian posterior over the assignment of the whole spectrum, reported
per feature as a marginal.

Joint rather than per feature, because the features are not independent and
pretending otherwise would discard the evidence this board is built on. Two
features from a shared upper level constrain each other through the branching
constraint of `docs/decisions/intensities.md`. Every feature in one source
shares the calibration terms of `docs/decisions/uncertainty-model.md`. And the
fraction of features with no owner in the offered level set, defined below, is
one quantity for the spectrum rather than one per feature.

The generative story, which is what a reimplementation needs.

Each source carries the two shared position terms and the isotonic intensity fit
that the uncertainty and intensity records define. Each spectrum carries a
fraction u of features that no transition in the offered level set produces.
Each feature is, with probability u, unowned, and its position is then a draw
from the empirical density of positions in its region; otherwise it is produced
by a hypothesis drawn from a prior over multiset size and over the transitions
themselves, where the transition prior is the multipole prior and the weighted
rules of the selection-rules record, in the log-odds those tables give. Given a
hypothesis, the position likelihood is the residual under the uncertainty model,
and the intensity likelihood is the order constraint and, where its four
conditions hold, the branching constraint.

The posterior is over the whole configuration. The per-feature number an answer
reports is the marginal of that posterior, and the marginal is not exact: the
configuration space is combinatorial and no exact marginalisation is available
at the size of a real spectrum. What is reported is therefore a computed
approximation with a stated procedure and a stated error, which is issue #38's
work and not a detail this record can hide. An answer that prints a marginal
without the error of the procedure that produced it has made a claim its own
machinery does not support.

## Where the accidental rate enters, and why the list getting shorter is not enough

Issue #12 names the trap precisely. A probability that renormalises over the
enumerated candidates goes up when the list gets shorter, and the list is
shortest where the level set is most incomplete, so every unidentified feature
in a partially analysed spectrum would be reported with high confidence against
whichever pair happened to be nearest.

The construction above avoids it in two separate places, and both are needed.

The likelihood side is calibrated by the null of
`docs/decisions/chance-coincidence.md`. The position evidence a candidate
carries is how surprising the match is, and a match is surprising only in
proportion to how rarely a displaced feature acquires a candidate in that region
at that tolerance. That is the measured rate, per region, per run. A sparse
level set makes a match more surprising, which is correct on its own terms and
is not by itself the answer to the trap.

The prior side is where the trap is actually closed. The fraction u is not a
constant and is not assumed small. It is a nuisance parameter fitted per
spectrum under the prior below, so a level set that explains few features well
drives u up, and a large u puts mass on none-of-these for every feature at once,
including the ones with a nearby pair. The two effects pull against each other
on purpose: the likelihood says this particular match is unlikely to be an
accident, the prior says this particular spectrum is full of features nothing
here explains, and the reported number is where they settle.

The property that keeps this from decaying back into a renormalisation is
checkable and is stated as such. For any two hypotheses of one feature, the ratio
of their probabilities depends on those two hypotheses alone and not on which
other hypotheses are in the list. An implementation that computes a score per
candidate, divides by the sum over candidates, and then adds a none-of-these
value afterwards violates it, and violates it in the flattering direction. Issue
#40 carries the test.

The second property is destructive and it is issue #30's: remove from the level
set a level the truth needs, and the features that depended on it come back
unassigned rather than reassigned to the nearest surviving pair. That test is the
one that would catch this record being right on paper and wrong in the code.

## The priors and nuisance parameters

Every default below is an explicit prior. None of them is a measurement, none
was fitted to any spectrum, and each names what would replace it.

| Quantity | Switch | Default | What would replace it |
| --- | --- | --- | --- |
| Prior on u, the unowned fraction of a spectrum, as a Beta | `prior.unowned.alpha`, `prior.unowned.beta` | 1.0, 1.0 | The measured unassigned fraction across the validation spectra of #48 |
| Prior on the blend rate among owned features, as a Beta | `prior.blend.alpha`, `prior.blend.beta` | 1.0, 9.0 | The measured blend fraction in a solved spectrum at a stated resolution |
| Cap on hypothesis size | `prior.max_components` | 2 | The measured share of features a solved spectrum resolves into three or more |
| Number of posterior samples behind a reported marginal | `posterior.samples` | 4000 | A convergence measurement on a real spectrum, which is #38 |
| Reported marginal error target | `posterior.marginal_tolerance` | 0.01 | The same measurement |

Beta(1, 1) on the unowned fraction is uniform, which is the deliberate choice
rather than a placeholder. Any informative prior here is a statement about how
complete somebody else's level set is, and this board is not in a position to
make one. Beta(1, 9) on the blend rate has a prior mean of 0.1 and is
informative, because a blend hypothesis that is always available and always
cheap wins everywhere, which is the failure issue #28 names.

Both fitted values reach the answer file, u per spectrum and the blend rate per
spectrum, each with its posterior interval. A run whose fitted u is 0.6 has said
something important about the pairing of that level set with that spectrum, and
it has said it in a field rather than in a mood.

Everything else the model needs is defined in another record and is not restated
here, because a table copied between two files drifts against the one that
decides. The multipole priors and the weighted selection rules are in
`docs/decisions/selection-rules.md`. The distributions, the correlation default
and the two per-source calibration terms are in
`docs/decisions/uncertainty-model.md`. The intensity terms and their thresholds
are in `docs/decisions/intensities.md`. The reporting floor is in
`docs/decisions/competitors.md`.

## The constructions that were rejected

A posterior over the enumerated candidates alone, conditional on the truth being
among them. It is the easy answer and it is the one this record exists to
refuse. It is the whole trap above, and its failure is not a rare corner: it is
worst exactly in the partially analysed spectra the board is pointed at.

A likelihood ratio against the accidental-match null, reported instead of a
probability. It needs no prior, which is its attraction, and it answers a
different question: how surprising the match is, not how likely the assignment
is. The two get confused in one direction only, the flattering one, because a
large ratio reads as a large probability to almost every reader. It is not
discarded. The ratio is a good statistic and it is computed and reported, as
part of what the null of `docs/decisions/chance-coincidence.md` produces. It is
not the number that carries the word probability.

A bootstrap over resampled inputs. It measures how stable an assignment is
against resampling, which is worth having and is not a claim about correctness:
a systematically wrong assignment is perfectly stable. It stays available as a
diagnostic and is not the reported number.

A conformal construction. It would give a coverage guarantee under
exchangeability, which is the attraction, and exchangeability is not credible
here. Features share calibration terms, share upper levels, and cluster where
levels are dense, and the whole of the uncertainty record is an argument that
they are not exchangeable. A guarantee whose condition is known to fail is worse
than no guarantee, because it is quotable.

## The words an answer is allowed to use

The number and the claim made about it are fixed together here so they cannot
drift apart. An answer file states a probability in this form and no other:

    Given the level set <level-set-id>, the line list <line-list-id> and the
    priors recorded in this run, the probability that feature <feature-id> is
    <hypothesis> rather than any other hypothesis, including that no transition
    in this level set produces it, is <p>. The probability that no transition in
    this level set produces it is <p_none>.

Both sentences, always, together. The second is not a footnote to the first and
is not omitted when it is small, which is the rule
`docs/decisions/competitors.md` already sets and this record repeats because it
is a rule about wording and this is the wording.

Four phrasings are refused by name.

"Eighty per cent likely to be correct", because correct is a claim about nature
and this number is conditional on one level set.

"Eighty per cent confidence", because confidence is a different construction
with a different guarantee and using its word for this number invites the
guarantee to be assumed.

Any sentence that gives p without p_none.

Any sentence that gives p without naming the level set and the line list it is
conditional on. A probability from this board quoted alone in a methods section
is a probability that has lost the only thing that made it interpretable, and
the run record of issue #19 exists so that the identifiers are short enough to
carry.

The report a person reads and the file a machine reads use the same words for
the same quantity, which is issue #43. Where a rendering is too narrow for the
sentence above, it carries the identifiers and the two numbers and nothing else
is dropped.
