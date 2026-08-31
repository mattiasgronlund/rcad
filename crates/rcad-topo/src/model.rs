use rcad_geom::{Isometry, Orientation, Point, Scale, Surface};
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
#[derive(Clone, Debug)]
pub struct Model {
    id: ModelId,
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
            data: ModelData::new(scale),
            index: Index::default(),
            worst_approximation: 0.0,
        }
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
    /// If the key came from another model (debug builds), or names an entity
    /// that is not there.
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
    /// If the key came from another model (debug builds), or names an entity
    /// that is not there.
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
    /// If the key came from another model (debug builds), or names an entity
    /// that is not there.
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
    /// If the key came from another model (debug builds), or names an entity
    /// that is not there.
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
    /// If the key came from another model (debug builds), or names an entity
    /// that is not there.
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
    /// If the key came from another model (debug builds), or names an entity
    /// that is not there.
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
    /// If the key came from another model (debug builds), or names an entity
    /// that is not there.
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
    /// meet along it.
    ///
    /// # Panics
    ///
    /// If the key came from another model.
    pub fn set_edge_approximation(&mut self, key: EdgeKey, tag: Approximation) -> Result<()> {
        let slot = key.slot_in(self.id);
        let edge = &self.data.edges[slot];
        for end in [edge.start, edge.end] {
            let on_vertex = magnitude_of(self.data.vertices[end].approximation);
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
    /// of the face's own edges carries.
    ///
    /// # Panics
    ///
    /// If the key came from another model.
    pub fn set_face_approximation(&mut self, key: FaceKey, tag: Approximation) -> Result<()> {
        let slot = key.slot_in(self.id);
        for edge in self.edges_of_face(slot) {
            let on_edge = magnitude_of(self.data.edges[edge].approximation);
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
        let slot = self.data.edges.insert(EdgeData {
            start,
            end,
            approximation: None,
        });
        self.index.coedges_of_edge.insert(slot, SmallVec::new());
        self.index.edges_of_vertex[start].push(slot);
        self.index.edges_of_vertex[end].push(slot);
        slot
    }

    pub(crate) fn add_coedge(&mut self, edge: EdgeSlot, orientation: Orientation) -> CoedgeSlot {
        let slot = self.data.coedges.insert(CoedgeData { edge, orientation });
        self.index.coedges_of_edge[edge].push(slot);
        slot
    }

    pub(crate) fn add_wire(&mut self, coedges: SmallVec<[CoedgeSlot; 4]>) -> WireSlot {
        let slot = self.data.wires.insert(WireData {
            coedges: coedges.clone(),
        });
        for coedge in coedges {
            self.index.wire_of_coedge.insert(coedge, slot);
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
        assert!(
            wire.coedges.len() >= 3,
            "a closed wire needs at least three coedges, this one has {}",
            wire.coedges.len()
        );
        let mut previous_end = None;
        let mut first_start = None;
        for &coedge in &wire.coedges {
            let (start, end) = self.coedge_ends(coedge);
            if let Some(previous) = previous_end {
                assert!(start == previous, "a touched wire does not close");
            } else {
                first_start = Some(start);
            }
            previous_end = Some(end);
            self.check_touched_edge(self.data.coedges[coedge].edge);
        }
        assert!(
            previous_end == first_start,
            "a touched wire does not close back to where it started"
        );
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
        let coedges = self.index.coedges_of_edge(slot);
        assert!(
            coedges.len() == 2,
            "a closed solid uses every edge exactly twice, this one {} times",
            coedges.len()
        );
        assert!(
            self.data.coedges[coedges[0]].orientation != self.data.coedges[coedges[1]].orientation,
            "the two coedges of an edge must run opposite ways"
        );
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

    fn edges_of_face(&self, face: FaceSlot) -> Vec<EdgeSlot> {
        self.data.faces[face]
            .wires
            .iter()
            .flat_map(|&wire| self.data.wires[wire].coedges.iter())
            .map(|&coedge| self.data.coedges[coedge].edge)
            .collect()
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
            for &coedge in &wire.coedges {
                index.wire_of_coedge.insert(coedge, slot);
            }
        }
        for (slot, face) in &data.faces {
            for &wire in &face.wires {
                index.face_of_wire.insert(wire, slot);
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

/// A straight edge between two vertices.
///
/// There is no curve stored here. M0's edges are straight (decisions.md §3.1),
/// so the geometry is exactly the two endpoints and storing it twice would only
/// create a way for the two copies to disagree.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct EdgeData {
    pub start: VertexSlot,
    pub end: VertexSlot,
    pub approximation: Option<Approximation>,
}

/// One use of an edge by one wire, running one way round.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct CoedgeData {
    pub edge: EdgeSlot,
    pub orientation: Orientation,
}

/// A closed chain of coedges.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WireData {
    pub coedges: SmallVec<[CoedgeSlot; 4]>,
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
    use crate::{Error, Invalid, SolidKey};

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

    #[test]
    fn a_location_on_a_shell_and_one_on_a_face_compose() {
        let (mut model, cube) = a_cube();
        let on_the_face =
            rcad_geom::Isometry::translation(Vector::new(5.0, -2.0, 1.0)).expect("a finite shift");
        for shell in model.data.shells.values_mut() {
            for used in &mut shell.faces {
                used.loc = on_the_face;
            }
        }
        let stored: Vec<_> = model.faces().map(crate::FaceView::plane).collect();

        let on_the_shell = a_movement();
        model.place_solid(cube, &on_the_shell);

        let both = on_the_shell.compose(&on_the_face);
        for (before, face) in stored.iter().zip(model.solid(cube).faces()) {
            assert_eq!(face.plane(), before.transformed(&both));
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
            .coedges
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
