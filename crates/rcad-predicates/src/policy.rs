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
    max_enclosure: f64,
    dynamic_range_check: Severity,
    symbolic_perturbation: bool,
}

impl Policy {
    /// Decide everything or say so. An undecided predicate stays undecided and
    /// the operation fails.
    pub fn certified() -> Self {
        Self {
            strictness: Strictness::Certified,
            max_enclosure: 0.0,
            dynamic_range_check: Severity::Fail,
            symbolic_perturbation: false,
        }
    }

    /// Allow a small amount of slop rather than failing.
    ///
    /// `scale` is the working size of the model — millimetres if the model is
    /// in millimetres.
    ///
    /// `orient3d` and friends are triple products, so their value is a signed
    /// volume and the budget has to be one too. The round-off nobody can avoid
    /// at this working size is about `7 * eps * scale³`, since that is what the
    /// level 1 error bound comes to when the permanent is `scale³`. The budget
    /// is 64 of those roundings, which leaves room for a decision that was
    /// only ever lost to rounding to come back, without letting real geometry
    /// through.
    ///
    /// A predicate of a different degree needs its enclosure scaled to match
    /// before it is compared against [`Strictness::TolerantFallback`]'s
    /// `max_slop`. Enclosure widths carry the units of whatever produced them.
    ///
    /// # Panics
    ///
    /// If `scale` is not a positive finite number.
    pub fn tolerant(scale: f64) -> Self {
        assert!(
            scale.is_finite() && scale > 0.0,
            "a tolerant policy needs a positive working size, got {scale}"
        );
        let max_slop = 64.0 * f64::EPSILON * scale * scale * scale;
        Self {
            strictness: Strictness::TolerantFallback { max_slop },
            max_enclosure: max_slop,
            dynamic_range_check: Severity::Warn,
            symbolic_perturbation: false,
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
    /// geometry as broken.
    ///
    /// Nothing in this crate reads this field — no predicate here judges a
    /// whole operation. It is carried on `Policy` for `rcad-topo` and the
    /// facade to read, since judging a whole operation is the operation's job
    /// (decisions.md §5).
    pub fn max_enclosure(self) -> f64 {
        self.max_enclosure
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

    /// Whether an enclosure this wide is within what the policy will report.
    pub fn enclosure_is_acceptable(self, enclosure: f64) -> bool {
        enclosure <= self.max_enclosure
    }
}

/// Whether an operation may trade certainty for an answer.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Strictness {
    /// Every decision must be proved. Undecided stays undecided.
    Certified,
    /// An undecided decision whose enclosure is no wider than `max_slop` may
    /// be read as zero, at the cost of an [`Approximation`] tag.
    TolerantFallback {
        /// The widest enclosure that may be read as zero, in the units of the
        /// predicate that produced it.
        max_slop: f64,
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
/// unchanged; under [`Strictness::TolerantFallback`] an enclosure no wider
/// than `max_slop` is read as [`Decision::Zero`] — the honest reading of "the
/// true value is somewhere in a band around zero this narrow" — and the record
/// picks up an [`Approximation`] for the caller to hang on the entity it
/// builds.
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
        Strictness::TolerantFallback { max_slop } => {
            if enclosure <= max_slop {
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
    use super::{Policy, Severity, Strictness, resolve_under_policy};
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
        assert_eq!(policy.max_enclosure(), 0.0);
        assert_eq!(policy.dynamic_range_check(), Severity::Fail);
        assert!(!policy.symbolic_perturbation());
        assert!(!policy.enclosure_is_acceptable(1e-30));
    }

    #[test]
    fn a_tolerant_policy_scales_its_slop_with_the_cube_of_the_model_size() {
        let small = Policy::tolerant(1.0);
        let large = Policy::tolerant(10.0);
        let (
            Strictness::TolerantFallback {
                max_slop: small_slop,
            },
            Strictness::TolerantFallback {
                max_slop: large_slop,
            },
        ) = (small.strictness(), large.strictness())
        else {
            panic!("tolerant policies must carry a slop budget");
        };
        assert_eq!(large_slop, small_slop * 1000.0);
        assert!(!large.symbolic_perturbation());
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
        assert_eq!(policy.max_enclosure(), 0.0);
        assert!(!policy.symbolic_perturbation());
    }

    #[test]
    fn a_certified_policy_leaves_an_undecided_answer_undecided_but_still_logs_it() {
        let mut log = ProvenanceLog::new();
        let outcome =
            resolve_under_policy(record_with_enclosure(1e-17), &Policy::certified(), &mut log);

        assert_eq!(outcome, Decision::Undecidable { enclosure: 1e-17 });
        assert_eq!(log.len(), 1);
        assert_eq!(log.records()[0].approximation(), None);
    }

    #[test]
    fn a_tolerant_policy_reads_a_narrow_enclosure_as_zero_and_tags_it() {
        let policy = Policy::tolerant(1.0);
        let mut log = ProvenanceLog::new();
        let outcome = resolve_under_policy(record_with_enclosure(1e-17), &policy, &mut log);

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
        let outcome = resolve_under_policy(record_with_enclosure(1.0), &policy, &mut log);

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
        let _ = resolve_under_policy(record, &Policy::certified(), &mut ProvenanceLog::new());
    }
}
