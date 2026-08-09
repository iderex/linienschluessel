# The hold-out protocol, and the circularity it has to break

Record for issue #45. `docs/validation-metrics.md` fixed what counts as
reproducing an assignment before the first number existed. This file fixes how
the run that produces such a number is set up, which is the other half and the
one where the whole exercise can be invalidated without any metric looking
wrong.

No validation run has been performed in this repository, and there is still
nothing here to perform one with. There is a source tree now, and what it holds
is the input reader. Read at a23385a:

    git grep -l '' -- '*.rs' | cut -d/ -f2 | sort -u
    spectro-contract

    git ls-files '*/src/main.rs' 'crates/*/src/bin' ; echo "exit=$?"
    exit=0

One crate of thirteen holds any code and there is no binary target to invoke, so
the sentence changed and the state it describes did not. A run needs the
candidates, the objective, the solver and the posterior, and every crate
`docs/decisions/layout.md` puts those in is empty.

Issue #45 is not closed by this file. Its Done-when asks for a command that runs
each procedure and for a test over a small fixture whose truth is known by
construction, and it asks that every reported validation number name its
procedure as a field in the result file. The first two need a program. The third
is a requirement this file states and nothing here enforces.

## The trap

Published level energies are usually not independent of the lines being
assigned. They are the output of a least-squares optimisation over exactly those
observed wavelengths, which is what the program LOPT does and how the Ritz
wavelengths and their uncertainties in the standard compilations are produced,
described in Comput. Phys. Commun. 182, 419 (2011).

That paragraph is the premise issue #45 supplies, with the reference it supplies,
and this protocol is built on it. No reading of that paper was made here, and
saying so is the difference between a citation and a claim.

What follows from it is arithmetic rather than opinion. Feed those levels and
those lines into an assignment engine and the residuals will be tiny for the true
assignment, because the level energies were chosen to make them tiny. An engine
that recovers the published assignment under those conditions has demonstrated
that it can find the minimum of a function somebody else already minimised. The
recovery figure will be high, every metric in `docs/validation-metrics.md` will
be computable, and none of them will be measuring what the reader thinks.

So the protocol has to break the circle, and no single procedure breaks it
cleanly. Three do, partially and in different directions, and each is named at
every number it produces.

## The three procedures

### Line hold-out with re-optimisation

Withhold a subset of the observed lines. Re-optimise the level energies from the
lines that remain. Assign the withheld lines against the re-optimised level set.

What it measures. Whether the engine recovers a published assignment when the
answer to that particular assignment has not been folded into the energies it is
scored against. This is the closest of the three to the situation of a
spectroscopist extending an existing analysis, which is the commonest real use.

What it cannot measure. It does not break the circle, it narrows it. The level
still exists because the original analysis found it, and its parity and its J
still come from that analysis. A level supported mostly or entirely by the
withheld lines becomes badly determined or undetermined when they are removed, so
the problem changes rather than being held fixed, and the withheld set therefore
has to be chosen so that every level it touches retains enough support. That
choice is itself a decision the result document states.

It also has a prerequisite this board does not have. Re-optimising level energies
is a least-squares fit over the retained lines, and there is no level
optimisation here and no open issue that owes one. That was checked rather than
assumed:

    gh issue list --state all --limit 200 --json number,title,body \
      --jq '.[] | select((.title + " " + .body)
            | test("optimis|optimiz|LOPT|least.squares"; "i"))
            | "#\(.number) \(.title)"'
    #45 The hold-out protocol, and the circularity it has to break
    #32 Position agreement, weighted by the uncertainties that were declared
    #31 The objective, in one document and one module
    #25 Pin the snapshot an answer was computed from
    #14 Decide and record the uncertainty model, and what is correlated with what
    #2 Record the choice of implementation language and toolchain, with the reasons

Run 2026-08-08. Six issues mention the words and none of them owes a level
optimiser: #45 is this one, and the other five use the words about the position
term, the objective, snapshot pinning, the uncertainty model and the choice of
language. So this procedure cannot be run until an issue is opened for the
optimiser and it lands, and until then a validation report that claims this
procedure has claimed something that does not exist.

Whether that optimiser is the same machinery as proposing a level, which is
entry 3 of issue #1 and the maintainer's to answer, is a real question and this
file does not settle it. Re-fitting the energies of levels somebody else
identified is a smaller thing than proposing a level nobody has, and the two are
close enough that answering the entry one way may decide this.

### Predicted levels

Replace the optimised level set with levels from a structure calculation for the
same species, and assign the full observed line list against those.

What it measures. The situation this board is actually aimed at, where the lines
are unidentified precisely because no optimised level set exists for them. It is
the honest analogue, and it is the one whose result should be believed when the
three disagree.

What it cannot measure. Recovery at anything like the accuracy the published
levels have. Predicted levels sit far from observed ones, so the per-configuration
shared offset that `docs/decisions/uncertainty-model.md` fits is carrying much of
the work, and a poor result is ambiguous between an engine that failed and a
structure calculation that was wrong in that region. The result document
therefore reports the fitted offsets and their posterior intervals per
configuration beside the metrics, so a reader can see which of the two is being
measured.

It needs a predicted level set as an input. `docs/decisions/input-contract.md`
already carries `origin` with values `measured` and `predicted` for exactly this,
so the input side is defined and no contract change is owed.

### Whole-level hold-out

Remove one or more levels entirely from the level set, keeping every observed
line in the line list, including the lines those levels own.

What it measures. Whether the engine declines. The only right answer for a
feature whose true level was withheld is the none-of-these hypothesis of
`docs/decisions/what-an-assignment-is.md`, and the metric is `reach(t)` in
`docs/validation-metrics.md`, reported as how often the engine reached for a
neighbour rather than how often it did not. This is the procedure that tests the
failure this board says is the most damaging one available to it.

What it cannot measure. Anything about recovery, since the assignments it is
asking about have had their answer removed. It also perturbs the thing the null
of `docs/decisions/chance-coincidence.md` measures: removing levels changes the
local level density, so the accidental match rate in the affected regions moves,
and the null is re-measured under the modified level set rather than carried over
from the full one. A report that quotes an accidental rate from the unmodified
set beside a reach figure from the modified one has compared two different
spectra.

### The fourth setting, which is not a procedure

A run against the full published level set with nothing withheld is the setting
the trap describes. It is worth performing, because it is the upper bound the
other three are read against, and it is not a validation of anything. It carries
`holdout_procedure` with the value `none` so that a number produced under it
cannot be quoted as though it came from one of the three.

## Naming the procedure at every number

`docs/validation-metrics.md` already requires that every number name the
procedure it came from, as a field in the result file rather than a sentence in
prose. The field is `holdout_procedure` and its vocabulary is exactly four
values:

| Value | Procedure |
| --- | --- |
| `line_holdout_reoptimised` | Line hold-out with re-optimisation |
| `predicted_levels` | Predicted levels |
| `level_holdout` | Whole-level hold-out |
| `none` | Full published level set, no hold-out, not a validation number |

A result file carrying a metric without this field is refused rather than read
with an assumed value. There is no default, because the three procedures measure
different things and the difference between them is exactly what a default would
hide.

Each value carries the parameters that make it reproducible: which lines or
levels were withheld and by what rule they were chosen, the identity of the
predicted level set where one was used, and the seed if the withheld set was
drawn. The run is a run like any other, so it also carries everything in the
provenance list of `docs/decisions/repeatable-runs.md`, and the withheld set is
part of the input identity rather than a note beside it.

## Tuning and testing

Every weight, threshold, prior and default in these records is a thing somebody
can adjust until a spectrum looks good. `docs/decisions/selection-rules.md`
already says that changing a weight is a tuning action governed by this file, so
this is where the rule is.

Tuning means any change to any value in any parameter table of any record here,
and any change to a rule that decides which candidates are generated. It does not
mean fixing a defect, and the difference is that a defect is a disagreement
between the code and a record while tuning is a change to the record.

Two tiers.

The available set may be looked at while tuning, run any number of times, and
argued with. Numbers from it are reported and are not evidence that the method
generalises.

The reserved set may not be run at all until the numbers it produces are the
numbers being reported. Each reserved spectrum is evaluated once per released
engine version, and every evaluation is recorded whether or not its result was
published. That last clause is the one that matters: an unrecorded evaluation is
how a reserved spectrum quietly becomes a tuning spectrum, and the count of
evaluations is more informative about the honesty of a figure than the figure is.

A spectrum never moves from reserved to available. It may move the other way and
there is no reason to.

What makes this checkable is the history rather than a check. The commit that
places a spectrum in the register precedes the commit that reports a number from
it:

    git log --oneline -- docs/validation-protocol.md

Nothing in this repository refuses a result that quotes a spectrum added to the
register afterwards. That gap is real, it is the same gap
`docs/validation-metrics.md` records for the disagreement arguments, and stating
it here is what this file does about it today.

## The register

Both lists are empty. No spectrum has been chosen for either tier, and this file
is where they are written down when they are.

Available for tuning:

    (none)

Reserved:

    (none)

Issue #46 chooses the first solved spectrum and argues the choice, including what
was rejected. Issue #48 names the second before its numbers are seen, together
with what is expected to be hard about it and what would count as success, and
its own declared scope includes this file, so it is edited here rather than
recorded somewhere parallel.

An empty register is the correct state today and it is not a placeholder for a
list somebody forgot. It is also a hard bound on what this file can be used for:
no validation number can be produced under this protocol until at least one
spectrum is in it, and the first number produced from a spectrum in the available
tier is not evidence that the method generalises whatever it says.

## The commands, which do not exist

Issue #45 asks for the command that runs each procedure. Nothing here implements
one, so what follows is a requirement rather than a description, in the same
shape `docs/decisions/input-contract.md` uses for its validator.

Each procedure is one invocation of the same binary, differing only in its
inputs and in the hold-out parameters, and never in the objective, the priors or
the code path that scores a candidate. A procedure implemented as a separate mode
of the engine would be measuring a different engine from the one an operator
runs, which is the shape of failure this protocol exists to catch in the data and
would be embarrassing to reintroduce in the harness. The hold-out is applied to
the inputs, before the run, and the run does not know which procedure it is part
of.

The three are slow. They belong to the integration harness of issue #7 rather
than to the default gate, and the gate says so on every run rather than being
silent about a set it did not cover.

## What the fixture test owes

Issue #45 asks for a test that runs the whole protocol on a small fixture whose
truth is known by construction. Known by construction means generated rather than
published: a level set drawn to a chosen density with chosen parities and J
values, a line list computed from it, and a recorded truth that no fit produced,
so that the circle this file is about does not exist in the fixture at all.
`docs/decisions/layout.md` puts that generator on the generic side, in
`assoc-synth`, and the spectroscopic dressing of it in the spectroscopy crate
that instantiates it.

The test asserts, for each of the three procedures, that the protocol runs end to
end, that every reported number carries its `holdout_procedure`, and that the
whole-level hold-out produces the none-of-these hypothesis for features whose
level was removed. It does not assert a recovery figure, because a figure
asserted against a generated fixture is a property of the generator.

The near-miss worth spending the effort on is the one the trap is about: a
fixture where the level energies were fitted to the same lines being assigned,
run through the line hold-out procedure with the re-optimisation step skipped, so
that the recovery figure comes out high and the test has to notice that the
levels the run scored against were not the re-optimised ones. That is the mistake
somebody will actually make, because skipping the expensive step leaves every
number looking better.
