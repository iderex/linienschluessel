# The gate this board is aiming at, and every deviation from it

Document for issue #50. The target is the gate the sso board runs. That board
is public, so the target is printed here and not paraphrased, and every
difference between it and this board is deliberate and carries its reason.

The two boards are different things. One is a plugin inside a managed runtime
and this one is a numerical engine, so parity here means the same standard of
evidence and not the same list of jobs.

## This file is re-derived, never remembered

Every list below is a transcript of one run on one date. It is not maintained by
hand and it is not the authority for what either board requires. Both rulesets
move, and a list in a document drifts against the thing it describes, so the
command is given beside each list and again at the end, where somebody comparing
the two boards will need it.

## The target, printed

    gh api repos/iderex/jellyfin-plugin-sso/rules/branches/main \
      --jq '.[] | select(.type=="required_status_checks")
            | .parameters.required_status_checks[].context'
    build
    ABI floor build
    Package (JPRM) / Build package
    Package (JPRM) / Generate SBOM
    CodeQL
    Analyze (csharp)
    DCO sign-off
    Deterministic PR-hygiene checks
    Enforce greppable invariants
    Reject Trojan Source Unicode
    Audit workflows (zizmor)
    prettier
    dependency-review

Run 2026-08-07. Thirteen contexts.

## What this board requires today

    gh api repos/iderex/linienschluessel/rules/branches/main \
      --jq '.[] | select(.type=="required_status_checks")
            | .parameters.required_status_checks[].context'
    DCO sign-off
    dependency-review
    Reject Trojan Source Unicode
    Audit workflows (zizmor)
    Coverage on the deciding surface

    gh api repos/iderex/linienschluessel/rules/branches/main \
      --jq '[.[].type] | sort | join(", ")'
    deletion, non_fast_forward, pull_request, required_status_checks

Run 2026-08-09. Five contexts, against thirteen on the target. That is a
deviation of content and no longer a deviation of kind: a red check here stops a
merge, so the word "exists" in the table below means the check runs, and for
these five it also means the check gates.

The number was zero until 2026-08-09 and the sentence that stood here said so.
While it was zero, every claim that a guard caught something rested on somebody
having looked at a run before the branch went away, because a red check on a
pull request that merges anyway leaves the same trace as a green one.

## What runs on a pull request and is not in that set

The required set is not every name a pull request reports under. On the head
commit of #104:

    gh api repos/iderex/linienschluessel/commits/c5de088/check-runs \
      --jq '.check_runs[] | "\(.name)\t\(.conclusion)"' | sort | uniq -c
      1 Analyze (actions)	success
      1 Analyze (rust)	success
      1 Audit workflows (zizmor)	success
      1 CodeQL	success
      1 Coverage on the deciding surface	success
      1 DCO sign-off	success
      1 dependency-review	success
      2 Every fixture carries its record	success
      2 Invariants from the decision records	success
      2 Reject Trojan Source Unicode	success
      1 zizmor	success

Read 2026-08-11. Eleven names, fourteen runs. The transcript that stood here was
taken on the head of #87 and printed six names and seven runs, at a commit where
the scanner, the invariant job and the fixture job had not reported on a pull
request at all.

Three names appear twice and it is one reason each time. `unicode-guard.yml`,
`invariants.yml` and `fixtures.yml` each trigger on both push and pull request
and name their job once, so a single context covers both runs and no second
context is owed.

Five of the eleven are the required set printed above. The other six report and
stop nothing: `Analyze (actions)`, `Analyze (rust)`, `CodeQL`, `Every fixture
carries its record`, `Invariants from the decision records` and `zizmor`. The
last of those is the workflow name beside the job name `Audit workflows
(zizmor)`, which is required, so one tool reports under two names of which one
gates.

One further name exists and cannot be required. `Scorecard analysis` runs on
push and not on a pull request, so it never reports on a head commit a merge is
waiting on:

    gh api repos/iderex/linienschluessel/commits/7e38065/check-runs \
      --jq '.check_runs[].name' | sort
    Audit workflows (zizmor)
    Coverage on the deciding surface
    Reject Trojan Source Unicode
    Scorecard analysis

Read 2026-08-09 on the merge commit of #87. A required context that never
reports would block every merge instead of gating one, so a check that runs
only on push is reported and never required, and that follows from its
trigger. It says nothing about the check's value.

## What is still not required, and is owed

`build`, `test`, `format` and `lint` are #5's and none of them exists yet, so
none of them is in the set above. Adding a check to the required set is a
second settings change after the check's first green run, and that ordering is
now the normal case here, no longer something decided once: a context cannot
be required before it has reported, and the set above was set from what had
already run.

## The map

Every row is a target context, this board's equivalent, whether that equivalent
exists here today, and the issue that owes it where it does not.

| Target context | Equivalent here | State | Owed by |
| --- | --- | --- | --- |
| `build` | A build of the workspace with the dependency lock asserted | owed | #5 |
| `ABI floor build` | A build against the minimum toolchain version the pinned file declares | owed | #4 for the pin, #5 for the job |
| `Package (JPRM) / Build package` | No counterpart. The release artefact per platform | owed | #63 |
| `Package (JPRM) / Generate SBOM` | A bill of materials for the release artefact | owed | #51 |
| `CodeQL` | CodeQL, unchanged in mechanism | exists | |
| `Analyze (csharp)` | The same analysis, on the two languages the platform derives here, reporting as `Analyze (rust)` and `Analyze (actions)` | exists | |
| `DCO sign-off` | The same check, already here | exists | |
| `Deterministic PR-hygiene checks` | The same, in two tiers | owed | #57 |
| `Enforce greppable invariants` | The same mechanism, entirely different content, reporting as `Invariants from the decision records` | exists | |
| `Reject Trojan Source Unicode` | The same check, already here | exists | |
| `Audit workflows (zizmor)` | The same check, already here | exists | |
| `prettier` | A formatter check, split by what it formats | owed | #5 |
| `dependency-review` | The same check, already here | exists | |

Seven of the thirteen exist here. The other six are owed and each names the
issue that owes it.

Existing and gating are not the same statement, and the split falls inside those
seven. Four of them are in the required set printed above: `DCO sign-off`,
`Reject Trojan Source Unicode`, `Audit workflows (zizmor)` and
`dependency-review`. The three that arrived since the count read four report and
stop nothing. So a row reading `exists` says the check runs, and the required-set
command is the authority for which of them a merge waits on.

## The deviations, and the reason for each

The floor build. The target board builds against an oldest supported host
because a plugin can be loaded by an older application than the one it was
compiled against. There is no host application here, so the purpose of that
job, catching a dependency on something the declared minimum does not have,
transfers to a build against the declared minimum toolchain version; it does
not disappear.

The packaging job. Its runtime half has no counterpart, because nothing here is
packaged for somebody else's runtime. Its useful half is the bill of materials
for what ships, and that transfers to the release artefact of #63 rather than to
a packaging step, so the two halves land in two different places.

The code scanner. Issue #50 asks whether the scanner supports the language
chosen here, as a fact to check rather than assume. It does. The GitHub
documentation on code scanning with CodeQL, read on 2026-08-07 from
https://docs.github.com/en/code-security/code-scanning/introduction-to-code-scanning/about-code-scanning-with-codeql,
lists the supported languages as C/C++, C#, Go, Java/Kotlin,
JavaScript/TypeScript, Python, Ruby, Rust, Swift and GitHub Actions workflows,
with Rust among them and carrying no separate qualification in that list. So
this deviation is a language change inside an unchanged mechanism, and not a
named replacement. If that changes, the replacement is named in #52 and this
paragraph is the thing to correct.

What the scanner's contexts are worth was measured rather than assumed, on #95,
a branch carrying three deliberate findings and closed unmerged. On its head
`ee61502`, `CodeQL`, `Analyze (rust)` and `Analyze (actions)` all report success
while `Audit workflows (zizmor)` reports failure, and the one alert filed against
that head came from zizmor:

    gh api 'repos/iderex/linienschluessel/code-scanning/analyses?ref=refs/pull/95/merge' \
      --jq '.[] | "\(.category)\t\(.tool.name)\tresults=\(.results_count)\trules=\(.rules_count)"'
    /language:rust	CodeQL	results=0	rules=25
    /language:actions	CodeQL	results=0	rules=17
    zizmor	zizmor	results=1	rules=1

Read 2026-08-11. Two things follow and both belong beside the row above. The
context that reports a scan is not the context that judges one, so a green
`CodeQL` is compatible with a finding sitting in the security tab, and what
went red there was a job re-running its tool and failing on the result, and no
alert did it. And forty-two rules across the two languages returned nothing on
three findings written to be found, which is a number worth carrying because a
scanner that finds nothing and a scanner with nothing to find leave the same
green check. Neither of those is repaired by this page. #52 holds them.

The formatter. The target board runs one formatter over what it holds. The
language here is different, so the check is different, and it splits: the source
has a formatter that ships with its own toolchain, and the markdown, the YAML
and the tracked data files do not. The check exists either way and #5 carries
the shape of it. What matters for parity is that formatting is decided by a
tool and refused by a job, not which tool.

The invariant check. The mechanism transfers unchanged and the content changes
entirely. On the target board the invariants are about authentication. Here they
are about tolerances, units and where the objective may be computed, and the
first three that the landed decision records already imply are that no constant
tolerance appears in the source, that no conversion between media happens
outside a reader, and that no module reachable from the objective references an
adapter. #53 carries them.

The coverage bar. On the target board it keys on the surface that decides
authentication outcomes rather than on the whole repository. The equivalent
surface here is the one that decides an assignment: the objective, the
probability model and the input readers. Whole-repository coverage is reported
and does not gate, for the same reason it does not there. The section below
names that surface precisely enough for a job to read, and #54 is where it was
argued.

## The surface coverage is measured over, and the bar

This block is data. `.github/workflows/coverage.yml` reads the surface and the
bar out of it and holds nothing of its own, so the two cannot drift apart and
moving a crate into or out of the surface is an edit to this page rather than a
line in a job nobody opens. A run that cannot find this block, or finds it
without a bar or without a crate, is red.

```coverage-surface
bar 85
crate spectro-contract
crate spectro-candidates
crate spectro-objective
crate assoc-posterior
```

Four crates, and each is here because a line in it decides an assignment rather
than because it is important. `spectro-contract` holds the readers, and a reader
that accepts a malformed file produces a confident wrong answer instead of an
error. `spectro-candidates` decides what the rest of a run is allowed to
consider at all. `spectro-objective` holds every score term. `assoc-posterior`
holds the posterior construction and the mass that stays on none of these, which
is the number a person would put in a paper.

A crate is named, not a directory and not a module path. The names are the
units `docs/decisions/layout.md` fixes, a crate cannot be added without
choosing a side, and the job refuses a name with no crate behind it, so a
crate that is renamed or removed reds this check instead of quietly leaving
the surface.

### The bar, and why that number

Eighty-five per cent of executable lines, over the four crates together.

It is set just under what the surface measures today, and no round number was
chosen in advance, so it bites on a real fall and not on the difference
between one test and the next. Today, with one of the four crates carrying
code:

    cargo llvm-cov --workspace --locked --json --output-path coverage.json
    jq -r '.data[0].totals.lines | "\(.covered)/\(.count) \(.percent)"' coverage.json
    941/1063 88.52304797742238

That whole-repository number and the surface number are the same number today,
because `spectro-contract` is the only crate in the tree with an executable line
in it. They separate the day a crate outside the surface grows code, and the job
prints both so that the day is visible.

It is a floor and raising it is a decision somebody makes here, in this file,
with the reason. A bar that tracks the measurement upward automatically is a
ratchet nobody chose, and the first change that has to lower it then argues with
a number rather than with a person.

### What this bar does not do

It is an aggregate, so a surface crate at zero per cent is hidden by a larger
one above the bar. That is a real hole and it is worth knowing which way it
points: three of the four crates named above are empty today, they contribute no
line to either side of the ratio, and the whole number is `spectro-contract`'s.
So this check currently measures the readers and says nothing about the other
three, and it will keep saying nothing about a crate until that crate has a line
in it. What it does do from the first such line is fall, because an untested
module arrives as uncovered lines in the denominator.

It gates. `Coverage on the deciding surface` is one of the five contexts the
ruleset requires, read at the top of this page on 2026-08-09, so a fall below
the bar stops a merge and does not only report one. The sentence that stood
here said this job reports and reports only, which was true while nothing on
this board was required.

What that does not change is the hole above it. A gate over an aggregate is
still an aggregate, so the number stopping a merge is one this section has
already said is `spectro-contract`'s alone.

One crate that decides an assignment is not on the list. `spectro-quantity`
holds the conversions of `docs/decisions/line-position.md` and is the only place
a medium is converted, so a fault there moves every position in a run. #54 names
the objective, the probability model, the candidate generation, the readers and
the none-of-these mass, and the conversions are in none of those five, so
adding it here would be a widening decided by whoever wrote the block. It is
written down instead, for whoever lands the conversions to raise.

## What is reported and does not gate

Three things run outside the required set on the target board and the same
posture is taken here deliberately.

Mutation testing over the scoring core, reported and not gating, with an
infrastructure failure in it made loud. A mutation score that gates becomes a
number people tune to, and a mutation run that fails to start and reports
nothing is indistinguishable from one that found nothing unless the failure is
made loud. #55 carries it.

Fuzzing, running out of band, with the seed corpus replayed inside the gating
job. That arrangement is what makes a parser regression red on the change that
caused it and not days later, and it is worth copying exactly; approximating
it with a nightly job alone is not the same thing. #56 carries it.

Whole-repository coverage, reported beside the bar that gates on the deciding
surface, so that a fall in coverage somewhere that does not gate is still
visible.

## The commands again

Here, where somebody comparing the two boards will want them.

    gh api repos/iderex/jellyfin-plugin-sso/rules/branches/main \
      --jq '.[] | select(.type=="required_status_checks")
            | .parameters.required_status_checks[].context'

    gh api repos/iderex/linienschluessel/rules/branches/main \
      --jq '.[] | select(.type=="required_status_checks")
            | .parameters.required_status_checks[].context'

Run both. A row in the table above that neither command supports is a row that
has gone stale, and the commands are the authority.
