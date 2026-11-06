//! A cube turned a little about two axes and moved along `x` so that its
//! corner lands on another cube's edge (queue rows Q2254 and Q3195).
//!
//! The block is `add_cube(10.0)`, `[0, 10]³`, at `Scale::millimetres(100.0)`.
//! The turned cube is a second `add_cube(10.0)` placed by
//! `Isometry::try_from(([tx, 0, 0], q))`: `q` is the turn `0.0043` about `z`
//! and the turn `0.0081` about `x`, put together either way round and divided
//! by its length ([`divided_by_its_length`]) as every read divided it until
//! queue row Q3234, and `tx` is `5` or `5.0000001`. Its corner at the origin is stored at `(tx, 0, 0)`, on
//! the block's bottom front edge, where `y = 0` meets `z = 0`. As meant, the
//! turned cube's bottom, front and left planes pass through that point too,
//! the two skins cross there at one corner, and no answer has anything small
//! in it.
//!
//! **Stored, the planes do not pass through it.** Each placed plane's offset
//! is `n · t`, rounded. The turned cube's bottom, front and left planes then
//! meet outside the block: past `y = 0` by `3.5e-19` to `4.8e-19` at three
//! placements, and below `z = 0` by `1.7e-21` at the fourth. Where a face of
//! one cube crosses a face of the other near that corner, the line they cross
//! along runs between two meets of planes a rounding apart, `2.1e-19` to
//! `1.1e-16`, and that line is an edge of every answer: the union, what the
//! two share, and each difference. Worked out exactly from the stored numbers,
//! every answer's shortest edge is `1.7e-21` to `1.1e-16` long, against the
//! cubes' reach of `15`: a spread of `1.4e17` to `8.7e21`, where a model
//! supports `1e11`.
//!
//! **Under `Policy::certified()` 18 of the 24 fuses are refused for that
//! edge**, `DynamicRangeTooWide`. The six that answer are those with the turn
//! about `x` first and the move by exactly `5`: there the fuse writes the
//! edge's ends as the one corner `(5, 0, 0)`, the shape as meant. Under
//! `Policy::tolerant(100.0)`, which takes the range check, all 24 answer and
//! validate.
//!
//! **The refusals are a limit** (`docs/decisions.md` §16.10). The answer of
//! the numbers the model holds has the edge, so no fuse of those numbers can
//! answer without it. The families of fix, and why none of them reaches past
//! that:
//!
//! - Doing the arithmetic in another order, or more exactly. The edge is in
//!   the exact answer of the stored numbers, which
//!   [`every_answer_of_the_stored_planes_has_an_edge_a_rounding_long_where_the_skins_cross`]
//!   works out.
//! - Scaling the whole shape by a power of two. That scales the edge and the
//!   coordinates alike, so the spread stays.
//! - Taking two points a rounding apart as one. That is a tolerance, and
//!   `Policy::tolerant(100.0)` is that tolerance.
//! - Keeping, from the placement, the fact that the turned corner lies on the
//!   block's edge. That answers the shape as meant rather than the numbers
//!   stored, and carrying such a fact is `docs/decisions.md` §448's question
//!   (queue row Q1998, parked on §448.7).
//!
//! §294.21 is the same limit for a pyramid placed and cut through its apex.
//! Q2254's probe, which was not committed, counted 15 refusals where these
//! tests count 18; how that probe put the two turns together is not known.

use std::ops::Range;

use rcad_geom::{Deflection, DynamicRange, Isometry, Orientation, Point, Scale};
use rcad_predicates::{Decision, Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
use rcad_topo::{Error, Fused, Invalid, Model, Result, Rule, SolidKey};

/// The turn `0.0081` about `x` and then `0.0043` about `z`, the quaternion
/// `q_z · q_x`, `[w, i, j, k]`, from each half angle's cosine and sine as
/// `f64::cos` and `f64::sin` give them.
const TURNED_ABOUT_X_FIRST: [f64; 4] = [
    0.999_989_487_531_055_4,
    0.004_049_979_567_788_275,
    8.707_469_487_508_162e-6,
    0.002_149_980_710_954_735_7,
];

/// The same two turns the other way round, `0.0043` about `z` first:
/// `q_x · q_z`.
const TURNED_ABOUT_Z_FIRST: [f64; 4] = [
    0.999_989_487_531_055_4,
    0.004_049_979_567_788_275,
    -8.707_469_487_508_162e-6,
    0.002_149_980_710_954_735_7,
];

/// The one placement `Policy::certified()` answers every fuse of.
const ANSWERED_UNDER_CERTIFIED: ([f64; 4], f64) = (TURNED_ABOUT_X_FIRST, 5.0);

/// The three placements `Policy::certified()` refuses every fuse of.
const REFUSED_UNDER_CERTIFIED: [([f64; 4], f64); 3] = [
    (TURNED_ABOUT_X_FIRST, 5.000_000_1),
    (TURNED_ABOUT_Z_FIRST, 5.0),
    (TURNED_ABOUT_Z_FIRST, 5.000_000_1),
];

const SIX_FUSES: [(Rule, Order); 6] = [
    (Rule::Union, Order::BlockFirst),
    (Rule::Union, Order::TurnedFirst),
    (Rule::Intersection, Order::BlockFirst),
    (Rule::Intersection, Order::TurnedFirst),
    (Rule::Difference, Order::BlockFirst),
    (Rule::Difference, Order::TurnedFirst),
];

/// Where the block's planes are in [`the_twelve_planes`], and where the
/// turned cube's are.
const BLOCK: Range<usize> = 0..6;
const TURNED: Range<usize> = 6..12;

/// Half the side of the box round `(tx, 0, 0)` that the short edge is found
/// in: `2⁻⁴⁹`, about `1.8e-15`. Twice the gap between doubles at `5`, so
/// `tx ± HALF_A_BOX` is exact.
const HALF_A_BOX: f64 = 8.0 * f64::EPSILON;

/// Under `Policy::certified()` every fuse of the three other placements is
/// refused for the edge a rounding long that the module doc describes,
/// `DynamicRangeTooWide`, at a spread of `1.7e16` to `1e37`. The spread is
/// measured on the corners as the fuse rounds them, so it can stand far above
/// the exact one, and both are past the `1e11` a model supports.
#[test]
fn certified_refuses_eighteen_of_the_twenty_four_fuses_for_an_edge_a_rounding_long() {
    let mut refused = 0;
    for (turn, tx) in REFUSED_UNDER_CERTIFIED {
        for (rule, order) in SIX_FUSES {
            let asked = format!("{turn:?} to {tx}, {rule:?}, {order:?}");
            match fused(turn, tx, rule, order, &Policy::certified()).1 {
                Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => assert!(
                    ratio > 1e16,
                    "{asked}: a spread of {ratio:e} is not an edge a rounding long"
                ),
                Err(why) => panic!("{asked}: {why}"),
                Ok(_) => panic!("{asked}: answered"),
            }
            refused += 1;
        }
    }
    assert_eq!(refused, 18);
}

/// Under `Policy::certified()` the six fuses with the turn about `x` first and
/// the move by exactly `5` answer and validate. The exact answer of the
/// stored numbers has the edge here too, `1.1e-16` long, but the fuse writes
/// no corner near `(5, 0, 0)` other than `(5, 0, 0)` itself, where the turned
/// cube's corner is stored: the shape as meant. Which of the ways the fuse
/// takes two names as one corner (`docs/decisions.md` §365.7) does it here was
/// not traced.
#[test]
fn certified_answers_the_six_fuses_turned_about_x_first_and_moved_by_exactly_five() {
    let (turn, tx) = ANSWERED_UNDER_CERTIFIED;
    let policy = Policy::certified();
    for (rule, order) in SIX_FUSES {
        let asked = format!("{turn:?} to {tx}, {rule:?}, {order:?}");
        let (model, answer) = fused(turn, tx, rule, order, &policy);
        let answer = answer.unwrap_or_else(|why| panic!("{asked}: {why}"));
        let worked = match rule {
            Rule::Union => 1_501.463_990_222_888_5,
            Rule::Intersection => 498.536_009_777_111_13,
            Rule::Difference => 501.463_990_222_888_9,
            other => panic!("{asked}: no volume pinned for {other:?}"),
        };
        assert_about(volume_of(&model, &answer, &policy, &asked), worked, &asked);
        for corner in the_corners_of(&model, answer.solids()) {
            let near = (corner.x() - tx).abs() < 1e-9
                && corner.y().abs() < 1e-9
                && corner.z().abs() < 1e-9;
            if near {
                assert_eq!(
                    [corner.x(), corner.y(), corner.z()],
                    [tx, 0.0, 0.0],
                    "{asked}: a corner a rounding from the turned cube's"
                );
            }
        }
    }
}

/// Under `Policy::tolerant(100.0)`, which takes the range check, every fuse
/// of every placement answers and validates, and the volumes agree with one
/// another: the union is both cubes less what they share, and each difference
/// is one cube less it.
#[test]
fn tolerant_answers_all_twenty_four_fuses_and_their_volumes_agree() {
    let policy = Policy::tolerant(100.0);
    for (turn, tx) in std::iter::once(ANSWERED_UNDER_CERTIFIED).chain(REFUSED_UNDER_CERTIFIED) {
        let volume = |rule: Rule, order: Order| {
            let asked = format!("{turn:?} to {tx}, {rule:?}, {order:?}");
            let (model, answer) = fused(turn, tx, rule, order, &policy);
            let answer = answer.unwrap_or_else(|why| panic!("{asked}: {why}"));
            (volume_of(&model, &answer, &policy, &asked), asked)
        };
        let (shared, asked) = volume(Rule::Intersection, Order::BlockFirst);
        assert!(
            (498.0..499.0).contains(&shared),
            "{asked}: the turned cube's half inside the block measured {shared}"
        );
        for (rule, order) in SIX_FUSES {
            let (got, asked) = volume(rule, order);
            let worked = match rule {
                Rule::Union => 2000.0 - shared,
                Rule::Intersection => shared,
                Rule::Difference => 1000.0 - shared,
                other => panic!("{asked}: no volume worked for {other:?}"),
            };
            assert_about(got, worked, &asked);
        }
    }
}

/// **Every answer of the stored numbers has an edge shorter than `6.2e-15`
/// near `(tx, 0, 0)`, at every placement.** Worked out exactly with
/// [`side_of_a_plane_where_three_planes_meet`], which asks the four planes
/// rather than a rounded meet.
///
/// What the two cubes share is where all twelve planes have a point behind
/// them or on them. The test finds two of its corners `M1` and `M2`, each the
/// meet of three planes, inside a box `2⁻⁴⁹` each way round `(tx, 0, 0)`. They
/// share a plane `P` of the block and a plane `Q` of the turned cube, and each
/// of the other ten planes has `M1` or `M2` strictly behind it. So between
/// `M1` and `M2` only `P` and `Q` pass through the line they share, and past
/// each end the third plane of that end has the line in front of it. Near a
/// point between the two, the block is what is behind `P` and the turned cube
/// what is behind `Q`. Those two half spaces cross along the line, so the
/// stretch from `M1` to `M2` is an edge of the union, of what the two share,
/// and of each difference, and it ends at `M1` and `M2`, where one cube stops.
/// It is no longer than the box's diagonal, `2√3 · 2⁻⁴⁹`, about `6.2e-15`, at a
/// corner whose `x` is about `5`: a spread of `8e14`, past the `1e11` a model
/// supports.
///
/// At `tx = 0` each placed offset `n · 0` is exactly `0`, all six planes near
/// the corner pass through it, and the same search finds no such edge: the
/// rounding of `n · t` is what makes it.
#[test]
fn every_answer_of_the_stored_planes_has_an_edge_a_rounding_long_where_the_skins_cross() {
    let longest = 2.0 * 3f64.sqrt() * HALF_A_BOX;
    assert!((5.0 - HALF_A_BOX) / longest > DynamicRange::LARGEST_SUPPORTED_RATIO);
    for (turn, tx) in std::iter::once(ANSWERED_UNDER_CERTIFIED).chain(REFUSED_UNDER_CERTIFIED) {
        assert_eq!((tx + HALF_A_BOX) - tx, HALF_A_BOX, "{tx}: the box is exact");
        assert_eq!(tx - (tx - HALF_A_BOX), HALF_A_BOX, "{tx}: the box is exact");
        let planes = the_twelve_planes(turn, tx);
        let corners = the_corners_of_what_they_share_near(&planes, tx);
        let edges = the_edges_where_the_skins_cross(&planes, &corners);
        assert!(
            !edges.is_empty(),
            "{turn:?} to {tx}: no edge where the skins cross among the corners {corners:?}"
        );
    }
    for turn in [TURNED_ABOUT_X_FIRST, TURNED_ABOUT_Z_FIRST] {
        let planes = the_twelve_planes(turn, 0.0);
        let corners = the_corners_of_what_they_share_near(&planes, 0.0);
        let edges = the_edges_where_the_skins_cross(&planes, &corners);
        assert_eq!(edges, [], "{turn:?} to 0, among the corners {corners:?}");
    }
}

/// **The turned cube's corner is stored on the block's edge, and its planes
/// meet outside the block.** The corner is at `(tx, 0, 0)` exactly, but that
/// point is off one of the turned cube's bottom, front and left planes or
/// more, since each placed offset `n · t` is rounded. Those three planes
/// meet in front of the block's `y = 0` or `z = 0`, a rounding outside the
/// block, so the meets of the five planes near the corner are points a
/// rounding apart, not one.
#[test]
fn the_turned_cubes_planes_meet_outside_the_block_though_its_corner_is_stored_on_its_edge() {
    let y_is_0 = [0.0, -1.0, 0.0, 0.0];
    let z_is_0 = [0.0, 0.0, -1.0, 0.0];
    let x_is = |tx: f64| [1.0, 0.0, 0.0, -tx];
    for (turn, tx) in std::iter::once(ANSWERED_UNDER_CERTIFIED).chain(REFUSED_UNDER_CERTIFIED) {
        let asked = format!("{turn:?} to {tx}");
        let (model, _, turned) = two_cubes(turn, tx);
        assert!(
            the_corners_of(&model, &[turned]).iter().any(|corner| [
                corner.x(),
                corner.y(),
                corner.z()
            ] == [tx, 0.0, 0.0]),
            "{asked}: no corner stored at ({tx}, 0, 0)"
        );

        let planes = the_twelve_planes(turn, tx);
        assert!(planes[BLOCK].contains(&y_is_0) && planes[BLOCK].contains(&z_is_0));
        let lowest = |axis: usize| {
            TURNED
                .min_by(|&one, &other| planes[one][axis].total_cmp(&planes[other][axis]))
                .expect("six planes")
        };
        let [left, front, bottom] = [0, 1, 2].map(|axis| planes[lowest(axis)]);
        let the_stored_corner = [y_is_0, z_is_0, x_is(tx)];
        assert!(
            [left, front, bottom]
                .iter()
                .any(|&plane| side(the_stored_corner, plane) != Decision::Zero),
            "{asked}: the stored corner is on all three of its planes"
        );
        let where_they_meet = [bottom, front, left];
        assert!(
            side(where_they_meet, y_is_0) == Decision::Pos
                || side(where_they_meet, z_is_0) == Decision::Pos,
            "{asked}: the turned cube's planes meet inside the block or on its skin"
        );
    }
}

fn assert_about(got: f64, worked: f64, asked: &str) {
    assert!(
        (got - worked).abs() <= 1e-9 * worked,
        "{asked}: measured {got}, worked {worked}"
    );
}

#[derive(Clone, Copy, Debug)]
enum Order {
    BlockFirst,
    TurnedFirst,
}

/// The block and the turned cube fused by `rule` in `order` under `policy`,
/// and the model the answer is written into.
fn fused(
    turn: [f64; 4],
    tx: f64,
    rule: Rule,
    order: Order,
    policy: &Policy,
) -> (Model, Result<Fused>) {
    let (mut model, block, turned) = two_cubes(turn, tx);
    let (arguments, tools) = match order {
        Order::BlockFirst => ([block], [turned]),
        Order::TurnedFirst => ([turned], [block]),
    };
    let answer = model.fuse(&arguments, &tools, rule, policy, &mut ProvenanceLog::new());
    (model, answer)
}

/// The volume of `answer`, once `model`, which it is written into, validates
/// under `policy`.
fn volume_of(model: &Model, answer: &Fused, policy: &Policy, asked: &str) -> f64 {
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{asked}: {why}"));
    answer
        .solids()
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.1).expect("a deflection"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{asked} does not measure: {why}"))
                .volume()
        })
        .sum()
}

/// Every corner of every face of `solids`, once for each outline it is on.
fn the_corners_of(model: &Model, solids: &[SolidKey]) -> Vec<Point> {
    let mut corners = Vec::new();
    for &solid in solids {
        for face in model.solid(solid).expect("this model made the key").faces() {
            for wire in face.wires() {
                corners.extend(wire.points());
            }
        }
    }
    corners
}

/// The block's six planes and then the turned cube's, each as
/// `[nx, ny, nz, d]` with `n · p + d` positive outside its cube.
fn the_twelve_planes(turn: [f64; 4], tx: f64) -> [[f64; 4]; 12] {
    let (model, block, turned) = two_cubes(turn, tx);
    let mut planes = Vec::new();
    for solid in [block, turned] {
        for face in model.solid(solid).expect("this model made the key").faces() {
            assert_eq!(face.orientation(), Orientation::Forward);
            planes.push(face.plane().coefficients());
        }
    }
    planes.try_into().unwrap_or_else(|planes: Vec<_>| {
        panic!("two cubes have twelve faces, not {}", planes.len())
    })
}

/// Each meet of three of `planes` that every one of them has behind it or on
/// it, and that lies in the box `HALF_A_BOX` each way round `(tx, 0, 0)`: the
/// corners of what the two cubes share near the turned cube's corner.
fn the_corners_of_what_they_share_near(planes: &[[f64; 4]; 12], tx: f64) -> Vec<[usize; 3]> {
    let half = HALF_A_BOX;
    let the_box = [
        [1.0, 0.0, 0.0, -(tx + half)],
        [-1.0, 0.0, 0.0, tx - half],
        [0.0, 1.0, 0.0, -half],
        [0.0, -1.0, 0.0, -half],
        [0.0, 0.0, 1.0, -half],
        [0.0, 0.0, -1.0, -half],
    ];
    let mut corners = Vec::new();
    for first in 0..planes.len() {
        for second in first + 1..planes.len() {
            for third in second + 1..planes.len() {
                let three = [first, second, third].map(|at| planes[at]);
                match side_of_a_plane_where_three_planes_meet(three, three[0]) {
                    Ok(_) => {}
                    Err(rcad_predicates::Error::PlanesDoNotMeetInAPoint { .. }) => continue,
                    Err(why) => panic!("{three:?}: {why}"),
                }
                if planes
                    .iter()
                    .chain(&the_box)
                    .all(|&plane| side(three, plane) != Decision::Pos)
                {
                    corners.push([first, second, third]);
                }
            }
        }
    }
    corners
}

/// Each two of `corners` that share one plane of the block and one of the
/// turned cube, where every other plane has one corner of the two or both
/// strictly behind it: the two ends of an edge where the skins cross.
fn the_edges_where_the_skins_cross(
    planes: &[[f64; 4]; 12],
    corners: &[[usize; 3]],
) -> Vec<([usize; 3], [usize; 3])> {
    let mut edges = Vec::new();
    for (at, &one) in corners.iter().enumerate() {
        for &other in &corners[at + 1..] {
            let shared: Vec<usize> = one
                .iter()
                .copied()
                .filter(|plane| other.contains(plane))
                .collect();
            let [on_the_block, on_the_turned_cube] = shared[..] else {
                continue;
            };
            if !(BLOCK.contains(&on_the_block) && TURNED.contains(&on_the_turned_cube)) {
                continue;
            }
            let [at_one, at_other] = [one, other].map(|corner| corner.map(|at| planes[at]));
            let each_other_plane_has_one_strictly_behind = (0..planes.len())
                .filter(|plane| !shared.contains(plane))
                .all(|plane| {
                    side(at_one, planes[plane]) == Decision::Neg
                        || side(at_other, planes[plane]) == Decision::Neg
                });
            if each_other_plane_has_one_strictly_behind {
                edges.push((one, other));
            }
        }
    }
    edges
}

/// Which side of `plane` the point where `three` meet lies on, worked out
/// exactly.
fn side(three: [[f64; 4]; 3], plane: [f64; 4]) -> Decision {
    let side = side_of_a_plane_where_three_planes_meet(three, plane)
        .unwrap_or_else(|why| panic!("{three:?} meet in a point: {why}"));
    assert!(
        matches!(side, Decision::Neg | Decision::Zero | Decision::Pos),
        "exact arithmetic decides {three:?} against {plane:?}, and gave {side:?}"
    );
    side
}

/// The block, `[0, 10]³`, and a second cube of side ten turned by `turn` and
/// moved to `(tx, 0, 0)`.
fn two_cubes(turn: [f64; 4], tx: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a scale"));
    let block = a_cube(&mut model);
    let turned = a_cube(&mut model);
    let placement =
        Isometry::try_from(([tx, 0.0, 0.0], divided_by_its_length(turn))).expect("a placement");
    model
        .place_solid(turned, &placement)
        .expect("this model made the key");
    (model, block, turned)
}

/// [`divided_by_its_length`] moves each of the four parts of both turns one
/// step further from zero.
#[test]
fn dividing_either_turn_by_its_length_moves_every_part_one_step_further_from_zero() {
    for turn in [TURNED_ABOUT_X_FIRST, TURNED_ABOUT_Z_FIRST] {
        assert_eq!(
            divided_by_its_length(turn).map(f64::to_bits),
            turn.map(|part| part.to_bits() + 1),
            "{turn:?}"
        );
    }
}

/// `turn` divided by its length, as `Isometry::try_from` divided every turn
/// until queue row Q3234, by nalgebra's `UnitQuaternion::from_quaternion`,
/// which adds the squares as `(i² + k²) + (j² + w²)`.
///
/// **Every outcome this file pins was measured on these bits.** The read now
/// keeps a turn this near length one as it is handed, and dividing moves each
/// of the four parts of both turns one step further from zero. The stored
/// planes miss the corner by a rounding, so that step moves what the fuses
/// meet: on the kept bits, with the turn about `z` first and the move by
/// exactly `5`, the stored planes have no edge where the skins cross, and the
/// certified union answers.
fn divided_by_its_length([w, i, j, k]: [f64; 4]) -> [f64; 4] {
    let length = ((i * i + k * k) + (j * j + w * w)).sqrt();
    [w, i, j, k].map(|part| part / length)
}

fn a_cube(model: &mut Model) -> SolidKey {
    model
        .add_cube(10.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube")
}
