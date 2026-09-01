//! Which of two points on a crossing comes first.
//!
//! `docs/decisions.md` §31.2 row 8, the half of it that did not exist. A
//! straight crossing is ordered by a comparison on its direction's dominant
//! axis, and `rcad-topo`'s arrangement has done that since M1. A **closed** one
//! needs something else, because there is no dominant axis to climb and no end
//! to measure from.
//!
//! # A conic is convex, and that is the whole of the trick
//!
//! Three points on a convex closed curve run round it in the order they wind,
//! and which way they wind is the sign of the area they sweep. So no angle is
//! constructed anywhere, nothing is projected into the surface's own two
//! coordinates, and no `atan2` is called — the question comes down to one
//! determinant of three vectors, which is [`orient3d`] with the fourth point put
//! one normal away from the first (§15.5).
//!
//! # Which way round is "forward"
//!
//! The crossing of `S` and `T` runs along `∇S × ∇T`, which is §17.4's
//! derivation and the same fact §31.3 leans on for a closed edge's direction. So
//! the pair is ordered and the answer flips when the two surfaces are swapped —
//! that is the geometry and not a convention. For each pair the direction
//! `∇S × ∇T` points anticlockwise about the plane normal named below:
//!
//! | The pair | Anticlockwise about |
//! |---|---|
//! | a plane and a ball, or a plane and a pipe | the plane's stored normal |
//! | the same two the other way round | the same normal negated |
//! | two balls | the second centre minus the first |
//!
//! The pipe row takes a line of work to see and is worth having written down.
//! Its crossing lies in the cutting plane, its tangent is `n × ρ` for the
//! outward radial direction `ρ`, and whether that runs anticlockwise about `n`
//! is the sign of `u · ρ` for `u` the step from the ellipse's centre. Writing
//! `a` for the foot of the point on the axis, `u · ρ` is `‖u × ŵ‖²`, which is
//! above zero for every `u` in a plane the axis is not parallel to.

use rcad_predicates::{
    Decision, Filtered, Interval, NOTHING_IS_KNOWN, Point as PredicatePoint, Predicate,
    are_real_numbers, decide, is_a_real_point,
};

use crate::enclosure::{cross_enclosing, dot_enclosing, exactly};
use crate::exact::Exact;
use crate::{Error, Result, Surface, Vector};

/// The name the ordering question goes by in provenance records.
pub const ROUND_A_CROSSING: &str = "round a crossing";

/// Which of `one` and `other` comes first, going round the closed curve where
/// `pair` cross, starting from `from`.
///
/// [`Decision::Pos`] when `one` does, matching `order_along`'s convention over
/// in the arrangement. [`Decision::Neg`] when `other` does.
///
/// [`Decision::Zero`] when the three sweep no area, which on a conic means two
/// of them are the same place on it — a line meets a conic twice, so three
/// *different* points on one are never in a line.
///
/// All three points may be boxes, and usually are: a corner where three surfaces
/// meet is one. Where a box is wide enough to straddle another point's the
/// honest answer is [`Decision::Undecidable`], and the caller decides what to do
/// with it.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] for a pair whose crossing is
/// not one closed curve lying in a plane: two planes, a plane along a pipe's
/// axis, two pipes, and a ball against a pipe.
///
/// **"A plane along a pipe's axis" is decided here by a rounded `f64` and by
/// [`intersect`](crate::intersect) exactly**, so the two disagree about a plane
/// whose dot with the axis cancels to zero in `f64` and does not cancel
/// exactly. Such a pair *is* a conic, `intersect` names it one closed curve, and
/// this refuses it —
/// `a_conic_the_intersector_names_is_refused_here_because_the_two_ask_it_differently`
/// is the demonstration. A **gap**: the answer exists and is worked out
/// correctly one file away. `docs/decisions.md` §39.
pub fn which_comes_first_round_a_crossing(
    pair: [&Surface; 2],
    from: PredicatePoint,
    one: PredicatePoint,
    other: PredicatePoint,
) -> Result<Decision> {
    let anticlockwise_about = the_way_this_crossing_winds(pair)?;
    if !(is_a_real_point(from)
        && is_a_real_point(one)
        && is_a_real_point(other)
        && are_real_numbers(anticlockwise_about.components()))
    {
        return Ok(NOTHING_IS_KNOWN);
    }
    Ok(decide(&HowThreeOnACurveWind {
        from,
        one,
        other,
        anticlockwise_about: anticlockwise_about.components(),
    })
    .0)
}

/// The normal the crossing of these two surfaces runs anticlockwise about.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] for a pair that does not cross
/// in one closed curve lying in a plane.
fn the_way_this_crossing_winds(pair: [&Surface; 2]) -> Result<Vector> {
    let winds = match pair {
        [Surface::Plane(flat), Surface::Sphere(_)] => flat.normal(),
        [Surface::Sphere(_), Surface::Plane(flat)] => -flat.normal(),
        // A plane along a pipe's axis cuts it in straight lines, and those are
        // ordered by their dominant axis rather than round anything. `walk`
        // dispatches on the same comparison for the same reason.
        [Surface::Plane(flat), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Plane(flat)]
            if flat.normal().dot(pipe.axis().direction()) == 0.0 =>
        {
            return Err(Error::TheseTwoSurfacesDoNotCrossInAConic);
        }
        [Surface::Plane(flat), Surface::Cylinder(_)] => flat.normal(),
        [Surface::Cylinder(_), Surface::Plane(flat)] => -flat.normal(),
        // Two balls cross in a circle in their radical plane, whose normal is
        // the line between the centres. Two balls with the same centre cross in
        // nothing at all, and there is no such line.
        [Surface::Sphere(one), Surface::Sphere(other)] => other.centre() - one.centre(),
        // Two planes cross in a straight line; a ball against a pipe and two
        // pipes cross in a degree-four space curve that is neither flat nor
        // convex, so neither the determinant nor the argument behind it applies.
        _ => return Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
    };
    if winds == Vector::zero() {
        return Err(Error::TheseTwoSurfacesDoNotCrossInAConic);
    }
    Ok(winds)
}

/// `((one − from) × (other − from)) · anticlockwise_about`, evaluated the two
/// ways the cascade can use.
///
/// [`orient3d`](rcad_predicates::orient3d) with the fourth point at
/// `from + anticlockwise_about`, written as the determinant it is rather than
/// built as a point — because building that point would round it, and the whole
/// value of the exact level is that it does not.
struct HowThreeOnACurveWind {
    from: PredicatePoint,
    one: PredicatePoint,
    other: PredicatePoint,
    anticlockwise_about: [f64; 3],
}

impl Predicate for HowThreeOnACurveWind {
    fn name(&self) -> &'static str {
        ROUND_A_CROSSING
    }

    /// No level 1.
    ///
    /// The same choice `intersect`'s seven questions made and for the same
    /// reason (§27.1): a forward error bound is a derived claim that §15.6 says
    /// needs a test of its own, and level 2 decides everything level 1 would
    /// have, by construction rather than by derivation.
    fn on_floats(&self) -> Option<Filtered> {
        None
    }

    fn on_intervals(&self) -> Interval {
        let base = self.from.enclosure();
        let step = |to: PredicatePoint| {
            let there = to.enclosure();
            core::array::from_fn(|axis| there[axis] - base[axis])
        };
        dot_enclosing(
            cross_enclosing(step(self.one), step(self.other)),
            exactly(self.anticlockwise_about),
        )
    }

    fn exactly(&self) -> Option<Decision> {
        let base = self.from.exact_coordinates()?;
        let here = Exact::gap_between(base, self.one.exact_coordinates()?);
        let there = Exact::gap_between(base, self.other.exact_coordinates()?);

        let mut total = Exact::of(0.0);
        for axis in 0..3 {
            let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
            let across = here[next]
                .times(&there[after])?
                .minus(&here[after].times(&there[next])?);
            total = total.plus(&across.times(&Exact::of(self.anticlockwise_about[axis]))?);
        }

        match total.sign() {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            // A product overflowed on the way in. Exact arithmetic that is not
            // exact has nothing to offer, so this level cannot run.
            Decision::Undecidable { .. } => None,
            other => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Axis, Cylinder, Plane, Point, Sphere};

    fn a_ball(radius: f64) -> Surface {
        Surface::Sphere(Sphere::new(Point::origin(), radius).expect("a radius"))
    }

    fn the_equators_plane() -> Surface {
        Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        )
    }

    fn round_the_equator(radius: f64, eighths: i32) -> PredicatePoint {
        let angle = core::f64::consts::TAU * f64::from(eighths) / 8.0;
        PredicatePoint::from([radius * angle.cos(), radius * angle.sin(), 0.0])
    }

    /// The equator of a ball, cut by the plane through its middle. Going round
    /// from the point on `+x`, a quarter turn comes before a half turn.
    #[test]
    fn three_points_on_a_circle_come_back_in_the_order_they_run_round_it() {
        let pair = [the_equators_plane(), a_ball(5.0)];
        let along = [&pair[0], &pair[1]];
        let at = |eighths| round_the_equator(5.0, eighths);

        assert_eq!(
            which_comes_first_round_a_crossing(along, at(0), at(2), at(4)),
            Ok(Decision::Pos),
            "a quarter turn comes before a half turn"
        );
        assert_eq!(
            which_comes_first_round_a_crossing(along, at(0), at(4), at(2)),
            Ok(Decision::Neg),
            "and asking the other way round says so"
        );
        assert_eq!(
            which_comes_first_round_a_crossing(along, at(0), at(2), at(2)),
            Ok(Decision::Zero),
            "the same place twice sweeps no area"
        );
    }

    /// The order is the crossing's own, so swapping the two surfaces reverses
    /// it. That is §31.3's rule and not a convention this file gets to pick.
    #[test]
    fn swapping_the_two_surfaces_runs_the_same_three_points_the_other_way() {
        let pair = [the_equators_plane(), a_ball(5.0)];
        let at = |eighths| round_the_equator(5.0, eighths);
        assert_eq!(
            which_comes_first_round_a_crossing([&pair[0], &pair[1]], at(0), at(2), at(4)),
            Ok(Decision::Pos)
        );
        assert_eq!(
            which_comes_first_round_a_crossing([&pair[1], &pair[0]], at(0), at(2), at(4)),
            Ok(Decision::Neg)
        );
    }

    /// A bar's cap circle, which is the ellipse a plane cuts out of a pipe, and
    /// the case the pipe row of the table was worked out for.
    #[test]
    fn three_points_on_a_bars_cap_circle_come_back_in_the_order_they_run_round_it() {
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 4.0).expect("a radius"));
        let cap = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 7.0), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        );
        let at = |eighths: i32| {
            let angle = core::f64::consts::TAU * f64::from(eighths) / 8.0;
            PredicatePoint::from([4.0 * angle.cos(), 4.0 * angle.sin(), 7.0])
        };
        assert_eq!(
            which_comes_first_round_a_crossing([&cap, &pipe], at(0), at(1), at(3)),
            Ok(Decision::Pos)
        );
        assert_eq!(
            which_comes_first_round_a_crossing([&cap, &pipe], at(0), at(3), at(1)),
            Ok(Decision::Neg)
        );
    }

    /// Two balls cross in a circle in their radical plane, and the order runs
    /// anticlockwise about the line between the centres.
    #[test]
    fn three_points_on_the_circle_where_two_balls_cross_run_round_the_line_between_them() {
        let here = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
        let there = Surface::Sphere(Sphere::new(Point::new(6.0, 0.0, 0.0), 5.0).expect("a radius"));
        // The circle sits at x = 3 with radius 4.
        let at = |eighths: i32| {
            let angle = core::f64::consts::TAU * f64::from(eighths) / 8.0;
            PredicatePoint::from([3.0, 4.0 * angle.cos(), 4.0 * angle.sin()])
        };
        assert_eq!(
            which_comes_first_round_a_crossing([&here, &there], at(0), at(1), at(3)),
            Ok(Decision::Pos)
        );
        assert_eq!(
            which_comes_first_round_a_crossing([&there, &here], at(0), at(1), at(3)),
            Ok(Decision::Neg)
        );
    }

    /// The pairs whose crossing is not one closed curve in a plane are refused
    /// rather than ordered by a rule that does not hold for them.
    #[test]
    fn a_crossing_that_is_not_a_conic_is_refused_rather_than_ordered() {
        let ground = the_equators_plane();
        let wall = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
                .expect("the x axis is a unit normal"),
        );
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 4.0).expect("a radius"));
        let ball = a_ball(5.0);
        let somewhere = PredicatePoint::from([1.0, 0.0, 0.0]);

        for pair in [
            [&ground, &wall],
            // A plane down the pipe's own axis: two straight lines, not a conic.
            [&wall, &pipe],
            [&ball, &pipe],
            [&pipe, &pipe],
            // Two balls with the same centre have no line between them.
            [&ball, &ball],
        ] {
            assert_eq!(
                which_comes_first_round_a_crossing(pair, somewhere, somewhere, somewhere),
                Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
                "{pair:?} does not cross in a conic"
            );
        }
    }

    /// Two authorities on "is this plane along this pipe's axis", asking it two
    /// different ways and disagreeing.
    ///
    /// [`intersect`](crate::intersect) runs the certified cascade on the exact
    /// dot of the two stored triples and names one closed curve.
    /// [`the_way_this_crossing_winds`] runs `== 0.0` on the same two triples in
    /// plain `f64`, reads "two straight lines", and refuses. `(1, 0, 3)` against
    /// a plane facing `(−12, −12, 4)` is a pair of small whole numbers where the
    /// `f64` dot cancels and the exact one does not, and the two answers are as
    /// far apart as they can be.
    ///
    /// **A gap, and this test pins today's answer so a fix fails it loudly**
    /// (§19's shape). Closing it means the two asking the same question, which is
    /// §23.8, and it moves `walk`'s own dispatch too — `walk.rs` has the same
    /// `== 0.0` for the same reason — so it is a lane rather than a gate. §39
    /// records it.
    #[test]
    fn a_conic_the_intersector_names_is_refused_here_because_the_two_ask_it_differently() {
        let pipe = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::origin(), Vector::new(1.0, 0.0, 3.0)).expect("a direction"),
                2.0,
            )
            .expect("a radius"),
        );
        let leaning = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(-12.0, -12.0, 4.0))
                .expect("a normal with length"),
        );

        let Surface::Cylinder(along) = &pipe else {
            panic!("a pipe");
        };
        let direction = along.axis().direction();
        let normal = leaning.as_plane().expect("a plane").normal();
        assert_eq!(
            normal.dot(direction),
            0.0,
            "in plain f64 these two look exactly square"
        );
        assert_eq!(
            crate::exact::Exact::dot_of(normal.components(), direction.components())
                .as_ref()
                .map(crate::exact::Exact::sign),
            Some(Decision::Pos),
            "and exactly they are not, so the crossing is a conic"
        );

        let named = crate::intersect(
            &leaning,
            &pipe,
            &rcad_predicates::Policy::certified(),
            &mut rcad_predicates::ProvenanceLog::new(),
        )
        .expect("the intersector names this pair");
        assert!(
            matches!(named, crate::SurfaceMeet::Curves(ref runs) if runs.len() == 1),
            "the intersector says one closed curve: {named:?}"
        );

        let somewhere = PredicatePoint::from([1.0, 0.0, 0.0]);
        assert_eq!(
            which_comes_first_round_a_crossing([&leaning, &pipe], somewhere, somewhere, somewhere),
            Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
            "and this refuses the very same pair"
        );
    }

    /// A point known only as a box that straddles another's is refused rather
    /// than ordered, and the two comfortably apart are ordered.
    #[test]
    fn two_points_whose_boxes_overlap_are_not_told_apart() {
        let pair = [the_equators_plane(), a_ball(5.0)];
        let along = [&pair[0], &pair[1]];
        let smeared = |eighths: i32, by: f64| {
            let angle = core::f64::consts::TAU * f64::from(eighths) / 8.0;
            PredicatePoint::Constructed(
                [5.0 * angle.cos(), 5.0 * angle.sin(), 0.0]
                    .map(|part| Interval::new(part - by, part + by).expect("a range")),
            )
        };
        assert!(matches!(
            which_comes_first_round_a_crossing(
                along,
                smeared(0, 0.0),
                smeared(2, 9.0),
                smeared(3, 9.0)
            ),
            Ok(Decision::Undecidable { .. })
        ));
        assert_eq!(
            which_comes_first_round_a_crossing(
                along,
                smeared(0, 0.0),
                smeared(2, 1e-9),
                smeared(3, 1e-9)
            ),
            Ok(Decision::Pos)
        );
    }
}
