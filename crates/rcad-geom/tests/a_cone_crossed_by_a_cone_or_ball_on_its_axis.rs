//! A cone crossed by a second cone or by a ball that shares its axis: circles
//! square to that axis, named by `intersect` and drawn by `walk`.
//!
//! A cone is both nappes, so two cones on one axis meet where
//! `s₁·|h| = s₂·|h − d|` — two circles, one halfway between the apexes where the
//! slopes are one, and only the apex where the apexes are one point. A ball
//! centred on the axis meets the cone where `(1 + s²)h² − 2eh + e² − R² = 0`:
//! two circles, one it touches all the way round, or none. Every height below is
//! that closed form worked out by hand, and every point walked is checked
//! against **both** surfaces with the bound the walker publishes for it.
//!
//! Two cones on one apex whose axes lean apart are the one pair here off a
//! shared axis that is named: they cross in straight lines through the apex.
//!
//! What stays refused is a pair off the axis, and a ball whose skin runs through
//! the apex, which crosses the cone in a circle and the apex apart from it.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Error, NamedRun, Picker, Point, Sphere, Surface, SurfaceMeet,
    Vector, intersect, walk, where_a_closed_crossing_starts,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};

fn a_cone(apex: [f64; 3], along: [f64; 3], slope: f64) -> Surface {
    Surface::Cone(
        Cone::new(
            Axis::new(Point::from(apex), Vector::from(along)).expect("a direction"),
            slope,
        )
        .expect("a slope above zero"),
    )
}

/// The cone of slope one about `+z` with its apex at the origin.
fn the_right_angled_cone() -> Surface {
    a_cone([0.0; 3], [0.0, 0.0, 1.0], 1.0)
}

fn a_ball(centre: [f64; 3], radius: f64) -> Surface {
    Surface::Sphere(Sphere::new(Point::from(centre), radius).expect("a radius above zero"))
}

/// What `intersect` says of the pair, which is the same whichever is handed over
/// first.
fn named(one: &Surface, other: &Surface) -> Result<SurfaceMeet, Error> {
    let [said, turned] = [[one, other], [other, one]].map(|[first, second]| {
        intersect(
            first,
            second,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
    });
    assert_eq!(said, turned, "the pair is not written in a fixed order");
    said
}

/// Every circle the pair crosses in, walked both ways round and started both
/// ways round, as `(height, radius)` read off where it starts.
///
/// Each point walked is checked against both surfaces with the deviation the
/// walk itself publishes, so this is a check of that bound and not of a second
/// copy of the circle (§24.3).
fn the_circles_walked(one: &Surface, other: &Surface) -> Vec<(f64, f64)> {
    let named = named(one, other).expect("a pair on one axis is named");
    let SurfaceMeet::Curves(runs) = named else {
        panic!("circles, and not {named:?}");
    };
    runs.iter()
        .map(|run| {
            for along in [[one, other], [other, one]] {
                let walked = walk(
                    run,
                    along,
                    Deflection::of(0.01).expect("a length above zero"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a circle is walked without complaint");
                assert!(
                    walked.points().len() > 8,
                    "a circle at a deflection of a hundredth takes more than eight pieces"
                );
                let room = walked.deviation().magnitude();
                for point in walked.points() {
                    for surface in along {
                        let off = how_far_off(surface, *point);
                        assert!(
                            off <= room,
                            "{point:?} is {off:e} off {surface:?}, past the published {room:e}"
                        );
                    }
                }
            }
            let [start, turned] = [[one, other], [other, one]].map(|along| {
                where_a_closed_crossing_starts(run, along, &mut ProvenanceLog::new())
                    .expect("a circle has a start")
            });
            assert_eq!(
                start, turned,
                "where a circle starts does not move with the order"
            );
            (start.z(), start.x().hypot(start.y()))
        })
        .collect()
}

/// How far `point` is off a cone or a ball, as a plain distance.
///
/// The distance across a cone's axis is a cross product, for the reason
/// `walking_the_two_circles_a_cone_and_a_pipe_cut.rs` gives: the other form
/// quantizes a small miss to nothing.
fn how_far_off(surface: &Surface, point: Point) -> f64 {
    match surface {
        Surface::Cone(cone) => {
            let along = cone.axis().direction();
            let out = point - cone.apex();
            (out.cross(along).length() - cone.slope() * out.dot(along).abs()).abs()
                / (along.length() * cone.slope().hypot(1.0))
        }
        Surface::Sphere(ball) => ((point - ball.centre()).length() - ball.radius()).abs(),
        Surface::Plane(_) | Surface::Cylinder(_) | Surface::Torus(_) | _ => {
            panic!("this file crosses cones and balls, not {surface:?}")
        }
    }
}

/// Each `(height, radius)` against the one worked out by hand, in order.
fn assert_the_circles_are(walked: &[(f64, f64)], expected: &[(f64, f64)]) {
    assert_eq!(
        walked.len(),
        expected.len(),
        "{walked:?} against {expected:?}"
    );
    for (&(height, radius), &(wanted_height, wanted_radius)) in walked.iter().zip(expected) {
        assert!(
            (height - wanted_height).abs() <= 1e-12 * wanted_height.abs().max(1.0)
                && (radius - wanted_radius).abs() <= 1e-12 * wanted_radius.max(1.0),
            "{walked:?} against {expected:?}"
        );
    }
}

/// Slopes one and two, apexes nine apart: `|z| = 2|z − 9|` at `z = 6` and
/// `z = 18`, one between the apexes and one beyond the steeper cone's.
#[test]
fn two_cones_of_two_slopes_on_one_axis_cross_in_two_circles() {
    let steeper = a_cone([0.0, 0.0, 9.0], [0.0, 0.0, 1.0], 2.0);
    let walked = the_circles_walked(&the_right_angled_cone(), &steeper);
    let mut heights = walked.clone();
    heights.sort_by(|one, other| one.0.total_cmp(&other.0));
    assert_the_circles_are(&heights, &[(6.0, 6.0), (18.0, 18.0)]);
}

/// The same pair with the steeper cone's axis written pointing down, which is
/// the same double cone and so the same two circles.
#[test]
fn a_second_cone_whose_axis_is_written_backwards_crosses_the_first_in_the_same_circles() {
    let backwards = a_cone([0.0, 0.0, 9.0], [0.0, 0.0, -1.0], 2.0);
    let mut heights = the_circles_walked(&the_right_angled_cone(), &backwards);
    heights.sort_by(|one, other| one.0.total_cmp(&other.0));
    assert_the_circles_are(&heights, &[(6.0, 6.0), (18.0, 18.0)]);
}

/// One slope and two apexes ten apart: `|z| = |z − 10|` has the one root
/// `z = 5`, and the circle there is a whole closed run with nothing picking it.
#[test]
fn two_cones_of_one_slope_cross_in_the_one_circle_halfway_between_their_apexes() {
    let downwards = a_cone([0.0, 0.0, 10.0], [0.0, 0.0, -1.0], 1.0);
    assert_eq!(
        named(&the_right_angled_cone(), &downwards),
        Ok(SurfaceMeet::Curves(vec![
            NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
                .expect("a whole circle needs nothing picking it")
        ]))
    );
    assert_the_circles_are(
        &the_circles_walked(&the_right_angled_cone(), &downwards),
        &[(5.0, 5.0)],
    );
}

/// One apex and two slopes: the two nappes of the narrower cone sit inside the
/// wider one's all the way out, and the two meet at the apex alone — the
/// answer `intersect` already gives a plane through a cone's apex square to its
/// axis.
#[test]
fn two_cones_with_one_apex_and_two_slopes_touch_there_and_nowhere_else() {
    for [one, other] in [
        [
            the_right_angled_cone(),
            a_cone([0.0; 3], [0.0, 0.0, 1.0], 2.0),
        ],
        [
            a_cone([1.0, 2.0, 3.0], [0.0, 0.0, 1.0], 1.0),
            a_cone([1.0, 2.0, 3.0], [0.0, 0.0, -1.0], 0.5),
        ],
    ] {
        let Surface::Cone(cone) = one else {
            panic!("a cone");
        };
        assert_eq!(
            named(&one, &other),
            Ok(SurfaceMeet::TouchAtOnePoint {
                at: PredicatePoint::from(cone.apex().coordinates()),
            }),
            "{one:?} against {other:?}"
        );
    }
}

/// A ball centred one up the axis, radius three: the apex is inside it, so the
/// two circles are one in each nappe, at `z = (1 ± √17)/2`.
#[test]
fn a_ball_round_the_apex_crosses_the_cone_once_in_each_nappe() {
    let ball = a_ball([0.0, 0.0, 1.0], 3.0);
    let root = 17f64.sqrt();
    let mut heights = the_circles_walked(&the_right_angled_cone(), &ball);
    heights.sort_by(|one, other| one.0.total_cmp(&other.0));
    let [low, high] = [(1.0 - root) / 2.0, (1.0 + root) / 2.0];
    assert_the_circles_are(&heights, &[(low, low.abs()), (high, high)]);
}

/// A ball centred ten up, radius eight: it sits in the upper nappe and pushes
/// out through the side between `z = (10 − √28)/2` and `z = (10 + √28)/2`. Both
/// circles are below its centre and above the apex, so neither stored point
/// lies between them, and the half-space halfway between is what tells them
/// apart.
#[test]
fn a_ball_pushing_out_through_one_nappe_crosses_it_in_two_circles_on_one_side_of_its_centre() {
    let ball = a_ball([0.0, 0.0, 10.0], 8.0);
    let root = 28f64.sqrt();
    let mut heights = the_circles_walked(&the_right_angled_cone(), &ball);
    heights.sort_by(|one, other| one.0.total_cmp(&other.0));
    let [low, high] = [(10.0 - root) / 2.0, (10.0 + root) / 2.0];
    assert_the_circles_are(&heights, &[(low, low), (high, high)]);
}

/// A ball centred on the apex itself, radius two: `z = ±√2`, mirror images.
#[test]
fn a_ball_centred_on_the_apex_crosses_both_nappes_alike() {
    let ball = a_ball([0.0; 3], 2.0);
    let root = 2f64.sqrt();
    let mut heights = the_circles_walked(&the_right_angled_cone(), &ball);
    heights.sort_by(|one, other| one.0.total_cmp(&other.0));
    assert_the_circles_are(&heights, &[(-root, root), (root, root)]);
}

/// The ball of radius three centred five up a cone of slope three quarters is
/// inscribed in it: `(1 + s²)R² − s²e² = (25/16)·9 − (9/16)·25 = 0` exactly, so
/// the two touch all the way round the one circle at `z = 5/(25/16) = 3.2`,
/// radius `2.4`, which is one closed run, as a ball inscribed in a pipe is.
#[test]
fn a_ball_inscribed_in_a_cone_touches_it_all_the_way_round_one_circle() {
    let cone = a_cone([0.0; 3], [0.0, 0.0, 1.0], 0.75);
    let ball = a_ball([0.0, 0.0, 5.0], 3.0);
    assert_eq!(
        named(&cone, &ball),
        Ok(SurfaceMeet::Curves(vec![
            NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
                .expect("a whole circle needs nothing picking it")
        ]))
    );
    assert_the_circles_are(&the_circles_walked(&cone, &ball), &[(3.2, 2.4)]);
}

/// A ball of radius two centred four up sits inside the upper nappe, `4/√2`
/// from the side, and misses: nothing is named, and a closed run stated on the
/// pair names a circle that is not there, which both halves of the walker
/// refuse the way they refuse one stated on a plane that misses a ball.
#[test]
fn a_ball_inside_one_nappe_crosses_nothing_and_a_run_stated_on_it_is_refused() {
    let cone = the_right_angled_cone();
    let ball = a_ball([0.0, 0.0, 4.0], 2.0);
    assert_eq!(named(&cone, &ball), Ok(SurfaceMeet::Nothing));

    let stated = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a whole circle needs nothing picking it");
    for along in [[&cone, &ball], [&ball, &cone]] {
        assert_eq!(
            walk(
                &stated,
                along,
                Deflection::of(0.01).expect("a length above zero"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .err(),
            Some(Error::TheseTwoSurfacesDoNotCrossInAConic)
        );
        assert_eq!(
            where_a_closed_crossing_starts(&stated, along, &mut ProvenanceLog::new()).err(),
            Some(Error::TheseTwoSurfacesDoNotCrossInAConic)
        );
    }
}

/// A ball centred two up with radius two runs through the apex. The crossing is
/// then the circle at `z = 2` **and the apex apart from it**, and no answer
/// `intersect` has says both, so namer and walker refuse it by the row's name.
#[test]
fn a_ball_whose_skin_runs_through_the_apex_is_refused_by_name_in_both_halves() {
    let cone = the_right_angled_cone();
    let ball = a_ball([0.0, 0.0, 2.0], 2.0);
    assert_eq!(
        named(&cone, &ball),
        Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface)
    );

    let stated = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a whole circle needs nothing picking it");
    for along in [[&cone, &ball], [&ball, &cone]] {
        assert_eq!(
            where_a_closed_crossing_starts(&stated, along, &mut ProvenanceLog::new()).err(),
            Some(Error::ACrossingBetweenAConeAndAnotherCurvedSurface)
        );
    }
}

/// Two cones of slope one half on one apex, the second turned `0.3` about `x`,
/// share two rulings through the apex and nothing else, so the namer names the
/// four halves the apex cuts the two into and the walker says each is straight.
///
/// Where both axes are written along unit directions `w₁` and `w₂`, a point of
/// both cones is as far up one axis as up the other, so the two rulings lie in
/// the flat surface through the apex square to `w₁ − w₂`. That surface holds
/// the bisector `b = (0, −sin 0.15, cos 0.15)` and `x`, and a ruling `φ` off
/// `b` in it leans `α` off `w₁` where `cos φ · cos 0.15 = cos α`, for the
/// cone's half angle `α = atan ½`. So the four halves leave the apex along
/// `±(cos φ · b ± sin φ · x)`, and each of the four is held by one named half
/// and ruled out by the other three.
#[test]
fn two_cones_on_one_apex_leaning_apart_cross_in_the_four_halves_of_two_rulings() {
    let turn = 0.3_f64;
    let upright = a_cone([0.0; 3], [0.0, 0.0, 1.0], 0.5);
    let turned = a_cone([0.0; 3], [0.0, -turn.sin(), turn.cos()], 0.5);
    let met = named(&upright, &turned);
    let Ok(SurfaceMeet::Curves(runs)) = &met else {
        panic!("the two rulings are named: {met:?}");
    };
    assert_eq!(runs.len(), 4, "the four halves of two rulings: {runs:?}");

    let off_the_bisector = (0.5_f64.atan().cos() / (turn / 2.0).cos()).acos();
    let [up, across] = [off_the_bisector.cos(), off_the_bisector.sin()];
    let bisector = [0.0, -(turn / 2.0).sin(), (turn / 2.0).cos()];
    let mut halves = Vec::new();
    for way in [1.0, -1.0] {
        for side in [1.0, -1.0] {
            halves.push(Point::new(
                way * side * across,
                way * up * bisector[1],
                way * up * bisector[2],
            ));
        }
    }
    assert_these_are_the_halves_of_the_rulings(runs, &halves, [&upright, &turned]);
}

/// Two cones of slope two on one apex, about `z` and about `x`, are wide
/// enough that both flat surfaces the crossing lies in cut them, so they share
/// four rulings and the namer names the eight halves the apex cuts them into.
///
/// `x² + y² = 4z²` and `y² + z² = 4x²` take `x² = z²` between them, and then
/// `y² = 3x²`, so the eight halves leave the apex along `(±1, ±√3, ±1)`, each
/// sign on its own.
#[test]
fn two_wide_cones_on_one_apex_at_a_right_angle_cross_in_the_eight_halves_of_four_rulings() {
    let upright = a_cone([0.0; 3], [0.0, 0.0, 1.0], 2.0);
    let lying = a_cone([0.0; 3], [1.0, 0.0, 0.0], 2.0);
    let met = named(&upright, &lying);
    let Ok(SurfaceMeet::Curves(runs)) = &met else {
        panic!("the four rulings are named: {met:?}");
    };
    assert_eq!(runs.len(), 8, "the eight halves of four rulings: {runs:?}");

    let mut halves = Vec::new();
    for x in [1.0, -1.0] {
        for y in [3.0_f64.sqrt(), -3.0_f64.sqrt()] {
            for z in [1.0, -1.0] {
                halves.push(Point::new(x, y, z));
            }
        }
    }
    assert_these_are_the_halves_of_the_rulings(runs, &halves, [&upright, &lying]);
}

/// Two cones of slope one half on one apex at a right angle are each a quarter
/// turn wide and stand a quarter turn apart, so they share the apex and no
/// ruling at all.
#[test]
fn two_narrow_cones_on_one_apex_at_a_right_angle_touch_there_and_nowhere_else() {
    assert_eq!(
        named(
            &a_cone([0.0; 3], [0.0, 0.0, 1.0], 0.5),
            &a_cone([0.0; 3], [1.0, 0.0, 0.0], 0.5),
        ),
        Ok(SurfaceMeet::TouchAtOnePoint {
            at: PredicatePoint::from([0.0; 3])
        })
    );
}

/// Two cones of slope one on one apex at a right angle touch along the two
/// rulings `(1, 0, ±1)`: `x² + y² = z²` and `y² + z² = x²` take `y² = 0`, a
/// double root. The enclosure of how far apart the two rulings of that flat
/// surface spread holds nothing, so the pair is refused rather than named as
/// four halves that are really two — a **gap**: the two touching rulings are
/// there.
#[test]
fn two_cones_on_one_apex_that_touch_along_two_rulings_are_refused() {
    assert_eq!(
        named(
            &a_cone([0.0; 3], [0.0, 0.0, 1.0], 1.0),
            &a_cone([0.0; 3], [1.0, 0.0, 0.0], 1.0),
        ),
        Err(Error::CannotNameWhichPieceOfACrossingThisIs)
    );
}

/// Each of `halves` is a way out of the apex on both of `pair`, held by exactly
/// one of `runs`, every picker of which is a flat surface through the apex, and
/// ruled out by every other run; and every run is walked as a straight line.
fn assert_these_are_the_halves_of_the_rulings(
    runs: &[NamedRun],
    halves: &[Point],
    pair: [&Surface; 2],
) {
    for &out in halves {
        for cone in pair {
            let off = how_far_off(cone, out);
            assert!(off <= 1e-15, "{out:?} is {off:e} off {cone:?}");
        }
        let holding: Vec<&NamedRun> = runs
            .iter()
            .filter(|run| {
                let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                    panic!("a piece of the crossing: {run:?}");
                };
                let sides: Vec<Decision> = picked_by
                    .iter()
                    .map(|picker| {
                        assert!(
                            matches!(picker, Picker::InFrontOf(_)),
                            "a flat surface: {picker:?}"
                        );
                        picker.side_of_point(PredicatePoint::from(out.coordinates()))
                    })
                    .collect();
                assert!(
                    sides.iter().all(|sign| *sign == Decision::Pos)
                        || sides.contains(&Decision::Neg),
                    "{out:?} is held strictly or ruled out: {sides:?}"
                );
                sides.iter().all(|sign| *sign == Decision::Pos)
            })
            .collect();
        assert_eq!(holding.len(), 1, "one half holds {out:?}");
    }

    let [one, other] = pair;
    for run in runs {
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
            panic!("a piece of the crossing: {run:?}");
        };
        assert!(
            picked_by.iter().all(
                |picker| picker.side_of_point(PredicatePoint::from([0.0; 3])) == Decision::Zero
            ),
            "every picker passes through the apex: {picked_by:?}"
        );
        let from_the_apex = NamedRun::new(
            run.run().clone(),
            [
                PredicatePoint::from([0.0; 3]),
                PredicatePoint::from([0.0, 0.0, 1.0]),
            ],
        )
        .expect("two ends that name places");
        for along in [[one, other], [other, one]] {
            let walked = walk(
                &from_the_apex,
                along,
                Deflection::of(0.01).expect("a length above zero"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a ruling is walked as a straight line");
            assert!(
                walked.points().is_empty(),
                "nothing between the ends of a straight line"
            );
        }
    }
}

/// A second cone and a ball one to the side of the axis cross the first in a
/// degree-four curve and are refused by name, by the namer and by the walker.
#[test]
fn a_second_cone_or_a_ball_off_the_axis_is_refused_by_name_in_both_halves() {
    let cone = the_right_angled_cone();
    let stated = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a whole circle needs nothing picking it");
    for partner in [
        a_cone([1.0, 0.0, 9.0], [0.0, 0.0, 1.0], 2.0),
        a_cone([0.0, 0.0, 9.0], [1.0, 0.0, 1.0], 2.0),
        a_ball([1.0, 0.0, 1.0], 3.0),
    ] {
        assert_eq!(
            named(&cone, &partner),
            Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
            "{partner:?}"
        );
        for along in [[&cone, &partner], [&partner, &cone]] {
            assert_eq!(
                where_a_closed_crossing_starts(&stated, along, &mut ProvenanceLog::new()).err(),
                Some(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
                "{partner:?}"
            );
        }
    }
}

/// Two apexes one ulp apart along the diagonal, `1e150` out: the nearer circle
/// stands about `3e131` from the steeper apex, and one rounding of the picking
/// half-space's offset there is three orders bigger. The two circles exist and
/// are told apart by nothing `f64` can write down, so the pair is refused as
/// such — a **gap**, the one `two_circles_round_a_shared_axis` has. On `z`, the
/// same two apexes `1e300` out are named, because a dot product with a
/// coordinate direction never rounds.
#[test]
fn two_apexes_too_close_for_their_own_rounding_are_refused_rather_than_guessed() {
    let far = 1e150_f64;
    let diagonal = [1.0, 1.0, 1.0];
    let near_apex = a_cone([far; 3], diagonal, 1000.0);
    let next_apex = a_cone([far.next_up(); 3], diagonal, 1.0);
    assert_eq!(
        named(&near_apex, &next_apex),
        Err(Error::CannotNameWhichPieceOfACrossingThisIs)
    );

    let high = 1e300_f64;
    let on_z = [0.0, 0.0, 1.0];
    let named_on_z = named(
        &a_cone([0.0, 0.0, high], on_z, 1000.0),
        &a_cone([0.0, 0.0, high.next_up()], on_z, 1.0),
    )
    .expect("a coordinate direction reads the apex exactly");
    let SurfaceMeet::Curves(runs) = named_on_z else {
        panic!("two circles, and not {named_on_z:?}");
    };
    assert_eq!(runs.len(), 2);
}
