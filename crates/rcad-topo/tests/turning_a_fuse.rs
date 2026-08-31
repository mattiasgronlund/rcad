//! Turning the answer to a fuse against fusing the turned inputs, under a
//! general rotation where the two cannot agree bit for bit.
//!
//! §17.6's tenth piece, and the missing half of roadmap §11.2's "start with
//! transform invariance". `fuses_that_must_agree.rs` already runs `T(op(A, B))`
//! against `op(T(A), T(B))` for the isometries that are exactly representable —
//! quarter turns about the extrusion axis and whole-number shifts — where the
//! claim is bit for bit and no tolerance appears anywhere. This file is the
//! other half: an arbitrary rotation, where turning a point rounds, so the two
//! answers are two different `f64` answers to the same question and the only
//! honest claim is a bounded one.
//!
//! That is why it was written last. §14.1 question 1: a bounded relation is
//! worth nothing until the bound is published beside the expression it bounds,
//! because otherwise the tolerance gets tuned until the test passes and then
//! the test is measuring the tolerance. The expression here is
//! [`Isometry::apply_to_point`], and its bound is
//! [`Isometry::POINT_ROUNDOFF`], derived and tested in `rcad-geom`.
//!
//! # What is compared
//!
//! Not the partition. Two valid runs cut an answer into different but equally
//! valid faces, so topological equality is the wrong predicate (roadmap §11.2).
//! Volume, surface area and centroid are compared instead, totalled over the
//! whole answer so that splitting it into a different number of solids does not
//! enter into it.
//!
//! # What is derived and what is judged
//!
//! [`how_far_the_volume_may_move`], [`how_far_the_area_may_move`] and
//! [`how_far_the_centroid_may_move`] are derived from
//! [`Isometry::POINT_ROUNDOFF`] and nothing else: a coordinate that has slipped
//! by `POINT_ROUNDOFF · L` moves each triangle's contribution to those sums by
//! an amount their doc comments work out, and there are so many triangles.
//!
//! [`TURNED_FUSE_SLACK`] is the part that is *not* derived, and it is here
//! rather than in `rcad-geom` because that is where §15.6 rule 2 puts a
//! judgement. What it covers is said at the constant.
//!
//! # §14.1's four questions
//!
//! 1. **Exactly true, or needing a bound?** Needing a bound, and that is the
//!    whole reason this file exists separately from `fuses_that_must_agree.rs`.
//! 2. **Do the two runs take different paths?** Yes, and further apart than
//!    anywhere else in the suite. One fuses coordinates that are whole numbers
//!    and turns the answer; the other turns first and fuses coordinates with no
//!    exactly representable relationship left between them, so every corner it
//!    invents comes out of a Cramer solve on turned planes. The refusal counts
//!    below are the plainest evidence that these are not the same path.
//! 3. **What family of inputs?** [`a_turned_pair`], shared with §17.7's
//!    headline row. §13 rule 5 asks for *varying* generators across pieces, and
//!    the rotated family is this piece's; a fourth family would be a different
//!    thing to test, not a better test of this one.
//! 4. **Which gap would this have caught?** §12.3's. A refusal is an expected
//!    outcome under a general rotation, so the temptation is to drop refused
//!    cases on the floor — which is exactly how `random_extrusions` kept a
//!    wrongly refused input invisible. Nothing here filters: every fuse is
//!    counted, and a rotation that changes *whether* the kernel could decide is
//!    counted separately and reported, because that is a fact about our filters
//!    and not about the geometry.

mod pairs;

use std::cell::Cell;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, TestCaseError, TestRunner};
use rcad_geom::Isometry;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

use pairs::{Configuration, a_turned_pair};

/// How many configurations the property sees.
///
/// Twenty-four, which is 144 fuses: six per configuration, three rules two ways
/// round. A debug build validates the whole model after every commit and after
/// every placement, so that costs about two seconds — the same band as the fuse
/// properties next door and well inside the slowest of them. The workspace
/// suite runs in about nine seconds and that is worth protecting.
const CASES: u32 = 24;

/// How much room to leave on top of the derived propagation.
///
/// The three propagation bounds are derived, and they cover exactly one thing:
/// the rounding in turning an answer that has already been computed. Three
/// things they do not cover, in the order of how much they are worth:
///
/// - **The fuse's own arithmetic in the turned frame.** Every corner it invents
///   is `Plane::meet_of_three`, which is Cramer's rule, which divides. A bound
///   on how far that lands from the truth needs a bound on how badly
///   conditioned the three turned planes are, and nothing in this tree has one.
///   `fuses_that_must_agree.rs` records the same gap from the other side, which
///   is why its inclusion-and-exclusion property is not run on turned pairs.
/// - **A different arrangement.** Near a degeneracy the two runs may genuinely
///   decide differently and still both be right, and then the answers differ by
///   a sliver rather than by a rounding.
/// - **Summation.** The two runs add the same triangles up in a different
///   order, and neither sum is exact.
///
/// So this is a judgement about how much room to leave, not a fact about any
/// expression, and §15.6 rule 2 is why it is a separate named constant here
/// rather than folded into [`Isometry::POINT_ROUNDOFF`] over in `rcad-geom`.
/// The shape of each bound — how it grows with the triangle count, the largest
/// coordinate and the volume — is derived; only this multiplier is not.
///
/// Sixteen, the same figure and the same kind of judgement as
/// `Model::validate`'s `MEASURING_SLACK`, which allows for a plane having been
/// fitted through corners that were already rounded.
///
/// The test prints what share of the bound the worst case used, so the headroom
/// is a number a reader can see rather than a claim. It is about one part in
/// ten thousand, and almost none of that is this multiplier: over five thousand
/// comparisons the worst case used a three-hundredth of the derived propagation
/// on its own. A bound is a worst case, and the worst case — every triangle's
/// rounding pushing the same way, every corner as far from the origin as the
/// answer reaches — does not happen.
///
/// That headroom reads worse than it is. Multiplying one side's volume by
/// `1 + 1e-9` overshoots the bound nearly four hundredfold, so what slips
/// through is not a fuse that has broken but a fuse whose last few bits have
/// got worse. The refusal counts below are what watch for the second thing.
const TURNED_FUSE_SLACK: f64 = 16.0;

/// How many of the pairings the property has to have actually compared.
///
/// §14.7's half of it. A property only ever looks at the fuses that came back,
/// so a run where the turned side refused everything would assert nothing at all
/// and pass. Three quarters of the pairings get compared on a good seed and
/// getting on for half on a bad one, so a quarter is the floor that says this
/// file has not gone quietly vacuous.
///
/// **Not §17.7's criterion 8 threshold.** That number belongs in that section
/// and is measured by `fuses_that_must_agree.rs`; a second one here would be a
/// target rather than a measurement.
const SOME_PAIRINGS_MUST_BE_COMPARED: f64 = 0.25;

/// The property this file exists for.
///
/// Turning the answer and answering the turned question are two different
/// arithmetics reaching the same solid, and this is what says they reach it.
#[test]
fn turning_the_answer_to_a_fuse_agrees_with_fusing_the_turned_inputs() {
    let tally = Tally::default();
    let mut runner = TestRunner::new(Config {
        cases: CASES,
        source_file: Some(file!()),
        // Said out loud rather than left to the default, which looks for the
        // crate root above the test file, does not find one, and warns on every
        // run.
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..Config::default()
    });

    let outcome = runner.run(&a_turned_pair(), |configuration| {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            both_ways_round(&tally, &configuration, rule)?;
        }
        Ok(())
    });

    tally.report();
    if let Err(why) = outcome {
        panic!("{why}");
    }

    let pairings = tally.fuses.get() / 2;
    assert!(
        tally.compared.get() as f64 > SOME_PAIRINGS_MUST_BE_COMPARED * pairings as f64,
        "only {} of {pairings} pairings had an answer on both sides, so this property \
         barely asserted anything",
        tally.compared.get()
    );
}

/// Fuse and then turn, turn and then fuse, and insist the two measure the same.
fn both_ways_round(
    tally: &Tally,
    configuration: &Configuration,
    rule: Rule,
) -> Result<(), TestCaseError> {
    let turn = configuration
        .turn
        .expect("a turned pair is generated with a turn")
        .isometry();
    let still = Configuration {
        solids: configuration.solids.clone(),
        turn: None,
        elsewhere: configuration.elsewhere,
    };

    let turned_last = fused(tally, &tally.still, &still, rule)?;
    let turned_first = fused(tally, &tally.turned, configuration, rule)?;

    match (turned_last.is_some(), turned_first.is_some()) {
        (true, false) => tally
            .only_the_lattice_side
            .set(tally.only_the_lattice_side.get() + 1),
        (false, true) => tally
            .only_the_turned_side
            .set(tally.only_the_turned_side.get() + 1),
        (true, true) | (false, false) => (),
    }
    let (Some((mut lattice, answer)), Some((turned, turned_answer))) = (turned_last, turned_first)
    else {
        return Ok(());
    };

    for &solid in &answer {
        lattice.place_solid(solid, &turn);
    }

    let fused_then_turned = Totals::of(&lattice, &answer);
    let turned_then_fused = Totals::of(&turned, &turned_answer);
    tally.compared.set(tally.compared.get() + 1);
    if fused_then_turned.solids != turned_then_fused.solids {
        tally
            .split_differently
            .set(tally.split_differently.get() + 1);
    }

    let triangles = fused_then_turned.triangles.max(turned_then_fused.triangles);
    let largest = fused_then_turned
        .largest_coordinate
        .max(turned_then_fused.largest_coordinate);
    let slip = Isometry::POINT_ROUNDOFF * largest;

    let apart = (fused_then_turned.volume - turned_then_fused.volume).abs();
    let allowed = how_far_the_volume_may_move(triangles, slip, largest);
    tally.watch(&tally.worst_volume, apart / allowed);
    prop_assert!(
        apart <= allowed,
        "{rule:?}: turning the answer gives volume {} and fusing the turned pair gives {}, \
         which is further apart than {allowed}",
        fused_then_turned.volume,
        turned_then_fused.volume
    );

    let apart = (fused_then_turned.area - turned_then_fused.area).abs();
    let allowed = how_far_the_area_may_move(triangles, slip, largest);
    tally.watch(&tally.worst_area, apart / allowed);
    prop_assert!(
        apart <= allowed,
        "{rule:?}: turning the answer gives surface area {} and fusing the turned pair gives {}, \
         which is further apart than {allowed}",
        fused_then_turned.area,
        turned_then_fused.area
    );

    if let (Some(mine), Some(theirs)) = (fused_then_turned.centroid, turned_then_fused.centroid) {
        let volume = fused_then_turned.volume.min(turned_then_fused.volume);
        let allowed = how_far_the_centroid_may_move(triangles, slip, largest, volume);
        let apart = (0..3)
            .map(|axis| (mine[axis] - theirs[axis]).abs())
            .fold(0.0, f64::max);
        tally.watch(&tally.worst_centroid, apart / allowed);
        prop_assert!(
            apart <= allowed,
            "{rule:?}: the two centroids {mine:?} and {theirs:?} are more than {allowed} apart"
        );
    }
    Ok(())
}

/// How far a coordinate slip of `slip` can move the volume of an answer made of
/// `triangles` triangles whose largest coordinate is `largest`.
///
/// Volume is `(1/6)·Σ a·(b × c)` over the triangles. The triple product is
/// linear in each corner separately, so moving all three corners changes it by
/// at most `|Δa||b||c| + |a||Δb||c| + |a||b||Δc|`. A corner whose coordinates
/// have each slipped by `slip` has moved at most `√3·slip`, and a corner whose
/// coordinates are each at most `largest` is at most `√3·largest` from the
/// origin, so each of those three terms is at most `3√3·slip·largest²` and the
/// whole triple product moves by at most `9√3·slip·largest²`.
///
/// Divide by six and multiply by the triangle count: `(3√3/2)·triangles·slip·
/// largest²`. Then [`TURNED_FUSE_SLACK`], for what the derivation does not
/// reach.
fn how_far_the_volume_may_move(triangles: usize, slip: f64, largest: f64) -> f64 {
    TURNED_FUSE_SLACK * 1.5 * f64::sqrt(3.0) * triangles as f64 * slip * largest * largest
}

/// The same for surface area.
///
/// A face's area is the length of the sum of its triangles' vector areas
/// `(b − a) × (c − a) / 2`, so the sum over faces of the whole answer moves by
/// at most the sum over triangles of how far one vector area moves. An edge
/// vector is at most `2√3·largest` long and slips by at most `2√3·slip`, and
/// the cross product is linear in each of the two, so one vector area moves by
/// at most `(2√3·slip·2√3·largest) · 2 / 2 = 12·slip·largest`.
fn how_far_the_area_may_move(triangles: usize, slip: f64, largest: f64) -> f64 {
    TURNED_FUSE_SLACK * 12.0 * triangles as f64 * slip * largest
}

/// The same for one coordinate of the centroid.
///
/// The centroid is `moment / (4·six_volumes)`, where the moment sums
/// `a·(b × c) · (aᵢ + bᵢ + cᵢ)`. Each of those terms moves by at most
/// `9√3·slip·largest² · 3·largest + 3√3·largest³ · 3·slip = 36√3·slip·largest³`,
/// using the two magnitudes the volume bound already established. The divisor
/// moves too, and dividing a moved numerator by a moved divisor leaves
/// `(Δmoment + 4·|centroid|·Δsix_volumes) / (4·six_volumes)`; with `|centroid|`
/// at most `largest` and `six_volumes` equal to `6·volume`, that comes to
/// `3√3·triangles·slip·largest³ / volume`.
///
/// This is the one bound that needs the answer's volume, and a fuse can
/// genuinely return a sliver — which is honest rather than awkward. A centroid
/// of a nearly flat solid really is that badly determined.
fn how_far_the_centroid_may_move(triangles: usize, slip: f64, largest: f64, volume: f64) -> f64 {
    TURNED_FUSE_SLACK * 3.0 * f64::sqrt(3.0) * triangles as f64 * slip * largest.powi(3) / volume
}

/// Fuse a copy of the configuration's model, counting what came back into
/// `side`.
///
/// A copy, so each of the two paths starts from a model nothing else has
/// touched. `None` means there was no answer to compare — refused, or not a
/// solid this kernel can hold — and which of those it was has been counted.
///
/// A refusal on the lattice side fails outright. §17.7's control row: every
/// meet in a rectilinear configuration on the integer grid is exactly
/// representable, so a certified policy must decide all of it, and that is what
/// makes [`Tally::only_one_side`] unambiguous — it is always the turned side
/// that went missing.
fn fused(
    tally: &Tally,
    side: &Side,
    configuration: &Configuration,
    rule: Rule,
) -> Result<Option<(Model, Vec<SolidKey>)>, TestCaseError> {
    let (mut model, keys) = configuration.build();
    let mut log = ProvenanceLog::new();
    tally.fuses.set(tally.fuses.get() + 1);

    match model.fuse(&[keys[0]], &[keys[1]], rule, &Policy::certified(), &mut log) {
        Ok(fused) => {
            let solids = fused.solids().to_vec();
            Ok(Some((model, solids)))
        }
        Err(Error::Undecidable { records }) => {
            side.refused.set(side.refused.get() + 1);
            if side.refusals_are_a_bug {
                return Err(TestCaseError::fail(format!(
                    "a {rule:?} over the lattice pair refused, with {} records",
                    records.len()
                )));
            }
            Ok(None)
        }
        Err(Error::ResultIsNotManifold { .. }) => {
            side.not_a_solid.set(side.not_a_solid.get() + 1);
            Ok(None)
        }
        Err(why) => Err(TestCaseError::fail(format!("{rule:?} went wrong: {why}"))),
    }
}

/// What every fuse this file ran came back with.
///
/// Counted rather than filtered, because §12.3 is the section where discarding
/// refused inputs is what kept a wrongly refused one invisible. The number that
/// earns its keep is the pair of counts of pairings where only one side had an
/// answer: a rotation is supposed to change how hard the arithmetic is, and if
/// it changes *whether* the kernel can answer at all then the two sides of this
/// property are not seeing the same question, and no amount of agreement between
/// the cases that did come back would say otherwise. Which way round it goes is
/// counted separately, because "the turned side refused" and "the turned side
/// answered where the lattice side could not build a solid" are two different
/// facts.
struct Tally {
    fuses: Cell<usize>,
    /// The lattice pair, fused and then turned.
    still: Side,
    /// The same pair turned first and then fused.
    turned: Side,
    compared: Cell<usize>,
    /// Pairings only the lattice side answered.
    only_the_lattice_side: Cell<usize>,
    /// Pairings only the turned side answered.
    only_the_turned_side: Cell<usize>,
    /// Answers cut into a different number of solids each way round. Allowed —
    /// that is why the measurements are totalled — and worth seeing.
    split_differently: Cell<usize>,
    worst_volume: Cell<f64>,
    worst_area: Cell<f64>,
    worst_centroid: Cell<f64>,
}

/// What one of the two paths came back with.
#[derive(Default)]
struct Side {
    refused: Cell<usize>,
    not_a_solid: Cell<usize>,
    refusals_are_a_bug: bool,
}

impl Default for Tally {
    fn default() -> Self {
        Self {
            fuses: Cell::default(),
            still: Side {
                refusals_are_a_bug: true,
                ..Side::default()
            },
            turned: Side::default(),
            compared: Cell::default(),
            only_the_lattice_side: Cell::default(),
            only_the_turned_side: Cell::default(),
            split_differently: Cell::default(),
            worst_volume: Cell::default(),
            worst_area: Cell::default(),
            worst_centroid: Cell::default(),
        }
    }
}

impl Tally {
    fn watch(&self, worst: &Cell<f64>, share_of_the_bound: f64) {
        if share_of_the_bound.is_finite() {
            worst.set(worst.get().max(share_of_the_bound));
        }
    }

    fn report(&self) {
        println!(
            "fuse-then-turn against turn-then-fuse, {} fuses over turned pairs",
            self.fuses.get()
        );
        println!(
            "    lattice side: {} refused, {} not a solid",
            self.still.refused.get(),
            self.still.not_a_solid.get()
        );
        println!(
            "    turned side: {} refused, {} not a solid",
            self.turned.refused.get(),
            self.turned.not_a_solid.get()
        );
        println!(
            "    {} pairings compared, {} only the lattice side answered, \
             {} only the turned side answered, {} split into a different number of solids",
            self.compared.get(),
            self.only_the_lattice_side.get(),
            self.only_the_turned_side.get(),
            self.split_differently.get()
        );
        println!(
            "    worst share of the bound used: volume {:.3e}, area {:.3e}, centroid {:.3e}",
            self.worst_volume.get(),
            self.worst_area.get(),
            self.worst_centroid.get()
        );
    }
}

/// An answer's decomposition-invariant measurements, and the size of the sums
/// they came out of.
///
/// Totalled over every solid of the answer rather than kept per solid, because
/// the two runs may split the same shape into a different number of solids and
/// that is not a disagreement about the shape.
struct Totals {
    volume: f64,
    area: f64,
    /// `None` when the answer is empty and there is nowhere to balance it.
    centroid: Option<[f64; 3]>,
    solids: usize,
    triangles: usize,
    largest_coordinate: f64,
}

impl Totals {
    fn of(model: &Model, solids: &[SolidKey]) -> Self {
        let mut volume = 0.0;
        let mut area = 0.0;
        let mut moment = [0.0f64; 3];
        let mut triangles = 0;
        let mut largest_coordinate = 0.0f64;

        for &solid in solids {
            let solid = model.solid(solid);
            let measure = solid.measure();
            volume += measure.volume();
            area += measure.surface_area();
            let middle = measure.centroid().coordinates();
            for (axis, total) in moment.iter_mut().enumerate() {
                *total += measure.volume() * middle[axis];
            }
            for face in solid.faces() {
                face.for_each_triangle(|corners| {
                    triangles += 1;
                    for corner in corners {
                        largest_coordinate =
                            largest_coordinate.max(corner.largest_coordinate_magnitude());
                    }
                });
            }
        }

        Self {
            volume,
            area,
            centroid: (volume > 0.0).then(|| moment.map(|axis| axis / volume)),
            solids: solids.len(),
            triangles,
            largest_coordinate,
        }
    }
}
