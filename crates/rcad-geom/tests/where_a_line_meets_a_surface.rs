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

/// A line and a plane written `2⁻⁵²⁸` across meet where the same two do at size
/// one, bit for bit.
///
/// A plane is the one surface here that answers without a slide: every product
/// it takes is a length times a normal, near `2⁻⁵²⁸` and not near `2⁻¹⁰⁵⁶`, so
/// nothing falls off the bottom. `docs/decisions.md` §349.8.
#[test]
fn a_line_and_a_plane_written_about_1e_159_across_meet_where_they_do_at_size_one() {
    let floor = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, 1.0))
            .expect("a normal with a length"),
    );
    the_same_steps_at_both_sizes(&Question {
        from: [1.0, 1.0, 7.0],
        along: [1.0, 2.0, -3.0],
        surface: floor,
    });
}

/// A line and a ball written `2⁻⁵²⁸` across meet where the same two do at size
/// one, bit for bit. Every product of two lengths there is near `2⁻¹⁰⁵⁶`.
/// `docs/decisions.md` §349.8.
#[test]
fn a_line_and_a_ball_written_about_1e_159_across_meet_where_they_do_at_size_one() {
    let ball = Surface::Sphere(Sphere::new(Point::new(1.0, 2.0, 3.0), 5.0).expect("a radius"));
    the_same_steps_at_both_sizes(&Question {
        from: [0.0, 0.0, 0.0],
        along: [1.0, 1.0, 1.0],
        surface: ball,
    });
}

/// A line and a pipe written `2⁻⁵²⁸` across meet where the same two do at size
/// one, bit for bit. `docs/decisions.md` §349.8.
#[test]
fn a_line_and_a_pipe_written_about_1e_159_across_meet_where_they_do_at_size_one() {
    let pipe = Surface::Cylinder(
        Cylinder::new(
            Axis::z()
                .moved_to(Point::new(1.0, 0.0, 0.0))
                .expect("a finite origin"),
            3.0,
        )
        .expect("a radius"),
    );
    the_same_steps_at_both_sizes(&Question {
        from: [0.0, 1.0, 5.0],
        along: [1.0, 1.0, 0.0],
        surface: pipe,
    });
}

/// A line and a cone written `2⁻⁵²⁸` across meet where the same two do at size
/// one, bit for bit. `docs/decisions.md` §349.8.
#[test]
fn a_line_and_a_cone_written_about_1e_159_across_meet_where_they_do_at_size_one() {
    let cone = Surface::Cone(
        Cone::new(
            Axis::z()
                .moved_to(Point::new(0.0, 0.0, 1.0))
                .expect("a finite apex"),
            1.0,
        )
        .expect("a slope"),
    );
    the_same_steps_at_both_sizes(&Question {
        from: [0.0, 0.0, 5.0],
        along: [1.0, 1.0, 0.0],
        surface: cone,
    });
}

/// A line and a ring written `2⁻⁵²⁸` across meet where the same two do at size
/// one, bit for bit, at all four places. A ring's quartic is a length to the
/// fourth, so where they stand every coefficient is near `2⁻²¹¹²` and is
/// nothing at all in `f64`. `docs/decisions.md` §349.8.
#[test]
fn a_line_and_a_ring_written_about_1e_159_across_meet_where_they_do_at_size_one() {
    the_same_steps_at_both_sizes(&Question {
        from: [0.0, 1.0, 0.0],
        along: [1.0, 0.0, 0.0],
        surface: Surface::Torus(a_ring()),
    });
}

/// A line along the top of a ring's tube touches it twice, at `±√15`, where no
/// `f64` lands — and written `2⁻⁵²⁸` across it still touches it there.
///
/// A touch no `f64` names comes back with how far the ring's quartic reaches
/// over its box, and that is a length to the fourth. It is worked out where the
/// question was slid to, so it is slid back down by four times the slide, and
/// outward: what comes back still straddles zero, which is the whole of what it
/// says. `docs/decisions.md` §349.8.
#[test]
fn a_line_touching_a_ring_written_about_1e_159_across_touches_it_where_it_does_at_size_one() {
    the_same_steps_at_both_sizes(&Question {
        from: [0.0, 1.0, 1.0],
        along: [1.0, 0.0, 0.0],
        surface: Surface::Torus(a_ring()),
    });
}

/// One line and one surface, written at size one.
struct Question {
    from: [f64; 3],
    along: [f64; 3],
    surface: Surface,
}

/// `2⁻⁵²⁸`, the size §349's three surfaces `1e-159` across were written at. A
/// product of two lengths that small is near `2⁻¹⁰⁵⁶`, among the subnormals.
const TINY: i32 = -528;

/// The question asked at size one and again slid down to `2⁻⁵²⁸`, where both
/// must answer, the tiny one must hold what size one says, and the tiny one is
/// measured for how much wider it is.
///
/// **A step along the line does not move when the question is slid**, because
/// the line's direction is a length too and slides with its point: `2ᵏ·from +
/// t·2ᵏ·along` is `2ᵏ` times the point at `t`. So the tiny answer scaled back
/// up *is* the tiny answer, and the two are compared as they come.
fn the_same_steps_at_both_sizes(question: &Question) {
    let at_size_one = asked_certified(question, 0)
        .unwrap_or_else(|refused| panic!("the question at size one is answered: {refused:?}"));
    let mut log = ProvenanceLog::new();
    let tiny = where_a_line_meets_a_surface(
        exactly(question.from.map(|side| slid_exactly(side, TINY))),
        exactly(question.along.map(|side| slid_exactly(side, TINY))),
        &slid(&question.surface, TINY),
        &Policy::certified(),
        &mut log,
    )
    .unwrap_or_else(|refused| panic!("the same question 2^-528 across is answered: {refused:?}"));
    assert!(log.is_empty(), "a certified answer logs nothing: {log:?}");

    let (WhereALineMeetsIt::At(home), WhereALineMeetsIt::At(found)) = (&at_size_one, &tiny) else {
        panic!("both sizes meet somewhere: {at_size_one:?} and {tiny:?}");
    };
    assert_eq!(found.len(), home.len(), "{tiny:?} against {at_size_one:?}");
    for (step, step_at_home) in found.iter().zip(home) {
        match (step, step_at_home) {
            (MeetsAt::Here(step), MeetsAt::Here(step_at_home)) => {
                assert_eq!(
                    (step.lo().to_bits(), step.hi().to_bits()),
                    (step_at_home.lo().to_bits(), step_at_home.hi().to_bits()),
                    "the step 2^-528 across is the step at size one, bit for bit: \
                     {step:?} against {step_at_home:?}"
                );
            }
            (
                MeetsAt::MaybeATouch {
                    inside,
                    how_far_it_reaches,
                },
                MeetsAt::MaybeATouch {
                    inside: inside_at_home,
                    how_far_it_reaches: reach_at_home,
                },
            ) => {
                assert_eq!(
                    (inside.lo().to_bits(), inside.hi().to_bits()),
                    (inside_at_home.lo().to_bits(), inside_at_home.hi().to_bits()),
                    "the touch 2^-528 across is in the box it is at size one"
                );
                assert!(
                    how_far_it_reaches.contains_zero() && reach_at_home.contains_zero(),
                    "a touch no f64 names reaches either side of zero at both sizes: \
                     {how_far_it_reaches:?} and {reach_at_home:?}"
                );
            }
            (step, step_at_home) => {
                panic!("the two sizes read one step two ways: {step:?} and {step_at_home:?}")
            }
        }
    }
}

fn asked_certified(question: &Question, exponent: i32) -> Result<WhereALineMeetsIt, Error> {
    where_a_line_meets_a_surface(
        exactly(question.from.map(|side| slid_exactly(side, exponent))),
        exactly(question.along.map(|side| slid_exactly(side, exponent))),
        &slid(&question.surface, exponent),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

/// The same surface with space scaled by `2^exponent`: every length multiplied
/// and every direction, normal and slope left as its bits are.
fn slid(surface: &Surface, exponent: i32) -> Surface {
    let by = |value: f64| slid_exactly(value, exponent);
    let place = |at: Point| Point::from(at.coordinates().map(by));
    match surface {
        Surface::Plane(flat) => {
            Surface::Plane(flat.moved_to(by(flat.offset())).expect("a finite offset"))
        }
        Surface::Sphere(ball) => {
            Surface::Sphere(Sphere::new(place(ball.centre()), by(ball.radius())).expect("a radius"))
        }
        Surface::Cylinder(pipe) => Surface::Cylinder(
            Cylinder::new(
                pipe.axis()
                    .moved_to(place(pipe.axis().origin()))
                    .expect("a finite origin"),
                by(pipe.radius()),
            )
            .expect("a radius"),
        ),
        Surface::Cone(cone) => Surface::Cone(
            Cone::new(
                cone.axis()
                    .moved_to(place(cone.apex()))
                    .expect("a finite apex"),
                cone.slope(),
            )
            .expect("the same slope"),
        ),
        Surface::Torus(ring) => Surface::Torus(
            Torus::new(
                ring.axis()
                    .moved_to(place(ring.centre()))
                    .expect("a finite centre"),
                by(ring.ring_radius()),
                by(ring.tube_radius()),
            )
            .expect("the same two radii"),
        ),
        other => panic!("nothing here can slide {other:?}"),
    }
}

/// `value * 2^exponent`, which has to be exactly that.
fn slid_exactly(value: f64, exponent: i32) -> f64 {
    let moved = value * 2f64.powi(exponent);
    assert!(
        value == 0.0 || moved.is_normal(),
        "{value:e} does not slide by 2^{exponent} exactly"
    );
    moved
}
