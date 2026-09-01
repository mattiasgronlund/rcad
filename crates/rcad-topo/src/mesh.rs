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

use rcad_geom::{Deflection, Isometry, NamedRun, POINT_ALONG_A_CROSSING_ROUNDOFF, Point, Surface};
use rcad_mesh::{CannotTriangulate, Loop, TrimmedDomain, triangulate};
use rcad_predicates::{
    Approximation, Cause, Decision, Level, Measures, Point as PredicatePoint, Policy,
    ProvenanceLog, Record,
};
use smallvec::SmallVec;

use crate::view::{EdgeView, FaceView, VertexView, WireStepView, WireView};
use crate::{EdgeKey, Error, FaceKey, Result, SolidView, VertexKey};

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
    /// a closer fit than a point can be placed to, and
    /// [`Error::MoreMeshPointsThanASolidWillHold`] when it asks for more points
    /// than one mesh holds. [`Error::CannotMeshAFace`] when the triangulator
    /// refuses a face, carrying what it said. [`Error::Undecidable`] when a
    /// predicate inside a triangulation could not settle a sign.
    /// [`Error::AWireWillNotLieFlatOnItsFace`],
    /// [`Error::CurvedFaceWithAHoleGoesTooFarRound`] and
    /// [`Error::AnEdgeWithoutTwoFacesAlongItCannotBeWalked`] for a solid that
    /// arrived from a file with a shape no builder here makes. All three are
    /// refusals rather than asserts precisely because
    /// [`Model::validate`](crate::Model::validate) meshes what it is given, and
    /// a sweep over untrusted bytes has to hand back a refusal (§16.11, §4.1).
    /// Whatever [`rcad_geom::walk`] makes of an edge it cannot follow, which
    /// includes a seam whose half-spaces pick both halves of its meridian or
    /// neither.
    ///
    /// # Panics
    ///
    /// If a wire fans into fewer than three corners in the surface's own two
    /// coordinates. Nothing does: a wire has three steps or it closes on one
    /// edge, and a closed edge on a curved face is walked four ways round
    /// whatever the deflection, because the evaluator will not step past a
    /// quarter turn either.
    pub fn mesh(
        self,
        deflection: Deflection,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Mesh> {
        let deflection = self.room_a_mesh_has_to_work_in(deflection, policy)?;
        let mut building = BuildingAMesh::new(deflection, self.surfaces_along_each_edge());
        for face in self.faces() {
            building.walk_the_edges_this_face_can_place(face, policy, log)?;
        }
        for face in self.faces() {
            building.cut_up(face, log)?;
        }
        Ok(building.finish())
    }

    /// The surfaces of the faces either side of each edge, already moved into
    /// place.
    ///
    /// One entry per *use*, so a seam — one face using one edge twice — comes
    /// back as that face's surface twice, which is exactly what
    /// [`rcad_geom::walk`] wants to be told about a seam.
    fn surfaces_along_each_edge(self) -> HashMap<EdgeKey, SmallVec<[Surface; 2]>> {
        let mut along: HashMap<EdgeKey, SmallVec<[Surface; 2]>> = HashMap::new();
        for face in self.faces() {
            let surface = face.surface();
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
    fn room_a_mesh_has_to_work_in(self, asked: Deflection, policy: &Policy) -> Result<Deflection> {
        let closest = self
            .faces()
            .map(how_far_a_mesh_point_of_this_face_rounds)
            .fold(0.0, f64::max);
        if asked.length() >= closest {
            return Ok(asked);
        }
        if !policy.wobble_is_acceptable(closest) {
            return Err(Error::DeflectionBelowWhatAMeshCanReach {
                asked: asked.length(),
                closest,
            });
        }
        Ok(Deflection::of(closest * ROOM_ABOVE_THE_FLOOR)?)
    }
}

/// How far above the floor a tolerant mesh is actually cut.
///
/// Exactly at the floor there is no room left to put a point: the longest chord
/// a triangle may have shrinks to nothing and the refinement has nothing it can
/// do. Twice the floor leaves a chord of about `√(3 · radius · floor)`, which is
/// small and is not zero.
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
/// Every cut that happens at all makes both pieces narrower, and four pieces is
/// the answer for every face this kernel builds. Reaching this means a piece is
/// being cut without getting any smaller, which is a bug here rather than a
/// shape somebody asked for.
const MOST_PIECES_A_FACE_IS_CUT_INTO: usize = 16;

/// A mesh part way through being built.
struct BuildingAMesh {
    deflection: Deflection,
    points: Vec<Point>,
    at_vertex: HashMap<VertexKey, u32>,
    along_edge: HashMap<EdgeKey, Vec<u32>>,
    triangles: Vec<MeshTriangle>,
    worst: f64,
    along_each_edge: HashMap<EdgeKey, SmallVec<[Surface; 2]>>,
}

impl BuildingAMesh {
    fn new(
        deflection: Deflection,
        along_each_edge: HashMap<EdgeKey, SmallVec<[Surface; 2]>>,
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
        let stored = face.surface().transformed(&face.location().inverse());
        if stored.as_plane().is_some() {
            return Ok(());
        }
        let room = self.deflection.length() - how_far_placing_this_face_rounds(face);
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if self.along_edge.contains_key(&edge.key()) {
                    continue;
                }
                let walked = self.walk(edge, face, &stored, room, policy, log)?;
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
    /// pipe. The way this face's own longitude increases is what says so, and it
    /// is a choice of direction rather than a second opinion about position: a
    /// reversed list is the same points.
    fn walk(
        &mut self,
        edge: EdgeView<'_>,
        face: FaceView<'_>,
        stored: &Surface,
        room: f64,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Vec<u32>> {
        let back = face.location().inverse();
        let (from, to) = ends_in_the_stored_order(edge);
        let run = NamedRun::new(
            edge.run().clone(),
            [from, to]
                .map(|end| PredicatePoint::from(back.apply_to_point(end.point()).coordinates())),
        );
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
        if edge.is_closed()
            && how_far_round_this_face_it_goes(stored, back.apply_to_point(from.point()), &points)
                < 0.0
        {
            points.reverse();
        }

        self.worst = self
            .worst
            .max(along_it.deviation().magnitude() + how_far_placing_this_face_rounds(face));
        let there = face.location();
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
        let stored = face.surface().transformed(&face.location().inverse());
        let room = self.deflection.length() - how_far_placing_this_face_rounds(face);

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
        let widest = whole_turn / 4.0;
        if how_far_round(&walked[0]) <= widest {
            return Ok(vec![walked]);
        }
        // Cutting a face with a hole in it would have to say which piece the
        // hole ends up in, and nothing does. No *builder* makes such a face —
        // `add_round_bar` and `add_ball` each give a curved face one wire,
        // `add_extrusion` makes planes only, and the fuse refuses a curved solid
        // outright — but `Model` is `Deserialize` and `validate` is the
        // untrusted-import boundary (§16.11), so one arrives from a file and
        // this is a refusal rather than an assert.
        if walked.len() != 1 {
            return Err(Error::CurvedFaceWithAHoleGoesTooFarRound { face: face.key() });
        }

        let mut pieces = vec![walked.into_iter().next().expect("just checked")];
        let mut done: Vec<Vec<WalkedLoop>> = Vec::new();
        while let Some(piece) = pieces.pop() {
            assert!(
                pieces.len() + done.len() < MOST_PIECES_A_FACE_IS_CUT_INTO,
                "cutting this face up is not making the pieces any smaller"
            );
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
        let steps = how_many_steps(
            stored,
            arriving[0] - leaving[0],
            arriving[1] - leaving[1],
            room,
        );
        if self.points.len() + steps > MOST_POINTS_A_MESH_WILL_HOLD {
            return Err(Error::MoreMeshPointsThanASolidWillHold {
                most: MOST_POINTS_A_MESH_WILL_HOLD,
            });
        }

        let there = face.location();
        let mut along_the_cut = WalkedLoop {
            corners: Vec::with_capacity(steps - 1),
            points: Vec::with_capacity(steps - 1),
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
            loops.push(Loop::through(one.corners).expect(
                "a wire of a valid solid fans into at least three corners made \
                 of finite numbers",
            ));
        }
        let (outer, holes) = loops
            .split_first()
            .expect("a face of a valid solid has at least one wire");

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
        let there = face.location();
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
        self.worst = self
            .worst
            .max(cut.deviation().magnitude() + how_far_placing_this_face_rounds(face));
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
    fn walk_a_wire_round(
        &mut self,
        wire: WireView<'_>,
        face: FaceView<'_>,
        stored: &Surface,
    ) -> Option<WalkedLoop> {
        let back = face.location().inverse();
        let poles = poles_of(wire);
        let mut walk = WalkingAWire::new(how_far_the_longitude_wraps(stored));

        for step in wire.steps() {
            match step {
                WireStepView::Along(coedge) => {
                    let start = coedge.start();
                    let at = self.remember_the_corner(start);
                    let where_it_is = back.apply_to_point(start.point());
                    walk.arrive(
                        at,
                        stored.coordinates_of(where_it_is),
                        poles.contains(&start.key()),
                    );

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
}

impl WalkedLoop {
    /// The stretch from corner `from` round to corner `to`, both ends kept.
    fn between(&self, from: usize, to: usize) -> WalkedLoop {
        let mut taken = WalkedLoop {
            corners: Vec::new(),
            points: Vec::new(),
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

    /// The same corners, walked backwards.
    fn the_other_way(&self) -> WalkedLoop {
        WalkedLoop {
            corners: self.corners.iter().rev().copied().collect(),
            points: self.points.iter().rev().copied().collect(),
        }
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
    longitude: f64,
    started: bool,
    will_not_lie_flat: bool,
    pole_side: f64,
    corners: Vec<[f64; 2]>,
    points: Vec<u32>,
}

impl WalkingAWire {
    fn new(wraps: Option<f64>) -> Self {
        Self {
            wraps,
            longitude: 0.0,
            started: false,
            will_not_lie_flat: false,
            pole_side: 1.0,
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
        self.longitude += self.pole_side * whole;
        self.pole_side = -self.pole_side;
    }

    /// The loop, or `None` if it will not lie flat: the walk stood still at a
    /// pole on a surface that does not pinch to one, or it came back to a
    /// different longitude from the one it set off at.
    ///
    /// **A refusal and not an assert, and the difference is `Deserialize`.** A
    /// wire of a face this kernel builds does neither: only a sphere here has a
    /// pole, no step goes more than a quarter turn (see [`how_many_steps`]), and
    /// a face's two pole sides are crossed opposite ways. A face that arrived
    /// from a *file* has neither promise behind it.
    fn finish(self) -> Option<WalkedLoop> {
        if self.will_not_lie_flat {
            return None;
        }
        if let (Some(whole), Some(first)) = (self.wraps, self.corners.first())
            && (self.longitude - first[0]).abs() >= whole / 2.0
        {
            return None;
        }
        Some(WalkedLoop {
            corners: self.corners,
            points: self.points,
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

/// The corners of `wire` where the surface pinches to a point.
fn poles_of(wire: WireView<'_>) -> HashSet<VertexKey> {
    wire.steps()
        .filter_map(|step| match step {
            WireStepView::Pole(vertex) => Some(vertex.key()),
            WireStepView::Along(_) => None,
        })
        .collect()
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
    along_each_edge: &HashMap<EdgeKey, SmallVec<[Surface; 2]>>,
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
    Ok([first.transformed(back), second.transformed(back)])
}

/// How far round this face's own longitude a closed edge's points go, following
/// them from the corner it starts at.
///
/// A whole turn one way or the other for a cap circle, and the **sign** is the
/// whole of what this is read for. Summed the short way round from step to step,
/// which is why no step of a walk is allowed past a quarter turn: past a half
/// one there is no telling a step from one the other way.
///
/// Zero for a closed edge that does not go round the face at all — a small
/// circle cut out of a ball by a plane that misses its axis — and then the
/// evaluator's own direction is left alone.
///
/// **That is the right answer for the wire this is read for and not for every
/// wire**, and the difference is worth naming rather than glossing. What the
/// sign settles is whether a face's *outer* wire comes back to the longitude it
/// set off from, and an edge that goes nowhere round the face cannot make it
/// fail that whichever way it runs. A closed edge that is a **hole** in a curved
/// face is a different question — there the direction is what makes it read as a
/// hole rather than as a second outline — and nothing here answers it. No
/// builder makes such a face; one arriving from a file would need a curved face
/// with a closed-edge hole in it, which is M4's ball with a pipe through it.
fn how_far_round_this_face_it_goes(stored: &Surface, from: Point, points: &[Point]) -> f64 {
    let Some(whole_turn) = how_far_the_longitude_wraps(stored) else {
        return 0.0;
    };
    let mut at = stored.coordinates_of(from).0;
    let mut round = 0.0;
    for &point in points.iter().chain(core::iter::once(&from)) {
        let next = stored.coordinates_of(point).0;
        let step = the_short_way_round(next - at, whole_turn);
        round += step;
        at = next;
    }
    round
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
        Surface::Sphere(_) | Surface::Cylinder(_) => Some(core::f64::consts::TAU),
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
/// # Panics
///
/// On a surface this has not been taught — see [`how_far_the_longitude_wraps`].
fn how_many_steps(stored: &Surface, round: f64, up: f64, room: f64) -> usize {
    let (radius, turns_over) = match stored {
        Surface::Plane(_) => return 1,
        // A ball bends both ways, so both count towards the turn as well.
        Surface::Sphere(ball) => (ball.radius(), round.hypot(up)),
        // A pipe is flat along its axis, so a piece stretched up one bends by
        // nothing at all and only the turn counts.
        Surface::Cylinder(pipe) => (pipe.radius(), round.abs()),
        other => panic!("this crate cannot walk along {other:?}"),
    };
    let longest_chord = (3.0 * radius * room * WHAT_THE_BOUNDARY_TAKES_OF_THE_BUDGET).sqrt();
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
/// **Two chains in two frames, and keeping them apart is §19 again.** The
/// evaluator follows a crossing in the frame the face is **stored** in and
/// publishes [`POINT_ALONG_A_CROSSING_ROUNDOFF`] of how far out the surfaces
/// reach *there*; the movement that carries the point to where the solid sits
/// rounds against how far out the face reaches *there*. A ball stored at the
/// origin and placed a billion units away pays the second and not the first, and
/// charging it both against the billion would put the floor under a mesh fifty
/// times higher than the arithmetic asks for.
///
/// The evaluator's share is divided by [`WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO`],
/// because that is the fraction of the deflection it is handed and it refuses a
/// deflection it cannot place a point inside.
fn how_far_a_mesh_point_of_this_face_rounds(face: FaceView<'_>) -> f64 {
    POINT_ALONG_A_CROSSING_ROUNDOFF / WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO
        * how_far_the_points_of_reach(face, &face.location().inverse())
        + how_far_placing_this_face_rounds(face)
}

/// How far moving this face's points from the frame it is stored in to where the
/// solid sits can shift one of them.
///
/// Zero for a flat face, and exactly zero: a plane's mesh is drawn through
/// corners the model already held and nothing here made any of them. The points a
/// bar's *cap* is drawn through were made on the wall beside it, and the wall
/// charges for them — [`Mesh::deviation`] is the worst over every face, so the
/// number that comes out still covers them.
fn how_far_placing_this_face_rounds(face: FaceView<'_>) -> f64 {
    Isometry::POINT_ROUNDOFF * how_far_the_points_of_reach(face, &Isometry::identity())
}

/// The largest coordinate any point of this face can have, seen `through` a
/// movement.
///
/// Its corners, plus the radius of the surface it sits on — because a face
/// bulges away from its own corners, which is the whole of what
/// [`SolidView::box_around_its_corners`] stopped being able to say (§25.9).
/// Zero for a flat face, for the reason above.
///
/// # Panics
///
/// On a surface this has not been taught — see [`how_far_the_longitude_wraps`].
fn how_far_the_points_of_reach(face: FaceView<'_>, through: &Isometry) -> f64 {
    let radius = match face.surface() {
        Surface::Plane(_) => return 0.0,
        Surface::Sphere(ball) => ball.radius(),
        Surface::Cylinder(pipe) => pipe.radius(),
        other => panic!("this crate cannot say how far {other:?} reaches"),
    };
    let corners = face
        .wires()
        .flat_map(WireView::points)
        .map(|corner| {
            through
                .apply_to_point(corner)
                .largest_coordinate_magnitude()
        })
        .fold(0.0, f64::max);
    corners + radius
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
    use rcad_geom::{EdgeRun, Orientation, Plane, Scale, Vector};
    use rcad_predicates::{Decision, Interval};

    use super::*;
    use crate::Model;
    use crate::validate::side_of_surface;

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
                edge.run().clone(),
                [from, to].map(|end| PredicatePoint::from(end.point().coordinates())),
            );
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
                edge.run().clone(),
                [from, to].map(|end| PredicatePoint::from(end.point().coordinates())),
            );
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
                _ => vec![along[0], along[1]],
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
}
