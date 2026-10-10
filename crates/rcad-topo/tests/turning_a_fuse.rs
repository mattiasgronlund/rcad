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
//! **Two of those three are continuous in the input and one is not.** Where two
//! faces of the pair touch, the answer's *surface area* jumps: lose a contact of
//! area `a` and both of its faces stay, so the total goes up by `2a` however
//! small the slip that lost it. Volume and the centroid do not move. Placing a
//! pair by one turn moves the two solids by a rounding relative to each other —
//! each carries its own placement and composing rounds — so a contact really can
//! be lost, and a bound derived from [`Isometry::POINT_ROUNDOFF`] cannot cover
//! it, because it is not a rounding in the answer but a different answer.
//! [`how_many_faces_touch`] is what tells the two apart; `docs/decisions.md`
//! §41.13 is the run that found it and the measurement behind it.
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

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use std::cell::Cell;

use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};
use rcad_geom::{DYNAMIC_RANGE_CHECK, Deflection, Isometry, Surface, SurfaceMeet, intersect};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew, Turn, a_turned_pair};

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
///   a sliver rather than by a rounding. **That sentence was too comfortable,
///   and §41.13 is where it was priced.** When the degeneracy is two faces
///   touching, the sliver is not small: the contact's two faces both stay, and
///   the areas differ by twice the contact whatever this multiplier is. No
///   slack covers that and none is asked to — [`how_many_faces_touch`] names it
///   instead.
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
/// and pass. The shipped seed compares 75 of its 84 pairings, and a soak at
/// [`SOAK_FACTOR`] ten 670 of 732, so a quarter is the floor that says this
/// file has not gone quietly vacuous.
///
/// **Not §17.7's criterion 8 threshold.** That number belongs in that section
/// and is measured by `fuses_that_must_agree.rs`; a second one here would be a
/// target rather than a measurement.
const SOME_PAIRINGS_MUST_BE_COMPARED: f64 = 0.25;

/// The environment variable that multiplies [`CASES`] for a soak.
const SOAK_FACTOR: &str = "RCAD_SOAK_FACTOR";

/// The environment variable that replaces the seed for a soak. Same two names
/// as `fuses_that_must_agree.rs`, so one soak drives both files.
const SOAK_SEED: &str = "RCAD_SOAK_SEED";

/// The property this file exists for.
///
/// Turning the answer and answering the turned question are two different
/// arithmetics reaching the same solid, and this is what says they reach it.
///
/// **From a fixed seed, and that is deliberate.** It used to run
/// `TestRunner::new`, whose `Config::default().rng_seed` is `RngSeed::Random`
/// and which proptest turns into `ChaChaRng::from_os_rng` — a fresh seed from
/// the operating system on every run. Two things were wrong with that. Every
/// number the report below prints was a different number each run, so §39.6's
/// method for a threshold that moves — attribute the move to a commit, never
/// widen the number — could not be applied to any of them. And the property
/// itself went green or red by the draw: `docs/decisions.md` §41.13 is the run
/// where it went red on a defect four lanes had already shipped past.
///
/// A fixed seed sweeps less than a fresh one, and that is what the soak is for:
/// [`SOAK_FACTOR`] multiplies the corpus and [`SOAK_SEED`] replaces the seed, so
/// the searching is a thing someone asks for rather than a thing the gate does
/// by accident. With neither set — which is how the suite runs it — this is
/// [`CASES`] configurations from one seed.
#[test]
fn turning_the_answer_to_a_fuse_agrees_with_fusing_the_turned_inputs() {
    let tally = Tally::default();
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases: how_many_cases(),
            source_file: Some(file!()),
            // Said out loud rather than left to the default, which looks for the
            // crate root above the test file, does not find one, and warns on
            // every run.
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
                "proptest-regressions",
            ))),
            ..Config::default()
        },
        the_seed(),
    );

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

/// The pair the shrinker found, pinned, with the numbers written out.
///
/// A `2 x 1 x 1` block sitting on a `3 x 3 x 3` one, meeting exactly on `z = 0`
/// over a rectangle of area 2. Fused where they stand they are one solid of ten
/// faces; turned by `TAU * 44 / 64` about `(2, 3, -2)` first, they are two
/// solids of twelve, because placing the second one composed the turn onto its
/// own lift and the composition moved it a fraction of an ulp off the first.
///
/// **The volumes agree and the areas differ by twice the contact**, which is the
/// whole of `docs/decisions.md` §41.13 in one case: surface area is
/// discontinuous where two faces meet and volume is not, so the propagation
/// bound this file publishes covers one of them and cannot cover the other.
///
/// This is here as well as in the `.proptest-regressions` file beside it because
/// that file is a cache — anyone may delete it — and this case is the reason the
/// area comparison has an excuse at all.
#[test]
fn a_pair_that_touches_face_to_face_can_stop_touching_when_the_pair_is_turned() {
    let still = two_blocks_meeting_on_one_plane(None);
    let turned = two_blocks_meeting_on_one_plane(Some(Turn {
        direction: [2, 3, -2],
        steps: 44,
    }));

    let room = how_much_a_lost_contact_can_add(&still, &turned);
    assert_eq!(
        room, 4.0,
        "the contact that goes missing is the `2 x 1` one, and both its faces stay"
    );

    let glued = united(&still);
    let apart = united(&turned);
    assert_eq!((glued.solids, glued.volume, glued.area), (1, 29.0, 60.0));
    assert_eq!(apart.solids, 2);
    assert!(
        (glued.volume - apart.volume).abs() < 1e-13,
        "volume is continuous across a lost contact: {} against {}",
        glued.volume,
        apart.volume
    );
    assert!(
        (apart.area - glued.area - room).abs() < 1e-13,
        "the whole of the contact is what the areas differ by, so the room this \
         leaves is not slack: {} against {}",
        glued.area,
        apart.area
    );
}

/// The two blocks of the case above, optionally turned.
fn two_blocks_meeting_on_one_plane(turn: Option<Turn>) -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Block { width: 2, depth: 1 },
                skew: Skew::none(),
                height: 1,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Block { width: 3, depth: 3 },
                skew: Skew::none(),
                height: 3,
                at: [-1, -1, -3],
            },
        ],
        turn,
        elsewhere: Elsewhere::here(),
    }
}

/// What uniting a configuration's two solids measures.
fn united(configuration: &Configuration) -> Totals {
    let (mut model, keys) = configuration.build();
    let solids = model
        .fuse(
            &[keys[0]],
            &[keys[1]],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two blocks on the lattice unite")
        .solids()
        .to_vec();
    Totals::of(&model, &solids)
}

/// The pair Q478's soak found, pinned: a `2 x 2 x 1` block at the origin and a
/// `1 x 1 x 3` one standing through its corner, `[0, 1] x [0, 1] x [-2, 1]`,
/// turned together by `TAU * 19 / 64` about `(0, -2, -1)`.
///
/// On the lattice the two share three walls exactly — `x = 0`, `y = 0` and the
/// top `z = 1` — and every rule both ways round answers the volume worked by
/// hand. Turned, each block carries its own placement and composing the turn
/// rounds its offsets, so those three walls come out a rounding apart
/// ([`turning_a_tall_block_with_a_flat_one_leaves_three_of_its_walls_a_rounding_off_the_flat_ones`]).
/// The shared cube does not care and the intersection still answers. The
/// union and both differences have to keep the slab between two of those
/// walls, an edge about `1e-16` long beside corners of nearly 3, and the fuse's
/// own range check refuses it (`Assembly::check_dynamic_range` in
/// `fuse/assemble.rs`).
///
/// **That refusal is a limit and not a gap.** The ratio is a length over a
/// length, so no scaling of the answer moves it, and the slab is where the two
/// turned inputs really are; the volumes the tolerant policy answers below are
/// the ones worked by hand. Each ratio times its slab's thickness lands
/// between 1 and 3: the largest coordinate is at most 3, and an edge across a
/// slab is no shorter than the slab is thick, so a product in that window says
/// it is that slab's edge the check measured and not some other.
///
/// With Q1288's rule undone by hand (`63a3cce8`, a piece with a corner on a
/// separating plane is still outside, in `fuse/select.rs`), all six fuses come
/// back `Undecidable` under either policy instead of reaching the check. That
/// is why the property above passed on this pair before Q1288: it counted the
/// `Undecidable`, and it had no arm for the range check until Q1560.
#[test]
fn a_tall_block_through_a_flat_one_turned_shares_a_cube_and_refuses_the_rules_that_keep_a_slab_a_rounding_thin()
 {
    let still = a_tall_block_through_the_corner_of_a_flat_one(None);
    let turned = a_tall_block_through_the_corner_of_a_flat_one(Some(THE_TURN));
    for (rule, first, worked_by_hand, slab) in A_TALL_BLOCK_THROUGH_A_FLAT_ONE {
        let mut log = ProvenanceLog::new();
        let answer = fused_on_its_own(&still, rule, first, &Policy::certified(), &mut log)
            .unwrap_or_else(|why| panic!("{rule:?} with solid {first} first, unturned: {why}"));
        has_the_volume(&answer, worked_by_hand, rule, first);

        match (
            fused_on_its_own(&turned, rule, first, &Policy::certified(), &mut log),
            slab,
        ) {
            (Ok(answer), None) => has_the_volume(&answer, worked_by_hand, rule, first),
            (Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })), Some(thickness)) => {
                assert!(
                    (1.0..=3.0).contains(&(ratio * thickness)),
                    "{rule:?} with solid {first} first was refused at a spread of {ratio}, \
                     which is not the slab {thickness} thick"
                );
            }
            (other, slab) => panic!(
                "{rule:?} with solid {first} first, turned, came back {:?} where the slab \
                 it keeps is {slab:?}",
                other.map(|answer| answer.volume)
            ),
        }
        assert!(
            log.is_empty(),
            "{rule:?} with solid {first} first logged {:?} under a certified policy",
            log.records()
        );
    }
}

/// The same turned pair under a tolerant policy answers every rule, and each
/// answer is the volume worked by hand.
///
/// So the four refusals above are of answers that exist, which is what makes
/// them a limit of `f64` rather than a wrong decision: the tolerant run keeps
/// the slab, warns once that the answer is too wide to carry, and measures what
/// the lattice measures.
#[test]
fn a_tall_block_through_a_flat_one_turned_answers_every_rule_under_a_tolerant_policy() {
    let turned = a_tall_block_through_the_corner_of_a_flat_one(Some(THE_TURN));
    for (rule, first, worked_by_hand, slab) in A_TALL_BLOCK_THROUGH_A_FLAT_ONE {
        let mut log = ProvenanceLog::new();
        let answer = fused_on_its_own(&turned, rule, first, &Policy::tolerant(3.0), &mut log)
            .unwrap_or_else(|why| panic!("{rule:?} with solid {first} first, turned: {why}"));
        has_the_volume(&answer, worked_by_hand, rule, first);

        let warned: Vec<&str> = log
            .records()
            .iter()
            .map(|record| record.predicate())
            .collect();
        let expected: &[&str] = if slab.is_some() {
            &[DYNAMIC_RANGE_CHECK]
        } else {
            &[]
        };
        assert_eq!(warned, expected, "{rule:?} with solid {first} first");
    }
}

/// Where the slabs come from: turning the pair rounds each block's offsets on
/// its own.
///
/// The two blocks are turned by one turn, so each wall of the tall block keeps
/// the very normal of the flat block's wall it shared on the lattice, bit for
/// bit. The offsets do not agree. Of the three walls the two blocks shared, one
/// of the tall block's lies [`POKES_OUT`] outside the flat block's, another
/// [`STOPS_SHORT`] inside it, and its top [`STOPS_SHORT_AT_THE_TOP`] below. The
/// other three pairs of walls are a whole one or two apart.
#[test]
fn turning_a_tall_block_with_a_flat_one_leaves_three_of_its_walls_a_rounding_off_the_flat_ones() {
    let (model, keys) = a_tall_block_through_the_corner_of_a_flat_one(Some(THE_TURN)).build();
    let planes = |solid: SolidKey| -> Vec<[f64; 4]> {
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .map(|face| face.plane().coefficients())
            .collect()
    };
    let (flat, tall) = (planes(keys[0]), planes(keys[1]));

    let mut gaps: Vec<f64> = tall
        .iter()
        .map(|tall| {
            let beside: Vec<&[f64; 4]> = flat
                .iter()
                .filter(|flat| (0..3).all(|axis| flat[axis].to_bits() == tall[axis].to_bits()))
                .collect();
            let [flat] = beside.as_slice() else {
                panic!("the tall block's wall {tall:?} faces the way {beside:?} do");
            };
            flat[3] - tall[3]
        })
        .filter(|gap| gap.abs() < 0.5)
        .collect();
    gaps.sort_by(f64::total_cmp);
    assert_eq!(gaps, [-STOPS_SHORT_AT_THE_TOP, -STOPS_SHORT, POKES_OUT]);
}

/// The turn Q478's soak drew.
const THE_TURN: Turn = Turn {
    direction: [0, -2, -1],
    steps: 19,
};

/// How far one of the tall block's turned walls lies outside the flat block's.
const POKES_OUT: f64 = 1.0408340855860843e-16;

/// How far another lies inside.
const STOPS_SHORT: f64 = 7.216449660063518e-16;

/// How far the tall block's top lies below the flat block's. Only the flat
/// block less the tall one keeps the slab between them, and that answer keeps
/// the thinner [`STOPS_SHORT`] one as well.
const STOPS_SHORT_AT_THE_TOP: f64 = 1.3322676295501878e-15;

/// Each rule with each block first: which goes first, the volume worked by
/// hand, and how thin a slab the turned answer has to keep, or `None` where it
/// keeps none.
///
/// The union keeps the sliver of tall block that pokes out through the flat
/// one's wall, and so does the tall block less the flat one. The flat block
/// less the tall one keeps the slivers of flat block that the tall one's walls
/// stop short of, and the thinner of those is [`STOPS_SHORT`]. The shared cube
/// keeps neither.
const A_TALL_BLOCK_THROUGH_A_FLAT_ONE: [(Rule, usize, f64, Option<f64>); 6] = [
    (Rule::Union, 0, 6.0, Some(POKES_OUT)),
    (Rule::Union, 1, 6.0, Some(POKES_OUT)),
    (Rule::Intersection, 0, 1.0, None),
    (Rule::Intersection, 1, 1.0, None),
    (Rule::Difference, 0, 3.0, Some(STOPS_SHORT)),
    (Rule::Difference, 1, 2.0, Some(POKES_OUT)),
];

/// Fuse a fresh copy of the pair, solid `first` against the other, and measure
/// the answer.
fn fused_on_its_own(
    configuration: &Configuration,
    rule: Rule,
    first: usize,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Totals, Error> {
    let (mut model, keys) = configuration.build();
    let solids = model
        .fuse(&[keys[first]], &[keys[1 - first]], rule, policy, log)?
        .solids()
        .to_vec();
    Ok(Totals::of(&model, &solids))
}

/// Insist an answer has the volume worked by hand, to within what turning its
/// corners can move it by.
fn has_the_volume(answer: &Totals, worked_by_hand: f64, rule: Rule, first: usize) {
    let allowed = how_far_the_volume_may_move(
        answer.triangles,
        Isometry::POINT_ROUNDOFF * answer.largest_coordinate,
        answer.largest_coordinate,
    );
    assert!(
        (answer.volume - worked_by_hand).abs() <= allowed,
        "{rule:?} with solid {first} first has volume {}, not {worked_by_hand} to within \
         {allowed}",
        answer.volume
    );
}

/// The two blocks of the cases above, optionally turned.
fn a_tall_block_through_the_corner_of_a_flat_one(turn: Option<Turn>) -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Block { width: 2, depth: 2 },
                skew: Skew::none(),
                height: 1,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Block { width: 1, depth: 1 },
                skew: Skew::none(),
                height: 3,
                at: [0, 0, -2],
            },
        ],
        turn,
        elsewhere: Elsewhere::here(),
    }
}

/// [`CASES`], multiplied by whatever [`SOAK_FACTOR`] says.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number above zero, or
/// to one whose product is more than a run can count. A soak that silently ran
/// the default corpus would be worse than one that stopped, and a product that
/// wrapped runs **fewer** cases than the shipped corpus while the operator reads
/// the number as the wide one (§48.2, §50.9).
fn how_many_cases() -> u32 {
    let Ok(factor) = std::env::var(SOAK_FACTOR) else {
        return CASES;
    };
    let factor: u32 = factor
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_FACTOR} has to be a whole number, not {factor:?}"));
    assert!(factor > 0, "{SOAK_FACTOR} has to be at least one");
    CASES
        .checked_mul(factor)
        .unwrap_or_else(|| panic!("{CASES} cases times {factor} is more than one run can count"))
}

/// The fixed seed, or the one [`SOAK_SEED`] names.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number.
fn the_seed() -> TestRng {
    let Ok(seed) = std::env::var(SOAK_SEED) else {
        return TestRng::deterministic_rng(RngAlgorithm::ChaCha);
    };
    let seed: u64 = seed
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_SEED} has to be a whole number, not {seed:?}"));
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    TestRng::from_seed(RngAlgorithm::ChaCha, &bytes)
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
        lattice.place_solid(solid, &turn)?;
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

    let moved = turned_then_fused.area - fused_then_turned.area;
    let allowed = how_far_the_area_may_move(triangles, slip, largest);
    if moved.abs() <= allowed {
        tally.watch(&tally.worst_area, moved.abs() / allowed);
    } else {
        // Not watched: what this pairing measures is how big a contact was, and
        // averaging that in would make the headroom the report prints mean
        // nothing.
        let room = how_much_a_lost_contact_can_add(&still, configuration);
        tally
            .contact_lost_in_placing
            .set(tally.contact_lost_in_placing.get() + 1);
        prop_assert!(
            moved >= -allowed && moved <= allowed + room,
            "{rule:?}: turning the answer gives surface area {} and fusing the turned pair \
             gives {}, which is further apart than {allowed}, and the contacts placing the \
             pair broke are worth only {room}",
            fused_then_turned.area,
            turned_then_fused.area
        );
    }

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

/// How much surface area the answer can gain from the contacts that placing the
/// pair broke.
///
/// **This is a derived quantity and not a slack**, which is why the area
/// comparison is one assertion with a wider window on one side rather than an
/// excuse with a branch in it. It is worked out from the two *inputs*, before
/// either of them is fused, and it is zero for every pairing whose contacts all
/// survived — which is nearly all of them.
///
/// A lattice pair is written in whole numbers, so two of its faces sit on one
/// plane exactly or a whole unit apart. Turning the pair does **not** move its
/// two solids by one movement: each carries its own placement, `place_solid`
/// composes the turn onto it, and composing rounds. Two faces that were touching
/// come out a fraction of an ulp apart; the fuse reads the four numbers a plane
/// holds as exact, answers that the solids do not touch, and keeps two faces it
/// would otherwise have dropped. `rcad-geom`'s
/// `a_contact_between_two_solids_at_two_placements_does_not_survive_every_turn`
/// is that arithmetic on its own — 11100 of the 21546 turns this file can draw
/// lose a contact that way. `docs/decisions.md` §41.13.
///
/// **Why twice the smaller face.** Away from the contact the turned pair is the
/// lattice pair to within the rounding [`how_far_the_area_may_move`] already
/// covers. Over the contact it is not: each of the two coincident faces is
/// either kept there or dropped there, so the two totals differ by at most twice
/// that region on top of the bound. The region is inside both faces, so the
/// smaller face bounds it.
///
/// **Loosely, and only where it costs nothing.** Two big faces coincident over a
/// small patch make this many times the jump they can actually cause. It is
/// added only to a pairing that has already broken the plain bound — none of the
/// shipped corpus and one of about five thousand under a soak — and it is a
/// bound on the *inputs*, so no answer this test looks at can widen its own
/// window. What closes the looseness is clipping the two outlines against each
/// other in their shared plane, which is a polygon intersection and belongs
/// where one is already written.
///
/// The window is only widened **upwards**. A contact can be lost and never
/// gained: two faces of a lattice pair are parallel only when their normals are
/// the same signed axis, and then their offsets are whole numbers, so they are
/// one plane exactly or a whole unit apart — and placing a solid moves an offset
/// by less than `Isometry::POINT_ROUNDOFF` times its size. So a turned answer
/// with *less* area than the lattice one has dropped a face, and no lost contact
/// excuses it.
fn how_much_a_lost_contact_can_add(still: &Configuration, turned: &Configuration) -> f64 {
    let (here, mine) = still.build();
    let (there, theirs) = turned.build();
    let lattice = [each_face_of(&here, mine[0]), each_face_of(&here, mine[1])];
    let moved = [
        each_face_of(&there, theirs[0]),
        each_face_of(&there, theirs[1]),
    ];
    assert_eq!(
        [lattice[0].len(), lattice[1].len()],
        [moved[0].len(), moved[1].len()],
        "the same configuration turned has the same faces in the same order"
    );

    let mut room = 0.0;
    for (one, (surface, area)) in lattice[0].iter().enumerate() {
        for (other, (against, its_area)) in lattice[1].iter().enumerate() {
            if on_one_surface(surface, against)
                && !on_one_surface(&moved[0][one].0, &moved[1][other].0)
            {
                room += 2.0 * area.min(*its_area);
            }
        }
    }
    room
}

/// The surface each face of a solid sits on, and how much of it the face is.
fn each_face_of(model: &Model, solid: SolidKey) -> Vec<(Surface, f64)> {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .map(|face| {
            let mut area = 0.0;
            face.for_each_triangle(|[a, b, c]| {
                area += (b - a).cross(c - a).length() / 2.0;
            });
            (face.surface(), area)
        })
        .collect()
}

/// Whether two faces are on the same surface, which is the question the fuse
/// asks in `cuts::gather` and answers `SurfaceMeet::SameSurface` to.
fn on_one_surface(surface: &Surface, against: &Surface) -> bool {
    matches!(
        intersect(
            surface,
            against,
            &Policy::certified(),
            &mut ProvenanceLog::new()
        ),
        Ok(SurfaceMeet::SameSurface)
    )
}

/// Fuse a copy of the configuration's model, counting what came back into
/// `side`.
///
/// A copy, so each of the two paths starts from a model nothing else has
/// touched. `None` means there was no answer to compare — refused, an answer
/// with an edge too short to hold beside its own coordinates, or not a solid
/// this kernel can hold — and which of those it was has been counted.
///
/// **An edge too short is a limit on the turned side**, refused
/// [`Invalid::DynamicRangeTooWide`]. Each solid carries its own placement and
/// composing the turn rounds its offsets, so two walls that share a plane on
/// the lattice come out a rounding apart, and an answer can have to keep the
/// slab between them: an edge `1e-16` long beside corners of 3, a length over a
/// length that no scaling moves (`Assembly::check_dynamic_range` in
/// `fuse/assemble.rs`).
/// [`a_tall_block_through_a_flat_one_turned_shares_a_cube_and_refuses_the_rules_that_keep_a_slab_a_rounding_thin`]
/// is the pair that showed it, and `fuses_that_must_agree.rs` counts the same
/// refusal by name for its turned family (`a_flat_refusal`).
///
/// A refusal of either kind on the lattice side fails outright. §17.7's
/// control row: every meet in a rectilinear configuration on the integer grid
/// is exactly representable, so a certified policy must decide all of it, and
/// two of its walls share a plane exactly or lie a whole unit apart, so no
/// answer keeps a slab thinner than that. That is what makes
/// [`Tally::only_one_side`] unambiguous — it is always the turned side that
/// went missing.
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
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => {
            side.an_edge_too_short.set(side.an_edge_too_short.get() + 1);
            if side.refusals_are_a_bug {
                return Err(TestCaseError::fail(format!(
                    "a {rule:?} over the lattice pair kept an edge too short beside its \
                     coordinates, a spread of {ratio}"
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
    /// Pairings whose areas disagree because placing the pair broke a contact
    /// between two of its faces. See [`how_many_faces_touch`].
    contact_lost_in_placing: Cell<usize>,
    worst_volume: Cell<f64>,
    worst_area: Cell<f64>,
    worst_centroid: Cell<f64>,
}

/// What one of the two paths came back with.
#[derive(Default)]
struct Side {
    /// Fuses refused `Undecidable`: the arithmetic gave up.
    refused: Cell<usize>,
    /// Fuses refused because their answer keeps an edge too short to hold
    /// beside its own coordinates: the answer exists and `f64` cannot carry it.
    an_edge_too_short: Cell<usize>,
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
            contact_lost_in_placing: Cell::default(),
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
        for (name, side) in [("lattice", &self.still), ("turned", &self.turned)] {
            println!(
                "    {name} side: {} refused, {} refused for an edge too short, {} not a solid",
                side.refused.get(),
                side.an_edge_too_short.get(),
                side.not_a_solid.get()
            );
        }
        println!(
            "    {} pairings compared, {} only the lattice side answered, \
             {} only the turned side answered, {} split into a different number of solids",
            self.compared.get(),
            self.only_the_lattice_side.get(),
            self.only_the_turned_side.get(),
            self.split_differently.get()
        );
        println!(
            "    {} pairings whose areas disagree because placing the pair broke a contact",
            self.contact_lost_in_placing.get()
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
            let solid = model.solid(solid).expect("this model made the key");
            let measure = solid
                .measure(
                    a_deflection(),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a solid this size measures");
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

/// The deflection the measurements here are asked for.
///
/// Every solid in this file is a polyhedron, and a polyhedron ignores it: its
/// own fan is the answer rather than a picture of one (`docs/decisions.md`
/// §25.4).
fn a_deflection() -> Deflection {
    Deflection::of(0.1).expect("a tenth of a millimetre is a length")
}
