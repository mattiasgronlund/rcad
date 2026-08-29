//! Straight edge geometry.

use serde::{Deserialize, Serialize};

use crate::isometry::Isometry;
use crate::point::{Point, Vector};
use crate::{Error, Result};

/// A straight run from one point to another — the geometry under an edge.
///
/// M0 has straight edges and nothing else (`docs/decisions.md` §3.1), so this
/// is a concrete type rather than a variant of a curve enum. When curves arrive
/// they bring the enum with them.
///
/// The parameter along a segment is **distance from the start**, running from
/// `0.0` to [`Segment::length`]. Measuring it in the model's own length unit
/// means a tolerance on the parameter is a tolerance in space, with no scaling
/// in between.
///
/// Serializes as `(start, end)` and is re-checked on the way in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(Point, Point)", into = "(Point, Point)")]
pub struct Segment {
    start: Point,
    end: Point,
    direction: Vector,
    length: f64,
}

impl Segment {
    /// The straight run from `start` to `end`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if either end is not made of finite numbers or the
    /// two are so far apart that the length overflows, and
    /// [`Error::SegmentHasNoLength`] if the two ends are the same point.
    pub fn new(start: Point, end: Point) -> Result<Self> {
        if !start.is_finite() || !end.is_finite() {
            return Err(Error::NotFinite {
                what: "end of a segment",
            });
        }
        let along = end - start;
        let length = along.length();
        if length == 0.0 {
            return Err(Error::SegmentHasNoLength);
        }
        if !length.is_finite() {
            return Err(Error::NotFinite {
                what: "length of a segment",
            });
        }
        Ok(Self {
            start,
            end,
            direction: along.normalized_as("direction of a segment")?,
            length,
        })
    }

    /// Where the run begins.
    #[must_use]
    pub fn start(&self) -> Point {
        self.start
    }

    /// Where the run ends.
    #[must_use]
    pub fn end(&self) -> Point {
        self.end
    }

    /// Which way it runs. Always has length one.
    #[must_use]
    pub fn direction(&self) -> Vector {
        self.direction
    }

    /// How far it is from one end to the other. Always greater than zero.
    #[must_use]
    pub fn length(&self) -> f64 {
        self.length
    }

    /// The smallest and largest parameter on the segment: `(0.0, length)`.
    #[must_use]
    pub fn parameter_range(&self) -> (f64, f64) {
        (0.0, self.length)
    }

    /// The point `distance_from_start` along the run.
    ///
    /// At `0.0` this is exactly [`Segment::start`]. At [`Segment::length`] it is
    /// [`Segment::end`] up to rounding, not bit for bit.
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideSegment`] if the distance is negative, past the
    /// far end, or not a number. A parameter outside the domain is bad caller
    /// input, not a broken invariant (`docs/decisions.md` §4.1).
    pub fn point_at(&self, distance_from_start: f64) -> Result<Point> {
        if !(distance_from_start >= 0.0 && distance_from_start <= self.length) {
            return Err(Error::ParameterOutsideSegment {
                parameter: distance_from_start,
                length: self.length,
            });
        }
        Ok(self.start + self.direction * distance_from_start)
    }

    /// The point on the segment nearest to `point`.
    ///
    /// Never runs off the ends: a point beyond either end gets that end.
    #[must_use]
    pub fn closest_point(&self, point: Point) -> Point {
        self.start + self.direction * self.parameter_of_closest_point(point)
    }

    /// How far along the segment the nearest point to `point` sits, clamped to
    /// the segment's own range.
    #[must_use]
    pub fn parameter_of_closest_point(&self, point: Point) -> f64 {
        self.direction
            .dot(point - self.start)
            .clamp(0.0, self.length)
    }

    /// The same run in the other direction.
    #[must_use]
    pub fn reversed(&self) -> Self {
        Self {
            start: self.end,
            end: self.start,
            direction: -self.direction,
            length: self.length,
        }
    }

    /// The same run after `movement` is applied.
    ///
    /// # Panics
    ///
    /// If the moved ends land on top of each other. A rigid movement keeps
    /// distances, so reaching this means the segment or the movement was
    /// already broken.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        Self::new(
            movement.apply_to_point(self.start),
            movement.apply_to_point(self.end),
        )
        .expect("a rigid movement cannot collapse a segment that had a length")
    }
}

impl TryFrom<(Point, Point)> for Segment {
    type Error = Error;

    fn try_from((start, end): (Point, Point)) -> Result<Self> {
        Self::new(start, end)
    }
}

impl From<Segment> for (Point, Point) {
    fn from(segment: Segment) -> Self {
        (segment.start, segment.end)
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::FRAC_PI_2;

    use super::*;
    use crate::Axis;

    fn segment_along_x() -> Segment {
        Segment::new(Point::origin(), Point::new(10.0, 0.0, 0.0)).unwrap()
    }

    #[test]
    fn an_edge_between_a_point_and_itself_is_refused() {
        let point = Point::new(1.0, 2.0, 3.0);
        assert_eq!(Segment::new(point, point), Err(Error::SegmentHasNoLength));
    }

    #[test]
    fn an_edge_to_a_coordinate_that_is_not_a_number_is_refused() {
        assert_eq!(
            Segment::new(Point::origin(), Point::new(f64::NAN, 0.0, 0.0)),
            Err(Error::NotFinite {
                what: "end of a segment"
            })
        );
    }

    #[test]
    fn the_length_is_the_distance_between_the_two_ends() {
        let segment = Segment::new(Point::origin(), Point::new(3.0, 4.0, 0.0)).unwrap();
        assert_eq!(segment.length(), 5.0);
        assert_eq!(segment.direction(), Vector::new(0.6, 0.8, 0.0));
        assert_eq!(segment.parameter_range(), (0.0, 5.0));
    }

    #[test]
    fn the_parameter_is_measured_in_the_same_length_as_the_points() {
        let segment = segment_along_x();
        assert_eq!(segment.point_at(0.0).unwrap(), Point::origin());
        assert_eq!(segment.point_at(2.5).unwrap(), Point::new(2.5, 0.0, 0.0));
        assert!(
            segment
                .point_at(10.0)
                .unwrap()
                .distance_to(Point::new(10.0, 0.0, 0.0))
                < 1e-15
        );
    }

    #[test]
    fn a_parameter_past_the_end_of_the_segment_is_refused() {
        assert_eq!(
            segment_along_x().point_at(10.5),
            Err(Error::ParameterOutsideSegment {
                parameter: 10.5,
                length: 10.0
            })
        );
    }

    #[test]
    fn a_parameter_before_the_start_of_the_segment_is_refused() {
        assert_eq!(
            segment_along_x().point_at(-0.5),
            Err(Error::ParameterOutsideSegment {
                parameter: -0.5,
                length: 10.0
            })
        );
    }

    #[test]
    fn a_parameter_that_is_not_a_number_is_refused() {
        assert!(segment_along_x().point_at(f64::NAN).is_err());
    }

    #[test]
    fn the_closest_point_to_something_alongside_is_the_foot_of_the_perpendicular() {
        assert_eq!(
            segment_along_x().closest_point(Point::new(4.0, 7.0, 0.0)),
            Point::new(4.0, 0.0, 0.0)
        );
    }

    #[test]
    fn the_closest_point_never_runs_off_either_end() {
        let segment = segment_along_x();
        assert_eq!(
            segment.closest_point(Point::new(-100.0, 1.0, 0.0)),
            Point::origin()
        );
        assert_eq!(
            segment.closest_point(Point::new(100.0, 1.0, 0.0)),
            Point::new(10.0, 0.0, 0.0)
        );
    }

    #[test]
    fn reversing_swaps_the_ends_and_flips_the_direction() {
        let back = segment_along_x().reversed();
        assert_eq!(back.start(), Point::new(10.0, 0.0, 0.0));
        assert_eq!(back.end(), Point::origin());
        assert_eq!(back.direction(), Vector::new(-1.0, 0.0, 0.0));
        assert_eq!(back.length(), 10.0);
    }

    #[test]
    fn moving_a_segment_moves_both_of_its_ends_and_keeps_its_length() {
        let segment = segment_along_x();
        let turn = Isometry::rotation(&Axis::z(), FRAC_PI_2).unwrap();
        let moved = segment.transformed(&turn);
        assert!(moved.end().distance_to(Point::new(0.0, 10.0, 0.0)) < 1e-14);
        assert!((moved.length() - segment.length()).abs() < 1e-14);
    }

    #[test]
    fn a_segment_rebuilt_from_stored_ends_that_are_the_same_point_is_refused() {
        let stored = (Point::origin(), Point::origin());
        assert_eq!(Segment::try_from(stored), Err(Error::SegmentHasNoLength));
    }
}
