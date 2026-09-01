//! Points along the curve where two surfaces cross.
//!
//! [`intersect`](crate::intersect) *names* a crossing and never draws one
//! (`docs/decisions.md` §24.3). A mesher still needs points along an ellipse,
//! and this is where that lands: the fidelity does not belong to the crossing,
//! it belongs to evaluating points along it, and it is measured where it is
//! made.
//!
//! # One authority on where an edge is
//!
//! **The evaluator here walks every edge, and `rcad-topo` walks none** (§27.5).
//! The mesher used to step in a straight line through the surface's own two
//! coordinates, which is right for a pipe's ruling, a cap circle and a ball's
//! meridian and wrong for the ellipse a tilted plane cuts out of a pipe. Two
//! authorities on where the points along an edge are is the shape §23.8 kept
//! finding, and that straight line was one of them.
//!
//! What crosses the seam is a [`NamedRun`], the surfaces of the two faces either
//! side, and a [`Deflection`]. What comes back is 3D points and one
//! [`Approximation`]. **No `(u, v)` crosses it**: a shared edge lands in two
//! different coordinate systems, and everything downstream of here already works
//! from 3D points.
//!
//! # The crossings that lie in a plane
//!
//! All four of them are a conic in a plane, so one parametrization covers the
//! lot: a centre and two semi-axis vectors square to each other, walked round in
//! an angle. That is what makes the closed form an oracle the marcher can be
//! judged against (§24.1).

use rcad_predicates::{
    Approximation, Cause, Decision, Interval, Level, Measures, Point as PredicatePoint, Policy,
    ProvenanceLog, Record, SideOfSurface,
};

use crate::exact::Exact;
use crate::meet::nowhere_in_particular;
use crate::{
    Cylinder, Deflection, EdgeRun, Error, NamedRun, Plane, Point, Result, SIDE_OF_CYLINDER,
    SIDE_OF_PLANE, SIDE_OF_SPHERE, Sphere, Surface, Vector,
};

/// How far a point [`walk`] hands back can sit off the curve it was worked out
/// along, as a multiple of the largest number involved.
///
/// The size to multiply by is the largest magnitude among the numbers the two
/// surfaces place themselves with — a plane's offset, a ball's centre, a pipe's
/// axis origin — their radii, and the conic's own centre and reach. It is
/// absolute in that size and never relative to the answer, for the reason
/// [`Plane::SIGNED_DISTANCE_ROUNDOFF`] opens with.
///
/// Where the number comes from, with `u = f64::EPSILON / 2` and `L` the size
/// above. Every step is a *relative* one, so the chain is a count of roundings
/// on terms no bigger than a few `L`:
///
/// - The conic's **centre** is a dot product (three products and two sums), a
///   subtraction, a division and three scaled additions: about twelve roundings
///   on terms under `2L`, so under `24u·L`.
/// - Its **reach** is a square root of a ratio whose numerator is summed through
///   exact expansion arithmetic rather than in `f64`. That is what keeps this
///   term relative: written out in `f64` the numerator of a nearly-tangent
///   circle cancels to nothing and the radius that comes out of it is wrong by
///   `u·L²/reach`, which grows without bound as the circle shrinks. Summed
///   exactly it is right to within a rounding or two, so under `4u·L`.
/// - The two **semi-axis directions** are normalizations: a dot, a square root
///   and three divisions each, under `8u·L` apiece.
/// - `cos` and `sin` round, and Rust does not say how accurately. Allow a few
///   ulps each: under `8u·L` for the pair.
/// - Putting them together is three products and three sums per coordinate on
///   terms under `3L`, so under `12u·L`.
///
/// That comes to about `68u·L`, which is `34 · f64::EPSILON · L`. The constant
/// is seven times it, and the room is for the trigonometry, on the same argument
/// as [`Surface::ROUND_TRIP_ROUNDOFF`](crate::Surface::ROUND_TRIP_ROUNDOFF):
/// this has to hold for a libm that is a few ulps out rather than one that is
/// perfect.
///
/// **One step of that chain is not a rounding but a subtraction that had to be
/// put there**, and leaving it out is the one way this bound stops being one.
/// See `two_directions_in`.
///
/// **An error in the angle does not come into it.** Getting `angle` wrong slides
/// the point *along* the curve, and this is how far it can sit *off* it.
///
/// `every_point_along_a_crossing_is_on_both_surfaces.rs` measures the worst a
/// search can find and prints the figure.
pub const POINT_ALONG_A_CROSSING_ROUNDOFF: f64 = 256.0 * f64::EPSILON;

/// The most points one crossing is walked into.
///
/// A million. What sets it is not memory but honesty: a deflection close to
/// [`POINT_ALONG_A_CROSSING_ROUNDOFF`] asks for tens of millions of points
/// standing in for a curve they are barely closer to than a thousand would be,
/// and building them quietly is worse than saying so.
pub const MOST_POINTS_ALONG_ONE_CROSSING: usize = 1 << 20;

/// Points along a crossing, and how far off it they may sit.
#[derive(Clone, Debug, PartialEq)]
pub struct PointsAlongACrossing {
    points: Vec<Point>,
    deviation: Approximation,
}

impl PointsAlongACrossing {
    /// The points, in the direction the run is stored running, **with neither
    /// end included**.
    ///
    /// The ends are corners the model already holds, so whoever reads this puts
    /// them back at either side. An edge that needs no help at all — a straight
    /// one — gives an empty list.
    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// How far any of them may sit off the curve, blaming
    /// [`Cause::PointAlongACrossing`].
    ///
    /// **One number for the whole run and never one per point** (§24.13). A mesh
    /// of a curved face is thousands of points and §5 records provenance per
    /// decision rather than per coordinate; a record each would be that rule
    /// exactly inverted, arriving dressed as thoroughness.
    #[must_use]
    pub fn deviation(&self) -> Approximation {
        self.deviation
    }
}

/// Walk one run of a crossing into the points that stand in for it.
///
/// `along` is the surfaces of the two faces either side. Where those are the
/// same surface — a ball's seam is — the run's own
/// [`EdgeRun::PartOfWhereItCrosses`] names the second one, and that is the whole
/// reason that variant carries it.
///
/// # How fine, and what happens when it cannot be proven
///
/// Two knobs and they do different jobs (§23.5). `deflection` says how far the
/// straight pieces between these points may stray from the curve; `policy` says
/// what happens when a *point* cannot be placed that close to the curve at all.
/// A caller who reached for the policy to ask for a rougher walk would have
/// silently asked the kernel to certify less.
///
/// # Errors
///
/// [`Error::CannotPlaceAPointThatCloseToACrossing`] when a point cannot be
/// worked out as close to the curve as `deflection` asks and the policy will not
/// accept being that far out.
///
/// [`Error::TooManyPointsAlongOneCrossing`] when the deflection asks for more
/// than [`MOST_POINTS_ALONG_ONE_CROSSING`].
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the half-spaces on the
/// run pick both of the two arcs between its ends, or neither.
///
/// [`Error::CrossingRunsToAQuartic`] for the two pairs the closed form does not
/// name, and [`Error::Undecidable`] when the conic's own size could not be
/// worked out because a product fell out of range.
pub fn walk(
    run: &NamedRun,
    along: [&Surface; 2],
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<PointsAlongACrossing> {
    // A straight run is the whole of what its two ends say, so there is nothing
    // to put between them.
    if run.run() == &EdgeRun::Straight {
        return Ok(nothing_between_the_ends());
    }

    let Some(conic) = the_conic_between(the_two_surfaces_it_runs_between(run, along))? else {
        return Ok(nothing_between_the_ends());
    };
    let deviation = POINT_ALONG_A_CROSSING_ROUNDOFF * conic.how_far_it_reaches();
    let tag = Approximation::new(deviation, Cause::PointAlongACrossing);
    let Some(room) = room_left_over(deflection, deviation, policy)? else {
        return Ok(PointsAlongACrossing {
            points: Vec::new(),
            deviation: tag,
        });
    };

    let (from, to) = conic.the_stretch_this_run_covers(run, deviation, log)?;
    let steps = how_many_steps(&conic, to - from, room)?;
    let points = (1..steps)
        .map(|step| conic.at(from + (to - from) * (step as f64) / (steps as f64)))
        .collect();

    Ok(PointsAlongACrossing {
        points,
        deviation: tag,
    })
}

/// What comes back for a run that needs no help at all: its two ends are the
/// whole of it.
fn nothing_between_the_ends() -> PointsAlongACrossing {
    PointsAlongACrossing {
        points: Vec::new(),
        deviation: Approximation::new(0.0, Cause::PointAlongACrossing),
    }
}

/// The two surfaces the crossing is between.
///
/// The faces either side name it, except along a seam, where both faces are the
/// same surface and the run carries the second one itself.
fn the_two_surfaces_it_runs_between(run: &NamedRun, along: [&Surface; 2]) -> [Surface; 2] {
    match run.run() {
        EdgeRun::PartOfWhereItCrosses { and, .. } if along[0] == along[1] => [*along[0], *and],
        _ => [*along[0], *along[1]],
    }
}

/// The conic the crossing of these two surfaces lies on, or `None` when the
/// crossing is straight and its two ends are the whole of it.
///
/// All four planar pairs come down to a plane and a quadric, and two balls come
/// down to that too — through the radical plane, which is built **here** and not
/// in the naming half, because it is only ever needed to work points out
/// (§27.1).
fn the_conic_between(pair: [Surface; 2]) -> Result<Option<Conic>> {
    match pair {
        [Surface::Plane(_), Surface::Plane(_)] => Ok(None),
        [Surface::Plane(flat), Surface::Sphere(ball)]
        | [Surface::Sphere(ball), Surface::Plane(flat)] => {
            where_a_plane_cuts_a_ball(&flat, &ball).map(Some)
        }
        // A plane along a pipe's axis cuts it in straight lines, and a run that
        // is one of those is picked out by a half-space rather than being
        // `Straight`, so it reaches here and there is no ellipse to build.
        [Surface::Plane(flat), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Plane(flat)] => {
            if flat.normal().dot(pipe.axis().direction()) == 0.0 {
                return Ok(None);
            }
            where_a_plane_cuts_a_pipe(&flat, &pipe).map(Some)
        }
        [Surface::Sphere(one), Surface::Sphere(other)] => {
            where_two_balls_cut(&one, &other).map(Some)
        }
        [first, second] => Err(Error::CrossingRunsToAQuartic {
            first: name_of(&first),
            second: name_of(&second),
        }),
    }
}

/// How much of the deflection is left for the straight pieces to spend, or
/// `None` when a tolerant caller accepted a walk that cannot be refined at all.
fn room_left_over(deflection: Deflection, deviation: f64, policy: &Policy) -> Result<Option<f64>> {
    let room = deflection.length() - deviation;
    if room > 0.0 {
        return Ok(Some(room));
    }
    if policy.wobble_is_acceptable(deviation) {
        return Ok(None);
    }
    Err(Error::CannotPlaceAPointThatCloseToACrossing {
        asked: deflection.length(),
        closest: deviation,
    })
}

/// How many straight pieces this stretch of the conic has to be walked into.
///
/// Two questions, and the answer is the larger. **How far it bends**: the arc
/// from one corner to the next stands off its own chord by at most
/// `reach · (1 − cos(step / 2))`, so that is what has to stay inside `room`.
/// **How far it turns**: a step of more than half a turn cannot be told from one
/// the other way, so none is allowed past a quarter — which is the same bound
/// `rcad-topo`'s mesher already refuses to step past, for the same reason.
fn how_many_steps(conic: &Conic, turns_over: f64, room: f64) -> Result<usize> {
    let widest = (1.0 - room / conic.how_far_it_reaches())
        .clamp(-1.0, 1.0)
        .acos()
        * 2.0;
    let by_bending = (turns_over.abs() / widest.max(f64::MIN_POSITIVE)).ceil();
    let by_turning = (turns_over.abs() / (core::f64::consts::TAU / 4.0)).ceil();
    let steps = by_bending.max(by_turning).max(1.0);
    if steps.is_nan() || steps > MOST_POINTS_ALONG_ONE_CROSSING as f64 {
        return Err(Error::TooManyPointsAlongOneCrossing {
            most: MOST_POINTS_ALONG_ONE_CROSSING,
        });
    }
    Ok(steps as usize)
}

/// A conic in a plane: a centre and two semi-axis vectors square to each other.
///
/// One shape for all four planar pairs. A circle has two semi-axes the same
/// length, an ellipse two different ones, and nothing else about walking them
/// differs.
#[derive(Clone, Copy, Debug)]
struct Conic {
    centre: Point,
    across: Vector,
    and: Vector,
    /// The largest magnitude any of this had to be worked out from, which is
    /// what [`POINT_ALONG_A_CROSSING_ROUNDOFF`] is a multiple of.
    size: f64,
}

impl Conic {
    /// The point at `angle` radians round, measured from
    /// [`Conic::across`](Conic) towards [`Conic::and`](Conic).
    fn at(&self, angle: f64) -> Point {
        self.centre + self.across * angle.cos() + self.and * angle.sin()
    }

    /// Which angle `point` sits at, on `atan2`'s branch of `(-π, π]`.
    fn angle_of(&self, point: Point) -> f64 {
        let from_centre = point - self.centre;
        (from_centre.dot(self.and) / self.and.length_squared())
            .atan2(from_centre.dot(self.across) / self.across.length_squared())
    }

    /// The furthest from the centre this conic goes, and the size everything
    /// about it was worked out from.
    fn how_far_it_reaches(&self) -> f64 {
        self.size
    }

    /// The stretch of angle `run` covers, as a pair the second of which may be
    /// more or less than the first.
    ///
    /// A run that goes right round covers a whole turn from wherever it starts.
    /// A run that is one piece of a crossing has two ends and **two** arcs
    /// between them, and the half-spaces on it are what say which — asked at the
    /// middle of each arc, of the box that point is really known to be inside,
    /// so a witness that came out of trigonometry cannot certify a confident
    /// sign about noise (§27.7).
    fn the_stretch_this_run_covers(
        &self,
        run: &NamedRun,
        deviation: f64,
        log: &mut ProvenanceLog,
    ) -> Result<(f64, f64)> {
        let ends = run.ends().map(|end| {
            (end != nowhere_in_particular())
                .then(|| Point::from(end.enclosure().map(|side| (side.lo() + side.hi()) / 2.0)))
        });

        let from = ends[0].map_or(0.0, |end| self.angle_of(end));
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
            return Ok((from, from + core::f64::consts::TAU));
        };
        let to = ends[1].map_or(from + core::f64::consts::TAU, |end| self.angle_of(end));

        // The two arcs between the same two ends, going each way round.
        let onwards = from + wrapped(to - from);
        let backwards = from + wrapped(to - from) - core::f64::consts::TAU;
        let mut picked: Vec<(f64, f64)> = Vec::new();
        for end in [onwards, backwards] {
            if self.picked_by_all(picked_by, (from + end) / 2.0, deviation, log) {
                picked.push((from, end));
            }
        }
        match picked.as_slice() {
            [only] => Ok(*only),
            _ => Err(Error::CannotNameWhichPieceOfACrossingThisIs),
        }
    }

    /// Whether the point half way along an arc is on or outside every one of
    /// `picked_by`.
    fn picked_by_all(
        &self,
        picked_by: &[Surface],
        middle: f64,
        deviation: f64,
        log: &mut ProvenanceLog,
    ) -> bool {
        let at = somewhere_within(self.at(middle), deviation);
        let mut on_or_outside = true;
        for half_space in picked_by {
            let (decision, named, measures) = match half_space {
                Surface::Plane(flat) => (flat.side_of_point(at), SIDE_OF_PLANE, Measures::Length),
                Surface::Sphere(ball) => (ball.side_of(at), SIDE_OF_SPHERE, Measures::Area),
                Surface::Cylinder(pipe) => (pipe.side_of(at), SIDE_OF_CYLINDER, Measures::Area),
            };
            match decision {
                Decision::Pos | Decision::Zero => (),
                Decision::Neg => on_or_outside = false,
                // A half-space that could not tell rules out neither arc, so
                // both arcs come back picked and the caller is refused rather
                // than handed a coin toss. It is recorded because it happened,
                // and because the refusal that follows says nothing about which
                // half-space gave up.
                Decision::Undecidable { .. } => log.push(Record::new(
                    named,
                    &[at],
                    Level::Interval,
                    decision,
                    measures,
                )),
                other => panic!("a half-space came back with a decision nothing knows: {other:?}"),
            }
        }
        on_or_outside
    }
}

/// The circle where a plane cuts a ball.
///
/// The centre is the foot of the perpendicular from the ball's centre, and the
/// radius comes from `n·n · R² − (n·c − d)²` — the very discriminant that named
/// the crossing, divided by `n·n`. **Summed through [`Exact`] and not in `f64`**,
/// which is what keeps [`POINT_ALONG_A_CROSSING_ROUNDOFF`] a relative bound: a
/// nearly-tangent circle is a small difference of two large squares, and in
/// `f64` that cancels to noise.
fn where_a_plane_cuts_a_ball(flat: &Plane, ball: &Sphere) -> Result<Conic> {
    let normal = flat.normal().components();
    let centre = ball.centre().coordinates();
    let radius = ball.radius();

    let worked_out = || {
        let squared_normal = Exact::dot_of(normal, normal)?;
        let reach = Exact::dot_of(normal, centre)?.minus(&Exact::of(flat.offset()));
        let room = squared_normal
            .times(&Exact::product_of(radius, radius)?)?
            .minus(&reach.times(&reach)?);
        Some((
            room.nearest_f64(),
            squared_normal.nearest_f64(),
            reach.nearest_f64(),
        ))
    };
    let Some((room, squared_normal, reach)) = worked_out() else {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    };
    let across_the_plane = reach / squared_normal;
    let middle = ball.centre() - flat.normal() * across_the_plane;
    let (across, and) = flat.basis();
    let radius_here = (room / squared_normal).max(0.0).sqrt();

    Ok(Conic {
        centre: middle,
        across: across * radius_here,
        and: and * radius_here,
        size: how_big_this_is(&[middle, ball.centre()], &[radius, radius_here]),
    })
}

/// The ellipse where a plane cuts a pipe it leans across.
///
/// The centre is where the axis meets the plane. Across the pipe the ellipse is
/// the pipe's own radius; along the plane's slope it is that radius stretched by
/// how far the plane leans, which is `√(n·n) / |n·w|` — and that is a division
/// by the very number "is this plane parallel to the axis" decided is not zero.
fn where_a_plane_cuts_a_pipe(flat: &Plane, pipe: &Cylinder) -> Result<Conic> {
    let normal = flat.normal();
    let along = pipe.axis().direction();
    let leans = normal.dot(along);
    assert!(
        leans != 0.0,
        "a plane along a pipe's axis crosses it in straight lines, which are walked by their ends"
    );

    let up_the_axis = (flat.offset() - normal.dot(pipe.axis().origin() - Point::origin())) / leans;
    let middle = pipe.axis().origin() + along * up_the_axis;

    let (across, and) = two_directions_in(flat, along);
    let radius_across = pipe.radius() / along.length();
    let radius_down_the_slope = pipe.radius() * normal.length() / leans.abs();

    Ok(Conic {
        centre: middle,
        across: across * radius_across,
        and: and * radius_down_the_slope,
        size: how_big_this_is(
            &[middle, pipe.axis().origin()],
            &[pipe.radius(), radius_across, radius_down_the_slope],
        ),
    })
}

/// Two directions in `flat`, square to each other, the first of them square to
/// `along` as well.
///
/// **The subtraction on the second line is what keeps the published bound a
/// bound**, and it is easy to leave out. `along × normal` is worked out to an
/// absolute rounding of about `u`, and its own length is the sine of the angle
/// between the two — so for a plane nearly square to the pipe the direction that
/// comes out of scaling it to length one is tilted *out of the plane* by about
/// `u / sin`, and every point built on it misses the plane by the radius times
/// that. Taking the part of it that lies in the plane leaves a tilt of order `u`
/// squared instead, which is nothing.
///
/// Where the plane is square to the axis the cross product has no length at all,
/// and there is nothing to work out: the crossing is a plain circle and any two
/// directions across the pipe will do. [`Cylinder::basis`] is the fixed pair the
/// pipe already answers with, so the same pipe always gives the same one.
fn two_directions_in(flat: &Plane, along: Vector) -> (Vector, Vector) {
    let normal = flat.normal();
    let leaning = along.cross(normal);
    let across = leaning - normal * (leaning.dot(normal) / normal.length_squared());
    match across.normalized() {
        Ok(across) => {
            let and = across.cross(normal);
            let and = and - normal * (and.dot(normal) / normal.length_squared());
            (
                across,
                and.normalized()
                    .expect("two square directions cross into a third with a length"),
            )
        }
        Err(_) => flat.basis(),
    }
}

/// The circle where two balls cross, which lies in their radical plane.
///
/// The radius squared is `−(D² − (R₁+R₂)²)(D² − (R₁−R₂)²) / 4D²` — the two
/// discriminants that named the crossing, multiplied together. Written that way
/// rather than as `R₁² − t²D²` for the same reason as the ball above: the second
/// form cancels to noise for two balls that barely reach each other, and this one
/// is a product of two numbers each known to full relative accuracy.
fn where_two_balls_cut(one: &Sphere, other: &Sphere) -> Result<Conic> {
    let [here, there] = [one, other].map(|ball| ball.centre().coordinates());
    let [mine, theirs] = [one, other].map(Sphere::radius);
    let apart = other.centre() - one.centre();

    let worked_out = || {
        let squared_gap = (0..3).try_fold(Exact::of(0.0), |so_far, axis| {
            let gap = Exact::of(there[axis]).minus(&Exact::of(here[axis]));
            Some(so_far.plus(&gap.times(&gap)?))
        })?;
        let apart_by = |sign: f64| {
            let together = Exact::product_of(mine, sign * theirs)?;
            let reach = Exact::product_of(mine, mine)?
                .plus(&together)
                .plus(&together)
                .plus(&Exact::product_of(theirs, theirs)?);
            Some(squared_gap.minus(&reach))
        };
        let room = apart_by(1.0)?.times(&apart_by(-1.0)?)?;
        Some((room.nearest_f64(), squared_gap.nearest_f64()))
    };
    let Some((room, squared_gap)) = worked_out() else {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    };
    let part = (squared_gap + mine * mine - theirs * theirs) / (2.0 * squared_gap);
    let middle = one.centre() + apart * part;
    let radius_here = (-room / (4.0 * squared_gap)).max(0.0).sqrt();

    let radical = Plane::from_point_and_normal(middle, apart)?;
    let (across, and) = radical.basis();
    Ok(Conic {
        centre: middle,
        across: across * radius_here,
        and: and * radius_here,
        size: how_big_this_is(
            &[middle, one.centre(), other.centre()],
            &[mine, theirs, radius_here],
        ),
    })
}

/// The largest magnitude anything about a conic was worked out from.
fn how_big_this_is(places: &[Point], lengths: &[f64]) -> f64 {
    let widest = places
        .iter()
        .map(Point::largest_coordinate_magnitude)
        .chain(lengths.iter().map(|length| length.abs()))
        .fold(0.0, f64::max);
    // A conic of no size at all still has its points worked out by arithmetic
    // that rounds against the smallest normal number, and a size of zero would
    // publish a bound of zero.
    widest.max(f64::MIN_POSITIVE)
}

/// The box a computed point is really known to be inside.
///
/// Handed to a predicate as an exact triple instead, a point that came out of
/// trigonometry buys a certified sign about noise (§2.5, §27.7).
fn somewhere_within(point: Point, deviation: f64) -> PredicatePoint {
    PredicatePoint::Constructed(point.coordinates().map(|coordinate| {
        Interval::new(coordinate - deviation, coordinate + deviation)
            .expect("a deviation is a positive length, so the low end is below the high one")
    }))
}

/// `angle` moved into `[0, 2π)`.
fn wrapped(angle: f64) -> f64 {
    let whole_turn = core::f64::consts::TAU;
    let inside = angle % whole_turn;
    if inside < 0.0 {
        inside + whole_turn
    } else {
        inside
    }
}

/// What a surface is called in a refusal a person has to read.
fn name_of(surface: &Surface) -> &'static str {
    match surface {
        Surface::Plane(_) => "plane",
        Surface::Sphere(_) => "ball",
        Surface::Cylinder(_) => "pipe",
    }
}
