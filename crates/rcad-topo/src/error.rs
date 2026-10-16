use rcad_predicates::ProvenanceLog;

use crate::EdgeKey;

/// What went wrong.
///
/// Bad caller input and broken models are errors. A predicate that could not
/// decide is *not* an error on its own — that is
/// [`Decision::Undecidable`](rcad_predicates::Decision::Undecidable) and it is
/// a result. It only becomes [`Error::Undecidable`] when a whole operation is
/// left with nothing to return (decisions.md §4.2).
///
/// A broken internal invariant is neither: it panics.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A piece of geometry refused to be built.
    #[error("geometry: {0}")]
    Geometry(rcad_geom::Error),

    /// A key made by one model was handed to a different one.
    ///
    /// `slotmap`'s generation counter catches a key whose slot was freed and
    /// refilled *in the same* model. It cannot see a key from another live
    /// model reaching a live slot in this one, and slot zero of any model is a
    /// slot every model has — so the wrong answer this stops is a plausible
    /// one, not a crash: a one millimetre cube's key measured 1000 mm³ against
    /// a model holding a ten millimetre block (decisions.md §82.10).
    ///
    /// It is bad caller input rather than a broken invariant, because which
    /// model a key came from is not something this crate chose. It was a panic
    /// until §86, on the grounds that the panic was the smaller fix; every one
    /// of the calls that can raise it reads and writes nothing before it does,
    /// which is the case §4.1's amendment under §66 wants an `Err` for.
    #[error("this {entity} key was made by a different model")]
    KeyFromAnotherModel {
        /// Which kind of entity the key names: `"vertex"`, `"edge"`,
        /// `"coedge"`, `"wire"`, `"face"`, `"shell"` or `"solid"`.
        entity: &'static str,
    },

    /// A profile loop has fewer than three points, so it encloses nothing.
    #[error("a profile loop needs at least three points, this one has {count}")]
    TooFewPoints {
        /// How many points were given.
        count: usize,
    },

    /// A profile point is infinite or not a number.
    #[error("the profile point at position {position} is not a finite number")]
    PointNotFinite {
        /// Where in the loop the point sits.
        position: usize,
    },

    /// Two points in a row are the same, which would make an edge of no length.
    #[error("the profile repeats the point at position {position}, making an edge of no length")]
    RepeatedPoint {
        /// The position of the second of the two identical points.
        position: usize,
    },

    /// Two profile edges touch or cross where they should not.
    #[error("the profile crosses itself")]
    ProfileCrossesItself,

    /// A hole is not wholly inside the outline it was given with.
    #[error("hole {hole} is not inside the outline")]
    HoleNotInsideOutline {
        /// Which hole, counting from zero.
        hole: usize,
    },

    /// One hole sits inside another. Holes have to be side by side.
    #[error("hole {hole} is inside another hole")]
    HoleInsideAnotherHole {
        /// Which hole, counting from zero.
        hole: usize,
    },

    /// An extrusion was asked for with a height of zero or less.
    #[error("an extrusion needs a positive finite height, got {height}")]
    HeightNotPositive {
        /// The height that was asked for.
        height: f64,
    },

    /// A cap plane does not cut one closed curve out of the bar's wall.
    ///
    /// A plane running down the inside of a pipe cuts two straight lines, one
    /// along its skin cuts one, and one clear of it cuts nothing. None of the
    /// three is a cap, and which of them it is came back from the intersector
    /// rather than from anybody's assumption.
    #[error("cap {which} does not cut right round the bar")]
    ACapDoesNotCutRightRoundABar {
        /// Which cap: 0 is the bottom and 1 the top.
        which: usize,
    },

    /// A cap plane's normal points along the axis towards the other cap, so the
    /// bar it would make is inside out.
    #[error("cap {which} faces into the bar rather than out of it")]
    ACapOfABarFacesIntoIt {
        /// Which cap: 0 is the bottom and 1 the top.
        which: usize,
    },

    /// The two caps of a bar cross inside it, so there is no body between them.
    #[error("the two caps of this bar meet inside it")]
    TheCapsOfABarMeetInsideIt,

    /// A solid encloses more space than an `f64` can hold.
    ///
    /// Not a broken model. A cube of side `1e300` has its faces pointing
    /// outwards and classifies points correctly; what it does not have is a
    /// volume of `1e900` that fits in the type it would be returned in.
    ///
    /// **Both kinds, so [`Error::limit_or_gap`] says not known** (§16.10).
    /// [`SolidView::measure`](crate::SolidView::measure) of a cube of side
    /// `1e103` is a *limit*: the answer is a volume of `1e309`, and no ordering
    /// and no rescaling of the arithmetic reaches a number `f64` cannot
    /// represent. From a fuse it is a
    /// *gap*: uniting a cube of side `2^342` with its copy is refused, though
    /// the answer is that same cube and the model already holds it, because the
    /// fuse measures every shell it writes and refuses one whose volume
    /// overflows.
    #[error("this solid encloses more space than an f64 can hold")]
    VolumeTooLargeToMeasure,

    /// A solid has more skin than an `f64` can hold.
    ///
    /// Reachable without [`Error::VolumeTooLargeToMeasure`], because area and
    /// volume grow at different rates: a slab that is wide and thin can run out
    /// of room for its area while its volume still fits.
    ///
    /// **Both kinds, so [`Error::limit_or_gap`] says not known** (§16.10). A
    /// *limit* when the area is past `f64::MAX`: a face of a cube of side
    /// `1e200` is `1e400`. A *gap* when the area is under it and the rounded
    /// sum of the face's fan is not: a parallelogram whose area, worked out
    /// exactly from its corners, is just under `f64::MAX` is refused by
    /// [`SolidView::measure_one_face`](crate::SolidView::measure_one_face).
    #[error("this solid has more surface area than an f64 can hold")]
    SurfaceAreaTooLargeToMeasure,

    /// An edge or a wire is longer than an `f64` can hold.
    ///
    /// **Both kinds, so [`Error::limit_or_gap`] says not known** (§16.10). A
    /// *limit* when the length is past `f64::MAX`: two corners at `-1e308` and
    /// `1e308` are `2e308` apart, and no ordering or rescaling of the
    /// arithmetic hands back a number `f64` cannot represent.
    /// [`EdgeView::length`](crate::EdgeView::length) already rescales by a power
    /// of two so that nothing short of that overflows on the way. A *gap* when
    /// the length is an `f64` the bounds round past: a face of a cube of side
    /// `f64::MAX / 4` is exactly `f64::MAX` round, and
    /// [`WireView::length`](crate::WireView::length) rounds the sum of its four
    /// sides outwards, past `f64::MAX`.
    #[error("this edge or wire is longer than an f64 can hold")]
    LengthTooLargeToMeasure,

    /// How long this edge is has an answer, and nothing here works it out yet.
    ///
    /// **A gap and not a limit** (§16.10). An ellipse, every other curve a plane
    /// leaves on a quadric, and a degree-four crossing of two quadrics all have
    /// a length that is an ordinary `f64`;
    /// [`EdgeView::length`](crate::EdgeView::length) reaches circles only — a
    /// plane square to a pipe, and a plane through a ball — by the closed form,
    /// radius times angle. A circle whose squareness cannot be proved exactly,
    /// and an arc whose two candidate pieces its pickers cannot tell apart at
    /// their midpoints, land here too: both lengths exist and this code did not
    /// reach them.
    ///
    /// What would reach the rest, family by family:
    ///
    /// - **Every crossing that lies in a plane.** A plane crosses a quadric in a
    ///   conic, and a conic is convex, so a polygon inscribed in it is shorter
    ///   than it and the polygon of tangents at the same points is longer
    ///   (Archimedes). [`walk`](rcad_geom::walk) already gives the points, the
    ///   gap between the two polygons shrinks as the square of the step — about
    ///   the deflection over the radius, relatively — and it stops at the walk's
    ///   [`MOST_POINTS_ALONG_ONE_CROSSING`](rcad_geom::MOST_POINTS_ALONG_ONE_CROSSING).
    /// - **A whole ellipse** has a closed form besides: the complete elliptic
    ///   integral of the second kind, which the arithmetic-geometric mean gives
    ///   to full precision in a handful of steps. An arc of one wants the
    ///   incomplete integral, which is the walk again.
    /// - **A degree-four crossing** is not flat, so the tangent polygon bounds
    ///   nothing. The inscribed polygon is still a lower bound — it is for any
    ///   curve — and the upper bound wants how far the curve can bend between
    ///   two walked points, which is the same bound the walk's deflection check
    ///   already has to make.
    #[error("nothing here works out yet how long this edge is along its curve")]
    CannotYetMeasureHowLongThisCurveIs {
        /// Which edge.
        edge: EdgeKey,
    },

    /// The model breaks one of the invariants [`validate`] checks.
    ///
    /// [`validate`]: crate::Model::validate
    #[error("the model is broken: {0}")]
    Invalid(#[from] Invalid),

    /// This kernel built an answer that does not validate, and threw it away
    /// rather than hand it over.
    ///
    /// **It does not say whether the fault is this kernel's or the input's.** A
    /// fuse does not ask [`Model::validate`] of its inputs, and both have been
    /// seen to end here:
    ///
    /// - **Sound inputs, and a gap here.** A quarter bend described through
    ///   `SolidBuilder`, united with a second one built from the same numbers,
    ///   was written back with both coedges of one edge running the same way,
    ///   `broken` [`Invalid::CoedgesRunTheSameWay`] (§243.5), while the mesher
    ///   read a torus facing into its tube and the bend's wall was used turned
    ///   round to suit it. Built the way a file writes a bend, it answers
    ///   (§361).
    /// - **Sound inputs a rounding apart.** A cube turned so that its corner
    ///   rests on another cube's face, with its three planes meeting a rounding
    ///   outside that face, leaves the fuse a triangle a rounding across. The
    ///   union ends here with `broken` [`Invalid::HoleWindsWithTheOuterWire`],
    ///   and the intersection under a tolerant policy with
    ///   [`Invalid::OuterWireWindsAgainstTheNormal`]
    ///   (`a_turned_cube_with_its_corner_on_a_face.rs`, Q1287). The answer the
    ///   designer means exists, so under a tolerant policy that is a gap
    ///   (Q1288).
    /// - **A broken input, carried through.** A round bar whose top corner was
    ///   moved a millimetre off its surfaces in the model's bytes, read back and
    ///   fused under `Policy::certified` against a partner that leaves that
    ///   corner in the answer — six fuses, three moves against two partners —
    ///   comes back as this every time, with `broken` equal to the [`Invalid`]
    ///   that [`Model::validate`] gives the bar before any fuse runs (§294.2).
    ///   There the refusal blames the fuse for the caller's own file. Pinned by
    ///   `a_fuse_that_keeps_the_moved_corner_blames_itself_for_the_broken_file`
    ///   in `a_broken_two_surface_corner_read_back_from_bytes.rs`.
    ///
    /// Asking [`Model::validate`] of the inputs tells the two apart, and a fuse
    /// leaves that to the caller on purpose. The same broken bar fused against a
    /// partner that cuts the moved corner away gives the answer the sound bar
    /// gives, in all twenty such fuses §294.2 counts, and §294.5 says why asking
    /// at the door would refuse those twenty right answers too. The same file
    /// pins them:
    /// `a_fuse_that_cuts_the_moved_corner_away_gives_the_answer_the_sound_bar_gives`
    /// eighteen, and
    /// `a_cut_along_the_seam_refuses_a_corner_moved_in_the_cut_and_answers_one_moved_out_of_it`
    /// the other two.
    ///
    /// It is separate from [`Error::Invalid`], which is what a caller gets when
    /// the *description* they gave is refused before anything is built —
    /// [`SolidBuilder::build`](crate::SolidBuilder::build) returns that for a
    /// corner off its own plane. This one refuses what a fuse built.
    ///
    /// A fuse works its answer out in a throwaway model and sweeps that
    /// (`docs/decisions.md` §87), so a caller who sees this has a model exactly
    /// as it was before the call. What it means is that a fuse gave up where it
    /// used to crash: §4.1 wants a panic for a broken invariant, and this is the
    /// case §66.4 asked to be carved out of that — nothing is corrupt, because
    /// nothing was written.
    ///
    /// [`Invalid::DynamicRangeTooWide`] never arrives wrapped in here, because
    /// the sweep of the throwaway model does not ask the coordinate range —
    /// twelve of `Model::validate`'s thirteen checks, and that is the
    /// thirteenth. A fuse asks it once, in `Assembly::check_dynamic_range` and
    /// behind a guard, so that a caller whose own solids sit further apart than
    /// `f64` can resolve still gets the answer the arithmetic reached; §87.3 has
    /// the argument. Everything else the sweep refuses is wrapped in here, with
    /// no list of exceptions to keep honest.
    ///
    /// **One arrives before the sweep.** A face joined from coplanar pieces
    /// whose rings include two that both wind as its outline is refused while
    /// the pieces are joined, since nothing after that can say which ring is
    /// the hole. `broken` is then [`Invalid::HoleWindsWithTheOuterWire`], what
    /// the sweep would say of that face written down, and the log gets a
    /// record holding both rings' corners.
    ///
    /// [`Model::validate`]: crate::Model::validate
    #[error("this kernel built an answer that does not validate, and wrote nothing: {broken}")]
    TheFuseWroteABrokenAnswer {
        /// What the sweep of the throwaway model refused the answer for, or
        /// would have.
        broken: Invalid,
    },

    /// A serialized model was written by a different version of this format.
    #[error("this model was written in format version {found}, and we read version {expected}")]
    WrongFormatVersion {
        /// The version the bytes claim.
        found: u32,
        /// The only version this build understands.
        expected: u32,
    },

    /// One of the solids handed to a fuse runs through itself: two of its own
    /// faces cross somewhere that is not an edge they share.
    ///
    /// Deliberately not something [`Model::validate`] looks for. Every face
    /// against every other is quadratic in the face count, and a fuse is
    /// already working out where faces meet, so it notices this for free
    /// (decisions.md §17.4, §4.3).
    ///
    /// [`Model::validate`]: crate::Model::validate
    #[error("one of the solids handed over runs through itself")]
    ArgumentIntersectsItself,

    /// The result of a fuse would have an edge with something other than two
    /// faces along it, so it is not a solid this kernel can hold.
    ///
    /// A `Model` means an edge is used by exactly two coedges everywhere, and
    /// relaxing that changes what a model *is* rather than what one boolean does
    /// (decisions.md §17.4).
    #[error("the result would not be a solid: an edge would have {count} faces along it")]
    ResultIsNotManifold {
        /// How many faces would meet along the first bad edge found. The check
        /// stops there, so a worse edge may be sitting further along.
        count: usize,
    },

    /// One face of a solid could not be cut into triangles.
    ///
    /// Carries what the triangulator said. Most of those are about the
    /// *request*: this deflection needs more triangles than one patch will
    /// hold, or the loops handed over are not loops. The fix is a coarser mesh,
    /// or a sounder solid.
    ///
    /// Two of them are not about the request, and are reached from here only by
    /// a solid whose own numbers are extreme.
    /// [`CannotTriangulate::CoordinateOutOfRange`] is a triangulation that takes
    /// a narrower range than `f64`, and
    /// [`CannotTriangulate::NoMeshCanGetThatCloseToTheSurface`] is the floor
    /// [`Error::DeflectionBelowWhatAMeshCanReach`] normally catches first,
    /// arriving here in the hair's breadth above it where one face rounds a
    /// little worse than the solid does.
    ///
    /// [`CannotTriangulate::CoordinateOutOfRange`]: rcad_mesh::CannotTriangulate::CoordinateOutOfRange
    /// [`CannotTriangulate::NoMeshCanGetThatCloseToTheSurface`]: rcad_mesh::CannotTriangulate::NoMeshCanGetThatCloseToTheSurface
    #[error("a face of this solid could not be cut into triangles: {why}")]
    CannotMeshAFace {
        /// Which face.
        face: crate::FaceKey,
        /// What the triangulator said about it.
        why: rcad_mesh::CannotTriangulate,
    },

    /// A wire the boundary walk placed does not fan into a loop the
    /// triangulator can use — fewer than three distinct corners in the
    /// surface's own two coordinates, or one that is not a finite number.
    ///
    /// **There used to be two ways to reach it. The first was a gap and it is
    /// closed.** A face bounded by an arc and its chord has two corners in
    /// space, and `rcad_geom::walk` never took fewer than one step — so at the
    /// deflection [`Model::validate`] meshes with, an arc of a quarter turn or
    /// less put nothing between its ends and the face fanned into two corners.
    /// The walk now takes at least two pieces of any bend, because one piece
    /// shows **none** of it at any deflection and hands back the chord, which
    /// is what a straight run already is.
    ///
    /// The other way is bad input, and a panic is the wrong answer to it. A
    /// face's *stored surface* is data, not a fact this checks before walking
    /// it: a wall face carrying a
    /// [`Plane`](rcad_geom::Plane) where its own [`Cylinder`](rcad_geom::Cylinder)
    /// belongs still has a four-step wire in space, and the same wire fans
    /// into one corner in the plane's own two coordinates instead of four.
    /// `Model` derives
    /// `Deserialize` and nothing between a file and this call has agreed the
    /// surface matches the wire (CLAUDE.md's rule on panics vs. bad caller
    /// input, not §16.11, which is about the dynamic-range check being a
    /// ratio and says nothing about this).
    #[error("a wire of this face does not fan into a loop the triangulator can use: {why}")]
    AWireDoesNotFanIntoALoopToMesh {
        /// Which face.
        face: crate::FaceKey,
        /// What was wrong with the corners it fanned into.
        why: rcad_mesh::NotALoop,
    },

    /// The deflection asks for a mesh closer to the surfaces than a point of one
    /// can be placed.
    ///
    /// A point on a curved surface comes out of an evaluator that rounds, and is
    /// then carried into place by the solid's own placement, which rounds again.
    /// A mesh made of those points is no nearer the surface than they are, so a
    /// deflection under that is not a request for more triangles: it is a
    /// request for arithmetic `f64` does not have. The answer does not exist,
    /// which makes this a *limit* and not a §16.10 gap.
    ///
    /// **The floor depends on where the solid sits and not only on how big it
    /// is**, because both roundings are charged against how far out the surface
    /// reaches. The same ball a kilometre from the origin cannot be meshed as
    /// finely as one at the origin.
    ///
    /// **And a scaling reaches it, in the one direction §18 allows.** The floor
    /// is a fixed multiple of the solid's own coordinates, so shrinking the
    /// whole model shrinks the floor in exact step: what is refused is an
    /// absolute deflection under the spacing of the `f64` grid where the solid
    /// sits, and never a *relative* one. A point cannot be put between two
    /// neighbouring `f64`s by reordering the arithmetic, by rescaling it, or by
    /// summing it exactly, so the family of fixes is empty rather than
    /// unexamined.
    ///
    /// # Two reasons under one name, and only the first is a limit
    ///
    /// Everything above is about *rounding*, and it is a limit. The second
    /// reason is the **step cap**: `rcad_geom::walk` will not put more than
    /// `MOST_POINTS_ALONG_ONE_CROSSING` points along one crossing, so there is a
    /// finest deflection it will take whatever the rounding allows, and it is
    /// the larger of the two. That one is a §16.10 **gap and not a limit** — the
    /// answer exists, `f64` reaches it, and a bigger cap would hand it back. It
    /// is deferred because the mesh at the cap is already a million points along
    /// one circle and nobody has asked for two.
    ///
    /// **`closest` carries the second of those.** It is the finest deflection
    /// the walk will take along this solid's crossings, and not the finest a
    /// point of it can be placed to — because this is refused only where the
    /// mesher was picking the number itself, and what it picks goes straight to
    /// the walk with no-one else to look at it. The two floors are 78.93 times
    /// apart for a curved solid at the origin: a ball of radius one there has a
    /// point placed to `3.0716170347962663e-13` and walked to
    /// `2.4244102800489844e-11`. A caller who names a number between them is not
    /// refused here — a number a caller chose is that caller's to answer for.
    ///
    /// Both halved when a ball stopped being charged from its poles and started
    /// being charged from its centre; the 78.93 between them did not move,
    /// because it belongs to the term the walk adds and not to either shape.
    /// `how_far_apart_the_two_floors_under_a_mesh_are.rs` is the test under all
    /// three.
    #[error(
        "no mesh of this solid gets closer than {closest} to it, and the deflection asks {asked}"
    )]
    DeflectionBelowWhatAMeshCanReach {
        /// What was asked for, in the model's own unit.
        asked: f64,
        /// The finest deflection [`rcad_geom::walk`] will take along this
        /// solid's crossings, in the same unit — not the closest one of its mesh
        /// points can be placed, which is smaller.
        closest: f64,
    },

    /// A wire round a face will not lie flat in that face's own two
    /// coordinates.
    ///
    /// Two ways one can fail to. It can come back to a different longitude from
    /// the one it set off at, so the loop handed to the triangulator would not
    /// close; or it can stand still at a pole on a surface that does not pinch
    /// to one, which a plane never does.
    ///
    /// **A refusal and not an assert**, and the difference is
    /// [`Model`](crate::Model)'s `Deserialize`. Neither shape is one a builder
    /// here makes, and neither is one anything earlier in
    /// [`Model::validate`](crate::Model::validate) looks at: a ball missing one
    /// of its two pole steps is three wire steps that close up as topology, and
    /// a flat face with a pole tacked onto its wire winds exactly as it did
    /// before. A panic is the wrong answer to a caller's bytes (§4.1).
    #[error("a wire of this face will not lie flat in the face's own two coordinates")]
    AWireWillNotLieFlatOnItsFace {
        /// Which face.
        face: crate::FaceKey,
    },

    /// Two patches of the mesh drew the same side through their insides, so the
    /// mesh would come back with that side used more than twice.
    ///
    /// A side the triangulator draws inside a patch belongs to that patch alone,
    /// and a third use of one is two patches folded onto each other — a flat
    /// pocket with nothing inside it (`docs/decisions.md` §187.4). A leaning
    /// cap and its wall did it before the triangulator learned not to cut a
    /// corner inside one edge.
    ///
    /// **A refusal and not an assert, because a caller's bytes reach it.** A
    /// ball whose wire takes a pole step at a corner standing on the ball's
    /// centre, where the surface pinches nowhere, collapses a whole quarter of
    /// the equator to that one corner, and both faces of the slit it is cut
    /// along then draw the same two triangles. Nothing in
    /// [`Model::validate`](crate::Model::validate) asks where a pole step's
    /// corner stands, and a panic is the wrong answer to a caller's bytes
    /// (§4.1).
    #[error(
        "two patches of the mesh drew the same side through their insides: {drawn} times, \
         and {used} uses in all where there should be two"
    )]
    TwoPatchesOfTheMeshDrewTheSameSide {
        /// How many triangles drawn inside a patch use the side.
        drawn: u32,
        /// How many triangles of the whole mesh use it.
        used: u32,
    },

    /// Moving this solid's mesh into place took away the space it encloses.
    ///
    /// A placed curved face is meshed in the frame it is stored in, and its
    /// points are then moved out to where the solid sits. Out where one ulp of
    /// the placement is as wide as the solid, that movement rounds a whole side
    /// of it onto one coordinate: a unit ball moved to
    /// `x = 1.0808639105689192e16` is a sound octahedron where it is meshed, and
    /// every one of its points lands on the plane `x = 1.0808639105689192e16`,
    /// so each triangle lies on a twin and the mesh encloses nothing
    /// (`docs/decisions.md` §333.6). Handed back, that mesh is a wrong answer
    /// that no check of distance or topology catches.
    ///
    /// **Raised by [`SolidView::mesh`](crate::SolidView::mesh) for a placed
    /// solid only**, when the mesh encloses space where it was made and none,
    /// or less than none, where it lands — both signs settled exactly
    /// (§379.2). A solid at no placement is moved by a shift that loses
    /// nothing, so it cannot reach this.
    ///
    /// **The solid is sound.** It is the kernel's own `add_ball` in the case
    /// above, checked when it was built; what is refused is the mesh of it at
    /// that placement. A mesh that encloses nothing where it was made is not
    /// this: that is about the solid, and
    /// [`Model::validate`](crate::Model::validate) says so with
    /// [`Invalid::VolumeIsNotPositive`].
    #[error(
        "moving this solid into place took away the space its mesh encloses: \
         {enclosed_where_made} where it was made, {enclosed_where_placed} where it lands"
    )]
    PlacingItTookAwayTheSpaceItsMeshEncloses {
        /// The mesh's volume in the frame each face is stored in, where its
        /// points were worked out.
        enclosed_where_made: f64,
        /// The mesh's volume where the solid sits: zero or less.
        enclosed_where_placed: f64,
    },

    /// No mesh of this curved solid says which way its faces point.
    ///
    /// [`Model::validate`](crate::Model::validate) and the fuse read a curved
    /// solid's sign off what a mesh of it encloses, and a sum says it only once
    /// it is larger than all the space the mesh can leave out between its
    /// triangles and the solid's faces. A solid thinner than that at every mesh
    /// asked of it has no mesh that answers, and the sum the closest of them
    /// left was zero or below. A solid turned inside out leaves such a sum, and
    /// so can a sound sliver: two quarter bends of ring 4 and tube 1 placed
    /// round by `−1.984` and a quarter turn back overlap in a wedge `4π` times
    /// the angle between their caps, and their intersection is refused this way
    /// (`a_piece_no_ray_can_place_names_its_refusal.rs`, Q2089). Until Q2089
    /// that was [`Invalid::VolumeIsNotPositive`], "its faces do not point
    /// outwards", which nothing had shown. So was a fin `2⁻⁵¹` thick whose
    /// every mesh point rounds onto one plane, so that every mesh of it encloses
    /// exactly `0` (`an_ellipse_start_between_two_places.rs`, Q1938), and a
    /// half ball of 5 less a bowl four roundings smaller, which holds about
    /// `54`, and whose closest mesh encloses `−38.7` and can be off by `2384`
    /// (`a_piece_grazed_by_a_trimming_nothing_follows.rs`).
    ///
    /// A sum above zero that no mesh answered is still taken as the volume, so
    /// a solid turned inside out can still pass on one (Q2084).
    #[error(
        "no mesh of this solid says which way its faces point: the closest encloses {enclosed}, \
         and can be off by {leaves_out:?}"
    )]
    CannotTellWhichWayASolidsFacesPoint {
        /// What the mesh closest to the solid encloses: zero or less.
        enclosed: f64,
        /// How far that sum can be from the solid's volume: the space the mesh
        /// can leave out and the sum's own rounding. `None` when the solid has
        /// a face that space cannot be charged for yet, a cone's, and then
        /// every mesh asked of it lay flat.
        leaves_out: Option<f64>,
    },

    /// A closed edge of the answer would start at a **pole** of the surface it is
    /// drawn on, where nothing says which way it runs.
    ///
    /// A closed edge has one corner, so its two ends say nothing about its
    /// direction and only the surface's own parametrisation does: the way its
    /// longitude takes the points along it, which is
    /// `a_closed_crossing_is_walked_the_way_it_is_stored` and is what the mesher
    /// reads back. At a pole every longitude runs through the one point, so the
    /// vector that rule dots against is the zero vector exactly, and both faces
    /// along the edge come back walking it the same way, which is not a solid.
    /// This is about that exact case and says nothing about a corner a rounding
    /// *away* from a pole, where the vector is tiny and the sign of a dot with it
    /// is decided in `f64` the way it always was.
    ///
    /// A ball cut by a plane containing its **polar axis** reaches it, and only
    /// that: the great circle such a plane cuts runs through both poles, so the
    /// one corner the arrangement puts on that closed crossing is one of them.
    /// A plane through the ball's centre that does *not* contain the polar axis
    /// is answered — measured, at the equator and at a plane leaning 0.4 of a
    /// radian through the middle.
    ///
    /// **A gap and not a limit** (§16.10), and the direction rule is not what
    /// makes it one. Handed a direction the rule cannot derive, the same fuse
    /// gets two steps further and refuses
    /// [`Error::ACurvedFaceGoesRightRoundWithNoSeam`]: the piece is half a ball
    /// bounded by one curve right round its skin, with the poles on that curve
    /// rather than inside the piece, and no face of this kernel holds that shape.
    /// §38.3 records the run that measured it. Closing it takes the other half of
    /// what §38.1 built — a *cut* closed face written out with a seam and poles
    /// of its own.
    #[error(
        "a closed edge of the answer would start at a pole, where nothing says which way it runs"
    )]
    AClosedEdgeOfTheAnswerWouldStartAtAPole {
        /// Which face of the input the piece was cut from.
        face: crate::FaceKey,
    },

    /// A curved face is bounded by one closed curve — **or by nothing at all** —
    /// and there is no way to cut it open into a patch.
    ///
    /// Three shapes reach it and they are not one shape. All three have no seam
    /// and no pole in the model: a [`WireStep::Pole`](crate::model::WireStep)
    /// and a seam edge are what say those, and a fuse can invent neither from a
    /// single closed edge, because a seam wants two corners at one longitude and
    /// such a face has one corner in all.
    ///
    /// **Nothing bounding it, on a surface that never closes.** §38.1's: a face
    /// with no boundary once its seam is left out is the whole of its surface,
    /// and the fuse writes one back out by cutting it open along a seam of its
    /// own, which has to end somewhere. A ball pinches at two poles and is
    /// answered; a plane and a pipe run on for ever, so a face on one of them
    /// with nothing trimming it is unbounded. **A limit** — no answer exists.
    /// `Model` is `Deserialize` and `validate` is the untrusted import boundary
    /// (§16.11), so it is a refusal and not an assert.
    ///
    /// **One curve right round a surface that never closes**, which is half an
    /// endless tube round a cylinder — not a face at all, so the model's
    /// topology and its geometry disagree. **A limit** for the same reason.
    ///
    /// **One wire on a ball with the material outside it**, which is the whole
    /// ball less a disc: a ball with a cap taken off, or either half of a
    /// two-ball union. The mesher answers that now by opening the ball along a
    /// seam half a turn from the rim and cutting from the rim up to both poles
    /// (§202.1), whether the rim is one closed curve or several arcs that
    /// between them pass every point of the wire once (§402.3). What still
    /// reaches this name is a rim that cut cannot be shown to stay inside, and
    /// that is **a gap**. What tells the two sides of a curve apart is the sign
    /// of the area the wire encloses in the surface's own two coordinates,
    /// because a wire keeps its face's material on its left.
    #[error("a curved face is bounded by one curve right round it and has no seam to open at")]
    ACurvedFaceGoesRightRoundWithNoSeam {
        /// Which face.
        face: crate::FaceKey,
    },

    /// A curved face could not be cut into pieces narrow enough to mesh.
    ///
    /// Every cut that happens at all makes both pieces narrower, and four pieces
    /// is the answer for every face this kernel builds — the fuse's rings
    /// included, which are cut across into exactly four. Reaching the cap means
    /// a piece is being cut without getting any smaller.
    ///
    /// **This was an `assert!` until M4**, on the grounds that no builder makes
    /// such a face. `validate` walks it and [`Model`](crate::Model) is
    /// `Deserialize`, so what reaches it is a caller's bytes — and a panic is
    /// the wrong answer to those (decisions.md §4.1, §16.11). A fuse invents
    /// topology that was in no input, which puts its answers on the same side of
    /// that line.
    #[error("a curved face cannot be cut into pieces narrow enough to mesh")]
    ACurvedFaceCannotBeCutSmallEnoughToMesh {
        /// Which face.
        face: crate::FaceKey,
    },

    /// A curved face with a hole in it goes too far round its surface to be cut
    /// into triangles in one piece.
    ///
    /// A face that goes more than a quarter turn round is cut into pieces first,
    /// or every triangle in it would be stretched from one side of the seam to
    /// the other and have no area at all.
    ///
    /// **A ring — two curves that each go right round and nothing else** — is
    /// cut across into four patches with one loop each. That is the one the
    /// fuse makes on a pipe: a tube of a bar's wall between two planes.
    ///
    /// **A ball's skin with only holes** is opened into strips instead, one
    /// meridian for each stretch of longitude no hole reaches, and each strip
    /// cut up to both poles past its own run of holes (§402.3). What still
    /// reaches this name there is a strip whose holes neither lie one above
    /// the other nor clear each other's straight cuts, and that is **a gap**:
    /// the skin exists, and a cut that ends on another hole, which this cut
    /// family does not try, would reach it (§402.4).
    ///
    /// **A ball's cap cut up to its pole, with an island beside it**, no
    /// longer reaches this name when the island goes far enough round that
    /// two of the rim's cuts up to the pole land on it instead of missing it:
    /// since Q921 the strip between two such cuts is split into two patches
    /// instead (§212.6). The same shape without the ball under it — one wire
    /// right round beside one that is not — is refused the same way on a
    /// cone, where no fuse is known to build it; only arena surgery reaches it
    /// there (§212.6).
    ///
    /// **A refusal about the solid and not about the request**, and no deflection
    /// avoids it. §31.10 item 5 filed this doc as a claim about today's builders
    /// that M4 would falsify, and it did: the sentence that used to be here —
    /// *"nothing this kernel builds is such a face"* — stopped being true when
    /// `Error::FuseNeedsFlatFaces` came off. What is left arrives from a file,
    /// because [`Model`](crate::Model) is `Deserialize`, and a panic is the
    /// wrong answer to a caller's bytes (decisions.md §4.1).
    #[error("a curved face with a hole in it goes too far round its surface to be meshed")]
    CurvedFaceWithAHoleGoesTooFarRound {
        /// Which face.
        face: crate::FaceKey,
    },

    /// Three or more faces meet along this edge, so no pair of them says which
    /// two surfaces it runs between.
    ///
    /// Where an edge is is what the surfaces either side of it say (§23.2), and
    /// the evaluator that walks it is handed both. An edge used once has the same
    /// face on both sides, which is what a seam is; an edge used three times has
    /// no such pair and is not part of a closed solid at all.
    ///
    /// **A refusal and not an assert.** `check_edges` refuses an edge without
    /// two coedges, but [`SolidView::mesh`](crate::SolidView::mesh) is public in
    /// its own right and [`Model`](crate::Model) is `Deserialize`, so a solid
    /// that never went through the sweep reaches the mesher first (§16.11).
    #[error("this edge does not have two faces along it to say where it runs")]
    AnEdgeWithoutTwoFacesAlongItCannotBeWalked {
        /// Which edge.
        edge: crate::EdgeKey,
    },

    /// An edge bends, every face along it is flat, and nothing names the curve
    /// it bends along.
    ///
    /// A plane cannot say where a curve drawn on it goes, so an edge that is not
    /// [`EdgeRun::Straight`](rcad_geom::EdgeRun::Straight) is placed by a curved
    /// face beside it — and here there is none. The one other thing that can
    /// place it is a curved surface the run itself names, which is how a circle
    /// imprinted on a flat face is read from a file. An edge with neither is a
    /// curve whose centre and size are not in the model at all.
    ///
    /// **Meshing it as a chord is what this replaces**, and it was wrong by a
    /// whole radius: `docs/decisions.md` §176.2. An `ELLIPSE` imprinted on a flat
    /// face arrives here, and so does a whole circle, which a file names by its
    /// one corner and no surface.
    #[error("an edge bends but every face along it is flat and nothing names its curve")]
    AnEdgeBendsButEveryFaceAlongItIsFlat {
        /// Which edge.
        edge: crate::EdgeKey,
    },

    /// Walking this solid's boundaries at this deflection needs more points than
    /// one mesh will hold.
    ///
    /// Counted as the walk goes, so this is a refusal rather than a wait.
    /// `rcad-mesh` draws the same line one patch at a time, and this one is
    /// needed as well, because the boundaries are walked before any patch is
    /// handed over.
    #[error("this deflection needs more than {most} points for one solid")]
    MoreMeshPointsThanASolidWillHold {
        /// How many points one mesh will hold.
        most: usize,
    },

    /// A face was asked about against a solid that does not use it.
    ///
    /// What trims a face is the surfaces of the faces around it, and those are
    /// only reachable — already moved into place — through the solid the face
    /// sits in (§3.2). A key from somewhere else names no such thing here.
    #[error("this solid does not use that face")]
    NoSuchFaceInThisSolid {
        /// The key that named nothing.
        face: crate::FaceKey,
    },

    /// One edge of a face runs along a *piece* of a curve rather than the whole
    /// of one, and whether the face holds a point cannot be read off a sign.
    ///
    /// Which piece of a curve a point is on is a question about where that curve
    /// runs, and answering it means meeting three surfaces at once and then
    /// ordering points along the answer — §31.2's rows 4 and 8, and neither of
    /// them reaches a pair of quadrics (§31.2's limit under row 4).
    ///
    /// **A gap and not a limit** (§16.10): the point is on the piece or it is
    /// not, and an algebraic solver says which. `docs/decisions.md` §35 names
    /// the consumer.
    #[error("an edge of this face runs along part of a curve rather than the whole of one")]
    AFaceTrimmedByACurveThisCannotFollow {
        /// Which face.
        face: crate::FaceKey,
    },

    /// A fuse drew an edge of this face along where the face's plane crosses
    /// the plane of the face across the edge, and the edge runs along the
    /// curve its run names instead.
    ///
    /// A fuse draws each edge of a face where the face's surface crosses the
    /// surface of the face across it. Between two flat faces that is the
    /// straight line their planes cross in, and an edge between them whose run
    /// names a third surface, a circle round a pipe say, runs along that
    /// surface's crossing with the face's plane. Drawn on the line, a closed
    /// edge starts and ends at one node of a crossing that does not close, and
    /// an arc and its chord both cover the one stretch of the line between
    /// their two corners. A round bar whose lid is split by such a circle into
    /// an annulus and a disc leaning off the lid, or by half of it and its
    /// chord into a half disc leaning off the lid, is each of the two
    /// (`a_fuse_writes_no_piece_of_a_lid_off_the_lid_into_the_callers_model`).
    ///
    /// **A gap for a solid that passes the sweep** (§16.10): drawn along the
    /// named surface's crossing with the face's own plane, the edge is where
    /// the face puts it, and the fuse has its answer (Q2125). A fuse does not
    /// sweep what it is handed, so a solid read back from bytes whose two
    /// planes put the edge in two places reaches this too, and there no answer
    /// exists. Nothing here tells the two apart, so
    /// [`Error::limit_or_gap`] says not known.
    #[error("an edge of this face is drawn where two planes cross, not along the curve it names")]
    AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve {
        /// Which face.
        face: crate::FaceKey,
    },

    /// A fuse had to say whether a hollow of its answer is inside another
    /// shell, and no point of the hollow it asked could fire a ray at the
    /// other shell: the ray would have had to meet this face, which sits on a
    /// ring.
    ///
    /// Where a ray meets a ring is worked out only for a ray whose numbers are
    /// exact, and when they are not, an exact point beside the asked one is
    /// fired from instead — but only where no face of the shell comes near it,
    /// and a corner of a hollow that is on the shell's skin has a face right
    /// there ([`Containment::CannotFireARayAtAFacesSurface`](crate::Containment)).
    ///
    /// Deliberately not [`Error::Undecidable`], which says every predicate
    /// was asked and none could prove an answer: here the question was never
    /// put, so no policy makes it go away (`docs/decisions.md` §4.2, §35.6).
    /// What a caller can change is the shape: a corner of the hollow clear of
    /// the other shell's skin is placed by a point beside it (§323). `face` is
    /// the caller's face the ring face was cut from.
    #[error("no ray could be fired at this face's surface to place a hollow")]
    CannotFireARayAtAFacesSurface {
        /// Which face.
        face: crate::FaceKey,
    },

    /// A face runs along some but not all of the curves its surface and a
    /// neighbour's cross in.
    ///
    /// One of the two disks a pipe leaves in a ball's skin is the case. Its one
    /// boundary loop is a whole closed curve, but the pair of surfaces crosses in
    /// *two*, so "the piece of the ball inside the pipe" names both disks and not
    /// this one. Reading the face off a sign would hold the other disk's points
    /// as well.
    ///
    /// **A gap and not a limit**, and what closes it is a way to tell one loop
    /// from another — which is what a
    /// [`Picker`](rcad_geom::Picker) already does for an edge (§33.1).
    #[error("this face runs along some but not all of where its surfaces cross")]
    AFaceMissesPartOfWhereItsSurfacesCross {
        /// Which face.
        face: crate::FaceKey,
    },

    /// Nothing in the model says which side of a surface trimming it a face lies
    /// on.
    ///
    /// A corner of the face that is off that surface says; a face whose every
    /// corner sits on it has none, and a face with corners off it on *both*
    /// sides is not one of the two pieces at all.
    ///
    /// The other place the answer could come from is which way round the wire
    /// runs, and a closed edge has no direction rule to read it from — §30.8
    /// item 10, which §31.3 settles on paper and hands to the lane that builds
    /// the curved arrangement.
    #[error("nothing says which side of the surfaces trimming it this face lies on")]
    NothingSaysWhichSideOfItsTrimmingAFaceLiesOn {
        /// Which face.
        face: crate::FaceKey,
    },

    /// A face's boundary edge runs along a curve where its own surface and the
    /// one across that edge do not meet at all.
    ///
    /// Two faces that share an edge cross along it by definition, so this is a
    /// model whose topology and geometry disagree — a plane and a ball that miss
    /// each other with an edge drawn between them.
    #[error("a face has an edge along a curve its two surfaces do not have")]
    AFaceMeetsItsNeighbourNowhere {
        /// Which face.
        face: crate::FaceKey,
    },

    /// A face's boundary edge runs between two faces that are on **one**
    /// surface.
    ///
    /// Two faces on one surface with an edge between them carry no curve for
    /// that edge to run along, so they are one face and the edge should not be
    /// there. The fuse's joiner removes such edges: over every answer a fuse
    /// gives, and over a wholly flat input before it is cut. It joins pieces of
    /// one plane, cone or pipe (§224.2) and never pieces of a ball or a ring.
    /// So this is raised for pieces of a cone or pipe the joiner leaves apart —
    /// in a curved input, which it is never asked about, or where it could not
    /// prove the two are one surface used the same way round — and for pieces
    /// of a ball or a ring, which it never joins. Refused rather than arranged
    /// wrongly.
    #[error("a face and the face across one of its edges are on the same surface")]
    AFaceMeetsItsNeighbourOnItsOwnSurface {
        /// Which face.
        face: crate::FaceKey,
    },

    /// Two surfaces cross in more than one curve and nothing says which of them
    /// an edge of a face runs along.
    ///
    /// A plane down the inside of a pipe cuts two straight lines and a ball
    /// crossing a pipe off-axis leaves two loops; which one an edge is, is the
    /// half-spaces on its own run. A picker that could not decide rules out
    /// nothing (§33.8), so a corner held by both curves — or by neither — is a
    /// refusal here and never a guess.
    #[error("nothing says which of the curves two surfaces cross in an edge of a face runs along")]
    CannotTellWhichCurveOfACrossingAFaceRunsAlong {
        /// Which face.
        face: crate::FaceKey,
    },

    /// Two curves drawn on one face meet at a point without crossing there, or
    /// run along each other from it.
    ///
    /// Two curves tangent at a node leave the arrangement with no anticlockwise
    /// order round it, so there is no way round the cell they bound. A **gap**:
    /// the pieces are there and telling them apart takes following one of the
    /// curves past the touch, which nothing in this kernel does.
    ///
    /// **Coincidence is the far end of the same thing** (§38.2). A `3 × 4` block
    /// corner on a bar of radius five puts the block's two walls on the one
    /// ruling `(3, 4, z)` of the bar's, so the two curves do not merely touch —
    /// they are one line held by the arrangement as two crossings, and no order
    /// round the node exists to be found.
    #[error("two curves on one face touch without crossing")]
    TwoCurvesTouchWithoutCrossing {
        /// Which face.
        face: crate::FaceKey,
    },

    /// Two walks on one face could not be told apart into the cells they bound.
    ///
    /// The walks the arrangement leaves have to be grouped into the regions of
    /// the surface they bound, and two of them in different lumps of the drawing
    /// are the same region exactly when they lie on the same side of every curve
    /// that is drawn in full. A curve that runs on for ever is never drawn in
    /// full, so a face bounded by straight lines has to be placed some other way
    /// — and where neither route settles it, two regions come out joined that
    /// the surface keeps apart, and the winding count says so.
    ///
    /// A **gap**: the regions are there and telling them apart takes a point
    /// inside each one, which is a construction this arrangement does not make.
    /// It is deliberately not a broken invariant, because a face that really did
    /// overlap itself would look the same from here and only one of the two is a
    /// bug in this crate.
    #[error("two regions of one face could not be told apart")]
    CannotTellTheCellsOfAFaceApart {
        /// Which face.
        face: crate::FaceKey,
    },

    /// A fuse joined pieces of one plane into one face, and nothing could
    /// place its rings: which one is the outline and which are holes.
    ///
    /// A ring's corners usually say, by the way they wind round the plane's
    /// normal. Corners that sweep no area say nothing, and such a ring is
    /// placed by the region it bounds instead — but that region is read only
    /// for a ring along one pipe, ball, cone or ring, or one whose corners lie
    /// on one line. Two or more rings of one face read neither way are this.
    /// The cap along four pipes in `a_joined_flat_face_whose_rings_enclose_no_area.rs`,
    /// whose corners sweep exactly nothing, with its own shape an eighth the
    /// size standing through it, is refused this in the union and in the big
    /// cap less the small one (Q1253).
    ///
    /// **A gap and not a limit** (§16.10): the face is there, and a point
    /// inside the region each ring bounds would place it. `face` is one of the
    /// caller's faces the joined face was cut from.
    #[error("nothing could say which ring of a face joined from flat pieces is its outline")]
    CannotPlaceTheRingsOfAJoinedFlatFace {
        /// One of the faces it was cut from.
        face: crate::FaceKey,
    },

    /// Points along a curve a face is cut by cannot be put in order.
    ///
    /// A straight crossing is ordered by the axis it climbs fastest along and a
    /// closed conic by the area three of its points sweep. A ball against a
    /// pipe, or two pipes, is neither: the curve is a quartic, and ordering
    /// points along one takes the marcher rather than a determinant. A **gap**,
    /// and §34.10's own row records the same one dimension down.
    #[error("nothing can put points in order along this curve")]
    ACrossingThisCannotBeOrderedAlong,

    /// An edge of a fused solid runs along *part* of a curved crossing, and
    /// nothing can say which part.
    ///
    /// A crossing of two surfaces that closes and is cut at two corners leaves
    /// two arcs with the same two ends, so the ends do not name one of them and
    /// [`EdgeRun::PartOfWhereItCrosses`](crate::model::EdgeRun) wants the
    /// half-space that does.
    ///
    /// **Which arc is no longer what this refuses** (`docs/decisions.md` §36.23).
    /// The half-space through the chord square to the crossing's own plane tells
    /// them apart, its normal is `v(c·c) − c(c·v)` for `v` a direction into the
    /// arc, and its offset is proved down until the predicate says both ends are
    /// on or in front. What is left is the one case where that normal has no
    /// length to point along: a chord that lies along the crossing's own
    /// direction, which for a conic means an arc so short the chord rounds onto
    /// its own tangent. A **gap** — the arc is there and the half-space exists;
    /// what cannot reach it is `f64` at that separation.
    ///
    /// `an_arc_whose_chord_rounds_onto_its_own_tangent_is_refused_by_name`.
    #[error("an edge of the answer runs along part of a curved crossing and cannot be named")]
    ACurvedEdgeThatIsPartOfACrossingCannotBeNamed,

    /// A handle made by one [`SolidBuilder`](crate::SolidBuilder) was used on
    /// another.
    ///
    /// A handle is an index into the builder that made it, so the same index in
    /// a second builder names a different entity and resolves silently to the
    /// wrong one. Refused rather than asserted against, because a builder takes
    /// caller input from the first call to the last (`docs/decisions.md` §42.5).
    #[error("this {entity} handle was made by a different builder")]
    ADraftHandleCameFromAnotherBuilder {
        /// Which kind of handle came from elsewhere.
        entity: &'static str,
    },

    /// A point handed to a builder is infinite or is not a number.
    #[error("the point described at position {vertex} is not a finite point")]
    ADraftPointIsNotFinite {
        /// Which vertex of the description, counting from zero.
        vertex: usize,
    },

    /// An edge was described between one vertex and itself, or between two
    /// vertices at the same point, and its run does not say it comes back there.
    ///
    /// A closed edge is not this: it comes back to where it started because it
    /// runs right round a whole curve, which is what its
    /// [`EdgeRun`](rcad_geom::EdgeRun) says. **Straight is not the whole of what
    /// is refused here**, and the message used to say it was: the two runs that
    /// name a curve are the two that may start and finish at one corner, and a
    /// kind of run this kernel has never seen is not taken for one of them
    /// (`SolidBuilder`'s `the_ends_are_the_whole_of_it`).
    #[error(
        "the edge described at position {edge} has both ends in one place, and \
         its run does not say it comes back there"
    )]
    ADraftEdgeHasNoLength {
        /// Which edge of the description, counting from zero.
        edge: usize,
    },

    /// A wire was described with too few steps to close round anything.
    ///
    /// Three is the floor, and there are two exceptions. One step is enough
    /// when that step runs right round a closed curve — a round cap's boundary,
    /// which is one whole circle. Two are enough when they are two *different*
    /// edges and at least one of them bends: an arc and the chord that closes
    /// it bound a real face. Two steps on one edge run out and back along the
    /// same curve, and two straight edges between the same two corners are the
    /// same segment written twice; both sweep no area at all.
    #[error(
        "a wire of {steps} steps closes round nothing: it needs three, or one edge that closes \
         already, or two different edges that are not both straight"
    )]
    ADraftWireIsTooShort {
        /// How many steps the wire was given.
        steps: usize,
    },

    /// A wire was described whose steps do not join up end to end.
    #[error("a wire does not close: step {position} does not start where the one before it ended")]
    ADraftWireDoesNotClose {
        /// Which step breaks the chain. The count itself means the last step
        /// does not come back to where the first one set off.
        position: usize,
    },

    /// A face was described with no wires, so nothing says what trims it.
    #[error("a face was described with no wires to trim it")]
    ADraftFaceHasNoWires,

    /// A shell was described with no faces, so it is not a skin at all.
    #[error("a shell was described with no faces")]
    ADraftShellHasNoFaces,

    /// Something in the description is used where the data model allows it only
    /// once: a coedge by two wires, a wire by two faces, or a face by two
    /// shells.
    #[error("the {entity} described at position {index} is used more than once")]
    ADraftEntityIsUsedTwice {
        /// Which kind of entity is over-used.
        entity: &'static str,
        /// Which one, counting from zero.
        index: usize,
    },

    /// Something in the description is not used by anything above it.
    ///
    /// A vertex no edge and no pole names, a coedge no wire walks, a wire no
    /// face is trimmed by, or a face the shell does not hold. A solid is
    /// exactly what its shell reaches, so anything else is a description of
    /// something that is not there.
    #[error("the {entity} described at position {index} is not used by anything")]
    ADraftEntityIsUsedByNothing {
        /// Which kind of entity is unused.
        entity: &'static str,
        /// Which one, counting from zero.
        index: usize,
    },

    /// An edge of the description is not walked by exactly two coedges, so the
    /// skin does not close along it.
    ///
    /// This is what an unclosed shell looks like from inside a builder, and it
    /// names the edge the skin is open along rather than the shell as a whole.
    #[error(
        "the edge described at position {edge} is walked by {count} coedges; a closed skin walks every edge exactly twice"
    )]
    ADraftEdgeIsNotWalkedByTwoCoedges {
        /// Which edge of the description, counting from zero.
        edge: usize,
        /// How many coedges name it.
        count: usize,
    },

    /// Both coedges of an edge run the same way, so the two faces meeting there
    /// do not agree which side the material is on.
    #[error("both coedges of the edge described at position {edge} run the same way round")]
    ADraftEdgeIsWalkedTheSameWayTwice {
        /// Which edge of the description, counting from zero.
        edge: usize,
    },

    /// The sweep could not walk along the crossing one edge of the description
    /// runs on.
    ///
    /// [`Error::ACrossingAlongThisEdgeCouldNotBeFollowed`] said the same thing
    /// about a key in the throwaway model
    /// [`SolidBuilder::build`](crate::SolidBuilder::build) swept, and that model
    /// is dropped the moment the build refuses. So the key is turned back into
    /// the position the caller handed the edge over at, which is the one address
    /// a caller can still look up — `rcad-step` keeps the `#id` it read each
    /// edge from in exactly that order.
    ///
    /// **The message is the geometry's own and does not name the position.**
    /// `probe` deduplicates a file's reasons by their text, so a position in the
    /// text would split one wall into as many reasons as there are edges that
    /// hit it, and a caller holding an index into its own description does not
    /// learn anything from being read it back.
    #[error("geometry: {why}")]
    ADraftEdgeHasACrossingThatCannotBeFollowed {
        /// Which edge of the description, counting from zero.
        edge: usize,
        /// What the marcher gave up on, which carries the two surfaces and
        /// which of its questions it gave up at.
        why: Box<rcad_geom::Error>,
    },

    /// The sweep could not walk along the crossing one edge runs on.
    ///
    /// This is [`Error::Geometry`] with the edge attached. The marcher's own
    /// refusal carries the two surfaces it gave up between, and coordinates are
    /// not an address: a caller that wants to know which edge of *its* model
    /// that was could only match those surfaces back by value, which
    /// `docs/decisions.md` §110.4 measured at zero matches out of forty-four
    /// files. The key is the address, and the sweep has it in hand.
    #[error("geometry: {why}")]
    ACrossingAlongThisEdgeCouldNotBeFollowed {
        /// The edge whose crossing could not be followed.
        edge: EdgeKey,
        /// What the marcher gave up on.
        why: Box<rcad_geom::Error>,
    },

    /// The sweep could not cut one face of the description into triangles.
    ///
    /// [`Error::CannotMeshAFace`] said the same thing about a key in the
    /// throwaway model [`SolidBuilder::build`](crate::SolidBuilder::build)
    /// swept, and that model is dropped the moment the build refuses. So the
    /// key is turned back into the position the caller handed the face over
    /// at, which is the one address a caller can still look up — `rcad-step`
    /// keeps the `#id` it read each face from in exactly that order.
    /// `docs/decisions.md` §123.5, and §113.4 for the edge this is the same
    /// move for.
    ///
    /// **The message is the triangulator's own and does not name the
    /// position.** `probe` deduplicates a file's reasons by their text, so a
    /// position in the text would split one wall into as many reasons as
    /// there are faces that hit it, and a caller holding an index into its
    /// own description does not learn anything from being read it back.
    #[error("triangulating: {why}")]
    ADraftFaceCouldNotBeMeshed {
        /// Which face of the description, counting from zero.
        face: usize,
        /// What the triangulator gave up on.
        why: rcad_mesh::CannotTriangulate,
    },

    /// A second shell was described, and a solid built face by face has one.
    ///
    /// A void inside a solid, and a solid in two separate lumps, are both
    /// `shells` of length two in the data model and neither is built, meshed or
    /// measured anywhere in this kernel. Refused by name rather than written
    /// and hoped for; `docs/decisions.md` §13 rule 3.
    #[error("a solid built face by face has exactly one shell")]
    ASolidBuiltFaceByFaceHasOneShell,

    /// A solid was asked for before any shell was described.
    #[error("no shell was described, so there is no solid to build")]
    ADraftSolidHasNoShell,

    /// Two solids of one model fill some of the same space.
    ///
    /// Comes back from [`Model::check_no_two_solids_share_space`](crate::Model::check_no_two_solids_share_space),
    /// which is a pass of its own and not part of
    /// [`Model::validate`](crate::Model::validate): the sweep looks at one solid
    /// at a time and this takes a pair.
    #[error("two solids of this model fill some of the same space")]
    TwoSolidsShareSpace {
        /// One of the two.
        one: crate::SolidKey,
        /// The other.
        other: crate::SolidKey,
    },

    /// Whether two solids fill any of the same space could not be settled.
    ///
    /// Its own error rather than [`Error::Undecidable`] because it carries the
    /// pair and the width that straddled instead of a log of records: the search
    /// asks [`SolidView::classify_point`](crate::SolidView::classify_point),
    /// which puts its own records in the caller's log and hands back a width.
    #[error(
        "could not tell whether two solids share space; the widest enclosure that straddled was {enclosure}"
    )]
    CannotTellWhetherTwoSolidsShareSpace {
        /// One of the two.
        one: crate::SolidKey,
        /// The other.
        other: crate::SolidKey,
        /// How wide the widest enclosure that straddled zero was.
        enclosure: f64,
    },

    /// Which way a closed edge runs round its two surfaces could not be settled.
    ///
    /// The rule is `which_way_a_closed_crossing_turns`, and it answers with a
    /// [`Decision`](rcad_predicates::Decision) like any predicate. Three of its
    /// four answers are usable: a sign either way says which way the edge runs,
    /// and [`Decision::Zero`](rcad_predicates::Decision::Zero) says the two
    /// surfaces touch rather than cross there and the edge has no direction to
    /// name. The fourth is this.
    ///
    /// Its own error rather than [`Error::Undecidable`] because there is no log
    /// to carry: the rule takes no [`Policy`](rcad_predicates::Policy) and keeps
    /// no records — it is three cross products and a dot, and the width that
    /// straddled zero is the whole of what there is to say. It is what a
    /// hemisphere sitting on a pipe of the same radius and very nearly the same
    /// axis comes back with: the wire round that crossing is shorter than the
    /// rounding in the cross product that built it (`docs/decisions.md` §111).
    #[error(
        "could not tell which way a closed edge runs round its surfaces; the enclosure that straddled was {enclosure} wide"
    )]
    CannotTellWhichWayAClosedEdgeRuns {
        /// How wide the enclosure that straddled zero was.
        enclosure: f64,
    },

    /// The operation ran out of things it could prove.
    ///
    /// Carries the records of every decision that could not be settled, so the
    /// caller can see which predicate gave up and how wide its enclosure was.
    #[error("could not settle {} decisions the answer depends on", records.len())]
    Undecidable {
        /// The decisions that could not be settled.
        records: ProvenanceLog,
    },
}

impl Error {
    /// Whether the answer this refusal stands in for exists (§16.10).
    ///
    /// `docs/decisions.md` §437.6. A **limit** says no answer this kernel can
    /// hold exists, a **gap** that one exists and the arithmetic did not reach
    /// it, and **not known** that nobody has shown either. Every limit and gap
    /// claimed here is reached and asserted by
    /// `what_a_fuse_refusal_says_about_its_answer.rs`.
    ///
    /// A variant that can be either kind, and carries nothing that tells them
    /// apart, says not known. [`Error::VolumeTooLargeToMeasure`] is a limit
    /// from [`SolidView::measure`](crate::SolidView::measure) of a cube of side
    /// `1e103`, and a gap from a fuse uniting a cube of side `2^342` with its
    /// copy, which the model already holds the answer to.
    /// [`Error::LengthTooLargeToMeasure`] is a limit for a wire `4e308` round,
    /// and a gap for one exactly `f64::MAX` round whose sum, rounded outwards,
    /// runs past it. [`Error::SurfaceAreaTooLargeToMeasure`] is a limit for a
    /// face of `1e400`, and a gap for a parallelogram just under `f64::MAX`
    /// whose fan, summed with rounding, comes out past it.
    ///
    /// [`Error::Geometry`] and the two refusals that carry a `why` say what the
    /// refusal of geometry they carry says, and [`Error::Invalid`] what
    /// [`Invalid::limit_or_gap`] says.
    ///
    /// The solid builder's refusals are limits of the draft as it was handed
    /// over: a wire of two steps, or a skin open along an edge, describes no
    /// solid. An importer whose own reading of a file made the draft has its
    /// own refusal to class, because a file has reached several of them for a
    /// shape that exists (§43.6, §218.4, §250.7).
    ///
    /// One match and no wildcard, so a new variant does not compile until it is
    /// given a class.
    #[must_use]
    pub fn limit_or_gap(&self) -> rcad_geom::LimitOrGap {
        use rcad_geom::LimitOrGap;
        match self {
            Self::Geometry(why) => why.limit_or_gap(),
            Self::ADraftEdgeHasACrossingThatCannotBeFollowed { why, .. }
            | Self::ACrossingAlongThisEdgeCouldNotBeFollowed { why, .. } => why.limit_or_gap(),
            Self::Invalid(invalid) => invalid.limit_or_gap(),
            Self::KeyFromAnotherModel { .. }
            | Self::TooFewPoints { .. }
            | Self::PointNotFinite { .. }
            | Self::RepeatedPoint { .. }
            | Self::HeightNotPositive { .. }
            | Self::ACapOfABarFacesIntoIt { .. }
            | Self::NoSuchFaceInThisSolid { .. }
            | Self::ADraftHandleCameFromAnotherBuilder { .. }
            | Self::ADraftPointIsNotFinite { .. }
            | Self::ADraftEdgeHasNoLength { .. }
            | Self::ADraftWireIsTooShort { .. }
            | Self::ADraftWireDoesNotClose { .. }
            | Self::ADraftFaceHasNoWires
            | Self::ADraftShellHasNoFaces
            | Self::ADraftEntityIsUsedTwice { .. }
            | Self::ADraftEntityIsUsedByNothing { .. }
            | Self::ADraftEdgeIsNotWalkedByTwoCoedges { .. }
            | Self::ADraftEdgeIsWalkedTheSameWayTwice { .. }
            | Self::ADraftSolidHasNoShell => LimitOrGap::Limit,
            Self::CannotYetMeasureHowLongThisCurveIs { .. } => LimitOrGap::Gap,
            Self::ProfileCrossesItself
            | Self::HoleNotInsideOutline { .. }
            | Self::HoleInsideAnotherHole { .. }
            | Self::ACapDoesNotCutRightRoundABar { .. }
            | Self::TheCapsOfABarMeetInsideIt
            | Self::VolumeTooLargeToMeasure
            | Self::SurfaceAreaTooLargeToMeasure
            | Self::LengthTooLargeToMeasure
            | Self::TheFuseWroteABrokenAnswer { .. }
            | Self::WrongFormatVersion { .. }
            | Self::ArgumentIntersectsItself
            | Self::ResultIsNotManifold { .. }
            | Self::CannotMeshAFace { .. }
            | Self::AWireDoesNotFanIntoALoopToMesh { .. }
            | Self::DeflectionBelowWhatAMeshCanReach { .. }
            | Self::AWireWillNotLieFlatOnItsFace { .. }
            | Self::TwoPatchesOfTheMeshDrewTheSameSide { .. }
            | Self::PlacingItTookAwayTheSpaceItsMeshEncloses { .. }
            | Self::CannotTellWhichWayASolidsFacesPoint { .. }
            | Self::AClosedEdgeOfTheAnswerWouldStartAtAPole { .. }
            | Self::ACurvedFaceGoesRightRoundWithNoSeam { .. }
            | Self::ACurvedFaceCannotBeCutSmallEnoughToMesh { .. }
            | Self::CurvedFaceWithAHoleGoesTooFarRound { .. }
            | Self::AnEdgeWithoutTwoFacesAlongItCannotBeWalked { .. }
            | Self::AnEdgeBendsButEveryFaceAlongItIsFlat { .. }
            | Self::MoreMeshPointsThanASolidWillHold { .. }
            | Self::AFaceTrimmedByACurveThisCannotFollow { .. }
            | Self::AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve { .. }
            | Self::CannotFireARayAtAFacesSurface { .. }
            | Self::AFaceMissesPartOfWhereItsSurfacesCross { .. }
            | Self::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn { .. }
            | Self::AFaceMeetsItsNeighbourNowhere { .. }
            | Self::AFaceMeetsItsNeighbourOnItsOwnSurface { .. }
            | Self::CannotTellWhichCurveOfACrossingAFaceRunsAlong { .. }
            | Self::TwoCurvesTouchWithoutCrossing { .. }
            | Self::CannotTellTheCellsOfAFaceApart { .. }
            | Self::CannotPlaceTheRingsOfAJoinedFlatFace { .. }
            | Self::ACrossingThisCannotBeOrderedAlong
            | Self::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed
            | Self::ADraftFaceCouldNotBeMeshed { .. }
            | Self::ASolidBuiltFaceByFaceHasOneShell
            | Self::TwoSolidsShareSpace { .. }
            | Self::CannotTellWhetherTwoSolidsShareSpace { .. }
            | Self::CannotTellWhichWayAClosedEdgeRuns { .. }
            | Self::Undecidable { .. } => LimitOrGap::NotKnown,
        }
    }
}

/// An undecided *operation* down in `rcad-geom` stays undecided up here rather
/// than being buried inside [`Error::Geometry`].
///
/// Naming where two surfaces cross is an operation and can give up (§24.10), and
/// what it gives up with is `rcad_geom::Error::Undecidable`. Wrapped, a caller
/// would have to unwrap two layers to learn that what happened was "could not
/// decide" rather than "bad input" — and §17.7's breakdown of what gave up would
/// file it under the wrong tier, which is the one thing that breakdown is for.
/// [`rcad::Error`](https://docs.rs/rcad) does the same for this crate's own.
impl From<rcad_geom::Error> for Error {
    fn from(error: rcad_geom::Error) -> Self {
        match error {
            rcad_geom::Error::Undecidable { records } => Error::Undecidable { records },
            other => Error::Geometry(other),
        }
    }
}

/// The same conversion for the two walls that are about following a crossing,
/// which keep the edge they were following.
///
/// **Only those two.** `walk` refuses for other reasons too — it could not
/// place a point as close to the crossing as it was asked to, it could not name
/// which of two arcs this is, it could not decide — and none of those is a
/// crossing that could not be followed. Wrapping them under a name that says it
/// was would make the name a lie and cost every caller that reads one of them
/// an extra layer for nothing; they go on through [`Error::Geometry`]
/// unchanged. A new refusal is not added to this list by default, which is the
/// point of spelling the two out.
///
/// [`rcad_geom::walk`] names the two surfaces it gave up between and cannot
/// name the edge: it was handed two surfaces and a run and never saw a model.
/// Matching those surfaces back to an edge by value is what `docs/decisions.md`
/// §110.4 tried. The marcher does still work in a frame of the mesher's own, but
/// the pair no longer *leaves* the mesher in it: §116 moves it back, so what a
/// caller reads is in the model's frame. A match by value is therefore exact
/// where the face's placement is a shift, and strays about a part in `1e13`
/// where it is a turn (§116.4). The edge is carried rather than recovered
/// because carrying it is exact either way.
///
/// An undecided *operation* still comes through as [`Error::Undecidable`]
/// rather than being buried a layer deeper, for §17.7's reason: what gave up
/// belongs in the undecided tier whoever was walking.
pub(crate) fn walking_along_this_edge_refused(edge: EdgeKey, why: rcad_geom::Error) -> Error {
    if !matches!(
        why,
        rcad_geom::Error::CannotFollowACrossingWhereItsSurfacesAreTangent { .. }
            | rcad_geom::Error::CannotFollowACrossingBecauseNoStepStaysOnIt { .. }
    ) {
        return why.into();
    }
    Error::ACrossingAlongThisEdgeCouldNotBeFollowed {
        edge,
        why: Box::new(why),
    }
}

/// An invariant a model must hold and does not.
///
/// These come back from [`Model::validate`](crate::Model::validate), which is
/// the sweep run at test and import boundaries. The same list is checked on
/// touched entities after every mutation, where breaking one is a bug in this
/// crate and panics instead (decisions.md §4.3).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum Invalid {
    /// A stored key points at a slot that is not there any more.
    #[error("a {entity} is referred to but is not in the model")]
    DanglingKey {
        /// Which kind of entity is missing.
        entity: &'static str,
    },

    /// A coordinate is infinite or not a number.
    #[error("a vertex is not at a finite point")]
    VertexNotFinite,

    /// An edge starts and ends in the same place, and its run does not say it
    /// comes back there.
    ///
    /// A closed edge is not this: it comes back to where it started because it
    /// is a whole circle, which is what its
    /// [`EdgeRun`](crate::model::EdgeRun) says. The sweep reads the same list
    /// the builder's door does, so a run neither of them knows is refused here
    /// too rather than read as *not straight*.
    #[error("an edge starts and ends at the same point")]
    EdgeHasNoLength,

    /// A corner of a curved face is certified to be off the surface that face
    /// sits on.
    ///
    /// There is no distance in this one, and that is the point. A flat face
    /// reports how far off its plane a corner sits, because a plane has a
    /// signed distance to be near. A curved face asks a sign predicate, which
    /// says which side and never how far.
    #[error("a corner of a curved face is not on the surface that face sits on")]
    VertexOffItsFaceSurface,

    /// A wire stands still at a corner — a pole step — and that corner is not
    /// where the face's surface pinches to a point.
    ///
    /// A ball pinches at its two poles, straight above and below its centre,
    /// and a cone at its apex. A plane, a pipe and a ring pinch nowhere, so a
    /// pole step on one of them is always this. The mesher trusts a pole step
    /// and squashes a whole side of the face into that one corner, so a pole
    /// that is somewhere else draws a mesh folded over itself
    /// (`docs/decisions.md` §188.2).
    ///
    /// Refused only when the corner is *provably* away from every pinch, after
    /// the same room [`Invalid::VertexOffItsFaceSurface`] gives a corner: the
    /// slop recorded on it, or what writing a point down in `f64` costs,
    /// whichever is wider.
    #[error(
        "a wire stands still at a pole at {at:?}, but its face's surface pinches to a point at {pinch:?}"
    )]
    APoleStandsWhereTheSurfaceDoesNotPinch {
        /// Where the pole step's corner stands, in the face's stored frame.
        at: rcad_geom::Point,
        /// The pinch point nearest it, or `None` when the face's surface
        /// pinches nowhere.
        pinch: Option<rcad_geom::Point>,
    },

    /// An edge named as part of a crossing sits outside a half-space that was
    /// meant to pick which part.
    ///
    /// A crossing of two surfaces can have more than one piece — a plane cuts a
    /// sphere in a circle, and two points on a circle leave two arcs — so an
    /// edge names the half-spaces it lies in. If an end of the edge is outside
    /// one of them, the name picks a piece the edge is not.
    #[error("an end of an edge is outside a half-space that was meant to pick it out")]
    EdgeOutsideWhatPickedIt,

    /// An edge is not used by exactly two coedges.
    #[error("an edge is used by {count} coedges; a closed solid uses every edge exactly twice")]
    EdgeCoedgeCount {
        /// How many coedges name this edge.
        count: usize,
    },

    /// Both coedges of an edge run the same way, so the two faces meeting there
    /// do not agree about which side the material is on.
    #[error("both coedges of an edge run the same way round")]
    CoedgesRunTheSameWay,

    /// An edge is walked once from each of two shells, so neither shell
    /// closes on its own.
    ///
    /// A shell is a closed skin, and along every edge of a closed skin both
    /// faces meeting there are in it. Counted over the whole model, an edge
    /// along a cut through a skin still has its two coedges running opposite
    /// ways, one from each side of the cut. So a cube's six faces shared out
    /// between two solids passed every other check at the door, and each half
    /// was then measured, meshed and asked which side a point is on as if it
    /// were closed. A fuse of a round bar's half panicked looking for the face
    /// across the cut among the faces of its own half (`docs/decisions.md`
    /// §342).
    #[error("an edge is walked once from each of two shells; a shell must close on its own")]
    AnEdgeIsWalkedOnceFromEachOfTwoShells,

    /// A face that sits on a plane is used reversed.
    ///
    /// A plane welds "which surface" to "which way", so a plane that faces the
    /// other way is stored turned round and is then used forwards; only a curved
    /// surface, which has no turned-round form, is ever used reversed
    /// (`docs/decisions.md` §31.3, §36.24). Both the fuse and the STEP reader
    /// write that table; until this refusal existed, nothing read it back.
    ///
    /// What a reversed flat face is, is a wall of the solid facing inwards, and
    /// nothing else in the sweep can see one. Reversing a face turns its normal
    /// *and* walks its wire the other way, so the face's own winding is
    /// unchanged and every coedge in the model is exactly where it was; the
    /// volume stays positive, because flipping one wall of a `10³` box leaves
    /// the divergence sum at `333.3`. The skin is torn all the same.
    #[error(
        "a face on a plane is used reversed; a plane that faces the other way is stored \
             turned round instead"
    )]
    AFlatFaceIsUsedReversed,

    /// A shell places one of its faces somewhere other than where that face is
    /// stored.
    ///
    /// **Placement lives on the shell**, which is where every builder in the
    /// kernel writes it and where [`Model::place_solid`](crate::Model::place_solid)
    /// composes onto (`docs/decisions.md` §44.8). Every site that builds a face
    /// use writes an identity, so nothing but a file can carry one — and a
    /// face is used by exactly one shell, so there is no sharing for a per-use
    /// placement to serve either (§42.7 makes that argument about shells and
    /// `Invalid::UsedMoreThanOnce` makes it about faces).
    ///
    /// What one does instead is tear the skin without moving a single corner. A
    /// face's surface is read at the placement the face use carries and its
    /// corners are read at the same one, so the face stays self-consistent and
    /// every check that looks at one face passes; what changes is that the two
    /// faces along an edge now put that edge in two different places. The mesh
    /// cannot see it either, because the mesher names a corner by its vertex and
    /// both faces name the same vertex — a wall moved a thousand units away and
    /// a wall nudged a quarter of one both used to validate.
    #[error(
        "a shell places one of its faces away from where that face is stored; a placement \
             belongs on the shell"
    )]
    AFaceUseCarriesAPlacement,

    /// A closed edge — one whose [`EdgeRun`](crate::model::EdgeRun) says it
    /// goes right round where its two faces cross — sits between two flat
    /// faces.
    ///
    /// Two planes cross in a straight line, which never closes, so a name that
    /// claims a closed curve where two planes meet describes a curve that is
    /// not there. `Model` derives `Deserialize`, so nothing between a caller's
    /// bytes and this check has agreed the edge's own claim is even possible.
    #[error(
        "a closed edge is walked between two flat faces, and two planes cross in a straight \
         line, which never closes"
    )]
    ClosedEdgeBetweenTwoFlatFaces,

    /// A closed edge sits between two flat faces on two different planes, and
    /// its run names a curved surface to say where it goes.
    ///
    /// The curve the run names is where that surface crosses a face's plane —
    /// a circle or an ellipse — and it lies in that one plane and leaves every
    /// other. So the two faces put the edge in two different places, and which
    /// one the kernel drew depended on which face the solid listed first: a
    /// bar of radius 3 whose lid is split by a disc tilted through the circle's
    /// corner measured `113.003` one way round and `111.644` the other, each
    /// `± 0.376`, where the bar is `36π` (Q386, Q574).
    ///
    /// Two faces on one plane are not this, however that plane is spelt: turned
    /// round, or written twice a hair apart, which is what a file writes when
    /// it gives each face its own `PLANE` (`docs/decisions.md` §22.6, §219.6).
    #[error(
        "a closed edge is walked between two flat faces on different planes, and a closed curve \
         drawn on one plane leaves every other"
    )]
    ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes,

    /// An open arc sits between two flat faces on two different planes, and
    /// its run names a curved surface to say where it goes.
    ///
    /// [`Invalid::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes`] for a piece
    /// of the curve rather than the whole of it. Both ends of the arc are on
    /// both planes, so on the line where the two planes cross, but the arc
    /// between them is drawn on one plane and bends off every other. So the
    /// two faces put the edge in two different places, and which one the
    /// kernel drew depended on which face the solid listed first: a bar of
    /// radius 3 whose lid is split by a half disc, tilted along its chord,
    /// measured `113.088` with the lid listed first and `113.488` with the half
    /// disc first, each `± 0.038`, where the bar is `36π` (Q604).
    ///
    /// Since a flat face is charged for its points off its plane (Q2088), the
    /// two come with `± 0.196` and `± 2.71` instead, and overlap. They must:
    /// whichever face is drawn second has its triangles off the plane it
    /// names, by `0.066` of space for the half disc and `0.333` for the lid,
    /// and the two meshes differ by those two slabs, `0.400` together. The
    /// volume still depends on which face is listed first; the measure now
    /// owns up to it.
    ///
    /// Two faces on one plane are not this, however that plane is spelt; nor
    /// are two faces that cross in one of a pipe's own straight lines, where
    /// the arc the run names is that line.
    #[error(
        "an arc is walked between two flat faces on different planes, and a curve drawn on one \
         plane bends off every other"
    )]
    AnArcBetweenTwoFlatFacesOnDifferentPlanes,

    /// An edge bends, every face along it is flat, and its run names no curved
    /// surface that would say where it bends.
    ///
    /// The sweep's name for what [`SolidView::mesh`](crate::SolidView::mesh)
    /// refuses as [`Error::AnEdgeBendsButEveryFaceAlongItIsFlat`], and by the
    /// same rule: a plane cannot say where a curve drawn on it goes, and a run
    /// naming a plane names where two planes cross, which is a straight line.
    /// Such an edge is a curve whose centre and size are not in the model, and
    /// the mesher used to draw it as a chord (`docs/decisions.md` §176.5,
    /// §187.1).
    ///
    /// A closed edge between two flat faces is the same shape, and keeps its
    /// own older name, [`Invalid::ClosedEdgeBetweenTwoFlatFaces`].
    #[error(
        "an edge bends but every face along it is flat and its run names no curved surface \
         to say where"
    )]
    AnEdgeBendsButNothingAlongItIsCurved,

    /// A curved face stands on both sides of a surface that trims it.
    ///
    /// This is the curved twin of [`Invalid::OuterWireWindsAgainstTheNormal`],
    /// and it is the part of "every wire keeps the face's material on its left"
    /// (`docs/decisions.md` §31.3) that can be asked without a direction rule for
    /// a closed edge. A crossing that goes right round cuts a surface into
    /// exactly two pieces; a face whose whole boundary is that crossing is one of
    /// them, so a corner on each side is a model whose topology and geometry
    /// disagree.
    ///
    /// Only asked of a face whose every side is a whole closed crossing. A face
    /// trimmed by a *piece* of one straddles that piece's surface quite properly
    /// — a bar with a notch cut in its wall does — and §35 says what is still
    /// owed there.
    #[error("a curved face stands on both sides of a surface that trims it")]
    AFaceStraddlesSomethingTrimmingIt,

    /// A face walks a closed edge against the way that edge is stored running.
    ///
    /// A closed edge starts and ends at the one corner it has, so its two ends
    /// say nothing about which way it goes and only a rule can. The rule is that
    /// the points along it come back the way the **longitude of the curved
    /// surface either side** increases, and that a wire walks it forward exactly
    /// when the face's material lies on the partner's negative side — which is
    /// `(∇S × ∇T) × ∇S · ∇T = −‖∇S × ∇T‖²` by Cauchy–Schwarz, everywhere, with
    /// no case analysis (`docs/decisions.md` §36.6, §36.18).
    ///
    /// Nothing else would notice: a bar's wall with both its cap circles walked
    /// the same way round closes up, counts right, and is bounded by nothing at
    /// all.
    #[error("a face walks a closed edge against the way its surface winds")]
    AClosedEdgeIsWalkedAgainstItsSurface,

    /// A wire has too few steps to close round anything.
    ///
    /// The same rule [`Error::ADraftWireIsTooShort`] states for a description,
    /// asked of a model that already exists: three steps, or one edge that
    /// closes already, or two different edges that are not both straight.
    #[error(
        "a wire of {count} steps closes round nothing: it needs three, or one edge that closes \
         already, or two different edges that are not both straight"
    )]
    WireTooShort {
        /// How many steps the wire has, poles counted.
        count: usize,
    },

    /// A wire does not close: one coedge does not start where the last ended.
    #[error(
        "a wire does not close: coedge {position} does not start where the one before it ended"
    )]
    WireDoesNotClose {
        /// Which coedge in the wire breaks the chain.
        position: usize,
    },

    /// A face's vertices are not on the face's own plane.
    #[error("a vertex of a face is {distance} away from the face's plane")]
    VertexOffItsFacePlane {
        /// How far off the plane the worst vertex is.
        distance: f64,
    },

    /// A face has no wires at all.
    #[error("a face has no wires")]
    FaceHasNoWires,

    /// A face's outer wire winds the wrong way round its normal, so the
    /// material would be on the wrong side.
    #[error("the outer wire of a face winds against the face's normal")]
    OuterWireWindsAgainstTheNormal,

    /// A hole in a face winds the same way as the outer wire instead of against
    /// it.
    #[error("a hole in a face winds the same way as the outer wire")]
    HoleWindsWithTheOuterWire,

    /// A solid's faces do not point outwards, so it encloses negative space.
    #[error("a solid encloses {volume} of space; its faces do not all point outwards")]
    VolumeIsNotPositive {
        /// The volume that came out of the divergence-theorem sum.
        volume: f64,
    },

    /// Two faces meeting along an edge do not agree which way they face, so the
    /// solid's skin does not close up.
    ///
    /// This is what catches a round cap put on back to front, and it is the
    /// only thing that can. A cap's boundary is one closed edge and one corner,
    /// so there is no polygon to wind and the winding check has nothing to ask;
    /// a *tessellated* cap does have one, and a cap facing the wrong way sends
    /// its triangles round the shared circle the same way as the wall's instead
    /// of the opposite way (decisions.md §23.9, §23.4).
    #[error("two faces of this solid meet along an edge without agreeing which way they face")]
    FacesDoNotAgreeWhichWayTheyFace,

    /// Something in the model is not used by anything above it.
    #[error("a {entity} is not used by anything")]
    Orphan {
        /// Which kind of entity is unused.
        entity: &'static str,
    },

    /// Something is used more times than the data model allows.
    #[error("a {entity} is used {count} times, and it may only be used once")]
    UsedMoreThanOnce {
        /// Which kind of entity is over-used.
        entity: &'static str,
        /// How many times it is used.
        count: usize,
    },

    /// An approximation on an entity is larger than the one on the smaller
    /// entity it is built from: `tol(vertex) >= tol(edge) >= tol(face)`
    /// (roadmap §5.4).
    #[error(
        "a {outer} carries {outer_magnitude} of slop but the {inner} it is built from carries only {inner_magnitude}"
    )]
    ApproximationOrdering {
        /// The bigger entity, which must carry the smaller number.
        outer: &'static str,
        /// How much slop it carries.
        outer_magnitude: f64,
        /// The smaller entity, which must carry the larger number.
        inner: &'static str,
        /// How much slop it carries.
        inner_magnitude: f64,
    },

    /// An entity says it is out of place by an amount that is not a length —
    /// negative, infinite, or not a number.
    ///
    /// Infinity is the one that costs something: `not_provably_off` keeps any
    /// magnitude above zero and widens the corner it is asked about into an
    /// interval reaching every surface, so a model tagged that way passes the
    /// certified check without being anywhere near its own faces
    /// (`docs/decisions.md` §44.5).
    #[error("a {entity} says it is {magnitude} out of place, which is not a length")]
    ApproximationIsNotALength {
        /// What carries the tag: a vertex, an edge or a face.
        entity: &'static str,
        /// The magnitude it claims.
        magnitude: f64,
    },

    /// The model's coordinates span too wide a range to compute in
    /// (decisions.md §3.5).
    #[error("coordinates span a range of {ratio:e}, which is too wide to compute in")]
    DynamicRangeTooWide {
        /// Largest coordinate magnitude divided by smallest feature size.
        ratio: f64,
    },
}

impl Invalid {
    /// Whether the answer the refusal stands in for exists (§16.10), as
    /// [`Error::limit_or_gap`] says it for the refusals that carry this one.
    ///
    /// A model wider than `f64` resolves is a limit of today's kernel, which
    /// holds every number as one `f64` (§437.6), and
    /// `what_a_fuse_refusal_says_about_its_answer.rs` reaches it through a
    /// fuse. Every other invariant here says not known, since nobody has shown
    /// which kind any of them is.
    ///
    /// One match and no wildcard, so a new variant does not compile until it is
    /// given a class.
    #[must_use]
    pub fn limit_or_gap(&self) -> rcad_geom::LimitOrGap {
        use rcad_geom::LimitOrGap;
        match self {
            Self::DynamicRangeTooWide { .. } => LimitOrGap::Limit,
            Self::DanglingKey { .. }
            | Self::VertexNotFinite
            | Self::EdgeHasNoLength
            | Self::VertexOffItsFaceSurface
            | Self::APoleStandsWhereTheSurfaceDoesNotPinch { .. }
            | Self::EdgeOutsideWhatPickedIt
            | Self::EdgeCoedgeCount { .. }
            | Self::CoedgesRunTheSameWay
            | Self::AnEdgeIsWalkedOnceFromEachOfTwoShells
            | Self::AFlatFaceIsUsedReversed
            | Self::AFaceUseCarriesAPlacement
            | Self::ClosedEdgeBetweenTwoFlatFaces
            | Self::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes
            | Self::AnArcBetweenTwoFlatFacesOnDifferentPlanes
            | Self::AnEdgeBendsButNothingAlongItIsCurved
            | Self::AFaceStraddlesSomethingTrimmingIt
            | Self::AClosedEdgeIsWalkedAgainstItsSurface
            | Self::WireTooShort { .. }
            | Self::WireDoesNotClose { .. }
            | Self::VertexOffItsFacePlane { .. }
            | Self::FaceHasNoWires
            | Self::OuterWireWindsAgainstTheNormal
            | Self::HoleWindsWithTheOuterWire
            | Self::VolumeIsNotPositive { .. }
            | Self::FacesDoNotAgreeWhichWayTheyFace
            | Self::Orphan { .. }
            | Self::UsedMoreThanOnce { .. }
            | Self::ApproximationOrdering { .. }
            | Self::ApproximationIsNotALength { .. } => LimitOrGap::NotKnown,
        }
    }
}

/// The result type this crate returns.
pub type Result<T, E = Error> = core::result::Result<T, E>;
