//! Q992's notched plate, extruded one up `z`, with a half ball sunk in its top,
//! fused every way (Q2061, Q2118).
//!
//! The plate is the eight by eight square less a notch in each side, `57` in
//! area, and `53` with its square hole `[3, 5] × [3, 5]`. The half ball is the
//! ball of radius `0.6` or `0.75` about `(5.5, 2.5, 1)` cut by the plane
//! `x = 5.5` through its centre: the ball intersected with the block
//! `[4, 5.5] × [1.5, 3.5] × [0, 2]`. Its centre is on the plate's top and on
//! the plane that cuts it, so the plate holds a quarter of the ball, `πr³/3`,
//! and the rest of the half ball stands above it.
//!
//! **Every one of the 24 fuses was refused `Undecidable` at "order along a
//! crossing"**, round a circle whose walk starts where the plane `x = 5.5` cuts
//! it. The circle the plate's top cuts the ball of radius `0.6` in is walked
//! from `(5.5, 3.1, 1)`, and the circle the plane `y = 3` cuts the ball in from
//! its foot, `(5.5, 3, 0.441)` at radius `0.75`. The node where `x = 5.5` cuts
//! each is a box round the walked start that reads exactly nought along the way
//! the circle runs, and nothing put it first against the nodes clear of it.
//!
//! At the larger radius the ball reaches into the hole: its centre is `√2/2`
//! from the hole's corner `(5, 3)` across the plate's top, under `0.75`. So
//! with the hole the plate holds the quarter ball less the piece of it inside
//! the hole, [`IN_THE_HOLE`].

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The area of the notched plate, which is its volume at a height of one: the
/// square of side eight less the notches `2 × 1`, `1 × 1`, `2 × 1` and `1 × 2`.
const THE_PLATE: f64 = 57.0;

/// The area of the plate's square hole.
const THE_HOLE: f64 = 4.0;

/// The piece of the quarter ball of radius `0.75` inside the hole: the part
/// of the ball below its centre with `x ≤ 5` and `y ≥ 3`, a corner `1/2` off
/// the centre both ways.
///
/// Across the slice `x = 5.5 − u` the ball is a disc of radius
/// `ρ = √(r² − u²)`, and the piece of it beyond `y = 3` and below the plate's
/// top is half a segment, `(ρ² arccos(a/ρ) − a√(ρ² − a²))/2` with `a = 1/2`.
/// Simpson's rule over `u` from `1/2` to `√(r² − 1/4)`, in 200 000 steps and
/// in 400 000, agrees to thirteen figures. The ball of radius `0.6` reaches
/// no further than `0.6 < √2/2` from its centre, so it has no such piece.
const IN_THE_HOLE: f64 = 2.434_371_009_458e-4;

/// The meshes every volume here is read off. Fine enough that the error each
/// measure publishes is a small part of the half ball.
const A_DEFLECTION: f64 = 0.002;

/// Each of the 24 fuses answers one solid, and that solid measures its closed
/// form within the error its measure publishes: the union `P + H − I`, the
/// intersection `I`, the plate less the ball `P − I`, and the ball less the
/// plate `H − I`, for `P` the plate, `H = 2πr³/3` the half ball and `I` what
/// they share.
#[test]
fn a_half_ball_sunk_in_a_notched_plate_answers_every_fuse_with_its_volume() {
    let mut wrong = Vec::new();
    for radius in [0.6, 0.75] {
        for holed in [false, true] {
            let plate = if holed {
                THE_PLATE - THE_HOLE
            } else {
                THE_PLATE
            };
            let half_ball = 2.0 * PI * radius * radius * radius / 3.0;
            let shared = half_ball / 2.0
                - if holed && radius > 0.7 {
                    IN_THE_HOLE
                } else {
                    0.0
                };
            for (rule, plate_first, closed_form) in [
                (Rule::Union, true, plate + half_ball - shared),
                (Rule::Union, false, plate + half_ball - shared),
                (Rule::Intersection, true, shared),
                (Rule::Intersection, false, shared),
                (Rule::Difference, true, plate - shared),
                (Rule::Difference, false, half_ball - shared),
            ] {
                let what = format!(
                    "radius {radius}, holed {holed}, {rule:?} with the plate first {plate_first}"
                );
                if let Some(why) = what_is_wrong_with(radius, holed, rule, plate_first, closed_form)
                {
                    wrong.push(format!("{what}: {why}"));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// What the fuse did instead of answering one sound solid of `closed_form`, or
/// `None` where it did that.
fn what_is_wrong_with(
    radius: f64,
    holed: bool,
    rule: Rule,
    plate_first: bool,
    closed_form: f64,
) -> Option<String> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let plate = the_notched_plate(&mut model, holed);
    let half_ball = a_half_ball(&mut model, radius);
    let (argument, tool) = if plate_first {
        (plate, half_ball)
    } else {
        (half_ball, plate)
    };
    let mut log = ProvenanceLog::new();
    let fused = match model.fuse(&[argument], &[tool], rule, &Policy::certified(), &mut log) {
        Ok(fused) => fused,
        Err(why) => return Some(format!("refused {why:?}")),
    };
    if !log.records().is_empty() {
        return Some(format!("a certified fuse recorded {:?}", log.records()));
    }
    if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
        return Some(format!("the model does not validate: {why:?}"));
    }
    let [solid] = fused.solids() else {
        return Some(format!("{} solids", fused.solids().len()));
    };
    let measured = model
        .solid(*solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(A_DEFLECTION).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures");
    ((measured.volume() - closed_form).abs() > measured.volume_error()).then(|| {
        format!(
            "measured {} ± {}, and the closed form is {closed_form}",
            measured.volume(),
            measured.volume_error()
        )
    })
}

/// The plate of `a_hole_whose_only_ray_out_meets_two_cuts_a_rounding_apart_is_placed_by_the_planes`
/// in `fuse/arrange.rs`, from `z = 0` to `1`, with its square hole or without.
fn the_notched_plate(model: &mut Model, holed: bool) -> SolidKey {
    let outline = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [3.0, 0.0],
        [3.0, 1.0],
        [5.0, 1.0],
        [5.0, 0.0],
        [8.0, 0.0],
        [8.0, 5.0],
        [7.0, 5.0],
        [7.0, 6.0],
        [8.0, 6.0],
        [8.0, 8.0],
        [5.0, 8.0],
        [5.0, 7.0],
        [3.0, 7.0],
        [3.0, 8.0],
        [0.0, 8.0],
        [0.0, 5.0],
        [1.0, 5.0],
        [1.0, 3.0],
        [0.0, 3.0],
    ]);
    let outline = if holed {
        outline.with_hole(vec![[3.0, 3.0], [5.0, 3.0], [5.0, 5.0], [3.0, 5.0]])
    } else {
        outline
    };
    let profile = outline
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the notched plate's outline");
    model
        .add_extrusion(&profile, 1.0, &Isometry::identity())
        .expect("building a plate needs no decision")
}

/// The ball of `radius` about `(5.5, 2.5, 1)` intersected with the block
/// `[4, 5.5] × [1.5, 3.5] × [0, 2]`, which is its half at `x ≤ 5.5`.
fn a_half_ball(model: &mut Model, radius: f64) -> SolidKey {
    let centre = Isometry::translation(Vector::new(5.5, 2.5, 1.0)).expect("a finite placement");
    let ball = model.add_ball(radius, &centre).expect("a ball");
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [1.5, 0.0], [1.5, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let corner = Isometry::translation(Vector::new(4.0, 1.5, 0.0)).expect("a finite placement");
    let block = model
        .add_extrusion(&rectangle, 2.0, &corner)
        .expect("building a box needs no decision");
    let halved = model
        .fuse(
            &[ball],
            &[block],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a plane through a ball's centre halves it");
    let [half] = halved.solids() else {
        panic!("a ball halved is one solid, got {}", halved.solids().len());
    };
    *half
}
