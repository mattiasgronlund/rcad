//! §290.4 B's one refused turned bar, pinned now that §321 fixes it.
//!
//! Case 687 of `RCAD_SOAK_SEED=112 RCAD_SOAK_FACTOR=66`: a bar of radius 1
//! through a block of side 7, at `(5, 2)`, turned 48 of 64 steps about
//! `(-2, -1, 0)`. Its union refused `CannotMeshAFace { why:
//! CoordinateOutOfRange { position: 2, why: TooSmall } }` on `566db6f1` and on
//! `199a606e` — a corner the triangulator's scaled coordinates underflowed to
//! something smaller than its own floor, which is exactly what §321's flush
//! now catches and sets to zero before spade ever sees it. Q229 drew this case
//! by hand and found it; this pins the answer §321 gives it.
//!
//! Built the way the property family and Q229's probe build a turned curved
//! pair — [`CurvedPair::build`] then [`Turn::isometry`] — rather than from
//! coordinates copied out of a print, so a change to either keeps drawing the
//! same case this row demonstrates.

mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use pairs::{CurvedPair, Fixture, Turn, Turned};

/// The case §290.4 B drew: case 687 of `RCAD_SOAK_SEED=112 RCAD_SOAK_FACTOR=66`.
fn the_configuration() -> Turned<CurvedPair> {
    Turned {
        fixture: CurvedPair {
            side: 7,
            radius: 1,
            at: [5, 2],
        },
        turn: Turn {
            direction: [-2, -1, 0],
            steps: 48,
        },
    }
}

/// The union answers, is a sound model, agrees in face count with the upright
/// union turned by the same rotation, and agrees in volume within what the two
/// meshes declare between them — the same comparison
/// `a_turned_ball_united_with_a_block_holds_what_the_upright_one_does` makes
/// in `fuses_that_must_agree.rs`.
///
/// A turned fixture's solids are the upright fixture's solids with one more
/// placement composed on, so the upright union, turned by that same
/// `Isometry`, is the answer this union should give, written in the numbers
/// the kernel would have to write it in regardless of which route built it
/// (`docs/decisions.md` §16.10).
#[test]
fn a_turned_bar_through_a_block_unites_and_agrees_with_the_upright_union_turned() {
    let fixture = the_configuration();
    let deflection = Deflection::of(0.05).expect("a length");

    let (turned_model, turned_keys) = fixture.build();
    let (turned, turned_solids) = united(&turned_model, &turned_keys);
    turned
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the turned union is a sound model");

    let (upright_model, upright_keys) = fixture.fixture.built();
    let (mut upright, upright_solids) = united(&upright_model, &upright_keys);
    upright
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the upright union is a sound model");
    let turn = fixture.turn.isometry();
    for &solid in &upright_solids {
        upright
            .place_solid(solid, &turn)
            .expect("this model made the key");
    }
    upright
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the upright union, turned, is still a sound model");

    assert_eq!(
        turned_solids.len(),
        upright_solids.len(),
        "the turned union and the upright union turned come back as a different number of solids"
    );
    assert_eq!(
        faces(&turned, &turned_solids),
        faces(&upright, &upright_solids),
        "the turned union and the upright union turned have different numbers of faces"
    );

    let (turned_volume, turned_error) = total_volume(&turned, &turned_solids, deflection);
    let (upright_volume, upright_error) = total_volume(&upright, &upright_solids, deflection);
    let declared = turned_error + upright_error;
    // §47.4 item 1: a tolerance nothing can exceed has measured nothing.
    assert!(
        declared.is_finite() && declared > 0.0 && declared < turned_volume,
        "the two measurements declared {declared}, and nothing is worth asserting against that"
    );
    let apart = (turned_volume - upright_volume).abs();
    assert!(
        apart <= declared,
        "the turned union holds {turned_volume}, the upright union turned holds \
         {upright_volume}: {apart} apart, past the {declared} the two declare"
    );
}

/// Union without a tally, for a test naming one case rather than drawing a
/// family.
///
/// # Panics
///
/// If the fuse refuses, or records a decision it could not prove.
fn united(model: &Model, keys: &[SolidKey]) -> (Model, Vec<SolidKey>) {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(
            &[keys[0]],
            &[keys[1]],
            Rule::Union,
            &Policy::certified(),
            &mut log,
        )
        .unwrap_or_else(|why| panic!("this union is decided by proof alone: {why}"));
    assert!(
        log.records().is_empty(),
        "a certified union recorded {} decisions it could not prove",
        log.records().len()
    );
    let solids = fused.solids().to_vec();
    (model, solids)
}

/// How many faces the named solids have between them.
fn faces(model: &Model, solids: &[SolidKey]) -> usize {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .faces()
                .count()
        })
        .sum()
}

/// The volume the named solids hold between them, and what the mesh declares
/// it could be out by.
fn total_volume(model: &Model, solids: &[SolidKey], deflection: Deflection) -> (f64, f64) {
    solids.iter().fold((0.0, 0.0), |(volume, error), &solid| {
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(deflection, &Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("the union meshes at a deflection of 0.05: {why}"));
        (volume + measured.volume(), error + measured.volume_error())
    })
}
