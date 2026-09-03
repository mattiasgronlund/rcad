//! The file that named the consumer of `validate.rs`'s winding gap, and now
//! imports.
//!
//! `winding_of` used to read which way a face goes off its **corners** alone, so
//! a face whose area is carried by its arcs swept zero and was refused. That was
//! written down as a gap rather than a limit and then deferred, because "no file
//! in the corpus is refused for it, so it waits for one that is".
//!
//! This is one that was. [`a_plate_bounded_by_two_half_circles`] is a whole
//! AP203 `manifold_solid_brep` describing a sound solid — a half-round plate
//! with a half-round rib under it — whose flat top and bottom are each bounded
//! by two half-circles and two straights. All four of their corners sit on the
//! `x` axis, so the polygon through them sweeps exactly nothing.
//!
//! The STEP mapping always took every part of it: `Walk::area_swept_by` adds
//! each arc's bulge past its own chord for exactly this shape, which is why it
//! can pick an outer bound out of a face whose corners are no polygon at all.
//! What refused the file was the winding rule in `rcad-topo`'s `validate.rs`,
//! reached through `SolidBuilder::build`, with
//! `Invalid::OuterWireWindsAgainstTheNormal`. That rule now winds the boundary
//! the wire traces — its corners with the points `rcad_geom::walk` puts along
//! each arc — so the two windings in one pipeline agree about this face.
//!
//! The corpus does not name this consumer and was asked: no KiCad or NIST file
//! is refused for a winding — they stop earlier, on B-splines and on corners off
//! their curved faces. A file had to be written for it, and this is it.

mod written_by_hand;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::a_plate_bounded_by_two_half_circles;

const OUTER: f64 = 3.0;
const INNER: f64 = 1.0;
const HEIGHT: f64 = 5.0;

/// The file imports, the model it brings sweeps clean, and the solid encloses
/// what its two half-circles say.
///
/// The volume is the part that says it passed for the right reason. A rule that
/// merely stopped refusing this face could still have it wound the wrong way
/// round; a solid wound the wrong way round does not enclose
/// `π(OUTER² + INNER²)·HEIGHT/2`.
#[test]
fn a_file_whose_area_is_all_in_its_arcs_imports_and_measures_what_it_says() {
    let text = a_plate_bounded_by_two_half_circles(OUTER, INNER, HEIGHT, ".MILLI.");
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();

    let import = Import::read(&file, &policy, &mut log).expect("a plate bounded by two arcs reads");
    import
        .model()
        .validate(&policy, &mut log)
        .expect("and the full sweep is happy with what it brought");

    let [(_, plate)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let measured = import
        .model()
        .solid(*plate)
        .measure(
            rcad_geom::Deflection::of(0.001).expect("a fine mesh"),
            &policy,
            &mut log,
        )
        .expect("a plate measures");

    let space = core::f64::consts::PI * (OUTER * OUTER + INNER * INNER) / 2.0 * HEIGHT;
    assert!(
        (measured.volume() - space).abs() <= measured.volume_error(),
        "the plate encloses {space} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}
