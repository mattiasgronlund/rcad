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
    ProvenanceLog, Record,
};

use crate::exact::Exact;
use crate::march::{march, settled_with_a_bound};
use crate::meet::nowhere_in_particular;
use crate::{
    Cylinder, Deflection, EdgeRun, Error, NamedRun, Picker, Plane, Point, Result, Sphere, Surface,
    Vector,
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
/// [`Error::Undecidable`] when the conic's own size could not be worked out
/// because a product fell out of range.
///
/// [`Error::AnEndOfARunNamesNoPlace`] when an end is neither the box that says
/// nothing at all is known nor a box with a middle.
pub fn walk(
    run: &NamedRun,
    along: [&Surface; 2],
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<PointsAlongACrossing> {
    if !run.ends().into_iter().all(an_end_that_names_a_place) {
        return Err(Error::AnEndOfARunNamesNoPlace);
    }
    // A straight run is the whole of what its two ends say, so there is nothing
    // to put between them.
    if run.run() == &EdgeRun::Straight {
        return Ok(nothing_between_the_ends());
    }

    let pair = the_two_surfaces_it_runs_between(run, along);
    if a_crossing_that_runs_to_a_quartic(&pair) {
        // Two pipes down one direction cross in straight lines, and a run that
        // is one of those is picked out by a half-space rather than being
        // `Straight`, so it reaches here — with nothing to put between its ends.
        if let [Surface::Cylinder(one), Surface::Cylinder(other)] = pair
            && one.axis().direction().cross(other.axis().direction()) == Vector::zero()
        {
            return Ok(nothing_between_the_ends());
        }
        return follow_the_quartic(run, pair, deflection, policy, log);
    }

    let Some(conic) = the_conic_between(pair)? else {
        return Ok(nothing_between_the_ends());
    };
    let deviation = POINT_ALONG_A_CROSSING_ROUNDOFF * conic.the_size_it_was_worked_out_from();
    // A conic drawn near the top of `f64`'s range has a bound that overflows,
    // and a bound of infinity is not a bound. The size of this crossing is what
    // could not be worked out, which is the same refusal the naming half has one
    // step earlier.
    if !deviation.is_finite() {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    }
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

/// Whether this end says where it is.
///
/// Two ways it can. The box whose every side runs the whole number line is how a
/// pair of surfaces says it names a curve and nothing on it, and that is an
/// answer rather than a gap. Anything else has to have a middle — because the
/// middle is what the angle round the conic is worked out from, and an angle
/// that is not a number turns every point built on it into one, right down to a
/// box that is not a range.
///
/// **A caller can hand over neither**, which is why this is asked at all. An
/// exact triple with a `NaN` coordinate, or a box with one infinite side, are
/// both things [`NamedRun::new`] takes.
fn an_end_that_names_a_place(end: PredicatePoint) -> bool {
    end == nowhere_in_particular()
        || end
            .enclosure()
            .iter()
            .all(|side| middle_of(*side).is_finite())
}

/// The middle of an enclosure, which is where a point known only as a box is
/// read as being.
fn middle_of(side: Interval) -> f64 {
    (side.lo() + side.hi()) / 2.0
}

/// Whether this pair crosses in a curve no conic stands in for.
///
/// A ball against a pipe, or two pipes. Every other pair crosses in something
/// that lies in a plane (§24.5).
fn a_crossing_that_runs_to_a_quartic(pair: &[Surface; 2]) -> bool {
    matches!(
        pair,
        [Surface::Sphere(_), Surface::Cylinder(_)]
            | [Surface::Cylinder(_), Surface::Sphere(_)]
            | [Surface::Cylinder(_), Surface::Cylinder(_)]
    )
}

/// A run of a degree-four crossing, followed by the marcher.
///
/// # The budget is split down the middle
///
/// Half the deflection goes to the straight pieces between the points and half
/// to the points themselves. The closed form works the other way round — it
/// knows its per-point bound before it starts and spends what is left on the
/// pieces — and the marcher cannot, because how far off a point sits is
/// something it finds out by settling the point. So the split is fixed in
/// advance and the half spent on the points is checked afterwards, under the
/// policy, exactly as the closed form checks its own.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the half-spaces on the
/// run pick both of the loops the crossing leaves, or neither.
///
/// [`Error::CannotPlaceAPointThatCloseToACrossing`] when the marcher's own bound
/// takes more than its half of the deflection and the policy will not accept it.
///
/// The marcher's two refusals, and
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a seed cannot be worked out at
/// all.
fn follow_the_quartic(
    run: &NamedRun,
    pair: [Surface; 2],
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<PointsAlongACrossing> {
    let half_the_budget = deflection.length() / 2.0;
    let surfaces = [&pair[0], &pair[1]];
    let picked_by: &[Picker] = match run.run() {
        EdgeRun::PartOfWhereItCrosses { picked_by, .. } => picked_by,
        _ => &[],
    };

    // A run that nothing picks is the whole of a single loop, and the two seeds
    // are two places on it — either will do. A run that *is* picked is one of
    // two loops, and then exactly one of the seeds may pass: both or neither and
    // the half-spaces have not told them apart after all.
    let mut seeds = Vec::new();
    for seed in where_a_quartic_can_be_picked_up(&pair)? {
        let (settled, off) = settled_with_a_bound(surfaces, seed)?;
        if picked_by.is_empty() || every_half_space_holds(picked_by, settled, off, log) {
            seeds.push(settled);
        }
    }
    let seed = match (picked_by.is_empty(), seeds.as_slice()) {
        (true, [first, ..]) => first,
        (false, [only]) => only,
        _ => return Err(Error::CannotNameWhichPieceOfACrossingThisIs),
    };

    let walked = march(
        surfaces,
        *seed,
        half_the_budget,
        MOST_POINTS_ALONG_ONE_CROSSING,
    )?;
    let tag = Approximation::new(walked.deviation, Cause::PointAlongACrossing);
    if walked.deviation > half_the_budget && !policy.wobble_is_acceptable(walked.deviation) {
        return Err(Error::CannotPlaceAPointThatCloseToACrossing {
            asked: deflection.length(),
            closest: walked.deviation,
        });
    }
    Ok(PointsAlongACrossing {
        points: walked.points,
        deviation: tag,
    })
}

/// One point near each piece a degree-four crossing can come in, for the marcher
/// to start from.
///
/// **Where the two pieces are pulled apart matters.** A ball and a pipe leave
/// two loops either side of the plane through the centre square to the axis, so
/// the two seeds are the far ends of the same reach along that axis. Two pipes
/// leave two loops either side of the plane the naming half picked them apart
/// with, and reaching both of them means walking round the **thinner** of the
/// two: on the fatter one the same two loops show up as two arcs of the same
/// sweep, and a pair of seeds taken there would both land on one of them.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a seed comes out as something
/// that is not a point.
fn where_a_quartic_can_be_picked_up(pair: &[Surface; 2]) -> Result<Vec<Point>> {
    let seeds = match pair {
        [Surface::Sphere(ball), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Sphere(ball)] => {
            where_a_ball_reaches_across_a_pipe(ball, pipe)
        }
        [Surface::Cylinder(one), Surface::Cylinder(other)] => {
            let certified = |pipe: &Cylinder| pipe.radius() / pipe.axis().direction().length();
            if certified(one) <= certified(other) {
                where_one_pipe_reaches_across_another(one, other)
            } else {
                where_one_pipe_reaches_across_another(other, one)
            }
        }
        [first, second] => panic!("{first:?} and {second:?} do not cross in a quartic"),
    };
    if seeds.iter().any(|seed| !seed.is_finite()) {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    }
    Ok(seeds.to_vec())
}

/// The two places a ball reaches furthest along a pipe's axis, one on each of
/// the two loops it can leave.
///
/// Both are on the ray from the axis out towards the ball's centre, where the
/// crossing runs furthest from the plane through the centre square to the axis.
/// A ball centred exactly on the axis has no such ray and any direction across
/// the pipe will do, so the pipe's own basis stands in.
fn where_a_ball_reaches_across_a_pipe(ball: &Sphere, pipe: &Cylinder) -> [Point; 2] {
    let along = pipe.axis().direction() * pipe.axis().direction().length().recip();
    let skin = pipe.radius() / pipe.axis().direction().length();
    let out = ball.centre() - pipe.axis().origin();
    let up = out.dot(along);
    let sideways = out - along * up;
    let (towards, off_the_axis) = match sideways.normalized() {
        Ok(towards) => (towards, sideways.length()),
        Err(_) => (pipe.basis().0, 0.0),
    };

    let radius = ball.radius();
    let nearest = skin - off_the_axis;
    let reach = (radius * radius - nearest * nearest).max(0.0).sqrt();
    let middle = pipe.axis().origin() + along * up + towards * skin;
    [middle + along * reach, middle - along * reach]
}

/// The two places one pipe reaches furthest along its own axis through another,
/// one on each of the two loops the pair can leave.
///
/// `thin` is walked round to the angle where the other pipe reaches furthest,
/// and the two roots of the quadratic in how far along its axis the point sits
/// are the two seeds. Where the pair leaves only one loop both roots are on it.
fn where_one_pipe_reaches_across_another(thin: &Cylinder, fat: &Cylinder) -> [Point; 2] {
    let [thin_way, fat_way] = [thin, fat].map(|pipe| {
        let along = pipe.axis().direction();
        along * along.length().recip()
    });
    let [thin_skin, fat_skin] =
        [thin, fat].map(|pipe| pipe.radius() / pipe.axis().direction().length());
    let leaning = thin_way.dot(fat_way);
    let sideways = thin_way.cross(fat_way);
    let across = sideways * sideways.length().recip();

    // The foot of the common perpendicular on the thin pipe's axis, which is
    // where its own sweep is measured from.
    let between = fat.axis().origin() - thin.axis().origin();
    let up = (between.dot(thin_way) - between.dot(fat_way) * leaning) / (1.0 - leaning * leaning);
    let foot = thin.axis().origin() + thin_way * up;
    let and_up = {
        let up = fat_way - thin_way * leaning;
        up * up.length().recip()
    };

    // The sweep that reaches furthest is the one where the fat pipe's axis comes
    // nearest, and trying a handful of candidates is cheaper than solving for
    // it: the two that put the axis nearest, and the three ends of the range.
    let apart = between.dot(across);
    let at = |turned: f64| {
        let turned = turned.clamp(-1.0, 1.0);
        foot + across * (thin_skin * turned)
            + and_up * (thin_skin * (1.0 - turned * turned).max(0.0).sqrt())
    };
    let mut round = at(apart / thin_skin);
    let (mut room, mut middle) = how_far_a_pipe_reaches_from(fat, round, thin_way, fat_skin);
    for turned in [-apart / thin_skin, 1.0, -1.0, 0.0] {
        let here = at(turned);
        let (reaches, along) = how_far_a_pipe_reaches_from(fat, here, thin_way, fat_skin);
        if reaches > room {
            (room, middle, round) = (reaches, along, here);
        }
    }

    let reach = room.max(0.0).sqrt();
    [
        round + thin_way * (middle + reach),
        round + thin_way * (middle - reach),
    ]
}

/// How far along `thin_way` from `round` the pipe `fat` is reached, as the
/// discriminant of the quadratic and the distance to its middle.
fn how_far_a_pipe_reaches_from(
    fat: &Cylinder,
    round: Point,
    thin_way: Vector,
    fat_skin: f64,
) -> (f64, f64) {
    let fat_way = fat.axis().direction() * fat.axis().direction().length().recip();
    let out = round - fat.axis().origin();
    let sideways = thin_way.cross(fat_way);
    let leaning = sideways.length_squared();
    let straight_on = out.cross(fat_way).dot(sideways);
    let already = out.cross(fat_way).length_squared() - fat_skin * fat_skin;
    (
        (straight_on * straight_on - leaning * already) / (leaning * leaning),
        -straight_on / leaning,
    )
}

/// Whether the box `at` is known to be inside is on or outside every one of
/// `picked_by`.
fn every_half_space_holds(
    picked_by: &[Picker],
    at: Point,
    deviation: f64,
    log: &mut ProvenanceLog,
) -> bool {
    let at = somewhere_within(at, deviation);
    let mut on_or_outside = true;
    for picker in picked_by {
        if which_side_of(picker, at, log) == Decision::Neg {
            on_or_outside = false;
        }
    }
    on_or_outside
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
///
/// # Panics
///
/// On a pair that crosses in a degree-four curve.
/// [`a_crossing_that_runs_to_a_quartic`] matches exactly those three pairs and
/// [`walk`] hands every one of them to the marcher before reaching here, so this
/// is a broken invariant of this file and not something a caller can ask for.
/// **It used to be a refusal**, `CrossingRunsToAQuartic`, and the claim audit
/// found that the second stage had filled the arm and left the variant behind:
/// a public refusal nothing could produce, whose own doc said the evaluator
/// still raised it.
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
        [first, second] => panic!(
            "{first:?} and {second:?} cross in a quartic, so the marcher \
             should already have this run"
        ),
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

    /// The largest magnitude anything about this conic was worked out from,
    /// which is what [`POINT_ALONG_A_CROSSING_ROUNDOFF`] is a multiple of.
    fn the_size_it_was_worked_out_from(&self) -> f64 {
        self.size
    }

    /// The furthest from its own centre this conic goes.
    ///
    /// **Not [`Conic::the_size_it_was_worked_out_from`], and the difference
    /// decides how many pieces a stretch of it is walked into.** That one is the
    /// biggest number in the chain — a plane's offset, a ball's centre, an axis
    /// origin — and it is the right scale for a *rounding*. It is the wrong one
    /// for a *bend*: the cap circle at the top of a bar ten tall bends exactly as
    /// the one at the bottom does, and charging the top one against the ten
    /// walked it into more pieces than the bottom, which put the middle of a
    /// meshed bar off half way up. Where a circle sits is not allowed to change
    /// how finely it is cut (§19).
    fn how_far_it_reaches(&self) -> f64 {
        self.across.length().max(self.and.length())
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
            (end != nowhere_in_particular()).then(|| Point::from(end.enclosure().map(middle_of)))
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
        picked_by: &[Picker],
        middle: f64,
        deviation: f64,
        log: &mut ProvenanceLog,
    ) -> bool {
        every_half_space_holds(picked_by, self.at(middle), deviation, log)
    }
}

/// Which side of a half-space the box `at` is on, recording the answer when
/// nothing could tell.
///
/// A half-space that could not tell rules out neither piece, so both come back
/// picked and the caller is refused rather than handed a coin toss. It is
/// recorded because it happened, and because the refusal that follows says
/// nothing about which half-space gave up.
fn which_side_of(picker: &Picker, at: PredicatePoint, log: &mut ProvenanceLog) -> Decision {
    let decision = picker.side_of_point(at);
    let named = picker.predicate_name();
    let measures = match picker {
        Picker::InFrontOf(_) => Measures::Length,
        Picker::OutsideABall(_) | Picker::OutsideAPipe(_) => Measures::Area,
    };
    match decision {
        Decision::Pos | Decision::Zero | Decision::Neg => (),
        Decision::Undecidable { .. } => log.push(Record::new(
            named,
            &[at],
            Level::Interval,
            decision,
            measures,
        )),
        other => panic!("a half-space came back with a decision nothing knows: {other:?}"),
    }
    decision
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
    // **Scaled to length one from wherever it is**, and that is not fussiness.
    // A plane a hair off square to the pipe crosses into something whose
    // components are around `1e-180`, and squaring those to take a length puts
    // them in the subnormals and brings them back as noise — so the ordinary
    // normalize hands back a vector that is not a unit vector, the two semi-axes
    // stop being the same length, and every point built on them misses the pipe
    // by a share of its radius that no published bound covers.
    match across.scaled_to_length_one_from_anywhere("direction across a crossing") {
        Ok(across) => {
            let and = across.cross(normal);
            let and = and - normal * (and.dot(normal) / normal.length_squared());
            (
                across,
                and.scaled_to_length_one_from_anywhere("direction along a crossing")
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
        // How far along the centre line the circle sits, as a numerator only:
        // `D² + R₁² − R₂²`. **Summed exactly and for the same reason as the
        // radius**, and this one was found by a fuzzer rather than by argument.
        // Two balls of almost the same size whose centres are almost in the same
        // place have `R₁² − R₂²` cancelling to a millionth of either square, and
        // that share is then multiplied by a `D²` in the denominator which is
        // smaller still — so a relative rounding of the difference comes out of
        // the division magnified by however many times `D` goes into the radius,
        // and the centre lands nowhere near the published bound of the answer.
        let along_the_centre_line = squared_gap
            .plus(&Exact::product_of(mine, mine)?)
            .minus(&Exact::product_of(theirs, theirs)?);
        Some((
            room.nearest_f64(),
            squared_gap.nearest_f64(),
            along_the_centre_line.nearest_f64(),
        ))
    };
    let Some((room, squared_gap, along_the_centre_line)) = worked_out() else {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    };
    let part = along_the_centre_line / (2.0 * squared_gap);
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

#[cfg(test)]
mod tests {
    use smallvec::smallvec;

    use super::*;
    use crate::Vector;

    fn a_ball_and_the_meridian_of_its_seam() -> (Surface, Surface, Picker) {
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a ball"));
        let meridian = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal"),
        );
        let picker = Picker::from(
            crate::HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
                .expect("the x axis has length"),
        );
        (ball, meridian, picker)
    }

    /// An end that does not say where it is is refused, and does not take the
    /// arithmetic with it.
    ///
    /// **Found by the claim audit, and reachable from outside the workspace.**
    /// `walk` is public and so is `NamedRun::new`, and nothing between them
    /// narrows what an end may be: a `Point` is a triple *or a box*, and neither
    /// `f64` nor `Interval` refuses the two shapes here. Both make the angle
    /// round the conic not a number, which used to reach `Interval::new(NaN,
    /// NaN)` and panic — a panic on a caller's argument, which §4.1 says is the
    /// wrong answer.
    ///
    /// The box with every side running the whole number line is the case this
    /// must *not* catch: that is how a pair of surfaces says it names a curve
    /// and nothing on it, and `walk` has to walk the whole curve for it.
    #[test]
    fn an_end_that_does_not_say_where_it_is_is_refused_rather_than_walked() {
        let (ball, meridian, picker) = a_ball_and_the_meridian_of_its_seam();
        let north = PredicatePoint::from([0.0, 0.0, 5.0]);
        let nowhere_useful = [
            PredicatePoint::from([f64::NAN, 0.0, 0.0]),
            PredicatePoint::Constructed([
                Interval::new(0.0, f64::INFINITY).expect("a half-open range"),
                Interval::exactly(0.0),
                Interval::exactly(0.0),
            ]),
        ];

        for end in nowhere_useful {
            let run = NamedRun::new(
                EdgeRun::PartOfWhereItCrosses {
                    and: meridian,
                    picked_by: smallvec![picker],
                },
                [north, end],
            );
            assert_eq!(
                walk(
                    &run,
                    [&ball, &ball],
                    Deflection::of(0.1).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                ),
                Err(Error::AnEndOfARunNamesNoPlace),
                "an end of {end:?} said where it was after all"
            );
        }
    }

    /// A conic too big to publish a bound for is refused, and the same pair
    /// written smaller is walked.
    ///
    /// **A gap and not a limit**, which is what the refusal table at §30 needs a
    /// demonstration for. An ellipse a plane cuts out of a pipe it is nearly
    /// parallel to is stretched by `‖n‖ / |n·w|` down the slope, so a fat enough
    /// pipe and a shallow enough tilt put its long semi-axis past the top of
    /// `f64` — and a bound of infinity is not a bound, so
    /// [`Approximation::new`]'s "a magnitude is a real amount" would be the next
    /// thing to fail. The answer exists; this arithmetic at this scale could not
    /// reach it, and the second half of the test is what says so.
    #[test]
    fn a_crossing_too_big_to_put_a_bound_on_is_refused_and_the_same_one_smaller_is_walked() {
        let shallow = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 1e-20))
                .expect("a direction with length"),
        );
        // The deflection scales with the shape, because "written smaller" means
        // the whole question and not only the pipe.
        let walked = |radius: f64, deflection: f64| {
            let pipe =
                Surface::Cylinder(Cylinder::new(crate::Axis::z(), radius).expect("a radius"));
            walk(
                &NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross),
                [&pipe, &shallow],
                Deflection::of(deflection).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .map(|along| along.points().len())
        };

        assert_eq!(
            walked(1e300, 1e298),
            Err(Error::CannotWorkOutHowBigACrossingIs)
        );
        assert!(
            walked(1e10, 1e28).is_ok_and(|points| points > 8),
            "the same pipe and plane written smaller are walked: {:?}",
            walked(1e10, 1e28)
        );
    }

    /// The whole of a curve still walks, and that is what stops the check above
    /// from being a check that nothing gets through.
    #[test]
    fn a_run_with_no_ends_at_all_is_walked_right_round() {
        let (ball, meridian, _) = a_ball_and_the_meridian_of_its_seam();
        let whole = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross);
        let along_it = walk(
            &whole,
            [&ball, &meridian],
            Deflection::of(0.1).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a pair of surfaces names a whole circle");
        assert!(along_it.points().len() > 8);
    }
}
