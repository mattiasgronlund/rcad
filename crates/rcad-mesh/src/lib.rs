//! Turning one trimmed patch of one surface into triangles.
//!
//! # What crosses the line into this crate
//!
//! A patch boundary, already walked, as points in the surface's own two
//! coordinates. **No topology.** This crate cannot see an edge, a wire or a
//! face, and the dependency list in `Cargo.toml` is what makes that a fact the
//! compiler checks rather than a rule somebody remembers.
//!
//! Two things follow, and both were settled before the work fanned out
//! (`docs/decisions.md` §23).
//!
//! **A mesh from here carries no face keys.** A `FaceKey` lives in `rcad-topo`,
//! which is above this crate, so a triangle here could not name one without a
//! dependency cycle. What a caller wants — a mesh whose triangles say which
//! face they came from — is built one layer up, by pairing what comes out of
//! here with the key it was asked about. The plan for this milestone had those
//! two requirements written down separately and they could not both be met as
//! stated; this is the shape that meets both.
//!
//! **Watertightness is not this crate's to promise.** Two faces meet along an
//! edge, and this crate is only ever shown one face at a time. What makes a
//! mesh watertight is that the caller works out the points along a shared edge
//! **once** and hands the same 3D points to both sides. Each side is free to
//! disagree about where those points sit in *its* `(u, v)`, because nothing
//! downstream reads `(u, v)` — the triangulation uses it to decide connectivity
//! and the coordinate that reaches the mesh is the shared one, bit for bit.
//! That reads like a defect until you see that the parameter values are a
//! working space and never an answer.
//!
//! # Who refines what
//!
//! **The loops that arrive here are final.** This crate adds points inside the
//! patch and never on its boundary. It cannot see that a boundary is shared —
//! it is shown one face at a time — so if it subdivided a loop, the two faces
//! along an edge would each refine it in their own `(u, v)` and land on
//! different 3D points, which is exactly the promise above being broken. The
//! caller refines every boundary once and hands the same corners to both sides
//! (`docs/decisions.md` §25.1).
//!
//! # What is not here yet
//!
//! Refinement inside the patch, and the number saying how far the triangles
//! stray from the surface. [`triangulate`] currently returns the corners it was
//! given and nothing more.

#![forbid(unsafe_code)]

mod domain;
mod triangulate;

pub use crate::domain::{Loop, NotALoop, TrimmedDomain};
pub use crate::triangulate::{CannotTriangulate, Patch, triangulate};
