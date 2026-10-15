//! A box standing inside a frustum, touching nothing.
//!
//! The frustum is `cones::a_frustum` of slope one half from `z = 8` to
//! `z = 16`, so its wall is 4 from the axis at the bottom and 8 at the top. The
//! box is 6 by 6 by 3, from `(−3, −3, 10)` to `(3, 3, 13)`: its upright edges
//! are `√18 ≈ 4.24` from the axis, and the wall is 5 out at the box's floor.
//!
//! **A fuse cuts along surfaces, not faces**, so the box's four walls cross the
//! frustum's wall in eight hyperbola arcs from the bottom rim to the top one,
//! and its floor and roof cross it in the circles at `z = 10` and `z = 13`.
//! That is the grid `a_box_inside_a_bar.rs` cuts a bar's wall into, but no
//! line of it is a ruling: the box's walls do not pass through the cone's axis.
//! While a seam across a band of more than one edge had to be a chain of
//! rulings, the wall came back in 28 pieces where the frustum has one, however
//! the two were fused (queue row Q868). A seam that is a chain of any edges
//! between pieces (`fuse/assemble.rs`, `a_seam_across_the_band`, §224.2)
//! answers all three.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod cones;

/// The frustum's volume, `π/3 · (1/2)² · (16³ − 8³)`.
fn the_frustums_volume() -> f64 {
    PI / 3.0 * 0.25 * (16.0f64.powi(3) - 8.0f64.powi(3))
}

const THE_BOXS_VOLUME: f64 = 6.0 * 6.0 * 3.0;

/// United either way round, the box is swallowed and the answer is the
/// frustum: two caps and **one** wall.
#[test]
fn a_box_inside_a_frustum_united_either_way_round_is_the_frustum_with_one_wall() {
    for frustum_first in [true, false] {
        let asked = format!("the union, the frustum first {frustum_first}");
        let mut model = a_model();
        let [frustum, block] = the_frustum_and_the_box(&mut model);
        let solid = the_one_solid(
            &mut model,
            frustum,
            block,
            Rule::Union,
            frustum_first,
            &asked,
        );
        assert_holds(&model, solid, the_frustums_volume(), &asked);
        assert_eq!(
            flat_and_curved_faces(&model, solid),
            [2, 1],
            "{asked}: how many flat faces and how many walls"
        );
    }
}

/// The frustum less the box is the frustum with a box-shaped hollow in it: one
/// solid of two shells, the two caps and one wall round the outside and the
/// box's six faces round the hollow.
#[test]
fn the_frustum_less_a_box_inside_it_is_the_frustum_with_a_hollow() {
    let asked = "the frustum less the box";
    let mut model = a_model();
    let [frustum, block] = the_frustum_and_the_box(&mut model);
    let solid = the_one_solid(&mut model, frustum, block, Rule::Difference, true, asked);
    assert_holds(
        &model,
        solid,
        the_frustums_volume() - THE_BOXS_VOLUME,
        asked,
    );
    assert_eq!(
        model
            .solid(solid)
            .expect("this model made the key")
            .shells()
            .count(),
        2,
        "{asked}: the outside and the hollow"
    );
    assert_eq!(
        flat_and_curved_faces(&model, solid),
        [8, 1],
        "{asked}: how many flat faces and how many walls"
    );
}

/// The frustum with one wall can be fused again, and the box united with it
/// again or taken out of it again answers as it did the first time.
#[test]
fn the_frustum_with_one_wall_takes_the_same_box_again() {
    for (rule, volume, faces) in [
        (Rule::Union, the_frustums_volume(), [2, 1]),
        (
            Rule::Difference,
            the_frustums_volume() - THE_BOXS_VOLUME,
            [8, 1],
        ),
    ] {
        let asked = format!("the union, then {rule:?} with the box again");
        let mut model = a_model();
        let [frustum, block] = the_frustum_and_the_box(&mut model);
        let united = the_one_solid(&mut model, frustum, block, Rule::Union, true, &asked);
        let again = a_box(&mut model);
        let solid = the_one_solid(&mut model, united, again, rule, true, &asked);
        assert_holds(&model, solid, volume, &asked);
        assert_eq!(
            flat_and_curved_faces(&model, solid),
            faces,
            "{asked}: how many flat faces and how many walls"
        );
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn the_frustum_and_the_box(model: &mut Model) -> [SolidKey; 2] {
    let frustum = cones::a_frustum(model, 0.5, 8.0, 16.0);
    [frustum, a_box(model)]
}

/// The box from `(−3, −3, 10)` to `(3, 3, 13)`.
fn a_box(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![[-3.0, -3.0], [3.0, -3.0], [3.0, 3.0], [-3.0, 3.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(
            &square,
            3.0,
            &Isometry::translation(Vector::new(0.0, 0.0, 10.0)).expect("a finite shift"),
        )
        .expect("a positive height")
}

fn the_one_solid(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    one_first: bool,
    asked: &str,
) -> SolidKey {
    let [first, second] = if one_first {
        [one, other]
    } else {
        [other, one]
    };
    let answer = model
        .fuse(
            &[first],
            &[second],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{asked} is refused: {why:?}"));
    let [solid] = answer.solids()[..] else {
        panic!("{asked} is one solid, not {:?}", answer.solids());
    };
    solid
}

fn assert_holds(model: &Model, solid: SolidKey, volume: f64, asked: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{asked} left a model that does not validate: {why}"));
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{asked} does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{asked} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// How many of the solid's faces are flat, and how many are on the frustum's
/// cone.
fn flat_and_curved_faces(model: &Model, solid: SolidKey) -> [usize; 2] {
    let (flat, curved): (Vec<_>, Vec<_>) = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .partition(|face| face.surface().as_plane().is_some());
    [flat.len(), curved.len()]
}
