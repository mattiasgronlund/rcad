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

use core::cmp::Ordering;

use rcad_predicates::{
    Approximation, Cause, Decision, Filtered, Interval, Level, Measures, Point as PredicatePoint,
    Policy, Predicate, ProvenanceLog, Record, decide, resolve_under_policy,
};
use smallvec::SmallVec;

use crate::enclosure::{
    cross_enclosing, dot_enclosing, exactly, gap_enclosing, squared_length_enclosing,
};
use crate::exact::Exact;
use crate::intersect::{
    Question, WHERE_A_BALL_AND_A_PIPE_CROSS, WHERE_TWO_PIPES_CROSS, ask,
    how_a_ball_on_a_cones_axis_meets_it, one_cone_either_way_round,
    whether_a_ball_runs_through_a_cones_apex, whether_a_ball_sits_on_a_cones_axis,
    whether_a_cone_and_a_pipe_share_an_axis, whether_two_cones_share_an_axis,
};
use crate::march::{Marched, march, settled_with_a_bound};
use crate::meet::nowhere_in_particular;
use crate::{
    Cone, Cylinder, Deflection, EdgeRun, Error, NamedRun, Picker, Plane, Point, Result, Sphere,
    Surface, SurfaceMeet, Torus, Vector,
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
/// **An arc of a plane's ellipse that [`walk`] places round a cone is charged
/// this too**, against the arc's own size and not the ellipse's. Its chain is a
/// different one, derived at `AnArcRoundACone`, and comes to under a tenth of
/// this; one constant for both is what lets `rcad-topo`'s floor under a mesh,
/// a multiple of it, hold whichever way a run was walked.
///
/// `every_point_along_a_crossing_is_on_both_surfaces.rs` measures the worst a
/// search can find and prints the figure.
pub const POINT_ALONG_A_CROSSING_ROUNDOFF: f64 = 256.0 * f64::EPSILON;

/// How far a point [`walk`] hands back can sit off an **endless bend** — the
/// parabola or one branch of the hyperbola a plane cuts a cone in — as a
/// multiple of the largest number involved.
///
/// **A separate constant because it is a separate chain.**
/// [`POINT_ALONG_A_CROSSING_ROUNDOFF`] is derived for a centre, a reach, two
/// normalizations and a `cos`/`sin` pair, and none of the three shapes here has
/// that shape of arithmetic. What they have instead is a **division by `K`**,
/// the number the quadratic's linear term carries, and that division is the one
/// step neither the conic nor the marcher takes. A bound derived for one chain
/// is not a bound for another, and borrowing it would be the paragraph §15.6
/// refuses.
///
/// Written with `u = f64::EPSILON / 2`, `L` the size below, `h` for how far the
/// apex stands off the plane, `t` for the cone's slope, and
///
/// ```text
/// K = 2(1 + t²)·h·(n̂·ŵ)·(and·ŵ),   C = h²(1 − (1 + t²)(n̂·ŵ)²)
/// ```
///
/// so that the parabola is `vertex + across·s + bend·s²` with
/// `vertex = foot − and·(C/K)` and `bend = −and/K`. `L` is the largest of the
/// vertex's own coordinates, the apex's, `|K|`, `|C/K|` and `|h|`, which is what
/// [`how_big_this_is`] is handed — and `|K|` and `|C/K|` are in that list
/// **because this derivation needs them there**, not for tidiness.
///
/// - The **foot** is a dot product, a division and three scaled subtractions:
///   about six roundings on terms under `2L`, so under `12u·L`.
/// - `n̂·ŵ` and `and·ŵ` are each an exactly-summed dot over a square root of a
///   dot: about ten roundings apiece, and both are ratios no bigger than one
///   with nothing to cancel, so each is right to a relative `10u`.
/// - **`K`** is five more products on top of those: relative `25u`. It is a
///   length, `|K| = 2|h|t`, and `L` covers it.
/// - **`C`** is where the cancellation is. `(1 + t²)(n̂·ŵ)²` is `t² + E` and `E`
///   is zero on the parabola, so the bracket `1 − (1 + t²)(n̂·ŵ)²` is a
///   difference that loses its leading digits when `t` is near one. Its
///   *absolute* error stays under `4u·t²`, so `C` is out by under `6u·h²t²` —
///   and that is the term the division turns into a position:
///   `6u·h²t²/|K| = 3u·|h|t`, which is `1.5u·|K|` and so under `2u·L`.
/// - **`C/K`** also carries `K`'s own relative `25u`, on a term `L` covers:
///   under `25u·L`. Adding the vertex's own two roundings per coordinate gives
///   the vertex under `43u·L` all told.
/// - **`bend`** is `−and/K`: relative `27u`, on a vector of length `1/|K|`.
/// - **Evaluating** `vertex + across·s + bend·s²` over `|s| ≤ |K|` puts both
///   moving terms at no more than `|K| ≤ L`, so it is five roundings per
///   coordinate on terms under `3L` — under `15u·L` — plus `across`'s own
///   relative `4u` and `bend`'s `27u` on terms under `L`.
/// - **The parameter** `s` is worked out as `from + (to − from)·step/steps`,
///   three roundings on a term under `|K|`; the curve moves by at most
///   `‖across + 2·bend·s‖ ≤ 3` times that, so under `9u·L`.
///
/// That comes to about `98u·L`, which is `49 · f64::EPSILON · L`. The constant
/// is `64 · f64::EPSILON`, a factor of 1.3 on top — **and no more, because a
/// safety factor nothing measures is a guess** (§15.6). There is no `cos` or
/// `sin` here to buy a libm's slack for; `cosh` and `sinh` on the hyperbola
/// branch are the one place that does, and the two roundings the parabola does
/// not spend on `C`'s cancellation pay for them.
///
/// **This is the parabola's chain, and the worst a search finds is a
/// hyperbola.** `cosh`/`sinh` and the `centre + out·cosh σ + across·sinh σ`
/// evaluation are not counted above; they get the one sentence at the end of the
/// last bullet and no term count of their own. So §15.6's "a published bound
/// needs a derivation" is, as this stands, satisfied for the shape that is *not*
/// the measured worst case. That is a **known hole in the derivation, not a
/// claim that the hyperbola is cheaper** — writing its chain out is work this
/// note does not do.
///
/// # What the test can and cannot fail on, with the number
///
/// `every_point_along_an_endless_bend_is_on_both_surfaces_inside_the_published_bound`
/// is the test §15.6 asks for. Over 158,623 walks and 8,462,349 points it finds
/// a worst of **0.071 of this bound** — about `4.6 · ε · L`, on a random plane
/// against a random cone.
///
/// **So halving this constant would not fail that test**, and the reason to
/// believe that is a table rather than a paragraph. Six families were tried,
/// each on top of the last, and the test prints what each one reached: sweeping
/// the slope and the height alone gets 0.0392; adding the deflection over 21
/// octaves, slopes off the eighths grid, planes barely steeper than a ruling,
/// scales that are not powers of two and finally 120,000 random pairs walks it
/// up to 0.0713. **Five families the old sweep never varied moved the number by
/// 1.8x between them**, so the honest reading is that a search pins the constant
/// to within a factor of about fourteen, not that it is tight. §15.6 says the
/// rest in as many words — *"that test cannot show a bound is tight — rounding
/// errors cancel"* — and what makes `64 · ε` a bound here is the derivation
/// above and not the search.
///
/// **The measurement itself had to be fixed before the search could be
/// widened, and the old 0.049 was an artefact of it.** The harness read the
/// distance to the cone off `√(‖out‖² − up²) − slope·|up|` in plain `f64`, and
/// both of those subtractions cancel. Checked against exact rational arithmetic
/// on two points: on a narrow cone it reported 3.0x the true distance, and at
/// the place the widened search now calls its worst it reported 0.5x — so it was
/// not even reliably pessimistic. Re-running the *identical* old sweep with the
/// residual worked out in twice the precision gives 0.0392 where it used to
/// print 0.0490.
///
/// The half a search *can* pin is the **scaling**, and
/// `what_an_endless_bend_strays_grows_with_the_size_it_was_worked_out_from` is
/// that half: the same bend at four scales 32,768 apart uses the same 0.0078 of
/// its own bound at every one of them, which is what says the error really is
/// proportional to `L` and not an absolute number that happens to hold once. A
/// term dropped out of [`how_big_this_is`]'s list would pass the wide search and
/// fail that one.
pub const POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF: f64 = 64.0 * f64::EPSILON;

/// The most points one crossing is walked into.
///
/// A million. What sets it is not memory but honesty: a deflection close to
/// [`POINT_ALONG_A_CROSSING_ROUNDOFF`] asks for tens of millions of points
/// standing in for a curve they are barely closer to than a thousand would be,
/// and building them quietly is worse than saying so.
pub const MOST_POINTS_ALONG_ONE_CROSSING: usize = 1 << 20;

/// The name a walk a tolerant caller waved through goes by in provenance
/// records.
///
/// Pushed by [`room_left_over`] and nowhere else. A walk whose points already
/// sit further off the crossing than the caller's deflection allows cannot be
/// refined by drawing more of them, and a tolerant caller gets it anyway — which
/// is a non-certified outcome and so gets a record (§5).
pub const ROOM_LEFT_TO_REFINE_A_WALK: &str = "room left to refine a walk";

/// Points along a crossing, and how far off it they may sit.
#[derive(Clone, Debug, PartialEq)]
pub struct PointsAlongACrossing {
    points: Vec<Point>,
    deviation: Approximation,
}

impl PointsAlongACrossing {
    /// How many times further up a cone's axis the ellipse a plane cuts it in
    /// has to reach than a run's two ends do before [`walk`] places that run's
    /// points round the cone and not round the ellipse.
    ///
    /// **Either way is a bound, and this only says which bound is published.**
    /// Round the ellipse, a point rounds against the ellipse's own centre and
    /// reach; round the cone, against the apex and the run's own ends. A plane a
    /// rounding off parallel to a ruling cuts an ellipse `4.5e16` long whose
    /// arcs by the frustum are `4` across, and round the ellipse their middle
    /// points came out `1` to `2` off the cone
    /// (`an_arc_of_a_plane_cut_nearly_along_a_cones_ruling.rs`). An ellipse a
    /// plane cuts at an ordinary lean reaches a few times further than its arcs
    /// and keeps the points it always had: a million is far from both.
    ///
    /// Public because `rcad-topo`'s mesher has to charge such a run by its
    /// ends where this walks it by them, and it does so past **twice** this,
    /// so that the two never disagree about a run on the line between them.
    /// Which of the two arcs the run is, the mesher asks
    /// [`PointsAlongACrossing::are_placed_round_the_cone_from_the_ends_of`].
    pub const WALKED_ROUND_THE_CONE_PAST: f64 = 1_048_576.0;

    /// Whether [`walk`] places `run`'s points round a cone from its two ends,
    /// so that no point of it is further from the apex than one of them.
    ///
    /// **That is the arc that misses the ellipse's far end, and only that
    /// one.** Past [`PointsAlongACrossing::WALKED_ROUND_THE_CONE_PAST`], the
    /// walk asks the run's half-spaces which of the two arcs between its ends
    /// they pick; the other arc is walked round the ellipse and charged at the
    /// ellipse's size, since it runs out to the far end. A mesher that charged
    /// that arc by its ends would hand the walk a deflection under what it can
    /// place a point to: a face on a cone of slope a half, cut by a lid `2⁻²⁴`
    /// off a ruling and a wall square to it, whose one curved edge runs from
    /// two ends `1.25` up round the far side to `1.7e7` up, was refused
    /// `CannotPlaceAPointThatCloseToACrossing` for a deflection `rcad-topo`
    /// picked itself (`a_cone_arc_past_the_far_end_of_a_long_ellipse.rs`).
    ///
    /// **Asked under a certified policy, and `false` whenever that does not
    /// settle it**: a run the walk could not name, a half-space it could not
    /// decide, a crossing that is not a plane's long ellipse round a cone. So a
    /// mesher that charges by the ends only on `true` can charge more than the
    /// walk needs, and never less.
    ///
    /// `along` is the surfaces of the two faces either side, as [`walk`] takes
    /// them, and `run` and `along` are in one frame.
    #[must_use]
    pub fn are_placed_round_the_cone_from_the_ends_of(
        run: &NamedRun,
        along: [&Surface; 2],
    ) -> bool {
        let pair = which_two_surfaces_a_run_is_between(run.run(), along);
        let Some(arc) = AnArcRoundACone::of(pair, run) else {
            return false;
        };
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let Ok(WhatARunLiesOn::Conic(conic)) =
            what_a_run_between_these_two_lies_on(pair, run, &policy, &mut log)
        else {
            return false;
        };
        let deviation = POINT_ALONG_A_CROSSING_ROUNDOFF * conic.the_size_it_was_worked_out_from();
        matches!(
            arc.which_arc(&conic, deviation, &policy, &mut log),
            Ok(WhichArc::Near(..))
        )
    }

    /// The points, in the direction the run is stored running, **with neither
    /// end included**.
    ///
    /// The ends are corners the model already holds, so whoever reads this puts
    /// them back at either side. An edge that needs no help at all — a straight
    /// one — gives an empty list.
    ///
    /// **The one exception is a run that names no end at all on a crossing that
    /// runs on for ever** — a plane and a cone crossing in a parabola or a
    /// hyperbola. There is no corner on that side for the model to put back and
    /// nothing else knows where the walk was cut off, so the extreme point of
    /// the stretch comes back with the rest. [`walk`] says which stretch and
    /// why.
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
/// # A walk that could not be refined leaves a record
///
/// When the points' own deviation has eaten the whole deflection there is no
/// step small enough to help, so a tolerant caller is handed the two ends and
/// nothing between them. That is a walk that did not do what it was asked, and
/// it lands in `log` under [`ROOM_LEFT_TO_REFINE_A_WALK`]. A walk that met the
/// deflection records nothing, which is what makes an empty log worth reading
/// (§5).
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
/// run pick both of the two arcs between its ends, or neither, and on a
/// degree-four crossing when its two ends are on two of its loops, or cannot
/// be told to be on one.
///
/// [`Error::Undecidable`] when the conic's own size could not be worked out
/// because a product fell out of range.
///
/// [`Error::AnEndOfARunNamesNoPlace`] when an end is neither the box that says
/// nothing at all is known nor a box with a middle.
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] when the run says its two
/// surfaces cross in a conic and they do not: a plane that misses the ball it is
/// named against, or touches it at one point, leaves a circle of no radius, and
/// there is no angle to walk round one. Two balls round one centre with two
/// radii get it too, for the same reason and one surface up, and so does a ball
/// on a cone's axis that misses the cone. A **limit** and not
/// a gap (§16.10) — the curve the run names does not exist at any precision.
///
/// [`Error::TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing`] for two balls round
/// one centre with one radius, which is the one surface written twice. Also a
/// limit.
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] and
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] for a run between a
/// cone or a torus and another curved surface — the same two refusals
/// [`intersect`](crate::intersect) gives those pairs, and a cone against a pipe,
/// a ball or a second cone that shares its axis, or a torus against a pipe, a
/// ball or a cone on its axis, is drawn here exactly as it is named there. The
/// torus's refusal also covers a whole circle such a partner shares with it
/// that the run names by the two faces alone, which
/// `where_a_ring_and_a_surface_on_its_axis_cut` keeps refused on purpose.
///
/// [`Error::TheCrossingOfTheseTwoSurfacesIsNotClosed`] for a run a caller
/// states as going right round a crossing that runs on for ever — a plane and a
/// cone crossing in a parabola or a hyperbola. The two shapes through the apex
/// are straight lines, and a straight run gets the same empty answer here that a
/// run between two planes does.
///
/// # An arc of a very long ellipse round a cone is walked round the cone
///
/// A plane a rounding off parallel to a cone's ruling cuts it in an ellipse
/// whose centre can be `2.2e16` out while an arc of it by a frustum is `4`
/// across, and a point placed from that centre rounds by more than the arc is
/// wide. Where the ellipse reaches more than
/// [`PointsAlongACrossing::WALKED_ROUND_THE_CONE_PAST`] times further up the
/// cone's axis than the run's two ends do, the arc that misses the ellipse's far
/// end is placed along the cone's rulings instead, and charged and counted by
/// its own size and bend. `AnArcRoundACone` says how.
///
/// # A run with no ends keeps the two points at the ends of what was walked
///
/// [`PointsAlongACrossing::points`] leaves both ends out because they are
/// corners the model already holds. **An endless bend has no such corners** —
/// [`where_a_closed_crossing_starts`] refuses it, and nothing else knows where
/// the walk was cut off — so the two extreme points of the stretch come back
/// with the rest. An end the run *does* name is left out, exactly as everywhere
/// else, and a run that names one end and not the other has one of each.
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

    let pair = which_two_surfaces_a_run_is_between(run.run(), along);
    let conic = match what_a_run_between_these_two_lies_on(pair, run, policy, log)? {
        WhatARunLiesOn::StraightLine => return Ok(nothing_between_the_ends()),
        WhatARunLiesOn::QuarticPickedUpAt(seeds, held_apart_by) => {
            return follow_the_quartic(run, pair, seeds, held_apart_by, deflection, policy, log);
        }
        WhatARunLiesOn::Parabola(bend) => {
            return walk_an_endless_bend(&bend, run, deflection, policy, log);
        }
        WhatARunLiesOn::OneBranchOfAHyperbola(branch) => {
            return walk_an_endless_bend(&branch, run, deflection, policy, log);
        }
        WhatARunLiesOn::Conic(conic) => conic,
    };
    let deviation = POINT_ALONG_A_CROSSING_ROUNDOFF * conic.the_size_it_was_worked_out_from();
    // A conic drawn near the top of `f64`'s range has a bound that overflows,
    // and a bound of infinity is not a bound. The size of this crossing is what
    // could not be worked out, which is the same refusal the naming half has one
    // step earlier.
    if !deviation.is_finite() {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    }
    if let Some(arc) = AnArcRoundACone::of(pair, run) {
        return arc.walk(&conic, deviation, deflection, policy, log);
    }
    let Some(room) = room_left_over(deflection, deviation, policy, log)? else {
        return Ok(nothing_walked(deviation));
    };

    let stretch = conic.the_stretch_this_run_covers(run, deviation, policy, log)?;
    points_round_a_conic(&conic, stretch, deviation, room)
}

/// Points along `stretch` of a conic, `room` apart from the curve at most.
fn points_round_a_conic(
    conic: &Conic,
    (from, to): (f64, f64),
    deviation: f64,
    room: f64,
) -> Result<PointsAlongACrossing> {
    let steps = how_many_steps(conic, to - from, room)?;
    let points = (1..steps)
        .map(|step| conic.at(from + (to - from) * (step as f64) / (steps as f64)))
        .collect();

    Ok(PointsAlongACrossing {
        points,
        deviation: Approximation::new(deviation, Cause::PointAlongACrossing),
    })
}

/// What a walk a tolerant caller waved through hands back: no points between
/// the ends, and how far off they would have been.
fn nothing_walked(deviation: f64) -> PointsAlongACrossing {
    PointsAlongACrossing {
        points: Vec::new(),
        deviation: Approximation::new(deviation, Cause::PointAlongACrossing),
    }
}

/// The one place on a closed crossing that a vertex can go.
///
/// **A wire needs a coedge, an edge needs two ends, and a closed edge needs both
/// of them to be one vertex** — so a fuse that cuts a face along a whole closed
/// crossing has to put a vertex somewhere on it, and there is no corner to take
/// (`docs/decisions.md` §31.3). Where it goes must depend on the two surfaces
/// and on nothing else, or `fuse(A, B)` and `fuse(B, A)` put it in two places.
///
/// This is where [`walk`] starts, and that is the whole of the design: asking
/// the question twice, once here and once there, is the shape §23.8 kept
/// finding. For a conic it is the point at angle zero; for a degree-four loop it
/// is the seed the marcher is handed, settled onto both surfaces.
///
/// # It is a point and not a box, and that is not a promise it is exact
///
/// The same thing [`walk`]'s points are. How far it may sit off the crossing is
/// [`PointsAlongACrossing::deviation`] for the same run — one number for the
/// whole run and never one per point (§24.13) — so a caller that needs the box
/// takes it from there.
///
/// # Errors
///
/// [`Error::TheCrossingOfTheseTwoSurfacesIsNotClosed`] for a pair that crosses
/// in straight lines: two planes, a plane down a pipe's own axis, and two pipes
/// pointing the same way.
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the half-spaces on a
/// degree-four run pick both loops or neither, and
/// [`Error::CannotWorkOutHowBigACrossingIs`] when the crossing's size or a seed
/// on it falls out of `f64`'s range — a conic's point at angle zero included,
/// which is the one this used to hand back anyway (§74).
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] for a pair whose conic is not a
/// curve — a plane that misses or grazes the ball it is named against — exactly
/// as [`walk`] does, and for the same reason: there is nowhere on a circle of no
/// radius that the pair alone picks out.
///
/// [`Error::TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing`] for two balls round
/// one centre with one radius, exactly as [`walk`] does.
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] and
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] for a cone or a
/// torus against another curved surface, exactly as [`walk`] does.
///
/// [`Error::TheCrossingOfTheseTwoSurfacesIsNotClosed`] for a plane against a
/// cone that cross in a parabola or a hyperbola, which is the same refusal as
/// for the straight lines above and for the same reason: a curve that runs on
/// for ever has no one place a vertex has to go. A plane through the apex
/// crosses the cone in straight lines and gets it too.
///
/// # It takes no policy, so it asks the half-spaces under a certified one
///
/// [`walk`] hands its caller's policy down to the half-spaces that say which
/// piece of a crossing a run is, so a tolerant caller's budget is spent there.
/// This does not, because it has no policy to hand down, and a picker it cannot
/// decide leaves it with the same tie a certified caller gets.
///
/// **That is a gap and not a limit**, and the reason it is still here is that
/// closing it changes a `pub` signature that `rcad-topo`'s fuse and half a dozen
/// test files call. Nothing measured reaches a picker through this door: while
/// the STEP corpus still left undecided half-spaces behind they all came in
/// through [`walk`], from `Model::validate` checking which way a flat face's
/// wire winds, and since the closed form below the corpus leaves none at all
/// (`docs/decisions.md` §118.8).
///
/// **"Nothing measured" is a fact about the paths one corpus takes, not about
/// this door, so here is the door itself.** Inside the kernel the only callers
/// are in `rcad-topo`'s `fuse::arrange`, so a consumer reaches this by fusing.
/// Importing and meshing do not pass through it — `Import::read` builds and
/// validates, and `SolidView::mesh` never asks which piece of a crossing a run
/// is. The other way in is this function itself, which is re-exported from
/// `rcad` and which anyone may call.
///
/// So this is reached only by a caller that **fuses**, and it hands that caller a
/// certified answer whatever policy it asked for. Check yourself against that
/// rather than against the census.
///
/// **A display layer's own policy is not evidence about its application's.** The
/// library that draws a `Model` is usually not the code that built it: rscene,
/// for one, takes `&Model` and never mutates one, so its own tolerant policy is
/// spent on meshing and never on arranging a crossing — the two sit on opposite
/// sides of that seam by design. An application that fuses tolerantly *does* sit
/// on this gap, and neither the kernel nor the drawing layer can see it, because
/// the fuse happens before the model reaches either of them.
pub fn where_a_closed_crossing_starts(
    run: &NamedRun,
    along: [&Surface; 2],
    log: &mut ProvenanceLog,
) -> Result<Point> {
    let certified = Policy::certified();
    let pair = which_two_surfaces_a_run_is_between(run.run(), along);
    match what_a_run_between_these_two_lies_on(pair, run, &certified, log)? {
        WhatARunLiesOn::StraightLine => Err(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed),
        WhatARunLiesOn::QuarticPickedUpAt(seeds, held_apart_by) => {
            the_seed_this_run_is_picked_up_at(run, &pair, seeds, held_apart_by, &certified, log)
        }
        WhatARunLiesOn::Parabola(_) | WhatARunLiesOn::OneBranchOfAHyperbola(_) => {
            Err(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed)
        }
        // **Asked of the point and not of the conic's size**, which is the
        // guard [`walk`] has one step further on and it is not the same guard.
        // A conic whose size overflowed skips the degenerate check in
        // `what_a_run_between_these_two_lies_on` on purpose, and what comes
        // through is a semi-axis of infinity — so `and · sin 0` is `∞ · 0`, and
        // every coordinate of the start is `NaN`. Checking the size would catch
        // that pair and leave a sufficiency claim over six conic builders and
        // the direction vectors each of them scales; checking the answer leaves
        // none (§74).
        WhatARunLiesOn::Conic(conic) => {
            let start = conic.at(0.0);
            if !start.is_finite() {
                return Err(Error::CannotWorkOutHowBigACrossingIs);
            }
            Ok(start)
        }
    }
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

/// What a run between two surfaces lies on: one of three things, and which one
/// is worked out in a single place.
///
/// **Exhaustive on purpose.** A sixth shape of crossing would mean a sixth
/// way of walking one, and the point of this enum is that both callers handle
/// every shape there is.
enum WhatARunLiesOn {
    /// A straight line, so the run's two ends are the whole of it and there is
    /// no closed curve to start anywhere.
    StraightLine,
    /// A conic in a plane, walked round in an angle.
    Conic(Conic),
    /// A degree-four loop, with one point near each piece it can come in for
    /// the marcher to start from, and what holds those two pieces apart when
    /// there are two.
    QuarticPickedUpAt([Point; 2], TwoLoopsHeldApartBy),
    /// A parabola: one curve that runs on for ever both ways and bends.
    Parabola(Parabola),
    /// One of a hyperbola's two branches, which runs on for ever both ways and
    /// bends too. The other branch is the sibling run, and which of the two
    /// this is was settled by the run's ends, or by its half-spaces when it
    /// has none.
    OneBranchOfAHyperbola(HyperbolaBranch),
}

/// The flat surface that holds a degree-four crossing's two loops apart, kept
/// as the two surfaces it comes from, so that which side of it a point is on is
/// asked of the numbers they store and of nothing rounded from them.
///
/// **Exhaustive on purpose.** These are the two pairs that are marched, and a
/// third would need its own flat surface worked out before it could be.
#[derive(Clone, Copy)]
enum TwoLoopsHeldApartBy {
    /// The one through the ball's centre square to the pipe's axis, which
    /// [`intersect`](crate::intersect)'s `two_loops_either_side_of_a_ball`
    /// picks with a rounding of.
    ABallsCentre { ball: Sphere, pipe: Cylinder },
    /// The one through the fat pipe's axis square to the thin pipe's way across
    /// it. `two_loops_either_side_of_a_pipe` picks with one that leans off it
    /// wherever that way cannot be stored square.
    AFatPipesAxis { thin: Cylinder, fat: Cylinder },
}

/// Which of those three the crossing of these two surfaces is, and the thing
/// needed to walk it.
///
/// **This is the one list of what a pair of surfaces does**, and it is one list
/// because three of them were kept in step by hand and were not. A guard said
/// which pairs the marcher had a seed for, a `match` said which had a conic, and
/// a third `match` said where a seed came from; a cone and a torus landed and
/// were added to none of the three, so twelve pairs fell off the end of the
/// second one into a `panic!` that a real KiCad file reached (§24.5). Every pair
/// is now named once, here, and a sixth kind of [`Surface`] is a compile error
/// in this `match` — which is the only place that has to know.
///
/// # Errors
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] and
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] for the pairs
/// [`intersect`](crate::intersect) refuses under those same two names, in the
/// same order — a cone against a torus is the torus's refusal in both halves,
/// so the namer and the evaluator never disagree about what a pair is called.
/// **The evaluator refuses exactly what the namer refuses**, which is why the
/// cone's arms against a pipe, a ball and a second cone ask whether the two
/// share an axis rather than taking the caller's word for it, and the torus's
/// arms against them ask `intersect` itself. Both refusals are gaps: the
/// crossing is there and none of the machinery for a degree-four or
/// degree-eight curve with a cone or a torus in it exists. A ball through a
/// cone's apex is refused under the cone's name in both halves too, for the
/// reason `intersect`'s cone-and-ball arm gives.
///
/// **One pair is refused here under that name and answered there**: two cones
/// on one axis with one apex and two slopes, which `intersect` answers as
/// touching at the apex. A closed run stated on them names nothing, and the
/// refusal keeps an older name — `where_two_coaxial_cones_cut` says why.
///
/// **One cone twice is not one of those pairs**, and that is the one place the
/// evaluator says more than the namer's table does: the namer answers it as one
/// surface, and a run right round where that surface meets itself is a circle
/// the run's own end names (§214). "One cone" is the namer's own exact test, so
/// a cone written about `+z` against the same cone about `−z` is one cone here
/// too (§357.2). Every other run on that pair still refuses.
///
/// [`Error::ACrossingPieceHasNoEndsAndNothingPickingIt`] for such a closed run
/// with no ends, which leaves nothing to put the circle through.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a seed for a degree-four
/// crossing comes out as something that is not a point, or when that circle's
/// own numbers do not come out finite.
///
/// [`Error::TheCrossingOfTheseTwoSurfacesIsNotClosed`] for a run a caller named
/// as closed between a plane and a cone that cross in a parabola or a
/// hyperbola, and [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the
/// half-spaces on a hyperbola's run hold both branches or neither.
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] when the pair has a conic
/// builder and what comes out of it is not a curve — a plane that misses a ball
/// altogether, or touches it at one point, is the shape a stranger's model
/// reached this by. See [`Conic::has_an_angle_round_it`]. Two balls round one
/// centre with two radii are the same thing one surface up, refused by
/// [`where_two_balls_cut`] before a conic is built at all, and so is a ball on
/// a cone's axis that misses the cone.
///
/// [`Error::TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing`] for two balls round
/// one centre with one radius.
///
/// Whatever the conic builders refuse, for the pairs that have one.
fn what_a_run_between_these_two_lies_on(
    pair: [Surface; 2],
    run: &NamedRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhatARunLiesOn> {
    let lies_on = match pair {
        [Surface::Plane(_), Surface::Plane(_)] => Ok(WhatARunLiesOn::StraightLine),
        [Surface::Plane(flat), Surface::Sphere(ball)]
        | [Surface::Sphere(ball), Surface::Plane(flat)] => {
            where_a_plane_cuts_a_ball(&flat, &ball).map(WhatARunLiesOn::Conic)
        }
        // A plane along a pipe's axis cuts it in straight lines, and a run that
        // is one of those is picked out by a half-space rather than being
        // `Straight`, so it reaches here and there is no ellipse to build.
        [Surface::Plane(flat), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Plane(flat)] => {
            if flat.normal().dot(pipe.axis().direction()) == 0.0 {
                return Ok(WhatARunLiesOn::StraightLine);
            }
            where_a_plane_cuts_a_pipe(&flat, &pipe).map(WhatARunLiesOn::Conic)
        }
        // Every shape a plane and a cone cross in reaches here, and every one
        // of them is answered: `where_a_plane_cuts_a_cone` asks the same two
        // signs the namer asks and builds whichever of the four shapes they
        // name. It is the second of the two places that reads a run rather than
        // only a pair, because a hyperbola comes in two branches and which one
        // this is is the picker's to say.
        [Surface::Plane(flat), Surface::Cone(cone)]
        | [Surface::Cone(cone), Surface::Plane(flat)] => {
            where_a_plane_cuts_a_cone(&flat, &cone, run, policy, log)
        }
        // A plane cuts a torus in **two** circles as often as in one, and which
        // of them this run is is the picker's to say — which is why this is the
        // one arm here that reads the run rather than only the pair.
        [Surface::Plane(flat), Surface::Torus(ring)]
        | [Surface::Torus(ring), Surface::Plane(flat)] => {
            where_a_plane_cuts_a_torus(&flat, &ring, run, policy, log).map(WhatARunLiesOn::Conic)
        }
        // **In a fixed order, and it is not tidiness.** `where_two_balls_cut`
        // builds the radical plane through the gap between the two centres, so
        // the pair the other way round gets that plane reversed, a different
        // `basis` out of it, and a whole different starting angle — measured,
        // forty of forty swapped pairs walked into different points, and not by
        // a rounding. The other three conic pairs are already order-free
        // because both arms of their `match` call the same function with the
        // same arguments. This makes the fourth one so, which is what
        // `where_a_closed_crossing_starts` needs and what §31.3 assumed
        // (§34).
        [Surface::Sphere(one), Surface::Sphere(other)] => {
            let (first, second) = if Surface::Sphere(one)
                .how_it_sorts_against(&Surface::Sphere(other))
                == Ordering::Greater
            {
                (other, one)
            } else {
                (one, other)
            };
            where_two_balls_cut(&first, &second).map(WhatARunLiesOn::Conic)
        }
        // A ball centred on a pipe's axis and wider than it crosses it in two
        // circles square to the axis, and which of them this run is, is its
        // ends' and its picker's to say, as for a cone and a pipe on one axis.
        // Marched instead, a closed run with a corner on the second circle was
        // walked round the first, because a run nothing picks starts at the
        // first seed (Q1241). A ball no wider than the pipe is marched as
        // before: drawing the one circle a ball of the pipe's own width touches
        // it round lets `SolidBuilder` build a rim it refuses today, which
        // `rcad-step` pins.
        [Surface::Sphere(ball), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Sphere(ball)] => {
            if ball.radius() > pipe.radius()
                && whether_a_ball_sits_on_a_pipes_axis(&ball, &pipe) == Decision::Zero
            {
                let ball_first = matches!(pair[0], Surface::Sphere(_));
                where_a_ball_on_a_pipes_axis_cuts_it(&ball, &pipe, ball_first, run, policy, log)
                    .map(WhatARunLiesOn::Conic)
            } else {
                a_quartic_picked_up_at(
                    where_a_ball_reaches_across_a_pipe(&ball, &pipe),
                    TwoLoopsHeldApartBy::ABallsCentre { ball, pipe },
                )
            }
        }
        [Surface::Cylinder(one), Surface::Cylinder(other)] => {
            // Two pipes down one direction cross in straight lines, and a run
            // that is one of those is picked out by a half-space rather than
            // being `Straight`, so it reaches here.
            if one.axis().direction().cross(other.axis().direction()) == Vector::zero() {
                return Ok(WhatARunLiesOn::StraightLine);
            }
            // Two pipes of one width whose axes actually meet cross in two
            // ellipses and not in a quartic, and the marcher cannot follow
            // them: the four branches run into each other where the walls
            // touch. Asked before the sort below, because the answer is the
            // same either way round and the closed form does its own sorting.
            if whether_two_pipes_are_one_width_on_axes_that_meet(&one, &other, policy, log)? {
                return where_two_equal_pipes_on_crossing_axes_cut(&one, &other, run, policy, log)
                    .map(WhatARunLiesOn::Conic);
            }
            // Two pipes of the same radius are neither thinner, and
            // then which one is walked round has to be settled by the numbers
            // they store rather than by which of them the caller wrote first —
            // otherwise the seed, and the vertex `where_a_closed_crossing_starts`
            // puts on the loop, move when the pair is swapped.
            let mine_is_thinner = match one.radius().total_cmp(&other.radius()) {
                Ordering::Equal => {
                    Surface::Cylinder(one).how_it_sorts_against(&Surface::Cylinder(other))
                        != Ordering::Greater
                }
                Ordering::Less => true,
                Ordering::Greater => false,
            };
            let (thin, fat) = if mine_is_thinner {
                (one, other)
            } else {
                (other, one)
            };
            a_quartic_picked_up_at(
                where_one_pipe_reaches_across_another(&thin, &fat),
                TwoLoopsHeldApartBy::AFatPipesAxis { thin, fat },
            )
        }
        // A cone and a pipe on one axis cross in two circles and everything
        // else in that row is refused, so this arm reads the run: which of the
        // two circles it is, is the picker's to say.
        [Surface::Cone(cone), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Cone(cone)] => {
            where_a_coaxial_cone_and_pipe_cut(&cone, &pipe, run, policy, log)
                .map(WhatARunLiesOn::Conic)
        }
        // One cone twice, which is one surface and crosses nothing. `intersect`
        // has said so since it was written — its `SameSurface` arm — and this
        // had no arm for the pair at all, so a fuse that cut a cone face in two
        // was told a frustum and a block held a second curved surface between
        // them (§214). A **guarded** arm on purpose: the refusal below still
        // covers a cone against a different cone, and a sixth kind of `Surface`
        // is still a compile error here, which a wildcard would have cost.
        //
        // **One cone whichever way each face writes its axis**, asked exactly as
        // `intersect` asks it (§357.2), and the circle built from the spelling
        // that sorts first. The two spellings build one circle round opposite
        // ways, so taking the one listed first would walk one rim two ways for
        // the two faces either side of it; the one that sorts first is also the
        // one `rcad-topo`'s `the_curved_one_of` reads a closed edge's direction
        // off.
        [Surface::Cone(one), Surface::Cone(other)]
            if one_cone_either_way_round(&one, &other)
                && run.run() == &EdgeRun::RightRoundWhereItsFacesCross =>
        {
            let first = match Surface::Cone(one).how_it_sorts_against(&Surface::Cone(other)) {
                Ordering::Greater => other,
                Ordering::Less | Ordering::Equal => one,
            };
            the_circle_a_closed_run_round_one_cone_lies_on(&first, run).map(WhatARunLiesOn::Conic)
        }
        // Two cones on one axis cross in one circle or two, and a ball on a
        // cone's axis in none, one or two; which one this run is, is its ends'
        // and its picker's to say, as for the coaxial pipe.
        [Surface::Cone(one), Surface::Cone(other)] => {
            where_two_coaxial_cones_cut(&one, &other, run, policy, log).map(WhatARunLiesOn::Conic)
        }
        [Surface::Cone(cone), Surface::Sphere(ball)]
        | [Surface::Sphere(ball), Surface::Cone(cone)] => {
            where_a_ball_on_a_cones_axis_cuts_it(&cone, &ball, run, policy, log)
                .map(WhatARunLiesOn::Conic)
        }
        // A ring and a ball, a pipe or a cone on its axis cross in circles
        // square to that axis, and which of them this run is, is its ends' and
        // its pickers' to say, as for a cone and a pipe on one axis.
        [
            Surface::Torus(ring),
            partner @ (Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_)),
        ]
        | [
            partner @ (Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_)),
            Surface::Torus(ring),
        ] => where_a_ring_and_a_surface_on_its_axis_cut(&ring, partner, run, policy, log)
            .map(WhatARunLiesOn::Conic),
        [Surface::Torus(_), Surface::Torus(_)] => {
            Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface)
        }
    }?;

    // A conic builder is handed the pair the caller's run *says* it lies on,
    // and a stranger's model says whatever it likes: a seam plane 512 away from
    // the ball it claims to cut leaves `where_a_plane_cuts_a_ball` a negative
    // discriminant, which clamps to a radius of zero. Every road out of a conic
    // that shrank to a point runs through `NaN`, so it is stopped here, once,
    // rather than at each of them.
    //
    // **Asked only of a conic whose size is a number**, because a conic too big
    // to put a bound on arrives here looking exactly like one that shrank: an
    // overflowed semi-axis length times a direction with a zero in it is `NaN`
    // in that coordinate, so no length of it is > 0. That one already has a name
    // one step later in `walk` and it is a different one —
    // `CannotWorkOutHowBigACrossingIs`, a gap rather than a limit, because the
    // same pair written smaller is walked without complaint.
    if let WhatARunLiesOn::Conic(conic) = &lies_on
        && conic.the_size_it_was_worked_out_from().is_finite()
        && !conic.has_an_angle_round_it()
    {
        return Err(Error::TheseTwoSurfacesDoNotCrossInAConic);
    }
    Ok(lies_on)
}

/// The two seeds of a degree-four crossing, once they have been checked for
/// being points at all.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when either comes out as something
/// that is not a point.
fn a_quartic_picked_up_at(
    seeds: [Point; 2],
    held_apart_by: TwoLoopsHeldApartBy,
) -> Result<WhatARunLiesOn> {
    if seeds.iter().any(|seed| !seed.is_finite()) {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    }
    Ok(WhatARunLiesOn::QuarticPickedUpAt(seeds, held_apart_by))
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
/// run pick both of the loops the crossing leaves, or neither, and when a
/// run's corner cannot be put on one side of what holds the two loops apart,
/// or is across it from both seeds. For a run between two corners, also when
/// its corners are on different seeds' sides, sit at one place on the loop, or
/// leave two arcs between them that its half-spaces both hold or neither
/// holds ([`the_arc_between`]).
///
/// [`Error::CannotPlaceAPointThatCloseToACrossing`] when the marcher's own bound
/// takes more than its half of the deflection and the policy will not accept it.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a settled point has no box
/// round it.
///
/// The marcher's two refusals.
fn follow_the_quartic(
    run: &NamedRun,
    pair: [Surface; 2],
    seeds: [Point; 2],
    held_apart_by: TwoLoopsHeldApartBy,
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<PointsAlongACrossing> {
    let half_the_budget = deflection.length() / 2.0;
    let surfaces = [&pair[0], &pair[1]];
    let seed = the_seed_this_run_is_picked_up_at(run, &pair, seeds, held_apart_by, policy, log)?;

    let walked = march(
        surfaces,
        seed,
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
    let points = if let Some(corner) = the_corner_a_closed_run_starts_at(run) {
        let (start, off) = settled_with_a_bound(surfaces, seed)?;
        begun_beside(corner, start, off, walked.points)?
    } else if let Some(ends) = the_two_corners_of_an_open_run(run) {
        let (start, _) = settled_with_a_bound(surfaces, seed)?;
        let picked_by = the_half_spaces_that_pick(run);
        the_arc_between(ends, picked_by, surfaces, start, walked, policy, log)?
    } else {
        walked.points
    };
    Ok(PointsAlongACrossing {
        points,
        deviation: tag,
    })
}

/// A closed loop the marcher walked from `start`, turned round so that it
/// begins and ends beside `corner`.
///
/// **Handed back as it was walked when `corner` is `start`**, to within what
/// the arithmetic knows of either: the corner's own box and the box `off` round
/// `start` share a point. A march already begins and ends one step from its
/// start, and the caller puts the corner at both ends, so putting `start` back
/// would put the corner among the points a second time. A fuse puts the corner
/// at the seed ([`where_a_closed_crossing_starts`]), and a mesh shifts it into
/// its own frame and settles the seed again there, so the two can land an ulp
/// apart, and a loop that held both was refused by the mesh as two corners in
/// one place (Q1508).
///
/// Anywhere else `start` is put back among the points, because it is not an
/// end any more. The corner goes between the two points it sits between — the
/// nearest and whichever of its two neighbours is nearer — so splitting that
/// one step at a point on the curve is all that changes, and a step split at a
/// point on the curve strays from it less than the step did.
///
/// **Turned and never re-marched from the corner**, and that is measured: the
/// LED's corner sits where its two surfaces touch, and a march begun there
/// bounds its first point `1.58` off the crossing against a deflection of `1.30`
/// asked, where the march begun at the seed is walked (§189.3).
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when `start` has no box round it.
fn begun_beside(
    corner: PredicatePoint,
    start: Point,
    off: f64,
    walked: Vec<Point>,
) -> Result<Vec<Point>> {
    let Some(round_the_start) = somewhere_within(start, off) else {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    };
    if the_boxes_share_a_point(corner, round_the_start) {
        return Ok(walked);
    }
    let corner = Point::from(corner.enclosure().map(middle_of));
    let mut round = Vec::with_capacity(walked.len() + 1);
    round.push(start);
    round.extend(walked);
    let count = round.len();
    let how_far = |at: usize| (round[at % count] - corner).length();
    let nearest = (0..count)
        .min_by(|one, other| how_far(*one).total_cmp(&how_far(*other)))
        .expect("the loop holds at least the point it was walked from");
    let first = if how_far(nearest + count - 1) < how_far(nearest + 1) {
        nearest
    } else {
        nearest + 1
    };
    round.rotate_left(first % count);
    Ok(round)
}

/// The points of a closed loop the marcher walked from `start` that lie between
/// a run's two corners, along whichever of the two arcs between them the run's
/// half-spaces hold, in order from the first corner to the second.
///
/// **A corner the march passed through is not handed back as a point**, for the
/// same reason [`begun_beside`] leaves out a corner that is the start: the caller
/// puts both corners at the ends, and a corner among the points as well is two
/// corners in one place (Q1508). A fuse puts a run's corner where the crossing
/// meets a third surface, and the seed of a crossing of two pipes can be exactly
/// there: a thin bar through a fat one cut by a wall through both axes has its
/// corners and its seeds all on the wall.
///
/// The half-spaces are asked at the middle point of each arc, of the box the
/// march says that point is inside, the way a conic's arc is asked at its middle
/// angle ([`Conic::the_stretch_this_run_covers`]). An arc with no marched point
/// on it is asked at the middle of its two corners, settled onto the crossing.
/// Every point handed back is a point the march settled and bounded, so the
/// march's bound holds of them all.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the two corners sit at
/// one place on the loop, or the half-spaces hold both arcs or neither.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a point has no box round it,
/// and what settling the middle of an arc with no point on it refuses.
fn the_arc_between(
    [from, to]: [PredicatePoint; 2],
    picked_by: &[Picker],
    surfaces: [&Surface; 2],
    start: Point,
    walked: Marched,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<Point>> {
    let deviation = walked.deviation;
    let mut round = Vec::with_capacity(walked.points.len() + 1);
    round.push(start);
    round.extend(walked.points);
    let count = round.len();
    let mut kept = vec![true; count];
    let first_end = where_on_the_loop_this_end_is(from, &round, deviation, &mut kept)?;
    let second_end = where_on_the_loop_this_end_is(to, &round, deviation, &mut kept)?;
    let laps = count as f64;
    let reach = (second_end - first_end).rem_euclid(laps);
    if reach == 0.0 {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let mut onwards = Vec::new();
    let mut backwards = Vec::new();
    let first = first_end.floor() as usize;
    for step in 1..=count {
        let at = (first + step) % count;
        if !kept[at] {
            continue;
        }
        let along = (at as f64 - first_end).rem_euclid(laps);
        if along > 0.0 && along < reach {
            onwards.push(round[at]);
        } else {
            backwards.push(round[at]);
        }
    }
    backwards.reverse();

    let mut held = [false; 2];
    for (arc, holds) in [&onwards, &backwards].into_iter().zip(&mut held) {
        *holds = match arc.get(arc.len() / 2) {
            Some(&middle) => every_half_space_holds(picked_by, middle, deviation, policy, log)?,
            None => {
                let [one, other] =
                    [from, to].map(|end| Point::from(end.enclosure().map(middle_of)));
                let between = one + (other - one) * 0.5;
                let (middle, off) = settled_with_a_bound(surfaces, between)?;
                every_half_space_holds(picked_by, middle, off, policy, log)?
            }
        };
    }
    match held {
        [true, false] => Ok(onwards),
        [false, true] => Ok(backwards),
        [true, true] | [false, false] => Err(Error::CannotNameWhichPieceOfACrossingThisIs),
    }
}

/// Where on the closed loop of points `round` an end of a run sits: the index
/// of the point it is, or of the point that starts the step it sits along plus
/// how far along that step it is.
///
/// The end is put on the step nearest it, and is that step's point at either
/// end when its own box and the box `deviation` round that point share a point;
/// that point is then marked as not `kept`.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a point has no box round it.
fn where_on_the_loop_this_end_is(
    end: PredicatePoint,
    round: &[Point],
    deviation: f64,
    kept: &mut [bool],
) -> Result<f64> {
    let count = round.len();
    let at = Point::from(end.enclosure().map(middle_of));
    let along_the_step_from = |here: usize| {
        let way = round[(here + 1) % count] - round[here];
        let along = (at - round[here]).dot(way) / way.length_squared().max(f64::MIN_POSITIVE);
        (along.clamp(0.0, 1.0), way)
    };
    let how_far = |here: usize| {
        let (along, way) = along_the_step_from(here);
        (round[here] + way * along - at).length()
    };
    let nearest = (0..count)
        .min_by(|one, other| how_far(*one).total_cmp(&how_far(*other)))
        .expect("the loop holds at least the point it was walked from");
    for corner in [nearest, (nearest + 1) % count] {
        let Some(round_the_corner) = somewhere_within(round[corner], deviation) else {
            return Err(Error::CannotWorkOutHowBigACrossingIs);
        };
        if the_boxes_share_a_point(end, round_the_corner) {
            kept[corner] = false;
            return Ok(corner as f64);
        }
    }
    Ok(nearest as f64 + along_the_step_from(nearest).0)
}

/// Whether two boxes have a point in common, so that nothing the arithmetic
/// knows tells the two places they stand for apart.
fn the_boxes_share_a_point(one: PredicatePoint, other: PredicatePoint) -> bool {
    one.enclosure()
        .into_iter()
        .zip(other.enclosure())
        .all(|(mine, theirs)| mine.lo() <= theirs.hi() && theirs.lo() <= mine.hi())
}

/// The corner a closed run starts and ends at, when it names one.
///
/// **The points come back with neither end included, so they have to start
/// beside the corner.** The caller puts the corner back at both ends of what is
/// walked; a march begun anywhere else hands back a loop that opens with a
/// chord from the corner to wherever the seed was. A fuse puts its vertex at
/// the seed ([`where_a_closed_crossing_starts`]), so there the two agree by
/// construction. A file does not: `LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm`'s
/// body meets its dome at a corner a quarter turn from the seed
/// (`docs/decisions.md` §189.3).
///
/// `None` for a run whose ends are not one named place, which is an open run
/// or a whole curve nothing has put a corner on yet.
fn the_corner_a_closed_run_starts_at(run: &NamedRun) -> Option<PredicatePoint> {
    let [from, to] = run.ends();
    (from == to && from != nowhere_in_particular()).then_some(from)
}

/// The two corners a run that is one piece of a crossing runs between, when it
/// names two places and they are not one.
///
/// `None` for a closed run, a whole curve, and a run with one end that names
/// nothing, which are all walked round a whole loop as before.
fn the_two_corners_of_an_open_run(run: &NamedRun) -> Option<[PredicatePoint; 2]> {
    let [from, to] = run.ends();
    let is_a_piece = matches!(run.run(), EdgeRun::PartOfWhereItCrosses { .. });
    (is_a_piece && from != to && from != nowhere_in_particular() && to != nowhere_in_particular())
        .then_some([from, to])
}

/// The place on the loop this run names that the marcher starts from, settled
/// onto both surfaces.
///
/// A run that *is* picked is one of two loops, and then exactly one of the
/// seeds may pass: both or neither and the half-spaces have not told them apart
/// after all.
///
/// A run that nothing picks is either the whole of a single loop or a closed
/// run whose corner names which loop it is. The first may start at either
/// seed, since both are on it. The second starts at the seed on its corner's
/// loop, and the flat surface between the two loops says which that is
/// ([`the_seed_on_the_loop_a_corner_is_on`]). Starting it at the
/// first seed walked a ball `0.1` off a pipe's axis with its corner at
/// `z = −3.9` round the loop at `z = +3.9` (Q1262).
///
/// **A run between two corners starts on its corners' loop the same way**, and
/// its half-spaces are not asked: they pick which of the two arcs between the
/// corners the run is, not which loop. A wall through both axes of a thin bar
/// through a fat one holds the seed of both loops, which sit on the wall, and
/// was refused for it (Q1659). Both corners are asked, and a run whose two
/// corners are on different seeds' sides is refused: two loops would put them
/// on two loops, and one loop that reaches across the flat surface between
/// them is not told from two here.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the half-spaces pick
/// both loops or neither, when a run's two corners are on different seeds'
/// sides, and whatever [`the_seed_on_the_loop_a_corner_is_on`] refuses.
fn the_seed_this_run_is_picked_up_at(
    run: &NamedRun,
    pair: &[Surface; 2],
    seeds: [Point; 2],
    held_apart_by: TwoLoopsHeldApartBy,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Point> {
    let surfaces = [&pair[0], &pair[1]];
    let picked_by = the_half_spaces_that_pick(run);

    if let Some([from, to]) = the_two_corners_of_an_open_run(run) {
        let settled = [
            settled_with_a_bound(surfaces, seeds[0])?,
            settled_with_a_bound(surfaces, seeds[1])?,
        ];
        let seed = the_seed_on_the_loop_a_corner_is_on(from, held_apart_by, settled, policy, log)?;
        if the_seed_on_the_loop_a_corner_is_on(to, held_apart_by, settled, policy, log)? != seed {
            return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
        }
        return Ok(seed);
    }
    if picked_by.is_empty() {
        let settled = [
            settled_with_a_bound(surfaces, seeds[0])?,
            settled_with_a_bound(surfaces, seeds[1])?,
        ];
        return match the_corner_a_closed_run_starts_at(run) {
            Some(corner) => {
                the_seed_on_the_loop_a_corner_is_on(corner, held_apart_by, settled, policy, log)
            }
            None => Ok(settled[0].0),
        };
    }
    let mut picked = SmallVec::<[Point; 2]>::new();
    for seed in seeds {
        let (settled, off) = settled_with_a_bound(surfaces, seed)?;
        if every_half_space_holds(picked_by, settled, off, policy, log)? {
            picked.push(settled);
        }
    }
    match picked.as_slice() {
        [only] => Ok(*only),
        _ => Err(Error::CannotNameWhichPieceOfACrossingThisIs),
    }
}

/// The half-spaces that say which piece of a crossing a run is, and none for a
/// run that is not one piece of one.
fn the_half_spaces_that_pick(run: &NamedRun) -> &[Picker] {
    match run.run() {
        EdgeRun::PartOfWhereItCrosses { picked_by, .. } => picked_by,
        EdgeRun::Straight
        | EdgeRun::RightRoundWhereItsFacesCross
        | EdgeRun::RightRoundWhereItCrosses { .. } => &[],
    }
}

/// Of the two settled seeds, each with how far off the crossing it may be, the
/// first on `corner`'s side of the flat surface between the two loops, or on
/// that surface.
///
/// **Why that answers which loop the corner is on.** Where the crossing leaves
/// two loops, every point of each is on its own side of that surface and well
/// clear of it: for a ball at least `√(R² − (r + d)²)` along the pipe's axis
/// from its centre, `d` how far the centre is off that axis, and for two pipes
/// at least `√(r₂² − (δ + r₁)²)` across the fat one's axis, as `intersect`'s
/// `two_loops_either_side_of_a_pipe` says. The two seeds are the two ends of
/// one reach along a line, either side of the point of that line on the
/// surface, so there is one on each loop. So the corner's side names its loop,
/// and exactly one seed is on that side. Where the crossing leaves one loop,
/// both seeds are on it and either will do, and they sit one each side of the
/// surface or both on it, so one of them is on the corner's side or on the
/// surface.
///
/// **The surface asked is the true one, never the half-space the naming half
/// stores**, and that is the whole of what this is right about. For two pipes
/// [`crate::HalfSpace::through_a_point_square_to`] stores a normal square to the fat
/// axis that leans off the thin pipe's way across it wherever that way cannot be
/// stored square. Where the naming half proves that leaning surface holds the
/// loops apart anyway, it names them; where the loops nearly touch it refuses,
/// and the leaning surface cuts through a loop. Asked of it, a corner on the
/// loop it cut was walked round the other one with none of its points on its
/// own (Q1520).
///
/// A corner or a seed **on** the flat surface is on no loop of two, so the
/// crossing is a single loop and the first seed, or that seed, is taken. A
/// corner or a seed whose side cannot be told is refused rather than guessed
/// at. An exact corner's side is told unless a product leaves the range exact
/// arithmetic holds, so a corner refused for its own side is in practice a box
/// that reaches across the surface.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the corner's side or
/// a seed's cannot be told, or both seeds are across from the corner.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a seed has no box round it.
fn the_seed_on_the_loop_a_corner_is_on(
    corner: PredicatePoint,
    held_apart_by: TwoLoopsHeldApartBy,
    seeds: [(Point, f64); 2],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Point> {
    let the_corners_side =
        match which_side_of_what_holds_two_loops_apart(held_apart_by, corner, policy, log) {
            Decision::Pos => Decision::Pos,
            Decision::Neg => Decision::Neg,
            Decision::Zero => return Ok(seeds[0].0),
            Decision::Undecidable { .. } => {
                return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
            }
            other => panic!("a side of a flat surface came back as {other:?}"),
        };
    for (seed, off) in seeds {
        let Some(at) = somewhere_within(seed, off) else {
            return Err(Error::CannotWorkOutHowBigACrossingIs);
        };
        match which_side_of_what_holds_two_loops_apart(held_apart_by, at, policy, log) {
            Decision::Zero => return Ok(seed),
            side @ (Decision::Pos | Decision::Neg) if side == the_corners_side => {
                return Ok(seed);
            }
            Decision::Pos | Decision::Neg => {}
            Decision::Undecidable { .. } => {
                return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
            }
            other => panic!("a side of a flat surface came back as {other:?}"),
        }
    }
    Err(Error::CannotNameWhichPieceOfACrossingThisIs)
}

/// Which side of the flat surface that holds a degree-four crossing's two
/// loops apart the box `at` is on, spending the policy's budget on the answer
/// when nothing could tell and recording it, as [`which_side_of`] does.
///
/// The sign of `w·(p − c)` for a ball centred at `c` and a pipe along `w`, and
/// of `a·(p − o₂)` for two pipes, with `a = w₁(w₂·w₂) − w₂(w₁·w₂)` the thin
/// pipe's way across the fat one's axis and `o₂` the fat axis's origin. Both
/// are asked of the numbers the surfaces store, so nothing rounded stands
/// between the surface and the answer, and exactly where `at` is exact.
fn which_side_of_what_holds_two_loops_apart(
    held_apart_by: TwoLoopsHeldApartBy,
    at: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Decision {
    let near = at.enclosure();
    let exact_at = at.exact_coordinates();
    let from = |here: [f64; 3]| -> [Interval; 3] {
        core::array::from_fn(|axis| near[axis] - Interval::exactly(here[axis]))
    };
    let (named, (decision, level)) = match held_apart_by {
        TwoLoopsHeldApartBy::ABallsCentre { ball, pipe } => {
            let centre = ball.centre().coordinates();
            let along = pipe.axis().direction().components();
            let question = Question {
                named: WHERE_A_BALL_AND_A_PIPE_CROSS,
                enclosing: || dot_enclosing(exactly(along), from(centre)),
                exactly: || {
                    Exact::dot_of_two_exact(
                        &Exact::of_each(along),
                        &Exact::gap_between(centre, exact_at?),
                    )
                },
            };
            (question.named, decide(&question))
        }
        TwoLoopsHeldApartBy::AFatPipesAxis { thin, fat } => {
            let origin = fat.axis().origin().coordinates();
            let [thin_way, fat_way] = [thin, fat].map(|pipe| pipe.axis().direction().components());
            let question = Question {
                named: WHERE_TWO_PIPES_CROSS,
                enclosing: || {
                    let squared_fat = squared_length_enclosing(exactly(fat_way));
                    let leaning = dot_enclosing(exactly(thin_way), exactly(fat_way));
                    let across = core::array::from_fn(|axis| {
                        Interval::exactly(thin_way[axis]) * squared_fat
                            - Interval::exactly(fat_way[axis]) * leaning
                    });
                    dot_enclosing(across, from(origin))
                },
                exactly: || {
                    let squared_fat = Exact::dot_of(fat_way, fat_way)?;
                    let leaning = Exact::dot_of(thin_way, fat_way)?;
                    let across = |axis: usize| {
                        Some(
                            Exact::of(thin_way[axis])
                                .times(&squared_fat)?
                                .minus(&Exact::of(fat_way[axis]).times(&leaning)?),
                        )
                    };
                    Exact::dot_of_two_exact(
                        &[across(0)?, across(1)?, across(2)?],
                        &Exact::gap_between(origin, exact_at?),
                    )
                },
            };
            (question.named, decide(&question))
        }
    };
    match decision {
        Decision::Pos | Decision::Zero | Decision::Neg => decision,
        Decision::Undecidable { enclosure } => resolve_under_policy(
            Record::new(named, &[at], level, enclosure, Measures::Length),
            policy,
            log,
        ),
        other => panic!("a side of a flat surface came back as {other:?}"),
    }
}

/// A run that goes on for ever and bends, walked over the stretch of it this
/// module chose.
///
/// # How far along an endless run to walk is a choice, and here is the argument
///
/// A parabola and a hyperbola branch have no ends, so *something* has to say
/// where the drawing stops. Two families of answer were weighed.
///
/// **(a) Let the topology say** — and the cheapest member of that family is
/// already here. [`NamedRun::new`] is `pub` and takes two ends, and the walk
/// below reads them: `ends[0].unwrap_or(-out)`. A fuse that cuts a face along a
/// parabola and knows both places the face's own wire reaches names them, and
/// nothing in this module chooses anything. **So (a) is not rejected, and the
/// rule below settles only the case (a) leaves open: a run that names no end on
/// a side.** There is no corner there for anything to have worked out.
///
/// The *expensive* member of the family — a new [`EdgeRun`] variant carrying its
/// own stretch as two numbers — is the one that is not taken, and only one of
/// the three costs once charged against the whole family really is a cost:
///
/// - It has no consumer. No fuse in the tree cuts a face along a cone's
///   parabola; a frustum against a block refuses at *ordering* with
///   `ACrossingThisCannotBeOrderedAlong` in 90 of 96 pairs, which is upstream of
///   any walk. A variant with no consumer is what §13 defers.
///
/// The other two were charged against the wrong thing. Growing an arm on a
/// `#[non_exhaustive]` enum (§40) and taking geometry from a caller (§27.5) are
/// prices of the *variant*, not of letting the topology say where a run stops —
/// which two `PredicatePoint`s on a run already do, through a public
/// constructor, with neither price paid.
///
/// **(b) Pick a stretch from the two surfaces alone** for the ends nothing
/// names, which is what is done.
/// The rule is one sentence: **walk out until the curve has bent one of its own
/// characteristic lengths away from its vertex, each way.** For a parabola
/// `vertex + across·s + bend·s²` that is `|s| ≤ 1/‖bend‖`, where the `bend·s²`
/// term is exactly `1/‖bend‖` long; for a hyperbola branch it is
/// `cosh σ ≤ 2`, where the curve has moved one transverse semi-axis past its
/// vertex.
///
/// Two properties earn it, and a third rules out the obvious alternative:
///
/// - **It depends on the two surfaces and on nothing else**, so `walk(A, B)` and
///   `walk(B, A)` agree, which is what §34 and §31.3 need.
/// - **It is translation-invariant and scales with the model.** Moving the plane
///   and the cone together does not change a single point of what comes back
///   relative to them, and doubling both doubles the stretch. That is §19: where
///   a shape sits is not allowed to change how it is cut.
/// - **The 20-minute spike's `√(size/‖bend‖)` fails the second of those**, and
///   that is why it is not the rule. `size` is
///   [`how_big_this_is`], which reads the *coordinates* of the vertex and the
///   apex — so the same parabola written a thousand units further out gets a
///   longer stretch and different points.
///   `moving_a_cone_and_its_plane_together_moves_the_walked_parabola_and_nothing_else`
///   is the demonstration.
///
/// **What (b) is not.** It is not a claim that this stretch is the right one for
/// any particular consumer; it is a claim that it is a *function of the two
/// surfaces*, which is the only property anything downstream relies on. A
/// consumer that knows where the run stops names its ends and never reaches
/// this rule at all.
trait AnEndlessBend {
    /// The point at `along` — a length for a parabola, an angle-like number for
    /// a hyperbola branch.
    fn at(&self, along: f64) -> Point;
    /// Which `along` `point` sits at, so a run that names an end is walked
    /// between the ends it names.
    fn along_of(&self, point: Point) -> f64;
    /// How far either way from the vertex this is walked when the run names no
    /// end on that side. See the note above.
    fn how_far_out_it_is_walked(&self) -> f64;
    /// How far the curve stands off its own chord over a step of `1`, so that a
    /// step of `Δ` anywhere between `from` and `to` stands off by at most this
    /// times `Δ²`.
    ///
    /// **The stretch is an argument because a hyperbola's bend is not
    /// constant.** A run that names its own ends can reach further out than
    /// [`AnEndlessBend::how_far_out_it_is_walked`] would, and a curvature read
    /// at the wrong end of the stretch is a step count that is too small.
    fn how_far_it_stands_off_a_chord(&self, from: f64, to: f64) -> f64;
    /// The largest magnitude anything about it was worked out from, which is
    /// what [`POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF`] is a multiple of.
    fn the_size_it_was_worked_out_from(&self) -> f64;
}

/// Points along one endless bend, over the stretch [`AnEndlessBend`] argues for.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when the bend's own size does not
/// come out finite, [`Error::CannotPlaceAPointThatCloseToACrossing`] when no
/// point can be put as close to the curve as `deflection` asks, and
/// [`Error::TooManyPointsAlongOneCrossing`] when it asks for more than
/// [`MOST_POINTS_ALONG_ONE_CROSSING`].
fn walk_an_endless_bend(
    bend: &impl AnEndlessBend,
    run: &NamedRun,
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<PointsAlongACrossing> {
    let deviation = POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF * bend.the_size_it_was_worked_out_from();
    if !deviation.is_finite() {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    }
    let tag = Approximation::new(deviation, Cause::PointAlongACrossing);
    let Some(room) = room_left_over(deflection, deviation, policy, log)? else {
        return Ok(PointsAlongACrossing {
            points: Vec::new(),
            deviation: tag,
        });
    };

    let out = bend.how_far_out_it_is_walked();
    let ends = run.ends().map(|end| {
        (end != nowhere_in_particular())
            .then(|| bend.along_of(Point::from(end.enclosure().map(middle_of))))
    });
    let from = ends[0].unwrap_or(-out);
    let to = ends[1].unwrap_or(out);
    let steps = how_many_pieces_an_endless_bend_takes(
        bend.how_far_it_stands_off_a_chord(from, to),
        to - from,
        room,
    )?;

    // The two ends of the stretch are kept when the run names no corner there,
    // because nothing else knows where the walk was cut off. See [`walk`].
    let first = usize::from(ends[0].is_some());
    let last = steps - usize::from(ends[1].is_some());
    let points = (first..=last)
        .map(|step| bend.at(from + (to - from) * (step as f64) / (steps as f64)))
        .collect();

    Ok(PointsAlongACrossing {
        points,
        deviation: tag,
    })
}

/// How many straight pieces a stretch of an endless bend has to be walked into.
///
/// **Derived exactly rather than bounded**, which is what a bend with a constant
/// or monotone second derivative allows and [`how_many_steps`]'s
/// `reach · (1 − cos(step / 2))` does not. A curve stands off the chord of a
/// step of `Δ` by at most `‖p″‖ · Δ² / 8`, and for a parabola that is an
/// equality with `‖p″‖ = 2‖bend‖`. So the widest step is `√(room / that)`, and
/// there is no turning limit to add: an endless bend never comes back round.
///
/// **The floor is the same floor, and it was the same defect.** A tolerant
/// caller with room to spare got one piece here too, and one piece of a
/// parabola is the chord across the whole stretch — with the two extreme points
/// kept, the walk handed back a straight line standing in for a bend. See
/// [`FEWEST_STEPS_THAT_SHOW_A_BEND`], whose `1 − 1/n²` is an equality for this
/// shape rather than a limit.
///
/// **An arc walked round a cone is counted here too**, with a bound on `‖p″‖`
/// in place of its value; `AnArcRoundACone` derives the bound.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when the count is not a number,
/// which is the same refusal and the same reason [`how_many_steps`] gives it,
/// and [`Error::TooManyPointsAlongOneCrossing`] past
/// [`MOST_POINTS_ALONG_ONE_CROSSING`].
fn how_many_pieces_an_endless_bend_takes(
    stands_off_a_chord: f64,
    over: f64,
    room: f64,
) -> Result<usize> {
    let widest = (room / stands_off_a_chord).sqrt();
    let steps = (over.abs() / widest.max(f64::MIN_POSITIVE)).ceil();
    if steps.is_nan() {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    }
    if steps > MOST_POINTS_ALONG_ONE_CROSSING as f64 {
        return Err(Error::TooManyPointsAlongOneCrossing {
            most: MOST_POINTS_ALONG_ONE_CROSSING,
        });
    }
    Ok(steps.max(FEWEST_STEPS_THAT_SHOW_A_BEND as f64) as usize)
}

/// The fewest pieces any curved run is walked into, whatever the deflection and
/// however little the run bends.
///
/// **Two, and the reason is a cliff and not a taste.** Walk a curve into `n`
/// equal pieces and the polyline stands off the run's own chord over a fraction
/// of the area the curve stands off it by; for a parabola that fraction is
/// exactly `1 − 1/n²`, since the area between a parabola and a chord over a span
/// of `D` is `‖bend‖·D³/6` and `n` pieces leave `n` slivers of `‖bend‖(D/n)³/6`.
/// For a circular arc turning by `θ` it is
/// `(n·sin(θ/n) − sin θ) / (θ − sin θ)`, which is `1 − 1/n²` as the pieces get
/// short and never below `2/π = 0.6366`, which it is exactly at `n = 2`,
/// `θ = π`.
///
/// **That floor needs two restrictions and not one.** [`how_many_steps`] keeps
/// a piece under a quarter turn, so `θ/n ≤ π/2`; and it walks a conic, which
/// closes, so `θ ≤ 2π`. Drop the second and the floor is false — at `n = 5`,
/// `θ = 5π/2` the pieces are exactly quarter turns and the fraction is `0.5836`.
/// Under both, `2/π` is the infimum over every `n ≥ 2` and every `θ`.
///
/// So the sequence is `0`, about `3/4`, about `8/9`. **One piece shows none of
/// the bend, for every `θ`, at every deflection** — the run comes back as its
/// own chord, which is what [`EdgeRun::Straight`] already comes back as, and a
/// caller cannot tell a curve from a straight line. Two is the first count that
/// is not that, and it is where the floor goes because that is where the cliff
/// is: nought to three quarters. Three quarters to eight ninths is *refinement*,
/// and refinement is the question `room` already asks — charging every walk for
/// it would be spending points on a bend nobody asked to see more of.
///
/// A quarter arc is what made this visible. `by_turning` allows a whole quarter
/// in one step and `by_bending` asks for nothing at all when the caller has
/// room to spare, so at
/// `rcad-topo`'s `as_coarse_as_a_mesh_gets` a quarter arc came back with no
/// point between its ends at all, and a flat face bounded by that arc and its
/// chord fanned into two corners rather than a piece of disc.
const FEWEST_STEPS_THAT_SHOW_A_BEND: usize = 2;

/// A parabola: `vertex + across·s + bend·s²`, with `across` of length one and
/// square to `bend`.
///
/// **[`Conic`] cannot hold one**, and that is why this type exists rather than a
/// flag on that one. A conic is a centre and two semi-axes walked round in an
/// angle; a parabola has no centre, no semi-axis and no angle to go round.
#[derive(Clone, Copy, Debug)]
struct Parabola {
    vertex: Point,
    across: Vector,
    bend: Vector,
    size: f64,
}

impl AnEndlessBend for Parabola {
    fn at(&self, along: f64) -> Point {
        self.vertex + self.across * along + self.bend * (along * along)
    }

    fn along_of(&self, point: Point) -> f64 {
        (point - self.vertex).dot(self.across)
    }

    fn how_far_out_it_is_walked(&self) -> f64 {
        self.bend.length().recip()
    }

    /// An **equality** and not a bound: a quadratic's second derivative is the
    /// same everywhere, so the stretch does not come into it and the furthest a
    /// step of `Δ` gets from its own chord is exactly `‖bend‖Δ²/4`.
    fn how_far_it_stands_off_a_chord(&self, _from: f64, _to: f64) -> f64 {
        self.bend.length() / 4.0
    }

    fn the_size_it_was_worked_out_from(&self) -> f64 {
        self.size
    }
}

/// One branch of a hyperbola: `centre + out·cosh σ + across·sinh σ`, with `out`
/// the transverse semi-axis pointing the branch's own way and `across` the
/// conjugate one.
///
/// **A hyperbola has a centre and two axes, so it very nearly is a [`Conic`] —
/// and it still is not one.** What differs is not the numbers but the two
/// functions the parameter goes through: `cos`/`sin` close the curve after a
/// turn and `cosh`/`sinh` run off to infinity, and everything [`Conic`] does
/// with an angle — `angle_of` on `atan2`'s branch, the two arcs between two
/// ends, `from + τ` for a whole turn — is about a curve that closes. Sharing the
/// four fields and branching on a flag inside `at` would put a curve with no
/// turn through code whose every other method assumes one, so the fields are
/// shared by being written twice and the methods are not shared at all.
#[derive(Clone, Copy, Debug)]
struct HyperbolaBranch {
    centre: Point,
    out: Vector,
    across: Vector,
    size: f64,
}

impl AnEndlessBend for HyperbolaBranch {
    fn at(&self, along: f64) -> Point {
        self.centre + self.out * along.cosh() + self.across * along.sinh()
    }

    fn along_of(&self, point: Point) -> f64 {
        ((point - self.centre).dot(self.across) / self.across.length_squared()).asinh()
    }

    /// Out to `cosh σ = 2`: one transverse semi-axis past the vertex, which is
    /// the hyperbola's reading of [`AnEndlessBend`]'s one sentence.
    fn how_far_out_it_is_walked(&self) -> f64 {
        2.0f64.acosh()
    }

    /// `p″(σ)` is `p(σ) − centre`, whose length grows with `|σ|`, so the widest
    /// it gets is at whichever end of the stretch is further from the vertex.
    fn how_far_it_stands_off_a_chord(&self, from: f64, to: f64) -> f64 {
        let furthest_out = from.abs().max(to.abs());
        let furthest = self.out * furthest_out.cosh() + self.across * furthest_out.sinh();
        furthest.length() / 8.0
    }

    fn the_size_it_was_worked_out_from(&self) -> f64 {
        self.size
    }
}

/// The two places a ball reaches furthest along a pipe's axis, one on each of
/// the two loops it can leave.
///
/// **Where the two pieces are pulled apart matters.** A ball and a pipe leave
/// two loops either side of the plane through the centre square to the axis, so
/// the two seeds are the far ends of the same reach along that axis.
///
/// Both are on the ray from the axis out towards the ball's centre, where the
/// crossing runs furthest from the plane through the centre square to the axis.
/// A ball centred exactly on the axis has no such ray and any direction across
/// the pipe will do, so the pipe's own basis stands in.
fn where_a_ball_reaches_across_a_pipe(ball: &Sphere, pipe: &Cylinder) -> [Point; 2] {
    let along = pipe.axis().direction() * pipe.axis().direction().length().recip();
    let skin = pipe.radius();
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

/// Whether a ball's centre is on a pipe's axis, **proved and not rounded**: the
/// sign of `‖(c − o) × w‖²`, which is the question
/// [`intersect`](crate::intersect) splits the pair on before it names two
/// circles or two quartic loops, and the one `rcad-topo`'s arrangement asks
/// before it orders round a circle.
///
/// **No [`Policy`]**, for the reason [`whether_a_cone_and_a_pipe_share_an_axis`]
/// gives: a `Zero` nobody earned would draw two circles where the two cross in
/// a quartic. Anything but `Zero` is marched, as every ball and pipe was before.
fn whether_a_ball_sits_on_a_pipes_axis(ball: &Sphere, pipe: &Cylinder) -> Decision {
    let centre = ball.centre().coordinates();
    let origin = pipe.axis().origin().coordinates();
    let along = pipe.axis().direction().components();
    decide(&Question {
        named: WHERE_A_BALL_AND_A_PIPE_CROSS,
        enclosing: || {
            squared_length_enclosing(cross_enclosing(
                gap_enclosing(origin, centre),
                exactly(along),
            ))
        },
        exactly: || {
            Exact::squared_length_of(&Exact::cross_of(
                &Exact::gap_between(origin, centre),
                along,
            )?)
        },
    })
    .0
}

/// The circle of the crossing of a pipe and a ball centred on its axis and
/// wider than it that the run names.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] as
/// [`which_of_these_circles_this_run_is`] gives them.
fn where_a_ball_on_a_pipes_axis_cuts_it(
    ball: &Sphere,
    pipe: &Cylinder,
    ball_first: bool,
    run: &NamedRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Conic> {
    which_of_these_circles_this_run_is(
        the_two_circles_a_ball_on_a_pipes_axis_cuts(ball, pipe, ball_first),
        run,
        policy,
        log,
    )
}

/// The two circles a ball centred on a pipe's axis and wider than it crosses
/// it in, `√(R² − r²)` either way along the axis from the centre, each of the
/// pipe's radius. `R² − r²` is written `(R − r)(R + r)`, which does not cancel.
///
/// # Which way round each goes
///
/// Each starts on the pipe's own first direction across, where the marcher's
/// seed for it was, and runs the way `along_a_crossing` orders it: the crossing
/// of `S` and `T` runs along `∇S × ∇T`, which for the ball first is
/// anticlockwise about the axis on the circle ahead of the centre and about its
/// opposite on the circle behind (`the_way_a_circle_round_a_pipes_axis_winds`
/// derives it). The pipe first turns both round.
fn the_two_circles_a_ball_on_a_pipes_axis_cuts(
    ball: &Sphere,
    pipe: &Cylinder,
    ball_first: bool,
) -> Vec<Conic> {
    let radius = ball.radius();
    let skin = pipe.radius();
    let reach = ((radius - skin) * (radius + skin)).sqrt();
    let along = pipe.axis().direction();
    let up = along * along.length().recip();
    let (across, and) = pipe.basis();
    [reach, -reach]
        .into_iter()
        .map(|reach| {
            let middle = ball.centre() + up * reach;
            let about_the_axis = (reach > 0.0) == ball_first;
            let and = if about_the_axis { and } else { -and };
            Conic {
                centre: middle,
                across: across * skin,
                and: and * skin,
                size: how_big_this_is(&[middle, ball.centre()], &[skin, radius, reach]),
            }
        })
        .collect()
}

/// Whether two pipes that do not point one way are the same width **and** have
/// axes that meet, which is the one pair of pipes whose crossing is flat.
///
/// Both halves are asked as certified sign questions rather than compared
/// against a tolerance, because this family is exactly degenerate and not nearly
/// so: the radii of a countersunk bore written out by a CAD system come from one
/// literal, and two placements that share a coordinate put their axes in one
/// plane to the bit. Asking it any other way would answer `true` for a pair that
/// is merely close, and a pair that is merely close crosses in a quartic.
///
/// A pipe is `rᵢ` wide whatever its stored direction's length turned out to be
/// (`docs/decisions.md` §118.11), so the two widths match exactly when
/// `r₂² − r₁²` is zero — degree two in the radii and degree zero in the
/// directions. It used to be
/// `r₂²‖w₁‖² − r₁²‖w₂‖²`, degree four, because the surface the predicate
/// certified was then the pipe of radius `rᵢ/‖wᵢ‖` and the stored lengths had to
/// be carried along. That form answered `false` for a bore and a countersink
/// written from one radius literal whenever their two directions rounded to
/// different lengths, which is what the two Finder relays in the STEP corpus
/// refused on. The axes meet exactly when `(o₂ − o₁)·(w₁ × w₂)` is zero, which
/// is the numerator of the distance between them and degree three. Neither is
/// the eight-degree comparison [`intersect`](crate::intersect) runs out of room
/// on (§29), because neither is a sum of roots: each is one polynomial against
/// zero.
///
/// Both questions are symmetric under swapping the pair — swapping negates the
/// volume and the radius difference and leaves what is being asked, which is
/// whether they are zero — so this may be asked before the caller has settled
/// which pipe is which.
///
/// # Errors
///
/// [`Error::Undecidable`] when the policy will not certify either sign.
fn whether_two_pipes_are_one_width_on_axes_that_meet(
    one: &Cylinder,
    other: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    let at = [one, other].map(|pipe| PredicatePoint::from(pipe.axis().origin().coordinates()));
    let [here, there] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());
    let [my_radius, their_radius] = [one, other].map(Cylinder::radius);

    let same_width = Question {
        named: WHERE_TWO_PIPES_CROSS,
        enclosing: || {
            Interval::exactly(their_radius) * Interval::exactly(their_radius)
                - Interval::exactly(my_radius) * Interval::exactly(my_radius)
        },
        exactly: || {
            Some(
                Exact::product_of(their_radius, their_radius)?
                    .minus(&Exact::product_of(my_radius, my_radius)?),
            )
        },
    };
    if ask(&same_width, Measures::Area, &at, policy, log)? != Decision::Zero {
        return Ok(false);
    }

    let axes_meet = Question {
        named: WHERE_TWO_PIPES_CROSS,
        enclosing: || {
            let sideways = [0, 1, 2].map(|axis| {
                let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
                Interval::exactly(my_way[next]) * Interval::exactly(their_way[after])
                    - Interval::exactly(my_way[after]) * Interval::exactly(their_way[next])
            });
            (0..3).fold(Interval::exactly(0.0), |so_far, axis| {
                so_far
                    + (Interval::exactly(there[axis]) - Interval::exactly(here[axis]))
                        * sideways[axis]
            })
        },
        exactly: || {
            let sideways = Exact::cross_of(&Exact::of_each(my_way), their_way)?;
            Exact::dot_of_two_exact(&Exact::gap_between(here, there), &sideways)
        },
    };
    Ok(ask(&axes_meet, Measures::Volume, &at, policy, log)? == Decision::Zero)
}

/// The ellipse two pipes of one width on axes that meet cross in, picked out of
/// the two of them by the run's own ends and half-spaces.
///
/// # Two planes, at any crossing angle
///
/// Written about the point where the axes meet, with `ê` and `û` the two unit
/// directions and `r` the radius both share, the two pipes are
/// `‖p‖² − (p·ê)² = r²` and `‖p‖² − (p·û)² = r²`. Subtracting leaves
/// `(p·(û − ê))·(p·(û + ê)) = 0`, so every point of the crossing is on one of
/// the two bisector planes, whatever the angle between the axes. Each plane cuts
/// a pipe in an ellipse: `r` across, and `r` over the sine of the angle the
/// plane's own in-plane direction makes with the axis the long way.
///
/// **The pair is sorted before the ellipses are built**, for the reason the two
/// balls are (§34): `û − ê` turns into `ê − û` when the caller writes the pair
/// the other way round, and an ellipse whose long axis vector has flipped starts
/// somewhere else, which moves the vertex `where_a_closed_crossing_starts` puts
/// on the loop.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when an ellipse's own numbers do
/// not come out finite, and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when what is on the run
/// holds both ellipses or neither.
fn where_two_equal_pipes_on_crossing_axes_cut(
    one: &Cylinder,
    other: &Cylinder,
    run: &NamedRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Conic> {
    let (first, second) = if Surface::Cylinder(*one)
        .how_it_sorts_against(&Surface::Cylinder(*other))
        == Ordering::Greater
    {
        (other, one)
    } else {
        (one, other)
    };
    which_of_these_circles_this_run_is(
        the_two_ellipses_equal_pipes_on_crossing_axes_cut(first, second),
        run,
        policy,
        log,
    )
}

/// The two ellipses, the one whose plane the axes' sum lies in first.
fn the_two_ellipses_equal_pipes_on_crossing_axes_cut(
    first: &Cylinder,
    second: &Cylinder,
) -> Vec<Conic> {
    let [mine, theirs] = [first, second].map(|pipe| {
        let along = pipe.axis().direction();
        along * along.length().recip()
    });
    let radius = first.radius();
    let sideways = mine.cross(theirs);
    let across = sideways * sideways.length().recip();
    let middle = where_two_axes_meet(first, second);

    [theirs + mine, theirs - mine]
        .into_iter()
        .map(|bisector| {
            let the_long_way = bisector * bisector.length().recip();
            // The ellipse is as long as the pipe is wide over the sine of the
            // angle the plane's own long direction leans away from the axis,
            // and for two unit vectors that sine is the length of their cross.
            let reach = radius / the_long_way.cross(mine).length();
            Conic {
                centre: middle,
                across: the_long_way * reach,
                and: across * radius,
                size: how_big_this_is(&[middle], &[reach, radius]),
            }
        })
        .collect()
}

/// Where two axes that are known to meet do meet.
///
/// The foot of the common perpendicular on the first axis, which is the
/// crossing point itself once the two are known to reach each other.
/// [`intersect`](crate::intersect) asks of the same point that it is exactly on
/// both axes before it names the four halves.
pub(crate) fn where_two_axes_meet(first: &Cylinder, second: &Cylinder) -> Point {
    let [mine, theirs] = [first, second].map(|pipe| pipe.axis().direction());
    let sideways = mine.cross(theirs);
    let between = second.axis().origin() - first.axis().origin();
    first.axis().origin() + mine * (between.cross(theirs).dot(sideways) / sideways.length_squared())
}

/// The two places one pipe reaches furthest along its own axis through another,
/// one on each of the two loops the pair can leave.
///
/// `thin` is walked round to the angle where the other pipe reaches furthest,
/// and the two roots of the quadratic in how far along its axis the point sits
/// are the two seeds. Where the pair leaves only one loop both roots are on it.
///
/// **Which of the two is walked round matters**, which is why the caller sorts
/// them. Two pipes leave two loops either side of the plane the naming half
/// picked them apart with, and reaching both of them means walking round the
/// thinner: on the fatter one the same two loops show up as two arcs of the
/// same sweep, and a pair of seeds taken there would both land on one of them.
fn where_one_pipe_reaches_across_another(thin: &Cylinder, fat: &Cylinder) -> [Point; 2] {
    let [thin_way, fat_way] = [thin, fat].map(|pipe| {
        let along = pipe.axis().direction();
        along * along.length().recip()
    });
    let [thin_skin, fat_skin] = [thin, fat].map(Cylinder::radius);
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
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when `at` is not a point, so there
/// is no box round it to ask about. A conic that is a curve at all
/// ([`Conic::has_an_angle_round_it`]) places every corner in `f64`'s range
/// except by overflowing it, which is what that name already covers.
fn every_half_space_holds(
    picked_by: &[Picker],
    at: Point,
    deviation: f64,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    let Some(at) = somewhere_within(at, deviation) else {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    };
    let mut on_or_outside = true;
    for picker in picked_by {
        if which_side_of(picker, at, policy, log) == Decision::Neg {
            on_or_outside = false;
        }
    }
    Ok(on_or_outside)
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
/// same surface and the run carries the second one itself — and except where
/// both faces are flat and the run names a curved surface.
///
/// **Two flat faces are a seam in all but their bits.** A circle a file
/// imprinted on a flat face has that face's plane on both sides, written twice
/// and so not always equal by value: two planes 1.2e-15 apart on
/// `kicad/LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step` (`docs/decisions.md`
/// §176.2). Two planes cross in a straight line or nowhere, so a run between
/// them that names a curved surface can only mean that surface.
///
/// **Two curved faces that name a plane are a face and that plane** — the face
/// [`Surface`]'s own fixed order puts first, which is the ball if one of them is
/// a ball (see below for why that order). Where a ball meets a
/// pipe of its own radius centred on its axis the two touch all the way round a
/// circle and never cross, so the marcher has no crossing to follow, and as
/// written down the ball's centre is `2.1e-16` off the pipe's axis, so no closed
/// form certified by exact signs can place the circle either
/// (`kicad/LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step`, `docs/decisions.md`
/// §189.3). A pipe, a cone or a torus written twice a hair apart is the same
/// failure one kind over: the two spellings cross nowhere near the rim the run's
/// plane names, and the pair either side loses it (§220.4). The run's plane is
/// the circle's own, and it crosses the face in that circle, which is drawn
/// directly.
///
/// **This trusts the plane, and the trust is earned upstream, not here.** A
/// plane crosses a torus in up to two circles and a slanted pipe in an ellipse,
/// so a plane that is not the curve's own would hand back a curve nobody drew.
/// Since Q1118, the kernel's own fuse can be where the name comes from:
/// `rcad-topo`'s `the_plane_of_a_rim_between_two_curved_faces` puts the cap's
/// plane in `and` for a closed rim between two different curved faces — the
/// dome's pipe and ball, a frustum's cone and a bar's pipe, two cones base to
/// base — once `meet_of_three_surfaces` says the plane and the two curved
/// surfaces share more than a point, and that check is the trust earned there.
/// Everywhere else the name is a reader's, and `rcad-step` writes a plane
/// there only for a circle it has checked lies on every face along it, and one
/// of the two faces everywhere else — every ellipse, and every arc along a
/// cone or a torus. Moving either check moves this arm's premise.
///
/// **Two curved faces that name a third curved surface are a face and that
/// surface.** This is the paragraph above one surface up, and it is the one
/// `along[0] == along[1]` cannot reach: two balls meant to be the one ball but
/// written down 1e-15 apart are not equal by bits, so the seam arm misses and
/// the pair either side goes to `where_two_balls_cut` — which answers finitely
/// and **wrongly**, with a great circle of the ball's own radius through its own
/// centre, in the plane halfway between the two spellings. Measured at a rim
/// 2.05 to 3.65 away from a centre it should be exactly 4 from
/// (`two_spheres_a_hair_apart_are_one_sphere_and_a_second_surface.rs`,
/// `docs/decisions.md` §218.6).
///
/// **`and` repeating one of the two faces is what keeps the honest pair.** A
/// run whose two faces really do cross names both of them already, so
/// `rcad-topo`'s `the_surface_that_names_it_twice` puts one of the pair in `and`
/// — bit for bit, it is one of the two [`Surface`]s that came in. That is the
/// writer saying "`and` adds nothing here", and it is why this asks whether
/// `and` is a third surface rather than whether the two faces are one.
///
/// **That holds for a closed run too, and it is asked first for both kinds.**
/// A closed run used to take `[along[0], and]` whatever was either side, so
/// `and` equal to `along[0]` came back as one surface twice — a pair that names
/// no crossing at all — while the same edge with its faces listed the other way
/// round came back as the honest pair. Which face a caller happens to list first
/// is not a fact about the edge, so the answer may not depend on it; the pair
/// either side is what both orders agree on (`docs/decisions.md` §23.8, §221.5).
/// Nothing writes that run today — the fuse puts a cutting surface in `and` only
/// where both faces are one surface and the cutter is not it, and `rcad-step`
/// only a surface that is neither face — so this is the answer for a model
/// built by hand or read back from bytes.
///
/// # Which of two curved faces goes with a third surface
///
/// **The one [`Surface::how_it_sorts_against`] puts first, so the answer does
/// not depend on which face is listed first.** It used to be the first face
/// listed, or the ball, and callers list the two faces in the order they meet
/// them: `Model::validate` puts first the face whose coedge it is checking. So
/// the two coedges of one rim between a pipe and a cone were checked against
/// two different curves, and a bar that flares out into a cone, with its pipe's
/// axis written pointing down, was refused whichever way round its file listed
/// the faces; the same rim measured `6π` from one face and was refused a length
/// from the other; and a half ball on a pipe of its own radius meshed from one
/// face and not the other (`docs/decisions.md` §222.1 item 3, §336).
///
/// That order puts a ball before a pipe, a pipe before a cone and a cone before
/// a torus. The ball first is what gets its poles into the rim, because the
/// mesher reads this pair to find them; the pipe before a cone or a torus is
/// the pair `rcad-topo`'s edge length has a circle for; and it is the surface
/// `rcad-topo`'s and `rcad-step`'s `the_curved_one_of` pick to say which way a
/// closed edge runs, so the curve an edge is walked along and the surface that
/// says which way it goes are one surface.
///
/// **Where a face is flat, the first face listed still stands.** Two flat faces
/// a hair apart with a curved `and` are one plane written twice, and
/// `one_answer_for_which_two_surfaces_a_run_is_between.rs` pins the first of
/// them; a flat and a curved face beside a third surface are something nothing
/// in the kernel or the reader writes. Both still depend on the order.
///
/// # One answer, read by everyone who asks
///
/// [`walk`] and [`where_a_closed_crossing_starts`] read the pair off this, and
/// `rcad-topo`'s `Model::validate` and mesher read it off this too, so that the
/// curve a closed edge is checked for winding along is the curve that was
/// walked. Three copies of this question stood in three places until §221.5,
/// and two of them had drifted from it: which is the shape §23.8 kept finding.
///
/// Asking it twice changes nothing: every pair it hands back names `and` as one
/// of its two surfaces, or is the pair either side where `and` said nothing, so
/// handing the answer back in gives the same answer again.
pub fn which_two_surfaces_a_run_is_between(run: &EdgeRun, along: [&Surface; 2]) -> [Surface; 2] {
    let both_flat = along.iter().all(|side| side.as_plane().is_some());
    let both_curved = along.iter().all(|side| side.as_plane().is_none());
    match run {
        EdgeRun::RightRoundWhereItCrosses { and } | EdgeRun::PartOfWhereItCrosses { and, .. }
            if and == along[0] || and == along[1] =>
        {
            [*along[0], *along[1]]
        }
        // Two curved faces and an `and` that is neither of them — the arm above
        // took the ones that repeat a face — whether `and` is the circle's own
        // plane or a third curved surface, and whether the two faces are one
        // surface or two: the run names a third surface, which is what a reader
        // writes only when the pair either side names nothing.
        EdgeRun::RightRoundWhereItCrosses { and } | EdgeRun::PartOfWhereItCrosses { and, .. }
            if both_curved =>
        {
            [the_face_that_sorts_first(along), *and]
        }
        // A closed run whose two faces are one surface names its second surface
        // the same way a ball's seam does, and for the same reason: the pair
        // either side is one surface twice and says nothing at all. It needs no
        // `along[0] == along[1]` guard because it carries no `picked_by` to fall
        // back on — the edge is the whole of the closed curve.
        EdgeRun::RightRoundWhereItCrosses { and } => [*along[0], *and],
        EdgeRun::PartOfWhereItCrosses { and, .. } if along[0] == along[1] => [*along[0], *and],
        EdgeRun::PartOfWhereItCrosses { and, .. } if both_flat && and.as_plane().is_none() => {
            [*along[0], *and]
        }
        EdgeRun::Straight
        | EdgeRun::RightRoundWhereItsFacesCross
        | EdgeRun::PartOfWhereItCrosses { .. } => [*along[0], *along[1]],
    }
}

/// Whichever of the two faces [`Surface::how_it_sorts_against`] puts first.
///
/// That order depends on nothing but the numbers each surface stores, so two
/// callers that list the same two faces either way round get the same face
/// back. [`which_two_surfaces_a_run_is_between`] says why it is this order.
fn the_face_that_sorts_first(along: [&Surface; 2]) -> Surface {
    let [one, other] = along;
    match other.how_it_sorts_against(one) {
        Ordering::Less => *other,
        Ordering::Equal | Ordering::Greater => *one,
    }
}

/// How much of the deflection is left for the straight pieces to spend, or
/// `None` when a tolerant caller accepted a walk that cannot be refined at all.
///
/// # The wave-through is recorded, and what the record says
///
/// A certified caller is refused here. A tolerant one is not, and what it gets
/// back is a walk with no points between its ends — so the record is the only
/// thing that separates it from a run that needed none. The return value carries
/// the deviation as an [`Approximation`] either way, which is why the record does
/// not carry one: two signals for one event is a reader having to work out which
/// of them to believe.
///
/// **The outcome is [`Decision::Undecidable`] even though nothing was
/// undecided**, for the reason [`DynamicRange::check_under_policy`] records one:
/// this is not a predicate and has no sign to be uncertain about, and the honest
/// thing an enclosure of `deviation` says is that every point this walk hands
/// back is uncertain by that much. `Measures::Length` because `deviation` is
/// already a distance and needs no converting to be read as a wobble (§15.2).
///
/// **The record is left unresolved**, so [`Record::approximation`] is `None` and
/// no [`Cause`] is written here. [`Cause::TolerantFallback`] is what this event
/// is, and `Record::resolve_to` is private to `rcad-predicates` — see the note in
/// `a_waved_through_walk_leaves_a_record.rs`.
///
/// [`DynamicRange::check_under_policy`]: crate::DynamicRange::check_under_policy
/// [`Record::approximation`]: rcad_predicates::Record::approximation
fn room_left_over(
    deflection: Deflection,
    deviation: f64,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<f64>> {
    let room = deflection.length() - deviation;
    if room > 0.0 {
        return Ok(Some(room));
    }
    if policy.wobble_is_acceptable(deviation) {
        log.push(Record::new(
            ROOM_LEFT_TO_REFINE_A_WALK,
            &[],
            Level::Float,
            deviation,
            Measures::Length,
        ));
        return Ok(None);
    }
    Err(Error::CannotPlaceAPointThatCloseToACrossing {
        asked: deflection.length(),
        closest: deviation,
    })
}

/// How many straight pieces this stretch of the conic has to be walked into.
///
/// Two questions, and the answer is the larger, held up to a floor.
/// **How far it bends**: the arc from one corner to the next stands off its own
/// chord by at most `reach · (1 − cos(step / 2))`, so that is what has to stay
/// inside `room`. **How far it turns**: a step of more than half a turn cannot
/// be told from one the other way, so none is allowed past a quarter — which is
/// the same bound `rcad-topo`'s mesher already refuses to step past, for the
/// same reason. Neither question has anything to say about a shallow arc a
/// tolerant caller asked nothing of, and that is what
/// [`FEWEST_STEPS_THAT_SHOW_A_BEND`] is for.
///
/// # The floor goes on last, and it used to go on first
///
/// `f64::max` hands back the *other* operand when one of them is `NaN`, so
/// `by_bending.max(by_turning).max(1.0)` turned a `NaN` into a one and the
/// `is_nan` check below it could never fire. What arrived that way was a whole
/// closed cap circle walked in one step — no points at all between its ends —
/// and a flat cap whose only wire then fanned into a single corner, which
/// `rcad-topo`'s mesher asserts against. `Conic::angle_of` divides by
/// `and.length_squared()`, so a bar of radius `1e-200` has that underflow to
/// zero and hands `0.0/0.0` back as the angle it turns over.
///
/// So the floor goes on after the checks rather than before them, and a `NaN`
/// is [`Error::CannotWorkOutHowBigACrossingIs`] — which is the honest name for
/// it and already carries the classification: a **gap**, because the same two
/// surfaces written higher up the number line are walked without complaint.
fn how_many_steps(conic: &Conic, turns_over: f64, room: f64) -> Result<usize> {
    let widest = (1.0 - room / conic.how_far_it_reaches())
        .clamp(-1.0, 1.0)
        .acos()
        * 2.0;
    let by_bending = (turns_over.abs() / widest.max(f64::MIN_POSITIVE)).ceil();
    let by_turning = (turns_over.abs() / (core::f64::consts::TAU / 4.0)).ceil();
    let steps = by_bending.max(by_turning);
    if steps.is_nan() {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    }
    if steps > MOST_POINTS_ALONG_ONE_CROSSING as f64 {
        return Err(Error::TooManyPointsAlongOneCrossing {
            most: MOST_POINTS_ALONG_ONE_CROSSING,
        });
    }
    Ok(steps.max(FEWEST_STEPS_THAT_SHOW_A_BEND as f64) as usize)
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
    /// Whether there is an angle to go round this at all.
    ///
    /// A conic that shrank to a point is not a curve, and every road out of one
    /// runs through `NaN`: [`Conic::angle_of`] divides by a semi-axis's own
    /// squared length, so a semi-axis of zero hands back `0/0`, and then
    /// [`Conic::at`] builds a corner out of that angle, [`somewhere_within`]
    /// builds a box out of that corner, and the box is `NaN` on both sides.
    /// A semi-axis carrying a `NaN` already fails `> 0.0` here.
    ///
    /// **The centre is asked about too, because a size that is a number is no
    /// promise the centre is a point.** [`how_big_this_is`] folds through
    /// [`f64::max`], which drops a `NaN` rather than spreading it, so a centre of
    /// `(NaN, ∞, NaN)` — which is what `normal * ∞` makes of a normal with a zero
    /// in it — leaves a perfectly finite size and a `NaN` in every corner
    /// [`Conic::at`] places.
    ///
    /// **Only the caller knows whether a semi-axis that is not a length means
    /// "no curve" or "too big to measure"**, which is why this asks neither
    /// about infinity nor about `NaN` by name and the one call site checks the
    /// size first.
    ///
    /// **[`Vector::length`] and not `length_squared`, and that is the whole
    /// difference between this refusal and §39's.** A bar of radius `1e-200` has
    /// a cap circle whose squared semi-axis is `1e-400` and underflows to zero,
    /// and that circle *is* a curve — the same bar written larger is walked, so
    /// what ran out is `f64`'s exponent and the answer is
    /// [`Error::CannotWorkOutHowBigACrossingIs`], a gap
    /// (`a_bar_too_thin_for_its_own_squares_is_refused_rather_than_asserted_against`).
    /// [`Vector::length`] scales the squares back into range and answers
    /// `1e-200`, so asking it here tells that bar apart from a circle whose
    /// radius really is zero, which no scaling rescues.
    fn has_an_angle_round_it(&self) -> bool {
        self.centre.is_finite() && self.across.length() > 0.0 && self.and.length() > 0.0
    }

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

    /// How far `point` sits off this conic, measured in its own plane and
    /// across it.
    ///
    /// Two candidates that lie in one plane and differ only in size are told
    /// apart by the first term; the second is what says a point is on the plane
    /// at all.
    ///
    /// **A circle takes the exact distance and anything else takes a first-order
    /// one**, because there is no closed form for how far a point sits off an
    /// ellipse. `(x/a)² + (y/b)² − 1` divided by the length of its own gradient
    /// is that distance to first order, which is all a *test* of being on the
    /// curve needs: a point on it reads zero either way, and a point off it
    /// reads something of the right size. The two agree in the limit for a
    /// circle, and the circle arm is kept anyway so that the torus and the
    /// coaxial cone — the two callers that only ever ask about circles, and
    /// whose answers are compared against a bound a few ulps wide — get the same
    /// bits they always did.
    ///
    /// A point at the centre of an ellipse has no gradient to divide by; it
    /// comes back as the whole of the reach, which is honest and is the branch
    /// `a_point_at_an_ellipses_own_centre_is_not_on_it` pins. A circle at its own
    /// centre already read the same way.
    fn how_far_off_it(&self, point: Point) -> f64 {
        let out = point - self.centre;
        let reach = self.across.length();
        let other_way = self.and.length();
        let across = out.dot(self.across) / reach;
        let and = out.dot(self.and) / other_way;
        let up = out - self.across * (across / reach) - self.and * (and / other_way);

        let in_the_plane = if reach == other_way {
            (across.hypot(and) - reach).abs()
        } else {
            let residual = (across / reach).powi(2) + (and / other_way).powi(2) - 1.0;
            let slope = (2.0 * across / (reach * reach)).hypot(2.0 * and / (other_way * other_way));
            if slope > 0.0 {
                residual.abs() / slope
            } else {
                reach
            }
        };
        in_the_plane + up.length()
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
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<(f64, f64)> {
        let ends = run.ends().map(|end| {
            (end != nowhere_in_particular()).then(|| Point::from(end.enclosure().map(middle_of)))
        });

        let from = ends[0].map_or(0.0, |end| self.angle_of(end));
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
            return Ok((from, from + core::f64::consts::TAU));
        };
        // **A run with no ends at all is the whole of its curve, whatever picks
        // it**, and [`NamedRun::the_whole_of`] is how a pair of surfaces says so.
        // The two arcs below are then the same whole turn twice over, so both
        // are picked and the half-spaces are blamed for a tie they did not
        // make. Nothing reached this before a torus did: every other
        // `PartOfWhereItCrosses` that lies on a conic is a seam and has two
        // poles for ends, and the two loops of a degree-four crossing go to the
        // marcher instead.
        if run.is_the_whole_curve() {
            return Ok((from, from + core::f64::consts::TAU));
        }
        let to = ends[1].map_or(from + core::f64::consts::TAU, |end| self.angle_of(end));

        // The two arcs between the same two ends, going each way round.
        let onwards = from + wrapped(to - from);
        let backwards = from + wrapped(to - from) - core::f64::consts::TAU;
        let mut picked: Vec<(f64, f64)> = Vec::new();
        for end in [onwards, backwards] {
            if self.picked_by_all(picked_by, (from + end) / 2.0, deviation, policy, log)? {
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
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<bool> {
        every_half_space_holds(picked_by, self.at(middle), deviation, policy, log)
    }
}

/// A piece of the ellipse a plane cuts a cone in, walked round the cone and not
/// round the ellipse.
///
/// # Why round the cone
///
/// [`Conic::at`] puts a point at `centre + across·cos θ + and·sin θ` and rounds
/// against the largest of those. A plane a rounding off parallel to a ruling
/// cuts an ellipse whose centre is `2.2e16` out, so the points of an arc of it
/// by the frustum, `4` across, are the difference of two numbers near `2.2e16`,
/// and the one middle point of each such arc came out `1` to `2` off the cone
/// (§406.5).
///
/// The ruling at angle `a` round the cone is `apex + r·g(a)`, with
/// `g = slope·dir(a) + nappe·w` and `dir(a)` the direction [`Cone::point_at`]
/// steps across the axis by. The plane's signed distance along it is
/// `s₀ + r·(n·g)`, a straight line in `r`, so the ruling meets the plane at
/// `r = −s₀ / (n·g)`. Nothing in that is larger than the apex, the plane's
/// offset and the arc's own reach. The point is on the cone because it is on a
/// ruling, and on the plane because of the division.
///
/// # How far off, derived
///
/// With `u = f64::EPSILON / 2` and `L` the largest of the apex's coordinates,
/// the ends' coordinates and how far each end is from the apex, which is what
/// [`how_big_this_is`] is handed:
///
/// - `s₀`, [`Plane::signed_distance_to`] the apex, rounds by under
///   `8u·max(|apex|, |offset|)`, and the offset is under `√3·L` because the
///   ends are on the plane: under `14u·L`.
/// - `n·g` sums three terms each under `|g| ≤ slope + 1`, so it is out by under
///   `7u·|g|`, and times `r` that moves the point off the plane by under
///   `7u·r·|g| ≤ 7u·L`. The division adds `u·|s₀| ≤ u·L`.
/// - `dir(a)` is two unit vectors, each normalised, times a `cos` and a `sin`:
///   within `16u` of length one and of square to the axis, which moves the
///   point off the cone by under `16u·r·slope ≤ 16u·L`.
/// - [`Cone::point_at`] adds three terms under `L` per coordinate: `9u·L`.
///
/// That is about `47u·L`, under `24 · f64::EPSILON · L`. It is charged
/// [`POINT_ALONG_A_CROSSING_ROUNDOFF`], the ellipse's own constant and ten
/// times that, so that `rcad-topo`'s floor under a mesh, a multiple of that
/// constant, holds whichever way a run was walked.
///
/// # Only the arc that misses the ellipse's far end
///
/// **No point of such an arc is further from the apex than one of its ends.**
/// The distance along a ruling is `|s₀| / |n·g(a)|`, and `|n·g(a)|` is least
/// at the angle of the ellipse's far end and grows steadily from there both
/// ways round to the near end; so on an arc that misses the far end it is
/// least at one of the arc's ends. That is what lets the ends stand for the
/// arc's size. The arc that passes the far end is walked round the ellipse as
/// before, since its points are out where the ellipse's own numbers are, and
/// `rcad-topo`'s mesher asks which of the two a run is through
/// [`PointsAlongACrossing::are_placed_round_the_cone_from_the_ends_of`] before
/// it charges a face by the ends.
#[derive(Clone, Debug)]
struct AnArcRoundACone {
    flat: Plane,
    cone: Cone,
    picked_by: SmallVec<[Picker; 1]>,
    /// `1` when the ellipse is up the cone's axis from the apex, `-1` when it
    /// is down it.
    nappe: f64,
    /// `s₀`, the plane's signed distance to the apex.
    apex_off_the_plane: f64,
    ends: [Point; 2],
    /// The angle round the cone each end is at.
    angles: [f64; 2],
    /// The angle round the cone the ellipse's far end is at.
    far_end: f64,
    /// The smaller of `|n·g|` at the two ends, which is the least it gets on
    /// the arc that misses the far end.
    least_lean: f64,
    size: f64,
}

/// Which of the two arcs between a run's two ends its half-spaces pick.
enum WhichArc {
    /// The arc that misses the ellipse's far end, as angles round the cone.
    Near(f64, f64),
    /// The arc past the far end, as angles round the ellipse.
    Far(f64, f64),
}

impl AnArcRoundACone {
    /// The arc `run` names on the ellipse `pair` crosses in, when `pair` is a
    /// plane and a cone, the run is a piece between two named ends, and the
    /// ellipse reaches more than
    /// [`PointsAlongACrossing::WALKED_ROUND_THE_CONE_PAST`] times further up
    /// the cone's axis than either end does.
    ///
    /// **How far up the axis the ellipse reaches is worked out as `rcad-topo`'s
    /// mesher works it out**, `|s₀|` over `|n·ŵ| − slope·√(1 − (n·ŵ)²)`, so the
    /// two measure one thing. Where the bottom of that comes to nothing or less,
    /// which in an ellipse is a rounding, the ellipse is taken to reach for
    /// ever.
    ///
    /// `None` too when the ruling at an end does not meet the plane on the
    /// ends' own side of the apex, which every ruling of an ellipse does: the
    /// run is not what it says, and the walk round the ellipse gets to say so.
    fn of(pair: [Surface; 2], run: &NamedRun) -> Option<Self> {
        let ([Surface::Plane(flat), Surface::Cone(cone)]
        | [Surface::Cone(cone), Surface::Plane(flat)]) = pair
        else {
            return None;
        };
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
            return None;
        };
        if run.is_the_whole_curve() || run.ends().contains(&nowhere_in_particular()) {
            return None;
        }
        let ends = run
            .ends()
            .map(|end| Point::from(end.enclosure().map(middle_of)));
        let apex = cone.apex();
        let straight_on = cone.axis().direction() * cone.axis().direction().length().recip();
        let ends_along = ends.map(|end| straight_on.dot(end - apex).abs());
        let down = flat.normal().dot(straight_on).abs();
        let sideways = (1.0 - down * down).max(0.0).sqrt();
        let room = down - cone.slope() * sideways;
        let apex_off_the_plane = flat.signed_distance_to(apex);
        let far_along = if room > 0.0 {
            apex_off_the_plane.abs() / room
        } else {
            f64::INFINITY
        };
        let dwarfs_its_ends = far_along
            > PointsAlongACrossing::WALKED_ROUND_THE_CONE_PAST * ends_along[0].max(ends_along[1]);
        if !dwarfs_its_ends {
            return None;
        }

        let nappe = if straight_on.dot(ends[0] - apex) < 0.0 {
            -1.0
        } else {
            1.0
        };
        let (across, and) = cone.basis();
        let normal = flat.normal();
        let up = nappe * normal.dot(cone.axis().direction());
        let (towards_across, towards_and) = (normal.dot(across), normal.dot(and));
        let far_end = if up > 0.0 {
            (-towards_and).atan2(-towards_across)
        } else {
            towards_and.atan2(towards_across)
        };
        let mut arc = Self {
            flat,
            cone,
            picked_by: picked_by.clone(),
            nappe,
            apex_off_the_plane,
            ends,
            angles: ends.map(|end| cone.coordinates_of(end).0),
            far_end,
            least_lean: 0.0,
            size: how_big_this_is(
                &[apex, ends[0], ends[1]],
                &[(ends[0] - apex).length(), (ends[1] - apex).length()],
            ),
        };
        let leans = arc.angles.map(|round| normal.dot(arc.ruling(round)));
        let out_along_each = leans.map(|lean| -apex_off_the_plane / lean);
        if !out_along_each
            .iter()
            .all(|along| along.is_finite() && *along > 0.0)
        {
            return None;
        }
        arc.least_lean = leans[0].abs().min(leans[1].abs());
        Some(arc)
    }

    /// The run's points, round the cone when its half-spaces pick the arc that
    /// misses the ellipse's far end and round the ellipse when they pick the
    /// other.
    ///
    /// # Errors
    ///
    /// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when they pick both or
    /// neither, and whatever [`room_left_over`] and the step counts refuse.
    fn walk(
        &self,
        conic: &Conic,
        deviation_round_the_ellipse: f64,
        deflection: Deflection,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<PointsAlongACrossing> {
        match self.which_arc(conic, deviation_round_the_ellipse, policy, log)? {
            WhichArc::Near(from, to) => self.walk_round_the_cone(from, to, deflection, policy, log),
            WhichArc::Far(from, to) => {
                let Some(room) =
                    room_left_over(deflection, deviation_round_the_ellipse, policy, log)?
                else {
                    return Ok(nothing_walked(deviation_round_the_ellipse));
                };
                points_round_a_conic(conic, (from, to), deviation_round_the_ellipse, room)
            }
        }
    }

    /// Which arc the run's half-spaces pick, asked at the middle of each.
    ///
    /// **The arc past the far end is asked at its middle round the ellipse**,
    /// placed by [`Conic::at`] and boxed by the ellipse's own deviation. Round
    /// the cone, its middle is where the ruling runs nearly along the plane,
    /// and there `n·g` is a rounding and `−s₀ / (n·g)` can be anything.
    fn which_arc(
        &self,
        conic: &Conic,
        deviation_round_the_ellipse: f64,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<WhichArc> {
        let [from, to] = self.angles;
        let onwards = from + wrapped(to - from);
        let near = if wrapped(self.far_end - from) < wrapped(to - from) {
            onwards - core::f64::consts::TAU
        } else {
            onwards
        };
        let near_held = every_half_space_holds(
            &self.picked_by,
            self.at((from + near) / 2.0),
            self.deviation(),
            policy,
            log,
        )?;

        let [start, finish] = self.ends.map(|end| conic.angle_of(end));
        let round_the_ellipse = start + wrapped(finish - start);
        let how_far_out = |end: f64| (conic.at((start + end) / 2.0) - self.cone.apex()).length();
        let past_the_far_end = if how_far_out(round_the_ellipse)
            >= how_far_out(round_the_ellipse - core::f64::consts::TAU)
        {
            round_the_ellipse
        } else {
            round_the_ellipse - core::f64::consts::TAU
        };
        let far_held = every_half_space_holds(
            &self.picked_by,
            conic.at((start + past_the_far_end) / 2.0),
            deviation_round_the_ellipse,
            policy,
            log,
        )?;

        match (near_held, far_held) {
            (true, false) => Ok(WhichArc::Near(from, near)),
            (false, true) => Ok(WhichArc::Far(start, past_the_far_end)),
            (true, true) | (false, false) => Err(Error::CannotNameWhichPieceOfACrossingThisIs),
        }
    }

    /// Points round the cone from angle `from` to `to`.
    ///
    /// **Counted by how the arc bends and by how far it turns**, the two
    /// questions [`how_many_steps`] asks of an ellipse, asked of angle round the
    /// cone. A step of more than a quarter turn round the cone is one the
    /// mesher cannot tell from a step the other way.
    fn walk_round_the_cone(
        &self,
        from: f64,
        to: f64,
        deflection: Deflection,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<PointsAlongACrossing> {
        let deviation = self.deviation();
        let Some(room) = room_left_over(deflection, deviation, policy, log)? else {
            return Ok(nothing_walked(deviation));
        };
        let by_bending = how_many_pieces_an_endless_bend_takes(
            self.how_far_a_step_of_one_stands_off_its_chord(),
            to - from,
            room,
        )?;
        let by_turning = ((to - from).abs() / (core::f64::consts::TAU / 4.0)).ceil() as usize;
        let steps = by_bending.max(by_turning);
        let points = (1..steps)
            .map(|step| self.at(from + (to - from) * (step as f64) / (steps as f64)))
            .collect();

        Ok(PointsAlongACrossing {
            points,
            deviation: Approximation::new(deviation, Cause::PointAlongACrossing),
        })
    }

    /// Where the ruling at angle `round` meets the plane.
    fn at(&self, round: f64) -> Point {
        let along = -self.apex_off_the_plane / self.flat.normal().dot(self.ruling(round));
        self.cone.point_at(round, self.nappe * along)
    }

    /// `g(round)`: the step one unit along the ruling at `round` takes, on the
    /// ellipse's own nappe.
    fn ruling(&self, round: f64) -> Vector {
        let (across, and) = self.cone.basis();
        (across * round.cos() + and * round.sin()) * self.cone.slope()
            + self.cone.axis().direction() * self.nappe
    }

    fn deviation(&self) -> f64 {
        POINT_ALONG_A_CROSSING_ROUNDOFF * self.size
    }

    /// How far the arc stands off the chord of a step round the cone, per
    /// square radian.
    ///
    /// A curve `p(a)` stands off the chord of a step `Δ` by at most
    /// `‖p″‖·Δ²/8`, as [`how_many_pieces_an_endless_bend_takes`] says. Here
    /// `p − apex = −s₀·g/D` with `D = n·g`, so
    ///
    /// ```text
    /// (g/D)″ = g″/D − 2g′D′/D² − g·D″/D² + 2g·D′²/D³
    /// ```
    ///
    /// and with `|g′| = |g″| = slope`, `|g| ≤ slope + ‖w‖`, `|D′|` and `|D″|`
    /// at most `slope·‖n‖`, and `|D|` at least [`AnArcRoundACone::least_lean`]
    /// on this arc, every term has a bound.
    fn how_far_a_step_of_one_stands_off_its_chord(&self) -> f64 {
        let slope = self.cone.slope();
        let turns = slope * self.flat.normal().length();
        let ruling = slope + self.cone.axis().direction().length();
        let least = self.least_lean;
        let bends = slope / least
            + (2.0 * slope + ruling) * turns / (least * least)
            + 2.0 * ruling * turns * turns / (least * least * least);
        self.apex_off_the_plane.abs() * bends / 8.0
    }
}

/// Which side of a half-space the box `at` is on, spending the policy's budget
/// on the answer when nothing could tell and recording it either way.
///
/// A half-space that could not tell rules out neither piece, so both come back
/// picked and the caller is refused rather than handed a coin toss. It is
/// recorded because it happened, and because the refusal that follows says
/// nothing about which half-space gave up.
///
/// **A tolerant caller is not refused before its budget is looked at.** The box
/// handed in is `deviation` wide on every side, so "undecided" here means the
/// box straddles the surface — and a caller that has already said it will accept
/// a point that far out of place has said it will accept this. So the record
/// goes through [`resolve_under_policy`], which reads the enclosure as
/// [`Decision::Zero`] when the budget covers it and leaves it undecided when it
/// does not. A certified policy is unmoved: it takes the `Certified` arm there,
/// which hands back the same `Undecidable` and pushes the same record this used
/// to push by hand.
///
/// `Decision::Zero` is the picker holding the piece, exactly as a certified zero
/// is: only [`Decision::Neg`] rules a piece out.
fn which_side_of(
    picker: &Picker,
    at: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Decision {
    let decision = picker.side_of_point(at);
    let named = picker.predicate_name();
    let measures = match picker {
        Picker::InFrontOf(_) => Measures::Length,
        Picker::OutsideABall(_) | Picker::OutsideAPipe(_) | Picker::InsideAPipe(_) => {
            Measures::Area
        }
    };
    match decision {
        Decision::Pos | Decision::Zero | Decision::Neg => decision,
        Decision::Undecidable { enclosure } => resolve_under_policy(
            Record::new(named, &[at], Level::Interval, enclosure, measures),
            policy,
            log,
        ),
        other => panic!("a half-space came back with a decision nothing knows: {other:?}"),
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
/// how far the plane leans, which is `‖n‖‖w‖ / |n·w|` — and that is a division
/// by the very number "is this plane parallel to the axis" decided is not zero.
/// The `‖w‖` is there because `n·w` reads the stored direction at whatever
/// length it has while the lean is an angle, and the pipe is `radius` wide at
/// any such length (`docs/decisions.md` §118.10).
///
/// # Panics
///
/// Never, and unlike [`where_a_plane_cuts_a_cone`] below this one can say so
/// without asking anything twice. The only call is in
/// [`what_a_run_between_these_two_lies_on`], which answers `StraightLine` three
/// lines earlier on `flat.normal().dot(pipe.axis().direction()) == 0.0` — the
/// same two `f64`s through the same product as the assert reads, in the same
/// function, with no branch between. A caller-stated run gets that answer and
/// never arrives here, which
/// `a_closed_run_a_caller_names_on_a_crossing_of_straight_lines_is_refused_by_name`
/// (`crates/rcad-geom/tests/the_degeneracies_a_crossing_has_to_answer.rs`)
/// walks in through both public doors to show.
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
    let radius_across = pipe.radius();
    let radius_down_the_slope = pipe.radius() * normal.length() * along.length() / leans.abs();

    Ok(Conic {
        centre: middle,
        across: across * radius_across,
        and: and * radius_down_the_slope,
        size: how_big_this_is(
            &[middle, pipe.axis().origin()],
            &[radius_across, radius_down_the_slope],
        ),
    })
}

/// Whichever of a plane and a cone's four bending shapes this pair makes: an
/// ellipse, a parabola, one branch of a hyperbola, or a straight line.
///
/// # The two semi-axes, derived
///
/// Write `n̂` and `ŵ` for the plane's normal and the cone's axis scaled to length
/// one, `h = n̂·apex − d/‖n‖` for how far the apex stands off the plane, `A =
/// n̂·ŵ`, `k = ‖n̂ × ŵ‖`, and `t` for the slope. Set two directions in the plane:
/// `û₂ = (n̂ × ŵ)/k` and `û₁ = û₂ × n̂`, so `û₁ × û₂ = n̂`. A point of the plane is
/// `apex − h·n̂ + s·û₁ + r·û₂`, and putting that into the cone's own equation
/// `‖x‖² = (1 + t²)(x·ŵ)²` gives, after one completion of the square,
///
/// ```text
/// E·(s − s_c)² + r² = h²t²/E,   E = (1 + t²)A² − t²,   s_c = −(1 + t²)hAk/E
/// ```
///
/// — the `t²` on the right is what is left after `(1 + t²)²A²k² + E² + E(t² − 1)`
/// collapses, and it collapsing is why the answer is this short. So the
/// semi-axis along `û₁` is `|h|t/E` and the one along `û₂` is `|h|t/√E`, and the
/// centre is `apex − h·n̂ + s_c·û₁`.
///
/// `E` is the very number [`intersect`](crate::intersect) decided the sign of to
/// call this an ellipse, and it is **asked again here** rather than assumed of
/// the caller: a run between any two surfaces is something a caller states and a
/// file stores, so all four shapes arrive here. It is summed through [`Exact`]
/// for the same reason a nearly-tangent circle's radius is: a plane leaning at
/// nearly the cone's own half-angle makes it a small difference of two large
/// numbers.
///
/// # The other three shapes come out of the same completed square
///
/// Written in the two directions [`two_directions_in`] hands back, with `s`
/// along the second and `r` along the first, the line above is one equation:
///
/// ```text
/// r² + E·s² + K·s + C = 0,   K = 2(1 + t²)h·(n̂·ŵ)·(and·ŵ),   C = h²(1 − (1 + t²)(n̂·ŵ)²)
/// ```
///
/// — and `K²/(4E) − C` is `h²t²/E`, which is how the ellipse's two semi-axes
/// above come out of it. The other three are the other signs of `E`:
///
/// - **`E = 0` is the statement that the quadratic loses a term**, and what is
///   left is `r² + K·s + C = 0`, a parabola `s = −(r² + C)/K` with its vertex
///   `C/K` along `and` from the foot of the perpendicular from the apex and its
///   bend `−and/K`. `K` cannot be zero in this row: `n̂·ŵ = 0` makes `E = −t²`
///   and `‖n̂ × ŵ‖ = 0` makes `E = 1`, so neither factor vanishes where `E` does.
/// - **`E < 0` is a hyperbola**, and the *same three formulas the ellipse uses*
///   give it, read with `|E|`: the transverse semi-axis along `and` is `|h|t/|E|`
///   and the conjugate one across is `|h|t/√|E|`, about the same centre
///   `s_c = −K/(2E)`, which the signed `E` gets right on its own. **It is not a
///   [`Conic`]**, because a hyperbola's two branches are `cosh`/`sinh` and not
///   `cos`/`sin` — [`HyperbolaBranch`] says why sharing the four fields would be
///   worse than writing them twice.
/// - **`E ≤ 0` with the apex *on* the plane** is a ruling or a pair of them,
///   which are straight and are walked by their ends.
///
/// This used to assert that `where_a_plane_and_a_cone_cross` had already refused
/// the two bending shapes — but that is a claim about a *caller*, and this is
/// reached from [`walk`] and [`where_a_closed_crossing_starts`], both public,
/// both taking whatever pair of surfaces and whatever [`NamedRun`] they are
/// handed. `EdgeRun` is `Deserialize` too, so a file states such a run.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a product leaves `f64`'s
/// range, [`Error::TheCrossingOfTheseTwoSurfacesIsNotClosed`] for a run a caller
/// states as closed on a crossing that runs on for ever, and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the half-spaces on a
/// hyperbola's run hold both of its branches or neither.
///
/// A plane through the apex that does cut across the cone meets it at the apex
/// and nowhere else, and that comes back as a conic of no size centred exactly
/// on the apex — the same answer, built the same way, that a plane resting on a
/// ball gets from [`where_a_plane_cuts_a_ball`].
///
/// **The two directions come from [`two_directions_in`]**, which hands back
/// `(−û₂, −û₁)` — the same pair a pipe uses, in the same roles: its first is
/// square to the axis, which for a cone is the *minor* axis, and its second runs
/// down the plane's slope, which is the major one. Negating both leaves
/// `across × and` where it was, so the ellipse is traced the way a pipe's is.
fn where_a_plane_cuts_a_cone(
    flat: &Plane,
    cone: &Cone,
    run: &NamedRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhatARunLiesOn> {
    let normal = flat.normal().components();
    let along = cone.axis().direction().components();
    let apex = cone.apex();
    let slope = cone.slope();

    let worked_out = || {
        let squared_normal = Exact::dot_of(normal, normal)?;
        let squared_along = Exact::dot_of(along, along)?;
        let up = Exact::dot_of(normal, along)?;
        let sideways = Exact::squared_length_of(&Exact::cross_of(&Exact::of_each(normal), along)?)?;
        let leaning = up
            .times(&up)?
            .minus(&Exact::product_of(slope, slope)?.times(&sideways)?);
        let reach = Exact::dot_of(normal, apex.coordinates())?.minus(&Exact::of(flat.offset()));
        Some((
            leaning.nearest_f64(),
            squared_normal.nearest_f64(),
            squared_along.nearest_f64(),
            up.nearest_f64(),
            sideways.nearest_f64(),
            reach.nearest_f64(),
        ))
    };
    let Some((leaning, squared_normal, squared_along, up, sideways, reach)) = worked_out() else {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    };
    // `E = leaning / (‖n‖²‖w‖²)`, `h = reach / ‖n‖`, `A = up / (‖n‖‖w‖)` and
    // `k = √sideways / (‖n‖‖w‖)`, all substituted into the three lines above and
    // the lengths cancelled by hand so that nothing takes a root it need not.
    let normal_length = squared_normal.sqrt();
    let along_length = squared_along.sqrt();
    let (across, and) = two_directions_in(flat, cone.axis().direction());
    let foot = apex - flat.normal() * (reach / squared_normal);
    // `−s_c = K/(2E)`, which the signed `leaning` gets right for both the
    // ellipse and the hyperbola. Read in neither of the other two rows, where
    // there is no centre to be off, and a division by a `leaning` of zero is
    // why it waits until it is.
    let how_far_off_the_foot_the_centre_is =
        || -(1.0 + slope * slope) * reach * up * sideways.sqrt() / (normal_length * leaning);

    if leaning <= 0.0 {
        if reach == 0.0 {
            return Ok(WhatARunLiesOn::StraightLine);
        }
        if run.run().it_comes_back_to_where_it_started() {
            return Err(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed);
        }
        let standing_off = reach / normal_length;
        let axis_up = up / (normal_length * along_length);
        let bend_up = and.dot(cone.axis().direction()) / along_length;
        let linear_term = 2.0 * (1.0 + slope * slope) * standing_off * axis_up * bend_up;
        if leaning == 0.0 {
            let constant_term =
                standing_off * standing_off * (1.0 - (1.0 + slope * slope) * axis_up * axis_up);
            return Ok(WhatARunLiesOn::Parabola(
                the_parabola_a_plane_along_a_ruling_cuts(
                    apex,
                    foot,
                    across,
                    and,
                    [standing_off, linear_term, constant_term],
                ),
            ));
        }
        let towards = reach.abs() * slope * normal_length * squared_along / leaning.abs();
        let sideways_reach = reach.abs() * slope * along_length / leaning.abs().sqrt();
        let centre = foot - and * how_far_off_the_foot_the_centre_is();
        return which_branch_of_a_hyperbola_this_run_is(
            [towards, sideways_reach, standing_off],
            [apex, centre],
            [across, and, cone.axis().direction()],
            run,
            policy,
            log,
        );
    }

    let major = reach.abs() * slope * normal_length * squared_along / leaning;
    let minor = reach.abs() * slope * along_length / leaning.sqrt();
    let off_centre = how_far_off_the_foot_the_centre_is();
    let centre = foot - and * off_centre;

    Ok(WhatARunLiesOn::Conic(Conic {
        centre,
        across: across * minor,
        and: and * major,
        size: how_big_this_is(&[centre, apex], &[major, minor, off_centre]),
    }))
}

/// The parabola `foot − and·(C/K) + across·s − and·(s²/K)`, put together from
/// the three numbers `where_a_plane_cuts_a_cone` derived.
///
/// `spread` is `[h, K, C]`, and all three go into the size this publishes its
/// bound against: `|K|` and `|C/K|` are lengths this parabola is genuinely made
/// of — `|K|` is twice the focal parameter and `|C/K|` is how far the vertex
/// stands from the foot — and
/// [`POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF`]'s derivation needs both of them
/// there to stay a bound.
fn the_parabola_a_plane_along_a_ruling_cuts(
    apex: Point,
    foot: Point,
    across: Vector,
    and: Vector,
    spread: [f64; 3],
) -> Parabola {
    let [standing_off, linear_term, constant_term] = spread;
    let off_the_foot = constant_term / linear_term;
    let vertex = foot - and * off_the_foot;
    Parabola {
        vertex,
        across,
        bend: and * -linear_term.recip(),
        size: how_big_this_is(&[vertex, apex], &[linear_term, off_the_foot, standing_off]),
    }
}

/// Which of a hyperbola's two branches this run names: the one its own ends are
/// on, and the one its half-spaces hold when it has no ends.
///
/// Each branch lies wholly in one nappe of the cone, so which side of the plane
/// through the apex square to the axis a point is on says which branch it can
/// be on — the sign of `(x − apex)·w`, which does not change anywhere on one
/// branch.
///
/// **An end is asked before a picker, because a picker can hold the wrong
/// branch.** A half-space through the chord of an arc holds the arc and the
/// whole of the other branch too, and not this branch's vertex, so asking the
/// pickers at each vertex walked an arc across the apex onto the other nappe.
/// The fuse wrote exactly such pickers until e2cd175d, and [`walk`] is public
/// and takes any run, so what a writer chose cannot be what this depends on.
/// Once the branch is named, the ends are what say how much of it to walk.
/// [`which_way_the_branch_these_ends_are_on_opens`] says how the side the ends
/// are on names the branch without asking any point of it.
///
/// A run with no ends is the pair [`intersect`](crate::intersect) mints, and
/// there the pickers are the whole of what tells the branches apart: the
/// same shape as [`where_a_plane_cuts_a_torus`], each candidate asked whether
/// every picker holds its vertex, one held candidate an answer and two or none
/// not. `intersect`'s picker is that same plane through the apex, with the
/// axis as its normal bit for bit, so one point settles the whole branch.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when an end cannot be put
/// on one side of the apex, when the two ends are on different sides of it,
/// and, for a run with no ends, when the pickers hold both branches or neither.
fn which_branch_of_a_hyperbola_this_run_is(
    spread: [f64; 3],
    places: [Point; 2],
    directions: [Vector; 3],
    run: &NamedRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhatARunLiesOn> {
    let [towards, sideways_reach, standing_off] = spread;
    let [apex, centre] = places;
    let [across, and, axis] = directions;
    let size = how_big_this_is(&[centre, apex], &[towards, sideways_reach, standing_off]);
    let the_branch_opening = |opening: f64| HyperbolaBranch {
        centre,
        out: and * (towards * opening),
        across: across * sideways_reach,
        size,
    };

    let ends: Vec<PredicatePoint> = run
        .ends()
        .into_iter()
        .filter(|end| *end != nowhere_in_particular())
        .collect();
    if !ends.is_empty() {
        let opening = which_way_the_branch_these_ends_are_on_opens(axis, apex, &ends, policy, log)?;
        return Ok(WhatARunLiesOn::OneBranchOfAHyperbola(the_branch_opening(
            opening,
        )));
    }

    let deviation = POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF * size;
    let picked_by: &[Picker] = match run.run() {
        EdgeRun::PartOfWhereItCrosses { picked_by, .. } => picked_by,
        EdgeRun::Straight
        | EdgeRun::RightRoundWhereItsFacesCross
        | EdgeRun::RightRoundWhereItCrosses { .. } => &[],
    };
    let mut held = Vec::new();
    for branch in [1.0, -1.0].map(the_branch_opening) {
        if every_half_space_holds(picked_by, branch.at(0.0), deviation, policy, log)? {
            held.push(branch);
        }
    }
    match held.as_slice() {
        [only] => Ok(WhatARunLiesOn::OneBranchOfAHyperbola(*only)),
        _ => Err(Error::CannotNameWhichPieceOfACrossingThisIs),
    }
}

/// The `s` of the branch `centre + and·towards·s·cosh σ + across·sinh σ` that
/// `ends` are on: `−1` when they are on the side of the apex the axis points
/// to, and `+1` when they are on the other.
///
/// **Branch `s` lies on the side `−s` names, on every hyperbola.** `across` is
/// square to the axis `w` and `towards` is positive, so far out along branch
/// `s` the point runs along `s·and`, and it is on the side of the apex that the
/// sign of `s·(and·w)` names. Since the whole branch is on one side, that is
/// the branch's side. And `and·w` is never positive. `and` is `â × n̂`, with
/// `â` along `w × n`, and `(w × n) × n · w = (w·n)² − |w|²|n|² = −|w × n|²`,
/// so `and·w` is `−|w × n̂|`. That is zero only where the plane is square to
/// the axis, and such a plane cuts a cone in a circle and never in a
/// hyperbola.
///
/// **So no point of the branch is asked.** Asking a branch's vertex which side
/// of the apex it is on, which this did until Q1405, refuses wherever the
/// plane misses the apex by a rounding, because both vertices are then a
/// rounding from the apex and each is placed only to
/// [`POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF`] times the size (§294.14). The
/// identity holds of the exact `and`, and the branch built on the rounded one
/// with this `s` is the rounding of the exact branch with this `s`, so nothing
/// rounded is read to pick it.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when an end cannot be put
/// on one side of the apex, and when two ends are on different sides of it.
/// A point on the plane through the apex square to the axis is on neither
/// branch, since the plane the hyperbola lies in misses the apex, so a side a
/// tolerant policy reads as `Zero` is refused just as one it could not tell.
fn which_way_the_branch_these_ends_are_on_opens(
    axis: Vector,
    apex: Point,
    ends: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<f64> {
    let mut opening = None;
    for end in ends {
        let this_ones = match which_side_of_the_apex(axis, apex, *end, policy, log) {
            Decision::Pos => -1.0,
            Decision::Neg => 1.0,
            Decision::Zero | Decision::Undecidable { .. } => {
                return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
            }
            other => {
                panic!("a side of the apex came back with a decision nothing knows: {other:?}")
            }
        };
        if opening.is_some_and(|so_far| so_far != this_ones) {
            return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
        }
        opening = Some(this_ones);
    }
    Ok(opening.expect("a run with ends has at least one end"))
}

/// Which side of the plane through `apex` square to `axis` the point `at` is
/// on: the sign of `(at − apex)·axis`, `Pos` on the side the axis points to.
///
/// A sign the cascade cannot tell is recorded in `log` and resolved under
/// `policy`, so a tolerant policy can hand it back as `Zero`.
fn which_side_of_the_apex(
    axis: Vector,
    apex: Point,
    at: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Decision {
    let question = SideOfTheApex {
        axis: axis.components(),
        apex: apex.coordinates(),
        at,
    };
    let (decided, level) = decide(&question);
    if let Decision::Undecidable { enclosure } = decided {
        resolve_under_policy(
            Record::new(SIDE_OF_THE_APEX, &[at], level, enclosure, Measures::Length),
            policy,
            log,
        )
    } else {
        decided
    }
}

/// The name [`which_side_of_the_apex`] records an undecided sign under.
const SIDE_OF_THE_APEX: &str = "which side of a cone's apex";

/// `(at − apex)·axis`, for the cascade.
///
/// There is no `f64` level: the reading is only ever taken twice per run, and
/// the interval level already settles every point that is not within rounding
/// of the apex's plane.
struct SideOfTheApex {
    axis: [f64; 3],
    apex: [f64; 3],
    at: PredicatePoint,
}

impl Predicate for SideOfTheApex {
    fn name(&self) -> &'static str {
        SIDE_OF_THE_APEX
    }

    fn on_floats(&self) -> Option<Filtered> {
        None
    }

    fn on_intervals(&self) -> Interval {
        let at = self.at.enclosure();
        let term = |axis: usize| {
            Interval::exactly(self.axis[axis]) * (at[axis] - Interval::exactly(self.apex[axis]))
        };
        term(0) + term(1) + term(2)
    }

    fn exactly(&self) -> Option<Decision> {
        let at = self.at.exact_coordinates()?;
        let reading = Exact::dot_of(self.axis, at)?.minus(&Exact::dot_of(self.axis, self.apex)?);
        match reading.sign() {
            Decision::Pos => Some(Decision::Pos),
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            // An entry that is not a real number, which takes an overflow: the
            // exact level cannot run.
            Decision::Undecidable { .. } => None,
            other => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
        }
    }
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
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when one of those products falls out
/// of `f64`'s range.
///
/// [`Error::TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing`] for two balls with
/// one centre and one radius, and
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] for two balls with one centre
/// and two radii. Both are **limits** (§16.10): the first pair shares its whole
/// skin and the second shares none of it, and neither has a circle to hand back
/// at any precision. Everything the radius and the middle are worked out from
/// divides by the gap between the two centres, so a pair with no gap is the one
/// pair the formula above says nothing about.
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
    // Two centres in the same place leave nothing to divide by. Of one radius
    // that put `0 / 0` in every coordinate of `middle`, and
    // `Plane::from_point_and_normal` refused it as a "point on a plane" that is
    // not finite — the radical plane named for a fault in the pair it was built
    // from. Of two radii the numerator is not zero, so `middle` runs off to
    // infinity instead, and the two are told apart here because one pair is the
    // same skin twice and the other is two skins that never touch.
    if squared_gap == 0.0 {
        return Err(if mine == theirs {
            Error::TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing
        } else {
            Error::TheseTwoSurfacesDoNotCrossInAConic
        });
    }
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

/// The circle where a plane cuts a torus, picked out of the two the crossing can
/// come in.
///
/// **Two families reach here from [`intersect`](crate::intersect) and no
/// others.** A plane square to the axis cuts circles round the axis; a plane
/// holding the axis cuts the two meridian circles. Every other plane is
/// refused by name there before a run exists, so this never has to stand in
/// for a quartic.
///
/// **A run read from a file has not been through
/// [`intersect`](crate::intersect), and can be a rounding off square.**
/// `rcad-step` reads a seam's plane with the normal the file wrote and a
/// ring's axis scaled to length one again, so a bend turned about `x` by 2.5
/// came in with `n × w = (5.55e-17, 0, 0)` for two spellings of one written
/// direction. This sends such a plane to the meridian circles, neither of
/// which holds the seam's ends, and it is refused below rather than drawn
/// (Q842, `docs/decisions.md` §241.6).
///
/// # Which of the two circles this run is, is asked two ways
///
/// **The run's own ends first.** An end that names a place is a point on the
/// crossing, and a point on one of two disjoint circles says which one it is
/// with nothing to decide. That is what a *closed* edge on a torus needs: a cap
/// of a bend is one whole circle of the two its plane cuts, so it is
/// `RightRoundWhereItsFacesCross` with one corner at both ends and no picker on
/// it at all, and the corner is the whole of what tells the two apart.
///
/// **Then the run's pickers**, at the point each surviving candidate starts
/// from, taken as the box it is really known to be inside — the same question
/// `the_seed_this_run_is_picked_up_at` puts to the marcher's seeds, asked the
/// same way. That is what a run with **no** ends needs: what
/// [`intersect`](crate::intersect) hands back names a whole curve and nothing
/// on it.
///
/// A candidate has to pass both, and exactly one may.
///
/// # Which family this is, is asked in `f64`
///
/// `n × w == 0` is one exact comparison of stored triples and not a cascade
/// question, exactly as the pipe arm above asks `n·w == 0.0`. §39.7 item 6
/// records that as a standing gap — two authorities on the same question, one
/// exact in [`intersect`](crate::intersect) and one an `f64` comparison here —
/// and this arm is a third instance of it rather than a new one.
///
/// **What saves it is the gap between the two families, not the arithmetic.**
/// `a·d − b·c` in `f64` rounds a non-zero to zero all the time, and it does so
/// for exactly the triples these two are: `a_cross_product_of_unit_triples_can_round_to_zero`
/// builds two unit vectors one `f64` apart whose cross product comes out zero
/// with a true value near `1e-17`. So a test on the computed cross product would
/// be worth nothing if the two families it separates ran up against each other.
/// They do not. A run [`intersect`](crate::intersect) named reaches here only
/// in one of the two families it certified **exactly**, and the family this
/// arm sends away is the one holding the axis, where `n·w = 0` for two vectors
/// of length one — so `‖n × w‖` is one there, and no rounding of anything walks
/// a whole unit. The pipe arm's `n·w == 0.0` has a three-term sum in it and
/// separates its own two families the same way.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when the circle's own numbers do
/// not come out finite, and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the run's ends and
/// pickers hold both candidates or neither.
fn where_a_plane_cuts_a_torus(
    flat: &Plane,
    ring: &Torus,
    run: &NamedRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Conic> {
    let candidates = if flat.normal().cross(ring.axis().direction()) == Vector::zero() {
        circles_round_the_axis(flat, ring)
    } else {
        the_two_meridian_circles(flat, ring)
    };
    which_of_these_circles_this_run_is(candidates, run, policy, log)
}

/// The circles a plane square to a torus's axis cuts, outer first.
///
/// The centre of both is where the axis meets the plane, and the two radii are
/// `R ± √(r² − t²)` for `t` the axial distance from the torus's centre to the
/// plane. Where the plane rests exactly on the tube the root is zero, both
/// radii are `R`, and the two candidates are the same circle twice — which is
/// right: the crossing really is one circle then, and a run with no picker on it
/// takes it.
fn circles_round_the_axis(flat: &Plane, ring: &Torus) -> Vec<Conic> {
    let along = ring.axis().direction();
    let up = along * along.length_squared().recip();
    let reach = (flat.offset() - flat.normal().dot(ring.centre() - Point::origin()))
        / flat.normal().dot(up);
    let middle = ring.centre() + up * reach;
    let axial = reach * along.length();
    let tube = ring.tube_radius();
    let half_width = (tube * tube - axial * axial).max(0.0).sqrt();
    let (across, and) = ring.basis();

    // The plane resting exactly on the tube cuts **one** circle and not two.
    // Written out rather than left to the two radii coming out equal, because a
    // caller with no picker on its run needs one candidate to take, and two
    // copies of the same circle would be a tie.
    let radii: &[f64] = if half_width == 0.0 {
        &[ring.ring_radius()]
    } else {
        &[
            ring.ring_radius() + half_width,
            ring.ring_radius() - half_width,
        ]
    };
    radii
        .iter()
        .copied()
        .map(|radius| Conic {
            centre: middle,
            across: across * radius,
            and: and * radius,
            size: how_big_this_is(&[middle, ring.centre()], &[radius, tube]),
        })
        .collect()
}

/// The two circles a plane through a torus's axis cuts, each of the tube's own
/// radius, centred where the plane meets the centre circle.
fn the_two_meridian_circles(flat: &Plane, ring: &Torus) -> Vec<Conic> {
    let along = ring.axis().direction();
    let up = along * along.length_squared().recip();
    let sideways = match ring.axis().direction().cross(flat.normal()).normalized() {
        Ok(across) => across,
        // Unreachable: this arm is only taken with `n × w` non-zero, which is
        // the very thing a normalize needs.
        Err(_) => return Vec::new(),
    };
    let tube = ring.tube_radius();

    [ring.ring_radius(), -ring.ring_radius()]
        .into_iter()
        .map(|out| {
            let middle = ring.centre() + sideways * out;
            Conic {
                centre: middle,
                across: sideways * (tube * out.signum()),
                and: up * tube,
                size: how_big_this_is(&[middle, ring.centre()], &[out, tube]),
            }
        })
        .collect()
}

/// The circle a cone and a pipe on one axis cross in, picked out of the two of
/// them by the run's own half-space.
///
/// # Errors
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] where the two are not
/// **proved** to share an axis, which is the same refusal
/// [`intersect`](crate::intersect) gives that pair and for the same reason. The
/// question is asked here rather than assumed from the caller: [`walk`] is
/// public, and a file read in states a cone against a pipe whenever it holds a
/// countersink met by a bore.
///
/// Anything but [`Decision::Zero`] refuses, including an undecided answer, and
/// that is the invariant `whether_a_cone_and_a_pipe_share_an_axis`'s
/// pass-through arm rests on: only a proof of one shared axis may draw circles.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when the circles' own numbers do
/// not come out finite, and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the pickers hold both
/// circles or neither.
fn where_a_coaxial_cone_and_pipe_cut(
    cone: &Cone,
    pipe: &Cylinder,
    run: &NamedRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Conic> {
    if whether_a_cone_and_a_pipe_share_an_axis(cone, pipe) != Decision::Zero {
        return Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface);
    }
    which_of_these_circles_this_run_is(
        the_two_circles_a_cone_and_a_pipe_cut(cone, pipe),
        run,
        policy,
        log,
    )
}

/// The circle two cones on one axis cross in that the run names: the one
/// halfway between their apexes where they have one slope, and one of the two
/// `intersect` names where they have two.
///
/// # Errors
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] where the two are not
/// **proved** to share an axis, asked here for the reason
/// [`where_a_coaxial_cone_and_pipe_cut`] gives.
///
/// **The same name where they share an apex, and there it is the wrong name.**
/// Two cones on one axis with one apex and two slopes meet at the apex and
/// nowhere else — `intersect` answers them
/// [`SurfaceMeet::TouchAtOnePoint`](crate::SurfaceMeet::TouchAtOnePoint) — so a
/// closed run stated on them names a curve that is not there, and
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] is what a plane touching a
/// ball gets for the same thing. `a_circle_walked_between_two_spellings_of_one_cone.rs`
/// pins the old name on one such pair and is outside the change that named
/// these cones, so the two move together or not at all. One cone twice under
/// any run but a whole circle is refused here too, as it always was.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] as
/// [`which_of_these_circles_this_run_is`] gives them.
fn where_two_coaxial_cones_cut(
    one: &Cone,
    other: &Cone,
    run: &NamedRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Conic> {
    if whether_two_cones_share_an_axis(one, other) != Decision::Zero || one.apex() == other.apex() {
        return Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface);
    }
    which_of_these_circles_this_run_is(
        the_circles_two_coaxial_cones_cut(one, other),
        run,
        policy,
        log,
    )
}

/// The circles two cones on one axis with two apexes cross in, where
/// `intersect` names them: from the steeper cone's apex, `d` up its axis to the
/// other apex, at `d·s_shallow/(s_steep + s_shallow)` and
/// `−d·s_shallow/(s_steep − s_shallow)`, or the one at `d/2` where the slopes
/// are one.
///
/// **Drawn round the steeper cone, or the one that sorts first where neither
/// is**, so the pair handed over either way round starts each circle at one
/// point and walks it one way.
fn the_circles_two_coaxial_cones_cut(one: &Cone, other: &Cone) -> Vec<Conic> {
    let (steep, shallow) = match one.slope().total_cmp(&other.slope()) {
        Ordering::Greater => (one, other),
        Ordering::Less => (other, one),
        Ordering::Equal => match Surface::Cone(*one).how_it_sorts_against(&Surface::Cone(*other)) {
            Ordering::Greater => (other, one),
            Ordering::Less | Ordering::Equal => (one, other),
        },
    };
    let along = steep.axis().direction();
    let apart = (shallow.apex() - steep.apex()).dot(along * along.length().recip());
    let reaches = if steep.slope() == shallow.slope() {
        vec![apart / 2.0]
    } else {
        let tilted = apart * shallow.slope();
        vec![
            tilted / (steep.slope() + shallow.slope()),
            -tilted / (steep.slope() - shallow.slope()),
        ]
    };
    reaches
        .into_iter()
        .map(|reach| a_circle_round_a_cones_axis(steep, reach, shallow.apex(), apart))
        .collect()
}

/// The circle of the crossing of a cone and a ball on its axis that the run
/// names.
///
/// # Errors
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] where the ball's
/// centre is not **proved** on the axis, and where its skin runs through the
/// apex — the two refusals `intersect` gives the pair, under the same name.
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] where the ball misses the cone,
/// so the run names a circle that is not there — a **limit**, as for a plane
/// that misses a ball.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] where a sign the circles turn on
/// could not be settled, which takes a product leaving `f64`'s range, and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] as
/// [`which_of_these_circles_this_run_is`] gives it.
fn where_a_ball_on_a_cones_axis_cuts_it(
    cone: &Cone,
    ball: &Sphere,
    run: &NamedRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Conic> {
    if whether_a_ball_sits_on_a_cones_axis(cone, ball) != Decision::Zero {
        return Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface);
    }
    let two = match how_a_ball_on_a_cones_axis_meets_it(cone, ball) {
        Decision::Neg => return Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
        Decision::Zero => false,
        Decision::Pos => match whether_a_ball_runs_through_a_cones_apex(cone, ball) {
            Decision::Neg | Decision::Pos => true,
            Decision::Zero => return Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
            Decision::Undecidable { .. } | _ => {
                return Err(Error::CannotWorkOutHowBigACrossingIs);
            }
        },
        Decision::Undecidable { .. } | _ => return Err(Error::CannotWorkOutHowBigACrossingIs),
    };
    which_of_these_circles_this_run_is(
        the_circles_a_ball_on_a_cones_axis_cuts(cone, ball, two),
        run,
        policy,
        log,
    )
}

/// The one circle or the two a ball on a cone's axis cuts it in: `h` along the
/// axis from the apex with `(1 + s²)h² − 2eh + e² − R² = 0`, `e` how far up the
/// axis the centre sits.
///
/// The discriminant over four is written `R² + s²(R − |e|)(R + |e|)` rather than
/// `(1 + s²)R² − s²e²`, which is the same number without the two large squares
/// cancelling. It is only read where `intersect`'s exact sign said it is above
/// zero, so a value that rounded below is taken as nothing.
fn the_circles_a_ball_on_a_cones_axis_cuts(cone: &Cone, ball: &Sphere, two: bool) -> Vec<Conic> {
    let along = cone.axis().direction();
    let centre_at = (ball.centre() - cone.apex()).dot(along * along.length().recip());
    let radius = ball.radius();
    let squared_slope = cone.slope() * cone.slope();
    let widening = 1.0 + squared_slope;
    let middle = centre_at / widening;
    let reaches = if two {
        let room = radius * radius
            + squared_slope * (radius - centre_at.abs()) * (radius + centre_at.abs());
        let half = room.max(0.0).sqrt() / widening;
        vec![middle + half, middle - half]
    } else {
        vec![middle]
    };
    reaches
        .into_iter()
        .map(|reach| a_circle_round_a_cones_axis(cone, reach, ball.centre(), radius))
        .collect()
}

/// The circle on `cone` square to its axis, `reach` up it from the apex, sized
/// against `beside` and `length` too, which are what `reach` was worked out
/// from.
fn a_circle_round_a_cones_axis(cone: &Cone, reach: f64, beside: Point, length: f64) -> Conic {
    let along = cone.axis().direction();
    let middle = cone.apex() + along * (reach / along.length());
    let radius = cone.radius_at(reach);
    let (across, and) = cone.basis();
    Conic {
        centre: middle,
        across: across * radius,
        and: and * radius,
        size: how_big_this_is(&[middle, cone.apex(), beside], &[radius, reach, length]),
    }
}

/// The two circles a cone and a pipe on one axis cross in, the one the axis
/// points towards first.
///
/// Both have the pipe's radius, and they sit that over the slope along the axis
/// from the apex either way — which is the `±r/slope` the namer proved there
/// were exactly two of.
fn the_two_circles_a_cone_and_a_pipe_cut(cone: &Cone, pipe: &Cylinder) -> Vec<Conic> {
    let along = cone.axis().direction();
    let up = along * along.length().recip();
    let radius = pipe.radius();
    let standoff = radius / cone.slope();
    let (across, and) = cone.basis();

    [standoff, -standoff]
        .into_iter()
        .map(|reach| {
            let middle = cone.apex() + up * reach;
            Conic {
                centre: middle,
                across: across * radius,
                and: and * radius,
                size: how_big_this_is(&[middle, cone.apex()], &[radius, standoff]),
            }
        })
        .collect()
}

/// The circle a ring and a ball, a pipe or a cone on its axis share that the
/// run names, drawn as the partner and the circle's own plane draw it.
///
/// # The circles are the namer's, asked again
///
/// [`intersect`](crate::intersect) names each circle by the plane square to
/// the axis it lies in, and `rcad-topo`'s arrangement splits the faces on those
/// planes and picks each circle by the planes halfway to its neighbours
/// (§378.9). So the candidates here are those planes, bit for bit, and each is
/// drawn as `[partner, plane]` — the pair
/// [`which_two_surfaces_a_run_is_between`] reads a run naming that plane between
/// the two faces as. An arc the arrangement names by the ring and the partner
/// and a whole circle a file names by its plane are then one curve, started at
/// one point and walked one way.
///
/// # What the plane costs, and the size that pays for it
///
/// A plane's offset is the middle of an enclosure of the circle's height no
/// wider than sixteen roundings of the larger of the offset and `R + r`, so the
/// circle drawn is on the partner and the plane to their own rounding and off
/// the ring by up to eight of those. The size the walk charges its bound
/// against is widened to the ring's centre and `R + r` for that reason:
/// [`POINT_ALONG_A_CROSSING_ROUNDOFF`] is 256 roundings of it.
///
/// # A whole circle named by the two faces alone is still refused
///
/// A run right round one of these circles that names no plane is refused under
/// the pair's old name, and the reason is `rcad-topo`'s `Model::validate`, not
/// the arithmetic. Validate holds a ring's face to walking its two circles
/// round the axis opposite ways round the tube by the plane the run names
/// (§243.8), so a circle named by the two faces alone is never asked: answered
/// here, a ring's band bounded by a pipe of thirteen with both circles walked
/// the same way — a description that bounds no piece, refused
/// `AClosedEdgeIsWalkedAgainstItsSurface` when the runs name their planes —
/// built and validated (`a_ring_face_bounded_by_a_circle_on_a_coaxial_surface.rs`).
/// The circle is there, so this is a **gap**, kept closed until validate asks
/// its band question of such a circle too. A file names its plane, and so does
/// `intersect`; an arc between two corners is answered.
///
/// # Errors
///
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] where the partner
/// is not proved on the ring's axis, and [`Error::Undecidable`] where a sign
/// could not be settled — `intersect`'s two refusals for the pair, so the
/// namer and the evaluator refuse the same pairs under the same names. The
/// first also for a run right round a circle, as above, and for every run on a
/// pipe of the tube's radius lying along the ring's core circle: `intersect`
/// names a quartic loop there beside the circle the two touch in, nothing draws
/// the loop, and a run's ends do not say which of the two it is on.
///
/// [`Error::TheseTwoSurfacesDoNotCrossInAConic`] where the partner misses the
/// tube, so the run names a circle that is not there: a **limit**, as for a
/// plane that misses a ball.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] as
/// [`which_of_these_circles_this_run_is`] gives them — the second for a run
/// with no end and nothing picking it on a pair that shares two circles or
/// more, which names none of them.
fn where_a_ring_and_a_surface_on_its_axis_cut(
    ring: &Torus,
    partner: Surface,
    run: &NamedRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Conic> {
    if run.run().it_comes_back_to_where_it_started() {
        return Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface);
    }
    let named = match crate::intersect::intersect(&Surface::Torus(*ring), &partner, policy, log)? {
        SurfaceMeet::Curves(circles) => circles,
        SurfaceMeet::Nothing => return Err(Error::TheseTwoSurfacesDoNotCrossInAConic),
        other @ (SurfaceMeet::TouchAtOnePoint { .. } | SurfaceMeet::SameSurface) => {
            unreachable!("a ring and a surface on its axis share circles or nothing: {other:?}")
        }
    };
    let mut candidates = Vec::with_capacity(named.len());
    for circle in named {
        let EdgeRun::RightRoundWhereItCrosses {
            and: own @ Surface::Plane(_),
        } = circle.run()
        else {
            return Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface);
        };
        // A plane square to a pipe's or a cone's axis, or any plane against a
        // ball, crosses it in a conic, and one of no size is refused below in
        // `what_a_run_between_these_two_lies_on` rather than handed back.
        let WhatARunLiesOn::Conic(drawn) =
            what_a_run_between_these_two_lies_on([partner, *own], run, policy, log)?
        else {
            unreachable!("a plane square to the axis of {partner:?} crosses it in a conic");
        };
        candidates.push(Conic {
            size: how_big_this_is(
                &[ring.centre()],
                &[drawn.size, ring.ring_radius() + ring.tube_radius()],
            ),
            ..drawn
        });
    }
    which_of_these_circles_this_run_is(candidates, run, policy, log)
}

/// Which of a handful of candidate conics the run names, told by its ends and,
/// where those do not settle it, by the half-spaces on it.
///
/// **Four shapes of crossing need this and they are not alike.** A plane cuts a
/// torus in two circles, a cone and a pipe on one axis cut two more, a ring and
/// a ball, a pipe or a cone on its axis up to four, and two pipes of one width
/// on axes that meet cut two ellipses; in all four the pair of surfaces names
/// *both* and only the run says which. A run with neither an end nor a picker
/// on it takes the only candidate there is, which is how a plane resting on a
/// torus's tube gets its one circle.
///
/// # The ends answer first, and a picker is only a tie-break
///
/// A picker on a run that came out of a file picks **an arc** and not a curve:
/// `rcad-step` builds it from the chord between the edge's two vertices, so it
/// rules out the far half of the very curve the edge is on. Asked as a curve
/// question — which is what putting the candidate's point at angle zero to it
/// does — it throws away the right answer about half the time. So the ends are
/// asked on their own first and the pickers only run where the ends left more
/// than one candidate standing, which is every run that has no ends at all and
/// so every run `intersect` builds.
///
/// # The pickers are asked at two points of a candidate, half a turn apart
///
/// A candidate is held where its pickers hold its point at angle zero **or** at
/// angle π. For every candidate that lies wholly on one side of each picker
/// the two answers are the same. They differ only for a run that is half of a
/// conic, cut where it crosses another candidate, and that is the four halves
/// two pipes of one width on axes that meet are named as: each ellipse is cut
/// at angles `±π/2`, where it crosses the other, so one half holds angle zero
/// and the other angle π, and each of the four corners that pick them holds
/// one of those two points of one ellipse and neither of the other.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a candidate's own numbers do
/// not come out finite, and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when what is on the run
/// holds more than one of them or none.
fn which_of_these_circles_this_run_is(
    candidates: Vec<Conic>,
    run: &NamedRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Conic> {
    if candidates.iter().any(|circle| {
        !(circle.centre.is_finite() && circle.across.is_finite() && circle.and.is_finite())
    }) {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    }

    let ends: Vec<Point> = run
        .ends()
        .into_iter()
        .filter(|end| *end != nowhere_in_particular())
        .map(|end| Point::from(end.enclosure().map(middle_of)))
        .collect();
    let on_the_ends: Vec<Conic> = candidates
        .into_iter()
        .filter(|circle| {
            let deviation =
                POINT_ALONG_A_CROSSING_ROUNDOFF * circle.the_size_it_was_worked_out_from();
            ends.iter()
                .all(|end| circle.how_far_off_it(*end) <= deviation)
        })
        .collect();
    if let [only] = on_the_ends.as_slice() {
        return Ok(*only);
    }

    let picked_by: &[Picker] = match run.run() {
        EdgeRun::PartOfWhereItCrosses { picked_by, .. } => picked_by,
        EdgeRun::Straight
        | EdgeRun::RightRoundWhereItsFacesCross
        | EdgeRun::RightRoundWhereItCrosses { .. } => &[],
    };
    let mut held: Vec<Conic> = Vec::new();
    for circle in on_the_ends {
        let deviation = POINT_ALONG_A_CROSSING_ROUNDOFF * circle.the_size_it_was_worked_out_from();
        if picked_by.is_empty()
            || every_half_space_holds(picked_by, circle.at(0.0), deviation, policy, log)?
            || every_half_space_holds(
                picked_by,
                circle.at(core::f64::consts::PI),
                deviation,
                policy,
                log,
            )?
        {
            held.push(circle);
        }
    }
    match held.as_slice() {
        [only] => Ok(*only),
        _ => Err(Error::CannotNameWhichPieceOfACrossingThisIs),
    }
}

/// The circle a closed run whose two faces are the one cone lies on: through
/// the run's own end, square to the cone's axis.
///
/// **Which spelling of the cone it is handed changes which way round the circle
/// goes, and nothing else.** Turning the axis round negates both `up` and
/// `reach` below, so `middle`, `radius` and the size come out the same bits,
/// and [`Cone::basis`] keeps its first direction across and negates its second.
/// So the circle is one point set and starts at one point either way, and runs
/// right-handed about whichever axis it was handed — which is why the caller
/// hands it the spelling that sorts first
/// (`a_circle_walked_between_two_spellings_of_one_cone.rs`).
///
/// **The pair of surfaces names nothing here**, the way a ball crossed with
/// itself names nothing, so what names the curve is the run. It is closed, so
/// its two ends are one point; the closed curve on a cone through a point and
/// square to the axis is the circle centred where the axis is level with that
/// point, of the radius it stands out by. Nothing about it is degree four.
///
/// # What the run does not say, and what that costs
///
/// Which plane made the cut. A plane leaning across the axis cuts a cone in an
/// **ellipse**, which is closed too, and a run that came off such a cut carries
/// nothing here to tell it from a circle — so this names the circle and is wrong
/// about that curve. §214.4's fix (d) is what would tell them apart, by writing
/// the cutting plane into the run; this is (a), which is smaller and is what
/// §214.4 recommends. The one family measured against it moved no answers, only
/// refusals: `how_often_a_fuse_with_a_cone_refuses.rs`.
///
/// # Errors
///
/// [`Error::ACrossingPieceHasNoEndsAndNothingPickingIt`] for a closed run whose
/// ends name nowhere at all. Nothing in this crate builds such a run — a
/// [`NamedRun`] with no ends comes out of [`intersect`](crate::intersect), which
/// answers this pair as one surface and never names a run for it — but [`walk`]
/// is public and takes one.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when the circle's own numbers do
/// not come out finite.
fn the_circle_a_closed_run_round_one_cone_lies_on(cone: &Cone, run: &NamedRun) -> Result<Conic> {
    let Some(end) = run
        .ends()
        .into_iter()
        .find(|end| *end != nowhere_in_particular())
        .map(|end| Point::from(end.enclosure().map(middle_of)))
    else {
        return Err(Error::ACrossingPieceHasNoEndsAndNothingPickingIt);
    };

    let along = cone.axis().direction();
    let up = along * along.length().recip();
    let reach = (end - cone.apex()).dot(up);
    let middle = cone.apex() + up * reach;
    let radius = (end - middle).length();
    let (across, and) = cone.basis();
    let circle = Conic {
        centre: middle,
        across: across * radius,
        and: and * radius,
        size: how_big_this_is(&[middle, end, cone.apex()], &[radius, reach]),
    };
    if !(circle.centre.is_finite() && circle.across.is_finite() && circle.and.is_finite()) {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    }
    Ok(circle)
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

/// The box a computed point is really known to be inside, or `None` when there
/// is no such box.
///
/// Handed to a predicate as an exact triple instead, a point that came out of
/// trigonometry buys a certified sign about noise (§2.5, §27.7).
///
/// **A sign is not the whole of what an interval refuses**, and the `expect`
/// this used to carry said it was: "a deviation is a positive length, so the low
/// end is below the high one" reasons about the *order* of two numbers and is
/// silent about the case where they are not ordered at all. A two-hour
/// `cargo fuzz` soak of `Model::fuse` walked a seam whose conic had shrunk to a
/// point, so both ends came out `NaN`, and a claim about signs cannot cover
/// that. Nor can a wider one be earned here: the family of fixes is a claim, an
/// assert, or an answer, and the first two are the same bet twice over — this
/// runs on a stranger's bytes, and `walk` already hands its caller an `Err`.
/// So the answer is `None` and the caller names the refusal.
///
/// **The ends are rounded outwards**, which plain subtraction does not do:
/// nearest rounding pulls each end up to half an ulp inwards, and where the
/// deviation is under half an ulp of the coordinate both ends land back on the
/// coordinate and the side stops being a side. A crossing a metre across, walked
/// sixteen million metres from the origin, is inside what this kernel is for
/// (`docs/decisions.md` §85).
fn somewhere_within(point: Point, deviation: f64) -> Option<PredicatePoint> {
    let [x, y, z] = point.coordinates();
    let side = |coordinate: f64| Interval::around(coordinate, deviation).ok();
    Some(PredicatePoint::Constructed([side(x)?, side(y)?, side(z)?]))
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

    /// A two-to-one ellipse in the `xy` plane, five wide and two and a half up.
    fn a_stretched_ellipse() -> Conic {
        Conic {
            centre: Point::origin(),
            across: Vector::new(5.0, 0.0, 0.0),
            and: Vector::new(0.0, 2.5, 0.0),
            size: 5.0,
        }
    }

    /// The end of the short axis is **on** the ellipse, and the circle formula
    /// read it as two and a half out.
    ///
    /// The whole of what stopped two bores of one width from being drawn once
    /// the closed form had built them: `which_of_these_circles_this_run_is` asks
    /// each candidate whether it holds the run's ends, and the answer for an
    /// ellipse was a distance to a circle of its long radius.
    #[test]
    fn a_point_on_an_ellipses_short_axis_is_on_the_ellipse() {
        let ellipse = a_stretched_ellipse();
        assert!(ellipse.how_far_off_it(Point::new(0.0, 2.5, 0.0)) < 1e-15);
        assert!(ellipse.how_far_off_it(Point::new(5.0, 0.0, 0.0)) < 1e-15);
        assert!(ellipse.how_far_off_it(Point::new(0.0, 0.0, 1.0)) > 1.0);
    }

    /// A point at an ellipse's own centre has no gradient to divide by, and it
    /// is not on the ellipse.
    #[test]
    fn a_point_at_an_ellipses_own_centre_is_not_on_it() {
        assert_eq!(a_stretched_ellipse().how_far_off_it(Point::origin()), 5.0);
    }

    /// A circle goes on reading exactly what it read before.
    #[test]
    fn the_circle_arm_still_takes_the_plain_distance() {
        let circle = Conic {
            centre: Point::origin(),
            across: Vector::new(3.0, 0.0, 0.0),
            and: Vector::new(0.0, 3.0, 0.0),
            size: 3.0,
        };
        assert_eq!(circle.how_far_off_it(Point::new(4.0, 0.0, 0.0)), 1.0);
        assert_eq!(circle.how_far_off_it(Point::origin()), 3.0);
    }

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

    /// A cross product of two unit triples comes out zero in `f64` while its
    /// true value is not zero, so `n × w == 0` is not an exact question.
    ///
    /// `where_a_plane_cuts_a_torus` used to say it was, on the grounds that
    /// every component of a cross product is `a·d − b·c` with no sum to cancel.
    /// That is true of the shape of the expression and says nothing about the
    /// rounding: `a·d` and `b·c` each round on their own, and two roundings that
    /// land on the same `f64` leave a difference of exactly zero. Below, two
    /// unit normals one `f64` apart — both straight out of
    /// [`Plane::from_point_and_normal`], so both are triples a model really
    /// holds — cross to `(0, 0, 0)` in `f64` with a true second component near
    /// `−2.1e-17` and a third near `1.8e-17`.
    ///
    /// The pair was `(1, 2, 6)` against `(1, 2, 6.next_up())` until §121 rounded
    /// the stored normal correctly; those two then crossed to `5.6e-17` in `f64`
    /// and stopped witnessing anything. `(2, 6, 7)` against
    /// `(2, 6, 7.next_up())` is one of the twenty-odd pairs of whole numbers
    /// under ten that still do it, so **the defect did not move with its
    /// witness.**
    ///
    /// What that leaves standing is the argument in
    /// `where_a_plane_cuts_a_torus`'s own doc: the two families it separates are
    /// a whole unit apart, so a test that cannot see `1e-17` still tells them
    /// apart. This pins the counterexample so the false reason cannot come back.
    #[test]
    fn a_cross_product_of_unit_triples_can_round_to_zero() {
        let unit = |normal: Vector| {
            Plane::from_point_and_normal(Point::origin(), normal)
                .expect("a normal with length")
                .normal()
        };
        let one = unit(Vector::new(2.0, 6.0, 7.0));
        let other = unit(Vector::new(2.0, 6.0, 7.0f64.next_up()));

        assert_ne!(one, other, "these are two different unit triples");
        assert_eq!(
            one.cross(other),
            Vector::zero(),
            "the f64 cross product of {one:?} and {other:?} is not zero after all"
        );

        let [p, q, r] = one.components();
        let [u, v, w] = other.components();
        let truly = [
            Exact::product_of(q, w).and_then(|it| Some(it.minus(&Exact::product_of(r, v)?))),
            Exact::product_of(r, u).and_then(|it| Some(it.minus(&Exact::product_of(p, w)?))),
            Exact::product_of(p, v).and_then(|it| Some(it.minus(&Exact::product_of(q, u)?))),
        ];
        assert!(
            truly.iter().any(|component| matches!(
                component
                    .as_ref()
                    .expect("unit triples multiply in range")
                    .sign(),
                Decision::Pos | Decision::Neg
            )),
            "the true cross product is zero too, so this is not a counterexample"
        );
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
            )
            .expect("a picked run does not have to come back to where it started");
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

    /// A quarter circle on a flat face, with a flat face on both sides that are
    /// the same plane written two ways, is walked round the circle its run's
    /// cylinder names — not read as the straight line two planes cross in.
    ///
    /// The two normals are `(0, 1, −1.158e-15)` and `(5.6e-17, 1, 0)`, the pair
    /// `docs/decisions.md` §176.2 measured on a KiCad LED. Unequal by value, so
    /// the seam rule alone never read the run's own surface.
    #[test]
    fn two_flat_faces_a_hair_apart_are_walked_along_the_cylinder_their_run_names() {
        let face = |normal: Vector| {
            Surface::Plane(
                Plane::from_point_and_normal(Point::new(0.0, -1.27, 0.0), normal)
                    .expect("a normal with a length"),
            )
        };
        let one = face(Vector::new(0.0, 1.0, -1.158_067_919_543e-15));
        let other = face(Vector::new(5.551_115_123_126e-17, 1.0, 0.0));
        assert_ne!(one, other, "the two faces are two different values");

        let radius = 1.5;
        let centre = Point::new(1.27, -1.27, -10.0);
        let round = Surface::Cylinder(
            crate::Cylinder::new(
                crate::Axis::new(centre, Vector::new(0.0, 1.0, 0.0)).expect("an axis"),
                radius,
            )
            .expect("a pipe"),
        );
        let from = Point::new(1.27 + radius, -1.27, -10.0);
        let to = Point::new(1.27, -1.27, -10.0 - radius);
        let picking = crate::HalfSpace::through_a_point_facing(
            from,
            (to - from).cross(Vector::new(0.0, 1.0, 0.0)),
        )
        .expect("a chord with a length");
        let run = NamedRun::new(
            EdgeRun::PartOfWhereItCrosses {
                and: round,
                picked_by: smallvec![Picker::from(picking)],
            },
            [from, to].map(|end| PredicatePoint::from(end.coordinates())),
        )
        .expect("a picked run");

        let walked = walk(
            &run,
            [&one, &other],
            Deflection::of(0.001).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a quarter circle is walked");

        assert!(
            walked.points().len() > 4,
            "a quarter circle at a thousandth came back with {} points",
            walked.points().len()
        );
        for point in walked.points() {
            let off = (*point - centre).cross(Vector::new(0.0, 1.0, 0.0)).length() - radius;
            assert!(off.abs() < 1e-12, "{point:?} is {off} off the circle");
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
                &NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
                    .expect("a closed curve is never a piece with nothing picking it"),
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
        let whole = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it");
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

    /// The one vertex a closed edge needs goes where `walk` starts, and that
    /// place is the same whichever of the edge's two faces asks for it.
    ///
    /// **Two balls used to fail this and not by a rounding.**
    /// `where_two_balls_cut` builds the radical plane through the gap between
    /// the centres, so the pair the other way round got that plane reversed, a
    /// different `basis` out of it, and a start a quarter of the way round the
    /// circle. Forty of forty swapped pairs walked into different points. §31.3
    /// leans on this being order-free and it was not.
    #[test]
    fn a_closed_crossing_starts_at_the_same_place_whichever_of_its_faces_asks() {
        let whole = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it");
        let start = |along: [&Surface; 2]| {
            where_a_closed_crossing_starts(&whole, along, &mut ProvenanceLog::new())
                .expect("these pairs all cross")
                .coordinates()
        };
        for [one, other] in every_pair_that_crosses_in_a_closed_curve() {
            assert_eq!(
                start([&one, &other]),
                start([&other, &one]),
                "{one:?} and {other:?} start their closed crossing in two places"
            );
        }
    }

    /// The four pairs whose crossing is a conic walk into the same points
    /// whichever face asks, and the two degree-four pairs walk the same loop
    /// **the other way round**.
    ///
    /// The difference is real and it is the marcher's: a conic is parametrized
    /// from a frame the unordered pair fixes, and a marched loop is followed
    /// along `∇S × ∇T`, which flips when the two surfaces are swapped. Neither
    /// is a bug and §31.3 already says which of the two faces walks a closed
    /// edge forward is decided by the material on its left. It is written down
    /// here because the two halves of one function behaving differently is
    /// exactly the thing a caller assumes away.
    #[test]
    fn a_conic_walks_the_same_way_round_whichever_face_asks_and_a_marched_loop_does_not() {
        let whole = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it");
        let walked = |along: [&Surface; 2]| {
            walk(
                &whole,
                along,
                Deflection::of(0.05).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("these pairs all cross")
            .points()
            .to_vec()
        };

        for [one, other] in every_pair_that_crosses_in_a_closed_curve() {
            let (forwards, backwards) = (walked([&one, &other]), walked([&other, &one]));
            let marched = matches!(
                what_a_run_between_these_two_lies_on(
                    [one, other],
                    &whole,
                    &Policy::certified(),
                    &mut ProvenanceLog::new()
                ),
                Ok(WhatARunLiesOn::QuarticPickedUpAt(..))
            );
            if marched {
                let seed = where_a_closed_crossing_starts(
                    &whole,
                    [&one, &other],
                    &mut ProvenanceLog::new(),
                )
                .expect("these pairs all cross");
                let step = |points: &[Point]| points[0] - seed;
                assert!(
                    step(&forwards).dot(step(&backwards)) < 0.0,
                    "{one:?} and {other:?} left their seed the same way round both times"
                );
            } else {
                assert_eq!(
                    forwards, backwards,
                    "{one:?} and {other:?} walk into different points when swapped"
                );
            }
        }
    }

    /// Where a closed crossing starts is a point on both the surfaces that named
    /// it, which is what makes it fit to be a vertex.
    #[test]
    fn where_a_closed_crossing_starts_is_on_both_the_surfaces_that_named_it() {
        let whole = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it");
        for [one, other] in every_pair_that_crosses_in_a_closed_curve() {
            let start =
                where_a_closed_crossing_starts(&whole, [&one, &other], &mut ProvenanceLog::new())
                    .expect("these pairs all cross");
            for surface in [&one, &other] {
                assert!(
                    how_far_off(surface, start) < 1e-12,
                    "the start of the crossing of {one:?} and {other:?} sits {} off {surface:?}",
                    how_far_off(surface, start)
                );
            }
        }
    }

    /// A crossing that runs on for ever has no place on it the pair alone picks
    /// out, and says so rather than putting a vertex at an arbitrary spot.
    #[test]
    fn a_crossing_that_is_not_closed_has_no_place_to_put_a_vertex() {
        let ground = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("a unit normal"),
        );
        let wall = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
                .expect("a unit normal"),
        );
        let pipe = Surface::Cylinder(Cylinder::new(crate::Axis::z(), 3.0).expect("a radius"));
        let beside_it = Surface::Cylinder(
            Cylinder::new(
                crate::Axis::new(Point::new(4.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
                    .expect("a direction"),
                3.0,
            )
            .expect("a radius"),
        );
        let straight = NamedRun::the_whole_of(EdgeRun::Straight)
            .expect("a straight run is never a piece with nothing picking it");

        for [one, other] in [[&ground, &wall], [&wall, &pipe], [&pipe, &beside_it]] {
            assert_eq!(
                where_a_closed_crossing_starts(&straight, [one, other], &mut ProvenanceLog::new()),
                Err(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed),
                "{one:?} and {other:?} cross in straight lines"
            );
        }
    }

    /// One of each pair whose crossing is a closed curve: the four conic ones
    /// and the two degree-four ones.
    fn every_pair_that_crosses_in_a_closed_curve() -> Vec<[Surface; 2]> {
        let tilted = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 1.0), Vector::new(1.0, 2.0, 3.0))
                .expect("a direction with length"),
        );
        let ball = Surface::Sphere(Sphere::new(Point::new(0.5, 0.0, 0.0), 6.0).expect("a radius"));
        let bigger =
            Surface::Sphere(Sphere::new(Point::new(3.0, 1.0, 0.5), 7.0).expect("a radius"));
        let pipe = Surface::Cylinder(
            Cylinder::new(
                crate::Axis::new(Point::origin(), Vector::new(1.0, 1.0, 4.0)).expect("a direction"),
                2.0,
            )
            .expect("a radius"),
        );
        let leaning = Surface::Cylinder(
            Cylinder::new(
                crate::Axis::new(Point::new(0.0, 1.0, 0.0), Vector::new(3.0, 1.0, 0.0))
                    .expect("a direction"),
                2.0,
            )
            .expect("a radius"),
        );
        let same_size = Surface::Cylinder(
            Cylinder::new(
                crate::Axis::new(Point::new(0.0, 1.0, 0.0), Vector::new(3.0, 1.0, 0.0))
                    .expect("a direction"),
                2.0,
            )
            .expect("a radius"),
        );
        vec![
            [tilted, ball],
            [tilted, pipe],
            [ball, bigger],
            [ball, pipe],
            [pipe, leaning],
            // Two pipes of exactly the same radius, where which of them is
            // "thinner" is a tie the stored numbers have to break.
            [pipe, same_size],
        ]
    }

    /// How far a point sits off a surface, in the metric that surface's own
    /// stored numbers name.
    fn how_far_off(surface: &Surface, point: Point) -> f64 {
        match surface {
            Surface::Plane(flat) => {
                (flat.normal().dot(point - Point::origin()) - flat.offset()).abs()
                    / flat.normal().length()
            }
            Surface::Sphere(ball) => ((point - ball.centre()).length() - ball.radius()).abs(),
            Surface::Cylinder(pipe) => {
                let along = pipe.axis().direction();
                ((point - pipe.axis().origin()).cross(along).length() / along.length()
                    - pipe.radius())
                .abs()
            }
            Surface::Cone(cone) => {
                let along = cone.axis().direction();
                let out = point - cone.apex();
                (out.cross(along).length() - cone.slope() * out.dot(along).abs()).abs()
                    / (along.length() * cone.slope().hypot(1.0))
            }
            Surface::Torus(ring) => {
                let along = ring.axis().direction();
                let out = point - ring.centre();
                let across = out.cross(along).length() / along.length() - ring.ring_radius();
                let axial = out.dot(along) / along.length();
                (across.hypot(axial) - ring.tube_radius()).abs()
            }
        }
    }

    /// A ball through the side of a pipe leaves one loop, and a file that puts
    /// that loop's corner away from where the marcher starts still gets points
    /// that begin and end beside the corner (`docs/decisions.md` §189.3).
    ///
    /// The marcher starts at `(2, 0, ±√3)`, on the side the ball leans to. The
    /// corner here is where the loop crosses `z = 0`, at `cos θ = 1/4` round the
    /// pipe: `(2cos θ − 1)² + 4sin²θ + z² = 4` is `z² = 4cos θ − 1`. It is nearly
    /// three away from the start, and the walk at a hundredth steps far less than
    /// that.
    #[test]
    fn a_closed_loop_a_file_puts_its_corner_on_is_walked_from_that_corner() {
        let pipe = Surface::Cylinder(Cylinder::new(crate::Axis::z(), 2.0).expect("a radius"));
        let ball = Surface::Sphere(Sphere::new(Point::new(1.0, 0.0, 0.0), 2.0).expect("a radius"));
        let corner = Point::new(0.5, (4.0_f64 - 0.25).sqrt(), 0.0);
        let run = NamedRun::new(
            EdgeRun::RightRoundWhereItsFacesCross,
            [corner, corner].map(|end| PredicatePoint::from(end.coordinates())),
        )
        .expect("a closed run with one corner");

        let walked = walk(
            &run,
            [&ball, &pipe],
            Deflection::of(0.01).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("one loop of a ball through a pipe is walked");

        let points = walked.points();
        let longest_step = points
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).length())
            .fold(0.0, f64::max);
        let [first, last] = [points[0], points[points.len() - 1]];
        for (which, end) in [("first", first), ("last", last)] {
            let gap = (end - corner).length();
            assert!(
                gap <= longest_step,
                "the {which} point is {gap} from the corner, past the longest step {longest_step}"
            );
        }
    }

    /// A ball of radius four centred on a pipe of radius one crosses it in two
    /// circles, at `z = ±√15`, and a closed run whose corner is on either of
    /// them is walked round that one, whichever face is listed first.
    ///
    /// A fuse writes each circle of its answer as a closed run with its corner
    /// on it and nothing picking it. The marcher started every such run at the
    /// first of its two seeds, the one ahead along the axis, so the circle
    /// behind the centre was walked round the circle ahead, and a ball with a
    /// bar through its centre was refused by its own mesh (Q1241).
    #[test]
    fn a_closed_run_round_either_circle_of_a_ball_on_a_pipes_axis_is_walked_round_that_one() {
        let pipe = Surface::Cylinder(Cylinder::new(crate::Axis::z(), 1.0).expect("a radius"));
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 4.0).expect("a radius"));
        let height = 15.0_f64.sqrt();
        for level in [height, -height] {
            let corner = Point::new(1.0, 0.0, level);
            let run = NamedRun::new(
                EdgeRun::RightRoundWhereItsFacesCross,
                [corner, corner].map(|end| PredicatePoint::from(end.coordinates())),
            )
            .expect("a closed run with one corner");
            for pair in [[&ball, &pipe], [&pipe, &ball]] {
                let walked = walk(
                    &run,
                    pair,
                    Deflection::of(0.01).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a circle of a ball on a pipe's axis is walked");
                let off = walked.deviation().magnitude();
                assert!(
                    walked.points().len() > 2,
                    "the circle at {level} has points"
                );
                for point in walked.points() {
                    let [x, y, z] = point.coordinates();
                    assert!(
                        (z - level).abs() <= off
                            && how_far_off(&ball, *point) <= off
                            && how_far_off(&pipe, *point) <= off,
                        "the run round the circle at {level} was walked through ({x}, {y}, {z})"
                    );
                }
            }
        }
    }

    /// A ball of radius four a tenth off the axis of a pipe of radius one
    /// crosses it in two loops, one either side of the plane through the ball's
    /// centre square to the axis, and a closed run whose corner is on the loop
    /// below is walked round that one, whichever face is listed first.
    ///
    /// The marcher has a seed on each loop and started every run nothing picks
    /// at the first, the one above, so this run was walked round the loop above
    /// (Q1262). The corner is on the side of the pipe away from where the ball
    /// leans, where `(x − 0.1)² + z² = 16` puts it at `z = −√14.79`, so it is
    /// not either seed.
    #[test]
    fn a_closed_run_round_the_lower_loop_of_a_ball_off_a_pipes_axis_is_walked_round_that_one() {
        let pipe = Surface::Cylinder(Cylinder::new(crate::Axis::z(), 1.0).expect("a radius"));
        let ball = Surface::Sphere(Sphere::new(Point::new(0.1, 0.0, 0.0), 4.0).expect("a radius"));
        let corner = Point::new(-1.0, 0.0, -(14.79_f64.sqrt()));
        let run = NamedRun::new(
            EdgeRun::RightRoundWhereItsFacesCross,
            [corner, corner].map(|end| PredicatePoint::from(end.coordinates())),
        )
        .expect("a closed run with one corner");
        for pair in [[&ball, &pipe], [&pipe, &ball]] {
            let walked = walk(
                &run,
                pair,
                Deflection::of(0.01).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a loop of a ball off a pipe's axis is walked");
            assert!(walked.points().len() > 2, "the loop below has points");
            for point in walked.points() {
                let [x, y, z] = point.coordinates();
                assert!(
                    z < 0.0,
                    "the run round the loop below was walked through ({x}, {y}, {z})"
                );
            }
        }
    }

    /// A thin pipe right through a fat one crosses it in two loops, one where
    /// it goes in and one where it comes out, and a closed run whose corner is
    /// on the second is walked round that one, whichever face is listed first.
    ///
    /// The fat pipe is radius four up `z` and the thin one radius one along `x`
    /// half a unit off it, so the loops are either side of `x = 0`. The seed on
    /// the `+x` loop comes first (Q1262). The corner is where the thin pipe's
    /// top line, `y = 0.5, z = 1`, meets the fat wall at `x = −√15.75`.
    #[test]
    fn a_closed_run_round_the_far_loop_of_a_thin_pipe_through_a_fat_one_is_walked_round_that_one() {
        let fat = Surface::Cylinder(Cylinder::new(crate::Axis::z(), 4.0).expect("a radius"));
        let thin = Surface::Cylinder(
            Cylinder::new(
                crate::Axis::new(Point::new(0.0, 0.5, 0.0), Vector::new(1.0, 0.0, 0.0))
                    .expect("a direction"),
                1.0,
            )
            .expect("a radius"),
        );
        let corner = Point::new(-(15.75_f64.sqrt()), 0.5, 1.0);
        let run = NamedRun::new(
            EdgeRun::RightRoundWhereItsFacesCross,
            [corner, corner].map(|end| PredicatePoint::from(end.coordinates())),
        )
        .expect("a closed run with one corner");
        for pair in [[&fat, &thin], [&thin, &fat]] {
            let walked = walk(
                &run,
                pair,
                Deflection::of(0.01).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a loop of a thin pipe through a fat one is walked");
            assert!(walked.points().len() > 2, "the far loop has points");
            for point in walked.points() {
                let [x, y, z] = point.coordinates();
                assert!(
                    x < 0.0,
                    "the run round the far loop was walked through ({x}, {y}, {z})"
                );
            }
        }
    }

    /// A thin pipe along `x` at `z = 5` right through a fat one up `z`, cut by
    /// the wall `y = 0` through both axes: the wall puts corners at
    /// `(−4, 0, 4)` and `(−4, 0, 6)` on the `−x` loop, and a run between them
    /// picked by `y ≥ 0` or `y ≤ 0` is walked along that half of that loop only,
    /// from its first corner to its second, either way round and whichever face
    /// is listed first.
    ///
    /// Both loops' seeds are on the wall, so `y ≥ 0` held both and the run was
    /// refused; picked by its corners, the whole loop came back (Q1659). The
    /// seed of the `−x` loop is `(−4, 0, 6)`, one of the corners, so a point
    /// walked on a corner is checked for too: it is on neither half.
    #[test]
    fn a_run_between_two_corners_of_a_loop_of_a_thin_pipe_through_a_fat_one_is_walked_along_its_half()
     {
        let fat = Surface::Cylinder(Cylinder::new(crate::Axis::z(), 4.0).expect("a radius"));
        let thin = Surface::Cylinder(
            Cylinder::new(
                crate::Axis::new(Point::new(-10.0, 0.0, 5.0), Vector::new(1.0, 0.0, 0.0))
                    .expect("a direction"),
                1.0,
            )
            .expect("a radius"),
        );
        let corners = [Point::new(-4.0, 0.0, 4.0), Point::new(-4.0, 0.0, 6.0)];
        for side in [1.0, -1.0] {
            let picker = Picker::from(
                crate::HalfSpace::through_a_point_facing(
                    Point::origin(),
                    Vector::new(0.0, side, 0.0),
                )
                .expect("the y axis has length"),
            );
            for [from, to] in [corners, [corners[1], corners[0]]] {
                let run = NamedRun::new(
                    EdgeRun::PartOfWhereItCrosses {
                        and: thin,
                        picked_by: smallvec![picker],
                    },
                    [from, to].map(|end| PredicatePoint::from(end.coordinates())),
                )
                .expect("a run between two corners");
                for pair in [[&fat, &thin], [&thin, &fat]] {
                    let walked = walk(
                        &run,
                        pair,
                        Deflection::of(0.01).expect("a length"),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .expect("half a loop of a thin pipe through a fat one is walked");
                    let points = walked.points();
                    assert!(points.len() > 2, "half the loop has points");
                    for point in points {
                        let [x, y, z] = point.coordinates();
                        assert!(
                            x < 0.0 && y * side > 0.0,
                            "the run on y·{side} > 0 from {from:?} was walked through ({x}, {y}, {z})"
                        );
                    }
                    let [first, last] = [points[0], points[points.len() - 1]];
                    assert!(
                        (first - from).length() < (first - to).length()
                            && (last - to).length() < (last - from).length(),
                        "the run from {from:?} to {to:?} was walked from {first:?} to {last:?}"
                    );
                }
            }
        }
    }

    /// A run between two corners closer together than one step of the march
    /// has no marched point on it, and is still told from the rest of its loop:
    /// the half-spaces are asked at the middle of its two corners, settled onto
    /// the crossing, and nothing comes back between the corners.
    ///
    /// The loop is the `−x` one of the thin pipe through the fat one above, and
    /// the second corner is the box `1e-9` round where `y = 1/20` on its lower
    /// half: `x = −√(16 − y²)`, `z = 5 − √(1 − y²)`. The run is picked by
    /// `y ≥ 0` and `z ≤ 4.5`. By `y ≥ 0` alone it is refused, though the rest
    /// of the loop dips below the wall: the middle point of the rest is the
    /// seed `(−4, 0, 6)`, on the wall, so `y ≥ 0` holds at both arcs' middles.
    #[test]
    fn a_run_shorter_than_a_step_of_the_march_is_told_from_the_rest_of_its_loop() {
        let fat = Surface::Cylinder(Cylinder::new(crate::Axis::z(), 4.0).expect("a radius"));
        let thin = Surface::Cylinder(
            Cylinder::new(
                crate::Axis::new(Point::new(-10.0, 0.0, 5.0), Vector::new(1.0, 0.0, 0.0))
                    .expect("a direction"),
                1.0,
            )
            .expect("a radius"),
        );
        let y = 0.05_f64;
        let near = Point::new(-(16.0 - y * y).sqrt(), y, 5.0 - (1.0 - y * y).sqrt());
        let ends = [
            PredicatePoint::from(Point::new(-4.0, 0.0, 4.0).coordinates()),
            somewhere_within(near, 1e-9).expect("a box round a point"),
        ];
        let over = Picker::from(
            crate::HalfSpace::through_a_point_facing(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis has length"),
        );
        let low = Picker::from(
            crate::HalfSpace::through_a_point_facing(
                Point::new(0.0, 0.0, 4.5),
                Vector::new(0.0, 0.0, -1.0),
            )
            .expect("the z axis has length"),
        );
        let run = NamedRun::new(
            EdgeRun::PartOfWhereItCrosses {
                and: thin,
                picked_by: smallvec![over, low],
            },
            ends,
        )
        .expect("a run between two corners");
        let walked = walk(
            &run,
            [&fat, &thin],
            Deflection::of(1.0).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a short run of a thin pipe through a fat one is walked");
        assert_eq!(walked.points(), &[] as &[Point]);
    }

    /// The quarter of a circle the Fujitsu FTR-F1 relays put where a pin's pipe
    /// meets the ball of its own radius that rounds its end, from
    /// `#2404 = (10.05, 1.15, 16.5)` to `#1030 = (10.15, 1.15, 16.4)`, is
    /// refused: the ball and the pipe cross in one thin loop over the half of
    /// the ball away from `#1030`, and `#1030` is `0.14` from every point of it
    /// (§335.5).
    ///
    /// Picked by its corners, the arc was walked at `0.1`, `0.2` and `1.0`
    /// along the file's rim, whose points are `0.14` off the thin loop at
    /// `#1030`'s end, with a deviation of about `1e-6` claimed for them, and
    /// both relay files imported a curve their ball and pipe do not cross in
    /// (Q1659). At `0.001`, `0.01` and `0.4` the march gave up instead.
    /// `#1030` cannot be settled onto both surfaces at all, since they are
    /// tangent there, so it is asked first and the answer is one refusal at
    /// every deflection. The ball, the leaning pipe and the half-space on the
    /// chord are the file's, as the reader names the arc.
    #[test]
    fn a_quarter_circle_ending_where_a_ball_and_its_own_pipe_do_not_cross_is_refused() {
        let ball =
            Surface::Sphere(Sphere::new(Point::new(10.05, 1.15, 16.4), 0.1).expect("a radius"));
        let pipe = Surface::Cylinder(
            Cylinder::new(
                crate::Axis::new(
                    Point::new(10.05, -19.75, 16.4),
                    Vector::new(3.663_735_981_263e-15, 1.0, 0.0),
                )
                .expect("a direction"),
                0.1,
            )
            .expect("a radius"),
        );
        let corners = [Point::new(10.05, 1.15, 16.5), Point::new(10.15, 1.15, 16.4)];
        let on_the_chord = Picker::from(
            crate::HalfSpace::through_a_point_facing(corners[0], Vector::new(0.1, 0.0, 0.1))
                .expect("the chord crosses the axis"),
        );
        let run = NamedRun::new(
            EdgeRun::PartOfWhereItCrosses {
                and: pipe,
                picked_by: smallvec![on_the_chord],
            },
            corners.map(|end| PredicatePoint::from(end.coordinates())),
        )
        .expect("a run between two corners");
        for deflection in [0.001, 0.01, 0.1, 0.2, 0.4, 1.0] {
            let walked = walk(
                &run,
                [&ball, &pipe],
                Deflection::of(deflection).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .map(|walked| walked.points().len());
            assert!(
                matches!(
                    walked,
                    Err(Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
                        at: crate::WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces,
                        ..
                    })
                ),
                "#1030 sits where the ball and the pipe are tangent, and the arc to it was \
                 walked at {deflection} as {walked:?}"
            );
        }
    }

    /// A fuse puts a closed loop's corner where the marcher starts
    /// ([`where_a_closed_crossing_starts`]), and the caller puts that corner at
    /// both ends of what comes back — so nothing walked may sit on it, nor a
    /// rounding off it. A mesh shifts each corner into its own frame and the
    /// seed is worked out again there, which can land the two an ulp apart, and
    /// a loop that held both was refused as two corners in one place (Q1508).
    #[test]
    fn a_closed_loop_started_at_its_own_corner_holds_that_corner_only_at_its_ends() {
        let pipe = Surface::Cylinder(Cylinder::new(crate::Axis::z(), 2.0).expect("a radius"));
        let ball = Surface::Sphere(Sphere::new(Point::new(1.0, 0.0, 0.0), 2.0).expect("a radius"));
        let whole = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it");
        let start =
            where_a_closed_crossing_starts(&whole, [&ball, &pipe], &mut ProvenanceLog::new())
                .expect("one loop of a ball through a pipe starts somewhere");
        let [x, y, z] = start.coordinates();
        for corner in [start, Point::new(x.next_up(), y, z)] {
            let run = NamedRun::new(
                EdgeRun::RightRoundWhereItsFacesCross,
                [corner, corner].map(|end| PredicatePoint::from(end.coordinates())),
            )
            .expect("a closed run with one corner");
            for deflection in [0.01, 0.1] {
                let walked = walk(
                    &run,
                    [&ball, &pipe],
                    Deflection::of(deflection).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("one loop of a ball through a pipe is walked");
                let nearest = walked
                    .points()
                    .iter()
                    .map(|point| (*point - corner).length())
                    .fold(f64::INFINITY, f64::min);
                assert!(
                    nearest > 1e-6,
                    "at {deflection} a point walked is {nearest} from the corner {corner:?}"
                );
            }
        }
    }

    /// A thin pipe of radius one slanting through a fat one of radius four,
    /// its axis `2.9` or `2.99` from the fat one's, leaves two loops that
    /// nearly touch, and a closed run whose corner is on either is walked round
    /// that one, whichever face is listed first.
    ///
    /// The naming half refuses to name these two loops apart: of the flat
    /// surfaces it can store square to the fat pipe's axis, none leans far
    /// enough towards the thin pipe's way across it. The walk asked which side
    /// of that stored surface the corner was on, and the surface cuts through
    /// the loop on the `−` side, so a run with its corner on that loop was
    /// walked round the other one with none of its points on its own (Q1520).
    /// The loops are either side of `â·p = 0`, `â` the thin pipe's way across
    /// the fat one's axis, and the corner is where the thin pipe's line
    /// furthest from that axis meets the fat wall.
    #[test]
    fn a_closed_run_round_either_of_two_nearly_touching_loops_of_slanted_pipes_is_walked_round_that_one()
     {
        for (thin_way, apart) in [
            (Vector::new(0.3, -0.2, 0.9), 2.9),
            (Vector::new(0.3, -0.2, 0.9), 2.99),
            (Vector::new(0.7, 0.1, 0.7), 2.99),
            (Vector::new(0.1, 0.5, -0.8), 2.99),
        ] {
            either_nearly_touching_loop_is_walked_round_that_one(thin_way, apart);
        }
    }

    fn either_nearly_touching_loop_is_walked_round_that_one(thin_way: Vector, apart: f64) {
        let fat_way = Vector::new(1.0, 1.0, 0.0);
        let [thin_radius, fat_radius] = [1.0, 4.0];
        let w1 = thin_way.normalized().expect("a direction");
        let w2 = fat_way.normalized().expect("a direction");
        let off_the_fat_axis = w1.cross(w2).normalized().expect("axes that cross");
        let leaning = w1.dot(w2);
        let across = (w1 - w2 * leaning).normalized().expect("axes that cross");
        let thin_origin = Point::origin() + off_the_fat_axis * apart;
        let fat = Surface::Cylinder(
            Cylinder::new(
                crate::Axis::new(Point::origin(), fat_way).expect("a direction"),
                fat_radius,
            )
            .expect("a radius"),
        );
        let thin = Surface::Cylinder(
            Cylinder::new(
                crate::Axis::new(thin_origin, thin_way).expect("a direction"),
                thin_radius,
            )
            .expect("a radius"),
        );
        assert_eq!(
            crate::intersect(&fat, &thin, &Policy::certified(), &mut ProvenanceLog::new()),
            Err(Error::CannotNameWhichPieceOfACrossingThisIs),
            "the naming half tells the loops of a pipe along {thin_way:?} {apart} off apart, \
             so they do not test the walk"
        );
        let furthest_out = apart + thin_radius;
        let along = (fat_radius * fat_radius - furthest_out * furthest_out).sqrt()
            / (1.0 - leaning * leaning).sqrt();
        for side in [1.0, -1.0] {
            let corner = thin_origin + off_the_fat_axis * thin_radius + w1 * (side * along);
            let run = NamedRun::new(
                EdgeRun::RightRoundWhereItsFacesCross,
                [corner, corner].map(|end| PredicatePoint::from(end.coordinates())),
            )
            .expect("a closed run with one corner");
            for pair in [[&fat, &thin], [&thin, &fat]] {
                let walked = walk(
                    &run,
                    pair,
                    Deflection::of(0.01).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a loop of two slanted pipes is walked");
                let on_its_own_loop = walked
                    .points()
                    .iter()
                    .filter(|point| (**point - Point::origin()).dot(across) * side > 0.0)
                    .count();
                assert!(
                    walked.points().len() > 2 && on_its_own_loop == walked.points().len(),
                    "for a pipe along {thin_way:?} {apart} off, the run round the loop on the \
                     {side} side has {on_its_own_loop} of its {} points on that loop",
                    walked.points().len()
                );
            }
        }
    }

    /// A closed run whose corner is known only as a box reaching across the
    /// flat surface between a crossing's loops is refused, even where the
    /// crossing is one loop and either seed would do. A box that stops short of
    /// that surface, and an exact corner a hair either side of it, are walked.
    ///
    /// Nothing in the kernel builds such a corner: a mesh and `validate` hand
    /// [`walk`] a closed run's corner as the exact point its vertex stores, and
    /// a fuse puts that vertex at the seed ([`where_a_closed_crossing_starts`]).
    /// [`NamedRun::new`] takes a box all the same, and this is what a box gets
    /// (Q1520).
    #[test]
    fn a_closed_run_whose_corner_is_a_box_across_the_surface_between_its_loops_is_refused() {
        let walk_round = |corner: PredicatePoint, pair: [&Surface; 2]| {
            let run = NamedRun::new(EdgeRun::RightRoundWhereItsFacesCross, [corner, corner])
                .expect("a closed run with one corner");
            walk(
                &run,
                pair,
                Deflection::of(0.01).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
        };

        let pipe = Surface::Cylinder(Cylinder::new(crate::Axis::z(), 2.0).expect("a radius"));
        let ball = Surface::Sphere(Sphere::new(Point::new(1.0, 0.0, 0.0), 2.0).expect("a radius"));
        let on_the_one_loop = |z: f64| [0.5, 3.75_f64.sqrt(), z];
        let round_where_it_crosses =
            on_the_one_loop(0.0).map(|middle| Interval::around(middle, 1e-12).expect("a box"));
        assert_eq!(
            walk_round(
                PredicatePoint::Constructed(round_where_it_crosses),
                [&ball, &pipe]
            )
            .map(|walked| walked.points().len()),
            Err(Error::CannotNameWhichPieceOfACrossingThisIs),
            "a box round where the one loop crosses the surface through the ball's centre"
        );
        for z in [1e-300, -1e-300] {
            let walked = walk_round(PredicatePoint::from(on_the_one_loop(z)), [&ball, &pipe])
                .expect("an exact corner on the one loop is walked");
            assert!(
                walked.points().len() > 2,
                "the loop from the corner at z = {z} has points"
            );
        }

        let pipe = Surface::Cylinder(Cylinder::new(crate::Axis::z(), 1.0).expect("a radius"));
        let ball = Surface::Sphere(Sphere::new(Point::new(0.1, 0.0, 0.0), 4.0).expect("a radius"));
        let on_the_loop_below = -(14.79_f64.sqrt());
        let reaching_up_to = |top: f64| {
            PredicatePoint::Constructed([
                Interval::exactly(-1.0),
                Interval::exactly(0.0),
                Interval::new(on_the_loop_below, top).expect("a box"),
            ])
        };
        assert_eq!(
            walk_round(reaching_up_to(0.5), [&ball, &pipe]).map(|walked| walked.points().len()),
            Err(Error::CannotNameWhichPieceOfACrossingThisIs),
            "a box from the loop below up past the surface between the two loops"
        );
        let walked = walk_round(reaching_up_to(-1e-9), [&ball, &pipe])
            .expect("a box that stops short of the surface between the loops is walked");
        assert!(walked.points().len() > 2, "the loop below has points");
        for point in walked.points() {
            let [x, y, z] = point.coordinates();
            assert!(
                z < 0.0,
                "the run from a box below the surface was walked through ({x}, {y}, {z})"
            );
        }
    }
}
