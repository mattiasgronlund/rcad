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
    Axis, Deflection, DynamicRange, EdgeRun, Isometry, MOST_POINTS_ALONG_ONE_CROSSING, NamedRun,
    Orientation, POINT_ALONG_A_CROSSING_ROUNDOFF, Point, SURFACE_AFTER_SHIFTING_ROUNDOFF, Surface,
    Vector, which_two_surfaces_a_run_is_between,
};
use rcad_mesh::{CannotTriangulate, Loop, NotALoop, TrimmedDomain, triangulate};
use rcad_predicates::{
    Approximation, Cause, Decision, Level, Measures, Point as PredicatePoint, Policy,
    ProvenanceLog, Record,
};
use smallvec::SmallVec;

use crate::error::walking_along_this_edge_refused;
use crate::fuse::name_of_surface_bits;
use crate::holds::{Trimming, the_face_across, the_side_this_face_lies_on};
use crate::validate::{
    nothing_curved_says_where_it_bends, side_of_surface, the_way_it_is_parametrised_about,
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
    /// `BuildingAMesh::walk` for why the model needs that rule at all. Where that
    /// rule reads nothing at the corner, as on a ball's pole, it runs the way
    /// its forward use walks it.
    ///
    /// # Empty, and the three ways to be empty
    ///
    /// An edge that needed no help has an empty list: a straight edge is drawn
    /// by its two ends alone, so every edge of a polyhedron is empty even though
    /// it was walked. **One straight edge is not**: a curved face's straight
    /// edge lying across a flat face of the same solid, between two corners of
    /// that face, has the one point in its middle that stops the flat face
    /// drawing its side a second time (see `BuildingAMesh::walk`).
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
    /// Each **curved** face is cut up **in the frame it is stored in**, and the
    /// points are moved into place afterwards. A `(u, v)` is a working space and
    /// never an answer (§23.4), so where a solid sits must not change how a
    /// curved face is cut up — which is §19 arriving for meshes.
    ///
    /// It is also the only thing that works. A ball is the same ball however it
    /// is turned, so [`Sphere::transformed`] under a turn about the centre moves
    /// nothing at all, and a turned ball's seam is then **not** a line of
    /// constant longitude of the turned sphere's coordinates: its face is not a
    /// rectangle in them and does not even cover them. In the frame the ball is
    /// stored in it is a rectangle, and turning the ball turns the mesh and
    /// nothing else.
    ///
    /// **A flat face is cut up where the solid sits.** A curved face's frame
    /// decides where its points go; a flat face has no points of its own, only a
    /// choice of which of the points round it its triangles join, and points of
    /// one circle make that choice a tie. Made in the stored frame, the tie went
    /// one way for a placed cap and the other for the same cap read back out of
    /// its STEP file, which has no placement (§362). Where the solid sits is the
    /// frame every copy of a flat face shares, so moving a flat face can change
    /// which of two tied triangles it gets, and a placed flat face is cut the
    /// same way as its read-back — see [`the_surface_this_face_is_cut_up_on`].
    /// A disc whose rim the face across reads at one height, or a ring whose
    /// two rims are each read that way, is cut by the order of its corners
    /// instead, which no movement changes
    /// ([`BuildingAMesh::nothing_across_its_rim_joins_two_of_its_points`]).
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
    /// [`Error::PlacingItTookAwayTheSpaceItsMeshEncloses`] when the solid is
    /// placed and its mesh, moved out to where it sits, no longer encloses the
    /// space it did where it was made.
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
    /// bytes may not panic (`docs/decisions.md` §4.1). Whatever [`rcad_geom::walk`]
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
        let deflection = match std::env::var("RCAD_PROBE_CLAMP") {
            Ok(most) => Deflection::of(deflection.length().min(most.parse().unwrap())).unwrap(),
            Err(_) => deflection,
        };
        if std::env::var("RCAD_PROBE_Q1494").is_ok() {
            eprintln!("PROBE mesh at {:e}", deflection.length());
        }
        let mut building = BuildingAMesh::new(deflection, along_each_edge);
        for face in self.faces() {
            building.walk_the_edges_this_face_can_place(self, face, policy, log)?;
        }
        for face in self.faces() {
            building.cut_up(face, self, log)?;
        }
        let made = core::mem::take(&mut building.made);
        let mesh = building.finish()?;
        if self
            .faces()
            .all(|face| face.location() == Isometry::identity())
        {
            return Ok(mesh);
        }
        Self::unless_placing_it_took_away_its_space(mesh, made)
    }

    /// `mesh`, unless it encloses space where its points were made and none
    /// where they land.
    ///
    /// **The one promise of a mesh of a solid that moving it can break on its
    /// own.** Moved out to where a ball one ulp across sits, the octahedron it
    /// was meshed as keeps every point within the deviation it claims, every
    /// side used twice and every corner apart — and lands flat, each triangle
    /// on a twin (`docs/decisions.md` §333.6). Three questions asked one
    /// triangle or one side at a time were turned down because a sound placed
    /// solid can fail each of them, or the flat ball can pass it (§379.2); the
    /// space the whole mesh encloses is what tells the two apart.
    ///
    /// **Both signs, and not only the one where it lands.** A mesh that
    /// encloses nothing where it was made is about the solid, and
    /// [`Model::validate`](crate::Model::validate) already says so, or meshes
    /// it finer first. Only the sign the movement lost is this refusal's.
    ///
    /// `made` runs with the mesh's points, each where its face is stored —
    /// see [`BuildingAMesh::made`]. Both sums are
    /// [`signed_volume_over`](crate::measure::signed_volume_over), which
    /// settles its sign exactly where the rounded sum cannot.
    fn unless_placing_it_took_away_its_space(mut mesh: Mesh, made: Vec<Point>) -> Result<Mesh> {
        let enclosed_where_placed = crate::measure::signed_volume_over(&mesh);
        if enclosed_where_placed > 0.0 {
            return Ok(mesh);
        }
        let placed = core::mem::replace(&mut mesh.points, made);
        let enclosed_where_made = crate::measure::signed_volume_over(&mesh);
        mesh.points = placed;
        if enclosed_where_made > 0.0 {
            return Err(Error::PlacingItTookAwayTheSpaceItsMeshEncloses {
                enclosed_where_made,
                enclosed_where_placed,
            });
        }
        Ok(mesh)
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

/// How far a longitude read off a point can be from the one the point stands
/// at, in radians, and so how far past a quarter turn a piece may reach before
/// [`BuildingAMesh::cut_in_two`] cuts it.
///
/// A point carries roundings of about `f64::EPSILON` times its largest
/// coordinate — from its placement, from a file's decimals, from the frame it
/// is read in — and read as an angle round a circle of radius `ρ` that is
/// `f64::EPSILON · |P| / ρ`. `ρ` is a tube's, a pipe's or a ball's radius, a
/// feature of the model, so [`DynamicRange::LARGEST_SUPPORTED_RATIO`] bounds
/// `|P| / ρ` and this is four such roundings at that ratio, `8.9e-5`.
///
/// **What it is set against is a quarter turn, `1.57`, and not anything the
/// mesh is measured by.** Any piece narrower than half a turn is one whose
/// triangles the triangulator can see straying, so a piece this much over a
/// quarter meshes as one at a quarter does; see `cut_in_two` for what cutting
/// it costs instead.
const HOW_FAR_A_READ_LONGITUDE_CAN_BE_OFF: f64 =
    4.0 * DynamicRange::LARGEST_SUPPORTED_RATIO * f64::EPSILON;

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
    /// Where each point of `points` was made, in the frame its face is stored
    /// in: worked out there for a placed face, and the point itself for a face
    /// at no placement, whose frame is undone by a shift that loses nothing.
    ///
    /// Two readers. [`BuildingAMesh::walk_a_wire_round`] reads a placed curved
    /// face's points here rather than moving them out and back, and
    /// [`SolidView::unless_placing_it_took_away_its_space`] asks how much space
    /// the mesh encloses on them. The faces of one shell share one placement, so
    /// each shell is closed in one frame here.
    made: Vec<Point>,
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
            made: Vec::new(),
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
        solid: SolidView<'_>,
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
                let walked = self.walk(solid, edge, face, room, policy, log)?;
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
    /// **A crossing that runs through a ball's pole has the pole among its
    /// points**, whether or not the walk would have landed there — see
    /// [`put_in_the_poles_a_crossing_passes`]. An open one has it only where
    /// the pole is between its two ends (Q1270).
    ///
    /// # The corners it starts from are the ones the arena stores
    ///
    /// For a placed face, the frame this walks in is the placement's own
    /// inverse, and the edge's two ends used to arrive as the located corners
    /// moved back into it: the placement and then its inverse, rounded twice.
    /// `(3, 0, 0)` turned an eighth about `z` and back is `(3, −4.4e−16, 0)`, and
    /// a walk that starts from a different corner puts its points somewhere else
    /// — 75 of the 252 points of a bar turned that way were not the unplaced
    /// bar's points moved out (`docs/decisions.md` §249.3). The surface was
    /// already read out of the arena for this reason
    /// ([`the_surface_this_face_is_meshed_on`]), and now the corners are too
    /// ([`where_this_corner_is_in_the_frame_it_is_meshed_in`]). Every point of a
    /// placed bar is then the unplaced bar's point moved out, bit for bit, under
    /// twelve placements with and without a shift;
    /// `a_placed_bar_is_meshed_from_the_corners_it_stores.rs` is the
    /// demonstration.
    ///
    /// **It is right because the corner is looked at through the face's own
    /// placement.** A view hands its location down unchanged to every wire,
    /// coedge, edge and vertex under it, and `edge` came out of `face`'s own
    /// wires, so the stored corner is exactly the point the located one stands
    /// for. A corner looked at through any other location would need the move
    /// back, and is a panic in the helper rather than a silent wrong corner.
    ///
    /// **It frees nothing in [`how_far_moving_this_face_rounds`].** That charge
    /// is one [`Isometry::POINT_ROUNDOFF`] for carrying a mesh point *out*, which
    /// every point below still does, plus what shifting the surfaces round a face
    /// costs, which is zero for a placed face by its first line. The move of the
    /// corners back in was never charged, so there is nothing to take off.
    ///
    /// **What it does not reach.** [`BuildingAMesh::walk_a_wire_round`] now
    /// reads a placed curved face's points where they were made too (§379), so
    /// a placed bar's wall is cut into the unplaced wall's triangles, corner for
    /// corner, under the same twelve placements. Its caps are cut where the
    /// solid sits, like their read-back
    /// ([`the_surface_this_face_is_cut_up_on`]), and by the order of their
    /// corners, so they match too
    /// ([`BuildingAMesh::nothing_across_its_rim_joins_two_of_its_points`],
    /// §379.5). A solid read back from a file is meshed in the world, not in
    /// its stored frame, so this does not make a STEP round trip measure the
    /// same (§249.5).
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
    /// # The pair it walks between is the one everyone reads
    ///
    /// The two faces either side are handed through
    /// [`which_two_surfaces_a_run_is_between`], the pair [`rcad_geom::walk`],
    /// `Model::validate` and the fuse read off the same run, so that the
    /// direction rule for a closed edge below reads the curve that was walked.
    /// For an edge with only flat faces along it that is one face's plane and
    /// the curved surface the run names — the circle's own cylinder, for a
    /// circle a file imprinted on a flat face — since two planes cross in a
    /// straight line or not at all.
    ///
    /// **This used to be a narrower copy that stopped there** (§222.1 item 1),
    /// and it cost a rim between two faces on one ball. The fuse writes that rim
    /// naming the plane that cut it, and the copy left the pair as the ball
    /// twice, which has no crossing direction, so the rule below fell back to
    /// the ball's longitude at the corner. Where the corner is on the equator
    /// and the plane stands upright, that longitude is square to the rim, and
    /// the fallback then read nothing but how the circle curves: the rim kept
    /// whichever way the walker drew it, and five of eight such unions put each
    /// face's triangles on the other face's side of the plane
    /// (`a_ball_cut_through_its_equator_meshes_each_half_on_its_own_side.rs`).
    /// The ball and the plane read the latitude there, as the fuse does.
    ///
    /// **A rim where a ball rests on a pipe of its radius reads the same pair**,
    /// the ball and the rim's plane, and so is walked the way the ball's latitude
    /// runs where its longitude is square to the rim. The copy read the ball and
    /// the pipe, which only touch, so no rule answered and the rim kept the
    /// walker's way round. A solid built to that way is walked against the rule
    /// now, and `Model::validate` reads the rim against the same plane and
    /// refuses it by `AClosedEdgeIsWalkedAgainstItsSurface` before it is meshed
    /// (`a_half_ball_on_a_pipe_of_its_own_radius.rs`, `docs/decisions.md`
    /// §229.4). Meshed without that sweep, its pipe's wire will not lie flat.
    fn walk(
        &mut self,
        solid: SolidView<'_>,
        edge: EdgeView<'_>,
        face: FaceView<'_>,
        room: f64,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Vec<u32>> {
        let back = the_frame_this_face_is_meshed_in(face);
        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        let (from, to) = ends_in_the_stored_order(edge);
        let ends =
            [from, to].map(|end| where_this_corner_is_in_the_frame_it_is_meshed_in(end, face));
        let run = NamedRun::new(
            edge.stored_run()
                .shifted(the_shift_this_face_is_meshed_with(face))?,
            ends.map(|end| PredicatePoint::from(end.coordinates())),
        )?;
        let along = the_two_surfaces_either_side(edge, &self.along_each_edge, &back)?;
        if nothing_curved_says_where_it_bends(run.run(), along) {
            return Err(Error::AnEdgeBendsButEveryFaceAlongItIsFlat { edge: edge.key() });
        }
        let along = which_two_surfaces_a_run_is_between(run.run(), [&along[0], &along[1]]);

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
        // **A straight edge lying across a flat face of the same solid gets a
        // point in its middle**, or that face draws its side a second time. The
        // wedge less the frustum it rests on keeps its slanted face whole and
        // the cone cut open along the ruling the two touch on, which is how
        // `Assembly::part_each_line_two_faces_rest_against_each_other_along`
        // leaves such a line. The ruling's two ends are corners of the slanted
        // face, each in the middle of a straight side of it, and both of that
        // face's triangulations with no flat triangle draw the side between
        // them, which was the ruling's own side: four uses where a mesh has two
        // (`degenerate_curved_fuses.rs`, queue row Q953). A point between the
        // ends makes the cone's sides along the ruling two shorter ones that no
        // other face names. Halving is exact and the sum rounds once, by half
        // an ulp of each coordinate, which `halved` charges.
        let mut halved = 0.0;
        if points.is_empty()
            && *edge.stored_run() == EdgeRun::Straight
            && a_flat_face_off_it_holds_both_its_ends(solid, edge)
        {
            let [from, to] = ends;
            let [x, y, z] = core::array::from_fn(|axis| {
                from.coordinates()[axis] * 0.5 + to.coordinates()[axis] * 0.5
            });
            points.push(Point::new(x, y, z));
            halved = f64::EPSILON
                * from
                    .largest_coordinate_magnitude()
                    .max(to.largest_coordinate_magnitude());
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
        //
        // Where nothing at the corner names a way, as at a ball's pole, the way
        // the faces walk the edge does, read only then because it reads every
        // corner of a face: [`the_way_the_faces_along_a_closed_edge_walk_it`].
        if edge.is_closed()? {
            let corner = ends[0];
            let read_at_the_corner = the_way_a_closed_edge_is_stored_running(&along, corner)
                .or_else(|| the_way_a_closed_edge_where_its_faces_touch_runs(&along, corner));
            let read_off_the_faces = match read_at_the_corner {
                Some(way) => Some(way),
                None => {
                    the_way_the_faces_along_a_closed_edge_walk_it(solid, face, edge, &back, corner)?
                }
            };
            // **A flat face whose one wire is the whole of this edge holds the
            // disk inside the ball across it**, so it has a side with no corner
            // off the ball to read it from (`docs/decisions.md` §220.6). A cap
            // of a ball through its pole, lidded, has no corner off the rim at
            // all, and before this its points kept the walker's order, which
            // turns round with the way the rim's plane faces: one of the two
            // ways of writing the lid's use was refused
            // `FacesDoNotAgreeWhichWayTheyFace`, and which one depended on the
            // plane (`a_closed_edge_neither_direction_rule_answers.rs`).
            let stored_way = match read_off_the_faces {
                Some(way) => Some(way),
                None => the_way_a_flat_disk_across_a_ball_walks_its_rim(
                    solid, face, edge, &back, corner,
                )?,
            };
            if let Some(first) = points.first()
                && let Some(stored_way) = stored_way
                && (*first - corner).dot(stored_way) < 0.0
            {
                points.reverse();
            }
        }
        // **Both kinds of closed run get their poles**, and the pair the second
        // kind hands over is the run's own and not the two faces. A rim written
        // [`EdgeRun::RightRoundWhereItCrosses`] has the one surface either side
        // of it — that is why it carries `and` at all — so the faces say
        // `[ball, ball]`, which [`put_in_the_poles_a_crossing_passes`]
        // reads as a ball crossed with itself and walks straight past. The pair
        // below is the one `rcad_geom::walk` read off the same run to draw the
        // circle with (`which_two_surfaces_a_run_is_between`), so the poles are
        // put in on the curve that was actually walked. An open run gets the
        // poles between its ends, read off the same pair.
        let moved = match run.run() {
            EdgeRun::RightRoundWhereItsFacesCross | EdgeRun::RightRoundWhereItCrosses { .. } => {
                put_in_the_poles_a_crossing_passes(
                    &mut points,
                    ends,
                    &which_two_surfaces_a_run_is_between(run.run(), [&along[0], &along[1]]),
                    HowMuchOfTheCrossing::AllOfIt,
                )
            }
            EdgeRun::PartOfWhereItCrosses { .. } => put_in_the_poles_a_crossing_passes(
                &mut points,
                ends,
                &along,
                HowMuchOfTheCrossing::BetweenItsEnds,
            ),
            EdgeRun::Straight | _ => 0.0,
        };
        if self.points.len() + points.len() > MOST_POINTS_A_MESH_WILL_HOLD {
            return Err(Error::MoreMeshPointsThanASolidWillHold {
                most: MOST_POINTS_A_MESH_WILL_HOLD,
            });
        }

        let moving = how_far_moving_this_face_rounds(face, &self.along_each_edge);
        self.worst = self
            .worst
            .max(along_it.deviation().magnitude() + moved + halved + moving);
        Ok(points
            .into_iter()
            .map(|along| self.remember_made_on(face, &there, along))
            .collect())
    }

    /// Cut one face of `solid` into triangles and put them in the solid's own
    /// table.
    fn cut_up(
        &mut self,
        face: FaceView<'_>,
        solid: SolidView<'_>,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let surface = the_surface_this_face_is_cut_up_on(face);
        let by_corner_order = surface.as_plane().is_some()
            && self.nothing_across_its_rim_joins_two_of_its_points(face, solid);
        let room =
            self.deflection.length() - how_far_moving_this_face_rounds(face, &self.along_each_edge);

        let mut walked = Vec::with_capacity(face.wire_count());
        let probe = std::env::var("RCAD_PROBE_Q1494").is_ok();
        if probe {
            eprintln!(
                "PROBE face {:?} {:?} surface {:?}",
                face.key(),
                face.orientation(),
                surface
            );
        }
        for wire in face.wires() {
            let one = self.walk_a_wire_round(wire, face, &surface);
            if probe {
                let pts: Vec<_> = wire.points().map(|p| p.coordinates()).collect();
                eprintln!(
                    "PROBE   wire of {} coedges corners {:?}",
                    wire.coedges().count(),
                    pts
                );
                match &one {
                    Some(w) => eprintln!(
                        "PROBE   walked {} corners round {} climbed {} : {:?}",
                        w.corners.len(),
                        w.goes_right_round,
                        w.climbed,
                        w.corners
                    ),
                    None => eprintln!("PROBE   walk refused"),
                }
            }
            walked.push(one.ok_or(Error::AWireWillNotLieFlatOnItsFace { face: face.key() })?);
        }
        if let Some(whole) = how_far_the_second_coordinate_wraps(&surface)
            && let Some((first, later)) = walked.split_first_mut()
        {
            for chain in later {
                *chain = with_its_second_coordinate_in_the_turn_of(chain, first, whole);
            }
        }
        // **A wire that sets off from one corner of the model twice is two loops
        // touching there**, and handed over whole it names that corner twice,
        // which the triangulator refuses as `CornersCoincide` and must: a loop
        // through one point twice says nothing about which of the pieces it
        // draws is the face. A ball touching the edge between two walls leaves
        // one: the wall's circle and the ball's other circle both start and end
        // where the edge touches it, so the ball's face between them is bounded
        // by a figure eight, and the wall's wire runs down the edge, round the
        // circle and on down the edge. Cut apart there, the wall is an outline
        // with a hole touching it, and the ball is two holes touching each
        // other, which the triangulator and the strips below take
        // (`docs/decisions.md` §364).
        let walked: Vec<WalkedLoop> = face
            .wires()
            .zip(walked)
            .flat_map(|(wire, around)| {
                apart_where_it_passes_a_corner_twice(
                    around,
                    &self.corners_a_wire_sets_off_from_twice(wire),
                )
            })
            .collect();
        // **A wire that winds is only allowed beside wires that wind too**, or
        // as the rim of a cap on a ball beside islands, or as one of the two
        // rims of a ring on a pipe beside islands. The loops of a face cut the
        // chart into stretches, and the one the face is has every one of its
        // wires in its boundary. Two that go right round bound the band between
        // them and [`BuildingAMesh::cut_a_ring_across`] cuts it. One that goes
        // right round beside others that do not is a cap with islands in it
        // when the surface is a ball and every other wire is a hole: the cap's
        // side of its rim is the side holding the pole the walk keeps on its
        // left, and each hole lies on the face, so on that side
        // ([`BuildingAMesh::cut_a_cap_up_to_its_pole`]). Two that go right
        // round a pipe beside holes are a ring with islands in it, the band
        // between the two with each hole on it, and the ring is cut across as
        // a ring is, round the islands its cuts meet. Anything else of that
        // kind is refused the way a wire that will not lie flat has always been
        // refused. On a torus that is asked of each way round, the tube and the
        // ring, and a wire that went right round both was refused in
        // [`WalkingAWire::finish`].
        let winding_round =
            |way: fn(&WalkedLoop) -> bool| walked.iter().filter(|&around| way(around)).count();
        let round_the_first_way = winding_round(|around| around.goes_right_round);
        let round_the_second_way = winding_round(|around| around.goes_right_round_the_second_way);
        if !a_cap_on_a_ball_with_islands_in_it(&surface, &walked)
            && !a_ring_on_a_pipe_with_islands_in_it(&surface, &walked)
            && [round_the_first_way, round_the_second_way]
                .iter()
                .any(|&winding| winding > 0 && winding < walked.len())
        {
            return Err(Error::AWireWillNotLieFlatOnItsFace { face: face.key() });
        }
        let patches = if round_the_second_way > 0 {
            self.cut_a_band_right_round_the_ring(face, &surface, &walked, room)?
        } else {
            let patches = self.cut_off_what_goes_too_far_round(face, &surface, walked, room)?;
            self.cut_what_reaches_right_round_the_ring(face, &surface, patches, room)?
        };
        for patch in patches {
            self.hand_over(face, &surface, patch, room, by_corner_order, log)?;
        }
        Ok(())
    }

    /// The entries in the point table of every corner of the model that starts
    /// two coedges of `wire`, each once.
    fn corners_a_wire_sets_off_from_twice(&self, wire: WireView<'_>) -> Vec<u32> {
        let mut starts: HashMap<VertexKey, u32> = HashMap::new();
        let mut twice = Vec::new();
        for coedge in wire.coedges() {
            let start = coedge.start().key();
            let times = starts.entry(start).or_default();
            *times += 1;
            if *times == 2 {
                twice.push(
                    *self.at_vertex.get(&start).expect(
                        "walking the wire put the corner every coedge starts at in the table",
                    ),
                );
            }
        }
        twice
    }

    /// Whether `face` is bounded by one closed edge, or by two — an outline
    /// and one hole, a ring between two circles — and every face across each
    /// reads every point of it at one height of its own chart, so that none of
    /// them can draw a side between two of those points. A flat `face` that
    /// passes is cut by the order of its corners
    /// ([`TrimmedDomain::cut_by_corner_order`]), the same way however the
    /// solid is placed.
    ///
    /// **A ring also needs no face across both rims.** Its cut joins a corner
    /// of the outline to a corner of the hole and never two of one rim, and
    /// only a face that holds points of both rims could draw one of those. So
    /// a ring needs less of each rim than a disc does; it is asked on the same
    /// read of each rim all the same, the one that keeps a ball's lids with
    /// the Delaunay cut, since that is the read measured against them
    /// (`docs/decisions.md` §379.5, Q1109).
    ///
    /// **Why one height is enough, and exact.** The face across hands its
    /// triangulator these points on one straight line, at the second
    /// coordinate's very bits, and a side between two of them that are not next
    /// to each other along the edge would run through the points between: no
    /// triangulation draws one. A bar's wall reads its cap circle at the cap's
    /// height. The read is the one [`BuildingAMesh::walk_a_wire_round`] makes,
    /// and only where the second coordinate does not wrap, since there the walk
    /// hands it over as it came ([`WalkingAWire::arrive`]).
    ///
    /// **Why it is needed.** The cut joins corners that are not next to each
    /// other, and a ball's patch along a lid's rim joins some of the same two
    /// points: the lid of `a_rim_cornered_on_the_pole_builds_one_way_round_and_that_way_measures`,
    /// now `a_rim_cornered_on_the_pole_builds_either_way_round_and_both_ways_measure`,
    /// was refused as two patches drawing one side, and 68 of the 1440 meshes
    /// of `no_sliver_cut_through_a_balls_pole_folds_at_any_deflection` folded
    /// where 38 did then (`docs/decisions.md` §379.5), and 2 have since Q1027
    /// kept a ball's rim point beside its pole from being cut off from either
    /// end of the pole side. A ball reads a tilted rim at
    /// many latitudes, so its lids stay with the Delaunay cut.
    ///
    /// The points strictly along the edge are on no other face, so only the
    /// faces across it could join two of them.
    fn nothing_across_its_rim_joins_two_of_its_points(
        &self,
        face: FaceView<'_>,
        solid: SolidView<'_>,
    ) -> bool {
        if !(1..=2).contains(&face.wire_count()) {
            return false;
        }
        let mut across_each_rim = Vec::with_capacity(2);
        for wire in face.wires() {
            let Some(across) = self.the_faces_across_a_rim_read_at_one_height(wire, face, solid)
            else {
                return false;
            };
            across_each_rim.push(across);
        }
        match &across_each_rim[..] {
            [_] => true,
            [outline, hole] => outline.iter().all(|key| !hole.contains(key)),
            _ => unreachable!("a face of one or two wires has one or two rims"),
        }
    }

    /// The faces across `wire`, when it is one closed edge and every face
    /// across reads every point of it at one height of a second coordinate
    /// that does not wrap; `None` otherwise.
    fn the_faces_across_a_rim_read_at_one_height(
        &self,
        wire: WireView<'_>,
        face: FaceView<'_>,
        solid: SolidView<'_>,
    ) -> Option<Vec<FaceKey>> {
        if !wire.closes_on_one_edge() {
            return None;
        }
        let rim = wire.coedges().next()?.edge().key();
        let walked = self.along_edge.get(&rim)?;
        let mut across = Vec::new();
        for other in solid.faces().filter(|other| other.key() != face.key()) {
            for coedge in other
                .wires()
                .flat_map(WireView::coedges)
                .filter(|coedge| coedge.edge().key() == rim)
            {
                let chart = the_surface_this_face_is_cut_up_on(other);
                if chart.as_plane().is_some()
                    || how_far_the_second_coordinate_wraps(&chart).is_some()
                {
                    return None;
                }
                let reads = HowAFaceReadsThePointsRoundIt::of(other);
                let height = |at: Point| chart.coordinates_of(at).1.to_bits();
                let corner = height(reads.a_corner(coedge.start(), other));
                if walked
                    .iter()
                    .any(|&at| height(reads.a_point(at, self)) != corner)
                {
                    return None;
                }
                across.push(other.key());
            }
        }
        (!across.is_empty()).then_some(across)
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
        //
        // **The rim need not be one closed curve.** A turned cube's corner
        // poking into a ball cuts it in three arcs that meet at three corners,
        // and the ball's skin inside the cube is the ball less that triangle.
        // Asked only of one closed curve, it fell through to be filled inside
        // out, and the intersection was refused
        // `Invalid(FacesDoNotAgreeWhichWayTheyFace)` (`docs/decisions.md`
        // §402). Any wire that passes each of its points once is an island in
        // the rectangle the same way; one that passes a point twice, a pole
        // side or a slit, is not, and is left to the paths below as before.
        let widest = whole_turn / 4.0;
        if walked.len() == 1
            && (walked[0].closes_on_one_curve || it_passes_each_of_its_points_once(&walked[0]))
            && !walked[0].goes_right_round
            && stored.where_it_pinches().is_some()
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
            return self.cut_each_narrow_enough(
                face,
                stored,
                Chart::Upright,
                vec![one, other],
                widest,
                room,
            );
        }
        // **Several holes, each with the material outside it, are the same
        // shape cut into strips.** Handed over whole, the first loop was taken
        // for the outer one and the triangulator filled the discs, not the
        // ball less the discs. So the ball is opened along a meridian in every
        // stretch of longitude no hole reaches, each strip from pole to pole
        // holds the holes between its two meridians, and each strip is cut up
        // to both poles the way a ball less one disc is.
        if walked.len() > 1
            && stored.where_it_pinches() == Some(core::f64::consts::FRAC_PI_2)
            && walked.iter().all(|around| {
                !around.goes_right_round
                    && it_passes_each_of_its_points_once(around)
                    && how_much_of_the_chart_this_loop_encloses(around) <= 0.0
            })
        {
            let Some(strips) = self.open_the_ball_into_strips(face, stored, &walked, room)? else {
                return Err(Error::CurvedFaceWithAHoleGoesTooFarRound { face: face.key() });
            };
            let touching = the_points_two_loops_pass(&walked);
            let mut pieces = Vec::with_capacity(2 * strips.len());
            for AStripAndItsHoles { strip, holes } in strips {
                let Some(cut) =
                    self.cut_holes_up_to_both_poles(face, stored, &strip, &holes, room)?
                else {
                    return Err(Error::CurvedFaceWithAHoleGoesTooFarRound { face: face.key() });
                };
                pieces.extend(
                    cut.into_iter()
                        .flat_map(|piece| apart_where_two_holes_touch(piece, &touching)),
                );
            }
            return self.cut_each_narrow_enough(face, stored, Chart::Upright, pieces, widest, room);
        }
        // **A cap with islands in it is cut up to its pole as a cap is**, each
        // island then put whole into the one patch it lies in, as a hole of
        // that patch. A ball cut by a block whose one face crosses it apart
        // from the rest leaves one: the rest cut the rim right round the pole,
        // and the lone face's circle is an island (`docs/decisions.md` §212.6).
        if a_cap_on_a_ball_with_islands_in_it(stored, &walked) {
            let (rims, islands): (Vec<WalkedLoop>, Vec<WalkedLoop>) = walked
                .into_iter()
                .partition(|around| around.goes_right_round);
            let [rim] = &rims[..] else {
                unreachable!("a cap with islands has one rim, not {}", rims.len());
            };
            let Some(patches) =
                self.cut_a_cap_up_to_its_pole(face, stored, rim, &islands, whole_turn, room)?
            else {
                return Err(Error::CurvedFaceWithAHoleGoesTooFarRound { face: face.key() });
            };
            return Ok(patches);
        }
        // **A ring with islands in it is cut across as a ring is**, a cut that
        // meets an island going round it, and each island no cut meets put
        // whole into the one patch it lies in. A thin bar through a fat one
        // off its axis leaves the fat bar's wall this shape, with a window
        // where the thin bar goes in and one where it comes out (Q1494). Asked
        // before anything reads `walked[0]` as the outline, since an island
        // may be stored first.
        if a_ring_on_a_pipe_with_islands_in_it(stored, &walked) {
            let (rims, islands): (Vec<WalkedLoop>, Vec<WalkedLoop>) = walked
                .into_iter()
                .partition(|around| around.goes_right_round);
            let Some(patches) = self.cut_a_ring_across(
                face,
                stored,
                Chart::Upright,
                &rims,
                &islands,
                whole_turn,
                room,
            )?
            else {
                return Err(Error::CurvedFaceWithAHoleGoesTooFarRound { face: face.key() });
            };
            return Ok(patches);
        }
        if walked.len() == 1
            && let Some((one, other)) = self.open_a_slit(face, stored, &walked[0], room)?
        {
            return self.cut_each_narrow_enough(
                face,
                stored,
                Chart::Upright,
                vec![one, other],
                widest,
                room,
            );
        }
        if how_far_round(&walked[0]) <= widest {
            return Ok(vec![walked]);
        }
        // **One curve right round and nothing else is a cap**, and a cap has a
        // pole to cut up to even though the model never said so. A *ring* — two
        // such curves — is cut across to the other one instead, below.
        if walked.len() == 1 && walked[0].goes_right_round {
            let Some(patches) =
                self.cut_a_cap_up_to_its_pole(face, stored, &walked[0], &[], whole_turn, room)?
            else {
                return Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { face: face.key() });
            };
            return Ok(patches);
        }
        // A **ring** is the face with a hole that the fuse makes most: a tube
        // of a bar's wall between two planes, bounded by two curves that each
        // go right round and by nothing else. Cut across both of them at once
        // and what comes out is four patches with one loop each, which is the
        // shape everything below expects.
        //
        // §31.10 item 5 and §35.7 both say this claim is the one lane 5
        // falsifies, and the sentence that stops being true is *"the fuse
        // refuses a curved solid outright"*. The other face with a hole this
        // cuts is a ball's skin whose outer loop reaches both poles and whose
        // hole reaches neither, which a half ball cut again leaves: it is cut
        // from the hole up to each pole instead. A ball's skin with only holes
        // was opened into strips above, a cap on a ball with islands in it
        // was cut up to its pole, and a ring on a pipe with islands in it was
        // cut across round them. Any other face with one loop right round and
        // another that is not has already been refused in
        // [`BuildingAMesh::cut_up`]. `Model` is `Deserialize` and `validate`
        // is the untrusted-import boundary (§16.11), so that is a refusal
        // rather than an assert.
        //
        // **The two curves need not be one closed curve each.** A frustum's
        // band between its own near cap circle and a ring of arcs a cube's
        // upper planes cut is a ring too, and the arcs are the ring §208.4
        // taught the walk to come back a whole turn on.
        if walked.len() != 1 {
            if let Some(patches) = self.cut_a_ring_across(
                face,
                stored,
                Chart::Upright,
                &walked,
                &[],
                whole_turn,
                room,
            )? {
                return Ok(patches);
            }
            let Some((one, other)) =
                self.cut_a_hole_up_to_both_poles(face, stored, &walked, room)?
            else {
                return Err(Error::CurvedFaceWithAHoleGoesTooFarRound { face: face.key() });
            };
            return self.cut_each_narrow_enough(
                face,
                stored,
                Chart::Upright,
                vec![one, other],
                widest,
                room,
            );
        }
        let only = walked.into_iter().next().expect("just checked");
        self.cut_each_narrow_enough(face, stored, Chart::Upright, vec![only], widest, room)
    }

    /// Cut a torus face whose two wires each go right round the **ring** —
    /// the band between two circles round the tube's own middle, with no seam
    /// across the tube — into patches no wider than a quarter turn either way.
    ///
    /// It is [`BuildingAMesh::cut_a_ring_across`] on the loops turned on their
    /// side ([`Chart::OnItsSide`]), so that the angle round the ring is
    /// the one it cuts across at four quarters, and then each quarter is cut
    /// round the tube as any piece is. The top half of a ring written with one
    /// wire per equator is this shape, and so is the band between the inner
    /// and outer equator that §250.7's half ring needs.
    ///
    /// **The second loop is put on the side of the first the face lies on
    /// before anything is cut**, in the angle round the tube, because a straight
    /// cut from one loop to the other goes over the top of the tube or under
    /// it according to which turn of that angle the second loop is in
    /// ([`with_its_tube_angle_on_the_faces_side_of`]).
    ///
    /// # Errors
    ///
    /// [`Error::ACurvedFaceGoesRightRoundWithNoSeam`] for a face that has a
    /// number of such wires other than two: one alone bounds the whole torus
    /// slit along it, which is not a face, and three or more bound no band
    /// between them.
    fn cut_a_band_right_round_the_ring(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        walked: &[WalkedLoop],
        room: f64,
    ) -> Result<Vec<Vec<WalkedLoop>>> {
        let (Some(round_the_tube), Some(round_the_ring)) = (
            how_far_the_longitude_wraps(stored),
            how_far_the_second_coordinate_wraps(stored),
        ) else {
            panic!("only a torus has a wire that goes right round its second coordinate");
        };
        let [first, second] = walked else {
            return Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { face: face.key() });
        };
        let second = with_its_tube_angle_on_the_faces_side_of(second, first, round_the_tube);
        let on_their_side = [first.turned_on_its_side(), second.turned_on_its_side()];
        let Some(quarters) = self.cut_a_ring_across(
            face,
            stored,
            Chart::OnItsSide,
            &on_their_side,
            &[],
            round_the_ring,
            room,
        )?
        else {
            return Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { face: face.key() });
        };
        let mut patches = Vec::with_capacity(quarters.len());
        for quarter in quarters {
            let upright = quarter.iter().map(WalkedLoop::turned_on_its_side).collect();
            patches.extend(self.cut_each_narrow_enough(
                face,
                stored,
                Chart::Upright,
                upright,
                round_the_tube / 4.0,
                room,
            )?);
        }
        Ok(patches)
    }

    /// Cut each patch of a torus face that reaches a whole turn round the
    /// **ring** into pieces no wider than a quarter turn that way, and hand
    /// every other patch back as it came.
    ///
    /// **A face reaches a whole turn round the ring by a seam across the tube**:
    /// one wire passes the seam's points twice, a whole turn apart in the angle
    /// round the ring, which is how a file writes a fillet round a disc and how
    /// the crown of a ring is written. In the chart that patch closes, so
    /// nothing before this saw anything wrong with it, but it is a whole turn
    /// wide, and a triangle's chord across a whole turn is nothing at all: the
    /// crown folded (`TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }`,
    /// `docs/decisions.md` §280.8). So it is cut the way
    /// [`BuildingAMesh::cut_off_what_goes_too_far_round`] cuts a piece round the
    /// tube, with its corners turned on their side ([`Chart::OnItsSide`]).
    ///
    /// **A patch that does not reach a whole turn round the ring is left as it
    /// was**, bit for bit, though it may be wider than a quarter: a band three
    /// quarters of the way round measured `6π²` within its stated error in one
    /// patch per quarter of the tube (`docs/decisions.md` §343.3). A chord stops
    /// growing past half a turn, and it comes to nothing only at a whole one.
    ///
    /// # Errors
    ///
    /// [`Error::CurvedFaceWithAHoleGoesTooFarRound`] for a patch of more than one
    /// loop that reaches a whole turn round the ring: nothing here cuts a piece
    /// with a hole in it.
    fn cut_what_reaches_right_round_the_ring(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        patches: Vec<Vec<WalkedLoop>>,
        room: f64,
    ) -> Result<Vec<Vec<WalkedLoop>>> {
        let Some(round_the_ring) = how_far_the_second_coordinate_wraps(stored) else {
            return Ok(patches);
        };
        let mut cut = Vec::with_capacity(patches.len());
        for patch in patches {
            if !patch
                .iter()
                .any(|around| reaches_a_whole_turn_round_the_second_way(around, round_the_ring))
            {
                cut.push(patch);
                continue;
            }
            let [only] = patch.as_slice() else {
                return Err(Error::CurvedFaceWithAHoleGoesTooFarRound { face: face.key() });
            };
            let pieces = self.cut_each_narrow_enough(
                face,
                stored,
                Chart::OnItsSide,
                vec![only.turned_on_its_side()],
                round_the_ring / 4.0,
                room,
            )?;
            cut.extend(pieces.into_iter().map(|piece| {
                piece
                    .iter()
                    .map(WalkedLoop::turned_on_its_side)
                    .collect::<Vec<_>>()
            }));
        }
        Ok(cut)
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
            stored.where_it_pinches(),
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
            let at = self.remember_made_on(face, &there, stored.point_at(seam, latitude));
            self.on_a_cut.insert(at);
            at
        });
        let up = self.points_along_a_cut(
            face,
            stored,
            Chart::Upright,
            [seam, -pole],
            [seam, pole],
            room,
        )?;
        Ok(Some(a_strip_from_pole_to_pole(
            [seam, seam + whole],
            [&up, &up],
            [south, north],
            pole,
        )))
    }

    /// A ball face whose every loop is a hole opened into strips from pole to
    /// pole, each with the holes it holds moved into its own turn of
    /// longitude, or `None` when no meridian misses every hole or a hole does
    /// not lie strictly between its strip's two meridians.
    ///
    /// **One meridian in every stretch of longitude no hole reaches**, at its
    /// middle, so each strip holds one run of holes whose longitudes overlap
    /// one another and nothing else does. The first is in the widest stretch,
    /// and it is the seam the strips are counted from; the rest are cuts like
    /// any other. Each meridian's points are made here once and used by the
    /// strips either side of it, which is what makes their two sides one line
    /// in space, and the two poles are one point each that every strip shares.
    ///
    /// **Holes in one strip are not refused here for overlapping in
    /// longitude.** [`BuildingAMesh::cut_holes_up_to_both_poles`] chains them
    /// from the top down, and refuses them only where one does not lie wholly
    /// above the next.
    fn open_the_ball_into_strips(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        holes: &[WalkedLoop],
        room: f64,
    ) -> Result<Option<Vec<AStripAndItsHoles>>> {
        let (Some(whole), Some(pole)) = (
            how_far_the_longitude_wraps(stored),
            stored.where_it_pinches(),
        ) else {
            return Ok(None);
        };
        let spans: Vec<(f64, f64)> = holes.iter().map(longitudes_of).collect();
        if spans.iter().any(|&(least, most)| most - least >= whole) {
            return Ok(None);
        }
        let Some(seam) = the_middle_of_the_widest_stretch_no_hole_reaches(&spans, whole) else {
            return Ok(None);
        };

        let mut from_the_seam: Vec<(f64, f64, usize)> = spans
            .iter()
            .enumerate()
            .map(|(at, &(least, most))| {
                let start = (least - seam).rem_euclid(whole);
                (start, start + (most - least), at)
            })
            .collect();
        from_the_seam.sort_by(|one, other| one.0.total_cmp(&other.0));
        // **Two holes that touch go in one run whatever their longitudes say.**
        // The one longitude they share is read off each from the seam on its
        // own, the end of the one as its start plus its width and the start of
        // the other as itself, and the two round apart by an ulp either way: a
        // ball touching a prism's edge from `−y` put a cut through the point
        // they share, and neither hole lay strictly inside its strip (§364).
        let touch = |one: usize, other: usize| {
            holes[one]
                .points
                .iter()
                .any(|at| holes[other].points.contains(at))
        };
        let mut runs: Vec<(f64, f64, Vec<usize>)> = Vec::new();
        for (start, end, at) in from_the_seam {
            match runs.last_mut() {
                Some(run) if start <= run.1 || run.2.iter().any(|&member| touch(member, at)) => {
                    run.1 = run.1.max(end);
                    run.2.push(at);
                }
                _ => runs.push((start, end, vec![at])),
            }
        }
        let mut cuts = Vec::with_capacity(runs.len() + 1);
        cuts.push(seam);
        cuts.extend(
            runs.windows(2)
                .map(|pair| seam + (pair[0].1 + pair[1].0) / 2.0),
        );

        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        let [south, north] = [-pole, pole].map(|latitude| {
            let at = self.remember_made_on(face, &there, stored.point_at(seam, latitude));
            self.on_a_cut.insert(at);
            at
        });
        let mut meridians = Vec::with_capacity(cuts.len());
        for &longitude in &cuts {
            meridians.push(self.points_along_a_cut(
                face,
                stored,
                Chart::Upright,
                [longitude, -pole],
                [longitude, pole],
                room,
            )?);
        }
        cuts.push(seam + whole);

        let mut strips = Vec::with_capacity(runs.len());
        for (at, (_, _, members)) in runs.into_iter().enumerate() {
            let (first, then) = (&meridians[at], &meridians[(at + 1) % meridians.len()]);
            let (least, most) = spans[members[0]];
            let [left, right] = [cuts[at], cuts[at + 1]].map(|cut| {
                cut - whole_turns_between(
                    (cuts[at] + cuts[at + 1]) / 2.0,
                    (least + most) / 2.0,
                    whole,
                )
            });
            let strip =
                a_strip_from_pole_to_pole([left, right], [first, then], [south, north], pole);
            let mut held = Vec::with_capacity(members.len());
            for member in members {
                let hole = &holes[member];
                let (least, most) = spans[member];
                let by = whole_turns_between((least + most) / 2.0, (left + right) / 2.0, whole);
                let moved = WalkedLoop {
                    corners: hole
                        .corners
                        .iter()
                        .map(|corner| [corner[0] - by, corner[1]])
                        .collect(),
                    ..hole.clone()
                };
                let (least, most) = longitudes_of(&moved);
                if !(left < least && most < right) {
                    return Ok(None);
                }
                held.push(moved);
            }
            strips.push(AStripAndItsHoles { strip, holes: held });
        }
        Ok(Some(strips))
    }

    /// Cut each piece in two until none of them goes further round than
    /// `widest` in the first coordinate of its corners, or until no cut makes
    /// one any narrower. `chart` says which of the surface's own two that first
    /// one is.
    fn cut_each_narrow_enough(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        chart: Chart,
        mut pieces: Vec<WalkedLoop>,
        widest: f64,
        room: f64,
    ) -> Result<Vec<Vec<WalkedLoop>>> {
        let mut done: Vec<Vec<WalkedLoop>> = Vec::new();
        while let Some(piece) = pieces.pop() {
            if pieces.len() + done.len() >= MOST_PIECES_A_FACE_IS_CUT_INTO {
                return Err(Error::ACurvedFaceCannotBeCutSmallEnoughToMesh { face: face.key() });
            }
            match self.cut_in_two(face, stored, chart, &piece, widest, room)? {
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
        let cut = self.points_along_a_cut(
            face,
            stored,
            Chart::Upright,
            opened.corners[far],
            opened.corners[tip],
            room,
        )?;
        let mut one = opened.between(tip, far);
        one.take_in(&cut);
        let mut other = opened.between(far, tip);
        other.take_in(&cut.the_other_way());
        Ok(Some((one, other)))
    }

    /// Cut a **cap** — one loop right round a surface that pinches to a point,
    /// with nothing else bounding it but islands — into four patches with one
    /// loop each, or more where cuts through one island split a strip.
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
    /// **An island in the cap goes whole into the one patch it lies in**, as a
    /// hole of it, when no cut meets it. The cuts leave from corners whose
    /// meridian up to the pole meets no island where the rim has four, and the
    /// quarters are turned so that the middle of the widest stretch of
    /// longitude no island reaches is half a quarter from one of them, which
    /// puts a lone island narrower than a quarter between two cuts. Where the
    /// rim is walked at too few corners for that, a cut whose meridian meets
    /// an island is taken through it instead, and each patch beside it takes
    /// half the island into its one loop
    /// ([`BuildingAMesh::a_cut_through_an_island`]).
    ///
    /// **Two cuts can meet one island whichever way the quarters are turned**:
    /// every cut leaves from a rim corner, and the cube of side 2 turned 2 about
    /// `(2, −1, 1)` with a unit ball at `(0.8, 0.7, 0.65)` leaves a rim of four
    /// corners and an island reaching `3.24` radians round, over two of them.
    /// Each such cut goes through the island, and the strip between two of
    /// them is two patches, one under the island and one over it
    /// ([`BuildingAMesh::cuts_through_one_island`]), so a cap may come back in
    /// more than four.
    ///
    /// `None` when the surface does not pinch to a point. A cylindrical face
    /// bounded by one curve right round it is half an endless tube, which is not
    /// a face, and refusing it is right. `None` as well when one cut meets two
    /// islands, a cut through an island cannot be shown to stay inside the
    /// face, the cuts through one island cannot be put in an order that leaves
    /// each patch one loop, or an island lies in no patch.
    fn cut_a_cap_up_to_its_pole(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        cap: &WalkedLoop,
        islands: &[WalkedLoop],
        whole_turn: f64,
        room: f64,
    ) -> Result<Option<Vec<Vec<WalkedLoop>>>> {
        let Some(pinch) = stored.where_it_pinches() else {
            return Ok(None);
        };
        let Some(towards) = self.which_pole_this_cap_reaches(face, stored, cap) else {
            return Ok(None);
        };

        // A quarter turn at a time, **the way the loop itself runs**, so that
        // `between` walks a quarter of the way round and not three quarters.
        let way = if runs_up(cap) { 1.0 } else { -1.0 };
        let cap = &starting_off_the_poles(cap, pinch);
        let Some(from) = where_the_quarters_start(cap, islands, way, whole_turn) else {
            return Ok(None);
        };
        let quarters: [f64; 4] =
            [0, 1, 2, 3].map(|step| from + way * f64::from(step) * whole_turn / 4.0);
        let cap = &with_a_corner_where_a_pole_side_passes(cap, &quarters, pinch);
        let along =
            corners_the_quarter_cuts_leave_from(cap, quarters, pinch, towards, whole_turn, islands);

        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        let at_the_pole = stored.point_at(from, towards);
        if !at_the_pole.is_finite() {
            return Ok(None);
        }
        let at_the_pole = self.remember_made_on(face, &there, at_the_pole);
        self.on_a_cut.insert(at_the_pole);

        let mut met: Vec<Option<usize>> = Vec::with_capacity(4);
        for &corner in &along {
            let meets: Vec<usize> = (0..islands.len())
                .filter(|&island| {
                    its_meridian_meets_an_island(
                        cap.corners[corner],
                        towards,
                        core::slice::from_ref(&islands[island]),
                        whole_turn,
                    )
                })
                .collect();
            match meets[..] {
                [] => met.push(None),
                [island] => met.push(Some(island)),
                _ => return Ok(None),
            }
        }

        let mut cuts: Vec<Option<ACutUpToThePole>> = (0..4).map(|_| None).collect();
        let mut split: Vec<Option<AStripAnIslandSplits>> = (0..4).map(|_| None).collect();
        let mut cut_through = vec![false; islands.len()];
        for (at, &corner) in along.iter().enumerate() {
            self.on_a_cut.insert(cap.points[corner]);
            let leaving = cap.corners[corner];
            let Some(island) = met[at] else {
                let up = self.points_along_a_cut(
                    face,
                    stored,
                    Chart::Upright,
                    leaving,
                    [leaving[0], towards],
                    room,
                )?;
                cuts[at] = Some(ACutUpToThePole {
                    down: up.the_other_way(),
                    up,
                });
                continue;
            };
            if cut_through[island] {
                continue;
            }
            cut_through[island] = true;
            let meeting: Vec<usize> = (0..4).filter(|&cut| met[cut] == Some(island)).collect();
            if meeting.len() > 1 {
                let Some(through) = self.cuts_through_one_island(
                    face,
                    stored,
                    CutsThroughOneIsland {
                        cap,
                        along,
                        meeting: &meeting,
                        islands,
                        which: island,
                        way,
                        towards,
                        whole: whole_turn,
                    },
                    room,
                )?
                else {
                    return Ok(None);
                };
                for OneOfTheCutsThroughAnIsland { place, cut, strip } in through {
                    cuts[place] = Some(cut);
                    split[place] = strip;
                }
                continue;
            }
            let others: Vec<[f64; 2]> = along
                .iter()
                .enumerate()
                .filter(|&(other, _)| other != at)
                .map(|(_, &other)| cap.corners[other])
                .collect();
            let Some(through) = self.a_cut_through_an_island(
                face,
                stored,
                ACutThroughAnIsland {
                    cap,
                    leaving,
                    islands,
                    which: island,
                    others: &others,
                    towards,
                    whole: whole_turn,
                },
                room,
            )?
            else {
                return Ok(None);
            };
            cuts[at] = Some(through);
        }
        let cuts: Vec<ACutUpToThePole> = cuts
            .into_iter()
            .map(|cut| cut.expect("every one of the four cuts up to the pole was made"))
            .collect();

        let mut patches = Vec::with_capacity(5);
        for step in 0..4 {
            let next = (step + 1) % 4;
            let mut one = unwrapped(&cap.between(along[step], along[next]), whole_turn);
            if let Some(strip) = &split[step] {
                one.take_in_unwrapped(&strip.under, whole_turn);
                let mut over = unwrapped(&strip.over, whole_turn);
                with_the_pole_side(&mut over, towards, at_the_pole);
                over.take_in_unwrapped(&strip.down_from_the_pole, whole_turn);
                if how_much_of_the_chart_this_loop_encloses(&one) <= 0.0
                    || how_much_of_the_chart_this_loop_encloses(&over) <= 0.0
                {
                    return Ok(None);
                }
                patches.push(vec![one]);
                patches.push(vec![over]);
                continue;
            }
            one.take_in_unwrapped(&cuts[next].up, whole_turn);
            with_the_pole_side(&mut one, towards, at_the_pole);
            one.take_in_unwrapped(&cuts[step].down, whole_turn);
            patches.push(vec![one]);
        }
        for (island, _) in islands.iter().zip(&cut_through).filter(|&(_, &cut)| !cut) {
            let Some((patch, moved)) = the_patch_an_island_lies_in(&patches, island, whole_turn)
            else {
                return Ok(None);
            };
            patches[patch].push(moved);
        }
        Ok(Some(patches))
    }

    /// A cut from a cap's rim up to its pole whose meridian meets an island:
    /// straight from the rim corner to the island's nearest corner it can
    /// reach, round the island to its corner nearest the pole, and up that
    /// corner's meridian to the pole. The patch ahead of the cut takes the
    /// island's half on its side and the patch behind takes the other, so the
    /// island is a stretch of each patch's one loop and not a hole in either.
    ///
    /// **Each leg is shown to stay inside the face.** The leg to the island
    /// crosses no side of the rim or of any island and no other cut, and only
    /// touches the rim and the island at its own two ends. The leg up from
    /// the island meets neither the rim nor another island, and leaves its own
    /// island at its corner nearest the pole, past which no side of it reaches.
    /// `None` when no corner of the island can be reached that way.
    fn a_cut_through_an_island(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        cut: ACutThroughAnIsland<'_>,
        room: f64,
    ) -> Result<Option<ACutUpToThePole>> {
        let island = &cut.the_island_in_its_turn();
        let top = the_corner_furthest(island, cut.towards.signum());
        if !cut.leaves_up_from(island, top) {
            return Ok(None);
        }
        let mut by_distance: Vec<usize> =
            (0..island.corners.len()).filter(|&at| at != top).collect();
        cut.nearest_first(island, &mut by_distance);
        let Some(bottom) = by_distance.into_iter().find(|&at| cut.reaches(island, at)) else {
            return Ok(None);
        };
        let [to_the_island, to_the_pole] =
            self.the_two_legs_of(face, stored, &cut, island, [bottom, top], room)?;
        Ok(Some(ACutUpToThePole::round_an_island(
            island,
            [bottom, top],
            &to_the_island,
            &to_the_pole,
        )))
    }

    /// Two or more of a cap's cuts up to the pole whose meridians all meet one
    /// island, each cut through it: each with its place among the four, and
    /// each but the last along the island with the strip between it and the
    /// next.
    ///
    /// **A strip between two cuts through one island is two patches, each of
    /// one loop**: under the island, from the rim up the cut ahead to the
    /// island, back along its underside and down the cut behind; and over it,
    /// along its top from the cut behind to the cut ahead, up to the pole and
    /// down the cut behind. A single cut through an island leaves it at its
    /// highest corner, so two of them would leave up one meridian; here each
    /// cut reaches the island at a corner of its own and leaves it at another.
    /// Round the island the way it is walked, they have to come in the order
    /// the patches take the island's sides in: the first cut's bottom, every
    /// top from the first cut's to the last's, then every bottom from the
    /// last cut's back to the second's. The patch behind the first cut and the
    /// patch ahead of the last take the island's two ends, as the patches
    /// either side of a single cut take its two halves.
    ///
    /// The cuts are put in the order they meet the island, the way the rim
    /// runs, which on a rim of four corners that the island reaches most of the
    /// way round is not the order of the four. A top is the corner, among
    /// those whose meridian up to the pole is clear
    /// ([`ACutThroughAnIsland::leaves_up_from`]), nearest its cut's own
    /// meridian; a bottom is the corner under the tops nearest the cut's rim
    /// corner that a leg from it reaches ([`ACutThroughAnIsland::reaches`]).
    /// No leg of one cut meets a leg of another.
    ///
    /// `None` when the cuts are not next to each other along the rim, a cut has
    /// no such top or bottom, or they do not come in that order.
    fn cuts_through_one_island(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        through: CutsThroughOneIsland<'_>,
        room: f64,
    ) -> Result<Option<Vec<OneOfTheCutsThroughAnIsland>>> {
        let CutsThroughOneIsland {
            cap,
            along,
            meeting,
            islands,
            which,
            way,
            towards,
            whole,
        } = through;
        let others: Vec<Vec<[f64; 2]>> = meeting
            .iter()
            .map(|&place| {
                (0..4)
                    .filter(|&other| other != place)
                    .map(|other| cap.corners[along[other]])
                    .collect()
            })
            .collect();
        let mut run: Vec<(usize, ACutThroughAnIsland<'_>)> = meeting
            .iter()
            .zip(&others)
            .map(|(&place, others)| {
                let cut = ACutThroughAnIsland {
                    cap,
                    leaving: cap.corners[along[place]],
                    islands,
                    which,
                    others,
                    towards,
                    whole,
                };
                (place, cut)
            })
            .collect();
        let (least, most) = longitudes_of(&islands[which]);
        let middle = (least + most) / 2.0;
        let where_it_meets = |cut: &ACutThroughAnIsland<'_>| {
            way * (cut.leaving[0] + whole_turns_between(middle, cut.leaving[0], whole))
        };
        run.sort_by(|(_, one), (_, other)| where_it_meets(one).total_cmp(&where_it_meets(other)));
        if run.windows(2).any(|pair| pair[1].0 != (pair[0].0 + 1) % 4) {
            return Ok(None);
        }
        let moved: Vec<WalkedLoop> = run
            .iter()
            .map(|(_, cut)| cut.the_island_in_its_turn())
            .collect();

        let count = islands[which].corners.len();
        let mut tops = Vec::with_capacity(run.len());
        for ((_, cut), island) in run.iter().zip(&moved) {
            let Some(top) = (0..count)
                .filter(|&top| cut.leaves_up_from(island, top))
                .min_by(|&one, &other| {
                    let away = |at: usize| (island.corners[at][0] - cut.leaving[0]).abs();
                    away(one).total_cmp(&away(other))
                })
            else {
                return Ok(None);
            };
            tops.push(top);
        }
        let (first_top, last_top) = (tops[0], tops[tops.len() - 1]);
        let past = |from: usize, at: usize| (at + count - from) % count;
        let underneath: Vec<usize> = (1..count)
            .map(|step| (last_top + step) % count)
            .take_while(|&at| at != first_top)
            .collect();
        let mut bottoms = Vec::with_capacity(run.len());
        for ((_, cut), island) in run.iter().zip(&moved) {
            let mut by_distance = underneath.clone();
            cut.nearest_first(island, &mut by_distance);
            let Some(bottom) = by_distance.into_iter().find(|&at| cut.reaches(island, at)) else {
                return Ok(None);
            };
            bottoms.push(bottom);
        }
        let in_order = tops
            .windows(2)
            .all(|pair| past(first_top, pair[0]) < past(first_top, pair[1]))
            && bottoms
                .windows(2)
                .all(|pair| past(last_top, pair[0]) > past(last_top, pair[1]));
        if !in_order {
            return Ok(None);
        }
        let low: Vec<[[f64; 2]; 2]> = run
            .iter()
            .zip(&moved)
            .zip(&bottoms)
            .map(|(((_, cut), island), &bottom)| [cut.leaving, island.corners[bottom]])
            .collect();
        let high: Vec<[f64; 2]> = moved
            .iter()
            .zip(&tops)
            .map(|(island, &top)| island.corners[top])
            .collect();
        let legs_meet = low.iter().enumerate().any(|(one, &leg)| {
            [-whole, 0.0, whole].into_iter().any(|by| {
                let moved_by = |[longitude, latitude]: [f64; 2]| [longitude + by, latitude];
                low.iter().enumerate().any(|(other, &[from, to])| {
                    other != one && two_sides_meet(leg, [moved_by(from), moved_by(to)])
                }) || high.iter().any(|&[longitude, from]| {
                    a_side_meets_a_cut(leg, longitude + by, [from, towards])
                })
            })
        });
        if legs_meet {
            return Ok(None);
        }

        let mut legs = Vec::with_capacity(run.len());
        for (((_, cut), island), (&bottom, &top)) in
            run.iter().zip(&moved).zip(bottoms.iter().zip(&tops))
        {
            legs.push(self.the_two_legs_of(face, stored, cut, island, [bottom, top], room)?);
        }
        let mut made = Vec::with_capacity(run.len());
        for (at, (place, _)) in run.iter().enumerate() {
            let [to_the_island, to_the_pole] = &legs[at];
            let cut = ACutUpToThePole::round_an_island(
                &moved[at],
                [bottoms[at], tops[at]],
                to_the_island,
                to_the_pole,
            );
            let strip = legs
                .get(at + 1)
                .map(|[ahead_to_the_island, ahead_to_the_pole]| {
                    let mut under = ahead_to_the_island.clone();
                    under.take_in(&moved[at + 1].between(bottoms[at + 1], bottoms[at]));
                    under.take_in(&to_the_island.the_other_way());
                    let mut over = moved[at].between(tops[at], tops[at + 1]);
                    over.take_in(ahead_to_the_pole);
                    AStripAnIslandSplits {
                        under,
                        over,
                        down_from_the_pole: to_the_pole.the_other_way(),
                    }
                });
            made.push(OneOfTheCutsThroughAnIsland {
                place: *place,
                cut,
                strip,
            });
        }
        Ok(Some(made))
    }

    /// The points strictly along a cut's two straight legs through an island:
    /// from its rim corner to the island's corner `bottom`, and from the
    /// island's corner `top` up its meridian to the pole. Both corners are on a
    /// cut from now on.
    fn the_two_legs_of(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        cut: &ACutThroughAnIsland<'_>,
        island: &WalkedLoop,
        [bottom, top]: [usize; 2],
        room: f64,
    ) -> Result<[WalkedLoop; 2]> {
        let to_the_island = self.points_along_a_cut(
            face,
            stored,
            Chart::Upright,
            cut.leaving,
            island.corners[bottom],
            room,
        )?;
        let up_from = island.corners[top];
        let to_the_pole = self.points_along_a_cut(
            face,
            stored,
            Chart::Upright,
            up_from,
            [up_from[0], cut.towards],
            room,
        )?;
        self.on_a_cut
            .extend([island.points[bottom], island.points[top]]);
        Ok([to_the_island, to_the_pole])
    }

    /// Which of a surface's two poles a cap reaches, as the latitude of it:
    /// **the one on the left of the walk round its rim.**
    ///
    /// A wire keeps its face's material on its left about the stored surface's
    /// own normal, whichever way the face is used (`holds.rs`,
    /// `which_side_of_its_neighbour_a_side_keeps`, says why), and a ball's and
    /// a cone's two coordinates are right-handed about that normal
    /// (`every_surface_but_a_torus_crosses_its_two_coordinates_out_of_the_material`),
    /// so left of the longitude growing is the latitude growing. A cap goes
    /// right round, so its walk gains a whole turn or loses one, and
    /// [`runs_up`] says which: a cap walked up reaches the pinch above every
    /// corner of its rim, and one walked down the pinch below them all. That
    /// is the count [`BuildingAMesh::cut_a_cap_up_to_its_pole`] steps its
    /// quarters by, so the pole and the way round each patch come from one
    /// reading and cannot disagree. No sign is asked of the arithmetic here: a
    /// walk that goes right round is a whole turn from nothing.
    ///
    /// **What stood here read the pole off the surface across the rim's first
    /// edge**: the step along that edge against `∇S × ∇T` said which side of
    /// that surface the material is on, and the first pole on that side was
    /// taken. That needs a surface across that puts the two poles on different
    /// sides, and a block crossing a ball round its south pole leaves two caps
    /// with none (`a_cap_whose_pole_no_partner_separates.rs`). The cap the block
    /// keeps is cut by `x = ±1` and `y = ±1`, with both poles on the material's
    /// side of all four, so the north pole was taken for a cap round the south
    /// one, its patches were drawn inside out, and the fuse refused
    /// `FacesDoNotAgreeWhichWayTheyFace`. The cap the ball keeps has the ball
    /// itself across every edge of its rim, because the fuse cuts the skin
    /// along the whole of each circle and never joins the pieces back, so there
    /// was no surface across to read and the fuse refused
    /// `ACurvedFaceGoesRightRoundWithNoSeam`. The ball less a block over one
    /// corner of it was refused the same way (`a_ball_less_a_wedge.rs`).
    ///
    /// **Never the pole the rim runs through.** That pole is on the face's
    /// edge, not inside it, so a cap whose rim runs through one pole reaches
    /// the other, and its walk keeps the other on its left. The walk put the
    /// pole on its rim as a corner at its latitude
    /// ([`put_in_the_poles_a_crossing_passes`]), so that pole is not
    /// strictly beyond every corner and is never taken
    /// (`a_ball_less_a_cap_whose_rim_runs_through_its_north_pole.rs`,
    /// `docs/decisions.md` §199.4).
    ///
    /// `None` when no pinch is strictly beyond every corner the way the walk
    /// keeps the material: a cone's apex on the side away from the material,
    /// or a rim with a corner at the pole it would reach. Neither bounds a
    /// face, and the caller refuses it.
    fn which_pole_this_cap_reaches(
        &self,
        _: FaceView<'_>,
        stored: &Surface,
        cap: &WalkedLoop,
    ) -> Option<f64> {
        let pinch = stored.where_it_pinches()?;
        let up = runs_up(cap);
        [pinch, -pinch].into_iter().find(|&towards| {
            cap.corners.iter().all(|&[_, latitude]| {
                if up {
                    latitude < towards
                } else {
                    latitude > towards
                }
            })
        })
    }

    /// Cut a **ring** — two loops that each go right round the surface, and
    /// the `islands` between them — into four patches, each with one loop and
    /// whichever islands no cut met.
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
    /// **Round the first coordinate of the corners it is handed**, which `chart`
    /// says is the surface's own first or its second. A torus band between two
    /// circles round the tube's middle goes right round the ring instead, and
    /// [`BuildingAMesh::cut_a_band_right_round_the_ring`] hands it here on its
    /// side, so that the four cuts run across the band from one circle to the
    /// other.
    ///
    /// **A cut whose straight line meets an island goes round it**: straight
    /// from its corner on one loop to the island's corner nearest that loop,
    /// round the island to its corner nearest the other loop, and straight on
    /// from there. The patch either side takes the island's half on its side
    /// into its one loop ([`ACutAcrossARing::round_an_island`]). An island no
    /// cut meets goes whole into the one patch it lies in, as a hole of it, as
    /// on a cap ([`BuildingAMesh::cut_a_cap_up_to_its_pole`]). With no islands
    /// every cut is straight, and the patches are the ones a ring was always
    /// cut into, bit for bit.
    ///
    /// `None` when the face is not a ring: two loops is not enough, they both
    /// have to go right round, or there is nothing across to cut to. `None` as
    /// well when a cut meets two islands, two cuts meet one, a leg round an
    /// island meets a loop, another island or another cut, or an island lies
    /// in no patch.
    #[expect(
        clippy::too_many_arguments,
        reason = "the face, its surface and chart, its two kinds of loop, the turn and the room are each read"
    )]
    fn cut_a_ring_across(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        chart: Chart,
        walked: &[WalkedLoop],
        islands: &[WalkedLoop],
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
        let pinch = match chart {
            Chart::Upright => stored.where_it_pinches(),
            Chart::OnItsSide => None,
        };
        let (up, down) =
            with_a_corner_where_a_pole_side_of_either_passes(up, down, quarters, pinch, whole_turn);
        let along_the_top = quarters.map(|at| nearest_longitude(&up, at, pinch));
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
        let along_the_bottom = quarters
            .map(|at| nearest_longitude(&down, in_the_turn_of(&down, at, whole_turn), pinch));

        let mut ends = Vec::with_capacity(4);
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
            ends.push([leaving, arriving]);
        }
        let islands: Vec<WalkedLoop> = islands
            .iter()
            .map(|island| in_the_same_turn_as(island, &up, whole_turn))
            .collect();
        let Some(through) = where_a_rings_cuts_go_round_islands(&ends, &islands, whole_turn) else {
            return Ok(None);
        };
        if !the_legs_round_islands_stay_inside_a_ring(
            [&up, &down],
            &islands,
            &ends,
            &through,
            whole_turn,
        ) {
            return Ok(None);
        }

        let mut cuts = Vec::with_capacity(4);
        for (&[leaving, arriving], round) in ends.iter().zip(&through) {
            let Some(round) = round else {
                let across =
                    self.points_along_a_cut(face, stored, chart, leaving, arriving, room)?;
                cuts.push(ACutAcrossARing {
                    back: across.the_other_way(),
                    across,
                });
                continue;
            };
            let [reached, left] = round.corners;
            let to_the_island = self.points_along_a_cut(
                face,
                stored,
                chart,
                leaving,
                round.island.corners[reached],
                room,
            )?;
            let from_the_island = self.points_along_a_cut(
                face,
                stored,
                chart,
                round.island.corners[left],
                arriving,
                room,
            )?;
            self.on_a_cut
                .extend([round.island.points[reached], round.island.points[left]]);
            cuts.push(ACutAcrossARing::round_an_island(
                round,
                arriving[1] > leaving[1],
                &to_the_island,
                &from_the_island,
            ));
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
            one.take_in_unwrapped(&cuts[next].across, whole_turn);
            one.take_in_unwrapped(
                &down
                    .between(along_the_bottom[step], along_the_bottom[next])
                    .the_other_way(),
                whole_turn,
            );
            one.take_in_unwrapped(&cuts[step].back, whole_turn);
            patches.push(vec![one]);
        }
        let met: Vec<usize> = through.iter().flatten().map(|round| round.which).collect();
        for (_, island) in islands
            .iter()
            .enumerate()
            .filter(|(at, _)| !met.contains(at))
        {
            let Some((patch, moved)) = the_patch_an_island_lies_in(&patches, island, whole_turn)
            else {
                return Ok(None);
            };
            patches[patch].push(moved);
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
        let (outer, hole) = match [first, second].map(how_much_of_the_chart_this_loop_encloses) {
            [one, other] if one > 0.0 && other < 0.0 => (first, second),
            [one, other] if one < 0.0 && other > 0.0 => (second, first),
            _ => return Ok(None),
        };
        let Some(pieces) = self.cut_holes_up_to_both_poles(
            face,
            stored,
            outer,
            core::slice::from_ref(hole),
            room,
        )?
        else {
            return Ok(None);
        };
        let [one, other] = <[WalkedLoop; 2]>::try_from(pieces)
            .unwrap_or_else(|pieces| panic!("one hole cuts a face in two, not {}", pieces.len()));
        Ok(Some((one, other)))
    }

    /// [`BuildingAMesh::cut_a_hole_up_to_both_poles`] for an outer loop with
    /// any number of holes in it, one above the other: the cut up to the top
    /// pole leaves from the highest corner of the highest hole, the cut down
    /// to the bottom one from the lowest corner of the lowest, and a cut
    /// between each hole and the next joins the lowest corner of the one to
    /// the highest corner of the other. One hole is cut exactly as that one
    /// is, into two pieces.
    ///
    /// **A cut between two holes stays inside the face when each hole lies
    /// wholly above the next.** A side is a straight line between two corners
    /// in these coordinates, so a hole lies within the latitudes of its highest
    /// and lowest corners, and the cut runs strictly between the latitudes of
    /// the two holes it joins, where no hole reaches. The outer loop is only
    /// ever a strip from pole to pole when there is more than one hole
    /// ([`BuildingAMesh::open_the_ball_into_strips`]), which holds every
    /// straight line between two points inside it. The cuts up to the poles
    /// are the ones the one-hole case already shows stay inside. Holes that do
    /// not lie one above the other are cut side by side instead
    /// ([`BuildingAMesh::cut_holes_side_by_side_up_to_both_poles`]).
    ///
    /// **A cut between two holes carries a point of its own.** Its two ends
    /// are corners of the model, and a face across both holes would be free to
    /// draw the same pair, which is the fold
    /// [`BuildingAMesh::cut_in_two`] gives the same answer to.
    fn cut_holes_up_to_both_poles(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        outer: &WalkedLoop,
        holes: &[WalkedLoop],
        room: f64,
    ) -> Result<Option<Vec<WalkedLoop>>> {
        if stored.where_it_pinches() != Some(core::f64::consts::FRAC_PI_2)
            || outer.goes_right_round
            || holes.iter().any(|hole| hole.goes_right_round)
        {
            return Ok(None);
        }
        if holes.iter().any(|hole| {
            (0..hole.points.len())
                .any(|at| hole.points[at] == hole.points[(at + 1) % hole.points.len()])
        }) {
            return Ok(None);
        }
        let mut from_the_top: Vec<(&WalkedLoop, [usize; 2])> = holes
            .iter()
            .map(|hole| {
                (
                    hole,
                    [
                        the_corner_furthest(hole, 1.0),
                        the_corner_furthest(hole, -1.0),
                    ],
                )
            })
            .collect();
        from_the_top.sort_by(|(one, [one_highest, _]), (other, [other_highest, _])| {
            other.corners[*other_highest][1].total_cmp(&one.corners[*one_highest][1])
        });
        if from_the_top.windows(2).any(|pair| {
            let [(upper, [_, its_lowest]), (lower, [its_highest, _])] = pair else {
                unreachable!("windows of two");
            };
            lower.corners[*its_highest][1] >= upper.corners[*its_lowest][1]
        }) {
            return self.cut_holes_side_by_side_up_to_both_poles(face, stored, outer, holes, room);
        }
        let (top_hole, [highest, _]) = from_the_top[0];
        let (bottom_hole, [_, lowest]) = from_the_top[from_the_top.len() - 1];
        let [up_at, down_at] =
            [(top_hole, highest, 1.0), (bottom_hole, lowest, -1.0)].map(|(hole, corner, way)| {
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
        let top_corner = [top_hole.corners[highest][0], up_at.1];
        let bottom_corner = [bottom_hole.corners[lowest][0], down_at.1];
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
        let to_the_top = self.points_along_a_cut(
            face,
            stored,
            Chart::Upright,
            top_hole.corners[highest],
            top_corner,
            room,
        )?;
        let from_the_bottom = self.points_along_a_cut(
            face,
            stored,
            Chart::Upright,
            bottom_corner,
            bottom_hole.corners[lowest],
            room,
        )?;
        let mut up_from_each_hole_below = Vec::with_capacity(from_the_top.len() - 1);
        for pair in from_the_top.windows(2) {
            let [(upper, [_, its_lowest]), (lower, [its_highest, _])] = pair else {
                unreachable!("windows of two");
            };
            let leaving = lower.corners[*its_highest];
            let arriving = upper.corners[*its_lowest];
            let steps = how_many_steps(stored, leaving, arriving, room).max(2);
            up_from_each_hole_below.push(self.points_along_a_cut_in(
                face,
                stored,
                Chart::Upright,
                [leaving, arriving],
                steps,
            )?);
        }

        one.take_in(&from_the_bottom);
        for (at, &(hole, [highest, lowest])) in from_the_top.iter().enumerate().rev() {
            one.take_in(&hole.between(lowest, highest));
            if at > 0 {
                one.take_in(&up_from_each_hole_below[at - 1]);
            }
        }
        one.take_in(&to_the_top);
        other.take_in(&to_the_top.the_other_way());
        for (at, &(hole, [highest, lowest])) in from_the_top.iter().enumerate() {
            other.take_in(&hole.between(highest, lowest));
            if let Some(cut) = up_from_each_hole_below.get(at) {
                other.take_in(&cut.the_other_way());
            }
        }
        other.take_in(&from_the_bottom.the_other_way());

        if [&one, &other]
            .iter()
            .any(|piece| how_much_of_the_chart_this_loop_encloses(piece) <= 0.0)
        {
            return Ok(None);
        }
        for &(hole, [highest, lowest]) in &from_the_top {
            self.on_a_cut
                .extend([hole.points[highest], hole.points[lowest]]);
        }
        self.on_a_cut
            .extend([outer.points[up_at.0], outer.points[down_at.0]]);
        Ok(Some(vec![one, other]))
    }

    /// Cut a strip of a ball holding several holes, none wholly above
    /// another, into one more piece than it has holes: each hole is cut from
    /// its highest corner straight up to the top pole and from its lowest
    /// straight down to the bottom one, and each piece runs between two holes'
    /// cuts, or between the strip's own side and the first or last hole's.
    /// `None` when a cut cannot be shown to stay inside the face.
    ///
    /// **A cut stays inside the face when the first thing it meets is a pole
    /// side.** No side of a hole reaches above its highest corner or below its
    /// lowest, so the cut leaves its own hole straight into the face, and it is
    /// refused when it meets a side of another hole, or of the strip, before
    /// the pole. Two cuts along one longitude would lie on each other, and are
    /// refused too, so every cut runs along a longitude of its own. Then no
    /// two cuts cross, and no cut crosses a hole, so the cuts of two holes
    /// cannot swap order between the top pole and the bottom one: holes put in
    /// order along the one are in order along the other, and a strip where
    /// they are not is refused, not cut.
    ///
    /// The holes found here overlap in longitude: a turned cube's corner and
    /// one of its faces both poking into a ball leave two holes side by side,
    /// the one's end a few hundredths of a radian past the other's start, and
    /// no meridian between them (`docs/decisions.md` §402).
    fn cut_holes_side_by_side_up_to_both_poles(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        outer: &WalkedLoop,
        holes: &[WalkedLoop],
        room: f64,
    ) -> Result<Option<Vec<WalkedLoop>>> {
        let mut cut: Vec<AHoleCutStraightUpAndDown<'_>> = Vec::with_capacity(holes.len());
        for (at, hole) in holes.iter().enumerate() {
            let (highest, lowest) = (
                the_corner_furthest(hole, 1.0),
                the_corner_furthest(hole, -1.0),
            );
            let reaches = [(highest, 1.0), (lowest, -1.0)].map(|(corner, way)| {
                let [longitude, latitude] = hole.corners[corner];
                let (side, pole) =
                    the_pole_side_a_line_from_a_hole_reaches(outer, longitude, latitude, way)?;
                let another_is_in_the_way = holes.iter().enumerate().any(|(other, around)| {
                    other != at && a_line_meets(around, longitude, [latitude, pole])
                });
                (!another_is_in_the_way).then_some((side, pole))
            });
            let [Some(up), Some(down)] = reaches else {
                return Ok(None);
            };
            cut.push(AHoleCutStraightUpAndDown {
                hole,
                highest,
                lowest,
                up,
                down,
            });
        }
        cut.sort_by(|one, other| one.longitude_up().total_cmp(&other.longitude_up()));
        let (top, bottom) = (cut[0].up.0, cut[0].down.0);
        let in_order = cut.windows(2).all(|pair| {
            pair[0].longitude_up() < pair[1].longitude_up()
                && pair[0].longitude_down() < pair[1].longitude_down()
        });
        let sides =
            |end: usize| outer.corners[end][0] - outer.corners[(end + 1) % outer.corners.len()][0];
        if !in_order
            || top == bottom
            || sides(top) <= 0.0
            || sides(bottom) >= 0.0
            || cut
                .iter()
                .any(|each| each.up.0 != top || each.down.0 != bottom)
        {
            return Ok(None);
        }

        let mut opened = WalkedLoop::a_stretch(outer.corners.len() + 2 * cut.len());
        let mut top_at = vec![0; cut.len()];
        let mut bottom_at = vec![0; cut.len()];
        for at in 0..outer.corners.len() {
            opened.corners.push(outer.corners[at]);
            opened.points.push(outer.points[at]);
            if at == top {
                for (each, hole) in cut.iter().enumerate().rev() {
                    top_at[each] = opened.corners.len();
                    opened.corners.push([hole.longitude_up(), hole.up.1]);
                    opened.points.push(outer.points[top]);
                }
            }
            if at == bottom {
                for (each, hole) in cut.iter().enumerate() {
                    bottom_at[each] = opened.corners.len();
                    opened.corners.push([hole.longitude_down(), hole.down.1]);
                    opened.points.push(outer.points[bottom]);
                }
            }
        }
        let mut to_the_top = Vec::with_capacity(cut.len());
        let mut from_the_bottom = Vec::with_capacity(cut.len());
        for (each, hole) in cut.iter().enumerate() {
            to_the_top.push(self.points_along_a_cut(
                face,
                stored,
                Chart::Upright,
                hole.hole.corners[hole.highest],
                opened.corners[top_at[each]],
                room,
            )?);
            from_the_bottom.push(self.points_along_a_cut(
                face,
                stored,
                Chart::Upright,
                opened.corners[bottom_at[each]],
                hole.hole.corners[hole.lowest],
                room,
            )?);
        }

        let last = cut.len() - 1;
        let mut pieces = Vec::with_capacity(cut.len() + 1);
        let mut first = opened.between(top_at[0], bottom_at[0]);
        first.take_in(&from_the_bottom[0]);
        first.take_in(&cut[0].left_half());
        first.take_in(&to_the_top[0]);
        pieces.push(first);
        for each in 0..last {
            let next = each + 1;
            let mut piece = opened.between(top_at[next], top_at[each]);
            piece.take_in(&to_the_top[each].the_other_way());
            piece.take_in(&cut[each].right_half());
            piece.take_in(&from_the_bottom[each].the_other_way());
            piece.take_in(&opened.between(bottom_at[each], bottom_at[next]));
            piece.take_in(&from_the_bottom[next]);
            piece.take_in(&cut[next].left_half());
            piece.take_in(&to_the_top[next]);
            pieces.push(piece);
        }
        let mut after = opened.between(bottom_at[last], top_at[last]);
        after.take_in(&to_the_top[last].the_other_way());
        after.take_in(&cut[last].right_half());
        after.take_in(&from_the_bottom[last].the_other_way());
        pieces.push(after);

        if pieces
            .iter()
            .any(|piece| how_much_of_the_chart_this_loop_encloses(piece) <= 0.0)
        {
            return Ok(None);
        }
        for each in &cut {
            self.on_a_cut.extend([
                each.hole.points[each.highest],
                each.hole.points[each.lowest],
            ]);
        }
        self.on_a_cut
            .extend([outer.points[top], outer.points[bottom]]);
        Ok(Some(pieces))
    }

    /// The points strictly between two corners of one face, walked along the
    /// straight line between them in the surface's own two coordinates, and
    /// as many of them as [`how_many_steps`] says the line needs.
    ///
    /// The two corners and the points that come back are in the order `chart`
    /// says; only where a point is put on the surface are its two numbers given
    /// back in the surface's own order.
    fn points_along_a_cut(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        chart: Chart,
        leaving: [f64; 2],
        arriving: [f64; 2],
        room: f64,
    ) -> Result<WalkedLoop> {
        let steps = how_many_steps(
            stored,
            chart.as_the_surface_takes_them(leaving),
            chart.as_the_surface_takes_them(arriving),
            room,
        );
        self.points_along_a_cut_in(face, stored, chart, [leaving, arriving], steps)
    }

    /// The points strictly between two corners of one face, `steps` steps
    /// apart along the straight line between them.
    fn points_along_a_cut_in(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        chart: Chart,
        [leaving, arriving]: [[f64; 2]; 2],
        steps: usize,
    ) -> Result<WalkedLoop> {
        if self.points.len() + steps > MOST_POINTS_A_MESH_WILL_HOLD {
            return Err(Error::MoreMeshPointsThanASolidWillHold {
                most: MOST_POINTS_A_MESH_WILL_HOLD,
            });
        }
        let there = back_out_of_the_frame_this_face_is_meshed_in(face);
        let mut along = WalkedLoop::a_stretch(steps - 1);
        for step in 1..steps {
            let part = step as f64 / steps as f64;
            let sits_at =
                [0, 1].map(|which| leaving[which] + part * (arriving[which] - leaving[which]));
            let [first, second] = chart.as_the_surface_takes_them(sits_at);
            let on_it = stored.point_at(first, second);
            along.corners.push(sits_at);
            let at = self.remember_made_on(face, &there, on_it);
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
    /// asking for.
    ///
    /// **A piece no further over `widest` than its longitudes can be read off
    /// by is not cut either** ([`HOW_FAR_A_READ_LONGITUDE_CAN_BE_OFF`]), because
    /// a cut can make it smaller by a rounding and what it cuts off is a sliver.
    /// A quarter bend of ring 8 and tube 1 from `(7, 0, 0)` round to
    /// `(0, 7, 0)`, turned 27.1 degrees about `x`, written to a file and read
    /// back, reads its seam's points an ulp either side of `π`, so the quarter
    /// of its wall between the cut at `π/2` and the seam went
    /// `1.5707963267948974` round against `1.5707963267948966`. The middle of
    /// that is a rounding nearer the seam than the cut, so [`where_to_cut`]
    /// pulled both ends onto the seam's two corners; [`a_chord_stays_inside`]
    /// passed the chord, since the seam's points between them are an ulp off
    /// it; and the half between the chord and the seam was `1.8e-15` wide, the
    /// other half two ulps narrower than the whole. The triangulator fanned the
    /// sliver from its one corner off the line and left the rest of it empty,
    /// and at a deflection of 1.8 the read-back bend was refused
    /// `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 1, used: 1 }` at 20 of
    /// 3600 tenth-degree turns; at 137 degrees and a deflection of 2 the same
    /// sliver was refused `CornersCoincide`
    /// (`a_quarter_bend_turned_about_x_measures_after_its_file.rs`).
    ///
    /// **A sliver comes only off a piece within that margin of `widest`.** A
    /// half only roundings wide has both ends of the cut at one end of the
    /// piece's longitudes, or the other half would reach as far round as the
    /// whole. Each end is the corner nearer the middle of a side the middle
    /// crosses, so that side's other corner is about as far the other way: it
    /// runs the piece's whole width, and no side runs further than a quarter
    /// turn. **And a piece left whole meshes**: a triangle's chord grows with
    /// the turn it spans until that turn reaches a half, so the triangulator
    /// sees how far its triangles stray on a piece of any width short of half a
    /// turn, and this margin is `6e-5` of a quarter. Cutting it instead buys a
    /// rounding of width and a patch with no area.
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
    /// **So does a cut between two points of that curve in a piece cut off
    /// it.** A half keeps its rim corners on the one edge, and the face across
    /// still holds them all; only the flag that says so is gone, since a half
    /// is a stretch and a cut ([`WalkedLoop::between`]). So it is read off the
    /// face and not the piece: the face's one wire is one closed edge, and both
    /// ends are that edge's corner or points walked along it. A point strictly
    /// inside an earlier cut is this face's alone, so a cut to one stays bare.
    /// Q889's shape is where it showed. A block whose face plane misses a
    /// ball's `N` by half a rounding leaves a cap with `N` on its rim; its
    /// first cut ran from the rim up to `N` with a point on it, and the half it
    /// left, a little over a quarter turn wide, was cut again from a rim point
    /// straight to the middle of the pole side with nothing between. The disc
    /// fanned the same rim from `N`, drew the same chord, and ball less block
    /// was refused `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 2, used: 4 }`
    /// (Q964, `a_turned_block_whose_face_holds_a_balls_pole.rs`).
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
    ///
    /// **A cut round a torus's ring is one that has**, and every one of those
    /// carries a point. The crown of a ring is a piece of torus between two
    /// circles that one flat face also bounds, written with a seam across the
    /// tube: each cut round the ring runs from a point of one circle to a point
    /// of the other, a pair the flat face across both is free to draw. With
    /// bare cuts its build was refused `TwoPatchesOfTheMeshDrewTheSameSide {
    /// drawn: 4, used: 4 }`, and with a point on each it builds and holds what
    /// Pappus gives (`docs/decisions.md` §390). Every face cut that way was
    /// refused before such a cut existed, so the point moves no answer the
    /// kernel gave.
    ///
    /// **A cut that leaves the piece is no cut, and another chord is found.**
    /// [`where_to_cut`] pulls each end back to the nearer corner of the side
    /// the middle crosses, and near a pole one side of a coarse walk spans
    /// most of a half turn, so the ends can land far either side of the
    /// middle. Where the piece bends back past the middle, the chord between
    /// them runs out of it. A cube of side 2 at the origin turned 1 rad about
    /// `(−1, 3, 2)` and a unit ball at `(0.7, 0.65, 0.6)` in its frame leave a
    /// ball face shaped like a `C` in the chart, its two horns at longitudes
    /// `−1.37` and `−1.18` either side of the `y = 0` circle, which reaches in
    /// to `−0.13`. Walked at a deflection of `0.3`, the cut across the middle
    /// `0.176` was pulled back to the tips of the two horns, ran across that
    /// circle, and the patch was refused `BoundaryCrossesItself` at all three
    /// deflections (`a_cap_whose_rim_bends_back_past_its_own_meridian.rs`).
    /// So a chord that leaves the piece is replaced by the chord between two of
    /// its corners that stays inside and leaves the wider half narrowest
    /// ([`the_most_even_chord_that_stays_inside`]).
    ///
    /// **Where none does, the piece is left whole**, as one no cut makes any
    /// narrower is above. A ball of radius 4 at `(1, 1, 5)` resting on the
    /// block `[0, 4] × [0, 4] × [0, 1]` leaves a face, the ball's skin past
    /// `x = 0` between `y = 0` and `y = 4`, that goes `2.03` round in the
    /// chart: from the two corners where `x = 0` meets `y = 4`, at longitude
    /// `−4.391`, to the two where it meets `y = 0`, at `−2.356`. The only chord
    /// that parts the first two from the second runs between the points walked
    /// on the `x = 0` arc above and below the first, both at `−4.014`, and the
    /// lens `y = 4` cuts off the face reaches in between them to `−3.990`, so
    /// that chord crosses it. Kept, as it was until Q1099, the piece was
    /// refused `BoundaryCrossesItself` at the coarsest mesh and at the next,
    /// just under 4, and so were union both ways and ball less block of 60 such shapes:
    /// 180 of a sweep of 16 200 fuses of whole-number balls resting on
    /// whole-number blocks. Left whole, every one of the 16 200 answers and
    /// measures what its closed form says (`fuses_that_touch.rs`,
    /// `a_ball_whose_skin_the_walls_cut_into_a_piece_no_straight_cut_narrows_fuses_by_every_rule`).
    fn cut_in_two(
        &mut self,
        face: FaceView<'_>,
        stored: &Surface,
        chart: Chart,
        piece: &WalkedLoop,
        widest: f64,
        room: f64,
    ) -> Result<Option<(WalkedLoop, WalkedLoop)>> {
        let span = how_far_round(piece);
        if span <= widest + HOW_FAR_A_READ_LONGITUDE_CAN_BE_OFF {
            return Ok(None);
        }
        let (least, most) = longitudes_of(piece);
        let Some((opened, at)) = where_to_cut(piece, (least + most) / 2.0) else {
            return Ok(None);
        };
        let (opened, at) = if a_chord_stays_inside(&opened, at) {
            (opened, at)
        } else {
            let Some(at) = the_most_even_chord_that_stays_inside(piece, span) else {
                return Ok(None);
            };
            (piece.clone(), at)
        };
        let mut one = opened.between(at[0], at[1]);
        let mut other = opened.between(at[1], at[0]);
        if how_far_round(&one) >= span || how_far_round(&other) >= span {
            return Ok(None);
        }

        let leaving = opened.corners[at[1]];
        let arriving = opened.corners[at[0]];
        let ends = [opened.points[at[0]], opened.points[at[1]]];
        let on_the_one_curve_its_face_closes_on = |end: u32| {
            let mut wires = face.wires();
            let (Some(wire), None) = (wires.next(), wires.next()) else {
                return false;
            };
            wire.closes_on_one_edge()
                && wire.coedges().any(|coedge| {
                    self.at_vertex.get(&coedge.start().key()) == Some(&end)
                        || self
                            .along_edge
                            .get(&coedge.edge().key())
                            .is_some_and(|walked| walked.contains(&end))
                })
        };
        let least_steps = match chart {
            Chart::OnItsSide => 2,
            Chart::Upright if piece.closes_on_one_curve => 2,
            Chart::Upright
                if ends
                    .iter()
                    .all(|&end| on_the_one_curve_its_face_closes_on(end)) =>
            {
                2
            }
            Chart::Upright => 1,
        };
        let steps = how_many_steps(
            stored,
            chart.as_the_surface_takes_them(leaving),
            chart.as_the_surface_takes_them(arriving),
            room,
        )
        .max(least_steps);
        self.on_a_cut.extend(ends);
        let along_the_cut =
            self.points_along_a_cut_in(face, stored, chart, [leaving, arriving], steps)?;

        one.take_in(&along_the_cut);
        other.take_in(&along_the_cut.the_other_way());
        Ok(Some((one, other)))
    }

    /// Hand one patch to the triangulator and take what comes back into the
    /// solid's own table, asking for it to be cut by the order of its corners
    /// when `by_corner_order` says nothing across can share a side with that
    /// cut ([`BuildingAMesh::nothing_across_its_rim_joins_two_of_its_points`]).
    fn hand_over(
        &mut self,
        face: FaceView<'_>,
        surface: &Surface,
        patch: Vec<WalkedLoop>,
        room: f64,
        by_corner_order: bool,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let mut corner_is = Vec::new();
        let mut loops = Vec::with_capacity(patch.len());
        let mut along_a_loop = HashSet::new();
        let mut between_two_edges = Vec::new();
        let mut model_corners: HashSet<u32> = self.at_vertex.values().copied().collect();
        if let Some(corner) = self.the_corner_of_the_one_closed_edge_round(face) {
            model_corners.remove(&corner);
        }
        for one in patch {
            for (at, &from) in one.points.iter().enumerate() {
                along_a_loop.insert(a_side(from, one.points[(at + 1) % one.points.len()]));
                // A loop that is one closed edge passes its own corner and stays
                // on that edge, and so does a piece of a face bounded by that one
                // edge alone. Anywhere else a corner of the model, or a point
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

        let mut domain = TrimmedDomain::new(
            *surface,
            outer.clone(),
            holes.to_vec(),
            Deflection::of(room)?,
        )
        .with_corners_between_two_edges(between_two_edges);
        if by_corner_order {
            domain = domain.cut_by_corner_order();
        }
        let cut = match triangulate(&domain) {
            Ok(cut) => cut,
            Err(why) => {
                if std::env::var("RCAD_PROBE_Q1494").is_ok() {
                    eprintln!("PROBE hand_over refused {face:?} {why:?}");
                    eprintln!("PROBE   outer {:?}", outer);
                    for hole in holes {
                        eprintln!("PROBE   hole {:?}", hole);
                    }
                    let pts: Vec<_> = corner_is
                        .iter()
                        .map(|&at| (at, self.points[at as usize].coordinates()))
                        .collect();
                    eprintln!("PROBE   points {:?}", pts);
                }
                return Err(refusal_for(face, why, &corner_is, &self.points, log));
            }
        };

        // Everything past the corners this patch handed over is a point the
        // refinement put strictly inside it, so it belongs to this patch alone
        // and nothing else will ever name it (§25.1).
        //
        // A flat patch gets none: a flat piece of a plane strays nothing, so the
        // triangulator has nothing to refine. That is also what keeps
        // `self.made` in one frame per shell, since a flat face is cut up where
        // the solid sits and not where it is stored.
        assert!(
            surface.as_plane().is_none() || cut.corners().len() == corner_is.len(),
            "the triangulator put {} points inside flat face {:?}",
            cut.corners().len() - corner_is.len(),
            face.key()
        );
        let there = back_out_of_the_frame_this_face_is_cut_up_in(face);
        for corner in &cut.corners()[corner_is.len()..] {
            let inside = surface.point_at(corner[0], corner[1]);
            corner_is.push(self.remember_made_on(face, &there, inside));
        }

        // The triangulator draws every triangle anticlockwise in the chart, and
        // for every surface but a torus `∂first × ∂second` points out of it the
        // way its own normal does. A torus's chart takes the tube angle first
        // (for its seam, `Torus::point_at`), so there the chart's turn points
        // into the tube while the torus itself faces out of it, as the standard,
        // the reader and `validate.rs` read it (`docs/decisions.md` §329.1).
        let inside_out = face.orientation().is_reversed() != matches!(surface, Surface::Torus(_));
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

    /// The entry in the point table of the one corner of the one closed edge
    /// `face` is bounded by, when that edge is all it is bounded by.
    ///
    /// **Cut into pieces, such a face still has that edge on both sides of its
    /// corner**, so the corner is no place where the boundary passes from one
    /// neighbour to another, and the face across sees the same three points in
    /// a row there. Marked as such a place, it let a curved piece cut the corner
    /// off with a side the face across draws too. A unit ball less a cube whose
    /// face plane tilts 30 degrees and misses `N` by half a rounding leaves a
    /// cap with `N` on its rim, cut in two from the rim up to `N`; at tilt
    /// direction 250 the half holding the circle's corner cut it off as an ear
    /// with `N` and the rim point past it, the disc cut off the same ear, and
    /// measured at a deflection of `0.05` the answer was refused
    /// `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }` (Q1026,
    /// `a_turned_block_whose_face_holds_a_balls_pole.rs`). A corner a cut
    /// leaves from stays marked, since there the boundary does pass from the
    /// rim to the cut.
    ///
    /// **Only a face with one wire is asked**, the same reading
    /// [`BuildingAMesh::cut_in_two`] makes of a cut's two ends, because that is
    /// where the collision was shown. In a piece of a face with several wires,
    /// each closing on one edge, those corners are still marked.
    fn the_corner_of_the_one_closed_edge_round(&self, face: FaceView<'_>) -> Option<u32> {
        let mut wires = face.wires();
        let (Some(wire), None) = (wires.next(), wires.next()) else {
            return None;
        };
        if !wire.closes_on_one_edge() {
            return None;
        }
        let coedge = wire
            .coedges()
            .next()
            .expect("a wire that closes on one edge has that edge's coedge");
        Some(
            *self
                .at_vertex
                .get(&coedge.start().key())
                .expect("walking the wire put the corner its coedge starts at in the table"),
        )
    }

    /// One wire, as corners in the face's own two coordinates and the points of
    /// the solid's table they stand at. The coordinates are those of `surface`,
    /// which is [`the_surface_this_face_is_cut_up_on`], read off each point
    /// where it stands in [`the_frame_this_face_is_cut_up_in`].
    ///
    /// **A placed curved face reads each point where it was made**, its corners
    /// out of the arena and the rest out of [`BuildingAMesh::made`], and not
    /// moved out and back: that round trip rounds twice, and out where a solid
    /// is one ulp across it put the middle of a ball's seam on the axis between
    /// its poles (`docs/decisions.md` §333.6, §379). Reading a point where it was
    /// made can hand back a mesh that is sound where it was made and flat where
    /// it lands, which is [`SolidView::unless_placing_it_took_away_its_space`]'s
    /// to refuse. A flat face is cut up where the solid sits and a face at no
    /// placement is moved by an exact shift, so both read the table as it is.
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
    /// among them by [`put_in_the_poles_a_crossing_passes`] (§193.3), along
    /// an open edge as well as a closed one since Q1270.
    fn walk_a_wire_round(
        &mut self,
        wire: WireView<'_>,
        face: FaceView<'_>,
        surface: &Surface,
    ) -> Option<WalkedLoop> {
        let reads = HowAFaceReadsThePointsRoundIt::of(face);
        let corner_in = |vertex: VertexView<'_>| reads.a_corner(vertex, face);
        let poles = poles_of(wire);
        let sits_at_a_pole = |at: Point| {
            the_surface_pinches_at(surface, surface.coordinates_of(at).1)
                || the_pole_a_point_sits_within_a_rounding_of(surface, at).is_some()
        };
        let at_a_pole = |vertex: &VertexView<'_>| {
            poles.contains(&vertex.key()) || sits_at_a_pole(corner_in(*vertex))
        };
        let mut used = HashSet::new();
        let no_side_of_this_wire_is_a_seam = wire
            .coedges()
            .all(|coedge| used.insert(coedge.edge().key()));
        let mut walk = WalkingAWire::new(
            how_far_the_longitude_wraps(surface),
            how_far_the_second_coordinate_wraps(surface),
            wire.closes_on_one_edge(),
            no_side_of_this_wire_is_a_seam,
        );

        let steps = steps_from_somewhere_that_has_a_longitude(wire, &at_a_pole);
        for (index, &step) in steps.iter().enumerate() {
            match step {
                WireStepView::Along(coedge) => {
                    let start = coedge.start();
                    let at = self.remember_the_corner(start);
                    let where_it_is = corner_in(start);
                    let start_is_a_pole = at_a_pole(&start);
                    walk.arrive(at, surface.coordinates_of(where_it_is), start_is_a_pole);
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
                        let where_it_is = reads.a_point(at, self);
                        let sits_at = surface.coordinates_of(where_it_is);
                        let passes_a_pole = sits_at_a_pole(where_it_is);
                        walk.arrive(at, sits_at, passes_a_pole);
                        if passes_a_pole {
                            walk.turn_at_a_corner_on_a_pole();
                        }
                    }
                }
                WireStepView::Pole(vertex) => {
                    let at = self.remember_the_corner(vertex);
                    let where_it_is = corner_in(vertex);
                    walk.arrive(at, surface.coordinates_of(where_it_is), true);
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
        let at = self.remember(vertex.point(), vertex.stored_point());
        self.at_vertex.insert(vertex.key(), at);
        at
    }

    /// Put a point `face` made in the table, moved out to where the solid sits by
    /// `there`, and say where it went.
    ///
    /// Where it was made is kept as [`BuildingAMesh::made`] says: as made for a
    /// placed face, and as moved out for a face at no placement, whose `there`
    /// is an exact shift and whose stored frame is where the solid sits.
    fn remember_made_on(&mut self, face: FaceView<'_>, there: &Isometry, made: Point) -> u32 {
        let at = there.apply_to_point(made);
        if face.location() == Isometry::identity() {
            return self.remember(at, at);
        }
        self.remember(at, made)
    }

    /// Put a point in the table, with where it was made beside it, and say
    /// where it went.
    fn remember(&mut self, point: Point, made: Point) -> u32 {
        let at = u32::try_from(self.points.len())
            .expect("a mesh is refused long before it holds four billion points");
        self.points.push(point);
        self.made.push(made);
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

/// Where a face reads the points round it, in the frame it is cut up in: a
/// placed curved face reads each where it was made, and any other face reads
/// the point table moved into that frame — see
/// [`BuildingAMesh::walk_a_wire_round`] for why.
///
/// One reader for the face's own walk and for
/// [`BuildingAMesh::nothing_across_its_rim_joins_two_of_its_points`], which has
/// to read the points the face across will read, bit for bit.
#[derive(Clone, Copy)]
struct HowAFaceReadsThePointsRoundIt {
    back: Isometry,
    as_made: bool,
}

impl HowAFaceReadsThePointsRoundIt {
    fn of(face: FaceView<'_>) -> Self {
        let back = the_frame_this_face_is_cut_up_in(face);
        Self {
            back,
            as_made: face.location() != Isometry::identity() && back != Isometry::identity(),
        }
    }

    /// A corner of the model, looked at through `face`'s own wires.
    fn a_corner(self, vertex: VertexView<'_>, face: FaceView<'_>) -> Point {
        if self.as_made {
            where_this_corner_is_in_the_frame_it_is_meshed_in(vertex, face)
        } else {
            self.back.apply_to_point(vertex.point())
        }
    }

    /// The point at `at` in the table `building` holds.
    fn a_point(self, at: u32, building: &BuildingAMesh) -> Point {
        if self.as_made {
            building.made[at as usize]
        } else {
            self.back.apply_to_point(building.points[at as usize])
        }
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
    /// The same two for the second coordinate, which only wraps on a torus,
    /// where it is the angle round the ring: whether the walk went right round
    /// the ring, and how far round it the walk climbed.
    goes_right_round_the_second_way: bool,
    climbed_the_second_way: f64,
}

impl WalkedLoop {
    /// A stretch or a cut, with room for `corners` corners and none yet: it
    /// is no whole loop, so it goes right round nothing.
    fn a_stretch(corners: usize) -> WalkedLoop {
        WalkedLoop {
            corners: Vec::with_capacity(corners),
            points: Vec::with_capacity(corners),
            closes_on_one_curve: false,
            goes_right_round: false,
            climbed: 0.0,
            goes_right_round_the_second_way: false,
            climbed_the_second_way: 0.0,
        }
    }

    /// The same loop with the two numbers of every corner swapped, and with
    /// them which way round it goes each way: the corners in
    /// [`Chart::OnItsSide`] read from [`Chart::Upright`], and back again.
    fn turned_on_its_side(&self) -> WalkedLoop {
        WalkedLoop {
            corners: self
                .corners
                .iter()
                .map(|&[first, second]| [second, first])
                .collect(),
            points: self.points.clone(),
            closes_on_one_curve: self.closes_on_one_curve,
            goes_right_round: self.goes_right_round_the_second_way,
            climbed: self.climbed_the_second_way,
            goes_right_round_the_second_way: self.goes_right_round,
            climbed_the_second_way: self.climbed,
        }
    }

    /// The stretch from corner `from` round to corner `to`, both ends kept.
    fn between(&self, from: usize, to: usize) -> WalkedLoop {
        let mut taken = WalkedLoop::a_stretch(0);
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
            goes_right_round_the_second_way: self.goes_right_round_the_second_way,
            climbed_the_second_way: -self.climbed_the_second_way,
        }
    }
}

/// One cut from a cap's rim up to its pole, as the patch ahead of it walks it
/// up and the patch behind walks it down: the points strictly between the rim
/// corner and the pole. A straight cut is the same points both ways; one
/// through an island goes round the island's two halves, one each way
/// ([`BuildingAMesh::a_cut_through_an_island`]).
struct ACutUpToThePole {
    up: WalkedLoop,
    down: WalkedLoop,
}

impl ACutUpToThePole {
    /// A cut that reaches `island` at its corner `bottom` and leaves it up
    /// from its corner `top`: up takes the island's side from `bottom` round
    /// to `top` the way the island is walked, and down the side from `top`
    /// round to `bottom`.
    fn round_an_island(
        island: &WalkedLoop,
        [bottom, top]: [usize; 2],
        to_the_island: &WalkedLoop,
        to_the_pole: &WalkedLoop,
    ) -> Self {
        let mut up = to_the_island.clone();
        up.take_in(&island.between(bottom, top));
        up.take_in(to_the_pole);
        let mut down = to_the_pole.the_other_way();
        down.take_in(&island.between(top, bottom));
        down.take_in(&to_the_island.the_other_way());
        Self { up, down }
    }
}

/// One of a ring's four cuts, as the patch behind it walks it across from the
/// loop running up to the loop running down, and as the patch ahead of it
/// walks it back: the points strictly between its two ends. The patch behind
/// is the one at lower longitude. A straight cut is the same points both
/// ways; one round an island goes round the island's two halves, one each way.
struct ACutAcrossARing {
    across: WalkedLoop,
    back: WalkedLoop,
}

impl ACutAcrossARing {
    /// A cut that reaches its island at one corner and leaves it from another,
    /// the island's half on each side going to the patch on that side.
    ///
    /// **An island is walked clockwise in the chart**, since the material is
    /// outside it, so from the corner the cut reaches round to the one it
    /// leaves from, the way the island is walked, is the half on the left of
    /// the way the cut runs. The patch behind, at lower longitude, is on the
    /// cut's left where it runs up the chart and on its right where it runs
    /// down.
    fn round_an_island(
        round: &ARingsCutRoundAnIsland,
        runs_up_the_chart: bool,
        to_the_island: &WalkedLoop,
        from_the_island: &WalkedLoop,
    ) -> Self {
        let [reached, left] = round.corners;
        let on_its_left = round.island.between(reached, left);
        let on_its_right = round.island.between(left, reached).the_other_way();
        let (behind, ahead) = if runs_up_the_chart {
            (on_its_left, on_its_right)
        } else {
            (on_its_right, on_its_left)
        };
        let round_one_half = |half: &WalkedLoop| {
            let mut cut = to_the_island.clone();
            cut.take_in(half);
            cut.take_in(from_the_island);
            cut
        };
        Self {
            across: round_one_half(&behind),
            back: round_one_half(&ahead).the_other_way(),
        }
    }
}

/// Where one of a ring's cuts goes round an island: which of the ring's
/// islands it is, the island moved by whole turns of longitude so that its
/// middle is within half a turn of the corner the cut leaves from, and the
/// island's corner the cut reaches it at and the one it leaves it from.
struct ARingsCutRoundAnIsland {
    which: usize,
    island: WalkedLoop,
    corners: [usize; 2],
}

/// What [`BuildingAMesh::a_cut_through_an_island`] needs to find a cut that
/// stays inside the face: the cap, the rim corner the cut leaves from, the
/// cap's islands and which of them the cut goes through, the corners the other
/// three cuts leave from, the pole and how far round a whole turn is.
struct ACutThroughAnIsland<'a> {
    cap: &'a WalkedLoop,
    leaving: [f64; 2],
    islands: &'a [WalkedLoop],
    which: usize,
    others: &'a [[f64; 2]],
    towards: f64,
    whole: f64,
}

impl ACutThroughAnIsland<'_> {
    /// The island the cut goes through, moved by whole turns of longitude so
    /// that its middle is within half a turn of the rim corner the cut leaves
    /// from.
    fn the_island_in_its_turn(&self) -> WalkedLoop {
        in_the_turn_nearest(&self.islands[self.which], self.leaving[0], self.whole)
    }

    /// Whether the meridian up from the island's corner `top` to the pole
    /// meets no other island, no side of its own island but the two at `top`,
    /// and not the rim.
    fn leaves_up_from(&self, island: &WalkedLoop, top: usize) -> bool {
        let up_from = island.corners[top];
        let others_in_the_way = self.other_islands().any(|other| {
            its_meridian_meets_an_island(
                up_from,
                self.towards,
                core::slice::from_ref(other),
                self.whole,
            )
        });
        let its_own_in_the_way = (0..island.corners.len()).any(|at| {
            let next = (at + 1) % island.corners.len();
            at != top
                && next != top
                && a_side_meets_a_cut(
                    [island.corners[at], island.corners[next]],
                    up_from[0],
                    [up_from[1], self.towards],
                )
        });
        !(others_in_the_way
            || its_own_in_the_way
            || a_cut_up_meets_the_rim(self.cap, up_from, self.towards, self.whole))
    }

    /// Whether a straight leg from the rim corner to the island's corner
    /// `bottom` meets no side of the rim, no side of its island but the two at
    /// `bottom`, no other island, and the meridian of no other cut.
    fn reaches(&self, island: &WalkedLoop, bottom: usize) -> bool {
        let leg = [self.leaving, island.corners[bottom]];
        !a_leg_meets_the_rim(self.cap, leg, self.whole)
            && !a_leg_meets_an_island(island, leg, Some(bottom), self.whole)
            && self
                .other_islands()
                .all(|other| !a_leg_meets_an_island(other, leg, None, self.whole))
            && self
                .others
                .iter()
                .all(|&[longitude, from]| !a_side_meets_a_cut(leg, longitude, [from, self.towards]))
    }

    /// The island's corners `corners` names, nearest the rim corner first.
    fn nearest_first(&self, island: &WalkedLoop, corners: &mut [usize]) {
        corners.sort_by(|&one, &other| {
            let away = |at: usize| {
                let [x, y] = island.corners[at];
                (x - self.leaving[0]).hypot(y - self.leaving[1])
            };
            away(one).total_cmp(&away(other))
        });
    }

    fn other_islands(&self) -> impl Iterator<Item = &WalkedLoop> {
        self.islands
            .iter()
            .enumerate()
            .filter(move |&(at, _)| at != self.which)
            .map(|(_, other)| other)
    }
}

/// What [`BuildingAMesh::cuts_through_one_island`] needs: the cap, the rim
/// corners the four cuts leave from, which of the four meet the island, the
/// cap's islands and which one it is, the way the rim runs, the pole and how
/// far round a whole turn is.
struct CutsThroughOneIsland<'a> {
    cap: &'a WalkedLoop,
    along: [usize; 4],
    meeting: &'a [usize],
    islands: &'a [WalkedLoop],
    which: usize,
    way: f64,
    towards: f64,
    whole: f64,
}

/// One of the cuts [`BuildingAMesh::cuts_through_one_island`] makes: its place
/// among the four, the cut, and the strip between it and the next cut, when
/// that one goes through the same island.
struct OneOfTheCutsThroughAnIsland {
    place: usize,
    cut: ACutUpToThePole,
    strip: Option<AStripAnIslandSplits>,
}

/// The strip between two cuts through one island, which the island splits in
/// two patches. `under` runs on from the rim corner of the cut ahead: up its
/// leg to the island, back along the island's underside and down the leg of the
/// cut behind. `over` starts at the top of the cut behind and runs along the
/// island's top to the top of the cut ahead and up its meridian, and
/// `down_from_the_pole` comes back down the meridian of the cut behind.
struct AStripAnIslandSplits {
    under: WalkedLoop,
    over: WalkedLoop,
    down_from_the_pole: WalkedLoop,
}

/// One strip of a ball from pole to pole and the holes inside it, moved into
/// the strip's own turn of longitude
/// ([`BuildingAMesh::open_the_ball_into_strips`]).
struct AStripAndItsHoles {
    strip: WalkedLoop,
    holes: Vec<WalkedLoop>,
}

/// A hole cut straight up from its highest corner and straight down from its
/// lowest ([`BuildingAMesh::cut_holes_side_by_side_up_to_both_poles`]): which
/// side of the outer loop each cut reaches, and the latitude it reaches it at.
struct AHoleCutStraightUpAndDown<'a> {
    hole: &'a WalkedLoop,
    highest: usize,
    lowest: usize,
    up: (usize, f64),
    down: (usize, f64),
}

impl AHoleCutStraightUpAndDown<'_> {
    fn longitude_up(&self) -> f64 {
        self.hole.corners[self.highest][0]
    }

    fn longitude_down(&self) -> f64 {
        self.hole.corners[self.lowest][0]
    }

    /// The hole from its lowest corner up to its highest: the half on the
    /// left, since a hole runs clockwise.
    fn left_half(&self) -> WalkedLoop {
        self.hole.between(self.lowest, self.highest)
    }

    fn right_half(&self) -> WalkedLoop {
        self.hole.between(self.highest, self.lowest)
    }
}

/// Which order a patch's corners give a surface's two coordinates in.
///
/// Everything that cuts a face into pieces cuts across the **first** number of
/// the corners it is handed: [`BuildingAMesh::cut_in_two`] at the middle of
/// how far round a piece goes, [`BuildingAMesh::cut_a_ring_across`] at four
/// quarters of a turn. On every surface but a torus only the first of the
/// surface's own two coordinates wraps, so that is the only way there is to
/// cut. A torus wraps both ways, round the tube and round the ring, and a face
/// can go right round either. Turning its corners on their side lets the same
/// cuts run round the ring, and the two numbers are given back in the
/// surface's own order only where a point is put on the surface.
#[derive(Clone, Copy, Debug)]
enum Chart {
    /// The surface's first coordinate first: on a torus, the angle round the
    /// tube.
    Upright,
    /// The surface's second coordinate first: on a torus, the angle round the
    /// ring.
    OnItsSide,
}

impl Chart {
    /// A corner's two numbers in the order the surface's own
    /// [`Surface::point_at`] and [`how_many_steps`] take them.
    fn as_the_surface_takes_them(self, [one, other]: [f64; 2]) -> [f64; 2] {
        match self {
            Chart::Upright => [one, other],
            Chart::OnItsSide => [other, one],
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

/// Whether a face's loops are the rim of a cap on a ball with islands in it:
/// one loop right round, and at least one more, each passing each of its
/// points once with the material outside it.
///
/// **Only on a ball**, where this was reached and measured. A cone's ring
/// round its apex beside an island is the same shape and is still refused,
/// for want of a solid that reaches it.
fn a_cap_on_a_ball_with_islands_in_it(stored: &Surface, walked: &[WalkedLoop]) -> bool {
    let (rims, islands): (Vec<&WalkedLoop>, Vec<&WalkedLoop>) =
        walked.iter().partition(|around| around.goes_right_round);
    rims.len() == 1
        && !islands.is_empty()
        && stored.where_it_pinches() == Some(core::f64::consts::FRAC_PI_2)
        && how_far_the_second_coordinate_wraps(stored).is_none()
        && islands.iter().all(|island| {
            it_passes_each_of_its_points_once(island)
                && how_much_of_the_chart_this_loop_encloses(island) < 0.0
        })
}

/// Whether a face's loops are a ring on a pipe with islands in it: two loops
/// right round, and at least one more, each passing each of its points once
/// with the material outside it.
///
/// **Only on a pipe**, a surface whose longitude wraps and which pinches
/// nowhere, where this was reached and measured. A band on a ball or a cone
/// beside an island is the same shape and is still refused, for want of a
/// solid that reaches it, and so is one on a torus.
fn a_ring_on_a_pipe_with_islands_in_it(stored: &Surface, walked: &[WalkedLoop]) -> bool {
    let (rims, islands): (Vec<&WalkedLoop>, Vec<&WalkedLoop>) =
        walked.iter().partition(|around| around.goes_right_round);
    rims.len() == 2
        && !islands.is_empty()
        && how_far_the_longitude_wraps(stored).is_some()
        && stored.where_it_pinches().is_none()
        && how_far_the_second_coordinate_wraps(stored).is_none()
        && islands.iter().all(|island| {
            it_passes_each_of_its_points_once(island)
                && how_much_of_the_chart_this_loop_encloses(island) < 0.0
        })
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

/// A ring's two loops on a surface that pinches at `pinch`, each with a corner
/// put in wherever one of its pole sides passes one of `quarters` at any whole
/// turn, so that no side the ring's patches are built from runs as far as half
/// a turn along a pole. With no `pinch` there are no pole sides and the loops
/// come back as they were.
///
/// **A ring's patches are put together a side at a time, each side the short
/// way round** ([`WalkedLoop::take_in_unwrapped`], [`unwrapped`]). That is
/// right for every side but a pole side longer than half a turn, where the
/// short way is the other way round the pole, and the patch's pole side then
/// folds back across the patch. A half ball holed along a circle through its
/// north pole has such a side: a smooth curve through a pole leaves it half a
/// turn round from where it arrived, so the hole's side along the pole, round
/// the skin, is half a turn and a little. Every spelling of it was refused
/// `BoundaryCrossesItself`
/// (`a_half_ball_holed_through_its_pole_and_capped_on_the_same_ball.rs`).
///
/// A corner at a quarter is where the cut from the other loop's quarter lands,
/// a meridian up to the pole, as the cuts of a cap through the other pole do
/// ([`with_a_corner_where_a_pole_side_passes`]).
fn with_a_corner_where_a_pole_side_of_either_passes(
    up: WalkedLoop,
    down: WalkedLoop,
    quarters: [f64; 4],
    pinch: Option<f64>,
    whole: f64,
) -> (WalkedLoop, WalkedLoop) {
    let Some(pinch) = pinch else {
        return (up, down);
    };
    let in_every_turn: Vec<f64> = [-whole, 0.0, whole]
        .into_iter()
        .flat_map(|by| quarters.map(|quarter| quarter + by))
        .collect();
    (
        with_a_corner_where_a_pole_side_passes(&up, &in_every_turn, pinch),
        with_a_corner_where_a_pole_side_passes(&down, &in_every_turn, pinch),
    )
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

/// One of a cap's patches with its side along the pole put on: the pole twice,
/// at the two longitudes the meridians arrive at. It is one point in space and
/// a whole side of the patch in the surface's coordinates, which is what a
/// `WireStep::Pole` says about a ball this kernel builds.
///
/// **The side ends at the meridian the cut down leaves by, which is the
/// patch's own first corner, and not a quarter turn back.** The cuts leave from
/// the rim corners nearest the quarters, and on a coarse rim those are not a
/// quarter apart: the dome of §191's turned ball and block has four of them
/// `2.74`, `0.40`, `2.74` and `0.40` radians apart, so a quarter turn back ran
/// `1.17` past the next cut's meridian in two patches, the side folded back on
/// itself, and the mesher refused it as a boundary that crosses itself.
/// `a_turned_ball_united_with_a_block_holds_what_the_upright_one_does`.
fn with_the_pole_side(patch: &mut WalkedLoop, towards: f64, at_the_pole: u32) {
    let corner = patch.corners.last().map_or(0.0, |corner| corner[0]);
    patch.corners.push([corner, towards]);
    patch.points.push(at_the_pole);
    let back = patch.corners[0][0];
    patch.corners.push([back, towards]);
    patch.points.push(at_the_pole);
}

/// The corners of a cap the four quarter cuts leave from: each the corner
/// nearest its quarter, and **four different ones, in the order the loop runs**.
///
/// The four patches are the stretches of the loop between one cut and the
/// next, so two cuts from one corner leave a patch of no width, and cuts out of
/// the loop's order leave stretches that overlap. On a coarse walk the nearest
/// corners can do the first. A block that keeps under a third of a turn of a
/// pointed cone's rim leaves a cap walked as both ends of that rim, its middle,
/// and the middle of the crossing round the far side of the apex; two quarters
/// fall on the crossing, both are nearest its one corner, and the patch between
/// their cuts was that corner and the apex twice, which the triangulator
/// refused as `CornersCoincide` (§285.2).
///
/// So when the nearest corners do not run in order, each quarter after the
/// first is given the nearest of the corners after the one before it, leaving
/// one for each quarter still to come. When they do, that is the same choice,
/// and it is kept as it was. A cap of fewer than four corners has no four to
/// give, and goes on to the triangulator as it was.
///
/// **A cut leaves only from a corner whose meridian reaches the pole without
/// meeting the rim again**, when the cap has four such corners. A rim can bend
/// back past a meridian it has already crossed, and a cut from where it first
/// crosses runs out of the face and back in. A cube of side 2 at the origin
/// and a unit ball at `(0.7, 0.65, 0.6)` intersect in a face whose rim leaves
/// the cube's `y` edge at longitude `2.636` going back to `2.361`, round the
/// `x = 0` circle: the cut up `2.636` crossed that circle at `x = -0.175`,
/// outside the cube, and the triangulator refused the patch
/// `BoundaryCrossesItself`. With the ball at `(0.75, 0.6, 0.7)` both points
/// where the cube's `z` edge pierces the ball are on the rim, at one
/// longitude, and the cut from the lower ran through the upper; the mesh was
/// refused `TwoPatchesOfTheMeshDrewTheSameSide`
/// (`a_cap_whose_rim_bends_back_past_its_own_meridian.rs`).
///
/// **Why four cuts from such corners, in the loop's order, leave four patches
/// that are each one simple loop.** Each cut lies inside the
/// face, so two of them split it in two, and the piece left of the rim between
/// them is a disk that touches neither pole but at the one its cuts meet at. It
/// therefore unwraps whole into the chart, where its pole side is its top edge
/// and runs back the way the rim came, so the rim gains between its two cuts
/// more than nothing and less than a turn.
///
/// The cut has to start at a corner the rim already has, for the reason
/// [`where_to_cut`] gives. So the cut from where a quarter's own meridian last
/// crosses the rim, which is the cut that stays in the face, is taken at the
/// nearest corner that is a last crossing of its own meridian.
///
/// **A cut leaves from a corner whose meridian misses every island**, too,
/// when there are four such corners, so that each island goes whole into one
/// patch. A coarse rim may not have four: §400.5's ball walks its rim at four
/// corners, one of them under the island, and that cut is taken through the
/// island instead ([`BuildingAMesh::a_cut_through_an_island`]).
fn corners_the_quarter_cuts_leave_from(
    cap: &WalkedLoop,
    quarters: [f64; 4],
    pinch: f64,
    towards: f64,
    whole: f64,
    islands: &[WalkedLoop],
) -> [usize; 4] {
    let clear: Vec<usize> = (0..cap.corners.len())
        .filter(|&corner| !a_cut_up_meets_the_rim(cap, cap.corners[corner], towards, whole))
        .collect();
    let clear_of_the_islands: Vec<usize> = clear
        .iter()
        .copied()
        .filter(|&corner| {
            !its_meridian_meets_an_island(cap.corners[corner], towards, islands, whole)
        })
        .collect();
    let among = if clear_of_the_islands.len() >= 4 {
        clear_of_the_islands
    } else if clear.len() >= 4 {
        clear
    } else {
        (0..cap.corners.len()).collect()
    };
    let count = among.len();
    let nearest_of = |room: &mut dyn Iterator<Item = usize>, at: f64| {
        nearest_corner_off_the_rims_poles(cap, room, &among, at, pinch)
    };
    let nearest = quarters.map(|at| nearest_of(&mut (0..count), at));
    let first = nearest[0];
    let past_the_first = |place: usize| (place + count - first) % count;
    let in_loop_order = nearest
        .windows(2)
        .all(|pair| past_the_first(pair[0]) < past_the_first(pair[1]));
    if in_loop_order || count < 4 {
        return nearest.map(|place| among[place]);
    }
    let mut along = nearest;
    let mut taken = 0;
    for step in 1..4 {
        let mut room = (taken + 1..=count - (4 - step)).map(|offset| (first + offset) % count);
        along[step] = nearest_of(&mut room, quarters[step]);
        taken = past_the_first(along[step]);
    }
    along.map(|place| among[place])
}

/// Whether the cut up the meridian of `corner` to the pole at `towards` meets
/// an island of the cap, at any whole turn of longitude, touching included.
fn its_meridian_meets_an_island(
    [longitude, latitude]: [f64; 2],
    towards: f64,
    islands: &[WalkedLoop],
    whole: f64,
) -> bool {
    islands.iter().any(|island| {
        let (least, most) = longitudes_of(island);
        let first_turn = ((least - longitude) / whole).ceil() as i64;
        let last_turn = ((most - longitude) / whole).floor() as i64;
        (first_turn..=last_turn)
            .any(|turn| a_line_meets(island, longitude + turn as f64 * whole, [latitude, towards]))
    })
}

/// Where the first of a cap's four quarters is put, in the cap's own turn of
/// longitude: at its first corner when it has no islands, as it always was,
/// and otherwise half a quarter on from the middle of the widest stretch of
/// longitude no island reaches, so that the quarters' cuts fall either side of
/// the islands rather than through them. `None` when the islands reach every
/// longitude, where no cut up to the pole misses them all.
fn where_the_quarters_start(
    cap: &WalkedLoop,
    islands: &[WalkedLoop],
    way: f64,
    whole: f64,
) -> Option<f64> {
    let first = cap.corners[0][0];
    if islands.is_empty() {
        return Some(first);
    }
    let spans: Vec<(f64, f64)> = islands.iter().map(longitudes_of).collect();
    let quarter = whole / 4.0;
    let middle = the_middle_of_the_widest_stretch_no_hole_reaches(&spans, whole)?;
    let on = (way * (middle + quarter / 2.0 - first)).rem_euclid(quarter);
    Some(first + way * on)
}

/// Which of a cap's patches an island lies in, and the island moved by whole
/// turns of longitude into that patch's own turn: the patch whose loop holds
/// the island's first corner. The island crosses no cut and no rim, so a patch
/// holding one corner of it holds all of it. `None` when no patch holds it.
fn the_patch_an_island_lies_in(
    patches: &[Vec<WalkedLoop>],
    island: &WalkedLoop,
    whole: f64,
) -> Option<(usize, WalkedLoop)> {
    let [longitude, latitude] = island.corners[0];
    patches.iter().enumerate().find_map(|(at, patch)| {
        let (least, most) = longitudes_of(&patch[0]);
        let first_turn = ((least - longitude) / whole).ceil() as i64;
        let last_turn = ((most - longitude) / whole).floor() as i64;
        (first_turn..=last_turn).find_map(|turn| {
            let by = turn as f64 * whole;
            a_loop_holds(&patch[0], [longitude + by, latitude]).then(|| {
                let moved = WalkedLoop {
                    corners: island
                        .corners
                        .iter()
                        .map(|corner| [corner[0] + by, corner[1]])
                        .collect(),
                    ..island.clone()
                };
                (at, moved)
            })
        })
    })
}

/// Whether a point lies strictly inside a loop in the chart, by how many of
/// the loop's sides a line from it straight up the latitude crosses.
fn a_loop_holds(around: &WalkedLoop, [longitude, latitude]: [f64; 2]) -> bool {
    let count = around.corners.len();
    let mut inside = false;
    for at in 0..count {
        let (here, next) = (around.corners[at], around.corners[(at + 1) % count]);
        if (here[0] > longitude) == (next[0] > longitude) {
            continue;
        }
        let crossing = here[1] + (longitude - here[0]) / (next[0] - here[0]) * (next[1] - here[1]);
        if crossing > latitude {
            inside = !inside;
        }
    }
    inside
}

/// Whether the cut up the meridian from `[longitude, from]` to the pole at
/// `towards` meets the cap's rim anywhere but at its own start, in the chart,
/// where the cut is a straight line.
///
/// **Touching counts as meeting.** A rim corner on the cut is a point both
/// patches either side of it would draw from, which is the second example in
/// [`corners_the_quarter_cuts_leave_from`]. A rim side running along the cut
/// meets it too, which is what keeps a cut off the step a rim through the
/// other pole leaves it by ([`starting_off_the_poles`]).
///
/// The rim's longitudes are the walk's own, unwrapped, so they run a whole turn
/// and more where the rim bends back; the cut is checked a whole turn either
/// side as well.
fn a_cut_up_meets_the_rim(
    cap: &WalkedLoop,
    [longitude, from]: [f64; 2],
    towards: f64,
    whole: f64,
) -> bool {
    the_sides_of_a_rim(cap, whole).any(|side| a_side_meets_a_cut(side, longitude, [from, towards]))
}

/// Every side of a cap's rim, a whole turn either side as well, with the side
/// that closes it run on to its first corner a whole `climbed` on.
fn the_sides_of_a_rim(cap: &WalkedLoop, whole: f64) -> impl Iterator<Item = [[f64; 2]; 2]> + '_ {
    let count = cap.corners.len();
    (0..count).flat_map(move |at| {
        let here = cap.corners[at];
        let mut there = cap.corners[(at + 1) % count];
        if at + 1 == count {
            there[0] += cap.climbed;
        }
        [-whole, 0.0, whole].map(|by| [[here[0] + by, here[1]], [there[0] + by, there[1]]])
    })
}

/// Which island each of a ring's cuts goes round, from the corner it leaves
/// at each end of `ends`: `None` for a cut whose straight line meets none.
/// It reaches the island at the island's corner furthest towards the loop it
/// leaves, and leaves it from the corner furthest towards the loop it arrives
/// on. `None` overall when a cut meets two islands, two cuts meet one, or a
/// cut that meets one runs level across the chart, where neither loop is the
/// nearer to any corner.
fn where_a_rings_cuts_go_round_islands(
    ends: &[[[f64; 2]; 2]],
    islands: &[WalkedLoop],
    whole: f64,
) -> Option<Vec<Option<ARingsCutRoundAnIsland>>> {
    let mut met = vec![false; islands.len()];
    let mut through = Vec::with_capacity(ends.len());
    for &[leaving, arriving] in ends {
        let meets: Vec<usize> = (0..islands.len())
            .filter(|&at| a_leg_meets_an_island(&islands[at], [leaving, arriving], None, whole))
            .collect();
        let which = match meets[..] {
            [] => {
                through.push(None);
                continue;
            }
            [which] if !met[which] && leaving[1] != arriving[1] => which,
            _ => return None,
        };
        met[which] = true;
        let island = in_the_turn_nearest(&islands[which], leaving[0], whole);
        let towards_leaving = (leaving[1] - arriving[1]).signum();
        let corners = [
            the_corner_furthest(&island, towards_leaving),
            the_corner_furthest(&island, -towards_leaving),
        ];
        through.push(Some(ARingsCutRoundAnIsland {
            which,
            island,
            corners,
        }));
    }
    Some(through)
}

/// Whether each leg of a ring's cuts round islands stays inside the face: it
/// meets neither loop but at the corner it leaves, no side of its own island
/// but the two at the corner it ends at, no other island, and no other cut,
/// a whole turn either side as well.
fn the_legs_round_islands_stay_inside_a_ring(
    rims: [&WalkedLoop; 2],
    islands: &[WalkedLoop],
    ends: &[[[f64; 2]; 2]],
    through: &[Option<ARingsCutRoundAnIsland>],
    whole: f64,
) -> bool {
    let sides_of = |cut: usize| -> Vec<[[f64; 2]; 2]> {
        let [leaving, arriving] = ends[cut];
        match &through[cut] {
            None => vec![[leaving, arriving]],
            Some(round) => {
                let [reached, left] = round.corners.map(|corner| round.island.corners[corner]);
                vec![[leaving, reached], [arriving, left]]
            }
        }
    };
    through.iter().enumerate().all(|(cut, round)| {
        let Some(round) = round else {
            return true;
        };
        let [leaving, arriving] = ends[cut];
        let [reached, left] = round.corners;
        [(leaving, reached), (arriving, left)]
            .into_iter()
            .all(|(from, corner)| {
                let leg = [from, round.island.corners[corner]];
                rims.iter().all(|rim| !a_leg_meets_the_rim(rim, leg, whole))
                    && !a_leg_meets_an_island(&round.island, leg, Some(corner), whole)
                    && islands
                        .iter()
                        .enumerate()
                        .filter(|&(at, _)| at != round.which)
                        .all(|(_, other)| !a_leg_meets_an_island(other, leg, None, whole))
                    && (0..ends.len())
                        .filter(|&other| other != cut)
                        .flat_map(sides_of)
                        .all(|side| {
                            [-whole, 0.0, whole]
                                .iter()
                                .all(|&by| !two_sides_meet(leg, side.map(|[x, y]| [x + by, y])))
                        })
            })
    })
}

/// Whether a straight leg from a rim corner meets a side of the rim other
/// than at that corner.
fn a_leg_meets_the_rim(cap: &WalkedLoop, leg: [[f64; 2]; 2], whole: f64) -> bool {
    the_sides_of_a_rim(cap, whole)
        .filter(|side| side[0] != leg[0] && side[1] != leg[0])
        .any(|side| two_sides_meet(leg, side))
}

/// Whether a straight leg meets a side of an island, a whole turn either side
/// as well, leaving out the two sides at the island's corner `ending_at`, the
/// corner the leg ends at.
fn a_leg_meets_an_island(
    island: &WalkedLoop,
    leg: [[f64; 2]; 2],
    ending_at: Option<usize>,
    whole: f64,
) -> bool {
    let count = island.corners.len();
    (0..count).any(|at| {
        let next = (at + 1) % count;
        if ending_at.is_some_and(|corner| corner == at || corner == next) {
            return false;
        }
        [-whole, 0.0, whole].iter().any(|&by| {
            let side = [at, next].map(|corner| {
                let [longitude, latitude] = island.corners[corner];
                [longitude + by, latitude]
            });
            two_sides_meet(leg, side)
        })
    })
}

/// Whether two straight sides in the chart cross or touch.
fn two_sides_meet([a, b]: [[f64; 2]; 2], [c, d]: [[f64; 2]; 2]) -> bool {
    let turn = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    };
    let lies_along = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        passes_or_ends_at(r[0], p[0], q[0]) && passes_or_ends_at(r[1], p[1], q[1])
    };
    let [one, two, three, four] = [turn(c, d, a), turn(c, d, b), turn(a, b, c), turn(a, b, d)];
    let apart = |x: f64, y: f64| (x > 0.0 && y < 0.0) || (x < 0.0 && y > 0.0);
    (apart(one, two) && apart(three, four))
        || (one == 0.0 && lies_along(c, d, a))
        || (two == 0.0 && lies_along(c, d, b))
        || (three == 0.0 && lies_along(a, b, c))
        || (four == 0.0 && lies_along(a, b, d))
}

/// Whether the side between two corners meets the cut at `longitude` from
/// latitude `from` to `to`, leaving `from` itself out.
fn a_side_meets_a_cut([one, other]: [[f64; 2]; 2], longitude: f64, [from, to]: [f64; 2]) -> bool {
    let on_the_cut = |latitude: f64| latitude != from && passes_or_ends_at(latitude, from, to);
    if one[0] == longitude && other[0] == longitude {
        let (low, high) = (one[1].min(other[1]), one[1].max(other[1]));
        return on_the_cut(low)
            || on_the_cut(high)
            || passes_strictly(from, low, high)
            || passes_strictly(to, low, high);
    }
    if !passes_or_ends_at(longitude, one[0], other[0]) {
        return false;
    }
    let part = (longitude - one[0]) / (other[0] - one[0]);
    on_the_cut(one[1] + part * (other[1] - one[1]))
}

/// Whether `at` lies between `one` and `other`, or on either.
fn passes_or_ends_at(at: f64, one: f64, other: f64) -> bool {
    one.min(other) <= at && at <= one.max(other)
}

/// Which of the corners of a cap that `room` names places in `among` for sits
/// nearest a longitude, as its place in `among`, and of two as near, the one
/// off the pole.
///
/// The end of a pole side shares its longitude with the rim corner beside it,
/// because the rim leaves the pole up that meridian, and a cut from the pole
/// end would run along the rim's own step (see [`starting_off_the_poles`]).
fn nearest_corner_off_the_rims_poles(
    cap: &WalkedLoop,
    room: impl Iterator<Item = usize>,
    among: &[usize],
    at: f64,
    pinch: f64,
) -> usize {
    let on_a_pole = |place: usize| {
        let latitude = cap.corners[among[place]][1];
        latitude == pinch || latitude == -pinch
    };
    room.min_by(|&one, &other| {
        let away = |place: usize| (cap.corners[among[place]][0] - at).abs();
        away(one)
            .total_cmp(&away(other))
            .then(on_a_pole(one).cmp(&on_a_pole(other)))
    })
    .expect("a walked loop has a corner, and a quarter still to place has one left")
}

/// Which corner of a loop sits nearest a longitude.
///
/// **Not counting whole turns as no distance at all**, which is the one thing a
/// chain that goes right round makes wrong: its first corner and its last are a
/// whole turn apart and would be equally near everything, so which of the two
/// came back would decide whether the stretch after it goes a quarter of the way
/// round or three quarters.
///
/// **Of two as near, the one off the pole at `pinch`**, for the reason
/// [`nearest_corner_off_the_rims_poles`] gives a cap: the end of a pole side
/// shares its longitude with the corner the loop leaves the pole down to, and a
/// cut that lands on the pole end from the far side of that meridian crosses
/// the loop's own step down it. A ring whose hole runs through a ball's pole
/// and starts there had its first quarter on that end, and the patch was
/// refused `BoundaryCrossesItself`
/// (`a_half_ball_holed_through_its_pole_and_capped_on_the_same_ball.rs`).
fn nearest_longitude(around: &WalkedLoop, at: f64, pinch: Option<f64>) -> usize {
    let on_a_pole = |which: usize| {
        let latitude = around.corners[which][1];
        pinch.is_some_and(|pinch| latitude == pinch || latitude == -pinch)
    };
    (0..around.corners.len())
        .min_by(|&one, &other| {
            let away = |which: usize| (around.corners[which][0] - at).abs();
            away(one)
                .total_cmp(&away(other))
                .then(on_a_pole(one).cmp(&on_a_pole(other)))
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
        ..chain.clone()
    }
}

/// An island moved by whole turns of longitude so that its middle is within
/// half a turn of `longitude`.
fn in_the_turn_nearest(island: &WalkedLoop, longitude: f64, whole: f64) -> WalkedLoop {
    let (least, most) = longitudes_of(island);
    let by = whole_turns_between((least + most) / 2.0, longitude, whole);
    WalkedLoop {
        corners: island
            .corners
            .iter()
            .map(|corner| [corner[0] - by, corner[1]])
            .collect(),
        ..island.clone()
    }
}

/// One loop moved by whole turns of its second coordinate into the turn the
/// face puts it in against `frame`.
///
/// **Each loop of a torus face is unwrapped round the ring from its own first
/// corner**, so two loops of one face can come out a whole turn apart, which in
/// the chart puts a hole outside the loop it is a hole in. A hole's ring angles
/// lie inside its outer loop's, so the middle of a hole is at most half the
/// outer loop's span from the outer loop's middle. For an outer loop spanning
/// less than a whole turn round the ring that is less than half a turn in
/// exactly one turn, and this finds it whichever of the two is the face's
/// first wire. A loop already in that turn keeps its numbers bit for bit.
/// Two loops that each go right round the ring span a whole turn each, so any
/// turn is theirs; [`BuildingAMesh::cut_a_ring_across`] brings them into one,
/// and what places them is the angle round the tube
/// ([`with_its_tube_angle_on_the_faces_side_of`]).
///
/// **Two loops side by side are placed by which way `frame` runs, and never by
/// which way round is nearer.** Two loops that each go right round the tube,
/// neither inside the other, bound two bands round the ring. The nearer way
/// round is the right band only when the face is the narrower one, and a face
/// between two circles half a turn apart is a tie: the nearer way broke it
/// towards the lower half whichever half the face was, and the builder refused
/// the upper half `FacesDoNotAgreeWhichWayTheyFace`
/// (`two_loops_side_by_side_on_a_torus.rs`, `docs/decisions.md` §343).
///
/// The side is read off a quarter band that validates and measures what Pappus
/// gives: the loop that runs down the tube angle is the lower one in ring
/// angle. So the face lies above a loop running down and below one running up,
/// and the other loop goes in the nearest turn on that side. A face used the
/// other way round with every coedge turned over is walked the same way here,
/// because a view walks a reversed face's wires backwards.
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
    let turns = (middle(chain) - middle(frame)) / whole;
    let by = if chain.goes_right_round && frame.goes_right_round {
        if runs_up(frame) {
            turns.ceil() * whole
        } else {
            turns.floor() * whole
        }
    } else {
        whole_turns_between(middle(chain), middle(frame), whole)
    };
    WalkedLoop {
        corners: chain
            .corners
            .iter()
            .map(|corner| [corner[0], corner[1] - by])
            .collect(),
        ..chain.clone()
    }
}

/// One loop that goes right round a torus's ring moved by whole turns of the
/// angle round the tube onto the side of `frame` the face lies on.
///
/// Two such loops bound two bands, one over the top of the tube between them
/// and one under it, and only which way `frame` runs says which the face is —
/// the same question [`with_its_second_coordinate_in_the_turn_of`] answers for
/// two loops round the tube, asked the other way round. A torus face keeps its
/// material on its left, and the chart's turn `∂u × ∂v` points into the tube
/// while the face points out of it (§329.1), so the loop runs clockwise in
/// `(u, v)`: up the angle round the ring along the low side of the band in
/// `u`, and down it along the high side. So the face lies above a loop running
/// up the ring and below one running down it, and the other loop goes in the
/// nearest turn on that side. The top half of a ring is the case that checks
/// it: its outer equator at `u = 0` runs up the ring, its inner one at `u = π`
/// runs down, and the face is the `u` between them over the top of the tube.
///
/// **Middles are enough**, by 343.2's argument with the two coordinates
/// swapped: two disjoint loops that each go once round the ring are one beside
/// the other in `u`.
fn with_its_tube_angle_on_the_faces_side_of(
    chain: &WalkedLoop,
    frame: &WalkedLoop,
    whole: f64,
) -> WalkedLoop {
    let middle = |around: &WalkedLoop| {
        let (least, most) = longitudes_of(around);
        (least + most) / 2.0
    };
    let turns = (middle(chain) - middle(frame)) / whole;
    let by = if frame.climbed_the_second_way > 0.0 {
        turns.floor() * whole
    } else {
        turns.ceil() * whole
    };
    WalkedLoop {
        corners: chain
            .corners
            .iter()
            .map(|corner| [corner[0] - by, corner[1]])
            .collect(),
        ..chain.clone()
    }
}

/// Whether a loop on a surface whose second coordinate wraps passes one point
/// at two corners a whole turn apart that way — on a torus, a seam across the
/// tube, so that its patch reaches a whole turn round the ring and closes.
fn reaches_a_whole_turn_round_the_second_way(around: &WalkedLoop, whole: f64) -> bool {
    let mut first_seen_at: HashMap<u32, f64> = HashMap::new();
    around
        .points
        .iter()
        .zip(&around.corners)
        .any(|(&at, corner)| {
            let first_seen = *first_seen_at.entry(at).or_insert(corner[1]);
            (corner[1] - first_seen).abs() >= whole / 2.0
        })
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
        ..chain.clone()
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

/// Whether no point of a loop stands at two of its corners: no pole side, no
/// seam and no slit, so that it is a curve drawn once round in the chart.
fn it_passes_each_of_its_points_once(around: &WalkedLoop) -> bool {
    let mut seen = HashSet::with_capacity(around.points.len());
    around.points.iter().all(|&at| seen.insert(at))
}

/// One walked wire cut into the loops it is made of, at each of `twice` it
/// passes twice, the loop that holds the most of the chart first: the outline
/// before a hole that touches it.
///
/// **Only a cut into loops that each pass every point once is taken.** A slit
/// runs out along one edge and back from a corner that starts two of its
/// coedges, and the loop out and back passes each point of the slit twice, so
/// it stays whole for [`BuildingAMesh::open_a_slit`]. So does a wire that goes
/// right round either way: a seam's corner starts two coedges a whole turn
/// apart, and [`apart_at`] would not cut there anyway.
fn apart_where_it_passes_a_corner_twice(around: WalkedLoop, twice: &[u32]) -> Vec<WalkedLoop> {
    if around.goes_right_round || around.goes_right_round_the_second_way {
        return vec![around];
    }
    let mut loops = vec![around];
    for &at in twice {
        loops = loops
            .into_iter()
            .flat_map(|one| match apart_at(&one, at) {
                Some(parts) if parts.iter().all(it_passes_each_of_its_points_once) => {
                    parts.to_vec()
                }
                Some(_) | None => vec![one],
            })
            .collect();
    }
    loops.sort_by(|one, other| {
        how_much_of_the_chart_this_loop_encloses(other)
            .abs()
            .total_cmp(&how_much_of_the_chart_this_loop_encloses(one).abs())
    });
    loops
}

/// A piece between two holes that touch, cut apart where they touch into the
/// pieces it is made of, or the piece whole unless each of them has the
/// face's material inside it.
///
/// Two holes touching at a point leave the piece between them pinched shut
/// there: one piece above the point and one below, joined at it.
/// [`BuildingAMesh::cut_holes_side_by_side_up_to_both_poles`] walks round both
/// as one loop through that point twice, which the triangulator refuses.
fn apart_where_two_holes_touch(piece: WalkedLoop, touching: &[u32]) -> Vec<WalkedLoop> {
    let mut pieces = vec![piece];
    for &at in touching {
        pieces = pieces
            .into_iter()
            .flat_map(|one| match apart_at(&one, at) {
                Some(parts)
                    if parts
                        .iter()
                        .all(|part| how_much_of_the_chart_this_loop_encloses(part) > 0.0) =>
                {
                    parts.to_vec()
                }
                Some(_) | None => vec![one],
            })
            .collect();
    }
    pieces
}

/// Every point that two different loops of `loops` pass, once in the list.
fn the_points_two_loops_pass(loops: &[WalkedLoop]) -> Vec<u32> {
    let mut on: HashMap<u32, usize> = HashMap::new();
    for (which, around) in loops.iter().enumerate() {
        for &at in &around.points {
            on.entry(at).or_insert(which);
        }
    }
    let mut shared: Vec<u32> = loops
        .iter()
        .enumerate()
        .flat_map(|(which, around)| {
            let on = &on;
            around
                .points
                .iter()
                .copied()
                .filter(move |at| on[at] != which)
        })
        .collect();
    shared.sort_unstable();
    shared.dedup();
    shared
}

/// `around` cut in two where it passes point `at`: the loop from the first
/// time it gets there up to the second, and the rest.
///
/// `None` unless it passes `at` exactly twice, **at one corner of the chart
/// both times**, with at least three corners each side. The corner is what
/// makes each part close in the chart as `around` did; a seam's corner, a whole
/// turn apart, or a pole's two ends of a side, are no place to cut.
fn apart_at(around: &WalkedLoop, at: u32) -> Option<[WalkedLoop; 2]> {
    let count = around.points.len();
    let mut visits = (0..count).filter(|&corner| around.points[corner] == at);
    let (Some(first), Some(second), None) = (visits.next(), visits.next(), visits.next()) else {
        return None;
    };
    if around.corners[first] != around.corners[second]
        || second - first < 3
        || count - (second - first) < 3
    {
        return None;
    }
    Some([
        around.between(first, second - 1),
        around.between(second, (first + count - 1) % count),
    ])
}

/// The longitude halfway across the widest stretch of longitude that none of
/// `spans` reaches, each span the least and most longitude of one hole in the
/// hole's own turn, or `None` when they reach every longitude between them.
///
/// A stretch starts where a hole ends and no other hole goes on, and runs to
/// the nearest place a hole starts, its own included a whole turn on. Two holes
/// that touch at one longitude leave no stretch between them there.
fn the_middle_of_the_widest_stretch_no_hole_reaches(
    spans: &[(f64, f64)],
    whole: f64,
) -> Option<f64> {
    let mut widest: Option<(f64, f64)> = None;
    for (at, &(_, ends)) in spans.iter().enumerate() {
        let another_goes_on = spans.iter().enumerate().any(|(other, &(least, most))| {
            other != at && (ends - least).rem_euclid(whole) < most - least
        });
        if another_goes_on {
            continue;
        }
        let across = spans
            .iter()
            .map(|&(least, _)| (least - ends).rem_euclid(whole))
            .fold(whole, f64::min);
        if across > 0.0 && widest.is_none_or(|(most, _)| across > most) {
            widest = Some((across, ends + across / 2.0));
        }
    }
    widest.map(|(_, middle)| middle)
}

/// A strip of a ball from pole to pole between the meridians at `left` and
/// `right`, walked anticlockwise because the material is inside it: along the
/// bottom pole side, up the right meridian, back along the top pole side and
/// down the left one. Each meridian is the points strictly between its two
/// poles, walked upwards, and its corners take the strip's own longitude.
fn a_strip_from_pole_to_pole(
    [left, right]: [f64; 2],
    [left_side, right_side]: [&WalkedLoop; 2],
    [south, north]: [u32; 2],
    pole: f64,
) -> WalkedLoop {
    let mut strip = WalkedLoop {
        corners: vec![[left, -pole], [right, -pole]],
        points: vec![south, south],
        ..WalkedLoop::a_stretch(0)
    };
    for (corner, &point) in right_side.corners.iter().zip(&right_side.points) {
        strip.corners.push([right, corner[1]]);
        strip.points.push(point);
    }
    strip.corners.extend([[right, pole], [left, pole]]);
    strip.points.extend([north, north]);
    for (corner, &point) in left_side.corners.iter().zip(&left_side.points).rev() {
        strip.corners.push([left, corner[1]]);
        strip.points.push(point);
    }
    strip
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
/// only call site whose sign matters is guarded by [`Surface::where_it_pinches`],
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
    let mut opened = WalkedLoop::a_stretch(count + 2);
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

/// Whether the straight chord between corners `from` and `to` of a loop lies
/// inside it in the chart: it meets no side but at its own two ends, and a
/// point half way along it is inside.
///
/// **Touching counts as meeting**, as it does for a cut up to a cap's pole
/// ([`a_cut_up_meets_the_rim`]): a chord through a third corner is two sides
/// for the triangulator, and a chord along a side draws that side twice. A
/// chord that meets no side is wholly inside or wholly outside, which the
/// point half way along tells apart.
fn a_chord_stays_inside(around: &WalkedLoop, [from, to]: [usize; 2]) -> bool {
    let count = around.corners.len();
    let chord = [around.corners[from], around.corners[to]];
    let meets_a_side = (0..count).any(|at| {
        let next = (at + 1) % count;
        let side = [around.corners[at], around.corners[next]];
        match [at, next]
            .iter()
            .filter(|&&end| end == from || end == to)
            .count()
        {
            0 => two_sides_meet(chord, side),
            1 => {
                let far = if at == from || at == to {
                    side[1]
                } else {
                    side[0]
                };
                let own = if at == from || next == from {
                    chord[1]
                } else {
                    chord[0]
                };
                a_point_lies_along(chord, far) || a_point_lies_along(side, own)
            }
            _ => true,
        }
    });
    let half_way = [0, 1].map(|which| (chord[0][which] + chord[1][which]) / 2.0);
    !meets_a_side && a_loop_holds(around, half_way)
}

/// Whether `point` lies on the straight side between `ends`, either end
/// included.
fn a_point_lies_along(ends: [[f64; 2]; 2], point: [f64; 2]) -> bool {
    let [a, b] = ends;
    (b[0] - a[0]) * (point[1] - a[1]) - (b[1] - a[1]) * (point[0] - a[0]) == 0.0
        && passes_or_ends_at(point[0], a[0], b[0])
        && passes_or_ends_at(point[1], a[1], b[1])
}

/// The two corners, in loop order, whose chord stays inside a piece that
/// goes `span` round and leaves the wider of its two halves narrowest, both
/// narrower than `span`; of two as good, the one found first. `None` when no
/// chord does.
///
/// Both ends are corners the piece already has, for the reason
/// [`where_to_cut`] gives, and two corners at one point — the ends of a pole
/// side, or a seam's — are not a chord. Each half's reach is read off running
/// least and most longitudes, so finding it costs the square of the corners,
/// and each chord that might beat the best so far is checked against every
/// side.
fn the_most_even_chord_that_stays_inside(piece: &WalkedLoop, span: f64) -> Option<[usize; 2]> {
    let count = piece.corners.len();
    let longitude = |corner: usize| piece.corners[corner][0];
    let mut from_the_end = vec![(f64::MAX, f64::MIN); count + 1];
    for corner in (0..count).rev() {
        let (least, most) = from_the_end[corner + 1];
        from_the_end[corner] = (least.min(longitude(corner)), most.max(longitude(corner)));
    }
    let mut best: Option<(f64, [usize; 2])> = None;
    let mut up_to_from = (f64::MAX, f64::MIN);
    for from in 0..count {
        up_to_from = (
            up_to_from.0.min(longitude(from)),
            up_to_from.1.max(longitude(from)),
        );
        let mut from_to = (longitude(from), longitude(from));
        for (to, &(least, most)) in from_the_end.iter().enumerate().take(count).skip(from + 1) {
            from_to = (from_to.0.min(longitude(to)), from_to.1.max(longitude(to)));
            if to == from + 1 || (from == 0 && to == count - 1) {
                continue;
            }
            if piece.points[from] == piece.points[to] {
                continue;
            }
            let round_the_rest = up_to_from.1.max(most) - up_to_from.0.min(least);
            let wider = (from_to.1 - from_to.0).max(round_the_rest);
            if wider >= span || best.is_some_and(|(narrowest, _)| wider >= narrowest) {
                continue;
            }
            if a_chord_stays_inside(piece, [from, to]) {
                best = Some((wider, [from, to]));
            }
        }
    }
    best.map(|(_, at)| at)
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

/// Whether a side of `around` meets the line along `longitude` between two
/// latitudes, both ends included, or lies along any of it.
fn a_line_meets(around: &WalkedLoop, longitude: f64, [from, to]: [f64; 2]) -> bool {
    let (low, high) = (from.min(to), from.max(to));
    let count = around.corners.len();
    (0..count).any(|at| {
        let (here, next) = (around.corners[at], around.corners[(at + 1) % count]);
        if !passes(longitude, here[0], next[0]) {
            return false;
        }
        if here[0] == next[0] {
            return here[1].max(next[1]) >= low && here[1].min(next[1]) <= high;
        }
        let crossing = here[1] + (longitude - here[0]) / (next[0] - here[0]) * (next[1] - here[1]);
        (low..=high).contains(&crossing)
    })
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
    /// a seam. That is the whole of
    /// [`WalkingAWire::may_come_back_a_whole_turn_on`].
    no_side_of_this_wire_is_a_seam: bool,
    longitude: f64,
    started: bool,
    will_not_lie_flat: bool,
    /// A pole side the walk has crossed and whose far end is not written yet,
    /// because it is wherever the next corner with a longitude of its own sits.
    /// See [`APoleSideStillOpen`].
    crossing: Option<APoleSideStillOpen>,
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
                Some(APoleSideStillOpen::AWholeTurn { way }) => {
                    let past_it = self.longitude + way * whole;
                    past_it + the_short_way_round(raw - past_it, whole)
                }
                Some(APoleSideStillOpen::BetweenTwoMeridians { way }) => {
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
    /// turns straight back up itself, so its pole side is a whole turn. A
    /// **lune** — a sphere face between two different half-meridians, which is
    /// the skin of a quarter of a ball — arrives down one and leaves up the
    /// other, and its pole side is the angle between them. Charging a whole
    /// turn there too made a quarter's skin a turn and a quarter wide and its
    /// volume `5.209` where `π/3` is the truth (§166).
    ///
    /// **Either way the far end is left open until the next corner with a
    /// longitude is walked**, and that corner's longitude is the far end's:
    /// see [`APoleSideStillOpen::AWholeTurn`] for what a seam's cost when it
    /// was settled here instead.
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
        if self.wraps.is_none() {
            self.will_not_lie_flat = true;
            return;
        }
        self.still_at_a_pole = None;
        let Some(way) = self.which_way_the_pole_side_runs() else {
            self.will_not_lie_flat = true;
            return;
        };
        self.crossing = Some(if a_seam_turns_back_here {
            APoleSideStillOpen::AWholeTurn { way }
        } else {
            APoleSideStillOpen::BetweenTwoMeridians { way }
        });
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
    ///
    /// **Nothing to do either while a pole side is still open.** A corner on a
    /// pole met before any corner with a longitude stands on that same side, a
    /// hair along it at most, so it changes neither which way the side runs nor
    /// how far. Opened again here, a seam's whole turn became
    /// [`APoleSideStillOpen::BetweenTwoMeridians`], and that reads how far apart
    /// two readings of the seam's one meridian are: nothing, which it counts as
    /// a whole turn, but a hair when the readings round a hair apart, which
    /// they do once the walk has carried the longitude a whole turn. A ball of
    /// radius `0.001` whose seam has a corner on its equator and one `3e-14`
    /// from a pole crossed that pole a hair rather than a whole turn at 22 of
    /// 720 whole degrees and poles, and was refused as a wire that will not lie
    /// flat (Q1309, `a_pole_corner_met_while_a_pole_side_is_open.rs`).
    fn turn_at_a_corner_on_a_pole(&mut self) {
        if !self.started || self.wraps.is_none() || self.crossing.is_some() {
            return;
        }
        let Some(way) = self.which_way_the_pole_side_runs() else {
            self.will_not_lie_flat = true;
            return;
        };
        self.crossing = Some(APoleSideStillOpen::BetweenTwoMeridians { way });
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
        // A pole side still open at the end closes on the corner the walk set
        // off from, which is where the loop closes; its far end is that corner,
        // and so is the far corner of a pole the walk is still standing on.
        //
        // **That corner goes as many whole turns on from the first corner as
        // the side's count put the walk, and not at the first corner's
        // longitude as read.** A ball's piece below a plane whose circle runs
        // through `N` is walked to `N` last, and the side there carries the
        // walk a whole turn on from the rim's corner, so `climbed` below says
        // a whole turn. Written at the rim corner's longitude as read, the
        // side ran back across the piece instead, and the block less the ball
        // and the two kept in common were refused
        // `FacesDoNotAgreeWhichWayTheyFace` (Q963). The turns are added to the
        // first corner's longitude rather than the walk's own taken, because
        // the walk's own comes back as much as `4.4e-16` off the first corner
        // where it comes back no turns on, in rcad-topo's own turned ball and
        // block tests, and that side still closes on the first corner exactly,
        // as it always did.
        if let (Some(whole), Some(side), Some(first)) = (
            self.wraps,
            self.crossing.take(),
            self.corners.first().copied(),
        ) {
            self.longitude += match side {
                APoleSideStillOpen::AWholeTurn { way } => way * whole,
                APoleSideStillOpen::BetweenTwoMeridians { way } => {
                    way * how_far_a_pole_side_runs(way * (first[0] - self.longitude), whole)
                }
            };
            self.leave_the_pole_at(first[0] + whole_turns_between(self.longitude, first[0], whole));
        }
        // A walk that ends at a pole with no side open leaves it by the way it
        // set off, because that is where the loop closes, so the pole side is
        // the last side of the patch and the longitude the walk ends at is the
        // one it started at.
        if let Some(first) = self.corners.first().copied()
            && self.still_at_a_pole.is_some()
        {
            self.leave_the_pole_at(first[0]);
            self.longitude = first[0];
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
        // **Round the ring the same rule holds**, read the same way, on a
        // torus, whose second coordinate is the angle round the ring. A wire
        // that comes back a whole turn on round the ring is one of the two
        // circles of a band round it, which
        // [`BuildingAMesh::cut_a_band_right_round_the_ring`] cuts; it may come
        // back a whole turn on only where it may round the tube. One that came
        // back a whole turn on both ways round is no loop a face has.
        //
        // **A seam across the tube is not refused here any more.** Its wire
        // passes the seam's points at two corners a whole turn apart round the
        // ring and closes in the chart, so it is an ordinary loop a whole turn
        // wide, and [`BuildingAMesh::cut_what_reaches_right_round_the_ring`]
        // cuts it narrower (`docs/decisions.md` §390).
        let (climbed_the_second_way, goes_right_round_the_second_way) =
            match (self.second_wraps, self.corners.first(), self.corners.last()) {
                (Some(whole), Some(first), Some(last)) => {
                    let climbed =
                        last[1] - first[1] + the_short_way_round(first[1] - last[1], whole);
                    let went_round = climbed.abs() >= whole / 2.0;
                    if went_round
                        && (goes_right_round
                            || !self.closes_on_one_curve && !self.may_come_back_a_whole_turn_on())
                    {
                        return None;
                    }
                    (climbed, went_round)
                }
                _ => (0.0, false),
            };
        Some(WalkedLoop {
            corners: self.corners,
            points: self.points,
            closes_on_one_curve: self.closes_on_one_curve,
            goes_right_round,
            climbed,
            goes_right_round_the_second_way,
            climbed_the_second_way,
        })
    }

    /// Whether this wire is allowed to end a whole turn of longitude on from
    /// where it set off — a **ring**, one simple closed curve drawn on the
    /// surface, rather than a boundary that has to close in the chart.
    ///
    /// A wire that uses an edge twice uses it as a **seam**: the two sides of
    /// one meridian, which the patch lies between, so the walk down the far
    /// side has to bring the longitude back to where the walk up the near side
    /// left it. That may not wind, and it is all this keeps out.
    ///
    /// **A wire through one pole may wind round the other**, and keeping it
    /// out cost a correct answer. The half frustum `x ≥ 0` against a ball four
    /// up its axis leaves the ball less the piece inside the half as one face
    /// whose one ring of two arcs runs over the north pole along the wall
    /// `x = 0`. The face holds the south pole, so the walk round that ring comes
    /// back `−6.283185307179586`, a whole turn, and it was refused
    /// `AWireWillNotLieFlatOnItsFace` because it had stood at a pole
    /// (`a_cone_hollowed_along_its_axis.rs`,
    /// `a_half_frustum_whose_wall_crosses_the_circle_it_meets_a_bar_or_a_ball_in_answers_every_rule`).
    /// The pole side's width is not a guess: [`WalkingAWire::cross_the_pole`]
    /// and [`WalkingAWire::turn_at_a_corner_on_a_pole`] run it the way that
    /// keeps the face on the left, so the turn the walk climbs is the turn the
    /// wire makes. Which pole such a cap reaches is
    /// [`BuildingAMesh::which_pole_this_cap_reaches`]'s, and that never takes a
    /// pole a corner of the rim stands on, so a rim through one pole reaches
    /// the other, and a rim through both reaches neither and is refused there
    /// as `ACurvedFaceGoesRightRoundWithNoSeam`.
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
    /// at three corners off the apex. Three distinct edges and no seam — so the
    /// ring closes right round the longitude, and the face is the piece of the
    /// nappe the apex is in. [`BuildingAMesh::cut_a_cap_up_to_its_pole`] is
    /// what fills it, and [`BuildingAMesh::which_pole_this_cap_reaches`] is
    /// what says which of the two pieces the ring bounds is the face: the one
    /// on the left of the walk, and the apex either is on that side or the face
    /// is the unbounded rest of the nappe, which is no face at all and stays
    /// refused.
    ///
    /// One coedge whose two ends are its one corner is a ring too, and it is
    /// the one this kernel already meshed: `closes_on_one_curve` says it, the
    /// caller asks it first, and it needs none of the reasoning above.
    fn may_come_back_a_whole_turn_on(&self) -> bool {
        self.no_side_of_this_wire_is_a_seam
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

/// A pole side the walk has crossed and not yet left, and which way the
/// longitude runs along it: `1` with it, `-1` against it, as
/// [`WalkingAWire::which_way_the_pole_side_runs`] reads it.
///
/// **Its far end is written at the longitude of the corner the walk leaves the
/// pole by, and is that number.** [`WalkingAWire::arrive`] works the corner's
/// longitude out once and [`WalkingAWire::leave_the_pole_at`] writes the far end
/// with it, so the two cannot come out a rounding apart, however either was
/// rounded.
#[derive(Clone, Copy)]
enum APoleSideStillOpen {
    /// A seam that arrives at the pole and goes straight back up itself, so the
    /// side is a whole turn and the corner after it is the corner before it, a
    /// whole turn on.
    ///
    /// **Settling it where the pole is crossed wrote its two ends a rounding
    /// apart.** The far end went in as the seam's longitude plus a whole turn,
    /// rounded once, and the corner back up the seam as that plus the short way
    /// round to the seam's own reading, rounded three times. Where the seam's
    /// longitude plus a whole turn lands between 8 and 16 a rounding there is
    /// `1.8e-15`, the two came out that far apart, and the patch got a pole side
    /// one rounding wide with nowhere beside it for the triangulator to put a
    /// point: a whole ball cut open at 99 degrees from `x` was refused "the mesh
    /// still strays 1.67", as were 13 of 3600 seams a tenth of a degree apart,
    /// from 99.0 to 114.1 degrees (Q1304,
    /// `a_ball_whose_seam_is_a_little_past_a_quarter_turn.rs`).
    AWholeTurn { way: f64 },
    /// Two different meridians, so the side runs as far as they are apart, the
    /// way it runs: [`how_far_a_pole_side_runs`].
    BetweenTwoMeridians { way: f64 },
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

/// Where a corner of `face` sits in the frame the face is meshed in.
///
/// **Read out of the arena for a placed face**, for
/// [`the_surface_this_face_is_meshed_on`]'s reason: that frame is the
/// placement's own inverse, so moving [`VertexView::point`] into it is the
/// placement composed with its own inverse, and the stored corner rounded twice.
/// `(3, 0, 0)` turned an eighth about `z` and back is `(3, −4.4e−16, 0)`.
///
/// That holds only while the corner is looked at through the same placement as
/// the face, and it always is: a view hands its own location down unchanged to
/// every wire, coedge, edge and vertex under it, and every corner here is
/// reached through the face's own wires. A corner through any other location
/// would be a view built somewhere else and handed in by mistake, and reading
/// its stored point would put it where the face is not.
///
/// A face at no placement at all is moved by [`a_shift_that_loses_nothing`],
/// which is exact, and its corner is its stored point to begin with — so that
/// face keeps the movement it always had.
fn where_this_corner_is_in_the_frame_it_is_meshed_in(
    corner: VertexView<'_>,
    face: FaceView<'_>,
) -> Point {
    if face.location() != Isometry::identity() {
        assert!(
            corner.location() == face.location(),
            "a corner of {:?} looked at through {:?} rather than the face's own {:?}",
            face.key(),
            corner.location(),
            face.location(),
        );
        return corner.stored_point();
    }
    the_frame_this_face_is_meshed_in(face).apply_to_point(corner.point())
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
    surface
        .where_it_pinches()
        .is_some_and(|pinch| latitude.abs() == pinch)
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

/// Whether some flat face of `solid` that does not run along `edge` has both of
/// the edge's ends among the corners of its wires.
///
/// **Only then can a flat face draw a side straight between them.** A flat face
/// is handed to the triangulator with no point inside it (see
/// [`BuildingAMesh::hand_over`]), so every side it draws joins two corners of
/// its own wires. Two corners a straight edge runs between put that whole edge
/// in the face's plane, and a face that does not run along it holds it across
/// its middle or not at all: a solid that touches itself along a line, which
/// the fuse leaves where two faces rest against each other along one.
fn a_flat_face_off_it_holds_both_its_ends(solid: SolidView<'_>, edge: EdgeView<'_>) -> bool {
    let ends = [edge.start().key(), edge.end().key()];
    if ends[0] == ends[1] {
        return false;
    }
    solid.faces().any(|face| {
        let coedges = || face.wires().flat_map(WireView::coedges);
        face.surface().as_plane().is_some()
            && coedges().all(|coedge| coedge.edge().key() != edge.key())
            && ends
                .iter()
                .all(|&end| coedges().any(|coedge| coedge.start().key() == end))
    })
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

/// Which way a closed edge runs as the faces along it walk it, for an edge
/// that nothing read at its corner names a way for.
///
/// **There the way an edge is stored running is not a fact, and the way each
/// face walks it is.** A rim through a ball's pole meets the ball's longitude
/// and its latitude where both are exactly zero, so
/// [`the_way_a_closed_edge_is_stored_running`] has nothing to read and
/// `Model::validate` skips the edge: written either way round, the solid is
/// sound (`docs/decisions.md` §341.4). Which way each use runs in space does
/// not need the longitude. A face's material lies on its left, and left of
/// `outward × ∇P` is `P`'s negative side, with `P` the second surface the run
/// is between — the rule `Model::validate` reads a closed edge by
/// ([`crate::validate::a_closed_crossing_is_walked_the_way_it_is_stored`]),
/// read without the reference that vanishes at the pole. So a use walks along
/// `outward × ∇P` where its face lies inside `P` and against it where it lies
/// outside, and the edge runs the way its forward use walks.
///
/// **What it replaced was the walker's own order**, `∇S × ∇T`, which turns
/// round with whichever way the rim's plane faces. A half ball on a pipe of its
/// own radius with its rim cornered on the pole built in two of the four ways
/// it can be written, and which two depended on that plane
/// (`a_half_ball_on_a_pipe_of_its_own_radius.rs`, §341.5).
///
/// Asked of the face walking the edge first and then of the face across. The
/// ball's half of that rim has every corner on the rim's plane, so it has no
/// side to read and the pipe's corner on the lid answers. Two faces of a sound
/// solid give one answer, since their uses run opposite ways.
///
/// `None` for a seam, and where neither face has a corner off `P` or its
/// gradient crosses `P`'s to nothing — two faces that touch rather than cross,
/// which [`the_way_a_closed_edge_where_its_faces_touch_runs`] is asked first.
///
/// # Errors
///
/// Whatever reading a corner's side refuses with, the same refusals
/// `Model::validate` gives on the same corners.
fn the_way_the_faces_along_a_closed_edge_walk_it(
    solid: SolidView<'_>,
    walking: FaceView<'_>,
    edge: EdgeView<'_>,
    back: &Isometry,
    at: Point,
) -> Result<Option<Vector>> {
    let Some(across) = the_face_across(solid, walking, edge)? else {
        return Ok(None);
    };
    for (face, other) in [(walking, across), (across, walking)] {
        if let Some(way) =
            the_way_this_face_walks_a_closed_edge(solid, [face, other], edge, back, at)?
        {
            return Ok(Some(way));
        }
    }
    Ok(None)
}

/// Which way the edge runs if `face`'s use of it is right, read off which side
/// of the run's second surface `face` lies on.
///
/// `back` is the frame the edge is walked in and `at` its corner there. The
/// gradients are read in that frame, and the side where the solid sits, as
/// `Model::validate` reads it: a movement keeps which side a corner is on.
///
/// # Panics
///
/// If `face` has no use of `edge`, which a face found along the edge always
/// has.
fn the_way_this_face_walks_a_closed_edge(
    solid: SolidView<'_>,
    [face, other]: [FaceView<'_>; 2],
    edge: EdgeView<'_>,
    back: &Isometry,
    at: Point,
) -> Result<Option<Vector>> {
    let in_the_frame = |surface: Surface| {
        SurfaceAlongAnEdge {
            stored: surface,
            placed_at: face.location(),
        }
        .in_the_frame(back)
    };
    let partner = which_two_surfaces_a_run_is_between(
        edge.stored_run(),
        [&face.stored_surface(), &other.stored_surface()],
    )[1];
    let outward = gradient_of(&in_the_frame(face.stored_surface()), at) * face.orientation().sign();
    let forward = outward.cross(gradient_of(&in_the_frame(partner), at));
    if forward == Vector::zero() {
        return Ok(None);
    }
    let Some(material) =
        which_side_of_a_surface_a_run_names_this_face_lies_on(solid, face, partner)?
    else {
        return Ok(None);
    };
    let walked = match material {
        Decision::Neg => forward,
        Decision::Pos => -forward,
        Decision::Zero | Decision::Undecidable { .. } | _ => {
            panic!("a face with no side to its material: {material:?}")
        }
    };
    let used = face
        .wires()
        .flat_map(WireView::coedges)
        .find(|coedge| coedge.edge().key() == edge.key())
        .expect("a face found along an edge uses it");
    Ok(Some(match used.orientation_on_its_edge() {
        Orientation::Forward => walked,
        Orientation::Reversed => -walked,
    }))
}

/// Which way a closed edge runs, read off a flat face across it that is a
/// disk over a ball — one wire, the whole edge, with the ball on its other
/// side — as `outward × ∇(the ball)`.
///
/// **A disk like this has no corner off the ball to read a side from**
/// (`docs/decisions.md` §220.6): a flat cut across a ball, or a cap through
/// the ball's pole with its rim lidded, closes with one wire and nothing
/// else. Asked of `face` first and then of the face across it, since only one
/// of the two can be the disk.
///
/// `None` where neither face is such a disk, or the disk's gradient crosses
/// the ball's to nothing at `corner`.
///
/// # Errors
///
/// Whatever `the_face_across` refuses with.
fn the_way_a_flat_disk_across_a_ball_walks_its_rim(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    edge: EdgeView<'_>,
    back: &Isometry,
    corner: Point,
) -> Result<Option<Vector>> {
    let Some(across) = the_face_across(solid, face, edge)? else {
        return Ok(None);
    };
    Ok([(face, across), (across, face)]
        .into_iter()
        .find_map(|(flat, round)| {
            let a_disk = flat.stored_surface().as_plane().is_some()
                && flat.wire_count() == 1
                && flat.outer_wire().closes_on_one_edge()
                && matches!(round.stored_surface(), Surface::Sphere(_));
            if !a_disk {
                return None;
            }
            let in_the_frame = |surface: Surface| {
                SurfaceAlongAnEdge {
                    stored: surface,
                    placed_at: flat.location(),
                }
                .in_the_frame(back)
            };
            let outward = gradient_of(&in_the_frame(flat.stored_surface()), corner)
                * flat.orientation().sign();
            let walked = outward.cross(gradient_of(&in_the_frame(round.stored_surface()), corner));
            let used = flat.outer_wire().coedges().next()?;
            (walked != Vector::zero()).then_some(match used.orientation_on_its_edge() {
                Orientation::Forward => walked,
                Orientation::Reversed => -walked,
            })
        }))
}

/// Which side of `named`, a surface a run of this face's names, the face lies
/// on, as its corners off `named` say.
///
/// **`Model::validate` asks this same question of the same corners**, in its
/// own private copy, and the two must stay one reading: the corners on `named`
/// are the ends of every edge of this face whose run is between something and
/// `named`, which no rounding can take off it ([`Trimming`]), and every other
/// corner is read. `None` when every corner of the face is on it.
fn which_side_of_a_surface_a_run_names_this_face_lies_on(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    named: Surface,
) -> Result<Option<Decision>> {
    let mut corners_on_it = Vec::new();
    for coedge in face.wires().flat_map(WireView::coedges) {
        let edge = coedge.edge();
        let beyond = the_face_across(solid, face, edge)?.unwrap_or(face);
        if which_two_surfaces_a_run_is_between(
            edge.stored_run(),
            [&face.stored_surface(), &beyond.stored_surface()],
        )
        .contains(&named)
        {
            corners_on_it.push(edge.start().key());
            corners_on_it.push(edge.end().key());
        }
    }
    the_side_this_face_lies_on(
        face,
        &Trimming {
            surface: named.transformed(&face.location()),
            corners_on_it,
        },
    )
}

/// Make every pole of a ball that a crossing runs through one of the points
/// walked along it, and say how far a walked point was moved to do it.
///
/// **The wire walk knows a pole only when a walked point lands on it**
/// ([`BuildingAMesh::walk_a_wire_round`]). A plane through a ball's axis cuts
/// it in a circle through both poles, and a walk that steps past a pole
/// between two points jumps half a turn of longitude with nothing to say which
/// way. Whether it landed on the pole was luck: at one deflection the walk of
/// `a_rim_through_both_poles_measures.rs` did, and at every finer one it did
/// not (§193.3).
///
/// A closed crossing is the whole curve, so a pole is on it exactly when both
/// surfaces hold the pole — two certified predicates and no distance compared
/// with anything. A circle between a ball and a plane that misses the pole by
/// one rounding or less is given it too, and the miss is charged — see
/// [`how_far_a_circle_that_passes_within_a_rounding_of_a_pole_misses_it`].
/// Any other pole whose coordinates round off the ball, or whose sides cannot
/// be decided, is left out, and the walk then goes on as it did before this
/// existed: [`WalkingAWire::finish`] checks it.
///
/// **An open crossing is asked as well, since Q1270**, and there the pole has
/// to lie between the two walked points it goes between, because the edge is
/// only part of the curve ([`HowMuchOfTheCrossing::BetweenItsEnds`]). A half
/// frustum `x ≥ 0` and a ball on its axis keep the ball's skin inside the
/// frustum above the circle they cross in, and the wall `x = 0` bounds it with
/// one arc over the ball's north pole, between two corners on the circle. The
/// walk stepped past the pole from `+y` to `−y`, half a turn the wrong way: the
/// skin was read as a cap right round the pole and cut up to it along the
/// meridian that arc already runs down, which left a patch through the pole
/// twice, refused `CornersCoincide`
/// (`a_half_frustum_whose_wall_crosses_the_circle_it_meets_a_bar_or_a_ball_in_answers_every_rule`).
fn put_in_the_poles_a_crossing_passes(
    points: &mut Vec<Point>,
    ends: [Point; 2],
    along: &[Surface; 2],
    how_much: HowMuchOfTheCrossing,
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
        let pinch = ball
            .where_it_pinches()
            .expect("a ball pinches at its poles");
        for up in [1.0, -1.0] {
            let pole = sphere.centre() + Vector::new(0.0, 0.0, up * sphere.radius());
            if let Some(far) =
                put_in_a_pole(points, ends, [ball, other], pole, up * pinch, how_much)
            {
                moved = moved.max(far);
            }
        }
    }
    moved
}

/// How much of the curve two surfaces cross in an edge's walked points stand
/// for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum HowMuchOfTheCrossing {
    /// The whole closed curve, so a pole both surfaces hold is on it.
    AllOfIt,
    /// The stretch between the edge's two ends, so a pole both surfaces hold
    /// may be on the rest of the curve instead.
    BetweenItsEnds,
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
///
/// **Of an open crossing the pair has to have the pole between them**, since
/// the far place the curve changes side is then the only one when the edge
/// stops short of the pole: `(a − pole)·(b − pole)` is below zero exactly when
/// the pole is inside the ball with `a` and `b` at either end of a diameter,
/// which a point of a circle is when it is on the shorter arc between them.
fn put_in_a_pole(
    points: &mut Vec<Point>,
    ends: [Point; 2],
    [ball, other]: [&Surface; 2],
    pole: Point,
    latitude: f64,
    how_much: HowMuchOfTheCrossing,
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
    if how_much == HowMuchOfTheCrossing::BetweenItsEnds
        && (every[gap] - pole).dot(every[gap + 1] - pole) >= 0.0
    {
        return None;
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
///
/// **Not a cone's apex, and that was measured.** The same rule for an apex
/// answers the two blocks less the cone whose steep face misses the apex by a
/// rounding, and takes two cones less the block that answered right without
/// it: the cone's face and the steep face then both draw the same sides across
/// the strip between the crossing's two arms just above the apex (§354).
fn the_pole_a_point_sits_within_a_rounding_of(stored: &Surface, at: Point) -> Option<f64> {
    let Surface::Sphere(sphere) = stored else {
        return None;
    };
    let pinch = stored
        .where_it_pinches()
        .expect("a ball pinches at its poles");
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

/// This face's surface, in the frame the face is cut into triangles in.
///
/// **A curved face is cut up on [`the_surface_this_face_is_meshed_on`]**, in
/// the frame its edges were walked in, for [`SolidView::mesh`]'s reason.
/// Everything that cuts a curved face into pieces before it is handed over reads
/// that frame itself, and it is this one.
///
/// **A flat face is cut up on its plane where the solid sits.** None of its
/// mesh points is made here: its corners are the model's, and the points round
/// a circle on it were walked by the curved face along that circle, or by the
/// plane the circle's run names. So the one thing decided here is which of those
/// points the triangles join, and the triangulator's Delaunay test decides it
/// on the numbers it is handed. Points of one circle are a tie for that test in
/// any four, and a tie goes whichever way the last bits push it. Moved back
/// into the frame a placed cap is stored in, each point was a rounded
/// subtraction away from the mesh point it stands for, and the tie went the
/// other way from the same cap read back out of its STEP file, which has no
/// placement: a bar of radius 3 moved by `(0.1, 0.2, 0.3)` had 20 of the 121
/// triangles on each cap join other points (§362). Cut up where the solid
/// sits, the test is decided on the mesh points themselves, and a cap is cut
/// the same way whichever frame it was stored in.
///
/// **A cap of a bar is no longer cut by that test at all**, but by the order
/// of its corners, because the same cap placed and unplaced still came out in
/// other triangles: 141 to 199 of a bar's 500 under twelve placements (§379.5).
/// Nor is a pipe's flat end, a ring between two circles, which moved 2 of its
/// triangles in nine of the same twelve (Q1109). Only a disc or a ring whose
/// rims the faces across cannot join two points of is cut that way
/// ([`BuildingAMesh::nothing_across_its_rim_joins_two_of_its_points`]);
/// every other flat face is still decided here, on these numbers.
///
/// A flat face at no placement was cut up there already. A plane is never
/// shifted ([`a_shift_that_loses_nothing`]), and moving one by nothing changes
/// no bit ([`Plane::transformed`](rcad_geom::Plane::transformed)).
fn the_surface_this_face_is_cut_up_on(face: FaceView<'_>) -> Surface {
    let located = face.surface();
    if located.as_plane().is_some() {
        return located;
    }
    the_surface_this_face_is_meshed_on(face)
}

/// The movement that carries a mesh point into the frame this face is cut into
/// triangles in: none for a flat face, which is cut up where the solid sits —
/// see [`the_surface_this_face_is_cut_up_on`].
fn the_frame_this_face_is_cut_up_in(face: FaceView<'_>) -> Isometry {
    if face.stored_surface().as_plane().is_some() {
        return Isometry::identity();
    }
    the_frame_this_face_is_meshed_in(face)
}

/// The movement that carries a point the triangulator made back out of the
/// frame this face is cut into triangles in: none for a flat face, for
/// [`the_frame_this_face_is_cut_up_in`]'s reason.
fn back_out_of_the_frame_this_face_is_cut_up_in(face: FaceView<'_>) -> Isometry {
    if face.stored_surface().as_plane().is_some() {
        return Isometry::identity();
    }
    back_out_of_the_frame_this_face_is_meshed_in(face)
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
///
/// **A piece of a parabola or a hyperbola a plane cuts a cone in is its two
/// ends as well**, and that is asked before any arithmetic on the plane's lean
/// is — see [`a_plane_cuts_this_cone_in_a_curve_that_runs_on_for_ever`]. So is
/// a piece of an ellipse that reaches millions of times further than the
/// piece's ends, which the walk places from those ends — see
/// [`a_plane_cuts_this_cone_in_an_ellipse_its_ends_are_walked_by`].
fn how_far_along_its_axis_this_face_runs(
    face: FaceView<'_>,
    axis: &Axis,
    radius: f64,
    slope: f64,
    along_each_edge: &HashMap<EdgeKey, SmallVec<[SurfaceAlongAnEdge; 2]>>,
) -> Option<f64> {
    let straight_on = axis.direction() * axis.direction().length().recip();
    let how_far_along = |point: Point| straight_on.dot(point - axis.origin()).abs();
    let ours = face.surface();
    let mut furthest: f64 = 0.0;
    for coedge in face.wires().flat_map(WireView::coedges) {
        let edge = coedge.edge();
        let its_ends = how_far_along(edge.start().point()).max(how_far_along(edge.end().point()));
        if *edge.stored_run() == EdgeRun::Straight {
            furthest = furthest.max(its_ends);
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
        // for exactly that reason, and only that reason — an edge with a
        // different face either side already has its second surface in
        // `others`, and `and` there can be any plane that picks out this edge's
        // own piece of a curve shared with other edges, not a bound on where
        // this one runs. A chord through a cone's apex is such a plane: it cuts
        // the cone in two rulings, which run on for ever, and folding that in
        // charged a face's own upper arc its two ends at `z = 4` though the arc
        // reaches `z = 8` between them, bounded correctly by the plane the two
        // faces already named (`a_plane_face_cut_by_a_cone_it_crosses_in_an_ellipse.rs`,
        // §406).
        if others.iter().all(|other| *other == ours)
            && let EdgeRun::PartOfWhereItCrosses { and, .. } = edge.run()
        {
            others.push(and);
        }
        let bounded = others
            .into_iter()
            .filter_map(|other| {
                if a_plane_cuts_this_cone_in_a_curve_that_runs_on_for_ever(&ours, &other) {
                    return Some(its_ends);
                }
                let whole = how_far_along_an_axis_a_crossing_with_this_surface_runs(
                    axis.origin(),
                    straight_on,
                    radius,
                    slope,
                    &other,
                );
                if a_plane_cuts_this_cone_in_an_ellipse_its_ends_are_walked_by(
                    &ours, &other, edge, whole, its_ends,
                ) {
                    return Some(its_ends);
                }
                whole
            })
            .fold(f64::INFINITY, f64::min);
        if !bounded.is_finite() {
            return None;
        }
        furthest = furthest.max(bounded);
    }
    Some(furthest)
}

/// Whether `other` is a plane that cuts `ours`, a cone, in a parabola, a
/// hyperbola or the rulings through its apex — every way of crossing a cone that
/// does not close — **decided exactly**, by
/// [`Cone::is_cut_in_a_closed_curve_by`](rcad_geom::Cone::is_cut_in_a_closed_curve_by),
/// the one place that question is asked and the sign [`rcad_geom::walk`] walks
/// the edge by.
///
/// **Then the edge's own two ends are how far up the axis it gets.** `walk`
/// writes a parabola as `vertex + across·s + bend·s²` and a hyperbola branch as
/// `centre + out·cosh σ + across·sinh σ`, with `across` square to the cone's
/// axis. So up the axis the one is `t_v + (bend·ŵ)·s²` and the other
/// `t_c + (out·ŵ)·cosh σ`: each has its least at the vertex and grows steadily
/// either way, and a curve that runs on for ever inside one nappe can only grow
/// away from the apex. An edge is one stretch of one branch between its two
/// corners, so no point of it is further up the axis than the further corner. A
/// ruling through the apex is straight and is its two ends for the plainer
/// reason.
///
/// **Why it is not left to the plane's lean, worked out in `f64`.** A plane
/// along a ruling has `down_the_axis − slope · sideways` exactly zero, and the
/// two rounded terms leave `5.55e-17` of it: the block below the frustum in
/// `a_plane_cut_along_a_cones_other_nappe.rs`, leaned `2` against a slope of a
/// half, made its parabola reach `5.7e16` up the axis that way, and every mesh of
/// a piece of cone it bounds was refused under a floor of two million.
fn a_plane_cuts_this_cone_in_a_curve_that_runs_on_for_ever(
    ours: &Surface,
    other: &Surface,
) -> bool {
    let (Surface::Cone(cone), Surface::Plane(flat)) = (ours, other) else {
        return false;
    };
    cone.is_cut_in_a_closed_curve_by(flat) == Some(false)
}

/// Whether `other` is a plane that cuts `ours`, a cone, in an ellipse reaching
/// so much further up the axis than this edge's ends that [`rcad_geom::walk`]
/// places the edge's points round the cone from its ends — so that the ends,
/// and not the ellipse, are how far up the axis the edge gets.
///
/// Only for an edge that is a piece of the crossing: one that goes right round
/// is the whole ellipse and is walked round it.
///
/// `whole` is how far the whole ellipse reaches, from
/// [`how_far_along_an_axis_a_crossing_with_this_surface_runs`], and `None` is a
/// lean so near a ruling's that the bottom of that division rounded to nothing,
/// which reaches for ever. The walk works the same number out the same way and
/// walks round the cone past
/// [`PointsAlongACrossing::WALKED_ROUND_THE_CONE_PAST`](rcad_geom::PointsAlongACrossing::WALKED_ROUND_THE_CONE_PAST)
/// times the ends; this charges by the ends only past **twice** that, so a
/// rounding between the frame the face is stored in and the one it is meshed
/// in cannot put the two on opposite sides of the line.
///
/// **And only for the arc that misses the ellipse's far end**, which the walk
/// is asked about through
/// [`PointsAlongACrossing::are_placed_round_the_cone_from_the_ends_of`](rcad_geom::PointsAlongACrossing::are_placed_round_the_cone_from_the_ends_of):
/// the edge's half-spaces pick one of the two arcs between its ends, and the
/// arc past the far end is walked round the ellipse and runs out to it. Charged
/// by its ends, a face on a cone of slope a half whose one curved edge runs from
/// two ends `1.25` up round the far side to `1.7e7` up had a floor of
/// `4.5e-11`, and the walk refused every deflection between that and `5.3e-7`:
/// `CannotPlaceAPointThatCloseToACrossing` under a certified policy, and under a
/// tolerant one a walk with nothing between the ends, which left the face
/// `ACurvedFaceGoesRightRoundWithNoSeam`
/// (`a_cone_arc_past_the_far_end_of_a_long_ellipse.rs`, §406.5). The walk
/// answers under a certified policy and says no where that does not settle it,
/// so an edge it cannot name is charged the whole ellipse: more than its walk
/// needs, which is a mesh refused that could have been cut, and never less.
///
/// The edge's run and ends are read where the solid sits, as `ours` and `other`
/// are, and not in the frame the face is meshed in; a half-space decides which
/// arc at each arc's middle, far from both ends, where a shift that loses
/// nothing moves no answer.
fn a_plane_cuts_this_cone_in_an_ellipse_its_ends_are_walked_by(
    ours: &Surface,
    other: &Surface,
    edge: EdgeView<'_>,
    whole: Option<f64>,
    its_ends: f64,
) -> bool {
    let (Surface::Cone(_), Surface::Plane(_), EdgeRun::PartOfWhereItCrosses { .. }) =
        (ours, other, edge.stored_run())
    else {
        return false;
    };
    let reaches_far_past_its_ends = whole.is_none_or(|whole| {
        whole > 2.0 * rcad_geom::PointsAlongACrossing::WALKED_ROUND_THE_CONE_PAST * its_ends
    });
    if !reaches_far_past_its_ends {
        return false;
    }
    let (from, to) = ends_in_the_stored_order(edge);
    let Ok(run) = NamedRun::new(
        edge.run(),
        [from, to].map(|end| PredicatePoint::from(end.point().coordinates())),
    ) else {
        return false;
    };
    rcad_geom::PointsAlongACrossing::are_placed_round_the_cone_from_the_ends_of(&run, [ours, other])
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
///
/// **"Exactly" holds of the real numbers and not of the rounded ones.** Each
/// `room` is a difference of rounded terms, and where it is zero in the reals
/// it can come out a rounding above zero and divide a length by `1e-17`. The
/// caller asks the plane-and-cone case exactly before it gets here
/// ([`a_plane_cuts_this_cone_in_a_curve_that_runs_on_for_ever`]); the other
/// arms have no shape known to reach theirs.
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
                .walk_the_edges_this_face_can_place(solid, face, &policy, &mut log)
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
            .cut_up(wall, solid, &mut log)
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

    /// An arc of the circle `x = 0` cuts a unit ball in, from `+y` on the
    /// equator down through the south pole to `−y`, gets the south pole among
    /// its points and not the north one.
    ///
    /// **The north pole is on the circle and not on the arc**, and the walked
    /// points still change side of the plane square to the crossing there, at
    /// the bottom, where the arc passes the south pole. Read as all of the
    /// circle, as a closed crossing is, the north pole went in at the bottom
    /// between the two walked points, so the walk would climb to it and back
    /// in one step (Q1270).
    #[test]
    fn an_open_arc_gets_the_pole_it_passes_and_not_the_one_beyond_its_ends() {
        let ball = Surface::Sphere(
            Sphere::new(Point::origin(), 1.0).expect("a ball of radius one at the origin"),
        );
        let wall = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
                .expect("the plane x = 0"),
        );
        let half = core::f64::consts::FRAC_1_SQRT_2;
        let walked = vec![Point::new(0.0, half, -half), Point::new(0.0, -half, -half)];
        let ends = [Point::new(0.0, 1.0, 0.0), Point::new(0.0, -1.0, 0.0)];

        let mut points = walked.clone();
        let moved = put_in_the_poles_a_crossing_passes(
            &mut points,
            ends,
            &[ball, wall],
            HowMuchOfTheCrossing::BetweenItsEnds,
        );

        assert_eq!(
            moved, 0.0,
            "both poles are certified on the ball and the wall"
        );
        assert_eq!(
            points,
            vec![walked[0], Point::new(0.0, 0.0, -1.0), walked[1]],
            "the south pole between the two walked points, and nothing else"
        );
    }

    /// Of the two bands of a ring between two circles a quarter turn apart, the
    /// short one meshes to a quarter of the ring and the long one to three
    /// quarters, each at three deflections.
    ///
    /// **Built without the sweep, because the sweep refuses the long one.**
    /// `Model::validate` reads which side of a cap's plane the wall keeps off
    /// the other circle's corner, and for a band wider than half the ring that
    /// corner is on the other side (§243.3, Q103). [`SolidView::mesh`] is public
    /// in its own right, so a model read from a file reaches the mesher without
    /// the sweep. Before §343 the long band's wall was meshed as the short
    /// band's, between its own two caps facing away from it, and at the
    /// coarsest deflection the solid measured `13.07` where Pappus gives
    /// `6π² = 59.22`, allowed `4.07`. Nothing in that mesh was folded, so
    /// nothing refused it.
    #[test]
    fn a_band_between_two_circles_meshes_the_way_round_its_circles_are_walked() {
        let (ring, tube) = (4.0, 1.0);
        for (quarters, the_way_round) in [(1.0, Orientation::Forward), (3.0, Orientation::Reversed)]
        {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let band = a_band_between_two_circles(&mut model, ring, tube, the_way_round);
            let pappus = quarters
                * core::f64::consts::FRAC_PI_2
                * ring
                * core::f64::consts::PI
                * tube
                * tube;
            for length in [0.1, 0.03, 0.01] {
                let measured = model
                    .solid(band)
                    .expect("this model made the key")
                    .measure(
                        Deflection::of(length).expect("a length"),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .expect("a band between two circles measures");
                assert!(
                    (measured.volume() - pappus).abs() <= measured.volume_error(),
                    "{quarters} quarters at {length}: {} where Pappus gives {pappus}, allowed {}",
                    measured.volume(),
                    measured.volume_error()
                );
            }
        }
    }

    /// The band of the ring about `+z` between the circles the planes `y = 0`
    /// and `x = 0` cut round `(ring, 0, 0)` and `(0, ring, 0)`, capped flat on
    /// both planes.
    ///
    /// `Forward` is the quarter from the first circle anticlockwise to the
    /// second, built as `tests/rings/mod.rs` builds a bend with its seam left
    /// out: caps facing `(0, −1, 0)` and `(−1, 0, 0)`, the wall used forwards.
    /// `Reversed` is the rest of the ring between the same two circles: every
    /// closed-edge coedge and both cap normals are turned over, and the shell
    /// is the same.
    fn a_band_between_two_circles(
        model: &mut Model,
        ring: f64,
        tube: f64,
        the_way_round: Orientation,
    ) -> crate::SolidKey {
        let inner = ring - tube;
        let facing = the_way_round.sign();
        let first_corner = model.add_vertex(Point::new(inner, 0.0, 0.0));
        let second_corner = model.add_vertex(Point::new(0.0, inner, 0.0));
        let first_circle = model.add_closed_edge(first_corner);
        let second_circle = model.add_closed_edge(second_corner);

        let cap = |model: &mut Model, facing: Vector, circle, turn| {
            let plane = Plane::from_point_and_normal(Point::origin(), facing)
                .expect("a coordinate direction is a unit normal");
            let round_it = model.add_coedge(circle, turn);
            let wire = model.add_wire(smallvec::smallvec![round_it]);
            model.add_face(Surface::Plane(plane), smallvec::smallvec![wire])
        };
        let first_cap = cap(
            model,
            Vector::new(0.0, -facing, 0.0),
            first_circle,
            the_way_round,
        );
        let second_cap = cap(
            model,
            Vector::new(-facing, 0.0, 0.0),
            second_circle,
            the_way_round.reversed(),
        );
        let wires = [
            (first_circle, the_way_round.reversed()),
            (second_circle, the_way_round),
        ]
        .map(|(circle, turn)| {
            let round_it = model.add_coedge(circle, turn);
            model.add_wire(smallvec::smallvec![round_it])
        });
        let wall = model.add_face(
            Surface::Torus(rcad_geom::Torus::new(Axis::z(), ring, tube).expect("a ring")),
            wires.into_iter().collect(),
        );

        let shell = model.add_shell(
            [
                (first_cap, Orientation::Forward),
                (second_cap, Orientation::Forward),
                (wall, Orientation::Forward),
            ]
            .into_iter()
            .map(|(face, orient)| crate::model::FaceUse {
                face,
                loc: Isometry::identity(),
                orient,
            })
            .collect(),
        );
        let solid = model.add_solid(smallvec::smallvec![crate::model::ShellUse {
            shell,
            loc: Isometry::identity(),
            orient: Orientation::Forward,
        }]);
        model.key_for_solid(solid)
    }

    /// A bar's wall with a window in it measures, whether a quarter cut runs
    /// through the window or the window lies between two cuts.
    ///
    /// The wall is a ring with an island: its two cap circles go right round
    /// and the window does not. A thin bar through a fat one off its axis
    /// leaves the fat bar's wall this shape, with two windows, and its union
    /// was refused `AWireWillNotLieFlatOnItsFace` at the check after the
    /// arrangement (Q1494). The window from `(4, −3)` to `(4, 3)` is centred
    /// on the circles' corner at `+x`, so the quarter cut from it runs through
    /// the window at every deflection; the one from `(4, 3)` to `(3, 4)` lies
    /// between the cuts at `+x` and `+y`.
    ///
    /// **Built without the sweep**, as [`a_band_between_two_circles`] is: no
    /// fuse here leaves a pipe wall with a window but the thin bar through the
    /// fat one, whose own walk is refused for other reasons, and
    /// [`SolidView::mesh`] is public in its own right.
    #[test]
    fn a_bar_with_a_wedge_cut_out_through_a_window_in_its_wall_measures() {
        for (sides, through_a_cut) in [
            ([[4.0, -3.0], [4.0, 3.0]], true),
            ([[4.0, 3.0], [3.0, 4.0]], false),
        ] {
            for window_first in [false, true] {
                let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                let bar =
                    a_bar_with_a_wedge_cut_out_through_its_wall(&mut model, sides, window_first);
                let [from, to] = sides;
                let wedge = 25.0 * (to[1].atan2(to[0]) - from[1].atan2(from[0]));
                let wanted = 250.0 * core::f64::consts::PI - wedge;
                for length in [1.0, 0.1, 0.01] {
                    let case = format!(
                        "through a cut {through_a_cut}, window first {window_first}, at {length}"
                    );
                    let measured = model
                        .solid(bar)
                        .expect("this model made the key")
                        .measure(
                            Deflection::of(length).expect("a length"),
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        )
                        .unwrap_or_else(|why| panic!("{case}: refused {why:?}"));
                    assert!(
                        (measured.volume() - wanted).abs() <= measured.volume_error(),
                        "{case}: {} where {wanted} is wanted, allowed {}",
                        measured.volume(),
                        measured.volume_error()
                    );
                }
            }
        }
    }

    /// A bar of radius 5 about `z` from `z = 0` to `z = 10`, its cap circles
    /// starting at `+x`, less the wedge from its axis out to the points
    /// `sides` of the circle `x² + y² = 25`, between `z = 4` and `z = 6`.
    ///
    /// The wedge's two flat sides meet along the axis, its floor and its
    /// ceiling are flat, and where it leaves the wall is a window of two
    /// straight lines up the wall and two arcs round it. The first of `sides`
    /// is the one less far round from `+x`, and the two are less than half a
    /// turn apart. The wall's wires are its bottom circle, its top one and the
    /// window, or with `window_first` the window, the top circle and the
    /// bottom one: neither is the outer one, and the mesher puts its cuts
    /// round the circle it is handed first.
    fn a_bar_with_a_wedge_cut_out_through_its_wall(
        model: &mut Model,
        [from, to]: [[f64; 2]; 2],
        window_first: bool,
    ) -> crate::SolidKey {
        use Orientation::{Forward, Reversed};
        let flat = |at: Point, facing: Vector| {
            Plane::from_point_and_normal(at, facing).expect("a normal with a length")
        };
        let bottom_corner = model.add_vertex(Point::new(5.0, 0.0, 0.0));
        let top_corner = model.add_vertex(Point::new(5.0, 0.0, 10.0));
        let bottom_circle = model.add_closed_edge(bottom_corner);
        let top_circle = model.add_closed_edge(top_corner);

        let [low_from, low_to, high_to, high_from] =
            [(from, 4.0), (to, 4.0), (to, 6.0), (from, 6.0)]
                .map(|([x, y], z)| model.add_vertex(Point::new(x, y, z)));
        let [low_middle, high_middle] =
            [4.0, 6.0].map(|z| model.add_vertex(Point::new(0.0, 0.0, z)));
        let half_way = Vector::new(from[0] + to[0], from[1] + to[1], 0.0);
        let arc = |model: &mut Model, start, end, z: f64| {
            let picker = rcad_geom::HalfSpace::through_a_point_facing(Point::origin(), half_way)
                .expect("a direction");
            model.add_edge_running(
                start,
                end,
                EdgeRun::PartOfWhereItCrosses {
                    and: Surface::Plane(flat(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, 1.0))),
                    picked_by: SmallVec::from_buf([rcad_geom::Picker::from(picker)]),
                },
            )
        };
        let low_arc = arc(model, low_from, low_to, 4.0);
        let high_arc = arc(model, high_from, high_to, 6.0);
        let up_from = model.add_edge(low_from, high_from);
        let up_to = model.add_edge(low_to, high_to);
        let low_out_to_from = model.add_edge(low_middle, low_from);
        let low_out_to_to = model.add_edge(low_middle, low_to);
        let high_out_to_from = model.add_edge(high_middle, high_from);
        let high_out_to_to = model.add_edge(high_middle, high_to);
        let axis = model.add_edge(low_middle, high_middle);

        let face = |model: &mut Model, surface, wires: &[&[(_, Orientation)]]| {
            let wires = wires
                .iter()
                .map(|uses| {
                    let coedges = uses
                        .iter()
                        .map(|&(edge, turn)| model.add_coedge(edge, turn))
                        .collect();
                    model.add_wire(coedges)
                })
                .collect();
            model.add_face(surface, wires)
        };
        let bottom = face(
            model,
            Surface::Plane(flat(Point::origin(), Vector::new(0.0, 0.0, -1.0))),
            &[&[(bottom_circle, Reversed)]],
        );
        let top = face(
            model,
            Surface::Plane(flat(Point::new(0.0, 0.0, 10.0), Vector::new(0.0, 0.0, 1.0))),
            &[&[(top_circle, Forward)]],
        );
        let bottom_rim: &[_] = &[(bottom_circle, Forward)];
        let top_rim: &[_] = &[(top_circle, Reversed)];
        let window: &[_] = &[
            (high_arc, Forward),
            (up_to, Reversed),
            (low_arc, Reversed),
            (up_from, Forward),
        ];
        let wall = face(
            model,
            Surface::Cylinder(Cylinder::new(Axis::z(), 5.0).expect("a pipe")),
            if window_first {
                &[window, top_rim, bottom_rim]
            } else {
                &[bottom_rim, top_rim, window]
            },
        );
        let floor = face(
            model,
            Surface::Plane(flat(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0))),
            &[&[
                (low_out_to_from, Forward),
                (low_arc, Forward),
                (low_out_to_to, Reversed),
            ]],
        );
        let ceiling = face(
            model,
            Surface::Plane(flat(Point::new(0.0, 0.0, 6.0), Vector::new(0.0, 0.0, -1.0))),
            &[&[
                (high_out_to_to, Forward),
                (high_arc, Reversed),
                (high_out_to_from, Reversed),
            ]],
        );
        let side_to = face(
            model,
            Surface::Plane(flat(Point::origin(), Vector::new(to[1], -to[0], 0.0))),
            &[&[
                (low_out_to_to, Forward),
                (up_to, Forward),
                (high_out_to_to, Reversed),
                (axis, Reversed),
            ]],
        );
        let side_from = face(
            model,
            Surface::Plane(flat(Point::origin(), Vector::new(-from[1], from[0], 0.0))),
            &[&[
                (axis, Forward),
                (high_out_to_from, Forward),
                (up_from, Reversed),
                (low_out_to_from, Reversed),
            ]],
        );

        let shell = model.add_shell(
            [bottom, top, wall, floor, ceiling, side_to, side_from]
                .into_iter()
                .map(|face| crate::model::FaceUse {
                    face,
                    loc: Isometry::identity(),
                    orient: Orientation::Forward,
                })
                .collect(),
        );
        let solid = model.add_solid(smallvec::smallvec![crate::model::ShellUse {
            shell,
            loc: Isometry::identity(),
            orient: Orientation::Forward,
        }]);
        model.key_for_solid(solid)
    }
}
