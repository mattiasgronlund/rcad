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
        Family::Lattice | Family::Turned | Family::ABall => 1.0,
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
        Family::Lattice | Family::Turned | Family::AtAPowerOfTwo => model
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
        model.place_solid(block, &turn).ok()?;
        model.place_solid(curved, &turn).ok()?;
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
    for family in [Family::Lattice, Family::ABall, Family::AtAPowerOfTwo] {
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

/// The turned family, measured rather than judged against a number this file
/// made up (§31.7 offers none at the gate and says the lane measures it).
///
/// **Judged at zero, and §39 is where that number is re-derived.** The ceiling
/// here was three fifths, written at §36.14 when the ball family beside it
/// refused every fuse; §36.21 took that family to none and left the ceiling
/// where it was. A threshold of 60% over a measurement of 0.00% catches nothing
/// short of a catastrophe, and "a rate that moves is a rate to re-derive, not to
/// relax" cuts both ways — a rate that falls to zero wants a threshold that
/// follows it down (§19.1 is the precedent).
///
/// Zero is not a guess: [`Draw`] runs from a fixed seed, so this is 384 fuses
/// that come out the same on every machine, and every one of them is decided.
/// The headline now says exactly what the control says, and that is the honest
/// state of affairs rather than a weakness in the test — no draw of *this*
/// generator reaches a shape that refuses, and the shapes that do are
/// hand-written in `degenerate_curved_fuses.rs`. Widening the generator until it
/// reaches one is what would make a rate of this family mean anything, and it is
/// the day this number gets re-derived again.
#[test]
fn how_often_a_certified_fuse_of_a_turned_curved_pair_refuses() {
    let (fuses, gave_up, recorded) = how_each_fuse_went(Family::Turned);
    let refused: usize = gave_up.values().sum();
    println!(
        "turned: {:.1}% of {fuses} fuses refused, {recorded} recorded something: {gave_up:?}",
        100.0 * refused as f64 / fuses as f64
    );
    assert!(fuses > 300, "only {fuses} fuses were drawn");
    assert_eq!(
        refused, 0,
        "the turned family is measured at none refused, and this is that measurement"
    );
    assert_eq!(recorded, 0, "nor did any of them record a decision");
}

/// A block notching a bar, turned against it rather than with it, and every
/// answer put through the sweep.
///
/// **This is the family §39's claim audit found missing, and the defect it
/// found.** Every other family in this file turns both solids by one rotation,
/// which is an affine map of the pair: the block's cut planes stay square to the
/// bar's own frame, every corner of every answer lands on a coordinate `f64`
/// holds exactly, and `check_edges`'s demand that a corner read *exactly*
/// `Decision::Zero` against the surface across the edge is met by luck. Turn the
/// block on its own and a corner is `through + t·direction` over a divisor, one
/// ulp off the plane it was solved from — and the fuse wrote a solid its own
/// sweep called broken: `Ok` with a corrupt answer in a release build, a panic
/// in a debug one.
///
/// So this asserts the sweep and not a rate: a fuse either refuses by name or
/// hands back a model that passes [`Model::validate`]. The count that answered
/// is printed and held to every fuse answered (Q1228), the measurement made
/// since a fold in `validate.rs` got its own floor (§188.3): a family that
/// refused even one would sweep less than it does now and the count would say
/// so.
#[test]
fn every_answer_over_a_block_turned_against_a_bar_is_a_solid_that_validates() {
    let policy = Policy::certified();
    let mut draw = Draw(0x5EED);
    let mut fuses = 0;
    let mut answered = 0;
    let mut gave_up: BTreeMap<String, usize> = BTreeMap::new();
    for _ in 0..DRAWS {
        let Some((mut model, bar, block)) = a_bar_notched_by_a_turned_block(&mut draw) else {
            continue;
        };
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for (arguments, tools) in [([bar], [block]), ([block], [bar])] {
                fuses += 1;
                let mut log = ProvenanceLog::new();
                match model.fuse(&arguments, &tools, rule, &policy, &mut log) {
                    Ok(_) => {
                        answered += 1;
                        model
                            .validate(&policy, &mut ProvenanceLog::new())
                            .expect("a fuse hands back a sound model or refuses");
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
    println!(
        "a block turned against a bar: {answered} of {fuses} fuses answered, refused {gave_up:?}"
    );
    assert!(fuses > 300, "only {fuses} fuses were drawn");
    assert_eq!(
        answered, fuses,
        "this family is measured at every fuse answered, and this is that measurement"
    );
}

/// A unit block cutting a notch out of a bar's wall, turned about the bar's own
/// axis so that its cut planes are oblique to everything the bar is written in.
///
/// The block has to be *small* and sit where the bar's wall is: a block the bar
/// runs right through is cut by two caps square to the axis whatever the block
/// is turned by, and a corner of that is exact again.
fn a_bar_notched_by_a_turned_block(draw: &mut Draw) -> Option<(Model, SolidKey, SolidKey)> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model.add_round_bar(1.0, 4.0, &Isometry::identity()).ok()?;
    let block = model
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .ok()?;
    let up = 3.0 + draw.fraction();
    let across = Isometry::translation(Vector::new(0.5, -0.5, up)).ok()?;
    let turn = Isometry::rotation(&Axis::z(), 0.05 + draw.fraction()).ok()?;
    model.place_solid(block, &across.compose(&turn)).ok()?;
    Some((model, bar, block))
}

/// A bar notched by a block, which is the shape §35.10 hands lane 5 as the one
/// that makes a face trimmed by a *piece* of a crossing.
///
/// The wall of the bar ends up bounded by a stretch of the block's planes and
/// then stops. Naming which stretch was `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed`
/// until §36.23, and what closes it is the half-space through the chord square
/// to the plane the crossing lies in — which needs neither plane built.
///
/// What this file asserts about it is the *rate*: a certified fuse of it decides
/// everything and records nothing, the same claim the four families above make.
/// That it also validates, meshes and comes to the right volume is §31.7
/// criterion 13, and `a_bar_notched_by_a_block_meshes_into_a_closed_surface`
/// is where that lives.
#[test]
fn a_bar_notched_by_a_block_is_decided_and_records_nothing() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(2.0, 8.0, &Isometry::identity())
        .expect("a bar of radius two");
    let block = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a small block");
    model
        .place_solid(
            block,
            &Isometry::translation(Vector::new(1.0, -1.0, 3.0)).expect("a shift"),
        )
        .expect("this model made the key");

    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(
            &[bar],
            &[block],
            Rule::Difference,
            &Policy::certified(),
            &mut log,
        )
        .expect(
            "a notch in a bar's wall is a face trimmed by a piece of a crossing, and it is \
                 named",
        );

    assert_eq!(fused.solids().len(), 1, "a notched bar is one solid");
    assert!(
        log.records().is_empty(),
        "nothing here had to go unsettled, got {:?}",
        log.records()
    );
}
