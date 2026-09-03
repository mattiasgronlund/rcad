//! §42.12 criterion 3: a file written by hand reads in and measures what its
//! numbers say.
//!
//! The closed form is the check. A block of sides `a`, `b`, `c` encloses `abc`,
//! its skin is `2(ab + bc + ca)` and its middle is at half of each side — none
//! of which is read off this kernel, so a mapping that turned the file into some
//! *other* sound solid would fail here rather than agree with itself.

mod written_by_hand;

use rcad_geom::{Deflection, Unit};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::{a_block, a_quarter_disc_plate};

/// The measurements of one block, read out of a file this test wrote.
fn measure(text: &str) -> (f64, f64, [f64; 3]) {
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &Policy::certified(), &mut log).expect("a block imports");
    assert!(
        log.is_empty(),
        "a block of exact numbers needs nothing approximated: {log:?}"
    );
    import
        .model()
        .validate(&Policy::certified(), &mut log)
        .expect("and the model it made is sound");

    let [(_, key)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let solid = import.model().solid(*key);
    let measured = solid
        .measure(
            Deflection::of(0.01).expect("a tenth of the smallest side"),
            &Policy::certified(),
            &mut log,
        )
        .expect("a block measures");
    (
        measured.volume(),
        measured.surface_area(),
        measured.centroid().coordinates(),
    )
}

#[test]
fn a_block_written_by_hand_in_a_step_file_measures_what_its_numbers_say() {
    let (volume, area, centroid) = measure(&a_block([2.0, 4.0, 8.0], ".MILLI."));

    assert_eq!(volume, 2.0 * 4.0 * 8.0);
    assert_eq!(area, 2.0 * (2.0 * 4.0 + 4.0 * 8.0 + 8.0 * 2.0));
    assert_eq!(centroid, [1.0, 2.0, 4.0]);
}

#[test]
fn a_file_written_in_inches_comes_back_in_inches() {
    let inches = a_block([1.0, 1.0, 1.0], ".MILLI.").replace(
        "#1=(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.MILLI.,.METRE.));",
        "#6=(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.MILLI.,.METRE.));\n\
         #7=LENGTH_MEASURE_WITH_UNIT(LENGTH_MEASURE(25.4),#6);\n\
         #1=(CONVERSION_BASED_UNIT('INCH',#7)LENGTH_UNIT()NAMED_UNIT(*));",
    );
    let file = StepFile::read(inches.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    let import =
        Import::read(&file, &Policy::certified(), &mut log).expect("an inch block imports");

    assert_eq!(import.declared_unit(), Unit::Inch);
    assert_eq!(import.model().scale().unit(), Unit::Inch);
    let [(_, key)] = import.solids() else {
        panic!("one solid");
    };
    let solid = import.model().solid(*key);
    let measured = solid
        .measure(
            Deflection::of(0.01).expect("a hundredth of the side"),
            &Policy::certified(),
            &mut log,
        )
        .expect("a block measures");
    assert_eq!(
        measured.volume(),
        1.0,
        "one cubic inch, and not 16.387 of anything: nothing was converted"
    );
}

#[test]
fn a_unit_the_kernel_does_not_have_is_refused_rather_than_scaled() {
    let nanometres =
        a_block([1.0, 1.0, 1.0], ".MILLI.").replace(".MILLI.,.METRE.", ".NANO.,.METRE.");
    let file = StepFile::read(nanometres.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();

    let why = Import::read(&file, &Policy::certified(), &mut log)
        .expect_err("this kernel has no nanometre");
    assert!(
        matches!(
            why,
            rcad_step::Refusal::ALengthUnitThisKernelDoesNotHave { .. }
        ),
        "expected a named refusal, got {why:?}"
    );
}

#[test]
fn a_file_declaring_more_precision_than_its_numbers_carry_says_so() {
    let file = StepFile::read(a_block([2.0, 4.0, 8.0], ".MILLI.").as_bytes()).expect("reads");
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &Policy::certified(), &mut log).expect("imports");

    assert_eq!(import.declared_uncertainty(), Some(1e-7));
    assert!(
        !import.asks_for_more_precision_than_its_numbers_carry(),
        "a tenth of a micron on an eight millimetre block is a promise f64 keeps"
    );

    // The same block, asking for a precision a hundred times finer than the
    // spacing between the `f64` its own largest coordinate sits on.
    let too_fine = a_block([2.0, 4.0, 8.0], ".MILLI.").replace(
        "LENGTH_MEASURE(1.E-7)",
        &format!(
            "LENGTH_MEASURE({})",
            written_by_hand::real(8.0 * f64::EPSILON / 100.0)
        ),
    );
    let file = StepFile::read(too_fine.as_bytes()).expect("reads");
    let import = Import::read(&file, &Policy::certified(), &mut log).expect("imports");
    assert!(
        import.asks_for_more_precision_than_its_numbers_carry(),
        "an uncertainty below the coordinate spacing is a promise the numbers do not keep"
    );
}

/// The curved half of the hand-written control: a plate whose top and bottom
/// are quarter-discs.
///
/// A quarter-disc of radius `r` and height `h` encloses `πr²h/4` and its skin is
/// `πr²/2 + 2rh + πrh/2`. Those are the numbers this checks against, and the
/// measurement of a curved solid is over a mesh, so this is the one place in
/// this file where a comparison is a bound rather than an equality — and the
/// bound is the one the measurement publishes for itself.
#[test]
fn a_quarter_disc_plate_written_by_hand_measures_what_its_numbers_say() {
    let (radius, height) = (2.0, 4.0);
    let file = StepFile::read(a_quarter_disc_plate(radius, height, ".MILLI.").as_bytes())
        .expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &Policy::certified(), &mut log).expect("a plate imports");
    import
        .model()
        .validate(&Policy::certified(), &mut log)
        .expect("and the model it made is sound");

    let [(_, key)] = import.solids() else {
        panic!("one solid");
    };
    let measured = import
        .model()
        .solid(*key)
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut log,
        )
        .expect("a plate measures");

    let space = core::f64::consts::PI * radius * radius * height / 4.0;
    let skin = core::f64::consts::PI * radius * radius / 2.0
        + 2.0 * radius * height
        + core::f64::consts::PI * radius * height / 2.0;
    assert!(
        (measured.volume() - space).abs() <= measured.volume_error(),
        "a quarter-disc plate encloses {space} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
    assert!(
        (measured.surface_area() - skin).abs() <= measured.area_error(),
        "a quarter-disc plate has {skin} of skin and this one says {} give or take {}",
        measured.surface_area(),
        measured.area_error()
    );
}
