# Nothing about the operator or their data leaves the host

The rule for issue #58. The material this board is pointed at is often somebody's
unpublished measurement, and the person running it is usually working on a
spectrum nobody else has seen. So a run reads files and writes files, and it
does nothing else.

Nothing about the operator or their data leaves this machine unless the operator
deliberately sends it. That means no telemetry, no usage counting, no crash
reporting, no update check and no remote configuration. Not off by default. Not
present.

The list is written out because each of those five is normally argued for on its
own and each of them is a channel. An update check is the one that gets waved
through most often: it sends a version, an address and a time to somebody else's
server every time the tool starts, which is enough to say who is working on what
and when, and it is exactly the kind of fact an operator working on an
unpublished spectrum has a reason to keep.

## Why this is a separate rule from the offline suite

Issue #6 already says the default test suite runs with no network route, and the
two rules look the same from a distance. They are not, and either could be
relaxed without the other.

The testability rule is about a contributor being able to run the suite on a
laptop, and it constrains the tests. This rule is about an operator's data, and
it constrains the program the operator runs. A suite could be made to allow the
network for a good reason and this rule would still stand; the program could
grow one network call and every test would still pass offline, because a call
that never happens in a test is not a call a test can see.

They also fail differently. A broken testability rule stops a contributor. A
broken version of this rule is silent, works perfectly, and is discovered by
somebody reading a firewall log.

## Where the inputs come from, and why a run still needs nothing

Retrieving a level set or a line list from an upstream is a real operation, and
it is the operator's, not the run's. `docs/decisions/input-contract.md` fixes
that a conforming input is a file rather than a service, so an assignment run is
handed files that already exist on the host. The adapters of issues #22 and #23
convert an export that is already on disk. The harness of issue #7 is where a
path that genuinely needs the network lives, and it is not part of a run.

That is what makes the rule checkable rather than aspirational. There is no
legitimate network call anywhere inside a run, so the invariant does not need an
exception list, and an exception list is the thing that turns a rule of this kind
into a formality.

## The check, and what it is worth today

The greppable half is a word list run over the crates. The words are the ones
that reach a socket in this language, whether through the standard library or
through the common clients:

    std::net  TcpStream  TcpListener  UdpSocket  SocketAddr  to_socket_addrs
    reqwest  hyper  ureq  curl::  tokio::net

Run against this repository as it stands:

    git grep -nE 'std::net|TcpStream|TcpListener|UdpSocket|reqwest|hyper|ureq|curl::|tokio::net|SocketAddr|to_socket_addrs' -- 'crates' ; echo "grep exit=$?"
    grep exit=1

That result is worth more than it did and still not what this record claims, and
both halves are printed, because a reader who sees a clean grep should also see
how much of this board it covered. Read at dde1110:

    git ls-files 'crates/*/Cargo.toml' | wc -l
    14
    git ls-files 'crates/*/src/lib.rs' | wc -l
    13
    git grep -l '' -- 'crates/*/src/lib.rs' | wc -l
    3

    git grep -l '' -- '*.rs' | cut -d/ -f2 | sort -u
    assoc-model
    linienschluessel
    spectro-adapters
    spectro-contract

    git grep -c '' -- 'crates/*/src/*.rs' 'crates/*/tests/*.rs' \
      | awk -F: '{f++; s+=$NF} END {print f" files, "s" lines"}'
    26 files, 5946 lines

Fourteen crates. Ten of the thirteen libraries hold a `lib.rs` with nothing in
it, every line of Rust in this repository sits in the other three, and the
fourteenth crate is the binary, whose `main.rs` does nothing. What changed since
the count above was last taken is the one thing this record had singled out. It
said the parts of a run with a reason to open a socket were the retrieval paths
of issues #22 and #23, and that neither was written. Both are written now, and the
grep covers both. Neither opens a socket, because `docs/decisions/input-contract.md`
makes retrieval the operator's act and hands an adapter bytes that are already on
disk.

So the clean result covers the two places this record expected the violation to
appear, which is more than a clean search over an empty tree and is still not
the property. Ten crates hold nothing, the objective and the solver among them,
and a grep can be walked around by a dependency that opens the socket on the
program's behalf. The sentence has changed three times and the conclusion has
not.

The half that will be worth something is a run of the whole pipeline with no
network route available, asserting it succeeds. A grep can be walked around by
a dependency that opens the socket on the program's behalf; a run with no route
cannot, because the thing either happens or it does not. Both are wanted, and
neither exists yet.

## What an operator can check for themselves

Disconnect the machine and run the tool. The rule says the answer is identical
and the run does not complain, and that is a thing an operator can do without
trusting a sentence in this file, which is the point of stating it here.

Nothing in this section asks the operator to inspect processes or to hold
administrative rights. A check that needs elevation is a check most people will
not run.

## If federation is ever added

It would be a per-action decision by the operator, and the action would show what
it is about to send before it sends it.

It would not be a setting. A setting somewhere that quietly enables sharing is
the exact shape this rule exists to forbid: it moves the decision away from the
moment the data leaves, it is made once and forgotten, and it is inherited by
everyone who copies a configuration.

Nothing about federation is planned. This paragraph exists so that if the
question is ever opened, it is opened against a shape that was decided before
anybody wanted a particular answer.

## What is enforced, and what that enforcement cannot do

One half of the greppable rule above is refused by a machine, and it was not
when this record landed. `.github/workflows/invariants.yml` carries the word
list of this record as a rule named `no-network-call-inside-a-run`, searching
every path under `crates`, and the rule names this record so that a contributor
who trips it is sent to the argument rather than to a pattern. It was made red
on a seeded call, `pub fn local(addr: std::net::SocketAddr)` in
`assoc-determinism`, in
https://github.com/iderex/linienschluessel/actions/runs/31306510501, which is
issue #53's change and is where that proof lives.

Three things that check cannot do, and none of them is small.

It does not stop a merge. The branch ruleset's required contexts are what
`gh api repos/iderex/linienschluessel/rulesets` prints, and the invariants job is
not among them, so a red run is read by a person or not at all.

It reads bytes rather than behaviour. A dependency that opens a socket on this
program's behalf carries none of those words into this tree, and the rule cannot
see it. That is the case the offline run exists for.

It covers what the grep covers, which is the section above and its four crates
out of fourteen.

The offline pipeline test is the half with no mechanism at all. It needs a run
to make, and the job it would run in is issue #5's. Until it exists this record
is held there by whoever reads a change, and issue #58 stays open holding it.
