//! Exact expansion arithmetic the quadric predicates share.

use rcad_predicates::expansion::{expansion_sum, product_is_exact, scale_expansion, two_diff};

/// `a - b` as a two-entry expansion, smallest first.
///
/// Exact in two entries whenever `a - b` is finite, which is why, unlike the
/// products below, this one has no gate of its own. When the difference
/// overflows the pair is infinite and `NaN` and stands for nothing, and nothing
/// here says so — the non-finite entry travels on until
/// [`square_of_expansion`]'s [`product_is_exact`] check refuses it, or, where a
/// scaling by zero carries it past that check, until `expansion_sign` refuses
/// to name a sign for it.
pub(crate) fn difference(a: f64, b: f64) -> [f64; 2] {
    let (high, low) = two_diff(a, b);
    [low, high]
}

/// The longest expansion [`square_of_expansion`] will square.
///
/// A cylinder's cross-product component is the longest thing squared here:
/// two two-entry differences scaled by an `f64` each give four entries, and
/// their difference gives eight.
pub(crate) const LONGEST_SQUARED_INPUT: usize = 8;

/// How many entries the square of an expansion that long can need.
///
/// Squaring is one [`scale_expansion`] per entry, each at most twice the input
/// length, all summed together.
pub(crate) const LONGEST_SQUARE: usize = 2 * LONGEST_SQUARED_INPUT * LONGEST_SQUARED_INPUT;

/// The exact square of an expansion, written into `out`.
///
/// Returns how many entries of `out` were used, or `None` when one of the
/// products it needs would round — which is what
/// [`product_is_exact`] is for. `None` means level 3 of the cascade cannot run,
/// not that the answer is zero.
///
/// # Panics
///
/// If `value` is longer than [`LONGEST_SQUARED_INPUT`], or if `out` is shorter
/// than `2 * value.len() * value.len()`.
pub(crate) fn square_of_expansion(value: &[f64], out: &mut [f64]) -> Option<usize> {
    assert!(
        value.len() <= LONGEST_SQUARED_INPUT,
        "squaring an expansion of {} entries needs a bigger scratch buffer",
        value.len()
    );
    assert!(
        out.len() >= 2 * value.len() * value.len(),
        "squaring an expansion can square its length and double it"
    );

    let mut running = [0.0f64; LONGEST_SQUARE];
    let mut running_len = 0;
    let mut merged = [0.0f64; LONGEST_SQUARE];
    let mut scaled = [0.0f64; 2 * LONGEST_SQUARED_INPUT];

    for &factor in value {
        if !value.iter().all(|&entry| product_is_exact(entry, factor)) {
            return None;
        }
        let scaled_len = scale_expansion(value, factor, &mut scaled);
        running_len = expansion_sum(&running[..running_len], &scaled[..scaled_len], &mut merged);
        core::mem::swap(&mut running, &mut merged);
    }

    out[..running_len].copy_from_slice(&running[..running_len]);
    Some(running_len)
}

#[cfg(test)]
mod tests {
    use rcad_predicates::Decision;
    use rcad_predicates::expansion::{expansion_sign, two_sum};

    use super::{LONGEST_SQUARE, difference, square_of_expansion};

    #[test]
    fn squaring_a_single_number_gives_its_square() {
        let mut squared = [0.0; LONGEST_SQUARE];
        let len = square_of_expansion(&[3.0], &mut squared).unwrap();
        assert_eq!(&squared[..len], &[9.0]);
    }

    #[test]
    fn squaring_is_never_negative_however_the_expansion_is_signed() {
        for value in [-7.5, 7.5] {
            let mut squared = [0.0; LONGEST_SQUARE];
            let len = square_of_expansion(&[value], &mut squared).unwrap();
            assert_eq!(expansion_sign(&squared[..len]), Decision::Pos);
        }
    }

    /// `(2^60 + 1)^2` is `2^120 + 2^61 + 1`. That trailing one is 120 bits
    /// below the leading one, so no single `f64` and no pair of them holds the
    /// whole answer — an expansion does.
    #[test]
    fn squaring_a_two_entry_expansion_keeps_the_bits_an_f64_would_drop() {
        let big = 2f64.powi(60);
        assert_eq!(big + 1.0, big, "even the input does not fit in one f64");
        let (high, low) = two_sum(big, 1.0);

        let mut squared = [0.0; LONGEST_SQUARE];
        let len = square_of_expansion(&[low, high], &mut squared).unwrap();
        let expansion = &squared[..len];

        assert!(expansion.contains(&1.0), "got {expansion:?}");
        assert_eq!(expansion.last(), Some(&2f64.powi(120)));
        assert_eq!(expansion_sign(expansion), Decision::Pos);
    }

    #[test]
    fn a_product_too_small_to_be_exact_stops_the_exact_level() {
        let mut squared = [0.0; LONGEST_SQUARE];
        assert_eq!(square_of_expansion(&[2f64.powi(-600)], &mut squared), None);
    }

    /// [`difference`] has no gate of its own, and this is the demonstration
    /// that it does not need one: an overflowed pair stops the exact level
    /// instead of reaching a sign.
    #[test]
    fn a_difference_that_overflows_stops_the_exact_level() {
        let gap = difference(f64::MAX, -f64::MAX);
        assert!(gap[1].is_infinite());
        assert!(gap[0].is_nan());

        let mut squared = [0.0; LONGEST_SQUARE];
        assert_eq!(square_of_expansion(&gap, &mut squared), None);
    }

    #[test]
    #[should_panic(expected = "bigger scratch buffer")]
    fn squaring_an_expansion_longer_than_the_scratch_buffer_is_a_bug() {
        let mut squared = [0.0; LONGEST_SQUARE];
        let _ = square_of_expansion(&[1.0; 9], &mut squared);
    }
}
