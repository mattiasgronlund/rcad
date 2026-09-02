//! Where a polynomial in one unknown is zero.
//!
//! A quadric answers "where does this line meet me" with a quadratic, and
//! [`three_surfaces`](crate::three_surfaces) reads its two roots straight off
//! the discriminant. A torus answers with a **quartic**, and there is no
//! discriminant to read: Ferrari's closed form goes through a cube root and a
//! square root of a number that is itself a root, and every one of those steps
//! is a place where two large numbers cancel and the answer walks away.
//!
//! # What is here instead: the derivative says where to look, the sign says what
//! is there
//!
//! Between two roots of a polynomial there is a turning point, so the turning
//! points cut the line into stretches each holding **at most one** root. The
//! turning points are the roots of the derivative, which is one degree lower —
//! so a quartic asks a cubic, a cubic asks a quadratic, a quadratic asks a line,
//! and a line asks nothing. Four levels and no algebra.
//!
//! On each stretch the two ends have signs, and those signs are worked out
//! **exactly**: the coefficients are [`Exact`] expansions, and a polynomial
//! evaluated at an `f64` by Horner's rule is products and sums of exact numbers,
//! which [`Exact`] does without rounding. A sign change is then a root and no
//! sign change is no root — neither is a guess, and no tolerance decides either
//! of them. The interval evaluation above it is a filter and never an answer: it
//! settles the sign where it is plainly away from zero, and hands over where it
//! is not.
//!
//! Narrowing a root is bisection on the **bit pattern** rather than on the
//! value, so a bracket from `-1e300` to `1e300` closes onto two neighbouring
//! `f64` in about sixty-four steps rather than two thousand.
//!
//! # A touch is a real answer, and there is one it cannot give
//!
//! A line tangent to a torus is a **double root** — the polynomial reaches zero
//! and turns back without changing sign — and it is a meeting point like any
//! other. Where the touch lands on an `f64`, the exact evaluation there comes
//! out zero and it is answered with no width at all; a bend of pipe's own seam
//! is that case, because the seam runs along a circle the arithmetic hits
//! square.
//!
//! Where it does not land on an `f64`, a turning point a hair above zero and one
//! a hair below cannot be told apart by any number of sign evaluations, because
//! neither has a sign change to find. Those come back as [`Maybe`] — the box the
//! turning point is in and how far the curve reaches over it — and it is the
//! caller's policy that says what to do about it. A **gap** and not a limit: the
//! answer exists and this arithmetic does not reach it. The family of fixes is
//! (a) the quartic's discriminant, exactly, which is degree six in coefficients
//! that are already degree fourteen in the stored numbers — sixteen terms each a
//! product of six expansions, which `Exact::times` grows past anything usable;
//! (b) a subresultant chain, which is the same blow-up one step cheaper; (c)
//! evaluating the turning point itself to more than `f64`, which wants an
//! arithmetic this kernel does not have. All three are the same milestone of
//! algebra and none of them is this lane.

use rcad_predicates::{Decision, Interval, NOTHING_IS_KNOWN};

use crate::exact::Exact;

/// Where a polynomial is zero.
pub(crate) struct WhereItIsZero {
    /// One box per root, ascending, each certified to hold one.
    pub(crate) at: Vec<Interval>,
    /// One box per turning point whose height could not be told from zero.
    pub(crate) or_maybe_at: Vec<Maybe>,
}

/// A turning point the curve may or may not reach zero at.
pub(crate) struct Maybe {
    /// The box the turning point sits in. Two neighbouring `f64`, or one.
    pub(crate) inside: Interval,
    /// How far the curve reaches over that box. It straddles zero, or this
    /// would not be here.
    pub(crate) how_far_it_reaches: Interval,
}

/// Where `coefficients` is zero, highest power first.
///
/// `None` when a product the exact arithmetic needs falls out of `f64`'s range,
/// or when a root is so far out that the search for a bracket round it runs off
/// the end of the number line. Both are the same **gap** every discriminant in
/// [`intersect`](crate::intersect) has, and the same three surfaces written
/// somewhere else on the number line are answered.
///
/// # Panics
///
/// If the leading coefficient is zero. A caller hands over a polynomial of a
/// degree it has already established; a quartic whose leading term vanished is
/// a different question asked through the same door.
pub(crate) fn where_a_polynomial_is_zero(coefficients: &[Exact]) -> Option<WhereItIsZero> {
    let degree = coefficients.len() - 1;
    if degree == 0 {
        return Some(WhereItIsZero {
            at: Vec::new(),
            or_maybe_at: Vec::new(),
        });
    }
    let leading = coefficients[0].sign();
    assert!(
        matches!(leading, Decision::Pos | Decision::Neg),
        "a polynomial of degree {degree} has a leading coefficient that is not zero"
    );
    let turning = where_a_polynomial_is_zero(&the_derivative_of(coefficients)?)?;

    let mut at = Vec::new();
    let mut or_maybe_at = Vec::new();
    let boxes = the_boxes_it_turns_in(&turning);
    // **Every stretch's own ends, first and on their own.** A touch is a place
    // the curve reaches zero and turns back without changing sign, so a search
    // for a sign change never finds one; what finds it is asking the ends of the
    // boxes the curve turns in, exactly, and a bend of pipe's seam is exactly
    // that case.
    for turn in &boxes {
        for end in [turn.lo(), turn.hi()] {
            if matches!(a_settled_sign_at(coefficients, end)?, Decision::Zero) {
                at.push(Interval::exactly(end));
            }
        }
    }
    for stretch in the_stretches_between(&boxes) {
        what_lies_on(
            coefficients,
            &stretch,
            leading,
            degree,
            &mut at,
            &mut or_maybe_at,
        )?;
    }
    at.sort_by(|one, other| one.lo().total_cmp(&other.lo()));
    at.dedup_by(|one, other| one.lo() == other.lo() && one.hi() == other.hi());
    Some(WhereItIsZero { at, or_maybe_at })
}

/// The derivative's coefficients, exactly, highest power first.
fn the_derivative_of(coefficients: &[Exact]) -> Option<Vec<Exact>> {
    let degree = coefficients.len() - 1;
    (0..degree)
        .map(|power| {
            let power_it_carried =
                u32::try_from(degree - power).expect("a degree here is four at most");
            coefficients[power].times(&Exact::of(f64::from(power_it_carried)))
        })
        .collect()
}

/// The boxes every turning point lies in, ascending and none of them touching.
///
/// A turning point that was placed and one that could only be bracketed do the
/// same job here — what the stretches below need is that **no turning point is
/// outside these boxes**, and both kinds carry that.
fn the_boxes_it_turns_in(turning: &WhereItIsZero) -> Vec<Interval> {
    let mut boxes: Vec<Interval> = turning
        .at
        .iter()
        .copied()
        .chain(turning.or_maybe_at.iter().map(|maybe| maybe.inside))
        .collect();
    boxes.sort_by(|one, other| one.lo().total_cmp(&other.lo()));

    let mut merged: Vec<Interval> = Vec::new();
    for each in boxes {
        match merged.last_mut() {
            Some(last) if each.lo() <= last.hi() => {
                *last = Interval::new(last.lo(), last.hi().max(each.hi()))
                    .expect("a box that starts where an earlier one has not ended yet");
            }
            _ => merged.push(each),
        }
    }
    merged
}

/// One stretch of the line, and whether a turning point is inside it.
struct Stretch {
    /// Where it starts, or `None` for the far end of the number line.
    from: Option<f64>,
    /// Where it ends, or `None` for the far end of the number line.
    to: Option<f64>,
    /// Whether the curve may turn inside it. A stretch that cannot holds **at
    /// most one** root and a sign change settles it; one that can holds up to
    /// two and a sign change is not the whole story.
    it_may_turn: bool,
}

/// The line cut into stretches by the boxes the curve turns in.
fn the_stretches_between(boxes: &[Interval]) -> Vec<Stretch> {
    let mut stretches = Vec::new();
    let mut so_far: Option<f64> = None;
    for turn in boxes {
        stretches.push(Stretch {
            from: so_far,
            to: Some(turn.lo()),
            it_may_turn: false,
        });
        if turn.hi() > turn.lo() {
            stretches.push(Stretch {
                from: Some(turn.lo()),
                to: Some(turn.hi()),
                it_may_turn: true,
            });
        }
        so_far = Some(turn.hi());
    }
    stretches.push(Stretch {
        from: so_far,
        to: None,
        it_may_turn: false,
    });
    stretches
}

/// What roots one stretch holds, added to `at` and `or_maybe_at`.
///
/// **A stretch whose own end the curve sits on is finished there**, and that is
/// the monotone argument being spent rather than a corner cut: a stretch the
/// curve cannot turn on holds at most one root, so an end that is one is the
/// only one, and it was recorded when the ends were read. A stretch the curve
/// *can* turn on is two neighbouring `f64` wide, so there is nothing strictly
/// inside it for a second root to be at.
fn what_lies_on(
    coefficients: &[Exact],
    stretch: &Stretch,
    leading: Decision,
    degree: usize,
    at: &mut Vec<Interval>,
    or_maybe_at: &mut Vec<Maybe>,
) -> Option<()> {
    let start = match stretch.from {
        Some(place) => Some((place, a_settled_sign_at(coefficients, place)?)),
        None => None,
    };
    let end = match stretch.to {
        Some(place) => Some((place, a_settled_sign_at(coefficients, place)?)),
        None => None,
    };
    let below = start.map_or_else(|| the_sign_far_below(leading, degree), |(_, sign)| sign);
    let above = end.map_or(leading, |(_, sign)| sign);
    if matches!(below, Decision::Zero) || matches!(above, Decision::Zero) {
        return Some(());
    }

    if below == above {
        if stretch.it_may_turn
            && let (Some((low, _)), Some((high, _))) = (start, end)
        {
            let inside = Interval::new(low, high).expect("a stretch runs upwards");
            let how_far_it_reaches = enclosing(coefficients, inside);
            if how_far_it_reaches.contains_zero() {
                or_maybe_at.push(Maybe {
                    inside,
                    how_far_it_reaches,
                });
            }
        }
        return Some(());
    }

    let (low, high) = match (start, end) {
        (Some((low, _)), Some((high, _))) => (low, high),
        (None, Some((high, _))) => match out_from(coefficients, high, false, below)? {
            Reached::TheRoot(place) => return pushed(at, place),
            Reached::A(place) => (place, high),
        },
        (Some((low, _)), None) => match out_from(coefficients, low, true, above)? {
            Reached::TheRoot(place) => return pushed(at, place),
            Reached::A(place) => (low, place),
        },
        (None, None) => {
            let sign = a_settled_sign_at(coefficients, 0.0)?;
            if matches!(sign, Decision::Zero) {
                return pushed(at, 0.0);
            }
            if sign == below {
                match out_from(coefficients, 0.0, true, above)? {
                    Reached::TheRoot(found) => return pushed(at, found),
                    Reached::A(found) => (0.0, found),
                }
            } else {
                match out_from(coefficients, 0.0, false, below)? {
                    Reached::TheRoot(found) => return pushed(at, found),
                    Reached::A(found) => (found, 0.0),
                }
            }
        }
    };
    at.push(narrowed(coefficients, low, high)?);
    Some(())
}

/// The sign of the curve out beyond every root, on the low side.
fn the_sign_far_below(leading: Decision, degree: usize) -> Decision {
    if degree.is_multiple_of(2) {
        return leading;
    }
    if matches!(leading, Decision::Pos) {
        return Decision::Neg;
    }
    assert!(
        matches!(leading, Decision::Neg),
        "a leading coefficient is not zero: {leading:?}"
    );
    Decision::Pos
}

/// One root, recorded, and nothing left to do on this stretch.
fn pushed(at: &mut Vec<Interval>, place: f64) -> Option<()> {
    at.push(Interval::exactly(place));
    Some(())
}

/// Where a search outwards for a bracket ended up.
enum Reached {
    /// A place where the curve has the sign that was wanted.
    A(f64),
    /// The root itself, landed on square.
    TheRoot(f64),
}

/// A finite place out beyond `from` where the curve has `wanted`.
///
/// The step doubles, so the search reaches a root at `1e300` in about a thousand
/// evaluations and one at `3.5` in two. `None` when the line runs out first,
/// which means the root is past the top of `f64`'s range and there is nothing
/// left for a box to be around.
fn out_from(coefficients: &[Exact], from: f64, upwards: bool, wanted: Decision) -> Option<Reached> {
    let mut step = from.abs().max(1.0);
    loop {
        let place = if upwards { from + step } else { from - step };
        if !place.is_finite() {
            return None;
        }
        let sign = a_settled_sign_at(coefficients, place)?;
        if matches!(sign, Decision::Zero) {
            return Some(Reached::TheRoot(place));
        }
        if sign == wanted {
            return Some(Reached::A(place));
        }
        step *= 2.0;
        if !step.is_finite() {
            return None;
        }
    }
}

/// The root between two places whose signs differ, narrowed until the two are
/// neighbours.
fn narrowed(coefficients: &[Exact], low: f64, high: f64) -> Option<Interval> {
    let (mut low, mut high) = (low, high);
    let at_the_bottom = a_settled_sign_at(coefficients, low)?;
    while let Some(middle) = between(low, high) {
        let sign = a_settled_sign_at(coefficients, middle)?;
        if matches!(sign, Decision::Zero) {
            return Some(Interval::exactly(middle));
        }
        if sign == at_the_bottom {
            low = middle;
        } else {
            high = middle;
        }
    }
    Some(Interval::new(low, high).expect("a bracket runs upwards"))
}

/// The `f64` halfway between two others **by bit pattern**, or `None` when they
/// are already neighbours.
///
/// Halving the value would take two thousand steps to cross `f64`'s range and
/// would stall entirely on a bracket that straddles the subnormals. Halving the
/// bit pattern crosses the whole range in sixty-four, because that is what an
/// exponent field is.
fn between(low: f64, high: f64) -> Option<f64> {
    let (bottom, top) = (in_order(low), in_order(high));
    if top - bottom < 2 {
        return None;
    }
    Some(from_order(bottom + (top - bottom) / 2))
}

/// An `f64`'s bit pattern rearranged so that comparing the patterns is
/// comparing the numbers.
fn in_order(value: f64) -> i128 {
    let bits = i128::from(value.to_bits() as i64);
    if bits < 0 {
        i128::from(i64::MIN) - bits
    } else {
        bits
    }
}

/// [`in_order`] the other way round.
#[expect(
    clippy::cast_possible_truncation,
    reason = "every key here came out of in_order, so it names an f64"
)]
fn from_order(key: i128) -> f64 {
    let bits = if key < 0 {
        i128::from(i64::MIN) - key
    } else {
        key
    };
    f64::from_bits(bits as i64 as u64)
}

/// The sign of the curve at `place`, decided and never guessed.
///
/// The interval evaluation is a filter: where it stays clear of zero it is the
/// answer and costs nothing, and where it does not it says nothing at all and
/// the exact evaluation below it decides. [`Decision::Undecidable`] when a
/// product that evaluation needs falls out of `f64`'s range — exact arithmetic
/// that is not exact has nothing to offer, so that level cannot run.
fn sign_at(coefficients: &[Exact], place: f64) -> Decision {
    let enclosed = enclosing(coefficients, Interval::exactly(place));
    if enclosed.lo() > 0.0 {
        return Decision::Pos;
    }
    if enclosed.hi() < 0.0 {
        return Decision::Neg;
    }
    let Some(exactly) = exactly_at(coefficients, place) else {
        return NOTHING_IS_KNOWN;
    };
    exactly.sign()
}

/// The sign of the curve at `place`, or `None` where it could not be decided,
/// which is what every caller here does with that.
fn a_settled_sign_at(coefficients: &[Exact], place: f64) -> Option<Decision> {
    let sign = sign_at(coefficients, place);
    if matches!(sign, Decision::Undecidable { .. }) {
        return None;
    }
    Some(sign)
}

/// How far the curve reaches over `over`, by Horner's rule in intervals.
fn enclosing(coefficients: &[Exact], over: Interval) -> Interval {
    coefficients[1..]
        .iter()
        .fold(coefficients[0].enclosure(), |so_far, next| {
            so_far * over + next.enclosure()
        })
}

/// The curve's value at `place`, exactly, by Horner's rule.
fn exactly_at(coefficients: &[Exact], place: f64) -> Option<Exact> {
    let step = Exact::of(place);
    let mut running = coefficients[0].clone();
    for next in &coefficients[1..] {
        running = running.times(&step)?.plus(next);
    }
    Some(running)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roots_of(coefficients: &[f64]) -> Vec<f64> {
        let exact: Vec<Exact> = coefficients.iter().copied().map(Exact::of).collect();
        let found = where_a_polynomial_is_zero(&exact).expect("all of this fits");
        assert!(
            found.or_maybe_at.is_empty(),
            "expected every root settled, and {} were not",
            found.or_maybe_at.len()
        );
        found
            .at
            .iter()
            .map(|root| (root.lo() + root.hi()) / 2.0)
            .collect()
    }

    #[test]
    fn a_straight_line_crosses_zero_where_it_says() {
        assert_eq!(roots_of(&[2.0, -6.0]), vec![3.0]);
        assert_eq!(roots_of(&[-1.0, -5.0]), vec![-5.0]);
    }

    #[test]
    fn a_quadratic_gives_both_roots_or_the_one_it_touches_at_or_none() {
        assert_eq!(roots_of(&[1.0, 0.0, -4.0]), vec![-2.0, 2.0]);
        assert_eq!(roots_of(&[1.0, -4.0, 4.0]), vec![2.0]);
        assert_eq!(roots_of(&[1.0, 0.0, 4.0]), Vec::<f64>::new());
    }

    #[test]
    fn a_cubic_with_three_roots_gives_all_three() {
        assert_eq!(roots_of(&[1.0, -6.0, 11.0, -6.0]), vec![1.0, 2.0, 3.0]);
    }

    /// `(s+5)(s+3)(s−3)(s−5)`, which is what the `x` axis makes of a ring of
    /// four and one.
    #[test]
    fn a_quartic_with_four_roots_gives_all_four() {
        assert_eq!(
            roots_of(&[1.0, 0.0, -34.0, 0.0, 225.0]),
            vec![-5.0, -3.0, 3.0, 5.0]
        );
    }

    /// `s²(s−4)(s+4)` — a double root at zero and two simple ones, which is
    /// three places and not four.
    #[test]
    fn a_quartic_that_touches_zero_once_and_crosses_twice_gives_three_places() {
        assert_eq!(roots_of(&[1.0, 0.0, -16.0, 0.0, 0.0]), vec![-4.0, 0.0, 4.0]);
    }

    /// `(s−2)²(s²+1)` — one place, and it is a touch.
    #[test]
    fn a_quartic_that_only_touches_gives_one_place() {
        assert_eq!(roots_of(&[1.0, -4.0, 5.0, -4.0, 4.0]), vec![2.0]);
    }

    /// A root nowhere near an `f64` still comes back inside two neighbouring
    /// ones, and the polynomial changes sign across them.
    #[test]
    fn a_root_that_is_not_a_number_comes_back_between_the_two_that_surround_it() {
        let exact: Vec<Exact> = [1.0, 0.0, -2.0].iter().copied().map(Exact::of).collect();
        let found = where_a_polynomial_is_zero(&exact).expect("all of this fits");
        assert_eq!(found.at.len(), 2);
        let upper = found.at[1];
        assert_eq!(upper.hi(), upper.lo().next_up());
        // `√2` is between the two, which is what the two ends squaring to less
        // and to more than two says. The `f64` nearest `√2` is one of them, so
        // comparing against that constant would say nothing.
        assert!(upper.lo() * upper.lo() < 2.0);
        assert!(upper.hi() * upper.hi() > 2.0);
    }

    /// **The one thing this cannot answer, demonstrated rather than argued.**
    ///
    /// `(3s − 1)²` touches zero at `1/3`, which is not an `f64`. The two
    /// neighbouring `f64` either side of `1/3` both give a value a hundred
    /// binary orders above zero and both give it the same sign, so no sign
    /// evaluation anywhere distinguishes this from `(3s − 1)² + tiny`, which
    /// touches nothing. It comes back as a box and a reach, and the caller's
    /// policy says what that is worth.
    ///
    /// A **gap**: the touch is real and this arithmetic does not reach it. The
    /// module's own doc names the family of fixes.
    #[test]
    fn a_touch_that_does_not_land_on_a_number_is_neither_claimed_nor_denied() {
        let exact: Vec<Exact> = [9.0, -6.0, 1.0].iter().copied().map(Exact::of).collect();
        let found = where_a_polynomial_is_zero(&exact).expect("all of this fits");
        assert!(found.at.is_empty(), "nothing here crosses zero");
        assert_eq!(found.or_maybe_at.len(), 1);
        let unsure = &found.or_maybe_at[0];
        // The `f64` nearest a third is below a third, so the box is that one
        // and the one above it, and a third really is inside.
        assert_eq!(unsure.inside.lo(), 1.0 / 3.0);
        assert_eq!(unsure.inside.hi(), (1.0f64 / 3.0).next_up());
        assert!(unsure.how_far_it_reaches.contains_zero());
    }

    /// The same touch moved onto a number the arithmetic can hold is answered
    /// with no width at all. `(2s − 1)²` touches at `1/2`, and a half is an
    /// `f64`.
    #[test]
    fn a_touch_that_does_land_on_a_number_is_answered_exactly() {
        assert_eq!(roots_of(&[4.0, -4.0, 1.0]), vec![0.5]);
    }

    /// The root is at `1e200`, far past where a step of one would ever reach it.
    /// The search doubles, so it gets there.
    #[test]
    fn a_root_far_out_along_the_line_is_still_bracketed() {
        let found = roots_of(&[1.0, -1e200]);
        assert_eq!(found.len(), 1);
        assert!((found[0] - 1e200).abs() <= 1e185);
    }

    /// Bisection on the bit pattern crosses the whole range, including the part
    /// of it below one, in about sixty-four steps rather than two thousand.
    #[test]
    fn halfway_between_two_numbers_is_halfway_through_their_bit_patterns() {
        assert_eq!(between(-1.0, 1.0), Some(0.0));
        assert_eq!(between(1.0, 1.0f64.next_up()), None);
        assert_eq!(between(0.0, 0.0), None);
        let mut steps = 0;
        let (mut low, mut high) = (-1e300, 1e300);
        while let Some(middle) = between(low, high) {
            if middle < 0.5 {
                low = middle;
            } else {
                high = middle;
            }
            steps += 1;
        }
        assert!(steps < 80, "narrowing the whole range took {steps} steps");
        assert_eq!(high, 0.5);
    }
}
