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
    /// # Errors
    ///
    /// [`Error::KeyFromAnotherModel`] if the key was made by a different model.
    /// It reads nothing before it says so, so the caller's model is untouched
    /// (`docs/decisions.md` §86). That answer used to be a panic, and before
    /// §84 it was no answer at all in a release build — the model id was
    /// zero-sized there, so two ids compared equal whichever models they came
    /// from and the key resolved to whatever sat in that slot.
    ///
    /// # Panics
    ///
    /// If the key is this model's own and names nothing. **A key this model
    /// handed out always names something**, which is what makes that an
    /// internal invariant and not a bad-input case. Nothing in the crate
    /// removes an entity from an arena outside a test, and a file whose stored
    /// slots resolve to nothing never becomes a `Model` at all — `Deserialize`
    /// refuses it before the index is built, demonstrated by
    /// `a_file_naming_a_corner_that_is_not_there_never_becomes_a_model` in
    /// [`serialize`](crate::serialize).
    pub fn solid(&self, key: SolidKey) -> Result<SolidView<'_>> {
        let slot = key.try_slot_in(self.id)?;
        assert!(self.data.solids.contains_key(slot), "no such solid");
        Ok(SolidView::new(
            self,
            slot,
            Isometry::identity(),
            Orientation::Forward,
        ))
    }

    /// A view of one shell, in the model's own frame.
    ///
    /// # Errors
    ///
    /// [`Error::KeyFromAnotherModel`], the same way as [`Model::solid`].
    ///
    /// # Panics
    ///
    /// If the key is this model's own and names nothing, the same way and with
    /// the same demonstration as [`Model::solid`].
    pub fn shell(&self, key: ShellKey) -> Result<ShellView<'_>> {
        let slot = key.try_slot_in(self.id)?;
        assert!(self.data.shells.contains_key(slot), "no such shell");
        Ok(ShellView::new(
            self,
            slot,
            Isometry::identity(),
            Orientation::Forward,
        ))
    }

    /// A view of one face, in the model's own frame.
    ///
    /// A face reached through a solid carries the location and orientation
    /// composed down from that solid instead. This one is the face as stored,
    /// which is what you want for tagging it or reading its own plane.
    ///
    /// # Errors
    ///
    /// [`Error::KeyFromAnotherModel`], the same way as [`Model::solid`].
    ///
    /// # Panics
    ///
    /// If the key is this model's own and names nothing, the same way and with
    /// the same demonstration as [`Model::solid`].
    pub fn face(&self, key: FaceKey) -> Result<FaceView<'_>> {
        let slot = key.try_slot_in(self.id)?;
        assert!(self.data.faces.contains_key(slot), "no such face");
        Ok(FaceView::new(
            self,
            slot,
            Isometry::identity(),
            Orientation::Forward,
        ))
    }

    /// A view of one wire, in the model's own frame.
    ///
    /// # Errors
    ///
    /// [`Error::KeyFromAnotherModel`], the same way as [`Model::solid`].
    ///
    /// # Panics
    ///
    /// If the key is this model's own and names nothing, the same way and with
    /// the same demonstration as [`Model::solid`].
    pub fn wire(&self, key: WireKey) -> Result<WireView<'_>> {
        let slot = key.try_slot_in(self.id)?;
        assert!(self.data.wires.contains_key(slot), "no such wire");
        Ok(WireView::new(
            self,
            slot,
            Isometry::identity(),
            Orientation::Forward,
        ))
    }

    /// A view of one coedge, in the model's own frame.
    ///
    /// # Errors
    ///
    /// [`Error::KeyFromAnotherModel`], the same way as [`Model::solid`].
    ///
    /// # Panics
    ///
    /// If the key is this model's own and names nothing, the same way and with
    /// the same demonstration as [`Model::solid`].
    pub fn coedge(&self, key: CoedgeKey) -> Result<CoedgeView<'_>> {
        let slot = key.try_slot_in(self.id)?;
        assert!(self.data.coedges.contains_key(slot), "no such coedge");
        Ok(CoedgeView::new(
            self,
            slot,
            Isometry::identity(),
            Orientation::Forward,
        ))
    }

    /// A view of one edge, in the model's own frame.
    ///
    /// # Errors
    ///
    /// [`Error::KeyFromAnotherModel`], the same way as [`Model::solid`].
    ///
    /// # Panics
    ///
    /// If the key is this model's own and names nothing, the same way and with
    /// the same demonstration as [`Model::solid`].
    pub fn edge(&self, key: EdgeKey) -> Result<EdgeView<'_>> {
        let slot = key.try_slot_in(self.id)?;
        assert!(self.data.edges.contains_key(slot), "no such edge");
        Ok(EdgeView::new(
            self,
            slot,
            Isometry::identity(),
            Orientation::Forward,
        ))
    }

    /// A view of one vertex, in the model's own frame.
    ///
    /// # Errors
    ///
    /// [`Error::KeyFromAnotherModel`], the same way as [`Model::solid`].
    ///
    /// # Panics
    ///
    /// If the key is this model's own and names nothing, the same way and with
    /// the same demonstration as [`Model::solid`].
    pub fn vertex(&self, key: VertexKey) -> Result<VertexView<'_>> {
        let slot = key.try_slot_in(self.id)?;
        assert!(self.data.vertices.contains_key(slot), "no such vertex");
        Ok(VertexView::new(self, slot, Isometry::identity()))
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
    /// # Errors
    ///
    /// [`Error::KeyFromAnotherModel`] if the key was made by a different model.
    /// Nothing is moved before that is answered, so the model is left as it was
    /// (`docs/decisions.md` §86).
    ///
    /// # Panics
    ///
    /// If the mutation leaves the solid broken. By then the movement is already
    /// composed onto the shells and this call cannot take it back, which is the
    /// case §4.1's amendment under §66 keeps as a panic.
    pub fn place_solid(&mut self, key: SolidKey, movement: &Isometry) -> Result<()> {
        let slot = key.try_slot_in(self.id)?;
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
        Ok(())
    }

    /// A second solid in this model, the same shape as the one `key` names and
    /// sharing nothing with it, so that moving, fusing or tagging either one
    /// leaves the other as it was.
    ///
    /// **The copy goes all the way down to the corners, and has to.** A shell
    /// belongs to one solid and a face to one shell — `Deserialize` and
    /// [`Model::validate`] both refuse a model where one is used twice. An edge
    /// is walked by exactly two coedges running opposite ways, so a copied face
    /// walking the original's edges would give each of them a third and a
    /// fourth. And a corner the two solids shared would be tagged in both by one
    /// [`Model::set_vertex_approximation`]. So every vertex, edge, coedge, wire,
    /// face and shell under the solid is written again, carrying its point, run,
    /// surface, placement, orientation and approximation over bit for bit. The
    /// shells are used at the placements the original uses them at, so the copy
    /// sits exactly where the original does until one of them is moved.
    ///
    /// **Each level is written in the order of the original's slots.** With no
    /// empty slot in an arena, a new entity lands after every entity already
    /// there, so the copy's entities sort the way the originals they came from
    /// sort, and the lists the model keeps of the edges at a corner and the
    /// coedges along an edge come out as the original's with the names changed.
    /// A walk down from the copy then visits everything in the order the same
    /// walk down from the original does, which is what makes the copy measure
    /// the same as the original bit for bit. Nothing in this crate removes an
    /// entity, so every model it builds has no empty slot. A model read from a
    /// file can: the file lists each arena's slots, empty ones included, and
    /// the next entities written go into those first. The copy is then still
    /// the same shape and still shares nothing, but its entities need not sort
    /// the way the originals do —
    /// `a_copy_written_into_empty_slots_does_not_sort_the_way_its_original_does`
    /// is one that does not — and a bit-for-bit measure is not promised.
    ///
    /// The old-to-new lookup is a binary search over what was copied rather
    /// than a map as long as the arena, so a copy costs what the solid holds
    /// and not what the model holds (decisions.md §4.3).
    /// [`Model::which_model_this_is`] does not change: this is the same model
    /// with one more solid in it.
    ///
    /// **Every edge the solid reaches is walked from inside it alone**, so the
    /// copy's edges are each walked twice. Both coedges of an edge are in one
    /// shell ([`Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells`]) and a shell
    /// is in one solid. A file whose skin closes only together with another
    /// solid's is refused at the door, which
    /// `a_solid_whose_skin_only_closes_with_another_never_reaches_a_copy`
    /// shows on the model that used to reach a refusal here
    /// (`docs/decisions.md` §342).
    ///
    /// # Errors
    ///
    /// [`Error::KeyFromAnotherModel`] if the key was made by a different model,
    /// answered before anything is written, so the model is left as it was
    /// (`docs/decisions.md` §86).
    ///
    /// # Panics
    ///
    /// If the copy breaks a check run on a touched solid, or leaves more slop
    /// in the model than there was. Every tag it writes is one the model
    /// already held, so either is a bug here.
    pub fn copy_solid(&mut self, key: SolidKey) -> Result<SolidKey> {
        let slot = key.try_slot_in(self.id)?;
        let approximation_before = self.worst_approximation;

        let solid = self.data.solids.get(slot).expect("no such solid to copy");
        let shells: Vec<ShellSlot> = solid.shells.iter().map(|used| used.shell).collect();
        let faces: Vec<FaceSlot> = shells
            .iter()
            .flat_map(|&shell| self.data.shells[shell].faces.iter().map(|used| used.face))
            .collect();
        let wires: Vec<WireSlot> = faces
            .iter()
            .flat_map(|&face| self.data.faces[face].wires.iter().copied())
            .collect();
        let mut coedges: Vec<CoedgeSlot> = Vec::new();
        let mut vertices: Vec<VertexSlot> = Vec::new();
        for &wire in &wires {
            for &step in &self.data.wires[wire].steps {
                match step {
                    WireStep::Along(coedge) => coedges.push(coedge),
                    WireStep::Pole(vertex) => vertices.push(vertex),
                }
            }
        }
        coedges.sort_unstable();
        coedges.dedup();
        let edges: Vec<EdgeSlot> = coedges
            .iter()
            .map(|&coedge| self.data.coedges[coedge].edge)
            .collect();
        for &edge in &edges {
            let edge = &self.data.edges[edge];
            vertices.extend([edge.start, edge.end]);
        }

        let vertices = Renaming::copying(vertices, |vertex| {
            let VertexData {
                point,
                approximation,
            } = self.data.vertices[vertex];
            let copy = self.add_vertex(point);
            self.data.vertices[copy].approximation = approximation;
            copy
        });
        let edges = Renaming::copying(edges, |edge| {
            let EdgeData {
                start,
                end,
                ref run,
                approximation,
            } = self.data.edges[edge];
            let run = run.clone();
            let copy = self.add_edge_running(vertices.of(start), vertices.of(end), run);
            self.data.edges[copy].approximation = approximation;
            copy
        });
        let coedges = Renaming::copying(coedges, |coedge| {
            let CoedgeData { edge, orientation } = self.data.coedges[coedge];
            self.add_coedge(edges.of(edge), orientation)
        });
        let wires = Renaming::copying(wires, |wire| {
            let steps = self.data.wires[wire]
                .steps
                .iter()
                .map(|&step| match step {
                    WireStep::Along(coedge) => WireStep::Along(coedges.of(coedge)),
                    WireStep::Pole(vertex) => WireStep::Pole(vertices.of(vertex)),
                })
                .collect();
            self.add_wire_of_steps(steps)
        });
        let faces = Renaming::copying(faces, |face| {
            let face = &self.data.faces[face];
            let (surface, approximation) = (face.surface, face.approximation);
            let bounded_by = face.wires.iter().map(|&wire| wires.of(wire)).collect();
            let copy = self.add_face(surface, bounded_by);
            self.data.faces[copy].approximation = approximation;
            copy
        });
        let shells = Renaming::copying(shells, |shell| {
            let used = self.data.shells[shell]
                .faces
                .iter()
                .map(|used| FaceUse {
                    face: faces.of(used.face),
                    ..*used
                })
                .collect();
            self.add_shell(used)
        });
        let used = self.data.solids[slot]
            .shells
            .iter()
            .map(|used| ShellUse {
                shell: shells.of(used.shell),
                ..*used
            })
            .collect();
        let copy = self.add_solid(used);

        self.check_touched_solid(copy);
        self.assert_approximation_did_not_grow(approximation_before);
        Ok(self.key_for_solid(copy))
    }

    /// Tag a vertex with the slop a decision put into it.
    ///
    /// # Errors
    ///
    /// [`Invalid::ApproximationOrdering`] if the tag would be smaller than one
    /// of the edges built on this vertex. A point cannot be known better than
    /// the edges through it (roadmap §5.4).
    /// [`Error::KeyFromAnotherModel`] if the key was made by a different model.
    pub fn set_vertex_approximation(&mut self, key: VertexKey, tag: Approximation) -> Result<()> {
        let slot = key.try_slot_in(self.id)?;
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
    /// meet along it. [`Error::KeyFromAnotherModel`] if the key was made by a
    /// different model.
    ///
    /// # Panics
    ///
    /// If the edge's own stored ends do not resolve. Nothing in the public
    /// API removes an entity from an arena, and every door that fills one —
    /// `Deserialize`, the builder, a fuse — resolves its keys before a
    /// `Model` comes out of it, so a dangling end here is a broken invariant
    /// and not bad input.
    pub fn set_edge_approximation(&mut self, key: EdgeKey, tag: Approximation) -> Result<()> {
        let slot = key.try_slot_in(self.id)?;
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
    /// of the face's own edges carries. [`Error::KeyFromAnotherModel`] if the
    /// key was made by a different model.
    ///
    /// # Panics
    ///
    /// If the face's own stored wires, coedges or edges do not resolve, the
    /// same way and for the same reason as
    /// [`Model::set_edge_approximation`]'s own broken-invariant panic.
    pub fn set_face_approximation(&mut self, key: FaceKey, tag: Approximation) -> Result<()> {
        let slot = key.try_slot_in(self.id)?;
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
        let slot = self.data.shells.insert(ShellData { faces });
        for used in &self.data.shells[slot].faces {
            self.index.shell_of_face.insert(used.face, slot);
        }
        slot
    }

    pub(crate) fn add_solid(&mut self, shells: SmallVec<[ShellUse; 1]>) -> SolidSlot {
        self.data.solids.insert(SolidData { shells })
    }

    pub(crate) fn key_for_solid(&self, slot: SolidSlot) -> SolidKey {
        SolidKey::new(slot, self.id)
    }

    /// A view of one solid, for a slot this crate already holds.
    ///
    /// [`Model::solid`] answers `Err` for a key another model made, which is
    /// the only thing it can say about data a caller chose. A slot is not that
    /// data — a `*Slot` is `pub(crate)` and every one of them came out of this
    /// model's own arena — so these four doors take the question away rather
    /// than answering it, and the inside of the crate uses them
    /// (decisions.md §86).
    pub(crate) fn solid_at(&self, slot: SolidSlot) -> SolidView<'_> {
        assert!(self.data.solids.contains_key(slot), "no such solid");
        SolidView::new(self, slot, Isometry::identity(), Orientation::Forward)
    }

    /// A view of one face, in the model's own frame, for a slot this crate
    /// already holds. The same door as [`Model::solid_at`].
    pub(crate) fn face_at(&self, slot: FaceSlot) -> FaceView<'_> {
        assert!(self.data.faces.contains_key(slot), "no such face");
        FaceView::new(self, slot, Isometry::identity(), Orientation::Forward)
    }

    /// A view of one edge, in the model's own frame, for a slot this crate
    /// already holds. The same door as [`Model::solid_at`].
    pub(crate) fn edge_at(&self, slot: EdgeSlot) -> EdgeView<'_> {
        assert!(self.data.edges.contains_key(slot), "no such edge");
        EdgeView::new(self, slot, Isometry::identity(), Orientation::Forward)
    }

    /// A view of one vertex, in the model's own frame, for a slot this crate
    /// already holds. The same door as [`Model::solid_at`].
    pub(crate) fn vertex_at(&self, slot: VertexSlot) -> VertexView<'_> {
        assert!(self.data.vertices.contains_key(slot), "no such vertex");
        VertexView::new(self, slot, Isometry::identity())
    }

    /// The cheap checks that run after every mutation, on the entities that
    /// mutation touched and no others.
    ///
    /// A full [`Model::validate`] here would make a test that performs *n*
    /// operations cost O(n²) and the suite stops being runnable long before the
    /// milestone that needs it (decisions.md §4.3). These walk one solid rather
    /// than the whole model, which is what keeps them out of that.
    ///
    /// **They run in every build** (decisions.md §66.3). They ran in debug
    /// builds only until then, and the shape that decided it is worth keeping:
    /// a file holding a wire that does not close read back cleanly, and
    /// [`Model::place_solid`] on it crashed a debug-built caller and said
    /// nothing to a release-built one — the same bytes, two answers, decided by
    /// how the caller happened to be compiled.
    /// [`Model::check_shape_is_possible`] shut that door for deserialized
    /// shapes, but not for a broken invariant this crate writes itself, and for
    /// those the gate left a release build with neither of §4.1's two answers:
    /// no panic, no `Err`, and a solid that does not validate handed back as
    /// `Ok`. The cost §4.3 gated them for is the full sweep's and was never
    /// measured for these; measured, it is 50–65 ns per face of the solid
    /// handed in (decisions.md §66.2).
    ///
    /// # Panics
    ///
    /// If any of them fail, in every build. Breaking one of these is a bug in
    /// this crate, and a corrupt solid is worse than a crash. Whether a release
    /// build should instead record and return an internal error is a separate
    /// question, still open in decisions.md §66.4.
    pub(crate) fn check_touched_solid(&self, slot: SolidSlot) {
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
        let face = self.data.faces.get(slot).expect("touched face is gone");
        assert!(!face.wires.is_empty(), "a face with no wires");
        for &wire in &face.wires {
            self.check_touched_wire(wire);
        }
    }

    pub(crate) fn check_touched_wire(&self, slot: WireSlot) {
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
        matches!(
            self.data.edges[self.data.coedges[only].edge].run,
            EdgeRun::RightRoundWhereItsFacesCross | EdgeRun::RightRoundWhereItCrosses { .. }
        )
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

    /// That both ends are there, that two coedges walk the edge opposite ways,
    /// and that something curved along it says where it bends.
    ///
    /// **It does not ask whether the flat faces along it put it in one place**
    /// (`check_flat_faces_put_an_edge_in_one_place`, Q574, Q604), and it has no
    /// need to (Q614). Every mutation that ends here is one of three kinds:
    ///
    /// - [`Model::place_solid`] and [`Model::copy_solid`] write no plane, no
    ///   corner and no run: a movement goes on the shell, and a copy is the
    ///   original bit for bit. An edge that check refuses reaches them only in a
    ///   solid that already held it, and the one door such a solid comes in by
    ///   is `Deserialize`, which asks nothing that needs arithmetic
    ///   (decisions.md §44.5). That is the caller's input, which is an `Err` and
    ///   not a panic, and [`Model::validate`] gives the `Err`.
    /// - The kernel's own solids — [`Model::add_cube`], [`Model::add_extrusion`],
    ///   [`Model::add_round_bar`], [`Model::add_ball`] and
    ///   [`Model::add_bar_cut_off_by`] — have no edge that check asks about with
    ///   only flat faces along it. Every run naming a crossing they write has a
    ///   pipe or a ball beside it.
    /// - [`SolidBuilder::build`](crate::SolidBuilder::build) and a fuse sweep
    ///   what they will write in a model of their own first, that check among
    ///   the sweep's, with the room of the solid or the answer itself, and write
    ///   nothing when it refuses.
    ///
    /// `tests/a_closed_edge_between_two_planes_its_curve_leaves.rs` holds each
    /// kind: `moving_or_copying_a_tilted_piece_read_back_from_bytes_leaves_it_for_the_sweep`,
    /// `no_solid_the_kernel_makes_has_a_named_crossing_between_flat_faces_alone`,
    /// `a_disc_tilted_off_the_lid_is_refused_whichever_face_is_listed_first` and
    /// `a_fuse_writes_no_piece_of_a_lid_off_the_lid_into_the_callers_model`.
    pub(crate) fn check_touched_edge(&self, slot: EdgeSlot) {
        let edge = self.data.edges.get(slot).expect("touched edge is gone");
        assert!(
            self.data.vertices.contains_key(edge.start)
                && self.data.vertices.contains_key(edge.end),
            "a touched edge names a vertex that is gone"
        );
        self.check_edge_is_walked_by_two_opposite_coedges(slot)
            .unwrap_or_else(|broken| panic!("a touched edge is broken: {broken}"));
        self.check_something_curved_says_where_this_edge_bends(slot)
            .unwrap_or_else(|broken| panic!("a touched edge is broken: {broken}"));
    }

    /// The slop check §4.3 budgets for every build — as, since §66.3, the
    /// touched checks above are.
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
    /// # Panics
    ///
    /// If one of the face's own stored wires or coedges does not resolve.
    /// Nothing in the public API removes an entity from an arena, and every
    /// door that fills one — `Deserialize`, the builder, a fuse — resolves
    /// its keys before a `Model` comes out of it, so a dangling wire or
    /// coedge here is a broken invariant and not bad input.
    fn edges_of_face(&self, face: FaceSlot) -> Vec<EdgeSlot> {
        let mut edges = Vec::new();
        for &wire in &self.data.faces[face].wires {
            let wire = &self.data.wires[wire];
            for coedge in wire.coedges() {
                edges.push(self.data.coedges[coedge].edge);
            }
        }
        edges
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
/// That one survives a clone: a clone's keys still address the original's
/// slots, so the copied id has to keep matching them. This one does not,
/// because the clone is a second model that a cache built from the original
/// was not built from.
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

/// Which entity of a copy each entity of the original became, for one level
/// of [`Model::copy_solid`].
///
/// Two lists side by side and a binary search, rather than a `SecondaryMap`:
/// a `SecondaryMap` grows to the length of the arena, and a copy has to cost
/// what the solid holds.
struct Renaming<K> {
    originals: Vec<K>,
    copies: Vec<K>,
}

impl<K: Ord + Copy + core::fmt::Debug> Renaming<K> {
    /// Copy each of `originals` once, in slot order, with `copy`.
    fn copying(mut originals: Vec<K>, mut copy: impl FnMut(K) -> K) -> Self {
        originals.sort_unstable();
        originals.dedup();
        let copies = originals.iter().map(|&original| copy(original)).collect();
        Self { originals, copies }
    }

    /// # Panics
    ///
    /// If `original` was not copied. Every level is gathered from the one
    /// above it before anything is written, so that is a bug in
    /// [`Model::copy_solid`].
    fn of(&self, original: K) -> K {
        let at = self
            .originals
            .binary_search(&original)
            .unwrap_or_else(|_| panic!("{original:?} is under the solid and was not copied"));
        self.copies[at]
    }
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
    shell_of_face: SecondaryMap<FaceSlot, ShellSlot>,
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

    pub(crate) fn shell_of_face(&self, face: FaceSlot) -> Option<ShellSlot> {
        self.shell_of_face.get(face).copied()
    }

    /// The upward links read back off the downward ones.
    ///
    /// **The three single-valued links assert rather than overwrite, and the
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
    /// per coedge, per wire and per face, beside an insert that already writes
    /// a word.
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
        for (slot, shell) in &data.shells {
            for used in &shell.faces {
                let already = index.shell_of_face.insert(used.face, slot);
                assert!(
                    already.is_none(),
                    "one face is in two shells, {already:?} and {slot:?}"
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

    use super::{Model, WireStep};
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
        model
            .place_solid(cube, &movement)
            .expect("this model made the key");

        let moved: Vec<_> = model
            .solid(cube)
            .expect("this model made the key")
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
            .expect("this model made the key")
            .faces()
            .map(crate::FaceView::outward_normal)
            .collect();
        let volume = model
            .solid(cube)
            .expect("this model made the key")
            .signed_volume();

        for shell in model.data.shells.values_mut() {
            for used in &mut shell.faces {
                used.orient = used.orient.reversed();
            }
        }

        let turned = model.solid(cube).expect("this model made the key");
        for (before, face) in outward.iter().zip(turned.faces()) {
            assert_eq!(face.outward_normal(), -*before);
        }
        assert_eq!(turned.signed_volume(), -volume);
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

    /// `measure_one_face`'s doc comment says it hands back whatever
    /// `Model::validate` refuses for a face with no area, rather than the panic
    /// it takes for a face `validate` would have accepted. This puts a cube's
    /// first face through exactly that.
    #[test]
    fn measuring_a_face_with_no_area_gives_validates_refusal() {
        let (mut model, _) = a_cube();

        let target = model.data.faces.keys().next().expect("a cube has faces");
        let wire = model.data.faces[target].wires[0];
        let corners: Vec<_> = model.data.wires[wire]
            .steps
            .iter()
            .map(|step| match *step {
                WireStep::Along(coedge) => {
                    let coedge = &model.data.coedges[coedge];
                    let edge = &model.data.edges[coedge.edge];
                    if coedge.orientation.is_reversed() {
                        edge.end
                    } else {
                        edge.start
                    }
                }
                WireStep::Pole(vertex) => vertex,
            })
            .collect();
        assert_eq!(corners.len(), 4, "a cube's face is a quadrilateral");

        // Leave the first two corners alone and put the other two on the line
        // through them, so the wire sweeps no area at all.
        let first = model.data.vertices[corners[0]].point;
        let second = model.data.vertices[corners[1]].point;
        let along = second - first;
        model.data.vertices[corners[2]].point = second + along;
        model.data.vertices[corners[3]].point = first - along;

        let bytes = postcard::to_allocvec(&model).expect("a model writes out");
        let read_back: Model = postcard::from_bytes(&bytes).expect("and reads back");

        let swept = read_back
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect_err("a face with no area is not a sound model");
        assert!(
            matches!(
                swept,
                Error::Invalid(Invalid::OuterWireWindsAgainstTheNormal)
            ),
            "validate gave {swept:?}"
        );

        let solid = read_back.solids().next().expect("the cube came back");
        let moved_to = second + along;
        let face = solid
            .faces()
            .find(|face| face.outer_wire().points().any(|point| point == moved_to))
            .expect("the face whose corners were moved came back too")
            .key();

        let refused = solid
            .measure_one_face(
                face,
                rcad_geom::Deflection::of(0.1).expect("a length a polyhedron ignores"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a face with no area has no measurement");
        assert!(
            matches!(
                refused,
                Error::Invalid(Invalid::OuterWireWindsAgainstTheNormal)
            ),
            "measure_one_face gave {refused:?}"
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
            .expect("this model made the key")
            .faces()
            .map(|face| face.outer_wire().points().collect())
            .collect();

        for shell in model.data.shells.values_mut() {
            for used in &mut shell.faces {
                used.orient = used.orient.reversed();
            }
        }

        let turned = model.solid(cube).expect("this model made the key");
        for (forwards, face) in one_way.iter().zip(turned.faces()) {
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
        model
            .place_solid(cube, &movement)
            .expect("this model made the key");

        let placed = model.solid(cube).expect("this model made the key");
        let face = placed.faces().next().expect("a cube has faces");
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

    /// The whole suite for this lives in
    /// `tests/a_key_from_another_model.rs`, which asks it of all eleven public
    /// doors and of a key of every kind. This one is here so that the crate's
    /// own tests fail too if the id ever stops being compared.
    #[test]
    fn a_key_from_one_model_may_not_be_used_on_another() {
        let (_first, cube) = a_cube();
        let (second, _) = a_cube();

        let refused = second.solid(cube).map(|_| ());

        assert!(
            matches!(refused, Err(Error::KeyFromAnotherModel { entity }) if entity == "solid"),
            "expected a foreign-key refusal, got {refused:?}"
        );
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

    /// `data_mut` is the one door round the invariant (test-only, Q84): no
    /// public method removes an entity from an arena, so this breaks the
    /// model by hand to prove `set_edge_approximation` panics rather than
    /// silently trusting a dangling end.
    #[test]
    #[should_panic(expected = "invalid SlotMap key used")]
    fn tagging_an_edge_whose_stored_end_is_gone_panics() {
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

        let _ = model
            .set_edge_approximation(edge, Approximation::new(1e-6, Cause::ImportedUncertainty));
    }

    /// The same case one arena further out: a face's stored wire can point at
    /// a wire the arenas do not hold.
    #[test]
    #[should_panic(expected = "invalid SlotMap key used")]
    fn tagging_a_face_whose_stored_wire_is_gone_panics() {
        let (mut model, _) = a_cube();
        let face = model.data().faces.keys().next().expect("a cube has faces");
        let wire = model.data().faces[face].wires[0];
        let face = crate::FaceKey::new(face, model.id());
        model.data_mut().wires.remove(wire);

        let _ = model
            .set_face_approximation(face, Approximation::new(1e-6, Cause::ImportedUncertainty));
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
        model
            .place_solid(cube, &a_movement())
            .expect("this model made the key");
    }

    /// In every build, because [`Model::check_touched_wire`] runs in every
    /// build (decisions.md §66.3). Until then this test was `#[cfg(debug_assertions)]`
    /// and a release profile let the mutation below through unnoticed.
    #[test]
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

        model
            .place_solid(cube, &a_movement())
            .expect("this model made the key");
    }

    /// Every build, for the reason
    /// [`a_mutation_notices_a_wire_that_no_longer_closes`] gives.
    #[test]
    #[should_panic(expected = "exactly twice")]
    fn a_mutation_notices_an_edge_that_has_grown_a_third_coedge() {
        let (mut model, cube) = a_cube();
        let edge = model.data.edges.keys().next().expect("a cube has edges");
        model.add_coedge(edge, Orientation::Forward);

        model
            .place_solid(cube, &a_movement())
            .expect("this model made the key");
    }

    /// Every build, for the reason
    /// [`a_mutation_notices_a_wire_that_no_longer_closes`] gives, and by the
    /// rule the sweep refuses the same edge by (`docs/decisions.md` §187.1).
    ///
    /// A round bar with its wall swapped for a plane leaves each cap circle
    /// between two flat faces, so nothing in the model says where it bends.
    #[test]
    #[should_panic(expected = "a closed edge is walked between two flat faces")]
    fn a_mutation_notices_an_edge_that_bends_with_nothing_curved_along_it() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = model
            .add_round_bar(5.0, 10.0, &rcad_geom::Isometry::identity())
            .expect("a bar five across and ten tall");
        let wall = model
            .data
            .faces
            .values_mut()
            .find(|face| face.surface.as_plane().is_none())
            .expect("a bar has one curved face");
        wall.surface = rcad_geom::Surface::Plane(
            rcad_geom::Plane::from_point_and_normal(
                Point::new(5.0, 0.0, 0.0),
                Vector::new(1.0, 0.0, 0.0),
            )
            .expect("the x axis is a unit normal"),
        );

        model
            .place_solid(bar, &a_movement())
            .expect("this model made the key");
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

        model
            .place_solid(cube, &a_movement())
            .expect("this model made the key");
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
        model
            .place_solid(
                other,
                &rcad_geom::Isometry::translation(Vector::new(1.0, 1.0, 1.0))
                    .expect("a finite shift"),
            )
            .expect("this model made the key");
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
        assert_eq!(
            copy.solid(cube)
                .expect("a clone's keys address the same slots")
                .signed_volume(),
            8.0
        );
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

    /// A cube's six faces split three and three between two solids: each
    /// skin is open, and the edges round the cut are walked once from each
    /// side. The door used to let that in, because every edge in the file
    /// still has two coedges running opposite ways, and a copy of one half
    /// refused with `EdgeCoedgeCount { count: 1 }` rather than write an edge
    /// walked once. This is the model that reached that refusal. The door
    /// refuses its bytes now, so no caller holds a model to copy a half out
    /// of, and `copy_solid` asks nothing before it writes (`docs/decisions.md`
    /// §342).
    #[test]
    fn a_solid_whose_skin_only_closes_with_another_never_reaches_a_copy() {
        let (mut model, cube) = a_cube();
        let shell = model.data().solids[cube.slot_in(model.id())].shells[0].shell;
        let other_half = model.data_mut().shells[shell].faces.split_off(3);
        let other_shell = model.add_shell(other_half);
        model.add_solid(smallvec::smallvec![super::ShellUse {
            shell: other_shell,
            loc: rcad_geom::Isometry::identity(),
            orient: Orientation::Forward,
        }]);
        let bytes = postcard::to_allocvec(&model).expect("a model writes out");

        assert_eq!(
            postcard::from_bytes::<Model>(&bytes).err(),
            Some(postcard::Error::SerdeDeCustom),
            "the two halves read back"
        );
        assert!(
            matches!(
                Model::from_parts(model.data().clone()).check_shape_is_possible(),
                Err(Error::Invalid(
                    Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells
                ))
            ),
            "the door's own check names the cut"
        );
    }

    /// Two corners added and taken out again leave two empty slots, and a file
    /// keeps them: it lists every slot of an arena, empty ones included. The
    /// model read back fills those first, the later of the two first of all, so
    /// the copy's first two corners land in the reverse of their originals'
    /// order. The copy is still whole and shares nothing; what it gives up is
    /// sorting the way its original sorts, which is what
    /// [`Model::copy_solid`] says it does not promise.
    #[test]
    fn a_copy_written_into_empty_slots_does_not_sort_the_way_its_original_does() {
        let (mut model, _) = a_cube();
        let stray = [
            model.add_vertex(Point::new(9.0, 9.0, 9.0)),
            model.add_vertex(Point::new(9.0, 9.0, 8.0)),
        ];
        for vertex in stray {
            model.data_mut().vertices.remove(vertex);
        }
        let bytes = postcard::to_allocvec(&model).expect("a model writes out");
        let mut read_back: Model = postcard::from_bytes(&bytes).expect("and reads back");
        let original = read_back.solids().next().expect("the cube").key();

        let copy = read_back
            .copy_solid(original)
            .expect("this model made the key");

        assert_ne!(
            corners_in_slot_order(&read_back, copy),
            corners_in_slot_order(&read_back, original)
        );
        read_back
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("the copy is whole and shares nothing");
    }

    /// Every corner of the solid, as bits, in the order of the slots they sit
    /// in.
    fn corners_in_slot_order(model: &Model, solid: SolidKey) -> Vec<[u64; 3]> {
        let mut corners: Vec<_> = model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .flat_map(|face| face.wires().flat_map(|wire| wire.coedges()))
            .map(|coedge| {
                let corner = coedge.start();
                (
                    corner.key().slot_in(model.id()),
                    corner.point().coordinates().map(f64::to_bits),
                )
            })
            .collect();
        corners.sort_unstable();
        corners.dedup();
        corners.into_iter().map(|(_, at)| at).collect()
    }
}
