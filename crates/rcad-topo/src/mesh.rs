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

use std::collections::{HashMap, HashSet};

use rcad_geom::{Deflection, Isometry, Point, Surface};
use rcad_mesh::{CannotTriangulate, Loop, TrimmedDomain, triangulate};
use rcad_predicates::{
    Approximation, Cause, Decision, Level, Measures, Point as PredicatePoint, Policy,
    ProvenanceLog, Record,
};

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
    /// This does, all of it. The triangulator may not put a point on a loop it
    /// was handed, because it is shown one face at a time and cannot know a loop
    /// is shared (§25.1). So a curved edge is walked into intermediate points
    /// **here**, once, and the identical [`Point`] values go to both faces along
    /// it — which is the whole of §23.4's promise, kept by construction rather
    /// than by rounding two answers into agreement.
    ///
    /// # Errors
    ///
    /// [`Error::DeflectionBelowWhatAMeshCanReach`] when the deflection asks for
    /// a closer fit than a point can be placed to, and
    /// [`Error::MoreMeshPointsThanASolidWillHold`] when it asks for more points
    /// than one mesh holds. [`Error::CannotMeshAFace`] when the triangulator
    /// refuses a face, carrying what it said. [`Error::Undecidable`] when a
    /// predicate inside a triangulation could not settle a sign.
    ///
    /// # Panics
    ///
    /// If a wire of this solid does not come back to the longitude it set off
    /// from, or fans into fewer than three corners. Both mean a model
    /// [`Model::validate`](crate::Model::validate) would have rejected, and a
    /// corrupt mesh is worse than a crash.
    pub fn mesh(
        self,
        deflection: Deflection,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Mesh> {
        let deflection = self.room_a_mesh_has_to_work_in(deflection, policy)?;
        let mut building = BuildingAMesh::new(deflection);
        for face in self.faces() {
            building.walk_the_edges_this_face_can_place(face)?;
        }
        for face in self.faces() {
            building.cut_up(face, log)?;
        }
        Ok(building.finish())
    }

    /// The deflection to work to, which is the one asked for unless that is
    /// under the floor.
    ///
    /// **There is a floor, and it depends on where the solid sits and not only
    /// on how big it is.** A point on a curved surface comes out of an evaluator
    /// that rounds against how far out that surface reaches, and is then carried
    /// into place by the solid's own placement, which rounds again against the
    /// same thing. So the same ball a kilometre from the origin cannot be meshed
    /// as finely as one at the origin, and asking for finer is asking for
    /// arithmetic `f64` does not have.
    ///
    /// This is the one thing the policy decides, and it is the one §23.5 says it
    /// should: a certified caller is refused, and a tolerant one — which has
    /// already said how far out of place it will let a point be — gets a mesh as
    /// fine as the arithmetic reaches, with [`Mesh::deviation`] saying what that
    /// came to.
    fn room_a_mesh_has_to_work_in(self, asked: Deflection, policy: &Policy) -> Result<Deflection> {
        let closest = self
            .faces()
            .map(|face| HOW_FAR_A_MESH_POINT_ROUNDS * how_far_the_points_of_reach(face))
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

/// How many roundings deep a mesh point sits, as a multiple of how far out the
/// surface it stands on reaches.
///
/// Two chains, and only the second is this layer's own. Eight `f64::EPSILON` for
/// the evaluator that puts a point on the surface — `rcad-mesh` charges the same
/// eight for the same thing, so this is a bound on that and not a second one —
/// and [`Isometry::POINT_ROUNDOFF`] for the movement that carries the point out
/// of the frame the face is stored in and into where the solid sits.
const HOW_FAR_A_MESH_POINT_ROUNDS: f64 = 8.0 * f64::EPSILON + Isometry::POINT_ROUNDOFF;

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
}

impl BuildingAMesh {
    fn new(deflection: Deflection) -> Self {
        Self {
            deflection,
            points: Vec::new(),
            at_vertex: HashMap::new(),
            along_edge: HashMap::new(),
            triangles: Vec::new(),
            worst: 0.0,
        }
    }

    /// Walk every edge round `face` that has not been walked yet, if this face
    /// is the kind that can say where a curve goes.
    ///
    /// **A plane cannot.** A round bar's cap circle has a flat cap on one side
    /// and the wall on the other; walked in the cap's coordinates it would come
    /// out a straight chord, because a plane has nothing to say about where a
    /// circle drawn on it bends. So a boundary is walked on the curved surface
    /// that bends round it, and the flat face along the same edge reads the
    /// points that came out — the same ones, bit for bit.
    fn walk_the_edges_this_face_can_place(&mut self, face: FaceView<'_>) -> Result<()> {
        let stored = face.surface().transformed(&face.location().inverse());
        if stored.as_plane().is_some() {
            return Ok(());
        }
        let room = self.deflection.length() - how_far_placing_this_face_rounds(face);
        for wire in face.wires() {
            let poles = poles_of(wire);
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if self.along_edge.contains_key(&edge.key()) {
                    continue;
                }
                let walked = self.walk(edge, face, &stored, &poles, room)?;
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
    /// **A straight line in the surface's own two coordinates.** That is exactly
    /// right for the three kinds of edge this kernel builds — a pipe's ruling is
    /// a line of constant longitude, a cap circle is a line of constant height a
    /// whole turn long, and a ball's meridian seam is a line of constant
    /// longitude. It is not right in general: a circle cut out of a ball by a
    /// plane that misses the centre is not a straight line in these coordinates,
    /// and nothing builds one yet.
    fn walk(
        &mut self,
        edge: EdgeView<'_>,
        face: FaceView<'_>,
        stored: &Surface,
        poles: &HashSet<VertexKey>,
        room: f64,
    ) -> Result<Vec<u32>> {
        let back = face.location().inverse();
        let (from, to) = ends_in_the_stored_order(edge);
        let (mut at_start, up_at_start) = stored.coordinates_of(back.apply_to_point(from.point()));
        let (mut at_end, up_at_end) = stored.coordinates_of(back.apply_to_point(to.point()));
        let whole_turn = how_far_the_longitude_wraps(stored)
            .expect("a plane has no curve for this to walk along");

        // A pole names every longitude at once, so what it answers about its own
        // is not a fact about the surface — it is `atan2(0, 0)`. Where one end
        // is a pole, the other end says which meridian this is. Where both are,
        // nothing does, and zero is the answer. That is right for the only edge
        // in this kernel with a pole at each end: a ball's seam, which `add_ball`
        // puts at `y = 0, x ≥ 0` — longitude zero in the frame the ball is
        // stored in, and `the_seam_a_balls_mesh_is_cut_along_is_the_seam_the_model_has`
        // is the test that says so. A ball whose seam went elsewhere would get a
        // watertight mesh of the right ball cut in the wrong place; reading the
        // longitude off `EdgeRun::PartOfWhereItCrosses` is what would fix it, and
        // there is nothing yet that could tell the difference.
        match (poles.contains(&from.key()), poles.contains(&to.key())) {
            (false, false) if edge.is_closed() => at_end = at_start + whole_turn,
            (false, false) => {
                at_end = at_start + the_short_way_round(at_end - at_start, whole_turn)
            }
            (true, false) => at_start = at_end,
            (false, true) => at_end = at_start,
            (true, true) => {
                at_start = 0.0;
                at_end = 0.0;
            }
        }

        let steps = how_many_steps(stored, at_end - at_start, up_at_end - up_at_start, room);
        if self.points.len() + steps > MOST_POINTS_A_MESH_WILL_HOLD {
            return Err(Error::MoreMeshPointsThanASolidWillHold {
                most: MOST_POINTS_A_MESH_WILL_HOLD,
            });
        }

        let there = face.location();
        let mut walked = Vec::with_capacity(steps - 1);
        for step in 1..steps {
            let part = step as f64 / steps as f64;
            let along = stored.point_at(
                at_start + part * (at_end - at_start),
                up_at_start + part * (up_at_end - up_at_start),
            );
            walked.push(self.remember(there.apply_to_point(along)));
        }
        Ok(walked)
    }

    /// Cut one face into triangles and put them in the solid's own table.
    fn cut_up(&mut self, face: FaceView<'_>, log: &mut ProvenanceLog) -> Result<()> {
        let stored = face.surface().transformed(&face.location().inverse());
        let room = self.deflection.length() - how_far_placing_this_face_rounds(face);

        let mut walked = Vec::with_capacity(face.wire_count());
        for wire in face.wires() {
            walked.push(self.walk_a_wire_round(wire, face, &stored));
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
        // The two builders that make a curved face — `add_round_bar` and
        // `add_ball` — each give it one wire, `add_extrusion` makes planes only,
        // and the fuse refuses a curved solid outright. So no public route
        // reaches this, and cutting a face with a hole would have to say which
        // piece the hole ends up in.
        assert_eq!(
            walked.len(),
            1,
            "nothing in this kernel builds a curved face with a hole in it"
        );

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
    fn walk_a_wire_round(
        &mut self,
        wire: WireView<'_>,
        face: FaceView<'_>,
        stored: &Surface,
    ) -> WalkedLoop {
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
    /// the one it was already at.
    fn arrive(&mut self, at: u32, sits_at: (f64, f64), is_a_pole: bool) {
        let (raw, up) = sits_at;
        let longitude = match self.wraps {
            _ if is_a_pole => self.longitude,
            None => raw,
            Some(_) if !self.started => raw,
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
    fn cross_the_pole(&mut self) {
        let whole = self
            .wraps
            .expect("only a surface that wraps round can pinch to a pole");
        self.longitude += self.pole_side * whole;
        self.pole_side = -self.pole_side;
    }

    /// # Panics
    ///
    /// If the walk did not come back to the longitude it set off from. The loop
    /// handed over would not close, which is a bug here rather than bad input.
    fn finish(self) -> WalkedLoop {
        if let (Some(whole), Some(first)) = (self.wraps, self.corners.first()) {
            let drift = (self.longitude - first[0]).abs();
            assert!(
                drift < whole / 2.0,
                "a wire came back {drift} away in longitude from where it set off"
            );
        }
        WalkedLoop {
            corners: self.corners,
            points: self.points,
        }
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
/// it turns**: a step of more than half a turn of longitude cannot be told from
/// one the other way when the corner is read back, so no step is allowed past a
/// quarter of one.
///
/// # Panics
///
/// On a surface this has not been taught — see [`how_far_the_longitude_wraps`].
fn how_many_steps(stored: &Surface, round: f64, up: f64, room: f64) -> usize {
    let (radius, bends_over) = match stored {
        Surface::Plane(_) => return 1,
        // A ball bends both ways.
        Surface::Sphere(ball) => (ball.radius(), ball.radius() * round.hypot(up)),
        // A pipe is flat along its axis, so a piece stretched up one bends by
        // nothing at all and only the turn counts.
        Surface::Cylinder(pipe) => (pipe.radius(), pipe.radius() * round.abs()),
        other => panic!("this crate cannot walk along {other:?}"),
    };
    let longest_chord = (3.0 * radius * room * WHAT_THE_BOUNDARY_TAKES_OF_THE_BUDGET).sqrt();
    let by_bending = (bends_over / longest_chord).ceil();
    let by_turning = (round.abs() / (core::f64::consts::TAU / 4.0)).ceil();
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

/// How far moving this face's points from the frame it is stored in to where the
/// solid sits can shift one of them.
///
/// Zero for a flat face, and exactly zero: a plane's mesh is drawn through
/// corners the model already held and nothing here made any of them. The points a
/// bar's *cap* is drawn through were made on the wall beside it, and the wall
/// charges for them — [`Mesh::deviation`] is the worst over every face, so the
/// number that comes out still covers them.
fn how_far_placing_this_face_rounds(face: FaceView<'_>) -> f64 {
    Isometry::POINT_ROUNDOFF * how_far_the_points_of_reach(face)
}

/// The largest coordinate any point of this face can have.
///
/// Its corners, plus the radius of the surface it sits on — because a face
/// bulges away from its own corners, which is the whole of what
/// [`SolidView::box_around_its_corners`] stopped being able to say (§25.9).
/// Zero for a flat face, for the reason above.
///
/// # Panics
///
/// On a surface this has not been taught — see [`how_far_the_longitude_wraps`].
fn how_far_the_points_of_reach(face: FaceView<'_>) -> f64 {
    let radius = match face.surface() {
        Surface::Plane(_) => return 0.0,
        Surface::Sphere(ball) => ball.radius(),
        Surface::Cylinder(pipe) => pipe.radius(),
        other => panic!("this crate cannot say how far {other:?} reaches"),
    };
    let corners = face
        .wires()
        .flat_map(WireView::points)
        .map(|corner| corner.largest_coordinate_magnitude())
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
