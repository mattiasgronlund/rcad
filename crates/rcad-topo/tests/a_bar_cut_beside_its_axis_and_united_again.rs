//! A round bar of radius 1 cut in two on a plane `x = c` beside its axis, and
//! its two halves united again (`docs/decisions.md` §191.5, row Q1834).
//!
//! The halves are the bar less a block over `x < c` and the bar shared with
//! that block, so the two lie flush along the cut face, a rectangle whose
//! corners are where the plane crosses the bar's two rims. The fuse picks a
//! point inside a piece of that face on the line through two of its opposite
//! corners, as a box a rounding wide, and asks the other half's face whether it
//! holds it. The walk round that face aimed its half-line at one of those two
//! corners, so the other sat on the half-line's line, its side had no sign, and
//! the union was refused `Undecidable` for every `c` off the axis. On the axis
//! the corners are exact and so is the point, and the sign there is zero.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Model, OnAFace, ProfileBuilder, Rule, SolidKey};

/// Where the plane cuts the bar: off the axis on either side, and through it.
const CUTS: [f64; 6] = [0.5, 0.25, 0.125, -0.5, 0.75, 0.0];

/// Where the bar's bottom is and how tall it is.
const BARS: [(f64, f64); 3] = [(2.0, 2.0), (0.0, 1.0), (0.0, 2.0)];

/// The two halves united, either way round, are one solid of the whole bar's
/// volume, `π h`, and a certified fuse logs nothing on the way.
#[test]
fn a_bar_cut_beside_its_axis_and_united_again_is_the_whole_bar() {
    for cut in CUTS {
        for (bottom, height) in BARS {
            for halves_in_order in [true, false] {
                let asked = format!(
                    "cut at x = {cut}, bar from z = {bottom} up {height}, halves in order {halves_in_order}"
                );
                let mut model = a_model();
                let [kept, cut_off] = the_two_halves(&mut model, cut, bottom, height);
                let [one, other] = if halves_in_order {
                    [kept, cut_off]
                } else {
                    [cut_off, kept]
                };
                let mut log = ProvenanceLog::new();
                let united = model
                    .fuse(
                        &[one],
                        &[other],
                        Rule::Union,
                        &Policy::certified(),
                        &mut log,
                    )
                    .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
                assert!(
                    log.is_empty(),
                    "{asked}: a certified fuse logs nothing: {log:?}"
                );
                let [solid] = united.solids() else {
                    panic!("{asked}: one solid, not {:?}", united.solids());
                };
                holds(&model, *solid, PI * height, &asked);
            }
        }
    }
}

/// The half shared with the block has the cut as its face on `x = c`, a
/// rectangle. A point a quarter of the way along either diagonal from either
/// end, as a box a rounding wide, is inside that face, and the face says so.
#[test]
fn a_half_bars_cut_face_holds_a_box_on_either_diagonal() {
    for cut in CUTS {
        for (bottom, height) in BARS {
            let mut model = a_model();
            let [_, half] = the_two_halves(&mut model, cut, bottom, height);
            let view = model.solid(half).expect("this model made the key");
            let face = view
                .faces()
                .find(|face| match face.surface() {
                    Surface::Plane(flat) => {
                        flat.normal().components() == [1.0, 0.0, 0.0] && flat.offset() == cut
                    }
                    _ => false,
                })
                .expect("the half has the cut as a face");
            let corners: Vec<[f64; 3]> = face
                .wires()
                .flat_map(|wire| wire.coedges())
                .map(|side| side.start().point().coordinates())
                .collect();
            let [first, second, third, fourth] = corners[..] else {
                panic!("the cut face is a rectangle: {corners:?}");
            };
            for [from, to] in [
                [first, third],
                [third, first],
                [second, fourth],
                [fourth, second],
            ] {
                let point = a_quarter_of_the_way(from, to);
                let mut log = ProvenanceLog::new();
                let said = view
                    .face_holds_the_point(face.key(), point, &Policy::certified(), &mut log)
                    .unwrap_or_else(|refused| {
                        panic!("cut at x = {cut}, {point:?} towards {to:?}: {refused:?}")
                    });
                assert_eq!(said, OnAFace::Held, "cut at x = {cut}, {point:?}");
                assert!(log.is_empty(), "a certified answer logs nothing: {log:?}");
            }
        }
    }
}

/// The point a quarter of the way from `from` to `to`, as a box one rounding
/// wide either side along every axis the two differ on.
fn a_quarter_of_the_way(from: [f64; 3], to: [f64; 3]) -> PredicatePoint {
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        if from[axis] == to[axis] {
            return Interval::exactly(from[axis]);
        }
        let near = (3.0 * from[axis] + to[axis]) / 4.0;
        Interval::new(near.next_down(), near.next_up()).expect("a box round a finite number")
    }))
}

/// The bar less a block over `x < cut`, and the bar shared with that block.
fn the_two_halves(model: &mut Model, cut: f64, bottom: f64, height: f64) -> [SolidKey; 2] {
    let below = [[-10.0, cut], [-10.0, 10.0], [-10.0, 10.0]];
    let [one, other] = [a_bar(model, bottom, height), a_bar(model, bottom, height)];
    let [block, its_copy] = [a_box(model, below), a_box(model, below)];
    [
        the_one_solid(model, one, block, Rule::Difference),
        the_one_solid(model, other, its_copy, Rule::Intersection),
    ]
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

fn a_bar(model: &mut Model, bottom: f64, height: f64) -> SolidKey {
    model
        .add_round_bar(
            1.0,
            height,
            &Isometry::translation(Vector::new(0.0, 0.0, bottom)).expect("a finite shift"),
        )
        .expect("a bar of radius one")
}

fn a_box(model: &mut Model, [x, y, z]: [[f64; 2]; 3]) -> SolidKey {
    let rectangle =
        ProfileBuilder::new(vec![[x[0], y[0]], [x[1], y[0]], [x[1], y[1]], [x[0], y[1]]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            z[1] - z[0],
            &Isometry::translation(Vector::new(0.0, 0.0, z[0])).expect("a finite shift"),
        )
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

fn holds(model: &Model, solid: SolidKey, volume: f64, asked: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{asked}: {} ± {} against {volume}",
        measured.volume(),
        measured.volume_error()
    );
}
