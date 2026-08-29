use crate::interval::determinant;
use crate::{Error, Interval, Point};

/// Where three planes meet.
///
/// A plane is given as `[nx, ny, nz, d]`, meaning the points where
/// `nx*x + ny*y + nz*z + d` is zero. `rcad-geom` owns the real `Plane` type and
/// calls this with its coefficients; taking them as plain numbers keeps the
/// dependency running the right way round (decisions.md §7.2).
///
/// The answer is a [`Point::Constructed`]: a box that the true meeting point is
/// somewhere inside. Cramer's rule needs a division, and a division of
/// intervals almost never lands on an `f64`, so the box has real width even
/// when the planes are perfectly conditioned.
///
/// That width is the whole point. A predicate over a constructed point cannot
/// fall through to exact arithmetic, so it can genuinely come back
/// [`Decision::Undecidable`] (decisions.md §2.5).
///
/// # Errors
///
/// [`Error::PlanesDoNotMeetInAPoint`] when the three normals are coplanar — so
/// there is no single meeting point — or when they are close enough to coplanar
/// that we cannot bound one.
///
/// ```
/// # use rcad_predicates::{meet_of_three_planes, Point};
/// // x + y + z = 1, x = y, y = z. They meet at (1/3, 1/3, 1/3).
/// let meet = meet_of_three_planes(
///     [1.0, 1.0, 1.0, -1.0],
///     [1.0, -1.0, 0.0, 0.0],
///     [0.0, 1.0, -1.0, 0.0],
/// )
/// .unwrap();
/// assert!(meet.exact_coordinates().is_none());
/// assert!(meet.uncertainty() > 0.0, "a third is not an f64");
/// ```
///
/// [`Decision::Undecidable`]: crate::Decision::Undecidable
pub fn meet_of_three_planes(
    first: [f64; 4],
    second: [f64; 4],
    third: [f64; 4],
) -> Result<Point, Error> {
    let normals = [
        [
            Interval::exactly(first[0]),
            Interval::exactly(first[1]),
            Interval::exactly(first[2]),
        ],
        [
            Interval::exactly(second[0]),
            Interval::exactly(second[1]),
            Interval::exactly(second[2]),
        ],
        [
            Interval::exactly(third[0]),
            Interval::exactly(third[1]),
            Interval::exactly(third[2]),
        ],
    ];
    let offsets = [
        Interval::exactly(-first[3]),
        Interval::exactly(-second[3]),
        Interval::exactly(-third[3]),
    ];

    let volume = determinant(normals);
    if volume.contains_zero() {
        return Err(Error::PlanesDoNotMeetInAPoint {
            determinant_enclosure: volume.width(),
        });
    }

    let mut meet = [Interval::exactly(0.0); 3];
    for (axis, coordinate) in meet.iter_mut().enumerate() {
        let mut substituted = normals;
        for (row, offset) in substituted.iter_mut().zip(offsets) {
            row[axis] = offset;
        }
        *coordinate = determinant(substituted).divided_by(volume);
    }

    Ok(Point::Constructed(meet))
}

#[cfg(test)]
mod tests {
    use super::meet_of_three_planes;
    use crate::{Error, Point};

    #[test]
    fn three_axis_planes_meet_at_the_offsets_they_name() {
        let meet = meet_of_three_planes(
            [1.0, 0.0, 0.0, -2.0],
            [0.0, 1.0, 0.0, -3.0],
            [0.0, 0.0, 1.0, -4.0],
        )
        .unwrap();
        let Point::Constructed(box_) = meet else {
            panic!("a meet is always a constructed point");
        };
        assert_eq!(box_[0].lo(), 2.0);
        assert_eq!(box_[1].lo(), 3.0);
        assert_eq!(box_[2].lo(), 4.0);
        assert_eq!(meet.uncertainty(), 0.0, "these divisions are all by one");
    }

    #[test]
    fn a_meet_that_is_not_an_f64_comes_back_as_a_box_around_it() {
        let meet = meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .unwrap();
        let Point::Constructed(box_) = meet else {
            panic!("a meet is always a constructed point");
        };
        for side in box_ {
            assert!(side.lo() <= 1.0 / 3.0 && 1.0 / 3.0 <= side.hi());
            assert!(side.width() > 0.0);
        }
        assert!(meet.uncertainty() < 1e-15);
    }

    #[test]
    fn two_parallel_planes_have_no_meeting_point() {
        let outcome = meet_of_three_planes(
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0, -1.0],
            [1.0, 0.0, 0.0, 0.0],
        );
        assert_eq!(
            outcome,
            Err(Error::PlanesDoNotMeetInAPoint {
                determinant_enclosure: 0.0
            })
        );
    }

    #[test]
    fn three_planes_sharing_a_line_have_no_single_meeting_point() {
        let outcome = meet_of_three_planes(
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0, 0.0],
        );
        assert!(matches!(
            outcome,
            Err(Error::PlanesDoNotMeetInAPoint { .. })
        ));
    }

    #[test]
    fn planes_too_close_to_parallel_to_bound_are_refused() {
        // The determinant is exactly -2^-60, but the products it comes from
        // round at 2^-53, so level 2 cannot tell it from zero.
        let nudge = 2f64.powi(-30);
        let outcome = meet_of_three_planes(
            [1.0 + nudge, 1.0, 0.0, -1.0],
            [1.0, 1.0 - nudge, 0.0, -1.0],
            [0.0, 0.0, 1.0, 0.0],
        );
        let Err(Error::PlanesDoNotMeetInAPoint {
            determinant_enclosure,
        }) = outcome
        else {
            panic!("expected a refusal, got {outcome:?}");
        };
        assert!(determinant_enclosure > 0.0, "the planes are not parallel");
    }
}
