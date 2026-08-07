# How selection rules enter

Decision record for issue #13. Selection rules enter this engine as two
different kinds of thing, and which kind a rule is decides whether it can
delete a candidate or only move its score.

A hard rule removes a candidate. A weighted rule changes a candidate's score
and can never remove it. Getting that backwards is how an assignment tool
becomes silently unable to see the lines it exists to find.

Nothing in this repository refuses a violation of this record today, because
there is no engine here yet. Issue #27 is where these rules are implemented and
where the hard and weighted markings become a table in the code rather than a
repetition at each site.

## Which multipoles the first release considers

Electric dipole (E1), magnetic dipole (M1) and electric quadrupole (E2).

E1 is the bulk of a laboratory emission spectrum. M1 and E2 are in because
forbidden lines are a large part of why lanthanide and transition-metal spectra
are interesting in astrophysical and fusion sources at all, and a tool that
generates only E1 candidates will answer a question nobody asked in exactly the
spectra this board is aimed at.

Everything above those three, magnetic quadrupole and the higher electric
multipoles, is out of the first release. This is a real limitation and not a
statement that such lines do not occur. A feature whose only explanation is a
multipole the release does not generate has no candidate offered for it, so the
none-of-these hypothesis that issue #11 defines is what wins, and the run
reports the feature as unassigned rather than
forcing it onto an E1 pair. The condition that would bring a further multipole
in is a spectrum in the validation set where declining costs more than the extra
candidate volume, measured rather than argued.

## The hard rules

These follow from the multipole operator and hold whatever the coupling scheme.
Each one removes a candidate for that multipole outright.

| Multipole | Rule | Kind |
| --- | --- | --- |
| E1 | Parity changes between the two levels | hard |
| E1 | Delta J is 0, +1 or -1 | hard |
| E1 | J = 0 to J = 0 is forbidden | hard |
| M1 | Parity does not change | hard |
| M1 | Delta J is 0, +1 or -1 | hard |
| M1 | J = 0 to J = 0 is forbidden | hard |
| E2 | Parity does not change | hard |
| E2 | Delta J is 0, +1, +2, -1 or -2 | hard |
| E2 | J = 0 to J = 0, J = 0 to J = 1, and J = 1/2 to J = 1/2 are forbidden | hard |

A hard rule needs the parity and the J of both levels. Where the level set does
not give one of them, the rule is not applied and cannot remove the candidate.
The candidate is marked as having an unevaluated hard rule, and the run reports
how many candidates carry that mark, because a hard rule that quietly did not
run is otherwise indistinguishable from one that passed. Never guess a parity
or a J in order to have something to test.

## The weighted rules

These come with LS coupling. They are approximations, and they weaken exactly
where the spin-orbit interaction is large, which is the heavy and open-shell
end this board is pointed at. Intercombination lines violating the spin rule are
observed and are sometimes strong, so none of these may ever remove a candidate.

| Multipole | Rule violated | Switch | Default weight |
| --- | --- | --- | --- |
| E1 | Delta S is not 0 (intercombination) | `weights.e1.spin_change` | -3.0 |
| E1 | Delta L is outside 0, +1, -1 | `weights.e1.delta_l` | -2.0 |
| E1 | L = 0 to L = 0 | `weights.e1.l_zero_to_zero` | -2.0 |
| E1 | Not a single-electron jump | `weights.e1.multi_electron_jump` | -2.5 |
| M1 | Delta S is not 0 | `weights.m1.spin_change` | -3.0 |
| M1 | The two levels are not in the same configuration | `weights.m1.configuration_change` | -3.5 |
| E2 | Delta S is not 0 | `weights.e2.spin_change` | -3.0 |
| E2 | Delta L is outside 0, +1, +2, -1, -2 | `weights.e2.delta_l` | -2.0 |

A candidate also carries a prior for the multipole itself, because a forbidden
line is rarer than an allowed one in the same spectrum without being absent from
it.

| Rule | Switch | Default weight |
| --- | --- | --- |
| Candidate is M1 rather than E1 | `weights.multipole.m1` | -6.0 |
| Candidate is E2 rather than E1 | `weights.multipole.e2` | -6.0 |

Weights are additive contributions in natural log-odds, so -3.0 is a factor of
about twenty against and -6.0 a factor of about four hundred.

## Where the weights come from

Every default above is an explicit prior. None of them is a measurement, and no
number in the two tables was fitted to any spectrum. That is the whole origin
statement and it is not softened anywhere else in this record.

What each prior asserts is an ordering and a rough size. The size was chosen
against the position term so that neither kind of evidence silently dominates
the other: if that term ends up a Gaussian log-likelihood, which is issue #32's
decision and not this record's, then a weight of -3.0 is the same penalty as a
position residual of sqrt(2 * 3.0), about 2.4 times the declared uncertainty.
An intercombination candidate is therefore about as disfavoured as a line
sitting 2.4 sigma off where it should be, which is disfavoured and not excluded.
The multipole priors sit further out because a run that treats E1, M1 and E2 as
equally likely triples its candidate volume and spends it on the rarest
explanations.

Each prior has a measurement that would replace it, and until that measurement
exists the value stays a prior and is labelled one in the output. For the spin
rule the measurement is the observed frequency of catalogued intercombination
lines among catalogued E1 lines in a comparable spectrum. For the multipole
priors it is the same count over catalogued M1 and E2 lines. Neither has been
run here.

Changing a weight is a tuning action, so it is governed by the tuning and
testing split that issue #45 defines. A weight adjusted
until a reserved spectrum looks better has turned this record into a knob, which
is the thing the board says it is replacing.

## The bound that keeps a weighted rule weighted

A weighted rule allowed to grow without limit becomes a hard rule without anyone
editing this file. So the configuration accepts a weight only in the closed
interval from -10.0 to 0.0 and refuses any value outside it, including negative
infinity and any value that is not a finite number. A candidate's total score is
therefore always finite, and no combination of weighted rules at any allowed
setting can drive a candidate's contribution to zero or remove it from the
answer.

Candidates leave the answer for exactly two reasons: a hard rule from the table
above, or a position that lies outside the tolerance the declared uncertainties
imply. Neither is a weight. Issue #27 carries the property test that asserts
this over the whole allowed weight range.

## What a run reports

Each candidate carries the multipole assigned to it, the hard rules that were
evaluated, the hard rules that could not be evaluated for want of a parity or a
J, and every weighted rule that fired with the weight it contributed. A reader
who disagrees with a prior can then see what it did to that candidate rather
than being told the total.
