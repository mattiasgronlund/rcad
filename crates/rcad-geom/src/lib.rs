//! Analytic geometry for the `rcad` kernel.
//!
//! This crate knows about points, moves, surfaces and straight edges. It knows
//! nothing about topology — no faces, no arena, no keys. That separation is what
//! lets this crate ship on its own (`docs/decisions.md` §7.2).
//!
//! # What lives behind a constructor
//!
//! [`Point`] and [`Vector`] are plain data: three numbers each, no invariants,
//! built with `new` and never rejected. Everything else — [`Axis`], [`Plane`],
//! [`Sphere`], [`Cylinder`], [`Segment`], [`Isometry`], [`Scale`] — carries an
//! invariant the type system cannot state (a unit direction, a positive radius,
//! two ends that are not the same point). Those have private fields and a
//! checking constructor that returns [`Error`] when the caller hands over
//! something that cannot be a valid piece of geometry.
//!
//! Bad caller input is an `Err`. A broken invariant *inside* an already-valid
//! value is a panic. See `docs/decisions.md` §4.1.
//!
//! # Three kinds of answer, and they are not interchangeable
//!
//! [`Plane::signed_distance_to`] is ordinary `f64` arithmetic. Its magnitude is
//! a real distance; its *sign*, close to zero, is not something to trust, and
//! nothing may branch on it.
//!
//! The trustworthy answers come from the predicate cascade and are named for
//! it: [`Plane::side_of_point`], and
//! [`SideOfSurface::side_of`](rcad_predicates::SideOfSurface::side_of) for
//! [`Sphere`] and [`Cylinder`]. They return a
//! [`Decision`](rcad_predicates::Decision), so every call site has to say what
//! it does when the cascade could not tell.
//!
//! Even those are only exact about the numbers a surface holds. Every
//! constructor here scales a normal or a direction to length one, and that
//! division rounds — so a point that helped build a face is provably a rounding
//! off the face's *stored* plane while being exactly on its true one. Read the
//! note on [`Plane::side_of_point`] before using it to answer that question.
//!
//! The third kind is the splines'. A point on a curve is worked out by
//! arithmetic, so it is neither a trustworthy `f64` nor an exact input to a
//! predicate. The honest answer is an *enclosed* one — a box the true point is
//! somewhere inside — and that is what a curved surface can offer a predicate
//! without lying to it. [`BSplineCurve::evaluate_with_enclosure`] and
//! [`BSplineSurface::evaluate_with_enclosure`] give it, and it is the only form
//! a predicate may be asked about a spline point in.
//!
//! A box wide enough to straddle a surface makes the answer
//! [`Decision::Undecidable`](rcad_predicates::Decision::Undecidable), and that
//! is the truth about the point rather than a shortcoming of the cascade: a
//! point exactly on a circle sits at coordinates no `f64` can name, so no
//! amount of precision will place it on one side. Handing the same point over
//! as an exact `f64` triple instead buys a certified `Pos` or `Neg` — an exact
//! answer to the wrong question (`docs/decisions.md` §2.5). Where the divide
//! happens to round nothing away, the box has zero width and an *exact* point
//! comes back out, which is stronger than a box and is what keeps the
//! exactly-computable cases decidable.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod along_a_crossing;
mod bspline_curve;
mod bspline_surface;
mod cone;
mod conic;
mod cylinder;
mod de_boor;
mod deflection;
mod enclosure;
mod error;
mod exact;
mod half_space;
mod intersect;
mod isometry;
mod knot_vector;
mod march;
mod meet;
mod moved_corner;
mod orientation;
mod plane;
mod point;
mod real_roots;
mod scale;
mod segment;
mod sphere;
mod surface;
mod three_surfaces;
mod torus;
mod walk;
mod which_way_a_closed_crossing_turns;

pub use crate::along_a_crossing::{ROUND_A_CROSSING, which_comes_first_round_a_crossing};
pub use crate::bspline_curve::{BSplineCurve, REFINEMENT_ROUNDOFF};
pub use crate::bspline_surface::BSplineSurface;
pub use crate::cone::{Cone, SIDE_OF_CONE};
pub use crate::cylinder::{Cylinder, SIDE_OF_CYLINDER};
pub use crate::de_boor::EVALUATION_ROUNDOFF;
pub use crate::deflection::Deflection;
pub use crate::enclosure::root_enclosing;
pub use crate::error::{Error, LimitOrGap, Result, WhereFollowingACrossingGaveUp};
pub use crate::half_space::{HalfSpace, Picker, SIDE_OF_A_HALF_SPACE};
pub use crate::intersect::{
    WHERE_A_BALL_AND_A_PIPE_CROSS, WHERE_A_PLANE_AND_A_BALL_CROSS, WHERE_A_PLANE_AND_A_PIPE_CROSS,
    WHERE_A_PLANE_AND_A_TORUS_CROSS, WHERE_TWO_BALLS_CROSS, WHERE_TWO_PIPES_CROSS,
    WHERE_TWO_PLANES_CROSS, how_a_plane_leans_against_a_cone, how_a_plane_leans_against_a_pipe,
    intersect, where_a_cones_apex_stands_against_a_plane,
};
pub use crate::isometry::Isometry;
pub use crate::knot_vector::{KnotVector, MAX_DEGREE};
pub use crate::meet::{EdgeRun, NamedRun, SurfaceMeet};
pub use crate::moved_corner::{
    CORNER_AFTER_MOVING_ROUNDOFF, SURFACE_AFTER_SHIFTING_ROUNDOFF,
    where_a_corner_lands_after_moving,
};
pub use crate::orientation::Orientation;
pub use crate::plane::{Plane, PlaneAgreement, SIDE_OF_PLANE};
pub use crate::point::{Axis, Point, Vector};
pub use crate::scale::{DYNAMIC_RANGE_CHECK, DynamicRange, Scale, Unit};
pub use crate::segment::Segment;
pub use crate::sphere::{SIDE_OF_SPHERE, Sphere};
pub use crate::surface::Surface;
pub use crate::three_surfaces::{
    MeetOfThree, MeetsAt, WHERE_A_LINE_MEETS_A_SURFACE, WHERE_THREE_SURFACES_MEET,
    WhereALineMeetsIt, meet_of_three_surfaces, where_a_line_meets_a_surface,
};
pub use crate::torus::{SIDE_OF_TORUS, Torus};
pub use crate::walk::{
    MOST_POINTS_ALONG_ONE_CROSSING, POINT_ALONG_A_CROSSING_ROUNDOFF,
    POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF, PointsAlongACrossing, ROOM_LEFT_TO_REFINE_A_WALK, walk,
    where_a_closed_crossing_starts, which_two_surfaces_a_run_is_between,
};
pub use crate::which_way_a_closed_crossing_turns::which_way_a_closed_crossing_turns;

/// How far the nearest `f64` triple to a point sits from that point, per unit
/// of the point's largest coordinate.
///
/// Rounding a coordinate `x` to the nearest `f64` moves it by at most half the
/// gap between neighbouring `f64` there, and that half-gap is at most
/// `|x| · f64::EPSILON / 2`. Three coordinates each moved that far move the
/// point by at most `√3 · f64::EPSILON / 2 ≈ 0.87 · f64::EPSILON` times the
/// largest of them, which is under one.
///
/// That is a *relative* argument like every other bound here, so it holds only
/// while the coordinates are normal numbers. A subnormal coordinate is rounded
/// by an absolute amount — up to `2⁻¹⁰⁷⁵`, which can be the whole of it — and
/// no multiple of the coordinate bounds that.
///
/// So a point that truly lies on a curved surface, written down as three `f64`,
/// lands within `NEAREST_F64_POINT_ROUNDOFF · L` of that surface and nowhere
/// nearer is available. **This is the width of the number and not a modelling
/// tolerance:** a corner inside it is one no `f64` could have written any
/// nearer, and a corner outside it is off a surface somebody chose rather than
/// off a surface that rounded.
///
/// # Why a length, when a curved surface answers with a sign
///
/// [`Plane`] has [`Plane::SIGNED_DISTANCE_ROUNDOFF`] because reading a distance
/// off a plane is `f64` arithmetic that rounds. A ball, a pipe, a cone and a
/// ring have no distance to read: they answer
/// [`SideOfSurface::side_of`](rcad_predicates::SideOfSurface::side_of), which is
/// a [`Decision`](rcad_predicates::Decision) and carries no length out. Each of
/// the four does carry a private forward error bound for level 1 of the
/// cascade, but that bounds the *float filter's* own arithmetic as a fraction of
/// its permanent — a squared length for a ball, a pipe and a cone, a quartic for
/// a ring. It is what settles a corner well clear of its surface, and it is not
/// what settles one a rounding off it: that one falls past level 1, which is
/// what a filter bound is for.
///
/// A budget in the predicate's units would therefore have to be a new question
/// of the form "is the value smaller than this", which is `|f(p)| < tol` — the
/// wrong question §2.5 names and
/// `a_point_exactly_on_a_circle_is_never_certified_off_it.rs` is built to keep
/// out. It would also not be one number. Moving a point by `d` moves the value
/// by about `|∇f|·d`, and `|∇f|` is a fact about *where* on the surface the
/// corner sits rather than a constant: `2r` on a ball and on a pipe, but on a
/// cone it is set by how far from the axis the corner is and grows without
/// bound along the axis, and a ring's value is a quartic whose slope depends on
/// both radii and on where round the tube the corner is.
///
/// The length is spendable as it stands. Widening a corner into a box of this
/// half-width and asking the surface which side the *box* is on is what
/// `rcad-topo`'s `not_provably_off` does, with
/// `Model::what_writing_a_point_down_costs` as the caller's own slack factor on
/// top (`docs/decisions.md` §15.6). What comes back is the cascade's ordinary
/// **conservative** answer about a box and not an exact one — a box has no exact
/// level to reach, and the interval arithmetic left over-estimates. On a pipe, a
/// cone and a ring that over-estimate is already worth about a coordinate
/// spacing at ordinary sizes, so for those three this bound is a floor under the
/// reach rather than what decides. The search prints both figures.
///
/// # What this bounds, and what it does not
///
/// The corner's own three numbers. It says nothing about the surface's numbers —
/// a centre, a radius, an axis direction scaled to length one — each of which
/// rounds too, so the surface the numbers *are* is a few roundings from the
/// surface the numbers *mean*. A caller allowing for that wants this times a
/// slack factor of its own.
///
/// `how_far_off_a_curved_surface_the_nearest_f64_can_sit.rs` is the search that
/// would catch this number being too small (§15.6).
pub const NEAREST_F64_POINT_ROUNDOFF: f64 = f64::EPSILON;
