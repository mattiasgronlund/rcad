//! Which way the wire round a closed crossing turns, as a decision and not a
//! bare sign.

use rcad_predicates::Decision;

use crate::enclosure::{cross_enclosing, dot_enclosing, exactly};
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
/// taken exactly, and the answer is whatever that enclosure can prove.
/// [`Decision::Undecidable`] is a result here and not a failure — it is the
/// caller that has to turn it into one (§2.1, §4.1).
///
/// **What this still does not enclose** is the error already in `outward`,
/// `gradient` and `axis`, which reached the caller as `f64` evaluations of
/// exact formulas and carry about an ulp each. Those are taken as exact, so
/// what is proved is a fact about the numbers handed over. Every certified
/// answer this now gives is one the old comparison also gave; what has gone is
/// the answers it gave that its own arithmetic could not support.
pub fn which_way_a_closed_crossing_turns(
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

#[cfg(test)]
mod tests {
    use rcad_predicates::Decision;

    use super::which_way_a_closed_crossing_turns;
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

    /// The answer the old comparison against `0.0` gave and could not support.
    ///
    /// `0.2 × 0.9` and `0.3 × 0.6` are the same number and neither is exact, so
    /// the difference the cross product takes is the two roundings and nothing
    /// else. A sign read off that is a coin toss; the enclosure says so.
    #[test]
    fn a_direction_written_twice_over_leaves_a_wire_of_only_rounding() {
        let round = Vector::new(0.1, 0.2, 0.3);
        let straight = Vector::new(0.3, 0.6, 0.9);
        let walked = round.cross(straight);
        assert_ne!(
            walked,
            Vector::zero(),
            "the whole point is that f64 makes this look like a direction"
        );
        assert!(matches!(
            which_way_a_closed_crossing_turns([round, straight], round, Vector::new(1.0, 0.0, 0.0)),
            Decision::Undecidable { .. }
        ));
    }
}
