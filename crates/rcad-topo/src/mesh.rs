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
    which_two_surfaces_a_run_is_between,
};
use rcad_mesh::{CannotTriangulate, Loop, NotALoop, TrimmedDomain, triangulate};
use rcad_predicates::{
    Approximation, Cause, Decision, Level, Measures, Point as PredicatePoint, Policy,
    ProvenanceLog, Record,
};
use smallvec::SmallVec;

use crate::error::walking_along_this_edge_refused;
use crate::fuse::name_of_surface_bits;
use crate::validate::{
    nothing_curved_says_where_it_bends, side_of_surface, the_curved_surface_a_run_names,
    the_way_it_is_parametrised_about,
};
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
    /// [`Error::TwoPatchesOfTheMeshDrewTheSameSide`] when two patches folded
    /// onto each other and a side would be used more than twice.
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
    /// twelve lean-and-deflection cells. All four are refusals
    /// rather than asserts precisely because
    /// [`Model::validate`](crate::Model::validate) is a separate call this
    /// does not require first, `Model` derives `Deserialize`, and a caller's
    /// bytes may not panic (CLAUDE.md, §4.1). Whatever [`rcad_geom::walk`]
    /// makes of an edge it cannot follow, which includes a seam whose
    /// half-spaces pick both halves of its meridian or neither.
    /// [`Error::AnEdgeBendsButEveryFaceAlongItIsFlat`] for an edge that is not
    /// straight, has only flat faces along it, and whose run names no curved
    /// surface to place it by — a circle a file imprinted on a flat face and
    /// named by its own plane, or a wall face storing a
    /// [`Plane`](rcad_geom::Plane) where its own
    /// [`Cylinder`](rcad_geom::Cylinder) belongs.
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
        building.finish()
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
    /// How many triangles use each side that is not along the boundary of the
    /// patch it was drawn in — see [`BuildingAMesh::finish`].
    drawn_inside_a_patch: HashMap<(u32, u32), u32>,
    /// The points where a face was cut into pieces, ends included.
    on_a_cut: HashSet<u32>,
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
            drawn_inside_a_patch: HashMap::new(),
            on_a_cut: HashSet::new(),
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
    ///
    /// **So a flat face walks only the edges no curved face will.** Those are
    /// the ones every face along is flat, and of them only the ones that bend:
    /// a straight edge's two ends are the whole of it and it gets no entry at
    /// all. One that bends is a circle imprinted on a flat face, and the
    /// surface that says where it goes is the one its run names — see
    /// [`Self::walk`].
    fn walk_the_edges_this_face_can_place(
        &mut self,
        face: FaceView<'_>,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let flat = the_surface_this_face_is_meshed_on(face)
            .as_plane()
            .is_some();
        let room =
            self.deflection.length() - how_far_moving_this_face_rounds(face, &self.along_each_edge);
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if self.along_edge.contains_key(&edge.key()) {
                    continue;
                }
                if flat && !only_a_flat_face_can_place(edge, &self.along_each_edge) {
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
    ///
    /// **A closed crossing that runs through a ball's pole has the pole among
    /// its points**, whether or not the walk would have landed there — see
    /// [`put_in_the_poles_a_closed_crossing_passes`].
    ///
    /// # What a refusal from here is in, and why it is not what was walked
    ///
    /// The points ride back out of [`the_frame_this_face_is_meshed_in`] and so
    /// does the refusal: a `rcad_geom::Error` carrying two `Surface`s has them
    /// moved by [`Error::with_its_surfaces_moved`](rcad_geom::Error::with_its_surfaces_moved)
    /// before it goes anywhere. Handing the shifted pair out is handing out
    /// numbers that are in a frame this function chose, which nothing outside
    /// this file can name — a published field that lies, which is what
    /// `docs/decisions.md` §113.6 found and §116 argues the fix for.
    ///
    /// # Errors
    ///
    /// Whatever [`rcad_geom::walk`] refuses with, through
    /// [`walking_along_this_edge_refused`], and
    /// [`Error::MoreMeshPointsThanASolidWillHold`] when the points this edge
    /// alone adds take the mesh past its cap, and
    /// [`Error::AnEdgeBendsButEveryFaceAlongItIsFlat`] for an edge with only flat
    /// faces along it whose run names no curved surface — the rule
    /// [`nothing_curved_says_where_it_bends`], which the sweep refuses by too,
    /// so only a solid that never went through the sweep reaches that refusal.
    ///
    /// # An edge with only flat faces along it
    ///
    /// Two planes cross in a straight line or not at all, so for such an edge
    /// the pair this hands the walk is one face's plane and the curved surface
    /// the run names — the circle's own cylinder, for a circle a file imprinted
    /// on a flat face. It is the pair [`rcad_geom::walk`] reads off the same run
    /// too, handed over already read so that the direction rule for a closed
    /// edge below sees a curved surface as well.
    fn walk(
        &mut self,
        edge: EdgeView<'_>,
        face: FaceView<'_>,
        room: f64,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Vec<u32>> {
        let back = the_frame_this_face_is_meshed_in(face);
        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        let (from, to) = ends_in_the_stored_order(edge);
        let run = NamedRun::new(
            edge.stored_run()
                .shifted(the_shift_this_face_is_meshed_with(face))?,
            [from, to]
                .map(|end| PredicatePoint::from(back.apply_to_point(end.point()).coordinates())),
        )?;
        let along = the_two_surfaces_either_side(edge, &self.along_each_edge, &back)?;
        if nothing_curved_says_where_it_bends(run.run(), along) {
            return Err(Error::AnEdgeBendsButEveryFaceAlongItIsFlat { edge: edge.key() });
        }
        let along = match the_curved_surface_a_run_names(run.run()) {
            Some(named) if along.iter().all(|side| side.as_plane().is_some()) => [along[0], named],
            Some(_) | None => along,
        };

        let along_it = rcad_geom::walk(
            &run,
            [&along[0], &along[1]],
            Deflection::of(room * WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO)?,
            policy,
            log,
        )
        .map_err(|why| {
            walking_along_this_edge_refused(edge.key(), why.with_its_surfaces_moved(&there))
        })?;
        let mut points = along_it.points().to_vec();
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
                    .or_else(|| the_way_a_closed_edge_where_its_faces_touch_runs(&along, corner))
                && (*first - corner).dot(stored_way) < 0.0
            {
                points.reverse();
            }
        }
        // **Both kinds of closed run get their poles**, and the pair the second
        // kind hands over is the run's own and not the two faces. A rim written
        // [`EdgeRun::RightRoundWhereItCrosses`] has the one surface either side
        // of it — that is why it carries `and` at all — so the faces say
        // `[ball, ball]`, which [`put_in_the_poles_a_closed_crossing_passes`]
        // reads as a ball crossed with itself and walks straight past. The pair
        // below is the one `rcad_geom::walk` read off the same run to draw the
        // circle with (`which_two_surfaces_a_run_is_between`), so the poles are
        // put in on the curve that was actually walked.
        let ends = [from, to].map(|end| back.apply_to_point(end.point()));
        let moved = match run.run() {
            EdgeRun::RightRoundWhereItsFacesCross | EdgeRun::RightRoundWhereItCrosses { .. } => {
                put_in_the_poles_a_closed_crossing_passes(
                    &mut points,
                    ends,
                    &which_two_surfaces_a_run_is_between(run.run(), [&along[0], &along[1]]),
                )
            }
            EdgeRun::Straight | EdgeRun::PartOfWhereItCrosses { .. } | _ => 0.0,
        };
        if self.points.len() + points.len() > MOST_POINTS_A_MESH_WILL_HOLD {
            return Err(Error::MoreMeshPointsThanASolidWillHold {
                most: MOST_POINTS_A_MESH_WILL_HOLD,
            });
        }

        let moving = how_far_moving_this_face_rounds(face, &self.along_each_edge);
        self.worst = self
            .worst
            .max(along_it.deviation().magnitude() + moved + moving);
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
        if let Some(whole) = how_far_the_second_coordinate_wraps(&stored)
            && let Some((first, later)) = walked.split_first_mut()
        {
            for chain in later {
                *chain = with_its_second_coordinate_in_the_turn_of(chain, first, whole);
            }
        }
        // **A wire that winds is only allowed beside wires that wind too.** The
        // loops of a face cut the chart into stretches, and the one the face is
        // has every one of its wires in its boundary. Two that go right round
        // bound the band between them and
        // [`BuildingAMesh::cut_a_ring_across`] cuts it; one that goes right
        // round beside one that does not says nothing about which side of the
        // winding one the other sits, so the face is refused the way a wire
        // that will not lie flat has always been refused.
        if walked.iter().any(|around| around.goes_right_round)
            && !walked.iter().all(|around| around.goes_right_round)
        {
            return Err(Error::AWireWillNotLieFlatOnItsFace { face: face.key() });
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
        }; // **A loop drawn on a ball bounds two pieces and only one of them is a
        // patch this can fill.** The triangulator fills the inside of what it
        // is handed, and the inside of a loop in the surface's own two
        // coordinates is the piece holding no pole. When the material is on the
        // *other* side — the piece holding both poles, which a ball with a cap
        // taken off it is — the answer is the whole ball less a disc, and
        // writing it needs a seam and two poles the way §38.1 wrote an uncut
        // one. [`BuildingAMesh::open_the_ball_along_a_seam`] puts them in, and
        // what it cannot open is refused by name rather than filled inside out:
        // filled, the union of two balls of radius two whose centres are two
        // apart came back `Ok` measuring 10.40, where `18π = 56.55` is the
        // union and `10π/3 = 10.47` is the *lens* between them — the
        // complement of the piece that was asked for.
        //
        // **Only asked of a surface that pinches**, which here is only a ball,
        // and on a pipe there is nothing to ask. A pipe's two coordinates are
        // longitude and height and it runs on for ever, so a loop that does not
        // go right round it bounds a patch one way and an **unbounded** region
        // the other. A solid is bounded, so its faces are, so the material is
        // the patch — there is no second case for a sign to tell apart.
        let widest = whole_turn / 4.0;
        if walked.len() == 1
            && walked[0].closes_on_one_curve
            && !walked[0].goes_right_round
            && where_this_surface_pinches(stored).is_some()
            && how_much_of_the_chart_this_loop_encloses(&walked[0]) <= 0.0
        {
            let Some(outer) = self.open_the_ball_along_a_seam(face, stored, &walked[0], room)?
            else {
                return Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { face: face.key() });
            };
            let Some((one, other)) =
                self.cut_a_hole_up_to_both_poles(face, stored, &[outer, walked[0].clone()], room)?
            else {
                return Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { face: face.key() });
            };
            return self.cut_each_narrow_enough(face, stored, vec![one, other], widest, room);
        }
        if walked.len() == 1
            && let Some((one, other)) = self.open_a_slit(face, stored, &walked[0], room)?
        {
            return self.cut_each_narrow_enough(face, stored, vec![one, other], widest, room);
        }
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
        // refuses a curved solid outright"*. The other face with a hole this
        // cuts is a ball's skin whose outer loop reaches both poles and whose
        // hole reaches neither, which a half ball cut again leaves: it is cut
        // from the hole up to each pole instead. A face with one loop right
        // round and another that is not has already been refused in
        // [`BuildingAMesh::cut_up`], because nothing says which side of the
        // winding loop the other sits. `Model` is `Deserialize` and `validate`
        // is the untrusted-import boundary (§16.11), so that is a refusal
        // rather than an assert.
        //
        // **The two curves need not be one closed curve each.** A frustum's
        // band between its own near cap circle and a ring of arcs a cube's
        // upper planes cut is a ring too, and the arcs are the ring §208.4
        // taught the walk to come back a whole turn on.
        if walked.len() != 1 {
            if let Some(patches) =
                self.cut_a_ring_across(face, stored, &walked, whole_turn, room)?
            {
                return Ok(patches);
            }
            let Some((one, other)) =
                self.cut_a_hole_up_to_both_poles(face, stored, &walked, room)?
            else {
                return Err(Error::CurvedFaceWithAHoleGoesTooFarRound { face: face.key() });
            };
            return self.cut_each_narrow_enough(face, stored, vec![one, other], widest, room);
        }
        let only = walked.into_iter().next().expect("just checked");
        self.cut_each_narrow_enough(face, stored, vec![only], widest, room)
    }

    /// The outer loop of a ball less a disc, for a face whose one wire is the
    /// rim of the disc: a seam from pole to pole and the two poles, the
    /// rectangle §38.1 walks an uncut ball as. `None` when the surface does not
    /// pinch or the rim reaches a whole turn round, where no meridian misses it.
    ///
    /// **The seam is the meridian half a turn from the middle of the rim**, so
    /// its two sides, a whole turn apart, both lie strictly outside the
    /// longitudes the rim runs between and the rim is an island in the
    /// rectangle — the shape [`BuildingAMesh::cut_a_hole_up_to_both_poles`]
    /// cuts. Nothing in the model is along it, so its points are made here once
    /// and used by both sides, which is what makes the two sides one line in
    /// space. The rectangle is walked anticlockwise, bottom pole side first,
    /// because the material is inside it.
    fn open_the_ball_along_a_seam(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        rim: &WalkedLoop,
        room: f64,
    ) -> Result<Option<WalkedLoop>> {
        let (Some(whole), Some(pole)) = (
            how_far_the_longitude_wraps(stored),
            where_this_surface_pinches(stored),
        ) else {
            return Ok(None);
        };
        let (least, most) = longitudes_of(rim);
        if most - least >= whole {
            return Ok(None);
        }
        let seam = (least + most) / 2.0 - whole / 2.0;
        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        let [south, north] = [-pole, pole].map(|latitude| {
            let at = self.remember(there.apply_to_point(stored.point_at(seam, latitude)));
            self.on_a_cut.insert(at);
            at
        });
        let up = self.points_along_a_cut(face, stored, [seam, -pole], [seam, pole], room)?;

        let mut outer = WalkedLoop {
            corners: vec![[seam, -pole], [seam + whole, -pole]],
            points: vec![south, south],
            closes_on_one_curve: false,
            goes_right_round: false,
            climbed: 0.0,
        };
        for (corner, &point) in up.corners.iter().zip(&up.points) {
            outer.corners.push([corner[0] + whole, corner[1]]);
            outer.points.push(point);
        }
        outer.corners.extend([[seam + whole, pole], [seam, pole]]);
        outer.points.extend([north, north]);
        outer.take_in(&up.the_other_way());
        Ok(Some(outer))
    }

    /// Cut each piece in two until none of them goes further round than
    /// `widest`, or until no cut makes one any narrower.
    fn cut_each_narrow_enough(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        mut pieces: Vec<WalkedLoop>,
        widest: f64,
        room: f64,
    ) -> Result<Vec<Vec<WalkedLoop>>> {
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

    /// Cut a face in two along the line that carries on from a **slit** in its
    /// wire, or `None` when the wire has no slit.
    ///
    /// A slit is one edge the wire runs out along and straight back, ending at
    /// a corner inside the face: in the surface's two coordinates the loop
    /// passes the same points twice, and the triangulator refuses a loop with
    /// two corners in one place. A seam is written that way whenever the file's
    /// own chart of a closed surface is not this kernel's, and the one that
    /// found it is `kicad/LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step`'s dome.
    /// The file's ball has its axis along `−y`, so its seam runs from its pole
    /// — the top of the dome — to the rim, and in its chart the two uses are a
    /// whole turn apart. A [`Sphere`](rcad_geom::Sphere) here always has its
    /// axis along `z`, where the top of the dome is an ordinary point of the
    /// equator; there the seam is a line from `(−π/2, 0)` to the pole at
    /// `(−π/2, π/2)`, used twice on the same points, and **no longitude it
    /// could be moved to by whole turns stops that**, because walking round an
    /// ordinary point gains no longitude.
    ///
    /// So the face is cut instead, on from the slit's inner end in the direction
    /// away from the slit, to the first side of the loop that line reaches.
    /// Each half then has one side of the slit and no point twice. The cut ends
    /// on points both halves already have, by the rule [`where_to_cut`] keeps.
    fn open_a_slit(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        around: &WalkedLoop,
        room: f64,
    ) -> Result<Option<(WalkedLoop, WalkedLoop)>> {
        let Some(tip) = the_inner_end_of_a_slit(around) else {
            return Ok(None);
        };
        let Some((opened, [tip, far])) = where_a_slit_carries_on_to(around, tip) else {
            return Ok(None);
        };
        self.on_a_cut.insert(opened.points[tip]);
        self.on_a_cut.insert(opened.points[far]);
        let cut =
            self.points_along_a_cut(face, stored, opened.corners[far], opened.corners[tip], room)?;
        let mut one = opened.between(tip, far);
        one.take_in(&cut);
        let mut other = opened.between(far, tip);
        other.take_in(&cut.the_other_way());
        Ok(Some((one, other)))
    }

    /// Cut a **cap** — one loop right round a surface that pinches to a point,
    /// with nothing else bounding it — into four patches with one loop each.
    ///
    /// A ball cut by a plane square to its polar axis leaves one, and it is
    /// about the simplest curved boolean there is. A plane at any other angle
    /// leaves a loop that climbs in longitude and comes straight back, which
    /// goes to [`BuildingAMesh::cut_in_two`] instead and needs no pole at all —
    /// unless the plane runs through one pole, when the piece holding the
    /// other is a cap again, with a pole side in its rim.
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
        let Some(pinch) = where_this_surface_pinches(stored) else {
            return Ok(None);
        };
        let Some(towards) = self.which_pole_this_cap_reaches(face, stored, cap) else {
            return Ok(None);
        };

        // A quarter turn at a time, **the way the loop itself runs**, so that
        // `between` walks a quarter of the way round and not three quarters.
        let way = if runs_up(cap) { 1.0 } else { -1.0 };
        let cap = &starting_off_the_poles(cap, pinch);
        let from = cap.corners[0][0];
        let quarters: [f64; 4] =
            [0, 1, 2, 3].map(|step| from + way * f64::from(step) * whole_turn / 4.0);
        let cap = &with_a_corner_where_a_pole_side_passes(cap, &quarters, pinch);
        let along = quarters.map(|at| nearest_corner_off_the_rims_poles(cap, at, pinch));

        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        let at_the_pole = stored.point_at(from, towards);
        if !at_the_pole.is_finite() {
            return Ok(None);
        }
        let at_the_pole = self.remember(there.apply_to_point(at_the_pole));
        self.on_a_cut.insert(at_the_pole);

        let mut cuts = Vec::with_capacity(4);
        for &corner in &along {
            self.on_a_cut.insert(cap.points[corner]);
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
            //
            // **The side ends at the meridian the cut down leaves by, which is
            // the patch's own first corner, and not a quarter turn back.** The
            // cuts leave from the rim corners nearest the quarters, and on a
            // coarse rim those are not a quarter apart: the dome of §191's
            // turned ball and block has four of them `2.74`, `0.40`, `2.74` and
            // `0.40` radians apart, so a quarter turn back ran `1.17` past the
            // next cut's meridian in two patches, the side folded back on
            // itself, and the mesher refused it as a boundary that crosses
            // itself. `a_turned_ball_united_with_a_block_holds_what_the_upright_one_does`.
            let corner = one.corners.last().map_or(0.0, |corner| corner[0]);
            one.corners.push([corner, towards]);
            one.points.push(at_the_pole);
            let back = one.corners[0][0];
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
            if runs_up(around) {
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
        // **Each quarter is brought into the bottom loop's own turn first.**
        // Both loops run a whole turn, but each starts wherever the pair that
        // cut it started, so a quarter taken off the top can land outside the
        // longitudes the bottom's corners run between — and
        // [`nearest_longitude`] would then answer the same end corner for two
        // different quarters. A frustum's band, ringed by a corner-up cube's
        // arcs, has its bottom running `-2.66` to `3.58` against a top running
        // `0` to `6.28`: the quarter at `4.71` fell off the end, a patch took a
        // sliver of the bottom where it took a quarter of the top, and the mesh
        // came back `BoundaryCrossesItself`.
        //
        // **A bar through a block was cut that way too**, and it took letting
        // the frustum's ring in to notice: its bottom corners came back
        // `[1, 2, 3, 3]` for the four quarters, so one patch had no width along
        // the bottom at all and the next spanned half the turn — which is what
        // [`BuildingAMesh::cut_off_what_goes_too_far_round`]'s own argument says
        // no patch may do.
        let along_the_bottom =
            quarters.map(|at| nearest_longitude(&down, in_the_turn_of(&down, at, whole_turn)));

        let mut cuts = Vec::with_capacity(4);
        for step in 0..4 {
            let leaving = up.corners[along_the_top[step]];
            let landing = down.corners[along_the_bottom[step]];
            // The same place on the surface, at whichever of its longitudes
            // lies nearest the corner the cut sets off from, so the cut runs
            // across the band and not a whole turn round it. A shift of no
            // turns leaves the longitude byte for byte, which is why the turns
            // are taken off rather than the difference added back on.
            let arriving = [
                landing[0] - whole_turns_between(landing[0], leaving[0], whole_turn),
                landing[1],
            ];
            self.on_a_cut.insert(up.points[along_the_top[step]]);
            self.on_a_cut.insert(down.points[along_the_bottom[step]]);
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

    /// Cut a face on a ball whose outer loop reaches both poles and whose one
    /// hole reaches neither into two pieces with one loop each, or `None` when
    /// the face is not that shape or a cut cannot be shown to stay on it.
    ///
    /// A half ball cut again by a plane square to its flat face leaves one: its
    /// outer wire is the great circle through both poles, walked as two edges
    /// between them, and its hole is the whole small circle the second plane
    /// cuts. In the ball's two coordinates the outer loop is a rectangle whose
    /// top and bottom are the two poles and the hole is an island inside it, so
    /// **two straight cuts along one longitude each** — from the hole's highest
    /// corner up to the top pole and from its lowest down to the bottom one —
    /// leave a piece either side, each walked round once.
    ///
    /// **Each cut ends on points both pieces already have.** At the hole's end
    /// that is one of its corners, which the flat face across the hole has too;
    /// at the pole's end it is the pole, which is one point wherever along its
    /// side the cut lands (the rule [`where_to_cut`] keeps).
    ///
    /// **A cut that is not shown to stay inside the face is refused rather than
    /// drawn.** Starting at the hole's highest corner means no side of the hole
    /// reaches above it, because a side is a straight line between two corners in
    /// these coordinates. The cut then runs to the *nearest* side of the outer
    /// loop above it, and that side has to be a pole side; one that is not, or
    /// one lying along the cut, is `None`. Last, each piece has to enclose some
    /// of the chart the way the face's material does, which is what a piece
    /// walked inside out would not.
    fn cut_a_hole_up_to_both_poles(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        walked: &[WalkedLoop],
        room: f64,
    ) -> Result<Option<(WalkedLoop, WalkedLoop)>> {
        let [first, second] = walked else {
            return Ok(None);
        };
        if where_this_surface_pinches(stored) != Some(core::f64::consts::FRAC_PI_2)
            || first.goes_right_round
            || second.goes_right_round
        {
            return Ok(None);
        }
        let (outer, hole) = match [first, second].map(how_much_of_the_chart_this_loop_encloses) {
            [one, other] if one > 0.0 && other < 0.0 => (first, second),
            [one, other] if one < 0.0 && other > 0.0 => (second, first),
            _ => return Ok(None),
        };
        if (0..hole.points.len())
            .any(|at| hole.points[at] == hole.points[(at + 1) % hole.points.len()])
        {
            return Ok(None);
        }
        let highest = the_corner_furthest(hole, 1.0);
        let lowest = the_corner_furthest(hole, -1.0);
        let [up_at, down_at] = [(highest, 1.0), (lowest, -1.0)].map(|(corner, way)| {
            let [longitude, latitude] = hole.corners[corner];
            the_pole_side_a_line_from_a_hole_reaches(outer, longitude, latitude, way)
        });
        let (Some(up_at), Some(down_at)) = (up_at, down_at) else {
            return Ok(None);
        };
        if up_at.0 == down_at.0 {
            return Ok(None);
        }

        let mut opened = outer.clone();
        let top_corner = [hole.corners[highest][0], up_at.1];
        let bottom_corner = [hole.corners[lowest][0], down_at.1];
        let (top, bottom) = if up_at.0 > down_at.0 {
            opened.corners.insert(up_at.0 + 1, top_corner);
            opened.points.insert(up_at.0 + 1, outer.points[up_at.0]);
            opened.corners.insert(down_at.0 + 1, bottom_corner);
            opened.points.insert(down_at.0 + 1, outer.points[down_at.0]);
            (up_at.0 + 2, down_at.0 + 1)
        } else {
            opened.corners.insert(down_at.0 + 1, bottom_corner);
            opened.points.insert(down_at.0 + 1, outer.points[down_at.0]);
            opened.corners.insert(up_at.0 + 1, top_corner);
            opened.points.insert(up_at.0 + 1, outer.points[up_at.0]);
            (up_at.0 + 1, down_at.0 + 2)
        };

        let mut one = opened.between(top, bottom);
        let mut other = opened.between(bottom, top);
        let to_the_top =
            self.points_along_a_cut(face, stored, hole.corners[highest], top_corner, room)?;
        let from_the_bottom =
            self.points_along_a_cut(face, stored, bottom_corner, hole.corners[lowest], room)?;

        one.take_in(&from_the_bottom);
        one.take_in(&hole.between(lowest, highest));
        one.take_in(&to_the_top);
        other.take_in(&to_the_top.the_other_way());
        other.take_in(&hole.between(highest, lowest));
        other.take_in(&from_the_bottom.the_other_way());

        if [&one, &other]
            .iter()
            .any(|piece| how_much_of_the_chart_this_loop_encloses(piece) <= 0.0)
        {
            return Ok(None);
        }
        for at in [
            hole.points[highest],
            hole.points[lowest],
            outer.points[up_at.0],
            outer.points[down_at.0],
        ] {
            self.on_a_cut.insert(at);
        }
        Ok(Some((one, other)))
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
            climbed: 0.0,
        };
        for step in 1..steps {
            let part = step as f64 / steps as f64;
            let sits_at =
                [0, 1].map(|which| leaving[which] + part * (arriving[which] - leaving[which]));
            let on_it = stored.point_at(sits_at[0], sits_at[1]);
            along.corners.push(sits_at);
            let at = self.remember(there.apply_to_point(on_it));
            self.on_a_cut.insert(at);
            along.points.push(at);
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
    ///
    /// **A piece that is still one whole closed curve gets a cut carrying a
    /// point of its own**, however little curve there is to step. A cut's two
    /// ends are points the rest of the model already holds, so a cut with
    /// nothing strictly inside it is a bare *side* between two shared points —
    /// and every corner of a piece that is one closed curve sits on that one
    /// edge, which the face on the other side of it holds whole. That face is
    /// free to draw the same pair as a triangle's diagonal, and then the side is
    /// used four times and the mesh is refused
    /// `TwoPatchesOfTheMeshDrewTheSameSide`. The ball cut through its south pole
    /// is where it happened: the sliver's rim has the pole as one of the four
    /// corners a coarse walk gives it, the cut ran from the pole to the corner
    /// opposite, and the flat face closing the sliver picked exactly that
    /// diagonal (`the_sliver_that_holds_no_pole_is_answered_or_refused_by_name`).
    /// A point strictly inside the cut belongs to this face alone (§25.1), so
    /// there is no pair left for a neighbour to draw — and on a ball it is also
    /// the one point off the rim's own plane that a coarse sliver would
    /// otherwise have none of, which is what turned `VolumeIsNotPositive` into
    /// an answer.
    ///
    /// **It is asked for where the collision was shown and not everywhere.**
    /// Charging every cut a point moved a round bar's measured volume by `3.6e-4` when the bar was
    /// shifted three thousand units out, against `1.8e-13` before, because a
    /// point worked out from the chart carries the placement's rounding where
    /// a corner of the model does not
    /// (`a_bend_moved_far_out_stays_inside_its_bound_and_slips_further_than_a_bar`).
    /// **That is a cost bound and not a proof.** Nothing here says a bare cut
    /// across a piece bounded by more than one closed curve could never meet a
    /// neighbour's diagonal — only that no solid the gate builds has shown one,
    /// and that the ones that do reach this go on paying nothing for the
    /// possibility.
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
        let least_steps = if piece.closes_on_one_curve { 2 } else { 1 };
        let steps = how_many_steps(stored, leaving, arriving, room).max(least_steps);
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
            climbed: 0.0,
        };
        self.on_a_cut.insert(opened.points[at[0]]);
        self.on_a_cut.insert(opened.points[at[1]]);
        for step in 1..steps {
            let part = step as f64 / steps as f64;
            let sits_at =
                [0, 1].map(|which| leaving[which] + part * (arriving[which] - leaving[which]));
            let on_it = stored.point_at(sits_at[0], sits_at[1]);
            along_the_cut.corners.push(sits_at);
            let at = self.remember(there.apply_to_point(on_it));
            self.on_a_cut.insert(at);
            along_the_cut.points.push(at);
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
        let mut along_a_loop = HashSet::new();
        let mut between_two_edges = Vec::new();
        let model_corners: HashSet<u32> = self.at_vertex.values().copied().collect();
        for one in patch {
            for (at, &from) in one.points.iter().enumerate() {
                along_a_loop.insert(a_side(from, one.points[(at + 1) % one.points.len()]));
                // A loop that is one closed edge passes its own corner and stays
                // on that edge. Anywhere else a corner of the model, or a point
                // where this face was cut into pieces, is taken to be where the
                // boundary passes from one neighbour to another; where that is
                // not so — two edges between the same two faces — `finish` is
                // what stands between a shared side and an `Ok`.
                between_two_edges.push(
                    !one.closes_on_one_curve
                        && (model_corners.contains(&from) || self.on_a_cut.contains(&from)),
                );
            }
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
        )
        .with_corners_between_two_edges(between_two_edges);
        let cut = match triangulate(&domain) {
            Ok(cut) => cut,
            Err(why) => return Err(refusal_for(face, why, &corner_is, &self.points, log)),
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
            for (from, to) in [(first, second), (second, third), (third, first)] {
                let side = a_side(from, to);
                if !along_a_loop.contains(&side) {
                    *self.drawn_inside_a_patch.entry(side).or_default() += 1;
                }
            }
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
    /// `None` when the walk wound round the surface and this wire is not one
    /// that may — see [`WalkingAWire::finish`] and
    /// [`WalkingAWire::may_come_back_a_whole_turn_on`].
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
    ///
    /// **A point walked along an edge that stands at a pole is a pole too**,
    /// and its side is opened the way a corner's is. So is a point or a corner
    /// within one rounding of a pole in space, whatever `atan2` reads its
    /// latitude as ([`the_pole_a_point_sits_within_a_rounding_of`]). A plane through a ball's
    /// axis cuts it in a circle through both poles, and an edge with its one
    /// corner off them passes both in its middle; taking each the short way
    /// round is a coin toss at exactly half a turn (§192.1). A walk that would
    /// have stepped past the pole between two points has had the pole put
    /// among them by [`put_in_the_poles_a_closed_crossing_passes`] (§193.3).
    fn walk_a_wire_round(
        &mut self,
        wire: WireView<'_>,
        face: FaceView<'_>,
        stored: &Surface,
    ) -> Option<WalkedLoop> {
        let back = the_frame_this_face_is_meshed_in(face);
        let poles = poles_of(wire);
        let sits_at_a_pole = |at: Point| {
            the_surface_pinches_at(stored, stored.coordinates_of(at).1)
                || the_pole_a_point_sits_within_a_rounding_of(stored, at).is_some()
        };
        let at_a_pole = |vertex: &VertexView<'_>| {
            poles.contains(&vertex.key()) || sits_at_a_pole(back.apply_to_point(vertex.point()))
        };
        let mut used = HashSet::new();
        let no_side_of_this_wire_is_a_seam = wire
            .coedges()
            .all(|coedge| used.insert(coedge.edge().key()));
        let mut walk = WalkingAWire::new(
            how_far_the_longitude_wraps(stored),
            how_far_the_second_coordinate_wraps(stored),
            wire.closes_on_one_edge(),
            no_side_of_this_wire_is_a_seam,
        );

        let steps = steps_from_somewhere_that_has_a_longitude(wire, &at_a_pole);
        for (index, &step) in steps.iter().enumerate() {
            match step {
                WireStepView::Along(coedge) => {
                    let start = coedge.start();
                    let at = self.remember_the_corner(start);
                    let where_it_is = back.apply_to_point(start.point());
                    let start_is_a_pole = at_a_pole(&start);
                    walk.arrive(at, stored.coordinates_of(where_it_is), start_is_a_pole);
                    let a_pole_step_came_first =
                        index > 0 && matches!(steps[index - 1], WireStepView::Pole(_));
                    if start_is_a_pole && !a_pole_step_came_first {
                        walk.turn_at_a_corner_on_a_pole();
                    }

                    let edge = coedge.edge();
                    let walked = match self.along_edge.get(&edge.key()) {
                        Some(walked) => walked.clone(),
                        // Every edge that is not straight was walked or refused
                        // by `walk_the_edges_this_face_can_place`, which ran over
                        // every face first — so a missing entry for one is that
                        // pass broken, and reading it as "no points" is how a
                        // circle came out as a triangle (§176.2).
                        None => {
                            assert!(
                                *edge.stored_run() == EdgeRun::Straight,
                                "an edge that bends was never walked: {:?}",
                                edge.key()
                            );
                            Vec::new()
                        }
                    };
                    let running_back = edge.orientation().is_reversed();
                    for step in 0..walked.len() {
                        let at = if running_back {
                            walked[walked.len() - 1 - step]
                        } else {
                            walked[step]
                        };
                        let where_it_is = back.apply_to_point(self.points[at as usize]);
                        let sits_at = stored.coordinates_of(where_it_is);
                        let passes_a_pole = sits_at_a_pole(where_it_is);
                        walk.arrive(at, sits_at, passes_a_pole);
                        if passes_a_pole {
                            walk.turn_at_a_corner_on_a_pole();
                        }
                    }
                }
                WireStepView::Pole(vertex) => {
                    let at = self.remember_the_corner(vertex);
                    let where_it_is = back.apply_to_point(vertex.point());
                    walk.arrive(at, stored.coordinates_of(where_it_is), true);
                    walk.cross_the_pole(a_seam_turns_back_at(&steps, index));
                }
            }
        }
        if !the_walk_comes_back_up_the_edge_it_set_off_along(&steps) {
            walk.arrive_back_at_the_pole_the_walk_set_off_from();
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

    /// **Every side the triangulator drew inside a patch is used by exactly the
    /// two triangles either side of it, both from that patch.** A side along a
    /// loop is the model's to count — a file may hand over an edge with three
    /// faces along it — but a side drawn inside a patch belongs to that patch
    /// alone, so any third use is two patches having drawn the same side, the
    /// pocket of §187.4, and the mesh would come back with a side used four
    /// times.
    ///
    /// **Refused rather than asserted**, although the leaning cap that found it
    /// was this crate's own arithmetic: a caller's bytes reach it too. A ball
    /// whose pole step stands on its centre rather than a pole folds two patches
    /// onto each other the same way, and
    /// [`Error::TwoPatchesOfTheMeshDrewTheSameSide`] says what it is for both.
    /// The side reported is the lowest-numbered one, so the same model is
    /// refused the same way every time.
    fn finish(self) -> Result<Mesh> {
        let mut times_used: HashMap<(u32, u32), u32> = HashMap::new();
        for triangle in &self.triangles {
            let [first, second, third] = triangle.corners;
            for (from, to) in [(first, second), (second, third), (third, first)] {
                *times_used.entry(a_side(from, to)).or_default() += 1;
            }
        }
        let folded = self
            .drawn_inside_a_patch
            .iter()
            .map(|(&side, &drawn)| (side, drawn, times_used[&side]))
            .filter(|&(_, drawn, used)| drawn != 2 || used != 2)
            .min_by_key(|&(side, _, _)| side);
        if let Some((_, drawn, used)) = folded {
            return Err(Error::TwoPatchesOfTheMeshDrewTheSameSide { drawn, used });
        }
        Ok(Mesh {
            points: self.points,
            triangles: self.triangles,
            at_vertex: self.at_vertex,
            along_edge: self.along_edge,
            deviation: Approximation::new(self.worst, Cause::FlatStandIn),
        })
    }
}

/// A side between two mesh points, whichever way round it is walked.
fn a_side(from: u32, to: u32) -> (u32, u32) {
    (from.min(to), from.max(to))
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
    /// How much longitude the walk gained all the way round, as the walk itself
    /// counted it: pole sides as far as [`WalkingAWire::cross_the_pole`] said
    /// they run, not the short way round. Only a whole walked loop has one; a
    /// stretch or a cut says `0`.
    climbed: f64,
}

impl WalkedLoop {
    /// The stretch from corner `from` round to corner `to`, both ends kept.
    fn between(&self, from: usize, to: usize) -> WalkedLoop {
        let mut taken = WalkedLoop {
            corners: Vec::new(),
            points: Vec::new(),
            closes_on_one_curve: false,
            goes_right_round: false,
            climbed: 0.0,
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
            climbed: -self.climbed,
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

/// Whether a loop that goes right round is walked in the direction the
/// longitude grows.
///
/// **Read off the longitude the walk itself gained, and never by adding the
/// corners up the short way round.** A cap whose rim runs through the *other*
/// pole — a ball cut by a plane through its south pole, keeping the north one —
/// crosses a pole side there, because a smooth curve through a pole leaves up
/// the meridian half a turn from the one it came down. A pole side runs as far
/// as [`WalkingAWire::cross_the_pole`] says, and on a coarse walk that is more
/// than half a turn, which the short way reads as the long way back: the sum
/// came out `0` for a cap the walk had counted a whole turn up, the cap was cut
/// the wrong way round, and the build refused 12 of the 63 solids in
/// `a_cap_whose_rim_runs_through_the_other_pole.rs` as faces that disagree
/// which way they face. The walk's own count decides it, the same one
/// [`WalkingAWire::finish`] decides "right round" by.
fn runs_up(around: &WalkedLoop) -> bool {
    assert!(
        around.goes_right_round,
        "only a loop that goes right round runs one way round the surface"
    );
    around.climbed > 0.0
}

/// The same cap, walked from its first corner that is not on a pole.
///
/// A rim through a pole leaves it straight up a meridian, so a cut up to the
/// far pole that leaves from the pole corner runs along the rim's own first
/// step, and the patches either side of that stretch fold onto each other.
/// Starting the quarters at a corner off the pole keeps every cut off the rim.
/// The corners before it move on a whole `climbed`, so the longitudes stay
/// the walk's own unwrapped ones.
fn starting_off_the_poles(cap: &WalkedLoop, pinch: f64) -> WalkedLoop {
    let Some(start) = cap
        .corners
        .iter()
        .position(|corner| corner[1] != pinch && corner[1] != -pinch)
    else {
        return cap.clone();
    };
    let order = (start..cap.corners.len()).chain(0..start);
    WalkedLoop {
        corners: order
            .clone()
            .map(|at| {
                let [longitude, latitude] = cap.corners[at];
                let on = if at < start { cap.climbed } else { 0.0 };
                [longitude + on, latitude]
            })
            .collect(),
        points: order.map(|at| cap.points[at]).collect(),
        ..cap.clone()
    }
}

/// The cap with a corner put in at each of `quarters` that falls strictly
/// inside one of its pole sides, so that a cut up to the far pole can leave
/// from there.
///
/// **A cap whose rim runs through the other pole spends much of its turn on
/// that pole**, as a side from the meridian the rim came down to the one half
/// a turn round that it leaves up. On a coarse walk that side is most of the
/// turn: the build's check of the radius 1, lean 0.7, turn 0.4 cap in
/// `a_cap_whose_rim_runs_through_the_other_pole.rs` walked five corners, two of
/// them the pole side's ends, spanning under a third of the turn, so three of
/// the four quarters came back as the same nearest corner and the patches
/// between them had no width. A corner on the pole side at the quarter's own longitude is
/// the same point, which is what a pole side is, and a cut from it is a whole
/// meridian up to the far pole.
///
/// The loop's longitudes are the walk's own, unwrapped, and the closing side
/// runs to the first corner a whole `climbed` on.
fn with_a_corner_where_a_pole_side_passes(
    cap: &WalkedLoop,
    quarters: &[f64],
    pinch: f64,
) -> WalkedLoop {
    let count = cap.corners.len();
    let mut opened = WalkedLoop {
        corners: Vec::with_capacity(count + quarters.len()),
        points: Vec::with_capacity(count + quarters.len()),
        ..cap.clone()
    };
    for at in 0..count {
        let here = cap.corners[at];
        opened.corners.push(here);
        opened.points.push(cap.points[at]);
        let onwards = (at + 1) % count;
        let mut there = cap.corners[onwards];
        if onwards == 0 {
            there[0] += cap.climbed;
        }
        let on_a_pole = here[1] == pinch || here[1] == -pinch;
        if !on_a_pole || cap.points[at] != cap.points[onwards] || here[1] != there[1] {
            continue;
        }
        let mut passed: Vec<f64> = quarters
            .iter()
            .copied()
            .filter(|&quarter| passes_strictly(quarter, here[0], there[0]))
            .collect();
        passed.sort_by(|one, other| (one - here[0]).abs().total_cmp(&(other - here[0]).abs()));
        for quarter in passed {
            opened.corners.push([quarter, here[1]]);
            opened.points.push(cap.points[at]);
        }
    }
    opened
}

/// Whether `at` lies strictly between `one` and `other`, whichever is larger.
fn passes_strictly(at: f64, one: f64, other: f64) -> bool {
    one.min(other) < at && at < one.max(other)
}

/// Which corner of a cap sits nearest a longitude, and of two as near, the one
/// off the pole.
///
/// The end of a pole side shares its longitude with the rim corner beside it,
/// because the rim leaves the pole up that meridian, and a cut from the pole
/// end would run along the rim's own step (see [`starting_off_the_poles`]).
fn nearest_corner_off_the_rims_poles(cap: &WalkedLoop, at: f64, pinch: f64) -> usize {
    let on_a_pole = |which: usize| {
        let latitude = cap.corners[which][1];
        latitude == pinch || latitude == -pinch
    };
    (0..cap.corners.len())
        .min_by(|&one, &other| {
            let away = |which: usize| (cap.corners[which][0] - at).abs();
            away(one)
                .total_cmp(&away(other))
                .then(on_a_pole(one).cmp(&on_a_pole(other)))
        })
        .expect("a walked loop has a corner")
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

/// A longitude moved by whole turns into the turn a loop's own corners run in,
/// so that [`nearest_longitude`] answers a corner of the stretch it names rather
/// than whichever end of the loop is closest.
///
/// A loop that goes right round spans one whole turn, so every longitude has
/// exactly one place in it and the answer is never a choice. A longitude already
/// in that turn comes back byte for byte, which is what keeps a ring the mesher
/// already cut meshing exactly as it did.
fn in_the_turn_of(around: &WalkedLoop, at: f64, whole: f64) -> f64 {
    let least = least_longitude(around);
    at - ((at - least) / whole).floor() * whole
}

/// How many whole turns of longitude have to come off `at` to bring it within
/// half a turn of `of`, as an angle.
fn whole_turns_between(at: f64, of: f64, whole: f64) -> f64 {
    ((at - of) / whole).round() * whole
}

/// One loop's longitudes moved by whole turns into the frame another was walked
/// in.
fn in_the_same_turn_as(chain: &WalkedLoop, frame: &WalkedLoop, whole: f64) -> WalkedLoop {
    let by = ((least_longitude(frame) - least_longitude(chain)) / whole).round() * whole;
    WalkedLoop {
        corners: chain
            .corners
            .iter()
            .map(|corner| [corner[0] + by, corner[1]])
            .collect(),
        points: chain.points.clone(),
        closes_on_one_curve: chain.closes_on_one_curve,
        goes_right_round: chain.goes_right_round,
        climbed: chain.climbed,
    }
}

/// One loop moved by whole turns of its second coordinate, so that the middle
/// of the span it covers is within half a turn of the middle of `frame`'s.
///
/// **Each loop of a torus face is unwrapped round the ring from its own first
/// corner**, so two loops of one face can come out a whole turn apart, which in
/// the chart puts a hole outside the loop it is a hole in. A hole's ring angles
/// lie inside its outer loop's, so the middle of a hole is at most half the
/// outer loop's span from the outer loop's middle. An outer loop spanning less
/// than a whole turn round the ring therefore has its holes less than half a
/// turn from its middle in exactly one turn, and this finds it whichever of the
/// two is the face's first wire. A loop already in that turn keeps its numbers
/// bit for bit.
///
/// **What this does not settle is two loops side by side**, neither inside the
/// other, such as two circles round the tube. They bound two bands round the
/// ring, and this puts them the nearer way round, which is only the right band
/// when the face is the narrower one. The loops' directions say which it is,
/// and nothing here reads them.
fn with_its_second_coordinate_in_the_turn_of(
    chain: &WalkedLoop,
    frame: &WalkedLoop,
    whole: f64,
) -> WalkedLoop {
    let middle = |around: &WalkedLoop| {
        let (least, most) = around
            .corners
            .iter()
            .fold((f64::MAX, f64::MIN), |(least, most), corner| {
                (least.min(corner[1]), most.max(corner[1]))
            });
        (least + most) / 2.0
    };
    let by = whole_turns_between(middle(chain), middle(frame), whole);
    WalkedLoop {
        corners: chain
            .corners
            .iter()
            .map(|corner| [corner[0], corner[1] - by])
            .collect(),
        ..chain.clone()
    }
}

/// The lowest longitude any corner of a loop sits at.
fn least_longitude(around: &WalkedLoop) -> f64 {
    around
        .corners
        .iter()
        .map(|corner| corner[0])
        .fold(f64::MAX, f64::min)
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
        climbed: chain.climbed,
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
        climbed: 0.0,
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

/// The corner a slit ends at inside the face: the one whose two neighbours are
/// the same point and which is not that point itself.
///
/// A pole side is two corners at one point, but its neighbours are two
/// different points, so it is never taken for one.
fn the_inner_end_of_a_slit(around: &WalkedLoop) -> Option<usize> {
    let count = around.points.len();
    if count < 3 {
        return None;
    }
    (0..count).find(|&at| {
        let before = around.points[(at + count - 1) % count];
        let after = around.points[(at + 1) % count];
        before == after && around.points[at] != after
    })
}

/// Where the line carrying on from a slit's inner end at `tip` meets the loop:
/// the loop with that end put into it where it lands on a pole side, and the
/// positions of the tip and of that end.
///
/// The line runs along the tip's own longitude, away from the latitude the
/// slit leaves by, to the nearest side it reaches. It ends where [`where_to_cut`]
/// would end it: in the middle of a pole side, or else at the nearer corner of
/// the side it crosses.
///
/// `None` when the slit leaves along the tip's own latitude, when the line
/// reaches nothing, or when it reaches the slit again.
fn where_a_slit_carries_on_to(around: &WalkedLoop, tip: usize) -> Option<(WalkedLoop, [usize; 2])> {
    let count = around.corners.len();
    let [longitude, latitude] = around.corners[tip];
    let slit_leaves_by = around.corners[(tip + 1) % count][1] - latitude;
    if slit_leaves_by == 0.0 {
        return None;
    }
    let mut on_the_slit = HashSet::from([around.points[tip]]);
    let mut out = 1;
    while 2 * out < count
        && around.points[(tip + out) % count] == around.points[(tip + count - out) % count]
    {
        on_the_slit.insert(around.points[(tip + out) % count]);
        out += 1;
    }

    let mut nearest: Option<(f64, usize, f64)> = None;
    for at in 0..count {
        let onwards = (at + 1) % count;
        if at == tip || onwards == tip {
            continue;
        }
        let (here, next) = (around.corners[at], around.corners[onwards]);
        if !passes(longitude, here[0], next[0]) {
            continue;
        }
        let crossing = if here[0] == next[0] {
            if (here[1] - latitude).abs() <= (next[1] - latitude).abs() {
                here[1]
            } else {
                next[1]
            }
        } else {
            here[1] + (longitude - here[0]) / (next[0] - here[0]) * (next[1] - here[1])
        };
        let away = (latitude - crossing) * slit_leaves_by.signum();
        if away > 0.0 && nearest.is_none_or(|(least, _, _)| away < least) {
            nearest = Some((away, at, crossing));
        }
    }
    let (_, at, crossing) = nearest?;
    let onwards = (at + 1) % count;
    let (here, next) = (around.corners[at], around.corners[onwards]);

    let mut opened = around.clone();
    let mut tip = tip;
    let far = if here == [longitude, crossing] {
        at
    } else if next == [longitude, crossing] {
        onwards
    } else if around.points[at] == around.points[onwards] {
        opened.corners.insert(at + 1, [longitude, crossing]);
        opened.points.insert(at + 1, around.points[at]);
        if tip > at {
            tip += 1;
        }
        at + 1
    } else if the_nearer_of(longitude, here[0], next[0]) {
        at
    } else {
        onwards
    };
    if on_the_slit.contains(&opened.points[far]) {
        return None;
    }
    Some((opened, [tip, far]))
}

/// The corner of a loop furthest along latitude the way `way`'s sign says: the
/// highest for `1.0`, the lowest for `-1.0`. The first of them on a tie.
fn the_corner_furthest(around: &WalkedLoop, way: f64) -> usize {
    (0..around.corners.len())
        .reduce(|best, at| {
            if way * around.corners[at][1] > way * around.corners[best][1] {
                at
            } else {
                best
            }
        })
        .expect("a walked loop has a corner")
}

/// Where the line along `longitude` from `latitude`, going the way `way`'s sign
/// says, first meets `around`: the side it meets and the latitude it meets it
/// at. `None` unless that side is a **pole side** — two corners at one point —
/// met strictly beyond `latitude`, or when a side of `around` lies along the
/// line itself, where nothing says which of its points is first.
fn the_pole_side_a_line_from_a_hole_reaches(
    around: &WalkedLoop,
    longitude: f64,
    latitude: f64,
    way: f64,
) -> Option<(usize, f64)> {
    let count = around.corners.len();
    let mut nearest: Option<(f64, usize, f64)> = None;
    for at in 0..count {
        let onwards = (at + 1) % count;
        let (here, next) = (around.corners[at], around.corners[onwards]);
        if !passes(longitude, here[0], next[0]) {
            continue;
        }
        if here[0] == next[0] {
            return None;
        }
        let crossing = here[1] + (longitude - here[0]) / (next[0] - here[0]) * (next[1] - here[1]);
        let away = (crossing - latitude) * way;
        if away > 0.0 && nearest.is_none_or(|(least, _, _)| away < least) {
            nearest = Some((away, at, crossing));
        }
    }
    let (_, at, crossing) = nearest?;
    (around.points[at] == around.points[(at + 1) % count]).then_some((at, crossing))
}

/// The running state of a walk round one wire.
struct WalkingAWire {
    wraps: Option<f64>,
    /// How far the second coordinate runs before it comes back, which only a
    /// torus's does: it is the angle round the ring. See
    /// [`how_far_the_second_coordinate_wraps`].
    second_wraps: Option<f64>,
    /// Whether this wire is one whole closed curve, so that it is allowed to
    /// come back to where it started a whole turn on rather than at the
    /// longitude it left. Whether it *did* is measured in
    /// [`WalkingAWire::finish`] and is a different question.
    closes_on_one_curve: bool,
    /// Whether no edge of this wire is used twice, so that none of its sides is
    /// a seam. Part of [`WalkingAWire::may_come_back_a_whole_turn_on`]; the rest
    /// is measured as the walk goes.
    no_side_of_this_wire_is_a_seam: bool,
    /// Whether the walk ever stood at a pole — a pole step, a corner on one, or
    /// a point along an edge that passes one.
    stood_at_a_pole: bool,
    longitude: f64,
    started: bool,
    will_not_lie_flat: bool,
    /// Which way the longitude runs along a pole side the walk has crossed and
    /// whose far end is not known yet, because it is wherever the next corner
    /// with a longitude of its own sits. Only a pole between two **different**
    /// meridians leaves one; a seam turning back is a whole turn and settled on
    /// the spot.
    crossing: Option<f64>,
    /// The corner at a pole the walk has arrived at and not yet left: the entry
    /// in the point table and the latitude, waiting for the longitude the walk
    /// leaves by. A pole is a whole side of the patch, so it needs a corner at
    /// each end of it, and only the first is known when the walk gets there.
    still_at_a_pole: Option<(u32, f64)>,
    /// Whether the first corner is at a pole, so that the loop closes over a
    /// pole side whose near end the walk only reaches when it ends.
    set_off_at_a_pole: bool,
    corners: Vec<[f64; 2]>,
    points: Vec<u32>,
}

impl WalkingAWire {
    fn new(
        wraps: Option<f64>,
        second_wraps: Option<f64>,
        closes_on_one_curve: bool,
        no_side_of_this_wire_is_a_seam: bool,
    ) -> Self {
        Self {
            wraps,
            second_wraps,
            closes_on_one_curve,
            no_side_of_this_wire_is_a_seam,
            stood_at_a_pole: false,
            longitude: 0.0,
            started: false,
            will_not_lie_flat: false,
            crossing: None,
            still_at_a_pole: None,
            set_off_at_a_pole: false,
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
    ///
    /// **On a torus the second coordinate is unwrapped too**, and for the same
    /// reason. It is the angle round the ring, which `atan2` answers in
    /// `(−π, π]` as well, and a wall whose ring angles run past `π` was read as
    /// one corner at `+π` and the rest near `−π`: the other three quarters of
    /// the ring, with the seam folded back on itself. The builder refused a
    /// quarter bend turned half a turn for it (§280).
    fn arrive(&mut self, at: u32, sits_at: (f64, f64), is_a_pole: bool) {
        let (raw, up) = sits_at;
        let up = self.in_the_turn_of_the_last_corner(up);
        if is_a_pole {
            self.stood_at_a_pole = true;
            if self.corners.is_empty() {
                self.set_off_at_a_pole = true;
            }
            // Past a pole side whose far end is still open, this is that far
            // end, and its longitude is the next corner's: `leave_the_pole_at`
            // puts it there.
            if self.crossing.is_none() {
                self.corners.push([self.longitude, up]);
                self.points.push(at);
            }
            self.still_at_a_pole = Some((at, up));
            return;
        }
        let longitude = match self.wraps {
            None => raw,
            // Everything walked so far was a pole and had no longitude of its
            // own, so this corner's raw one is the first fact there is. The
            // turns `cross_the_pole` counted between those poles are still
            // real: shift them all by the same amount rather than drop them.
            //
            // A pole side still open here has no near end to measure from, and
            // `steps_from_somewhere_that_has_a_longitude` never opens a walk on
            // a pole step, so only an edge with no points strictly along it gets
            // here: refused rather than guessed.
            Some(_) if !self.started => {
                if self.crossing.take().is_some() {
                    self.will_not_lie_flat = true;
                }
                let by = raw - self.longitude;
                for corner in &mut self.corners {
                    corner[0] += by;
                }
                raw
            }
            Some(whole) => match self.crossing.take() {
                Some(way) => {
                    self.longitude
                        + way * how_far_a_pole_side_runs(way * (raw - self.longitude), whole)
                }
                None => self.longitude + the_short_way_round(raw - self.longitude, whole),
            },
        };
        self.started = true;
        self.longitude = longitude;
        self.leave_the_pole_at(longitude);
        self.corners.push([longitude, up]);
        self.points.push(at);
    }

    /// The second coordinate moved by whole turns to within half a turn of the
    /// corner before it, where that coordinate wraps; as it came everywhere
    /// else, and at the first corner, which is what the rest are measured from.
    ///
    /// **The turns are taken off rather than the difference added back on**, so
    /// a corner that needs no turn keeps its number bit for bit, and a wire
    /// that never crosses `π` round the ring meshes exactly as it did.
    fn in_the_turn_of_the_last_corner(&self, up: f64) -> f64 {
        match (self.second_wraps, self.corners.last()) {
            (Some(whole), Some(last)) => up - whole_turns_between(up, last[1], whole),
            (Some(_), None) | (None, _) => up,
        }
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
    /// surface's coordinates, running from the meridian the walk arrived down to
    /// the meridian it leaves by.
    ///
    /// **How far that side runs is how far apart the two meridians are, and a
    /// whole turn only when they are one.** A ball's seam arrives at a pole and
    /// turns straight back up itself, so its pole side is a whole turn, and that
    /// is settled here. A **lune** — a sphere face between two different
    /// half-meridians, which is the skin of a quarter of a ball — arrives down
    /// one and leaves up the other, and its pole side is the angle between
    /// them. Charging a whole turn there too made a quarter's skin a turn and a
    /// quarter wide and its volume `5.209` where `π/3` is the truth (§166). The
    /// far end is left open until the next corner with a longitude is walked.
    ///
    /// **Which way the side runs is read off where the walk came from.** A
    /// pinching surface's two coordinates are right-handed about its normal and
    /// a wire keeps its face on its left, so a patch is walked anticlockwise:
    /// arriving at a pole from above, the pole is the bottom of the patch and
    /// its side runs with the longitude; from below, the top, and against it.
    /// Alternating the sign from the first pole crossed, which is what stood
    /// here, is the same thing only for a walk whose first pole is a bottom one.
    ///
    /// A flat surface pinches to nothing, so a pole step on one is a wire that
    /// will not lie flat, and [`WalkingAWire::finish`] says so — as is a pole
    /// the walk arrived at without ever having been off it.
    fn cross_the_pole(&mut self, a_seam_turns_back_here: bool) {
        let Some(whole) = self.wraps else {
            self.will_not_lie_flat = true;
            return;
        };
        self.still_at_a_pole = None;
        let Some(way) = self.which_way_the_pole_side_runs() else {
            self.will_not_lie_flat = true;
            return;
        };
        if a_seam_turns_back_here {
            self.longitude += way * whole;
        } else {
            self.crossing = Some(way);
        }
    }

    /// Open the pole side at an ordinary corner of the wire that sits on a
    /// pole, so it runs the way the face lies and not the short way round.
    ///
    /// **The short way round is a guess, and at half a turn it is a coin that
    /// lands wrong.** A ball's equator circle read in a chart whose axis lies
    /// in the circle's plane is two meridians half a turn apart, and a face
    /// bounded by it has a corner at each pole. `the_short_way_round` answers
    /// a jump of exactly `π` with `−π` — `round` goes away from zero — so the
    /// south pole side of `kicad/LED_D3.0mm_Horizontal_O3.81mm_Z6.0mm.step`'s
    /// dome ran from `−π` to `−2π` instead of to `0`, and the walk came back a
    /// whole turn from where it set off. The rule `cross_the_pole` uses does
    /// not guess: a wire keeps its face on its left, so a pole below the walk
    /// is crossed with the longitude and one above it against. A pole side
    /// wider than half a turn, which the short way can never give, comes out
    /// right the same way.
    ///
    /// Nothing to do before the walk has a longitude of its own: the far end
    /// is then where the walk set off, and [`WalkingAWire::finish`] closes it.
    fn turn_at_a_corner_on_a_pole(&mut self) {
        if !self.started || self.wraps.is_none() {
            return;
        }
        let Some(way) = self.which_way_the_pole_side_runs() else {
            self.will_not_lie_flat = true;
            return;
        };
        self.crossing = Some(way);
    }

    /// `1` when the pole just arrived at is below everything walked before it
    /// reached it, and `-1` when it is above.
    fn which_way_the_pole_side_runs(&self) -> Option<f64> {
        let pole = self.corners.last()?[1];
        let came_from = self.corners.iter().rev().find(|corner| corner[1] != pole)?;
        Some(if came_from[1] > pole { 1.0 } else { -1.0 })
    }

    /// The loop, or `None` if it will not lie flat: the walk stood still at a
    /// pole on a surface that does not pinch to one, or it wound round the
    /// surface when this wire is not one that may.
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
            if self.crossing.is_none() {
                self.longitude = first[0];
            }
        }
        // A pole side still open at the end closes on the corner the walk set
        // off from, which is where the loop closes; its far end is that corner.
        if let (Some(whole), Some(way), Some(first)) =
            (self.wraps, self.crossing.take(), self.corners.first())
        {
            self.longitude +=
                way * how_far_a_pole_side_runs(way * (first[0] - self.longitude), whole);
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
        //
        // **It is measured on the walk's own longitude, and not by summing the
        // corners the short way round.** A pole side runs as far as
        // `cross_the_pole` said, and between two meridians half a turn apart
        // that is a half turn, which the short way reads as a coin toss decided
        // by the last bit of each meridian. A hemisphere bounded by a circle
        // through both poles crosses two such sides, one each way: read the
        // short way, `π` from `−1.8925468811915387` to `1.2490457723982544`
        // came out `−π` and the `−3.1415926535897927` back stayed `−π`, so the
        // lune was called a cap wound right round and refused as one with no
        // seam. Only the closing step is left to the short way, and it is an
        // ordinary step of the walk.
        //
        // **And it is the same number the refusal is read off.** The walk's own
        // gain stops one step short of the loop — the step back onto the first
        // corner is the closing one — so a ring of three arcs round a cone's
        // apex came back `−5.235987755982988` where a whole turn is `6.283…`,
        // and a check on the gain alone read that as a wire that had not come
        // back to the longitude it set off from. With the closing step counted
        // it is `−6.283185307179586` exactly: once right round. What is left
        // over after the whole turns are taken out is always the closing step,
        // never the error, so there is nothing else here to threshold.
        let (climbed, goes_right_round) = match (self.wraps, self.corners.first()) {
            (Some(whole), Some(first)) => {
                let climbed = self.longitude - first[0]
                    + the_short_way_round(first[0] - self.longitude, whole);
                let went_round = climbed.abs() >= whole / 2.0;
                if went_round && !self.closes_on_one_curve && !self.may_come_back_a_whole_turn_on()
                {
                    return None;
                }
                (climbed, went_round)
            }
            _ => (0.0, false),
        };
        if let Some(whole) = self.second_wraps
            && self.climbs_a_whole_turn_the_second_way(whole)
        {
            return None;
        }
        Some(WalkedLoop {
            corners: self.corners,
            points: self.points,
            closes_on_one_curve: self.closes_on_one_curve,
            goes_right_round,
            climbed,
        })
    }

    /// Whether the loop comes back a whole turn on in its second coordinate,
    /// which on a torus is a wire right round the ring with no seam across the
    /// tube.
    ///
    /// **This mesher has no cut for that, so it is refused and not meshed.**
    /// A loop in the chart has to close, and the one place a wire may come
    /// back a whole turn on is round the tube, where
    /// [`WalkingAWire::may_come_back_a_whole_turn_on`] says which wires may and
    /// [`BuildingAMesh::cut_off_what_goes_too_far_round`] cuts what they bound.
    /// Nothing does that round the ring. A face right round the ring **with**
    /// a seam across the tube climbs nothing, goes round and comes back, and is
    /// meshed as a patch a whole turn wide.
    ///
    /// The climb is measured the way [`WalkingAWire::finish`] measures the
    /// longitude's, with the closing step taken the short way round.
    fn climbs_a_whole_turn_the_second_way(&self, whole: f64) -> bool {
        let (Some(first), Some(last)) = (self.corners.first(), self.corners.last()) else {
            return false;
        };
        let climbed = last[1] - first[1] + the_short_way_round(first[1] - last[1], whole);
        climbed.abs() >= whole / 2.0
    }

    /// Whether this wire is allowed to end a whole turn of longitude on from
    /// where it set off — a **ring**, one simple closed curve drawn on the
    /// surface, rather than a boundary that has to close in the chart.
    ///
    /// A wire that uses an edge twice uses it as a **seam**: the two sides of
    /// one meridian, which the patch lies between, so the walk down the far
    /// side has to bring the longitude back to where the walk up the near side
    /// left it. A wire that stands at a **pole** has a side of the patch whose
    /// width came from [`WalkingAWire::cross_the_pole`] rather than from a step
    /// of a curve, and a wire through a pole is not a wire round one anyway.
    /// Neither of those may wind, and that is what this keeps out.
    ///
    /// **A ring is a cap only when it is the face's one wire**, and where it is
    /// not, [`BuildingAMesh::cut_off_what_goes_too_far_round`] is what says
    /// which shape the face is. A face on a surface that wraps whose wires
    /// *all* go right round is a band between the two of them and nothing else,
    /// because the loops cut the chart into stretches and only the one between
    /// them has both in its boundary — that is
    /// [`BuildingAMesh::cut_a_ring_across`]'s shape, and a frustum's band
    /// between its own near cap circle and a ring of arcs from a cube's upper
    /// planes is one. A face with a wire that winds beside one that does not
    /// is the shape nobody has an answer for, and it is refused there rather
    /// than here.
    ///
    /// **What is left may, and a cone is where it has to.** A pointed cone
    /// inside a cube stood on its corner keeps a piece of nappe round the apex
    /// bounded by three arcs, one from each of the cube's upper planes, meeting
    /// at three corners off the apex. Three distinct edges, no seam, no pole
    /// step and no corner on the apex — so the ring closes right round the
    /// longitude, and the face is the piece of the nappe the apex is in.
    /// [`BuildingAMesh::cut_a_cap_up_to_its_pole`] is what fills it, and
    /// [`BuildingAMesh::which_pole_this_cap_reaches`] is what says which of the
    /// two pieces the ring bounds is the face: the step the wire takes in space
    /// against `∇S × ∇T` says which side of the cutting surface the material is
    /// on, and the apex either is on that side or the face is the unbounded
    /// rest of the nappe, which is no face at all and stays refused.
    ///
    /// One coedge whose two ends are its one corner is a ring too, and it is
    /// the one this kernel already meshed: `closes_on_one_curve` says it, the
    /// caller asks it first, and it needs none of the reasoning above.
    fn may_come_back_a_whole_turn_on(&self) -> bool {
        self.no_side_of_this_wire_is_a_seam && !self.stood_at_a_pole
    }

    /// Cross the pole side the loop closes over, when the walk set off from a
    /// corner at a pole and came back to it down a different meridian.
    ///
    /// **A lune between two meridians half a turn apart is the case that
    /// needs it.** A plane down a ball's polar axis cuts it into two halves
    /// whose wire is two half-meridians, both of whose corners are poles, so
    /// the walk can only set off from one. The pole side the walk crosses in
    /// the middle gets a corner at each end from [`WalkingAWire::arrive`]; the
    /// one it set off from was never arrived at, so it had no near corner and
    /// the walk ended half a turn from where it began, which the check in
    /// [`WalkingAWire::finish`] refused (§194.5). This is that arrival: a
    /// second corner at the same point, at the longitude the walk comes back
    /// by, and the side crossed the way [`WalkingAWire::cross_the_pole`] reads
    /// it, which brings the longitude back onto the first corner's for a loop
    /// that lies flat and a whole turn off for one that does not.
    ///
    /// Nothing to do when the walk already stands on that pole, which is a
    /// ball's own wire closing its last pole step, or when it came back down
    /// the meridian it left by.
    ///
    /// **Not asked of a seam that turns back at the pole with no pole step
    /// there.** That is a ball's wire with its last pole step taken away, and
    /// read as a pole side it would be a whole turn and walk as a whole ball.
    /// A pole side a whole turn wide is what a pole step is written for, so
    /// the wire without one stays refused, as
    /// `a_wire_that_will_not_lie_flat_is_refused_rather_than_asserted_against`
    /// holds; the caller asks
    /// [`the_walk_comes_back_up_the_edge_it_set_off_along`] first.
    fn arrive_back_at_the_pole_the_walk_set_off_from(&mut self) {
        let (Some(whole), Some(&first), Some(&last)) =
            (self.wraps, self.corners.first(), self.corners.last())
        else {
            return;
        };
        if !self.set_off_at_a_pole
            || self.crossing.is_some()
            || self.still_at_a_pole.is_some()
            || last[1] == first[1]
            || self.longitude == first[0]
        {
            return;
        }
        self.corners.push([self.longitude, first[1]]);
        self.points.push(self.points[0]);
        let way = self.which_way_the_pole_side_runs().expect(
            "the corner before the pole just written is off that pole's latitude, so it says \
             which way the side runs",
        );
        self.longitude += way * how_far_a_pole_side_runs(way * (first[0] - self.longitude), whole);
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
///
/// The fuse asks it too, before it decides whether a circle that starts here
/// needs cutting in two, so that a corner the mesh will call a pole is never
/// left the one corner of a closed edge (§195.2).
pub(crate) fn the_surface_pinches_at(surface: &Surface, latitude: f64) -> bool {
    where_this_surface_pinches(surface).is_some_and(|pinch| latitude.abs() == pinch)
}

/// The steps of `wire`, turned so the walk sets off from a corner that has a
/// longitude, when the wire has one to set off from — and otherwise along an
/// edge rather than on a pole step.
///
/// A lune's wire has no corner off a pole, and opened on a pole step it would
/// cross a pole side before it had walked anywhere to say which way the side
/// runs or where its near end is.
fn steps_from_somewhere_that_has_a_longitude<'m>(
    wire: WireView<'m>,
    at_a_pole: &impl Fn(&VertexView<'m>) -> bool,
) -> Vec<WireStepView<'m>> {
    let mut steps: Vec<WireStepView<'m>> = wire.steps().collect();
    let away = steps
        .iter()
        .position(|step| match step {
            WireStepView::Along(coedge) => !at_a_pole(&coedge.start()),
            WireStepView::Pole(_) => false,
        })
        .or_else(|| {
            steps
                .iter()
                .position(|step| matches!(step, WireStepView::Along(_)))
        });
    if let Some(away) = away {
        steps.rotate_left(away);
    }
    steps
}

/// Whether the pole step at `index` sits between two uses of one edge, which
/// is a seam arriving at the pole and going straight back up itself.
fn a_seam_turns_back_at(steps: &[WireStepView<'_>], index: usize) -> bool {
    let edge_of = |step: &WireStepView<'_>| match step {
        WireStepView::Along(coedge) => Some(coedge.edge().key()),
        WireStepView::Pole(_) => None,
    };
    let before = edge_of(&steps[(index + steps.len() - 1) % steps.len()]);
    let after = edge_of(&steps[(index + 1) % steps.len()]);
    before.is_some() && before == after
}

/// Whether the walk's last step and its first run along one edge, so that the
/// corner the loop closes on is a seam turning back.
///
/// **A wire of one step is not one**: it is a closed edge, walked once, whose
/// one corner is where it set off and where it ends. When that corner sits at
/// a pole the loop closes over a pole side like any other.
fn the_walk_comes_back_up_the_edge_it_set_off_along(steps: &[WireStepView<'_>]) -> bool {
    if steps.len() == 1 {
        return false;
    }
    match (steps.first(), steps.last()) {
        (Some(WireStepView::Along(first)), Some(WireStepView::Along(last))) => {
            first.edge().key() == last.edge().key()
        }
        _ => false,
    }
}

/// How far a pole side between two different meridians runs, given how far
/// apart they are taken the way it runs: that distance brought into
/// `(0, whole]`.
///
/// No turn at all is read as a whole one, because two different edges that
/// meet at a pole on one meridian bound either nothing or the whole surface,
/// and a face of nothing is not a face.
fn how_far_a_pole_side_runs(apart: f64, whole: f64) -> f64 {
    let runs = apart.rem_euclid(whole);
    if runs == 0.0 { whole } else { runs }
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

/// Whether no curved face along `edge` will place it and its two ends are not
/// the whole of it — so a flat face has to.
///
/// # Panics
///
/// If `edge` has no face along it at all. It was reached through a face, and
/// [`SolidView::surfaces_along_each_edge`] lists every face's every edge.
fn only_a_flat_face_can_place(
    edge: EdgeView<'_>,
    along_each_edge: &HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
) -> bool {
    let sides = along_each_edge
        .get(&edge.key())
        .expect("an edge reached through a face has that face along it");
    *edge.stored_run() != EdgeRun::Straight
        && sides.iter().all(|side| side.stored.as_plane().is_some())
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

/// Which way a closed edge runs where its two faces touch at its corner rather
/// than cross there: the way the longitude of the first curved surface either
/// side that has one there increases.
///
/// [`the_way_a_closed_edge_is_stored_running`] has nothing to read there, because
/// `∇S × ∇T` has no length, and leaving the points in whatever order the
/// marcher produced them is what refused the body of
/// `LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm` (`docs/decisions.md` §189.3): the wire
/// walk climbed two whole turns instead of none.
///
/// **The first surface in `Surface`'s own order is not always the one**, and on
/// that LED it is not: the ball sorts before the pipe, and the corner is the
/// ball's pole, where its longitude is exactly zero. The STEP reader then has no
/// winding to orient the edge's two uses by either, and takes the file's own
/// `CIRCLE`, whose axis is the pipe's. So the pipe's longitude is the way the
/// wire was written, and a surface further down the order is asked where the
/// first says nothing.
///
/// **This is a hint that is checked, not a decision.** A wire whose points run
/// the wrong way round climbs two whole turns where it should climb none, and
/// the mesher refuses the face by `AWireWillNotLieFlatOnItsFace` rather than
/// drawing it.
///
/// `None` where no curved surface either side has a longitude at the corner.
fn the_way_a_closed_edge_where_its_faces_touch_runs(
    along: &[Surface; 2],
    at: Point,
) -> Option<Vector> {
    let mut curved: Vec<Surface> = along
        .iter()
        .filter(|surface| surface.as_plane().is_none())
        .copied()
        .collect();
    curved.sort_by_key(name_of_surface_bits);
    curved.into_iter().find_map(|surface| {
        let longitude =
            the_way_it_is_parametrised_about(&surface, at).cross(gradient_of(&surface, at));
        (longitude != Vector::zero()).then_some(longitude)
    })
}

/// Make every pole of a ball that a closed crossing runs through one of the
/// points walked along it, and say how far a walked point was moved to do it.
///
/// **The wire walk knows a pole only when a walked point lands on it**
/// ([`BuildingAMesh::walk_a_wire_round`]). A plane through a ball's axis cuts
/// it in a circle through both poles, and a walk that steps past a pole
/// between two points jumps half a turn of longitude with nothing to say which
/// way. Whether it landed on the pole was luck: at one deflection the walk of
/// `a_rim_through_both_poles_measures.rs` did, and at every finer one it did
/// not (§193.3).
///
/// The crossing is the whole closed curve, so a pole is on it exactly when
/// both surfaces hold the pole — two certified predicates and no distance
/// compared with anything. A circle between a ball and a plane that misses
/// the pole by one rounding or less is given it too, and the miss is charged —
/// see [`how_far_a_circle_that_passes_within_a_rounding_of_a_pole_misses_it`].
/// Any other pole whose coordinates round off the ball, or whose sides cannot
/// be decided, is left out, and the walk then goes on as it did before this
/// existed: [`WalkingAWire::finish`] checks it.
fn put_in_the_poles_a_closed_crossing_passes(
    points: &mut Vec<Point>,
    ends: [Point; 2],
    along: &[Surface; 2],
) -> f64 {
    let mut moved: f64 = 0.0;
    for (index, ball) in along.iter().enumerate() {
        let Surface::Sphere(sphere) = ball else {
            continue;
        };
        let other = &along[1 - index];
        if other == ball {
            continue;
        }
        let pinch = where_this_surface_pinches(ball).expect("a ball pinches at its poles");
        for up in [1.0, -1.0] {
            let pole = sphere.centre() + Vector::new(0.0, 0.0, up * sphere.radius());
            if let Some(far) = put_in_a_pole(points, ends, [ball, other], pole, up * pinch) {
                moved = moved.max(far);
            }
        }
    }
    moved
}

/// Put `pole` among `points`, between the two it lies between, when both
/// surfaces certainly hold it or the crossing passes within one rounding of
/// it; `None` when neither, and otherwise how far the mesh was moved to put it
/// there.
///
/// **Between which two is read off the crossing's own direction at the
/// pole.** The plane square to `∇S × ∇T` through the pole cuts a closed
/// crossing twice, at the pole and somewhere across the curve from it, so the
/// walked points change side of it twice; the pair nearer the pole is the
/// one. The two sums of distances compared are about one step against about
/// the curve's width, so which is smaller is not close.
///
/// **A walked point that already reads as this pole is moved onto it** rather
/// than given a twin beside it. That is what the wire walk calls a pole —
/// [`the_surface_pinches_at`]'s equality on the latitude, or one rounding in
/// space ([`the_pole_a_point_sits_within_a_rounding_of`]) — so nothing is
/// called the pole here that the wire walk would not already call the pole; a
/// twin would open the pole side twice. A corner that reads as the pole is left
/// alone, since the wire walk already turns there, and a twin beside it would
/// be one rounding away, which the flat face across the edge reads as the same
/// corner.
fn put_in_a_pole(
    points: &mut Vec<Point>,
    ends: [Point; 2],
    [ball, other]: [&Surface; 2],
    pole: Point,
    latitude: f64,
) -> Option<f64> {
    let exactly = PredicatePoint::from(pole.coordinates());
    let holds_it = |surface: &Surface| match side_of_surface(surface, exactly) {
        Decision::Zero => true,
        // An exact point is undecidable only past overflow or into the
        // subnormals; the walk is then left as it was, and checked.
        Decision::Pos | Decision::Neg | Decision::Undecidable { .. } => false,
        _ => false,
    };
    let reads_as_this_pole = |at: Point| {
        ball.coordinates_of(at).1 == latitude
            || the_pole_a_point_sits_within_a_rounding_of(ball, at) == Some(latitude)
    };
    let missed_by = if holds_it(ball) && holds_it(other) {
        0.0
    } else {
        how_far_a_circle_that_passes_within_a_rounding_of_a_pole_misses_it([ball, other], pole)?
    };
    if ends.into_iter().any(reads_as_this_pole) {
        return None;
    }
    let forward = gradient_of(ball, pole).cross(gradient_of(other, pole));
    if forward == Vector::zero() {
        return None;
    }

    let every: Vec<Point> = core::iter::once(ends[0])
        .chain(points.iter().copied())
        .chain(core::iter::once(ends[1]))
        .collect();
    let ahead = |at: Point| (at - pole).dot(forward) > 0.0;
    let how_near = |pair: &[Point]| (pair[0] - pole).length() + (pair[1] - pole).length();
    let gap = every
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| ahead(pair[0]) != ahead(pair[1]))
        .min_by(|(_, one), (_, other)| how_near(one).total_cmp(&how_near(other)))
        .map(|(gap, _)| gap)?;

    let walked_either_side = [gap.checked_sub(1), (gap < points.len()).then_some(gap)];
    for beside in walked_either_side.into_iter().flatten() {
        if reads_as_this_pole(points[beside]) {
            let far = (points[beside] - pole).length();
            points[beside] = pole;
            return Some(far.max(missed_by));
        }
    }
    points.insert(gap, pole);
    Some(missed_by)
}

/// How far a circle a ball and a plane cross in passes from `pole`, when the
/// point of it nearest the pole is within one rounding of the pole; `None`
/// when it is not, or when the pair is not a ball and a plane.
///
/// **A plane meant to hold the ball's axis almost never does.** Its stored
/// normal and offset are rounded turns of whole numbers, so the exact circle
/// misses both poles by about `1e-17` and neither is certified on it. Left
/// out, the walk steps past each pole in about half a turn of longitude, and
/// whether that is `+π` or `−π` is decided by which side of the pole the
/// rounding put the circle: the ball and block of side four turned a quarter
/// turn about `(−1, −3, 0)` came back wound right round and was refused as a
/// face with no seam; about `(1, 3, 0)` too; and about `(3, −2, 0)` one pole
/// read as on the circle by `atan2`'s equality and the other, four times as far
/// off, did not, so the half ball came out a cap round one pole with its rim
/// through the other and was cut up to the wrong pole.
///
/// So the rule is one rounding and not `atan2`'s equality, the same
/// [`POINT_ALONG_A_CROSSING_ROUNDOFF`] of the largest coordinate the fuse
/// allows a point along a crossing, and it takes both poles of that family in.
/// Both halves of the ball are then lunes the walk reads the same way whichever
/// side of each pole the rounding fell. Putting the pole in moves the mesh how
/// far the pole is from the circle, which is charged, plus the rounding of the
/// nearest point itself.
fn how_far_a_circle_that_passes_within_a_rounding_of_a_pole_misses_it(
    [ball, other]: [&Surface; 2],
    pole: Point,
) -> Option<f64> {
    let (Surface::Sphere(sphere), Surface::Plane(flat)) = (ball, other) else {
        return None;
    };
    let centre = flat.project(sphere.centre());
    let down = sphere.centre() - centre;
    let across = (sphere.radius() * sphere.radius() - down.dot(down)).sqrt();
    let towards = flat.project(pole) - centre;
    let length = towards.length();
    // A plane square to the axis cuts a line of latitude, which has no one
    // point nearest the pole, and a circle too small to square-root is no
    // circle.
    if length == 0.0 || !across.is_finite() {
        return None;
    }
    let nearest = centre + towards * (across / length);
    let rounding = one_rounding_among([nearest, pole, sphere.centre()]);
    let missed_by = (nearest - pole).length();
    (missed_by <= rounding).then_some(missed_by + rounding)
}

/// The latitude of the pole of a ball that `at` sits within one rounding of,
/// or `None` when it sits near neither or the surface is not a ball.
///
/// **A corner can be that close to a pole and still read a longitude of its
/// own.** A plane through a ball's axis turned a quarter turn about `y` starts
/// its circle `4.4e-16` off the south pole, where `atan2` reads the latitude
/// one ulp short of `−π/2` and the longitude as a clean `0`. Walked as an
/// ordinary corner, the circle swings half a turn round the pole through it,
/// and which way is the side of the pole the rounding put it on. Walked as a
/// pole, the swing is a pole side and runs the way the face lies. The pole
/// cannot be put in beside it instead: the two are one rounding apart and the
/// flat face across the edge reads them as one corner.
fn the_pole_a_point_sits_within_a_rounding_of(stored: &Surface, at: Point) -> Option<f64> {
    let Surface::Sphere(sphere) = stored else {
        return None;
    };
    let pinch = where_this_surface_pinches(stored).expect("a ball pinches at its poles");
    [1.0, -1.0].into_iter().find_map(|up| {
        let pole = sphere.centre() + Vector::new(0.0, 0.0, up * sphere.radius());
        ((at - pole).length() <= one_rounding_among([at, pole, sphere.centre()]))
            .then_some(up * pinch)
    })
}

/// [`POINT_ALONG_A_CROSSING_ROUNDOFF`] of the largest coordinate of `points`,
/// and of one.
fn one_rounding_among(points: [Point; 3]) -> f64 {
    POINT_ALONG_A_CROSSING_ROUNDOFF
        * points
            .map(|point| point.largest_coordinate_magnitude())
            .into_iter()
            .fold(1.0, f64::max)
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

/// How far this surface's **second** coordinate has to run to come back to
/// where it started, or `None` if it never does.
///
/// Only a torus's does: it is the angle round the ring. A ball's second
/// coordinate is a latitude and a pipe's or a cone's a height, and none of
/// those wraps.
///
/// # Panics
///
/// On a surface this has not been taught, as [`how_far_the_longitude_wraps`]
/// does.
fn how_far_the_second_coordinate_wraps(surface: &Surface) -> Option<f64> {
    match surface {
        Surface::Torus(_) => Some(core::f64::consts::TAU),
        Surface::Plane(_) | Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) => None,
        other => panic!("this crate cannot say whether {other:?} wraps round a second way"),
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
/// **Two of the refusals count corners, and the count is this file's own.** The
/// domain handed over was assembled here — a wire walked from wherever
/// [`steps_from_somewhere_that_has_a_longitude`] chose to start, with a point
/// dropped on every bend of every edge — so "the corner at 7" addresses a list
/// no caller of [`SolidView::mesh`](crate::SolidView::mesh) ever saw. Both of
/// them carry a place in space beside the count, and both places are moved into
/// the **model's own frame** here, which is the same move §116 made for the
/// tangency pair and for the same reason (§113.6, §123).
///
/// One of the two is not about the request at all. `CannotTellIfLoopsTouch` is
/// an exact predicate that ran out of room, and `Undecidable` at the level of a
/// whole operation is a failure rather than a result (§4.2) — so it goes in the
/// log with its enclosure, where the caller can see the point and how wide.
/// Every other refusal is about the deflection or about the loops, and neither is
/// a thing a provenance log has anything to say about.
fn refusal_for(
    face: FaceView<'_>,
    why: CannotTriangulate,
    corner_is: &[u32],
    points: &[Point],
    log: &mut ProvenanceLog,
) -> Error {
    if let CannotTriangulate::LoopsTouch { corner, side, .. } = why {
        return Error::CannotMeshAFace {
            face: face.key(),
            why: CannotTriangulate::LoopsTouch {
                corner,
                side,
                at: where_the_model_holds_this_corner(corner, corner_is, points),
            },
        };
    }
    let CannotTriangulate::CannotTellIfLoopsTouch {
        corner, enclosure, ..
    } = why
    else {
        return Error::CannotMeshAFace {
            face: face.key(),
            why,
        };
    };
    let at = where_the_model_holds_this_corner(corner, corner_is, points);
    log.push(Record::new(
        "orient2d",
        &[PredicatePoint::from(at.coordinates())],
        Level::Exact,
        enclosure,
        Measures::Area,
    ));
    Error::Undecidable {
        records: core::mem::take(log),
    }
}

/// Where the model holds the corner the triangulator numbered `corner`.
///
/// `corner_is` was filled loop by loop in the order the domain's corners went
/// out — outer first, then the holes — so the two lists line up entry for entry,
/// and the answer is a point of the solid's own table rather than one worked out
/// again. A number past the end is this file having handed over a domain it
/// cannot read back, which is a bug here and not bad input.
///
/// **This replaces a walk of the face's wires, which named a different point.**
/// A wire has one point per coedge; a walked loop has those *and* every point
/// the edge walk put between them, and it may start at another corner
/// altogether — so `nth(corner)` into the wires' points agreed with the
/// triangulator's numbering only on a face whose edges are all straight and
/// whose walk happened not to be turned. §123.
fn where_the_model_holds_this_corner(corner: usize, corner_is: &[u32], points: &[Point]) -> Point {
    let at = *corner_is
        .get(corner)
        .expect("the triangulator numbered a corner this walk never handed it");
    points[at as usize]
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Axis, Cylinder, EdgeRun, Orientation, Plane, Scale, Sphere, Vector};
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
            .expect("this model made the key")
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

    /// A refusal about two loops touching names the touch where the **model**
    /// holds it, not where the mesher worked it out.
    ///
    /// `docs/decisions.md` §123, and §113.7 for the wall this is the third
    /// bucket of. The refusal used to say `LoopsTouch { corner: 7, side: 4 }`
    /// and nothing else, and those two numbers count corners of a domain
    /// [`Mesher::hand_over`] assembled — so a caller reading them was reading an
    /// address into a list it never built.
    ///
    /// **A placed solid is what makes the two frames differ by something a test
    /// can see.** The box below sits at `(1024, 2048, 4096)` and its faces are
    /// stored at the origin, so the mesher cuts the bottom cap up in the box's
    /// own numbers and the point it would otherwise publish is `(8, 3, 0)` —
    /// a point of no model at all.
    ///
    /// **The route is `Deserialize` and not a builder**, as for the two tests
    /// above: no builder here puts a hole in a cap that reaches out and touches
    /// its outline, and [`SolidView::mesh`](crate::SolidView::mesh) is public in
    /// its own right.
    ///
    /// Which of the hole's two corners on the outline the sweep meets first is
    /// the order [`every_side`](rcad_mesh) hands the sides out in, which is not
    /// a thing to pin down here — so both are allowed, and each has to be exact.
    #[test]
    fn a_placed_solids_touching_loops_are_named_where_the_model_holds_them() {
        let a_long_way_off = Isometry::translation(Vector::new(1024.0, 2048.0, 4096.0))
            .expect("a shift down the axes");

        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let square =
            crate::ProfileBuilder::new(vec![[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]])
                .build(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("a square eight across");
        let box_of_eight = model
            .add_extrusion(&square, 8.0, &a_long_way_off)
            .expect("a box eight across and eight tall");

        let cap = model
            .data()
            .faces
            .iter()
            .find(|(_, face)| {
                face.surface
                    .as_plane()
                    .is_some_and(|flat| flat.signed_distance_to(Point::origin()) == 0.0)
            })
            .map(|(slot, _)| slot)
            .expect("a box has a cap through the origin");

        // Two corners exactly on the cap's own side from (8, 0) to (8, 8), which
        // is the shape `refuse_loops_that_touch` was written for.
        let corners = [
            model.add_vertex(Point::new(8.0, 3.0, 0.0)),
            model.add_vertex(Point::new(4.0, 4.0, 0.0)),
            model.add_vertex(Point::new(8.0, 5.0, 0.0)),
        ];
        let coedges: SmallVec<[_; 4]> = (0..3)
            .map(|at| {
                let edge = model.add_edge(corners[at], corners[(at + 1) % 3]);
                model.add_coedge(edge, Orientation::Forward)
            })
            .collect();
        let hole = model.add_wire(coedges);
        model.data_mut().faces[cap].wires.push(hole);

        let refusal = model
            .solid(box_of_eight)
            .expect("this model made the key")
            .mesh(
                Deflection::of(1.0).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a hole touching its own outline cannot be cut up");

        let Error::CannotMeshAFace {
            why: CannotTriangulate::LoopsTouch { at, .. },
            ..
        } = refusal
        else {
            panic!("got {refusal:?}")
        };
        assert!(
            at == Point::new(1032.0, 2051.0, 4096.0) || at == Point::new(1032.0, 2053.0, 4096.0),
            "the touch is where the model holds the hole's corner, not at {at:?}"
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
    ///
    /// **It is refused one step earlier now, and more truly.** With the wall a
    /// plane, both cap circles have only flat faces along them and nothing
    /// naming the curve, so no face can place them — which is
    /// [`Error::AnEdgeBendsButEveryFaceAlongItIsFlat`], raised before any wire
    /// is fanned out. This test used to reach `AWireDoesNotFanIntoALoopToMesh`
    /// with the two circles already read as having no points at all (§176.2).
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
            .expect("this model made the key")
            .mesh(
                Deflection::of(1.0).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a wire fanned onto the wrong surface cannot be cut up");
        assert!(
            matches!(refusal, Error::AnEdgeBendsButEveryFaceAlongItIsFlat { .. }),
            "got {refusal:?}"
        );
    }

    /// A refusal about two tangent surfaces names them where the **model**
    /// holds them, not where the mesher worked them out.
    ///
    /// `docs/decisions.md` §116, and §113.6 for the defect. The pair reaches
    /// the marcher in [`the_frame_this_face_is_meshed_in`], which is a frame
    /// this file picked and nothing outside it can name — so a caller matching
    /// the refusal and reading `surfaces` got numbers belonging to no model at
    /// all. A **placed** solid is what makes the two frames differ by something
    /// a test can see: the ball below sits at `(1024, 2048, 4096)` and the
    /// mesher works it out about the origin.
    ///
    /// **The tangency is real and not a rounding.** A pipe of the ball's own
    /// radius, its axis through the centre and square to the meridian, holds
    /// the whole meridian circle: at every point of it the ball's normal is
    /// `out/‖out‖` and the pipe's is `sideways/‖sideways‖`, and `sideways` is
    /// `out` with nothing taken off it, because `out` is square to the axis
    /// there and that dot product is exactly zero. The two normals are the same
    /// bits, so `‖u − v‖²` is exactly zero and the settling has nothing to
    /// divide by — which is [`WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces`],
    /// the site all 44 corpus files of §110.3 stop at.
    ///
    /// **The route is `Deserialize` and not a builder**, as for the two tests
    /// above: `SolidBuilder::build` walks the crossing in its own sweep and
    /// would refuse this description before a model existed, and
    /// [`SolidView::mesh`](crate::SolidView::mesh) is public in its own right.
    #[test]
    fn a_placed_solids_tangency_refusal_names_its_surfaces_where_the_model_holds_them() {
        const RADIUS: f64 = 1.0;
        let a_long_way_off = Isometry::translation(Vector::new(1024.0, 2048.0, 4096.0))
            .expect("a shift down the axes");

        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let ball = model
            .add_ball(RADIUS, &a_long_way_off)
            .expect("a ball of radius one, placed");

        let touching_all_along_the_meridian = Cylinder::new(
            Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("the y axis"),
            RADIUS,
        )
        .expect("a pipe of radius one");
        let seam = model
            .data()
            .edges
            .keys()
            .next()
            .expect("a ball has one edge");
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = model.data().edges[seam].run.clone()
        else {
            panic!("a ball's seam is part of a crossing")
        };
        model.data_mut().edges[seam].run = EdgeRun::PartOfWhereItCrosses {
            and: Surface::Cylinder(touching_all_along_the_meridian),
            picked_by,
        };

        let solid = model.solid(ball).expect("this model made the key");
        let where_the_model_holds_it = solid.faces().next().expect("a ball has one face").surface();
        let refusal = solid
            .mesh(
                Deflection::of(0.1).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a seam running where its two surfaces touch cannot be followed");

        let Error::ACrossingAlongThisEdgeCouldNotBeFollowed { why, .. } = &refusal else {
            panic!("got {refusal:?}")
        };
        let rcad_geom::Error::CannotFollowACrossingWhereItsSurfacesAreTangent { surfaces, at } =
            &**why
        else {
            panic!("got {why:?}")
        };
        assert_eq!(
            *at,
            rcad_geom::WhereFollowingACrossingGaveUp::SettlingAPointOntoBothSurfaces,
            "the normals are the same bits, so it is the settling that has nothing to divide by"
        );

        let ball_it_named = surfaces
            .iter()
            .find(|surface| matches!(surface, Surface::Sphere(_)))
            .expect("one of the pair is the ball");
        assert_eq!(
            *ball_it_named, where_the_model_holds_it,
            "the refusal names the ball where the model holds it, and the model holds it at \
             (1024, 2048, 4096)"
        );
        let Surface::Sphere(named) = ball_it_named else {
            panic!("just matched on it")
        };
        assert_ne!(
            named.centre(),
            Point::origin(),
            "the origin is where the mesher works this face out, and no model has a ball there"
        );

        let Some(Surface::Cylinder(pipe)) = surfaces
            .iter()
            .find(|surface| matches!(surface, Surface::Cylinder(_)))
        else {
            panic!("the other of the pair is the pipe")
        };
        assert_eq!(
            pipe.axis().origin(),
            Point::new(1024.0, 2048.0, 4096.0),
            "and the pipe rode out of the frame with it"
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
            .expect("this model made the key")
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
            .expect("this model made the key")
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
        let mut walk = WalkingAWire::new(Some(whole), None, false, false);
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

    /// Half a ball's skin cut down its polar axis is a lune half a turn wide
    /// whose two corners are both poles, and the walk round it sets off from
    /// one of them. It crosses the other pole in the middle and **arrives back
    /// at the first at the end**, a corner at each end of both pole sides.
    ///
    /// Before §194.5 the walk stopped at the last point on the second meridian,
    /// half a turn from the first corner, and `finish` refused it.
    #[test]
    fn a_walk_that_sets_off_from_a_pole_arrives_back_at_it_down_the_other_meridian() {
        use core::f64::consts::{FRAC_PI_2, PI};
        let whole = 2.0 * PI;
        let mut walk = WalkingAWire::new(Some(whole), None, false, false);
        walk.arrive(0, (0.0, FRAC_PI_2), true);
        walk.arrive(1, (FRAC_PI_2, 0.0), false);
        walk.arrive(2, (0.0, -FRAC_PI_2), true);
        walk.turn_at_a_corner_on_a_pole();
        walk.arrive(3, (-FRAC_PI_2, 0.0), false);
        walk.arrive_back_at_the_pole_the_walk_set_off_from();

        let walked = walk.finish().expect("a lune half a turn wide lies flat");
        let three_quarters = 3.0 * FRAC_PI_2;
        assert_eq!(
            walked.corners,
            vec![
                [FRAC_PI_2, FRAC_PI_2],
                [FRAC_PI_2, 0.0],
                [FRAC_PI_2, -FRAC_PI_2],
                [three_quarters, -FRAC_PI_2],
                [three_quarters, 0.0],
                [three_quarters, FRAC_PI_2],
            ]
        );
        assert_eq!(walked.points, vec![0, 1, 2, 2, 3, 0], "each pole twice");
    }

    /// A walk that ends at a pole leaves it by the longitude it set off from,
    /// because that is where the loop closes.
    #[test]
    fn a_walk_that_ends_at_a_pole_closes_the_side_back_to_where_it_started() {
        let whole = 2.0 * core::f64::consts::PI;
        let mut walk = WalkingAWire::new(Some(whole), None, false, false);
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

    /// A ring of arcs right round a cone's apex closes a whole turn on, and the
    /// walk's own gain is one step short of saying so.
    ///
    /// Six corners a sixth of a turn apart, which is the shape §206.6's face
    /// has: three arcs from the cube's three upper planes, each walked into two
    /// corners. The walk stops at the sixth, `−5.235987755982988` from the
    /// first, and the step back onto the first is the sixth of a turn that is
    /// missing. `climbed` counts it and comes to a whole turn exactly.
    #[test]
    fn a_ring_of_arcs_round_a_pinch_comes_back_a_whole_turn_on() {
        let whole = 2.0 * core::f64::consts::PI;
        let mut walk = WalkingAWire::new(Some(whole), None, false, true);
        for step in 0..6 {
            walk.arrive(step, (-f64::from(step) * whole / 6.0, 2.5), false);
        }

        let walked = walk.finish().expect("a ring lies flat");
        assert_eq!(walked.climbed, -whole, "once right round");
        assert!(walked.goes_right_round);
        assert!(
            !walked.closes_on_one_curve,
            "three arcs are not one closed curve, and this is the case that used to \
             turn on being one"
        );
    }

    /// The same walk over a wire that uses one of its edges twice will not lie
    /// flat, exactly as it did not before.
    ///
    /// An edge used twice is a **seam** — the two sides of one meridian, with
    /// the patch between them — so the walk down the far side has to bring the
    /// longitude back to where the walk up the near side left it. A seam that
    /// ends a whole turn on has not done that, and nothing about a ring makes
    /// it acceptable. A face with a hole is refused here for the same reason
    /// and by the same flag.
    #[test]
    fn a_wire_with_a_seam_that_ends_a_whole_turn_on_still_will_not_lie_flat() {
        let whole = 2.0 * core::f64::consts::PI;
        let mut walk = WalkingAWire::new(Some(whole), None, false, false);
        for step in 0..6 {
            walk.arrive(step, (-f64::from(step) * whole / 6.0, 2.5), false);
        }
        assert!(walk.finish().is_none());
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
            let has_to_be_on =
                which_two_surfaces_a_run_is_between(run.run(), [&along[0], &along[1]]);
            for &point in along_it.points() {
                let at = PredicatePoint::Constructed(point.coordinates().map(|coordinate| {
                    Interval::around(coordinate, slack).expect("a bound is a length")
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
            .expect("this model made the key")
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

        let solid = model.solid(bar).expect("this model made the key");
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
            .expect("this model made the key")
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
                        model
                            .solid(ball)
                            .expect("this model made the key")
                            .faces()
                            .count(),
                        1,
                        "a ball whose one face is its sphere is what makes the two-movement \
                         ceiling below the right one to charge it"
                    );

                    for (kind, solid) in [ball, bar].into_iter().enumerate() {
                        let placed = model.solid(solid).expect("this model made the key");
                        for face in placed.faces() {
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
                                &placed.surfaces_along_each_edge(),
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
            .expect("this model made the key")
            .mesh(
                Deflection::of(0.05).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a ball meshes");

        let uses = every_use_of_an_edge_of(model.solid(ball).expect("this model made the key"));
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
            .expect("this model made the key")
            .mesh(
                Deflection::of(0.5).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a bar meshes");

        let bottom: Vec<EdgeView<'_>> =
            every_use_of_an_edge_of(model.solid(bar).expect("this model made the key"))
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
            .expect("this model made the key")
            .mesh(
                Deflection::of(0.5).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a cube meshes");

        assert_eq!(mesh.points().len(), 8, "a cube is its own eight corners");
        let mut seen = HashSet::new();
        for corner in every_corner_of(model.solid(cube).expect("this model made the key")) {
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

        for edge in every_use_of_an_edge_of(model.solid(cube).expect("this model made the key")) {
            assert!(
                mesh.points_along_edge(edge.key()).is_empty(),
                "a plane has nothing to say about a curve on it, and a straight edge \
                 needs nothing said"
            );
        }
        for corner in every_corner_of(model.solid(another).expect("this model made the key")) {
            assert_eq!(
                mesh.point_at_vertex(corner.key()),
                None,
                "a corner of a solid this mesh was not cut from"
            );
        }
        for edge in every_use_of_an_edge_of(model.solid(another).expect("this model made the key"))
        {
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
                .expect("this model made the key")
                .mesh(
                    Deflection::of(0.5).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("both of these mesh")
        };

        let meshed_bar = cut(bar);
        let seam: Vec<EdgeView<'_>> =
            every_use_of_an_edge_of(model.solid(bar).expect("this model made the key"))
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
        for edge in every_use_of_an_edge_of(model.solid(cube).expect("this model made the key")) {
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
                .expect("this model made the key")
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
        )
        // Each corner of the cell is where two steps meet, as it is in the walk.
        .with_corners_between_two_edges(vec![true; 4]);
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
