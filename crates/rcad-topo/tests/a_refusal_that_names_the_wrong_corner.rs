//! Two ways the outline walk gave up and reported a corner it had decided.
//!
//! Both crash `Record::new` — *"orient3d: a certified outcome does not get a
//! provenance record"* — because a certified sign has no enclosure to record and
//! the walk handed one over anyway. A fuzzer found the first through
//! `Model::fuse` (`two_solids_fused_by_a_stranger`, artifact
//! `crash-f984851ca89020e1ff939c087667a0aa86bc352d`) after a soak of a model
//! whose coordinates were spread from `4.2e-314` to `2.4e21`. Neither test needs
//! the fuzzer, a spread, or a fuse: `SolidView::classify_uncertain_point` is
//! public and takes a box, which is the whole of what it takes to get here.
//!
//! `docs/decisions.md` §69 has the diagnosis and why the assert was right.
//!
//! # The one shape both need
//!
//! The walk runs a half-line from the point to the outline's first corner and
//! counts the sides crossing it, so what it asks about corner `k` is which side
//! of the *spoke* from the first corner to corner `k` the point falls. A box
//! sitting on one of those spokes cannot be told, and the walk stops — correctly.
//! What it then has to say is *which* corner it could not read.
//!
//! Getting a box onto a spoke and still reaching the walk needs a corner that
//! points back into the outline. The walk is only asked about a point already on
//! the face's plane, and the way a polygon says so is one of the triangles it
//! fans from its first corner reporting that the ray starts on it. Every fan
//! triangle beside a spoke has that spoke for an edge, so a box on a spoke grazes
//! both of its neighbours and neither can report anything. `OUTLINE`'s notch is
//! what fixes that: the fan overlaps itself there, so a spoke runs through the
//! middle of a triangle that is not beside it, and *that* triangle answers.

use rcad_geom::{Isometry, Scale};
use rcad_predicates::{
    Decision, Interval, Measures, Point as PredicatePoint, Policy, ProvenanceLog,
    orient3d_of_points,
};
use rcad_topo::{Containment, Model, ProfileBuilder, SolidKey};

/// A six-sided outline with a corner that points back into it.
///
/// `[6, 0]` is the notch: the outline turns right there where every other corner
/// turns left. It is what puts `[6, 0]` and the spoke to `[160, -80]` inside the
/// triangle fanned over `[0, 0]`, `[160, -80]`, `[160, 80]`, which is not beside
/// either of them.
///
/// The far pair is ten times out on purpose, and §69.6 has the arithmetic: it is
/// what separates the two enclosures the walk sees, so one is inside a tolerant
/// budget and the other is not.
const OUTLINE: [[f64; 2]; 6] = [
    [0.0, 0.0],
    [6.0, 0.0],
    [4.0, -4.0],
    [160.0, -80.0],
    [160.0, 80.0],
    [0.0, 80.0],
];

const HEIGHT: f64 = 4.0;

/// A point on a spoke the walk cannot read, and the walk names its neighbour.
///
/// The box straddles the spoke from `[0, 0]` to `[160, -80]` — the line
/// `y = -x/2`, which `[3, -1.5]` is exactly on — and is a fifth of a nanometre
/// wide, which is nothing beside a 160 mm part and still enough that no amount of
/// precision settles which side of that spoke the point is on.
///
/// The walk reaches the fourth corner with `sides` reading
/// `[Zero, Pos, Neg, Undecidable { enclosure: 2.56e-7 }, Pos, Pos]` and gives up
/// on the pair `(Neg, Undecidable)` — the same pair, in the same order, that the
/// fuzzer's model produced as `(Neg, Undecidable { enclosure: 3.65e47 })`.
/// Before the fix it reported the `Neg`.
///
/// The policy is tolerant because a certified one refuses the *first* corner —
/// the spoke to itself is a determinant with two equal rows, which is exactly
/// zero and which interval arithmetic cannot see is zero — and the walk stops
/// there having named the undecided corner correctly. §69.6.
#[test]
fn a_walk_that_cannot_read_one_corner_must_not_report_the_one_beside_it() {
    let (model, prism) = the_notched_prism();
    let on_the_spoke = a_flat_box([3.0, 3.0], [-1.5 - 2e-10, -1.5 + 2e-10], HEIGHT);

    let mut log = ProvenanceLog::new();
    let answer = model.solid(prism).classify_uncertain_point(
        on_the_spoke,
        &Policy::tolerant(100.0),
        &mut log,
    );

    let Containment::Undecidable { enclosure } = answer else {
        panic!("a point on a spoke nobody can read is not placeable, and got {answer:?}");
    };
    assert!(
        enclosure > 0.0 && enclosure.is_finite(),
        "the refusal has to carry what was measured, and carried {enclosure}"
    );
}

/// A point on a side, reaching past the end of it, and no predicate refused.
///
/// The box lies along the outline's first side — from `[0, 0]` to `[6, 0]`, so
/// its `y` is exactly zero and every `orient3d` about it certifies — and runs
/// from `x = 5` to `x = 7`, which reaches a whole millimetre past the corner at
/// `[6, 0]`. Whether the point is on the side or past its end is then an ordering
/// along one axis (§17.4), and an ordering is not a predicate: there is no
/// `orient3d` here that gave up, and before the fix the walk recorded one anyway.
///
/// `[6, 0]` being the notch is what lets the box reach past it and stay inside a
/// fan triangle, so this is the same corner doing the same work as above.
///
/// A certified policy, and nothing here needs a tolerant one: every decision on
/// this path really is decided, which is the point.
#[test]
fn a_refusal_no_predicate_made_must_not_be_recorded_as_one() {
    let (model, prism) = the_notched_prism();
    let past_the_corner = a_flat_box([5.0, 7.0], [0.0, 0.0], HEIGHT);

    let mut log = ProvenanceLog::new();
    let answer = model.solid(prism).classify_uncertain_point(
        past_the_corner,
        &Policy::certified(),
        &mut log,
    );

    let Containment::Undecidable { enclosure } = answer else {
        panic!("a box reaching past the end of a side is not placeable, and got {answer:?}");
    };
    assert!(
        enclosure > 0.0 && enclosure.is_finite(),
        "the refusal has to carry what was measured, and carried {enclosure}"
    );
}

/// The window the test above sits in, checked rather than trusted.
///
/// The two enclosures the walk sees have to fall either side of the tolerant
/// budget: the first corner's has to resolve, or the walk stops before it
/// reaches the spoke, and the spoke's must not, or there is nothing it cannot
/// read. Both are linear in the width of the box, so shrinking the box moves
/// them together — what separates them is `[160, -80]` sitting ten times out.
/// §69.6.
///
/// Without this, the test above could stop proving anything and still pass: a
/// walk that gave up at the first corner returns the same `Undecidable`.
#[test]
fn the_first_corner_resolves_and_the_spoke_does_not() {
    let policy = Policy::tolerant(100.0);
    let on_the_spoke = a_flat_box([3.0, 3.0], [-1.5 - 2e-10, -1.5 + 2e-10], HEIGHT);
    let first_corner = PredicatePoint::Exact([0.0, 0.0, HEIGHT]);
    let off_the_plane = PredicatePoint::Exact([0.0, 0.0, 2.0 * HEIGHT]);
    let far_corner = PredicatePoint::Exact([160.0, -80.0, HEIGHT]);

    let enclosure_of = |corner| {
        let outcome = orient3d_of_points(corner, first_corner, off_the_plane, on_the_spoke);
        let Decision::Undecidable { enclosure } = outcome else {
            panic!("this is the pair the walk cannot read, and got {outcome:?}");
        };
        enclosure
    };

    let at_the_first_corner = Measures::Volume.wobble_of(enclosure_of(first_corner), 100.0);
    let at_the_spoke = Measures::Volume.wobble_of(enclosure_of(far_corner), 100.0);

    assert!(
        policy.wobble_is_acceptable(at_the_first_corner),
        "the walk has to get past its first corner, and {at_the_first_corner} is too wide"
    );
    assert!(
        !policy.wobble_is_acceptable(at_the_spoke),
        "the walk has to have something it cannot read, and {at_the_spoke} resolves"
    );
}

fn the_notched_prism() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let outline = ProfileBuilder::new(OUTLINE.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a six-sided outline");
    let prism = model
        .add_extrusion(&outline, HEIGHT, &Isometry::identity())
        .expect("building a prism needs no decision");
    (model, prism)
}

/// A box spanning `x` and `y`, flat on the plane `z`.
///
/// Exact in `z` on purpose, and in one of `x` and `y` at each of the two call
/// sites. A box with room in `z` would straddle the face's own plane, and the
/// walk is only ever asked about a point already on it.
fn a_flat_box(x: [f64; 2], y: [f64; 2], z: f64) -> PredicatePoint {
    let span = |[lo, hi]: [f64; 2]| Interval::new(lo, hi).expect("a real range");
    PredicatePoint::Constructed([span(x), span(y), Interval::exactly(z)])
}
