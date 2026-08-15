# What a run records, so that it can be repeated

Decision record for issue #19. An answer that cannot be reproduced is a claim,
and this board is meant to produce evidence. Reproducing a run means the same
output bytes from the same inputs, and that is a property of four small
decisions that are easy to get wrong quietly: what order things are in, how they
are added up, where randomness comes from, and what the output says about
itself.

Nothing in this repository refuses a violation of this record today. There is a
source tree now, and the crate this record's rules belong to holds nothing.
`docs/decisions/layout.md` puts the ordered containers, the summation order, the
comparison rule and the seed derivation in `assoc-determinism`. Read at
a23385a:

    git grep -l '' -- '*.rs' | cut -d/ -f2 | sort -u
    spectro-contract

    git grep -c '' -- crates/assoc-determinism/src/lib.rs ; echo "exit=$?"
    exit=1

One crate in the workspace holds any Rust at all and it is not that one, so the
reason has moved from an empty repository to an empty crate and the conclusion
has not moved with it.

That is also why issue #19 is not closed by this file. Its Done-when asks for
a test that runs one input twice at different thread counts and compares the
output bytes, and for a linked run where a deliberately introduced
nondeterminism makes that test red. Both need a program, and what the tree now
carries is a reader and not a run.

## Three claims, kept apart

Only the first is this record's.

Two runs of one binary on one machine, given the same inputs, produce the same
answer bytes. That is what this record fixes, and it is the claim the test in
issue #19 makes.

Two runs of one version on two platforms agree. That is weaker, it is measured
and never assumed, and it is issue #39's, which states the tolerance the
scores are compared at and asserts the assignments and their ordering are
identical.

Two builds of one commit produce the same binary bytes. That is a different
property with different causes and it is issue #4's to demonstrate or to refuse
in writing. Nothing here asserts it.

## The canonical order

One ordering rule stands behind the tie-breaking, the summation and the
output, so it is stated once and referred back to, and never restated three
times.

Every identifier is compared as its raw UTF-8 byte sequence, shortest-prefix
first. Not as a locale collation, not case-insensitively, and not by any
normalisation of the text. A locale-aware comparison is the classic way a
program produces one order on the author's machine and another on somebody
else's, and it is a dependency this board does not need to acquire.

Features are ordered by `spectrum_id` and then by `feature_id`, both by bytes.
This is the order features are processed in, summed in and written in, and it is
derived from the identifiers rather than from the input file's row order, so a
producer who sorts their export differently gets the same answer. The cost of
that choice is that the identifiers have to be unique and stable, which
`docs/decisions/input-contract.md` already requires of both.

Transitions are ordered by `upper_level_id`, then `lower_level_id`, then the
multipole, all by bytes. The identity of a hypothesis is the ordered list of its
transitions under that rule, and hypotheses are compared by that list. Because
the empty list is a prefix of every other, the none-of-these hypothesis sorts
first without an exception being written for it.

That identity is finer than the structural key of
`docs/decisions/competitors.md`. The key is a pair of level multisets, so two
hypotheses over the same level pair under different multipoles share a key and
are two distinct hypotheses. The key is what competitor deduplication and
validation comparison use; the identity is what ordering uses. Using the key for
ordering would leave those two hypotheses tied with nothing to separate them.

Nothing ordered is read out of a hash container. `docs/decisions/means.md`
measured why: the hash map in this language randomises its iteration order
between runs of one binary, which is the hazard announcing itself loudly,
and the rule this record takes from it is that the container which would tempt
somebody is the one that fails first.

## Ties, and what the epsilon is actually for

Hypotheses of one feature are sorted by probability descending, and where two
probabilities are exactly equal the identity order above breaks the tie. That is
a total order, it is decided by the input identifiers, and it does not depend on
which candidate the search reached first.

The tolerance for treating two scores as equal is deliberately not part of that
sort. An epsilon-equality is not transitive, so a sort that used one would give
an order depending on which pairs happened to be compared, which is the same
class of defect as an iteration order and harder to see.

So `compare.score_epsilon` is a reporting threshold instead. After the exact
sort, any adjacent pair whose scores differ by no more than
`compare.score_epsilon * max(1, |a|, |b|)` is reported as an unresolved tie, on
both members, saying that their relative order was decided by the identity rule
and not by the evidence. A reader then knows that the second-ranked hypothesis
was not beaten, and the same pair reported in the other order on another
platform is the run's own declared behaviour rather than a contradiction.

The default is 1e-12, relative. It is an explicit prior and not a fitted number,
and the measurement that sets its scale is the one below.

## Summation

The objective adds many small numbers and compares sums, and floating point
addition is not associative, so the order is part of the answer. The size of
that effect was measured rather than assumed. This program, compiled with
`rustc -O` under rustc 1.97.0 on x86_64-pc-windows-msvc, adds the same thousand
values in the two directions:

    fn main() {
        let v: Vec<f64> = (1..=1000).map(|i| 1.0 / (i as f64)).collect();
        let fwd: f64 = v.iter().fold(0.0, |a, b| a + b);
        let rev: f64 = v.iter().rev().fold(0.0, |a, b| a + b);
        println!("forward  {:.17e}", fwd);
        println!("reverse  {:.17e}", rev);
        println!("equal    {}", fwd == rev);
        println!("diff     {:.4e}  rel {:.3e}",
                 (fwd - rev).abs(), (fwd - rev).abs() / fwd);
    }

    forward  7.48547086055034328e0
    reverse  7.48547086055034061e0
    equal    false
    diff     2.6645e-15  rel 3.560e-16

A relative difference of 3.560e-16 on one thousand terms, which is enough to
decide which of two close candidates wins. The default epsilon above is about
three thousand times that,
which is headroom for a spectrum with many more terms than a thousand and is not
a number fitted to anything.

The rules that follow from it.

Every sum over a set is taken in the canonical order of that set. A sum whose
order is inherited from a container, a thread schedule or an insertion sequence
is the defect this rule exists against.

The reduction is sequential in the first release. `docs/decisions/means.md`
records that this language adds no parallelism unless a dependency is chosen for
it, so the day a reduction is split across threads is a deliberate change with a
summation order attached to it. When that day comes, the split carries a fixed
block size and a fixed tree shape, both derived from the canonical order and
neither from the thread count, and it becomes a different value of
`summation_order` in the provenance. Two runs with different `summation_order`
values are not expected to agree byte for byte, and the field is what lets a
reader see that rather than infer it.

No fused multiply-add is used. `docs/decisions/means.md` measured the two forms
producing different results in this language, `1.734723475976807e-18` against
`9.020562075079397e-19`, and recorded that choosing between them is the author's
act. This record makes the choice: the separate form everywhere, and any use of
the fused form is a named exception in the source with its reason.

## Seeds

Any stochastic search carries a seed, the seed appears in the output, and a run
repeated with that seed reproduces the answer.

The root seed is a field the operator may set. Where it is not set, it is
derived from the input digests rather than from the clock or from the operating
system's entropy, so that a run nobody thought about seeding is still
reproducible and a second operator with the same files gets the same number. It
is written into the answer either way, and there is no path by which a run has a
seed the answer does not carry.

Streams are derived per unit of work, by a keyed derivation from the root seed
and a stable label such as the spectrum identifier or the null's region index.
They are not drawn in sequence from one generator. A single global stream makes
the numbers a spectrum receives depend on how many other spectra were processed
before it, so adding an unrelated spectrum to a run changes the answer for every
spectrum after it, and the change looks like evidence. The derivation function is
named in the provenance, because changing it changes every stochastic answer
this board has ever produced and that has to be visible.

## What the output carries

Two blocks, and the split between them is the point.

The reproducible block is compared byte for byte by the test issue #19 owes.
Every field in it is a function of the inputs, the parameters and the engine
version, and of nothing else.

| Field | What it holds |
| --- | --- |
| `engine_version` | The released version string |
| `engine_commit` | The commit the binary was built from |
| `input.role` | `level_set`, `line_list`, `rates` or `covariance`, one entry per file read |
| `input.name` | The name the operator gave the file, never its path |
| `input.digest_algorithm`, `input.digest` | The digest of the bytes actually read, which is issue #25 |
| `input.declared_id` | The `level_set_id` or `line_list_id` the file declares |
| `input.contract_version` | The version marker the file declares |
| `input.retrieval_record` | The retrieval record supplied with the file, where one was |
| `profile` | The named profile the run used |
| `seed` | The root seed, set or derived |
| `seed_derivation` | The name of the stream derivation function |
| `parameters` | Every switch in force, with its value and whether it is the default or was overridden |
| `tolerance_rule` | The name of the rule that derives a tolerance from declared uncertainties, never a constant |
| `null_procedure` | The null of `docs/decisions/chance-coincidence.md` and its parameters |
| `posterior_procedure` | The approximation used for the marginal, with `posterior.samples`, `posterior.marginal_tolerance` and the marginal error achieved |
| `summation_order` | The identifier of the summation rule in force |
| `compare.score_epsilon` | The reporting threshold above |
| `report.competitor_floor` | The floor of `docs/decisions/competitors.md` |
| `validation_record` | The validation record of the engine version, which is issue #49, and the explicit statement that there is none where there is none |

The envelope is everything true of this particular execution and of no other. It
is written beside the reproducible block, it is excluded from the byte
comparison, and it holds `started_at`, `finished_at`, the wall time, the thread
count, the target triple the binary was built for, and the digest of the
reproducible block.

The thread count is in the envelope rather than in the block because that is the
whole claim: changing it changes the envelope and must not change a byte of what
the envelope's digest covers. A target triple names a platform and is what issue
#39 compares across; it is not a machine name, and no machine name, user name or
absolute path appears in either block. That allowlist is issue #59's and this
table is the list it holds.

Nothing outside the two lists is written. A field arriving because it happened to
be in a variable is the leak issue #59 describes, and an allowlist is what makes
that a decision rather than an accident.

## Quoting a run

`docs/decisions/probability-model.md` refuses any sentence giving a
probability without naming the level set and the line list it is conditional
on, and points here for identifiers short enough to carry. They are: the
declared identifier and the first bytes of the digest for each input, the
profile, the seed and the engine version.

Whether a run also gets a minted identifier, and whether the record behind it is
published, is entry 10 of issue #1. It is the maintainer's to answer and nothing
here decides it. What this record fixes is that everything such an identifier
would have to stand for is already in the answer file, so the answer does not
have to be reissued whichever way that goes.

## What the test owes when it lands

It runs one fixture twice at two different thread counts and compares the
reproducible block byte for byte, having first asserted that the two envelopes
differ, so that a test comparing two identical empty outputs cannot pass for the
real one.

It runs one fixture twice with the features presented in two different row
orders and compares the same bytes, which is the half that proves the canonical
order is derived from identifiers rather than from the file.

And the pull request that lands it links a run where a deliberately introduced
nondeterminism makes it red. The near-miss worth spending the effort on is not a
random number with no seed, which any test would catch. It is a sum taken in the
iteration order of a hash container over a set whose canonical order is almost
always the same, so that the test fails on a small fraction of runs, since that
is the mistake somebody will actually make and the one a single green run does
not refute.
