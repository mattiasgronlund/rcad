//! What can go wrong when building geometry.

/// Everything this crate refuses to build, and why.
///
/// All of these are bad caller input: numbers that cannot describe the shape
/// they are meant to describe. A broken invariant *inside* an already-valid
/// value is not here — that is a bug, and it panics
/// (`docs/decisions.md` §4.1).
///
/// `Undecidable` is not here either. A predicate that cannot decide returns a
/// `Decision`; it is an operation that cannot decide which is a failure, and
/// operations live above this crate.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// A coordinate was infinite or not a number.
    #[error("the {what} has a coordinate that is not a finite number")]
    NotFinite {
        /// Which piece of input it was.
        what: &'static str,
    },

    /// A vector that has to say which way it points has no length to say it
    /// with.
    #[error("the {what} has no length, so it does not point anywhere")]
    ZeroLength {
        /// Which piece of input it was.
        what: &'static str,
    },

    /// Three points that were meant to give a plane all sit on one line.
    #[error("the three points are all on one line, so they do not give a plane")]
    PointsAreInALine,

    /// A sphere or cylinder was asked for with a radius that is not a size.
    #[error("the radius must be greater than zero, but it was {radius}")]
    RadiusNotPositive {
        /// What the caller asked for.
        radius: f64,
    },

    /// The two ends of a segment are the same point.
    #[error("the two ends of the segment are the same point")]
    SegmentHasNoLength,

    /// A parameter fell outside the stretch of curve it was meant to name.
    #[error("the parameter {parameter} is not between 0 and the segment length {length}")]
    ParameterOutsideSegment {
        /// What the caller asked for.
        parameter: f64,
        /// How far along the segment it could have gone.
        length: f64,
    },

    /// A model was declared with a size that is not a size.
    #[error("the nominal size must be greater than zero, but it was {nominal_size}")]
    NominalSizeNotPositive {
        /// What the caller asked for.
        nominal_size: f64,
    },

    /// The smallest feature in a model was given as zero or less.
    #[error("the smallest feature size must be greater than zero, but it was {size}")]
    FeatureSizeNotPositive {
        /// What the caller asked for.
        size: f64,
    },

    /// The model spans too many orders of magnitude to be resolved in `f64`.
    #[error(
        "the model spans {ratio:e} times its smallest feature, past the {limit:e} that f64 can carry",
        limit = crate::DynamicRange::LARGEST_SUPPORTED_RATIO
    )]
    DynamicRangeTooWide {
        /// Largest coordinate magnitude divided by smallest feature size.
        ratio: f64,
    },
}

/// The result of anything in this crate that can refuse.
pub type Result<T, E = Error> = core::result::Result<T, E>;
