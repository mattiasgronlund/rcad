use rcad_predicates::ProvenanceLog;

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
        /// How many faces would meet along the worst edge.
        count: usize,
    },

    /// The fuse was handed a solid with a curved face.
    ///
    /// The fuse cuts faces along the lines where their planes cross, and a
    /// cylinder has no plane to cross. Extending it to curved crossings is M4
    /// (decisions.md §1.7); until then this is a refusal and not a failure —
    /// the input is a sound solid and the operation is the thing that is not
    /// ready for it.
    ///
    /// Deliberately not [`Error::Undecidable`]. Nothing was in doubt here.
    #[error("the fuse needs every face to be flat, and one of these solids has a curved one")]
    FuseNeedsFlatFaces,

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
    #[error(
        "no mesh of this solid gets closer than {closest} to it, and the deflection asks {asked}"
    )]
    DeflectionBelowWhatAMeshCanReach {
        /// What was asked for, in the model's own unit.
        asked: f64,
        /// The closest a mesh of this solid can be, in the same unit.
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

    /// A curved face with a hole in it goes too far round its surface to be cut
    /// into triangles in one piece.
    ///
    /// A face that goes more than a quarter turn round is cut into pieces first,
    /// or every triangle in it would be stretched from one side of the seam to
    /// the other and have no area at all. Cutting a face that has a hole would
    /// have to say which piece the hole ends up in, and nothing does.
    ///
    /// **A refusal about the solid and not about the request**, and no deflection
    /// avoids it. Nothing this kernel *builds* is such a face — the two builders
    /// that make a curved one give it a single wire — but [`Model`](crate::Model)
    /// is `Deserialize`, so one can arrive from a file, and a panic is the wrong
    /// answer to a caller's bytes (decisions.md §4.1).
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

    /// Two curves drawn on one face meet at a point without crossing there.
    ///
    /// Two curves tangent at a node leave the arrangement with no anticlockwise
    /// order round it, so there is no way round the cell they bound. A **gap**:
    /// the pieces are there and telling them apart takes following one of the
    /// curves past the touch, which nothing in this kernel does.
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

    /// A straight edge starts and ends in the same place.
    ///
    /// A closed edge is not this: it comes back to where it started because it
    /// is a whole circle, which is what its
    /// [`EdgeRun`](crate::model::EdgeRun) says.
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

    /// A wire has fewer than three coedges.
    #[error("a wire has {count} coedges; a closed wire needs at least three")]
    WireTooShort {
        /// How many coedges the wire has.
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
