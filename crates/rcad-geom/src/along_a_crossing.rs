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
//! | a plane and a torus, cut in the two meridian circles | the plane's stored normal |
//! | a plane square to a torus's axis, the circle outside the ring | the plane's stored normal |
//! | a plane square to a torus's axis, the circle inside the ring | the same normal negated |
//! | any of those the other way round | the same answer negated |
//! | two balls | the second centre minus the first |
//!
//! The pipe row takes a line of work to see and is worth having written down.
//! Its crossing lies in the cutting plane, its tangent is `n × ρ` for the
//! outward radial direction `ρ`, and whether that runs anticlockwise about `n`
//! is the sign of `u · ρ` for `u` the step from the ellipse's centre. Writing
//! `a` for the foot of the point on the axis, `u · ρ` is `‖u × ŵ‖²`, which is
//! above zero for every `u` in a plane the axis is not parallel to.
//!
//! # The torus rows, and why one of them is upside down
//!
//! The same `sign(u · ρ)` settles a torus, with `ρ` now the step out of the
//! tube's own centre circle. On a meridian circle the tube's centre *is* the
//! circle's centre, so `u` and `ρ` are the same vector and `u · ρ = ‖u‖²`, above
//! zero on both of the two. On a circle round the axis the two part company: the
//! circle's centre is on the axis, so `‖u‖` is the distance out to the axis and
//! `u · ρ` comes to `‖u‖(‖u‖ − R)` for `R` the ring radius. The two circles sit
//! at `R ± √(r² − t²)`, one either side of `R`, so **the outer one winds about
//! the plane's normal and the inner one about its opposite**.
//!
//! Which of the two a run is on is not a fact about the pair, so it cannot be
//! read off the two surfaces the way every other row here is. It is read off
//! the point the ordering starts from, against the pipe of the ring's own radius
//! — the very pipe [`intersect`](crate::intersect) hands back as the picker that
//! tells the two circles apart.

use rcad_predicates::{
    Decision, Filtered, Interval, NOTHING_IS_KNOWN, Point as PredicatePoint, Predicate,
    SideOfSurface, are_real_numbers, decide, is_a_real_point,
};

use crate::enclosure::{cross_enclosing, dot_enclosing, exactly};
use crate::exact::Exact;
use crate::{
    Cylinder, Error, Plane, Result, Surface, Torus, Vector, WhereFollowingACrossingGaveUp,
};

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
/// axis, two pipes, a ball against a pipe, a plane cutting a torus in a quartic,
/// and every pair with a cone or a torus on it but a plane against either.
///
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] for a plane
/// resting on a torus's tube, which touches it all the way round one circle
/// instead of crossing it, so `∇S × ∇T` is zero everywhere on that circle and
/// there is no direction to run round it in.
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
    let anticlockwise_about = match the_way_this_crossing_winds(pair, from)? {
        WhichWayItWinds::About(normal) => normal,
        WhichWayItWinds::CannotTell => return Ok(NOTHING_IS_KNOWN),
    };
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

/// The normal a crossing runs anticlockwise about, when that can be worked out.
///
/// **Exhaustive on purpose.** There is a normal or there is not, and a caller
/// that cannot have one has nothing left to order by.
enum WhichWayItWinds {
    /// The crossing runs anticlockwise about this.
    About(Vector),
    /// Which of two circles the run is on could not be decided from the point
    /// the ordering starts at — the honest answer is that nothing is known, and
    /// not a refusal.
    CannotTell,
}

/// The normal the crossing of these two surfaces runs anticlockwise about.
///
/// `from` is a point on the crossing, and only the torus rows read it: a plane
/// square to a torus's axis cuts two circles that wind opposite ways, and which
/// one this is is not a fact about the pair.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] for a pair that does not cross
/// in one closed curve lying in a plane, and
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] for a plane
/// resting on a torus's tube.
///
/// # Every pair is named, and there is no `_`
///
/// This used to end in a wildcard, and a wildcard is where the next surface kind
/// goes to be ignored (§40). A cone and a torus landed and fell into it without
/// a word — the same thing that happened to `walk`'s three hand-kept lists
/// (§24.5), through a wildcard rather than through a missing arm.
fn the_way_this_crossing_winds(
    pair: [&Surface; 2],
    from: PredicatePoint,
) -> Result<WhichWayItWinds> {
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
        // A plane cuts a cone in a conic lying in that plane, exactly as it cuts
        // a pipe. The four shapes that are not one closed curve — a parabola, a
        // hyperbola, one ruling and two — never reach here, because
        // `where_a_plane_and_a_cone_cross` refuses each of them by name and no
        // run carrying one exists.
        [Surface::Plane(flat), Surface::Cone(_)] => flat.normal(),
        [Surface::Cone(_), Surface::Plane(flat)] => -flat.normal(),
        // Two balls cross in a circle in their radical plane, whose normal is
        // the line between the centres. Two balls with the same centre cross in
        // nothing at all, and there is no such line.
        [Surface::Sphere(one), Surface::Sphere(other)] => other.centre() - one.centre(),
        // Two planes cross in a straight line, which has nothing to wind about.
        [Surface::Plane(_), Surface::Plane(_)] => {
            return Err(Error::TheseTwoSurfacesDoNotCrossInAConic);
        }
        // A ball against a pipe and two pipes cross in a degree-four space curve
        // that is neither flat nor convex, so neither the determinant nor the
        // argument behind it applies.
        [Surface::Sphere(_), Surface::Cylinder(_)]
        | [
            Surface::Cylinder(_),
            Surface::Sphere(_) | Surface::Cylinder(_),
        ] => return Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
        // The two flat families a plane cuts a torus in are circles, and only
        // those two reach a normal — everything else a plane and a torus make is
        // a quartic and is refused by name, here as in
        // `intersect`(crate::intersect).
        [Surface::Plane(flat), Surface::Torus(ring)] => {
            return the_way_a_plane_cuts_a_torus(flat, ring, from);
        }
        [Surface::Torus(ring), Surface::Plane(flat)] => {
            return Ok(match the_way_a_plane_cuts_a_torus(flat, ring, from)? {
                WhichWayItWinds::About(normal) => WhichWayItWinds::About(-normal),
                WhichWayItWinds::CannotTell => WhichWayItWinds::CannotTell,
            });
        }
        // A cone or a torus against another curved surface: `intersect` refuses
        // to name that crossing at all, so there is no run along it to order.
        [
            Surface::Cone(_),
            Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_),
        ]
        | [
            Surface::Torus(_),
            Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_),
        ]
        | [
            Surface::Sphere(_) | Surface::Cylinder(_),
            Surface::Cone(_) | Surface::Torus(_),
        ] => return Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
    };
    if winds == Vector::zero() {
        return Err(Error::TheseTwoSurfacesDoNotCrossInAConic);
    }
    Ok(WhichWayItWinds::About(winds))
}

/// The normal the circle a plane cuts out of a torus runs anticlockwise about.
///
/// # Which family this is, is asked exactly
///
/// `‖n × w‖²` says whether the plane is square to the axis, and `n·w` with
/// `n·c − d` say whether it holds the axis — the same three signs
/// [`intersect`](crate::intersect) asks, on the same stored numbers, so the two
/// agree about which family a pair is in. They are asked here through
/// [`Exact`] rather than through the cascade because this function has no policy
/// and no log to run one with; every one of them is a polynomial in stored
/// numbers, so the exact level always decides and no rounding ever moves a pair
/// from one family to another.
///
/// **`walk` asks the first of the three in plain `f64`**, so a plane whose
/// `n × w` cancels to zero in `f64` and not exactly is a circle round the axis
/// there and a quartic here, and this refuses a run that one builds. A **gap**
/// of the shape §39.7 item 6 already records, on the same question rather than a
/// new one, and it goes the safe way: a refusal, never a wrong order.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] for every plane that cuts this
/// torus in a quartic, and
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] for a plane
/// resting on the tube, whose one circle the two surfaces touch along rather
/// than cross.
fn the_way_a_plane_cuts_a_torus(
    flat: &Plane,
    ring: &Torus,
    from: PredicatePoint,
) -> Result<WhichWayItWinds> {
    let normal = flat.normal().components();
    let along = ring.axis().direction().components();
    let centre = ring.centre().coordinates();
    if !(are_real_numbers(normal)
        && are_real_numbers(along)
        && are_real_numbers(centre)
        && are_real_numbers([flat.offset()]))
    {
        return Ok(WhichWayItWinds::CannotTell);
    }

    let Some(across) = Exact::cross_of(&Exact::of_each(normal), along) else {
        return Ok(WhichWayItWinds::CannotTell);
    };
    if across.iter().all(|part| part.sign() == Decision::Zero) {
        return the_way_a_circle_round_the_axis_winds(flat, ring, from);
    }

    let (Some(runs_along), Some(reaches_the_centre)) = (
        Exact::dot_of(normal, along),
        Exact::dot_of(normal, centre).map(|reach| reach.minus(&Exact::of(flat.offset()))),
    ) else {
        return Ok(WhichWayItWinds::CannotTell);
    };
    if runs_along.sign() != Decision::Zero || reaches_the_centre.sign() != Decision::Zero {
        return Err(Error::TheseTwoSurfacesDoNotCrossInAConic);
    }
    // A plane holding the axis cuts the two meridian circles, and each of them
    // is centred where the tube's own centre circle passes through the plane. So
    // `u` and `ρ` are one vector and `u · ρ` is a length squared, above zero on
    // both circles — which is why this row needs nothing read off `from`.
    Ok(WhichWayItWinds::About(flat.normal()))
}

/// Which way one of the two circles a plane square to a torus's axis cuts runs,
/// told apart by which side of the ring's own pipe `from` sits.
///
/// The pipe is the one [`intersect`](crate::intersect) hands back as the picker
/// on these two runs, built here from the same two numbers, so a run that names
/// the outer circle is ordered by the outer circle's normal. `Cylinder::new`
/// cannot fail on it: a torus's ring radius is above zero and its axis is finite
/// by construction, which is the claim `two_circles_round_the_axis` makes about
/// the same call.
fn the_way_a_circle_round_the_axis_winds(
    flat: &Plane,
    ring: &Torus,
    from: PredicatePoint,
) -> Result<WhichWayItWinds> {
    let ring_pipe = Cylinder::new(ring.axis(), ring.ring_radius())?;
    match ring_pipe.side_of(from) {
        Decision::Pos => Ok(WhichWayItWinds::About(flat.normal())),
        Decision::Neg => Ok(WhichWayItWinds::About(-flat.normal())),
        // On the pipe itself, which is where the two circles have run together
        // into the one the plane rests on the tube along. The surfaces touch
        // there instead of crossing, so `∇S × ∇T` is zero all the way round and
        // no ordering follows from it.
        Decision::Zero => Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
            surfaces: Box::new([Surface::Plane(*flat), Surface::Torus(*ring)]),
            at: WhereFollowingACrossingGaveUp::OrderingTheCircleAPlaneRestsOnARingAlong,
        }),
        Decision::Undecidable { .. } => Ok(WhichWayItWinds::CannotTell),
        other => unreachable!("which side of a pipe a point is on is a sign: {other:?}"),
    }
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

    /// **Swept two ways, and the overlap is the answer.** `(one − from) ×
    /// (other − from)` and `(one − from) × (other − one)` are one number for
    /// every three points, since the difference is `(one − from) × (one −
    /// from)`, which is nothing. So each enclosure holds every value the three
    /// boxes allow, and so does where they overlap.
    ///
    /// The second is the one that decides where `one` and `other` are close
    /// together and far from `from`. There each step from `from` rounds by an
    /// ulp of how far it reaches, and the first form multiplies that rounding
    /// by the other long step; the second subtracts two boxes that are close
    /// together and rounds by an ulp of the short step between them. The block
    /// leaned along a cone's ruling and turned `1` in
    /// `a_plane_cut_along_a_cones_other_nappe.rs` cuts the cone in an ellipse
    /// whose start is `2.2e16` out, and two nodes `3.9` apart near the frustum
    /// swept from it straddle zero by `1.9e17` (§406.1).
    fn on_intervals(&self) -> Interval {
        let [from, one, other] = [self.from, self.one, self.other].map(PredicatePoint::enclosure);
        let gap = |here: [Interval; 3], there: [Interval; 3]| -> [Interval; 3] {
            core::array::from_fn(|axis| there[axis] - here[axis])
        };
        let about = exactly(self.anticlockwise_about);
        let swept_from_the_start =
            dot_enclosing(cross_enclosing(gap(from, one), gap(from, other)), about);
        let swept_from_one = dot_enclosing(cross_enclosing(gap(from, one), gap(one, other)), about);
        where_two_enclosures_of_one_number_overlap(swept_from_the_start, swept_from_one)
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

/// Where two enclosures of one number overlap.
///
/// # Panics
///
/// If they do not. Two boxes that each hold the same number share at least that
/// number, so an empty overlap is a broken enclosure.
fn where_two_enclosures_of_one_number_overlap(one: Interval, other: Interval) -> Interval {
    Interval::new(one.lo().max(other.lo()), one.hi().min(other.hi())).unwrap_or_else(|_| {
        panic!("two enclosures of the same number do not overlap: {one:?} and {other:?}")
    })
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

    /// A ring of four and one about `+z`, centred on the origin. Its outer
    /// equator is at radius five, its inner at three, and it reaches one unit
    /// either side of `z = 0`.
    fn a_ring() -> Surface {
        Surface::Torus(crate::Torus::new(Axis::z(), 4.0, 1.0).expect("a tube inside its ring"))
    }

    /// The way out of the tube at a point on [`a_ring`], which is the torus's
    /// own gradient there up to a factor above zero.
    fn out_of_the_tube_at(point: [f64; 3]) -> Vector {
        let [x, y, z] = point;
        let out = (x * x + y * y).sqrt();
        Vector::new(x - 4.0 * x / out, y - 4.0 * y / out, z)
    }

    /// That the step from `at` to `next` runs the way the crossing of a plane
    /// facing `facing` and [`a_ring`] does, which is `n × ∇T` (§17.4).
    ///
    /// Every torus test below asserts this before it asserts an order, so what
    /// they pin is the geometry and not a convention this file picked.
    fn the_step_runs_the_way_the_two_surfaces_cross(facing: Vector, at: [f64; 3], next: [f64; 3]) {
        let runs = facing.cross(out_of_the_tube_at(at));
        let step = Vector::new(next[0] - at[0], next[1] - at[1], next[2] - at[2]);
        assert!(
            runs.dot(step) > 0.0,
            "the crossing runs {runs:?} at {at:?} and the step to {next:?} is {step:?}"
        );
    }

    /// The cap of a bend of pipe: one of the two circles a plane holding the
    /// ring's axis cuts. Both of them wind about the plane's own normal, so the
    /// same listing orders both.
    #[test]
    fn three_points_on_the_cap_of_a_bend_come_back_in_the_order_they_run_round_it() {
        let ring = a_ring();
        let facing = Vector::new(0.0, 1.0, 0.0);
        let cap = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), facing).expect("a unit normal"),
        );
        // Anticlockwise about `+y` from the outside of the ring, for the circle
        // centred `out` along `+x`.
        let round = |out: f64, eighths: i32| {
            let angle = core::f64::consts::TAU * f64::from(eighths) / 8.0;
            [out + angle.cos(), 0.0, -angle.sin()]
        };

        for out in [4.0, -4.0] {
            let at = |eighths| PredicatePoint::from(round(out, eighths));
            the_step_runs_the_way_the_two_surfaces_cross(facing, round(out, 0), round(out, 1));
            assert_eq!(
                which_comes_first_round_a_crossing([&cap, &ring], at(0), at(1), at(3)),
                Ok(Decision::Pos),
                "the circle {out} along the x axis"
            );
            assert_eq!(
                which_comes_first_round_a_crossing([&cap, &ring], at(0), at(3), at(1)),
                Ok(Decision::Neg)
            );
            assert_eq!(
                which_comes_first_round_a_crossing([&ring, &cap], at(0), at(1), at(3)),
                Ok(Decision::Neg),
                "and the pair the other way round runs the other way"
            );
        }
    }

    /// The two circles a plane square to the axis cuts are concentric and lie in
    /// one plane, and they wind **opposite ways** — the outer about the plane's
    /// normal and the inner about its opposite. So the same three points listed
    /// anticlockwise about that normal are in order on one and out of order on
    /// the other, which is the whole reason this row reads the point it starts
    /// from.
    #[test]
    fn the_two_circles_round_a_rings_axis_run_opposite_ways() {
        let ring = a_ring();
        let facing = Vector::new(0.0, 0.0, 1.0);
        let through = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), facing).expect("a unit normal"),
        );
        let round = |radius: f64, eighths: i32| {
            let angle = core::f64::consts::TAU * f64::from(eighths) / 8.0;
            [radius * angle.cos(), radius * angle.sin(), 0.0]
        };

        let outer = |eighths| PredicatePoint::from(round(5.0, eighths));
        the_step_runs_the_way_the_two_surfaces_cross(facing, round(5.0, 0), round(5.0, 1));
        assert_eq!(
            which_comes_first_round_a_crossing([&through, &ring], outer(0), outer(1), outer(3)),
            Ok(Decision::Pos)
        );
        assert_eq!(
            which_comes_first_round_a_crossing([&through, &ring], outer(0), outer(3), outer(1)),
            Ok(Decision::Neg)
        );

        // The same eighths on the inner circle, which the crossing runs round
        // the other way.
        let inner = |eighths| PredicatePoint::from(round(3.0, eighths));
        the_step_runs_the_way_the_two_surfaces_cross(facing, round(3.0, 0), round(3.0, -1));
        assert_eq!(
            which_comes_first_round_a_crossing([&through, &ring], inner(0), inner(1), inner(3)),
            Ok(Decision::Neg)
        );
        assert_eq!(
            which_comes_first_round_a_crossing([&through, &ring], inner(0), inner(3), inner(1)),
            Ok(Decision::Pos)
        );
    }

    /// A plane resting on the tube touches the ring all the way round one circle
    /// instead of crossing it. `intersect` names that circle, and `∇S × ∇T` is
    /// zero at every point of it, so there is no direction to order along and
    /// this refuses.
    ///
    /// A **limit** and not a gap: the answer does not exist. The family of fixes
    /// is a convention picked out of thin air — the plane's own two basis
    /// vectors, say — and a convention is not the crossing's direction, which is
    /// what §17.4 says a coedge is oriented by.
    #[test]
    fn a_plane_resting_on_a_rings_tube_has_no_direction_to_be_ordered_along() {
        let ring = a_ring();
        let resting = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 1.0), Vector::new(0.0, 0.0, 1.0))
                .expect("a unit normal"),
        );
        let on_it = PredicatePoint::from([4.0, 0.0, 1.0]);
        assert_eq!(
            which_comes_first_round_a_crossing([&resting, &ring], on_it, on_it, on_it),
            Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
                surfaces: Box::new([resting, ring]),
                at: WhereFollowingACrossingGaveUp::OrderingTheCircleAPlaneRestsOnARingAlong,
            })
        );
        assert_eq!(
            out_of_the_tube_at([4.0, 0.0, 1.0]).cross(Vector::new(0.0, 0.0, 1.0)),
            Vector::zero(),
            "the two surfaces have one gradient between them there"
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
        let cone = Surface::Cone(crate::Cone::new(Axis::z(), 1.0).expect("a slope"));
        let bend = a_ring();
        // Parallel to the ring's axis but off it, which cuts a quartic.
        let beside = Surface::Plane(
            Plane::from_point_and_normal(Point::new(2.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
                .expect("the x axis is a unit normal"),
        );
        // Leaning across it, which cuts a quartic too.
        let leaning = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 1.0))
                .expect("a normal with a length"),
        );
        let somewhere = PredicatePoint::from([1.0, 0.0, 0.0]);

        for pair in [
            [&ground, &wall],
            // A plane down the pipe's own axis: two straight lines, not a conic.
            [&wall, &pipe],
            [&ball, &pipe],
            [&pipe, &pipe],
            // Two balls with the same centre have no line between them.
            [&ball, &ball],
            // A cone or a torus against another curved surface, which
            // `intersect` refuses to name a crossing for at all.
            [&cone, &ball],
            [&ball, &cone],
            [&cone, &pipe],
            [&cone, &cone],
            [&bend, &ball],
            [&bend, &pipe],
            [&bend, &cone],
            [&bend, &bend],
            // The planes that cut a torus in a quartic rather than in circles,
            // both ways round. The two that cut circles are ordered, three
            // tests up.
            [&beside, &bend],
            [&bend, &beside],
            [&leaning, &bend],
            [&bend, &leaning],
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
    /// plain `f64`, reads "two straight lines", and refuses. `(2, 0, 3)` against
    /// a plane facing `(−12, −7, 8)` is a pair of small whole numbers where the
    /// `f64` dot cancels and the exact one does not, and the two answers are as
    /// far apart as they can be.
    ///
    /// **This witness has dissolved twice and the gap has not moved once.** It
    /// was `(1, 0, 3)` against `(−12, −12, 4)` until §120 rounded the stored axis
    /// correctly and that pair's `f64` dot came out at `−2.8e-17`; it was
    /// `(2, 0, 3)` against `(−12, −12, 8)` until §121 rounded the stored normal
    /// correctly and the *exact* dot of that pair came out at zero, which is a
    /// pair the two authorities agree about. A sweep of small whole-number pipes
    /// against small whole-number planes still finds dozens where the `f64` dot
    /// cancels and the exact one does not — for this one pipe alone, more than
    /// twenty normals inside `±12` — and this is one of them.
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
                Axis::new(Point::origin(), Vector::new(2.0, 0.0, 3.0)).expect("a direction"),
                2.0,
            )
            .expect("a radius"),
        );
        let leaning = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(-12.0, -7.0, 8.0))
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

    /// Two meets `3.9` apart by the frustum, swept from where the ellipse a
    /// block's top cuts in the cone starts, `2.2e16` out: the block leaned
    /// along a ruling and turned `1` in `a_plane_cut_along_a_cones_other_nappe.rs`,
    /// by its stored numbers. Swept from the start the two steps each round by
    /// an ulp of `2.2e16` and the sweep straddles nothing by `1.9e17`; swept
    /// from `one` it is decided, and it is the sign the two middles give
    /// exactly (§406.1).
    #[test]
    fn two_points_close_together_far_from_the_start_are_ordered_round_a_long_ellipse() {
        let boxed = |sides: [[f64; 2]; 3]| {
            PredicatePoint::Constructed(
                sides.map(|[lo, hi]| Interval::new(lo, hi).expect("a range")),
            )
        };
        let from = PredicatePoint::from([
            -6052001170552803.0,
            -9425433034941358.0,
            2.2402277211438692e16,
        ]);
        let one = boxed([
            [-5.676794340559188, -5.67679434055918],
            [0.41299523114051584, 0.41299523114052195],
            [11.383595046548264, 11.383595046548281],
        ]);
        let other = boxed([
            [-3.8082163550265475, -3.8082163550265435],
            [1.2237190009590946, 1.2237190009590968],
            [8.0, 8.0],
        ]);
        let anticlockwise_about = [0.48326107372841764, 0.7526345292496597, 0.447213595499958];
        let swept = HowThreeOnACurveWind {
            from,
            one,
            other,
            anticlockwise_about,
        };

        let step = |to: PredicatePoint| -> [Interval; 3] {
            core::array::from_fn(|axis| to.enclosure()[axis] - from.enclosure()[axis])
        };
        assert!(
            dot_enclosing(
                cross_enclosing(step(one), step(other)),
                exactly(anticlockwise_about)
            )
            .contains_zero(),
            "swept from the start alone the two are already told apart"
        );

        let middle = |point: PredicatePoint| {
            PredicatePoint::from(
                point
                    .enclosure()
                    .map(|side| 0.5 * side.lo() + 0.5 * side.hi()),
            )
        };
        let at_the_middles = decide(&HowThreeOnACurveWind {
            from,
            one: middle(one),
            other: middle(other),
            anticlockwise_about,
        })
        .0;
        assert!(matches!(at_the_middles, Decision::Pos | Decision::Neg));
        assert_eq!(decide(&swept).0, at_the_middles);
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
