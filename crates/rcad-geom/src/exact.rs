//! Exact expansion arithmetic the quadric predicates share.

use rcad_predicates::Decision;
use rcad_predicates::expansion::{
    expansion_sign, expansion_sum, product_is_exact, scale_expansion, two_diff, two_product,
};

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

/// One exact number, held as an expansion that grows as it needs to.
///
/// [`square_of_expansion`] above writes into a fixed slice, because the two
/// predicates that call it know their own sizes. An intersector's discriminants
/// do not: `n·n · r² − w·w · (n·o − d)²` multiplies a six-entry expansion by a
/// ninety-eight-entry one, and the worst case a fixed buffer would have to hold
/// is over a thousand `f64` for a level that runs on a small minority of calls
/// (`docs/decisions.md` §27.1). §27.1 weighs the same three fixes for the
/// quartics one degree up and picks the heap scratch, for the same reason: it
/// costs an allocation on a rare path and refuses nothing.
///
/// Every operation here is exact or refuses. There is no rounding anywhere in
/// this type, which is what makes [`Exact::sign`] a proof rather than a guess.
#[derive(Clone, Debug)]
pub(crate) struct Exact(Vec<f64>);

impl Exact {
    /// The number `value`, exactly.
    pub(crate) fn of(value: f64) -> Self {
        Self(if value == 0.0 {
            Vec::new()
        } else {
            vec![value]
        })
    }

    /// `a * b`, exactly, or `None` when the product's low half would be a
    /// subnormal and the pair would no longer add up to the true product.
    pub(crate) fn product_of(a: f64, b: f64) -> Option<Self> {
        if !product_is_exact(a, b) {
            return None;
        }
        let (high, low) = two_product(a, b);
        Some(Self(vec![low, high]))
    }

    /// `a · b`, exactly, over two triples of stored numbers.
    pub(crate) fn dot_of(a: [f64; 3], b: [f64; 3]) -> Option<Self> {
        let mut total = Self::of(0.0);
        for axis in 0..3 {
            total = total.plus(&Self::product_of(a[axis], b[axis])?);
        }
        Some(total)
    }

    /// This plus `other`, exactly.
    pub(crate) fn plus(&self, other: &Exact) -> Exact {
        let mut out = vec![0.0; self.0.len() + other.0.len()];
        let len = expansion_sum(&self.0, &other.0, &mut out);
        out.truncate(len);
        Exact(out)
    }

    /// This minus `other`, exactly.
    pub(crate) fn minus(&self, other: &Exact) -> Exact {
        self.plus(&Exact(other.0.iter().map(|entry| -entry).collect()))
    }

    /// This times `other`, exactly, or `None` when one of the products it needs
    /// would round.
    pub(crate) fn times(&self, other: &Exact) -> Option<Exact> {
        let mut running = Exact::of(0.0);
        let mut scaled = vec![0.0; 2 * self.0.len()];
        for &factor in &other.0 {
            if !self.0.iter().all(|&entry| product_is_exact(entry, factor)) {
                return None;
            }
            let len = scale_expansion(&self.0, factor, &mut scaled);
            running = running.plus(&Exact(scaled[..len].to_vec()));
        }
        Some(running)
    }

    /// The sign of this number.
    ///
    /// [`Decision::Undecidable`] only when an entry is not a real number, which
    /// takes an overflow somewhere upstream — every operation above is exact, so
    /// there is nothing else here for a sign to be uncertain about.
    pub(crate) fn sign(&self) -> Decision {
        expansion_sign(&self.0)
    }

    /// The nearest `f64` to this number, or as near as adding its entries up
    /// gets.
    ///
    /// An expansion is stored smallest first and its entries do not overlap, so
    /// adding them in that order is right to within a rounding or two of the
    /// total. That is far tighter than the same expression evaluated in `f64`
    /// from the start, which is what lets [`crate::walk`] publish a bound that
    /// holds for a nearly-tangent crossing as well as for an ordinary one.
    pub(crate) fn nearest_f64(&self) -> f64 {
        self.0.iter().sum()
    }
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

#[cfg(test)]
mod an_exact_number {
    use rcad_predicates::Decision;

    use super::Exact;

    /// Two whole-number vectors whose dot product is a hair away from zero, and
    /// that hair is a hundred bits below the terms that make it. `f64` sees
    /// nothing but cancellation; an expansion sees the sign.
    #[test]
    fn a_dot_product_that_cancels_to_nothing_in_f64_still_has_a_sign() {
        let big = 2f64.powi(60);
        let one = [big, 1.0, 0.0];
        let other = [1.0, -big, 1.0];

        let in_f64 = one[0] * other[0] + one[1] * other[1] + one[2] * other[2];
        assert_eq!(in_f64, 0.0, "every term cancels once it has been rounded");

        assert_eq!(Exact::dot_of(one, other).unwrap().sign(), Decision::Zero);
        assert_eq!(
            Exact::dot_of(one, [1.0, -big, 1.0])
                .unwrap()
                .plus(&Exact::of(1.0))
                .sign(),
            Decision::Pos
        );
    }

    /// A sum of squares is never below zero, however the numbers are signed,
    /// and that is what lets "are these parallel" be asked as one sign question.
    #[test]
    fn a_sum_of_squares_is_never_below_zero() {
        for value in [-7.5, 0.0, 7.5] {
            let squared = Exact::of(value).times(&Exact::of(value)).unwrap();
            let expected = if value == 0.0 {
                Decision::Zero
            } else {
                Decision::Pos
            };
            assert_eq!(squared.sign(), expected, "{value}");
        }
    }

    /// A product whose low half would land among the subnormals is refused
    /// rather than rounded, because a pair that does not add back up to the true
    /// product turns exact arithmetic into confident arithmetic.
    #[test]
    fn a_product_too_small_to_be_exact_refuses_rather_than_rounds() {
        let tiny = 2f64.powi(-600);
        assert!(Exact::product_of(tiny, tiny).is_none());
        assert!(Exact::of(tiny).times(&Exact::of(tiny)).is_none());
        assert!(Exact::dot_of([tiny, 0.0, 0.0], [tiny, 0.0, 0.0]).is_none());
        assert!(Exact::product_of(tiny, 1.0).is_some());
    }

    /// An overflow travels as a number that is not a number, and the sign
    /// refuses it rather than reading a confident answer off an infinity.
    #[test]
    fn a_value_that_overflowed_on_the_way_in_has_no_sign() {
        let overflowed = Exact::product_of(1e300, 1e300);
        match overflowed.map(|value| value.sign()) {
            Some(Decision::Undecidable { enclosure }) => assert!(enclosure.is_infinite()),
            None => (),
            other => panic!("an overflowed product has no sign: {other:?}"),
        }
    }
}
