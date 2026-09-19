//! Boxes around values nobody can write down exactly.
//!
//! Level 2 of the cascade works in [`Interval`], and so does everything that
//! has to *publish* a bound rather than decide a sign — the marcher's per-point
//! deviation, and the separation tests that say a half-space really does hold
//! two pieces of a crossing apart.
//!
//! # Why there is a square root here and not in `rcad-predicates`
//!
//! Naming where two pipes cross compares a distance against a sum of two radii,
//! and the distance is `|T| / √K` for two polynomials in the stored numbers.
//! Squaring that comparison away costs eight degrees and runs into the limit
//! `docs/decisions.md` §29 measures; enclosing the root costs one rounding.
//!
//! `rcad-predicates` is frozen for M3 (§24.4), and this needs nothing from it:
//! `f64::sqrt` is correctly rounded, so stepping one `f64` outwards at each end
//! of an enclosure gives an enclosure of the root.

use rcad_predicates::Interval;

use crate::de_boor::quotient_enclosing;
use rcad_predicates::expansion::{product_is_exact, two_product};

/// `√value`, as an enclosure, for a value that cannot be below zero.
///
/// The low end is clamped at zero: a sum of squares is never negative, and an
/// enclosure of one that dips a rounding below zero is that rounding and not a
/// negative number.
///
/// # Panics
///
/// If the whole enclosure is below zero, which means a caller asked for the root
/// of something that is not a square.
pub(crate) fn root_enclosing(value: Interval) -> Interval {
    assert!(
        value.hi() >= 0.0,
        "this is not a square, so nothing here takes its root: {value:?}"
    );
    let lo = rounded_down(value.lo().max(0.0)).max(0.0);
    let hi = rounded_up(value.hi());
    Interval::new(lo, hi).expect("a root grows with what it is taken of")
}

/// The largest `f64` at or below `√value`.
///
/// **A step outwards only where one is needed.** Widening every root by an ulp
/// would cost the exact cases their exactness: a pipe whose axis is a coordinate
/// direction has `w·w` exactly one, and an enclosure of `√1` two ulps wide
/// spreads through every point built on it and leaves a contact point that no
/// predicate can place. `f64::sqrt` is correctly rounded, so squaring what it
/// gives back and comparing exactly says which side of the true root it landed.
fn rounded_down(value: f64) -> f64 {
    let root = value.sqrt();
    if squared_is_at_most(root, value) {
        root
    } else {
        root.next_down()
    }
}

/// The smallest `f64` at or above `√value`.
fn rounded_up(value: f64) -> f64 {
    let root = value.sqrt();
    if squared_is_at_most(root, value) && !squared_is_exactly(root, value) {
        root.next_up()
    } else {
        root
    }
}

/// Whether `root · root` is at or below `value`, exactly.
fn squared_is_at_most(root: f64, value: f64) -> bool {
    if !product_is_exact(root, root) {
        return false;
    }
    let (high, low) = two_product(root, root);
    high < value || (high == value && low <= 0.0)
}

/// Whether `root · root` is exactly `value`.
fn squared_is_exactly(root: f64, value: f64) -> bool {
    if !product_is_exact(root, root) {
        return false;
    }
    let (high, low) = two_product(root, root);
    high == value && low == 0.0
}

/// `over / under` as an enclosure, or everything where `under` touches zero.
///
/// [`quotient_enclosing`] panics on a divisor that straddles zero, and it is
/// right to: every caller in the spline evaluator has already proved its divisor
/// is away from zero, and a panic there is a broken invariant.
///
/// **The intersector's divisors are not like that.** They are lengths worked out
/// from the numbers a caller handed over — a direction's own length, the
/// distance between two axes, the reach of a cross product — and a caller is
/// entitled to hand over a direction whose components are so small that its
/// squared length underflows to nothing. That is bad input and not a broken
/// invariant, so what comes back is a box that says nothing is known, which
/// travels through the arithmetic above it and turns into a refusal (§4.1).
///
/// **It travels unless something multiplies it by an exact zero**, and one
/// caller does: `two_lines_told_apart` forms `leaning * how_far` with a
/// `leaning` that is analytically zero and summed exactly, and `[0,0] * ENTIRE`
/// is `[0,0]`. There the unknown is annihilated and no refusal follows — which
/// is the right answer, because an exact zero is a certificate and the true
/// quotient is finite wherever the picking normal existed to be built at all.
/// Do not read the paragraph above as a guarantee when writing a new caller;
/// §142.7 has the measurement and what it does not claim.
pub(crate) fn quotient_or_everything(over: Interval, under: Interval) -> Interval {
    if under.contains_zero() {
        return Interval::ENTIRE;
    }
    quotient_enclosing(over, under)
}

/// `u · v` as an enclosure.
pub(crate) fn dot_enclosing(u: [Interval; 3], v: [Interval; 3]) -> Interval {
    (0..3).fold(Interval::exactly(0.0), |so_far, axis| {
        so_far + u[axis] * v[axis]
    })
}

/// `u × v` as an enclosure.
pub(crate) fn cross_enclosing(u: [Interval; 3], v: [Interval; 3]) -> [Interval; 3] {
    core::array::from_fn(|axis| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        u[next] * v[after] - u[after] * v[next]
    })
}

/// `u · u` as an enclosure.
pub(crate) fn squared_length_enclosing(u: [Interval; 3]) -> Interval {
    dot_enclosing(u, u)
}

/// `there − here` as an enclosure, componentwise.
pub(crate) fn gap_enclosing(here: [f64; 3], there: [f64; 3]) -> [Interval; 3] {
    core::array::from_fn(|axis| Interval::exactly(there[axis]) - Interval::exactly(here[axis]))
}

/// Three stored numbers as an enclosure of no width at all.
pub(crate) fn exactly(value: [f64; 3]) -> [Interval; 3] {
    value.map(Interval::exactly)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_root_of_a_square_encloses_the_number_it_was_squared_from() {
        for value in [1.0, 2.0, 3.0, 1e-30, 1e30, 0.1] {
            let enclosed = root_enclosing(Interval::exactly(value * value));
            assert!(
                enclosed.lo() <= value && value <= enclosed.hi(),
                "√({value}²) came back as {enclosed:?}"
            );
        }
    }

    /// A sum of squares that rounded a hair below zero is that rounding, not a
    /// negative number, and the root of it is zero rather than a panic.
    #[test]
    fn the_root_of_a_range_that_dips_below_zero_starts_at_zero() {
        let barely = Interval::new(-1e-300, 4.0).expect("a range");
        let enclosed = root_enclosing(barely);
        assert_eq!(enclosed.lo(), 0.0);
        assert!(enclosed.hi() >= 2.0);
    }

    #[test]
    #[should_panic(expected = "not a square")]
    fn the_root_of_a_number_that_is_plainly_negative_is_a_bug() {
        let _ = root_enclosing(Interval::new(-4.0, -1.0).expect("a range"));
    }

    /// A square whose root is an `f64` comes back as that `f64` and nothing
    /// wider. Everything built on a coordinate direction depends on it: `w·w` is
    /// exactly one there, and a root of one two ulps wide would spread through
    /// every point worked out from it.
    #[test]
    fn the_root_of_a_perfect_square_has_no_width_at_all() {
        for value in [1.0f64, 4.0, 9.0, 0.25, 1024.0] {
            let enclosed = root_enclosing(Interval::exactly(value));
            assert_eq!(
                (enclosed.lo(), enclosed.hi()),
                (value.sqrt(), value.sqrt()),
                "√{value}"
            );
        }
    }

    /// A direction whose squared length underflows to nothing divides into a
    /// box that says nothing is known, rather than into a panic.
    #[test]
    fn dividing_by_a_length_that_underflowed_to_nothing_says_nothing_is_known() {
        let nothing = Interval::exactly(0.0);
        assert_eq!(
            quotient_or_everything(Interval::exactly(1.0), nothing),
            Interval::ENTIRE
        );
        assert_eq!(
            quotient_or_everything(Interval::exactly(6.0), Interval::exactly(2.0)),
            Interval::exactly(3.0)
        );
    }

    #[test]
    fn a_cross_product_of_two_coordinate_directions_is_the_third() {
        let across = cross_enclosing(exactly([1.0, 0.0, 0.0]), exactly([0.0, 1.0, 0.0]));
        assert_eq!(across.map(Interval::lo), [0.0, 0.0, 1.0]);
        assert_eq!(across.map(Interval::hi), [0.0, 0.0, 1.0]);
    }
}
