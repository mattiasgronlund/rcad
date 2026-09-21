//! A ball of radius one half sunk into the floor of a block of side five at
//! `(3, 3)`, the pair turned a quarter turn about `x`: the union used to refuse
//! with `AClosedEdgeOfTheAnswerWouldStartAtAPole`.
//!
//! `fuses_that_must_agree` drew it and shrank it to this, seed `ae27b55d`. It is
//! the same fixture the suite builds, by the same code, so it is the same `f64`.

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use pairs::{BallAndBlock, Turn, Turned};

#[test]
fn a_ball_and_block_turned_a_quarter_turn_about_x_unite_and_hold_what_they_should() {
    let (model, keys) = the_pair().build();
    let (model, solids) = united(&model, keys[0], keys[1]);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the union is a sound model");
    let [solid] = solids[..] else {
        panic!(
            "a ball and a block unite into one solid, not {}",
            solids.len()
        )
    };

    // The block, and the half of the ball standing above its floor.
    let exact = 125.0 + 2.0 * core::f64::consts::PI / 3.0;
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.03).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the union measures");
    let out_by = (measured.volume() - exact).abs();
    assert!(
        out_by <= measured.volume_error(),
        "the union holds {}, {out_by} away from {exact}, and the mesh declared only {}",
        measured.volume(),
        measured.volume_error()
    );
}

fn the_pair() -> Turned<BallAndBlock> {
    Turned {
        fixture: BallAndBlock {
            side: 5,
            radius: 1,
            at: [3, 3],
        },
        turn: Turn {
            direction: [1, 0, 0],
            steps: 16,
        },
    }
}

fn united(model: &Model, block: SolidKey, ball: SolidKey) -> (Model, Vec<SolidKey>) {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(&[block], &[ball], Rule::Union, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("the union answers: {why}"));
    assert!(
        log.records().is_empty(),
        "a certified union recorded {} decisions it could not prove",
        log.records().len()
    );
    let solids = fused.solids().to_vec();
    (model, solids)
}
