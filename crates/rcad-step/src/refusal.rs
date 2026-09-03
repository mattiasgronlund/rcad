use rcad_geom::Unit;

use crate::graph::EntityId;

/// What stopped a file's solids from becoming a [`Model`](rcad_topo::Model),
/// and which entity stopped them.
///
/// Every variant that can name an entity names one. That is the whole promise
/// of this half of the reader (`docs/decisions.md` §42.10): a file that will not
/// import says which `#id` it was working on and what about it could not be
/// held, so the next thing anybody does is open the file at that line.
///
/// **Nothing here is repaired.** No vertex is merged with a nearby one, no gap
/// is sewn, no surface is swapped for one the kernel does have, and no missing
/// value is filled in with a default (§42.9, which chose option A for M5).
///
/// # Limit or gap
///
/// The doc comment on each variant says which it is (§16.10). A **limit** is
/// where the answer does not exist — a file with no solid in it has no solid to
/// hand back. A **gap** is where the answer plainly exists and this code could
/// not reach it, and every one of those names what would close it.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Refusal {
    /// Nothing in the file is a `MANIFOLD_SOLID_BREP`.
    ///
    /// **Limit.** There is no solid to hand back. A file can be a perfectly
    /// good STEP file and hold none: a drawing, a tessellated export, an
    /// assembly of nothing but annotations.
    #[error("this file holds no MANIFOLD_SOLID_BREP, so it holds no solid")]
    TheFileHoldsNoSolid,

    /// A `BREP_WITH_VOIDS` — a solid with a hollow inside it.
    ///
    /// **Gap.** `SolidData::shells` is a list and a void is a second shell, so
    /// the shape is representable; what is missing is a kernel that means
    /// anything by one. `SolidBuilder::add_shell` refuses a second shell by
    /// name and `Model::validate` never counts how many a solid has (§42.5's
    /// lane, its §2). What closes it is that counting, and a mesher and a
    /// measure that read more than one shell.
    #[error("{id} is a BREP_WITH_VOIDS, and this kernel holds one shell to a solid")]
    ASolidWithAVoidInIt {
        /// The `BREP_WITH_VOIDS` instance.
        id: EntityId,
    },

    /// A surface kind the kernel has no [`Surface`](rcad_geom::Surface) for.
    ///
    /// **Limit for today's kernel and a gap for the roadmap**: the surface
    /// exists and a B-spline patch or a surface of revolution is a real shape,
    /// but there is no variant to put it in and no crossing, no sign predicate
    /// and no mesher chart for it. §42.4 counts what a variant costs. This is
    /// the commonest refusal this reader makes and §42.11 says so up front.
    #[error("{surface} is a {keyword}, which this kernel has no surface for (on face {face})")]
    ASurfaceThisKernelDoesNotHold {
        /// The face that sits on it.
        face: EntityId,
        /// The surface instance.
        surface: EntityId,
        /// The type name the file gave it.
        keyword: String,
    },

    /// A curve kind this reader cannot turn into an
    /// [`EdgeRun`](rcad_geom::EdgeRun).
    ///
    /// **Limit for today's kernel.** An edge is named by the two surfaces
    /// either side of it and never by a stored curve (§24.3), so a B-spline
    /// edge has nothing to be named by until a B-spline is a `Surface`.
    #[error("{curve} is a {keyword}, which this reader cannot name an edge by (on edge {edge})")]
    ACurveThisKernelDoesNotHold {
        /// The edge that runs along it.
        edge: EntityId,
        /// The curve instance.
        curve: EntityId,
        /// The type name the file gave it.
        keyword: String,
    },

    /// A face bound that is not an `EDGE_LOOP` — a `VERTEX_LOOP` standing on a
    /// cone's apex, or a `POLY_LOOP`.
    ///
    /// **Gap.** A wire standing still at one point is exactly
    /// [`DraftStep::Pole`](rcad_topo::DraftStep::Pole), which a ball's boundary
    /// already uses twice — but a wire of *only* a pole is one step that is not
    /// an edge, and neither door takes it. Both ask for three steps, or one edge
    /// that closes already, or two different edges that are not both straight
    /// (§44.11 item 3), and a lone pole is none of the three. What closes it is
    /// a fourth arm on that rule — one pole meaning "this whole bound is that
    /// point" — in `SolidBuilder::add_wire` and `Invalid::WireTooShort`
    /// together, which is where the two-step arm went.
    #[error("{id} is a {keyword} and this reader walks EDGE_LOOPs (on face {face})")]
    ALoopThisReaderDoesNotWalk {
        /// The face it bounds.
        face: EntityId,
        /// The loop instance.
        id: EntityId,
        /// The type name the file gave it.
        keyword: String,
    },

    /// A face with several bounds and no `FACE_OUTER_BOUND` among them.
    ///
    /// **Limit.** With several `FACE_BOUND`s and no `FACE_OUTER_BOUND`, what
    /// says which is outer is the way each winds — ISO winds an outline one way
    /// round the face's normal and a hole the other — and this is what is left
    /// when *none* of them winds forwards or *several* do. Either way the file
    /// has said something that is not a face.
    ///
    /// Neither corpus reaches it: 0 of 44 and 0 of 323. What used to reach it,
    /// 23 KiCad files, were faces bounded by two half-circles, whose corners
    /// are two points and no polygon at all — and reading those needed an arc's
    /// bulge past its own chord rather than a wider refusal.
    #[error("{face} has {bounds} bounds and none of them is a FACE_OUTER_BOUND")]
    AFaceThatDoesNotSayWhichBoundIsTheOuterOne {
        /// The face.
        face: EntityId,
        /// How many bounds it has.
        bounds: usize,
    },

    /// An arc whose two ends are at the same place, but which the file did not
    /// write as one closed edge.
    ///
    /// **Limit.** Two ends at one point name either the whole circle or nothing
    /// at all, and a file that meant the whole circle says so by giving the
    /// edge one vertex twice.
    #[error("{edge} is an arc whose two ends are the same point, so it names no one piece")]
    AnArcThatStartsAndEndsInTheSamePlace {
        /// The `EDGE_CURVE`.
        edge: EntityId,
    },

    /// A length unit the kernel's [`Unit`] does not have — an `SI_UNIT` with a
    /// prefix outside micro, milli, centi and none, or a
    /// `CONVERSION_BASED_UNIT` whose factor is not one this kernel knows.
    ///
    /// **Limit, and deliberately so.** [`Scale`](rcad_geom::Scale) carries a
    /// unit and never multiplies by one; two models in different units are two
    /// different models (§3.5). So a unit with no variant cannot be converted
    /// into one that has — that would be the silent rescale §42.12 criterion 6
    /// exists to forbid — and it is refused instead.
    #[error("{id} declares a length unit this kernel does not have: {described}")]
    ALengthUnitThisKernelDoesNotHave {
        /// The unit instance.
        id: EntityId,
        /// What the file said, in the file's own words.
        described: String,
    },

    /// The file declares no length unit at all.
    ///
    /// **Limit.** Coordinates with no unit are numbers and not lengths, and
    /// picking one would be the guessed default §42.9 refuses.
    #[error("this file declares no length unit, so its coordinates are not lengths")]
    TheFileDeclaresNoLengthUnit,

    /// Two geometric contexts in one file declare different length units.
    ///
    /// **Limit.** The kernel's coordinates are plain numbers in one declared
    /// unit; putting two units in one model would mean converting one of them,
    /// which §3.5 forbids.
    #[error("this file declares two different length units, {first:?} and {second:?}")]
    TheFileDeclaresTwoDifferentLengthUnits {
        /// The first one found.
        first: Unit,
        /// The one that disagreed with it.
        second: Unit,
    },

    /// A placement this reader would have to apply to the geometry underneath
    /// it — an assembly occurrence, or a mapped item, or a representation
    /// relationship carrying a transformation that is not the identity.
    ///
    /// **Gap, and a measured one.** §42.8 bakes world coordinates into every
    /// imported solid, so applying a placement means moving every point,
    /// every surface and every half-space that names an arc. `Surface` moves
    /// (`transformed`); a [`Picker`](rcad_geom::Picker) does not — it has
    /// `shifted` and nothing else — so an arc's own name cannot be turned.
    /// What closes it is one function, `HalfSpace::transformed`, in
    /// `rcad-geom`, which is the same gap §42.5's lane found from the other
    /// side.
    ///
    /// A transformation that *is* the identity is not refused: nothing has to
    /// move, so there is nothing this reader cannot do.
    #[error("{id} is a {keyword} placing geometry somewhere this reader cannot move it to")]
    APlacementThisReaderCannotApply {
        /// The entity carrying the placement.
        id: EntityId,
        /// Its type name.
        keyword: &'static str,
    },

    /// An entity is not the kind the schema says it has to be.
    ///
    /// **Limit.** The file is not the file it says it is, and guessing what was
    /// meant is exactly what this reader does not do.
    #[error("{id} is a {found} where the schema wants a {wanted}")]
    AnEntityIsNotTheKindTheSchemaWants {
        /// The entity.
        id: EntityId,
        /// What the schema wants there.
        wanted: &'static str,
        /// What the file has instead.
        found: String,
    },

    /// An attribute is missing, or is not the shape the schema says.
    ///
    /// **Limit**, for the same reason.
    #[error("{id}'s {keyword} attribute {position} is not the {wanted} the schema wants")]
    AnAttributeIsNotWhatTheSchemaWants {
        /// The entity.
        id: EntityId,
        /// Its type name.
        keyword: &'static str,
        /// Which attribute, counting from zero.
        position: usize,
        /// What the schema wants there.
        wanted: &'static str,
    },

    /// An integer written where the schema wants a real, too big for an `f64`
    /// to hold it exactly.
    ///
    /// **Limit.** Part 21 writes a real with a decimal point and this reader
    /// keeps `1` and `1.` apart, so an integer standing in for a real is read
    /// as the real it names — but only while the two are the same number.
    /// Past `2^53` they are not.
    #[error(
        "{id}'s attribute {position} is the integer {value}, too big to be the real it stands for"
    )]
    AnIntegerTooBigToBeTheRealItStandsFor {
        /// The entity.
        id: EntityId,
        /// Which attribute, counting from zero.
        position: usize,
        /// The integer the file wrote.
        value: i64,
    },

    /// An entity mentions an id the file never defines.
    ///
    /// **Limit.** There is nothing there to read. Every one of these is also in
    /// [`StepFile::references_that_point_nowhere`](crate::StepFile::references_that_point_nowhere);
    /// this is the one the walk actually fell into.
    #[error("{from} mentions {to}, which this file never defines")]
    AReferencePointsNowhere {
        /// The entity holding the reference.
        from: EntityId,
        /// The id it points at.
        to: EntityId,
    },

    /// A piece of geometry the file describes is not a working one — a
    /// direction of no length, a radius of zero, a coordinate that is not a
    /// finite number.
    ///
    /// **Limit.** The shape the numbers name does not exist.
    #[error("{id} does not describe a working piece of geometry")]
    TheGeometryIsNotAWorkingShape {
        /// The entity.
        id: EntityId,
        /// What the geometry crate made of it.
        #[source]
        why: rcad_geom::Error,
    },

    /// The builder refused one of the entities the file described, and this is
    /// which one.
    ///
    /// **Whether it is a limit or a gap is the builder's to say** — its twenty
    /// refusals are classified in §42.5's lane, its §2 — and what this adds is
    /// the `#id` the file wrote it under, which is the difference between "a
    /// wire does not close" and "`#1874` does not close".
    #[error("{entity}, in the solid {solid}, is not one this kernel can build")]
    ThisEntityOfTheSolidWasRefused {
        /// The `MANIFOLD_SOLID_BREP` being built.
        solid: EntityId,
        /// The entity the builder named.
        entity: EntityId,
        /// What the builder said.
        #[source]
        why: Box<rcad_topo::Error>,
    },

    /// What the file describes is not a solid, and the refusal is about the
    /// whole of it rather than about one entity.
    ///
    /// This is `Model::validate`'s own answer, arrived at through
    /// `SolidBuilder::build`: a face whose corners are off its surface, a skin
    /// that encloses no volume, a wire winding against its face's normal.
    /// **Classified where the sweep classifies it** (§39.4).
    #[error("what {solid} describes is not a solid this kernel can hold")]
    TheSolidTheFileDescribesIsNotOne {
        /// The `MANIFOLD_SOLID_BREP`.
        solid: EntityId,
        /// What the sweep said.
        #[source]
        why: Box<rcad_topo::Error>,
    },

    /// An arc whose picking half-space could not be written down.
    ///
    /// **Gap, and neither corpus reaches it — 0 of 44 and 0 of 323.** What does
    /// reach it is coordinates the dot product overflows on, which
    /// `an_arc_whose_own_numbers_overflow_a_picking_half_space_is_refused_by_name`
    /// writes at `1e200`. The half-space that says
    /// which of two arcs an edge is stands on the chord, and its offset is
    /// dropped by the width the dot product that computes it carries — eight
    /// epsilons of the sum of the term magnitudes, which is five more than the
    /// bound needs. This is what is left if that argument is wrong, or if the
    /// file's numbers overflow it: a refusal rather than a picker that refuses
    /// the very corners it was built from. `a_chord_never_leaves_its_own_ends_outside`
    /// is the test the bound is published against.
    #[error("{edge} is an arc this reader could not write a picking half-space for")]
    AnArcThisReaderCouldNotName {
        /// The `EDGE_CURVE`.
        edge: EntityId,
    },

    /// A corner the file puts on a curved face is provably not on it, and this
    /// is how far off.
    ///
    /// **This is the commonest thing a real file's own numbers do wrong, and
    /// the measurement is the point of the variant.** `Model::validate` asks a
    /// corner of a curved face for `Decision::Zero` or for a box round it that
    /// straddles the surface — a corner on a pipe of radius `r` about the `z`
    /// axis has `x² + y² − r²` cancelling to nothing — and a corner written as
    /// decimal text almost never lands on either.
    ///
    /// **And it is two different problems wearing one name**, which is what
    /// `off_by` and `coordinate_spacing` are for. In 88 of the 161 KiCad files
    /// with a corner off a curved face the worst one is inside **one**
    /// coordinate spacing: the file wrote the closest `f64` there is and the
    /// exact predicate still says the corner is off, because `x² + y² − r²`
    /// worked out exactly is not zero at any `f64` near the circle. In the rest
    /// — and the tail reaches 20 777 spacings, and 13 169 in the NIST corpus —
    /// the file carries a modelling tolerance somebody chose, and no arithmetic
    /// would put those corners on their surfaces.
    ///
    /// **The sweep no longer refuses the first kind, and this variant is still
    /// here for the second.** `Model::what_writing_a_point_down_costs` is the
    /// width of the number, `not_provably_off` widens the corner into a box that
    /// wide, and the cascade says whether anything can still prove it off. That
    /// is §42.9's option A asked with the right budget rather than option C,
    /// which tags a corner with a slop of its own and stays out of scope: a tag
    /// equal to the miss makes the check pass by construction, which is a check
    /// that has stopped asking anything.
    ///
    /// **This variant's own scan is now looser than the sweep it explains**, and
    /// says so here rather than in a report. It names a corner on the exact
    /// predicate alone, so it fires for a file the sweep would have let through
    /// and which was refused for something else — the name is read as *what this
    /// file's numbers do* and no longer as *what stopped this file*.
    ///
    /// **Option C costs one decision and no new mechanism.**
    /// `Cause::ImportedUncertainty` and `Measures::Length` both already exist,
    /// and the `Cause` is constructed in tests alone, so what is missing is a
    /// settled answer to what an imported corner's slop *is* and not a variant
    /// to carry it (`docs/decisions.md` §44.11 item 1, correcting §42.9 and
    /// §43.8 item 12).
    #[error("{corner}, on the curved face {face}, is {off_by} off the surface it sits on")]
    ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn {
        /// The `MANIFOLD_SOLID_BREP` being built.
        solid: EntityId,
        /// The face.
        face: EntityId,
        /// The `VERTEX_POINT`.
        corner: EntityId,
        /// How far off, in the file's own length unit.
        off_by: f64,
        /// How far apart two neighbouring `f64` are at this corner, which is
        /// the smallest miss its own coordinates could possibly have.
        coordinate_spacing: f64,
    },

    /// A corner at the end of an arc is provably not on the plane the arc lies
    /// in, and this is how far off.
    ///
    /// **This refusal is about the kernel's vocabulary and not about the
    /// file.** Two ends on a closed curve name two arcs, so an edge that is one
    /// carries a second surface saying which — the plane the circle lies in —
    /// and `Model::validate` asks its corners to be exactly on that plane.
    /// A [`Plane`](rcad_geom::Plane) scales its normal to length one and rounds
    /// its offset, so a corner is exactly on it only where the numbers cancel,
    /// which is an axis-aligned circle whose ends share the centre's coordinate
    /// along the axis. The corner is on the *curve*; what it is off is the
    /// stored name of the curve.
    ///
    /// **A gap.** `check_flat_face` already allows a corner one
    /// `Model::allowed_wobble` against the plane its face *sits on*, and
    /// `check_edges` allows none at all against the plane an edge is merely
    /// *named* by, which is the weaker claim of the two. What closes it is in
    /// `rcad-topo`, which is not this crate.
    #[error("{corner}, at the end of the arc {edge}, is {off_by} off the plane that arc lies in")]
    ACornerIsNotOnThePlaneOfTheArcItEndsOn {
        /// The `MANIFOLD_SOLID_BREP` being built.
        solid: EntityId,
        /// The `EDGE_CURVE`.
        edge: EntityId,
        /// The `VERTEX_POINT`.
        corner: EntityId,
        /// How far off, in the file's own length unit.
        off_by: f64,
        /// How far apart two neighbouring `f64` are at this corner.
        coordinate_spacing: f64,
    },

    /// The file's coordinates leave no working [`Scale`](rcad_geom::Scale) — a
    /// model of no size at all.
    ///
    /// **Limit.** Every coordinate in the file is zero, so there is no part
    /// there to be a size of.
    #[error("every coordinate in this file is zero, so the model has no size")]
    TheFileHasNoSize,
}
