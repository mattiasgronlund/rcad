//! **A cone against a pipe or a ball off its axis**, queue rows Q2666 and
//! Q2788.
//!
//! The two cross in a curve of degree four, and `intersect` used to refuse
//! every such pair `ACrossingBetweenAConeAndAnotherCurvedSurface`, so every
//! fuse of the three shapes here refused before any run was walked:
//!
//! - a tapered shaft with a pin pushed through it square to its axis, which
//!   crosses the shaft's wall in two loops, one each side;
//! - a pointed cone with a bar leaning less than the cone's half-angle, which
//!   crosses the cone's upper nappe in one loop and the nappe below the apex,
//!   which no face here holds, in another;
//! - a pointed cone with a ball beside its axis poking out through its wall,
//!   which crosses it in one loop.
//!
//! Since row Q2788 `intersect` names those loops and `walk` marches them, and
//! the fuses get further. **What each fuse gives now is pinned here**, under
//! every rule both ways round, for the row after Q2788 to turn round: the shaft
//! and the pin are not a solid by any rule, the leaning bar is refused where a
//! crossing is asked to be a conic, and the ball answers three of its six
//! fuses. Each answer that comes back is certified, sound, and measures as the
//! others that came back say it must: the union and the intersection together
//! hold each solid once, and each solid less the other is that solid less the
//! intersection.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

/// What `fuse` says where an edge of what it built has one face along it.
const NOT_A_SOLID: &str = "the result would not be a solid: an edge would have 1 faces along it";

/// What `fuse` says where a crossing it needs as a conic is not one.
const NOT_A_CONIC: &str =
    "geometry: these two surfaces do not cross in one closed curve lying in a plane";

/// What `fuse` says where a face's wire will not lie flat in its coordinates.
const NOT_FLAT: &str = "a wire of this face will not lie flat in the face's own two coordinates";

/// **A tapered shaft with a cross pin.** The shaft is a frustum of slope a
/// quarter, of radius one at its narrow end and four at its wide end. The pin
/// has radius a half and runs along `y` at height ten, where the shaft has
/// radius two and a half, so it pokes out six minus that each side.
///
/// Every fuse builds an edge with one face along it.
#[test]
fn a_tapered_shaft_fused_with_a_pin_through_it_is_not_a_solid_by_any_rule() {
    assert_eq!(
        every_fuse_of(shaft_and_pin).refusals(),
        [[Some(NOT_A_SOLID); 2]; 3]
    );
}

/// **A pointed cone with a bar leaning through it.** The cone has slope a
/// half, so its half-angle is about 0.46 radians, and the bar leans 0.3
/// radians about the `x` axis through the point four up the cone's axis.
///
/// Every fuse is refused where it asks for a crossing as a conic.
#[test]
fn a_pointed_cone_fused_with_a_bar_leaning_through_it_is_refused_by_every_rule() {
    assert_eq!(
        every_fuse_of(cone_and_leaning_bar).refusals(),
        [[Some(NOT_A_CONIC); 2]; 3]
    );
}

/// **A pointed cone with a ball poking out of its side.** The ball has radius
/// two and sits one out from the axis at height five, where the cone has radius
/// two and a half.
///
/// The intersection answers both ways round, and so does the ball less the
/// cone. The union both ways and the cone less the ball build a face whose wire
/// will not lie flat.
#[test]
fn a_pointed_cone_fused_with_a_ball_poking_out_of_its_side_answers_three_rules_of_six() {
    assert_eq!(
        every_fuse_of(cone_and_ball).refusals(),
        [
            [Some(NOT_FLAT), Some(NOT_FLAT)],
            [None, None],
            [Some(NOT_FLAT), None]
        ]
    );
}

fn shaft_and_pin(model: &mut Model) -> (SolidKey, SolidKey) {
    let shaft = a_frustum(model, 0.25, 4.0, 16.0);
    let along_y = Isometry::translation(Vector::new(0.0, -6.0, 10.0))
        .expect("a shift")
        .compose(&Isometry::quarter_turns(&Axis::x(), -1).expect("a quarter turn"));
    let pin = model
        .add_round_bar(0.5, 12.0, &along_y)
        .expect("a bar of radius a half");
    (shaft, pin)
}

fn cone_and_leaning_bar(model: &mut Model) -> (SolidKey, SolidKey) {
    let cone = a_pointed_cone(model, 0.5, 8.0);
    let through =
        Axis::new(Point::new(0.0, 0.0, 4.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
    let placement = Isometry::rotation(&through, 0.3)
        .expect("a turn")
        .compose(&Isometry::translation(Vector::new(0.0, 0.0, -2.0)).expect("a shift"));
    let bar = model
        .add_round_bar(1.0, 12.0, &placement)
        .expect("a bar of radius one");
    (cone, bar)
}

fn cone_and_ball(model: &mut Model) -> (SolidKey, SolidKey) {
    let cone = a_pointed_cone(model, 0.5, 8.0);
    let ball = model
        .add_ball(
            2.0,
            &Isometry::translation(Vector::new(1.0, 0.0, 5.0)).expect("a shift"),
        )
        .expect("a ball of radius two");
    (cone, ball)
}

/// What [`every_fuse_of`] got: for the union, the intersection and the
/// difference, in that order, cone first and then the other first, the volume
/// and the error `measure` publishes with it, or the words of the refusal.
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
    let (cone, other) = shape(&mut model);
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
                    "{rule:?}, cone first {}: the answer is sound: {why}",
                    argument == cone
                )
            });
        Ok(what_they_measure(&answer, fused.solids()))
    };
    let both_ways = |rule: Rule| [fuse(rule, cone, other), fuse(rule, other, cone)];
    let answers = [
        both_ways(Rule::Union),
        both_ways(Rule::Intersection),
        both_ways(Rule::Difference),
    ];

    let [unions, intersections, differences] = &answers;
    let both_alone = [alone(cone), alone(other)];
    let together = (
        both_alone[0].0 + both_alone[1].0,
        both_alone[0].1 + both_alone[1].1,
    );
    for (way, order) in ["cone first", "the other first"].into_iter().enumerate() {
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
            .expect("a cone, a pipe and a ball measure");
        (volume + measured.volume(), error + measured.volume_error())
    })
}
