//! Whether each refusal of this crate is a limit, a gap or not known.
//!
//! `docs/decisions.md` §437.6. A **limit** says no answer this kernel can hold
//! exists; a **gap** says one exists and the arithmetic did not reach it; and
//! **not known** says nobody has shown either. Every variant that claims a
//! limit or a gap is reached here, through the public API, and asked which it
//! is. Not known needs no test, since it claims nothing.
//!
//! A gap is asserted only on a shape refused **every way round**: a pair that
//! refuses one way and answers the other proves nothing about the refusal
//! (§140.5).

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, BSplineCurve, Cone, Cylinder, Deflection, DynamicRange, EdgeRun, Error, Isometry,
    KnotVector, MAX_DEGREE, NamedRun, Plane, Point, Scale, Segment, Sphere, Surface, Torus, Unit,
    Vector, intersect, meet_of_three_surfaces, walk, where_a_closed_crossing_starts,
};
use rcad_predicates::{Decision, Policy, ProvenanceLog, SideOfSurface};

/// Numbers that describe no shape at all are limits: there is nothing for any
/// arithmetic to reach.
#[test]
fn numbers_that_describe_no_shape_are_limits() {
    let refusals = [
        Sphere::new(Point::origin(), 0.0).expect_err("a ball of no size"),
        Cylinder::new(Axis::z(), -1.0).expect_err("a pipe of negative size"),
        Cone::new(Axis::z(), 0.0).expect_err("a cone with no spread"),
        Cone::from_half_angle(Axis::z(), core::f64::consts::FRAC_PI_2)
            .expect_err("a cone opened out flat"),
        Torus::new(Axis::z(), 1.0, 1.0).expect_err("a tube as wide as its ring"),
        Deflection::of(0.0).expect_err("a deflection of nothing"),
        Segment::new(Point::origin(), Point::origin()).expect_err("a segment with one end"),
        Segment::new(Point::origin(), Point::new(1.0, 0.0, 0.0))
            .expect("a segment of length one")
            .point_at(2.0)
            .expect_err("a point past the end"),
        Scale::new(Unit::Millimetre, 0.0).expect_err("a model of no size"),
        DynamicRange::measure(1.0, 0.0).expect_err("a feature of no size"),
    ];
    let names = refusals.each_ref().map(name_of);
    assert_eq!(
        names,
        [
            "RadiusNotPositive",
            "RadiusNotPositive",
            "SlopeNotPositive",
            "HalfAngleIsNotACone",
            "ATorusThatRunsThroughItself",
            "DeflectionNotPositive",
            "SegmentHasNoLength",
            "ParameterOutsideSegment",
            "NominalSizeNotPositive",
            "FeatureSizeNotPositive",
        ]
    );
    for refusal in &refusals {
        assert_eq!(refusal.limit_or_gap().to_string(), "limit", "{refusal:?}");
    }
}

/// A model wider than its smallest feature by more than `f64` carries is a
/// limit of today's kernel, which holds every number as one `f64` (§437.6).
#[test]
fn a_model_too_wide_for_its_smallest_feature_is_a_limit() {
    let refusal = DynamicRange::measure(1e20, 1e-10)
        .expect("both sizes are positive and finite")
        .check()
        .expect_err("thirty orders of magnitude is past what f64 resolves");
    assert!(
        matches!(refusal, Error::DynamicRangeTooWide { .. }),
        "{refusal:?}"
    );
    assert_eq!(refusal.limit_or_gap().to_string(), "limit");
}

/// A quarter turn about any axis but `x`, `y` or `z` rounds, so the turn that
/// rounds nothing does not exist there.
#[test]
fn a_quarter_turn_about_a_diagonal_is_a_limit() {
    let diagonal = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0)).expect("a direction");
    let refusal = Isometry::quarter_turns(&diagonal, 1).expect_err("a diagonal is not x, y or z");
    assert!(
        matches!(refusal, Error::QuarterTurnsNeedAnAxisAlongXYOrZ { .. }),
        "{refusal:?}"
    );
    assert_eq!(refusal.limit_or_gap().to_string(), "limit");
}

/// Every way a spline can be asked for that names no spline is a limit, and
/// so is asking a good spline for a point, a knot or a split it does not have.
#[test]
fn a_spline_that_is_not_one_is_a_limit() {
    let line = BSplineCurve::new(
        KnotVector::new(1, vec![0.0, 0.0, 0.5, 1.0, 1.0]).expect("a clamped degree one vector"),
        &[
            Point::origin(),
            Point::new(1.0, 0.0, 0.0),
            Point::new(2.0, 0.0, 0.0),
        ],
        &[1.0, 1.0, 1.0],
    )
    .expect("a straight spline of two pieces");
    let two_points = [Point::origin(), Point::new(1.0, 0.0, 0.0)];
    let one_piece = || KnotVector::new(1, vec![0.0, 0.0, 1.0, 1.0]).expect("one span");

    let refusals = [
        KnotVector::new(MAX_DEGREE + 1, vec![0.0; 2 * MAX_DEGREE + 4])
            .expect_err("a degree past the largest carried"),
        KnotVector::new(3, vec![0.0, 0.0, 1.0, 1.0]).expect_err("four knots for degree three"),
        KnotVector::new(1, vec![0.0, 0.0, 1.0, 0.5, 2.0, 2.0]).expect_err("a knot that drops"),
        KnotVector::new(1, vec![0.0, 0.0, 0.5, 0.5, 1.0, 1.0])
            .expect_err("an interior knot twice in degree one"),
        KnotVector::new(1, vec![1.0, 1.0, 1.0, 1.0]).expect_err("every knot the same"),
        BSplineCurve::new(one_piece(), &two_points[..1], &[1.0])
            .expect_err("one control point where two are asked for"),
        BSplineCurve::new(one_piece(), &two_points, &[1.0, 0.0]).expect_err("a weight of zero"),
        line.evaluate(2.0).expect_err("past the end of the domain"),
        line.with_knot_inserted(0.5)
            .expect_err("a knot already as often as degree one allows"),
        line.split_at(0.0)
            .expect_err("nothing below the bottom end"),
    ];
    let names = refusals.each_ref().map(name_of);
    assert_eq!(
        names,
        [
            "DegreeNotSupported",
            "TooFewKnots",
            "KnotsGoBackwards",
            "KnotRepeatedTooOften",
            "KnotVectorHasNoSpan",
            "WrongNumberOfControlPoints",
            "WeightNotPositive",
            "ParameterOutsideDomain",
            "CannotRepeatKnotAgain",
            "NothingOnOneSideOfTheSplit",
        ]
    );
    for refusal in &refusals {
        assert_eq!(refusal.limit_or_gap().to_string(), "limit", "{refusal:?}");
    }
}

/// A run that names no place, a closed run whose ends differ, and a piece
/// with nothing to say which piece: each names a curve that is not there.
#[test]
fn a_run_that_names_no_curve_is_a_limit() {
    let floor = a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let wall = a_plane([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
    let nowhere = NamedRun::new(
        EdgeRun::Straight,
        [[f64::NAN, 0.0, 0.0], [1.0, 0.0, 0.0]].map(rcad_predicates::Point::from),
    )
    .expect("a straight run's ends are not checked against each other");
    let refusals = [
        walk(
            &nowhere,
            [&floor, &wall],
            Deflection::of(0.1).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("an end that is not a number"),
        NamedRun::new(
            EdgeRun::RightRoundWhereItsFacesCross,
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]].map(rcad_predicates::Point::from),
        )
        .expect_err("a closed run that ends somewhere else"),
        NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
            and: floor,
            picked_by: Default::default(),
        })
        .expect_err("a piece with no ends and no half-space"),
    ];
    let names = refusals.each_ref().map(name_of);
    assert_eq!(
        names,
        [
            "AnEndOfARunNamesNoPlace",
            "AClosedRunDoesNotComeBackToWhereItStarted",
            "ACrossingPieceHasNoEndsAndNothingPickingIt",
        ]
    );
    for refusal in &refusals {
        assert_eq!(refusal.limit_or_gap().to_string(), "limit", "{refusal:?}");
    }
}

/// A loop narrower than the grid of numbers where it sits has no `f64` point on
/// both its pipes. `how_small_a_crossing_can_be_before_no_point_sits_on_it.rs`
/// walks the same pair with a fatter thin pipe, which is the demonstration.
#[test]
fn a_crossing_smaller_than_the_numbers_where_it_sits_is_a_limit() {
    let fat = a_pipe(
        [7.234461236871035e-292, 1.2701238075670642e-101, 1.0],
        [1191182336.0, 0.0, 0.0],
        2147482112.0,
    );
    let thin = a_pipe([0.0, 1.0, 0.0], [0.0, 0.0, 0.0], 4.3601508761683463e-106);
    let Ok(rcad_geom::SurfaceMeet::Curves(runs)) =
        intersect(&fat, &thin, &Policy::certified(), &mut ProvenanceLog::new())
    else {
        panic!("these two pipes cross in two loops");
    };
    assert_eq!(runs.len(), 2);
    for run in &runs {
        let refusal = walk(
            run,
            [&fat, &thin],
            Deflection::of(f64::MAX / 4.0).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("no point sits on both pipes");
        assert_eq!(
            refusal,
            Error::ACrossingTooSmallForAnyPointToSitOnBothItsSurfaces
        );
        assert_eq!(refusal.limit_or_gap().to_string(), "limit");
    }
}

/// A plane along a cone's side cuts a parabola, which does not close, so the
/// vertex a closed run would need does not exist.
#[test]
fn a_crossing_that_runs_on_for_ever_asked_to_close_is_a_limit() {
    let cone = Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a slope"));
    let along_a_ruling = a_plane([0.0, 1.0, 1.0], [0.0, 0.0, 4.0]);
    for pair in [[&along_a_ruling, &cone], [&cone, &along_a_ruling]] {
        let refusal =
            where_a_closed_crossing_starts(&right_round(), pair, &mut ProvenanceLog::new())
                .expect_err("a parabola has no place a closed edge starts");
        assert_eq!(refusal, Error::TheCrossingOfTheseTwoSurfacesIsNotClosed);
        assert_eq!(refusal.limit_or_gap().to_string(), "limit");
    }
}

/// One ball written twice crosses nothing, at any precision.
#[test]
fn one_surface_written_twice_is_a_limit() {
    let ball = a_ball([0.0, 0.0, 0.0], 5.0);
    let refusal =
        where_a_closed_crossing_starts(&right_round(), [&ball, &ball], &mut ProvenanceLog::new())
            .expect_err("one ball names no crossing with itself");
    assert_eq!(
        refusal,
        Error::TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing
    );
    assert_eq!(refusal.limit_or_gap().to_string(), "limit");
}

/// A ball one out with a radius of three reaches exactly across a pipe of
/// radius two, so the two loops it cuts pinch together at `(-2, 0, 0)`, a
/// point both surfaces hold exactly.
#[test]
fn a_crossing_that_pinches_is_a_gap() {
    let ball = a_ball([1.0, 0.0, 0.0], 3.0);
    let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
    let pinch = rcad_predicates::Point::from([-2.0, 0.0, 0.0]);
    let (Surface::Sphere(round), Surface::Cylinder(straight)) = (&ball, &pipe) else {
        unreachable!("built as a ball and a pipe just above");
    };
    assert_eq!(round.side_of(pinch), Decision::Zero);
    assert_eq!(straight.side_of(pinch), Decision::Zero);
    for [one, other] in [[&ball, &pipe], [&pipe, &ball]] {
        let refusal = intersect(one, other, &Policy::certified(), &mut ProvenanceLog::new())
            .expect_err("a crossing that runs through itself is not named");
        assert_eq!(
            refusal,
            Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo
        );
        assert_eq!(refusal.limit_or_gap().to_string(), "gap");
    }
}

/// A plane beside a ring's axis, through the middle of its tube, cuts the tube
/// in a Cassini oval: a real curve with no name in this crate's vocabulary.
#[test]
fn a_plane_cutting_a_ring_in_a_quartic_is_a_gap() {
    let ring = Surface::Torus(Torus::new(Axis::z(), 4.0, 1.0).expect("a ring"));
    let beside = a_plane([0.0, 1.0, 0.0], [0.0, 4.5, 0.0]);
    for [one, other] in [[&beside, &ring], [&ring, &beside]] {
        let refusal = intersect(one, other, &Policy::certified(), &mut ProvenanceLog::new())
            .expect_err("a quartic is not named");
        assert_eq!(
            refusal,
            Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic
        );
        assert_eq!(refusal.limit_or_gap().to_string(), "gap");
    }
}

/// A ball sitting on a ring's tube, off its axis, crosses it in a space curve
/// that is plainly there: the ball's centre is on the ring's centre circle.
#[test]
fn a_ring_against_a_ball_off_its_axis_is_a_gap() {
    let ring = Surface::Torus(Torus::new(Axis::z(), 4.0, 1.0).expect("a ring"));
    let ball = a_ball([4.0, 0.0, 0.0], 1.5);
    for [one, other] in [[&ring, &ball], [&ball, &ring]] {
        let refusal = intersect(one, other, &Policy::certified(), &mut ProvenanceLog::new())
            .expect_err("a ring against an off-axis ball is not named");
        assert_eq!(
            refusal,
            Error::ACrossingBetweenATorusAndAnotherCurvedSurface
        );
        assert_eq!(refusal.limit_or_gap().to_string(), "gap");
    }
}

/// The ellipse a nearly flat plane cuts from a pipe of radius `1e300` runs off
/// the top of `f64`, and the same pair at `1e200` starts somewhere:
/// `where_a_closed_crossing_starts_is_a_place_or_a_refusal.rs`.
#[test]
fn a_crossing_whose_size_overflows_is_a_gap() {
    let shallow = a_plane([1.0, 0.0, 1e-20], [0.0, 0.0, 0.0]);
    let pipe_of =
        |radius: f64| Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a radius"));
    let (huge, large) = (pipe_of(1e300), pipe_of(1e200));
    for [one, other] in [[&huge, &shallow], [&shallow, &huge]] {
        let refusal =
            where_a_closed_crossing_starts(&right_round(), [one, other], &mut ProvenanceLog::new())
                .expect_err("an ellipse too big to hold");
        assert_eq!(refusal, Error::CannotWorkOutHowBigACrossingIs);
        assert_eq!(refusal.limit_or_gap().to_string(), "gap");
    }
    for [one, other] in [[&large, &shallow], [&shallow, &large]] {
        let start =
            where_a_closed_crossing_starts(&right_round(), [one, other], &mut ProvenanceLog::new())
                .expect("the same pair written smaller starts somewhere");
        assert!(start.is_finite(), "{start:?}");
    }
}

/// The plane `x + 16z = 8`, a pipe of radius one along `x` and a ball of
/// radius three about `(0, ½, 0)` meet in four points, and no order of the
/// three reaches them.
///
/// On the pipe the ball is `x² = y + 7.75`, so a point is on all three where
/// `y² + ((8 − x)/16)² = 1` with that `y`, and that sum is below one at
/// `x = ±2.8` and above it at `x = ±2.598` and `±2.96`: two roots on each
/// side. The plane leans well off square to the pipe's axis, and the ball's
/// centre is half a unit off the wall through that axis and the plane's
/// normal, so no wall mirrors the three (Q2196). The floor across a pipe of
/// radius twelve along `(0, 3, 4)` and a ball of thirteen about `(0, 3, 4)`,
/// which held `(±12, 0, 0)` exactly, was this test's shape until such a wall
/// answered it.
#[test]
fn a_plane_leaning_across_a_pipe_and_a_ball_off_its_axis_is_a_gap() {
    let leaning = a_plane([1.0, 0.0, 16.0], [8.0, 0.0, 0.0]);
    let pipe = a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
    let round = a_ball([0.0, 0.5, 0.0], 3.0);
    for order in [
        [&leaning, &pipe, &round],
        [&leaning, &round, &pipe],
        [&pipe, &leaning, &round],
        [&pipe, &round, &leaning],
        [&round, &leaning, &pipe],
        [&round, &pipe, &leaning],
    ] {
        let refusal =
            meet_of_three_surfaces(order, &Policy::certified(), &mut ProvenanceLog::new())
                .expect_err("one flat surface and two curved ones");
        assert_eq!(refusal, Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat);
        assert_eq!(refusal.limit_or_gap().to_string(), "gap");
    }
}

fn name_of(refusal: &Error) -> String {
    let debug = format!("{refusal:?}");
    let end = debug
        .find(|c: char| !c.is_ascii_alphanumeric())
        .unwrap_or(debug.len());
    debug[..end].to_owned()
}

fn right_round() -> NamedRun {
    NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed curve is never a piece with nothing picking it")
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

fn a_ball(centre: [f64; 3], radius: f64) -> Surface {
    Surface::Sphere(Sphere::new(Point::from(centre), radius).expect("a radius"))
}
