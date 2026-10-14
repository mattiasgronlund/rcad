//! The floor's rim of a bar sliced a hair inside its circle, and which of the
//! two arcs between the slice's ends a plane through both ends picks (Q1465,
//! from Q1460).
//!
//! The bar is `rcad-step`'s `a_bar_sliced_a_hair_inside_its_rim_is_built_and_read_back_alike`:
//! a pipe of radius `R = 38613965`, cut by the floor `z = 0` and sliced along
//! the chord between `(x + 1, −x)` and `(x, −(x + 1))` with `x = 27304196`,
//! two points of the circle one step apart each way, all scaled by
//! `(1 + 2⁻²⁰) · 2⁻²³`, which keeps both ends exactly on the circle. The
//! picker is the builder's plane, through the first end and square to the
//! chord, and it holds both ends exactly.
//!
//! The short arc stands `7.7e-16` off its chord, the sagitta `(c/2)²/2R` with
//! `c = √2 (far − near)`, so its middle is inside the box `256 ε` times the
//! circle's size that the walk asks the picker about, and the picker cannot
//! tell which side that middle is on. The long arc's middle, across the
//! circle, is told outright. Until Q1837 the walk read the short arc's
//! undecided sign as holding it: facing the axis it picked both arcs and
//! refused, and facing away it picked the short arc but left an undecided
//! record behind, which a certified fuse refuses.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point, Surface,
    Vector, walk,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog, Record};

/// The plane through both ends facing the bar's axis picks the long arc: some
/// point of the walk is across the circle from the slice.
#[test]
fn a_plane_through_both_ends_facing_the_axis_picks_the_long_arc() {
    let [_, _, round] = numbers();
    let (walked, records) = walked_with(Vector::new(-1.0, 1.0, 0.0));
    let points = walked.unwrap_or_else(|why| panic!("refused {why:?}, records {records:#?}"));
    assert!(
        points.iter().any(|point| point.y() - point.x() > round),
        "the long arc passes (−R/√2, R/√2), and these points do not come near it: {points:?}"
    );
    assert!(records.is_empty(), "left undecided: {records:#?}");
}

/// The same plane facing away from the axis picks the short arc: every point
/// of the walk is on the slice's side of the circle, and nothing is left
/// undecided.
#[test]
fn a_plane_through_both_ends_facing_away_from_the_axis_picks_the_short_arc() {
    let [_, _, round] = numbers();
    let (walked, records) = walked_with(Vector::new(1.0, -1.0, 0.0));
    let points = walked.unwrap_or_else(|why| panic!("refused {why:?}, records {records:#?}"));
    assert!(
        points.iter().all(|point| point.y() - point.x() < -round),
        "the short arc is {:?}, and these points are not on it: {points:?}",
        ends()
    );
    assert!(records.is_empty(), "left undecided: {records:#?}");
}

/// Walk the floor's rim between [`ends`], picked by the plane through the
/// first end facing `facing`, under a certified policy. The plane is checked
/// to hold both ends exactly first: that is what makes which arc it picks a
/// question with an answer.
fn walked_with(facing: Vector) -> (Result<Vec<Point>, Error>, Vec<Record>) {
    let [_, _, round] = numbers();
    let ends = ends();
    let picker = HalfSpace::through_a_point_facing(ends[0], facing)
        .expect("a direction square to the chord has length");
    for end in ends {
        assert_eq!(
            picker.side_of_point(PredicatePoint::from(end.coordinates())),
            Decision::Zero,
            "the plane facing {facing:?} holds {end:?}"
        );
    }
    let bottom = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("an axis direction has length");
    let pipe = Cylinder::new(Axis::z(), round).expect("a positive radius");
    let run = NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(bottom),
            picked_by: [Picker::from(picker)].into(),
        },
        ends.map(|end| PredicatePoint::from(end.coordinates())),
    )
    .expect("an arc with two ends");
    let mut log = ProvenanceLog::new();
    let walked = walk(
        &run,
        [&Surface::Plane(bottom), &Surface::Cylinder(pipe)],
        Deflection::of(0.01).expect("a positive length"),
        &Policy::certified(),
        &mut log,
    )
    .map(|walked| walked.points().to_vec());
    (walked, log.records().to_vec())
}

/// Where the slice cuts the floor's rim: `(far, −near, 0)` and `(near, −far, 0)`.
fn ends() -> [Point; 2] {
    let [near, far, _] = numbers();
    [Point::new(far, -near, 0.0), Point::new(near, -far, 0.0)]
}

/// `near`, `far` and the radius, scaled. `near² + far² = round²` holds exactly
/// before the scaling and after it.
fn numbers() -> [f64; 3] {
    let s = 1.0 + f64::from_bits((1023 - 20) << 52);
    let scaled =
        [27_304_196.0, 27_304_197.0, 38_613_965.0].map(|number| number * s * 2f64.powi(-23));
    let [near, far, round] = scaled;
    assert_eq!(
        near * near + far * far,
        round * round,
        "both ends on the circle"
    );
    scaled
}
