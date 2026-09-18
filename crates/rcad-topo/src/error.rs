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
    /// volume of `1e900` that fits in the type it would be returned in. The
    /// answer does not exist here, which makes this a *limit* and not a §16.10
    /// gap: no ordering and no rescaling of the arithmetic reaches a number
    /// `f64` cannot represent.
    #[error("this solid encloses more space than an f64 can hold")]
    VolumeTooLargeToMeasure,

    /// A solid has more skin than an `f64` can hold.
    ///
    /// Reachable without [`Error::VolumeTooLargeToMeasure`], because area and
    /// volume grow at different rates: a slab that is wide and thin can run out
    /// of room for its area while its volume still fits.
    #[error("this solid has more surface area than an f64 can hold")]
    SurfaceAreaTooLargeToMeasure,

    /// The model breaks one of the invariants [`validate`] checks.
    ///
    /// [`validate`]: crate::Model::validate
    #[error("the model is broken: {0}")]
    Invalid(#[from] Invalid),

    /// This kernel built an answer that does not validate, and threw it away
    /// rather than hand it over.
    ///
    /// **This is a bug here, not bad input.** It is separate from
    /// [`Error::Invalid`], which is what a caller gets when the *description*
    /// they gave was unsound — [`SolidBuilder::build`](crate::SolidBuilder::build)
    /// returns that for a corner off its own plane. Reading either as the other
    /// sends the reader looking in the wrong place, so they are two variants.
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
    #[error("this kernel built an answer that does not validate, and wrote nothing: {broken}")]
    TheFuseWroteABrokenAnswer {
        /// What the sweep of the throwaway model refused the answer for.
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
    /// **One curve on a ball with the material outside it**, which is the whole
    /// ball less a disc: a ball with a cap taken off, or either half of a
    /// two-ball union. The answer plainly exists — `18π` for two balls of radius
    /// two whose centres are two apart — and writing it wants the other half of
    /// what §38.1 built, a *cut* closed face carrying a seam and two poles of
    /// its own. Nothing builds that, so this one is **a gap**. The piece on the
    /// other side of the very same curve is answered, cut into patches no more
    /// than a quarter turn wide, and what tells the two apart is the sign of the
    /// area the wire encloses in the surface's own two coordinates, because a
    /// wire keeps its face's material on its left.
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
    /// **A face with a hole is cut only where it is a ring** — two curves that
    /// each go right round and nothing else — because then there is somewhere to
    /// cut *across* to and the answer is four patches with one loop each. That
    /// is the one the fuse makes: a tube of a bar's wall between two planes. A
    /// hole that does not go right round has nothing to cut across to, and
    /// nothing says which piece it would end up in.
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
    /// that edge to run along. The joiner in the fuse's baking step takes such a
    /// pair apart before the arrangement sees it, and a pair it does not reach —
    /// two faces on one *cylinder*, which nothing joins yet — is refused rather
    /// than arranged wrongly.
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

/// The result type this crate returns.
pub type Result<T, E = Error> = core::result::Result<T, E>;
