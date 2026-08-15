# Pinning the toolchain, pinning the graph, and what a rebuild reproduces

Decision record for issue #4. A number this board produces is only as
reproducible as the binary that produced it, so the compiler version and the
dependency graph are both pinned in tracked files, and the build refuses to
proceed when a restore would move either of them.

This record does not close #4. What #4 asks for is those files in this
repository and a build job that reads them, and neither exists here:

    git ls-files 'Cargo.toml' 'Cargo.lock' 'rust-toolchain.toml' 'crates' 'crates/*' '*.rs' ; echo "exit=$?"
    exit=0

So every measurement below was made on a scratch crate outside this repository,
in the same way `docs/decisions/means.md` measured the language before there was
a file to measure. The recipes are written out so a reader can rebuild each one.
What remains owed to #4 is written at the end.

The toolchain throughout was cargo 1.97.0 and rustc 1.97.0 on
x86_64-pc-windows-msvc. Every result below is a result on that target with
that version. No other target was measured, and where that matters it is said
again so the reader does not have to infer it.

## The compiler version comes from a file

`rust-toolchain.toml` at the workspace root carries the channel, the profile and
the components:

    [toolchain]
    channel = "1.97.0"
    profile = "minimal"
    components = ["rustfmt", "clippy"]

The point of a file, where a job definition would have carried a string, is
that one place decides the version for the person running the suite and for
the machine running it, so the two cannot drift apart while both look correct.
rustup reads it without being asked. In a directory carrying the file above:

    rustup show active-toolchain
    1.97.0-x86_64-pc-windows-msvc (overridden by '<elided>/rust-toolchain.toml')

and in a directory that does not carry one, the same command on the same machine
answers `stable-x86_64-pc-windows-msvc (default)`. Absolute paths are elided
here and below. A tracked file quoting a working directory carries a user name
into a public repository, which is what issue #59 exists to refuse in the answer
files, and the recipe reconstructs the full line for a reader who wants it.

The version pinned is exact, never a channel name. `stable` is a moving
target by construction, and a board whose output is meant to be cited cannot
have the compiler move underneath a published number without anyone editing a
tracked file.

## The graph is locked, and a restore that would move it fails

`Cargo.lock` is tracked, and every build that is not a developer's local
experiment passes `--locked`. Two failures matter and both were measured.

A graph that would grow. With the lock in sync, `cargo build --release --locked`
finishes and exits 0. Adding one dependency to the manifest and running the same
command again:

    error: cannot update the lock file <elided>/Cargo.lock because --locked was passed to prevent this
    help: to generate the lock file without accessing the network, remove the --locked flag and use --offline instead.

    exit=101

A graph that has been altered underneath. Changing a single character of one
`checksum` line in the lock and building again:

    error: checksum for `itoa v1.0.18` changed between lock files

    this could be indicative of a few possible errors:

        * the lock file is corrupt
        * a replacement source in use (e.g., a mirror) returned a different checksum
        * the source itself may be corrupt in one way or another

    unable to verify that `itoa v1.0.18` is the same as when the lockfile was generated

    exit=101

Restoring the lock returns the build to exit 0, so the refusal is the lock's
content and not a poisoned cache.

Both messages diagnose one cargo version and promise nothing about every
future one, and the second names the package that was tampered with, which is
the field a reader needs.

The first message does not name the repair. It names a different flag, and a
contributor who follows its help line drops the guard instead of updating the
lock. The repair is to build once without `--locked`, read the resulting change
to `Cargo.lock`, and commit it. Issue #4 asks that the failure carry that
sentence, so the job that runs the build is what has to add it; cargo will not.
That is a job this repository does not have yet, and it is part of what remains
owed below.

## Whether two builds of one source are byte-identical

They are not, by twenty bytes out of 126976, and the twenty are locatable.

One scratch crate was built in release three times from a clean `target`,
without touching a source byte between builds, and the resulting executables
compared:

    cargo clean && cargo build --release
    cmp -l build-a.exe build-b.exe | wc -l
    20

The differing offsets fall in two groups, and a hexdump identifies both.

Sixteen of the twenty are one contiguous run inside the CodeView record of the
debug directory, immediately after the `RSDS` signature. That is the PDB
signature GUID, generated fresh by the linker on every link, and nothing in the
source decides it.

The other four are single bytes at four separate offsets, and all four are the
low byte of a `TimeDateStamp` field. One is in the COFF header, at the fixed
offset four bytes past the machine type and section count; the other three are
the `TimeDateStamp` of the three `IMAGE_DEBUG_DIRECTORY` entries the linker
emits, of types 2, 12 and 13. Only the low byte of each differed because the
three builds were close together; a wider separation moves more of each field
and changes no conclusion. The timestamp values themselves are elided for the
same reason the paths are.

So the two fields that differ are the link timestamp and the PDB signature, and
the remaining 126956 bytes are identical. Nothing else in this configuration is
non-deterministic.

### The absolute path is not among the differences

Issue #4 supposes that embedded paths commonly differ, and on this target with
this configuration they do not. The whole crate directory was copied to a second
absolute path, cleaned and rebuilt, and compared against the build from the
first path:

    cmp -l build-b.exe build-c.exe | wc -l
    20

The same twenty offsets, and no others. The build directory is not in the
binary here. This is a result for a release profile with `debug = false` on one
target, and it is not a general statement about Rust: a profile carrying debug
information embeds source paths, and a target whose linker behaves differently
was not measured.

That last sentence turned out to be the operative one. The result above is a
result about a linked executable, and the artefacts this workspace produces are
not linked executables. Measured on this repository, the absolute path is in
them, in readable form. The section headed `The path is in this workspace's
artefacts` below has the numbers and supersedes this heading for anything built
here.

### One linker flag removes both differences

The MSVC linker replaces the timestamp with a hash of the content, and derives
the PDB signature the same way, when it is given `/Brepro`. Passing it through
cargo and rebuilding twice from clean, in two different directories:

    RUSTFLAGS="-Clink-arg=-Brepro" cargo build --release
    sha256sum repro-1.exe repro-2.exe
    d5cebf14df72fefe078b48d66111c04686258333c02bd81fd9d3df015f370d0d *repro-1.exe
    d5cebf14df72fefe078b48d66111c04686258333c02bd81fd9d3df015f370d0d *repro-2.exe

    cmp -l <first path>/rb-probe.exe repro-2.exe | wc -l
    0

Identical across rebuilds and across paths, and the binary still runs and prints
what it printed before. So on this target byte-identity is reachable, it costs
one flag, and the decision is to take it.

The sentence that followed said the release profile carries the flag when the
workspace lands. The workspace has landed and the profile does not carry it,
and that is deliberate and not an oversight. `/Brepro` is a flag to the MSVC
linker, and the workspace links nothing: thirteen library crates produce
thirteen `.rlib` archives and no executable, so there is no link step for the
flag to reach and no timestamp or PDB signature in the output for it to
replace.

    cargo build --release --locked --offline
    ls target/release/*.rlib | wc -l
    13
    ls target/release/*.exe
    ls: cannot access 'target/release/*.exe': No such file or directory

Adding the flag now would put a line in the manifest whose effect nobody could
measure, and an unmeasurable setting is the one shape refused here. The binary
crate that would give it something to act on is entry 6 of #1, and the
decision stands waiting for it.

What that flag costs was not measured here. Debuggers and symbol servers key a
binary to its PDB through exactly the two fields it replaces, so a workflow that
looks a build up by timestamp is the place to expect trouble, and this record
makes no claim about how much.

### What is not claimed

Byte-identity was reached on one target, with one toolchain version, on one
machine, for a crate with one dependency and no build script. A second
platform is issue #39's measurement to take, and no claim of it is made here.
A build script, a proc macro or a code generator is a source of variation this
crate did not have. Nothing here says a rebuild in a year reproduces these
bytes; it says which fields moved when nothing else did, and which flag
stopped them.

## The same measurements, made here

The workspace landed in #78, so everything above that was measured on a scratch
crate can be measured on a commit of this repository instead, and this section
is that repeat. It is the section a reader who has only this repository can run.

The two pin files are tracked from this change onwards:

    git ls-files rust-toolchain.toml Cargo.lock
    Cargo.lock
    rust-toolchain.toml

Every command below was run on x86_64-pc-windows-msvc with the toolchain
`rust-toolchain.toml` names, and rustup takes the version from that file without
being asked:

    rustup show active-toolchain
    1.97.0-x86_64-pc-windows-msvc (overridden by '<elided>/rust-toolchain.toml')
    cargo --version
    cargo 1.97.0 (c980f4866 2026-06-30)

Nothing below depends on this record's own text, so a later commit that edits
only prose reproduces it unchanged. What the results do depend on is the crate
sources, the manifests, the lock and the pin file, and a change to any of
those is a reason to run the commands again; citing the outputs below is not
enough.

### Two builds of one commit are byte-identical

Built twice from a clean `target`, with no source byte touched between the
two, and the thirteen archives compared by content, never by name:

    cargo clean && cargo build --release --locked --offline   # first build
    cargo clean && cargo build --release --locked --offline   # second build
    diff <(sha256sum first/*.rlib  | sed 's# .*/# #') \
         <(sha256sum second/*.rlib | sed 's# .*/# #') ; echo "exit=$?"
    exit=0

The hashes themselves are not pasted, because a list of them in a document is
wrong the first time anybody edits a crate, while the comparison stays right.
Thirteen files were compared and thirteen matched:

    ls target/release/*.rlib | wc -l
    13

The suite passes offline with the lock enforced, which is the other half of a
clean clone building this commit:

    cargo test --locked --offline
    cargo test --locked --offline 2>&1 | grep -E '^test result' \
      | awk -F'[ ;]' '{p+=$4} END {print p}'
    57
    cargo test --locked --offline 2>&1 | grep -c '^test result: FAILED'
    0

### The lock refuses a graph that would move, measured here

One dependency added to a member manifest, the lock left alone:

    printf '\n[dependencies]\nassoc-model = { path = "../assoc-model" }\n' \
      >> crates/assoc-synth/Cargo.toml
    cargo build --release --locked --offline
    error: cannot update the lock file <elided>/Cargo.lock because --locked was passed to prevent this
    help: to generate the lock file without accessing the network, remove the --locked flag and use --offline instead.
    exit=101

The lock is byte-for-byte what it was before that run, so the flag refuses; it
does not repair quietly and then complain:

    sha256sum Cargo.lock        # before and after the refusing run
    4eaad7b98d193fbc2a61d7c1c7ed8eaa4aafbd70dbd314d955008c5e2ab20e1a *Cargo.lock

Cargo's `help:` line still answers a different question, and a contributor who
follows it drops the guard instead of updating the lock. That is the same
finding as above, now made against this repository rather than a scratch crate,
and the repair sentence is still the job's to print.

The second refusal above, an altered `checksum` line, cannot be measured here at
all. Every package in this lock is a member of this workspace and none of them
is fetched, so the lock carries no checksum to alter:

    grep -c '^\[\[package\]\]' Cargo.lock
    13
    grep -c '^checksum' Cargo.lock
    0

That refusal therefore stays a scratch-crate result until this workspace takes
its first external dependency, and it should be re-measured on the change that
adds one. Nothing here carries over on its own.

### The path is in this workspace's artefacts

The scratch executable did not carry its build directory. These archives do, and
in readable form. The tree was copied to a second absolute path, built there
with the same commands, and the two sets compared:

    diff <(sha256sum first/*.rlib       | sed 's# .*/# #') \
         <(sha256sum second-path/*.rlib | sed 's# .*/# #') ; echo "exit=$?"
    exit=1

All thirteen differ. They differ in two different ways, and the split matters
because only one of the two is legible to anyone who opens the file.

The twelve crates whose `src/lib.rs` is empty differ inside one run of at most
sixteen byte positions and nowhere else, and each pair of files is the same
size, so no path is stored in them:

    cmp -l first/libassoc_model.rlib second-path/libassoc_model.rlib \
      | awk 'NR==1{f=$1} {l=$1} END{print "first="f" last="l" span="(l-f+1)" count="NR}'
    first=1589 last=1604 span=16 count=16

Two of the twelve report fifteen rather than sixteen, which is two hashes
agreeing on one byte and not a different shape. Sixteen bytes that move with the
build directory are a fingerprint of the path rather than the path. What is in
`spectro-contract`, the one crate with source in it, is the path itself, twice
per source file:

    ls crates/spectro-contract/src | wc -l
    9
    strings -a first/libspectro_contract.rlib | grep -c '<the first root>'
    18
    strings -a second-path/libspectro_contract.rlib | grep -c '<the second root>'
    18

Nine of the eighteen are the source file's own absolute path and nine are the
bare working directory the compiler ran in. The strings are absolute and begin
at the drive letter, so each one carries the account name of whoever ran the
build. Nothing here reaches an answer file and
this is not the leak #59 is about, but it is the same class, and it is worth
knowing before anybody ships a compiled artefact from a developer machine.

`--remap-path-prefix` removes the readable half and does not close the gap.
Building at each root with that root remapped to one name:

    RUSTFLAGS="--remap-path-prefix=<root>=/src" cargo build --release --locked --offline
    strings -a <either>/libspectro_contract.rlib | grep -c '<either root>'
    0

Twelve of the thirteen archives then hash the same at both paths. The
thirteenth does not, and the residue is in its metadata section, not in any
string:

    diff <(sha256sum remapped-first/*.rlib | sed 's# .*/# #') \
         <(sha256sum remapped-second/*.rlib | sed 's# .*/# #') | grep -c '^[<>]'
    2
    diff <(strings -a remapped-first/libspectro_contract.rlib | sort -u) \
         <(strings -a remapped-second/libspectro_contract.rlib | sort -u) \
      | grep 'lib.rmeta/'
    < lib.rmeta/      0           0     0     644     261889    `
    > lib.rmeta/      0           0     0     644     266725    `

Four thousand eight hundred and thirty-six bytes of `lib.rmeta` that still move
with the path, and what occupies them was not identified. So the honest state is
that one flag removes every path a reader can see and does not make two builds
at two paths equal, and this record makes no claim about what the remainder is.

None of this touches the result at the top of this section. Two builds at one
path are identical; the path is what has to be held fixed, and holding it fixed
is a property of a build environment rather than of this repository.

### What is not claimed here either

One machine, one target, one toolchain version, one profile. No second platform,
which is #39's. No claim that a rebuild in a year reproduces these archives. The
`.rlib` is cargo's intermediate rather than a thing anybody ships, and a release
artefact for an operator is #63's, so byte-identity for the thing that eventually
gets published has not been measured because that thing does not exist.

## What remains owed to #4

One thing, and it is another issue's file.

The build job that reads the version from `rust-toolchain.toml` rather than from
a string of its own, passes `--locked`, and prints the repair sentence beside
cargo's help line, is issue #5's job to create. This repository's workflows are
the ones `git ls-files .github/workflows` prints, and none of them builds or
tests anything. Until that job exists, the pin file and the lock are read by
whoever runs the commands above and by nothing else, and #4 says so rather than
counting a tracked file as a mechanism.

The proof that job owes is the one that catches the mistake worth catching: the
job made to fail when the version it used is not the version the file names.
A job installing a toolchain by naming a version string of its own satisfies
nothing here, even on the day the two strings agree.
