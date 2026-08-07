# The gate this board is aiming at, and every deviation from it

Document for issue #50. The target is the gate the sso board runs. That board is
public, so the target is printed rather than described, and every difference
between it and this board is deliberate and carries its reason.

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
    (no output)

    gh api repos/iderex/linienschluessel/rules/branches/main \
      --jq '[.[].type] | sort | join(", ")'
    deletion, non_fast_forward, pull_request

Run 2026-08-07. No status check is required to merge here.

That is the largest deviation on this page and it is not a deviation of content.
Four workflows run on a pull request in this repository and pass, and none of
them can stop a merge, so every claim that a guard caught something rests on
somebody having looked. Issue #69 carries it, and until it lands, the word
"exists" in the table below means the check runs and reports, never that it
gates.

## The map

Every row is a target context, this board's equivalent, whether that equivalent
exists here today, and the issue that owes it where it does not.

| Target context | Equivalent here | State | Owed by |
| --- | --- | --- | --- |
| `build` | A build of the workspace with the dependency lock asserted | owed | #5 |
| `ABI floor build` | A build against the minimum toolchain version the pinned file declares | owed | #4 for the pin, #5 for the job |
| `Package (JPRM) / Build package` | No counterpart. The release artefact per platform | owed | #63 |
| `Package (JPRM) / Generate SBOM` | A bill of materials for the release artefact | owed | #51 |
| `CodeQL` | CodeQL, unchanged in mechanism | owed | #52 |
| `Analyze (csharp)` | The same analysis on this board's language | owed | #52 |
| `DCO sign-off` | The same check, already here | exists | |
| `Deterministic PR-hygiene checks` | The same, in two tiers | owed | #57 |
| `Enforce greppable invariants` | The same mechanism, entirely different content | owed | #53 |
| `Reject Trojan Source Unicode` | The same check, already here | exists | |
| `Audit workflows (zizmor)` | The same check, already here | exists | |
| `prettier` | A formatter check, split by what it formats | owed | #5 |
| `dependency-review` | The same check, already here | exists | |

Four of the thirteen exist here. The other nine are owed and each names the
issue that owes it.

## The deviations, and the reason for each

The floor build. The target board builds against an oldest supported host
because a plugin can be loaded by an older application than the one it was
compiled against. There is no host application here, so the purpose of that job,
catching a dependency on something the declared minimum does not have,
transfers to a build against the declared minimum toolchain version rather than
disappearing.

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
and does not gate, for the same reason it does not there. #54 carries it.

## What is reported and does not gate

Three things run outside the required set on the target board and the same
posture is taken here deliberately.

Mutation testing over the scoring core, reported and not gating, with an
infrastructure failure in it loud rather than silent. A mutation score that
gates becomes a number people tune to, and a mutation run that fails to start
and reports nothing is indistinguishable from one that found nothing unless the
failure is made loud. #55 carries it.

Fuzzing, running out of band, with the seed corpus replayed inside the gating
job. That arrangement is what makes a parser regression red on the change that
caused it rather than days later, and it is worth copying exactly rather than
approximating with a nightly job alone. #56 carries it.

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
