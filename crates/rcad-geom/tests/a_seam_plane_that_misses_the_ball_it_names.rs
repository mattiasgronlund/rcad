//! A run that says a ball's seam lies on a plane the ball does not reach.
//!
//! A model states which plane a face's seam is cut by, and nothing checks that
//! the plane goes anywhere near the face — `Model::validate` cannot see
//! geometric coincidence at all. So a ball of radius 5 sitting 512 away from the
//! plane its seam names is a run a stranger's file can state, and
//! `where_a_plane_cuts_a_ball` answers it with a **circle of no radius**: the
//! discriminant comes out negative and clamps to zero.
//!
//! There is no angle round a circle of no radius. `Conic::angle_of` divides by a
//! semi-axis's own squared length, so both ends of the run come back `0/0`, and
//! `Conic::at` then puts a `NaN` in every coordinate of the point the
//! half-spaces are asked about. That point used to reach interval arithmetic,
//! where `Interval::new(NaN, NaN)` was refused by an `expect` reasoning about
//! the *sign* of a deviation — a claim that said nothing about `NaN` at all.
//! A two-hour `cargo fuzz` soak of `Model::fuse` found it through
//! `Model::validate`.
//!
//! The bytes were a stranger's, so the answer is an `Err` and not a panic
//! (CLAUDE.md, "Panic on broken internal invariants. Return `Err` on bad caller
//! input"), and the name is the one that says what is wrong with the run rather
//! than what the arithmetic did about it.

#![forbid(unsafe_code)]

use rcad_geom::{
    Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point, Sphere, Surface, Vector,
    walk, where_a_closed_crossing_starts,
};
use rcad_predicates::{Policy, ProvenanceLog};
use smallvec::smallvec;

/// A ball's seam, stated as lying where the ball meets `meridian`.
///
/// The two ends are the ball's own poles, which is where a real seam's ends are.
fn a_seam_cut_by(meridian: Surface, centre: [f64; 3], radius: f64) -> NamedRun {
    let facing_x =
        HalfSpace::through_a_point_facing(Point::from(centre), Vector::new(1.0, 0.0, 0.0))
            .expect("the x axis has length");
    NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: meridian,
            picked_by: smallvec![Picker::from(facing_x)],
        },
        [
            rcad_predicates::Point::from([centre[0], centre[1], centre[2] + radius]),
            rcad_predicates::Point::from([centre[0], centre[1], centre[2] - radius]),
        ],
    )
    .expect("a picked run does not have to come back to where it started")
}

fn a_ball(centre: [f64; 3], radius: f64) -> Surface {
    Surface::Sphere(Sphere::new(Point::from(centre), radius).expect("a radius"))
}

/// The plane through the origin square to `y`.
fn the_plane_through_the_origin() -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
            .expect("the y axis is a unit normal"),
    )
}

/// The shape the soak found, at the second reproducer's own numbers.
#[test]
fn a_ball_of_radius_5_sitting_512_from_the_plane_its_seam_names_has_no_circle_to_walk() {
    let centre = [0.0, 512.0, 0.0];
    let radius = 5.0;
    let ball = a_ball(centre, radius);
    let seam = a_seam_cut_by(the_plane_through_the_origin(), centre, radius);

    assert_eq!(
        walk(
            &seam,
            [&ball, &ball],
            Deflection::of(0.01).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|walked| walked.points().len()),
        Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
        "a plane 512 away from a ball of radius 5 cuts no circle in it"
    );

    assert_eq!(
        where_a_closed_crossing_starts(&seam, [&ball, &ball], &mut ProvenanceLog::new()),
        Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
        "and there is nowhere on that circle for a vertex to go either"
    );
}

/// The neighbouring shape, which reaches the same circle of no radius by the
/// one road where the discriminant is not clamped but genuinely zero.
///
/// A plane resting on a ball touches it at one point. `intersect` calls that
/// `SurfaceMeet::TouchAtOnePoint` and never hands back a run, so a run saying
/// otherwise is a caller's word against the geometry, exactly as above.
#[test]
fn a_plane_resting_on_a_ball_touches_it_at_one_point_and_that_is_not_a_circle() {
    let radius = 3.0;
    let centre = [0.0, radius, 0.0];
    let ball = a_ball(centre, radius);
    let seam = a_seam_cut_by(the_plane_through_the_origin(), centre, radius);

    assert_eq!(
        walk(
            &seam,
            [&ball, &ball],
            Deflection::of(0.01).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|walked| walked.points().len()),
        Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
        "a plane the ball rests on cuts a circle of no radius, which is a point"
    );
}

/// The same ball with its seam where a seam really goes, so the refusal above is
/// about the plane missing and not about the shape of the run.
#[test]
fn the_same_seam_through_the_ball_itself_is_walked_without_complaint() {
    let radius = 5.0;
    let centre = [0.0, 0.0, 0.0];
    let ball = a_ball(centre, radius);
    let seam = a_seam_cut_by(the_plane_through_the_origin(), centre, radius);

    let walked = walk(
        &seam,
        [&ball, &ball],
        Deflection::of(0.01).expect("a length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a ball this size is walked without complaint");
    assert!(walked.points().len() > 8);
}
