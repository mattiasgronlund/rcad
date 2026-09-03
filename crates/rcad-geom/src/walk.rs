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
    Approximation, Cause, Decision, Interval, Level, Measures, Point as PredicatePoint, Policy,
    ProvenanceLog, Record,
};

use crate::exact::Exact;
use crate::march::{march, settled_with_a_bound};
use crate::meet::nowhere_in_particular;
use crate::{
    Cone, Cylinder, Deflection, EdgeRun, Error, NamedRun, Picker, Plane, Point, Result, Sphere,
    Surface, Torus, Vector,
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
/// # What the test can and cannot fail on, with the number
///
/// `every_point_along_an_endless_bend_is_on_both_surfaces_inside_the_published_bound`
/// is the test §15.6 asks for. Over 10,981 walks and 339,576 points it finds a
/// worst of **0.049 of this bound** — about `3.1 · ε · L`.
///
/// **So halving this constant would not fail that test, and saying so is the
/// point.** 98 roundings charged their full half-ulp each is a *count*; the
/// same 98 signed errors add to about `√98/√3` of the count, which is a
/// sixteenth of it, and 0.049 is that sixteenth with the derivation's own 1.3
/// on top. §15.6 says this in as many words — *"that test cannot show a bound
/// is tight — rounding errors cancel"* — and what makes `64 · ε` a bound here
/// is the derivation above and not the search.
///
/// The half a search *can* pin is the **scaling**, and
/// `what_an_endless_bend_strays_grows_with_the_size_it_was_worked_out_from` is
/// that half: the same bend at four scales 32,768 apart uses the same 0.011 of
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
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] and
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] for a run between a
/// cone or a torus and another curved surface — the same two refusals
/// [`intersect`](crate::intersect) gives those pairs. Nothing in this crate
/// builds such a run, and a file read in states one whenever it holds a pipe
/// running into a bend.
///
/// [`Error::TheCrossingOfTheseTwoSurfacesIsNotClosed`] for a run a caller
/// states as going right round a crossing that runs on for ever — a plane and a
/// cone crossing in a parabola or a hyperbola. The two shapes through the apex
/// are straight lines, and a straight run gets the same empty answer here that a
/// run between two planes does.
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

    let pair = the_two_surfaces_it_runs_between(run, along);
    let conic = match what_a_run_between_these_two_lies_on(pair, run, log)? {
        WhatARunLiesOn::StraightLine => return Ok(nothing_between_the_ends()),
        WhatARunLiesOn::QuarticPickedUpAt(seeds) => {
            return follow_the_quartic(run, pair, seeds, deflection, policy, log);
        }
        WhatARunLiesOn::Parabola(bend) => {
            return walk_an_endless_bend(&bend, run, deflection, policy);
        }
        WhatARunLiesOn::OneBranchOfAHyperbola(branch) => {
            return walk_an_endless_bend(&branch, run, deflection, policy);
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
/// on it falls out of `f64`'s range.
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
pub fn where_a_closed_crossing_starts(
    run: &NamedRun,
    along: [&Surface; 2],
    log: &mut ProvenanceLog,
) -> Result<Point> {
    let pair = the_two_surfaces_it_runs_between(run, along);
    match what_a_run_between_these_two_lies_on(pair, run, log)? {
        WhatARunLiesOn::StraightLine => Err(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed),
        WhatARunLiesOn::QuarticPickedUpAt(seeds) => {
            the_seed_this_run_is_picked_up_at(run, &pair, seeds, log)
        }
        WhatARunLiesOn::Parabola(_) | WhatARunLiesOn::OneBranchOfAHyperbola(_) => {
            Err(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed)
        }
        WhatARunLiesOn::Conic(conic) => Ok(conic.at(0.0)),
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
    /// the marcher to start from.
    QuarticPickedUpAt([Point; 2]),
    /// A parabola: one curve that runs on for ever both ways and bends.
    Parabola(Parabola),
    /// One of a hyperbola's two branches, which runs on for ever both ways and
    /// bends too. The other branch is the sibling run, and which of the two
    /// this is was settled by the half-spaces on it.
    OneBranchOfAHyperbola(HyperbolaBranch),
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
/// **The evaluator refuses exactly what the namer refuses**, and both are gaps:
/// the crossing is there and none of the machinery for a degree-four or
/// degree-eight curve with a cone or a torus in it exists.
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when a seed for a degree-four
/// crossing comes out as something that is not a point.
///
/// [`Error::TheCrossingOfTheseTwoSurfacesIsNotClosed`] for a run a caller named
/// as closed between a plane and a cone that cross in a parabola or a
/// hyperbola, and [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the
/// half-spaces on a hyperbola's run hold both branches or neither.
///
/// Whatever the conic builders refuse, for the pairs that have one.
fn what_a_run_between_these_two_lies_on(
    pair: [Surface; 2],
    run: &NamedRun,
    log: &mut ProvenanceLog,
) -> Result<WhatARunLiesOn> {
    match pair {
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
            where_a_plane_cuts_a_cone(&flat, &cone, run, log)
        }
        // A plane cuts a torus in **two** circles as often as in one, and which
        // of them this run is is the picker's to say — which is why this is the
        // one arm here that reads the run rather than only the pair.
        [Surface::Plane(flat), Surface::Torus(ring)]
        | [Surface::Torus(ring), Surface::Plane(flat)] => {
            where_a_plane_cuts_a_torus(&flat, &ring, run, log).map(WhatARunLiesOn::Conic)
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
        [Surface::Sphere(ball), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Sphere(ball)] => {
            a_quartic_picked_up_at(where_a_ball_reaches_across_a_pipe(&ball, &pipe))
        }
        [Surface::Cylinder(one), Surface::Cylinder(other)] => {
            // Two pipes down one direction cross in straight lines, and a run
            // that is one of those is picked out by a half-space rather than
            // being `Straight`, so it reaches here.
            if one.axis().direction().cross(other.axis().direction()) == Vector::zero() {
                return Ok(WhatARunLiesOn::StraightLine);
            }
            let certified = |pipe: &Cylinder| pipe.radius() / pipe.axis().direction().length();
            // Two pipes of the same certified radius are neither thinner, and
            // then which one is walked round has to be settled by the numbers
            // they store rather than by which of them the caller wrote first —
            // otherwise the seed, and the vertex `where_a_closed_crossing_starts`
            // puts on the loop, move when the pair is swapped.
            let mine_is_thinner = match certified(&one).total_cmp(&certified(&other)) {
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
            a_quartic_picked_up_at(where_one_pipe_reaches_across_another(&thin, &fat))
        }
        [
            Surface::Cone(_),
            Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_),
        ]
        | [Surface::Sphere(_) | Surface::Cylinder(_), Surface::Cone(_)] => {
            Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface)
        }
        [
            Surface::Torus(_),
            Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_),
        ]
        | [
            Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_),
            Surface::Torus(_),
        ] => Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface),
    }
}

/// The two seeds of a degree-four crossing, once they have been checked for
/// being points at all.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when either comes out as something
/// that is not a point.
fn a_quartic_picked_up_at(seeds: [Point; 2]) -> Result<WhatARunLiesOn> {
    if seeds.iter().any(|seed| !seed.is_finite()) {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    }
    Ok(WhatARunLiesOn::QuarticPickedUpAt(seeds))
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
/// The marcher's two refusals.
fn follow_the_quartic(
    run: &NamedRun,
    pair: [Surface; 2],
    seeds: [Point; 2],
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<PointsAlongACrossing> {
    let half_the_budget = deflection.length() / 2.0;
    let surfaces = [&pair[0], &pair[1]];
    let seed = the_seed_this_run_is_picked_up_at(run, &pair, seeds, log)?;

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
    Ok(PointsAlongACrossing {
        points: walked.points,
        deviation: tag,
    })
}

/// The place on the loop this run names that the marcher starts from, settled
/// onto both surfaces.
///
/// A run that nothing picks is the whole of a single loop, and the two seeds are
/// two places on it — either will do. A run that *is* picked is one of two loops,
/// and then exactly one of the seeds may pass: both or neither and the
/// half-spaces have not told them apart after all.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the half-spaces pick
/// both loops or neither.
fn the_seed_this_run_is_picked_up_at(
    run: &NamedRun,
    pair: &[Surface; 2],
    seeds: [Point; 2],
    log: &mut ProvenanceLog,
) -> Result<Point> {
    let surfaces = [&pair[0], &pair[1]];
    let picked_by: &[Picker] = match run.run() {
        EdgeRun::PartOfWhereItCrosses { picked_by, .. } => picked_by,
        EdgeRun::Straight | EdgeRun::RightRoundWhereItsFacesCross => &[],
    };

    let mut settled_seeds = Vec::new();
    for seed in seeds {
        let (settled, off) = settled_with_a_bound(surfaces, seed)?;
        if picked_by.is_empty() || every_half_space_holds(picked_by, settled, off, log) {
            settled_seeds.push(settled);
        }
    }
    match (picked_by.is_empty(), settled_seeds.as_slice()) {
        (true, [first, ..]) => Ok(*first),
        (false, [only]) => Ok(*only),
        _ => Err(Error::CannotNameWhichPieceOfACrossingThisIs),
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
/// **(a) Let the topology say.** Refuse an endless run in [`walk`] and add an
/// [`EdgeRun`] that carries its own stretch, so a fuse that cuts a face along a
/// parabola hands over the two places the face's own wire reaches.
/// The refusal this replaced said as much in its own doc, and it is honest —
/// the range really does belong where the topology is. It is **not free**, and
/// the cost is the reason it is not taken here:
///
/// - Every match on [`EdgeRun`] outside this crate grows an arm, and §40's own
///   note says a wildcard is where the next variant goes to be ignored. The
///   variant is `Serialize`, so a file states one, so every reader has to handle
///   a stretch it did not compute.
/// - **The stretch would then be geometry a caller supplies**, and this module's
///   whole reason for existing is that there is *one* authority on where the
///   points along an edge are (§27.5). A run carrying two parameters is a second
///   one, and the two would have to be kept in step by hand — which is the
///   shape §23.8 kept finding.
/// - It cannot be built now anyway: no fuse in the tree cuts a face along a
///   cone's parabola, so the variant would arrive with no consumer, which §13
///   is about.
///
/// **(b) Pick a stretch from the two surfaces alone**, which is what is done.
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
/// consumer that needs more takes (a), and (a) becomes short the day a fuse
/// wants it.
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
) -> Result<PointsAlongACrossing> {
    let deviation = POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF * bend.the_size_it_was_worked_out_from();
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
    Ok(steps.max(1.0) as usize)
}

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
        EdgeRun::Straight
        | EdgeRun::RightRoundWhereItsFacesCross
        | EdgeRun::PartOfWhereItCrosses { .. } => [*along[0], *along[1]],
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
    Ok(steps.max(1.0) as usize)
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

    /// How far `point` sits off this circle, measured in its own plane and
    /// across it.
    ///
    /// Only ever asked of a circle — both semi-axes the same length — which is
    /// the whole of what a plane and a torus cross in. Two candidates that lie
    /// in one plane and differ only in radius are told apart by the first term;
    /// the second is what says a point is on the plane at all.
    fn how_far_off_it(&self, point: Point) -> f64 {
        let out = point - self.centre;
        let radius = self.across.length();
        let across = out.dot(self.across) / radius;
        let and = out.dot(self.and) / self.and.length();
        let up = out - self.across * (across / radius) - self.and * (and / self.and.length());
        (across.hypot(and) - radius).abs() + up.length()
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
        Picker::OutsideABall(_) | Picker::OutsideAPipe(_) | Picker::InsideAPipe(_) => {
            Measures::Area
        }
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
        if run.run() == &EdgeRun::RightRoundWhereItsFacesCross {
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
            [across, and],
            run,
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

/// Which of a hyperbola's two branches this run names, told by the half-spaces
/// on it.
///
/// **The same shape as [`where_a_plane_cuts_a_torus`]**, and for the same
/// reason: the two candidates are built, each is asked whether every picker on
/// the run holds it, and one held candidate is an answer while two or none is
/// not. Each branch lies wholly in one nappe of the cone, so the half-space
/// through the apex square to the axis is what tells them apart — and
/// `intersect` mints exactly that pair.
///
/// The point each is asked at is its own vertex, and one point settles the
/// whole branch: the picker's normal is the cone's axis direction bit for bit,
/// so the sign it reads is the sign of `(x − apex)·ŵ`, which does not change
/// anywhere on one branch.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the pickers hold both
/// branches or neither, which is what a run with nothing picking it gets.
fn which_branch_of_a_hyperbola_this_run_is(
    spread: [f64; 3],
    places: [Point; 2],
    directions: [Vector; 2],
    run: &NamedRun,
    log: &mut ProvenanceLog,
) -> Result<WhatARunLiesOn> {
    let [towards, sideways_reach, standing_off] = spread;
    let [apex, centre] = places;
    let [across, and] = directions;
    let size = how_big_this_is(&[centre, apex], &[towards, sideways_reach, standing_off]);
    let picked_by: &[Picker] = match run.run() {
        EdgeRun::PartOfWhereItCrosses { picked_by, .. } => picked_by,
        EdgeRun::Straight | EdgeRun::RightRoundWhereItsFacesCross => &[],
    };

    let mut held = Vec::new();
    for side in [1.0, -1.0] {
        let branch = HyperbolaBranch {
            centre,
            out: and * (towards * side),
            across: across * sideways_reach,
            size,
        };
        let deviation = POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF * size;
        if every_half_space_holds(picked_by, branch.at(0.0), deviation, log) {
            held.push(branch);
        }
    }
    match held.as_slice() {
        [only] => Ok(WhatARunLiesOn::OneBranchOfAHyperbola(*only)),
        _ => Err(Error::CannotNameWhichPieceOfACrossingThisIs),
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

/// The circle where a plane cuts a torus, picked out of the two the crossing can
/// come in.
///
/// **Two families reach here and no others.** A plane square to the axis cuts
/// circles round the axis; a plane holding the axis cuts the two meridian
/// circles. Every other plane is refused by name in
/// [`intersect`](crate::intersect) before a run exists, so this never has to
/// stand in for a quartic.
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
/// They do not. A run only reaches here in one of the two families
/// [`intersect`](crate::intersect) certified **exactly**, and the family this
/// arm sends away is the one holding the axis, where `n·w = 0` for two vectors
/// of length one — so `‖n × w‖` is one there, and no rounding of anything walks
/// a whole unit. The pipe arm's `n·w == 0.0` has a three-term sum in it and
/// separates its own two families the same way.
///
/// # Errors
///
/// [`Error::CannotWorkOutHowBigACrossingIs`] when the circle's own numbers do
/// not come out finite, and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the pickers hold both
/// candidates or neither.
fn where_a_plane_cuts_a_torus(
    flat: &Plane,
    ring: &Torus,
    run: &NamedRun,
    log: &mut ProvenanceLog,
) -> Result<Conic> {
    let candidates = if flat.normal().cross(ring.axis().direction()) == Vector::zero() {
        circles_round_the_axis(flat, ring)
    } else {
        the_two_meridian_circles(flat, ring)
    };
    if candidates.iter().any(|circle| {
        !(circle.centre.is_finite() && circle.across.is_finite() && circle.and.is_finite())
    }) {
        return Err(Error::CannotWorkOutHowBigACrossingIs);
    }

    let picked_by: &[Picker] = match run.run() {
        EdgeRun::PartOfWhereItCrosses { picked_by, .. } => picked_by,
        EdgeRun::Straight | EdgeRun::RightRoundWhereItsFacesCross => &[],
    };
    let ends: Vec<Point> = run
        .ends()
        .into_iter()
        .filter(|end| *end != nowhere_in_particular())
        .map(|end| Point::from(end.enclosure().map(middle_of)))
        .collect();

    let mut held: Vec<Conic> = Vec::new();
    for circle in candidates {
        let deviation = POINT_ALONG_A_CROSSING_ROUNDOFF * circle.the_size_it_was_worked_out_from();
        let holds_every_end = ends
            .iter()
            .all(|end| circle.how_far_off_it(*end) <= deviation);
        if holds_every_end
            && (picked_by.is_empty()
                || every_half_space_holds(picked_by, circle.at(0.0), deviation, log))
        {
            held.push(circle);
        }
    }
    match held.as_slice() {
        [only] => Ok(*only),
        _ => Err(Error::CannotNameWhichPieceOfACrossingThisIs),
    }
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
    /// holds — cross to `(0, 0, 0)` in `f64` with a true first component near
    /// `5.2e-17`.
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
        let one = unit(Vector::new(1.0, 2.0, 6.0));
        let other = unit(Vector::new(1.0, 2.0, 6.0f64.next_up()));

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
                    &mut ProvenanceLog::new()
                ),
                Ok(WhatARunLiesOn::QuarticPickedUpAt(_))
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
                ((point - pipe.axis().origin()).cross(along).length() - pipe.radius()).abs()
                    / along.length()
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
}
