use rcad_geom::{EdgeRun, Isometry, Orientation, Point, Scale, Surface};
use rcad_predicates::Approximation;
use serde::{Deserialize, Serialize};
use slotmap::{SecondaryMap, SlotMap};
use smallvec::SmallVec;

use crate::key::{
    CoedgeSlot, EdgeSlot, FaceSlot, ModelId, ShellSlot, SolidSlot, VertexSlot, WireSlot,
};
use crate::{
    CoedgeKey, CoedgeView, EdgeKey, EdgeView, Error, FaceKey, FaceView, Invalid, Result, ShellKey,
    ShellView, SolidKey, SolidView, VertexKey, VertexView, WireKey, WireView,
};

/// A whole B-Rep model: every vertex, edge, face and solid in one place.
///
/// The arena is the model. Entities live in flat `slotmap` arenas and refer to
/// each other by key, so there are no reference cycles, no `Arc`s and nothing
/// to keep alive (decisions.md §1.3). Downward links — a face's wires, a wire's
/// coedges — are stored; upward links are derived into a side index, so there
/// is only ever one copy of the truth.
///
/// Nothing hands out a reference to an entity's stored data. Every accessor
/// returns a *view*, which carries the borrow together with the location
/// composed on the way down. Geometry read out of a view has already been moved
/// into place, which is the whole reason views exist (decisions.md §3.3).
///
/// `&Model` is `Sync`. There is no interior mutability and no lazy cache, so
/// read-only work — measuring, classifying points, validating — parallelizes
/// with nothing more than a shared reference (roadmap §10).
#[derive(Debug)]
pub struct Model {
    id: ModelId,
    which_model_this_is: WhichModelThisIs,
    data: ModelData,
    index: Index,
    worst_approximation: f64,
}

impl Model {
    /// An empty model that works at the given size.
    ///
    /// The scale is carried for interop and for default tolerances; it never
    /// takes part in arithmetic (decisions.md §3.5).
    #[must_use]
    pub fn new(scale: Scale) -> Self {
        Self {
            id: ModelId::next(),
            which_model_this_is: WhichModelThisIs::next(),
            data: ModelData::new(scale),
            index: Index::default(),
            worst_approximation: 0.0,
        }
    }

    /// Which model this is, for a caller holding something built from it.
    ///
    /// Editing a model does not change the answer, so work cached against a
    /// model survives a fuse. Cloning or reading one in does change it, because
    /// what comes back is a second model and not the one the cache was built
    /// from.
    #[must_use]
    pub fn which_model_this_is(&self) -> WhichModelThisIs {
        self.which_model_this_is
    }

    /// The unit and working size this model was built at.
    #[must_use]
    pub fn scale(&self) -> Scale {
        self.data.scale
    }

    /// The largest [`Approximation`] anywhere in the model, or zero if nothing
    /// is approximate.
    ///
    /// This is the true maximum over the entities, not a high-water mark: tag
    /// the only approximate vertex in a model down to a smaller number and this
    /// comes down with it. A mark that only ever rose would disagree with the
    /// same model written out and read back, and it would hand every later
    /// growth check a baseline more permissive than the model deserves.
    ///
    /// Every mutation checks this has not grown, in release builds as well as
    /// debug. Tolerance inflation is the failure mode that turns into a week of
    /// bisection if it is noticed downstream instead of where it happened
    /// (decisions.md §4.3, roadmap §11.2).
    #[must_use]
    pub fn worst_approximation(&self) -> f64 {
        self.worst_approximation
    }

    /// How many vertices the model holds.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.data.vertices.len()
    }

    /// How many edges the model holds.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.data.edges.len()
    }

    /// How many coedges the model holds.
    #[must_use]
    pub fn coedge_count(&self) -> usize {
        self.data.coedges.len()
    }

    /// How many wires the model holds.
    #[must_use]
    pub fn wire_count(&self) -> usize {
        self.data.wires.len()
    }

    /// How many faces the model holds.
    #[must_use]
    pub fn face_count(&self) -> usize {
        self.data.faces.len()
    }

    /// How many shells the model holds.
    #[must_use]
    pub fn shell_count(&self) -> usize {
        self.data.shells.len()
    }

    /// How many solids the model holds.
    #[must_use]
    pub fn solid_count(&self) -> usize {
        self.data.solids.len()
    }

    /// A view of one solid, in the model's own frame.
    ///
    /// # Panics
    ///
    /// If the key came from another model. Debug builds say so by name; a
    /// release build cannot, because the model id is a zero-sized type there
    /// (decisions.md §1.3), so all the panic can report is that no solid of
    /// that name is here.
    ///
    /// **A key this model handed out always names something**, which is what
    /// makes that an internal invariant and not a bad-input case. Nothing in
    /// the crate removes an entity from an arena outside a test, and a file
    /// whose stored slots resolve to nothing never becomes a `Model` at all —
    /// `Deserialize` refuses it before the index is built, demonstrated by
    /// `a_file_naming_a_corner_that_is_not_there_never_becomes_a_model` in
    /// [`serialize`](crate::serialize).
    #[must_use]
    pub fn solid(&self, key: SolidKey) -> SolidView<'_> {
        let slot = key.slot_in(self.id);
        assert!(self.data.solids.contains_key(slot), "no such solid");
        SolidView::new(self, slot, Isometry::identity(), Orientation::Forward)
    }

    /// A view of one shell, in the model's own frame.
    ///
    /// # Panics
    ///
    /// If the key came from another model, the same way and with the same
    /// demonstration as [`Model::solid`].
    #[must_use]
    pub fn shell(&self, key: ShellKey) -> ShellView<'_> {
        let slot = key.slot_in(self.id);
        assert!(self.data.shells.contains_key(slot), "no such shell");
        ShellView::new(self, slot, Isometry::identity(), Orientation::Forward)
    }

    /// A view of one face, in the model's own frame.
    ///
    /// A face reached through a solid carries the location and orientation
    /// composed down from that solid instead. This one is the face as stored,
    /// which is what you want for tagging it or reading its own plane.
    ///
    /// # Panics
    ///
    /// If the key came from another model, the same way and with the same
    /// demonstration as [`Model::solid`].
    #[must_use]
    pub fn face(&self, key: FaceKey) -> FaceView<'_> {
        let slot = key.slot_in(self.id);
        assert!(self.data.faces.contains_key(slot), "no such face");
        FaceView::new(self, slot, Isometry::identity(), Orientation::Forward)
    }

    /// A view of one wire, in the model's own frame.
    ///
    /// # Panics
    ///
    /// If the key came from another model, the same way and with the same
    /// demonstration as [`Model::solid`].
    #[must_use]
    pub fn wire(&self, key: WireKey) -> WireView<'_> {
        let slot = key.slot_in(self.id);
        assert!(self.data.wires.contains_key(slot), "no such wire");
        WireView::new(self, slot, Isometry::identity(), Orientation::Forward)
    }

    /// A view of one coedge, in the model's own frame.
    ///
    /// # Panics
    ///
    /// If the key came from another model, the same way and with the same
    /// demonstration as [`Model::solid`].
    #[must_use]
    pub fn coedge(&self, key: CoedgeKey) -> CoedgeView<'_> {
        let slot = key.slot_in(self.id);
        assert!(self.data.coedges.contains_key(slot), "no such coedge");
        CoedgeView::new(self, slot, Isometry::identity(), Orientation::Forward)
    }

    /// A view of one edge, in the model's own frame.
    ///
    /// # Panics
    ///
    /// If the key came from another model, the same way and with the same
    /// demonstration as [`Model::solid`].
    #[must_use]
    pub fn edge(&self, key: EdgeKey) -> EdgeView<'_> {
        let slot = key.slot_in(self.id);
        assert!(self.data.edges.contains_key(slot), "no such edge");
        EdgeView::new(self, slot, Isometry::identity(), Orientation::Forward)
    }

    /// A view of one vertex, in the model's own frame.
    ///
    /// # Panics
    ///
    /// If the key came from another model, the same way and with the same
    /// demonstration as [`Model::solid`].
    #[must_use]
    pub fn vertex(&self, key: VertexKey) -> VertexView<'_> {
        let slot = key.slot_in(self.id);
        assert!(self.data.vertices.contains_key(slot), "no such vertex");
        VertexView::new(self, slot, Isometry::identity())
    }

    /// Every solid, in a fixed order that does not change between runs.
    pub fn solids(&self) -> impl Iterator<Item = SolidView<'_>> + '_ {
        self.data
            .solids
            .keys()
            .map(|slot| SolidView::new(self, slot, Isometry::identity(), Orientation::Forward))
    }

    /// Every face, in the model's own frame, in a fixed order.
    pub fn faces(&self) -> impl Iterator<Item = FaceView<'_>> + '_ {
        self.data
            .faces
            .keys()
            .map(|slot| FaceView::new(self, slot, Isometry::identity(), Orientation::Forward))
    }

    /// Every edge, in the model's own frame, in a fixed order.
    pub fn edges(&self) -> impl Iterator<Item = EdgeView<'_>> + '_ {
        self.data
            .edges
            .keys()
            .map(|slot| EdgeView::new(self, slot, Isometry::identity(), Orientation::Forward))
    }

    /// Every vertex, in the model's own frame, in a fixed order.
    pub fn vertices(&self) -> impl Iterator<Item = VertexView<'_>> + '_ {
        self.data
            .vertices
            .keys()
            .map(|slot| VertexView::new(self, slot, Isometry::identity()))
    }

    /// Move a solid. Everything read through it afterwards comes out moved.
    ///
    /// The movement composes onto the placement each of the solid's shells is
    /// already used at, so calling this twice is the same as calling it once
    /// with the two movements composed.
    ///
    /// # Panics
    ///
    /// If the key came from another model, or if the mutation leaves the solid
    /// broken.
    pub fn place_solid(&mut self, key: SolidKey, movement: &Isometry) {
        let slot = key.slot_in(self.id);
        let approximation_before = self.worst_approximation;

        let solid = self
            .data
            .solids
            .get_mut(slot)
            .expect("no such solid to place");
        for used in &mut solid.shells {
            used.loc = movement.compose(&used.loc);
        }

        self.check_touched_solid(slot);
        self.assert_approximation_did_not_grow(approximation_before);
    }

    /// Tag a vertex with the slop a decision put into it.
    ///
    /// # Errors
    ///
    /// [`Invalid::ApproximationOrdering`] if the tag would be smaller than one
    /// of the edges built on this vertex. A point cannot be known better than
    /// the edges through it (roadmap §5.4).
    ///
    /// # Panics
    ///
    /// If the key came from another model.
    pub fn set_vertex_approximation(&mut self, key: VertexKey, tag: Approximation) -> Result<()> {
        let slot = key.slot_in(self.id);
        for &edge in self.index.edges_of_vertex(slot) {
            let on_edge = magnitude_of(self.data.edges[edge].approximation);
            if tag.magnitude() < on_edge {
                return Err(ordering_broken("edge", on_edge, "vertex", tag.magnitude()));
            }
        }
        let replaced = self.data.vertices[slot].approximation.replace(tag);
        self.record_approximation(replaced, tag);
        Ok(())
    }

    /// Tag an edge with the slop a decision put into it.
    ///
    /// # Errors
    ///
    /// [`Invalid::ApproximationOrdering`] if the tag would be larger than one
    /// of the edge's vertices carries, or smaller than one of the faces that
    /// meet along it. [`Invalid::DanglingKey`] if one of the edge's own
    /// stored ends does not resolve — this is a caller-input case and not a
    /// broken invariant: the end is a `VertexSlot` stored inside `EdgeData`,
    /// which `Model`'s `Deserialize` reads wholesale with no check that it
    /// resolves, and calling this does not require `Model::validate` (which
    /// does catch it) to have run first.
    ///
    /// # Panics
    ///
    /// If the key came from another model.
    pub fn set_edge_approximation(&mut self, key: EdgeKey, tag: Approximation) -> Result<()> {
        let slot = key.slot_in(self.id);
        let edge = &self.data.edges[slot];
        for end in [edge.start, edge.end] {
            let on_vertex = magnitude_of(
                self.data
                    .vertices
                    .get(end)
                    .ok_or(Invalid::DanglingKey { entity: "vertex" })?
                    .approximation,
            );
            if on_vertex < tag.magnitude() {
                return Err(ordering_broken(
                    "edge",
                    tag.magnitude(),
                    "vertex",
                    on_vertex,
                ));
            }
        }
        for face in self.faces_along_edge(slot) {
            let on_face = magnitude_of(self.data.faces[face].approximation);
            if tag.magnitude() < on_face {
                return Err(ordering_broken("face", on_face, "edge", tag.magnitude()));
            }
        }
        let replaced = self.data.edges[slot].approximation.replace(tag);
        self.record_approximation(replaced, tag);
        Ok(())
    }

    /// Tag a face with the slop a decision put into it.
    ///
    /// # Errors
    ///
    /// [`Invalid::ApproximationOrdering`] if the tag would be larger than one
    /// of the face's own edges carries. [`Invalid::DanglingKey`] if one of
    /// the face's own stored wires, coedges or edges does not resolve —
    /// caller-input, the same way and for the same reason as
    /// [`Model::set_edge_approximation`]'s own `DanglingKey` case.
    ///
    /// # Panics
    ///
    /// If the key came from another model.
    pub fn set_face_approximation(&mut self, key: FaceKey, tag: Approximation) -> Result<()> {
        let slot = key.slot_in(self.id);
        for edge in self.edges_of_face(slot)? {
            let on_edge = magnitude_of(
                self.data
                    .edges
                    .get(edge)
                    .ok_or(Invalid::DanglingKey { entity: "edge" })?
                    .approximation,
            );
            if on_edge < tag.magnitude() {
                return Err(ordering_broken("face", tag.magnitude(), "edge", on_edge));
            }
        }
        let replaced = self.data.faces[slot].approximation.replace(tag);
        self.record_approximation(replaced, tag);
        Ok(())
    }

    pub(crate) fn id(&self) -> ModelId {
        self.id
    }

    pub(crate) fn data(&self) -> &ModelData {
        &self.data
    }

    pub(crate) fn index(&self) -> &Index {
        &self.index
    }

    /// The stored arena, so a test can break exactly one invariant and watch
    /// the sweep catch it.
    ///
    /// Test-only on purpose. Writing here goes round the methods that keep
    /// [`Index`] and the recorded worst approximation in step with the data,
    /// which is the whole point when the test wants a broken model and is a bug
    /// anywhere else.
    #[cfg(test)]
    pub(crate) fn data_mut(&mut self) -> &mut ModelData {
        &mut self.data
    }

    pub(crate) fn from_parts(data: ModelData) -> Self {
        let index = Index::rebuilt_from(&data);
        let worst_approximation = data.worst_approximation();
        Self {
            id: ModelId::next(),
            which_model_this_is: WhichModelThisIs::next(),
            data,
            index,
            worst_approximation,
        }
    }

    pub(crate) fn add_vertex(&mut self, point: Point) -> VertexSlot {
        let slot = self.data.vertices.insert(VertexData {
            point,
            approximation: None,
        });
        self.index.edges_of_vertex.insert(slot, SmallVec::new());
        slot
    }

    pub(crate) fn add_edge(&mut self, start: VertexSlot, end: VertexSlot) -> EdgeSlot {
        self.add_edge_running(start, end, EdgeRun::Straight)
    }

    /// An edge right round the closed curve where its two faces cross, starting
    /// and ending at `vertex`.
    pub(crate) fn add_closed_edge(&mut self, vertex: VertexSlot) -> EdgeSlot {
        self.add_edge_running(vertex, vertex, EdgeRun::RightRoundWhereItsFacesCross)
    }

    pub(crate) fn add_edge_running(
        &mut self,
        start: VertexSlot,
        end: VertexSlot,
        run: EdgeRun,
    ) -> EdgeSlot {
        let slot = self.data.edges.insert(EdgeData {
            start,
            end,
            run,
            approximation: None,
        });
        self.index.coedges_of_edge.insert(slot, SmallVec::new());
        self.index.edges_of_vertex[start].push(slot);
        if end != start {
            self.index.edges_of_vertex[end].push(slot);
        }
        slot
    }

    pub(crate) fn add_coedge(&mut self, edge: EdgeSlot, orientation: Orientation) -> CoedgeSlot {
        let slot = self.data.coedges.insert(CoedgeData { edge, orientation });
        self.index.coedges_of_edge[edge].push(slot);
        slot
    }

    pub(crate) fn add_wire(&mut self, coedges: SmallVec<[CoedgeSlot; 4]>) -> WireSlot {
        self.add_wire_of_steps(coedges.into_iter().map(WireStep::Along).collect())
    }

    pub(crate) fn add_wire_of_steps(&mut self, steps: SmallVec<[WireStep; 4]>) -> WireSlot {
        let slot = self.data.wires.insert(WireData {
            steps: steps.clone(),
        });
        for step in steps {
            if let WireStep::Along(coedge) = step {
                self.index.wire_of_coedge.insert(coedge, slot);
            }
        }
        slot
    }

    pub(crate) fn add_face(
        &mut self,
        surface: Surface,
        wires: SmallVec<[WireSlot; 1]>,
    ) -> FaceSlot {
        let slot = self.data.faces.insert(FaceData {
            surface,
            wires: wires.clone(),
            approximation: None,
        });
        for wire in wires {
            self.index.face_of_wire.insert(wire, slot);
        }
        slot
    }

    pub(crate) fn add_shell(&mut self, faces: Vec<FaceUse>) -> ShellSlot {
        self.data.shells.insert(ShellData { faces })
    }

    pub(crate) fn add_solid(&mut self, shells: SmallVec<[ShellUse; 1]>) -> SolidSlot {
        self.data.solids.insert(SolidData { shells })
    }

    pub(crate) fn key_for_solid(&self, slot: SolidSlot) -> SolidKey {
        SolidKey::new(slot, self.id)
    }

    /// The cheap checks that run after every mutation, on the entities that
    /// mutation touched and no others.
    ///
    /// A full [`Model::validate`] here would make a test that performs *n*
    /// operations cost O(n²) and the suite stops being runnable long before the
    /// milestone that needs it (decisions.md §4.3).
    ///
    /// **These run in debug builds only, and that is now a choice about cost
    /// rather than a difference in what a caller sees.** It was the second
    /// thing until [`Model::check_shape_is_possible`] closed the door: a file
    /// holding a wire that does not close read back cleanly, and
    /// [`Model::place_solid`] on it crashed a debug-built caller and said
    /// nothing to a release-built one — the same bytes, two answers, decided by
    /// how the caller happened to be compiled. A shape these can refuse cannot
    /// be deserialized any more, so what is left for them to catch is a bug in
    /// this crate, which is what a debug assertion is for.
    ///
    /// # Panics
    ///
    /// If any of them fail. Breaking one of these is a bug in this crate, and a
    /// corrupt solid is worse than a crash.
    pub(crate) fn check_touched_solid(&self, slot: SolidSlot) {
        if !cfg!(debug_assertions) {
            return;
        }
        let solid = self.data.solids.get(slot).expect("touched solid is gone");
        for used in &solid.shells {
            let shell = self
                .data
                .shells
                .get(used.shell)
                .expect("touched solid names a shell that is gone");
            for face_use in &shell.faces {
                self.check_touched_face(face_use.face);
            }
        }
    }

    pub(crate) fn check_touched_face(&self, slot: FaceSlot) {
        if !cfg!(debug_assertions) {
            return;
        }
        let face = self.data.faces.get(slot).expect("touched face is gone");
        assert!(!face.wires.is_empty(), "a face with no wires");
        for &wire in &face.wires {
            self.check_touched_wire(wire);
        }
    }

    pub(crate) fn check_touched_wire(&self, slot: WireSlot) {
        if !cfg!(debug_assertions) {
            return;
        }
        let wire = self.data.wires.get(slot).expect("touched wire is gone");
        self.check_wire_closes(wire)
            .unwrap_or_else(|broken| panic!("a touched wire is broken: {broken}"));
        for &step in &wire.steps {
            if let WireStep::Along(coedge) = step {
                self.check_touched_edge(self.data.coedges[coedge].edge);
            }
        }
    }

    /// Where one step round a wire starts and where it leaves off.
    ///
    /// A pole starts and ends at its own vertex, because it goes nowhere: the
    /// surface has pinched to a point and the boundary is standing on it.
    pub(crate) fn step_ends(&self, step: WireStep) -> (VertexSlot, VertexSlot) {
        match step {
            WireStep::Along(coedge) => self.coedge_ends(coedge),
            WireStep::Pole(vertex) => (vertex, vertex),
        }
    }

    /// Whether this wire is one coedge on an edge that already comes back to
    /// where it started — a round cap's boundary, which is one whole circle.
    pub(crate) fn wire_closes_on_one_edge(&self, wire: &WireData) -> bool {
        let [WireStep::Along(only)] = wire.steps[..] else {
            return false;
        };
        self.data.edges[self.data.coedges[only].edge].run == EdgeRun::RightRoundWhereItsFacesCross
    }

    /// Whether this wire is two steps that go somewhere and come back a
    /// different way, which is a face — an arc and the chord that closes it, or
    /// two arcs bulging opposite ways.
    ///
    /// One edge walked out and back retraces the same curve, and two straight
    /// edges between the same two corners are the same segment written twice.
    /// Both sweep exactly no area.
    ///
    /// [`SolidBuilder`](crate::SolidBuilder)'s `two_steps_that_bound_a_face` is
    /// the same rule asked of a description, and says why those two and not
    /// every two-step wire that bounds nothing — two curved edges can name the
    /// same curve too, and what says so is the pair of faces either side rather
    /// than the run.
    pub(crate) fn wire_of_two_steps_bounds_a_face(&self, wire: &WireData) -> bool {
        let [WireStep::Along(one), WireStep::Along(other)] = wire.steps[..] else {
            return false;
        };
        let (one, other) = (self.data.coedges[one].edge, self.data.coedges[other].edge);
        one != other
            && (self.data.edges[one].run != EdgeRun::Straight
                || self.data.edges[other].run != EdgeRun::Straight)
    }

    pub(crate) fn check_touched_edge(&self, slot: EdgeSlot) {
        if !cfg!(debug_assertions) {
            return;
        }
        let edge = self.data.edges.get(slot).expect("touched edge is gone");
        assert!(
            self.data.vertices.contains_key(edge.start)
                && self.data.vertices.contains_key(edge.end),
            "a touched edge names a vertex that is gone"
        );
        self.check_edge_is_walked_by_two_opposite_coedges(slot)
            .unwrap_or_else(|broken| panic!("a touched edge is broken: {broken}"));
    }

    /// The check that runs in release builds too (decisions.md §4.3).
    ///
    /// An operation that meant to leave slop behind passes the amount its
    /// provenance accounts for. What this catches is slop that appeared with
    /// nothing accounting for it.
    ///
    /// The comparison is only worth making if the number being compared is
    /// still the truth, so in debug builds this first walks the arena and
    /// checks it. That walk is the half that can catch a mistake made *inside*
    /// this crate: an entity written to directly instead of through
    /// [`Model::set_vertex_approximation`] and its siblings leaves the recorded
    /// figure stale, and a stale figure would be compared against a budget
    /// derived from it and agree with itself. The walk costs a pass over every
    /// entity, which is why it does not run in release — §4.3 budgets one
    /// comparison there, and a pass per operation is what makes a test suite
    /// quadratic.
    ///
    /// # Panics
    ///
    /// If the model's worst approximation ended up above `allowed`, or if the
    /// recorded figure no longer matches the entities.
    pub(crate) fn assert_approximation_did_not_grow(&self, allowed: f64) {
        debug_assert_eq!(
            self.worst_approximation,
            self.data.worst_approximation(),
            "slop was written into an entity without being recorded on the model"
        );
        assert!(
            self.worst_approximation <= allowed,
            "this operation left {} of slop behind and only {allowed} was accounted for",
            self.worst_approximation
        );
    }

    pub(crate) fn coedge_ends(&self, coedge: CoedgeSlot) -> (VertexSlot, VertexSlot) {
        let coedge = &self.data.coedges[coedge];
        let edge = &self.data.edges[coedge.edge];
        if coedge.orientation.is_reversed() {
            (edge.end, edge.start)
        } else {
            (edge.start, edge.end)
        }
    }

    pub(crate) fn faces_along_edge(&self, edge: EdgeSlot) -> impl Iterator<Item = FaceSlot> + '_ {
        self.index
            .coedges_of_edge(edge)
            .iter()
            .filter_map(move |&coedge| self.index.wire_of_coedge.get(coedge))
            .filter_map(move |&wire| self.index.face_of_wire.get(wire).copied())
    }

    /// Every edge along one of `face`'s wires.
    ///
    /// # Errors
    ///
    /// [`Invalid::DanglingKey`] if one of the face's own stored wires or
    /// coedges does not resolve. `Model` derives `Deserialize`, and `wires`
    /// and `coedges` are stored slots this reads wholesale, so a wire or
    /// coedge slot a file names can point at nothing.
    fn edges_of_face(&self, face: FaceSlot) -> Result<Vec<EdgeSlot>> {
        let mut edges = Vec::new();
        for &wire in &self.data.faces[face].wires {
            let wire = self
                .data
                .wires
                .get(wire)
                .ok_or(Invalid::DanglingKey { entity: "wire" })?;
            for coedge in wire.coedges() {
                let coedge = self
                    .data
                    .coedges
                    .get(coedge)
                    .ok_or(Invalid::DanglingKey { entity: "coedge" })?;
                edges.push(coedge.edge);
            }
        }
        Ok(edges)
    }

    /// Bring [`Model::worst_approximation`] back in step after one entity's tag
    /// changed from `replaced` to `tag`.
    ///
    /// Taking the larger of the two is only right while tags go up. A tag that
    /// comes down may have been the one holding the maximum, and then the only
    /// way to know the new maximum is to look, so that case walks the arena.
    fn record_approximation(&mut self, replaced: Option<Approximation>, tag: Approximation) {
        if magnitude_of(replaced) > tag.magnitude() {
            self.worst_approximation = self.data.worst_approximation();
        } else {
            self.worst_approximation = self.worst_approximation.max(tag.magnitude());
        }
    }
}

/// Written out rather than derived, because the two identities a model carries
/// have to part company here.
///
/// The clone's keys address the same slots as the original's, so the per-key
/// model id is copied across and a key handed out before the clone goes on
/// working against either model. [`WhichModelThisIs`] is minted fresh, because
/// the clone is a second model that can be edited on its own, and anything
/// cached against the original was not built from it.
impl Clone for Model {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            which_model_this_is: WhichModelThisIs::next(),
            data: self.data.clone(),
            index: self.index.clone(),
            worst_approximation: self.worst_approximation,
        }
    }
}

/// The answer to [`Model::which_model_this_is`]: an opaque mark that says one
/// model apart from every other.
///
/// It is for comparing, and nothing else. There is no way to make one, and no
/// way to read a number out of one, so the only thing a caller can do with two
/// of them is ask whether they match. That is what a cache keyed on a model
/// needs: check the mark it stored against the mark the model carries now, and
/// refuse to use the cache when they differ.
///
/// This is **not** the per-key model id the crate stamps into every key so that
/// one model's key cannot resolve against another's arena (decisions.md §1.3).
/// That one is zero-sized in release builds and survives a clone; this one is
/// present in every build and does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WhichModelThisIs(u64);

impl WhichModelThisIs {
    fn next() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};

        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

fn magnitude_of(tag: Option<Approximation>) -> f64 {
    tag.map_or(0.0, Approximation::magnitude)
}

fn ordering_broken(
    outer: &'static str,
    outer_magnitude: f64,
    inner: &'static str,
    inner_magnitude: f64,
) -> Error {
    Error::Invalid(Invalid::ApproximationOrdering {
        outer,
        outer_magnitude,
        inner,
        inner_magnitude,
    })
}

/// Everything a model owns that survives being written to disk.
///
/// Split out from [`Model`] so the derived index and the run-local model id
/// never reach the file. The index is rebuilt on the way back in, which means
/// there is no way for a file to carry an index that disagrees with the data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ModelData {
    pub scale: Scale,
    pub vertices: SlotMap<VertexSlot, VertexData>,
    pub edges: SlotMap<EdgeSlot, EdgeData>,
    pub coedges: SlotMap<CoedgeSlot, CoedgeData>,
    pub wires: SlotMap<WireSlot, WireData>,
    pub faces: SlotMap<FaceSlot, FaceData>,
    pub shells: SlotMap<ShellSlot, ShellData>,
    pub solids: SlotMap<SolidSlot, SolidData>,
}

impl ModelData {
    fn new(scale: Scale) -> Self {
        Self {
            scale,
            vertices: SlotMap::with_key(),
            edges: SlotMap::with_key(),
            coedges: SlotMap::with_key(),
            wires: SlotMap::with_key(),
            faces: SlotMap::with_key(),
            shells: SlotMap::with_key(),
            solids: SlotMap::with_key(),
        }
    }

    /// Every slot stored in one entity names an entity the arenas hold.
    ///
    /// The seven arenas point at each other by slot and [`Model`] derives
    /// `Deserialize`, so a file names whatever slots it likes: a solid can name
    /// a shell that is not there, and so can every other level. Nothing
    /// downstream survives one. A view built over a missing slot indexes its
    /// arena, and that is a panic — which is how a stranger's bytes crashed the
    /// STEP writer *and* [`Model::validate`], the check whose job was to say
    /// the bytes were unsound.
    ///
    /// So this runs before anything walks the model: first in `validate`, and
    /// on the way in from `Deserialize` before [`Model::from_parts`] builds the
    /// index, because a slot index a file made up is unbounded and
    /// `SecondaryMap::insert` grows to fit it.
    ///
    /// # Errors
    ///
    /// [`Invalid::DanglingKey`] naming the kind of entity the first slot that
    /// resolves to nothing was reaching for.
    pub(crate) fn check_keys_resolve(&self) -> Result<()> {
        for edge in self.edges.values() {
            for end in [edge.start, edge.end] {
                if !self.vertices.contains_key(end) {
                    return Err(Invalid::DanglingKey { entity: "vertex" }.into());
                }
            }
        }
        for coedge in self.coedges.values() {
            if !self.edges.contains_key(coedge.edge) {
                return Err(Invalid::DanglingKey { entity: "edge" }.into());
            }
        }
        for wire in self.wires.values() {
            for coedge in wire.coedges() {
                if !self.coedges.contains_key(coedge) {
                    return Err(Invalid::DanglingKey { entity: "coedge" }.into());
                }
            }
            for &step in &wire.steps {
                if let WireStep::Pole(vertex) = step
                    && !self.vertices.contains_key(vertex)
                {
                    return Err(Invalid::DanglingKey { entity: "vertex" }.into());
                }
            }
        }
        for face in self.faces.values() {
            for &wire in &face.wires {
                if !self.wires.contains_key(wire) {
                    return Err(Invalid::DanglingKey { entity: "wire" }.into());
                }
            }
        }
        for shell in self.shells.values() {
            for used in &shell.faces {
                if !self.faces.contains_key(used.face) {
                    return Err(Invalid::DanglingKey { entity: "face" }.into());
                }
            }
        }
        for solid in self.solids.values() {
            for used in &solid.shells {
                if !self.shells.contains_key(used.shell) {
                    return Err(Invalid::DanglingKey { entity: "shell" }.into());
                }
            }
        }
        Ok(())
    }

    /// No entity is claimed by two owners: one wire belongs to one face, one
    /// coedge to one wire, one face to one shell, one shell to one solid.
    ///
    /// **A check the rule in `Model::check_shape_is_possible` reaches**
    /// (`docs/decisions.md` §44.5) — no [`Policy`](rcad_predicates::Policy), no
    /// arithmetic on any coordinate, and no sweep over pairs. It asks nothing
    /// of any *pair* of entities: one pass over the containers that name
    /// entities adds one to a counter, a second pass over those counters looks
    /// for a two, and the whole of it is one `SecondaryMap<K, usize>` per
    /// level.
    ///
    /// **It is asked here, on the data, rather than in
    /// `Model::check_shape_is_possible`, because [`Index::rebuilt_from`] is what
    /// it protects and the index is built first.** A wire named by two faces
    /// reaches `face_of_wire.insert` twice, and the second `insert` used to win
    /// silently: the model that came out of the door said six faces with one
    /// wire each while its index gave one of those wires to the wrong face, and
    /// nothing asked it a question until something downstream believed the
    /// index. So the count runs before the index exists, and `rebuilt_from`
    /// asserts what this has already refused.
    ///
    /// [`Model::validate`] counts the same four again, and that is not a
    /// duplicate: a model can be mutated after it comes through the door, and
    /// the sweep is what catches that.
    ///
    /// # Errors
    ///
    /// [`Invalid::UsedMoreThanOnce`] naming the first level where some entity
    /// has more than one owner.
    pub(crate) fn check_nothing_is_used_more_than_once(&self) -> Result<()> {
        no_owner_claims_the_same_entity_twice(
            "coedge",
            self.wires.values().flat_map(WireData::coedges),
        )?;
        no_owner_claims_the_same_entity_twice(
            "wire",
            self.faces
                .values()
                .flat_map(|face| face.wires.iter().copied()),
        )?;
        no_owner_claims_the_same_entity_twice(
            "face",
            self.shells
                .values()
                .flat_map(|shell| shell.faces.iter().map(|used| used.face)),
        )?;
        no_owner_claims_the_same_entity_twice(
            "shell",
            self.solids
                .values()
                .flat_map(|solid| solid.shells.iter().map(|used| used.shell)),
        )
    }

    /// Everything the arenas hold is used by something above it: every coedge
    /// walked by a wire, every wire bounding a face, every face in a shell,
    /// every shell in a solid.
    ///
    /// The same four counters as
    /// [`ModelData::check_nothing_is_used_more_than_once`], asked for a zero
    /// instead of a two, and it passes the same rule (`docs/decisions.md`
    /// §44.5): one pass per level, no [`Policy`](rcad_predicates::Policy) and no
    /// arithmetic on any coordinate.
    ///
    /// A file can hold a face no shell names, and until this ran it read back
    /// clean — the keys all resolve, nothing is claimed twice, and every shape
    /// question `Model::check_shape_is_possible` asks has the right answer. The
    /// model that came out then said one thing about itself and another to
    /// anything that walked it from the solids down: the orphan is in
    /// `Model::faces()` and in the bytes written back out, and in no solid's
    /// skin. [`Model::validate`] has always refused one; `Deserialize` does not
    /// call `validate`.
    ///
    /// A vertex used by no edge is the fifth orphan and is not asked here.
    /// Answering it means the upward links, which is [`Index`] — built after
    /// this — so `validate` keeps that one.
    ///
    /// # Errors
    ///
    /// [`Invalid::Orphan`] naming the first level holding something nothing
    /// uses.
    pub(crate) fn check_everything_has_an_owner(&self) -> Result<()> {
        everything_at_this_level_is_used(
            "coedge",
            self.coedges.keys(),
            self.wires.values().flat_map(WireData::coedges),
        )?;
        everything_at_this_level_is_used(
            "wire",
            self.wires.keys(),
            self.faces
                .values()
                .flat_map(|face| face.wires.iter().copied()),
        )?;
        everything_at_this_level_is_used(
            "face",
            self.faces.keys(),
            self.shells
                .values()
                .flat_map(|shell| shell.faces.iter().map(|used| used.face)),
        )?;
        everything_at_this_level_is_used(
            "shell",
            self.shells.keys(),
            self.solids
                .values()
                .flat_map(|solid| solid.shells.iter().map(|used| used.shell)),
        )
    }

    fn worst_approximation(&self) -> f64 {
        let vertices = self
            .vertices
            .values()
            .map(|entity| magnitude_of(entity.approximation));
        let edges = self
            .edges
            .values()
            .map(|entity| magnitude_of(entity.approximation));
        let faces = self
            .faces
            .values()
            .map(|entity| magnitude_of(entity.approximation));
        vertices.chain(edges).chain(faces).fold(0.0, f64::max)
    }
}

/// One pass to count how many times each entity is named, one pass to find the
/// first named twice.
fn no_owner_claims_the_same_entity_twice<K: slotmap::Key>(
    entity: &'static str,
    named: impl Iterator<Item = K>,
) -> Result<()> {
    let mut used: SecondaryMap<K, usize> = SecondaryMap::new();
    for slot in named {
        if let Some(entry) = used.entry(slot) {
            *entry.or_insert(0) += 1;
        }
    }
    for (_, &count) in &used {
        if count > 1 {
            return Err(Invalid::UsedMoreThanOnce { entity, count }.into());
        }
    }
    Ok(())
}

/// One pass to mark every entity an owner names, one pass over the arena to
/// find the first that nothing marked.
fn everything_at_this_level_is_used<K: slotmap::Key>(
    entity: &'static str,
    held: impl Iterator<Item = K>,
    named: impl Iterator<Item = K>,
) -> Result<()> {
    let mut used: SecondaryMap<K, ()> = SecondaryMap::new();
    for slot in named {
        if let Some(entry) = used.entry(slot) {
            entry.or_insert(());
        }
    }
    for slot in held {
        if !used.contains_key(slot) {
            return Err(Invalid::Orphan { entity }.into());
        }
    }
    Ok(())
}

/// The upward links, derived from the downward ones.
///
/// Storing both directions would mean two copies of the same truth and a way
/// for them to disagree. Storing only downward links and deriving this leaves
/// one (roadmap §2.3). `SecondaryMap` keys off the slot index, so none of this
/// hashes.
#[derive(Clone, Debug, Default)]
pub(crate) struct Index {
    edges_of_vertex: SecondaryMap<VertexSlot, SmallVec<[EdgeSlot; 4]>>,
    coedges_of_edge: SecondaryMap<EdgeSlot, SmallVec<[CoedgeSlot; 2]>>,
    wire_of_coedge: SecondaryMap<CoedgeSlot, WireSlot>,
    face_of_wire: SecondaryMap<WireSlot, FaceSlot>,
}

impl Index {
    pub(crate) fn edges_of_vertex(&self, vertex: VertexSlot) -> &[EdgeSlot] {
        self.edges_of_vertex.get(vertex).map_or(&[], |list| list)
    }

    pub(crate) fn coedges_of_edge(&self, edge: EdgeSlot) -> &[CoedgeSlot] {
        self.coedges_of_edge.get(edge).map_or(&[], |list| list)
    }

    pub(crate) fn wire_of_coedge(&self, coedge: CoedgeSlot) -> Option<WireSlot> {
        self.wire_of_coedge.get(coedge).copied()
    }

    pub(crate) fn face_of_wire(&self, wire: WireSlot) -> Option<FaceSlot> {
        self.face_of_wire.get(wire).copied()
    }

    /// The upward links read back off the downward ones.
    ///
    /// **The two single-valued links assert rather than overwrite, and the
    /// assert is unconditional.** A wire named by two faces has two candidate
    /// owners and `insert` would keep whichever face was written last, leaving
    /// an index that disagrees with the data it was derived from — a model that
    /// contradicts itself before anything asks it a question. That is a broken
    /// internal invariant and not bad caller input: the only route here is
    /// [`Model::from_parts`], which is `pub(crate)`, and the one route to it
    /// from outside the crate is `Deserialize` — which has already run
    /// [`ModelData::check_nothing_is_used_more_than_once`] and turned a second
    /// claimant into an `Err`. CLAUDE.md says panic on the first and return
    /// `Err` on the second, and both halves are here in that order.
    ///
    /// `debug_assert!` would be the wrong half of that rule. The build a user
    /// reads a stranger's file in is the release build, and a check that is
    /// absent exactly there is not a check. The cost is one `Option::is_none`
    /// per wire and per coedge, beside an insert that already writes a word.
    fn rebuilt_from(data: &ModelData) -> Self {
        let mut index = Self::default();
        for vertex in data.vertices.keys() {
            index.edges_of_vertex.insert(vertex, SmallVec::new());
        }
        for edge in data.edges.keys() {
            index.coedges_of_edge.insert(edge, SmallVec::new());
        }
        for (slot, edge) in &data.edges {
            for end in [edge.start, edge.end] {
                if let Some(list) = index.edges_of_vertex.get_mut(end) {
                    list.push(slot);
                }
            }
        }
        for (slot, coedge) in &data.coedges {
            if let Some(list) = index.coedges_of_edge.get_mut(coedge.edge) {
                list.push(slot);
            }
        }
        for (slot, wire) in &data.wires {
            for coedge in wire.coedges() {
                let already = index.wire_of_coedge.insert(coedge, slot);
                assert!(
                    already.is_none(),
                    "one coedge is walked by two wires, {already:?} and {slot:?}"
                );
            }
        }
        for (slot, face) in &data.faces {
            for &wire in &face.wires {
                let already = index.face_of_wire.insert(wire, slot);
                assert!(
                    already.is_none(),
                    "one wire bounds two faces, {already:?} and {slot:?}"
                );
            }
        }
        index
    }
}

/// A point in space, and how much slop a decision put there.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct VertexData {
    pub point: Point,
    pub approximation: Option<Approximation>,
}

/// An edge between two vertices, and what it runs along between them.
///
/// No curve is stored, and that is the decision rather than an omission
/// (decisions.md §22, Q7/Q18). A curve here would be a second authority on
/// where the edge is, sitting beside the surfaces of the faces either side of
/// it, and the first time the two disagreed by an ulp there would be no way to
/// say which was right. What an edge carries instead is its two ends and a
/// [`EdgeRun`] naming *which* of the things those surfaces describe it is.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct EdgeData {
    pub start: VertexSlot,
    pub end: VertexSlot,
    pub run: EdgeRun,
    pub approximation: Option<Approximation>,
}

/// One use of an edge by one wire, running one way round.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct CoedgeData {
    pub edge: EdgeSlot,
    pub orientation: Orientation,
}

/// A closed chain of steps round the boundary of a face.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WireData {
    pub steps: SmallVec<[WireStep; 4]>,
}

impl WireData {
    /// The coedges of this wire, skipping any poles.
    pub fn coedges(&self) -> impl Iterator<Item = CoedgeSlot> + '_ {
        self.steps.iter().filter_map(|step| match step {
            WireStep::Along(coedge) => Some(*coedge),
            WireStep::Pole(_) => None,
        })
    }
}

/// One step round a wire.
///
/// A wire is *almost* always a chain of coedges, and was only that until a ball
/// arrived. What a ball adds is the pole: the top of a sphere is one point in
/// space, and the whole top side of the face's `(u, v)` rectangle maps onto it.
/// The boundary has to go somewhere between arriving at the pole and leaving
/// it, and where it goes is nowhere at all.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) enum WireStep {
    /// Along one use of an edge.
    Along(CoedgeSlot),
    /// Standing still at a pole, where the surface pinches to a point.
    ///
    /// **Deliberately not an edge of zero length.** Roadmap §7.4 lists
    /// zero-length edges in the degeneracy catalogue as something to detect and
    /// refuse, and OCCT's answer — a degenerate edge that is zero-long in space
    /// and full-length in `(u, v)` — makes that entry unaskable, because the
    /// check can no longer tell the good kind from the broken kind. Giving the
    /// pole its own step keeps a zero-length edge unconditionally wrong
    /// (decisions.md §19 of the M2 plan).
    Pole(VertexSlot),
}

/// A patch of surface, trimmed by its wires.
///
/// The first wire is the outer one and the rest are holes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct FaceData {
    pub surface: Surface,
    pub wires: SmallVec<[WireSlot; 1]>,
    pub approximation: Option<Approximation>,
}

/// A closed skin of faces.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ShellData {
    pub faces: Vec<FaceUse>,
}

/// A lump of material, bounded by its shells.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SolidData {
    pub shells: SmallVec<[ShellUse; 1]>,
}

/// How one shell uses one face: where it is and which way round.
///
/// Orientation belongs here rather than on the face, because the same face is
/// forward for the solid on one side of it and reversed for the solid on the
/// other (roadmap §5.2).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) struct FaceUse {
    pub face: FaceSlot,
    pub loc: Isometry,
    pub orient: Orientation,
}

/// How one solid uses one shell: where it is and which way round.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) struct ShellUse {
    pub shell: ShellSlot,
    pub loc: Isometry,
    pub orient: Orientation,
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Axis, Orientation, Point, Scale, Vector};
    use rcad_predicates::{Approximation, Cause, Policy, ProvenanceLog};

    use super::Model;
    use crate::{Error, Invalid, Rule, SolidKey};

    fn a_cube() -> (Model, SolidKey) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cube = model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side two");
        (model, cube)
    }

    fn a_movement() -> rcad_geom::Isometry {
        let axis = Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length");
        rcad_geom::Isometry::rotation(&axis, 0.7).expect("a rotation")
    }

    #[test]
    fn a_face_reached_through_a_placed_solid_comes_back_already_moved() {
        let (mut model, cube) = a_cube();
        let stored: Vec<_> = model.faces().map(crate::FaceView::plane).collect();

        let movement = a_movement();
        model.place_solid(cube, &movement);

        let moved: Vec<_> = model
            .solid(cube)
            .faces()
            .map(crate::FaceView::plane)
            .collect();
        for (before, after) in stored.iter().zip(&moved) {
            assert_eq!(*after, before.transformed(&movement));
        }
    }

    /// **This used to be `a_location_on_a_shell_and_one_on_a_face_compose`**,
    /// which put a shift on every face use and checked the two composed. They
    /// do, and the composition is still there in [`SolidView::faces`]; what has
    /// gone is any way to reach it, because a face use that carries a placement
    /// is now [`Invalid::AFaceUseCarriesAPlacement`] and this is the same shift
    /// asked for as a file.
    ///
    /// The shell half of §3.2's composition is what a caller actually has, and
    /// `a_face_reached_through_a_placed_solid_comes_back_already_moved` above is
    /// where that lives.
    #[test]
    fn a_file_that_shifts_one_face_off_its_shell_never_becomes_a_model() {
        let (mut model, _) = a_cube();
        assert!(
            postcard::from_bytes::<Model>(
                &postcard::to_allocvec(&model).expect("a model writes out")
            )
            .is_ok(),
            "the same cube with nothing shifted reads back"
        );

        let shell = model
            .data
            .shells
            .keys()
            .next()
            .expect("a cube has one shell");
        model.data.shells[shell].faces[0].loc =
            rcad_geom::Isometry::translation(Vector::new(5.0, -2.0, 1.0)).expect("a finite shift");

        assert_eq!(
            postcard::from_bytes::<Model>(
                &postcard::to_allocvec(&model).expect("a model writes out")
            )
            .expect_err("a face placed off its shell is refused"),
            postcard::Error::SerdeDeCustom
        );
    }

    /// The nudge, which is the half nothing else could ever have seen.
    ///
    /// A quarter of a unit leaves every corner of the model where it was — the
    /// mesher names a corner by its vertex, and both faces along the edge name
    /// the same vertex — so the mesh closes and the volume is right and the skin
    /// is torn all the same. A thousand units is no different in kind and only
    /// easier to notice by eye.
    #[test]
    fn a_face_nudged_off_its_shell_is_caught_as_surely_as_one_thrown_across_the_room() {
        for shift in [0.25, 1000.0] {
            let (mut model, _) = a_cube();
            let shell = model
                .data
                .shells
                .keys()
                .next()
                .expect("a cube has one shell");
            model.data.shells[shell].faces[0].loc =
                rcad_geom::Isometry::translation(Vector::new(shift, 0.0, 0.0))
                    .expect("a finite shift");

            let refused = model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect_err("a face placed off its shell is not a model");
            assert!(
                matches!(refused, Error::Invalid(Invalid::AFaceUseCarriesAPlacement)),
                "a shift of {shift} gave {refused:?}"
            );
        }
    }

    #[test]
    fn using_a_face_the_other_way_round_turns_its_outward_normal_and_its_volume() {
        let (mut model, cube) = a_cube();
        let outward: Vec<_> = model
            .solid(cube)
            .faces()
            .map(crate::FaceView::outward_normal)
            .collect();
        let volume = model.solid(cube).signed_volume();

        for shell in model.data.shells.values_mut() {
            for used in &mut shell.faces {
                used.orient = used.orient.reversed();
            }
        }

        for (before, face) in outward.iter().zip(model.solid(cube).faces()) {
            assert_eq!(face.outward_normal(), -*before);
        }
        assert_eq!(model.solid(cube).signed_volume(), -volume);
    }

    /// §20.1 settled that `measure` panics on a solid whose faces point
    /// inwards, because the only way to get one was a bug in this crate.
    /// `Model` is `Deserialize`, so there is a second way: these bytes read
    /// back with nothing forged, and a caller's bytes may not panic.
    ///
    /// The number in the refusal is the one `validate` gives for the same
    /// bytes, which is what makes the two answers comparable rather than
    /// merely both being errors.
    ///
    /// **The inside-out cube is written the way a file has to write one**: every
    /// plane turned round, and every wire walked the other way so that it still
    /// winds with the normal it is given. Turning the six *face uses* round
    /// instead is the same solid to look at and is no longer a file at all — a
    /// plane welds "which surface" to "which way", so a plane that faces the
    /// other way is stored turned round and
    /// [`Model::check_no_flat_face_is_used_reversed`] refuses those bytes at the
    /// door. Written this way the sweep has nothing to say about any one face
    /// and the volume is the only thing left that can notice, which is what this
    /// test is for; `a_flat_face_used_reversed_is_caught` in `validate.rs`
    /// covers the other spelling.
    #[test]
    fn measuring_a_solid_read_back_inside_out_is_refused_rather_than_crashing() {
        let (mut model, _) = a_cube();
        for face in model.data.faces.values_mut() {
            let flat = face.surface.as_plane().expect("a cube is six planes");
            face.surface = rcad_geom::Surface::Plane(flat.reversed());
        }
        for coedge in model.data.coedges.values_mut() {
            coedge.orientation = coedge.orientation.reversed();
        }
        for wire in model.data.wires.values_mut() {
            wire.steps.reverse();
        }
        let bytes = postcard::to_allocvec(&model).expect("a model writes out");
        let read_back: Model = postcard::from_bytes(&bytes).expect("and reads back");

        let refused = read_back
            .solids()
            .next()
            .expect("the cube came back")
            .measure(
                rcad_geom::Deflection::of(0.1).expect("a length a polyhedron ignores"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a solid enclosing negative space has no measurement");

        assert!(
            matches!(
                refused,
                Error::Invalid(Invalid::VolumeIsNotPositive { volume }) if volume == -8.0
            ),
            "measure gave {refused:?}"
        );
        let swept = read_back
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect_err("validate refuses the same bytes");
        assert!(
            matches!(
                swept,
                Error::Invalid(Invalid::VolumeIsNotPositive { volume }) if volume == -8.0
            ),
            "validate gave {swept:?}"
        );
    }

    /// The same for a solid with no fan, whose volume is summed over a mesh.
    ///
    /// A ball is the one shape the direction rule cannot catch — its only seam
    /// runs pole to pole, so no closed edge is walked against its surface — and
    /// the sum over its triangles is all that is left to notice it (§23.9).
    #[test]
    fn measuring_a_ball_read_back_inside_out_is_refused_rather_than_crashing() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        model
            .add_ball(2.0, &rcad_geom::Isometry::identity())
            .expect("a ball of radius two");
        for shell in model.data.shells.values_mut() {
            for used in &mut shell.faces {
                used.orient = used.orient.reversed();
            }
        }
        let bytes = postcard::to_allocvec(&model).expect("a model writes out");
        let read_back: Model = postcard::from_bytes(&bytes).expect("and reads back");

        let refused = read_back
            .solids()
            .next()
            .expect("the ball came back")
            .measure(
                rcad_geom::Deflection::of(0.1).expect("a length a ball can be meshed to"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a ball enclosing negative space has no measurement");

        assert!(
            matches!(
                refused,
                Error::Invalid(Invalid::VolumeIsNotPositive { volume }) if volume < 0.0
            ),
            "measure gave {refused:?}"
        );
    }

    #[test]
    fn a_face_used_the_other_way_round_walks_its_wires_the_other_way() {
        let (mut model, cube) = a_cube();
        let one_way: Vec<Vec<Point>> = model
            .solid(cube)
            .faces()
            .map(|face| face.outer_wire().points().collect())
            .collect();

        for shell in model.data.shells.values_mut() {
            for used in &mut shell.faces {
                used.orient = used.orient.reversed();
            }
        }

        for (forwards, face) in one_way.iter().zip(model.solid(cube).faces()) {
            let backwards: Vec<Point> = face.outer_wire().points().collect();
            let mut expected = vec![forwards[0]];
            expected.extend(forwards[1..].iter().rev().copied());
            assert_eq!(backwards, expected);
        }
    }

    #[test]
    fn a_vertex_of_a_placed_solid_comes_back_already_moved() {
        let (mut model, cube) = a_cube();
        let movement = a_movement();
        model.place_solid(cube, &movement);

        let face = model.solid(cube).faces().next().expect("a cube has faces");
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let vertex = coedge.start();
                let stored = model.data.vertices[vertex.key().slot_in(model.id)].point;
                assert_eq!(vertex.point(), movement.apply_to_point(stored));
            }
        }
    }

    #[test]
    fn every_edge_of_a_cube_knows_the_two_faces_that_meet_along_it() {
        let (model, _) = a_cube();
        for edge in model.edges() {
            assert_eq!(edge.coedges().count(), 2);
            assert_eq!(edge.neighbouring_faces().count(), 2);
        }
        for vertex in model.vertices() {
            assert_eq!(vertex.edges().count(), 3);
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "made by a different model")]
    fn a_key_from_one_model_may_not_be_used_on_another() {
        let (_first, cube) = a_cube();
        let (second, _) = a_cube();
        let _ = second.solid(cube);
    }

    #[test]
    fn tagging_a_face_with_more_slop_than_its_edges_carry_is_refused() {
        let (mut model, _) = a_cube();
        let face = model.faces().next().expect("a cube has faces").key();

        let refused =
            model.set_face_approximation(face, Approximation::new(1e-6, Cause::TolerantFallback));

        assert!(matches!(
            refused,
            Err(Error::Invalid(Invalid::ApproximationOrdering { .. }))
        ));
        assert_eq!(model.worst_approximation(), 0.0);
    }

    /// `Model` derives `Deserialize`, so an edge's stored end can point at a
    /// vertex the arenas do not hold — nothing between the bytes and this
    /// call confirms it does, since `set_edge_approximation` does not
    /// require `Model::validate` (which does catch it) to have run first.
    #[test]
    fn tagging_an_edge_whose_stored_end_is_gone_is_refused_rather_than_asserted_against() {
        let (mut model, _) = a_cube();
        let vertex = model
            .data()
            .vertices
            .keys()
            .next()
            .expect("a cube has corners");
        let edge = model
            .data()
            .edges
            .iter()
            .find(|(_, data)| data.start == vertex || data.end == vertex)
            .map(|(slot, _)| slot)
            .expect("a corner is on some edge");
        let edge = crate::EdgeKey::new(edge, model.id());
        model.data_mut().vertices.remove(vertex);

        assert!(matches!(
            model
                .set_edge_approximation(edge, Approximation::new(1e-6, Cause::ImportedUncertainty)),
            Err(Error::Invalid(Invalid::DanglingKey { entity: "vertex" }))
        ));
    }

    /// The same case one arena further out: a face's stored wire can point at
    /// a wire the arenas do not hold.
    #[test]
    fn tagging_a_face_whose_stored_wire_is_gone_is_refused_rather_than_asserted_against() {
        let (mut model, _) = a_cube();
        let face = model.data().faces.keys().next().expect("a cube has faces");
        let wire = model.data().faces[face].wires[0];
        let face = crate::FaceKey::new(face, model.id());
        model.data_mut().wires.remove(wire);

        assert!(matches!(
            model
                .set_face_approximation(face, Approximation::new(1e-6, Cause::ImportedUncertainty)),
            Err(Error::Invalid(Invalid::DanglingKey { entity: "wire" }))
        ));
    }

    #[test]
    fn slop_may_be_tagged_from_the_vertices_inwards() {
        let (mut model, cube) = a_cube();
        let face = model.faces().next().expect("a cube has faces").key();

        for vertex in model.vertices().map(|v| v.key()).collect::<Vec<_>>() {
            model
                .set_vertex_approximation(
                    vertex,
                    Approximation::new(1e-6, Cause::ImportedUncertainty),
                )
                .expect("a vertex may carry the most slop");
        }
        for edge in model.edges().map(|e| e.key()).collect::<Vec<_>>() {
            model
                .set_edge_approximation(edge, Approximation::new(1e-7, Cause::ImportedUncertainty))
                .expect("an edge may carry less than its vertices");
        }
        model
            .set_face_approximation(face, Approximation::new(1e-8, Cause::ImportedUncertainty))
            .expect("a face may carry less than its edges");

        assert_eq!(model.worst_approximation(), 1e-6);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("the ordering holds");
        model.place_solid(cube, &a_movement());
    }

    /// Only in debug, because the check this exercises is only there in debug.
    /// [`Model::check_touched_wire`] returns at once without `debug_assertions`
    /// — a full sweep per operation is what §4.3 keeps out of release — so in a
    /// release profile the mutation below goes unnoticed and there is no panic
    /// to expect.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "does not close")]
    fn a_mutation_notices_a_wire_that_no_longer_closes() {
        let (mut model, cube) = a_cube();
        model
            .data
            .wires
            .values_mut()
            .next()
            .expect("a cube has wires")
            .steps
            .swap(0, 1);

        model.place_solid(cube, &a_movement());
    }

    /// Debug only, for the reason
    /// [`a_mutation_notices_a_wire_that_no_longer_closes`] gives.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "exactly twice")]
    fn a_mutation_notices_an_edge_that_has_grown_a_third_coedge() {
        let (mut model, cube) = a_cube();
        let edge = model.data.edges.keys().next().expect("a cube has edges");
        model.add_coedge(edge, Orientation::Forward);

        model.place_solid(cube, &a_movement());
    }

    #[test]
    #[should_panic(expected = "only 0 was accounted for")]
    fn an_operation_that_leaves_more_slop_behind_than_it_found_is_a_bug() {
        let (mut model, _) = a_cube();
        let vertex = model.vertices().next().expect("a cube has corners").key();
        let before = model.worst_approximation();
        model
            .set_vertex_approximation(vertex, Approximation::new(1e-6, Cause::ImportedUncertainty))
            .expect("a vertex may carry slop");

        model.assert_approximation_did_not_grow(before);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "without being recorded")]
    fn slop_written_straight_into_an_entity_is_caught_by_the_next_operation() {
        let (mut model, cube) = a_cube();
        model
            .data
            .vertices
            .values_mut()
            .next()
            .expect("a cube has corners")
            .approximation = Some(Approximation::new(1e-6, Cause::ImportedUncertainty));

        model.place_solid(cube, &a_movement());
    }

    #[test]
    fn tagging_a_vertex_back_down_brings_the_model_s_worst_slop_down_with_it() {
        let (mut model, _) = a_cube();
        let vertex = model.vertices().next().expect("a cube has corners").key();
        model
            .set_vertex_approximation(vertex, Approximation::new(1e-6, Cause::ImportedUncertainty))
            .expect("a vertex may carry slop");
        assert_eq!(model.worst_approximation(), 1e-6);

        model
            .set_vertex_approximation(
                vertex,
                Approximation::new(1e-12, Cause::ImportedUncertainty),
            )
            .expect("and may be tagged again with less");

        assert_eq!(model.worst_approximation(), 1e-12);
    }

    /// The first of the four things a cache keyed on a model needs
    /// [`Model::which_model_this_is`] to promise (`docs/consumers.md`, rscene).
    #[test]
    fn two_models_built_one_after_the_other_are_not_the_same_model() {
        let (one, _) = a_cube();
        let (other, _) = a_cube();

        assert_ne!(one.which_model_this_is(), other.which_model_this_is());
    }

    /// The second, and the one the consumer is actually buying: work cached
    /// against a model has to survive the model being edited, or the cache is
    /// thrown away on every change and never earns its keep.
    #[test]
    fn editing_a_model_leaves_it_the_same_model() {
        let (mut model, cube) = a_cube();
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let before = model.which_model_this_is();

        let other = model
            .add_cube(2.0, &policy, &mut log)
            .expect("a second cube");
        model.place_solid(
            other,
            &rcad_geom::Isometry::translation(Vector::new(1.0, 1.0, 1.0)).expect("a finite shift"),
        );
        assert_eq!(
            model.which_model_this_is(),
            before,
            "building into a model does not make it another model"
        );

        model
            .fuse(&[cube], &[other], Rule::Union, &policy, &mut log)
            .expect("two cubes overlapping at a corner union");

        assert_eq!(
            model.which_model_this_is(),
            before,
            "and neither does a fuse"
        );
    }

    /// The third. A clone is a second model that can be edited on its own, so a
    /// cache built for the original is not built for it — and the keys the
    /// original handed out still address the clone's slots, which is why the
    /// per-key model id goes the other way and is copied across.
    #[test]
    fn a_clone_is_a_different_model_but_its_keys_still_work() {
        let (model, cube) = a_cube();

        let copy = model.clone();

        assert_ne!(copy.which_model_this_is(), model.which_model_this_is());
        assert_eq!(copy.solid(cube).signed_volume(), 8.0);
    }

    /// The fourth. Reading a file gives back a model that no cache was built
    /// from, however the bytes were produced.
    #[test]
    fn a_model_read_back_in_is_not_the_model_that_was_written_out() {
        let (model, _) = a_cube();
        let bytes = postcard::to_allocvec(&model).expect("a model writes out");

        let read_back: Model = postcard::from_bytes(&bytes).expect("and reads back");

        assert_ne!(read_back.which_model_this_is(), model.which_model_this_is());
    }

    #[test]
    fn one_corner_coming_down_leaves_the_worst_slop_where_the_other_corners_are() {
        let (mut model, _) = a_cube();
        let corners: Vec<_> = model.vertices().map(|vertex| vertex.key()).collect();
        for &corner in &corners {
            model
                .set_vertex_approximation(
                    corner,
                    Approximation::new(1e-6, Cause::ImportedUncertainty),
                )
                .expect("a vertex may carry slop");
        }

        model
            .set_vertex_approximation(
                corners[0],
                Approximation::new(1e-12, Cause::ImportedUncertainty),
            )
            .expect("and may be tagged again with less");

        assert_eq!(model.worst_approximation(), 1e-6);
    }
}
