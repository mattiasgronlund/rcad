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
//! Two shapes that also reach four darts that way are pinned refused, because
//! their answer is one solid pinched along the line rather than two lumps:
//! a block less a prism lying inside it along its edge, and a block united
//! with a solid lying on it along one edge and reaching down into it somewhere
//! else. Both validate if they are parted, which is why the parting asks.
//!
//! **A solid whose upright wall rests along the edge and turns back below the
//! top past the block's end answers too** (queue row Q1117, which guessed it
//! refused): the parting asks every corner of a piece of the wall, and the
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

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Decision, Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The block's volume: eight by three by ten.
const BLOCK: f64 = 240.0;

/// The prism's triangle, its corner at the origin, both walls leaning up.
const RESTING: [[f64; 2]; 3] = [[0.0, 0.0], [3.0, 2.0], [-1.0, 2.0]];

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

/// The prism turned over and moved into the block, its edge on the top face
/// from below: the triangle `(0, 0)`, `(−1, −2)`, `(3, −2)`, from `z = 2` to
/// `z = 8`. The union is the block, the intersection the prism, and the prism
/// less the block nothing.
///
/// The block less the prism has a prism-shaped hollow whose edge touches the
/// top face, which is one solid pinched along the line, and it stays refused.
/// The four darts are the same two pairs as on top, but the prism's walls,
/// turned round, lie on the inner side of the top face; parted, the answer
/// came back as one solid of two shells, 216, that validates.
#[test]
fn a_block_less_a_prism_lying_inside_it_along_its_edge_is_refused_and_the_rest_answer() {
    let mut wrong = Vec::new();
    let inside = [[0.0, 0.0], [-1.0, -2.0], [3.0, -2.0]];
    for swapped in [false, true] {
        for rule in EVERY_RULE {
            let what = format!("{rule:?}, prism first {swapped}");
            let (model, answer) = fuse(swapped, rule, |model| {
                [a_block(model), a_prism(model, inside, [0.0, 0.0, 2.0], 6.0)]
            });
            let outcome = match (rule, swapped) {
                (Rule::Difference, false) => refused_as_four_faces_along_one_edge(answer),
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
/// The union is one solid that touches itself along the line, and it stays
/// refused. Parted, it came back as one shell of 276 that validates. The
/// intersection is the bit inside, 6, and the differences are 234 and 36.
#[test]
fn a_solid_lying_on_a_block_along_an_edge_and_reaching_into_it_is_refused_united() {
    let mut wrong = Vec::new();
    for swapped in [false, true] {
        for rule in EVERY_RULE {
            let what = format!("{rule:?}, hook first {swapped}");
            let (model, answer) = fuse(swapped, rule, |model| [a_block(model), a_hook(model)]);
            let outcome = match (rule, swapped) {
                (Rule::Union, _) => refused_as_four_faces_along_one_edge(answer),
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
/// 4 }`: `rest_on_the_outer_side_of` in `fuse/assemble.rs` parts the line only
/// where every corner of the two faces resting along it is on the outer side
/// of the top, and this wall has two corners at `y = −2`. It answers because
/// what is asked is a piece of the wall, never the wall whole, and the fuse
/// cuts each flat face along the line of every plane that crosses it, its own
/// solid's and the other's (`fuse/cuts.rs`). That makes the piece along the
/// edge lie on one side of the edge twice over. The lines the wall's own edges
/// lie on cut it, so each piece is convex, and a convex piece with a side on a
/// line lies on one side of that line. And the top's plane cuts it along
/// `y = 0` right across, past the block's end too, since the top touches the
/// wall along the edge and so is not a face that misses the wall's plane. On
/// one side of the top's plane, every corner of the piece being on the outer
/// side is the side the face leaves the line to.
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
            let measured = model
                .solid(solid)
                .expect("the fuse made it")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a flat solid this size measures");
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

fn refused_as_four_faces_along_one_edge(answer: rcad_topo::Result<Fused>) -> Option<String> {
    match answer {
        Err(Error::ResultIsNotManifold { count: 4 }) => None,
        Err(why) => Some(format!("refused {why:?}, not as four faces along one edge")),
        Ok(fused) => Some(format!(
            "answered {:?}, which is one solid pinched along a line",
            fused.solids()
        )),
    }
}

/// Which side of the block's top each end of the prism's edge lies on, the
/// prism moved by `at` and `length` long: the meet of its two walls and that
/// end's plane, asked of the planes exactly. [`Decision::Pos`] is above.
fn sides_of_the_prisms_edge_against_the_blocks_top(at: [f64; 3], length: f64) -> [Decision; 2] {
    let planes = planes_round_the_edge(at, length);
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
    /// The block's top, `y = 0`, facing up.
    top: [f64; 4],
    /// The prism's two walls, which lean, in no particular order.
    walls: [[f64; 4]; 2],
    /// The prism's two ends, square to `z`.
    ends: [[f64; 4]; 2],
}

/// The block's top and the prism's walls and ends, the prism moved by `at` and
/// `length` long, as the model stores them.
fn planes_round_the_edge(at: [f64; 3], length: f64) -> RoundTheEdge {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let block = a_block(&mut model);
    let prism = a_prism(&mut model, RESTING, at, length);
    let planes_of = |solid: SolidKey| -> Vec<[f64; 4]> {
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .map(|face| face.plane().coefficients())
            .collect()
    };
    let top = [0.0, 1.0, 0.0, 0.0];
    assert!(
        planes_of(block).contains(&top),
        "the block's top is y = 0, facing up"
    );
    let prism = planes_of(prism);
    let two_of = |which: fn(&[f64; 4]) -> bool| -> [[f64; 4]; 2] {
        let found: Vec<[f64; 4]> = prism.iter().copied().filter(which).collect();
        found
            .try_into()
            .unwrap_or_else(|found| panic!("a prism has two of these, not {found:?}"))
    };
    RoundTheEdge {
        top,
        walls: two_of(|plane| plane[0] != 0.0),
        ends: two_of(|plane| plane[2] != 0.0),
    }
}

fn a_block(model: &mut Model) -> SolidKey {
    a_prism(
        model,
        [[-4.0, -3.0], [4.0, -3.0], [4.0, 0.0], [-4.0, 0.0]],
        [0.0, 0.0, 0.0],
        10.0,
    )
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
    let one = united(model, wedge, bar);
    united(model, one, tab)
}

fn united(model: &mut Model, one: SolidKey, other: SolidKey) -> SolidKey {
    let fused = model
        .fuse(
            &[one],
            &[other],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two flat solids that overlap unite");
    let &[solid] = fused.solids() else {
        panic!(
            "two solids that overlap unite into one, not {:?}",
            fused.solids()
        );
    };
    solid
}

#[test]
fn probe_one() {
    let r = 3.0_f64.sqrt();
    let wedge = [[0.0, 0.0], [1.0, r], [0.0, r]];
    let turn = 3.0 * 0.025;
    let rotation = Isometry::rotation(&rcad_geom::Axis::x(), turn).expect("a finite turn");
    let shift = Isometry::translation(Vector::new(0.0, 0.0, 2.0)).expect("a finite shift");
    let placed = rotation.compose(&shift);
    let (_, answer) = fuse(false, Rule::Union, |model| {
        [
            a_placed_prism(
                model,
                [[-4.0, -3.0], [4.0, -3.0], [4.0, 0.0], [-4.0, 0.0]],
                &rotation,
                10.0,
            ),
            a_placed_prism(model, wedge, &placed, 6.0),
        ]
    });
    panic!("{answer:?}");
}

#[test]
fn probe_turned_about_x() {
    let r = 3.0_f64.sqrt();
    let wedge = [[0.0, 0.0], [1.0, r], [0.0, r]];
    let mut table: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut examples: Vec<String> = Vec::new();
    for step in 1..=60 {
        let turn = f64::from(step) * 0.025;
        for (name, outline, volume) in [("wedge", wedge, 3.0 * r), ("resting", RESTING, 24.0)] {
            let rotation = Isometry::rotation(&rcad_geom::Axis::x(), turn).expect("a finite turn");
            let shift = Isometry::translation(Vector::new(0.0, 0.0, 2.0)).expect("a finite shift");
            let placed = rotation.compose(&shift);
            let sides = {
                let mut model =
                    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                let block = a_placed_prism(
                    &mut model,
                    [[-4.0, -3.0], [4.0, -3.0], [4.0, 0.0], [-4.0, 0.0]],
                    &rotation,
                    10.0,
                );
                let prism = a_placed_prism(&mut model, outline, &placed, 6.0);
                let planes_of = |solid: SolidKey| -> Vec<[f64; 4]> {
                    model
                        .solid(solid)
                        .expect("made")
                        .faces()
                        .map(|face| face.plane().coefficients())
                        .collect()
                };
                let up = rotation
                    .apply_to_vector(Vector::new(0.0, 1.0, 0.0))
                    .components();
                let along = rotation
                    .apply_to_vector(Vector::new(0.0, 0.0, 1.0))
                    .components();
                let top: Vec<[f64; 4]> = planes_of(block)
                    .into_iter()
                    .filter(|p| p[..3] == up[..] && p[3] == 0.0)
                    .collect();
                let prism_planes = planes_of(prism);
                let ends: Vec<[f64; 4]> = prism_planes
                    .iter()
                    .copied()
                    .filter(|p| p[..3] == along[..] || p[..3] == along.map(|c| -c)[..])
                    .collect();
                let walls: Vec<[f64; 4]> = prism_planes
                    .iter()
                    .copied()
                    .filter(|p| p[0] != 0.0)
                    .collect();
                if top.len() != 1 || ends.len() != 2 || walls.len() != 2 {
                    format!("planes? top {top:?} ends {ends:?} walls {walls:?}")
                } else {
                    format!(
                        "{:?}",
                        [ends[0], ends[1]].map(|end| side_of_a_plane_where_three_planes_meet(
                            [walls[0], walls[1], end],
                            top[0]
                        )
                        .expect("meet"))
                    )
                }
            };
            for swapped in [false, true] {
                let (model, answer) = fuse(swapped, Rule::Union, |model| {
                    [
                        a_placed_prism(
                            model,
                            [[-4.0, -3.0], [4.0, -3.0], [4.0, 0.0], [-4.0, 0.0]],
                            &rotation,
                            10.0,
                        ),
                        a_placed_prism(model, outline, &placed, 6.0),
                    ]
                });
                let outcome = match &answer {
                    Ok(_) => differs(&model, answer, &[volume, BLOCK])
                        .map_or("answers".to_owned(), |why| why.chars().take(80).collect()),
                    Err(why) => format!("{why:?}").chars().take(60).collect(),
                };
                let key = format!("{name} {sides} {outcome}");
                if name == "wedge" && outcome.starts_with("Result") {
                    examples.push(format!("REFUSED step {step} swapped {swapped}"));
                }
                if !table.contains_key(&key) {
                    examples.push(format!("{key} at turn {turn} swapped {swapped}"));
                }
                *table.entry(key).or_default() += 1;
            }
        }
    }
    panic!("{table:#?}\n{}", examples.join("\n"));
}

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

/// An outline stood `length` long along `z`, moved by `at`.
fn a_prism<const N: usize>(
    model: &mut Model,
    outline: [[f64; 2]; N],
    [x, y, z]: [f64; 3],
    length: f64,
) -> SolidKey {
    let outline = ProfileBuilder::new(outline.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a simple outline");
    model
        .add_extrusion(
            &outline,
            length,
            &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        )
        .expect("a positive length")
}
