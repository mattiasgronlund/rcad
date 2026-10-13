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
//! points is **how far any of them may sit off the curve**, and this file
//! prints the worst share of that bound a search could find.
//!
//! **Off the curve is not off the two surfaces**, and for a long time this file
//! measured the second and called it the first. A point that is a rounding away
//! from a bar and a rounding away from the cap that cuts it can still be a long
//! way from the ellipse the two of them meet in — because where two surfaces
//! meet at a shallow angle, sliding a point along the sliver they share barely
//! changes how far it is from either one. Measured here on a bar of radius one:
//! the per-surface distance is between half a rounding and two roundings at
//! every lean from a flat cap to one leaning by a thousand, and at that steepest
//! lean the distance to the curve is **two hundred and fifty-six roundings**.
//!
//! `every_point_along_a_leaning_cap_is_within_its_bound_of_the_curve_the_two_surfaces_cross_in`
//! is the row that measures the named quantity, and it carries the old one
//! along beside it on every line so a reader can see the family is not blind to
//! the difference (§52.7's shape). The six family rows above it still measure
//! per-surface distance, and what they are worth is said on the table in
//! [`NO_POINT_MAY_USE_ALL_OF_ITS_BOUND`].

#![forbid(unsafe_code)]

mod crossings;
mod soak;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, TestRunner};
use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point,
    PointsAlongACrossing, Sphere, Surface, SurfaceMeet, Torus, Vector, intersect, walk,
};
use rcad_predicates::{
    Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface,
};
use smallvec::smallvec;

use crossings::{
    a_crossing_a_hair_from_the_knife_edge, a_lattice_crossing, a_leaning_crossing,
    a_quartic_lattice_crossing, a_quartic_turned_crossing, a_turned_crossing,
};

/// How many crossings each measured row walks on the shipped corpus, which
/// `soak::cases` multiplies and `soak::seeded` reseeds.
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

/// How fine the walks in this file are, as a share of how big the shapes are.
///
/// A thousandth, matching `Deflection::RELATIVE_DEFAULT`. It is a share and not
/// a length because the somewhere-else family is not in this file but the
/// leaning one is, and an ellipse over a nearly-parallel plane reaches a long
/// way further than the shapes that made it.
const HOW_FINE: f64 = 1e-3;

/// How far the cap of a round bar leans, for the row that measures against the
/// curve itself.
///
/// A lean of zero is a flat cap and a circle; a lean of one is the forty-five
/// degree chamfer anybody would draw first; the rest climb by powers of two
/// until the cap is very nearly parallel to the bar it is cutting.
const HOW_FAR_A_CAP_LEANS: [f64; 9] = [0.0, 0.5, 1.0, 1.5, 2.0, 4.0, 16.0, 128.0, 1024.0];

/// How many roundings of a bar's own radius the **per-surface** distance is
/// allowed to be, at any lean.
///
/// Eight, and the whole point of the number is that nothing in the sweep ever
/// approaches it. The worst measured, over both radii and all nine leans, is
/// **exactly two roundings**, and it is reached at a lean of four and again at a
/// lean of a hundred and twenty-eight; at the steepest lean of all it is back to
/// one. That flatness is the blindness.
const A_PER_SURFACE_DISTANCE_STAYS_THIS_FLAT: f64 = 8.0;

/// How many roundings of a bar's radius the distance **to the curve** has to
/// reach by the steepest lean in the sweep.
///
/// A hundred, against a per-surface distance pinned at two. Measured: `256`
/// roundings at radius one and `192` at radius four, both at a lean of a
/// thousand. Two quantities one of which is a hundred times the other are not
/// the same measurement, and that is what this constant is here to say.
const THE_DISTANCE_TO_THE_CURVE_CLIMBS_PAST_THIS: f64 = 100.0;

/// The most of its own published bound a walked point was ever found to use.
///
/// A share, printed by every row and asserted here. **One is the failure**: at
/// one the bound is no longer a bound.
///
/// Every row measured, at the shipped corpus and at `RCAD_SOAK_FACTOR=2` and
/// `8`:
///
/// | row | shipped | ×2 | ×8 |
/// |---|---|---|---|
/// | the control | 0.006 | 0.006 | 0.008 |
/// | leaning | 0.008 | 0.008 | 0.011 |
/// | turned | 0.012 | 0.012 | 0.013 |
/// | the quartic control | 0.738 | 0.738 | 0.738 |
/// | the quartic headline | 0.552 | 0.726 | 0.726 |
/// | the band | 0.389 | 0.389 | 0.477 |
///
/// **The two marched rows were re-measured when the marcher stopped measuring
/// against a pipe `r(1/‖w‖ − 1)` narrower than the one it was handed**
/// (§118.11). The headline read `0.355 | 0.361 | —` before that, and the change
/// moves both halves of the share at once: the marcher settles its points
/// against the surface the caller named, and this file's own `how_far_off` reads
/// them against it too. The band's `0.410 | 0.410` is older than that and was
/// already reading `0.389` before this lane touched anything.
///
/// **Those six measure per-surface distance, which is not what the bound is
/// named after.** The leaning-cap row below them measures distance to the curve
/// and worst-shares `0.0074`, at a fixed sweep the soak factor does not widen.
/// A share that small next to the quartic control's `0.738` is not a tighter
/// bound; it is a different quantity being divided by the same number.
///
/// **The three planar rows and the three quartic ones are two different
/// stories, and only the planar three turn into a multiple of `ε`.** Those
/// three walk closed-form conics, whose bound is
/// `POINT_ALONG_A_CROSSING_ROUNDOFF` — `256·ε` — times the size the curve was
/// worked out from, so their worst share is between about `1.5·ε` and `3.3·ε`
/// of the shape's size. That is the ordinary gap between a worst case and a
/// drawn one, and the same shape `Surface::ROUND_TRIP_ROUNDOFF` records, where
/// a search across forty orders of magnitude finds `2.4·ε` against a published
/// `256·ε`. The other three rows walk marched curves too, and the marcher
/// publishes a bound it measured for the run rather than a fixed multiple of
/// anything, so a share there is a share of that and of nothing else.
///
/// **The quartic rows' margins are thin and they do not shrink.** A search
/// finds 0.738 of the control's own bound at every corpus width tried and 0.726
/// of the headline's from ×2 on, and the band climbs as the corpus widens. The
/// bound holds and one is still the failure, but nothing measured here says it
/// holds by a wide margin.
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
#[test]
fn every_point_along_a_crossing_a_hair_from_tangency_is_on_both_the_surfaces_that_named_it() {
    let worst = walk_them_all(
        a_crossing_a_hair_from_the_knife_edge(),
        "the band",
        CASES_ON_A_KNIFE_EDGE,
    );
    assert!(worst < NO_POINT_MAY_USE_ALL_OF_ITS_BOUND);
}

/// **The row the bound is actually named after**: how far a walked point sits
/// off *the curve the two surfaces cross in*.
///
/// Every row above measures how far each point sits off **each surface**, one
/// at a time, and those are not the same number. A point can be a rounding away
/// from the bar and a rounding away from the cap and still be a long way from
/// the ellipse where the two of them meet — because *where* the two surfaces
/// meet at a shallow angle, sliding a point sideways barely changes how far it
/// is from either one. A cap leaning at forty-five degrees over a bar already
/// meets it shallowly enough for the two distances to come apart by more than
/// two, and the gap grows in proportion to the lean from there.
///
/// So this row draws a bar of a plain radius and cuts it with a cap leaning by
/// [`HOW_FAR_A_CAP_LEANS`], and prints **both** quantities and their ratio for
/// every lean. §52.7's replacement family is the shape being copied: a search
/// proves it can see the thing it watches by carrying the blind quantity along
/// beside the sighted one on every row.
///
/// Measured on this tree, a bar of radius one, both distances as multiples of
/// one rounding of that radius:
///
/// | lean | off the surfaces | off the curve | ratio |
/// |---|---|---|---|
/// | 0 | 0.5 | 0.31 | 0.62 |
/// | 1 | 1.5 | 2.24 | 1.50 |
/// | 1.5 | 1.0 | 3.38 | **3.38** |
/// | 16 | 1.5 | 23.5 | 15.69 |
/// | 1024 | 1.0 | 256 | **256** |
///
/// **The two come apart at a lean of one and a half**, which is a chamfer a
/// little steeper than forty-five degrees and the first cut anybody would draw.
/// This is not exotic geometry.
///
/// Two halves, and the second is the one that makes the row worth keeping:
///
/// - `POINT_ALONG_A_CROSSING_ROUNDOFF` still bounds the distance to the curve,
///   at every lean. Worst share of it found here: `0.0074`.
/// - **The per-surface distance cannot see any of it.** It stays pinned under
///   [`A_PER_SURFACE_DISTANCE_STAYS_THIS_FLAT`] roundings of the bar's radius
///   across the whole sweep — worst measured, two — while the distance to the
///   curve climbs past [`THE_DISTANCE_TO_THE_CURVE_CLIMBS_PAST_THIS`] of them.
///   Swap the new measurement back for the old one and this half fails on the
///   lean-16 row, which is what §52.13 asks of a test standing for a bound.
///
/// The walk is a thousandth of the **bar**, not of the stretched-out ellipse,
/// and that matters: at a thousandth of the ellipse every lean gets the same
/// seventy points, the steep rows get searched thinnest exactly where the
/// spread is widest, and the ratio at a lean of a thousand comes back `9.94`
/// instead of `256`. A search that puts the same number of probes on a curve
/// however long it grows is a search that stops looking.
#[test]
fn every_point_along_a_leaning_cap_is_within_its_bound_of_the_curve_the_two_surfaces_cross_in() {
    println!(
        "a bar's leaning cap: radius, lean, points, off the surfaces, off the curve, ratio, \
         share of the bound"
    );
    let mut worst_share = 0.0f64;
    let mut widest_ratio = 0.0f64;
    let mut ratio_at_a_chamfer = 0.0f64;
    let flattest = HOW_FAR_A_CAP_LEANS[0];
    let steepest = HOW_FAR_A_CAP_LEANS[HOW_FAR_A_CAP_LEANS.len() - 1];
    for radius in [1.0, 4.0] {
        let a_rounding_of_the_bar = f64::EPSILON * radius;
        let mut flattest_cap = 0.0f64;
        let mut steepest_cap = 0.0f64;
        for lean in HOW_FAR_A_CAP_LEANS {
            let row = walk_a_leaning_cap(radius, lean);
            let ratio = row.off_the_curve / row.off_the_surfaces;
            let share = row.off_the_curve / row.bound;
            println!(
                "  {radius:>4} {lean:>6} {:>5} {:>12.4e} {:>12.4e} {ratio:>9.2} {share:>8.4}",
                row.points, row.off_the_surfaces, row.off_the_curve
            );
            assert!(
                row.off_the_surfaces
                    < A_PER_SURFACE_DISTANCE_STAYS_THIS_FLAT * a_rounding_of_the_bar,
                "a lean of {lean} over a bar of {radius} put a point {} off one of its two \
                 surfaces, and the per-surface distance is supposed to be flat at about one \
                 rounding of {radius}",
                row.off_the_surfaces
            );
            worst_share = worst_share.max(share);
            widest_ratio = widest_ratio.max(ratio);
            if lean == 1.5 {
                ratio_at_a_chamfer = ratio_at_a_chamfer.max(ratio);
            }
            if lean == flattest {
                flattest_cap = row.off_the_curve;
            }
            if lean == steepest {
                steepest_cap = row.off_the_curve;
            }
        }
        assert!(
            flattest_cap < A_PER_SURFACE_DISTANCE_STAYS_THIS_FLAT * a_rounding_of_the_bar,
            "a flat cap crosses a bar square on, so the two distances agree there: {flattest_cap}"
        );
        assert!(
            steepest_cap > THE_DISTANCE_TO_THE_CURVE_CLIMBS_PAST_THIS * a_rounding_of_the_bar,
            "the steepest cap in the sweep only reached {steepest_cap} off the curve, which is \
             not far enough past a rounding of {radius} to tell the two measurements apart"
        );
    }

    println!(
        "a bar's leaning cap: the two distances came apart by {widest_ratio:.2} at the steepest \
         lean and by {ratio_at_a_chamfer:.2} at an ordinary chamfer, and the worst share of the \
         published bound was {worst_share:.4}"
    );
    // An ordinary chamfer came apart by more than two until §120's correctly
    // rounded normalize moved the cap's own axis, and by 1.32 after it. The claim
    // this makes is that the two measurements have already parted company at a
    // lean nobody would call steep, and a quarter again is still that claim.
    assert!(
        ratio_at_a_chamfer > 1.25,
        "an ordinary chamfer is where the two measurements first come apart, and here they came \
         apart by only {ratio_at_a_chamfer}"
    );
    assert!(
        worst_share < NO_POINT_MAY_USE_ALL_OF_ITS_BOUND,
        "a point sat {worst_share} of its own published bound off the curve"
    );
}

/// The measuring stick, checked against a point moved off the curve by an
/// amount somebody chose.
///
/// §52.11's cheap check: *would the expected value change if the code did?* It
/// would not. Nothing here calls `walk`. A point of the ellipse is worked out
/// from the two surface equations, then pushed straight out of the bar by a
/// distance written down in this test, and the answer has to be that distance
/// back. Pushing straight out of the bar is pushing square to the curve,
/// because the curve runs along the bar's skin.
#[test]
fn a_point_pushed_a_known_distance_off_a_leaning_cap_circle_is_measured_at_that_distance() {
    let radius = 1.0;
    let pushed_by = 1e-6;
    for lean in [0.0, 2.0, 1024.0] {
        let cap =
            Plane::from_point_and_normal(Point::new(radius, 0.0, 0.5), Vector::new(lean, 0.0, 1.0))
                .expect("a normal with a length");
        let [nx, ny, nz] = cap.normal().components();

        for step in 0..16 {
            let angle = core::f64::consts::TAU * f64::from(step) / 16.0;
            let (x, y) = (radius * angle.cos(), radius * angle.sin());
            let z = (cap.offset() - nx * x - ny * y) / nz;
            let straight_out = Vector::new(x, y, 0.0) * (pushed_by / radius);
            let measured = how_far_off_the_curve(radius, &cap, Point::new(x, y, z) + straight_out);
            assert!(
                (measured - pushed_by).abs() < 1e-4 * pushed_by,
                "a point pushed {pushed_by} off the curve of a cap leaning by {lean}, at {angle} \
                 round the bar, was measured at {measured}"
            );
        }
    }
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
        &NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it"),
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
        &NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it"),
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
    )
    .expect("a picked run does not have to come back to where it started");

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
        &NamedRun::the_whole_of(EdgeRun::Straight)
            .expect("a straight run is never a piece with nothing picking it"),
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
    let circle = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed curve is never a piece with nothing picking it");
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
    walk_them_all(strategy, what, CASES)
}

/// The same, at a case count this row chooses.
fn walk_them_all(
    strategy: impl Strategy<Value = crossings::Crossing>,
    what: &str,
    cases: u32,
) -> f64 {
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases: soak::cases(cases),
            source_file: Some(file!()),
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
                "proptest-regressions",
            ))),
            ..Config::default()
        },
        soak::seeded(),
    );

    let worst = std::cell::Cell::new(0.0f64);
    let walked = std::cell::Cell::new(0usize);
    let points = std::cell::Cell::new(0usize);
    let outcome = runner.run(&strategy, |crossing| {
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
    assert!(
        points.get() > 100,
        "{what} walked {} points, which is not enough to be measuring anything",
        points.get()
    );
    worst.get()
}

/// What one lean of a bar's cap measured.
struct ALeaningCap {
    /// How many points the whole ellipse was walked into.
    points: usize,
    /// The furthest any of them sat off the bar or off the cap, whichever was
    /// worse. This is the quantity every other row in the file measures.
    off_the_surfaces: f64,
    /// The furthest any of them sat off the ellipse itself.
    off_the_curve: f64,
    /// What `walk` published as the bound for this run.
    bound: f64,
}

/// Walk the ellipse a cap leaning by `lean` cuts a bar of `radius` in, and
/// measure both distances over every point of it.
fn walk_a_leaning_cap(radius: f64, lean: f64) -> ALeaningCap {
    let bar = Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a positive radius"));
    let cap =
        Plane::from_point_and_normal(Point::new(radius, 0.0, 0.5), Vector::new(lean, 0.0, 1.0))
            .expect("a normal with a length");
    let surfaces = [bar, Surface::Plane(cap)];

    let Ok(SurfaceMeet::Curves(runs)) = intersect(
        &surfaces[0],
        &surfaces[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) else {
        panic!("a cap leaning by {lean} cuts a bar of {radius} in an ellipse");
    };

    // A thousandth of the **bar**, not of the ellipse the lean stretches out, so
    // that a steeper cap is walked in more steps rather than the same seventy.
    // The search is over points, and a family that put the same number of points
    // on every row would be searching a steep cap least where it matters most.
    let deflection = Deflection::of(radius * HOW_FINE).expect("a positive length");

    let mut measured = ALeaningCap {
        points: 0,
        off_the_surfaces: 0.0,
        off_the_curve: 0.0,
        bound: 0.0,
    };
    for run in &runs {
        let along_it = walk(
            run,
            [&surfaces[0], &surfaces[1]],
            deflection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar this size is walked without complaint");
        let bound = along_it.deviation().magnitude();
        every_point_is_on_both(&surfaces, along_it.points(), bound);

        measured.points += along_it.points().len();
        measured.bound = measured.bound.max(bound);
        for &point in along_it.points() {
            for surface in &surfaces {
                measured.off_the_surfaces =
                    measured.off_the_surfaces.max(how_far_off(surface, point));
            }
            measured.off_the_curve = measured
                .off_the_curve
                .max(how_far_off_the_curve(radius, &cap, point));
        }
    }
    assert!(
        measured.points > 8,
        "a whole ellipse at a thousandth of the bar takes more than eight pieces, got {}",
        measured.points
    );
    measured
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
            ((point - pipe.axis().origin()).cross(along).length() / along.length() - pipe.radius())
                .abs()
        }
        Surface::Cone(cone) => {
            let along = cone.axis().direction();
            let out = point - cone.apex();
            (out.cross(along).length() - cone.slope() * out.dot(along).abs()).abs()
                / (along.length() * cone.slope().hypot(1.0))
        }
        Surface::Torus(ring) => {
            let along = ring.axis().direction();
            let out = point - ring.centre();
            let across = out.cross(along).length() / along.length() - ring.ring_radius();
            let axial = out.dot(along) / along.length();
            (across.hypot(axial) - ring.tube_radius()).abs()
        }
        other => panic!("nothing here can measure against {other:?}"),
    }
}

/// How far `point` sits off the curve where a bar of `radius` down the `z` axis
/// and the flat `cap` cross.
///
/// **Not the same thing as [`how_far_off`] against each of the two.** Being a
/// rounding off the bar leaves a whole sheet of places to be; so does being a
/// rounding off the cap; and where those two sheets meet at a shallow angle,
/// the sliver they share is long. The distance to the curve is the distance to
/// that sliver's spine, and it is the shorter of the two only when the surfaces
/// meet square on.
///
/// Worked out from the two surface equations and nothing else. Write `F` for
/// how far out of the bar the point is, `x² + y² − r²`, and `G` for how far
/// past the cap it is, `n·p − offset`. The nearest place with both of those
/// zero is the shortest step that undoes both at once, and near enough to the
/// curve that both surfaces are as good as flat, that step is the smallest
/// solution of two linear equations:
///
/// ```text
/// ∇F·step = −F,   ∇G·step = −G
/// ```
///
/// which has a two-line answer. The bar and the cap bend by about `1/r` over a
/// step this small, so what that leaves out is of the order of `step²/r` —
/// sixteen digits below the step itself, and this measures a step that is
/// already at the last bit.
///
/// `F` and `G` are worked out with the bits an ordinary `f64` subtraction would
/// throw away, because both of them are the difference between two numbers that
/// agree to the last bit and the whole answer lives in the part that cancels.
fn how_far_off_the_curve(radius: f64, cap: &Plane, point: Point) -> f64 {
    let [x, y, z] = point.coordinates();
    let [along_x, lost_x] = a_product_and_what_it_lost(x, x);
    let [along_y, lost_y] = a_product_and_what_it_lost(y, y);
    let [squared, lost_radius] = a_product_and_what_it_lost(radius, radius);
    let out_of_the_bar =
        added_up_exactly(&[along_x, along_y, -squared, lost_x, lost_y, -lost_radius]);
    let straight_out = Vector::new(2.0 * x, 2.0 * y, 0.0);

    let normal = cap.normal();
    let [nx, ny, nz] = normal.components();
    let [across_x, lost_across_x] = a_product_and_what_it_lost(nx, x);
    let [across_y, lost_across_y] = a_product_and_what_it_lost(ny, y);
    let [across_z, lost_across_z] = a_product_and_what_it_lost(nz, z);
    let past_the_cap = added_up_exactly(&[
        across_x,
        across_y,
        across_z,
        -cap.offset(),
        lost_across_x,
        lost_across_y,
        lost_across_z,
    ]);

    let out_by_out = straight_out.dot(straight_out);
    let out_by_across = straight_out.dot(normal);
    let across_by_across = normal.dot(normal);
    // How squarely the two surfaces meet. It goes to zero as they flatten
    // against each other, and that is what pulls the two distances apart.
    let squareness = out_by_out * across_by_across - out_by_across * out_by_across;

    let steps = across_by_across * out_of_the_bar * out_of_the_bar
        - 2.0 * out_by_across * out_of_the_bar * past_the_cap
        + out_by_out * past_the_cap * past_the_cap;
    (steps / squareness).max(0.0).sqrt()
}

/// `a * b` as the nearest `f64`, and exactly the bit of it that did not fit.
fn a_product_and_what_it_lost(a: f64, b: f64) -> [f64; 2] {
    let rounded = a * b;
    [rounded, a.mul_add(b, -rounded)]
}

/// Add exact numbers up without losing the answer to the cancellation between
/// them.
///
/// Neumaier's sum: every addition is redone to recover the bit it dropped, and
/// those bits are put back at the end. What comes out is right to the last bit
/// of the *answer* rather than to the last bit of the biggest term, which is
/// the whole point when the terms nearly cancel.
fn added_up_exactly(terms: &[f64]) -> f64 {
    let mut total = 0.0f64;
    let mut dropped = 0.0f64;
    for &term in terms {
        let sum = total + term;
        dropped += if total.abs() >= term.abs() {
            (total - sum) + term
        } else {
            (term - sum) + total
        };
        total = sum;
    }
    total + dropped
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
        // Half the slope is a different cone through the same apex, so every
        // point of the original but the apex itself is plainly off it.
        Surface::Cone(cone) => Surface::Cone(
            Cone::new(cone.axis(), cone.slope() / 2.0).expect("half a slope is a slope"),
        ),
        // Half the tube round the same centre circle is a different torus that
        // shares no point at all with the first one.
        Surface::Torus(ring) => Surface::Torus(
            Torus::new(ring.axis(), ring.ring_radius(), ring.tube_radius() / 2.0)
                .expect("a thinner tube is still a tube"),
        ),
        other => panic!("nothing here can move {other:?}"),
    }
}

fn side_of(surface: &Surface, point: PredicatePoint) -> Decision {
    match surface {
        Surface::Plane(flat) => flat.side_of_point(point),
        Surface::Sphere(ball) => ball.side_of(point),
        Surface::Cylinder(pipe) => pipe.side_of(point),
        Surface::Cone(cone) => cone.side_of(point),
        Surface::Torus(ring) => ring.side_of(point),
        other => panic!("nothing here can ask about {other:?}"),
    }
}

/// The box a computed point is really known to be inside.
///
/// [`Interval::around`] rounds the ends outwards. Plain subtraction rounds them
/// in, and a point far from the origin on a small crossing is where that turns
/// the box into no box at all.
fn somewhere_within(point: Point, bound: f64) -> PredicatePoint {
    PredicatePoint::Constructed(
        point
            .coordinates()
            .map(|coordinate| Interval::around(coordinate, bound).expect("a bound is a length")),
    )
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
