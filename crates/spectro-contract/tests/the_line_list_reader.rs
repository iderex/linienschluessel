//! The line list reader of issue #21.
//!
//! Issue #18's suite holds what the validator refuses and issue #20's holds
//! what the level reader produces. This one holds the files issue #21 names: a
//! line below 200 nm, one between 200 nm and 2000 nm, one above 2000 nm, a line
//! with no uncertainty, and a file holding two spectra.
//!
//! The three regions are not three sizes of number. `docs/decisions/line-position.md`
//! forbids deciding a medium from the magnitude of a position, because a number
//! between 200 and 2000 is an air wavelength in nm on one source's convention, a
//! vacuum wavelength on another's and a wavenumber on a third's, and the ranges
//! overlap. A reader that picked by magnitude would be right most of the time,
//! which is what makes it dangerous. So the regions are here to be crossed by a
//! declared medium that a magnitude rule would have got wrong, which is the
//! second test below.
//!
//! Two spectra in one file are not decoration either. The uncertainty model
//! fits one offset term per spectrum, so two spectra in one file have to stay
//! distinguishable after the read or that term cannot be fitted at all.
//!
//! Every file is a literal in this source. `docs/fixtures.md` gives the reason.

use spectro_contract::{Refusals, line_list, validate_line_list};

const HEADER: &str = "#contract_version\t1.0\n\
                      #position_unit\tnm\n\
                      #position_medium\tvacuum\n\
                      #line_list_id\tarc-2026-08-09\n";

const LABELS: &str = "feature_id\tspectrum_id\tsegment_id\tposition\tposition_medium\t\
                      position_uncertainty\tuncertainty_kind\tuncertainty_class\t\
                      intensity\tintensity_scale\tflags\tritz_position\n";

/// Three regions, two spectra, and a line whose uncertainty the source never
/// gave. `F4` sits beside `F2` in position and is declared in the other medium.
const FOUR_LINES: &str = "F1\tS1\tvuv\t185.9\tvacuum\t0.0009\tstandard_deviation\t\t40\tphotographic\t\t\n\
                          F2\tS1\tord3\t259.9396\tair_standard\t0.0004\tstandard_deviation\t\t120\tphotographic\t\t\n\
                          F3\tS1\tir1\t2445.7\tvacuum\tnone\tclass\tB\t\t\t\t\n\
                          F4\tS2\tord3\t259.9401\tvacuum\t0.0006\tstandard_deviation\t\t95\tphotographic\t\t\n";

fn refused(result: Result<(), Refusals>) -> Refusals {
    match result {
        Ok(()) => panic!("the reader accepted a file this test built to be refused"),
        Err(refusals) => refusals,
    }
}

/// Whether some refusal names this line and this field.
fn names(refusals: &Refusals, line: Option<usize>, field: &str) -> bool {
    refusals
        .iter()
        .any(|refusal| refusal.line == line && refusal.field.as_deref() == Some(field))
}

// ------------------------------------------- the files issue #21 names

#[test]
fn three_regions_two_spectra_and_a_line_whose_uncertainty_was_never_given() {
    let file = format!("{HEADER}{LABELS}{FOUR_LINES}");
    let lines = line_list::read(file.as_bytes()).expect("all four are conforming");
    assert_eq!(lines.lines.len(), 4);

    // Below 200 nm, between 200 and 2000, and above 2000. The reader carries
    // the number it was given; nothing here has converted anything.
    assert_eq!(lines.lines[0].position, 185.9);
    assert_eq!(lines.lines[1].position, 259.9396);
    assert_eq!(lines.lines[2].position, 2445.7);

    // An uncertainty the source never gave is an absence rather than a zero,
    // and the kind that stands in its place travels with it.
    assert_eq!(lines.lines[2].position_uncertainty, None);
    assert_eq!(lines.lines[2].uncertainty_kind, "class");
    assert_eq!(lines.lines[2].uncertainty_class.as_deref(), Some("B"));
    assert_eq!(lines.lines[0].position_uncertainty, Some(0.0009));

    // Two spectra in one file stay two spectra.
    assert_eq!(lines.lines[1].spectrum_id, "S1");
    assert_eq!(lines.lines[3].spectrum_id, "S2");
    assert_ne!(lines.lines[1].spectrum_id, lines.lines[3].spectrum_id);
}

#[test]
fn the_medium_is_read_from_the_file_and_never_from_the_magnitude() {
    let file = format!("{HEADER}{LABELS}{FOUR_LINES}");
    let lines = line_list::read(file.as_bytes()).expect("all four are conforming");

    // `F2` and `F4` are five ten-thousandths of a nanometre apart and are
    // declared in different media. Any rule that assigned a medium from the
    // position would have to give these two the same one.
    assert!((lines.lines[1].position - lines.lines[3].position).abs() < 0.001);
    assert_eq!(lines.lines[1].position_medium, "air_standard");
    assert_eq!(lines.lines[3].position_medium, "vacuum");

    // And the two ends of the file are not decided by which side of a boundary
    // they fall on either.
    assert_eq!(lines.lines[0].position_medium, "vacuum");
    assert_eq!(lines.lines[2].position_medium, "vacuum");
}

// ----------------------------------------- a position with no declared medium

#[test]
fn a_file_with_no_medium_header_is_refused_naming_the_field() {
    let file = format!(
        "#contract_version\t1.0\n\
         #position_unit\tnm\n\
         #line_list_id\tarc-2026-08-09\n\
         {LABELS}{FOUR_LINES}"
    );
    let refusals = refused(validate_line_list(file.as_bytes()));

    // No line, because a header field the file never carried is a fact about
    // the whole file rather than about one row.
    assert!(names(&refusals, None, "position_medium"), "{refusals}");
    assert!(line_list::read(file.as_bytes()).is_err());
}

#[test]
fn a_row_stating_no_medium_with_no_header_default_is_refused_naming_the_line() {
    let file = format!(
        "#contract_version\t1.0\n\
         #position_unit\tnm\n\
         #line_list_id\tarc-2026-08-09\n\
         {LABELS}\
         F1\tS1\tvuv\t185.9\t\t0.0009\tstandard_deviation\t\t40\tphotographic\t\t\n"
    );
    let refusals = refused(validate_line_list(file.as_bytes()));

    // The label row is line 4, so the only body row is line 5. Both statements
    // are made: the file carries no default, and this row has no medium of its
    // own to fall back from.
    assert!(names(&refusals, None, "position_medium"), "{refusals}");
    assert!(names(&refusals, Some(5), "position_medium"), "{refusals}");
}
