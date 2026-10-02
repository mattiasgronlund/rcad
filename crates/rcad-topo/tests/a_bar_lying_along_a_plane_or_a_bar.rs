//! A bar lying along a wall, in an inside corner, or along another bar: the
//! three touches along a line `fuses_that_touch.rs` draws, one of each written
//! out by hand, under every rule and both ways round.
//!
//! Along the touching line the two surfaces' gradients are parallel, so `∇S ×
//! ∇Q`, which every other crossing runs along, is exactly nothing. Every fuse of
//! these used to refuse `Undecidable` on *which way a crossing runs* (queue row
//! Q649, §39.4). The direction is the pipe's own axis, and `fuse/arrange.rs`'s
//! `AlongATouch` carries it, turns ways out round it, and gives both sides of the
//! line the one side of the partner the face lies on.
//!
//! **The intersection and both differences are answered**, and they are the
//! answers that exist: the insides are apart, so the intersection is no solid at
//! all and each difference is the first solid as it stood. **The union is
//! refused** `ResultIsNotManifold { count: 4 }`. The likely cause is that both
//! solids keep every face along the touch, so the line comes out one edge with
//! four faces round it, but nobody has traced that. Its answer is the two solids
//! as they stand, so that refusal is a gap, and queue row Q371
//! (`fuse/assemble.rs`) traces and fixes it. It is pinned here as it refuses.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// How far under the closed form a bar's meshed volume may be.
///
/// The bars here are radius one and two, and a mesh at [`A_DEFLECTION`] round
/// a circle of radius one is 0.33% short of it. A ceiling on a measurement, not
/// a tolerance anything is decided by.
const HOW_FAR_A_MESH_MAY_BE: f64 = 0.005;

/// How fine the meshes the volumes are read off are.
const A_DEFLECTION: f64 = 0.02;

/// A cube of side four and a bar of radius one standing beside its wall `x = 4`,
/// touching it along `(4, 2, z)`. The bar's seam is at `(6, 2, z)`, across the
/// bar from the touch.
#[test]
fn a_bar_standing_against_a_blocks_wall_answers_every_rule_but_the_union() {
    every_rule_of(
        |model| {
            let cube = a_block(model, [4.0, 4.0, 4.0]);
            let bar = a_bar(model, 1.0, 2.0, [5.0, 2.0, 1.0]);
            [(cube, 64.0), (bar, 2.0 * PI)]
        },
        "a bar against a block's wall",
    );
}

/// The same with the bar on the other side of the cube, so its seam `(0, 2, z)`
/// is the touching line itself.
#[test]
fn a_bar_whose_seam_is_the_line_it_touches_a_wall_along_answers_every_rule_but_the_union() {
    every_rule_of(
        |model| {
            let cube = a_block(model, [4.0, 4.0, 4.0]);
            let bar = a_bar(model, 1.0, 2.0, [-1.0, 2.0, 1.0]);
            [(cube, 64.0), (bar, 2.0 * PI)]
        },
        "a bar touching a wall along its seam",
    );
}

/// An L of two arms six long and two thick, four high, with a bar of radius one
/// in its inside corner, touching both walls: `x = 2` along `(2, 3, z)` and
/// `y = 2` along `(3, 2, z)`. The fillet a part's inside corner is.
#[test]
fn a_bar_in_an_inside_corner_answers_every_rule_but_the_union() {
    every_rule_of(
        |model| {
            let corner = an_l(model);
            let bar = a_bar(model, 1.0, 2.0, [3.0, 3.0, 1.0]);
            [(corner, 80.0), (bar, 2.0 * PI)]
        },
        "a bar in an inside corner",
    );
}

/// A bar of radius two and one of radius one three apart along `y`, touching
/// along `(0, 2, z)`. Both seams are off the touch, at `(2, 0, z)` and
/// `(1, 3, z)`.
#[test]
fn two_bars_side_by_side_answer_every_rule_but_the_union() {
    every_rule_of(
        |model| {
            let thick = a_bar(model, 2.0, 4.0, [0.0, 0.0, 0.0]);
            let thin = a_bar(model, 1.0, 2.0, [0.0, 3.0, 1.0]);
            [(thick, 16.0 * PI), (thin, 2.0 * PI)]
        },
        "two bars side by side",
    );
}

/// Every rule, both ways round, of a pair whose insides are apart: no solid for
/// the intersection, the first solid for each difference, and the union refused.
fn every_rule_of(build: fn(&mut Model) -> [(SolidKey, f64); 2], pair: &str) {
    for swapped in [false, true] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let [one, other] = build(&mut model);
            let [(first, first_volume), (second, _)] =
                if swapped { [other, one] } else { [one, other] };
            let what = format!("{pair}, {rule:?}, swapped {swapped}");
            let mut log = ProvenanceLog::new();
            let answer = model.fuse(&[first], &[second], rule, &Policy::certified(), &mut log);
            if rule == Rule::Union {
                assert!(
                    matches!(answer, Err(Error::ResultIsNotManifold { count: 4, .. })),
                    "{what}: an edge with four faces round it, got {answer:?}"
                );
                continue;
            }
            let answer = answer.unwrap_or_else(|why| panic!("{what} refused: {why}"));
            assert!(
                log.records().is_empty(),
                "{what}: every decision is proved, got {:?}",
                log.records()
            );
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what} wrote a model that does not validate: {why}"));
            if rule == Rule::Intersection {
                assert!(
                    answer.solids().is_empty(),
                    "{what}: two solids whose insides are apart share no solid"
                );
                continue;
            }
            assert_eq!(answer.solids().len(), 1, "{what}: the first solid, whole");
            let found = volume_of(&model, answer.solids()[0]);
            assert!(
                found <= first_volume
                    && (first_volume - found) / first_volume < HOW_FAR_A_MESH_MAY_BE,
                "{what}: measured {found}, and the first solid is {first_volume}"
            );
        }
    }
}

fn volume_of(model: &Model, solid: SolidKey) -> f64 {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(A_DEFLECTION).expect("a fiftieth of a millimetre is a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures")
        .volume()
}

/// A box with its low corner at the origin.
fn a_block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, along_z, &Isometry::identity())
        .expect("building a box needs no decision")
}

/// An L standing on the floor, four high: two arms two thick and six long,
/// meeting at the origin, with the inside corner at `(2, 2)`. Its area is
/// `36 − 16 = 20`, so its volume is 80.
fn an_l(model: &mut Model) -> SolidKey {
    let outline = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [6.0, 0.0],
        [6.0, 2.0],
        [2.0, 2.0],
        [2.0, 6.0],
        [0.0, 6.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("an L");
    model
        .add_extrusion(&outline, 4.0, &Isometry::identity())
        .expect("building an L needs no decision")
}

/// A bar standing on its foot, its seam a radius along `+x` from its axis.
fn a_bar(model: &mut Model, radius: f64, height: f64, [x, y, z]: [f64; 3]) -> SolidKey {
    model
        .add_round_bar(
            radius,
            height,
            &Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift"),
        )
        .expect("a bar")
}
