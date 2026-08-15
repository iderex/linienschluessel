# The uncertainty model, and what is correlated with what

Decision record for issue #14. Two uncertainties meet in every candidate, the
uncertainty of the observed position and the uncertainty of the predicted
position built out of two level energies. This record fixes the distribution of
each and, which matters more, what is correlated with what.

Everything below is written on the internal representation of
`docs/decisions/line-position.md`, so every sigma in this file is a vacuum
wavenumber in cm-1 unless it is explicitly relative.

Nothing in this repository refuses a violation of this record today. Issue #32
is where the position term is implemented against these distributions, issue #26
is where the tolerance derived from them decides which candidates exist at all,
and issue #24 is where the counts this record asks a run to report become part
of the output.

## The three inputs

### Observed line positions

Gaussian on the wavenumber, with the standard deviation the line list declared,
plus the shared per-source terms of the next section.

Gaussian and not something heavier tailed, deliberately. The tail cases here are
a blended feature, a misidentified feature and a line from another species, and
none of them is a wide draw from the same distribution: each is a different
hypothesis, and the model of `docs/decisions/competitors.md` already carries
them as hypotheses. Widening the position distribution to absorb them would let
one term stand in for three explanations and would make the engine quieter about
exactly the cases it exists to surface. Where a heavier tail is nonetheless the
right repair, it is a change to the position term in issue #32 and it is made
there with its reason, not smuggled in as a robustness default here.

Line positions within one source are not independent, which is the whole of the
next section.

### Level energies from a least-squares optimisation

Gaussian, and correlated, and the correlation is not a subtlety anybody has to
be persuaded of: it is why the covariance matrix of the level fit is what
produces Ritz wavenumber uncertainties in the first place, as in the program
LOPT described in Comput. Phys. Commun. 182, 419 (2011).

A difference of two levels from one fit has variance

    var(E_u - E_l) = var(E_u) + var(E_l) - 2 * rho * sd(E_u) * sd(E_l)

and rho is positive for a pair the fit connected, because both were pulled by
the same observed lines and both are referred to the same ground state. Setting
rho to zero therefore does not fail safe. It inflates the predicted position's
uncertainty, a residual divided by an inflated sigma looks smaller than it is,
and every candidate scores better than it should. That is the flattering
direction, and it is the direction an engine drifts into by default.

So the model does not set rho to zero and does not pretend to a covariance it
was not given. The input contract of issue #18 carries an optional covariance
for the level set. Where it is supplied, it is used, and the formula above is
evaluated with the real off-diagonal term. Where it is not, and no public
compilation this board has looked at ships one, the run uses a single declared
correlation `uncertainty.level_pair_correlation`, default 0.5.

That default is an explicit prior. It is not a measurement, it was not fitted to
any level system, and no covariance matrix has been read here. What it asserts
is a direction and a rough size: that levels from one optimisation are
substantially rather than negligibly correlated. The measurement that would
replace it is the covariance matrix of the fit that produced the level set, or a
re-run of that optimisation from the line list it consumed.

Because a prior is doing work, the run reports how much work. Every run states
the share of its accepted assignments whose acceptance changes when rho is set
to zero, at the run's own thresholds, as one number with its numerator and
denominator. A run in which that share is large is a run whose answers rest on
this default, and the reader is told so and does not have to discover it.

### Level energies from a structure calculation

Not Gaussian, correlated within a configuration, and with a size that is a
property of the method rather than of the level.

The distribution is a Student t with `uncertainty.predicted.dof` degrees of
freedom, default 4, scaled so that its standard deviation is the declared
spread. The heavy tail is right here for the reason it was wrong for the
observed positions: a structure calculation does not miss by a wide draw from
a narrow distribution, it misses because a configuration interaction was
truncated or a term was placed in the wrong order, and those are common, not
extreme.

Within one configuration the errors move together, so the model carries one
shared offset per configuration, with a prior width of
`uncertainty.predicted.configuration_offset`. Both numbers above are explicit
priors, neither is a measurement, and neither was fitted to any calculation. The
measurement that would replace them is the distribution of differences between
predicted and later-measured levels for a comparable species, which is a study
this board has not run.

A prediction that arrives with a spread rather than with a standard deviation is
read as a spread and recorded as one. The spread is taken as the scale of the
distribution above, and the level carries a flag saying its uncertainty came
from a spread. That flag travels into every candidate the level appears in and
into the answer, so a number derived from a spread is never printed as though it
came from a standard deviation.

A level set that mixes measured and predicted levels without saying which is
which cannot be used, and issue #20 carries that refusal.

## The per-source terms

A wavelength scale is calibrated, and a calibration error moves a whole spectrum
or a whole grating order together. If every line is treated as independently
scattered, a systematic shift shows up as many mutually reinforcing coincidences
and the engine becomes most confident exactly where the data are worst.

A shared term is not well defined until the scale it is additive on is named,
and the two natural errors are additive on different scales. A dispersion scale
error is a constant fractional error, so it is a constant in delta sigma over
sigma. A zero point error is a constant shift in wavelength, which in wavenumber
grows as the square of the wavenumber. The model therefore carries both, per
source, and does not choose between them:

- a relative term, constant in delta sigma over sigma, with prior width
  `uncertainty.source.relative`
- an absolute term, constant in delta sigma, with prior width
  `uncertainty.source.absolute`

A source here is the identity the line list declares for a spectrum, and where
the line list declares grating orders or wavelength segments, the terms are
carried per segment rather than per file, because that is the unit a calibration
was actually done on.

Both prior widths are explicit priors and neither is a measurement. Both fitted
values appear in the answer file, per source and per segment, with the prior
width beside each so a reader can see how far the fit moved from it. A fitted
offset larger than the calibration of that spectrum can explain is a finding
about the spectrum, not a correction to be applied silently, and the run says so
by reporting the value rather than only its effect.

The first release fits no higher term than these two. A curvature in the
dispersion residual is real and this model cannot represent it, so a source
with one will show it as an inflated scatter that the position term reads as
noise. That residual risk is stated and not closed, and the measurement that
would justify a third term is the pattern of position residuals against
wavenumber within one source, which a run reports and nobody has yet looked at
here.

## What is independent of what

Observed positions and level energies are independent, except when they are not,
and the exception is the one that matters.

If the lines being assigned are among the lines that were fed to the
optimisation that produced the level set, then the level energies are functions
of those observations, and the residual between an observed position and its own
Ritz prediction is not a test of anything. It is small by construction, and the
smaller it is the more confident this engine becomes. The model does not attempt
to unwind that correlation, because unwinding it needs the fit's design matrix
and nothing here will have it.

What it does instead is refuse to be quiet about it. The input contract carries,
per level set, whether the line list being assigned was among its inputs, and a
run whose two inputs stand in that relation says so in its answer file in every
place a probability appears. Breaking the circularity is a protocol rather than
a distribution, and that protocol is issue #45.

Between two sources, no term is shared. Two spectra were calibrated separately
and a shared term between them would be an assertion nobody made.

## An accuracy class is not an uncertainty

Some line lists give an accuracy class rather than a number. Turning a class
into a number is an assumption, and the core refuses to make it.

No mapping from a class to a number exists anywhere inside the engine. A class
is one upstream's published vocabulary with one upstream's published meaning, so
if it is to become a number that happens in that upstream's adapter, which is
issue #23, under three obligations: the mapping is the one that upstream
publishes, it is cited with the page and the date it was read from, and every
value it produces is flagged as derived from a class.

The flag travels with the number into the candidate, into the score and into the
answer file, and every run reports how many of its positions carried it. A
number derived from a guessed uncertainty is then visibly a different kind of
number from a measured one, everywhere it is used, which is the whole point of
refusing the mapping in the core.

An adapter that cannot map a class truthfully reports the uncertainty as
unavailable, and the case below applies.

## When there is no uncertainty at all

No default is substituted. That is the rule, and the rest of this section is
what happens instead.

A feature whose position carries no declared uncertainty is read, kept and
reported. It cannot enter the position term, because there is no scale to
measure a residual against, and it cannot generate candidates, because the
tolerance in issue #26 is derived from declared uncertainties and there is
nothing to derive one from.

The operator may declare a fallback for a source, `uncertainty.fallback`, which
has no default value and is unset unless somebody sets it. Where it is set, the
features of that source use it, every position so treated is flagged, the flag
travels into the answer as the class flag above does, and the run reports the
count. Where it is not set, those features are reported unassigned with the
reason `no_declared_uncertainty`, alongside the other unassigned reasons of
issue #30.

The same applies to a level with no declared uncertainty, and there the
consequence is larger, because one such level removes every candidate it would
have appeared in. The run reports that count too, as part of the completeness
report of issue #24, so a level set that is unusable for this reason says so in
the answer rather than producing a short one.
