//! §24.18's criterion 3, and the bound `walk` publishes beside it.
//!
//! §24.7 calls the acceptance test one sentence: *every point the evaluator
//! produces must be certified ON both surfaces by `SideOfSurface`*. Writing it
//! turned up a sharpening that section could not have had, and §28 records it.
//!
//! # "Certified ON" has to mean "never certified OFF"
//!
//! A point on the circle where a plane cuts a ball is at coordinates no `f64`
//! can name, so a `Decision::Zero` about it is not something any amount of
//! precision reaches. Handed over as an exact triple it gets a certified `Pos`
//! or `Neg` — an exact answer to the wrong question (`docs/decisions.md` §2.5).
//! Handed over as **the box it is really known to be inside**, which is the
//! point plus and minus the bound `walk` publishes, the honest answer is that
//! nothing can place it, and a certified sign either way is the failure.
//!
//! That is not a weakening. It is the same oracle
//! `a_point_exactly_on_a_circle_is_never_certified_off_it.rs` already argues at
//! length one milestone down, and it has the same two halves:
//!
//! - **Nothing on the curve is ever certified off either surface.** A `Pos` or
//!   a `Neg` is a bug.
//! - **Everything comfortably off a surface is decided.** The same points
//!   against a ball half the size, or a pipe twice the radius, must come back
//!   with a certified sign — otherwise the first half would pass just as well
//!   for an evaluator whose bound was a mile wide.
//!
//! # The published bound, measured
//!
//! §15.6: a published bound needs a test. What `walk` hands back beside its
//! points is how far any of them may sit off the curve, and this file measures
//! the real distance from every point to both surfaces and prints the worst
//! share of that bound a search could find.

#![forbid(unsafe_code)]

mod crossings;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngAlgorithm, TestRng, TestRunner};
use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point,
    PointsAlongACrossing, Sphere, Surface, SurfaceMeet, Vector, intersect, walk,
};
use rcad_predicates::{
    Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface,
};
use smallvec::smallvec;

use crossings::{
    a_crossing_a_hair_from_the_knife_edge, a_lattice_crossing, a_leaning_crossing,
    a_quartic_lattice_crossing, a_quartic_turned_crossing, a_turned_crossing,
};

/// How many crossings each measured row walks.
const CASES: u32 = 512;

/// How many the band walks.
///
/// A quarter as many, because a crossing a few last bits from tangency is a long
/// thin loop that the marcher has to take small steps round: this row alone
/// walks more points than every other row in the file put together.
///
/// **It is also, on its own, most of what the whole suite costs**, and M4's
/// first lane is why. Measured at 128 draws: 43 runs and 45 442 points, of which
/// two closed conics account for 43 415 — those are cheap, because the closed
/// form parametrizes them. The expensive nine hundred are the *marched* ones, on
/// pieces of a nearly-tangent quartic that this crate used to refuse to name at
/// all for want of a half-space telling them apart (§33). Naming them is the
/// point of the lane, and walking them is what checks the naming, so the draws
/// stay where they are and the cost is recorded rather than trimmed.
const CASES_ON_A_KNIFE_EDGE: u32 = 128;

/// How many tests the band's draws are shared between.
///
/// Eight. One test walking all 128 took ninety seconds of the suite's
/// ninety-six; eight walking sixteen draws each run side by side on a machine
/// with cores to spare, and the suite comes back to about a quarter of that.
/// **No draw is cut and none is moved** — see [`Shard`].
const HOW_MANY_TESTS_THE_BAND_IS_WALKED_BY: u32 = 8;

/// How fine the walks in this file are, as a share of how big the shapes are.
///
/// A thousandth, matching `Deflection::RELATIVE_DEFAULT`. It is a share and not
/// a length because the somewhere-else family is not in this file but the
/// leaning one is, and an ellipse over a nearly-parallel plane reaches a long
/// way further than the shapes that made it.
const HOW_FINE: f64 = 1e-3;

/// The most of its own published bound a walked point was ever found to use.
///
/// A share, printed by every row and asserted here. **One is the failure**: at
/// one the bound is no longer a bound. Well under one is the ordinary gap
/// between a worst case and a drawn one, because roundings mostly cancel — the
/// same shape `Surface::ROUND_TRIP_ROUNDOFF` records, where a search across
/// forty orders of magnitude finds `2.4·ε` against a published `256·ε`.
///
/// Measured on the shipped corpus: 0.6% of the bound on the control, 0.8%
/// leaning and 1.2% turned, over about thirty thousand points. So the worst a
/// search finds is about `3·ε` of the shape's size against a published
/// `256·ε` — the same order as its neighbour's.
const NO_POINT_MAY_USE_ALL_OF_ITS_BOUND: f64 = 1.0;

/// The control: whole numbers, planes square to an axis.
#[test]
fn every_point_along_a_lattice_crossing_is_on_both_the_surfaces_that_named_it() {
    let worst = walk_every_crossing(a_lattice_crossing(), "the control");
    assert!(worst < NO_POINT_MAY_USE_ALL_OF_ITS_BOUND);
}

/// Ellipses. A plane leaning over a pipe is the shape M2 could not declare at
/// all, and the one §24.2 built this milestone around.
#[test]
fn every_point_along_a_leaning_crossing_is_on_both_the_surfaces_that_named_it() {
    let worst = walk_every_crossing(a_leaning_crossing(), "leaning");
    assert!(worst < NO_POINT_MAY_USE_ALL_OF_ITS_BOUND);
}

#[test]
fn every_point_along_a_turned_crossing_is_on_both_the_surfaces_that_named_it() {
    let worst = walk_every_crossing(a_turned_crossing(), "turned");
    assert!(worst < NO_POINT_MAY_USE_ALL_OF_ITS_BOUND);
}

/// The two rows M3's second stage added, and the ones the **marcher** walks.
///
/// Criterion 3 asks the same of a marched point as of a parametrized one, and
/// that is the whole reason it is worth asking twice: the closed form's bound is
/// derived from a chain of roundings and the marcher's is measured from the two
/// certified distances the point ends up at, so the two arms of the evaluator
/// reach the same claim by different roads.
#[test]
fn every_point_along_a_quartic_lattice_crossing_is_on_both_the_surfaces_that_named_it() {
    let worst = walk_every_crossing(a_quartic_lattice_crossing(), "the quartic control");
    assert!(worst < NO_POINT_MAY_USE_ALL_OF_ITS_BOUND);
}

#[test]
fn every_point_along_a_quartic_turned_crossing_is_on_both_the_surfaces_that_named_it() {
    let worst = walk_every_crossing(a_quartic_turned_crossing(), "the quartic headline");
    assert!(worst < NO_POINT_MAY_USE_ALL_OF_ITS_BOUND);
}

/// **The band**, and the row §24.1 says the marcher was built for.
///
/// Every pair here is drawn exactly tangent and moved a few last bits off it, so
/// the crossings are the nearly-pinched loops and hair-thin arcs where a
/// marcher's answer stops being obvious. What has to hold is what holds
/// everywhere else: no point may be certified *off* either surface, and the same
/// points against a surface they are plainly not on must be decided.
///
/// # Why this is eight tests and not one
///
/// It used to be one, and it took ninety of the suite's ninety-six seconds
/// (§33.5). The thousand marched points that cost the eighty-eight are the most
/// valuable in the file — they are exactly the pieces M4's first lane learned to
/// name — so cutting draws to buy CI time back would be §14.7's failure on
/// purpose, and §33.5 said the honest fix was a tooling decision.
///
/// **It is not.** The whole family is still drawn, in the same order, from the
/// same seed, and every one of the 128 draws is still walked; what changed is
/// that the eight tests each walk one draw in eight of it, and the test runner
/// puts them on eight cores. The counts the eight print add up to what the one
/// printed, because [`Shard`] decides which draws a test *works on* and never
/// which draws the runner *makes* — the strategy is sampled 128 times in every
/// one of them.
#[test]
fn every_point_along_the_first_eighth_of_the_band_is_on_both_the_surfaces_that_named_it() {
    walk_one_eighth_of_the_band(0);
}

#[test]
fn every_point_along_the_second_eighth_of_the_band_is_on_both_the_surfaces_that_named_it() {
    walk_one_eighth_of_the_band(1);
}

#[test]
fn every_point_along_the_third_eighth_of_the_band_is_on_both_the_surfaces_that_named_it() {
    walk_one_eighth_of_the_band(2);
}

#[test]
fn every_point_along_the_fourth_eighth_of_the_band_is_on_both_the_surfaces_that_named_it() {
    walk_one_eighth_of_the_band(3);
}

#[test]
fn every_point_along_the_fifth_eighth_of_the_band_is_on_both_the_surfaces_that_named_it() {
    walk_one_eighth_of_the_band(4);
}

#[test]
fn every_point_along_the_sixth_eighth_of_the_band_is_on_both_the_surfaces_that_named_it() {
    walk_one_eighth_of_the_band(5);
}

#[test]
fn every_point_along_the_seventh_eighth_of_the_band_is_on_both_the_surfaces_that_named_it() {
    walk_one_eighth_of_the_band(6);
}

#[test]
fn every_point_along_the_last_eighth_of_the_band_is_on_both_the_surfaces_that_named_it() {
    walk_one_eighth_of_the_band(7);
}

/// One eighth of the band's draws, walked and checked like every other row.
fn walk_one_eighth_of_the_band(which: u32) {
    let worst = walk_them_all(
        a_crossing_a_hair_from_the_knife_edge(),
        &format!("the band, eighth {}", which + 1),
        CASES_ON_A_KNIFE_EDGE,
        Shard {
            which,
            of: HOW_MANY_TESTS_THE_BAND_IS_WALKED_BY,
        },
    );
    assert!(worst < NO_POINT_MAY_USE_ALL_OF_ITS_BOUND);
}

/// The cap circle of a round bar, worked out rather than declared.
///
/// This is §24.2's shape: M2 built this bar by *declaring* its cap edge, and
/// here the same circle comes out of the cylinder and the cap's plane with
/// nothing declared at all.
#[test]
fn the_cap_circle_of_a_round_bar_is_walked_into_points_on_the_bar() {
    let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a pipe of radius three"));
    let cap = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal"),
    );

    let walked = walk_it(
        &NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross),
        [&pipe, &cap],
        3.0,
    );
    assert!(
        walked.points().len() >= 8,
        "a whole circle at a thousandth of its radius takes more than eight pieces, got {}",
        walked.points().len()
    );
    for point in walked.points() {
        assert!(
            (point.z() - 5.0).abs() <= walked.deviation().magnitude(),
            "a point on the cap circle sits in the cap's own plane: {point:?}"
        );
    }
    every_point_is_on_both(
        &[pipe, cap],
        walked.points(),
        walked.deviation().magnitude(),
    );
}

/// The shape M2 could not build: a bar cut off by a tilted plane, whose edge is
/// an ellipse and not a circle.
#[test]
fn a_bar_cut_off_at_an_angle_is_walked_into_points_on_an_ellipse() {
    let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a pipe of radius three"));
    let cut = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 1.0, 1.0))
            .expect("a normal with a length"),
    );

    let walked = walk_it(
        &NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross),
        [&pipe, &cut],
        3.0,
    );
    every_point_is_on_both(
        &[pipe, cut],
        walked.points(),
        walked.deviation().magnitude(),
    );

    // An ellipse and not a circle. Measured from the middle, which is where the
    // axis meets the cut: the narrow way across is the pipe's own radius, and the
    // long way is that stretched by how far the plane leans, `√2` here.
    let middle = Point::new(0.0, 0.0, 5.0);
    let from_the_middle = walked
        .points()
        .iter()
        .map(|point| (*point - middle).length());
    let narrowest = from_the_middle.clone().fold(f64::MAX, f64::min);
    let widest = from_the_middle.fold(f64::MIN, f64::max);
    assert!(
        (narrowest - 3.0).abs() < 0.01,
        "across the pipe the ellipse is the pipe's own radius, got {narrowest}"
    );
    assert!(
        (widest - 3.0 * f64::sqrt(2.0)).abs() < 0.01,
        "down the slope it is stretched by the lean, got {widest}"
    );
}

/// A ball's seam: both faces are the same sphere, the second surface comes off
/// the run, and the half-space is the whole of what says which half of the
/// meridian circle it is.
///
/// The hardest edge the kernel builds, and the one §27.5 settled the seam
/// around.
#[test]
fn a_balls_seam_is_walked_down_the_half_of_the_meridian_its_half_space_picks() {
    let radius = 3.0;
    let ball = Surface::Sphere(Sphere::new(Point::origin(), radius).expect("a radius"));
    let meridian = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("the y axis is a unit normal");
    let facing_x = HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis has length");

    let seam = NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(meridian),
            picked_by: smallvec![Picker::from(facing_x)],
        },
        [
            PredicatePoint::from([0.0, 0.0, radius]),
            PredicatePoint::from([0.0, 0.0, -radius]),
        ],
    );

    let walked = walk_it(&seam, [&ball, &ball], radius);
    every_point_is_on_both(
        &[ball, Surface::Plane(meridian)],
        walked.points(),
        walked.deviation().magnitude(),
    );
    for point in walked.points() {
        assert!(
            point.x() >= -walked.deviation().magnitude(),
            "the half-space picks the half of the meridian with x above zero, not {point:?}"
        );
    }
    assert!(
        walked
            .points()
            .first()
            .expect("a seam takes more than one step")
            .z()
            > walked.points().last().expect("more than one").z(),
        "the points come back in the direction the run is stored running, north to south"
    );
}

/// A straight run needs nothing between its ends, and says so rather than
/// putting a point somewhere it would have to guess.
#[test]
fn a_straight_run_is_walked_into_no_points_at_all_and_no_slop() {
    let ground = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("a unit normal"),
    );
    let wall = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
            .expect("a unit normal"),
    );
    let walked = walk_it(
        &NamedRun::the_whole_of(EdgeRun::Straight),
        [&ground, &wall],
        1.0,
    );
    assert!(walked.points().is_empty());
    assert_eq!(walked.deviation().magnitude(), 0.0);
}

/// The two knobs do different jobs (§23.5). A deflection finer than a point can
/// be placed is refused under a certified policy and taken under a tolerant one,
/// and neither of those is the caller asking for fewer triangles.
#[test]
fn a_deflection_finer_than_a_point_can_be_placed_is_a_policy_question() {
    let ball = Surface::Sphere(Sphere::new(Point::origin(), 3.0).expect("a radius"));
    let through = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("a unit normal"),
    );
    let circle = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross);
    let hopeless = Deflection::of(1e-18).expect("a length");

    match walk(
        &circle,
        [&ball, &through],
        hopeless,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Err(Error::CannotPlaceAPointThatCloseToACrossing { asked, closest }) => {
            assert_eq!(asked, 1e-18);
            assert!(closest > asked);
        }
        other => panic!("no point can be placed within 1e-18 of this circle: {other:?}"),
    }

    let taken = walk(
        &circle,
        [&ball, &through],
        hopeless,
        // A working size of a hundred against a shape of three, which is the
        // ordinary relation: `Policy::tolerant` allows `64·ε` of the *model's*
        // size and a walked point sits `256·ε` of the *shape's* deep, so a
        // caller whose model is a good deal bigger than one crossing of it can
        // take this and one whose model is the crossing cannot.
        &Policy::tolerant(100.0),
        &mut ProvenanceLog::new(),
    )
    .expect("a tolerant caller has already said how far out of place it will let a point be");
    assert!(taken.deviation().magnitude() > 0.0);
    assert!(
        taken.points().is_empty(),
        "there is no room left to refine in"
    );
}

/// Walk every crossing a family draws, and hand back the worst share of its own
/// published bound any point was found to use.
fn walk_every_crossing(strategy: impl Strategy<Value = crossings::Crossing>, what: &str) -> f64 {
    walk_them_all(strategy, what, CASES, Shard::THE_WHOLE_FAMILY)
}

/// Which draws of a family one test does the work for.
///
/// **This picks which draws are *walked* and never which are *made*.** The
/// runner samples the strategy `cases` times whatever this says, so every test
/// sharing a family sees the identical sequence from the identical seed, and the
/// shards partition it exactly: draw `n` is walked by the one test whose `which`
/// is `n % of`. Splitting a family this way therefore loses nothing and moves
/// nothing — the union of the shards is the family, draw for draw.
#[derive(Clone, Copy)]
struct Shard {
    which: u32,
    of: u32,
}

impl Shard {
    /// Every draw, in one test. What every row but the band uses.
    const THE_WHOLE_FAMILY: Self = Self { which: 0, of: 1 };

    /// Whether this test does the work for the draw at `drawn`.
    fn holds(self, drawn: u32) -> bool {
        drawn % self.of == self.which
    }
}

/// The same, at a case count this row chooses and over one slice of its draws.
fn walk_them_all(
    strategy: impl Strategy<Value = crossings::Crossing>,
    what: &str,
    cases: u32,
    shard: Shard,
) -> f64 {
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases,
            source_file: Some(file!()),
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
                "proptest-regressions",
            ))),
            ..Config::default()
        },
        TestRng::deterministic_rng(RngAlgorithm::ChaCha),
    );

    let worst = std::cell::Cell::new(0.0f64);
    let walked = std::cell::Cell::new(0usize);
    let points = std::cell::Cell::new(0usize);
    let drawn = std::cell::Cell::new(0u32);
    let outcome = runner.run(&strategy, |crossing| {
        let index = drawn.get();
        drawn.set(index + 1);
        if !shard.holds(index) {
            return Ok(());
        }
        let surfaces = crossing.surfaces();
        let Ok(SurfaceMeet::Curves(runs)) = intersect(
            &surfaces[0],
            &surfaces[1],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) else {
            return Ok(());
        };
        let deflection = Deflection::of(crossing.how_big() * HOW_FINE).expect("a positive length");

        for run in &runs {
            let along = [&surfaces[0], &surfaces[1]];
            let Ok(along_it) = walk(
                run,
                along,
                deflection,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            ) else {
                continue;
            };
            walked.set(walked.get() + 1);
            points.set(points.get() + along_it.points().len());
            let bound = along_it.deviation().magnitude();
            worst.set(
                worst
                    .get()
                    .max(how_much_of_its_bound(&surfaces, along_it.points(), bound)),
            );
            every_point_is_on_both(&surfaces, along_it.points(), bound);
        }
        Ok(())
    });
    if let Err(why) = outcome {
        panic!("{why}");
    }

    println!(
        "{what}: {} runs walked into {} points, the furthest any of them sat off its own curve \
         was {:.3} of the bound published for it",
        walked.get(),
        points.get(),
        worst.get()
    );
    // A hundred points for the family, shared out. A row split into shards is
    // still one family and the floor is still the family's; what each of its
    // tests owes is its share of it.
    assert!(
        points.get() * shard.of as usize > 100,
        "{what} walked {} points, which is not enough to be measuring anything",
        points.get()
    );
    worst.get()
}

/// Both halves of the acceptance oracle, for every point of one run.
fn every_point_is_on_both(surfaces: &[Surface; 2], points: &[Point], bound: f64) {
    for &point in points {
        let box_it_is_in = somewhere_within(point, bound);
        for surface in surfaces {
            let decision = side_of(surface, box_it_is_in);
            assert!(
                !matches!(decision, Decision::Pos | Decision::Neg),
                "this point is on {surface:?} and came back {decision:?}: {point:?}"
            );
            let decision = side_of(&clearly_somewhere_else(surface), box_it_is_in);
            assert!(
                matches!(decision, Decision::Pos | Decision::Neg | Decision::Zero),
                "this point is a long way off {surface:?} shrunk to half, and nothing would \
                 say so: {decision:?}"
            );
        }
    }
}

/// The worst share of `bound` any of these points was actually found to use.
fn how_much_of_its_bound(surfaces: &[Surface; 2], points: &[Point], bound: f64) -> f64 {
    points
        .iter()
        .flat_map(|&point| {
            surfaces
                .iter()
                .map(move |surface| how_far_off(surface, point))
        })
        .fold(0.0, f64::max)
        / bound
}

/// How far `point` really is from `surface`, in `f64`.
///
/// Ordinary arithmetic, and that is sound here for one reason: every one of
/// these is a difference between two numbers of about the same size as the shape
/// itself, so what it carries is a rounding of the shape and not a cancellation
/// of the answer. The answers it is compared against are hundreds of roundings
/// wide.
fn how_far_off(surface: &Surface, point: Point) -> f64 {
    match surface {
        Surface::Plane(flat) => {
            (flat.normal().dot(point - Point::origin()) - flat.offset()).abs()
                / flat.normal().length()
        }
        Surface::Sphere(ball) => ((point - ball.centre()).length() - ball.radius()).abs(),
        Surface::Cylinder(pipe) => {
            let along = pipe.axis().direction();
            ((point - pipe.axis().origin()).cross(along).length() - pipe.radius()).abs()
                / along.length()
        }
        other => panic!("nothing here can measure against {other:?}"),
    }
}

/// The same surface moved far enough that a point on the original is plainly
/// not on this one.
///
/// The acceptance half of the oracle. Half the radius for a ball or a pipe;
/// for a plane, the same plane moved by its own reach, which is never nothing
/// because a plane through the origin still gets moved by one.
fn clearly_somewhere_else(surface: &Surface) -> Surface {
    match surface {
        Surface::Plane(flat) => {
            let moved = flat.offset() + flat.offset().abs().max(1.0);
            Surface::Plane(
                Plane::from_point_and_normal(
                    Point::origin() + flat.normal() * moved,
                    flat.normal(),
                )
                .expect("a stored normal has a length"),
            )
        }
        Surface::Sphere(ball) => Surface::Sphere(
            Sphere::new(ball.centre(), ball.radius() / 2.0).expect("half a radius is a radius"),
        ),
        Surface::Cylinder(pipe) => Surface::Cylinder(
            Cylinder::new(pipe.axis(), pipe.radius() / 2.0).expect("half a radius is a radius"),
        ),
        other => panic!("nothing here can move {other:?}"),
    }
}

fn side_of(surface: &Surface, point: PredicatePoint) -> Decision {
    match surface {
        Surface::Plane(flat) => flat.side_of_point(point),
        Surface::Sphere(ball) => ball.side_of(point),
        Surface::Cylinder(pipe) => pipe.side_of(point),
        other => panic!("nothing here can ask about {other:?}"),
    }
}

/// The box a computed point is really known to be inside.
fn somewhere_within(point: Point, bound: f64) -> PredicatePoint {
    PredicatePoint::Constructed(point.coordinates().map(|coordinate| {
        Interval::new(coordinate - bound, coordinate + bound).expect("a bound is a length")
    }))
}

/// One run, walked as finely as [`HOW_FINE`] asks for a shape this big.
fn walk_it(run: &NamedRun, along: [&Surface; 2], how_big: f64) -> PointsAlongACrossing {
    walk(
        run,
        along,
        Deflection::of(how_big * HOW_FINE).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a shape this size is walked without complaint")
}
