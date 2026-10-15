//! Flat faces with an arc among their sides, asked about a point on the line
//! through two of their corners, given as a box a rounding wide
//! (`docs/decisions.md` §191.5, row Q2035).
//!
//! `holds.rs` winds such a face's own sides, each arc read as its chord, and
//! runs its half-line from the point towards one corner. A box on the line
//! from that corner through another straddles it, so that other corner's side
//! of the line has no sign, and the face was refused `Undecidable` with no
//! second aim — the refusal row Q1834 lifted from faces with straight sides
//! alone. How often the sides wind round a point does not depend on which way
//! the half-line runs, so the face tries every corner and then the middle of
//! every side, as a face with straight sides does.
//!
//! Two faces reach that walk. A round bar cut in two beside its axis, its half
//! shared with the block over `x < c` bitten by a ball whose circle on the cut
//! takes an arc out of the top of that rectangle: the steps that read a face as
//! the part in front of all its cuts say nothing about it. And the top of a
//! plate with a round tab and a round hole, which winds straight away because
//! of the hole's circle.

use rcad_geom::{Isometry, Scale, Surface, Vector};
use rcad_predicates::{Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{FaceView, Model, OnAFace, ProfileBuilder, Rule, SolidKey, SolidView};

/// Where the plane cuts the bar: off the axis on either side, and through it.
/// The ball stays clear of the bar's wall for each.
const CUTS: [f64; 5] = [0.5, 0.25, 0.125, -0.5, 0.0];

/// Where the bar's bottom is and how tall it is.
const BARS: [(f64, f64); 3] = [(2.0, 2.0), (0.0, 1.0), (0.0, 2.0)];

/// A point a quarter of the way along either diagonal of the cut face's
/// rectangle, from either end, as a box a rounding wide, is inside the face,
/// and the face says so.
#[test]
fn a_half_bars_cut_face_bitten_by_a_ball_holds_a_box_on_either_diagonal() {
    for cut in CUTS {
        for (bottom, height) in BARS {
            let mut model = a_model();
            let half = a_half_bar_bitten_by_a_ball(&mut model, cut, bottom, height);
            let view = model.solid(half).expect("this model made the key");
            let (face, [first, second, third, fourth]) =
                the_cut_face(view, cut, bottom, bottom + height);
            for [from, to] in [
                [first, third],
                [third, first],
                [second, fourth],
                [fourth, second],
            ] {
                let point = a_box_part_of_the_way(from, to, 4.0);
                let asked = format!("cut at x = {cut}, bar from z = {bottom}, {point:?}");
                assert_eq!(the_face_holds(view, face, point, &asked), OnAFace::Held);
            }
        }
    }
}

/// The middle of the rectangle, as a box a rounding wide, is on both its
/// diagonals, so a half-line aimed at any of its four corners runs on through
/// the opposite one, and it is still inside the face, clear of the bite.
#[test]
fn a_half_bars_cut_face_bitten_by_a_ball_holds_a_box_at_its_middle() {
    for cut in CUTS {
        for (bottom, height) in BARS {
            let mut model = a_model();
            let half = a_half_bar_bitten_by_a_ball(&mut model, cut, bottom, height);
            let view = model.solid(half).expect("this model made the key");
            let (face, [first, _, third, _]) = the_cut_face(view, cut, bottom, bottom + height);
            let point = a_box_part_of_the_way(first, third, 2.0);
            let asked = format!("cut at x = {cut}, bar from z = {bottom}, {point:?}");
            assert_eq!(the_face_holds(view, face, point, &asked), OnAFace::Held);
        }
    }
}

/// A point a quarter of a diagonal past either end of it, as a box a rounding
/// wide, is on the same line and off the face, and the face says that too.
#[test]
fn a_half_bars_cut_face_bitten_by_a_ball_holds_no_box_past_a_corner_on_a_diagonal() {
    for cut in CUTS {
        for (bottom, height) in BARS {
            let mut model = a_model();
            let half = a_half_bar_bitten_by_a_ball(&mut model, cut, bottom, height);
            let view = model.solid(half).expect("this model made the key");
            let (face, [first, second, third, fourth]) =
                the_cut_face(view, cut, bottom, bottom + height);
            for [from, to] in [
                [first, third],
                [third, first],
                [second, fourth],
                [fourth, second],
            ] {
                let point = a_box_part_of_the_way(from, to, 0.8);
                let asked = format!("cut at x = {cut}, bar from z = {bottom}, {point:?}");
                assert_eq!(the_face_holds(view, face, point, &asked), OnAFace::NotHeld);
            }
        }
    }
}

/// The top of a plate `10 × 10 × 2` with a round tab of radius 3 at `(10, 5)`
/// and a round hole of radius 1 at `(3, 5)`. The square's corners are exact,
/// but a box a rounding wide round a point of either diagonal still straddles
/// that diagonal, so the half-line aimed along it had no sign for the corner at
/// its other end. Either diagonal passes the hole `√2` from its centre.
#[test]
fn a_plates_top_with_a_round_tab_and_a_round_hole_holds_a_box_on_either_diagonal() {
    let mut model = a_model();
    let plate = a_box(&mut model, [[0.0, 10.0], [0.0, 10.0], [0.0, 2.0]]);
    let tab = model
        .add_round_bar(3.0, 2.0, &shift(10.0, 5.0, 0.0))
        .expect("a bar of radius three");
    let tabbed = the_one_solid(&mut model, plate, tab, Rule::Union);
    let hole = model
        .add_round_bar(1.0, 4.0, &shift(3.0, 5.0, -1.0))
        .expect("a bar of radius one");
    let holed = the_one_solid(&mut model, tabbed, hole, Rule::Difference);
    let view = model.solid(holed).expect("this model made the key");
    let top = view
        .faces()
        .find(|face| {
            matches!(face.surface(), Surface::Plane(flat) if flat.origin().coordinates()[2] == 2.0)
        })
        .expect("the plate has a top");
    let [first, second, third, fourth] = [
        [0.0, 0.0, 2.0],
        [10.0, 0.0, 2.0],
        [10.0, 10.0, 2.0],
        [0.0, 10.0, 2.0],
    ];
    for ([from, to], parts, held) in [
        ([first, third], 4.0, OnAFace::Held),
        ([third, first], 4.0, OnAFace::Held),
        ([second, fourth], 4.0, OnAFace::Held),
        ([fourth, second], 4.0, OnAFace::Held),
        ([first, third], 2.0, OnAFace::Held),
        ([first, third], 0.8, OnAFace::NotHeld),
        ([second, fourth], 0.8, OnAFace::NotHeld),
    ] {
        let point = a_box_part_of_the_way(from, to, parts);
        assert_eq!(
            the_face_holds(view, top, point, &format!("{point:?}")),
            held,
            "{point:?}"
        );
    }
}

#[test]
fn probe_two_arcs_on_different_lines() {
    let mut model = a_model();
    let piece = a_piece_with_two_arcs_on_crossing_chords(&mut model, 0.0, 2.0);
    let view = model.solid(piece).expect("this model made the key");
    for z in [0.0, 2.0] {
        let face = the_flat_face_at(view, z);
        let corners: Vec<[f64; 3]> = face
            .wires()
            .flat_map(|wire| wire.coedges())
            .map(|side| side.start().point().coordinates())
            .collect();
        eprintln!("piece z = {z}: corners {corners:?}");
        let point = PredicatePoint::from([0.0, 0.0, z]);
        let mut log = ProvenanceLog::new();
        let said = view.face_holds_the_point(face.key(), point, &Policy::certified(), &mut log);
        eprintln!("piece z = {z}: {said:?} log empty {}", log.is_empty());
    }
    let mut model = a_model();
    let cutter = a_piece_with_two_arcs_on_crossing_chords(&mut model, -1.0, 4.0);
    let diamond = ProfileBuilder::new(vec![[20.0, 0.0], [0.0, 20.0], [-20.0, 0.0], [0.0, -20.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a diamond");
    let plate = model
        .add_extrusion(&diamond, 2.0, &shift(0.0, 0.0, 0.0))
        .expect("a positive height");
    let holed = the_one_solid(&mut model, plate, cutter, Rule::Difference);
    let view = model.solid(holed).expect("this model made the key");
    for z in [0.0, 2.0] {
        let face = the_flat_face_at(view, z);
        let corners: Vec<[f64; 3]> = face
            .wires()
            .flat_map(|wire| wire.coedges())
            .map(|side| side.start().point().coordinates())
            .collect();
        eprintln!("plate z = {z}: corners {corners:?}");
        let point = PredicatePoint::from([0.0, 0.0, z]);
        let mut log = ProvenanceLog::new();
        let said = view.face_holds_the_point(face.key(), point, &Policy::certified(), &mut log);
        eprintln!("plate z = {z}: {said:?} log empty {}", log.is_empty());
    }
}

fn a_piece_with_two_arcs_on_crossing_chords(
    model: &mut Model,
    bottom: f64,
    height: f64,
) -> SolidKey {
    let lower_bar = model
        .add_round_bar(5.0, 10.0, &shift(0.0, 4.0, -4.0))
        .expect("a bar of radius five");
    let under_the_line = a_prism(
        model,
        vec![[-15.0, -30.0], [21.0, -30.0], [21.0, -24.0], [-15.0, 24.0]],
        -5.0,
        12.0,
    );
    let bitten = the_one_solid(model, lower_bar, under_the_line, Rule::Intersection);
    let beside = a_prism(
        model,
        vec![[-18.0, -30.0], [12.0, -30.0], [-18.0, 30.0]],
        -6.0,
        14.0,
    );
    let joined = the_one_solid(model, beside, bitten, Rule::Union);
    for face in model.solid(joined).unwrap().faces() {
        let corners: Vec<[f64; 3]> = face
            .wires()
            .flat_map(|wire| wire.coedges())
            .map(|side| side.start().point().coordinates())
            .collect();
        eprintln!("U face {:?}: {corners:?}", face.surface());
    }
    let bar = model
        .add_round_bar(5.0, height, &shift(0.0, -1.0, bottom))
        .expect("a bar of radius five");
    the_one_solid(model, bar, joined, Rule::Intersection)
}

fn a_prism(model: &mut Model, corners: Vec<[f64; 2]>, bottom: f64, height: f64) -> SolidKey {
    let profile = ProfileBuilder::new(corners)
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a polygon");
    model
        .add_extrusion(&profile, height, &shift(0.0, 0.0, bottom))
        .expect("a positive height")
}

fn the_flat_face_at(view: SolidView<'_>, z: f64) -> FaceView<'_> {
    view.faces()
        .find(|face| {
            let Surface::Plane(flat) = face.surface() else {
                return false;
            };
            flat.normal().components()[2].abs() == 1.0 && flat.origin().coordinates()[2] == z
        })
        .unwrap_or_else(|| panic!("a flat face at z = {z}"))
}

/// The bar of radius 1 from `z = bottom` up `height` shared with the block over
/// `x < cut`, less a ball of radius `0.3` at `(cut + 0.1, 0, top + 0.1)`. The
/// ball's circle on the cut has radius `√0.08` and its centre `0.1` above the
/// top, so it takes an arc out of the top of the cut face between
/// `y = ±√0.07`, and its poles are off the cut.
fn a_half_bar_bitten_by_a_ball(model: &mut Model, cut: f64, bottom: f64, height: f64) -> SolidKey {
    let bar = model
        .add_round_bar(1.0, height, &shift(0.0, 0.0, bottom))
        .expect("a bar of radius one");
    let block = a_box(model, [[-10.0, cut], [-10.0, 10.0], [-10.0, 10.0]]);
    let half = the_one_solid(model, bar, block, Rule::Intersection);
    let ball = model
        .add_ball(0.3, &shift(cut + 0.1, 0.0, bottom + height + 0.1))
        .expect("a ball of radius 0.3");
    the_one_solid(model, half, ball, Rule::Difference)
}

/// The half's face on the plane `x = cut`, and the four corners of the
/// rectangle the bar cuts that plane in, which are corners of the face too:
/// low `y` and high `y` at the bottom, then high `y` and low `y` at the top.
fn the_cut_face(
    view: SolidView<'_>,
    cut: f64,
    bottom: f64,
    top: f64,
) -> (FaceView<'_>, [[f64; 3]; 4]) {
    let face = view
        .faces()
        .find(|face| {
            let Surface::Plane(flat) = face.surface() else {
                return false;
            };
            flat.normal().components() == [1.0, 0.0, 0.0] && flat.offset() == cut
        })
        .expect("the half has the cut as a face");
    let corners: Vec<[f64; 3]> = face
        .wires()
        .flat_map(|wire| wire.coedges())
        .map(|side| side.start().point().coordinates())
        .collect();
    let widest = |z: f64, sign: f64| {
        *corners
            .iter()
            .filter(|corner| corner[2] == z && corner[1] * sign > 0.0)
            .max_by(|one, other| one[1].abs().total_cmp(&other[1].abs()))
            .unwrap_or_else(|| panic!("a corner at z = {z} on that side: {corners:?}"))
    };
    (
        face,
        [
            widest(bottom, -1.0),
            widest(bottom, 1.0),
            widest(top, 1.0),
            widest(top, -1.0),
        ],
    )
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

/// The point `1 / parts` of the way from `from` to `to`, as a box one rounding
/// wide either side along every axis the two differ on.
fn a_box_part_of_the_way(from: [f64; 3], to: [f64; 3], parts: f64) -> PredicatePoint {
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        if from[axis] == to[axis] {
            return Interval::exactly(from[axis]);
        }
        let near = ((parts - 1.0) * from[axis] + to[axis]) / parts;
        Interval::new(near.next_down(), near.next_up()).expect("a box round a finite number")
    }))
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
