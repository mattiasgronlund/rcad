//! A tolerant fuse writes an answer a certified sweep will not accept.
//!
//! Sixteen inputs out of a 9 975-input `two_solids_fused_by_a_stranger` corpus
//! fuse to `Ok` and then fail `Model::validate` under `Policy::certified()`.
//! Every one of them fused under `Policy::tolerant`, and every one of them
//! validates under that same tolerant policy. Nothing contradicted itself and
//! nothing was corrupted: the model is exactly the answer, and two policies read
//! it two ways.
//!
//! All sixteen are also the *same shape as this file*. Their inputs span between
//! `1.0` and `1.5000053564996183`, well inside
//! [`DynamicRange::LARGEST_SUPPORTED_RATIO`](rcad_geom::DynamicRange), and their
//! answers span between `4.323455642275676e17` and `inf` — so it is the fuse
//! that grew the range, not the caller who handed it one already too wide.
//! `docs/decisions.md` §65 has the sixteen measurements.
//!
//! What is pinned here is that the split is the **policy** and not the answer:
//! the same model, unchanged, passes one sweep and fails the other. The exit
//! check that would have refused is live and reached — the certified fuse of the
//! very same pair refuses — and a tolerant policy stood it down, which is what a
//! tolerant policy is for.

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

/// The thinnest hair two *translated* blocks can be fused a tolerant policy
/// still answers on.
///
/// Fifteen millimetres over `1e-200` is `1.5e201`, twenty decades past what
/// `f64` carries, and `a_fuse_that_builds_a_sliver.rs` already pins that a
/// tolerant fuse writes this one rather than refusing it.
///
/// Ten of the sixteen corpus answers span `inf` — fifteen millimetres over a
/// hair below `8.3e-308`, where the division itself overflows. **A pair of
/// translated blocks cannot be made to reach that**, and the reason is worth
/// writing down: at a hair of `2.78e-308` this same fuse comes back
/// `Undecidable`, because *where three surfaces meet* is an area, an area is
/// two such hairs multiplied, and that underflows to nothing. The corpus's ten
/// got past it by carrying their hair in a plane's own coefficients — a normal
/// of `[6.3e-322, -1.0, 0.0]` at an offset of `3.16e-321` — where no area is
/// ever formed from two of them. Nothing below turns on which side of that line
/// the hair sits; both sides split the two sweeps the same way.
const HAIR_THE_CERTIFIED_SWEEP_REFUSES: f64 = 1e-200;

/// The hair the narrowest of the sixteen corpus answers was built on.
///
/// `15.0 / 3.469446951953614e-17` is `4.323455642275676e17` — the span that
/// corpus entry's answer came out at, and a long way from overflow. Nothing
/// about these sixteen needs absurd numbers.
const HAIR_A_LONG_WAY_FROM_OVERFLOW: f64 = 3.469446951953614e-17;

/// The same model, two sweeps, two answers.
#[test]
fn a_tolerant_fuse_writes_a_sliver_only_a_tolerant_sweep_calls_valid() {
    let (mut model, [low, high]) = two_blocks_a_hair_apart(HAIR_THE_CERTIFIED_SWEEP_REFUSES);
    let tolerant = Policy::tolerant(model.scale().nominal_size());

    model
        .fuse(
            &[low, high],
            &[],
            Rule::Union,
            &tolerant,
            &mut ProvenanceLog::new(),
        )
        .expect("a tolerant policy warns about a spread this wide rather than refusing it");

    model
        .validate(&tolerant, &mut ProvenanceLog::new())
        .expect("and the sweep under that same policy warns about it too");

    match model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => {
            assert_eq!(ratio, 15.0 / HAIR_THE_CERTIFIED_SWEEP_REFUSES);
        }
        other => panic!("expected a certified sweep to refuse the spread, got {other:?}"),
    }
}

/// And the same at a spread with four decades of room left before overflow.
///
/// `inf` invites the reading that the numbers were absurd. This one is
/// `4.3e17` — four decades inside what `f64` divides — and it splits the two
/// sweeps exactly the same way.
#[test]
fn the_two_sweeps_split_the_same_way_a_long_way_from_overflow() {
    let (mut model, [low, high]) = two_blocks_a_hair_apart(HAIR_A_LONG_WAY_FROM_OVERFLOW);
    let tolerant = Policy::tolerant(model.scale().nominal_size());

    model
        .fuse(
            &[low, high],
            &[],
            Rule::Union,
            &tolerant,
            &mut ProvenanceLog::new(),
        )
        .expect("a tolerant policy warns about a spread this wide rather than refusing it");

    model
        .validate(&tolerant, &mut ProvenanceLog::new())
        .expect("and the sweep under that same policy warns about it too");

    match model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => {
            assert_eq!(ratio, 15.0 / HAIR_A_LONG_WAY_FROM_OVERFLOW);
        }
        other => panic!("expected a certified sweep to refuse the spread, got {other:?}"),
    }
}

/// The inputs fit and the answer does not, so the fuse is what grew the spread.
///
/// This is the measurement that decides where a refusal would have to go, and it
/// says the exit and not the door: nothing about either input is out of range,
/// and refusing at the door would refuse two ordinary blocks.
#[test]
fn the_inputs_span_one_and_a_half_and_the_answer_the_fuse_builds_spans_far_more() {
    let (mut model, [low, high]) = two_blocks_a_hair_apart(HAIR_A_LONG_WAY_FROM_OVERFLOW);
    assert_eq!(
        widest_spread_in(&model),
        1.5,
        "fifteen millimetres of coordinates over a ten millimetre edge"
    );

    model
        .fuse(
            &[low, high],
            &[],
            Rule::Union,
            &Policy::tolerant(model.scale().nominal_size()),
            &mut ProvenanceLog::new(),
        )
        .expect("a tolerant policy warns about a spread this wide rather than refusing it");

    assert_eq!(
        widest_spread_in(&model),
        15.0 / HAIR_A_LONG_WAY_FROM_OVERFLOW,
        "and the answer is the sliver the fuse built, beside the coordinates it kept"
    );
}

/// The exit check is live: the same pair under a certified policy is refused.
///
/// Without this the two tests above would be equally well explained by the
/// answer's range never being measured at all, which is the reading the fix
/// would have to be different for.
#[test]
fn a_certified_fuse_of_the_very_same_pair_refuses_the_answer_it_would_have_built() {
    let (mut model, [low, high]) = two_blocks_a_hair_apart(HAIR_A_LONG_WAY_FROM_OVERFLOW);

    let refused = model.fuse(
        &[low, high],
        &[],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    match refused {
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => {
            assert_eq!(ratio, 15.0 / HAIR_A_LONG_WAY_FROM_OVERFLOW);
        }
        other => panic!("expected the answer's own spread to be refused, got {other:?}"),
    }
    assert_eq!(
        model.solids().count(),
        2,
        "and a refused fuse leaves the model alone"
    );
}

/// The two blocks, the second moved up by `hair`.
///
/// `10.0 + hair` is `10.0` for every hair used here, so the moved block's top
/// face does not move and only its bottom face lands a hair above the other's.
fn two_blocks_a_hair_apart(hair: f64) -> (Model, [SolidKey; 2]) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let low = block(&mut model, Vector::new(0.0, 0.0, 0.0));
    let high = block(&mut model, Vector::new(5.0, hair, 5.0));
    (model, [low, high])
}

fn block(model: &mut Model, at: Vector) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(
            &square,
            10.0,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a box needs no decision")
}

/// The largest coordinate in the model over its shortest open edge.
///
/// The same two numbers `Model::validate` measures, without the radii — every
/// face here is a plane.
fn widest_spread_in(model: &Model) -> f64 {
    let largest = model
        .solids()
        .flat_map(|solid| solid.faces())
        .flat_map(|face| face.wires())
        .flat_map(|wire| wire.points())
        .map(|corner| corner.largest_coordinate_magnitude())
        .fold(0.0, f64::max);
    let shortest = model
        .edges()
        .filter(|edge| {
            !edge
                .is_closed()
                .expect("a fuse of two blocks leaves straight edges")
        })
        .map(|edge| edge.distance_between_its_ends())
        .fold(f64::INFINITY, f64::min);
    largest / shortest
}
