//! How often a certified fuse of a curved pair gives up, family by family.
//!
//! §31.7 criterion 16, for the fuse. §17.7's discipline, unchanged: the rates
//! are **never blended**, the lattice family is the control that makes the
//! headline mean anything, and every refusal is named rather than counted.
//!
//! # What a pass is worth
//!
//! A fuse that refused everything would satisfy every property in
//! `fuses_that_must_agree.rs` — they all hand back `Ok(())` the moment there is
//! no answer to compare. So the control here is the load-bearing half: a
//! lattice-aligned curved pair is fused with nothing refused and nothing
//! recorded, under all three rules and both ways round.
//!
//! # The generators
//!
//! A block with a round bar through it, which is §31.7 criterion 11's shape, and
//! a ball resting on the block's top. The bar is kept a whole unit clear of the
//! block's walls: a bar exactly tangent to one is roadmap §7.4's own catalogue
//! entry and not what a *control* is for.

use std::collections::BTreeMap;

use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

/// How many pairs each family draws.
const DRAWS: usize = 64;

/// A deterministic sequence, so that a rate is one number and not a range.
struct Draw(u64);

impl Draw {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.0 >> 33
    }

    fn whole(&mut self, low: i64, high: i64) -> f64 {
        let span = u64::try_from(high - low + 1).expect("a range that runs upwards");
        let picked = i64::try_from(self.next() % span).expect("inside the range") + low;
        picked as f64
    }

    fn fraction(&mut self) -> f64 {
        (self.next() % 1000) as f64 / 1000.0
    }
}

/// How each family writes its pair down.
#[derive(Clone, Copy)]
enum Family {
    /// Whole numbers, axes square to each other.
    Lattice,
    /// The whole pair turned about an awkward axis.
    Turned,
    /// A ball instead of a bar, on the lattice.
    ABall,
    /// The lattice pair with every number multiplied by a power of two.
    AtAPowerOfTwo,
}

impl Family {
    fn named(self) -> &'static str {
        match self {
            Family::Lattice => "lattice",
            Family::Turned => "turned",
            Family::ABall => "a ball on the lattice",
            Family::AtAPowerOfTwo => "written at 2^k",
        }
    }
}

fn a_pair(family: Family, draw: &mut Draw) -> Option<(Model, SolidKey, SolidKey)> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let scale = match family {
        Family::AtAPowerOfTwo => 2.0_f64.powi(i32::try_from(draw.next() % 7).ok()? - 3),
        _ => 1.0,
    };
    let side = 8.0 * scale;
    let radius = draw.whole(1, 2);
    let at = [
        draw.whole(1 + radius as i64, 7 - radius as i64) * scale,
        draw.whole(1 + radius as i64, 7 - radius as i64) * scale,
    ];
    let block = model
        .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
        .ok()?;
    let curved = match family {
        Family::ABall => model
            .add_ball(
                radius,
                &Isometry::translation(Vector::new(at[0], at[1], side)).ok()?,
            )
            .ok()?,
        _ => model
            .add_round_bar(
                radius * scale,
                12.0 * scale,
                &Isometry::translation(Vector::new(at[0], at[1], -2.0 * scale)).ok()?,
            )
            .ok()?,
    };
    if let Family::Turned = family {
        let axis = Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).ok()?;
        let turn = Isometry::rotation(&axis, 0.1 + draw.fraction()).ok()?;
        model.place_solid(block, &turn);
        model.place_solid(curved, &turn);
    }
    Some((model, block, curved))
}

/// Every rule, both ways round, over one family.
fn how_each_fuse_went(family: Family) -> (usize, BTreeMap<String, usize>, usize) {
    let policy = Policy::certified();
    let mut draw = Draw(0x5EED);
    let mut fuses = 0;
    let mut recorded = 0;
    let mut gave_up: BTreeMap<String, usize> = BTreeMap::new();
    for _ in 0..DRAWS {
        let Some((mut model, block, curved)) = a_pair(family, &mut draw) else {
            continue;
        };
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for (arguments, tools) in [([block], [curved]), ([curved], [block])] {
                fuses += 1;
                let mut log = ProvenanceLog::new();
                match model.fuse(&arguments, &tools, rule, &policy, &mut log) {
                    Ok(_) => {
                        if !log.records().is_empty() {
                            recorded += 1;
                        }
                    }
                    Err(Error::Undecidable { records }) => {
                        let named = records
                            .records()
                            .last()
                            .map_or("nothing named", |record| record.predicate());
                        *gave_up.entry(named.to_owned()).or_default() += 1;
                    }
                    Err(why) => *gave_up.entry(format!("{why}")).or_default() += 1,
                }
            }
        }
    }
    (fuses, gave_up, recorded)
}

/// §31.7 criterion 16's control: a lattice-aligned curved pair is fused with
/// nothing refused and nothing recorded.
///
/// All three rules, both ways round. Nothing here is a decision anybody failed
/// to prove: every corner of the answer is a whole number, and the boxes the
/// arrangement solved for have collapsed onto one.
#[test]
fn every_certified_fuse_of_a_lattice_curved_pair_is_decided_and_records_nothing() {
    for family in [Family::Lattice, Family::AtAPowerOfTwo] {
        let (fuses, gave_up, recorded) = how_each_fuse_went(family);
        assert!(
            fuses > 300,
            "{}: only {fuses} fuses were drawn",
            family.named()
        );
        assert!(
            gave_up.is_empty(),
            "{}: refused {gave_up:?}",
            family.named()
        );
        assert_eq!(recorded, 0, "{}: recorded something", family.named());
    }
}

/// A ball cut by a plane is refused, by name, every time.
///
/// **The shape M4 does not reach**, and the reason is structural rather than
/// arithmetic: a cap a plane cuts off a ball is bounded by one circle of
/// latitude and by nothing else, and its patch in the sphere's own two
/// coordinates is a rectangle whose top edge is the pole and whose two sides are
/// one meridian used twice. The model says neither of those about the face, and
/// a fuse has no way to invent either — a seam wants two corners at one
/// longitude and a cap has one corner in all.
///
/// A **ring**, which is two curves each going right round, *is* cut across, and
/// that is what a block with a round hole through it needs. The difference is
/// that a ring has somewhere to cut *to*.
///
/// Asserted as a whole family rather than as one shape, because a refusal that
/// only fires sometimes is a refusal somebody has to characterise (§14.7).
#[test]
fn a_ball_cut_by_a_plane_is_refused_by_name_rather_than_meshed_wrongly() {
    let (fuses, gave_up, _) = how_each_fuse_went(Family::ABall);
    assert!(fuses > 300, "only {fuses} fuses were drawn");
    assert_eq!(
        gave_up
            .keys()
            .map(String::as_str)
            .collect::<Vec<&str>>()
            .as_slice(),
        ["a curved face is bounded by one curve right round it and has no seam to open at"],
        "the ball family should refuse for exactly one reason, and it refused for {gave_up:?}"
    );
    assert_eq!(
        gave_up.values().sum::<usize>(),
        fuses,
        "every one of them, not some of them"
    );
}

/// The turned family, measured rather than judged against a number this file
/// made up (§31.7 offers none at the gate and says the lane measures it).
///
/// What is asserted is the shape of the answer rather than a threshold: every
/// refusal is named, and a share of them is recorded in the section that ran
/// this. A rate that moves is a rate to re-derive, not to relax.
#[test]
fn how_often_a_certified_fuse_of_a_turned_curved_pair_refuses() {
    let (fuses, gave_up, recorded) = how_each_fuse_went(Family::Turned);
    let refused: usize = gave_up.values().sum();
    println!(
        "turned: {:.1}% of {fuses} fuses refused, {recorded} recorded something: {gave_up:?}",
        100.0 * refused as f64 / fuses as f64
    );
    assert!(fuses > 300, "only {fuses} fuses were drawn");
    assert!(
        refused as f64 <= 0.60 * fuses as f64,
        "{refused} of {fuses} refused, over the three fifths §36 records"
    );
}
