//! The levels export adapter of issue #22.
//!
//! The fixtures are synthetic and every one of them is a literal in this
//! source, which is `docs/fixtures.md`'s safer construction: the bytes reaching
//! the adapter are decided here rather than by anything on the way into git.
//!
//! Synthetic, and deliberately so. The shape is the shape of the exports
//! retrieved on 2026-08-09 from the query form `docs/sources.md` names, down to
//! the quotation marks, the bare `g` cell on a limit row, the padded leading
//! percentages and the column that appears in one spectrum's export and not in
//! another's. None of the numbers is one of that source's numbers. Whether this
//! repository may carry an extract of it is entry 2 of issue #1 and is the
//! maintainer's to answer, and nothing here waits on that answer.
//!
//! The property each fixture was built to have is written above the fixture.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use spectro_adapters::nist_asd_levels::{self, FIELDS, Naming, Source};
use spectro_contract::{level_set, validate_level_set};

fn naming() -> Naming {
    Naming {
        species: "Ne II".to_owned(),
        level_set_id: "levels-2026-08-09".to_owned(),
    }
}

/// One export with every shape this module has to survive standing in it.
///
/// Rows 2 and 3 are ordinary measured levels of opposite parity. Row 4 carries
/// no J. Row 5 carries a suffix the contract has no field for. Row 6 carries no
/// uncertainty and a term with no asterisk. Row 7 is an ab-initio value in
/// parentheses. Row 8 carries the autoionisation annotation. Row 9 gives two
/// possible J values, an empty configuration and a bare parity marker for a
/// term. Row 10 is a term the source lists with no level. Row 11 stands on an
/// unestablished connection to the rest of the spectrum. Row 12 is an
/// ionisation limit, with the bare unquoted `g` cell those rows carry. Row 13
/// is an interpolated value in square brackets.
const ONE_SPECTRUM: &str = concat!(
    "Configuration\tTerm\tJ\tg\tPrefix\tLevel (cm-1)\tSuffix\tUncertainty (cm-1)\tLeading percentages\tReference\n",
    "\"2s2.2p5\"\t\"2P*\"\t\"3/2\"\t4\t\"\"\t\"0.0000\"\t\"\"\t\"0\"\t\" 100          \"\t\"L1\"\n",
    "\"2s.2p6\"\t\"2S\"\t\"1/2\"\t2\t\"\"\t\"216333.412\"\t\"\"\t\"0.09\"\t\"  98          \"\t\"L1\"\n",
    "\"2s2.2p4.(3P<2>).6h\"\t\"2[5]*\"\t\"\"\t22\t\"\"\t\"318111.507\"\t\"\"\t\"0.012\"\t\" 100          \"\t\"L1\"\n",
    "\"2s2.2p4.(1D).4d\"\t\"2D\"\t\"5/2\"\t6\t\"\"\t\"329001\"\t\"?\"\t\"3\"\t\"  39          \"\t\"L2\"\n",
    "\"3d9.(2D<5/2>).4s\"\t\"(5/2,1/2)\"\t\"3\"\t7\t\"\"\t\"1694333\"\t\"\"\t\"\"\t\" 100          \"\t\"\"\n",
    "\"1s.2s\"\t\"3S\"\t\"1\"\t3\t\"(\"\t\"7311818.12\"\t\")\"\t\"0.40\"\t\" 100          \"\t\"L3\"\n",
    "\"3p5.3d10.4p\"\t\"(2,7/2)*\"\t\"3/2\"\t4\t\"\"\t\"21275111\"\t\"a\"\t\"15000\"\t\"  78          \"\t\"L4\"\n",
    "\"\"\t\"*\"\t\"1 or 2\"\t3\t\"\"\t\"12297.111\"\t\"\"\t\"\"\t\"              \"\t\"\"\n",
    "\"4f3.(4I*).5d.6s2\"\t\"5L*\"\t\"10\"\t21\t\"\"\t\"\"\t\"\"\t\"\"\t\"  99          \"\t\"\"\n",
    "\"\"\t\"*\"\t\"2\"\t5\t\"\"\t\"30875.111\"\t\"+x\"\t\"\"\t\"              \"\t\"\"\n",
    "\"Ne III (2s2.2p4 3P<2>)\"\t\"Limit\"\t\"---\"\t\t\"\"\t\"331111.7\"\t\"\"\t\"0.3\"\t\"              \"\t\"L1\"\n",
    "\"2s2.2p4.(3P).3p\"\t\"4P*\"\t\"5/2\"\t6\t\"[\"\t\"246777.111\"\t\"]\"\t\"0.0014\"\t\"  99          \"\t\"L5\"\n",
);

fn converted() -> nist_asd_levels::Conversion {
    nist_asd_levels::convert(ONE_SPECTRUM.as_bytes(), &naming())
        .expect("the fixture is an export this adapter can read")
}

fn levels() -> level_set::LevelSet {
    level_set::read(converted().level_set.as_bytes()).expect("the emitted file reads back")
}

// ------------------------------------------------ what the Done-when asks for

#[test]
fn the_export_becomes_a_level_set_the_validator_accepts() {
    let conversion = converted();
    validate_level_set(conversion.level_set.as_bytes())
        .expect("the emitted file is a conforming level set");

    // Twelve data rows, three of which are not levels of this species: the term
    // with no level, the one on an unestablished connection, and the limit.
    let read = levels();
    assert_eq!(read.levels.len(), 9);
    assert_eq!(read.energy_unit, "cm-1");
    assert_eq!(read.energy_reference, "ground_level");
    assert_eq!(read.level_set_id, "levels-2026-08-09");
    assert!(read.levels.iter().all(|level| level.species == "Ne II"));
}

#[test]
fn every_contract_field_is_mapped_or_reported_unavailable() {
    let conversion = converted();

    // What the emitted file carries, taken out of the file rather than out of a
    // list: the names in its header block and the labels of its label row.
    let mut carried: BTreeSet<&str> = BTreeSet::new();
    for line in conversion.level_set.lines() {
        match line.strip_prefix('#') {
            Some(header) => {
                carried.insert(header.split('\t').next().expect("a header line has a name"));
            }
            None => {
                carried.extend(line.split('\t'));
                break;
            }
        }
    }

    // The reader is the authority for the contract's own field list, and it is
    // the authority in exactly two directions. Accepting the file says every
    // field the contract requires is in it. Reporting no unknown header field
    // and no unknown column says the file carries nothing the contract does not
    // know. Between the two, the label row above is the contract's required set
    // rather than this module's idea of it.
    validate_level_set(conversion.level_set.as_bytes()).expect("conforming");
    let read = levels();
    assert!(read.unknown_header_fields.is_empty());
    assert!(read.unknown_columns.is_empty());

    // What no reading of the tree gives is the contract's optional header
    // fields, because the fields of a struct cannot be enumerated from inside
    // the language. The two this upstream cannot fill are therefore named here
    // rather than derived, and each is asserted absent from the emitted file as
    // well as declared unavailable, so a later change that started emitting one
    // would move both halves or fail.
    assert_eq!(read.covariance_file, None);
    assert_eq!(read.derived_from_line_lists, None);

    let present: BTreeSet<&str> = FIELDS
        .iter()
        .filter(|(_, source)| !matches!(source, Source::Unavailable(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(present, carried, "FIELDS and the emitted file disagree");

    let absent: BTreeSet<&str> = FIELDS
        .iter()
        .filter(|(_, source)| matches!(source, Source::Unavailable(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        absent,
        BTreeSet::from(["covariance_file", "derived_from_line_lists"])
    );

    // An entry whose reason is empty says nothing, which is what this table
    // exists not to be.
    for (name, source) in FIELDS {
        let said = match source {
            Source::Column(label) => *label,
            Source::Operator => "the operator",
            Source::Fixed(reason) | Source::Empty(reason) | Source::Unavailable(reason) => *reason,
        };
        assert!(!said.is_empty(), "`{name}` names no source");
    }
}

/// The boundary of `docs/decisions/input-contract.md`, in the half a suite can
/// hold. No crate but this one may declare a dependency on this one, and the
/// compiler then refuses an import from any of them for want of the crate.
///
/// The whole-tree half is the workflow rule
/// `no-adapter-reached-from-the-rest-of-the-tree`, which searches the source as
/// well as the manifests and runs on a push. This is the half that runs offline
/// in the default suite, so a contributor meets it before pushing rather than
/// after.
#[test]
fn no_crate_outside_this_one_declares_a_dependency_on_it() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("this crate sits two directories under the workspace root");

    let mut reaching: Vec<String> = Vec::new();
    for entry in fs::read_dir(root.join("crates")).expect("the workspace has a crates directory") {
        let crate_directory = entry.expect("a readable directory entry").path();
        if crate_directory.file_name() == Some("spectro-adapters".as_ref()) {
            continue;
        }
        let manifest = crate_directory.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&manifest) else {
            continue;
        };
        if text.contains("spectro-adapters") {
            reaching.push(manifest.display().to_string());
        }
    }
    assert!(
        reaching.is_empty(),
        "these manifests reach the adapters: {reaching:?}"
    );

    // The workspace root is a virtual manifest, so it declares no dependency of
    // its own and there is nowhere in it for one to hide. The day it becomes a
    // package, this fails rather than quietly stopping to cover it.
    let root_manifest = fs::read_to_string(root.join("Cargo.toml")).expect("a workspace manifest");
    assert!(
        !root_manifest.contains("[dependencies]"),
        "the workspace root has grown a dependency section and this check no longer covers it"
    );
}

// ------------------------------------- what each reading is, and what it is not

#[test]
fn a_column_is_found_by_its_label_and_never_by_its_position() {
    // The same rows, from a spectrum whose export carries the measured Lande
    // factor, so every column past the uncertainty sits one place further along.
    // One retrieval of each on 2026-08-09 differed in exactly this way.
    let widened: String = ONE_SPECTRUM
        .lines()
        .map(|line| {
            let mut cells: Vec<&str> = line.split('\t').collect();
            cells.insert(
                8,
                if cells[0] == "Configuration" {
                    "Lande"
                } else {
                    "\"1.170\""
                },
            );
            format!("{}\n", cells.join("\t"))
        })
        .collect();

    let from_widened = nist_asd_levels::convert(widened.as_bytes(), &naming())
        .expect("the widened export is one this adapter can read");
    assert_eq!(from_widened.level_set, converted().level_set);
}

#[test]
fn a_bracketed_or_bracketless_level_decides_measured_against_predicted() {
    let read = levels();
    let origin = |energy: f64| {
        read.levels
            .iter()
            .find(|level| level.energy == energy)
            .unwrap_or_else(|| panic!("{energy} is in the emitted file"))
            .origin
    };

    // A bare value is an evaluated experimental one.
    assert_eq!(origin(216333.412), "measured");
    // Parentheses are an ab-initio calculation, square brackets a semi-empirical
    // procedure, and the contract has one word for both.
    assert_eq!(origin(7311818.12), "predicted");
    assert_eq!(origin(246777.111), "predicted");
}

#[test]
fn an_unknown_prefix_is_refused_rather_than_read_as_a_measurement() {
    let seeded = ONE_SPECTRUM.replace("\"(\"\t\"7311818.12\"", "\"{\"\t\"7311818.12\"");
    let refusals = nist_asd_levels::convert(seeded.as_bytes(), &naming())
        .expect_err("a prefix this module does not know is refused");
    assert_eq!(refusals.len(), 1);
    let refusal = refusals.iter().next().expect("one refusal");
    assert_eq!(refusal.field.as_deref(), Some("origin"));
    assert_eq!(refusal.line, Some(7));
}

#[test]
fn parity_comes_from_the_asterisk_and_an_unlabelled_term_leaves_it_unknown() {
    let read = levels();
    let parity = |energy: f64| {
        read.levels
            .iter()
            .find(|level| level.energy == energy)
            .unwrap_or_else(|| panic!("{energy} is in the emitted file"))
            .parity
    };

    assert_eq!(parity(0.0), "odd"); // 2P*
    assert_eq!(parity(216333.412), "even"); // 2S
    assert_eq!(parity(1694333.0), "even"); // (5/2,1/2)
    assert_eq!(parity(21275111.0), "odd"); // (2,7/2)*
    assert_eq!(parity(12297.111), "odd"); // a bare parity marker and no term

    // An empty term states no parity, and the contract's word for that is not
    // `even`.
    let seeded = ONE_SPECTRUM.replace("\"2S\"\t\"1/2\"", "\"\"\t\"1/2\"");
    let unlabelled = nist_asd_levels::convert(seeded.as_bytes(), &naming()).expect("readable");
    let read = level_set::read(unlabelled.level_set.as_bytes()).expect("reads back");
    let level = read
        .levels
        .iter()
        .find(|level| level.energy == 216333.412)
        .expect("still emitted");
    assert_eq!(level.parity, "unknown");
    assert_eq!(level.term, "");
}

#[test]
fn an_ionisation_limit_is_not_a_level_of_the_spectrum_it_stands_in() {
    let conversion = converted();
    assert!(
        !conversion.level_set.contains("331111.7"),
        "a limit reached the emitted level set"
    );
    let reported = conversion
        .not_taken
        .iter()
        .find(|not_taken| not_taken.value == "331111.7")
        .expect("the limit is reported rather than dropped");
    assert_eq!(reported.column, "Term");
    assert_eq!(reported.line, 12);
    assert!(reported.reason.contains("next ion"));
}

#[test]
fn a_level_with_no_established_connection_carries_no_energy_into_the_file() {
    let conversion = converted();
    assert!(
        !conversion.level_set.contains("30875.111"),
        "a level standing on an unknown offset reached the emitted level set"
    );
    let reported = conversion
        .not_taken
        .iter()
        .find(|not_taken| not_taken.line == 11)
        .expect("the row is reported rather than dropped");
    assert_eq!(reported.column, "Suffix");
    assert_eq!(reported.value, "30875.111+x");
}

#[test]
fn a_term_the_source_lists_without_a_level_is_reported_rather_than_dropped() {
    let reported = converted()
        .not_taken
        .into_iter()
        .find(|not_taken| not_taken.line == 10)
        .expect("the row is reported");
    assert_eq!(reported.column, "Level");
    assert_eq!(reported.value, "5L*");
}

#[test]
fn a_j_that_names_two_values_becomes_unknown_and_the_cell_is_reported() {
    let read = levels();
    let level = read
        .levels
        .iter()
        .find(|level| level.energy == 12297.111)
        .expect("the level is emitted");
    assert_eq!(level.j, None);

    let reported = converted()
        .not_taken
        .into_iter()
        .find(|not_taken| not_taken.column == "J")
        .expect("the cell is reported");
    assert_eq!(reported.value, "1 or 2");
    assert!(reported.reason.contains("inventing"));

    // A J the source did give is not reported and is not lost.
    let carried = read
        .levels
        .iter()
        .find(|level| level.energy == 216333.412)
        .expect("the level is emitted");
    assert_eq!(carried.j.map(|j| j.twice()), Some(1));

    // A J the source never determined is `unknown` and is not a report either:
    // an empty cell is the whole statement and nothing was dropped making it.
    let never = read
        .levels
        .iter()
        .find(|level| level.energy == 318111.507)
        .expect("the level is emitted");
    assert_eq!(never.j, None);
}

#[test]
fn an_absent_uncertainty_is_an_absence_rather_than_a_number() {
    let read = levels();
    let declared = read
        .levels
        .iter()
        .find(|level| level.energy == 216333.412)
        .expect("emitted");
    assert_eq!(declared.energy_uncertainty, Some(0.09));
    assert_eq!(declared.uncertainty_kind, "standard_deviation");
    assert_eq!(declared.uncertainty_class, None);

    let absent = read
        .levels
        .iter()
        .find(|level| level.energy == 1694333.0)
        .expect("emitted");
    assert_eq!(absent.energy_uncertainty, None);
    assert_eq!(absent.uncertainty_kind, "standard_deviation");
}

#[test]
fn an_annotation_the_contract_has_no_field_for_is_reported_and_not_read() {
    let conversion = converted();
    let annotations: Vec<&str> = conversion
        .not_taken
        .iter()
        .filter(|not_taken| not_taken.column == "Suffix" && !not_taken.value.contains('+'))
        .map(|not_taken| not_taken.value.as_str())
        .collect();
    assert_eq!(annotations, vec!["?", "a"]);

    // And the closing half of a bracket is not among them, because it says what
    // the prefix already said.
    assert!(
        conversion
            .not_taken
            .iter()
            .all(|not_taken| not_taken.value != "]" && not_taken.value != ")")
    );
}

// ------------------------------------------------- what the file may not be

#[test]
fn an_export_retrieved_without_its_term_column_is_refused() {
    let stripped = without_column(ONE_SPECTRUM, 1);
    let refusals = nist_asd_levels::convert(stripped.as_bytes(), &naming())
        .expect_err("an export with no term column is refused");
    let fields: Vec<&str> = refusals
        .iter()
        .filter_map(|refusal| refusal.field.as_deref())
        .collect();
    assert!(fields.contains(&"Term"), "{refusals}");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.reason.contains("parity")),
        "the refusal says what the emitted file would otherwise have claimed"
    );
}

#[test]
fn an_export_retrieved_without_its_j_column_is_refused() {
    let stripped = without_column(ONE_SPECTRUM, 2);
    let refusals = nist_asd_levels::convert(stripped.as_bytes(), &naming())
        .expect_err("an export with no J column is refused");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.field.as_deref() == Some("J")),
        "{refusals}"
    );
}

#[test]
fn an_export_retrieved_without_its_prefix_column_is_refused() {
    let stripped = without_column(ONE_SPECTRUM, 4);
    let refusals = nist_asd_levels::convert(stripped.as_bytes(), &naming())
        .expect_err("an export with no prefix column is refused");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.field.as_deref() == Some("Prefix")),
        "{refusals}"
    );
}

#[test]
fn an_uncertainty_column_in_another_unit_is_refused() {
    let seeded = ONE_SPECTRUM.replace("Uncertainty (cm-1)", "Uncertainty (eV)");
    let refusals = nist_asd_levels::convert(seeded.as_bytes(), &naming())
        .expect_err("an uncertainty in another unit is not this level's uncertainty");
    assert_eq!(refusals.len(), 1);
    assert_eq!(
        refusals.iter().next().and_then(|r| r.field.as_deref()),
        Some("Uncertainty")
    );
}

#[test]
fn a_data_row_before_any_header_row_is_refused_rather_than_guessed_at() {
    let mut lines: Vec<&str> = ONE_SPECTRUM.lines().collect();
    lines.swap(0, 1);
    let swapped = format!("{}\n", lines.join("\n"));
    let refusals = nist_asd_levels::convert(swapped.as_bytes(), &naming())
        .expect_err("a row before the header row is refused");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.line == Some(1) && refusal.field.is_none()),
        "{refusals}"
    );
}

#[test]
fn a_row_of_the_wrong_width_is_refused_rather_than_read_off_by_one() {
    let seeded = ONE_SPECTRUM.replace(
        "\"2s.2p6\"\t\"2S\"\t\"1/2\"\t2\t",
        "\"2s.2p6\"\t\"2S\"\t\"1/2\"\t2\t\"\"\t",
    );
    let refusals = nist_asd_levels::convert(seeded.as_bytes(), &naming())
        .expect_err("a row that does not match the header row is refused");
    assert!(
        refusals.iter().any(|refusal| refusal.line == Some(3)),
        "{refusals}"
    );
}

#[test]
fn a_species_carrying_a_tab_is_refused_before_it_reaches_a_cell() {
    let naming = Naming {
        species: "Ne\tII".to_owned(),
        level_set_id: "levels-2026-08-09".to_owned(),
    };
    let refusals = nist_asd_levels::convert(ONE_SPECTRUM.as_bytes(), &naming)
        .expect_err("a species carrying a tab is refused");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.field.as_deref() == Some("species")),
        "{refusals}"
    );
}

/// A cell padded to a width is what an adapter takes off, and this is the proof
/// that the taking-off bites.
///
/// The padding here is put in by this fixture. The column the retrieved exports
/// pad is `Leading percentages`, which this module does not read, so this is
/// evidence that the guard works and not evidence that this source pads a level
/// value.
#[test]
fn a_padded_cell_reaches_the_emitted_file_without_its_padding() {
    let padded = ONE_SPECTRUM.replace("\"216333.412\"", "\"  216333.412  \"");
    let conversion =
        nist_asd_levels::convert(padded.as_bytes(), &naming()).expect("padding is taken off");
    assert_eq!(conversion.level_set, converted().level_set);
}

/// Drop one column from every row of an export, header row included.
fn without_column(export: &str, column: usize) -> String {
    export
        .lines()
        .map(|line| {
            let mut cells: Vec<&str> = line.split('\t').collect();
            cells.remove(column);
            format!("{}\n", cells.join("\t"))
        })
        .collect()
}
