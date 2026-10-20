//! A plate with a block united onto it, and a round bar standing through the
//! plate against the block's wall, all turned a quarter turn back about `z`.
//!
//! At `Scale::millimetres(100.0)` the plate is `x` and `y` 0 to 4, `z` 0 to 1;
//! the block on it is `x` 0.5 to 1.5, `y` 1.5 to 3.5, `z` 1 to 2, united with
//! it first, so the foot of the block's wall `x = 1.5` is a side of the
//! plate's top. The bar, of radius `0.5`, stands at `(2, 2.5)` from `z = −1`
//! to `2.5`, so its pipe touches that wall along the line `(1.5, 2.5)`.
//!
//! The bar's pipe cuts the plate's top in a circle that crosses nothing else
//! drawn there, a lump of the drawing of its own, and its one node is where
//! the circle starts: the circle's point along the first direction of the
//! top's plane, `+y`, wherever the turn puts the bar. Turned by `−π/2`, `+y`
//! is the way to the block's wall, so the node is the point where the pipe
//! touches the wall's foot, a rounding from that side of the top. The count
//! of how often the top's rim winds round the node could not be read there,
//! and the top was refused `CannotTellTheCellsOfAFaceApart` under every rule
//! both ways round (Q2353).
//!
//! Turned, the wall's plane misses the pipe by a rounding, where unturned it
//! touches it: the bar and the block are a rounding apart. So the bar met with
//! the plate and the bar less the plate keep nothing of the gap and answer,
//! while the union and the plate less the bar hold it, a rounding wide, and
//! are refused `DynamicRangeTooWide`.

use core::f64::consts::{FRAC_PI_2, PI};

use rcad_geom::{Axis, Deflection, Isometry, Scale, Surface, SurfaceMeet, Vector, intersect};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Invalid, Model, ProfileBuilder, Rule, SolidKey};

const BAR_IN_THE_PLATE: f64 = 0.25 * PI;
const BAR_BELOW_THE_PLATE: f64 = 0.25 * PI;
const BAR_ABOVE_THE_PLATE: f64 = 0.375 * PI;

#[test]
fn a_bar_against_a_blocks_wall_turned_a_quarter_turn_back_is_met_and_cut_by_the_plate() {
    for plate_first in [true, false] {
        let (model, fused) = fused(Rule::Intersection, plate_first);
        holds(
            &model,
            &fused.unwrap_or_else(|refused| panic!("met, plate first {plate_first}: {refused:?}")),
            &[BAR_IN_THE_PLATE],
        );
    }
    let (model, fused) = fused(Rule::Difference, false);
    holds(
        &model,
        &fused.unwrap_or_else(|refused| panic!("the bar less the plate: {refused:?}")),
        &[BAR_BELOW_THE_PLATE, BAR_ABOVE_THE_PLATE],
    );
}

#[test]
fn a_bar_a_rounding_from_a_blocks_wall_is_refused_where_the_answer_keeps_the_gap() {
    let (model, keys) = turned();
    let plate = model
        .solid(keys.plate)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface());
    let pipe = model
        .solid(keys.bar)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .find(|surface| matches!(surface, Surface::Cylinder(_)))
        .expect("a bar has a pipe");
    let flat_faces_it_meets = plate
        .filter(|surface| matches!(surface, Surface::Plane(_)))
        .filter(|flat| {
            !matches!(
                intersect(flat, &pipe, &Policy::certified(), &mut ProvenanceLog::new()),
                Ok(SurfaceMeet::Nothing)
            )
        })
        .count();
    assert_eq!(
        flat_faces_it_meets, 3,
        "the pipe meets the plate's floor and top and the block's top, and the wall's plane nowhere"
    );

    for (rule, plate_first) in [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Difference, true),
    ] {
        let (_, fused) = fused(rule, plate_first);
        assert!(
            matches!(
                fused,
                Err(Error::Invalid(Invalid::DynamicRangeTooWide { .. }))
            ),
            "{rule:?}, plate first {plate_first}: {:?}",
            fused.map(|fused| fused.solids().len())
        );
    }
}

/// The plate and the bar of [`turned`], fused under `rule`.
fn fused(rule: Rule, plate_first: bool) -> (Model, rcad_topo::Result<Fused>) {
    let (mut model, keys) = turned();
    let (first, second) = if plate_first {
        (keys.plate, keys.bar)
    } else {
        (keys.bar, keys.plate)
    };
    let fused = model.fuse(
        &[first],
        &[second],
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    if fused.is_ok() {
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?}, plate first {plate_first}: {why}"));
    }
    (model, fused)
}

/// Checks the answer is one solid for each of `volumes`, measuring them in
/// order of size.
fn holds(model: &Model, fused: &Fused, volumes: &[f64]) {
    let mut measured: Vec<(f64, f64)> = fused
        .solids()
        .iter()
        .map(|&solid| {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.001).expect("a fine mesh"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|refused| panic!("{refused:?}"));
            (measured.volume(), measured.volume_error())
        })
        .collect();
    measured.sort_by(|one, other| one.0.total_cmp(&other.0));
    assert_eq!(
        measured.len(),
        volumes.len(),
        "{measured:?} against {volumes:?}"
    );
    for (&(volume, error), &meant) in measured.iter().zip(volumes) {
        assert!(
            (volume - meant).abs() <= error,
            "{volume} ± {error} against {meant}"
        );
    }
}

struct TheTwo {
    plate: SolidKey,
    bar: SolidKey,
}

/// The plate with its block, and the bar, both turned by `−π/2` about `z`.
fn turned() -> (Model, TheTwo) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let plate = a_plate_with_a_block_on_it(&mut model);
    let bar = model
        .add_round_bar(
            0.5,
            3.5,
            &Isometry::translation(Vector::new(2.0, 2.5, -1.0)).expect("a finite placement"),
        )
        .expect("a bar");
    let turn = Isometry::rotation(&Axis::z(), -FRAC_PI_2).expect("a finite angle");
    model
        .place_solid(plate, &turn)
        .expect("this model made the key");
    model
        .place_solid(bar, &turn)
        .expect("this model made the key");
    (model, TheTwo { plate, bar })
}

fn a_plate_with_a_block_on_it(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let plate = model
        .add_extrusion(&square, 1.0, &Isometry::identity())
        .expect("building a plate needs no decision");
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let block = model
        .add_extrusion(
            &rectangle,
            1.0,
            &Isometry::translation(Vector::new(0.5, 1.5, 1.0)).expect("a finite placement"),
        )
        .expect("building a box needs no decision");
    let united = model
        .fuse(
            &[plate],
            &[block],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a block standing on a plate unites with it");
    let [one] = united.solids() else {
        panic!(
            "a block on a plate unites into one solid, got {}",
            united.solids().len()
        );
    };
    *one
}
