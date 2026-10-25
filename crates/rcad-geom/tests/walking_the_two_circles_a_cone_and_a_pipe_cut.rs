//! The two circles a cone and a pipe on one axis cross in, drawn.
//!
//! Naming the pair without drawing it would be half an answer: a fuse that is
//! handed two circles it cannot walk has a face it cannot mesh. So the evaluator
//! gets the same arm the namer got, and this file checks the two halves agree —
//! about which pairs are refused, and about where the points land.
//!
//! Every point below is checked against **both** surfaces rather than against a
//! closed form written out here. A second copy of the circle would be a second
//! authority on where it is (`docs/decisions.md` §24.3), and the cone and the
//! pipe already say where it is exactly.

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, Error, NamedRun, POINT_ALONG_A_CROSSING_ROUNDOFF,
    Picker, Point, Sphere, Surface, SurfaceMeet, Vector, intersect, walk,
    where_a_closed_crossing_starts,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// A cone of slope one half about `+z` with its apex at the origin, against a
/// bar of radius two on the same axis: the circles sit at `z = ±4`.
fn a_frustum_and_a_bar() -> (Cone, Cylinder) {
    (
        Cone::new(Axis::z(), 0.5).expect("a slope above zero"),
        Cylinder::new(Axis::z(), 2.0).expect("a radius above zero"),
    )
}

/// The runs `intersect` names for a pair, which is what the walker is handed.
fn the_runs(cone: &Cone, pipe: &Cylinder) -> Vec<NamedRun> {
    let named = intersect(
        &Surface::Cone(*cone),
        &Surface::Cylinder(*pipe),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a coaxial cone and pipe are named");
    let SurfaceMeet::Curves(runs) = named else {
        panic!("two circles, and not {named:?}");
    };
    runs
}

/// How far `point` sits off the cone, as a plain distance.
///
/// The distance across the axis is a **cross product** and not
/// `√(‖p‖² − (p·ŵ)²)`. The second form subtracts two numbers of size sixteen to
/// leave one of size four, so everything under about `2e-15` quantizes to
/// exactly zero and the measurement reads perfect however wrong the point is.
/// `the_measurement_can_see_a_point_that_is_off` is the row that keeps this
/// honest.
fn how_far_off_the_cone(cone: &Cone, point: Point) -> f64 {
    let along = cone.axis().direction();
    let out = point - cone.apex();
    (out.cross(along).length() - cone.slope() * out.dot(along).abs()).abs()
        / (along.length() * cone.slope().hypot(1.0))
}

/// How far `point` sits off the pipe, measured the same way.
fn how_far_off_the_pipe(pipe: &Cylinder, point: Point) -> f64 {
    let along = pipe.axis().direction();
    ((point - pipe.axis().origin()).cross(along).length() / along.length() - pipe.radius()).abs()
}

/// The measurement above reads zero on a point that is on both surfaces and
/// something honest on one that is not.
///
/// **A row about the instrument and not about the kernel.** The bound below is
/// checked by comparing a measured miss against it, and a measurement that
/// cannot see a miss passes that check whatever the walker did.
#[test]
fn the_measurement_can_see_a_point_that_is_off() {
    let (cone, pipe) = a_frustum_and_a_bar();
    let on_both = Point::new(2.0, 0.0, 4.0);
    let nudged = Point::new(2.0 + 1e-12, 0.0, 4.0);

    assert!(how_far_off_the_cone(&cone, on_both) < 1e-15);
    assert!(how_far_off_the_pipe(&pipe, on_both) < 1e-15);
    assert!(
        how_far_off_the_cone(&cone, nudged) > 1e-13,
        "a point a picometre out reads as one against the cone"
    );
    assert!(
        how_far_off_the_pipe(&pipe, nudged) > 1e-13,
        "and against the pipe"
    );
}

/// Every point of both circles is on both surfaces, inside the bound the walker
/// publishes for a conic.
#[test]
fn every_point_walked_along_either_circle_is_on_the_cone_and_on_the_pipe() {
    let (cone, pipe) = a_frustum_and_a_bar();
    let along = [Surface::Cone(cone), Surface::Cylinder(pipe)];
    // The circles reach two out and stand four up, so eight is the size the
    // walker's bound is a multiple of.
    let room = POINT_ALONG_A_CROSSING_ROUNDOFF * 8.0;

    let mut walked = 0;
    let mut worst: f64 = 0.0;
    for run in the_runs(&cone, &pipe) {
        let points = walk(
            &run,
            [&along[0], &along[1]],
            Deflection::of(0.01).expect("a length above zero"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a circle is walked without complaint");
        assert!(
            points.points().len() > 8,
            "a circle of radius two at a deflection of a hundredth takes more than eight pieces"
        );
        for point in points.points() {
            worst = worst.max(how_far_off_the_cone(&cone, *point));
            worst = worst.max(how_far_off_the_pipe(&pipe, *point));
            walked += 1;
        }
    }
    assert!(walked > 0, "there were points to check");
    eprintln!("worst miss {worst:e} against a published {room:e} over {walked} points");
    assert!(
        worst <= room,
        "worst miss {worst:e} is inside the published {room:e} over {walked} points"
    );
}

/// The two circles are walked at `z = +4` and `z = −4`, one each, and the walker
/// tells them apart by the picker `intersect` put on the run.
#[test]
fn the_two_runs_are_walked_round_two_different_circles() {
    let (cone, pipe) = a_frustum_and_a_bar();
    let along = [Surface::Cone(cone), Surface::Cylinder(pipe)];

    let heights: Vec<f64> = the_runs(&cone, &pipe)
        .iter()
        .map(|run| {
            let points = walk(
                run,
                [&along[0], &along[1]],
                Deflection::of(0.01).expect("a length above zero"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a circle is walked without complaint");
            let up = points.points()[0].coordinates()[2];
            for point in points.points() {
                assert!(
                    (point.coordinates()[2] - up).abs() < 1e-9,
                    "a circle square to the axis stays at one height"
                );
            }
            up
        })
        .collect();

    assert_eq!(heights.len(), 2);
    assert!(
        (heights[0] - 4.0).abs() < 1e-9,
        "the first run is the circle in the nappe the axis points along: {heights:?}"
    );
    assert!(
        (heights[1] + 4.0).abs() < 1e-9,
        "and the second is the other one: {heights:?}"
    );
}

/// A closed crossing has one place a vertex can go, and it is on both surfaces.
///
/// Both circles are closed, so a fuse cutting a face along either has to put a
/// vertex somewhere, and where that goes may not depend on which face asked
/// (§31.3).
#[test]
fn where_each_circle_starts_is_on_both_surfaces_and_does_not_move_when_the_pair_is_swapped() {
    let (cone, pipe) = a_frustum_and_a_bar();
    let one_way = [Surface::Cone(cone), Surface::Cylinder(pipe)];
    let other_way = [Surface::Cylinder(pipe), Surface::Cone(cone)];

    for run in the_runs(&cone, &pipe) {
        let start = where_a_closed_crossing_starts(
            &run,
            [&one_way[0], &one_way[1]],
            &mut ProvenanceLog::new(),
        )
        .expect("a circle is closed and has a place to start");
        let turned = where_a_closed_crossing_starts(
            &run,
            [&other_way[0], &other_way[1]],
            &mut ProvenanceLog::new(),
        )
        .expect("the same, asked the other way round");
        assert_eq!(start, turned, "the pair is not written in a fixed order");

        let room = POINT_ALONG_A_CROSSING_ROUNDOFF * 8.0;
        let off_the_cone = how_far_off_the_cone(&cone, start);
        let off_the_pipe = how_far_off_the_pipe(&pipe, start);
        assert!(
            off_the_cone <= room,
            "the start is {off_the_cone:e} off the cone"
        );
        assert!(
            off_the_pipe <= room,
            "the start is {off_the_pipe:e} off the pipe"
        );
    }
}

/// The walker works the question out rather than trusting the run it was
/// handed, and a whole run stated on a pair off one axis is refused under the
/// name of what is wrong with it, or walked round one loop.
///
/// A caller can state any run at all between two surfaces — `walk` is public and
/// a file read in states runs this crate never named. So a cone and a partner
/// off its axis handed over as one closed run with nothing picking it has to
/// come back with the answer that pair has, and never with a circle worked out
/// from numbers that do not describe one:
///
/// - The pipe leaning along one of the cone's own straight lines is the
///   refusal `intersect` gives that pair.
/// - The ball stands one to the side of the axis inside the upper nappe and
///   misses the cone, so the run names a crossing that is not there, refused
///   as a ball on the axis that misses the cone is. Until row Q2788 the walk
///   called it too small to sit on and its start was a point off the ball.
/// - The pipe standing one to the side of the axis goes through both nappes
///   and crosses the cone in a loop round each, which `intersect` names as two
///   pieces. A run naming neither is walked round one of them, as it is on a
///   thin pipe through a fat one, and starts at the same place whichever face
///   asks. The coaxial pair's run below is refused instead: its two circles
///   are not marched from seeds.
///
/// On the axis, a ball is drawn the way the pipe is, and
/// `a_cone_crossed_by_a_cone_or_ball_on_its_axis.rs` walks it.
#[test]
fn a_whole_run_stated_on_a_pair_off_one_axis_is_refused_by_name_or_walked_round_one_loop() {
    let cone = Cone::new(Axis::z(), 1.0).expect("a slope above zero");
    let leaning = Cylinder::new(
        Axis::new(Point::new(0.0, 2.0, 0.0), Vector::new(1.0, 0.0, 1.0)).expect("a direction"),
        2.0,
    )
    .expect("a radius above zero");
    let beside_it = Cylinder::new(
        Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
        2.0,
    )
    .expect("a radius above zero");
    let ball = Sphere::new(Point::new(1.0, 0.0, 4.0), 2.0).expect("a radius above zero");

    let stated = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed curve is never a piece with nothing picking it");
    let cone = Surface::Cone(cone);
    for (partner, refused) in [
        (
            Surface::Cylinder(leaning),
            Error::ACrossingBetweenAConeAndAnotherCurvedSurface,
        ),
        (
            Surface::Sphere(ball),
            Error::TheseTwoSurfacesDoNotCrossInAConic,
        ),
    ] {
        for pair in [[&cone, &partner], [&partner, &cone]] {
            assert_eq!(
                walk(
                    &stated,
                    pair,
                    Deflection::of(0.01).expect("a length above zero"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .map(|walked| walked.points().len()),
                Err(refused.clone()),
                "{pair:?}"
            );
            assert_eq!(
                where_a_closed_crossing_starts(&stated, pair, &mut ProvenanceLog::new()),
                Err(refused.clone()),
                "{pair:?}"
            );
        }
    }

    let beside_it = Surface::Cylinder(beside_it);
    let start = |pair: [&Surface; 2]| {
        where_a_closed_crossing_starts(&stated, pair, &mut ProvenanceLog::new())
            .expect("the first loop has a seed")
    };
    assert_eq!(start([&cone, &beside_it]), start([&beside_it, &cone]));
    for pair in [[&cone, &beside_it], [&beside_it, &cone]] {
        let walked = walk(
            &stated,
            pair,
            Deflection::of(0.01).expect("a length above zero"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the run is walked round the first loop");
        let way = walked.points()[0].z().signum();
        assert!(walked.points().len() > 8, "{pair:?}");
        for point in walked.points() {
            let [x, y, z] = point.coordinates();
            assert!(z * way > 0.0, "{pair:?}: {point:?} is on the first loop");
            assert!(
                (x.hypot(y) - z.abs()).abs() < 1e-12,
                "{point:?} is on the cone"
            );
            assert!(
                ((x - 1.0).hypot(y) - 2.0).abs() < 1e-12,
                "{point:?} is on the pipe"
            );
        }
    }
}

/// A run on a coaxial pair with nothing picking it holds both circles, and that
/// is refused rather than answered with whichever came first.
#[test]
fn a_run_that_names_neither_circle_is_refused_rather_than_guessed() {
    let (cone, pipe) = a_frustum_and_a_bar();
    let along = [Surface::Cone(cone), Surface::Cylinder(pipe)];
    let stated = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed curve is never a piece with nothing picking it");

    assert_eq!(
        walk(
            &stated,
            [&along[0], &along[1]],
            Deflection::of(0.01).expect("a length above zero"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .err(),
        Some(Error::CannotNameWhichPieceOfACrossingThisIs),
        "there are two circles and this run says nothing about which"
    );
}

/// The picker alone says which circle a run is: rebuilt from the second run's
/// half-space and nothing else, it still lands on the second circle.
#[test]
fn the_picker_is_the_whole_of_what_says_which_circle_a_run_is() {
    let (cone, pipe) = a_frustum_and_a_bar();
    let along = [Surface::Cone(cone), Surface::Cylinder(pipe)];
    let runs = the_runs(&cone, &pipe);
    let EdgeRun::PartOfWhereItCrosses { and, picked_by } = runs[1].run().clone() else {
        panic!("each circle is a piece of the crossing");
    };
    assert!(matches!(picked_by.as_slice(), [Picker::InFrontOf(_)]));

    let swapped = NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses { and, picked_by })
        .expect("picked_by holds a half-space");
    let started =
        where_a_closed_crossing_starts(&swapped, [&along[0], &along[1]], &mut ProvenanceLog::new())
            .expect("the second picker names the second circle");
    assert!(
        (started.coordinates()[2] + 4.0).abs() < 1e-9,
        "the second picker holds the circle at z = -4, not the one at +4: {started:?}"
    );
}
