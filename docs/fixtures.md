# Fixtures this repository is allowed to carry

The rule for issue #8. A fixture decides what the suite is able to prove, so
every fixture says where it came from, under what terms it is here, and whether
it is real or made up. A file that says none of those is refused rather than
quietly trusted.

This repository carries no fixture today:

    git ls-files 'tests/fixtures' 'tests/fixtures/*' ; echo "exit=$?"
    exit=0

Whether real extracts may be carried at all is entry 2 of issue #1 and is the
maintainer's to answer. The rule below does not depend on that answer: it says
what a fixture has to carry, and it is the same requirement whether the file is
an extract of somebody's published table or a file this repository generated.

## What every fixture carries

One record per fixture file, in a sidecar beside it with the fixture's own name
and the suffix `.origin.md`. A sidecar rather than one index, because an index
listing every fixture drifts against the directory the moment somebody adds a
file and forgets, and the drift is invisible. A missing sidecar is not.

Every record says:

The origin. For a real extract, the database or the publication, the query or
the table it came from, and the date it was retrieved. For a generated file, the
tracked generator that produced it and the arguments it was given.

The terms. What the source said its terms were on the date they were read, with
the link or the command that read them, and what attribution an output computed
from it has to carry. `docs/sources.md` is where the same question is answered
per upstream rather than per file, and a fixture record points at the entry there
rather than restating it.

Real or synthetic, in one word, because the two support different claims and a
reader should not have to infer which one they are holding.

A synthetic fixture says two more things. The property it was built to have, in
one sentence, so a reviewer sees what the file is for instead of inferring it
from the bytes. And the generator, which is tracked in this repository and
committed, so the file can be produced again rather than being a set of bytes
somebody once made.

A real extract says one more thing. Whether it was edited after retrieval, and
if it was, exactly what was changed. An extract that was tidied on the way in is
no longer evidence about what people publish, which is the only reason to carry
one.

## Why the bytes are the point

An input reader is a thing that meets carriage returns, byte order marks and
trailing whitespace, and the fixtures that prove it handles them are the
fixtures most likely to be damaged on the way into the repository. This is not a
worry. It was measured here.

A two-line tab-separated file with CRLF endings, 33 bytes, was offered to git as
if it were a fixture:

    printf 'wavelength\tintensity\r\n500.0\t120\r\n' > probe.tsv
    git hash-object --path tests/fixtures/probe.tsv probe.tsv
    057d47b1d68953dca548b9c10797c2b34c2a4f27

and the blob git would have stored is 31 bytes, with both carriage returns gone:

    git cat-file -s 057d47b1d68953dca548b9c10797c2b34c2a4f27
    31
    git cat-file -p 057d47b1d68953dca548b9c10797c2b34c2a4f27 | xxd
    00000000: 7761 7665 6c65 6e67 7468 0969 6e74 656e  wavelength.inten
    00000010: 7369 7479 0a35 3030 2e30 0931 3230 0a    sity.500.0.120.

The cause is `core.autocrlf`, which was `true` on the machine this was measured
on, from the system, the global and the local configuration alike. That is the
ordinary state of a Windows clone rather than an unusual setting. The byte a
fixture exists to prove is deleted, the deletion is silent, the file still looks
right in an editor, and the test that was supposed to prove the reader survives a
carriage return passes because it never met one.

The worse half is that nothing in the repository decided any of this. There was
no tracked `.gitattributes`, so the answer came from each clone's own
configuration, and two contributors with different settings would disagree about
the bytes of the same fixture without either of them doing anything wrong.

## The repair, and the proof that it bites

`.gitattributes` at the root now carries one line:

    tests/fixtures/** -text

With that file present, the same offer keeps both carriage returns and the blob
is 33 bytes:

    git check-attr -a tests/fixtures/probe.tsv
    tests/fixtures/probe.tsv: text: unset
    git hash-object --path tests/fixtures/probe.tsv probe.tsv
    23b7e9fa24b4faefb3c428675877c21d2e329bc3
    git cat-file -s 23b7e9fa24b4faefb3c428675877c21d2e329bc3
    33
    git cat-file -p 23b7e9fa24b4faefb3c428675877c21d2e329bc3 | xxd
    00000000: 7761 7665 6c65 6e67 7468 0969 6e74 656e  wavelength.inten
    00000010: 7369 7479 0d0a 3530 302e 3009 3132 300d  sity..500.0.120.
    00000020: 0a                                       .

Two different blob hashes for one file, and the only difference between the two
runs is whether that line exists. Delete the line and the first hash comes back,
which is the whole proof and is the reason it is written out rather than
asserted.

The rule is held to `tests/fixtures/**` on purpose. A repository-wide `-text`
would change how every tracked file here is stored, for a reason that is only
about fixtures, and `git check-attr -a README.md` prints nothing under the line
above, which is the check that it did not reach further than it was meant to.

The line is not a substitute for looking. It stops git from rewriting a fixture,
and it cannot stop an editor from doing the same thing before git ever sees the
file. Where the exact bytes are the point of a fixture, the safer construction is
a fixture built in the test from an encoded literal, so that the bytes reaching
the reader are decided by the source and not by anything on the way in. That
choice belongs with the test rather than with this rule, and both routes need
the record above.

## What is not here yet

Issue #8 asks for a check that refuses a fixture file carrying no record, and for
a run in which a fixture added without one turns that check red. There is no
check, because there is nothing here to run one: this repository has no source
tree and no job that builds or tests anything. Issue #8 stays open with that
written into it.

Until the check exists the rule is held by whoever reads a change, which is worth
saying plainly rather than leaving a reader to assume a mechanism stands behind
a document. The half that does have a mechanism today is the byte preservation
above, and it is git's rather than this repository's code.
