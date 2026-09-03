//! The file that names the consumer of `validate.rs`'s winding gap.
//!
//! `winding_of` reads which way a face goes off its **corners** alone, and says
//! so: a face whose area is carried by its arcs sweeps zero and is refused. That
//! was written down as a gap rather than a limit and then deferred, because
//! "no file in the corpus is refused for it, so it waits for one that is".
//!
//! This is one that is. [`a_plate_bounded_by_two_half_circles`] is a whole
//! AP203 `manifold_solid_brep` describing a sound solid — a half-round plate
//! with a half-round rib under it — whose flat top and bottom are each bounded
//! by two half-circles and two straights. All four of their corners sit on the
//! `x` axis, so the polygon through them sweeps exactly nothing.
//!
//! The STEP mapping takes every part of it: `Walk::area_swept_by` adds each
//! arc's bulge past its own chord for exactly this shape, which is why it can
//! pick an outer bound out of a face whose corners are no polygon at all. What
//! refuses the file is the winding rule in `rcad-topo`'s `validate.rs`, reached
//! through `SolidBuilder::build`.
//!
//! The corpus does not name this consumer and was asked: no KiCad or NIST file
//! is refused for a winding — they stop earlier, on B-splines and on corners off
//! their curved faces. A file had to be written for it, and this is it.

mod written_by_hand;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use written_by_hand::a_plate_bounded_by_two_half_circles;

/// What today does with it, so the evidence keeps rather than being argued.
///
/// The two windings in one pipeline disagree about one face: the mapping's own
/// sweep adds the arcs' bulge and the kernel's reads the corners, and the
/// corners of this one are four points on a line.
#[test]
fn a_file_whose_area_is_all_in_its_arcs_is_refused_for_winding_nothing() {
    let text = a_plate_bounded_by_two_half_circles(3.0, 1.0, 5.0, ".MILLI.");
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();

    let refused = Import::read(&file, &Policy::certified(), &mut log);

    let Err(Refusal::TheSolidTheFileDescribesIsNotOne { why, .. }) = refused else {
        panic!("got {refused:?}");
    };
    assert!(
        matches!(
            *why,
            rcad_topo::Error::Invalid(rcad_topo::Invalid::OuterWireWindsAgainstTheNormal)
        ),
        "got {why:?}"
    );
}
