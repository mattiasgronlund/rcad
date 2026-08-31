//! Exact `+`, `-` and `*` on sums of `f64`, after Shewchuk.
//!
//! An *expansion* is a list of `f64` whose sum is the value being represented.
//! The list is kept sorted by increasing magnitude and the entries never
//! overlap in the bits they occupy, so the largest entry alone decides the sign
//! of the whole thing. Every operation here is exact: no rounding is ever
//! thrown away, it is captured as another entry.
//!
//! This is level 3 of the cascade. It only ever has to produce a *sign*, so
//! there is no division and no rational arithmetic — dyadic fractions are
//! closed under `+`, `-` and `*`, which is all we need (decisions.md §2.3).
//!
//! The operations are Shewchuk's `fast_expansion_sum_zeroelim` and
//! `scale_expansion_zeroelim` from "Adaptive Precision Floating-Point
//! Arithmetic and Fast Robust Geometric Predicates" (1997), with his
//! `Fast_Two_Sum` shortcut replaced by the unconditional [`two_sum`]. The
//! shortcut is only correct when the first argument is the larger of the two;
//! giving up a branch buys an operation that cannot be misused.
//!
//! Every function writes into a caller-supplied slice, so a predicate that
//! knows its own sizes never allocates.
//!
//! ```
//! # use rcad_predicates::expansion::{expansion_sign, expansion_sum, two_sum};
//! # use rcad_predicates::Decision;
//! // 2^60 + 1 is not an f64, but it is an expansion of two entries, and
//! // subtracting 2^60 from it leaves the 1 that plain f64 threw away.
//! let (high, low) = two_sum(2f64.powi(60), 1.0);
//! let mut difference = [0.0; 3];
//! let len = expansion_sum(&[low, high], &[-(2f64.powi(60))], &mut difference);
//! assert_eq!(&difference[..len], &[1.0]);
//! assert_eq!(expansion_sign(&difference[..len]), Decision::Pos);
//! ```

use crate::Decision;

/// The exact sum of two expansions, written into `out`.
///
/// Returns how many entries of `out` were used. Entries that came out exactly
/// zero are dropped, so the answer is often much shorter than the inputs.
///
/// # Panics
///
/// If `out` is shorter than `left.len() + right.len()`, which is the most the
/// sum can need.
pub fn expansion_sum(left: &[f64], right: &[f64], out: &mut [f64]) -> usize {
    assert!(
        out.len() >= left.len() + right.len(),
        "an expansion sum needs room for every entry of both inputs"
    );
    if left.is_empty() && right.is_empty() {
        return 0;
    }

    let mut taken_from_left = 0;
    let mut taken_from_right = 0;
    let mut used = 0;
    let mut running = next_smallest(left, &mut taken_from_left, right, &mut taken_from_right);
    while taken_from_left < left.len() || taken_from_right < right.len() {
        let entry = next_smallest(left, &mut taken_from_left, right, &mut taken_from_right);
        let (sum, roundoff) = two_sum(running, entry);
        running = sum;
        if roundoff != 0.0 {
            out[used] = roundoff;
            used += 1;
        }
    }
    if running != 0.0 || used == 0 {
        out[used] = running;
        used += 1;
    }
    used
}

/// Takes the entry with the smaller magnitude from whichever input still has
/// one, so the merged stream stays sorted by increasing magnitude.
fn next_smallest(
    left: &[f64],
    taken_from_left: &mut usize,
    right: &[f64],
    taken_from_right: &mut usize,
) -> f64 {
    let take_left = if *taken_from_left >= left.len() {
        false
    } else if *taken_from_right >= right.len() {
        true
    } else {
        left[*taken_from_left].abs() <= right[*taken_from_right].abs()
    };

    if take_left {
        let entry = left[*taken_from_left];
        *taken_from_left += 1;
        entry
    } else {
        let entry = right[*taken_from_right];
        *taken_from_right += 1;
        entry
    }
}

/// The exact product of an expansion and a single `f64`, written into `out`.
///
/// Returns how many entries of `out` were used.
///
/// # Panics
///
/// If `out` is shorter than `2 * expansion.len()`, which is the most the
/// product can need.
pub fn scale_expansion(expansion: &[f64], factor: f64, out: &mut [f64]) -> usize {
    assert!(
        out.len() >= 2 * expansion.len(),
        "scaling an expansion can double its length"
    );
    if expansion.is_empty() {
        return 0;
    }

    let (mut running, roundoff) = two_product(expansion[0], factor);
    let mut used = 0;
    if roundoff != 0.0 {
        out[used] = roundoff;
        used += 1;
    }
    for &entry in &expansion[1..] {
        let (product, product_roundoff) = two_product(entry, factor);
        let (sum, sum_roundoff) = two_sum(running, product_roundoff);
        if sum_roundoff != 0.0 {
            out[used] = sum_roundoff;
            used += 1;
        }
        let (carried, carry_roundoff) = two_sum(product, sum);
        running = carried;
        if carry_roundoff != 0.0 {
            out[used] = carry_roundoff;
            used += 1;
        }
    }
    if running != 0.0 || used == 0 {
        out[used] = running;
        used += 1;
    }
    used
}

/// The sign of the value an expansion represents.
///
/// The entries never overlap, so the last one — the one with the largest
/// magnitude — outweighs everything before it and settles the sign on its own.
///
/// An entry that is not finite means a product overflowed on the way in. There
/// is no honest sign for that, so it comes back as
/// [`Decision::Undecidable`] with an infinite enclosure rather than a
/// confident answer built on `inf`.
pub fn expansion_sign(expansion: &[f64]) -> Decision {
    if !expansion.iter().all(|entry| entry.is_finite()) {
        return Decision::Undecidable {
            enclosure: f64::INFINITY,
        };
    }
    let Some(&largest) = expansion.last() else {
        return Decision::Zero;
    };
    if largest > 0.0 {
        Decision::Pos
    } else if largest < 0.0 {
        Decision::Neg
    } else {
        Decision::Zero
    }
}

/// `a + b` as a rounded sum plus the exact amount that rounding lost.
///
/// The two returned values add up to `a + b` exactly whenever that sum is
/// finite — for finite `a` and `b`, whenever it does not overflow. Otherwise
/// the roundoff term is worked out from a difference of infinities and comes
/// back `NaN`, and the pair stands for nothing. Nothing here rejects that; see
/// [`two_diff`] for what does.
///
/// Knuth's six-operation version, which needs no comparison between `a` and
/// `b`.
pub fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let sum = a + b;
    let b_part = sum - a;
    let a_part = sum - b_part;
    (sum, (a - a_part) + (b - b_part))
}

/// `a - b` as a rounded difference plus the exact amount that rounding lost.
///
/// The two returned values add up to `a - b` exactly whenever that difference
/// is finite — for finite `a` and `b`, whenever it does not overflow. Otherwise
/// the roundoff term comes back `NaN`, as in [`two_sum`], and the pair stands
/// for nothing.
///
/// Neither routine rejects its own overflow, and it is [`expansion_sign`] that
/// catches it downstream. That works because [`expansion_sum`] and
/// [`scale_expansion`] drop an entry only when it compares equal to zero, and
/// no infinity or `NaN` ever does — not even after being scaled by zero. So a
/// non-finite entry always reaches the sign, which refuses to name one for it.
pub fn two_diff(a: f64, b: f64) -> (f64, f64) {
    let difference = a - b;
    let b_part = a - difference;
    let a_part = difference + b_part;
    ((difference), (a - a_part) + (b_part - b))
}

/// `a * b` as a rounded product plus the exact amount that rounding lost.
///
/// The two returned values always multiply out to `a * b` exactly. Uses a
/// fused multiply-add, which computes `a * b - product` with one rounding at
/// the end and so hands back the error directly. The workspace builds for
/// `x86-64-v3` and AArch64, both of which have the instruction.
pub fn two_product(a: f64, b: f64) -> (f64, f64) {
    let product = a * b;
    (product, a.mul_add(b, -product))
}

/// Whether [`two_product`] on `a` and `b` is exact.
///
/// It is exact when the roundoff term is itself a normal number, and it is not
/// when the product is small enough to push that term into the subnormals,
/// because then the term cannot hold all its own bits. Shewchuk's predicates
/// state this as a precondition and leave it to the caller; the `robust` crate
/// takes it for granted and returns a confidently wrong sign on coordinates
/// near the bottom of the `f64` range. A CAD kernel cannot afford that, so
/// check.
///
/// The threshold is `2^-969`, which is the smallest product whose half-ulp —
/// the largest the roundoff term can be — is still normal.
///
/// [`two_sum`] and [`two_diff`] need no such check: they are exact for every
/// pair of `f64` that does not overflow, subnormals included.
///
/// ```
/// # use rcad_predicates::expansion::product_is_exact;
/// assert!(product_is_exact(1.5, 2.5));
/// assert!(product_is_exact(0.0, 3.5e-323));
/// assert!(!product_is_exact(1.5, 3.5e-323));
/// ```
pub fn product_is_exact(a: f64, b: f64) -> bool {
    const SMALLEST_EXACT_PRODUCT: f64 = f64::MIN_POSITIVE * 9007199254740992.0;
    a == 0.0 || b == 0.0 || (a * b).abs() >= SMALLEST_EXACT_PRODUCT
}

#[cfg(test)]
mod tests {
    use super::{expansion_sign, expansion_sum, scale_expansion, two_diff, two_product, two_sum};
    use crate::Decision;

    #[test]
    fn adding_one_to_a_huge_number_keeps_the_one() {
        let huge = 2f64.powi(60);
        let (sum, roundoff) = two_sum(huge, 1.0);
        assert_eq!(sum, huge);
        assert_eq!(roundoff, 1.0);
    }

    #[test]
    fn subtracting_leaves_the_bits_that_fell_off_the_bottom() {
        let huge = 2f64.powi(60);
        let (difference, roundoff) = two_diff(huge, 1.0);
        assert_eq!(difference, huge);
        assert_eq!(roundoff, -1.0);
    }

    #[test]
    fn a_product_of_two_full_mantissas_needs_both_halves() {
        let a = 1.0 + 2f64.powi(-30);
        let b = 1.0 + 2f64.powi(-30);
        let (product, roundoff) = two_product(a, b);
        assert_eq!(product, 1.0 + 2f64.powi(-29));
        assert_eq!(roundoff, 2f64.powi(-60));
    }

    #[test]
    fn a_product_that_fits_has_no_roundoff() {
        assert_eq!(two_product(3.0, 5.0), (15.0, 0.0));
        assert_eq!(two_sum(0.5, 0.25), (0.75, 0.0));
        assert_eq!(two_diff(0.5, 0.25), (0.25, 0.0));
    }

    #[test]
    fn the_roundoff_term_is_what_decides_the_sign() {
        // 2^60 + 1 is bigger than 2^60, and only the roundoff entry says so.
        let huge = 2f64.powi(60);
        let (sum, roundoff) = two_sum(huge, 1.0);
        let mut difference = [0.0; 3];
        let len = expansion_sum(&[roundoff, sum], &[-huge], &mut difference);
        assert_eq!(&difference[..len], &[1.0]);
        assert_eq!(expansion_sign(&difference[..len]), Decision::Pos);
    }

    #[test]
    fn two_expansions_that_cancel_exactly_come_back_as_zero() {
        let value = [2f64.powi(-40), 1.0, 2f64.powi(70)];
        let negated = [-2f64.powi(-40), -1.0, -2f64.powi(70)];
        let mut out = [0.0; 6];
        let len = expansion_sum(&value, &negated, &mut out);
        assert_eq!(&out[..len], &[0.0]);
        assert_eq!(expansion_sign(&out[..len]), Decision::Zero);
    }

    #[test]
    fn a_sum_keeps_every_bit_of_three_far_apart_magnitudes() {
        let mut out = [0.0; 4];
        let len = expansion_sum(&[1.0], &[2f64.powi(-60), 2f64.powi(60)], &mut out);
        assert_eq!(&out[..len], &[2f64.powi(-60), 1.0, 2f64.powi(60)]);
    }

    #[test]
    fn scaling_by_a_power_of_two_is_exact_and_stays_one_entry() {
        let mut out = [0.0; 4];
        let len = scale_expansion(&[1.0, 2f64.powi(60)], 4.0, &mut out);
        assert_eq!(&out[..len], &[4.0, 2f64.powi(62)]);
    }

    #[test]
    fn scaling_keeps_the_bits_a_single_multiply_would_lose() {
        let a = 1.0 + 2f64.powi(-30);
        let mut out = [0.0; 2];
        let len = scale_expansion(&[a], a, &mut out);
        assert_eq!(&out[..len], &[2f64.powi(-60), 1.0 + 2f64.powi(-29)]);
        let total: f64 = out[..len].iter().sum();
        assert_eq!(total, a * a);
    }

    #[test]
    fn scaling_by_zero_gives_zero() {
        let mut out = [0.0; 4];
        let len = scale_expansion(&[1.0, 2f64.powi(60)], 0.0, &mut out);
        assert_eq!(expansion_sign(&out[..len]), Decision::Zero);
    }

    #[test]
    fn an_empty_expansion_is_zero() {
        assert_eq!(expansion_sign(&[]), Decision::Zero);
    }

    #[test]
    fn an_overflowed_expansion_refuses_to_claim_a_sign() {
        let (product, roundoff) = two_product(1e200, 1e200);
        assert!(!product.is_finite());
        assert_eq!(
            expansion_sign(&[roundoff, product]),
            Decision::Undecidable {
                enclosure: f64::INFINITY
            }
        );
    }

    /// The pair is exact only when its answer is finite, and this is what the
    /// other side of that looks like. Pinned so the sentence on [`two_diff`]
    /// cannot drift away from the code again.
    #[test]
    fn a_sum_or_difference_that_overflows_hands_back_a_pair_that_stands_for_nothing() {
        let (difference, roundoff) = two_diff(f64::MAX, -f64::MAX);
        assert_eq!(difference, f64::INFINITY);
        assert!(roundoff.is_nan());

        let (sum, roundoff) = two_sum(f64::MAX, f64::MAX);
        assert_eq!(sum, f64::INFINITY);
        assert!(roundoff.is_nan());

        // A hair below the overflow and both entries are exact again, so it is
        // the overflow doing this and not the size of the inputs.
        let (difference, roundoff) = two_diff(f64::MAX, -f64::MIN_POSITIVE);
        assert_eq!(difference, f64::MAX);
        assert_eq!(roundoff, f64::MIN_POSITIVE);
    }

    /// The demonstration behind [`two_diff`]'s claim that an overflowed pair is
    /// always caught further down.
    ///
    /// Both routines that could shorten an expansion drop an entry only when it
    /// compares equal to zero. Scaling by zero is the strongest test of that: it
    /// flattens every finite entry and still cannot make a non-finite one go
    /// away.
    #[test]
    fn a_non_finite_entry_cannot_be_dropped_on_its_way_to_a_sign() {
        let (difference, roundoff) = two_diff(f64::MAX, -f64::MAX);
        let overflowed = [roundoff, difference];
        let refused = Decision::Undecidable {
            enclosure: f64::INFINITY,
        };

        let mut summed = [0.0; 4];
        let len = expansion_sum(&overflowed, &[1.0, 2.0], &mut summed);
        assert_eq!(expansion_sign(&summed[..len]), refused);

        let mut scaled = [0.0; 4];
        let len = scale_expansion(&overflowed, 0.0, &mut scaled);
        assert_eq!(expansion_sign(&scaled[..len]), refused);
    }

    #[test]
    #[should_panic(expected = "room for every entry")]
    fn a_sum_refuses_to_write_past_the_end() {
        let mut out = [0.0; 1];
        expansion_sum(&[1.0, 2.0], &[3.0], &mut out);
    }

    #[test]
    #[should_panic(expected = "double its length")]
    fn a_scale_refuses_to_write_past_the_end() {
        let mut out = [0.0; 1];
        scale_expansion(&[1.0], 3.0, &mut out);
    }

    /// Adds a list of `f64` exactly, then checks the answer against the same
    /// list added in plain `f64` when the plain sum happens to be exact.
    #[test]
    fn an_exact_sum_of_many_terms_matches_plain_arithmetic_when_plain_is_exact() {
        let terms: [f64; 5] = [0.5, 0.25, 0.125, 0.0625, 0.03125];
        let mut total = [0.0f64; 8];
        let mut total_len = 0;
        let mut merged = [0.0f64; 8];
        for &term in &terms {
            total_len = expansion_sum(&total[..total_len], &[term], &mut merged);
            core::mem::swap(&mut total, &mut merged);
        }
        let exact: f64 = total[..total_len].iter().sum();
        assert_eq!(exact, 0.96875);
    }
}
