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
//! | coplanar faces, one dimension up | a surface against its own copy | `SameSurface` |
//! | closed periodic surfaces | a plane through a whole ball or pipe | one closed curve, named with no seam in it |
//! | seam-crossing intersections | walking a crossing that runs across a face's seam | 3D points, and the seam never comes into it |
//! | tangent sphere–cylinder | **refused**, and named: the second stage's |
//! | tangent cylinder–cylinder | **refused**, and named: the second stage's |
//! | two lines no `f64` half-space separates | **refused**, and it is a gap |
//!
//! # Why tangency is an answer and not a band
//!
//! For two quadrics whose coefficients are exactly representable, "are they
//! tangent" is the sign of a discriminant, so level 3 of the cascade decides it
//! exactly (§24.10). There is no near-tangency band to widen or narrow, and
//! nothing in this file needs a tolerance anywhere.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Error, NamedRun, Plane, Point, Sphere, Surface,
    SurfaceMeet, Vector, intersect, walk,
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
        &NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross),
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
    let facing_x = a_plane([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);

    let seam = NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: meridian,
            picked_by: smallvec![facing_x],
        },
        [
            rcad_predicates::Point::from([0.0, 0.0, radius]),
            rcad_predicates::Point::from([0.0, 0.0, -radius]),
        ],
    );

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
            rcad_predicates::Interval::new(coordinate - slack, coordinate + slack)
                .expect("a bound is a length")
        }));
        for surface in [&ball, &meridian] {
            assert_ne!(side_of(surface, at), Decision::Pos, "off {surface:?}");
            assert_ne!(side_of(surface, at), Decision::Neg, "off {surface:?}");
        }
        assert_ne!(
            side_of(&facing_x, at),
            Decision::Neg,
            "outside the half-space that picked this half of the meridian: {point:?}"
        );
    }
}

/// The rows the closed form does not answer, said out loud.
///
/// A ball against a pipe and a pipe against a pipe cross in a degree-four space
/// curve, whose sign questions run to the eighth and twelfth power and need an
/// expansion buffer M3's second stage builds (§27.1). **They come back naming
/// what they are** rather than answering something that looks like an answer,
/// which is the difference between a deferral and a wrong result.
#[test]
fn the_two_pairs_that_run_to_a_quartic_refuse_by_name() {
    let ball = a_ball([0.0, 0.0, 0.0], 3.0);
    let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 3.0);
    for (first, second, named) in [
        (&ball, &pipe, ("ball", "pipe")),
        (&pipe, &pipe, ("pipe", "pipe")),
    ] {
        assert_eq!(
            intersect(
                first,
                second,
                &Policy::certified(),
                &mut ProvenanceLog::new()
            ),
            Err(Error::CrossingRunsToAQuartic {
                first: named.0,
                second: named.1
            })
        );
    }
}

/// The one refusal the closed form has that is not a deferral, and it is a
/// **gap**: the two lines are there, and what runs out is `f64`'s supply of unit
/// vectors exactly square to a given direction.
///
/// The same configuration on the lattice, where that direction is a coordinate
/// axis, is named without complaint — which is the demonstration `CLAUDE.md`
/// asks for in place of a paragraph.
#[test]
fn two_lines_no_half_space_can_separate_refuse_and_the_same_pair_on_the_lattice_does_not() {
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
    let cut = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.2, 0.0, 0.0), normal).expect("a unit normal"),
    );
    assert_eq!(
        intersect(
            &cut,
            &leaning,
            &Policy::certified(),
            &mut ProvenanceLog::new()
        ),
        Err(Error::CannotNameWhichPieceOfACrossingThisIs)
    );

    let met = crossing(
        &a_plane([1.0, 0.0, 0.0], [0.2, 0.0, 0.0]),
        &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
    );
    let SurfaceMeet::Curves(runs) = &met else {
        panic!("on the lattice this cuts two lines: {met:?}");
    };
    assert_eq!(runs.len(), 2);
}
