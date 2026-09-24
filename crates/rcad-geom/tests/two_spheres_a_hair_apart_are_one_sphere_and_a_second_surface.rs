//! A rim between two bands of one ball, where the ball is written down twice.
//!
//! The faces either side of such a rim are the same sphere, so the pair either
//! side names nothing at all and the run carries the second surface itself —
//! exactly a seam's shape. `the_two_surfaces_it_runs_between` used to find that
//! case by asking `along[0] == along[1]`, which is equality on the stored
//! `f64`s, so a model that writes the one ball twice with 1e-15 between the two
//! spellings slipped past it and handed `[Sphere, Sphere]` to
//! `where_two_balls_cut`.
//!
//! That pair **answers**, and the answer is the wrong circle. With one radius
//! `R` and a gap `D` between the two centres, `squared_gap = D²` and
//! `along_the_centre_line = D² + R² − R² = D²`, so the circle sits halfway
//! between the two spellings and comes out with the ball's own radius: a great
//! circle through the ball's own centre, nowhere near the rim the run named.
//! `docs/decisions.md` §218.6, and §176.2 is the same shape one surface down on
//! a pair of planes.
//!
//! **The wrong place is the whole of the test**, which is why the two walking
//! tests below assert where the circle is rather than that something came back.
//! Before the fix this run answered with seven points nowhere in particular:
//! measured, they stood 2.0497 to 3.6531 from a centre they should stand
//! exactly 4.0 from, and their `z` ran from 3.4425 to 4.9664 on a rim that lies
//! flat at `z = 3`. The control below, one ball written once, walked the same
//! run into eleven points every one of which is on the rim.
//!
//! The last two tests are the other end of the same road: two balls with their
//! centres in exactly the same place, which is the case that divides by nothing.

#![forbid(unsafe_code)]

use rcad_geom::{
    Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Point, Sphere, Surface, Vector, walk,
    where_a_closed_crossing_starts,
};
use rcad_predicates::{Policy, ProvenanceLog};
use smallvec::smallvec;

/// The ball whose skin both faces of the rim are on.
fn the_ball() -> Surface {
    a_ball([0.0, 0.0, 0.0], 5.0)
}

/// The same ball, written down with its centre 1e-15 along `x`.
///
/// One ulp of 5.0 is about 8.9e-16, so this is a little over a single step of
/// the number line away from `the_ball` and is the one ball by any reading but
/// a bitwise one.
fn the_same_ball_written_again() -> Surface {
    a_ball([1e-15, 0.0, 0.0], 5.0)
}

fn a_ball(centre: [f64; 3], radius: f64) -> Surface {
    Surface::Sphere(Sphere::new(Point::from(centre), radius).expect("a radius"))
}

/// The second ball the rim is really where the first one meets.
///
/// Radius 5 with its centre 6 up the `z` axis, so the two balls cross in the
/// circle at `z = 3` of radius 4 — the rim every walking test below asks for.
fn the_ball_that_cuts_it() -> Surface {
    a_ball([0.0, 0.0, 6.0], 5.0)
}

const WHERE_THE_RIM_IS: [f64; 3] = [0.0, 0.0, 3.0];
const HOW_BIG_THE_RIM_IS: f64 = 4.0;

/// A quarter-turn of that rim, said the way a reader says one: the second
/// surface in `and`, and one half-space to pick which of the two arcs between
/// the ends this is.
fn a_quarter_of_the_rim() -> NamedRun {
    let facing_the_short_way = HalfSpace::through_a_point_facing(
        Point::from(WHERE_THE_RIM_IS),
        Vector::new(1.0, 1.0, 0.0),
    )
    .expect("that direction has length");
    NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: the_ball_that_cuts_it(),
            picked_by: smallvec![Picker::from(facing_the_short_way)],
        },
        [
            rcad_predicates::Point::from([4.0, 0.0, 3.0]),
            rcad_predicates::Point::from([0.0, 4.0, 3.0]),
        ],
    )
    .expect("a picked run does not have to come back to where it started")
}

/// Every point walked, checked against the circle the run named.
fn every_point_is_on_the_rim(along: [&Surface; 2], what: &str) {
    let walked = walk(
        &a_quarter_of_the_rim(),
        along,
        Deflection::of(0.01).expect("a length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .unwrap_or_else(|refused| panic!("{what} is a rim this kernel can walk, not {refused:?}"));

    assert!(
        walked.points().len() > 8,
        "{what}: a quarter of a circle of radius 4 at a deflection of 0.01 takes more than eight \
         points, not {}",
        walked.points().len()
    );
    let middle = Point::from(WHERE_THE_RIM_IS);
    for point in walked.points() {
        let off_the_plane = (point.coordinates()[2] - WHERE_THE_RIM_IS[2]).abs();
        let off_the_radius = ((*point - middle).length() - HOW_BIG_THE_RIM_IS).abs();
        assert!(
            off_the_plane < 1e-12 && off_the_radius < 1e-12,
            "{what}: {point:?} is {off_the_plane} off the plane the rim lies in and \
             {off_the_radius} off its radius of {HOW_BIG_THE_RIM_IS}"
        );
    }
}

/// The defect: one ball written twice, a hair apart.
#[test]
fn a_ball_written_twice_a_hair_apart_walks_the_rim_its_run_names() {
    every_point_is_on_the_rim(
        [&the_ball(), &the_same_ball_written_again()],
        "one ball written twice 1e-15 apart",
    );
}

/// The control, so the test above is about the 1e-15 and not about the run.
#[test]
fn the_same_run_with_the_ball_written_once_walks_the_same_rim() {
    every_point_is_on_the_rim([&the_ball(), &the_ball()], "one ball written once");
}

/// And the vertex on the closed rim goes to the same place either way, which is
/// what `fuse(A, B)` and `fuse(B, A)` agreeing rests on.
#[test]
fn the_closed_rim_starts_in_the_same_place_however_the_ball_is_spelt() {
    let right_round = NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
        and: the_ball_that_cuts_it(),
        picked_by: smallvec![Picker::from(
            HalfSpace::through_a_point_facing(
                Point::from(WHERE_THE_RIM_IS),
                Vector::new(0.0, 0.0, 1.0),
            )
            .expect("the z axis has length")
        )],
    })
    .expect("a run with a half-space on it does not need ends");

    let written_once = where_a_closed_crossing_starts(
        &right_round,
        [&the_ball(), &the_ball()],
        &mut ProvenanceLog::new(),
    );
    let written_twice = where_a_closed_crossing_starts(
        &right_round,
        [&the_ball(), &the_same_ball_written_again()],
        &mut ProvenanceLog::new(),
    );
    assert_eq!(written_once, written_twice);
    assert_eq!(
        written_once.expect("the rim is there"),
        Point::from([0.0, 4.0, 3.0]),
        "the vertex goes on the rim at z = 3, not on a great circle through the ball's centre"
    );
}

/// Two balls that really are the one ball, asked where they cross.
///
/// There is no answer: the two share every point of their skin and cross
/// nowhere. What the arithmetic did about it was divide nothing by nothing,
/// which put a `NaN` in every coordinate of the circle's middle and came back as
/// `NotFinite { what: "point on a plane" }` — a refusal about the radical plane
/// rather than about the pair that has none (`docs/decisions.md` §218.6).
#[test]
fn two_balls_that_are_the_one_ball_are_refused_by_name_rather_than_dividing_by_nothing() {
    let run = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a run right round its faces' crossing needs nothing picking it");
    let ball = the_ball();

    assert_eq!(
        walk(
            &run,
            [&ball, &ball],
            Deflection::of(0.01).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|walked| walked.points().len()),
        Err(Error::TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing),
    );
    assert_eq!(
        where_a_closed_crossing_starts(&run, [&ball, &ball], &mut ProvenanceLog::new()),
        Err(Error::TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing),
        "and there is nowhere on a circle that is not there for a vertex to go",
    );
}

/// The other pair with nothing between its centres, which is a different thing
/// and gets the name it already had.
///
/// Two balls round one centre with two radii are two skins that never touch, so
/// what they have no crossing in is a *conic* — the same refusal a plane that
/// misses the ball it names gets.
#[test]
fn two_balls_round_one_centre_with_two_radii_do_not_cross_in_a_circle_either() {
    let run = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a run right round its faces' crossing needs nothing picking it");

    assert_eq!(
        walk(
            &run,
            [&the_ball(), &a_ball([0.0, 0.0, 0.0], 3.0)],
            Deflection::of(0.01).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|walked| walked.points().len()),
        Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
    );
}
