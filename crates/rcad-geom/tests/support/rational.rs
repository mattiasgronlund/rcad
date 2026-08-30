//! The de Boor recursion in arithmetic that does not round.
//!
//! Two suites need a curve's *true* point rather than another `f64` run at it:
//! `evaluating_a_spline_stays_inside_its_published_bound.rs`, which asks how
//! far one evaluation strayed, and
//! `splitting_a_curve_leaves_both_halves_meeting_at_one_point.rs`, which asks
//! whether the seam a split lands on is on the curve at all. Both questions
//! need an answer arrived at without rounding, and there is one of those here
//! rather than one in each file.
//!
//! Sharing it is safe in the way `docs/decisions.md` §13 rule 5 is about:
//! sharing a *generator* ties two suites to the same family of shapes, and this
//! is not a generator. What it is, is the one place the exact answer is worked
//! out — so a mistake in it shows up as both suites failing together, which is
//! louder than one of two copies drifting.
//!
//! It runs on `i128` and reduces by `gcd` at every step. Every operation is
//! checked, so a case that outgrew `i128` fails loudly rather than wrapping.
//! The answer to that is a smaller case, never a looser assertion — which is
//! why the callers here keep their knots, control coordinates and weights to
//! small whole numbers and read them at parameters that are exact `f64`
//! fractions.

use rcad_geom::BSplineCurve;

/// The homogeneous point at `at`, worked out exactly.
///
/// `span` is the span of the curve `at` falls in. What comes back is
/// `[w·x, w·y, w·z, w]` as exact rationals, so a coordinate of the point itself
/// is one of the first three divided by the fourth.
pub fn exact_de_boor(curve: &BSplineCurve, at: Exact, span: usize) -> [Exact; 4] {
    let degree = curve.degree();
    let knots: Vec<Exact> = curve
        .knots()
        .knots()
        .iter()
        .map(|knot| Exact::from_f64(*knot))
        .collect();
    let mut points: Vec<[Exact; 4]> = (0..=degree)
        .map(|j| {
            curve
                .homogeneous_control_point(j + span - degree)
                .map(Exact::from_f64)
        })
        .collect();

    let one = Exact::whole(1);
    for round in 1..=degree {
        for j in (round..=degree).rev() {
            let below = knots[j + span - degree];
            let above = knots[j + 1 + span - round];
            let share = at.minus(below).over(above.minus(below));
            let rest = one.minus(share);
            let blended: [Exact; 4] = core::array::from_fn(|axis| {
                rest.times(points[j - 1][axis])
                    .plus(share.times(points[j][axis]))
            });
            points[j] = blended;
        }
    }
    points[degree]
}

/// The true point of `curve` at `at`, one coordinate per axis, as exact
/// rationals.
///
/// # Panics
///
/// If `at` is not a parameter of the curve, or if the exact arithmetic outgrows
/// `i128` — both of which are the caller handing this a case it said it would
/// not.
pub fn the_true_point_at(curve: &BSplineCurve, at: f64) -> [Exact; 3] {
    assert!(
        curve.knots().contains(at),
        "{at} is not a parameter of this curve"
    );
    let homogeneous = exact_de_boor(
        curve,
        Exact::from_f64(at),
        curve.knots().span_containing(at),
    );
    let weight = homogeneous[3];
    core::array::from_fn(|axis| homogeneous[axis].over(weight))
}

/// A rational number on `i128`, always reduced and always with a positive
/// bottom.
///
/// Every operation is checked. A case that outgrows `i128` is a case to make
/// smaller, not an assertion to loosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exact {
    top: i128,
    bottom: i128,
}

impl Exact {
    /// A whole number.
    pub fn whole(value: i128) -> Self {
        Self {
            top: value,
            bottom: 1,
        }
    }

    /// The exact value of an `f64`: mantissa over a power of two.
    ///
    /// # Panics
    ///
    /// If the number is not finite, or is so large that its mantissa shifted up
    /// does not fit in an `i128`.
    pub fn from_f64(value: f64) -> Self {
        assert!(
            value.is_finite(),
            "{value} is not a number to be exact about"
        );
        if value == 0.0 {
            return Self::whole(0);
        }
        let bits = value.to_bits();
        let sign = if bits >> 63 == 1 { -1i128 } else { 1 };
        let raw_exponent = ((bits >> 52) & 0x7ff) as i32;
        let raw_mantissa = i128::from(bits & ((1u64 << 52) - 1));
        let (mantissa, exponent) = if raw_exponent == 0 {
            (raw_mantissa, -1074)
        } else {
            (raw_mantissa | (1i128 << 52), raw_exponent - 1075)
        };

        if exponent >= 0 {
            Self::new(sign * mantissa * two_to_the(exponent), 1)
        } else {
            Self::new(sign * mantissa, two_to_the(-exponent))
        }
    }

    /// The sum.
    pub fn plus(self, other: Self) -> Self {
        let left = checked(self.top, other.bottom, "adding");
        let right = checked(other.top, self.bottom, "adding");
        Self::new(
            left.checked_add(right).expect("a sum too big for i128"),
            checked(self.bottom, other.bottom, "adding"),
        )
    }

    /// The difference.
    pub fn minus(self, other: Self) -> Self {
        self.plus(Self {
            top: -other.top,
            bottom: other.bottom,
        })
    }

    /// The product.
    pub fn times(self, other: Self) -> Self {
        let left = Self::new(self.top, other.bottom);
        let right = Self::new(other.top, self.bottom);
        Self::new(
            checked(left.top, right.top, "multiplying"),
            checked(left.bottom, right.bottom, "multiplying"),
        )
    }

    /// The quotient.
    ///
    /// # Panics
    ///
    /// If `other` is zero.
    pub fn over(self, other: Self) -> Self {
        assert!(other.top != 0, "an exact division by zero");
        self.times(Self::new(other.bottom, other.top))
    }

    /// The magnitude.
    pub fn abs(self) -> Self {
        Self {
            top: self.top.abs(),
            bottom: self.bottom,
        }
    }

    /// The nearest `f64`, which is only ever used to compare a difference
    /// against a bound many orders of magnitude larger than the conversion.
    pub fn to_f64(self) -> f64 {
        self.top as f64 / self.bottom as f64
    }

    /// A rational from a top and a bottom, reduced.
    ///
    /// # Panics
    ///
    /// If the bottom is zero.
    fn new(top: i128, bottom: i128) -> Self {
        assert!(bottom != 0, "a rational cannot be over zero");
        let shared = gcd(top.unsigned_abs(), bottom.unsigned_abs()).max(1) as i128;
        let sign = if bottom < 0 { -1 } else { 1 };
        Self {
            top: sign * (top / shared),
            bottom: sign * (bottom / shared),
        }
    }
}

fn checked(left: i128, right: i128, what: &str) -> i128 {
    left.checked_mul(right)
        .unwrap_or_else(|| panic!("{what} {left} and {right} outgrew i128; make the case smaller"))
}

fn two_to_the(exponent: i32) -> i128 {
    assert!(
        (0..127).contains(&exponent),
        "2^{exponent} does not fit in an i128"
    );
    1i128 << exponent
}

fn gcd(left: u128, right: u128) -> u128 {
    if right == 0 {
        left
    } else {
        gcd(right, left % right)
    }
}
