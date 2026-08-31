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

    /// A spline was asked for at a degree this kernel does not carry.
    #[error(
        "the degree must be between 1 and {limit}, but it was {degree}",
        limit = crate::MAX_DEGREE
    )]
    DegreeNotSupported {
        /// What the caller asked for.
        degree: usize,
    },

    /// A knot vector too short to be clamped at both ends.
    ///
    /// A degree `p` spline needs `p + 1` knots at each end before anything else
    /// can be said about it.
    #[error("{knots} knots is too few for a degree {degree} spline")]
    TooFewKnots {
        /// The degree that was asked for.
        degree: usize,
        /// How many knots came with it.
        knots: usize,
    },

    /// A knot is smaller than the one before it.
    #[error("the knot at position {at} is smaller than the one before it")]
    KnotsGoBackwards {
        /// Where the drop is.
        at: usize,
    },

    /// The first and last knot are so far apart that subtracting one from the
    /// other overflows.
    ///
    /// Every difference the de Boor recursion forms is between two of the
    /// stored knots, so none of them is wider than this one. A range that
    /// overflows makes every de Boor coefficient come out as zero, and the
    /// published evaluation bound certifies the wrong point that follows,
    /// because its derivation assumes the subtractions did not overflow.
    #[error("the knots run from {lo} to {hi}, which is too far apart to subtract in f64")]
    KnotsTooFarApart {
        /// The first knot.
        lo: f64,
        /// The last knot.
        hi: f64,
    },

    /// The first or last knot is not repeated enough times to pin the curve to
    /// its end control point.
    #[error(
        "the first and last knot must each be repeated one more time than the degree; \
         periodic knot vectors first arrive with STEP import at M5"
    )]
    KnotVectorNotClamped,

    /// An interior knot appears more often than the degree allows, which would
    /// leave a control point with no basis function of its own.
    #[error("the interior knot {knot} appears {times} times, more than the degree {degree}")]
    KnotRepeatedTooOften {
        /// The knot value that repeats.
        knot: f64,
        /// How many times it appears in the whole vector.
        times: usize,
        /// The degree it was measured against.
        degree: usize,
    },

    /// Every span between the ends has zero width, so the knot vector names no
    /// stretch of parameter at all.
    #[error("every span between the first and last knot has zero width")]
    KnotVectorHasNoSpan,

    /// The control net does not have the number of points the knot vector asks
    /// for.
    #[error("the knot vector asks for {expected} control points, but {given} were given")]
    WrongNumberOfControlPoints {
        /// How many the knot vector asks for.
        expected: usize,
        /// How many the caller handed over.
        given: usize,
    },

    /// A weight that cannot be divided back out.
    #[error("every weight must be a finite number greater than zero, but one was {weight}")]
    WeightNotPositive {
        /// The weight that was refused.
        weight: f64,
    },

    /// A weight was positive but down among the subnormal numbers, where a
    /// blend of two of them can round away to nothing.
    ///
    /// Refusing loses no curve. A rational spline depends on its weights only
    /// through their ratios, so multiplying every weight by a power of two
    /// gives back the identical curve, exactly and with no bits lost.
    #[error(
        "the weight {weight:e} is too small to blend; scale every weight up by a power of two \
         for the same curve"
    )]
    WeightTooSmallToBlend {
        /// The weight that was refused.
        weight: f64,
    },

    /// A parameter fell outside the stretch of parameter a spline is defined
    /// over.
    #[error("the parameter {parameter} is not between {lo} and {hi}")]
    ParameterOutsideDomain {
        /// What the caller asked for.
        parameter: f64,
        /// The bottom of the domain.
        lo: f64,
        /// The top of the domain.
        hi: f64,
    },

    /// A knot was asked to be inserted where it already repeats as often as the
    /// degree allows.
    ///
    /// One more copy would leave a control point with no basis function of its
    /// own, which is the same thing [`Error::KnotRepeatedTooOften`] refuses at
    /// construction. It is a separate variant because the caller here did not
    /// hand over a bad knot vector — it handed over a good one and asked for a
    /// refinement of it that does not exist.
    #[error(
        "the knot {knot} already appears {times} times, and a degree {degree} spline allows no more"
    )]
    CannotRepeatKnotAgain {
        /// The knot the caller asked to insert.
        knot: f64,
        /// How many times it already appears.
        times: usize,
        /// The degree it was measured against.
        degree: usize,
    },

    /// A curve was asked to be split at one end of its own domain, where one of
    /// the two halves would be no curve at all.
    #[error(
        "splitting at {parameter} leaves nothing on one side; the domain runs from {lo} to {hi}"
    )]
    NothingOnOneSideOfTheSplit {
        /// What the caller asked for.
        parameter: f64,
        /// The bottom of the domain.
        lo: f64,
        /// The top of the domain.
        hi: f64,
    },

    /// A predicate refused the input it was handed — three planes with no
    /// single meeting point, for instance.
    #[error(transparent)]
    Predicate(#[from] rcad_predicates::Error),
}

/// The result of anything in this crate that can refuse.
pub type Result<T, E = Error> = core::result::Result<T, E>;
