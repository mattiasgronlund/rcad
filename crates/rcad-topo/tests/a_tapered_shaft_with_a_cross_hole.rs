//! **A cone against a pipe or a ball off its axis**, queue row Q2666.
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
//! Each is fused here under every rule both ways round, and each answer is
//! certified, sound, and measures as the others say it must: the union and the
//! intersection together hold each solid once, and each solid less the other
//! is that solid less the intersection. The shaft's intersection with the pin
//! is checked against a sum worked out here without the kernel.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

/// **A tapered shaft with a cross pin.** The shaft is a frustum of slope a
/// quarter, of radius one at its narrow end and four at its wide end. The pin
/// has radius a half and runs along `y` at height ten, where the shaft has
/// radius two and a half, so it pokes out six minus that each side.
#[test]
fn a_tapered_shaft_fused_with_a_pin_through_it_answers_every_rule() {
    let fused = every_fuse_of(shaft_and_pin);
    let slope = 0.25;
    let (radius, height) = (0.5, 10.0);
    let worked_out = the_pin_inside_the_shaft(slope, radius, height);
    let (inside, error) = fused.intersection;
    assert!(
        (inside - worked_out).abs() <= error + 1e-6,
        "the pin inside the shaft measures {inside} give or take {error}, \
         and the sum worked out without the kernel is {worked_out}"
    );
}

/// **A pointed cone with a bar leaning through it.** The cone has slope a
/// half, so its half-angle is about 0.46 radians, and the bar leans 0.3
/// radians about the `x` axis through the point four up the cone's axis.
#[test]
fn a_pointed_cone_fused_with_a_bar_leaning_through_it_answers_every_rule() {
    every_fuse_of(cone_and_leaning_bar);
}

/// **A pointed cone with a ball poking out of its side.** The ball has radius
/// two and sits one out from the axis at height five, where the cone has radius
/// two and a half.
#[test]
fn a_pointed_cone_fused_with_a_ball_poking_out_of_its_side_answers_every_rule() {
    every_fuse_of(cone_and_ball);
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

/// What [`every_fuse_of`] measured: the intersection's volume and the error
/// `measure` publishes with it.
struct Fused {
    intersection: (f64, f64),
}

/// Fuses the two solids `shape` builds under every rule both ways round, and
/// checks that each answer is certified, sound, and measures as the others say
/// it must.
fn every_fuse_of(shape: fn(&mut Model) -> (SolidKey, SolidKey)) -> Fused {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let (cone, other) = shape(&mut model);
    let alone = |solid: SolidKey| what_they_measure(&model, &[solid]);
    let fuse = |rule: Rule, argument: SolidKey, tool: SolidKey| {
        let mut answer = model.clone();
        let mut log = ProvenanceLog::new();
        let fused = answer
            .fuse(&[argument], &[tool], rule, &policy, &mut log)
            .unwrap_or_else(|why| panic!("{rule:?}, cone first {}: {why}", argument == cone));
        assert!(log.is_empty(), "a certified fuse logs nothing: {log:?}");
        answer
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| {
                panic!(
                    "{rule:?}, cone first {}: the answer is sound: {why}",
                    argument == cone
                )
            });
        what_they_measure(&answer, fused.solids())
    };

    let (cone_alone, other_alone) = (alone(cone), alone(other));
    let intersection = fuse(Rule::Intersection, cone, other);
    for (argument, tool) in [(cone, other), (other, cone)] {
        let union = fuse(Rule::Union, argument, tool);
        let the_other_way = fuse(Rule::Intersection, argument, tool);
        let less = fuse(Rule::Difference, argument, tool);
        let argument_alone = if argument == cone {
            cone_alone
        } else {
            other_alone
        };
        let order = format!("cone first {}", argument == cone);
        about_the_same(
            the_other_way,
            intersection,
            &format!("{order}: the intersection against the cone's"),
        );
        about_the_same(
            (union.0 + intersection.0, union.1 + intersection.1),
            (cone_alone.0 + other_alone.0, cone_alone.1 + other_alone.1),
            &format!("{order}: the union and the intersection against the two alone"),
        );
        about_the_same(
            less,
            (
                argument_alone.0 - intersection.0,
                argument_alone.1 + intersection.1,
            ),
            &format!("{order}: the difference against the argument less the intersection"),
        );
    }
    assert!(
        intersection.0 > intersection.1,
        "the two overlap, so the intersection holds something: {intersection:?}"
    );
    Fused { intersection }
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

/// The volume of a pin of `radius` along `y` at `height` inside a shaft of
/// `slope` about `z`, summed in polar steps over the pin's cross-section: at
/// `(x, z)` the pin is inside the shaft for a length `2 √(slope² z² − x²)`.
fn the_pin_inside_the_shaft(slope: f64, radius: f64, height: f64) -> f64 {
    let steps = 2000;
    let (out, round) = (
        radius / f64::from(steps),
        std::f64::consts::TAU / f64::from(steps),
    );
    (0..steps)
        .flat_map(|i| (0..steps).map(move |j| (i, j)))
        .map(|(i, j)| {
            let rho = (f64::from(i) + 0.5) * out;
            let phi = (f64::from(j) + 0.5) * round;
            let (x, z) = (rho * phi.cos(), height + rho * phi.sin());
            2.0 * (slope * slope * z * z - x * x).sqrt() * rho * out * round
        })
        .sum()
}
