//! A plate with a round tab, and a block whose corner `(12.4, 8.2)` sits on
//! the tab's rim (Q2232).
//!
//! The plate is the square `[0, 10]²` extruded 2 high, united with a round bar
//! of radius 3 and height 2 at `(10, 10, 0)`. `2.4² + 1.8² = 3²`, so the corner
//! is on the rim, and in `f64` it is a rounding outside it. The block's walls
//! `x = 12.4` and `y = 8.2` cross the rim's circle at two points a rounding
//! apart, and every fuse was refused on which of the two comes first round the
//! circle. The walls and the plate's top or floor meet at the corner, an `f64`
//! triple, and `fuse/arrange.rs` now reads the order off them where the policy
//! would refuse it.
//!
//! Under a certified policy every fuse was then refused at the sliver of the
//! block's face between the corner and the rim, which holds no `f64` point:
//! `fuse/select.rs`'s `by_a_corner` passed the corner over because it lies on
//! the plane of the plate's top or floor. It now passes a corner over only on
//! the plane of a flat face with straight sides, which the ray reads by its
//! corners, and asks it on every other face (Q2243). So the block less the
//! plate and the two's common part answer, and the rest are refused as below.
//! A tolerant policy reads the two crossings as one point, and there every
//! fuse of a block cornered on the rim once answers.
//!
//! What is still refused, and whether its answer exists (§16.10, §437.6):
//!
//! - **The union, a limit.** It holds the sliver of the block's floor or roof
//!   outside the rim, a face thinner than a rounding of its own coordinates,
//!   so it is refused `DynamicRangeTooWide`.
//! - **The plate less the block, a gap.** It is refused the same way, though
//!   its answer is the plate itself, which has no such face.
//! - **Each fuse of the block `x 7.6 to 12.4` by `y 8.2 to 11.8`, a gap.** Its
//!   other corner `(7.6, 11.8)` is on the rim too, and the ray from there meets
//!   the tab's wall, which `holds.rs` cannot follow (Q2231).

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Invalid, Model, ProfileBuilder, Rule, SolidKey};

const INSIDE_A_PIECE: &str = "a point inside a piece of a face";

/// Every rule, with the plate first and second.
const EVERY_FUSE: [(Rule, bool); 6] = [
    (Rule::Union, true),
    (Rule::Union, false),
    (Rule::Intersection, true),
    (Rule::Intersection, false),
    (Rule::Difference, true),
    (Rule::Difference, false),
];

/// Each block `[x0, y0, y1]` runs from `(x0, y0)` to `(12.4, y1)`, and its
/// corner `(12.4, 8.2)` is the only one on the rim.
const BLOCKS: [[f64; 3]; 3] = [[11.4, 8.2, 9.2], [11.9, 8.2, 8.7], [11.2, 8.2, 9.4]];

/// A block whose corners `(12.4, 8.2)` and `(7.6, 11.8)` are both on the rim.
const A_BLOCK_CORNERED_TWICE: [f64; 3] = [7.6, 8.2, 11.8];

/// Each block, set on the plate's top or under its floor, fused with the plate
/// by every rule, plate first and second.
#[test]
fn a_block_cornered_on_a_round_tabs_rim_is_never_refused_on_which_rim_crossing_comes_first() {
    let mut blocks = BLOCKS.to_vec();
    blocks.push(A_BLOCK_CORNERED_TWICE);
    let wrong = what_the_fuses_get_wrong(
        &blocks,
        &EVERY_FUSE,
        &Policy::certified(),
        Should::NotBeRefusedOnTheOrderAlongACrossing,
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Under a certified policy the block less the plate is the block, and the
/// two have nothing in common, since they touch along a face.
///
/// All 18 were refused `Undecidable` at the sliver of the block's face between
/// the corner and the rim until `by_a_corner` asked the corner (Q2243).
#[test]
fn a_block_cornered_on_a_round_tabs_rim_less_the_plate_and_their_common_part_answer_under_a_certified_policy()
 {
    let wrong = what_the_fuses_get_wrong(
        &BLOCKS,
        &[
            (Rule::Intersection, true),
            (Rule::Intersection, false),
            (Rule::Difference, false),
        ],
        &Policy::certified(),
        Should::Answer,
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Under a certified policy the union holds the sliver of the block's floor or
/// roof between the corner and the rim, which is thinner than a rounding of
/// `12.4`. A face that thin is read as one rounding thick, and the model's
/// largest coordinate, `13`, over that is past what `f64` resolves: a limit
/// (§437.6).
#[test]
fn a_block_cornered_on_a_round_tabs_rim_united_with_the_plate_is_refused_for_a_sliver_thinner_than_a_rounding()
 {
    let wrong = what_the_fuses_get_wrong(
        &BLOCKS,
        &[(Rule::Union, true), (Rule::Union, false)],
        &Policy::certified(),
        Should::BeRefusedAsTooWideARangeToComputeIn,
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Under a certified policy the plate less the block is refused as the union
/// is, though its answer is the plate itself, which holds no sliver: a gap
/// (§16.10), filed from Q2243. When it answers, move these fuses into
/// `a_block_cornered_on_a_round_tabs_rim_less_the_plate_and_their_common_part_answer_under_a_certified_policy`'s
/// list.
#[test]
fn the_plate_less_a_block_cornered_on_its_round_tabs_rim_is_refused_though_its_answer_is_the_plate()
{
    let wrong = what_the_fuses_get_wrong(
        &BLOCKS,
        &[(Rule::Difference, true)],
        &Policy::certified(),
        Should::BeRefusedAsTooWideARangeToComputeIn,
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Under a tolerant policy the two rim crossings a rounding apart are one
/// point, the corner is on the rim, and each block cornered on it once answers
/// with its volume.
#[test]
fn a_block_cornered_on_a_round_tabs_rim_answers_with_its_volume_when_the_corner_is_read_as_on_it() {
    let wrong = what_the_fuses_get_wrong(
        &BLOCKS,
        &EVERY_FUSE,
        &Policy::tolerant(100.0),
        Should::Answer,
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The block's other corner `(7.6, 11.8)` is on the rim too, and every fuse is
/// refused at the sliver of the block's face there, under either policy. The
/// ray from that corner meets the tab's wall, a round face whose top and
/// bottom are each three arcs of where the square and the bar crossed, and
/// `holds.rs` cannot follow it (Q2231). When Q2231 lands, these fuses should
/// answer as the other blocks do.
#[test]
fn a_block_cornered_twice_on_a_round_tabs_rim_is_refused_at_its_second_corner_under_either_policy()
{
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        let wrong = what_the_fuses_get_wrong(
            &[A_BLOCK_CORNERED_TWICE],
            &EVERY_FUSE,
            &policy,
            Should::BeRefusedAtTheSliverAtItsSecondCorner,
        );
        assert!(wrong.is_empty(), "{policy:?}:\n{}", wrong.join("\n"));
    }
}

/// What a fuse of a block with the plate should come to.
#[derive(Clone, Copy)]
enum Should {
    /// Anything but a refusal on which rim crossing comes first.
    NotBeRefusedOnTheOrderAlongACrossing,
    /// One solid that measures what it should, or none.
    Answer,
    /// `DynamicRangeTooWide`.
    BeRefusedAsTooWideARangeToComputeIn,
    /// `Undecidable` at a point inside a piece of a face with a corner at
    /// `(7.6, 11.8)` on the face of the plate the block touches.
    BeRefusedAtTheSliverAtItsSecondCorner,
}

/// Each of `blocks`, set on the plate's top or under its floor, fused with the
/// plate by each of `fuses` (a rule and whether the plate is first) under
/// `policy`: one line for each fuse that does not come to what it `should`.
fn what_the_fuses_get_wrong(
    blocks: &[[f64; 3]],
    fuses: &[(Rule, bool)],
    policy: &Policy,
    should: Should,
) -> Vec<String> {
    let mut wrong = Vec::new();
    for &[x0, y0, y1] in blocks {
        for [z0, z1] in [[2.0, 3.0], [-1.0, 0.0]] {
            for &(rule, plate_first) in fuses {
                let mut model =
                    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                let plate = a_plate_with_a_round_tab(&mut model);
                let block = a_block(&mut model, [x0, y0, z0], [12.4, y1, z1]);
                let (arguments, tools) = if plate_first {
                    ([plate], [block])
                } else {
                    ([block], [plate])
                };
                let fused = model.fuse(&arguments, &tools, rule, policy, &mut ProvenanceLog::new());
                let what = match should {
                    Should::NotBeRefusedOnTheOrderAlongACrossing => {
                        is_not_refused_on_the_order_along_a_crossing(&fused)
                    }
                    Should::Answer => {
                        let plate_holds = 2.0 * (100.0 + 6.75 * core::f64::consts::PI);
                        let block_holds = (12.4 - x0) * (y1 - y0) * (z1 - z0);
                        let should_hold = match (rule, plate_first) {
                            (Rule::Union, _) => Some(plate_holds + block_holds),
                            (Rule::Intersection, _) => None,
                            (Rule::Difference, true) => Some(plate_holds),
                            (Rule::Difference, false) => Some(block_holds),
                            (other, _) => panic!("no volume for {other:?}"),
                        };
                        fused
                            .map_err(|why| format!("refused {why:?}"))
                            .and_then(|fused| {
                                what_it_holds(&model, fused.solids(), should_hold, policy)
                            })
                    }
                    Should::BeRefusedAsTooWideARangeToComputeIn => {
                        is_refused_as_too_wide_a_range(&fused)
                    }
                    Should::BeRefusedAtTheSliverAtItsSecondCorner => {
                        let touching = if z0 < 0.0 { z1 } else { z0 };
                        is_refused_at_a_piece_with_this_corner(
                            &fused,
                            PredicatePoint::Exact([7.6, 11.8, touching]),
                        )
                    }
                };
                if let Err(why) = what {
                    wrong.push(format!(
                        "x {x0} to 12.4, y {y0} to {y1}, z {z0} to {z1}, {rule:?}, \
                         plate first {plate_first}: {why}"
                    ));
                }
            }
        }
    }
    wrong
}

fn is_not_refused_on_the_order_along_a_crossing(
    fused: &rcad_topo::Result<Fused>,
) -> Result<(), String> {
    match fused {
        Err(why) if format!("{why:?}").contains("order along a crossing") => {
            Err(format!("refused {why:?}"))
        }
        Ok(_) | Err(_) => Ok(()),
    }
}

fn is_refused_as_too_wide_a_range(fused: &rcad_topo::Result<Fused>) -> Result<(), String> {
    match fused {
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { .. })) => Ok(()),
        Err(why) => Err(format!("refused {why:?}")),
        Ok(fused) => Err(format!("answered with {} solids", fused.solids().len())),
    }
}

fn is_refused_at_a_piece_with_this_corner(
    fused: &rcad_topo::Result<Fused>,
    corner: PredicatePoint,
) -> Result<(), String> {
    match fused {
        Err(Error::Undecidable { records })
            if records.records().iter().any(|record| {
                record.predicate() == INSIDE_A_PIECE && record.inputs().contains(&corner)
            }) =>
        {
            Ok(())
        }
        Err(why) => Err(format!("refused {why:?}")),
        Ok(fused) => Err(format!("answered with {} solids", fused.solids().len())),
    }
}

/// Whether `solids` is one solid that validates and measures `should_hold`
/// within its error, or none where `should_hold` is `None`.
fn what_it_holds(
    model: &Model,
    solids: &[SolidKey],
    should_hold: Option<f64>,
    policy: &Policy,
) -> Result<(), String> {
    model
        .validate(policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("does not validate: {why:?}"))?;
    let (&[solid], Some(should_hold)) = (solids, should_hold) else {
        return match (solids.len(), should_hold) {
            (0, None) => Ok(()),
            (count, _) => Err(format!("{count} solids")),
        };
    };
    let measured = model
        .solid(solid)
        .expect("the fuse made the key")
        .measure(
            Deflection::of(0.01).expect("a length"),
            policy,
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("does not measure: {why:?}"))?;
    if (measured.volume() - should_hold).abs() > measured.volume_error() {
        return Err(format!(
            "measured {} ± {}, and it should hold {should_hold}",
            measured.volume(),
            measured.volume_error()
        ));
    }
    Ok(())
}

fn a_plate_with_a_round_tab(model: &mut Model) -> SolidKey {
    let square = a_block(model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
    let tab = model
        .add_round_bar(
            3.0,
            2.0,
            &Isometry::translation(Vector::new(10.0, 10.0, 0.0)).expect("a finite shift"),
        )
        .expect("a bar");
    let fused = model
        .fuse(
            &[square],
            &[tab],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the square and the tab unite");
    assert_eq!(fused.solids().len(), 1, "the plate is one solid");
    fused.solids()[0]
}

fn a_block(model: &mut Model, [x0, y0, z0]: [f64; 3], [x1, y1, z1]: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    model
        .add_extrusion(
            &rectangle,
            z1 - z0,
            &Isometry::translation(Vector::new(0.0, 0.0, z0)).expect("a finite shift"),
        )
        .expect("a rectangle extrudes")
}
