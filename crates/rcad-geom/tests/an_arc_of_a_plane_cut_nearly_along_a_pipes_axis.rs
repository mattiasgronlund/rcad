//! A short arc of the ellipse a plane cuts a pipe in, where the plane is a
//! rounding off parallel to the pipe's axis.
//!
//! The plane and the two ends are the ones a fuse built in
//! `two_bars_on_one_pipe_and_a_box_united_in_one_call.rs`: the inner wall of a
//! square bar standing on an edge along `y`, stored as `-y - 2⁻⁵² z = -1.5 - 2⁻⁵²`,
//! against a round bar of radius `2` up `z`. So the plane leans `2⁻⁵²` off the
//! axis, the ellipse runs from about `z = -2.25e15` to `z = 1.58e16`, and the arc
//! between the two ends is `1.23` long.
//!
//! Walked round the ellipse, the arc's middle came out from a centre `6.75e15`
//! up the axis, and the half-space that picks the arc could not tell the two
//! arcs apart: `CannotNameWhichPieceOfACrossingThisIs` (Q1882). Every point
//! here is checked against **both** surfaces, which say exactly where the
//! crossing is.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, NamedRun, Picker, Plane, Point, Surface,
    Vector, walk,
};
use rcad_predicates::{Policy, ProvenanceLog};
use smallvec::smallvec;

/// The round bar: radius `2` about `z`.
fn the_pipe() -> Cylinder {
    Cylinder::new(Axis::z(), 2.0).expect("a radius above zero")
}

/// The square bar's inner wall, with the bits the fuse handed the walk.
fn the_wall() -> Plane {
    Plane::from_coefficients([0.0, -1.0, -f64::EPSILON, 1.500_000_000_000_000_2]).expect("a plane")
}

/// The wall above is the plane the fuse walked, bit for bit.
#[test]
fn the_wall_is_the_plane_the_fuse_walked() {
    let wall = the_wall();
    assert_eq!(
        (wall.normal().components(), wall.offset()),
        (
            [0.0, -1.0, -2.220_446_049_250_313e-16],
            -1.500_000_000_000_000_2
        )
    );
}

/// Where the wall, the pipe and the box's two lids meet, as the fuse placed
/// them: an ulp apart across, `1.23` apart up the pipe.
fn the_ends() -> [Point; 2] {
    [
        Point::new(1.322_875_655_532_295, 1.5, 0.384_231_125_654_252_3),
        Point::new(1.322_875_655_532_295_4, 1.5, 1.615_768_874_345_747_7),
    ]
}

/// The arc between the two ends that stays on the `+x` side of the pipe,
/// picked by the plane through its chord square to the wall. The arc's middle
/// is within a rounding of that plane, and the other arc runs round to `x = -2`.
fn the_arc() -> NamedRun {
    let on_the_ends_side =
        HalfSpace::new(Vector::new(1.0, 0.0, 0.0), 1.322_875_655_532_295).expect("a half-space");
    NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(the_wall()),
            picked_by: smallvec![Picker::InFrontOf(on_the_ends_side)],
        },
        the_ends().map(|end| rcad_predicates::Point::from(end.coordinates())),
    )
    .expect("a run with two ends")
}

fn walked(deflection: f64, along: [&Surface; 2]) -> rcad_geom::PointsAlongACrossing {
    walk(
        &the_arc(),
        along,
        Deflection::of(deflection).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("the arc is walked")
}

/// Every point is on the wall and on the pipe inside the deviation the walk
/// publishes, on the ends' side of the pipe, and between the ends in the order
/// the run is stored; and that deviation is a rounding of the arc's own size
/// and not of the ellipse's.
fn every_point_is_on_both_surfaces_between_the_ends(deflection: f64, along: [&Surface; 2]) {
    let walked = walked(deflection, along);
    let deviation = walked.deviation().magnitude();
    assert!(
        deviation < 1e-12,
        "charged {deviation:e}, which is a rounding of the ellipse and not the arc"
    );
    assert!(
        !walked.points().is_empty(),
        "a curved arc is walked into at least one point between its ends"
    );
    let [first, last] = the_ends().map(|end| end.coordinates()[2]);
    let mut below = first;
    for point in walked.points() {
        let [x, y, z] = point.coordinates();
        let off_the_wall = the_wall().signed_distance_to(*point).abs();
        let off_the_pipe = (x.hypot(y) - the_pipe().radius()).abs();
        assert!(
            off_the_wall <= deviation && off_the_pipe <= deviation,
            "{point:?} is {off_the_wall:e} off the wall and {off_the_pipe:e} off the pipe, \
             against a deviation of {deviation:e}"
        );
        assert!(x > 0.0, "{point:?} is on the other arc, round the far side");
        assert!(
            below < z && z < last,
            "{point:?} is not between {below} and {last} up the pipe"
        );
        below = z;
    }
}

#[test]
fn walked_coarsely_the_arc_is_on_the_wall_and_on_the_pipe() {
    let [pipe, wall] = [Surface::Cylinder(the_pipe()), Surface::Plane(the_wall())];
    every_point_is_on_both_surfaces_between_the_ends(1.0, [&pipe, &wall]);
    every_point_is_on_both_surfaces_between_the_ends(1.0, [&wall, &pipe]);
}

#[test]
fn walked_finely_the_arc_is_on_the_wall_and_on_the_pipe() {
    let [pipe, wall] = [Surface::Cylinder(the_pipe()), Surface::Plane(the_wall())];
    every_point_is_on_both_surfaces_between_the_ends(1e-3, [&pipe, &wall]);
    every_point_is_on_both_surfaces_between_the_ends(1e-3, [&wall, &pipe]);
}

/// The arc's middle is within a rounding of the half-space that picks it, so
/// asking there is never decided, and under a certified policy a record of it
/// makes the fuse that asked refuse `Undecidable` (Q1882). The far arc's point
/// across the pipe is plainly behind it, which is all the walk needs to know.
#[test]
fn walked_under_a_certified_policy_the_arc_logs_nothing() {
    let [pipe, wall] = [Surface::Cylinder(the_pipe()), Surface::Plane(the_wall())];
    for along in [[&pipe, &wall], [&wall, &pipe]] {
        let mut log = ProvenanceLog::new();
        walk(
            &the_arc(),
            along,
            Deflection::of(1e-3).expect("a positive length"),
            &Policy::certified(),
            &mut log,
        )
        .expect("the arc is walked");
        assert!(
            log.is_empty(),
            "a certified walk logged {:?}",
            log.records()
        );
    }
}
