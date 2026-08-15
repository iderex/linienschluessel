# The implementation language and toolchain

Decision record for issue #2. The core of this board is written in Rust, built
with cargo, against a toolchain version pinned in a tracked file. This record
was written before the first source file so that the first one would inherit a
decision rather than make one, and the source files are here now. Read at
a23385a:

    git ls-files 'crates/*/src/lib.rs' | wc -l
    13
    git grep -l '' -- '*.rs' | cut -d/ -f2 | sort -u
    spectro-contract

Thirteen crates, and one of them holding every line of Rust in the repository.
The channel and the dependency lock are tracked beside them:

    git ls-files 'rust-toolchain.toml' 'Cargo.lock'
    Cargo.lock
    rust-toolchain.toml

What that does not amount to is a build that refuses to proceed without them,
which is issue #4's and is not this record's to claim. Issue #3 is where the
layout that carries the boundary between the engine and the spectroscopy is
drawn, and what refuses a violation of that boundary, together with how narrow
the refusal is, is measured in `docs/decisions/layout.md` and is not restated
here.

## The five points, answered

### Refusing a confusion of conventions before the program runs

Air wavelength against vacuum wavelength, wavenumber against energy against
wavelength, observed against Ritz: none of these produces a crash. Each produces
a number of the right magnitude and the wrong meaning, and a test suite catches
such a thing one case at a time, in the cases somebody thought of.

Rust removes the class; it does not merely sample it, because the result type
of an operator is chosen by the author. The difference of two level energies
is a transition wavenumber and is not itself a level energy, which is the Ritz
combination principle written into the type system instead of into a comment.
The following program is the whole argument, and it is short enough to paste:

    #[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
    struct Kayser(f64);          // vacuum wavenumber, cm^-1
    #[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
    struct LevelEnergy(f64);     // level energy as cm^-1 above the reference
    #[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
    struct AirWavelengthNm(f64); // wavelength in standard air, nm

    impl std::ops::Sub for LevelEnergy {
        type Output = Kayser;
        fn sub(self, rhs: LevelEnergy) -> Kayser { Kayser(self.0 - rhs.0) }
    }

    fn main() {
        let upper = LevelEnergy(45_000.0);
        let lower = LevelEnergy(20_000.0);
        let sigma: Kayser = upper - lower;
        let lambda = AirWavelengthNm(400.0);
        let _wrong = sigma - lambda;                  // 1
        let _also_wrong: LevelEnergy = sigma;         // 2
        let _third = upper - lower - lower;           // 3
    }

Compiled with `rustc --edition 2021 units.rs` under rustc 1.97.0 on
x86_64-pc-windows-msvc, that is three errors and no binary: E0369 on line 1,
because no `Sub<AirWavelengthNm>` exists for `Kayser`; E0308 on line 2, because
a wavenumber is not a level energy; and E0369 again on line 3, because the
second subtraction is being applied to a wavenumber. Line 3 is the one worth
looking at twice. It is dimensionally meaningless, it is the sort of thing a
tired author writes while chasing an index, and no unit annotation in a comment
would have stopped it.

Go was compiled against the same three lines and stops after the first of them.
`go build` refuses `LevelEnergy - AirWavelengthNm` as mismatched types, which is
the crude half of the property and is real. It cannot refuse lines 2 and 3: a
binary operator in Go returns one of its operand types, so the difference of two
`LevelEnergy` values is a `LevelEnergy`, the assignment on line 2 is legal, and
the second subtraction on line 3 is legal too. The same file with line 1 removed
built and ran and printed `25000 5000`. So the distinction between the two
languages here is not that one has named types and the other does not. It is
that only one of them can say what an operation between two quantities produces,
and on this board almost every operation is a difference.

### An order of summation that is fixed, never inherited

The engine sums a score over thousands of features and compares sums, and
floating point addition is not associative, so the order is part of the answer.
Two hazards carry that order without anyone choosing it: the iteration order of
a container, and a reduction split across threads.

The first is measurable. This program, compiled with `rustc -O` and run three
times on one machine, printed the `HashMap` keys as `bhfegdca`, then `dhcefgab`,
then `acebhgdf`, while the `BTreeMap` keys printed `abcdefgh` every time:

    use std::collections::{BTreeMap, HashMap};
    fn main() {
        let keys = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let h: HashMap<&str, u32> =
            keys.iter().enumerate().map(|(i, k)| (*k, i as u32)).collect();
        let b: BTreeMap<&str, u32> =
            keys.iter().enumerate().map(|(i, k)| (*k, i as u32)).collect();
        println!("hash  {}", h.keys().cloned().collect::<Vec<_>>().join(""));
        println!("btree {}", b.keys().cloned().collect::<Vec<_>>().join(""));
    }

The three hash strings are what one occasion produced and are not the
reproducible part of that measurement. A reader who runs it will get three
different strings, and getting three different strings is the observation. The
`BTreeMap` line is the half that is expected to be identical everywhere.

That is the language making the hazard loud instead of quiet. An order that
differs between two runs of one binary is found on the first afternoon;
an order that is stable on the author's machine and different on somebody
else's is found by the person who cannot reproduce a published number. So the
rule the engine takes from this is that no ordered result is read out of a hash
container, and the container that would tempt somebody is the one that fails
first.

The second hazard is left switched off, and nothing manages it. Rust adds no
parallelism unless a dependency is chosen for it, so a reduction is sequential
until somebody makes it otherwise, and the day that changes it is a deliberate
change with a summation order attached to it.

Contraction is the third and smallest. A compiler that folds `a * b + c` into a
single fused instruction rounds once where the source rounds twice, and the two
differ. On this board that is a difference of the sixteenth significant figure
and it still ends a bit-identical comparison. Rust offers the fused form as an
explicit method, and the separate form measured here was not contracted into it:
compiled with `rustc -O`, `0.1 * 0.1 + -0.01` printed `1.734723475976807e-18`
while `0.1_f64.mul_add(0.1, -0.01)` printed `9.020562075079397e-19`. That is one
measurement on one target with one compiler version and it is not a guarantee
about every target; what it establishes is that the two forms are distinguishable
in this language and that choosing between them is the author's act. Whether
that survives a second platform is issue #39's measurement and not this record's
claim.

### A million candidate pairs, in one language

A few thousand lines against a few hundred levels is of the order of a million
level-pair differences before pruning, and the objective is evaluated over sets
of them. A means that is comfortable at that size only after the numerical inner
loop is rewritten in something else has not paid for itself: it has deferred the
cost to the point where the two halves disagree about a convention, which is the
first point again in a worse place.

Rust runs the inner loop at the same speed as the C++ alternative without a
second language in the tree, and it does so with the quantity types above
compiled away, since a struct wrapping one `f64` has the layout of one `f64`.
The property in the first section therefore costs nothing at the size this
section is about, which is the reason the two are not in tension.

The honest bound: no measurement of this board's real inner loop has been made,
because there is no inner loop yet. Issue #26 is where a time and a peak memory
for a stated line and level count get quoted with the command that produced
them, and where the scaling when the level count doubles is stated. Until that
lands, this paragraph is an argument from the shape of the language and not a
number.

### Something an operator can run in three years

The artefact is a single executable with no runtime to install and no dependency
tree to resolve at the far end. That is what makes a number reproducible after
the environment everybody has today has gone, and it is worth more here than in
most projects, because the output of this board is meant to be cited.

Rust links its standard library into the binary and needs no interpreter beside
it. Cross-compiling is more work than in Go and less than in C++, and the target
list is issue #63's to fix rather than this record's. Byte-identical rebuilds
are a separate and harder claim, which #4 is required to either demonstrate or
refuse in writing, and nothing here asserts it.

### Headless tests, property tests and fuzzing in the toolchain

Every test in the default suite has to run with no display, no elevated rights
and no network, which is issue #6. Rust's test runner is part of cargo, needs no
service, no display and no listening socket, and the input readers are the part
of this board that most wants property testing and coverage-guided fuzzing.
Both exist as cargo subcommands rather than as a parallel apparatus, so the
fuzzing corpus and the property tests live beside the unit tests and are run by
the same person who runs those. Issue #56 is where the seed corpus becomes part
of the gate.

## The candidates that lost, and what each lost on

Go. It has the simplest build and the best cross-compilation of the five, and
its test runner and its coverage-guided fuzzing are in the toolchain as well,
so it wins or draws on the last two points. It loses on the first, and it
loses on the point this board's failure mode lives in. The measurement is in
the first section: Go refuses a subtraction between two differently named
types and cannot refuse a subtraction that returns the wrong quantity, because
the result type of an operator is not the author's to choose. On a board where
the canonical defect is a plausible wrong number produced by a difference,
that is the deciding gap, and no preference about syntax comes into it.

Python. It is where the people who would use this work already are, and where
the reference optimisation libraries live, which is a real cost to give up and
is why entry 4 of issue #1 exists at all. It cannot refuse a unit confusion
before the program runs, and it ships a runtime, so it loses the first and the
fourth points outright rather than narrowly. A binding that puts this board
inside somebody's existing script is a different question from what the core is
written in, and this record does not answer it.

Julia. Numerically the strongest of the five, and the only one with a mature
units library, which is the first point solved by a package rather than by
hand. It loses on the fourth: a self-contained artefact is the weakest thing
about it, and on this board the artefact is what a citation points at. It also
adds a runtime and an ecosystem this tree carries nothing else from.

C++. It has the libraries, and templates can express the quantity algebra of the
first section. It loses on nearly every other line: no test runner, no fuzzing
and no dependency resolution in the toolchain, so all three arrive as a build
system somebody has to maintain, and the memory-safety surface is paid for by
every reader of every input file this board is designed to be pointed at.

## What this costs

Rust's numerical ecosystem is thinner than Julia's or Python's, so more of the
numerical code on this board gets written here and tested here rather than
imported. That is a cost in effort and a gain in the thing the three rules ask
for, since code in this tree can be refused by a check in this tree.

There is a compile step, and a contributor pays it before the suite runs.

And a spectroscopist who wants to read the scoring function may not read Rust.
That is the sharpest of the three costs, because the argument of this project is
that its reasoning should be inspectable. The answer is not that Rust is
readable. It is that the decisions this board makes are written in these records,
in prose, before the code exists, so that disagreeing with the engine does not
require reading it. If that stops being true, this cost has come due.

## What would overturn this

Any of the following, and each is a measurement rather than an opinion.

A Python-facing binding is required in the first release and is measured to be
cheaper as a Python core with a compiled kernel than as a Rust core with a
binding on it. That question is entry 4 of issue #1 and is the maintainer's.

The quantity types of the first section turn out to be unusable in practice, in
the specific sense that the engine's own code is found converting through raw
`f64` at more sites than it uses them, which would mean the property was bought
and not kept.

The performance argument in the third section fails when #26 measures it, and
the repair needs a second language in the inner loop. At that point the first
point has to be re-answered for two languages rather than one, and this record
is the wrong answer.

A dependency the objective genuinely needs exists in one ecosystem and nowhere
else, and reimplementing it here is measured to be larger than the whole engine.
That is a real possibility for the solver, and issue #36 is where the problem is
stated in a form that lets a solver be swapped, which is the smaller repair to
reach for first.
