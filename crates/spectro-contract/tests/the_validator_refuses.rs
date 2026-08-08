//! What the validator accepts and what it refuses.
//!
//! Every file in this suite is built from a literal in this source rather than
//! from a file on disk. `docs/fixtures.md` gives the reason and measured it: a
//! fixture carrying a carriage return, a byte order mark or a trailing space is
//! the fixture most likely to be damaged on its way into the repository, and a
//! test proving the reader survives a carriage return passes for the wrong
//! reason once the carriage return has been deleted. A literal decides the
//! bytes in the source, so nothing between the author and the reader can change
//! them.
//!
//! Each refusal test perturbs exactly one thing in a file the suite has already
//! shown to be accepted, so a red result is a result about that one thing.

use spectro_contract::{
    Refusals, covariance, level_set, line_list, rates, validate_covariance, validate_level_set,
    validate_line_list, validate_rate_table,
};

// ---------------------------------------------------------------- the fixtures

const LEVEL_HEADER: &str = "#contract_version\t1.0\n\
                            #energy_unit\tcm-1\n\
                            #energy_reference\tground_state\n\
                            #level_set_id\tfe2-2026-08-08\n";

const LEVEL_LABELS: &str = "level_id\tspecies\tenergy\tenergy_uncertainty\t\
                            uncertainty_kind\tuncertainty_class\tparity\tj\torigin\t\
                            configuration\tterm\n";

const LEVEL_ROWS: &str = "L1\tFe II\t0.0\t0.002\tstandard_deviation\t\teven\t4.5\tmeasured\t3d6.(5D).4s\ta6D\n\
                          L2\tFe II\t384.7908\t0.004\tspread\t\teven\t3.5\tmeasured\t3d6.(5D).4s\ta6D\n\
                          L3\tFe II\t62171.625\tnone\tclass\tD\todd\tunknown\tpredicted\t\t\n";

/// A level set every part of the contract accepts.
fn level_file() -> String {
    format!("{LEVEL_HEADER}{LEVEL_LABELS}{LEVEL_ROWS}")
}

const LINE_HEADER: &str = "#contract_version\t1.0\n\
                           #position_unit\tnm\n\
                           #position_medium\tair_standard\n\
                           #line_list_id\tarc-2026-08-08\n";

const LINE_LABELS: &str = "feature_id\tspectrum_id\tsegment_id\tposition\tposition_medium\t\
                           position_uncertainty\tuncertainty_kind\tuncertainty_class\t\
                           intensity\tintensity_scale\tflags\tritz_position\n";

const LINE_ROWS: &str = "F1\tS1\tord3\t259.9396\t\t0.0004\tstandard_deviation\t\t120\tphotographic\t\t\n\
                         F2\tS1\t\t260.0\tvacuum\tnone\tclass\tB\t\t\tsaturated,blended\t259.998\n";

/// A line list every part of the contract accepts.
fn line_file() -> String {
    format!("{LINE_HEADER}{LINE_LABELS}{LINE_ROWS}")
}

const COVARIANCE_FILE: &str = "#contract_version\t1.0\n\
                               #level_set_id\tfe2-2026-08-08\n\
                               level_id_a\tlevel_id_b\tcovariance\n\
                               L1\tL1\t4e-6\n\
                               L1\tL2\t1e-6\n";

const RATE_FILE: &str = "#contract_version\t1.0\n\
                         #level_set_id\tfe2-2026-08-08\n\
                         #rate_unit\ts-1\n\
                         upper_level_id\tlower_level_id\trate\trate_uncertainty\n\
                         L3\tL1\t2.4e5\t0.3e5\n\
                         L3\tL2\t1.1e5\tnone\n";

// ------------------------------------------------------------------ the helpers

fn refused(result: Result<(), Refusals>) -> Refusals {
    match result {
        Ok(()) => panic!("the validator accepted a file this test built to be refused"),
        Err(refusals) => refusals,
    }
}

/// Whether some refusal names this line and this field.
fn names(refusals: &Refusals, line: Option<usize>, field: &str) -> bool {
    refusals
        .iter()
        .any(|refusal| refusal.line == line && refusal.field.as_deref() == Some(field))
}

/// Replace the first occurrence of `from` with `to`, refusing to be a no-op.
///
/// A perturbation that changed nothing would leave a test asserting a refusal
/// against the file it was derived from, which is a test that proves the wrong
/// thing rather than a test that fails.
fn change(text: &str, from: &str, to: &str) -> String {
    assert!(
        text.contains(from),
        "the perturbation `{from}` is not in the file it was meant to change"
    );
    text.replacen(from, to, 1)
}

// --------------------------------------------------- what the validator accepts

#[test]
fn a_conforming_level_set_is_accepted() {
    validate_level_set(level_file().as_bytes()).expect("the level set is conforming");
}

#[test]
fn a_conforming_line_list_is_accepted() {
    validate_line_list(line_file().as_bytes()).expect("the line list is conforming");
}

#[test]
fn a_conforming_rate_table_is_accepted() {
    validate_rate_table(RATE_FILE.as_bytes()).expect("the rate table is conforming");
}

#[test]
fn a_conforming_covariance_companion_is_accepted() {
    validate_covariance(COVARIANCE_FILE.as_bytes()).expect("the companion is conforming");
}

#[test]
fn the_reader_keeps_the_two_distinctions_the_contract_is_about() {
    let levels = level_set::read(level_file().as_bytes()).expect("the level set is conforming");

    // Unknown and absent are not the same. A file with no `j` column is refused
    // outright, so a `None` here is always a fact about the spectrum.
    assert_eq!(levels.levels[2].j, None);
    assert!(levels.levels[0].j.is_some());

    // Measured and predicted are not the same, and the reader carries which.
    assert_eq!(levels.levels[0].origin, "measured");
    assert_eq!(levels.levels[2].origin, "predicted");

    // A declared absence of uncertainty is not a zero.
    assert_eq!(levels.levels[2].energy_uncertainty, None);
    assert_eq!(levels.levels[1].uncertainty_kind, "spread");
    assert_eq!(levels.levels[2].uncertainty_class.as_deref(), Some("D"));
}

#[test]
fn a_row_medium_overrides_the_header_and_a_silent_row_takes_it() {
    let lines = line_list::read(line_file().as_bytes()).expect("the line list is conforming");
    assert_eq!(lines.lines[0].position_medium, "air_standard");
    assert_eq!(lines.lines[1].position_medium, "vacuum");
}

#[test]
fn a_ritz_position_is_read_and_kept_apart() {
    let lines = line_list::read(line_file().as_bytes()).expect("the line list is conforming");
    assert_eq!(lines.lines[0].ritz_position, None);
    assert_eq!(lines.lines[1].ritz_position, Some(259.998));
}

// ---------------------------------------------------------- the version marker

#[test]
fn an_absent_contract_version_is_refused_rather_than_defaulted() {
    let file = change(&level_file(), "#contract_version\t1.0\n", "");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, None, "contract_version"));
}

#[test]
fn a_major_version_this_reader_does_not_know_is_refused_naming_both() {
    let file = change(
        &level_file(),
        "#contract_version\t1.0",
        "#contract_version\t2.0",
    );
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, Some(1), "contract_version"));
    let said = refusals.to_string();
    assert!(said.contains("major version 2"), "{said}");
    assert!(said.contains("major version 1"), "{said}");
}

#[test]
fn a_later_minor_is_read_and_what_it_added_is_reported_rather_than_dropped() {
    let file = change(
        &level_file(),
        "#contract_version\t1.0",
        "#contract_version\t1.4",
    );
    let file = change(&file, "#energy_unit", "#lifetime_unit\ts\n#energy_unit");
    let file = change(&file, "\tterm\n", "\tterm\tlifetime\n");
    let file = file
        .replace("\ta6D\n", "\ta6D\t1.2\n")
        .replace("\t\t\n", "\t\t\t9e-3\n");

    let levels = level_set::read(file.as_bytes()).expect("a later minor is readable");
    assert_eq!(levels.contract_version.minor, 4);
    assert_eq!(
        levels.unknown_header_fields,
        vec!["lifetime_unit".to_owned()]
    );
    assert_eq!(levels.unknown_columns, vec!["lifetime".to_owned()]);
}

// ------------------------------------------- an absent column, and an empty cell

#[test]
fn a_file_with_no_j_column_at_all_is_refused() {
    let file = change(&level_file(), "\tparity\tj\torigin", "\tparity\torigin");
    let file = file
        .replace("\teven\t4.5\tmeasured", "\teven\tmeasured")
        .replace("\teven\t3.5\tmeasured", "\teven\tmeasured")
        .replace("\todd\tunknown\tpredicted", "\todd\tpredicted");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, None, "j"));
}

#[test]
fn an_absent_column_and_an_empty_cell_are_refused_differently() {
    let absent = change(&level_file(), "\tspecies\tenergy", "\tenergy");
    let absent = absent
        .replace("\tFe II\t0.0", "\t0.0")
        .replace("\tFe II\t384.7908", "\t384.7908")
        .replace("\tFe II\t62171.625", "\t62171.625");
    let absent = refused(validate_level_set(absent.as_bytes()));

    let empty = change(&level_file(), "L1\tFe II\t", "L1\t\t");
    let empty = refused(validate_level_set(empty.as_bytes()));

    // The absent column is a fact about the file and has no line. The empty
    // cell is a fact about one row and names it.
    assert!(names(&absent, None, "species"));
    assert!(names(&empty, Some(6), "species"));
    assert_ne!(absent.to_string(), empty.to_string());
}

#[test]
fn an_unlabelled_column_is_refused_naming_the_label_row() {
    let file = change(&level_file(), "\tterm\n", "\tterm\t\n");
    let file = file
        .replace("a6D\n", "a6D\t\n")
        .replace("\t\t\n", "\t\t\t\n");
    let refusals = refused(validate_level_set(file.as_bytes()));
    let said = refusals.to_string();
    assert!(said.contains("line 5"), "{said}");
    assert!(said.contains("carries no label"), "{said}");
}

#[test]
fn a_duplicated_column_label_is_refused() {
    let file = change(&level_file(), "\tterm\n", "\tterm\tterm\n");
    let file = file
        .replace("a6D\n", "a6D\tx\n")
        .replace("\t\t\n", "\t\tx\n");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, Some(5), "term"));
}

// --------------------------------------------------------------- the vocabulary

#[test]
fn a_parity_outside_the_vocabulary_is_refused_naming_the_vocabulary() {
    let file = change(&level_file(), "\teven\t4.5\t", "\tEven\t4.5\t");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, Some(6), "parity"));
    let said = refusals.to_string();
    assert!(
        said.contains("`even`") && said.contains("`odd`") && said.contains("`unknown`"),
        "{said}"
    );
}

#[test]
fn a_j_that_is_neither_a_half_integer_nor_unknown_is_refused() {
    let file = change(&level_file(), "\t4.5\tmeasured", "\t4.3\tmeasured");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, Some(6), "j"));
    assert!(refusals.to_string().contains("multiple of one half"));
}

#[test]
fn a_negative_j_is_refused() {
    let file = change(&level_file(), "\t4.5\tmeasured", "\t-0.5\tmeasured");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, Some(6), "j"));
}

#[test]
fn both_spellings_of_a_half_integer_j_are_read_and_they_agree() {
    let decimal = level_set::read(level_file().as_bytes()).expect("conforming");
    let fraction = change(&level_file(), "\t4.5\tmeasured", "\t9/2\tmeasured");
    let fraction = level_set::read(fraction.as_bytes()).expect("conforming");
    assert_eq!(decimal.levels[0].j, fraction.levels[0].j);
    assert_eq!(decimal.levels[0].j.expect("a J").to_string(), "9/2");
}

#[test]
fn a_class_uncertainty_with_no_class_named_is_refused() {
    let file = change(&level_file(), "\tnone\tclass\tD\t", "\tnone\tclass\t\t");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, Some(8), "uncertainty_class"));
}

// ------------------------------------------------------------------ the numbers

#[test]
fn a_padded_number_is_refused_because_no_cell_is_trimmed() {
    let file = change(&level_file(), "\t384.7908\t", "\t 384.7908\t");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, Some(7), "energy"));
    assert!(refusals.to_string().contains("no cell is trimmed"));
}

#[test]
fn an_energy_with_no_number_in_it_is_refused_naming_the_field_and_the_line() {
    let file = change(&level_file(), "\t62171.625\t", "\t62171,625\t");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, Some(8), "energy"));
}

#[test]
fn an_infinite_energy_is_refused() {
    let file = change(&level_file(), "\t62171.625\t", "\tinf\t");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, Some(8), "energy"));
}

// ------------------------------------------------------------------- the bytes

#[test]
fn crlf_endings_are_read_and_the_carriage_return_is_not_part_of_a_field() {
    let file = level_file().replace('\n', "\r\n");
    let levels = level_set::read(file.as_bytes()).expect("CRLF is a conforming ending");
    assert_eq!(levels.energy_unit, "cm-1");
    assert_eq!(levels.levels[0].term, "a6D");
}

#[test]
fn a_file_mixing_lf_and_crlf_is_refused() {
    let file = change(
        &level_file(),
        "#energy_unit\tcm-1\n",
        "#energy_unit\tcm-1\r\n",
    );
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(refusals.to_string().contains("may not mix LF and CRLF"));
}

#[test]
fn a_carriage_return_inside_a_line_is_refused_rather_than_treated_as_whitespace() {
    let file = change(&level_file(), "\ta6D\n", "\ta6D\r\ta6D\n");
    let file = change(&file, "\tterm\n", "\tterm\tterm_two\n");
    let file = change(&file, "\ta6D\nL3", "\ta6D\t\nL3");
    let file = change(&file, "\t\t\n", "\t\t\t\n");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(refusals.to_string().contains("carriage return"));
}

#[test]
fn a_utf8_byte_order_mark_is_accepted_and_is_not_part_of_the_first_field() {
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(level_file().as_bytes());
    let levels = level_set::read(&bytes).expect("a leading UTF-8 byte order mark is accepted");
    assert_eq!(levels.level_set_id, "fe2-2026-08-08");
}

#[test]
fn a_utf16_byte_order_mark_is_refused() {
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend_from_slice(level_file().as_bytes());
    let refusals = refused(validate_level_set(&bytes));
    assert!(refusals.to_string().contains("UTF-16"));
}

#[test]
fn bytes_that_are_not_utf8_are_refused_naming_where() {
    let mut bytes = level_file().into_bytes();
    let at = bytes.len() - 1;
    bytes[at] = 0xC3;
    let refusals = refused(validate_level_set(&bytes));
    assert!(refusals.to_string().contains("not UTF-8"));
}

#[test]
fn a_file_with_no_trailing_terminator_is_accepted() {
    let file = level_file();
    let trimmed = file.strip_suffix('\n').expect("the fixture ends with LF");
    validate_level_set(trimmed.as_bytes()).expect("a last line may end with no terminator");
}

// ------------------------------------------------------------------ the shape

#[test]
fn a_comment_below_the_label_row_is_refused_rather_than_skipped() {
    let file = change(&level_file(), "L3\tFe II", "#L3 was withdrawn\nL3\tFe II");
    let refusals = refused(validate_level_set(file.as_bytes()));
    let said = refusals.to_string();
    assert!(said.contains("line 8"), "{said}");
    assert!(said.contains("begins with `#`"), "{said}");
}

#[test]
fn a_row_with_a_dropped_tab_is_refused_as_a_row_shape() {
    let file = change(&level_file(), "L2\tFe II\t", "L2\tFe II");
    let refusals = refused(validate_level_set(file.as_bytes()));
    let said = refusals.to_string();
    assert!(said.contains("line 7"), "{said}");
    assert!(said.contains("10 cells against 11 labels"), "{said}");
}

#[test]
fn a_blank_line_in_the_body_is_refused() {
    let file = change(&level_file(), "L3\tFe II", "\nL3\tFe II");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(refusals.to_string().contains("no bytes in it"));
}

#[test]
fn a_file_with_no_label_row_is_refused() {
    let refusals = refused(validate_level_set(LEVEL_HEADER.as_bytes()));
    assert!(refusals.to_string().contains("no column label row"));
}

#[test]
fn a_repeated_level_id_is_refused() {
    let file = change(&level_file(), "L2\tFe II", "L1\tFe II");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(names(&refusals, Some(7), "level_id"));
}

// -------------------------------------------------------------- the line list

#[test]
fn an_intensity_with_no_scale_beside_it_is_refused() {
    let file = change(&line_file(), "\t120\tphotographic\t", "\t120\t\t");
    let refusals = refused(validate_line_list(file.as_bytes()));
    assert!(names(&refusals, Some(6), "intensity_scale"));
}

#[test]
fn a_file_of_intensities_with_no_scale_column_at_all_is_refused() {
    let file = change(
        &line_file(),
        "\tintensity\tintensity_scale\t",
        "\tintensity\t",
    );
    let file = change(&file, "\t120\tphotographic\t", "\t120\t");
    let file = change(&file, "\tB\t\t\tsaturated", "\tB\t\tsaturated");
    let refusals = refused(validate_line_list(file.as_bytes()));
    assert!(names(&refusals, None, "intensity_scale"));
}

#[test]
fn a_line_list_carrying_no_intensity_column_is_accepted() {
    let file = change(&line_file(), "\tintensity\tintensity_scale\t", "\t");
    let file = change(&file, "\t120\tphotographic\t", "\t");
    let file = change(&file, "\tB\t\t\tsaturated", "\tB\tsaturated");
    validate_line_list(file.as_bytes()).expect("intensity is optional");
}

#[test]
fn a_header_medium_that_names_a_rule_rather_than_a_value_is_refused() {
    let file = change(
        &line_file(),
        "#position_medium\tair_standard",
        "#position_medium\tair_above_200nm",
    );
    let refusals = refused(validate_line_list(file.as_bytes()));
    assert!(names(&refusals, Some(3), "position_medium"));
    assert!(refusals.to_string().contains("value rather than a rule"));
}

#[test]
fn a_flag_outside_the_vocabulary_is_refused() {
    let file = change(&line_file(), "saturated,blended", "saturated,smeared");
    let refusals = refused(validate_line_list(file.as_bytes()));
    assert!(names(&refusals, Some(7), "flags"));
}

#[test]
fn a_repeated_feature_id_is_refused() {
    let file = change(&line_file(), "F2\tS1", "F1\tS1");
    let refusals = refused(validate_line_list(file.as_bytes()));
    assert!(names(&refusals, Some(7), "feature_id"));
}

// ---------------------------------------------------------- across two files

#[test]
fn a_covariance_entry_naming_a_level_the_set_does_not_carry_is_refused() {
    let file = change(COVARIANCE_FILE, "L1\tL2\t1e-6", "L1\tL9\t1e-6");
    let companion = covariance::read(file.as_bytes()).expect("the companion is well formed");
    let levels = level_set::read(level_file().as_bytes()).expect("conforming");
    let refusals = refused(companion.check_against(&levels));
    assert!(names(&refusals, Some(5), "level_id_b"));
}

#[test]
fn a_rate_naming_a_level_the_set_does_not_carry_is_refused() {
    let file = change(RATE_FILE, "L3\tL2\t1.1e5", "L3\tL9\t1.1e5");
    let table = rates::read(file.as_bytes()).expect("the table is well formed");
    let levels = level_set::read(level_file().as_bytes()).expect("conforming");
    let refusals = refused(table.check_against(&levels));
    assert!(names(&refusals, Some(6), "lower_level_id"));
}

#[test]
fn a_companion_stated_against_another_level_set_is_refused() {
    let file = change(COVARIANCE_FILE, "fe2-2026-08-08", "fe2-2026-07-01");
    let companion = covariance::read(file.as_bytes()).expect("the companion is well formed");
    let levels = level_set::read(level_file().as_bytes()).expect("conforming");
    let refusals = refused(companion.check_against(&levels));
    assert!(names(&refusals, None, "level_set_id"));
}

#[test]
fn a_level_set_naming_a_companion_that_was_not_offered_is_refused() {
    let file = change(
        &level_file(),
        "#level_set_id",
        "#covariance_file\tfe2.cov.tsv\n#level_set_id",
    );
    let refusals = refused(spectro_contract::validate_input(
        file.as_bytes(),
        None,
        None,
    ));
    assert!(names(&refusals, None, "covariance_file"));
}

#[test]
fn a_level_set_with_its_companions_is_accepted_together() {
    let file = change(
        &level_file(),
        "#level_set_id",
        "#covariance_file\tfe2.cov.tsv\n#level_set_id",
    );
    spectro_contract::validate_input(
        file.as_bytes(),
        Some(COVARIANCE_FILE.as_bytes()),
        Some(RATE_FILE.as_bytes()),
    )
    .expect("the three files agree");
}

// ---------------------------------- the validator is the reader, not its cousin

/// The finding written into issue #18: a validator written beside the readers
/// diverges from them, and the day it does, a file the validator accepted is a
/// file a reader rejects, which is worse for a producer than no validator.
///
/// This holds the property rather than the implementation. It runs both entry
/// points over every file this suite carries, conforming and not, and requires
/// them to agree in both directions.
#[test]
fn the_validator_accepts_exactly_what_the_reader_accepts() {
    let mut files: Vec<String> = vec![
        level_file(),
        change(&level_file(), "#contract_version\t1.0\n", ""),
        change(&level_file(), "\t4.5\tmeasured", "\t4.3\tmeasured"),
        change(&level_file(), "\teven\t4.5\t", "\tEven\t4.5\t"),
        change(&level_file(), "L2\tFe II\t", "L2\tFe II"),
        LEVEL_HEADER.to_owned(),
    ];
    files.push(level_file().replace('\n', "\r\n"));

    for file in &files {
        let read = level_set::read(file.as_bytes());
        let validated = validate_level_set(file.as_bytes());
        assert_eq!(
            read.is_ok(),
            validated.is_ok(),
            "the reader and the validator disagree about a file"
        );
        if let (Err(read), Err(validated)) = (read, validated) {
            assert_eq!(read, validated, "the two disagree about what was wrong");
        }
    }
}

/// Every refusal names a place. A refusal that said only that a file was
/// invalid would send the producer back to read their own file against a
/// document, which is the work the validator exists to do for them.
#[test]
fn every_refusal_in_this_suite_names_a_field_or_a_line() {
    let files = [
        change(&level_file(), "\t4.5\tmeasured", "\t4.3\tmeasured"),
        change(&level_file(), "L2\tFe II\t", "L2\tFe II"),
        change(&level_file(), "\t384.7908\t", "\t 384.7908\t"),
        change(&level_file(), "#contract_version\t1.0\n", ""),
    ];
    for file in &files {
        for refusal in &refused(validate_level_set(file.as_bytes())) {
            assert!(
                refusal.line.is_some() || refusal.field.is_some(),
                "a refusal named no place: {refusal}"
            );
        }
    }
}

/// What a producer actually reads when their emitter is wrong.
///
/// The exact text is asserted rather than its shape, because the reason this
/// crate exists is that the message is the whole product for somebody checking
/// an emitter, and a message that drifts into vagueness would pass every other
/// test in this file.
#[test]
fn the_message_a_producer_sees() {
    let file = change(&level_file(), "\t4.5\tmeasured", "\t4.3\tmeasured");
    let file = change(&file, "\teven\t3.5\t", "\tEven\t3.5\t");
    let file = change(&file, "\t62171.625\t", "\t 62171.625\t");
    let said = refused(validate_level_set(file.as_bytes())).to_string();
    assert_eq!(
        said,
        "line 6, field `j`: `4.3` is not a multiple of one half, and a J is never rounded to one, \
         and a J that was never determined is written `unknown`\n\
         line 7, field `parity`: `Even` is not one of `even`, `odd`, `unknown`\n\
         line 8, field `energy`: ` 62171.625` is padded with whitespace, and no cell is trimmed \
         before it is read"
    );
}
