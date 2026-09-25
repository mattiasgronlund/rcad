//! Putting the kept pieces back together, and writing them into the arena once.
//!
//! Everything here runs in scratch until the very last step (§17.4). A
//! half-applied boolean is the corrupt solid §4.1 would rather crash than emit,
//! so every question that can be answered `Err` — is the result a manifold, is
//! any shell inside out, which shell is a void of which — is settled before
//! [`Assembly::commit`] touches the model. `commit` builds the answer in a
//! throwaway model of its own and sweeps that, so the last question of all —
//! is what we built even valid — is answered before the caller's arena is
//! touched too (`docs/decisions.md` §87).
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

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use rcad_geom::{
    DynamicRange, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, PlaneAgreement, Point,
    Surface, SurfaceMeet, Vector, which_way_a_closed_crossing_turns,
};
use rcad_predicates::{
    Approximation, Cause, Decision, Measures, Point as PredicatePoint, Policy, ProvenanceLog,
};
use smallvec::SmallVec;

use crate::holds::{
    WhereTheCurveIsCentred, a_picker_that_tells_the_two_arcs_apart,
    where_a_closed_crossing_is_centred,
};
use crate::key::{EdgeSlot, FaceSlot, ShellSlot, SolidSlot, VertexSlot, WireSlot};
use crate::model::{FaceUse, ShellUse, WireStep};
use crate::validate::{
    a_closed_crossing_is_walked_the_way_it_is_stored, gradient_of,
    how_much_space_a_curved_solid_encloses, the_curved_one_of, winding_round,
};
use crate::view::how_far_apart;
use crate::{
    Containment, EdgeKey, Error, FaceKey, Invalid, Model, Result, SolidView, VertexKey, VertexView,
};

use super::arrange::{
    a_plane_cuts_a_cone_in_rulings, a_plane_lies_along_a_pipes_axis, agreement_of_two_planes,
    name_of_surface_bits, name_of_the_surface_either_way_round, two_pipes_run_along_one_line,
};
use super::bake::{grown_by, what_writing_the_corner_down_cost};
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
/// a void has nothing round it, [`Error::Undecidable`] when one shell could not
/// be placed inside or outside another, and
/// [`Error::AClosedEdgeOfTheAnswerWouldStartAtAPole`] when a closed edge's one
/// corner is a pole of the surface it is drawn on.
pub(crate) fn assemble(
    model: &Model,
    scratch: &Scratch,
    chosen: &[Chosen],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Assembly> {
    let mut assembly = Assembly::from_the_pieces(model, scratch, chosen)?;
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

    /// Refuse an answer whose own shortest edge is too small to resolve beside
    /// its own coordinates.
    ///
    /// **A fuse can build a feature neither input had.** Two blocks whose faces
    /// nearly line up meet in a sliver as thin as the gap between the two
    /// planes, and that gap is whatever the two placements make it — a number
    /// with no lower bound short of zero. Both inputs pass
    /// `Model::check_dynamic_range` and the answer does not, so this is the one
    /// thing the fuse writes that its inputs said nothing about.
    ///
    /// It is a **limit** and not a gap. The ratio is a length over a length, so
    /// scaling the whole answer by any power of two leaves it exactly where it
    /// was: there is no scaled copy in which the sliver and the coordinates fit
    /// in `f64` together, which is the family of fix §16.10 asks to be ruled out
    /// by name. Nothing here can make the answer smaller, either — the sliver is
    /// where the two inputs really are.
    ///
    /// Only the chords are measured, where `Model::check_dynamic_range` also
    /// takes the radii of curved faces. A curved face of the answer sits on a
    /// surface inherited from its ancestor bit for bit (§17.1), so every radius
    /// in the answer is a radius an input already carried and was already
    /// measured on.
    ///
    /// # Errors
    ///
    /// [`Invalid::DynamicRangeTooWide`] when the answer's largest coordinate
    /// over its shortest edge is more than `f64` carries, under a policy that
    /// fails on that. [`Error::Geometry`] when two corners of the answer with
    /// different names landed on the very same `f64` triple, which leaves an
    /// edge of no length at all.
    pub(crate) fn check_dynamic_range(
        &self,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let largest = self
            .corners
            .iter()
            .map(|corner| corner.point.largest_coordinate_magnitude())
            .fold(0.0, f64::max);
        // A closed edge has one corner and a chord of zero, and that is not how
        // small it is — the same reasoning, and the same skip, as
        // `Model::check_dynamic_range`.
        let shortest = self
            .edges
            .iter()
            .filter(|edge| edge.ends[0] != edge.ends[1])
            .map(|edge| {
                how_far_apart(
                    self.corners[edge.ends[0]].point,
                    self.corners[edge.ends[1]].point,
                )
            })
            .fold(f64::INFINITY, f64::min);
        if !shortest.is_finite() || largest == 0.0 {
            return Ok(());
        }

        match DynamicRange::measure(largest, shortest)?.check_under_policy(policy, log) {
            // The tag says how much slop an answer this wide carries. What a
            // caller can act on is the record it pushed into `log`, the same as
            // in the sweep.
            Ok(_) => Ok(()),
            // Both layers have a name for this. Keep reporting the topology one,
            // since a fuse is what built it.
            Err(rcad_geom::Error::DynamicRangeTooWide { ratio }) => {
                Err(Invalid::DynamicRangeTooWide { ratio }.into())
            }
            Err(other) => Err(other.into()),
        }
    }

    /// Build the answer twice: once in a throwaway model, to find out whether it
    /// is sound, and then — only if it is — in the caller's.
    ///
    /// **A refusal leaves `model` untouched, down to the last slot.** That is the
    /// whole point of the second write, and it is the shape
    /// [`SolidBuilder::build`](crate::SolidBuilder::build) already had: the
    /// answer used to go straight into the caller's arena and be judged
    /// afterwards, so a broken one was already in their model when the panic
    /// fired, and nothing in this crate rolls back (`docs/decisions.md` §87).
    ///
    /// [`Assembly::write_into`] reads nothing out of the model it writes into and
    /// names no entity that was already there, so writing it twice writes the
    /// same answer twice; `the_same_answer_is_written_into_an_empty_model_and_into_a_busy_one`
    /// measures that rather than asserting it (§16.10).
    ///
    /// **The throwaway model is swept by twelve of `Model::validate`'s thirteen
    /// checks**, [`Model::check_everything_but_the_coordinate_range`]. It holds
    /// the answer and nothing else, so a whole-model sweep of it is already local
    /// to what this fuse wrote — which is what §66.3 wanted and had to hand-cut
    /// eleven checks out of the sweep to get. Sweeping it here gets the
    /// keys-resolve pass back.
    ///
    /// The thirteenth, the coordinate range, is the one this must not ask.
    /// [`Assembly::check_dynamic_range`] asks it before either write and only
    /// behind `the_inputs_fit_in_f64`, so that a caller whose own solids sit
    /// further apart than `f64` can resolve still gets an answer the arithmetic
    /// reached exactly. A fold over a model holding only the answer would put
    /// that question straight back and refuse those answers again (§87.3).
    ///
    /// New corners are tagged with how wide their box is, as
    /// [`Cause::ConstructedPoint`] — representation wobble, which is reported and
    /// never refused on (§17.2). On top of that everything the fuse builds
    /// carries at least the worst tag the inputs carried: a face cut by a face
    /// that was only known to a millimetre is only known to a millimetre itself,
    /// and §17.2 is explicit that a certified fuse carries an input's tag through
    /// rather than refusing it. The throwaway is tagged the same way before it is
    /// swept, because the sweep asks whether a face claims less slop than its own
    /// edges and an untagged answer would be a different question.
    ///
    /// # Errors
    ///
    /// Every [`Invalid`] the sweep finds, as
    /// [`Error::TheFuseWroteABrokenAnswer`]: a shape this crate built and cannot
    /// stand behind, which is a bug here and not bad input — but a bug that
    /// costs the caller nothing, since nothing was written. The blame is blanket
    /// because the sweep no longer asks the one question whose answer was the
    /// caller's geometry rather than this crate's arithmetic.
    ///
    /// Whatever else the sweep says, unchanged — [`Error::Undecidable`] where a
    /// predicate gave up on the answer, say. "The kernel wrote something broken"
    /// is not what those say.
    ///
    /// # Panics
    ///
    /// If the answer that swept clean does not survive the second write, or if
    /// writing it made the model's worst approximation grow past what this fuse
    /// accounted for. **The operation is atomic up to those two, and no
    /// further**: both fire after the real write, so a caller who sees either
    /// has the answer in their model. They are kept as panics because reaching
    /// them means the two writes disagreed, which is corruption in this crate
    /// rather than a broken answer (§4.1, §87).
    ///
    /// [`Assembly::write_into`]: Assembly::write_into
    pub(crate) fn commit(
        &self,
        model: &mut Model,
        inherited: Option<Approximation>,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Written> {
        let mut throwaway = Model::new(model.scale());
        let rehearsal = self.write_into(&mut throwaway, &self.solids, &self.runs);
        self.tag_everything_it_built(&mut throwaway, &rehearsal, inherited);
        let mut mine = ProvenanceLog::new();
        let swept = throwaway.check_everything_but_the_coordinate_range(policy, &mut mine);
        mine.drain_into(log);
        swept.map_err(|refused| {
            let Error::Invalid(broken) = refused else {
                return refused;
            };
            Error::TheFuseWroteABrokenAnswer { broken }
        })?;

        let approximation_before = model.worst_approximation();
        let written = self.write_into(model, &self.solids, &self.runs);
        let worst = self.tag_everything_it_built(model, &written, inherited);
        model.assert_approximation_did_not_grow(approximation_before.max(worst));
        for &solid in &written.solids {
            model.check_touched_solid(solid);
        }
        Ok(written)
    }

    /// One face per kept piece, and the corners and edges they share.
    ///
    /// # Errors
    ///
    /// [`Error::AClosedEdgeOfTheAnswerWouldStartAtAPole`] when a closed edge's
    /// one corner is a pole of the surface it is drawn on, where nothing says
    /// which way it runs.
    fn from_the_pieces(model: &Model, scratch: &Scratch, chosen: &[Chosen]) -> Result<Self> {
        let mut corners = Corners::default();
        let mut edges = Edges::default();
        let mut faces = Vec::new();
        for kept in chosen {
            let piece = &scratch.pieces[kept.piece];
            let turned_round = piece.orientation.is_reversed() != kept.reversed;
            let wires = piece
                .wires
                .iter()
                .map(|wire| {
                    sides_of(
                        wire,
                        turned_round,
                        piece.ancestors[0],
                        model,
                        &mut corners,
                        &mut edges,
                    )
                })
                .collect::<Result<_>>()?;
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
                is_a_whole_closed_surface: piece.is_the_whole_surface,
            });
        }
        Ok(Self {
            corners: corners.made,
            edges: edges.made,
            runs: Vec::new(),
            faces,
            shells: Vec::new(),
            solids: Vec::new(),
        })
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
                is_a_whole_closed_surface: false,
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
        let mut dissolved = self.edges_with_one_face_on_both_sides(&along, log)?;
        let staying = self.edges_a_curved_face_keeps(&along, &dissolved);
        for edge in staying {
            dissolved[edge] = false;
        }
        let same = joined_across(&along, &dissolved, self.faces.len());
        let joined = self.rings_of_each_joined_face(&same, &along, &dissolved)?;
        let renumbered = self.what_is_left(&joined, &along, &dissolved);
        // **A closed curve cut where nothing meets it** is joined back up even
        // where no face was: a pointed cone less a block whose walls are tangent
        // to its rim keeps the rim in arcs otherwise, and a ring of arcs right
        // round a cone with no seam is refused (`AWireWillNotLieFlatOnItsFace`)
        // where the answer is the cone as it stood. Only then: rebuilding every
        // answer dissolves corners at the poles of a ball that the pole rules
        // need, measured as 23 failures across the ball-and-block files.
        if !same.joined_anything() && !renumbered.closed_a_curve_up {
            return Ok(false);
        }
        self.rebuild_from(&joined, renumbered)?;
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
                let (mine, theirs) = (&self.faces[one.face], &self.faces[other.face]);
                match (mine.surface.as_plane(), theirs.surface.as_plane()) {
                    (Some(mine), Some(theirs)) => {
                        let agreement = agreement_of_two_planes(mine, theirs, log)?;
                        Ok(agreement == PlaneAgreement::SameWay)
                    }
                    (None, None) => Ok(one_cone_or_pipe_the_same_way_round(mine, theirs)),
                    (Some(_), None) | (None, Some(_)) => Ok(false),
                }
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
                    is_a_whole_closed_surface: self.faces[only].is_a_whole_closed_surface,
                    rings: (0..self.faces[only].wires.len())
                        .map(|wire| self.whole_ring(only, wire))
                        .collect(),
                    members,
                });
                continue;
            }
            let (surface, used, rings) = match self.faces[members[0]].surface.as_plane() {
                Some(_) => {
                    let plane = plane_of_the_joined_face(&members, &self.faces);
                    let rings = self.rings_round(&members, along, dissolved, plane)?;
                    (Surface::Plane(plane), Orientation::Forward, rings)
                }
                None => {
                    let first = &self.faces[members[0]];
                    (
                        first.surface,
                        first.used,
                        self.rings_of_a_curved_face(&members, along, dissolved),
                    )
                }
            };
            joined.push(JoinedFace {
                members,
                surface,
                used,
                is_a_whole_closed_surface: false,
                rings,
            });
        }
        Ok(joined)
    }

    /// The edges between pieces of one cone or pipe that stay edges after all:
    /// one seam across each band the joining makes, and every edge of a group
    /// that cannot be joined.
    ///
    /// **A ring that goes right round needs a seam**, and the pieces a fuse
    /// leaves carry none — the arrangement cuts a cone's wall into pieces that
    /// each lie flat, and the input's own seam is not among their sides. Joined,
    /// a wall the block took a notch out of is a band with two rings right
    /// round it and nothing across, which is not a face the writer can lay out
    /// flat (`AWireWillNotLieFlatOnItsFace`). So one of the dissolved edges
    /// that runs from one ring to the other stays, as the seam (§224.2 item 2).
    /// Which one is read off its corners, lowest first, and never off its index
    /// here, which depends on the order the solids were handed over — the same
    /// rule [`Assembly::which_to_keep_first`] keeps a corner by.
    ///
    /// Anything else that goes right round — a group with other than two such
    /// rings, no edge across, or a seam that still leaves a ring going round —
    /// and a group whose rings cannot be counted, stays apart. The answer is
    /// then the one the fuse gave before it joined curved faces at all.
    fn edges_a_curved_face_keeps(&self, along: &[[Dart; 2]], dissolved: &[bool]) -> Vec<usize> {
        let same = joined_across(along, dissolved, self.faces.len());
        let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for face in 0..self.faces.len() {
            groups.entry(same.leader(face)).or_default().push(face);
        }
        let mut staying = Vec::new();
        for members in groups.into_values() {
            if members.len() == 1 || self.faces[members[0]].surface.as_plane().is_some() {
                continue;
            }
            let inside: Vec<usize> = (0..self.edges.len())
                .filter(|&edge| dissolved[edge] && members.contains(&along[edge][0].face))
                .collect();
            if self.lies_flat(&members, along, dissolved) {
                continue;
            }
            let seam = self.a_seam_across_the_band(&members, along, dissolved, &inside);
            match seam {
                Some(seam) => staying.push(seam),
                None => staying.extend(inside),
            }
        }
        staying
    }

    /// Whether no ring of the face `members` join into goes right round.
    fn lies_flat(&self, members: &[usize], along: &[[Dart; 2]], dissolved: &[bool]) -> bool {
        let surface = self.faces[members[0]].surface;
        self.rings_of_a_curved_face(members, along, dissolved)
            .iter()
            .all(|ring| self.a_ring_goes_right_round(ring, &surface) == Some(false))
    }

    /// The dissolved edge that runs from one ring of a band to the other with
    /// the lowest corners, where keeping it leaves a face that lies flat.
    fn a_seam_across_the_band(
        &self,
        members: &[usize],
        along: &[[Dart; 2]],
        dissolved: &[bool],
        inside: &[usize],
    ) -> Option<usize> {
        let surface = self.faces[members[0]].surface;
        let rings = self.rings_of_a_curved_face(members, along, dissolved);
        let [one, other] = rings.as_slice() else {
            return None;
        };
        let corners_of = |ring: &[Dart]| -> BTreeSet<usize> {
            ring.iter().map(|&dart| self.starts_at(dart)).collect()
        };
        let (one, other) = (corners_of(one), corners_of(other));
        let lowest_first = |edge: usize| {
            let mut ends = self.edges[edge].ends;
            ends.sort_by(|&a, &b| self.which_to_keep_first(a, b));
            ends
        };
        let seam = inside
            .iter()
            .copied()
            .filter(|&edge| {
                let [a, b] = self.edges[edge].ends;
                (one.contains(&a) && other.contains(&b)) || (one.contains(&b) && other.contains(&a))
            })
            .min_by(|&a, &b| {
                let (a, b) = (lowest_first(a), lowest_first(b));
                self.which_to_keep_first(a[0], b[0])
                    .then_with(|| self.which_to_keep_first(a[1], b[1]))
            })?;
        let mut with_the_seam = dissolved.to_vec();
        with_the_seam[seam] = false;
        let rings = self.rings_of_a_curved_face(members, along, &with_the_seam);
        rings
            .iter()
            .all(|ring| self.a_ring_goes_right_round(ring, &surface) == Some(false))
            .then_some(seam)
    }

    /// The rings round a joined face on a cone or a pipe, in the order they are
    /// first come to.
    ///
    /// **No ring is the outline**, and that is §31.3's rule and not a shortcut:
    /// a curved face's invariant is that every wire keeps the material on its
    /// left, not that the first winds one way and the rest the other. Every dart
    /// walked here came out of a piece that kept its material on its left, so
    /// the rings do too, and there is no winding to ask about.
    fn rings_of_a_curved_face(
        &self,
        members: &[usize],
        along: &[[Dart; 2]],
        dissolved: &[bool],
    ) -> Vec<Vec<Dart>> {
        let mut walked: BTreeSet<Dart> = BTreeSet::new();
        let mut rings = Vec::new();
        for &face in members {
            for wire in 0..self.faces[face].wires.len() {
                for start in self.whole_ring(face, wire) {
                    if dissolved[self.edge_of(start)] || !walked.insert(start) {
                        continue;
                    }
                    rings.push(self.ring_from(start, along, dissolved, &mut walked));
                }
            }
        }
        rings
    }

    /// Whether a ring on a cone or a pipe goes right round its axis — once,
    /// either way — or `None` where the arithmetic here cannot count it.
    ///
    /// The count is the angle about the axis, unwrapped along each side
    /// (§224.2 item 2). A ruling keeps its angle. An arc of a plane's crossing
    /// changes it monotonically — a plane off the apex meets each ruling once
    /// — so its turn is the angle between its ends, taken the way the arc
    /// leaves its named corner. A closed side goes right round, one way or the
    /// other.
    ///
    /// **Which way a closed side goes is never asked**, because the answer does
    /// not need it: a simple ring on a cone or a pipe goes round zero times or
    /// once, so it goes round once exactly when the open sides' turns and the
    /// closed sides between them add up to an odd number of whole turns.
    ///
    /// `None` for a side whose other surface is not a plane, which may cross the
    /// skin in a curve that turns back on itself; for an end on the axis, which
    /// has no angle; and for an arc whose two ends read the same angle to within
    /// what `atan2` rounds, where a sliver and a whole turn look alike.
    fn a_ring_goes_right_round(&self, ring: &[Dart], surface: &Surface) -> Option<bool> {
        let axis = match surface {
            Surface::Cone(cone) => cone.axis(),
            Surface::Cylinder(pipe) => pipe.axis(),
            _ => return None,
        };
        let round = |at: Point| axis.direction().cross(at - axis.origin());
        let mut closed = 0u32;
        let mut turned = 0.0f64;
        for &dart in ring {
            let side = self.side_of(dart);
            let edge = &self.edges[side.edge];
            let partner = edge.carried_by.iter().find(|other| *other != surface)?;
            partner.as_plane()?;
            let [from, to] = [self.starts_at(dart), self.stops_at(side)];
            if from == to {
                closed += 1;
                continue;
            }
            let WhichArc::LeavingACorner { corner, towards } = edge.arc else {
                continue;
            };
            let [start, end] = [from, to].map(|at| round(self.corners[at].point));
            if start == Vector::zero() || end == Vector::zero() {
                return None;
            }
            let apart = axis.direction().dot(start.cross(end)).atan2(start.dot(end));
            if apart.abs() < f64::EPSILON.sqrt() {
                return None;
            }
            let leaving = towards.dot(round(self.corners[corner].point));
            if leaving == 0.0 {
                return None;
            }
            let upwards = (leaving > 0.0) == (corner == from);
            turned += match (upwards, apart > 0.0) {
                (true, true) | (false, false) => apart,
                (true, false) => apart + core::f64::consts::TAU,
                (false, true) => apart - core::f64::consts::TAU,
            };
        }
        let turns = turned / core::f64::consts::TAU;
        let whole = turns.round();
        if (turns - whole).abs() > 0.25 {
            return None;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a ring of a face turns a handful of times at most"
        )]
        let whole = whole as i64;
        Some((whole + i64::from(closed)) % 2 != 0)
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
    /// If more than one ring winds with the plane's normal, or if none does.
    /// Two outlines means the joined region touches itself at a corner, which
    /// is a solid this kernel cannot hold. No outline means the region is
    /// bounded by holes alone, which is a region with no outside — every face
    /// that got here came out of [`Assembly::from_the_pieces`] or
    /// [`Assembly::from_one_input_solid`] with an outline of its own, and
    /// joining only ever takes an edge *away* from between two of them.
    ///
    /// **A ring whose corners enclose nothing is placed by that same rule**,
    /// because its corners cannot place it. A half disk has two corners, so the
    /// hole a half ball pokes through a knife's face, joined across the axis
    /// from the knife's other half, winds round no area at all (§200.1's two
    /// knives at 90 degrees, as a union). With one outline found among the other
    /// rings it is a hole; with none, and it the only such ring, it is the
    /// outline. Two or more such rings and no outline is the one case the rule
    /// cannot settle, and that still panics.
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
        let mut enclosing_nothing = Vec::new();
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
                        Decision::Zero => enclosing_nothing.push(ring),
                        other => panic!(
                            "a ring of a joined face came back winding neither way: {other:?}"
                        ),
                    }
                }
            }
        }
        let outline = match outline {
            Some(outline) => {
                holes.extend(enclosing_nothing);
                outline
            }
            None => {
                assert!(
                    enclosing_nothing.len() == 1,
                    "every face joined here brought an outline of its own and joining only takes \
                     edges away, so the joined face has one too, and {} rings enclose no area to \
                     say which it is",
                    enclosing_nothing.len()
                );
                enclosing_nothing.remove(0)
            }
        };
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

    /// The corner a side arrives at.
    fn stops_at(&self, side: &Side) -> usize {
        self.edges[side.edge].ends[usize::from(side.forward)]
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
    fn rebuild_from(&mut self, joined: &[JoinedFace], renumbered: Renumbered) -> Result<()> {
        let edges = self.edges_after_joining(&renumbered);

        let faces = joined
            .iter()
            .map(|face| {
                Ok(Face {
                    ancestors: each_one_once(
                        face.members
                            .iter()
                            .flat_map(|&member| self.faces[member].ancestors.iter().copied()),
                    ),
                    surface: face.surface,
                    used: face.used,
                    is_a_whole_closed_surface: face.is_a_whole_closed_surface,
                    wires: face
                        .rings
                        .iter()
                        .map(|ring| self.sides_of(ring, &renumbered, &edges))
                        .collect::<Result<_>>()?,
                })
            })
            .collect::<Result<_>>()?;

        self.corners = core::mem::take(&mut self.corners)
            .into_iter()
            .zip(&renumbered.corner_now)
            .filter_map(|(corner, now)| now.map(|_| corner))
            .collect();
        self.edges = edges;
        self.faces = faces;
        Ok(())
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

        // **A corner where pieces of one cone or pipe met stays a corner**, and
        // that departs from §224.2 item 3 on purpose. It has only two surfaces
        // round it, which §31.3 reads out of the model rather than re-deriving,
        // so nothing needs it gone. And folding the arcs either side into one
        // can make an arc longer than half its circle — a frustum's rim with a
        // notch taken out of it — whose centre is then in front of any picker
        // that holds the whole arc, and the next fuse cannot follow it (§224.1).
        let mut where_curved_pieces_met = vec![false; self.corners.len()];
        for (edge, [one, _]) in along.iter().enumerate() {
            if dissolved[edge] && self.faces[one.face].surface.as_plane().is_none() {
                for end in self.edges[edge].ends {
                    where_curved_pieces_met[end] = true;
                }
            }
        }

        let mut straight_through: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut still_a_corner = vec![false; self.corners.len()];
        for (&(lump, corner), by) in &leaving {
            // **Two different edges**, and the word "different" is what a closed
            // edge changes. A circle starts and ends at its one corner, so it
            // leaves that corner twice and this list is one edge written down
            // twice — which is not a run of two edges to fold into one, it is
            // the only corner a closed curve has. Dissolving it leaves the
            // joined edge with no ends at all.
            match by[..] {
                [one, other] if one != other && !where_curved_pieces_met[corner] => {
                    straight_through.insert((lump, corner));
                }
                _ => still_a_corner[corner] = true,
            }
        }
        let kept = self.a_corner_to_keep_on_each_closed_run(&leaving, &straight_through);
        let closed_a_curve_up = !kept.is_empty();
        for (lump, corner) in kept {
            straight_through.remove(&(lump, corner));
            still_a_corner[corner] = true;
        }
        let runs = runs_through(self.edges.len(), &leaving, &straight_through);

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
            closed_a_curve_up,
        }
    }

    /// One corner to keep on each run of edges that closes on itself with every
    /// corner along it walked straight through.
    ///
    /// **A closed curve cut into pieces and joined back up** is such a run: a
    /// pointed cone's rim, cut where a block's walls are tangent to it and
    /// holding its own one corner, keeps two edges at every one of those
    /// corners once the pieces are joined. Folding them all leaves a joined edge
    /// with no ends at all (`fuses_that_touch.rs`,
    /// `a_pointed_cone_on_its_apex_whose_rim_touches_two_walls_fuses_by_every_rule`).
    /// The run becomes one closed edge with one corner, as the input had.
    ///
    /// **Only a run that is all pieces of one input edge**, which is then a
    /// closed input edge given back whole. A ball's meridian circle cut at both
    /// poles is a closed run too, of pieces of two different edges, and closing
    /// it up puts a closed edge's one corner on a pole, where nothing says which
    /// way it runs: 22 tests across the ball-and-block files failed when every
    /// closed run was closed up.
    ///
    /// Keeping every corner instead leaves a ring of arcs going right round a
    /// cone with no seam, which the writer refuses
    /// (`AWireWillNotLieFlatOnItsFace`) — measured on that same pair.
    ///
    /// Which corner is chosen off what the corner *is* and never off its index
    /// here, which depends on the order the solids were handed over: an input
    /// vertex first, then the lowest point.
    fn a_corner_to_keep_on_each_closed_run(
        &self,
        leaving: &BTreeMap<(usize, usize), Vec<usize>>,
        straight_through: &BTreeSet<(usize, usize)>,
    ) -> Vec<(usize, usize)> {
        let runs = runs_through(self.edges.len(), leaving, straight_through);
        let mut corners_of_run: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
        let mut run_keeps_a_corner: BTreeSet<usize> = BTreeSet::new();
        let mut pieces_of: BTreeMap<usize, Vec<&[EdgeKey]>> = BTreeMap::new();
        for (&(lump, corner), by) in leaving {
            if straight_through.contains(&(lump, corner)) {
                corners_of_run
                    .entry(runs.leader(by[0]))
                    .or_default()
                    .push((lump, corner));
            } else {
                run_keeps_a_corner.extend(by.iter().map(|&edge| runs.leader(edge)));
            }
            for &edge in by {
                pieces_of
                    .entry(runs.leader(edge))
                    .or_default()
                    .push(&self.edges[edge].from);
            }
        }
        corners_of_run
            .into_iter()
            .filter(|(run, _)| !run_keeps_a_corner.contains(run))
            .filter(|(run, _)| is_one_input_edge(&pieces_of[run]))
            .map(|(_, corners)| {
                corners
                    .into_iter()
                    .min_by(|&(_, one), &(_, other)| self.which_to_keep_first(one, other))
                    .expect("a run is listed with the corners it walks through")
            })
            .collect()
    }

    fn which_to_keep_first(&self, one: usize, other: usize) -> core::cmp::Ordering {
        let (one, other) = (&self.corners[one], &self.corners[other]);
        let (at_one, at_other) = (one.point.coordinates(), other.point.coordinates());
        one.from.is_none().cmp(&other.from.is_none()).then_with(|| {
            (0..3)
                .map(|axis| at_one[axis].total_cmp(&at_other[axis]))
                .find(|order| order.is_ne())
                .unwrap_or(core::cmp::Ordering::Equal)
        })
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
            // One corner twice is a side that **closes**, not a side of no
            // length — a cap circle and a round hole are exactly that, and
            // `Edge::how_it_runs` reads the pair as `RightRoundWhereItsFacesCross`.
            // What cannot happen is a *straight* side coming back to where it
            // started: two planes cross in a line and a line does not close.
            let flat = self.edges[leader]
                .carried_by
                .iter()
                .all(|surface| surface.as_plane().is_some());
            assert!(
                ends[0] != ends[1] || !flat,
                "a piece of a face has a straight side with no length"
            );
            made.push(Edge {
                ends: ends.map(|end| {
                    renumbered.corner_now[end]
                        .expect("an end the lump does not walk straight through is still a corner")
                }),
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
        // A run that closed up is the whole curve, and a whole curve is the one
        // arc there is.
        if ends[0] == ends[1] || arcs.iter().all(|arc| matches!(arc, WhichArc::TheOnlyOne)) {
            return WhichArc::TheOnlyOne;
        }
        WhichArc::LeavingACorner {
            corner: renumber(ends[0]),
            towards: self.corners[inside].point - self.corners[ends[0]].point,
        }
    }

    fn sides_of(
        &self,
        ring: &[Dart],
        renumbered: &Renumbered,
        edges: &[Edge],
    ) -> Result<Vec<Side>> {
        let lump = renumbered.lump_of_edge[self.edge_of(ring[0])]
            .expect("a ring is walked along edges that stay");
        let opens = ring
            .iter()
            .position(|&dart| {
                !renumbered
                    .straight_through
                    .contains(&(lump, self.starts_at(dart)))
            })
            .expect(
                "a corner walked straight through has the two edges either side of it in one \
                 straight run, and a ring cannot be straight the whole way round, so some corner \
                 of it is still a corner",
            );

        let mut sides: Vec<Side> = Vec::new();
        for step in 0..ring.len() {
            let dart = ring[(opens + step) % ring.len()];
            // A dart whose corner the joining dissolved is the rest of the side
            // before it and not a side of its own. Skipping on the *corner*
            // rather than on the edge is what leaves a seam alone: a seam is one
            // edge walked twice by one wire, so each of its poles has exactly one
            // edge of this lump leaving it and neither is a corner anything
            // dissolved. A corner is counted per lump, so another solid's vertex
            // landing on a pole is a corner of *its* lump and not of this one.
            if renumbered
                .straight_through
                .contains(&(lump, self.starts_at(dart)))
            {
                continue;
            }
            let edge = renumbered.runs.leader(self.edge_of(dart));
            let edge = renumbered.edge_now[edge].expect("a run of edges that stays becomes one");
            let from = renumbered.corner_now[self.starts_at(dart)]
                .expect("the step just above skipped every corner walked straight through");
            // A **closed** edge starts and ends at its one corner, so the corner
            // says nothing about which way this face walks it. A closed edge
            // that was closed already keeps the side it had: its corner has it
            // twice and nothing else, so nothing was joined onto it. One that a
            // run of pieces closed up into is asked of the surfaces.
            let piece = &self.edges[self.edge_of(dart)];
            sides.push(Side {
                edge,
                forward: if edges[edge].ends[0] != edges[edge].ends[1] {
                    edges[edge].ends[0] == from
                } else if piece.ends[0] == piece.ends[1] {
                    self.side_of(dart).forward
                } else {
                    self.walks_a_closed_up_run_the_way_it_is_stored(dart)?
                },
            });
        }
        // Three sides, **or one that closes on itself**, which is the rule
        // `Model::validate` reads: a cap circle and the rim of a round hole are
        // one whole curve and have one corner between them. A seam is the third
        // shape a wire comes in — one edge walked both ways, with a pole between
        // the two walks — and it has four steps once the poles are counted. And
        // **two different sides that are not both straight** bound a face too, as
        // `Model::wire_of_two_steps_bounds_a_face` has it: a chord and the arc
        // across it, which is the half disk a flat cut leaves inside a ball.
        let straight = |side: &Side| {
            edges[side.edge]
                .carried_by
                .iter()
                .all(|surface| surface.as_plane().is_some())
        };
        let bounds_something = match &sides[..] {
            [only] => edges[only.edge].ends[0] == edges[only.edge].ends[1],
            [one, other] => one.edge != other.edge && !(straight(one) && straight(other)),
            _ => false,
        };
        assert!(
            sides.len() >= 3
                || bounds_something
                || self.faces[ring[0].face].is_a_whole_closed_surface,
            "a face has at least three sides, one that closes, or two not both straight; \
             this one has {}",
            sides.len()
        );
        Ok(sides)
    }

    /// Whether a face that walks `piece` away from `from` walks the closed edge
    /// that piece was joined into the way that edge is stored running.
    ///
    /// The way the face walks is read off the piece's own arc, which names a
    /// direction along the crossing out of one of its corners: the face leaves
    /// `from` along it, so it runs that way where the arc is named from `from`
    /// and the other way where it is named from the far end, which the face
    /// arrives at. Against `∇S × ∇T` of the two surfaces that is a sign, and a
    /// face whose material is inside its partner walks along `∇S × ∇T` — so the
    /// sign is handed to [`a_closed_crossing_is_walked_the_way_it_is_stored`] as
    /// that material side, and the stored way is read with the rule
    /// `Model::validate` reads it back with.
    ///
    /// # Errors
    ///
    /// [`Error::CannotTellWhichWayAClosedEdgeRuns`] where the rule cannot tell,
    /// and [`Error::AClosedEdgeOfTheAnswerWouldStartAtAPole`] where it says the
    /// crossing has no direction at the arc's corner.
    ///
    /// # Panics
    ///
    /// If the piece's arc is not named from a corner — a piece of a closed curve
    /// between two different corners always is — or names a direction square to
    /// the crossing.
    fn walks_a_closed_up_run_the_way_it_is_stored(&self, dart: Dart) -> Result<bool> {
        let piece = &self.edges[self.edge_of(dart)];
        let from = self.starts_at(dart);
        let WhichArc::LeavingACorner { corner, towards } = piece.arc else {
            panic!("a piece of a closed curve between two corners names which arc it is");
        };
        let at = self.corners[corner].point;
        let [one, other] = piece.carried_by;
        let outward = [gradient_of(&one, at), gradient_of(&other, at)];
        let along = towards.dot(outward[0].cross(outward[1]));
        assert!(
            along != 0.0,
            "an arc's direction out of its corner is along its crossing"
        );
        let walks_along_the_cross = (along > 0.0) == (corner == from);
        let material_side = if walks_along_the_cross {
            Decision::Neg
        } else {
            Decision::Pos
        };
        a_closed_crossing_is_walked_the_way_it_is_stored(
            [&one, &other],
            outward,
            at,
            material_side,
        )?
        .ok_or(Error::AClosedEdgeOfTheAnswerWouldStartAtAPole {
            face: self.faces[dart.face].ancestors[0],
        })
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
            let volume = how_much_space_it_encloses(apart.solid_at(solid), policy, log)?;
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
        let solid = apart.solid_at(written.solids[outer]);
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
                    | Containment::CannotFollowAFacesTrimming { .. }
                    | Containment::CannotFireARayAtAFacesSurface { .. } => (),
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
        let cut_by = self.the_surface_that_cut_each_closed_rim();
        self.edges
            .iter()
            .zip(cut_by)
            .map(|(edge, cut_by)| edge.how_it_runs(&self.corners, cut_by))
            .collect()
    }

    /// Which surface cut each edge, for a closed rim whose two faces **in the
    /// answer** sit on the one surface and so name nothing between them.
    ///
    /// [`Edge::carried_by`] is the pair of faces that *met*, so the cutting
    /// plane is in scope down in [`Edge::how_it_runs`]. What is not in scope
    /// there is which two faces of the answer ended up either side of the
    /// finished rim, and that is what decides this: a round bar's cap circle has
    /// a cylinder one side and a plane the other, and the two of them say where
    /// it runs; a frustum's wall cut in two by a tool plane has the one cone
    /// both sides, which says nothing at all, and
    /// [`EdgeRun::RightRoundWhereItCrosses`] is that rim naming the surface that
    /// cut it instead (`docs/decisions.md` §216).
    ///
    /// `None` everywhere else, which is where [`EdgeRun::RightRoundWhereItsFacesCross`]
    /// stays. Three shapes come back `None` for a reason worth naming: an edge
    /// that is not closed, which is not this kind of run at all; an edge other
    /// than two faces claim, which [`Assembly::group_into_shells`] refuses as
    /// [`Error::ResultIsNotManifold`] a step later under its own name rather
    /// than here; and a crossing whose pair does not name one surface of the
    /// answer and one other, which leaves nothing to put in `and`.
    fn the_surface_that_cut_each_closed_rim(&self) -> Vec<Option<Surface>> {
        let mut claimants: Vec<Vec<usize>> = vec![Vec::new(); self.edges.len()];
        for (face, walked) in self.faces.iter().enumerate() {
            for side in walked.wires.iter().flatten() {
                claimants[side.edge].push(face);
            }
        }
        self.edges
            .iter()
            .zip(&claimants)
            .map(|(edge, by)| {
                if edge.ends[0] != edge.ends[1] {
                    return None;
                }
                let [one, other] = <[usize; 2]>::try_from(by.as_slice()).ok()?;
                let either_side = self.faces[one].surface;
                if self.faces[other].surface != either_side {
                    return None;
                }
                match edge.carried_by {
                    [met, across] if met == either_side && across != either_side => Some(across),
                    [across, met] if met == either_side && across != either_side => Some(across),
                    _ => None,
                }
            })
            .collect()
    }

    /// Build every entity of the result in `model`, in a fixed order.
    ///
    /// Used three times: once on a scratch model to ask which shell is inside
    /// which, once on a throwaway model to find out whether the answer sweeps
    /// clean, and once on the caller's model, which is the commit.
    ///
    /// **It judges nothing.** [`Model::check_touched_solid`] used to run here on
    /// each solid as it was built, which put a panic in front of every one of
    /// those three writes — including the throwaway one whose whole job is to
    /// turn a broken answer into an [`Error`] (`docs/decisions.md` §87).
    /// [`Assembly::commit`] asks it of the real write instead.
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
                let mut steps: SmallVec<[WireStep; 4]> = SmallVec::new();
                for side in wire {
                    let turn = if side.forward {
                        Orientation::Forward
                    } else {
                        Orientation::Reversed
                    };
                    steps.push(WireStep::Along(model.add_coedge(edges[side.edge], turn)));
                    // A whole closed surface is cut open along a seam, and a
                    // seam ends where the surface pinches to a point. The
                    // boundary has to go somewhere between arriving at a pole
                    // and leaving it, and where it goes is nowhere at all —
                    // which is `WireStep::Pole` and the reason it exists.
                    if face.is_a_whole_closed_surface {
                        steps.push(WireStep::Pole(vertices[self.stops_at(side)]));
                    }
                }
                wires.push(model.add_wire_of_steps(steps));
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
            solids.push(model.add_solid(used));
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
///
/// The four arenas anybody asks about afterwards. A solid names its shells,
/// which name their faces, which name their wires, which name their coedges, so
/// the other three are reachable from `solids` and listing them as well would be
/// a second copy of what the arena already says.
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
    /// `cut_by` is the one thing this cannot work out for itself:
    /// [`Assembly::the_surface_that_cut_each_closed_rim`] says why, and a rim
    /// that comes with one names it rather than its two faces.
    ///
    /// # Errors
    ///
    /// [`Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed`] for an arc of a
    /// curved crossing that nothing told from its sibling.
    fn how_it_runs(&self, corners: &[Corner], cut_by: Option<Surface>) -> Result<EdgeRun> {
        let flat = self
            .carried_by
            .iter()
            .all(|surface| surface.as_plane().is_some());
        match (flat, self.ends[0] == self.ends[1]) {
            (true, false) => Ok(EdgeRun::Straight),
            (_, true) => Ok(match cut_by {
                Some(and) => EdgeRun::RightRoundWhereItCrosses { and },
                None => EdgeRun::RightRoundWhereItsFacesCross,
            }),
            (false, false) => match self.arc {
                WhichArc::TheOnlyOne => Ok(EdgeRun::Straight),
                WhichArc::LeavingACorner { corner, towards } => {
                    let ends = self.ends.map(|end| corners[end].point);
                    if let Some(branch) = self.the_branch_of_a_hyperbola_this_is_on(ends)? {
                        return Ok(EdgeRun::PartOfWhereItCrosses {
                            and: self.the_surface_that_names_it_twice(),
                            picked_by: SmallVec::from_buf([branch]),
                        });
                    }
                    let chord = corners[self.other_end_than(corner)].point - corners[corner].point;
                    let facing = towards * chord.dot(chord) - chord * chord.dot(towards);
                    let flat = self.carried_by.iter().find_map(Surface::as_plane);
                    let centred = flat.and_then(|flat| {
                        let centre = self
                            .carried_by
                            .iter()
                            .find_map(|surface| where_a_closed_crossing_is_centred(surface, flat));
                        Some(flat).zip(centre)
                    });
                    let picker = the_half_space_both_ends_are_on(
                        facing,
                        ends,
                        centred.as_ref().map(|(flat, centre)| (*flat, centre)),
                    )
                    .ok_or(Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed)?;
                    Ok(EdgeRun::PartOfWhereItCrosses {
                        and: self.the_surface_that_names_it_twice(),
                        picked_by: SmallVec::from_buf([picker]),
                    })
                }
            },
        }
    }

    /// The half-space holding the one nappe this edge runs on, when it runs
    /// along a plane crossing a cone in a **hyperbola**, and `None` for every
    /// other crossing.
    ///
    /// **The half-space through the chord does not name an arc of a
    /// hyperbola.** A line meets a hyperbola in at most two points, both on the
    /// branch the chord joins, so the whole of the *other* branch lies on one
    /// side of the chord — and it is always the side the arc bulges to. The chord
    /// half-space holds the arc and the whole other branch: two pieces, where
    /// [`EdgeRun::PartOfWhereItCrosses`] promises one. And `rcad_geom::walk`
    /// picks the branch by asking the pickers at each branch's vertex, which
    /// holds only for a picker square to the axis; asked of a chord half-space it
    /// picks the other branch whenever the arc does not run over its own vertex.
    /// Measured on the lens and bar of
    /// `tests/the_way_into_an_ellipse_arc_in_a_fuse.rs`: the union's arc along
    /// `x = 0.5` from `(0.5, −1.68, 3.5)` to `(0.5, −2.19, 4.5)` was walked
    /// through `(0.5, −1.92, −3.97)`, its mirror on the nappe below the apex, and
    /// the cone face spanning both nappes came back from the mesher as a
    /// coordinate too large to triangulate.
    ///
    /// **A branch is an open curve, so its two ends name one arc of it**, and
    /// the nappe is all the picking there is to do. The picker is the one
    /// [`rcad_geom::intersect`] mints for that branch — the half-space through
    /// the apex square to the axis, its offset proved against the branch's own
    /// vertex — chosen as the one that holds both ends.
    ///
    /// Both signs are exact polynomials in stored `f64`, so no policy changes
    /// them; `intersect` is asked under a certified one because this has none to
    /// hand down, the same choice `rcad_geom::where_a_closed_crossing_starts`
    /// makes and for the same reason.
    ///
    /// # Errors
    ///
    /// [`Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed`] when either sign
    /// is undecided, when `intersect` cannot tell the branches apart, or when
    /// the ends are not both held by exactly one branch's half-space.
    fn the_branch_of_a_hyperbola_this_is_on(&self, ends: [Point; 2]) -> Result<Option<Picker>> {
        let ([Surface::Plane(flat), Surface::Cone(cone)]
        | [Surface::Cone(cone), Surface::Plane(flat)]) = self.carried_by
        else {
            return Ok(None);
        };
        let apex = rcad_geom::where_a_cones_apex_stands_against_a_plane(&flat, &cone);
        if apex == Decision::Zero {
            return Ok(None);
        }
        if apex != Decision::Pos && apex != Decision::Neg {
            return Err(Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed);
        }
        let leaning = rcad_geom::how_a_plane_leans_against_a_cone(&flat, &cone);
        if leaning == Decision::Pos || leaning == Decision::Zero {
            return Ok(None);
        }
        if leaning != Decision::Neg {
            return Err(Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed);
        }
        let Ok(SurfaceMeet::Curves(branches)) = rcad_geom::intersect(
            &Surface::Plane(flat),
            &Surface::Cone(cone),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) else {
            return Err(Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed);
        };
        let asked = ends.map(|end| PredicatePoint::from(end.coordinates()));
        let holding: Vec<Picker> = branches
            .iter()
            .filter_map(|branch| {
                let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = branch.run() else {
                    return None;
                };
                let [one] = picked_by.as_slice() else {
                    return None;
                };
                Some(*one)
            })
            .filter(|picker| {
                asked
                    .iter()
                    .all(|end| picker.side_of_point(*end) == Decision::Pos)
            })
            .collect();
        match holding.as_slice() {
            [only] => Ok(Some(*only)),
            _ => Err(Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed),
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
    /// `Model::validate` asks that both ends are not *provably* off `and`, and a
    /// flat `and` is the cheapest of the three to be sure about. So the flat one
    /// goes in wherever there is one, and `name_of_surface_bits` gives that for
    /// free by sorting a plane ahead of a ball or a pipe.
    ///
    /// **The sentence that used to stand here said the check was an exact
    /// `Zero`, and that a corner this fuse solved for is exactly on a plane it
    /// was solved from.** Both halves were wrong, and the second is why nobody
    /// checked the first: a corner is `through + t·direction` over a divisor, and
    /// it lands on the plane exactly only when the line's direction is
    /// axis-aligned. A block turned against a bar is enough to break it, and the
    /// fix is the one the old sentence named as future work — the same slop
    /// `check_curved_face` reads off a solved corner (§36.13), read in
    /// `check_edges` too. §39.
    ///
    /// A crossing with **two** curved surfaces and two ends is a pair of balls
    /// cut twice. Two balls united, which cut each other once, are answered
    /// since §202.1. `fuse([ballA], [ballB, cube])` is a pair of
    /// balls cut twice and is reachable. What stops *it* is a crate down —
    /// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, because a corner of such an arc
    /// has two quadrics round it (§31.2's limit under row 4).
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
    /// Whether this face is the **whole** of a closed surface, so that its one
    /// wire is a seam and its two corners are poles the boundary stands still
    /// at rather than corners it turns at (§38.1).
    is_a_whole_closed_surface: bool,
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
    /// Whether some closed curve cut into pieces is joined back into one edge.
    closed_a_curve_up: bool,
}

/// One face of the result while it is being put back together out of the pieces
/// that turned out to be one flat region.
struct JoinedFace {
    /// The faces it was made of, in the order they were built.
    members: Vec<usize>,
    surface: Surface,
    used: Orientation,
    /// Carried through the joining untouched: only flat faces and pieces of a
    /// cone or a pipe are ever joined, and a whole closed surface is a ball.
    is_a_whole_closed_surface: bool,
    /// On a plane, the outline first and then one ring per hole. On a cone or a
    /// pipe, in the order they were first come to (§31.3).
    rings: Vec<Vec<Dart>>,
}

/// How much space one shell of the answer encloses, signed by which way its
/// faces point.
///
/// A polyhedron's own fan *is* its volume and is summed exactly. A **curved**
/// shell has no such fan — a bar's wall fans into two triangles on its seam and
/// a round cap into none at all (§35.4) — so it goes to
/// [`how_much_space_a_curved_solid_encloses`], which is the same question
/// `Model::validate` asks one layer up and now the same code as well. Only the
/// sign is read here.
///
/// **Both halves of that are asked here rather than left to the sweep, and that
/// is the difference between a refusal and a corrupt solid.**
/// `Assembly::commit` re-validates what it wrote, but only in a debug build and
/// only as a *panic* — it is there to catch a bug in this crate, and a release
/// build would write the answer out regardless. A curved answer whose two faces
/// along one side disagree which way they face is exactly that, so it is asked
/// before anything is written and comes back as a refusal (§4.1: refusing beats
/// corrupting, and both beat crashing on a caller's solid).
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
    how_much_space_a_curved_solid_encloses(solid, policy, log)
}

/// Whether two curved faces are pieces of one cone or one pipe, used the same
/// way round, so that an edge between them is not boundary at all.
///
/// **The surface's own bits decide it, and nothing else does** (§224.2 item 1).
/// A piece inherits its surface from its ancestor bit for bit (§31.3), so two
/// pieces of one input face carry the same bits, and those are the pieces a
/// fuse leaves cut apart. The same surface used the other way round is a sheet
/// of no thickness with material either side, which is two faces and not one.
/// A surface that is only the same up to how it is written — its axis turned
/// round, say — is left apart: nothing here proves the two are one surface,
/// and the answer with them apart is the sound one the fuse always gave.
///
/// **Only a cone or a pipe.** Their rings are asked how often they go round the
/// axis ([`Assembly::a_ring_goes_right_round`]), and a ball or a ring has no
/// one axis to ask about, so pieces of those stay apart.
fn one_cone_or_pipe_the_same_way_round(mine: &Face, theirs: &Face) -> bool {
    matches!(mine.surface, Surface::Cone(_) | Surface::Cylinder(_))
        && name_of_surface_bits(&mine.surface) == name_of_surface_bits(&theirs.surface)
        && mine.used == theirs.used
}

/// Which faces are one once every dissolved edge is gone.
fn joined_across(along: &[[Dart; 2]], dissolved: &[bool], faces: usize) -> Sameness {
    let mut same = Sameness::of(faces);
    for (edge, [one, other]) in along.iter().enumerate() {
        if dissolved[edge] {
            same.join(one.face, other.face);
        }
    }
    same
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

/// Whether every piece of a run was cut out of the one same input edge and
/// nothing else.
fn is_one_input_edge(pieces: &[&[EdgeKey]]) -> bool {
    pieces.first().is_some_and(|first| first.len() == 1)
        && pieces.iter().all(|piece| *piece == pieces[0])
}

/// Which run of edges each edge is part of, when the edges either side of every
/// corner walked straight through are one run.
fn runs_through(
    edges: usize,
    leaving: &BTreeMap<(usize, usize), Vec<usize>>,
    straight_through: &BTreeSet<(usize, usize)>,
) -> Sameness {
    let mut runs = Sameness::of(edges);
    for (at, by) in leaving {
        if straight_through.contains(at) {
            runs.join(by[0], by[1]);
        }
    }
    runs
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
    Meet([[u64; 9]; 3], u8),
    /// The one point on a closed crossing that nothing else cuts, named by the
    /// two surfaces it runs between.
    RoundACurve([[u64; 9]; 2]),
    /// The point across a closed crossing from where it starts, named the same
    /// way, for a crossing that starts at a pole.
    HalfWayRoundACurve([[u64; 9]; 2]),
    /// A pole of a closed surface, named by that surface and which of its two
    /// poles this is (§38.1).
    Pole([u64; 9], bool),
}

impl Corners {
    fn of(&mut self, model: &Model, end: &MeetPoint) -> usize {
        let name = match end {
            MeetPoint::Existing { vertex, .. } => CornerName::Vertex(vertex.slot_in(model.id())),
            MeetPoint::OfThreeSurfaces {
                surfaces, which, ..
            } => CornerName::Meet(names_of_the_three_surfaces(surfaces), *which),
            MeetPoint::WhereAClosedCurveStarts { carried_by, .. } => {
                CornerName::RoundACurve(names_of_the_two_surfaces(carried_by))
            }
            MeetPoint::HalfWayRoundAClosedCurve { carried_by, .. } => {
                CornerName::HalfWayRoundACurve(names_of_the_two_surfaces(carried_by))
            }
            MeetPoint::APoleOfAClosedSurface {
                surface,
                at_the_top,
                ..
            } => CornerName::Pole(name_of_surface_bits(surface), *at_the_top),
        };
        let point = end.point();
        let at = middle_of(point);
        if let Some(&already) = self.named.get(&name) {
            self.settle_on_the_lower_rounding(already, at, point.uncertainty());
            return already;
        }
        let was = match end {
            MeetPoint::Existing { vertex, .. } => Some(*vertex),
            MeetPoint::OfThreeSurfaces { .. }
            | MeetPoint::WhereAClosedCurveStarts { .. }
            | MeetPoint::HalfWayRoundAClosedCurve { .. }
            | MeetPoint::APoleOfAClosedSurface { .. } => None,
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

    /// Keep the lower of two roundings of one corner, so that which face asked
    /// first does not decide where it is.
    ///
    /// Two spellings of one name solve for the same corner and Cramer's rule
    /// rounds the two a bit apart — [`names_of_the_three_surfaces`] says how a
    /// plane comes to be spelled two ways, and sorting the three planes puts a
    /// reversed one in a different row, which permutes the solve. Both answers
    /// are inside the box the meet handed back and neither is truer than the
    /// other, so all that is wanted is that the same one wins every time. First
    /// come does not do it: swapping the two sides of a fuse walks the faces in
    /// the other order and the other rounding wins, and a `fuse(A, B)` whose
    /// corner sits a bit off `fuse(B, A)`'s is exactly what §17.4 forbids.
    ///
    /// **This is not a coordinate deciding what is one corner.** The name has
    /// already decided that. It picks between two roundings of a corner both
    /// spellings agree on, and the wobble carries how far apart they were.
    fn settle_on_the_lower_rounding(&mut self, corner: usize, at: Point, uncertainty: f64) {
        let made = &mut self.made[corner];
        made.wobble = made.wobble.max(uncertainty);
        let lower = at
            .coordinates()
            .iter()
            .zip(made.point.coordinates().iter())
            .map(|(mine, theirs)| mine.total_cmp(theirs))
            .find(|order| order.is_ne())
            .is_some_and(Ordering::is_lt);
        if lower {
            let left_behind = exactly_at(made.point);
            made.point = at;
            // No corner sits on the rounding this one just left, so a later
            // corner of another name landing there would be joined to this one
            // over a coordinate neither of them is on (§49.3). Another corner
            // that has since settled onto that coordinate *is* on it, and keeps
            // its claim.
            if self.at.get(&left_behind) == Some(&corner) {
                self.at.remove(&left_behind);
            }
            self.at.insert(exactly_at(at), corner);
        }
    }
}

/// The three names in a [`CornerName::Meet`], sorted so the three arrive the
/// same way round whichever face asked.
///
/// **Three planes are named whichever way each of them faces** (§48, W5). Two
/// solids flush against each other work the plane they share out separately and
/// one of them names it reversed, so a corner on that plane is solved from
/// `P, Q, R` by one face and from `−P, Q, R` by another. The two are one corner
/// and Cramer's rule rounds them a bit or two apart, so the coordinates do not
/// join them either: the answer got two corners, the edge between them split in
/// two, and each half was left with one face along it. It is the hazard
/// [`name_of_the_surface_either_way_round`] was written for, arriving at the
/// corners instead of at the edges.
///
/// **A plane in a triple with something curved in it is named as each face
/// spells it**, and that is not caution left in for its own sake. Three surfaces
/// meet at up to four points; `which` numbers them along the line the two planes
/// cross in; and reversing a plane reverses that line, so the same point is
/// `which: 0` under one spelling and `which: 1` under the other. Folding the
/// spellings together there would name two *different* corners alike, which is
/// worse than leaving two names on one. Three planes have no such point to
/// confuse: they meet at exactly one place
/// ([`MeetOfThree::At`](rcad_geom::MeetOfThree) with a single entry), so `which`
/// is always zero and the three names settle the corner on their own.
///
/// **A pipe, a cone or a bend is folded whatever else is in the triple**, and
/// the reason the plane rule does not carry over is that a curved surface is
/// never one of the two planes whose line `which` is counted along. A plane
/// sorts before every curved kind, and two of the three must be flat for the
/// meet to be worked out at all, so the curved one is always the third row and
/// the line is fixed by the other two. Turning its axis round is then neutral to
/// the whole solve, which
/// `turning_a_stored_axis_round_moves_no_corner_and_renumbers_none` measures
/// over 12 960 triples of pipes, cones and bends: same points, same order, bit
/// for bit.
fn names_of_the_three_surfaces(surfaces: &[Surface; 3]) -> [[u64; 9]; 3] {
    let all_flat = surfaces.iter().all(|surface| surface.as_plane().is_some());
    let mut named = surfaces.map(|surface| {
        if all_flat || surface.as_plane().is_none() {
            name_of_the_surface_either_way_round(&surface)
        } else {
            name_of_surface_bits(&surface)
        }
    });
    named.sort_unstable();
    named
}

/// The two surfaces a closed crossing runs between, named either way round and
/// sorted, so both faces along it name its corners alike.
fn names_of_the_two_surfaces(carried_by: &[Surface; 2]) -> [[u64; 9]; 2] {
    let mut named = carried_by.map(|surface| name_of_the_surface_either_way_round(&surface));
    named.sort_unstable();
    named
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
/// either side of it. See [`which_curve_it_is`]. The third is which of the two
/// arcs between the same two corners it is of a closed crossing between a plane
/// and a surface that curves right round — a ball's circle, or a cone's rim
/// circle; see [`which_arc_of_a_circle_this_is`].
///
/// The fourth is whether the edge is straight: both surfaces either side are
/// planes. **A straight side and a curved one can share both corners**, and
/// they are two edges. A half disk is one: `z ≤ 0.5` cut out of a half ball
/// leaves on the plane `z = 0.5` a chord along `x = 0` and an arc round the
/// ball, both from `(0, −√3/2, 0.5)` to `(0, √3/2, 0.5)`, and named by their
/// corners alone the two were one edge with four faces along it. Whether the
/// surfaces are planes is not a number, so it cannot split one edge in two
/// the way the surfaces' own names would (see [`which_curve_it_is`]).
type EdgeName = ([usize; 2], Option<[[u64; 9]; 2]>, Option<bool>, bool);

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
    ///
    /// `turned_round` is whether the rule turns this face's piece round, which
    /// only a closed edge starting at a ball's pole needs to know — see
    /// [`the_way_a_circle_from_a_pole_is_walked`].
    fn of(
        &mut self,
        meet: &FaceMeet,
        from: usize,
        to: usize,
        face: FaceKey,
        turned_round: bool,
    ) -> Result<Side> {
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
        let key = (
            [from.min(to), from.max(to)],
            named,
            which_arc_of_a_circle_this_is(meet, from, to),
            carried_by
                .iter()
                .all(|surface| surface.as_plane().is_some()),
        );
        let edge = match self.between.get(&key) {
            Some(&already) => already,
            None => {
                let arc = if from == to {
                    WhichArc::TheOnlyOne
                } else {
                    which_arc_this_is(meet, from, at)?
                };
                self.made.push(Edge {
                    ends: [from.min(to), from.max(to)],
                    carried_by,
                    arc,
                    from: along.to_vec(),
                });
                self.between.insert(key, self.made.len() - 1);
                self.made.len() - 1
            }
        };
        let forward = if from == to {
            which_way_a_closed_edge_is_walked(meet, at, turned_round)?
                .ok_or(Error::AClosedEdgeOfTheAnswerWouldStartAtAPole { face })?
        } else {
            self.made[edge].ends[0] == from
        };
        Ok(Side { edge, forward })
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
///
/// # Errors
///
/// Whatever [`a_closed_crossing_is_walked_the_way_it_is_stored`] gives up with,
/// which is [`Error::CannotTellWhichWayAClosedEdgeRuns`] when the wire round the
/// crossing is shorter than the rounding in the cross product that built it
/// (§111).
///
/// **This used to be an `.expect`**, on the argument that the flat check below
/// ruled out the one error the rule could give. That argument was about
/// [`Invalid::ClosedEdgeBetweenTwoFlatFaces`] and stayed written down after a
/// second error arrived beside it. A fuse's arguments are a caller's solids and
/// a reader's file, so a crossing this thin is caller input and gets an `Err`
/// rather than a panic — and a green gate is not a demonstration that no input
/// reaches it (§16.10).
fn which_way_a_closed_edge_is_walked(
    meet: &FaceMeet,
    at: Point,
    turned_round: bool,
) -> Result<Option<bool>> {
    let [mine, partner] = meet.carried_by;
    if the_curved_one_of(&meet.carried_by).is_none() {
        // Two planes cross in a straight line, which does not close, so nothing
        // reaches here — and a wire with no curved surface either side of it has
        // no longitude to run along.
        return Ok(Some(true));
    }
    let outward = [gradient_of(&mine, at), gradient_of(&partner, at)];
    match a_closed_crossing_is_walked_the_way_it_is_stored(
        [&mine, &partner],
        outward,
        at,
        meet.material_side,
    )? {
        Some(forward) => Ok(Some(forward)),
        None => {
            let Some(lid) = the_lid_a_circle_from_a_pole_bounds(meet, turned_round) else {
                return Ok(None);
            };
            let runs_along_stored = match which_way_a_closed_crossing_turns(
                outward,
                gradient_of(&lid.ball, at),
                lid.outward,
            ) {
                Decision::Pos => true,
                Decision::Neg => false,
                Decision::Zero => return Ok(None),
                Decision::Undecidable { enclosure } => {
                    return Err(Error::CannotTellWhichWayAClosedEdgeRuns { enclosure });
                }
                other => panic!("a fifth answer out of the closed-edge rule: {other:?}"),
            };
            Ok(Some(match meet.material_side {
                Decision::Neg => runs_along_stored,
                Decision::Pos => !runs_along_stored,
                other => panic!("a face with no side to its material: {other:?}"),
            }))
        }
    }
}

/// The ball, and the way out of the answer through the flat face, either side
/// of a closed edge between a ball and a plane; `None` for any other pair.
///
/// **At a pole the mesh stores the edge the way the walk takes it.** Every
/// longitude runs through the one point, so the rule that orders every other
/// closed edge has nothing to read there (§38.3), and the mesh leaves the points
/// in the walk's own order. A circle between a ball and a plane is walked from
/// the plane's first basis direction towards its second, which is anticlockwise
/// about that plane's normal — **the plane as the answer stores it**, and the
/// answer stores a flat face pointing out of its own material. So the reference
/// the rule reads is the longitude about that normal, `N × ∇C`, in place of the
/// one about the ball's polar axis: the ball's gradient at a point of the circle
/// is its radius there, the plane's normal is square to the step from the ball's
/// centre to the circle's, and `N × (p − c)` is `N × (p − m)`, the way the walk
/// runs at `p`. That is the direction `SolidBuilder`'s caps are written in too:
/// the lid walks the rim forward and the skin backward, for every cap in
/// `a_cap_whose_rim_runs_through_the_other_pole.rs` (§200.2).
///
/// Which way `N` points is read off the piece, because a piece's surfaces point
/// out of its own material and the answer's out of the answer's. A flat piece's
/// own plane is `N`; a ball's piece has the answer's material on the side of the
/// plane its own material is, so `N` points to the other side. Either is turned
/// round with the piece.
fn the_lid_a_circle_from_a_pole_bounds(meet: &FaceMeet, turned_round: bool) -> Option<Lid> {
    let (ball, outward) = match meet.carried_by {
        [Surface::Plane(flat), ball @ Surface::Sphere(_)] => (ball, flat.normal()),
        [ball @ Surface::Sphere(_), Surface::Plane(flat)] => match meet.material_side {
            Decision::Neg => (ball, flat.normal()),
            Decision::Pos => (ball, -flat.normal()),
            other => panic!("a face with no side to its material: {other:?}"),
        },
        _ => return None,
    };
    Some(Lid {
        ball,
        outward: if turned_round { -outward } else { outward },
    })
}

/// What [`the_lid_a_circle_from_a_pole_bounds`] finds.
struct Lid {
    ball: Surface,
    outward: Vector,
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
/// # Errors
///
/// [`Error::ACrossingThisCannotBeOrderedAlong`] when
/// [`the_crossing_of_these_runs_straight`] has no answer for the pair — a sixth
/// kind of [`Surface`], a pipe against a plane or another pipe whose axes it can
/// prove neither flush nor apart, or a plane against a cone whose apex it cannot
/// place or whose lean it cannot read. **One arc and two arcs are both answers, so
/// neither is the safe reading of a pair nobody has decided.** Guessing *two*
/// is the wrong answer commit `4b62fd0` records the whole cone row getting for
/// three units: an arc where the surfaces cross in a ruling comes out as two
/// arcs, and nothing downstream asks again.
///
/// # Panics
///
/// If the material side is anything but [`Decision::Neg`] or [`Decision::Pos`].
/// [`FaceMeet::material_side`] is written from which way round the arrangement
/// walked the arc and costs no arithmetic, so anything else is a bug here.
fn which_arc_this_is(meet: &FaceMeet, starts_at: usize, from: Point) -> Result<WhichArc> {
    match the_crossing_of_these_runs_straight(&meet.carried_by) {
        Some(true) => return Ok(WhichArc::TheOnlyOne),
        Some(false) => {}
        None => return Err(Error::ACrossingThisCannotBeOrderedAlong),
    }
    let [mine, partner] = meet.carried_by;
    let along_the_crossing = gradient_of(&mine, from).cross(gradient_of(&partner, from));
    Ok(WhichArc::LeavingACorner {
        corner: starts_at,
        towards: match meet.material_side {
            Decision::Neg => along_the_crossing,
            Decision::Pos => along_the_crossing * -1.0,
            other => panic!("a side of a face was walked with no material side: {other:?}"),
        },
    })
}

/// Whether these two surfaces cross in straight lines, so that two corners on
/// one leave one piece and not two.
///
/// The same three cases [`walk`](rcad_geom::walk) has nothing to put between the
/// ends of, spelled the same way — a second spelling of this is how the two come
/// to disagree about how many pieces there are. **Read off the surfaces and
/// never off the chord**: a short enough arc has a chord that rounds to its own
/// tangent, and this question has to stay one about the pair.
///
/// Every pair of the five surfaces there are is named below, and each of them
/// gets a `Some` for the surfaces this kernel can tell apart. The `_` left at
/// the bottom is for a sixth [`Surface`] and nothing else, which is what §40
/// asks for — and because [`Surface`] is `#[non_exhaustive]` and belongs to
/// another crate, a sixth kind lands in it without a compile error here. **So it
/// answers `None`, which is not an answer.** It used to answer `false`, and
/// `false` is not the safe side of anything: it is the two-arc branch of
/// [`which_arc_this_is`], which puts two arcs where a straight run has one. That
/// exact wrong answer is what commit `4b62fd0` records having gone out for the
/// whole cone row for three units, under the same blanket verdict.
///
/// **Three rows are the other `None`, and all three are arithmetic rather than
/// a missing row.** Whether two pipes' axes run along one line is
/// [`two_pipes_run_along_one_line`], whether a plane holds a pipe's axis is
/// [`a_plane_lies_along_a_pipes_axis`], and whether a plane cuts a cone in
/// rulings is [`a_plane_cuts_a_cone_in_rulings`]; each proves its answer or says
/// it cannot. They used to be `w₁.cross(w₂) == Vector::zero()`,
/// `n.dot(w) == 0.0` and a pair of rounded comparisons about a cone's apex and
/// how the plane leans against it — and neither an `f64` cross product, nor an
/// `f64` sum of three products, nor an `f64` degree-six `E` rounding to nothing
/// is a proof that it is nothing. All three predicates' own docs carry the
/// counterexample, and `arrange`'s
/// `two_pipes_a_rounded_cross_product_calls_parallel_are_not`,
/// `a_plane_a_rounded_dot_calls_flush_with_an_axis_is_not` and
/// `a_rounded_apex_and_a_rounded_lean_together_call_a_closed_ellipse_two_rulings`
/// are the demonstrations.
///
/// `tests/a_sixth_kind_of_surface.rs` is what notices the day a sixth kind
/// arrives; `every_pair_of_the_five_kinds_there_are_is_answered` below is what
/// says the `None` covers nothing that exists today.
fn the_crossing_of_these_runs_straight(carried_by: &[Surface; 2]) -> Option<bool> {
    match carried_by {
        [Surface::Plane(_), Surface::Plane(_)] => Some(true),
        [Surface::Plane(flat), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Plane(flat)] => {
            match a_plane_lies_along_a_pipes_axis(flat, pipe) {
                Decision::Zero => Some(true),
                Decision::Neg | Decision::Pos => Some(false),
                Decision::Undecidable { .. } | _ => None,
            }
        }
        [Surface::Cylinder(one), Surface::Cylinder(other)] => {
            match two_pipes_run_along_one_line(one, other) {
                Decision::Zero => Some(true),
                Decision::Pos => Some(false),
                Decision::Neg => unreachable!("a squared length is never below zero"),
                Decision::Undecidable { .. } | _ => None,
            }
        }
        // A plane and a cone cross in rulings when the plane goes through the
        // apex and leans no shallower than the cone's own side: one ruling the
        // plane rests along, two where it is steeper. Both are straight, and
        // both are what `where_a_plane_and_a_cone_cross` hands back for those
        // two signs.
        //
        // This used to answer the whole cone row `false` and say a cone never
        // reached here running straight, because the two shapes above came back
        // refused by name. They are named now, so a `FaceMeet` carrying such a
        // pair is built by the ordinary path and the old claim would put two
        // arcs where a ruling has one.
        //
        // Both of the signs it turns on are proved rather than rounded, and
        // `None` when neither can be — the third row of the same trap the two
        // above were in, and its own doc carries the pair of numbers.
        [Surface::Plane(flat), Surface::Cone(cone)]
        | [Surface::Cone(cone), Surface::Plane(flat)] => a_plane_cuts_a_cone_in_rulings(flat, cone),
        // A cone against a ball, a pipe or another cone; the ring row below
        // covers the fourth. The three do not rest on the same thing.
        //
        // **A ball is bounded and a straight line is not**, so a cone against a
        // ball holds no line for the reason the ball row below gives.
        //
        // **A cone coaxial with a pipe crosses it in exactly two circles**, and a
        // circle holds no line. `slope·|h| = ρ` has two roots and never one and
        // never none, because a cone is both its nappes and a pipe's radius is
        // above zero (§50.11) — so `false` is about the geometry **for a
        // coaxial pair**, and about what `intersect` will hand over for every
        // other one. This arm is `Cone × Cylinder` whatever the two axes do,
        // and a cone and a pipe that are not coaxial genuinely can share a
        // line: a pipe of radius 2 about `(1, 0, 1)` through `(0, 2, 0)` holds
        // the ruling `(t, 0, t)` of `Cone::new(Axis::z(), 1.0)`, since
        // `‖(p − o) × d‖² = 8 = 4‖d‖²`. What keeps the arm right is that no
        // such pair reaches here: `intersect` refuses a cone against a pipe
        // that is not coaxial with it by name.
        // `a_cone_coaxial_with_a_pipe_crosses_it_in_circles_which_hold_no_line`
        // and `a_pipe_alongside_one_of_a_cones_rulings_holds_it` are the two
        // demonstrations.
        //
        // **`false` rests on `intersect` and not on the geometry** for cone
        // against cone as well: two cones that share an apex meet along
        // rulings, which are lines. No such pair reaches here — `intersect`
        // answers `ACrossingBetweenAConeAndAnotherCurvedSurface` for every
        // cone-and-cone that is not the same cone.
        //
        // So **two** of the three rows here are right only for as long as
        // those refusals stand, and both are rows to revisit the day one
        // lifts: cone against cone, and cone against a pipe it is not coaxial
        // with. Only the coaxial pipe and the ball answer out of the shapes
        // themselves.
        [
            Surface::Cone(_),
            Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_),
        ]
        | [Surface::Sphere(_) | Surface::Cylinder(_), Surface::Cone(_)] => Some(false),
        // **A ball's skin and a ring's skin are bounded, and a straight line is
        // not, so no line lies on either** — which settles every remaining pair
        // at once and without a case analysis. Whatever the partner is, the
        // crossing lies on the ball or the ring, so it holds no line, so two
        // corners on it leave two arcs and not one. `Torus::new` refuses a tube
        // wider than its ring, so every ring here is a ring and not a pinched
        // or self-crossing one, and that is what makes "bounded" true of the
        // *stored* shape rather than only of the nice case.
        //
        // A plane and a ball is the pair that arrives: it reached
        // `which_arc_this_is` twelve times in one run of this crate's suite,
        // and was answered by the `_` arm that used to sit below here, unnamed
        // and uncounted. `a_crossing_with_a_ball_or_a_ring_on_it_holds_no_line`
        // is the demonstration rather than the paragraph.
        [
            Surface::Sphere(_),
            Surface::Plane(_) | Surface::Sphere(_) | Surface::Cylinder(_),
        ]
        | [Surface::Plane(_) | Surface::Cylinder(_), Surface::Sphere(_)] => Some(false),
        [
            Surface::Torus(_),
            Surface::Plane(_)
            | Surface::Sphere(_)
            | Surface::Cylinder(_)
            | Surface::Cone(_)
            | Surface::Torus(_),
        ]
        | [
            Surface::Plane(_) | Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_),
            Surface::Torus(_),
        ] => Some(false),
        _ => None,
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
/// The loop stops once the total is past twice the size of the arithmetic. That
/// is past any reading `facing · p` can have by Cauchy–Schwarz, so by then both
/// ends read in front and the loop has already returned — which is why the one
/// way out of here is `facing` having no length to point along. That is a chord
/// lying along the crossing's own direction, which for a conic means an arc so
/// short its chord rounds onto its own tangent
/// (`an_arc_whose_chord_rounds_onto_its_own_tangent_is_refused_by_name`).
///
/// # Room for the corners the next fuse will solve
///
/// A fuse that takes this answer back in does not read the stored corners: it
/// re-solves each one from three surfaces into a box, and that box sits up to
/// [`what_writing_the_corner_down_cost`] from the stored point. A picker the
/// stored ends only just clear is one those boxes straddle, and the next fuse
/// cannot tell the arcs apart (§223.2). So the picker is first lowered until the
/// ends **grown by that room** read on or in front.
///
/// **Lowering must not take the plane past the curve's centre.** For a closed
/// crossing the arc is named by its ends in front and its centre on or behind
/// ([`a_picker_that_tells_the_two_arcs_apart`]), and an arc within a few
/// `1e-14` of a semicircle has its centre that close behind the chord. There
/// the room is not granted: the picker is the one the stored ends alone earn,
/// exactly as before the room existed, and the next fuse may refuse it as it
/// always did — `a_near_semicircle_keeps_its_centre_behind_its_picker` measures
/// the band. Two exact ends on the chord's own plane need neither, and keep it.
fn the_half_space_both_ends_are_on(
    facing: Vector,
    ends: [Point; 2],
    centred: Option<(&Plane, &WhereTheCurveIsCentred)>,
) -> Option<Picker> {
    let readings = ends.map(|end| facing.dot(end - Point::origin()));
    let size = facing.length()
        * ends
            .iter()
            .map(|end| (*end - Point::origin()).length())
            .fold(0.0, f64::max);
    let from_here = readings[0].min(readings[1]);
    let exactly = ends.map(|end| PredicatePoint::from(end.coordinates()));
    let through_the_chord = HalfSpace::new(facing, from_here).ok()?;
    if exactly
        .iter()
        .all(|end| through_the_chord.side_of_point(*end) == Decision::Zero)
    {
        return Some(Picker::from(through_the_chord));
    }
    let with_room =
        exactly.map(|end| grown_by(end, what_writing_the_corner_down_cost(end.enclosure())));
    let with_room_and_centre_behind = |half: &HalfSpace| match centred {
        Some((flat, centre)) => {
            a_picker_that_tells_the_two_arcs_apart(half, flat, centre, with_room)
        }
        None => with_room
            .iter()
            .all(|end| matches!(half.side_of_point(*end), Decision::Pos | Decision::Zero)),
    };
    let stored_ends_alone = |half: &HalfSpace| {
        exactly
            .iter()
            .all(|end| half.side_of_point(*end) != Decision::Neg)
    };
    lowered_until(facing, from_here, size, with_room_and_centre_behind)
        .or_else(|| lowered_until(facing, from_here, size, stored_ends_alone))
        .map(Picker::from)
}

/// The half-space facing `facing`, its offset brought down from `from_here` by
/// a step that starts at one rounding of `size` and doubles, the first time
/// `holds` says yes — or `None` once it has come down past `2·size`.
fn lowered_until(
    facing: Vector,
    from_here: f64,
    size: f64,
    holds: impl Fn(&HalfSpace) -> bool,
) -> Option<HalfSpace> {
    let mut down_by = 0.0;
    let mut step = (size * f64::EPSILON).max(f64::MIN_POSITIVE);
    while down_by <= 2.0 * size {
        let half = HalfSpace::new(facing, from_here - down_by).ok()?;
        if holds(&half) {
            return Some(half);
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
fn which_curve_it_is(from: usize, to: usize, carried_by: [Surface; 2]) -> Option<[[u64; 9]; 2]> {
    if from != to {
        return None;
    }
    let mut named = carried_by.map(|surface| name_of_the_surface_either_way_round(&surface));
    named.sort_unstable();
    Some(named)
}

/// Which of the two arcs between its corners a side is of a closed crossing
/// between a plane and a surface that curves right round — a ball's circle, or
/// a cone's rim circle and the ellipse a steeper plane leaves — or `None` for
/// any other side.
///
/// **Both arcs run between the same two corners and along the same two
/// surfaces**, so nothing [`which_curve_it_is`] reads tells them apart. Most
/// arcs of a closed crossing never meet their other half in one answer —
/// whatever cut the crossing twice kept one arc and threw the other away — but
/// two cases keep both. A circle cut only to give it a direction, because it
/// started at the ball's pole: the dome on one side and the hole in the floor
/// on the other each walk the whole of it. And a half ball's own rim, the
/// circle `x = 0` walked as two edges from pole to pole: cut again by
/// `x ≤ 0.5`, the skin and the flat half disk are both kept whole, and named
/// by their corners the two edges were one edge with four faces along it.
///
/// **What tells them apart is a sign, and no number is built.** `∇round ×
/// ∇plane` runs one way round the circle all the way along it, so there is
/// exactly one arc that leaves the lower-numbered corner along it. A side's
/// material side says whether it leaves its first corner along `∇mine ×
/// ∇partner` ([`which_arc_this_is`]'s rule); the plane's face has the two the
/// other way round, and a side walked from the higher-numbered corner leaves
/// the lower one against that direction when it arrives. Both faces along an
/// arc, and both orders of the arguments, come to the same answer, because the
/// corner numbers are the answer's and both faces share them.
///
/// # A cone's rim circle is the same shape of thing, and its seam is not
///
/// Nothing above is about a ball. The sign rule needs the crossing to close and
/// `∇round × ∇plane` never to vanish along it, and
/// [`Cone::is_cut_in_a_closed_curve_by`](rcad_geom::Cone::is_cut_in_a_closed_curve_by)
/// is that property decided exactly
/// for a cone — the circle a cap plane leaves, and the ellipse any leaning
/// plane leaves, both of them on one nappe. A hyperbola and a parabola never
/// close and stay [`None`], because two corners on an endless bend really do
/// leave one arc.
///
/// **The seam guard below is the ball's and stays the ball's.**
/// [`MeetPoint::APoleOfAClosedSurface`] has one maker in this crate,
/// `Arrangement::the_whole_surface`, and that refuses every surface but a ball
/// — so no cone-and-plane side can carry one, and the guard is a no-op for the
/// rim rather than a rule that had to be carried over. A cone's own singular
/// point, its apex, is not one of these poles and needs no guard here either: a
/// crossing this function names has the apex **off** the plane, which is half
/// of what `Cone::is_cut_in_a_closed_curve_by` asks, so the curve does not
/// pass through it. `tests/which_arc_of_a_cones_rim_a_side_is.rs` fuses a
/// pointed cone — the shape that has an apex on its own face — and gets a sound
/// answer.
///
/// # Panics
///
/// If the material side is anything but [`Decision::Neg`] or [`Decision::Pos`],
/// as [`which_arc_this_is`] does and for the same reason.
fn which_arc_of_a_circle_this_is(meet: &FaceMeet, from: usize, to: usize) -> Option<bool> {
    let round_one = which_of_them_a_closed_crossing_runs_round(&meet.carried_by)?;
    // A whole ball's seam is walked down and back up by the one face, with a
    // material side that names the half of the meridian it is and not a side
    // (`Arrangement::the_whole_surface`), so the rule below would read its two
    // walks as two arcs. It is one edge, and it has only the one arc to be.
    let a_seam = meet
        .ends
        .iter()
        .any(|end| matches!(end, MeetPoint::APoleOfAClosedSurface { .. }));
    if from == to || a_seam {
        return None;
    }
    let walked_from_the_far_corner = from > to;
    let along_mine_then_partner = match meet.material_side {
        Decision::Neg => true,
        Decision::Pos => false,
        other => panic!("a side of a face was walked with no material side: {other:?}"),
    };
    let mine_is_the_round_one = round_one == 0;
    let along_the_round_one_then_the_plane = along_mine_then_partner == mine_is_the_round_one;
    Some(along_the_round_one_then_the_plane != walked_from_the_far_corner)
}

/// Which of two surfaces a closed crossing runs round — `0` or `1` — or `None`
/// when this pair has no closed crossing to name.
///
/// A ball and a plane cut a circle whenever they cut anything at all. A cone and
/// a plane cut seven different shapes and only two of them close, which is
/// [`Cone::is_cut_in_a_closed_curve_by`](rcad_geom::Cone::is_cut_in_a_closed_curve_by)'s
/// question — one predicate in `rcad-geom`, asked here and by `select.rs`'s
/// `in_the_bulge_of` and `holds.rs`'s `is_a_family_this_follows`, so the three
/// cannot drift apart (§23.8, §208.6).
///
/// **A pipe and a plane cut an ellipse that closes too, and it is named here.**
/// It used to be left out because `select.rs`'s bulge count did not reach a
/// pipe, so no side of one got this far and the row had no consumer (§13).
/// Counting the pipe there gave it one, and this row is what pays for it:
/// without it, two arcs of one ellipse between the same corners are named as one
/// edge and four faces meet along it. With the pipe counted in `select.rs` and
/// not here,
/// `two_tools_meeting_on_a_plane_that_cuts_a_round_wall_are_answered` and the
/// catalogue's coincident-bar row both came back
/// `ResultIsNotManifold { count: 4 }` — the cost `in_the_bulge_of`'s own doc
/// warned of, measured rather than quoted, and it is the whole of it.
///
/// **A pipe's crossing never loses its direction where this is asked.** The
/// sign rule needs `∇round × ∇plane` to vanish nowhere along the curve, and a
/// pipe's gradient is square to its axis while a closed crossing has `n·a ≠ 0`
/// — so the two can never point the same way. That is stronger than the cone's
/// version, which has an apex to keep off the plane.
fn which_of_them_a_closed_crossing_runs_round(carried_by: &[Surface; 2]) -> Option<usize> {
    let closes_round = |round: &Surface, flat: &Surface| {
        let flat = flat.as_plane()?;
        if matches!(round, Surface::Sphere(_)) {
            return Some(true);
        }
        if let Surface::Cone(cone) = round {
            return cone.is_cut_in_a_closed_curve_by(flat);
        }
        if let Surface::Cylinder(pipe) = round {
            return pipe.is_cut_in_a_closed_curve_by(flat);
        }
        Some(false)
    };
    if closes_round(&carried_by[0], &carried_by[1]) == Some(true) {
        Some(0)
    } else if closes_round(&carried_by[1], &carried_by[0]) == Some(true) {
        Some(1)
    } else {
        None
    }
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
    face: FaceKey,
    model: &Model,
    corners: &mut Corners,
    edges: &mut Edges,
) -> Result<Vec<Side>> {
    let mut sides = Vec::new();
    for meet in wire {
        let from = corners.of(model, &meet.ends[0]);
        let to = corners.of(model, &meet.ends[1]);
        sides.push(edges.of(meet, from, to, face, turned_round)?);
    }
    if turned_round {
        sides.reverse();
        for side in &mut sides {
            side.forward = !side.forward;
        }
    }
    Ok(sides)
}

#[cfg(test)]
mod tests {
    use rcad_geom::{
        Axis, Cone, Cylinder, EdgeRun, Isometry, Orientation, Plane, Scale, Sphere, Surface,
        SurfaceMeet, Torus,
    };
    use rcad_predicates::{Policy, ProvenanceLog};

    use crate::{DraftStep, Invalid, Rule, SolidBuilder};

    use super::super::Scratch;
    use super::super::select::choose;
    use super::{
        Assembly, Corner, Corners, Edge, Error, Face, Lump, MeetPoint, Model, Point,
        PredicatePoint, Side, Vector, WhichArc, WireStep, Written, assemble,
        names_of_the_three_surfaces, the_crossing_of_these_runs_straight,
        the_half_space_both_ends_are_on,
    };

    /// A corner that settles on a lower rounding leaves the coordinate it came
    /// from, and nothing may be joined to it there afterwards.
    ///
    /// Once the lower rounding wins, no corner sits on the higher one. A later
    /// corner with a **different** name that does land there is a different
    /// corner, and joining the two puts one vertex where the answer needs two —
    /// §49.3's latent defect, which nothing else in the suite reaches.
    #[test]
    fn a_corner_that_moved_does_not_swallow_a_later_one_where_it_used_to_be() {
        let model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let flat = |normal: Vector| {
            Surface::Plane(
                Plane::from_point_and_normal(Point::origin(), normal)
                    .expect("a normal with length"),
            )
        };
        let across = flat(Vector::new(1.0, 0.0, 0.0));
        let along = flat(Vector::new(0.0, 1.0, 0.0));
        let upright = flat(Vector::new(0.0, 0.0, 1.0));
        let starts = |carried_by: [Surface; 2], at: [f64; 3]| MeetPoint::WhereAClosedCurveStarts {
            carried_by,
            point: PredicatePoint::from(at),
        };

        let mut corners = Corners::default();
        let moved = corners.of(&model, &starts([across, along], [1.0, 0.0, 0.0]));
        let again = corners.of(&model, &starts([across, along], [0.0, 0.0, 0.0]));
        assert_eq!(again, moved, "one name is one corner");
        assert_eq!(
            corners.made[moved].point,
            Point::origin(),
            "the lower of the two roundings is the one kept"
        );

        let other = corners.of(&model, &starts([across, upright], [1.0, 0.0, 0.0]));
        assert_ne!(
            other, moved,
            "a corner was joined to one that had already moved off this coordinate"
        );
        assert_eq!(
            corners.made.len(),
            2,
            "two names on one place is two corners"
        );
    }

    /// Why every pair with a ball or a ring on it is answered `false`.
    ///
    /// Three points on the crossing, each checked onto **both** surfaces with
    /// exact arithmetic — every number here is a whole one or a 3-4-5 side, so
    /// the equalities are the real ones and not near ones — and then shown not
    /// to be collinear. A crossing three of whose points miss a line is not a
    /// line, so two corners on it leave two arcs, which is what `false` says.
    ///
    /// This replaces a `_` arm that answered the same `false` without being
    /// counted or named. A plane and a ball is the pair that actually arrives.
    #[test]
    fn a_crossing_with_a_ball_or_a_ring_on_it_holds_no_line() {
        let up = Axis::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
            .expect("a direction with length");
        let ball = Sphere::new(Point::new(0.0, 0.0, 0.0), 5.0).expect("a ball of radius five");
        let flat =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
                .expect("the plane through the ball's middle");
        let other_ball =
            Sphere::new(Point::new(8.0, 0.0, 0.0), 5.0).expect("a ball eight away, of radius five");
        let pipe = Cylinder::new(up, 3.0).expect("a pipe of radius three");
        let ring = Torus::new(up, 2.0, 1.0).expect("a ring of radius two round a tube of one");

        let on_the_ball = |p: Point| (p - Point::new(0.0, 0.0, 0.0)).length_squared() == 25.0;
        let on_the_other_ball = |p: Point| (p - Point::new(8.0, 0.0, 0.0)).length_squared() == 25.0;
        let on_the_flat = |p: Point| p.z() == 0.0;
        let on_the_pipe = |p: Point| p.x() * p.x() + p.y() * p.y() == 9.0;
        let on_the_ring = |p: Point| {
            let from_the_axis = (p.x() * p.x() + p.y() * p.y()).sqrt() - 2.0;
            from_the_axis * from_the_axis + p.z() * p.z() == 1.0
        };

        let three_points_off_a_line =
            |pair: [Surface; 2],
             points: [Point; 3],
             on_one: &dyn Fn(Point) -> bool,
             on_other: &dyn Fn(Point) -> bool| {
                for point in points {
                    assert!(on_one(point), "{point:?} is not on the first surface");
                    assert!(on_other(point), "{point:?} is not on the second surface");
                }
                let [first, second, third] = points;
                assert_ne!(
                    (second - first).cross(third - first),
                    Vector::zero(),
                    "three points of this crossing lie on a line"
                );
                for [one, other] in [pair, [pair[1], pair[0]]] {
                    assert_eq!(
                        the_crossing_of_these_runs_straight(&[one, other]),
                        Some(false),
                        "a crossing with three points off a line was called straight"
                    );
                }
            };

        three_points_off_a_line(
            [Surface::Plane(flat), Surface::Sphere(ball)],
            [
                Point::new(5.0, 0.0, 0.0),
                Point::new(0.0, 5.0, 0.0),
                Point::new(-5.0, 0.0, 0.0),
            ],
            &on_the_flat,
            &on_the_ball,
        );
        three_points_off_a_line(
            [Surface::Sphere(ball), Surface::Sphere(other_ball)],
            [
                Point::new(4.0, 3.0, 0.0),
                Point::new(4.0, 0.0, 3.0),
                Point::new(4.0, -3.0, 0.0),
            ],
            &on_the_ball,
            &on_the_other_ball,
        );
        three_points_off_a_line(
            [Surface::Sphere(ball), Surface::Cylinder(pipe)],
            [
                Point::new(3.0, 0.0, 4.0),
                Point::new(0.0, 3.0, 4.0),
                Point::new(-3.0, 0.0, 4.0),
            ],
            &on_the_ball,
            &on_the_pipe,
        );
        three_points_off_a_line(
            [Surface::Torus(ring), Surface::Plane(flat)],
            [
                Point::new(3.0, 0.0, 0.0),
                Point::new(0.0, 3.0, 0.0),
                Point::new(-3.0, 0.0, 0.0),
            ],
            &on_the_ring,
            &on_the_flat,
        );
    }

    /// A plane and a cone run straight exactly where the plane goes through the
    /// apex and leans no shallower than the cone's own side.
    ///
    /// The four rows are the four this pair has, and each one is put to
    /// `intersect` as well so that the two never disagree about what the pair
    /// is. Two of them are straight lines a caller has to leave **one** arc
    /// between and not two — the ruling a tangent plane rests along, and the two
    /// rulings a steeper plane cuts. The other two are not straight: a plane
    /// cutting across leaves an ellipse, and a plane through the apex shallower
    /// than the cone shares the apex with it and nothing else.
    ///
    /// This row used to be a blanket `false` resting on the claim that no run
    /// with a cone on it ever arrived here straight. That was true only while
    /// `intersect` refused both apex shapes by name, and it does not any more.
    #[test]
    fn a_plane_and_a_cone_run_straight_at_the_apex_and_no_shallower() {
        let cone = Cone::new(Axis::z(), 0.5).expect("a cone leaning a half");
        let flat = |normal: Vector, through: Point| {
            Plane::from_point_and_normal(through, normal).expect("a normal with length")
        };

        // A tangent plane holding the ruling `(t/2, 0, t)`, a plane down the
        // axis that cuts two rulings, a plane square to the axis clear of the
        // apex, and the same plane slid onto the apex.
        let touching = flat(Vector::new(-1.0, 0.0, 0.5), Point::origin());
        let down_the_axis = flat(Vector::new(0.0, 1.0, 0.0), Point::origin());
        let across = flat(Vector::new(0.0, 0.0, 1.0), Point::new(0.0, 0.0, 4.0));
        let on_the_apex = flat(Vector::new(0.0, 0.0, 1.0), Point::origin());

        for (named, plane, runs_straight) in [
            ("a plane resting along one ruling", touching, true),
            (
                "a plane down the axis cutting two rulings",
                down_the_axis,
                true,
            ),
            ("a plane cutting across the cone", across, false),
            (
                "a plane through the apex square to the axis",
                on_the_apex,
                false,
            ),
        ] {
            for pair in [
                [Surface::Plane(plane), Surface::Cone(cone)],
                [Surface::Cone(cone), Surface::Plane(plane)],
            ] {
                assert_eq!(
                    the_crossing_of_these_runs_straight(&pair),
                    Some(runs_straight),
                    "{named} was read the wrong way round {pair:?}"
                );
            }

            // A second spelling of this question is how the two come to
            // disagree about how many pieces a pair has, so `intersect` is asked
            // the same four rows and has to hand back straight lines for exactly
            // the two above: one [`EdgeRun::Straight`] for the ruling a tangent
            // plane rests along, and two [`EdgeRun::PartOfWhereItCrosses`] with
            // a picker each for the pair a steeper plane cuts.
            let met = rcad_geom::intersect(
                &Surface::Plane(plane),
                &Surface::Cone(cone),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{named} is a shape `intersect` names: {why:?}"));
            let intersect_says_lines = matches!(&met, SurfaceMeet::Curves(runs) if runs
            .iter()
            .all(|run| matches!(
                run.run(),
                EdgeRun::Straight | EdgeRun::PartOfWhereItCrosses { .. }
            )));
            assert_eq!(
                intersect_says_lines, runs_straight,
                "{named} came back from `intersect` as {met:?}"
            );
        }
    }

    /// Why a cone coaxial with a pipe is answered `false` about the geometry
    /// rather than about `intersect`'s refusal.
    ///
    /// The two meet where `slope·|h|` is the pipe's radius, and that is two
    /// circles — never one and never none, because a cone is both its nappes so
    /// `h` takes both signs, and a pipe's radius is above zero so neither root
    /// is the apex (§50.11). Three points of one circle, checked onto both
    /// surfaces with whole numbers and then shown to miss a line: a crossing
    /// three of whose points miss a line is not a line.
    #[test]
    fn a_cone_coaxial_with_a_pipe_crosses_it_in_circles_which_hold_no_line() {
        let cone = Cone::new(Axis::z(), 1.0).expect("a cone leaning one");
        let pipe = Cylinder::new(Axis::z(), 3.0).expect("a pipe of radius three");

        let circle = [
            Point::new(3.0, 0.0, 3.0),
            Point::new(0.0, 3.0, 3.0),
            Point::new(-3.0, 0.0, 3.0),
        ];
        for point in circle {
            let from_the_axis = point.x() * point.x() + point.y() * point.y();
            assert_eq!(from_the_axis, 9.0, "{point:?} is not on the pipe");
            assert_eq!(
                from_the_axis,
                point.z() * point.z(),
                "{point:?} is not on the cone"
            );
        }
        let [first, second, third] = circle;
        assert_ne!(
            (second - first).cross(third - first),
            Vector::zero(),
            "three points of this crossing lie on a line"
        );

        for pair in [
            [Surface::Cone(cone), Surface::Cylinder(pipe)],
            [Surface::Cylinder(pipe), Surface::Cone(cone)],
        ] {
            assert_eq!(
                the_crossing_of_these_runs_straight(&pair),
                Some(false),
                "a crossing that is two circles was called straight {pair:?}"
            );
        }
    }

    /// A pipe that is not coaxial with a cone can hold one of the cone's
    /// rulings, which is why `intersect` still refuses every cone and pipe but
    /// the coaxial one.
    ///
    /// The ruling `(t, 0, t)` of a cone leaning one lies on a pipe of radius two
    /// whose axis runs the same way two units off it: every point of the ruling
    /// is exactly two from that axis, by whole numbers. `false` above would be
    /// the wrong answer for this pair, and the refusal is the only thing keeping
    /// it away.
    #[test]
    fn a_pipe_alongside_one_of_a_cones_rulings_holds_it() {
        let cone = Cone::new(Axis::z(), 1.0).expect("a cone leaning one");
        let along_the_ruling =
            Axis::new(Point::new(0.0, 2.0, 0.0), Vector::new(1.0, 0.0, 1.0)).expect("a direction");
        let alongside = Cylinder::new(along_the_ruling, 2.0).expect("a pipe of radius two");

        let ruling = [
            Point::new(1.0, 0.0, 1.0),
            Point::new(2.0, 0.0, 2.0),
            Point::new(3.0, 0.0, 3.0),
        ];
        for point in ruling {
            assert_eq!(
                point.x() * point.x() + point.y() * point.y(),
                point.z() * point.z(),
                "{point:?} is not on the cone"
            );
            let across = (point - along_the_ruling.origin()).cross(along_the_ruling.direction());
            assert_eq!(
                across.length_squared(),
                4.0 * along_the_ruling.direction().length_squared(),
                "{point:?} is not two from the pipe's axis"
            );
        }
        let [first, second, third] = ruling;
        assert_eq!(
            (second - first).cross(third - first),
            Vector::zero(),
            "these three points were meant to be a straight line"
        );

        assert!(
            matches!(
                rcad_geom::intersect(
                    &Surface::Cone(cone),
                    &Surface::Cylinder(alongside),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                ),
                Err(rcad_geom::Error::ACrossingBetweenAConeAndAnotherCurvedSurface)
            ),
            "a cone and a pipe sharing a ruling is named, and `the_crossing_of_these_runs_straight` answers it `false`"
        );
    }

    /// **This door reads two pipes' axes with the same proof the other one
    /// does**, so a pair that is not provably side by side is never read as
    /// running straight here either.
    ///
    /// The same two directions `arrange`'s
    /// `two_pipes_a_rounded_cross_product_calls_parallel_are_not` builds: `(a, 0,
    /// c)` and `(b, 0, c)`, one ulp apart, whose `f64` cross product is exactly
    /// nothing while the axes are not along one line. `intersect` names their
    /// crossing one closed loop, and this row used to call it straight — which is
    /// [`which_arc_this_is`]'s one-arc branch, so the two corners of a closed
    /// loop would have left one arc where they leave two.
    ///
    /// Two pipes whose axes really are along one line still answer `Some(true)`,
    /// and the second row is what says so.
    #[test]
    fn two_pipes_are_read_straight_only_where_their_axes_are_proved_along_one_line() {
        let (a, b, c) = (
            0.393_507_725_793_881_44_f64,
            0.393_507_725_793_881_4_f64,
            0.919_321_309_304_058_f64,
        );
        let pipe = |origin: Point, along: Vector| {
            Surface::Cylinder(
                Cylinder::new(Axis::new(origin, along).expect("an axis"), 1.0).expect("a pipe"),
            )
        };
        let leaning = [
            pipe(Point::origin(), Vector::new(a, 0.0, c)),
            pipe(Point::new(0.0, 1.0, 0.0), Vector::new(b, 0.0, c)),
        ];
        let side_by_side = [
            pipe(Point::origin(), Vector::new(0.0, 0.0, 1.0)),
            pipe(Point::new(1.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0)),
        ];

        let [Surface::Cylinder(one), Surface::Cylinder(other)] = leaning else {
            panic!("both of them are pipes");
        };
        assert_eq!(
            one.axis().direction().cross(other.axis().direction()),
            Vector::zero(),
            "the `f64` cross product rounds to nothing, and that is what used to \
             decide this row"
        );

        for (named, pair, runs_straight) in [
            (
                "two pipes a rounded cross product calls parallel",
                leaning,
                false,
            ),
            (
                "two pipes whose axes really are along one line",
                side_by_side,
                true,
            ),
        ] {
            for turned in [pair, [pair[1], pair[0]]] {
                assert_eq!(
                    the_crossing_of_these_runs_straight(&turned),
                    Some(runs_straight),
                    "{named} was read the wrong way round {turned:?}"
                );
            }

            let met = rcad_geom::intersect(
                &pair[0],
                &pair[1],
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{named} is a shape `intersect` names: {why:?}"));
            let intersect_says_lines = matches!(&met, SurfaceMeet::Curves(runs) if runs
            .iter()
            .all(|run| matches!(
                run.run(),
                EdgeRun::Straight | EdgeRun::PartOfWhereItCrosses { .. }
            )));
            assert_eq!(
                intersect_says_lines, runs_straight,
                "{named} came back from `intersect` as {met:?}"
            );
        }
    }

    /// **This door reads a plane against a cone with the same two proofs the
    /// other one does**, so a closed ellipse is never read as a pair of rulings
    /// here either.
    ///
    /// The same plane and cone `arrange`'s
    /// `a_rounded_apex_and_a_rounded_lean_together_call_a_closed_ellipse_two_rulings`
    /// builds: the cone's apex one ulp off a plane through the origin, so the
    /// `f64` distance to it is exactly nothing, and the plane a rounding off
    /// tangent to the cone, so `f64`'s own `E` is at or below nothing. Both
    /// rounded tests say two rulings and `intersect` names one closed ellipse.
    ///
    /// This row used to answer `Some(true)` for it, which is
    /// [`which_arc_this_is`]'s one-arc branch — one arc where two corners on a
    /// closed ellipse leave two.
    #[test]
    fn a_plane_and_a_cone_are_read_straight_only_where_both_signs_are_proved() {
        let normal = Vector::new(
            -0.363_397_689_122_298_75,
            -0.399_628_971_525_338,
            0.841_569_251_255_161_7,
        );
        let along = Vector::new(
            -0.426_496_054_832_981_4,
            0.483_888_527_748_997_2,
            0.764_168_180_392_778_8,
        );
        let apex = Point::new(
            -0.162_754_196_529_426_2,
            0.361_000_414_878_634_9,
            0.101_146_430_287_779_8,
        );
        let slope = 0.759_265_215_818_364_9;
        let flat = Plane::from_point_and_normal(Point::origin(), normal).expect("a plane");
        let cone = Cone::new(Axis::new(apex, along).expect("an axis"), slope).expect("a cone");
        assert_eq!(
            flat.signed_distance_to(cone.apex()),
            0.0,
            "the `f64` distance to the apex rounds to nothing, and that is half of \
             what used to decide this row"
        );
        let up = normal.dot(along);
        assert!(
            up * up - slope * slope * normal.cross(along).length_squared() <= 0.0,
            "and `f64`'s own `E` is at or below nothing, which is the other half"
        );

        for turned in [
            [Surface::Plane(flat), Surface::Cone(cone)],
            [Surface::Cone(cone), Surface::Plane(flat)],
        ] {
            assert_eq!(
                the_crossing_of_these_runs_straight(&turned),
                Some(false),
                "a closed ellipse read the wrong way round {turned:?}"
            );
        }

        let met = rcad_geom::intersect(
            &Surface::Plane(flat),
            &Surface::Cone(cone),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a plane cutting across a cone is a shape `intersect` names");
        assert!(
            matches!(&met, SurfaceMeet::Curves(runs) if runs.len() == 1
                && runs[0].run() == &EdgeRun::RightRoundWhereItsFacesCross),
            "both doors ask one question of one pair of surfaces, and `intersect` \
             says {met:?}"
        );
    }

    /// No pair of the five surface kinds there are falls into the `_`, so what
    /// the `_` now refuses is nothing that exists.
    ///
    /// That is the half of this claim a test in this tree can reach. The other
    /// half — that a pair with no row **refuses** instead of being read as
    /// bending — is a change of return type, and it is the type that carries it:
    /// this test does not compile against a
    /// [`the_crossing_of_these_runs_straight`] that hands back a bare `bool`,
    /// because a bare `bool` has already chosen an answer for the pair it has
    /// never seen. Which answer it chose was `false`, and `false` is the
    /// two-arc branch of `which_arc_this_is`: two arcs where a ruling has one.
    /// Commit `4b62fd0` is that answer having gone out for three units over the
    /// whole cone row.
    ///
    /// Forging a sixth [`Surface`] to drive the refusal itself cannot be done
    /// from this crate — [`Surface`] is `#[non_exhaustive]` and lives in
    /// `rcad-geom` — and `tests/a_sixth_kind_of_surface.rs` says why at length.
    #[test]
    fn every_pair_of_the_five_kinds_there_are_is_answered() {
        let up = Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("a direction with length");
        let five = [
            Surface::Plane(
                Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                    .expect("a normal with length"),
            ),
            Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a ball of radius five")),
            Surface::Cylinder(Cylinder::new(up, 3.0).expect("a pipe of radius three")),
            Surface::Cone(Cone::new(up, 0.5).expect("a cone leaning a half")),
            Surface::Torus(Torus::new(up, 2.0, 1.0).expect("a ring round a tube of one")),
        ];

        for one in five {
            for other in five {
                assert!(
                    the_crossing_of_these_runs_straight(&[one, other]).is_some(),
                    "{one:?} against {other:?} has no row here, so a fuse over the \
                     two would be refused rather than built"
                );
            }
        }
    }

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
        assert!(the_half_space_both_ends_are_on(facing, [from, to], None).is_none());
    }

    /// Two solids flush against each other name the plane they share opposite
    /// ways round, and a corner on it must still be one corner.
    ///
    /// §48, W5. The corner is where `x = 3`, `y = 0` and `z = 0` meet; one face
    /// arrives with `x = 3` facing towards `+x` and the other with it facing
    /// towards `−x`, which is the same plane with all four coefficients negated.
    /// Nothing about the corner has moved. This holds no coordinates at all —
    /// the two faces solve for it separately and Cramer's rule rounds the two
    /// answers a bit or two apart, which is exactly why the join is by name.
    #[test]
    fn one_of_three_planes_facing_the_other_way_names_the_same_corner() {
        let at = |normal, through| {
            Plane::from_point_and_normal(through, normal).expect("a normal with length")
        };
        let wall = at(Vector::new(1.0, 0.0, 0.0), Point::new(3.0, 0.0, 0.0));
        let floor = at(Vector::new(0.0, 0.0, 1.0), Point::new(0.0, 0.0, 0.0));
        let side = at(Vector::new(0.0, 1.0, 0.0), Point::new(0.0, 0.0, 0.0));

        let one_way = [wall, floor, side].map(Surface::Plane);
        let other_way = [wall.reversed(), floor, side].map(Surface::Plane);

        assert_ne!(
            wall.coefficients(),
            wall.reversed().coefficients(),
            "a plane turned round is different numbers, which is the whole difficulty"
        );
        assert_eq!(
            names_of_the_three_surfaces(&one_way),
            names_of_the_three_surfaces(&other_way)
        );
    }

    /// A triple with a ball in it keeps each face's spelling, and that is on
    /// purpose.
    ///
    /// Two planes and a ball meet at two points, `which` numbers them along the
    /// line the planes cross in, and turning one of the planes round turns that
    /// line round — so the same point is `which: 0` under one spelling and
    /// `which: 1` under the other. Folding the two spellings together there
    /// would name two different corners alike.
    #[test]
    fn a_ball_in_the_triple_is_named_as_each_face_spells_it() {
        let at = |normal, through| {
            Plane::from_point_and_normal(through, normal).expect("a normal with length")
        };
        let floor = at(Vector::new(0.0, 0.0, 1.0), Point::new(0.0, 0.0, 0.0));
        let side = at(Vector::new(0.0, 1.0, 0.0), Point::new(0.0, 0.0, 0.0));
        let ball = Surface::Sphere(
            Sphere::new(Point::new(0.0, 0.0, 0.0), 5.0).expect("a ball of radius five"),
        );

        let one_way = [Surface::Plane(floor), Surface::Plane(side), ball];
        let other_way = [Surface::Plane(floor.reversed()), Surface::Plane(side), ball];

        assert_ne!(
            names_of_the_three_surfaces(&one_way),
            names_of_the_three_surfaces(&other_way)
        );
    }

    /// A triple with a pipe in it is named the same whichever way round the
    /// pipe's axis is written, and still spells each plane as its own face
    /// holds it.
    ///
    /// The corner half of the fix, and both halves of it matter. Two solids
    /// pressed together on one pipe hold its axis both ways, and a corner on it
    /// solved from `plane, plane, pipe` by one face and from
    /// `plane, plane, pipe reversed` by another is one corner. Turning a
    /// **plane** round is still two names, because that would renumber `which`.
    #[test]
    fn a_pipe_in_the_triple_is_named_the_same_either_way_round() {
        let at = |normal, through| {
            Plane::from_point_and_normal(through, normal).expect("a normal with length")
        };
        let floor = at(Vector::new(0.0, 0.0, 1.0), Point::new(0.0, 0.0, 7.0));
        let side = at(Vector::new(0.0, 1.0, 0.0), Point::origin());
        let up = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
        let down = Surface::Cylinder(Cylinder::new(Axis::z().reversed(), 3.0).expect("a radius"));

        let one_way = [Surface::Plane(floor), Surface::Plane(side), up];
        let pipe_turned_round = [Surface::Plane(floor), Surface::Plane(side), down];
        let floor_turned_round = [Surface::Plane(floor.reversed()), Surface::Plane(side), up];

        assert_eq!(
            names_of_the_three_surfaces(&one_way),
            names_of_the_three_surfaces(&pipe_turned_round),
            "one pipe written two ways named one corner two things"
        );
        assert_ne!(
            names_of_the_three_surfaces(&one_way),
            names_of_the_three_surfaces(&floor_turned_round),
            "turning a plane round renumbers `which`, so the two spellings stay apart"
        );
    }

    /// Two bars end to end on one pipe are refused before the spelling of that
    /// pipe can matter, and this pin is what says so.
    ///
    /// The defect wants two solids flush on a shared curved face and one of them
    /// holding its axis the other way round. Two bars on one line is that pair,
    /// and the fuse does not reach it: it stops at *a point in a piece that
    /// every solid could place*, undecidable, and stops there **just the same
    /// when both bars spell the pipe the same way**. A point *was* found inside
    /// the piece here; what could not be done was classifying it against both
    /// bars, and §216.4 split the two so this pin says which. So the fold above
    /// is demonstrated on the names
    /// it fixes and not on an answer, and nothing here should be read as saying
    /// the fuse handles two solids on one pipe.
    ///
    /// The pin fails the day that wall moves, which is the only thing that makes
    /// a refusal worth keeping (§51.1).
    #[test]
    fn two_bars_on_one_pipe_are_refused_the_same_however_the_pipe_is_written() {
        let up = Cylinder::new(Axis::z(), 1.0).expect("a radius");
        let down = Cylinder::new(Axis::z().reversed(), 1.0).expect("a radius");
        assert_ne!(
            up.axis().direction(),
            down.axis().direction(),
            "the same pipe written two ways is different numbers, which is the difficulty"
        );

        let how_far_it_gets = |wall| {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let policy = Policy::certified();
            let mut log = ProvenanceLog::new();
            let lower = a_bar_between(&mut model, 0.0, 2.0, up, &policy, &mut log);
            let upper = a_bar_between(&mut model, 2.0, 4.0, wall, &policy, &mut log);
            format!(
                "{:?}",
                model
                    .fuse(
                        &[lower],
                        &[upper],
                        Rule::Union,
                        &policy,
                        &mut ProvenanceLog::new(),
                    )
                    .map(|fused| fused.solids().len())
            )
        };

        let both_ways_round = how_far_it_gets(down);
        assert_eq!(
            both_ways_round,
            how_far_it_gets(up),
            "the spelling of the pipe changed how far the fuse got"
        );
        assert!(
            both_ways_round.contains("a point in a piece that every solid could place"),
            "the wall moved, so the fold can be demonstrated on an answer now: {both_ways_round}"
        );
    }

    /// A bar of radius one standing between two heights, on the pipe it is
    /// handed, built the way [`Model::add_round_bar`] builds one.
    ///
    /// Built here rather than by that builder because the whole point is the
    /// spelling of the pipe, and the builder writes its own.
    fn a_bar_between(
        model: &mut Model,
        from: f64,
        to: f64,
        wall: Cylinder,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> crate::SolidKey {
        let mut draft = SolidBuilder::new();
        let bottom_corner = draft
            .add_vertex(Point::new(1.0, 0.0, from))
            .expect("a corner");
        let top_corner = draft
            .add_vertex(Point::new(1.0, 0.0, to))
            .expect("a corner");
        let seam = draft
            .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
            .expect("the seam up the wall");
        let bottom_circle = draft
            .add_edge(
                bottom_corner,
                bottom_corner,
                EdgeRun::RightRoundWhereItsFacesCross,
            )
            .expect("a cap circle");
        let top_circle = draft
            .add_edge(
                top_corner,
                top_corner,
                EdgeRun::RightRoundWhereItsFacesCross,
            )
            .expect("a cap circle");

        // A closed edge is stored running along `ŵ × ∇C` for the wall's
        // **stored** direction `ŵ` (`a_closed_crossing_is_walked_the_way_it_is_stored`),
        // so a bar whose wall is written with the axis pointing down walks both
        // its cap circles the other way round. Same bar, same skin, and the one
        // thing that is not even in the spelling.
        let round_a_circle = |turn| match (turn, wall.axis().direction().z() > 0.0) {
            (turn, true) => turn,
            (Orientation::Forward, false) => Orientation::Reversed,
            (_, false) => Orientation::Forward,
        };

        let mut cap = |z: f64, outward: f64, circle, turn| {
            let plane = Plane::from_point_and_normal(
                Point::new(0.0, 0.0, z),
                Vector::new(0.0, 0.0, outward),
            )
            .expect("the z axis is a unit normal");
            let coedge = draft
                .add_coedge(circle, round_a_circle(turn))
                .expect("a use of the circle");
            let wire = draft
                .add_wire(&[DraftStep::Along(coedge)])
                .expect("a cap is one circle");
            draft
                .add_face(Surface::Plane(plane), &[wire])
                .expect("a cap face")
        };
        let bottom = cap(from, -1.0, bottom_circle, Orientation::Reversed);
        let top = cap(to, 1.0, top_circle, Orientation::Forward);

        let round_the_wall = [
            draft
                .add_coedge(bottom_circle, round_a_circle(Orientation::Forward))
                .expect("a use"),
            draft.add_coedge(seam, Orientation::Forward).expect("a use"),
            draft
                .add_coedge(top_circle, round_a_circle(Orientation::Reversed))
                .expect("a use"),
            draft
                .add_coedge(seam, Orientation::Reversed)
                .expect("a use"),
        ]
        .map(DraftStep::Along);
        let wire = draft.add_wire(&round_the_wall).expect("round the wall");
        let side = draft
            .add_face(Surface::Cylinder(wall), &[wire])
            .expect("the wall");

        draft
            .add_shell(&[
                (bottom, Orientation::Forward),
                (top, Orientation::Forward),
                (side, Orientation::Forward),
            ])
            .expect("a closed skin");
        draft.build(model, policy, log).expect("a round bar")
    }

    /// A triangle walked forwards and then walked forwards again, which is one
    /// wire that closes and one that does not.
    ///
    /// Every edge is used by two coedges running opposite ways, so the checks
    /// before the wires have nothing to say; the second face's wire is the one
    /// thing wrong with the answer.
    fn a_triangle_whose_other_side_is_walked_the_wrong_way_round() -> Assembly {
        let up = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
        let corner = |x: f64, y: f64| Corner {
            point: Point::new(x, y, 0.0),
            wobble: 0.0,
            from: None,
        };
        let edge = |ends: [usize; 2]| Edge {
            ends,
            carried_by: [Surface::Plane(up), Surface::Plane(up.reversed())],
            arc: WhichArc::TheOnlyOne,
            from: Vec::new(),
        };
        let face = |surface: Plane, forward: bool| Face {
            ancestors: Vec::new(),
            surface: Surface::Plane(surface),
            used: Orientation::Forward,
            wires: vec![(0..3).map(|edge| Side { edge, forward }).collect()],
            is_a_whole_closed_surface: false,
        };

        Assembly {
            corners: vec![corner(0.0, 0.0), corner(1.0, 0.0), corner(0.0, 1.0)],
            edges: vec![edge([0, 1]), edge([1, 2]), edge([2, 0])],
            runs: vec![EdgeRun::Straight, EdgeRun::Straight, EdgeRun::Straight],
            faces: vec![face(up, true), face(up.reversed(), false)],
            shells: vec![vec![0, 1]],
            solids: vec![Lump {
                outer: 0,
                voids: Vec::new(),
            }],
        }
    }

    /// How many of each kind of entity a model holds.
    fn what_is_in_it(model: &Model) -> [usize; 7] {
        [
            model.data().vertices.len(),
            model.data().edges.len(),
            model.data().coedges.len(),
            model.data().wires.len(),
            model.data().faces.len(),
            model.data().shells.len(),
            model.data().solids.len(),
        ]
    }

    /// A broken answer is refused, and the caller's model does not so much as
    /// grow a slot.
    ///
    /// Before `docs/decisions.md` §87 the answer went straight into the
    /// caller's arena and was judged afterwards, so this same assembly left
    /// three corners, three edges, six coedges, two wires, two faces, a shell
    /// and a solid behind and *then* crashed. Nothing in this crate rolls back,
    /// so the crash was the only thing standing between the caller and a model
    /// holding a face whose wire does not close.
    #[test]
    fn an_answer_whose_wire_does_not_close_is_refused_and_nothing_is_written() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side two");
        let before = what_is_in_it(&model);

        let refused = a_triangle_whose_other_side_is_walked_the_wrong_way_round().commit(
            &mut model,
            None,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        match refused {
            Err(Error::TheFuseWroteABrokenAnswer { broken }) => assert_eq!(
                broken,
                Invalid::WireDoesNotClose { position: 1 },
                "the second face's wire is what the sweep should have caught"
            ),
            Err(other) => panic!("a wire that does not close is a broken answer, and gave {other}"),
            Ok(_) => panic!("a wire that does not close is not an answer to hand back"),
        }
        assert_eq!(
            what_is_in_it(&model),
            before,
            "a refused answer leaves the model exactly as it was"
        );
    }

    /// What one write of an answer made, in a form two models can be compared
    /// by.
    ///
    /// Slots are not comparable across models — the second write lands in
    /// whatever slots the first model had spare — so everything here is said in
    /// positions within [`Written`], which is the order `write_into` built
    /// things in. Wires and coedges have no list of their own there, so they are
    /// walked to through the faces that name them.
    #[derive(Debug, PartialEq)]
    struct Shape {
        corners: Vec<Point>,
        edges: Vec<(usize, usize, EdgeRun)>,
        faces: Vec<(Surface, Vec<Vec<Step>>)>,
        solids: Vec<Vec<Vec<(usize, Orientation)>>>,
    }

    /// One step of one wire: along an edge one way or the other, or standing
    /// still on a pole.
    #[derive(Debug, PartialEq)]
    enum Step {
        Along(usize, Orientation),
        Pole(usize),
    }

    fn shape_of(model: &Model, written: &Written) -> Shape {
        Shape {
            corners: written
                .vertices
                .iter()
                .map(|&slot| model.data().vertices[slot].point)
                .collect(),
            edges: written
                .edges
                .iter()
                .map(|&slot| {
                    let edge = &model.data().edges[slot];
                    (
                        at(&written.vertices, edge.start),
                        at(&written.vertices, edge.end),
                        edge.run.clone(),
                    )
                })
                .collect(),
            faces: written
                .faces
                .iter()
                .map(|&slot| {
                    let face = &model.data().faces[slot];
                    let wires = face
                        .wires
                        .iter()
                        .map(|&wire| {
                            model.data().wires[wire]
                                .steps
                                .iter()
                                .map(|&step| match step {
                                    WireStep::Along(coedge) => {
                                        let coedge = &model.data().coedges[coedge];
                                        Step::Along(
                                            at(&written.edges, coedge.edge),
                                            coedge.orientation,
                                        )
                                    }
                                    WireStep::Pole(vertex) => {
                                        Step::Pole(at(&written.vertices, vertex))
                                    }
                                })
                                .collect()
                        })
                        .collect();
                    (face.surface, wires)
                })
                .collect(),
            solids: written
                .solids
                .iter()
                .map(|&slot| {
                    model.data().solids[slot]
                        .shells
                        .iter()
                        .map(|used| {
                            model.data().shells[used.shell]
                                .faces
                                .iter()
                                .map(|used| (at(&written.faces, used.face), used.orient))
                                .collect()
                        })
                        .collect()
                })
                .collect(),
        }
    }

    /// Which position in the write this slot was, which is the one name for an
    /// entity that two models can agree on.
    fn at<K: Copy + PartialEq>(slots: &[K], wanted: K) -> usize {
        slots
            .iter()
            .position(|&slot| slot == wanted)
            .expect("an answer names only what it wrote")
    }

    /// The second write writes the same answer as the first.
    ///
    /// `Assembly::commit` builds its answer twice — once in a throwaway model
    /// it sweeps, and once in the caller's — and everything §87 claims rests on
    /// the two being the same answer. `Assembly::write_into` reads nothing out
    /// of the model it writes into, which is the argument; this is the
    /// demonstration CLAUDE.md's §16.10 rule asks for instead of the argument.
    ///
    /// The two models here are as unalike as the caller's and the throwaway
    /// ever are: one is empty, the other already holds both input cubes, so
    /// every slot the answer lands in differs between them.
    #[test]
    fn the_same_answer_is_written_into_an_empty_model_and_into_a_busy_one() {
        let scale = Scale::millimetres(100.0).expect("100 mm is a working size");
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();

        let mut busy = Model::new(scale);
        let one = busy.add_cube(2.0, &policy, &mut log).expect("a cube");
        let other = busy
            .add_cube(2.0, &policy, &mut log)
            .expect("a second cube");
        let along_the_diagonal = Isometry::translation(Vector::new(1.0, 1.0, 1.0))
            .expect("a whole-number step is a translation");
        busy.place_solid(other, &along_the_diagonal)
            .expect("this model made the key");

        let scratch = Scratch::split_both_groups(&busy, &[one], &[other], &policy, &mut log)
            .expect("two overlapping cubes split each other");
        let chosen =
            choose(&busy, &scratch, Rule::Union, &policy, &mut log).expect("a union of two cubes");
        let assembly =
            assemble(&busy, &scratch, &chosen, &policy, &mut log).expect("an answer to write");

        let mut empty = Model::new(scale);
        let into_empty = assembly
            .commit(&mut empty, None, &policy, &mut log)
            .expect("the answer is sound");
        let into_busy = assembly
            .commit(&mut busy, None, &policy, &mut log)
            .expect("the answer is sound in the model that holds the inputs too");

        assert_eq!(
            shape_of(&empty, &into_empty),
            shape_of(&busy, &into_busy),
            "the two writes wrote different answers"
        );
    }
}
