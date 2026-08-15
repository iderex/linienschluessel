# How intensities are used

Decision record for issue #15. Intensity is what separates this board from a
difference-matching script, and it is the information most easily misused,
because the numbers a source publishes are not on any scale that survives
leaving that source.

Nothing in this repository refuses a violation of this record today. Issue #33
is where the intensity term is implemented and where the invariance property
below becomes a test.

## What the source actually says

The NIST Atomic Spectra Database line help page, retrieved 2026-08-07 from
https://physics.nist.gov/PhysRefData/ASD/Html/lineshelp.html, states that
relative intensities are source dependent and typically useful only as
guidelines for low density sources, that in most cases the values represent
blackening of photographic emulsions used to record an observed spectrum, that
there is no common scale for relative intensities, and that the relative
intensities have meaning only within a given spectrum, that is, within the
spectrum of a given element in a given stage of ionization.

That is the whole reason for what follows. An objective comparing a recorded
intensity against a transition probability in absolute terms is comparing two
numbers that were never on one scale, and it will produce a plausible wrong
answer and not an error.

## What enters the objective, and in what form

Recorded intensity enters in exactly one form: as an ordering constraint.

Each spectrum carries a latent log photon flux for every feature in it. The
recorded intensities do not supply the values of those fluxes. They supply only
the order: if feature a is recorded brighter than feature b in the same
spectrum, then the latent flux of a is at least that of b. The magnitudes come
from the model side, out of the transition rates and the level populations the
candidate hypotheses imply, fitted subject to that order.

So the recorded numbers are used for what they are, a ranking within one
spectrum, and for nothing else. Features whose recorded intensities are equal
carry no constraint between them. Features in different spectra carry no
constraint between them at all, because the sources never claimed one.

## Why not the rough spacing as well

Issue #15 recommends an unknown monotone transform per source, fitted as a
nuisance, so that the model uses the ordering and the rough spacing of
intensities without asserting the scale. This record takes the ordering and
declines the spacing, and the reason is a property rather than a preference.

Issue #33 requires that replacing every intensity in a spectrum by any strictly
increasing function of itself leaves the assignment unchanged. That property is
machine-decidable, which is worth more than any amount of prose saying
intensities are relative. But spacing is not invariant under an arbitrary
strictly increasing function: a transform mapping every intensity to its own
rank preserves the order and destroys the spacing entirely. Any objective that
reads the spacing therefore changes its answer under such a transform and cannot
pass the test in #33.

The order is the largest functional of the recorded intensities that survives
every strictly increasing transform, so it is what the record uses. The cost is
real. Information that a line is ten times brighter rather than slightly
brighter is discarded, and where a source did publish a calibrated scale this
board will use less of it than it could. The condition that would reopen this is
an input contract that carries a scale label meaning calibrated radiance for a
whole spectrum, at which point that spectrum could take a second term stated
separately rather than by weakening the invariance for everyone.

Because the fit depends on the recorded intensities only through their order,
the invariance is exact rather than approximate, and #33's property test needs
no tolerance. A parametric monotone family with a fixed number of knots would
not give that, since composing it with an arbitrary monotone function leaves the
family, so the fit is a nonparametric isotonic one over the ranks.

## The branching constraint

Two lines from the same upper level share a population, and the population
cancels in their ratio. Their intensity ratio is then a branching ratio, fixed
by the transition rates alone, with no calibration anywhere in the argument.
This is the strongest evidence intensity can give on this board, for the fewest
assumptions.

Written against the latent fluxes above, for two lines i and j from a shared
upper level u observed in the same spectrum, the constraint is that the
difference of latent log fluxes equals log(A_i / A_j) for a photon-counting
scale, or log(A_i * nu_i / (A_j * nu_j)) for an energy scale, with nu the
transition frequency. Which of the two applies is read from the scale label the
line list carries, and a feature whose scale label is arbitrary or photographic
takes the photon-counting form with the response caveat below. The scale label
is required rather than guessed, which is issue #21.

The constraint applies only when all of the following hold, and the run records
which of them failed for every upper level it did not apply it to.

- Both lines are assigned to the same upper level under the hypothesis being
  scored.
- Both lines come from the same spectrum, identified by the spectrum identity
  the line list carries.
- Transition rates are available for both. Where they are not, the constraint is
  not applied and no substitute is invented; what a run does with a shared upper
  level whose rates are missing is issue #34.
- Neither feature is flagged saturated, and neither is a blend, unless the blend
  is decomposed as described below.

## The instrument response, and where it is not modelled

The latent flux above is flux at the detector, not flux emitted. Between the two
sits an instrument response that varies with wavelength, and a per-spectrum
monotone fit cannot represent it, because the response is a function of
wavelength and the fit is a function of recorded intensity. Two branches from
one upper level separated by a wide interval are therefore compared through an
unknown factor even inside one spectrum.

The first release does not fit a response curve. It inflates the variance of the
branching constraint with the wavelength separation of the two lines, by a
standard deviation of `weights.intensity.response_drift` multiplied by the
absolute value of log(lambda_i / lambda_j), with a default of 0.35 in natural
log units. That default is an explicit prior. It is not a measurement, it was
not fitted to any spectrum, and no measurement of a real response curve has been
made here. The measurement that would replace it is a calibration lamp spectrum
from the same instrument, which the input contract does not currently carry.

The residual risk this leaves is stated rather than closed: two branches far
apart in wavelength can disagree by an amount the model attributes to response
drift when the real cause is a wrong upper level. The run reports the branching
residual for every shared upper level it evaluated, so a reader can see the
disagreement instead of being handed the conclusion.

## The saturated end

Saturation and self-absorption break monotonicity at the bright end: a stronger
line records as no brighter, or as fainter, than a weaker one. Because the fit
is isotonic and non-decreasing rather than strictly increasing, it can flatten
there without failing, and a flat run in the fit is exactly the signature.

What the model does. A feature the source itself flags as saturated or
self-absorbed is excluded from the branching constraint and keeps only its order
constraint. A feature that is not flagged but falls inside a flat run of the
fitted isotonic curve whose recorded-intensity width exceeds
`weights.intensity.flat_run_report`, default 0.15 of the spectrum's recorded
intensity range, is reported as suspected saturation, with the range of the flat
run. Suspicion changes the report and does not change the score, because the
flat run is already the model declining to read anything into that region.

## The blended end

Intensity belongs to the feature, not to the transition. A blend is one recorded
feature with more than one owner, so the model predicts the feature's flux as
the sum of the fluxes of its component transitions, and the order constraint
applies to that sum. A blend therefore never has its recorded intensity handed
to any single component.

The branching constraint is applied to a blended feature only through the sum,
which usually means it is not applied at all, since a sum of branches from
different upper levels constrains none of them individually. A feature the
source flags as blended, and a feature the run assigns a blend hypothesis to,
are both treated this way; the two cases are recorded separately in the output
so a reader can tell a blend the source knew about from one the run proposed.

Non-detection is not used. A transition that should have produced a line and
did not is real evidence, and reading it needs a detection threshold per
spectrum that the input contract does not carry. The first release does not
use it, and saying so here is the disclosure, and it is not a plan.

## What appears in the output

The fitted isotonic curve for each spectrum, as the recorded intensity and the
fitted latent log flux for every feature, so the fit can be plotted and argued
with. Every flat run in it with its width. Every branching residual the run
evaluated, with the upper level, the lines and the amount. Every upper level
where the constraint was not applied, with which of the conditions above failed.
And the flag on every feature reported as suspected saturation.

A transform that has to work very hard, or a run of flatness covering half a
spectrum, is the model telling the operator something about their data. That is
diagnostic and it belongs in the answer file rather than in a log nobody keeps.
