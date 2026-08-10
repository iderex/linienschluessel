//! The seed corpus replay of issue #56.
//!
//! Coverage-guided fuzzing runs out of band and finds crashers. This is the
//! other half: the committed seeds are handed back to their readers on every
//! run of the suite, so a regression in a reader reds the change that caused it
//! instead of being found days later by a scheduled job.
//!
//! It needs none of the fuzzing apparatus. Every reader here takes a byte slice
//! and returns a result, so the replay is ordinary Rust on the channel
//! `rust-toolchain.toml` pins: no nightly, no sanitizer, no added dependency.
//! Which channel the coverage-guided half is built on is undecided and is
//! written on issue #56; nothing below waits on that answer.
//!
//! Why the whole corpus sits in this crate rather than beside each reader.
//! `no-adapter-reached-from-the-rest-of-the-tree` in
//! `.github/workflows/invariants.yml` refuses the string `spectro-adapters`
//! anywhere under `crates` outside this crate, and the record behind it is
//! `docs/decisions/layout.md`. So no other crate may name the adapters, and
//! this one already depends on `spectro-contract`, which makes it the only
//! place a single replay can reach every reader in the tree. The cost is that
//! the contract readers' seeds do not sit beside the contract readers, and
//! somebody changing one of those has to be sent here by this comment.
//!
//! What the replay asserts is not that a reader accepts a seed. Half the corpus
//! is bytes a reader has to refuse, and a refusal is the reader working. The
//! assertion is that no seed makes a reader panic, and that no seed is skipped
//! in silence.
//!
//! The targets are the directories. Adding a directory adds a target and adding
//! a file to one adds a seed, with nothing to remember; a directory this file
//! binds to no reader is refused rather than walked past, so a corpus cannot
//! grow a target that runs nothing. It does not hold in the other direction: a
//! reader added to the tree with no corpus directory of its own is not caught
//! here, and nothing else catches it either.

use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use spectro_adapters::{nist_asd_levels, nist_asd_lines};
use spectro_contract::{
    validate_covariance, validate_level_set, validate_line_list, validate_rate_table,
};

/// The suffix `docs/fixtures.md` gives a fixture's record, which is not a seed.
const RECORD_SUFFIX: &str = ".origin.md";

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fuzz-corpus")
}

/// Hand one seed to the reader the directory names, and say whether one was
/// found. The result of the read is thrown away on purpose: a conforming seed
/// is accepted and a hostile one is refused, and both are the reader working.
fn hand_to_the_reader(target: &str, bytes: &[u8]) -> bool {
    match target {
        "level_set" => {
            let _ = validate_level_set(bytes);
        }
        "line_list" => {
            let _ = validate_line_list(bytes);
        }
        "rate_table" => {
            let _ = validate_rate_table(bytes);
        }
        "covariance" => {
            let _ = validate_covariance(bytes);
        }
        "nist_asd_levels" => {
            let naming = nist_asd_levels::Naming {
                species: "Fe II".to_owned(),
                level_set_id: "seed-level-set".to_owned(),
            };
            let _ = nist_asd_levels::convert(bytes, &naming);
        }
        "nist_asd_lines" => {
            let naming = nist_asd_lines::Naming {
                spectrum_id: "S1".to_owned(),
                line_list_id: "seed-line-list".to_owned(),
            };
            let _ = nist_asd_lines::convert(bytes, &naming);
        }
        _ => return false,
    }
    true
}

/// The entries of a directory, sorted, so two runs replay in one order.
fn sorted_entries(directory: &Path) -> Vec<PathBuf> {
    let read = fs::read_dir(directory).unwrap_or_else(|error| {
        panic!(
            "the corpus at {} cannot be read: {error}",
            directory.display()
        )
    });
    let mut entries: Vec<PathBuf> = read
        .map(|entry| entry.expect("a corpus entry cannot be read").path())
        .collect();
    entries.sort();
    entries
}

#[test]
fn every_seed_in_the_corpus_reaches_a_reader_and_none_of_them_panics() {
    let root = corpus_root();
    assert!(
        root.is_dir(),
        "the corpus root {} is not there, so this run replayed nothing and a pass would mean nothing",
        root.display()
    );

    let mut targets: Vec<PathBuf> = Vec::new();
    for entry in sorted_entries(&root) {
        assert!(
            entry.is_dir(),
            "{} sits in the corpus root rather than in a target directory, so no reader would be handed it",
            entry.display()
        );
        targets.push(entry);
    }
    assert!(
        !targets.is_empty(),
        "the corpus at {} holds no target directory, so this run replayed nothing",
        root.display()
    );

    let mut replayed = 0usize;
    for target in &targets {
        let name = target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| {
                panic!(
                    "{} is not a name a target can be read from",
                    target.display()
                )
            })
            .to_owned();

        let seeds: Vec<PathBuf> = sorted_entries(target)
            .into_iter()
            .filter(|path| !path.to_string_lossy().ends_with(RECORD_SUFFIX))
            .collect();
        assert!(
            !seeds.is_empty(),
            "the corpus directory `{name}` holds no seed, so the reader behind it was replayed against nothing"
        );

        for seed in seeds {
            assert!(
                seed.is_file(),
                "{} sits inside the target directory `{name}` and is not a file, so it is neither a seed nor a target and nothing would replay it",
                seed.display()
            );
            let bytes = fs::read(&seed).unwrap_or_else(|error| {
                panic!("the seed {} cannot be read: {error}", seed.display())
            });
            let found = catch_unwind(AssertUnwindSafe(|| hand_to_the_reader(&name, &bytes)))
                .unwrap_or_else(|_| {
                    panic!(
                        "the reader behind `{name}` panicked on the seed {}. A crasher is a finding with its own repair, minimised and fixed in the reader rather than patched here.",
                        seed.display()
                    )
                });
            assert!(
                found,
                "the corpus directory `{name}` names no reader in this file, so its seeds are carried and replayed by nothing"
            );
            replayed += 1;
        }
    }

    eprintln!(
        "replayed {replayed} seed(s) across {} target(s) under {}",
        targets.len(),
        root.display()
    );
}
