//! Where a line meets one surface, asked directly and not through two planes.
//!
//! [`meet_of_three_surfaces`] only ever sees a line as the crossing of two
//! planes, because that is what a fuse's corners are built from. A ray fired at
//! a face for point classification has no second plane to hand over — it has a
//! point and a direction — so `classify.rs` in `rcad-topo` needed a public entry
//! that takes a line on its own terms.
//!
//! **What it hands back is `t`, and both ends of the line are boxes**, because
//! that is what a ray caster has and what it wants. Ahead of where the ray
//! started or behind it is the sign of `t` and nothing else, and the point being
//! classified is one the fuse *built*, which is a box and not a triple of `f64`.
//! An entry point that took exact line data only could not be called with the
//! thing the caller is holding.
//!
//! Every `t` below is checked against a closed form and never against the
//! solver's own opinion. Every direction has length one and every closed-form
//! answer is a whole number, so no assertion here is a tolerance.
//!
//! The torus case also checks the new entry against [`meet_of_three_surfaces`]
//! itself: the same line, handed over directly here and built as two crossing
//! planes there, must land on the same four places. Two ways of asking the same
//! question that disagree is the exact defect this milestone paid for once
//! already.

use rcad_geom::{
    Axis, Cone, Cylinder, Error, MeetOfThree, MeetsAt, Plane, Point, Sphere, Surface, Torus,
    Vector, WhereALineMeetsIt, meet_of_three_surfaces, where_a_line_meets_a_surface,
};
use rcad_predicates::{Decision, Interval, Policy, ProvenanceLog, SideOfSurface};

fn exactly(triple: [f64; 3]) -> [Interval; 3] {
    triple.map(Interval::exactly)
}

fn asked(from: [Interval; 3], along: [Interval; 3], surface: &Surface) -> WhereALineMeetsIt {
    where_a_line_meets_a_surface(
        from,
        along,
        surface,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("every line below meets its surface, or misses it cleanly")
}

fn meet(from: [f64; 3], along: [f64; 3], surface: &Surface) -> WhereALineMeetsIt {
    asked(exactly(from), exactly(along), surface)
}

/// Every step the line meets the surface at, ascending, and each one exactly —
/// a step that is a box rather than a number is a failure here, because every
/// case below has a whole-number answer the arithmetic reaches with no
/// rounding.
fn steps(answer: &WhereALineMeetsIt) -> Vec<f64> {
    let WhereALineMeetsIt::At(found) = answer else {
        panic!("expected places along the line, got {answer:?}");
    };
    let mut settled: Vec<f64> = found
        .iter()
        .map(|step| {
            let MeetsAt::Here(step) = step else {
                panic!("expected a settled place, got {step:?}");
            };
            assert_eq!(
                step.lo(),
                step.hi(),
                "this answer is a whole number and came back as a box"
            );
            step.lo()
        })
        .collect();
    settled.sort_by(f64::total_cmp);
    settled
}

fn flat(normal: [f64; 3]) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(
            Point::origin(),
            Vector::new(normal[0], normal[1], normal[2]),
        )
        .expect("a normal with a length"),
    )
}

/// A line square to a plane meets it once, as far along as it started away.
#[test]
fn a_line_square_to_a_plane_meets_it_at_the_foot_of_the_perpendicular() {
    let floor = flat([0.0, 0.0, 1.0]);
    assert_eq!(
        steps(&meet([2.0, 3.0, 5.0], [0.0, 0.0, -1.0], &floor)),
        [5.0]
    );
}

/// A line inside a plane shares the whole of it, and a line parallel to a
/// plane and off it shares nothing.
#[test]
fn a_line_in_a_plane_is_more_than_a_point_and_one_beside_it_is_nowhere() {
    let floor = flat([0.0, 0.0, 1.0]);
    assert_eq!(
        meet([0.0, 0.0, 0.0], [1.0, 1.0, 0.0], &floor),
        WhereALineMeetsIt::MoreThanAPoint
    );
    assert_eq!(
        meet([0.0, 0.0, 4.0], [1.0, 1.0, 0.0], &floor),
        WhereALineMeetsIt::Nowhere
    );
}

/// A ray started at a ball's centre meets it once ahead and once behind, and
/// the two are told apart by the sign of `t` alone.
///
/// **This is what the entry point is for.** A point classifier counts only the
/// crossings ahead of where the ray started; recovering that from a returned
/// *point* would mean re-deriving `t` from three coordinates the solver had
/// already worked it out from.
#[test]
fn a_ray_from_a_balls_centre_meets_it_once_ahead_and_once_behind() {
    let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
    let found = steps(&meet([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], &ball));
    assert_eq!(found, [-5.0, 5.0]);
    assert_eq!(found.iter().filter(|step| **step > 0.0).count(), 1);
}

/// A line tangent to a ball is a double root, not a miss — the same grazing
/// answer the torus case gives, one degree lower.
#[test]
fn a_line_tangent_to_a_ball_meets_it_once() {
    let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
    assert_eq!(steps(&meet([5.0, 0.0, 0.0], [0.0, 1.0, 0.0], &ball)), [0.0]);
    assert_eq!(
        meet([6.0, 0.0, 0.0], [0.0, 1.0, 0.0], &ball),
        WhereALineMeetsIt::Nowhere
    );
}

/// A line across a pipe meets its wall on both sides.
#[test]
fn a_line_across_a_pipe_meets_both_walls() {
    let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
    assert_eq!(
        steps(&meet([0.0, 0.0, 7.0], [1.0, 0.0, 0.0], &pipe)),
        [-3.0, 3.0]
    );
}

/// A line down a pipe's own wall shares the whole of it.
#[test]
fn a_line_down_a_pipes_own_wall_is_more_than_a_point() {
    let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
    assert_eq!(
        meet([3.0, 0.0, 0.0], [0.0, 0.0, 1.0], &pipe),
        WhereALineMeetsIt::MoreThanAPoint
    );
}

/// A cone opening at slope one has radius `|z|` at height `z`, so a level cut
/// meets it exactly where `x² = z²`.
#[test]
fn a_level_line_across_a_cone_meets_it_where_the_radius_says() {
    let point = Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a slope"));
    assert_eq!(
        steps(&meet([0.0, 0.0, 5.0], [1.0, 0.0, 0.0], &point)),
        [-5.0, 5.0]
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
    let direct = steps(&meet([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], &ring));
    assert_eq!(direct, [-5.0, -3.0, 3.0, 5.0]);

    let by_two_planes = meet_of_three_surfaces(
        [&flat([0.0, 1.0, 0.0]), &flat([0.0, 0.0, 1.0]), &ring],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("two planes crossing in the x axis meet this ring");
    let MeetOfThree::At(points) = &by_two_planes else {
        panic!("expected four points, got {by_two_planes:?}");
    };
    // The two planes cross in the `x` axis through the origin, so a step along
    // that line *is* the `x` coordinate of the point it names.
    let mut through_the_planes: Vec<f64> = points
        .iter()
        .map(|point| {
            let across = point.enclosure()[0];
            assert_eq!(across.lo(), across.hi(), "this corner is a whole number");
            across.lo()
        })
        .collect();
    through_the_planes.sort_by(f64::total_cmp);
    assert_eq!(
        direct, through_the_planes,
        "a line asked directly and the same line built from two planes must agree"
    );
}

/// A line tangent to the ring's outer equator is a double root: one place, and
/// it is a real answer rather than a near miss read as nothing.
#[test]
fn a_line_tangent_to_the_rings_outer_equator_meets_it_once() {
    let ring = Surface::Torus(a_ring());
    assert_eq!(steps(&meet([5.0, 0.0, 0.0], [0.0, 1.0, 0.0], &ring)), [0.0]);
}

/// A line a whole unit clear of the ring meets it nowhere, and every place the
/// solver does find is checked against the ring's own implicit equation.
#[test]
fn a_line_clear_of_the_ring_meets_it_nowhere_and_every_found_place_is_on_the_skin() {
    let ring = a_ring();
    assert_eq!(
        meet([6.0, 0.0, 0.0], [0.0, 1.0, 0.0], &Surface::Torus(ring)),
        WhereALineMeetsIt::Nowhere
    );

    for step in steps(&meet(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        &Surface::Torus(ring),
    )) {
        let place = Point::new(step, 0.0, 0.0);
        let side = ring.side_of(place.into());
        assert!(
            !matches!(side, Decision::Pos | Decision::Neg),
            "a place the solver found is off the ring it came from: {side:?}"
        );
    }
}

/// A box that names one `f64` per side **is** that triple, so a corner a fuse
/// built on the lattice gets the whole exact cascade — including the torus,
/// whose roots are found by exact signs and nothing else.
///
/// Without this the entry point would be unusable for the thing it exists for:
/// what a point classifier holds is a [`Point::Constructed`] box, never a bare
/// triple, and a ray caster that turned every one of those away would answer
/// nothing at all.
#[test]
fn a_box_that_names_a_number_gets_the_exact_answer_a_number_would() {
    let ring = Surface::Torus(a_ring());
    let collapsed = [0.0, 0.0, 0.0]
        .map(|side: f64| Interval::new(side, side).expect("a box with no width is still a box"));
    assert_eq!(
        steps(&asked(collapsed, exactly([1.0, 0.0, 0.0]), &ring)),
        [-5.0, -3.0, 3.0, 5.0]
    );
}

/// A ray whose start is a box a whole unit wide still crosses a ball it runs
/// clean through, and the two steps come back as boxes rather than numbers.
///
/// The quadric route reads its answer off a discriminant, and a discriminant
/// that is a box still says which way it points as long as the box stays clear
/// of zero. What is lost is width in the answer, not the answer.
#[test]
fn a_ray_from_a_box_still_crosses_a_ball_it_runs_clean_through() {
    let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
    let smeared = [0.0; 3].map(|_| Interval::new(-0.5, 0.5).expect("a box"));
    let WhereALineMeetsIt::At(found) = asked(smeared, exactly([0.0, 0.0, 1.0]), &ball) else {
        panic!("a ray from inside the ball crosses it twice");
    };
    assert_eq!(found.len(), 2);
    let mut ahead = 0;
    for step in &found {
        let MeetsAt::Here(step) = step else {
            panic!("a ball has no touch a number cannot name: {step:?}");
        };
        assert!(step.lo() < step.hi(), "a boxed ray gives a boxed step");
        assert!(
            step.lo() > -6.0 && step.hi() < 6.0,
            "a step off a ball of five landed at {step:?}"
        );
        if step.lo() > 0.0 {
            ahead += 1;
        }
    }
    assert_eq!(ahead, 1, "one of the two crossings is ahead of the start");
}

/// A ray whose start box straddles the ball's own skin is refused under a
/// certified policy, because whether it crosses twice or not at all differs
/// across the box.
///
/// **Not a tolerance and not a failure of the arithmetic**: the two answers are
/// both true of lines inside that box, so there is no single answer to give.
#[test]
fn a_ray_whose_box_straddles_the_skin_is_refused_rather_than_guessed() {
    let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
    let across_the_skin = [
        Interval::new(4.5, 5.5).expect("a box"),
        Interval::exactly(0.0),
        Interval::exactly(0.0),
    ];
    let answer = where_a_line_meets_a_surface(
        across_the_skin,
        exactly([0.0, 1.0, 0.0]),
        &ball,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(answer, Err(Error::Undecidable { .. })),
        "a ray whose start straddles the skin came back as {answer:?}"
    );
}

/// A torus asked about a line that is only a box is refused **by name**, and
/// that name says which refusal it is.
///
/// A quartic's roots are found by evaluating its sign along the line, and a
/// coefficient that is a box has no sign to evaluate near a root — so this is
/// not the same refusal as a discriminant that could not be settled, and it does
/// not become one by being wrapped in [`Error::Undecidable`].
#[test]
fn a_torus_asked_about_a_line_that_is_a_box_is_refused_by_name() {
    let ring = Surface::Torus(a_ring());
    let smeared = [0.0; 3].map(|_| Interval::new(-0.5, 0.5).expect("a box"));
    let answer = where_a_line_meets_a_surface(
        smeared,
        exactly([1.0, 0.0, 0.0]),
        &ring,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert_eq!(answer, Err(Error::WhereALineMeetsARingNeedsTheLineExactly));
}
