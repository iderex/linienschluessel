# The input contract

Decision record for issue #18. This board consumes level sets and line lists
that other people produce. It does not carry a copy of anybody's table, because
three copies of one table drift apart and the drift is found by whoever first
compares two of them.

Neither of the sibling efforts this board is meant to consume has a released
format, so waiting for one means starting nothing and inventing one for them
risks a contract they cannot emit. What follows is defined as this board's input
requirement only. It asks for what the objective uses and nothing else, it
carries a version marker from its first commit, and it is checkable by a
validator any producer can run without importing anything from here.

This record covers the first half of what issue #18 asks for. The validator it
describes does not exist, so #18 is not closed by it, and the reason is written
into #18 rather than left to be inferred from an open issue.

## The means

A UTF-8 text file with a labelled header and a tab-separated body. No new
language, no runtime and no dependency, which is the same test
`docs/decisions/means.md` applies to everything else here.

The header is what a bare tabular format cannot carry and is the reason for the
whole shape: the unit, the medium, the reference point and the contract version
are properties of a file rather than of a row, and a format with no place to
state them is a format that invites the guessing
`docs/decisions/line-position.md` forbids. Text rather than a binary columnar
format because an input to this board has to be readable by the spectroscopist
who produced it, diffable in a history, editable by hand to build a fixture, and
exact at the byte level so that a reader can be tested against a carriage return
it was given. A binary format costs a dependency and gives up all four.

The rejected candidates. JSON, because ten thousand rows of it are unreadable by
a person and the format has no natural place for a header that governs the
table. A bare comma-separated file, because it has nowhere to declare a unit or
a medium, which is the one thing this contract exists to require. Parquet or
HDF5, because they add a dependency to every producer that wants to emit one and
cannot be inspected without it.

## Version, and the rule for changing it

The header carries `contract_version`, as a major and a minor number.

A minor increment may only add optional fields and add members to a vocabulary
where the reader already has a defined behaviour for a value it does not
recognise. A reader written for an earlier minor accepts a later one, reports
every unknown field and unknown vocabulary member it saw, and does not silently
drop them.

Anything else is a major increment: removing a field, making an optional field
required, changing a field's meaning, changing a unit's default, or narrowing a
vocabulary. A reader refuses a major version it was not written for and names
the version it found and the versions it knows. It does not attempt a best
effort read, because a best effort read of a file whose fields have changed
meaning is the plausible wrong number this board is built to avoid.

An absent `contract_version` is refused. It is not defaulted to the current one.

## The level set

One file. Every field below is required unless it is marked optional, and each
one is here because the objective would refuse to run without it.

Header fields.

| Field | Why the objective needs it |
| --- | --- |
| `contract_version` | The rule above |
| `energy_unit` | Positions are compared in one representation and the reader converts once, which is `docs/decisions/line-position.md` |
| `energy_reference` | A difference is independent of the reference point only when both levels share one, so a file that does not say cannot be checked |
| `level_set_id` | Every probability is conditional on the level set, and the identifier is what an answer quotes |
| `covariance_file` (optional) | Level uncertainties from one fit are correlated, and `docs/decisions/uncertainty-model.md` uses the real off-diagonal term where it is supplied |
| `derived_from_line_lists` (optional) | Where the lines being assigned were inputs to the fit that produced these levels, the residual is small by construction, and the run has to say so wherever a probability appears |

Row fields.

| Field | Vocabulary or type | Why the objective needs it |
| --- | --- | --- |
| `level_id` | text, unique in the file | Points back at the source record and names the level in every answer |
| `species` | text, one canonical spelling, per row | A level set that mixes ionisation stages is a normal thing to be given, and intensity ordering is within one species in one spectrum |
| `energy` | number, in `energy_unit` | The Ritz difference |
| `energy_uncertainty` | number, or `none` | The tolerance of issue #26 and the position term of issue #32 are derived from declared uncertainties |
| `uncertainty_kind` | `standard_deviation`, `spread`, `class` | The three enter the uncertainty model as different distributions, and a spread is never printed as a standard deviation |
| `uncertainty_class` | text, required when `uncertainty_kind` is `class` | Carries the upstream's own label so the flag can travel into the answer |
| `parity` | `even`, `odd`, `unknown` | Every hard rule in `docs/decisions/selection-rules.md` needs it |
| `j` | non-negative half-integer, or `unknown` | The same |
| `origin` | `measured`, `predicted` | The two carry different uncertainty structures and a set that mixes them without saying which is which cannot be used |
| `configuration` | text, verbatim, may be empty | The magnetic dipole configuration rule, and it is the first thing a spectroscopist reads |
| `term` | text, verbatim, may be empty | The same |

`unknown` and an absent column are different statements and the contract keeps
them apart. A level whose J was never determined carries `unknown`, which is a
fact about the spectrum. A file with no `j` column at all is a fact about the
file, and it is refused, because a contract that accepted it would leave the
reader unable to tell the two apart afterwards.

`configuration` and `term` may be empty and the column may not be absent, for
the same reason.

## The covariance companion

Where `covariance_file` is present it names one further file in the same
directory, and the input is then the pair. That is the one place this contract
is not a single file, and it is stated here rather than left as a surprise: a
dense covariance over a few hundred levels is tens of thousands of numbers and
does not belong inline in a file somebody reads.

The companion carries the same `contract_version` and `level_set_id`, and its
body is `level_id` by `level_id` by value, upper triangle including the
diagonal. A pair not listed is zero. A `level_id` in it that is not in the level
set is refused rather than ignored.

## The line list

One file.

Header fields.

| Field | Why the objective needs it |
| --- | --- |
| `contract_version` | The rule above |
| `position_unit` | As for the level set |
| `position_medium` | The default medium for rows that do not override it |
| `line_list_id` | Every probability is conditional on the line list |

Row fields.

| Field | Vocabulary or type | Why the objective needs it |
| --- | --- | --- |
| `feature_id` | text, unique in the file | Names the feature in every answer and in every unassigned report |
| `spectrum_id` | text | Intensity ordering and the branching constraint hold within one spectrum, and the calibration terms are fitted per spectrum |
| `segment_id` | text, optional | Where a producer knows its grating orders, the calibration terms are carried per segment, which is the unit a calibration was done on |
| `position` | number, in `position_unit` | The observation |
| `position_medium` | `vacuum`, `air_standard`, optional per row | Overrides the header default, because the medium of a number can vary within one source's output |
| `position_uncertainty` | number, or `none` | As for the level set |
| `uncertainty_kind` | `standard_deviation`, `spread`, `class` | As for the level set |
| `uncertainty_class` | text, required when `uncertainty_kind` is `class` | As for the level set |
| `intensity` | number, optional | Enters only as an ordering constraint, which is `docs/decisions/intensities.md` |
| `intensity_scale` | `arbitrary`, `photographic`, `photon`, `energy`, required when `intensity` is present | The branching constraint takes a different form on a photon scale than on an energy scale, and the record requires the label rather than a guess |
| `flags` | comma-separated from `saturated`, `self_absorbed`, `blended`, may be empty | Three of the four conditions under which the branching constraint applies |
| `ritz_position` | number, optional | Reported and never read by the objective, for the reason below |

A Ritz position was computed from a level set, so scoring it against a
difference of that same level set measures arithmetic rather than a spectrum. It
is in the contract because a producer that has one should not have to throw it
away to conform, and no field the objective reads may be filled from it.

`position_medium` is a value, never a rule. A header saying that the medium
follows some source's own boundary convention is refused. Resolving a source
convention into an explicit per-row medium is exactly what an adapter is for,
and a contract that accepted the rule instead of the answer would have taken one
upstream's convention into itself.

## The transition rates, which are a third table

The two contracts issue #18 names are one table short, and the shortfall is
worth stating rather than discovering later.

The branching constraint needs a transition rate per transition. A transition is
a pair of levels, so a rate is a property of neither an observed feature nor a
single level, and it cannot be a column in either file above. It arrives as an
optional third table keyed by `upper_level_id` and `lower_level_id` against the
same `level_set_id`, carrying the rate, its unit and its uncertainty.

Optional, and the board runs without it. A shared upper level whose rates are
missing is issue #34's case, and the run reports every upper level where the
constraint was not applied together with which condition failed, which is the
rule `docs/decisions/intensities.md` already sets.

## Adapters, and where the boundary is

Nobody has a file in this contract and everybody has an export from a public
database, so adapters are what make the board usable. They are also where one
upstream's column layout would become an assumption inside the scoring function
if the boundary were not held.

The boundary is a property rather than a directory: no module reachable from the
objective may reference an adapter module, and an adapter's only output is a
conforming file or the in-memory equivalent of one. The shape the source tree
takes to express that is issue #3's decision, and the check that refuses a
violation is issue #22's, which also carries the run showing a deliberately
added import make it red.

An adapter that cannot supply a required field does not invent one. It reports
the field as unavailable from that upstream, and the run then behaves as the
uncertainty record's absent-uncertainty case describes.

## A file, not a service

A conforming input is a file on disk. Nothing in this contract may be satisfied
by a network call, and nothing in it names a host.

That is what makes the default test suite of issue #6 able to run with no
network at all, and it is what makes a run reproducible three years later
against a snapshot rather than against whatever a server returns that day.
Retrieving a snapshot is a real operation this board performs and it happens
before a run, not inside one, which is issue #25.

## What the validator has to do, and where it now is

A validator exists. It is `crates/spectro-contract`, it landed under issue #18,
which is closed, and the entry points are the five functions the crate exports:

    git grep -n 'pub fn validate_' -- crates/spectro-contract/src/lib.rs
    crates/spectro-contract/src/lib.rs:44:pub fn validate_level_set(bytes: &[u8]) -> Result<(), Refusals> {
    crates/spectro-contract/src/lib.rs:49:pub fn validate_line_list(bytes: &[u8]) -> Result<(), Refusals> {
    crates/spectro-contract/src/lib.rs:54:pub fn validate_rate_table(bytes: &[u8]) -> Result<(), Refusals> {
    crates/spectro-contract/src/lib.rs:59:pub fn validate_covariance(bytes: &[u8]) -> Result<(), Refusals> {
    crates/spectro-contract/src/lib.rs:68:pub fn validate_input(

Read at a23385a. The sentence that stood here, that no validator exists and the
list below is a requirement rather than a description, is no longer the state of
the repository.

What the tree above is and is not evidence of. That the five functions exist was
read off it. Whether every clause below is discharged by them was not measured
in writing this record and nothing here asserts it.
`crates/spectro-contract/tests/` is where that argument lives and issue #18 is
where it was made. What those files hold, and it is the whole of what this
workspace runs:

    git grep -c '#\[test\]' -- crates/spectro-contract/tests
    crates/spectro-contract/tests/the_level_set_reader.rs:9
    crates/spectro-contract/tests/the_validator_refuses.rs:48

    cargo test --offline 2>&1 | grep -oE '^test result: ok\. [0-9]+ passed' \
      | awk '{s+=$4} END {print s}'
    57

What the crate is answerable to, which is this record's half of it. It accepts a
conforming level set, line list and rate table. It refuses, naming the field and
the line: an absent
`contract_version`; a major version it does not know; an absent required column,
which is not the same as an empty value in a present one; a value outside a
declared vocabulary; an `intensity` with no `intensity_scale`; an
`uncertainty_kind` of `class` with no `uncertainty_class`; a `position_medium`
that is a rule rather than a value; a covariance entry naming a level that is
not in the set; and a `j` that is neither a non-negative half-integer nor
`unknown`. It runs with no network and it is the same code the readers use, so
that a file the validator accepts cannot be a file a reader then rejects.
