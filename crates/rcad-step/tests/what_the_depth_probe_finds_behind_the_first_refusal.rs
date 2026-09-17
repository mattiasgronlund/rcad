//! `probe` keeps walking where `Import::read` stops, and the two must agree
//! about everything else.
//!
//! §45.2: the count of files one refusal stops **first** is an upper bound and
//! never a projection, because a file freed of its first blocker usually meets
//! a second right behind it. [`rcad_step::probe`] exists to measure what is
//! behind the first one, and these tests hold it to the two things that make
//! its numbers mean anything:
//!
//! - **it finds what `Import::read` cannot**, so a file broken two independent
//!   ways names both reasons where the reader names one — a probe that
//!   quietly stopped at the first would pass every count it publishes while
//!   measuring the thing it was built to replace;
//! - **it invents nothing**, so one break is one reason. A dropped entity
//!   leaves a hole — a wire missing a step, a face missing a bound — and
//!   asking the builder to assemble the remains reports *that hole* as a
//!   second reason the file does not have. That was live: it cost `an arc this
//!   reader could not name` every file where it was the only reason, until the
//!   walk stopped asking. Taking the rule back out puts `a wire does not
//!   close` straight back into the two tests below, which is how it is known
//!   to be load-bearing rather than argued to be.
//!
//! Every file here is [`a_block`], which imports — `a_block_imports` in
//! `what_the_mapping_refuses_and_by_what_name.rs` proves it — with one or two
//! lines cut out of it, so every reason counted below is a reason a break put
//! there.

mod written_by_hand;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile, probe};
use written_by_hand::a_block;

/// A block of sides two, four and eight, in millimetres.
fn a_working_block() -> String {
    a_block([2.0, 4.0, 8.0], ".MILLI.")
}

/// A face this kernel holds no surface for: one of the block's six planes,
/// renamed to a patch.
const A_BROKEN_SURFACE: (&str, &str) = (
    "#450=PLANE('',#460);",
    "#450=B_SPLINE_SURFACE_WITH_KNOTS('',#460);",
);

/// An edge this kernel holds no curve for: one of the block's twelve lines,
/// renamed to a spline.
const A_BROKEN_CURVE: (&str, &str) = (
    "#300=LINE('',#100,#320);",
    "#300=B_SPLINE_CURVE_WITH_KNOTS('',#100,#320);",
);

fn broken(text: &str, (from, to): (&str, &str)) -> String {
    let cut = text.replace(from, to);
    assert_ne!(
        cut, text,
        "the line this test breaks is `{from}`, and the fixture no longer has it"
    );
    cut
}

/// What `Import::read` made of a file, or the one refusal it stopped at.
fn read(text: &str) -> Result<Import, Refusal> {
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    Import::read(&file, &Policy::certified(), &mut log)
}

/// Every reason the probe found in a file, and every fatal one, as the names
/// the report groups them under.
fn probed(text: &str) -> (Vec<String>, Vec<String>) {
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    let found = probe(&file, &Policy::certified(), &mut log);
    (
        found.reasons.into_iter().collect(),
        found.fatal.into_iter().collect(),
    )
}

#[test]
fn a_file_that_imports_gives_the_probe_nothing_to_report() {
    let working = a_working_block();
    assert_eq!(
        read(&working).expect("a block imports").solids().len(),
        1,
        "the fixture has to import, or every count below is about a broken block"
    );

    let (reasons, fatal) = probed(&working);
    assert!(
        reasons.is_empty() && fatal.is_empty(),
        "a file that imports has no refusals to find: {reasons:?} {fatal:?}"
    );
}

#[test]
fn two_faces_broken_two_different_ways_name_both_reasons_where_the_reader_names_one() {
    let twice_broken = broken(
        &broken(&a_working_block(), A_BROKEN_SURFACE),
        A_BROKEN_CURVE,
    );

    let stopped_at = read(&twice_broken).expect_err("a block broken twice does not import");
    let one_reason = matches!(
        stopped_at,
        Refusal::ASurfaceThisKernelDoesNotHold { .. } | Refusal::ACurveThisKernelDoesNotHold { .. }
    );
    assert!(
        one_reason,
        "the reader stops at one of the two breaks, and it reported {stopped_at}"
    );

    let (reasons, fatal) = probed(&twice_broken);
    assert_eq!(
        reasons,
        [
            "a curve this kernel does not hold: B_SPLINE_CURVE_WITH_KNOTS",
            "a surface this kernel does not hold: B_SPLINE_SURFACE_WITH_KNOTS",
        ],
        "both breaks are named, and nothing else is"
    );
    assert!(fatal.is_empty(), "neither break stops the walk: {fatal:?}");
}

#[test]
fn one_broken_edge_is_one_reason_and_never_a_wire_that_does_not_close_as_well() {
    let once_broken = broken(&a_working_block(), A_BROKEN_CURVE);

    let (reasons, fatal) = probed(&once_broken);
    assert_eq!(
        reasons,
        ["a curve this kernel does not hold: B_SPLINE_CURVE_WITH_KNOTS"],
        "the edge the file cannot name is the whole of what is wrong with this file"
    );
    assert!(
        fatal.is_empty(),
        "a named edge does not stop the walk: {fatal:?}"
    );
}

#[test]
fn one_broken_surface_is_one_reason_and_nothing_the_dropped_face_left_behind() {
    let once_broken = broken(&a_working_block(), A_BROKEN_SURFACE);

    let (reasons, fatal) = probed(&once_broken);
    assert_eq!(
        reasons,
        ["a surface this kernel does not hold: B_SPLINE_SURFACE_WITH_KNOTS"],
        "the face the kernel cannot hold is the whole of what is wrong with this file"
    );
    assert!(
        fatal.is_empty(),
        "a named surface does not stop the walk: {fatal:?}"
    );
}

#[test]
fn a_file_stopped_before_it_has_a_face_list_reports_one_fatal_reason_and_no_others() {
    let two_units = broken(
        &a_working_block(),
        (
            "#501=MANIFOLD_SOLID_BREP('',#500);",
            "#900=(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT($,.METRE.));\n\
             #901=(GEOMETRIC_REPRESENTATION_CONTEXT(3)\
             GLOBAL_UNIT_ASSIGNED_CONTEXT((#900))REPRESENTATION_CONTEXT('',''));\n\
             #501=MANIFOLD_SOLID_BREP('',#500);",
        ),
    );
    assert!(
        read(&two_units).is_err(),
        "two length units is a refusal, and the fixture has to reach it"
    );

    let (reasons, fatal) = probed(&two_units);
    assert_eq!(
        fatal,
        ["the file declares two different length units"],
        "a gate before the solids is fatal, because nothing was skipped past to reach it"
    );
    assert!(
        reasons.is_empty(),
        "a walk that never started skipped nothing: {reasons:?}"
    );
}
