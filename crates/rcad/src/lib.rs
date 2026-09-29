//! A tolerant B-Rep CAD kernel.
//!
//! This crate is the front door. It re-exports the whole public surface and
//! nothing else: `rcad-predicates`, `rcad-geom`, `rcad-mesh`, `rcad-topo` and
//! `rcad-step` are implementation details of how the kernel is built and how
//! it reads and writes a STEP file — not crates a caller should have to name
//! directly (`docs/decisions.md` §3.6, §7.2).
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
//! [`Deflection`] is the *other* knob and it is not this one. It says how far a
//! mesh may stray from the surface it stands for — how fine a picture you want
//! — and a coarse answer is a request rather than a failure.
//! [`SolidView::mesh`] and [`SolidView::measure`] both take one, and a caller
//! who reached for [`Policy`] to ask for fewer triangles would have silently
//! asked the kernel to certify less (`docs/decisions.md` §23.5, §25.8).
//!
//! ```
//! use rcad::{Deflection, Scale};
//!
//! // A thousandth of the nominal size, which for a 100 mm part is 0.1 mm.
//! let scale = Scale::millimetres(100.0)?;
//! assert_eq!(Deflection::for_scale(scale).length(), 0.1);
//! assert_eq!(Deflection::of(0.25)?.length(), 0.25);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # A worked example
//!
//! Build a cube, measure it, ask where a point sits, and write it out and read
//! it back.
//!
//! ```
//! use rcad::{Containment, Deflection, Model, Point, Policy, ProvenanceLog, Scale};
//!
//! let scale = Scale::millimetres(100.0)?;
//! let mut model = Model::new(scale);
//! let mut log = ProvenanceLog::new();
//! let cube = model.add_cube(10.0, &Policy::certified(), &mut log)?;
//! assert!(log.is_empty(), "a cube of side ten needs nothing approximated");
//!
//! model.validate(&Policy::certified(), &mut log)?;
//! assert!(log.is_empty(), "a clean sweep records nothing at all");
//!
//! // A cube is a polyhedron: its own fan is the answer, so the deflection is
//! // ignored and nothing here is approximated.
//! let how_fine = Deflection::for_scale(scale);
//! let measured = model
//!     .solid(cube)?
//!     .measure(how_fine, &Policy::certified(), &mut log)?;
//! assert_eq!(measured.volume(), 1000.0);
//! assert_eq!(measured.surface_area(), 600.0);
//! assert_eq!(measured.centroid(), Point::new(5.0, 5.0, 5.0));
//! // Each error is how far the fan's own sums rounded, and on this cube
//! // nothing did. Whole-number corners alone do not promise that: an L
//! // extruded from a four-by-four square less a two-by-two corner has its
//! // middle at five thirds along `x` and `y`, which no `f64` holds, and
//! // publishes a wobble of `1.05e-16` (`docs/decisions.md` §338.5).
//! assert_eq!(measured.volume_error(), 0.0, "a cubed unit");
//! assert_eq!(measured.area_error(), 0.0, "a squared one");
//! assert_eq!(measured.centroid_wobble(), 0.0, "and a length");
//!
//! let policy = Policy::certified();
//! match model
//!     .solid(cube)?
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
//! assert_eq!(
//!     same_cube.measure(how_fine, &Policy::certified(), &mut log)?,
//!     measured
//! );
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
//!
//! Curved solids come three ways and the third is the one that is not a
//! declaration. [`Model::add_round_bar`] and [`Model::add_ball`] *say* what
//! their edges run along, out of a person's knowledge that a plane square to a
//! pipe cuts a circle. [`Model::add_bar_cut_off_by`] **asks** — it puts each cap
//! plane and the wall to [`intersect`] and writes down whatever comes back — so
//! it reaches a shape the other two cannot make: a bar cut off at an angle,
//! whose cap edge is an ellipse (`docs/decisions.md` §24.2).
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
//! in one pass, straight out of the divergence theorem, each with how far out it
//! could be. A polyhedron is summed over its own fan — no quadrature, no mesh,
//! and its three errors are how far those sums rounded, exactly zero where
//! nothing did (`docs/decisions.md` §338) — and a solid with a curved face is
//! summed over a [`Mesh`], where [`Measurement::volume_error`],
//! [`Measurement::area_error`] and [`Measurement::centroid_wobble`] say what the
//! triangles could be hiding. Three quantities in three units, deliberately
//! (`docs/decisions.md` §25.4). [`EdgeView::length`] and [`WireView::length`]
//! give how long an edge or a wire is as a [`Length`], with the same bound on
//! how far out it could be: straight edges and circles, and a refusal by name
//! for every other curve rather than a guess.
//!
//! **Mesh.** [`SolidView::mesh`] cuts a whole solid into [`MeshTriangle`]s that
//! stay within a [`Deflection`] of its surfaces, through one point table for the
//! whole solid — which is what makes the result watertight by index rather than
//! by tolerance (§23.4, §25.7). Each triangle says which [`FaceKey`] it stands
//! in for.
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
//!
//! # Reading and writing a STEP file
//!
//! [`StepFile::read`] turns the bytes of an ISO 10303-21 ("STEP", "Part 21")
//! file into an entity graph, and [`Import::read`] walks that graph's solids
//! into a [`Model`], or refuses by a [`Refusal`] that names the entity that
//! stopped it and why. [`write`] goes back the other way. `rcad-step` is a real
//! dependency of this crate, not a second one a caller has to add and match a
//! second error type against: [`StepSyntaxError`], [`Refusal`] and
//! [`CannotWrite`] all convert into [`Error`], through [`Error::StepSyntax`],
//! [`Error::StepImport`] and [`Error::StepWrite`].
//!
//! ```
//! use rcad::{Deflection, Import, Model, Policy, ProvenanceLog, Scale, StepFile, write};
//!
//! let scale = Scale::millimetres(10.0)?;
//! let mut model = Model::new(scale);
//! let mut log = ProvenanceLog::new();
//! model.add_cube(10.0, &Policy::certified(), &mut log)?;
//!
//! // Out to a Part 21 file and back, naming nothing but `rcad`.
//! let text = write(&model, "a cube round-tripped through rcad alone")?;
//! let file = StepFile::read(text.as_bytes())?;
//! let import = Import::read(&file, &Policy::certified(), &mut log)?;
//!
//! let [(_, key)] = import.solids() else {
//!     panic!("one solid written is one solid read back")
//! };
//! let measured = import.model().solid(*key)?.measure(
//!     Deflection::for_scale(scale),
//!     &Policy::certified(),
//!     &mut log,
//! )?;
//! assert_eq!(measured.volume(), 1000.0);
//! # Ok::<(), rcad::Error>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;

pub use crate::error::{Error, Result};

pub use rcad_geom::Error as GeometryError;
pub use rcad_predicates::Error as PredicateError;
pub use rcad_topo::Error as TopologyError;

/// Why one face could not be cut into triangles.
///
/// Out here because [`TopologyError::CannotMeshAFace`] carries one, and a caller
/// that wants to know *which* refusal it was would otherwise have to name
/// `rcad-mesh` — the dependency this facade exists to hide
/// (`docs/decisions.md` §3.6, §7.2).
pub use rcad_mesh::CannotTriangulate;

/// What crossing two surfaces comes back with.
///
/// Out here ahead of the intersector that produces it, because these are the
/// shapes M3's two stages both write against and §13 rule 4 says a meaning two
/// people settle separately gets settled twice, differently. Quadric
/// intersection with certified results and an explicit refusal has users who are
/// not kernel users, which is §24.15's argument for exporting it at all.
pub use rcad_geom::{EdgeRun, NamedRun, SurfaceMeet};

/// What picks one piece of a crossing out of the rest, and the flat kind of it.
///
/// A [`Picker`] is out here because [`EdgeRun::PartOfWhereItCrosses`] holds a
/// list of them, so a caller matching on a run meets one. [`HalfSpace`] is out
/// here because building a run means building one — and because its whole point
/// is that it stores the four numbers it is handed, which a caller that wants a
/// normal exactly square to a direction it holds has to be able to rely on
/// (§33).
pub use rcad_geom::{HalfSpace, Picker, SIDE_OF_A_HALF_SPACE};

/// Naming where two surfaces cross, and working out points along it.
///
/// §24.15: the entry point has no topology in its signature, so exporting it
/// costs nothing, and quadric intersection with certified results and an
/// explicit `Undecidable` has users who are not kernel users. The `along`
/// bookkeeping — which input edges a piece of a crossing runs along — stays
/// inside `rcad-topo`, because those name keys. [`ROOM_LEFT_TO_REFINE_A_WALK`]
/// is [`walk`]'s own provenance name, out here for the same reason as the
/// `SIDE_OF_*` names below (§15.6): a caller reading its own log needs the
/// name, not a copy of the string.
pub use rcad_geom::{
    MOST_POINTS_ALONG_ONE_CROSSING, POINT_ALONG_A_CROSSING_ROUNDOFF, PointsAlongACrossing,
    ROOM_LEFT_TO_REFINE_A_WALK, WHERE_A_BALL_AND_A_PIPE_CROSS, WHERE_A_PLANE_AND_A_BALL_CROSS,
    WHERE_A_PLANE_AND_A_PIPE_CROSS, WHERE_TWO_BALLS_CROSS, WHERE_TWO_PIPES_CROSS,
    WHERE_TWO_PLANES_CROSS, intersect, walk, where_a_closed_crossing_starts,
};

/// Where three surfaces meet, how many points that is, and which of two points
/// on a closed crossing comes first.
///
/// Out here for the same reason as [`intersect`]: none of the three has any
/// topology in its signature, and a caller that wants the corner of a curved
/// arrangement without a kernel is the user §24.15 exports for. Where a corner
/// read out of a model lands once it has been placed is here too, because it is
/// what a caller has to use instead of moving the point itself (§31.3).
pub use rcad_geom::{
    CORNER_AFTER_MOVING_ROUNDOFF, MeetOfThree, ROUND_A_CROSSING, WHERE_THREE_SURFACES_MEET,
    meet_of_three_surfaces, where_a_corner_lands_after_moving, which_comes_first_round_a_crossing,
};

pub use rcad_geom::{
    Axis, BSplineCurve, BSplineSurface, Cone, Cylinder, Deflection, EVALUATION_ROUNDOFF, Isometry,
    KnotVector, MAX_DEGREE, Orientation, Plane, PlaneAgreement, Point, REFINEMENT_ROUNDOFF, Scale,
    Segment, Sphere, Surface, Torus, Unit, Vector,
};

/// The name each surface's sign predicate goes by in a provenance record.
///
/// [`Record::predicate`] hands back one of these, and until all six were out
/// here a caller reading its own log could compare it against a string literal
/// and nothing else — the one bound of the six the facade did export
/// ([`SIDE_OF_A_HALF_SPACE`]) is for a picker rather than for a surface, so
/// every surface the kernel supports was a literal. These are published bounds
/// on results the caller was handed, and a caller cannot name what its own
/// results were certified against if the name is not exported (§15.6).
pub use rcad_geom::{SIDE_OF_CONE, SIDE_OF_CYLINDER, SIDE_OF_PLANE, SIDE_OF_SPHERE, SIDE_OF_TORUS};

/// How far apart a model's coordinates may be, and the name that check goes by
/// in a provenance record.
///
/// Both halves of §15.6 for one check. The bound is
/// [`DynamicRange::LARGEST_SUPPORTED_RATIO`], and a caller handed
/// [`Invalid::DynamicRangeTooWide`] reads a ratio out of it that means nothing
/// on its own: with no bound out here there is nothing to compare it against and
/// no way to reach the same ratio from the model it came from, so copying the
/// digits of a constant that can move is the only route left.
///
/// [`DYNAMIC_RANGE_CHECK`] is the tolerant half of the same check. Under
/// [`Policy::tolerant`] it warns instead of refusing, and what it leaves behind
/// is a [`Record`] whose [`predicate`](Record::predicate) is this name — so a
/// caller reading its own log can count records without it and say which check
/// they came from with it. Out here for the same reason as the `SIDE_OF_*` names
/// above.
pub use rcad_geom::{DYNAMIC_RANGE_CHECK, DynamicRange};
pub use rcad_predicates::{
    Approximation, Cause, Decision, Interval, Level, Measures, ORIENT2D, ORIENT3D,
    Point as PredicatePoint, Policy, ProvenanceLog, Record, Severity, SideOfSurface, Strictness,
    orient2d, orient2d_tolerantly, orient3d, orient3d_of_points, orient3d_tolerantly,
};
pub use rcad_topo::{
    CoedgeKey, CoedgeView, Containment, DraftCoedge, DraftEdge, DraftFace, DraftStep, DraftVertex,
    DraftWire, EdgeKey, EdgeView, FORMAT_VERSION, FaceKey, FaceMeasurement, FaceRef, FaceView,
    Fused, Invalid, Length, Measurement, Mesh, MeshTriangle, Model, OnAFace, Profile,
    ProfileBuilder, ROOM_FOR_ATAN2, Rule, ShellKey, ShellView, SolidBuilder, SolidKey, SolidView,
    VertexKey, VertexView, WhichModelThisIs, WireKey, WireStepView, WireView,
};

/// Reading and writing an ISO 10303-21 ("STEP", "Part 21") file.
///
/// Out here so a caller who wants to read or write one names only `rcad`: see
/// *Reading and writing a STEP file* above. [`Error`]'s own rustdoc says how
/// [`StepSyntaxError`], [`Refusal`] and [`CannotWrite`] land in it.
///
/// [`rcad_step::Error`] is renamed [`StepSyntaxError`] on the way out — it is
/// what [`StepFile::read`] returns, and the bare name `Error` is already this
/// crate's own. [`Refusal`], [`Problem`] and [`CannotWrite`] need no such
/// rename: none of those three names was taken.
///
/// [`EntityId`] and [`Problem`] are out here too, and for the same reason
/// [`Invalid`] already is: [`Refusal`] names its entity by one and
/// [`StepSyntaxError`] names its problem by the other, so a caller matching
/// either down to a variant needs both in scope. What is *not* out here is the
/// entity graph itself — [`StepFile::instances`](rcad_step::StepFile::instances)
/// and its `Header`, `Instance`, `Record` and `Value` — because reading a file
/// into a `Model` is the consumer this facade has; walking the raw graph by
/// hand is `rcad-step`'s own, more advanced, surface.
pub use rcad_step::{
    CannotWrite, EntityId, Error as StepSyntaxError, Import, Problem, Refusal, StepFile, write,
};
