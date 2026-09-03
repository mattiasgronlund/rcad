//! Measuring a distance must not care how big the numbers are.
//!
//! `Point::distance_to` used to add up the squares of the three differences and
//! take the root, which is right for numbers of an ordinary size and wrong at
//! both ends of the `f64` range. A separation of `3.8e-211` squares to
//! `1.4e-421`, below the smallest `f64` there is, so the length came back
//! **exactly zero** and two distinct points were reported as one — the
//! confident wrong answer `docs/decisions.md` §4.1 ranks below both crashing
//! and refusing. A separation of `1e200` squares to infinity and came back
//! infinite. Both answers are ordinary `f64`s the intermediate could not hold,
//! so both are **gaps** and not limits (§16.10), and the fix is the one this
//! milestone has now reached for three times: recondition the arithmetic with
//! an exact power-of-two rescale rather than widen anything.
//!
//! The oracle is the strongest one there is — exactly true. A `3, 4, 5`
//! triangle scaled by a power of two has sides that are `f64`s to the last bit
//! and a hypotenuse that is one too, at every exponent from the smallest
//! subnormal that can hold `5 · 2^e` up to the largest that does not overflow.
//! So the whole sweep asserts equality with no tolerance anywhere.
//!
//! It is not a tautology because the arithmetic it replaced is still here, in
//! `squares_and_a_root`, and the test counts how many of the same triangles that
//! one gets wrong. If that count is not large, this file is testing nothing and
//! says so.

use rcad_geom::{Point, Vector};

/// Every exponent where `3 · 2^e`, `4 · 2^e` and `5 · 2^e` are all exact.
///
/// The bottom is `-1072`: `5 · 2^-1072` is twenty of the smallest subnormals,
/// which needs three significant bits and has them. The top is `1021`, the
/// largest exponent at which `5 · 2^e` is still finite.
const SMALLEST_EXPONENT: i32 = -1072;
const LARGEST_EXPONENT: i32 = 1021;

#[test]
fn a_three_four_five_triangle_has_a_hypotenuse_of_five_at_every_scale() {
    let mut measured = 0usize;
    let mut squares_got_it_wrong = 0usize;

    for exponent in SMALLEST_EXPONENT..=LARGEST_EXPONENT {
        let scale = two_to_the(exponent);
        let corner = Point::new(3.0 * scale, 4.0 * scale, 0.0);
        let five = 5.0 * scale;

        assert!(
            (3.0 * scale) / scale == 3.0 && five / scale == 5.0,
            "2^{exponent} does not hold a 3, 4, 5 triangle exactly, so this row \
             is testing the generator and not the kernel"
        );

        assert_eq!(
            Point::origin().distance_to(corner),
            five,
            "a 3, 4, 5 triangle scaled by 2^{exponent}"
        );
        measured += 1;

        if squares_and_a_root(corner) != five {
            squares_got_it_wrong += 1;
        }
    }

    println!(
        "{measured} scaled copies of one triangle: adding up the squares first gets \
         {squares_got_it_wrong} of them wrong ({:.1}%)",
        100.0 * squares_got_it_wrong as f64 / measured as f64
    );
    assert!(
        measured > 2000,
        "only {measured} exponents, which is not the sweep this claims to be"
    );
    assert!(
        squares_got_it_wrong > 400,
        "the arithmetic this replaced is meant to fail over most of the range, and \
         here it failed only {squares_got_it_wrong} times out of {measured}"
    );
}

/// Two raised to `exponent`, in two halves.
///
/// `2.0f64.powi(-1072)` is not `2^-1072`: `powi` raises to the positive power
/// and takes the reciprocal, and `2^1072` is already an infinity, so the answer
/// comes back zero. Two halves each stay in range, and the product of two exact
/// powers of two is exact all the way down to the smallest subnormal.
fn two_to_the(exponent: i32) -> f64 {
    let half = exponent / 2;
    let scale = 2.0f64.powi(half) * 2.0f64.powi(exponent - half);
    assert!(
        scale != 0.0 && scale.is_finite(),
        "2^{exponent} is not an f64"
    );
    scale
}

/// The same distance the old way, kept so the comparison above is a measurement
/// rather than a memory.
fn squares_and_a_root(corner: Point) -> f64 {
    let [x, y, z] = corner.coordinates();
    (x * x + y * y + z * z).sqrt()
}

/// The rescale must not touch what it was not written for.
///
/// A power-of-two rescale that fired in the ordinary range would move lengths
/// all over the kernel, and this is the check that it does not: over every
/// direction on a small lattice, at exponents from `2^-508` to `2^508` — where
/// even the largest component on the lattice has a normal square — the answer
/// is bit for bit what adding up the squares gives.
#[test]
fn a_length_of_an_ordinary_size_is_the_same_bits_it_always_was() {
    let mut compared = 0usize;

    for x in -4i32..=4 {
        for y in -4i32..=4 {
            for z in -4i32..=4 {
                for exponent in [-508, -400, -100, -20, 0, 20, 100, 400, 508] {
                    let scale = two_to_the(exponent);
                    let corner = Point::new(
                        f64::from(x) * scale,
                        f64::from(y) * scale,
                        f64::from(z) * scale,
                    );
                    if corner == Point::origin() {
                        continue;
                    }
                    assert_eq!(
                        Point::origin().distance_to(corner),
                        squares_and_a_root(corner),
                        "{x}, {y}, {z} at 2^{exponent}"
                    );
                    compared += 1;
                }
            }
        }
    }

    println!("{compared} lengths of an ordinary size, all of them unmoved");
    assert!(
        compared > 6000,
        "only {compared} lengths compared, which is not the sweep this claims to be"
    );
}

/// What `f64::hypot` would have cost, measured rather than asserted.
///
/// `hypot` is the obvious instrument for this and it does work. It is a
/// different rounding of the same real number, though, and this counts how
/// often it disagrees with the arithmetic the kernel has always used, over
/// lengths that are nowhere near either end of the range. Every disagreement
/// here is a length that would have moved for no reason.
#[test]
fn hypot_would_have_moved_lengths_that_had_nothing_wrong_with_them() {
    let mut state = 0x243F_6A88_85A3_08D3u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        ((state >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    };

    let mut drawn = 0usize;
    let mut hypot_differed = 0usize;
    let mut worst_ulps = 0i64;

    for _ in 0..200_000 {
        let (x, y, z) = (next(), next(), next());
        let ours = Vector::new(x, y, z).length();
        let theirs = x.hypot(y).hypot(z);
        drawn += 1;
        if ours != theirs {
            hypot_differed += 1;
            worst_ulps = worst_ulps.max((ours.to_bits() as i64 - theirs.to_bits() as i64).abs());
        }
    }

    println!(
        "{drawn} lengths in the unit cube: hypot gave different bits for {hypot_differed} of them \
         ({:.1}%), by up to {worst_ulps} ulps",
        100.0 * hypot_differed as f64 / drawn as f64
    );
    assert!(
        hypot_differed > 0,
        "hypot is meant to be a different rounding; if it agrees on all {drawn} of these then the \
         reason this is not hypot has gone away and the doc on Vector::length is wrong"
    );
}
