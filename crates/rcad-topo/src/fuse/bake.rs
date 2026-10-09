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
//! **Read and moved, and not asked about.** With nothing to solve, the point
//! handed on is the stored corner whatever its surfaces say, so asking them
//! could only refuse. And the stored corner is the point `Model::validate` asks
//! these same surfaces about, with the room `validate` grants.
//! [`derive_a_corner`] asks a solved corner with that room too, plus the
//! corner's own rounding (§368), so asking here the same way would refuse only
//! a solid `validate` refuses: §286.1's answer (c), `validate`'s own question
//! asked again at the door, and not built for §286.1's reason. When §286 was
//! decided, [`derive_a_corner`] asked with the rounding alone, and asking here
//! that way refuses a block `validate` accepts and nothing else in the
//! workspace. **It is not a certificate either.**
//! This used to say `check_curved_face` had certified the corner
//! [`Decision::Zero`] on its surface: `validate` asks for "not provably off,
//! within its room", 305 corners the workspace bakes are proved off a surface
//! until they are grown, and nothing runs `validate` before a fuse bakes
//! (§75.6).
//!
//! The moved corner becomes a box of its own, because
//! `Isometry::apply_to_point` rounds and a moved corner handed on as an exact
//! triple would certify a confident sign about a rounding (§34.6).
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
//! **An edge's [`Picker`]s are carried, moved into place.** They are the only
//! thing an edge stores that says which of the two arcs between its ends it is,
//! and an edge whose ends are two nodes of one closed crossing needs exactly
//! that to be cut into the right arcs — see [`BakedEdge::picked_by`].

use std::collections::{BTreeMap, BTreeSet};

use rcad_geom::{
    EdgeRun, Isometry, MeetOfThree, Orientation, Picker, Plane, Point, Surface,
    WHERE_THREE_SURFACES_MEET, meet_of_three_surfaces, where_a_corner_lands_after_moving,
    which_two_surfaces_a_run_is_between,
};
use rcad_predicates::{
    Decision, Interval, Measures, Point as PredicatePoint, Policy, ProvenanceLog,
};
use smallvec::SmallVec;

use crate::key::{FaceSlot, VertexSlot};
use crate::validate::{
    CornerRoom, MEASURING_SLACK, a_closed_crossing_is_walked_the_way_it_is_stored, gradient_of,
    side_of_surface,
};
use crate::view::CoedgeView;
use crate::{EdgeKey, FaceKey, FaceView, Invalid, Model, Result, SolidKey, SolidView, VertexKey};

use super::assemble::{InputFace, joined_faces_of_an_input, one_surface};
use super::{Sameness, as_exact_as_it_is, gave_up, middle_of, name_of_surface_bits};

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
/// around a corner could be shown to meet at a point. And what reading which
/// way a closed edge is walked refuses
/// ([`the_side_a_closed_edge_is_walked_for`]):
/// [`Error::CannotTellWhichWayAClosedEdgeRuns`](crate::Error::CannotTellWhichWayAClosedEdgeRuns),
/// [`Invalid::ClosedEdgeBetweenTwoFlatFaces`], and
/// [`Error::ACrossingThisCannotBeOrderedAlong`](crate::Error::ACrossingThisCannotBeOrderedAlong)
/// for a kind of run this crate has never seen.
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
    /// The wires that trim it, each walked with the face on its left about the
    /// face's own surface, **with its seams left out** — see this module's
    /// header.
    ///
    /// **That is the way a face used forwards stores its wire, and not the way
    /// a face used reversed does.** `fuse/assemble.rs` stores a reversed curved
    /// face's wire the way the solid uses it, so that every edge's two coedges
    /// run opposite ways in the arena, and [`bake_one_face`] turns it round
    /// again, as `WireView::coedges` does. The arrangement counts the face on
    /// the left of its boundary about `∇S`, and handed the stored walk it took
    /// the rest of the surface for the face (`docs/decisions.md` §331.6).
    ///
    /// The first is the face's own first wire, which [`FaceData`] calls the
    /// outer one by fiat. On a closed surface that is an order the face stores
    /// and not a fact about the geometry (§31.3), so nothing below reads it as
    /// one.
    ///
    /// [`FaceData`]: crate::model::FaceData
    pub(crate) wires: Vec<Vec<BakedEdge>>,
    /// The second surface of each crossing cutting this face, and each surface
    /// touching it at one point, filled in by the gathering step. The first is
    /// always the face's own.
    pub(crate) cuts: Vec<Surface>,
    /// A box round the face.
    pub(crate) bounds: Bounds,
    /// Sides of planes the whole face is proved to lie in, closed: a plane
    /// may hold points of its boundary but none of its inside. Empty for
    /// every face but a pipe's or a ring's, one input face with a closed edge
    /// across a plane ([`the_sides_of_planes_it_lies_in`]).
    pub(crate) lies_in: SmallVec<[ASideOfAPlane; 2]>,
    /// The cuts every face carrying which meets this one on its boundary
    /// alone, filled in by the gathering step. A crossing with one of these
    /// crosses no boundary of either solid inside the face, and the
    /// arrangement leaves it out where that splits no edge
    /// (`docs/decisions.md` §422).
    pub(crate) cuts_that_reach_only_its_boundary: Vec<Surface>,
}

/// One side of a plane: behind it, its negative side, or in front.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ASideOfAPlane {
    pub(crate) plane: Plane,
    pub(crate) behind: bool,
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
    /// What picks this edge out of the rest of the curve it runs along, moved
    /// into place with the face. Empty for an edge whose run carries none.
    ///
    /// Two surfaces and two ends do not say which way round a closed curve an
    /// edge goes: a half ball's skin is bound by two edges between the same two
    /// poles on one great circle, and only these say which half is which.
    pub(crate) picked_by: SmallVec<[Picker; 1]>,
    /// For an edge that runs right round a closed curve, whether the face lies
    /// on the negative side of `across`, as the way its wire walks the edge
    /// says.
    ///
    /// **The model records this where the corners may not.** A face whose every
    /// corner is on `across` has no corner to read its side off: a half ball
    /// hanging under a block's floor has one corner, on the floor, and both
    /// halves of the ball have that same wire. Which way the wire walks its
    /// circle is what tells them apart, and the rule that turns that into a side
    /// is the one `Model::validate` checks every closed edge against,
    /// [`a_closed_crossing_is_walked_the_way_it_is_stored`], read backwards as
    /// `holds.rs` reads it for a ring's face (`docs/decisions.md` §340.5).
    ///
    /// `None` for an edge with two corners, and for a closed edge the rule says
    /// nothing about: one whose run names some third surface rather than the
    /// face across, and one along which the two surfaces touch rather than
    /// cross.
    pub(crate) inside_across: Option<bool>,
    /// The cuts the face on the other side of this edge leaves out as meeting
    /// it on its boundary alone ([`BakedFace::cuts_that_reach_only_its_boundary`]),
    /// filled in by the gathering step. A crossing that ends on this edge
    /// splits it on that side only if it is in here, so it may be left out on
    /// this side too.
    pub(crate) its_neighbour_leaves_out: Vec<Surface>,
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
    /// Every face surface that meets here, in a fixed order, then the surface
    /// each edge left out here ran along when it is none of those
    /// ([`what_each_edge_left_out_ran_along`]). Which crossings run through a
    /// corner is then a question about the model rather than about arithmetic,
    /// which is what keeps the arrangement from having to prove a corner is on
    /// a curve it is on by construction.
    ///
    /// **Three or more, and every one of them was asked about the corner.**
    /// [`derive_a_corner`] solves such a corner from one triple and then asks
    /// each surface here about it, grown by the room `Model::validate_solid`
    /// grants a corner off that surface plus [`what_writing_the_corner_down_cost`],
    /// and a surface that proves it off is a refusal. Fewer than three, and the
    /// corner was read out of the model and nothing here asked (§286).
    /// `a_corner_is_this_place` in the arrangement leans on the first half:
    /// a meet of three planes from this list is this corner while it lies
    /// within [`BakedCorner::room_its_planes_were_asked_with`] of it (§356,
    /// §395.8).
    pub(crate) around: Vec<Surface>,
    /// How far the corner was grown before each plane in `around` was asked
    /// about it: the room `Model::validate_solid` grants a corner off a plane
    /// plus [`what_writing_the_corner_down_cost`], the sum [`derive_a_corner`]
    /// asks with. `None` for a corner read out of the model, which nothing
    /// asked.
    ///
    /// **Kept so the arrangement reads the room `bake` used, and does not work
    /// it out again.** A meet of three planes from `around` is this corner as
    /// long as every point of it lies within this room of the corner, and no
    /// further: three planes that share a line in the file meet wherever
    /// rounding puts them on it, 326 units off in a KiCad relay (§395.7).
    pub(crate) room_its_planes_were_asked_with: Option<f64>,
    /// For a corner read out of the model on a pipe's wall or a cone's, the
    /// plane through the axis and the corner, moved into place — the plane a
    /// round bar's or a frustum's seam lies in — when the stored corner is
    /// exactly on it as stored. `None` for every other corner.
    ///
    /// **Kept apart from `around`, and never asked.** Every surface in
    /// `around` meets here or was asked about the corner, and the arrangement
    /// reads three planes from it as planes the corner was solved among; this
    /// one is neither. [`BakedCorner::is_on_by_construction`] is the one
    /// reader, and says why it may be taken as through the corner.
    pub(crate) seam_plane: Option<Plane>,
}

impl BakedCorner {
    /// Whether `surface` runs through this corner by construction, so that
    /// nothing has to ask it: one of the surfaces round it, however either is
    /// written ([`one_surface`]), or its [`BakedCorner::seam_plane`] either way
    /// round.
    ///
    /// **Another spelling of a surface round the corner is that surface**, so
    /// it runs through the corner as surely. A pipe with its axis turned round,
    /// or its origin slid along the axis where that folds exactly, is the same
    /// point set ([`name_of_the_surface_either_way_round`]). Two bars on one
    /// pipe, the upper one's axis turned round, and a box cutting a window in
    /// the lower one were refused at the window's corners: each was asked of
    /// the upper pipe as a box solved from the lower pipe and two planes, which
    /// straddles the pipe it is on, so "side of cylinder" could not be told
    /// (Q1094). The same pipe written the one way was taken through them
    /// unasked.
    ///
    /// [`name_of_the_surface_either_way_round`]: super::arrange::name_of_the_surface_either_way_round
    ///
    /// **The seam plane is through the corner because both were moved from
    /// one exact incidence.** A round bar turned 0.4 about `x` and cut by a
    /// block along its seam, turned the same way, is the case. As stored, the
    /// bottom seam corner `(3, 0, 0)` is on the plane `y = 0` exactly, and is
    /// one of the two points where that plane crosses the bottom circle. The
    /// turn rounds the corner into a box `8.5e-14` either way, and the block's
    /// face — the same plane moved by the same [`Plane::transformed`], and so
    /// the same bits — runs through that box, so asking which side of it the
    /// corner is cannot be answered, and the fuse refused (§294.9). But the
    /// question has an answer that exists before any rounding: the stored
    /// corner was on the stored plane, and a turn keeps every incidence. So
    /// the crossing the face makes with that plane meets the circle at this
    /// corner and not at a second point a rounding away, and that is what this
    /// says.
    ///
    /// It is sound for the reason the surfaces round a corner are taken
    /// through it unasked. Moved, the corner is no more exactly on its own cap
    /// and wall than on the seam plane; each is a moved copy of an incidence
    /// the model held. And a corner with two surfaces round it is a cached
    /// sample of the curve they share (§17.1): the cap and the wall are
    /// bounded by the whole circle, and the seam is not a boundary (this
    /// module's header), so taking the corner as the point where the plane
    /// crosses the circle, a rounding from where it was written, moves no
    /// point of the solid. The plane holds the axis, so it crosses the circle
    /// again only on the far side of the axis, a diameter away, and the corner
    /// is one of its two crossings and never both.
    ///
    /// **On a cone it is the same plane and the same argument** (§294.12). A
    /// frustum's seam is a ruling, the line through the apex and the corner,
    /// and the plane through the axis and the corner holds it, since the axis
    /// starts at the apex. It crosses the cone in that ruling and its mirror
    /// across the axis, which meet only at the apex, and a cap square to the
    /// axis in two points a diameter apart, as on a pipe; which of the
    /// crossings the corner is on, the crossings' own pickers say. The turned
    /// frustum cut along its seam refused `side of plane` at its bottom seam
    /// corner until this held for cones too.
    ///
    /// **Bit for bit, or not at all.** A plane an ulp from the seam plane is
    /// a different plane, crossing the circle an ulp from the corner, and that
    /// is still asked: the bar turned and then shifted meets a block face whose
    /// offset is one ulp off the seam plane's, and refuses as it did.
    pub(crate) fn is_on_by_construction(&self, surface: &Surface) -> bool {
        self.around.iter().any(|round| one_surface(round, surface))
            || self.seam_plane.is_some_and(|seam| {
                surface
                    .as_plane()
                    .is_some_and(|flat| *flat == seam || *flat == seam.reversed())
            })
    }
}

/// A box round some geometry, in world coordinates.
///
/// Round a face, it is the box round the face as the model stores it
/// ([`Bounds::round_a_face`]), or round its corners as the joiner derived them,
/// **grown on every side by [`what_writing_the_corner_down_cost`]** at the
/// box's farthest corner. The fuse meets a face where the baked surfaces cross,
/// and a corner written down is that place rounded to `f64`, so the box round
/// the corners alone can stop a rounding short of the face. Ungrown, the top
/// face of a block leaned 2 under a frustum stopped at `x = 1.341640786499874`,
/// its corners as stored, while its ridge met the cone at `x` from
/// `1.341640786499873` to `1.3416407864998736`; the cut through those two
/// meets was dropped as missing the face, and thirty fuses in three families
/// refused `ResultIsNotManifold` for it (`docs/decisions.md` §340.10).
///
/// Still plain `f64`, read off stored geometry (§35.4), and not grown by the
/// room `Model::validate_solid` grants a corner off its plane, so it is the
/// rounding of the corners it allows for and not every way a face can reach
/// past them.
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

    /// Whether the box holds no part of `point`'s own box.
    ///
    /// Round a face, a point whose whole enclosure is clear of the box is taken
    /// as off the face, and `arrange.rs` drops a cut whose every node is. That
    /// is as sure as the box is, and [`Bounds`] says what it allows for.
    pub(crate) fn provably_misses(self, point: PredicatePoint) -> bool {
        let box_of_it = point.enclosure();
        (0..3).any(|axis| {
            box_of_it[axis].lo() > self.high[axis] || box_of_it[axis].hi() < self.low[axis]
        })
    }

    /// The whole box, as a point that may be anywhere in it, so that a
    /// surface asked about it answers for every point of the box at once.
    /// `None` for a box with nothing in it, which a face with no box to
    /// measure has.
    ///
    /// Round a face, a surface that proves this on one side of it shares no
    /// point with the face, as far as [`Bounds`] holds the face, and
    /// `arrange.rs` leaves out a cut whose surface does.
    pub(crate) fn as_a_point_anywhere_inside(self) -> Option<PredicatePoint> {
        let sides: [Option<Interval>; 3] =
            core::array::from_fn(|axis| Interval::new(self.low[axis], self.high[axis]).ok());
        Some(PredicatePoint::Constructed([
            sides[0]?, sides[1]?, sides[2]?,
        ]))
    }

    fn nothing() -> Self {
        Self {
            low: [f64::INFINITY; 3],
            high: [f64::NEG_INFINITY; 3],
        }
    }

    /// A box round one face of one solid, curved or flat, grown by what writing
    /// a corner down costs.
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
            }
            .grown_by_what_writing_a_corner_down_costs(),
        }
    }

    /// The box grown on every side by [`what_writing_the_corner_down_cost`] at
    /// its farthest corner, which no corner of the face is farther out than.
    ///
    /// # Panics
    ///
    /// If the box is empty, which a box round a face with a corner never is.
    fn grown_by_what_writing_a_corner_down_costs(self) -> Self {
        let room = what_writing_the_corner_down_cost(core::array::from_fn(|axis| {
            Interval::new(self.low[axis], self.high[axis])
                .expect("a box round a face's corners holds at least one point")
        }));
        Self {
            low: self.low.map(|side| side - room),
            high: self.high.map(|side| side + room),
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
/// really one region first.
///
/// A solid built by this fuse never has such a pair on a plane (§17.4's
/// amendment of 2026-08-30), and one built by `ProfileBuilder` from three
/// profile points in a row does. Two faces on one surface with an edge between
/// them carry no crossing for that edge to run along, so the arrangement would
/// be handed a boundary edge with nowhere to be, and it refuses one by name
/// ([`Error::AFaceMeetsItsNeighbourOnItsOwnSurface`](crate::Error::AFaceMeetsItsNeighbourOnItsOwnSurface)).
///
/// **A wholly flat solid is joined by the joiner**, which re-derives the
/// planes and wires of the joined faces and drops every corner a lump of faces
/// runs straight through, so that each corner left has three planes round it.
///
/// **Any other solid is joined here, at the bake, and only on a ball, a pipe,
/// a cone or a ring** ([`faces_baked_as_one`]). A curved solid can carry two
/// faces on one ball, pipe or cone: a caller builds one, and a union whose
/// pieces the joiner leaves apart writes one (rows Q575 and Q898,
/// `docs/decisions.md` §231.5, §325.5). Those faces are baked as one face, and
/// an edge between two of them is left out the way a seam is. Nothing has to be
/// re-derived, because a corner with fewer than three surfaces round it is read
/// out of the model rather than solved (§31.3), and every corner left keeps the
/// surfaces it had — and gains the one each edge left out at it ran along
/// ([`what_each_edge_left_out_ran_along`]).
fn bake_one_solid(
    model: &Model,
    key: SolidKey,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<BakedSolid> {
    let slot = key.slot_in(model.id());
    let solid = model.solid_at(slot);
    let validate_allows = model.room_a_corner_of_this_solid_gets(slot);
    if solid.is_wholly_flat()
        && let Some(joined) = joined_faces_of_an_input(model, solid, log)?
    {
        return bake_the_joined_faces(model, key, solid, &joined, validate_allows, policy, log);
    }

    let mut surfaces = surface_of_every_face(model, solid);
    let groups = faces_baked_as_one(model, solid, &surfaces);
    spell_each_group_one_way(model, &groups, &mut surfaces);
    let left_out = what_each_edge_left_out_ran_along(model, &groups);
    let corners = corner_of_every_vertex(
        model,
        solid,
        &surfaces,
        &left_out,
        validate_allows,
        policy,
        log,
    )?;

    let mut faces = Vec::new();
    let mut bounds = Bounds::nothing();
    for members in &groups {
        let baked = bake_one_face(model, solid, members, &surfaces, &corners)?;
        bounds.take_in(baked.bounds);
        faces.push(baked);
    }
    Ok(BakedSolid { key, faces, bounds })
}

/// The faces of a solid, in the groups that are baked as one face each, in the
/// order the model stores the first face of each group.
///
/// Two faces are in one group when an edge runs between them and they are one
/// ball, pipe, cone or ring used one way round
/// ([`one_curved_surface_used_one_way`]).
/// Then the edge is not where the solid's skin stops but only where two faces
/// of it were written apart, which is what a seam is too. Every other face is a
/// group of one.
fn faces_baked_as_one<'model>(
    model: &'model Model,
    solid: SolidView<'model>,
    surfaces: &BTreeMap<FaceSlot, PlacedSurface>,
) -> Vec<Vec<FaceView<'model>>> {
    let faces: Vec<FaceView<'model>> = solid.faces().collect();
    let position: BTreeMap<FaceSlot, usize> = faces
        .iter()
        .enumerate()
        .map(|(at, face)| (face.key().slot_in(model.id()), at))
        .collect();
    let mut one_face = Sameness::of(faces.len());
    for (at, face) in faces.iter().enumerate() {
        let mine = face.key().slot_in(model.id());
        for wire in model.face_at(mine).wires() {
            for coedge in wire.coedges() {
                if let Some(across) = face_across(model, mine, coedge.edge().key())
                    && one_curved_surface_used_one_way(surfaces[&mine], surfaces[&across])
                {
                    one_face.join(at, position[&across]);
                }
            }
        }
    }

    let mut groups: Vec<Vec<FaceView<'model>>> = Vec::new();
    let mut group_led_by: BTreeMap<usize, usize> = BTreeMap::new();
    for (at, face) in faces.into_iter().enumerate() {
        let group = *group_led_by.entry(one_face.leader(at)).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[group].push(face);
    }
    groups
}

/// Whether two faces are one ball, one pipe, one cone or one ring, however
/// each is written, and the solid uses both the same way round.
///
/// `used` means the same thing for every spelling of these: a pipe, a cone or
/// a ring written with its axis turned round has the same gradient everywhere,
/// and a ball has one spelling. A cone's is even in how far along the axis a
/// point sits, because its surface is both nappes (`rcad_geom::Cone`), so
/// turning the axis round changes no sign. So the two faces face the same way
/// exactly when the solid uses them the same way.
///
/// **A plane is left to the joiner**, which runs on a wholly flat solid before
/// this does. Two faces on one plane of a curved solid are not joined here: a
/// plane does have two spellings that face opposite ways, and nothing here
/// builds such a solid to try it on.
///
/// **A cone was left apart until its corners were read whole** (row Q898).
/// Joined without [`what_each_edge_left_out_ran_along`], the pieces of the cone
/// `tests/pieces_of_one_curved_surface.rs` takes back in got past
/// `AFaceMeetsItsNeighbourOnItsOwnSurface` only to be refused `Undecidable` on
/// "order along a crossing", at all five of that file's walls.
fn one_curved_surface_used_one_way(one: PlacedSurface, other: PlacedSurface) -> bool {
    matches!(
        one.placed,
        Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_)
    ) && one_surface(&one.placed, &other.placed)
        && one.used == other.used
}

/// Write the one spelling each group is baked on into every face of it, as
/// [`PlacedSurface::baked_as`]: the lowest of its faces' spellings by
/// [`name_of_surface_bits`], which depends on the faces and not on the order
/// they are stored in.
///
/// **Every surface the fuse is handed is then one spelling**: the group's own,
/// the `across` of a side of a face next to it, and a corner's `around`. A
/// closed side is walked one way or the other as its pair of surfaces says
/// (§336), and turning a pipe's axis round turns that way round, so a cap
/// reading its circle off the band's own spelling while the wall reads it off
/// the group's walked it the same way as the wall did, and the answer came
/// back `CoedgesRunTheSameWay`. And a corner where two spellings of one surface
/// meet is two surfaces round it, not three.
fn spell_each_group_one_way(
    model: &Model,
    groups: &[Vec<FaceView<'_>>],
    surfaces: &mut BTreeMap<FaceSlot, PlacedSurface>,
) {
    for members in groups {
        let slots: Vec<FaceSlot> = members
            .iter()
            .map(|face| face.key().slot_in(model.id()))
            .collect();
        let spelling = slots
            .iter()
            .map(|slot| surfaces[slot].placed)
            .min_by_key(name_of_surface_bits)
            .expect("a group of faces has a face in it");
        for slot in &slots {
            surfaces
                .get_mut(slot)
                .expect("every face of the solid has its surface")
                .baked_as = spelling;
        }
    }
}

/// The surface each edge between two faces of one group ran along, beside the
/// group's own, written down against the corners at both its ends.
///
/// Such an edge is left out of the face the group is baked as
/// ([`bake_one_face`]), and with it the one place that said its corners are on
/// that surface. The corner at the end of a cut a fuse ran down a cone to its
/// rim has the rim's plane and the cone round it as faces, and the cut's plane
/// only as the run of the edge left out. Yet the cut's plane can still be a
/// crossing of the baked face, where another stretch of the same cut is kept
/// as boundary, and the arrangement then meets it at the rim as a solved point
/// a few ulps from the corner as stored. Two points of one crossing that close
/// cannot be put in order, so the next fuse refused `Undecidable` on "order
/// along a crossing" (`tests/pieces_of_one_curved_surface.rs`, row Q898).
/// Knowing the surface, the corner is solved from the three, and the
/// arrangement names that meet the corner rather than a point beside it.
///
/// An edge whose run names no third surface, [`EdgeRun::Straight`] or
/// [`EdgeRun::RightRoundWhereItsFacesCross`], has none to add.
fn what_each_edge_left_out_ran_along(
    model: &Model,
    groups: &[Vec<FaceView<'_>>],
) -> BTreeMap<VertexSlot, Vec<Surface>> {
    let mut ran_along: BTreeMap<VertexSlot, Vec<Surface>> = BTreeMap::new();
    for members in groups {
        let in_the_group: BTreeSet<FaceSlot> = members
            .iter()
            .map(|face| face.key().slot_in(model.id()))
            .collect();
        for face in members {
            let mine = face.key().slot_in(model.id());
            for wire in model.face_at(mine).wires() {
                for coedge in wire.coedges() {
                    let edge = coedge.edge();
                    if !face_across(model, mine, edge.key())
                        .is_some_and(|across| in_the_group.contains(&across))
                    {
                        continue;
                    }
                    let (EdgeRun::PartOfWhereItCrosses { and, .. }
                    | EdgeRun::RightRoundWhereItCrosses { and }) = edge.stored_run()
                    else {
                        continue;
                    };
                    let and = and.transformed(&face.location());
                    for end in [coedge.start().key(), coedge.end().key()] {
                        let surfaces = ran_along.entry(end.slot_in(model.id())).or_default();
                        if !surfaces.contains(&and) {
                            surfaces.push(and);
                        }
                    }
                }
            }
        }
    }
    ran_along
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
    validate_allows: CornerRoom,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<BakedSolid> {
    let corners = corner_of_every_vertex_left(model, solid, joined, validate_allows, policy, log)?;

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
                    picked_by: SmallVec::new(),
                    inside_across: None,
                    its_neighbour_leaves_out: Vec::new(),
                });
            }
            wires.push(walked);
        }
        let round_this_face = round_this_face.grown_by_what_writing_a_corner_down_costs();
        bounds.take_in(round_this_face);
        faces.push(BakedFace {
            keys: face.faces.clone(),
            surface: Surface::Plane(face.plane),
            orientation: Orientation::Forward,
            wires,
            cuts: Vec::new(),
            bounds: round_this_face,
            lies_in: SmallVec::new(),
            cuts_that_reach_only_its_boundary: Vec::new(),
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
    validate_allows: CornerRoom,
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
            Ok((
                slot,
                derive_a_corner(vertex, stored, planes, validate_allows, policy, log)?,
            ))
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
                baked_as: face.surface(),
                movement: face.location(),
                used: face.orientation(),
            },
        );
        assert!(
            already_there.is_none(),
            "a solid uses one face twice, so it has two surfaces"
        );
    }
    surfaces
}

/// One face's surface, moved into place, the movement that put it there, and
/// which way round the solid uses the face.
///
/// The movement is what a corner read out of the model needs: the box round
/// where it lands is worked out from the *stored* corner and the *stored*
/// surfaces, and applying the movement to each of them separately is what
/// [`where_a_corner_lands_after_moving`] does. The use is what the closed-edge
/// rule needs of the face across a closed edge ([`BakedEdge::inside_across`]).
///
/// `baked_as` is `placed`, or another spelling of it when the face is baked
/// together with faces on other spellings ([`spell_each_group_one_way`]). It is
/// what the arrangement is handed; `stored` and `placed` stay the face's own,
/// for the questions asked of the face itself.
#[derive(Clone, Copy, Debug)]
struct PlacedSurface {
    stored: Surface,
    placed: Surface,
    baked_as: Surface,
    movement: Isometry,
    used: Orientation,
}

/// Every corner of the solid, placed from the surfaces of its faces and of the
/// edges left out at it ([`what_each_edge_left_out_ran_along`]).
fn corner_of_every_vertex(
    model: &Model,
    solid: SolidView<'_>,
    surfaces: &BTreeMap<FaceSlot, PlacedSurface>,
    left_out: &BTreeMap<VertexSlot, Vec<Surface>>,
    validate_allows: CornerRoom,
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
                    let also_on = left_out.get(&slot).map_or(&[][..], Vec::as_slice);
                    corners.insert(
                        slot,
                        place_a_corner(
                            model,
                            vertex,
                            slot,
                            &around,
                            also_on,
                            validate_allows,
                            policy,
                            log,
                        )?,
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
/// the model and moved. **Its surfaces are not asked about it**, and this
/// module's header says why (§286).
///
/// The surfaces round it are its faces', and then `also_on`: what an edge left
/// out at it ran along, unless that is one of its faces' surfaces however
/// written ([`what_each_edge_left_out_ran_along`]).
///
/// # Panics
///
/// If the faces round one corner are at different placements. Then the corner
/// has two places at once, and a solid like that is broken rather than awkward.
#[expect(
    clippy::too_many_arguments,
    reason = "the corner, where it is stored, what is round it, and how strictly to place it"
)]
fn place_a_corner(
    model: &Model,
    vertex: VertexKey,
    slot: VertexSlot,
    around: &[PlacedSurface],
    also_on: &[Surface],
    validate_allows: CornerRoom,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<BakedCorner> {
    let where_the_model_puts_it = where_the_model_puts_a_corner(model, slot, around)?;

    let mut distinct: Vec<Surface> = Vec::with_capacity(around.len() + also_on.len());
    for face in around {
        if !distinct.contains(&face.baked_as) {
            distinct.push(face.baked_as);
        }
    }
    for surface in also_on {
        if !distinct.iter().any(|known| one_surface(known, surface)) {
            distinct.push(*surface);
        }
    }
    if distinct.len() >= 3 {
        return derive_a_corner(
            vertex,
            where_the_model_puts_it,
            distinct,
            validate_allows,
            policy,
            log,
        );
    }
    Ok(BakedCorner {
        vertex,
        point: where_the_model_puts_it,
        around: distinct,
        seam_plane: the_seam_plane_a_corner_is_on(model, slot, around),
        room_its_planes_were_asked_with: None,
    })
}

/// The plane through the axis of a pipe or a cone round a corner read out of
/// the model, and through the stored corner, moved into place with the corner
/// ([`BakedCorner::seam_plane`]).
///
/// **Only when the stored corner is exactly on it**, asked of the stored
/// numbers. The plane is built from the corner, so it is through the corner
/// as written, but its normal is scaled to length one and that rounds unless
/// the axis runs along an axis of the frame. A plane that misses the stored
/// corner by a rounding crosses the circle a rounding away from it, which is a
/// second point and not this one.
fn the_seam_plane_a_corner_is_on(
    model: &Model,
    slot: VertexSlot,
    around: &[PlacedSurface],
) -> Option<Plane> {
    let stored = model.data().vertices[slot].point;
    around.iter().find_map(|face| {
        let axis = match face.stored {
            Surface::Cylinder(pipe) => pipe.axis(),
            Surface::Cone(cone) => cone.axis(),
            Surface::Plane(_) | Surface::Sphere(_) | Surface::Torus(_) => return None,
            // A surface kind added to `rcad-geom` after this has not been
            // shown to have a seam the plane through its axis holds.
            _ => return None,
        };
        // A corner on the axis has no plane through the two. On a pipe only
        // a corner moved off its wall in the file can be there; on a cone,
        // the apex is on the axis too, and a corner there is no seam corner.
        let seam_plane = Plane::from_point_and_normal(
            axis.origin(),
            axis.direction().cross(stored - axis.origin()),
        )
        .ok()?;
        (seam_plane.side_of_point(PredicatePoint::from(stored.coordinates())) == Decision::Zero)
            .then(|| seam_plane.transformed(&face.movement))
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
    // A solid standing where it was built hands on the very bits it stored.
    // The general road would give the same place, exact, but only after adding
    // a shift of zero, which turns a stored `-0.0` into `0.0` on the path every
    // unplaced solid takes.
    if movement == Isometry::identity() {
        return Ok(PredicatePoint::from(stored.coordinates()));
    }
    // Every surface round the corner and not two of them: a shift that rounds
    // nothing hands the corner back exact, and that is only true of a corner
    // whose every surface was carried exactly (§34.6's confident sign about a
    // rounding, otherwise).
    //
    // The width of the box is the whole of what the move cost, and the
    // `Approximation` beside it says the same number a second time. §34.9's trap
    // is that the box is for asking predicates and its middle is for storing, so
    // the one thing that must not happen is the two drifting apart — and the way
    // to make that impossible is to keep only one of them.
    let (point, _) = where_a_corner_lands_after_moving(
        stored,
        around.iter().map(|face| &face.stored),
        &movement,
    )?;
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
/// # The corner is grown before it is asked, by what `validate` allows
///
/// The box a triple solves the corner as says how well *that solve* is known.
/// It says nothing about the fourth surface, which was fitted through corners
/// that had already been rounded and so passes a few ulps from where the other
/// three cross. Asking it about the bare box is asking it to agree bit for bit
/// with a number it never saw, and it does not: a chamfered pin header whose
/// two 45° walls are written with a `1.7e-18` tilt reaches a corner three
/// planes place and the fourth reads ten ulps off, certified, so a solid this
/// kernel imports and `Model::validate` accepts was one the fuse could not
/// bake.
///
/// So before each surface is asked, the corner is grown by two things added
/// together. The first is **the room `Model::validate_solid` grants a corner
/// off that kind of surface**, `validate_allows`, read from
/// [`Model::room_a_corner_of_this_solid_gets`](crate::Model) and not copied: a
/// modelling tolerance and a rounding off a plane, a rounding alone off a
/// curved surface, both at the solid's largest coordinate. The fourth surface
/// may miss the stored corner by that much and the sweep still accepts the
/// solid, so the fuse has to take the same miss (§75.6, §368). The second is
/// [`what_writing_the_corner_down_cost`] at the corner's own size, because the
/// solid's size is read where its corners are stored or where its placement
/// puts them, whichever is smaller (§363), and the fuse meets the corner where
/// the placement puts it. A surface the grown box straddles answers
/// [`Decision::Undecidable`]. The sum a plane was asked with is kept on the
/// corner ([`BakedCorner::room_its_planes_were_asked_with`]), because the
/// arrangement's fold reads it (§395.8).
///
/// **This does not reopen §70.3.** That corner is three surfaces at a grazing
/// angle and a stored corner far from where they cross, and what stops it is
/// [`the_place_the_model_points_at`] declining to pick a place at all — a test
/// that runs first, on every corner. Its room is a modelling tolerance now,
/// `1e-9` of the solid's size at most, and §70.3's corner is `1.414` from the
/// crossing in a solid of size two
/// (`a_corner_near_three_grazing_planes_but_far_from_where_they_cross_is_refused`).
///
/// Every triple failing is not a geometry error: it is a corner nothing can
/// place, which leaves the operation with nothing to return, so it is
/// [`Error::Undecidable`](crate::Error::Undecidable) with a record naming
/// [`WHERE_THREE_SURFACES_MEET_HERE`].
fn derive_a_corner(
    vertex: VertexKey,
    where_the_model_puts_it: PredicatePoint,
    around: Vec<Surface>,
    validate_allows: CornerRoom,
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
                let widest_validate_allows = triple
                    .iter()
                    .map(|surface| validate_allows.off(surface))
                    .fold(0.0, f64::max);
                let Some(place) = the_place_the_model_points_at(
                    &places,
                    where_the_model_puts_it,
                    widest_validate_allows,
                ) else {
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

    let writing_it_down = what_writing_the_corner_down_cost(point.enclosure());
    for surface in &around {
        let asked_with_room = grown_by(point, writing_it_down + validate_allows.off(surface));
        match side_of_surface(surface, asked_with_room) {
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
        seam_plane: None,
        room_its_planes_were_asked_with: Some(writing_it_down + validate_allows.off_a_plane),
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
/// So the stored box is grown on every side by `validate_allows`, the widest
/// room `Model::validate_solid` grants the corner off any of the three
/// surfaces, plus [`what_writing_the_corner_down_cost`] at the corner's own
/// size — the same two the re-check in [`derive_a_corner`] adds, for the same
/// reasons. Off a plane that room is a **modelling tolerance** and not only a
/// rounding: `validate` lets a flat face's corner sit `1e-9` of the solid's
/// size off its plane, so a corner somebody placed that far off is one the
/// sweep accepts, and a solid the sweep accepts is one the fuse has to be able
/// to bake (§75.6). A room of the rounding alone refused a two-millimetre block
/// with a corner a nanometre off one plane (§136.8, §368).
///
/// **This never places the point.** It picks among places the surfaces already
/// agree on; the geometry comes from the meet, which is what keeps a wrong
/// cached corner from moving a right one. A corner placed within the room is
/// moved onto where its planes cross, by no more than `validate` let it sit
/// off them.
fn the_place_the_model_points_at(
    places: &[PredicatePoint],
    where_the_model_puts_it: PredicatePoint,
    validate_allows: f64,
) -> Option<PredicatePoint> {
    let stored = where_the_model_puts_it.enclosure();
    let room = what_writing_the_corner_down_cost(stored) + validate_allows;
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

/// How far a corner can read off a plane it is on purely because the plane
/// and the corner were written down in `f64`, at the corner's own size.
///
/// **In [`derive_a_corner`] this is the half of the room that follows the
/// corner**, added to the half `Model::validate_solid` grants. The solid's size
/// that half is measured at can be smaller than where the fuse meets the corner
/// (§363), and the planes the fuse asks were moved there and rounded there.
///
/// Floored at one for the same reason [`select`](super::select) floors its own
/// round-trip room: a corner at the origin still had a plane written down
/// against it.
pub(super) fn what_writing_the_corner_down_cost(stored: [Interval; 3]) -> f64 {
    let size = stored
        .iter()
        .map(|side| side.lo().abs().max(side.hi().abs()))
        .fold(0.0, f64::max);
    MEASURING_SLACK * Plane::SIGNED_DISTANCE_ROUNDOFF * size.max(1.0)
}

/// The corner with `room` added on every side, so a surface that passes through
/// the grown box answers [`Decision::Undecidable`] rather than a sign.
///
/// A box and not a distance because only a plane hands a signed length out; a
/// ball, a pipe, a cone and a ring answer with a sign and nothing else, so the
/// allowance has to be spent on the point before the question rather than on
/// the answer after it. That is the same move `not_provably_off` in
/// [`validate`](crate::validate) makes for the same reason.
pub(super) fn grown_by(point: PredicatePoint, room: f64) -> PredicatePoint {
    assert!(
        room.is_finite() && room >= 0.0,
        "an allowance read off a finite corner is finite and not negative, and this one is {room}"
    );
    PredicatePoint::Constructed(point.enclosure().map(|side| {
        Interval::new(side.lo() - room, side.hi() + room)
            .expect("growing a finite box by a finite room leaves a finite box")
    }))
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

/// One group of faces of a solid ([`faces_baked_as_one`]) in world coordinates,
/// as one face, with its seams and every edge between two of its faces left
/// out.
///
/// The wires are every wire of every face of the group, each with those edges
/// left out, so a wire can be left empty, as a ball's skin's one wire is. What
/// is left of each is walked with its own face on the left about that face's
/// surface ([`BakedFace::wires`]): the stored walk, turned round for a face used
/// reversed. The faces of a group face one way, so an edge walked with one of
/// them on its left has the group on its left.
///
/// [`BakedEdge::inside_across`] is read off the stored walk, before it is
/// turned round, because that is the walk the rule it reads,
/// [`a_closed_crossing_is_walked_the_way_it_is_stored`], is checked against by
/// `Model::validate`: the coedge's turn as stored, against the faces' outward
/// gradients.
///
/// **The face is on the one spelling [`spell_each_group_one_way`] gave the
/// group**, and so is the `across` of each side whose face across is in a
/// group. A closed side's [`BakedEdge::inside_across`] is still read off the
/// two faces' own spellings: it is a side of the surface across, and every
/// spelling of a curved surface has one inside and one outside.
///
/// # Errors
///
/// Whatever [`the_side_a_closed_edge_is_walked_for`] refuses.
///
/// # Panics
///
/// If `members` is empty.
fn bake_one_face(
    model: &Model,
    solid: SolidView<'_>,
    members: &[FaceView<'_>],
    surfaces: &BTreeMap<FaceSlot, PlacedSurface>,
    corners: &BTreeMap<VertexSlot, BakedCorner>,
) -> Result<BakedFace> {
    let in_the_group: BTreeSet<FaceSlot> = members
        .iter()
        .map(|face| face.key().slot_in(model.id()))
        .collect();

    let mut wires = Vec::new();
    let mut bounds = Bounds::nothing();
    for face in members {
        bounds.take_in(Bounds::round_a_face(solid, *face));
        let mine = face.key().slot_in(model.id());
        for wire in model.face_at(mine).wires() {
            let mut walked = Vec::new();
            for coedge in wire.coedges() {
                let Some(across) = face_across(model, mine, coedge.edge().key()) else {
                    continue;
                };
                if in_the_group.contains(&across) {
                    continue;
                }
                walked.push(BakedEdge {
                    keys: core::iter::once(coedge.edge().key()).collect(),
                    start: corners[&coedge.start().key().slot_in(model.id())].clone(),
                    end: corners[&coedge.end().key().slot_in(model.id())].clone(),
                    across: surfaces[&across].baked_as,
                    picked_by: pickers_moved_into_place(
                        coedge.edge().stored_run(),
                        &face.location(),
                    ),
                    inside_across: the_side_a_closed_edge_is_walked_for(
                        coedge,
                        [surfaces[&mine], surfaces[&across]],
                    )?,
                    its_neighbour_leaves_out: Vec::new(),
                });
            }
            if surfaces[&mine].used.is_reversed() {
                walked.reverse();
                for edge in &mut walked {
                    core::mem::swap(&mut edge.start, &mut edge.end);
                }
            }
            wires.push(walked);
        }
    }

    let first = surfaces[&members
        .first()
        .expect("a group of faces has a face in it")
        .key()
        .slot_in(model.id())];
    let mut baked = BakedFace {
        keys: members.iter().map(|face| face.key()).collect(),
        surface: first.baked_as,
        orientation: first.used,
        wires,
        cuts: Vec::new(),
        bounds,
        lies_in: SmallVec::new(),
        cuts_that_reach_only_its_boundary: Vec::new(),
    };
    baked.lies_in = the_sides_of_planes_it_lies_in(&baked);
    Ok(baked)
}

/// The side of each plane across a closed edge of a pipe's or a ring's face
/// that the whole face lies on, where that is proved, and nothing for any
/// other face.
///
/// **One input face, or nothing.** A face of the model is in one piece, and
/// so its inside, which misses each plane below, lies on one side of it. A
/// group joined from several faces need not be: a bar with a groove round it
/// has its wall in two pieces either side of the groove.
///
/// **A pipe's.** A plane that crosses a pipe in a closed curve leans to its
/// axis, so it crosses the pipe in that one ellipse and nowhere else. The
/// face's inside misses the edge, so it misses the plane, and is on the side
/// [`BakedEdge::inside_across`] says.
///
/// **A ring's**, between two planes through its axis and no other edges: each
/// plane crosses the ring in two circles, the edge and its mirror across the
/// axis, and the face lies inside both sides only where it turns less than a
/// half-turn about the axis. That is the test `holds.rs` makes of the same
/// face: each circle's corner is strictly on the kept side of the other plane.
/// Two edges on one plane leave each corner on the other's plane, and nothing
/// is kept.
///
/// A ball's or a cone's face between planes would be sound to do too, and
/// nothing needs it yet (§13).
fn the_sides_of_planes_it_lies_in(face: &BakedFace) -> SmallVec<[ASideOfAPlane; 2]> {
    if face.keys.len() != 1 {
        return SmallVec::new();
    }
    let edges: Vec<&BakedEdge> = face.wires.iter().flatten().collect();
    let sides: SmallVec<[ASideOfAPlane; 2]> = edges
        .iter()
        .filter_map(|edge| {
            Some(ASideOfAPlane {
                plane: *edge.across.as_plane()?,
                behind: edge.inside_across?,
            })
        })
        .collect();
    if let Surface::Cylinder(_) = face.surface {
        return sides;
    }
    if let Surface::Torus(ring) = face.surface {
        let ([one, other], [one_side, other_side]) = (edges.as_slice(), sides.as_slice()) else {
            return SmallVec::new();
        };
        let through_the_axis = [one_side, other_side]
            .iter()
            .all(|side| crate::holds::how_the_axis_lies_on(&side.plane, &ring) == Decision::Zero);
        let each_corner_in_front_of_the_other_plane = [(one, other_side), (other, one_side)]
            .iter()
            .all(|(edge, side)| side.plane.side_of_point(edge.start.point) == side.kept());
        if through_the_axis && each_corner_in_front_of_the_other_plane {
            return sides;
        }
    }
    SmallVec::new()
}

impl ASideOfAPlane {
    /// The sign the plane gives a point strictly on this side of it.
    pub(crate) fn kept(self) -> Decision {
        if self.behind {
            Decision::Neg
        } else {
            Decision::Pos
        }
    }

    /// The sign the plane gives a point strictly on the other side of it,
    /// which no point of a face lying on this side is.
    pub(crate) fn past_it(self) -> Decision {
        if self.behind {
            Decision::Pos
        } else {
            Decision::Neg
        }
    }
}

/// Whether a face lies on the negative side of the face across a closed edge
/// of it, as the way its wire walks that edge says; `None` for an edge that
/// does not close, and see [`BakedEdge::inside_across`] for when else.
///
/// Read where both faces are stored, for the reason
/// `Model::validate`'s own reading of this rule gives: a ball stores no axis
/// and is wound about the `z` of the frame it is read in, so read where a
/// turned solid sits it would be wound about a different line from the one
/// its edges were walked by. A movement keeps which side of a surface a face
/// is on, so the side read here is the side of `across` moved into place.
///
/// `faces` is this face and the face across, in that order.
///
/// # Errors
///
/// [`Error::ACrossingThisCannotBeOrderedAlong`](crate::Error::ACrossingThisCannotBeOrderedAlong)
/// for a kind of run this crate has never seen, and whatever
/// [`a_closed_crossing_is_walked_the_way_it_is_stored`] refuses: a closed edge
/// between two planes, and one whose direction its rule cannot tell.
fn the_side_a_closed_edge_is_walked_for(
    coedge: CoedgeView<'_>,
    [mine, across]: [PlacedSurface; 2],
) -> Result<Option<bool>> {
    let edge = coedge.edge();
    if !edge.is_closed()? {
        return Ok(None);
    }
    let names_it =
        which_two_surfaces_a_run_is_between(edge.stored_run(), [&mine.stored, &across.stored]);
    if names_it[1] != across.stored {
        return Ok(None);
    }
    let at = edge.stored_ends()[0];
    let outward = [
        gradient_of(&mine.stored, at) * mine.used.sign(),
        gradient_of(&across.stored, at) * across.used.sign(),
    ];
    let Some(forward_if_it_keeps_the_back) = a_closed_crossing_is_walked_the_way_it_is_stored(
        [&names_it[0], &names_it[1]],
        outward,
        at,
        Decision::Neg,
    )?
    else {
        return Ok(None);
    };
    let walked_forward = coedge.orientation_on_its_edge() == Orientation::Forward;
    let keeps_the_back = walked_forward == forward_if_it_keeps_the_back;
    // The back is the side of `across` as the solid uses it; a face used
    // turned round has its stored surface's negative side in front.
    Ok(Some(keeps_the_back != across.used.is_reversed()))
}

/// An edge's pickers, moved by the placement its face stands at.
///
/// A face standing where it was built is not moved at all, so that a picker
/// whose plane holds an edge's ends exactly still does.
fn pickers_moved_into_place(run: &EdgeRun, movement: &Isometry) -> SmallVec<[Picker; 1]> {
    let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run else {
        return SmallVec::new();
    };
    if *movement == Isometry::identity() {
        return picked_by.clone();
    }
    picked_by
        .iter()
        .map(|picker| picker.transformed(movement))
        .collect()
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

    use super::{Bounds, bake, derive_a_corner, what_writing_the_corner_down_cost};
    use crate::VertexKey;
    use crate::validate::{CornerRoom, MEASURING_SLACK, side_of_surface};
    use crate::{Error, Model, SolidKey};
    use rcad_predicates::Decision;

    /// **A face used reversed is baked walked the way the solid's view walks
    /// it**, with the face on the left about its own surface, and not the way it
    /// is stored.
    ///
    /// A bar less an overlapping bar is a crescent whose inner wall is used
    /// reversed, and `fuse/assemble.rs` stores that wall's wire the way the
    /// solid uses it, up the ruling at `(3, 4)`. The arrangement reads a face on
    /// the left of its boundary about `∇S`, so handed the stored walk it counted
    /// the rest of the pipe as the wall, and a block over the crescent's top
    /// half cut a piece of the pipe above `z = 10` out of it
    /// (`docs/decisions.md` §331.6).
    #[test]
    fn a_face_used_reversed_is_baked_walked_the_way_its_view_walks_it() {
        let policy = Policy::certified();
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let here = model
            .add_round_bar(5.0, 10.0, &Isometry::identity())
            .expect("a bar of radius five, ten tall");
        let there = model
            .add_round_bar(
                5.0,
                10.0,
                &Isometry::translation(Vector::new(6.0, 0.0, 0.0)).expect("a shift along `x`"),
            )
            .expect("a second bar six along `x`");
        let fused = model
            .fuse(
                &[here],
                &[there],
                crate::Rule::Difference,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("one bar less the other is a crescent");
        let [crescent] = fused.solids() else {
            panic!("a crescent is one solid: {:?}", fused.solids());
        };
        let baked = bake(&model, &[*crescent], &policy, &mut ProvenanceLog::new())
            .expect("a crescent bakes");
        let solid = model.solid(*crescent).expect("this model made the key");

        let mut used_reversed = 0;
        for (face, view) in baked[0].faces.iter().zip(solid.faces()) {
            assert_eq!(face.keys, vec![view.key()], "one baked face per face");
            let walked: Vec<[VertexKey; 2]> = face
                .wires
                .iter()
                .flatten()
                .map(|edge| [edge.start.vertex, edge.end.vertex])
                .collect();
            let viewed: Vec<[VertexKey; 2]> = view
                .wires()
                .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
                .map(|coedge| [coedge.start().key(), coedge.end().key()])
                .collect();
            assert_eq!(
                walked,
                viewed,
                "{:?}, used {:?}",
                view.surface(),
                view.orientation()
            );
            used_reversed += usize::from(view.orientation().is_reversed());
        }
        assert_eq!(
            used_reversed, 1,
            "the crescent has one face used reversed, its inner wall"
        );
    }

    fn a_cube_of_side_two() -> (Model, SolidKey) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cube = model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side two");
        (model, cube)
    }

    /// The room `Model::validate_solid` grants a corner of `solid`, which is
    /// the room the fuse grants it.
    fn what_validate_allows(model: &Model, solid: SolidKey) -> CornerRoom {
        model.room_a_corner_of_this_solid_gets(solid.slot_in(model.id()))
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

    /// A round bar's wall lies between its caps, on the side of each cap's
    /// plane the bar is on. That side is [`super::BakedEdge::inside_across`]
    /// read as the negative side of the plane across, which is how §422's
    /// rule reads it; read the other way round, the wall would lie outside
    /// both caps.
    #[test]
    fn a_round_bars_wall_lies_on_the_side_of_each_cap_the_bar_is_on() {
        for placement in [Isometry::identity(), a_turn()] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let bar = model.add_round_bar(3.0, 5.0, &placement).expect("a bar");
            let at = |height: f64| {
                PredicatePoint::from(
                    placement
                        .apply_to_point(Point::new(0.0, 0.0, height))
                        .coordinates(),
                )
            };

            let baked = baked(&model, &[bar]);

            let wall = baked[0]
                .faces
                .iter()
                .find(|face| matches!(face.surface, Surface::Cylinder(_)))
                .expect("the bar has a wall");
            assert_eq!(wall.lies_in.len(), 2, "one side of each cap");
            for side in &wall.lies_in {
                assert_eq!(
                    side.plane.side_of_point(at(2.5)),
                    side.kept(),
                    "the middle of the bar is on the side kept"
                );
            }
            for outside in [-2.5, 7.5] {
                assert!(
                    !wall
                        .lies_in
                        .iter()
                        .all(|side| side.plane.side_of_point(at(outside)) == side.kept()),
                    "a point past a cap is outside one of the sides"
                );
            }
            for cap in baked[0]
                .faces
                .iter()
                .filter(|face| face.surface.as_plane().is_some())
            {
                assert!(cap.lies_in.is_empty(), "a flat face lies in nothing here");
            }
        }
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

    /// A corner on a pipe's wall carries the plane through the axis and itself
    /// only when it is exactly on that plane as stored
    /// ([`BakedCorner::seam_plane`](super::BakedCorner)). A bar's seam corners
    /// are, on `y = 0`, and carry it moved with the bar. A corner written at
    /// `(1, 3, 5)` is not: the plane's normal `(-3, 1, 0)` scaled to length one
    /// rounds, and misses it.
    #[test]
    fn a_corner_carries_its_seam_plane_only_when_it_is_exactly_on_it() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let bar = model.add_round_bar(3.0, 5.0, &a_turn()).expect("a bar");
        let through = |normal| {
            Plane::from_point_and_normal(Point::origin(), normal).expect("a normal with length")
        };
        let the_seams = Some(through(Vector::new(0.0, 1.0, 0.0)).transformed(&a_turn()));
        let mut top = None;
        for face in &baked(&model, &[bar])[0].faces {
            for side in face.wires.iter().flatten() {
                assert_eq!(side.start.seam_plane, the_seams);
                if model
                    .vertex(side.start.vertex)
                    .expect("this model made the key")
                    .point()
                    .coordinates()[2]
                    == 5.0
                {
                    top = Some(side.start.vertex);
                }
            }
        }
        let top = top.expect("the bar has a top corner");

        let written = Point::new(1.0, 3.0, 5.0);
        assert_ne!(
            through(Vector::new(-3.0, 1.0, 0.0))
                .side_of_point(PredicatePoint::from(written.coordinates())),
            Decision::Zero,
            "the plane through the axis and the written corner misses it"
        );
        let slot = top.slot_in(model.id());
        model.data_mut().vertices[slot].point = written;
        for face in &baked(&model, &[bar])[0].faces {
            for side in face.wires.iter().flatten() {
                let expected = if side.start.vertex == top {
                    None
                } else {
                    the_seams
                };
                assert_eq!(side.start.seam_plane, expected);
            }
        }
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
    fn three_surfaces_that_meet_at_both_poles() -> (CornerRoom, VertexKey, Vec<Surface>) {
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
        (what_validate_allows(&model, cube), vertex, around)
    }

    #[test]
    fn a_corner_that_three_surfaces_place_at_two_poles_is_the_pole_the_model_stored() {
        let (validate_allows, vertex, around) = three_surfaces_that_meet_at_both_poles();
        let mut log = ProvenanceLog::new();

        let at_each_pole = [1.0, -1.0].map(|up| {
            derive_a_corner(
                vertex,
                PredicatePoint::from([0.0, 0.0, up]),
                around.clone(),
                validate_allows,
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
        let (validate_allows, vertex, around) = three_surfaces_that_meet_at_both_poles();
        let mut log = ProvenanceLog::new();
        let both_poles_at_once = PredicatePoint::Constructed(
            [0; 3].map(|_| Interval::new(-2.0, 2.0).expect("a box round both poles")),
        );

        let refused = derive_a_corner(
            vertex,
            both_poles_at_once,
            around,
            validate_allows,
            &Policy::certified(),
            &mut log,
        );

        assert!(
            matches!(refused, Err(Error::Undecidable { .. })),
            "two places both holding the stored corner is not an answer, it was {refused:?}"
        );
    }

    /// A fourth plane that misses the corner by less than writing the corner
    /// down cost is agreement, and asking it about the bare corner was not.
    ///
    /// Three planes place a corner. The fourth was fitted through *other*
    /// corners of its own face and had its normal scaled to length one on the
    /// way in, so it passes a few ulps from where the three cross. Asking it
    /// about a box of no width asks it to agree bit for bit with a number it
    /// never saw, and it does not.
    ///
    /// **The sizes are `PinHeader_1x11_P2.54mm_Vertical.step`'s own**, read off
    /// the corner that actually failed rather than invented: the part runs
    /// 20.64 mm along `y`, its 45° chamfer came off the file with a `2.45e-18`
    /// tilt in its direction cosines, and that plane read `3.553e-15` — one ulp
    /// of that `y` — from the corner three others placed. The allowance there
    /// is `2.933e-13`, so the miss is 82 times inside it and this is not a
    /// marginal call. Both pin headers in the corpus stopped round-tripping
    /// without the growth, which is what `every_kicad_file_that_imports_round_
    /// trips_to_the_same_solid` pins and this demonstrates (§16.10).
    ///
    /// **The first assertion is what stops this being a tautology.** It checks
    /// that the bare corner still reads a *certified* sign, so the test cannot
    /// go green by the fourth plane having quietly become agreeable — it fails
    /// on the unfixed code because of the last assertion, and it fails on a
    /// kernel where the miss stopped being provable because of the first.
    #[test]
    fn a_fourth_plane_a_few_ulps_off_the_corner_three_others_place_is_agreement() {
        let (model, cube) = a_cube_of_side_two();
        let vertex = corners_as_stored(&model, cube)[0].0;
        let corner = Point::new(-0.32, -20.64, -2.84);
        let exactly_the_corner = PredicatePoint::from(corner.coordinates());

        let face_through_the_corner = |normal| {
            Surface::Plane(
                Plane::from_point_and_normal(corner, normal).expect("an axis is a direction"),
            )
        };
        let a_pin_header_chamfer_misses_by = 3.552713678800501e-15;
        let tilted = Plane::from_point_and_normal(corner, Vector::new(-1.0, -2.4539e-18, -1.0))
            .expect("a 45 degree wall with a tilt is still a direction");
        let chamfer = tilted
            .moved_to(tilted.offset() + a_pin_header_chamfer_misses_by)
            .expect("a plane a few ulps along its own normal");

        assert!(
            matches!(
                side_of_surface(&Surface::Plane(chamfer), exactly_the_corner),
                Decision::Pos | Decision::Neg
            ),
            "the bare corner reads a certified sign, which is the refusal this change is about"
        );
        let miss = chamfer.signed_distance_to(corner).abs();
        let room = what_writing_the_corner_down_cost(exactly_the_corner.enclosure());
        assert!(
            miss > 0.0 && miss < room,
            "the miss {miss:e} is inside the {room:e} that writing this corner down cost"
        );

        let baked = derive_a_corner(
            vertex,
            exactly_the_corner,
            vec![
                face_through_the_corner(Vector::new(1.0, 0.0, 0.0)),
                face_through_the_corner(Vector::new(0.0, 1.0, 0.0)),
                face_through_the_corner(Vector::new(0.0, 0.0, 1.0)),
                Surface::Plane(chamfer),
            ],
            what_validate_allows(&model, cube),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a fourth plane inside the allowance agrees rather than proving a miss");

        assert_eq!(
            baked.point.exact_coordinates(),
            Some(corner.coordinates()),
            "the three that place it still place it, and the fourth moved nothing"
        );
    }

    /// The same disagreement the test above accepts, at the origin of a solid
    /// `1e5` across, is baked — because the re-check now grants the room
    /// `Model::validate_solid` grants that solid, and not only the rounding at
    /// the corner's own size (§136.4, §368).
    ///
    /// [`what_writing_the_corner_down_cost`] floors the size at one, so a
    /// corner at the origin is granted `1.421e-14` however big the part is. A
    /// chamfer pinned through its own corner `1e5` away misses the origin by
    /// `1.735e-13`, 12 times that. Until §368 that miss was the whole of the
    /// room, and this corner was refused though `validate` compared it against
    /// a room 8000 times wider.
    ///
    /// The solid whose room is granted is a cube of side `1e5`, standing in
    /// for the part the chamfer's far corner belongs to.
    #[test]
    fn a_corner_at_the_origin_of_a_big_solid_is_baked_for_a_miss_validate_allows() {
        let (mut model, cube) = a_cube_of_side_two();
        let vertex = corners_as_stored(&model, cube)[0].0;
        let corner = Point::origin();
        let exactly_the_corner = PredicatePoint::from(corner.coordinates());
        let face_through_the_corner = |normal| {
            Surface::Plane(
                Plane::from_point_and_normal(corner, normal).expect("an axis is a direction"),
            )
        };

        // (r, r, -r) is on the chamfer through the origin, so pinning the plane
        // there rather than here is what a face plane fitted through its own far
        // corner is, and the miss at the origin is what that cost.
        let reach = 1e5;
        let chamfer = Plane::from_point_and_normal(
            Point::new(reach, reach, -reach),
            Vector::new(-1.0, -2.4539e-18, -1.0),
        )
        .expect("a 45 degree wall with a tilt is still a direction");

        let a_big_solid = model
            .add_cube(reach, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube as big as the part");
        let validate_allows = what_validate_allows(&model, a_big_solid);

        let miss = chamfer.signed_distance_to(corner).abs();
        let the_rounding_alone = what_writing_the_corner_down_cost(exactly_the_corner.enclosure());
        assert!(
            the_rounding_alone < miss && miss < validate_allows.off_a_plane,
            "the miss {miss:e} is past the {the_rounding_alone:e} the corner's own rounding \
             grants and inside the {:e} validate grants a corner of this solid",
            validate_allows.off_a_plane
        );

        derive_a_corner(
            vertex,
            exactly_the_corner,
            vec![
                face_through_the_corner(Vector::new(1.0, 0.0, 0.0)),
                face_through_the_corner(Vector::new(0.0, 1.0, 0.0)),
                face_through_the_corner(Vector::new(0.0, 0.0, 1.0)),
                Surface::Plane(chamfer),
            ],
            validate_allows,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a miss validate allows is one the fuse has to be able to bake");
    }

    /// §70.3's ill-conditioned corner is still refused, with the whole room
    /// `Model::validate_solid` grants a corner of a solid of size two (§368).
    ///
    /// Three planes at a `1e-14` graze cross at the origin. A corner stored at
    /// `(1, 1, 0)` is **within what writing each of those planes down cost of
    /// all three of them** — `1e-14` against an allowance of `1.421e-14`, so
    /// `Model::validate`, which compares a corner with the planes through it,
    /// accepts this corner — and yet it sits `1.414` from where they actually
    /// cross. That is §70.3's sentence, that a corner close to all three
    /// surfaces can still be far from where they meet, written as numbers
    /// instead of as a paragraph (§16.10).
    ///
    /// What refuses it is [`the_place_the_model_points_at`], which is asked
    /// about the **stored** corner and runs before the re-check, which is asked
    /// about the **solved** one. **The error variant is the evidence of which
    /// of the two fired**: nothing placed the corner, so it is
    /// [`Error::Undecidable`] and not an [`Invalid::VertexOffItsFacePlane`]
    /// from the re-check. Growing the solved corner cannot reach a corner that
    /// never got solved.
    ///
    /// The second half is the control, and without it a regression that made a
    /// grazing triple unusable would leave the first half green for the wrong
    /// reason: the same three planes, with the corner stored where they cross,
    /// place it exactly. So the graze is not what refused the first.
    #[test]
    fn a_corner_near_three_grazing_planes_but_far_from_where_they_cross_is_refused() {
        let (model, cube) = a_cube_of_side_two();
        let vertex = corners_as_stored(&model, cube)[0].0;
        let a_graze = 1e-14;
        let through_the_origin = |normal| {
            Plane::from_point_and_normal(Point::origin(), normal).expect("a direction with length")
        };
        let grazing = [
            through_the_origin(Vector::new(0.0, 0.0, 1.0)),
            through_the_origin(Vector::new(0.0, a_graze, 1.0)),
            through_the_origin(Vector::new(a_graze, 0.0, 1.0)),
        ];

        let far_from_the_meet = Point::new(1.0, 1.0, 0.0);
        let what_writing_a_plane_down_cost = MEASURING_SLACK * Plane::SIGNED_DISTANCE_ROUNDOFF;
        for plane in &grazing {
            let off_it = plane.signed_distance_to(far_from_the_meet).abs();
            assert!(
                off_it <= what_writing_a_plane_down_cost,
                "this corner is {off_it:e} off one of its planes, inside the \
                 {what_writing_a_plane_down_cost:e} that writing the plane down cost, \
                 so the per-plane check accepts it"
            );
        }
        let validate_allows = what_validate_allows(&model, cube);
        let from_the_meet =
            far_from_the_meet.coordinates()[0].hypot(far_from_the_meet.coordinates()[1]);
        assert!(
            validate_allows.off_a_plane < from_the_meet,
            "the whole room validate grants, {:e}, is nowhere near the {from_the_meet} this \
             corner sits from where its planes cross",
            validate_allows.off_a_plane
        );

        let mut log = ProvenanceLog::new();
        let refused = derive_a_corner(
            vertex,
            PredicatePoint::from(far_from_the_meet.coordinates()),
            grazing.map(Surface::Plane).to_vec(),
            validate_allows,
            &Policy::certified(),
            &mut log,
        );
        assert!(
            matches!(refused, Err(Error::Undecidable { .. })),
            "a corner far from where its surfaces cross is one nothing placed, not one \
             a surface proved off its face, and it was {refused:?}"
        );

        let placed = derive_a_corner(
            vertex,
            PredicatePoint::from([0.0, 0.0, 0.0]),
            grazing.map(Surface::Plane).to_vec(),
            validate_allows,
            &Policy::certified(),
            &mut log,
        )
        .expect("the same three planes place a corner stored where they cross");
        assert_eq!(
            placed.point.exact_coordinates(),
            Some([0.0, 0.0, 0.0]),
            "so the graze is not what refused the corner above"
        );
    }
}
