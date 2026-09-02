use rcad_predicates::ProvenanceLog;

/// What went wrong, whichever layer it went wrong in.
///
/// Three kinds of thing land here, and only three:
///
/// - **Bad caller input** — a profile that crosses itself, a radius of zero, a
///   file written by another version of the format. It is data, not a bug, so
///   it is an error.
/// - **A model that breaks an invariant**, found by
///   [`Model::validate`](crate::Model::validate). Same reasoning: a file
///   arriving broken is data.
/// - **An operation with nothing left to return**, which is
///   [`Error::Undecidable`].
///
/// A broken invariant *inside* the kernel is not here. That panics: emitting a
/// corrupt solid is worse than dying (`docs/decisions.md` §4.1).
///
/// A predicate that cannot decide is not here either. That is
/// [`Decision::Undecidable`](crate::Decision::Undecidable), and a point that
/// cannot be placed is [`Containment::Undecidable`](crate::Containment::Undecidable).
/// Both are results and both are perfectly normal. Only when a whole operation
/// is left with nothing to hand back does it become
/// [`Error::Undecidable`](crate::Error::Undecidable), carrying the records of
/// the decisions that gave up (`docs/decisions.md` §4.2).
///
/// The three wrapped types are re-exported as [`PredicateError`],
/// [`GeometryError`] and [`TopologyError`], so matching all the way through to
/// an inner variant needs nothing but `rcad::` paths. They stay three separate
/// layers rather than one flattened enum: which layer refused says what kind of
/// thing went wrong, and that is worth keeping visible.
///
/// Reading and writing a STEP file are two more layers, each with its own
/// reason to refuse that has nothing to do with the kernel's own three:
/// [`Error::StepSyntax`] is bytes that are not a Part 21 file at all,
/// [`Error::StepImport`] is a Part 21 file whose solids could not become a
/// `Model`, and [`Error::StepWrite`] is a `Model` this writer could not put in
/// one. [`StepSyntaxError`], [`Refusal`] and [`CannotWrite`] are the wrapped
/// types, re-exported the same way as the kernel's own three.
///
/// [`PredicateError`]: crate::PredicateError
/// [`GeometryError`]: crate::GeometryError
/// [`TopologyError`]: crate::TopologyError
/// [`StepSyntaxError`]: crate::StepSyntaxError
/// [`Refusal`]: crate::Refusal
/// [`CannotWrite`]: crate::CannotWrite
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A predicate refused the input it was handed.
    #[error(transparent)]
    Predicate(#[from] crate::PredicateError),

    /// A piece of geometry refused to be built.
    #[error(transparent)]
    Geometry(crate::GeometryError),

    /// Building, reading or checking a model went wrong.
    #[error(transparent)]
    Topology(crate::TopologyError),

    /// The operation ran out of things it could prove.
    ///
    /// Carries every decision that could not be settled, so the caller can see
    /// which predicate gave up and how wide its enclosure was.
    #[error("could not settle {} decisions the answer depends on", records.len())]
    Undecidable {
        /// The decisions that could not be settled.
        records: ProvenanceLog,
    },

    /// Bytes handed to [`StepFile::read`](crate::StepFile::read) were not a
    /// Part 21 file this reader could parse.
    #[error(transparent)]
    StepSyntax(#[from] crate::StepSyntaxError),

    /// A Part 21 file parsed, but [`Import::read`](crate::Import::read) could
    /// not turn its solids into a `Model`.
    #[error(transparent)]
    StepImport(#[from] crate::Refusal),

    /// A `Model` could not be written back out as a Part 21 file by
    /// [`write`](crate::write).
    #[error(transparent)]
    StepWrite(#[from] crate::CannotWrite),
}

/// An undecided *operation* keeps its own variant here rather than being buried
/// inside [`Error::Topology`] or [`Error::Geometry`].
///
/// The facade promises that an operation which cannot decide is
/// [`Error::Undecidable`] carrying the records. If either layer's own undecided
/// error stayed wrapped, a caller would have to unwrap two layers to find out
/// that is what happened, and the promise would be worth nothing.
///
/// Two layers can raise one now. `rcad-topo`'s fuse has since M2.5, and naming
/// where two surfaces cross is an operation in `rcad-geom` that can give up the
/// same way (`docs/decisions.md` §24.10).
impl From<crate::TopologyError> for Error {
    fn from(error: crate::TopologyError) -> Self {
        match error {
            crate::TopologyError::Undecidable { records } => Error::Undecidable { records },
            other => Error::Topology(other),
        }
    }
}

impl From<crate::GeometryError> for Error {
    fn from(error: crate::GeometryError) -> Self {
        match error {
            crate::GeometryError::Undecidable { records } => Error::Undecidable { records },
            other => Error::Geometry(other),
        }
    }
}

/// The result of anything in this kernel that can refuse.
pub type Result<T, E = Error> = core::result::Result<T, E>;
