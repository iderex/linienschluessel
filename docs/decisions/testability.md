# The default suite runs with no display, no elevation and no network

Decision record for issue #6. Three things the default test suite may not
require, stated before there is a suite, because each of them is cheap to keep
from the first test and expensive to recover once it has been broken.

No display. Nothing in the default suite opens a window, needs a desktop
session, or renders anything to a screen.

No elevation. Nothing in the default suite needs administrative rights, and
nothing in it can cause a consent or firewall dialog to appear. A contributor
whose suite stops on a dialog does not have a suite.

No network. Nothing in the default suite resolves a name, opens a connection or
depends on a server. Retrieving an upstream snapshot is a real thing this board
does, and it belongs to the harness of issue #7 rather than to the suite
everybody runs.

## Why elevation is the one to design against

The other two announce themselves. A test that needs a display fails on a
machine without one, and a test that needs the network fails when the network is
gone. A test that raises a consent dialog does neither: it waits, and the person
who cannot answer the dialog cannot run the suite at all.

The specific shape issue #6 names is a test binding a listening socket to the
machine's own interface address rather than to loopback. On Windows that raises
a firewall consent dialog, only an administrator can answer it, and the dialog is
keyed to the executable path, so a fresh build directory asks again. The rule
this record fixes is that no test in the default suite binds a listening socket
at all, which costs nothing on a board that reads files and writes files.

That claim was not put to the test here, and it is not going to be. Reproducing
the dialog means raising a consent prompt on somebody's machine, which is the
harm the rule exists to prevent, and a rule verified by committing the offence
is not worth what it costs. So this paragraph is an argument and not a
measurement, and it is written as one. What would settle it without the harm is
a job on a machine nobody is sitting at, which is where the enforcement belongs
anyway.

## What a green run does not tell you

The reason the rule needs a mechanism rather than a sentence is that the test
runner cannot tell a pure test from one that reaches out. A scratch crate with
two tests, one arithmetic and one opening a TCP connection to a public host, was
run with `cargo test`:

    running 2 tests
    test tests::a_difference_is_a_difference ... ok
    test tests::this_one_reaches_the_network_and_nothing_says_so ... ok

    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Two lines, both green, and nothing in the output distinguishes them. The toolchain
was cargo 1.97.0 on x86_64-pc-windows-msvc. Nothing in cargo is being blamed for
this: a test runner is not a sandbox and does not claim to be. What it means is
that an accidental fetch inside a test that looks pure is invisible to the
person who added it, to the reviewer reading the diff, and to every run of the
suite, until the day the far end is down and the failure arrives somewhere
unrelated.

So the network half is enforced by running the suite where there is no route,
and a test like the second one above turns red for the only reason that can be
trusted, which is that it could not do the thing it was not supposed to do.

## Where the enforcement goes, and that it is not here yet

Issue #6 asks this record to name the job that enforces the network half. There
is no such job, and naming one that does not exist would be the defect this
board's records are written to avoid. The workflows this repository has today
are what `git ls-files .github/workflows` prints, and none of them builds or
tests anything.

The job is issue #5's to create, since it is the same job that builds and runs
the suite, and the network half is one property of it rather than a job of its
own. The harness that is allowed the network is issue #7's. Until both exist,
this record is a rule a reader keeps, and issue #6 stays open with the reason
written into it.

The offline half was also not demonstrated on the machine this record was
written on. Removing a network route on Windows needs administrative rights, and
taking them to prove a rule about not needing them is the same trade refused
above. It is a job property, it is measured where the job runs, and it is not
measured here.

## What the rule costs

A path that genuinely needs the network, an instrument or a display is a path
the default suite does not cover, and the coverage of the harness that does
cover it is only as good as the frequency somebody runs it. That is the trade,
and it is the right one on a board whose ordinary contributor is running the
suite on a laptop between two other things.

It also means the readers of issue #20 and issue #21 are tested against files
rather than against the servers those files come from, so a change in an
upstream's output is caught by the harness or by a person, and never by the
suite. The adapters of issues #22 and #23 exist partly for that reason: a
synthetic file in an upstream's layout exercises every line of an adapter
without anybody's server being involved.
