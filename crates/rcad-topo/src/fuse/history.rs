//! What became of the input.
//!
//! One image map per kind of entity, and nothing else (§17.4). Generated,
//! modified and deleted are **derived** from those maps rather than stored
//! beside them: three stored lists can disagree with each other and with the
//! model, and one map cannot.
//!
//! An input key stays valid after a fuse. Nothing is deleted from the arena, so
//! "deleted" means an input with no image and not a key that has stopped
//! resolving — which is what makes asking about one safe.

use std::collections::BTreeMap;

use crate::key::{EdgeSlot, FaceSlot, ModelId, VertexSlot};
use crate::{EdgeKey, FaceKey, Model, SolidKey, VertexKey};

use super::Scratch;
use super::assemble::{Assembly, Written};

/// What one fuse made, and what it made it from.
#[derive(Clone, Debug)]
pub struct Fused {
    model: ModelId,
    solids: Vec<SolidKey>,
    made: Vec<FaceKey>,
    faces: BTreeMap<FaceSlot, Vec<FaceKey>>,
    edges: BTreeMap<EdgeSlot, Vec<EdgeKey>>,
    vertices: BTreeMap<VertexSlot, Vec<VertexKey>>,
}

impl Fused {
    /// The solids the fuse made. Empty is a real answer, not a failure: two
    /// solids that do not touch have an empty intersection and the caller asked
    /// a well-formed question (§17.4).
    #[must_use]
    pub fn solids(&self) -> &[SolidKey] {
        &self.solids
    }

    /// What became of an input face: the faces it turned into. Empty means it
    /// was deleted.
    ///
    /// # Panics
    ///
    /// If the key came from another model.
    #[must_use]
    pub fn images_of_face(&self, face: FaceKey) -> &[FaceKey] {
        self.faces
            .get(&face.slot_in(self.model))
            .map_or(&[], Vec::as_slice)
    }

    /// What became of an input edge. Empty means it was deleted.
    ///
    /// # Panics
    ///
    /// If the key came from another model.
    #[must_use]
    pub fn images_of_edge(&self, edge: EdgeKey) -> &[EdgeKey] {
        self.edges
            .get(&edge.slot_in(self.model))
            .map_or(&[], Vec::as_slice)
    }

    /// What became of an input vertex. Empty means it was deleted.
    ///
    /// # Panics
    ///
    /// If the key came from another model.
    #[must_use]
    pub fn images_of_vertex(&self, vertex: VertexKey) -> &[VertexKey] {
        self.vertices
            .get(&vertex.slot_in(self.model))
            .map_or(&[], Vec::as_slice)
    }

    /// Faces in the result with no input ancestor.
    ///
    /// Always empty while the fuse is planar: every piece is cut out of exactly
    /// one input face and nothing here invents geometry. It is derived rather
    /// than asserted so that it stops being empty the day something does.
    pub fn generated_faces(&self) -> impl Iterator<Item = FaceKey> + '_ {
        self.made
            .iter()
            .copied()
            .filter(|face| !self.faces.values().any(|images| images.contains(face)))
    }

    /// Input faces that came through as more than one face, or as one face that
    /// is not themselves.
    ///
    /// Which, while a fuse never reuses an input face, is every input face that
    /// survived at all: the answer's faces are new faces, because the input
    /// solids are still in the model using the old ones and a face may only be
    /// used once.
    pub fn modified_faces(&self) -> impl Iterator<Item = FaceKey> + '_ {
        self.faces
            .iter()
            .filter(|(_, images)| !images.is_empty())
            .map(|(&slot, _)| FaceKey::new(slot, self.model))
    }

    /// Input faces with no image.
    pub fn deleted_faces(&self) -> impl Iterator<Item = FaceKey> + '_ {
        self.faces
            .iter()
            .filter(|(_, images)| images.is_empty())
            .map(|(&slot, _)| FaceKey::new(slot, self.model))
    }
}

/// Work out what became of every input entity.
///
/// Every input face, edge and vertex gets an entry, empty or not, so that
/// "deleted" is a question with an answer rather than a lookup that misses.
pub(crate) fn history_of(
    model: &Model,
    scratch: &Scratch,
    assembly: &Assembly,
    written: &Written,
) -> Fused {
    let id = model.id();
    let mut fused = Fused {
        model: id,
        solids: written
            .solids
            .iter()
            .map(|&solid| model.key_for_solid(solid))
            .collect(),
        made: written
            .faces
            .iter()
            .map(|&face| FaceKey::new(face, id))
            .collect(),
        faces: BTreeMap::new(),
        edges: BTreeMap::new(),
        vertices: BTreeMap::new(),
    };

    for solid in scratch.arguments.iter().chain(&scratch.tools) {
        for face in model.solid(solid.key).faces() {
            fused.faces.entry(face.key().slot_in(id)).or_default();
            for wire in model.face(face.key()).wires() {
                for coedge in wire.coedges() {
                    fused
                        .edges
                        .entry(coedge.edge().key().slot_in(id))
                        .or_default();
                    fused
                        .vertices
                        .entry(coedge.start().key().slot_in(id))
                        .or_default();
                }
            }
        }
    }

    for (index, ancestors) in assembly.face_ancestors().enumerate() {
        for ancestor in ancestors {
            fused
                .faces
                .entry(ancestor.slot_in(id))
                .or_default()
                .push(FaceKey::new(written.faces[index], id));
        }
    }
    for (index, ancestors) in assembly.edge_ancestors().enumerate() {
        for ancestor in ancestors {
            fused
                .edges
                .entry(ancestor.slot_in(id))
                .or_default()
                .push(EdgeKey::new(written.edges[index], id));
        }
    }
    for (index, ancestor) in assembly.corner_ancestors().enumerate() {
        if let Some(ancestor) = ancestor {
            fused
                .vertices
                .entry(ancestor.slot_in(id))
                .or_default()
                .push(VertexKey::new(written.vertices[index], id));
        }
    }
    fused
}
