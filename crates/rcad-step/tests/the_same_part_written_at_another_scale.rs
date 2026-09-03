//! The same part, written twice at two scales, has to come back as the same
//! part at those two scales.
//!
//! This is §19's shape one level up. §19 asks that a *movement* applied to a
//! whole model move it and change nothing else; this asks the same of a
//! *scaling* applied to the file before the reader ever sees it. It is the
//! strongest property an importer can have, because it is about the reader and
//! the builder and the sweep and the measurement all at once, and a mapping that
//! rounded anywhere it did not have to would break it.
//!
//! **A power of two is exact and everything else is not**, and the two halves
//! are tested apart. Multiplying an `f64` by a power of two moves the exponent
//! and touches no bit of the mantissa, so a file written at `2^k` times the size
//! holds *exactly* the numbers of the first one scaled — and so must every
//! number that comes out. A thousand is not a power of two: a part written in
//! metres instead of millimetres has each of its coordinates rounded once on the
//! way into the file, and what that costs is [`WHAT_A_SCALE_THAT_IS_NOT_A_POWER_OF_TWO_COSTS`].

mod written_by_hand;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::a_block;

/// How far apart two measurements of the same part can be when one file's
/// numbers are the other's divided by something that is not a power of two.
///
/// Relative, and it is four roundings deep. Each of the three sides is one
/// division and carries at most half a coordinate spacing, `u = ε/2`. A block's
/// volume is degree three in its coordinates, so those alone move it by at most
/// `3u`. The fan the measurement sums has the same shape and the same order for
/// both files — the same faces in the same order, because the reader writes the
/// file's own order — so what it adds is its own rounding on a different set of
/// numbers, and for a block that is a handful of terms. **Eight epsilons**
/// leaves room for the handful and is the number
/// `a_part_written_in_metres_is_the_same_part_as_one_written_in_millimetres`
/// checks, which is what a published bound owes (§15.6).
pub const WHAT_A_SCALE_THAT_IS_NOT_A_POWER_OF_TWO_COSTS: f64 = 8.0 * f64::EPSILON;

/// The volume, the surface area and the centroid of the one solid in a file.
fn measure(text: &str) -> (f64, f64, [f64; 3]) {
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &Policy::certified(), &mut log).expect("a block imports");
    import
        .model()
        .validate(&Policy::certified(), &mut log)
        .expect("and the model it made is sound");
    let [(_, key)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let measured = import
        .model()
        .solid(*key)
        .measure(
            Deflection::of(1.0).expect("a block is flat, so nothing is meshed"),
            &Policy::certified(),
            &mut log,
        )
        .expect("a block measures");
    (
        measured.volume(),
        measured.surface_area(),
        measured.centroid().coordinates(),
    )
}

/// The same block at a hundred and one scales, every one of them a power of
/// two, from a part a thousand million million times smaller than the first to
/// one that much bigger.
///
/// Every comparison is `assert_eq!` on `f64`. Nothing here is allowed to be
/// close.
#[test]
fn a_part_written_at_a_power_of_two_scale_is_exactly_the_same_part_at_that_scale() {
    let sides = [2.0, 4.0, 8.0];
    let (volume, area, centroid) = measure(&a_block(sides, ".MILLI."));
    assert_eq!(volume, 64.0);

    for power in -50..=50 {
        let factor = f64::from(2.0_f32).powi(power);
        let (scaled_volume, scaled_area, scaled_centroid) =
            measure(&a_block(sides.map(|side| side * factor), ".MILLI."));

        assert_eq!(
            scaled_volume,
            volume * factor * factor * factor,
            "a block written at 2^{power} times the size encloses 2^{} times the space",
            3 * power
        );
        assert_eq!(scaled_area, area * factor * factor);
        for axis in 0..3 {
            assert_eq!(scaled_centroid[axis], centroid[axis] * factor);
        }
    }
}

/// The unit the file declares does not scale anything, and that is the whole
/// reason a unit costs nothing.
///
/// The same numbers under four different unit declarations give four models
/// with the same coordinates and four different units. `Scale` carries a unit
/// and never multiplies by one (§3.5), so this is an `assert_eq!` and not a
/// bound.
#[test]
fn declaring_a_different_unit_does_not_move_a_single_number() {
    let sides = [2.0, 4.0, 8.0];
    let (volume, area, centroid) = measure(&a_block(sides, ".MILLI."));
    for unit in ["$", ".CENTI.", ".MICRO."] {
        let (other_volume, other_area, other_centroid) = measure(&a_block(sides, unit));
        assert_eq!(other_volume, volume);
        assert_eq!(other_area, area);
        assert_eq!(other_centroid, centroid);
    }
}

/// The same part written in metres instead of millimetres, which divides every
/// coordinate by a thousand.
///
/// A thousand is `2^3 · 125` and the `125` is what rounds. So this one is a
/// bound and not an equality, and the bound is
/// [`WHAT_A_SCALE_THAT_IS_NOT_A_POWER_OF_TWO_COSTS`].
#[test]
fn a_part_written_in_metres_is_the_same_part_as_one_written_in_millimetres() {
    let millimetres = [2.0, 4.0, 8.0];
    let metres = millimetres.map(|side| side / 1000.0);
    // `q * 1000` worked out exactly, against the number it came from. A fused
    // multiply-add rounds once at the end and not in the middle, so this is the
    // division's own leftover and not another rounding on top of it.
    assert!(
        metres
            .iter()
            .zip(millimetres)
            .any(|(small, big)| f64::mul_add(*small, -1000.0, big) != 0.0),
        "if a thousandth were exact this test would be checking nothing"
    );

    let (big_volume, big_area, big_centroid) = measure(&a_block(millimetres, ".MILLI."));
    let (small_volume, small_area, small_centroid) = measure(&a_block(metres, "$"));

    let worst = [
        how_far_apart(big_volume, small_volume * 1e9),
        how_far_apart(big_area, small_area * 1e6),
        how_far_apart(big_centroid[0], small_centroid[0] * 1e3),
        how_far_apart(big_centroid[1], small_centroid[1] * 1e3),
        how_far_apart(big_centroid[2], small_centroid[2] * 1e3),
    ]
    .into_iter()
    .fold(0.0, f64::max);

    println!("a thousandth costs {} epsilons", worst / f64::EPSILON);
    assert!(
        worst <= WHAT_A_SCALE_THAT_IS_NOT_A_POWER_OF_TWO_COSTS,
        "a thousandth moved the part by {worst}, past the published \
         {WHAT_A_SCALE_THAT_IS_NOT_A_POWER_OF_TWO_COSTS}"
    );
    assert!(
        worst > 0.0,
        "if a thousandth cost nothing at all the bound would be describing something \
         that does not happen"
    );
}

/// How far apart two numbers are, as a fraction of the bigger one.
fn how_far_apart(one: f64, other: f64) -> f64 {
    (one - other).abs() / one.abs().max(other.abs())
}
