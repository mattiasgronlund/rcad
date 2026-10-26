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
//! `walk` marches them, and those fuses get further. **What each fuse gives now
//! is pinned here**, under every rule both ways round, for the row after Q2667
//! to turn round: the funnel and the spout are refused where a crossing is asked
//! to be a conic, and the two pointed cones are still refused as a cone
//! crossing another curved surface.
//!
//! A third shape came with row Q2818: two pointed cones of different slopes,
//! the narrower leaning off the wider, which cross in a loop and two branches
//! that run off for ever. No one flat surface picks that loop out, so
//! `intersect` names it with two, and the fuse used to panic at the second.
//! Since Q2818 a closed crossing is named by every picker it has, and three
//! of the six fuses answer; the other three are refused where a wire will not
//! lie flat in its face (§43.6).
//!
//! Each answer that comes back is to be certified, sound, and measure as the
//! others that came back say it must: the union and the intersection together
//! hold each solid once, and each solid less the other is that solid less the
//! intersection.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

/// What `fuse` says where a crossing it needs as a conic is not one.
const NOT_A_CONIC: &str =
    "geometry: these two surfaces do not cross in one closed curve lying in a plane";

/// What `fuse` says where `intersect` cannot name how a cone crosses a second
/// curved surface.
const A_CONE_ACROSS_A_CURVED_SURFACE: &str =
    "geometry: a cone crossing another curved surface is a degree-four space curve";

/// What `fuse` says where it cannot lay a face's wire flat in the face's own
/// two coordinates to mesh it.
const A_WIRE_THAT_WILL_NOT_LIE_FLAT: &str =
    "a wire of this face will not lie flat in the face's own two coordinates";

/// **A funnel with a leaning spout.** The funnel is a frustum of slope a half
/// with its apex at the origin, cut two and eight up, so of radius one and
/// four. The spout is a frustum of slope an eighth cut two and twelve up, so of
/// radius a quarter and one and a half, leaned 0.8 radians about the `x` axis
/// through the point four up the funnel's axis.
///
/// Every fuse is refused where it asks for a crossing as a conic: the walk
/// marches both loops, and the fuse cannot yet tell which way round one runs.
#[test]
fn a_funnel_fused_with_a_spout_leaning_across_it_is_refused_by_every_rule() {
    assert_eq!(
        every_fuse_of(funnel_and_spout).refusals(),
        [[Some(NOT_A_CONIC); 2]; 3]
    );
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
/// shows the touch on a pair whose numbers are exact.
///
/// Every fuse is refused where `intersect` names the crossing.
#[test]
fn two_pointed_cones_one_leaning_off_the_other_are_refused_by_every_rule() {
    assert_eq!(
        every_fuse_of(two_pointed_cones).refusals(),
        [[Some(A_CONE_ACROSS_A_CURVED_SURFACE); 2]; 3]
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
/// The intersection answers both ways round, and so does the leaning cone less
/// the upright one. The union both ways round and the upright cone less the
/// leaning one are refused where a wire will not lie flat in its face.
#[test]
fn two_pointed_cones_of_two_slopes_crossing_in_a_loop_named_by_two_flats_fuse_or_refuse_by_name() {
    assert_eq!(
        every_fuse_of(pointed_cones_of_two_slopes).refusals(),
        [
            [Some(A_WIRE_THAT_WILL_NOT_LIE_FLAT); 2],
            [None, None],
            [Some(A_WIRE_THAT_WILL_NOT_LIE_FLAT), None],
        ]
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
/// as the others that came back say it must.
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
