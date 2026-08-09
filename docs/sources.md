# The terms of every source this board can read

The record for issue #60. One entry per upstream this board is able to read,
saying what the source is, where it was retrieved from, what its terms said on
the date they were read, what attribution an output computed from it has to
carry, and whether this repository carries any of that data itself.

The date is a field rather than a courtesy. Terms change, and an undated summary
of somebody else's terms is a claim about a page as it happened to read once. An
entry here says when it was read and gives the command that read it, so a reader
can find out whether it still says that.

This file is also the source the software reads for the attribution half. An
answer computed from a source that requires citation names that source in the
answer file, so the requirement travels with the result rather than living in a
document the operator did not open. That half is not built yet and the last
section says so.

An entry exists before an adapter for that source lands, not after. A reader
that arrives without one is a reader nobody checked the terms for.

## NIST Atomic Spectra Database

What it is. NIST Standard Reference Database 78, version 5.12, last update to
data content November 2024. It holds energy levels and observed and Ritz lines
for atoms and ions, published separately, which is the pairing this board's
whole problem is stated over. It is the source the levels adapter of issue #22
and the line adapter of issue #23 are written against.

Where it is read from. The query forms at
`https://physics.nist.gov/PhysRefData/ASD/levels_form.html` and
`https://physics.nist.gov/PhysRefData/ASD/lines_form.html`, which emit
tab-delimited and comma-separated output. Both were retrieved with
`curl -sS -L` on 2026-08-08 and answered 200. The two exports are one source
with one set of terms and one entry, and the two adapters do not change that.

The version and the citation were read on 2026-08-08 from

    curl -sS -L https://physics.nist.gov/PhysRefData/ASD/Html/verhist.shtml

which carries, under "Example of how to reference this online database":

    Kramida, A., Ralchenko, Yu., Reader, J. and NIST ASD Team (2024). NIST
    Atomic Spectra Database (version 5.12), [Online]. Available:
    https://physics.nist.gov/asd [11/8/2024]. National Institute of Standards
    and Technology, Gaithersburg, MD. DOI: https://doi.org/10.18434/T4W30F

and the database designation was read the same day from

    curl -sS -L https://physics.nist.gov/asd

which redirects to `https://www.nist.gov/pml/atomic-spectra-database` and carries
the heading `NIST Standard Reference Database 78`.

### The terms, and the belief they contradict

The widely held belief is that data from a United States federal agency is free
of copyright. For this source that belief is wrong, and the difference is the
whole reason this entry exists.

NIST publishes two different statements and the one that applies depends on
whether a product is Standard Reference Data. Read on 2026-08-08 from

    curl -sS -L https://www.nist.gov/open/copyright-fair-use-and-licensing-statements-srd-data-software-and-technical-series-publications

the page says that the Standard Reference Data Act, 15 U.S.C. § 290e, empowers
the Secretary of Commerce to secure copyright on behalf of the United States in
Standard Reference Data prepared by NIST, and that such products should include:

    Copyright protection on this compilation of data has been secured by the
    Secretary of the U.S. Department of Commerce on behalf of the United States
    in the United States and all countries that are parties to the Universal
    Copyright Convention, pursuant to Section 290(e) of Title 15 of the United
    States Code.

    NIST Standard Reference Data (SRD);
    ©Copyright [©YEAR] by the U.S. Secretary of Commerce on behalf of the United
    States of America. All rights reserved.

The statement people usually quote is on the same page under a different
heading, and it is the one for works that are not covered by that Act:

    Data/works created by NIST employees that are not covered by the Standard
    Reference Data Act are subject to 17 U.S.C. §105 and generally are not
    subject to copyright protection within the United States.

The Atomic Spectra Database is Standard Reference Database 78, so the first
statement is the one that reaches it and the second is not. That is a reading of
two public pages and not legal advice, and it is stated here so that a decision
is made against what the pages say rather than against a habit.

The same page says the data is provided as is and that NIST makes no warranty of
any kind, express, implied or statutory, including the implied warranties of
merchantability, fitness for a particular purpose, non-infringement and data
accuracy. That sentence belongs in front of anyone treating a published
uncertainty from this source as exact.

### What an answer has to carry

Every answer file computed from this source names the source, the version it was
retrieved at, the date of retrieval, and the citation above. The citation is
what the source asks for, and it is not optional.

Where the answer file is redistributed, the copyright statement above travels
with the part of the answer derived from this source. An answer file that mixes
this board's assignments with upstream values is the place that question
actually bites, and how far it reaches is a question about the answer format
rather than about this file.

### Whether this repository carries any of it

None of it:

    git ls-files 'tests/fixtures' 'tests/fixtures/*' ; echo "exit=$?"
    exit=0

That path was the only place a data file could have been when the command was
first written down. There is a source tree now, so the same question is asked of
it: every tracked file under `crates/` is Rust or a manifest, and none of them
is a table.

    git ls-files -- crates | grep -vE '\.rs$|\.toml$' ; echo "grep exit=$?"
    grep exit=1

Read at a23385a.

Whether it may is entry 2 of issue #1 and is the maintainer's to answer. This
entry is what that answer needs in front of it, because the two statements above
are not the same question, and an extract carried under a belief about the wrong
one is a licence problem in a public repository rather than an untidiness.

## What is not here yet

Issue #60 asks for two more things and neither exists.

An answer file naming the sources it used and the attribution each requires,
derived from this file rather than restated in code. There is no answer file and
nothing that would write one. `docs/decisions/layout.md` puts the answer schema,
the provenance allowlist and the writer in `spectro-answer`, and that crate is
empty:

    git grep -c '' -- crates/spectro-answer/src/lib.rs ; echo "exit=$?"
    exit=1

Read at a23385a. There is code in this repository now, in one crate of thirteen,
and it reads inputs rather than emitting answers.

A test asserting that a run reading a source with an attribution requirement
produces an answer file carrying it. Same reason.

So today this file is read by a person, and the requirement it carries reaches
an output only if somebody puts it there. Issue #60 stays open with that written
into it.

One absence is deliberate rather than an omission. What this board takes from
the boards it is meant to consume is a contract, written down in
`docs/decisions/input-contract.md`, and a contract has no terms to read. The day
one of them ships data this board actually reads, it gets an entry here like any
other source, and the entry comes first.
