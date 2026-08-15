# The repository layout, and the boundary between the matching engine and the spectroscopy

Decision record for issue #3. Underneath the spectroscopy this board is a data
association problem: a set of observations, a set of hypotheses about what
produced each one, a score, a rule saying which hypotheses cannot both be true,
a search, and a posterior over what survived. That half can be built and tested
without a wavelength in sight, and this record draws the line between the two
halves before there is any code to draw it through.

The tree this record describes exists. Read at e8e27be:

    git ls-files 'crates' 'crates/*' '*.rs' 'Cargo.toml' | wc -l
    51

    grep -c '^    "crates/' Cargo.toml
    14

Both halves of this record are refused by something now, not only stated. The
section on the dependency edge below carries that measurement and how narrow
it is, and the section carrying the word list names the check that reads it.

Every unit this record names is in the workspace. The binary above both sides
was the last one in, once entry 6 of issue #1 answered what the command is
called, and it is the fourteenth member:

    git ls-files '*/src/main.rs' 'crates/*/src/bin' ; echo "exit=$?"
    crates/linienschluessel/src/main.rs
    exit=0

The command this record used to put beside that one no longer says anything
about it, and it is kept here so that nobody reads its empty result as the
absence it once was. Cargo takes a binary target from `src/main.rs` without a
section declaring one:

    git grep -c '\[\[bin\]\]' -- 'crates' ; echo "grep exit=$?"
    grep exit=1

Issue #3 asks for a tree that matches this document; what remains of that half
is written into #3 and is not left to be inferred.

## Why the line is drawn now and not later

Drawing it later means drawing it through code that has already grown across
it, which is the ordinary argument and the weaker one.

The stronger argument is an experiment that the boundary either makes possible
or forecloses. `docs/validation-metrics.md` reports a calibration figure, and
the question behind it is whether a stated probability of 0.8 is right about
eighty per cent of the time. Answered against solved spectra alone, that figure
rests on a handful of instances, each expensive to obtain and none of them
carrying a truth that is known rather than published. Answered against synthetic
association instances whose truth is known by construction, it rests on
thousands of instances generated in milliseconds, with no spectrum involved at
all. The second experiment exists only if the probability machinery can be run
without the spectroscopy attached to it. If the two are entangled, the
calibration claim this board rests on is measured on the smaller sample for the
life of the project.

So the boundary is not tidiness. It is the difference between a calibration
number with a denominator in the thousands and one with a denominator in the
single figures.

## The boundary test

Could a matching problem from another field use the generic side unchanged? If
a type on the generic side mentions a wavelength, a parity or an intensity, the
line is in the wrong place.

That test decides most cases by reading a type name. Three cases it does not
decide by itself are worked through below, because they are the ones where the
line would otherwise be drawn by whoever wrote the code first.

## The units

The tree is a cargo workspace. Every unit is a crate under `crates/`, and the
side a crate is on is derivable from its name rather than looked up in a list:
`assoc-` is the generic side and `spectro-` is the spectroscopy side. A new
crate cannot be added without choosing a side, and no document has to be updated
when one is, which is the property a list in a file would not have. One crate
carries neither prefix, and it is the binary described below, which is on
neither side. A second crate carrying neither is a side nobody chose.

The generic side.

| Unit | What it holds | What it may not hold |
| --- | --- | --- |
| `assoc-model` | Observations, hypotheses as multisets of items over named slots, the structural key, mutual exclusion, and the scoring interface as a trait it never implements | Any score term, and any quantity with a physical unit |
| `assoc-determinism` | The ordered containers, the fixed summation order, the score comparison rule and the seed derivation of `docs/decisions/repeatable-runs.md` | Anything that knows what is being summed |
| `assoc-solve` | The exact solver of issue #37 and the search of issue #38, both against `assoc-model` alone | The objective |
| `assoc-posterior` | The posterior construction of `docs/decisions/probability-model.md`, the marginal, the competitor selection of `docs/decisions/competitors.md`, and the calibration metrics | The priors' physical meaning |
| `assoc-synth` | Generated association instances whose truth is known by construction, for the experiment above | Anything drawn from a real spectrum |

The spectroscopy side.

| Unit | What it holds |
| --- | --- |
| `spectro-quantity` | The quantity types of `docs/decisions/means.md` and the conversions of `docs/decisions/line-position.md`, which is the only place a medium is converted |
| `spectro-contract` | The readers and the validator for `docs/decisions/input-contract.md`, and nothing else reads an input file |
| `spectro-adapters` | One module per upstream export, each emitting a conforming input and nothing else |
| `spectro-candidates` | Level pair enumeration and the hard and weighted rules of `docs/decisions/selection-rules.md` |
| `spectro-objective` | Every score term, each a named function, which is issue #31's module |
| `spectro-null` | The accidental match null of `docs/decisions/chance-coincidence.md` and its diagnostics |
| `spectro-answer` | The answer schema, the provenance allowlist and the writer |
| `spectro-report` | The human-readable report, generated from the answer file alone |

One binary crate sits above both and wires them together. Its name is what ends
up in other people's scripts, which is entry 6 of issue #1, and the answer there
is `linienschluessel`: the same word as the repository and as the citation, so a
methods section quoting one of the three is unambiguous about the other two. It
holds nothing else yet. What the command takes, what it writes and the exit
codes that separate a refused input from an internal failure are issue #62's.

Two directories are not crates. `tests/integration/` is the harness of issue #7,
which the default gate does not run. `docs/` is where every decision above was
argued before the crate that carries it existed.

## The boundary is a dependency edge, not a convention

No `assoc-` crate declares a dependency on any `spectro-` crate. That sentence
is worth more than a naming convention because the toolchain refuses the
violation, and the refusal is measured in this workspace rather than in a
scratch one somewhere else.

Adding to `crates/assoc-model/src/lib.rs` a function returning a type from
`spectro-candidates`, without touching either manifest, is refused by
`cargo build --offline -p assoc-model`, twice, once for the return type and once
for the constructor:

    error[E0433]: cannot find module or crate `spectro_candidates` in this scope
      = help: if you wanted to use a crate named `spectro_candidates`, use
        `cargo add spectro_candidates` to add it to your `Cargo.toml`

The obvious way round it is to take the help text and declare the dependency,
and that is refused too, before any source file is read:

    error: cyclic package dependency: package `assoc-model` depends on itself.
    Cycle: package `assoc-model` ... which satisfies path dependency
    `assoc-model` of package `spectro-candidates` ... which satisfies path
    dependency `spectro-candidates` of package `assoc-model`

Absolute paths in both messages are elided and the elision is deliberate: a
tracked file quoting a working directory carries a user name into a public
repository, which is what issue #59 exists to refuse in the answer files. The
recipe above reconstructs both messages for a reader who wants them.

The toolchain was cargo 1.97.0 and rustc 1.97.0 on x86_64-pc-windows-msvc,
which is the channel `rust-toolchain.toml` pins. Both messages are diagnostics
of one toolchain version and not a promise about every future one.

The second refusal is narrower than the first and the width of it is now
countable instead of argued. Cargo refuses the cycle only because
`spectro-candidates` already depends on `assoc-model`. Three crates declare such
an edge and every other crate declares none:

    git grep -n 'assoc-model = { path' -- 'crates/*/Cargo.toml'
    crates/assoc-solve/Cargo.toml:9:assoc-model = { path = "../assoc-model" }
    crates/spectro-candidates/Cargo.toml:10:assoc-model = { path = "../assoc-model" }
    crates/spectro-objective/Cargo.toml:10:assoc-model = { path = "../assoc-model" }

Five `assoc-` crates and eight `spectro-` crates make forty ordered pairs in
the forbidden direction. Two of them close a cycle, `assoc-model` on
`spectro-candidates` and `assoc-model` on `spectro-objective`. Those two are
what the grep above implies; the other thirty-eight are derived from the same
grep and were not each run, and what they are derived to is that cargo accepts
them.

One of the thirty-eight was run and not derived, and it is worth pasting
because it is the failure this section would otherwise be read as excluding.
`spectro-quantity` has no edge back. Naming it from inside `assoc-model` and
declaring the dependency the help text suggested compiles:

    pub use spectro_quantity as _;

    cargo build --offline -p assoc-model
    warning: unused import: `spectro_quantity as _`
        Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.47s
    exit=0

A warning, and the build succeeds. So rustc covers the violation written in
code only while no manifest declares the edge, and cargo covers the manifest
only where the reverse edge is already there. Thirty-eight of the forty pairs
have neither in front of them today, which is why the grep below is not
redundant with the toolchain and neither replaces the other.

## The greppable half, and what it cannot catch

Issue #3 asks that no identifier on the generic side name a quantity specific to
spectroscopy, and that it be greppable. The check runs over `crates/assoc-*`
against a word list in a tracked file, and it reads identifiers rather than
prose, so a comment explaining why a term is absent does not trip it.

The list starts with the words whose appearance is unambiguous: `wavelength`,
`wavenumber`, `kayser`, `parity`, `multipole`, `ritz`, `vacuum`, `angstrom`,
`spectrum`, `spectra`, `ionisation`, `ionization`, `transition`.

Three words that belong to the failure this check exists to catch are
deliberately not on it. `level`, `line` and `intensity` all have ordinary
meanings a generic crate will legitimately use, and a check that refused a log
level, a line of input or a colour intensity would be worked around within a
week, which is worse than a check that does not fire. That is a real hole and
the boundary test in prose is what covers it, which is to say a reader covers
it. The condition that would close it is a check reading the parsed identifiers
of the crate rather than its bytes, at which point `level` as a field name on an
association type is distinguishable from `level` in `log_level`, and that is a
larger tool than this board needs today.

The list is the check's data and not a paragraph in this file, for the same
reason `docs/quality-parity.md` prints its lists instead of remembering them.
The check is `no-spectroscopic-identifier-on-the-generic-side` in
`.github/workflows/invariants.yml`, which carries that list as a row naming
this record, so a contributor who trips it is sent to the argument rather than
to a pattern.

It proves the pattern still catches the mistake on every run rather than once.
The job refuses a rule whose own declared demonstration its pattern does not
match, and this rule's demonstration is a struct field called `wavenumber` on a
generic observation type. What it searched at e8e27be is printed by the job
rather than claimed here:

    no-spectroscopic-identifier-on-the-generic-side: clean over 7 Rust file(s),
    1031 line(s) of Rust, out of 12 tracked file(s) and 1060 line(s) under
    [crates/assoc-*]. Record: docs/decisions/layout.md

That is a clean result over the thirteen words on the list. It says nothing
about the three the paragraph above keeps off it, which is the hole that
paragraph is about.

## Three cases the test does not decide by itself

The structural key. `docs/decisions/competitors.md` defines it as the multiset of
upper levels together with the multiset of lower levels, which is spectroscopy
vocabulary, and competitor selection is generic work. The line goes between the
shape and the names: `assoc-model` knows that an item occupies a fixed set of
named slots and that the structural key is one multiset per slot, and it does
not know that there are two of them or that they are called upper and lower.
`spectro-candidates` declares the two slots. So upper-different and
lower-different are one generic operation, slot-different, applied to two slots
a spectroscopy crate named.

The Ritz cycle closure of issue #35. Three assignments that must sum to zero is
a consistency constraint over the latent values of the items, which is generic,
and the fact that the latent value is a level energy and the constraint follows
from the combination principle is not. `assoc-model` carries constraints that
span more than one observation and evaluates them through the scoring interface.
Which cycles exist, and the bound on which of them are examined, are
`spectro-objective`'s.

The branching constraint of `docs/decisions/intensities.md`. This one is not
split, and the reason is that its generic statement would have exactly one
instance. Two observations sharing a latent quantity whose ratio is fixed by a
known constant is a real generic shape, and writing it generically here would
mean writing a mechanism whose only user is the transition rate table, in the
crate that is supposed to be reusable, to hold a rule that is entirely about
atomic structure. It stays in `spectro-objective` and reaches the solver as an
additive score contribution like every other term. The condition that would move
it is a second instance.

## Where the generic half eventually lives

Whether `assoc-*` becomes a separate repository, and whether the sibling boards
share it, is entry 7 of issue #1 and is not decided here. This record requires
only that it is separable, so that whichever answer comes back does not force a
rewrite. The dependency edge above is what makes that true: a set of crates
that depend on nothing in this tree can be lifted out of it, and a set that has
grown a reference to a wavelength cannot.

## What this costs

More crates than a single-crate layout, and a contributor pays that in build
graph and in manifest files before writing anything. It is paid once and the
alternative is paid every time somebody wants to run the generic side alone.

A generic type is harder to read than a specific one. `slot` is less obvious
than `upper level` to a spectroscopist reading `assoc-model`, and this board's
own argument is that its reasoning should be inspectable. The answer is the same
one `docs/decisions/means.md` gives: the decisions are argued in these records
in prose, before the code, so disagreeing with the engine does not require
reading it. Where a generic name is genuinely opaque, the spectroscopy crate
that instantiates it is where the reader is sent, and that crate uses the
spectroscopist's words.

And a boundary drawn before the code exists can be drawn in the wrong place.
The three cases above are the ones already visible; the fourth will arrive with
the first term nobody anticipated. What this record fixes is the test to apply
to it, not the answer.
