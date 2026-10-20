//! A ball made by `Model::add_ball`, cut by a block whose face runs through
//! the ball's south pole.
//!
//! The cap that keeps the north pole is the one
//! `a_cap_whose_rim_runs_through_the_other_pole.rs` builds face by face; here
//! the fuse builds it, from the same 63 planes: radii 0.5, 1 and 2, leans 0.3,
//! 0.7 and 1.1, and seven turns. The block's face goes through the pole
//! exactly, because it is placed there by a shift and a turn about an axis
//! through the pole, and both surfaces certify the pole is on them.
//!
//! **On the base every one of these was refused or answered by luck** (§200.2).
//! The circle was given its one corner where the walk round it happens to
//! start. Where that was a rounding off the pole, the circle was cut again
//! straight across and the mesh refused the cap `AWireWillNotLieFlatOnItsFace`;
//! elsewhere the pole sat part way along the one edge and, for 18 of the 63,
//! the mesh refused it `CannotMeshAFace`. Now the circle's one corner is the
//! pole, as `SolidBuilder` writes it, and the edge is stored running the way
//! the mesh walks it there. All 63 caps answer both ways they can be asked for.
//!
//! The sliver on the other side of the plane, which holds neither pole,
//! answers now as well — a cap bounded by a circle whose one corner is a pole,
//! and that holds no pole, was the shape `mesh.rs` could not cut up. What
//! stopped it was a cut with no point of its own, and
//! [`the_sliver_that_holds_no_pole_is_answered_or_refused_by_name`] says how.
//!
//! The answers that keep the block's face whole but for a round hole — the
//! union, either way round, and the block less the ball — keep a face that
//! runs on past the circle, away from the ball. Those are
//! [`a_ball_united_with_a_block_through_its_south_pole_measures_both`] and
//! [`a_ball_taken_out_of_a_block_through_its_south_pole_leaves_a_dent_that_measures_right`].

use core::f64::consts::PI;
use std::collections::BTreeMap;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

const RADII: [f64; 3] = [0.5, 1.0, 2.0];
const LEANS: [f64; 3] = [0.3, 0.7, 1.1];
const TURNS: [f64; 7] = [0.0, 0.4, 1.3, PI / 2.0, 2.5, PI, -2.0];

/// The ball kept where a block leaning `lean` off the floor stands on its
/// south pole: a cap holding the north pole, measured against its closed form.
#[test]
fn a_ball_cut_through_its_south_pole_by_a_block_keeps_a_cap_that_measures_right() {
    for (radius, lean, turn) in every_plane() {
        let what = format!("radius {radius}, leaning {lean}, turned {turn}");
        let (model, answer) = cut(radius, lean, turn, Rule::Intersection, First::TheBall);
        let answer = answer.unwrap_or_else(|why| panic!("{what}: the cap is answered: {why}"));
        measures_under_by_at_most_three_deflections(
            &model,
            &answer,
            radius,
            a_caps_volume(radius, radius + radius * lean.cos()),
            &what,
        );
    }
}

/// The same caps, left behind when a block below the plane is taken out of the
/// ball. The block's face is turned round in the answer, so the plane the mesh
/// walks the rim round is the other way up from the one the fuse cut with.
#[test]
fn a_block_taken_out_of_a_ball_through_its_south_pole_leaves_the_same_cap() {
    for (radius, lean, turn) in every_plane() {
        let what = format!("radius {radius}, leaning {lean}, turned {turn}");
        let (model, answer) = cut(radius, PI + lean, turn, Rule::Difference, First::TheBall);
        let answer = answer.unwrap_or_else(|why| panic!("{what}: the cap is answered: {why}"));
        measures_under_by_at_most_three_deflections(
            &model,
            &answer,
            radius,
            a_caps_volume(radius, radius + radius * lean.cos()),
            &what,
        );
    }
}

/// The sliver between the plane and the south pole, which holds neither pole.
///
/// Every answer is sound and measures its closed form within what the mesh
/// declares; every refusal is one of two, both the mesh's.
///
/// **All 126 answer now**, where 27 of each 63 did before. What the 72
/// refusals turned on was a cut of the mesher's own that carried no point: the
/// sliver's rim has the pole as one of the four corners a coarse walk gives it,
/// so cutting the sliver in two ran from the pole straight to the corner
/// opposite, with nothing strictly inside — and the flat face closing the
/// sliver drew that very pair as its own diagonal. Four triangles on one side is
/// `TwoPatchesOfTheMeshDrewTheSameSide`; where the flat face picked the other
/// diagonal instead, the curved face had no point off the plane of its rim, the
/// solid meshed flat, and §41.6's retry could not rescue a zero its finer meshes
/// were refused for. `mesh.rs`'s `cut_in_two` now always carries a point of its
/// own.
#[test]
fn the_sliver_that_holds_no_pole_is_answered_or_refused_by_name() {
    let mut by_intersection = BTreeMap::new();
    let mut by_difference = BTreeMap::new();
    for (radius, lean, turn) in every_plane() {
        let closed_form = a_caps_volume(radius, radius - radius * lean.cos());
        for (rule, lean_of_the_block, tally) in [
            (Rule::Intersection, PI + lean, &mut by_intersection),
            (Rule::Difference, lean, &mut by_difference),
        ] {
            let what = format!("{rule:?}, radius {radius}, leaning {lean}, turned {turn}");
            let (model, answer) = cut(radius, lean_of_the_block, turn, rule, First::TheBall);
            let outcome = match answer {
                Ok(answer) => {
                    measures_within_what_the_mesh_declares(
                        &model,
                        &answer,
                        radius,
                        closed_form,
                        &what,
                    );
                    "answered"
                }
                Err(Error::Invalid(Invalid::VolumeIsNotPositive { .. })) => "VolumeIsNotPositive",
                Err(Error::TwoPatchesOfTheMeshDrewTheSameSide { .. }) => {
                    "TwoPatchesOfTheMeshDrewTheSameSide"
                }
                Err(other) => panic!("{what}: refused {other:?}"),
            };
            *tally.entry(outcome).or_insert(0) += 1;
        }
    }
    assert_eq!(
        by_intersection,
        BTreeMap::from([("answered", 63)]),
        "the sliver kept by intersection"
    );
    assert_eq!(
        by_difference,
        BTreeMap::from([("answered", 63)]),
        "the sliver kept by difference"
    );
}

/// The ball united with the block, handed over either way round: the block and
/// the part of the ball outside it, which is the sliver where the block stands
/// north of its plane and the cap where it stands south.
///
/// **On the base all but 63 of these 252 were refused**
/// `FacesDoNotAgreeWhichWayTheyFace`, and so was every one of the block less
/// the ball below. The block's face runs on past the circle, away from the
/// ball, so it points the other way from the lid that would close the ball's
/// piece, and the ball's piece read the circle against that lid. Both faces
/// then walked the circle the same way round.
#[test]
fn a_ball_united_with_a_block_through_its_south_pole_measures_both() {
    for (radius, lean, turn) in every_plane() {
        let sliver = a_caps_volume(radius, radius - radius * lean.cos());
        let cap = a_caps_volume(radius, radius + radius * lean.cos());
        for first in [First::TheBall, First::TheBlock] {
            for (lean_of_the_block, outside_the_block) in [(lean, sliver), (PI + lean, cap)] {
                let what = format!(
                    "{first:?} first, radius {radius}, block leaning {lean_of_the_block}, turned {turn}"
                );
                let (model, answer) = cut(radius, lean_of_the_block, turn, Rule::Union, first);
                let answer =
                    answer.unwrap_or_else(|why| panic!("{what}: the union is answered: {why}"));
                measures_the_block_with(
                    &model,
                    &answer,
                    radius,
                    a_blocks_volume(radius) + outside_the_block,
                    outside_the_block,
                    &what,
                );
            }
        }
    }
}

/// The block less the ball: the block with a dent the shape of the cap where it
/// stands north of its plane, and of the sliver where it stands south.
#[test]
fn a_ball_taken_out_of_a_block_through_its_south_pole_leaves_a_dent_that_measures_right() {
    for (radius, lean, turn) in every_plane() {
        let sliver = a_caps_volume(radius, radius - radius * lean.cos());
        let cap = a_caps_volume(radius, radius + radius * lean.cos());
        for (lean_of_the_block, inside_the_block) in [(lean, cap), (PI + lean, sliver)] {
            let what = format!("radius {radius}, block leaning {lean_of_the_block}, turned {turn}");
            let (model, answer) = cut(
                radius,
                lean_of_the_block,
                turn,
                Rule::Difference,
                First::TheBlock,
            );
            let answer = answer
                .unwrap_or_else(|why| panic!("{what}: the block less the ball is answered: {why}"));
            measures_the_block_with(
                &model,
                &answer,
                radius,
                a_blocks_volume(radius) - inside_the_block,
                inside_the_block,
                &what,
            );
        }
    }
}

fn every_plane() -> impl Iterator<Item = (f64, f64, f64)> {
    RADII.into_iter().flat_map(|radius| {
        LEANS
            .into_iter()
            .flat_map(move |lean| TURNS.into_iter().map(move |turn| (radius, lean, turn)))
    })
}

/// A ball of `radius` about the origin fused by `rule` with a block standing on
/// its south pole: the block's bottom face runs through the pole, and the block
/// is turned `lean` about the level axis square to the direction `turn` round
/// from `x`. At `lean` it stands on the north side of the plane leaning `lean`
/// off the floor, and at `π + lean` on the south side of the same plane.
///
/// The block is `6r` square and `3r` tall, and its face is centred on the
/// pole, so the part of the ball on its side, within `2r` of the pole, lies
/// wholly inside it.
fn cut(
    radius: f64,
    lean: f64,
    turn: f64,
    rule: Rule,
    first: First,
) -> (Model, Result<Vec<SolidKey>, Error>) {
    let mut model = Model::new(Scale::millimetres(25.0 * radius).expect("a positive size"));
    let ball = model
        .add_ball(radius, &Isometry::identity())
        .expect("a ball");
    let wide = 3.0 * radius;
    let square = ProfileBuilder::new(vec![
        [-wide, -wide],
        [wide, -wide],
        [wide, wide],
        [-wide, wide],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let leaning = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(turn.sin(), -turn.cos(), 0.0))
            .expect("a level direction"),
        lean,
    )
    .expect("a turn");
    let on_the_pole = Isometry::translation(Vector::new(0.0, 0.0, -radius))
        .expect("a shift")
        .compose(&leaning);
    let block = model
        .add_extrusion(&square, wide, &on_the_pole)
        .expect("a block");
    let (one, other) = match first {
        First::TheBall => (ball, block),
        First::TheBlock => (block, ball),
    };
    let answer = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec());
    if answer.is_ok() {
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a fuse writes a sound model or writes nothing");
    }
    (model, answer)
}

/// Which solid `cut` hands the fuse first.
#[derive(Clone, Copy, Debug)]
enum First {
    TheBall,
    TheBlock,
}

/// `cut`'s block: `6r` by `6r` by `3r`.
fn a_blocks_volume(radius: f64) -> f64 {
    108.0 * radius * radius * radius
}

/// One solid, under `closed_form` by no more than `3δ/r` of it.
///
/// The bound is `a_half_ball_skin_with_a_round_hole.rs`'s: the shell a mesh
/// leaves out is at most the curved area times `δ`, and a cap holding the
/// ball's centre has volume at least that area times `r/3`. Every cap here
/// holds the centre: the plane through the south pole leans less than a
/// quarter turn.
fn measures_under_by_at_most_three_deflections(
    model: &Model,
    answer: &[SolidKey],
    radius: f64,
    closed_form: f64,
    what: &str,
) {
    let deflection = 0.001 * radius;
    let found = volume(model, answer, deflection, what).volume();
    let under = (closed_form - found) / closed_form;
    assert!(
        (0.0..3.0 * deflection / radius).contains(&under),
        "{what}: measured {found}, closed form {closed_form}, {:.4}% under",
        100.0 * under
    );
}

/// One solid, within the volume error its own mesh declares of `closed_form`;
/// handed back is what it declares.
///
/// A sliver does not hold the ball's centre, so the bound above says nothing
/// about it; what the measurement declares does.
fn measures_within_what_the_mesh_declares(
    model: &Model,
    answer: &[SolidKey],
    radius: f64,
    closed_form: f64,
    what: &str,
) -> f64 {
    let measured = volume(model, answer, 0.001 * radius, what);
    assert!(
        (measured.volume() - closed_form).abs() <= measured.volume_error(),
        "{what}: measured {}, closed form {closed_form}, declared {}",
        measured.volume(),
        measured.volume_error()
    );
    measured.volume_error()
}

/// One solid, within what its mesh declares of the block and `part` together,
/// where what it declares is less than `part`: an answer that left the ball's
/// part out, or put it in where it was to be taken out, would not pass.
fn measures_the_block_with(
    model: &Model,
    answer: &[SolidKey],
    radius: f64,
    closed_form: f64,
    part: f64,
    what: &str,
) {
    let declared = measures_within_what_the_mesh_declares(model, answer, radius, closed_form, what);
    assert!(
        declared < part,
        "{what}: the mesh declares {declared}, and cannot tell the answer from the block alone, {part} away"
    );
}

fn volume(
    model: &Model,
    answer: &[SolidKey],
    deflection: f64,
    what: &str,
) -> rcad_topo::Measurement {
    let [solid] = answer else {
        panic!("{what}: one solid, not {}", answer.len());
    };
    model
        .solid(*solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(deflection).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the answer measures: {why:?}"))
}

/// The cap of height `h` off a ball of `radius`: `π h² (3r − h) / 3`.
fn a_caps_volume(radius: f64, h: f64) -> f64 {
    PI * h * h * (3.0 * radius - h) / 3.0
}
