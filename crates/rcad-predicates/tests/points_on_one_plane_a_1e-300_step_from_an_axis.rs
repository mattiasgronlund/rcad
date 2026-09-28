//! Points on one plane, or on one line, near the origin, one of them `1e-300`
//! off an axis — decided as exactly zero rather than refused.
//!
//! `docs/decisions.md` §324. The four points are the ones a fuse of two
//! `[0, 2]³` boxes asked about when one box had its corner `(2, 0, 0)` stored
//! as `(2, 1e-300, 0)` (§316.6): a triangle of that box's `y = 0` face and the
//! point `(0.5, 0, 0.5)`. The fuse refused `Undecidable` in both orders under
//! both rules on this one call.
//!
//! **Why zero is the right answer**, without asking the kernel: the first point
//! is the origin and the fourth is exactly a quarter of the third, so three of
//! the four lie on one line through the origin, and four points three of which
//! are on a line always lie on one plane. The same holds for the three points
//! in a line given to `orient2d`: the third is exactly twice the second, both
//! coordinates, and the first is the origin. Every order of the points is asked,
//! because which point the predicate subtracts from the rest changes every
//! difference it forms.
//!
//! **What refused, and why it no longer does.** Level 1 had nothing to offer —
//! its value is zero and its error bound fell subnormal — and level 2's
//! enclosure straddled zero. Level 3 would have summed the determinant to
//! exactly zero, but it declines any product below `2^-969`, and products like
//! `1e-300 * 0.5` are below that even though they are exact. The retry that
//! rescales by a power of two read only the largest *difference*, which is
//! already in `[1, 2)`, so it tried exponent zero and gave up. A second
//! exponent now lifts the largest *coordinate* as high as the top of the range
//! allows, and every product level 3 forms comes up with it.

use rcad_predicates::{Decision, Point, orient2d_reporting_level, orient3d_reporting_level};

#[test]
fn a_triangle_of_the_moved_boxs_face_and_a_point_on_that_face_are_on_one_plane() {
    let points = [
        [0.0, 0.0, 0.0],
        [2.0, 1e-300, 0.0],
        [2.0, 0.0, 2.0],
        [0.5, -0.0, 0.5],
    ];
    let mut asked = 0;
    for order in every_order_of_four() {
        let [a, b, c, d] = order.map(|index| Point::from(points[index]));
        assert_eq!(
            orient3d_reporting_level(a, b, c, d).0,
            Decision::Zero,
            "{a:?} {b:?} {c:?} {d:?}"
        );
        asked += 1;
    }
    assert_eq!(asked, 24);
}

fn every_order_of_four() -> impl Iterator<Item = [usize; 4]> {
    (0..256_usize)
        .map(|n| [n & 3, (n >> 2) & 3, (n >> 4) & 3, (n >> 6) & 3])
        .filter(|order| (0..4).all(|index| order.contains(&index)))
}

#[test]
fn three_points_on_a_line_through_the_origin_a_1e_300_step_off_the_x_axis() {
    let points = [[0.0, 0.0], [3.0, 1e-300], [6.0, 2e-300]];
    for [a, b, c] in [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
        assert_eq!(
            orient2d_reporting_level(points[a], points[b], points[c]).0,
            Decision::Zero,
            "{:?} {:?} {:?}",
            points[a],
            points[b],
            points[c]
        );
    }
}
