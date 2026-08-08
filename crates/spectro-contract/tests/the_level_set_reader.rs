//! The level set reader of issue #20.
//!
//! Issue #18's suite holds what the validator refuses. This one holds what the
//! reader produces, the five files issue #20 names, and the two distinctions
//! that decide whether a level set can be used at all.
//!
//! Which of the five files are refusals is decided on #20 rather than here: a
//! measured level, a predicted level carrying a spread and a level with an
//! unknown J are conforming and produce level objects, a file mixing two
//! ionisation stages is conforming because `species` is a row field precisely
//! so that a mixed set is a normal thing to be given, and a file with no `j`
//! column at all is refused because `unknown` and an absent column are
//! different statements.
//!
//! Every file is a literal in this source. `docs/fixtures.md` gives the reason.

use spectro_contract::{Refusals, level_set, validate_level_set};

const HEADER: &str = "#contract_version\t1.0\n\
                      #energy_unit\tcm-1\n\
                      #energy_reference\tground_state\n\
                      #level_set_id\tfe2-2026-08-08\n";

const LABELS: &str = "level_id\tspecies\tenergy\tenergy_uncertainty\tuncertainty_kind\t\
                      uncertainty_class\tparity\tj\torigin\tconfiguration\tterm\n";

/// A measured level, a predicted level carrying a spread, and a level whose J
/// was never determined.
const THREE_KINDS: &str = "L1\tFe II\t0.0\t0.002\tstandard_deviation\t\teven\t4.5\tmeasured\t3d6.(5D).4s\ta6D\n\
                           L2\tFe II\t62171.625\t40\tspread\t\todd\t2.5\tpredicted\t3d6.(5D).4p\tz6F\n\
                           L3\tFe II\t384.7908\t0.004\tstandard_deviation\t\teven\tunknown\tmeasured\t3d6.(5D).4s\ta6D\n";

/// One file holding two ionisation stages, which the contract admits.
const TWO_STAGES: &str = "L1\tFe I\t0.0\t0.001\tstandard_deviation\t\teven\t4.0\tmeasured\t3d6.4s2\ta5D\n\
                          L2\tFe II\t0.0\t0.002\tstandard_deviation\t\teven\t4.5\tmeasured\t3d6.(5D).4s\ta6D\n";

fn refused(result: Result<(), Refusals>) -> Refusals {
    match result {
        Ok(()) => panic!("the reader accepted a file this test built to be refused"),
        Err(refusals) => refusals,
    }
}

// -------------------------------------------- the five files issue #20 names

#[test]
fn a_measured_level_a_predicted_one_with_a_spread_and_one_with_an_unknown_j() {
    let file = format!("{HEADER}{LABELS}{THREE_KINDS}");
    let levels = level_set::read(file.as_bytes()).expect("all three are conforming");
    assert_eq!(levels.levels.len(), 3);

    let measured = &levels.levels[0];
    assert_eq!(measured.origin, "measured");
    assert_eq!(measured.uncertainty_kind, "standard_deviation");
    assert_eq!(measured.energy_uncertainty, Some(0.002));

    // A spread is not a standard deviation and the reader never prints one as
    // the other, so the kind travels beside the number rather than being
    // resolved into it here.
    let predicted = &levels.levels[1];
    assert_eq!(predicted.origin, "predicted");
    assert_eq!(predicted.uncertainty_kind, "spread");
    assert_eq!(predicted.energy_uncertainty, Some(40.0));

    let unknown_j = &levels.levels[2];
    assert_eq!(unknown_j.j, None);
    assert_eq!(unknown_j.parity, "even");
}

#[test]
fn a_file_mixing_two_ionisation_stages_is_read_and_the_stages_are_kept_apart() {
    let file = format!("{HEADER}{LABELS}{TWO_STAGES}");
    let levels =
        level_set::read(file.as_bytes()).expect("a mixed set is a normal thing to be given");
    assert_eq!(levels.levels[0].species, "Fe I");
    assert_eq!(levels.levels[1].species, "Fe II");
    assert_ne!(levels.levels[0].species, levels.levels[1].species);
}

#[test]
fn a_file_with_no_j_column_at_all_is_refused_and_produces_no_level() {
    let file = format!(
        "{HEADER}\
         level_id\tspecies\tenergy\tenergy_uncertainty\tuncertainty_kind\t\
         uncertainty_class\tparity\torigin\tconfiguration\tterm\n\
         L1\tFe II\t0.0\t0.002\tstandard_deviation\t\teven\tmeasured\t3d6.(5D).4s\ta6D\n"
    );
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.field.as_deref() == Some("j")),
        "{refusals}"
    );
    assert!(level_set::read(file.as_bytes()).is_err());
}

// ----------------------------------- where a refusal points, header against row

#[test]
fn a_header_refusal_names_the_header_field_and_the_file_and_a_row_refusal_names_the_line() {
    let header_fault = format!(
        "#contract_version\t1.0\n\
         #energy_reference\tground_state\n\
         #level_set_id\tfe2-2026-08-08\n\
         {LABELS}{THREE_KINDS}"
    );
    let refusals = refused(validate_level_set(header_fault.as_bytes()));
    let unit: Vec<_> = refusals
        .iter()
        .filter(|refusal| refusal.field.as_deref() == Some("energy_unit"))
        .collect();
    assert_eq!(unit.len(), 1, "{refusals}");
    assert_eq!(
        unit[0].line, None,
        "a header refusal names the file, not a line"
    );
    assert_eq!(
        unit[0].to_string(),
        "field `energy_unit`: the file carries no such header field"
    );

    let row_fault = format!("{HEADER}{LABELS}{THREE_KINDS}").replace("\t4.5\t", "\t4.3\t");
    let refusals = refused(validate_level_set(row_fault.as_bytes()));
    assert_eq!(
        refusals
            .iter()
            .filter(|refusal| refusal.line == Some(6) && refusal.field.as_deref() == Some("j"))
            .count(),
        1,
        "{refusals}"
    );
}

#[test]
fn an_energy_with_no_unit_refuses_the_whole_file_rather_than_one_row() {
    let file = format!("{HEADER}{LABELS}{THREE_KINDS}").replace("#energy_unit\tcm-1\n", "");
    let refusals = refused(validate_level_set(file.as_bytes()));
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.field.as_deref() == Some("energy_unit")),
        "{refusals}"
    );

    // Not one row short of a unit, but no level objects at all. A reader that
    // handed back the levels it managed to parse would be handing back energies
    // in no stated unit.
    assert!(level_set::read(file.as_bytes()).is_err());
}

// ------------------------------------------ configuration and term, verbatim

#[test]
fn configuration_and_term_arrive_byte_for_byte_as_they_were_given() {
    // The trailing space and the double space are the point of this file. They
    // are what a reader that tidies would remove, and a reader that tidies
    // produces a file nobody can afterwards tell from one that did not.
    let odd = "L9\tFe II\t100.0\tnone\tstandard_deviation\t\teven\t0.5\tmeasured\t\
               3d6.(5D).4s  (2)\tz6F* \n";
    let file = format!("{HEADER}{LABELS}{odd}");
    let levels = level_set::read(file.as_bytes()).expect("conforming");
    assert_eq!(levels.levels[0].configuration, "3d6.(5D).4s  (2)");
    assert_eq!(levels.levels[0].term, "z6F* ");
    assert_eq!(levels.levels[0].term.as_bytes().last(), Some(&b' '));
}

#[test]
fn configuration_and_term_may_be_empty_and_their_columns_may_not_be_absent() {
    let empty = "L9\tFe II\t100.0\tnone\tstandard_deviation\t\teven\t0.5\tpredicted\t\t\n";
    let file = format!("{HEADER}{LABELS}{empty}");
    let levels = level_set::read(file.as_bytes()).expect("both may be empty");
    assert_eq!(levels.levels[0].configuration, "");
    assert_eq!(levels.levels[0].term, "");

    let absent = format!(
        "{HEADER}\
         level_id\tspecies\tenergy\tenergy_uncertainty\tuncertainty_kind\t\
         uncertainty_class\tparity\tj\torigin\tterm\n\
         L9\tFe II\t100.0\tnone\tstandard_deviation\t\teven\t0.5\tpredicted\t\n"
    );
    let refusals = refused(validate_level_set(absent.as_bytes()));
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.field.as_deref() == Some("configuration")),
        "{refusals}"
    );
}

// --------------------------------------- no default is ever substituted, and
// --------------------------------------- the search that refuses one

/// Every source file of the reader, by name, read at compile time.
///
/// `include_str!` rather than a walk of the directory, so the search is over
/// exactly the files this crate is built from and needs nothing from the
/// filesystem at run time.
const READER_SOURCE: &[(&str, &str)] = &[
    ("lib.rs", include_str!("../src/lib.rs")),
    ("document.rs", include_str!("../src/document.rs")),
    ("reading.rs", include_str!("../src/reading.rs")),
    ("refusal.rs", include_str!("../src/refusal.rs")),
    ("value.rs", include_str!("../src/value.rs")),
    ("level_set.rs", include_str!("../src/level_set.rs")),
    ("line_list.rs", include_str!("../src/line_list.rs")),
    ("rates.rs", include_str!("../src/rates.rs")),
    ("covariance.rs", include_str!("../src/covariance.rs")),
];

/// Issue #20 asks that no default is ever substituted for a missing unit, and
/// that it be greppable. The absence of a thing is not greppable, so this is
/// the positive form: the shapes that would put a default into the reader.
///
/// Two families. A unit written into the reader as a literal, which is the
/// direct form. And a fallback expression, which is the form that arrives by
/// accident, because a reader that has to construct a value before it knows
/// whether the file was readable will reach for one.
///
/// It reads the bytes of the source rather than its parsed identifiers, so a
/// comment mentioning one of these would trip it. That is the conservative
/// direction and it is the same bound `docs/decisions/layout.md` states for its
/// own word list.
const NO_DEFAULT: &[&str] = &[
    "unwrap_or",
    ".unwrap()",
    ".expect(",
    "\"cm-1\"",
    "\"nm\"",
    "\"angstrom\"",
    "\"eV\"",
    "\"s-1\"",
    "\"Hz\"",
];

#[test]
fn no_default_is_ever_substituted_for_a_missing_unit() {
    let mut found: Vec<String> = Vec::new();
    for (name, source) in READER_SOURCE {
        for shape in NO_DEFAULT {
            if source.contains(shape) {
                found.push(format!("{name} carries `{shape}`"));
            }
        }
    }
    assert!(
        found.is_empty(),
        "the reader may state no unit of its own and may fall back to no value: {}",
        found.join("; ")
    );
}

/// The search above is only worth what it catches, so this runs it against a
/// source that carries the defect and requires it to fire. Without this, a
/// search over an empty word list would pass identically.
#[test]
fn the_search_fires_on_a_reader_that_carries_a_default_unit() {
    let defective = "let energy_unit = header(\"energy_unit\").unwrap_or(\"cm-1\");";
    let caught: Vec<_> = NO_DEFAULT
        .iter()
        .filter(|shape| defective.contains(*shape))
        .collect();
    assert_eq!(caught.len(), 2, "{caught:?}");
}
