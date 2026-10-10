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
use crate::{
    Cone, Cylinder, Error, Plane, Point, Result, Sphere, Surface, Torus, Vector,
    WhereFollowingACrossingGaveUp,
};

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

/// How many times a step may be shortened before the marcher gives up on it.
///
/// Fifty. A try at worst halves the step and the chord branch below cuts it
/// further, so fifty of them reach at least a thousandth of a millionth of a
/// millionth of the step it started at — and the sagitta it is being judged on
/// falls with the *square* of the step, so fifty tries ask a piece to bend at
/// most `1e-30` of what the first one did. Reaching the end of that means the
/// curve is not being followed at all rather than that the budget was tight.
///
/// **Not "fifty halvings puts any step below the smallest normal number"**,
/// which is what this used to say and is out by twenty binary orders of
/// magnitude: `2^-50` of a step of one is `9e-16`, and the smallest normal number
/// is `2e-308`. The claim audit caught it. Fifty is right for the reason above
/// and the arithmetic in the old reason was wrong.
const SHORTENINGS_BEFORE_GIVING_UP: usize = 50;

/// How many multiples of the deflection budget one step may grow to.
///
/// A step doubles whenever the last one's sagitta was comfortably inside the
/// budget, so a large gentle loop is not walked at the step a tight corner
/// needed. A million, and what it stops is a loop so gentle that the step runs
/// past the whole curve and the walk closes on its second point without having
/// gone anywhere.
const MOST_A_STEP_MAY_GROW_TO: f64 = 1e6;

/// How many enclosure widths a settled seed may sit off either of its surfaces.
///
/// A million. Over the 991 marches the test suite runs, every walk that closed
/// its loop had a seed within **12.8** widths, and the four that crawled to the
/// point cap had one at **1.7e15**. A million sits eighty thousand times above
/// what a working walk has ever needed and a billion times below the failure,
/// which is as far from both as the measurement allows.
///
/// **An enclosure of no width is let through, and that is not a hole.** No width
/// says every operation along the way was exact, so a residual beside it would
/// be a real distance and this question would have no scale to ask it against.
/// Measured: of 3 333 seed checks the suite makes, 580 come back with no width,
/// and every one of those has a residual of exactly zero with it. A settling
/// that lands exactly is what an exact enclosure is reporting.
const WIDTHS_A_SETTLED_SEED_MAY_SIT_OFF: f64 = 1e6;

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
/// shortened [`SHORTENINGS_BEFORE_GIVING_UP`] times before the run is refused,
/// so a curve of finite length is walked in finitely many steps. **How far a step
/// moves is measured and not decreed**: the trial point is settled onto both
/// surfaces afterwards, so what the step actually covers is whatever the
/// settling left, and all that is checked is that it is not nothing — nor, on a
/// trial that came back short, so little that its two ends' own bounds could
/// account for all of it ([`what_a_trial_of_this_length_measures`]). That makes
/// the argument one about the curve rather than about the arithmetic, so it is
/// not left to stand on its own: `most` caps the count outright, and reaching
/// that cap is a refusal rather than a truncated answer.
///
/// **The hole in that argument is steps that shrink**, because their lengths can
/// sum to something finite and the walk then converges on a point instead of
/// traversing a curve. That is what a crossing smaller than the numbers where it
/// sits does to this loop, and [`the_settling_did_not_land_on_both`] turns the
/// seed away before the first step rather than leaving `most` to notice a
/// million points later (`docs/decisions.md` §94).
///
/// # Errors
///
/// [`Error::ACrossingTooSmallForAnyPointToSitOnBothItsSurfaces`] when the seed,
/// settled, is still far enough off one of the surfaces that nothing near it can
/// be on both.
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
    if the_settling_did_not_land_on_both(surfaces, start) {
        return Err(Error::ACrossingTooSmallForAnyPointToSitOnBothItsSurfaces);
    }
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

/// Whether the settled seed is still so far off one of the two surfaces that no
/// point near it can be on both.
///
/// The settling moves a point onto both surfaces as far as `f64` lets it, and
/// how far that is is [`how_far_off_enclosing`]'s own width: an enclosure of the
/// distance is as narrow as the arithmetic at that point can make it, so a
/// residual of a few of those widths is what a settled point looks like. A
/// residual of a great many of them is a point the settling could not move,
/// which happens when the crossing is so much smaller than the place it sits
/// that no `f64` point lands within reach of both surfaces at once
/// (`docs/decisions.md` §94).
///
/// **Asked of the seed and of nothing else.** Measured over the whole suite,
/// the first *marched* point of a walk that goes on to close its loop sits up to
/// `4.8e10` widths off, so the same question asked of every point would refuse
/// walks that work. The seed is what the rest is built on, and it is the one
/// point whose residual the marcher has spent eight Newton rounds on before
/// anything else has happened.
fn the_settling_did_not_land_on_both(surfaces: [&Surface; 2], at: Point) -> bool {
    surfaces.iter().any(|surface| {
        let (off, _) = how_far_off_enclosing(surface, at);
        let width = off.hi() - off.lo();
        width > 0.0 && furthest_from_zero(off).hi() > width * WIDTHS_A_SETTLED_SEED_MAY_SIT_OFF
    })
}

/// `from` settled onto both surfaces, by Newton in the plane the two normals
/// span.
///
/// Each round moves the point by the two signed distances, corrected for the two
/// normals not being square to each other. With `c` the cosine of the angle
/// between them, the exact solution of the linearised pair moves `(s₁ − s₂c)/(1 − c²)`
/// along the first normal and `(s₂ − s₁c)/(1 − c²)` along the second, so on a
/// quadric it converges as Newton does.
///
/// # The two subtractions that are not written down the way they are named
///
/// `1 − c·c` and `s₁ − s₂c` both cancel to nothing as the two surfaces come
/// parallel, and the tightest crossing a ball on a pipe can express — one
/// rounding of radius apart, leaning `2.1e-8` — already lands past the end of
/// both. There `c` **rounds to exactly one**, so `1 − c·c` is exactly zero, and
/// that zero was the whole of the marcher's reachable near-tangency refusal
/// (`docs/decisions.md` §110.3: all 44 corpus files stopped there). One step
/// less parallel it is `6.66e-16` where the truth is `4.44e-16`, a fifty per
/// cent error in the size of every move. `s₁ − s₂c` collapses beside it: with
/// `c` rounded to one and the two distances equal it is exactly zero too, so a
/// denominator mended on its own would leave the point sitting still.
///
/// Both have a spelling that says the same thing without the cancellation, and
/// neither changes what is computed anywhere else:
///
/// ```text
/// 1 − c² = (1 − c)(1 + c)     1 − c = ‖u − v‖²/2     1 + c = ‖u + v‖²/2
/// s₁ − s₂c = (s₁ − s₂) + s₂(1 − c)
/// ```
///
/// The two halves of the denominator are **sums of squares**, so neither can
/// cancel at all, and their product carries the near-tangency in `1 − c` alone
/// where it is a relative error and not an absolute one. The numerator's
/// subtraction becomes `s₁ − s₂`, which is the distance between the two
/// surfaces at the point and is what the answer is actually made of.
///
/// **This is deliberately not the sum-and-difference form**, which spells the
/// pair as `(s₁ + s₂)/(1 + c)` and `(s₁ − s₂)/(1 − c)` and has no cancellation
/// in it anywhere. That form is worse: where the normals are square to each
/// other and one distance is far larger than the other, it reaches `u·s₁ + v·s₂`
/// by cancelling two terms of size `s₁` against each other, and the `s₂` part is
/// lost. Measured, it pushed a coarse walk's published bound from inside the
/// pipe the crossing is drawn on to `1.26e6` against a pipe `1.05e6` across.
///
/// `docs/decisions.md` §112 has both measurements and what the change did to the
/// refusal rate.
///
/// # Errors
///
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] when the two
/// normals are bit-for-bit the same vector, or bit-for-bit opposite, so `1 − c`
/// or `1 + c` is exactly zero and there is no plane to correct in. That is the
/// same wall [`which_way_the_crossing_runs`] has, and
/// `the_angle_at_which_a_crossing_stops_having_a_direction` shows it to be a
/// limit rather than a gap. A normal that is not a number arrives here as the
/// same refusal, which is the mis-report that function's doc comment records.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a round lands somewhere that
/// is not a point.
fn settled(surfaces: [&Surface; 2], from: Point) -> Result<Point> {
    let mut here = from;
    for _ in 0..ROUNDS_OF_SETTLING {
        let [(first, one_way), (second, other_way)] =
            surfaces.map(|surface| how_far_off_and_which_way(surface, here));
        let leaning_apart = (one_way - other_way).length_squared() * 0.5;
        let leaning_together = (one_way + other_way).length_squared() * 0.5;
        let across = leaning_apart * leaning_together;
        if across <= 0.0 || !across.is_finite() {
            return Err(gave_up(
                surfaces,
                WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces,
            ));
        }
        let between_them = first - second;
        let along_one = (between_them + second * leaning_apart) / across;
        let along_other = (-between_them + first * leaning_apart) / across;
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
    let [first, second] = surfaces.map(|surface| {
        let (off, way) = how_far_off_enclosing(surface, at);
        (furthest_from_zero(off), way)
    });
    let sideways = cross_enclosing(first.1, second.1);
    let leaning = root_enclosing(squared_length_enclosing(sideways));
    if leaning.lo() <= 0.0 {
        return Err(gave_up(
            surfaces,
            WhereFollowingACrossingGaveUp::BoundingHowFarOffTheCrossingAPointIs,
        ));
    }
    let reach = quotient_or_everything(first.0 + second.0, leaning);
    if !reach.hi().is_finite() {
        return Err(gave_up(
            surfaces,
            WhereFollowingACrossingGaveUp::TheBoundOnHowFarOffIsBiggerThanAnyNumber,
        ));
    }
    Ok(reach.hi())
}

/// Which way the crossing runs at `at`: the two normals crossed together, scaled
/// to length one.
///
/// # At what lean this stops working, and why nothing rescues it
///
/// Measured, not reasoned. Two unit normals leaning `θ` apart cross to a vector
/// of length `sin θ`, and each of its three components is a difference of two
/// products of numbers no bigger than one — so each carries about a rounding of
/// error **whatever the answer is**. Once `sin θ` falls to a rounding the
/// components are that error and nothing else, and they cancel to exactly zero
/// often enough to refuse: `crates/rcad-geom/tests/the_angle_at_which_a_crossing_stops_having_a_direction.rs`
/// measures every orientation still answering down to a lean of `1.11e-16`, one
/// rounding, and most of them refusing a sixteenth of a rounding below it.
///
/// **So this is cancellation and not underflow.** Underflow would want `sin θ`
/// near `1e-308`, nineteen powers of ten further down, and it never gets a turn.
///
/// The family of fixes, and what each is worth:
///
/// - **Scaling either or both normals.** A power of two is exact, so the cross
///   product comes back with the same significands and a moved exponent: a
///   direction lost is lost by the same bits. `scaling_both_normals_by_a_power_of_two_leaves_the_same_bits_behind`
///   checks that bit-for-bit at `2^±500` and `2^±60`. Anything that is not a
///   power of two only adds a rounding of its own, and reshuffles which
///   orientations happen to cancel to zero — turning a refusal into a wrong
///   answer rather than a right one.
/// - **A different order of the same subtractions**, which does not change that
///   each component is a difference of two products of the same size.
/// - **Taking the cancellation out**: a correctly rounded `a·b − c·d` by fused
///   multiply-add, or a compensated sum, or an exactly rounded determinant.
///   These do work — they hand back the direction the two `f64` normals really
///   span at any lean at all — and it buys nothing, because by then the error is
///   in the normals rather than in the cross product.
///   `taking_the_cancellation_out_of_the_cross_product_buys_nothing_against_the_truth`
///   measures both against the crossing's true direction and they stay within a
///   factor of one and a half of each other all the way down.
///
/// That family is everything that reads only the two unit normals, and
/// `one_rounding_of_either_normal_swings_the_direction_a_quarter_turn` is why it
/// is everything: at a lean of one rounding, flipping the last bit of either
/// normal swings the exactly computed cross product by more than half a radian.
/// The direction is not in the inputs, so no function of the inputs can return
/// it. A **limit**, not a gap (§16.10).
///
/// # And the refusal here is not the one a near-tangent crossing meets
///
/// Nothing that gets into [`march`] presents a lean anywhere near a rounding.
/// [`walk`](fn@crate::walk) hands it exactly two pairs — a ball reaching across
/// a pipe and one pipe reaching across another — and for the first of those
/// `the_smallest_lean_a_ball_on_a_pipe_can_be_given_is_the_root_of_a_rounding`
/// measures the smallest non-zero lean the pair can be *given* at `2.1e-8`,
/// eight powers of ten above where this arithmetic fails, scale-invariantly so:
/// the lean is a **square root** of the gap between the two radii, so one
/// rounding of gap is the root of a rounding of lean. The step below it is not a
/// smaller lean but no lean at all. **The other pair is not measured here**, so
/// that floor is claimed for a ball on a pipe and not for surfaces in general.
///
/// Below that, and only there,
/// `the_enclosing_bound_reaches_zero_before_the_plain_cross_of_normals_does`
/// races this against [`how_far_off_the_crossing`], which [`march`] asks first:
/// over 8 000 leans between `5.7e-14` and `1.7e-18` the enclosing sine had
/// already reached zero at every one of the 1 782 where this lost its direction.
///
/// **At the leans a ball on a pipe really does reach it is neither of them.**
/// `the_settling_and_not_the_stepping_gives_up_on_a_ball_a_rounding_bigger_than_its_pipe`
/// takes that pair apart one call at a time: [`how_far_off_the_crossing`]
/// answers with a bound of `1.05e-8`, this function answers, and [`one_step`]
/// is what refuses — from [`settled`], on its first trial, because `1 − c·c` is
/// exactly zero where the step lands. So the near-tangency wall the marcher
/// actually has is the **settling's**, eight powers of ten above this one, and
/// whether *that* refusal is a limit or a gap is not settled here.
///
/// # Errors
///
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] when the two
/// normals are parallel, so the crossing has no direction there.
///
/// **This also swallows a normal that is not a number**, which is a different
/// thing and is reported as the wrong one:
/// `a_normal_that_is_not_a_number_is_reported_as_two_surfaces_being_tangent`
/// pins that. Whether [`march`] can settle a point onto the one place that
/// happens — a ball's own centre — is not demonstrated either way here, so the
/// mis-report is recorded rather than fixed.
fn which_way_the_crossing_runs(surfaces: [&Surface; 2], at: Point) -> Result<Vector> {
    let [(_, one_way), (_, other_way)] =
        surfaces.map(|surface| how_far_off_and_which_way(surface, at));
    one_way.cross(other_way).normalized().map_err(|_| {
        gave_up(
            surfaces,
            WhereFollowingACrossingGaveUp::CrossingTheTwoNormalsForADirection,
        )
    })
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
/// **Unless the trial settled back onto `here`**: a chord of exactly nothing,
/// or a short one no longer than its two ends' own bounds put together
/// ([`what_a_trial_of_this_length_measures`]). Such a trial has measured nothing
/// about the curve, as a trial that could not be settled has not, so it is
/// retried at the square root of the ask the way that one is, below, and never
/// kept. Halving it was not enough. A pipe of radius 1 through the axis of one
/// of radius 4, walked from `(4, 0, 6)` with `f64::MAX / 4` of room, settles
/// every trial from `6.7e153` down back onto that seed exactly, so fifty
/// halvings ran out at `2.4e139` and the walk was refused as the two pipes being
/// tangent, which they are nowhere; rooted, the seventh trial, `6.4e4`, lands on
/// the loop (`a_trial_that_settles_back_onto_its_start_is_shortened_by_its_root`,
/// `docs/decisions.md` §421). Keeping one that settled a rounding off was worse:
/// the steps doubled from it, so the walk's first points sat closer together
/// than it knew any of them to be
/// (`a_trial_that_settles_a_rounding_off_its_start_is_shortened_and_not_kept`,
/// §347.9). Below an ask of four the root is the longer of the two, so there a
/// trial that settled back is halved, and that is the only place a fine walk
/// meets a chord of nothing: measured over `rcad-geom`'s 1 147 tests, 1 391
/// trials came back with a chord of nothing, and the 263 of them asked four or
/// more were all in coarse walks, with `2.2e307` and `4.5e307` of room. A short
/// chord within its ends' bounds is met in fine walks too: 1 311 trials, of
/// which 460 the walk used to keep in walks of `0.5` of room or less, each end
/// known only to between `8e-9` and `3e-2`.
///
/// **The two cases are kept apart rather than merged into `min`** because the
/// merged form also shortens steps in the ordinary fine walk, where the trial
/// came back a little short for honest reasons, and that moved the marcher's
/// published bound from `0.577` of its own enclosure to `0.871` — still a bound,
/// but past the three quarters `how_far_the_marcher_strays_from_a_crossing_that_is_a_circle`
/// publishes. Splitting on whether the trial overshot leaves the fine walk at
/// `0.577` exactly (`docs/decisions.md` §89).
///
/// # A trial that cannot be measured at all is a trial, not an answer
///
/// The questions a trial asks — settle the far end, settle the middle, read the
/// direction at the far end, and on a trial that came back short, bound how far
/// the far end may sit off the crossing — can each refuse, and a refusal there
/// is about the **trial point**, not about where the step started from. [`march`]
/// takes `tangent` from [`which_way_the_crossing_runs`] at `here`, so the two
/// surfaces are known to cross transversally at `here` before this is ever
/// called; shortening walks the trial back towards that point, which is the same
/// answer this function already gives every other kind of bad trial.
///
/// **This used to propagate the first such refusal straight out**, which is why
/// a coarse walk asking `f64::MAX / 4` of room could stop on a pair it crosses
/// perfectly well: one step that long lands the settling somewhere it diverges
/// from, and a point that has run away to `1e307` and come back onto a pipe's own
/// axis has no normal to settle against. Nothing about that is the two surfaces
/// being tangent.
///
/// Nothing is swallowed: the last refusal is kept, and it is what comes back if
/// all [`SHORTENINGS_BEFORE_GIVING_UP`] of them refuse, so a crossing that really
/// is tangent still says so and still names its site.
///
/// **A refused trial is retried at the square root of the ask, not at half of
/// it.** Halving searches the exponent one bit at a time, and the coarse walk
/// starts at `f64::MAX / 4`: measured, fifty halvings of `2.25e307` reach
/// `9.3e43` and every one of them refuses, because the whole range is still
/// absurd. A trial that could not be settled has said nothing about the curve —
/// not even how long it is, which is what the chord rule below reads off a short
/// trial — so the one thing known is that the ask was too big by an unknown
/// number of orders of magnitude, and the search that finds that is over the
/// exponent rather than the value. Eleven roots take `2.25e307` to about one.
/// The `min` is what keeps it a shortening: below an ask of one, a root would
/// grow the step, so it halves there instead.
///
/// # Errors
///
/// [`Error::CannotFollowACrossingBecauseNoStepStaysOnIt`] when shortening the
/// step [`SHORTENINGS_BEFORE_GIVING_UP`] times still leaves a piece that strays
/// too far. This is reachable and carries its own name —
/// `a_step_asked_the_wrong_way_round_a_circle_is_refused_as_stepping_and_not_tangency`
/// is the demonstration — but no walk reaches it, because a walk takes its
/// direction from the point it is standing on.
///
/// Whatever a trial refused with, when every trial refused. Today that is
/// [`Error::CannotFollowACrossingWhereItsSurfacesAreTangent`] or
/// [`Error::CannotWorkOutHowBigACrossingIs`], from [`settled`],
/// [`which_way_the_crossing_runs`] or [`how_far_off_the_crossing`].
fn one_step(
    surfaces: [&Surface; 2],
    here: Point,
    tangent: Vector,
    from: f64,
    room: f64,
) -> Result<(Point, f64)> {
    let mut step = from;
    let mut why_the_last_trial_could_not_be_measured = None;
    for _ in 0..SHORTENINGS_BEFORE_GIVING_UP {
        let (next, chord, sagitta, turned, settled_back) =
            match what_a_trial_of_this_length_measures(surfaces, here, tangent, step) {
                Ok(measured) => measured,
                Err(why) => {
                    why_the_last_trial_could_not_be_measured = Some(why);
                    step = shortened_by_its_exponent(step);
                    continue;
                }
            };
        if sagitta <= room * SAGITTA_MAY_TAKE
            && sagitta <= chord * SAGITTA_MAY_TAKE_OF_ITS_CHORD
            && turned >= 0.0
            && !settled_back
        {
            return Ok((next, chord));
        }

        // A trial that covered less than half of what it asked for never stepped
        // along the curve: the settling pulled it back, and it will land in the
        // same place however far the ask is halved, so the halvings run out with
        // the chord where it started. There the chord is the one thing the trial
        // did measure about the curve, so the next ask comes from that — unless
        // it settled back onto `here`, which says no more than a trial that could
        // not be settled. A trial that did step along the curve and only bent too
        // far is halved as before, which is what the published bound was measured
        // under.
        step = if settled_back {
            shortened_by_its_exponent(step)
        } else if chord * 2.0 < step {
            chord * 0.5
        } else {
            step * 0.5
        };
    }
    Err(why_the_last_trial_could_not_be_measured.unwrap_or_else(|| {
        Error::CannotFollowACrossingBecauseNoStepStaysOnIt {
            surfaces: Box::new([*surfaces[0], *surfaces[1]]),
            shortenings: SHORTENINGS_BEFORE_GIVING_UP,
        }
    }))
}

/// Where a step of `step` lands, how long the straight piece it leaves behind
/// is, how far that piece bows away from the crossing at its middle, how much
/// of the direction it was asked to go in it is still going in, and whether it
/// settled back onto `here`.
///
/// **Settled back is a chord of nothing, or a trial that came back short to
/// no further from `here` than the two ends' own bounds reach.**
/// [`how_far_off_the_crossing`] encloses how far each end may sit off the
/// crossing, so a chord no longer than the two together can be the two ends
/// sitting off the crossing on different sides and nothing else: the trial
/// has not been shown to move along the curve at all. Measured, the one that
/// made this rule settled one rounding off in `x` and in `z`, a chord of
/// `2.3e-16` with nothing along the loop, between two ends each known to
/// `6.9e-16`
/// (`a_trial_that_settles_a_rounding_off_its_start_is_shortened_and_not_kept`).
///
/// That bound is the arithmetic's own: how far the settling's roundings may
/// have left each end from the crossing. It is not a tolerance on the answer,
/// because nothing it reads is kept or moved. A trial it catches is shortened
/// and tried again, and no point the walk hands back is any nearer or further
/// from the crossing for it. It is asked only of a trial that covered less than
/// half of its ask, so a walk whose steps are honestly that small, asked and
/// covered, is walked as it was.
///
/// # Errors
///
/// Whatever [`settled`] or [`which_way_the_crossing_runs`] refuse with at the
/// trial point, and [`how_far_off_the_crossing`]'s refusal there on a trial
/// that came back short. [`one_step`] reads that as a trial to shorten rather
/// than as an answer.
fn what_a_trial_of_this_length_measures(
    surfaces: [&Surface; 2],
    here: Point,
    tangent: Vector,
    step: f64,
) -> Result<(Point, f64, f64, f64, bool)> {
    let next = settled(surfaces, here + tangent * step)?;
    let straight_on = next - here;
    let chord = straight_on.length();
    let middle = here + straight_on * 0.5;
    let sagitta = (settled(surfaces, middle)? - middle).length();
    let turned = which_way_the_crossing_runs(surfaces, next)?.dot(tangent);
    let settled_back = chord == 0.0
        || (chord * 2.0 < step
            && chord
                <= how_far_off_the_crossing(surfaces, here)?
                    + how_far_off_the_crossing(surfaces, next)?);
    Ok((next, chord, sagitta, turned, settled_back))
}

/// The next ask after a trial that measured nothing about how long a step
/// should be: its square root, or half of it below four, where half is the
/// shorter of the two.
fn shortened_by_its_exponent(step: f64) -> f64 {
    step.sqrt().min(step * 0.5)
}

/// The refusal the four questions that read the two normals raise, carrying the
/// pair it refused about.
///
/// One function rather than four spellings of the same `Box::new`, because the
/// pair is the half a caller cannot get back any other way and it would be easy
/// to leave one site handing back a surface it happened to have rather than
/// both.
fn gave_up(surfaces: [&Surface; 2], at: WhereFollowingACrossingGaveUp) -> Error {
    Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
        surfaces: Box::new([*surfaces[0], *surfaces[1]]),
        at,
    }
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
            (reach - pipe.radius(), sideways * reach.recip())
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

/// An enclosure of how far `at` sits off `surface`, **signed**, and an enclosure
/// of the unit normal there.
///
/// Signed, and the width is kept, because two callers want two different things
/// out of it. [`how_far_off_the_crossing`] wants the magnitude and takes it with
/// [`furthest_from_zero`]. [`the_settling_did_not_land_on_both`] wants the width,
/// which is how close to the surface the arithmetic at this point can place
/// anything at all.
fn how_far_off_enclosing(surface: &Surface, at: Point) -> (Interval, [Interval; 3]) {
    match surface {
        Surface::Plane(flat) => how_far_off_a_plane_enclosing(flat, at),
        Surface::Sphere(ball) => how_far_off_a_ball_enclosing(ball, at),
        Surface::Cylinder(pipe) => how_far_off_a_pipe_enclosing(pipe, at),
        Surface::Cone(cone) => how_far_off_a_cone_enclosing(cone, at),
        Surface::Torus(ring) => how_far_off_a_torus_enclosing(ring, at),
    }
}

/// `(n·p − d) / √(n·n)`, and `n / √(n·n)`.
fn how_far_off_a_plane_enclosing(flat: &Plane, at: Point) -> (Interval, [Interval; 3]) {
    let normal = exactly(flat.normal().components());
    let length = root_enclosing(squared_length_enclosing(normal));
    let off = quotient_or_everything(
        dot_enclosing(normal, exactly(at.coordinates())) - Interval::exactly(flat.offset()),
        length,
    );
    (off, normal.map(|part| quotient_or_everything(part, length)))
}

/// `‖p − c‖ − R`, and `(p − c) / ‖p − c‖`.
fn how_far_off_a_ball_enclosing(ball: &Sphere, at: Point) -> (Interval, [Interval; 3]) {
    let out = gap_enclosing(ball.centre().coordinates(), at.coordinates());
    let reach = root_enclosing(squared_length_enclosing(out));
    (
        reach - Interval::exactly(ball.radius()),
        out.map(|part| quotient_or_everything(part, reach)),
    )
}

/// `‖(p − o) × w‖ / √(w·w) − r`, and the unit direction out from the axis.
///
/// The radius is subtracted from a plain distance and not from one scaled by
/// `√(w·w)`: a pipe's surface is every point exactly `radius` from the axis,
/// whatever length the stored direction rounded to (§118.11).
fn how_far_off_a_pipe_enclosing(pipe: &Cylinder, at: Point) -> (Interval, [Interval; 3]) {
    let along = exactly(pipe.axis().direction().components());
    let squared_direction = squared_length_enclosing(along);
    let out = gap_enclosing(pipe.axis().origin().coordinates(), at.coordinates());
    let sideways: [Interval; 3] = {
        let up = quotient_or_everything(dot_enclosing(out, along), squared_direction);
        core::array::from_fn(|axis| out[axis] - along[axis] * up)
    };
    let reach = root_enclosing(squared_length_enclosing(sideways));
    let off = reach - Interval::exactly(pipe.radius());
    (
        off,
        sideways.map(|part| quotient_or_everything(part, reach)),
    )
}

/// `(‖(p − apex)_perp‖ − slope·|(p − apex)_along|) / √(1 + slope²)`, and the
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
    (off, outwards)
}

/// `‖(‖(p − c)_perp‖ − R, (p − c)·ŵ)‖ − r`, and the unit normal there.
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
        off_the_circle - Interval::exactly(ring.tube_radius()),
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
    use core::cmp::Ordering;
    use rcad_predicates::{Decision, Point as PredicatePoint, SideOfSurface};

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

    /// **A trial that settles back onto the point it set out from is shortened
    /// by its exponent, as one that could not be settled is.** A pipe of radius
    /// 4 about `z` and one of radius 1 along `x` at `z = 5` cross through the
    /// fat one's axis in two loops, and the coarse walk's room is `f64::MAX / 4`.
    /// From `(4, 0, 6)` along `−y` the first trial, `4.5e307`, cannot be
    /// settled, and every trial from its root `6.7e153` down settles back onto
    /// `(4, 0, 6)` exactly, a chord of `0`. Halved, the fifty shortenings ran
    /// out at `2.4e139` and the walk was refused as the two pipes being tangent
    /// while settling, which they are nowhere. Rooted, the trials from `6.7e153`
    /// to `1.7e19` settle onto the seed, `4.1e9` cannot be settled, `6.4e4`
    /// lands on the loop `1.8` away, and half that chord is taken.
    #[test]
    fn a_trial_that_settles_back_onto_its_start_is_shortened_by_its_root() {
        let fat = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 4.0);
        let thin = a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 5.0], 1.0);

        let walked = march(
            [&fat, &thin],
            Point::new(4.0, 0.0, 6.0),
            f64::MAX / 4.0,
            1 << 20,
        )
        .expect("a thin pipe through a fat one's axis crosses it in two loops");

        println!("{walked:?}");
        for point in &walked.points {
            assert!(
                (f64::hypot(point.x(), point.y()) - 4.0).abs() <= walked.deviation
                    && (f64::hypot(point.y(), point.z() - 5.0) - 1.0).abs() <= walked.deviation,
                "a point on both pipes, not {point:?} give or take {}",
                walked.deviation
            );
        }
        assert!(
            walked.points.iter().any(|point| point.z() < 5.0)
                && walked.points.iter().any(|point| point.y() < 0.0)
                && walked.points.iter().any(|point| point.y() > 0.0),
            "the walk goes right round the thin pipe from above it: {:?}",
            walked.points
        );
    }

    /// **A trial that settles back to within a rounding of its start is
    /// shortened, as one that settles back onto it exactly is.** A ball of
    /// radius 2 at the origin and a pipe of radius 1 up `z` at `x = 1/2` cross
    /// in two loops, and the upper one's top is `(−1/2, 0, √15/2)`. From there
    /// with the coarse walk's room of `f64::MAX / 4`, the trial `6.7e153`
    /// settles back onto the top exactly and is rooted, and `8.2e76` settles
    /// one rounding off in `x` and in `z` and not at all along the loop: a
    /// chord of `2.3e-16`, between two ends each known to `6.9e-16` of the
    /// crossing. That was kept as the walk's first step, and the steps after it
    /// doubled from there, so the first four points sat closer to the one
    /// before than the walk knew any of its points to be. The mesh of a ball
    /// with a bar through it off its axis refused such a loop as two corners
    /// in one place (`docs/decisions.md` §347.9, Q1598).
    ///
    /// Now `8.2e76` and then `2.9e38` settle that same rounding off, both are
    /// rooted, and the first step kept lands `0.78` along the loop. The walk
    /// closes in six points, every step `0.78` to `1.12` long, each point known
    /// to `1.3e-15`.
    #[test]
    fn a_trial_that_settles_a_rounding_off_its_start_is_shortened_and_not_kept() {
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.5, 0.0, -3.0], 1.0);
        let ball = a_ball([0.0, 0.0, 0.0], 2.0);
        let top = Point::new(-0.5, 0.0, 3.75_f64.sqrt());

        let walked = march([&pipe, &ball], top, f64::MAX / 4.0, 1 << 20)
            .expect("a pipe through a ball off its axis crosses it in two loops");

        let gaps: Vec<f64> = core::iter::once(top)
            .chain(walked.points.iter().copied())
            .collect::<Vec<_>>()
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).length())
            .collect();
        println!("each point known to {:e}, gaps {gaps:?}", walked.deviation);
        assert!(
            gaps.iter().all(|&gap| gap > 2.0 * walked.deviation),
            "every step goes further than the two ends' own bounds account for: \
             gaps {gaps:?}, each point known to {:e}",
            walked.deviation
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
            Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
                surfaces: Box::new([ball, pipe]),
                at: WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces,
            }),
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
            Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
                surfaces: Box::new([ball, pipe]),
                at: WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces,
            })
        );
    }

    /// How far apart the two normals lean at `at`, as a sine: the length of the
    /// cross product [`which_way_the_crossing_runs`] then tries to scale.
    fn how_far_the_two_surfaces_lean_apart(surfaces: [&Surface; 2], at: Point) -> f64 {
        let [(_, one_way), (_, other_way)] =
            surfaces.map(|surface| how_far_off_and_which_way(surface, at));
        one_way.cross(other_way).length()
    }

    /// **Why the cross of two normals cannot be what gives up on a ball on a
    /// pipe.** `the_angle_at_which_a_crossing_stops_having_a_direction` says the
    /// cross of two unit normals stops carrying a direction once they lean less
    /// than about `1.1e-16` apart. This says the smallest lean this pair can be
    /// *given* is `2.1e-8` — eight powers of ten above it — and that the step
    /// below that is not a smaller lean but no lean at all.
    ///
    /// Which is not to say the marcher walks that crossing: it refuses it, with
    /// this very error, from somewhere else. See
    /// `the_stepping_gives_up_on_a_ball_a_rounding_bigger_than_its_pipe_and_not_the_normals`.
    ///
    /// A ball of radius `R` centred on a pipe of radius `r` crosses it where the
    /// two normals lean `√(R² − r²)/R` apart. That is a **square root** of the
    /// gap between the radii, so one rounding of gap — the least that two
    /// different `f64` radii can differ by — comes out as the root of a
    /// rounding, `√(2·2⁻⁵³) = 2.1e-8`. There is nothing between that and zero to
    /// write down.
    ///
    /// Scale-invariantly so: the radii and the lean scale together, and the root
    /// of a relative rounding does not know what scale it was taken at.
    #[test]
    fn the_smallest_lean_a_ball_on_a_pipe_can_be_given_is_the_root_of_a_rounding() {
        for power in [-40i32, -20, 0, 20, 40] {
            let pipe_radius = 2.0_f64.powi(power);
            let ball_radius = f64::from_bits(pipe_radius.to_bits() + 1);
            let ball = a_ball([0.0, 0.0, 0.0], ball_radius);
            let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], pipe_radius);
            let up = f64::sqrt(ball_radius * ball_radius - pipe_radius * pipe_radius);
            let on_the_crossing = Point::new(pipe_radius, 0.0, up);

            let lean = how_far_the_two_surfaces_lean_apart([&ball, &pipe], on_the_crossing);
            assert!(
                (1e-8..1e-7).contains(&lean),
                "one rounding of radius at 2^{power} leans them {lean:e} apart, and the root \
                 of a rounding is 2.1e-8"
            );
            which_way_the_crossing_runs([&ball, &pipe], on_the_crossing).unwrap_or_else(|why| {
                panic!("a lean of {lean:e} is eight powers of ten clear of the noise: {why:?}")
            });
        }

        let same = 3.0;
        let ball = a_ball([0.0, 0.0, 0.0], same);
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], same);
        let touching = Point::new(same, 0.0, 0.0);
        assert_eq!(
            how_far_the_two_surfaces_lean_apart([&ball, &pipe], touching),
            0.0,
            "the step below the root of a rounding is no lean at all"
        );
        assert_eq!(
            which_way_the_crossing_runs([&ball, &pipe], touching),
            Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
                surfaces: Box::new([ball, pipe]),
                at: WhereFollowingACrossingGaveUp::CrossingTheTwoNormalsForADirection,
            }),
            "and there the refusal is right: two surfaces that touch have no crossing to follow"
        );
    }

    /// **Which of the four places that raise this error is the one a real
    /// near-tangent crossing meets.** Asked one call at a time rather than
    /// reasoned about, because the answer is not the one the reasoning gave.
    ///
    /// A ball one rounding of radius bigger than the pipe it is centred on is
    /// the tightest crossing that pair can express —
    /// `the_smallest_lean_a_ball_on_a_pipe_can_be_given_is_the_root_of_a_rounding`
    /// measures its lean at `2.1e-8`. At that lean:
    ///
    /// - [`settled`] leaves the seed exactly where it was, and
    ///   [`the_settling_did_not_land_on_both`] is content;
    /// - [`how_far_off_the_crossing`] answers, with a bound of `1.05e-8`, so its
    ///   enclosed sine did not reach zero;
    /// - [`which_way_the_crossing_runs`] answers;
    /// - [`one_step`] **used to refuse** — from [`settled`], on its very first
    ///   trial, not from running out of shortenings.
    ///
    /// **That last line is the one the site tag changed.** Until the refusal
    /// carried [`WhereFollowingACrossingGaveUp`] this test could only see that
    /// [`one_step`] gave up, and `docs/decisions.md` §109.6 read that as the
    /// shortening budget running out. It is not: the seed settles, the step
    /// lands somewhere the two normals round to parallel, and `1 − c·c` is
    /// exactly zero there. §109.6's own untested hypothesis — that `settled`'s
    /// Newton correction reaches zero at a lean of `2.5e-8` while `|u×v|²` is
    /// still good at `4.9e-11` — is what was happening, and §110 records it.
    ///
    /// **And it was a gap, not a limit** (§16.10). `1 − c·c` is a subtraction of
    /// two numbers that have both rounded to one; `(1 − c)(1 + c)`, with each
    /// half taken as half a squared length, is the same quantity with no
    /// subtraction of near-equal numbers in it, and at this lean it hands back
    /// `4.4e-16` where the subtraction hands back exactly zero. The answer was
    /// representable the whole time. This test measures both spellings at the
    /// very point that used to refuse, and then walks the crossing that used to
    /// be unreachable. `docs/decisions.md` §112.
    ///
    /// The limit is still there, eight powers of ten further down, where the two
    /// normals are bit-for-bit the same vector —
    /// `the_angle_at_which_a_crossing_stops_having_a_direction` pins it at
    /// `1.1e-16` and shows that nothing recovers a direction below it.
    #[test]
    fn a_ball_a_rounding_bigger_than_its_pipe_is_walked_once_the_lean_stops_being_a_subtraction() {
        let a_ball_this_many_roundings_bigger_than_its_pipe = |roundings: u64| {
            let pipe_radius = 1.0_f64;
            let ball_radius = f64::from_bits(pipe_radius.to_bits() + roundings);
            let up = f64::sqrt(ball_radius * ball_radius - pipe_radius * pipe_radius);
            (
                a_ball([0.0, 0.0, 0.0], ball_radius),
                a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], pipe_radius),
                Point::new(pipe_radius, 0.0, up),
            )
        };

        let (ball, pipe, on_the_crossing) = a_ball_this_many_roundings_bigger_than_its_pipe(1);
        let surfaces = [&ball, &pipe];
        let lean = how_far_the_two_surfaces_lean_apart(surfaces, on_the_crossing);
        assert!(
            (2e-8..3e-8).contains(&lean),
            "the tightest crossing a ball on a pipe has leans 2.1e-8, not {lean:e}"
        );

        let start = settled(surfaces, on_the_crossing).expect("the seed is already on both");
        assert_eq!(
            start, on_the_crossing,
            "the seed is exactly on both surfaces, so the settling has nowhere to move it"
        );
        assert!(
            !the_settling_did_not_land_on_both(surfaces, start),
            "a point exactly on both surfaces has landed on both"
        );
        let bound = how_far_off_the_crossing(surfaces, start)
            .expect("the enclosed sine has not reached zero at a lean of 2.1e-8");
        assert!(
            (1e-8..2e-8).contains(&bound),
            "a point that close to two surfaces leaning 2.1e-8 apart is known to \
             1.05e-8, not {bound:e}"
        );
        let tangent = which_way_the_crossing_runs(surfaces, start)
            .expect("and the cross of the two normals answers here too");

        // The first trial `one_step` takes, and the one it used to die on. Both
        // spellings of `1 − c²` are read here rather than argued about.
        let [(_, one_way), (_, other_way)] =
            surfaces.map(|surface| how_far_off_and_which_way(surface, start + tangent * 0.01));
        let leaning = one_way.dot(other_way);
        let by_subtracting = 1.0 - leaning * leaning;
        let by_multiplying = (one_way - other_way).length_squared()
            * 0.5
            * ((one_way + other_way).length_squared() * 0.5);
        println!(
            "at the first trial the two normals lean c − 1 = {:e}; \
             1 − c·c is {by_subtracting:e} and (1 − c)(1 + c) is {by_multiplying:e}",
            leaning - 1.0
        );
        assert_eq!(
            by_subtracting, 0.0,
            "the subtraction is the whole of the old refusal: c rounds to exactly one \
             at the first trial, so `1 − c·c` is exactly zero"
        );
        // `1 − c² = sin²θ`, and the lean is measured above, so this is the
        // closed form and not a tolerance.
        let truth = lean * lean;
        assert!(
            (by_multiplying / truth - 1.0).abs() < 1e-2,
            "and the product of the two halves is the answer that was there all along: \
             {by_multiplying:e} against a true sin² of {truth:e}"
        );

        let chord = one_step(surfaces, start, tangent, 0.01, 0.01)
            .expect("so one_step takes its first trial rather than giving up on it")
            .1;
        assert!(
            chord > 0.0,
            "and the step it took went somewhere: {chord:e}"
        );

        let walked = march(surfaces, on_the_crossing, 0.01, 1 << 16)
            .expect("and the whole march goes through at the tightest lean this pair has");
        assert!(
            walked.points.len() > 20,
            "drawing the circle rather than stopping on it: {} points",
            walked.points.len()
        );
        // The crossing of a ball threaded on its own pipe's axis is two circles
        // of the pipe's radius at ±√(R² − r²), and the seed is a point of one of
        // them. That is a closed form, so every marched point is checked against
        // it rather than against the marcher.
        let [across, _, up] = on_the_crossing.coordinates();
        let furthest_off = walked
            .points
            .iter()
            .map(|point| {
                let [x, y, z] = point.coordinates();
                (x.hypot(y) - across).abs().max((z.abs() - up).abs())
            })
            .fold(0.0_f64, f64::max);
        assert!(
            furthest_off < 1e-7,
            "and the points it drew are on that circle, not merely near the pair: \
             the furthest sat {furthest_off:e} off it"
        );

        let (ball, pipe, on_the_crossing) = a_ball_this_many_roundings_bigger_than_its_pipe(4);
        let surfaces = [&ball, &pipe];
        let lean = how_far_the_two_surfaces_lean_apart(surfaces, on_the_crossing);
        assert!(
            (4e-8..5e-8).contains(&lean),
            "four roundings of radius lean 4.2e-8 apart, not {lean:e}"
        );
        let walked = march(surfaces, on_the_crossing, 0.01, 1 << 16)
            .expect("two roundings of lean further out, the whole march goes through");
        assert!(
            walked.points.len() > 20,
            "and it draws the circle rather than stopping on it: {} points",
            walked.points.len()
        );
    }

    /// The shortening budget is reachable, and what it hands back is not the
    /// tangency refusal.
    ///
    /// Asked of two balls half a radius apart, whose crossing is an honest
    /// circle nothing leans along, and stepped **against** the way that circle
    /// runs. The quarter-turn test compares the direction at the trial point
    /// with the direction it was asked to go in, and those stay half a turn
    /// apart however far the step is shortened — so all fifty tries fail and
    /// none of it is about two surfaces coming parallel.
    ///
    /// **[`march`] never asks this**, because it takes its direction from
    /// [`which_way_the_crossing_runs`] at the point it is standing on. So this
    /// demonstrates the budget is a live path carrying its own name, and
    /// nothing about how a real walk reaches it. Two ways of reaching it
    /// through the sagitta tests instead were tried and neither works: on unit
    /// balls and on balls a million across the midpoint settles onto both
    /// surfaces exactly, the sagitta is `0.0`, and `0.0 <= room * 0.75` holds
    /// however small the room is made.
    #[test]
    fn a_step_asked_the_wrong_way_round_a_circle_is_refused_as_stepping_and_not_tangency() {
        let one = a_ball([0.0, 0.0, 0.0], 1.0);
        let other = a_ball([0.5, 0.0, 0.0], 1.0);
        let surfaces = [&one, &other];
        let start = settled(surfaces, Point::new(0.25, 1.0, 0.0)).expect("a seed near the circle");
        let backwards = -which_way_the_crossing_runs(surfaces, start)
            .expect("two balls half a radius apart cross transversally");
        assert_eq!(
            one_step(surfaces, start, backwards, 0.1, 0.1).map(|(_, chord)| chord),
            Err(Error::CannotFollowACrossingBecauseNoStepStaysOnIt {
                surfaces: Box::new([one, other]),
                shortenings: SHORTENINGS_BEFORE_GIVING_UP,
            }),
            "the budget running out is its own refusal and carries its own pair"
        );
    }

    /// **Of the two arithmetic refusals, which gets there first.** [`march`]
    /// asks [`how_far_off_the_crossing`] before it ever asks
    /// [`which_way_the_crossing_runs`], and that one works the same sine out in
    /// *enclosures* and refuses as soon as the enclosure reaches zero. An
    /// enclosure of a cross product of two unit normals is about a rounding
    /// wide, which is exactly the lean at which the plain cross product goes to
    /// noise — so the plain one is never left holding a question it cannot
    /// answer.
    ///
    /// Raced here rather than argued. The two normals are handed to the
    /// enclosing branch **with no width at all**, which is the most generous
    /// input it can be given — a real one comes out of
    /// [`how_far_off_enclosing`] already widened by its own divisions. So a lean
    /// where the exact-input enclosure has already reached zero is one where the
    /// real one reached it sooner.
    ///
    /// **This is a race over leans from `5.7e-14` down to `1.7e-18`, and
    /// nothing reachable presents one.** It says which of the two dies first
    /// down there and no more than that; at the leans a ball on a pipe can
    /// really be given, neither of them dies at all and the refusal comes from
    /// [`one_step`] —
    /// `the_stepping_gives_up_on_a_ball_a_rounding_bigger_than_its_pipe_and_not_the_normals`.
    #[test]
    fn the_enclosing_bound_reaches_zero_before_the_plain_cross_of_normals_does() {
        let mut spinner = 0x5eed_0005_u64;
        let mut roll = || {
            spinner = spinner
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((spinner >> 11) as f64) / ((1u64 << 53) as f64) * 2.0 - 1.0
        };

        let mut raced = 0_usize;
        let mut the_plain_cross_refused = 0_usize;
        for _ in 0..500 {
            let one_way = Vector::new(roll(), roll(), roll())
                .normalized()
                .expect("a direction");
            let sideways = Vector::new(roll(), roll(), roll());
            let across = (sideways - one_way * sideways.dot(one_way))
                .normalized()
                .expect("a direction square to the first");

            for halvings in 44..60 {
                let lean = 0.5_f64.powi(halvings);
                let other_way = (one_way * lean.cos() + across * lean.sin())
                    .normalized()
                    .expect("leaning a unit vector leaves a unit vector");

                let plain_refused = one_way.cross(other_way).normalized().is_err();
                let enclosed = root_enclosing(squared_length_enclosing(cross_enclosing(
                    exactly(one_way.components()),
                    exactly(other_way.components()),
                )));
                let bound_refused = enclosed.lo() <= 0.0;

                assert!(
                    bound_refused || !plain_refused,
                    "at a lean of {lean:e} the plain cross gave up while the enclosing sine \
                     still had {:e} of room, so the plain refusal is reachable after all",
                    enclosed.lo()
                );
                raced += 1;
                the_plain_cross_refused += usize::from(plain_refused);
            }
        }
        assert!(
            the_plain_cross_refused > raced / 10,
            "the sweep has to reach leans the plain cross really does refuse at, and it \
             refused at only {the_plain_cross_refused} of {raced}"
        );
        println!(
            "{the_plain_cross_refused} of {raced} leans lost the plain cross its direction, \
             and the enclosing sine had already reached zero at every one of them"
        );
    }

    /// **A finding, pinned rather than fixed.**
    /// [`which_way_the_crossing_runs`] maps every way
    /// [`Vector::normalized`](crate::Vector::normalized) can fail onto one
    /// error, and one of those ways is not tangency at all: a normal with a
    /// component that is not a number. At a ball's own centre the outward
    /// direction is `0/0`, and the answer that comes back says the two surfaces
    /// are tangent, which is a statement about a shape rather than about a
    /// point that fell over.
    ///
    /// It is pinned and not fixed because the fix wants a name for the other
    /// case and this lane could not demonstrate that [`march`] reaches it —
    /// [`settled`] would have to leave a point on a ball's centre. §16.10 wants
    /// a demonstration for "no caller hits this" too, so neither claim is made.
    #[test]
    fn a_normal_that_is_not_a_number_is_reported_as_two_surfaces_being_tangent() {
        let ball = a_ball([0.0, 0.0, 0.0], 3.0);
        let other = a_ball([0.0, 0.0, 1.0], 3.0);
        let centre = Point::new(0.0, 0.0, 0.0);

        let (_, no_way_out) = how_far_off_and_which_way(&ball, centre);
        assert!(
            !no_way_out.is_finite(),
            "a ball has no outward direction at its own centre, and got {no_way_out:?}"
        );
        assert_eq!(
            which_way_the_crossing_runs([&ball, &other], centre),
            Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
                surfaces: Box::new([ball, other]),
                at: WhereFollowingACrossingGaveUp::CrossingTheTwoNormalsForADirection,
            }),
            "two balls a third of a radius apart are nowhere near tangent, and this says they are"
        );
    }

    /// **Every way into the marcher refuses at a point whose normal is not a
    /// number, and none of them hands one back.**
    ///
    /// The place is a pipe's own axis, where `sideways * reach.recip()` is
    /// `0 * inf`. It matters because `settled` can *land* there: driven from an
    /// absurd trial step, its Newton iterate ran away to `1e307` and came back
    /// onto `[0, 0, 0]`, which is the thin pipe's axis in
    /// `the_same_two_pipes_with_a_fatter_thin_one_are_walked`. So this is not a
    /// point only a caller could ask about.
    ///
    /// `NaN` quietly propagating is neither of this repo's two answers — it is
    /// not a panic on a broken invariant and not an `Err` on bad input — so what
    /// is checked here is that every entry point turns it into an `Err`. The
    /// guard in [`settled`] is `!across.is_finite()`, and it catches this
    /// because a `NaN` normal makes both squared lengths `NaN`.
    ///
    /// **Each of them names the site it actually refused at**, and the sites are
    /// asserted rather than the mere fact of a refusal, because that is what
    /// would notice a guard moving. Reading the source suggests
    /// [`how_far_off_the_crossing`] should slip past its first guard —
    /// `leaning.lo() <= 0.0` is *false* for a `NaN`, as every comparison against
    /// one is — and be caught further down by `!reach.hi().is_finite()`. It is
    /// not: it refuses at the sine guard. The enclosing arithmetic does not hand
    /// that comparison a `NaN` to be fooled by, so the interval layer is doing
    /// work here that the plain `f64` layer above it could not.
    #[test]
    fn nothing_hands_back_a_point_whose_normal_is_not_a_number() {
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 1.0);
        let ball = a_ball([0.0, 0.0, 0.0], 2.0);
        let surfaces = [&pipe, &ball];
        let on_the_axis = Point::new(0.0, 0.0, 0.0);

        let (how_far, no_way_out) = how_far_off_and_which_way(&pipe, on_the_axis);
        assert!(
            !no_way_out.is_finite(),
            "a pipe has no outward direction on its own axis, and got {no_way_out:?}"
        );
        assert!(
            how_far.is_finite(),
            "though the distance to it is an ordinary number, {how_far:e}, which is \
             why a guard that only watched the distance would miss this"
        );

        let refused_at = |outcome: Result<()>| match outcome {
            Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent { at, .. }) => Some(at),
            _ => None,
        };
        for (what, outcome, expected) in [
            (
                "settled",
                settled(surfaces, on_the_axis).map(|_| ()),
                WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces,
            ),
            (
                "how_far_off_the_crossing",
                how_far_off_the_crossing(surfaces, on_the_axis).map(|_| ()),
                WhereFollowingACrossingGaveUp::BoundingHowFarOffTheCrossingAPointIs,
            ),
            (
                "settled_with_a_bound",
                settled_with_a_bound(surfaces, on_the_axis).map(|_| ()),
                WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces,
            ),
            (
                "which_way_the_crossing_runs",
                which_way_the_crossing_runs(surfaces, on_the_axis).map(|_| ()),
                WhereFollowingACrossingGaveUp::CrossingTheTwoNormalsForADirection,
            ),
            (
                "march",
                march(surfaces, on_the_axis, 0.1, 1 << 10).map(|_| ()),
                WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces,
            ),
        ] {
            assert_eq!(
                refused_at(outcome),
                Some(expected),
                "{what} must refuse at a point whose normal is not a number, and name \
                 the site it refused at"
            );
        }
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

    /// A pipe is the points exactly its own radius from its axis (§118.11), and
    /// both of the marcher's distances have to be measured against that pipe and
    /// not against one `r(1/‖w‖ − 1)` narrower.
    ///
    /// # Why this point can see a difference of `1e-16 · r`
    ///
    /// Every pipe here runs through the coordinate origin along a line lying
    /// flat in the `xy` plane, and the probe is `(0, 0, r)` — the construction
    /// `a_pipe_written_at_another_axis_scale_is_the_same_pipe.rs` is built on.
    /// The `z` axis is square to the `xy` plane, so that point is **exactly** `r`
    /// from every one of those lines, and every number the two distances are
    /// worked out from is exact:
    ///
    /// - `p − o` is `(0, 0, r)` down to the bit, the origin being all zeros;
    /// - `(p − o) · w` is exactly zero, the direction having no `z`, so the
    ///   sideways part is `(0, 0, r)` again with nothing rounded off it;
    /// - `r = 0.5` squares exactly and roots back exactly, so `reach` is `0.5`
    ///   and [`root_enclosing`] hands back an enclosure of no width at all.
    ///
    /// So the plain answer is `0.5 − 0.5` and the enclosure is `[0, 0]`, with no
    /// rounding anywhere for a failure to be blamed on. Divide by `‖w‖` instead
    /// and the answer is the direction's own rounding: the lever is a stored
    /// `‖w‖²` that is not one, which is what the second assertion below pins.
    ///
    /// The two directions are not two spellings of one line —
    /// `Vector::scaled_to_length_one_from_anywhere` is a function of the ratios
    /// between the components, so `(1, 2, 0)` and `(3, 6, 0)` store identically
    /// and a test comparing two spellings would pass vacuously. Each is asked
    /// about the geometry instead, and they straddle: `(1, 2, 0)` stores a `‖w‖²`
    /// one rounding below one and `(1, 1, 0)` one rounding above, so the old form
    /// was wrong in both directions and not just short.
    ///
    /// The pair was `(1, 1, 0)` below and `(1, 6, 0)` above until §120's
    /// correctly rounded normalize moved every stored direction: `(1, 1, 0)`
    /// crossed to the other side of one and `(1, 6, 0)` landed exactly on it.
    #[test]
    fn a_point_a_radius_out_from_a_pipes_axis_is_measured_as_being_on_the_pipe() {
        let radius = 0.5;
        let on_the_skin = Point::new(0.0, 0.0, radius);

        for (written, rounded) in [
            ([1.0, 2.0, 0.0], Ordering::Less),
            ([1.0, 1.0, 0.0], Ordering::Greater),
        ] {
            let surface = a_pipe(written, [0.0, 0.0, 0.0], radius);
            let Surface::Cylinder(pipe) = surface else {
                unreachable!("a_pipe builds a pipe")
            };
            let stored = pipe.axis().direction();
            assert_eq!(
                stored.components()[2],
                0.0,
                "{written:?} was meant to store a direction lying flat in the xy plane",
            );
            assert_eq!(
                stored.length_squared().total_cmp(&1.0),
                rounded,
                "{written:?} stores {stored:?}, whose squared length is {:e} from one — \
                 a row whose squared length is one exactly has no rounding in it to tell \
                 the two pipes apart, and the two rows have to fall either side of one",
                stored.length_squared() - 1.0,
            );
            assert_eq!(
                pipe.side_of(PredicatePoint::from(on_the_skin.coordinates())),
                Decision::Zero,
                "the kernel certifies that {on_the_skin:?} is on the pipe about {stored:?}, \
                 and the marcher has to be measuring against that same pipe",
            );

            let (off, _) = how_far_off_and_which_way(&surface, on_the_skin);
            assert_eq!(
                off, 0.0,
                "a point a radius out from the axis of the pipe about {stored:?} is on it",
            );

            let (enclosed, _) = how_far_off_enclosing(&surface, on_the_skin);
            assert_eq!(
                (enclosed.lo(), enclosed.hi()),
                (0.0, 0.0),
                "everything this enclosure is built from is exact, so what it certifies \
                 about a point on the skin of the pipe about {stored:?} is zero and \
                 nothing either side of it",
            );
        }
    }
}
