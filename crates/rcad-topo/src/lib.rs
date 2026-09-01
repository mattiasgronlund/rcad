//! The topology of a solid: which faces meet along which edges, and where.
//!
//! A [`Model`] is one flat arena per kind of entity, with a typed key per kind
//! so handing a face key to something that wants an edge is a compile error.
//! Entities store their downward links — a face's wires, a wire's coedges —
//! and the upward ones are derived into a side index, so there are no
//! reference cycles and only ever one copy of the truth (decisions.md §1.3).
//!
//! # Views, and why there is no other way in
//!
//! Nothing here hands out a reference to an entity's stored data. Every
//! accessor gives back a *view* — [`FaceView`], [`EdgeView`], [`VertexView`]
//! and friends — which carries the borrow together with the location and
//! orientation composed on the way down from the solid. A plane read out of a
//! [`FaceView`] has already been moved into place.
//!
//! That is not an ergonomics decision. Handing out stored data hands out
//! geometry that has not been located, and the bug that follows is exactly the
//! one this design exists to prevent (decisions.md §3.2, §3.3).
//!
//! # What M0 builds
//!
//! Planes and straight edges. [`ProfileBuilder`] checks a flat outline with
//! holes in it, [`Model::add_extrusion`] turns it into a solid, and
//! [`Model::add_cube`] is the short way to the obvious test case.
//!
//! ```
//! use rcad_geom::{Deflection, Point, Scale};
//! use rcad_predicates::{Policy, ProvenanceLog};
//! use rcad_topo::{Containment, Model};
//!
//! let scale = Scale::millimetres(100.0)?;
//! let mut model = Model::new(scale);
//! let mut log = ProvenanceLog::new();
//! let cube = model.add_cube(2.0, &Policy::certified(), &mut log)?;
//! assert!(log.is_empty(), "a cube of side two needs nothing approximated");
//!
//! model.validate(&Policy::certified(), &mut log)?;
//! assert!(log.is_empty(), "a clean sweep records nothing at all");
//!
//! // A cube is a polyhedron: the deflection is ignored, its own fan is summed,
//! // and all three error terms come back exactly zero.
//! let measured = model.solid(cube).measure(
//!     Deflection::for_scale(scale),
//!     &Policy::certified(),
//!     &mut log,
//! )?;
//! assert_eq!(measured.volume(), 8.0);
//! assert_eq!(measured.surface_area(), 24.0);
//! assert_eq!(measured.centroid(), Point::new(1.0, 1.0, 1.0));
//! assert_eq!(measured.volume_error(), 0.0);
//! assert_eq!(measured.area_error(), 0.0);
//! assert_eq!(measured.centroid_wobble(), 0.0);
//!
//! let policy = Policy::certified();
//! let middle = model
//!     .solid(cube)
//!     .classify_point(Point::new(1.0, 1.0, 1.0), &policy, &mut log);
//! assert_eq!(middle, Containment::Inside);
//! assert!(log.is_empty(), "nothing had to be approximated");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Errors, panics and answers
//!
//! Three tiers, kept apart on purpose (decisions.md §4.1):
//!
//! - **Bad caller input** — a wire that does not close, a hole outside its
//!   outline, a height of zero — is an [`Error`]. It is data, not a bug.
//! - **A broken internal invariant** — an edge with three coedges, a key
//!   resolving to a freed slot, an approximation that grew — **panics**.
//!   Emitting a corrupt solid is worse than dying.
//! - **Not being able to decide** is neither. At the predicate level it is
//!   [`Decision::Undecidable`](rcad_predicates::Decision::Undecidable) and it
//!   is a result: [`SolidView::classify_point`] returns
//!   [`Containment::Undecidable`] and carries on. At the *operation* level
//!   there is nothing left to return, and that is [`Error::Undecidable`].
//!
//! # Strictness
//!
//! [`Policy`](rcad_predicates::Policy) travels as an explicit argument to
//! everything that makes a decision. It is never a field on [`Model`] and never
//! thread-local: model state that silently changes results is the footgun this
//! kernel exists to avoid (decisions.md §2.6).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod build;
mod classify;
mod cross;
mod error;
mod fuse;
mod holds;
mod key;
mod measure;
mod mesh;
mod model;
pub mod serialize;
mod validate;
mod view;

pub use build::{Profile, ProfileBuilder};
pub use classify::Containment;
pub use error::{Error, Invalid, Result};
pub use fuse::{Fused, Rule};
pub use holds::OnAFace;
pub use key::{CoedgeKey, EdgeKey, FaceKey, ShellKey, SolidKey, VertexKey, WireKey};
pub use measure::Measurement;
pub use mesh::{Mesh, MeshTriangle};
pub use model::Model;
pub use serialize::FORMAT_VERSION;
pub use view::{
    CoedgeView, EdgeView, FaceRef, FaceView, ShellView, SolidView, VertexView, WireView,
};

/// `&Model` is `Sync`, and this is where that is proved rather than claimed.
///
/// Everything in roadmap §10 — measuring, classifying and validating in
/// parallel for free — rests on it. It holds because a model is read-only
/// behind a shared reference: no interior mutability, no lazy caches hiding
/// behind `&self`, nothing memoized. If someone adds a `Cell` or a `RefCell`
/// to make an accessor faster, this line stops compiling, which is the point.
const _: () = {
    const fn is_shareable_between_threads<T: Sync + Send>() {}
    is_shareable_between_threads::<&Model>();
};
