//! How far from the origin a model can be written, and what stops it there.
//!
//! Two defects lived here, each cut down to the smallest case that reproduced
//! it: a cube whose faces sat past about `2^72` could not be fused at all, and a
//! solid past about `2^28` was said to enclose negative space. Both were found
//! by `pairs::a_pair_somewhere_else` — the family that takes an existing lattice
//! configuration and multiplies every coordinate by `2^k` and moves it by `2^m`
//! — and both were **gaps** rather than limits (`docs/decisions.md` §16.10): the
//! answer existed and was exactly representable, and the arithmetic we chose
//! could not reach it. Both are fixed, and each test now runs out to where the
//! shape itself stops fitting in an `f64`, with a companion test naming what
//! ends the range.
//!
//! They were also §16.11's open question with teeth. `DynamicRange` is a
//! *ratio*, so it saw nothing wrong with either model — and it saw even less
//! than that, because it read the coordinates as the arena stores them rather
//! than where the solids are; `Model::check_dynamic_range` is where that is
//! written up.
//!
//! `SMALLEST_SCALE`, `LARGEST_SCALE` and `FURTHEST_SHIFT_EXPONENT` in
//! `pairs/mod.rs` bound the generated family's own range, and carry a
//! measurement each.

use rcad_geom::{Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

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

/// A cube fuses with a copy of itself at every size `f64` can hold the answer
/// at, up to and including `2^341`.
///
/// It could not past about `2^72`. `fuse/select.rs`'s `InPlane::of` needs a
/// point it can *prove* is off a face's plane, and it looked for one by stepping
/// from the plane's own nearest point to the origin along the normal by `1.0`,
/// then `1024.0`, then `1048576.0` — three absolute distances in the model's own
/// unit. A plane `2^73` out has an `f64` spacing of `2^21`, wider than the
/// largest of the three, so every step landed back exactly on the plane, no
/// candidate could be proved off it, and the fuse returned
/// [`Error::Undecidable`](rcad_topo::Error::Undecidable) naming
/// `"a point off a face's plane"`. Measured over two overlapping three-by-three
/// blocks, the first power of two that refused was `2^71` with no shift and
/// `2^43` with a `2^30` shift — the same plane offset of about `2^73` both
/// times. The reach is now a fraction of the plane's own offset, so there is no
/// size at which it vanishes.
///
/// **Where the range ends now, and why that end is a limit.** `2^342`, and the
/// fuse has nothing to do with it: a cube that big encloses `2^1026` of space,
/// which is past the largest `f64`. The answer does not exist in the number
/// type, which is what makes it a limit rather than a gap
/// (`docs/decisions.md` §16.10).
///
/// **This paragraph used to say `2^341`, and called that a limit too.** The
/// reason it gave — "a cube that big encloses `2^1023` of space, *six times
/// which* is past the largest `f64`" — is the gap test failing on its own
/// evidence. `2^1023` is a perfectly ordinary `f64`; only the `6 ×` on the way
/// to it was not, because `signed_volume` scaled its sum back up in one
/// multiplication. The answer existed and the arithmetic could not reach it,
/// which is §16.10's definition of a gap, written down two lines under a
/// sentence calling it a limit. `SolidView::measure` now undoes its rescale one
/// factor at a time and `2^341` measures, so the range is one binade wider and
/// the end of it is real.
#[test]
fn a_cube_a_long_way_from_the_origin_fuses_with_a_copy_of_itself() {
    for exponent in [0, 36, 71, 72, 73, 100, 200, 340, 341] {
        let far = two_to_the(exponent);
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
            "a cube united with a copy of itself is that cube, and this one is written at 2^{exponent}: {:?}",
            answer.err()
        );
    }
}

/// The end of that range, and what ends it.
///
/// A published bound needs a test (`docs/decisions.md` §15.6). The one above
/// says `2^342` is out of reach because the volume itself does not fit rather
/// than because anything in the fuse gave up, so this is the sentence that would
/// have to change if the fuse ever became the reason again.
///
/// **The refusal is not `Invalid`.** A cube of side `2^342` has its faces
/// pointing outwards and classifies points correctly; what it does not have is
/// a volume `f64` can hold. Calling that a broken model, which this test used
/// to assert, said something false about the solid.
#[test]
fn the_first_cube_too_big_to_fuse_is_the_first_whose_volume_overflows() {
    let far = two_to_the(342);
    let mut model = Model::new(Scale::millimetres(100.0 * far).expect("a working size"));
    let one = block(&mut model, [far, far, far], Vector::zero());
    let copy = block(&mut model, [far, far, far], Vector::zero());

    let refusal = model
        .fuse(
            &[one],
            &[copy],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("a cube of 2^1026 units of space is past what an f64 volume holds");
    assert!(
        matches!(refusal, Error::VolumeTooLargeToMeasure),
        "the fuse decided everything it was asked; the volume is what overflowed: {refusal:?}"
    );
}

/// The largest cube that still measures, and all three answers for it.
///
/// The binade below the bound above, and the one the old arithmetic could not
/// reach. All three numbers are exact: `2^1023`, `6 × 2^682`, and half the side.
#[test]
fn the_largest_cube_that_fits_measures_its_volume_area_and_middle() {
    let side = two_to_the(341);
    let mut model = Model::new(Scale::millimetres(100.0 * side).expect("a working size"));
    let cube = block(&mut model, [side, side, side], Vector::zero());

    let measured = model
        .solid(cube)
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a cube of 2^1023 units of space is the largest that fits");
    assert_eq!(measured.volume(), two_to_the(1023));
    assert_eq!(measured.surface_area(), 6.0 * two_to_the(682));
    assert_eq!(
        measured.centroid(),
        Point::new(side / 2.0, side / 2.0, side / 2.0)
    );
}

/// A frame united with a copy of itself encloses eight units of space wherever
/// it is drawn, out to `2^50`.
///
/// It was said to enclose *negative* space past about `2^28`. `SolidView::measure`
/// and `SolidView::signed_volume` sum `a · (b × c)` over the answer's triangles,
/// and `a`, `b` and `c` used to be measured **from the world origin**, so a solid
/// `2^m` out added up terms of size `2^{3m}` to reach a volume of order one. Past
/// `m = 28` the `f64` spacing of those terms was larger than the answer, what came
/// back had no correct digits and could be negative, and `Model::validate` — which
/// the fuse runs over its own answer in a debug build — refused a perfectly good
/// solid as one whose faces do not point outwards. The tetrahedra now hang from a
/// corner of the solid, so every term is the size of the solid and the three
/// subtractions are exact by Sterbenz's lemma when the coordinates share an
/// exponent, which is exactly this case.
///
/// A `3 × 3` frame with walls one thick is the shape that used to go first,
/// because its volume is the smallest of the four the generators draw and a box's
/// terms cancel in opposite pairs where a frame's do not. `DynamicRange` saw a
/// ratio of `2.7e8` here and was satisfied, which is §16.11 exactly.
///
/// **Where the range ends now.** `2^51` and `2^52` refuse, and past `2^53` there
/// is no frame to fuse: a wall one unit thick against a coordinate of `2^53` is
/// below the spacing there, so the two corners of a wall are the same `f64` and
/// the extrusion has an edge with no length. Both are limits rather than gaps —
/// the shape stops existing in the number type. Past about `2^37` the model is
/// outside [`DynamicRange::LARGEST_SUPPORTED_RATIO`] and a validate boundary says
/// so; that is a separate statement from the volume, which stays exact.
///
/// [`DynamicRange::LARGEST_SUPPORTED_RATIO`]: rcad_geom::DynamicRange::LARGEST_SUPPORTED_RATIO
#[test]
fn a_frame_a_long_way_from_the_origin_encloses_what_a_frame_encloses() {
    for exponent in [0, 28, 30, 36, 40, 50] {
        let far = two_to_the(exponent);
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let one = frame(&mut model, 3.0, 1.0, Vector::new(far, far, far));
        let copy = frame(&mut model, 3.0, 1.0, Vector::new(far, far, far));

        let united = model
            .fuse(
                &[one],
                &[copy],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| {
                panic!("a frame at 2^{exponent} unites with a copy of itself: {why}")
            });

        let volume: f64 = united
            .solids()
            .iter()
            .map(|&solid| {
                model
                    .solid(solid)
                    .measure(
                        a_deflection(),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .expect("a solid this size measures")
                    .volume()
            })
            .sum();
        assert_eq!(
            volume, 8.0,
            "a 3 by 3 frame with walls one thick encloses eight, and this one is at 2^{exponent}"
        );
    }
}

/// The end of *that* range, and what ends it.
///
/// A published bound needs a test (`docs/decisions.md` §15.6). The frame above
/// stops at `2^50` because of what `f64` can hold, not because of how the volume
/// is summed, and this is the sentence that says so: at `2^53` a wall one unit
/// thick has both its corners on the same number.
#[test]
fn a_frame_so_far_out_that_a_wall_is_thinner_than_the_spacing_is_not_a_frame() {
    let far = two_to_the(53);
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let one = frame(&mut model, 3.0, 1.0, Vector::new(far, far, far));
    let copy = frame(&mut model, 3.0, 1.0, Vector::new(far, far, far));

    let refusal = model
        .fuse(
            &[one],
            &[copy],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("a wall of one unit at 2^53 has no thickness to fuse");
    assert!(
        matches!(refusal, Error::Invalid(Invalid::EdgeHasNoLength)),
        "the frame is what stopped being a frame, not the fuse: {refusal:?}"
    );
}

/// The deflection the measurements here are asked for.
///
/// Every solid in this file is a polyhedron, and a polyhedron ignores it: its
/// own fan is the answer rather than a picture of one (`docs/decisions.md`
/// §25.4).
fn a_deflection() -> Deflection {
    Deflection::of(0.1).expect("a tenth of a millimetre is a length")
}
