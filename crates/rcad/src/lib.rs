//! A tolerant B-Rep CAD kernel.
//!
//! This crate is the front door. It re-exports the whole M0 public surface and
//! nothing else: the three crates underneath it — `rcad-predicates`,
//! `rcad-geom` and `rcad-topo` — are an implementation detail of how the kernel
//! is built, not something a caller should have to name
//! (`docs/decisions.md` §3.6, §7.2).
//!
//! # The one thing to understand first
//!
//! Three different things in this kernel can fail to reach an answer, and they
//! are **not** the same thing. Mixing them up is the single most expensive
//! mistake a new caller makes.
//!
//! | Level | Cannot decide is | What you do about it |
//! |---|---|---|
//! | A predicate | [`Decision::Undecidable`] | Handle it. It is a result, not a failure. |
//! | A classification | [`Containment::Undecidable`] | Handle it. Same. |
//! | A whole operation | [`Error::Undecidable`] | It failed. Read the records it carries. |
//!
//! A predicate that says `Undecidable` has told you something true: the sign of
//! this quantity does not follow from the numbers you gave it, and here is how
//! wide the interval that straddled zero was. That is a *better* answer than a
//! guess, and it is why [`Decision`] has four variants and no `bool` anywhere
//! near it (`docs/decisions.md` §2.1).
//!
//! An *operation* that ends up with nothing to return is a different matter.
//! There is no shape to hand back, so it fails — [`Error::Undecidable`], which
//! carries the [`ProvenanceLog`] of exactly which decisions gave up and how
//! wide their enclosures were (`docs/decisions.md` §4.2).
//!
//! [`Error`] keeps the three layers underneath it visible rather than
//! flattening them: [`Error::Predicate`], [`Error::Geometry`] and
//! [`Error::Topology`] wrap [`PredicateError`], [`GeometryError`] and
//! [`TopologyError`], all three re-exported here. Matching through to an inner
//! variant never needs a dependency on a crate this facade exists to hide.
//!
//! Every call site that receives a [`Decision`] or a [`Containment`] must say
//! what it does with all of the cases. That is the point of the type.
//!
//! # Strictness travels with the call
//!
//! [`Policy`] is an argument, never a field on a [`Model`] and never
//! thread-local. Model state that silently changes results is the footgun this
//! kernel exists to avoid (`docs/decisions.md` §2.6). [`Policy::certified`]
//! proves everything or says it could not; [`Policy::tolerant`] trades a
//! bounded amount of slop for an answer and records what it traded.
//!
//! # A worked example
//!
//! Build a cube, measure it, ask where a point sits, and write it out and read
//! it back.
//!
//! ```
//! use rcad::{Containment, Model, Point, Policy, ProvenanceLog, Scale};
//!
//! let mut model = Model::new(Scale::millimetres(100.0)?);
//! let mut log = ProvenanceLog::new();
//! let cube = model.add_cube(10.0, &Policy::certified(), &mut log)?;
//! assert!(log.is_empty(), "a cube of side ten needs nothing approximated");
//!
//! model.validate(&Policy::certified(), &mut log)?;
//! assert!(log.is_empty(), "a clean sweep records nothing at all");
//!
//! let measured = model.solid(cube).measure();
//! assert_eq!(measured.volume(), 1000.0);
//! assert_eq!(measured.surface_area(), 600.0);
//! assert_eq!(measured.centroid(), Point::new(5.0, 5.0, 5.0));
//!
//! let policy = Policy::certified();
//! match model
//!     .solid(cube)
//!     .classify_point(Point::new(5.0, 5.0, 5.0), &policy, &mut log)
//! {
//!     Containment::Inside => {}
//!     Containment::OnBoundary => panic!("the middle of a cube is not its skin"),
//!     Containment::Outside => panic!("the middle of a cube is inside it"),
//!     Containment::Undecidable { enclosure } => {
//!         panic!("the middle of a cube is not a close call: {enclosure}")
//!     }
//!     // `Containment` grows a variant at every milestone. Guessing what a new
//!     // one means is worse than stopping.
//!     other => panic!("unhandled answer: {other:?}"),
//! }
//! assert!(log.is_empty(), "nothing had to be approximated");
//!
//! let bytes = postcard::to_allocvec(&model)?;
//! let read_back: Model = postcard::from_bytes(&bytes)?;
//! read_back.validate(&Policy::certified(), &mut log)?;
//! let same_cube = read_back.solids().next().expect("the cube came back");
//! assert_eq!(same_cube.measure(), measured);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Asking a predicate on its own
//!
//! Everything above decides through the predicate cascade, and the cascade is
//! callable directly too. [`orient2d`] says which way three points turn,
//! [`orient3d`] which side of the plane through three points a fourth one falls
//! on, and [`SideOfSurface::side_of`] whether a point is in, on or out of a
//! [`Sphere`] or a [`Cylinder`]. Each answers with a [`Decision`], so there is
//! no threshold anywhere for a caller to pick. [`orient2d_tolerantly`] and
//! [`orient3d_tolerantly`] are the same questions asked under a [`Policy`], and
//! they are separately named so nobody reaches one by accident
//! (`docs/decisions.md` §2.1).
//!
//! *How* the cascade reaches an answer — the interval type, the expansion
//! arithmetic, asking which tier settled a certified answer — is deliberately
//! not out here. That is how the answer is built, not what it says
//! (`docs/decisions.md` §3.6). The one place a tier is worth knowing is when
//! one gave up, and that is on the [`Record`] in the log, as a [`Level`].
//!
//! ## Two kinds of point, and they are not the same thing
//!
//! [`Point`] is a place: three coordinates in the model's unit. It is what
//! shapes are built out of and what [`SolidView::classify_point`] takes.
//!
//! [`PredicatePoint`] is what a predicate is asked *about*, and it comes two
//! ways. [`PredicatePoint::Exact`] is three `f64` read straight off a shape;
//! every [`Point`] turns into one with `.into()`.
//! [`PredicatePoint::Constructed`] is a point nobody can write down — where
//! three planes meet, say — and all that is known of it is a box it lies
//! somewhere inside.
//!
//! That second kind is the whole reason [`Decision::Undecidable`] exists.
//! Exact coordinates always get a certified answer. A box that straddles the
//! thing being asked about cannot, and no amount of precision recovers what was
//! never there (`docs/decisions.md` §2.5).
//!
//! ## A certified answer is about the numbers you named
//!
//! [`Plane::side_of_point`] gives the exact sign of `normal · point - offset`
//! for the four numbers that plane holds. Building a plane scales its normal to
//! length one, and that division rounds, so the stored plane sits a rounding
//! away from the one that was asked for — and the predicate will then say `Pos`
//! or `Neg`, with total confidence, about the very points the plane was built
//! through.
//!
//! To ask about the plane you *named*, hand [`orient3d`] the three points that
//! defined it. It never divides, so it is certified about the geometry the
//! caller actually gave it. [`orient3d_of_points`] is the same question where
//! any of the four points may be a constructed one.
//!
//! [`Plane::meet_of_three`] is where a constructed point comes from, and this
//! is the case `docs/decisions.md` §2.5 asks M0 to reach: three planes cross at
//! a point that is not an `f64`, and a fourth plane through it that nothing can
//! prove the point is on.
//!
//! ```
//! use rcad::{Decision, Plane, Point, orient3d_of_points};
//!
//! // x + y + z = 1, x = y and y = z, crossing at (1/3, 1/3, 1/3).
//! let meet = Plane::meet_of_three(
//!     &Plane::from_coefficients([1.0, 1.0, 1.0, -1.0])?,
//!     &Plane::from_coefficients([1.0, -1.0, 0.0, 0.0])?,
//!     &Plane::from_coefficients([0.0, 1.0, -1.0, 0.0])?,
//! )?;
//! assert!(meet.exact_coordinates().is_none(), "a third is not an f64");
//!
//! // These three points span x + y + z = 1, which runs through the meet.
//! match orient3d_of_points(
//!     Point::new(1.0, 0.0, 0.0).into(),
//!     Point::new(0.0, 1.0, 0.0).into(),
//!     Point::new(0.0, 0.0, 1.0).into(),
//!     meet,
//! ) {
//!     Decision::Neg | Decision::Pos => panic!("the box is on both sides at once"),
//!     Decision::Zero => panic!("nothing here can prove the point is on the plane"),
//!     Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
//!     // `Decision` gains variants at every milestone. Guessing what a new one
//!     // means is worse than stopping.
//!     other => panic!("unhandled decision: {other:?}"),
//! }
//! # Ok::<(), rcad::Error>(())
//! ```
//!
//! # What M0 can do
//!
//! **Build.** [`Model::new`] takes a [`Scale`], which says what the numbers
//! mean and roughly how big the part is. [`ProfileBuilder`] checks a flat
//! outline with holes in it and hands back a [`Profile`];
//! [`Model::add_extrusion`] turns that into a solid and places it with an
//! [`Isometry`]. [`Model::add_cube`] is the short way to the obvious case.
//! M0's geometry is planar: [`Plane`] and straight edges
//! (`docs/decisions.md` §3.1). [`Sphere`] and [`Cylinder`] exist as types that
//! answer [`SideOfSurface::side_of`] but do not carry a face yet.
//!
//! **Look.** Every accessor hands back a *view* — [`SolidView`], [`FaceView`],
//! [`EdgeView`], [`VertexView`] and the rest — which carries the borrow
//! together with the location and orientation composed on the way down. There
//! is deliberately no way to reach an entity's stored data, because that would
//! hand out geometry that has not been located
//! (`docs/decisions.md` §3.2, §3.3).
//!
//! **Name.** Every kind of entity has its own key type — [`SolidKey`],
//! [`FaceKey`], [`EdgeKey`] and friends — so handing a face key to something
//! that wants an edge is a compile error. In debug builds a key also remembers
//! which model made it.
//!
//! **Measure.** [`SolidView::measure`] gives volume, surface area and centroid
//! in one pass, straight out of the divergence theorem. No quadrature; planar
//! faces do not need any.
//!
//! **Classify.** [`SolidView::classify_point`] returns a [`Containment`],
//! decided by exact predicates with no tolerance anywhere in the loop.
//! [`SolidView::classify_uncertain_point`] is the same body asked about a
//! [`PredicatePoint`], which may be known only as a box — the meet of three
//! planes. That is where [`Containment::Undecidable`] comes from: a box
//! straddling a face cannot be placed however much precision is spent on it.
//!
//! **Ask.** The predicates those answers are built from are callable on their
//! own: [`orient2d`], [`orient3d`], [`orient3d_of_points`],
//! [`Plane::side_of_point`] and [`SideOfSurface::side_of`] on a [`Sphere`] or a
//! [`Cylinder`]. See *Asking a predicate on its own* above for which of them
//! answers which question.
//!
//! **Check.** [`Model::validate`] is the full sweep over every invariant, meant
//! for test boundaries and for data arriving from outside. Cheap per-entity
//! checks run after every mutation and *panic*, because a corrupt solid is
//! worse than a crash (`docs/decisions.md` §4.1, §4.3).
//!
//! **Store.** [`Model`] is `serde`-serializable. The format is **unstable**,
//! stamped with [`FORMAT_VERSION`], and refuses anything that does not match.
//! Do not archive anything in it (`docs/decisions.md` §6).
//!
//! **Account.** Every operation that can make a decision takes a `&Policy` and
//! a `&mut ProvenanceLog`, and anything it could not certify lands in that log
//! as a [`Record`] — on the way out of a successful operation as much as a
//! failed one. A fully certified operation logs nothing at all, which is itself
//! the signal that nothing was fudged. Slop left behind in the model is an
//! [`Approximation`] tag on the entity carrying it: a tolerant
//! [`ProfileBuilder::build`] hands its slop to the [`Profile`], and
//! [`Model::add_extrusion`] passes it on to every vertex, edge and face it
//! builds from those loops (`docs/decisions.md` §5, §1.2).
//!
//! `&Model` is `Sync`, with no interior mutability and no lazy cache, so all of
//! the reading above parallelizes behind a shared reference with no cooperation
//! from inside the crate (roadmap §10).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;

pub use crate::error::{Error, Result};

pub use rcad_geom::Error as GeometryError;
pub use rcad_predicates::Error as PredicateError;
pub use rcad_topo::Error as TopologyError;

pub use rcad_geom::{
    Axis, Cylinder, Isometry, Orientation, Plane, PlaneAgreement, Point, Scale, Segment, Sphere,
    Surface, Unit, Vector,
};
pub use rcad_predicates::{
    Approximation, Cause, Decision, Level, Measures, ORIENT2D, ORIENT3D, Point as PredicatePoint,
    Policy, ProvenanceLog, Record, Severity, SideOfSurface, Strictness, orient2d,
    orient2d_tolerantly, orient3d, orient3d_of_points, orient3d_tolerantly,
};
pub use rcad_topo::{
    CoedgeKey, CoedgeView, Containment, EdgeKey, EdgeView, FORMAT_VERSION, FaceKey, FaceRef,
    FaceView, Fused, Invalid, Measurement, Model, Profile, ProfileBuilder, Rule, ShellKey,
    ShellView, SolidKey, SolidView, VertexKey, VertexView, WireKey, WireView,
};
