//! Roadmap §7.4's degeneracy catalogue, for the pairs whose crossing lies in a
//! plane.
//!
//! §24.11 asks for these as a **pass/refuse table and not a rate**, and §24.18's
//! fourth criterion asks that touching at a point and being the same surface
//! come back as *answers* rather than errors. One test per row, and the rows the
//! second stage owns say so out loud rather than being left off the list.
//!
//! | Roadmap §7.4 row | Here | Answer |
//! |---|---|---|
//! | tangent spheres, outside | two balls resting against each other | `TouchAtOnePoint` |
//! | tangent spheres, inside | one ball resting in another | `TouchAtOnePoint` |
//! | tangent cylinder–plane | a plane resting along a pipe | one straight line |
//! | tangent sphere–plane | a plane resting on a ball | `TouchAtOnePoint` |
//! | tangent sphere–cylinder, at a point | a ball resting on the outside of a pipe | `TouchAtOnePoint` |
//! | tangent sphere–cylinder, along a curve | a ball inscribed in a pipe | one closed curve |
//! | tangent sphere–cylinder, through itself | a ball resting on the far inside of a pipe | **refused**, and it is a limit |
//! | tangent cylinder–cylinder, at a point | two pipes leaning and resting against each other | `TouchAtOnePoint` |
//! | tangent cylinder–cylinder, along a line | two pipes side by side and resting | one straight line |
//! | tangent cylinder–cylinder, through itself | two pipes of one thickness whose axes meet | **refused**, and it is a limit |
//! | coplanar faces, one dimension up | a surface against its own copy | `SameSurface` |
//! | closed periodic surfaces | a plane through a whole ball or pipe | one closed curve, named with no seam in it |
//! | seam-crossing intersections | walking a crossing that runs across a face's seam | 3D points, and the seam never comes into it |
//! | two lines no `f64` half-space separates | **refused**, and it is a gap |
//!
//! The last two tests are not §7.4's. They are pairs a file can state —
//! a pipe into a bend, a pipe into a countersink — that the evaluator used to
//! *panic* on where the namer refused, so what they checked was that both
//! halves said the same word. Since row Q1376 the namer no longer refuses the
//! bend: it names the two circles the pipe's rim and the tube's share, and
//! since row Q1396 the evaluator walks an arc of either, refusing only a whole
//! circle named by the two faces alone. The countersink pair is unchanged,
//! both halves still refusing it under its own name.
//!
//! # Why tangency is an answer and not a band
//!
//! For two quadrics whose coefficients are exactly representable, "are they
//! tangent" is the sign of a discriminant, so level 3 of the cascade decides it
//! exactly (§24.10). There is no near-tangency band to widen or narrow, and
//! nothing in this file needs a tolerance anywhere.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point, Sphere,
    Surface, SurfaceMeet, Torus, Vector, intersect, walk, where_a_closed_crossing_starts,
};
use rcad_predicates::{Decision, Policy, ProvenanceLog, SideOfSurface};
use smallvec::smallvec;

fn crossing(first: &Surface, second: &Surface) -> SurfaceMeet {
    let mut log = ProvenanceLog::new();
    let met = intersect(first, second, &Policy::certified(), &mut log)
        .expect("this pair is exactly nameable");
    assert!(log.is_empty(), "nothing here needs approximating: {log:?}");
    met
}

fn a_ball(centre: [f64; 3], radius: f64) -> Surface {
    Surface::Sphere(Sphere::new(Point::from(centre), radius).expect("a radius"))
}

fn a_plane(normal: [f64; 3], through: [f64; 3]) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(Point::from(through), Vector::from(normal))
            .expect("a normal with a length"),
    )
}

fn a_pipe(along: [f64; 3], through: [f64; 3], radius: f64) -> Surface {
    Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::from(through), Vector::from(along)).expect("a direction"),
            radius,
        )
        .expect("a radius"),
    )
}

fn side_of(surface: &Surface, at: rcad_predicates::Point) -> Decision {
    match surface {
        Surface::Plane(flat) => flat.side_of_point(at),
        Surface::Sphere(ball) => ball.side_of(at),
        Surface::Cylinder(pipe) => pipe.side_of(at),
        Surface::Cone(cone) => cone.side_of(at),
        Surface::Torus(ring) => ring.side_of(at),
        other => panic!("nothing here can ask about {other:?}"),
    }
}

/// Two tangent spheres, both ways round, and the point they share is on both.
#[test]
fn two_balls_resting_against_each_other_answer_with_the_one_point_they_share() {
    for (centre, radius) in [([8.0, 0.0, 0.0], 5.0), ([1.0, 0.0, 0.0], 2.0)] {
        let one = a_ball([0.0, 0.0, 0.0], 3.0);
        let other = a_ball(centre, radius);
        let SurfaceMeet::TouchAtOnePoint { at } = crossing(&one, &other) else {
            panic!("a ball at {centre:?} of radius {radius} rests against the unit ball");
        };
        assert_eq!(side_of(&one, at), Decision::Zero);
        assert_eq!(side_of(&other, at), Decision::Zero);
    }
}

/// A plane resting on a ball.
#[test]
fn a_plane_resting_on_a_ball_answers_with_the_one_point_they_share() {
    let flat = a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 3.0]);
    let ball = a_ball([0.0, 0.0, 0.0], 3.0);
    let SurfaceMeet::TouchAtOnePoint { at } = crossing(&flat, &ball) else {
        panic!("a plane a radius from the centre rests on the ball");
    };
    assert_eq!(side_of(&flat, at), Decision::Zero);
    assert_eq!(side_of(&ball, at), Decision::Zero);
}

/// A plane resting along a pipe. **One line, and not a point**, which is the
/// row §24.10 wrote down by hand.
#[test]
fn a_plane_resting_along_a_pipe_answers_with_one_endless_line() {
    let met = crossing(
        &a_plane([1.0, 0.0, 0.0], [2.0, 0.0, 0.0]),
        &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
    );
    let SurfaceMeet::Curves(runs) = &met else {
        panic!("a plane resting along a pipe touches it along a line: {met:?}");
    };
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].run(), &EdgeRun::Straight);
    assert!(runs[0].is_the_whole_curve(), "a line has no ends");
}

/// Roadmap §7.4's "coplanar faces", one dimension up: a surface against its own
/// copy is two-dimensional, and every kind of surface can be.
#[test]
fn a_surface_against_its_own_copy_answers_that_they_are_one_surface() {
    for surface in [
        a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 4.0]),
        a_ball([1.0, 2.0, 3.0], 4.0),
    ] {
        assert_eq!(crossing(&surface, &surface), SurfaceMeet::SameSurface);
    }
}

/// Roadmap §7.4's "closed periodic surfaces". A plane through a whole ball or a
/// whole pipe crosses it in one **closed** curve, and what names that curve is
/// the pair of surfaces — no seam, no start, no parameter that has to wrap.
#[test]
fn a_plane_through_a_closed_surface_names_one_closed_curve_with_no_seam_in_it() {
    for curved in [
        a_ball([0.0, 0.0, 0.0], 3.0),
        a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 3.0),
    ] {
        let met = crossing(&a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]), &curved);
        let SurfaceMeet::Curves(runs) = &met else {
            panic!("a plane through the middle of {curved:?} cuts one closed curve: {met:?}");
        };
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].run(), &EdgeRun::RightRoundWhereItsFacesCross);
        assert!(
            runs[0].is_the_whole_curve(),
            "there is nothing on this circle for it to start at"
        );
    }
}

/// Roadmap §7.4's "seam-crossing intersections".
///
/// A round bar's wall has a seam down one ruling, and the cap circle runs
/// straight across it. **Nothing here notices.** A seam is a face's own
/// bookkeeping and what crosses `walk`'s seam is 3D points, never a `(u, v)`
/// (§27.5) — so the walk comes back evenly spaced right the way round, with no
/// gap and no doubled point where a wrapped parameter would have left one.
#[test]
fn a_crossing_that_runs_across_a_faces_seam_is_walked_without_noticing_it() {
    let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 3.0);
    let cap = a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 5.0]);

    let walked = walk(
        &NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it"),
        [&pipe, &cap],
        Deflection::of(0.01).expect("a length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a bar this size is walked without complaint");

    let points = walked.points();
    assert!(
        points.len() > 20,
        "a whole circle takes more than twenty steps"
    );
    let step = |from: usize| (points[from + 1] - points[from]).length();
    let shortest = (0..points.len() - 1).map(step).fold(f64::MAX, f64::min);
    let longest = (0..points.len() - 1).map(step).fold(f64::MIN, f64::max);
    assert!(
        longest - shortest < 1e-12,
        "the steps run from {shortest} to {longest}, so something in the walk noticed a seam"
    );

    // The seam of a bar built by this kernel is where `Cylinder::basis` puts
    // `u = 0`, which for a pipe on `z` is `+x`. The walk goes straight through
    // it: some point sits on the far side of the seam from where it started.
    assert!(
        points.iter().any(|point| point.y() < 0.0),
        "a walk right round a circle passes the far side of the seam"
    );
}

/// §24.18's second criterion, asked along the whole of a run rather than at its
/// two ends.
///
/// `Model::validate` checks that a named run sits on both surfaces and inside
/// every half-space that picked it, and it can only ask that of the two ends it
/// has. Every point of the walk is the strictly stronger form of the same
/// question, and here it is a ball's seam — the run that forced
/// `EdgeRun::PartOfWhereItCrosses` in the first place.
#[test]
fn every_point_of_a_picked_run_is_inside_every_half_space_that_picked_it() {
    let radius = 3.0;
    let ball = a_ball([0.0, 0.0, 0.0], radius);
    let meridian = a_plane([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
    let facing_x = HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis has length");

    let seam = NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: meridian,
            picked_by: smallvec![Picker::from(facing_x)],
        },
        [
            rcad_predicates::Point::from([0.0, 0.0, radius]),
            rcad_predicates::Point::from([0.0, 0.0, -radius]),
        ],
    )
    .expect("a picked run does not have to come back to where it started");

    let walked = walk(
        &seam,
        [&ball, &ball],
        Deflection::of(0.01).expect("a length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a ball this size is walked without complaint");
    assert!(walked.points().len() > 8);

    let slack = walked.deviation().magnitude();
    for point in walked.points() {
        let at = rcad_predicates::Point::Constructed(point.coordinates().map(|coordinate| {
            rcad_predicates::Interval::around(coordinate, slack).expect("a bound is a length")
        }));
        for surface in [&ball, &meridian] {
            assert_ne!(side_of(surface, at), Decision::Pos, "off {surface:?}");
            assert_ne!(side_of(surface, at), Decision::Neg, "off {surface:?}");
        }
        assert_ne!(
            facing_x.side_of_point(at),
            Decision::Neg,
            "outside the half-space that picked this half of the meridian: {point:?}"
        );
    }
}

/// Roadmap §7.4's tangent sphere–cylinder row, all three ways it can happen.
///
/// **Tangency is not one row here, it is three**, and only the third refuses.
/// Two quadrics can rest against each other at a point, along a whole curve, or
/// at a point *on* their crossing curve — and the third is the only one this
/// vocabulary cannot say, because a curve that runs through itself is neither
/// one closed curve nor a piece picked out by a half-space.
#[test]
fn a_ball_tangent_to_a_pipe_answers_three_different_ways_and_refuses_only_the_third() {
    let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);

    // At a point: a ball five out with a radius of three rests on the skin.
    let resting = a_ball([5.0, 0.0, 0.0], 3.0);
    let SurfaceMeet::TouchAtOnePoint { at } = crossing(&resting, &pipe) else {
        panic!("a ball five out with a radius of three rests on a pipe of radius two");
    };
    assert_eq!(side_of(&resting, at), Decision::Zero);
    assert_eq!(side_of(&pipe, at), Decision::Zero);

    // Along a curve: a ball inscribed in the pipe touches it all the way round
    // one circle, and that is an answer with a curve in it.
    let inscribed = crossing(&a_ball([0.0, 0.0, 0.0], 2.0), &pipe);
    let SurfaceMeet::Curves(runs) = &inscribed else {
        panic!("a ball inscribed in a pipe meets it all the way round: {inscribed:?}");
    };
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].run(), &EdgeRun::RightRoundWhereItsFacesCross);

    // Through itself: a ball one out with a radius of three reaches exactly the
    // pipe's whole width, so the two loops it would leave pinch together.
    assert_eq!(
        intersect(
            &a_ball([1.0, 0.0, 0.0], 3.0),
            &pipe,
            &Policy::certified(),
            &mut ProvenanceLog::new()
        ),
        Err(Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo)
    );
}

/// Roadmap §7.4's tangent cylinder–cylinder row, the same three ways, and all
/// three answer since Q1603 named the third.
#[test]
fn a_pipe_tangent_to_a_pipe_answers_all_three_ways() {
    let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);

    // At a point: two skew pipes exactly `r₁ + r₂` apart.
    let leaning = a_pipe([1.0, 0.0, 0.0], [0.0, 5.0, 0.0], 3.0);
    let SurfaceMeet::TouchAtOnePoint { at } = crossing(&pipe, &leaning) else {
        panic!("five apart with radii of two and three rest against each other");
    };
    assert_eq!(side_of(&pipe, at), Decision::Zero);
    assert_eq!(side_of(&leaning, at), Decision::Zero);

    // Along a line: two pipes side by side, resting against each other.
    let beside = crossing(&pipe, &a_pipe([0.0, 0.0, 1.0], [5.0, 0.0, 0.0], 3.0));
    let SurfaceMeet::Curves(runs) = &beside else {
        panic!("two pipes side by side and resting meet along one line: {beside:?}");
    };
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].run(), &EdgeRun::Straight);

    // Through itself: two pipes of one thickness whose axes meet cross in two
    // ellipses, and those two cross each other twice, where the walls touch.
    // Each ellipse is cut there, so the crossing is four halves.
    let through_itself = crossing(&pipe, &a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], 2.0));
    let SurfaceMeet::Curves(runs) = &through_itself else {
        panic!("two equal pipes whose axes meet cross in curves: {through_itself:?}");
    };
    assert_eq!(runs.len(), 4, "two ellipses, each in two halves");
    for run in runs {
        assert!(
            matches!(run.run(), EdgeRun::PartOfWhereItCrosses { picked_by, .. } if picked_by.len() == 2),
            "each half is picked by a corner of two flat surfaces: {run:?}"
        );
    }
}

/// Two pipes side by side cut each other in straight lines, and a straight run
/// is the whole of what its two ends say — even though the pair that named it is
/// the one the evaluator otherwise hands to the marcher.
///
/// The lines come back as pieces of a crossing rather than as
/// [`EdgeRun::Straight`], because a half-space is what tells the two of them
/// apart, so nothing about the run itself says there is nothing to draw.
#[test]
fn two_pipes_side_by_side_are_walked_into_no_points_at_all_and_no_slop() {
    let one = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
    let other = a_pipe([0.0, 0.0, 1.0], [3.0, 0.0, 0.0], 2.0);
    let met = crossing(&one, &other);
    let SurfaceMeet::Curves(runs) = &met else {
        panic!("two pipes side by side and overlapping cut two lines: {met:?}");
    };
    assert_eq!(runs.len(), 2);

    for run in runs {
        let walked = walk(
            run,
            [&one, &other],
            Deflection::of(0.01).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a straight run needs nothing between its ends");
        assert!(walked.points().is_empty());
        assert_eq!(walked.deviation().magnitude(), 0.0);
    }
}

/// A degree-four crossing is named *and* drawn: the naming half says there are
/// two loops and which half-space picks each, and the marcher follows one of
/// them right round.
#[test]
fn a_quartic_crossing_is_named_in_two_pieces_and_each_piece_is_walked_right_round() {
    let ball = a_ball([0.0, 0.0, 0.0], 5.0);
    let pipe = a_pipe([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 3.0);
    let met = crossing(&ball, &pipe);
    let SurfaceMeet::Curves(runs) = &met else {
        panic!("a ball this size swallows a whole cross-section of the pipe: {met:?}");
    };
    assert_eq!(runs.len(), 2, "one loop either side of the ball's centre");

    let mut heights = Vec::new();
    for run in runs {
        let walked = walk(
            run,
            [&ball, &pipe],
            Deflection::of(0.01).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a shape this size is marched without complaint");
        assert!(
            walked.points().len() > 20,
            "a whole loop takes more than twenty steps, got {}",
            walked.points().len()
        );

        let slack = walked.deviation().magnitude();
        for point in walked.points() {
            let at = rcad_predicates::Point::Constructed(point.coordinates().map(|coordinate| {
                rcad_predicates::Interval::around(coordinate, slack).expect("a bound is a length")
            }));
            for surface in [&ball, &pipe] {
                assert_ne!(side_of(surface, at), Decision::Pos, "off {surface:?}");
                assert_ne!(side_of(surface, at), Decision::Neg, "off {surface:?}");
            }
        }
        heights.push(
            walked
                .points()
                .iter()
                .map(|point| point.z())
                .fold(0.0f64, |so_far, height| so_far.max(height.abs())),
        );
        assert!(
            walked
                .points()
                .iter()
                .all(|point| point.z().signum() == walked.points()[0].z().signum()),
            "a loop picked out by a half-space stays on its own side of it"
        );
    }
    assert_eq!(heights.len(), 2);
}

/// **The refusal that used to be here is gone**, and this row is what it turned
/// into.
///
/// The closed form's one non-deferral refusal was a **gap**: two lines a plane
/// cuts out of a pipe it runs down are there and are tellable apart, and what
/// ran out was `f64`'s supply of *unit* vectors exactly square to a given
/// direction. Since M4 the picker is a `HalfSpace`, which stores whatever normal
/// it is handed, and the supply of square normals is three for every direction —
/// so the turned pair is named exactly as the lattice one always was.
///
/// The name of the run is what this checks, not only that something came back:
/// each of the two lines is picked out by a half-space whose normal is square to
/// the pipe's stored axis direction with nothing rounded.
#[test]
fn two_lines_off_the_lattice_are_named_just_as_the_same_pair_on_it_is() {
    let along = Vector::new(1.0, 2.0, 3.0)
        .normalized()
        .expect("a direction");
    let normal = Vector::new(2.0, -1.0, 0.0)
        .normalized()
        .expect("a direction");
    assert_eq!(
        along.dot(normal),
        0.0,
        "exactly along the axis, or it is an ellipse"
    );

    let leaning = a_pipe(along.components(), [0.0, 0.0, 0.0], 2.0);
    let Surface::Cylinder(pipe) = leaning else {
        panic!("a pipe is a pipe");
    };
    let cut = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.2, 0.0, 0.0), normal).expect("a unit normal"),
    );

    let met = crossing(&cut, &leaning);
    let SurfaceMeet::Curves(runs) = &met else {
        panic!("off the lattice this cuts two lines too: {met:?}");
    };
    assert_eq!(runs.len(), 2);
    for run in runs {
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
            panic!("one of two lines is a piece of the crossing: {run:?}");
        };
        let Picker::InFrontOf(half_space) = picked_by[0] else {
            panic!("the picker for two lines is flat: {picked_by:?}");
        };
        let stored_way = pipe.axis().direction().components();
        let picking = half_space.normal().components();
        assert_eq!(
            picking[0] * stored_way[0] + picking[1] * stored_way[1] + picking[2] * stored_way[2],
            0.0,
            "the picking normal leans into the direction the lines run along"
        );
    }

    let on_the_lattice = crossing(
        &a_plane([1.0, 0.0, 0.0], [0.2, 0.0, 0.0]),
        &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
    );
    let SurfaceMeet::Curves(runs) = &on_the_lattice else {
        panic!("on the lattice this cuts two lines: {on_the_lattice:?}");
    };
    assert_eq!(runs.len(), 2);
}

/// A pipe and a bend, exactly as one KiCad file holds them. In the reals the
/// pipe's radius, `0.5`, is the ring's radius `0.4` plus its tube's radius
/// `0.1` on the nose, so the pipe should rest on the tube's outer rim all the
/// way round — `the_heights_a_ring_crosses_a_pipe_at`'s "zero" row, one
/// circle. `0.5 − 0.4` stored as `f64` is `0.09999999999999998`, one bit short
/// of `0.1`: squared, `0.009999999999999995`, one bit less than
/// `tube² = 0.1 × 0.1 = 0.010000000000000002`. Carried through as exact
/// rationals on those bits rather than rounded again, `room = tube² − (a −
/// R)²` is `7205759403792793 / 2¹¹⁰`, about `5.55e-18` — positive on the bits
/// stored, not on a print rounded to look like zero. So `f64` lands on the
/// "above zero" row instead: two circles a hair's width (`2.356e-9`) apart,
/// where the reals give one.
///
/// Since row Q1376, `intersect` names both, and since row Q1396 `walk` follows
/// them: an arc of each, between two corners on it, is walked round that
/// circle and not the other, `2.356e-9` away, which is three thousand times
/// what a point it places may be off. A whole circle named by the two faces
/// alone is still refused `ACrossingBetweenATorusAndAnotherCurvedSurface`, on
/// purpose: `walk`'s `where_a_ring_and_a_surface_on_its_axis_cut` says why.
///
/// The shapes are the ones the corpus panicked on before Q1376, to the digit.
#[test]
fn a_pipe_that_should_rest_on_a_bends_tube_crosses_it_in_two_circles_because_0_5_minus_0_4_rounds_short()
 {
    let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 0.5);
    let bend = Surface::Torus(
        Torus::new(
            Axis::new(Point::new(0.0, 0.0, 16.15), Vector::new(0.0, 0.0, -1.0))
                .expect("a direction"),
            0.4,
            0.1,
        )
        .expect("a ring wider than its tube"),
    );

    let met = intersect(
        &pipe,
        &bend,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a positive room, however thin, is two circles, not a refusal");
    let SurfaceMeet::Curves(runs) = &met else {
        panic!("a pipe just outside a bend's tube is two circles: {met:?}");
    };
    assert_eq!(runs.len(), 2, "{met:?}");
    for (run, offset) in runs
        .iter()
        .zip([-16.150_000_002_356_08, -16.149_999_997_643_917])
    {
        let EdgeRun::RightRoundWhereItCrosses {
            and: Surface::Plane(flat),
        } = run.run()
        else {
            panic!("each circle names its own plane: {run:?}");
        };
        assert_eq!(flat.normal(), Vector::new(0.0, 0.0, -1.0));
        assert_eq!(flat.offset(), offset);

        let up = -offset;
        let quarter = NamedRun::new(
            EdgeRun::PartOfWhereItCrosses {
                and: bend,
                picked_by: smallvec![Picker::from(
                    HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.5)
                        .expect("a normal with a length")
                )],
            },
            [[0.5, 0.0, up], [0.0, 0.5, up]].map(rcad_predicates::Point::from),
        )
        .expect("a piece with two ends");
        for along in [[&pipe, &bend], [&bend, &pipe]] {
            let walked = walk(
                &quarter,
                along,
                Deflection::of(0.01).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("the circle at {up}: {why:?}"));
            let allowed = walked.deviation().magnitude();
            assert!(allowed * 1000.0 < 2.356e-9, "{allowed}");
            assert!(!walked.points().is_empty());
            for point in walked.points() {
                let [_, _, z] = point.coordinates();
                assert!(
                    (z - up).abs() <= allowed,
                    "a point of the circle at {up} is at {z}"
                );
            }
        }
    }

    let whole = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed curve is never a piece with nothing picking it");
    for along in [[&pipe, &bend], [&bend, &pipe]] {
        assert_eq!(
            walk(
                &whole,
                along,
                Deflection::of(0.01).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .map(|walked| walked.points().len()),
            Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface),
            "a whole circle named by the two faces alone is refused, whichever face asks"
        );
        assert_eq!(
            where_a_closed_crossing_starts(&whole, along, &mut ProvenanceLog::new()),
            Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface),
            "and so is the one place a vertex could go on it"
        );
    }
}

/// An arc of a circle a ring shares with a pipe, a cone or a ball on its axis,
/// between two corners on it and picked by the half-space through its chord,
/// is walked on both surfaces whichever face is listed first: the quarter one
/// side of the chord keeps, or the three quarters the other side keeps. Row
/// Q1396 (§378.9); before it `walk` had no row for a ring against a curved
/// surface and refused every such arc
/// `ACrossingBetweenATorusAndAnotherCurvedSurface`.
///
/// The circles are `a_ring_face_bounded_by_a_circle_on_a_coaxial_surface.rs`'s,
/// on the ring of ten and five, and the two the bar of four crosses the bend of
/// four and one in, which `how_often_a_fuse_with_a_torus_refuses.rs` fuses.
#[test]
fn an_arc_of_a_circle_a_ring_shares_with_a_surface_on_its_axis_is_walked_on_both() {
    let bend = a_bend([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 4.0, 1.0);
    let ring = a_bend([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 10.0, 5.0);
    let cone = a_cone([0.0, 0.0, -1.0], [0.0, 0.0, 9.0], 1.0);
    let ball = a_ball([0.0, 0.0, 0.0], 185f64.sqrt());
    for (torus, partner, (out, up)) in [
        (
            bend,
            a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 4.0),
            (4.0, 1.0),
        ),
        (
            bend,
            a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 4.0),
            (4.0, -1.0),
        ),
        (
            ring,
            a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 13.0),
            (13.0, -4.0),
        ),
        (
            ring,
            a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 7.0),
            (7.0, 4.0),
        ),
        (ring, cone, (13.0, -4.0)),
        (ring, cone, (6.0, 3.0)),
        (ring, ball, (13.0, -4.0)),
        (ring, ball, (13.0, 4.0)),
    ] {
        for (facing, quarter) in [(1.0, true), (-1.0, false)] {
            let run = NamedRun::new(
                EdgeRun::PartOfWhereItCrosses {
                    and: partner,
                    picked_by: smallvec![Picker::from(
                        HalfSpace::new(Vector::new(facing, facing, 0.0), facing * out)
                            .expect("a normal with a length")
                    )],
                },
                [[out, 0.0, up], [0.0, out, up]].map(rcad_predicates::Point::from),
            )
            .expect("a piece with two ends");
            for along in [[&torus, &partner], [&partner, &torus]] {
                let walked = walk(
                    &run,
                    along,
                    Deflection::of(0.01).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{partner:?} at {up}: {why:?}"));
                let allowed = walked.deviation().magnitude();
                assert!(!walked.points().is_empty(), "{partner:?} at {up}");
                for point in walked.points() {
                    let [x, y, z] = point.coordinates();
                    let round = y.atan2(x);
                    let inside_the_quarter = round > 0.0 && round < core::f64::consts::FRAC_PI_2;
                    assert_eq!(
                        inside_the_quarter, quarter,
                        "{partner:?} at {up}: {point:?}"
                    );
                    for surface in [&torus, &partner] {
                        assert!(
                            how_far_off(surface, *point) <= allowed,
                            "{partner:?} at {up}: {point:?} is {} off {surface:?}, allowed {allowed}",
                            how_far_off(surface, *point)
                        );
                    }
                    assert!((z - up).abs() <= allowed, "{partner:?} at {up}: {point:?}");
                }
            }
        }
    }
}

/// How far `point` is from `surface`, for the surfaces these tests stand on
/// the `z` axis or round it.
fn how_far_off(surface: &Surface, point: Point) -> f64 {
    let [x, y, z] = point.coordinates();
    match surface {
        Surface::Cylinder(pipe) => (x.hypot(y) - pipe.radius()).abs(),
        Surface::Sphere(ball) => ((point - ball.centre()).length() - ball.radius()).abs(),
        Surface::Torus(ring) => {
            let [_, _, height] = ring.centre().coordinates();
            ((x.hypot(y) - ring.ring_radius()).hypot(z - height) - ring.tube_radius()).abs()
        }
        Surface::Cone(cone) => {
            let [_, _, apex] = cone.apex().coordinates();
            let slope = cone.slope();
            (x.hypot(y) - slope * (z - apex).abs()).abs() / slope.hypot(1.0)
        }
        Surface::Plane(flat) => panic!("nothing here stands {flat:?} round the z axis"),
        other => panic!("a kind of surface this file does not know: {other:?}"),
    }
}

/// The same for a cone, which the naming half refuses under its own name.
///
/// A countersink met by a pipe: two faces a file can state and this crate can
/// never name a crossing for.
#[test]
fn a_countersink_meeting_a_pipe_is_refused_by_name_rather_than_crashing_the_evaluator() {
    let countersink = Surface::Cone(rcad_geom::Cone::new(Axis::z(), 1.0).expect("a slope"));
    let pipe = a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 2.0], 0.5);

    assert_eq!(
        intersect(
            &countersink,
            &pipe,
            &Policy::certified(),
            &mut ProvenanceLog::new()
        ),
        Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
    );

    let whole = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed curve is never a piece with nothing picking it");
    for along in [[&countersink, &pipe], [&pipe, &countersink]] {
        assert_eq!(
            walk(
                &whole,
                along,
                Deflection::of(0.01).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .map(|walked| walked.points().len()),
            Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
        );
        assert_eq!(
            where_a_closed_crossing_starts(&whole, along, &mut ProvenanceLog::new()),
            Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
        );
    }
}

/// A cone, for the sweep below.
fn a_cone(along: [f64; 3], apex: [f64; 3], slope: f64) -> Surface {
    Surface::Cone(
        rcad_geom::Cone::new(
            Axis::new(Point::from(apex), Vector::from(along)).expect("a direction"),
            slope,
        )
        .expect("a slope"),
    )
}

/// A bend of pipe, for the sweep below.
fn a_bend(along: [f64; 3], centre: [f64; 3], ring: f64, tube: f64) -> Surface {
    Surface::Torus(
        Torus::new(
            Axis::new(Point::from(centre), Vector::from(along)).expect("a direction"),
            ring,
            tube,
        )
        .expect("a ring wider than its tube"),
    )
}

/// All twenty-five ordered pairs of the five kinds of surface, past the naming
/// half and both halves of the evaluator.
///
/// Twelve of them are refused, and what this checks is that the namer, the
/// walker and the starter all say so about the same twelve, and under the same
/// name for each.
///
/// **One of the twelve is refused on the kind of surface**, before a number is
/// looked at: the bend against a second bend.
///
/// **The other eleven are refused on where the two stand**: a cone against a
/// pipe, a ball or a second cone, and a torus against a ball, a pipe or a
/// cone. Each of those pairs crosses in circles and is named — by `intersect`
/// and, since row Q1396 for a torus, walked — where the two share an axis, so
/// every partner below stands off the other's axis, which is what makes it a
/// refusal at all. A cone and a pipe that share an axis are
/// `where_a_coaxial_cone_and_pipe_cross.rs`'s half. Reading this row as "these
/// kinds are refused" would state something the dispatch contradicts, and the
/// pair that used to sit here was coaxial and stopped being refused underneath
/// the sweep.
///
/// **A sweep and not a row per pair**, because what went wrong was the *list*.
/// The two lists were kept in step by hand, a cone and a torus landed and were
/// added to neither, and those twelve pairs crashed the evaluator where the
/// namer refused them by name — reached from `Import::read` on a real KiCad file
/// (§24.5). A list is only checked by walking all of it.
///
/// Three pairs cross in straight lines and are not on the twelve. They have
/// their own row, `a_closed_run_a_caller_names_on_a_crossing_of_straight_lines_is_refused_by_name`,
/// because they are not refused *on kind* — the same two kinds cross in a conic
/// as often as in a line, and which one it is depends on the numbers.
#[test]
fn the_namer_and_the_evaluator_refuse_the_same_twelve_pairs_under_the_same_name() {
    let one = [
        a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
        a_ball([0.0, 0.0, 0.0], 3.0),
        a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 1.0),
        a_cone([0.0, 0.0, 1.0], [0.0, 0.0, -5.0], 1.0),
        a_bend([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 3.0, 1.5),
    ];
    let other = [
        a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]),
        a_ball([1.0, 0.0, 0.0], 3.0),
        a_pipe([1.0, 0.0, 1.0], [0.0, 0.0, 0.0], 0.7),
        a_cone([0.0, 0.0, 1.0], [1.0, 0.0, 5.0], 1.0),
        a_bend([0.0, 0.0, 1.0], [0.5, 0.0, 1.0], 3.0, 1.5),
    ];
    let curved_refusals = [
        Error::ACrossingBetweenAConeAndAnotherCurvedSurface,
        Error::ACrossingBetweenATorusAndAnotherCurvedSurface,
    ];

    let whole = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed curve is never a piece with nothing picking it");
    let mut refused = 0;
    for first in &one {
        for second in &other {
            let named = intersect(
                first,
                second,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            let walked = walk(
                &whole,
                [first, second],
                Deflection::of(0.01).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .map(|walked| walked.points().len());
            let started =
                where_a_closed_crossing_starts(&whole, [first, second], &mut ProvenanceLog::new());

            match curved_refusals
                .iter()
                .find(|name| named == Err((*name).clone()))
            {
                Some(name) => {
                    refused += 1;
                    assert_eq!(
                        walked,
                        Err(name.clone()),
                        "the namer refuses {first:?} against {second:?} by name and walking it does not"
                    );
                    assert_eq!(
                        started,
                        Err(name.clone()),
                        "the namer refuses {first:?} against {second:?} by name and starting it does not"
                    );
                }
                None => {
                    for name in &curved_refusals {
                        assert_ne!(
                            walked,
                            Err(name.clone()),
                            "walking {first:?} against {second:?} refuses by name and the namer does not"
                        );
                        assert_ne!(
                            started,
                            Err(name.clone()),
                            "starting {first:?} against {second:?} refuses by name and the namer does not"
                        );
                    }
                }
            }
        }
    }
    assert_eq!(
        refused, 12,
        "one pair refused on kind, plus eleven refused for standing off one another's axis, is \
         twelve of the twenty-five"
    );
}

/// The three pairs that cross in straight lines, walked as a closed run a caller
/// named on them.
///
/// `where_a_plane_cuts_a_pipe` asserts that a plane never runs down a pipe's own
/// axis, and the two-pipe conic builder is never asked for a pair pointing the
/// same way. Both are true, and this is why: `what_a_run_between_these_two_lies_on`
/// answers `StraightLine` for all three *before* it reaches either builder, on
/// the same `f64` expression the assert reads. This walks the three pairs in
/// through the two public doors so the guard is exercised rather than argued
/// for — the same demand that turned four plane-and-cone panics into names.
#[test]
fn a_closed_run_a_caller_names_on_a_crossing_of_straight_lines_is_refused_by_name() {
    let down_the_axis = a_plane([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
    let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);

    for pair in [
        [
            a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
            a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 4.0]),
        ],
        [down_the_axis, pipe],
        [pipe, down_the_axis],
        [pipe, a_pipe([0.0, 0.0, 1.0], [5.0, 0.0, 0.0], 2.0)],
    ] {
        let [first, second] = [&pair[0], &pair[1]];
        let whole = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it");

        assert_eq!(
            where_a_closed_crossing_starts(&whole, [first, second], &mut ProvenanceLog::new()),
            Err(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed),
            "{first:?} against {second:?}"
        );
        // Walking one is not a refusal: a straight run is its own two ends, and
        // a whole-curve run between two of these has no ends to put down.
        let walked = walk(
            &whole,
            [first, second],
            Deflection::of(0.01).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a straight run is walked by its ends and never reaches a conic builder");
        assert!(
            walked.points().is_empty(),
            "{first:?} against {second:?} put points on a crossing with no curve in it"
        );
    }
}
