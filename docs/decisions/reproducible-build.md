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
x86_64-pc-windows-msvc. Every result below is a result on that target with that
version. No other target was measured, and where that matters it is said again
rather than left to the reader.

## The compiler version comes from a file

`rust-toolchain.toml` at the workspace root carries the channel, the profile and
the components:

    [toolchain]
    channel = "1.97.0"
    profile = "minimal"
    components = ["rustfmt", "clippy"]

The point of the file rather than a string in a job definition is that one place
decides the version for the person running the suite and for the machine running
it, so the two cannot drift apart while both look correct. rustup reads it
without being asked. In a directory carrying the file above:

    rustup show active-toolchain
    1.97.0-x86_64-pc-windows-msvc (overridden by '<elided>/rust-toolchain.toml')

and in a directory that does not carry one, the same command on the same machine
answers `stable-x86_64-pc-windows-msvc (default)`. Absolute paths are elided
here and below. A tracked file quoting a working directory carries a user name
into a public repository, which is what issue #59 exists to refuse in the answer
files, and the recipe reconstructs the full line for a reader who wants it.

The version pinned is exact rather than a channel name. `stable` is a moving
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

Both messages are diagnostics of one cargo version rather than a promise about
every future one, and the second names the package that was tampered with, which
is the field a reader needs.

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
one flag, and the decision is to take it: the release profile carries the flag
when the workspace lands, and the comparison above is what proves it still
holds rather than a sentence saying it does.

What that flag costs was not measured here. Debuggers and symbol servers key a
binary to its PDB through exactly the two fields it replaces, so a workflow that
looks a build up by timestamp is the place to expect trouble, and this record
makes no claim about how much.

### What is not claimed

Byte-identity was reached on one target, with one toolchain version, on one
machine, for a crate with one dependency and no build script. A second platform
is issue #39's measurement rather than this record's claim, and a build script,
a proc macro or a code generator is a source of variation this crate did not
have. Nothing here says a rebuild in a year reproduces these bytes; it says
which fields moved when nothing else did, and which flag stopped them.

## What remains owed to #4

The pins have to be files in this repository, and the workspace they would sit
in is issue #3's. Until that lands there is nothing here to pin.

The build job that reads the version from `rust-toolchain.toml` rather than from
a string of its own, passes `--locked`, and wraps cargo's help line with the
repair sentence, is issue #5's job to create and does not exist. This
repository's workflows today are the ones `git ls-files .github/workflows`
prints, and none of them builds anything.

The double-build comparison in this record was made on a scratch crate. #4 asks
for it on a commit of this repository, and that comparison is the one that can
be repeated by a reader who has only this repository. It is owed as soon as
there is something here to build.
