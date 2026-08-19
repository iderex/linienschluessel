# Security policy

## What this repository is, before anything else

This is a Rust workspace for assigning observed spectral lines to level
transitions. `Cargo.toml` lists fourteen crates. Ten of them hold a `lib.rs`
with no bytes in it, and the binary, `crates/linienschluessel/src/main.rs`, is
`fn main() {}`. Every line of Rust that does anything sits in three crates:

- `crates/spectro-contract`, the readers and the validator for the input files,
- `crates/spectro-adapters`, which converts a NIST Atomic Spectra Database
  export into a file those readers accept,
- `crates/assoc-model`, which holds the assignment types and performs no I/O.

The solver, the objective, the posterior and the report are empty crates, and
running the binary does nothing. So this policy is about libraries that turn
bytes somebody else produced into structure a later run would trust, and not
about a finished tool. The README still says two crates hold code; three do. The
tree is the thing to trust, and this file was written from it.

## Where to report

Private vulnerability reporting on this repository:

    https://github.com/iderex/linienschluessel/security/advisories/new

That channel answers today. Measured rather than assumed, on 2026-08-19:

    gh api repos/iderex/linienschluessel/private-vulnerability-reporting
    {"enabled":true}

If that ever prints `{"enabled":false}`, the link above is a door that does not
open, and the honest fallback is a public issue with the mechanism described and
the crashing bytes attached.

I promise no acknowledgement deadline, and I would rather say so than print one.
There is no rota behind this repository. A reporter told to expect an answer
within some stated number of days, who then hears nothing on the day after,
cannot tell a slow reply from a report that never arrived, and has to make a
disclosure decision on that guess. Holding no date from me is worse than a date
I keep and better than a date I miss.

## The surface that is actually here

Four places, each a path from bytes I did not write to a value this program
would go on to believe.

`Document::lex` in `crates/spectro-contract/src/document.rs`. It takes `&[u8]`
rather than `&str` on purpose, strips a UTF-8 byte order mark, refuses a UTF-16
one, validates UTF-8, then splits on newlines and tabs. It is what the four
contract readers share, and it is not the only lexer in this tree. Those four
call sites are all of them:

    git grep -n 'Document::lex' -- crates
    crates/spectro-contract/src/covariance.rs:44
    crates/spectro-contract/src/level_set.rs:73
    crates/spectro-contract/src/line_list.rs:72
    crates/spectro-contract/src/rates.rs:51

Each adapter `convert` runs a second lexer of its own instead, and neither of
them names `Document`: `std::str::from_utf8` over the whole slice, a split on
newlines for lines, a stripped trailing carriage return on each line, and a
split on tabs for cells. That second lexer strips no byte order mark, so a mark
`Document::lex` would have removed lands inside the first cell of the first
line, and stripping a carriage return line by line accepts the mixed LF and CRLF
file `document.rs` refuses by rule. Two lexers over the same shape of file,
disagreeing on what they accept, is worth a report by itself, and it also means
a finding against one of them says nothing about the other.

The five public validators in `crates/spectro-contract/src/lib.rs`:
`validate_level_set`, `validate_line_list`, `validate_rate_table` and
`validate_covariance`, each taking a byte slice, and `validate_input`, which
takes a level set with an optional covariance companion and an optional rate
table and makes the cross-file checks no single file can make alone. This is the
intended front door, and a caller is expected to hand it whatever an operator
had on disk.

`nist_asd_lines::convert` and `nist_asd_levels::convert` in
`crates/spectro-adapters/src/`. These parse an upstream export, so those bytes
sit further from the operator's control than their own measurement does.

The workflow files under `.github/workflows/`. Eight of the nine run on
`pull_request`, which means they run on a branch that lives in a fork, and that
makes an injectable expression or a job permission wider than the step needs a
genuine finding. `Audit workflows (zizmor)` is a required status check for that
reason. `scorecard.yml` is the one that does not run on `pull_request`: it runs
on `push` to `main`, on a weekly schedule and on `branch_protection_rule`.

Concretely: a byte sequence that panics a reader, one that does not terminate or
allocates without bound on a small file, or one accepted where the contract says
it must be refused, in a way that lets a bad row travel onward silently. A panic
is not a matter of taste here. The replay in
`crates/spectro-adapters/tests/the_seed_corpus_replay.rs` already asserts that
no committed seed panics a reader, so bytes that do break a property this
project claims.

One I have not settled. `Refusal` in `crates/spectro-contract/src/refusal.rs`
implements `Display` and its reasons interpolate the failing cell verbatim, so
whatever prints a refusal prints bytes from the input file. If you can take that
further than text on a terminal, I want the report.

## What is not a vulnerability here

Nothing in a run reaches the network. `docs/data-on-the-host.md` carries the
word list, the search over `crates`, and its clean result, and it is honest that
a grep cannot prove the property: a dependency could open a socket on this
program's behalf without any of those words appearing in this tree. The reading
that closes that gap today is `Cargo.lock`, which names the fourteen workspace
members and nothing else. There are no third-party dependencies at all, so there
is nothing for that caveat to hide in yet, and that stops being true the day the
first dependency lands. Until then, reports about transport security,
certificate handling, telemetry or an update check are not about this program,
because none of those exists here to be wrong. Retrieving a spectrum from an
upstream is the operator's act and not the program's.

There are no accounts, no roles, no sessions, no privilege boundary, nothing
that listens and no rendered page. Account takeover, privilege escalation,
session fixation, CSRF, XSS and SQL injection have no machinery here to happen
in, and a report naming one would be describing some other program.

Nothing is distributed. Zero releases, zero tags, `publish = false` across the
workspace. There is no artifact anybody downloads, so there is no build to have
been compromised on the way to a user, and no version matrix either: `main` is
the only thing there is to fix.

The ten empty crates are empty. A report that `spectro-objective` or
`assoc-solve` fails to check something is a report about a file with no bytes in
it, and the answer is that the decision behind that crate is not made yet.

A wrong assignment is not a security report. This project exists because of the
plausible wrong number, so a reader that accepts a malformed row matters to me a
great deal, but it belongs on the public issue tracker where it can be argued
against the record it contradicts. Send it privately only if you can show it
crossing into a crash or into memory unsafety.

What you load and the terms attached to it are yours. `docs/sources.md` carries
the terms of every source this program can read, with the date each was read.

## If you send one

Attach the bytes, as a file or base64, rather than a description of them, and
name the entry point you handed them to and the commit you were on. I would
rather have a seed I can add to the corpus than a paragraph I have to
reconstruct. I will credit you unless you tell me not to, and there is no
bounty.
