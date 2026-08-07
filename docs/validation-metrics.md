# What counts as reproducing an assignment

Decision record for issue #47. These metrics are written before the first
validation number exists, because afterwards there is always a metric under
which the result looks better, and choosing one then is not measurement.

No validation result has been produced in this repository. That is checkable
rather than asserted:

    git log --oneline -- docs/validation/
    (no output)

The same command run later shows whether this file preceded the first result.
Nothing in this repository refuses a result document that omits a metric defined
here; the rule below is carried by review and by the reporting requirement in
issue #49, and saying that is the disclosure rather than a plan.

## Notation

Write F for the features the run was given in a validation spectrum. For each
feature f the run reports a winner w(f), which is a hypothesis in the sense of
`docs/decisions/competitors.md`, and a probability p(f).

Write R for the published reference assignment, a partial map, so dom(R) is the
set of features the reference assigns and F minus dom(R) is the set it leaves
alone. Write H for the set of features whose true level the hold-out procedure
withheld, defined by issue #45. Two hypotheses are equal when their structural
keys are equal, which is the multiset of upper levels together with the multiset
of lower levels.

At a probability threshold t:

    Claimed(t)  = { f in F : w(f) is not none-of-these and p(f) >= t }
    Agree(t)    = { f in Claimed(t) and f in dom(R) : w(f) = R(f) }
    Disagree(t) = { f in Claimed(t) and f in dom(R) : w(f) != R(f) }
    Outside(t)  = { f in Claimed(t) : f not in dom(R) }

Every metric is reported as a fraction and with its numerator and denominator,
never as a percentage alone. A recovery of 0.80 over five features and one over
five hundred are different results and read identically when the counts are
dropped.

## The metrics

Recovery.

    recovery(t) = |Agree(t)| / |dom(R) minus H|

The share of the published assignments still available to be recovered that the
engine recovers at t. Features whose level was withheld are removed from the
denominator, because recovering them is not what the hold-out asked for.

Disagreement.

    disagreement(t) = |Disagree(t)| / (|Agree(t)| + |Disagree(t)|)

The share of the engine's claims about referenced features that contradict the
published assignment. This is the number that matters most and the one a
recovery figure hides, since an engine can raise recovery by claiming more and
carry every extra error along with it.

Claims the reference does not cover.

    outside_rate(t) = |Outside(t)| / |Claimed(t)|

Features the reference left alone that the engine assigns anyway. This is not
automatically an error. A published compilation may have left a line unassigned
for reasons that have nothing to do with whether it is assignable. It is
reported because a rising outside rate with a flat recovery is the signature of
an engine filling crowded regions with accidental matches, which is the most
damaging failure available here.

Reach on withheld levels.

    reach(t) = |{ f in H : f in Claimed(t) }| / |H|

Among features whose true level was withheld, the share where the engine reached
for a neighbour instead of declining. The only right answer for these features
is to decline, so this metric is reported rather than its complement: the
quantity of interest is how often the engine was wrong, not how often it was
right.

Calibration.

Over the features whose truth is known, meaning dom(R) together with H, with the
truth for a feature in H being the none-of-these hypothesis where the withheld
level was that feature's only owner, and the feature excluded from calibration
otherwise. Bin the reported probabilities into the ten equal-width bins of
[0, 1]. For bin b let n_b be the count, acc_b the share of those features where
the winner equals the truth, and pbar_b the mean reported probability in the bin.

    ECE = sum over b of (n_b / N) * |acc_b - pbar_b|

Report the ten bins with their counts alongside the single number, because an
ECE computed over bins that are mostly empty is not a measurement of anything.
Report in addition the Brier score and the mean negative log score over the full
per-feature probability distribution, not only over the winner, since a model
can be well calibrated on its winners and badly wrong about how it splits the
rest.

## The thresholds, and why

Every metric above except calibration is a function of t, and all of them are
reported over the grid

    t in { 0.50, 0.60, 0.70, 0.80, 0.90, 0.95, 0.99 }

so a reader sees the curve rather than a chosen point. A single number invites
the reader to believe the engine has one operating point, and it does not.

Two are headlined. At t = 0.50 the winner carries more probability mass than
every alternative combined, including none-of-these, which is the weakest
threshold under which the phrase "the engine's answer" means anything at all.
At t = 0.90 the engine is promising about one error in ten, which is the point
where the calibration claim is most directly checkable against the disagreement
rate measured beside it.

Neither is a recommended acceptance threshold and neither is shipped as a
default. Whether this board ships an acceptance default is entry 5 of issue #1
and is not decided here or anywhere else yet.

## Disagreement with a published assignment

Published assignments are sometimes wrong. That is the premise of this project,
so a rule that counted every disagreement as an error would make the project
unable to state its own case, and a rule that let any disagreement be waived
would excuse anything.

The rule. A disagreement counts as an error. It stops counting as an error only
when it has been argued individually, in writing, in its own file under
`docs/validation/disagreements/`, naming the feature, the published assignment,
the engine's assignment, and the evidence for preferring the second. A category
of disagreements cannot be argued in one file; the unit is one feature.

Every result document reports:

- the metric with no argument admitted, which is the honest default
- the metric with the admitted arguments removed from the error count
- `arguments_admitted`, the count of files admitted, and their identifiers

Both numbers, always, in that order. A result document carrying only the second
has turned a disclosure into an assurance.

The argument may not be written after seeing the metric it improves. What makes
that checkable is the history: the commit adding an argument file precedes the
commit adding the result document that admits it, and

    git log --oneline --diff-filter=A -- docs/validation/disagreements/

lists them with their order. No check in this repository refuses a result
document that admits an argument committed after it. That gap is real and
stating it here is what this record does about it today.

## What every later result document owes

Every metric defined above, at every threshold in the grid, with numerators and
denominators, including the ones that came out badly. A result document that
reports recovery and omits disagreement is not a shorter report, it is a
different claim.

Where a metric could not be computed, the document names it and says why, and
that is not the same as omitting it. A metric absent without a reason is read as
a metric that was computed and not liked.

Every number names the hold-out procedure it came from, from the three that
issue #45 defines, as a field in the result file rather than as a sentence in
prose. The three procedures measure different things and a number quoted without
the procedure that produced it is not interpretable.
