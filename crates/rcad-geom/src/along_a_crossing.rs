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
//! The one loop a ball off a pipe's axis crosses it in, where the ball reaches
//! into the pipe's side, is not read that way. It is ordered by how far round
//! a line each point is, and that comes down to determinants too, with no
//! angle worked out (`HowTwoPointsSitRoundALine`).
//!
//! # Which way round is "forward"
//!
//! The crossing of `S` and `T` runs along `∇S × ∇T`, which is §17.4's
//! derivation and the same fact §31.3 leans on for a closed edge's direction. So
//! the pair is ordered and the answer flips when the two surfaces are swapped —
//! that is the geometry and not a convention. For each pair the direction
//! `∇S × ∇T` points anticlockwise about the direction named below:
//!
//! | The pair | Anticlockwise about |
//! |---|---|
//! | a plane and a ball, or a plane and a pipe | the plane's stored normal |
//! | a plane and a torus, cut in the two meridian circles | the plane's stored normal |
//! | a plane square to a torus's axis, the circle outside the ring | the plane's stored normal |
//! | a plane square to a torus's axis, the circle inside the ring | the same normal negated |
//! | a ball and a pipe whose whole cross-section it swallows, the circle or loop ahead of the centre along the axis | the pipe's stored axis direction |
//! | the same, the circle or loop behind the centre | the same direction negated |
//! | a ball off a pipe's axis that reaches into its side, the one loop | the step from the ball's centre back to the axis, going round the line along that step |
//! | a cone and a pipe, a shallower cone or a ball on its axis, the circle ahead of the picker | the cone's stored axis direction negated |
//! | the same, the circle behind the picker | the cone's stored axis direction |
//! | any of those the other way round | the same answer negated |
//! | two balls | the second centre minus the first |
//! | two cones of one slope on one axis | the first apex minus the second |
//! | a thin pipe and a fat one swallowing it, the loop where the thin one comes out | the thin pipe's stored axis direction negated |
//! | the same, the loop where it goes in | the thin pipe's stored axis direction |
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
//!
//! # A ball and a pipe through it are the same shape again
//!
//! A ball on a pipe's axis crosses it in two circles either side of the
//! ball's centre, square to the axis, and they wind opposite ways: `∇S × ∇T`
//! there is `h ŵ × ρ`, for `h` how far along the axis the circle sits from the
//! centre. So which way a run winds is read off the point it starts from too,
//! against the half-space through the centre facing along the axis, which is
//! `intersect`'s picker on those two runs. A ball off the axis that swallows
//! the pipe's whole cross-section crosses it in two quartic loops instead, one
//! either side of the same half-space. Each runs once round the pipe's axis,
//! so the determinant reads three points of one down the axis, as it does for
//! two pipes below, and `∇S × ∇T` there is `h ŵ × ρ` and a part along the axis
//! that does not change which way it reads (Q1554).
//! `the_way_a_loop_round_a_pipe_through_a_ball_winds` has the product. A ball
//! that reaches into the pipe's side crosses it in one loop that runs back and
//! forth round the axis. Every half-plane edged by the line through the ball's
//! centre square to the axis crosses that loop once, so it is ordered by how
//! far round that line each point is (Q1767).
//! `the_way_the_one_loop_a_ball_off_a_pipes_axis_crosses_it_in_winds` has the
//! argument. Whether the ball is off the axis, whether it swallows the pipe,
//! and whether it reaches into its side are each asked exactly.
//!
//! # So is a cone on one axis with a pipe, a second cone or a ball
//!
//! Each crosses it in circles square to the axis, and `∇S × ∇T` on one of them
//! is a multiple of `ŵ × ρ` whose sign changes at one plane square to the axis:
//! the cone's apex against a pipe, the steeper apex for two cones, and the
//! point `intersect` puts its picker through for a ball. So it is read off the
//! point the ordering starts from against that same picker, and the circle
//! ahead of it along the cone's axis winds about that axis negated.
//! `the_way_a_circle_round_a_cones_axis_winds` has the product. Two cones of
//! one slope cross in one circle halfway between their apexes and need no
//! picker.
//!
//! # Two pipes, where one swallows the other, are not a conic and still order
//!
//! A fat pipe crosses a thin one whose whole cross-section it swallows in two
//! quartic loops, neither flat. Each runs once round the thin pipe's axis, so
//! looked at down that axis three points of one are three points of the thin
//! pipe's circle in the same order, and the determinant reads them down it:
//! the circle is what is convex. The loop where the thin pipe comes out of the
//! fat one winds about the thin axis negated, and which loop a run is on is
//! read off the point it starts from, against the plane through the fat axis
//! that `intersect`'s picker puts between them (Q1483).
//! `the_way_a_loop_round_a_thin_pipe_winds` has the product. Two pipes that
//! cross in one loop run back and forth round both axes, and are refused.

use core::cmp::Ordering;

use rcad_predicates::{
    Decision, Filtered, Interval, NOTHING_IS_KNOWN, Point as PredicatePoint, Predicate,
    SideOfSurface, are_real_numbers, decide, is_a_real_point,
};

use crate::enclosure::{cross_enclosing, dot_enclosing, exactly, gap_enclosing};
use crate::exact::Exact;
use crate::intersect::{
    halfway_between_a_cones_apex_and_a_ball_on_its_axis, how_a_ball_on_a_cones_axis_meets_it,
    whether_a_ball_runs_through_a_cones_apex, whether_a_ball_sits_on_a_cones_axis,
    whether_a_cone_and_a_pipe_share_an_axis, whether_two_cones_share_an_axis,
};
use crate::{
    Cone, Cylinder, Error, HalfSpace, Plane, Point, Result, Sphere, Surface, Torus, Vector,
    WhereFollowingACrossingGaveUp,
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
/// On the one loop a ball off a pipe's axis crosses it in, which is ordered
/// round a line, a point at `from`'s own angle round the line comes first, and
/// [`Decision::Zero`] means `one` and `other` are the same place on the loop.
///
/// All three points may be boxes, and usually are: a corner where three surfaces
/// meet is one. Where a box is wide enough to straddle another point's the
/// honest answer is [`Decision::Undecidable`], and the caller decides what to do
/// with it.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] for a pair whose crossing is
/// not one closed curve lying in a plane, nor one of the two loops a fat pipe
/// crosses a thin one it swallows whole in, nor one of the two loops a ball
/// crosses a pipe it swallows whole in, nor the one loop a ball off a pipe's
/// axis crosses it in where it reaches into its side: two planes, a plane
/// along a pipe's axis, two pipes of one radius, two running the same way, two
/// whose crossing is one loop, a ball off a pipe's axis that misses it,
/// touches it at one point, or touches it from inside in a figure of eight, a
/// plane cutting a torus in a quartic, every pair with a torus
/// on it but a plane, a cone against a pipe, a second cone or a ball off its
/// axis, two cones of one apex, and a ball on a cone's axis that misses it or
/// runs through its apex.
///
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] for a plane
/// resting on a torus's tube, which touches it all the way round one circle
/// instead of crossing it, so `∇S × ∇T` is zero everywhere on that circle and
/// there is no direction to run round it in — and for a ball inscribed in a
/// pipe or in a cone, which touches it round one circle the same way.
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
    let winds = the_way_this_crossing_winds(pair, from)?;
    if !(is_a_real_point(from) && is_a_real_point(one) && is_a_real_point(other)) {
        return Ok(NOTHING_IS_KNOWN);
    }
    match winds {
        WhichWayItWinds::About(normal) if are_real_numbers(normal.components()) => {
            Ok(decide(&HowThreeOnACurveWind {
                from,
                one,
                other,
                anticlockwise_about: normal.components(),
            })
            .0)
        }
        WhichWayItWinds::About(_) | WhichWayItWinds::CannotTell => Ok(NOTHING_IS_KNOWN),
        WhichWayItWinds::RoundALine(line) => {
            Ok(which_comes_first_round_a_line(line, from, one, other))
        }
    }
}

/// What a crossing runs round, when that can be worked out.
///
/// **Exhaustive on purpose.** There is a normal, a line, or neither, and a
/// caller that has neither has nothing left to order by.
enum WhichWayItWinds {
    /// The crossing runs anticlockwise about this.
    About(Vector),
    /// The crossing is the one loop a ball off a pipe's axis crosses it in,
    /// which runs back and forth round the pipe's axis, about no normal, and
    /// once round this line.
    RoundALine(RoundALine),
    /// Which of two circles the run is on could not be decided from the point
    /// the ordering starts at — the honest answer is that nothing is known, and
    /// not a refusal.
    CannotTell,
}

impl WhichWayItWinds {
    /// The answer for the same two surfaces handed over the other way round,
    /// whose `∇S × ∇T` is the other way round too.
    fn the_other_way(self) -> Self {
        match self {
            Self::About(normal) => Self::About(-normal),
            Self::RoundALine(line) => Self::RoundALine(RoundALine {
                backwards: !line.backwards,
                ..line
            }),
            Self::CannotTell => Self::CannotTell,
        }
    }
}

/// The line through a ball's centre `c` that meets a pipe's axis square to it,
/// and which way round it the one loop the two cross in runs.
///
/// Every half-plane with this line for its edge crosses the loop once, so
/// going round the loop goes once round the line, and three points of the loop
/// are ordered by how far round the line each one is.
#[derive(Clone, Copy, Debug)]
struct RoundALine {
    centre: [f64; 3],
    /// A point of the pipe's axis.
    on_the_axis: [f64; 3],
    /// The pipe's axis direction `w`.
    along: [f64; 3],
    /// `false` for the pair taken ball first, which runs anticlockwise about
    /// the step from the centre back to the axis.
    backwards: bool,
}

/// The normal the crossing of these two surfaces runs anticlockwise about, or
/// the line it runs round.
///
/// `from` is a point on the crossing, and only the rows with two curves read
/// it: a plane square to a torus's axis, a ball on a pipe's axis, and a cone on
/// one axis with a pipe, a cone or a ball each cut two circles that wind
/// opposite ways, and a fat pipe cuts a thin one it swallows, and a ball a pipe
/// it swallows off its axis, in two loops that do, and which one this is is not
/// a fact about the pair.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] for a pair that does not cross
/// in one closed curve lying in a plane, in one of the two loops round a
/// pipe's axis that a fat pipe or a ball swallowing it crosses it in, or in
/// the one loop round a line that a ball off a pipe's axis reaching into its
/// side crosses it in, and
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] for a plane
/// resting on a torus's tube and a ball inscribed in a pipe or a cone.
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
        // A ball centred on a pipe's axis crosses it in two circles square to
        // the axis, a ball off it that swallows the pipe's whole cross-section
        // in two loops round the axis, and a ball off it that reaches into its
        // side in one loop round a line through the ball's centre. A ball off
        // it that misses it, touches it at one point, or touches it from
        // inside in a figure of eight is refused inside.
        [Surface::Sphere(ball), Surface::Cylinder(pipe)] => {
            return the_way_a_circle_round_a_pipes_axis_winds(ball, pipe, from);
        }
        [Surface::Cylinder(pipe), Surface::Sphere(ball)] => {
            return Ok(the_way_a_circle_round_a_pipes_axis_winds(ball, pipe, from)?.the_other_way());
        }
        // A cone against a pipe, a second cone or a ball on its axis crosses it
        // in circles square to that axis, and off the axis in a quartic,
        // refused inside.
        [Surface::Cone(cone), Surface::Cylinder(pipe)] => {
            return the_way_a_cone_and_a_pipe_on_its_axis_wind(cone, pipe, from);
        }
        [Surface::Cylinder(pipe), Surface::Cone(cone)] => {
            return Ok(
                the_way_a_cone_and_a_pipe_on_its_axis_wind(cone, pipe, from)?.the_other_way(),
            );
        }
        [Surface::Cone(one), Surface::Cone(other)] => {
            return the_way_two_cones_on_one_axis_wind(one, other, from);
        }
        [Surface::Cone(cone), Surface::Sphere(ball)] => {
            return the_way_a_cone_and_a_ball_on_its_axis_wind(cone, ball, from);
        }
        [Surface::Sphere(ball), Surface::Cone(cone)] => {
            return Ok(
                the_way_a_cone_and_a_ball_on_its_axis_wind(cone, ball, from)?.the_other_way(),
            );
        }
        // Two pipes cross in a degree-four space curve that is neither flat nor
        // convex. Where one swallows the other's whole cross-section, each of
        // its two loops runs once round the thin pipe's axis, and that is what
        // orders it; every other pair of pipes is refused inside.
        [Surface::Cylinder(one), Surface::Cylinder(other)] => {
            return match one.radius().total_cmp(&other.radius()) {
                Ordering::Less => the_way_a_loop_round_a_thin_pipe_winds(one, other, from),
                Ordering::Greater => {
                    Ok(the_way_a_loop_round_a_thin_pipe_winds(other, one, from)?.the_other_way())
                }
                Ordering::Equal => Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
            };
        }
        // The two flat families a plane cuts a torus in are circles, and only
        // those two reach a normal — everything else a plane and a torus make is
        // a quartic and is refused by name, here as in
        // `intersect`(crate::intersect).
        [Surface::Plane(flat), Surface::Torus(ring)] => {
            return the_way_a_plane_cuts_a_torus(flat, ring, from);
        }
        [Surface::Torus(ring), Surface::Plane(flat)] => {
            return Ok(the_way_a_plane_cuts_a_torus(flat, ring, from)?.the_other_way());
        }
        // A torus against another curved surface: `intersect` refuses to name
        // that crossing at all, so there is no run along it to order.
        [
            Surface::Torus(_),
            Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_),
        ]
        | [
            Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_),
            Surface::Torus(_),
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

/// Which way one of the two circles a ball centred on a pipe's axis crosses it
/// in runs, for the pair taken ball first.
///
/// # Which circle, and why the two wind opposite ways
///
/// With the centre `c` on the axis, a point `p` of the crossing is `ρ + hŵ` on
/// from `c`, for `ρ` the step out from the axis and `h` how far along it `p`
/// sits. So `∇S × ∇T` is `(ρ + hŵ) × ρ = h ŵ × ρ`, and `ŵ × ρ` runs
/// anticlockwise about `ŵ`: the circle at `h = +√(R² − r²)` winds about the
/// axis direction and the one at `−√(R² − r²)` about its opposite.
///
/// Which of the two a run is on is read off the point the ordering starts from,
/// against the half-space through the centre facing along the axis — the very
/// one [`intersect`](crate::intersect) hands back as the picker on these two
/// runs, built here from the same two numbers, so a run that names the circle
/// in front of it is ordered by that circle's normal. The torus row reads its
/// two circles apart the same way.
///
/// # Whether the centre is on the axis is asked exactly
///
/// `(c − o) × w` through [`Exact`], for the reason
/// [`the_way_a_plane_cuts_a_torus`] gives: there is no policy here to run a
/// cascade with, and the exact level decides every pair it can multiply out.
/// That is the sign `intersect` splits this pair on, and the one the
/// arrangement asks before it lets the pair through to be ordered.
///
/// A centre off the axis, by however little, crosses the pipe in quartic loops
/// and not circles, two where the ball swallows the pipe's whole cross-section
/// and one where it reaches into its side, and
/// [`the_way_a_loop_round_a_pipe_through_a_ball_winds`] orders both.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] for a ball whose centre is off
/// the axis and that touches the pipe from inside, misses it, or touches it at
/// one point, from [`the_way_a_loop_round_a_pipe_through_a_ball_winds`].
///
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] for `from` on the
/// plane through the centre, which is on the crossing only where the ball is
/// inscribed in the pipe and touches it round that one circle. `h` is nothing
/// there, so `∇S × ∇T` is nothing all the way round.
fn the_way_a_circle_round_a_pipes_axis_winds(
    ball: &Sphere,
    pipe: &Cylinder,
    from: PredicatePoint,
) -> Result<WhichWayItWinds> {
    let centre = ball.centre().coordinates();
    let origin = pipe.axis().origin().coordinates();
    let along = pipe.axis().direction().components();
    if !(are_real_numbers(centre) && are_real_numbers(origin) && are_real_numbers(along)) {
        return Ok(WhichWayItWinds::CannotTell);
    }
    let Some(across) = Exact::cross_of(&Exact::gap_between(origin, centre), along) else {
        return Ok(WhichWayItWinds::CannotTell);
    };
    let mut off_the_axis = Decision::Zero;
    for part in &across {
        match part.sign() {
            Decision::Zero => {}
            Decision::Pos | Decision::Neg => {
                return the_way_a_loop_round_a_pipe_through_a_ball_winds(ball, pipe, &across, from);
            }
            undecided @ Decision::Undecidable { .. } => off_the_axis = undecided,
            other => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
        }
    }
    if off_the_axis != Decision::Zero {
        return Ok(WhichWayItWinds::CannotTell);
    }

    let between = HalfSpace::through_a_point_facing(ball.centre(), pipe.axis().direction())?;
    match between.side_of_point(from) {
        Decision::Pos => Ok(WhichWayItWinds::About(between.normal())),
        Decision::Neg => Ok(WhichWayItWinds::About(-between.normal())),
        Decision::Zero => Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
            surfaces: Box::new([Surface::Sphere(*ball), Surface::Cylinder(*pipe)]),
            at: WhereFollowingACrossingGaveUp::CrossingTheTwoNormalsForADirection,
        }),
        Decision::Undecidable { .. } => Ok(WhichWayItWinds::CannotTell),
        other => unreachable!("which side of a half-space a point is on is a sign: {other:?}"),
    }
}

/// Which way the loop `from` is on runs, of the two a ball crosses a pipe in
/// whose axis misses its centre, for the pair taken ball first. `across` is
/// `(c − o) × w`, already known not to be nothing.
///
/// # Each loop runs once round the pipe's axis
///
/// Write a point `p` of the crossing as `c + hŵ + ρ − d`, with `d` the step
/// out from the axis to the centre, `a = ‖d‖`, `ρ` the step out from the axis
/// to `p` and `h` how far along the axis `p` sits from the centre. On the ball,
/// `h² = R² − ‖ρ − d‖²`, and `‖ρ − d‖ ≤ r + a`. Where `R > r + a` that is above
/// nothing for every `ρ`, so each line of the pipe's wall along its axis
/// crosses the ball twice, once either side of the plane through the centre
/// square to the axis, and each loop meets each line once: going round a loop
/// goes once round the axis. Three points of a loop, looked at down `w`, are
/// three points of the pipe's circle in the same order, and the module's
/// determinant reads them down `w`.
///
/// # Which way round
///
/// `∇S × ∇T` is `(hŵ + ρ − d) × ρ = h ŵ × ρ − d × ρ`. `d × ρ` lies along `ŵ`,
/// since both are square to it, so against `ŵ × ρ`, which runs anticlockwise
/// about `ŵ`, the crossing runs `h r²`: exactly the circles' reading, and the
/// loop ahead of the centre winds about the axis direction and the one behind
/// about its opposite. Which loop a run is on is read off `from` against the
/// same half-space through the centre, which is the picker
/// [`intersect`](crate::intersect) hands back on these two runs.
///
/// A ball that does not swallow the pipe's whole cross-section, `R − r < a`,
/// and still reaches into its side crosses it in one loop instead, and
/// [`the_way_the_one_loop_a_ball_off_a_pipes_axis_crosses_it_in_winds`] orders
/// that.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] where `R − r = a`, a ball that
/// touches the pipe from inside and crosses it in a figure of eight, and from
/// [`the_way_the_one_loop_a_ball_off_a_pipes_axis_crosses_it_in_winds`] for a
/// ball that misses the pipe or touches it at one point.
fn the_way_a_loop_round_a_pipe_through_a_ball_winds(
    ball: &Sphere,
    pipe: &Cylinder,
    across: &[Exact; 3],
    from: PredicatePoint,
) -> Result<WhichWayItWinds> {
    match whether_a_ball_swallows_a_pipe(ball, pipe, across) {
        Some(Decision::Pos) => {}
        Some(Decision::Zero) => return Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
        Some(Decision::Neg) => {
            return the_way_the_one_loop_a_ball_off_a_pipes_axis_crosses_it_in_winds(
                ball, pipe, across,
            );
        }
        Some(Decision::Undecidable { .. }) | None => return Ok(WhichWayItWinds::CannotTell),
        Some(other) => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
    }
    let between = HalfSpace::through_a_point_facing(ball.centre(), pipe.axis().direction())?;
    match between.side_of_point(from) {
        Decision::Pos => Ok(WhichWayItWinds::About(between.normal())),
        Decision::Neg => Ok(WhichWayItWinds::About(-between.normal())),
        // No point of either loop is on the plane through the centre, so which
        // loop `from` is on cannot be told, and the answer is the one a box
        // straddling it gets.
        Decision::Zero | Decision::Undecidable { .. } => Ok(WhichWayItWinds::CannotTell),
        other => unreachable!("which side of a half-space a point is on is a sign: {other:?}"),
    }
}

/// The sign of `(R − r)² W − A` for the ball's radius `R` above the pipe's
/// `r`, with `W = w · w` and `A = ‖across‖² = ‖(c − o) × w‖²`. That is
/// `W (R − r − a)(R − r + a)` for `a` how far the centre is off the axis, so
/// its sign is the sign of `R − r − a`: above nothing where the ball swallows
/// the pipe's whole cross-section. `None` where a product leaves what [`Exact`]
/// can hold.
fn whether_a_ball_swallows_a_pipe(
    ball: &Sphere,
    pipe: &Cylinder,
    across: &[Exact; 3],
) -> Option<Decision> {
    let wider = Exact::of(ball.radius()).minus(&Exact::of(pipe.radius()));
    if wider.sign() != Decision::Pos {
        return Some(Decision::Neg);
    }
    let along = pipe.axis().direction().components();
    Some(
        wider
            .times(&wider)?
            .times(&Exact::dot_of(along, along)?)?
            .minus(&Exact::squared_length_of(across)?)
            .sign(),
    )
}

/// The line the one loop runs round, of a ball off a pipe's axis that does not
/// swallow the pipe's whole cross-section, for the pair taken ball first.
/// `across` is `(c − o) × w`, already known not to be nothing.
///
/// # Every half-plane edged by the line crosses the loop once
///
/// Put the axis along `z` with the centre at `(a, 0, 0)`, so the line is the
/// `x` axis. The loop is there where `|r − a| < R < r + a`. A half-plane edged
/// by the line at angle `φ` round it cuts the pipe in half an ellipse centred
/// at the axis, of height squared `(r² − x²) / cos² φ` above the line, and the
/// ball in half a circle centred at `c`, of height squared `R² − (x − a)²`.
/// Both are there for `x` from `a − R`, where the circle starts inside the
/// pipe, to `r`, where the ellipse ends inside the ball. The ellipse's height
/// squared less the circle's is a quadratic in `x` whose `x²` is taken
/// `−tan² φ` times, so it is concave; it is above nothing at `a − R` and below
/// it at `r`, so it is nothing once, and crosses there rather than touching.
/// So the loop is a smooth graph over the angle round the line, the line
/// misses it, and the loop runs in the order of that angle.
///
/// # Which way round
///
/// With `X = (c − o) × w`, `X × w` is `−W d` for `d` the step out from the axis
/// to the centre, so `(p − c) · w` and `(p − c) · X` are coordinates square to
/// `d` that turn anticlockwise about `−d`. At the top of the loop, `(r, 0, z)`
/// with `z` above nothing, `∇S × ∇T = (p − c) × ρ` is `(0, rz, 0)`, which runs
/// anticlockwise about `−x`, the step from the centre back to the axis.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] where the ball is proved to
/// miss the pipe or to touch it at one point, `R ≤ |r − a|`.
fn the_way_the_one_loop_a_ball_off_a_pipes_axis_crosses_it_in_winds(
    ball: &Sphere,
    pipe: &Cylinder,
    across: &[Exact; 3],
) -> Result<WhichWayItWinds> {
    match whether_a_balls_equator_misses_a_pipes_circle(ball, pipe, across) {
        Some(Decision::Neg) => Ok(WhichWayItWinds::RoundALine(RoundALine {
            centre: ball.centre().coordinates(),
            on_the_axis: pipe.axis().origin().coordinates(),
            along: pipe.axis().direction().components(),
            backwards: false,
        })),
        Some(Decision::Zero | Decision::Pos) => Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
        Some(Decision::Undecidable { .. }) | None => Ok(WhichWayItWinds::CannotTell),
        Some(other) => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
    }
}

/// The sign of `V² − 4r²AW`, with `V = (r² − R²)W + A` and `W` and `A` as in
/// [`whether_a_ball_swallows_a_pipe`]. That is `W²` times `M² − 4a²r²` for
/// `M = r² + a² − R²`, which is `W² ((r − a)² − R²)((r + a)² − R²)`: the
/// sign of whether the ball's circle through its centre square to the axis
/// misses the pipe's circle in that plane. Below nothing where the two cross
/// twice, which with a ball that does not swallow the pipe's cross-section is
/// the one loop. `None` where a product leaves what [`Exact`] can hold.
fn whether_a_balls_equator_misses_a_pipes_circle(
    ball: &Sphere,
    pipe: &Cylinder,
    across: &[Exact; 3],
) -> Option<Decision> {
    let along = pipe.axis().direction().components();
    let squared_along = Exact::dot_of(along, along)?;
    let squared_across = Exact::squared_length_of(across)?;
    let pipe_squared = Exact::product_of(pipe.radius(), pipe.radius())?;
    let ball_squared = Exact::product_of(ball.radius(), ball.radius())?;
    let reach = pipe_squared
        .minus(&ball_squared)
        .times(&squared_along)?
        .plus(&squared_across);
    let crossed = Exact::of(4.0)
        .times(&pipe_squared)?
        .times(&squared_across)?
        .times(&squared_along)?;
    Some(reach.square_minus(&crossed)?.sign())
}

/// Which way one of the two circles a cone and a pipe on its axis cross in
/// runs, for the pair taken cone first.
///
/// The pipe is a cone of slope nothing in the argument on
/// [`the_way_a_circle_round_a_cones_axis_winds`], so the circle ahead of the
/// apex along the cone's axis winds about the axis direction negated, and the
/// one behind about the direction itself. The picker is the half-space through
/// the apex facing along the cone's axis, which is the one
/// [`intersect`](crate::intersect) hands back on these two runs.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] where the two are proved not
/// to share an axis: they cross in a quartic. Asked by the same exact question
/// `walk` asks before it draws the two circles.
fn the_way_a_cone_and_a_pipe_on_its_axis_wind(
    cone: &Cone,
    pipe: &Cylinder,
    from: PredicatePoint,
) -> Result<WhichWayItWinds> {
    if let Some(not_a_circle) =
        unless_proved_on_one_axis(whether_a_cone_and_a_pipe_share_an_axis(cone, pipe))
    {
        return not_a_circle;
    }
    the_way_a_circle_round_a_cones_axis_winds(cone, cone.apex(), from)
}

/// Which way the circle two cones on one axis cross in runs, for the pair in
/// the order handed over.
///
/// Two slopes cross in two circles, and the steeper cone winds them the way
/// [`the_way_a_circle_round_a_cones_axis_winds`] says, read against the
/// half-space through its apex: the picker
/// [`intersect`](crate::intersect) hands back on those two runs. One slope
/// crosses in one circle halfway between the apexes, which winds about the
/// first apex less the second.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] where the two are proved not
/// to share an axis, and where they share an apex: two slopes there meet at the
/// apex and nowhere else, and one slope is one cone. Neither is a circle.
fn the_way_two_cones_on_one_axis_wind(
    one: &Cone,
    other: &Cone,
    from: PredicatePoint,
) -> Result<WhichWayItWinds> {
    if let Some(not_a_circle) =
        unless_proved_on_one_axis(whether_two_cones_share_an_axis(one, other))
    {
        return not_a_circle;
    }
    if one.apex() == other.apex() {
        return Err(Error::TheseTwoSurfacesDoNotCrossInAConic);
    }
    match one.slope().total_cmp(&other.slope()) {
        // `σ₂s − σ₁s` with the circle at `h₁ = d/2` and `h₂ = −d/2`, so `σ₁` is
        // the sign of `d` and `σ₂` the other one: the circle winds about `−d ŵ`,
        // which is the way from the second apex to the first.
        Ordering::Equal => Ok(WhichWayItWinds::About(one.apex() - other.apex())),
        Ordering::Greater => the_way_a_circle_round_a_cones_axis_winds(one, one.apex(), from),
        Ordering::Less => Ok(
            the_way_a_circle_round_a_cones_axis_winds(other, other.apex(), from)?.the_other_way(),
        ),
    }
}

/// Which way one of the circles a ball on a cone's axis crosses it in runs, for
/// the pair taken cone first.
///
/// The picker is the half-space square to the axis through
/// [`halfway_between_a_cones_apex_and_a_ball_on_its_axis`](crate::intersect::halfway_between_a_cones_apex_and_a_ball_on_its_axis),
/// the one function [`intersect`](crate::intersect) builds its own half-space
/// through on these two runs, so the picker can never drift from the circle
/// `intersect` hands back. That point is exactly where
/// [`the_way_a_circle_round_a_cones_axis_winds`]'s sign for a ball changes, so
/// the circle ahead of it winds about the cone's axis direction negated and the
/// one behind about the direction itself.
///
/// # Errors
///
/// The three [`intersect`](crate::intersect) and `walk` give the pair, asked by
/// the same exact questions:
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] where the centre is proved off
/// the axis, which is a quartic; where the ball misses the cone, which crosses
/// in nothing; and where the ball's skin runs through the apex, which is one
/// circle and the apex apart from it.
///
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] for a ball
/// inscribed in the cone, which touches it round one circle at `h = e/(1 + s²)`,
/// where `∇S × ∇T` is nothing all the way round — a limit, the same one as a
/// ball inscribed in a pipe.
fn the_way_a_cone_and_a_ball_on_its_axis_wind(
    cone: &Cone,
    ball: &Sphere,
    from: PredicatePoint,
) -> Result<WhichWayItWinds> {
    if let Some(not_a_circle) =
        unless_proved_on_one_axis(whether_a_ball_sits_on_a_cones_axis(cone, ball))
    {
        return not_a_circle;
    }
    match how_a_ball_on_a_cones_axis_meets_it(cone, ball) {
        Decision::Pos => {}
        Decision::Neg => return Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
        Decision::Zero => {
            return Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
                surfaces: Box::new([Surface::Cone(*cone), Surface::Sphere(*ball)]),
                at: WhereFollowingACrossingGaveUp::CrossingTheTwoNormalsForADirection,
            });
        }
        Decision::Undecidable { .. } => return Ok(WhichWayItWinds::CannotTell),
        other => unreachable!("how much room a ball leaves round a cone is a sign: {other:?}"),
    }
    match whether_a_ball_runs_through_a_cones_apex(cone, ball) {
        Decision::Pos | Decision::Neg => {}
        Decision::Zero => return Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
        Decision::Undecidable { .. } => return Ok(WhichWayItWinds::CannotTell),
        other => unreachable!("where a ball stands against an apex is a sign: {other:?}"),
    }
    let halfway = halfway_between_a_cones_apex_and_a_ball_on_its_axis(cone, ball);
    the_way_a_circle_round_a_cones_axis_winds(cone, halfway, from)
}

/// `None` where the pair is proved to share an axis, and otherwise what the
/// ordering answers: a refusal where it is proved not to, and nothing known
/// where neither could be shown.
fn unless_proved_on_one_axis(sharing: Decision) -> Option<Result<WhichWayItWinds>> {
    match sharing {
        Decision::Zero => None,
        Decision::Pos => Some(Err(Error::TheseTwoSurfacesDoNotCrossInAConic)),
        Decision::Undecidable { .. } => Some(Ok(WhichWayItWinds::CannotTell)),
        Decision::Neg => {
            unreachable!("how far off an axis is a sum of squares, never below nothing")
        }
        other => unreachable!("unhandled decision: {other:?}"),
    }
}

/// Which way the circle round `cone`'s axis that `from` is on runs, for the
/// pair taken cone first, told apart by the half-space through `through` facing
/// along the cone's axis.
///
/// # The sign of one product says which way
///
/// Write `ŵ` for the unit axis, `ρ` for the step out from it to a point of the
/// circle, and `h` for how far along it the point sits from a surface's own
/// apex or centre. A cone of slope `s` has `∇ = ρ − s²h ŵ` up to a factor above
/// nothing, a pipe the same with `s = 0`, and a ball `ρ + h ŵ`. So both
/// gradients are `αρ̂ + βŵ` with `α = ‖ρ‖`, and
/// `∇S × ∇T = (β_S α_T − α_S β_T) ŵ × ρ̂`, where `ŵ × ρ̂` runs anticlockwise
/// about `ŵ`. With `S` the cone and `σ` the sign of each surface's own `h`:
///
/// | `T` | `β_S α_T − α_S β_T` | below nothing where |
/// |---|---|---|
/// | a pipe | `−s ‖ρ‖² σ_S` | the circle is ahead of the apex |
/// | a shallower cone | `‖ρ‖²(s_T σ_T − s σ_S)` | the circle is ahead of the steep apex |
/// | a ball centred `e` up the axis from the apex | `‖ρ‖(e − (1 + s²)h)` | the circle is ahead of `h = e/(1 + s²)` |
///
/// The first two rows use `s²h = s σ_S ‖ρ‖` on the cone, and the second that
/// `s_T < s` puts `s_T σ_T − s σ_S` on the side of `−σ_S` whatever `σ_T` is. The
/// third has `h` from the apex for both, the ball's own being `h − e`. So in every row
/// the circle ahead of the picker winds about `−ŵ` and the one behind about
/// `ŵ`, and the picker's stored normal **is** the cone's stored axis direction,
/// for the reason `two_circles_round_a_cones_axis` gives in `intersect`.
///
/// # On the picker's own plane, nothing is known
///
/// [`Decision::Zero`] puts `from` on a plane that holds no point of either
/// circle: through a cone's apex the cone is that one point, which the pipe and
/// the shallower cone miss, and halfway between a ball's two circles is where
/// they would have run together, which the discriminant has already said they
/// do not. Which circle `from` is on cannot be told, so the answer is the one a
/// box straddling the plane gets.
fn the_way_a_circle_round_a_cones_axis_winds(
    cone: &Cone,
    through: Point,
    from: PredicatePoint,
) -> Result<WhichWayItWinds> {
    let between = HalfSpace::through_a_point_facing(through, cone.axis().direction())?;
    match between.side_of_point(from) {
        Decision::Pos => Ok(WhichWayItWinds::About(-between.normal())),
        Decision::Neg => Ok(WhichWayItWinds::About(between.normal())),
        Decision::Zero | Decision::Undecidable { .. } => Ok(WhichWayItWinds::CannotTell),
        other => unreachable!("which side of a half-space a point is on is a sign: {other:?}"),
    }
}

/// Which way the loop `from` is on runs, of the two a fat pipe crosses a thin
/// one in, for the pair taken thin pipe first.
///
/// # Each loop runs once round the thin pipe's axis
///
/// Look down the fat pipe's axis. It is a circle of radius `R`, and each line
/// of the thin pipe's wall along its own axis is a line across that circle, a
/// distance from its centre between `δ − r` and `δ + r`, for `δ` how far apart
/// the two axes are and `r` the thin pipe's radius. Where `R > δ + r` every
/// one of those lines crosses the circle twice and touches it nowhere, once
/// going in and once coming out. So each loop meets each line of the wall
/// once, and going round a loop goes once round the thin pipe's axis `w`.
///
/// So three points of a loop, looked at down `w`, are three points of the
/// thin pipe's circle in the same order round it, and the sweep
/// `((one − from) × (other − from)) · w` only reads them down `w`: a circle is
/// convex, and the module's determinant orders it.
///
/// # Which way round
///
/// Write `ρ_t` and `ρ_f` for the steps out from each axis to a point of the
/// loop, which are the two gradients up to factors above nothing. The loop
/// runs along `ρ_t × ρ_f`, and against `w × ρ_t`, which is anticlockwise about
/// `w`, that is `(ρ_t × ρ_f) · (w × ρ_t) = −r² (ρ_f · w)`. So the loop where
/// the thin pipe comes out of the fat one, `ρ_f · w > 0`, winds about `−w`,
/// and the loop where it goes in winds about `w`. Each loop crosses every line
/// of the wall the same way, so the sign holds all the way round it, and
/// `ρ_f · w` has the sign of `((from − o_f) × w_f) · (w × w_f)`, which is
/// what is asked at `from`. [`intersect`](crate::intersect)'s picker on these
/// two runs is a plane through the fat axis that `w`'s part across that axis
/// is square to, so the loop it keeps in front is the one coming out.
///
/// # Errors
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] where the fat pipe is proved
/// not to swallow the thin one, `R − r ≤ δ`, asked exactly as `(R − r)² K ≤
/// T²` with `K = ‖w × w_f‖²` and `T = (o_f − o_t) · (w × w_f)`: two pipes
/// running the same way cross in lines, and two whose crossing is one loop
/// run back and forth round both axes.
fn the_way_a_loop_round_a_thin_pipe_winds(
    thin: &Cylinder,
    fat: &Cylinder,
    from: PredicatePoint,
) -> Result<WhichWayItWinds> {
    let [thin_at, fat_at] = [thin, fat].map(|pipe| pipe.axis().origin().coordinates());
    let [thin_way, fat_way] = [thin, fat].map(|pipe| pipe.axis().direction().components());
    if !(are_real_numbers(thin_at)
        && are_real_numbers(fat_at)
        && are_real_numbers(thin_way)
        && are_real_numbers(fat_way))
    {
        return Ok(WhichWayItWinds::CannotTell);
    }
    match whether_a_fat_pipe_swallows_a_thin_one(thin, fat) {
        Some(Decision::Pos) => {}
        Some(Decision::Zero | Decision::Neg) => {
            return Err(Error::TheseTwoSurfacesDoNotCrossInAConic);
        }
        Some(Decision::Undecidable { .. }) | None => return Ok(WhichWayItWinds::CannotTell),
        Some(other) => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
    }
    match whether_a_thin_pipe_comes_out_of_a_fat_one_at(from, thin_way, fat_at, fat_way) {
        Decision::Pos => Ok(WhichWayItWinds::About(-thin.axis().direction())),
        Decision::Neg => Ok(WhichWayItWinds::About(thin.axis().direction())),
        // On the plane through the fat axis that the thin one's direction is
        // square to there is no point of either loop, so which loop `from` is
        // on cannot be told, and the answer is the one a box straddling it
        // gets.
        Decision::Zero | Decision::Undecidable { .. } => Ok(WhichWayItWinds::CannotTell),
        other => unreachable!("unhandled decision: {other:?}"),
    }
}

/// The sign of `(R − r)² K − T²` for the fat pipe's radius `R` above the thin
/// one's `r`, which is `K (R − r − δ)(R − r + δ)` and so the sign of
/// `R − r − δ`: above nothing where the fat pipe swallows the thin one's whole
/// cross-section. `None` where a product leaves what [`Exact`] can hold.
fn whether_a_fat_pipe_swallows_a_thin_one(thin: &Cylinder, fat: &Cylinder) -> Option<Decision> {
    let wider = Exact::of(fat.radius()).minus(&Exact::of(thin.radius()));
    if wider.sign() != Decision::Pos {
        return Some(Decision::Neg);
    }
    let across = Exact::cross_of(
        &Exact::of_each(thin.axis().direction().components()),
        fat.axis().direction().components(),
    )?;
    let squared_across = Exact::squared_length_of(&across)?;
    let gap = Exact::gap_between(
        thin.axis().origin().coordinates(),
        fat.axis().origin().coordinates(),
    );
    let reach = Exact::dot_of_two_exact(&gap, &across)?;
    Some(
        wider
            .times(&wider)?
            .times(&squared_across)?
            .minus(&reach.times(&reach)?)
            .sign(),
    )
}

/// The sign of `((at − o_f) × w_f) · (w × w_f)`, which is `‖w_f‖²` times
/// `ρ_f · w`: above nothing where the thin pipe, running along `w`, is on its
/// way out of the fat one at `at`. Over `at`'s box first, and exactly where
/// that straddles nothing and `at` is a point.
fn whether_a_thin_pipe_comes_out_of_a_fat_one_at(
    at: PredicatePoint,
    thin_way: [f64; 3],
    fat_at: [f64; 3],
    fat_way: [f64; 3],
) -> Decision {
    let out = at.enclosure();
    let from_the_axis: [Interval; 3] =
        core::array::from_fn(|axis| out[axis] - Interval::exactly(fat_at[axis]));
    let boxed = dot_enclosing(
        cross_enclosing(from_the_axis, exactly(fat_way)),
        cross_enclosing(exactly(thin_way), exactly(fat_way)),
    )
    .sign();
    if !matches!(boxed, Decision::Undecidable { .. }) {
        return boxed;
    }
    let Some(point) = at.exact_coordinates() else {
        return boxed;
    };
    let exactly_out =
        Exact::cross_of(&Exact::gap_between(fat_at, point), fat_way).and_then(|out| {
            let across = Exact::cross_of(&Exact::of_each(thin_way), fat_way)?;
            Exact::dot_of_two_exact(&out, &across)
        });
    exactly_out.map_or(boxed, |out| out.sign())
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

/// Which of `one` and `other` comes first round `line` from `from`: the answer
/// [`which_comes_first_round_a_crossing`] gives on the one loop a ball off a
/// pipe's axis crosses it in.
///
/// How far round the line a point is is never worked out as an angle. Each of
/// the two is put in the half turn ahead of `from` or the half turn after it,
/// by how it turns round the line from `from`. Two in different halves are
/// ordered by that alone, and two in one half by how they turn from each
/// other, which within half a turn is the order they come in.
///
/// A point at `from`'s own angle comes first, as the arrangement's own start
/// does, and [`Decision::Zero`] means `one` and `other` are the same place on
/// the loop. A point the two readings cannot place gets the answer a box
/// straddling it gets: how far either of the two is from the other is not
/// what was measured, so the enclosure is not handed on.
fn which_comes_first_round_a_line(
    line: RoundALine,
    from: PredicatePoint,
    one: PredicatePoint,
    other: PredicatePoint,
) -> Decision {
    match [one, other].map(|place| how_far_round(line, from, place)) {
        [HowFarRound::AtTheStart, HowFarRound::AtTheStart] => Decision::Zero,
        [HowFarRound::CannotTell, _] | [_, HowFarRound::CannotTell] => NOTHING_IS_KNOWN,
        [HowFarRound::AtTheStart, _]
        | [HowFarRound::LessThanHalfWay, HowFarRound::HalfWayOrMore] => Decision::Pos,
        [_, HowFarRound::AtTheStart]
        | [HowFarRound::HalfWayOrMore, HowFarRound::LessThanHalfWay] => Decision::Neg,
        [HowFarRound::LessThanHalfWay, HowFarRound::LessThanHalfWay]
        | [HowFarRound::HalfWayOrMore, HowFarRound::HalfWayOrMore] => {
            // Plain variants and no `name @ (...)`: `just check-wildcards` does
            // not see a variant named inside one, and counts the `other` arm
            // after it as a wildcard over it (§40).
            let turned = line.turns(one, other);
            match turned {
                Decision::Zero => {
                    let faced = line.faces(one, other);
                    match faced {
                        Decision::Pos => Decision::Zero,
                        Decision::Undecidable { .. } => NOTHING_IS_KNOWN,
                        // Both are off the line and in one half turn, which
                        // holds no two points half a turn apart.
                        Decision::Zero | Decision::Neg => unreachable!(
                            "two points in one half turn round a line, at one angle, face apart: \
                             {faced:?}"
                        ),
                        other => unreachable!("unhandled decision: {other:?}"),
                    }
                }
                Decision::Pos | Decision::Neg | Decision::Undecidable { .. } => turned,
                other => unreachable!("unhandled decision: {other:?}"),
            }
        }
    }
}

/// How far round a [`RoundALine`] a point is from where the ordering starts.
///
/// **Exhaustive on purpose.** A turn starting at nothing splits into these and
/// nothing else.
#[derive(Clone, Copy, Debug)]
enum HowFarRound {
    /// At the start's own angle, nothing round.
    AtTheStart,
    /// Above nothing and under half a turn round.
    LessThanHalfWay,
    /// Half a turn round or more, and under a whole one.
    HalfWayOrMore,
    /// Too near the start's angle, or the angle half a turn from it, to tell,
    /// or on the line itself, which has no angle round it.
    CannotTell,
}

/// How far round `line` `place` is from `from`.
fn how_far_round(line: RoundALine, from: PredicatePoint, place: PredicatePoint) -> HowFarRound {
    match line.turns(from, place) {
        Decision::Pos => HowFarRound::LessThanHalfWay,
        Decision::Neg => HowFarRound::HalfWayOrMore,
        Decision::Zero => match line.faces(from, place) {
            Decision::Pos => HowFarRound::AtTheStart,
            Decision::Neg => HowFarRound::HalfWayOrMore,
            // The loop misses the line, but the arrangement asks about a step
            // ahead of the start too, and that can be on it.
            Decision::Zero | Decision::Undecidable { .. } => HowFarRound::CannotTell,
            other => unreachable!("unhandled decision: {other:?}"),
        },
        Decision::Undecidable { .. } => HowFarRound::CannotTell,
        other => unreachable!("unhandled decision: {other:?}"),
    }
}

impl RoundALine {
    /// [`Decision::Pos`] where `ahead` is under half a turn ahead of `place`
    /// round the line, the way the loop runs, [`Decision::Neg`] where it is
    /// under half a turn behind, and [`Decision::Zero`] where the two are at
    /// one angle or half a turn apart.
    fn turns(self, place: PredicatePoint, ahead: PredicatePoint) -> Decision {
        let [first, second] = if self.backwards {
            [ahead, place]
        } else {
            [place, ahead]
        };
        decide(&HowTwoPointsSitRoundALine {
            line: self,
            first,
            second,
            asks: WhatIsAsked::HowTheyTurn,
        })
        .0
    }

    /// [`Decision::Pos`] where two points that [`turns`](Self::turns) put at
    /// one angle or half a turn apart are at one angle, and [`Decision::Neg`]
    /// where they are half a turn apart. Nothing where either is on the line.
    fn faces(self, first: PredicatePoint, second: PredicatePoint) -> Decision {
        decide(&HowTwoPointsSitRoundALine {
            line: self,
            first,
            second,
            asks: WhatIsAsked::WhetherTheyFaceOneWay,
        })
        .0
    }
}

/// How two points sit round a [`RoundALine`], evaluated the two ways the
/// cascade can use.
///
/// A point `p` is read as `s = (p − c) · w` and `t = (p − c) · X`, for
/// `X = (c − o) × w`: two coordinates square to the line, which turn
/// anticlockwise about the step from the centre back to the axis. How two
/// points turn is `t₁ s₂ − s₁ t₂`, which is `W ((p₁ − c) × (p₂ − c)) · (−d)`,
/// a volume like the sweep [`HowThreeOnACurveWind`] works out. Whether they
/// face one way is `s₁ s₂ + t₁ t₂`, whose two terms are of different units,
/// since `X` is a length long and `w` is not; so it is a sign and not a size,
/// and it is asked only where the two turn by nothing. There the two steps
/// out from the line lie along one line, and stretching one coordinate does
/// not change the sign.
struct HowTwoPointsSitRoundALine {
    line: RoundALine,
    first: PredicatePoint,
    second: PredicatePoint,
    asks: WhatIsAsked,
}

/// Which of the two readings of [`HowTwoPointsSitRoundALine`] is asked.
///
/// **Exhaustive on purpose.** These two decide where a point is round a
/// line, and nothing else is asked of two points there.
#[derive(Clone, Copy, Debug)]
enum WhatIsAsked {
    HowTheyTurn,
    WhetherTheyFaceOneWay,
}

impl Predicate for HowTwoPointsSitRoundALine {
    fn name(&self) -> &'static str {
        ROUND_A_CROSSING
    }

    /// No level 1, for the reason [`HowThreeOnACurveWind`] gives.
    fn on_floats(&self) -> Option<Filtered> {
        None
    }

    /// How the two turn is swept two ways, and the overlap is the answer, as
    /// in [`HowThreeOnACurveWind`]: `t₁ s₂ − s₁ t₂` is `t₁ (s₂ − s₁) −
    /// s₁ (t₂ − t₁)` for every two points, and the second reads the short step
    /// between two points close together rather than two long ones from the
    /// centre.
    fn on_intervals(&self) -> Interval {
        let along = exactly(self.line.along);
        let out = cross_enclosing(
            gap_enclosing(self.line.on_the_axis, self.line.centre),
            along,
        );
        let gap = |here: [Interval; 3], there: [Interval; 3]| -> [Interval; 3] {
            core::array::from_fn(|axis| there[axis] - here[axis])
        };
        let read = |step: [Interval; 3]| [dot_enclosing(step, along), dot_enclosing(step, out)];
        let centre = exactly(self.line.centre);
        let [first, second] = [self.first, self.second].map(PredicatePoint::enclosure);
        let [s_first, t_first] = read(gap(centre, first));
        let [s_second, t_second] = read(gap(centre, second));
        match self.asks {
            WhatIsAsked::HowTheyTurn => {
                let [s_step, t_step] = read(gap(first, second));
                where_two_enclosures_of_one_number_overlap(
                    t_first * s_second - s_first * t_second,
                    t_first * s_step - s_first * t_step,
                )
            }
            WhatIsAsked::WhetherTheyFaceOneWay => s_first * s_second + t_first * t_second,
        }
    }

    fn exactly(&self) -> Option<Decision> {
        let out = Exact::cross_of(
            &Exact::gap_between(self.line.on_the_axis, self.line.centre),
            self.line.along,
        )?;
        let along = Exact::of_each(self.line.along);
        let read = |point: PredicatePoint| -> Option<[Exact; 2]> {
            let step = Exact::gap_between(self.line.centre, point.exact_coordinates()?);
            Some([
                Exact::dot_of_two_exact(&step, &along)?,
                Exact::dot_of_two_exact(&step, &out)?,
            ])
        };
        let [s_first, t_first] = read(self.first)?;
        let [s_second, t_second] = read(self.second)?;
        let value = match self.asks {
            WhatIsAsked::HowTheyTurn => t_first.times(&s_second)?.minus(&s_first.times(&t_second)?),
            WhatIsAsked::WhetherTheyFaceOneWay => {
                s_first.times(&s_second)?.plus(&t_first.times(&t_second)?)
            }
        };
        match value.sign() {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            // A product overflowed on the way in, as in `HowThreeOnACurveWind`.
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

    /// A ball of five centred on the origin and a pipe of four up `z`, which
    /// cross in two circles of radius four at `z = ±3`.
    fn a_ball_on_a_pipes_axis() -> (Surface, Surface) {
        (
            a_ball(5.0),
            Surface::Cylinder(Cylinder::new(Axis::z(), 4.0).expect("a radius")),
        )
    }

    /// The point `eighths` of a turn round the circle of radius four at height
    /// `z`, anticlockwise about `+z`.
    fn round_the_pipe(z: f64, eighths: i32) -> [f64; 3] {
        let angle = core::f64::consts::TAU * f64::from(eighths) / 8.0;
        [4.0 * angle.cos(), 4.0 * angle.sin(), z]
    }

    /// That the step from `at` to `next` runs the way the crossing of a ball
    /// centred on the origin and a pipe up `z` does: `∇S × ∇T`, with `∇S` the
    /// way out of the ball, `at` itself, and `∇T` the way out of the pipe.
    fn the_step_runs_the_way_the_ball_and_the_pipe_cross(at: [f64; 3], next: [f64; 3]) {
        let runs = Vector::new(at[0], at[1], at[2]).cross(Vector::new(at[0], at[1], 0.0));
        let step = Vector::new(next[0] - at[0], next[1] - at[1], next[2] - at[2]);
        assert!(
            runs.dot(step) > 0.0,
            "the crossing runs {runs:?} at {at:?} and the step to {next:?} is {step:?}"
        );
    }

    /// The two circles a ball centred on a pipe's axis crosses it in run
    /// **opposite ways** round the axis: the one ahead of the centre along the
    /// axis winds about the axis direction, and the one behind about its
    /// opposite. So the same three eighths are in order on one and out of
    /// order on the other, which is why this row reads the point it starts
    /// from.
    #[test]
    fn the_two_circles_a_ball_on_a_pipes_axis_crosses_it_in_run_opposite_ways() {
        let (ball, pipe) = a_ball_on_a_pipes_axis();
        for (z, ahead) in [(3.0, true), (-3.0, false)] {
            let at = |eighths| PredicatePoint::from(round_the_pipe(z, eighths));
            let forwards = if ahead { 1 } else { -1 };
            the_step_runs_the_way_the_ball_and_the_pipe_cross(
                round_the_pipe(z, 0),
                round_the_pipe(z, forwards),
            );
            let (in_order, out_of_order) = if ahead {
                (Decision::Pos, Decision::Neg)
            } else {
                (Decision::Neg, Decision::Pos)
            };
            assert_eq!(
                which_comes_first_round_a_crossing([&ball, &pipe], at(0), at(1), at(3)),
                Ok(in_order),
                "the circle at z = {z}"
            );
            assert_eq!(
                which_comes_first_round_a_crossing([&ball, &pipe], at(0), at(3), at(1)),
                Ok(out_of_order)
            );
            assert_eq!(
                which_comes_first_round_a_crossing([&pipe, &ball], at(0), at(1), at(3)),
                Ok(out_of_order),
                "and the pair the other way round runs the other way"
            );
        }
    }

    /// A ball one rounding off a pipe's axis crosses it in two quartic loops
    /// and not two circles, and still swallows the pipe's whole cross-section,
    /// so its loops are ordered round the axis the way the same ball's circles
    /// are on it. Until Q1554 the one off the axis was refused.
    ///
    /// The pipe's axis runs up `z` through `(1, 0, 0)` and the ball's centre is
    /// the next `f64` past it along `x`, `2⁻⁵²` out.
    #[test]
    fn a_ball_one_rounding_off_a_pipes_axis_is_ordered_as_one_on_it_is() {
        let pipe = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
                4.0,
            )
            .expect("a radius"),
        );
        let one_past = f64::from_bits(1.0_f64.to_bits() + 1);
        assert_eq!(one_past - 1.0, f64::EPSILON, "one rounding of 1 out");
        let at = |eighths| {
            let [x, y, z] = round_the_pipe(3.0, eighths);
            PredicatePoint::from([x + 1.0, y, z])
        };
        for centre in [1.0, one_past] {
            let ball =
                Surface::Sphere(Sphere::new(Point::new(centre, 0.0, 0.0), 5.0).expect("a radius"));
            assert_eq!(
                which_comes_first_round_a_crossing([&ball, &pipe], at(0), at(1), at(3)),
                Ok(Decision::Pos),
                "the ball centred at x = {centre}"
            );
            assert_eq!(
                which_comes_first_round_a_crossing([&pipe, &ball], at(0), at(1), at(3)),
                Ok(Decision::Neg),
                "the ball centred at x = {centre}, the pair the other way round"
            );
        }
    }

    /// A ball inscribed in a pipe touches it round one circle, through its
    /// centre, instead of crossing it, so `∇S × ∇T` is nothing all the way
    /// round and there is nothing to order along — a limit, the same one as a
    /// plane resting on a ring's tube.
    #[test]
    fn a_ball_inscribed_in_a_pipe_has_no_direction_to_be_ordered_along() {
        let ball = a_ball(4.0);
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 4.0).expect("a radius"));
        let on_it = PredicatePoint::from([4.0, 0.0, 0.0]);
        assert_eq!(
            which_comes_first_round_a_crossing([&ball, &pipe], on_it, on_it, on_it),
            Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
                surfaces: Box::new([ball, pipe]),
                at: WhereFollowingACrossingGaveUp::CrossingTheTwoNormalsForADirection,
            })
        );
    }

    /// A cone of `slope` up `z` with its apex `apex` up it.
    fn a_cone_up_z(apex: f64, slope: f64) -> crate::Cone {
        crate::Cone::new(
            Axis::new(Point::new(0.0, 0.0, apex), Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
            slope,
        )
        .expect("a slope")
    }

    /// The way out of a cone of `slope` up `z` with its apex `apex` up it, at
    /// `at`: `ρ − s²h ŵ`, worked out here and not by the code under test.
    fn out_of_the_cone(apex: f64, slope: f64, at: [f64; 3]) -> Vector {
        Vector::new(at[0], at[1], -slope * slope * (at[2] - apex))
    }

    /// The point `eighths` of a turn round the circle of `radius` at height
    /// `z`, anticlockwise about `+z`.
    fn round_the_axis(radius: f64, z: f64, eighths: i32) -> [f64; 3] {
        let angle = core::f64::consts::TAU * f64::from(eighths) / 8.0;
        [radius * angle.cos(), radius * angle.sin(), z]
    }

    /// That `∇S × ∇T`, handed over as worked out by each test, runs the way
    /// `anticlockwise` says about `+z` at the point a quarter turn round, and
    /// then that the pair orders three points of the circle that way, and the
    /// pair the other way round the other way.
    fn ordered_the_way_the_crossing_runs(
        pair: [&Surface; 2],
        radius: f64,
        z: f64,
        runs: Vector,
        anticlockwise: bool,
    ) {
        let step = Vector::new(-1.0, 0.0, 0.0);
        assert_eq!(
            runs.dot(step) > 0.0,
            anticlockwise,
            "∇S × ∇T is {runs:?} at a quarter turn round the circle at z = {z}"
        );
        let at = |eighths| PredicatePoint::from(round_the_axis(radius, z, eighths));
        let (in_order, out_of_order) = if anticlockwise {
            (Decision::Pos, Decision::Neg)
        } else {
            (Decision::Neg, Decision::Pos)
        };
        assert_eq!(
            which_comes_first_round_a_crossing(pair, at(0), at(1), at(3)),
            Ok(in_order),
            "the circle at z = {z}"
        );
        assert_eq!(
            which_comes_first_round_a_crossing(pair, at(0), at(3), at(1)),
            Ok(out_of_order)
        );
        assert_eq!(
            which_comes_first_round_a_crossing([pair[1], pair[0]], at(0), at(1), at(3)),
            Ok(out_of_order),
            "and the pair the other way round runs the other way"
        );
    }

    /// A cone of slope 1 up `z` from the origin and a pipe of radius 4 up `z`
    /// cross in the circles at `z = ±4`. At `(0, 4, ±4)` the cone's way out is
    /// `(0, 4, ∓4)` and the pipe's `(0, 4, 0)`, so `∇S × ∇T` is `(±16, 0, 0)`:
    /// clockwise about `+z` ahead of the apex and anticlockwise behind it. The
    /// pipe has no apex, and turned to point down the axis it changes nothing.
    ///
    /// A point on the plane through the apex is on neither circle, and which
    /// one it is on cannot be told.
    #[test]
    fn the_two_circles_a_cone_and_a_pipe_on_its_axis_cross_in_run_opposite_ways() {
        let cone = Surface::Cone(a_cone_up_z(0.0, 1.0));
        for pipe in [Vector::new(0.0, 0.0, 1.0), Vector::new(0.0, 0.0, -1.0)].map(|along| {
            Surface::Cylinder(
                Cylinder::new(Axis::new(Point::origin(), along).expect("an axis"), 4.0)
                    .expect("a radius"),
            )
        }) {
            for (z, anticlockwise) in [(4.0, false), (-4.0, true)] {
                let at = [0.0, 4.0, z];
                let runs = out_of_the_cone(0.0, 1.0, at).cross(Vector::new(0.0, 4.0, 0.0));
                ordered_the_way_the_crossing_runs([&cone, &pipe], 4.0, z, runs, anticlockwise);
            }
            let on_the_apex_plane = PredicatePoint::from([4.0, 0.0, 0.0]);
            assert!(matches!(
                which_comes_first_round_a_crossing(
                    [&cone, &pipe],
                    on_the_apex_plane,
                    on_the_apex_plane,
                    on_the_apex_plane
                ),
                Ok(Decision::Undecidable { .. })
            ));
        }
    }

    /// A cone of slope 2 with its apex at `z = 2` and one of slope 1 from the
    /// origin cross where `2|z − 2| = |z|`: at `z = 4`, radius 4, ahead of the
    /// steep apex, and at `z = 4/3`, radius `4/3`, behind it. At `(0, 4, 4)`
    /// the two ways out are `(0, 4, −8)` and `(0, 4, −4)`, and at
    /// `(0, 4/3, 4/3)` they are `(0, 4/3, 8/3)` and `(0, 4/3, −4/3)`, so
    /// `∇S × ∇T` is `(16, 0, 0)` and then `(−48/9, 0, 0)`: clockwise about `+z`
    /// ahead and anticlockwise behind, whichever cone is handed over first —
    /// the steeper one first, below, and then the shallower first by the swap.
    ///
    /// The steep cone stored pointing down its axis changes nothing, since the
    /// picker's normal turns with it.
    #[test]
    fn the_two_circles_two_cones_on_one_axis_cross_in_run_opposite_ways() {
        let shallow = Surface::Cone(a_cone_up_z(0.0, 1.0));
        let pointing_down = crate::Cone::new(
            Axis::new(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, -1.0)).expect("an axis"),
            2.0,
        )
        .expect("a slope");
        for steep in [a_cone_up_z(2.0, 2.0), pointing_down].map(Surface::Cone) {
            for (radius, anticlockwise) in [(4.0, false), (4.0 / 3.0, true)] {
                let at = [0.0, radius, radius];
                let runs = out_of_the_cone(2.0, 2.0, at).cross(out_of_the_cone(0.0, 1.0, at));
                ordered_the_way_the_crossing_runs(
                    [&steep, &shallow],
                    radius,
                    radius,
                    runs,
                    anticlockwise,
                );
            }
        }
    }

    /// Two cones of slope 1 with their apexes at the origin and four up cross
    /// in one circle halfway, `z = 2` with radius 2. At `(0, 2, 2)` the two
    /// ways out are `(0, 2, −2)` and `(0, 2, 2)`, so `∇S × ∇T` is `(8, 0, 0)`:
    /// clockwise about `+z`, which is about the first apex less the second.
    #[test]
    fn the_circle_two_cones_of_one_slope_cross_in_runs_about_the_first_apex_less_the_second() {
        let [lower, upper] = [0.0, 4.0].map(|apex| Surface::Cone(a_cone_up_z(apex, 1.0)));
        let at = [0.0, 2.0, 2.0];
        let runs = out_of_the_cone(0.0, 1.0, at).cross(out_of_the_cone(4.0, 1.0, at));
        ordered_the_way_the_crossing_runs([&lower, &upper], 2.0, 2.0, runs, false);
    }

    /// A ball of radius 5 centred seven up the axis of a cone of slope 1 from
    /// the origin crosses it where `2h² − 14h + 24 = 0`: at `z = 3`, radius 3,
    /// and `z = 4`, radius 4, either side of `h = 7/2`, which is where
    /// `intersect`'s picker is. At `(0, 3, 3)` the ways out are `(0, 3, −3)` and
    /// `(0, 3, −4)`, and at `(0, 4, 4)` they are `(0, 4, −4)` and `(0, 4, −3)`,
    /// so `∇S × ∇T` is `(−3, 0, 0)` and then `(4, 0, 0)`: anticlockwise about
    /// `+z` behind the picker and clockwise ahead of it.
    #[test]
    fn the_two_circles_a_ball_on_a_cones_axis_crosses_it_in_run_opposite_ways() {
        let cone = Surface::Cone(a_cone_up_z(0.0, 1.0));
        let ball = Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 7.0), 5.0).expect("a radius"));
        for (radius, anticlockwise) in [(3.0, true), (4.0, false)] {
            let at = [0.0, radius, radius];
            let runs = out_of_the_cone(0.0, 1.0, at).cross(Vector::new(at[0], at[1], at[2] - 7.0));
            ordered_the_way_the_crossing_runs([&cone, &ball], radius, radius, runs, anticlockwise);
        }
    }

    /// A ball of radius 3 centred five up the axis of a cone of slope `3/4`
    /// touches it round the circle at `z = 16/5`: `(1 + s²)R² − s²e²` is
    /// `(25/16)·9 − (9/16)·25`, nothing, exactly. The two ways out are along
    /// one line there, so there is nothing to order along — the limit a ball
    /// inscribed in a pipe is.
    #[test]
    fn a_ball_inscribed_in_a_cone_has_no_direction_to_be_ordered_along() {
        let cone = Surface::Cone(a_cone_up_z(0.0, 0.75));
        let ball = Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 5.0), 3.0).expect("a radius"));
        let on_it = PredicatePoint::from([2.4, 0.0, 3.2]);
        let refused = Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
            surfaces: Box::new([cone, ball]),
            at: WhereFollowingACrossingGaveUp::CrossingTheTwoNormalsForADirection,
        });
        assert_eq!(
            which_comes_first_round_a_crossing([&cone, &ball], on_it, on_it, on_it),
            refused
        );
        assert_eq!(
            which_comes_first_round_a_crossing([&ball, &cone], on_it, on_it, on_it),
            refused
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
        // One off the pipe's axis by exactly how much wider than the pipe it
        // is, so it touches the pipe from inside at `(−4, 0, 0)` and crosses it
        // in a figure of eight. A ball on the axis is ordered, and so is one off
        // it that swallows the pipe whole, below.
        let ball_beside =
            Surface::Sphere(Sphere::new(Point::new(1.0, 0.0, 0.0), 5.0).expect("a radius"));
        let cone = Surface::Cone(crate::Cone::new(Axis::z(), 1.0).expect("a slope"));
        // Its apex one unit off the others' axis, which crosses each of them in
        // a quartic. On the axis they are ordered, below.
        let cone_beside = Surface::Cone(
            crate::Cone::new(
                Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
                1.0,
            )
            .expect("a slope"),
        );
        let steeper_of_one_apex = Surface::Cone(crate::Cone::new(Axis::z(), 2.0).expect("a slope"));
        // A ball on the cone's axis whose skin runs through the apex: one circle
        // and the apex apart from it.
        let ball_through_the_apex =
            Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 5.0), 5.0).expect("a radius"));
        // A ball on the cone's axis inside one nappe, missing it altogether.
        let ball_inside_the_cone =
            Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 10.0), 1.0).expect("a radius"));
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
            [&ball_beside, &pipe],
            [&pipe, &ball_beside],
            [&pipe, &pipe],
            // Two balls with the same centre have no line between them.
            [&ball, &ball],
            // A cone against a pipe, a ball or a second cone off its axis,
            // which `intersect` refuses to name a crossing for at all.
            [&cone_beside, &ball],
            [&ball, &cone_beside],
            [&cone_beside, &pipe],
            [&pipe, &cone_beside],
            [&cone, &cone_beside],
            // Two cones of one apex meet there and nowhere else, and one cone
            // twice is one surface.
            [&cone, &steeper_of_one_apex],
            [&cone, &cone],
            [&cone, &ball_through_the_apex],
            [&ball_inside_the_cone, &cone],
            // A torus against another curved surface, which `intersect`
            // refuses to name a crossing for at all.
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

    /// A pipe of radius 1 along `x` at `z = 5`, `off` along `y` from the axis
    /// of a pipe of radius 4 about `z` that swallows it, and that fat pipe: a
    /// cross-drilled shaft. They cross in two quartic loops, one each side of
    /// `x = 0`.
    fn a_thin_pipe_and_a_fat_one(off: f64) -> [Surface; 2] {
        let thin = Cylinder::new(
            Axis::new(Point::new(0.0, off, 5.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis"),
            1.0,
        )
        .expect("a radius");
        let fat = Cylinder::new(Axis::z(), 4.0).expect("a radius");
        [Surface::Cylinder(thin), Surface::Cylinder(fat)]
    }

    /// The point of [`a_thin_pipe_and_a_fat_one`]'s loop on the `side` of
    /// `x = 0` that is `eighths` eighths of a turn round the thin pipe's axis
    /// from `+y` towards `+z`.
    fn round_the_thin_pipe(off: f64, side: f64, eighths: i32) -> [f64; 3] {
        let angle = core::f64::consts::TAU * f64::from(eighths) / 8.0;
        let [y, z] = [off + angle.cos(), 5.0 + angle.sin()];
        [side * (16.0 - y * y).sqrt(), y, z]
    }

    /// Each loop a fat pipe crosses a thin one in runs once round the thin
    /// pipe's axis, so three points of one come back in the order they run
    /// round it, from wherever the ordering starts. The loop where the thin
    /// pipe, running along `+x`, comes out of the fat one runs about `−x`, and
    /// the one where it goes in about `+x`; each step is checked against
    /// `∇S × ∇T` before the order is asked, so what this pins is the geometry.
    #[test]
    fn three_points_on_a_loop_where_two_pipes_cross_come_back_in_the_order_they_run_round_it() {
        for off in [0.0, 0.5] {
            let [thin, fat] = a_thin_pipe_and_a_fat_one(off);
            for (side, forward) in [(1.0, -1), (-1.0, 1)] {
                let round = |eighths: i32| round_the_thin_pipe(off, side, forward * eighths);
                let at = |eighths| PredicatePoint::from(round(eighths));
                for start in 0..8 {
                    let [here, next] = [round(start), round(start + 1)];
                    let runs = Vector::new(0.0, here[1] - off, here[2] - 5.0)
                        .cross(Vector::new(here[0], here[1], 0.0));
                    let step = Vector::new(next[0] - here[0], next[1] - here[1], next[2] - here[2]);
                    assert!(
                        runs.dot(step) > 0.0,
                        "off {off}, side {side}: the crossing runs {runs:?} at {here:?} and the \
                         step to {next:?} is {step:?}"
                    );
                    let [first, second, third] = [start, start + 1, start + 3].map(at);
                    assert_eq!(
                        which_comes_first_round_a_crossing([&thin, &fat], first, second, third),
                        Ok(Decision::Pos),
                        "off {off}, side {side}, from {start} eighths"
                    );
                    assert_eq!(
                        which_comes_first_round_a_crossing([&thin, &fat], first, third, second),
                        Ok(Decision::Neg),
                        "off {off}, side {side}, from {start} eighths, asked the other way"
                    );
                    assert_eq!(
                        which_comes_first_round_a_crossing([&fat, &thin], first, second, third),
                        Ok(Decision::Neg),
                        "off {off}, side {side}, from {start} eighths, the pipes swapped"
                    );
                }
            }
        }
    }

    /// Ten points of the one loop a pipe reaching out of the side of a pipe of
    /// radius 4 up `z` crosses it in come back in the order they run round it,
    /// from each of the ten, every pair of the other nine both ways and with
    /// the two pipes swapped. The loop runs back and forth round both axes, so
    /// no normal orders it the way a conic is ordered. The reaching pipe runs
    /// along `x` with radius 1, which is the bar Q1857 cut in half; along
    /// `x + z`, so the two axes are not square to each other; and along `x`
    /// with radius 4, so neither pipe is the thinner. Each step is checked
    /// against `∇S × ∇T` before the order is asked, so what this pins is the
    /// geometry.
    #[test]
    fn ten_points_on_the_one_loop_where_two_pipes_cross_come_back_in_the_order_they_run_round_it() {
        let off = 3.5;
        let fat = Surface::Cylinder(Cylinder::new(Axis::z(), 4.0).expect("a radius"));
        for (radius, lean) in [(1.0, 0.0), (1.0, 1.0), (4.0, 0.0)] {
            let way = Vector::new(1.0, 0.0, lean);
            let reaching = Surface::Cylinder(
                Cylinder::new(
                    Axis::new(Point::new(0.0, off, 5.0), way).expect("an axis"),
                    radius,
                )
                .expect("a radius"),
            );
            let round = round_the_one_loop_two_pipes_cross_in(radius, off, lean);
            let count = round.len();
            for start in 0..count {
                let [here, next] = [round[start], round[(start + 1) % count]];
                let from_the_axis = Vector::new(here[0], here[1] - off, here[2] - 5.0);
                let along = way * (from_the_axis.dot(way) / way.dot(way));
                let runs = (from_the_axis - along).cross(Vector::new(here[0], here[1], 0.0));
                let step = Vector::new(next[0] - here[0], next[1] - here[1], next[2] - here[2]);
                assert!(
                    runs.dot(step) > 0.0,
                    "radius {radius}, lean {lean}: the crossing runs {runs:?} at {here:?} and \
                     the step to {next:?} is {step:?}"
                );
            }
            for start in 0..count {
                let at = |ahead: usize| PredicatePoint::from(round[(start + ahead) % count]);
                for one in 1..count {
                    for other in one + 1..count {
                        let what = format!(
                            "radius {radius}, lean {lean}, from point {start}, {one} then {other} \
                             ahead"
                        );
                        assert_eq!(
                            which_comes_first_round_a_crossing(
                                [&reaching, &fat],
                                at(0),
                                at(one),
                                at(other)
                            ),
                            Ok(Decision::Pos),
                            "{what}"
                        );
                        assert_eq!(
                            which_comes_first_round_a_crossing(
                                [&reaching, &fat],
                                at(0),
                                at(other),
                                at(one)
                            ),
                            Ok(Decision::Neg),
                            "{what}, asked the other way"
                        );
                        assert_eq!(
                            which_comes_first_round_a_crossing(
                                [&fat, &reaching],
                                at(0),
                                at(one),
                                at(other)
                            ),
                            Ok(Decision::Neg),
                            "{what}, the pair swapped"
                        );
                    }
                }
            }
        }
    }

    /// Ten points of the one loop a pipe of `radius` along `x + lean z`
    /// through `(0, off, 5)` crosses a pipe of radius 4 up `z` in, in the order
    /// the crossing runs with the leaning pipe taken first: on the side of
    /// `x = 0` that is `x > 0`, going round the leaning pipe's axis from
    /// below `z = 5`'s side to above it, then back on the side `x < 0`.
    fn round_the_one_loop_two_pipes_cross_in(radius: f64, off: f64, lean: f64) -> Vec<[f64; 3]> {
        use core::f64::consts::PI;
        let widest = ((4.0 - off) / radius).acos();
        let on_the_side = |side: f64, step: i32| {
            let angle = PI + (PI - widest) * f64::from(step) / 3.0;
            let y = off + radius * angle.cos();
            let x = side * (16.0 - y * y).sqrt();
            [
                x,
                y,
                5.0 + lean * x + (1.0 + lean * lean).sqrt() * radius * angle.sin(),
            ]
        };
        let ahead = (-2..=2).rev().map(|step| on_the_side(1.0, step));
        let back = (-2..=2).map(|step| on_the_side(-1.0, step));
        ahead.chain(back).collect()
    }

    /// Two pipes of one radius, two running the same way, and a thin pipe the
    /// fat one does not swallow whole, which crosses it in one loop that runs
    /// back and forth round both axes, are refused rather than ordered.
    #[test]
    fn two_pipes_that_do_not_cross_in_two_loops_round_the_thinner_are_refused() {
        let pipe = |at: Point, along: Vector, radius: f64| {
            Surface::Cylinder(
                Cylinder::new(Axis::new(at, along).expect("an axis"), radius).expect("a radius"),
            )
        };
        let [along_x, along_z] = [Vector::new(1.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0)];
        let fat = pipe(Point::origin(), along_z, 4.0);
        let pairs = [
            (
                "of one radius",
                pipe(Point::new(0.0, 0.0, 5.0), along_x, 4.0),
                [0.0, 4.0, 5.0],
            ),
            (
                "running the same way",
                pipe(Point::new(4.0, 0.0, 0.0), along_z, 1.0),
                [3.875, 0.984_375_f64.sqrt(), 5.0],
            ),
            (
                "not swallowed whole",
                pipe(Point::new(0.0, 3.5, 5.0), along_x, 1.0),
                [3.75_f64.sqrt(), 3.5, 6.0],
            ),
        ];
        for (what, other, on_both) in pairs {
            let [from, one, other_point] =
                [on_both, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]].map(PredicatePoint::from);
            for pair in [[&fat, &other], [&other, &fat]] {
                assert_eq!(
                    which_comes_first_round_a_crossing(pair, from, one, other_point),
                    Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
                    "two pipes {what}"
                );
            }
        }
    }

    /// A ball of `radius` centred `off` along `x`, and a pipe of radius 1 up
    /// `z`: a ball drilled through off its centre.
    fn a_ball_off_a_pipes_axis(radius: f64, off: f64) -> [Surface; 2] {
        [
            Surface::Sphere(Sphere::new(Point::new(off, 0.0, 0.0), radius).expect("a radius")),
            Surface::Cylinder(Cylinder::new(Axis::z(), 1.0).expect("a radius")),
        ]
    }

    /// The point of [`a_ball_off_a_pipes_axis`]'s loop on the `side` of
    /// `z = 0` that is `eighths` eighths of a turn round the pipe's axis from
    /// `+x`, anticlockwise about `+z`.
    fn round_the_drilled_pipe(radius: f64, off: f64, side: f64, eighths: i32) -> [f64; 3] {
        let angle = core::f64::consts::TAU * f64::from(eighths) / 8.0;
        let [x, y] = [angle.cos(), angle.sin()];
        let across = (x - off) * (x - off) + y * y;
        [x, y, side * (radius * radius - across).sqrt()]
    }

    /// Each loop a ball crosses a pipe in, where the ball swallows the pipe's
    /// whole cross-section off its axis, runs once round the pipe's axis, so
    /// three points of one come back in the order they run round it, from
    /// wherever the ordering starts. The loop above the centre runs about `+z`
    /// and the one below about `−z`; each step is checked against `∇S × ∇T`
    /// before the order is asked, so what this pins is the geometry. The
    /// centre is off the axis by less than the pipe's radius, so the axis runs
    /// through the ball's equator inside the pipe, and by more, so it does not.
    #[test]
    fn three_points_on_a_loop_where_a_ball_crosses_a_pipe_off_its_axis_come_back_in_the_order_they_run_round_it()
     {
        for (radius, off) in [(3.0, 0.5), (4.0, 2.0)] {
            let [ball, pipe] = a_ball_off_a_pipes_axis(radius, off);
            for (side, forward) in [(1.0, 1), (-1.0, -1)] {
                let round =
                    |eighths: i32| round_the_drilled_pipe(radius, off, side, forward * eighths);
                let at = |eighths| PredicatePoint::from(round(eighths));
                for start in 0..8 {
                    let [here, next] = [round(start), round(start + 1)];
                    let runs = Vector::new(here[0] - off, here[1], here[2])
                        .cross(Vector::new(here[0], here[1], 0.0));
                    let step = Vector::new(next[0] - here[0], next[1] - here[1], next[2] - here[2]);
                    assert!(
                        runs.dot(step) > 0.0,
                        "ball {radius} off {off}, side {side}: the crossing runs {runs:?} at \
                         {here:?} and the step to {next:?} is {step:?}"
                    );
                    let [first, second, third] = [start, start + 1, start + 3].map(at);
                    let what =
                        format!("ball {radius} off {off}, side {side}, from {start} eighths");
                    assert_eq!(
                        which_comes_first_round_a_crossing([&ball, &pipe], first, second, third),
                        Ok(Decision::Pos),
                        "{what}"
                    );
                    assert_eq!(
                        which_comes_first_round_a_crossing([&ball, &pipe], first, third, second),
                        Ok(Decision::Neg),
                        "{what}, asked the other way"
                    );
                    assert_eq!(
                        which_comes_first_round_a_crossing([&pipe, &ball], first, second, third),
                        Ok(Decision::Neg),
                        "{what}, the pair swapped"
                    );
                }
            }
        }
    }

    /// Ten points of the one loop a ball off a pipe's axis that does not
    /// swallow its whole cross-section crosses it in come back in the order
    /// they run round it, from each of the ten, every pair of the other nine
    /// both ways and with the pair of surfaces swapped. The loop runs back and
    /// forth round the pipe's axis, so no normal orders it the way a conic is
    /// ordered. The ball that reaches into the pipe's side is wide enough that
    /// the loop's shadow on a plane square to the line from the axis to the
    /// centre is not convex: `1² + 1.5²` is less than `2²`. The other is no
    /// wider than the pipe, and its loop runs through the pipe's axis. Each
    /// step is checked against `∇S × ∇T` before the order is asked, so what
    /// this pins is the geometry.
    #[test]
    fn ten_points_on_the_one_loop_where_a_ball_crosses_a_pipe_off_its_axis_come_back_in_the_order_they_run_round_it()
     {
        for (radius, off) in [(2.0, 1.5), (1.0, 1.0)] {
            let [ball, pipe] = a_ball_off_a_pipes_axis(radius, off);
            let round = round_the_one_loop(radius, off);
            let count = round.len();
            for start in 0..count {
                let [here, next] = [round[start], round[(start + 1) % count]];
                let runs = Vector::new(here[0] - off, here[1], here[2])
                    .cross(Vector::new(here[0], here[1], 0.0));
                let step = Vector::new(next[0] - here[0], next[1] - here[1], next[2] - here[2]);
                assert!(
                    runs.dot(step) > 0.0,
                    "ball {radius} off {off}: the crossing runs {runs:?} at {here:?} and the \
                     step to {next:?} is {step:?}"
                );
            }
            for start in 0..count {
                let at = |ahead: usize| PredicatePoint::from(round[(start + ahead) % count]);
                for one in 1..count {
                    for other in one + 1..count {
                        let what = format!(
                            "ball {radius} off {off}, from point {start}, {one} then {other} ahead"
                        );
                        assert_eq!(
                            which_comes_first_round_a_crossing(
                                [&ball, &pipe],
                                at(0),
                                at(one),
                                at(other)
                            ),
                            Ok(Decision::Pos),
                            "{what}"
                        );
                        assert_eq!(
                            which_comes_first_round_a_crossing(
                                [&ball, &pipe],
                                at(0),
                                at(other),
                                at(one)
                            ),
                            Ok(Decision::Neg),
                            "{what}, asked the other way"
                        );
                        assert_eq!(
                            which_comes_first_round_a_crossing(
                                [&pipe, &ball],
                                at(0),
                                at(one),
                                at(other)
                            ),
                            Ok(Decision::Neg),
                            "{what}, the pair swapped"
                        );
                    }
                }
            }
        }
    }

    /// Ten points of the one loop [`a_ball_off_a_pipes_axis`] crosses its pipe
    /// in, in the order the crossing runs with the ball taken first: above
    /// `z = 0` with the angle round the pipe's axis rising, then below it with
    /// the angle falling.
    fn round_the_one_loop(radius: f64, off: f64) -> Vec<[f64; 3]> {
        let widest = ((1.0 + off * off - radius * radius) / (2.0 * off)).acos();
        let on_the_side = |side: f64, step: i32| {
            let angle = widest * f64::from(step) / 3.0;
            let [x, y] = [angle.cos(), angle.sin()];
            let across = (x - off) * (x - off) + y * y;
            [x, y, side * (radius * radius - across).sqrt()]
        };
        let above = (-2..=2).map(|step| on_the_side(1.0, step));
        let below = (-2..=2).rev().map(|step| on_the_side(-1.0, step));
        above.chain(below).collect()
    }

    /// A ball off a pipe's axis that misses the pipe, or touches its side from
    /// outside at one point, crosses it in no loop, and is refused rather than
    /// ordered. A ball that touches the pipe from inside is in
    /// [`a_crossing_that_is_not_a_conic_is_refused_rather_than_ordered`].
    #[test]
    fn a_ball_off_a_pipes_axis_that_misses_it_or_touches_its_side_is_refused() {
        for (what, radius, off) in [("missing it", 0.5, 2.0), ("touching its side", 1.0, 2.0)] {
            let [ball, pipe] = a_ball_off_a_pipes_axis(radius, off);
            let [from, one, other] =
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]].map(PredicatePoint::from);
            for pair in [[&ball, &pipe], [&pipe, &ball]] {
                assert_eq!(
                    which_comes_first_round_a_crossing(pair, from, one, other),
                    Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
                    "a ball {what}"
                );
            }
        }
    }
}
