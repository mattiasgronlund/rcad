//! A round bar `add_round_bar(r, 2r)` and a block over `x ∈ [0.6r, 2r]`,
//! `y ∈ [−2r, 0.8r]`, `z ∈ [−r, 3r]`, whose corner edge `(0.6r, 0.8r)` stands
//! on the bar's wall, under a tolerant policy (rows Q2028, Q2067, Q2068,
//! Q2074, Q2079, Q2080, Q2081).
//!
//! **Q2074: with the corner edge exactly on the wall, every fuse was refused
//! `ResultIsNotManifold { count: 1 }`**, at radius `5`, whose corner is
//! `(3, 4)`, and at the radii round it. The plane `x = 0.6r` crosses the pipe
//! in a ruling that is the block's corner edge, where `x = 0.6r` meets
//! `y = 0.8r`. Assembly keyed an edge partly on whether both its surfaces are
//! planes, so the one segment between the corner's two ends came out as two
//! edges, one walked only by the pipe's face and one only by the face
//! `y = 0.8r`.
//!
//! **Q2067: at radius `17.92` and its power-of-two images, the union and the
//! block less the bar wrote a wire of two sides.** A sliver of the face
//! `x = 0.6r` lies between the pipe's ruling and the corner edge, and the caps
//! join their ends, so it is a ring of two straight sides that
//! `leave_out_each_ring_of_two_straight_sides` in `fuse/assemble.rs` left in,
//! since one side lies on the pipe; `Model::validate` reads the ruling as
//! straight and refused the answer `WireTooShort { count: 2 }`.
//!
//! Both are answered by asking whether an edge's two surfaces cross in a
//! straight line, which a pipe's ruling does, rather than whether both are
//! planes (Q2079).
//!
//! **Q2081: the union, bar first, did not come back** at radius `24.32` and
//! working size `6.4`, nor at its power-of-two image, radius `0.38` and size
//! `0.1`, where every other fuse of the family takes milliseconds. The block's
//! faces `y = 0.8r` and `x = 0.6r` cut the pipe in two rulings a hair apart,
//! since the corner `(0.6r, 0.8r)`, rounded, is a hair outside the bar, and
//! each cap has an arc between their feet two roundings long. Its two ends
//! are stored with one `y` though the circle runs 37 degrees off `x` there.
//! The first wrong step was facing that arc:
//! `Edge::the_way_across_the_chord_into_the_arc` in `fuse/assemble.rs` read
//! its side off the stored chord, and on the bottom cap faced it towards the
//! bar's axis, so its picker held the other 286 degrees of the circle. The
//! volume search of `nest_the_shells` then meshed that cap the long way round,
//! every rung of `the_coarsest_mesh_whose_sides_do_not_cross` in `validate.rs`
//! was refused `BoundaryCrossesItself`, and the ladder walked down towards
//! half the arc's length, each rung costing about twice the last. The arc is
//! now faced by which way along its chord it sets off, as for any arc of a
//! closed curve whose ends' rounding chose its side.
//!
//! **What the union came back as then was a refusal, and a gap**: the answer
//! exists. The mesher found two corners of a cap at one stored point,
//! `(0.6r, 0.8r)` rounded: the foot of the ruling where `x = 0.6r` cuts the
//! pipe, and the foot of the block's corner edge, which in exact arithmetic is
//! `1.78e−15` from it, outside the bar.
//! Q2079's change, which answers Q2074 and Q2067, left both unions and the
//! bar less the block refused the same way (Q2106). Since Q2238 the mesh cuts
//! a cap's loop at a place it names twice (`what_the_triangulator_can_cut` in
//! `mesh.rs`), and every fuse at radius `24.32` answers with its volume.
//!
//! **Q2080: at working size `1`, every fuse at radii `8`, `12.8`, `16` and
//! `25.6` was refused `Undecidable` at "order along a crossing".** On each cap
//! the feet of the two rulings a hair apart are stored in boxes that overlap,
//! and the sweep that orders them round the rim was undecided and filed as an
//! area, `2.13e−14` at radius `8`, which `Policy::tolerant(1)` refuses, while
//! at working sizes `10` to `100` the same fuses get past it. Two boxes that
//! overlap may hold one point, so that order is now filed as a length: how far
//! apart a point of one box may be from a point of the other, which bounds
//! what a merge of the two moves a point by: `1.26e−15` at radius `8`, where
//! the wider box's side is `1.78e−15`. That answers every fuse at radii `8`
//! and `16`. At `12.8` and `25.6` the order is decided, and no fuse logs one,
//! so they get past it without that change: put back to file the sweep as an
//! area, the pins at those two radii still pass. Both
//! intersections and both differences answered with their volumes, and the
//! union in either order was refused `CannotMeshAFace` with `CornersCoincide`,
//! not on a cap but on the piece of the bar's wall between the two rulings,
//! one rounding of angle wide, whose loop named two places twice (Q2145's
//! trace). Each cap's arc between the two rulings' feet is one rounding long,
//! and the one point the walk put inside it was stored at one of its ends.
//! Since Q2352 `BuildingAMesh::walk` in `mesh.rs` drops such a point, and
//! every fuse at both radii answers with its volume.

#![forbid(unsafe_code)]

use std::sync::mpsc;
use std::time::Duration;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Measures, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// Q2074's shapes, as `(radius, working size)`: radius `5`, whose corner
/// `(3, 4)` is exactly on the wall, at four working sizes, and the radii round
/// it.
const THE_CORNER_EDGE_ON_THE_WALL: [(f64, f64); 11] = [
    (5.0, 1.0),
    (5.0, 10.0),
    (5.0, 25.0),
    (5.0, 100.0),
    (0.5, 10.0),
    (1.0, 10.0),
    (2.0, 10.0),
    (4.0, 10.0),
    (10.0, 10.0),
    (16.0, 10.0),
    (16.0, 100.0),
];

/// Q2067's shapes: radius `17.92` and its power-of-two images `0.28`, `1.12`
/// and `4.48`.
const A_SLIVER_BESIDE_THE_CORNER_EDGE: [(f64, f64); 7] = [
    (17.92, 6.4),
    (17.92, 9.6),
    (0.28, 0.1),
    (0.28, 0.5),
    (0.28, 0.9),
    (1.12, 6.4),
    (4.48, 6.4),
];

#[test]
fn every_fuse_with_the_corner_edge_on_the_wall_answers_with_its_volume() {
    let wrong = what_each_fuse_gets_wrong(&THE_CORNER_EDGE_ON_THE_WALL);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn every_fuse_at_radius_17_92_and_its_images_answers_with_its_volume() {
    let wrong = what_each_fuse_gets_wrong(&A_SLIVER_BESIDE_THE_CORNER_EDGE);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Q2080's shapes: radii `8` and `16`, one an image of the other under a scale
/// by two, at working size `1`.
const THE_RULINGS_FEET_A_HAIR_APART_AT_WORKING_SIZE_1: [(f64, f64); 2] = [(8.0, 1.0), (16.0, 1.0)];

#[test]
fn every_fuse_with_the_rulings_feet_a_hair_apart_at_working_size_1_answers_with_its_volume() {
    let wrong = what_each_fuse_gets_wrong(&THE_RULINGS_FEET_A_HAIR_APART_AT_WORKING_SIZE_1);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// At radius `8` and working size `1` every fuse files some order along a
/// crossing as how far apart a point of one of its two boxes may be from a
/// point of the other, rounded up by a few ulps, a length: `1.26e−15` at the
/// widest, where the wider of those two boxes is `1.78e−15` wide. Filed as
/// the wider box's side, the merge would have been read as wider than any
/// two points of the boxes can be apart.
#[test]
fn the_rulings_feet_at_radius_8_are_ordered_by_how_far_apart_their_boxes_let_them_be() {
    let (radius, size) = (8.0, 1.0);
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for bar_first in [true, false] {
            let (mut model, bar, block) = the_bar_and_the_block(radius, size);
            let (arguments, tools) = if bar_first {
                ([bar], [block])
            } else {
                ([block], [bar])
            };
            let mut log = ProvenanceLog::new();
            model
                .fuse(&arguments, &tools, rule, &Policy::tolerant(size), &mut log)
                .unwrap_or_else(|why| panic!("{rule:?} {bar_first}: {why:?}"));
            let widest = log
                .records()
                .iter()
                .filter(|record| record.predicate() == "order along a crossing")
                .filter_map(|record| {
                    let [one, other] = record.inputs() else {
                        return None;
                    };
                    let apart = how_far_apart_the_two_may_be(*one, *other);
                    let side = one.uncertainty().max(other.uncertainty());
                    let enclosure = record.enclosure();
                    (record.measures() == Measures::Length
                        && apart <= enclosure
                        && enclosure <= apart * (1.0 + 8.0 * f64::EPSILON)
                        && enclosure < side)
                        .then_some((enclosure, side))
                })
                .max_by(|a, b| a.0.total_cmp(&b.0));
            let Some((enclosure, side)) = widest else {
                panic!("{rule:?} {bar_first}: {:#?}", log.records());
            };
            println!("{rule:?} bar first {bar_first}: filed {enclosure:e}, wider side {side:e}");
        }
    }
}

/// How far apart a point of the first box may be from a point of the second.
fn how_far_apart_the_two_may_be(one: PredicatePoint, other: PredicatePoint) -> f64 {
    let (mine, theirs) = (one.enclosure(), other.enclosure());
    (0..3)
        .map(|axis| {
            let furthest =
                (mine[axis].hi() - theirs[axis].lo()).max(theirs[axis].hi() - mine[axis].lo());
            furthest * furthest
        })
        .sum::<f64>()
        .sqrt()
}

/// The rest of Q2080's shapes: radii `12.8` and `25.6`, one an image of the
/// other under a scale by two, at working size `1`. Their fuses get past the
/// order of the rulings' feet, whichever way it is filed. The two rulings are
/// one rounding of angle apart, and the two unions then reach the piece of the
/// wall between them.
const THE_RULINGS_ONE_ROUNDING_APART_AT_WORKING_SIZE_1: [(f64, f64); 2] =
    [(12.8, 1.0), (25.6, 1.0)];

/// Every fuse at radii `12.8` and `25.6` answers with its volume.
///
/// **Both unions were refused by the mesher until Q2352**, on a piece of the
/// bar's wall one rounding of angle wide (Q2145's trace, radius `12.8`, bar
/// first): between the ruling where `y = 0.8r` crosses the wall, `B`, stored
/// at `(7.679999999999999, 10.240000000000002)`, and the ruling where
/// `x = 0.6r` does, stored at the block's corner `K`,
/// `(7.68, 10.240000000000002)`. Each cap's arc from `B` to `K` is one
/// rounding long, so the one point the walk put inside it was stored at an
/// end: at `K` on the bottom cap and at `B` on the top. The loop named two
/// places twice; cut there, the lobe left was one rounding wide and refused
/// `ASideCutsOffOneCornerOfACurvedPatch`, so the piece was refused
/// `CornersCoincide` for the whole loop. At radius `24.32` the same piece is
/// two roundings wide and the top arc's point falls between its ends. The
/// pieces of the wall stay apart because `Assembly::a_ring_goes_right_round`
/// cannot count a ring with an arc one rounding long. `BuildingAMesh::walk` in
/// `mesh.rs` now drops a point of the walk stored at one of its edge's ends.
#[test]
fn every_fuse_at_radii_12_8_and_25_6_answers_with_its_volume() {
    let wrong = what_each_fuse_gets_wrong(&THE_RULINGS_ONE_ROUNDING_APART_AT_WORKING_SIZE_1);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn the_union_at_radius_24_32_comes_back_within_ten_seconds() {
    assert!(
        the_union_within_ten_seconds(24.32, 6.4).is_some(),
        "the union of the bar and the block cornered on its wall did not come back in 10 s"
    );
}

/// The exact image of the case above under a scale by `2⁻⁶`, which ran past
/// 290 s before Q2081.
#[test]
fn the_union_at_radius_0_38_comes_back_within_ten_seconds() {
    assert!(
        the_union_within_ten_seconds(0.38, 0.1).is_some(),
        "the union of the bar and the block cornered on its wall did not come back in 10 s"
    );
}

/// Q2081's shapes, radius `24.32` at working size `6.4` and its image under a
/// scale by `2⁻⁶`: every fuse, though a cap holds two corners stored at one
/// point, as the module's doc describes.
#[test]
fn every_fuse_at_radius_24_32_and_its_image_answers_with_its_volume() {
    let wrong = what_each_fuse_gets_wrong(&[(24.32, 6.4), (0.38, 0.1)]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Each of the six fuses of each shape, every rule both ways round, that does
/// not answer one solid which validates and measures within its error of
/// [`the_volume_it_should_have`], with what it did instead.
fn what_each_fuse_gets_wrong(shapes: &[(f64, f64)]) -> Vec<String> {
    let mut wrong = Vec::new();
    for &(radius, size) in shapes {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for bar_first in [true, false] {
                if let Err(why) = the_fuse_answers_with_its_volume(radius, size, rule, bar_first) {
                    wrong.push(format!(
                        "radius {radius}, working size {size}, {rule:?}, bar first \
                         {bar_first}: {why}"
                    ));
                }
            }
        }
    }
    wrong
}

/// What went wrong with one fuse, if anything did.
fn the_fuse_answers_with_its_volume(
    radius: f64,
    size: f64,
    rule: Rule,
    bar_first: bool,
) -> Result<(), String> {
    let policy = Policy::tolerant(size);
    let (mut model, bar, block) = the_bar_and_the_block(radius, size);
    let (arguments, tools) = if bar_first {
        ([bar], [block])
    } else {
        ([block], [bar])
    };
    let fused = model
        .fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("refused {why:?}"))?;
    let &[solid] = fused.solids() else {
        return Err(format!("{} solids", fused.solids().len()));
    };
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("does not validate: {why:?}"))?;
    let measured = model
        .solid(solid)
        .expect("the fuse made the key")
        .measure(
            Deflection::of(radius / 10.0).expect("a length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("does not measure: {why:?}"))?;
    let closed_form = the_volume_it_should_have(radius, rule, bar_first);
    if (measured.volume() - closed_form).abs() > measured.volume_error() {
        return Err(format!(
            "measured {} ± {}, and the closed form is {closed_form}",
            measured.volume(),
            measured.volume_error()
        ));
    }
    Ok(())
}

/// The volume of the answer. The bar holds `2πr³` and the block
/// `1.4r · 2.8r · 4r`. Since the corner `(0.6r, 0.8r)` is on the wall, the
/// part of the disk with `x ≥ 0.6r` has `y ≤ 0.8r` throughout, so what the two
/// share is that circular segment, `r²(acos 0.6 − 0.6 · 0.8)`, over the bar's
/// height `2r`.
fn the_volume_it_should_have(radius: f64, rule: Rule, bar_first: bool) -> f64 {
    let cubed = radius.powi(3);
    let bar = 2.0 * core::f64::consts::PI * cubed;
    let block = 1.4 * 2.8 * 4.0 * cubed;
    let both = 2.0 * (0.6_f64.acos() - 0.6 * 0.8) * cubed;
    match (rule, bar_first) {
        (Rule::Union, _) => bar + block - both,
        (Rule::Intersection, _) => both,
        (Rule::Difference, true) => bar - both,
        (Rule::Difference, false) => block - both,
        (other, _) => panic!("no closed form for {other:?}"),
    }
}

/// The union, bar first, or `None` when it has not come back within ten
/// seconds. Every other fuse of the family takes milliseconds, so ten seconds
/// is room and not a race.
fn the_union_within_ten_seconds(radius: f64, size: f64) -> Option<rcad_topo::Result<usize>> {
    let (send, recv) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = send.send(the_union_of_the_bar_and_the_block(radius, size));
    });
    recv.recv_timeout(Duration::from_secs(10)).ok()
}

/// What the union, bar first, comes back as: how many solids it holds, or the
/// refusal.
fn the_union_of_the_bar_and_the_block(radius: f64, size: f64) -> rcad_topo::Result<usize> {
    the_fuse_of_the_bar_and_the_block(radius, size, Rule::Union, true)
}

/// What one fuse comes back as: how many solids it holds, or the refusal.
fn the_fuse_of_the_bar_and_the_block(
    radius: f64,
    size: f64,
    rule: Rule,
    bar_first: bool,
) -> rcad_topo::Result<usize> {
    let (mut model, bar, block) = the_bar_and_the_block(radius, size);
    let (arguments, tools) = if bar_first {
        ([bar], [block])
    } else {
        ([block], [bar])
    };
    model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::tolerant(size),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().len())
}

/// A model at working size `size` holding the bar of radius `radius` and the
/// block cornered on its wall, and the two.
fn the_bar_and_the_block(radius: f64, size: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(size).expect("a working size"));
    let bar = model
        .add_round_bar(radius, 2.0 * radius, &Isometry::identity())
        .expect("a bar");
    let notch = ProfileBuilder::new(vec![
        [0.6 * radius, -2.0 * radius],
        [2.0 * radius, -2.0 * radius],
        [2.0 * radius, 0.8 * radius],
        [0.6 * radius, 0.8 * radius],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    let block = model
        .add_extrusion(
            &notch,
            4.0 * radius,
            &Isometry::translation(Vector::new(0.0, 0.0, -radius)).expect("a shift"),
        )
        .expect("a block");
    (model, bar, block)
}
