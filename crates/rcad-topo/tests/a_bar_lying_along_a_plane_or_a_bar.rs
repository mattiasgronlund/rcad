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
//! **Every rule is answered**, with the answers that exist: the insides are
//! apart, so the union is the two solids as they stood, two lumps whose volumes
//! are the two inputs', the intersection is no solid at all, and each difference
//! is the first solid as it stood.
//!
//! The union used to be refused `ResultIsNotManifold { count: 4 }`. A union
//! keeps both sides of the touching line, of the wall and of the bar's skin
//! alike, so the line came out one edge with four darts along it: two pieces of
//! the wall, and the skin's one piece cut open along the line and walking it
//! both ways (queue row Q760 printed them). `fuse/assemble.rs` now parts such an
//! edge into one per input face, and the wall's is dissolved as any edge between
//! two pieces of one plane is.
//!
//! **A bar whose seam is the touching line keeps its seam's history** (queue row
//! Q801). The seam is left out of the arrangement, so the side the touch cuts
//! along it named no input edge and the seam read as deleted where the answer
//! has an edge on it. `fuse/assemble.rs` names the seam for a straight side
//! between the seam's own two corners, gathers every side's input edges onto
//! the edge they share, and hands each half of a parted line its own face's.
//!
//! **A wall that only touches part of the seam loses the whole seam's history**
//! (queue row Q808, a gap of its own, not yet fixed). `the_seam_a_side_runs_along`
//! names the seam only for a side whose two ends are the seam's own two
//! corners; a wall that stops partway up the bar cuts the touching side off at
//! a new corner the wall's own top edge makes, so neither half of the split
//! seam has a side between the seam's own two ends, and both lose their
//! history at once. `a_bar_whose_seam_runs_past_where_a_wall_stops_touching_it`
//! pins this as measured, and the fixing row belongs with `fuse/assemble.rs`
//! (or `fuse/bake.rs`) in its write set, not here.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{EdgeKey, Model, ProfileBuilder, Rule, SolidKey};

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
fn a_bar_standing_against_a_blocks_wall_answers_every_rule() {
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
fn a_bar_whose_seam_is_the_line_it_touches_a_wall_along_answers_every_rule() {
    every_rule_of(
        |model| {
            let cube = a_block(model, [4.0, 4.0, 4.0]);
            let bar = a_bar(model, 1.0, 2.0, [-1.0, 2.0, 1.0]);
            [(cube, 64.0), (bar, 2.0 * PI)]
        },
        "a bar touching a wall along its seam",
    );
}

/// What became of that bar's seam, under every rule both ways round: the edges
/// of the answer along the touching line, and nothing else.
///
/// The union and the bar less the cube keep the bar whole, seam and all, so the
/// seam has an image there. The intersection and the cube less the bar keep none
/// of the bar's skin, and the cube's wall is one face again, so it has none.
#[test]
fn a_bar_whose_seam_is_the_line_it_touches_a_wall_along_keeps_its_seam_in_history() {
    let mut wrong: Vec<String> = Vec::new();
    for swapped in [false, true] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let cube = a_block(&mut model, [4.0, 4.0, 4.0]);
            let bar = a_bar(&mut model, 1.0, 2.0, [-1.0, 2.0, 1.0]);
            let seam = the_one_straight_edge_of(&model, bar);
            let [first, second] = if swapped { [bar, cube] } else { [cube, bar] };
            let what = format!("{rule:?}, bar first {swapped}");
            let answer = model
                .fuse(
                    &[first],
                    &[second],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what} refused: {why}"));
            let images = answer
                .images_of_edge(seam)
                .expect("this model made the key");
            let along_the_touch: Vec<EdgeKey> = answer
                .solids()
                .iter()
                .flat_map(|&solid| edges_of(&model, solid))
                .filter(|&edge| runs_up_the_line(&model, edge, [0.0, 2.0]))
                .collect();
            let keeps_the_bar = rule == Rule::Union || (rule == Rule::Difference && swapped);
            if along_the_touch.len() != usize::from(keeps_the_bar) || images != along_the_touch {
                wrong.push(format!(
                    "{what}: the answer's edges along the touch are {along_the_touch:?} and the \
                     seam's images {images:?}"
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "the seam's images are the one edge of the answer along it where the answer keeps the \
         bar, and nothing where it does not:\n{}",
        wrong.join("\n")
    );
}

/// A cube `4 × 4 × 2` (`z` in `0..2`) and a bar of radius one, two high, at
/// `(−1, 2, 1)`. The bar's own two seam corners are `(0, 2, 1)` and `(0, 2, 3)`
/// (`a_bar`'s doc comment), so its seam runs from `z = 1` to `z = 3`, but the
/// cube's wall at `x = 0` only stands up to `z = 2`: the touch is the line
/// `(0, 2, z)` for `z` in `[1, 2]` only, and the wall's own top edge crosses
/// the bar's skin at `(0, 2, 2)`, a corner the seam's own two ends do not name.
///
/// Every rule both ways round: never refused, and every fuse keeps the bar
/// whole wherever `a_bar_whose_seam_is_the_line_it_touches_a_wall_along_keeps_its_seam_in_history`
/// does (union, and the bar less the cube). There the answer's seam is cut in
/// two at `(0, 2, 2)` — `(0, 2, 1)`–`(0, 2, 2)` along the touch, and
/// `(0, 2, 2)`–`(0, 2, 3)` above it, where the wall is no longer there to touch.
///
/// **The seam's images are both halves** there, and nothing where the answer
/// keeps none of the bar's skin. Until Q1761 they were empty every time:
/// neither half's two ends are the seam's own two ends, and only a side
/// between those was read as running along the seam.
#[test]
fn a_bar_whose_seam_runs_past_where_a_wall_stops_touching_it() {
    let mut wrong: Vec<String> = Vec::new();
    for swapped in [false, true] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let cube = a_block(&mut model, [4.0, 4.0, 2.0]);
            let bar = a_bar(&mut model, 1.0, 2.0, [-1.0, 2.0, 1.0]);
            let seam = the_one_straight_edge_of(&model, bar);
            let [first, second] = if swapped { [bar, cube] } else { [cube, bar] };
            let what = format!("{rule:?}, bar first {swapped}");
            let answer = model
                .fuse(
                    &[first],
                    &[second],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what} refused: {why}"));
            let lowest_first = |one: &EdgeKey, other: &EdgeKey| {
                heights_of_the_ends_of(&model, *one)[0]
                    .total_cmp(&heights_of_the_ends_of(&model, *other)[0])
            };
            let mut images = answer
                .images_of_edge(seam)
                .expect("this model made the key")
                .to_vec();
            images.sort_by(lowest_first);
            let mut along_the_line: Vec<EdgeKey> = answer
                .solids()
                .iter()
                .flat_map(|&solid| edges_of(&model, solid))
                .filter(|&edge| runs_up_the_line(&model, edge, [0.0, 2.0]))
                .collect();
            along_the_line.sort_by(lowest_first);
            let heights: Vec<[f64; 2]> = along_the_line
                .iter()
                .map(|&edge| heights_of_the_ends_of(&model, edge))
                .collect();
            let keeps_the_bar = rule == Rule::Union || (rule == Rule::Difference && swapped);
            let wanted: &[[f64; 2]] = if keeps_the_bar {
                &[[1.0, 2.0], [2.0, 3.0]]
            } else {
                &[]
            };
            if heights != wanted || images != along_the_line {
                wrong.push(format!(
                    "{what}: the answer's edges along the line are {along_the_line:?}, from and to \
                     the heights {heights:?} (wanted {wanted:?}), and the seam's images are \
                     {images:?}"
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "the seam's images are its two halves either side of where the wall stops, where the \
         answer keeps the bar, and nothing where it does not:\n{}",
        wrong.join("\n")
    );
}

/// The heights of an upright edge's two ends, lower first.
fn heights_of_the_ends_of(model: &Model, edge: EdgeKey) -> [f64; 2] {
    let edge = model.edge(edge).expect("this model made the key");
    let [one, other] = [edge.start().point().z(), edge.end().point().z()];
    [one.min(other), one.max(other)]
}

/// A bar of radius one in the hole of a tube, radius seven less radius three,
/// touching the inner wall along `(3, 0, z)`, which is the bar's seam. United,
/// both skins keep the line, so the edge along it is parted into one per skin
/// and both stay: the bar's seam is the bar's edge, and not the tube's too.
#[test]
fn a_bar_whose_seam_touches_a_tubes_inner_wall_keeps_its_seam_in_its_own_lump() {
    for swapped in [false, true] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let outer = a_bar(&mut model, 7.0, 4.0, [0.0, 0.0, 0.0]);
        let through = a_bar(&mut model, 3.0, 6.0, [0.0, 0.0, -1.0]);
        let tube = model
            .fuse(
                &[outer],
                &[through],
                Rule::Difference,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a bar less a thinner bar through it is a tube")
            .solids()[0];
        let bar = a_bar(&mut model, 1.0, 4.0, [2.0, 0.0, 0.0]);
        let seam = the_one_straight_edge_of(&model, bar);
        let [first, second] = if swapped { [bar, tube] } else { [tube, bar] };
        let what = format!("bar first {swapped}");
        let answer = model
            .fuse(
                &[first],
                &[second],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} refused: {why}"));
        let lumps = answer.solids();
        assert_eq!(lumps.len(), 2, "{what}: the tube and the bar, apart");
        let the_bar = if volume_of(&model, lumps[0]) < volume_of(&model, lumps[1]) {
            lumps[0]
        } else {
            lumps[1]
        };
        let images = answer
            .images_of_edge(seam)
            .expect("this model made the key");
        let the_bars_edge_along_the_touch: Vec<EdgeKey> = edges_of(&model, the_bar)
            .into_iter()
            .filter(|&edge| runs_up_the_line(&model, edge, [3.0, 0.0]))
            .collect();
        assert_eq!(
            images, the_bars_edge_along_the_touch,
            "{what}: the seam's one image is the bar's own edge along the touch"
        );
    }
}

/// An L of two arms six long and two thick, four high, with a bar of radius one
/// in its inside corner, touching both walls: `x = 2` along `(2, 3, z)` and
/// `y = 2` along `(3, 2, z)`. The fillet a part's inside corner is.
#[test]
fn a_bar_in_an_inside_corner_answers_every_rule() {
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
fn two_bars_side_by_side_answer_every_rule() {
    every_rule_of(
        |model| {
            let thick = a_bar(model, 2.0, 4.0, [0.0, 0.0, 0.0]);
            let thin = a_bar(model, 1.0, 2.0, [0.0, 3.0, 1.0]);
            [(thick, 16.0 * PI), (thin, 2.0 * PI)]
        },
        "two bars side by side",
    );
}

/// Every rule, both ways round, of a pair whose insides are apart: both solids
/// as they stand for the union, no solid for the intersection, and the first
/// solid for each difference.
fn every_rule_of(build: fn(&mut Model) -> [(SolidKey, f64); 2], pair: &str) {
    for swapped in [false, true] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let [one, other] = build(&mut model);
            let [(first, first_volume), (second, second_volume)] =
                if swapped { [other, one] } else { [one, other] };
            let what = format!("{pair}, {rule:?}, swapped {swapped}");
            let mut log = ProvenanceLog::new();
            let answer = model
                .fuse(&[first], &[second], rule, &Policy::certified(), &mut log)
                .unwrap_or_else(|why| panic!("{what} refused: {why}"));
            assert!(
                log.records().is_empty(),
                "{what}: every decision is proved, got {:?}",
                log.records()
            );
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what} wrote a model that does not validate: {why}"));
            if rule == Rule::Union {
                let mut found: Vec<f64> = answer
                    .solids()
                    .iter()
                    .map(|&solid| volume_of(&model, solid))
                    .collect();
                found.sort_by(f64::total_cmp);
                let mut expected = [first_volume, second_volume];
                expected.sort_by(f64::total_cmp);
                assert_eq!(
                    found.len(),
                    2,
                    "{what}: two solids touching along a line are two lumps, got {found:?}"
                );
                for (found, expected) in found.iter().zip(expected) {
                    assert!(
                        *found <= expected && (expected - found) / expected < HOW_FAR_A_MESH_MAY_BE,
                        "{what}: measured {found}, and the solid is {expected}"
                    );
                }
                continue;
            }
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

/// A bar's seam: of its three edges, the two caps' rims close and the seam is
/// straight.
fn the_one_straight_edge_of(model: &Model, bar: SolidKey) -> EdgeKey {
    let straight: Vec<EdgeKey> = edges_of(model, bar)
        .into_iter()
        .filter(|&edge| {
            !model
                .edge(edge)
                .expect("this model made the key")
                .is_closed()
                .expect("a bar's edges are straight or round a circle")
        })
        .collect();
    assert_eq!(straight.len(), 1, "a bar has one seam, got {straight:?}");
    straight[0]
}

/// Every edge of a solid, once each.
fn edges_of(model: &Model, solid: SolidKey) -> Vec<EdgeKey> {
    let mut edges: Vec<EdgeKey> = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge().key();
                if !edges.contains(&edge) {
                    edges.push(edge);
                }
            }
        }
    }
    edges
}

/// Whether an edge runs along the upright line through `(x, y)`: not round a
/// cap's rim, which starts and ends on the line, and with both ends on it. The
/// ends are sums of whole numbers, so they come out exactly.
fn runs_up_the_line(model: &Model, edge: EdgeKey, [x, y]: [f64; 2]) -> bool {
    let edge = model.edge(edge).expect("this model made the key");
    let closes = edge
        .is_closed()
        .expect("the edges here are straight or round a circle");
    !closes
        && [edge.start().point(), edge.end().point()]
            .iter()
            .all(|end| end.x() == x && end.y() == y)
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
