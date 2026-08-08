# The repository layout, and the boundary between the matching engine and the spectroscopy

Decision record for issue #3. Underneath the spectroscopy this board is a data
association problem: a set of observations, a set of hypotheses about what
produced each one, a score, a rule saying which hypotheses cannot both be true,
a search, and a posterior over what survived. That half can be built and tested
without a wavelength in sight, and this record draws the line between the two
halves before there is any code to draw it through.

Nothing in this repository refuses a violation of this record today, because
there is no source tree here for anything to refuse:

    git ls-files 'crates' 'crates/*' '*.rs' 'Cargo.toml' ; echo "exit=$?"
    exit=0

That is also why issue #3 is not closed by this file. Its Done-when asks for a
tree that matches the document, and the tree is the half that does not exist.
The reason is written into #3 rather than left to be inferred from an open
issue.

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
when one is, which is the property a list in a file would not have.

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
up in other people's scripts, which is entry 6 of issue #1 and is the
maintainer's to answer, so this record names the unit and not the command.

Two directories are not crates. `tests/integration/` is the harness of issue #7,
which the default gate does not run. `docs/` is where every decision above was
argued before the crate that carries it existed.

## The boundary is a dependency edge, not a convention

No `assoc-` crate declares a dependency on any `spectro-` crate. That sentence
is worth more than a naming convention because the compiler refuses the
violation, and the refusal was measured rather than assumed.

Two crates were built in a scratch workspace, `assoc-model` with no declared
dependencies and `spectro-quantity` depending on it by path. Adding to
`assoc-model` a function returning a type from `spectro-quantity`, without
touching either manifest, is refused by `cargo build --offline`, twice, once for
the return type and once for the constructor:

    error[E0433]: cannot find module or crate `spectro_quantity` in this scope
      = help: if you wanted to use a crate named `spectro_quantity`, use
        `cargo add spectro_quantity` to add it to your `Cargo.toml`

The obvious way round it is to take the help text and declare the dependency,
and that is refused too, before any source file is read:

    error: cyclic package dependency: package `assoc-model` depends on itself.
    Cycle: package `assoc-model` ... which satisfies path dependency
    `assoc-model` of package `spectro-quantity` ... which satisfies path
    dependency `spectro-quantity` of package `assoc-model`

Absolute paths in both messages are elided and the elision is deliberate: a
tracked file quoting a working directory carries a user name into a public
repository, which is what issue #59 exists to refuse in the answer files. The
recipe above reconstructs both messages for a reader who wants them.

The toolchain was cargo 1.97.0 and rustc 1.97.0 on x86_64-pc-windows-msvc. Both
messages are diagnostics of one toolchain version and not a promise about every
future one. What they establish is that on this board the boundary has a
mechanism behind it as soon as the workspace exists, and that the mechanism
refuses both the direct violation and the first thing somebody reaches for to
work around it.

The cycle only bites while the dependency runs the other way. A `spectro-` crate
that depends on no `assoc-` crate at all leaves the reverse edge legal, so the
grep below is not redundant with the compiler and neither replaces the other.

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
larger tool than this board needs before it has a tree.

The list is the check's data rather than a paragraph in this file, for the same
reason `docs/quality-parity.md` prints its lists rather than remembering them.
The check itself is issue #53's, and the run showing a deliberately added
identifier make it red is owed there.

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
