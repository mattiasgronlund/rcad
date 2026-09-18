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

/// One sweep down the expansion and back up, packing the value into fewer
/// entries, written into `out`.
///
/// Returns how many entries of `out` were used, never more than the input had.
///
/// # Why an exact answer still needs this
///
/// [`expansion_sum`] and [`scale_expansion`] give back an expansion whose
/// entries do not overlap, which is all a *sign* needs. It is not enough to make
/// the expansion a function of the value it stands for: the same number reached
/// by two routes comes back in two different lists of entries, both correct and
/// one of them longer and reaching further down the range than the other.
///
/// That matters because [`product_is_exact`] is a question about *entries*.
/// A multiplication refuses as soon as one entry-by-entry product would land in
/// the subnormals, so a needlessly long expansion refuses multiplications that
/// the same value written shortly would have taken — and which route produced it
/// then decides whether an answer that exists can be reached. Asking one pair of
/// cylinders which shape they cross in, both ways round, found exactly that:
/// `W₁W₂` came back eight entries and 239 binary orders wide one way and nine
/// entries and 284 wide the other, and only the shorter one could be multiplied
/// by what came next (`docs/decisions.md` §88).
///
/// # One sweep is not always a fixed point
///
/// Shewchuk's `COMPRESS`, with `FAST-TWO-SUM` replaced by the unconditional
/// [`two_sum`] as everywhere else here. **It does not always settle in one go**:
/// the leaning-pipes question above comes out of a sweep ten entries reaching
/// down to `2^-483`, and sweeping *that* gives ten entries reaching to `2^-504`,
/// which is then stable. So this is a step and not a normal form, and a caller
/// that needs the entries to depend on the value alone has to run it to a fixed
/// point — which is what `Exact::canonical` does and what
/// `sweeping_twice_can_differ_from_sweeping_once` pins here.
///
/// # Panics
///
/// If `out` is shorter than `expansion`.
pub fn compress(expansion: &[f64], out: &mut [f64]) -> usize {
    assert!(
        out.len() >= expansion.len(),
        "compressing an expansion cannot lengthen it"
    );
    let Some((&largest, rest)) = expansion.split_last() else {
        return 0;
    };

    // Down from the largest entry, pushing what each addition rounds away into
    // the entry below it. This leaves the entries stacked against the top of the
    // range, written from `bottom` upwards.
    let mut bottom = expansion.len();
    let mut running = largest;
    for &entry in rest.iter().rev() {
        let (sum, roundoff) = two_sum(running, entry);
        if roundoff == 0.0 {
            running = sum;
            continue;
        }
        bottom -= 1;
        out[bottom] = sum;
        running = roundoff;
    }
    bottom -= 1;
    out[bottom] = running;

    // And back up the same way, which is the pass that makes each entry the
    // rounded total of itself and everything under it. `running` is still the
    // entry just written, so the sweep carries straight on from it. Reads run
    // ahead of writes — `used` is always below `index` — so one slice serves for
    // both.
    let mut used = 0;
    for index in bottom + 1..expansion.len() {
        let (sum, roundoff) = two_sum(out[index], running);
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

/// `a * b - c * d`, exactly. `None` when either product would round, which is
/// where exact arithmetic stops being exact.
///
/// This is the two by two determinant every larger one is built out of, and
/// both `orient3d` and the three-plane meet expand along a column into three of
/// them. One copy of it here rather than one in each file: the two products and
/// their two roundoff terms are four entries that have to be summed in the
/// right order, and a second copy is a second chance to get that wrong.
///
/// ```
/// # use rcad_predicates::expansion::{difference_of_two_products, expansion_sign};
/// # use rcad_predicates::Decision;
/// // 2^53 + 1 is not an f64, but the difference of these two products is,
/// // and plain f64 loses it.
/// let big = 2f64.powi(53);
/// let (entries, len) = difference_of_two_products(big + 2.0, 1.0, big, 1.0).unwrap();
/// assert_eq!(&entries[..len], &[2.0]);
/// assert_eq!(expansion_sign(&entries[..len]), Decision::Pos);
/// ```
pub fn difference_of_two_products(a: f64, b: f64, c: f64, d: f64) -> Option<([f64; 4], usize)> {
    if !product_is_exact(a, b) || !product_is_exact(c, d) {
        return None;
    }
    let (forwards, forwards_roundoff) = two_product(a, b);
    let (backwards, backwards_roundoff) = two_product(c, d);
    let mut out = [0.0; 4];
    let len = expansion_sum(
        &[forwards_roundoff, forwards],
        &[-backwards_roundoff, -backwards],
        &mut out,
    );
    Some((out, len))
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
/// A product can miss at both ends of the range, and this rejects both.
///
/// **At the bottom**, it is exact when the roundoff term is itself a normal
/// number, and it is not when the product is small enough to push that term
/// into the subnormals, because then the term cannot hold all its own bits.
/// Shewchuk's predicates state this as a precondition and leave it to the
/// caller; the `robust` crate takes it for granted and returns a confidently
/// wrong sign on coordinates near the bottom of the `f64` range. A CAD kernel
/// cannot afford that, so check. The threshold is `2^-969`, the smallest
/// product whose half-ulp — the largest the roundoff term can be — is still
/// normal.
///
/// **At the top**, an overflowed product is not exact either, and this used to
/// answer `true` for `1e200` squared. The doc was accurate about what the
/// function tested and the name was not: at twenty-seven call sites
/// `product_is_exact` reads as the gate that keeps unrepresentable products out
/// of the expansion arithmetic, and it only ever guarded one end. Nothing was
/// wrong downstream — an infinity reaches [`expansion_sign`], which refuses to
/// name a sign for it, and that is still the backstop. But a gate that lets
/// through the case its name promises to catch is one nobody can read.
///
/// The zero test is on the arguments and not on the product, and has to be:
/// `1e-200` squared underflows to zero without either argument being zero, and
/// that is the bottom-end case above rather than an exact one.
///
/// [`two_sum`] and [`two_diff`] need no such check: they are exact for every
/// pair of `f64` that does not overflow, subnormals included.
///
/// ```
/// # use rcad_predicates::expansion::product_is_exact;
/// assert!(product_is_exact(1.5, 2.5));
/// assert!(product_is_exact(0.0, 3.5e-323));
/// assert!(!product_is_exact(1.5, 3.5e-323));
/// assert!(!product_is_exact(1e200, 1e200));
/// assert!(!product_is_exact(1e-200, 1e-200));
/// ```
pub fn product_is_exact(a: f64, b: f64) -> bool {
    const SMALLEST_EXACT_PRODUCT: f64 = f64::MIN_POSITIVE * 9007199254740992.0;
    let product = a * b;
    if !product.is_finite() {
        return false;
    }
    a == 0.0 || b == 0.0 || product.abs() >= SMALLEST_EXACT_PRODUCT
}

#[cfg(test)]
mod tests {
    use super::{
        compress, expansion_sign, expansion_sum, product_is_exact, scale_expansion, two_diff,
        two_product, two_sum,
    };
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
    /// The overflow end of the gate, which used to be open.
    ///
    /// `1e200` squared is `1e400`, and `product_is_exact` answered `true` for
    /// it. Nothing downstream was wrong — the next test says why — but the name
    /// promised a check that half existed.
    #[test]
    fn a_product_that_overflows_is_not_exact() {
        assert!(!product_is_exact(1e200, 1e200));
        assert!(!product_is_exact(f64::MAX, 2.0));
        assert!(!product_is_exact(0.0, f64::INFINITY));
        assert!(!product_is_exact(f64::NAN, 1.0));
    }

    /// The zero test has to be on the arguments, not on the product.
    ///
    /// `1e-200` squared underflows to zero with neither argument zero. Reading
    /// the zero off the product instead would call that exact, which is the
    /// subnormal end of the gate failing open — the end that was already
    /// closed, and the `robust` crate's wrong-sign bug.
    #[test]
    fn a_product_that_underflows_to_zero_is_not_exact_though_the_product_is_zero() {
        assert_eq!(1e-200 * 1e-200, 0.0);
        assert!(!product_is_exact(1e-200, 1e-200));
        assert!(product_is_exact(0.0, 1e-200));
    }

    /// Why closing the gate changes no answer: the backstop was already there.
    ///
    /// The pair `product_is_exact` now rejects is exactly the pair that used to
    /// go through and be refused one level down.
    /// `a_non_finite_entry_cannot_be_dropped_on_its_way_to_a_sign` is the
    /// general invariant; this is the overflow case of it, reached through
    /// [`two_product`] rather than [`two_diff`], and it is what makes this
    /// change legibility rather than a fix.
    #[test]
    fn an_overflowed_product_was_already_refused_by_the_sign() {
        assert!(!product_is_exact(1e200, 1e200));

        let (product, roundoff) = two_product(1e200, 1e200);
        let mut scaled = [0.0f64; 4];
        let len = scale_expansion(&[roundoff, product], 1.0, &mut scaled);
        assert_eq!(
            expansion_sign(&scaled[..len]),
            Decision::Undecidable {
                enclosure: f64::INFINITY
            }
        );
    }

    /// An expansion of `terms`, built by adding them up one at a time.
    fn an_expansion_of(terms: &[f64]) -> Vec<f64> {
        let mut total: Vec<f64> = Vec::new();
        for &term in terms {
            let mut merged = vec![0.0; total.len() + 1];
            let len = expansion_sum(&total, &[term], &mut merged);
            merged.truncate(len);
            total = merged;
        }
        total
    }

    /// Whether `left` and `right` are the same number, through
    /// [`expansion_sum`] alone — which knows nothing about [`compress`] and so
    /// can be used to judge it.
    fn the_same_number(left: &[f64], right: &[f64]) -> bool {
        let negated: Vec<f64> = right.iter().map(|entry| -entry).collect();
        let mut difference = vec![0.0; left.len() + negated.len()];
        let len = expansion_sum(left, &negated, &mut difference);
        expansion_sign(&difference[..len]) == Decision::Zero
    }

    /// The one thing compressing must never do.
    ///
    /// Judged by [`expansion_sum`], which is the arithmetic
    /// [`compress`] is not built on, so a bug in one cannot hide in the other.
    #[test]
    fn compressing_does_not_change_the_number() {
        for terms in [
            vec![2f64.powi(-60), 1.0, 2f64.powi(60)],
            vec![1.0, 2f64.powi(-53), 2f64.powi(-106)],
            vec![0.1, 0.2, 0.3, 0.4, 0.5],
            vec![1e-300, 1e300, -1e300, 7.5],
            vec![1.0, -1.0, 2f64.powi(-70)],
            vec![3.0],
            vec![],
        ] {
            let expansion = an_expansion_of(&terms);
            let mut out = vec![0.0; expansion.len()];
            let len = compress(&expansion, &mut out);
            assert!(
                the_same_number(&expansion, &out[..len]),
                "compressing {expansion:?} gave {:?}, which is a different number",
                &out[..len]
            );
            assert!(len <= expansion.len().max(1));
        }
    }

    /// The property the leaning-pipes fix rests on: the entries depend on the
    /// number and not on the route that reached it.
    ///
    /// `W₁W₂` is the quantity that broke — reached one way it came back eight
    /// entries wide and the other way nine, and only the shorter could be
    /// multiplied by what came next. Here are the same two routes in miniature.
    /// `left × right`, accumulated one entry of `right` at a time — the way
    /// `Exact::times` does it, and so the place the operand order gets in.
    fn the_product_of(left: &[f64], right: &[f64]) -> Vec<f64> {
        let mut total: Vec<f64> = Vec::new();
        for &factor in right {
            let mut scaled = vec![0.0; 2 * left.len()];
            let len = scale_expansion(left, factor, &mut scaled);
            let mut merged = vec![0.0; total.len() + len];
            let merged_len = expansion_sum(&total, &scaled[..len], &mut merged);
            merged.truncate(merged_len);
            total = merged;
        }
        total
    }

    /// On an ordinary expansion one sweep settles it.
    #[test]
    fn compressing_a_compressed_expansion_usually_changes_nothing() {
        let expansion = an_expansion_of(&[0.1, 0.2, 0.3, 1e-200, 1e200, 2f64.powi(-1000)]);
        let mut once = vec![0.0; expansion.len()];
        let first = compress(&expansion, &mut once);
        let mut twice = vec![0.0; first];
        let second = compress(&once[..first], &mut twice);
        assert_eq!(&once[..first], &twice[..second]);
    }

    /// But not always, which is why `Exact::canonical` sweeps to a fixed point
    /// rather than sweeping once.
    ///
    /// This was found the expensive way. One sweep was taken for a normal form,
    /// and the entries it gave for `M` — the leaning-pipes question — were not
    /// the entries a second sweep gave. Ten entries either way, but reaching to
    /// `2^-483` after one and `2^-504` after two, and only the second is stable.
    /// A claim that a value has one written form is worth nothing if the
    /// function making it is not run to where it stops moving.
    /// These six numbers are the smallest instance a search turned up: one sweep
    /// drops four entries to three and leaves the smallest of them wrong against
    /// the settled answer, the second gets it, and the third changes nothing.
    #[test]
    fn sweeping_twice_can_differ_from_sweeping_once() {
        let expansion = an_expansion_of(&[
            0.000_485_599_498_242_050_23,
            27_086.019_292_511_162,
            -1.783_184_736_861_294_4e-13,
            6.850_717_877_827_191,
            -1.222_739_631_755_684e-12,
            53_911.450_270_901_83,
        ]);
        let once = swept(&expansion);
        let twice = swept(&once);

        assert_ne!(
            once, twice,
            "this test is only worth running while one sweep really does leave work behind"
        );
        assert_eq!(twice, swept(&twice), "and the second sweep settles it");
        assert!(
            the_same_number(&expansion, &once) && the_same_number(&expansion, &twice),
            "every sweep is exact, settled or not"
        );
    }

    /// One sweep of `entries`, as a fresh list.
    fn swept(entries: &[f64]) -> Vec<f64> {
        let mut out = vec![0.0; entries.len()];
        let len = compress(entries, &mut out);
        out.truncate(len);
        out
    }

    /// The property the whole of §88 rests on: once the sweeping has settled,
    /// the entries are a function of the value and of nothing else.
    ///
    /// Two expansions are built for the same number by routes that give
    /// different entries — `a×b` against `b×a`, which is exactly the pair that
    /// made one crossing name two different shapes — and they have to settle on
    /// the same list. Searched rather than argued, because sweeping to a fixed
    /// point is a normal form here by demonstration and not by a proof anyone
    /// has written down.
    #[test]
    fn two_routes_to_one_number_end_at_the_same_entries() {
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut a_number = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let exponent = ((state >> 52) % 120) as i32 - 60;
            let mantissa = 1.0 + (state & 0xf_ffff_ffff_ffff) as f64 / 4.503_6e15;
            mantissa * 2f64.powi(exponent) * if state & 1 == 0 { 1.0 } else { -1.0 }
        };

        let mut differed_before_settling = 0;
        for _ in 0..20_000 {
            let left = an_expansion_of(&[a_number(), a_number(), a_number()]);
            let right = an_expansion_of(&[a_number(), a_number(), a_number()]);
            let one_way = the_product_of(&left, &right);
            let the_other_way = the_product_of(&right, &left);

            assert!(
                the_same_number(&one_way, &the_other_way),
                "multiplying {left:?} by {right:?} gave two different numbers"
            );
            if one_way != the_other_way {
                differed_before_settling += 1;
            }
            assert_eq!(
                settled(&one_way),
                settled(&the_other_way),
                "the same number reached two ways has to settle on the same entries: \
                 {left:?} times {right:?}"
            );
        }
        assert!(
            differed_before_settling > 0,
            "this search proves nothing unless the two orders really do differ before settling"
        );
    }

    /// Sweeping `entries` until it stops changing, as `Exact::canonical` does.
    fn settled(entries: &[f64]) -> Vec<f64> {
        let mut at = entries.to_vec();
        for _ in 0..=entries.len() {
            let next = swept(&at);
            if next == at {
                return at;
            }
            at = next;
        }
        panic!("sweeping {entries:?} never settled");
    }

    /// The ordering the rest of the module relies on survives compressing:
    /// entries increase in magnitude, so the last one settles the sign.
    #[test]
    fn a_compressed_expansion_is_still_sorted_smallest_first() {
        let expansion = an_expansion_of(&[2f64.powi(-90), 0.1, 3.7, 2f64.powi(80), 0.25]);
        let mut out = vec![0.0; expansion.len()];
        let len = compress(&expansion, &mut out);
        for pair in out[..len].windows(2) {
            assert!(
                pair[0].abs() < pair[1].abs(),
                "compressed entries run smallest first: {:?}",
                &out[..len]
            );
        }
        assert_eq!(expansion_sign(&out[..len]), Decision::Pos);
    }

    #[test]
    #[should_panic(expected = "cannot lengthen")]
    fn compressing_refuses_to_write_past_the_end() {
        let mut out = [0.0; 1];
        compress(&[2f64.powi(-60), 1.0, 2f64.powi(60)], &mut out);
    }
}
