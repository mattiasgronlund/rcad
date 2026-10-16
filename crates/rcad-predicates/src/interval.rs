use core::fmt;
use core::ops::{Add, Mul, Neg, Sub};

use crate::{Decision, Error};

/// A range of `f64` that is guaranteed to contain the true value.
///
/// This is level 2 of the cascade. `inari` does the directed rounding; wrapping
/// it here means a future port is a bounded job on one file rather than a
/// rewrite, and it keeps `inari` out of every signature in the workspace
/// (decisions.md §2.4).
#[derive(Clone, Copy, PartialEq)]
pub struct Interval(inari::Interval);

impl Interval {
    /// Every real number. What we fall back to when we know nothing at all.
    pub const ENTIRE: Self = Self(inari::Interval::ENTIRE);

    /// The single value `value`.
    ///
    /// A value that is not finite is not a point on the real line, so it gets
    /// [`Interval::ENTIRE`], which keeps the hot path free of a `Result` and is
    /// sound as an *enclosure*: everything is a superset of anything.
    ///
    /// This used to add "an enclosure of everything can never certify a sign",
    /// and that does not follow. A coefficient of zero annihilates it —
    /// `ENTIRE * [0, 0]` is `[0, 0]` — so the unknown drops out of the sum and
    /// the rest of the expression certifies a sign as though the not-a-number
    /// had never been passed in. Sound, and still a confident answer about a
    /// point that is not a point.
    /// [`is_a_real_point`](crate::is_a_real_point) is what stops that now, at
    /// each predicate's entry point and before the cascade runs.
    ///
    /// ```
    /// # use rcad_predicates::Interval;
    /// assert_eq!(Interval::exactly(2.5).lo(), 2.5);
    /// assert_eq!(Interval::exactly(2.5).width(), 0.0);
    /// assert_eq!(Interval::exactly(f64::NAN), Interval::ENTIRE);
    /// ```
    pub fn exactly(value: f64) -> Self {
        match inari::Interval::try_from((value, value)) {
            Ok(interval) => Self(interval),
            Err(_) => Self::ENTIRE,
        }
    }

    /// The range from `lo` to `hi`.
    ///
    /// # Errors
    ///
    /// [`Error::NotAnInterval`] when `lo` is above `hi`, when either bound is
    /// not a number, or when the range would be empty.
    pub fn new(lo: f64, hi: f64) -> Result<Self, Error> {
        inari::Interval::try_from((lo, hi))
            .map(Self)
            .map_err(|_| Error::NotAnInterval { lo, hi })
    }

    /// The range `centre` give or take `reach`, rounded **outward**.
    ///
    /// This and not [`Interval::new`] whenever the range is a box some point is
    /// known to be inside. `centre - reach` and `centre + reach` round to
    /// nearest, and nearest rounds *inward*: each end lands up to half an ulp
    /// short of where it was asked for. Where `reach` is under half an ulp of
    /// `centre` the two ends land back on `centre` and the box collapses to the
    /// single point the caller was saying it did not have. A predicate handed
    /// that box certifies a sign about somewhere the true point need not be.
    ///
    /// That is not hypothetical: a pipe of radius 1.98 sixteen million units
    /// out is where the nightly fuzzer found it (`docs/decisions.md` §85).
    ///
    /// ```
    /// # use rcad_predicates::Interval;
    /// let far_out = 16252928.89923649;
    /// let hair = 4.895112993500317e-10;
    /// assert_eq!(far_out - hair, far_out, "a hair is under half an ulp out here");
    /// assert!(Interval::around(far_out, hair).unwrap().width() > 0.0);
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::NotAnInterval`] when `reach` is below zero or is not a number,
    /// and when `centre` is not finite. A box round a place that is not a place
    /// is not a box, and the alternative — [`Interval::ENTIRE`], which
    /// [`Interval::exactly`] hands back for the same input — would turn every
    /// caller that refuses one today into one that carries on knowing nothing.
    pub fn around(centre: f64, reach: f64) -> Result<Self, Error> {
        if !centre.is_finite() {
            return Err(Error::NotAnInterval {
                lo: centre - reach,
                hi: centre + reach,
            });
        }
        Ok(Self::exactly(centre) + Self::new(-reach, reach)?)
    }

    /// The lower bound.
    pub fn lo(self) -> f64 {
        self.0.inf()
    }

    /// The upper bound.
    pub fn hi(self) -> f64 {
        self.0.sup()
    }

    /// How far the upper bound is above the lower bound.
    pub fn width(self) -> f64 {
        self.0.wid()
    }

    /// Whether zero is somewhere inside, endpoints included.
    pub fn contains_zero(self) -> bool {
        self.0.contains(0.0)
    }

    /// The sign of everything inside, or [`Decision::Undecidable`] when the
    /// range straddles zero.
    ///
    /// # Panics
    ///
    /// If the interval is empty. Nothing in the public API can produce one, so
    /// an empty interval here means a bug in this crate.
    pub fn sign(self) -> Decision {
        assert!(
            !self.0.is_empty(),
            "an empty interval encloses nothing, so it has no sign"
        );
        let (lo, hi) = (self.lo(), self.hi());
        if lo > 0.0 {
            Decision::Pos
        } else if hi < 0.0 {
            Decision::Neg
        } else if lo == 0.0 && hi == 0.0 {
            Decision::Zero
        } else {
            Decision::Undecidable {
                enclosure: self.width(),
            }
        }
    }

    /// The square root of every number in the range that is not below zero, or
    /// `None` when every number in it is.
    ///
    /// Each end is rounded outwards by the hardware's own directed rounding,
    /// so no range of two `f64`s that holds the root is narrower: a square
    /// whose root is an `f64` comes back as that `f64` and nothing wider, and
    /// `√2` one step of an `f64` wide. That holds down into the subnormals,
    /// where squaring a root back to check which way it rounded no longer can.
    ///
    /// A range that dips below zero starts its root at zero. A sum of squares
    /// that rounded a hair below zero is that rounding and not a negative
    /// number.
    ///
    /// A range wholly below zero is `None` and not a panic, because whether
    /// one is a bug is the caller's to say. `inari` hands back an empty range
    /// there, and an empty range is what [`Interval::sign`] refuses.
    ///
    /// ```
    /// # use rcad_predicates::Interval;
    /// let root = |lo: f64, hi: f64| Interval::new(lo, hi).unwrap().sqrt();
    /// let four = root(4.0, 4.0).unwrap();
    /// assert_eq!((four.lo(), four.hi()), (2.0, 2.0));
    /// let two = root(2.0, 2.0).unwrap();
    /// assert_eq!(two.hi(), 2.0_f64.sqrt(), "which rounds √2 up");
    /// assert_eq!(two.lo(), 2.0_f64.sqrt().next_down());
    /// assert_eq!(root(-1.0, 4.0).unwrap().lo(), 0.0);
    /// assert_eq!(root(-4.0, -1.0), None);
    /// ```
    pub fn sqrt(self) -> Option<Self> {
        if self.hi() < 0.0 {
            return None;
        }
        Some(Self(self.0.sqrt()))
    }

    /// `self` divided by `divisor`.
    ///
    /// Kept out of the public API: dividing by a range that touches zero gives
    /// an empty or unbounded result, and every caller in this crate checks the
    /// divisor first. Exposing it would make [`Interval::sign`]'s
    /// no-empty-interval invariant something a caller could break.
    pub(crate) fn divided_by(self, divisor: Self) -> Self {
        debug_assert!(
            !divisor.contains_zero(),
            "dividing by a range that touches zero"
        );
        Self(self.0 / divisor.0)
    }
}

/// The determinant of a three by three matrix given as rows.
///
/// Expanded along the third column, which is the arrangement Shewchuk's
/// `orient3d` uses, so level 2 and level 1 evaluate the same expression.
pub(crate) fn determinant(rows: [[Interval; 3]; 3]) -> Interval {
    let [r0, r1, r2] = rows;
    r0[2] * (r1[0] * r2[1] - r2[0] * r1[1])
        + r1[2] * (r2[0] * r0[1] - r0[0] * r2[1])
        + r2[2] * (r0[0] * r1[1] - r1[0] * r0[1])
}

impl Add for Interval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Interval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul for Interval {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl Neg for Interval {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str("[empty]");
        }
        write!(f, "[{:?}, {:?}]", self.lo(), self.hi())
    }
}

#[cfg(test)]
mod tests {
    use super::{Interval, determinant};
    use crate::{Decision, Error};

    #[test]
    fn a_bad_pair_of_bounds_is_rejected() {
        assert_eq!(
            Interval::new(1.0, 0.0),
            Err(Error::NotAnInterval { lo: 1.0, hi: 0.0 })
        );
        assert!(matches!(
            Interval::new(f64::NAN, 1.0),
            Err(Error::NotAnInterval { .. })
        ));
    }

    /// The pipe the nightly fuzzer found, in one line: sixteen million units
    /// out, one ulp is 1.86e-9, and the walk knew its point to 4.9e-10. Asking
    /// for that box with plain subtraction gives back no box at all.
    #[test]
    fn a_box_thinner_than_half_an_ulp_is_still_a_box() {
        let centre = 16252928.89923649;
        let reach = 4.895112993500317e-10;
        assert_eq!(
            (centre - reach, centre + reach),
            (centre, centre),
            "nearest rounding puts both ends back on the centre"
        );

        let around = Interval::around(centre, reach).unwrap();
        assert!(
            around.lo() < centre && centre < around.hi(),
            "the centre has to be strictly inside, or the box says nothing"
        );
        assert!(
            around.width() >= 2.0 * reach,
            "a box that is not at least as wide as it was asked for is not an enclosure"
        );
    }

    /// The three ways there is no box to hand back. The last one matters most:
    /// every caller of this refused a centre that is not a place before it was
    /// written, by way of [`Interval::new`], and none of them wants
    /// [`Interval::ENTIRE`] in its stead.
    #[test]
    fn a_box_round_nowhere_is_refused() {
        for (centre, reach) in [
            (1.0, -1.0),
            (1.0, f64::NAN),
            (f64::INFINITY, 1.0),
            (f64::NAN, 1.0),
        ] {
            assert!(
                matches!(
                    Interval::around(centre, reach),
                    Err(Error::NotAnInterval { .. })
                ),
                "a box {reach} round {centre} is not a box"
            );
            assert!(
                Interval::new(centre - reach, centre + reach).is_err(),
                "and the two subtractions this replaced refused it too, which is what \
                 makes swapping them in a safe change at every call site"
            );
        }
    }

    /// Nothing is widened that does not need to be: a reach that lands on the
    /// grid gives back exactly the two ends asked for, and no reach at all
    /// gives back the point.
    #[test]
    fn a_box_that_needs_no_rounding_is_not_widened() {
        let exact = Interval::around(4.0, 0.25).unwrap();
        assert_eq!((exact.lo(), exact.hi()), (3.75, 4.25));
        assert_eq!(Interval::around(4.0, 0.0).unwrap().width(), 0.0);
    }

    #[test]
    fn adding_three_tenths_widens_the_enclosure() {
        let tenth = Interval::exactly(0.1);
        let sum = tenth + tenth + tenth;
        assert!(sum.width() > 0.0, "the third addition has to round");
        assert!(sum.lo() <= 0.1 + 0.1 + 0.1 && 0.1 + 0.1 + 0.1 <= sum.hi());
        assert!(sum.hi() > 0.3, "three f64 tenths add up to more than 0.3");
    }

    #[test]
    fn a_range_that_straddles_zero_reports_its_width() {
        let straddling = Interval::new(-0.25, 0.75).unwrap();
        assert!(straddling.contains_zero());
        assert_eq!(straddling.sign(), Decision::Undecidable { enclosure: 1.0 });
    }

    #[test]
    fn a_range_that_avoids_zero_has_a_sign() {
        assert_eq!(Interval::new(1.0, 2.0).unwrap().sign(), Decision::Pos);
        assert_eq!(Interval::new(-2.0, -1.0).unwrap().sign(), Decision::Neg);
        assert_eq!(Interval::exactly(0.0).sign(), Decision::Zero);
    }

    #[test]
    fn a_range_touching_zero_from_one_side_is_still_undecided() {
        assert_eq!(
            Interval::new(0.0, 1.0).unwrap().sign(),
            Decision::Undecidable { enclosure: 1.0 }
        );
    }

    #[test]
    fn multiplying_by_a_range_around_zero_widens_both_ways() {
        let around_zero = Interval::new(-1.0, 2.0).unwrap();
        let product = around_zero * Interval::exactly(-3.0);
        assert_eq!(product.lo(), -6.0);
        assert_eq!(product.hi(), 3.0);
    }

    #[test]
    fn the_identity_matrix_has_determinant_one() {
        let one = Interval::exactly(1.0);
        let nil = Interval::exactly(0.0);
        let identity = [[one, nil, nil], [nil, one, nil], [nil, nil, one]];
        assert_eq!(determinant(identity), one);
    }

    #[test]
    fn swapping_two_rows_flips_the_determinant() {
        let value = |x: f64| Interval::exactly(x);
        let rows = [
            [value(1.0), value(2.0), value(3.0)],
            [value(0.0), value(4.0), value(5.0)],
            [value(1.0), value(0.0), value(6.0)],
        ];
        let swapped = [rows[1], rows[0], rows[2]];
        assert_eq!(determinant(rows).lo(), -determinant(swapped).lo());
    }

    /// The smallest subnormal is `2^-1074`, whose root `2^-537` is an `f64`,
    /// and twice it has a root that is not. Each comes back as tight as an
    /// `f64` range can be: no width at all, and one step.
    #[test]
    fn a_root_down_in_the_subnormals_is_as_tight_as_anywhere() {
        let smallest = f64::from_bits(1);
        let exact = Interval::exactly(smallest).sqrt().expect("a root");
        assert_eq!(
            (exact.lo(), exact.hi()),
            (2.0_f64.powi(-537), 2.0_f64.powi(-537))
        );

        let twice = Interval::exactly(2.0 * smallest).sqrt().expect("a root");
        assert_eq!(twice.hi(), twice.lo().next_up());
        let scale = 2.0_f64.powi(537);
        let (lo, hi) = (twice.lo() * scale, twice.hi() * scale);
        assert!(
            lo.mul_add(lo, -2.0) < 0.0 && hi.mul_add(hi, -2.0) > 0.0,
            "√2 · 2^-537 lies inside, once both are scaled back up exactly"
        );
    }

    #[test]
    fn a_range_wholly_below_zero_has_no_root() {
        assert_eq!(Interval::new(-4.0, -1.0).unwrap().sqrt(), None);
        assert_eq!(Interval::exactly(-0.0).sqrt(), Some(Interval::exactly(0.0)));
    }

    #[test]
    fn dividing_narrows_nothing_it_should_not() {
        let third = Interval::exactly(1.0).divided_by(Interval::exactly(3.0));
        assert!(third.lo() < third.hi(), "one third is not an f64");
        assert!(third.lo() < 1.0 / 3.0 || third.hi() > 1.0 / 3.0);
    }
}
