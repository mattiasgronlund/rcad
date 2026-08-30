//! Geometric predicates that either prove their answer or admit they cannot.
//!
//! Every predicate here returns a [`Decision`], never a bare sign, so a caller
//! has to say what it does when the answer is [`Decision::Undecidable`]
//! (decisions.md §2.1). There is no silent fallback to a tolerance.
//!
//! # The cascade
//!
//! An answer is worked out in up to three tiers, cheapest first
//! (decisions.md §2.2):
//!
//! 1. Plain `f64`, with a forward error bound. If the value is clear of the
//!    bound, its sign is proved and that is the end of it. This settles almost
//!    everything.
//! 2. Interval arithmetic. If the enclosure avoids zero, the sign is proved.
//! 3. Exact expansion arithmetic. Always decides — for exact `f64` inputs.
//!
//! Level 3 is where the third state comes from. A point built by intersecting
//! three planes is only known to lie inside a box, and no amount of precision
//! recovers what was never there: the answer is [`Decision::Undecidable`] with
//! the width of the box that straddled zero.
//!
//! ```
//! use rcad_predicates::{Decision, Point, meet_of_three_planes, orient3d_of_points};
//!
//! // Three planes meeting at (1/3, 1/3, 1/3), which is not an f64.
//! let meet = meet_of_three_planes(
//!     [1.0, 1.0, 1.0, -1.0],
//!     [1.0, -1.0, 0.0, 0.0],
//!     [0.0, 1.0, -1.0, 0.0],
//! )
//! .unwrap();
//!
//! // The plane x + y + z = 1 passes exactly through that meeting point.
//! let decision = orient3d_of_points(
//!     Point::from([1.0, 0.0, 0.0]),
//!     Point::from([0.0, 1.0, 0.0]),
//!     Point::from([0.0, 0.0, 1.0]),
//!     meet,
//! );
//!
//! match decision {
//!     Decision::Neg | Decision::Zero | Decision::Pos => panic!("we cannot know this"),
//!     Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
//!     // `Decision` gains variants at every milestone, so code outside this
//!     // crate needs a last arm. Never let `Undecidable` land in it.
//!     other => panic!("unhandled decision: {other:?}"),
//! }
//! ```
//!
//! # Strictness
//!
//! [`Policy`] travels as an explicit argument to anything that can make a
//! decision — never a field on a model, never thread-local. The tolerant entry
//! points are separately named so nobody reaches one by accident, and anything
//! they approximate lands in a [`ProvenanceLog`]. A fully certified operation
//! leaves that log empty, which is itself the signal that nothing was fudged.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod cascade;
mod decision;
mod error;
mod interval;
mod orient2d;
mod orient3d;
mod planes;
mod point;
mod policy;
mod provenance;
mod surface;

pub mod expansion;

pub use cascade::{Filtered, Level, Predicate, decide, product_is_normal};
pub use decision::Decision;
pub use error::Error;
pub use interval::Interval;
pub use orient2d::{ORIENT2D, orient2d, orient2d_reporting_level, orient2d_tolerantly};
pub use orient3d::{
    ORIENT3D, orient3d, orient3d_of_points, orient3d_reporting_level, orient3d_tolerantly,
};
pub use planes::meet_of_three_planes;
pub use point::Point;
pub use policy::{Measures, Policy, Severity, Strictness, resolve_under_policy};
pub use provenance::{Approximation, Cause, ProvenanceLog, Record};
pub use surface::SideOfSurface;
