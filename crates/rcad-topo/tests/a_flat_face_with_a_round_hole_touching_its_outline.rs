//! Flat faces whose one wire runs right round a whole circle at one corner
//! among its other sides: a round hole touching the face's outline at one
//! point (row Q3250, found by Q3246).
//!
//! A pin of radius 1 standing through a plate and touching the plate's side
//! leaves the plate's top a square less the pin's disk, and the disk's circle
//! touches the square's side at one corner. The fuse makes that one wire: along
//! the side to the corner, right round the circle back to it, and on along the
//! side. A thin bar standing through a thick one and touching its wall does the
//! same to the thick bar's ends, where the outline is arcs of the thick bar's
//! rim and the block's straight sides.
//!
//! Each such face was refused `AFaceTrimmedByACurveThisCannotFollow`: one with
//! only straight sides besides the circle where its polygon is walked, and one
//! with arcs where the way each side runs is read. So was every fuse that asked
//! one about a point. The circle is now left out of the polygon, the disk read
//! against the pipe across it the way a round wire's disk is, and the face held
//! where the polygon alone says it is, turned round once for each disk that
//! holds the point.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{FaceView, Model, OnAFace, ProfileBuilder, Rule, SolidKey, SolidView};

/// The top and the bottom of a plate `4 × 4 × 1` with a pin of radius 1 at
/// `(1, 2)` standing through it, touching its side `x = 0` at `(0, 2)`, hold
/// the plate outside the pin, and not the pin's disk.
#[test]
fn a_plates_top_and_bottom_with_a_pin_touching_its_side_hold_the_plate_outside_the_pin() {
    let mut model = a_model();
    let pinned = a_plate_with_a_pin_touching_its_side(&mut model);
    let view = model.solid(pinned).expect("this model made the key");
    for z in [0.0, 1.0] {
        let face = the_flat_face_at(view, z);
        for ([x, y], held) in [
            ([3.0, 2.0], OnAFace::Held),
            ([2.5, 2.0], OnAFace::Held),
            ([0.5, 0.5], OnAFace::Held),
            ([0.5, 3.5], OnAFace::Held),
            ([3.5, 3.5], OnAFace::Held),
            ([1.0, 2.0], OnAFace::NotHeld),
            ([0.5, 2.0], OnAFace::NotHeld),
            ([1.0, 2.5], OnAFace::NotHeld),
            ([5.0, 2.0], OnAFace::NotHeld),
            ([-1.0, 2.0], OnAFace::NotHeld),
            ([2.0, 2.0], OnAFace::OnItsBoundary),
            ([0.0, 2.0], OnAFace::OnItsBoundary),
            ([0.0, 1.0], OnAFace::OnItsBoundary),
            ([4.0, 4.0], OnAFace::OnItsBoundary),
        ] {
            let asked = format!("the plate's face at z = {z}, ({x}, {y})");
            let point = PredicatePoint::from([x, y, z]);
            assert_eq!(the_face_holds(view, face, point, &asked), held, "{asked}");
        }
    }
}

/// That plate and pin united with a cube `0.5` wide on the plate's corner
/// `(4, 4)`, from `z = 0.5` to `1.5`: the plate, the pin's two ends past it,
/// and the half of the cube above the plate, `16 + 2π + 0.125`.
#[test]
fn a_plate_with_a_pin_touching_its_side_unites_with_a_cube_on_its_corner() {
    let mut model = a_model();
    let pinned = a_plate_with_a_pin_touching_its_side(&mut model);
    let cube = a_box(&mut model, [[3.0, 3.5], [3.0, 3.5], [0.5, 1.5]]);
    let whole = the_one_solid(&mut model, pinned, cube, Rule::Union);
    valid_and_of_volume(&model, whole, 16.0 + 2.0 * PI + 0.125, "the cube");
}

/// The ends `z = −2` and `z = 2` of a thick bar with a block on its side and a
/// thin bar touching its wall from inside hold the thick bar and the block
/// outside the thin bar, and not the thin bar's disk.
#[test]
fn a_thick_bars_ends_with_a_thin_bar_touching_its_wall_hold_what_is_outside_the_thin_bar() {
    let mut model = a_model();
    let pinned = a_thick_bar_with_a_block_and_a_thin_bar_touching_its_wall(&mut model);
    let view = model.solid(pinned).expect("this model made the key");
    for z in [-2.0, 2.0] {
        let face = the_flat_face_at(view, z);
        for ([x, y], held) in [
            ([1.0, 0.0], OnAFace::Held),
            ([4.0, 1.0], OnAFace::Held),
            ([-1.0, 1.5], OnAFace::Held),
            ([-1.0, -1.5], OnAFace::Held),
            ([0.5, 1.0], OnAFace::Held),
            ([-1.0, 0.0], OnAFace::NotHeld),
            ([-1.5, 0.0], OnAFace::NotHeld),
            ([-1.0, 0.5], OnAFace::NotHeld),
            ([-1.5, 1.5], OnAFace::NotHeld),
            ([6.0, 0.0], OnAFace::NotHeld),
            ([0.0, 0.0], OnAFace::OnItsBoundary),
            ([-2.0, 0.0], OnAFace::OnItsBoundary),
            ([0.0, 2.0], OnAFace::OnItsBoundary),
            ([5.0, 0.0], OnAFace::OnItsBoundary),
        ] {
            let asked = format!("the end at z = {z}, ({x}, {y})");
            let point = PredicatePoint::from([x, y, z]);
            assert_eq!(the_face_holds(view, face, point, &asked), held, "{asked}");
        }
    }
}

/// That bar united with a plate across either end, `x` −3 to 6, `y` −3 to 3,
/// `1` thick and centred on the end: the bar less the half of its end the
/// plate takes in, with the plate, `3.5 A + 1.5π + 54`.
#[test]
fn a_plate_across_either_end_of_a_thick_bar_with_a_thin_bar_touching_its_wall_unites_with_it() {
    for (end, z) in [("top", 1.5), ("bottom", -2.5)] {
        let mut model = a_model();
        let pinned = a_thick_bar_with_a_block_and_a_thin_bar_touching_its_wall(&mut model);
        let plate = a_box(&mut model, [[-3.0, 6.0], [-3.0, 3.0], [z, z + 1.0]]);
        let whole = the_one_solid(&mut model, pinned, plate, Rule::Union);
        let meant = 3.5 * the_bar_and_the_blocks_cross_section() + 1.5 * PI + 54.0;
        valid_and_of_volume(&model, whole, meant, &format!("the plate across the {end}"));
    }
}

/// That bar united with a cube `1` wide on the block's top, `x` 3 to 4, `y`
/// −0.5 to 0.5, `z` 1 to 3: the bar and the half of the cube above it.
#[test]
fn a_cube_on_the_block_of_a_thick_bar_with_a_thin_bar_touching_its_wall_unites_with_it() {
    let mut model = a_model();
    let pinned = a_thick_bar_with_a_block_and_a_thin_bar_touching_its_wall(&mut model);
    let cube = a_box(&mut model, [[3.0, 4.0], [-0.5, 0.5], [1.0, 3.0]]);
    let whole = the_one_solid(&mut model, pinned, cube, Rule::Union);
    valid_and_of_volume(&model, whole, the_bars_volume() + 1.0, "the cube");
}

/// That bar less a slot `0.5` wide across its top end, `x` −3 to 6, `|y| ≤
/// 0.25`, `z` 1.5 to 2.5. The slot takes the strip `|y| ≤ 0.25` of the thick
/// bar and the block `0.5` deep, and of the thin bar's end `0.5` deep.
#[test]
fn a_slot_cut_across_the_end_of_a_thick_bar_with_a_thin_bar_touching_its_wall() {
    let mut model = a_model();
    let pinned = a_thick_bar_with_a_block_and_a_thin_bar_touching_its_wall(&mut model);
    let slot = a_box(&mut model, [[-3.0, 6.0], [-0.25, 0.25], [1.5, 2.5]]);
    let whole = the_one_solid(&mut model, pinned, slot, Rule::Difference);
    let half_wide = 0.25;
    let the_strip_of_a_disk = |radius: f64| {
        2.0 * (half_wide * (radius * radius - half_wide * half_wide).sqrt()
            + radius * radius * (half_wide / radius).asin())
    };
    let the_bar_and_the_block = 0.5 * (5.0 * 2.0 * half_wide + 0.5 * the_strip_of_a_disk(2.0));
    let the_thin_bar = 0.5 * the_strip_of_a_disk(1.0);
    let meant = the_bars_volume() - the_bar_and_the_block - the_thin_bar;
    valid_and_of_volume(&model, whole, meant, "the slot");
}

/// A plate `4 × 5 × 1` with two pins of radius 1 standing through it at
/// `(1, 1.25)` and `(1, 3.75)`, both touching its side `x = 0`: its top and
/// bottom have two such circles in one wire, and hold the plate outside both
/// pins. The second union asks the first one's top and bottom where the second
/// pin goes, and the plate with the pins' four ends past it is `20 + 4π`.
#[test]
fn a_plate_with_two_pins_touching_one_side_holds_what_is_outside_both() {
    let mut model = a_model();
    let plate = a_box(&mut model, [[0.0, 4.0], [0.0, 5.0], [0.0, 1.0]]);
    let [one, other] = [1.25, 3.75].map(|y| {
        model
            .add_round_bar(1.0, 3.0, &shift(1.0, y, -1.0))
            .expect("a bar of radius one")
    });
    let pinned = the_one_solid(&mut model, plate, one, Rule::Union);
    let whole = the_one_solid(&mut model, pinned, other, Rule::Union);
    let view = model.solid(whole).expect("this model made the key");
    for z in [0.0, 1.0] {
        let face = the_flat_face_at(view, z);
        for ([x, y], held) in [
            ([3.0, 2.5], OnAFace::Held),
            ([0.5, 2.5], OnAFace::Held),
            ([0.25, 0.1], OnAFace::Held),
            ([1.0, 1.25], OnAFace::NotHeld),
            ([1.0, 3.75], OnAFace::NotHeld),
            ([0.5, 3.75], OnAFace::NotHeld),
            ([-1.0, 2.5], OnAFace::NotHeld),
            ([0.0, 1.25], OnAFace::OnItsBoundary),
            ([0.0, 3.75], OnAFace::OnItsBoundary),
            ([2.0, 3.75], OnAFace::OnItsBoundary),
            ([0.0, 2.5], OnAFace::OnItsBoundary),
        ] {
            let asked = format!("the plate's face at z = {z}, ({x}, {y})");
            let point = PredicatePoint::from([x, y, z]);
            assert_eq!(the_face_holds(view, face, point, &asked), held, "{asked}");
        }
    }
    valid_and_of_volume(&model, whole, 20.0 + 4.0 * PI, "two pins");
}

/// A plate `4 × 4 × 1` on `z = 0` united with a pin of radius 1 at `(1, 2)`
/// from `z = −1` to 2, whose wall touches the plate's side `x = 0` at
/// `(0, 2)`. The fuse answers it at the base; it is the faces it leaves that
/// were refused.
fn a_plate_with_a_pin_touching_its_side(model: &mut Model) -> SolidKey {
    let plate = a_box(model, [[0.0, 4.0], [0.0, 4.0], [0.0, 1.0]]);
    let pin = model
        .add_round_bar(1.0, 3.0, &shift(1.0, 2.0, -1.0))
        .expect("a bar of radius one");
    the_one_solid(model, plate, pin, Rule::Union)
}

/// A thick bar of radius 2 on the `z` axis from `z = −2` to 2, united with a
/// block `x` 0 to 5, `y` −1.5 to 1.5 over the same height, then with a thin bar
/// of radius 1 at `(−1, 0)` from `z = −3` to 3, which touches the thick bar's
/// wall along `x = −2, y = 0` (Q3241's shape, unturned and with one thin bar).
fn a_thick_bar_with_a_block_and_a_thin_bar_touching_its_wall(model: &mut Model) -> SolidKey {
    let thick = model
        .add_round_bar(2.0, 4.0, &shift(0.0, 0.0, -2.0))
        .expect("a bar of radius two");
    let block = a_box(model, [[0.0, 5.0], [-1.5, 1.5], [-2.0, 2.0]]);
    let thin = model
        .add_round_bar(1.0, 6.0, &shift(-1.0, 0.0, -3.0))
        .expect("a bar of radius one");
    let with_the_block = the_one_solid(model, thick, block, Rule::Union);
    the_one_solid(model, with_the_block, thin, Rule::Union)
}

/// The area `A` the thick bar and the block cover across `z`: the disk of
/// radius 2 and the rectangle `5 × 3`, less the part of the disk with `x ≥ 0`
/// and `|y| ≤ 1.5`, whose area is `1.5 √1.75 + 4 asin 0.75`.
fn the_bar_and_the_blocks_cross_section() -> f64 {
    let both = 1.5 * 1.75_f64.sqrt() + 4.0 * 0.75_f64.asin();
    4.0 * PI + 15.0 - both
}

/// The volume of [`a_thick_bar_with_a_block_and_a_thin_bar_touching_its_wall`]:
/// `A` over a height of 4, and the thin bar's two ends of length 1 past the
/// thick bar's, `4 A + 2π`. Between them the thin bar is inside the thick one.
fn the_bars_volume() -> f64 {
    4.0 * the_bar_and_the_blocks_cross_section() + 2.0 * PI
}

/// Panics, saying `which`, unless the model validates and `solid` measures
/// `meant` to within the measure's own error at a deflection of `0.01`.
fn valid_and_of_volume(model: &Model, solid: SolidKey, meant: f64, which: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{which}: {why}"));
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{which}: {refused:?}"));
    println!(
        "{which}: {} ± {} against {meant}",
        measured.volume(),
        measured.volume_error()
    );
    assert!(
        (measured.volume() - meant).abs() <= measured.volume_error(),
        "{which}: {} ± {} against {meant}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The face on the plane `z = height`, facing up or down.
fn the_flat_face_at(view: SolidView<'_>, height: f64) -> FaceView<'_> {
    view.faces()
        .find(|face| {
            let Surface::Plane(flat) = face.surface() else {
                return false;
            };
            flat.normal().components()[2].abs() == 1.0 && flat.origin().coordinates()[2] == height
        })
        .unwrap_or_else(|| panic!("a flat face at z = {height}"))
}

/// What `face` says of `point`, certified, having logged nothing on the way.
fn the_face_holds(
    view: SolidView<'_>,
    face: FaceView<'_>,
    point: PredicatePoint,
    asked: &str,
) -> OnAFace {
    let mut log = ProvenanceLog::new();
    let said = view
        .face_holds_the_point(face.key(), point, &Policy::certified(), &mut log)
        .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
    assert!(
        log.is_empty(),
        "{asked}: a certified answer logs nothing: {log:?}"
    );
    said
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

fn shift(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

fn a_box(model: &mut Model, [x, y, z]: [[f64; 2]; 3]) -> SolidKey {
    let rectangle =
        ProfileBuilder::new(vec![[x[0], y[0]], [x[1], y[0]], [x[1], y[1]], [x[0], y[1]]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a rectangle");
    model
        .add_extrusion(&rectangle, z[1] - z[0], &shift(0.0, 0.0, z[0]))
        .expect("a positive height")
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let fused = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{rule:?}: {refused:?}"));
    let [solid] = fused.solids() else {
        panic!("{rule:?} is one solid: {:?}", fused.solids());
    };
    *solid
}
