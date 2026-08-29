use serde::{Deserialize, Serialize};

use crate::{Decision, Level, Point};

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
    /// # Panics
    ///
    /// If `magnitude` is negative or not a number. An approximation you cannot
    /// compare is worse than none.
    pub fn new(magnitude: f64, cause: Cause) -> Self {
        assert!(
            magnitude >= 0.0,
            "an approximation magnitude must be a real amount, got {magnitude}"
        );
        Self { magnitude, cause }
    }

    /// How much slop was introduced, in the units of whatever produced it.
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
    /// The data arrived approximate — an imported file that did not close, or
    /// coordinates rounded by the exporter.
    ImportedUncertainty,
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
    approximation: Option<Approximation>,
}

impl Record {
    /// A record of `predicate` on `inputs` coming back `outcome` at `level`.
    ///
    /// # Panics
    ///
    /// If `outcome` is certified. A certified outcome does not get a record —
    /// that is the whole reason an empty log means something.
    pub fn new(predicate: &'static str, inputs: &[Point], level: Level, outcome: Decision) -> Self {
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

    /// How wide the interval that straddled zero was. Survives resolution, so
    /// it is still here after `outcome` has become a sign.
    pub fn enclosure(&self) -> f64 {
        self.enclosure
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

    /// The widest enclosure in the log, or zero if the log is empty.
    pub fn worst_enclosure(&self) -> f64 {
        self.records
            .iter()
            .map(Record::enclosure)
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
    use crate::{Decision, Level, Point};

    fn undecided_record() -> Record {
        Record::new(
            "orient3d",
            &[Point::from([0.0, 0.0, 0.0])],
            Level::Interval,
            Decision::Undecidable { enclosure: 1e-17 },
        )
    }

    #[test]
    fn a_fresh_log_is_empty() {
        let log = ProvenanceLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        assert_eq!(log.worst_enclosure(), 0.0);
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
    fn the_worst_enclosure_is_the_widest_one() {
        let mut log = ProvenanceLog::new();
        log.push(undecided_record());
        log.push(Record::new(
            "orient3d",
            &[],
            Level::Interval,
            Decision::Undecidable { enclosure: 4e-17 },
        ));
        assert_eq!(log.len(), 2);
        assert_eq!(log.worst_enclosure(), 4e-17);
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
        let _ = Record::new("orient3d", &[], Level::Float, Decision::Pos);
    }

    #[test]
    #[should_panic(expected = "must be a real amount")]
    fn a_negative_approximation_is_a_bug() {
        let _ = Approximation::new(-1.0, Cause::ImportedUncertainty);
    }

    #[test]
    fn an_approximation_survives_being_written_out_and_read_back() {
        let tag = Approximation::new(2.5e-9, Cause::TolerantFallback);
        let bytes = postcard::to_allocvec(&tag).unwrap();
        assert_eq!(postcard::from_bytes::<Approximation>(&bytes).unwrap(), tag);
    }
}
