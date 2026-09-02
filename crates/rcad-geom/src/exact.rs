//! Exact expansion arithmetic the quadric predicates share.

use rcad_predicates::expansion::{
    expansion_sign, expansion_sum, product_is_exact, scale_expansion, two_diff, two_product,
};
use rcad_predicates::{Decision, Interval};

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
    ///
    /// A low half of zero is dropped rather than stored. Every other operation
    /// here eliminates zero entries, and
    /// [`Exact::exponent_that_makes_its_square_exact`] reads the smallest entry
    /// to work out how far to slide — so a stored zero made the slide read
    /// "already at the bottom of the range" and refuse a square that a slide
    /// reaches.
    pub(crate) fn product_of(a: f64, b: f64) -> Option<Self> {
        if !product_is_exact(a, b) {
            return None;
        }
        let (high, low) = two_product(a, b);
        if low == 0.0 {
            return Some(Self(vec![high]));
        }
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

    /// `self² − already_squared`, multiplied by whatever positive power of two
    /// the squaring needed — so **a sign and never a value**.
    ///
    /// [`Exact::times`] refuses as soon as one product's low half would land
    /// among the subnormals, and squaring doubles the spread between an
    /// expansion's largest entry and its smallest. A number whose own spread
    /// fits inside the window `WIDEST_SQUARABLE_SPREAD` names but which sits too low or too
    /// high in the range for its own square to fit is refused where it stands
    /// and squares without complaint somewhere else, because **multiplying an
    /// expansion by a power of two is exact and moves every entry together**
    /// (`docs/decisions.md` §31.4).
    ///
    /// So this slides `self` into the window before squaring it, slides
    /// `already_squared` by twice as much so the two stay comparable, and slides
    /// nothing back. Both were multiplied by the same positive number, so the
    /// difference has the sign the caller asked for and a size the caller must
    /// not read.
    ///
    /// The slide is zero wherever the square already fits, so a comparison that
    /// decides today decides through exactly the same arithmetic.
    ///
    /// `None` when `self` is wider than the window at any scale, or when
    /// `already_squared` cannot follow it.
    pub(crate) fn square_minus(&self, already_squared: &Exact) -> Option<Exact> {
        let up = self.exponent_that_makes_its_square_exact()?;
        let slid = self.slid_by(up)?;
        Some(slid.times(&slid)?.minus(&already_squared.slid_by(2 * up)?))
    }

    /// The exponent that slides this number into the window where squaring it is
    /// exact, or `None` when its own spread is wider than the window.
    ///
    /// Zero whenever the square already fits. Otherwise the smallest move that
    /// does, so a slide never travels further than it has to.
    fn exponent_that_makes_its_square_exact(&self) -> Option<i32> {
        // A zero entry is worth nothing to the value and squares exactly
        // wherever it is put, so the entry that decides how far down the range
        // this expansion reaches is the smallest one that is not zero.
        let (Some(&smallest), Some(&largest)) =
            (self.0.iter().find(|entry| **entry != 0.0), self.0.last())
        else {
            return Some(0);
        };
        let lift_the_bottom = SMALLEST_ENTRY_A_SQUARE_CAN_HOLD - binary_exponent(smallest);
        let hold_the_top = LARGEST_ENTRY_A_SQUARE_CAN_HOLD - binary_exponent(largest);
        if lift_the_bottom > hold_the_top {
            return None;
        }
        Some(0.clamp(lift_the_bottom, hold_the_top))
    }

    /// This number multiplied by `2^exponent`, exactly, or `None` when that
    /// would push an entry out of the range where the multiplication is exact.
    ///
    /// A power of two only changes an entry's exponent field, so nothing rounds
    /// while the result is a normal number. Past the top it is infinity and
    /// below the bottom it is rounded onto the subnormal grid, and either way the
    /// slid copy stands for a different number than the one that was slid.
    fn slid_by(&self, exponent: i32) -> Option<Exact> {
        if exponent == 0 {
            return Some(self.clone());
        }
        let mut slid = self.0.clone();
        for entry in &mut slid {
            if *entry == 0.0 {
                continue;
            }
            let moved = multiplied_by_a_power_of_two(*entry, exponent);
            if !moved.is_normal() {
                return None;
            }
            *entry = moved;
        }
        Some(Exact(slid))
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

    /// `a × b`, exactly, where `a` is already exact and `b` is stored numbers.
    pub(crate) fn cross_of(a: &[Exact; 3], b: [f64; 3]) -> Option<[Exact; 3]> {
        let component = |axis: usize| {
            let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
            let left = a[next].times(&Exact::of(b[after]))?;
            let right = a[after].times(&Exact::of(b[next]))?;
            Some(left.minus(&right))
        };
        Some([component(0)?, component(1)?, component(2)?])
    }

    /// `u · u` for an already-exact triple, exactly.
    pub(crate) fn squared_length_of(value: &[Exact; 3]) -> Option<Exact> {
        Self::dot_of_two_exact(value, value)
    }

    /// `u · v` for two already-exact triples, exactly.
    pub(crate) fn dot_of_two_exact(u: &[Exact; 3], v: &[Exact; 3]) -> Option<Exact> {
        let mut total = Self::of(0.0);
        for axis in 0..3 {
            total = total.plus(&u[axis].times(&v[axis])?);
        }
        Some(total)
    }

    /// Three stored numbers, each exactly.
    pub(crate) fn of_each(value: [f64; 3]) -> [Exact; 3] {
        value.map(Self::of)
    }

    /// `there − here`, componentwise and exactly.
    pub(crate) fn gap_between(here: [f64; 3], there: [f64; 3]) -> [Exact; 3] {
        core::array::from_fn(|axis| Self::of(there[axis]).minus(&Self::of(here[axis])))
    }

    /// This number as an enclosure, which is as tight as an enclosure of it
    /// gets.
    ///
    /// An expansion's entries do not overlap and each of them is an exact `f64`,
    /// so adding them up in interval arithmetic gives a box whose width is a
    /// handful of roundings of the total — where the same expression built out
    /// of intervals from the start carries the width of every cancellation on
    /// the way.
    ///
    /// **That difference is what makes a nearly-parallel pair of axes decidable
    /// at all.** `‖w₁ × w₂‖²` for two directions a rounding apart is a sum of
    /// three squares of numbers that cancelled to nothing, and an interval
    /// enclosure of it reaches down to zero — so the distance between the two
    /// axes, which divides by its root, comes back as everything. Summed through
    /// the expansion instead, the same quantity is known to full relative
    /// accuracy and the division is ordinary (`docs/decisions.md` §29).
    pub(crate) fn enclosure(&self) -> Interval {
        self.0.iter().fold(Interval::exactly(0.0), |so_far, entry| {
            so_far + Interval::exactly(*entry)
        })
    }

    /// The sign of this number.
    ///
    /// [`Decision::Undecidable`] only when an entry is not a real number, which
    /// takes an overflow somewhere upstream — every operation above is exact, so
    /// there is nothing else here for a sign to be uncertain about.
    pub(crate) fn sign(&self) -> Decision {
        expansion_sign(&self.0)
    }

    /// How many `f64` this number is held in.
    ///
    /// The measurement §29 records. An expansion that has to be squared is only
    /// as squarable as its own dynamic range allows, and this is how a test
    /// shows where that range runs out.
    #[cfg(test)]
    pub(crate) fn entries(&self) -> usize {
        self.0.len()
    }

    /// How many binary orders of magnitude separate the largest entry from the
    /// smallest, or zero for a number held in one entry or none.
    ///
    /// **This is what decides whether an expansion can be squared**, and it is
    /// invariant under scaling every input by a power of two: multiplying the
    /// number scales every entry together. Squaring doubles the spread, and
    /// [`Exact::times`] refuses as soon as one product's low half would land
    /// among the subnormals — so a spread past about half of `f64`'s range
    /// cannot be squared at *any* scale (`docs/decisions.md` §29).
    #[cfg(test)]
    pub(crate) fn spread(&self) -> i32 {
        let (Some(first), Some(last)) = (self.0.first(), self.0.last()) else {
            return 0;
        };
        last.abs().log2().round() as i32 - first.abs().log2().round() as i32
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

/// The smallest binary order an entry may sit at and still be squarable.
///
/// [`product_is_exact`] wants a product of at least `2^-969`, which is the
/// smallest one whose half-ulp is still a normal number. An entry at `2^-484`
/// squares to `2^-968`, and one binade lower does not.
const SMALLEST_ENTRY_A_SQUARE_CAN_HOLD: i32 = -484;

/// The largest binary order an entry may sit at and still be squarable.
///
/// An entry below `2^512` squares to something below `2^1024`, which is the
/// first size `f64` cannot hold at all.
const LARGEST_ENTRY_A_SQUARE_CAN_HOLD: i32 = 511;

/// How far an expansion's largest entry may stand above its smallest and still
/// have a square somewhere in `f64`.
///
/// **This is the published window, and `an_expansion_wider_than_the_window_cannot_be_squared_at_any_scale`
/// is the test §15.6 asks for.** The two ends above leave room for magnitudes
/// from `2^-484` up to `2^512` — 996 binary orders — and an expansion whose own
/// ends are 995 binades apart is the widest that fits between them. A slide
/// moves every entry together and so cannot narrow a spread by even one order,
/// which is why past this there is no scale that helps and the refusal stops
/// being a gap (`docs/decisions.md` §29.6, §30.4).
#[cfg(test)]
pub(crate) const WIDEST_SQUARABLE_SPREAD: i32 =
    LARGEST_ENTRY_A_SQUARE_CAN_HOLD - SMALLEST_ENTRY_A_SQUARE_CAN_HOLD;

/// The `e` with `2^e <= |value| < 2^(e+1)`, read straight out of the bits.
///
/// Read rather than computed, because `log2` rounds and this has to be the
/// exact binade: one out and a slide that would have worked is refused, or one
/// that overflows is tried. `rcad-predicates` keeps the same three lines for its
/// own scaling and they are private to it (§31.5 freezes that crate), so the
/// seam is paid for in nine lines rather than reopened.
///
/// # Panics
///
/// If `value` is zero, which has no such exponent. Callers here skip zero
/// entries rather than trust an expansion not to hold one: the sentence that
/// used to stand here said an expansion's entries are non-zero, and
/// [`Exact::product_of`] stored a zero low half for every product that came out
/// exact, which is what kept a slide that would have worked from being tried.
pub(crate) fn binary_exponent(value: f64) -> i32 {
    assert!(value != 0.0, "zero has no binary exponent");
    let bits = value.abs().to_bits();
    let stored = (bits >> 52) as i32;
    if stored > 0 {
        return stored - 1023;
    }
    let mantissa = bits & ((1 << 52) - 1);
    63 - mantissa.leading_zeros() as i32 - 1074
}

/// `value * 2^exponent`, in two steps so the factor itself never leaves the
/// range.
///
/// `2f64.powi(1074)` is infinity, and a single multiplication by it throws away
/// products that would have been ordinary numbers. Halving the exponent keeps
/// both factors well inside what `f64` holds, and each step moves the magnitude
/// the same way, so neither intermediate is further out than the answer.
pub(crate) fn multiplied_by_a_power_of_two(value: f64, exponent: i32) -> f64 {
    let half = exponent / 2;
    value * 2f64.powi(half) * 2f64.powi(exponent - half)
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

    /// An expansion enclosed is as tight as an enclosure of it gets, and that is
    /// the whole reason two nearly-parallel axes are decidable.
    ///
    /// Two directions one last bit apart cross into something whose every
    /// component cancelled almost entirely away. Summed through the expansion
    /// the squared length of that cross product is known to full relative
    /// accuracy; built out of intervals from the start it carries a rounding of
    /// the *inputs* rather than of the answer, and reaches down to zero — so a
    /// distance that divides by its root comes back as everything.
    #[test]
    fn an_expansion_encloses_itself_far_more_tightly_than_the_same_sum_of_intervals_does() {
        let one: [f64; 3] = [
            0.267_261_241_912_424_4,
            0.534_522_483_824_848_8,
            0.801_783_725_737_273_2,
        ];
        let other = [one[0], one[1], one[2].next_up()];
        let across = |axis: usize| {
            let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
            (one[next], other[after], one[after], other[next])
        };

        let exact = (0..3)
            .try_fold(Exact::of(0.0), |so_far, axis| {
                let (a, b, c, d) = across(axis);
                let part = Exact::product_of(a, b)?.minus(&Exact::product_of(c, d)?);
                Some(so_far.plus(&part.times(&part)?))
            })
            .expect("everything here is of order one");
        let enclosed = exact.enclosure();
        assert!(
            enclosed.lo() > 0.0,
            "two directions a last bit apart are not parallel, and this says so: {enclosed:?}"
        );

        let by_intervals = (0..3).fold(rcad_predicates::Interval::exactly(0.0), |so_far, axis| {
            let (a, b, c, d) = across(axis);
            let part = rcad_predicates::Interval::exactly(a)
                * rcad_predicates::Interval::exactly(b)
                - rcad_predicates::Interval::exactly(c) * rcad_predicates::Interval::exactly(d);
            so_far + part * part
        });
        assert!(
            by_intervals.lo() <= 0.0,
            "the same sum built out of intervals carries a rounding of its inputs rather than \
             of its answer, and this one is {by_intervals:?}"
        );
        assert!(
            enclosed.width() <= enclosed.hi() * 8.0 * f64::EPSILON,
            "the expansion already is the answer, so enclosing it costs a rounding or two of \
             the answer and not of the inputs: {enclosed:?}"
        );
        assert!(
            by_intervals.width() > enclosed.hi() * 3.0,
            "where the interval sum is wider than the answer itself: {by_intervals:?}"
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

/// Where exact arithmetic runs out, measured rather than argued.
///
/// `docs/decisions.md` §24.17 asks for the largest of M3's limit-or-gap claims
/// to be made to stand up: *the general quartic has no exact naming*. This is
/// the half of that argument which is a demonstration.
///
/// **An expansion can only be squared while its own dynamic range leaves room.**
/// [`Exact::times`] refuses as soon as one product's low half would land among
/// the subnormals, and squaring doubles the spread between the largest entry and
/// the smallest. So there is a spread past which no scaling of the inputs helps
/// — and the general two-pipe discriminant is past it for two perfectly ordinary
/// pipes at unit size.
#[cfg(test)]
mod how_far_exact_arithmetic_reaches {
    use rcad_predicates::Decision;

    use super::{Exact, WIDEST_SQUARABLE_SPREAD};

    /// Two turned unit directions and an offset between them, none of whose
    /// coordinates is anything but a full-precision `f64`. Nothing about them is
    /// awkward: they are what one general rotation of a lattice pair leaves.
    const ONE_WAY: [f64; 3] = [
        0.267_261_241_912_424_4,
        0.534_522_483_824_848_8,
        0.801_783_725_737_273_2,
    ];
    const THE_OTHER_WAY: [f64; 3] = [
        -0.408_248_290_463_863,
        0.816_496_580_927_726,
        -0.408_248_290_463_863,
    ];
    const BETWEEN: [f64; 3] = [
        1.234_567_890_123_456_7,
        -2.345_678_901_234_568,
        3.456_789_012_345_679,
    ];

    /// `T = (o₂ − o₁)·(w₁ × w₂)` and `K = ‖w₁ × w₂‖²` for the pair above.
    fn a_leaning_pair() -> (Exact, Exact, Exact) {
        let sideways = Exact::cross_of(&Exact::of_each(ONE_WAY), THE_OTHER_WAY)
            .expect("two unit directions cross exactly");
        let volume = (0..3)
            .try_fold(Exact::of(0.0), |so_far, axis| {
                Some(so_far.plus(&Exact::of(BETWEEN[axis]).times(&sideways[axis])?))
            })
            .expect("an offset of order one times a direction of order one is exact");
        let squared_sideways =
            Exact::squared_length_of(&sideways).expect("a sum of three squares of order one");
        let squared_direction = Exact::dot_of(ONE_WAY, ONE_WAY).expect("a unit direction squared");
        (volume, squared_sideways, squared_direction)
    }

    /// The eighth-degree question a ball and a pipe ask stays inside twenty
    /// entries, so squaring it costs nothing and level 3 decides it.
    #[test]
    fn the_eighth_degree_question_a_ball_and_a_pipe_ask_is_short_enough_to_square() {
        let along = ONE_WAY;
        let from = Exact::gap_between([0.5, -0.25, 0.125], BETWEEN);
        let across = Exact::cross_of(&from, along).expect("a cross product of order one");
        let area = Exact::squared_length_of(&across).expect("a sum of three squares");
        let room = area
            .plus(&Exact::product_of(2.5, 2.5).expect("a radius squared"))
            .minus(
                &Exact::dot_of(along, along)
                    .expect("a unit direction squared")
                    .times(&Exact::product_of(3.75, 3.75).expect("a radius squared"))
                    .expect("both are of order one"),
            );

        assert!(room.entries() <= 20, "V is {} entries", room.entries());
        let squared = room.times(&room).expect("this one squares");
        assert!(
            squared.entries() <= 40,
            "V² is {} entries",
            squared.entries()
        );
    }

    /// The question two leaning pipes ask does **not** fit, and this is the
    /// demonstration §24.17 asks for.
    ///
    /// Deciding which of the four shapes two leaning pipes cross in compares
    /// `ρ₁ ± ρ₂` against `δ`, and those are `rᵢ/√Wᵢ` and `|T|/√K` — three square
    /// roots in three different metrics. Clearing them takes squaring
    /// `r₁²W₂K + r₂²W₁K − T²W₁W₂`, and that expansion is already **five hundred
    /// binary orders wide**: its smallest entry is `2^-500` of its largest, so
    /// the square of that entry lands a thousand orders down, past the bottom of
    /// `f64`.
    ///
    /// **No scaling of the inputs reaches it.** Multiplying every length by a
    /// power of two multiplies every entry of the expansion by the same factor,
    /// so the spread is exactly what a scaling cannot change — which is what
    /// makes this different in kind from
    /// `a_ball_too_small_for_its_own_squares_cannot_be_crossed_and_the_same_one_scaled_up_can`,
    /// where a rescale walks straight back out of the refusal.
    ///
    /// What a scaling *can* do is slide the doubled spread into the window `f64`
    /// leaves, and the second half of this test shows that it does: the same
    /// expansion written far enough up the range squares without complaint. So
    /// the refusal is a **gap** and not a limit — the answer exists and this
    /// arithmetic, at this scale, could not reach it — and its fix is
    /// [`Exact::square_minus`], which does that slide for the caller.
    /// `a_comparison_too_wide_to_square_is_decided_after_it_is_slid_up_the_range`
    /// is the same numbers going through it.
    #[test]
    fn the_question_two_leaning_pipes_ask_is_too_wide_to_square_where_it_stands() {
        let (volume, squared_sideways, squared_direction) = a_leaning_pair();
        let radii = |radius: f64| {
            Exact::product_of(radius, radius)
                .expect("a radius squared")
                .times(&squared_direction)
                .expect("both are of order one")
                .times(&squared_sideways)
                .expect("and so is this")
        };
        let against_the_gap = radii(1.0).plus(&radii(2.0)).minus(
            &volume
                .times(&volume)
                .expect("an offset squared")
                .times(&squared_direction)
                .expect("times a unit direction squared")
                .times(&squared_direction)
                .expect("and the other one"),
        );

        assert!(
            against_the_gap.spread() > 400,
            "this expansion spans {} binary orders, which was meant to be the point",
            against_the_gap.spread()
        );
        assert_eq!(
            against_the_gap.times(&against_the_gap).map(|_| ()),
            None,
            "squaring an expansion this wide runs off the bottom of f64"
        );

        // The same number written far enough up the range, which multiplies
        // every entry by one positive power of two and so cannot change its
        // sign. There the square fits.
        let up = Exact::of(2f64.powi(300));
        let higher = against_the_gap
            .times(&up)
            .expect("scaling by a power of two rounds nothing");
        assert!(
            higher.spread() > 400,
            "a scaling slides an expansion up the range; it does not narrow it, and this one \
             is still {} orders wide",
            higher.spread()
        );
        assert!(
            higher.times(&higher).is_some(),
            "the answer was there to be had, one scaling away"
        );
    }

    /// The answer that was one scaling away, taken.
    ///
    /// `M² − N²` for the same two leaning pipes, where `M` is the expansion the
    /// test above cannot square where it stands. [`Exact::square_minus`] slides
    /// `M` up until the square fits and `N²` up by twice as much, so both sides
    /// are multiplied by the same positive number and the difference keeps its
    /// sign.
    ///
    /// The oracle is a **second slide, by a different exponent**. A sign that
    /// depended on how far the comparison was slid would not be a sign, so
    /// asking twice from two different heights is what says this is arithmetic
    /// rather than luck.
    #[test]
    fn a_comparison_too_wide_to_square_is_decided_after_it_is_slid_up_the_range() {
        let (volume, squared_sideways, squared_direction) = a_leaning_pair();
        let radii = |radius: f64| {
            Exact::product_of(radius, radius)
                .expect("a radius squared")
                .times(&squared_direction)
                .expect("both are of order one")
                .times(&squared_sideways)
                .expect("and so is this")
        };
        let room = radii(1.0).plus(&radii(2.0)).minus(
            &volume
                .times(&volume)
                .expect("an offset squared")
                .times(&squared_direction)
                .expect("times a unit direction squared")
                .times(&squared_direction)
                .expect("and the other one"),
        );
        // `N² = 4r₁²r₂²K²W₁W₂`, with the radii one and two.
        let together_squared = Exact::product_of(4.0, 4.0)
            .expect("four times a radius times a radius")
            .times(&squared_sideways)
            .expect("K")
            .times(&squared_sideways)
            .expect("K again")
            .times(&squared_direction)
            .expect("W₁")
            .times(&squared_direction)
            .expect("W₂");

        assert_eq!(
            room.times(&room).map(|_| ()),
            None,
            "the whole point is that this one cannot be squared where it stands"
        );

        let slid = room
            .square_minus(&together_squared)
            .expect("the answer is inside the window");
        assert!(
            matches!(slid.sign(), Decision::Pos | Decision::Neg | Decision::Zero),
            "a slid comparison is a certified sign: {:?}",
            slid.sign()
        );

        let up = Exact::of(2f64.powi(300));
        let by_hand = {
            let higher = room.times(&up).expect("a power of two rounds nothing");
            higher
                .times(&higher)
                .expect("and up here the square fits")
                .minus(
                    &together_squared
                        .times(&up)
                        .expect("the other side follows")
                        .times(&up)
                        .expect("by twice as much"),
                )
        };
        assert_eq!(
            slid.sign(),
            by_hand.sign(),
            "two slides of different heights have to agree, or the sign was never there"
        );
    }

    /// The published window, tried at every height `f64` has rather than argued.
    ///
    /// §30.4 published it — every pairwise product exact needs the smallest entry
    /// above `2^-484` and the largest below `2^512` — and §15.6 says a published
    /// bound needs a test. This is that test, and it is a demonstration in
    /// CLAUDE.md's sense: an expansion one binary order wider than the window is
    /// written at **every** placement in the normal range and refused at all of
    /// them, and one exactly as wide as the window is squared at every one.
    #[test]
    fn an_expansion_wider_than_the_window_cannot_be_squared_at_any_scale() {
        let two_entries_apart = |high: i32, spread: i32| {
            Exact::of(2f64.powi(high)).plus(&Exact::of(2f64.powi(high - spread)))
        };
        let lowest_normal = -1022;

        let too_wide = WIDEST_SQUARABLE_SPREAD + 1;
        let mut refused = 0;
        for high in (lowest_normal + too_wide)..=1023 {
            let wide = two_entries_apart(high, too_wide);
            assert_eq!(wide.spread(), too_wide, "written at 2^{high}");
            assert_eq!(
                wide.square_minus(&Exact::of(0.0)).map(|_| ()),
                None,
                "an expansion {too_wide} orders wide has no square at 2^{high} either"
            );
            refused += 1;
        }
        assert!(refused > 1000, "only {refused} placements were tried");

        let widest = WIDEST_SQUARABLE_SPREAD;
        for high in (lowest_normal + widest)..=1023 {
            let fits = two_entries_apart(high, widest);
            assert!(
                fits.square_minus(&Exact::of(0.0)).is_some(),
                "an expansion exactly as wide as the window has a square somewhere, and at \
                 2^{high} this one did not find it"
            );
        }
    }

    /// A product whose low half came out zero still slides before it is squared.
    ///
    /// [`Exact::product_of`] stored `[0.0, high]` for every product that landed
    /// on an `f64` exactly, and
    /// [`Exact::exponent_that_makes_its_square_exact`] read that zero as the
    /// smallest entry and answered "already as low as it goes, do not slide" —
    /// so a number with a perfectly good square somewhere else was refused where
    /// it stood, whatever its magnitude. Nothing in the kernel hands
    /// [`Exact::square_minus`] a raw product today, which is why a green suite
    /// never saw it.
    ///
    /// `2^300 · 2^300` is that product: it is `2^600` and nothing else, and
    /// `2^600` squared is past the top of the range where it stands.
    #[test]
    fn a_product_whose_low_half_is_zero_is_still_slid_before_it_is_squared() {
        let big = Exact::product_of(2f64.powi(300), 2f64.powi(300)).expect("an exact product");
        assert_eq!(
            big.times(&big).map(|_| ()),
            None,
            "the whole point is that this one cannot be squared where it stands"
        );
        let slid = big
            .square_minus(&Exact::of(0.0))
            .expect("a slide reaches this square");
        assert_eq!(slid.sign(), Decision::Pos);
    }

    /// A comparison that already fits is not moved, so nothing this lane did
    /// changes the arithmetic of a question that was being answered yesterday.
    #[test]
    fn a_square_that_already_fits_is_worked_out_exactly_where_it_stands() {
        let ordinary = Exact::of(3.0).plus(&Exact::of(0.25));
        assert_eq!(
            ordinary
                .square_minus(&Exact::of(0.0))
                .expect("of order one")
                .nearest_f64(),
            3.25 * 3.25
        );
    }
}
