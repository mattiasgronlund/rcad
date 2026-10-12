//! A box whose top corner, where three planes meet, is moved in its file, read
//! back, and fused. `docs/decisions.md` §294.16 is the record and this file is
//! its measurement.
//!
//! §294 moved a round bar's top corner, which has only two surfaces round it,
//! and found no wrong `Ok` and no panic
//! (`a_broken_two_surface_corner_read_back_from_bytes.rs`). §294.7 left a corner
//! with three surfaces round it untried. Here it is the top corner of a box, `x`
//! and `y` from -3 to 3 and `z` from 0 to 10, where the planes `x = 3`, `y = 3`
//! and `z = 10` meet. The file moves it a millimetre four ways, at four
//! placements, and fuses each moved box against §294.2's partners and rules,
//! under `certified` and under `tolerant(100)`.
//!
//! **Every one of those fuses refuses with `Error::Undecidable`, one record
//! named "where three surfaces meet", and leaves the model as it was.** None is
//! a wrong `Ok` and none panics. Unlike the bar's, the box's moved corner is
//! refused even where the partner cuts it away, and even where the partner is
//! nowhere near. Before a fuse looks at the partner, it works out every corner
//! of every input from the surfaces round it. A corner with three planes round
//! it is where the three planes cross, and the corner the file stores only
//! says which crossing it is. A stored corner further from the crossing than
//! `validate` lets a corner sit off its plane says none, nothing else places
//! the corner, and the fuse gives up.
//!
//! That refusal names the arithmetic and not the file: `validate` is what says
//! the file is broken, `VertexOffItsFacePlane { distance: 1.0 }`. §294.16 says
//! why the fuse does not say it too.
//!
//! The corner is moved in the bytes because nothing in the public API writes a
//! corner off its surfaces: every builder validates what it built.

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector, WHERE_THREE_SURFACES_MEET};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

/// The box's footprint runs from `-SIDE` to `SIDE` in `x` and in `y`.
const SIDE: f64 = 3.0;
const HEIGHT: f64 = 10.0;

/// Where the box puts the corner the file moves: the top corner on the `+x`,
/// `+y` side, where the planes `x = 3`, `y = 3` and `z = 10` meet.
const WHERE_THE_BOX_PUTS_IT: [f64; 3] = [SIDE, SIDE, HEIGHT];

/// A millimetre out from the `x = 3` wall, still on the other wall and the top.
const OFF_A_WALL: [f64; 3] = [SIDE + 1.0, SIDE, HEIGHT];

/// A millimetre above the top, still on both walls.
const OFF_THE_TOP: [f64; 3] = [SIDE, SIDE, HEIGHT + 1.0];

/// A millimetre into the box from each of its three planes.
const INTO_THE_BOX: [f64; 3] = [SIDE - 1.0, SIDE - 1.0, HEIGHT - 1.0];

/// A millimetre out from each of its three planes.
const OFF_ALL_THREE: [f64; 3] = [SIDE + 1.0, SIDE + 1.0, HEIGHT + 1.0];

const EVERY_MOVE: [[f64; 3]; 4] = [OFF_A_WALL, OFF_THE_TOP, INTO_THE_BOX, OFF_ALL_THREE];

/// A block, as the two corners of its footprint, the height of its floor and
/// how tall it stands. The blocks are §294.2's and keep its names, which speak
/// of the bar's seam: the box has no seam, and the bar's seam corner and the
/// box's moved corner are both on the `+x` side.
#[derive(Clone, Copy, Debug)]
struct Block {
    low: [f64; 2],
    high: [f64; 2],
    floor: f64,
    tall: f64,
}

/// Nowhere near the box.
const FAR_AWAY: Block = Block {
    low: [20.0, 20.0],
    high: [22.0, 22.0],
    floor: 20.0,
    tall: 2.0,
};

/// Over the bottom two millimetres of the box.
const OVER_THE_BOTTOM: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: -1.0,
    tall: 3.0,
};

/// Over the top two millimetres of the box, and the moved corner with them.
const OVER_THE_TOP: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// Over the top two millimetres of the `x > 2` side, which holds the corner.
const OVER_THE_RIM_BY_THE_SEAM: Block = Block {
    low: [2.0, -5.0],
    high: [5.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// Over the top two millimetres of the `x < -2` side, away from the corner.
const OVER_THE_RIM_AWAY_FROM_THE_SEAM: Block = Block {
    low: [-5.0, -5.0],
    high: [-2.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// A notch in the top of the `y = 3` wall, halfway along it.
const A_NOTCH_A_QUARTER_TURN_ROUND: Block = Block {
    low: [-1.0, 2.0],
    high: [1.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// The whole `y > 0` half of the box, the corner with it.
const THE_HALF_PAST_THE_SEAM: Block = Block {
    low: [-5.0, 0.0],
    high: [5.0, 5.0],
    floor: -1.0,
    tall: 12.0,
};

/// §294.2's ten partners and rules, and `OVER_THE_TOP` kept as well, which
/// cuts the corner away.
const EVERY_PARTNER: [(Block, Rule); 11] = [
    (FAR_AWAY, Rule::Union),
    (OVER_THE_BOTTOM, Rule::Difference),
    (OVER_THE_TOP, Rule::Intersection),
    (OVER_THE_TOP, Rule::Difference),
    (OVER_THE_RIM_BY_THE_SEAM, Rule::Difference),
    (OVER_THE_RIM_BY_THE_SEAM, Rule::Union),
    (OVER_THE_RIM_BY_THE_SEAM, Rule::Intersection),
    (OVER_THE_RIM_AWAY_FROM_THE_SEAM, Rule::Difference),
    (A_NOTCH_A_QUARTER_TURN_ROUND, Rule::Difference),
    (THE_HALF_PAST_THE_SEAM, Rule::Difference),
    (THE_HALF_PAST_THE_SEAM, Rule::Intersection),
];

/// Each move takes the corner a millimetre off at least one of its planes, and
/// `validate` says so under both policies at every placement. The same bytes
/// with the corner left alone validate.
#[test]
fn a_box_whose_top_corner_is_moved_a_millimetre_in_its_file_reads_back_and_does_not_validate() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for placement in every_placement() {
            for moved_to in EVERY_MOVE {
                let (model, _) = read_back(placement, moved_to);
                let case = format!("{policy:?}, {placement:?}, the corner at {moved_to:?}");
                match model.validate(&policy, &mut ProvenanceLog::new()) {
                    Err(Error::Invalid(found)) => assert_eq!(
                        found,
                        Invalid::VertexOffItsFacePlane { distance: 1.0 },
                        "{case}"
                    ),
                    other => panic!("{case} validates as {other:?}"),
                }
            }
            let (sound, _) = read_back(placement, WHERE_THE_BOX_PUTS_IT);
            sound
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|error| {
                    panic!("{policy:?}, {placement:?}: the sound box does not validate: {error:?}")
                });
        }
    }
}

/// The control for the refusals below: the same bytes with the corner left
/// alone answer every partner, and the answer holds what box arithmetic says
/// the two boxes hold. So what refuses the moved box is the moved corner, not
/// the partner, the rule or the placement.
#[test]
fn the_sound_box_answers_every_partner_with_what_the_two_boxes_hold() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for placement in every_placement() {
            for (partner, rule) in EVERY_PARTNER {
                let case = format!("{policy:?}, {placement:?}, {partner:?} {rule:?}");
                let (mut model, solid) = read_back(placement, WHERE_THE_BOX_PUTS_IT);
                let block = a_block(&mut model, partner, &placement);
                let answer = model
                    .fuse(&[solid], &[block], rule, &policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|error| panic!("{case} refused: {error:?}"));
                model
                    .validate(&policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|error| {
                        panic!("{case}: the answer does not validate: {error:?}")
                    });

                let (solids, volume) = what_the_two_boxes_hold(partner, rule);
                assert_eq!(answer.solids().len(), solids, "{case}: how many solids");
                let measured: f64 = answer
                    .solids()
                    .iter()
                    .map(|&piece| {
                        model
                            .solid(piece)
                            .expect("a solid the fuse just wrote")
                            .measure(
                                Deflection::for_scale(model.scale()),
                                &policy,
                                &mut ProvenanceLog::new(),
                            )
                            .unwrap_or_else(|error| panic!("{case}: measuring refused: {error:?}"))
                            .volume()
                    })
                    .sum();
                assert!(
                    (measured - volume).abs() <= 1e-12 * volume,
                    "{case}: the answer holds {measured}, the boxes {volume}"
                );
            }
        }
    }
}

/// **Every partner, every rule, every placement and both policies give the
/// same refusal**: `Undecidable`, with one record, "where three surfaces meet",
/// and the model byte for byte as it was. That includes the partner far away
/// and the partners that cut the corner away, which the bar answered (§294.2):
/// the box's corners are all worked out before the partner is looked at, and
/// the moved one is too far from where its three planes cross to say that it
/// is that crossing.
#[test]
fn every_fuse_of_a_box_whose_top_corner_is_moved_refuses_where_three_surfaces_meet() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for placement in every_placement() {
            for moved_to in EVERY_MOVE {
                for (partner, rule) in EVERY_PARTNER {
                    let case = format!(
                        "{policy:?}, {placement:?}, the corner at {moved_to:?}, {partner:?} {rule:?}"
                    );
                    let (mut model, solid) = read_back(placement, moved_to);
                    let block = a_block(&mut model, partner, &placement);
                    let before = postcard::to_allocvec(&model).expect("a model writes out");
                    match model.fuse(&[solid], &[block], rule, &policy, &mut ProvenanceLog::new()) {
                        Err(Error::Undecidable { records }) => {
                            let named: Vec<&str> = records
                                .records()
                                .iter()
                                .map(|record| record.predicate())
                                .collect();
                            assert_eq!(named, [WHERE_THREE_SURFACES_MEET], "{case}");
                        }
                        other => panic!("{case} came back {other:?}"),
                    }
                    assert_eq!(
                        postcard::to_allocvec(&model).expect("a model writes out"),
                        before,
                        "{case}: a refused fuse leaves the model as it was"
                    );
                }
            }
        }
    }
}

fn every_placement() -> [Isometry; 4] {
    let shift = Isometry::translation(Vector::new(7.0, -5.0, 2.5)).expect("a plain translation");
    let turn = Isometry::rotation(&Axis::x(), 0.4).expect("a turn about x");
    [Isometry::identity(), shift, turn, shift.compose(&turn)]
}

/// The box at `placement`, written out with postcard, its corner overwritten in
/// the bytes with `moved_to`, and read back.
///
/// The placement goes on the box's shell, so the file stores the corner at
/// [`WHERE_THE_BOX_PUTS_IT`] at every placement, and `moved_to` is in that
/// stored frame too.
fn read_back(placement: Isometry, moved_to: [f64; 3]) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let footprint = ProfileBuilder::new(vec![
        [-SIDE, -SIDE],
        [SIDE, -SIDE],
        [SIDE, SIDE],
        [-SIDE, SIDE],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    model
        .add_extrusion(&footprint, HEIGHT, &placement)
        .expect("a box");
    let mut bytes = postcard::to_allocvec(&model).expect("a model writes out");

    let stored = the_bytes_of(WHERE_THE_BOX_PUTS_IT);
    let places: Vec<usize> = bytes
        .windows(stored.len())
        .enumerate()
        .filter(|(_, window)| *window == stored.as_slice())
        .map(|(at, _)| at)
        .collect();
    let [at] = places.as_slice() else {
        panic!("the corner is written once, and was found at {places:?}");
    };
    bytes[*at..*at + stored.len()].copy_from_slice(&the_bytes_of(moved_to));

    let read_back: Model =
        postcard::from_bytes(&bytes).expect("reading back does not ask where a corner is");
    let solid = read_back
        .solids()
        .next()
        .expect("the solid came back")
        .key();
    (read_back, solid)
}

fn the_bytes_of(point: [f64; 3]) -> Vec<u8> {
    point
        .iter()
        .flat_map(|coordinate| coordinate.to_le_bytes())
        .collect()
}

fn a_block(model: &mut Model, block: Block, placement: &Isometry) -> SolidKey {
    let [wide, deep] = [block.high[0] - block.low[0], block.high[1] - block.low[1]];
    let footprint = ProfileBuilder::new(vec![[0.0, 0.0], [wide, 0.0], [wide, deep], [0.0, deep]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let own = Isometry::translation(Vector::new(block.low[0], block.low[1], block.floor))
        .expect("a plain translation");
    model
        .add_extrusion(&footprint, block.tall, &placement.compose(&own))
        .expect("a block")
}

/// How many solids and how much volume the sound box and `partner` give under
/// `rule`, from where the two overlap along each axis.
fn what_the_two_boxes_hold(partner: Block, rule: Rule) -> (usize, f64) {
    let the_box = (2.0 * SIDE) * (2.0 * SIDE) * HEIGHT;
    let the_block =
        (partner.high[0] - partner.low[0]) * (partner.high[1] - partner.low[1]) * partner.tall;
    let shared = overlap([-SIDE, SIDE], [partner.low[0], partner.high[0]])
        * overlap([-SIDE, SIDE], [partner.low[1], partner.high[1]])
        * overlap([0.0, HEIGHT], [partner.floor, partner.floor + partner.tall]);
    match rule {
        Rule::Union if shared == 0.0 => (2, the_box + the_block),
        Rule::Union => (1, the_box + the_block - shared),
        Rule::Intersection => {
            assert!(shared > 0.0, "no partner here leaves nothing in common");
            (1, shared)
        }
        Rule::Difference => (1, the_box - shared),
        _ => panic!("no partner here is fused under {rule:?}"),
    }
}

fn overlap(one: [f64; 2], other: [f64; 2]) -> f64 {
    (one[1].min(other[1]) - one[0].max(other[0])).max(0.0)
}
