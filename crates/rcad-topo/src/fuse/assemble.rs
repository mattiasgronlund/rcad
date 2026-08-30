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
//! # Which shell is a void of which
//!
//! Nesting asks whether a point on one shell is inside another, and the thing
//! that answers that is [`SolidView::classify_point`](crate::SolidView), which
//! wants a [`Model`]. So the shells are written into a scratch model first, one
//! solid each, and asked there. The real model sees the finished answer or
//! nothing at all.

use std::collections::BTreeMap;

use rcad_geom::{Isometry, Orientation, Plane, Point, Surface};
use rcad_predicates::{
    Approximation, Cause, Measures, Point as PredicatePoint, Policy, ProvenanceLog,
};
use smallvec::SmallVec;

use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, ShellSlot, SolidSlot, VertexSlot, WireSlot};
use crate::model::{FaceUse, ShellUse};
use crate::{Containment, EdgeKey, Error, FaceKey, Invalid, Model, Result, VertexKey};

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
    assembly.shells = assembly.group_into_shells()?;
    assembly.solids = assembly.nest_the_shells(model, policy, log)?;
    Ok(assembly)
}

/// The whole result, in scratch.
pub(crate) struct Assembly {
    corners: Vec<Corner>,
    edges: Vec<Edge>,
    faces: Vec<Face>,
    shells: Vec<Vec<usize>>,
    solids: Vec<Lump>,
}

impl Assembly {
    /// The input face each face of the result was cut out of, in the order
    /// [`Assembly::write_into`] writes them.
    pub(crate) fn face_ancestors(&self) -> impl Iterator<Item = FaceKey> + '_ {
        self.faces.iter().map(|face| face.ancestor)
    }

    /// The input edge each edge of the result is a piece of, where it is a piece
    /// of one, in the order [`Assembly::write_into`] writes them.
    pub(crate) fn edge_ancestors(&self) -> impl Iterator<Item = Option<EdgeKey>> + '_ {
        self.edges.iter().map(|edge| edge.from)
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
    pub(crate) fn commit(
        &self,
        model: &mut Model,
        inherited: Option<Approximation>,
        policy: &Policy,
    ) -> Written {
        let approximation_before = model.worst_approximation();
        let written = self.write_into(model, &self.solids);
        let worst = self.tag_everything_it_built(model, &written, inherited);
        model.assert_approximation_did_not_grow(approximation_before.max(worst));

        if cfg!(debug_assertions) {
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
            faces.push(Face {
                ancestor: piece.ancestor,
                plane: if turned_round {
                    piece.plane.reversed()
                } else {
                    piece.plane
                },
                wires,
            });
        }
        Self {
            corners: corners.made,
            edges: edges.made,
            faces,
            shells: Vec::new(),
            solids: Vec::new(),
        }
    }

    /// Gather the faces into lumps joined by edges, refusing a result that is
    /// not a manifold.
    ///
    /// # Errors
    ///
    /// [`Error::ResultIsNotManifold`] when an edge is claimed by other than two
    /// faces. `Model` means an edge is used by exactly two coedges everywhere,
    /// and relaxing that changes what a model *is* rather than what one boolean
    /// does (§17.4).
    fn group_into_shells(&self) -> Result<Vec<Vec<usize>>> {
        let mut claimants: Vec<Vec<usize>> = vec![Vec::new(); self.edges.len()];
        for (index, face) in self.faces.iter().enumerate() {
            for side in face.wires.iter().flatten() {
                claimants[side.edge].push(index);
            }
        }
        if let Some(claimed) = claimants.iter().find(|by| by.len() != 2) {
            return Err(Error::ResultIsNotManifold {
                count: claimed.len(),
            });
        }

        let mut same = Sameness::of(self.faces.len());
        for by in &claimants {
            same.join(by[0], by[1]);
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
        let written = self.write_into(&mut apart, &one_each);

        let mut volumes = Vec::new();
        for &solid in &written.solids {
            let volume = apart.solid(apart.key_for_solid(solid)).signed_volume();
            if !volume.is_finite() || volume == 0.0 {
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
                    Containment::OnBoundary | Containment::Undecidable { .. } => (),
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

    /// Build every entity of the result in `model`, in a fixed order.
    ///
    /// Used twice: once on a scratch model, to ask which shell is inside which,
    /// and once on the caller's model, which is the commit.
    ///
    /// # Panics
    ///
    /// If what it just built breaks one of the invariants checked on touched
    /// entities. That is a bug here, not bad input.
    fn write_into(&self, model: &mut Model, lumps: &[Lump]) -> Written {
        let vertices: Vec<VertexSlot> = self
            .corners
            .iter()
            .map(|corner| model.add_vertex(corner.point))
            .collect();
        let edges: Vec<EdgeSlot> = self
            .edges
            .iter()
            .map(|edge| model.add_edge(vertices[edge.ends[0]], vertices[edge.ends[1]]))
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
            faces.push(model.add_face(Surface::Plane(face.plane), wires));
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
                        orient: Orientation::Forward,
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
    ends: [usize; 2],
    /// The input edge this is a piece of, when it is one.
    from: Option<EdgeKey>,
}

/// One face of the result.
struct Face {
    /// The input face it was cut out of. Every face of the result has exactly
    /// one, because nothing in the fuse invents geometry with no ancestor.
    ancestor: FaceKey,
    /// The plane, already turned round if the rule turned the face round.
    plane: Plane,
    wires: Vec<Vec<Side>>,
}

/// One side of one wire of one face: an edge, walked one way or the other.
struct Side {
    edge: usize,
    forward: bool,
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
/// Never a coordinate. Two faces solving the same three planes in a different
/// order round differently, so comparing what they rounded to would either miss
/// a join or invent one; comparing the *names* of the planes cannot.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CornerName {
    /// A corner the input already had.
    Vertex(VertexSlot),
    /// The meet of three planes, each written out as the bits of its four
    /// coefficients, sorted so the three arrive the same way round whichever
    /// face asked.
    Meet([[u64; 4]; 3]),
}

impl Corners {
    fn of(&mut self, model: &Model, end: &MeetPoint) -> usize {
        let name = match end {
            MeetPoint::Existing { vertex, .. } => CornerName::Vertex(vertex.slot_in(model.id())),
            MeetPoint::OfThreePlanes { planes, .. } => {
                let mut named = planes.map(|plane| plane.coefficients().map(f64::to_bits));
                named.sort_unstable();
                CornerName::Meet(named)
            }
        };
        if let Some(&already) = self.named.get(&name) {
            return already;
        }
        let point = end.point();
        let at = middle_of(point);
        let was = match end {
            MeetPoint::Existing { vertex, .. } => Some(*vertex),
            MeetPoint::OfThreePlanes { .. } => None,
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

/// The edges built so far, and how to find one again.
#[derive(Default)]
struct Edges {
    made: Vec<Edge>,
    between: BTreeMap<[usize; 2], usize>,
}

impl Edges {
    /// The edge between two corners, walked from `from` to `to`.
    ///
    /// # Panics
    ///
    /// If the two ends are the same corner. §17.4 prevents a side of no length
    /// rather than filtering one out afterwards, so one reaching here means the
    /// splitter is wrong.
    fn of(&mut self, from: usize, to: usize, along: Option<EdgeKey>) -> Side {
        assert!(from != to, "a piece of a face has a side with no length");
        let named = [from.min(to), from.max(to)];
        let edge = *self.between.entry(named).or_insert_with(|| {
            self.made.push(Edge {
                ends: named,
                from: along,
            });
            self.made.len() - 1
        });
        Side {
            edge,
            forward: self.made[edge].ends[0] == from,
        }
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
    model: &Model,
    corners: &mut Corners,
    edges: &mut Edges,
) -> Vec<Side> {
    let mut sides = Vec::new();
    for meet in wire {
        let from = corners.of(model, &meet.ends[0]);
        let to = corners.of(model, &meet.ends[1]);
        sides.push(edges.of(from, to, meet.along));
    }
    if turned_round {
        sides.reverse();
        for side in &mut sides {
            side.forward = !side.forward;
        }
    }
    sides
}
