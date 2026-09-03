//! Where a line meets one surface, asked directly and not through two planes.
//!
//! [`meet_of_three_surfaces`] only ever sees a line as the crossing of two
//! planes, because that is what a fuse's corners are built from. A ray fired at
//! a face for point classification has no second plane to hand over — it has a
//! point and a direction — so `classify.rs` in `rcad-topo` needed a public entry
//! that takes a line on its own terms.
//! [`where_a_line_meets_a_surface`] is that entry, and every test below checks
//! it against a closed form and never against the solver's own opinion.
//!
//! The torus case also checks the new entry against [`meet_of_three_surfaces`]
//! itself: the same line, built as an [`Axis`] here and as two crossing planes
//! there, must land on the same four points. Two ways of asking the same
//! question that disagree is the exact defect this milestone paid for once
//! already.

use rcad_geom::{
    Axis, Cone, Cylinder, MeetOfThree, Plane, Point, Sphere, Surface, Torus, Vector,
    meet_of_three_surfaces, where_a_line_meets_a_surface,
};
use rcad_predicates::{Decision, Policy, ProvenanceLog, SideOfSurface};

fn meet(line: &Axis, surface: &Surface) -> MeetOfThree {
    where_a_line_meets_a_surface(
        line,
        surface,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("every line below meets its surface, or misses it cleanly")
}

fn middles(answer: &MeetOfThree) -> Vec<[f64; 3]> {
    let MeetOfThree::At(points) = answer else {
        panic!("expected points, got {answer:?}");
    };
    let mut found: Vec<[f64; 3]> = points
        .iter()
        .map(|point| point.enclosure().map(|side| (side.lo() + side.hi()) / 2.0))
        .collect();
    found.sort_by(|one, other| {
        one.iter()
            .zip(other)
            .find(|(mine, theirs)| mine != theirs)
            .map_or(core::cmp::Ordering::Equal, |(mine, theirs)| {
                mine.total_cmp(theirs)
            })
    });
    found
}

fn axis(through: [f64; 3], direction: [f64; 3]) -> Axis {
    Axis::new(
        Point::new(through[0], through[1], through[2]),
        Vector::new(direction[0], direction[1], direction[2]),
    )
    .expect("a direction with a length")
}

/// A line square to a plane meets it once, where it says it does.
#[test]
fn a_line_square_to_a_plane_meets_it_at_the_foot_of_the_perpendicular() {
    let floor = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("a normal with a length"),
    );
    let down = axis([2.0, 3.0, 5.0], [0.0, 0.0, -1.0]);
    assert_eq!(middles(&meet(&down, &floor)), vec![[2.0, 3.0, 0.0]]);
}

/// A line inside a plane shares the whole of it, and a line parallel to a
/// plane and off it shares nothing.
#[test]
fn a_line_in_a_plane_is_more_than_a_point_and_one_beside_it_is_nowhere() {
    let floor = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("a normal with a length"),
    );
    let along = axis([0.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
    assert_eq!(meet(&along, &floor), MeetOfThree::MoreThanAPoint);

    let beside = axis([0.0, 0.0, 4.0], [1.0, 1.0, 0.0]);
    assert_eq!(meet(&beside, &floor), MeetOfThree::Nowhere);
}

/// A line through a ball's centre meets it at both poles.
#[test]
fn a_line_through_a_balls_centre_meets_it_at_both_poles() {
    let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
    let through = axis([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    assert_eq!(
        middles(&meet(&through, &ball)),
        vec![[0.0, 0.0, -5.0], [0.0, 0.0, 5.0]]
    );
}

/// A line tangent to a ball is a double root, not a miss — the same grazing
/// answer the torus case gives, one degree lower.
#[test]
fn a_line_tangent_to_a_ball_meets_it_once() {
    let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
    let grazing = axis([5.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    assert_eq!(middles(&meet(&grazing, &ball)), vec![[5.0, 0.0, 0.0]]);

    let clear = axis([6.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    assert_eq!(meet(&clear, &ball), MeetOfThree::Nowhere);
}

/// A line across a pipe meets its wall on both sides.
#[test]
fn a_line_across_a_pipe_meets_both_walls() {
    let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
    let across = axis([0.0, 0.0, 7.0], [1.0, 0.0, 0.0]);
    assert_eq!(
        middles(&meet(&across, &pipe)),
        vec![[-3.0, 0.0, 7.0], [3.0, 0.0, 7.0]]
    );
}

/// A line down a pipe's own wall shares the whole of it.
#[test]
fn a_line_down_a_pipes_own_wall_is_more_than_a_point() {
    let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
    let seam = axis([3.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    assert_eq!(meet(&seam, &pipe), MeetOfThree::MoreThanAPoint);
}

/// A cone opening at slope one has radius `|z|` at height `z`, so a level cut
/// meets it exactly where `x² = z²`.
#[test]
fn a_level_line_across_a_cone_meets_it_where_the_radius_says() {
    let point = Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a slope"));
    let level = axis([0.0, 0.0, 5.0], [1.0, 0.0, 0.0]);
    assert_eq!(
        middles(&meet(&level, &point)),
        vec![[-5.0, 0.0, 5.0], [5.0, 0.0, 5.0]]
    );
}

/// A ring of four and one about the `z` axis, the same one
/// `where_a_line_meets_a_torus.rs` uses, so the count here is checked against
/// the same hand-worked closed form: the outer equator is at radius five, the
/// inner at three.
fn a_ring() -> Torus {
    Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring")
}

/// The `x` axis runs across the hole and through both walls of the tube
/// twice — the four places a ring has that a quadric does not — and it is the
/// same answer whether the line is handed over directly or built as two
/// crossing planes.
#[test]
fn a_line_through_the_hole_meets_the_ring_four_times_and_agrees_with_two_crossing_planes() {
    let ring = Surface::Torus(a_ring());
    let line = axis([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    let direct = middles(&meet(&line, &ring));
    assert_eq!(
        direct,
        vec![
            [-5.0, 0.0, 0.0],
            [-3.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [5.0, 0.0, 0.0],
        ]
    );

    let plane = |normal: [f64; 3]| {
        Surface::Plane(
            Plane::from_point_and_normal(
                Point::origin(),
                Vector::new(normal[0], normal[1], normal[2]),
            )
            .expect("a normal with a length"),
        )
    };
    let by_two_planes = meet_of_three_surfaces(
        [&plane([0.0, 1.0, 0.0]), &plane([0.0, 0.0, 1.0]), &ring],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("two planes crossing in the x axis meet this ring");
    assert_eq!(
        direct,
        middles(&by_two_planes),
        "a line asked directly and the same line built from two planes must agree"
    );
}

/// A line tangent to the ring's outer equator is a double root: one place, and
/// it is a real answer rather than a near miss read as nothing.
#[test]
fn a_line_tangent_to_the_rings_outer_equator_meets_it_once() {
    let ring = Surface::Torus(a_ring());
    let tangent = axis([5.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    assert_eq!(middles(&meet(&tangent, &ring)), vec![[5.0, 0.0, 0.0]]);
}

/// A line a whole unit clear of the ring meets it nowhere, and every place the
/// solver does find is checked against the ring's own implicit equation.
#[test]
fn a_line_clear_of_the_ring_meets_it_nowhere_and_every_found_place_is_on_the_skin() {
    let ring = a_ring();
    let clear = axis([6.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    assert_eq!(meet(&clear, &Surface::Torus(ring)), MeetOfThree::Nowhere);

    let crossing = axis([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    let MeetOfThree::At(places) = meet(&crossing, &Surface::Torus(ring)) else {
        panic!("this line meets the ring");
    };
    for place in &places {
        let side = ring.side_of(*place);
        assert!(
            !matches!(side, Decision::Pos | Decision::Neg),
            "a place the solver found is off the ring it came from: {side:?}"
        );
    }
}
