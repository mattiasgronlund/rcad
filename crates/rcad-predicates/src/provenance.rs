use serde::{Deserialize, Serialize};

use crate::{Decision, Level, Measures, Point};

/// The tag an entity carries because a decision put slop into it.
///
/// Small, self-contained and serializable on purpose: it has to survive being
/// written to disk without the log it came from. A pointer into a log would
/// dangle the moment the model is serialized on its own (decisions.md §5).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Approximation {
    magnitude: f64,
    cause: Cause,
}

impl Approximation {
    /// A tag saying `magnitude` of slop was introduced, for the given reason.
    ///
    /// `magnitude` is a **length** — how far out of place a point could be, in
    /// the model's own unit. Never the raw value of a predicate, which carries
    /// whatever units that predicate's degree gives it. [`Measures::wobble_of`]
    /// is what converts one into the other, and decisions.md §15.2 is why there
    /// is only one answer to what this field means.
    ///
    /// # Panics
    ///
    /// If `magnitude` is negative, infinite, or not a number. An approximation
    /// you cannot compare is worse than none, and neither infinity nor `NaN` is
    /// a length `rcad-topo`'s `check_approximation_ordering` can put in order.
    pub fn new(magnitude: f64, cause: Cause) -> Self {
        assert!(
            magnitude.is_finite() && magnitude >= 0.0,
            "an approximation magnitude must be a real amount, got {magnitude}"
        );
        Self { magnitude, cause }
    }

    /// How far out of place a point could be, in the model's own unit.
    pub fn magnitude(self) -> f64 {
        self.magnitude
    }

    /// Why the slop is there.
    pub fn cause(self) -> Cause {
        self.cause
    }
}

/// Why an entity ended up approximate.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cause {
    /// A predicate could not decide and a tolerant policy accepted the
    /// enclosure as zero. Set by [`resolve_under_policy`].
    ///
    /// [`resolve_under_policy`]: crate::resolve_under_policy
    UndecidableResolved,
    /// An operation chose an approximate construction over failing. Set by
    /// the operation, not by this crate.
    TolerantFallback,
    /// A point that had to be built rather than read — the meeting of three
    /// planes, say — landed between two `f64`, and the magnitude is how far
    /// the nearest `f64` sits from where it should be.
    ///
    /// Nothing was guessed here. This happens under a certified policy, with
    /// every predicate deciding, because a division rarely lands on a number
    /// `f64` can hold. That is why it is not [`Cause::TolerantFallback`]:
    /// filing the two together would leave a reader unable to tell an
    /// operation that decided everything and still could not write the answer
    /// down from one that could not decide and took the answer anyway.
    ConstructedPoint,
    /// The data arrived approximate — an imported file that did not close, or
    /// coordinates rounded by the exporter.
    ImportedUncertainty,
    /// A flat triangle was put where a curved surface is, and the magnitude is
    /// the furthest any point of it can be from that surface.
    ///
    /// Neither a fallback nor a resolution. Nothing failed and nothing was
    /// guessed: the caller asked for exactly this when it named a `Deflection`,
    /// and a coarse mesh is a request rather than a failure
    /// (`docs/decisions.md` §23.5). Filing it as [`Cause::TolerantFallback`]
    /// would leave a reader unable to tell an operation that gave up something
    /// it wanted from one that did what it was asked, and
    /// [`Cause::ConstructedPoint`] is about a number too fine to write down
    /// rather than about a chord standing in for an arc.
    ///
    /// **Last on purpose.** `postcard` writes an enum as its variant number, so
    /// a variant added anywhere but the end renumbers the ones after it and
    /// every model already on disk reads back a different reason
    /// (`docs/decisions.md` §25.8).
    FlatStandIn,
}

/// One decision that was not certified, with enough context to explain it.
///
/// Returned by value in an operation's log. Never stored in a model and never
/// serialized: the inputs alone would outgrow the model they describe
/// (decisions.md §5).
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    predicate: &'static str,
    inputs: Vec<Point>,
    level: Level,
    outcome: Decision,
    enclosure: f64,
    measures: Measures,
    approximation: Option<Approximation>,
}

impl Record {
    /// A record of `predicate` on `inputs` coming back `outcome` at `level`.
    ///
    /// `measures` says what the predicate's value is a measure of, and it is
    /// not optional. Without it the enclosure below is a bare number whose
    /// units nobody can recover, and a maximum taken over two records of
    /// different degrees means nothing (decisions.md §15.3).
    ///
    /// # Panics
    ///
    /// If `outcome` is certified. A certified outcome does not get a record —
    /// that is the whole reason an empty log means something.
    pub fn new(
        predicate: &'static str,
        inputs: &[Point],
        level: Level,
        outcome: Decision,
        measures: Measures,
    ) -> Self {
        let enclosure = match outcome {
            Decision::Undecidable { enclosure } => enclosure,
            Decision::Neg | Decision::Zero | Decision::Pos => {
                panic!("{predicate}: a certified outcome does not get a provenance record")
            }
        };
        Self {
            predicate,
            inputs: inputs.to_vec(),
            level,
            outcome,
            enclosure,
            measures,
            approximation: None,
        }
    }

    /// Which predicate this was.
    pub fn predicate(&self) -> &'static str {
        self.predicate
    }

    /// What it was given.
    pub fn inputs(&self) -> &[Point] {
        &self.inputs
    }

    /// How far the cascade got before giving up.
    pub fn level(&self) -> Level {
        self.level
    }

    /// What the caller was handed in the end. Starts out
    /// [`Decision::Undecidable`] and becomes a sign if a policy resolved it.
    pub fn outcome(&self) -> Decision {
        self.outcome
    }

    /// How wide the interval that straddled zero was, in the units of
    /// [`Record::measures`]. Survives resolution, so it is still here after
    /// `outcome` has become a sign.
    ///
    /// This is the honest record of what the cascade saw and it is not
    /// comparable across predicate degrees. [`Record::wobble`] is the
    /// comparable form.
    pub fn enclosure(&self) -> f64 {
        self.enclosure
    }

    /// What the predicate's value was a measure of, which is what says how to
    /// read [`Record::enclosure`].
    pub fn measures(&self) -> Measures {
        self.measures
    }

    /// [`Record::enclosure`] read as how far out of place a point could be, at
    /// a model of size `working_size`.
    ///
    /// A length, so two records may be compared with this even when the
    /// predicates behind them have different degrees.
    ///
    /// # Panics
    ///
    /// If `working_size` is not a positive finite number. A model's working
    /// size is checked when the model is made, so a bad one arriving here is a
    /// bug in the caller rather than something a user typed.
    pub fn wobble(&self, working_size: f64) -> f64 {
        self.measures.wobble_of(self.enclosure, working_size)
    }

    /// The tag to put on any entity built from this decision, if a policy
    /// resolved it. `None` when the record is just "we could not tell".
    pub fn approximation(&self) -> Option<Approximation> {
        self.approximation
    }

    /// Record that a policy turned the undecided outcome into `outcome`.
    pub(crate) fn resolve_to(&mut self, outcome: Decision, approximation: Approximation) {
        assert!(
            outcome.is_certified(),
            "{}: resolving must produce a certified sign",
            self.predicate
        );
        self.outcome = outcome;
        self.approximation = Some(approximation);
    }
}

/// The provenance an operation hands back.
///
/// Empty when everything was certified, which is both the common case and a
/// meaningful signal on its own. `Vec::new` does not allocate, so a certified
/// operation pays nothing for carrying one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProvenanceLog {
    records: Vec<Record>,
}

impl ProvenanceLog {
    /// An empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether anything at all was non-certified.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// How many non-certified decisions were recorded.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Everything that was recorded, oldest first.
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    /// The furthest out of place any of these decisions could have put a
    /// point, at a model of size `working_size`. Zero if the log is empty.
    ///
    /// A length, which is the whole point. A log may hold records from
    /// predicates of different degrees — the fuse of decisions.md §1.7 will
    /// hold them by the thousand — and a maximum over their raw enclosures
    /// would be picking the largest of an area and a volume. That is
    /// §12.4's mistake one level up, which is why there is no
    /// `worst_enclosure` to reach for instead.
    ///
    /// # Panics
    ///
    /// If `working_size` is not a positive finite number.
    pub fn worst_wobble(&self, working_size: f64) -> f64 {
        self.records
            .iter()
            .map(|record| record.wobble(working_size))
            .fold(0.0, f64::max)
    }

    /// Add a record.
    pub fn push(&mut self, record: Record) {
        self.records.push(record);
    }

    /// Take everything out, leaving the log empty. For folding a sub-operation's
    /// log into its caller's.
    pub fn drain_into(&mut self, other: &mut ProvenanceLog) {
        other.records.append(&mut self.records);
    }
}

#[cfg(test)]
mod tests {
    use super::{Approximation, Cause, ProvenanceLog, Record};
    use crate::{Decision, Level, Measures, Point};

    fn undecided_record() -> Record {
        record_measuring(1e-17, Measures::Volume)
    }

    fn record_measuring(enclosure: f64, measures: Measures) -> Record {
        Record::new(
            "orient3d",
            &[Point::from([0.0, 0.0, 0.0])],
            Level::Interval,
            Decision::Undecidable { enclosure },
            measures,
        )
    }

    #[test]
    fn a_fresh_log_is_empty() {
        let log = ProvenanceLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        assert_eq!(log.worst_wobble(100.0), 0.0);
    }

    #[test]
    fn a_record_keeps_its_enclosure_after_being_resolved() {
        let mut record = undecided_record();
        assert_eq!(record.approximation(), None);

        record.resolve_to(
            Decision::Zero,
            Approximation::new(1e-17, Cause::UndecidableResolved),
        );

        assert_eq!(record.outcome(), Decision::Zero);
        assert_eq!(record.enclosure(), 1e-17);
        assert_eq!(
            record.approximation().map(Approximation::cause),
            Some(Cause::UndecidableResolved)
        );
    }

    #[test]
    fn the_worst_wobble_is_the_widest_one() {
        let mut log = ProvenanceLog::new();
        log.push(record_measuring(1e-17, Measures::Volume));
        log.push(record_measuring(4e-17, Measures::Volume));
        assert_eq!(log.len(), 2);
        assert_eq!(log.worst_wobble(1.0), 4e-17);
    }

    /// The reason there is no `worst_enclosure` to reach for.
    ///
    /// Two records, and the one with the *smaller* enclosure is the one that
    /// could have moved a point further — because an area and a volume are a
    /// hundredfold apart at this working size. A maximum taken over the raw
    /// enclosures picks the wrong record.
    #[test]
    fn the_widest_enclosure_is_not_the_worst_wobble_when_two_degrees_share_a_log() {
        let working_size = 100.0;
        let from_an_area = 2e-15;
        let from_a_volume = 1e-14;
        let mut log = ProvenanceLog::new();
        log.push(record_measuring(from_an_area, Measures::Area));
        log.push(record_measuring(from_a_volume, Measures::Volume));

        // The volume's enclosure is the wider number, so a maximum over raw
        // enclosures would pick it.
        assert!(from_a_volume > from_an_area);

        // The area's wobble is the worse answer, so the maximum that matters
        // picks the other one.
        let from_the_area = Measures::Area.wobble_of(from_an_area, working_size);
        let from_the_volume = Measures::Volume.wobble_of(from_a_volume, working_size);
        assert!(from_the_area > from_the_volume * 10.0);
        assert_eq!(log.worst_wobble(working_size), from_the_area);
    }

    #[test]
    fn draining_moves_every_record_and_leaves_the_source_empty() {
        let mut inner = ProvenanceLog::new();
        inner.push(undecided_record());
        let mut outer = ProvenanceLog::new();

        inner.drain_into(&mut outer);

        assert!(inner.is_empty());
        assert_eq!(outer.len(), 1);
    }

    #[test]
    #[should_panic(expected = "does not get a provenance record")]
    fn a_certified_outcome_may_not_be_recorded() {
        let _ = Record::new(
            "orient3d",
            &[],
            Level::Float,
            Decision::Pos,
            Measures::Volume,
        );
    }

    #[test]
    #[should_panic(expected = "must be a real amount")]
    fn a_negative_approximation_is_a_bug() {
        let _ = Approximation::new(-1.0, Cause::ImportedUncertainty);
    }

    /// The field is a length, and infinity is not one. Letting it through would
    /// put a number on an entity that no ordering check can act on, and it
    /// would survive to disk.
    #[test]
    #[should_panic(expected = "must be a real amount")]
    fn an_approximation_bigger_than_any_f64_is_a_bug() {
        let _ = Approximation::new(f64::INFINITY, Cause::ImportedUncertainty);
    }

    /// Two logs carrying the same amount of slop for different reasons must
    /// not look alike.
    ///
    /// One operation decided every predicate and still could not write its
    /// meeting point down exactly; the other could not decide and was allowed
    /// to take an answer anyway. Same wobble, and the difference is the whole
    /// diagnosis. Reusing [`Cause::TolerantFallback`] for the first would
    /// throw it away.
    #[test]
    fn a_point_that_would_not_fit_in_an_f64_is_not_a_tolerant_fallback() {
        let wobble = 1e-17;
        let causes_of = |log: &ProvenanceLog| {
            log.records()
                .iter()
                .filter_map(|record| record.approximation().map(Approximation::cause))
                .collect::<Vec<_>>()
        };

        let mut nothing_was_guessed = ProvenanceLog::new();
        nothing_was_guessed.push(resolved_with(Cause::ConstructedPoint, wobble));
        let mut something_was_guessed = ProvenanceLog::new();
        something_was_guessed.push(resolved_with(Cause::TolerantFallback, wobble));

        assert_eq!(
            nothing_was_guessed.worst_wobble(1.0),
            something_was_guessed.worst_wobble(1.0),
            "the two logs are the same size, so only the cause can tell them apart"
        );
        assert_eq!(causes_of(&nothing_was_guessed), [Cause::ConstructedPoint]);
        assert_eq!(causes_of(&something_was_guessed), [Cause::TolerantFallback]);
        assert_ne!(nothing_was_guessed, something_was_guessed);

        let tag = Approximation::new(wobble, Cause::ConstructedPoint);
        let bytes = postcard::to_allocvec(&tag).unwrap();
        assert_eq!(
            postcard::from_bytes::<Approximation>(&bytes)
                .unwrap()
                .cause(),
            Cause::ConstructedPoint,
            "the distinction has to survive being written to disk"
        );
    }

    /// A mesh's slop is not a fallback and not a rounding, and a reader has to
    /// be able to tell.
    ///
    /// All three can carry the same magnitude. A tolerant fallback means an
    /// operation wanted to be exact and settled; a constructed point means it
    /// *was* exact and `f64` had no room to say so; a flat stand-in means the
    /// caller asked for a rough picture and got one. Only the cause separates
    /// them, so only the cause can be checked.
    #[test]
    fn a_triangle_standing_in_for_a_curve_is_neither_a_fallback_nor_a_rounding() {
        let wobble = 1e-4;
        let for_the_mesh = Approximation::new(wobble, Cause::FlatStandIn);

        assert_ne!(
            for_the_mesh,
            Approximation::new(wobble, Cause::TolerantFallback)
        );
        assert_ne!(
            for_the_mesh,
            Approximation::new(wobble, Cause::ConstructedPoint)
        );

        let bytes = postcard::to_allocvec(&for_the_mesh).unwrap();
        assert_eq!(
            postcard::from_bytes::<Approximation>(&bytes)
                .unwrap()
                .cause(),
            Cause::FlatStandIn,
            "the distinction has to survive being written to disk"
        );
    }

    /// A cause is written as its number, so where a new one is added is part of
    /// the format.
    ///
    /// Putting [`Cause::FlatStandIn`] anywhere but the end would renumber the
    /// causes after it, and every `Approximation` already on disk would read
    /// back blaming something else. Nothing would notice: the bytes still parse
    /// and the magnitude is still right.
    #[test]
    fn a_new_cause_goes_on_the_end_so_the_ones_before_it_keep_their_numbers() {
        for (number, cause) in [
            Cause::UndecidableResolved,
            Cause::TolerantFallback,
            Cause::ConstructedPoint,
            Cause::ImportedUncertainty,
            Cause::FlatStandIn,
        ]
        .into_iter()
        .enumerate()
        {
            let number = u8::try_from(number).expect("five causes");
            assert_eq!(
                postcard::to_allocvec(&cause).unwrap(),
                [number],
                "{cause:?}"
            );
        }
    }

    fn resolved_with(cause: Cause, wobble: f64) -> Record {
        let mut record = undecided_record();
        record.resolve_to(Decision::Zero, Approximation::new(wobble, cause));
        record
    }

    #[test]
    fn an_approximation_survives_being_written_out_and_read_back() {
        let tag = Approximation::new(2.5e-9, Cause::TolerantFallback);
        let bytes = postcard::to_allocvec(&tag).unwrap();
        assert_eq!(postcard::from_bytes::<Approximation>(&bytes).unwrap(), tag);
    }
}
