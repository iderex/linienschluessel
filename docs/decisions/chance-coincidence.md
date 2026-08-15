# The null model for accidental matches

Decision record for issue #16. A few hundred levels produce tens of thousands of
level pairs, their differences cover the range densely, and some observed
feature will fall inside tolerance of some pair for no reason at all. Without a
number for that rate a match is not evidence. This record fixes the number, how
it is produced, and where it appears.

The rate is measured in every run and per region of the spectrum. It is not
quoted from a study somebody did on another element, because it depends on the
local density of levels, on the tolerance the declared uncertainties imply, on
the region, and on how complete the level set is, and all four move between
runs.

Nothing in this repository refuses a violation of this record today. Issue #29
is where the measurement is implemented and becomes a test, issue #12 is where
the rate enters the probability that the truth is absent from the candidate
list, and issue #43 is where the fields below become part of the answer schema.

## The null used in every run: a common displacement of the observed features

The reported null shifts every observed position by one common offset in
wavenumber and re-runs candidate enumeration against the unchanged level set.

The procedure, exactly enough to reimplement.

Take the observed feature list as the run received it, after the readers and
after the conversions of `docs/decisions/line-position.md`, so every position is
a vacuum wavenumber in cm-1 and every declared uncertainty travels with its
feature. Take the level set as the run received it. Take the tolerance rule the
run itself uses, which is issue #26's and is derived from the declared
uncertainties rather than from a constant.

For each offset d in the offset set below, form the displaced feature list by
adding d to every position and to nothing else. Uncertainties, intensities,
flags and source identities are unchanged, because the null is a statement about
positions and not about the rest of the record. Enumerate candidates for the
displaced list against the level set under the same tolerance rule and the same
hard rules of `docs/decisions/selection-rules.md`. Count, per region, the
features that acquired at least one candidate, and separately the total number
of candidates.

The offset set is `null.offsets`, by default twenty offsets, ten positive and
ten negative, spaced evenly over the interval from `null.offset_min` to
`null.offset_max`, with defaults of fifty times the median tolerance and one
part in two hundred of the spectral range covered. The lower end keeps every
true assignment destroyed. The upper end keeps the displaced list inside the
region the level set can reach, so the null measures accidental matching rather
than the edge of the covered range. Features whose displaced position leaves the
covered range are excluded from both numerator and denominator of that offset,
and the count excluded is reported.

The reported rate for a region is the mean over the offsets, and the spread over
the offsets is reported beside it. The spread is not decoration. A null rate
that is stable across twenty offsets is a rate; one that swings by a factor of
two between offsets is telling the reader that the level-difference set has
structure at the scale of the offsets, and that the mean alone would be
misleading.

## Why the null displaces the features and not the levels

Issue #16 offers, as the cheap option, displacing every level by a common
offset larger than the tolerance. That does not work, and the reason is
arithmetic and not a matter of degree.

Every quantity this engine compares against an observed position is a difference
of two level energies. Adding a common offset d to every level leaves every such
difference exactly unchanged, so the candidate set after the displacement is the
candidate set before it, and the measured null rate would be the real match rate
wearing a different name. Nothing about the size of d repairs that.

The nearest thing that does work is displacing one parity class, which does
shift every parity-changing difference by d. It is not the null used here for
two reasons. It shifts the electric dipole differences and leaves the magnetic
dipole and electric quadrupole differences alone, so it measures a different
null for each multipole in one pass and cannot be reported as one number. And a
level set whose two parity classes have different densities is displaced
asymmetrically by it, which is a bias that depends on the level set rather than
on the tolerance.

Displacing the features has neither problem. Line density, level density and the
whole pair structure of the level set survive exactly, the tolerance travels
with each feature, and the same shift applies to every multipole at once.

It has one property worth naming and not hiding. Real observed features
cluster where levels are dense, and a shifted feature list still lands
preferentially in dense regions. So this null is not a uniform sprinkle over
the range; it keeps the joint structure of the two inputs and asks what the
match rate would be if the correspondence between them were destroyed. That is
the question the board actually needs answered, and it is a different and
harder question than what a uniform random line list would give.

## Regions

A single number over a whole spectrum hides the thing the number is for, because
the level density and the tolerance both vary by more than an order of magnitude
across a range this board covers.

Regions are contiguous intervals in wavenumber, chosen so that each holds at
least `null.region_min_features` observed features, default 50, with the last
interval merged into its neighbour and never left short. Equal counts, not
equal widths, because a region holding three features gives a rate with a
denominator of three and reads as a measurement.

Each region reports its bounds, the number of observed features in it, the
number of level pairs whose difference falls in it, the median tolerance in it,
the null rate as the mean over offsets with its spread, and the number of
assignments the real run made in it. Those are the numbers a reader needs to see
side by side, and a region is the unit they are meaningful in.

No such measurement exists yet. There is no enumerator here, so every number in
this section is a field name and not a value, and issue #29 is where the first
of them acquires one.

## The alternatives, kept as diagnostics

Each answers a different question and none replaces the null above.

A synthetic level set, drawn to match the measured density of the real one in
each region together with its parity and J distributions, with the pair
structure not preserved. It answers how much of the accidental rate comes from
the structure of this level set rather than from its density alone. It needs a
model of that structure, and the model is an assumption the reported null does
not make, which is why this is a diagnostic.

A permutation of the observed positions within a window, drawn so that each
feature is reassigned a position from its own neighbourhood. It preserves the
line density and destroys the level correspondence differently from a common
shift. It answers whether the reported null depends on the shape of the shift,
and a large disagreement between the two is a signal that the offset range was
badly chosen.

A displacement of one parity class, as above. It answers the multipole-resolved
question the single reported null cannot: whether the accidental rate for the
forbidden multipoles differs from the rate for electric dipole, which matters
because those candidates are rarer and carry a heavier prior.

All three are computed only when asked for, and a run that did not compute
them says so and does not leave the fields absent. A diagnostic that was not
run and a diagnostic that returned nothing are different statements.

## When the accidental rate is the same size as the answer

This is the case the record exists for, and the board neither suppresses the
answer nor quietly deflates it.

Every region reports `null_ratio`, the null rate in that region divided by the
number of assignments the real run made there, with both counts printed. A
region where that ratio approaches one is a region where the engine found about
as many matches as chance produces, and no probability computed inside that
region means what its digits suggest.

Three things follow, and all three are reporting rather than filtering.

The region is marked `chance_dominated` when `null_ratio` exceeds
`null.dominated_ratio`, default 0.5. The mark is on the region and on every
assignment inside it, so it cannot be lost by a reader who looks at one row.

The summary of the answer file leads with the marked regions rather than
appending them. A run that produced two hundred assignments of which the
accidental rate can account for eighty is a run whose headline is that fact.

The null rate is an input to the probability model rather than a footnote to it.
The mass that `docs/decisions/competitors.md` puts on the none-of-these
hypothesis has to be informed by how easily a candidate arises by accident in
that region, and issue #12 is where that link is defined. A null measured and
then not used would be a number printed beside a probability it did not affect.

What the board does not do is drop the assignments or rescale their
probabilities by the ratio. Dropping them removes the evidence a reader would
use to judge the run. Rescaling by a ratio computed at the region level would
apply one correction to every feature in the region regardless of how well any
of them actually matched, which is a worse model wearing the appearance of
caution, and it would make the reported number no longer the number the
calibration work in issue #42 measured.
