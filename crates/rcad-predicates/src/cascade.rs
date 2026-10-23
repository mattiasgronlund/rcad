use crate::{Decision, Interval};

/// Which tier of the cascade produced an answer.
///
/// Useful for tests and benchmarks that want to know the cheap filters are
/// doing their job. A test suite where everything falls through to
/// [`Level::Exact`] proves nothing about the two levels above it.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// Plain `f64` with a forward error bound.
    Float,
    /// Interval arithmetic.
    Interval,
    /// Exact expansion arithmetic.
    Exact,
}

/// A plain `f64` evaluation of a predicate and how far it can be from the
/// truth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Filtered {
    /// The value the predicate polynomial came to in `f64`.
    pub value: f64,
    /// The bound on the sum `value` is rounded from — not on `value` itself,
    /// which takes one rounding more and can land past this bound while the
    /// sign it settles stays right: on one tetrahedron, `orient3d.rs`'s
    /// `level_one_can_land_past_its_error_bound_and_still_certify_the_right_sign`
    /// has `value` land 8/7 of `error_bound` from the truth. What [`decide`]'s
    /// sign test needs is the weaker claim: rounding the sum to `value` cannot
    /// carry it past `error_bound` unless the sum was past it already, so
    /// `value > error_bound` still proves the sum, and so the true value,
    /// positive. That test also needs this to be a normal number, which is
    /// what [`decide`] checks before believing it (decisions.md §464.7).
    pub error_bound: f64,
}

/// Whether multiplying `a` by `b` keeps full relative accuracy.
///
/// Every level 1 error bound is a *relative* one — a small multiple of the
/// permanent — and the analysis behind it assumes each rounding costs at most a
/// fixed fraction of the number being rounded. For `+` and `-` that holds for
/// any two `f64`. For `*` it holds only while the product stays out of the
/// subnormals: down there products are rounded onto a fixed grid, so what a
/// product loses is an *absolute* amount, up to half of `2^-1074`, and that can
/// be the whole of its value. Feed such a product into a relative bound and the
/// filter certifies a sign it has not earned — sometimes the wrong one. That is
/// the failure the `robust` crate has, and the reason
/// [`Predicate::on_floats`] must return `None` when this comes back `false`.
///
/// A product of exactly zero only counts as accurate when one of the factors is
/// zero. Two small factors whose product falls off the bottom of the range also
/// give zero, and that zero is wrong by everything the product was worth — so
/// the factors are what gets asked about, not the answer.
///
/// [`product_is_exact`](crate::expansion::product_is_exact) is the stricter
/// test level 3 needs: exact arithmetic wants the *roundoff term* to be normal,
/// which starts a hundred binades higher up.
///
/// ```
/// # use rcad_predicates::product_is_normal;
/// assert!(product_is_normal(1.5, 2.5));
/// assert!(product_is_normal(0.0, 3.5e-323));
/// assert!(!product_is_normal(2f64.powi(-538), 2f64.powi(-538)));
/// ```
#[must_use]
pub fn product_is_normal(a: f64, b: f64) -> bool {
    a == 0.0 || b == 0.0 || (a * b).abs() >= f64::MIN_POSITIVE
}

/// The three ways to evaluate one predicate, cheapest first.
///
/// A new predicate joins the cascade by implementing this and calling
/// [`decide`]. There is no level logic to copy.
///
/// `rcad-geom`'s quadric predicates implement this too; the quadric types live
/// there, so the evaluations do as well (decisions.md §7.2).
pub trait Predicate {
    /// Name for provenance records. Plain words, not a type path.
    fn name(&self) -> &'static str;

    /// Level 1. `None` when this level cannot be trusted to run, which is two
    /// things:
    ///
    /// - an input is not an exact `f64`, because a forward error bound derived
    ///   for exact inputs says nothing about uncertain ones; and
    /// - one of the multiplications on the way to `value` or to the permanent
    ///   landed in the subnormals, because the bound is relative and down there
    ///   rounding stops being relative. Every implementation checks its own
    ///   products through [`product_is_normal`].
    fn on_floats(&self) -> Option<Filtered>;

    /// Level 2. A range that is guaranteed to contain the true value.
    fn on_intervals(&self) -> Interval;

    /// Level 3. Must return a certified sign, or `None` when it cannot run —
    /// because an input is not exact, or because exact arithmetic overflowed.
    /// Returning [`Decision::Undecidable`] from here is a bug.
    fn exactly(&self) -> Option<Decision>;
}

/// Run a predicate through the cascade: `f64` filter, then interval, then
/// exact.
///
/// Alongside the answer it reports which level settled it.
///
/// The one case that comes back [`Decision::Undecidable`] is an interval that
/// straddles zero when level 3 cannot run. For exact `f64` inputs that never
/// happens, short of an intermediate product overflowing or falling into the
/// subnormals: level 3 always decides.
pub fn decide<P: Predicate + ?Sized>(predicate: &P) -> (Decision, Level) {
    if let Some(Filtered { value, error_bound }) = predicate.on_floats() {
        assert!(
            error_bound.is_nan() || error_bound >= 0.0,
            "a forward error bound cannot be negative, got {error_bound}"
        );
        // The bound only bounds anything while it is a normal number of its
        // own. Infinite or not-a-number means the predicate's magnitudes
        // overflowed. Subnormal means the bound was itself rounded onto the
        // grid at the bottom of the range, which can shave far more off it than
        // the slack the constants carry, so the comparison would be against a
        // number smaller than the error it stands for. Zero says the same thing
        // in the limit. Levels 2 and 3 are built to say "cannot tell"; this
        // level is not, so it hands over.
        if error_bound.is_normal() {
            if value > error_bound {
                return (Decision::Pos, Level::Float);
            }
            if -value > error_bound {
                return (Decision::Neg, Level::Float);
            }
        }
    }

    let straddle = match predicate.on_intervals().sign() {
        Decision::Neg => return (Decision::Neg, Level::Interval),
        Decision::Zero => return (Decision::Zero, Level::Interval),
        Decision::Pos => return (Decision::Pos, Level::Interval),
        Decision::Undecidable { enclosure } => enclosure,
    };

    match predicate.exactly() {
        Some(Decision::Neg) => (Decision::Neg, Level::Exact),
        Some(Decision::Zero) => (Decision::Zero, Level::Exact),
        Some(Decision::Pos) => (Decision::Pos, Level::Exact),
        Some(Decision::Undecidable { .. }) => unreachable!(
            "{}: exact arithmetic must certify a sign or report that it cannot run",
            predicate.name()
        ),
        None => (
            Decision::Undecidable {
                enclosure: straddle,
            },
            Level::Interval,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{Filtered, Level, Predicate, decide, product_is_normal};
    use crate::{Decision, Interval};

    /// A predicate that answers whatever the test told it to, so the level
    /// logic can be exercised on its own.
    struct Scripted {
        floats: Option<Filtered>,
        interval: Interval,
        exact: Option<Decision>,
    }

    impl Predicate for Scripted {
        fn name(&self) -> &'static str {
            "scripted"
        }

        fn on_floats(&self) -> Option<Filtered> {
            self.floats
        }

        fn on_intervals(&self) -> Interval {
            self.interval
        }

        fn exactly(&self) -> Option<Decision> {
            self.exact
        }
    }

    #[test]
    fn a_value_clear_of_its_error_bound_stops_at_the_first_level() {
        let predicate = Scripted {
            floats: Some(Filtered {
                value: -5.0,
                error_bound: 1.0,
            }),
            interval: Interval::ENTIRE,
            exact: None,
        };
        assert_eq!(decide(&predicate), (Decision::Neg, Level::Float));
    }

    #[test]
    fn a_value_inside_its_error_bound_falls_through_to_the_interval() {
        let predicate = Scripted {
            floats: Some(Filtered {
                value: 0.5,
                error_bound: 1.0,
            }),
            interval: Interval::new(2.0, 3.0).unwrap(),
            exact: None,
        };
        assert_eq!(decide(&predicate), (Decision::Pos, Level::Interval));
    }

    #[test]
    fn a_straddling_interval_falls_through_to_exact_arithmetic() {
        let predicate = Scripted {
            floats: None,
            interval: Interval::new(-1.0, 1.0).unwrap(),
            exact: Some(Decision::Zero),
        };
        assert_eq!(decide(&predicate), (Decision::Zero, Level::Exact));
    }

    #[test]
    fn a_straddling_interval_with_no_exact_level_is_undecidable() {
        let predicate = Scripted {
            floats: None,
            interval: Interval::new(-0.25, 0.75).unwrap(),
            exact: None,
        };
        assert_eq!(
            decide(&predicate),
            (Decision::Undecidable { enclosure: 1.0 }, Level::Interval)
        );
    }

    #[test]
    #[should_panic(expected = "must certify a sign")]
    fn exact_arithmetic_may_not_answer_undecidable() {
        let predicate = Scripted {
            floats: None,
            interval: Interval::new(-1.0, 1.0).unwrap(),
            exact: Some(Decision::Undecidable { enclosure: 2.0 }),
        };
        let _ = decide(&predicate);
    }

    /// A bound this small has been rounded onto the grid at the bottom of the
    /// `f64` range, so it no longer stands for the error it was derived to
    /// stand for. Level 1 has to hand over even though the value clears it.
    #[test]
    fn a_bound_too_small_to_be_a_normal_number_certifies_nothing() {
        let predicate = Scripted {
            floats: Some(Filtered {
                value: 5e-322,
                error_bound: 1e-322,
            }),
            interval: Interval::new(-1.0, 1.0).unwrap(),
            exact: Some(Decision::Neg),
        };
        assert_eq!(decide(&predicate), (Decision::Neg, Level::Exact));
    }

    /// The same for a bound that came out exactly zero, which is what a
    /// permanent built entirely from underflowed products comes to.
    #[test]
    fn a_bound_of_zero_certifies_nothing() {
        let predicate = Scripted {
            floats: Some(Filtered {
                value: 5e-324,
                error_bound: 0.0,
            }),
            interval: Interval::new(-1.0, 1.0).unwrap(),
            exact: Some(Decision::Neg),
        };
        assert_eq!(decide(&predicate), (Decision::Neg, Level::Exact));
    }

    #[test]
    fn a_bound_that_overflowed_certifies_nothing() {
        for error_bound in [f64::INFINITY, f64::NAN] {
            let predicate = Scripted {
                floats: Some(Filtered {
                    value: 1e300,
                    error_bound,
                }),
                interval: Interval::new(-1.0, 1.0).unwrap(),
                exact: Some(Decision::Zero),
            };
            assert_eq!(decide(&predicate), (Decision::Zero, Level::Exact));
        }
    }

    #[test]
    fn a_product_that_stays_normal_is_accurate_and_one_that_underflows_is_not() {
        assert!(product_is_normal(1.5, 2.5));
        assert!(product_is_normal(f64::MIN_POSITIVE, 1.0));
        assert!(!product_is_normal(f64::MIN_POSITIVE, 0.5));
    }

    /// Two numbers that multiply out to less than the smallest `f64` give zero,
    /// and that zero is wrong by the whole of what the product was worth. Only
    /// a factor that is itself zero makes a trustworthy zero.
    #[test]
    fn a_product_that_underflows_all_the_way_to_zero_is_not_accurate() {
        let tiny = 2f64.powi(-700);
        assert_eq!(tiny * tiny, 0.0);
        assert!(!product_is_normal(tiny, tiny));
        assert!(product_is_normal(0.0, tiny));
        assert!(product_is_normal(tiny, 0.0));
    }

    #[test]
    #[should_panic(expected = "cannot be negative")]
    fn a_negative_error_bound_is_a_bug() {
        let predicate = Scripted {
            floats: Some(Filtered {
                value: 0.0,
                error_bound: -1.0,
            }),
            interval: Interval::ENTIRE,
            exact: None,
        };
        let _ = decide(&predicate);
    }
}
