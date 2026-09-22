//! A ball of radius one half sunk into the floor of a block of side four at
//! `(2, 2)`, the pair turned a quarter turn about a slanting axis in the
//! `xy` plane: the fuse answered, and measuring the answer refused it as a
//! curved face going right round with no seam, or the fuse's own check
//! refused it as faces that disagree which way they face.
//!
//! `fuses_that_must_agree` drew four of them and shrank each to one of the
//! tests here: seed `7d4e1f9a` about `(−1, −3, 0)`, `a8b648d9` about
//! `(3, −2, 0)`, `9a699edb` about `(3, 1, 0)`, and one about `(1, 3, 0)`
//! whose seed nobody kept. Each is the same fixture the suite
//! builds, by the same code, so it is the same `f64`.
//!
//! Turned, the floor is a plane standing up through the ball's centre, and the
//! ball's poles stay on the world's `z`. So the circle they cross in is a
//! meridian circle, through both poles — except that the plane's normal is a
//! rounded turn, and the exact circle misses each pole by a rounding. The walk
//! round it then passes each pole in a step of about half a turn of longitude,
//! and whether the half ball came back wound right round or not was decided by
//! the last bit of two meridians.

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use pairs::{BallAndBlock, Turn, Turned};

/// Seed `7d4e1f9a`: measuring refused the half ball as a cap going right
/// round with no seam.
#[test]
fn a_ball_and_block_turned_a_quarter_turn_about_minus_one_minus_three_measure_what_they_hold() {
    measure_what_they_hold(turned_about([-1, -3, 0]));
}

/// Seed `a8b648d9`: a fix that put in the pole the walk reads as on the
/// circle turned this one from a named refusal into faces that disagree,
/// because it put in only the south pole and the cap was then cut up to its
/// pole the wrong way round.
#[test]
fn a_ball_and_block_turned_a_quarter_turn_about_three_minus_two_measure_what_they_hold() {
    measure_what_they_hold(turned_about([3, -2, 0]));
}

/// The third of the family the suite drew, about `(1, 3, 0)`.
#[test]
fn a_ball_and_block_turned_a_quarter_turn_about_one_three_measure_what_they_hold() {
    measure_what_they_hold(turned_about([1, 3, 0]));
}

/// Seed `9a699edb`, about `(3, 1, 0)`: drawn by this lane's own hunt on the
/// tree before the fix, the property replaying nothing but what `main` pinned.
#[test]
fn a_ball_and_block_turned_a_quarter_turn_about_three_one_measure_what_they_hold() {
    measure_what_they_hold(turned_about([3, 1, 0]));
}

/// Every rule answers one solid, sound, and it measures what a block and
/// half a ball of radius one hold, to what the mesh declares.
fn measure_what_they_hold(pair: Turned<BallAndBlock>) {
    let side = 4.0_f64;
    let half_a_ball = 2.0 * core::f64::consts::PI / 3.0;
    for (rule, exact) in [
        (Rule::Union, side.powi(3) + half_a_ball),
        (Rule::Intersection, half_a_ball),
        (Rule::Difference, side.powi(3) - half_a_ball),
    ] {
        let (model, keys) = pair.build();
        let (model, solid) = fused_into_one_solid(&model, keys[0], keys[1], rule);
        for how_fine in [0.3, 0.03, 0.01] {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(how_fine).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("the {rule:?} measures at {how_fine}: {why}"));
            let out_by = (measured.volume() - exact).abs();
            assert!(
                out_by <= measured.volume_error(),
                "the {rule:?} holds {} at {how_fine}, {out_by} away from {exact}, and the mesh \
                 declared only {}",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
}

fn turned_about(direction: [i32; 3]) -> Turned<BallAndBlock> {
    Turned {
        fixture: BallAndBlock {
            side: 4,
            radius: 1,
            at: [2, 2],
        },
        turn: Turn {
            direction,
            steps: 16,
        },
    }
}

fn fused_into_one_solid(
    model: &Model,
    block: SolidKey,
    ball: SolidKey,
    rule: Rule,
) -> (Model, SolidKey) {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(&[block], &[ball], rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("the {rule:?} answers: {why}"));
    assert!(
        log.records().is_empty(),
        "a certified {rule:?} recorded {} decisions it could not prove",
        log.records().len()
    );
    let [solid] = fused.solids()[..] else {
        panic!(
            "a ball and a block {rule:?} into one solid, not {}",
            fused.solids().len()
        )
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the {rule:?} is a sound model: {why}"));
    (model, solid)
}
