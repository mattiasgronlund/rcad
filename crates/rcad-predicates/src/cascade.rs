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
    /// The most `value` can differ from the true value. The sign of `value` is
    /// certain exactly when its magnitude is above this.
    pub error_bound: f64,
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

    /// Level 1. `None` when an input is not an exact `f64`, because a forward
    /// error bound derived for exact inputs says nothing about uncertain ones.
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
/// happens, short of an intermediate overflow: level 3 always decides.
pub fn decide<P: Predicate + ?Sized>(predicate: &P) -> (Decision, Level) {
    if let Some(Filtered { value, error_bound }) = predicate.on_floats() {
        // A bound that is not a number means the predicate's own magnitudes
        // overflowed; that certifies nothing, and the comparisons below are
        // false, which is exactly right. A negative bound is a bug.
        assert!(
            error_bound.is_nan() || error_bound >= 0.0,
            "a forward error bound cannot be negative, got {error_bound}"
        );
        if value > error_bound {
            return (Decision::Pos, Level::Float);
        }
        if -value > error_bound {
            return (Decision::Neg, Level::Float);
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
    use super::{Filtered, Level, Predicate, decide};
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
