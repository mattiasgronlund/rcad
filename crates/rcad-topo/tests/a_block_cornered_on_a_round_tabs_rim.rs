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
//! the plane of the plate's top or floor. It now asks a corner on the plane of
//! a flat face with a curved side, as those two faces are, since the ray hands
//! a point on that plane to the face's outline (Q2243). So the block less the
//! plate and the two's common part answer. The plate less the block answers
//! too, once the tab's wall, cut by the block's walls a rounding apart, is
//! joined back up across the arc a rounding long between the two cuts (Q2296).
//! A tolerant policy reads the two crossings as one point, and there every
//! fuse of a block cornered on the rim once answers.
//!
//! The block `x 7.6 to 12.4` by `y 8.2 to 11.8` has its other corner `(7.6,
//! 11.8)` on the rim too. On the plate's top, under a certified policy, the
//! block less the plate, the plate less it and their common part were refused
//! `Undecidable` at `side of cylinder` for a point a rounding from that corner,
//! over the records of a ray the classification threw away (Q2477). No answer
//! rests on such a ray, and since Q2489 its records are kept out of the log, so
//! the three answer: one of the two solids as it stands, or nothing, since they
//! touch along a face.
//!
//! What is still refused, and whether its answer exists (§16.10, §437.6):
//!
//! - **The union, a limit.** It holds the sliver of the block's floor or roof
//!   outside the rim, a face thinner than a rounding of its own coordinates,
//!   so it is refused `DynamicRangeTooWide`.
//! - **That block's union under a tolerant policy, not yet traced.** The mesh
//!   refuses it, a wire that does not fan into a loop with the block on the
//!   plate's top and two patches drawing the same side with it under the
//!   floor. Q2478 traces it and says whether it is a gap.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Invalid, Model, ProfileBuilder, Rule, SolidKey};

/// Every rule, with the plate first and second.
const EVERY_FUSE: [(Rule, bool); 6] = [
    (Rule::Union, true),
    (Rule::Union, false),
    (Rule::Intersection, true),
    (Rule::Intersection, false),
    (Rule::Difference, true),
    (Rule::Difference, false),
];

/// The union, with the plate first and second.
const THE_UNIONS: [(Rule, bool); 2] = [(Rule::Union, true), (Rule::Union, false)];

/// Every rule but the union, with the plate first and second.
const BUT_THE_UNIONS: [(Rule, bool); 4] = [
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
        [Should::NotBeRefusedOnTheOrderAlongACrossing; 2],
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Under a certified policy the block less the plate is the block, the plate
/// less the block is the plate, and the two have nothing in common, since they
/// touch along a face.
///
/// All 18 of the block less the plate and the common part were refused
/// `Undecidable` at the sliver of the block's face between the corner and the
/// rim until `by_a_corner` asked the corner (Q2243). The 6 of the plate less
/// the block were then refused `DynamicRangeTooWide`, since the tab's wall was
/// left in its 9 pieces: the walls `x = 12.4` and `y = 8.2` cut it a rounding
/// apart, and the ring of the wall joined back up was not counted round its
/// axis for the arc a rounding long between them (Q2296).
#[test]
fn a_block_cornered_on_a_round_tabs_rim_less_the_plate_and_their_common_part_answer_under_a_certified_policy()
 {
    let wrong = what_the_fuses_get_wrong(
        &BLOCKS,
        &BUT_THE_UNIONS,
        &Policy::certified(),
        [Should::Answer; 2],
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
        &THE_UNIONS,
        &Policy::certified(),
        [Should::BeRefusedAsTooWideARangeToComputeIn; 2],
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
        [Should::Answer; 2],
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The block's other corner `(7.6, 11.8)` is on the rim too, and its 24 fuses,
/// on the plate's top or under its floor, under either policy, come to this:
///
/// - Under a certified policy the union is refused `DynamicRangeTooWide`, as
///   each block cornered once is: it has the same corner `(12.4, 8.2)` on the
///   rim.
/// - Under a certified policy the block less the plate, the plate less the
///   block and their common part answer, on the plate's top and under its
///   floor. On the top they were refused `Undecidable` at `side of cylinder`
///   for a point a rounding from `(7.6, 11.8, 2)`, over the records of a ray
///   the classification threw away, until Q2489 kept those out of the log
///   (Q2477).
/// - Under a tolerant policy the union is refused by the mesh: a wire that
///   does not fan into a loop on the plate's top, and two patches drawing the
///   same side under its floor (Q2478). The rest answer.
///
/// Until Q2405 every one of them was refused at the sliver of the block's face
/// at `(7.6, 11.8)`, under either policy. The ray from that corner meets the
/// tab's wall, a round face whose top and bottom are each three arcs of where
/// the square and the bar crossed, and `holds.rs` could not follow it (Q2231).
/// It now counts that face along the ruling through the point.
///
/// Under the plate's floor the certified fuses were refused first at the piece
/// of the block's floor under the square's corner `(10, 10)`, a whole unit
/// below the plate. Its corner `(7.6, 8.2)` is cut off by an arc a rounding
/// long, the turn about that arc's chord cannot be settled, and asked it the
/// piece held no point. Such a turn now counts nothing, and the piece holds
/// its own points (Q2297).
#[test]
fn what_each_fuse_of_a_block_cornered_twice_on_a_round_tabs_rim_comes_to() {
    let certified = Policy::certified();
    let tolerant = Policy::tolerant(100.0);
    let mut wrong = Vec::new();
    for (policy, fuses, on_top, under) in [
        (
            &certified,
            &THE_UNIONS[..],
            Should::BeRefusedAsTooWideARangeToComputeIn,
            Should::BeRefusedAsTooWideARangeToComputeIn,
        ),
        (
            &certified,
            &BUT_THE_UNIONS[..],
            Should::Answer,
            Should::Answer,
        ),
        (
            &tolerant,
            &THE_UNIONS[..],
            Should::BeRefusedForAWireThatDoesNotFanIntoALoop,
            Should::BeRefusedForTwoPatchesOfTheMeshDrawingTheSameSide,
        ),
        (
            &tolerant,
            &BUT_THE_UNIONS[..],
            Should::Answer,
            Should::Answer,
        ),
    ] {
        for line in
            what_the_fuses_get_wrong(&[A_BLOCK_CORNERED_TWICE], fuses, policy, [on_top, under])
        {
            wrong.push(format!("{policy:?}: {line}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
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
    /// `AWireDoesNotFanIntoALoopToMesh`.
    BeRefusedForAWireThatDoesNotFanIntoALoop,
    /// `TwoPatchesOfTheMeshDrewTheSameSide`.
    BeRefusedForTwoPatchesOfTheMeshDrawingTheSameSide,
}

/// Each of `blocks`, set on the plate's top or under its floor, fused with the
/// plate by each of `fuses` (a rule and whether the plate is first) under
/// `policy`: one line for each fuse that does not come to what it `should`,
/// on the plate's top and under its floor, in that order.
fn what_the_fuses_get_wrong(
    blocks: &[[f64; 3]],
    fuses: &[(Rule, bool)],
    policy: &Policy,
    should: [Should; 2],
) -> Vec<String> {
    let mut wrong = Vec::new();
    for &[x0, y0, y1] in blocks {
        for ([z0, z1], should) in [[2.0, 3.0], [-1.0, 0.0]].into_iter().zip(should) {
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
                    Should::BeRefusedForAWireThatDoesNotFanIntoALoop => match &fused {
                        Err(Error::AWireDoesNotFanIntoALoopToMesh { .. }) => Ok(()),
                        other => Err(what_it_came_to_instead(other)),
                    },
                    Should::BeRefusedForTwoPatchesOfTheMeshDrawingTheSameSide => match &fused {
                        Err(Error::TwoPatchesOfTheMeshDrewTheSameSide { .. }) => Ok(()),
                        other => Err(what_it_came_to_instead(other)),
                    },
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
        other => Err(what_it_came_to_instead(other)),
    }
}

fn what_it_came_to_instead(fused: &rcad_topo::Result<Fused>) -> String {
    match fused {
        Err(why) => format!("refused {why:?}"),
        Ok(fused) => format!("answered with {} solids", fused.solids().len()),
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
