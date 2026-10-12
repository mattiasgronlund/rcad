//! A box whose top corner, where three planes meet, is moved in its file, read
//! back, and fused. `docs/decisions.md` §294.16 is the record and this file is
//! its measurement.

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// Where the box puts the corner the file moves: the top corner on the `+x`,
/// `+y` side, where the planes `x = 3`, `y = 3` and `z = 10` meet.
const WHERE_THE_BOX_PUTS_IT: [f64; 3] = [3.0, 3.0, 10.0];

const EVERY_MOVE: [[f64; 3]; 4] = [
    [4.0, 3.0, 10.0],
    [3.0, 3.0, 11.0],
    [2.0, 2.0, 9.0],
    [4.0, 4.0, 11.0],
];

#[derive(Clone, Copy, Debug)]
struct Block {
    low: [f64; 2],
    high: [f64; 2],
    floor: f64,
    tall: f64,
}

const FAR_AWAY: Block = Block {
    low: [20.0, 20.0],
    high: [22.0, 22.0],
    floor: 20.0,
    tall: 2.0,
};
const OVER_THE_BOTTOM: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: -1.0,
    tall: 3.0,
};
const OVER_THE_TOP: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};
const OVER_THE_RIM_BY_THE_SEAM: Block = Block {
    low: [2.0, -5.0],
    high: [5.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};
const OVER_THE_RIM_AWAY_FROM_THE_SEAM: Block = Block {
    low: [-5.0, -5.0],
    high: [-2.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};
const A_NOTCH_A_QUARTER_TURN_ROUND: Block = Block {
    low: [-1.0, 2.0],
    high: [1.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};
const THE_HALF_PAST_THE_SEAM: Block = Block {
    low: [-5.0, 0.0],
    high: [5.0, 5.0],
    floor: -1.0,
    tall: 12.0,
};

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

#[test]
fn probe_what_comes_back() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for placement in every_placement() {
            for moved_to in core::iter::once(WHERE_THE_BOX_PUTS_IT).chain(EVERY_MOVE) {
                let (model, _) = read_back(placement, moved_to);
                let validated = model.validate(&policy, &mut ProvenanceLog::new());
                eprintln!("{policy:?} {placement:?} {moved_to:?} validate: {validated:?}");
                for (partner, rule) in EVERY_PARTNER {
                    let (mut model, solid) = read_back(placement, moved_to);
                    let block = a_block(&mut model, partner, &placement);
                    let mut log = ProvenanceLog::new();
                    let fused = model.fuse(&[solid], &[block], rule, &policy, &mut log);
                    let what = match &fused {
                        Ok(answer) => {
                            let valid = model.validate(&policy, &mut ProvenanceLog::new());
                            let volume: f64 = answer
                                .solids()
                                .iter()
                                .map(|&solid| {
                                    model
                                        .solid(solid)
                                        .unwrap()
                                        .measure(
                                            Deflection::for_scale(model.scale()),
                                            &policy,
                                            &mut ProvenanceLog::new(),
                                        )
                                        .unwrap()
                                        .volume()
                                })
                                .sum();
                            format!(
                                "Ok {} solids, volume {volume}, validate {:?}",
                                answer.solids().len(),
                                valid.map(|_| ())
                            )
                        }
                        Err(Error::Undecidable { records }) => format!(
                            "Undecidable {:?}",
                            records
                                .records()
                                .iter()
                                .map(|record| record.predicate())
                                .collect::<Vec<_>>()
                        ),
                        Err(other) => format!("{other:?}"),
                    };
                    eprintln!("  {partner:?} {rule:?}: {what}");
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

/// The box, written out with postcard, its corner overwritten in the bytes
/// with `moved_to`, and read back.
fn read_back(placement: Isometry, moved_to: [f64; 3]) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let footprint = ProfileBuilder::new(vec![[-3.0, -3.0], [3.0, -3.0], [3.0, 3.0], [-3.0, 3.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(&footprint, 10.0, &placement)
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
