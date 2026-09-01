use crate::interval::determinant;
use crate::power_of_two::{exponent_that_brings_into_range, scaled_by_a_power_of_two};
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
/// Coefficients that are uniformly tiny or uniformly enormous can make every
/// term of the determinant underflow or overflow even though the three planes
/// are perfectly well conditioned. That is decided anyway: each plane is
/// retried scaled by its own exact power of two
/// ([`meet_after_rescaling_each_plane`]), which moves it back into range
/// without changing which points satisfy it.
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
    let refused = match meet_of_three_planes_at_this_scale(first, second, third) {
        Ok(point) => return Ok(point),
        Err(refused) => refused,
    };
    meet_after_rescaling_each_plane(first, second, third)
        .map(Ok)
        .unwrap_or(Err(refused))
}

/// [`meet_of_three_planes`], reading every coefficient exactly as given.
fn meet_of_three_planes_at_this_scale(
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

/// One more try, each plane moved by its own exact power of two. `None` when
/// one of the three has no such copy, or the rescaled copy is refused too.
///
/// A plane `[nx, ny, nz, d]` names the same points after every one of its four
/// coefficients is multiplied by the same nonzero number — that is what makes
/// it an equation rather than four independent numbers. So scaling plane `i`
/// by `sᵢ` changes nothing about where the three planes meet. What it changes
/// is whether the arithmetic can see that: [`meet_of_three_planes_at_this_scale`]
/// forms `normals[i]` and `offsets[i]` from that same plane's coefficients, so
/// scaling the plane scales both by exactly `sᵢ`, and every entry `substituted`
/// borrows from row `i` — whether from `normals[i]` or, for the column being
/// solved, from `offsets[i]` — scales by that same `sᵢ` too. A determinant is
/// linear in each of its rows, so both `volume` and every substituted
/// determinant scale by `s₁s₂s₃` together, and Cramer's ratio (the two divided
/// by each other) has that factor cancel exactly, row by row, whatever `s₁`,
/// `s₂` and `s₃` individually are — no shared exponent required, unlike
/// `orient3d`'s own retry, whose degree-3 homogeneity argument needs one
/// factor common to every coordinate.
///
/// Each plane picks its own exponent off its own largest coefficient
/// ([`exponent_that_brings_into_range`]), so three planes at three unrelated
/// scales are each brought into range on their own terms rather than forced
/// through one scale that might not fit all three at once. A plane that has no
/// exact scaled copy — an infinite or overflowing coefficient — sinks this
/// attempt and leaves the original refusal standing, which is correct: that
/// plane is not representable in range at any scale, so there is no copy to
/// have tried.
fn meet_after_rescaling_each_plane(
    first: [f64; 4],
    second: [f64; 4],
    third: [f64; 4],
) -> Option<Point> {
    let first = rescaled(first)?;
    let second = rescaled(second)?;
    let third = rescaled(third)?;
    meet_of_three_planes_at_this_scale(first, second, third).ok()
}

/// `plane`, moved by the exact power of two that brings its own largest
/// coefficient into range. `None` when the plane is all zeros (nothing to
/// bring anywhere) or when moving it loses precision at either end of the
/// `f64` range.
fn rescaled(plane: [f64; 4]) -> Option<[f64; 4]> {
    let exponent = exponent_that_brings_into_range(plane)?;
    scaled_by_a_power_of_two(plane, exponent)
}

#[cfg(test)]
mod tests {
    use super::{meet_of_three_planes, scaled_by_a_power_of_two};
    use crate::{Error, Point};

    /// The repro decisions.md §39.7 item 7 names: three mutually perpendicular
    /// planes, every coefficient written at `1e-200`. The true meeting point is
    /// `(2, 3, 4)`, in whole numbers with nothing rounded anywhere else — only
    /// the uniform tiny scale stops the unscaled arithmetic seeing it, which is
    /// what makes this a gap rather than a limit.
    #[test]
    fn three_mutually_perpendicular_planes_written_at_1e_200_still_answer() {
        let s = 1e-200_f64;
        let meet = meet_of_three_planes(
            [s, 0.0, 0.0, -2.0 * s],
            [0.0, s, 0.0, -3.0 * s],
            [0.0, 0.0, s, -4.0 * s],
        )
        .expect("the meeting point exists and is representable");
        let Point::Constructed(box_) = meet else {
            panic!("a meet is always a constructed point");
        };
        for (side, truth) in box_.into_iter().zip([2.0, 3.0, 4.0]) {
            assert!(
                side.lo() <= truth && truth <= side.hi(),
                "{truth} is not inside [{}, {}]",
                side.lo(),
                side.hi()
            );
        }
    }

    /// Each plane is retried at the exact power of two that brings its own
    /// largest coefficient into range, not at one exponent shared by all
    /// three. Three unrelated exponents have to reach the same answer as one
    /// shared exponent, or the row-by-row cancellation the fix relies on is
    /// not what is actually happening.
    ///
    /// The two triples below name the same three planes — `x = 2`, `y = 3`,
    /// `z = 4` — each plane multiplied by its own exact power of two. Scaling
    /// by a power of two never touches a mantissa, so retrying brings each
    /// plane back to the one canonical copy with its largest coefficient in
    /// `[1, 2)`, and that copy does not depend on which power of two the row
    /// started at. Two experiments that start from different exponents and
    /// land on the same canonical copy have to answer bit for bit alike; if
    /// the cancellation needed a shared exponent the way `orient3d`'s does,
    /// scaling the three rows apart like this would change the answer instead.
    #[test]
    fn three_different_per_plane_exponents_give_the_same_box_as_one_shared_exponent() {
        let base = (
            [1.0, 0.0, 0.0, -2.0],
            [0.0, 1.0, 0.0, -3.0],
            [0.0, 0.0, 1.0, -4.0],
        );

        let shared = meet_of_three_planes(
            written_at(base.0, -700),
            written_at(base.1, -700),
            written_at(base.2, -700),
        )
        .expect("a shared exponent still leaves a gap the retry can close");

        let different = meet_of_three_planes(
            written_at(base.0, -700),
            written_at(base.1, -600),
            written_at(base.2, -800),
        )
        .expect("three different exponents leave the same gap open to the same retry");

        assert_eq!(
            shared, different,
            "each plane cancels its own scale row by row, so the exponents \
             the three planes were written at should not have to match"
        );
    }

    fn written_at(plane: [f64; 4], exponent: i32) -> [f64; 4] {
        scaled_by_a_power_of_two(plane, exponent)
            .expect("an exponent this modest always has an exact scaled copy")
    }

    /// A coefficient that is infinite has no scaled copy at any exponent
    /// (`scaled_by_a_power_of_two` refuses as soon as one is not `is_normal()`),
    /// so the retry sinks and the refusal it started from has to stand. An
    /// out-of-range plane is a limit, not a gap, and the fix must not blur
    /// that line.
    #[test]
    fn a_plane_with_an_infinite_coefficient_is_not_rescued_by_the_retry() {
        let outcome = meet_of_three_planes(
            [f64::INFINITY, 0.0, 0.0, -2.0],
            [0.0, 1.0, 0.0, -3.0],
            [0.0, 0.0, 1.0, -4.0],
        );
        assert!(
            matches!(outcome, Err(Error::PlanesDoNotMeetInAPoint { .. })),
            "an infinite coefficient must stay refused, got {outcome:?}"
        );
    }

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
