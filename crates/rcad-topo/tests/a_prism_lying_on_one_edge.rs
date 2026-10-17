//! A prism lying along one of its edges on a block's top face, the edge inside
//! the face and the insides apart.
//!
//! The block is eight wide, three deep and ten long, `−4 ≤ x ≤ 4`,
//! `−3 ≤ y ≤ 0`, `0 ≤ z ≤ 10`, volume 240. Its top face is `y = 0`: an
//! extrusion runs along `z`, and a quarter turn to lay one on its side is not
//! exact in `f64`, so the block is stood on its side instead. The prism is the
//! triangle `(0, 0)`, `(3, 2)`, `(−1, 2)`, area 4, its walls leaning away from
//! the corner at the origin on either side, so it touches the top face along
//! that one edge and nowhere else. It is laid two ways: from `z = 2` to `z = 8`
//! with its edge at `x = 0`, volume 24, and from `z = 2.5` to `z = 7.25` with
//! its edge at `x = −1`, volume 19. Either way the edge is strictly inside the
//! top face, so the line is on no edge of the block.
//!
//! **The union used to be refused `ResultIsNotManifold { count: 4 }`** (queue
//! row Q803). Along the edge there were four darts: the top face's two pieces
//! walking it opposite ways, and the prism's two faces either side of its own
//! edge, two input faces. `fuse/assemble.rs` parted only a line whose two
//! pairs are each pieces of one input face — a bar lying along a wall (Q760) —
//! and now parts off the top face's pair too, so the union is the two solids as
//! they stood.
//!
//! **Two shapes that also reach four darts that way answer one solid touching
//! itself along the line** (queue row Q1901): a block less a prism lying
//! inside it along its edge, and a block united with a solid lying on it along
//! one edge and reaching down into it somewhere else. Both were refused
//! `ResultIsNotManifold { count: 4 }` from Q803 on, which parted the line only
//! where the prism's walls rested on the outer side of the top and nothing
//! else joined the two sides into one shell. Parted, each has two faces along
//! every edge, validates, and fuses on with a cube far from it
//! (`docs/decisions.md` §39.4). The parting asks no corner of either pair
//! since, so the corners the paragraphs below say it read are no longer read.
//!
//! **A solid whose upright wall rests along the edge and turns back below the
//! top past the block's end answers too** (queue row Q1117, which guessed it
//! refused): the parting asked every corner of a piece of the wall, and the
//! top's plane cuts the wall into pieces each on one side of it.
//!
//! **Moved elsewhere along `x`, the edge need not lie on the top at all**
//! (queue row Q1114). The edge is where the prism's two walls meet, and moving
//! the prism rounds the offset of each wall's plane. At `x = 0` and `x = −1`
//! the edge lies on the top exactly. At `x = −1.5` it lies `3.1e-17` inside
//! the block, so the two overlap in a sliver, and every exact answer then
//! needs a side shorter than the gap between two doubles, whose two ends round
//! to one corner. Since queue row Q1353 the union and both differences answer
//! as if the edge lay on the top, and since queue row Q1442 the intersection
//! answers no solid (`docs/decisions.md` §417). At `x = −2.5`,
//! `−1.25` and `1.5` it stands a rounding above the top, the two share no
//! point, and every answer is the two as they stand. That was a gap, refused
//! for want of a point inside a piece of a face, until queue row Q1279 let a
//! plane place a piece with corners on it (`docs/decisions.md` §365.8).
//!
//! **Turned, the edge can lie on the top exactly and the union still be
//! refused** (queue row Q1281). A wedge, the triangle `(0, 0)`, `(1, √3)`,
//! `(0, √3)`, its upright wall rising from its edge and its other wall leaning
//! with a normal whose `y` is exactly minus a half, lies on the block's top, the two
//! turned about `x` together. Turning rounds every plane but keeps the edge on
//! the top. The union was refused `ResultIsNotManifold { count: 4 }` at 14 of
//! 24 turns: the parting asked a corner at an end of the edge which side of the
//! top it was on by the point it is written down as, the middle of a box, and
//! on a tilted top that can sit a rounding inside. A corner of the top's own
//! pieces was no longer asked after that.
//!
//! **Turned, the resting prism has its ends a rounding above the top or a
//! rounding inside it** (queue row Q1582). With the triangle above in place of
//! the wedge, both ends are above at 17 of 60 turns and every rule both ways
//! round answers there. Where an end is inside, every exact answer needs the
//! sliver between the prism and the top, under `1.8e-15` deep, and the fuse
//! refuses most rules `DynamicRangeTooWide` or `HoleWindsWithTheOuterWire`:
//! limits under `Policy::certified()`. One was a gap: at the turn `0.625` the
//! parting read the prism's wall's far corner by its written point, a rounding
//! inside the top, and then asked where the corner's three planes meet
//! (`docs/decisions.md` §438).

use std::collections::{BTreeMap, BTreeSet};

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Decision, Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
use rcad_topo::{Error, Fused, Invalid, Measurement, Model, ProfileBuilder, Rule, SolidKey};

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The block's volume: eight by three by ten.
const BLOCK: f64 = 240.0;

/// The prism's triangle, its corner at the origin, both walls leaning up.
const RESTING: [[f64; 2]; 3] = [[0.0, 0.0], [3.0, 2.0], [-1.0, 2.0]];

/// The prism's triangle turned over, its corner at the origin and both walls
/// leaning down, so moved into the block it lies inside with its edge on the
/// top.
const INSIDE: [[f64; 2]; 3] = [[0.0, 0.0], [-1.0, -2.0], [3.0, -2.0]];

/// Where each prism is moved to, and how long it is. The first two leave its
/// edge on the block's top, and the last three, six long, a rounding above it.
const LAID: [([f64; 3], f64); 5] = [
    ([0.0, 0.0, 2.0], 6.0),
    ([-1.0, 0.0, 2.5], 4.75),
    ([-2.5, 0.0, 2.0], 6.0),
    ([-1.25, 0.0, 2.0], 6.0),
    ([1.5, 0.0, 2.0], 6.0),
];

/// Where a prism six long is moved to for its walls' rounding to leave its
/// edge `3.1e-17` inside the block's top.
const A_ROUNDING_INTO_THE_TOP: [f64; 3] = [-1.5, 0.0, 2.0];

/// The block's outline, across `x` and down from `y = 0`.
const THE_BLOCKS_OUTLINE: [[f64; 2]; 4] = [[-4.0, -3.0], [4.0, -3.0], [4.0, 0.0], [-4.0, 0.0]];

/// `√3`, the double nearest it.
const ROOT_THREE: f64 = 1.732_050_807_568_877_2;

/// A wedge with its edge at the origin, an upright wall `x = 0` rising from
/// it, and a wall leaning at sixty degrees whose normal has a `y` of exactly
/// minus a half: its triangle is `(0, 0)`, `(1, √3)`, `(0, √3)`.
const WEDGE: [[f64; 2]; 3] = [[0.0, 0.0], [1.0, ROOT_THREE], [0.0, ROOT_THREE]];

/// How far the block and the prism on it are turned about `x`, in fortieths
/// of a radian: up to one and a half.
const TURNS: std::ops::RangeInclusive<u8> = 1..=60;

/// The regression: every rule both ways round, checked against volumes. The
/// union is the block and the prism as they stood, two lumps; the
/// intersection is no solid; each difference is the first solid whole.
///
/// **With the edge a rounding above the top**, the last three of [`LAID`], the
/// two share no point (the test below), so the answers are the same. Every
/// rule both ways round was refused there for want of a point inside a piece
/// of a face until queue row Q1279 (`docs/decisions.md` §365.8). Under the
/// union at `x = −2.5`, either way round, the piece is the sliver of the
/// block's top from `z = 0` to `z = 2` between where the two walls' planes
/// meet it, outside the prism, with two of its corners on the plane of the
/// prism's end and the others in front of it; `by_a_plane` in `fuse/select.rs`
/// asked for every corner strictly in front.
#[test]
fn a_prism_lying_on_its_edge_on_a_blocks_top_answers_every_rule_both_ways_round() {
    let mut wrong = Vec::new();
    for (at, length) in LAID {
        let prism = 4.0 * length;
        for swapped in [false, true] {
            for rule in EVERY_RULE {
                let what =
                    format!("prism at {at:?} {length} long, {rule:?}, prism first {swapped}");
                let (model, answer) = fuse(swapped, rule, |model| {
                    [a_block(model), a_prism(model, RESTING, at, length)]
                });
                let should: &[f64] = match (rule, swapped) {
                    (Rule::Union, _) => &[prism, BLOCK],
                    (Rule::Intersection, _) => &[],
                    (Rule::Difference, false) => &[BLOCK],
                    (Rule::Difference, true) => &[prism],
                    _ => unreachable!("EVERY_RULE holds only these three"),
                };
                if let Some(why) = differs(&model, answer, should) {
                    wrong.push(format!("{what}: {why}"));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// **Where the edge lies is the walls' rounding.** Each end of the edge is the
/// meet of the two walls and that end's plane, and asked exactly of those
/// planes it lies on the block's top at `x = 0` and `x = −1`, strictly above
/// it at `x = −2.5`, `−1.25` and `1.5`, clear of the block, and strictly below
/// it at `x = −1.5`, inside the block.
#[test]
fn the_prisms_edge_is_on_the_blocks_top_or_above_it_wherever_every_fuse_answers() {
    let sides =
        LAID.map(|(at, length)| sides_of_the_prisms_edge_against_the_blocks_top(at, length));
    assert_eq!(
        sides,
        [
            [Decision::Zero; 2],
            [Decision::Zero; 2],
            [Decision::Pos; 2],
            [Decision::Pos; 2],
            [Decision::Pos; 2],
        ]
    );
    assert_eq!(
        sides_of_the_prisms_edge_against_the_blocks_top(A_ROUNDING_INTO_THE_TOP, 6.0),
        [Decision::Neg; 2]
    );
}

/// **At `x = −1.5` every exact answer has a side shorter than the gap between
/// two doubles**, so its two ends round to one corner. The edge is inside the
/// block, so the two overlap in a sliver from `z = 2` to `z = 8`, and at each
/// end every answer has a side along the block's top from where one wall meets
/// it to where the other does: where the top stops and a wall starts in the
/// union and in each difference, and across the sliver's own end in the
/// intersection. One wall meets the top at `x = −1.5` exactly and the other
/// strictly between that and the next double down, `−1.5000000000000002`, so
/// the side is under the `2.2e-16` between doubles at `1.5`. With the block
/// reaching 10 from the origin that is a spread above `4e16`, against the
/// `1e11` a model supports (`docs/decisions.md` §3.5).
#[test]
fn every_answer_with_the_edge_a_rounding_into_the_top_needs_a_side_shorter_than_one_double() {
    let at_one_and_a_half = where_each_wall_meets_the_blocks_top_against(-1.5);
    assert!(
        at_one_and_a_half.contains(&[Decision::Zero; 2])
            && at_one_and_a_half.contains(&[Decision::Neg; 2]),
        "{at_one_and_a_half:?}"
    );
    let the_next_double_down = -f64::from_bits(1.5_f64.to_bits() + 1);
    assert_eq!(the_next_double_down, -1.500_000_000_000_000_2);
    assert_eq!(
        where_each_wall_meets_the_blocks_top_against(the_next_double_down),
        [[Decision::Pos; 2]; 2]
    );
}

/// **Every rule both ways round answers as if the edge lay on the top**, under
/// `Policy::certified()`.
///
/// Every rule both ways round was refused `Undecidable` for want of *a point
/// inside a piece of a face* until queue row Q1353. With the block first the
/// piece was the sliver of the block's top from `z = 2` to `z = 8` between
/// where the two walls' planes meet it, inside the prism; with the prism first
/// it was the part of the prism's end at `z = 2` below the block's top, inside
/// the block. `by_a_plane` now places a piece inside a solid when every plane
/// of the solid has every corner of the piece on or behind it and one strictly
/// behind (`docs/decisions.md` §417), and both are such pieces.
///
/// The exact answers then need the side the test above measures, and its two
/// ends round to the one corner `(−1.5, 0, 2)` (and `(−1.5, 0, 8)`).
/// `fuse/assemble.rs` leaves out a side whose ends are one corner and a wire
/// that then walks out and back along one edge (`docs/decisions.md` §365.7), so
/// the sliver cut out of the block's top closes up, and what is written is the
/// union of a prism resting on the top: the two as they stand, which validate
/// and measure 240 and 24, as in [`LAID`]. Each difference is the first solid
/// whole.
///
/// **The intersection is the sliver itself, and it is no solid.** Every wire
/// of every piece of it walks out and back along the prism's edge, from
/// `(−1.5, 0, 2)` to `(−1.5, 0, 8)`, and is left out. It was refused
/// `ResultIsNotManifold { count: 0 }` until queue row Q1442: that edge was
/// still made, with no face along it. It is now left out with the wires that
/// walked it, and its two corners with it.
#[test]
fn the_prism_with_its_edge_a_rounding_into_the_blocks_top_answers_as_if_on_it() {
    let mut wrong = Vec::new();
    for swapped in [false, true] {
        for rule in EVERY_RULE {
            let what = format!("{rule:?}, prism first {swapped}");
            let (model, answer) = fuse(swapped, rule, |model| {
                [
                    a_block(model),
                    a_prism(model, RESTING, A_ROUNDING_INTO_THE_TOP, 6.0),
                ]
            });
            let outcome = match (rule, swapped) {
                (Rule::Union, _) => differs(&model, answer, &[24.0, BLOCK]),
                (Rule::Difference, false) => differs(&model, answer, &[BLOCK]),
                (Rule::Difference, true) => differs(&model, answer, &[24.0]),
                (Rule::Intersection, _) => differs(&model, answer, &[]),
                _ => unreachable!("EVERY_RULE holds only these three"),
            };
            if let Some(why) = outcome {
                wrong.push(format!("{what}: {why}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// **A wedge lying on its edge on a block's top, both turned about `x`**: every
/// rule both ways round at every one of [`TURNS`], each solid of the answer
/// checked against the turned solid it is, as it stood before the fuse
/// ([`differs_from_as_they_stand`]). The edge lies on the turned top
/// exactly (the test below), so the answers are those of the unturned prism:
/// the union the two as they stand, the intersection no solid, and each
/// difference its first solid whole.
///
/// **The union was refused `ResultIsNotManifold { count: 4 }`, both ways
/// round, at 14 of the first 24 turns** (queue row Q1281), all [`TURNS`] were
/// then. Queue row Q1582 ran them on to 60, and the wedge answered at each.
/// The parting in `fuse/assemble.rs` then asked each corner of the wedge's two
/// walls which side of the top it was on, by the point the corner is written
/// down as: the middle of the box round where three planes meet. On a tilted
/// top that middle can be a rounding inside the top. At a turn of `0.075` the
/// wedge's corner at the end `z = 2`, an end of its edge, was written down as
/// `(0, −0.14985941454548474, 1.994377636224415)` and read as inside. A corner
/// of the top's own pieces lies on the top by how it was made, and was no
/// longer asked; since queue row Q1901 no corner is.
#[test]
fn a_wedge_lying_on_its_edge_on_a_turned_blocks_top_answers_every_rule_both_ways_round() {
    let mut wrong = Vec::new();
    for turn in TURNS.map(a_turn) {
        let [block, wedge] = {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            a_turned_block_and_wedge(&mut model, turn).map(|solid| AsItStands::of(&model, solid))
        };
        for swapped in [false, true] {
            for rule in EVERY_RULE {
                let what = format!("turned {turn}, {rule:?}, wedge first {swapped}");
                let (model, answer) =
                    fuse(swapped, rule, |model| a_turned_block_and_wedge(model, turn));
                let should: &[&AsItStands] = match (rule, swapped) {
                    (Rule::Union, _) => &[&wedge, &block],
                    (Rule::Intersection, _) => &[],
                    (Rule::Difference, false) => &[&block],
                    (Rule::Difference, true) => &[&wedge],
                    _ => unreachable!("EVERY_RULE holds only these three"),
                };
                if let Some(why) = differs_from_as_they_stand(&model, answer, should) {
                    wrong.push(format!("{what}: {why}"));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// **The turned wedge's edge lies on the turned top exactly**, at every one
/// of [`TURNS`]: each end of it, the meet of its two walls and that end's
/// plane, asked of the planes exactly.
///
/// The turn rounds every plane but keeps them meeting. The edge is where the
/// upright wall `x = 0` meets the leaning wall, and the leaning wall's normal
/// has a `y` of exactly minus a half, so turned, its `y` and `z` are exactly
/// minus half the top's, which a turn about `x` leaves with no `x`. Both walls
/// pass through the origin, as the top does, so on `x = 0` the leaning wall is
/// the top turned round.
#[test]
fn the_turned_wedges_edge_is_on_the_turned_blocks_top_at_every_turn() {
    assert_eq!(ROOT_THREE, 3.0_f64.sqrt());
    let mut off = Vec::new();
    for turn in TURNS.map(a_turn) {
        let planes = planes_round_the_edge_turned(&turned_by(turn), |model| {
            a_turned_block_and_wedge(model, turn)
        });
        let [x, y, z, offset] = planes.top;
        let half_the_top_turned_round = |wall: &[f64; 4]| {
            let [_, wall_y, wall_z, wall_offset] = *wall;
            x == 0.0
                && offset == 0.0
                && wall_offset == 0.0
                && [wall_y, wall_z] == [-y / 2.0, -z / 2.0]
        };
        let leaning = planes
            .walls
            .iter()
            .filter(|wall| half_the_top_turned_round(wall))
            .count();
        let sides = sides_of_the_edge(&planes);
        if leaning != 1 || sides != [Decision::Zero; 2] {
            off.push(format!(
                "turned {turn}: {leaning} walls half the top turned round, top {:?}, walls {:?}, \
                 ends {sides:?}",
                planes.top, planes.walls
            ));
        }
    }
    assert!(off.is_empty(), "{}", off.join("\n"));
}

/// **The resting prism and the block turned by `0.625` unite as they stand**,
/// both ways round, as the intersection and both differences at that turn
/// already answer (queue row Q1582).
///
/// There the edge's end at `z = 2` is a rounding inside the turned top and the
/// end at `z = 8` a rounding above it. The sliver at `z = 2` closes up as
/// `docs/decisions.md` §417.4 writes it, and the four darts along the edge from
/// where it crosses the top to that end are the top's two pieces and the
/// prism's two walls. The union was refused `ResultIsNotManifold { count: 4 }`:
/// the parting in `fuse/assemble.rs`, which asked until queue row Q1901 that
/// the walls rest on the outer side of the top, read the wall's corner at
/// `z = 8`, the meet of the two walls and that end, by the point it is written
/// down as, `(3.36e-16, −4.680778183523698, 6.487704956041742)`, a rounding
/// inside the top, where the three planes meet strictly above it.
#[test]
fn a_resting_prism_with_one_end_a_rounding_inside_a_turned_top_unites_as_they_stand() {
    let turn = a_turn(25);
    let planes = planes_round_the_edge_turned(&turned_by(turn), |model| {
        a_turned_block_and_resting_prism(model, turn)
    });
    assert!(
        sides_of_the_edge(&planes).contains(&Decision::Neg)
            && sides_of_the_edge(&planes).contains(&Decision::Pos),
        "{:?}",
        sides_of_the_edge(&planes)
    );
    let [block, prism] = {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        a_turned_block_and_resting_prism(&mut model, turn)
            .map(|solid| AsItStands::of(&model, solid))
    };
    let mut wrong = Vec::new();
    for swapped in [false, true] {
        let (model, answer) = fuse(swapped, Rule::Union, |model| {
            a_turned_block_and_resting_prism(model, turn)
        });
        if let Some(why) = differs_from_as_they_stand(&model, answer, &[&prism, &block]) {
            wrong.push(format!("prism first {swapped}: {why}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// **The resting prism and the block, both turned, at every one of
/// [`TURNS`]**: where both ends of the edge are above the turned top, every
/// rule both ways round is the two as they stand; elsewhere each fuse is the
/// two as they stand or is refused by name, `DynamicRangeTooWide` or
/// `HoleWindsWithTheOuterWire`, never anything else (queue row Q1582).
///
/// Turning rounds the prism's walls, and each end of its edge, the meet of the
/// two walls and that end's plane, lands a rounding above the top or a
/// rounding inside it: both above at 17 of the 60 turns, one inside at 21, and
/// both inside at 22. "As they stand" with an end inside is §417.4's answer,
/// the edge taken to lie on the top, and 12 fuses give it: the intersection
/// both ways round and the block less the prism at `0.225` and `1.175`, both
/// ends inside, and every rule both ways round at `0.625`, one end inside, the
/// union only since the gap below was fixed. Of the other 246, 213 are refused `DynamicRangeTooWide` and 33
/// `HoleWindsWithTheOuterWire`, the union both ways round and the block less
/// the prism at 11 turns. Every refusal comes where an end is inside, and every
/// exact answer there needs the sliver, which the test below shows is under
/// two roundings deep. Under `Policy::certified()` those are limits.
///
/// **One gap was fixed.** At the turn `0.625`, one end inside, the union was
/// refused `ResultIsNotManifold { count: 4 }` both ways round, for a corner of
/// the prism's wall read by its written point (the test above).
#[test]
fn a_resting_prism_on_a_turned_blocks_top_answers_as_they_stand_or_is_refused_by_name() {
    let mut how_many: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    let mut turns_with_an_end_inside: BTreeMap<&str, BTreeSet<u8>> = BTreeMap::new();
    let mut wrong = Vec::new();
    for fortieths in TURNS {
        let turn = a_turn(fortieths);
        let ends = sides_of_the_edge(&planes_round_the_edge_turned(&turned_by(turn), |model| {
            a_turned_block_and_resting_prism(model, turn)
        }));
        let inside = ends.iter().filter(|end| **end == Decision::Neg).count();
        let above = ends.iter().filter(|end| **end == Decision::Pos).count();
        let where_the_ends_are = match (inside, above) {
            (0, 2) => "both ends above",
            (1, 1) => "one end inside",
            (2, 0) => "both ends inside",
            _ => {
                wrong.push(format!("turned {turn}: ends {ends:?}"));
                continue;
            }
        };
        *how_many.entry((where_the_ends_are, "turns")).or_default() += 1;
        let [block, prism] = {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            a_turned_block_and_resting_prism(&mut model, turn)
                .map(|solid| AsItStands::of(&model, solid))
        };
        for swapped in [false, true] {
            for rule in EVERY_RULE {
                let what = format!("turned {turn}, ends {ends:?}, {rule:?}, prism first {swapped}");
                let (model, answer) = fuse(swapped, rule, |model| {
                    a_turned_block_and_resting_prism(model, turn)
                });
                let should: &[&AsItStands] = match (rule, swapped) {
                    (Rule::Union, _) => &[&prism, &block],
                    (Rule::Intersection, _) => &[],
                    (Rule::Difference, false) => &[&block],
                    (Rule::Difference, true) => &[&prism],
                    _ => unreachable!("EVERY_RULE holds only these three"),
                };
                let outcome = match answer {
                    Err(Error::Invalid(Invalid::DynamicRangeTooWide { .. })) => {
                        "DynamicRangeTooWide"
                    }
                    Err(Error::TheFuseWroteABrokenAnswer {
                        broken: Invalid::HoleWindsWithTheOuterWire,
                    }) => "HoleWindsWithTheOuterWire",
                    answer => match differs_from_as_they_stand(&model, answer, should) {
                        None => "as they stand",
                        Some(why) => {
                            wrong.push(format!("{what}: {why}"));
                            continue;
                        }
                    },
                };
                if inside == 0 && outcome != "as they stand" {
                    wrong.push(format!("{what}: {outcome}"));
                }
                if inside > 0 && outcome != "DynamicRangeTooWide" {
                    turns_with_an_end_inside
                        .entry(outcome)
                        .or_default()
                        .insert(fortieths);
                }
                *how_many.entry((where_the_ends_are, outcome)).or_default() += 1;
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert_eq!(
        how_many,
        BTreeMap::from([
            (("both ends above", "turns"), 17),
            (("both ends above", "as they stand"), 102),
            (("one end inside", "turns"), 21),
            (("one end inside", "as they stand"), 6),
            (("one end inside", "DynamicRangeTooWide"), 93),
            (("one end inside", "HoleWindsWithTheOuterWire"), 27),
            (("both ends inside", "turns"), 22),
            (("both ends inside", "as they stand"), 6),
            (("both ends inside", "DynamicRangeTooWide"), 120),
            (("both ends inside", "HoleWindsWithTheOuterWire"), 6),
        ])
    );
    assert_eq!(
        turns_with_an_end_inside,
        BTreeMap::from([
            ("as they stand", BTreeSet::from([9, 25, 47])),
            (
                "HoleWindsWithTheOuterWire",
                BTreeSet::from([3, 4, 14, 31, 32, 33, 39, 42, 43, 50, 57])
            ),
        ]),
        "in fortieths"
    );
}

/// **Every end of the resting prism's edge inside the turned top is under two
/// roundings deep**, two gaps between doubles where the end is: the meet of
/// the two walls and that end's plane, asked exactly, is above the top moved
/// in by two gaps, at each of the 65 ends inside over [`TURNS`]. All but one
/// are under one gap; the end at `z = 8` turned `1.475` is `9.1e-16` deep,
/// just past the `8.9e-16` there.
///
/// The end at `z = 2` turned has its larger coordinate between `√2` and 2,
/// where doubles are `2.2e-16` apart, and the end at `z = 8` between `4√2` and
/// 8, where they are `8.9e-16` apart: half the end plane's offset times
/// `f64::EPSILON`. The depth is a rounding of the walls' turned normals, and
/// both walls pass through the origin, so it grows with the end's distance
/// from it.
///
/// Every exact answer with an end inside has the sliver between the prism and
/// the top there, and a side across it along the top, from where one wall
/// meets the top to where the other does. The walls lean out 3 across for 2
/// up and 1 across for 2 up, so that side is twice the depth, under `3.6e-15`,
/// beside a block reaching 10 from the origin: a range above `2.7e15`, against
/// the `1e11` a model supports (`docs/decisions.md` §3.5). Under
/// `Policy::certified()` each refusal of the test above is a limit by that
/// range, as §417.5 rules for the turned cube's tip (`docs/decisions.md`
/// §438). It is not shown that no double lies in the sliver: at a small turn
/// the end's smaller coordinate has finer doubles than its larger one.
#[test]
fn every_end_of_the_resting_prisms_edge_inside_the_turned_top_is_under_two_roundings_deep() {
    assert_eq!(
        2.0 * f64::EPSILON / 2.0,
        2.0_f64.next_down().next_up() - 2.0_f64.next_down()
    );
    assert_eq!(
        8.0 * f64::EPSILON / 2.0,
        8.0_f64.next_down().next_up() - 8.0_f64.next_down()
    );
    let mut deeper_than = [Vec::new(), Vec::new()];
    let mut inside = 0;
    for turn in TURNS.map(a_turn) {
        let planes = planes_round_the_edge_turned(&turned_by(turn), |model| {
            a_turned_block_and_resting_prism(model, turn)
        });
        let [x, y, z, offset] = planes.top;
        for (end, side) in planes.ends.iter().zip(sides_of_the_edge(&planes)) {
            if side != Decision::Neg {
                continue;
            }
            inside += 1;
            let a_gap_where_the_end_is = end[3].abs() / 2.0 * f64::EPSILON;
            for (gaps, deeper) in [1.0, 2.0].into_iter().zip(&mut deeper_than) {
                let the_top_moved_in = [x, y, z, offset + gaps * a_gap_where_the_end_is];
                let against_the_top_moved_in = side_of_a_plane_where_three_planes_meet(
                    [planes.walls[0], planes.walls[1], *end],
                    the_top_moved_in,
                )
                .expect("two walls and an end meet in a point");
                if against_the_top_moved_in != Decision::Pos {
                    deeper.push(format!("turned {turn}, end at {}", end[3].abs()));
                }
            }
        }
    }
    assert_eq!(inside, 65, "21 turns with one end inside and 22 with two");
    assert_eq!(
        deeper_than,
        [vec!["turned 1.475, end at 8".to_owned()], Vec::new()],
        "deeper than one gap, and than two"
    );
}

/// The prism turned over and moved into the block, its edge on the top face
/// from below: the triangle `(0, 0)`, `(−1, −2)`, `(3, −2)`, from `z = 2` to
/// `z = 8`. The union is the block, the intersection the prism, the prism
/// less the block nothing, and the block less the prism 216, the block with a
/// prism-shaped hollow whose edge touches the top face.
///
/// **The block less the prism was refused `ResultIsNotManifold { count: 4 }`**
/// until queue row Q1901. The four darts along the edge are the same two pairs
/// as on top, but the prism's walls, turned round, lie on the inner side of
/// the top face, and the parting asked that they lie on the outer side. A
/// solid touching itself along a line is an answer (`docs/decisions.md`
/// §39.4): parted, it is one solid of two shells touching along the line (the
/// test below).
#[test]
fn a_block_less_a_prism_lying_inside_it_along_its_edge_answers_every_rule_both_ways_round() {
    let mut wrong = Vec::new();
    for swapped in [false, true] {
        for rule in EVERY_RULE {
            let what = format!("{rule:?}, prism first {swapped}");
            let (model, answer) = fuse(swapped, rule, |model| {
                [a_block(model), a_prism_inside_the_block(model)]
            });
            let outcome = match (rule, swapped) {
                (Rule::Difference, false) => differs(&model, answer, &[BLOCK - 24.0]),
                (Rule::Union, _) => differs(&model, answer, &[BLOCK]),
                (Rule::Intersection, _) => differs(&model, answer, &[24.0]),
                _ => differs(&model, answer, &[]),
            };
            if let Some(why) = outcome {
                wrong.push(format!("{what}: {why}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A solid lying on the block's top along one edge, as the prism does, and
/// reaching down into the block a little further along: the outline `(0, 0)`,
/// `(2, 1)`, `(2, −1)`, `(3, −1)`, `(3, 2)`, `(−1, 2)`, area 7, from `z = 2` to
/// `z = 8`, volume 42, of which `1 × 1 × 6` is inside the block.
///
/// The union is one solid that touches itself along the line, `240 + 42 − 6 =
/// 276`, the intersection the bit inside, 6, and the differences 234 and 36.
///
/// **The union was refused `ResultIsNotManifold { count: 4 }`** until queue
/// row Q1901: the parting asked that nothing but the line join the block's
/// side of it to the hook's, and the bit reaching into the block joins them.
/// Parted, the union is one shell of 13 faces touching itself along the line
/// (the test below).
#[test]
fn a_solid_lying_on_a_block_along_an_edge_and_reaching_into_it_answers_every_rule_both_ways_round()
{
    let mut wrong = Vec::new();
    for swapped in [false, true] {
        for rule in EVERY_RULE {
            let what = format!("{rule:?}, hook first {swapped}");
            let (model, answer) = fuse(swapped, rule, |model| [a_block(model), a_hook(model)]);
            let outcome = match (rule, swapped) {
                (Rule::Union, _) => differs(&model, answer, &[BLOCK + 42.0 - 6.0]),
                (Rule::Intersection, _) => differs(&model, answer, &[6.0]),
                (Rule::Difference, false) => differs(&model, answer, &[234.0]),
                _ => differs(&model, answer, &[36.0]),
            };
            if let Some(why) = outcome {
                wrong.push(format!("{what}: {why}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// An L-shaped plate, the outline `(0, 0)`, `(2, 0)`, `(2, 1)`, `(1, 1)`,
/// `(1, 2)`, `(0, 2)` from `z = 0` to `z = 1`, volume 3, and a pocket cut up
/// into its corner square, the unit square at the origin from `z = −1` to
/// `z = 0.5`, volume 1.5, half of it inside the plate.
///
/// The union is 4, the intersection 0.5, and the pocket less the plate 1.
/// **The plate less the pocket is the plate's two arms, joined above the
/// pocket by the top half of the corner square**, `3 − 0.5 = 2.5`, one solid
/// touching itself along `x = 1, y = 1` from `z = 0` to `z = 0.5`. That line
/// is the plate's inner corner and the pocket's corner both, an edge of both
/// inputs, so the four faces along it are four input faces. It was refused
/// `ResultIsNotManifold { count: 4 }` until queue row Q1913: without the line
/// the four faces are one shell, joined round the plate's top and the top half
/// of the corner square, so the answer is one solid and not the two touching
/// solids §80.9 leaves unbuilt (`docs/decisions.md` §39.4). Queue row Q1910
/// ran it as a shape a designer draws on purpose that met the refusal of the
/// leaning frame less the comb in
/// `a_sliver_along_an_edge_of_a_face_it_lies_on.rs` and of the two bars less a
/// half bar in `a_lens_whose_hole_is_half_a_disk.rs` (§440.3).
#[test]
fn an_l_plate_less_a_pocket_in_its_corner_square_touches_itself_along_an_edge_of_both() {
    let mut wrong = Vec::new();
    for swapped in [false, true] {
        for rule in EVERY_RULE {
            let what = format!("{rule:?}, pocket first {swapped}");
            let (model, answer) = fuse(swapped, rule, |model| {
                [
                    a_prism(
                        model,
                        [
                            [0.0, 0.0],
                            [2.0, 0.0],
                            [2.0, 1.0],
                            [1.0, 1.0],
                            [1.0, 2.0],
                            [0.0, 2.0],
                        ],
                        [0.0, 0.0, 0.0],
                        1.0,
                    ),
                    a_prism(
                        model,
                        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                        [0.0, 0.0, -1.0],
                        1.5,
                    ),
                ]
            });
            let outcome = match (rule, swapped) {
                (Rule::Union, _) => differs(&model, answer, &[4.0]),
                (Rule::Intersection, _) => differs(&model, answer, &[0.5]),
                (Rule::Difference, false) => differs(&model, answer, &[2.5]),
                _ => differs(&model, answer, &[1.0]),
            };
            if let Some(why) = outcome {
                wrong.push(format!("{what}: {why}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The L plate of the test above less a triangular prism lying in its inner
/// corner: the triangle `(1, 1)`, `(0.25, 0.5)`, `(0.5, 0.25)`, area
/// `0.15625`, from `z = 0.25` to `z = 0.75`, volume `0.078125`. Its edge at
/// `(1, 1)` lies along the plate's inner corner and its walls lean into the
/// corner square, so it touches the plate's skin along that line and nowhere
/// else.
///
/// The union is the plate, 3, the intersection the prism, and the prism less
/// the plate nothing. **The plate less the prism is one solid of two shells**,
/// `3 − 0.078125`: the plate's eight faces round a hollow of five that touches
/// them along `x = 1, y = 1` from `z = 0.25` to `z = 0.75`, an edge of both
/// inputs. It was refused `ResultIsNotManifold { count: 4 }` (queue row
/// Q1923): nothing but the line joins the plate's faces to the hollow's, and
/// the parting in `fuse/assemble.rs` asked that a face of one pair be joined
/// to a face of the other walking the line the same way, which is one shell
/// (`docs/decisions.md` §39.4).
#[test]
fn an_l_plate_less_a_prism_lying_in_its_inner_corner_is_one_solid_of_two_shells() {
    let mut wrong = Vec::new();
    for swapped in [false, true] {
        for rule in EVERY_RULE {
            let what = format!("{rule:?}, prism first {swapped}");
            let (model, answer) = fuse(swapped, rule, |model| {
                [
                    an_l_plate(model),
                    a_prism_in_the_l_plates_inner_corner(model),
                ]
            });
            let outcome = match (rule, swapped) {
                (Rule::Union, _) => differs(&model, answer, &[3.0]),
                (Rule::Intersection, _) => differs(&model, answer, &[0.078_125]),
                (Rule::Difference, false) => differs(&model, answer, &[3.0 - 0.078_125]),
                _ => differs(&model, answer, &[]),
            };
            if let Some(why) = outcome {
                wrong.push(format!("{what}: {why}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// **Each answer touching itself along a line is parted where it touches**:
/// the block less the prism inside it is the block's six faces round the
/// hollow's five, two shells meeting only along the prism's edge, and the
/// block united with the hook is one shell of 13 faces. Each validates.
#[test]
fn each_answer_touching_itself_along_a_line_is_the_shells_its_faces_part_into() {
    for (what, touching, faces) in [
        (
            "the block less the prism inside it",
            the_block_less_the_prism_inside_it as fn(&mut Model) -> SolidKey,
            vec![5, 6],
        ),
        (
            "the block united with the hook",
            the_block_united_with_the_hook,
            vec![13],
        ),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let solid = touching(&mut model);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: {why}"));
        let mut found: Vec<usize> = model
            .solid(solid)
            .expect("this model made the key")
            .shells()
            .map(|shell| shell.faces().count())
            .collect();
        found.sort_unstable();
        assert_eq!(found, faces, "{what}: faces of each shell");
    }
}

/// **Each answer touching itself along a line fuses on**: the block less the
/// prism inside it, 216, and the block united with the hook, 276, each fused
/// with a cube of 1 far from it, every rule both ways round. The union is the
/// two as they stand, the intersection no solid, and each difference its
/// first solid whole. Queue row Q1902 asked whether the answer the parting
/// gives a bar resting on a stepped block's wall is refused
/// `ArgumentIntersectsItself` by every fuse after it; these two are not, and
/// since Q1902 that answer fuses on too
/// (`a_bar_resting_on_a_stepped_blocks_wall_fuses_on_with_a_far_cube`,
/// `a_bar_resting_on_a_stepped_blocks_wall_fuses_with_a_slab_across_the_touch`).
#[test]
fn each_answer_touching_itself_along_a_line_fuses_on_with_a_far_cube() {
    let mut wrong = Vec::new();
    for (what, touching, volume) in [
        (
            "the block less the prism inside it",
            the_block_less_the_prism_inside_it as fn(&mut Model) -> SolidKey,
            BLOCK - 24.0,
        ),
        (
            "the block united with the hook",
            the_block_united_with_the_hook,
            BLOCK + 36.0,
        ),
    ] {
        for swapped in [false, true] {
            for rule in EVERY_RULE {
                let (model, answer) =
                    fuse(swapped, rule, |model| [touching(model), a_far_cube(model)]);
                let should: &[f64] = match (rule, swapped) {
                    (Rule::Union, _) => &[volume, 1.0],
                    (Rule::Intersection, _) => &[],
                    (Rule::Difference, false) => &[volume],
                    (Rule::Difference, true) => &[1.0],
                    _ => unreachable!("EVERY_RULE holds only these three"),
                };
                if let Some(why) = differs(&model, answer, should) {
                    wrong.push(format!("{what}, {rule:?}, the cube first {swapped}: {why}"));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The solid [`a_solid_whose_wall_turns_back_past_the_blocks_end`] builds is
/// what queue row Q1117 asked for: one solid of 19 that validates, whose
/// upright wall `x = 0` is one face running along its edge on the block's top
/// and reaching two below the top past the block's end.
///
/// It is three pieces united. A wedge, the triangle `(0, 0)`, `(0, 2)`,
/// `(−2, 2)` from `z = 2` to `z = 8`, area 2 and volume 12, lies on the top
/// along its edge `x = y = 0`, its upright wall rising from the edge and its
/// other wall leaning away. A bar, `−1 ≤ x ≤ 0` and `1 ≤ y ≤ 2` from `z = 5` to
/// `z = 12`, volume 7, has the 3 up to `z = 8` inside the wedge, which at
/// height `y` spans `−y ≤ x ≤ 0`. A tab, `−1 ≤ x ≤ 0` and `−2 ≤ y ≤ 2` from
/// `z = 11` to `z = 12`, volume 4, has the 1 above `y = 1` inside the bar. The
/// solid is `12 + 7 − 3 + 4 − 1 = 19`. Each piece's wall on `x = 0` overlaps
/// the next one's, so the solid's is one face, a hook: along the edge from
/// `z = 2` to `8`, over the block's end at `y = 1`, and down to `y = −2`
/// between `z = 11` and `12`.
#[test]
fn the_wall_that_turns_back_is_one_face_reaching_two_below_the_blocks_top() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let turning = a_solid_whose_wall_turns_back_past_the_blocks_end(&mut model);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("three flat solids united on whole numbers validate");
    let solid = model.solid(turning).expect("this model made the key");
    let measured = solid
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a flat solid this size measures");
    assert_eq!((measured.volume(), measured.volume_error()), (19.0, 0.0));
    let walls: Vec<Vec<[f64; 3]>> = solid
        .faces()
        .filter(|face| face.plane().coefficients() == [1.0, 0.0, 0.0, 0.0])
        .map(|face| {
            let mut corners: Vec<[f64; 3]> = face
                .wires()
                .flat_map(|wire| wire.points().map(|point| point.coordinates()))
                .collect();
            corners.sort_by(|one, other| {
                one.partial_cmp(other)
                    .expect("a corner of a solid has real coordinates")
            });
            corners
        })
        .collect();
    assert_eq!(
        walls,
        [[
            [0.0, -2.0, 11.0],
            [0.0, -2.0, 12.0],
            [0.0, 0.0, 2.0],
            [0.0, 0.0, 8.0],
            [0.0, 1.0, 8.0],
            [0.0, 1.0, 11.0],
            [0.0, 2.0, 2.0],
            [0.0, 2.0, 12.0],
        ]]
    );
}

/// The solid whose wall turns back below the top, on the block: every rule
/// both ways round, checked against volumes. The two share only the wedge's
/// edge, `x = y = 0` from `z = 2` to `z = 8`, since the bar is above `y = 1`
/// and the tab is past the block's end at `z = 10`. So the union is the two as
/// they stood, 240 and 19, the intersection no solid, and each difference its
/// first solid whole.
///
/// **Queue row Q1117 guessed the union refused** `ResultIsNotManifold { count:
/// 4 }`: the parting in `fuse/assemble.rs` then parted the line only where
/// every corner of the two faces resting along it was on the outer side of the
/// top, and this wall has two corners at `y = −2`. It answered because what was
/// asked was a piece of the wall, cut along the top's plane `y = 0` right
/// across, never the wall whole. Since queue row Q1901 the parting asks no
/// corner (`docs/decisions.md` §39.4).
#[test]
fn a_wall_that_turns_back_below_the_top_past_the_blocks_end_answers_every_rule_both_ways_round() {
    let mut wrong = Vec::new();
    for swapped in [false, true] {
        for rule in EVERY_RULE {
            let what = format!("{rule:?}, the solid that turns back first {swapped}");
            let (model, answer) = fuse(swapped, rule, |model| {
                [
                    a_block(model),
                    a_solid_whose_wall_turns_back_past_the_blocks_end(model),
                ]
            });
            let should: &[f64] = match (rule, swapped) {
                (Rule::Union, _) => &[19.0, BLOCK],
                (Rule::Intersection, _) => &[],
                (Rule::Difference, false) => &[BLOCK],
                (Rule::Difference, true) => &[19.0],
                _ => unreachable!("EVERY_RULE holds only these three"),
            };
            if let Some(why) = differs(&model, answer, should) {
                wrong.push(format!("{what}: {why}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The block and the other solid fused, the block first unless `swapped`.
fn fuse(
    swapped: bool,
    rule: Rule,
    build: impl FnOnce(&mut Model) -> [SolidKey; 2],
) -> (Model, rcad_topo::Result<Fused>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [block, other] = build(&mut model);
    let [first, second] = if swapped {
        [other, block]
    } else {
        [block, other]
    };
    let answer = model.fuse(
        &[first],
        &[second],
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    (model, answer)
}

/// What is wrong with an answer that should be one solid per volume in
/// `should`, in any order, each measured to within the rounding its measure
/// owns up to; `None` when nothing is. That rounding is nothing where every
/// corner is on whole and half numbers, and a few `1e-15` where the prism's
/// walls are stored a rounding off them.
fn differs(model: &Model, answer: rcad_topo::Result<Fused>, should: &[f64]) -> Option<String> {
    let fused = match answer {
        Ok(fused) => fused,
        Err(why) => return Some(format!("refused {why:?}")),
    };
    if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
        return Some(format!("wrote a model that does not validate: {why}"));
    }
    let mut found: Vec<(f64, f64)> = fused
        .solids()
        .iter()
        .map(|&solid| {
            let measured = measured(model, solid);
            (measured.volume(), measured.volume_error())
        })
        .collect();
    found.sort_by(|one, other| one.0.total_cmp(&other.0));
    let mut should = should.to_vec();
    should.sort_by(f64::total_cmp);
    let agrees = found.len() == should.len()
        && found
            .iter()
            .zip(&should)
            .all(|(&(volume, error), &should)| (volume - should).abs() <= error);
    (!agrees)
        .then(|| format!("solids of {found:?}, each with its error, and the answer is {should:?}"))
}

/// What is wrong with an answer that should be the solids in `should`, each as
/// it stood before the fuse; `None` when nothing is.
///
/// Each solid of the answer must have the planes of one of them bit for bit,
/// and its volume within what the two measures say they may be out by and what
/// its corners' moves can add. **The corners move**: the fuse writes each one
/// down again from the planes it is the meet of, a rounding or so from where
/// the input had it, and moving the corners of a closed skin by at most `d`
/// changes what it encloses by at most its area times `d`, near enough
/// (`two_turned_cubes_a_rounding_apart.rs` has the argument). Turned by
/// `0.175`, the block came back `8.5e-14` smaller where the two measures'
/// errors add to `8.4e-15`.
fn differs_from_as_they_stand(
    model: &Model,
    answer: rcad_topo::Result<Fused>,
    should: &[&AsItStands],
) -> Option<String> {
    let fused = match answer {
        Ok(fused) => fused,
        Err(why) => return Some(format!("refused {why:?}")),
    };
    if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
        return Some(format!("wrote a model that does not validate: {why}"));
    }
    let mut wanted = should.to_vec();
    for &solid in fused.solids() {
        let found = AsItStands::of(model, solid);
        let Some(at) = wanted
            .iter()
            .position(|wanted| wanted.planes == found.planes)
        else {
            return Some(format!(
                "a solid on {:?} is none of those still wanted",
                found.planes
            ));
        };
        let it = wanted.swap_remove(at);
        let moved = found
            .corners
            .iter()
            .map(|corner| {
                it.corners
                    .iter()
                    .map(|theirs| corner.distance_to(*theirs))
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max);
        let room = found.measured.volume_error()
            + it.measured.volume_error()
            + found.measured.surface_area() * moved;
        if (found.measured.volume() - it.measured.volume()).abs() > room {
            return Some(format!(
                "volume {} against {}, corners moved {moved:e}",
                found.measured.volume(),
                it.measured.volume()
            ));
        }
    }
    (!wanted.is_empty()).then(|| format!("{} solids are missing", wanted.len()))
}

/// A solid as it stands: the plane of each face in one fixed order, each
/// corner of each face, and its measure.
struct AsItStands {
    planes: Vec<[f64; 4]>,
    corners: Vec<Point>,
    measured: Measurement,
}

impl AsItStands {
    fn of(model: &Model, solid: SolidKey) -> Self {
        let faces = || model.solid(solid).expect("this model made the key").faces();
        let mut planes: Vec<[f64; 4]> = faces().map(|face| face.plane().coefficients()).collect();
        planes.sort_by(|one, other| {
            one.iter()
                .zip(other)
                .map(|(one, other)| one.total_cmp(other))
                .find(|order| order.is_ne())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        AsItStands {
            planes,
            corners: faces()
                .flat_map(|face| face.wires().flat_map(|wire| wire.points()))
                .collect(),
            measured: measured(model, solid),
        }
    }
}

fn measured(model: &Model, solid: SolidKey) -> Measurement {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a flat solid this size measures")
}

/// Which side of the block's top each end of the prism's edge lies on, the
/// prism moved by `at` and `length` long: the meet of its two walls and that
/// end's plane, asked of the planes exactly. [`Decision::Pos`] is above.
fn sides_of_the_prisms_edge_against_the_blocks_top(at: [f64; 3], length: f64) -> [Decision; 2] {
    sides_of_the_edge(&planes_round_the_edge(at, length))
}

/// Which side of the block's top each end of the edge lies on: the meet of the
/// two walls and that end's plane, asked of the planes exactly.
/// [`Decision::Pos`] is above.
fn sides_of_the_edge(planes: &RoundTheEdge) -> [Decision; 2] {
    planes.ends.map(|end| {
        side_of_a_plane_where_three_planes_meet([planes.walls[0], planes.walls[1], end], planes.top)
            .expect("two walls and an end meet in a point")
    })
}

/// For each wall of the prism with its edge a rounding into the block's top,
/// which side of the plane `x = across` the wall meets the top on at each end:
/// the meet of the wall, the top and that end's plane, asked of the planes
/// exactly. [`Decision::Pos`] is the side `x > across`.
fn where_each_wall_meets_the_blocks_top_against(across: f64) -> [[Decision; 2]; 2] {
    let planes = planes_round_the_edge(A_ROUNDING_INTO_THE_TOP, 6.0);
    planes.walls.map(|wall| {
        planes.ends.map(|end| {
            side_of_a_plane_where_three_planes_meet(
                [wall, planes.top, end],
                [1.0, 0.0, 0.0, -across],
            )
            .expect("a wall, the top and an end meet in a point")
        })
    })
}

/// The planes round the prism's edge, as coefficients.
struct RoundTheEdge {
    /// The block's top, `y = 0` turned as the block is, facing up.
    top: [f64; 4],
    /// The prism's two walls, in no particular order: every plane of it with a
    /// part along `x`.
    walls: [[f64; 4]; 2],
    /// The prism's two ends, square to `z` turned as the prism is.
    ends: [[f64; 4]; 2],
}

/// The block's top and the prism's walls and ends, the prism moved by `at` and
/// `length` long, as the model stores them.
fn planes_round_the_edge(at: [f64; 3], length: f64) -> RoundTheEdge {
    planes_round_the_edge_turned(&Isometry::identity(), |model| {
        [a_block(model), a_prism(model, RESTING, at, length)]
    })
}

/// The block's top and the prism's walls and ends as the model stores them,
/// the block and the prism as `build` makes them, both turned by `turning`
/// and turned about `x` only.
fn planes_round_the_edge_turned(
    turning: &Isometry,
    build: impl FnOnce(&mut Model) -> [SolidKey; 2],
) -> RoundTheEdge {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [block, prism] = build(&mut model);
    let planes_of = |solid: SolidKey| -> Vec<[f64; 4]> {
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .map(|face| face.plane().coefficients())
            .collect()
    };
    let [x, y, z] = turning
        .apply_to_vector(Vector::new(0.0, 1.0, 0.0))
        .components();
    let top = [x, y, z, 0.0];
    assert!(
        planes_of(block).contains(&top),
        "the block's top is y = 0 turned, facing up, through the origin"
    );
    let along = turning
        .apply_to_vector(Vector::new(0.0, 0.0, 1.0))
        .components();
    let prism = planes_of(prism);
    let two_of = |which: &dyn Fn(&[f64; 4]) -> bool| -> [[f64; 4]; 2] {
        let found: Vec<[f64; 4]> = prism.iter().copied().filter(|plane| which(plane)).collect();
        found
            .try_into()
            .unwrap_or_else(|found| panic!("a prism has two of these, not {found:?}"))
    };
    RoundTheEdge {
        top,
        walls: two_of(&|plane| plane[0] != 0.0),
        ends: two_of(&|plane| plane[..3] == along || plane[..3] == along.map(|part| -part)),
    }
}

fn a_block(model: &mut Model) -> SolidKey {
    a_prism(model, THE_BLOCKS_OUTLINE, [0.0, 0.0, 0.0], 10.0)
}

/// The block and the wedge, both turned about `x` by `turn`, the wedge moved
/// to `z = 2` before it is turned.
fn a_turned_block_and_wedge(model: &mut Model, turn: f64) -> [SolidKey; 2] {
    let turning = turned_by(turn);
    let shift = Isometry::translation(Vector::new(0.0, 0.0, 2.0)).expect("a finite shift");
    [
        a_placed_prism(model, THE_BLOCKS_OUTLINE, &turning, 10.0),
        a_placed_prism(model, WEDGE, &turning.compose(&shift), 6.0),
    ]
}

/// The block and the resting prism of [`RESTING`], six long, both turned about
/// `x` by `turn`, the prism moved to `z = 2` before it is turned.
fn a_turned_block_and_resting_prism(model: &mut Model, turn: f64) -> [SolidKey; 2] {
    let turning = turned_by(turn);
    let shift = Isometry::translation(Vector::new(0.0, 0.0, 2.0)).expect("a finite shift");
    [
        a_placed_prism(model, THE_BLOCKS_OUTLINE, &turning, 10.0),
        a_placed_prism(model, RESTING, &turning.compose(&shift), 6.0),
    ]
}

fn turned_by(turn: f64) -> Isometry {
    Isometry::rotation(&Axis::x(), turn).expect("a finite turn")
}

fn a_turn(fortieths: u8) -> f64 {
    f64::from(fortieths) * 0.025
}

/// The prism of [`INSIDE`] moved to `z = 2`, six long, in the block with its
/// edge on the top.
fn a_prism_inside_the_block(model: &mut Model) -> SolidKey {
    a_prism(model, INSIDE, [0.0, 0.0, 2.0], 6.0)
}

/// The L plate of
/// [`an_l_plate_less_a_pocket_in_its_corner_square_touches_itself_along_an_edge_of_both`],
/// one high, its inner corner the line `x = 1, y = 1`.
fn an_l_plate(model: &mut Model) -> SolidKey {
    a_prism(
        model,
        [
            [0.0, 0.0],
            [2.0, 0.0],
            [2.0, 1.0],
            [1.0, 1.0],
            [1.0, 2.0],
            [0.0, 2.0],
        ],
        [0.0, 0.0, 0.0],
        1.0,
    )
}

/// A triangular prism inside the L plate, its edge along the plate's inner
/// corner from `z = 0.25` to `z = 0.75` and its walls leaning into the
/// corner square.
fn a_prism_in_the_l_plates_inner_corner(model: &mut Model) -> SolidKey {
    a_prism(
        model,
        [[1.0, 1.0], [0.25, 0.5], [0.5, 0.25]],
        [0.0, 0.0, 0.25],
        0.5,
    )
}

fn the_block_less_the_prism_inside_it(model: &mut Model) -> SolidKey {
    let block = a_block(model);
    let prism = a_prism_inside_the_block(model);
    one_solid(model, Rule::Difference, block, prism)
}

fn the_block_united_with_the_hook(model: &mut Model) -> SolidKey {
    let block = a_block(model);
    let hook = a_hook(model);
    one_solid(model, Rule::Union, block, hook)
}

/// A cube of 1 at `(20, 20, 20)`, far from the block and anything on it.
fn a_far_cube(model: &mut Model) -> SolidKey {
    let cube = model
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(20.0, 20.0, 20.0)).expect("a finite shift"),
        )
        .expect("this model made the key");
    cube
}

fn a_hook(model: &mut Model) -> SolidKey {
    a_prism(
        model,
        [
            [0.0, 0.0],
            [2.0, 1.0],
            [2.0, -1.0],
            [3.0, -1.0],
            [3.0, 2.0],
            [-1.0, 2.0],
        ],
        [0.0, 0.0, 2.0],
        6.0,
    )
}

/// A wedge lying on the block's top along its edge, united with a bar and a
/// tab that carry its upright wall over the block's end and down below the
/// top; the first test of the pair above has the shape and its volume.
fn a_solid_whose_wall_turns_back_past_the_blocks_end(model: &mut Model) -> SolidKey {
    let wedge = a_prism(
        model,
        [[0.0, 0.0], [0.0, 2.0], [-2.0, 2.0]],
        [0.0, 0.0, 2.0],
        6.0,
    );
    let bar = a_prism(
        model,
        [[-1.0, 1.0], [0.0, 1.0], [0.0, 2.0], [-1.0, 2.0]],
        [0.0, 0.0, 5.0],
        7.0,
    );
    let tab = a_prism(
        model,
        [[-1.0, -2.0], [0.0, -2.0], [0.0, 2.0], [-1.0, 2.0]],
        [0.0, 0.0, 11.0],
        1.0,
    );
    let one = one_solid(model, Rule::Union, wedge, bar);
    one_solid(model, Rule::Union, one, tab)
}

/// Two flat solids fused under `rule`, where that is one solid.
fn one_solid(model: &mut Model, rule: Rule, one: SolidKey, other: SolidKey) -> SolidKey {
    let fused = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{rule:?} refused: {why}"));
    let &[solid] = fused.solids() else {
        panic!("{rule:?} answers {:?}, not one solid", fused.solids());
    };
    solid
}

/// An outline stood `length` long along `z`, moved by `at`.
fn a_prism<const N: usize>(
    model: &mut Model,
    outline: [[f64; 2]; N],
    [x, y, z]: [f64; 3],
    length: f64,
) -> SolidKey {
    a_placed_prism(
        model,
        outline,
        &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        length,
    )
}

/// An outline stood `length` long along `z`, then moved by `placement`.
fn a_placed_prism<const N: usize>(
    model: &mut Model,
    outline: [[f64; 2]; N],
    placement: &Isometry,
    length: f64,
) -> SolidKey {
    let outline = ProfileBuilder::new(outline.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a simple outline");
    model
        .add_extrusion(&outline, length, placement)
        .expect("a positive length")
}
