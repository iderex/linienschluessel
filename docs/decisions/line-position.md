# One internal representation for a line position

Decision record for issue #10. Every position inside this engine is a vacuum
wavenumber in cm-1. Every conversion into that representation happens in a
reader, once, and no conversion happens anywhere else.

Nothing in this repository refuses a violation of this record today. Issue #20
is where the level reader refuses an energy with no unit, issue #21 is where the
line reader refuses a position with no declared medium, and issue #23 is where
observed and Ritz positions are kept apart in the line adapter.

## The representation, and why the operations pick it

The Ritz combination principle is additive in wavenumber. The wavenumber of a
transition is the difference of two level energies expressed in the same unit,
and it is not the difference of two wavelengths. Every core operation on this
board is a difference, a residual, or a sum of residuals, so the representation
has to be the one those operations are linear in.

Working internally in wavelength costs three things at once. Each comparison
becomes a reciprocal. A symmetric uncertainty on a level becomes an asymmetric
one on the comparison. And a single tolerance figure means a different physical
thing at each end of the range, so a window that is right at 200 nm is wrong at
2000 nm by the square of the ratio.

The unit is cm-1 rather than any other wavenumber unit for one reason that is
not aesthetic: it is the unit the level compilations this board consumes publish
in, so the common case converts by doing nothing, and a conversion that does
nothing cannot round.

Vacuum rather than air, because vacuum wavenumber is the only one of the four
common spellings of a line position that does not depend on a medium. That is
what makes the boundary rule in the next section well posed, and it is what lets
two files from two sources be compared at all.

## Air, and the boundary that is stated on the wavenumber

Air is not an optional complication. The NIST Atomic Spectra Database lines help
page, retrieved 2026-08-07 from
https://physics.nist.gov/PhysRefData/ASD/Html/lineshelp.html, states the medium
of its wavelength output as a rule on the wavenumber: for sigma at or above
50,000 cm-1 vacuum wavelengths, for sigma between 5000 and 50,000 cm-1 air
wavelengths, and for sigma at or below 5000 cm-1 vacuum wavelengths again.

Two things follow, and the second is the one that gets missed.

The medium of a number depends on where in the file that number sits, so a
reader that applies one rule to a whole file corrupts one end of it. Splitting
at the wrong place is worse than not splitting, because it looks like a fix.

And the rule is stated on the wavenumber rather than on the wavelength, which is
the only way it could be stated without circularity. A wavelength near a
boundary does not by itself say which side it is on, since which side it is on
is what decides the conversion that would tell you. Applied to sigma the
question does not arise, because sigma is the same number in either medium. So a
reader that meets a wavelength column converts under the medium the boundary
rule assigns to the wavenumber that wavelength implies, and near a boundary it
carries out that test on the vacuum wavenumber rather than on the wavelength.

None of this is a property of physics. It is a property of one source's output
convention, so it lives in that source's adapter, which is issue #23, and never
in the core.

## The index of refraction

The conversion between vacuum and standard air uses the index of refraction of
air from the five-parameter formula of E. R. Peck and K. Reeder, J. Opt. Soc.
Am. 62, 958 (1972), which is the formula the source above states it derives its
own index from. The coefficients are taken from that paper at the point the
function is written rather than transcribed into this record, because a constant
copied into a document is a constant that drifts against the code and cannot be
checked against either.

Standard air is what the formula defines it to be, dry air at 15 degrees Celsius
and 101325 Pa with the carbon dioxide content the formula assumes. It is not the
air in anybody's laboratory. A published air wavelength is a number on that
convention, so the board converts on that convention and does not attempt to
correct for the conditions of a measurement. The input contract carries no place
to declare a temperature or a pressure, which is issue #18, and inventing one
would be a correction applied to numbers that were never on the corrected scale.

## Whether the formula's uncertainty is propagated

It is propagated, and it is not neglected.

The same page states the relative uncertainty of the Peck and Reeder formula as
5e-9 for wavelengths above 400 nm, rising for shorter wavelengths according to
the approximate formula

    delta_lambda / lambda = (0.35734 + 38.24 / (lambda - 180.29)
                             + 0.000023 * lambda) * 1e-8

with lambda in nm, and gives the maximum as about 9e-8 at 185 nm.

At 5e-9 relative this is far inside any line uncertainty this board will meet,
and at 9e-8 near 185 nm it is still small against most of them. It is
propagated anyway, for two reasons. It costs one term in a variance that is
being computed regardless. And it is the term whose absence would be invisible:
a position converted from air carries an uncertainty contribution a position
read in vacuum does not, and if that contribution is dropped the two are
compared as though they had the same provenance. The uncertainty model in issue
#14 is where the term joins the others, and it enters as an independent relative
term on the converted position, since it is a property of the formula rather
than of the spectrum.

The term is recorded per position, so a run can report how many of its positions
carry a conversion uncertainty at all. A run over a level set and a line list
that were both already in vacuum wavenumbers reports zero, which is the honest
answer and not a missing field.

## The reverse conversion, and when it stops

One direction is arithmetic and the other is a fixed point.

From a vacuum wavenumber to an air wavelength is direct. The vacuum wavelength
is the reciprocal of the wavenumber, the index is evaluated at that wavenumber,
and the air wavelength is the vacuum wavelength divided by the index. Nothing
iterates.

From an air wavelength to a vacuum wavenumber is not, because the index has to
be evaluated at the vacuum wavelength, which is the thing being computed. The
iteration starts from the air wavelength treated as though it were the vacuum
one, evaluates the index there, divides to get a better vacuum wavelength, and
repeats.

The criterion is written here rather than left to whoever writes the function.
The iteration stops when the relative change in the vacuum wavenumber between
two successive steps falls below 1e-10. That is a factor of fifty inside the
5e-9 relative uncertainty the formula itself carries at its best, so the
iteration is stopped well below the accuracy of the thing being iterated and
well above the point where it would be chasing floating point noise.

It stops in one other way. After eight iterations without meeting the criterion
the conversion refuses the input and names the value it was given. It does not
return the last iterate. A fixed point that has not converged in eight steps on
a function this smooth is not a slow conversion, it is a number outside the
range the formula covers or a column that is not a wavelength, and returning the
last iterate would turn that into a plausible answer. No measurement of the
iteration count on real data has been made here, because there is no reader yet;
the cap is set from the shape of the map rather than from a count, and issue #21
is where the actual distribution of iteration counts can first be reported.

## A file that does not say

A position with no declared medium, or with no declared unit, is refused. The
reader names the field and the line and stops. It does not guess.

Guessing from the range is the specific thing this record forbids, because it is
the convention that caused the problem in the first place: a number between 200
and 2000 is an air wavelength in nm on one source's convention, a vacuum
wavelength on another's, and a wavenumber on a third's, and the ranges overlap.
A reader that picks by magnitude is right most of the time, which is what makes
it dangerous, and the cases it is wrong in are the unusual spectra this board
exists for.

The same refusal covers a level set that does not say what its energies are
measured from. Level energies enter as differences, and a difference is
independent of the reference point only when both levels share one, so a file
mixing two reference points is unusable and a file declaring none cannot be
checked. Issue #20 carries the refusal.

Where an energy arrives in a unit other than cm-1, the conversion factor is
taken from a named CODATA release, and the release is recorded in the run record
that issue #19 defines rather than assumed. Two runs that used different
releases of the constants are two runs that can disagree in the last digits for
a reason nobody would otherwise look for.

## Observed and Ritz are not interchangeable inputs

The same page states that Ritz wavelengths are derived from the lower and upper
levels of the transition and are usually more accurate than the observed
wavelengths, especially in the vacuum ultraviolet.

More accurate, and unusable as an observation on this board. A Ritz position was
computed from the levels, so scoring it against a difference of those same
levels is scoring the level set against itself, and the residual it produces is
a statement about arithmetic rather than about a spectrum. Every position this
engine treats as an observation is an observed one. A line list that carries
both keeps them in separate fields with the distinction preserved, which is
issue #23, and a line list that carries only Ritz positions is a line list this
board cannot validate against and says so rather than running.
