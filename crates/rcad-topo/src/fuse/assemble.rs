//! Putting the kept pieces back together, and writing them into the arena once.
//!
//! Everything here runs in scratch until the very last step (§17.4). A
//! half-applied boolean is the corrupt solid §4.1 would rather crash than emit,
//! so every question that can be answered `Err` — is the result a manifold, is
//! any shell inside out, which shell is a void of which — is settled before
//! [`Assembly::commit`] touches the model, and `commit` itself only calls arena
//! primitives that cannot fail.
//!
//! # How the pieces find each other
//!
//! Two pieces meet at a corner when they name the same corner, and the name is
//! either the input vertex it is or the three planes it is the meet of, sorted.
//! No coordinate is ever compared: two faces solving the same three planes in a
//! different order would round differently, and comparing what they rounded to
//! would either miss the join or invent one.
//!
//! An edge is then a pair of corners, a shell is a lump of faces joined by
//! edges, and an edge claimed by anything other than exactly two faces is
//! [`Error::ResultIsNotManifold`]. That is the whole manifold check, and it
//! lands here because this is where an edge with three claimants is *discovered*
//! rather than searched for.
//!
//! # Two pieces on one plane are one face
//!
//! Selection can keep two pieces that lie on the same plane facing the same way
//! with an edge between them — one cut from each input's face where the two
//! inputs are flush against each other. [`Assembly::join_up_what_is_one_face`]
//! folds every such run into one face and drops the corners that stop being
//! corners when it does. §17.4's amendment of 2026-08-30 says why, and the short
//! of it is that anything else hands back a solid §17.1 cannot read: a corner in
//! the middle of a straight edge has two planes round it, not three, so the next
//! fuse cannot re-derive it.
//!
//! An *input* can arrive in that state too — `ProfileBuilder` accepts three
//! profile points in a row and the extrusion of one has two side faces on a
//! single plane — so [`joined_faces_of_an_input`] runs the same joining over
//! each input solid before its corners are solved (§17.4, amended 2026-08-31).
//! One joiner, two callers.
//!
//! # Which shell is a void of which
//!
//! Nesting asks whether a point on one shell is inside another, and the thing
//! that answers that is [`SolidView::classify_point`](crate::SolidView), which
//! wants a [`Model`]. So the shells are written into a scratch model first, one
//! solid each, and asked there. The real model sees the finished answer or
//! nothing at all.

use std::collections::{BTreeMap, BTreeSet};

use rcad_geom::{
    EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, PlaneAgreement, Point, Surface,
    Vector,
};
use rcad_predicates::{
    Approximation, Cause, Decision, Measures, Point as PredicatePoint, Policy, ProvenanceLog,
};
use smallvec::SmallVec;

use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, ShellSlot, SolidSlot, VertexSlot, WireSlot};
use crate::measure::signed_volume_over;
use crate::model::{FaceUse, ShellUse};
use crate::validate::{
    a_closed_crossing_is_walked_the_way_it_is_stored, as_coarse_as_a_mesh_gets,
    every_side_of_the_skin_is_used_both_ways, gradient_of, the_curved_one_of, winding_round,
};
use crate::{
    Containment, EdgeKey, Error, FaceKey, Invalid, Model, Result, SolidView, VertexKey, VertexView,
};

use super::arrange::{
    agreement_of_two_planes, name_of_surface_bits, name_of_the_surface_either_way_round,
};
use super::select::Chosen;
use super::{FaceMeet, MeetPoint, Sameness, Scratch, gave_up, middle_of};

/// Work out the whole result: its corners, edges, faces, shells and solids.
///
/// Nothing here writes to `model`.
///
/// # Errors
///
/// [`Error::ResultIsNotManifold`] when an edge of the result would have other
/// than two faces along it, [`Error::Invalid`] when a shell encloses no space or
/// a void has nothing round it, and [`Error::Undecidable`] when one shell could
/// not be placed inside or outside another.
pub(crate) fn assemble(
    model: &Model,
    scratch: &Scratch,
    chosen: &[Chosen],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Assembly> {
    let mut assembly = Assembly::from_the_pieces(model, scratch, chosen);
    assembly.join_up_what_is_one_face(log)?;
    assembly.runs = assembly.how_each_edge_runs()?;
    assembly.shells = assembly.group_into_shells()?;
    assembly.solids = assembly.nest_the_shells(model, policy, log)?;
    Ok(assembly)
}

/// The faces of one input solid with any that are really one flat region joined
/// into one, or `None` when it has no such pair and nothing needs doing.
///
/// §17.4's amendment of 2026-08-30 joins coplanar faces of the fuse's own
/// answers, and §17.1 is why: a corner between two faces on one plane has two
/// planes round it and not three, so nothing can re-derive it. An input the fuse
/// did not build carries no such promise — `ProfileBuilder` accepts three
/// profile points in a row and the extrusion of one has two side faces on a
/// single plane — so the same joining runs on the way in, done by the same code.
///
/// # Errors
///
/// [`Error::Undecidable`] when two of the input's planes cannot be told apart at
/// all, and [`Error::ResultIsNotManifold`] when one of its edges is claimed by
/// other than two faces, which means the input was already broken.
pub(crate) fn joined_faces_of_an_input(
    model: &Model,
    solid: SolidView<'_>,
    log: &mut ProvenanceLog,
) -> Result<Option<Vec<InputFace>>> {
    let mut assembly = Assembly::from_one_input_solid(model, solid);
    if !assembly.join_up_what_is_one_face(log)? {
        return Ok(None);
    }
    Ok(Some(assembly.as_input_faces()?))
}

/// One face of an input solid once the coplanar ones have been joined.
pub(crate) struct InputFace {
    /// The input faces this is. More than one when coplanar faces were joined.
    pub(crate) faces: Vec<FaceKey>,
    /// The plane, turned to point out of the material.
    pub(crate) plane: Plane,
    /// The outline first and then one ring per hole, the outline walked
    /// anticlockwise round [`InputFace::plane`]'s normal and the holes
    /// clockwise.
    pub(crate) wires: Vec<Vec<InputSide>>,
}

/// One side of one wire of an [`InputFace`].
pub(crate) struct InputSide {
    /// The input edges this is. More than one when joining left a corner in the
    /// middle of a straight run and the edges either side of it became one.
    pub(crate) edges: SmallVec<[EdgeKey; 1]>,
    /// The corner the wire is at when it starts along here.
    pub(crate) start: VertexKey,
    /// The plane of the face on the other side.
    pub(crate) across: Plane,
}

/// The whole result, in scratch.
pub(crate) struct Assembly {
    corners: Vec<Corner>,
    edges: Vec<Edge>,
    /// What each edge runs along, worked out once so that the scratch model the
    /// shells are nested in and the caller's model are built the same way.
    runs: Vec<EdgeRun>,
    faces: Vec<Face>,
    shells: Vec<Vec<usize>>,
    solids: Vec<Lump>,
}

impl Assembly {
    /// The input faces each face of the result was cut out of, in the order
    /// [`Assembly::write_into`] writes them.
    ///
    /// More than one when pieces of two input faces turned out to be one flat
    /// region and were joined into a single face.
    pub(crate) fn face_ancestors(&self) -> impl Iterator<Item = &[FaceKey]> + '_ {
        self.faces.iter().map(|face| face.ancestors.as_slice())
    }

    /// The input edges each edge of the result is a piece of, in the order
    /// [`Assembly::write_into`] writes them.
    ///
    /// Empty for boundary the result invents, and more than one where joining
    /// two faces left a corner with nothing to be a corner of and the edges
    /// either side of it became one.
    pub(crate) fn edge_ancestors(&self) -> impl Iterator<Item = &[EdgeKey]> + '_ {
        self.edges.iter().map(|edge| edge.from.as_slice())
    }

    /// The input vertex each corner of the result is, where it is one, in the
    /// order [`Assembly::write_into`] writes them.
    pub(crate) fn corner_ancestors(&self) -> impl Iterator<Item = Option<VertexKey>> + '_ {
        self.corners.iter().map(|corner| corner.from)
    }

    /// Write the result into the model, once.
    ///
    /// The first and only mutation of the whole operation. Everything that could
    /// have refused has already refused.
    ///
    /// New corners are tagged with how wide their box is, as
    /// [`Cause::ConstructedPoint`] — representation wobble, which is reported and
    /// never refused on (§17.2). On top of that everything the fuse builds
    /// carries at least the worst tag the inputs carried: a face cut by a face
    /// that was only known to a millimetre is only known to a millimetre itself,
    /// and §17.2 is explicit that a certified fuse carries an input's tag through
    /// rather than refusing it.
    ///
    /// # Panics
    ///
    /// If the result breaks an invariant of the model — in debug builds, any
    /// invariant [`Model::validate`] checks. A fuse invents topology that was in
    /// no input, which puts it nearer an import than a mutation (§17.4), and a
    /// corrupt solid is worse than a crash.
    ///
    /// **Only when the model was sound before the write.** [`Model::validate`]
    /// sweeps everything, including the parts of the model the fuse never
    /// looked at, and a caller can build a model that does not pass it — the
    /// sweep runs at test and import boundaries, not after every mutation
    /// (§4.3). Panicking there would report somebody else's broken solid, or
    /// somebody else's dynamic range, as this fuse's bug. What the panic
    /// promises is that a fuse handed a sound model leaves a sound one.
    pub(crate) fn commit(
        &self,
        model: &mut Model,
        inherited: Option<Approximation>,
        policy: &Policy,
    ) -> Written {
        let was_sound =
            cfg!(debug_assertions) && model.validate(policy, &mut ProvenanceLog::new()).is_ok();

        let approximation_before = model.worst_approximation();
        let written = self.write_into(model, &self.solids, &self.runs);
        let worst = self.tag_everything_it_built(model, &written, inherited);
        model.assert_approximation_did_not_grow(approximation_before.max(worst));

        if was_sound {
            model
                .validate(policy, &mut ProvenanceLog::new())
                .expect("a fuse writes a valid model or does not write at all");
        }
        written
    }

    /// One face per kept piece, and the corners and edges they share.
    fn from_the_pieces(model: &Model, scratch: &Scratch, chosen: &[Chosen]) -> Self {
        let mut corners = Corners::default();
        let mut edges = Edges::default();
        let mut faces = Vec::new();
        for kept in chosen {
            let piece = &scratch.pieces[kept.piece];
            let turned_round = piece.orientation.is_reversed() != kept.reversed;
            let wires = piece
                .wires
                .iter()
                .map(|wire| sides_of(wire, turned_round, model, &mut corners, &mut edges))
                .collect();
            let (surface, used) = match (piece.surface.as_plane(), turned_round) {
                (Some(flat), true) => (Surface::Plane(flat.reversed()), Orientation::Forward),
                (Some(_) | None, false) => (piece.surface, Orientation::Forward),
                (None, true) => (piece.surface, Orientation::Reversed),
            };
            faces.push(Face {
                ancestors: piece.ancestors.clone(),
                surface,
                used,
                wires,
            });
        }
        Self {
            corners: corners.made,
            edges: edges.made,
            runs: Vec::new(),
            faces,
            shells: Vec::new(),
            solids: Vec::new(),
        }
    }

    /// One face per face of an input solid, ready to be joined.
    ///
    /// The planes are turned to point out of the material and the wires walked
    /// the way the solid uses them, which is the convention
    /// [`Assembly::from_the_pieces`] leaves the pieces in and the one
    /// [`Assembly::join_up_what_is_one_face`] reads.
    ///
    /// [`Corner::wobble`] is zero throughout and never read on this path.
    /// Nothing here is committed: what a corner of an input is worth is settled
    /// by re-deriving it from the planes round it afterwards (§17.1).
    fn from_one_input_solid(model: &Model, solid: SolidView<'_>) -> Self {
        let mut corner_of_vertex: BTreeMap<VertexSlot, usize> = BTreeMap::new();
        let mut corners: Vec<Corner> = Vec::new();
        let mut edge_of_input_edge: BTreeMap<EdgeSlot, usize> = BTreeMap::new();
        let mut edges: Vec<Edge> = Vec::new();
        let mut faces = Vec::new();

        for face in solid.faces() {
            let mut wires = Vec::new();
            for wire in face.wires() {
                let mut sides = Vec::new();
                for coedge in wire.coedges() {
                    let used = coedge.edge();
                    let from = corner_of(model, &mut corner_of_vertex, &mut corners, used.start());
                    let to = corner_of(model, &mut corner_of_vertex, &mut corners, used.end());
                    let slot = used.key().slot_in(model.id());
                    let edge = match edge_of_input_edge.get(&slot) {
                        Some(&already) => already,
                        None => {
                            edges.push(Edge {
                                ends: [from, to],
                                // The joiner is only ever asked about a wholly
                                // flat solid, so a straight run is what every
                                // one of these is, and nothing on this path
                                // reads it back.
                                carried_by: [face.surface(); 2],
                                arc: WhichArc::TheOnlyOne,
                                from: vec![used.key()],
                            });
                            edge_of_input_edge.insert(slot, edges.len() - 1);
                            edges.len() - 1
                        }
                    };
                    sides.push(Side {
                        edge,
                        forward: edges[edge].ends[0] == from,
                    });
                }
                wires.push(sides);
            }
            let plane = face.plane();
            faces.push(Face {
                ancestors: vec![face.key()],
                surface: Surface::Plane(if face.orientation().is_reversed() {
                    plane.reversed()
                } else {
                    plane
                }),
                used: Orientation::Forward,
                wires,
            });
        }

        Self {
            corners,
            edges,
            runs: Vec::new(),
            faces,
            shells: Vec::new(),
            solids: Vec::new(),
        }
    }

    /// The joined faces read back out, for `bake` to solve the corners of.
    ///
    /// # Errors
    ///
    /// [`Error::ResultIsNotManifold`] when an edge is claimed by other than two
    /// faces.
    ///
    /// # Panics
    ///
    /// If a corner left is not one of the input's own vertices. Joining drops
    /// corners and invents none.
    fn as_input_faces(&self) -> Result<Vec<InputFace>> {
        let along = self.darts_along_each_edge()?;
        let mut faces = Vec::new();
        for (index, face) in self.faces.iter().enumerate() {
            let mut wires = Vec::new();
            for (wire, ring) in face.wires.iter().enumerate() {
                let mut sides = Vec::new();
                for (position, side) in ring.iter().enumerate() {
                    let dart = Dart {
                        face: index,
                        wire,
                        position,
                    };
                    sides.push(InputSide {
                        edges: self.edges[side.edge].from.iter().copied().collect(),
                        start: self.corners[self.starts_at(dart)]
                            .from
                            .expect("a corner of an input solid is one of its own vertices"),
                        across: self.faces[across(dart, &along, side.edge).face].outward_plane(),
                    });
                }
                wires.push(sides);
            }
            faces.push(InputFace {
                faces: face.ancestors.clone(),
                plane: face.outward_plane(),
                wires,
            });
        }
        Ok(faces)
    }

    /// Fold every run of faces that is really one flat region into one face.
    ///
    /// Two faces of the result with an edge between them, lying on the same
    /// plane and facing the same way, are two halves of one face and the edge
    /// between them is not boundary at all. Selection produces them whenever two
    /// inputs are flush against each other: the flush region is cut out of both
    /// faces and kept once, while what is left of each face round it is kept
    /// too, so the answer comes back in three pieces where it has one face.
    ///
    /// Leaving them apart costs two things. The partition then depends on which
    /// side of the fuse each input was on, because the two coincident pieces are
    /// cut by different sets of lines and "keep once" picks whichever came first
    /// — which is §14.8's commutativity oracle failing. And the corner where two
    /// such pieces meet has only two planes round it, so §17.1 cannot re-derive
    /// it and the *next* fuse over the answer refuses.
    ///
    /// So the edge goes, and with it any corner that stops being a corner: a
    /// point with two faces round it sits in the middle of a straight edge and
    /// the two edges either side of it become one.
    ///
    /// Answers `true` when it joined something, which is what lets
    /// [`joined_faces_of_an_input`] leave an input that needs nothing alone.
    ///
    /// # Errors
    ///
    /// [`Error::ResultIsNotManifold`] when an edge is claimed by other than two
    /// faces, and [`Error::Undecidable`] when two planes cannot be told apart at
    /// all.
    ///
    /// # Panics
    ///
    /// If a joined face comes out with two outlines, which is the answer
    /// pinching to a point at a corner, or with a ring of fewer than three
    /// sides. Both are broken internal invariants and a corrupt solid is worse
    /// than a crash (§4.1).
    fn join_up_what_is_one_face(&mut self, log: &mut ProvenanceLog) -> Result<bool> {
        let along = self.darts_along_each_edge()?;
        let dissolved = self.edges_with_one_face_on_both_sides(&along, log)?;

        let mut same = Sameness::of(self.faces.len());
        for (edge, [one, other]) in along.iter().enumerate() {
            if dissolved[edge] {
                same.join(one.face, other.face);
            }
        }
        if !same.joined_anything() {
            return Ok(false);
        }

        let joined = self.rings_of_each_joined_face(&same, &along, &dissolved)?;
        self.rebuild_from(&joined, &along, &dissolved);
        Ok(true)
    }

    /// The two darts along each edge, refusing a result that is not a manifold.
    ///
    /// # Errors
    ///
    /// [`Error::ResultIsNotManifold`] when an edge is claimed by other than two
    /// faces. `Model` means an edge is used by exactly two coedges everywhere,
    /// and relaxing that changes what a model *is* rather than what one boolean
    /// does (§17.4).
    fn darts_along_each_edge(&self) -> Result<Vec<[Dart; 2]>> {
        let mut claimants: Vec<Vec<Dart>> = vec![Vec::new(); self.edges.len()];
        for (face, walked) in self.faces.iter().enumerate() {
            for (wire, ring) in walked.wires.iter().enumerate() {
                for (position, side) in ring.iter().enumerate() {
                    claimants[side.edge].push(Dart {
                        face,
                        wire,
                        position,
                    });
                }
            }
        }
        claimants
            .into_iter()
            .map(|by| {
                <[Dart; 2]>::try_from(by.as_slice())
                    .map_err(|_| Error::ResultIsNotManifold { count: by.len() })
            })
            .collect()
    }

    /// Which edges have the same flat region on both sides of them.
    ///
    /// [`Plane::agreement_with`] decides it exactly, with no tolerance anywhere
    /// (§17.4). An edge whose two darts belong to one face stays: a face that
    /// runs up against itself is still bounded there.
    fn edges_with_one_face_on_both_sides(
        &self,
        along: &[[Dart; 2]],
        log: &mut ProvenanceLog,
    ) -> Result<Vec<bool>> {
        along
            .iter()
            .map(|[one, other]| {
                if one.face == other.face {
                    return Ok(false);
                }
                // **Only two flat faces are ever joined**, and what that leaves
                // undone is worth naming. Two pieces of one *cylinder* facing
                // the same way with an edge between them really are one face,
                // and the answer keeps them as two — a sound solid, cut into
                // more faces than it has. Joining them would need the rings walk
                // below, which winds corners about a plane's normal and has no
                // curved twin. What used to make this urgent no longer does:
                // §17.1 could not re-derive a corner with two surfaces round it,
                // and §31.3's second route reads one out of the model.
                let (Some(mine), Some(theirs)) = (
                    self.faces[one.face].surface.as_plane(),
                    self.faces[other.face].surface.as_plane(),
                ) else {
                    return Ok(false);
                };
                let agreement = agreement_of_two_planes(mine, theirs, log)?;
                Ok(agreement == PlaneAgreement::SameWay)
            })
            .collect()
    }

    /// The rings bounding each face once the joined ones are one face.
    ///
    /// A face nothing was joined to keeps the rings it already had. For the
    /// rest, every dart on an edge that is staying is walked, and where the walk
    /// runs into a dissolved edge it steps across into the neighbouring face and
    /// carries on — so the rings that come out are the boundary of the joined
    /// region and nothing inside it. Which ring is the outline and which are
    /// holes is then the way each one winds, decided exactly by the same sweep
    /// [`Model::validate`] judges a face by.
    fn rings_of_each_joined_face(
        &self,
        same: &Sameness,
        along: &[[Dart; 2]],
        dissolved: &[bool],
    ) -> Result<Vec<JoinedFace>> {
        let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for face in 0..self.faces.len() {
            groups.entry(same.leader(face)).or_default().push(face);
        }

        let mut joined = Vec::new();
        for members in groups.into_values() {
            if let [only] = members[..] {
                joined.push(JoinedFace {
                    surface: self.faces[only].surface,
                    used: self.faces[only].used,
                    rings: (0..self.faces[only].wires.len())
                        .map(|wire| self.whole_ring(only, wire))
                        .collect(),
                    members,
                });
                continue;
            }
            // Only flat faces are ever joined — the walk below winds corners
            // about a plane's normal and has no curved twin — so a group with
            // more than one member is flat throughout.
            let plane = plane_of_the_joined_face(&members, &self.faces);
            let rings = self.rings_round(&members, along, dissolved, plane)?;
            joined.push(JoinedFace {
                members,
                surface: Surface::Plane(plane),
                used: Orientation::Forward,
                rings,
            });
        }
        Ok(joined)
    }

    fn whole_ring(&self, face: usize, wire: usize) -> Vec<Dart> {
        (0..self.faces[face].wires[wire].len())
            .map(|position| Dart {
                face,
                wire,
                position,
            })
            .collect()
    }

    /// # Panics
    ///
    /// If more than one ring winds with the plane's normal. Two outlines means
    /// the joined region touches itself at a corner, which is a solid this
    /// kernel cannot hold.
    fn rings_round(
        &self,
        members: &[usize],
        along: &[[Dart; 2]],
        dissolved: &[bool],
        plane: Plane,
    ) -> Result<Vec<Vec<Dart>>> {
        let mut walked: BTreeSet<Dart> = BTreeSet::new();
        let mut outline: Option<Vec<Dart>> = None;
        let mut holes = Vec::new();
        for &face in members {
            for wire in 0..self.faces[face].wires.len() {
                for start in self.whole_ring(face, wire) {
                    if dissolved[self.edge_of(start)] || !walked.insert(start) {
                        continue;
                    }
                    let ring = self.ring_from(start, along, dissolved, &mut walked);
                    let corners: Vec<[f64; 3]> = ring
                        .iter()
                        .map(|&dart| self.corners[self.starts_at(dart)].point.coordinates())
                        .collect();
                    match winding_round(&corners, plane.normal().components())? {
                        Decision::Pos => {
                            assert!(
                                outline.is_none(),
                                "a face joined out of coplanar pieces has one outline, and this \
                                 one has two, so the answer touches itself at a corner"
                            );
                            outline = Some(ring);
                        }
                        Decision::Neg => holes.push(ring),
                        other => panic!(
                            "a ring of a joined face came back winding neither way: {other:?}"
                        ),
                    }
                }
            }
        }
        let outline = outline.expect("a joined face has an outline");
        Ok(core::iter::once(outline).chain(holes).collect())
    }

    /// # Panics
    ///
    /// If a walk round a corner of the joined region never reaches an edge that
    /// is staying. Every corner a walk arrives at is on an edge that is staying,
    /// so it has one to leave by.
    fn ring_from(
        &self,
        start: Dart,
        along: &[[Dart; 2]],
        dissolved: &[bool],
        walked: &mut BTreeSet<Dart>,
    ) -> Vec<Dart> {
        let mut ring = vec![start];
        let mut at = start;
        loop {
            at = self.onwards(at, along, dissolved);
            if at == start {
                return ring;
            }
            assert!(
                walked.insert(at),
                "a ring of a joined face runs into one that is already walked"
            );
            ring.push(at);
        }
    }

    fn onwards(&self, dart: Dart, along: &[[Dart; 2]], dissolved: &[bool]) -> Dart {
        let mut at = self.next_round_the_ring(dart);
        for _ in 0..=self.edges.len() {
            let edge = self.edge_of(at);
            if !dissolved[edge] {
                return at;
            }
            at = self.next_round_the_ring(across(at, along, edge));
        }
        panic!("a corner of a joined face has no edge leaving it that is staying");
    }

    fn next_round_the_ring(&self, dart: Dart) -> Dart {
        Dart {
            position: (dart.position + 1) % self.faces[dart.face].wires[dart.wire].len(),
            ..dart
        }
    }

    fn edge_of(&self, dart: Dart) -> usize {
        self.side_of(dart).edge
    }

    /// The corner a dart leaves.
    fn starts_at(&self, dart: Dart) -> usize {
        let side = self.side_of(dart);
        self.edges[side.edge].ends[usize::from(!side.forward)]
    }

    fn side_of(&self, dart: Dart) -> &Side {
        &self.faces[dart.face].wires[dart.wire][dart.position]
    }

    /// Write the corners, edges and faces out again with the joined faces in
    /// place of their pieces and the corners that stopped being corners gone.
    ///
    /// Everything keeps the order it was in, so an answer that joined nothing
    /// comes out of here byte for byte the answer that went in.
    ///
    /// # Panics
    ///
    /// If a ring comes out with fewer than three sides, or if a run of edges
    /// that became one edge does not have exactly two ends left. Both mean the
    /// joining above is wrong.
    fn rebuild_from(&mut self, joined: &[JoinedFace], along: &[[Dart; 2]], dissolved: &[bool]) {
        let renumbered = self.what_is_left(joined, along, dissolved);
        let edges = self.edges_after_joining(&renumbered);

        let faces = joined
            .iter()
            .map(|face| Face {
                ancestors: each_one_once(
                    face.members
                        .iter()
                        .flat_map(|&member| self.faces[member].ancestors.iter().copied()),
                ),
                surface: face.surface,
                used: face.used,
                wires: face
                    .rings
                    .iter()
                    .map(|ring| self.sides_of(ring, &renumbered, &edges))
                    .collect(),
            })
            .collect();

        self.corners = core::mem::take(&mut self.corners)
            .into_iter()
            .zip(&renumbered.corner_now)
            .filter_map(|(corner, now)| now.map(|_| corner))
            .collect();
        self.edges = edges;
        self.faces = faces;
    }

    /// Which corners and edges the joining leaves, and what they are called now.
    ///
    /// Everything is asked one lump of joined faces at a time, and that is not
    /// pedantry: two solids that touch along a line have an edge each between the
    /// same two corners, and a corner sitting in the middle of one of them can be
    /// a real corner of the other. A corner stops being a corner for the lump it
    /// runs straight through and stays one for the lump it does not.
    fn what_is_left(
        &self,
        joined: &[JoinedFace],
        along: &[[Dart; 2]],
        dissolved: &[bool],
    ) -> Renumbered {
        let mut lump_of_face = vec![0usize; self.faces.len()];
        for (index, face) in joined.iter().enumerate() {
            for &member in &face.members {
                lump_of_face[member] = index;
            }
        }
        let mut lumps = Sameness::of(joined.len());
        for (edge, [one, other]) in along.iter().enumerate() {
            if !dissolved[edge] {
                lumps.join(lump_of_face[one.face], lump_of_face[other.face]);
            }
        }
        let lump_of_edge: Vec<Option<usize>> = (0..self.edges.len())
            .map(|edge| (!dissolved[edge]).then(|| lumps.leader(lump_of_face[along[edge][0].face])))
            .collect();

        let mut leaving: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
        for (edge, lump) in lump_of_edge.iter().enumerate() {
            let Some(lump) = *lump else { continue };
            for end in self.edges[edge].ends {
                leaving.entry((lump, end)).or_default().push(edge);
            }
        }

        let mut runs = Sameness::of(self.edges.len());
        let mut straight_through: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut still_a_corner = vec![false; self.corners.len()];
        for (&(lump, corner), by) in &leaving {
            if let [one, other] = by[..] {
                runs.join(one, other);
                straight_through.insert((lump, corner));
            } else {
                still_a_corner[corner] = true;
            }
        }

        let corner_now = numbered(still_a_corner.into_iter());
        let edge_now = numbered(
            (0..self.edges.len())
                .map(|edge| lump_of_edge[edge].is_some() && runs.leader(edge) == edge),
        );
        Renumbered {
            lump_of_edge,
            straight_through,
            runs,
            corner_now,
            edge_now,
        }
    }

    fn edges_after_joining(&self, renumbered: &Renumbered) -> Vec<Edge> {
        let mut made: Vec<Edge> = Vec::new();
        for (leader, now) in renumbered.edge_now.iter().enumerate() {
            if now.is_none() {
                continue;
            }
            let lump = renumbered.lump_of_edge[leader].expect("an edge that stays is in a lump");
            let mut kept = Vec::new();
            let mut dropped = Vec::new();
            let mut arcs = Vec::new();
            let mut from = Vec::new();
            for (edge, along) in self.edges.iter().enumerate() {
                if renumbered.lump_of_edge[edge].is_none() || renumbered.runs.leader(edge) != leader
                {
                    continue;
                }
                for end in along.ends {
                    if renumbered.straight_through.contains(&(lump, end)) {
                        dropped.push(end);
                    } else {
                        kept.push(end);
                    }
                }
                arcs.push(along.arc);
                from.extend(along.from.iter().copied());
            }
            let ends = <[usize; 2]>::try_from(kept.as_slice())
                .expect("a run of edges that became one edge has two ends left");
            assert!(
                ends[0] != ends[1],
                "a piece of a face has a side with no length"
            );
            made.push(Edge {
                ends: ends.map(|end| renumbered.corner_now[end].expect("an end is still a corner")),
                carried_by: self.edges[leader].carried_by,
                arc: self.arc_of_a_joined_run(&arcs, ends, &dropped, renumbered),
                from: each_one_once(from.into_iter()),
            });
        }
        made
    }

    /// Which arc a run of edges joined into one is.
    ///
    /// The pieces of a run are consecutive stretches of one arc, so the joined
    /// arc is named the same way any one of them is: a corner of it, and a
    /// direction from there into it. **A corner the joining dissolved is a point
    /// on the joined arc**, so the direction from a surviving end towards one of
    /// them points into it — and a corner the arrangement placed is not a point
    /// this had to build.
    ///
    /// # Panics
    ///
    /// If a run of one edge dissolved a corner, or a run of several dissolved
    /// none. Joining is what drops a corner and it drops exactly the ones
    /// between the pieces, so either would mean this and
    /// [`Assembly::what_is_left`] have stopped agreeing.
    fn arc_of_a_joined_run(
        &self,
        arcs: &[WhichArc],
        ends: [usize; 2],
        dropped: &[usize],
        renumbered: &Renumbered,
    ) -> WhichArc {
        let renumber = |corner: usize| {
            renumbered.corner_now[corner].expect("an end of a joined run is still a corner")
        };
        let Some(&inside) = dropped.first() else {
            assert!(
                arcs.len() == 1,
                "{} edges joined into one with no corner dissolved between them",
                arcs.len()
            );
            return match arcs[0] {
                WhichArc::TheOnlyOne => WhichArc::TheOnlyOne,
                WhichArc::LeavingACorner { corner, towards } => WhichArc::LeavingACorner {
                    corner: renumber(corner),
                    towards,
                },
            };
        };
        assert!(
            arcs.len() > 1,
            "one edge on its own dissolved a corner of itself"
        );
        if arcs.iter().all(|arc| matches!(arc, WhichArc::TheOnlyOne)) {
            return WhichArc::TheOnlyOne;
        }
        WhichArc::LeavingACorner {
            corner: renumber(ends[0]),
            towards: self.corners[inside].point - self.corners[ends[0]].point,
        }
    }

    fn sides_of(&self, ring: &[Dart], renumbered: &Renumbered, edges: &[Edge]) -> Vec<Side> {
        let lump = renumbered.lump_of_edge[self.edge_of(ring[0])]
            .expect("a ring is walked along edges that stay");
        let opens = ring
            .iter()
            .position(|&dart| {
                !renumbered
                    .straight_through
                    .contains(&(lump, self.starts_at(dart)))
            })
            .expect("a ring of a face has a corner on it");

        let mut sides: Vec<Side> = Vec::new();
        let mut carrying: Option<usize> = None;
        for step in 0..ring.len() {
            let dart = ring[(opens + step) % ring.len()];
            let edge = renumbered.runs.leader(self.edge_of(dart));
            if carrying == Some(edge) {
                continue;
            }
            carrying = Some(edge);
            let edge = renumbered.edge_now[edge].expect("a run of edges that stays becomes one");
            let from = renumbered.corner_now[self.starts_at(dart)]
                .expect("a run of edges starts at a corner");
            sides.push(Side {
                edge,
                forward: edges[edge].ends[0] == from,
            });
        }
        assert!(
            sides.len() >= 3,
            "a face has at least three sides, this one has {}",
            sides.len()
        );
        sides
    }

    /// Gather the faces into lumps joined by edges, refusing a result that is
    /// not a manifold.
    ///
    /// # Errors
    ///
    /// [`Error::ResultIsNotManifold`] when an edge is claimed by other than two
    /// faces.
    fn group_into_shells(&self) -> Result<Vec<Vec<usize>>> {
        let along = self.darts_along_each_edge()?;
        let mut same = Sameness::of(self.faces.len());
        for [one, other] in &along {
            same.join(one.face, other.face);
        }
        let mut shells: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for face in 0..self.faces.len() {
            shells.entry(same.leader(face)).or_default().push(face);
        }
        Ok(shells.into_values().collect())
    }

    /// Sort the shells into solids: each one that encloses space, with the ones
    /// that hollow it out.
    ///
    /// # Errors
    ///
    /// [`Error::Invalid`] when a shell encloses nothing at all, or when a void
    /// has no shell around it. [`Error::Undecidable`] when no point on a shell
    /// could be placed inside or outside another.
    fn nest_the_shells(
        &self,
        model: &Model,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Vec<Lump>> {
        if self.shells.is_empty() {
            return Ok(Vec::new());
        }
        let one_each: Vec<Lump> = (0..self.shells.len())
            .map(|shell| Lump {
                outer: shell,
                voids: Vec::new(),
            })
            .collect();
        let mut apart = Model::new(model.scale());
        let written = self.write_into(&mut apart, &one_each, &self.runs);

        let mut volumes = Vec::new();
        for &solid in &written.solids {
            let volume =
                how_much_space_it_encloses(apart.solid(apart.key_for_solid(solid)), policy, log)?;
            if !volume.is_finite() {
                return Err(Error::VolumeTooLargeToMeasure);
            }
            if volume == 0.0 {
                return Err(Invalid::VolumeIsNotPositive { volume }.into());
            }
            volumes.push(volume);
        }

        let mut lumps: Vec<Lump> = (0..self.shells.len())
            .filter(|&shell| volumes[shell] > 0.0)
            .map(|shell| Lump {
                outer: shell,
                voids: Vec::new(),
            })
            .collect();
        for hollow in (0..self.shells.len()).filter(|&shell| volumes[shell] < 0.0) {
            let mut holder: Option<usize> = None;
            for (index, lump) in lumps.iter().enumerate() {
                if !self.one_shell_is_inside_another(
                    &apart, &written, hollow, lump.outer, policy, log,
                )? {
                    continue;
                }
                if holder.is_none_or(|best| volumes[lumps[best].outer] > volumes[lump.outer]) {
                    holder = Some(index);
                }
            }
            let Some(holder) = holder else {
                return Err(Invalid::VolumeIsNotPositive {
                    volume: volumes[hollow],
                }
                .into());
            };
            lumps[holder].voids.push(hollow);
        }
        Ok(lumps)
    }

    /// Whether a point on `hollow` lies inside `outer`.
    ///
    /// A corner of a shell is on that shell and nowhere near the middle of it,
    /// which is exactly what makes it a fair question to ask about a *different*
    /// shell: it is inside that one or outside it, and only a shell touching this
    /// one leaves it on the skin. A corner that does land on the skin, or that
    /// nothing can place, is abandoned for the next one — choosing which corner
    /// to look at decides nothing (§5).
    ///
    /// # Errors
    ///
    /// [`Error::Undecidable`] when no corner of `hollow` could be placed.
    fn one_shell_is_inside_another(
        &self,
        apart: &Model,
        written: &Written,
        hollow: usize,
        outer: usize,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<bool> {
        let solid = apart.solid(apart.key_for_solid(written.solids[outer]));
        for &face in &self.shells[hollow] {
            for side in self.faces[face].wires.iter().flatten() {
                let at = self.corners[self.edges[side.edge].ends[0]].point;
                let mut trial = ProvenanceLog::new();
                let placed = solid.classify_point(at, policy, &mut trial);
                match placed {
                    Containment::Inside => {
                        trial.drain_into(log);
                        return Ok(true);
                    }
                    Containment::Outside => {
                        trial.drain_into(log);
                        return Ok(false);
                    }
                    // A corner nothing could place is abandoned for the next one,
                    // and a face nobody can follow is the same kind of silence:
                    // running out of corners is already a refusal, so nothing
                    // here has to be a panic. This was `unreachable!` on the
                    // grounds that the fuse refuses curved solids at entry —
                    // `docs/decisions.md` §35 records why that stops holding.
                    Containment::OnBoundary
                    | Containment::Undecidable { .. }
                    | Containment::CannotFollowAFacesTrimming { .. } => (),
                }
            }
        }
        Err(gave_up(
            ONE_SHELL_INSIDE_ANOTHER,
            &[PredicatePoint::from(self.corners[0].point.coordinates())],
            Measures::Length,
            log,
        ))
    }

    /// What each edge of the result runs along, by §31.3's table.
    ///
    /// Worked out once and handed to both writes, so the scratch model the
    /// shells are nested in and the caller's model are built the same way.
    ///
    /// # Errors
    ///
    /// [`Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed`] for an arc of a
    /// curved crossing.
    fn how_each_edge_runs(&self) -> Result<Vec<EdgeRun>> {
        self.edges
            .iter()
            .map(|edge| edge.how_it_runs(&self.corners))
            .collect()
    }

    /// Build every entity of the result in `model`, in a fixed order.
    ///
    /// Used twice: once on a scratch model, to ask which shell is inside which,
    /// and once on the caller's model, which is the commit.
    ///
    /// # Panics
    ///
    /// If what it just built breaks one of the invariants checked on touched
    /// entities. That is a bug here, not bad input.
    fn write_into(&self, model: &mut Model, lumps: &[Lump], runs: &[EdgeRun]) -> Written {
        let vertices: Vec<VertexSlot> = self
            .corners
            .iter()
            .map(|corner| model.add_vertex(corner.point))
            .collect();
        let edges: Vec<EdgeSlot> = self
            .edges
            .iter()
            .zip(runs)
            .map(|(edge, run)| {
                model.add_edge_running(vertices[edge.ends[0]], vertices[edge.ends[1]], run.clone())
            })
            .collect();

        let mut faces: Vec<FaceSlot> = Vec::new();
        for face in &self.faces {
            let mut wires: SmallVec<[WireSlot; 1]> = SmallVec::new();
            for wire in &face.wires {
                let mut coedges: SmallVec<[CoedgeSlot; 4]> = SmallVec::new();
                for side in wire {
                    let turn = if side.forward {
                        Orientation::Forward
                    } else {
                        Orientation::Reversed
                    };
                    coedges.push(model.add_coedge(edges[side.edge], turn));
                }
                wires.push(model.add_wire(coedges));
            }
            faces.push(model.add_face(face.surface, wires));
        }

        let shells: Vec<ShellSlot> = self
            .shells
            .iter()
            .map(|shell| {
                let used = shell
                    .iter()
                    .map(|&face| FaceUse {
                        face: faces[face],
                        loc: Isometry::identity(),
                        orient: self.faces[face].used,
                    })
                    .collect();
                model.add_shell(used)
            })
            .collect();

        let mut solids: Vec<SolidSlot> = Vec::new();
        for lump in lumps {
            let used = core::iter::once(lump.outer)
                .chain(lump.voids.iter().copied())
                .map(|shell| ShellUse {
                    shell: shells[shell],
                    loc: Isometry::identity(),
                    orient: Orientation::Forward,
                })
                .collect();
            let solid = model.add_solid(used);
            model.check_touched_solid(solid);
            solids.push(solid);
        }

        Written {
            vertices,
            edges,
            faces,
            solids,
        }
    }

    /// Hang the two kinds of slop on everything the fuse just built, and say how
    /// much the worst of it is.
    ///
    /// Vertices, then edges, then faces, so `tol(vertex) >= tol(edge) >=
    /// tol(face)` holds after every single step and not only at the end
    /// (roadmap §5.4) — the same order and for the same reason as
    /// `build.rs`'s `tag_everything_the_profile_made`.
    ///
    /// # Panics
    ///
    /// If a tag is refused. Everything here was built moments ago and carries no
    /// slop yet, so a refusal is a bug in this function.
    fn tag_everything_it_built(
        &self,
        model: &mut Model,
        written: &Written,
        inherited: Option<Approximation>,
    ) -> f64 {
        let from_the_inputs = inherited.map_or(0.0, Approximation::magnitude);
        let mut worst = 0.0f64;
        for (index, &vertex) in written.vertices.iter().enumerate() {
            let wobble = self.corners[index].wobble;
            let tag = if wobble > from_the_inputs {
                Approximation::new(wobble, Cause::ConstructedPoint)
            } else {
                match inherited {
                    Some(tag) => tag,
                    None => continue,
                }
            };
            worst = worst.max(tag.magnitude());
            model
                .set_vertex_approximation(VertexKey::new(vertex, model.id()), tag)
                .expect("a fresh corner carries no slop for this tag to undercut");
        }
        let Some(inherited) = inherited else {
            return worst;
        };
        for &edge in &written.edges {
            model
                .set_edge_approximation(EdgeKey::new(edge, model.id()), inherited)
                .expect("the corners were tagged with at least this much just now");
        }
        for &face in &written.faces {
            model
                .set_face_approximation(FaceKey::new(face, model.id()), inherited)
                .expect("the edges were tagged with exactly this much just now");
        }
        worst.max(inherited.magnitude())
    }
}

/// The name the search for a point that places one shell against another goes by
/// in provenance records.
const ONE_SHELL_INSIDE_ANOTHER: &str = "one shell inside another";

/// What one call to [`Assembly::write_into`] made, in the order it made it.
pub(crate) struct Written {
    pub(crate) vertices: Vec<VertexSlot>,
    pub(crate) edges: Vec<EdgeSlot>,
    pub(crate) faces: Vec<FaceSlot>,
    pub(crate) solids: Vec<SolidSlot>,
}

/// One solid of the result: the shell round the outside of it and the shells
/// hollowing it out.
struct Lump {
    outer: usize,
    voids: Vec<usize>,
}

/// One corner of the result.
struct Corner {
    point: Point,
    /// How wide the box this corner was solved as is. Representation wobble
    /// (§17.2): reported on the vertex, never a reason to refuse.
    wobble: f64,
    /// The input vertex this is, when it is one.
    from: Option<VertexKey>,
}

/// One edge of the result.
struct Edge {
    /// The two corners it runs between. **The same corner twice for an edge
    /// that closes**, which is what a cap circle and a round hole are.
    ends: [usize; 2],
    /// The two surfaces either side of it. Together they *are* where it runs
    /// (§17.4, §31.3), and they are what says which [`EdgeRun`] it is.
    carried_by: [Surface; 2],
    /// Which of the two arcs between [`Edge::ends`] this is. See
    /// [`which_arc_this_is`].
    arc: WhichArc,
    /// The input edges this is a piece of. Empty for boundary the result
    /// invents.
    from: Vec<EdgeKey>,
}

/// Which of the two arcs between one pair of corners a piece of a curved
/// crossing is.
#[derive(Clone, Copy)]
enum WhichArc {
    /// There is only one to be: the crossing runs straight between the two
    /// corners, or the edge is the whole of a closed curve.
    TheOnlyOne,
    /// The one that leaves `corner` towards `towards`. See
    /// [`which_arc_this_is`].
    LeavingACorner {
        /// Which of the edge's two corners the direction is measured from.
        corner: usize,
        /// A direction from that corner into this arc, along the crossing and
        /// not along the chord.
        towards: Vector,
    },
}

impl Edge {
    /// What this edge runs along, by §31.3's table and not by a judgement.
    ///
    /// Two planes with two different ends run straight. A pair naming one closed
    /// curve, with one corner twice, runs right round it. Everything else is a
    /// *piece* of a crossing, and which piece is
    /// [`Edge::arc`](which_arc_this_is).
    ///
    /// # Errors
    ///
    /// [`Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed`] for an arc of a
    /// curved crossing that nothing told from its sibling.
    fn how_it_runs(&self, corners: &[Corner]) -> Result<EdgeRun> {
        let flat = self
            .carried_by
            .iter()
            .all(|surface| surface.as_plane().is_some());
        match (flat, self.ends[0] == self.ends[1]) {
            (true, false) => Ok(EdgeRun::Straight),
            (_, true) => Ok(EdgeRun::RightRoundWhereItsFacesCross),
            (false, false) => match self.arc {
                WhichArc::TheOnlyOne => Ok(EdgeRun::Straight),
                WhichArc::LeavingACorner { corner, towards } => {
                    let ends = self.ends.map(|end| corners[end].point);
                    let chord = corners[self.other_end_than(corner)].point - corners[corner].point;
                    let facing = towards * chord.dot(chord) - chord * chord.dot(towards);
                    let picker = the_half_space_both_ends_are_on(facing, ends)
                        .ok_or(Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed)?;
                    Ok(EdgeRun::PartOfWhereItCrosses {
                        and: self.the_surface_that_names_it_twice(),
                        picked_by: SmallVec::from_buf([picker]),
                    })
                }
            },
        }
    }

    /// The end of this edge that is not `corner`.
    ///
    /// # Panics
    ///
    /// If `corner` is not an end of it, which would mean the arc was named
    /// against an edge it is not on.
    fn other_end_than(&self, corner: usize) -> usize {
        match self.ends {
            [one, other] if one == corner => other,
            [one, other] if other == corner => one,
            ends => panic!("an arc named from corner {corner}, which is not an end of {ends:?}"),
        }
    }

    /// Which of the two surfaces either side goes in
    /// [`EdgeRun::PartOfWhereItCrosses`]'s `and`.
    ///
    /// That field is there for a run whose two faces are the *same* surface — a
    /// ball's seam — and names the second one. An arc's two faces already name
    /// both, so either is a true answer, and what picks between them is a check:
    /// `Model::validate` asks that both ends read **exactly** `Zero` against
    /// `and`, and a corner this fuse solved for is exactly on a plane it was
    /// solved from and only within its own box on a quadric. So the flat one goes
    /// in wherever there is one, and `name_of_surface_bits` gives that for free
    /// by sorting a plane ahead of a ball or a pipe.
    ///
    /// A crossing with **two** curved surfaces and two ends is a pair of balls
    /// cut twice, and nothing builds one: `ACurvedFaceGoesRightRoundWithNoSeam`
    /// refuses two balls united before an arc of them is ever named
    /// (§36.24). When that stops being true, what has to give is the check — the
    /// same slop `check_curved_face` already reads off a solved corner (§36.13),
    /// read here too — and not this choice.
    fn the_surface_that_names_it_twice(&self) -> Surface {
        let mut either = self.carried_by;
        either.sort_by_key(name_of_surface_bits);
        either[0]
    }
}

/// One face of the result.
struct Face {
    /// The input faces it was cut out of. Every face of the result has at least
    /// one, because nothing in the fuse invents geometry with no ancestor, and
    /// more than one where coplanar pieces were joined.
    ancestors: Vec<FaceKey>,
    /// The surface it sits on, inherited from the ancestor bit for bit (§17.1).
    ///
    /// **A plane is stored already turned round** where the rule turned the face
    /// round, and a curved surface is not — because a sphere has no reversed
    /// form. That is §31.3's ON split arriving in the commit: which surface a
    /// face is on is geometry, and which way round it is used is topology, and
    /// only a plane welds the two together.
    surface: Surface,
    /// Which way round the answer's shell uses it. Always
    /// [`Orientation::Forward`] for a plane, because the plane above carries it.
    used: Orientation,
    wires: Vec<Vec<Side>>,
}

impl Face {
    /// The direction pointing out of the material, for a flat face.
    ///
    /// # Panics
    ///
    /// If the face is curved. Only the joiner asks, and it is only asked about
    /// flat faces — see [`Assembly::edges_with_one_face_on_both_sides`].
    fn outward_plane(&self) -> Plane {
        *self
            .surface
            .as_plane()
            .expect("only a flat face is asked which plane it is on")
    }
}

/// One side of one wire of one face: an edge, walked one way or the other.
struct Side {
    edge: usize,
    forward: bool,
}

/// Where one face walks along one edge: the face, the ring, and how far round
/// it.
///
/// Ordered so that a set of them can be kept in a [`BTreeSet`] — §17.4 keeps
/// hashed containers out of the fuse's working state, because their iteration
/// order would decide arena order and arena order is visible in every
/// serialized byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Dart {
    face: usize,
    wire: usize,
    position: usize,
}

/// What the joining left of every corner and every edge, and what each of them
/// is called now.
struct Renumbered {
    /// Which lump of joined faces each edge belongs to. `None` for an edge that
    /// had the same flat region on both sides of it and is gone.
    lump_of_edge: Vec<Option<usize>>,
    /// The corners a lump walks straight through, as lump and corner. Straight
    /// through means two edges of that lump meet there and nothing else does.
    straight_through: BTreeSet<(usize, usize)>,
    /// Which run of edges each edge is part of. A run becomes one edge.
    runs: Sameness,
    /// What each corner is called now, or `None` if it is gone.
    corner_now: Vec<Option<usize>>,
    /// What the first edge of each run is called now, or `None` if this edge is
    /// not the first of its run or is gone.
    edge_now: Vec<Option<usize>>,
}

/// One face of the result while it is being put back together out of the pieces
/// that turned out to be one flat region.
struct JoinedFace {
    /// The faces it was made of, in the order they were built.
    members: Vec<usize>,
    surface: Surface,
    used: Orientation,
    /// The outline first, then one ring per hole.
    rings: Vec<Vec<Dart>>,
}

/// How much space one shell of the answer encloses, signed by which way its
/// faces point.
///
/// A polyhedron's own fan *is* its volume and is summed exactly. A **curved**
/// shell has no such fan — a bar's wall fans into two triangles on its seam and
/// a round cap into none at all (§35.4) — so it is summed over a mesh instead,
/// which is what `Model::validate` already does about the same question one
/// layer up. Only the sign is read here, and the coarsest mesh there is says
/// which way round a shell is.
///
/// # Errors
///
/// Whatever meshing the shell makes of it.
fn how_much_space_it_encloses(
    solid: SolidView<'_>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<f64> {
    if solid.is_wholly_flat() {
        return Ok(solid.signed_volume());
    }
    let mesh = solid.mesh(as_coarse_as_a_mesh_gets(), policy, log)?;
    // **Asked here rather than left to the sweep, and that is the difference
    // between a refusal and a corrupt solid.** `Assembly::commit` re-validates
    // what it wrote, but only in a debug build and only as a *panic* — it is
    // there to catch a bug in this crate, and a release build would write the
    // answer out regardless. A curved answer whose two faces along one side
    // disagree which way they face is exactly that, so it is asked before
    // anything is written and comes back as a refusal (§4.1: refusing beats
    // corrupting, and both beat crashing on a caller's solid).
    every_side_of_the_skin_is_used_both_ways(&mesh)?;
    Ok(signed_volume_over(&mesh))
}

/// The other dart along the same edge.
fn across(dart: Dart, along: &[[Dart; 2]], edge: usize) -> Dart {
    let [one, other] = along[edge];
    if one == dart { other } else { one }
}

/// The plane a joined face sits on: the smallest of its pieces' planes, read as
/// four numbers.
///
/// The pieces all sit on the same plane and face the same way, but two solids
/// that are flush against each other worked their planes out separately and the
/// last bits may differ. Picking the smallest depends on the set of planes and
/// not on the order the pieces arrived in, which is what keeps the answer the
/// same when the two sides of the fuse are swapped.
fn plane_of_the_joined_face(members: &[usize], faces: &[Face]) -> Plane {
    members
        .iter()
        .map(|&member| faces[member].outward_plane())
        .min_by(|one, other| {
            let (one, other) = (one.coefficients(), other.coefficients());
            (0..4)
                .map(|index| one[index].total_cmp(&other[index]))
                .find(|order| order.is_ne())
                .unwrap_or(core::cmp::Ordering::Equal)
        })
        .expect("a joined face is made of at least one piece")
}

/// The corner an input vertex is, made the first time it is asked for.
fn corner_of(
    model: &Model,
    named: &mut BTreeMap<VertexSlot, usize>,
    made: &mut Vec<Corner>,
    vertex: VertexView<'_>,
) -> usize {
    *named
        .entry(vertex.key().slot_in(model.id()))
        .or_insert_with(|| {
            made.push(Corner {
                point: vertex.point(),
                wobble: 0.0,
                from: Some(vertex.key()),
            });
            made.len() - 1
        })
}

/// The keys in the order they arrive, with the repeats dropped.
///
/// The nine pieces one face was cut into all name that one face, and the answer
/// they were joined back into is one image of it, not nine.
fn each_one_once<K: Copy + PartialEq>(keys: impl Iterator<Item = K>) -> Vec<K> {
    let mut once: Vec<K> = Vec::new();
    for key in keys {
        if !once.contains(&key) {
            once.push(key);
        }
    }
    once
}

/// Number the things that are staying, in the order they are in.
fn numbered(staying: impl Iterator<Item = bool>) -> Vec<Option<usize>> {
    let mut next = 0;
    staying
        .map(|stays| {
            stays.then(|| {
                next += 1;
                next - 1
            })
        })
        .collect()
}

/// The corners built so far, and the two ways to find one again.
#[derive(Default)]
struct Corners {
    made: Vec<Corner>,
    named: BTreeMap<CornerName, usize>,
    at: BTreeMap<[u64; 3], usize>,
}

/// What two faces agree to call the same corner.
///
/// Never a coordinate. Two faces solving the same three surfaces in a different
/// order round differently, so comparing what they rounded to would either miss
/// a join or invent one; comparing the *names* of the surfaces cannot.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CornerName {
    /// A corner the input already had.
    Vertex(VertexSlot),
    /// One of the places three surfaces meet, each surface written out as the
    /// bits it stores and sorted so the three arrive the same way round
    /// whichever face asked, and **which** of their meeting points this is.
    ///
    /// Three surfaces can meet at more than one point, so the three names do not
    /// pick a corner on their own; §31.3 is where that field comes from and it
    /// is what stops two different corners being joined into one.
    Meet([[u64; 8]; 3], u8),
    /// The one point on a closed crossing that nothing else cuts, named by the
    /// two surfaces it runs between.
    RoundACurve([[u64; 8]; 2]),
}

impl Corners {
    fn of(&mut self, model: &Model, end: &MeetPoint) -> usize {
        let name = match end {
            MeetPoint::Existing { vertex, .. } => CornerName::Vertex(vertex.slot_in(model.id())),
            MeetPoint::OfThreeSurfaces {
                surfaces, which, ..
            } => {
                let mut named = surfaces.map(|surface| name_of_surface_bits(&surface));
                named.sort_unstable();
                CornerName::Meet(named, *which)
            }
            MeetPoint::WhereAClosedCurveStarts { carried_by, .. } => {
                let mut named =
                    carried_by.map(|surface| name_of_the_surface_either_way_round(&surface));
                named.sort_unstable();
                CornerName::RoundACurve(named)
            }
        };
        if let Some(&already) = self.named.get(&name) {
            return already;
        }
        let point = end.point();
        let at = middle_of(point);
        let was = match end {
            MeetPoint::Existing { vertex, .. } => Some(*vertex),
            MeetPoint::OfThreeSurfaces { .. } | MeetPoint::WhereAClosedCurveStarts { .. } => None,
        };
        let index = match self.at.get(&exactly_at(at)) {
            Some(&already) => {
                self.made[already].wobble = self.made[already].wobble.max(point.uncertainty());
                self.made[already].from = self.made[already].from.or(was);
                already
            }
            None => {
                self.made.push(Corner {
                    point: at,
                    wobble: point.uncertainty(),
                    from: was,
                });
                self.at.insert(exactly_at(at), self.made.len() - 1);
                self.made.len() - 1
            }
        };
        self.named.insert(name, index);
        index
    }
}

/// A point as three bit patterns, for finding two corners that are the same
/// point however they were named.
///
/// Two solids that touch each other have their own vertices at the shared
/// corner, and the two are different input vertices — so the plane names miss
/// each other and only the coordinates say they are one point. Emitting two
/// vertices there would quietly hand back a shell that is not a manifold and
/// call it valid, and `Model::validate` has no way to notice.
///
/// Only an *exact* match counts, and that is the whole of it: two corners the
/// same to within a rounding are not merged, so a pair that touch after a
/// general rotation still come apart. That is roadmap §7.4's vertex-on-face
/// entry, and it is the catalogue's to settle.
fn exactly_at(point: Point) -> [u64; 3] {
    point
        .coordinates()
        // A translation can land a coordinate on negative zero, which is the
        // same number as zero with different bits.
        .map(|coordinate| if coordinate == 0.0 { 0.0 } else { coordinate })
        .map(f64::to_bits)
}

/// What names one edge of the answer: the two corners it runs between, and —
/// for an edge that closes, where those are one corner — the two surfaces
/// either side of it. See [`which_curve_it_is`].
type EdgeName = ([usize; 2], Option<[[u64; 8]; 2]>);

/// The edges built so far, and how to find one again.
#[derive(Default)]
struct Edges {
    made: Vec<Edge>,
    between: BTreeMap<EdgeName, usize>,
}

impl Edges {
    /// The edge between two corners, walked from `from` to `to`, along the
    /// crossing of `carried_by`.
    ///
    /// **The two surfaces are part of what names it and not decoration.** Two
    /// corners no longer pick an edge out on their own: a closed edge starts and
    /// ends at the one corner it has, so a bar's two cap circles would be one
    /// edge if the pair were the key. Sorting the two surfaces is what makes the
    /// faces either side agree about which edge they share.
    ///
    /// # Panics
    ///
    /// If a straight side has no length. §17.4 prevents one rather than
    /// filtering it out afterwards, so one reaching here means the splitter is
    /// wrong — and a *closed* side is not one, which is why the two ends being
    /// the same corner is checked against what carries it.
    fn of(&mut self, meet: &FaceMeet, from: usize, to: usize) -> Side {
        let (carried_by, along) = (meet.carried_by, meet.along.as_slice());
        assert!(
            from != to
                || carried_by
                    .iter()
                    .any(|surface| surface.as_plane().is_none()),
            "a piece of a flat face has a side with no length"
        );
        let at = middle_of(meet.ends[0].point());
        let named = which_curve_it_is(from, to, carried_by);
        let key = ([from.min(to), from.max(to)], named);
        let edge = *self.between.entry(key).or_insert_with(|| {
            self.made.push(Edge {
                ends: [from.min(to), from.max(to)],
                carried_by,
                arc: if from == to {
                    WhichArc::TheOnlyOne
                } else {
                    which_arc_this_is(meet, from, at)
                },
                from: along.to_vec(),
            });
            self.made.len() - 1
        });
        let forward = if from == to {
            which_way_a_closed_edge_is_walked(meet, at)
        } else {
            self.made[edge].ends[0] == from
        };
        Side { edge, forward }
    }
}

/// Whether a face walks a **closed** edge the way that edge is stored running.
///
/// Its two ends are one corner and say nothing, so two facts have to meet.
///
/// **Which way the wire runs in space** is which side of the partner the face's
/// material is on: left of a crossing's forward direction `∇S × ∇T` is always
/// the partner's negative side — `(∇S × ∇T) × ∇S · ∇T = −‖∇S × ∇T‖²` by
/// Cauchy–Schwarz, with no case analysis (§36.6).
///
/// **Which way the edge is stored running** is
/// [`a_closed_crossing_is_walked_the_way_it_is_stored`], which `Model::validate`
/// reads back off the finished model with the same function.
///
/// The piece's two surfaces already point out of its material, so what goes in
/// as the used gradients are the stored ones. Where the rule turns the piece
/// round, `sides_of` turns every side of it round with it and the face's use
/// says so — which is what the sweep reads.
fn which_way_a_closed_edge_is_walked(meet: &FaceMeet, at: Point) -> bool {
    let [mine, partner] = meet.carried_by;
    if the_curved_one_of(&meet.carried_by).is_none() {
        // Two planes cross in a straight line, which does not close, so nothing
        // reaches here — and a wire with no curved surface either side of it has
        // no longitude to run along.
        return true;
    }
    a_closed_crossing_is_walked_the_way_it_is_stored(
        [&mine, &partner],
        [gradient_of(&mine, at), gradient_of(&partner, at)],
        at,
        meet.material_side,
    )
}

/// Which of the two arcs between an edge's two corners this face walks, as the
/// direction the wire leaves its first corner along.
///
/// Two corners on a closed crossing leave two arcs with the same two ends, and
/// a direction into one of them is the whole of what tells them apart — see
/// [`the_half_space_both_ends_are_on`] for what it is turned into.
///
/// **The direction is a sign and not a search.** Left of `∇S × ∇T` is always the
/// partner's negative side (§36.6), so the wire leaves its first corner along
/// that direction exactly when its material is inside the partner — which
/// [`FaceMeet::material_side`] already holds. **No point on the arc is built.**
///
/// # Panics
///
/// If the material side is anything but [`Decision::Neg`] or [`Decision::Pos`].
/// [`FaceMeet::material_side`] is written from which way round the arrangement
/// walked the arc and costs no arithmetic, so anything else is a bug here.
fn which_arc_this_is(meet: &FaceMeet, starts_at: usize, from: Point) -> WhichArc {
    if the_crossing_of_these_runs_straight(&meet.carried_by) {
        return WhichArc::TheOnlyOne;
    }
    let [mine, partner] = meet.carried_by;
    let along_the_crossing = gradient_of(&mine, from).cross(gradient_of(&partner, from));
    WhichArc::LeavingACorner {
        corner: starts_at,
        towards: match meet.material_side {
            Decision::Neg => along_the_crossing,
            Decision::Pos => along_the_crossing * -1.0,
            other => panic!("a side of a face was walked with no material side: {other:?}"),
        },
    }
}

/// Whether these two surfaces cross in straight lines, so that two corners on
/// one leave one piece and not two.
///
/// The same three cases [`walk`](rcad_geom::walk) has nothing to put between the
/// ends of, spelled the same way — a second spelling of this is how the two come
/// to disagree about how many pieces there are. **Read off the surfaces and
/// never off the chord**: a short enough arc has a chord that rounds to its own
/// tangent, and this question has to stay one about the pair.
fn the_crossing_of_these_runs_straight(carried_by: &[Surface; 2]) -> bool {
    match carried_by {
        [Surface::Plane(_), Surface::Plane(_)] => true,
        [Surface::Plane(flat), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Plane(flat)] => {
            flat.normal().dot(pipe.axis().direction()) == 0.0
        }
        [Surface::Cylinder(one), Surface::Cylinder(other)] => {
            one.axis().direction().cross(other.axis().direction()) == Vector::zero()
        }
        _ => false,
    }
}

/// The half-space that holds one of the two arcs between a pair of corners and
/// not the other, or `None` when none can be found.
///
/// What tells them apart is the half-space through the chord **square to the
/// plane the crossing lies in**: that plane meets the crossing's own plane in
/// the chord line, and a line meets a conic at exactly two points, so the two
/// arcs are strictly either side of it whichever one is the longer.
///
/// **The crossing's own plane is never built.** A direction `v` from one end
/// into this arc and the chord `c` both lie in it, so `v × c` is that plane's
/// normal and `c × (v × c)` is the one wanted. Written out that is
/// `v(c·c) − c(c·v)`, whose dot with `v` is `|c|²|v|² − (c·v)²` — never
/// negative, by Cauchy–Schwarz. So the normal already faces the way the arc
/// goes and there is no sign to case out.
///
/// **The offset is proved rather than rounded.** `Model::validate` asks the
/// predicate whether each end is on or in front, and the predicate is exact
/// where the dot products here are not — so the offset comes *down* from the
/// smaller of the two readings, by a step that starts at one rounding of the
/// numbers involved and doubles, until the predicate itself says so. How far
/// down it went is a number nobody needs.
///
/// It stops once the total is past twice the size of the arithmetic, which is
/// past any reading `facing · p` can have by Cauchy–Schwarz, so a half-space
/// that has not been found by then does not exist. Reaching that means `facing`
/// had no length to point along: the chord ran along the crossing's own
/// direction, which a conic's chord does not.
fn the_half_space_both_ends_are_on(facing: Vector, ends: [Point; 2]) -> Option<Picker> {
    let readings = ends.map(|end| facing.dot(end - Point::origin()));
    let size = facing.length()
        * ends
            .iter()
            .map(|end| (*end - Point::origin()).length())
            .fold(0.0, f64::max);
    let from_here = readings[0].min(readings[1]);
    let asked = ends.map(|end| PredicatePoint::from(end.coordinates()));

    let mut down_by = 0.0;
    let mut step = (size * f64::EPSILON).max(f64::MIN_POSITIVE);
    while down_by <= 2.0 * size {
        let picker = Picker::from(HalfSpace::new(facing, from_here - down_by).ok()?);
        if asked
            .iter()
            .all(|end| picker.side_of_point(*end) != Decision::Neg)
        {
            return Some(picker);
        }
        down_by += step;
        step *= 2.0;
    }
    None
}

/// What tells two edges between the same corners apart.
///
/// Two corners pick an edge out on their own while an edge is a straight line
/// between them, and that is every edge a planar fuse makes. **A closed edge
/// starts and ends at the one corner it has**, so a bar's two cap circles are
/// one corner pair twice and the surfaces either side are the only thing left to
/// tell them apart.
///
/// Asked *only* of an edge that closes, and that is not caution. Two solids
/// pressed against each other work the plane they share out separately, and the
/// two faces along one edge of the answer then name their partners with numbers
/// that differ in the last bits — so surfaces in the key of an open edge would
/// split it in two and leave each half with one face along it. Measured: the
/// skew family's not-a-solid share went from under three in a hundred to three
/// and a half.
fn which_curve_it_is(from: usize, to: usize, carried_by: [Surface; 2]) -> Option<[[u64; 8]; 2]> {
    if from != to {
        return None;
    }
    let mut named = carried_by.map(|surface| name_of_the_surface_either_way_round(&surface));
    named.sort_unstable();
    Some(named)
}

/// One wire of a piece as sides of the result, turned round when the rule turns
/// the face round.
///
/// Reversing a wire reverses which way it winds about the face's plane normal,
/// and the face's plane is turned round with it, so the two together leave the
/// wire winding the way [`Model::validate`] asks for. Doing it this way rather
/// than by using the face reversed in its shell is what keeps the two coedges of
/// every edge running opposite ways in the arena, which is the invariant a model
/// is built on.
fn sides_of(
    wire: &[FaceMeet],
    turned_round: bool,
    model: &Model,
    corners: &mut Corners,
    edges: &mut Edges,
) -> Vec<Side> {
    let mut sides = Vec::new();
    for meet in wire {
        let from = corners.of(model, &meet.ends[0]);
        let to = corners.of(model, &meet.ends[1]);
        sides.push(edges.of(meet, from, to));
    }
    if turned_round {
        sides.reverse();
        for side in &mut sides {
            side.forward = !side.forward;
        }
    }
    sides
}

#[cfg(test)]
mod tests {
    use super::{Point, Vector, the_half_space_both_ends_are_on};

    /// The whole of what
    /// [`Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed`] is left refusing,
    /// and a **gap** rather than a limit: the half-space exists and `f64` cannot
    /// reach it.
    ///
    /// Two corners a hair apart on the unit circle. The chord to a point that
    /// far round is `(cos δ − 1, sin δ)`, and for `δ` under an `f64` rounding
    /// `cos δ` is exactly one — so the chord lies exactly along the tangent, the
    /// normal `v(c·c) − c(c·v)` cancels to exactly nothing, and there is no
    /// direction left to separate the two arcs along.
    #[test]
    fn an_arc_whose_chord_rounds_onto_its_own_tangent_is_refused_by_name() {
        let from = Point::new(1.0, 0.0, 0.0);
        let to = Point::new(1.0, 1e-16, 0.0);
        let leaving = Vector::new(0.0, 1.0, 0.0);
        let chord = to - from;

        let facing = leaving * chord.dot(chord) - chord * chord.dot(leaving);
        assert_eq!(
            facing,
            Vector::zero(),
            "an arc this short has a chord along its own tangent"
        );
        assert!(the_half_space_both_ends_are_on(facing, [from, to]).is_none());
    }
}
