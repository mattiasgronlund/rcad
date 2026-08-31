use crate::{Decision, Point};

/// What a predicate answers when its input is not made of real numbers.
///
/// An enclosure of infinity, because infinity is how much is not known. It is
/// the same answer [`expansion_sign`](crate::expansion::expansion_sign) already
/// gives when an expansion stops being finite, so a refusal from the guard and
/// a refusal from level 3 look alike to a caller.
pub const NOTHING_IS_KNOWN: Decision = Decision::Undecidable {
    enclosure: f64::INFINITY,
};

/// Whether a point is a point at all, and so is somewhere a predicate can place
/// it.
///
/// Every predicate here asks this about every point it was handed, at its
/// public entry point and before the cascade runs, and answers
/// [`NOTHING_IS_KNOWN`] when it comes back false. [`are_real_numbers`] says why
/// the check has to be there.
///
/// **A constructed point always passes, and that is not an oversight.** Its box
/// is made of [`Interval`](crate::Interval)s, and every interval stands for a
/// non-empty set of real numbers — [`Interval::new`] refuses a not-a-number
/// bound, refuses `lo` above `hi` and refuses the degenerate `[inf, inf]`, and
/// [`Interval::sign`] already rests on nothing in the public API being able to
/// produce an empty one. So a certified sign over a constructed box, even one
/// as wide as [`Interval::ENTIRE`], is a true statement about every point in
/// that box. That is a theorem and worth keeping. An exact point holding a
/// not-a-number contains no points at all, and there is nothing to be said
/// about it.
///
/// [`Interval::new`]: crate::Interval::new
/// [`Interval::sign`]: crate::Interval::sign
/// [`Interval::ENTIRE`]: crate::Interval::ENTIRE
///
/// ```
/// # use rcad_predicates::{is_a_real_point, Interval, Point};
/// assert!(is_a_real_point(Point::from([1.0, 2.0, 3.0])));
/// assert!(!is_a_real_point(Point::from([f64::NAN, 2.0, 3.0])));
/// assert!(!is_a_real_point(Point::from([f64::INFINITY, 2.0, 3.0])));
/// assert!(is_a_real_point(Point::Constructed([Interval::ENTIRE; 3])));
/// ```
#[must_use]
pub fn is_a_real_point(point: Point) -> bool {
    match point {
        Point::Exact(coordinates) => are_real_numbers(coordinates),
        Point::Constructed(_) => true,
    }
}

/// Whether every one of these is a real number, and so is something a predicate
/// can work with.
///
/// The guard has to exist because a not-a-number does not stay unknown. It
/// reaches level 2 as [`Interval::ENTIRE`](crate::Interval::ENTIRE), and
/// `ENTIRE` times `[0, 0]` is `[0, 0]` — the unknown is annihilated, drops out
/// of the sum, and whatever is left certifies a sign as though the coordinate
/// had never been there. `orient3d` on `(NaN, 1, 0)`, `(1, 0, 1)`, `(2, 0, 3)`,
/// `(0, 0, 0)` came back [`Decision::Neg`] at
/// [`Level::Interval`](crate::Level::Interval) with nothing recorded. That sign
/// is true of every real the coordinate could have stood for, so it is not
/// unsound — it is a confident answer about a point that is not a point, and a
/// kernel whose premise is "prove it or admit you cannot" does not give those.
///
/// Surface parameters come through here too, not only coordinates.
/// `Sphere::transformed`, `Cylinder::transformed` and `Plane::transformed`
/// cannot fail and can overflow, so a surface built from finite numbers can end
/// up holding an infinite one — and a cylinder whose axis origin overflowed
/// certified [`Decision::Pos`] about a perfectly ordinary point.
///
/// ```
/// # use rcad_predicates::are_real_numbers;
/// assert!(are_real_numbers([1.0, -2.5, 0.0]));
/// assert!(!are_real_numbers([1.0, f64::NAN]));
/// assert!(!are_real_numbers([f64::NEG_INFINITY]));
/// ```
#[must_use]
pub fn are_real_numbers(numbers: impl IntoIterator<Item = f64>) -> bool {
    numbers.into_iter().all(f64::is_finite)
}

#[cfg(test)]
mod tests {
    use super::{NOTHING_IS_KNOWN, are_real_numbers, is_a_real_point};
    use crate::{Decision, Interval, Point};

    #[test]
    fn a_refusal_is_never_mistaken_for_an_answer() {
        assert!(!NOTHING_IS_KNOWN.is_certified());
        assert_eq!(
            NOTHING_IS_KNOWN,
            Decision::Undecidable {
                enclosure: f64::INFINITY
            }
        );
    }

    #[test]
    fn an_exact_point_needs_every_coordinate_to_be_a_real_number() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut coordinates = [1.0, 2.0, 3.0];
                coordinates[axis] = bad;
                assert!(!is_a_real_point(Point::from(coordinates)));
            }
        }
        assert!(is_a_real_point(Point::from([1.0, 2.0, 3.0])));
    }

    /// A box is a set of real numbers however wide it is, so there is nothing
    /// here to refuse. Only an exact point can fail to be a point.
    #[test]
    fn a_constructed_point_is_always_a_real_point() {
        assert!(is_a_real_point(Point::Constructed([Interval::ENTIRE; 3])));
        assert!(is_a_real_point(Point::Constructed([
            Interval::new(1.0, f64::INFINITY).unwrap(),
            Interval::new(f64::NEG_INFINITY, 1.0).unwrap(),
            Interval::exactly(0.0),
        ])));
    }

    /// The claim the line above rests on: nothing the public API can build
    /// stands for anything other than a non-empty set of reals.
    #[test]
    fn an_interval_that_is_not_a_set_of_reals_cannot_be_built() {
        for bounds in [
            (f64::NAN, f64::NAN),
            (f64::INFINITY, f64::INFINITY),
            (f64::NEG_INFINITY, f64::NEG_INFINITY),
            (1.0, 0.0),
        ] {
            assert!(Interval::new(bounds.0, bounds.1).is_err());
        }
    }

    #[test]
    fn an_empty_list_of_numbers_has_nothing_wrong_with_it() {
        assert!(are_real_numbers([]));
    }
}
