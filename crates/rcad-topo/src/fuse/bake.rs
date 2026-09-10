//! The input solids, moved into world coordinates with every corner re-derived.
//!
//! Placement is baked in here and nowhere else (§17.3). The transform's rounding
//! is representation wobble, and it is tagged as such rather than refused on.
//!
//! # A face is its surface, and a corner has two routes
//!
//! §17.1 says a face *is* its plane and the stored corners are cached samples of
//! it, so every corner is re-derived at the door. §31.3 changes one word and
//! adds the case that word opens up: a corner of a *round bar* has two surfaces
//! round it — the cap's plane and the wall's cylinder — and the whole cap circle
//! satisfies both, so there is nothing to solve. **A corner with three or more
//! surfaces round it is re-derived, and a corner with fewer is read out of the
//! model and moved.**
//!
//! Read and moved is not a retreat. `check_curved_face` has already certified
//! that the stored corner is [`Decision::Zero`] on its surface, which is a
//! stronger promise than the box a Cramer solve gives — and the moved corner
//! becomes a box of its own, because `Isometry::apply_to_point` rounds and a
//! moved corner handed on as an exact triple would certify a confident sign
//! about a rounding (§34.6).
//!
//! # A seam is not a boundary and is left out
//!
//! §31.3 says a seam is an ordinary boundary edge whose second surface is read
//! out of its own [`EdgeRun`]. **That is wrong, and [`seams_are_left_out`] is
//! the demonstration**: a seam is used *twice by one face*, so both uses run
//! along one arc of one crossing in opposite directions, and an arrangement that
//! took it as boundary would have one stretch of one curve claimed by two sides
//! of one wire at once. What a seam is, is where the surface's own two
//! coordinates were cut open — not where the face stops — which is the same
//! sentence [`what_trims_this_face`](crate::holds) already says on the other
//! side of the crate.
//!
//! One thing follows that §33.8 and §34.9 both expected to go the other way:
//! **[`Picker`](rcad_geom::Picker) still needs no `transformed`.** The only
//! stored picker in the kernel is the one that says which half of a meridian a
//! seam is, and nothing reads it here.

use std::collections::{BTreeMap, BTreeSet};

use rcad_geom::{
    Isometry, MeetOfThree, Orientation, Plane, Point, Surface, WHERE_THREE_SURFACES_MEET,
    meet_of_three_surfaces, where_a_corner_lands_after_moving,
};
use rcad_predicates::{
    Decision, Interval, Measures, Point as PredicatePoint, Policy, ProvenanceLog,
};
use smallvec::SmallVec;

use crate::key::{FaceSlot, VertexSlot};
use crate::validate::{MEASURING_SLACK, side_of_surface};
use crate::{EdgeKey, FaceKey, FaceView, Invalid, Model, Result, SolidKey, SolidView, VertexKey};

use super::assemble::{InputFace, joined_faces_of_an_input};
use super::{as_exact_as_it_is, gave_up, middle_of};

/// Move every solid in `solids` into world coordinates.
///
/// A corner is refused only when a surface around it *proves* it is somewhere
/// else, or when nothing can bound where the surfaces round it meet at all.
///
/// # Errors
///
/// [`Error::Invalid`](crate::Error::Invalid) when a corner's own faces disagree
/// about where it is by more than its box, and
/// [`Error::Undecidable`](crate::Error::Undecidable) when no three surfaces
/// around a corner could be shown to meet at a point.
///
/// # Panics
///
/// If a solid is broken in a way [`Model::validate`] rejects: a face used at two
/// placements, or a corner whose faces are not all at one placement.
pub(crate) fn bake(
    model: &Model,
    solids: &[SolidKey],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<BakedSolid>> {
    solids
        .iter()
        .map(|&solid| bake_one_solid(model, solid, policy, log))
        .collect()
}

/// One input solid, in world coordinates.
#[derive(Clone, Debug)]
pub(crate) struct BakedSolid {
    /// The solid this came from.
    pub(crate) key: SolidKey,
    /// Its faces, in the order the model stores them.
    pub(crate) faces: Vec<BakedFace>,
    /// A box round the whole solid, for skipping face pairs that cannot meet.
    pub(crate) bounds: Bounds,
}

/// One face of an input solid, in world coordinates.
#[derive(Clone, Debug)]
pub(crate) struct BakedFace {
    /// The faces this came from. Every piece cut out of it names them as its
    /// ancestors. More than one when coplanar faces of the input were joined
    /// into a single face on the way in.
    pub(crate) keys: Vec<FaceKey>,
    /// The surface it sits on, already moved into place. This is the face
    /// (§17.1, §31.3); the corners below are cached samples of it.
    pub(crate) surface: Surface,
    /// Which way round the solid uses the face.
    pub(crate) orientation: Orientation,
    /// The wires that trim it, each walked the way the face itself stores it,
    /// **with its seams left out** — see this module's header.
    ///
    /// The first is the face's own first wire, which [`FaceData`] calls the
    /// outer one by fiat. On a closed surface that is an order the face stores
    /// and not a fact about the geometry (§31.3), so nothing below reads it as
    /// one.
    ///
    /// [`FaceData`]: crate::model::FaceData
    pub(crate) wires: Vec<Vec<BakedEdge>>,
    /// The second surface of each crossing cutting this face, filled in by the
    /// gathering step. The first is always the face's own.
    pub(crate) cuts: Vec<Surface>,
    /// A box round the face.
    pub(crate) bounds: Bounds,
}

/// One edge of one wire of a face.
#[derive(Clone, Debug)]
pub(crate) struct BakedEdge {
    /// The input edges this is. Carried so that a piece of it can name the edges
    /// it came out of, which is what the history's edge images are made of. More
    /// than one when joining coplanar input faces dropped a corner between two
    /// of them.
    pub(crate) keys: SmallVec<[EdgeKey; 1]>,
    /// The corner the wire is at when it starts along this edge.
    pub(crate) start: BakedCorner,
    /// The corner it is at when it stops.
    ///
    /// **Not the next side's start**, and that is what a closed edge changes: a
    /// wire with a seam in it stops being a chain once the seam is left out, and
    /// a cap circle starts and ends at the one corner it has.
    pub(crate) end: BakedCorner,
    /// The surface of the face on the other side of this edge. Together with the
    /// face's own surface it *is* where the edge runs (§17.4, §31.3).
    pub(crate) across: Surface,
}

/// A corner of an input face, in world coordinates.
#[derive(Clone, Debug)]
pub(crate) struct BakedCorner {
    /// The input vertex this is.
    pub(crate) vertex: VertexKey,
    /// The box it is somewhere inside.
    ///
    /// How far the corner may be from where its surfaces really meet is the
    /// width of that box — representation wobble (§17.2), present even though
    /// every decision was proved, because a re-derivation divides and a
    /// placement rounds. It is carried on the point rather than beside it so
    /// that the number tagged on the committed vertex and the number the
    /// arrangement decided with cannot drift apart.
    pub(crate) point: PredicatePoint,
    /// Every face surface that meets here, in a fixed order. Which crossings run
    /// through a corner is then a question about the model rather than about
    /// arithmetic, which is what keeps the arrangement from having to prove a
    /// corner is on a curve it is on by construction.
    pub(crate) around: Vec<Surface>,
}

/// A box round some geometry, in world coordinates.
///
/// Only ever used to skip pairs that cannot meet. A box test changes no answer,
/// so plain `f64` is honest here in a way it would not be one line further on.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Bounds {
    low: [f64; 3],
    high: [f64; 3],
}

impl Bounds {
    /// Whether the two boxes have any point in common.
    pub(crate) fn overlap(self, other: Bounds) -> bool {
        (0..3).all(|axis| self.low[axis] <= other.high[axis] && other.low[axis] <= self.high[axis])
    }

    /// Whether the box provably holds no part of `point`'s own box.
    ///
    /// A proof and not a hint: [`Bounds`] round a face is a box the face is
    /// certainly inside, so a point whose whole enclosure is clear of it is
    /// certainly off the face. Plain `f64` is honest here because the comparison
    /// only ever *widens* what is kept.
    pub(crate) fn provably_misses(self, point: PredicatePoint) -> bool {
        let box_of_it = point.enclosure();
        (0..3).any(|axis| {
            box_of_it[axis].lo() > self.high[axis] || box_of_it[axis].hi() < self.low[axis]
        })
    }

    fn nothing() -> Self {
        Self {
            low: [f64::INFINITY; 3],
            high: [f64::NEG_INFINITY; 3],
        }
    }

    /// A box round one face of one solid, curved or flat.
    ///
    /// **Not a box round the face's corners**, which is what this used to read
    /// off [`FaceView::for_each_triangle`] and which is wrong for every curved
    /// face there is (§35.4): a round bar's wall fans into two triangles round
    /// its seam, a line the wall bulges a whole radius out of, and a round cap
    /// fans into nothing at all. A pair skipped on either would leave one of two
    /// faces along an edge split and the other not.
    fn round_a_face(solid: SolidView<'_>, face: FaceView<'_>) -> Self {
        match solid.box_around_one_face(face) {
            None => Self::nothing(),
            Some((low, high)) => Self {
                low: low.coordinates(),
                high: high.coordinates(),
            },
        }
    }

    fn take_in_point(&mut self, point: Point) {
        for (axis, coordinate) in point.coordinates().into_iter().enumerate() {
            self.low[axis] = self.low[axis].min(coordinate);
            self.high[axis] = self.high[axis].max(coordinate);
        }
    }

    fn take_in(&mut self, other: Bounds) {
        for axis in 0..3 {
            self.low[axis] = self.low[axis].min(other.low[axis]);
            self.high[axis] = self.high[axis].max(other.high[axis]);
        }
    }
}

/// Move one solid into world coordinates, joining any two faces of it that are
/// really one flat region first.
///
/// A solid built by this fuse never has such a pair (§17.4's amendment of
/// 2026-08-30), and one built by `ProfileBuilder` from three profile points in a
/// row does. Two faces on one plane with an edge between them carry no crossing
/// for that edge to run along, so the arrangement would be handed a boundary
/// edge with nowhere to be.
///
/// **The joiner is only asked about a wholly flat solid**, because it is written
/// in planes and nothing in this milestone builds a curved solid with two faces
/// on one surface. A curved *answer* of this fuse could have one; it would
/// arrive at [`split_a_face`](super::arrange::split_a_face) as a face whose
/// neighbour across an edge is the same surface, which refuses by name rather
/// than guessing. That refusal is where a lane that meets this finds it.
fn bake_one_solid(
    model: &Model,
    key: SolidKey,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<BakedSolid> {
    let solid = model.solid_at(key.slot_in(model.id()));
    if solid.is_wholly_flat()
        && let Some(joined) = joined_faces_of_an_input(model, solid, log)?
    {
        return bake_the_joined_faces(model, key, solid, &joined, policy, log);
    }

    let surfaces = surface_of_every_face(model, solid);
    let corners = corner_of_every_vertex(model, solid, &surfaces, policy, log)?;

    let mut faces = Vec::new();
    let mut bounds = Bounds::nothing();
    for face in solid.faces() {
        let baked = bake_one_face(model, solid, face, &surfaces, &corners);
        bounds.take_in(baked.bounds);
        faces.push(baked);
    }
    Ok(BakedSolid { key, faces, bounds })
}

/// The same, for a solid whose coplanar faces have just been joined.
///
/// The joined faces already carry the planes and wires the arrangement wants, so
/// all that is left is to solve each corner that survived the joining from the
/// planes now round it. Every one of them has three, which is the whole point of
/// having joined.
fn bake_the_joined_faces(
    model: &Model,
    key: SolidKey,
    solid: SolidView<'_>,
    joined: &[InputFace],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<BakedSolid> {
    let corners = corner_of_every_vertex_left(model, solid, joined, policy, log)?;

    let mut faces = Vec::new();
    let mut bounds = Bounds::nothing();
    for face in joined {
        let mut round_this_face = Bounds::nothing();
        let mut wires = Vec::new();
        for ring in &face.wires {
            let mut walked = Vec::new();
            for (position, side) in ring.iter().enumerate() {
                let start = corners[&side.start.slot_in(model.id())].clone();
                let next = ring[(position + 1) % ring.len()].start;
                round_this_face.take_in_point(middle_of(start.point));
                walked.push(BakedEdge {
                    keys: side.edges.clone(),
                    start,
                    end: corners[&next.slot_in(model.id())].clone(),
                    across: Surface::Plane(side.across),
                });
            }
            wires.push(walked);
        }
        bounds.take_in(round_this_face);
        faces.push(BakedFace {
            keys: face.faces.clone(),
            surface: Surface::Plane(face.plane),
            orientation: Orientation::Forward,
            wires,
            cuts: Vec::new(),
            bounds: round_this_face,
        });
    }
    Ok(BakedSolid { key, faces, bounds })
}

/// Solve every corner the joining left, from the planes of the joined faces
/// round it.
///
/// # Panics
///
/// If fewer than three joined faces meet at a corner. Joining drops exactly the
/// corners that a lump of faces runs straight through, so what is left is a
/// corner of a closed solid and has at least three faces round it.
fn corner_of_every_vertex_left(
    model: &Model,
    solid: SolidView<'_>,
    joined: &[InputFace],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<BTreeMap<VertexSlot, BakedCorner>> {
    let surfaces = surface_of_every_face(model, solid);
    let mut around: BTreeMap<VertexSlot, (VertexKey, Vec<Surface>)> = BTreeMap::new();
    for face in joined {
        let mut its_corners: BTreeMap<VertexSlot, VertexKey> = BTreeMap::new();
        for side in face.wires.iter().flatten() {
            its_corners.insert(side.start.slot_in(model.id()), side.start);
        }
        for (slot, vertex) in its_corners {
            around
                .entry(slot)
                .or_insert((vertex, Vec::new()))
                .1
                .push(Surface::Plane(face.plane));
        }
    }

    around
        .into_iter()
        .map(|(slot, (vertex, planes))| {
            assert!(
                planes.len() >= 3,
                "a corner of a closed solid has at least three faces round it, this one has {}",
                planes.len()
            );
            let stored = where_the_model_puts_a_corner(
                model,
                slot,
                &faces_around_a_vertex(model, slot, &surfaces),
            )?;
            Ok((slot, derive_a_corner(vertex, stored, planes, policy, log)?))
        })
        .collect()
}

/// Every face of the solid with its surface already moved into place, and where
/// it was moved from.
///
/// # Panics
///
/// If the solid uses one face twice. Two uses can sit at two placements, and
/// then "the surface of that face" has two answers; [`Model::validate`] refuses
/// it as [`Invalid::UsedMoreThanOnce`].
fn surface_of_every_face(model: &Model, solid: SolidView<'_>) -> BTreeMap<FaceSlot, PlacedSurface> {
    let mut surfaces = BTreeMap::new();
    for face in solid.faces() {
        let slot = face.key().slot_in(model.id());
        let already_there = surfaces.insert(
            slot,
            PlacedSurface {
                stored: model.data().faces[slot].surface,
                placed: face.surface(),
                movement: face.location(),
            },
        );
        assert!(
            already_there.is_none(),
            "a solid uses one face twice, so it has two surfaces"
        );
    }
    surfaces
}

/// One face's surface, moved into place, and the movement that put it there.
///
/// The movement is what a corner read out of the model needs: the box round
/// where it lands is worked out from the *stored* corner and the *stored*
/// surfaces, and applying the movement to each of them separately is what
/// [`where_a_corner_lands_after_moving`] does.
#[derive(Clone, Copy, Debug)]
struct PlacedSurface {
    stored: Surface,
    placed: Surface,
    movement: Isometry,
}

fn corner_of_every_vertex(
    model: &Model,
    solid: SolidView<'_>,
    surfaces: &BTreeMap<FaceSlot, PlacedSurface>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<BTreeMap<VertexSlot, BakedCorner>> {
    let mut corners = BTreeMap::new();
    for face in solid.faces() {
        for wire in model.face_at(face.key().slot_in(model.id())).wires() {
            for coedge in wire.coedges() {
                for vertex in [coedge.start().key(), coedge.end().key()] {
                    let slot = vertex.slot_in(model.id());
                    if corners.contains_key(&slot) {
                        continue;
                    }
                    let around = faces_around_a_vertex(model, slot, surfaces);
                    corners.insert(
                        slot,
                        place_a_corner(model, vertex, slot, &around, policy, log)?,
                    );
                }
            }
        }
    }
    Ok(corners)
}

/// The surfaces of every face of this solid that meets at this vertex.
///
/// # Panics
///
/// If none does, which means the vertex was reached through a face that does not
/// have it, so the model was already broken.
fn faces_around_a_vertex(
    model: &Model,
    vertex: VertexSlot,
    surfaces: &BTreeMap<FaceSlot, PlacedSurface>,
) -> Vec<PlacedSurface> {
    let mut faces = BTreeSet::new();
    for edge in model.vertex_at(vertex).edges().collect::<Vec<_>>() {
        for face in model.edge_at(edge.slot_in(model.id())).neighbouring_faces() {
            let slot = face.slot_in(model.id());
            if surfaces.contains_key(&slot) {
                faces.insert(slot);
            }
        }
    }
    assert!(
        !faces.is_empty(),
        "a corner of a solid is a corner of at least one of its faces"
    );
    faces.iter().map(|slot| surfaces[slot]).collect()
}

/// Where a corner of an input is, by whichever of §31.3's two routes its own
/// faces allow.
///
/// Three or more distinct surfaces round it and it is re-derived from the
/// best-conditioned triple. Fewer, and the whole of the curve those surfaces
/// share passes through it, so there is nothing to solve and it is read out of
/// the model and moved.
///
/// # Panics
///
/// If the faces round one corner are at different placements. Then the corner
/// has two places at once, and a solid like that is broken rather than awkward.
fn place_a_corner(
    model: &Model,
    vertex: VertexKey,
    slot: VertexSlot,
    around: &[PlacedSurface],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<BakedCorner> {
    let where_the_model_puts_it = where_the_model_puts_a_corner(model, slot, around)?;

    let mut distinct: Vec<Surface> = Vec::with_capacity(around.len());
    for face in around {
        if !distinct.contains(&face.placed) {
            distinct.push(face.placed);
        }
    }
    if distinct.len() >= 3 {
        return derive_a_corner(vertex, where_the_model_puts_it, distinct, policy, log);
    }
    Ok(BakedCorner {
        vertex,
        point: where_the_model_puts_it,
        around: distinct,
    })
}

/// Where the model itself says a corner is, once its solid's placement has been
/// applied — the box it is really known to be inside.
///
/// This is a **cached sample** of where the surfaces round it meet (§17.1) and
/// never the answer: [`place_a_corner`] returns it only for a corner with fewer
/// than three surfaces round it, where the whole of a shared curve satisfies
/// them and there is nothing to solve, and [`derive_a_corner`] reads it only to
/// tell two meeting points apart.
///
/// # Errors
///
/// Whatever [`where_a_corner_lands_after_moving`] makes of a corner, a placement
/// or a surface holding a number that is not finite.
///
/// # Panics
///
/// If the faces round one corner are at different placements. Then the corner
/// has two places at once, and a solid like that is broken rather than awkward.
fn where_the_model_puts_a_corner(
    model: &Model,
    slot: VertexSlot,
    around: &[PlacedSurface],
) -> Result<PredicatePoint> {
    let movement = around[0].movement;
    assert!(
        around.iter().all(|face| face.movement == movement),
        "the faces round one corner are at different placements, so the corner is in two places"
    );
    let stored = model.data().vertices[slot].point;
    // A solid standing where it was built is not moved at all, and a box round
    // a move that rounded nothing would turn an exact corner into an inexact
    // one — which is the whole of §17.7's lattice control.
    if movement == Isometry::identity() {
        return Ok(PredicatePoint::from(stored.coordinates()));
    }
    // A pole of a ball has one face round it and one surface, and its box is
    // that surface's rounding counted once for each of the two chains the bound
    // is derived over. Handing the same surface twice is what says so.
    let on = [around[0].stored, around[around.len() - 1].stored];
    // The width of the box is the whole of what the move cost, and the
    // `Approximation` beside it says the same number a second time. §34.9's trap
    // is that the box is for asking predicates and its middle is for storing, so
    // the one thing that must not happen is the two drifting apart — and the way
    // to make that impossible is to keep only one of them.
    let (point, _) = where_a_corner_lands_after_moving(stored, [&on[0], &on[1]], &movement)?;
    Ok(as_exact_as_it_is(point))
}

/// Solve a corner from the surfaces around it (§17.1, §31.3).
///
/// A corner where four or more faces meet is over-determined, so it is solved
/// from the best-conditioned triple and every remaining surface is then asked
/// whether it agrees. [`meet_of_three_surfaces`] answering
/// [`MeetOfThree::Nowhere`] or [`MeetOfThree::MoreThanAPoint`] *is* the
/// conditioning filter, so there is no second threshold here to argue about —
/// and neither of those is a refusal (§34.9), it is that triple saying it does
/// not place this corner while another may.
///
/// Three surfaces can meet at more than one point — two planes and a pipe cross
/// at both ends of a chord — and `where_the_model_puts_it` is what says which of
/// them this corner is. See [`the_place_the_model_points_at`] for how it picks
/// and when it declines to.
///
/// The search stops at the first triple that solves to an exact point, because
/// no later triple can beat a box of no width and the loop is cubic in the faces
/// round the corner (§70.3). That is the same answer the whole sweep gave: the
/// widths are compared with `<`, so a second exact triple never displaced the
/// first anyway.
///
/// A surface that comes back [`Decision::Pos`] or [`Decision::Neg`] has *proved*
/// the corner is somewhere else, which means the input's corners and its
/// surfaces disagree by more than roundoff. [`Decision::Undecidable`] is
/// agreement, not a failure: the surface passes through the corner's box, which
/// is as much as anything can say about a point that had to be built.
///
/// Every triple failing is not a geometry error: it is a corner nothing can
/// place, which leaves the operation with nothing to return, so it is
/// [`Error::Undecidable`](crate::Error::Undecidable) with a record naming
/// [`WHERE_THREE_SURFACES_MEET_HERE`].
fn derive_a_corner(
    vertex: VertexKey,
    where_the_model_puts_it: PredicatePoint,
    around: Vec<Surface>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<BakedCorner> {
    let mut best: Option<PredicatePoint> = None;
    'every_triple: for first in 0..around.len() {
        for second in (first + 1)..around.len() {
            for third in (second + 1)..around.len() {
                let triple = [around[first], around[second], around[third]];
                let Ok(MeetOfThree::At(places)) =
                    meet_of_three_surfaces([&triple[0], &triple[1], &triple[2]], policy, log)
                else {
                    continue;
                };
                let Some(place) = the_place_the_model_points_at(&places, where_the_model_puts_it)
                else {
                    continue;
                };
                let point = as_exact_as_it_is(place);
                if best.is_none_or(|best| point.uncertainty() < best.uncertainty()) {
                    best = Some(point);
                }
                if point.uncertainty() == 0.0 {
                    break 'every_triple;
                }
            }
        }
    }
    let Some(point) = best else {
        return Err(gave_up(
            WHERE_THREE_SURFACES_MEET,
            &[middle_of_the_places(&around)],
            Measures::Volume,
            log,
        ));
    };

    for surface in &around {
        match side_of_surface(surface, point) {
            Decision::Zero | Decision::Undecidable { .. } => (),
            Decision::Neg | Decision::Pos => {
                return Err(match surface {
                    Surface::Plane(flat) => Invalid::VertexOffItsFacePlane {
                        distance: flat.signed_distance_to(middle_of(point)).abs(),
                    },
                    Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | _ => {
                        Invalid::VertexOffItsFaceSurface
                    }
                }
                .into());
            }
            other => {
                panic!("side of a surface came back with a decision nothing here knows: {other:?}")
            }
        }
    }

    Ok(BakedCorner {
        vertex,
        point,
        around,
    })
}

/// Which of the places three surfaces meet at this corner is, or `None` when
/// this triple does not say.
///
/// The model's own corner is what picks. It is a cached sample of where these
/// surfaces meet (§17.1), moved into world coordinates and carrying the box that
/// move cost, so **the place whose box overlaps it is the one**.
///
/// **A single place is asked the same question**, which is §75.6's "comparison
/// as a refusal": the one thing the surfaces said is still only taken when the
/// model's corner is somewhere it could be. Skipping the test where there is
/// nothing to choose between meant a corner could sit anywhere at all and still
/// be replaced by the meet without a word — the ill-conditioned corner of §70.3,
/// three surfaces at a grazing angle and a corner far from where they cross.
///
/// Two places that both overlap it are two answers to one question, and a triple
/// where no place overlaps it at all is the model's corner and its surfaces
/// disagreeing by more than the boxes and that room allow. Both give `None`, and
/// neither is a refusal: it is this triple saying nothing about this corner while
/// another may still place it.
///
/// # The stored corner is grown first, and that is a tolerance
///
/// It would be nicer if it were not. Two certified boxes round two true points
/// need no tolerance to compare, and this test was written that way — but the
/// model's corner and the model's surfaces are not two readings of one point.
/// A face pins its plane through one corner, so every *other* corner of that
/// face satisfies `n·p = off` only to a rounding, and the meet those planes
/// solve to sits a few ulps from the corner the model wrote down. Both boxes
/// have no width at all when the arithmetic is exact, so a few ulps is a miss,
/// and asking for the corner bit for bit refused 30 solids the kernel had
/// already accepted, plus changed the reason for 3 more that were refused
/// already — the skew corpus went from 51 refused to 81 (§75.6).
///
/// So [`MEASURING_SLACK`] × [`Plane::SIGNED_DISTANCE_ROUNDOFF`] × the corner's
/// own size is added to the stored box on every side. That is **the same
/// allowance [`check_flat_face`](crate::validate) grants the same disagreement**,
/// read from the same constant rather than copied, because the two have to
/// agree: a solid `Model::validate` accepts is one the fuse has to be able to
/// bake. It is the width of the number and not a modelling tolerance —
/// `Scale::RELATIVE_LINEAR_TOLERANCE` is 70 369 times wider — not quite five
/// orders of magnitude — and would swallow corners somebody *placed* off their
/// planes.
///
/// **This never places the point.** It picks among places the surfaces already
/// agree on; the geometry comes from the meet, which is what keeps a wrong
/// cached corner from moving a right one.
fn the_place_the_model_points_at(
    places: &[PredicatePoint],
    where_the_model_puts_it: PredicatePoint,
) -> Option<PredicatePoint> {
    let stored = where_the_model_puts_it.enclosure();
    let room = what_writing_the_corner_down_cost(stored);
    let mut found = None;
    for place in places {
        if !boxes_overlap(place.enclosure(), stored, room) {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(*place);
    }
    found
}

/// How far the model's stored corner can sit from where its own surfaces cross
/// purely because a plane through one corner of a face misses the next one.
///
/// The size is the **corner's own**, not the model's largest coordinate the way
/// [`Model::what_writing_a_plane_down_costs`](crate::validate) measures it.
/// Nothing here has a model to ask — this is handed two points — and threading
/// one down would hand a corner at the origin the room a corner a kilometre
/// away earned, loosening the test everywhere one feature sits far out. The
/// price is a corner near the origin on a face whose other corners are huge:
/// the rounding there is the big face's and this grants the small corner's, so
/// the triple says nothing and another has to place it. That is a refusal
/// rather than a wrong answer, which is the direction to be wrong in.
///
/// Floored at one for the same reason [`select`](super::select) floors its own
/// round-trip room: a corner at the origin still had a plane written down
/// against it.
fn what_writing_the_corner_down_cost(stored: [Interval; 3]) -> f64 {
    let size = stored
        .iter()
        .map(|side| side.lo().abs().max(side.hi().abs()))
        .fold(0.0, f64::max);
    MEASURING_SLACK * Plane::SIGNED_DISTANCE_ROUNDOFF * size.max(1.0)
}

/// Whether two boxes have any point in common, sides included, once the second
/// has been grown by `room` on every side.
fn boxes_overlap(one: [Interval; 3], other: [Interval; 3], room: f64) -> bool {
    (0..3).all(|axis| {
        one[axis].lo() <= other[axis].hi() + room && other[axis].lo() - room <= one[axis].hi()
    })
}

/// The place a provenance record names a set of surfaces by: where the first of
/// them sits.
fn middle_of_the_places(around: &[Surface]) -> PredicatePoint {
    PredicatePoint::from(match around[0] {
        Surface::Plane(flat) => flat.origin().coordinates(),
        Surface::Sphere(ball) => ball.centre().coordinates(),
        Surface::Cylinder(pipe) => pipe.axis().origin().coordinates(),
        Surface::Cone(cone) => cone.apex().coordinates(),
        Surface::Torus(ring) => ring.centre().coordinates(),
        other => panic!("a surface nothing here can name: {other:?}"),
    })
}

/// One face of a solid in world coordinates, with its seams left out.
fn bake_one_face(
    model: &Model,
    solid: SolidView<'_>,
    face: FaceView<'_>,
    surfaces: &BTreeMap<FaceSlot, PlacedSurface>,
    corners: &BTreeMap<VertexSlot, BakedCorner>,
) -> BakedFace {
    let key = face.key();
    let bounds = Bounds::round_a_face(solid, face);
    let mine = key.slot_in(model.id());

    let wires = model
        .face_at(mine)
        .wires()
        .map(|wire| {
            wire.coedges()
                .filter_map(|coedge| {
                    let across = face_across(model, mine, coedge.edge().key())?;
                    Some(BakedEdge {
                        keys: core::iter::once(coedge.edge().key()).collect(),
                        start: corners[&coedge.start().key().slot_in(model.id())].clone(),
                        end: corners[&coedge.end().key().slot_in(model.id())].clone(),
                        across: surfaces[&across].placed,
                    })
                })
                .collect()
        })
        .collect();

    BakedFace {
        keys: vec![key],
        surface: face.surface(),
        orientation: face.orientation(),
        wires,
        cuts: Vec::new(),
        bounds,
    }
}

/// The face on the other side of one edge of this face, or `None` when this same
/// face is on both sides — a **seam**, which is left out of the arrangement
/// altogether (see this module's header).
///
/// # Panics
///
/// If an edge of the face has more than one face on the other side. A closed
/// solid uses every edge exactly twice, so reaching this means the model was
/// already broken.
fn face_across(model: &Model, mine: FaceSlot, edge: EdgeKey) -> Option<FaceSlot> {
    let across: Vec<FaceSlot> = model
        .edge_at(edge.slot_in(model.id()))
        .neighbouring_faces()
        .map(|other| other.slot_in(model.id()))
        .filter(|&other| other != mine)
        .collect();
    assert!(
        across.len() <= 1,
        "an edge of a closed solid has at most one face on the other side, this one has {}",
        across.len()
    );
    across.first().copied()
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Axis, Isometry, Plane, Point, Scale, Sphere, Surface, Vector};
    use rcad_predicates::{Interval, Point as PredicatePoint, Policy, ProvenanceLog};

    use super::{Bounds, bake, derive_a_corner};
    use crate::VertexKey;
    use crate::{Error, Model, SolidKey};

    fn a_cube_of_side_two() -> (Model, SolidKey) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cube = model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side two");
        (model, cube)
    }

    fn baked(model: &Model, solids: &[SolidKey]) -> Vec<super::BakedSolid> {
        bake(
            model,
            solids,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("it bakes")
    }

    /// Where the model says its corners are, with the solid's placement applied
    /// — which is what the baked surfaces have been moved by too.
    fn corners_as_stored(model: &Model, solid: SolidKey) -> Vec<(VertexKey, [f64; 3])> {
        let mut corners = Vec::new();
        for face in model.solid(solid).expect("this model made the key").faces() {
            for wire in face.wires() {
                for coedge in wire.coedges() {
                    corners.push((coedge.start().key(), coedge.start().point().coordinates()));
                }
            }
        }
        corners
    }

    fn stored(corners: &[(VertexKey, [f64; 3])], vertex: VertexKey) -> [f64; 3] {
        corners
            .iter()
            .find(|(key, _)| *key == vertex)
            .expect("every baked corner is a corner of the solid")
            .1
    }

    /// Where a solid's own placement puts one of its corners.
    ///
    /// `Model::vertex` hands back the corner in the model's frame and not in the
    /// solid's, and a placed solid is the whole point of this test.
    fn where_the_solid_puts(model: &Model, solid: SolidKey, vertex: VertexKey) -> [f64; 3] {
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .flat_map(|face| face.wires().collect::<Vec<_>>())
            .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
            .find(|coedge| coedge.start().key() == vertex)
            .expect("a corner of the solid")
            .start()
            .point()
            .coordinates()
    }

    fn a_turn() -> Isometry {
        let axis = Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length");
        Isometry::rotation(&axis, 0.7).expect("a rotation")
    }

    /// Every plane of a lattice-aligned cube is an axis plane, so the meet of
    /// three of them divides by one and the corner it solves for is an `f64`
    /// exactly.
    #[test]
    fn re_deriving_the_corners_of_a_lattice_cube_gives_back_the_corners_it_started_with() {
        let (model, cube) = a_cube_of_side_two();
        let log = ProvenanceLog::new();
        let corners = corners_as_stored(&model, cube);
        let baked = baked(&model, &[cube]);

        for face in &baked[0].faces {
            for wire in &face.wires {
                for edge in wire {
                    assert_eq!(
                        edge.start.point.exact_coordinates(),
                        Some(stored(&corners, edge.start.vertex)),
                        "an axis-aligned corner is an f64 and comes back as one"
                    );
                    assert_eq!(edge.start.point.uncertainty(), 0.0);
                }
            }
        }
        assert!(
            log.is_empty(),
            "baking refuses only on proof, so it never records anything"
        );
        assert!(Policy::certified().wobble_is_acceptable(log.worst_wobble(100.0)));
    }

    /// A turned cube's planes have irrational normals, so its corners are no
    /// longer `f64`. The box has to hold the stored corner all the same — the
    /// stored corner is the true one moved and rounded, and the box encloses the
    /// true one.
    ///
    /// A box against a bare point, not a box against a box: `corners_as_stored`
    /// goes through `VertexView::point()`, which hands back three `f64`s and no
    /// enclosure of its own. So containment is exact and is asserted with no
    /// slack. All 72 checks — the eight corners, each reached from three faces,
    /// times three axes — land strictly inside, the tightest by 1.665e-16, about
    /// six ulps at that coordinate's size.
    ///
    /// The only thing here that is not the same on every machine is the `sin`
    /// and `cos` of the turn, which come from the platform's libm and are not
    /// correctly rounded. Everything after them is `+ - * /` and `sqrt`, which
    /// IEEE-754 pins to the bit, and Rust does not fold a multiply and an add
    /// into an `fma` by itself. An ulp of disagreement there cannot reach this
    /// margin: stepping the angle one ulp at a time, containment holds for 331
    /// steps either side of 0.7 before the first failure, and within ten steps
    /// the tightest margin never falls below 8.3e-17.
    ///
    /// It is a measured property of this turn and not a law — 11 of the 840
    /// angles swept do fail — so if this ever goes red, the assert names the
    /// coordinate and the answer is a fresh measurement, not a bigger constant.
    /// It asked for `± 1e-12` until 2026-09-09; that slack absorbed nothing, and
    /// §75.6 read it as evidence that two certified boxes can fail to overlap on
    /// inputs this kernel accepts, then declined a correct change on the
    /// strength of it. §78 is what that cost.
    #[test]
    fn re_deriving_the_corners_of_a_turned_cube_gives_boxes_that_hold_them() {
        let (mut model, cube) = a_cube_of_side_two();
        model
            .place_solid(cube, &a_turn())
            .expect("this model made the key");
        let corners = corners_as_stored(&model, cube);
        let baked = baked(&model, &[cube]);

        let mut anything_was_built = false;
        for face in &baked[0].faces {
            for wire in &face.wires {
                for edge in wire {
                    let stored = stored(&corners, edge.start.vertex);
                    let box_of_it = edge.start.point.enclosure();
                    for axis in 0..3 {
                        let (lo, hi) = (box_of_it[axis].lo(), box_of_it[axis].hi());
                        assert!(
                            lo <= stored[axis] && stored[axis] <= hi,
                            "the box {lo}..{hi} does not hold the stored {}",
                            stored[axis]
                        );
                    }
                    if edge.start.point.uncertainty() > 0.0 {
                        anything_was_built = true;
                    }
                }
            }
        }
        assert!(
            anything_was_built,
            "a turned cube has corners that are not f64, and they must say so"
        );
    }

    /// Representation wobble is not a refusal (§17.2): a certified policy bakes
    /// a turned cube without a word in the log, because nothing here failed to
    /// be proved.
    #[test]
    fn a_certified_policy_bakes_a_turned_cube_without_recording_anything() {
        let (mut model, cube) = a_cube_of_side_two();
        model
            .place_solid(cube, &a_turn())
            .expect("this model made the key");
        let log = ProvenanceLog::new();

        baked(&model, &[cube]);

        assert!(log.is_empty());
        assert_eq!(log.worst_wobble(100.0), 0.0);
    }

    #[test]
    fn a_box_round_a_cube_holds_the_cube_and_misses_one_beside_it() {
        let (model, cube) = a_cube_of_side_two();
        let baked = baked(&model, &[cube]);
        let far_away = Bounds {
            low: [10.0, 10.0, 10.0],
            high: [12.0, 12.0, 12.0],
        };
        assert!(baked[0].bounds.overlap(baked[0].bounds));
        assert!(!baked[0].bounds.overlap(far_away));
    }

    /// §31.3 says a seam is an ordinary boundary edge whose second surface is
    /// read out of its own run. It is not one, and this is why.
    ///
    /// A seam is used **twice by one face**, so the two uses run along one arc
    /// of one crossing in opposite directions. Taking it as boundary hands the
    /// arrangement one stretch of one curve claimed by two sides of one wire at
    /// once, which no winding count can read. What a seam is, is where the
    /// surface's own two coordinates were cut open — not where the face stops —
    /// so it is left out, and a bar's wall comes back bounded by its two cap
    /// circles and a ball's skin by nothing at all.
    #[test]
    fn seams_are_left_out() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let bar = model
            .add_round_bar(3.0, 5.0, &Isometry::identity())
            .expect("a bar");
        let ball = model
            .add_ball(
                2.0,
                &Isometry::translation(Vector::new(20.0, 0.0, 0.0)).expect("a shift"),
            )
            .expect("a ball");

        let baked = baked(&model, &[bar, ball]);

        let wall = baked[0]
            .faces
            .iter()
            .find(|face| matches!(face.surface, Surface::Cylinder(_)))
            .expect("the bar has a wall");
        assert_eq!(wall.wires.len(), 1);
        assert_eq!(
            wall.wires[0].len(),
            2,
            "the wall's wire is two cap circles and a seam walked twice, and the seam is not \
             boundary"
        );
        for side in &wall.wires[0] {
            assert!(matches!(side.across, Surface::Plane(_)), "a cap");
        }

        let skin = &baked[1].faces[0];
        assert!(matches!(skin.surface, Surface::Sphere(_)));
        assert_eq!(skin.wires.len(), 1);
        assert!(
            skin.wires[0].is_empty(),
            "a ball's one wire is its seam twice, so nothing trims it"
        );
    }

    /// A corner of a round bar has two surfaces round it and the whole cap
    /// circle satisfies both, so there is nothing to solve (§31.3). It is read
    /// out of the model and moved, and moving it makes it a box (§34.6).
    #[test]
    fn a_corner_with_two_surfaces_round_it_is_read_out_of_the_model_and_moved() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let bar = model
            .add_round_bar(3.0, 5.0, &Isometry::identity())
            .expect("a bar");

        let standing_still = baked(&model, &[bar]);
        for face in &standing_still[0].faces {
            for side in face.wires.iter().flatten() {
                assert_eq!(
                    side.start.point.exact_coordinates(),
                    Some(
                        model
                            .vertex(side.start.vertex)
                            .expect("this model made the key")
                            .point()
                            .coordinates()
                    ),
                    "a bar that has not been moved has corners that are f64 exactly"
                );
                assert_eq!(side.start.around.len(), 2);
            }
        }

        model
            .place_solid(bar, &a_turn())
            .expect("this model made the key");
        let turned = baked(&model, &[bar]);
        let mut anything_was_built = false;
        for face in &turned[0].faces {
            for side in face.wires.iter().flatten() {
                let stored = where_the_solid_puts(&model, bar, side.start.vertex);
                let box_of_it = side.start.point.enclosure();
                for axis in 0..3 {
                    assert!(
                        box_of_it[axis].lo() <= stored[axis]
                            && stored[axis] <= box_of_it[axis].hi(),
                        "the box round a moved corner holds where the model says it went"
                    );
                }
                if side.start.point.uncertainty() > 0.0 {
                    anything_was_built = true;
                }
            }
        }
        assert!(anything_was_built, "a turned bar's corners are boxes");
    }

    /// Two planes crossing in the z axis and a ball of radius one round the
    /// origin meet at **both poles**, and nothing about the three surfaces says
    /// which pole a corner is. The model does, and this is the pair of runs that
    /// shows it is really read: the same three surfaces, two stored corners, two
    /// different answers.
    ///
    /// Everything here is a whole number, so the oracle is exact: the answer is
    /// `(0, 0, ±1)` bit for bit and not near it. Before this consulted the
    /// model, both runs came back `Error::Undecidable` — the one triple has two
    /// places, and `[only]` passed over every triple that had.
    fn three_surfaces_that_meet_at_both_poles() -> (Model, VertexKey, Vec<Surface>) {
        let (model, cube) = a_cube_of_side_two();
        let vertex = corners_as_stored(&model, cube)[0].0;
        let cutting = |normal| {
            Surface::Plane(
                Plane::from_point_and_normal(Point::origin(), normal)
                    .expect("an axis direction is a unit normal"),
            )
        };
        let around = vec![
            cutting(Vector::new(1.0, 0.0, 0.0)),
            cutting(Vector::new(0.0, 1.0, 0.0)),
            Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a ball of radius one")),
        ];
        (model, vertex, around)
    }

    #[test]
    fn a_corner_that_three_surfaces_place_at_two_poles_is_the_pole_the_model_stored() {
        let (_model, vertex, around) = three_surfaces_that_meet_at_both_poles();
        let mut log = ProvenanceLog::new();

        let at_each_pole = [1.0, -1.0].map(|up| {
            derive_a_corner(
                vertex,
                PredicatePoint::from([0.0, 0.0, up]),
                around.clone(),
                &Policy::certified(),
                &mut log,
            )
            .expect("the model says which pole this corner is")
        });

        assert_eq!(
            at_each_pole[0].point.exact_coordinates(),
            Some([0.0, 0.0, 1.0])
        );
        assert_eq!(
            at_each_pole[1].point.exact_coordinates(),
            Some([0.0, 0.0, -1.0])
        );
        assert!(
            log.is_empty(),
            "picking between two places proves a miss and never fudges one"
        );
    }

    /// The other half of the sentence: a corner the model cannot separate is
    /// passed over rather than guessed at.
    ///
    /// A corner moved by a big placement comes back as a box, and a box wide
    /// enough to hold both poles is two answers to one question. The one triple
    /// there is says nothing, so nothing places this corner and the operation
    /// has nothing to return.
    #[test]
    fn a_corner_whose_box_holds_both_poles_places_nothing() {
        let (_model, vertex, around) = three_surfaces_that_meet_at_both_poles();
        let mut log = ProvenanceLog::new();
        let both_poles_at_once = PredicatePoint::Constructed(
            [0; 3].map(|_| Interval::new(-2.0, 2.0).expect("a box round both poles")),
        );

        let refused = derive_a_corner(
            vertex,
            both_poles_at_once,
            around,
            &Policy::certified(),
            &mut log,
        );

        assert!(
            matches!(refused, Err(Error::Undecidable { .. })),
            "two places both holding the stored corner is not an answer, it was {refused:?}"
        );
    }
}
