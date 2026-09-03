//! Two blocks placed a hair apart, and the sliver the fuse builds between them.
//!
//! Both inputs are ordinary 10 mm boxes. One of them is moved along `y` by a
//! distance far too small to move its top face — `10.0 + 1e-200` is `10.0` —
//! but big enough to move its bottom face onto its own plane a hair above the
//! other's. The union therefore has a step in it one hair high, and that step
//! is a face and an edge the fuse has to write. **Neither input had a feature
//! anywhere near that small**, so nothing about the inputs predicted it, and
//! both of them pass `Model::validate` under `Policy::certified()`.
//!
//! A fuzzer found this shape (`two_solids_fused_by_a_stranger`, 165,786
//! executions) with the hair at `3.80218313259032e-211`, and the fuse crashed
//! on it: `Assembly::commit`'s own after-the-write check said the model it had
//! just written had a feature of size `0.0` in it. Two separate things were
//! wrong, and there is a test below for each.

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

/// A hair small enough that squaring it is zero.
///
/// `1e-200` squared is `1e-400`, and the smallest number `f64` holds at all is
/// about `4.9e-324`, so a distance worked out as the root of a sum of squares
/// comes back as exactly zero for a separation this small.
const TOO_SMALL_TO_SQUARE: f64 = 1e-200;

/// A hair small enough that a model cannot carry it, but big enough to square.
///
/// `1e-160` squared is `1e-320`, which `f64` still holds, so nothing underflows
/// on the way to measuring it. What it is too small for is
/// [`DynamicRange`](rcad_geom::DynamicRange): 15 mm of coordinates over a
/// `1e-160` mm feature is `1.5e161`, and `f64` carries `1e11`.
const TOO_SMALL_TO_RESOLVE: f64 = 1e-160;

/// The two blocks, the second moved up by `hair`.
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

/// The edge across the sliver is as long as the sliver, and not nothing.
///
/// A tolerant policy warns about a spread this wide rather than refusing it, so
/// this fuse writes its answer. What it must not write is an edge whose two
/// ends are different corners reported as being in the very same place: the
/// ends really are `1e-200` apart, and reporting `0.0` for them throws away an
/// answer `f64` holds perfectly well.
///
/// Before the fix this panicked in `Assembly::commit` — the sweep read the
/// sliver as a feature of size `0.0` and said the model the fuse had just
/// written was invalid.
#[test]
fn the_edge_across_a_sliver_is_as_long_as_the_sliver_is() {
    let (mut model, [low, high]) = two_blocks_a_hair_apart(TOO_SMALL_TO_SQUARE);
    let policy = Policy::tolerant(model.scale().nominal_size());

    model
        .fuse(
            &[low, high],
            &[],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a tolerant policy warns about a wide spread rather than refusing it");

    let shortest = model
        .edges()
        .filter(|edge| {
            !edge
                .is_closed()
                .expect("a fuse of two boxes leaves straight edges")
        })
        .map(|edge| edge.distance_between_its_ends())
        .fold(f64::INFINITY, f64::min);
    assert_eq!(
        shortest, TOO_SMALL_TO_SQUARE,
        "the sliver is exactly one hair across"
    );
}

/// A certified fuse refuses the sliver instead of writing it.
///
/// The answer is real — the step between the two blocks is genuinely there —
/// but a model with a `1e-160` mm feature next to 15 mm coordinates asks `f64`
/// for more digits than it has, and `Model::validate` says so. Refusing is the
/// only honest outcome, because writing the answer and then asserting the model
/// is valid is a crash. It is a **limit** and not a gap: scaling the whole
/// answer by any factor moves the coordinates and the sliver together and
/// leaves the ratio of the two exactly where it was, so no scaled copy of this
/// answer fits either.
///
/// The hair here is `1e-160` and not the `1e-200` above, because a certified
/// fuse of the smaller pair never gets this far — the two bottom planes are so
/// close that `where two planes cross` cannot settle their crossing at all, and
/// the answer is `Undecidable` from a stage earlier.
#[test]
fn a_certified_fuse_refuses_a_sliver_too_thin_to_resolve_rather_than_writing_it() {
    let (mut model, [low, high]) = two_blocks_a_hair_apart(TOO_SMALL_TO_RESOLVE);

    let refused = model.fuse(
        &[low, high],
        &[],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    match refused {
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => {
            assert_eq!(ratio, 15.0 / TOO_SMALL_TO_RESOLVE);
        }
        other => panic!("expected a refusal naming the spread, got {other:?}"),
    }
    assert_eq!(
        model.solids().count(),
        2,
        "a refused fuse leaves the model alone"
    );
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("and leaves it valid");
}
