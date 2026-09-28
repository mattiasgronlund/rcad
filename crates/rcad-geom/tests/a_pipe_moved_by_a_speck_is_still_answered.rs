//! A pipe moved `2⁻¹⁰⁰⁷` off the origin, asked the two questions a fuse asked
//! it and got no answer to (`docs/decisions.md` §246.6, §322).
//!
//! Both answers exist and neither is close to a knife edge once it is written
//! down: the room a plane two from the axis leaves is `2⁻¹⁰⁰⁵`, and the
//! discriminant of the line `x = 1, z = 0` against the pipe is `3`. What stopped
//! both was one product: `2⁻¹⁰⁰⁷` squared is `2⁻²⁰¹⁴`, below the smallest
//! subnormal, so the exact level refused the whole sum it sat in — and the
//! interval level had rounded the move away before it squared anything.
//!
//! **A gap and not a limit**, and the file's two tests are the demonstration
//! that a representable question reaches each answer. The family that reaches
//! them is an expansion that keeps a product that fell off the bottom and
//! carries a box round what it lost; the scaling family does not, because the
//! reading `2⁻¹⁰⁰⁷ − 2` spans 1008 binades and no slide squares more than 995
//! (`a_reading_no_slide_can_square_is_still_decided` in `exact.rs`).
//!
//! The controls are the same questions about the pipe that has not moved. They
//! answered before the fix and after it.

use rcad_geom::{
    Axis, Cylinder, MeetOfThree, Plane, Point, Surface, SurfaceMeet, Vector, intersect,
    meet_of_three_surfaces,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// `2⁻¹⁰⁰⁷`, the move §246.1 found in a fuzzer seed: its top byte is the one the
/// fuzzer changed. Built from its bits because `powi` with a run-time exponent
/// is not to be trusted down here (§246.3).
const SPECK: f64 = f64::from_bits(0x0100_0000_0000_0000);

const RADIUS: f64 = 2.0;

/// A round bar's wall, radius two, up `z`, moved `across` along `y`.
fn pipe_moved_by(across: f64) -> Surface {
    let axis = Axis::new(Point::new(0.0, across, 0.0), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a direction");
    Surface::Cylinder(Cylinder::new(axis, RADIUS).expect("two is a radius"))
}

fn flat(through: [f64; 3], normal: [f64; 3]) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(
            Point::new(through[0], through[1], through[2]),
            Vector::new(normal[0], normal[1], normal[2]),
        )
        .expect("an axis-aligned normal"),
    )
}

/// The plane `y = 2`: a cube's wall, lying flat against the unmoved pipe.
fn the_wall_the_pipe_leans_on() -> Surface {
    flat([0.0, 2.0, 0.0], [0.0, 1.0, 0.0])
}

#[test]
fn the_speck_is_the_number_the_fuzzer_found() {
    assert_eq!(SPECK, 7.291122019556398e-304);
    assert_eq!(
        SPECK * SPECK,
        0.0,
        "its square is below the smallest subnormal"
    );
}

/// Moved `2⁻¹⁰⁰⁷` towards the plane, the pipe pokes through it by that much, so
/// the plane cuts it in two lines about `2⁻⁵⁰²` apart. This refused as
/// "where a plane and a pipe cross" with an interval enclosure of `[0, 8.9e-16]`.
#[test]
fn a_plane_a_speck_inside_a_pipes_wall_cuts_it_in_two_lines() {
    let mut log = ProvenanceLog::new();
    let meet = intersect(
        &the_wall_the_pipe_leans_on(),
        &pipe_moved_by(SPECK),
        &Policy::certified(),
        &mut log,
    );

    let Ok(SurfaceMeet::Curves(runs)) = &meet else {
        panic!("expected two lines, got {meet:?}");
    };
    assert_eq!(
        runs.len(),
        2,
        "two lines, one either side of the touch: {runs:?}"
    );
    assert!(
        log.is_empty(),
        "a certified answer writes nothing down: {log:?}"
    );
}

/// The control: unmoved, the plane touches the pipe along one line.
#[test]
fn a_plane_flat_against_a_pipe_that_has_not_moved_touches_it_along_one_line() {
    let meet = intersect(
        &the_wall_the_pipe_leans_on(),
        &pipe_moved_by(0.0),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    let Ok(SurfaceMeet::Curves(runs)) = &meet else {
        panic!("expected one line, got {meet:?}");
    };
    assert_eq!(runs.len(), 1, "{runs:?}");
}

/// The line `x = 1, z = 0` — a cube's wall across a bar's bottom cap — meets
/// the moved pipe where `(y − 2⁻¹⁰⁰⁷)² = 3`. The quadratic's constant term
/// carries `2⁻²⁰¹⁴`, which cancels out of the discriminant, and this refused as
/// "where three surfaces meet" at the exact level with an enclosure of
/// infinity.
#[test]
fn a_line_across_the_moved_pipe_meets_it_either_side_of_its_axis() {
    let meet = meet_of_three_surfaces(
        [
            &flat([1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]),
            &flat([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
            &pipe_moved_by(SPECK),
        ],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    let Ok(MeetOfThree::At(corners)) = &meet else {
        panic!("expected two corners, got {meet:?}");
    };
    assert_eq!(corners.len(), 2, "{corners:?}");
    let root_three = 3f64.sqrt();
    let mut heights: Vec<f64> = corners
        .iter()
        .map(|corner| {
            let [x, y, z] = corner.enclosure();
            assert!(x.lo() <= 1.0 && 1.0 <= x.hi(), "{corner:?} is off x = 1");
            assert!(z.lo() <= 0.0 && 0.0 <= z.hi(), "{corner:?} is off z = 0");
            assert!(y.width() < 1e-14, "{corner:?} is not boxed tight");
            y.lo()
        })
        .collect();
    heights.sort_by(f64::total_cmp);
    assert!((heights[0] + root_three).abs() < 1e-14, "{heights:?}");
    assert!((heights[1] - root_three).abs() < 1e-14, "{heights:?}");
}

/// The control: the same three surfaces with the pipe where it was built.
#[test]
fn a_line_across_the_pipe_that_has_not_moved_meets_it_twice() {
    let meet = meet_of_three_surfaces(
        [
            &flat([1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]),
            &flat([0.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
            &pipe_moved_by(0.0),
        ],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    assert!(
        matches!(&meet, Ok(MeetOfThree::At(corners)) if corners.len() == 2),
        "{meet:?}"
    );
}
