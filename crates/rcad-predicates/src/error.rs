/// What can go wrong in this crate.
///
/// Only bad caller input lands here. A broken internal invariant panics, and a
/// predicate that cannot decide returns [`Decision::Undecidable`] instead
/// (decisions.md §4.1).
///
/// [`Decision::Undecidable`]: crate::Decision::Undecidable
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// `lo` was above `hi`, or one of them was not a number.
    #[error("[{lo}, {hi}] is not an interval")]
    NotAnInterval {
        /// The lower bound that was asked for.
        lo: f64,
        /// The upper bound that was asked for.
        hi: f64,
    },

    /// The three planes have no single meeting point, or are so close to
    /// parallel that we cannot bound where it would be.
    #[error(
        "the three planes do not meet in one point: the determinant enclosure is {determinant_enclosure} wide and straddles zero"
    )]
    PlanesDoNotMeetInAPoint {
        /// How wide the determinant enclosure was. Zero means the planes are
        /// exactly parallel; anything larger means we could not tell.
        determinant_enclosure: f64,
    },

    /// A file said something was out of place by an amount that is not a
    /// length — negative, infinite, or not a number.
    ///
    /// Raised by
    /// [`TryFrom<(f64, Cause)> for Approximation`](crate::Approximation),
    /// which is what a `serde` read of one goes through.
    #[error("{magnitude} is not an amount anything can be out of place by")]
    ApproximationIsNotALength {
        /// The magnitude the file asked for.
        magnitude: f64,
    },
}
