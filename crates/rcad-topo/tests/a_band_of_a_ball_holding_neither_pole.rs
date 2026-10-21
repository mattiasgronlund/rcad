//! A ball of 3 at the origin cut by a slab whose two faces both cross it off
//! its poles, and the band of the ball's skin left between them (row Q2556).
//!
//! # The shape
//!
//! The slab is a block 10 on a side but between two planes square to one axis,
//! `x` in `[−2, −1.7]`, `x` in `[0.5, 2]` or `y` in `[1, 2]`. Both planes cut the
//! ball in a circle that neither pole lies on, and the ball's skin between the
//! two circles is a band that holds neither pole. In the ball's own two
//! coordinates, longitude and latitude, the nearer circle is a loop round a
//! piece of the chart that reaches no pole, and the further circle is a hole
//! inside it.
//!
//! What the ball shares with the slab is that band and the two discs. The slab
//! less the ball is a plate with a round hole through it whose wall is the same
//! band, facing the other way.
//!
//! # Why it was refused
//!
//! Both fuses were refused `CurvedFaceWithAHoleGoesTooFarRound`, either solid
//! first, from the last return of `cut_off_what_goes_too_far_round` in
//! `mesh.rs`, where `cut_a_hole_up_to_both_poles` found no pole side for its
//! cuts to reach. `cut_a_hole_out_to_its_outline` is the cut for this shape, a
//! leg from the hole's highest corner up to the outline and one from its lowest
//! down to it, and it was asked first; but it took only a pipe, or a cone clear
//! of its apex, and said no to every ball.
//!
//! # A gap and not a limit
//!
//! The band is there and has a mesh. Away from the poles a ball's chart is as
//! regular as a pipe's: a piece of it lies between the least and the greatest
//! latitude of its corners, and so do the straight legs between them, so when
//! no corner sits on a pole neither does anything the cut draws. Let in on that
//! reading, the cut answers every one of these fuses, each validates, and each
//! measures the slab of the ball, `π(9s − s³/3)` between the two planes.
//!
//! # What answered already
//!
//! A slab across the ball's axis, `z` in `[1, 2]`, leaves a band whose two
//! circles go right round, which is cut across as a ring; and the slab `x` in
//! `[−1, 1]` leaves one whose outline runs through both poles, which is cut
//! from the hole up to each pole. Both answered before and still do.
//!
//! The fuses run certified, in a model of 100 mm.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

use std::f64::consts::PI;

/// The slabs, each as the axis it is square to and its two planes along it,
/// that leave a band of the ball's skin holding neither pole.
const SLABS_LEAVING_A_BAND_HOLDING_NEITHER_POLE: [(usize, f64, f64); 3] =
    [(0, -2.0, -1.7), (0, 0.5, 2.0), (1, 1.0, 2.0)];

/// The slab across the ball's axis and the one through both its poles, which
/// answered before row Q2556.
const SLABS_ACROSS_THE_AXIS_OR_THROUGH_BOTH_POLES: [(usize, f64, f64); 2] =
    [(2, 1.0, 2.0), (0, -1.0, 1.0)];

/// What the ball shares with each slab leaving a band that holds neither pole
/// is one solid of three faces, the band and two discs, either solid first.
#[test]
fn what_a_ball_shares_with_a_slab_leaving_a_band_that_holds_neither_pole_answers() {
    for (axis, low, high) in SLABS_LEAVING_A_BAND_HOLDING_NEITHER_POLE {
        for ball_first in [true, false] {
            let mut model = a_model();
            let said = fuse(
                &mut model,
                (axis, low, high),
                Rule::Intersection,
                ball_first,
            );
            answers(
                &mut model,
                said,
                the_ball_between(low, high),
                &[3],
                &format!(
                    "the ball and the slab [{low}, {high}] along axis {axis}, ball first: {ball_first}"
                ),
            );
        }
    }
}

/// The slab less the ball, for each slab leaving a band that holds neither
/// pole, is one plate of seven faces: its four sides, its two faces each with
/// a round hole, and the band as the hole's wall.
#[test]
fn a_slab_less_a_ball_leaving_a_band_that_holds_neither_pole_answers() {
    for (axis, low, high) in SLABS_LEAVING_A_BAND_HOLDING_NEITHER_POLE {
        let mut model = a_model();
        let said = fuse(&mut model, (axis, low, high), Rule::Difference, false);
        answers(
            &mut model,
            said,
            100.0 * (high - low) - the_ball_between(low, high),
            &[7],
            &format!("the slab [{low}, {high}] along axis {axis} less the ball"),
        );
    }
}

/// The slab across the ball's axis and the one through both its poles
/// answered before, and still do: what the ball shares with each, either solid
/// first, and each less the ball.
#[test]
fn a_slab_across_the_balls_axis_or_through_both_its_poles_still_answers() {
    for (axis, low, high) in SLABS_ACROSS_THE_AXIS_OR_THROUGH_BOTH_POLES {
        let shared = the_ball_between(low, high);
        for (rule, ball_first, volume) in [
            (Rule::Intersection, true, shared),
            (Rule::Intersection, false, shared),
            (Rule::Difference, false, 100.0 * (high - low) - shared),
        ] {
            let mut model = a_model();
            let said = fuse(&mut model, (axis, low, high), rule, ball_first);
            let what = format!(
                "{rule:?} of the ball and the slab [{low}, {high}] along axis {axis}, \
                 ball first: {ball_first}"
            );
            let solids = said.unwrap_or_else(|why| panic!("{what}: refused: {why:?}"));
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: the answer does not validate: {why:?}"));
            measures(&model, &solids, volume, &what);
        }
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// The fuse `rule` of the ball of 3 at the origin and the block 10 on a side
/// but between `low` and `high` along `axis`, the ball first or not.
fn fuse(
    model: &mut Model,
    (axis, low, high): (usize, f64, f64),
    rule: Rule,
    ball_first: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    let (mut from, mut to) = ([-5.0; 3], [5.0; 3]);
    from[axis] = low;
    to[axis] = high;
    let ball = model.add_ball(3.0, &Isometry::identity()).expect("a ball");
    let block = a_block(model, from, to);
    let (arguments, tools) = if ball_first {
        ([ball], [block])
    } else {
        ([block], [ball])
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

fn a_block(model: &mut Model, from: [f64; 3], to: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [to[0] - from[0], 0.0],
        [to[0] - from[0], to[1] - from[1]],
        [0.0, to[1] - from[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle is a profile");
    model
        .add_extrusion(
            &rectangle,
            to[2] - from[2],
            &Isometry::translation(Vector::new(from[0], from[1], from[2])).expect("a finite move"),
        )
        .expect("a rectangle extrudes")
}

/// How much of the ball of 3 at the origin lies between two planes square to
/// one axis, at `low` and `high` along it: `π(9s − s³/3)` from the one to the
/// other.
fn the_ball_between(low: f64, high: f64) -> f64 {
    let up_to = |s: f64| PI * (9.0 * s - s * s * s / 3.0);
    up_to(high) - up_to(low)
}

/// Assert that `said` is one solid for each of `faces`, each with that many
/// faces, that the model validates, and that together they measure `volume`
/// to within the mesh's own error. `what` names the fuse in every message.
fn answers(
    model: &mut Model,
    said: rcad_topo::Result<Vec<SolidKey>>,
    volume: f64,
    faces: &[usize],
    what: &str,
) {
    let solids = said.unwrap_or_else(|why| panic!("{what}: refused: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the answer does not validate: {why:?}"));
    let counted: Vec<usize> = solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("a solid the fuse made")
                .faces()
                .count()
        })
        .collect();
    assert_eq!(counted, faces, "{what}: faces of each solid");
    measures(model, &solids, volume, what);
}

/// Assert that `solids` together measure `volume` to within the mesh's own
/// error, at a deflection of a hundredth.
fn measures(model: &Model, solids: &[SolidKey], volume: f64, what: &str) {
    let (mut measured, mut error) = (0.0, 0.0);
    for &solid in solids {
        let measure = model
            .solid(solid)
            .expect("a solid the fuse made")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: the answer does not measure: {why:?}"));
        measured += measure.volume();
        error += measure.volume_error();
    }
    assert!(
        (measured - volume).abs() <= error,
        "{what}: measured {measured} against {volume}, and the mesh may miss by {error}"
    );
}
