//! The line export adapter of issue #23.
//!
//! The fixtures are synthetic and every one of them is a literal in this
//! source, which is `docs/fixtures.md`'s safer construction: the bytes reaching
//! the adapter are decided here rather than by anything on the way into git.
//!
//! Synthetic, and deliberately so. The shape is the shape of the export
//! retrieved on 2026-08-09 from the query form `docs/sources.md` names, down to
//! the quotation marks, the bare accuracy column, the trailing tab and the
//! header row that appears again where the medium changes. None of the numbers
//! is one of that source's numbers. Whether this repository may carry an
//! extract of it is entry 2 of issue #1 and is the maintainer's to answer, and
//! nothing here waits on that answer.
//!
//! The property each fixture was built to have is written above the fixture.

use spectro_adapters::{Naming, nist_asd_lines};
use spectro_contract::{line_list, validate_line_list};

fn naming() -> Naming {
    Naming {
        spectrum_id: "S1".to_owned(),
        line_list_id: "export-2026-08-09".to_owned(),
    }
}

/// Three regions in one file, so the medium changes twice and the export says
/// so by printing its header row again. A reader taking the medium from the
/// first header would call every row below `air`, and the rows it would be
/// wrong about are the two ends of the file.
///
/// Row 3 carries an accuracy grade beside a numeric uncertainty. Row 4 carries
/// a Ritz wavelength and no observation. Row 7 carries no uncertainty at all.
const THREE_BLOCKS: &str = concat!(
    "obs_wl_vac(nm)\tunc_obs_wl\tritz_wl_vac(nm)\tunc_ritz_wl\tintens\tAki(s^-1)\tAcc\tline_ref\t\n",
    "\"185.90100\"\t\"0.00030\"\t\"185.90112\"\t\"0.00005\"\t\"750\"\t\"1.88e+05\"\t\t\"L1\"\t\n",
    "\"192.44500\"\t\"0.00040\"\t\"192.44488\"\t\"0.00005\"\t\"80*\"\t\"3.60e+06\"\tC+\t\"L2\"\t\n",
    "\"\"\t\"\"\t\"197.33321\"\t\"0.00006\"\t\"\"\t\"3.41e+05\"\tB\t\"L3\"\t\n",
    "obs_wl_air(nm)\tunc_obs_wl\tritz_wl_air(nm)\tunc_ritz_wl\tintens\tAki(s^-1)\tAcc\tline_ref\t\n",
    "\"259.93960\"\t\"0.00004\"\t\"259.93955\"\t\"0.00005\"\t\"5000\"\t\"8.60e+06\"\tAA\t\"L4\"\t\n",
    "\"1987.73090\"\t\"\"\t\"1987.73130\"\t\"0.00060\"\t\"5\"\t\"9.34e+03\"\tC\t\"L5\"\t\n",
    "obs_wl_vac(nm)\tunc_obs_wl\tritz_wl_vac(nm)\tunc_ritz_wl\tintens\tAki(s^-1)\tAcc\tline_ref\t\n",
    "\"2445.70000\"\t\"0.00120\"\t\"2445.69910\"\t\"0.00060\"\t\"1\"\t\"\"\t\t\"L6\"\t\n",
);

fn converted() -> nist_asd_lines::Conversion {
    nist_asd_lines::convert(THREE_BLOCKS.as_bytes(), &naming())
        .expect("the fixture is an export this adapter can read")
}

// ------------------------------------------------ what the Done-when asks for

#[test]
fn the_export_becomes_a_line_list_the_validator_accepts() {
    let conversion = converted();
    validate_line_list(conversion.line_list.as_bytes())
        .expect("the emitted file is a conforming line list");

    // Six data rows, one of which carries no observation, so five lines.
    let lines = line_list::read(conversion.line_list.as_bytes()).expect("read back");
    assert_eq!(lines.lines.len(), 5);
    assert_eq!(lines.position_unit, "nm");
    assert_eq!(lines.line_list_id, "export-2026-08-09");
    assert!(lines.lines.iter().all(|line| line.spectrum_id == "S1"));
}

#[test]
fn the_medium_follows_the_region_and_never_the_first_header_row() {
    let lines =
        line_list::read(converted().line_list.as_bytes()).expect("the emitted file reads back");

    // Below 200 nm, twice.
    assert_eq!(lines.lines[0].position, 185.901);
    assert_eq!(lines.lines[0].position_medium, "vacuum");
    assert_eq!(lines.lines[1].position_medium, "vacuum");

    // Between 200 nm and 2000 nm, in the other medium.
    assert_eq!(lines.lines[2].position, 259.9396);
    assert_eq!(lines.lines[2].position_medium, "air_standard");
    assert_eq!(lines.lines[3].position, 1987.7309);
    assert_eq!(lines.lines[3].position_medium, "air_standard");

    // Above 2000 nm, back to vacuum.
    assert_eq!(lines.lines[4].position, 2445.7);
    assert_eq!(lines.lines[4].position_medium, "vacuum");

    // And no row leans on the file default to get there, which is what makes
    // the three answers above facts about the blocks rather than about the
    // header the reader happened to see first.
    let conversion = converted();
    let rows: Vec<&str> = conforming_rows(&conversion.line_list);
    assert!(
        rows.iter().all(|row| {
            let medium = row.split('\t').nth(4).expect("the medium column");
            medium == "vacuum" || medium == "air_standard"
        }),
        "every emitted row states its own medium"
    );
}

#[test]
fn a_ritz_only_entry_cannot_reach_the_engine_as_an_observation() {
    let conversion = converted();
    let lines = line_list::read(conversion.line_list.as_bytes()).expect("read back");

    // The Ritz-only entry of the fixture. Its number reaches no position, and
    // it reaches no Ritz column either, because no line was emitted for it.
    assert!(lines.lines.iter().all(|line| line.position != 197.33321));
    assert!(
        !conversion.line_list.contains("197.33321"),
        "{}",
        conversion.line_list
    );

    // It is reported rather than dropped.
    let reported = conversion
        .not_taken
        .iter()
        .find(|entry| entry.column == "obs_wl")
        .expect("the entry with no observation is reported");
    assert_eq!(reported.line, 4);
    assert_eq!(reported.value, "197.33321");
    assert!(reported.reason.contains("Ritz"), "{reported}");
}

#[test]
fn observed_and_ritz_stay_apart_in_the_output() {
    let lines =
        line_list::read(converted().line_list.as_bytes()).expect("the emitted file reads back");

    // Every emitted line carries both, in two fields, and the two differ on
    // every row of this fixture. One column away at all times.
    for line in &lines.lines {
        let ritz = line
            .ritz_position
            .expect("the fixture gives every row a Ritz value");
        assert_ne!(line.position, ritz, "line {}", line.line);
    }
    assert_eq!(lines.lines[0].position, 185.901);
    assert_eq!(lines.lines[0].ritz_position, Some(185.90112));
}

#[test]
fn an_entry_with_no_uncertainty_carries_the_absence_rather_than_a_number() {
    let lines =
        line_list::read(converted().line_list.as_bytes()).expect("the emitted file reads back");

    // The fixture's row 6 gave no uncertainty. Nothing was substituted for it,
    // and nothing else on that row moved.
    assert_eq!(lines.lines[3].position, 1987.7309);
    assert_eq!(lines.lines[3].position_uncertainty, None);
    assert_eq!(lines.lines[3].uncertainty_kind, "standard_deviation");
    assert_eq!(lines.lines[3].uncertainty_class, None);

    // Its neighbours kept theirs.
    assert_eq!(lines.lines[2].position_uncertainty, Some(0.00004));
}

#[test]
fn an_accuracy_grade_becomes_no_uncertainty_and_no_class() {
    let conversion = converted();
    let lines = line_list::read(conversion.line_list.as_bytes()).expect("read back");

    // `AA` sits on the row at 259.9396 and `C+` on the row at 192.445. Neither
    // reaches the emitted file at all: it grades a transition probability, and
    // a line list carries no transition probability to grade.
    assert!(
        !conversion.line_list.contains("AA"),
        "{}",
        conversion.line_list
    );
    assert!(
        !conversion.line_list.contains("C+"),
        "{}",
        conversion.line_list
    );
    assert!(
        lines
            .lines
            .iter()
            .all(|line| line.uncertainty_class.is_none())
    );

    // Reported, with the grade verbatim and the reason.
    let grades: Vec<&str> = conversion
        .not_taken
        .iter()
        .filter(|entry| entry.column == "Acc")
        .map(|entry| entry.value.as_str())
        .collect();
    assert_eq!(grades, vec!["C+", "AA", "C"]);
}

#[test]
fn an_unlabelled_intensity_scale_is_labelled_arbitrary_and_the_code_is_reported() {
    let conversion = converted();
    let lines = line_list::read(conversion.line_list.as_bytes()).expect("read back");

    // The source labels no scale, so every intensity it gave is arbitrary and
    // source dependent, which is the one thing the reader refuses to guess.
    assert_eq!(lines.lines[0].intensity, Some(750.0));
    assert_eq!(lines.lines[0].intensity_scale, Some("arbitrary"));

    // `80*` is eighty with a character code appended. The number is carried and
    // the code is reported, and no line flag is derived from it.
    assert_eq!(lines.lines[1].intensity, Some(80.0));
    assert!(lines.lines.iter().all(|line| line.flags.is_empty()));
    let code = conversion
        .not_taken
        .iter()
        .find(|entry| entry.column == "intens")
        .expect("the character code is reported");
    assert_eq!(code.value, "*");
}

// ------------------------------------------------------- what it refuses

#[test]
fn two_blocks_in_two_units_are_refused_naming_the_line() {
    let mixed = concat!(
        "obs_wl_air(nm)\tunc_obs_wl\tritz_wl_air(nm)\tunc_ritz_wl\tintens\tAcc\t\n",
        "\"259.93960\"\t\"0.00004\"\t\"259.93955\"\t\"0.00005\"\t\"5000\"\tAA\t\n",
        "obs_wl_air(A)\tunc_obs_wl\tritz_wl_air(A)\tunc_ritz_wl\tintens\tAcc\t\n",
        "\"2599.3960\"\t\"0.0004\"\t\"2599.3955\"\t\"0.0005\"\t\"5000\"\tAA\t\n",
    );
    let refusals = nist_asd_lines::convert(mixed.as_bytes(), &naming())
        .expect_err("one file, two units, and a conforming line list has one");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.line == Some(3)
                && refusal.field.as_deref() == Some("position_unit")),
        "{refusals}"
    );
}

#[test]
fn a_block_whose_ritz_column_is_in_the_other_medium_is_refused() {
    let crossed = concat!(
        "obs_wl_air(nm)\tunc_obs_wl\tritz_wl_vac(nm)\tunc_ritz_wl\tintens\tAcc\t\n",
        "\"259.93960\"\t\"0.00004\"\t\"259.86120\"\t\"0.00005\"\t\"5000\"\tAA\t\n",
    );
    let refusals = nist_asd_lines::convert(crossed.as_bytes(), &naming())
        .expect_err("two media in one block is not something to resolve here");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.field.as_deref() == Some("ritz_wl")),
        "{refusals}"
    );
}

#[test]
fn a_data_row_before_any_header_row_is_refused_rather_than_guessed_at() {
    let headless = "\"259.93960\"\t\"0.00004\"\t\"259.93955\"\t\"0.00005\"\t\"5000\"\tAA\t\n";
    let refusals = nist_asd_lines::convert(headless.as_bytes(), &naming())
        .expect_err("nothing says which medium or unit that position is in");
    assert!(
        refusals.iter().any(|refusal| refusal.line == Some(1)),
        "{refusals}"
    );
}

#[test]
fn a_spectrum_identifier_carrying_a_tab_is_refused_before_it_reaches_a_cell() {
    let naming = Naming {
        spectrum_id: "S1\tS2".to_owned(),
        line_list_id: "export-2026-08-09".to_owned(),
    };
    let refusals = nist_asd_lines::convert(THREE_BLOCKS.as_bytes(), &naming)
        .expect_err("a tab inside a cell moves every field to its right");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.field.as_deref() == Some("spectrum_id")),
        "{refusals}"
    );
}

/// The body rows of an emitted file, without its header block or label row.
fn conforming_rows(file: &str) -> Vec<&str> {
    file.lines()
        .skip_while(|line| line.starts_with('#'))
        .skip(1)
        .filter(|line| !line.is_empty())
        .collect()
}
