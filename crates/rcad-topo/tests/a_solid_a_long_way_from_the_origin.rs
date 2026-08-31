//! Two defects that only show up when a model is written somewhere else in
//! `f64`, each cut down to the smallest case that reproduces it.
//!
//! Both were found by `pairs::a_pair_somewhere_else` — the family that takes an
//! existing lattice configuration and multiplies every coordinate by `2^k` and
//! moves it by `2^m`. Neither is fixed here. Both are **gaps** rather than
//! limits (`docs/decisions.md` §16.10): the answer exists and is exactly
//! representable, and the arithmetic we chose cannot reach it. Both are also
//! §16.11's open question with teeth — `DynamicRange` is a *ratio*, so it sees
//! nothing wrong with either of these models, and it is the absolute magnitude
//! that breaks them.
//!
//! They are `#[ignore]`d so the suite stays green, and the family's own range is
//! bounded to stay below both edges — `SMALLEST_SCALE`, `LARGEST_SCALE` and
//! `FURTHEST_SHIFT_EXPONENT` in `pairs/mod.rs` carry a measurement each. Remove
//! the attribute to see either of these fail.

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// `2^exponent`, exactly.
fn two_to_the(exponent: i32) -> f64 {
    f64::from_bits(u64::try_from(1023 + exponent).expect("a normal exponent") << 52)
}

/// A box of the given lengths, with its low corner at `at`.
fn block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3], at: Vector) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            along_z,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a box needs no decision")
}

/// A frame: a square with a square hole through it, walls one unit thick.
fn frame(model: &mut Model, side: f64, wall: f64, at: Vector) -> SolidKey {
    let square = |low: f64, high: f64| vec![[low, low], [high, low], [high, high], [low, high]];
    let profile = ProfileBuilder::new(square(0.0, side))
        .with_hole(square(wall, side - wall))
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a frame");
    model
        .add_extrusion(
            &profile,
            wall,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a frame needs no decision")
}

/// A cube whose faces sit further than about `2^72` from the origin cannot be
/// fused with anything, including itself.
///
/// **The defect.** `fuse/select.rs`'s `InPlane::of` needs a point it can *prove*
/// is off a face's plane, and it looks for one by stepping from the plane's own
/// nearest point to the origin along the normal, by `1.0`, then `1024.0`, then
/// `1048576.0`. Those are three absolute distances in the model's own unit. A
/// plane `2^73` from the origin has an `f64` spacing of `2^21`, wider than the
/// largest of the three, so every step lands back exactly on the plane, no
/// candidate can be proved to be off it, and the fuse returns
/// [`Error::Undecidable`](rcad_topo::Error::Undecidable) naming
/// `"a point off a face's plane"`.
///
/// **Why it is a gap and not a limit.** The answer exists: `A ∪ A` is `A`, every
/// coordinate involved is exactly representable, and points off the plane are
/// plentiful — any multiple of the plane's own spacing would do. The three
/// reaches are a *family of fixes* (§16.10) that considered only how large a
/// step to take and never that the right step depends on where the plane is. The
/// doc comment on `InPlane::of` says as much and then does not act on it: "a
/// bigger reach only helps when the plane's own offset is large enough to
/// swallow a smaller one."
///
/// **Where the edge is.** Measured over two overlapping three-by-three blocks,
/// the first power of two that refuses is `2^71` with no shift and `2^43` with a
/// `2^30` shift — the same plane offset of about `2^73` both times.
#[test]
#[ignore = "a gap: InPlane::of steps by three absolute distances, so a plane past 2^72 has no reachable point off it"]
fn a_cube_too_far_from_the_origin_cannot_be_fused_at_all() {
    let far = two_to_the(80);
    let mut model = Model::new(Scale::millimetres(100.0 * far).expect("a working size"));
    let one = block(&mut model, [far, far, far], Vector::zero());
    let copy = block(&mut model, [far, far, far], Vector::zero());

    let answer = model.fuse(
        &[one],
        &[copy],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        answer.is_ok(),
        "a cube united with a copy of itself is that cube, whatever size it is written at: {:?}",
        answer.err()
    );
}

/// A solid whose corners are more than about `2^28` from the origin is reported
/// as enclosing a negative amount of space.
///
/// **The defect.** `SolidView::measure` and `SolidView::signed_volume` sum
/// `a · (b × c)` over the answer's triangles, where `a`, `b` and `c` are
/// measured **from the world origin**. A solid `2^m` from the origin therefore
/// adds up terms of size `2^{3m}` to reach a volume of order one, and past
/// `m = 28` the `f64` spacing of those terms is larger than the answer. What
/// comes back has no correct digits and can be negative, at which point
/// `Model::validate` — which the fuse runs over its own answer in a debug build
/// — refuses a perfectly good solid as one whose faces do not point outwards.
///
/// **Why it is a gap and not a limit.** The volume is a small whole number and
/// exactly representable; nothing about `f64` prevents computing it. Summing the
/// same triangles about the solid's own first corner instead of the world origin
/// gives the exact answer, because the differences are exact by Sterbenz's lemma
/// when the coordinates share an exponent. The family of fixes considered when
/// the divergence theorem was written down was "which point do the tetrahedra
/// hang from", and the origin was chosen for tidiness.
///
/// **Where the edge is.** Measured by uniting each of the four generated shapes
/// with a copy of itself at growing distances: a `3 × 3` frame with walls one
/// thick — the smallest volume of the four, at eight — is the first to go, at
/// `2^28`. A one-by-one cube survives to `2^30`, because a box's terms cancel in
/// opposite pairs and a frame's do not. `DynamicRange` sees a ratio of `2.7e8`
/// here and is satisfied, which is §16.11 exactly.
#[test]
#[ignore = "a gap: signed_volume sums tetrahedra hung from the world origin, so a solid past 2^28 away loses every digit"]
fn a_frame_a_long_way_from_the_origin_is_said_to_enclose_negative_space() {
    let far = two_to_the(28);
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let one = frame(&mut model, 3.0, 1.0, Vector::new(far, far, far));
    let copy = frame(&mut model, 3.0, 1.0, Vector::new(far, far, far));

    let answer = model.fuse(
        &[one],
        &[copy],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        answer.is_ok(),
        "a frame united with a copy of itself encloses what the frame does, \
         however far from the origin it is drawn: {:?}",
        answer.err()
    );
}
