//! Every knot vector on a small grid, judged twice and made to agree.
//!
//! Degrees 1, 2 and 3; knots drawn from the whole numbers 0 to 7; every length
//! from a single knot up to `2 * (degree + 1) + 4`. That is every non-decreasing
//! sequence those choices allow — **182,595** of them, of which **1,778**
//! describe a knot vector — and they are enumerated, not sampled.
//!
//! Both directions are asserted, because only one of them is the interesting
//! one. That bad input is refused is checked all over this tree; that good input
//! is *accepted* is not, and a constructor that refuses far more than it must
//! passes every refusal test there is (`docs/decisions.md` §12.3, §14.7). So:
//! zero wrongly refused and zero wrongly accepted.
//!
//! The judge is [`the_rules_call_this_a_knot_vector`] at the bottom of this
//! file: a flat transcription of the six invariants, one loop each, written to
//! be read against the rules rather than to be quick. It shares no code with
//! `KnotVector::new` on purpose — an oracle that calls the thing it is judging
//! agrees with it by construction.

use rcad_geom::{KnotVector, MAX_DEGREE};

/// The largest whole number the grid runs to.
const HIGHEST_KNOT: f64 = 7.0;

/// How many sequences the three degrees and their lengths come to.
const SEQUENCES_ON_THE_GRID: usize = 182_595;

/// How many of those are knot vectors.
const KNOT_VECTORS_AMONG_THEM: usize = 1_778;

#[test]
fn a_sequence_is_accepted_exactly_when_the_six_rules_say_it_is_a_knot_vector() {
    let mut enumerated = 0;
    let mut accepted = 0;

    for degree in 1..=3usize {
        for length in 1..=2 * (degree + 1) + 4 {
            for_each_non_decreasing_sequence(length, &mut Vec::new(), &mut |knots| {
                enumerated += 1;
                let rules_say_yes = the_rules_call_this_a_knot_vector(degree, knots);

                match (rules_say_yes, KnotVector::new(degree, knots.to_vec())) {
                    (true, Ok(built)) => {
                        accepted += 1;
                        assert_eq!(built.degree(), degree);
                        assert_eq!(built.knots(), knots);
                    }
                    (false, Err(_)) => {}
                    (true, Err(refusal)) => panic!(
                        "{knots:?} at degree {degree} keeps all six rules and was refused: {refusal}"
                    ),
                    (false, Ok(_)) => panic!(
                        "{knots:?} at degree {degree} breaks one of the six rules and was accepted"
                    ),
                }
            });
        }
    }

    assert_eq!(enumerated, SEQUENCES_ON_THE_GRID);
    assert_eq!(accepted, KNOT_VECTORS_AMONG_THEM);
}

/// Hands `visit` every non-decreasing sequence of `length` whole numbers
/// between zero and [`HIGHEST_KNOT`].
fn for_each_non_decreasing_sequence(
    length: usize,
    so_far: &mut Vec<f64>,
    visit: &mut impl FnMut(&[f64]),
) {
    if so_far.len() == length {
        visit(so_far);
        return;
    }

    let mut knot = so_far.last().copied().unwrap_or(0.0);
    while knot <= HIGHEST_KNOT {
        so_far.push(knot);
        for_each_non_decreasing_sequence(length, so_far, visit);
        so_far.pop();
        knot += 1.0;
    }
}

/// The six rules, one loop each, in the order `KnotVector::new` states them.
///
/// The degree cap and the length come first, because the two clamping rules
/// have nothing to say about a sequence too short to hold two clamps.
///
/// A knot at an interior position is counted across the *whole* sequence, not
/// only the interior of it. A first knot repeated `degree + 2` times is a
/// clamp of the right length with one interior copy stuck to it, and that
/// copy leaves the first control point with a basis function that is zero
/// everywhere.
#[allow(
    clippy::needless_range_loop,
    reason = "the rules are stated about positions, and this reads against them a line at a time"
)]
fn the_rules_call_this_a_knot_vector(degree: usize, knots: &[f64]) -> bool {
    if degree == 0 || degree > MAX_DEGREE {
        return false;
    }
    if knots.len() < 2 * (degree + 1) {
        return false;
    }
    let last = knots.len() - 1;

    for at in 0..=last {
        if !knots[at].is_finite() {
            return false;
        }
    }
    for at in 1..=last {
        if knots[at - 1] > knots[at] {
            return false;
        }
    }
    for at in 0..=degree {
        if knots[at] != knots[0] {
            return false;
        }
    }
    for at in last - degree..=last {
        if knots[at] != knots[last] {
            return false;
        }
    }
    for at in degree + 1..=last - degree - 1 {
        let mut times = 0;
        for other in 0..=last {
            if knots[other] == knots[at] {
                times += 1;
            }
        }
        if times > degree {
            return false;
        }
    }

    let mut some_span_has_width = false;
    for at in degree..last - degree {
        if knots[at] < knots[at + 1] {
            some_span_has_width = true;
        }
    }
    some_span_has_width
}
