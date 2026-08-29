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
//! # Deciding signs is not this crate's job yet
//!
//! Nothing here returns a `Decision`. [`Plane::signed_distance_to`] and friends
//! are ordinary `f64` arithmetic and their sign is not trustworthy near zero.
//! The certified sign predicates arrive with the predicate cascade; until then,
//! do not branch on the sign of a distance returned from this crate.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod cylinder;
mod error;
mod isometry;
mod orientation;
mod plane;
mod point;
mod scale;
mod segment;
mod sphere;
mod surface;

pub use crate::cylinder::Cylinder;
pub use crate::error::{Error, Result};
pub use crate::isometry::Isometry;
pub use crate::orientation::Orientation;
pub use crate::plane::Plane;
pub use crate::point::{Axis, Point, Vector};
pub use crate::scale::{DynamicRange, Scale, Unit};
pub use crate::segment::Segment;
pub use crate::sphere::Sphere;
pub use crate::surface::Surface;
