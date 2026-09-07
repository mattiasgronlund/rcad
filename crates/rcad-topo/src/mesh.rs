//! A whole solid cut into triangles.
//!
//! `rcad-mesh` cuts up one trimmed patch of one surface and knows nothing about
//! solids (`docs/decisions.md` §23.6). This is the layer above it: it walks each
//! face's wires into that face's own two coordinates, hands the patch over, and
//! puts what comes back into **one point table for the whole solid**.
//!
//! That table is what watertightness *is* here (§25.7). An edge's points are
//! worked out once and both faces along it name the same entries, so §23.4's
//! oracle — every mesh edge used by exactly two triangles the opposite way
//! round — is a claim about indices with no distance in it anywhere.
//!
//! **Where those points come from is not this crate's business.** §27.5 settled
//! it: [`rcad_geom::walk`] walks every edge and this layer walks none. What
//! crosses that seam is a [`NamedRun`], the surfaces of the two faces either
//! side and a [`Deflection`]; what comes back is 3D points. This layer still
//! decides *which way round* a closed edge is stored running, because that is a
//! fact about the wire and not about the curve — see [`BuildingAMesh::walk`].

use std::collections::{HashMap, HashSet};

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, MOST_POINTS_ALONG_ONE_CROSSING, NamedRun,
    POINT_ALONG_A_CROSSING_ROUNDOFF, Point, SURFACE_AFTER_SHIFTING_ROUNDOFF, Surface, Vector,
};
use rcad_mesh::{CannotTriangulate, Loop, NotALoop, TrimmedDomain, triangulate};
use rcad_predicates::{
    Approximation, Cause, Decision, Level, Measures, Point as PredicatePoint, Policy,
    ProvenanceLog, Record,
};
use smallvec::SmallVec;

use crate::fuse::name_of_surface_bits;
use crate::validate::{side_of_surface, the_way_it_is_parametrised_about};
use crate::view::{EdgeView, FaceView, VertexView, WireStepView, WireView};
use crate::{EdgeKey, Error, FaceKey, Invalid, Result, SolidView, VertexKey};

/// A solid cut into triangles.
///
/// **One point table for the whole solid**, and that is the design rather than
/// a convenience. Two faces meet along an edge; the points along that edge are
/// worked out once and both faces name the same entries of [`Mesh::points`], so
/// nothing has to be welded afterwards and no index can drift
/// (`docs/decisions.md` §25.7).
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    points: Vec<Point>,
    triangles: Vec<MeshTriangle>,
    at_vertex: HashMap<VertexKey, u32>,
    along_edge: HashMap<EdgeKey, Vec<u32>>,
    deviation: Approximation,
}

impl Mesh {
    /// Every point the triangles are drawn through.
    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// The triangles, each naming three points and the face it came from.
    ///
    /// Faces in the solid's stored order, and within a face whatever the
    /// triangulator gives back for a patch built the same way — so two runs
    /// produce the same triangles in the same order and any float sum over them
    /// comes out bit for bit the same (§25.7).
    #[must_use]
    pub fn triangles(&self) -> &[MeshTriangle] {
        &self.triangles
    }

    /// The points that stand in for one edge of the solid, as entries in
    /// [`Mesh::points`], in the order the **edge** is stored running.
    ///
    /// Only what lies **between** the edge's two ends. The ends are corners the
    /// model already held; they are [`Mesh::point_at_vertex`] of the edge's two
    /// vertices, and a caller drawing the whole edge puts them back at either
    /// end. A closed edge's one corner goes at both ends.
    ///
    /// # Which way round
    ///
    /// The direction the edge is *stored* running, which is not the direction of
    /// any one use of it. A round bar's bottom cap circle is used forwards by
    /// the wall and backwards by the cap, and both read this one list — the cap
    /// is what walks it backwards. A caller working from a coedge does the same,
    /// on `coedge.edge().orientation().is_reversed()`; the mesher's own wire
    /// walk is one copy of that and is what this convention is taken from.
    ///
    /// An edge with two different ends runs from the end it is stored starting
    /// at. A **closed** edge is pinned by neither end, and is stored running the
    /// way the longitude of the curved surface along it increases — see
    /// `BuildingAMesh::walk` for why the model needs that rule at all.
    ///
    /// # Empty, and the three ways to be empty
    ///
    /// An edge that needed no help has an empty list: a straight edge is drawn
    /// by its two ends alone, so every edge of a polyhedron is empty even though
    /// it was walked.
    ///
    /// An edge that was never walked comes back empty as well. A boundary is
    /// refined from a curved surface that bends round it, because a plane has
    /// nothing to say about where a curve drawn on it goes, so an edge with a
    /// plane on both sides is not walked at all. Nor is an edge of some other
    /// solid than the one this mesh was cut from. All three mean one thing to a
    /// caller — there is nothing to draw between the two ends — so they are not
    /// told apart here.
    ///
    /// # One list per edge, and what sharing it costs
    ///
    /// **Walked once.** The solid's faces are taken in stored order and the first
    /// curved face to reach an edge walks it; every other face along the edge
    /// reads those same entries. That is what makes the two sides of an edge name
    /// the same points rather than two sets that have to be welded (§25.7), and
    /// it is why there is one list for an edge and not one per face.
    ///
    /// **The sharing costs nothing.** The room the walk is given is the mesh's
    /// deflection less how far moving that face into place rounds, which is a
    /// fact about where the face sits and not about how tightly it bends — so the
    /// other face along the edge would have handed the walk the same number.
    /// Measured rather than argued: two balls whose radii differ by 20, 200 and
    /// 100 times, meeting in one circle, cut at three deflections each, come out
    /// with the same points, the same triangles and the same
    /// [`Mesh::deviation`] bit for bit when the stored order of the two faces is
    /// reversed — and inside that deviation every time, the closest at half of
    /// it. `whether_a_fine_face_is_handed_a_coarse_neighbours_edge.rs` is that.
    ///
    /// Those are balls, where a circle drawn on one is never wider than the ball
    /// itself, so a shared boundary always fits the room kept for it. Where a
    /// boundary does not fit, the triangulator **refuses** —
    /// [`CannotTriangulate::BoundarySideStraysFurtherThanTheDeflection`], which
    /// measures every side of a patch against the deflection before refining
    /// anything — rather than quietly cutting a coarser patch.
    #[must_use]
    pub fn points_along_edge(&self, edge: EdgeKey) -> &[u32] {
        self.along_edge.get(&edge).map_or(&[], Vec::as_slice)
    }

    /// Which entry of [`Mesh::points`] stands at a corner of the model.
    ///
    /// Exactly the point the model holds for that vertex, moved into place and
    /// moved nowhere else: a corner is never refined and never rounded off a
    /// surface, and the one corner read by every face round it is one entry
    /// rather than one per face.
    ///
    /// `None` for a vertex that is not a corner of the solid this mesh was cut
    /// from. Every corner of that solid has one, because every corner of a solid
    /// stands on a wire and every wire of every face is walked.
    #[must_use]
    pub fn point_at_vertex(&self, vertex: VertexKey) -> Option<u32> {
        self.at_vertex.get(&vertex).copied()
    }

    /// How far any point of any triangle can be from the surface it stands in
    /// for.
    ///
    /// A **length**, in the model's own unit, blaming
    /// [`Cause::FlatStandIn`](rcad_predicates::Cause::FlatStandIn), and one
    /// number for the whole mesh rather than one per face — the same rule
    /// `rcad-mesh` follows one level down (§25.3).
    ///
    /// Exactly zero for a wholly flat solid: every point of that mesh is a
    /// corner the model already held, and a flat triangle standing in for a flat
    /// surface stands in for nothing.
    ///
    /// **It is the gap under the middle of a triangle, not a distance the points
    /// were moved.** A chord cuts inside the arc it stands for, but the chord's
    /// *ends* do not: where a part sits, a certified mesh's points sit on the
    /// surface, and
    /// `a_certified_balls_mesh_points_sit_on_the_sphere_and_the_log_stays_empty`
    /// is what says so rather than this paragraph — over radii from `0.25` to
    /// `10`, at the origin and ten thousand units out. A caller who goes looking
    /// for the points to have moved will find them where they belong and
    /// conclude this number is wrong.
    ///
    /// **Far enough out they cannot sit on it, and that is arithmetic rather
    /// than a defect.** A ball smaller than the spacing of the `f64` grid it is
    /// placed on has no representable point on its own surface at all, so the
    /// points come off it by a share of the radius that grows with the distance.
    /// This number still covers them —
    /// `every_triangle_of_a_far_off_ball_stays_inside_the_deviation_the_mesh_reports`
    /// is what says so, out to `1e13` — which is the reason to read it rather
    /// than assume where the points went.
    ///
    /// It is also a **length and not a share of the shape**, which is the other
    /// half of the same surprise. On a ball whose radius is near the deflection
    /// asked for, a quarter of the radius of sag is an ordinary answer and not a
    /// defect: `a_quarter_of_a_radius_of_sag_is_what_a_deflection_near_the_radius_buys`
    /// pins it, and [`Deflection::for_scale`] is `nominal_size · 1e-3`, so a
    /// default taken from a model a thousand times a small ball's radius lands
    /// exactly there.
    ///
    /// [`Deflection::for_scale`]: rcad_geom::Deflection::for_scale
    ///
    /// # What this costs a volume measured over the triangles
    ///
    /// Write `d` for this number, `skin` for the total area of the triangles and
    /// `R` for the smallest radius any curved face of the solid bends at. A
    /// volume summed over this mesh is out by at most
    ///
    /// ```text
    ///     skin · d · (R / (R − d))²
    /// ```
    ///
    /// and that is `f64::INFINITY` unless `d < R`. Whether it is a number at all
    /// is the first thing to ask of it and not a footnote: §47.4 item 1 is a
    /// volume checked against a bound of this shape at `R = 0`, where the check
    /// could not fail.
    ///
    /// **Out either way, and not merely short.** A bar's wall bends away from
    /// its own material, so its chords lie inside the bar and the sum over them
    /// comes up short. A round *hole*'s wall bends the other way: the chords cut
    /// into the block, so the mesh is the block minus a prism inscribed in the
    /// hole and the sum comes out **over**. The shell between the triangles and
    /// the surface is the same shell and the same thickness either way, which is
    /// why the bound does not care and why it is stated as a distance.
    ///
    /// **Where it comes from.** [`Measurement::volume_error`] bounds that shell
    /// one curved face at a time, at `area · (R³ − (R − d)³) / (3(R − d)²)` with
    /// each face's own radius. Written over `u = R − d` that is
    /// `area · (d + d²/u + d³/(3u²))`, which plainly *falls* as `R` grows, so
    /// charging every face the smallest radius in the solid only raises it; and
    /// charging the whole skin rather than only its curved part raises it again.
    /// Finally `d + d²/u + d³/(3u²) ≤ d + 2d²/u + d³/u² = d·(R/u)²`, which is
    /// the expression above. So nothing is claimed here that that number does
    /// not already carry. What this form buys is that it takes three numbers
    /// instead of a walk over a mesh, so a caller who knows roughly how much
    /// skin a shape has can work out how much room to leave for it **before**
    /// meshing it — which is what a law over curved solids needs.
    ///
    /// **How tight it is, measured and not hoped for.** Four balls and eight
    /// bars, radii from 0.25 to 10, at six deflections from 1 down to 0.003:
    /// 63 of the 72 are inside the bound's own domain and the other 9 are the
    /// ones asking for a mesh a whole radius out. The closest any of the 63 came
    /// to the bound was **a fifth of it**, on a ball of radius three at a
    /// deflection of 0.003 — so five times of slack where it is tightest. It is
    /// loosest on a short wide bar, at over two hundred times what really goes
    /// missing, because it charges the flat caps' area as though it bent and
    /// nothing hides under a flat face at all. A caller who has already measured
    /// should read [`Measurement::volume_error`] instead, which charges only the
    /// curved faces: over the same 63 the closest it came was a **quarter** of
    /// itself, so four times of slack rather than five.
    ///
    /// **What goes missing falls in step with `d`, not with `d²`.** A
    /// least-squares slope of `log(missing)` against `log(deflection)` over the
    /// five finest deflections of each of the twelve shapes lands between 0.955
    /// and 0.996. Sixty measurements, because two of them fix a line through
    /// anything.
    ///
    /// **What it does not cover.** A cone that reaches its apex bends at no
    /// radius at all there, so `R` is zero and this hands back
    /// `f64::INFINITY` — honest, and not a bound. What would close that gap is
    /// written up beside the radius it needs, in `measure::radius_of_each_face`.
    /// Nor does it cover a mesh cut coarser than it was asked for: a tolerant
    /// policy may do that when the arithmetic cannot reach the request — see
    /// `SolidView::room_a_mesh_has_to_work_in` — and then the deflection the
    /// caller passed bounds nothing and only this number does. Under a certified
    /// policy this number is never more than the deflection asked for, so the
    /// deflection may be put in for it, and
    /// `a_certified_mesh_never_strays_further_than_the_deflection_it_was_asked_for`
    /// is what says so rather than a paragraph.
    ///
    /// `what_a_deflection_costs_a_measured_volume.rs` is all of the above, which
    /// is what §15.6 asks of a published bound.
    ///
    /// [`Measurement::volume_error`]: crate::Measurement::volume_error
    #[must_use]
    pub fn deviation(&self) -> Approximation {
        self.deviation
    }
}

/// One triangle of a [`Mesh`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeshTriangle {
    corners: [u32; 3],
    face: FaceKey,
}

impl MeshTriangle {
    /// Its three corners, as entries in [`Mesh::points`], wound anticlockwise
    /// seen from outside the material.
    #[must_use]
    pub fn corners(self) -> [u32; 3] {
        self.corners
    }

    /// Which face of the solid this triangle stands in for.
    #[must_use]
    pub fn face(self) -> FaceKey {
        self.face
    }
}

impl SolidView<'_> {
    /// Cut this solid into triangles that stay within `deflection` of its
    /// surfaces.
    ///
    /// # Where the work is done, and why it is not where the solid is
    ///
    /// Each face is cut up **in the frame it is stored in**, and the points are
    /// moved into place afterwards. A `(u, v)` is a working space and never an
    /// answer (§23.4), so where a solid sits must not change how it is cut up —
    /// which is §19 arriving for meshes.
    ///
    /// It is also the only thing that works. A ball is the same ball however it
    /// is turned, so [`Sphere::transformed`] under a turn about the centre moves
    /// nothing at all, and a turned ball's seam is then **not** a line of
    /// constant longitude of the turned sphere's coordinates: its face is not a
    /// rectangle in them and does not even cover them. In the frame the ball is
    /// stored in it is a rectangle, and turning the ball turns the mesh and
    /// nothing else.
    ///
    /// [`Sphere::transformed`]: rcad_geom::Sphere::transformed
    ///
    /// # Who refines the boundary
    ///
    /// The triangulator may not put a point on a loop it was handed, because it
    /// is shown one face at a time and cannot know a loop is shared (§25.1). So
    /// a curved edge is refined into intermediate points **once**, before any
    /// patch is handed over, and the identical [`Point`] values go to both faces
    /// along it — which is the whole of §23.4's promise, kept by construction
    /// rather than by rounding two answers into agreement.
    ///
    /// *Where* those points go is [`rcad_geom::walk`]'s answer and not this
    /// crate's (§27.5).
    ///
    /// # Errors
    ///
    /// [`Error::DeflectionBelowWhatAMeshCanReach`] when the deflection asks for
    /// a closer fit than a point can be placed to and the policy will not take
    /// the coarser one this would pick instead — the number it carries is that
    /// picked one, which is the larger of the two floors. And
    /// [`Error::MoreMeshPointsThanASolidWillHold`] when it asks for more points
    /// than one mesh holds. [`Error::CannotMeshAFace`] when the triangulator
    /// refuses a face, carrying what it said. [`Error::Undecidable`] when a
    /// predicate inside a triangulation could not settle a sign.
    /// [`Error::AWireWillNotLieFlatOnItsFace`],
    /// [`Error::CurvedFaceWithAHoleGoesTooFarRound`],
    /// [`Error::AnEdgeWithoutTwoFacesAlongItCannotBeWalked`] and
    /// [`Error::AWireDoesNotFanIntoALoopToMesh`] all name a malformed shape, and
    /// **only one of the three ways to reach them is one**. A solid that arrived
    /// from a file is that one. A fuse is the second: one reached
    /// `AWireWillNotLieFlatOnItsFace` with a block corner on a ball's pole, and
    /// it was the mesher that was wrong (see [`WalkingAWire::finish`]). **This
    /// call under a tolerant policy is the third**, and it is neither a file nor
    /// a bug. Where [`rcad_geom::walk`] cannot place a point as close to a
    /// crossing as it was asked to, a certified caller is refused with
    /// `CannotPlaceAPointThatCloseToACrossing` and its two numbers, and a
    /// tolerant one is waved through with a walk that has no points between its
    /// ends — so a closed edge's one corner becomes the whole of its wire, and
    /// `AWireDoesNotFanIntoALoopToMesh` arrives instead, naming the solid where
    /// the arithmetic is what gave up.
    /// [`Model::add_bar_cut_off_by`](crate::Model::add_bar_cut_off_by) makes
    /// such a solid under `Policy::certified()` with an empty provenance log, and
    /// [`Model::validate`](crate::Model::validate) passes it certified;
    /// `what_a_tolerant_policy_costs_a_mesh.rs` measures both answers over
    /// twelve lean-and-deflection cells. A wall face storing a
    /// [`Plane`](rcad_geom::Plane) where its own
    /// [`Cylinder`](rcad_geom::Cylinder) belongs fans a four-step wire into
    /// one corner in the plane's own two coordinates. All four are refusals
    /// rather than asserts precisely because
    /// [`Model::validate`](crate::Model::validate) is a separate call this
    /// does not require first, `Model` derives `Deserialize`, and a caller's
    /// bytes may not panic (CLAUDE.md, §4.1). Whatever [`rcad_geom::walk`]
    /// makes of an edge it cannot follow, which includes a seam whose
    /// half-spaces pick both halves of its meridian or neither.
    pub fn mesh(
        self,
        deflection: Deflection,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Mesh> {
        let along_each_edge = self.surfaces_along_each_edge();
        let deflection = self.room_a_mesh_has_to_work_in(deflection, policy, &along_each_edge)?;
        let mut building = BuildingAMesh::new(deflection, along_each_edge);
        for face in self.faces() {
            building.walk_the_edges_this_face_can_place(face, policy, log)?;
        }
        for face in self.faces() {
            building.cut_up(face, log)?;
        }
        Ok(building.finish())
    }

    /// The surfaces of the faces either side of each edge, as the arena holds
    /// them and with the placement kept beside them rather than composed in.
    ///
    /// **Kept apart for [`the_surface_this_face_is_meshed_on`]'s reason.** An
    /// edge is walked in the frame its face is meshed in, which for a placed
    /// face is that placement's own inverse; composing the placement in here
    /// would mean taking it straight back out again, and two movements of a
    /// surface for nothing.
    ///
    /// One entry per *use*, so a seam — one face using one edge twice — comes
    /// back as that face's surface twice, which is exactly what
    /// [`rcad_geom::walk`] wants to be told about a seam.
    fn surfaces_along_each_edge(self) -> HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>> {
        let mut along: HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>> = HashMap::new();
        for face in self.faces() {
            let surface = SurfaceAlongAnEdge {
                stored: face.stored_surface(),
                placed_at: face.location(),
            };
            for wire in face.wires() {
                for coedge in wire.coedges() {
                    along.entry(coedge.edge().key()).or_default().push(surface);
                }
            }
        }
        along
    }

    /// The deflection to work to, which is the one asked for unless that is
    /// under the floor.
    ///
    /// **There is a floor, and it depends on where the solid sits and not only
    /// on how big it is.** A point on a curved boundary comes out of an
    /// evaluator that rounds against how far out that surface reaches, and is
    /// then carried into place by the solid's own placement, which rounds again
    /// against how far out it reaches *there*. So the same ball a kilometre from
    /// the origin cannot be meshed as finely as one at the origin, and asking
    /// for finer is asking for arithmetic `f64` does not have.
    ///
    /// This is the one thing the policy decides, and it is the one §23.5 says it
    /// should: a certified caller is refused, and a tolerant one — which has
    /// already said how far out of place it will let a point be — gets a mesh as
    /// fine as the arithmetic reaches, with [`Mesh::deviation`] saying what that
    /// came to.
    ///
    /// # Two floors, and which one is read depends on who picked the number
    ///
    /// [`how_far_a_mesh_point_of_this_face_rounds`] is how close a point can be
    /// *placed*, and it is what a **caller's own** deflection is let through
    /// against. A caller who names a number gets it tried, and whatever the walk
    /// or the triangulator then says about it — including
    /// [`rcad_geom::Error::TooManyPointsAlongOneCrossing`] — is an answer about
    /// the number that caller chose.
    ///
    /// [`the_finest_deflection_the_walk_will_take`] is how fine a mesh can be
    /// *cut*, and it is what **this crate's own** number is measured against,
    /// because here there is no caller to hand the refusal to: whatever comes
    /// out of this line is walked without anyone else looking at it first. The
    /// old code took the wave-through against the placing floor as licence to
    /// use it for the cutting floor as well, and handed the walk twice a number
    /// the walk refuses — a bar of radius one at the origin got
    /// `TooManyPointsAlongOneCrossing` for a deflection this method had just
    /// worked out itself.
    ///
    /// **The policy is asked about the number it will actually get**, which is
    /// the second floor and not the first. Asking about the first and then
    /// handing back the second would be telling a caller its budget was kept and
    /// then straying seventy-nine times further than it allowed.
    fn room_a_mesh_has_to_work_in(
        self,
        asked: Deflection,
        policy: &Policy,
        along_each_edge: &HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
    ) -> Result<Deflection> {
        let closest = self
            .faces()
            .map(|face| how_far_a_mesh_point_of_this_face_rounds(face, along_each_edge))
            .fold(0.0, f64::max);
        if asked.length() >= closest {
            return Ok(asked);
        }
        let workable = self
            .faces()
            .map(|face| the_finest_deflection_the_walk_will_take(face, along_each_edge))
            .fold(0.0, f64::max);
        if !policy.wobble_is_acceptable(workable) {
            return Err(Error::DeflectionBelowWhatAMeshCanReach {
                asked: asked.length(),
                closest: workable,
            });
        }
        Ok(Deflection::of(workable * ROOM_ABOVE_THE_FLOOR)?)
    }
}

/// How far above the floor a tolerant mesh is actually cut.
///
/// Exactly at the floor there is no room left to put a point: the longest chord
/// a triangle may have shrinks to nothing and the refinement has nothing it can
/// do. Twice the floor leaves a chord of about `√(3 · radius · floor)`, which is
/// small and is not zero.
///
/// **It is not what makes the walk accept what it is handed** — that is
/// [`the_finest_deflection_the_walk_will_take`], which is the floor this is a
/// multiple of, and no multiple of the *placing* floor would have done it. What
/// the two buys on top of it is the last place or two: exactly at the floor the walk
/// asks `acos` for an angle three millionths of a radian wide, where a rounding
/// in `1 − room/reach` is amplified into the fifth digit of the answer, and a
/// step count that lands one over the cap is a refusal. Twice the room is
/// `√2` times the angle and 741 455 steps rather than 1 048 576.
const ROOM_ABOVE_THE_FLOOR: f64 = 2.0;

/// How many points one mesh will hold before this gives up on it.
///
/// Four million, which is some hundreds of megabytes once the triangles are
/// counted too. Not a knob to tune: it is the line between a mesh that is merely
/// fine and a request that will not finish, and §25.3 puts a refusal on that
/// line rather than a wait. `rcad-mesh` draws the same line one patch at a time,
/// and this one is needed as well, because the boundary walk happens before any
/// patch is handed over.
const MOST_POINTS_A_MESH_WILL_HOLD: usize = 4 * 1024 * 1024;

/// How many pieces one face may be cut into before this gives up on it.
///
/// Four pieces is the answer for every face this kernel builds — the fuse's
/// rings included, which are cut across into exactly four — and a fuse over
/// curved faces was measured, at deflections from `1e-4` to 3 and turned both
/// ways, at one or two.
///
/// **Reaching this needs sixteen cuts that each *do* make both halves
/// narrower.** The sentence that used to stand here said the opposite — that
/// reaching it means a piece is being cut without getting any smaller — and
/// [`cut_in_two`] is what says otherwise: a cut that does not narrow both halves
/// hands back `None` and the piece goes straight onto the pile. Naming the one
/// condition that provably cannot reach an error is worse than naming none,
/// because it sends whoever hits it looking for the wrong thing.
///
/// It is [`Error::ACurvedFaceCannotBeCutSmallEnoughToMesh`] rather than an assert
/// because `validate` walks this and a face can arrive from a file (§16.11).
const MOST_PIECES_A_FACE_IS_CUT_INTO: usize = 16;

/// A mesh part way through being built.
struct BuildingAMesh {
    deflection: Deflection,
    points: Vec<Point>,
    at_vertex: HashMap<VertexKey, u32>,
    along_edge: HashMap<EdgeKey, Vec<u32>>,
    triangles: Vec<MeshTriangle>,
    worst: f64,
    along_each_edge: HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
}

impl BuildingAMesh {
    fn new(
        deflection: Deflection,
        along_each_edge: HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
    ) -> Self {
        Self {
            deflection,
            points: Vec::new(),
            at_vertex: HashMap::new(),
            along_edge: HashMap::new(),
            triangles: Vec::new(),
            worst: 0.0,
            along_each_edge,
        }
    }

    /// Have every edge round `face` that has not been walked yet walked, if this
    /// face is the kind that can say where a curve goes.
    ///
    /// **A plane cannot.** A round bar's cap circle has a flat cap on one side
    /// and the wall on the other; refined in the cap's coordinates it would come
    /// out a straight chord, because a plane has nothing to say about where a
    /// circle drawn on it bends. So a boundary is refined from the curved
    /// surface that bends round it, and the flat face along the same edge reads
    /// the points that came out — the same ones, bit for bit.
    fn walk_the_edges_this_face_can_place(
        &mut self,
        face: FaceView<'_>,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let stored = the_surface_this_face_is_meshed_on(face);
        if stored.as_plane().is_some() {
            return Ok(());
        }
        let room =
            self.deflection.length() - how_far_moving_this_face_rounds(face, &self.along_each_edge);
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if self.along_edge.contains_key(&edge.key()) {
                    continue;
                }
                let walked = self.walk(edge, face, room, policy, log)?;
                self.along_edge.insert(edge.key(), walked);
            }
        }
        Ok(())
    }

    /// One edge, walked into the points that stand in for it, in the direction
    /// the edge is stored running.
    ///
    /// The two ends are not in the list: they are corners of the model, and
    /// whoever reads this puts them back at either end. So the list is empty for
    /// an edge that needs no help, which is every edge of a polyhedron.
    ///
    /// **[`rcad_geom::walk`] says where the points are and this says which way
    /// round they run**, and the two are different questions. An edge with two
    /// different ends is pinned by them, and a crossing is asked for the stretch
    /// between the two in that order. A **closed** edge starts and ends at one
    /// corner, so nothing about the crossing says which way round it goes — and
    /// the model needs the answer, because a bar's wall is a rectangle in
    /// `(longitude, height)` bounded by two cap circles used opposite ways, and
    /// it only closes if both circles are stored running the same way round the
    /// pipe. **The way the longitude of the curved surface either side of it
    /// increases is what says so**, and where both are curved `Surface`'s own
    /// fixed order picks one — so the answer depends on the edge and not on
    /// which of its two faces was walked first. It is a choice of direction
    /// rather than a second opinion about position: a reversed list is the same
    /// points.
    fn walk(
        &mut self,
        edge: EdgeView<'_>,
        face: FaceView<'_>,
        room: f64,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Vec<u32>> {
        let back = the_frame_this_face_is_meshed_in(face);
        let (from, to) = ends_in_the_stored_order(edge);
        let run = NamedRun::new(
            edge.stored_run()
                .shifted(the_shift_this_face_is_meshed_with(face))?,
            [from, to]
                .map(|end| PredicatePoint::from(back.apply_to_point(end.point()).coordinates())),
        )?;
        let along = the_two_surfaces_either_side(edge, &self.along_each_edge, &back)?;

        let along_it = rcad_geom::walk(
            &run,
            [&along[0], &along[1]],
            Deflection::of(room * WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO)?,
            policy,
            log,
        )?;
        let mut points = along_it.points().to_vec();
        if self.points.len() + points.len() > MOST_POINTS_A_MESH_WILL_HOLD {
            return Err(Error::MoreMeshPointsThanASolidWillHold {
                most: MOST_POINTS_A_MESH_WILL_HOLD,
            });
        }
        // **A closed edge is stored running the way the longitude of the curved
        // surface either side of it increases** — and *which* curved surface is
        // settled by `Surface`'s own fixed order rather than by which face got
        // here first. §30.8 item 10 has wanted a direction rule since M3 and
        // this is it.
        //
        // What the arbitrary version cost, measured: a bar's two cap circles are
        // both between a plane and one cylinder, so both were stored the
        // cylinder's way and the wall's rectangle closed — but a ball cut by a
        // plane leaves *two* caps in two different fuses, and each was stored
        // the way whichever face walked it first ran. A pole read off that is
        // right for half the caps there are.
        //
        // The rule is read **at the corner**, which is where the fuse writes it
        // and where `Model::validate` reads it back. Summing the longitude all
        // the way round instead comes to nothing on a circle that encircles no
        // pole, and leaving the direction to the evaluator there is what
        // [`the_way_a_closed_edge_is_stored_running`] measures the cost of.
        // A kind of run this crate cannot read is refused rather than taken for
        // an open edge, because the rule below is the only thing that says which
        // way round a closed edge is walked: skipping it hands back the points
        // in whichever order the walker produced them, and half the caps of a
        // ball come out walked backwards — a mesh that is wrong rather than one
        // that was refused.
        if edge.is_closed()? {
            let corner = back.apply_to_point(from.point());
            if let Some(first) = points.first()
                && let Some(stored_way) = the_way_a_closed_edge_is_stored_running(&along, corner)
                && (*first - corner).dot(stored_way) < 0.0
            {
                points.reverse();
            }
        }

        let moving = how_far_moving_this_face_rounds(face, &self.along_each_edge);
        self.worst = self.worst.max(along_it.deviation().magnitude() + moving);
        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        Ok(points
            .into_iter()
            .map(|along| {
                let at = there.apply_to_point(along);
                self.remember(at)
            })
            .collect())
    }

    /// Cut one face into triangles and put them in the solid's own table.
    fn cut_up(&mut self, face: FaceView<'_>, log: &mut ProvenanceLog) -> Result<()> {
        let stored = the_surface_this_face_is_meshed_on(face);
        let room =
            self.deflection.length() - how_far_moving_this_face_rounds(face, &self.along_each_edge);

        let mut walked = Vec::with_capacity(face.wire_count());
        for wire in face.wires() {
            walked.push(
                self.walk_a_wire_round(wire, face, &stored)
                    .ok_or(Error::AWireWillNotLieFlatOnItsFace { face: face.key() })?,
            );
        }
        for patch in self.cut_off_what_goes_too_far_round(face, &stored, walked, room)? {
            self.hand_over(face, &stored, patch, room, log)?;
        }
        Ok(())
    }

    /// The patches to hand over for one face: usually the one, but a face that
    /// goes far enough round its surface is cut into pieces first.
    ///
    /// **No patch may span more than a quarter turn, and the reason is not the
    /// shape of the triangles.** A triangle's distance from the surface is
    /// measured by its longest *chord*, which is all the triangulator can see,
    /// and a chord stops growing with the turn it spans once that turn passes a
    /// half: right the way round it is zero again. So a patch that goes all the
    /// way round is filled with triangles stretched from one seam side to the
    /// other, and every one of them has no area at all in space.
    ///
    /// A ball is where that is fatal rather than untidy. Its patch has corners
    /// on the two seam sides and nowhere else, so *every* triangle spans them,
    /// every one is dropped for standing on one point twice, and the mesh comes
    /// back empty. Half a turn is not enough either: two halves leave a mesh
    /// drawn on two meridians, which is a flat lens with no volume however many
    /// points are on it. A quarter is, because four meridians make a solid — and
    /// it is also the step [`how_many_steps`] already refuses to go past, for
    /// the same reason one level down.
    ///
    /// **A cut has to end on points both halves already have.** [`where_to_cut`]
    /// is where that is kept; everything strictly along a cut is inside the face
    /// and belongs to it alone.
    fn cut_off_what_goes_too_far_round(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        walked: Vec<WalkedLoop>,
        room: f64,
    ) -> Result<Vec<Vec<WalkedLoop>>> {
        let Some(whole_turn) = how_far_the_longitude_wraps(stored) else {
            return Ok(vec![walked]);
        };
        // **A loop drawn on a ball bounds two pieces and only one of them is a
        // patch this can fill.** The triangulator fills the inside of what it
        // is handed, and the inside of a loop in the surface's own two
        // coordinates is the piece holding no pole. When the material is on the
        // *other* side — the piece holding both poles, which a ball with a cap
        // taken off it is — the answer is the whole ball less a disc, and
        // writing it needs a seam and two poles the way §38.1 wrote an uncut
        // one. Nothing here builds that yet, so it is refused by name rather
        // than filled inside out: without this the union of two balls of radius
        // two whose centres are two apart came back `Ok` measuring 10.40, where
        // `18π = 56.55` is the union and `10π/3 = 10.47` is the *lens* between
        // them — the complement of the piece that was asked for.
        //
        // **Only asked of a surface that pinches**, which here is only a ball,
        // and on a pipe there is nothing to ask. A pipe's two coordinates are
        // longitude and height and it runs on for ever, so a loop that does not
        // go right round it bounds a patch one way and an **unbounded** region
        // the other. A solid is bounded, so its faces are, so the material is
        // the patch — there is no second case for a sign to tell apart.
        if walked.len() == 1
            && walked[0].closes_on_one_curve
            && !walked[0].goes_right_round
            && where_this_surface_pinches(stored).is_some()
            && how_much_of_the_chart_this_loop_encloses(&walked[0]) <= 0.0
        {
            return Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { face: face.key() });
        }
        let widest = whole_turn / 4.0;
        if how_far_round(&walked[0]) <= widest {
            return Ok(vec![walked]);
        }
        // **One curve right round and nothing else is a cap**, and a cap has a
        // pole to cut up to even though the model never said so. A *ring* — two
        // such curves — is cut across to the other one instead, below.
        if walked.len() == 1 && walked[0].goes_right_round {
            let Some(patches) =
                self.cut_a_cap_up_to_its_pole(face, stored, &walked[0], whole_turn, room)?
            else {
                return Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { face: face.key() });
            };
            return Ok(patches);
        }
        // A **ring** is the one face with a hole that this can cut, and it is
        // the one the fuse makes: a tube of a bar's wall between two planes,
        // bounded by two curves that each go right round and by nothing else.
        // Cut across both of them at once and what comes out is four patches
        // with one loop each, which is the shape everything below expects.
        //
        // §31.10 item 5 and §35.7 both say this claim is the one lane 5
        // falsifies, and the sentence that stops being true is *"the fuse
        // refuses a curved solid outright"*. What is still refused is a face
        // with a hole that is **not** a ring — one loop right round and another
        // that is not — because then there is nothing to cut across to, and
        // nothing says which piece the hole ends up in. `Model` is
        // `Deserialize` and `validate` is the untrusted-import boundary
        // (§16.11), so that is a refusal rather than an assert.
        if walked.len() != 1 {
            let Some(patches) = self.cut_a_ring_across(face, stored, &walked, whole_turn, room)?
            else {
                return Err(Error::CurvedFaceWithAHoleGoesTooFarRound { face: face.key() });
            };
            return Ok(patches);
        }

        let mut pieces = vec![walked.into_iter().next().expect("just checked")];
        let mut done: Vec<Vec<WalkedLoop>> = Vec::new();
        while let Some(piece) = pieces.pop() {
            if pieces.len() + done.len() >= MOST_PIECES_A_FACE_IS_CUT_INTO {
                return Err(Error::ACurvedFaceCannotBeCutSmallEnoughToMesh { face: face.key() });
            }
            match self.cut_in_two(face, stored, &piece, widest, room)? {
                Some((one, other)) => {
                    pieces.push(one);
                    pieces.push(other);
                }
                None => done.push(vec![piece]),
            }
        }
        Ok(done)
    }

    /// Cut a **cap** — one loop right round a surface that pinches to a point,
    /// with nothing else bounding it — into four patches with one loop each.
    ///
    /// A ball cut by a plane square to its polar axis leaves one, and it is
    /// about the simplest curved boolean there is. A plane at any other angle
    /// leaves a loop that climbs in longitude and comes straight back, which
    /// goes to [`BuildingAMesh::cut_in_two`] instead and needs no pole at all.
    /// Its patch in the sphere's own two coordinates is a
    /// rectangle whose top edge is the pole and whose two sides are one meridian
    /// used twice, and **the model says neither**: a
    /// [`WireStep::Pole`](crate::model::WireStep) and a seam are what say them,
    /// and a fuse has no way to invent either — a seam wants two corners at a
    /// single longitude and a cap has one corner in all. So they go in here,
    /// four times, where a pole is a corner and a meridian is a cut.
    ///
    /// **Which of the two poles the face reaches is read off the loop's own
    /// direction and costs no arithmetic.** A face keeps its material on the
    /// left of its wire, and a **ball's** two coordinates are right-handed about
    /// its outward normal — `∂u × ∂v` is `R² cos v` times it, and `cos v` is
    /// positive everywhere but the poles — so the left of "longitude
    /// increasing" is "latitude increasing". A loop running that way is the
    /// boundary of the piece **above** it. Said of a ball rather than of a
    /// surface: a torus crosses its coordinates the other way, and this reads a
    /// sign off the handedness (`every_surface_but_a_torus_crosses_its_two_coordinates_out_of_the_material`).
    ///
    /// `None` when the surface does not pinch to a point. A cylindrical face
    /// bounded by one curve right round it is half an endless tube, which is not
    /// a face, and refusing it is right.
    fn cut_a_cap_up_to_its_pole(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        cap: &WalkedLoop,
        whole_turn: f64,
        room: f64,
    ) -> Result<Option<Vec<Vec<WalkedLoop>>>> {
        if where_this_surface_pinches(stored).is_none() {
            return Ok(None);
        }
        let Some(towards) = self.which_pole_this_cap_reaches(face, stored, cap) else {
            return Ok(None);
        };

        // A quarter turn at a time, **the way the loop itself runs**, so that
        // `between` walks a quarter of the way round and not three quarters.
        let from = cap.corners[0][0];
        let way = if runs_up(cap, whole_turn) { 1.0 } else { -1.0 };
        let quarters: [f64; 4] =
            [0, 1, 2, 3].map(|step| from + way * f64::from(step) * whole_turn / 4.0);
        let along = quarters.map(|at| nearest_longitude(cap, at));

        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        let at_the_pole = stored.point_at(from, towards);
        if !at_the_pole.is_finite() {
            return Ok(None);
        }
        let at_the_pole = self.remember(there.apply_to_point(at_the_pole));

        let mut cuts = Vec::with_capacity(4);
        for &corner in &along {
            let leaving = cap.corners[corner];
            let arriving = [leaving[0], towards];
            cuts.push(self.points_along_a_cut(face, stored, leaving, arriving, room)?);
        }

        let mut patches = Vec::with_capacity(4);
        for step in 0..4 {
            let next = (step + 1) % 4;
            let mut one = unwrapped(&cap.between(along[step], along[next]), whole_turn);
            one.take_in_unwrapped(&cuts[next], whole_turn);
            // The pole twice, at the two longitudes the meridians arrive at. It
            // is one point in space and a whole side of the patch in the
            // surface's coordinates, which is what a `WireStep::Pole` says about
            // a ball this kernel builds.
            let corner = one.corners.last().map_or(0.0, |corner| corner[0]);
            one.corners.push([corner, towards]);
            one.points.push(at_the_pole);
            let back = corner - way * whole_turn / 4.0;
            one.corners.push([back, towards]);
            one.points.push(at_the_pole);
            one.take_in_unwrapped(&cuts[step].the_other_way(), whole_turn);
            patches.push(vec![one]);
        }
        Ok(Some(patches))
    }

    /// Which of a surface's two poles a cap reaches, as the latitude of it.
    ///
    /// **Not read off which way the loop runs**, and that is the trap this
    /// walked into once. A closed edge has no direction rule in the model
    /// (§30.8 item 10, still open), so which of its two coedges is the forward
    /// one says only which of the two faces along it came first — it is fixed
    /// within one solid, because the two have to disagree, and it is arbitrary
    /// against anything else. A pole chosen from it is right for half the caps
    /// there are.
    ///
    /// What is not arbitrary is the *step the loop actually takes in space*
    /// against `∇S × ∇T`. The wire keeps the face's material on its left, and
    /// left of a crossing's forward direction is always the negative side of the
    /// partner — `(∇S × ∇T) × ∇S · ∇T = −‖∇S × ∇T‖²` by Cauchy–Schwarz, with no
    /// case analysis (§36.6). So the step says which side of `T` the material is
    /// on, and the pole on that side is the one this face reaches.
    ///
    /// `None` when the face is not a cap this can read: it has no partner
    /// surface to be cut by, or its loop is too short to take a step along.
    fn which_pole_this_cap_reaches(
        &self,
        face: FaceView<'_>,
        stored: &Surface,
        cap: &WalkedLoop,
    ) -> Option<f64> {
        let pole = where_this_surface_pinches(stored)?;
        let back = the_frame_this_face_is_meshed_in(face);
        let mine = SurfaceAlongAnEdge {
            stored: face.stored_surface(),
            placed_at: face.location(),
        };
        let coedge = face.wires().flat_map(WireView::coedges).next()?;
        let partner = self
            .along_each_edge
            .get(&coedge.edge().key())?
            .iter()
            .find(|surface| **surface != mine)?
            .in_the_frame(&back);

        let [first, second] = [0, 1].map(|step| cap.points.get(step).copied());
        let (first, second) = (first?, second?);
        let at = back.apply_to_point(self.points[first as usize]);
        let step = back.apply_to_point(self.points[second as usize]) - at;
        let forward = gradient_of(stored, at).cross(gradient_of(&partner, at));
        // Along the crossing's own direction means the material is inside the
        // partner; against it means outside.
        let inside = step.dot(forward) > 0.0;

        for towards in [pole, -pole] {
            let corner = PredicatePoint::from(stored.point_at(0.0, towards).coordinates());
            let side = side_of_surface(&partner, corner);
            if (side == Decision::Neg) == inside {
                return Some(towards);
            }
        }
        None
    }

    /// Cut a **ring** — two loops that each go right round the surface, with
    /// nothing else bounding it — into four patches with one loop each.
    ///
    /// A tube is what a fuse leaves of a bar's wall between two planes, and it
    /// is the first face in this kernel with a hole in it that is not flat. It
    /// has no seam and cannot be given one: a seam wants two corners at one
    /// longitude, and the two curves bounding a tube start wherever the pair
    /// that cut each of them starts. So the seam is put in *here*, four times,
    /// where it is a cut in the surface's own two coordinates and not topology
    /// anybody has to hold.
    ///
    /// **A cut has to end on points both halves already have** — the same rule
    /// [`where_to_cut`] keeps — so each cut runs between a corner one loop
    /// already has and a corner the other already has, and the points strictly
    /// along it are made once and handed to the two patches either side.
    ///
    /// `None` when the face is not a ring: two loops is not enough, they both
    /// have to go right round, or there is nothing across to cut to.
    fn cut_a_ring_across(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        walked: &[WalkedLoop],
        whole_turn: f64,
        room: f64,
    ) -> Result<Option<Vec<Vec<WalkedLoop>>>> {
        let [outer, hole] = walked else {
            return Ok(None);
        };
        if [outer, hole].iter().any(|around| !around.goes_right_round) {
            return Ok(None);
        }

        // **The two loops of a ring run opposite ways round it**, which is what
        // keeps the material between them on the left of both, and a patch is
        // walked along the one going up and back along the one going down. Which
        // of the two is stored first is an order the face happens to hold
        // (§31.3), so it is asked rather than assumed.
        // **Both loops are walked the way the longitude grows**, and the patch
        // goes along one and back along the other. Which way each was stored is
        // an order the face happens to hold — on a closed surface neither wire
        // is the outer one (§31.3) — so it is turned to face the same way here
        // rather than assumed, and the two are brought into one longitude frame
        // before anything is measured against a longitude.
        let facing_up = |around: &WalkedLoop| {
            if runs_up(around, whole_turn) {
                around.the_other_way().the_other_way()
            } else {
                around.the_other_way()
            }
        };
        let up = facing_up(outer);
        let down = in_the_same_turn_as(&facing_up(hole), &up, whole_turn);

        let from = up.corners[0][0];
        let quarters: [f64; 4] = [0, 1, 2, 3].map(|step| from + f64::from(step) * whole_turn / 4.0);
        let along_the_top = quarters.map(|at| nearest_longitude(&up, at));
        let along_the_bottom = quarters.map(|at| nearest_longitude(&down, at));

        let mut cuts = Vec::with_capacity(4);
        for step in 0..4 {
            let leaving = up.corners[along_the_top[step]];
            let arriving = down.corners[along_the_bottom[step]];
            cuts.push(self.points_along_a_cut(face, stored, leaving, arriving, room)?);
        }

        // Every stretch is put on the end **unwrapped against what is already
        // there**. A patch that took the two chains as they came would step a
        // whole turn crossing from the end of one to the start of the other,
        // which in the surface's two coordinates is a boundary folded across
        // itself rather than a rectangle.
        let mut patches = Vec::with_capacity(4);
        for step in 0..4 {
            let next = (step + 1) % 4;
            let mut one = unwrapped(
                &up.between(along_the_top[step], along_the_top[next]),
                whole_turn,
            );
            one.take_in_unwrapped(&cuts[next], whole_turn);
            one.take_in_unwrapped(
                &down
                    .between(along_the_bottom[step], along_the_bottom[next])
                    .the_other_way(),
                whole_turn,
            );
            one.take_in_unwrapped(&cuts[step].the_other_way(), whole_turn);
            patches.push(vec![one]);
        }
        Ok(Some(patches))
    }

    /// The points strictly between two corners of one face, walked along the
    /// straight line between them in the surface's own two coordinates.
    fn points_along_a_cut(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        leaving: [f64; 2],
        arriving: [f64; 2],
        room: f64,
    ) -> Result<WalkedLoop> {
        let steps = how_many_steps(stored, leaving, arriving, room);
        if self.points.len() + steps > MOST_POINTS_A_MESH_WILL_HOLD {
            return Err(Error::MoreMeshPointsThanASolidWillHold {
                most: MOST_POINTS_A_MESH_WILL_HOLD,
            });
        }
        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        let mut along = WalkedLoop {
            corners: Vec::with_capacity(steps - 1),
            points: Vec::with_capacity(steps - 1),
            closes_on_one_curve: false,
            goes_right_round: false,
        };
        for step in 1..steps {
            let part = step as f64 / steps as f64;
            let sits_at =
                [0, 1].map(|which| leaving[which] + part * (arriving[which] - leaving[which]));
            let on_it = stored.point_at(sits_at[0], sits_at[1]);
            along.corners.push(sits_at);
            along
                .points
                .push(self.remember(there.apply_to_point(on_it)));
        }
        Ok(along)
    }

    /// Cut one piece in half across the middle of how far round it goes, or
    /// `None` if it is already narrow enough or cannot be cut any narrower.
    ///
    /// **Narrow enough is not the only reason to stop, and the other one is not
    /// a shortcoming.** A cut has to end where the loop already has a point, so
    /// the narrowest piece there is goes from one corner of a boundary chain to
    /// the next — and those are never more than a quarter turn apart, because
    /// [`how_many_steps`] will not walk further than that in one step. A piece
    /// that no cut makes smaller is therefore already about as narrow as this is
    /// asking for, and saying so is better than a fudge factor on the
    /// comparison, which is what a piece a rounding over a quarter turn would
    /// otherwise need.
    fn cut_in_two(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        piece: &WalkedLoop,
        widest: f64,
        room: f64,
    ) -> Result<Option<(WalkedLoop, WalkedLoop)>> {
        let span = how_far_round(piece);
        if span <= widest {
            return Ok(None);
        }
        let (least, most) = longitudes_of(piece);
        let Some((opened, at)) = where_to_cut(piece, (least + most) / 2.0) else {
            return Ok(None);
        };
        let mut one = opened.between(at[0], at[1]);
        let mut other = opened.between(at[1], at[0]);
        if how_far_round(&one) >= span || how_far_round(&other) >= span {
            return Ok(None);
        }

        let leaving = opened.corners[at[1]];
        let arriving = opened.corners[at[0]];
        let steps = how_many_steps(stored, leaving, arriving, room);
        if self.points.len() + steps > MOST_POINTS_A_MESH_WILL_HOLD {
            return Err(Error::MoreMeshPointsThanASolidWillHold {
                most: MOST_POINTS_A_MESH_WILL_HOLD,
            });
        }

        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        let mut along_the_cut = WalkedLoop {
            corners: Vec::with_capacity(steps - 1),
            points: Vec::with_capacity(steps - 1),
            closes_on_one_curve: false,
            goes_right_round: false,
        };
        for step in 1..steps {
            let part = step as f64 / steps as f64;
            let sits_at =
                [0, 1].map(|which| leaving[which] + part * (arriving[which] - leaving[which]));
            let on_it = stored.point_at(sits_at[0], sits_at[1]);
            along_the_cut.corners.push(sits_at);
            along_the_cut
                .points
                .push(self.remember(there.apply_to_point(on_it)));
        }

        one.take_in(&along_the_cut);
        other.take_in(&along_the_cut.the_other_way());
        Ok(Some((one, other)))
    }

    /// Hand one patch to the triangulator and take what comes back into the
    /// solid's own table.
    fn hand_over(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        patch: Vec<WalkedLoop>,
        room: f64,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let mut corner_is = Vec::new();
        let mut loops = Vec::with_capacity(patch.len());
        for one in patch {
            corner_is.extend(one.points);
            loops.push(Loop::through(one.corners).map_err(|why: NotALoop| {
                Error::AWireDoesNotFanIntoALoopToMesh {
                    face: face.key(),
                    why,
                }
            })?);
        }
        let (outer, holes) = loops.split_first().ok_or(Invalid::FaceHasNoWires)?;

        let domain = TrimmedDomain::new(
            *stored,
            outer.clone(),
            holes.to_vec(),
            Deflection::of(room)?,
        );
        let cut = match triangulate(&domain) {
            Ok(cut) => cut,
            Err(why) => return Err(refusal_for(face, why, log)),
        };

        // Everything past the corners this patch handed over is a point the
        // refinement put strictly inside it, so it belongs to this patch alone
        // and nothing else will ever name it (§25.1).
        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        for corner in &cut.corners()[corner_is.len()..] {
            let inside = stored.point_at(corner[0], corner[1]);
            corner_is.push(self.remember(there.apply_to_point(inside)));
        }

        let inside_out = face.orientation().is_reversed();
        for triangle in cut.triangles() {
            let [first, second, third] = triangle.map(|corner| corner_is[corner as usize]);
            // A pole is a *side* of the patch whose two corners are one point in
            // space, so the triangle resting on it covers nothing at all.
            // Dropping it is what turns the top row of a ball's rectangle into
            // the fan a pole really has, and the two sides left behind are one
            // and the same pair of points, used opposite ways (§25.6).
            if first == second || second == third || third == first {
                continue;
            }
            let corners = if inside_out {
                [first, third, second]
            } else {
                [first, second, third]
            };
            self.triangles.push(MeshTriangle {
                corners,
                face: face.key(),
            });
        }
        let moving = how_far_moving_this_face_rounds(face, &self.along_each_edge);
        self.worst = self.worst.max(cut.deviation().magnitude() + moving);
        Ok(())
    }

    /// One wire, as corners in the face's own two coordinates and the points of
    /// the solid's table they stand at.
    ///
    /// The two lists run together: corner `i` of the loop is point `points[i]`,
    /// and the same point may turn up at two corners. Both ends of a seam do,
    /// and so do both corners of a pole side — which is the whole reason
    /// identity lives in this table and not in the triangulator's index (§25.6).
    ///
    /// `None` when the walk does not come back to the longitude it set off
    /// from — see [`WalkingAWire::finish`].
    ///
    /// **The walk sets off from a corner that has a longitude of its own where
    /// there is one.** A corner at a pole has none — `coordinates_of` answers
    /// `atan2(0, 0)` there — and it is a whole *side* of the patch rather than a
    /// point, running from the longitude the walk arrives at to the one it
    /// leaves by. Starting on that side means the longitude it arrives at is not
    /// known until the loop closes, so the walk is turned to start elsewhere
    /// when the wire has anywhere else to start. A ball's own wire has not:
    /// every one of its steps stands at a pole, and [`WalkingAWire::arrive`]'s
    /// first-longitude rule is what carries that one.
    fn walk_a_wire_round(
        &mut self,
        wire: WireView<'_>,
        face: FaceView<'_>,
        stored: &Surface,
    ) -> Option<WalkedLoop> {
        let back = the_frame_this_face_is_meshed_in(face);
        let poles = poles_of(wire);
        let at_a_pole = |vertex: &VertexView<'_>| {
            poles.contains(&vertex.key())
                || the_surface_pinches_at(
                    stored,
                    stored.coordinates_of(back.apply_to_point(vertex.point())).1,
                )
        };
        let mut walk = WalkingAWire::new(
            how_far_the_longitude_wraps(stored),
            wire.closes_on_one_edge(),
        );

        for step in steps_from_somewhere_that_has_a_longitude(wire, &at_a_pole) {
            match step {
                WireStepView::Along(coedge) => {
                    let start = coedge.start();
                    let at = self.remember_the_corner(start);
                    let where_it_is = back.apply_to_point(start.point());
                    walk.arrive(at, stored.coordinates_of(where_it_is), at_a_pole(&start));

                    let edge = coedge.edge();
                    let walked = self
                        .along_edge
                        .get(&edge.key())
                        .cloned()
                        .unwrap_or_default();
                    let running_back = edge.orientation().is_reversed();
                    for step in 0..walked.len() {
                        let at = if running_back {
                            walked[walked.len() - 1 - step]
                        } else {
                            walked[step]
                        };
                        let where_it_is = back.apply_to_point(self.points[at as usize]);
                        walk.arrive(at, stored.coordinates_of(where_it_is), false);
                    }
                }
                WireStepView::Pole(vertex) => {
                    let at = self.remember_the_corner(vertex);
                    let where_it_is = back.apply_to_point(vertex.point());
                    walk.arrive(at, stored.coordinates_of(where_it_is), true);
                    walk.cross_the_pole();
                }
            }
        }
        walk.finish()
    }

    /// The entry in the point table for a corner of the model, put there the
    /// first time it is asked for.
    fn remember_the_corner(&mut self, vertex: VertexView<'_>) -> u32 {
        if let Some(&at) = self.at_vertex.get(&vertex.key()) {
            return at;
        }
        let at = self.remember(vertex.point());
        self.at_vertex.insert(vertex.key(), at);
        at
    }

    /// Put a point in the table and say where it went.
    fn remember(&mut self, point: Point) -> u32 {
        let at = u32::try_from(self.points.len())
            .expect("a mesh is refused long before it holds four billion points");
        self.points.push(point);
        at
    }

    fn finish(self) -> Mesh {
        Mesh {
            points: self.points,
            triangles: self.triangles,
            at_vertex: self.at_vertex,
            along_edge: self.along_edge,
            deviation: Approximation::new(self.worst, Cause::FlatStandIn),
        }
    }
}

/// One closed loop of a patch boundary: where each corner sits in the face's own
/// two coordinates, and which entry of the solid's point table it stands at.
///
/// The two lists run together, and the second is not a function of the first: a
/// seam has one point at two corners, and so does a pole side.
#[derive(Clone, Debug)]
struct WalkedLoop {
    corners: Vec<[f64; 2]>,
    points: Vec<u32>,
    /// Whether this wire is one whole closed curve — one coedge whose two ends
    /// are its one corner — with no seam and no pole side anywhere in it.
    closes_on_one_curve: bool,
    /// Whether it went right round the surface, gaining a whole turn of
    /// longitude rather than climbing and coming back.
    goes_right_round: bool,
}

impl WalkedLoop {
    /// The stretch from corner `from` round to corner `to`, both ends kept.
    fn between(&self, from: usize, to: usize) -> WalkedLoop {
        let mut taken = WalkedLoop {
            corners: Vec::new(),
            points: Vec::new(),
            closes_on_one_curve: false,
            goes_right_round: false,
        };
        let mut at = from;
        loop {
            taken.corners.push(self.corners[at]);
            taken.points.push(self.points[at]);
            if at == to {
                return taken;
            }
            at = (at + 1) % self.corners.len();
        }
    }

    /// Put `rest` on the end of this one.
    fn take_in(&mut self, rest: &WalkedLoop) {
        self.corners.extend_from_slice(&rest.corners);
        self.points.extend_from_slice(&rest.points);
    }

    /// The same, with `rest`'s longitudes moved by whole turns so that no step
    /// in the chain jumps more than half a turn.
    fn take_in_unwrapped(&mut self, rest: &WalkedLoop, whole: f64) {
        let mut longitude = self.corners.last().map_or(0.0, |corner| corner[0]);
        for corner in &rest.corners {
            longitude += the_short_way_round(corner[0] - longitude, whole);
            self.corners.push([longitude, corner[1]]);
        }
        self.points.extend_from_slice(&rest.points);
    }

    /// The same corners, walked backwards.
    fn the_other_way(&self) -> WalkedLoop {
        WalkedLoop {
            corners: self.corners.iter().rev().copied().collect(),
            points: self.points.iter().rev().copied().collect(),
            closes_on_one_curve: self.closes_on_one_curve,
            goes_right_round: self.goes_right_round,
        }
    }
}

/// Which way a surface faces at a point, as a polynomial in the numbers it
/// stores with no normalization in it (§31.2).
fn gradient_of(surface: &Surface, at: Point) -> Vector {
    match surface {
        Surface::Plane(flat) => flat.normal(),
        Surface::Sphere(ball) => at - ball.centre(),
        Surface::Cylinder(pipe) => {
            let (origin, along) = (pipe.axis().origin(), pipe.axis().direction());
            let out = at - origin;
            out * along.dot(along) - along * along.dot(out)
        }
        Surface::Cone(cone) => {
            let (apex, along) = (cone.apex(), cone.axis().direction());
            let out = at - apex;
            out * along.dot(along) - along * (along.dot(out) * (1.0 + cone.slope() * cone.slope()))
        }
        // `(w·w)(x·x + R² − r² − 2R²)x + 2R²(x·w)w` for `x = at − centre`, the
        // quartic's gradient divided by four. It is the only one here that
        // depends on how far the point is from the surface's own placing point,
        // because a torus is not a quadric.
        Surface::Torus(ring) => {
            let (centre, along) = (ring.centre(), ring.axis().direction());
            let out = at - centre;
            let (big, small) = (ring.ring_radius(), ring.tube_radius());
            let bulge = out.dot(out) - big * big - small * small;
            out * (along.dot(along) * bulge) + along * (2.0 * big * big * along.dot(out))
        }
        other => panic!("this crate cannot say which way {other:?} faces"),
    }
}

/// The latitude a surface pinches to a point at, or `None` for one that does
/// not pinch at all.
fn where_this_surface_pinches(surface: &Surface) -> Option<f64> {
    match surface {
        Surface::Sphere(_) => Some(core::f64::consts::FRAC_PI_2),
        // **A cone pinches at zero and only there**, where a ball pinches at
        // both `±π/2`. The caller tries `pole` and then `-pole`, and for a cone
        // those are the same place — `Cone::point_at(·, 0)` is the apex whatever
        // the angle — so the second try costs one predicate call and answers the
        // same, which is right rather than merely harmless: there is one apex.
        Surface::Cone(_) => Some(0.0),
        // **A torus pinches nowhere, and that is the whole of what makes its
        // chart different from a ball's.** A step of one radian round the ring
        // carries `R + r·cos v`, which is never less than `R − r` and so is
        // never nothing; a step round the tube carries `r` everywhere. Both
        // coordinates run right round instead, which costs a torus **two**
        // seams where a ball has one seam and two poles.
        Surface::Plane(_) | Surface::Cylinder(_) | Surface::Torus(_) => None,
        other => panic!("this crate cannot say whether {other:?} pinches"),
    }
}

/// Whether a loop is walked in the direction the longitude grows.
fn runs_up(around: &WalkedLoop, whole: f64) -> bool {
    how_far_the_longitude_climbed(&around.corners, whole) > 0.0
}

/// How much longitude a closed walk gains all the way round, whole turns and
/// all: zero for a loop that comes back the way it went, `±whole` for one that
/// encircles the surface's axis once.
///
/// The closing step from the last corner to the first is part of it, because a
/// loop that goes right round is stored as one turn's worth of corners with the
/// last step left implicit. No single step can be more than a quarter turn —
/// [`how_many_steps`] will not walk further than that — so taking each step the
/// short way round is taking it the way it actually went.
fn how_far_the_longitude_climbed(corners: &[[f64; 2]], whole: f64) -> f64 {
    let mut climbed = 0.0;
    for step in 0..corners.len() {
        climbed += the_short_way_round(
            corners[(step + 1) % corners.len()][0] - corners[step][0],
            whole,
        );
    }
    climbed
}

/// Which corner of a loop sits nearest a longitude.
///
/// **Not counting whole turns as no distance at all**, which is the one thing a
/// chain that goes right round makes wrong: its first corner and its last are a
/// whole turn apart and would be equally near everything, so which of the two
/// came back would decide whether the stretch after it goes a quarter of the way
/// round or three quarters.
fn nearest_longitude(around: &WalkedLoop, at: f64) -> usize {
    (0..around.corners.len())
        .min_by(|&one, &other| {
            let away = |which: usize| (around.corners[which][0] - at).abs();
            away(one).total_cmp(&away(other))
        })
        .expect("a walked loop has a corner")
}

/// One loop's longitudes moved by whole turns into the frame another was walked
/// in.
fn in_the_same_turn_as(chain: &WalkedLoop, frame: &WalkedLoop, whole: f64) -> WalkedLoop {
    let least = |around: &WalkedLoop| {
        around
            .corners
            .iter()
            .map(|corner| corner[0])
            .fold(f64::MAX, f64::min)
    };
    let by = ((least(frame) - least(chain)) / whole).round() * whole;
    WalkedLoop {
        corners: chain
            .corners
            .iter()
            .map(|corner| [corner[0] + by, corner[1]])
            .collect(),
        points: chain.points.clone(),
        closes_on_one_curve: chain.closes_on_one_curve,
        goes_right_round: chain.goes_right_round,
    }
}

/// The same corners with the whole turns put back, so that no step in the chain
/// jumps more than half a turn.
///
/// A loop that goes right round is walked as one turn's worth of longitudes and
/// then closes, so a stretch of it taken from anywhere but its own first corner
/// runs off the end and starts again — which in the surface's two coordinates is
/// a fold rather than a step. Unwrapping puts the jump back where it belongs, at
/// the seam of the patch being built.
fn unwrapped(chain: &WalkedLoop, whole: f64) -> WalkedLoop {
    let mut corners = Vec::with_capacity(chain.corners.len());
    let mut longitude = chain.corners.first().map_or(0.0, |corner| corner[0]);
    for corner in &chain.corners {
        longitude += the_short_way_round(corner[0] - longitude, whole);
        corners.push([longitude, corner[1]]);
    }
    WalkedLoop {
        corners,
        points: chain.points.clone(),
        closes_on_one_curve: chain.closes_on_one_curve,
        goes_right_round: chain.goes_right_round,
    }
}

/// The longitudes one loop runs between.
fn longitudes_of(around: &WalkedLoop) -> (f64, f64) {
    let of_each = around.corners.iter().map(|corner| corner[0]);
    (
        of_each.clone().fold(f64::MAX, f64::min),
        of_each.fold(f64::MIN, f64::max),
    )
}

/// How far round the surface one loop reaches.
fn how_far_round(around: &WalkedLoop) -> f64 {
    let (least, most) = longitudes_of(around);
    most - least
}

/// How much area one loop encloses in the surface's own two coordinates, signed:
/// **positive when the face's material is inside the loop** and negative when it
/// is outside.
///
/// A **pinching** surface's two coordinates are right-handed about its outward
/// normal — `∂u × ∂v` is `R² cos v` times it on a ball — and a wire keeps its
/// face's material on its left, so a loop with the material inside it runs
/// anticlockwise in `(u, v)` and the shoelace sum comes out positive. The number
/// itself is not used, only its sign, and it is an area in radians squared
/// rather than a size in space.
///
/// **Right-handed is not true of every surface, and the caller is what makes
/// that safe.** A torus crosses its two coordinates *into* the material, so the
/// sign of this sum would read backwards there. It never gets the chance: the
/// only call site whose sign matters is guarded by `where_this_surface_pinches`,
/// and a torus pinches nowhere. `every_surface_but_a_torus_crosses_its_two_coordinates_out_of_the_material`
/// in `rcad-geom` is the measurement, over all five surfaces.
fn how_much_of_the_chart_this_loop_encloses(around: &WalkedLoop) -> f64 {
    let corners = &around.corners;
    let mut twice_the_area = 0.0;
    for step in 0..corners.len() {
        let (here, next) = (corners[step], corners[(step + 1) % corners.len()]);
        twice_the_area += here[0] * next[1] - next[0] * here[1];
    }
    twice_the_area / 2.0
}

/// Where to cut a patch that goes too far round: the loop with the cut's two
/// ends put into it, and the positions they landed at.
///
/// **A cut has to end on points both halves already have**, or the face along
/// the boundary it lands on would stop meeting this one. So where the cut
/// reaches an ordinary side it is pulled back to whichever of that side's two
/// corners is nearer `middle` — those are corners of a shared edge and the
/// neighbouring face has them too. It is only allowed to land in the middle of a
/// side when that side's two ends are **one point**, which is a pole side, where
/// every longitude names the same place and there is nothing to tear.
///
/// So the cut is a chord of the loop rather than a line of constant longitude,
/// and it does not have to be: all that is asked of it is that neither half goes
/// as far round as the whole did.
///
/// `None` when the loop does not pass `middle` in exactly two places, which is
/// what happens when the pull-back above sends both ends of the cut to the same
/// corner: there is nothing to cut off there.
fn where_to_cut(around: &WalkedLoop, middle: f64) -> Option<(WalkedLoop, [usize; 2])> {
    let count = around.corners.len();
    let mut opened = WalkedLoop {
        corners: Vec::with_capacity(count + 2),
        points: Vec::with_capacity(count + 2),
        closes_on_one_curve: false,
        goes_right_round: false,
    };
    let mut moved_to = Vec::with_capacity(count);
    let mut places = Vec::new();

    for at in 0..count {
        let (here, next) = (around.corners[at], around.corners[(at + 1) % count]);
        moved_to.push(opened.corners.len());
        opened.corners.push(here);
        opened.points.push(around.points[at]);

        let one_end = around.points[at] == around.points[(at + 1) % count];
        if !passes(middle, here[0], next[0]) || !one_end {
            continue;
        }
        let part = (middle - here[0]) / (next[0] - here[0]);
        places.push(opened.corners.len());
        opened
            .corners
            .push([middle, here[1] + part * (next[1] - here[1])]);
        opened.points.push(around.points[at]);
    }

    for at in 0..count {
        let onwards = (at + 1) % count;
        let (here, next) = (around.corners[at], around.corners[onwards]);
        let one_end = around.points[at] == around.points[onwards];
        if !passes(middle, here[0], next[0]) || one_end {
            continue;
        }
        let nearer = if the_nearer_of(middle, here[0], next[0]) {
            at
        } else {
            onwards
        };
        places.push(moved_to[nearer]);
    }

    places.sort_unstable();
    places.dedup();
    (places.len() == 2).then(|| (opened, [places[0], places[1]]))
}

/// Whether the stretch from `from` to `to` covers `middle`.
fn passes(middle: f64, from: f64, to: f64) -> bool {
    middle >= from.min(to) && middle <= from.max(to)
}

/// Whether `here` is the one of the two to pull a cut back to.
///
/// Nearest to `middle`, and **on a tie the smaller of the two** rather than the
/// one the walk reached first. A face is cut where its boundary passes the
/// middle twice, once going out and once coming back, and the two are walked in
/// opposite directions: a tie broken by walk order sends them to opposite sides
/// and leaves a cut running diagonally across the piece, which need not make
/// either half any narrower than the whole. Broken by the number, both ends land
/// at the same longitude and every cut is a real one.
fn the_nearer_of(middle: f64, here: f64, next: f64) -> bool {
    let (mine, theirs) = ((here - middle).abs(), (next - middle).abs());
    mine < theirs || (mine == theirs && here < next)
}

/// The running state of a walk round one wire.
struct WalkingAWire {
    wraps: Option<f64>,
    /// Whether this wire is one whole closed curve, so that it is allowed to
    /// come back to where it started a whole turn on rather than at the
    /// longitude it left. Whether it *did* is measured in
    /// [`WalkingAWire::finish`] and is a different question.
    closes_on_one_curve: bool,
    longitude: f64,
    started: bool,
    will_not_lie_flat: bool,
    pole_side: f64,
    /// The corner at a pole the walk has arrived at and not yet left: the entry
    /// in the point table and the latitude, waiting for the longitude the walk
    /// leaves by. A pole is a whole side of the patch, so it needs a corner at
    /// each end of it, and only the first is known when the walk gets there.
    still_at_a_pole: Option<(u32, f64)>,
    corners: Vec<[f64; 2]>,
    points: Vec<u32>,
}

impl WalkingAWire {
    fn new(wraps: Option<f64>, closes_on_one_curve: bool) -> Self {
        Self {
            wraps,
            closes_on_one_curve,
            longitude: 0.0,
            started: false,
            will_not_lie_flat: false,
            pole_side: 1.0,
            still_at_a_pole: None,
            corners: Vec::new(),
            points: Vec::new(),
        }
    }

    /// Take the next corner: point `at` in the table, sitting at `sits_at` in
    /// the face's own two coordinates.
    ///
    /// **The longitude is unwrapped and never taken as it comes.**
    /// `coordinates_of` answers on one fixed `atan2` branch (§25.2), so a wire
    /// that goes all the way round a pipe would come back folded on top of
    /// itself and the triangulator would see a loop that crosses itself. Moving
    /// each corner by whole turns until it is within half a turn of the one
    /// before unfolds it — and that is also what gives a seam its second corner:
    /// a wire that has just been all the way round arrives back at the corner it
    /// left with a whole turn on the clock, so the patch is a rectangle rather
    /// than a loop pinched shut.
    ///
    /// A corner at a pole is the exception, because there the surface has no
    /// answer to give: every longitude names that one point, and the walk keeps
    /// the one it was already at. **A pole does not start the walk either**, and
    /// that is not the same statement. A ball's wire opens at a pole, so if a
    /// pole could set the starting longitude the walk would set off from
    /// `atan2(0, 0)` — zero — and every corner after it would be unwrapped
    /// towards a meridian the face is not on. A ball whose seam is half a turn
    /// away is where that stops being invisible and comes back as a wire that
    /// will not lie flat.
    fn arrive(&mut self, at: u32, sits_at: (f64, f64), is_a_pole: bool) {
        let (raw, up) = sits_at;
        if is_a_pole {
            self.corners.push([self.longitude, up]);
            self.points.push(at);
            self.still_at_a_pole = Some((at, up));
            return;
        }
        let longitude = match self.wraps {
            None => raw,
            // Everything walked so far was a pole and had no longitude of its
            // own, so this corner's raw one is the first fact there is. The
            // turns `cross_the_pole` counted between those poles are still
            // real: shift them all by the same amount rather than drop them.
            Some(_) if !self.started => {
                let by = raw - self.longitude;
                for corner in &mut self.corners {
                    corner[0] += by;
                }
                raw
            }
            Some(whole) => self.longitude + the_short_way_round(raw - self.longitude, whole),
        };
        self.started = true;
        self.longitude = longitude;
        self.leave_the_pole_at(longitude);
        self.corners.push([longitude, up]);
        self.points.push(at);
    }

    /// Close the pole side, if the walk is standing on one: a second corner at
    /// the same point, at the longitude the walk leaves by.
    ///
    /// Nothing to close when the two longitudes are the same number, which is
    /// what a wire that arrives at a pole and leaves along the same meridian
    /// does, and a side of no width is a corner written twice.
    fn leave_the_pole_at(&mut self, longitude: f64) {
        let Some((at, up)) = self.still_at_a_pole.take() else {
            return;
        };
        if self
            .corners
            .last()
            .is_some_and(|corner| corner[0] == longitude)
        {
            return;
        }
        self.corners.push([longitude, up]);
        self.points.push(at);
    }

    /// Cross the side of the patch that runs over a pole.
    ///
    /// A pole is one point in space and a whole side of the patch in the
    /// surface's coordinates, so the walk comes off it a full turn from where it
    /// went on. The two pole sides of a patch are walked opposite ways — round a
    /// rectangle, the bottom goes one way and the top the other — which is why
    /// the sign alternates, and why the walk comes back to the longitude it
    /// started at.
    /// A flat surface pinches to nothing, so a pole step on one is a wire that
    /// will not lie flat, and [`WalkingAWire::finish`] says so.
    fn cross_the_pole(&mut self) {
        let Some(whole) = self.wraps else {
            self.will_not_lie_flat = true;
            return;
        };
        // The wire said where the boundary goes between arriving at the pole and
        // leaving it, so the walk does not have to work it out: the step itself
        // is the far end of the pole side and the corner after it lands there.
        self.still_at_a_pole = None;
        self.longitude += self.pole_side * whole;
        self.pole_side = -self.pole_side;
    }

    /// The loop, or `None` if it will not lie flat: the walk stood still at a
    /// pole on a surface that does not pinch to one, or it came back to a
    /// different longitude from the one it set off at.
    ///
    /// **A refusal and not an assert, and the difference is `Deserialize`.**
    /// `Model` derives it and [`SolidView::mesh`] is public, so a wire this
    /// kernel never wrote can arrive here and a caller's bytes may not panic
    /// (§16.11). `a_wire_that_will_not_lie_flat_is_refused_rather_than_asserted_against`
    /// is what a ball missing one of its two pole steps does.
    ///
    /// **What it is not for is a face a fuse built, and saying so once cost a
    /// correct answer.** The sentence that stood here said a wire of a face this
    /// kernel builds cannot do either, because only a sphere has a pole and a
    /// face's two pole sides are crossed opposite ways. Two of a block's walls
    /// meeting a ball at its pole leave that pole an **ordinary corner of an
    /// ordinary wire** — no step that goes nowhere, and neither promise behind
    /// it. The walk read `atan2(0, 0)` there, one of the two faces round it came
    /// back 5.64 radians from the 1.571 it set off at and was refused, and the
    /// other was not refused and closed straight across the chart instead of
    /// along the pole;
    /// `a_pole_between_two_meridians_gets_a_corner_at_each_end_of_it` is the
    /// side that was missing.
    fn finish(mut self) -> Option<WalkedLoop> {
        if self.will_not_lie_flat {
            return None;
        }
        // A walk that ends at a pole leaves it by the way it set off, because
        // that is where the loop closes, so the pole side is the last side of
        // the patch and the longitude the walk ends at is the one it started at.
        if let Some(first) = self.corners.first().copied()
            && self.still_at_a_pole.is_some()
        {
            self.leave_the_pole_at(first[0]);
            self.longitude = first[0];
        }
        // A wire made of one whole closed curve is *allowed* to come back a
        // whole turn on — a cap circle of a tube the fuse cut out of a bar's
        // wall is one — so the check below is not about it.
        if let (Some(whole), Some(first)) = (self.wraps, self.corners.first())
            && !self.closes_on_one_curve
            && (self.longitude - first[0]).abs() >= whole / 2.0
        {
            return None;
        }
        // **Whether it went right round the surface is measured and not read
        // off the wire**, and that is §39.7 item 1. One coedge whose two ends
        // are one corner runs right round the curve its two faces cross in —
        // and on a sphere that curve is any circle at all, most of which
        // encircle no pole. A ball cut at `y = 1` leaves a cap bounded by one
        // closed edge spanning 2.64 radians of longitude and coming straight
        // back; calling that a cap right round the ball sent it looking for a
        // pole to fan up to, and it either found none (refused
        // `ACurvedFaceGoesRightRoundWithNoSeam`) or found the wrong one and
        // handed the triangulator a boundary that crosses itself.
        let goes_right_round = self.closes_on_one_curve
            && self.wraps.is_some_and(|whole| {
                how_far_the_longitude_climbed(&self.corners, whole).abs() >= whole / 2.0
            });
        Some(WalkedLoop {
            corners: self.corners,
            points: self.points,
            closes_on_one_curve: self.closes_on_one_curve,
            goes_right_round,
        })
    }
}

/// The two ends of an edge, in the order the edge is stored running rather than
/// the order this use of it runs.
///
/// A shared edge is walked once, so it has to be walked in a direction both its
/// uses agree on, and the stored one is the only direction they share.
fn ends_in_the_stored_order<'m>(edge: EdgeView<'m>) -> (VertexView<'m>, VertexView<'m>) {
    if edge.orientation().is_reversed() {
        (edge.end(), edge.start())
    } else {
        (edge.start(), edge.end())
    }
}

/// The corners `wire` itself calls poles, by taking a step that goes nowhere.
///
/// **Not every corner at a pole is one of these**, and that is the point of
/// keeping the two apart. A [`WireStep::Pole`](crate::model::WireStep) is put in
/// where a whole closed surface was cut open, and a fuse writes one only there.
/// Two cuts that meet a ball at its pole leave that pole an ordinary corner of
/// an ordinary wire, with an edge either side of it and no step that goes
/// nowhere — [`the_surface_pinches_at`] is what finds those.
fn poles_of(wire: WireView<'_>) -> HashSet<VertexKey> {
    wire.steps()
        .filter_map(|step| match step {
            WireStepView::Pole(vertex) => Some(vertex.key()),
            WireStepView::Along(_) => None,
        })
        .collect()
}

/// Whether the surface pinches to a point at this latitude, so that a corner
/// sitting there has no longitude of its own.
///
/// **Equality and no budget of its own**, which is not the same as saying it
/// answers only at the pole. A sphere reads latitude as `z.atan2(x.hypot(y))`,
/// and `atan2` rounds: at radius five that comes back exactly `π/2` for any
/// corner within about `5.5e-16` of the axis, not only for the one on it. So the
/// test catches the pole and the width of one rounding either side of it, and
/// that is the right width — a corner that close to the axis is that close to
/// the pole in space too, and the pole side written for it spans no area worth
/// the name. Widening it past what `atan2` already rounds away would start
/// throwing out longitudes that mean something.
fn the_surface_pinches_at(surface: &Surface, latitude: f64) -> bool {
    where_this_surface_pinches(surface).is_some_and(|pinch| latitude.abs() == pinch)
}

/// The steps of `wire`, turned so the walk sets off from a corner that has a
/// longitude, when the wire has one to set off from.
fn steps_from_somewhere_that_has_a_longitude<'m>(
    wire: WireView<'m>,
    at_a_pole: &impl Fn(&VertexView<'m>) -> bool,
) -> Vec<WireStepView<'m>> {
    let mut steps: Vec<WireStepView<'m>> = wire.steps().collect();
    let away = steps.iter().position(|step| match step {
        WireStepView::Along(coedge) => !at_a_pole(&coedge.start()),
        WireStepView::Pole(_) => false,
    });
    if let Some(away) = away {
        steps.rotate_left(away);
    }
    steps
}

/// The surfaces of the two faces either side of `edge`, brought into the frame
/// `back` moves into.
///
/// A seam has one face on both sides, and it is listed twice — which is what
/// tells [`rcad_geom::walk`] to read the second surface off the run itself. An
/// edge with only one use has that same face on both sides and is read the same
/// way; a closed solid has none, and one that arrived from a file may.
///
/// # Errors
///
/// [`Error::AnEdgeWithoutTwoFacesAlongItCannotBeWalked`] when three or more
/// faces meet along the edge, so no pair of them says where it runs. A refusal
/// rather than an assert, because [`SolidView::mesh`] is public and
/// [`Model`](crate::Model) is `Deserialize`: `check_edges` refuses an edge
/// without two coedges, but a solid that never went through the sweep reaches
/// here first (§16.11, §16.10).
fn the_two_surfaces_either_side(
    edge: EdgeView<'_>,
    along_each_edge: &HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
    back: &Isometry,
) -> Result<[Surface; 2]> {
    let sides = along_each_edge
        .get(&edge.key())
        .map(SmallVec::as_slice)
        .unwrap_or_default();
    let [first, second] = match sides {
        [only] => [only, only],
        [first, second] => [first, second],
        _ => {
            return Err(Error::AnEdgeWithoutTwoFacesAlongItCannotBeWalked { edge: edge.key() });
        }
    };
    Ok([first.in_the_frame(back), second.in_the_frame(back)])
}

/// A face's surface as the arena holds it, with where that face is placed kept
/// beside it rather than composed in.
#[derive(Clone, Copy, PartialEq)]
struct SurfaceAlongAnEdge {
    stored: Surface,
    placed_at: Isometry,
}

impl SurfaceAlongAnEdge {
    /// The surface seen from a frame, which for the frame a placed face is
    /// meshed in is the stored surface itself.
    ///
    /// [`the_surface_this_face_is_meshed_on`] says what that branch is worth.
    ///
    /// **A face at no placement is moved by `back` alone and not by a
    /// composition.** `back` is then [`a_shift_that_loses_nothing`], and
    /// composing anything onto it — identity included — stops it being the
    /// coordinate-at-a-time subtraction the shift's exactness rests on.
    ///
    /// The last line is for a face placed by something that is neither identity
    /// nor the movement being undone, which is an edge with faces on two
    /// differently placed shells along it. No builder makes one and
    /// [`Model`](crate::Model) is `Deserialize`, so it is composed rather than
    /// asserted against (§16.10).
    /// The surface where the solid really has it, with the placement composed in
    /// and nothing taken back out.
    ///
    /// A face at no placement is stored where it is used, and reading it out
    /// rather than moving it by the identity keeps it bit for bit — the same
    /// reason [`the_surface_this_face_is_meshed_on`] reads rather than moves.
    fn where_it_really_is(&self) -> Surface {
        if self.placed_at == Isometry::identity() {
            return self.stored;
        }
        self.stored.transformed(&self.placed_at)
    }

    fn in_the_frame(&self, back: &Isometry) -> Surface {
        if self.placed_at == Isometry::identity() {
            return self.stored.transformed(back);
        }
        if *back == self.placed_at.inverse() {
            return self.stored;
        }
        self.stored.transformed(&back.compose(&self.placed_at))
    }
}

/// Which way in space a closed edge is stored running, read at the corner it
/// starts at.
///
/// **The same rule the fuse wrote the edge with and `Model::validate` reads it
/// back with**, spelled once (§23.8):
/// [`a_closed_crossing_is_walked_the_way_it_is_stored`] asks whether the
/// crossing's forward direction `∇S × ∇T` runs with the longitude
/// `axis × ∇S` **at the corner**, and this hands back the direction that does.
///
/// **Summing the longitude all the way round instead is what this replaces, and
/// it was one authority short.** A cap circle of a bar gains a whole turn either
/// way and the sum reads it; a circle cut out of a ball by a plane that misses
/// its polar axis gains **nothing** — it climbs and comes straight back — so the
/// sum was exactly zero and the walk was left running whichever way the
/// evaluator happened to produce it. The same ball cut at `x = 1` and at `y = 1`
/// then came back with its cap walked opposite ways round: chart areas of
/// `−3.4749` and `+3.4749` for two caps that are one cap turned a quarter turn.
/// Nothing read the direction, so nothing failed — until something needs it, and
/// [`how_much_of_the_chart_this_loop_encloses`] does.
///
/// **The longitude alone is not enough, and the latitude finishes it.** Written
/// out with `n` the plane's normal, `r` the radius vector at the corner, `R` the
/// radius and `ẑ` the ball's axis, `(∇S × ∇T) · (ẑ × ∇C)` is
/// `(n·ẑ)R² − (n·r)(r·ẑ)`: a plane parallel to the ball's axis kills the first
/// term and a corner on the ball's equator the second, so a ball cut by a plane
/// along its axis — an everyday shape — leaves the dot exactly `0.0`. The
/// direction is a fact there all the same, so this is a gap and not a limit
/// (§16.10), and what it wants is a reference the wire cannot be square to.
///
/// The curved surface's frame at the corner has three legs — `∇C`, the longitude
/// `ẑ × ∇C` and the latitude `∇C × (ẑ × ∇C)` — mutually square and spanning.
/// `∇S × ∇T` is square to `∇C` because it is a cross product with it, so it lies
/// in the plane the other two span and, unless it has no length at all, cannot be
/// square to both. Read the
/// longitude first, which keeps every answer this rule gave before, and the
/// latitude only where the longitude says exactly nothing. This is the same two
/// references, read in the same order, as
/// [`a_closed_crossing_is_walked_the_way_it_is_stored`]; two spellings of one
/// question is how the fuse and `Model::validate` come to disagree (§23.8).
///
/// `None` when neither sign is a fact: neither surface is curved, so there is no
/// longitude at all; the two are tangent here, so there is no crossing
/// direction; or the corner is at a pole, where every longitude runs through the
/// one point and `axis × ∇S` is exactly zero (§38.3). Zero against both legs of
/// the frame leaves the wire with no length at all — it is square to `∇C` by
/// construction, so being square to the longitude and the latitude too leaves it
/// nothing to be but the zero vector — which is the two surfaces touching rather
/// than crossing and has no direction to name.
fn the_way_a_closed_edge_is_stored_running(along: &[Surface; 2], at: Point) -> Option<Vector> {
    let reference = the_curved_surface_that_names_a_closed_edge(along)?;
    let forward = gradient_of(&along[0], at).cross(gradient_of(&along[1], at));
    let gradient = gradient_of(&reference, at);
    let longitude = the_way_it_is_parametrised_about(&reference, at).cross(gradient);
    let with_the_longitude = forward.dot(longitude);
    let against_the_frame = if with_the_longitude == 0.0 {
        forward.dot(gradient.cross(longitude))
    } else {
        with_the_longitude
    };
    if against_the_frame == 0.0 {
        return None;
    }
    Some(if against_the_frame > 0.0 {
        forward
    } else {
        -forward
    })
}

/// Which of the two surfaces either side of a closed edge says which way it
/// runs: the curved one, and where both are curved the one `Surface`'s own fixed
/// order puts first.
///
/// `None` where neither is curved, which is a straight edge and has its two ends
/// to say which way it runs.
fn the_curved_surface_that_names_a_closed_edge(along: &[Surface; 2]) -> Option<Surface> {
    let mut curved: Vec<Surface> = along
        .iter()
        .filter(|surface| surface.as_plane().is_none())
        .copied()
        .collect();
    curved.sort_by_key(name_of_surface_bits);
    curved.first().copied()
}

/// How far this surface's first coordinate has to run to come back to where it
/// started, or `None` if it never does.
///
/// # Panics
///
/// On a surface this has not been taught. [`Surface`] grows every milestone, and
/// a new one arriving here unhandled is a broken invariant of the workspace
/// rather than bad input.
fn how_far_the_longitude_wraps(surface: &Surface) -> Option<f64> {
    match surface {
        Surface::Plane(_) => None,
        Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_) => {
            Some(core::f64::consts::TAU)
        }
        other => panic!("this crate cannot say whether {other:?} wraps round"),
    }
}

/// `angle` moved by whole turns until it is within half a turn of zero.
fn the_short_way_round(angle: f64, whole_turn: f64) -> f64 {
    angle - whole_turn * (angle / whole_turn).round()
}

/// How many pieces one edge has to be walked into.
///
/// Two questions, and the answer is the larger. **How far it bends**: a flat
/// piece with its corners on the surface strays about `chord² / (3 · radius)`
/// from it, which `rcad-mesh` derives in full beside the expression, so a chord
/// of `√(3 · radius · room)` is the longest that stays inside `room`. **How far
/// it turns**: a step of more than half a turn cannot be told from one the other
/// way when the corner is read back, so no step is allowed past a quarter of
/// one.
///
/// **The turn is the whole angle a step sweeps and not only its longitude**,
/// which on a ball means latitude counts too. A ball's seam runs pole to pole:
/// half a turn of latitude and no longitude at all. Charged on longitude alone
/// it comes back one step long, the face is then a rectangle whose only corners
/// are its two poles, every triangle of it stands on one point twice and is
/// dropped (§25.6), and a ball asked for a coarse mesh gets **no mesh at all**
/// with an honest-looking deviation on it. `a_ball_asked_for_a_rough_picture_
/// gets_a_rough_ball_and_not_an_empty_mesh` is the test.
///
/// **A cone is charged at the widest end of the step it is taking**, which is
/// why this is handed the two corners and not the gap between them. A radian
/// round a cone carries `slope · v`, so the same turn is a longer chord the
/// further from the apex it is asked, and the chord has to be measured where the
/// bound it is checked against was — `rcad-mesh`'s `chord_across_the_curve` says
/// the same thing one crate down.
///
/// # Panics
///
/// On a surface this has not been taught — see [`how_far_the_longitude_wraps`].
fn how_many_steps(stored: &Surface, leaving: [f64; 2], arriving: [f64; 2], room: f64) -> usize {
    let round = arriving[0] - leaving[0];
    let up = arriving[1] - leaving[1];
    let (radius, share, turns_over) = match stored {
        Surface::Plane(_) => return 1,
        // A ball bends both ways, so both count towards the turn as well.
        Surface::Sphere(ball) => (ball.radius(), 3.0, round.hypot(up)),
        // A pipe is flat along its axis, so a piece stretched up one bends by
        // nothing at all and only the turn counts.
        Surface::Cylinder(pipe) => (pipe.radius(), 3.0, round.abs()),
        // A cone is flat along its rulings too. Its radius of normal curvature
        // at the widest end of this step is `slope·v·√(1 + slope²)`, and the
        // chord bound over it is `C²/(2R)` rather than `C²/(3R)` —
        // `rcad-mesh`'s `how_far_a_flat_piece_strays` carries the derivation.
        Surface::Cone(cone) => (
            cone.radius_at(leaving[1].abs().max(arriving[1].abs())) * cone.slope().hypot(1.0),
            2.0,
            round.abs(),
        ),
        // **A torus bends both ways and by two different amounts**, which no
        // other surface here does: `r` round the tube and up to `R + r` round
        // the ring. `rcad-mesh`'s `how_far_a_flat_piece_strays` derives
        // `C²·(1/(3r) + 1/(2S))` for `S = R + r`, which is `C²/ρ` for the one
        // radius written below, so the share is one and the whole of the
        // stretching is in `ρ`. `turns_over` is then the arc length divided by
        // that radius, so that `radius · turns_over` is the arc length the way
        // it is for every arm above.
        Surface::Torus(ring) => {
            let widest = ring.ring_radius() + ring.tube_radius();
            let tube = ring.tube_radius();
            let effective = 6.0 * tube * widest / (2.0 * widest + 3.0 * tube);
            (
                effective,
                1.0,
                (tube * round).hypot(widest * up) / effective,
            )
        }
        other => panic!("this crate cannot walk along {other:?}"),
    };
    // A step that stays on a cone's apex covers no distance and bends by
    // nothing, so it is one piece — `how_far_a_flat_piece_strays` in `rcad-mesh`
    // makes the same allowance for the same shape and says why.
    if radius == 0.0 {
        return 1;
    }
    let longest_chord = (share * radius * room * WHAT_THE_BOUNDARY_TAKES_OF_THE_BUDGET).sqrt();
    let by_bending = (radius * turns_over / longest_chord).ceil();
    let by_turning = (turns_over / (core::f64::consts::TAU / 4.0)).ceil();
    let steps = by_bending.max(by_turning).max(1.0);
    // One past the cap, which is what refuses it a moment later. Said this way
    // round rather than cast, because `as` on a number this size — or on one
    // that is not a number at all — saturates quietly.
    if steps > MOST_POINTS_A_MESH_WILL_HOLD as f64 || steps.is_nan() {
        return MOST_POINTS_A_MESH_WILL_HOLD + 1;
    }
    steps as usize
}

/// How much of the deflection a boundary side may spend before the triangle it
/// ends up in has spent anything.
///
/// Half. The triangulator measures a triangle by its longest chord, which may
/// well be a boundary side, and adds a slop term of its own on top; leaving it
/// half the budget means a side that passed here is not the thing that makes a
/// triangle miss. A share of a budget rather than a bound on anything, so there
/// is no derivation to publish (§15.6) — what would fail if it were wrong is a
/// `BoundarySideStraysFurtherThanTheDeflection` out of a solid this kernel
/// builds, which every mesh in the suite would turn up.
const WHAT_THE_BOUNDARY_TAKES_OF_THE_BUDGET: f64 = 0.5;

/// How fine a curved edge is refined, as a share of the deflection.
///
/// Two factors, and only the first is a choice.
/// [`WHAT_THE_BOUNDARY_TAKES_OF_THE_BUDGET`] is the choice.
///
/// **Three eighths of that, and this one is arithmetic.** The evaluator sizes
/// its steps by the sagitta of the curve it is walking — `reach · (1 − cos x)`
/// for a step of `2x` radians — and `rcad-mesh` then judges the side that comes
/// out by `chord² / (3 · radius)`, the bound it derives for a flat piece with its
/// corners on a surface. Those are two measures of the same arc and the second
/// is the larger. The chord across the surface is at most `2 · reach · sin x`,
/// so what the triangulator reads is at most `(4/3) · reach · sin²x`, and
///
/// ```text
/// sin²x = (1 − cos x)(1 + cos x) ≤ 2(1 − cos x)
/// ```
///
/// puts that at eight thirds of what the evaluator was asked for. **Not a fudge
/// factor**: it is that inequality turned round, it holds for a whole turn and
/// not only for a short step, and it is why a side refined to the deflection by
/// one measure came back refused by the other the first time the two were wired
/// together.
const WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO: f64 = WHAT_THE_BOUNDARY_TAKES_OF_THE_BUDGET * 3.0 / 8.0;

/// How far out of place a point of this face's boundary can be before any
/// triangle has strayed anywhere, which is the floor under a deflection.
///
/// **Three chains in two frames, and keeping them apart is §19 again.** The
/// evaluator follows a crossing in the frame the face is **meshed** in and
/// publishes [`POINT_ALONG_A_CROSSING_ROUNDOFF`] of how far out the surfaces
/// reach *there*; the movement that carries the point to where the solid sits
/// rounds against how far out the face reaches *there*. A ball stored at the
/// origin and placed a billion units away pays the second and not the first, and
/// charging it both against the billion would put the floor under a mesh fifty
/// times higher than the arithmetic asks for.
///
/// The third is [`WHAT_SHIFTING_A_FACE_INTO_ITS_MESHING_FRAME_ROUNDS`], and a
/// face that is already worked out where it is stored does not pay it.
///
/// The evaluator's share is divided by [`WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO`],
/// because that is the fraction of the deflection it is handed and it refuses a
/// deflection it cannot place a point inside.
///
/// **This is where a point can be *placed*, and not where a mesh can be *cut*.**
/// [`the_finest_deflection_the_walk_will_take`] is the second of those and is
/// the larger — seventy-nine times larger on a bar of radius one at the origin.
/// Which of the two answers a caller depends on who chose the deflection, and
/// [`SolidView::room_a_mesh_has_to_work_in`] is where that is settled.
fn how_far_a_mesh_point_of_this_face_rounds(
    face: FaceView<'_>,
    along_each_edge: &HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
) -> f64 {
    POINT_ALONG_A_CROSSING_ROUNDOFF / WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO
        * how_far_the_points_of_reach(
            face,
            &the_frame_this_face_is_meshed_in(face),
            along_each_edge,
        )
        + how_far_moving_this_face_rounds(face, along_each_edge)
}

/// The finest deflection [`rcad_geom::walk`] will accept along a crossing of
/// this face, which is the floor under a deflection **this crate** picks.
///
/// The walk will not put more than [`MOST_POINTS_ALONG_ONE_CROSSING`] points
/// along one crossing, so the finest step it can be asked for is a whole turn
/// divided by that cap — and the arc of a step that size stands off its own
/// chord by [`how_far_a_step_at_the_cap_stands_off_its_chord`] of how far the
/// crossing reaches. That room has to survive both of the cuts on the way down:
/// [`how_far_moving_this_face_rounds`] comes off the deflection before the walk
/// is called and [`WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO`] is the share of what is
/// left that it is handed. So the two terms go in exactly where
/// [`how_far_a_mesh_point_of_this_face_rounds`]'s do, which is what makes this
/// that floor with one more thing added to it.
///
/// **It is the added term that decides.** Measured on a bar of radius one and
/// two tall at the origin: where a point can be placed is `9.2148511043888e-13`
/// and this is `7.273230840146953e-11`, seventy-nine times more. Asked for
/// `1e-300` under `Policy::tolerant(100.0)`, the mesher used to hand the walk
/// twice the first of those — `1.84297022087776e-12` — and get
/// `Error::TooManyPointsAlongOneCrossing` back on the first cap circle.
///
/// # The crossing's reach is not the face's reach, and both are worked out from
/// the crossings now
///
/// `walk`'s `how_many_steps` charges its room against `Conic::how_far_it_reaches`,
/// the longer semi-axis of the crossing itself. This charges
/// [`how_far_the_points_of_reach`], a box the whole face fits inside.
/// **Different numbers, and no longer different questions**: a face on a pipe or
/// a cone is held along its own axis by the pair of surfaces either side of each
/// of its edges, which is where its crossings are, and a surface that is bounded
/// to start with — a ball, a ring — is held by its own centre and its own reach
/// out of it.
///
/// A bar cut off by a leaning plane is what that bought. The ellipse is walked
/// as an edge of the **wall** face — a plane reaches zero here and returns
/// before any of this — and the wall's reach used to be `2` at every lean, its
/// corners a unit out and its radius one, neither of them knowing the plane
/// leaned. Measured now, on a bar of radius one cut at its seam by a plane
/// leaning `lean` in x for every 1 in z: `2000001.5` at `lean = 1e6` and
/// `4000000001.5` at `lean = 2e9`. That is `2 · lean + 1.5`, which is how the
/// ellipse itself climbs, and
/// `what_a_leaning_cut_costs_the_floor_under_a_mesh.rs` is where the floor is
/// read back off a refusal and divided by that oracle at four leans.
///
/// # Where the floor is still under the crossing's own rounding
///
/// [`how_far_along_its_axis_this_face_runs`] hands back `None` when nothing
/// along an edge bounds where that edge runs, and then the corners are all there
/// is again. **A cone crossed with a pipe on its own axis is such an edge**:
/// nothing about a direction square to the shared axis says how far along the
/// axis the crossing sits. One of them puts a whole face on that fallback, and
/// the circles a needle cone is worked out from round against a standoff its
/// corners know nothing about — so `room_left_over` empties the walk and the
/// mesh is refused by name rather than cut.
/// `what_the_mesher_does_with_an_empty_walk_on_a_closed_edge.rs` builds that
/// shape on purpose.
///
/// **So the claim here is the narrow one and not the wide one**: **not** that a
/// face's reach covers its crossings', which is still false on the fallback, but
/// that this floor is only ever read for a deflection this crate picks, and a
/// caller-named one returns from [`SolidView::room_a_mesh_has_to_work_in`] above
/// it. Extending this to a caller-named deflection would turn a floor that does
/// not bind into a wrong answer.
///
/// [`SolidView::room_a_mesh_has_to_work_in`]: SolidView::room_a_mesh_has_to_work_in
fn the_finest_deflection_the_walk_will_take(
    face: FaceView<'_>,
    along_each_edge: &HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
) -> f64 {
    (POINT_ALONG_A_CROSSING_ROUNDOFF + how_far_a_step_at_the_cap_stands_off_its_chord())
        / WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO
        * how_far_the_points_of_reach(
            face,
            &the_frame_this_face_is_meshed_in(face),
            along_each_edge,
        )
        + how_far_moving_this_face_rounds(face, along_each_edge)
}

/// How far the arc of one step stands off its own chord when a whole turn is
/// walked in as many steps as [`rcad_geom::walk`] allows, as a share of how far
/// the crossing reaches.
///
/// `walk`'s own `how_many_steps` asks for a step of `2·acos(1 − room/reach)` and
/// charges `turns_over` divided by that, so the cap it refuses past is
///
/// ```text
/// turns_over / (2·acos(1 − room/reach)) ≤ MOST_POINTS_ALONG_ONE_CROSSING
/// ```
///
/// which rearranges to `room ≥ reach · (1 − cos(turns_over / (2·cap)))`. **A
/// whole turn is the worst case and it is the usual one**: every cap circle of
/// every bar and every crossing circle a plane cuts a ball in goes right round,
/// and those are the crossings this kernel builds most of. A crossing that
/// covers less of a turn than that is held to a floor up to sixteen times
/// higher than it needed — the room goes as the square of the turn — which is a
/// mesh refused that could have been cut, and it is the safe way round: the
/// other way is the walk refusing a deflection this said was reachable.
///
/// Written as `2·sin²(x/2)` rather than `1 − cos x`, because `cos` of three
/// millionths of a radian is one to within a rounding and the subtraction keeps
/// about five digits of what is left. The `sin` form keeps them all.
fn how_far_a_step_at_the_cap_stands_off_its_chord() -> f64 {
    let widest = core::f64::consts::TAU / MOST_POINTS_ALONG_ONE_CROSSING as f64;
    2.0 * (widest / 4.0).sin().powi(2)
}

/// How far out of place a point of this face can be for reasons that have
/// nothing to do with how finely it was asked for: the movement into the frame
/// the face is meshed in, and the movement back out of it.
///
/// This is what a deflection loses before any curve is refined, so it comes off
/// the budget a boundary is refined against as well as standing in the floor.
fn how_far_moving_this_face_rounds(
    face: FaceView<'_>,
    along_each_edge: &HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
) -> f64 {
    how_far_placing_this_face_rounds(face, along_each_edge)
        + how_far_shifting_this_face_rounds(
            face,
            &the_frame_this_face_is_meshed_in(face),
            along_each_edge,
        )
}

/// This face's surface, in the frame the face is meshed in.
///
/// **Read out of the arena for a placed face, not moved there and back.** The
/// frame a placed face is meshed in is its placement's own inverse, so moving
/// [`FaceView::surface`] into it is that placement composed with its own
/// inverse — the stored surface, rounded twice for nothing.
///
/// **The two roundings were not free and nothing charged for them.** A movement
/// carries a surface by carrying the numbers that place it, and each of those
/// strays:
///
/// - A ball is placed by its centre alone, and a centre is a point, so one
///   movement strays [`Isometry::POINT_ROUNDOFF`] of the largest number
///   involved and the round trip strays twice that — `36 · f64::EPSILON · L`.
/// - A pipe, a cone and a ring are placed by a point **and a direction**. The
///   direction is turned by the same rotation arithmetic with the shift left
///   off, so it strays about as much of its own length, which is one; out at
///   `reach` from the placing point that is a displacement of the same size
///   again. So one movement strays up to `2 · Isometry::POINT_ROUNDOFF · L` and
///   the round trip up to `4 · Isometry::POINT_ROUNDOFF · L` —
///   `72 · f64::EPSILON · L`.
///
/// [`how_far_placing_this_face_rounds`] charges `Isometry::POINT_ROUNDOFF · L`
/// for carrying a mesh *point* back out to where the solid sits, and that is the
/// only placement term in the floor. The surface drifting under the point is a
/// second error of the same kind and up to four times the size, and
/// [`Mesh::deviation`] did not cover it. §41.8 item 1, §41.11 item 19.
///
/// **Neither the `256 · f64::EPSILON` of [`Surface::ROUND_TRIP_ROUNDOFF`] nor
/// the `18 · f64::EPSILON` §41.8 wrote down is the cost of this.** That constant
/// is for a round trip through a surface's *own two coordinates*, which has
/// trigonometry in it and is what `fuse::select` charges; this one is two
/// movements and nothing else.
///
/// The worst a search over four axes, fifteen angles and four distances can find
/// is `4 · f64::EPSILON · L`, and 156 of its 480 faces were moved at all;
/// `a_turned_curved_face_is_not_charged_for_a_round_trip_it_does_not_make`
/// measures both and prints them. A quarter turn about `(1, 2, 3)` takes a bar's
/// `(0, 0, 1)` axis to `(−1.1e−16, 0, 1 + 2ε)` — tilted, and no longer a unit
/// direction.
///
/// A face at no placement at all is a different thing: there is nothing to undo,
/// and the movement is [`a_shift_that_loses_nothing`], which is exact.
fn the_surface_this_face_is_meshed_on(face: FaceView<'_>) -> Surface {
    if face.location() != Isometry::identity() {
        return face.stored_surface();
    }
    face.surface()
        .transformed(&the_frame_this_face_is_meshed_in(face))
}

/// The movement that carries a point worked out in the frame this face is
/// meshed in back to where the solid sits.
///
/// **Not [`the_frame_this_face_is_meshed_in`] inverted, and the difference is a
/// whole ulp on an ordinary placement.** A placed face is meshed in its
/// placement's own inverse, so inverting that to get back out inverts the
/// placement twice — and [`Isometry::inverse`] is not an involution bit for bit.
/// It conjugates the quaternion, which only flips signs, but it replaces the
/// shift with `−R⁻¹t`, which turns a vector and rounds; inverting again turns it
/// back and rounds again. A pure shift survives that and so does a pure turn,
/// whose shift is zero and stays zero. A turn **with** a shift on it is the case
/// with nothing protecting it, and it is the ordinary case — it is what
/// [`Model::place_solid`](crate::Model::place_solid) is for and what an assembly
/// out of a file is made of. The placement itself is the exact inverse of the
/// frame, so it is what a point rides out on.
///
/// **What that ulp costs is a wrong answer and not a coarse one.** A cone's apex
/// is the one point of it with no normal, so a consumer shading a cone has to
/// find the apex among the mesh points to know not to ask — and it can only
/// find it by exact equality, because a mesh point a whisker off the apex sits
/// on a genuine ruling whose normal is a genuine answer, and handing that one
/// back for the apex would be confidently wrong rather than refused. Inverted
/// twice, a placed cone's apex point misses [`Cone::apex`](rcad_geom::Cone::apex)
/// read through a located [`FaceView`] by one ulp and the consumer shades the tip
/// with a radial direction. `a_placed_cones_mesh_holds_a_point_exactly_on_the_apex_its_face_reports`
/// is the demonstration, and it fails on `[3, 5, 7]` after a turn.
///
/// It also makes [`how_far_placing_this_face_rounds`] tell the truth: that term
/// charges one [`Isometry::POINT_ROUNDOFF`] for carrying a mesh point out to
/// where the solid sits, which is one movement, and this is now one movement.
///
/// A face at no placement at all rides out on the shift undone, which is a
/// negation and exact either way.
fn back_out_of_the_frame_this_face_is_meshed_in(face: FaceView<'_>) -> Isometry {
    if face.location() != Isometry::identity() {
        return face.location();
    }
    the_frame_this_face_is_meshed_in(face).inverse()
}

/// The frame this face's own geometry is worked out in.
///
/// Usually the frame it is **stored** in, which is where §17.1 puts a face and
/// which is what every solid this kernel builds itself keeps its curves in.
///
/// **But "stored" is not always near the origin.** A fuse bakes placement into
/// the surface it writes (§17.3), and a solid read out of a file arrives in world
/// coordinates with nothing left on it to say where it was put. The crossing
/// evaluator charges its rounding against the biggest number it was handed, so a
/// bar built a billion units up its own axis and then put through a boolean that
/// cuts nothing off it went from meshing at `1e-5` to refusing everything under
/// `3.07e-4` — a boolean that removed no material making its own answer 77 times
/// coarser (§39.7 item 2).
///
/// So a face whose stored numbers are far from the origin is shifted until they
/// are small, worked out there, and shifted back.
/// [`a_shift_that_loses_nothing`] says how far, and its name is the point.
///
/// **Only a face at no placement at all is shifted**, which is exactly the fused
/// and the imported one. A face that carries a placement is already worked out
/// in a frame somebody chose for it, and composing a shift onto a movement that
/// turns as well as shifts is no longer a subtraction one coordinate at a time —
/// which is the whole of what makes the shift exact.
fn the_frame_this_face_is_meshed_in(face: FaceView<'_>) -> Isometry {
    if face.location() != Isometry::identity() {
        return face.location().inverse();
    }
    a_shift_that_loses_nothing(face).unwrap_or_else(Isometry::identity)
}

/// How far the frame this face is meshed in was shifted off the frame it is
/// stored in, which is nothing at all for most faces.
///
/// **An [`EdgeRun`](rcad_geom::EdgeRun) is written in the frame its face is
/// stored in and has to be shifted with it.** A ball's seam is the plane its
/// meridian lies in together with the half-space picking one half of it, and a
/// ball worked out somewhere else with its seam left where it was picks a piece
/// of a crossing that is not there.
fn the_shift_this_face_is_meshed_with(face: FaceView<'_>) -> Vector {
    if face.location() != Isometry::identity() {
        return Vector::zero();
    }
    the_frame_this_face_is_meshed_in(face).translation_part()
}

/// A shift down the coordinate axes that brings this face's own numbers near the
/// origin, or `None` when there is nothing there to gain.
///
/// The numbers are the face's corners and the point its surface is placed by — a
/// ball's centre, a pipe's axis origin — taken one coordinate at a time, and
/// [`how_far_one_coordinate_can_be_shifted_for_nothing`] decides each.
///
/// **A flat face is never shifted**: its mesh is drawn through corners the model
/// already holds and rounds nothing, so [`how_far_the_points_of_reach`] gives it
/// a floor of zero wherever it sits.
fn a_shift_that_loses_nothing(face: FaceView<'_>) -> Option<Isometry> {
    let placed_by = match face.surface() {
        Surface::Plane(_) => return None,
        Surface::Sphere(ball) => ball.centre(),
        Surface::Cylinder(pipe) => pipe.axis().origin(),
        Surface::Cone(cone) => cone.apex(),
        Surface::Torus(ring) => ring.centre(),
        // A surface kind this has not been taught is worked out where it is,
        // which is what every face did before there was a shift at all.
        _ => return None,
    };
    let mut low = placed_by.coordinates();
    let mut high = low;
    for corner in face.wires().flat_map(WireView::points) {
        for (axis, coordinate) in corner.coordinates().into_iter().enumerate() {
            low[axis] = low[axis].min(coordinate);
            high[axis] = high[axis].max(coordinate);
        }
    }

    let shift = [0, 1, 2]
        .map(|axis| how_far_one_coordinate_can_be_shifted_for_nothing(low[axis], high[axis]));
    if shift == [0.0; 3] {
        return None;
    }
    Isometry::translation(Vector::new(-shift[0], -shift[1], -shift[2])).ok()
}

/// How much can be taken off every number between `low` and `high` without
/// losing a bit of any of them.
///
/// **It loses nothing, and that is a condition rather than a hope.** A coordinate
/// is moved only when every number in it has one sign and the largest is no more
/// than twice the smallest, and it is then moved by the smallest of them. That is
/// Sterbenz's condition — `a − b` is exact whenever `b/2 ≤ a ≤ 2b` — so each of
/// these subtractions loses nothing and the face worked out in the shifted frame
/// is the very same face.
///
/// A coordinate whose numbers straddle the origin, or spread over more than a
/// factor of two, comes back zero: there is nothing to take off it that would
/// make it smaller and still be exact.
///
/// `taking_the_smallest_of_a_run_of_numbers_off_all_of_them_loses_no_bits` is
/// the demonstration.
fn how_far_one_coordinate_can_be_shifted_for_nothing(low: f64, high: f64) -> f64 {
    if low > 0.0 && high <= 2.0 * low {
        low
    } else if high < 0.0 && low >= 2.0 * high {
        high
    } else {
        0.0
    }
}

/// How far moving the surfaces round this face into the frame it is meshed in
/// can put one of them from where it truly goes.
///
/// Zero for a face worked out where it is stored, because nothing moves.
///
/// **This is the neighbours' bill and not the face's own.**
/// [`a_shift_that_loses_nothing`] is chosen from one face's numbers and is exact
/// on them; the surface on the *other* side of an edge — the plane a bar's cap
/// sits on, the second pipe of a crossing — is moved by the same shift without
/// having been asked, and [`SURFACE_AFTER_SHIFTING_ROUNDOFF`] is what that
/// costs.
///
/// **This is the whole of what a fuse's baking still costs a mesh**, and it is a
/// factor of `(POINT_ROUNDOFF + SURFACE_AFTER_SHIFTING_ROUNDOFF) /
/// POINT_ROUNDOFF`, which is 1.44 rather than §39.7 item 2's 77. Measured by
/// `a_boolean_that_cuts_nothing_off_a_bar_does_not_make_it_much_coarser_to_mesh`.
/// It is a **gap** and not a limit — the answer at 1.00 exists, and reaching it
/// means measuring what the shift really cost each neighbour rather than
/// bounding it, which needs the surfaces along a face's edges at the time the
/// floor is worked out.
///
/// **A picker is not a distance and is not charged here.** The half-spaces in an
/// [`EdgeRun`](rcad_geom::EdgeRun) stray by the same amount when they are
/// shifted, and that does not move a mesh point anywhere — it decides which
/// piece of a crossing this edge is. [`HalfSpace`](rcad_geom::HalfSpace)'s own
/// doc says a strayed offset is allowed, because it moves the flat surface along
/// its own normal without turning it and so can only be wrong about *where
/// between* two pieces it sits (§29.8). To be wrong at all, the middle of the
/// piece being asked about would have to sit within the stray of the picking
/// surface — on a circle of radius one shifted a billion units, an arc under
/// about `1e-3` radians. Worth knowing rather than hidden: before the shift the
/// same decision was made against a point box `POINT_ALONG_A_CROSSING_ROUNDOFF`
/// wide in the *world's* numbers, fifty-seven times this, so nothing this can
/// get wrong was decided then either — but it was refused rather than answered,
/// and that difference is real.
fn how_far_shifting_this_face_rounds(
    face: FaceView<'_>,
    meshed_in: &Isometry,
    along_each_edge: &HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
) -> f64 {
    if *meshed_in == face.location().inverse() {
        return 0.0;
    }
    SURFACE_AFTER_SHIFTING_ROUNDOFF
        * how_far_the_points_of_reach(face, &Isometry::identity(), along_each_edge)
}

/// How far moving this face's points from the frame it is stored in to where the
/// solid sits can shift one of them.
///
/// Zero for a flat face, and exactly zero: a plane's mesh is drawn through
/// corners the model already held and nothing here made any of them. The points a
/// bar's *cap* is drawn through were made on the wall beside it, and the wall
/// charges for them — [`Mesh::deviation`] is the worst over every face, so the
/// number that comes out still covers them.
fn how_far_placing_this_face_rounds(
    face: FaceView<'_>,
    along_each_edge: &HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
) -> f64 {
    Isometry::POINT_ROUNDOFF
        * how_far_the_points_of_reach(face, &Isometry::identity(), along_each_edge)
}

/// The largest coordinate any point of this face can have, seen `through` a
/// movement.
///
/// Zero for a flat face, for the reason above.
///
/// # No surface here is charged from a corner
///
/// A **ball** and a **ring** are bounded surfaces, so the point they are
/// measured from is their own **centre**: nothing on a ball is further than
/// `radius` from it and nothing on a ring is further than `R + r`, whatever the
/// face is trimmed to. Both used to be charged `corner + radius` instead, from
/// whichever corner had the largest coordinate, and a corner is a place on the
/// surface and not the middle of it.
///
/// **That could read too low, and by at most two.** Any two points of a bounded
/// surface are within `2 · radius` of each other in every coordinate, so a face
/// reaches at most `corner + 2 · radius` while it was charged `corner + radius`,
/// and the ratio between those is worst when `corner` is zero.
///
/// **No solid this crate can build reaches that**, and both of the ones it can
/// read too *high* instead. A ball's poles sit at `centre ± radius · axis`, so
/// one of the two has a largest coordinate of at least the centre's; a bend's
/// two corners sit `R − r` from the centre along each of two square directions,
/// so between them the same holds. Either way `corner + radius` is already at
/// or above `|centre| + radius`, which holds the whole surface — a unit ball at
/// the origin was charged `2` for a face reaching `1`, and a bend of ring four
/// and tube one `8` for one reaching `5`. Reading it too low wants a face whose
/// corners **all** sit nearer the origin than its centre does, which takes a
/// trimmed ball or a trimmed ring; a fuse and a hand-written solid are the two
/// doors to one. `where_a_ball_and_a_ring_are_measured_from.rs` measures both
/// over-charges and says the same thing at more length.
///
/// A **pipe** and a **cone** are not bounded. They run on for ever along their own axis,
/// and how far along a face runs is not something the face's *corners* say — it
/// is what its **edges** say, and an edge is a curve with a middle. A bar of
/// radius one cut by a plane leaning `L` in x for every 1 in z has both its
/// corners on the seam and neither of them out of the ordinary, while the
/// ellipse those corners sit on climbs `L` up the far side. The corner charge
/// was `2` at every lean, against a face reaching `L`; §52.7 is the same
/// mistake one level up and `what_a_deflection_costs_a_measured_volume.rs` is
/// where the two charges are printed side by side.
///
/// So for those two the axis is charged from the **crossings**, which is what
/// [`how_far_along_its_axis_this_face_runs`] works out from the surfaces either
/// side of each edge. When it cannot — a crossing that runs on for ever on both
/// surfaces at once — this falls back to the corner charge, which is where the
/// under-estimate still lives, and says so there.
///
/// # Panics
///
/// On a surface this has not been taught — see [`how_far_the_longitude_wraps`].
fn how_far_the_points_of_reach(
    face: FaceView<'_>,
    through: &Isometry,
    along_each_edge: &HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
) -> f64 {
    let corners = face
        .wires()
        .flat_map(WireView::points)
        .map(|corner| {
            through
                .apply_to_point(corner)
                .largest_coordinate_magnitude()
        })
        .fold(0.0, f64::max);
    // A ball and a ring are held by their own centre and their own reach out of
    // it, whichever way a face on one wraps — and a face on a ring can wrap
    // **both** ways, so nothing about the trimming would help even if it were
    // read.
    let held_from = |centre: Point, out: f64| {
        corners.max(
            through
                .apply_to_point(centre)
                .largest_coordinate_magnitude()
                + out,
        )
    };
    // A pipe is `radius` out all the way along; a cone's axis origin is its apex
    // and it is `slope · along` out there. One pair of numbers covers both.
    let (axis, radius, slope) = match face.surface() {
        Surface::Plane(_) => return 0.0,
        Surface::Sphere(ball) => return held_from(ball.centre(), ball.radius()),
        Surface::Torus(ring) => {
            return held_from(ring.centre(), ring.ring_radius() + ring.tube_radius());
        }
        Surface::Cylinder(pipe) => (pipe.axis(), pipe.radius(), 0.0),
        Surface::Cone(cone) => (cone.axis(), 0.0, cone.slope()),
        other => panic!("this crate cannot say how far {other:?} reaches"),
    };
    let Some(along) =
        how_far_along_its_axis_this_face_runs(face, &axis, radius, slope, along_each_edge)
    else {
        return corners + how_far_out_this_face_is_where_its_corners_are(face);
    };
    // Every point of the face is `along` up the axis at most and `radius +
    // slope · along` out from it, so it is inside a box that far from the point
    // the axis is placed by — and a movement carries that whole box.
    let out = radius + slope * along;
    corners.max(
        through
            .apply_to_point(axis.origin())
            .largest_coordinate_magnitude()
            + along
            + out,
    )
}

/// How far out from its own axis this face gets, judged from its corners alone.
///
/// **The under-estimate this is named after.** A pipe is its radius out
/// everywhere, which is right; a cone is `slope · along` out and the corners are
/// the only places this looks, which is right only where a corner is the
/// furthest along the face gets. It is what [`how_far_the_points_of_reach`]
/// falls back to when no pair of surfaces along an edge bounds where that edge
/// runs. `the_floor_under_a_leaning_bar_follows_its_ellipse_and_not_its_corners`
/// says a bar cut by a leaning plane is no longer one of those, and
/// `what_the_mesher_does_with_an_empty_walk_on_a_closed_edge.rs` is a shape that
/// still is: a cone met by a pipe on its own axis, where the fallback charges
/// `2` against circles worked out from a standoff of sixteen million.
fn how_far_out_this_face_is_where_its_corners_are(face: FaceView<'_>) -> f64 {
    match face.surface() {
        Surface::Cylinder(pipe) => pipe.radius(),
        Surface::Cone(cone) => {
            let along = cone.axis().direction();
            face.wires()
                .flat_map(WireView::points)
                .map(|corner| cone.radius_at(along.dot(corner - cone.apex())))
                .fold(0.0, f64::max)
        }
        other => panic!("only a pipe and a cone run on along an axis, not {other:?}"),
    }
}

/// How far along `axis` from its origin a point of this face can be, or `None`
/// when nothing here bounds it.
///
/// The axis coordinate grows steadily over a pipe and over a cone, so it has no
/// high spot in the middle of a face and reaches its own furthest on the
/// **boundary** — which is edges and not corners. A straight edge is the two
/// ends the model already holds. Every other edge is a piece of where this
/// face's surface crosses the one on the other side of it, and either of the two
/// surfaces named along that edge may be the one that bounds it, so the smaller
/// answer is taken.
fn how_far_along_its_axis_this_face_runs(
    face: FaceView<'_>,
    axis: &Axis,
    radius: f64,
    slope: f64,
    along_each_edge: &HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
) -> Option<f64> {
    let straight_on = axis.direction() * axis.direction().length().recip();
    let how_far_along = |point: Point| straight_on.dot(point - axis.origin()).abs();
    let mut furthest: f64 = 0.0;
    for coedge in face.wires().flat_map(WireView::coedges) {
        let edge = coedge.edge();
        if *edge.stored_run() == EdgeRun::Straight {
            furthest = furthest
                .max(how_far_along(edge.start().point()))
                .max(how_far_along(edge.end().point()));
            continue;
        }
        let mut others: SmallVec<[Surface; 3]> = along_each_edge
            .get(&edge.key())
            .map(SmallVec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(SurfaceAlongAnEdge::where_it_really_is)
            .collect();
        // A seam has this face's own surface on both sides of it, and a surface
        // crossed with itself names nothing; the run carries the second surface
        // for exactly that reason.
        if let EdgeRun::PartOfWhereItCrosses { and, .. } = edge.run() {
            others.push(and);
        }
        let bounded = others
            .into_iter()
            .filter_map(|other| {
                how_far_along_an_axis_a_crossing_with_this_surface_runs(
                    axis.origin(),
                    straight_on,
                    radius,
                    slope,
                    &other,
                )
            })
            .fold(f64::INFINITY, f64::min);
        if !bounded.is_finite() {
            return None;
        }
        furthest = furthest.max(bounded);
    }
    Some(furthest)
}

/// How far along `straight_on` from `origin` a point can be that is on both a
/// surface `radius + slope · along` out from that axis and on `other`, or `None`
/// where no such bound exists.
///
/// Write a point of the crossing as `origin + t · straight_on + w`, with `w`
/// square to the axis and `|w| = radius + slope · |t|`. Each arm below puts that
/// into what `other` says about the same point and solves for `|t|`; every
/// inequality in them is a triangle inequality or a Cauchy–Schwarz, so each is a
/// bound and not an estimate. The three that can fail to bound do so exactly
/// when the crossing really does run on for ever — a plane holding the axis
/// direction cuts a pipe in two rulings, and a ruling is straight, which the
/// caller has already dealt with.
fn how_far_along_an_axis_a_crossing_with_this_surface_runs(
    origin: Point,
    straight_on: Vector,
    radius: f64,
    slope: f64,
    other: &Surface,
) -> Option<f64> {
    match other {
        // A ball holds the crossing inside itself, so its own shadow on the axis
        // is the answer and the shape of our surface does not come into it.
        Surface::Sphere(ball) => {
            Some(straight_on.dot(ball.centre() - origin).abs() + ball.radius())
        }
        Surface::Torus(ring) => Some(
            straight_on.dot(ring.centre() - origin).abs() + ring.ring_radius() + ring.tube_radius(),
        ),
        Surface::Plane(flat) => {
            let normal = flat.normal();
            let down_the_axis = normal.dot(straight_on).abs();
            let sideways = (1.0 - down_the_axis * down_the_axis).max(0.0).sqrt();
            let room = down_the_axis - slope * sideways;
            (room > 0.0).then(|| (flat.signed_distance_to(origin).abs() + radius * sideways) / room)
        }
        Surface::Cylinder(pipe) => {
            let (its_origin, its_way) = a_unit_axis(&pipe.axis());
            let room = square_to(straight_on, its_way).length() - slope;
            (room > 0.0).then(|| {
                (pipe.radius() + square_to(origin - its_origin, its_way).length() + radius) / room
            })
        }
        Surface::Cone(cone) => {
            let (apex, its_way) = a_unit_axis(&cone.axis());
            let leaning = square_to(straight_on, its_way).length();
            let room = leaning
                - cone.slope() * straight_on.dot(its_way).abs()
                - cone.slope() * slope * leaning
                - slope;
            (room > 0.0).then(|| {
                let out = origin - apex;
                (cone.slope() * its_way.dot(out).abs()
                    + cone.slope() * radius * leaning
                    + square_to(out, its_way).length()
                    + radius)
                    / room
            })
        }
        // A surface kind this has not been taught bounds nothing, which the
        // caller reads as "the corners are all there is" rather than as a stop:
        // a mesh refused for a shape it could have cut is worse than a floor
        // that is too low on it, and the floor was too low on every one of them
        // before this existed.
        _ => None,
    }
}

/// An axis's origin and its direction scaled to length one.
///
/// A turned axis is no longer a unit direction — a quarter turn about `(1, 2, 3)`
/// takes `(0, 0, 1)` to `(−1.1e−16, 0, 1 + 2ε)` — and every bound above divides
/// by the length of a component of this, so it has to be one.
fn a_unit_axis(axis: &Axis) -> (Point, Vector) {
    (
        axis.origin(),
        axis.direction() * axis.direction().length().recip(),
    )
}

/// The part of `vector` square to the unit direction `way`.
fn square_to(vector: Vector, way: Vector) -> Vector {
    vector - way * way.dot(vector)
}

/// What to hand back when the triangulator refuses a face.
///
/// One of the refusals is not about the request at all. `CannotTellIfLoopsTouch`
/// is an exact predicate that ran out of room, and `Undecidable` at the level of
/// a whole operation is a failure rather than a result (§4.2) — so it goes in the
/// log with its enclosure, where the caller can see which corner and how wide.
/// Every other refusal is about the deflection or about the loops, and neither is
/// a thing a provenance log has anything to say about.
fn refusal_for(face: FaceView<'_>, why: CannotTriangulate, log: &mut ProvenanceLog) -> Error {
    let CannotTriangulate::CannotTellIfLoopsTouch { corner, enclosure } = why else {
        return Error::CannotMeshAFace {
            face: face.key(),
            why,
        };
    };
    let at = face
        .wires()
        .flat_map(WireView::points)
        .nth(corner)
        .unwrap_or_else(Point::origin);
    log.push(Record::new(
        "orient2d",
        &[PredicatePoint::from(at.coordinates())],
        Level::Exact,
        Decision::Undecidable { enclosure },
        Measures::Area,
    ));
    Error::Undecidable {
        records: core::mem::take(log),
    }
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Axis, EdgeRun, Orientation, Plane, Scale, Sphere, Vector};
    use rcad_predicates::{Decision, Interval};

    use crate::validate::a_closed_crossing_is_walked_the_way_it_is_stored;

    use super::*;
    use crate::Model;

    /// A curved face with a hole in it is refused and does not panic.
    ///
    /// **The route is `Deserialize` and not a builder.** No builder here makes
    /// such a face, and the assert this replaces said so and stopped there —
    /// which is `docs/decisions.md` §16.10's mistake exactly: "no caller does
    /// this" is a statement about today's callers, and [`Model`] implements
    /// `Deserialize`, so any shape the arenas can hold arrives from a file
    /// (§16.11 records the same thing about the coordinate range).
    ///
    /// A round bar with a window cut through its wall and a face plugging the
    /// window is a **sound** solid with this shape, so the refusal is about what
    /// the mesher can cut up and not about the model. What is built here is the
    /// window without the plug, which the sweep's manifold check catches before
    /// it meshes anything — so the question is put to [`SolidView::mesh`], which
    /// is public in its own right and is where the panic was.
    #[test]
    fn a_curved_face_with_a_hole_is_refused_rather_than_asserted_against() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = model
            .add_round_bar(5.0, 10.0, &Isometry::identity())
            .expect("a bar five across and ten tall");

        let wall = model
            .data()
            .faces
            .iter()
            .find(|(_, face)| face.surface.as_plane().is_none())
            .map(|(slot, _)| slot)
            .expect("a bar has one curved face");

        // Three corners of a triangle exactly on `x² + y² = 25`, all inside a
        // tenth of a turn of each other.
        let corners = [
            model.add_vertex(Point::new(5.0, 0.0, 3.0)),
            model.add_vertex(Point::new(4.0, 3.0, 5.0)),
            model.add_vertex(Point::new(3.0, 4.0, 7.0)),
        ];
        let coedges: SmallVec<[_; 4]> = (0..3)
            .map(|at| {
                let edge = model.add_edge(corners[at], corners[(at + 1) % 3]);
                model.add_coedge(edge, Orientation::Forward)
            })
            .collect();
        let hole = model.add_wire(coedges);
        model.data_mut().faces[wall].wires.push(hole);

        let refusal = model
            .solid(bar)
            .mesh(
                Deflection::of(1.0).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a curved face with a hole cannot be cut up");
        assert!(
            matches!(refusal, Error::CurvedFaceWithAHoleGoesTooFarRound { .. }),
            "got {refusal:?}"
        );
    }

    /// A wire fanned onto the wrong surface is refused and does not panic.
    ///
    /// `docs/decisions.md` §39.7 item 3's third demonstrated panic: a round
    /// bar's wall face, its own [`rcad_geom::Cylinder`] swapped out for a
    /// [`Plane`] with nothing else changed. The wire round it is unchanged —
    /// walked from the same 3D edges, the same four steps — but a wire that
    /// fans out round a circle in the cylinder's own `(u, v)` collapses into
    /// one corner in the plane's, because every point on the circle sits
    /// directly below the plane's own normal at the corner where the seam
    /// meets it. **The route is `Deserialize` and not a builder**: no builder
    /// here stores a flat surface under a curved wire, but nothing stops
    /// `Model`'s `Deserialize` from holding it, and [`SolidView::mesh`] is
    /// public in its own right.
    #[test]
    fn a_wall_stored_with_the_wrong_surface_is_refused_rather_than_asserted_against() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = model
            .add_round_bar(5.0, 10.0, &Isometry::identity())
            .expect("a bar five across and ten tall");

        let wall = model
            .data()
            .faces
            .iter()
            .find(|(_, face)| face.surface.as_plane().is_none())
            .map(|(slot, _)| slot)
            .expect("a bar has one curved face");

        let flat =
            Plane::from_point_and_normal(Point::new(5.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
                .expect("the x axis is a unit normal");
        model.data_mut().faces[wall].surface = Surface::Plane(flat);

        let refusal = model
            .solid(bar)
            .mesh(
                Deflection::of(1.0).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a wire fanned onto the wrong surface cannot be cut up");
        assert!(
            matches!(refusal, Error::AWireDoesNotFanIntoALoopToMesh { .. }),
            "got {refusal:?}"
        );
    }

    /// An edge with three faces along it is refused and does not panic.
    ///
    /// Where an edge runs is what the surfaces either side of it say, so the
    /// evaluator is handed a pair — and three faces meeting along one edge have
    /// no pair to hand over. **The route is `Deserialize` and not a builder**,
    /// the same as the test above: `check_edges` refuses an edge without exactly
    /// two coedges, but [`SolidView::mesh`] is public in its own right and a
    /// solid that never went through the sweep reaches here first (§16.11).
    ///
    /// One coedge is not the same case and is not refused: an edge used once has
    /// the same face on both sides, which is what a seam is.
    #[test]
    fn an_edge_with_three_faces_along_it_is_refused_rather_than_asserted_against() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = model
            .add_round_bar(5.0, 10.0, &Isometry::identity())
            .expect("a bar five across and ten tall");

        let circle = model
            .data()
            .edges
            .iter()
            .find(|(_, edge)| edge.run == EdgeRun::RightRoundWhereItsFacesCross)
            .map(|(slot, _)| slot)
            .expect("a bar has two cap circles");
        let another = model.add_coedge(circle, Orientation::Forward);
        let somewhere = model
            .data()
            .wires
            .keys()
            .next()
            .expect("a bar has wires to hang it on");
        model.data_mut().wires[somewhere]
            .steps
            .push(crate::model::WireStep::Along(another));

        let refusal = model
            .solid(bar)
            .mesh(
                Deflection::of(1.0).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("three faces along one edge name no pair of surfaces");
        assert!(
            matches!(
                refusal,
                Error::AnEdgeWithoutTwoFacesAlongItCannotBeWalked { .. }
            ),
            "got {refusal:?}"
        );
    }

    /// A wire that will not lie flat is refused and does not panic.
    ///
    /// **The sharpest form of the same finding**, because this one reaches
    /// [`Model::validate`](crate::Model::validate) itself. A ball missing one of
    /// its two pole steps is three wire steps that chain up and close, so
    /// `check_wires` has nothing to say about it, its corners are still exactly
    /// on the sphere, and the sweep walks straight into the mesher — which used
    /// to assert there. `validate` is the untrusted-import boundary (§16.11);
    /// panicking is the one thing it may not do.
    #[test]
    fn a_wire_that_will_not_lie_flat_is_refused_rather_than_asserted_against() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let ball = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
        for wire in model.data_mut().wires.values_mut() {
            wire.steps.pop();
        }

        let refusal = model
            .solid(ball)
            .mesh(
                Deflection::of(1.0).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a ball with one pole is not a rectangle in its own coordinates");
        assert!(matches!(
            refusal,
            Error::AWireWillNotLieFlatOnItsFace { .. }
        ));

        assert!(matches!(
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect_err("the sweep meshes every curved solid"),
            Error::AWireWillNotLieFlatOnItsFace { .. }
        ));
    }

    /// A pole met between two different meridians is a whole side of the patch
    /// and gets a corner at each end of it.
    ///
    /// Two cuts that meet a ball at its pole leave it an ordinary corner of an
    /// ordinary wire, and the walk arrives along one meridian and leaves along
    /// another. Putting one corner there instead of two leaves the loop running
    /// diagonally across the chart from the longitude it came in on to the one
    /// it goes out on, over a region the face does not cover.
    #[test]
    fn a_pole_between_two_meridians_gets_a_corner_at_each_end_of_it() {
        let whole = 2.0 * core::f64::consts::PI;
        let mut walk = WalkingAWire::new(Some(whole), false);
        walk.arrive(0, (core::f64::consts::FRAC_PI_2, -0.5), false);
        walk.arrive(1, (0.0, -core::f64::consts::FRAC_PI_2), true);
        walk.arrive(2, (core::f64::consts::PI, 0.0), false);

        let walked = walk.finish().expect("this wire lies flat");
        assert_eq!(
            walked.corners,
            vec![
                [core::f64::consts::FRAC_PI_2, -0.5],
                [core::f64::consts::FRAC_PI_2, -core::f64::consts::FRAC_PI_2],
                [core::f64::consts::PI, -core::f64::consts::FRAC_PI_2],
                [core::f64::consts::PI, 0.0],
            ]
        );
        assert_eq!(
            walked.points,
            vec![0, 1, 1, 2],
            "the pole is one point twice"
        );
    }

    /// A walk that ends at a pole leaves it by the longitude it set off from,
    /// because that is where the loop closes.
    #[test]
    fn a_walk_that_ends_at_a_pole_closes_the_side_back_to_where_it_started() {
        let whole = 2.0 * core::f64::consts::PI;
        let mut walk = WalkingAWire::new(Some(whole), false);
        walk.arrive(0, (core::f64::consts::FRAC_PI_2, -0.5), false);
        walk.arrive(1, (0.0, -0.9), false);
        walk.arrive(2, (0.0, -core::f64::consts::FRAC_PI_2), true);

        let walked = walk.finish().expect("this wire lies flat");
        assert_eq!(
            walked.corners,
            vec![
                [core::f64::consts::FRAC_PI_2, -0.5],
                [0.0, -0.9],
                [0.0, -core::f64::consts::FRAC_PI_2],
                [core::f64::consts::FRAC_PI_2, -core::f64::consts::FRAC_PI_2],
            ]
        );
        assert_eq!(
            walked.points,
            vec![0, 1, 2, 2],
            "the pole is one point twice"
        );
    }

    /// A ball, with its seam moved to whichever half of whichever meridian.
    ///
    /// Every one of these is a **sound** ball: its two poles lie on any plane
    /// through the axis and outside neither half-space, so nothing
    /// [`Model::validate`](crate::Model::validate) checks about an edge can tell
    /// them apart. Only the mesher can, and until M3's gate it did not look.
    fn a_ball_seamed_along(normal: Vector, picked_by_facing: Vector) -> Model {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        model.add_ball(2.0, &Isometry::identity()).expect("a ball");

        let meridian = Plane::from_point_and_normal(Point::origin(), normal).expect("a normal");
        let picker =
            rcad_geom::HalfSpace::through_a_point_facing(Point::origin(), picked_by_facing)
                .expect("a normal");
        let edge = model.data().edges.keys().next().expect("the seam");
        model.data_mut().edges[edge].run = EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(meridian),
            picked_by: SmallVec::from_buf([rcad_geom::Picker::from(picker)]),
        };
        model
    }

    fn the_sphere_of(model: &Model) -> Surface {
        model
            .data()
            .faces
            .values()
            .map(|face| face.surface)
            .next()
            .expect("a ball has one face")
    }

    /// The seam is walked down the meridian the model records, not down
    /// longitude zero.
    ///
    /// Both ends of a pole-to-pole seam are points every longitude names, so the
    /// run is the only thing that says which meridian it is — the very field
    /// §24.4 item 2 widened. **This used to be read here and is now read in
    /// `rcad-geom`** (§27.5): the run's `and` is the plane the circle lies in and
    /// `picked_by` says which of the two arcs between the poles this is, and
    /// `walk` asks both. What this checks is the answer, off the points
    /// themselves.
    #[test]
    fn a_seam_is_walked_down_the_meridian_its_own_run_names() {
        let quarter = core::f64::consts::FRAC_PI_2;
        for (normal, facing, expected) in [
            (Vector::new(0.0, 1.0, 0.0), Vector::new(1.0, 0.0, 0.0), 0.0),
            (
                Vector::new(0.0, 1.0, 0.0),
                Vector::new(-1.0, 0.0, 0.0),
                2.0 * quarter,
            ),
            (
                Vector::new(1.0, 0.0, 0.0),
                Vector::new(0.0, 1.0, 0.0),
                quarter,
            ),
            (
                Vector::new(1.0, 0.0, 0.0),
                Vector::new(0.0, -1.0, 0.0),
                -quarter,
            ),
        ] {
            let model = a_ball_seamed_along(normal, facing);
            let stored = the_sphere_of(&model);
            let mesh = model
                .solids()
                .next()
                .expect("the ball")
                .mesh(
                    Deflection::of(0.1).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a ball seamed anywhere is a ball");

            let mut on_the_seam = 0;
            for point in mesh.points() {
                let (longitude, latitude) = stored.coordinates_of(*point);
                if latitude.abs() > 1.5 {
                    continue;
                }
                let off = the_short_way_round(longitude - expected, core::f64::consts::TAU);
                if off.abs() < 1e-12 {
                    on_the_seam += 1;
                }
            }
            assert!(
                on_the_seam >= 2,
                "a seam on the plane {normal:?} picked by {facing:?} put only \
                 {on_the_seam} points on longitude {expected}"
            );
        }
    }

    /// Everything the mesher would have to hand an evaluator, said in the
    /// vocabulary that evaluator takes.
    ///
    /// M3's first crossing is the first edge this mesher cannot walk: it steps
    /// in a straight line through the surface's own two coordinates, which is
    /// right for a ruling, a cap circle and a meridian and wrong for the ellipse
    /// a tilted plane cuts out of a pipe (§26.6 item 1). §27.5 settles that the
    /// evaluator in `rcad-geom` walks every edge and this layer walks none — so
    /// what crosses that seam is a [`NamedRun`], the two surfaces either side of
    /// the edge, and a deflection, and what comes back is 3D points.
    ///
    /// Nothing here builds an evaluator. What this says is that the half of the
    /// seam this crate owns can already be spoken: every edge of every solid the
    /// kernel builds turns into a `NamedRun` with nothing translated, and the
    /// closed-run invariant `NamedRun::new` asserts is one the model keeps.
    #[test]
    fn every_edge_the_kernel_builds_is_already_a_named_run() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        model
            .add_round_bar(3.0, 5.0, &Isometry::identity())
            .expect("a bar");
        model.add_ball(2.0, &Isometry::identity()).expect("a ball");
        model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube");

        let mut closed = 0;
        for slot in model.data().edges.keys() {
            let edge = EdgeView::new(&model, slot, Isometry::identity(), Orientation::Forward);
            let (from, to) = ends_in_the_stored_order(edge);
            let run = NamedRun::new(
                edge.stored_run().clone(),
                [from, to].map(|end| PredicatePoint::from(end.point().coordinates())),
            )
            .expect("every edge the kernel builds itself comes back to where it started");
            if run.run() == &EdgeRun::RightRoundWhereItsFacesCross {
                closed += 1;
                assert_eq!(
                    run.ends()[0],
                    run.ends()[1],
                    "a closed edge of the model comes back to the corner it left"
                );
            }
        }
        assert_eq!(closed, 2, "only a bar's two cap circles are closed edges");
    }

    /// The other half of the same seam: the evaluator answers about every one of
    /// those runs, and the points it hands back are on both faces.
    ///
    /// The gate checked the types could say it; this checks the arithmetic
    /// behind it answers. **The mesher still does not call `walk`** — swapping
    /// it over changes how many points every curved mesh gets and moves the pole
    /// and seam handling down a layer, which belongs with the milestone's
    /// closing gate rather than in the middle of its first stage (§28). What
    /// this says is that the swap is a wiring job and not a discovery: every
    /// curved edge the kernel builds already crosses the seam and comes back
    /// with points on the surfaces the mesher would have put them on.
    #[test]
    fn every_curved_edge_the_kernel_builds_is_walked_into_points_on_both_its_faces() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let somewhere = Isometry::translation(Vector::new(4.0, -2.0, 1.0)).expect("a move");
        model.add_round_bar(3.0, 5.0, &somewhere).expect("a bar");
        model.add_ball(2.0, &somewhere).expect("a ball");
        model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube");

        let mut curved = 0;
        let mut straight = 0;
        for slot in model.data().edges.keys() {
            let edge = EdgeView::new(&model, slot, Isometry::identity(), Orientation::Forward);
            let (from, to) = ends_in_the_stored_order(edge);
            let run = NamedRun::new(
                edge.stored_run().clone(),
                [from, to].map(|end| PredicatePoint::from(end.point().coordinates())),
            )
            .expect("every edge the kernel builds itself comes back to where it started");
            let faces: Vec<Surface> = model
                .faces_along_edge(slot)
                .map(|face| model.data().faces[face].surface)
                .collect();
            let along = match faces.as_slice() {
                // A seam has the same face both sides, listed once.
                [only] => [*only, *only],
                [first, second] => [*first, *second],
                other => panic!("an edge has one face or two along it, not {}", other.len()),
            };

            let along_it = rcad_geom::walk(
                &run,
                [&along[0], &along[1]],
                Deflection::of(0.01).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| {
                panic!("this edge came out of a model and would not walk: {why}")
            });

            if run.run() == &EdgeRun::Straight {
                straight += 1;
                assert!(along_it.points().is_empty(), "a line needs no filling in");
                assert_eq!(along_it.deviation().magnitude(), 0.0);
                continue;
            }

            curved += 1;
            assert!(
                along_it.points().len() > 8,
                "a curved edge of a shape this size takes more than eight pieces"
            );
            let slack = along_it.deviation().magnitude();
            let has_to_be_on: Vec<Surface> = match run.run() {
                EdgeRun::PartOfWhereItCrosses { and, .. } if along[0] == along[1] => {
                    vec![along[0], *and]
                }
                EdgeRun::Straight
                | EdgeRun::RightRoundWhereItsFacesCross
                | EdgeRun::PartOfWhereItCrosses { .. }
                | _ => vec![along[0], along[1]],
            };
            for &point in along_it.points() {
                let at = PredicatePoint::Constructed(point.coordinates().map(|coordinate| {
                    Interval::new(coordinate - slack, coordinate + slack)
                        .expect("a bound is a length")
                }));
                for surface in &has_to_be_on {
                    let decision = side_of_surface(surface, at);
                    assert!(
                        !matches!(decision, Decision::Pos | Decision::Neg),
                        "this point is on {surface:?} and came back {decision:?}: {point:?}"
                    );
                }
            }
        }
        assert_eq!(
            curved, 3,
            "a bar's two cap circles and a ball's seam; a bar's own seam is a straight ruling"
        );
        assert_eq!(straight, 13, "that ruling and a cube's twelve edges");
    }

    /// A ball seamed down the far half of its meridian is sound and used to mesh
    /// into nothing at all.
    ///
    /// The wire of a ball opens at a pole, and a pole names every longitude —
    /// so a walk that let the pole set its starting longitude set off from
    /// `atan2(0, 0)` and unwrapped every corner after it towards longitude zero.
    /// Half a turn away that is exactly the boundary
    /// [`WalkingAWire::finish`] refuses at, and a perfectly good ball came back
    /// [`Error::AWireWillNotLieFlatOnItsFace`].
    #[test]
    fn a_ball_seamed_down_the_far_meridian_still_meshes_into_a_ball() {
        let model = a_ball_seamed_along(Vector::new(0.0, 1.0, 0.0), Vector::new(-1.0, 0.0, 0.0));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a ball seamed down its far meridian is a sound ball");

        let solid = model.solids().next().expect("the ball").key();
        let mesh = model
            .solid(solid)
            .mesh(
                Deflection::of(0.1).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a sound ball meshes whichever meridian it is cut along");

        assert!(!mesh.triangles().is_empty());
        for point in mesh.points() {
            assert!(
                (point.distance_to(Point::origin()) - 2.0).abs() < 1e-9,
                "{point:?} is not on the ball"
            );
        }
    }

    /// A seam whose half-spaces pick neither half, or both, is refused.
    ///
    /// `validate` cannot catch this and never could: both ends of a pole-to-pole
    /// seam satisfy every half-space its true meridian satisfies, so
    /// `check_edges` has nothing to look at. Cutting at longitude zero and
    /// carrying on is what used to happen, and it is a confident wrong answer.
    ///
    /// **The refusal now comes from the evaluator and says so.** It is the same
    /// question — which of the two arcs between the same two ends is this — asked
    /// where the arcs are worked out rather than one layer up (§27.5).
    #[test]
    fn a_seam_that_picks_neither_half_of_its_meridian_or_both_is_refused() {
        for facing in [Vector::new(0.0, 1.0, 0.0), Vector::new(0.0, -1.0, 0.0)] {
            let model = a_ball_seamed_along(Vector::new(0.0, 1.0, 0.0), facing);
            let refusal = model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect_err("the meridian plane's own normal picks neither half of it");
            assert!(
                matches!(
                    refusal,
                    Error::Geometry(rcad_geom::Error::CannotNameWhichPieceOfACrossingThisIs)
                ),
                "a seam nothing picks came back {refusal:?}"
            );
        }
    }

    /// **The half of §51.7 the public suite cannot see: what the *wall* beside
    /// an emptied closed edge does.**
    ///
    /// `tests/what_the_mesher_does_with_an_empty_walk_on_a_closed_edge.rs`
    /// builds the same bar and puts the same question to [`SolidView::mesh`],
    /// and what comes back is the **cap's** refusal — one closed edge is the
    /// whole of a cap's wire, so its loop has one corner and
    /// [`Loop::through`](rcad_mesh::Loop::through) turns it away. A solid's
    /// faces are cut up in stored order and a bar's caps come first, so the wall
    /// is never reached from up there.
    ///
    /// The wall's wire is **not** one corner — it is the bottom circle's points,
    /// the seam corner twice and the top corner twice — so nothing about a count
    /// of three covers it, and whether a rectangle whose top side collapsed to
    /// one point is quietly filled with triangles has to be asked of
    /// [`BuildingAMesh::cut_up`] directly. It is not: the walk goes right round
    /// the bottom, up the seam, and then has only the one top corner to come
    /// back along, so it never returns to the longitude it set off from and
    /// [`WalkingAWire::finish`] refuses it.
    ///
    /// So **both** faces along an emptied closed edge refuse, by two different
    /// names, and neither of them meshes anything.
    ///
    /// It also pins the thing the public test can only infer: the list really is
    /// empty, and it is empty for exactly one of the bar's two closed edges. A
    /// run where both walked to nothing, or neither did, is a run that proves
    /// nothing.
    ///
    /// **The deflection is handed to [`BuildingAMesh`] and not asked of
    /// [`SolidView::room_a_mesh_has_to_work_in`].** This is about what `cut_up`
    /// does with an emptied list, and the floor is a different question that is
    /// asked in its own file. It used to come from the floor, and once the floor
    /// learned that a leaning cut makes a face big it refuses this deflection
    /// outright — an honest refusal, and one that would leave this test
    /// measuring the floor rather than `cut_up`.
    #[test]
    fn the_wall_beside_a_closed_edge_that_walked_to_nothing_is_refused_as_well() {
        const LEAN: f64 = 2e9;
        let policy = Policy::tolerant(50.0 * LEAN);
        let mut log = ProvenanceLog::new();
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
            .expect("the z axis is a unit normal");
        let leaning =
            Plane::from_point_and_normal(Point::new(1.0, 0.0, 0.5), Vector::new(LEAN, 0.0, 1.0))
                .expect("a direction with length");
        let bar = model
            .add_bar_cut_off_by(
                1.0,
                [flat, leaning],
                &Isometry::identity(),
                &policy,
                &mut log,
            )
            .expect("a leaning plane cuts an ellipse right round a pipe");

        let solid = model.solid(bar);
        let deflection = Deflection::of(5e-4).expect("a positive length");
        let mut building = BuildingAMesh::new(deflection, solid.surfaces_along_each_edge());
        for face in solid.faces() {
            building
                .walk_the_edges_this_face_can_place(face, &policy, &mut log)
                .expect("a tolerant caller is handed an empty list rather than refused");
        }

        let mut walked_to_nothing = 0;
        let mut walked_into_points = 0;
        for edge in solid
            .faces()
            .flat_map(FaceView::wires)
            .flat_map(WireView::coedges)
            .map(|coedge| coedge.edge())
            .filter(|edge| edge.is_closed().expect("a cap circle knows it is closed"))
        {
            if building.along_edge[&edge.key()].is_empty() {
                walked_to_nothing += 1;
            } else {
                walked_into_points += 1;
            }
        }
        assert!(
            walked_to_nothing > 0 && walked_into_points > 0,
            "{walked_to_nothing} of this bar's closed edges walked to nothing and \
             {walked_into_points} walked into points, and the test needs one of each"
        );

        let wall = solid
            .faces()
            .find(|face| face.surface().as_plane().is_none())
            .expect("a bar has one curved face");
        let refusal = building
            .cut_up(wall, &mut log)
            .expect_err("a wall whose top side collapsed to one point is not a patch");
        assert!(
            matches!(refusal, Error::AWireWillNotLieFlatOnItsFace { .. }),
            "the wall beside an emptied closed edge came back {refusal:?}"
        );
    }

    /// The other way a wire will not lie flat: a pole on a face that does not
    /// pinch to one.
    ///
    /// A plane never does. Tacked onto a flat face's wire at a corner it already
    /// has, a pole step chains up, closes, and leaves the winding check reading
    /// exactly the corners it read before — so again nothing earlier in the
    /// sweep looks at it.
    #[test]
    fn a_pole_on_a_flat_face_is_refused_rather_than_asserted_against() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = model
            .add_round_bar(5.0, 10.0, &Isometry::identity())
            .expect("a bar");
        let cap = model
            .data()
            .faces
            .iter()
            .find(|(_, face)| face.surface.as_plane().is_some())
            .map(|(slot, _)| slot)
            .expect("a bar has two flat caps");
        let wire = model.data().faces[cap].wires[0];
        let corner = model.data().edges[model.data().coedges
            [model.data().wires[wire].coedges().next().expect("a coedge")]
        .edge]
            .start;
        model.data_mut().wires[wire]
            .steps
            .push(crate::model::WireStep::Pole(corner));

        let refusal = model
            .solid(bar)
            .mesh(
                Deflection::of(1.0).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a plane pinches to nothing");
        assert!(matches!(
            refusal,
            Error::AWireWillNotLieFlatOnItsFace { .. }
        ));
    }

    /// A placed face is meshed on the surface the arena holds, read out of it
    /// rather than moved out by the placement and straight back in again.
    ///
    /// **The round trip was a gap and not a limit.** [`FaceView::surface`] moves
    /// the stored surface out by the face's placement, and
    /// [`the_frame_this_face_is_meshed_in`] for a placed face is that placement's
    /// inverse — so the mesher moved it straight back. Both movements round by
    /// [`Isometry::POINT_ROUNDOFF`], and what came back was a surface a few ulps
    /// off the one the arena holds. Every mesh point of that face was put on the
    /// wrong one of the two.
    ///
    /// **Nothing charged for it.** The floor under a deflection charges
    /// [`how_far_placing_this_face_rounds`] for carrying a mesh point back out to
    /// where the solid sits, and that is one movement of a *point*, worth
    /// `Isometry::POINT_ROUNDOFF · L`. The surface having moved under the point
    /// is a second error of the same kind, worth up to twice that because it is
    /// two movements, and no term covered it — so [`Mesh::deviation`] was
    /// understating what a mesh point costs. §41.8 item 1, §41.11 item 19.
    ///
    /// Reading the stored surface makes that stray **exactly zero**, so the
    /// claim here is an equality and not a bound: there is no constant to
    /// publish.
    ///
    /// **Two roads that must differ.** An equality that holds because both sides
    /// walked the same road proves nothing, so this also measures the road not
    /// taken and fails if the round trip never moves a surface at all. And it
    /// checks the stray against the ceiling the derivation gives it, because a
    /// stray past that would mean the derivation is wrong about what a round
    /// trip costs, however dead the code that makes one now is.
    ///
    /// **Each surface is charged its own ceiling and not the loosest one.**
    /// [`the_surface_this_face_is_meshed_on`] derives two different numbers:
    /// two movements of a point for a ball, `36 · f64::EPSILON · L`, because a
    /// ball is placed by its centre alone, and four for a pipe,
    /// `72 · f64::EPSILON · L`, because a pipe is placed by a point *and* a
    /// direction. Holding a ball to the pipe's number would leave half the
    /// derivation untested, so a ball here is held to 36.
    ///
    /// **A ball the mesher can reach has to be built by hand to move at all.**
    /// [`Model::add_ball`] stores its sphere on the origin of the frame the
    /// placement moves, and `M⁻¹(M(origin))` cancels to the bit whatever the
    /// placement is — so every ball *face* below strays exactly zero, and a
    /// count that lumped balls in with pipes would have said the road not taken
    /// was walked when for balls it never was. The ball's own ceiling is
    /// exercised by round-tripping a sphere stored **away** from that origin,
    /// which [`Builder::add_face`](crate::Builder::add_face) and a STEP import
    /// both accept.
    ///
    /// **The stray is not measured by sampling** and
    /// [`how_far_apart`](tests::how_far_apart) says why.
    #[test]
    fn a_turned_curved_face_is_not_charged_for_a_round_trip_it_does_not_make() {
        let leaning = [
            Vector::new(1.0, 2.0, 3.0),
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(1.0, -1.0, 0.0),
            Vector::new(7.0, 1.0, -3.0),
        ]
        .map(|way| Axis::new(Point::origin(), way).expect("a direction with length"));
        let charged = Isometry::POINT_ROUNDOFF / f64::EPSILON;
        let ceilings = [("a ball", 2.0 * charged), ("a pipe", 4.0 * charged)];
        let mut compared = 0_u32;
        let mut moved = [0_u32; 2];
        let mut worst = [0.0_f64; 2];
        let mut ball_faces_that_moved = 0_u32;

        for axis in &leaning {
            for sixteenths in 1..16 {
                for away in [0.0, 1.0, 1000.0, 1e6] {
                    let turn = Isometry::rotation(
                        axis,
                        f64::from(sixteenths) * core::f64::consts::TAU / 16.0,
                    )
                    .expect("an axis and an angle");
                    let placement = Isometry::translation(Vector::new(away, -away, away))
                        .expect("a finite offset")
                        .compose(&turn);

                    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                    let ball = model.add_ball(2.0, &placement).expect("a ball");
                    let bar = model.add_round_bar(3.0, 7.0, &placement).expect("a bar");
                    assert_eq!(
                        model.solid(ball).faces().count(),
                        1,
                        "a ball whose one face is its sphere is what makes the two-movement \
                         ceiling below the right one to charge it"
                    );

                    for (kind, solid) in [ball, bar].into_iter().enumerate() {
                        for face in model.solid(solid).faces() {
                            if face.surface().as_plane().is_some() {
                                continue;
                            }
                            assert_ne!(
                                face.location().rotation_quaternion(),
                                Isometry::identity().rotation_quaternion(),
                                "a placement that does not turn is not what this is about"
                            );
                            compared += 1;
                            assert_eq!(
                                the_surface_this_face_is_meshed_on(face),
                                face.stored_surface(),
                                "a face placed by {placement:?} is meshed on a surface \
                             that is not the one the arena holds"
                            );

                            let round_tripped = face
                                .surface()
                                .transformed(&the_frame_this_face_is_meshed_in(face));
                            let reach = how_far_the_points_of_reach(
                                face,
                                &the_frame_this_face_is_meshed_in(face),
                                &model.solid(solid).surfaces_along_each_edge(),
                            );
                            let stray =
                                how_far_apart(&face.stored_surface(), &round_tripped, reach);
                            if stray > 0.0 {
                                moved[kind] += 1;
                                if kind == 0 {
                                    ball_faces_that_moved += 1;
                                }
                            }
                            worst[kind] = worst[kind].max(stray / (f64::EPSILON * reach));
                        }
                    }

                    // `add_ball` stores its sphere on the origin of the frame
                    // the placement moves, and `M⁻¹(M(origin))` cancels to the
                    // bit whatever the placement is — so the ball faces above
                    // never move and never exercise a ball's own ceiling. A
                    // sphere stored away from that origin does move, and both
                    // `Builder::add_face` and a STEP import accept one, so it
                    // is not a shape nothing can build.
                    let off_centre = Surface::Sphere(
                        Sphere::new(Point::new(0.7, -1.3, 2.9), 2.0).expect("a positive radius"),
                    );
                    let there_and_back = off_centre
                        .transformed(&placement)
                        .transformed(&placement.inverse());
                    let reach = 2.9 + away;
                    let stray = how_far_apart(&off_centre, &there_and_back, reach);
                    if stray > 0.0 {
                        moved[0] += 1;
                    }
                    worst[0] = worst[0].max(stray / (f64::EPSILON * reach));
                }
            }
        }

        assert!(compared >= 400, "only {compared} placed curved faces");
        assert_eq!(
            ball_faces_that_moved, 0,
            "a ball face's round trip moved its sphere, which the note above says cannot \
             happen while add_ball stores that sphere on its placement's own origin — so \
             either the note is wrong now or the ball entries below are counting the wrong \
             thing"
        );
        for (kind, (surface, ceiling)) in ceilings.into_iter().enumerate() {
            println!(
                "{compared} placed curved faces, every one meshed on the surface the arena \
                 holds. Round trips on {surface}: {} moved it, and the worst strayed {} · \
                 f64::EPSILON · L against a ceiling of {ceiling}, with {charged} charged \
                 for placing a mesh point",
                moved[kind], worst[kind],
            );
            assert!(
                moved[kind] > 0,
                "the round trip moved no surface of {surface} at all, so the equality \
                 above is comparing a road with itself and proves nothing"
            );
            assert!(
                worst[kind] <= ceiling,
                "a round trip on {surface} strayed {} · f64::EPSILON · L, past the \
                 ceiling of {ceiling} the derivation gives it",
                worst[kind]
            );
        }
    }

    /// How far apart two surfaces of the same kind sit in space, out to `reach`
    /// from the point that places them.
    ///
    /// **Read off their own numbers and not sampled, and the first draft of this
    /// was sampled and wrong.** A pipe picks the two directions across itself
    /// from its axis, so a tilt of a tenth of an ulp can move where `u = 0`
    /// sits by a lot; two points at the same `(u, v)` then lie far apart while
    /// the two surfaces lie on top of each other. Sampled that way, a bar of
    /// radius 3 came out **4.2** from itself. That is the parametrisation
    /// moving, not the surface.
    ///
    /// Moving the placing point moves every point of the surface by as much.
    /// Tilting the axis moves a point `reach` away by `reach` times the tilt,
    /// and at these sizes the chord between the two directions is the tilt. A
    /// radius that differs moves the skin by the difference.
    fn how_far_apart(one: &Surface, other: &Surface, reach: f64) -> f64 {
        match (one, other) {
            (Surface::Sphere(here), Surface::Sphere(there)) => {
                here.centre().distance_to(there.centre()) + (here.radius() - there.radius()).abs()
            }
            (Surface::Cylinder(here), Surface::Cylinder(there)) => {
                here.axis().origin().distance_to(there.axis().origin())
                    + reach * (here.axis().direction() - there.axis().direction()).length()
                    + (here.radius() - there.radius()).abs()
            }
            (here, there) => panic!("nothing here compares {here:?} against {there:?}"),
        }
    }

    /// §16.10 for [`how_far_one_coordinate_can_be_shifted_for_nothing`]: the
    /// shift the mesher works a far-off face out in really does lose nothing.
    ///
    /// **The whole of the frame change rests on this.** A shift that lost a bit
    /// would move the surface the points are put on, and the floor under a
    /// deflection would be understating what a mesh point costs — which is the
    /// defect this frame change exists to fix, arriving by a different road.
    ///
    /// The check is exact and not a tolerance: Knuth's two-sum hands back what an
    /// addition threw away, to the last bit, and the claim is that it is zero.
    /// Searched over numbers sixty octaves apart, both signs, with the runs that
    /// are *refused* a shift counted too — a rule that shifted nothing at all
    /// would pass this whatever it said.
    #[test]
    fn taking_the_smallest_of_a_run_of_numbers_off_all_of_them_loses_no_bits() {
        let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next_bits = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        // A whole fifty-three bits of mantissa, because a number with only
        // twenty-four significant bits subtracts exactly from its neighbours
        // whatever the rule says, and a search over those proves nothing.
        let a_fraction = |bits: u64| (bits >> 11) as f64 / (1u64 << 53) as f64;
        let a_number = |bits: u64| {
            let octave = ((bits >> 20) % 60) as i32 - 30;
            let sign = if bits & 1 == 0 { 1.0 } else { -1.0 };
            sign * (1.0 + a_fraction(bits)) * 2.0f64.powi(octave)
        };

        let (mut shifted, mut left_alone) = (0, 0);
        for _ in 0..20_000 {
            // A place to draw the run around, and how far it spreads. Some
            // spreads are inside a factor of two of the base and some are not,
            // and the rule has to be right about both.
            let base = a_number(next_bits());
            let spread = 5.0 * a_fraction(next_bits());
            let run: [f64; 8] =
                [0, 1, 2, 3, 4, 5, 6, 7].map(|step| base * (1.0 + spread * f64::from(step) / 7.0));
            let low = run.into_iter().fold(f64::INFINITY, f64::min);
            let high = run.into_iter().fold(f64::NEG_INFINITY, f64::max);

            let by = how_far_one_coordinate_can_be_shifted_for_nothing(low, high);
            if by == 0.0 {
                left_alone += 1;
                continue;
            }
            shifted += 1;
            for value in run {
                let lost = what_an_addition_lost(value, -by);
                assert_eq!(
                    lost, 0.0,
                    "taking {by} off {value} out of a run over {low}..={high} \
                     lost {lost}"
                );
            }
        }

        println!("runs shifted {shifted}, runs left alone {left_alone}");
        assert!(
            shifted > 0 && left_alone > 0,
            "a search where nothing was shifted, or where everything was, is \
             not testing the rule: {shifted} shifted and {left_alone} left alone"
        );
    }

    /// What an addition threw away, exactly — Knuth's two-sum, which holds for
    /// any two `f64` whatever their sizes.
    fn what_an_addition_lost(x: f64, y: f64) -> f64 {
        let sum = x + y;
        let x_part = sum - y;
        (x - x_part) + (y - (sum - x_part))
    }

    /// [`Mesh::points_along_edge`] hands back an **open** edge's points running
    /// from the end the edge is stored starting at, whichever way round the
    /// caller's use of it runs.
    ///
    /// A ball's seam is the one edge here that is used both ways round by the
    /// same face, so a list handed out in the direction of the *use* would have
    /// to be two different lists — and both uses read this one. Where the points
    /// sit is [`rcad_geom::walk`]'s business and
    /// [`every_curved_edge_the_kernel_builds_is_walked_into_points_on_both_its_faces`]
    /// is what checks it; the claim here is which list they come back in and in
    /// what order.
    #[test]
    fn the_points_along_an_open_edge_run_from_the_end_it_is_stored_starting_at() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let ball = model.add_ball(1.0, &Isometry::identity()).expect("a ball");
        let mesh = model
            .solid(ball)
            .mesh(
                Deflection::of(0.05).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a ball meshes");

        let uses = every_use_of_an_edge_of(model.solid(ball));
        assert_eq!(
            uses.len(),
            2,
            "a ball's one seam, used twice by its one face"
        );
        assert!(
            uses[0].orientation().is_reversed() != uses[1].orientation().is_reversed(),
            "the two uses of a seam run opposite ways round it"
        );
        let ends = [
            ends_in_the_stored_order(uses[0]),
            ends_in_the_stored_order(uses[1]),
        ];
        assert_eq!(
            (ends[0].0.key(), ends[0].1.key()),
            (ends[1].0.key(), ends[1].1.key()),
            "both uses of an edge agree on which way round it is stored"
        );
        let (from, to) = ends[0];
        let height_of_the_end = |end: VertexView<'_>| end.point().coordinates()[2];
        assert!(
            height_of_the_end(from) != height_of_the_end(to),
            "a seam runs between the two poles"
        );

        let along = mesh.points_along_edge(uses[0].key());
        assert_eq!(
            along,
            mesh.points_along_edge(uses[1].key()),
            "one list, read by both uses"
        );
        assert!(
            along.len() > 4,
            "half a meridian of a ball cut to a twentieth of its radius takes more \
             than four pieces, and got {}",
            along.len()
        );

        let height = |at: u32| mesh.points()[at as usize].coordinates()[2];
        let downwards = height_of_the_end(from) > height_of_the_end(to);
        let mut last = height_of_the_end(from);
        for &at in along {
            let point = mesh.points()[at as usize];
            assert_eq!(
                point.coordinates()[1],
                0.0,
                "the seam is the half of the meridian circle in the plane y = 0"
            );
            assert!(
                point.coordinates()[0] > 0.0,
                "and it is the half with x >= 0, not the far one: {point:?}"
            );
            assert_eq!(
                height(at) < last,
                downwards,
                "the points run from the end the edge is stored starting at towards \
                 the other, and this one turned back at {:?}",
                point
            );
            last = height(at);
        }
        assert_eq!(
            last > height_of_the_end(to),
            downwards,
            "the last point stops short of the end the edge is stored running to"
        );

        for end in [from, to] {
            let at = mesh
                .point_at_vertex(end.key())
                .expect("every corner of the meshed solid has a point");
            assert!(
                !along.contains(&at),
                "the two ends are corners of the model and are not in the list"
            );
        }
    }

    /// A **closed** edge is stored running the way the longitude of the curved
    /// surface along it increases, and the face that reads it backwards is what
    /// reverses the list rather than getting a reversed one.
    ///
    /// A round bar's bottom cap circle is used forwards by the wall and
    /// backwards by the cap. Both read this one list, so a list handed out in
    /// the direction of the use would have to disagree with itself.
    #[test]
    fn a_cap_circle_runs_the_way_the_wall_round_it_turns_and_not_the_way_the_cap_reads_it() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = model
            .add_round_bar(5.0, 10.0, &Isometry::identity())
            .expect("a bar five across and ten tall");
        let mesh = model
            .solid(bar)
            .mesh(
                Deflection::of(0.5).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a bar meshes");

        let bottom: Vec<EdgeView<'_>> = every_use_of_an_edge_of(model.solid(bar))
            .into_iter()
            .filter(|edge| {
                edge.is_closed().expect("a bar's edges are all readable")
                    && edge.start().point().coordinates()[2] == 0.0
            })
            .collect();
        assert_eq!(
            bottom.len(),
            2,
            "the bottom cap circle, used by the wall and by the cap"
        );
        assert!(
            bottom[0].orientation().is_reversed() != bottom[1].orientation().is_reversed(),
            "the cap reads the circle the other way round from the wall"
        );
        assert_eq!(bottom[0].key(), bottom[1].key(), "and it is one edge");

        let along = mesh.points_along_edge(bottom[0].key());
        assert!(
            along.len() > 8,
            "a circle five across cut to half a millimetre takes more than eight \
             pieces, and got {}",
            along.len()
        );

        // The one corner of the circle sits at `x = radius, y = 0`, so it is at
        // longitude zero and every point after it is further round.
        let mut last = 0.0;
        for &at in along {
            let point = mesh.points()[at as usize];
            let [x, y, z] = point.coordinates();
            assert_eq!(z, 0.0, "the bottom circle lies in the bottom cap's plane");
            let longitude = y.atan2(x).rem_euclid(std::f64::consts::TAU);
            assert!(
                longitude > last,
                "the points run the way the wall's longitude increases, and this one \
                 went back from {last} to {longitude}"
            );
            last = longitude;
        }
        assert!(
            last < std::f64::consts::TAU,
            "and they stop short of coming back to the corner"
        );

        let corner = mesh
            .point_at_vertex(bottom[0].start().key())
            .expect("every corner of the meshed solid has a point");
        assert!(
            !along.contains(&corner),
            "a closed edge's one corner goes at both ends and is not in the list"
        );
    }

    /// Every point of a wholly flat solid's mesh is a corner it already held, so
    /// no edge of it has a point of its own — and a corner of a solid this mesh
    /// was not cut from has none at all.
    #[test]
    fn a_cube_is_its_eight_corners_and_no_edge_of_it_carries_a_point() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let cube = model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube");
        let another = model
            .add_cube(3.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("another cube");
        let mesh = model
            .solid(cube)
            .mesh(
                Deflection::of(0.5).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a cube meshes");

        assert_eq!(mesh.points().len(), 8, "a cube is its own eight corners");
        let mut seen = HashSet::new();
        for corner in every_corner_of(model.solid(cube)) {
            let at = mesh
                .point_at_vertex(corner.key())
                .expect("every corner of the meshed solid has a point");
            assert_eq!(
                mesh.points()[at as usize],
                corner.point(),
                "a corner is put in the table as the model holds it"
            );
            assert!(
                seen.insert(at),
                "one entry per corner, read by all three faces"
            );
        }
        assert_eq!(seen.len(), 8);

        for edge in every_use_of_an_edge_of(model.solid(cube)) {
            assert!(
                mesh.points_along_edge(edge.key()).is_empty(),
                "a plane has nothing to say about a curve on it, and a straight edge \
                 needs nothing said"
            );
        }
        for corner in every_corner_of(model.solid(another)) {
            assert_eq!(
                mesh.point_at_vertex(corner.key()),
                None,
                "a corner of a solid this mesh was not cut from"
            );
        }
        for edge in every_use_of_an_edge_of(model.solid(another)) {
            assert!(mesh.points_along_edge(edge.key()).is_empty());
        }
    }

    /// An edge walked into nothing and an edge nobody walked both come back
    /// empty, which is the choice [`Mesh::points_along_edge`] publishes.
    ///
    /// A bar's seam is a straight ruling up a curved wall: the wall walks it and
    /// there is nothing to put between its two ends. A cube's edges have a plane
    /// either side and are never walked at all. Only from in here can the two be
    /// told apart, which is the point of not making a caller try.
    #[test]
    fn an_edge_walked_into_nothing_and_an_edge_nobody_walked_look_the_same_from_outside() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = model
            .add_round_bar(5.0, 10.0, &Isometry::identity())
            .expect("a bar");
        let cube = model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube");
        let cut = |solid| {
            model
                .solid(solid)
                .mesh(
                    Deflection::of(0.5).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("both of these mesh")
        };

        let meshed_bar = cut(bar);
        let seam: Vec<EdgeView<'_>> = every_use_of_an_edge_of(model.solid(bar))
            .into_iter()
            .filter(|edge| !edge.is_closed().expect("a bar's edges are all readable"))
            .collect();
        assert_eq!(seam.len(), 2, "the seam, used twice by the wall");
        assert!(
            meshed_bar.along_edge.contains_key(&seam[0].key()),
            "the wall is curved, so it walks every edge round it including its seam"
        );
        assert!(
            meshed_bar.points_along_edge(seam[0].key()).is_empty(),
            "and a straight ruling comes back with nothing between its ends"
        );

        let meshed_cube = cut(cube);
        for edge in every_use_of_an_edge_of(model.solid(cube)) {
            assert!(
                !meshed_cube.along_edge.contains_key(&edge.key()),
                "an edge with a plane either side is never walked"
            );
            assert!(meshed_cube.points_along_edge(edge.key()).is_empty());
            assert!(
                !meshed_bar.along_edge.contains_key(&edge.key()),
                "nor is an edge of another solid"
            );
            assert!(meshed_bar.points_along_edge(edge.key()).is_empty());
        }
    }

    /// What the two maps cost a [`Mesh`]: each names a point of the mesh, and no
    /// point twice.
    ///
    /// A corner goes in the table once however many faces read it, and each
    /// point of a walk is put there by that one walk — so both maps together
    /// hold at most one `u32` per point of the mesh, beside what the maps
    /// themselves take per edge and per corner.
    #[test]
    fn the_two_maps_name_each_point_of_the_mesh_at_most_once() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let somewhere = Isometry::translation(Vector::new(4.0, -2.0, 1.0)).expect("a move");
        let solids = [
            model.add_round_bar(3.0, 5.0, &somewhere).expect("a bar"),
            model.add_ball(2.0, &somewhere).expect("a ball"),
            model
                .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a cube"),
        ];

        for solid in solids {
            let mesh = model
                .solid(solid)
                .mesh(
                    Deflection::of(0.1).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("all three of these mesh");
            let mut named = HashSet::new();
            let along = mesh.along_edge.values().flatten().copied();
            for at in along.chain(mesh.at_vertex.values().copied()) {
                assert!(
                    (at as usize) < mesh.points().len(),
                    "an index into the point table"
                );
                assert!(named.insert(at), "and no point is named twice");
            }
            assert!(!named.is_empty(), "a solid has corners");
        }
    }

    /// Every use of every edge of `solid`, each turned the way that use runs.
    fn every_use_of_an_edge_of(solid: SolidView<'_>) -> Vec<EdgeView<'_>> {
        solid
            .faces()
            .flat_map(FaceView::wires)
            .flat_map(WireView::coedges)
            .map(|coedge| coedge.edge())
            .collect()
    }

    /// Every corner of `solid`, once each.
    fn every_corner_of(solid: SolidView<'_>) -> Vec<VertexView<'_>> {
        let mut seen = HashSet::new();
        solid
            .faces()
            .flat_map(FaceView::wires)
            .flat_map(WireView::steps)
            .map(|step| match step {
                WireStepView::Along(coedge) => coedge.start(),
                WireStepView::Pole(vertex) => vertex,
            })
            .filter(|corner| seen.insert(corner.key()))
            .collect()
    }

    /// The four-cornered cell two of [`how_many_steps`]'s steps make, and what
    /// the triangulator charges it when nothing is allowed to refine it.
    ///
    /// A cell is the two steps sanctioned for the two edges leaving one corner,
    /// closed off into a rectangle in the surface's own two coordinates. The
    /// deflection handed to the triangulator is enormous on purpose: it stops
    /// the refinement before it starts, so what comes back is the charge on the
    /// cell as the walk drew it and not on whatever the triangulator would have
    /// cut it into.
    fn what_a_cell_of_two_steps_is_charged(stored: &Surface, at: [f64; 2], room: f64) -> f64 {
        let quarter = core::f64::consts::TAU / 4.0;
        let across = how_many_steps(stored, at, [at[0] + quarter, at[1]], room);
        let up = how_many_steps(stored, at, [at[0], at[1] + A_LONG_RULING], room);
        let cell = vec![
            [at[0], at[1]],
            [at[0] + quarter / across as f64, at[1]],
            [
                at[0] + quarter / across as f64,
                at[1] + A_LONG_RULING / up as f64,
            ],
            [at[0], at[1] + A_LONG_RULING / up as f64],
        ];
        let domain = TrimmedDomain::new(
            *stored,
            Loop::through(cell).expect("four corners of a rectangle"),
            Vec::new(),
            Deflection::of(1e9).expect("a length"),
        );
        triangulate(&domain)
            .expect("a rectangle of one cell")
            .deviation()
            .magnitude()
    }

    /// How far up a cone the cells below reach, and how far along every other
    /// surface's second coordinate they run.
    ///
    /// Forty, which on the cone of slope a half below is a frustum from two up
    /// to forty-two — twenty times as wide at the top as at the bottom. Nothing
    /// exotic: `cones::a_frustum` in this crate's tests builds one from one to
    /// three, and stretching that is the only thing this number does.
    const A_LONG_RULING: f64 = 40.0;

    /// A ball and a ring are handed cells that fit, and a cone is handed one
    /// charged eight times over.
    ///
    /// [`how_many_steps`] is asked for one edge at a time and never for the cell
    /// two of its steps close off, so the question is what the cell costs when
    /// the two answers are put together.
    ///
    /// **On a ball the cell fits, and it only just fits.** Both sides come back
    /// under `√(share · radius · room / 2)`, so a square one has a diagonal `√2`
    /// longer, which strays twice as far — and
    /// [`WHAT_THE_BOUNDARY_TAKES_OF_THE_BUDGET`] being a half is exactly that
    /// factor of two. The ball measured here uses **98.1%** of the deflection
    /// and nothing left over, so that constant is not a spare margin that could
    /// be spent elsewhere: raise it and the walk starts handing over cells that
    /// are already past the deflection before a triangle is cut. At `0.8` this
    /// test fails with the ball's cell charged `1.4640`.
    ///
    /// **A ring fits with room to spare, and the reason is not slack in the
    /// budget.** Its two terms `C_v²/(3r)` and `C_u²/(2S)` are *added*, and the
    /// one radius `how_many_steps` gives a torus is
    /// `1/(1/(3r) + 1/(2S))` — so the two directions **split** the half rather
    /// than each taking it. `0.481` and `0.435` for the outer and inner equators
    /// of a ring of four and one are that split, not headroom.
    ///
    /// **The pipe here is coverage and not evidence, and the difference is worth
    /// saying.** A cylinder is flat along its axis, so the step up one comes back
    /// as a single step and contributes nothing to what the cell is charged. Its
    /// `0.486` is one chord measured against half a budget — the same number a
    /// lone side would give. It says nothing about a diagonal, so it is checked
    /// on its own below rather than counted among the cells that carry the
    /// finding.
    ///
    /// **On a cone it does not fit, and by a factor that grows with the
    /// frustum.** A cone is flat along its rulings, so a step up one is charged
    /// `round.abs() == 0` and comes back a single step however long the frustum
    /// is; the step *round* it is then sized at the radius of the end it leaves
    /// from, while the cell it closes off reaches the radius at the other end.
    /// The frustum here is twenty times as wide at the top, and the cell is
    /// charged eight times the deflection.
    ///
    /// **A cell like that is refused and never meshed, which is the point of
    /// the second half of this test.** Handed to the triangulator at the
    /// deflection it was sized against it comes back
    /// `BoundarySideStraysFurtherThanTheDeflection`, straying `0.0801` against a
    /// deflection of `0.01`.
    ///
    /// **The diagonal never gets a say, and naming where the refusal comes from
    /// matters.** `triangulate` sweeps every boundary side against the whole
    /// deflection *before* it refines anything — `crates/rcad-mesh/src/
    /// triangulate.rs:181-188` — and the side that fails is the one at
    /// `v = 42` carrying the bottom rim's width round. It is over budget on its
    /// own, as a side, so the sweep refuses the patch and `refine_the_inside` is
    /// never called. (It is separately true that a boundary side is the one
    /// thing the refinement may not cut, `triangulate.rs:933`. That is not what
    /// stopped this cell.)
    ///
    /// # Why the walk's sizing cannot make a mesh stray, whatever it hands over
    ///
    /// Not an argument about which cells the walk happens to draw — that is a
    /// statement about today's callers, which §16.10 says is the wrong shape of
    /// claim. It is structural, and it is three links long.
    ///
    /// [`SolidView::mesh`] is the only function anywhere that returns a [`Mesh`].
    /// Its only feed is the [`triangulate`] call in [`BuildingAMesh::hand_over`].
    /// And `triangulate` returns `Ok` only when every triangle inside the domain
    /// satisfies `how_far_a_flat_piece_strays(...) <= deflection` — the boundary
    /// sweep above, and then `refine_the_inside` cutting until nothing misses or
    /// refusing when it cannot.
    ///
    /// So what the walk's sizing decides is **whether a mesh comes back at all**
    /// and **how many triangles it costs**. It cannot decide whether the answer
    /// is inside the deflection, because nothing that misses is ever returned.
    /// A cell charged eight times over is a refusal or a pile of extra
    /// triangles, never a mesh that strays.
    ///
    /// `how_far_inside_a_triangle_a_mesh_really_strays.rs` is the other end of
    /// that: it takes the triangles that do come back out of solids the builder
    /// makes and measures them against each surface's own closed-form distance,
    /// rather than reading the triangulator's bound back.
    #[test]
    fn the_cell_two_steps_close_off_fits_on_a_ball_and_is_eight_times_over_on_a_cone() {
        use rcad_geom::{Cone, Cylinder, Torus};

        let room = 1e-2;
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a ball"));
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 1.0).expect("a pipe"));
        let ring = Surface::Torus(Torus::new(Axis::z(), 4.0, 1.0).expect("a ring"));
        let cone = Surface::Cone(Cone::new(Axis::z(), 0.5).expect("a cone"));

        let a_pipes_cell = what_a_cell_of_two_steps_is_charged(&pipe, [0.0, 0.0], room) / room;
        assert!(
            a_pipes_cell <= 1.0,
            "a pipe's cell is charged {a_pipes_cell} of the deflection"
        );

        let fits = [
            ("a ball of radius one", ball, [0.0, 0.0]),
            ("a ring of four and one", ring, [0.0, 0.0]),
            (
                "the inner equator of that ring",
                ring,
                [core::f64::consts::PI, 0.0],
            ),
        ];
        for (what, stored, at) in fits {
            let share = what_a_cell_of_two_steps_is_charged(&stored, at, room) / room;
            assert!(
                share <= 1.0,
                "the cell the walk draws on {what} is charged {share} of the deflection"
            );
        }

        let ball_uses = what_a_cell_of_two_steps_is_charged(&ball, [0.0, 0.0], room) / room;
        assert!(
            (0.97..=1.0).contains(&ball_uses),
            "a ball's cell used {ball_uses} of the deflection, and the whole of it was spent"
        );

        let cone_uses = what_a_cell_of_two_steps_is_charged(&cone, [0.0, 2.0], room) / room;
        assert!(
            cone_uses > 8.0,
            "a cone's cell used {cone_uses} of the deflection"
        );

        let quarter = core::f64::consts::TAU / 4.0;
        let across = how_many_steps(&cone, [0.0, 2.0], [quarter, 2.0], room);
        let cell = vec![
            [0.0, 2.0],
            [quarter / across as f64, 2.0],
            [quarter / across as f64, 2.0 + A_LONG_RULING],
            [0.0, 2.0 + A_LONG_RULING],
        ];
        let refusal = triangulate(&TrimmedDomain::new(
            cone,
            Loop::through(cell).expect("four corners of a rectangle"),
            Vec::new(),
            Deflection::of(room).expect("a length"),
        ))
        .expect_err("a cell charged eight times over is not meshed");
        let CannotTriangulate::BoundarySideStraysFurtherThanTheDeflection { strays, .. } = refusal
        else {
            panic!("a cell charged {cone_uses} of the deflection was refused for {refusal:?}");
        };
        assert!(
            strays > room,
            "the side that was refused strays {strays}, which is inside the deflection"
        );
    }

    /// A ball cut by a plane along its own axis is walked one definite way
    /// round, and it is the way `Model::validate` reads the same edge.
    ///
    /// A unit ball at the origin cut by `x = -0.6`, with the one corner of the
    /// crossing circle at `(-0.6, ±0.8, 0)`: `0.36 + 0.64 == 1.0` exactly in
    /// `f64`, so the corner is on the ball to the last bit, the plane is
    /// parallel to the ball's axis and the corner is on its equator. That is
    /// the pair of coincidences that makes the longitude reference
    /// `(∇S × ∇T) · (ẑ × ∇C)` exactly zero — asserted below, so this test
    /// cannot stop exercising the second reference without saying so.
    ///
    /// **The two corners are the point.** They are the same circle read half a
    /// turn apart, and the direction is read *at the corner*, so a rule that
    /// answers at all has to answer differently at the two. Before the second
    /// reference this hands back `None` at both, and `walk` leaves the points in
    /// whichever order the walker produced — which is the same order at both
    /// corners, so half of these caps come out walked backwards.
    #[test]
    fn a_ball_cut_along_its_axis_does_not_mesh_its_cap_backwards() {
        let ball = Surface::Sphere(
            Sphere::new(Point::origin(), 1.0).expect("a ball of radius one at the origin"),
        );
        let flat = Surface::Plane(
            Plane::from_point_and_normal(Point::new(-0.6, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
                .expect("the plane x = -0.6"),
        );
        let along = [flat, ball];

        for corner in [Point::new(-0.6, 0.8, 0.0), Point::new(-0.6, -0.8, 0.0)] {
            let round = gradient_of(&ball, corner);
            let straight = gradient_of(&flat, corner);
            let forward = straight.cross(round);
            assert_eq!(
                forward.dot(Vector::new(0.0, 0.0, 1.0).cross(round)),
                0.0,
                "the longitude is square to the wire here, which is what this test is about"
            );

            let stored = the_way_a_closed_edge_is_stored_running(&along, corner)
                .expect("neither a pole nor a touch");
            let runs_along_stored = a_closed_crossing_is_walked_the_way_it_is_stored(
                [&flat, &ball],
                [straight, round],
                corner,
                Decision::Neg,
            )
            .expect("one of the two is curved")
            .expect("neither a pole nor a touch");
            assert_eq!(
                stored,
                if runs_along_stored { forward } else { -forward },
                "the mesher and the sweep have to spell one question one way (§23.8)"
            );
        }
    }
}
