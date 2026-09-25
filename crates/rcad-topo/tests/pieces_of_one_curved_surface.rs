//! A fuse answer that carries a curved surface as several faces, taken back in
//! by the next fuse.
//!
//! §223.4: `block ∪ frustum` leaves the frustum's cone as several faces, and the
//! fuse could not take that answer back in. Two gaps stand in the way, one
//! behind the other (§223.2):
//!
//! 1. **The arc's picker.** The first fuse lowered the picker of the block
//!    floor's arc just far enough for the corners *as it stored them* to read on
//!    or in front. The second fuse re-solves each corner into a box, the
//!    picker's plane ran through both boxes, and the second union refused with
//!    [`rcad_topo::Error::AFaceTrimmedByACurveThisCannotFollow`]. The picker now
//!    clears each corner grown by the room a re-solved corner is allowed.
//! 2. **The cone pieces are never joined.** Behind the picker, the second union
//!    refuses with [`rcad_topo::Error::AFaceMeetsItsNeighbourOnItsOwnSurface`]:
//!    nothing folds pieces of one curved surface into one face the way pieces of
//!    one plane are folded. That one is still open.
//!
//! # The risk the picker's room carries
//!
//! Lowering the picker further can take its plane past the centre of the circle
//! the arc is on. An arc is then no longer named by "both ends in front, centre
//! on or behind", and the next fuse refuses it. That happens for an arc within a
//! few `1e-14` of a semicircle, and it is built here: a block whose wall passes
//! `t` from the frustum's axis, for `t` down to `2⁻⁴⁸`.

mod cones;

use rcad_geom::{EdgeRun, Isometry, Picker, Scale, Vector};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

use cones::a_frustum;

/// `(block ∪ frustum) ∪ cube`, with a cube that touches nothing, gets past the
/// block floor's arc and reaches the unjoined cone pieces.
///
/// §222.3 items 4 and 5's first refusal. Before the picker cleared the
/// re-solved corners this refused with
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`].
#[test]
fn a_block_and_frustum_united_and_taken_back_in_reach_the_unjoined_cone_pieces() {
    let mut model = a_model();
    let frustum = a_frustum(&mut model, 0.5, 4.0, 9.0);
    let block = a_box(
        &mut model,
        [[0.0, 0.0], [4.5, 0.0], [4.5, 4.5], [0.0, 4.5]],
        9.0,
        [1.0, 1.0, 5.0],
    );
    let (model, united) = fused(&model, &[block], &[frustum], Rule::Union)
        .expect("a block and a frustum through its floor unite");

    let refusal = the_union_again_with_a_far_cube(&model, &united)
        .expect_err("the cone pieces are not joined yet");
    assert!(
        matches!(refusal, Error::AFaceMeetsItsNeighbourOnItsOwnSurface { .. }),
        "the second union refused before reaching the cone pieces: {refusal:?}"
    );
}

/// An arc of the block's floor that is nearly a semicircle keeps the circle's
/// centre behind its picker, however near a semicircle it is — down to `2⁻⁴⁸`
/// of the circle's radius 2.5, past which the first union no longer answers
/// for its own reasons. `t = 2⁻²` is left out because its wall meets the circle
/// at `(0, −2.5, 5)`, on the frustum's seam, and that refuses on its own.
///
/// **This is the risk the picker's room carried, and it was real.** Lowering
/// the picker against every corner grown by that room, with nothing else
/// changed, turned the centre to in front of it from `t = 2⁻⁴⁴` down: the room
/// is `64 ε` times the corner's size, about `7e-14` here, and the centre sits
/// `t` behind the chord. The same change also refused ten tests of half balls
/// whose arcs are exact semicircles. So the room is granted only while the
/// centre stays on or behind, and otherwise the picker is what the stored ends
/// alone earn, as before.
#[test]
fn a_near_semicircle_keeps_its_centre_behind_its_picker() {
    for exponent in [1, 10, 20, 30, 40, 42, 44, 45, 46, 47, 48] {
        let t = 2.0_f64.powi(-exponent);
        let (model, united) = a_frustum_united_with_a_wall_near_its_axis(t)
            .unwrap_or_else(|why| panic!("t = 2^-{exponent}: the first union refused: {why:?}"));
        let the_floors_arc = the_picker_facing_into_the_block(&model, &united, t);
        assert_eq!(
            the_floors_arc.side_of_point(PredicatePoint::from([0.0, 0.0, 5.0])),
            Decision::Neg,
            "t = 2^-{exponent}: the circle's centre is not behind the floor arc's picker"
        );
    }
}

/// With the centre kept behind, the arcs the room can be granted to get past
/// the picker on the way back in, and reach the cone pieces.
///
/// `t = 2⁻³⁰` and `2⁻⁴⁰` refused with
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] before the room. `2⁻⁴⁴` and
/// `2⁻⁴⁵` still do: there the room would take the plane past the centre, so it
/// is not granted, and the re-solved corners straddle the picker as before.
#[test]
fn a_frustum_and_a_wall_near_its_axis_taken_back_in_reach_the_unjoined_cone_pieces() {
    for exponent in [1, 10, 30, 40, 42] {
        let t = 2.0_f64.powi(-exponent);
        let (model, united) =
            a_frustum_united_with_a_wall_near_its_axis(t).expect("the first union answers");
        let refusal = the_union_again_with_a_far_cube(&model, &united)
            .expect_err("the cone pieces are not joined yet");
        assert!(
            matches!(refusal, Error::AFaceMeetsItsNeighbourOnItsOwnSurface { .. }),
            "t = 2^-{exponent}: the second union refused before reaching the cone pieces: \
             {refusal:?}"
        );
    }
}

/// A frustum about `z` and a block over `z ∈ [5, 8]` whose wall leans across
/// the axis and passes `t` from it at the floor, so the block's floor keeps an
/// arc of the cone's radius-2.5 circle that is `t` short of a semicircle.
///
/// The wall leans so its corners are not on the grid: a wall square to an axis
/// puts both ends of the arc exactly on the chord's own plane, and a picker
/// there is never lowered at all.
fn a_frustum_united_with_a_wall_near_its_axis(t: f64) -> Result<(Model, Vec<SolidKey>), Error> {
    let mut model = a_model();
    let frustum = a_frustum(&mut model, 0.5, 4.0, 9.0);
    let block = a_box(
        &mut model,
        [[t - 0.5, -5.0], [5.0, -5.0], [5.0, 5.0], [t + 0.5, 5.0]],
        3.0,
        [0.0, 0.0, 5.0],
    );
    fused(&model, &[block], &[frustum], Rule::Union)
}

/// The picker of the arc the block's floor keeps: the one at `z = 5` facing
/// into the block, towards `+x`.
fn the_picker_facing_into_the_block(
    model: &Model,
    solids: &[SolidKey],
    t: f64,
) -> rcad_geom::HalfSpace {
    let mut found = Vec::new();
    for &solid in solids {
        for face in model.solid(solid).expect("this model made the key").faces() {
            for wire in face.wires() {
                for coedge in wire.coedges() {
                    let edge = coedge.edge();
                    let on_the_floor = [edge.start().point(), edge.end().point()]
                        .iter()
                        .all(|end| end.coordinates()[2] == 5.0);
                    let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = edge.run() else {
                        continue;
                    };
                    for picker in &picked_by {
                        if let Picker::InFrontOf(half) = picker
                            && on_the_floor
                            && half.normal().x() > 0.0
                            && !found.iter().any(|(seen, _)| *seen == edge.key())
                        {
                            found.push((edge.key(), *half));
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        found.len(),
        1,
        "t = {t:e}: one arc on the floor faces into the block"
    );
    found[0].1
}

fn the_union_again_with_a_far_cube(model: &Model, united: &[SolidKey]) -> Result<(), Error> {
    let mut model = model.clone();
    let far = a_box(
        &mut model,
        [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
        2.0,
        [20.0, 20.0, 20.0],
    );
    fused(&model, united, &[far], Rule::Union).map(|_| ())
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

/// A box: `outline` in the `xy` plane, `tall` high, moved by `at`.
fn a_box(model: &mut Model, outline: [[f64; 2]; 4], tall: f64, at: [f64; 3]) -> SolidKey {
    let outline = ProfileBuilder::new(outline.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("four corners that turn one way");
    model
        .add_extrusion(
            &outline,
            tall,
            &Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite shift"),
        )
        .expect("a box")
}

/// Fuse a copy of `model`. A certified answer that recorded a decision it could
/// not prove is a failure.
fn fused(
    model: &Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Result<(Model, Vec<SolidKey>), Error> {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    let answer = model.fuse(arguments, tools, rule, &Policy::certified(), &mut log)?;
    assert!(
        log.is_empty(),
        "a certified {rule:?} recorded {} decisions it could not prove",
        log.len()
    );
    let solids = answer.solids().to_vec();
    Ok((model, solids))
}
