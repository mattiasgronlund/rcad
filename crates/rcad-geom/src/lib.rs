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

mod bspline_curve;
mod bspline_surface;
mod cylinder;
mod de_boor;
mod error;
mod exact;
mod isometry;
mod knot_vector;
mod orientation;
mod plane;
mod point;
mod scale;
mod segment;
mod sphere;
mod surface;

pub use crate::bspline_curve::{BSplineCurve, REFINEMENT_ROUNDOFF};
pub use crate::bspline_surface::BSplineSurface;
pub use crate::cylinder::{Cylinder, SIDE_OF_CYLINDER};
pub use crate::de_boor::EVALUATION_ROUNDOFF;
pub use crate::error::{Error, Result};
pub use crate::isometry::Isometry;
pub use crate::knot_vector::{KnotVector, MAX_DEGREE};
pub use crate::orientation::Orientation;
pub use crate::plane::{Plane, SIDE_OF_PLANE};
pub use crate::point::{Axis, Point, Vector};
pub use crate::scale::{DYNAMIC_RANGE_CHECK, DynamicRange, Scale, Unit};
pub use crate::segment::Segment;
pub use crate::sphere::{SIDE_OF_SPHERE, Sphere};
pub use crate::surface::Surface;
