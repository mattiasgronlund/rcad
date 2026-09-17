//! Two pipes leaning against each other, their axes exactly `r₁ + r₂` apart,
//! touch at one point — and which of the infinitely many vectors along each
//! axis someone wrote down cannot change that.
//!
//! **This kernel does not have that property yet**, and `docs/decisions.md`
//! §118.11 is the argument. `Cylinder::side_of` was made homogeneous, so a pipe
//! there is exactly `r` wide whatever its stored direction's length rounded to;
//! `crates/rcad-geom/src/intersect.rs` was not converted and still works in the
//! old convention. `certified_radii_enclosing` hands back `rᵢ/√Wᵢ`, with
//! `Wᵢ = wᵢ·wᵢ` the squared length of the stored direction, and
//! `where_two_leaning_pipes_cross` decides the whole shape of a crossing from
//! three sign questions built out of those. `√Wᵢ` is not exactly one, so the
//! pipe the namer measures with is a rounding wider or narrower than the pipe
//! that was written down.
//!
//! So two of the five tests here are red, deliberately, until `intersect.rs`
//! uses `rᵢ`. The other three are the controls that say the red is the kernel's
//! and not the probe geometry's.
//!
//! # The one crossing every test here asks about, and why it has teeth
//!
//! Both pipes are `0.5` wide. The first runs through the coordinate origin, the
//! second through `(0, 0, 1)`, and the two directions are **mirror images in
//! the `xz` plane**: `(p, q, 0)` and `(p, −q, 0)`. That buys three things at
//! once, and each one is asserted below rather than trusted.
//!
//! - **The two axes are exactly one apart.** `Axis::new` stores some rounded
//!   `(a, b, 0)` and `(a, −b, 0)`; their cross product is `(0, 0, −2ab)`, which
//!   points straight down the `z` axis whatever `a` and `b` rounded to. The gap
//!   vector between the two origins is `(0, 0, 1)`, so the distance between the
//!   axes is `1` — exactly, as real arithmetic and not as a rounding, with
//!   nothing left over for the division by `√K` to spoil.
//! - **`0.5 + 0.5` is `1`**, exactly. So `δ = r₁ + r₂` on the nose: the two
//!   pipes rest against each other and share exactly one point.
//! - **Both stored directions have the same `W`**, bit for bit, because they
//!   are the same two magnitudes with a sign moved. That is what keeps the
//!   comparison inside the exact level: `√(W₁W₂)` is then rational and
//!   `how_two_radii_and_a_gap_compare_exactly` never has to square the wide
//!   expansion that §29 says falls off the bottom of `f64`.
//!
//! Write the first of the three sign questions out under the old convention.
//! Scaled by the positive `W₁W₂K`, `ρ₁ + ρ₂ − δ` has the sign of
//!
//! ```text
//! K(r₁ + r₂)² − T²W  =  K[(r₁ + r₂)² − W·δ²]  =  K(1 − W),
//! ```
//!
//! using `T² = Kδ²` and `δ = r₁ + r₂ = 1`. So on the base the answer to "do
//! these two pipes touch" is **the sign of `1 − W`, and nothing else**. Made
//! homogeneous the `W` goes away, the two sides cancel to nothing, and the
//! answer is `Zero` — one touching point — for every direction and every length
//! it was written at.
//!
//! # What would make these pass for the wrong reason
//!
//! - **A probe built somewhere the axes are not really one apart.** A failure
//!   then looks identical from the outside.
//!   `two_pipes_on_exactly_unit_axes_that_rest_against_each_other_touch` is the
//!   control for it: the same radii, the same gap and the same arithmetic, on
//!   the one pair of directions that stores with `W` exactly one. It passes
//!   today, so the only thing varying in the reds is the length the directions
//!   got stored at.
//! - **A gap so slack that both conventions agree.** One ulp is what separates
//!   them, and `one_ulp_wider_or_narrower_is_already_clear_or_already_crossing`
//!   shows the neighbouring two gaps are decided `Nothing` and `Curves`
//!   already. Nothing looser than the knife edge itself is left to ask about.
//! - **Checking two spellings of one pipe against each other.** Scaling a
//!   written direction by a small whole number does not move the stored bits at
//!   all — `Vector::scaled_to_length_one_from_anywhere` divides by the largest
//!   component first — so two spellings can agree perfectly and both be wrong
//!   by the same `1e-16`. Every assertion here is against the crossing the
//!   geometry has, and never against another spelling of it.
//! - **A defect that only ever pushes one way.** `W` lands below one for
//!   `(1, 1, 0)` and above one for `(1, 6, 0)`, so the two reds below fail in
//!   opposite directions: one reports a crossing where the pipes only touch,
//!   the other reports nothing at all.
//!   `the_two_shapes_store_their_directions_one_above_and_one_below_unit_length`
//!   pins that, and it is the difference between a defect and a probe placed
//!   somewhere careless.

use rcad_geom::{Axis, Cylinder, Point, Surface, SurfaceMeet, Vector, intersect};
use rcad_predicates::{Policy, ProvenanceLog};

/// The width both pipes are written with. Two of them add to exactly one.
const RADIUS: f64 = 0.5;

/// How far apart the two axes are put, which is exactly `RADIUS + RADIUS`.
const GAP: f64 = 1.0;

/// The two direction shapes the mirrored pairs are built from.
///
/// `(1, 1, 0)` stores a direction whose squared length is a rounding **below**
/// one and `(1, 6, 0)` one a rounding **above** it, which is what makes the two
/// cases below fail opposite ways on the base.
const SHAPES: [[f64; 3]; 2] = [[1.0, 1.0, 0.0], [1.0, 6.0, 0.0]];

/// The control: two pipes whose stored directions are exactly unit vectors
/// already touch at one point.
///
/// `(1, 0, 0)` and `(0, 1, 0)` normalize to themselves, so `W` is exactly one
/// for both and the factor the old convention leaves behind is the number one.
/// This is what tells a red below apart from a crossing built somewhere it was
/// not meant to be.
#[test]
fn two_pipes_on_exactly_unit_axes_that_rest_against_each_other_touch() {
    let pipes = two_pipes_resting_against_each_other(
        Vector::new(1.0, 0.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        GAP,
    );
    assert_eq!(
        pipes[0].axis().direction().components(),
        [1.0, 0.0, 0.0],
        "nothing rounded on the way in",
    );
    assert_eq!(
        pipes[1].axis().direction().components(),
        [0.0, 1.0, 0.0],
        "nothing rounded on the way in",
    );
    let meet = name_the_crossing(&pipes);
    assert!(
        matches!(meet, Ok(SurfaceMeet::TouchAtOnePoint { .. })),
        "two pipes {RADIUS} wide whose axes are {GAP} apart share one point, not {meet:?}",
    );
}

/// The two shapes store directions on opposite sides of unit length.
///
/// The three sign questions of a leaning crossing are decided on the base by
/// the sign of `1 − W`, so a defect that only ever pushed one way would be
/// indistinguishable from a crossing built a rounding off where it was meant to
/// be. These are the two `W`s the reds below run on, and they straddle one.
#[test]
fn the_two_shapes_store_their_directions_one_above_and_one_below_unit_length() {
    let squared_lengths = SHAPES.map(|shape| {
        let stored = Axis::new(Point::origin(), Vector::from(shape))
            .expect("a direction with a length")
            .direction();
        stored.length_squared()
    });
    assert!(
        squared_lengths[0] < 1.0,
        "{:?} stores a direction shorter than one: {:?}",
        SHAPES[0],
        squared_lengths[0],
    );
    assert!(
        squared_lengths[1] > 1.0,
        "{:?} stores a direction longer than one: {:?}",
        SHAPES[1],
        squared_lengths[1],
    );
}

/// Two pipes whose axes are exactly the two radii apart touch at one point,
/// whichever way the stored directions' length rounded.
///
/// This is the one that is red on the base, and it is red both ways round: the
/// shorter stored direction makes the namer measure with pipes a rounding too
/// wide and it reports a crossing, the longer one makes it measure with pipes a
/// rounding too thin and it reports nothing at all.
#[test]
fn two_pipes_that_rest_against_each_other_touch_whichever_way_their_length_rounded() {
    // Both shapes are named before anything is asserted, so a reader of the
    // failure sees which way each one went rather than only the first.
    let named = SHAPES.map(|shape| {
        let [across, mirrored] = mirrored_directions(shape);
        let pipes = two_pipes_resting_against_each_other(across, mirrored, GAP);
        let squared_length = pipes[0].axis().direction().length_squared();
        (shape, squared_length, name_the_crossing(&pipes))
    });
    assert!(
        named
            .iter()
            .all(|(_, _, meet)| matches!(meet, Ok(SurfaceMeet::TouchAtOnePoint { .. }))),
        "two pipes {RADIUS} wide whose axes are {GAP} apart share one point, whatever \
         their stored directions' length rounded to: {named:#?}",
    );
}

/// How tight the touching above is: one ulp either side of the gap is already
/// clear of the pipes or already cutting through them.
///
/// This is the evidence that the knife edge has teeth. One ulp at a gap of one
/// is `2.2e-16`, and the surfaces the un-homogeneous namer measures with sit
/// about half of that off the ones that were written down — so the knife edge
/// is the only gap of the three whose answer the defect can move, and it moves
/// it. A test run at a slacker gap would agree with both conventions and prove
/// nothing at all.
///
/// This one passes today, on the shape whose `W` is a rounding below one.
#[test]
fn one_ulp_wider_or_narrower_is_already_clear_or_already_crossing() {
    let [across, mirrored] = mirrored_directions(SHAPES[0]);

    let clear = name_the_crossing(&two_pipes_resting_against_each_other(
        across,
        mirrored,
        GAP.next_up(),
    ));
    assert!(
        matches!(clear, Ok(SurfaceMeet::Nothing)),
        "one ulp wider than {GAP} the two pipes miss each other, not {clear:?}",
    );

    let crossing = name_the_crossing(&two_pipes_resting_against_each_other(
        across,
        mirrored,
        GAP.next_down(),
    ));
    assert!(
        matches!(crossing, Ok(SurfaceMeet::Curves(_))),
        "one ulp narrower than {GAP} the two pipes cut through each other, not {crossing:?}",
    );
}

/// The wrong answer is not only on the knife edge: for the longer-stored shape
/// it reaches a whole ulp past it.
///
/// `(1, 6, 0)` stores a direction whose squared length is `2.5e-16` above one,
/// which is more than the `2.2e-16` an ulp of the gap is worth. So pulling the
/// two axes one ulp closer than `RADIUS + RADIUS` — where the two pipes
/// genuinely cut through each other, by a thin lens but by a real one — is
/// still not enough to get a crossing back out of the namer.
///
/// This one is red on the base too, and it is the answer to anyone reading the
/// red above as a knife-edge quibble: there is no knife edge here, the pipes
/// overlap, and the kernel says they miss.
#[test]
fn pipes_that_overlap_are_still_reported_as_missing_when_their_axes_stored_long() {
    let [across, mirrored] = mirrored_directions(SHAPES[1]);
    let narrower = GAP.next_down();
    let crossing = name_the_crossing(&two_pipes_resting_against_each_other(
        across, mirrored, narrower,
    ));
    assert!(
        matches!(crossing, Ok(SurfaceMeet::Curves(_))),
        "axes {narrower} apart is nearer than {RADIUS} + {RADIUS}, so two pipes about \
         {:?} and its mirror cut through each other, not {crossing:?}",
        SHAPES[1],
    );
}

/// A direction and its mirror image in the `xz` plane.
///
/// The mirroring is what makes the two stored directions the same two
/// magnitudes with a sign moved, so their squared lengths are equal bit for bit
/// and their cross product points straight down the `z` axis.
fn mirrored_directions(shape: [f64; 3]) -> [Vector; 2] {
    let [x, y, z] = shape;
    [Vector::new(x, y, z), Vector::new(x, -y, z)]
}

/// Two pipes [`RADIUS`] wide, the first about a line through the coordinate
/// origin and the second about a line `gap` above it in `z`.
///
/// The two directions have to store flat in the `xy` plane with a cross product
/// along `z`, because that is what makes the distance between the two axes
/// exactly `gap` rather than a rounding of it. Both are asserted on the
/// **stored** directions and not on the written ones.
fn two_pipes_resting_against_each_other(written: Vector, and: Vector, gap: f64) -> [Cylinder; 2] {
    let axes = [
        Axis::new(Point::origin(), written).expect("a direction with a length"),
        Axis::new(Point::new(0.0, 0.0, gap), and).expect("a direction with a length"),
    ];
    let ways = axes.map(|axis| axis.direction());
    for way in ways {
        assert_eq!(
            way.components()[2],
            0.0,
            "{way:?} was meant to store a direction lying flat in the xy plane",
        );
    }
    let sideways = ways[0].cross(ways[1]);
    assert_eq!(
        [sideways.components()[0], sideways.components()[1]],
        [0.0, 0.0],
        "the two axes were meant to have their common perpendicular along z, \
         so that the gap between them is exactly {gap}, but they cross to {sideways:?}",
    );
    assert_ne!(
        sideways.components()[2],
        0.0,
        "the two axes were meant to lean against each other rather than run parallel",
    );
    axes.map(|axis| Cylinder::new(axis, RADIUS).expect("a positive radius"))
}

/// What the kernel calls the crossing of two pipes.
fn name_the_crossing(pipes: &[Cylinder; 2]) -> rcad_geom::Result<SurfaceMeet> {
    intersect(
        &Surface::Cylinder(pipes[0]),
        &Surface::Cylinder(pipes[1]),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}
