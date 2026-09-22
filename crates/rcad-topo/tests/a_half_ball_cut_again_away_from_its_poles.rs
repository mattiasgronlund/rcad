//! A half ball cut a second time, by a plane that misses its poles.
//!
//! Cutting a ball by a plane through its polar axis leaves a skin whose
//! boundary is one great circle, walked as two edges that meet at the poles:
//! one runs south to north, the other north to south. Both lie on the same
//! closed crossing when a later fuse arranges that face, and the one running
//! against the crossing's direction covers the stretch that **wraps** past the
//! crossing's start. Cutting those edges into arcs by their end indices alone
//! gave both of them the same stretch, and the fuse panicked with "two edges of
//! one face run along the same stretch of one crossing".
//!
//! The second cut does not have to go anywhere near the axis for that: every
//! cut here misses both poles.
//!
//! The union twin is there for the direction. A half ball made by intersection
//! and one made by union, on opposite sides of the same plane, have skins bound
//! by the same great circle and every corner of both sits on the plane, so a
//! rule that read which arc an edge covers off the plane alone would answer the
//! same for both — and one of them would come out inside out.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const RADIUS: f64 = 1.0;

/// How fine a mesh every volume here is read off.
const DEFLECTION: f64 = 0.001;

/// How far under the closed form a measured volume may fall, as a share.
///
/// Every point of an inscribed mesh sits within the deflection `δ` of the
/// sphere, so the shell it leaves out is at most the curved area `A` times `δ`.
/// The volume of a solid is the sum over its faces of a third of each face's
/// area times its distance from the centre: the sphere gives `A·r/3`, a plane
/// through the centre gives nothing, and a plane the solid lies on the centre's
/// side of gives something positive. So every solid here has volume at least
/// `A·r/3`, and the share missing is at most `3δ/r`. For the union the flat
/// block adds volume and no curved area, so the bound is looser still there.
const HOW_FAR_UNDER: f64 = 3.0 * DEFLECTION / RADIUS;

/// Where the second cut lies, measured from the centre.
const CUT_AT: f64 = 0.5;

#[test]
fn a_half_ball_cut_across_its_polar_axis_is_answered() {
    let mut model = a_model();
    let half = half_a_ball(&mut model, Rule::Intersection);
    let below = a_block_below(&mut model, CUT_AT);
    let answer = fused(&mut model, &half, &[below], Rule::Intersection);
    assert_eq!(answer.len(), 1);
    measures(
        volume_of(&model, &answer),
        (a_balls_volume() - a_caps_volume(RADIUS - CUT_AT)) / 2.0,
    );
}

#[test]
fn a_half_ball_cut_square_to_its_flat_face_is_answered() {
    let mut model = a_model();
    let half = half_a_ball(&mut model, Rule::Intersection);
    let before = block(
        &mut model,
        [7.0 * RADIUS + CUT_AT, 14.0 * RADIUS, 14.0 * RADIUS],
        Vector::new(-7.0 * RADIUS, -7.0 * RADIUS, -7.0 * RADIUS),
    );
    let answer = fused(&mut model, &half, &[before], Rule::Intersection);
    assert_eq!(answer.len(), 1);
    measures(
        volume_of(&model, &answer),
        a_balls_volume() / 2.0 - a_caps_volume(RADIUS - CUT_AT),
    );
}

/// The ball joined to the block `0 <= x <= 6r`, `|y|, |z| <= 6r`, then cut by
/// `z <= 0.5` inside a block that covers everything else.
///
/// What is left is the block's part below the cut, `6 × 12 × (6 + 0.5)`, and
/// the ball's part below the cut on the far side of `x = 0` — half of a ball
/// with its cap of height `r − 0.5` taken off. The ball's near half lies inside
/// the block and adds nothing.
#[test]
fn a_ball_joined_to_a_block_down_its_polar_axis_then_cut_across_it_is_answered() {
    let mut model = a_model();
    let joined = half_a_ball(&mut model, Rule::Union);
    let below = a_block_below(&mut model, CUT_AT);
    let answer = fused(&mut model, &joined, &[below], Rule::Intersection);
    assert_eq!(answer.len(), 1);
    let the_blocks_part = 6.0 * 12.0 * (6.0 + CUT_AT) * RADIUS.powi(3);
    let the_balls_part = (a_balls_volume() - a_caps_volume(RADIUS - CUT_AT)) / 2.0;
    measures(volume_of(&model, &answer), the_blocks_part + the_balls_part);
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(25.0 * RADIUS).expect("a positive size"))
}

/// A ball fused by `rule` with the block `0 <= x <= 6r`, `|y|, |z| <= 6r`,
/// whose face `x = 0` holds the polar axis.
fn half_a_ball(model: &mut Model, rule: Rule) -> Vec<SolidKey> {
    let ball = model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball");
    let knife = block(
        model,
        [6.0 * RADIUS, 12.0 * RADIUS, 12.0 * RADIUS],
        Vector::new(0.0, -6.0 * RADIUS, -6.0 * RADIUS),
    );
    fused(model, &[ball], &[knife], rule)
}

/// The block `|x|, |y| <= 7r`, `-7r <= z <= at`: everything the other solids
/// reach, below the plane `z = at`.
fn a_block_below(model: &mut Model, at: f64) -> SolidKey {
    block(
        model,
        [14.0 * RADIUS, 14.0 * RADIUS, 7.0 * RADIUS + at],
        Vector::new(-7.0 * RADIUS, -7.0 * RADIUS, -7.0 * RADIUS),
    )
}

fn a_balls_volume() -> f64 {
    4.0 / 3.0 * PI * RADIUS.powi(3)
}

/// The cap of height `h` off a ball: `π h² (3r − h) / 3`.
fn a_caps_volume(h: f64) -> f64 {
    PI * h * h * (3.0 * RADIUS - h) / 3.0
}

fn block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3], at: Vector) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            along_z,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a box needs no decision")
}

/// Fuse under a certified policy and insist the model it leaves is valid.
fn fused(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Vec<SolidKey> {
    let answer = model
        .fuse(
            arguments,
            tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("this {rule:?} is answered: {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");
    answer
}

fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(DEFLECTION).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("the answer measures")
                .volume()
        })
        .sum()
}

/// Insist a volume is under the closed form, by no more than [`HOW_FAR_UNDER`].
fn measures(found: f64, closed_form: f64) {
    let under = (closed_form - found) / closed_form;
    assert!(
        (0.0..HOW_FAR_UNDER).contains(&under),
        "measured {found}, closed form {closed_form}, {:.4}% under",
        100.0 * under
    );
}
