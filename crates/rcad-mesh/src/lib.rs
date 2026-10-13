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
//! One thing follows that is worth saying out loud, because it reads like a
//! defect: **a loop too coarse for the deflection is refused.** A side of a
//! loop is a chord of the surface, its distance from that surface is settled the
//! moment its two corners are chosen, and nothing this crate is allowed to do
//! moves it. So the answer is a refusal naming the side, and the fix is at the
//! caller — which is where it belongs, because the caller is the one that can
//! see both faces along that edge.
//!
//! # How close it says it is
//!
//! A [`Patch`] carries a length: the furthest any point of any triangle can be
//! from the surface, in the model's own unit, as an `Approximation` blaming
//! [`Cause::FlatStandIn`]. One number for the whole patch rather than one per
//! triangle, and a length rather than a `(u, v)` — a turn round a wide pipe and
//! a turn round a narrow one are not the same distance.
//!
//! [`triangulate`] takes **no policy and no log**, and that is a decision rather
//! than an omission. It makes no uncertified decision to record: it refines
//! until its own bound is under the [`Deflection`] it was handed, or it refuses.
//! Whether a given deflection is good enough is a judgement, and judgements
//! belong to the caller, where the policy already is. There is no best-effort
//! answer either, because a patch that quietly missed would need every caller to
//! compare two numbers, and none would (`docs/decisions.md` §25.3).
//!
//! [`Cause::FlatStandIn`]: rcad_predicates::Cause::FlatStandIn
//! [`Deflection`]: rcad_geom::Deflection

#![forbid(unsafe_code)]

mod domain;
mod triangulate;

pub use crate::domain::{Loop, NotALoop, TrimmedDomain};
pub use crate::triangulate::{CannotTriangulate, Patch, triangulate, zip_up_a_band};
