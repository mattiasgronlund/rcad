//! A flat face whose outline touches a round hole at one point, and that hole
//! touches a second round hole at another.
//!
//! The face's one wire runs round the outline, round the first hole from the
//! point it touches the outline, round the second from the point the two holes
//! touch, and so passes each of those two corners twice. The mesh cuts such a
//! wire into the loops it is made of where it passes a corner twice. It took a
//! cut only where both parts passed each point once, and cut at either corner
//! one part still passes the other corner twice, so it cut nowhere and the
//! triangulator was handed the wire whole and refused it `CornersCoincide`
//! (Q3246, Q3251). It now cuts at each such corner in turn and takes the cuts
//! when every loop left passes each point once.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// A plate `[0, 4] × [0, 4] × [0, 1]` and two bars from `z = −1` to 2, of
/// radius 1 round `(1, 2)`, touching the plate's side `x = 0`, and of radius
/// 0.5 round `(2.5, 2)`, touching the first at `(2, 2)`. The two bars are
/// united first. Each bar adds its two ends past the plate, `2π + 0.5π`.
#[test]
fn a_plate_unites_with_two_touching_bars_the_first_touching_its_side() {
    let mut model = a_model();
    let plate = a_block(&mut model, [0.0, 0.0, 0.0], [4.0, 4.0, 1.0]);
    let big = a_bar(&mut model, 1.0, 3.0, [1.0, 2.0, -1.0]);
    let small = a_bar(&mut model, 0.5, 3.0, [2.5, 2.0, -1.0]);
    let bars = fused(&mut model, &[big], &[small], || "the two bars".to_owned());
    let which = || "the plate with the two bars".to_owned();
    let [whole] = fused(&mut model, &[plate], &bars, which)[..] else {
        panic!("{}: one solid", which());
    };
    has_a_flat_face_passing_two_corners_twice(&model, whole, which);
    valid_and_of_volume(&model, whole, 16.0 + 2.5 * PI, which);
}

/// A bar of radius 2 up `z` from `(0, 0, −2)`, 4 long, with the block `[0, 5]
/// × [−1.5, 1.5] × [−2, 2]` united to it first, and two bars of radius 1 up
/// `z` from `(−1, 0, −3)` and `(1, 0, −3)`, 6 long, united with each other
/// first: the bar at `x = −1` touches the thick bar's rim and the other thin
/// bar, so each end of the answer is such a face.
///
/// The answer is the thick bar and the block, `4` tall: the disk of radius 2
/// and the rectangle `5 × 3`, less the part of the disk with `x ≥ 0` and `|y|
/// ≤ 1.5`, whose area is `1.5 √1.75 + 4 asin 0.75`. Each thin bar adds its two
/// ends of length 1 past `z = ±2`.
#[test]
fn a_block_on_a_thick_bar_unites_with_two_touching_thin_bars_united_first() {
    let mut model = a_model();
    let thick = a_bar(&mut model, 2.0, 4.0, [0.0, 0.0, -2.0]);
    let block = a_block(&mut model, [0.0, -1.5, -2.0], [5.0, 1.5, 2.0]);
    let left = a_bar(&mut model, 1.0, 6.0, [-1.0, 0.0, -3.0]);
    let right = a_bar(&mut model, 1.0, 6.0, [1.0, 0.0, -3.0]);
    let with_the_block = fused(&mut model, &[thick], &[block], || "the block".to_owned());
    let thin = fused(&mut model, &[left], &[right], || "the thin bars".to_owned());
    let which = || "the thick bar and block with the thin bars".to_owned();
    let [whole] = fused(&mut model, &with_the_block, &thin, which)[..] else {
        panic!("{}: one solid", which());
    };
    has_a_flat_face_passing_two_corners_twice(&model, whole, which);
    let both = 1.5 * 1.75_f64.sqrt() + 4.0 * 0.75_f64.asin();
    let meant = 4.0 * (4.0 * PI + 15.0 - both) + 4.0 * PI;
    valid_and_of_volume(&model, whole, meant, which);
}

/// Panics, saying `which`, unless some flat face of `solid` has a wire that
/// starts two of its sides at each of two corners: the shape these tests are
/// about, kept so that a fuse that writes the face another way is noticed.
fn has_a_flat_face_passing_two_corners_twice(
    model: &Model,
    solid: SolidKey,
    which: impl Fn() -> String,
) {
    let view = model.solid(solid).expect("this model made the key");
    let passes_two_corners_twice = view
        .faces()
        .filter(|face| face.surface().as_plane().is_some())
        .flat_map(|face| face.wires())
        .any(|wire| {
            let starts: Vec<_> = wire.coedges().map(|coedge| coedge.start().key()).collect();
            let twice = starts
                .iter()
                .enumerate()
                .filter(|&(at, start)| starts[..at].contains(start))
                .count();
            twice == 2
        });
    assert!(
        passes_two_corners_twice,
        "{}: no flat face has a wire through two corners twice",
        which()
    );
}

/// Panics, saying `which`, unless the model validates and `solid` measures
/// `meant` at a deflection of `0.01` to within the measure's own error.
fn valid_and_of_volume(model: &Model, solid: SolidKey, meant: f64, which: impl Fn() -> String) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{}: {why}", which()));
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{}: {refused:?}", which()));
    println!(
        "{}: {} ± {} against {meant}",
        which(),
        measured.volume(),
        measured.volume_error()
    );
    assert!(
        (measured.volume() - meant).abs() <= measured.volume_error(),
        "{}: {} ± {} against {meant}",
        which(),
        measured.volume(),
        measured.volume_error()
    );
}

/// `one` united with `other`, panicking with `which` when it is refused.
fn fused(
    model: &mut Model,
    one: &[SolidKey],
    other: &[SolidKey],
    which: impl Fn() -> String,
) -> Vec<SolidKey> {
    model
        .fuse(
            one,
            other,
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{}: {refused:?}", which()))
        .solids()
        .to_vec()
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

/// A bar of `radius` up `z` from `foot`, `height` long.
fn a_bar(model: &mut Model, radius: f64, height: f64, foot: [f64; 3]) -> SolidKey {
    model
        .add_round_bar(radius, height, &moved_to(foot))
        .expect("a bar")
}

/// The box from `low` to `high`.
fn a_block(model: &mut Model, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, high[2] - low[2], &moved_to([0.0, 0.0, low[2]]))
        .expect("a block")
}

fn moved_to([x, y, z]: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite move")
}
