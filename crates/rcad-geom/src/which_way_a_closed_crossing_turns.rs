//! Which way the wire round a closed crossing turns, as a decision and not a
//! bare sign.

use rcad_predicates::Decision;

use crate::enclosure::{cross_enclosing, dot_enclosing, exactly};
use crate::exact::Exact;
use crate::point::Vector;

/// Which way the wire round a closed crossing turns against the frame of the
/// curved surface it closes round.
///
/// A closed edge starts and ends at its one corner, so nothing about its ends
/// says which way it goes and only a rule can. The rule is two facts crossed
/// together, and both are read here as *signs* and never as angles, so nothing
/// is normalized (`docs/decisions.md` §36.18).
///
/// - **The wire runs along `outward[0] × outward[1]`** when the face's material
///   is inside its partner, because left of `∇S × ∇T` is always `T`'s negative
///   side: `(∇S × ∇T) × ∇S · ∇T = −‖∇S × ∇T‖²` by Cauchy–Schwarz, everywhere,
///   with no case analysis (§36.6).
/// - **The edge is stored running the way the curved surface's own longitude
///   increases**, which runs along `axis × gradient`.
///
/// [`Decision::Pos`] is the wire running the way the edge is stored.
///
/// # Two references, because one is not enough
///
/// The longitude alone leaves the answer to a wire that is square to it — a ball
/// poking a whisker through the side of a cube puts the dot at exactly zero and
/// both faces then read the same sign, which writes a solid whose one closed
/// edge is walked twice the same way. The surface's own frame at the corner has
/// three legs: the gradient, the longitude `axis × gradient`, and the latitude
/// `gradient × longitude`, mutually square and spanning. The wire is square to
/// the gradient by construction, so it lives in the plane the other two span and,
/// unless it has no length at all, cannot be square to both. So the longitude is
/// read first and the latitude only where the longitude says exactly nothing.
///
/// [`Decision::Zero`] out of both leaves the wire with nothing to be but the
/// zero vector, which is two surfaces touching rather than crossing and has no
/// direction to name. At a pole the longitude is itself exactly zero and this is
/// the answer for the same reason: every longitude runs through that one point,
/// so a sign against it is not a fact about the shape (§38.3).
///
/// # Why this is an enclosure and not a comparison against zero
///
/// The three vectors here are the numbers the caller holds, and every product
/// and difference below is rounded. `‖outward[0] × outward[1]‖²` came out at
/// `2.55e-29` on a hemisphere sitting on a pipe of the same radius and very
/// nearly the same axis, out of gradients whose own components are of order one:
/// a wire that short is the cross product's *rounding* and not its value, and
/// the sign read off it was a coin toss dressed as a fact (§109.7, §111).
///
/// So the whole chain runs in interval arithmetic over the caller's numbers
/// taken exactly, and anything that enclosure cannot prove goes to
/// [`exactly_which_way_a_closed_crossing_turns`] below, which is the cascade's
/// third level and decides every one of these signs that exists (§122).
/// [`Decision::Undecidable`] is what is left when even that cannot run, and it
/// is a result here and not a failure — it is the caller that has to turn it
/// into one (§2.1, §4.1).
///
/// **What neither level encloses** is the error already in `outward`,
/// `gradient` and `axis`, which reached the caller as `f64` evaluations of
/// exact formulas and carry about an ulp each. Those are taken as exact, so
/// what is proved is a fact about the numbers handed over. Every certified
/// answer this gives is one the old comparison against `0.0` also gave; what has
/// gone is the answers it gave that its own arithmetic could not support.
pub fn which_way_a_closed_crossing_turns(
    outward: [Vector; 2],
    gradient: Vector,
    axis: Vector,
) -> Decision {
    let from_the_enclosure = enclosed_which_way_a_closed_crossing_turns(outward, gradient, axis);
    let Decision::Undecidable { .. } = from_the_enclosure else {
        return from_the_enclosure;
    };
    match exactly_which_way_a_closed_crossing_turns(outward, gradient, axis) {
        Some(settled @ (Decision::Pos | Decision::Neg | Decision::Zero)) => settled,
        _ => from_the_enclosure,
    }
}

/// The rule in interval arithmetic: cheap, and right whenever it commits.
fn enclosed_which_way_a_closed_crossing_turns(
    outward: [Vector; 2],
    gradient: Vector,
    axis: Vector,
) -> Decision {
    let walked = cross_enclosing(
        exactly(outward[0].components()),
        exactly(outward[1].components()),
    );
    let gradient = exactly(gradient.components());
    let longitude = cross_enclosing(exactly(axis.components()), gradient);
    let along_the_longitude = dot_enclosing(walked, longitude);
    if along_the_longitude.sign() == Decision::Zero {
        return dot_enclosing(walked, cross_enclosing(gradient, longitude)).sign();
    }
    along_the_longitude.sign()
}

/// The same rule with no rounding in it at all, or `None` when the exact
/// arithmetic cannot run on these numbers.
///
/// Both references are polynomials in the nine stored doubles — degree four
/// along the longitude and degree five along the latitude — so
/// [`Exact`] settles either sign whenever its products stay in the range where
/// they are exact. `None` means level 3 declined, not that the answer is zero,
/// and the caller keeps the enclosure's refusal rather than inventing one.
///
/// # This level picks the reference as well as the sign, and has to
///
/// The rule reads the latitude only where the longitude says *exactly nothing*,
/// so which reference answers **is** the question of whether the longitude's dot
/// is zero — and that is a sign, decided here and nowhere cheaper. An enclosure
/// that straddles zero has not said the dot is nonzero; it has said it does not
/// know. So this level cannot be handed a reference chosen upstream and asked
/// only to finish the sign: fed a longitude that is truly zero it would answer
/// [`Decision::Zero`], which claims the two surfaces touch rather than cross,
/// while the latitude was there with a direction all along. It re-asks both
/// questions from the top instead, which costs one extra exact dot product on a
/// path that already gave up once (§122).
///
/// # Why the latitude is written turned round
///
/// The rule's latitude is `walked · (gradient × longitude)`, and
/// [`Exact::cross_of`] takes the stored triple on the right, not the left. A
/// triple product is unchanged by cycling its three vectors, so the same number
/// is `longitude · (walked × gradient)` — which puts `gradient` where the stored
/// numbers go. Both forms are exact here, so this is spelling and not
/// arithmetic.
fn exactly_which_way_a_closed_crossing_turns(
    outward: [Vector; 2],
    gradient: Vector,
    axis: Vector,
) -> Option<Decision> {
    let gradient = gradient.components();
    let walked = Exact::cross_of(
        &Exact::of_each(outward[0].components()),
        outward[1].components(),
    )?;
    let longitude = Exact::cross_of(&Exact::of_each(axis.components()), gradient)?;
    let along_the_longitude = Exact::dot_of_two_exact(&walked, &longitude)?;
    if along_the_longitude.sign() != Decision::Zero {
        return Some(along_the_longitude.sign());
    }
    let walked_across_the_gradient = Exact::cross_of(&walked, gradient)?;
    Some(Exact::dot_of_two_exact(&longitude, &walked_across_the_gradient)?.sign())
}

#[cfg(test)]
mod tests {
    use rcad_predicates::Decision;

    use super::{
        enclosed_which_way_a_closed_crossing_turns, exactly_which_way_a_closed_crossing_turns,
        which_way_a_closed_crossing_turns,
    };
    use crate::point::Vector;

    /// A pipe crossing a ball squarely, which is as far from the hard cases as a
    /// closed crossing gets: the two answers have to be opposite ways round.
    #[test]
    fn a_crossing_with_room_in_it_is_read_the_same_both_ways_round() {
        let (round, straight) = (Vector::new(2.0, 0.0, 0.0), Vector::new(0.0, 0.0, 2.0));
        let axis = Vector::new(0.0, 0.0, 1.0);
        assert_eq!(
            which_way_a_closed_crossing_turns([round, straight], round, axis),
            Decision::Neg
        );
        assert_eq!(
            which_way_a_closed_crossing_turns([straight, round], round, axis),
            Decision::Pos
        );
    }

    /// Two faces whose gradients point the same way are not crossing, and a
    /// crossing that is not one has no direction to name.
    ///
    /// Halves and quarters, so that every product the cross product takes is
    /// exact and the zero it comes to is the shape's and not a rounding's. The
    /// same pair written as tenths comes back [`Decision::Undecidable`], which is
    /// also right: `0.2 × 0.6` and `0.3 × 0.4` are two different roundings of one
    /// number and nothing can tell which way their difference falls.
    #[test]
    fn two_surfaces_that_touch_rather_than_cross_have_no_direction() {
        let round = Vector::new(0.5, 0.25, 0.125);
        assert_eq!(
            which_way_a_closed_crossing_turns(
                [round, Vector::new(1.0, 0.5, 0.25)],
                round,
                Vector::new(0.0, 0.0, 1.0),
            ),
            Decision::Zero
        );
    }

    /// The wire is square to the longitude and the latitude finishes it.
    ///
    /// A ball of radius one cut by `x = 0.6` with the corner at `(0.6, 0.8, 0)`:
    /// the plane is parallel to the ball's axis and the corner is on its equator,
    /// which is the pair of coincidences that makes the dot against the longitude
    /// exactly zero (§36.18).
    #[test]
    fn a_wire_square_to_the_longitude_is_still_read_two_ways() {
        let round = Vector::new(0.6, 0.8, 0.0);
        let straight = Vector::new(1.0, 0.0, 0.0);
        let axis = Vector::new(0.0, 0.0, 1.0);
        assert_ne!(
            which_way_a_closed_crossing_turns([round, straight], round, axis),
            which_way_a_closed_crossing_turns([straight, round], round, axis)
        );
    }

    /// A crossing the second level cannot settle and the third one does.
    ///
    /// `(0.1, 0.2, 0.3)` and `(0.3, 0.6, 0.9)` are a real crossing a third of an
    /// `f64` step wide, not a rounding: the doublings inside them are exact but
    /// `0.3` is not `3 × 0.1`, so the cross product is genuinely nonzero and
    /// `f64` even computes it exactly. What stops the enclosure is that the `y`
    /// component's true value is one ulp of the two products behind it, and an
    /// interval holds each of those to half an ulp.
    ///
    /// **Refusing it was a gap and not a limit** (§16.10, §111.3, §122): the sign
    /// is a polynomial in the stored doubles, and the cascade's third level
    /// decides such a thing whenever its products stay exact.
    #[test]
    fn a_crossing_barely_one_f64_step_wide_is_settled_by_the_exact_level() {
        let round = Vector::new(0.1, 0.2, 0.3);
        let straight = Vector::new(0.3, 0.6, 0.9);
        assert_eq!(
            round.cross(straight).components(),
            [2.7755575615628914e-17, -1.3877787807814457e-17, 0.0],
            "f64 gets the exact cross product here, so the crossing is real"
        );
        assert!(
            matches!(
                enclosed_which_way_a_closed_crossing_turns(
                    [round, straight],
                    round,
                    Vector::new(1.0, 0.0, 0.0)
                ),
                Decision::Undecidable { .. }
            ),
            "the interval level is still the one that cannot tell"
        );
        assert_eq!(
            which_way_a_closed_crossing_turns([round, straight], round, Vector::new(1.0, 0.0, 0.0)),
            Decision::Pos
        );
    }

    /// The exact level is not allowed to quietly change an answer the enclosure
    /// had already certified.
    ///
    /// Run over every case above that the enclosure settles, the two levels have
    /// to agree — an enclosure is sound, so a level that disagreed with one would
    /// be the level with the bug.
    #[test]
    fn the_exact_level_agrees_with_every_answer_the_enclosure_certifies() {
        let up = Vector::new(0.0, 0.0, 1.0);
        for (outward, gradient, axis) in [
            (
                [Vector::new(2.0, 0.0, 0.0), Vector::new(0.0, 0.0, 2.0)],
                Vector::new(2.0, 0.0, 0.0),
                up,
            ),
            (
                [Vector::new(0.0, 0.0, 2.0), Vector::new(2.0, 0.0, 0.0)],
                Vector::new(2.0, 0.0, 0.0),
                up,
            ),
            (
                [Vector::new(0.5, 0.25, 0.125), Vector::new(1.0, 0.5, 0.25)],
                Vector::new(0.5, 0.25, 0.125),
                up,
            ),
            (
                [Vector::new(0.6, 0.8, 0.0), Vector::new(1.0, 0.0, 0.0)],
                Vector::new(0.6, 0.8, 0.0),
                up,
            ),
            (
                [Vector::new(1.0, 0.0, 0.0), Vector::new(0.6, 0.8, 0.0)],
                Vector::new(0.6, 0.8, 0.0),
                up,
            ),
        ] {
            let enclosed = enclosed_which_way_a_closed_crossing_turns(outward, gradient, axis);
            assert!(
                !matches!(enclosed, Decision::Undecidable { .. }),
                "this test only says something about the cases the enclosure settles"
            );
            assert_eq!(
                Some(enclosed),
                exactly_which_way_a_closed_crossing_turns(outward, gradient, axis),
                "the two levels disagree about {outward:?}, {gradient:?}, {axis:?}"
            );
        }
    }

    /// Coordinates too small for the exact level's products to stay exact, which
    /// is the one way the third level declines.
    ///
    /// Every component is scaled by `2^-600`, so each product of two of them is
    /// around `2^-1200` — far below the `2^-969` where a product's low half stops
    /// being a normal number. The enclosure cannot tell either, because the same
    /// products flush to zero, so the whole rule refuses and that refusal is a
    /// real one.
    #[test]
    fn a_crossing_written_far_too_small_to_multiply_is_still_refused() {
        let tiny = 2f64.powi(-600);
        let round = Vector::new(0.1 * tiny, 0.2 * tiny, 0.3 * tiny);
        let straight = Vector::new(0.3 * tiny, 0.6 * tiny, 0.9 * tiny);
        assert_eq!(
            exactly_which_way_a_closed_crossing_turns(
                [round, straight],
                round,
                Vector::new(1.0, 0.0, 0.0)
            ),
            None
        );
        assert!(matches!(
            which_way_a_closed_crossing_turns([round, straight], round, Vector::new(1.0, 0.0, 0.0)),
            Decision::Undecidable { .. }
        ));
    }
}
