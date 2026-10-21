//! A ball of 10 across a round bar of 2, reaching past both its caps (row
//! Q2570, found by Q2557).
//!
//! # The shape
//!
//! The bar of `a_fold_between_two_curves_closer_than_any_two_corners.rs`:
//! radius 2, up `z` from −1 to 11, turned a quarter round `z` so that its seam
//! runs up `(0, 2, z)`. A ball of 10 centred `8.5` off the bar's axis at
//! `z = 5`, turned some way round it. The ball and the pipe cross in one closed
//! loop, which runs past the caps on the side facing the ball, at `z = −2.6`
//! and `12.6`, and comes back inside the bar on either side of it. Each cap's
//! plane cuts the loop at two corners, so inside the bar the loop is two arcs,
//! each from a corner on the top cap down to the corner below it on the bottom
//! cap.
//!
//! # Why it was refused
//!
//! A fuse names each of those arcs by the ball, the pipe and one half-space it
//! lies in, whose plane runs through both its corners and the bar's axis
//! (Q2557). Walking the arc, `the_arc_between` in `rcad-geom`'s `walk.rs`
//! marches the whole loop and keeps whichever of the two arcs between the
//! corners that half-space holds. Until Q2570 it asked at the middle marched
//! point of each arc only. The long arc, round past both caps, has its middle
//! on the far side of the bar, at `(−1.44, −1.39, 5.36)` for one of them, in
//! front of that plane, though the arc passes `(0, 2, −2.6)` behind it. So both
//! arcs were held, and every fuse of the pair was refused
//! `CannotNameWhichPieceOfACrossingThisIs`.
//!
//! An arc is now ruled out when any point of it is decided behind one of its
//! half-spaces, and each fuse here answers. The answer was there, and the walk
//! did not reach it: a gap, in §43.6's words, and not a limit.
//!
//! The fuses run certified, in a model of 100 mm.

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use std::f64::consts::PI;

/// How far the ball's centre is off the bar's axis.
const OFF_THE_AXIS: f64 = 8.5;

/// The union, either solid first, the bar less the ball and the ball less the
/// bar, with the ball on the bar's seam side, are one solid of four faces
/// each, which validates and measures what [`shared`] says it should.
#[test]
fn the_union_and_both_differences_of_a_ball_reaching_past_both_caps_of_a_bar_answer() {
    let bar = 48.0 * PI;
    let ball = 4000.0 * PI / 3.0;
    let shared = shared(100_000);
    for (rule, bar_first, volume) in [
        (Rule::Union, true, bar + ball - shared),
        (Rule::Union, false, bar + ball - shared),
        (Rule::Difference, true, bar - shared),
        (Rule::Difference, false, ball - shared),
    ] {
        let mut model = a_model();
        let said = fuse(&mut model, 0.0, rule, bar_first);
        answers(
            &mut model,
            said,
            volume,
            &format!("{rule:?}, bar first: {bar_first}"),
        );
    }
}

/// What the ball shares with the bar at six turns round it, either solid
/// first, is one solid of four faces, a piece of the pipe, a strip of the
/// ball and a lens of each cap, and it validates and measures [`shared`].
#[test]
fn what_a_ball_reaching_past_both_caps_of_a_bar_shares_with_it_answers_at_six_turns() {
    let shared = shared(100_000);
    for around in [0.0, 0.9, 2.2, PI, 4.0, 5.3] {
        for bar_first in [true, false] {
            let mut model = a_model();
            let said = fuse(&mut model, around, Rule::Intersection, bar_first);
            answers(
                &mut model,
                said,
                shared,
                &format!("the intersection {around} round the bar, bar first: {bar_first}"),
            );
        }
    }
}

/// [`shared`] at the 100 000 steps the fuses are held to is within a
/// ten-thousandth of the rule at 1000, and the mesh may miss by more than a
/// whole one.
#[test]
fn what_the_ball_shares_with_the_bar_is_within_a_ten_thousandth_of_a_rule_a_hundred_times_coarser()
{
    let (coarse, fine) = (shared(1000), shared(100_000));
    assert!(
        (coarse - fine).abs() < 1e-4,
        "{coarse} at 1000 steps, {fine} at 100 000"
    );
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// The fuse `rule` of the bar and the ball turned `around` the bar's axis from
/// its seam side, the bar first or not.
fn fuse(
    model: &mut Model,
    around: f64,
    rule: Rule,
    bar_first: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    let seam_up_y = moved_to([0.0, 0.0, -1.0])
        .compose(&Isometry::quarter_turns(&Axis::z(), 1).expect("a turn about z"));
    let bar = model.add_round_bar(2.0, 12.0, &seam_up_y).expect("a bar");
    let ball = model
        .add_ball(
            10.0,
            &moved_to([
                -OFF_THE_AXIS * f64::sin(around),
                OFF_THE_AXIS * f64::cos(around),
                5.0,
            ]),
        )
        .expect("a ball");
    let (arguments, tools) = if bar_first {
        ([bar], [ball])
    } else {
        ([ball], [bar])
    };
    model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}

fn moved_to(at: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite move")
}

/// Assert that `said` is one solid of four faces, that the model validates,
/// and that the solid measures `volume` to within the mesh's own error. `what`
/// names the fuse in every message.
fn answers(model: &mut Model, said: rcad_topo::Result<Vec<SolidKey>>, volume: f64, what: &str) {
    let policy = Policy::certified();
    let solids = said.unwrap_or_else(|why| panic!("{what}: refused: {why:?}"));
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the answer does not validate: {why:?}"));
    let [solid] = solids[..] else {
        panic!("{what}: {} solids, not one", solids.len());
    };
    let view = model.solid(solid).expect("a solid the fuse made");
    assert_eq!(view.faces().count(), 4, "{what}: faces");
    let measure = view
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the answer measures");
    assert!(
        (measure.volume() - volume).abs() <= measure.volume_error(),
        "{what}: measured {} against {volume}, and the mesh may miss by {}",
        measure.volume(),
        measure.volume_error()
    );
}

/// What the ball shares with the bar, by a midpoint rule of `steps` steps up
/// `z` from cap to cap.
///
/// At each height the ball's section is a disc of radius `R = √(100 − (z − 5)²)`,
/// between 8 at the caps and 10 at `z = 5`, whose centre is `d = 8.5` from the
/// bar's, and the bar's is a disc of `r = 2`. Since `d − r < R < d + r` at every
/// height, the two always overlap in a lens, of area
/// `r²·acos((d² + r² − R²)/2dr) + R²·acos((d² + R² − r²)/2dR) − ½√((−d + r + R)(d + r − R)(d − r + R)(d + r + R))`.
fn shared(steps: u32) -> f64 {
    let (d, r) = (OFF_THE_AXIS, 2.0_f64);
    let step = 12.0 / f64::from(steps);
    let mut sum = 0.0;
    for i in 0..steps {
        let z = -1.0 + step * (f64::from(i) + 0.5);
        let big = (100.0 - (z - 5.0) * (z - 5.0)).sqrt();
        sum += r * r * ((d * d + r * r - big * big) / (2.0 * d * r)).acos()
            + big * big * ((d * d + big * big - r * r) / (2.0 * d * big)).acos()
            - 0.5 * ((-d + r + big) * (d + r - big) * (d - r + big) * (d + r + big)).sqrt();
    }
    sum * step
}
