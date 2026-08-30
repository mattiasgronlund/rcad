use crate::{Approximation, Cause, Decision, ProvenanceLog, Record};

/// How strict an operation must be about the decisions it makes.
///
/// Travels as an explicit `&Policy` argument to everything that can make a
/// decision. Never a field on a model and never thread-local: model state that
/// silently changes results is the footgun this kernel exists to avoid
/// (decisions.md §2.6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Policy {
    strictness: Strictness,
    dynamic_range_check: Severity,
    symbolic_perturbation: bool,
}

impl Policy {
    /// Decide everything or say so. An undecided predicate stays undecided and
    /// the operation fails.
    pub fn certified() -> Self {
        Self {
            strictness: Strictness::Certified,
            dynamic_range_check: Severity::Fail,
            symbolic_perturbation: false,
        }
    }

    /// Allow a small amount of slop rather than failing.
    ///
    /// `scale` is the working size of the model — millimetres if the model is
    /// in millimetres. It is kept as it was handed over, and every budget is
    /// worked out from it on demand by [`Policy::max_slop`], because what a
    /// budget comes to depends on what the predicate asking for it measures.
    ///
    /// # Panics
    ///
    /// If `scale` is not a positive finite number.
    pub fn tolerant(scale: f64) -> Self {
        assert!(
            scale.is_finite() && scale > 0.0,
            "a tolerant policy needs a positive working size, got {scale}"
        );
        Self {
            strictness: Strictness::TolerantFallback {
                working_size: scale,
            },
            dynamic_range_check: Severity::Warn,
            symbolic_perturbation: false,
        }
    }

    /// The widest enclosure that may be read as zero, in the units of a
    /// predicate that measures `measures`. Always zero under
    /// [`Strictness::Certified`].
    ///
    /// The budget is 64 roundings at the working size, and a rounding at the
    /// working size is a different number for every degree: `eps * scale` for a
    /// length, `eps * scale²` for an area, `eps * scale³` for a volume. That is
    /// why the caller has to say which.
    ///
    /// ```
    /// use rcad_predicates::{Measures, Policy};
    ///
    /// // At a 100 mm working size an area budget and a volume budget are a
    /// // hundredfold apart. Judging one against the other is the units bug
    /// // this argument exists to prevent.
    /// let policy = Policy::tolerant(100.0);
    /// assert_eq!(
    ///     policy.max_slop(Measures::Volume),
    ///     policy.max_slop(Measures::Area) * 100.0
    /// );
    /// ```
    #[must_use]
    pub fn max_slop(self, measures: Measures) -> f64 {
        match self.strictness {
            Strictness::Certified => 0.0,
            Strictness::TolerantFallback { working_size } => {
                64.0 * f64::EPSILON * measures.at(working_size)
            }
        }
    }

    /// Replace the severity [`Policy::dynamic_range_check`] reports.
    ///
    /// A consuming, named method rather than a setter: turning off the check
    /// that catches a model too wide for `f64` (decisions.md §3.5) should read
    /// as a deliberate choice at the call site, not a field a caller reaches
    /// for out of habit.
    ///
    /// ```
    /// use rcad_predicates::{Policy, Severity};
    ///
    /// let policy = Policy::certified().with_dynamic_range_check(Severity::Ignore);
    /// assert_eq!(policy.dynamic_range_check(), Severity::Ignore);
    /// ```
    #[must_use]
    pub fn with_dynamic_range_check(mut self, severity: Severity) -> Self {
        self.dynamic_range_check = severity;
        self
    }

    /// Whether undecided predicates are allowed to be resolved.
    pub fn strictness(self) -> Strictness {
        self.strictness
    }

    /// The widest enclosure an operation may report before treating the
    /// geometry as broken, in the units of a predicate that measures
    /// `measures`.
    ///
    /// Same number as [`Policy::max_slop`], and a different question: that one
    /// asks whether a single undecided predicate may be read as zero, this one
    /// whether a whole operation's worst enclosure is small enough to trust the
    /// solid it built. It takes `measures` for the same reason — an enclosure
    /// carries the units of whatever produced it.
    ///
    /// Nothing in this crate reads it. No predicate here judges a whole
    /// operation; it is carried on `Policy` for `rcad-topo` and the facade to
    /// read, since judging a whole operation is the operation's job
    /// (decisions.md §5).
    #[must_use]
    pub fn max_enclosure(self, measures: Measures) -> f64 {
        self.max_slop(measures)
    }

    /// What to do when a model's coordinate range is too wide to work in
    /// (decisions.md §3.5).
    ///
    /// Nothing in this crate reads this field either — the dynamic range
    /// check itself lives in `rcad-geom`, next to the coordinates it
    /// measures. `Policy` only carries the caller's answer for that check
    /// (and `rcad-topo`'s validity check) to read.
    pub fn dynamic_range_check(self) -> Severity {
        self.dynamic_range_check
    }

    /// Always `false`.
    ///
    /// Simulation of Simplicity always produces a coherent answer, which
    /// competes head-on with returning [`Decision::Undecidable`]. Shipping
    /// both before a caller exists to say which one they want means shipping
    /// two answers to the same question (decisions.md §1.5). The knob is
    /// carried so the decision stays visible; there is deliberately no way to
    /// turn it on.
    pub fn symbolic_perturbation(self) -> bool {
        self.symbolic_perturbation
    }

    /// Whether an enclosure this wide, from a predicate measuring `measures`,
    /// is within what the policy will report.
    #[must_use]
    pub fn enclosure_is_acceptable(self, enclosure: f64, measures: Measures) -> bool {
        enclosure <= self.max_enclosure(measures)
    }
}

/// What a predicate's value measures, which is what says how a slop budget
/// turns into a number it can be compared against.
///
/// A predicate's value is a polynomial in coordinates, and its degree fixes its
/// units. `side of plane` is `normal · point - offset`: one coordinate deep, so
/// a length. `orient2d` is twice a triangle's area, and the sphere and cylinder
/// predicates are a squared distance, so those are areas. `orient3d` is six
/// times a tetrahedron's volume.
///
/// This is the difference between "how far out of place may a point be", which
/// is what a tolerance means to the person who set it, and the number a
/// predicate hands back. A wobble of `w` at working size `s` shows up as about
/// `w * s^(degree - 1)` in the value, so a budget for one degree says nothing
/// about another: at a 100 mm working size an area and a volume budget are a
/// hundredfold apart.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Measures {
    /// A distance. `side of plane`.
    Length,
    /// An area, or any squared distance. `orient2d`, `side of sphere`,
    /// `side of cylinder`.
    Area,
    /// A volume. `orient3d`.
    Volume,
}

impl Measures {
    /// The working size raised to as many dimensions as this measures.
    fn at(self, working_size: f64) -> f64 {
        match self {
            Measures::Length => working_size,
            Measures::Area => working_size * working_size,
            Measures::Volume => working_size * working_size * working_size,
        }
    }
}

/// Whether an operation may trade certainty for an answer.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Strictness {
    /// Every decision must be proved. Undecided stays undecided.
    Certified,
    /// An undecided decision narrow enough for the working size may be read as
    /// zero, at the cost of an [`Approximation`] tag.
    TolerantFallback {
        /// The working size of the model, in the model's own units.
        ///
        /// Not a budget: a budget is this raised into whatever the predicate
        /// measures, which is [`Policy::max_slop`]'s job. Keeping the raw size
        /// here is what stops a volume budget being compared against an area.
        working_size: f64,
    },
}

/// How loudly to complain about something.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// Say nothing.
    Ignore,
    /// Record it and carry on.
    Warn,
    /// Refuse to continue.
    Fail,
}

/// Apply a policy to a predicate result that was not certified, and log it.
///
/// This is the tolerant half of the two entry points decisions.md §2.1 asks
/// for. Under [`Strictness::Certified`] the answer comes back undecided
/// unchanged; under [`Strictness::TolerantFallback`] a narrow enough enclosure
/// is read as [`Decision::Zero`] — the honest reading of "the true value is
/// somewhere in a band around zero this narrow" — and the record picks up an
/// [`Approximation`] for the caller to hang on the entity it builds.
///
/// `measures` is what the predicate's value is a measure of. An enclosure
/// carries the units of whatever produced it, so without it there is no saying
/// whether the enclosure is inside the budget or a hundred times outside it.
///
/// Either way the record lands in `log`, because it is only ever called on a
/// non-certified outcome.
///
/// # Panics
///
/// If `record`'s outcome is already certified. Certified decisions are not
/// recorded (decisions.md §5), so calling this on one is a bug in the caller.
pub fn resolve_under_policy(
    mut record: Record,
    measures: Measures,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Decision {
    let enclosure = match record.outcome() {
        Decision::Undecidable { enclosure } => enclosure,
        Decision::Neg | Decision::Zero | Decision::Pos => panic!(
            "{}: a certified decision does not go through a policy",
            record.predicate()
        ),
    };

    let outcome = match policy.strictness() {
        Strictness::Certified => Decision::Undecidable { enclosure },
        Strictness::TolerantFallback { .. } => {
            if enclosure <= policy.max_slop(measures) {
                record.resolve_to(
                    Decision::Zero,
                    Approximation::new(enclosure, Cause::UndecidableResolved),
                );
                Decision::Zero
            } else {
                Decision::Undecidable { enclosure }
            }
        }
    };

    log.push(record);
    outcome
}

#[cfg(test)]
mod tests {
    use super::{Measures, Policy, Severity, Strictness, resolve_under_policy};
    use crate::{Cause, Decision, Level, Point, ProvenanceLog, Record};

    fn record_with_enclosure(enclosure: f64) -> Record {
        Record::new(
            "orient3d",
            &[Point::from([0.0, 0.0, 0.0])],
            Level::Interval,
            Decision::Undecidable { enclosure },
        )
    }

    #[test]
    fn a_certified_policy_refuses_every_kind_of_slop() {
        let policy = Policy::certified();
        assert_eq!(policy.strictness(), Strictness::Certified);
        assert_eq!(policy.max_enclosure(Measures::Volume), 0.0);
        assert_eq!(policy.dynamic_range_check(), Severity::Fail);
        assert!(!policy.symbolic_perturbation());
        assert!(!policy.enclosure_is_acceptable(1e-30, Measures::Volume));
    }

    #[test]
    fn a_tolerant_policy_keeps_the_working_size_it_was_given() {
        let policy = Policy::tolerant(10.0);
        let Strictness::TolerantFallback { working_size } = policy.strictness() else {
            panic!("a tolerant policy must carry a working size");
        };
        assert_eq!(working_size, 10.0);
        assert!(!policy.symbolic_perturbation());
    }

    /// The budget for a volume grows as the cube of the working size, for an
    /// area as the square, and for a length in step with it. Comparing an
    /// enclosure against the budget for the wrong degree is off by a whole
    /// factor of the working size for every degree of difference.
    #[test]
    fn each_degree_gets_its_own_budget() {
        let small = Policy::tolerant(1.0);
        let large = Policy::tolerant(10.0);

        assert_eq!(
            large.max_slop(Measures::Length),
            small.max_slop(Measures::Length) * 10.0
        );
        assert_eq!(
            large.max_slop(Measures::Area),
            small.max_slop(Measures::Area) * 100.0
        );
        assert_eq!(
            large.max_slop(Measures::Volume),
            small.max_slop(Measures::Volume) * 1000.0
        );

        // At a working size of one every degree comes to the same number, which
        // is exactly why a units bug of this shape hides in unit-sized tests.
        assert_eq!(
            small.max_slop(Measures::Length),
            small.max_slop(Measures::Volume)
        );
    }

    /// The bug this argument exists to stop: an enclosure that is far too wide
    /// to be read as zero for the area predicate that produced it would sail
    /// through if it were judged against a volume budget.
    #[test]
    fn an_area_judged_against_a_volume_budget_is_waved_through_when_it_should_not_be() {
        let policy = Policy::tolerant(100.0);
        let too_wide_for_an_area = policy.max_slop(Measures::Area) * 50.0;
        assert!(too_wide_for_an_area < policy.max_slop(Measures::Volume));

        let mut log = ProvenanceLog::new();
        let honest = resolve_under_policy(
            record_with_enclosure(too_wide_for_an_area),
            Measures::Area,
            &policy,
            &mut log,
        );
        assert_eq!(
            honest,
            Decision::Undecidable {
                enclosure: too_wide_for_an_area
            }
        );

        let with_the_wrong_units = resolve_under_policy(
            record_with_enclosure(too_wide_for_an_area),
            Measures::Volume,
            &policy,
            &mut log,
        );
        assert_eq!(with_the_wrong_units, Decision::Zero);
    }

    #[test]
    #[should_panic(expected = "positive working size")]
    fn a_tolerant_policy_needs_a_real_working_size() {
        let _ = Policy::tolerant(0.0);
    }

    #[test]
    fn with_dynamic_range_check_replaces_only_that_one_field() {
        let policy = Policy::certified().with_dynamic_range_check(Severity::Ignore);

        assert_eq!(policy.dynamic_range_check(), Severity::Ignore);
        assert_eq!(policy.strictness(), Strictness::Certified);
        assert_eq!(policy.max_enclosure(Measures::Volume), 0.0);
        assert!(!policy.symbolic_perturbation());
    }

    #[test]
    fn a_certified_policy_leaves_an_undecided_answer_undecided_but_still_logs_it() {
        let mut log = ProvenanceLog::new();
        let outcome = resolve_under_policy(
            record_with_enclosure(1e-17),
            Measures::Volume,
            &Policy::certified(),
            &mut log,
        );

        assert_eq!(outcome, Decision::Undecidable { enclosure: 1e-17 });
        assert_eq!(log.len(), 1);
        assert_eq!(log.records()[0].approximation(), None);
    }

    #[test]
    fn a_tolerant_policy_reads_a_narrow_enclosure_as_zero_and_tags_it() {
        let policy = Policy::tolerant(1.0);
        let mut log = ProvenanceLog::new();
        let outcome = resolve_under_policy(
            record_with_enclosure(1e-17),
            Measures::Volume,
            &policy,
            &mut log,
        );

        assert_eq!(outcome, Decision::Zero);
        let tag = log.records()[0]
            .approximation()
            .expect("a resolved record carries an approximation");
        assert_eq!(tag.magnitude(), 1e-17);
        assert_eq!(tag.cause(), Cause::UndecidableResolved);
        assert_eq!(log.records()[0].enclosure(), 1e-17);
    }

    #[test]
    fn a_tolerant_policy_still_refuses_an_enclosure_wider_than_its_budget() {
        let policy = Policy::tolerant(1.0);
        let mut log = ProvenanceLog::new();
        let outcome = resolve_under_policy(
            record_with_enclosure(1.0),
            Measures::Volume,
            &policy,
            &mut log,
        );

        assert_eq!(outcome, Decision::Undecidable { enclosure: 1.0 });
        assert_eq!(log.records()[0].approximation(), None);
    }

    #[test]
    #[should_panic(expected = "does not go through a policy")]
    fn a_certified_decision_may_not_be_run_through_a_policy() {
        let mut record = record_with_enclosure(1e-17);
        record.resolve_to(
            Decision::Zero,
            crate::Approximation::new(1e-17, Cause::UndecidableResolved),
        );
        let _ = resolve_under_policy(
            record,
            Measures::Volume,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
    }
}
