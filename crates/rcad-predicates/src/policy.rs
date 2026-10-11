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
    /// worked out from it on demand by [`Policy::max_wobble`], and an
    /// enclosure is converted to compare against it by [`Measures::wobble_of`].
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

    /// How far out of place a decision may leave a point before this policy
    /// stops accepting it. Always zero under [`Strictness::Certified`].
    ///
    /// A length — 64 roundings at the working size — and there is only one of
    /// them, whatever predicate is asking. A predicate's own enclosure is not
    /// comparable against this until [`Measures::wobble_of`] has turned it into
    /// a wobble, and that conversion is the only place the predicate's degree
    /// comes into it (decisions.md §15.3).
    ///
    /// ```
    /// use rcad_predicates::Policy;
    ///
    /// // Ten times the part, ten times the slack. Nothing else moves it.
    /// assert_eq!(
    ///     Policy::tolerant(1000.0).max_wobble(),
    ///     Policy::tolerant(100.0).max_wobble() * 10.0
    /// );
    /// assert_eq!(Policy::certified().max_wobble(), 0.0);
    /// ```
    #[must_use]
    pub fn max_wobble(self) -> f64 {
        match self.strictness {
            Strictness::Certified => 0.0,
            Strictness::TolerantFallback { working_size } => 64.0 * f64::EPSILON * working_size,
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

    /// Whether a point being this far out of place is within what the policy
    /// will accept.
    ///
    /// Asked of a single decision by [`resolve_under_policy`], and of a whole
    /// operation's worst wobble by whoever ran the operation — the fuse of
    /// decisions.md §1.7 is the first. One budget answers both, which is what
    /// stops an operation refusing a solid built out of decisions it had
    /// already accepted one at a time.
    #[must_use]
    pub fn wobble_is_acceptable(self, wobble: f64) -> bool {
        wobble <= self.max_wobble()
    }

    /// Whether this policy takes two points proved apart as one point, logging
    /// the move when it does.
    ///
    /// `record`'s enclosure is the furthest the two points may be apart, so it
    /// must measure a [`Measures::Length`]. Under [`Strictness::Certified`] the
    /// answer is `false` and nothing is logged. Under
    /// [`Strictness::TolerantFallback`] it is `true` when that distance is
    /// within [`Policy::max_wobble`], and then the record is resolved to
    /// [`Decision::Zero`] with a [`Cause::TolerantFallback`] tag of the distance
    /// and logged; otherwise it is `false` and nothing is logged, since a record
    /// wider than the budget would refuse the operation that asked, and
    /// declining the move already leaves that operation where it was.
    ///
    /// A power [`resolve_under_policy`] does not have: that one only reads a
    /// sign nothing could decide as zero, and here the two points were proved
    /// apart (`docs/decisions.md` §417.7).
    ///
    /// # Panics
    ///
    /// If `record` does not measure a length, or its outcome is already
    /// certified.
    pub fn take_as_one_point(self, mut record: Record, log: &mut ProvenanceLog) -> bool {
        assert_eq!(
            record.measures(),
            Measures::Length,
            "{}: how far apart two points are is a length",
            record.predicate()
        );
        assert!(
            !record.outcome().is_certified(),
            "{}: a certified decision does not go through a policy",
            record.predicate()
        );
        let Strictness::TolerantFallback { working_size } = self.strictness else {
            return false;
        };
        let wobble = record.wobble(working_size);
        if !self.wobble_is_acceptable(wobble) {
            return false;
        }
        record.resolve_to(
            Decision::Zero,
            Approximation::new(wobble, Cause::TolerantFallback),
        );
        log.push(record);
        true
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
    /// More lengths multiplied together than there is an everyday word for.
    ///
    /// A quadric's tangency discriminant is the first predicate here that goes
    /// past a volume. Asking whether a ball touches a pipe squares a value that
    /// is already an area, so what comes back grows with the *fourth* power of
    /// the model (decisions.md §27.4). Filed as a [`Measures::Volume`] it would
    /// be a whole working size out — ten thousandfold at 100 mm — which is the
    /// units mistake §15.3 exists to stop, arriving one degree above where that
    /// section could see it.
    ManyLengths {
        /// How many lengths are multiplied together. Four or more: one, two and
        /// three have names above, and two ways of spelling the same dimension
        /// is how a units bug hides.
        how_many: u32,
    },
}

impl Measures {
    /// What `value` says about how far out of place a point could be, at a
    /// model of size `working_size`.
    ///
    /// The conversion the rest of the kernel is built on. Everything stored on
    /// an entity or judged against a budget goes through here first, so that a
    /// number from one predicate can be compared with a number from another
    /// (decisions.md §15.2).
    ///
    /// ```
    /// use rcad_predicates::Measures;
    ///
    /// // The same wobble of a corner shows up a hundred times larger in a
    /// // volume than in an area, at a 100 mm working size. Reading one as if
    /// // it were the other is the units bug this exists to prevent.
    /// let area = Measures::Area.wobble_of(1e-9, 100.0);
    /// let volume = Measures::Volume.wobble_of(1e-9 * 100.0, 100.0);
    /// assert_eq!(area, volume);
    ///
    /// // A length is already a wobble and passes through untouched.
    /// assert_eq!(Measures::Length.wobble_of(1e-9, 100.0), 1e-9);
    /// ```
    ///
    /// Divided once per dimension above a length, rather than once by the
    /// working size raised to that power. Squaring the working size stops being
    /// finite above about `1.34e154`, and dividing by an infinite divisor hands
    /// back `0.0` — a magnitude saying nothing was approximated, which is the
    /// one thing decisions.md §5 makes that number mean. Dividing step by step
    /// never builds the divisor, so it has nothing to overflow.
    ///
    /// The answer is infinite when the true wobble is larger than any `f64`,
    /// which takes a working size down among the subnormals. Nothing is ever
    /// tagged with that: [`Policy::wobble_is_acceptable`] compares against a
    /// budget that is finite whenever the working size is, and no infinity is
    /// `<=` a finite number, so such a decision comes back undecided instead.
    ///
    /// # Panics
    ///
    /// If `working_size` is not a positive finite number, or if a
    /// [`Measures::ManyLengths`] names a dimension one of the three above it
    /// already has a word for.
    #[must_use]
    pub fn wobble_of(self, value: f64, working_size: f64) -> f64 {
        assert!(
            working_size.is_finite() && working_size > 0.0,
            "a wobble needs a positive working size, got {working_size}"
        );
        let dimension = match self {
            Measures::Length => 1,
            Measures::Area => 2,
            Measures::Volume => 3,
            Measures::ManyLengths { how_many } => {
                assert!(
                    how_many >= 4,
                    "{how_many} lengths multiplied together is a {}, which has its own name",
                    ["nothing", "length", "area", "volume"][how_many as usize]
                );
                how_many
            }
        };
        (1..dimension).fold(value, |so_far, _| so_far / working_size)
    }

    /// How much a predicate's value moves when a corner moves by one, at a
    /// model of size `working_size`.
    ///
    /// The working size raised to one dimension fewer than this measures: a
    /// length moves with the corner, an area sweeps a side along with it, a
    /// volume sweeps a face. The forward direction of [`Measures::wobble_of`],
    /// and only the tests need it — going forwards means building the very
    /// power that overflows, which is why nothing in the kernel does.
    #[cfg(test)]
    fn value_per_wobble(self, working_size: f64) -> f64 {
        let dimension = match self {
            Measures::Length => 1,
            Measures::Area => 2,
            Measures::Volume => 3,
            Measures::ManyLengths { how_many } => how_many,
        };
        (1..dimension).fold(1.0, |so_far, _| so_far * working_size)
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
        /// Not a budget: [`Policy::max_wobble`] is the budget, and it is a
        /// distance. Keeping the raw size here is what lets an enclosure of any
        /// degree be read as a distance before it is judged.
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
/// The record carries what its predicate measures, so the enclosure can be read
/// as a wobble and judged against the policy's one budget. Judging and tagging
/// use the same number on purpose: a tag that would not survive
/// [`Policy::wobble_is_acceptable`] is a tag an operation is about to refuse the
/// solid over.
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
        Strictness::TolerantFallback { working_size } => {
            let wobble = record.wobble(working_size);
            if policy.wobble_is_acceptable(wobble) {
                record.resolve_to(
                    Decision::Zero,
                    Approximation::new(wobble, Cause::UndecidableResolved),
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
        record_measuring(enclosure, Measures::Volume)
    }

    fn record_measuring(enclosure: f64, measures: Measures) -> Record {
        Record::new(
            "orient3d",
            &[Point::from([0.0, 0.0, 0.0])],
            Level::Interval,
            enclosure,
            measures,
        )
    }

    #[test]
    fn a_certified_policy_refuses_every_kind_of_slop() {
        let policy = Policy::certified();
        assert_eq!(policy.strictness(), Strictness::Certified);
        assert_eq!(policy.max_wobble(), 0.0);
        assert_eq!(policy.dynamic_range_check(), Severity::Fail);
        assert!(!policy.symbolic_perturbation());
        assert!(!policy.wobble_is_acceptable(1e-30));
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

    /// One policy permits one wobble, whatever predicate is asking. The widest
    /// enclosure each degree can get away with differs by a factor of the
    /// working size per degree, and all three come back to the same distance.
    #[test]
    fn one_policy_permits_one_wobble_whatever_the_predicate_measures() {
        let policy = Policy::tolerant(100.0);
        let budget = policy.max_wobble();

        for measures in every_dimension() {
            let widest_enclosure = measures.value_per_wobble(100.0) * budget;
            assert_eq!(measures.wobble_of(widest_enclosure, 100.0), budget);
            assert!(policy.wobble_is_acceptable(measures.wobble_of(widest_enclosure, 100.0)));
        }

        // The enclosures themselves are a hundredfold apart per degree. It is
        // only after converting that they are the same question.
        assert_eq!(
            Measures::Volume.value_per_wobble(100.0),
            Measures::Area.value_per_wobble(100.0) * 100.0
        );
    }

    /// Every dimension a predicate in this workspace hands back a value in.
    ///
    /// The fourth is M3's: a tangency discriminant squares a value that is
    /// already an area (decisions.md §27.4).
    fn every_dimension() -> [Measures; 4] {
        [
            Measures::Length,
            Measures::Area,
            Measures::Volume,
            Measures::ManyLengths { how_many: 4 },
        ]
    }

    /// The units mistake one degree above where §15.3 could see it, and it goes
    /// the other way: it refuses what it should accept.
    ///
    /// A tangency discriminant is four lengths deep. Filed as a volume its
    /// enclosure reads a whole working size too wide, so a decision the policy
    /// would have taken comes back undecided instead — a timid filter, which
    /// §24.10 says is the one thing a 0% control cannot tell apart from genuine
    /// undecidability. The other direction is the familiar one: a volume filed
    /// as four lengths is waved through.
    #[test]
    fn a_tangency_discriminant_filed_as_a_volume_refuses_what_the_policy_accepts() {
        let working_size = 100.0;
        let policy = Policy::tolerant(working_size);
        let four_lengths = Measures::ManyLengths { how_many: 4 };
        let at_the_edge = four_lengths.value_per_wobble(working_size) * policy.max_wobble();

        assert!(policy.wobble_is_acceptable(four_lengths.wobble_of(at_the_edge, working_size)));
        assert!(
            !policy.wobble_is_acceptable(Measures::Volume.wobble_of(at_the_edge, working_size))
        );

        let a_volume = Measures::Volume.value_per_wobble(working_size) * policy.max_wobble() * 50.0;
        assert!(!policy.wobble_is_acceptable(Measures::Volume.wobble_of(a_volume, working_size)));
        assert!(policy.wobble_is_acceptable(four_lengths.wobble_of(a_volume, working_size)));
    }

    /// Two ways to spell "an area" is how a units bug hides, so there is only
    /// one.
    #[test]
    #[should_panic(expected = "which has its own name")]
    fn a_dimension_that_already_has_a_word_may_not_be_spelled_as_a_count() {
        let _ = Measures::ManyLengths { how_many: 2 }.wobble_of(1.0, 100.0);
    }

    /// The bug this whole arrangement exists to stop, now at the tagging end.
    ///
    /// An enclosure fifty times too wide for the area predicate that produced
    /// it must be refused. Read as a volume it would be waved through, because
    /// the same number means a hundred times less wobble at this working size.
    #[test]
    fn an_area_read_as_a_volume_is_waved_through_when_it_should_not_be() {
        let policy = Policy::tolerant(100.0);
        let too_wide_for_an_area =
            Measures::Area.value_per_wobble(100.0) * policy.max_wobble() * 50.0;

        let mut log = ProvenanceLog::new();
        let honest = resolve_under_policy(
            record_measuring(too_wide_for_an_area, Measures::Area),
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
            record_measuring(too_wide_for_an_area, Measures::Volume),
            &policy,
            &mut log,
        );
        assert_eq!(with_the_wrong_units, Decision::Zero);
    }

    /// The point of §15.2: what lands on an entity is a distance, not the
    /// predicate's own value. At a working size of one hundred an `orient3d`
    /// enclosure is ten thousand times the wobble it stands for, so a tag that
    /// still held the enclosure would be out by that much.
    ///
    /// The old behaviour stored the enclosure, and every test that could have
    /// caught it ran at a working size of one — where the two are equal.
    #[test]
    fn a_tag_holds_how_far_out_of_place_a_point_could_be_not_the_predicates_own_value() {
        let policy = Policy::tolerant(100.0);
        let enclosure = 1e-20;
        let mut log = ProvenanceLog::new();

        let outcome = resolve_under_policy(
            record_measuring(enclosure, Measures::Volume),
            &policy,
            &mut log,
        );

        assert_eq!(outcome, Decision::Zero);
        let tag = log.records()[0]
            .approximation()
            .expect("a resolved record carries an approximation");
        assert_eq!(tag.magnitude(), enclosure / (100.0 * 100.0));
        assert_ne!(tag.magnitude(), enclosure);

        // The raw enclosure is still on the record, in its own units. Losing it
        // would be losing what the cascade actually saw.
        assert_eq!(log.records()[0].enclosure(), enclosure);
    }

    /// Judging and tagging read the same number, so an operation can never
    /// refuse a solid over slop that the very same policy accepted one decision
    /// at a time. Walking the boundary is where a second conversion, or a
    /// budget held in different units from the tag, would show up.
    #[test]
    fn anything_a_policy_resolves_is_something_the_same_policy_still_accepts() {
        for working_size in [1.0, 100.0, 1e6] {
            let policy = Policy::tolerant(working_size);
            for measures in every_dimension() {
                let at_the_edge = measures.value_per_wobble(working_size) * policy.max_wobble();
                for enclosure in [
                    at_the_edge * 0.5,
                    at_the_edge * (1.0 - f64::EPSILON),
                    at_the_edge,
                ] {
                    let mut log = ProvenanceLog::new();
                    let outcome = resolve_under_policy(
                        record_measuring(enclosure, measures),
                        &policy,
                        &mut log,
                    );
                    if outcome == Decision::Zero {
                        let tag = log.records()[0]
                            .approximation()
                            .expect("a resolved record carries an approximation");
                        assert!(
                            policy.wobble_is_acceptable(tag.magnitude()),
                            "resolved a {measures:?} enclosure of {enclosure} at working size \
                             {working_size} into a wobble of {} the same policy will not accept",
                            tag.magnitude()
                        );
                    }
                }
            }
        }
    }

    /// A working size whose square is not a finite number.
    ///
    /// The conversion used to build that square and divide by it, so the tag
    /// came back `0.0` — the number decisions.md §5 reserves for "nothing was
    /// approximated". The decision itself was never wrong here: the true wobble
    /// is far inside a budget this wide. The stored number was.
    #[test]
    fn a_working_size_too_large_to_square_still_gives_an_honest_wobble() {
        let working_size = 1e200f64;
        assert!(
            !(working_size * working_size).is_finite(),
            "this working size has to be one whose square overflows"
        );

        let enclosure = 2e290;
        let wobble = Measures::Volume.wobble_of(enclosure, working_size);
        assert_eq!(wobble, enclosure / working_size / working_size);
        assert!(
            wobble > 0.0,
            "the true wobble is not zero, so the tag is not"
        );

        let policy = Policy::tolerant(working_size);
        let mut log = ProvenanceLog::new();
        let outcome = resolve_under_policy(
            record_measuring(enclosure, Measures::Volume),
            &policy,
            &mut log,
        );

        assert_eq!(outcome, Decision::Zero);
        let tag = log.records()[0]
            .approximation()
            .expect("a resolved record carries an approximation");
        assert_eq!(tag.magnitude(), wobble);
        assert_eq!(log.worst_wobble(working_size), wobble);
    }

    /// The other end of the same conversion, and the demonstration behind
    /// `wobble_of`'s claim that an infinite wobble is never tagged.
    ///
    /// A working size among the subnormals puts the true wobble past anything
    /// an `f64` holds. Infinity is the honest answer, and the budget refuses it
    /// rather than an entity carrying it.
    #[test]
    fn a_wobble_larger_than_any_f64_is_refused_instead_of_tagged() {
        let working_size = 1e-200;
        let enclosure = 1e300;
        let wobble = Measures::Volume.wobble_of(enclosure, working_size);
        assert!(wobble.is_infinite());

        let policy = Policy::tolerant(working_size);
        assert!(policy.max_wobble().is_finite());
        assert!(!policy.wobble_is_acceptable(wobble));

        let mut log = ProvenanceLog::new();
        let outcome = resolve_under_policy(
            record_measuring(enclosure, Measures::Volume),
            &policy,
            &mut log,
        );

        assert_eq!(outcome, Decision::Undecidable { enclosure });
        assert_eq!(log.records()[0].approximation(), None);
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
        assert_eq!(policy.max_wobble(), 0.0);
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

    #[test]
    fn take_as_one_point_never_folds_under_a_certified_policy() {
        let mut log = ProvenanceLog::new();
        let distance = record_measuring(f64::MIN_POSITIVE, Measures::Length);

        assert!(!Policy::certified().take_as_one_point(distance, &mut log));
        assert!(log.is_empty());
    }

    #[test]
    fn a_tolerant_policy_takes_two_points_within_its_budget_as_one_and_tags_the_move() {
        let policy = Policy::tolerant(100.0);
        let distance = policy.max_wobble();
        let mut log = ProvenanceLog::new();

        assert!(policy.take_as_one_point(record_measuring(distance, Measures::Length), &mut log));
        assert_eq!(log.len(), 1);
        assert_eq!(log.records()[0].outcome(), Decision::Zero);
        let tag = log.records()[0]
            .approximation()
            .expect("a move carries how far it went");
        assert_eq!(tag.magnitude(), distance);
        assert_eq!(tag.cause(), Cause::TolerantFallback);
        assert!(policy.wobble_is_acceptable(log.worst_wobble(100.0)));
    }

    #[test]
    fn a_tolerant_policy_keeps_two_points_further_apart_than_its_budget_and_logs_nothing() {
        let policy = Policy::tolerant(100.0);
        let distance = policy.max_wobble().next_up();
        let mut log = ProvenanceLog::new();

        assert!(!policy.take_as_one_point(record_measuring(distance, Measures::Length), &mut log));
        assert!(log.is_empty());
    }

    #[test]
    #[should_panic(expected = "is a length")]
    fn how_far_apart_two_points_are_may_not_be_filed_as_a_volume() {
        let _ = Policy::tolerant(100.0)
            .take_as_one_point(record_with_enclosure(1e-30), &mut ProvenanceLog::new());
    }
}
