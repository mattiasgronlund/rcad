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
//! ways round: the two pointed cones are still refused as a cone crossing
//! another curved surface.
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
//! are mirror images too but cross in one hyperbola and do not touch. Since row Q2867 `intersect` names that hyperbola as the flat surface
//! halfway between the two cuts it out of either, and since row Q2879 the fuse
//! orders points along it as along that flat surface's cut of a cone, so every
//! fuse of the pair answers.
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

/// What `fuse` says where `intersect` cannot name how a cone crosses a second
/// curved surface.
const A_CONE_ACROSS_A_CURVED_SURFACE: &str =
    "geometry: a cone crossing another curved surface is a degree-four space curve";

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
/// Every fuse is refused where `intersect` names the crossing.
#[test]
fn two_pointed_cones_one_leaning_off_the_other_are_refused_by_every_rule() {
    assert_eq!(
        every_fuse_of(two_pointed_cones).refusals(),
        [[Some(A_CONE_ACROSS_A_CURVED_SURFACE); 2]; 3]
    );
}

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
        8.0,
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
        10.0,
    );
}

/// Fails unless every fuse of the two cones `shape` builds `high` high a
/// quarter turn apart answers, and the intersection measures what
/// [`what_two_cones_a_quarter_turn_apart_share`] adds up.
fn every_fuse_answers_and_the_intersection_is_what_the_two_share(
    shape: fn(&mut Model) -> (SolidKey, SolidKey),
    high: f64,
) {
    let fused = every_fuse_of(shape);
    assert_eq!(fused.refusals(), [[None; 2]; 3]);
    let [Ok(intersection), _] = fused.answers[1] else {
        unreachable!("every fuse answered")
    };
    about_the_same(
        intersection,
        (what_two_cones_a_quarter_turn_apart_share(high), 0.01),
        "the intersection against what the two share",
    );
}

/// What the two pointed cones [`two_pointed_cones_this_high_a_quarter_turn_apart`]
/// builds `high` high share, added up a slice at a time.
///
/// The first holds `x² + y² ≤ (z/2)²` for `z` from nought to `high`, and the
/// second is its mirror image across `y + z = 4`, which takes `(x, y, z)` to
/// `(x, 4 − z, 4 − y)`. So at height `z` and across at `y` the two share the
/// `x` with `x² ≤ (z/2)² − y²` and `x² ≤ ((4 − y)/2)² − (4 − z)²`, where
/// `4 − y` is from nought to `high`. Taken at the middle of each of 1000 steps
/// up and 1000 across, the sum is `46.2231`; it moves by `1.5e−3` from 500
/// steps and by `4.8e−4` on to 2000, so it is within a hundredth of the
/// volume.
fn what_two_cones_a_quarter_turn_apart_share(high: f64) -> f64 {
    const STEPS: u32 = 1000;
    let up = high / f64::from(STEPS);
    (0..STEPS)
        .map(|step_up| {
            let z = (f64::from(step_up) + 0.5) * up;
            let radius = z / 2.0;
            let across = 2.0 * radius / f64::from(STEPS);
            (0..STEPS)
                .map(|step_across| {
                    let y = -radius + (f64::from(step_across) + 0.5) * across;
                    let up_the_second = 4.0 - y;
                    if !(0.0..=high).contains(&up_the_second) {
                        return 0.0;
                    }
                    let in_the_first = radius * radius - y * y;
                    let in_the_second = (up_the_second / 2.0).powi(2) - (4.0 - z).powi(2);
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
