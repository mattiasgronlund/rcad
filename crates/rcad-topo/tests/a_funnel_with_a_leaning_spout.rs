//! **Two cones that share neither their axis nor their apex**, queue row
//! Q2667.
//!
//! The two cross in a curve of degree four, and `intersect` used to refuse
//! every such pair `ACrossingBetweenAConeAndAnotherCurvedSurface`, so every
//! fuse of the first two shapes here refused before any run was walked:
//!
//! - a funnel with a narrow spout leaning across it, every straight line of
//!   which runs into the funnel's skin and out again, so the two cross in two
//!   loops, one each side of a flat surface;
//! - two pointed cones of one slope, one leaning a little off the other, whose
//!   own directions overlap, so they cross in branches that run off for ever,
//!   and which touch at two points, being mirror images of each other.
//!
//! Since row Q2667 `intersect` names the funnel and spout's two loops and
//! `walk` marches them, and since row Q2807 the fuse orders the points of each
//! loop round the spout's axis, so every fuse of the funnel and the spout
//! answers. **What each fuse gives now is pinned here**, under every rule both
//! ways round. Since row Q2893 `intersect` names the two pointed cones'
//! crossing too, as one loop and two branches that pass close by each other,
//! and since row Q2894 `walk` marches the loop and a run between two corners
//! on a branch without stepping from one onto the other, but every fuse of
//! them is still refused. Where both caps' rims nearly meet, the two caps'
//! planes and the two cones meet in four points a rounding apart, not one.
//! Since row Q2961 the fuse orders those by exact signs and gets past them,
//! and is refused instead ordering points along a branch that reaches each
//! cone's wall there.
//!
//! A third shape came with row Q2818: two pointed cones of different slopes,
//! the narrower leaning off the wider, which cross in a loop and two branches
//! that run off for ever. No one flat surface picks that loop out, so
//! `intersect` names it with two, and the fuse used to panic at the second.
//! Since Q2818 a closed crossing is named by every picker it has, and all six
//! fuses answer.
//!
//! A fourth came with row Q2813: the two pointed cones of one slope again, the
//! second a quarter turn off the first, so that every number is exact and the
//! two are exactly mirror images and touch. Since row Q2868 `intersect` names
//! their crossing as four arcs of two flat cuts that meet where the two touch,
//! since row Q2886 the fuse puts a node at each touch and tells each face's
//! regions apart, and since row Q2902 it names each edge of the answer from
//! its end away from a touch, so every fuse of the pair answers and measures
//! what the two share. A fifth came with it: the same two side by side, which
//! are mirror images too but cross in one hyperbola and do not touch. Since
//! row Q2867 `intersect` names that hyperbola as the flat surface halfway
//! between the two cuts it out of either, and since row Q2879 the fuse orders
//! points along it as along that flat surface's cut of a cone, so every fuse
//! of the pair answers.
//!
//! The last came with row Q2914: the quarter turn again, with cones of slope
//! two, so each apex is inside the other and the two cross in the whole curves
//! of two flat cuts that do not touch. Since Q2914 every fuse answers where
//! one is eight high and the other ten. Both eight high, both caps' rims meet
//! on the flat surface halfway, where four surfaces meet in one point, and
//! since row Q2997 the fuse proves the meets named there one point, so every
//! fuse of that pair answers too.
//!
//! Each answer that comes back is to be certified, sound, and measure as the
//! two solids alone bound it and as the others that came back say it must: the
//! union and the intersection together hold each solid once, and each solid
//! less the other is that solid less the intersection.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

/// **A funnel with a leaning spout.** The funnel is a frustum of slope a half
/// with its apex at the origin, cut two and eight up, so of radius one and
/// four. The spout is a frustum of slope an eighth cut two and twelve up, so of
/// radius a quarter and one and a half, leaned 0.8 radians about the `x` axis
/// through the point four up the funnel's axis.
///
/// The two cross in two loops, and each runs once round the spout's axis, so
/// the fuse orders the points of a loop by how far round that axis they lie,
/// since row Q2807. Every fuse answers, under every rule both ways round.
#[test]
fn a_funnel_fused_with_a_spout_leaning_across_it_answers_every_fuse() {
    assert_eq!(every_fuse_of(funnel_and_spout).refusals(), [[None; 2]; 3]);
}

/// **Two pointed cones, one leaning off the other.** Both have slope a half and
/// height eight, and the second leans 0.3 radians about the `x` axis through
/// the point four up the first's axis, less than either's half-angle of about
/// 0.46 radians.
///
/// Turned about a point of the first's axis, the second is the first's mirror
/// image across the flat surface halfway between their apexes, so the two
/// **touch** at `(±1.5954, 0.1209, 3.2)`, inside both faces, and their crossing
/// runs through itself there. The turn's `sin` and `cos` are rounded, so the
/// two are not exact mirror images, and the branches pass by each other there
/// rather than through: about `3e-8` apart for the same turn written out with
/// `sin` and `cos` rounded once each, as `rcad-geom`'s tests write it.
/// `two_cones_of_one_slope_one_turned_about_a_point_of_the_others_axis_touch_at_two_points`
/// in `rcad-geom`
/// shows the touch on a pair whose numbers are exact, and
/// [`two_pointed_cones_eight_high_a_quarter_turn_apart_answer_every_fuse`]
/// on the same two cones a quarter turn apart, where every number is exact.
///
/// Since row Q2893 `intersect` names the crossing as one loop and two branches
/// that pass close by each other near each touch of the exact pair, with a
/// flat surface it proves runs between the loop and the branch there. Since
/// row Q2894 `walk` marches the loop right round, and a run between two
/// corners on a branch, and keeps every step in front of the flat surfaces
/// that hold the piece it is on apart from the others, so it no longer steps
/// across the gap onto the branch and runs off along it.
///
/// Where both caps' rims nearly meet, at about `(−3.9541, −0.6045, 8)`, the
/// two are mirror images but for a rounding, and equally high, so each rim is
/// nearly the other's mirror image. Row Q2910 traced it. There `z = 8`, the
/// leaning cap's plane and the two cones meet in four points, not one: each
/// three of the four surfaces meet once, and worked out exactly from the
/// stored numbers the four points are all within `4.3e-16` of each other. The
/// fuse finds each as a node of its own on each face it lies on, and orders
/// them along each crossing through them.
///
/// - Round the upright cap's rim from `(0, 4, 8)`, anticlockwise about `z`,
///   the meet with the leaning cone comes `4.15e-16` before the meet with the
///   leaning cap's plane. Since Q2910 the fuse reads that by how far round the
///   upright cone's axis each is turned.
/// - Along the line where the two caps' planes cross, the meet with the
///   leaning cone lies `1.84e-16` further along `−x` than the meet with the
///   upright cone. Both lie in one box an `ulp` wide along `x` and along `y`,
///   and `y` and `z` do not change along that line, so no box parts them.
/// - Round the leaning cap's rim, the meet with the upright cone and the meet
///   with `z = 8` are `4.15e-16` apart.
///
/// Since row Q2961 the fuse orders those last two pairs by the exact sign of
/// one cone at a meet of that line with the other, and puts the meet of that
/// line with the upright cone, a node of a walk on `z = 8`, inside the leaning
/// cone by the same exact sign. Until then every fuse was refused undecided
/// there.
///
/// **Every fuse is still refused**, under every rule both ways round, now on
/// each cone's wall: of the loop and the two branches `intersect` names the
/// crossing as, the branch beyond the flat surface that holds it apart from
/// the loop reaches the wall near those four points, and nothing puts points in
/// order along a branch that runs off for ever. The loop has a start
/// `where_a_closed_crossing_starts` finds; both branches it refuses
/// `ACrossingBetweenAConeAndAnotherCurvedSurface`.
#[test]
fn two_pointed_cones_one_leaning_off_the_other_are_refused_ordering_along_a_branch() {
    assert_eq!(
        every_fuse_of(two_pointed_cones).refusals(),
        [[Some(NOTHING_ORDERS_ALONG_A_BRANCH); 2]; 3]
    );
}

/// **The same two, the leaning one ten high**, row Q2917, so the two caps'
/// rims are apart. `intersect` names the crossing as one loop and two
/// branches that nearly meet, as for the pair both eight high. Every fuse is
/// to answer, under every rule both ways round. Until the fuse orders points
/// along the loop and each branch round the ruled cone's axis, every fuse is
/// refused as a crossing nothing can put points in order along.
#[test]
fn two_pointed_cones_eight_and_ten_high_one_leaning_off_the_other_answer_every_fuse() {
    assert_eq!(
        every_fuse_of(two_pointed_cones_eight_and_ten_high).refusals(),
        [[None; 2]; 3]
    );
}

/// What a fuse is refused with where nothing puts points in order along a
/// curve it is cut by.
const NOTHING_ORDERS_ALONG_A_BRANCH: &str = "nothing can put points in order along this curve";

/// **Two pointed cones, one a quarter turn off the other.** Both have slope a
/// half and height eight, and the second turns a quarter about the `x` axis
/// through the point four up the first's axis, which puts its apex at
/// `(0, 4, 4)` and its axis along `-y`, every number exact.
///
/// The second is then exactly the first's mirror image across the flat
/// surface `y + z = 4` halfway between their apexes, and the two **touch** at
/// `(±√1.92, 0.8, 3.2)`, inside both faces, where their crossing runs through
/// itself. Since row Q2868 `intersect` names that crossing as four arcs, from
/// touch to touch, of the two ellipses `y + z = 4` and `z − y = 2.4` cut out
/// of either cone, and `walk` marches them. Since row Q2886 the fuse puts a
/// node at each touch on both walls and orders each arc round a point of the
/// straight line through the touches, so it tells each face's regions apart.
///
/// The two are tangent at a touch, so which way the crossing runs there
/// cannot be read off the touch's rounded point, where `∇S × ∇T` is a
/// rounding. Read there, an edge from a touch was picked by that rounding, and
/// the union with the other solid first measured `273.28`, more than the
/// `268.08` the two cones hold together. Since row Q2902 `fuse/assemble.rs`
/// knows each touch from how the arrangement named it, as it knows two pipes'
/// (`docs/decisions.md` §448.3), and reads the way at an edge's other end.
/// Every fuse answers, under every rule both ways round, and the intersection
/// measures what the two share.
///
/// Eight high, the ellipse in `y + z = 4` reaches up to `(0, −4, 8)`, where
/// both caps' rims pass.
#[test]
fn two_pointed_cones_eight_high_a_quarter_turn_apart_answer_every_fuse() {
    every_fuse_answers_and_the_intersection_is_what_the_two_share(
        two_pointed_cones_a_quarter_turn_apart,
        (
            what_two_cones_a_quarter_turn_apart_share(0.5, 8.0, 8.0),
            0.01,
        ),
    );
}

/// **The same two pointed cones, ten high.** Neither cap meets the other
/// solid, so the crossing is the two ellipses alone. Every fuse answers, and
/// the intersection measures what the two share, which lies below both caps
/// and so is what the eight-high pair share.
#[test]
fn two_pointed_cones_ten_high_a_quarter_turn_apart_answer_every_fuse() {
    every_fuse_answers_and_the_intersection_is_what_the_two_share(
        two_pointed_cones_ten_high_a_quarter_turn_apart,
        (
            what_two_cones_a_quarter_turn_apart_share(0.5, 10.0, 10.0),
            0.01,
        ),
    );
}

/// Fails unless every fuse of the two cones `shape` builds a quarter turn
/// apart answers, and the intersection measures `shared`, what
/// [`what_two_cones_a_quarter_turn_apart_share`] adds up and how far that sum
/// can be from the volume.
fn every_fuse_answers_and_the_intersection_is_what_the_two_share(
    shape: fn(&mut Model) -> (SolidKey, SolidKey),
    shared: (f64, f64),
) {
    let fused = every_fuse_of(shape);
    assert_eq!(fused.refusals(), [[None; 2]; 3]);
    let [Ok(intersection), _] = fused.answers[1] else {
        unreachable!("every fuse answered")
    };
    about_the_same(
        intersection,
        shared,
        "the intersection against what the two share",
    );
}

/// What two pointed cones of `slope` a quarter turn apart share, the upright
/// one `upright_high` high and the turned one `turned_high`, added up a slice
/// at a time: [`two_pointed_cones_this_high_a_quarter_turn_apart`] builds them
/// at slope a half, and [`two_steep_cones_a_quarter_turn_apart`] at slope two.
///
/// The first holds `x² + y² ≤ (s z)²` for `z` from nought to `upright_high`,
/// and the second is its mirror image across `y + z = 4`, which takes
/// `(x, y, z)` to `(x, 4 − z, 4 − y)`. So at height `z` and across at `y` the
/// two share the `x` with `x² ≤ (s z)² − y²` and
/// `x² ≤ (s (4 − y))² − (4 − z)²`, where `4 − y` is from nought to
/// `turned_high`. Taken at the middle of each of 1000 steps up and 1000 across,
/// the sum at slope a half, both eight high, is `46.2231`; it moves by
/// `1.5e−3` from 500 steps and by `4.8e−4` on to 2000, so it is within a
/// hundredth of the volume. At slope two, eight and ten high, it is `805.998`,
/// and ten and eight high `805.957`, though the two are mirror images: each
/// moves by about `0.03` from 500 steps and by `0.011` on to 2000, where the
/// two are `0.019` apart, so each is within five hundredths of the volume.
/// Both eight high it is `608.821`: it moves by `0.064` from 500 steps and by
/// `0.0019` on to 2000, and is `0.0092` short of the sum at 16000 steps, which
/// moves by `2e−4` from 8000, so it is within five hundredths too.
fn what_two_cones_a_quarter_turn_apart_share(
    slope: f64,
    upright_high: f64,
    turned_high: f64,
) -> f64 {
    const STEPS: u32 = 1000;
    let up = upright_high / f64::from(STEPS);
    (0..STEPS)
        .map(|step_up| {
            let z = (f64::from(step_up) + 0.5) * up;
            let radius = slope * z;
            let across = 2.0 * radius / f64::from(STEPS);
            (0..STEPS)
                .map(|step_across| {
                    let y = -radius + (f64::from(step_across) + 0.5) * across;
                    let up_the_second = 4.0 - y;
                    if !(0.0..=turned_high).contains(&up_the_second) {
                        return 0.0;
                    }
                    let in_the_first = radius * radius - y * y;
                    let in_the_second = (slope * up_the_second).powi(2) - (4.0 - z).powi(2);
                    2.0 * in_the_first.min(in_the_second).max(0.0).sqrt() * across * up
                })
                .sum::<f64>()
        })
        .sum()
}

/// **Two pointed cones side by side.** Both have slope a half and height
/// eight, and the second stands three along `x` from the first.
///
/// They are mirror images across `x = 1.5` too, but do not touch: their skins
/// cross in one hyperbola in that flat surface, from `(1.5, 0, 3)` up to the
/// two tops. Since row Q2867 `intersect` names it, as the cut of `x = 1.5`
/// through the first cone, and `walk` marches it. Since row Q2879 the fuse
/// orders the points along it as it orders a plane's endless bend across a
/// cone, by that flat surface and that cone, and every fuse answers, under
/// every rule both ways round.
///
/// The intersection is two halves of one shape either side of `x = 1.5`: the
/// part of the first cone past it, which at height `2r` is the piece of a
/// circle of radius `r` past a line `c = 1.5` from its centre, of area
/// `r² acos(c/r) − c√(r² − c²)`, for `r` from `c` up to the top's four.
/// Taken up the height that is
/// `4[(r³/3) acos(c/r) − (2c/3) r√(r² − c²) + (c³/3) ln((r + √(r² − c²))/c)]`
/// at `r = 4`, about 49.2755, which the intersection measures within its
/// error.
#[test]
fn two_pointed_cones_side_by_side_do_not_touch_and_answer_every_fuse() {
    let fused = every_fuse_of(two_pointed_cones_side_by_side);
    assert_eq!(fused.refusals(), [[None; 2]; 3]);

    let (c, r) = (1.5_f64, 4.0_f64);
    let across = (r * r - c * c).sqrt();
    let shared = 4.0
        * (r.powi(3) / 3.0 * (c / r).acos() - 2.0 * c / 3.0 * r * across
            + c.powi(3) / 3.0 * ((r + across) / c).ln());
    let [Ok(intersection), _] = fused.answers[1] else {
        unreachable!("every fuse answered")
    };
    about_the_same(
        intersection,
        (shared, 0.0),
        "the intersection against what the two share",
    );
}

/// **Two pointed cones of different slopes, the narrower leaning off the
/// wider.** Both are eight high. The upright one has slope a half, the leaning
/// one a quarter, and the leaning one turns 0.3 radians about the `x` axis
/// through the point four up the upright one's axis.
///
/// The two cross in a loop and in two branches that run off for ever both ways
/// along the same two rulings, so no one flat surface holds the loop and none
/// of either branch: `intersect` picks the loop out with two flat surfaces
/// square to the leaning axis, one through the leaning cone's apex and one,
/// facing the other way, through the point the turn is about. The fuse
/// panicked at the second picker before row Q2818.
///
/// Every fuse answers, under every rule both ways round.
#[test]
fn two_pointed_cones_of_two_slopes_crossing_in_a_loop_named_by_two_flats_answer_every_fuse() {
    assert_eq!(
        every_fuse_of(pointed_cones_of_two_slopes).refusals(),
        [[None; 2]; 3]
    );
}

/// **Two pointed cones of slope two, one a quarter turn off the other**, both
/// eight high, the second turned a quarter about the `x` axis through
/// `(0, 0, 4)`, row Q2914. Each apex is inside the other, so the two are
/// mirror images across `y + z = 4` that touch nowhere, and since row Q2878
/// `intersect` names their crossing as the whole curves of two flat cuts held
/// apart, `y + z = 4` and `y − z = 2.4`, both hyperbolas. Both caps' rims meet
/// on the halfway cut at `(±√240, −4, 8)`.
///
/// Along the upright cap's rim the meet of the cap with both cones and the
/// meet of both caps with the upright cone are the one point `(√240, −4, 8)`,
/// where four surfaces meet: the line the two caps cross in lies in
/// `y + z = 4`, on which both cones cut one curve, and so along the turned
/// cap's rim. Every fuse was refused there on one sign before row Q2997, since
/// nothing in `fuse/arrange.rs` proved two such meets one point.
///
/// Since row Q2997 the fuse proves them one point where both are on one line
/// in `y + z = 4` and on both cones, and either cone's gradient along that
/// line keeps one sign over the box round the two. That settles the pairs on
/// each cap's rim, on the line the two caps cross in, and on the curve the two
/// cones cross in, where the meet of one cap with both cones and the meet of
/// the other cap with both are one point. Every fuse answers, under every rule
/// both ways round, and the intersection measures what the two share.
#[test]
fn two_steep_cones_eight_high_a_quarter_turn_apart_answer_every_fuse() {
    every_fuse_answers_and_the_intersection_is_what_the_two_share(
        two_steep_cones_eight_high_a_quarter_turn_apart,
        (
            what_two_cones_a_quarter_turn_apart_share(2.0, 8.0, 8.0),
            0.05,
        ),
    );
}

/// **The same two of slope two, the turned one ten high**, so only the
/// upright one's cap meets the other solid, and the halfway cut runs from that
/// cap's rim to its rim. Every fuse answers, either way round, and the
/// intersection measures what the two share.
///
/// Before row Q2914 every fuse was refused as a crossing nothing could put
/// points in order along. Ordered along each flat cut's curve as that flat's
/// bend across the cone, every fuse was refused `ResultIsNotManifold`: the
/// turned cone's piece between the halfway cut and the upright cap was placed
/// by a point in the turned cone beyond the other flat cut, `y − z = 2.4`.
/// `fuse/select.rs` now asks such a point to be on the piece's side of both
/// flat cuts.
#[test]
fn two_pointed_cones_of_slope_two_eight_and_ten_high_a_quarter_turn_apart_answer_every_fuse() {
    for (shape, upright_high, turned_high) in [
        (
            two_steep_cones_eight_and_ten_high_a_quarter_turn_apart as fn(&mut Model) -> _,
            8.0,
            10.0,
        ),
        (
            two_steep_cones_ten_and_eight_high_a_quarter_turn_apart,
            10.0,
            8.0,
        ),
    ] {
        every_fuse_answers_and_the_intersection_is_what_the_two_share(
            shape,
            (
                what_two_cones_a_quarter_turn_apart_share(2.0, upright_high, turned_high),
                0.05,
            ),
        );
    }
}

fn two_steep_cones_eight_high_a_quarter_turn_apart(model: &mut Model) -> (SolidKey, SolidKey) {
    two_steep_cones_a_quarter_turn_apart(model, 8.0, 8.0)
}

fn two_steep_cones_eight_and_ten_high_a_quarter_turn_apart(
    model: &mut Model,
) -> (SolidKey, SolidKey) {
    two_steep_cones_a_quarter_turn_apart(model, 8.0, 10.0)
}

fn two_steep_cones_ten_and_eight_high_a_quarter_turn_apart(
    model: &mut Model,
) -> (SolidKey, SolidKey) {
    two_steep_cones_a_quarter_turn_apart(model, 10.0, 8.0)
}

fn two_steep_cones_a_quarter_turn_apart(
    model: &mut Model,
    upright_high: f64,
    turned_high: f64,
) -> (SolidKey, SolidKey) {
    let upright = a_pointed_cone(model, 2.0, upright_high);
    let turned = a_pointed_cone(model, 2.0, turned_high);
    let through =
        Axis::new(Point::new(0.0, 0.0, 4.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
    model
        .place_solid(
            turned,
            &Isometry::quarter_turns(&through, 1).expect("an axis along x"),
        )
        .expect("this model made the key");
    (upright, turned)
}

fn funnel_and_spout(model: &mut Model) -> (SolidKey, SolidKey) {
    let funnel = a_frustum(model, 0.5, 2.0, 8.0);
    let spout = a_frustum(model, 0.125, 2.0, 12.0);
    let through =
        Axis::new(Point::new(0.0, 0.0, 4.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
    model
        .place_solid(spout, &Isometry::rotation(&through, 0.8).expect("a turn"))
        .expect("this model made the key");
    (funnel, spout)
}

fn two_pointed_cones(model: &mut Model) -> (SolidKey, SolidKey) {
    let upright = a_pointed_cone(model, 0.5, 8.0);
    let leaning = a_pointed_cone(model, 0.5, 8.0);
    let through =
        Axis::new(Point::new(0.0, 0.0, 4.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
    model
        .place_solid(leaning, &Isometry::rotation(&through, 0.3).expect("a turn"))
        .expect("this model made the key");
    (upright, leaning)
}

fn two_pointed_cones_eight_and_ten_high(model: &mut Model) -> (SolidKey, SolidKey) {
    let upright = a_pointed_cone(model, 0.5, 8.0);
    let leaning = a_pointed_cone(model, 0.5, 10.0);
    let through =
        Axis::new(Point::new(0.0, 0.0, 4.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
    model
        .place_solid(leaning, &Isometry::rotation(&through, 0.3).expect("a turn"))
        .expect("this model made the key");
    (upright, leaning)
}

fn two_pointed_cones_a_quarter_turn_apart(model: &mut Model) -> (SolidKey, SolidKey) {
    two_pointed_cones_this_high_a_quarter_turn_apart(model, 8.0)
}

fn two_pointed_cones_ten_high_a_quarter_turn_apart(model: &mut Model) -> (SolidKey, SolidKey) {
    two_pointed_cones_this_high_a_quarter_turn_apart(model, 10.0)
}

fn two_pointed_cones_this_high_a_quarter_turn_apart(
    model: &mut Model,
    high: f64,
) -> (SolidKey, SolidKey) {
    let upright = a_pointed_cone(model, 0.5, high);
    let turned = a_pointed_cone(model, 0.5, high);
    let through =
        Axis::new(Point::new(0.0, 0.0, 4.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
    model
        .place_solid(
            turned,
            &Isometry::quarter_turns(&through, 1).expect("an axis along x"),
        )
        .expect("this model made the key");
    (upright, turned)
}

fn two_pointed_cones_side_by_side(model: &mut Model) -> (SolidKey, SolidKey) {
    let one = a_pointed_cone(model, 0.5, 8.0);
    let beside = a_pointed_cone(model, 0.5, 8.0);
    model
        .place_solid(
            beside,
            &Isometry::translation(Vector::new(3.0, 0.0, 0.0)).expect("a move"),
        )
        .expect("this model made the key");
    (one, beside)
}

fn pointed_cones_of_two_slopes(model: &mut Model) -> (SolidKey, SolidKey) {
    let upright = a_pointed_cone(model, 0.5, 8.0);
    let leaning = a_pointed_cone(model, 0.25, 8.0);
    let through =
        Axis::new(Point::new(0.0, 0.0, 4.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
    model
        .place_solid(leaning, &Isometry::rotation(&through, 0.3).expect("a turn"))
        .expect("this model made the key");
    (upright, leaning)
}

/// What [`every_fuse_of`] got: for the union, the intersection and the
/// difference, in that order, the first solid first and then the other first,
/// the volume and the error `measure` publishes with it, or the words of the
/// refusal.
struct Fused {
    answers: [[Result<(f64, f64), String>; 2]; 3],
}

impl Fused {
    /// The words of each refusal, and `None` for each fuse that answered.
    fn refusals(&self) -> [[Option<&str>; 2]; 3] {
        self.answers.each_ref().map(|rule| {
            rule.each_ref()
                .map(|answer| answer.as_ref().err().map(String::as_str))
        })
    }
}

/// Fuses the two solids `shape` builds under every rule both ways round, and
/// checks that each answer that comes back is certified, sound, and measures
/// as the two solids alone bound it and as the others that came back say it
/// must: a union no less than the larger solid and no more than the two
/// together, an intersection no more than the smaller, and a difference no
/// more than its argument.
fn every_fuse_of(shape: fn(&mut Model) -> (SolidKey, SolidKey)) -> Fused {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let (first, other) = shape(&mut model);
    let alone = |solid: SolidKey| what_they_measure(&model, &[solid]);
    let fuse = |rule: Rule, argument: SolidKey, tool: SolidKey| -> Result<(f64, f64), String> {
        let mut answer = model.clone();
        let mut log = ProvenanceLog::new();
        let fused = answer
            .fuse(&[argument], &[tool], rule, &policy, &mut log)
            .map_err(|why| why.to_string())?;
        assert!(log.is_empty(), "a certified fuse logs nothing: {log:?}");
        answer
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| {
                panic!(
                    "{rule:?}, the first solid first {}: the answer is sound: {why}",
                    argument == first
                )
            });
        Ok(what_they_measure(&answer, fused.solids()))
    };
    let both_ways = |rule: Rule| [fuse(rule, first, other), fuse(rule, other, first)];
    let answers = [
        both_ways(Rule::Union),
        both_ways(Rule::Intersection),
        both_ways(Rule::Difference),
    ];

    let [unions, intersections, differences] = &answers;
    let both_alone = [alone(first), alone(other)];
    let together = (
        both_alone[0].0 + both_alone[1].0,
        both_alone[0].1 + both_alone[1].1,
    );
    let [smaller, larger] = if both_alone[0].0 <= both_alone[1].0 {
        both_alone
    } else {
        [both_alone[1], both_alone[0]]
    };
    for (way, order) in ["the first solid first", "the other first"]
        .into_iter()
        .enumerate()
    {
        if let Ok(&union) = unions[way].as_ref() {
            no_more_than(
                larger,
                union,
                &format!("{order}: the larger solid alone against the union"),
            );
            no_more_than(
                union,
                together,
                &format!("{order}: the union against the two alone"),
            );
        }
        if let Ok(&intersection) = intersections[way].as_ref() {
            no_more_than(
                intersection,
                smaller,
                &format!("{order}: the intersection against the smaller solid alone"),
            );
        }
        if let Ok(&less) = differences[way].as_ref() {
            no_more_than(
                less,
                both_alone[way],
                &format!("{order}: the difference against its argument alone"),
            );
        }
    }
    for (way, order) in ["the first solid first", "the other first"]
        .into_iter()
        .enumerate()
    {
        let Ok(&intersection) = intersections[way].as_ref() else {
            continue;
        };
        assert!(
            intersection.0 > intersection.1,
            "{order}: the two overlap, so the intersection holds something: {intersection:?}"
        );
        if let Ok(&the_other_way) = intersections[1 - way].as_ref() {
            about_the_same(
                the_other_way,
                intersection,
                &format!("{order}: the intersection against its other order's"),
            );
        }
        if let Ok(&union) = unions[way].as_ref() {
            about_the_same(
                (union.0 + intersection.0, union.1 + intersection.1),
                together,
                &format!("{order}: the union and the intersection against the two alone"),
            );
        }
        if let Ok(&less) = differences[way].as_ref() {
            let argument_alone = both_alone[way];
            about_the_same(
                less,
                (
                    argument_alone.0 - intersection.0,
                    argument_alone.1 + intersection.1,
                ),
                &format!("{order}: the difference against the argument less the intersection"),
            );
        }
    }
    Fused { answers }
}

/// Fails unless `got` and `want`, each a volume and the error published with
/// it, are within their errors of each other.
fn about_the_same(got: (f64, f64), want: (f64, f64), what: &str) {
    assert!(
        (got.0 - want.0).abs() <= got.1 + want.1,
        "{what}: {} give or take {} against {} give or take {}",
        got.0,
        got.1,
        want.0,
        want.1
    );
}

/// Fails unless `small`, a volume and the error published with it, can be no
/// more than `big` within their errors.
fn no_more_than(small: (f64, f64), big: (f64, f64), what: &str) {
    assert!(
        small.0 - small.1 <= big.0 + big.1,
        "{what}: {} give or take {} is more than {} give or take {}",
        small.0,
        small.1,
        big.0,
        big.1
    );
}

/// The volume of `solids` together, and the error `measure` publishes with it.
fn what_they_measure(model: &Model, solids: &[SolidKey]) -> (f64, f64) {
    solids.iter().fold((0.0, 0.0), |(volume, error), solid| {
        let measured = model
            .solid(*solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a hundredth"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a cone measures");
        (volume + measured.volume(), error + measured.volume_error())
    })
}
