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
//! The sliver on the other side of the plane, which holds neither pole, is
//! mostly still refused, and by the mesh: a cap bounded by a circle whose one
//! corner is a pole, and that holds no pole, is a shape `mesh.rs` does not cut
//! up yet. Those are pinned by name with their counts.

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
        let (model, answer) = cut(radius, lean, turn, Rule::Intersection);
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
        let (model, answer) = cut(radius, PI + lean, turn, Rule::Difference);
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
            let (model, answer) = cut(radius, lean_of_the_block, turn, rule);
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
        BTreeMap::from([
            ("answered", 27),
            ("TwoPatchesOfTheMeshDrewTheSameSide", 30),
            ("VolumeIsNotPositive", 6),
        ]),
        "the sliver kept by intersection"
    );
    assert_eq!(
        by_difference,
        BTreeMap::from([
            ("answered", 27),
            ("TwoPatchesOfTheMeshDrewTheSameSide", 18),
            ("VolumeIsNotPositive", 18),
        ]),
        "the sliver kept by difference"
    );
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
fn cut(radius: f64, lean: f64, turn: f64, rule: Rule) -> (Model, Result<Vec<SolidKey>, Error>) {
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
    let answer = model
        .fuse(
            &[ball],
            &[block],
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

/// One solid, within the volume error its own mesh declares of `closed_form`.
///
/// A sliver does not hold the ball's centre, so the bound above says nothing
/// about it; what the measurement declares does.
fn measures_within_what_the_mesh_declares(
    model: &Model,
    answer: &[SolidKey],
    radius: f64,
    closed_form: f64,
    what: &str,
) {
    let measured = volume(model, answer, 0.001 * radius, what);
    assert!(
        (measured.volume() - closed_form).abs() <= measured.volume_error(),
        "{what}: measured {}, closed form {closed_form}, declared {}",
        measured.volume(),
        measured.volume_error()
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
