//! Following a crossing that no conic stands in for.
//!
//! Four of the six pairs cross in a curve that lies in a plane, and
//! [`walk`](crate::walk) parametrizes those in closed form. A ball crossing a
//! pipe off its axis, and two pipes leaning across each other, cross in a
//! degree-four space curve instead. This is what draws one.
//!
//! # Why a marcher at all, when the answer is already named
//!
//! §24.1: *"the marcher is built where the answer is already known in closed
//! form, because that is the only setting where its tangency and near-tangency
//! branches have a ground truth to be judged against."* So this function is
//! deliberately general over the surface kinds rather than written for the two
//! quartic pairs alone: run over a ball threaded on a pipe's axis it draws two
//! circles whose radius and height are known exactly, and the difference between
//! what it draws and what the closed form names is §24.18's criterion 7.
//!
//! # What is certified and what is not
//!
//! The stepping is ordinary `f64`, and none of it is trusted. What is certified
//! is the **deviation**: after each point is settled onto both surfaces, an
//! enclosure of how far it really sits from each of them is worked out, and
//! divided by the sine of the angle between them. That is the radius of the
//! region a point that close to both surfaces can be in, and the largest of them
//! along a run is the number [`walk`](crate::walk) publishes.
//!
//! **Which is why near-tangency refuses rather than drifting.** As the two
//! surfaces come parallel that sine goes to nothing and the bound goes up
//! without limit, so a marcher creeping along a nearly-tangent crossing does not
//! quietly produce points — it says how far out they could be, and the policy
//! decides. Roadmap §6.3 calls near-tangential contact the place kernels produce
//! silent garbage; this is what stops the silence.

use rcad_predicates::Interval;

use crate::enclosure::{
    cross_enclosing, dot_enclosing, exactly, gap_enclosing, quotient_or_everything, root_enclosing,
    squared_length_enclosing,
};
use crate::{Cone, Cylinder, Error, Plane, Point, Result, Sphere, Surface, Torus, Vector};

/// How much of the deflection budget the sagitta of one straight piece may take
/// before the step is halved.
///
/// Three quarters. The remaining quarter is room for the chord's own ends being
/// a settled point rather than an exact one, and for the sagitta being measured
/// at the middle rather than searched for along the whole piece.
const SAGITTA_MAY_TAKE: f64 = 0.75;

/// How much of a straight piece's *own length* its sagitta may take.
///
/// [`SAGITTA_MAY_TAKE`] asks whether a piece is inside the caller's budget, and
/// a caller may hand over a budget larger than the whole crossing — the coarse
/// walk in `crates/rcad-geom/tests/fuzzing/mod.rs` asks `f64::MAX / 4.0` on
/// purpose, to find out how big a crossing is before asking for the real one.
/// Against a budget like that the question decides nothing, and a step of
/// `2.2e307` along the tangent is accepted wherever the settling happens to land
/// it. This is the question that does not go away: **a straight piece is a piece
/// of the curve only if it is straighter than it is long.**
///
/// A quarter. On a circular arc the sagitta is `tan(θ/4)/2` of the chord, so the
/// quarter-turn rule below sits at `0.2071` and a quarter leaves a little room
/// over it for curves that are not circles. What it rejects measured `0.44` to
/// `0.59` — half a turn and worse, a chord drawn clean across the curve rather
/// than along it (`docs/decisions.md` §89).
const SAGITTA_MAY_TAKE_OF_ITS_CHORD: f64 = 0.25;

/// How many times a step may be halved before the marcher gives up on it.
///
/// Fifty, which is a thousandth of a millionth of a millionth of the step it
/// started at — and the sagitta it is being judged on falls with the *square* of
/// the step, so fifty halvings ask a piece to bend `1e-30` of what the first one
/// did. Reaching the end of that means the curve is not being followed at all
/// rather than that the budget was tight.
///
/// **Not "fifty halvings puts any step below the smallest normal number"**,
/// which is what this used to say and is out by twenty binary orders of
/// magnitude: `2^-50` of a step of one is `9e-16`, and the smallest normal number
/// is `2e-308`. The claim audit caught it. Fifty is right for the reason above
/// and the arithmetic in the old reason was wrong.
const HALVINGS_BEFORE_GIVING_UP: usize = 50;

/// How many multiples of the deflection budget one step may grow to.
///
/// A step doubles whenever the last one's sagitta was comfortably inside the
/// budget, so a large gentle loop is not walked at the step a tight corner
/// needed. A million, and what it stops is a loop so gentle that the step runs
/// past the whole curve and the walk closes on its second point without having
/// gone anywhere.
const MOST_A_STEP_MAY_GROW_TO: f64 = 1e6;

/// How many rounds of settling a point onto both surfaces.
///
/// Eight. Newton doubles its digits each round from anywhere near the answer, so
/// eight is four more than the fifty-three bits need and the extra rounds cost
/// only where the point was not near.
const ROUNDS_OF_SETTLING: usize = 8;

/// One run of a crossing, followed into points.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Marched {
    /// The points, in the order they were walked, the seed not among them.
    pub(crate) points: Vec<Point>,
    /// How far the furthest of them may sit off the crossing.
    pub(crate) deviation: f64,
}

/// Follow the crossing of two surfaces from `seed`, right round if it closes.
///
/// `seed` only has to be near the crossing; it is settled onto both surfaces
/// before anything else happens. `room` is how far the straight pieces between
/// the points may stray from the curve, and `most` caps how many points one run
/// is allowed.
///
/// # The step rule
///
/// Three questions, and the last two are what make this a *marcher* rather than
/// a sampler. **How far it bends**: the middle of each straight piece is settled
/// onto the curve and the distance it moved is that piece's sagitta, measured
/// rather than derived, and a step whose sagitta takes more than
/// [`SAGITTA_MAY_TAKE`] of `room` is shortened and tried again. **How far it
/// bends across itself**: the same sagitta against the piece's own length rather
/// than against the budget, because a caller may hand over a budget larger than
/// the whole crossing and then the first question decides nothing
/// ([`SAGITTA_MAY_TAKE_OF_ITS_CHORD`]). **How far it turns**: no step may turn
/// the tangent by more than a quarter turn, because past a half turn a step
/// cannot be told from one the other way.
///
/// A step whose sagitta is comfortably inside the budget grows for the next one,
/// so a large gentle loop does not get walked at the step a tight corner needed.
///
/// # Why it terminates
///
/// Every accepted step moves the point somewhere it was not, and a step is only
/// halved [`HALVINGS_BEFORE_GIVING_UP`] times before the run is refused, so a
/// curve of finite length is walked in finitely many steps. **How far a step
/// moves is measured and not decreed**: the trial point is settled onto both
/// surfaces afterwards, so what the step actually covers is whatever the
/// settling left, and all that is checked is that it is not nothing. That makes
/// the argument one about the curve rather than about the arithmetic, so it is
/// not left to stand on its own: `most` caps the count outright, and reaching
/// that cap is a refusal rather than a truncated answer.
///
/// # Errors
///
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] when the two
/// surfaces are too nearly parallel at some point along the run for a tangent
/// direction to be worked out at all.
///
/// [`Error::TooManyPointsAlongOneCrossing`] when the run takes more than `most`
/// points, and [`Error::CannotWorkOutHowBigACrossingIs`] when a point cannot be
/// settled onto both surfaces.
pub(crate) fn march(
    surfaces: [&Surface; 2],
    seed: Point,
    room: f64,
    most: usize,
) -> Result<Marched> {
    let start = settled(surfaces, seed)?;
    let mut deviation = how_far_off_the_crossing(surfaces, start)?;
    let mut points = Vec::new();
    let mut here = start;
    let mut step = room.max(f64::MIN_POSITIVE);
    let mut travelled = 0.0;

    loop {
        let tangent = which_way_the_crossing_runs(surfaces, here)?;
        let (next, taken) = one_step(surfaces, here, tangent, step, room)?;

        // Coming back past where the walk started closes the loop, and the seed
        // is the point the caller already has.
        let back_to_the_start = here - start;
        if travelled > 3.0 * taken && back_to_the_start.length() <= taken {
            return Ok(Marched { points, deviation });
        }

        deviation = deviation.max(how_far_off_the_crossing(surfaces, next)?);
        points.push(next);
        if points.len() > most {
            return Err(Error::TooManyPointsAlongOneCrossing { most });
        }
        travelled += taken;
        step = (taken * 2.0).min(room.max(f64::MIN_POSITIVE) * MOST_A_STEP_MAY_GROW_TO);
        here = next;
    }
}

/// `seed` settled onto both surfaces, with an upper bound on how far off the
/// crossing it still is.
///
/// What a caller needs before it can ask a half-space which piece of a crossing
/// a seed is on: a point that came out of trigonometry has to be handed over as
/// the box it is really known to be inside, and this is that box's radius
/// (§2.5, §27.7).
///
/// # Errors
///
/// The same two [`march`] has, for the same reasons.
pub(crate) fn settled_with_a_bound(surfaces: [&Surface; 2], seed: Point) -> Result<(Point, f64)> {
    let settled = settled(surfaces, seed)?;
    let off = how_far_off_the_crossing(surfaces, settled)?;
    Ok((settled, off))
}

/// `from` settled onto both surfaces, by Newton in the plane the two normals
/// span.
///
/// Each round moves the point by the two signed distances, corrected for the two
/// normals not being square to each other: with `c` the cosine of the angle
/// between them the step is `(s₁ − s₂c)/(1 − c²)` along the first and
/// `(s₂ − s₁c)/(1 − c²)` along the second. That is the exact solution of the
/// linearised pair, so on a quadric it converges as Newton does.
///
/// # Errors
///
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] when the two
/// normals are parallel enough that the correction cannot be worked out.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a round lands somewhere that
/// is not a point.
fn settled(surfaces: [&Surface; 2], from: Point) -> Result<Point> {
    let mut here = from;
    for _ in 0..ROUNDS_OF_SETTLING {
        let [(first, one_way), (second, other_way)] =
            surfaces.map(|surface| how_far_off_and_which_way(surface, here));
        let leaning = one_way.dot(other_way);
        let across = 1.0 - leaning * leaning;
        if across <= 0.0 || !across.is_finite() {
            return Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent);
        }
        let along_one = (first - second * leaning) / across;
        let along_other = (second - first * leaning) / across;
        here = here - one_way * along_one - other_way * along_other;
        if !here.is_finite() {
            return Err(Error::CannotWorkOutHowBigACrossingIs);
        }
    }
    Ok(here)
}

/// An upper bound on how far `at` really sits off the curve the two surfaces
/// cross in.
///
/// A point `d₁` from the first surface and `d₂` from the second is inside the
/// two slabs those distances describe, and where the surfaces lean against each
/// other at an angle whose sine is `s` the widest that region gets is
/// `(d₁ + d₂)/s`. Every step of that is enclosed rather than evaluated, so what
/// comes back is a bound and not an estimate.
///
/// **The sine is where near-tangency shows up.** Two surfaces coming parallel
/// make it small, the bound large, and the caller's policy is what decides
/// whether that is acceptable — which is the whole reason §24.1 wanted a marcher
/// whose answer could be judged.
///
/// # Errors
///
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] when the sine
/// cannot be told from zero, so no bound can be put on the distance at all.
fn how_far_off_the_crossing(surfaces: [&Surface; 2], at: Point) -> Result<f64> {
    let [first, second] = surfaces.map(|surface| how_far_off_enclosing(surface, at));
    let sideways = cross_enclosing(first.1, second.1);
    let leaning = root_enclosing(squared_length_enclosing(sideways));
    if leaning.lo() <= 0.0 {
        return Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent);
    }
    let reach = quotient_or_everything(first.0 + second.0, leaning);
    if !reach.hi().is_finite() {
        return Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent);
    }
    Ok(reach.hi())
}

/// Which way the crossing runs at `at`: the two normals crossed together, scaled
/// to length one.
///
/// # Errors
///
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] when the two
/// normals are parallel, so the crossing has no direction there.
fn which_way_the_crossing_runs(surfaces: [&Surface; 2], at: Point) -> Result<Vector> {
    let [(_, one_way), (_, other_way)] =
        surfaces.map(|surface| how_far_off_and_which_way(surface, at));
    one_way
        .cross(other_way)
        .normalized()
        .map_err(|_| Error::CannotFollowACrossingWhereItsSurfacesAreTangent)
}

/// One step along the crossing, shortened until the straight piece it leaves
/// behind stays inside `room`, stays straighter than it is long, and turns no
/// more than a quarter turn.
///
/// Returns where the step landed and how long it turned out to be.
///
/// # What a rejected trial is retried at
///
/// Half of what was asked, **unless the trial covered less than half of it**.
/// A trial that came back short never stepped along the curve at all: the
/// settling pulled it back, and it will land in the same place however far the
/// ask is halved. Measured, a chord of `1.1003e39` sat still over twenty-one
/// halvings, and the trial that finally passed did so because the quarter-turn
/// test drifted into accepting it, not because anything had converged. There the
/// chord is the one thing the trial did measure about the curve, so the next ask
/// comes from that instead.
///
/// **The two cases are kept apart rather than merged into `min`** because the
/// merged form also shortens steps in the ordinary fine walk, where the trial
/// came back a little short for honest reasons, and that moved the marcher's
/// published bound from `0.577` of its own enclosure to `0.871` — still a bound,
/// but past the three quarters `how_far_the_marcher_strays_from_a_crossing_that_is_a_circle`
/// publishes. Splitting on whether the trial overshot leaves the fine walk at
/// `0.577` exactly (`docs/decisions.md` §89).
///
/// # Errors
///
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] when shortening
/// the step [`HALVINGS_BEFORE_GIVING_UP`] times still leaves a piece that strays
/// too far, which means the curve is not being followed rather than that the
/// budget was tight.
fn one_step(
    surfaces: [&Surface; 2],
    here: Point,
    tangent: Vector,
    from: f64,
    room: f64,
) -> Result<(Point, f64)> {
    let mut step = from;
    for _ in 0..HALVINGS_BEFORE_GIVING_UP {
        let next = settled(surfaces, here + tangent * step)?;
        let straight_on = next - here;
        let middle = here + straight_on * 0.5;
        let sagitta = (settled(surfaces, middle)? - middle).length();
        let turned = which_way_the_crossing_runs(surfaces, next)?.dot(tangent);
        let chord = straight_on.length();
        if sagitta <= room * SAGITTA_MAY_TAKE
            && sagitta <= chord * SAGITTA_MAY_TAKE_OF_ITS_CHORD
            && turned >= 0.0
            && chord > 0.0
        {
            return Ok((next, chord));
        }

        // A trial that covered less than half of what it asked for never stepped
        // along the curve: the settling pulled it back, and it will land in the
        // same place however far the ask is halved, so the halvings run out with
        // the chord where it started. There the chord is the one thing the trial
        // did measure about the curve, so the next ask comes from that. A trial
        // that did step along the curve and only bent too far is halved as
        // before, which is what the published bound was measured under.
        step = if chord > 0.0 && chord * 2.0 < step {
            chord * 0.5
        } else {
            step * 0.5
        };
    }
    Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent)
}

/// How far `at` sits off `surface`, signed, and the unit normal there.
///
/// Ordinary `f64` on both counts, because both feed the settling iteration and
/// nothing downstream of that is trusted: what is certified is worked out again
/// by [`how_far_off_enclosing`] once the point has stopped moving.
fn how_far_off_and_which_way(surface: &Surface, at: Point) -> (f64, Vector) {
    match surface {
        Surface::Plane(flat) => (
            flat.signed_distance_to(at) / flat.normal().length(),
            flat.normal() * flat.normal().length().recip(),
        ),
        Surface::Sphere(ball) => {
            let out = at - ball.centre();
            let reach = out.length();
            (reach - ball.radius(), out * reach.recip())
        }
        Surface::Cylinder(pipe) => {
            let along = pipe.axis().direction();
            let out = at - pipe.axis().origin();
            let sideways = out - along * (out.dot(along) / along.length_squared());
            let reach = sideways.length();
            (
                (reach * along.length() - pipe.radius()) / along.length(),
                sideways * reach.recip(),
            )
        }
        // Nothing in this crate marches a crossing with a cone on it — every
        // pair that reaches the marcher is a quartic, and
        // `Error::ACrossingBetweenAConeAndAnotherCurvedSurface` refuses those
        // before a run exists to follow. The arm is here because a `match` on a
        // growing enum with a wildcard in it is where the next variant goes to
        // be ignored (§40), and it does the arithmetic rather than panicking so
        // that the day a cone reaches the marcher the answer is right rather
        // than absent. The distance is `(‖x_perp‖ - slope*x_along) / hypot(1, slope)`,
        // which is the honest distance to the nappe the point stands over.
        Surface::Cone(cone) => {
            let along = cone.axis().direction();
            let out = at - cone.apex();
            let up = out.dot(along) / along.length_squared();
            let sideways = out - along * up;
            let reach = sideways.length();
            let steepness = cone.slope().hypot(1.0);
            let outwards = sideways * reach.recip();
            (
                (reach - cone.slope() * up.abs()) / steepness,
                (outwards - along * (cone.slope() * up.signum() / along.length()))
                    * steepness.recip(),
            )
        }
        // Nothing in this crate marches a crossing with a torus on it either;
        // `Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic` and
        // `Error::ACrossingBetweenATorusAndAnotherCurvedSurface` refuse those
        // before a run exists to follow. The arm does the arithmetic for the
        // same reason the cone's does, and here it is worth more: a torus's
        // distance function is **exact and everywhere finite**, `‖(s − R, a)‖ −
        // r` with `s` the distance from the axis and `a` the distance along it,
        // so the day a seed exists the marcher can already settle onto one.
        Surface::Torus(ring) => {
            let along = ring.axis().direction();
            let out = at - ring.centre();
            let up = out.dot(along) / along.length_squared();
            let sideways = out - along * up;
            let across = sideways.length() - ring.ring_radius();
            let axial = up * along.length();
            let off_the_circle = across.hypot(axial);
            (
                off_the_circle - ring.tube_radius(),
                (sideways * (across / sideways.length()) + along * (axial / along.length()))
                    * off_the_circle.recip(),
            )
        }
    }
}

/// An upper bound on how far `at` sits off `surface`, and an enclosure of the
/// unit normal there.
fn how_far_off_enclosing(surface: &Surface, at: Point) -> (Interval, [Interval; 3]) {
    match surface {
        Surface::Plane(flat) => how_far_off_a_plane_enclosing(flat, at),
        Surface::Sphere(ball) => how_far_off_a_ball_enclosing(ball, at),
        Surface::Cylinder(pipe) => how_far_off_a_pipe_enclosing(pipe, at),
        Surface::Cone(cone) => how_far_off_a_cone_enclosing(cone, at),
        Surface::Torus(ring) => how_far_off_a_torus_enclosing(ring, at),
    }
}

/// `|n·p − d| / √(n·n)`, and `n / √(n·n)`.
fn how_far_off_a_plane_enclosing(flat: &Plane, at: Point) -> (Interval, [Interval; 3]) {
    let normal = exactly(flat.normal().components());
    let length = root_enclosing(squared_length_enclosing(normal));
    let off = quotient_or_everything(
        dot_enclosing(normal, exactly(at.coordinates())) - Interval::exactly(flat.offset()),
        length,
    );
    (
        furthest_from_zero(off),
        normal.map(|part| quotient_or_everything(part, length)),
    )
}

/// `| ‖p − c‖ − R |`, and `(p − c) / ‖p − c‖`.
fn how_far_off_a_ball_enclosing(ball: &Sphere, at: Point) -> (Interval, [Interval; 3]) {
    let out = gap_enclosing(ball.centre().coordinates(), at.coordinates());
    let reach = root_enclosing(squared_length_enclosing(out));
    (
        furthest_from_zero(reach - Interval::exactly(ball.radius())),
        out.map(|part| quotient_or_everything(part, reach)),
    )
}

/// `| ‖(p − o) × w‖ − r | / √(w·w)`, and the unit direction out from the axis.
fn how_far_off_a_pipe_enclosing(pipe: &Cylinder, at: Point) -> (Interval, [Interval; 3]) {
    let along = exactly(pipe.axis().direction().components());
    let squared_direction = squared_length_enclosing(along);
    let out = gap_enclosing(pipe.axis().origin().coordinates(), at.coordinates());
    let sideways: [Interval; 3] = {
        let up = quotient_or_everything(dot_enclosing(out, along), squared_direction);
        core::array::from_fn(|axis| out[axis] - along[axis] * up)
    };
    let reach = root_enclosing(squared_length_enclosing(sideways));
    let off = quotient_or_everything(
        reach * root_enclosing(squared_direction) - Interval::exactly(pipe.radius()),
        root_enclosing(squared_direction),
    );
    (
        furthest_from_zero(off),
        sideways.map(|part| quotient_or_everything(part, reach)),
    )
}

/// `| ‖(p − apex)_perp‖ − slope·|(p − apex)_along| | / √(1 + slope²)`, and the
/// unit normal there.
///
/// Unreachable from anything this crate marches, for the reason
/// [`how_far_off_and_which_way`]'s cone arm gives; written out rather than
/// stubbed for the same reason.
fn how_far_off_a_cone_enclosing(cone: &Cone, at: Point) -> (Interval, [Interval; 3]) {
    let along = exactly(cone.axis().direction().components());
    let squared_direction = squared_length_enclosing(along);
    let out = gap_enclosing(cone.apex().coordinates(), at.coordinates());
    let up = quotient_or_everything(dot_enclosing(out, along), squared_direction);
    let sideways: [Interval; 3] = core::array::from_fn(|axis| out[axis] - along[axis] * up);
    let reach = root_enclosing(squared_length_enclosing(sideways));
    let slope = Interval::exactly(cone.slope());
    let steepness = root_enclosing(Interval::exactly(1.0) + slope * slope);
    let leaning = if up.hi() < 0.0 {
        -Interval::exactly(1.0)
    } else {
        Interval::exactly(1.0)
    };
    let off = quotient_or_everything(reach - slope * up * leaning, steepness);
    let outwards: [Interval; 3] = core::array::from_fn(|axis| {
        let across = quotient_or_everything(sideways[axis], reach);
        let down = quotient_or_everything(
            slope * leaning * along[axis],
            root_enclosing(squared_direction),
        );
        quotient_or_everything(across - down, steepness)
    });
    (furthest_from_zero(off), outwards)
}

/// `| ‖(‖(p − c)_perp‖ − R, (p − c)·ŵ)‖ − r |`, and the unit normal there.
///
/// Unreachable from anything this crate marches, for the reason
/// [`how_far_off_and_which_way`]'s torus arm gives; written out rather than
/// stubbed for the same reason.
fn how_far_off_a_torus_enclosing(ring: &Torus, at: Point) -> (Interval, [Interval; 3]) {
    let along = exactly(ring.axis().direction().components());
    let direction_length = root_enclosing(squared_length_enclosing(along));
    let out = gap_enclosing(ring.centre().coordinates(), at.coordinates());
    let up = quotient_or_everything(dot_enclosing(out, along), squared_length_enclosing(along));
    let sideways: [Interval; 3] = core::array::from_fn(|axis| out[axis] - along[axis] * up);
    let reach = root_enclosing(squared_length_enclosing(sideways));
    let across = reach - Interval::exactly(ring.ring_radius());
    let axial = up * direction_length;
    let off_the_circle = root_enclosing(across * across + axial * axial);
    let outwards: [Interval; 3] = core::array::from_fn(|axis| {
        let out_of_the_tube = quotient_or_everything(sideways[axis] * across, reach)
            + quotient_or_everything(along[axis] * axial, direction_length);
        quotient_or_everything(out_of_the_tube, off_the_circle)
    });
    (
        furthest_from_zero(off_the_circle - Interval::exactly(ring.tube_radius())),
        outwards,
    )
}

/// The magnitude of an enclosure, as an enclosure of no width: how far the
/// furthest end of it is from zero.
fn furthest_from_zero(value: Interval) -> Interval {
    Interval::exactly(value.lo().abs().max(value.hi().abs()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Axis;

    fn a_ball(centre: [f64; 3], radius: f64) -> Surface {
        Surface::Sphere(Sphere::new(Point::from(centre), radius).expect("a radius"))
    }

    fn a_pipe(along: [f64; 3], through: [f64; 3], radius: f64) -> Surface {
        Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::from(through), Vector::from(along)).expect("a direction"),
                radius,
            )
            .expect("a radius"),
        )
    }

    /// The oracle §24.1 says the marcher exists to be judged against: a ball
    /// threaded on a pipe's axis crosses it in two circles whose radius and
    /// height are known exactly, so what the marcher draws can be differenced
    /// against a number rather than against itself.
    #[test]
    fn marching_a_ball_threaded_on_a_pipe_walks_a_circle_of_the_pipes_own_radius() {
        let ball = a_ball([0.0, 0.0, 0.0], 5.0);
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 3.0);
        let up = f64::sqrt(25.0 - 9.0);

        let walked = march([&ball, &pipe], Point::new(3.0, 0.0, up), 0.01, 1 << 20)
            .expect("a ball threaded on a pipe crosses it in a circle");

        assert!(walked.points.len() > 20, "got {}", walked.points.len());
        for point in &walked.points {
            assert!(
                (f64::hypot(point.x(), point.y()) - 3.0).abs() <= walked.deviation,
                "a point on this crossing is the pipe's radius from its axis: {point:?}"
            );
            assert!(
                (point.z() - up).abs() <= walked.deviation,
                "and it is √(R² − r²) up the axis: {point:?}"
            );
        }
    }

    /// A quartic, and the shape §24.5 says two pipes leave: the marcher follows
    /// one closed loop right round and comes back to where it started.
    #[test]
    fn marching_where_two_pipes_lean_across_each_other_closes_the_loop_it_started_on() {
        let one = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 3.0);
        let other = a_pipe([1.0, 0.0, 0.0], [0.0, 2.0, 0.0], 2.0);

        // `x² + y² = 9` and `(y − 2)² + z² = 4` both hold at `(√5, 2, 2)`.
        let walked = march([&one, &other], Point::new(2.2, 2.0, 2.0), 0.01, 1 << 20)
            .expect("these two pipes overlap in one closed loop");
        assert!(walked.points.len() > 20, "got {}", walked.points.len());

        let first = *walked.points.first().expect("more than twenty");
        let last = *walked.points.last().expect("more than twenty");
        assert!(
            (last - first).length() < 1.0,
            "a closed loop comes back to where it started, not to {last:?} from {first:?}"
        );
    }

    /// Two surfaces tangent along their crossing have no direction to follow,
    /// and saying so is the branch §24.1 built this to be able to watch.
    #[test]
    fn marching_along_a_crossing_where_the_two_surfaces_are_tangent_refuses_by_name() {
        let ball = a_ball([0.0, 0.0, 0.0], 3.0);
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 3.0);
        assert_eq!(
            march([&ball, &pipe], Point::new(3.0, 0.0, 0.0), 0.01, 1 << 20),
            Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent),
            "a ball inscribed in a pipe touches it all the way round, so there is no crossing \
             to follow"
        );
    }

    /// **The demonstration behind §29.6's third claim.**
    ///
    /// Where a crossing runs through itself the two surfaces are tangent at the
    /// crossing point, so there is no direction to follow there — and that is
    /// why the naming half refuses such a pair rather than handing back two
    /// pieces the evaluator could not draw. §24.10: a partly undecidable answer
    /// refuses whole.
    ///
    /// A ball one out from a pipe's axis with a radius of three reaches exactly
    /// the pipe's whole width, so the two loops it would otherwise leave pinch
    /// together at `(-2, 0, 0)`.
    #[test]
    fn marching_from_where_a_crossing_runs_through_itself_has_no_direction_to_follow() {
        let ball = a_ball([1.0, 0.0, 0.0], 3.0);
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
        assert_eq!(
            march([&ball, &pipe], Point::new(-2.0, 0.0, 0.0), 0.01, 1 << 20),
            Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent)
        );
    }

    /// The cap is a refusal and not a truncated answer, which is what stops a
    /// caller reading a partly walked loop as a whole one (§24.10).
    #[test]
    fn a_crossing_that_takes_more_points_than_it_is_allowed_refuses_rather_than_stopping_short() {
        let ball = a_ball([0.0, 0.0, 0.0], 5.0);
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 3.0);
        assert_eq!(
            march([&ball, &pipe], Point::new(3.0, 0.0, 4.0), 0.01, 4),
            Err(Error::TooManyPointsAlongOneCrossing { most: 4 })
        );
    }
}
