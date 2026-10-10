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
//! An input vertex with exactly three planes round it is named by those planes
//! too, so a piece of the other solid that meets it there names it alike. No
//! coordinate is ever compared: two faces solving the same three planes in a
//! different order would round differently, and comparing what they rounded to
//! would either miss the join or invent one.
//!
//! An edge is then a pair of corners, a shell is a lump of faces joined by
//! edges, and an edge claimed by anything other than exactly two faces is
//! [`Error::ResultIsNotManifold`]. That is the whole manifold check, and it
//! lands here because this is where an edge with three claimants is *discovered*
//! rather than searched for. The one edge with four claimants that is not
//! refused is a line two faces rest against each other along, with both sides
//! of each kept: it is no edge of either input, or — since Q803 — one input's
//! own edge resting on a face of the other, kept as that input's edge; either
//! way [`Assembly::part_each_line_two_faces_rest_against_each_other_along`]
//! gives each face its own before anything is counted.
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
//! fuse cannot re-derive it. Two pieces on parallel planes facing the same way,
//! a rounding apart, are one face for the same reason (§426).
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

use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use rcad_geom::{
    Cone, DynamicRange, EdgeRun, HalfSpace, Isometry, MeetOfThree, Orientation, Picker, Plane,
    PlaneAgreement, Point, Surface, SurfaceMeet, Vector, meet_of_three_surfaces,
    which_way_a_closed_crossing_turns,
};
use rcad_predicates::{
    Approximation, Cause, Decision, Level, Measures, Point as PredicatePoint, Policy,
    ProvenanceLog, Record,
};
use smallvec::SmallVec;

use crate::holds::{
    WhereTheCurveIsCentred, a_picker_that_tells_the_two_arcs_apart,
    the_nappe_an_arc_that_does_not_close_is_on, the_nappe_of, the_picker_naming_the_circle_through,
    where_a_closed_crossing_is_centred,
};
use crate::key::{EdgeSlot, FaceSlot, ShellSlot, SolidSlot, VertexSlot, WireSlot};
use crate::model::{FaceUse, ShellUse, WireStep};
use crate::validate::{
    a_closed_crossing_is_walked_the_way_it_is_stored, gradient_of,
    how_much_space_a_curved_solid_encloses, the_curved_one_of, the_way_it_is_parametrised_about,
    winding_round,
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
use super::{
    FaceMeet, MeetPoint, NamesOfOnePoint, Sameness, Scratch, as_exact_as_it_is, gave_up, middle_of,
};

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
/// corner is a pole of the surface it is drawn on, and
/// [`Error::TheFuseWroteABrokenAnswer`] when two rings of a face joined from
/// coplanar pieces both wind as its outline.
pub(crate) fn assemble(
    model: &Model,
    scratch: &Scratch,
    chosen: &[Chosen],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Assembly> {
    let mut assembly = Assembly::from_the_pieces(model, scratch, chosen, policy, log)?;
    assembly.join_up_what_is_one_face(policy, log)?;
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
/// all, [`Error::ResultIsNotManifold`] when one of its edges is claimed by
/// other than two faces, which means the input was already broken, and
/// [`Error::TheFuseWroteABrokenAnswer`] when two rings of a face joined from
/// its coplanar faces both wind as its outline.
pub(crate) fn joined_faces_of_an_input(
    model: &Model,
    solid: SolidView<'_>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<Vec<InputFace>>> {
    let mut assembly = Assembly::from_one_input_solid(model, solid);
    if !assembly.join_up_what_is_one_face(policy, log)? {
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

/// Every side of every piece, each with the edge of an answer it would be,
/// named the way [`Assembly::from_the_pieces`] names the sides of the pieces it
/// keeps. Two sides of two pieces that come back with one edge are one side,
/// which is what `select` places a piece by when no point inside it can be
/// found (`docs/decisions.md` §365.3).
///
/// **The same naming, over every piece and not only the kept ones.** The
/// corners are [`Corners::of`], the edges [`Edges::of`] and so
/// [`which_arc_this_is`], and the vertices that only start a closed edge are
/// read through as they are for the kept pieces. A second matcher would be a
/// second answer to which sides are one, and the two could disagree. The one
/// place it parts from that naming is below, and it only splits a name.
///
/// **An arc between two places that came out one corner is named by the
/// places**, and not by [`Edges::of`]. Two places a hair apart, such as the
/// two at `16 ∓ 2⁻⁵¹` where a block's edge grazes a ball, can land on one
/// `f64` triple and so be one corner, and then both arcs of the ball's circle
/// through them run from that corner back to it. [`Edges::of`] reads that as
/// one closed edge and gives both arcs one number, so the ball's sliver
/// between the places matched the block's top along the long arc, read
/// inside the block there, and outside it across its true side on the wall,
/// and `select` refused it (Q1073,
/// `tests/a_block_edge_grazing_a_ball_at_cx_16.rs`). Here such a side, whose
/// two ends have two names ([`Corners::name_of`] read through
/// [`Corners::goes_by`]), is named by those two names and
/// [`which_arc_of_a_circle_walked_this_way`] with the names for the order, so
/// the two arcs are two names. **This only splits what [`Edges::of`] joined**:
/// the places and the sign tell apart sides it called one, and never join
/// two it called apart, so the rule in `select` can lose a match it had and
/// never gain one. A side the sign cannot name — a seam, or a pair it names
/// nothing for — is left out, as the straight side below is. The answer's own
/// naming stays [`Edges::of`]: an edge it builds from one corner to itself is
/// meshed as a whole closed curve ([`WhichArc::TheOnlyOne`]), and splitting the
/// name there would build a 2⁻⁵⁰ arc as a whole circle.
///
/// **A straight side whose two ends come out one corner is left out**, as
/// [`sides_of`] leaves it out of a kept piece (§365.7). Two corners of two
/// names that land on one `f64` triple are one corner ([`Corners::of`]), and a
/// piece can have a straight side between two such: a seed of the fuse
/// fuzzer's corpus and the draw of
/// `a_fused_draw_hands_its_provenance_back_through_the_door.rs` reach one in a
/// piece no rule keeps, and a KiCad pin header in pieces the rule keeps.
/// Leaving it out is sound for the reason it is in [`sides_of`]: every piece
/// names its ends through the one [`Corners`] here, so every piece that has
/// the side leaves it out, and a side is only ever compared by its `edge`
/// number. A side left out is one no piece can be placed across; the other
/// sides of its piece are still named. Until Q1034 the whole naming gave up
/// here, and no piece of the fuse could be placed by a side.
///
/// # Errors
///
/// Whatever [`Corners::with_each_closed_curve_started_once`],
/// [`Corners::of`] and [`Edges::of`] give.
pub(crate) fn the_edge_each_side_of_every_piece_is(
    model: &Model,
    scratch: &Scratch,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<Vec<NamedSide>>> {
    let on_three_planes = input_corners_on_exactly_three_planes(model, scratch);
    let no_corner_of_the_answer = vertices_read_through(
        model,
        scratch
            .pieces
            .iter()
            .flat_map(|piece| &piece.wires)
            .collect(),
        &scratch.names_of_one_point,
        &on_three_planes,
    );
    let wires: Vec<Vec<Cow<'_, [FaceMeet]>>> = scratch
        .pieces
        .iter()
        .map(|piece| {
            piece
                .wires
                .iter()
                .map(|wire| read_through(wire, &no_corner_of_the_answer, model))
                .collect()
        })
        .collect();
    let wires = with_the_halves_of_each_whole_curve_joined(wires, policy, log)?;
    let mut corners = Corners::with_each_closed_curve_started_once(
        model,
        on_three_planes,
        &scratch.names_of_one_point,
        wires.iter().flatten().flat_map(|wire| wire.iter()),
        policy,
        log,
    )?;
    let mut edges = Edges::default();
    let mut arcs_at_one_corner = BTreeMap::new();
    let mut named = Vec::with_capacity(scratch.pieces.len());
    for (piece, wires) in scratch.pieces.iter().zip(&wires) {
        let mut of_this_piece = Vec::new();
        for meet in wires.iter().flat_map(|wire| wire.iter()) {
            let from = corners.of(model, &meet.ends[0], policy, log)?;
            let to = corners.of(model, &meet.ends[1], policy, log)?;
            if from == to
                && meet
                    .carried_by
                    .iter()
                    .all(|surface| surface.as_plane().is_some())
            {
                continue;
            }
            if from == to {
                let ends = [
                    corners.goes_by(corners.name_of(model, &meet.ends[0], policy, log)?),
                    corners.goes_by(corners.name_of(model, &meet.ends[1], policy, log)?),
                ];
                if ends[0] != ends[1] {
                    let Some(arc) = which_arc_of_a_circle_walked_this_way(
                        meet,
                        ends[0] > ends[1],
                        policy,
                        log,
                    )?
                    else {
                        continue;
                    };
                    let key = ([ends[0].min(ends[1]), ends[0].max(ends[1])], arc);
                    let number = arcs_at_one_corner.len();
                    let number = *arcs_at_one_corner.entry(key).or_insert(number);
                    of_this_piece.push(NamedSide {
                        side: meet.clone(),
                        edge: SideName::AnArcBetweenTwoPlacesAtOneCorner(number),
                    });
                    continue;
                }
            }
            let seam = the_seam_a_side_runs_along(model, &piece.ancestors, meet);
            let side = edges.of(
                meet,
                seam,
                from,
                to,
                piece.ancestors[0],
                false,
                &corners,
                policy,
                log,
            )?;
            of_this_piece.push(NamedSide {
                side: meet.clone(),
                edge: SideName::Edge(side.edge),
            });
        }
        named.push(of_this_piece);
    }
    Ok(named)
}

/// One side of a piece, and the edge of an answer it would be.
pub(crate) struct NamedSide {
    pub(crate) side: FaceMeet,
    /// Two sides of two pieces with the same name here are one side.
    pub(crate) edge: SideName,
}

/// What [`the_edge_each_side_of_every_piece_is`] calls a side.
///
/// Exhaustive on purpose: it is private to this crate, and `select` only asks
/// whether two are equal.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SideName {
    /// The number of the edge [`Edges::of`] gives it.
    Edge(usize),
    /// An arc between two places whose corners came out one corner, numbered
    /// apart from every edge.
    AnArcBetweenTwoPlacesAtOneCorner(usize),
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

    /// Refuse an answer whose smallest feature is too small to resolve beside
    /// its own coordinates or its inputs'.
    ///
    /// **A fuse can build a feature neither input had.** Two blocks whose faces
    /// nearly line up meet in a sliver as thin as the gap between the two
    /// planes, and that gap is whatever the two placements make it — a number
    /// with no lower bound short of zero. A plane that passes a rounding beyond
    /// a cone's apex cuts the cone in an ellipse that rounding across. Both
    /// inputs pass `Model::check_dynamic_range` and the answer does not, so
    /// this is the one thing the fuse writes that its inputs said nothing
    /// about.
    ///
    /// It is a **limit** and not a gap. The ratio is a length over a length, so
    /// scaling the whole answer by any power of two leaves it exactly where it
    /// was: there is no scaled copy in which the sliver and the coordinates fit
    /// in `f64` together, which is the family of fix §16.10 asks to be ruled out
    /// by name. Nothing here can make the answer smaller, either — the sliver is
    /// where the two inputs really are.
    ///
    /// **The smallest feature is the shortest chord, or the cone's radius at a
    /// corner of a cone face**, the two `Model::check_dynamic_range` takes off
    /// an answer. The radius of a ball, a pipe or a ring is left out: a curved
    /// face of the answer sits on a surface inherited from its ancestor bit for
    /// bit (§17.1), and those radii are numbers of the surface that an input
    /// already carried. A cone's radius is not: it depends on how far up the
    /// axis the corner sits, and the corner is the fuse's own. The ellipse
    /// above is a closed edge, whose chord is zero, so only its radius says how
    /// small it is (Q1562, `docs/decisions.md` §350.5). A fuse writes no pole
    /// on a cone face, so a corner at the apex is an ordinary corner here and
    /// in `Model::check_dynamic_range` alike, and its radius of zero is passed
    /// over in both.
    ///
    /// **Measured against `inputs_reach`, the inputs' largest coordinate, as
    /// well as the answer's own.** The inputs stay in the caller's model beside
    /// the answer, and `Model::validate` folds over them together, so an answer
    /// whose own range fits — the tip of that cone, cut off whole, is a rounding
    /// across and sits a rounding from the origin — still leaves a model that
    /// does not. This is only asked after `the_inputs_fit_in_f64` has said the
    /// inputs fit on their own, so it refuses only a feature finer than any the
    /// inputs could carry.
    ///
    /// # Errors
    ///
    /// [`Invalid::DynamicRangeTooWide`] when the larger of `inputs_reach` and
    /// the answer's largest coordinate, over the answer's smallest feature, is
    /// more than `f64` carries, under a policy that fails on that.
    /// [`Error::Geometry`] when two corners of the answer with different names
    /// landed on the very same `f64` triple, which leaves an edge of no length
    /// at all.
    pub(crate) fn check_dynamic_range(
        &self,
        inputs_reach: f64,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let largest = self
            .corners
            .iter()
            .map(|corner| corner.point.largest_coordinate_magnitude())
            .fold(inputs_reach, f64::max);
        // A closed edge has one corner and a chord of zero, and that is not how
        // small it is — the same reasoning, and the same skip, as
        // `Model::check_dynamic_range`.
        let shortest_chord = self
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
        let shortest = shortest_chord.min(self.narrowest_a_cone_face_is_at_a_corner());
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

    /// The smallest radius above zero any cone face of the answer has at one
    /// of its corners, read as `Model::check_dynamic_range` reads it: the
    /// distance up the axis from the apex, times the slope. Infinite when no
    /// cone face has a corner off the apex.
    fn narrowest_a_cone_face_is_at_a_corner(&self) -> f64 {
        self.faces
            .iter()
            .filter_map(|face| {
                let Surface::Cone(cone) = face.surface else {
                    return None;
                };
                Some((face, cone))
            })
            .flat_map(|(face, cone)| {
                face.wires.iter().flatten().flat_map(move |side| {
                    self.edges[side.edge].ends.map(|corner| {
                        let up = cone.axis().direction();
                        cone.radius_at(up.dot(self.corners[corner].point - cone.apex()))
                    })
                })
            })
            .filter(|radius| *radius > 0.0)
            .fold(f64::INFINITY, f64::min)
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
    /// stand behind. That can be a gap here or an input that was already
    /// broken, since nothing above asks [`Model::validate`] of the inputs
    /// before fusing them — §294.2 measured six fuses that came back this way
    /// carrying exactly the `Invalid` `validate` gives the caller's own solid
    /// before the fuse runs. Either way it costs the caller nothing, since
    /// nothing was written. The blame is blanket because the sweep no longer
    /// asks the one question whose answer was the caller's geometry rather than
    /// this crate's arithmetic.
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
    /// A vertex that only starts a closed input edge is not a corner of the
    /// answer, and the two sides either side of it are read as one before any
    /// edge is named. [`vertices_that_only_start_a_closed_edge`] says why.
    ///
    /// # Errors
    ///
    /// [`Error::AClosedEdgeOfTheAnswerWouldStartAtAPole`] when a closed edge's
    /// one corner is a pole of the surface it is drawn on, where nothing says
    /// which way it runs, and whatever [`name_of_the_closed_crossing_through`]
    /// gives when it asks which of a plane's two circles round a ring a closed
    /// side is on.
    fn from_the_pieces(
        model: &Model,
        scratch: &Scratch,
        chosen: &[Chosen],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Self> {
        let on_three_planes = input_corners_on_exactly_three_planes(model, scratch);
        let no_corner_of_the_answer = vertices_read_through(
            model,
            chosen
                .iter()
                .flat_map(|kept| &scratch.pieces[kept.piece].wires)
                .collect(),
            &scratch.names_of_one_point,
            &on_three_planes,
        );
        let kept_wires: Vec<Vec<Cow<'_, [FaceMeet]>>> = chosen
            .iter()
            .map(|kept| {
                scratch.pieces[kept.piece]
                    .wires
                    .iter()
                    .map(|wire| read_through(wire, &no_corner_of_the_answer, model))
                    .collect()
            })
            .collect();
        let kept_wires = with_the_halves_of_each_whole_curve_joined(kept_wires, policy, log)?;
        let mut corners = Corners::with_each_closed_curve_started_once(
            model,
            on_three_planes,
            &scratch.names_of_one_point,
            kept_wires.iter().flatten().flat_map(|wire| wire.iter()),
            policy,
            log,
        )?;
        let mut edges = Edges::default();
        let mut faces = Vec::new();
        for (kept, kept_wires) in chosen.iter().zip(&kept_wires) {
            let piece = &scratch.pieces[kept.piece];
            let turned_round = piece.orientation.is_reversed() != kept.reversed;
            let wires: Vec<Vec<Side>> = kept_wires
                .iter()
                .map(|wire| {
                    sides_of(
                        wire,
                        turned_round,
                        &piece.ancestors,
                        model,
                        &mut corners,
                        &mut edges,
                        policy,
                        log,
                    )
                })
                .filter(|sides| {
                    piece.surface.as_plane().is_none()
                        || sides
                            .as_ref()
                            .map_or(true, |sides| encloses_something(sides))
                })
                .collect::<Result<_>>()?;
            if wires.is_empty() {
                continue;
            }
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
        let mut assembly = Self {
            corners: corners.made,
            edges: edges.made,
            runs: Vec::new(),
            faces,
            shells: Vec::new(),
            solids: Vec::new(),
        };
        assembly.leave_out_what_no_wire_walks();
        assembly.part_each_line_two_faces_rest_against_each_other_along(model);
        every_closed_edge_read_off_one_spelling(
            &assembly.edges,
            &assembly.corners,
            &mut assembly.faces,
        )?;
        Ok(assembly)
    }

    /// Leave out every edge no wire walks, and every corner no edge left ends
    /// at.
    ///
    /// **A flat wire that encloses nothing is left out after its sides are
    /// made** ([`encloses_something`]), and making a side makes its edge and
    /// its two corners. An edge only such wires walked has no face along it,
    /// and [`Assembly::darts_along_each_edge`] refused it as an edge with no
    /// faces. A prism lying on one of its edges on a block's top, with that
    /// edge a rounding into the top, is one: every corner of the sliver the
    /// two share rounds onto the prism's edge, every piece of the intersection
    /// is left out, and the prism's edge was all that was left
    /// (`a_prism_lying_on_one_edge.rs`). A cube turned so that three crossings
    /// of its planes with a face round onto one point and its corner rounds
    /// just beside it is another (`a_turned_cube_with_its_corner_on_a_face.rs`,
    /// queue row Q1442).
    ///
    /// **The corners go with the edges.** Where no face is joined there is no
    /// [`Assembly::rebuild_from`] to drop them, and a vertex with no edge is
    /// refused as an orphan when the answer is checked.
    ///
    /// Only a refusal can change. A fuse that answers passed
    /// [`Assembly::darts_along_each_edge`], so no edge of it had no face, and
    /// a corner with no edge was either dropped by the joining or refused.
    /// What stays keeps its order.
    fn leave_out_what_no_wire_walks(&mut self) {
        let mut walked = vec![false; self.edges.len()];
        for side in self
            .faces
            .iter()
            .flat_map(|face| face.wires.iter().flatten())
        {
            walked[side.edge] = true;
        }
        let mut an_end = vec![false; self.corners.len()];
        for (edge, &walked) in self.edges.iter().zip(&walked) {
            if walked {
                for end in edge.ends {
                    an_end[end] = true;
                }
            }
        }
        let edge_now = numbered(walked.iter().copied());
        let corner_now = numbered(an_end.iter().copied());
        let corner_now_of =
            |corner: usize| corner_now[corner].expect("an end of a walked edge stays a corner");

        for side in self
            .faces
            .iter_mut()
            .flat_map(|face| face.wires.iter_mut().flatten())
        {
            side.edge = edge_now[side.edge].expect("an edge a side walks stays an edge");
        }
        self.edges = core::mem::take(&mut self.edges)
            .into_iter()
            .zip(walked)
            .filter_map(|(edge, walked)| walked.then_some(edge))
            .map(|edge| Edge {
                ends: edge.ends.map(corner_now_of),
                arc: match edge.arc {
                    WhichArc::TheOnlyOne => WhichArc::TheOnlyOne,
                    WhichArc::LeavingACorner { corner, towards } => WhichArc::LeavingACorner {
                        corner: corner_now_of(corner),
                        towards,
                    },
                },
                ..edge
            })
            .collect();
        self.corners = core::mem::take(&mut self.corners)
            .into_iter()
            .zip(an_end)
            .filter_map(|(corner, an_end)| an_end.then_some(corner))
            .collect();
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
                        read_against: None,
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

    /// Part each line two faces rest against each other along into one edge
    /// per face, where the pieces made it one edge with four faces round it.
    ///
    /// **A bar lying along a wall or along another bar is cut along the line it
    /// touches on**, and a union keeps both sides of that line, of the wall and
    /// of the bar's skin alike: the insides are apart, so no piece of either is
    /// inside the other. The line then has four darts along it, two from pieces
    /// of the wall and two from the skin — two pieces, or one piece cut open
    /// along the line and walking it both ways — and it is not an edge of
    /// either input. Parted, the wall's edge has two pieces of one plane either
    /// side and [`Assembly::join_up_what_is_one_face`] dissolves it like any
    /// other, and the skin's is left to the same joining as any edge between
    /// pieces of one pipe; where one piece walks it both ways it stays, as the
    /// seam that piece is cut open along. The answer is the two solids as they
    /// stood, two lumps.
    ///
    /// **A prism lying along one of its edges on a block's top face** is the
    /// same with one difference: the prism's two darts are its two faces either
    /// side of its own edge, two input faces and not one. The top face's two
    /// pieces are parted off them, the top face is one face again, and the
    /// prism keeps its edge. That is parted only where it is two lumps:
    ///
    /// - **The prism's faces lie on the outer side of the face they rest on**
    ///   ([`Assembly::rest_on_the_outer_side_of`]). Inside it, they are the
    ///   walls of a prism lying in the block along its edge, kept turned round
    ///   in the block less the prism, and the answer is one solid pinched along
    ///   the line, not two.
    /// - **Nothing else joins the two pairs into one shell**
    ///   ([`Assembly::a_line_whose_two_sides_are_joined_some_other_way`]). A
    ///   solid lying on a block along one edge and reaching down into it
    ///   somewhere else is one solid that touches itself along the line, and a
    ///   shell that touches itself is what §17.4 refuses.
    ///
    /// **Four darts from four input faces stay one edge and are refused**: a
    /// line both solids have an edge along — two cubes touching along part of
    /// an edge, or the frame and the comb of `degenerate_fuses.rs`. No `Model`
    /// holds four faces along one edge, and §80.9 leaves two touching solids as
    /// an answer for that relation unbuilt.
    ///
    /// Which pair keeps the edge is the one whose first dart comes first; the
    /// other is moved onto a copy. **Each takes the input edges of its own input
    /// faces**, so that a bar whose seam is the touching line hands the seam to
    /// the skin's edge and not to the wall's, which is dissolved, and the
    /// prism's edge goes to the prism's; the line is no edge of the wall or the
    /// top face, so theirs takes none.
    ///
    /// # Panics
    ///
    /// If an input edge the line is a piece of runs along none of the four
    /// faces' input faces. Every one was named by a side of a piece of one of
    /// them, along that face's own boundary or its seam.
    fn part_each_line_two_faces_rest_against_each_other_along(&mut self, model: &Model) {
        let claimants = self.claimants_of_each_edge();
        let mut partings: Vec<Option<[[Dart; 2]; 2]>> = claimants
            .iter()
            .map(|by| self.the_two_pairs_along(by))
            .collect();
        while let Some(joined) =
            self.a_line_whose_two_sides_are_joined_some_other_way(&claimants, &partings)
        {
            partings[joined] = None;
        }
        for (kept, parting) in partings.into_iter().enumerate() {
            let Some([staying, moving]) = parting else {
                continue;
            };
            let runs_along = |input: EdgeKey, pair: [Dart; 2]| {
                model
                    .edge(input)
                    .expect("a side of a piece runs along an edge of the model the fuse was given")
                    .neighbouring_faces()
                    .any(|face| {
                        pair.iter()
                            .any(|dart| self.faces[dart.face].ancestors.contains(&face))
                    })
            };
            let (keeping, parting): (Vec<EdgeKey>, Vec<EdgeKey>) = self.edges[kept]
                .from
                .iter()
                .partition(|&&input| !runs_along(input, moving));
            for &input in &keeping {
                assert!(
                    runs_along(input, staying),
                    "an input edge a parted line is a piece of runs along one of its faces"
                );
            }
            let original = &self.edges[kept];
            let copy = Edge {
                ends: original.ends,
                carried_by: original.carried_by,
                arc: original.arc,
                from: parting,
            };
            self.edges[kept].from = keeping;
            self.edges.push(copy);
            let parted = self.edges.len() - 1;
            for dart in moving {
                self.faces[dart.face].wires[dart.wire][dart.position].edge = parted;
            }
        }
    }

    /// The two pairs four darts along one edge part into, the pair holding the
    /// first dart first; `None` where they do not part.
    ///
    /// Each pair walks the edge opposite ways and at least one is pieces of one
    /// input face. Where both are, it is a bar's skin on a wall or on another
    /// bar. Where only one is, the other two must rest on its outer side.
    fn the_two_pairs_along(&self, by: &[Dart]) -> Option<[[Dart; 2]; 2]> {
        let &[first, second, third, fourth] = by else {
            return None;
        };
        let both_ways =
            |[one, other]: [Dart; 2]| self.side_of(one).forward != self.side_of(other).forward;
        [
            [[first, second], [third, fourth]],
            [[first, third], [second, fourth]],
            [[first, fourth], [second, third]],
        ]
        .into_iter()
        .filter(|&[staying, moving]| both_ways(staying) && both_ways(moving))
        .find(|&[staying, moving]| {
            match (
                self.pieces_of_one_input_face(staying),
                self.pieces_of_one_input_face(moving),
            ) {
                (true, true) => {
                    self.faces[staying[0].face].ancestors != self.faces[moving[0].face].ancestors
                }
                (true, false) => self.rest_on_the_outer_side_of(moving, staying),
                (false, true) => self.rest_on_the_outer_side_of(staying, moving),
                (false, false) => false,
            }
        })
    }

    fn pieces_of_one_input_face(&self, [one, other]: [Dart; 2]) -> bool {
        self.faces[one.face].ancestors == self.faces[other.face].ancestors
    }

    /// Whether two faces resting along a line lie on the side the face they
    /// rest on points to: every corner of theirs on that side of its plane or
    /// on the plane, and at least one off it.
    ///
    /// The face rested on is stored with its plane pointing out of the
    /// material, turned round where the rule turned it, so the side it points
    /// to is outside whatever the rule.
    ///
    /// **Only flat faces resting on a flat face are asked**, and anything
    /// curved stays one edge and refused: a straight-sided flat face lies
    /// within its corners, and a curved one can bulge past them. **Every
    /// corner is asked, not only those near the line**, so a face with a corner
    /// on the inner side anywhere, or one the arithmetic cannot place, leaves
    /// the line one edge and refused. A piece of the face rested on has every
    /// corner on its plane and none off it, so it is never taken for one
    /// resting there.
    ///
    /// **Except a corner of the two pieces rested on**, the ends of the line
    /// among them: a corner of a piece of a face lies on that face's surface
    /// by how it was made, and asking the plane again asks only of the point
    /// it was written down as, the middle of a box round where three planes
    /// meet for a corner made that way. On a tilted top that middle can sit a
    /// rounding on the inner side, and `Plane::side_of_point` answers for the
    /// point it is handed. A wedge lying
    /// on its edge on a block's top, both turned about `x`, was refused united
    /// at 14 of 24 turns for its corner at an end of the edge read that way
    /// (queue row Q1281, `a_prism_lying_on_one_edge.rs`).
    fn rest_on_the_outer_side_of(&self, resting: [Dart; 2], on: [Dart; 2]) -> bool {
        let Some(plane) = self.faces[on[0].face].surface.as_plane() else {
            return false;
        };
        let on_the_plane: BTreeSet<usize> = on
            .iter()
            .flat_map(|dart| self.faces[dart.face].wires.iter().flatten())
            .flat_map(|side| self.edges[side.edge].ends)
            .collect();
        resting.iter().all(|dart| {
            let face = &self.faces[dart.face];
            if face.surface.as_plane().is_none() {
                return false;
            }
            let mut off_the_plane = false;
            for corner in face
                .wires
                .iter()
                .flatten()
                .flat_map(|side| self.edges[side.edge].ends)
                .filter(|corner| !on_the_plane.contains(corner))
            {
                match plane.side_of_point(PredicatePoint::from(
                    self.corners[corner].point.coordinates(),
                )) {
                    Decision::Pos => off_the_plane = true,
                    Decision::Zero => {}
                    Decision::Neg | Decision::Undecidable { .. } => return false,
                    other => panic!(
                        "which side of a plane a corner is on came back as a decision nothing \
                         here knows: {other:?}"
                    ),
                }
            }
            off_the_plane
        })
    }

    /// An edge parted onto two faces of one solid resting on a face of another
    /// whose two pairs other edges join into one shell all the same, or `None`
    /// when no parting leaves one.
    ///
    /// Each line a parting is kept for joins only its own pairs, and every
    /// other edge joins every face along it. Dropping one parting can only join
    /// more, so the caller asks again until this answers `None`. A bar's skin
    /// parted off a wall is not asked, and is parted as Q760 left it.
    fn a_line_whose_two_sides_are_joined_some_other_way(
        &self,
        claimants: &[Vec<Dart>],
        partings: &[Option<[[Dart; 2]; 2]>],
    ) -> Option<usize> {
        let mut shells = Sameness::of(self.faces.len());
        for (by, parting) in claimants.iter().zip(partings) {
            match parting {
                Some(pairs) => {
                    for [one, other] in pairs {
                        shells.join(one.face, other.face);
                    }
                }
                None => {
                    for dart in by {
                        shells.join(by[0].face, dart.face);
                    }
                }
            }
        }
        partings.iter().position(|parting| {
            parting.is_some_and(|[staying, moving]| {
                !(self.pieces_of_one_input_face(staying) && self.pieces_of_one_input_face(moving))
                    && shells.leader(staying[0].face) == shells.leader(moving[0].face)
            })
        })
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
    /// faces, [`Error::Undecidable`] when two planes cannot be told apart at
    /// all, [`Error::TheFuseWroteABrokenAnswer`] when two rings of a joined
    /// flat face both wind as its outline,
    /// [`Error::CannotPlaceTheRingsOfAJoinedFlatFace`] when nothing could say
    /// which of two or more rings that enclose nothing is its outline (both
    /// [`Assembly::rings_round`]), and whatever
    /// [`intersect`](rcad_geom::intersect) gives for a plane and a ring whose
    /// circle a ring of a joined face runs round.
    ///
    /// # Panics
    ///
    /// Where [`Assembly::rings_round`] cannot tell which ring of a joined flat
    /// face is its outline, as its own `# Panics` lists, or where a ring comes
    /// out bounding nothing once each ring of two straight sides is left out:
    /// fewer than three sides, unless it is one side that closes on itself, two
    /// different sides not both straight, or a ring of a whole closed surface
    /// ([`Assembly::with_each_ring_of_two_straight_sides_left_out`]). Both are
    /// broken internal invariants and a corrupt solid is worse than a crash
    /// (§4.1).
    fn join_up_what_is_one_face(
        &mut self,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<bool> {
        let along = self.darts_along_each_edge()?;
        let mut dissolved = self.edges_with_one_face_on_both_sides(&along, log)?;
        let staying = self.edges_a_curved_face_keeps(&along, &dissolved);
        for edge in staying {
            dissolved[edge] = false;
        }
        let same = joined_across(&along, &dissolved, self.faces.len());
        let mut joined = self.rings_of_each_joined_face(&same, &along, &dissolved, policy, log)?;
        let left_out = self.leave_out_each_flat_ring_another_plane_walks_back(
            &mut joined,
            &along,
            &mut dissolved,
            log,
        )?;
        let renumbered = self.what_is_left(&joined, &along, &dissolved);
        // **A closed curve cut where nothing meets it** is joined back up even
        // where no face was: a pointed cone less a block whose walls are tangent
        // to its rim keeps the rim in arcs otherwise, and a ring of arcs right
        // round a cone with no seam is refused (`AWireWillNotLieFlatOnItsFace`)
        // where the answer is the cone as it stood. Only then: rebuilding every
        // answer dissolves corners at the poles of a ball that the pole rules
        // need, measured as 23 failures across the ball-and-block files.
        if !same.joined_anything() && !renumbered.closed_a_curve_up && !left_out {
            return Ok(false);
        }
        self.turn_round_each_closed_edge_the_joined_spelling_stores_the_other_way(
            &joined, &along, &dissolved,
        )?;
        // **A ring of two straight sides left out sets two faces side by side**
        // that were not before: the turned frame's front and the block's front
        // under it, once the strip of the block's top between them goes
        // (queue row Q1446). They are one flat region (§426), so the joining
        // runs again. It runs again only after two or more edges were made
        // one, so there are fewer edges each time and it ends.
        if self.rebuild_from(&joined, renumbered)? {
            self.join_up_what_is_one_face(policy, log)?;
        }
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
        self.claimants_of_each_edge()
            .into_iter()
            .map(|by| {
                <[Dart; 2]>::try_from(by.as_slice())
                    .map_err(|_| Error::ResultIsNotManifold { count: by.len() })
            })
            .collect()
    }

    /// Every dart along each edge, however many there are.
    fn claimants_of_each_edge(&self) -> Vec<Vec<Dart>> {
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
    }

    /// Which edges have the same flat region on both sides of them.
    ///
    /// Two flat faces are one region when their planes have parallel normals
    /// facing the same way, whatever their offsets, and [`Plane::agreement_with`]
    /// decides both that and the same plane exactly, with no tolerance anywhere
    /// (§17.4). **Two parallel planes apart from each other meet nowhere**, so an
    /// edge between faces on them is one the rounding left between two pieces of
    /// one flat region, and kept, its ends have two plane directions round them
    /// where a corner needs three (§426). An edge whose two darts belong to one
    /// face stays: a face that runs up against itself is still bounded there.
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
                        Ok(agreement == PlaneAgreement::SameWay
                            || (agreement == PlaneAgreement::Different
                                && Self::facing_the_same_way(mine, theirs, log)?))
                    }
                    (None, None) => Ok(one_cone_pipe_or_ball_the_same_way_round(mine, theirs)),
                    (Some(_), None) | (None, Some(_)) => Ok(false),
                }
            })
            .collect()
    }

    /// Whether two planes have parallel normals facing the same way, wherever
    /// each sits.
    ///
    /// Both are slid to the origin and asked whether they are then one plane.
    /// [`Plane::moved_to`] keeps the normal's bits, so that is the exact
    /// question of the normals alone.
    fn facing_the_same_way(plane: &Plane, other: &Plane, log: &mut ProvenanceLog) -> Result<bool> {
        let through_the_origin =
            |plane: &Plane| plane.moved_to(0.0).expect("nought is a finite offset");
        let agreement =
            agreement_of_two_planes(&through_the_origin(plane), &through_the_origin(other), log)?;
        Ok(agreement == PlaneAgreement::SameWay)
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
        policy: &Policy,
        log: &mut ProvenanceLog,
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
                    let rings = self.rings_round(&members, along, dissolved, plane, policy, log)?;
                    (Surface::Plane(plane), Orientation::Forward, rings)
                }
                // The lowest spelling by bits, so the face does not depend on
                // which solid was handed over first.
                None => {
                    let lowest = members
                        .iter()
                        .map(|&member| &self.faces[member])
                        .min_by_key(|face| name_of_surface_bits(&face.surface))
                        .expect("a joined face is made of at least one piece");
                    (
                        lowest.surface,
                        lowest.used,
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

    /// The edges between pieces of one cone, pipe or ball that stay edges after
    /// all: one seam across each band the joining makes, and every edge of a
    /// group that cannot be joined.
    ///
    /// **A ball keeps no seam.** It has no one axis for a ring to go round, and
    /// the mesher opens a ball's face along meridians of its own: a ball less a
    /// disc along one half a turn from the rim, a ball less several in strips
    /// from pole to pole, a cap up to its pole (`open_the_ball_along_a_seam`,
    /// `open_the_ball_into_strips` and `cut_a_cap_up_to_its_pole` in `mesh.rs`,
    /// §202.1, §402). So every group on a ball is joined whole, and the ball's
    /// skin round a box's corner, cut by the box's three planes into eight
    /// pieces, comes back as the one face it is
    /// (`a_ball_kept_to_a_corner_of_a_box.rs`, §229.2). **Except the whole
    /// ball**: a group whose rings come out empty is every piece of it, and a
    /// joined face is never written as a whole closed surface, so its edges all
    /// stay and the ball stays in the pieces the fuse cut it into. Q1345's
    /// trial joined that too and failed 18 `rcad-topo` tests, a panic in
    /// `mesh.rs` among them.
    ///
    /// **A ring that goes right round needs a seam**, and the pieces a fuse
    /// leaves carry none — the arrangement cuts a cone's wall into pieces that
    /// each lie flat, and the input's own seam is not among their sides. Joined,
    /// a wall the block took a notch out of is a band with two rings right
    /// round it and nothing across, which is not a face the writer can lay out
    /// flat (`AWireWillNotLieFlatOnItsFace`). So dissolved edges that run from
    /// one ring to the other stay, as the seam (§224.2 item 2): one edge where
    /// one runs across, and otherwise a chain of rulings, as where a box inside
    /// a bar cuts its wall into a grid ([`Assembly::a_seam_across_the_band`]).
    /// Which is read off the corners, lowest first, and never off the indices
    /// here, which depend on the order the solids were handed over — the same
    /// rule [`Assembly::which_to_keep_first`] keeps a corner by.
    ///
    /// Anything else that goes right round — a group with other than two such
    /// rings, neither an edge nor a chain of rulings across, or a seam that
    /// still leaves a ring going round — and a group whose rings cannot be
    /// counted, stays apart. The answer is then the one the fuse gave before it
    /// joined curved faces at all.
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
            if matches!(self.faces[members[0]].surface, Surface::Sphere(_)) {
                if self
                    .rings_of_a_curved_face(&members, along, dissolved)
                    .is_empty()
                {
                    staying.extend(inside);
                }
                continue;
            }
            if self.needs_no_seam(&members, along, dissolved) {
                continue;
            }
            let seam = self.a_seam_across_the_band(&members, along, dissolved, &inside);
            match seam {
                Some(seam) => staying.extend(seam),
                None => staying.extend(inside),
            }
        }
        staying
    }

    /// Whether every ring of the face `members` join into either goes right
    /// round as one closed side on its own, or does not go right round at all.
    ///
    /// **A whole circle on its own is the band the arrangement already makes**: a frustum cut by a
    /// block's floor comes out as two bands, each between two whole circles
    /// and with no seam, and both are written and fused again as they are. So
    /// the two joined back across the circle between them need no seam either,
    /// and there is none to keep — a closed edge runs round a ring, never from
    /// one ring to the other. Leaving them apart kept that circle's one corner
    /// in the answer, and `A ∪ (A ∩ B)` came back with a corner `A` never had
    /// (`fuse_laws_over_a_cone.rs`,
    /// `a_cone_united_with_what_it_shares_with_a_block_is_the_cone`).
    ///
    /// A ring of several sides that goes right round beside a second ring is
    /// not covered: the band between them does need a seam, and
    /// [`Assembly::a_seam_across_the_band`] finds it.
    ///
    /// **One ring right round a cone and nothing else is a cap round its
    /// apex**, and needs no seam either, however many sides it has. The other
    /// piece that ring cuts the nappe into runs on for ever, and a face is
    /// bounded, so the face holds the apex, as the input's own skin does inside
    /// its rim. The mesher cuts such a face up to the apex
    /// (`BuildingAMesh::cut_a_cap_up_to_its_pole` in `mesh.rs`), as it cuts a
    /// nappe a cube stood on its corner leaves in three arcs. A cone less a
    /// block whose floor is just above the apex leaves its skin outside the
    /// block in three pieces whose one ring of four sides goes right round,
    /// and left apart they were two faces too many
    /// (`a_block_face_through_a_cones_apex.rs`, `docs/decisions.md` §347.4).
    /// On a pipe one ring right round bounds no face, so this is the cone's
    /// alone.
    fn needs_no_seam(&self, members: &[usize], along: &[[Dart; 2]], dissolved: &[bool]) -> bool {
        let surface = self.faces[members[0]].surface;
        let rings = self.rings_of_a_curved_face(members, along, dissolved);
        if let (Surface::Cone(_), [ring]) = (&surface, rings.as_slice())
            && self.a_ring_goes_right_round(ring, &surface) == Some(true)
        {
            return true;
        }
        rings.iter().all(|ring| {
            ring.len() == 1 || self.a_ring_goes_right_round(ring, &surface) == Some(false)
        })
    }

    /// The dissolved edges to keep as the seam across a band, where keeping
    /// them leaves a face that lies flat: one edge from one ring to the other
    /// where there is one, and otherwise the chain of rulings from ring to ring
    /// with the fewest edges.
    ///
    /// **A seam is a chain of edges where no one edge runs across.** A box
    /// standing inside a bar cuts the bar's wall at the box's floor and roof as
    /// well as down its four walls, so the wall comes back as a grid of 24
    /// pieces, 8 round by 3 up, and each line down it is three rulings broken
    /// where the floor and the roof cross it. No one edge runs from rim to rim,
    /// and asking for one left all 24 pieces apart where the answer has one
    /// wall (`a_box_inside_a_bar.rs`). The corners in the middle of the chain
    /// stay, as every corner where pieces of one cone or pipe met does
    /// ([`Assembly::what_is_left`]).
    ///
    /// **A chain is of rulings only**, which keep their angle round the axis,
    /// so it cuts the wall straight down as the input's own seam did. That is
    /// measured and not traced: a pointed cone united with a cube stood on its
    /// corner round its apex has no edge from its rim to the ring round its
    /// apex, and the fewest edges across are two arcs where the cube's planes
    /// cross the wall. Kept as the seam, they joined the nine pieces into a face
    /// the mesher refused, `TwoPatchesOfTheMeshDrewTheSameSide`
    /// (`a_wire_that_rings_a_cones_apex.rs`). Chains of any edges were refused
    /// that way in seven tests across four files, a frustum with a cube round
    /// its apex among them, and all seven answer with the pieces apart. A seam
    /// of one edge is kept whatever it runs along, as it was before chains.
    ///
    /// **Which is read off the corners and never off the indices here**, which
    /// depend on the order the solids were handed over: of the chains with the
    /// fewest edges, the one whose corners come lowest one after another by
    /// [`Assembly::which_to_keep_first`]. For one edge that is the edge whose
    /// lower end is lowest, then whose other end is, the rule a seam was chosen
    /// by before chains.
    fn a_seam_across_the_band(
        &self,
        members: &[usize],
        along: &[[Dart; 2]],
        dissolved: &[bool],
        inside: &[usize],
    ) -> Option<Vec<usize>> {
        let surface = self.faces[members[0]].surface;
        let rings = self.rings_of_a_curved_face(members, along, dissolved);
        let [one, other] = rings.as_slice() else {
            return None;
        };
        let corners_of = |ring: &[Dart]| -> BTreeSet<usize> {
            ring.iter().map(|&dart| self.starts_at(dart)).collect()
        };
        let (one, other) = (corners_of(one), corners_of(other));
        let rulings: Vec<usize> = inside
            .iter()
            .copied()
            .filter(|&edge| {
                let [from, to] = self.edges[edge].ends;
                from != to && matches!(self.edges[edge].arc, WhichArc::TheOnlyOne)
            })
            .collect();
        let seam = match self.the_lowest_way_across(inside, &one, &other)? {
            one_edge if one_edge.len() == 1 => one_edge,
            _ => self.the_lowest_way_across(&rulings, &one, &other)?,
        };
        let mut with_the_seam = dissolved.to_vec();
        for &edge in &seam {
            with_the_seam[edge] = false;
        }
        let rings = self.rings_of_a_curved_face(members, along, &with_the_seam);
        rings
            .iter()
            .all(|ring| self.a_ring_goes_right_round(ring, &surface) == Some(false))
            .then_some(seam)
    }

    /// The chain of `edges` from a corner of `one` to a corner of `other` with
    /// the fewest edges, and of those the one whose corners come lowest one
    /// after another, walked from whichever ring that puts first; or `None`
    /// where no chain joins them, or where two of `edges` join the same two
    /// corners of it, which would leave the choice between them to the indices.
    fn the_lowest_way_across(
        &self,
        edges: &[usize],
        one: &BTreeSet<usize>,
        other: &BTreeSet<usize>,
    ) -> Option<Vec<usize>> {
        let mut next_to: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for &edge in edges {
            let [a, b] = self.edges[edge].ends;
            next_to.entry(a).or_default().push(b);
            next_to.entry(b).or_default().push(a);
        }
        let there = self.the_lowest_walk(&next_to, one, other)?;
        let back = self.the_lowest_walk(&next_to, other, one)?;
        let back_is_lower = there
            .iter()
            .zip(&back)
            .map(|(&there, &back)| self.which_to_keep_first(back, there))
            .find(|order| order.is_ne())
            .is_some_and(Ordering::is_lt);
        let corners = if back_is_lower { back } else { there };
        corners
            .windows(2)
            .map(|step| {
                let [only] = edges
                    .iter()
                    .copied()
                    .filter(|&edge| {
                        let [a, b] = self.edges[edge].ends;
                        [a, b] == [step[0], step[1]] || [b, a] == [step[0], step[1]]
                    })
                    .collect::<Vec<_>>()[..]
                else {
                    return None;
                };
                Some(only)
            })
            .collect()
    }

    /// The corners of the chain through `next_to` from a corner of `from` to
    /// one of `to` with the fewest steps, and of those the one whose corners
    /// come lowest one after another, or `None` where none joins them.
    ///
    /// Every corner is first given how many steps it is from `to`. The walk
    /// then starts at the lowest corner of `from` that is fewest steps away,
    /// and steps each time to the lowest neighbour one step nearer. A corner in
    /// the middle is never on either ring: one on `to` would be no steps away,
    /// and one on `from` nearer than where the walk started would have been the
    /// start.
    fn the_lowest_walk(
        &self,
        next_to: &BTreeMap<usize, Vec<usize>>,
        from: &BTreeSet<usize>,
        to: &BTreeSet<usize>,
    ) -> Option<Vec<usize>> {
        let mut away: BTreeMap<usize, usize> = to.iter().map(|&corner| (corner, 0)).collect();
        let mut waiting: std::collections::VecDeque<usize> = to.iter().copied().collect();
        while let Some(corner) = waiting.pop_front() {
            let further = away[&corner] + 1;
            for &next in next_to.get(&corner).into_iter().flatten() {
                if let std::collections::btree_map::Entry::Vacant(unseen) = away.entry(next) {
                    unseen.insert(further);
                    waiting.push_back(next);
                }
            }
        }
        let lowest = |one: &usize, other: &usize| self.which_to_keep_first(*one, *other);
        let fewest = from.iter().filter_map(|corner| away.get(corner)).min()?;
        let mut at = from
            .iter()
            .copied()
            .filter(|corner| away.get(corner) == Some(fewest))
            .min_by(lowest)
            .expect("the fewest steps away is how far some corner of the ring is");
        let mut walk = vec![at];
        while away[&at] > 0 {
            let nearer = away[&at] - 1;
            at = next_to[&at]
                .iter()
                .copied()
                .filter(|next| away.get(next) == Some(&nearer))
                .min_by(lowest)
                .expect("a corner some steps away has a neighbour one step nearer");
            walk.push(at);
        }
        Some(walk)
    }

    /// The rings round a joined face on a cone, a pipe or a ball, in the order
    /// they are first come to.
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
            Surface::Plane(_) | Surface::Sphere(_) | Surface::Torus(_) => return None,
            // A surface kind added to `rcad-geom` after this has no ring count
            // here, so its pieces are left apart as a ring's are.
            _ => return None,
        };
        let round = |at: Point| axis.direction().cross(at - axis.origin());
        let mut closed = 0u32;
        let mut turned = 0.0f64;
        for &dart in ring {
            let side = self.side_of(dart);
            let edge = &self.edges[side.edge];
            let partner = edge
                .carried_by
                .iter()
                .find(|other| !one_surface(other, surface))?;
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

    /// The rings of one joined flat face, its outline first and then its holes.
    ///
    /// The outline is the one ring that winds with the plane's normal, and the
    /// holes wind against it. No outline at all would be a region bounded by
    /// holes alone, which has no outside — every face that got here came out of
    /// [`Assembly::from_the_pieces`] or [`Assembly::from_one_input_solid`] with
    /// an outline of its own, and joining only ever takes an edge *away* from
    /// between two of them.
    ///
    /// # Errors
    ///
    /// [`Error::Undecidable`] where a ring's winding cannot be decided.
    ///
    /// [`Error::TheFuseWroteABrokenAnswer`], `broken`
    /// [`Invalid::HoleWindsWithTheOuterWire`], where two rings wind with the
    /// normal. Either the joined region touches itself at a corner, which is a
    /// solid this kernel cannot hold, or a hole's corners lie within a rounding
    /// of each other and wind the wrong way as stored: a cube turned so its
    /// corner rests on another's face, with its three planes meeting a rounding
    /// outside the face, leaves the face a hole of that size
    /// (`a_turned_cube_with_its_corner_on_a_face.rs`, Q1287). Written, either
    /// face is one [`Model::validate`] refuses for that `broken`.
    ///
    /// [`Error::CannotPlaceTheRingsOfAJoinedFlatFace`] where two or more rings
    /// enclose nothing and nothing could place them: the ones
    /// [`Assembly::holds_the_other_rings`] answers `None` for, with no ring
    /// winding with the plane and none shown to hold the others. A cap along
    /// four pipes whose corners sweep exactly nothing, with its own shape an
    /// eighth the size standing through it, reaches it: united, or less the
    /// small one, its top is joined from nine pieces with two such rings,
    /// neither along one surface and neither with its corners on one line
    /// (`a_joined_flat_face_whose_rings_enclose_no_area.rs`, Q1253). It was a
    /// panic, but a caller builds that cap through `SolidBuilder`, and the face
    /// is a sound one whose rings this reading cannot place, so it is a gap and
    /// not a broken invariant. Of a ring along one torus, the rings nothing
    /// could place are one whose plane rests on the tube, which touches it
    /// round one circle no picker names, and one where no corner of the other
    /// rings is placed; no shape is known to reach those. The log gets a
    /// record holding the corners of each ring left unplaced.
    ///
    /// **A ring whose corners enclose nothing is placed by the rule above**,
    /// because its corners cannot place it. A half disk has two corners, so the
    /// hole a half ball pokes through a knife's face, joined across the axis
    /// from the knife's other half, winds round no area at all (§200.1's two
    /// knives at 90 degrees, as a union). With one outline found among the other
    /// rings it is a hole; with none, and it the only such ring, it is the
    /// outline.
    ///
    /// **Two or more such rings and no outline are placed by the regions they
    /// bound**, since the outline is the one ring that holds the others
    /// ([`Assembly::holds_the_other_rings`]). A tube's cap cut by the pipe of a
    /// bar standing inside the tube's wall is two pieces, and joined back up it
    /// is two whole circles of one corner each (`a_bar_inside_a_tubes_wall.rs`).
    /// A bar across the tube's wall, united with it, leaves an outline along
    /// two pipes and a hole along two pipes, two corners each, and the top
    /// halves of two half bends of one ring, united, leave on their joined
    /// floor an outline and a hole each along one of the two circles the floor
    /// cuts the ring in, two corners each
    /// (`a_joined_flat_face_whose_rings_enclose_no_area.rs`, Q1387).
    /// A ring shown to hold the others is the outline and the rest are holes;
    /// with none shown to, and only one ring left that nothing could place,
    /// that one is the outline by the rule above.
    ///
    /// # Panics
    ///
    /// Where the rings still cannot say which one is the outline:
    ///
    /// - **A ring winding neither way.** Nothing reaches it:
    ///   [`winding_round`] answers `Neg`, `Zero` or `Pos`, or refuses.
    /// - **No ring winding with the plane, and none of those enclosing nothing
    ///   holding the others**, every one of them answering `Some(false)`.
    ///   Nothing keeps answers off it: two bars united and then united with the
    ///   half bar whose corner is their lens's corner reach it, both ways round
    ///   (`a_half_bar_whose_corner_is_the_lens_corner_mostly_answers` in
    ///   `a_lens_whose_hole_is_half_a_disk.rs`, Q1290).
    /// - **Two or more rings each holding the others**, which would mean the
    ///   answer's rings cross. Nobody has traced whether any shape reaches it.
    fn rings_round(
        &self,
        members: &[usize],
        along: &[[Dart; 2]],
        dissolved: &[bool],
        plane: Plane,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Vec<Vec<Dart>>> {
        let corners_of = |ring: &[Dart]| -> Vec<[f64; 3]> {
            ring.iter()
                .map(|&dart| self.corners[self.starts_at(dart)].point.coordinates())
                .collect()
        };
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
                    let corners = corners_of(&ring);
                    match winding_round(&corners, plane.normal().components())? {
                        Decision::Pos => {
                            if let Some(first) = &outline {
                                return Err(no_one_ring_is_the_outline(
                                    &[corners_of(first), corners],
                                    Invalid::HoleWindsWithTheOuterWire,
                                    log,
                                ));
                            }
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
            None if enclosing_nothing.len() == 1 => enclosing_nothing.remove(0),
            None => {
                let mut held: Vec<Option<bool>> = Vec::with_capacity(enclosing_nothing.len());
                for ring in 0..enclosing_nothing.len() {
                    let others = enclosing_nothing
                        .iter()
                        .enumerate()
                        .filter(|&(other, _)| other != ring)
                        .map(|(_, other)| other)
                        .chain(&holes);
                    held.push(self.holds_the_other_rings(
                        &enclosing_nothing[ring],
                        others,
                        &plane,
                        policy,
                        log,
                    )?);
                }
                let holding: Vec<usize> = (0..held.len())
                    .filter(|&ring| held[ring] == Some(true))
                    .collect();
                let unplaced: Vec<usize> = (0..held.len())
                    .filter(|&ring| held[ring].is_none())
                    .collect();
                let which = match (holding.as_slice(), unplaced.as_slice()) {
                    ([only], _) | ([], [only]) => *only,
                    ([], []) => panic!(
                        "every face joined here brought an outline of its own and joining only \
                         takes edges away, so the joined face has one too, and none of its rings \
                         winds with the plane or bounds a region holding the others"
                    ),
                    ([], _) => {
                        let corners: Vec<PredicatePoint> = unplaced
                            .iter()
                            .flat_map(|&ring| {
                                let corners = corners_of(&enclosing_nothing[ring]);
                                let first = corners[0];
                                corners.into_iter().chain([first])
                            })
                            .map(PredicatePoint::from)
                            .collect();
                        log.push(Record::new(
                            WHICH_RING_IS_THE_OUTLINE,
                            &corners,
                            Level::Interval,
                            f64::INFINITY,
                            Measures::Area,
                        ));
                        return Err(Error::CannotPlaceTheRingsOfAJoinedFlatFace {
                            face: self.faces[members[0]].ancestors[0],
                        });
                    }
                    (_, _) => panic!(
                        "a face joined out of coplanar pieces has one outline, and {} of its rings \
                         each hold the others, so the answer's rings cross",
                        holding.len()
                    ),
                };
                let outline = enclosing_nothing.remove(which);
                holes.extend(enclosing_nothing);
                outline
            }
        };
        Ok(core::iter::once(outline).chain(holes).collect())
    }

    /// Whether the region `ring` bounds on its plane holds the rings `others`,
    /// or `None` where that region cannot be read off its sides.
    ///
    /// **A ring every side of which runs where the plane crosses one pipe, ball
    /// or cone is the whole of that crossing**, and the crossing closes: the
    /// ones that do not — two lines, a parabola, a hyperbola's two branches —
    /// cannot close a ring on their own. The region such a ring bounds is then
    /// the part of the plane inside that surface, so a corner of another ring
    /// is held exactly when the surface puts it inside. One corner answers for
    /// the whole question, because the outline holds every other ring of the
    /// face and a hole holds none of them. A corner on the surface, or one the
    /// arithmetic cannot place, is passed over for the next, the way
    /// [`Assembly::one_shell_is_inside_another`] passes over a corner.
    ///
    /// **A ring whose sides run along more than one surface, or straight, is
    /// placed by how its arcs bulge**, where its corners all lie on one line.
    /// The polygon through its corners then sweeps nothing, so what the ring
    /// winds round a point is the sum, over its arcs, of what each arc and its
    /// chord wind round it: one turn, the way the arc runs, for a point between
    /// the two, and nothing for any other. The piece of the plane between an
    /// arc and its chord is the part inside the arc's surface on the side of
    /// the chord the arc bulges to, because every arc counted is of a crossing
    /// that closes round the inside of its surface — a circle or an ellipse —
    /// or of a cone's parabola or one branch of its hyperbola, which bounds the
    /// inside of the cone's nappe it is on, a convex region the chord's line
    /// meets in the chord alone (`holds.rs`'s
    /// `the_way_into_an_arc_that_does_not_close` has why). On a hyperbola's
    /// plane the inside of the other nappe is a second region, and a point
    /// there is in no piece of the arc. A straight side is its own chord and
    /// adds nothing. So a corner is held when the turns add up to one, either
    /// way round, and not when they add up to none. The outline of a bar
    /// united across a tube's wall runs along two pipes and so does its hole,
    /// and a cut cone's wall united with a cone closed by a ball has an outline
    /// and a hole each with a side along a hyperbola
    /// (`a_joined_flat_face_whose_rings_enclose_no_area.rs`).
    ///
    /// A corner on an arc's surface is outside that arc's piece: it is not on
    /// this ring, so it is on the rest of the curve, which lies across the
    /// chord. A corner of this ring, one on a chord inside its arc's surface,
    /// or one the arithmetic cannot place, is passed over. Which side of its
    /// chord an arc bulges is read off the direction it leaves its named corner
    /// ([`WhichArc::LeavingACorner`]); every other sign is exact.
    /// `a_sliver_arc_on_a_joined_flat_face.rs` pins how far from wrong that
    /// reading stays: on every chord the fuse builds down to a few millionths
    /// long, the rounding never puts the point on the wrong side.
    ///
    /// **A ring every side of which runs along one torus is the whole of one of
    /// the two circles the plane cuts it in**, because the two circles do not
    /// meet. The region it bounds is that circle's disk, and not the part of
    /// the plane inside the torus: a plane square to the axis cuts it in two
    /// circles round the axis with the torus between them, and a plane through
    /// the axis in two meridian circles side by side. So the circle is named
    /// the way [`intersect`](rcad_geom::intersect) names it, and the torus and
    /// that picker together say which side of it a corner is on
    /// ([`Assembly::what_the_disk_of_a_rings_circle_holds`]). The floors of the
    /// top halves of two half bends united join into the flat ring between two
    /// such circles (`a_joined_flat_face_whose_rings_enclose_no_area.rs`).
    ///
    /// `None` where no corner is placed, and for a ring along a torus whose
    /// plane rests on the tube and touches it round one circle, which no picker
    /// names. Of a ring along more than one surface,
    /// `None` too where its corners are not on one line, where a closed side is
    /// one of several, for an arc along a torus, for an arc along a cone's
    /// parabola or hyperbola whose plane holds the apex or whose ends are not
    /// both on one nappe, for an arc whose bulge reads along its chord, and
    /// where a corner's nappe cannot be settled.
    ///
    /// # Errors
    ///
    /// Whatever [`intersect`](rcad_geom::intersect) gives for the plane and the
    /// torus a ring runs along.
    ///
    /// # Panics
    ///
    /// On a [`Decision`] this crate has not been taught, for the reason
    /// [`winding_round`] gives.
    fn holds_the_other_rings<'a>(
        &self,
        ring: &[Dart],
        mut others: impl Iterator<Item = &'a Vec<Dart>>,
        plane: &Plane,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Option<bool>> {
        let the_curved_one = |dart: Dart| {
            let [one, other] = self.edges[self.edge_of(dart)].carried_by;
            match (one.as_plane(), other.as_plane()) {
                (Some(_), None) => Some(other),
                (None, Some(_)) => Some(one),
                (Some(_), Some(_)) | (None, None) => None,
            }
        };
        let Some(&first) = ring.first() else {
            return Ok(None);
        };
        let along_one = the_curved_one(first).filter(|first| {
            ring.iter()
                .all(|&dart| the_curved_one(dart).is_some_and(|curved| one_surface(&curved, first)))
        });
        if let Some(surface) = along_one {
            match surface {
                Surface::Cylinder(_) | Surface::Sphere(_) | Surface::Cone(_) => (),
                Surface::Torus(_) => {
                    return self.what_the_disk_of_a_rings_circle_holds(
                        ring, others, plane, &surface, policy, log,
                    );
                }
                Surface::Plane(_) => return Ok(None),
                // A kind added to `rcad-geom` after this has not been shown to
                // bound the region inside it, so a ring along it is not placed.
                _ => return Ok(None),
            }
            return Ok(others.find_map(|other| {
                other.iter().find_map(|&dart| {
                    let at = self.corners[self.starts_at(dart)].point.coordinates();
                    match crate::validate::side_of_surface(&surface, PredicatePoint::from(at)) {
                        Decision::Neg => Some(true),
                        Decision::Pos => Some(false),
                        Decision::Zero | Decision::Undecidable { .. } => None,
                        other => panic!(
                            "which side of a surface a corner is on came back as a decision \
                             nothing here knows: {other:?}"
                        ),
                    }
                })
            }));
        }

        let normal = plane.normal().components();
        let Some(arcs) = how_each_arc_bulges(self, ring, plane, normal, the_curved_one) else {
            return Ok(None);
        };

        let own: Vec<usize> = ring.iter().map(|&dart| self.starts_at(dart)).collect();
        let at_corner = |corner: usize| self.corners[corner].point.coordinates();
        if let Some(&apart) = own.iter().find(|&&corner| corner != own[0]) {
            let line = [at_corner(own[0]), at_corner(apart)];
            let on_the_line = |corner: usize| {
                winding_round(&[line[0], line[1], at_corner(corner)], normal).ok()
                    == Some(Decision::Zero)
            };
            if !own.iter().all(|&corner| on_the_line(corner)) {
                return Ok(None);
            }
        }

        Ok(others.find_map(|other| {
            other.iter().find_map(|&dart| {
                let corner = self.starts_at(dart);
                if own.contains(&corner) {
                    return None;
                }
                match turns_round(&arcs, normal, at_corner(corner))? {
                    0 => Some(false),
                    1 | -1 => Some(true),
                    // A simple ring winds round a point once at most, so
                    // reaching more than that is a bug here. Which bug is not
                    // shown: forcing an arc's bulge to read the wrong way
                    // round gave no turns, or two rings each holding the
                    // other, not two turns (Q600, with a hook since
                    // reverted).
                    _ => None,
                }
            })
        }))
    }

    /// Whether the disk of the circle `ring` runs right round holds the rings
    /// `others`, for a ring every side of which is along `torus`, or `None`
    /// where no corner of theirs is placed.
    ///
    /// **The circle is named by the picker `intersect` gives it**: the outside
    /// of the pipe of the ring's own radius `R` for the outer of two circles
    /// round the axis, its inside for the inner, and a flat half-space for one
    /// of two meridian circles ([`the_picker_naming_the_circle_through`]). On
    /// the plane the torus holds the band between two circles round the axis,
    /// or the two meridian disks, so with `F` the torus's sign at a corner and
    /// the picker either keeping it (on or past the pipe, or on or in front of
    /// the half-space) or not:
    ///
    /// | circle | `F` below zero | `F` above zero, kept | `F` zero or above, not kept |
    /// |---|---|---|---|
    /// | outer, `OutsideAPipe` | held: in the band | not held: further out | held: nearer the axis than `R` |
    /// | inner, `InsideAPipe` | not held: in the band | held: in the hole | not held: further from the axis than `R` |
    /// | meridian, `InFrontOf` | held where kept, in this disk, and not in the other | not held | not held: on the other circle's side |
    ///
    /// A corner where `F` is zero and the picker keeps it is on this circle,
    /// and is passed over for the next, as is a corner either sign cannot
    /// place. Every sign is exact. A corner of the other ring is on the torus
    /// when that ring is the other circle, so `F` alone never places it: the
    /// picker does. `holds.rs`'s `which_side_of_a_rings_circle` reads a face's
    /// circle by the same table.
    ///
    /// # Errors
    ///
    /// Whatever [`intersect`](rcad_geom::intersect) gives for `plane` and
    /// `torus`.
    ///
    /// # Panics
    ///
    /// On a [`Decision`] this crate has not been taught, and where the circle's
    /// picker is a kind [`intersect`](rcad_geom::intersect) does not name a
    /// circle of a plane and a ring by.
    fn what_the_disk_of_a_rings_circle_holds<'a>(
        &self,
        ring: &[Dart],
        mut others: impl Iterator<Item = &'a Vec<Dart>>,
        plane: &Plane,
        torus: &Surface,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Option<bool>> {
        let at = |dart: Dart| {
            PredicatePoint::from(self.corners[self.starts_at(dart)].point.coordinates())
        };
        let Some(this_circle) = the_picker_naming_the_circle_through(
            &Surface::Plane(*plane),
            torus,
            at(ring[0]),
            policy,
            log,
        )?
        else {
            return Ok(None);
        };
        let outer = match this_circle {
            Picker::OutsideAPipe(_) => Some(true),
            Picker::InsideAPipe(_) => Some(false),
            Picker::InFrontOf(_) => None,
            Picker::OutsideABall(_) | _ => {
                panic!("intersect names a circle a plane cuts a ring in by {this_circle:?}")
            }
        };
        Ok(others.find_map(|other| {
            other.iter().find_map(|&dart| {
                let corner = at(dart);
                let kept = match this_circle.side_of_point(corner) {
                    Decision::Pos | Decision::Zero => true,
                    Decision::Neg => false,
                    Decision::Undecidable { .. } => return None,
                    other => {
                        panic!("a picker came back with a decision nothing here knows: {other:?}")
                    }
                };
                let inside_the_tube = match crate::validate::side_of_surface(torus, corner) {
                    Decision::Neg => true,
                    Decision::Pos => false,
                    Decision::Zero if kept => return None,
                    Decision::Zero => false,
                    Decision::Undecidable { .. } => return None,
                    other => panic!(
                        "which side of a ring a corner is on came back as a decision nothing \
                         here knows: {other:?}"
                    ),
                };
                Some(match (outer, inside_the_tube, kept) {
                    (Some(outer), true, _) => outer,
                    (Some(outer), false, true) => !outer,
                    (Some(outer), false, false) => outer,
                    (None, inside_the_tube, kept) => inside_the_tube && kept,
                })
            })
        }))
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

    /// Turn round both sides of every closed edge that the spelling its face
    /// is joined onto stores running the other way.
    ///
    /// **A closed edge is stored running the way the longitude of the lower of
    /// its two faces' curved surfaces by bits increases** — what the mesher and
    /// `Model::validate` both read, and what
    /// [`every_closed_edge_read_off_one_spelling`] already settled each side
    /// against. Where joining changes that surface, both sides of the edge turn
    /// round exactly when the new one stores it the other way, the flat cap's
    /// side too though the cap was never joined to anything:
    ///
    /// - **Two spellings of one pipe**: the longitude is `axis × gradient`, the
    ///   gradient does not change, and turning the axis round negates it, so
    ///   the dot product of the two axes says it with no rounding (§231.3).
    ///   Sliding the origin changes neither.
    /// - **Two different surfaces**, which is a curved partner whose bits sort
    ///   between two spellings of the wall: the two stored ways are read with
    ///   the rule itself, [`a_closed_crossing_is_walked_the_way_it_is_stored`],
    ///   along the crossing the edge was cut as.
    ///
    /// # Errors
    ///
    /// [`Error::CannotTellWhichWayAClosedEdgeRuns`] where the rule cannot tell
    /// which way a different surface stores the edge, and
    /// [`Error::AClosedEdgeOfTheAnswerWouldStartAtAPole`] where it says the
    /// crossing has no direction at the edge's corner.
    fn turn_round_each_closed_edge_the_joined_spelling_stores_the_other_way(
        &mut self,
        joined: &[JoinedFace],
        along: &[[Dart; 2]],
        dissolved: &[bool],
    ) -> Result<()> {
        let mut joined_on: Vec<Surface> = self.faces.iter().map(|face| face.surface).collect();
        for face in joined {
            for &member in &face.members {
                joined_on[member] = face.surface;
            }
        }
        for (edge, darts) in along.iter().enumerate() {
            let [start, end] = self.edges[edge].ends;
            if dissolved[edge] || start != end {
                continue;
            }
            let before = darts.map(|dart| self.faces[dart.face].surface);
            let after = darts.map(|dart| joined_on[dart.face]);
            if before.map(|surface| name_of_surface_bits(&surface))
                == after.map(|surface| name_of_surface_bits(&surface))
            {
                continue;
            }
            let at = self.corners[start].point;
            let [was, is] = [before, after].map(|pair| {
                the_curved_one_of(&pair)
                    .expect("a face that joining moved onto another spelling is curved")
            });
            if name_of_surface_bits(&was) == name_of_surface_bits(&is) {
                continue;
            }
            let turned = if one_surface(&was, &is) {
                the_way_it_is_parametrised_about(&was, at)
                    .dot(the_way_it_is_parametrised_about(&is, at))
                    < 0.0
            } else {
                let outward = self.edges[edge]
                    .carried_by
                    .map(|surface| gradient_of(&surface, at));
                let stored_way = |reference: &Surface| {
                    a_closed_crossing_is_walked_the_way_it_is_stored(
                        [reference, reference],
                        outward,
                        at,
                        Decision::Neg,
                    )?
                    .ok_or_else(|| {
                        Error::AClosedEdgeOfTheAnswerWouldStartAtAPole {
                            face: self.faces[darts[0].face].ancestors[0],
                        }
                    })
                };
                stored_way(&was)? != stored_way(&is)?
            };
            if turned {
                for dart in darts {
                    let side = &mut self.faces[dart.face].wires[dart.wire][dart.position];
                    side.forward = !side.forward;
                }
            }
        }
        Ok(())
    }

    /// Write the corners, edges and faces out again with the joined faces in
    /// place of their pieces and the corners that stopped being corners gone.
    ///
    /// Everything keeps the order it was in, so an answer that joined nothing
    /// comes out of here byte for byte the answer that went in.
    ///
    /// Answers whether a ring of two straight sides was left out, since that
    /// can set two faces side by side that the joining has not seen together.
    ///
    /// # Panics
    ///
    /// If a ring comes out bounding nothing, as
    /// [`Assembly::with_each_ring_of_two_straight_sides_left_out`] lists, or if
    /// a run of edges that became one edge does not have exactly two ends left.
    /// Both mean the joining above is wrong.
    fn rebuild_from(&mut self, joined: &[JoinedFace], renumbered: Renumbered) -> Result<bool> {
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
        let (edges, faces, left_a_ring_out) =
            Self::with_each_ring_of_two_straight_sides_left_out(edges, faces);

        self.corners = core::mem::take(&mut self.corners)
            .into_iter()
            .zip(&renumbered.corner_now)
            .filter_map(|(corner, now)| now.map(|_| corner))
            .collect();
        self.edges = edges;
        self.faces = faces;
        Ok(left_a_ring_out)
    }

    /// The joined faces with every ring of a flat face that is two straight
    /// sides left out, and the two edges of each such ring made one, then
    /// whether there was any such ring.
    ///
    /// **Two straight sides between the same two corners are one segment** as
    /// the answer's corners are written, so a ring of them goes out along it
    /// and back and encloses nothing. That is §365.7's rule for a piece's wire
    /// ([`encloses_something`]), met here by a ring the joining makes out of
    /// pieces that each enclose something. A turned frame with a block flush
    /// beneath it makes one: over the turned planes the block's front sits
    /// `2.8e-17` outside the frame's front, the union keeps the strip of the
    /// block's top between the two, and the frame's wall at `x = 4` cuts that
    /// strip into two triangles. Their corners on that wall are one ulp apart
    /// and stay two corners, while the strip's ends round onto one corner each
    /// ([`Corners::of`]). Joined, the two triangles are one ring of two sides
    /// from one end to the other, one along each front, once the two corners
    /// on the wall are walked straight through
    /// (`a_turned_frame_and_a_block_flush_beneath_it.rs`, queue row Q1426).
    ///
    /// **Making the two edges one keeps every edge's faces paired.** Every edge
    /// has two faces along it, walking it opposite ways, and such a ring walks
    /// each of its two edges once, one each way along the segment. So `k` edges
    /// made one by `k - 1` rings keep two uses between them, one each way: the
    /// block's front and the frame's front come out on either side of one edge
    /// between the strip's two ends, and the joining runs again to make them
    /// one face (§426, queue row Q1446). More rings than that would leave the edge
    /// with no face along it, which [`Assembly::group_into_shells`] refuses as
    /// not a manifold. The edge still runs straight, since
    /// [`Edge::how_it_runs`] reads two flat surfaces and two different ends as
    /// straight whichever planes they are, and its two ends keep every other
    /// edge they had.
    ///
    /// **Which edge stays is chosen off what it runs along**, the lowest pair of
    /// [`name_of_surface_bits`] it is carried by, and never off its index here,
    /// which depends on the order the solids were handed over.
    ///
    /// # Panics
    ///
    /// If a flat face loses its outline and keeps a hole, or if a ring comes
    /// out bounding nothing: fewer than three sides, unless it is one side that
    /// closes on itself, two different sides not both straight, or a ring of a
    /// whole closed surface. Both are broken internal invariants and a corrupt
    /// solid is worse than a crash (§4.1).
    fn with_each_ring_of_two_straight_sides_left_out(
        edges: Vec<Edge>,
        mut faces: Vec<Face>,
    ) -> (Vec<Edge>, Vec<Face>, bool) {
        let one_segment = Self::leave_out_each_ring_of_two_straight_sides(&edges, &mut faces);
        let left_a_ring_out = one_segment.joined_anything();
        let (edges, faces) = if left_a_ring_out {
            Self::each_segment_as_one_edge(edges, faces, &one_segment)
        } else {
            (edges, faces)
        };
        for face in &faces {
            for ring in &face.wires {
                assert!(
                    ring.len() >= 3
                        || a_ring_of_one_or_two_sides_bounds_something(ring, &edges)
                        || face.is_a_whole_closed_surface,
                    "a face has at least three sides, one that closes, or two not both \
                     straight; this one has {}",
                    ring.len()
                );
            }
        }
        (edges, faces, left_a_ring_out)
    }

    /// Take each ring of two straight sides out of its flat face, and answer
    /// which edges those rings say are one segment.
    fn leave_out_each_ring_of_two_straight_sides(edges: &[Edge], faces: &mut [Face]) -> Sameness {
        let mut one_segment = Sameness::of(edges.len());
        for face in faces
            .iter_mut()
            .filter(|face| face.surface.as_plane().is_some())
        {
            let left_out: Vec<bool> = face
                .wires
                .iter()
                .map(|ring| match &ring[..] {
                    [one, other] => {
                        one.edge != other.edge
                            && edges[one.edge].lies_on_two_planes()
                            && edges[other.edge].lies_on_two_planes()
                    }
                    _ => false,
                })
                .collect();
            assert!(
                !left_out[0] || left_out.iter().all(|&out| out),
                "a flat face whose outline encloses nothing has a hole that does"
            );
            face.wires = core::mem::take(&mut face.wires)
                .into_iter()
                .zip(left_out)
                .filter_map(|(ring, out)| {
                    if out {
                        one_segment.join(ring[0].edge, ring[1].edge);
                        None
                    } else {
                        Some(ring)
                    }
                })
                .collect();
        }
        one_segment
    }

    /// The edges with each class `one_segment` holds made one edge, and every
    /// side and face pointed at what is left. A face left with no ring goes.
    fn each_segment_as_one_edge(
        mut edges: Vec<Edge>,
        mut faces: Vec<Face>,
        one_segment: &Sameness,
    ) -> (Vec<Edge>, Vec<Face>) {
        let mut classes: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for edge in 0..edges.len() {
            classes
                .entry(one_segment.leader(edge))
                .or_default()
                .push(edge);
        }
        let mut kept_for: Vec<usize> = (0..edges.len()).collect();
        for class in classes.values_mut().filter(|class| class.len() > 1) {
            class.sort_by_key(|&edge| {
                let mut names = edges[edge]
                    .carried_by
                    .map(|surface| name_of_surface_bits(&surface));
                names.sort_unstable();
                names
            });
            for &edge in class.iter() {
                kept_for[edge] = class[0];
            }
            edges[class[0]].from = each_one_once(
                class
                    .iter()
                    .flat_map(|&edge| edges[edge].from.iter().copied()),
            );
        }
        let now = numbered((0..edges.len()).map(|edge| kept_for[edge] == edge));
        for side in faces
            .iter_mut()
            .flat_map(|face| face.wires.iter_mut())
            .flatten()
        {
            let kept = kept_for[side.edge];
            if kept != side.edge {
                let starts_at = edges[side.edge].ends[usize::from(!side.forward)];
                side.forward = edges[kept].ends[0] == starts_at;
            }
            side.edge = now[kept].expect("the edge each class keeps is numbered");
        }
        faces.retain(|face| !face.wires.is_empty());
        let edges = edges
            .into_iter()
            .zip(&now)
            .filter_map(|(edge, now)| now.map(|_| edge))
            .collect();
        (edges, faces)
    }

    /// Leave out each flat face whose one ring of straight sides one other flat
    /// face, on a different plane, walks back the other way, and take that
    /// stretch out of the other face's ring.
    ///
    /// **The straight sides that two faces on different planes share all lie on
    /// the one line where the two planes cross**, so in exact geometry a ring of them
    /// goes out along that line and back and encloses nothing. Only rounding
    /// makes one that encloses something. A cube tipped onto one edge along
    /// another cube's floor makes one: the floor cuts the tipped cube's bottom
    /// face along a line one rounding off the cube's own edge, and
    /// [`Corners::of`] keeps the meet `(4.999999999999999, 5, 0)` of the floor,
    /// the bottom face and the side `x = 5` apart from the cube's corner
    /// `(5, 5.000000000000001, 0)`. The union keeps the sliver of the bottom
    /// face between the two lines, a triangle with the floor on the other side
    /// of all three of its sides, and the floor, joined, walks the same three
    /// corners back as a notch in its outline. Left in, the two corners off the
    /// floor's edge each had those two edges and nothing else, so
    /// [`Assembly::what_is_left`] walked straight through both, and the three
    /// edges folded into one straight edge from the sliver's third corner back
    /// to itself (`a_cube_tipped_onto_one_edge_along_another_cubes_floor.rs`,
    /// queue row Q1506).
    ///
    /// **Leaving both out changes no volume.** The face's ring and the other
    /// face's stretch walk the same corners in opposite directions, so as the
    /// answer's corners are written they bound no space between them. **And
    /// every edge left keeps its two faces**: each edge of the ring loses both
    /// of its uses, the face's and the stretch's, so it is marked dissolved and
    /// goes, and every other edge keeps its own. A corner with no edge left goes
    /// with them. The same rule lets the tipped cube less the block answer,
    /// refused `VolumeIsNotPositive` before, and the unions of three of
    /// Q1208's random turns of a cube with its corner on a block's face, turns
    /// 8, 21 and 49, refused the same way
    /// (`a_turned_cube_with_its_corner_on_a_face.rs`).
    ///
    /// It is the fourth rule of its kind, after `docs/decisions.md` §365.7's
    /// two, a side whose ends round to one corner and a wire that walks out
    /// and back along one edge, and §417.4's third, an edge only left-out
    /// wires walked, and it runs under every policy as they do. The face it
    /// leaves out can be part of the exact answer: the tipped cube's edge, as
    /// its two stored planes meet, lies about `6e-18` below the floor, so a
    /// wedge of the cube that thin hangs below it. No double lies inside the
    /// wedge, its corners round onto the three the ring walks, and §417.4
    /// takes an answer written as its corners round as right.
    ///
    /// Answers `true` when it left something out.
    ///
    /// # Errors
    ///
    /// [`Error::Undecidable`] when two planes cannot be told apart at all.
    fn leave_out_each_flat_ring_another_plane_walks_back(
        &self,
        joined: &mut Vec<JoinedFace>,
        along: &[[Dart; 2]],
        dissolved: &mut [bool],
        log: &mut ProvenanceLog,
    ) -> Result<bool> {
        let mut left_out = false;
        while let Some(found) = self.a_flat_ring_another_plane_walks_back(joined, along, log)? {
            for &dart in &joined[found.face].rings[0] {
                dissolved[self.edge_of(dart)] = true;
            }
            let other = &mut joined[found.by];
            other.rings[found.ring].retain(|dart| !found.stretch.contains(dart));
            if other.rings[found.ring].is_empty() {
                other.rings.remove(found.ring);
            }
            let mut gone = vec![found.face];
            if other.rings.is_empty() {
                gone.push(found.by);
            }
            gone.sort_unstable();
            for index in gone.into_iter().rev() {
                joined.remove(index);
            }
            left_out = true;
        }
        Ok(left_out)
    }

    /// The first flat face with one ring of straight sides that one other flat
    /// face, on a different plane, walks back as one unbroken stretch of one of
    /// its rings — and not the whole outline of a face with holes, which would
    /// leave the holes with nothing round them.
    ///
    /// # Errors
    ///
    /// [`Error::Undecidable`] when two planes cannot be told apart at all.
    fn a_flat_ring_another_plane_walks_back(
        &self,
        joined: &[JoinedFace],
        along: &[[Dart; 2]],
        log: &mut ProvenanceLog,
    ) -> Result<Option<WalkedBack>> {
        let mut joined_into: Vec<Option<usize>> = vec![None; self.faces.len()];
        for (index, face) in joined.iter().enumerate() {
            for &member in &face.members {
                joined_into[member] = Some(index);
            }
        }
        for (face, candidate) in joined.iter().enumerate() {
            let (Some(plane), [ring]) = (candidate.surface.as_plane(), &candidate.rings[..]) else {
                continue;
            };
            let Some(stretch) = self.the_darts_walking_back_along_straight_sides(ring, along)
            else {
                continue;
            };
            let by_face = |dart: &Dart| {
                joined_into[dart.face].expect("an edge that stays has a joined face on each side")
            };
            let by = by_face(&stretch[0]);
            if by == face || stretch.iter().any(|dart| by_face(dart) != by) {
                continue;
            }
            let Some(other_plane) = joined[by].surface.as_plane() else {
                continue;
            };
            let Some(in_ring) = one_unbroken_stretch_of_one_ring(&joined[by].rings, &stretch)
            else {
                continue;
            };
            let would_leave_holes_alone = in_ring == 0
                && joined[by].rings.len() > 1
                && joined[by].rings[0].len() == ring.len();
            if would_leave_holes_alone
                || agreement_of_two_planes(plane, other_plane, log)? != PlaneAgreement::Different
            {
                continue;
            }
            return Ok(Some(WalkedBack {
                face,
                by,
                ring: in_ring,
                stretch,
            }));
        }
        Ok(None)
    }

    /// The other dart along each side of `ring`, or `None` when a side of it is
    /// not straight.
    fn the_darts_walking_back_along_straight_sides(
        &self,
        ring: &[Dart],
        along: &[[Dart; 2]],
    ) -> Option<Vec<Dart>> {
        ring.iter()
            .map(|&dart| {
                let edge = self.edge_of(dart);
                let [one, other] = along[edge];
                self.edges[edge]
                    .lies_on_two_planes()
                    .then_some(if one == dart { other } else { one })
            })
            .collect()
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

        // **A corner where pieces of one cone, pipe or ball met stays a corner**, and
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
                [one, other]
                    if one != other
                        && !where_curved_pieces_met[corner]
                        && !by
                            .iter()
                            .any(|&edge| self.is_the_apex_of_a_cone_along(corner, edge)) =>
                {
                    straight_through.insert((lump, corner));
                }
                _ => still_a_corner[corner] = true,
            }
        }
        let kept = self.the_corners_each_closed_run_keeps(&leaving, &straight_through);
        let closed_a_curve_up = !kept.closed_up.is_empty();
        for (lump, corner) in kept.closed_up.into_iter().chain(kept.left_in_pieces) {
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

    /// Whether `corner` is the apex of a cone `edge` is carried by, which no
    /// lump walks straight through.
    ///
    /// **Two rulings of one cone are not one straight line**, though both run
    /// through the apex. A plane through a pointed cone's apex cuts it in two
    /// rulings, and where nothing else meets there the lump has only those two
    /// edges at the apex. Folded, they were one straight edge between their far
    /// ends, beside the cap's chord between the same two corners, and the
    /// shell was refused by the mesher that measures it
    /// (`a_block_face_through_a_cones_apex.rs`, `docs/decisions.md` §347.4).
    ///
    /// **The apex is never a point walked straight through**, so keeping it is
    /// never wrong. A face on a cone lies on one nappe (the arrangement's
    /// `part_the_two_nappes_at_the_apex` keeps it to one), and every edge of it
    /// that reaches the apex leaves it heading along a ruling of that nappe.
    /// Two rulings of one nappe are never opposite, since a cone's slope is
    /// finite.
    ///
    /// **The match is exact, not a direction test at the corner.** The
    /// arrangement puts the node where two rulings cross at the apex as the
    /// cone stores it, so that corner is the apex bit for bit, and
    /// [`Corners::of`] already takes two corners at the same bits to be one
    /// ([`exactly_at`]). A corner only near the apex has other bits, and is not
    /// kept on this ground. A test of whether the two edges leave the corner in
    /// opposite directions would read two rounded directions instead, and on a
    /// nearly flat cone, whose slope is large, the two rulings a plane through
    /// its axis cuts are nearly opposite.
    fn is_the_apex_of_a_cone_along(&self, corner: usize, edge: usize) -> bool {
        let at = exactly_at(self.corners[corner].point);
        self.edges[edge]
            .carried_by
            .iter()
            .any(|surface| match surface {
                Surface::Cone(cone) => exactly_at(cone.apex()) == at,
                Surface::Plane(_)
                | Surface::Cylinder(_)
                | Surface::Sphere(_)
                | Surface::Torus(_) => false,
                // A surface kind added to `rcad-geom` after this is not read
                // as having an apex.
                _ => false,
            })
    }

    /// The corners to keep on each run of edges that closes on itself with
    /// every corner along it walked straight through: one corner where the run
    /// is closed up, and every corner where it is left in pieces. Folding all
    /// of them would leave a joined edge with no ends at all.
    ///
    /// **A closed curve cut into pieces and joined back up** is such a run: a
    /// pointed cone's rim, cut where a block's walls are tangent to it and
    /// holding its own one corner, keeps two edges at every one of those
    /// corners once the pieces are joined (`fuses_that_touch.rs`,
    /// `a_pointed_cone_on_its_apex_whose_rim_touches_two_walls_fuses_by_every_rule`).
    /// The run becomes one closed edge with one corner, as the input had.
    /// Keeping every corner there instead leaves a ring of arcs going right
    /// round a cone with no seam, which the writer refuses
    /// (`AWireWillNotLieFlatOnItsFace`) — measured on that same pair.
    ///
    /// **Only a run that is all pieces of one input edge is closed up**, which
    /// is then a closed input edge given back whole. A ball's meridian circle
    /// cut at both poles is a closed run too, of pieces of two different edges,
    /// and closing it up puts a closed edge's one corner on a pole, where
    /// nothing says which way it runs: 22 tests across the ball-and-block files
    /// failed when every closed run was closed up.
    ///
    /// **Any other closed run is left in the pieces it was cut into**, every
    /// corner kept, which is what the answer is when nothing else in the fuse
    /// is joined and the joining is skipped altogether. A unit ball united with
    /// a block that a second block's plane `x = 0` reaches through a chain is
    /// cut by that plane through both poles, into two arcs from pole to pole
    /// that are pieces of no input edge, and the poles have nothing else along
    /// them. Folded, the two arcs were one edge with no ends, and that fuse
    /// panicked here wherever another face of it was joined
    /// (`a_ball_cut_through_both_poles_by_a_linked_blocks_plane.rs`,
    /// `docs/decisions.md` §400.5).
    ///
    /// Which corner a closed-up run keeps is chosen off what the corner *is*
    /// and never off its index here, which depends on the order the solids
    /// were handed over: an input vertex first, then the lowest point.
    fn the_corners_each_closed_run_keeps(
        &self,
        leaving: &BTreeMap<(usize, usize), Vec<usize>>,
        straight_through: &BTreeSet<(usize, usize)>,
    ) -> ClosedRuns {
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
        let mut kept = ClosedRuns {
            closed_up: Vec::new(),
            left_in_pieces: Vec::new(),
        };
        for (run, corners) in corners_of_run {
            if run_keeps_a_corner.contains(&run) {
                continue;
            }
            if is_one_input_edge(&pieces_of[&run]) {
                kept.closed_up.push(
                    corners
                        .into_iter()
                        .min_by(|&(_, one), &(_, other)| self.which_to_keep_first(one, other))
                        .expect("a run is listed with the corners it walks through"),
                );
            } else {
                kept.left_in_pieces.extend(corners);
            }
        }
        kept
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
            // A flat face's ring that another plane walks back made one until
            // `leave_out_each_flat_ring_another_plane_walks_back` left it out.
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
                read_against: None,
            });
        }
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
    /// has no shell around it. [`Error::AFaceTrimmedByACurveThisCannotFollow`]
    /// naming the input face, when no point on a shell could be placed and one
    /// met a face of another shell that nobody can follow.
    /// [`Error::Undecidable`] when no point on a shell could be placed inside or
    /// outside another for any other reason.
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
    /// **Each side is asked from the corner it starts at**, so every corner of
    /// every wire is asked. Asking the first end of each edge instead asked a
    /// ball's one pole twice, down its seam and back up it, and its other pole
    /// never: a ball touching a block's top with its north pole was refused for
    /// want of the south pole, which is inside
    /// (`tests/a_shell_inside_another_names_its_refusal.rs`).
    ///
    /// **Running out of corners is a refusal, and it says which one.** A corner
    /// whose ray met a face of `outer` that [`SolidView::face_holds_the_point`]
    /// cannot follow was never asked at all, and that is
    /// [`Containment::CannotFollowAFacesTrimming`], deliberately not
    /// `Undecidable` (`docs/decisions.md` §4.2, §35.6). This used to fold it
    /// into [`Error::Undecidable`] with a record of a point it never asked —
    /// the first corner of the whole answer — so a caller read "loosen the
    /// policy" off a refusal no policy moves. Since Q1002 a flat face the cuts
    /// cannot follow winds its own chords and arcs, so this refusal needs a
    /// hollow whose rays still meet a face nobody can follow; nothing in this
    /// crate's tests reaches it as of this writing (`docs/decisions.md`
    /// §35.6).
    ///
    /// **So is a corner whose ray could not be fired at a face on a ring**,
    /// [`Containment::CannotFireARayAtAFacesSurface`], which is refused
    /// [`Error::CannotFireARayAtAFacesSurface`] for the same reason: that
    /// question was never put either. It used to stay with the undecided
    /// corners, for want of a variant that said so. No fuse is known to reach
    /// it: a fuse with a ring in it crosses every surface of the other solid
    /// with the ring, wherever the two faces are, and row Q883's probes were
    /// refused there — a ball inside a bend or under the top half of a ring
    /// `ACrossingBetweenATorusAndAnotherCurvedSurface`, and a block grooved by
    /// a half ring `ACrossingBetweenAPlaneAndATorusThatIsAQuartic` for a wall
    /// five clear of the ring.
    ///
    /// **A face nobody can follow wins over a corner left undecided**, when
    /// both happen, and so does a face no ray could be fired at; the first of
    /// those two wins over the second. Either refusal is true, and only one can
    /// be returned; naming the face folds an undecided corner into "never
    /// asked", which costs a caller a policy that might have helped, where the
    /// other way round folds "never asked" into "undecided", which is the
    /// mistake §35.6 keeps the two apart to stop. The face named is the
    /// caller's: `apart` is thrown away when the fuse returns, so a key into it
    /// names nothing.
    ///
    /// **The record names the corners that were asked.** When every corner came
    /// back on the skin of `outer` or undecided, the one record says "one shell
    /// inside another" over each of those corners once, in the order they were
    /// asked. It used to name the first corner of the whole answer, which
    /// nothing asked: a block less a ball touching its top and bottom with its
    /// two poles named the block's corner at the origin. That refusal is still
    /// a gap, since the ball is a hollow of the block and a point of it off its
    /// poles would say so.
    ///
    /// # Errors
    ///
    /// [`Error::AFaceTrimmedByACurveThisCannotFollow`] naming the input face
    /// a face of `outer` was cut from, when a corner of `hollow` met that face
    /// and none was placed. [`Error::CannotFireARayAtAFacesSurface`] the same
    /// way, when a corner's ray could not be fired at a face of `outer` on a
    /// ring and none was placed. [`Error::Undecidable`] when every corner of
    /// `hollow` asked came back on the skin of `outer` or undecided.
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
        let mut unfollowed = None;
        let mut unfired = None;
        let mut not_placed = Vec::new();
        for &face in &self.shells[hollow] {
            for side in self.faces[face].wires.iter().flatten() {
                let corner = self.edges[side.edge].ends[usize::from(!side.forward)];
                let at = self.corners[corner].point;
                let mut trial = ProvenanceLog::new();
                match solid.classify_point(at, policy, &mut trial) {
                    Containment::Inside => {
                        trial.drain_into(log);
                        return Ok(true);
                    }
                    Containment::Outside => {
                        trial.drain_into(log);
                        return Ok(false);
                    }
                    Containment::CannotFollowAFacesTrimming { face } => {
                        unfollowed.get_or_insert(face);
                    }
                    Containment::CannotFireARayAtAFacesSurface { face } => {
                        unfired.get_or_insert(face);
                    }
                    Containment::OnBoundary | Containment::Undecidable { .. } => {
                        if !not_placed.contains(&corner) {
                            not_placed.push(corner);
                        }
                    }
                }
            }
        }
        if let Some(face) = unfollowed {
            return Err(Error::AFaceTrimmedByACurveThisCannotFollow {
                face: self.the_input_face_it_was_cut_from(apart, written, face),
            });
        }
        if let Some(face) = unfired {
            return Err(Error::CannotFireARayAtAFacesSurface {
                face: self.the_input_face_it_was_cut_from(apart, written, face),
            });
        }
        let asked: Vec<PredicatePoint> = not_placed
            .iter()
            .map(|&corner| PredicatePoint::from(self.corners[corner].point.coordinates()))
            .collect();
        Err(gave_up(
            ONE_SHELL_INSIDE_ANOTHER,
            &asked,
            Measures::Length,
            log,
        ))
    }

    /// The caller's face that `face`, written into `apart`, was cut out of.
    ///
    /// The first of its ancestors, where coplanar pieces were joined: any of
    /// them is a face the piece is part of, and the first is the one a fuse
    /// names elsewhere too.
    ///
    /// # Panics
    ///
    /// If `face` is not one [`Assembly::write_into`] wrote into `apart`, or the
    /// face it was written from has no ancestor. Both are this crate's own
    /// bookkeeping.
    fn the_input_face_it_was_cut_from(
        &self,
        apart: &Model,
        written: &Written,
        face: FaceKey,
    ) -> FaceKey {
        let slot = face.slot_in(apart.id());
        let index = written
            .faces
            .iter()
            .position(|&each| each == slot)
            .expect("every face of `apart` was written from a face of the answer");
        self.faces[index].ancestors[0]
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
        let cut_by = self.the_surface_that_cut_each_edge_its_faces_do_not_name();
        self.edges
            .iter()
            .zip(cut_by)
            .map(|(edge, cut_by)| edge.how_it_runs(&self.corners, cut_by))
            .collect()
    }

    /// Which surface cut each edge whose two faces **in the answer** sit on the
    /// one surface and so name nothing between them, closed or not.
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
    /// **An arc between two pieces of one surface names the other surface as
    /// well.** The half frustum `x ≥ 0` united with a bar of radius 4 whose
    /// axis is the cone's keeps the bar's wall above and below the arc at
    /// `x < 0` of the circle where the cone crosses the pipe, in two pieces,
    /// with that arc between them. [`Edge::the_surface_that_names_it_twice`]
    /// put the pipe in its run's `and`, since a pipe sorts ahead of a cone, and
    /// a pipe either side and a pipe in `and` name nothing: the walk put no
    /// point between the arc's ends, the mesher read the half turn between them
    /// the wrong way round, and the piece above was refused
    /// [`Error::ACurvedFaceGoesRightRoundWithNoSeam`]
    /// (`a_cone_hollowed_along_its_axis.rs`, Q1338).
    ///
    /// **One surface is asked of the spelling folded away**
    /// ([`name_of_the_surface_either_way_round`]), never of the bits: two bars
    /// stacked on one pipe, one written with its axis pointing down, have two
    /// spellings of one wall either side of the circle between them, and read
    /// bit for bit that is two pipes crossing — which `rcad_geom::walk` then
    /// walks as a straight line with no points on it
    /// (`two_bars_on_one_pipe_answer_the_same_however_the_pipe_is_written`).
    ///
    /// **A rim between two different curved faces names the plane its edge was
    /// cut out of**, where its edge was named by one:
    /// [`the_plane_of_a_rim_between_two_curved_faces`] says why the two faces
    /// alone may name nothing to follow.
    ///
    /// `None` everywhere else, which is where [`EdgeRun::RightRoundWhereItsFacesCross`]
    /// stays for a rim, and [`Edge::the_surface_that_names_it_twice`] is asked
    /// for an arc. Three shapes come back `None` for a reason worth naming: an
    /// edge that is not closed between two different surfaces, which name it
    /// themselves; an edge other than two faces claim, which
    /// [`Assembly::group_into_shells`] refuses as [`Error::ResultIsNotManifold`]
    /// a step later under its own name rather than here; and a crossing whose
    /// pair does not name one surface of the answer and one other, which leaves
    /// nothing to put in `and`.
    fn the_surface_that_cut_each_edge_its_faces_do_not_name(&self) -> Vec<Option<Surface>> {
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
                let [one, other] = <[usize; 2]>::try_from(by.as_slice()).ok()?;
                let either_side = self.faces[one].surface;
                if !one_surface(&self.faces[other].surface, &either_side) {
                    if edge.ends[0] != edge.ends[1] {
                        return None;
                    }
                    return the_plane_of_a_rim_between_two_curved_faces(
                        [either_side, self.faces[other].surface],
                        &edge.carried_by,
                    );
                }
                match edge.carried_by {
                    [met, across]
                        if one_surface(&met, &either_side)
                            && !one_surface(&across, &either_side) =>
                    {
                        Some(across)
                    }
                    [across, met]
                        if one_surface(&met, &either_side)
                            && !one_surface(&across, &either_side) =>
                    {
                        Some(across)
                    }
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

/// The plane a closed rim between two different curved faces was cut out of,
/// where its edge was named by one: the `and` its run carries.
///
/// **Two curved faces either side of a rim may touch rather than cross.** A
/// ball of a bar's radius centred on the bar's top cap is tangent to the pipe
/// all the way round the rim, so the union keeps the pipe below the rim and
/// the ball above it, and `rcad_geom::walk` has no crossing of the two to
/// follow: it refused the rim
/// [`CannotFollowACrossingWhereItsSurfacesAreTangent`](rcad_geom::Error). The
/// cap's plane cuts each of them in that one circle, and a run naming it is
/// walked as the plane against a face.
///
/// **Only an edge whose name came under more than one pair reaches here with a
/// plane in it.** The edge carries the pair of the first side that named it,
/// and a side of a curved face names that face's surface and its partner. Two
/// different curved faces whose sides are named alike are therefore each other's
/// partners, with no plane between them, unless
/// [`the_name_each_closed_curve_goes_by`] joined the names of two pairs, and it
/// joins only a plane against a ball, a pipe or a cone whose one closed curve
/// the other pair shares. So the plane here crosses each face in this rim.
fn the_plane_of_a_rim_between_two_curved_faces(
    faces: [Surface; 2],
    carried_by: &[Surface; 2],
) -> Option<Surface> {
    if faces.iter().any(|surface| surface.as_plane().is_some()) {
        return None;
    }
    carried_by
        .iter()
        .find(|surface| surface.as_plane().is_some())
        .copied()
}

/// An arc of a ring, the surface it runs along, and whether it bulges
/// to the left of the chord from its first end to its second.
struct Bulging {
    ends: [[f64; 3]; 2],
    surface: Surface,
    left: bool,
    /// Where the plane cuts `surface`, a cone, in a parabola or a hyperbola,
    /// the cone and the nappe the arc is on; `None` for an arc of a curve
    /// that closes.
    nappe: Option<(Cone, Decision)>,
}

/// How each arc of `ring` bulges against its chord, for
/// [`Assembly::holds_the_other_rings`] to wind a point round; `None` where an
/// arc's surface, its bulge, or the ring itself cannot be placed.
fn how_each_arc_bulges(
    assembly: &Assembly,
    ring: &[Dart],
    plane: &Plane,
    normal: [f64; 3],
    the_curved_one: impl Fn(Dart) -> Option<Surface>,
) -> Option<Vec<Option<Bulging>>> {
    ring.iter()
        .map(|&dart| {
            let side = assembly.side_of(dart);
            let [from, to] = [assembly.starts_at(dart), assembly.stops_at(side)];
            if from == to {
                return None;
            }
            let WhichArc::LeavingACorner { corner, towards } = assembly.edges[side.edge].arc else {
                return Some(None);
            };
            let surface = the_curved_one(dart)?;
            let nappe = match surface {
                Surface::Cylinder(_) | Surface::Sphere(_) => None,
                Surface::Cone(_)
                    if where_a_closed_crossing_is_centred(&surface, plane).is_some() =>
                {
                    None
                }
                Surface::Cone(cone) => {
                    let ends = [from, to].map(|at| assembly.corners[at].point);
                    let on = the_nappe_an_arc_that_does_not_close_is_on(&cone, plane, ends)?;
                    Some((cone, on))
                }
                Surface::Plane(_) | Surface::Torus(_) => return None,
                // A kind added to `rcad-geom` after this has not been shown
                // to bound the region inside it, so an arc along it is not
                // counted.
                _ => return None,
            };
            let ends = [from, to].map(|at| assembly.corners[at].point.coordinates());
            let into_the_arc = (assembly.corners[corner].point + towards).coordinates();
            let left = match winding_round(&[ends[0], ends[1], into_the_arc], normal).ok()? {
                Decision::Pos => true,
                Decision::Neg => false,
                Decision::Zero | Decision::Undecidable { .. } => return None,
                other => panic!("which side of its chord an arc bulges came back as {other:?}"),
            };
            Some(Some(Bulging {
                ends,
                surface,
                left,
                nappe,
            }))
        })
        .collect::<Option<_>>()
}

/// How many times `arcs` winds round the point `at`, or `None` where an arc's
/// surface cannot place it.
fn turns_round(arcs: &[Option<Bulging>], normal: [f64; 3], at: [f64; 3]) -> Option<i32> {
    arcs.iter()
        .flatten()
        .map(|arc| {
            match crate::validate::side_of_surface(&arc.surface, PredicatePoint::from(at)) {
                Decision::Pos | Decision::Zero => return Some(0),
                Decision::Neg => (),
                Decision::Undecidable { .. } => return None,
                other => panic!(
                    "which side of a surface a corner is on came back as a decision \
                     nothing here knows: {other:?}"
                ),
            }
            if let Some((cone, nappe)) = &arc.nappe {
                let found = the_nappe_of(cone, PredicatePoint::from(at).enclosure());
                match found {
                    Decision::Pos | Decision::Neg | Decision::Zero if found != *nappe => {
                        return Some(0);
                    }
                    Decision::Pos | Decision::Neg | Decision::Zero => (),
                    Decision::Undecidable { .. } => return None,
                    other => panic!("which nappe a corner is on came back as {other:?}"),
                }
            }
            let left = match winding_round(&[arc.ends[0], arc.ends[1], at], normal).ok()? {
                Decision::Pos => true,
                Decision::Neg => false,
                Decision::Zero | Decision::Undecidable { .. } => return None,
                other => {
                    panic!("which side of a chord a corner is on came back as {other:?}")
                }
            };
            Some(match (left == arc.left, arc.left) {
                (false, _) => 0,
                (true, true) => -1,
                (true, false) => 1,
            })
        })
        .sum()
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
    /// [`Assembly::the_surface_that_cut_each_edge_its_faces_do_not_name`] says
    /// why. A rim that comes with one names it rather than its two faces, and an
    /// arc puts it in `and` in place of
    /// [`Edge::the_surface_that_names_it_twice`].
    ///
    /// # Errors
    ///
    /// [`Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed`] for an arc of a
    /// curved crossing that nothing told from its sibling.
    fn how_it_runs(&self, corners: &[Corner], cut_by: Option<Surface>) -> Result<EdgeRun> {
        match (self.lies_on_two_planes(), self.ends[0] == self.ends[1]) {
            (true, false) => Ok(EdgeRun::Straight),
            (_, true) => Ok(match cut_by {
                Some(and) => EdgeRun::RightRoundWhereItCrosses { and },
                None => EdgeRun::RightRoundWhereItsFacesCross,
            }),
            (false, false) => match self.arc {
                WhichArc::TheOnlyOne => Ok(EdgeRun::Straight),
                WhichArc::LeavingACorner { corner, towards } => {
                    let and = cut_by.unwrap_or_else(|| self.the_surface_that_names_it_twice());
                    let ends = self.ends.map(|end| corners[end].point);
                    if let Some(branch) = self.the_branch_of_a_hyperbola_this_is_on(ends)? {
                        return Ok(EdgeRun::PartOfWhereItCrosses {
                            and,
                            picked_by: SmallVec::from_buf([branch]),
                        });
                    }
                    let facing = self
                        .the_way_across_the_chord_into_the_arc(corners, corner, towards)
                        .ok_or(Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed)?;
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
                        and,
                        picked_by: SmallVec::from_buf([picker]),
                    })
                }
            },
        }
    }

    /// Whether both surfaces it runs between are flat, which makes it a straight
    /// line wherever its two ends are different corners.
    fn lies_on_two_planes(&self) -> bool {
        self.carried_by
            .iter()
            .all(|surface| surface.as_plane().is_some())
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

    /// The direction square to the chord, in the plane the arc lies in, that
    /// points from the chord into this arc — the `facing` of
    /// [`the_half_space_both_ends_are_on`] — or `None` when the arithmetic
    /// cannot say which side of the chord the arc is on.
    ///
    /// **Worked out at the same end of the arc whichever face named it first.**
    /// `towards` comes from whichever of the two faces along this edge reached
    /// it first, leaving the corner that face walked away from, and which face
    /// that is follows the order of the fuse's arguments. Worked out at either
    /// end, `v(c·c) − c(c·v)` is the same vector for a circle — the tangent
    /// makes the same angle with the chord at both ends — but the two sums
    /// round differently, and the picker that comes out of them differs in its
    /// last bits. Swapping the two sides of a certified intersection of a
    /// six-faced block and a ball then gave two answers that were the same
    /// solid and not the same numbers: `docs/decisions.md` §312.
    ///
    /// **The end is the one whose stored point comes first**, compared as
    /// numbers, so that `-0.0` and `0.0` choose alike. It is the one thing about
    /// the two ends that both orders agree on bit for bit: which corner has the
    /// lower index follows the order the pieces were walked in. Nothing is
    /// joined on the comparison; it only picks which end to start from.
    ///
    /// **Which side of the chord is the one thing taken from `towards`.** The
    /// named direction, squared off against the chord the same way, points to
    /// the arc's side of it whether `towards` is a tangent or, for a run of
    /// pieces joined into one ([`Assembly::arc_of_a_joined_run`]), the step to a
    /// corner the joining dissolved — and the crossing's direction at the chosen
    /// end, squared off, points to one side or the other. Their dot says which.
    /// Reading the tangent's sign off `towards · (∇S × ∇T)` instead is wrong for
    /// the joined case: the step to a point far round an arc leans back against
    /// the tangent, and a frustum through a block got both arcs of one circle
    /// the same picker that way (§312.6).
    ///
    /// A dot of exactly zero is `None`: either the named direction lay along
    /// its chord, which the old code refused too, or the crossing's direction
    /// at the chosen end does.
    fn the_way_across_the_chord_into_the_arc(
        &self,
        corners: &[Corner],
        named_from: usize,
        towards: Vector,
    ) -> Option<Vector> {
        let squared_off_from = |corner: usize, direction: Vector| {
            let chord = corners[self.other_end_than(corner)].point - corners[corner].point;
            direction * chord.dot(chord) - chord * chord.dot(direction)
        };
        let [one, other] = self.ends;
        let from = if corners[one].point.coordinates() <= corners[other].point.coordinates() {
            one
        } else {
            other
        };
        let at = corners[from].point;
        let [mine, partner] = self.carried_by;
        let across = squared_off_from(
            from,
            gradient_of(&mine, at).cross(gradient_of(&partner, at)),
        );
        let agreement = across.dot(squared_off_from(named_from, towards));
        if agreement > 0.0 {
            Some(across)
        } else if agreement < 0.0 {
            Some(-across)
        } else {
            None
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
    /// ball's seam — and names the second one. An arc between two different
    /// faces names both already, so either is a true answer, and what picks
    /// between them is a check: `Model::validate` asks that both ends are not
    /// *provably* off `and`, and a flat `and` is the cheapest of the three to be
    /// sure about. So the flat one goes in wherever there is one, and
    /// `name_of_surface_bits` gives that for free by sorting a plane ahead of a
    /// ball or a pipe. An arc between two pieces of one surface is not asked
    /// here: either may be the one its faces sit on, and
    /// [`Assembly::the_surface_that_cut_each_edge_its_faces_do_not_name`]
    /// hands it the other.
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
    /// For a side of a **closed** edge made in [`Edges::of`], what `forward`
    /// was read against. [`every_closed_edge_read_off_one_spelling`] reads it.
    /// `None` for every other side.
    read_against: Option<ReadAgainst>,
}

/// What [`Edges::of`] read a closed side's way round against, kept so that
/// [`every_closed_edge_read_off_one_spelling`] can read the side again against
/// what the side across was read against.
#[derive(Clone, Copy)]
struct ReadAgainst {
    /// The curved one of the piece's own crossing pair, which need not be a
    /// surface of either face the answer keeps along the edge. A disc cut from
    /// one bar is read against that bar's pipe, and the edge is stored by the
    /// other bar's wall.
    surface: Surface,
    /// The gradients of the piece's crossing pair at the edge's corner, as
    /// [`which_way_a_closed_crossing_turns`] takes them: the crossing the side
    /// walks.
    crossing: [Vector; 2],
    /// The gradient and the axis that crossing was read against: `surface`'s
    /// own, or at a ball's pole the ball's and its lid's way out
    /// ([`the_lid_a_circle_from_a_pole_bounds`]).
    frame: [Vector; 2],
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
    /// had the same flat region on both sides of it, or a ring another plane
    /// walks back, and is gone.
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

/// The corners kept on the runs of edges that close on themselves with every
/// corner walked straight through, as lump and corner.
struct ClosedRuns {
    /// One corner of each run that becomes one closed edge again.
    closed_up: Vec<(usize, usize)>,
    /// Every corner of each run that stays in the pieces it was cut into.
    left_in_pieces: Vec<(usize, usize)>,
}

/// One face of the result while it is being put back together out of the pieces
/// that turned out to be one flat region.
struct JoinedFace {
    /// The faces it was made of, in the order they were built.
    members: Vec<usize>,
    surface: Surface,
    used: Orientation,
    /// Carried through the joining untouched: a whole closed surface is a ball
    /// with no edge on it to be joined across, and pieces that join up into a
    /// whole ball are left apart ([`Assembly::edges_a_curved_face_keeps`]).
    is_a_whole_closed_surface: bool,
    /// On a plane, the outline first and then one ring per hole. On a cone, a
    /// pipe or a ball, in the order they were first come to (§31.3).
    rings: Vec<Vec<Dart>>,
}

/// A flat joined face whose one ring another flat face walks back
/// ([`Assembly::leave_out_each_flat_ring_another_plane_walks_back`]).
struct WalkedBack {
    /// The joined face whose one ring it is.
    face: usize,
    /// The joined face that walks it back.
    by: usize,
    /// Which ring of `by` holds the stretch.
    ring: usize,
    /// The darts of `by` along the ring's sides, one per side.
    stretch: Vec<Dart>,
}

/// Which of `rings` holds every one of `darts` as one unbroken stretch, going
/// round the ring, or the whole ring.
fn one_unbroken_stretch_of_one_ring(rings: &[Vec<Dart>], darts: &[Dart]) -> Option<usize> {
    let (index, ring) = rings
        .iter()
        .enumerate()
        .find(|(_, ring)| ring.contains(&darts[0]))?;
    let at: Vec<bool> = ring.iter().map(|dart| darts.contains(dart)).collect();
    let held = at.iter().filter(|&&held| held).count();
    let starts = (0..ring.len())
        .filter(|&position| at[position] && !at[(position + ring.len() - 1) % ring.len()])
        .count();
    (held == darts.len() && (starts == 1 || held == ring.len())).then_some(index)
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

/// Whether two curved faces are pieces of one cone, one pipe or one ball, used
/// the same way round, so that an edge between them is not boundary at all.
///
/// **A pipe is one pipe however it is written**: its axis turned round, or its
/// origin slid along it where [`name_of_the_surface_either_way_round`] folds
/// that exactly. Two bars stacked on one pipe, the upper one written in its own
/// frame, leave the wall as two spellings either side of the circle between
/// them. Left apart, that circle is an edge with the one surface on both sides,
/// which every later fuse of the answer has to bake the two faces as one to get
/// past (`docs/decisions.md` §231.5, §325.5, `faces_baked_as_one` in
/// [`bake`](super::bake)). Joined, the face takes the lowest of
/// its spellings ([`Assembly::rings_of_each_joined_face`]) and every closed
/// side read off another is read again
/// ([`Assembly::turn_round_each_closed_edge_the_joined_spelling_stores_the_other_way`]).
/// `used` means the same thing for every spelling, because a pipe's gradient
/// ([`gradient_of`]) does not change sign when its axis is turned round or its
/// origin slid.
///
/// **A cone still goes by its bits** (§224.2 item 1): a piece inherits its
/// surface from its ancestor bit for bit (§31.3), so two pieces of one input
/// face carry the same bits. A cone turned round is left apart until
/// `intersect` calls it one surface, which it does not yet.
///
/// The same surface used the other way round is a sheet of no thickness with
/// material either side, which is two faces and not one.
///
/// **A ball is one ball by its centre and radius**, which is all it holds and
/// all [`one_surface`] compares, so it has one spelling and nothing to turn
/// round. Its pieces join with no ring counted at all
/// ([`Assembly::edges_a_curved_face_keeps`], §229.2).
///
/// **Not a ring.** A cone's and a pipe's rings are asked how often they go
/// round the axis ([`Assembly::a_ring_goes_right_round`]), and a ring has two
/// ways round with no count here for either, so pieces of one stay apart.
fn one_cone_pipe_or_ball_the_same_way_round(mine: &Face, theirs: &Face) -> bool {
    let one = match mine.surface {
        Surface::Cylinder(_) | Surface::Sphere(_) => one_surface(&mine.surface, &theirs.surface),
        Surface::Cone(_) => {
            name_of_surface_bits(&mine.surface) == name_of_surface_bits(&theirs.surface)
        }
        Surface::Plane(_) | Surface::Torus(_) | _ => false,
    };
    one && mine.used == theirs.used
}

/// Whether two surfaces are one point set, however each is written.
pub(super) fn one_surface(one: &Surface, other: &Surface) -> bool {
    name_of_the_surface_either_way_round(one) == name_of_the_surface_either_way_round(other)
}

/// Turn round each side of a closed edge that was read off another spelling of
/// the surface the answer stores the edge by, or off another surface that runs
/// the other way round the edge.
///
/// [`Edges::of`] reads which way a side walks a closed edge off the curved
/// surface of that side's own crossing pair ([`Side::read_against`]), and a
/// pipe, a cone or a bend written with its axis the other way round is wound
/// about the other way. Two places that goes wrong, both `CoedgesRunTheSameWay`:
///
/// - **Two curved faces on two spellings of one surface**: each side is read
///   against its own face's winding, and a union of two bars stacked on one
///   pipe came back with both walls walking the circle between them
///   "backwards" (`docs/decisions.md` §231.3).
/// - **A flat face against a wall on another spelling**: a disc cut from one
///   bar is read against that bar's pipe while the wall across it, from the
///   other bar, is what the edge is stored by. Two bars overlapping on one pipe
///   written opposite ways, intersected or differenced, have that rim (§359).
///
/// So the surface compared is the one each side was read against, not its
/// face's.
///
/// **The spelling that counts is the curved one of the two faces' surfaces**,
/// the lower by bits where both are curved ([`the_curved_one_of`]). It is what
/// `Model::validate` reads the edge's way back off, and with two curved faces it
/// is what the mesher reads too (`the_way_a_closed_edge_where_its_faces_touch_runs`
/// in `mesh.rs`: the two gradients are parallel, so the crossing gives no
/// direction and the winding of that one surface is all there is). That is a
/// property of the two faces and not of which was handed over first, so the
/// answer does not depend on argument order.
///
/// **A side read against another spelling of that surface is turned by the
/// two windings.** Whether a spelling winds the other way is exact: turning an
/// axis round negates every product [`the_way_it_is_parametrised_about`] is
/// made of, so the two windings are exact negatives of each other and their
/// dot product is never a rounding away from zero.
///
/// **A side read against a different surface altogether is read again against
/// what the side across was read against, where that is the reference
/// itself** ([`runs_the_other_way_read_against`]). Shapes reach that since a
/// rim can come under two names ([`the_name_each_closed_curve_goes_by`]):
/// uniting the dome on a shaft of its radius reads the pipe's side against the
/// pipe and the ball's against the ball, and a frustum on a bar of its narrow
/// radius the cone's against the cone. Upright, the pipe's longitude and the
/// ball's run the same way round the rim, and the two readings agree. **Laid on
/// its side they do not** (Q1377): the rim is then a meridian of the ball, whose
/// longitude is square to it, so the ball's side is read against its latitude,
/// or at a pole against its lid, and along `+y` and `±x` that runs the other way
/// from the pipe's longitude. Left as read, both faces walked the rim the same
/// way, and 42 of the 72 fuses of `fuses_that_touch.rs`'s dome lying on its side
/// were refused `CoedgesRunTheSameWay` or `FacesDoNotAgreeWhichWayTheyFace`. A
/// side read against neither the reference nor a spelling of it, beside another
/// such side, is left as it was read.
///
/// # Errors
///
/// [`Error::CannotTellWhichWayAClosedEdgeRuns`] where a side read again cannot
/// be told.
///
/// # Panics
///
/// If a side of a closed edge does not say what it was read against. Every
/// side here was made by [`Edges::of`], which says it for each closed side.
fn every_closed_edge_read_off_one_spelling(
    edges: &[Edge],
    corners: &[Corner],
    faces: &mut [Face],
) -> Result<()> {
    let mut claimants: Vec<Vec<(usize, usize, usize)>> = vec![Vec::new(); edges.len()];
    for (face, walked) in faces.iter().enumerate() {
        for (wire, ring) in walked.wires.iter().enumerate() {
            for (position, side) in ring.iter().enumerate() {
                claimants[side.edge].push((face, wire, position));
            }
        }
    }
    for (edge, by) in edges.iter().zip(&claimants) {
        let &[one, other] = by.as_slice() else {
            continue;
        };
        if edge.ends[0] != edge.ends[1] {
            continue;
        }
        let Some(reference) =
            the_curved_one_of(&[one, other].map(|(face, _, _)| faces[face].surface))
        else {
            continue;
        };
        let at = corners[edge.ends[0]].point;
        let wound = the_way_it_is_parametrised_about(&reference, at);
        let reads = [one, other].map(|(face, wire, position)| {
            faces[face].wires[wire][position]
                .read_against
                .expect("Edges::of says what each closed side was read against")
        });
        let the_reference_itself = |read: &ReadAgainst| {
            name_of_surface_bits(&read.surface) == name_of_surface_bits(&reference)
        };
        for (index, (face, wire, position)) in [one, other].into_iter().enumerate() {
            let (read, across) = (&reads[index], &reads[1 - index]);
            let turn = if the_reference_itself(read) {
                false
            } else if one_surface(&read.surface, &reference) {
                the_way_it_is_parametrised_about(&read.surface, at).dot(wound) < 0.0
            } else if the_reference_itself(across) {
                runs_the_other_way_read_against(read, across)?
            } else {
                false
            };
            if turn {
                let side = &mut faces[face].wires[wire][position];
                side.forward = !side.forward;
            }
        }
    }
    Ok(())
}

/// Whether the crossing `read` walks runs the other way round against its own
/// frame from the way it runs against `across`'s.
///
/// Read with [`which_way_a_closed_crossing_turns`], the rule both sides were
/// read with, so this is the side read again against the other frame and not a
/// second rule.
///
/// `false` where `across`'s frame says nothing about this crossing: square to
/// both its legs, the crossing would run along the gradient of a surface the
/// edge lies on. Nothing is turned there, and the sweep reads the edge back.
///
/// # Errors
///
/// [`Error::CannotTellWhichWayAClosedEdgeRuns`] where either reading cannot be
/// told.
///
/// # Panics
///
/// If [`which_way_a_closed_crossing_turns`] grows a fifth answer, as
/// [`which_way_a_closed_edge_is_walked`] says.
fn runs_the_other_way_read_against(read: &ReadAgainst, across: &ReadAgainst) -> Result<bool> {
    let way = |[gradient, axis]: [Vector; 2]| {
        which_way_a_closed_crossing_turns(read.crossing, gradient, axis)
    };
    match (way(read.frame), way(across.frame)) {
        (Decision::Pos, Decision::Neg) | (Decision::Neg, Decision::Pos) => Ok(true),
        (Decision::Pos, Decision::Pos)
        | (Decision::Neg, Decision::Neg)
        | (Decision::Zero, _)
        | (_, Decision::Zero) => Ok(false),
        (Decision::Undecidable { enclosure }, _) | (_, Decision::Undecidable { enclosure }) => {
            Err(Error::CannotTellWhichWayAClosedEdgeRuns { enclosure })
        }
        other => panic!("a fifth answer out of the closed-edge rule: {other:?}"),
    }
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
/// The pieces all sit on one flat region and face the same way, but two solids
/// that are flush against each other worked their planes out separately and the
/// last bits may differ: in the normal, or in the offset of two parallel planes
/// a rounding apart (§426). Picking the smallest depends on the set of planes and
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

/// Refuse a joined flat face whose rings do not say which one is its outline,
/// and record their corners.
///
/// Each ring's winding was decided exactly, on the corners as stored, so
/// nothing here is undecided: the corners describe something no face can be.
/// `broken` is what [`Model::validate`] would refuse the face for if it were
/// written as it stands — a second ring winding with the plane goes in as a
/// hole that winds with its outline, [`Invalid::HoleWindsWithTheOuterWire`].
///
/// Each ring's first corner is recorded again after its last, so a reader can
/// tell where one ring stops and the next starts. The enclosure is infinite for
/// the reason [`gave_up`] gives: there is no width here for a policy to accept.
fn no_one_ring_is_the_outline(
    rings: &[Vec<[f64; 3]>],
    broken: Invalid,
    log: &mut ProvenanceLog,
) -> Error {
    let corners: Vec<PredicatePoint> = rings
        .iter()
        .flat_map(|ring| ring.iter().chain(ring.first()))
        .map(|&corner| PredicatePoint::from(corner))
        .collect();
    log.push(Record::new(
        WHICH_RING_IS_THE_OUTLINE,
        &corners,
        Level::Interval,
        f64::INFINITY,
        Measures::Area,
    ));
    Error::TheFuseWroteABrokenAnswer { broken }
}

/// The name the choice of a joined flat face's outline goes by in provenance
/// records.
const WHICH_RING_IS_THE_OUTLINE: &str = "which ring of a joined flat face is its outline";

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

/// Every input vertex the wires through it read past as though it were not
/// there ([`read_through`]): where a closed input edge only started
/// ([`vertices_that_only_start_a_closed_edge`]), and where every wire that
/// ends there walks on along one curve and nothing else ends
/// ([`corners_one_wire_walks_past`]).
///
/// **A wire every side of which would start at one of the second kind keeps
/// them.** It goes right round one closed curve with nothing else on it, so
/// read through it would be one side with no corner to start at. Those
/// vertices then stay, and the answer is refused as it was before Q1302.
fn vertices_read_through(
    model: &Model,
    wires: Vec<&Vec<FaceMeet>>,
    names_of_one_point: &[NamesOfOnePoint],
    on_three_planes: &BTreeMap<VertexSlot, [Plane; 3]>,
) -> BTreeSet<VertexSlot> {
    let mut read_past = vertices_that_only_start_a_closed_edge(model, wires.iter().copied());
    let mut walked_past =
        corners_one_wire_walks_past(model, &wires, names_of_one_point, on_three_planes);
    for wire in &wires {
        let starts: Vec<Option<VertexSlot>> = wire
            .iter()
            .map(|side| match &side.ends[0] {
                MeetPoint::Existing { vertex, .. } => Some(vertex.slot_in(model.id())),
                MeetPoint::OfThreeSurfaces { .. }
                | MeetPoint::WhereAClosedCurveStarts { .. }
                | MeetPoint::HalfWayRoundAClosedCurve { .. }
                | MeetPoint::APoleOfAClosedSurface { .. } => None,
            })
            .collect();
        let read = |start: &Option<VertexSlot>| {
            start.is_some_and(|slot| read_past.contains(&slot) || walked_past.contains(&slot))
        };
        if starts.iter().all(read) {
            for slot in starts.into_iter().flatten() {
                walked_past.remove(&slot);
            }
        }
    }
    read_past.append(&mut walked_past);
    read_past
}

/// The input vertices where every kept side that ends there is one of two
/// sides of a wire arriving along one curve and leaving along the same curve,
/// and no other side ends under any name the point goes by: one wire, on any
/// two surfaces, or any number of wires on the same two planes.
///
/// **One wire alone cannot make such a vertex a corner of the answer**, since
/// each of the two edges either side of it would have that one wire along it
/// and nothing else, and the answer would be refused as not a manifold. So
/// read past, it costs no answer that was one.
///
/// **Several wires on two planes walk one line, and the vertex is a point in
/// the middle of a straight edge.** Read past, the answer holds the same
/// points and has one edge where it had two, which is what §17.4's amendment
/// of 2026-08-30 does at the commit to a corner with two faces round it in
/// the middle of a straight edge. Kept, it cut the edge where the other solid
/// did not: a box described face by face with a corner halfway along its
/// front bottom edge, united with a box sharing that edge and nothing else,
/// had two edges of two faces each along the line and the other box one, and
/// was answered as two solids. The cube in its place is refused
/// `ResultIsNotManifold { count: 4 }`, as §80.1 and §80.9 keep it, and read
/// past, the block with the corner is refused the same (Q1568). Two curved
/// surfaces can meet in curves that cross, so for them the same two surfaces
/// do not show one curve, and several wires on them stay a corner; no test
/// reaches one.
///
/// **It is where pieces of one curved surface met in an earlier fuse.** The
/// joiner keeps such a corner on purpose (`what_is_left`), and the next fuse
/// bakes the pieces as one face, so the corner has only the curve's two
/// surfaces round it, and no surface through it cuts the other solid's face
/// along that curve. Two bars on one pipe, the lower one with a box through
/// its wall, then the upper one: the first union keeps `(1.5, ±1.3229, 2)` on
/// the circle the bars meet on, where the box's face `x = 1.5`, gone from the
/// union, split the wall. The second union drops both caps there and keeps the
/// lower wall walking the circle through those corners and the upper wall
/// walking past them, and was refused `ResultIsNotManifold { count: 1 }`
/// (Q1302). It is §245.5's rule for hole A, which was built only for a vertex
/// that starts a closed edge, taken where it is all a vertex can be.
///
/// **Every other name the point goes by is ruled out by name**, the ways
/// [`Corners::of`] would give it another side: an arrangement that found it to
/// be a meet of three surfaces too ([`NamesOfOnePoint`]), exactly three planes
/// round it ([`input_corners_on_exactly_three_planes`]), and another end that
/// lands on the very same `f64` triple. A closed side starting there ends
/// there twice, so it is counted and rules the vertex out.
fn corners_one_wire_walks_past(
    model: &Model,
    wires: &[&Vec<FaceMeet>],
    names_of_one_point: &[NamesOfOnePoint],
    on_three_planes: &BTreeMap<VertexSlot, [Plane; 3]>,
) -> BTreeSet<VertexSlot> {
    let mut sides_ending: BTreeMap<VertexSlot, usize> = BTreeMap::new();
    let mut walked_past: BTreeMap<VertexSlot, Vec<&[Surface; 2]>> = BTreeMap::new();
    let mut lands_at: BTreeMap<VertexSlot, [u64; 3]> = BTreeMap::new();
    let mut ends_landing: BTreeMap<[u64; 3], usize> = BTreeMap::new();
    for wire in wires {
        for (index, side) in wire.iter().enumerate() {
            for end in &side.ends {
                let at = exactly_at(middle_of(end.point()));
                *ends_landing.entry(at).or_default() += 1;
                if let MeetPoint::Existing { vertex, .. } = end {
                    let slot = vertex.slot_in(model.id());
                    *sides_ending.entry(slot).or_default() += 1;
                    lands_at.insert(slot, at);
                }
            }
            let next = &wire[(index + 1) % wire.len()];
            if let MeetPoint::Existing { vertex, .. } = &side.ends[1]
                && walks_on_along_one_curve(side, next, *vertex)
            {
                walked_past
                    .entry(vertex.slot_in(model.id()))
                    .or_default()
                    .push(&side.carried_by);
            }
        }
    }
    let another_name: BTreeSet<VertexSlot> = names_of_one_point
        .iter()
        .filter_map(|one_point| match &one_point.kept {
            MeetPoint::Existing { vertex, .. } => Some(vertex.slot_in(model.id())),
            MeetPoint::OfThreeSurfaces { .. }
            | MeetPoint::WhereAClosedCurveStarts { .. }
            | MeetPoint::HalfWayRoundAClosedCurve { .. }
            | MeetPoint::APoleOfAClosedSurface { .. } => None,
        })
        .chain(on_three_planes.keys().copied())
        .collect();
    sides_ending
        .into_iter()
        .filter(|&(slot, ending)| {
            walked_past
                .get(&slot)
                .is_some_and(|walks| every_side_ending_walks_on(walks, ending))
                && !another_name.contains(&slot)
                && ends_landing[&lands_at[&slot]] == ending
        })
        .map(|(slot, _)| slot)
        .collect()
}

/// Whether the wires that walk on past a vertex, carried by `walks`, are all
/// of the `ending` sides that end there, and walk one curve: one wire on any
/// two surfaces, or any number on the same two planes, which meet in one line.
fn every_side_ending_walks_on(walks: &[&[Surface; 2]], ending: usize) -> bool {
    let (&&[one, other], rest) = walks
        .split_first()
        .expect("a vertex is counted as walked past once a wire walks past it");
    walks.len() * 2 == ending
        && (rest.is_empty()
            || (one.as_plane().is_some()
                && other.as_plane().is_some()
                && rest
                    .iter()
                    .all(|&&walk| walk == [one, other] || walk == [other, one])))
}

/// The input vertices where the kept pieces do nothing but walk along one
/// closed input edge, which is all such a vertex was: where that edge started.
///
/// **A closed edge has to start somewhere, and where is the builder's choice.**
/// `add_round_bar` starts its cap circles at `(radius, 0, ·)`. No third surface
/// meets the circle there, so nothing the arrangement cuts another face by
/// knows the point: two bars side by side leave one bar's cap circle split at
/// that vertex in the faces cut from that bar, and whole as one cut in the
/// other bar's cap. Both were kept, the two sides and the one side were three
/// edges with one face each, and three of the six fuses of that pair refused
/// `ResultIsNotManifold { count: 1 }` (§245.3, hole A). So the vertex goes, and
/// every face that walked through it walks one side where it walked two.
///
/// **What is tested is the input's own edges and nothing about the geometry.**
/// A vertex qualifies when every kept side ending there
///
/// - runs along exactly one input edge, the same one for every side, and that
///   edge starts and ends at this vertex;
/// - does not close on itself, since a closed side needs its one corner; and
/// - is one of two sides of one wire walked straight through the vertex: one
///   arriving and the next leaving, on the same two surfaces, with the
///   material on the same side and along the same edge. The run the two make
///   must not come back to where it started, which would turn two sides into
///   a closed one.
///
/// A real corner fails the first test. Where a third surface meets the curve,
/// a side along that surface's crossing ends at the vertex, and the cut the
/// arrangement makes along it runs along no input edge at all.
/// `where_a_bar_started_its_cap_circle_stays_a_corner_where_the_other_circle_crosses_it`
/// is that case, in `tests/fusing_two_bars_side_by_side.rs`.
///
/// **The test is the input edge, and not the crossing by name as §245.5 put
/// it**, because one crossing can turn. A plane through a cone's apex cuts it
/// in two rulings that meet there, and every side at the apex then runs along
/// the same plane and the same cone. The rulings are cuts or two input edges,
/// never one closed edge. That case was argued here and not built, and today
/// it cannot arise: a pointed cone's apex is not a vertex of the model
/// (`tests/cones/mod.rs` builds none there), and only an existing vertex is
/// ever dropped.
///
/// **The input's closed edge is one curve, so its start is no bend.** An edge
/// that closes runs right round one closed crossing, and nothing on it is
/// special to anything but the builder who started it there.
///
/// **Only sides that name the vertex are counted**, the way [`Corners::of`]
/// names an input vertex. A side of the other solid that ends at the same
/// point under another name, as where three surfaces meet, is not counted. It
/// would need a surface through the vertex that cuts no face of the vertex's
/// own solid there, which is a tangency, and no test reaches one.
fn vertices_that_only_start_a_closed_edge<'a>(
    model: &Model,
    kept_wires: impl Iterator<Item = &'a Vec<FaceMeet>>,
) -> BTreeSet<VertexSlot> {
    let mut sides_ending: BTreeMap<VertexSlot, usize> = BTreeMap::new();
    let mut walked_through: BTreeMap<VertexSlot, usize> = BTreeMap::new();
    let mut edge_at: BTreeMap<VertexSlot, EdgeKey> = BTreeMap::new();
    let mut a_corner: BTreeSet<VertexSlot> = BTreeSet::new();
    for wire in kept_wires {
        for (index, side) in wire.iter().enumerate() {
            for end in &side.ends {
                let MeetPoint::Existing { vertex, .. } = end else {
                    continue;
                };
                let slot = vertex.slot_in(model.id());
                *sides_ending.entry(slot).or_default() += 1;
                match the_closed_edge_it_only_starts(model, side, *vertex) {
                    Some(edge) if *edge_at.entry(slot).or_insert(edge) == edge => {}
                    Some(_) | None => {
                        a_corner.insert(slot);
                    }
                }
            }
            let next = &wire[(index + 1) % wire.len()];
            if let MeetPoint::Existing { vertex, .. } = &side.ends[1]
                && walks_straight_through(side, next, *vertex)
            {
                *walked_through
                    .entry(vertex.slot_in(model.id()))
                    .or_default() += 1;
            }
        }
    }
    sides_ending
        .into_iter()
        .filter(|(slot, ending)| {
            !a_corner.contains(slot)
                && walked_through.get(slot).copied().unwrap_or(0) * 2 == *ending
        })
        .map(|(slot, _)| slot)
        .collect()
}

/// The one input edge `side` runs along, where that edge starts and ends at
/// `vertex` and `side` does not close on itself; `None` otherwise.
fn the_closed_edge_it_only_starts(
    model: &Model,
    side: &FaceMeet,
    vertex: VertexKey,
) -> Option<EdgeKey> {
    if closes_on_itself(side) {
        return None;
    }
    let [along] = side.along.as_slice() else {
        return None;
    };
    let edge = model
        .edge(*along)
        .expect("a side of a piece runs along an edge of the model the fuse was given");
    (edge.start().key() == vertex && edge.end().key() == vertex).then_some(*along)
}

/// Whether a wire arrives at `vertex` along `arriving` and leaves it along
/// `leaving` as though the vertex were not there.
fn walks_straight_through(arriving: &FaceMeet, leaving: &FaceMeet, vertex: VertexKey) -> bool {
    walks_on_along_one_curve(arriving, leaving, vertex) && arriving.along == leaving.along
}

/// Whether a wire arrives at `vertex` along `arriving` and leaves it along
/// `leaving` on the same two surfaces with the material on the same side,
/// along whichever input edges, and the two do not come back round to where
/// the first started, which would make them one closed side.
fn walks_on_along_one_curve(arriving: &FaceMeet, leaving: &FaceMeet, vertex: VertexKey) -> bool {
    matches!(&leaving.ends[0], MeetPoint::Existing { vertex: from, .. } if *from == vertex)
        && arriving.carried_by == leaving.carried_by
        && arriving.material_side == leaving.material_side
        && arriving.ends[0] != leaving.ends[1]
}

/// One wire of a kept piece, with the two sides either side of every vertex in
/// `walked_through` read as one side, running along the input edges of both.
///
/// Borrowed as it is when the wire passes through none of them, which is every
/// wire of almost every fuse.
///
/// # Panics
///
/// If every side of the wire starts at one of those vertices. A side leaving
/// one where a closed edge starts runs along that edge, so it can only stop
/// somewhere that edge is cut, and a vertex it could stop at would be where
/// another closed edge starts: then one side would run along two input edges.
/// And [`vertices_read_through`] keeps every vertex the wires walk past in a
/// wire where this would happen.
fn read_through<'a>(
    wire: &'a [FaceMeet],
    walked_through: &BTreeSet<VertexSlot>,
    model: &Model,
) -> Cow<'a, [FaceMeet]> {
    let starts_at_one = |side: &FaceMeet| {
        matches!(
            &side.ends[0],
            MeetPoint::Existing { vertex, .. } if walked_through.contains(&vertex.slot_in(model.id()))
        )
    };
    if !wire.iter().any(starts_at_one) {
        return Cow::Borrowed(wire);
    }
    let opens = wire
        .iter()
        .position(|side| !starts_at_one(side))
        .expect("a side leaving a vertex that only starts a closed edge stops at a real corner");
    let mut read: Vec<FaceMeet> = Vec::with_capacity(wire.len());
    for step in 0..wire.len() {
        let side = &wire[(opens + step) % wire.len()];
        if !starts_at_one(side) {
            read.push(side.clone());
            continue;
        }
        let arriving = read
            .pop()
            .expect("the side before one leaving a vertex walked through arrives at it");
        let mut along = arriving.along.clone();
        for edge in &side.along {
            if !along.contains(edge) {
                along.push(*edge);
            }
        }
        read.push(FaceMeet {
            ends: [arriving.ends[0].clone(), side.ends[1].clone()],
            approximation: the_wider(arriving.approximation, side.approximation),
            along,
            ..arriving
        });
    }
    Cow::Owned(read)
}

/// The wires of the pieces with the two halves of a closed curve cut at a pole
/// read as one closed side, wherever another side closes on the same curve
/// whole.
///
/// **A ball's side of a rim can come in halves where the other side's is
/// whole.** A ball's poles are straight up and down from its centre, so a
/// ball centred on the end cap of a bar lying along `x` or `y`, of the bar's
/// radius, has both poles on the cap's rim. `arrange` cuts the circle the
/// cap's plane cuts the ball in at both poles
/// (`across_a_circle_that_starts_at_a_pole`, §194.4), so a piece of the ball
/// walks the rim as two halves, while the bar's cap and pipe walk it as one
/// closed side at the bar's seam corner. [`the_name_each_closed_curve_goes_by`]
/// gave the two pairs one name, but only for sides that close, so the rim was
/// three edges with one face each and every fuse of that dome was refused
/// `ResultIsNotManifold { count: 1 }` (Q1143).
///
/// So the two halves become one side from where the first starts round to
/// where the second ends, which is the same place, and
/// [`Corners::with_each_closed_curve_started_once`] gives it the curve's one
/// corner with the whole sides' starts. **Only where some side closes on the
/// curve under a name the halves' name joins.** Halves that every face along
/// the curve brings are two edges with two faces each already, and joined
/// there a circle whose only corner is a pole would have no direction to read
/// (§194.4).
///
/// Two sides are halves of one curve when one arrives where the other leaves,
/// at the place half way round, on the same two surfaces, with the material on
/// the same side and along the same input edges, and the two together close.
///
/// # Errors
///
/// Whatever [`name_of_the_closed_crossing_through`] gives.
fn with_the_halves_of_each_whole_curve_joined<'a>(
    wires: Vec<Vec<Cow<'a, [FaceMeet]>>>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<Vec<Cow<'a, [FaceMeet]>>>> {
    let a_half = |side: &FaceMeet| {
        side.ends
            .iter()
            .any(|end| matches!(end, MeetPoint::HalfWayRoundAClosedCurve { .. }))
    };
    if !wires
        .iter()
        .flatten()
        .flat_map(|wire| wire.iter())
        .any(a_half)
    {
        return Ok(wires);
    }
    let mut named: Vec<(&FaceMeet, CurveName, bool)> = Vec::new();
    for side in wires.iter().flatten().flat_map(|wire| wire.iter()) {
        let closes = closes_on_itself(side);
        if closes || a_half(side) {
            let curve = name_of_the_closed_crossing_through(
                &side.carried_by,
                side.ends[0].point(),
                policy,
                log,
            )?;
            named.push((side, curve, closes));
        }
    }
    let goes_by = the_name_each_closed_curve_goes_by(
        named
            .iter()
            .map(|&(side, curve, _)| (&side.carried_by, curve, side.ends[0].point())),
        policy,
    );
    let one_name = |curve: CurveName| goes_by.get(&curve).copied().unwrap_or(curve);
    let whole: BTreeSet<CurveName> = named
        .iter()
        .filter(|&&(_, _, closes)| closes)
        .map(|&(_, curve, _)| one_name(curve))
        .collect();
    let to_join: BTreeSet<CurveName> = named
        .iter()
        .filter(|&&(_, curve, closes)| !closes && whole.contains(&one_name(curve)))
        .map(|&(_, curve, _)| curve)
        .collect();
    if to_join.is_empty() {
        return Ok(wires);
    }
    let mut joined = Vec::with_capacity(wires.len());
    for piece in wires {
        let mut of_this_piece = Vec::with_capacity(piece.len());
        for mut wire in piece {
            while let Some(at) = where_two_halves_of_one_of(&wire, &to_join, policy, log)? {
                let mut read = wire.to_vec();
                read.rotate_left(at);
                let leaving = read.remove(1);
                let arriving = &mut read[0];
                arriving.ends[1] = leaving.ends[1].clone();
                arriving.approximation = the_wider(arriving.approximation, leaving.approximation);
                wire = Cow::Owned(read);
            }
            of_this_piece.push(wire);
        }
        joined.push(of_this_piece);
    }
    Ok(joined)
}

/// Where in `wire` a side arrives half way round one of the curves in
/// `to_join` and the next side leaves from there to close it, as the place of
/// the first of the two; `None` where no two sides do.
///
/// # Errors
///
/// Whatever [`name_of_the_closed_crossing_through`] gives.
fn where_two_halves_of_one_of(
    wire: &[FaceMeet],
    to_join: &BTreeSet<CurveName>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<usize>> {
    for (at, arriving) in wire.iter().enumerate() {
        let leaving = &wire[(at + 1) % wire.len()];
        let halves = matches!(arriving.ends[1], MeetPoint::HalfWayRoundAClosedCurve { .. })
            && wire.len() > 1
            && leaving.ends[0] == arriving.ends[1]
            && leaving.ends[1] == arriving.ends[0]
            && leaving.carried_by == arriving.carried_by
            && leaving.material_side == arriving.material_side
            && leaving.along == arriving.along
            && matches!(arriving.ends[0], MeetPoint::WhereAClosedCurveStarts { .. });
        if halves
            && to_join.contains(&name_of_the_closed_crossing_through(
                &arriving.carried_by,
                arriving.ends[0].point(),
                policy,
                log,
            )?)
        {
            return Ok(Some(at));
        }
    }
    Ok(None)
}

/// The wider of two descriptions' slop, or whichever one there is.
fn the_wider(one: Option<Approximation>, other: Option<Approximation>) -> Option<Approximation> {
    match (one, other) {
        (Some(one), Some(other)) if other.magnitude() > one.magnitude() => Some(other),
        (Some(one), Some(_) | None) => Some(one),
        (None, other) => other,
    }
}

/// Each corner of an input solid with exactly three planes round it, and those
/// three planes, for [`Corners::of`] to name it by.
///
/// **An input corner can lie on an edge of the other solid**, and then that
/// solid's pieces bring the same corner as the meet of three planes. A comb's
/// tooth corner on the floor-and-wall edge of an ell, turned, is one: the two
/// solids store the floor and the wall with the same bits, selection keeps
/// the floor and the wall of whichever solid came first, and the tooth's own
/// wall brings the comb's vertex. Named as the vertex on one side and as the
/// three planes on the other, and rounded 4 ulps apart in `y`, the one corner
/// was two, and four edges got one face each, in one order only (§351.5).
///
/// Only three planes, because only there is the meet one point that the three
/// names settle on their own ([`names_of_the_three_surfaces`]). A corner with
/// four planes round it, or with anything curved, keeps its vertex's name.
fn input_corners_on_exactly_three_planes(
    model: &Model,
    scratch: &Scratch,
) -> BTreeMap<VertexSlot, [Plane; 3]> {
    let mut on_three_planes = BTreeMap::new();
    for solid in scratch.arguments.iter().chain(&scratch.tools) {
        for side in solid
            .faces
            .iter()
            .flat_map(|face| face.wires.iter().flatten())
        {
            let corner = &side.start;
            let [one, other, last] = corner.around.as_slice() else {
                continue;
            };
            let (Some(one), Some(other), Some(last)) =
                (one.as_plane(), other.as_plane(), last.as_plane())
            else {
                continue;
            };
            on_three_planes.insert(corner.vertex.slot_in(model.id()), [*one, *other, *last]);
        }
    }
    on_three_planes
}

/// The corners built so far, and the ways to find one again.
#[derive(Default)]
struct Corners {
    made: Vec<Corner>,
    named: BTreeMap<CornerName, usize>,
    at: BTreeMap<[u64; 3], usize>,
    /// Names that are another corner's, settled before any piece was walked:
    /// the one corner of a closed curve, wherever a piece started it. See
    /// [`Corners::with_each_closed_curve_started_once`].
    taken_over: BTreeMap<CornerName, usize>,
    /// The input corners named by their three planes rather than as vertices.
    /// See [`input_corners_on_exactly_three_planes`].
    on_three_planes: BTreeMap<VertexSlot, [Plane; 3]>,
    /// For every name some arrangement found to be one point with another, the
    /// one name all of that point's names go by: the least of them, so that
    /// which face folded which cannot matter. See
    /// [`Corners::join_the_names_of_each_point`].
    goes_by: BTreeMap<CornerName, CornerName>,
    /// Where a point of more than one name goes when one of the names is an
    /// input corner, and which corner that is, by the name the point goes by.
    put_at: BTreeMap<CornerName, (PredicatePoint, VertexKey)>,
    /// For every closed curve the kept sides bring under more than one name,
    /// the one name each of them goes by. See
    /// [`the_name_each_closed_curve_goes_by`].
    curves: BTreeMap<CurveName, CurveName>,
}

/// What one closed curve of the answer is called and where it could start,
/// gathered by [`Corners::with_each_closed_curve_started_once`].
#[derive(Default)]
struct OneClosedCurve<'a> {
    /// Every name a kept side brings it under.
    names: BTreeSet<CurveName>,
    /// Where each kept side along it starts: the input vertex, or `None` where
    /// its own two surfaces start it, with the name that side brings it under.
    starts: Vec<(Option<VertexSlot>, CurveName, &'a MeetPoint)>,
}

/// What two faces agree to call the same corner.
///
/// Never a coordinate. Two faces solving the same three surfaces in a different
/// order round differently, so comparing what they rounded to would either miss
/// a join or invent one; comparing the *names* of the surfaces cannot.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CornerName {
    /// A corner the input already had, unless exactly three planes meet there
    /// ([`input_corners_on_exactly_three_planes`]).
    Vertex(VertexSlot),
    /// One of the places three surfaces meet, each surface written out as the
    /// bits it stores and sorted so the three arrive the same way round
    /// whichever face asked, and **which** of their meeting points this is.
    /// An input corner with exactly three planes round it is this, with
    /// `which` zero.
    ///
    /// Three surfaces can meet at more than one point, so the three names do not
    /// pick a corner on their own; §31.3 is where that field comes from and it
    /// is what stops two different corners being joined into one.
    Meet([[u64; 9]; 3], u8),
    /// The one point on a closed crossing that nothing else cuts, named by the
    /// two surfaces it runs between and which of their closed crossings it is
    /// ([`name_of_the_closed_crossing_through`]), and by what picks that
    /// crossing out of the others the two cross in ([`which_curve_of_the_pair`]).
    RoundACurve(CurveName, WhichCurveOfThePair),
    /// The point across a closed crossing from where it starts, named the same
    /// way, for a crossing that starts at a pole.
    HalfWayRoundACurve(CurveName, WhichCurveOfThePair),
    /// A pole of a closed surface, named by that surface and which of its two
    /// poles this is (§38.1).
    Pole([u64; 9], bool),
}

impl Corners {
    /// No corners yet, but every closed curve of the answer already has the
    /// one corner it will start at, whichever kept piece brings it.
    ///
    /// A closed side's one corner is where its own face's wire happened to
    /// start: an input vertex where the input had one on the curve, and
    /// otherwise where the two surfaces alone put it
    /// ([`MeetPoint::WhereAClosedCurveStarts`]). Two coincident faces can bring
    /// one curve with two different corners. A frustum's cap has its seam vertex
    /// on its circle, and the block roof that cap lies flush in has the circle
    /// the cone cuts it in, started somewhere else. Selection keeps one of the
    /// two coincident disks, whichever input came first, so one argument order
    /// named one edge from both sides and the other named two edges with one
    /// face each, and was refused as not a manifold.
    ///
    /// So the corner is chosen before any is named, by what each candidate is
    /// and never by the order the pieces arrive: an input vertex that some
    /// other kept side also ends at, then any input vertex, then the lowest
    /// point, as [`Assembly::which_to_keep_first`] orders corners. Every other
    /// corner that does nothing but close that curve takes the chosen one. The
    /// start the two surfaces pick is never moved to an input vertex instead,
    /// because the pieces would then depend on more than the face and the set
    /// of surfaces cutting it (§17.4).
    ///
    /// A curve with two input vertices on it that other kept sides end at is
    /// left with both: it is cut there, and nothing here joins the halves.
    ///
    /// Every name an arrangement found to be one point with another goes by
    /// one name from the start, [`Corners::join_the_names_of_each_point`].
    ///
    /// **A curve is named by which of its two surfaces' closed crossings it
    /// is, and not by the two surfaces alone**
    /// ([`name_of_the_closed_crossing_through`]). A plane through a ring's
    /// axis cuts it in two circles. Two quarter bends end to end, from `+x`
    /// round to `+y` and on round to `−x`, keep a cap on `y = 0` at each end,
    /// and named by the plane and the ring alone the two circles were one
    /// curve: the vertex at `(3, 0, 0)` was handed over to the one at
    /// `(−3, 0, 0)`, and the union was refused with four faces along one edge
    /// (§378.3).
    ///
    /// **A curve cut by two pairs of surfaces is one curve and gets one
    /// corner**, chosen the same way from the starts of both
    /// ([`the_name_each_closed_curve_goes_by`]). Where none of them is an
    /// input vertex, the lowest start either pair puts on it is the corner.
    ///
    /// # Errors
    ///
    /// Whatever [`name_of_the_closed_crossing_through`] gives.
    fn with_each_closed_curve_started_once<'a>(
        model: &Model,
        on_three_planes: BTreeMap<VertexSlot, [Plane; 3]>,
        names_of_one_point: &[NamesOfOnePoint],
        kept_sides: impl Iterator<Item = &'a FaceMeet>,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Self> {
        let mut sides: Vec<(&FaceMeet, Option<CurveName>)> = Vec::new();
        for meet in kept_sides {
            let curve = if closes_on_itself(meet) {
                Some(name_of_the_closed_crossing_through(
                    &meet.carried_by,
                    meet.ends[0].point(),
                    policy,
                    log,
                )?)
            } else {
                None
            };
            sides.push((meet, curve));
        }
        let mut corners = Self {
            on_three_planes,
            curves: the_name_each_closed_curve_goes_by(
                sides.iter().filter_map(|&(meet, curve)| {
                    Some((&meet.carried_by, curve?, meet.ends[0].point()))
                }),
                policy,
            ),
            ..Self::default()
        };
        let mut closes: BTreeMap<VertexSlot, BTreeSet<Option<CurveName>>> = BTreeMap::new();
        let mut on_curve: BTreeMap<CurveName, OneClosedCurve<'_>> = BTreeMap::new();
        for (meet, curve) in sides {
            let one_name = curve.map(|curve| corners.curve_goes_by(curve));
            if let (Some(curve), Some(one_name)) = (curve, one_name) {
                on_curve.entry(one_name).or_default().names.insert(curve);
            }
            for end in &meet.ends {
                let slot = match end {
                    MeetPoint::Existing { vertex, .. } => Some(vertex.slot_in(model.id())),
                    MeetPoint::WhereAClosedCurveStarts { .. } => None,
                    MeetPoint::OfThreeSurfaces { .. }
                    | MeetPoint::HalfWayRoundAClosedCurve { .. }
                    | MeetPoint::APoleOfAClosedSurface { .. } => continue,
                };
                if let Some(slot) = slot {
                    closes.entry(slot).or_default().insert(one_name);
                }
                if let (Some(curve), Some(one_name)) = (curve, one_name) {
                    on_curve
                        .entry(one_name)
                        .or_default()
                        .starts
                        .push((slot, curve, end));
                }
            }
        }
        corners.join_the_names_of_each_point(model, names_of_one_point, policy, log)?;
        for (curve, OneClosedCurve { names, mut starts }) in on_curve {
            if names.len() == 1 && starts.iter().all(|(slot, _, _)| slot.is_none()) {
                continue;
            }
            let only_closes_this = |slot: &Option<VertexSlot>| {
                slot.is_some_and(|slot| closes[&slot] == BTreeSet::from([Some(curve)]))
            };
            let mut round = BTreeSet::new();
            for (_, _, end) in &starts {
                if matches!(end, MeetPoint::WhereAClosedCurveStarts { .. }) {
                    round.insert(corners.goes_by(corners.name_of(model, end, policy, log)?));
                }
            }
            starts.sort_by(|(one, one_named, at_one), (other, other_named, at_other)| {
                one.is_none()
                    .cmp(&other.is_none())
                    .then_with(|| only_closes_this(one).cmp(&only_closes_this(other)))
                    .then_with(|| lowest_first(at_one.point(), at_other.point()))
                    .then_with(|| one.cmp(other))
                    .then_with(|| one_named.cmp(other_named))
            });
            starts.dedup_by_key(|(slot, named, _)| (*slot, slot.is_none().then_some(*named)));
            let (_, _, first) = starts[0];
            let start = corners.of(model, first, policy, log)?;
            for name in round {
                corners.taken_over.insert(name, start);
            }
            for (slot, _, _) in &starts[1..] {
                if let Some(slot) = slot.filter(|slot| only_closes_this(&Some(*slot))) {
                    let vertex = corners.goes_by(CornerName::Vertex(slot));
                    corners.taken_over.insert(vertex, start);
                }
            }
        }
        Ok(corners)
    }

    /// The name every name of `curve`'s closed curve goes by: `curve` itself
    /// unless [`the_name_each_closed_curve_goes_by`] found that curve brought
    /// under more than one name.
    fn curve_goes_by(&self, curve: CurveName) -> CurveName {
        self.curves.get(&curve).copied().unwrap_or(curve)
    }

    /// Make every name of one point go by one name, and put the point where
    /// its input corner is, when one of its names is an input corner.
    ///
    /// **A union of names, not a map from one name to one corner.** A meet
    /// can be folded into a corner of one solid on one face and into a corner
    /// of the other solid on another, and then all three are one point; a meet
    /// can be folded into a second meet that a third face folds into a
    /// corner. Each fold was decided by an arrangement under the policy the
    /// fuse was asked under (§284.7), so nothing here asks again.
    ///
    /// The name a point goes by is the least of its names, and where it goes
    /// is the lowest of its input corners, put where [`Corners::of`] would put
    /// that corner alone. Both are taken over every face of both groups, so
    /// neither the order the solids came in nor which pieces are kept moves
    /// them. The input corner wins as it wins the fold, where its point is the
    /// one the faces round it agree on.
    ///
    /// # Errors
    ///
    /// Whatever [`Corners::name_of`] gives.
    fn join_the_names_of_each_point(
        &mut self,
        model: &Model,
        names_of_one_point: &[NamesOfOnePoint],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let mut leads_to: BTreeMap<CornerName, CornerName> = BTreeMap::new();
        let leader = |leads_to: &BTreeMap<CornerName, CornerName>, mut name: CornerName| {
            while let Some(&next) = leads_to.get(&name) {
                name = next;
            }
            name
        };
        for one_point in names_of_one_point {
            let kept = self.name_of(model, &one_point.kept, policy, log)?;
            for (surfaces, which) in &one_point.folded {
                let folded = CornerName::Meet(names_of_the_three_surfaces(surfaces), *which);
                let (one, other) = (leader(&leads_to, kept), leader(&leads_to, folded));
                match one.cmp(&other) {
                    Ordering::Less => leads_to.insert(other, one),
                    Ordering::Greater => leads_to.insert(one, other),
                    Ordering::Equal => None,
                };
            }
        }
        for &name in leads_to.keys() {
            let least = leader(&leads_to, name);
            self.goes_by.insert(name, least);
            self.goes_by.insert(least, least);
        }
        for one_point in names_of_one_point {
            let MeetPoint::Existing { vertex, .. } = &one_point.kept else {
                continue;
            };
            let point = self.where_it_goes(model, &one_point.kept);
            let name = self.goes_by(self.name_of(model, &one_point.kept, policy, log)?);
            let lower = match self.put_at.get(&name) {
                None => true,
                Some((already, theirs)) => lowest_first(point, *already)
                    .then_with(|| vertex.slot_in(model.id()).cmp(&theirs.slot_in(model.id())))
                    .is_lt(),
            };
            if lower {
                self.put_at.insert(name, (point, *vertex));
            }
        }
        Ok(())
    }

    /// The corner `end` names, made the first time it is asked for.
    ///
    /// **A corner on three planes is put where the three planes alone put
    /// it**, [`where_three_planes_meet_whichever_face_asks`], and not where the
    /// spelling that arrived first put it. An input corner on three planes was
    /// solved by `bake` from the three as its solid stores them, and a meet of
    /// the same three by `arrange` from the three turned to face the one way,
    /// so the two can round a few ulps apart. Which of them arrives depends on
    /// which pieces selection kept, and that depends on the order the solids
    /// came in. Picking the lower of what arrived would still let the order
    /// decide where the corner goes whenever only one of them arrives (§351.5).
    ///
    /// **A point of more than one name with an input corner among them goes
    /// where that corner goes**, whichever of its names arrives
    /// ([`Corners::join_the_names_of_each_point`]), and is as wide as the
    /// wider of the corner and what arrived.
    ///
    /// # Errors
    ///
    /// Whatever [`Corners::name_of`] gives.
    fn of(
        &mut self,
        model: &Model,
        end: &MeetPoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<usize> {
        let name = self.goes_by(self.name_of(model, end, policy, log)?);
        if let Some(&chosen) = self.taken_over.get(&name) {
            return Ok(chosen);
        }
        let (point, wobble, was) = match self.put_at.get(&name) {
            Some(&(input_corner, vertex)) => (
                input_corner,
                input_corner.uncertainty().max(end.point().uncertainty()),
                Some(vertex),
            ),
            None => {
                let point = self.where_it_goes(model, end);
                let was = match end {
                    MeetPoint::Existing { vertex, .. } => Some(*vertex),
                    MeetPoint::OfThreeSurfaces { .. }
                    | MeetPoint::WhereAClosedCurveStarts { .. }
                    | MeetPoint::HalfWayRoundAClosedCurve { .. }
                    | MeetPoint::APoleOfAClosedSurface { .. } => None,
                };
                (point, point.uncertainty(), was)
            }
        };
        let at = middle_of(point);
        if let Some(&already) = self.named.get(&name) {
            self.settle_on_the_lower_rounding(already, at, wobble);
            return Ok(already);
        }
        let index = match self.at.get(&exactly_at(at)) {
            Some(&already) => {
                self.made[already].wobble = self.made[already].wobble.max(wobble);
                self.made[already].from = self.made[already].from.or(was);
                already
            }
            None => {
                self.made.push(Corner {
                    point: at,
                    wobble,
                    from: was,
                });
                self.at.insert(exactly_at(at), self.made.len() - 1);
                self.made.len() - 1
            }
        };
        self.named.insert(name, index);
        Ok(index)
    }

    /// The name every name of `name`'s point goes by: `name` itself unless an
    /// arrangement found the point to be more than one name.
    fn goes_by(&self, name: CornerName) -> CornerName {
        self.goes_by.get(&name).copied().unwrap_or(name)
    }

    /// What `end` is called, before [`Corners::goes_by`] is asked.
    ///
    /// # Errors
    ///
    /// Whatever [`name_of_the_closed_crossing_through`] gives, for a point
    /// named by the closed crossing it is on.
    fn name_of(
        &self,
        model: &Model,
        end: &MeetPoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<CornerName> {
        Ok(match end {
            MeetPoint::Existing { vertex, .. } => {
                match self.on_three_planes.get(&vertex.slot_in(model.id())) {
                    Some(planes) => CornerName::Meet(
                        names_of_the_three_surfaces(&planes.map(Surface::Plane)),
                        0,
                    ),
                    None => CornerName::Vertex(vertex.slot_in(model.id())),
                }
            }
            MeetPoint::OfThreeSurfaces {
                surfaces, which, ..
            } => CornerName::Meet(names_of_the_three_surfaces(surfaces), *which),
            MeetPoint::WhereAClosedCurveStarts {
                carried_by,
                picked_by,
                point,
            } => CornerName::RoundACurve(
                name_of_the_closed_crossing_through(carried_by, *point, policy, log)?,
                which_curve_of_the_pair(picked_by),
            ),
            MeetPoint::HalfWayRoundAClosedCurve {
                carried_by,
                picked_by,
                point,
            } => CornerName::HalfWayRoundACurve(
                name_of_the_closed_crossing_through(carried_by, *point, policy, log)?,
                which_curve_of_the_pair(picked_by),
            ),
            MeetPoint::APoleOfAClosedSurface {
                surface,
                at_the_top,
                ..
            } => CornerName::Pole(name_of_surface_bits(surface), *at_the_top),
        })
    }

    /// Where `end` goes when it is the only name of its point: where its three
    /// planes alone put it, when it is on three planes, and where it arrived
    /// otherwise.
    fn where_it_goes(&self, model: &Model, end: &MeetPoint) -> PredicatePoint {
        let three_planes = match end {
            MeetPoint::Existing { vertex, .. } => self
                .on_three_planes
                .get(&vertex.slot_in(model.id()))
                .copied(),
            MeetPoint::OfThreeSurfaces { surfaces, .. } => {
                match surfaces.map(|surface| surface.as_plane().copied()) {
                    [Some(one), Some(other), Some(last)] => Some([one, other, last]),
                    [_, _, _] => None,
                }
            }
            MeetPoint::WhereAClosedCurveStarts { .. }
            | MeetPoint::HalfWayRoundAClosedCurve { .. }
            | MeetPoint::APoleOfAClosedSurface { .. } => None,
        };
        three_planes
            .and_then(where_three_planes_meet_whichever_face_asks)
            .unwrap_or_else(|| end.point())
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
    ///
    /// **A corner on three planes no longer reaches it with two roundings**:
    /// [`Corners::of`] puts every spelling of one at the same point. Picking
    /// the lower of what arrived is only order-free when every spelling
    /// arrives in both orders, and which ones arrive is selection's to decide.
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

/// Where three planes meet, worked out from the three alone: each turned to
/// face the one way, put in [`Surface::how_it_sorts_against`]'s order, and
/// solved by Cramer's rule.
///
/// That is the solve `arrange` makes for every meet of three planes
/// (`three_planes_faced_the_one_way`, then `meet_of_three_surfaces`), so a meet
/// that arrives from a piece lands on the same bits here, and an input corner
/// on the same three planes lands on them too.
///
/// `None` where Cramer's rule cannot bound the meet. `meet_of_three_surfaces`
/// then goes on along the line two of the planes cross in, which wants a
/// policy, and the caller keeps the point that arrived. A meet from a piece
/// already went that way, so it is the same point. An input corner keeps the
/// point `bake` solved, and that one case can still round apart from a meet of
/// the same planes.
fn where_three_planes_meet_whichever_face_asks(planes: [Plane; 3]) -> Option<PredicatePoint> {
    let mut faced = planes.map(|flat| {
        let backwards = flat
            .coefficients()
            .iter()
            .find(|coefficient| **coefficient != 0.0)
            .is_some_and(|coefficient| *coefficient < 0.0);
        Surface::Plane(if backwards { flat.reversed() } else { flat })
    });
    faced.sort_by(Surface::how_it_sorts_against);
    let [
        Surface::Plane(one),
        Surface::Plane(other),
        Surface::Plane(last),
    ] = faced
    else {
        unreachable!("three planes sorted are still three planes")
    };
    let point = Plane::meet_of_three(&one, &other, &last).ok()?;
    point
        .uncertainty()
        .is_finite()
        .then(|| as_exact_as_it_is(point))
}

/// Whether a side starts and ends at one corner, which only a closed curve does.
fn closes_on_itself(meet: &FaceMeet) -> bool {
    match &meet.ends {
        [
            MeetPoint::Existing { vertex: one, .. },
            MeetPoint::Existing { vertex: other, .. },
        ] => one == other,
        [
            MeetPoint::WhereAClosedCurveStarts { .. },
            MeetPoint::WhereAClosedCurveStarts { .. },
        ] => true,
        [
            MeetPoint::Existing { .. }
            | MeetPoint::OfThreeSurfaces { .. }
            | MeetPoint::WhereAClosedCurveStarts { .. }
            | MeetPoint::HalfWayRoundAClosedCurve { .. }
            | MeetPoint::APoleOfAClosedSurface { .. },
            _,
        ] => false,
    }
}

/// For every closed curve that comes under more than one name, the one name
/// all of them go by: the least, so that which side arrived first cannot
/// matter.
///
/// **One circle can be where more than two surfaces cross**, and each face
/// along it names it by its own surface and the partner it was cut by. A ball
/// of a bar's radius centred on the bar's top cap cuts the cap's plane in the
/// cap's own rim, which the pipe cuts it in too, and the pipe is tangent to
/// the ball all the way round. Uniting the two, the pipe's side of the rim is
/// the plane and the pipe, started at the bar's seam corner, and the ball's is
/// the plane and the ball, started where that pair starts a circle. Named by
/// its two surfaces alone that was two edges with one face along each, and
/// every rule both ways round was refused as not a manifold; a frustum on a bar
/// of its narrow cap's radius was the same with a cone for the ball (Q1118).
///
/// **Two names are one curve where both are one plane against a ball, a pipe
/// or a cone, and the plane and the two curved surfaces share more than a
/// point.** Such a pair crosses in one closed curve or in nothing that closes
/// ([`which_of_them_a_closed_crossing_runs_round`]), and either curve is a
/// conic in the plane. Two different conics in one plane share at most four
/// points, so where the three share more than a point
/// ([`MeetOfThree::MoreThanAPoint`]) the two are one conic.
///
/// **A plane through a ring's axis joins the ball, pipe or cone whose conic is
/// the circle its name picks.** That plane cuts the ring in two circles, one
/// either side of the axis, and its name says which
/// ([`which_of_the_pairs_closed_crossings_it_is`]). Where the plane and the two
/// curved surfaces share more than a point, the other's conic shares more than
/// four points with the two circles, so it is one of them; and it is the named
/// one exactly where a point of it, the start its own side brings, is in the
/// half-space that picks the named circle. A bar of a bend's tube radius
/// butted onto the bend's cap is this: the bend's side of the cap's rim is the
/// cap plane and the ring, and the bar's the cap plane and its pipe. Left
/// apart, the union had two rims with one face along each and was refused as
/// not a manifold (Q1470).
///
/// **A plane square to a ring's axis is left out, though its name says which
/// circle.** It cuts the ring in an outer and an inner circle, and since Q1513
/// each is named by the side of the pipe of the ring's own radius that keeps
/// it. The argument above would carry over, but no fuse has been seen to reach
/// such a join, so nothing would test it. The top half of a ring on a bar of
/// its own outer radius, whose top rim is the half ring's outer equator, is
/// the shape that would, and every fuse of the two, both ways round, is
/// refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, so no answer hangs on
/// the join
/// (`on_a_bar_of_its_outer_radius_every_fuse_is_refused_where_three_surfaces_meet`
/// in `tests/a_half_ring_beside_a_bar_clear_of_it.rs`, §428.5). So only a
/// circle picked by a flat half-space, whose name has `0` ahead of its numbers
/// ([`which_curve_of_the_pair`]), joins here.
///
/// **One plane, at any scale.** Each pair holds the plane as its own side was
/// cut, and a third solid's face on the same plane can cut the ball first and
/// spell it with numbers an ulp apart in scale from the cap's. The two
/// spellings were compared folding only the negation, so a ball on a bar's
/// cap that a block's face spelled that way was refused as not a manifold in
/// every fuse; they are compared by [`name_of_the_plane_at_any_scale`] now
/// (`tests/a_turned_plane_exactly_proportional_to_another.rs`).
///
/// **Two curved surfaces with no plane between them are left apart.** The
/// argument above is about conics in one plane. A cone and a pipe on one axis
/// cross in two circles, one either side of the apex, and only one of them can
/// be a plane's rim; their name says which
/// ([`which_of_the_pairs_closed_crossings_it_is`]), but nothing here asks
/// whether the three share it, so a join would need an argument of its own.
///
/// A meet of the three that cannot be worked out joins nothing, and its
/// records are not kept, as `arrange.rs` keeps none for the fold it makes of
/// the same three: the answer is then refused as not a manifold, as it was
/// before anything was asked.
fn the_name_each_closed_curve_goes_by<'a>(
    closed: impl Iterator<Item = (&'a [Surface; 2], CurveName, PredicatePoint)>,
    policy: &Policy,
) -> BTreeMap<CurveName, CurveName> {
    let mut cut_out_of: BTreeMap<CurveName, (Plane, Surface, PredicatePoint)> = BTreeMap::new();
    let mut cut_out_of_a_ring: BTreeMap<CurveName, (&[Surface; 2], Plane, Surface)> =
        BTreeMap::new();
    for (carried_by, curve, start) in closed {
        if let Some((flat, round)) = the_plane_and_the_ball_pipe_or_cone_of(carried_by) {
            cut_out_of.entry(curve).or_insert((flat, round, start));
        } else if curve.1.is_some_and(|which| which[0] == 0)
            && let Some((flat, ring)) = the_plane_and_the_ring_of(carried_by)
        {
            cut_out_of_a_ring
                .entry(curve)
                .or_insert((carried_by, flat, ring));
        }
    }
    let mut leads_to: BTreeMap<CurveName, CurveName> = BTreeMap::new();
    let leader = |leads_to: &BTreeMap<CurveName, CurveName>, mut name: CurveName| {
        while let Some(&next) = leads_to.get(&name) {
            name = next;
        }
        name
    };
    let join = |leads_to: &mut BTreeMap<CurveName, CurveName>, one, other| {
        let (one, other) = (leader(leads_to, one), leader(leads_to, other));
        match one.cmp(&other) {
            Ordering::Less => leads_to.insert(other, one),
            Ordering::Greater => leads_to.insert(one, other),
            Ordering::Equal => None,
        };
    };
    let share_more_than_a_point = |flat: Plane, one: &Surface, other: &Surface| {
        matches!(
            meet_of_three_surfaces(
                [&Surface::Plane(flat), one, other],
                policy,
                &mut ProvenanceLog::new(),
            ),
            Ok(MeetOfThree::MoreThanAPoint)
        )
    };
    let one_plane = |flat: Plane, other: Plane| {
        name_of_the_plane_at_any_scale(&Surface::Plane(flat))
            == name_of_the_plane_at_any_scale(&Surface::Plane(other))
    };
    let named: Vec<(&CurveName, &(Plane, Surface, PredicatePoint))> = cut_out_of.iter().collect();
    for (at, &(&one, &(flat, mine, _))) in named.iter().enumerate() {
        for &(&other, &(other_flat, theirs, _)) in &named[at + 1..] {
            if one_plane(flat, other_flat) && share_more_than_a_point(flat, &mine, &theirs) {
                join(&mut leads_to, one, other);
            }
        }
    }
    for (&on_the_ring, &(carried_by, flat, ring)) in &cut_out_of_a_ring {
        for &(&other, &(other_flat, round, start)) in &named {
            let the_named_circle_keeps_it = matches!(
                which_of_the_pairs_closed_crossings_it_is(
                    carried_by,
                    start,
                    policy,
                    &mut ProvenanceLog::new(),
                ),
                Ok(which) if which == on_the_ring.1
            );
            if one_plane(flat, other_flat)
                && the_named_circle_keeps_it
                && share_more_than_a_point(flat, &round, &ring)
            {
                join(&mut leads_to, on_the_ring, other);
            }
        }
    }
    let mut goes_by = BTreeMap::new();
    for &name in leads_to.keys() {
        let least = leader(&leads_to, name);
        goes_by.insert(name, least);
        goes_by.insert(least, least);
    }
    goes_by
}

/// The plane and the ball, pipe or cone of a pair, or `None` for any other
/// pair: two curved surfaces, two planes, or a plane and a ring.
fn the_plane_and_the_ball_pipe_or_cone_of(carried_by: &[Surface; 2]) -> Option<(Plane, Surface)> {
    let ([Surface::Plane(flat), round] | [round, Surface::Plane(flat)]) = carried_by else {
        return None;
    };
    match round {
        Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) => Some((*flat, *round)),
        Surface::Plane(_) | Surface::Torus(_) => None,
        // A kind added to `rcad-geom` after this has not been shown to cross a
        // plane in at most one closed curve.
        _ => None,
    }
}

/// The plane and the ring of a pair, or `None` for any other pair.
fn the_plane_and_the_ring_of(carried_by: &[Surface; 2]) -> Option<(Plane, Surface)> {
    match carried_by {
        [Surface::Plane(flat), ring @ Surface::Torus(_)]
        | [ring @ Surface::Torus(_), Surface::Plane(flat)] => Some((*flat, *ring)),
        [_, _] => None,
    }
}

/// The lower of two points, `x` first and then `y` and `z`, the order
/// [`Assembly::which_to_keep_first`] breaks a tie between two corners by.
fn lowest_first(one: PredicatePoint, other: PredicatePoint) -> Ordering {
    let (one, other) = (middle_of(one).coordinates(), middle_of(other).coordinates());
    (0..3)
        .map(|axis| one[axis].total_cmp(&other[axis]))
        .find(|order| order.is_ne())
        .unwrap_or(Ordering::Equal)
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
/// either side of it and which of their closed crossings it is. See
/// [`which_curve_it_is`]. The third is, for an arc of a
/// closed crossing between a plane and a surface that curves right round — a
/// ball's circle, a cone's rim circle, or one of the two circles a plane cuts a
/// ring in — that curved surface and that plane,
/// or for an arc of the circle two balls, or a cone and a second cone, a pipe
/// or a ball on its axis, cross in, or of a circle or loop a ball and a pipe
/// cross in, the two of them, and which of the crossing's two arcs between the
/// same two corners it is; see
/// [`which_arc_of_a_circle_this_is`]. The fourth is, for an arc of a
/// hyperbola or a parabola a plane cuts on a cone, that cone and that plane;
/// see [`which_open_crossing_of_a_cone_this_is`].
///
/// The fifth is whether the edge is straight: both surfaces either side are
/// planes. **A straight side and a curved one can share both corners**, and
/// they are two edges. A half disk is one: `z ≤ 0.5` cut out of a half ball
/// leaves on the plane `z = 0.5` a chord along `x = 0` and an arc round the
/// ball, both from `(0, −√3/2, 0.5)` to `(0, √3/2, 0.5)`, and named by their
/// corners alone the two were one edge with four faces along it. Whether the
/// surfaces are planes is not a number, so it cannot split one edge in two
/// the way the surfaces' own names would (see [`which_curve_it_is`]).
type EdgeName = (
    [usize; 2],
    Option<CurveName>,
    Option<([[u64; 9]; 2], bool)>,
    Option<[[u64; 9]; 2]>,
    bool,
);

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
    /// faces either side agree about which edge they share, and where the two
    /// faces were cut by two different pairs of surfaces, `corners` gives the
    /// one name the curve goes by ([`the_name_each_closed_curve_goes_by`]).
    ///
    /// **The input edges it is a piece of are every side's along it**, and not
    /// the first side's. Where a cut on one face runs along the boundary of the
    /// other — a wall along a bar's seam, where the wall's side names nothing and
    /// the skin's names the seam — which side comes first is the order the
    /// solids were handed over, and the history of the edge would depend on it.
    /// `seam` is [`the_seam_a_side_runs_along`], which `meet` cannot carry.
    ///
    /// **A closed side is read against the curved surface of `meet`'s own
    /// pair**, which the side carries as [`Side::read_against`] with the
    /// crossing and the frame it was read with, because the answer may keep
    /// another spelling of that surface along the edge, or another surface
    /// altogether, and [`every_closed_edge_read_off_one_spelling`] has to know
    /// which one this was and be able to read the side again.
    ///
    /// **A closed side is read at its edge's one corner**, and not where its
    /// own pair starts the curve. A side whose start another corner took over
    /// ([`Corners::with_each_closed_curve_started_once`]) starts somewhere the
    /// other sides along the edge are not read. The two halves of a rim cut at
    /// a ball's poles, read as one side
    /// ([`with_the_halves_of_each_whole_curve_joined`]), start at a pole, and
    /// a dome lying on a bar along `−y` read there had its ball walk the rim the
    /// way the pipe did and was refused `FacesDoNotAgreeWhichWayTheyFace`; read
    /// at the bar's seam corner it answers (Q1368).
    ///
    /// # Panics
    ///
    /// If a straight side has no length. Every caller leaves such a side out
    /// before asking — [`sides_of`] and
    /// [`the_edge_each_side_of_every_piece_is`] both drop it — so one
    /// reaching here means a caller stopped doing that. A *closed* side is not
    /// one, which is why the two ends being the same corner is checked against
    /// what carries it.
    ///
    /// `turned_round` is whether the rule turns this face's piece round, which
    /// only a closed edge starting at a ball's pole needs to know — see
    /// [`the_way_a_circle_from_a_pole_is_walked`].
    ///
    /// # Errors
    ///
    /// [`Error::AClosedEdgeOfTheAnswerWouldStartAtAPole`] as
    /// [`Assembly::from_the_pieces`] says, and whatever [`which_curve_it_is`]
    /// and [`which_way_a_closed_edge_is_walked`] give.
    #[expect(
        clippy::too_many_arguments,
        reason = "the policy and the log travel as arguments, never as fields (CLAUDE.md)"
    )]
    fn of(
        &mut self,
        meet: &FaceMeet,
        seam: Option<EdgeKey>,
        from: usize,
        to: usize,
        face: FaceKey,
        turned_round: bool,
        corners: &Corners,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Side> {
        let carried_by = meet.carried_by;
        let along: Vec<EdgeKey> = meet.along.iter().copied().chain(seam).collect();
        assert!(
            from != to
                || carried_by
                    .iter()
                    .any(|surface| surface.as_plane().is_none()),
            "a piece of a flat face has a side with no length"
        );
        let at = middle_of(meet.ends[0].point());
        let named = which_curve_it_is(from, to, meet, policy, log)?
            .map(|curve| corners.curve_goes_by(curve));
        let key = (
            [from.min(to), from.max(to)],
            named,
            which_arc_of_a_circle_this_is(meet, from, to, policy, log)?,
            which_open_crossing_of_a_cone_this_is(meet, from, to),
            carried_by
                .iter()
                .all(|surface| surface.as_plane().is_some()),
        );
        let edge = match self.between.get(&key) {
            Some(&already) => {
                for input in along {
                    if !self.made[already].from.contains(&input) {
                        self.made[already].from.push(input);
                    }
                }
                already
            }
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
                    from: along,
                });
                self.between.insert(key, self.made.len() - 1);
                self.made.len() - 1
            }
        };
        let (forward, read_against) = if from == to {
            which_way_a_closed_edge_is_walked(meet, corners.made[from].point, turned_round)?
                .ok_or(Error::AClosedEdgeOfTheAnswerWouldStartAtAPole { face })?
        } else {
            (self.made[edge].ends[0] == from, None)
        };
        Ok(Side {
            edge,
            forward,
            read_against,
        })
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
/// **The answer is against the curved one of `meet`'s pair**, handed back with
/// what it was read against, and [`every_closed_edge_read_off_one_spelling`]
/// turns it round afterwards where the answer stores the edge by another
/// spelling of that surface, or by another surface that runs the other way
/// round it. The path for a circle from a ball's pole reads against the lid's
/// normal and not against a winding, and is never turned by that pass itself:
/// the surface it is read against is a ball, a ball has no second spelling
/// (`name_of_the_surface_either_way_round` folds nothing for it), and where the
/// ball is what the edge is stored by, the pass leaves the ball's own side
/// alone and reads the side across again against the lid.
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
) -> Result<Option<(bool, Option<ReadAgainst>)>> {
    let [mine, partner] = meet.carried_by;
    let Some(curved) = the_curved_one_of(&meet.carried_by) else {
        // Two planes cross in a straight line, which does not close, so nothing
        // reaches here — and a wire with no curved surface either side of it has
        // no longitude to run along.
        return Ok(Some((true, None)));
    };
    let crossing = [gradient_of(&mine, at), gradient_of(&partner, at)];
    let read_against = |frame| {
        Some(ReadAgainst {
            surface: curved,
            crossing,
            frame,
        })
    };
    match a_closed_crossing_is_walked_the_way_it_is_stored(
        [&mine, &partner],
        crossing,
        at,
        meet.material_side,
    )? {
        Some(forward) => Ok(Some((
            forward,
            read_against([
                gradient_of(&curved, at),
                the_way_it_is_parametrised_about(&curved, at),
            ]),
        ))),
        None => {
            let Some(lid) = the_lid_a_circle_from_a_pole_bounds(meet, turned_round) else {
                return Ok(None);
            };
            let frame = [gradient_of(&lid.ball, at), lid.outward];
            let runs_along_stored =
                match which_way_a_closed_crossing_turns(crossing, frame[0], frame[1]) {
                    Decision::Pos => true,
                    Decision::Neg => false,
                    Decision::Zero => return Ok(None),
                    Decision::Undecidable { enclosure } => {
                        return Err(Error::CannotTellWhichWayAClosedEdgeRuns { enclosure });
                    }
                    other => panic!("a fifth answer out of the closed-edge rule: {other:?}"),
                };
            let forward = match meet.material_side {
                Decision::Neg => runs_along_stored,
                Decision::Pos => !runs_along_stored,
                other => panic!("a face with no side to its material: {other:?}"),
            };
            Ok(Some((forward, read_against(frame))))
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
        // **Cone against cone is half the geometry and half `intersect`.** Two
        // cones on one axis meet in circles — two, or one halfway between the
        // apexes for one slope — or at their one apex and nowhere else, which
        // `intersect` answers as touching and builds no run for. None of that
        // holds a line, so `false` is about the geometry there. Two cones that
        // share an apex with their axes leaning apart *do* meet along rulings,
        // which are lines (§50.2 item 4), and what keeps the arm right for them
        // is that `intersect` refuses every cone-and-cone off one axis by name.
        // `where_two_cones_cross` in `rcad-geom` carries the table.
        //
        // So **two** of the three rows here are right only for as long as
        // those refusals stand, and both are rows to revisit the day one
        // lifts: a cone against a cone or a pipe it is not coaxial with. The
        // ball, the coaxial pipe and the coaxial cone answer out of the shapes
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
/// the band.
///
/// **Two exact ends on the chord's own plane keep it, with no room**, and the
/// next fuse's boxes for them straddle it: the bottom arc of a window a box
/// cuts in a pipe's wall is stored with its ends at `x = 1.9364916731037085`
/// and picked by `0.5x ≥ 0.9682458365518543`, and the next fuse solves each end
/// into `x ∈ [1.9364916731037083, 1.9364916731037085]`. Neither of the two ways
/// [`a_picker_that_tells_the_two_arcs_apart`] knows can read such a picker. The
/// next fuse reads the arc by where the middle of the other arc sits instead:
/// `into_the_arc_away_from_the_centre` in `holds.rs` for the shorter, as the
/// window's is, and `into_the_arc_on_the_centres_side` for the longer.
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
///
/// **The two surfaces can cross in two closed curves**, and then which of them
/// this is goes in the name too, [`name_of_the_closed_crossing_through`].
///
/// # Errors
///
/// Whatever [`name_of_the_closed_crossing_through`] gives.
fn which_curve_it_is(
    from: usize,
    to: usize,
    meet: &FaceMeet,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<CurveName>> {
    if from != to {
        return Ok(None);
    }
    Ok(Some(name_of_the_closed_crossing_through(
        &meet.carried_by,
        meet.ends[0].point(),
        policy,
        log,
    )?))
}

/// What names a closed crossing: the two surfaces it runs between, a plane named
/// at any scale and anything else either way round, the two sorted, and which
/// of their closed crossings it is ([`which_of_the_pairs_closed_crossings_it_is`])
/// where that is known to need saying.
///
/// One curve can come under two of these, cut by two pairs, and then goes by
/// the least of them ([`the_name_each_closed_curve_goes_by`]).
type CurveName = ([[u64; 9]; 2], WhichCurveOfThePair);

/// The name of the closed crossing of `carried_by` through `point`, the same
/// from both faces along it and in both argument orders.
///
/// **Two surfaces can cross in more than one closed curve**, so their two names
/// do not pick one out on their own — as three surfaces can meet at more than
/// one point, which is why [`CornerName::Meet`] carries `which` (§31.3). A
/// plane through a ring's axis cuts the ring in two circles, one either side of
/// the axis. Named by the plane and the ring alone, two quarter bends from `+x`
/// round to `+y` and on round to `−x` had their two caps' circles on `y = 0`
/// made one curve, then one corner, then one edge with four faces along it, and
/// the union was refused (§378.3).
///
/// **A cone and a second cone, a pipe or a ball on its axis cross in two
/// circles too**, one either side of a flat half-space square to the axis. A
/// bar of radius one down a pointed cone's axis crosses the cone's surface at
/// `z = 1` and at `z = −1`, on the nappe beyond the apex. Their union, fused
/// again with a cube far from it, draws the bar's wall's edge on the cone again,
/// and the far nappe with it, so the wall comes apart at `z = −1`. Named by the
/// pipe and the cone alone the new circle was the wall's circle at `z = 1`: its
/// start was handed to the input corner at `z = 1`, the cone face and the two
/// bands' three wires ran along one edge, and every order was refused
/// `ResultIsNotManifold { count: 4 }`
/// (`tests/a_pointed_cone_drilled_along_its_axis.rs`).
///
/// **A plane square to a ring's axis cuts it in an outer and an inner
/// circle**, one either side of the pipe of the ring's own radius. The top
/// half of a ring, closed underneath by a flat face on that plane, had both
/// its equators named by the plane and the ring alone, so the two were one
/// curve and one edge with four faces along it, and every fuse that kept the
/// half ring's faces was refused `ResultIsNotManifold { count: 4 }`
/// (`tests/a_half_ring_beside_a_bar_clear_of_it.rs`, Q1513).
///
/// Only those pairs are told apart here. Other pairs that cross in more than
/// one closed curve — two rings, a ball and a pipe, and a thin pipe through a
/// fat one — are still named by their surfaces alone. No fuse of two rings
/// reaches this name, because `intersect` refuses a part of it (§378.8).
/// Whether a fuse of the last two can hand one loop's start to an input corner
/// on the other, as the cone's did, has not been asked.
///
/// **A plane is named at any scale** ([`name_of_the_plane_at_any_scale`]), as
/// the arcs of these crossings are, because two turned faces can spell one
/// plane in a ratio an ulp from one. A ball's skin carries the circle as the
/// spelling of whichever block's face cut it first, and the flat face along it
/// can be the other block's. Named either way round only, every fuse of a ball,
/// or a bar crossing the plane, with a block whose face holds the whole circle
/// and a second block spelling that plane at the other scale gave the circle's
/// two sides two names, two edges of one face each, and was refused
/// `ResultIsNotManifold { count: 1 }`
/// (`tests/a_turned_plane_exactly_proportional_to_another.rs`).
///
/// # Errors
///
/// Whatever [`which_of_the_pairs_closed_crossings_it_is`] gives.
fn name_of_the_closed_crossing_through(
    carried_by: &[Surface; 2],
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<CurveName> {
    let mut named = carried_by.map(|surface| {
        if surface.as_plane().is_some() {
            name_of_the_plane_at_any_scale(&surface)
        } else {
            name_of_the_surface_either_way_round(&surface)
        }
    });
    named.sort_unstable();
    Ok((
        named,
        which_of_the_pairs_closed_crossings_it_is(carried_by, point, policy, log)?,
    ))
}

/// Which of the two circles its two surfaces cross in passes through `point`,
/// for a plane and a ring and for a cone and a second cone, a pipe or a ball on
/// its axis: what `intersect` picks that circle by, as `holds.rs` looks it up
/// ([`the_picker_naming_the_circle_through`]), written out by
/// [`which_curve_of_the_pair`]. A plane through a ring's axis and a cone's pair
/// pick each circle by a flat half-space. A plane square to a ring's axis picks
/// its outer circle by the outside of the pipe of the ring's own radius about
/// the ring's own axis, and its inner circle by the inside (Q1513). `None` for
/// any other pair of surfaces, and for a point that no one circle certainly
/// keeps, which leaves the crossing named by its two surfaces alone, as it was
/// before this was asked.
///
/// **Every spelling of the two gives one circle the same four numbers, once
/// each zero is turned the right way up.** The two faces along one circle can
/// hold two spellings of the plane — a bend's cap faces out of the bend, and
/// the next bend's cap on the same plane faces the other way — and of the ring
/// or the cone, its axis written either way round. `intersect` writes the
/// circles' half-spaces from the two as stored, and the two circles get one
/// half-space and that half-space turned round. For a plane and a ring the
/// normal is the axis crossed with the plane's normal and the offset that
/// normal read at the ring's centre. For a cone the normal is the cone's axis
/// direction — the steeper cone's, for two — and the offset that direction read
/// at a point on the axis worked out from the apexes, the ball's centre and the
/// slopes, none of which turns with the axis. Turning the plane or an axis
/// round negates every number of both, because a product or a difference of
/// negated numbers rounds to the negation of the same sum, and so does an exact
/// dot rounded to the nearest `f64`, which is how
/// `HalfSpace::through_a_point_facing` writes an offset. So the half-space that
/// keeps a given circle holds the same numbers either way — except that a zero
/// may come back `-0.0`, which adding zero turns back, as in [`exactly_at`].
/// `intersect` reads a cone against anything the same in either order.
/// `each_circle_a_plane_through_a_rings_axis_cuts_is_one_name_however_the_two_are_spelled`
/// and `each_circle_round_a_cones_axis_is_one_name_however_the_two_are_spelled`
/// ask every spelling.
///
/// **A pipe picker is one name however the two are spelled.** The pipe is
/// built from the ring alone, its axis and its ring radius, so the plane facing
/// up or down never reaches it, and the ring's axis written either way round
/// is folded by [`name_of_the_surface_either_way_round`].
/// `each_circle_a_plane_square_to_a_rings_axis_cuts_is_one_name_however_the_two_are_spelled`
/// asks every spelling.
///
/// The two circles get two different names: two half-spaces, each picking its
/// own circle and leaving the other out, or the outside and the inside of one
/// pipe, which [`which_curve_of_the_pair`] writes with two different numbers
/// ahead of the pipe's.
///
/// # Errors
///
/// Whatever `intersect` gives for the two.
fn which_of_the_pairs_closed_crossings_it_is(
    carried_by: &[Surface; 2],
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhichCurveOfThePair> {
    match carried_by {
        [Surface::Plane(_), Surface::Torus(_)]
        | [Surface::Torus(_), Surface::Plane(_)]
        | [
            Surface::Cone(_),
            Surface::Cone(_) | Surface::Cylinder(_) | Surface::Sphere(_),
        ]
        | [Surface::Cylinder(_) | Surface::Sphere(_), Surface::Cone(_)] => (),
        [_, _] => return Ok(None),
    }
    let Some(this_circle) =
        the_picker_naming_the_circle_through(&carried_by[0], &carried_by[1], point, policy, log)?
    else {
        return Ok(None);
    };
    Ok(which_curve_of_the_pair(&[this_circle]))
}

/// Which curve of its pair a closed crossing's corner is on: what picks the
/// curve out of the others its two surfaces cross in, written out as bits by
/// [`which_curve_of_the_pair`], and `None` where nothing picks it.
type WhichCurveOfThePair = Option<[u64; 10]>;

/// The name of what picks a closed crossing out of the others its two surfaces
/// cross in, for the corner it starts at: `None` where the pair crosses in one
/// closed curve and nothing picks it.
///
/// **A bar down a pointed cone's axis is why.** The bar's wall crosses the
/// cone's surface in a circle either side of the apex, at `z = −1` and `z = 1`
/// for the cone of slope one, and named by the two surfaces alone both circles
/// started at one corner. The union and the bar less the cone keep the bar's
/// band between them, so the two circles were one closed edge with four faces
/// along it — the band's two wires, the band below and the cone — and both
/// were refused as not a manifold (`tests/a_pointed_cone_drilled_along_its_axis.rs`).
///
/// **The same from both faces along the curve and in both argument orders.**
/// Both faces carry the one pair of surfaces, and `intersect` builds what picks
/// each curve from the two whichever is handed to it first — the cone's apex
/// and axis, for a cone and a pipe on one axis. A flat picker is written as its
/// four numbers with each zero turned the right way up, as
/// [`which_of_the_pairs_closed_crossings_it_is`] writes one: a cone or a pipe
/// spelled with its axis turned round hands back the half-space holding a given
/// curve as the reverse of the one picking the other, which is the same four
/// numbers but for the sign of a zero. A curved picker is written as the surface named
/// either way round ([`name_of_the_surface_either_way_round`]) and which side
/// of it it keeps. The slot ahead of the numbers says which kind of picker it
/// is, so a ball and a pipe never share a name.
///
/// # Panics
///
/// If more than one picker picks the curve, or if the picker is of a kind this
/// crate has not been taught. `intersect` picks every closed curve it picks out
/// with one picker, and [`Picker`] is `#[non_exhaustive]` and lives in another
/// crate, so a change to either arrives with no compile error and this is what
/// notices. Two curves under one name are a wrong answer; a crash is not.
fn which_curve_of_the_pair(picked_by: &[Picker]) -> WhichCurveOfThePair {
    let picker = match picked_by {
        [] => return None,
        [only] => only,
        more => panic!("a closed crossing picked out by more than one picker: {more:?}"),
    };
    let mut name = [0; 10];
    let (kind, surface) = match picker {
        Picker::InFrontOf(half_space) => {
            let [x, y, z] = half_space.normal().components();
            for (slot, number) in name[1..].iter_mut().zip([x, y, z, half_space.offset()]) {
                *slot = (number + 0.0).to_bits();
            }
            return Some(name);
        }
        Picker::OutsideABall(ball) => (1, Surface::Sphere(*ball)),
        Picker::OutsideAPipe(pipe) => (2, Surface::Cylinder(*pipe)),
        Picker::InsideAPipe(pipe) => (3, Surface::Cylinder(*pipe)),
        other => panic!("a picker nothing here can name: {other:?}"),
    };
    name[0] = kind;
    name[1..].copy_from_slice(&name_of_the_surface_either_way_round(&surface));
    Some(name)
}

/// Which of the two arcs between its corners a side is of a closed crossing
/// between a plane and a surface that curves right round — a ball's circle, a
/// cone's rim circle and the ellipse a steeper plane leaves, or one of the two
/// circles a plane cuts a ring in — together with that curved surface and that
/// plane, or of the circle two balls cross in, or a cone and a second cone, a
/// pipe or a ball on its axis, or of a circle or loop a ball and a pipe cross
/// in, together with the two; `None` for any other side.
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
/// # Which crossing, as well as which arc
///
/// The sign tells apart the two arcs of **one** crossing. Two crossings can
/// pass through the same two corners, and an arc of each can leave the lower
/// corner along its own `∇round × ∇plane`, so both get the same sign and named
/// by that alone two edges were one edge with four faces along it. So both
/// surfaces are named beside the sign.
///
/// - **The curved surface**, for two crossings on one plane: the circles two
///   bars side by side leave on their shared cap plane both pass through
///   `(3, ±4, 0)` (§245.4).
/// - **The plane**, for two crossings on one curved surface: a ball of radius
///   five less the block over `x ≤ −3`, `z ≥ 0` keeps an arc of the circle
///   `x = −3` and an arc of the equator, both from `(−3, −4, 0)` to
///   `(−3, 4, 0)` (§330, `tests/a_ball_less_a_wedge.rs`).
///
/// **Both named either way round, because two faces along one arc can carry
/// two spellings of one surface** (§231.3) — a plane facing the other way, a
/// pipe written with its axis turned round or its origin slid along the axis —
/// and the fold is what keeps them one edge. The plane is folded further, to
/// every multiple of its four numbers and not only the negation
/// ([`name_of_the_plane_at_any_scale`]), because two turned faces can spell one
/// plane a factor an ulp from one apart. Every fold is exact, so two
/// different surfaces never come out with one name, and a spelling the fold
/// leaves apart can only split an edge, which is refused, and never pair the
/// faces of two different arcs (§330).
///
/// **The sign reads the plane the way its name is written, and not the way the
/// face spells it.** `∇plane` turns round with the spelling, so read as each
/// face spells it, a face naming the plane facing the other way gets the other
/// arc's sign. Two flat input faces on one plane facing apart do that: a curved
/// face they both cut along one curve holds the two cuts as one crossing, which
/// keeps the spelling of whichever cut came first (`Arrangement::crossing_like`
/// in `arrange.rs`), and the flat face along an
/// arc of it in the answer can be the other one. Read as spelled, one such pair
/// split its arc into two edges of one face each — a ball between two blocks
/// that meet its equator from above and from below was refused
/// `ResultIsNotManifold { count: 1 }` — and a pair along each arc of one
/// crossing paired the four faces crosswise: a lower half ball under two blocks
/// side by side, united, had two faces walk one edge the same way, which the
/// sweep refused (`tests/two_flat_faces_on_one_plane_facing_apart_along_an_arc.rs`).
/// **The curved surface needs no such fold**: a pipe's, a cone's and a ring's
/// gradient are even in the axis direction, a pipe's does not move when its
/// origin slides along the axis, and a ball has the one spelling.
///
/// **The plane costs here what it costs a closed edge, and not what it cost an
/// open straight one.** [`which_curve_it_is`] records a plane in *every* open
/// side's name splitting edges between two solids pressed flush, each working
/// the plane they share out for itself. Only a side of a closed crossing
/// between a plane and a ball, cone, pipe or ring reaches this name, and those
/// are the crossings every closed edge is already named by both surfaces of,
/// folded the same way.
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
/// # Two balls cross in a circle too
///
/// **And an answer can keep both its arcs between two corners.** A half ball
/// of radius 3 cut to `z ≤ 0`, with a ball of radius 1.5 at `(2, 0, 0)` poking
/// out of its rim: the small ball's skin is cut by the big ball's along the
/// circle `x = 2.6875` and by the flat face along `z = 0`, every cut drawn in
/// full, and the two circles cross at `(2.6875, ±1.3332, 0)`. The union keeps
/// three of the four pieces that leaves, two of them either side of the arc
/// above `z = 0`, and the third and the half ball's skin either side of the arc
/// below. Named
/// by their corners alone the two arcs were one edge with four faces along it,
/// and the union both ways round and the small ball less the half ball were
/// refused `ResultIsNotManifold { count: 4 }` (queue row Q1192,
/// `tests/a_rung_whose_ray_meets_a_face_nobody_follows.rs`).
///
/// **The sign rule holds as it stands.** At a point of the circle the two
/// gradients are the two radii there, and their cross product is zero only on
/// the line through both centres. The circle two balls cross in is square to
/// that line and centred on it, and has a radius that is not zero wherever it
/// has two corners, so it never touches the line. The two balls are read in
/// the order their names sort in, and **a ball has one spelling**
/// (`name_of_the_surface_either_way_round` folds nothing for it), so nothing is
/// turned round for how a face spells it. Where the two are one ball there is
/// no circle, and nothing is named.
///
/// # So does a cone and a second cone, a pipe or a ball on its axis
///
/// **And a wall through the axis leaves both arcs between the same two
/// corners.** The frustum of slope 1 cut to `x ≥ 0`, against a steeper frustum,
/// a bar of radius 4 or a ball of radius 3 on its axis: the wall `x = 0` puts
/// two corners on each circle the pair crosses in, at `(0, ±4, 4)` for the
/// first two. The partner's skin holds the whole circle, so a rule keeping the
/// partner's skin either side of the arc at `x < 0` keeps that arc as well as
/// the one along the cone. Named by their corners alone the two were one edge
/// with four faces along it, and the unions both ways round and the partner
/// less the half were refused `ResultIsNotManifold { count: 4 }` (queue row
/// Q1269, `tests/a_cone_hollowed_along_its_axis.rs`).
///
/// **The sign rule holds as it stands.** `intersect` names a cone against
/// another curved surface only where the two share an axis, and refuses every
/// other such pair, so a side carried by one of these pairs is on a circle
/// square to that axis and centred on it. Both gradients lie in the plane
/// through the axis and the point, so `∇S × ∇T` runs along the circle, and the
/// same turn about the axis carries it from one point to the next, so it never
/// vanishes on a circle where it does not vanish at one point. It vanishes
/// everywhere on the one circle a ball inscribed in a cone touches it round,
/// and that circle does not reach here: a frustum of slope 3/4 against the ball
/// of radius 3 it holds is refused before any edge is named, and so is the
/// half of it (`a_ball_a_frustum_touches_round_one_circle_is_refused_before_its_arcs_are_named`
/// in that test file). A cone and a pipe written with the axis turned round
/// have the same gradient, so the two are named either way round and nothing
/// is turned round for how a face spells them.
///
/// # A plane and a ring cross in two circles
///
/// **And an answer can keep both halves of each between the same two
/// corners.** The top halves, `z ≥ 0`, of the two half bends of one ring, ring
/// 4 and tube 1, meet flush on `y = 0`, and united keep both floors and both
/// walls: both halves of each circle `z = 0` cuts the ring in run between
/// `(±5, 0, 0)`, or `(±3, 0, 0)`. Named by their corners alone the two halves
/// were one edge with four faces along it, and the union both ways round was
/// refused `ResultIsNotManifold { count: 4 }` (queue row Q1387,
/// `tests/a_joined_flat_face_whose_rings_enclose_no_area.rs`).
///
/// **And where an answer keeps one half, the fuse still meets both.** A
/// quarter bend and the round bar of the ring's own radius on its axis: the
/// bar's wall cuts each cap's circle at the top and the bottom of the tube,
/// and the two halves between them run along the same plane and ring. With
/// them named alike, every rule both ways round was refused
/// `ACurvedFaceGoesRightRoundWithNoSeam`, a piece of the bend's wall built
/// with one cap's inner half and the other cap's outer half (Q1402). A half
/// bend under the corner of a half bar, two planes meeting across its tube,
/// united, bend first, was refused `ResultIsNotManifold { count: 4 }` (Q1327;
/// the other fuses were not run). This arm alone
/// answers both, under every rule (`tests/a_bend_under_two_planes.rs`, run
/// with nothing else of Q1387 added).
///
/// **The sign rule holds on every circle named here.** `intersect` cuts a ring
/// with a plane only where the plane is square to the ring's axis or holds it,
/// and refuses every other plane as a quartic. At a point of a circle round
/// the axis, `∇ring` lies in the plane through the axis and the point and has
/// a part along the radius, and `∇plane` is the axis, so `∇ring × ∇plane` runs
/// along the circle and the same turn about the axis carries it from one point
/// to the next. That part along the radius vanishes only where the plane rests
/// on the top or the bottom of the tube, and the one circle it touches there is
/// named `RightRoundWhereItsFacesCross` with no picker, so
/// [`the_picker_naming_the_circle_through`] answers `None` and so does this. On
/// a meridian circle `∇ring` lies in the plane, square to `∇plane`, so their
/// cross product runs along the circle and is never zero, since `∇ring` is not
/// zero anywhere on the ring. A side is named only where exactly one of the two
/// circles' pickers keeps its first end, which says the side is on that circle
/// and not on the other. A ring written with its axis turned round has
/// the same gradient, so the ring is named either way round, and the plane is
/// folded as for a ball.
///
/// # A ball and a pipe cross in a circle or a loop
///
/// **And a wall through the pipe's axis leaves both arcs between the same two
/// corners.** The ball of radius 5 at `(0, 0, 5)` cut to `x ≥ 0` against the
/// bar of radius 4 on its axis, or the bar of radius 4 along `x` cut to `z ≥ 0`
/// or `y ≥ 0` against the ball of radius 5 at its centre: the wall puts two
/// corners on each circle the pair crosses in, at `z = 2` and `z = 8` or at
/// `x = ±3`, and the whole partner's skin holds the whole circle. Named by their
/// corners alone the two arcs were one edge with four faces along it, and the
/// unions both ways round and the whole less the half were refused
/// `ResultIsNotManifold { count: 4 }` (queue row Q1329,
/// `tests/two_arcs_where_two_curved_surfaces_cross.rs`). Where the half is the
/// bar and the ball's poles are on its wall, those three now reach the mesh the
/// fuse checks its answer with, which refuses the ball's skin.
///
/// **The sign rule holds on every curve named here.** Put the pipe's axis along
/// `z` and the ball's centre at `(a, 0, h)`. Then `∇pipe` runs along
/// `(x, y, 0)` and `∇ball` along `(x − a, y, z − h)`, and their cross product
/// is `((z − h)y, −(z − h)x, a·y)`. On the pipe `x` and `y` are never both
/// zero, so it vanishes only where `z = h` and `a·y = 0`:
///
/// - **On the axis**, `a = 0`, that is the circle `z = h`, which is on the
///   crossing only where the ball has the pipe's radius and touches it all
///   round. That circle does not reach here between two corners: a wall
///   through it, the bar's half, the bar cut off the axis or the ball's half,
///   is refused ordering the points along the touch before any edge is named
///   (`a_ball_a_bar_holds_round_one_circle_is_refused_where_a_wall_crosses_it`
///   in that test file), and the bar's seam alone puts one corner on it, so
///   the edge closes and nothing is asked here.
/// - **Off the axis** it is the points `(±r, 0, h)`, with `r` the pipe's
///   radius, which are on a ball of radius `R` only where `a` is `R + r` or
///   `|R − r|`. There `where_a_ball_and_a_pipe_cross` in `rcad-geom` finds
///   `Δ = 0` with `V = 2r(R + r)` or `V = 2r(r − R)` for an axis of length
///   one: above zero it answers
///   that the ball rests on the pipe at one point, and below zero, where
///   `R > r`, it refuses `ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo`
///   (`a_bar_touching_a_ball_from_inside_is_refused_before_any_arrangement` in
///   `tests/two_curves_of_one_pair_that_meet.rs`).
///
/// So on every loop `intersect` names, and on both circles a ball on the axis
/// cuts, the cross product runs along the curve and never vanishes. No loop
/// off the axis has been brought here between two corners: a plane through
/// one is refused where it meets both curved surfaces
/// (`a_plane_through_a_loop_off_an_axis_is_refused_where_it_meets_both_curved_surfaces`),
/// so that half of the argument has no test that fails without it. A pipe
/// written with its axis turned round or slid along has the same gradient,
/// and a ball has one spelling, so the two are named either way round and
/// nothing is turned round for how a face spells them.
///
/// # Two pipes are not named here
///
/// **Their gradients can line up on a crossing.** Two bars of radius 1 whose
/// axes meet square line up at `(0, ±1, 0)`, and `intersect` refuses that
/// crossing before any edge exists
/// (`two_bars_of_one_radius_whose_axes_meet_are_refused_before_any_arrangement`
/// in `tests/two_curves_of_one_pair_that_meet.rs`, open row Q1558). No other
/// loop two pipes cross in has reached here between two corners: a plane
/// through one is refused where it meets both curved surfaces (the third shape
/// of the test above), and the cross-drilled shaft of
/// `tests/a_cone_hollowed_along_its_axis.rs`, whose fat bar's seam crosses a
/// loop, sent no side of the two pipes here between two corners when run with
/// a print in this function. So no test could fail for want of the pair. Two
/// arcs of such a loop that did meet here would be one edge with four faces
/// along it and refused `ResultIsNotManifold { count: 4 }`, not answered wrong.
///
/// # Panics
///
/// If the material side is anything but [`Decision::Neg`] or [`Decision::Pos`],
/// as [`which_arc_this_is`] does and for the same reason.
fn which_arc_of_a_circle_this_is(
    meet: &FaceMeet,
    from: usize,
    to: usize,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<([[u64; 9]; 2], bool)>> {
    if from == to {
        return Ok(None);
    }
    which_arc_of_a_circle_walked_this_way(meet, from > to, policy, log)
}

/// [`which_arc_of_a_circle_this_is`] for a side that is walked from the later
/// of its two ends in some order every face along it shares, when
/// `walked_from_the_far_end`.
///
/// That order is the corner numbers for an edge of the answer, and the names
/// of the two places for an arc whose two ends came out one corner
/// ([`the_edge_each_side_of_every_piece_is`]). The sign rule needs no more
/// than that both faces along an arc read its two ends in one order.
///
/// # Errors
///
/// As [`which_arc_of_a_circle_this_is`].
///
/// # Panics
///
/// As [`which_arc_of_a_circle_this_is`].
fn which_arc_of_a_circle_walked_this_way(
    meet: &FaceMeet,
    walked_from_the_far_end: bool,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<([[u64; 9]; 2], bool)>> {
    // A whole ball's seam is walked down and back up by the one face, with a
    // material side that names the half of the meridian it is and not a side
    // (`Arrangement::the_whole_surface`), so the rule below would read its two
    // walks as two arcs. It is one edge, and it has only the one arc to be.
    let a_seam = meet
        .ends
        .iter()
        .any(|end| matches!(end, MeetPoint::APoleOfAClosedSurface { .. }));
    if a_seam {
        return Ok(None);
    }
    let (named, mine_is_the_first, spelled_the_other_way_round) = match meet.carried_by {
        [ball @ Surface::Sphere(_), other @ Surface::Sphere(_)] => {
            let named = [name_of_surface_bits(&ball), name_of_surface_bits(&other)];
            match named[0].cmp(&named[1]) {
                Ordering::Less => (named, true, false),
                Ordering::Greater => ([named[1], named[0]], false, false),
                Ordering::Equal => return Ok(None),
            }
        }
        [flat @ Surface::Plane(_), ring @ Surface::Torus(_)]
        | [ring @ Surface::Torus(_), flat @ Surface::Plane(_)] => {
            let one_of_two_circles = the_picker_naming_the_circle_through(
                &flat,
                &ring,
                meet.ends[0].point(),
                policy,
                log,
            )?
            .is_some();
            if !one_of_two_circles {
                return Ok(None);
            }
            (
                [
                    name_of_the_surface_either_way_round(&ring),
                    name_of_the_plane_at_any_scale(&flat),
                ],
                matches!(meet.carried_by[0], Surface::Torus(_)),
                is_written_the_other_way_round(&flat),
            )
        }
        [
            one @ Surface::Cone(_),
            other @ (Surface::Cone(_) | Surface::Cylinder(_) | Surface::Sphere(_)),
        ]
        | [
            one @ (Surface::Cylinder(_) | Surface::Sphere(_)),
            other @ Surface::Cone(_),
        ]
        | [one @ Surface::Sphere(_), other @ Surface::Cylinder(_)]
        | [one @ Surface::Cylinder(_), other @ Surface::Sphere(_)] => {
            let named = [
                name_of_the_surface_either_way_round(&one),
                name_of_the_surface_either_way_round(&other),
            ];
            match named[0].cmp(&named[1]) {
                Ordering::Less => (named, true, false),
                Ordering::Greater => ([named[1], named[0]], false, false),
                Ordering::Equal => return Ok(None),
            }
        }
        [_, _] => {
            let Some(round_one) = which_of_them_a_closed_crossing_runs_round(&meet.carried_by)
            else {
                return Ok(None);
            };
            let flat = &meet.carried_by[1 - round_one];
            (
                [
                    name_of_the_surface_either_way_round(&meet.carried_by[round_one]),
                    name_of_the_plane_at_any_scale(flat),
                ],
                round_one == 0,
                is_written_the_other_way_round(flat),
            )
        }
    };
    let along_mine_then_partner = match meet.material_side {
        Decision::Neg => true,
        Decision::Pos => false,
        other => panic!("a side of a face was walked with no material side: {other:?}"),
    };
    let along_the_first_then_the_second = along_mine_then_partner == mine_is_the_first;
    Ok(Some((
        named,
        (along_the_first_then_the_second != walked_from_the_far_end) != spelled_the_other_way_round,
    )))
}

/// Which crossing of a plane and a cone a side is on, when that crossing is a
/// hyperbola or a parabola, as the cone named either way round and the plane
/// named at any scale; `None` for any other side.
///
/// **Two planes that each cut a cone in an open curve can cross it at the same
/// two points**, the two where the line the planes meet along goes through the
/// cone, and an answer can keep an arc of each between them. A frustum of slope
/// 1 under the planes `2x + z = 3` and `8x + 3z = −3` keeps an arc of each from
/// `(−6, −13.75, 15)` to `(−6, 13.75, 15)`, and the frustum the planes
/// `2x − z = 1` and `2x + z = 11` cut to a chip keeps an arc of each from
/// `(3, −4, 5)` to `(3, 4, 5)`. Named by their corners alone, the two arcs were
/// one edge with four faces along it, the cone's face twice and each plane's
/// once, and every fuse of either shape was refused
/// `ResultIsNotManifold { count: 4 }` (queue row Q1176,
/// `tests/a_walk_neither_winding_nor_the_axis_places.rs`).
///
/// **Which crossing is all there is to say, and no sign is.** A branch of a
/// hyperbola and a parabola are open curves, and two points of one bound one
/// piece of it. [`which_arc_of_a_circle_this_is`] needs a sign as well because
/// a crossing that closes leaves two arcs between two points.
///
/// **The names do not split one arc in two.** A hyperbola or a parabola does
/// not lie on a line, so it spans one plane, and the two faces along one arc
/// both carry that plane, whose name at any scale is the same for every
/// spelling of it ([`name_of_the_plane_at_any_scale`]). As with a closed
/// crossing (§330), a spelling of the cone the fold leaves apart can only split
/// an edge, which is refused, and never pairs the faces of two different arcs.
fn which_open_crossing_of_a_cone_this_is(
    meet: &FaceMeet,
    from: usize,
    to: usize,
) -> Option<[[u64; 9]; 2]> {
    let (flat, cone) = match meet.carried_by {
        [flat @ Surface::Plane(_), cone @ Surface::Cone(_)]
        | [cone @ Surface::Cone(_), flat @ Surface::Plane(_)] => (flat, cone),
        [_, _] => return None,
    };
    let open = the_crossing_of_these_runs_straight(&meet.carried_by) == Some(false)
        && which_of_them_a_closed_crossing_runs_round(&meet.carried_by).is_none();
    (from != to && open).then(|| {
        [
            name_of_the_surface_either_way_round(&cone),
            name_of_the_plane_at_any_scale(&flat),
        ]
    })
}

/// Whether a plane is spelled facing the other way from the spelling its name
/// is folded to: the one whose first coefficient that is not zero is positive,
/// as `written_the_one_way` in `arrange.rs` folds it.
///
/// # Panics
///
/// If the surface is not a plane.
/// [`which_of_them_a_closed_crossing_runs_round`] answers only for a pair one of
/// which is a plane, and this is asked of that one.
fn is_written_the_other_way_round(flat: &Surface) -> bool {
    let flat = flat
        .as_plane()
        .expect("a closed crossing this names runs between a plane and a curved surface");
    flat.coefficients()
        .into_iter()
        .find(|number| *number != 0.0)
        .is_some_and(|number| number < 0.0)
}

/// A name for the points of a plane, the same for every spelling of them:
/// two planes get one name exactly when their four coefficients are in one
/// ratio, whatever that ratio is.
///
/// **A ratio of `−1` is not the only one a fuse meets.** `intersect` calls two
/// planes one surface whenever their coefficients are exactly proportional,
/// and [`Plane::transformed`](rcad_geom::Plane) does not scale a turned normal
/// back to length one, so two solids turned to put a face on one plane can
/// spell it `(−c, c, 0, 0)` and `(c′, −c′, 0, 0)` with `c` and `c′` an ulp
/// apart. [`name_of_the_surface_either_way_round`] folds only the negation and
/// gave those two names, so the arc they both run along was two edges of one
/// face each, and so was a whole circle
/// ([`name_of_the_closed_crossing_through`]) and a bar's rim a ball sits on
/// ([`the_name_each_closed_curve_goes_by`];
/// `tests/a_turned_plane_exactly_proportional_to_another.rs`).
///
/// **The fold is exact.** Each coefficient that is not zero is an odd whole
/// number times a power of two. Multiplying all four by one number multiplies
/// every odd part by one odd ratio and adds one step to every power, so the odd
/// parts divided by their greatest common divisor, and each power less the
/// first one's, do not move — and the signs are turned so the first number that
/// is not zero is positive, as [`is_written_the_other_way_round`] reads it.
/// Two coefficient sets with one name are in one ratio by the same argument
/// read backwards, so two different planes never share one. Slot 0 is `0`, the
/// kind [`name_of_surface_bits`] gives a plane; slots 1 to 4 hold the odd parts
/// with the turned sign in the top bit, and slots 5 to 8 the powers.
///
/// # Panics
///
/// If the surface is not a plane, as [`is_written_the_other_way_round`] does,
/// or if one of its coefficients is not a finite number: `intersect` refuses
/// a plane holding one before any crossing of it is walked.
fn name_of_the_plane_at_any_scale(flat: &Surface) -> [u64; 9] {
    let flat = flat
        .as_plane()
        .expect("a closed crossing this names runs between a plane and a curved surface");
    let parts = flat.coefficients().map(|number| {
        assert!(
            number.is_finite(),
            "a plane with a crossing walked holds finite numbers: {number}"
        );
        (number != 0.0).then(|| odd_part_and_power_of_two(number))
    });
    let common = parts.iter().flatten().fold(0, |common, &(odd, _, _)| {
        greatest_common_divisor(common, odd)
    });
    let &(_, first_power, first_is_negative) = parts
        .iter()
        .flatten()
        .next()
        .expect("a plane's normal is never zero");
    let mut name = [0; 9];
    for (slot, part) in parts.iter().enumerate() {
        if let &Some((odd, power, negative)) = part {
            name[1 + slot] = (odd / common) | (u64::from(negative != first_is_negative) << 63);
            name[5 + slot] = (power - first_power).cast_unsigned();
        }
    }
    name
}

/// `number` as an odd whole number times two to a power, and whether it is
/// negative: `(odd, power, negative)`.
fn odd_part_and_power_of_two(number: f64) -> (u64, i64, bool) {
    const FRACTION_BITS: u64 = (1 << 52) - 1;
    let bits = number.abs().to_bits();
    let stored_power = i64::try_from(bits >> 52).expect("eleven bits");
    let (whole, power) = if stored_power == 0 {
        (bits & FRACTION_BITS, -1074)
    } else {
        ((bits & FRACTION_BITS) | (1 << 52), stored_power - 1075)
    };
    let zeros = whole.trailing_zeros();
    (whole >> zeros, power + i64::from(zeros), number < 0.0)
}

fn greatest_common_divisor(mut one: u64, mut other: u64) -> u64 {
    while other != 0 {
        (one, other) = (other, one % other);
    }
    one
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
///
/// **A straight side whose two ends are one corner is left out.** The
/// arrangement keeps two meets apart wherever it can order them along a
/// crossing, and over a KiCad pin header's stored planes it orders three meets
/// of planes that by design meet at one notch corner, `8e-18` to `1.6e-17`
/// apart. All three round to the corner's own `f64` triple, so [`Corners::of`]
/// makes them one corner, and a side between two of them has no length as the
/// answer's corners are written (`docs/decisions.md` §365.7). Every face that
/// has the side names its two ends through the same [`Corners`], so every one
/// of them leaves it out, and no edge is left with one face along it.
#[expect(
    clippy::too_many_arguments,
    reason = "the policy and the log travel as arguments, never as fields (CLAUDE.md)"
)]
fn sides_of(
    wire: &[FaceMeet],
    turned_round: bool,
    ancestors: &[FaceKey],
    model: &Model,
    corners: &mut Corners,
    edges: &mut Edges,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<Side>> {
    let mut sides = Vec::new();
    for meet in wire {
        let from = corners.of(model, &meet.ends[0], policy, log)?;
        let to = corners.of(model, &meet.ends[1], policy, log)?;
        if from == to
            && meet
                .carried_by
                .iter()
                .all(|surface| surface.as_plane().is_some())
        {
            continue;
        }
        let seam = the_seam_a_side_runs_along(model, ancestors, meet);
        sides.push(edges.of(
            meet,
            seam,
            from,
            to,
            ancestors[0],
            turned_round,
            corners,
            policy,
            log,
        )?);
    }
    if turned_round {
        sides.reverse();
        for side in &mut sides {
            side.forward = !side.forward;
        }
    }
    Ok(sides)
}

/// Whether a wire of a flat piece of the answer bounds anything: whether some
/// edge of it is walked more often one way than the other.
///
/// A wire on a plane that walks every edge of it as often each way goes out
/// along lines and back along the same lines, and encloses nothing as its
/// corners are written. Only a flat piece is asked: the one wire of a whole
/// closed surface is a seam walked out and back, and bounds the whole surface. A KiCad pin header's wall keeps such a wire: the strip between a
/// notch's chamfer and the plane of another notch's chamfer, which by design
/// meet the wall at one line, is `9.5e-17` wide over the stored planes, and its
/// four corners round to two `f64` triples, which [`Corners::of`] makes two
/// corners (`docs/decisions.md` §365.7). Its two short sides are then
/// [`sides_of`]'s sides with no length and are left out, and its two long ones
/// are one edge walked out and back.
///
/// **Leaving such a wire out keeps every edge's faces paired.** It walks each
/// of its edges once each way, so it takes one face's use off each side of every
/// one of them and leaves the uses the other faces make as they were. An edge
/// only such wires walked is left with no face at all, and
/// [`Assembly::leave_out_what_no_wire_walks`] leaves it out, with every corner
/// no edge left ends at.
fn encloses_something(sides: &[Side]) -> bool {
    let mut how_often: BTreeMap<usize, i64> = BTreeMap::new();
    for side in sides {
        *how_often.entry(side.edge).or_default() += if side.forward { 1 } else { -1 };
    }
    how_often.values().any(|&walked| walked != 0)
}

/// Whether a ring of one or two sides of the answer bounds a face.
///
/// **One side that closes on itself** does, which is the rule
/// `Model::validate` reads: a cap circle and the rim of a round hole are one
/// whole curve and have one corner between them. A seam is the third shape a
/// wire comes in — one edge walked both ways, with a pole between the two
/// walks — and it has four steps once the poles are counted. And **two
/// different sides that are not both straight** bound a face too, as
/// `Model::wire_of_two_steps_bounds_a_face` has it: a chord and the arc across
/// it, which is the half disk a flat cut leaves inside a ball.
fn a_ring_of_one_or_two_sides_bounds_something(ring: &[Side], edges: &[Edge]) -> bool {
    match ring {
        [only] => edges[only.edge].ends[0] == edges[only.edge].ends[1],
        [one, other] => {
            one.edge != other.edge
                && !(edges[one.edge].lies_on_two_planes() && edges[other.edge].lies_on_two_planes())
        }
        _ => false,
    }
}

/// The seam of a piece's input face that `side` runs the whole of, where it
/// runs along one.
///
/// **A seam is left out at the door** (`bake`'s header), so no arc of the
/// arrangement is boundary along it and no side ever names it the way a side
/// along a cap's rim names the rim. A cut can still run exactly along it: a bar
/// whose seam is the line it touches a wall along is cut open there, and the
/// side that cut leaves is a piece of the seam, which history has to say.
///
/// Only a straight side between the seam's own two corners, and only a straight
/// seam: two straight sides with the same two ends are one segment, so this
/// takes no arithmetic. A crossing that cannot be shown to run straight is not
/// named.
fn the_seam_a_side_runs_along(
    model: &Model,
    ancestors: &[FaceKey],
    side: &FaceMeet,
) -> Option<EdgeKey> {
    let [
        MeetPoint::Existing { vertex: start, .. },
        MeetPoint::Existing { vertex: end, .. },
    ] = &side.ends
    else {
        return None;
    };
    if start == end
        || !side.along.is_empty()
        || the_crossing_of_these_runs_straight(&side.carried_by) != Some(true)
    {
        return None;
    }
    ancestors.iter().find_map(|&face| {
        model
            .face(face)
            .expect("a piece's ancestor is a face of the model the fuse was given")
            .wires()
            .flat_map(|wire| wire.coedges())
            .map(|coedge| coedge.edge())
            .find(|edge| {
                let ends = [edge.start().key(), edge.end().key()];
                edge.neighbouring_faces().all(|other| other == face)
                    && *edge.stored_run() == EdgeRun::Straight
                    && ends.contains(start)
                    && ends.contains(end)
            })
            .map(|edge| edge.key())
    })
}

#[cfg(test)]
mod tests {
    use rcad_geom::{
        Axis, Cone, Cylinder, EdgeRun, Isometry, Orientation, Plane, Scale, Sphere, Surface,
        SurfaceMeet, Torus, intersect,
    };
    use std::collections::BTreeSet;

    use rcad_predicates::{Decision, Policy, ProvenanceLog};

    use crate::{DraftStep, Invalid, Rule, SolidBuilder};

    use super::super::Scratch;
    use super::super::select::choose;
    use super::{
        Assembly, Bulging, Corner, Corners, Dart, Edge, EdgeKey, Error, Face, FaceMeet, JoinedFace,
        Lump, MeetPoint, Model, Point, PredicatePoint, Side, SmallVec, Vector, VertexKey, WhichArc,
        WireStep, Written, assemble, name_of_the_closed_crossing_through,
        names_of_the_three_surfaces, the_crossing_of_these_runs_straight,
        the_half_space_both_ends_are_on, the_nappe_an_arc_that_does_not_close_is_on, turns_round,
        vertices_that_only_start_a_closed_edge, winding_round,
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
            picked_by: SmallVec::new(),
            point: PredicatePoint::from(at),
        };

        let mut corners = Corners::default();
        let mut corner_of = |end: MeetPoint| {
            corners
                .of(
                    &model,
                    &end,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("two planes ask no question")
        };
        let moved = corner_of(starts([across, along], [1.0, 0.0, 0.0]));
        let again = corner_of(starts([across, along], [0.0, 0.0, 0.0]));
        let other = corner_of(starts([across, upright], [1.0, 0.0, 0.0]));
        assert_eq!(again, moved, "one name is one corner");
        assert_eq!(
            corners.made[moved].point,
            Point::origin(),
            "the lower of the two roundings is the one kept"
        );

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

    /// The two circles a plane through a ring's axis cuts it in are two
    /// names, and each is one name in all four spellings of the plane and the
    /// ring, in either order (§378.6).
    #[test]
    fn each_circle_a_plane_through_a_rings_axis_cuts_is_one_name_however_the_two_are_spelled() {
        let flat = |facing: f64| {
            Surface::Plane(
                Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, facing, 0.0))
                    .expect("a coordinate direction is a unit normal"),
            )
        };
        let ring = |axis: Axis| {
            Surface::Torus(Torus::new(axis, 4.0, 1.0).expect("a tube narrower than its ring"))
        };
        let spellings = [
            [flat(-1.0), ring(Axis::z())],
            [flat(1.0), ring(Axis::z())],
            [ring(Axis::z().reversed()), flat(-1.0)],
            [ring(Axis::z().reversed()), flat(1.0)],
        ];
        let names_through = |x: f64| -> BTreeSet<_> {
            spellings
                .iter()
                .map(|carried_by| {
                    name_of_the_closed_crossing_through(
                        carried_by,
                        PredicatePoint::from([x, 0.0, 0.0]),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .expect("a plane through the axis cuts two circles")
                })
                .collect()
        };
        let (round_plus_x, round_minus_x) = (names_through(3.0), names_through(-3.0));
        assert_eq!(round_plus_x.len(), 1, "{round_plus_x:?}");
        assert_eq!(round_minus_x.len(), 1, "{round_minus_x:?}");
        assert_ne!(round_plus_x, round_minus_x, "two circles named as one");
    }

    /// The outer and the inner circle a plane square to a ring's axis cuts it
    /// in are two names, and each is one name with the plane facing up or down
    /// and the ring's axis written either way round, in either order (Q1513).
    ///
    /// Named by the plane and the ring alone, the top half of a ring had its
    /// two equators made one curve, and every fuse that kept its faces was
    /// refused `ResultIsNotManifold { count: 4 }`.
    #[test]
    fn each_circle_a_plane_square_to_a_rings_axis_cuts_is_one_name_however_the_two_are_spelled() {
        let flats = [1.0, -1.0].map(|facing| {
            Surface::Plane(
                Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, facing))
                    .expect("a coordinate direction is a unit normal"),
            )
        });
        let rings = [Axis::z(), Axis::z().reversed()].map(|axis| {
            Surface::Torus(Torus::new(axis, 4.0, 1.0).expect("a tube narrower than its ring"))
        });
        let names_through = |x: f64| -> BTreeSet<_> {
            let mut names = BTreeSet::new();
            for flat in flats {
                for ring in rings {
                    for carried_by in [[flat, ring], [ring, flat]] {
                        names.insert(
                            name_of_the_closed_crossing_through(
                                &carried_by,
                                PredicatePoint::from([x, 0.0, 0.0]),
                                &Policy::certified(),
                                &mut ProvenanceLog::new(),
                            )
                            .expect("a plane through the ring's middle cuts two circles"),
                        );
                    }
                }
            }
            names
        };
        let (outer, inner) = (names_through(5.0), names_through(3.0));
        assert_eq!(outer.len(), 1, "{outer:?}");
        assert_eq!(inner.len(), 1, "{inner:?}");
        assert_ne!(outer, inner, "two circles named as one");
    }

    /// The two circles a cone crosses a pipe, a ball or a steeper cone on its
    /// axis in are two names, and each is one name with every axis written
    /// either way round, in either order.
    ///
    /// The cone leans one from its apex at the origin, so its circle at height
    /// `z` has radius `|z|`. A pipe of radius one crosses it at `z = 1` and at
    /// `z = −1`, on the far nappe. A ball of radius `2.5` at `(0, 0, 3.5)` crosses
    /// it where `(z − 3.5)² + z² = 6.25`, which is `2z² − 7z + 6 = 0`, at
    /// `z = 1.5` and `z = 2`. A cone leaning two from `(0, 0, 3)` crosses it where
    /// `|z| = 2|z − 3|`, at `z = 2` and `z = 6`.
    #[test]
    fn each_circle_round_a_cones_axis_is_one_name_however_the_two_are_spelled() {
        let up_from = |height: f64| {
            let axis = Axis::new(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0))
                .expect("a direction with length");
            [axis, axis.reversed()]
        };
        let cone = |axis: Axis, slope: f64| {
            Surface::Cone(Cone::new(axis, slope).expect("a positive slope"))
        };
        let cones = up_from(0.0).map(|axis| cone(axis, 1.0));
        let ball = Surface::Sphere(
            Sphere::new(Point::new(0.0, 0.0, 3.5), 2.5).expect("a ball of radius 2.5"),
        );
        let partners = [
            (
                "a pipe",
                up_from(0.0).map(|axis| {
                    Surface::Cylinder(Cylinder::new(axis, 1.0).expect("a pipe of radius one"))
                }),
                [1.0, -1.0],
            ),
            ("a ball", [ball, ball], [1.5, 2.0]),
            (
                "a steeper cone",
                up_from(3.0).map(|axis| cone(axis, 2.0)),
                [2.0, 6.0],
            ),
        ];
        for (partner, spellings, heights) in partners {
            let names_at = |height: f64| -> BTreeSet<_> {
                let mut names = BTreeSet::new();
                for one in cones {
                    for other in spellings {
                        for carried_by in [[one, other], [other, one]] {
                            names.insert(
                                name_of_the_closed_crossing_through(
                                    &carried_by,
                                    PredicatePoint::from([height.abs(), 0.0, height]),
                                    &Policy::certified(),
                                    &mut ProvenanceLog::new(),
                                )
                                .unwrap_or_else(|why| {
                                    panic!("the cone and {partner} cross in two circles: {why:?}")
                                }),
                            );
                        }
                    }
                }
                names
            };
            let [one, other] = heights.map(names_at);
            assert_eq!(one.len(), 1, "the cone and {partner}: {one:?}");
            assert_eq!(other.len(), 1, "the cone and {partner}: {other:?}");
            assert_ne!(
                one, other,
                "the cone and {partner}: two circles named as one"
            );
        }
    }

    /// The two circles a bar of radius one down a cone's axis crosses the
    /// cone's surface in, at `z = 1` and `z = −1`, start at corners of two
    /// names, and each starts at a corner of one name whichever face asks,
    /// in either order, with the cone's axis written either way round.
    ///
    /// The pickers are what `intersect` hands each face, so this asks it for
    /// them as the arrangement does, the face's own surface first.
    #[test]
    fn each_circle_a_pipe_down_a_cones_axis_cuts_starts_at_a_corner_of_its_own() {
        let model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 1.0).expect("a pipe of radius one"));
        let cones = [Axis::z(), Axis::z().reversed()]
            .map(|axis| Surface::Cone(Cone::new(axis, 1.0).expect("a cone leaning one")));
        let corners = Corners::default();
        let names_at = |height: f64| {
            let on_it = PredicatePoint::from([1.0, 0.0, height]);
            let mut names = BTreeSet::new();
            for cone in cones {
                for carried_by in [[pipe, cone], [cone, pipe]] {
                    let SurfaceMeet::Curves(runs) = intersect(
                        &carried_by[0],
                        &carried_by[1],
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .expect("a pipe and a cone on one axis cross in circles") else {
                        panic!("a pipe and a cone on one axis cross in curves");
                    };
                    let picked_by = runs
                        .iter()
                        .find_map(|run| {
                            let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                                panic!("each of the two circles is picked out: {run:?}");
                            };
                            picked_by
                                .iter()
                                .all(|picker| picker.side_of_point(on_it) == Decision::Pos)
                                .then(|| picked_by.clone())
                        })
                        .expect("one of the two circles holds the point");
                    let starts = MeetPoint::WhereAClosedCurveStarts {
                        carried_by,
                        picked_by,
                        point: on_it,
                    };
                    names.insert(
                        corners
                            .name_of(
                                &model,
                                &starts,
                                &Policy::certified(),
                                &mut ProvenanceLog::new(),
                            )
                            .expect("no ring to ask about"),
                    );
                }
            }
            names
        };
        let (above, below) = (names_at(1.0), names_at(-1.0));
        assert_eq!(
            above.len(),
            1,
            "the circle at z = 1 under more than one name"
        );
        assert_eq!(
            below.len(),
            1,
            "the circle at z = −1 under more than one name"
        );
        assert!(above.is_disjoint(&below), "two circles start at one corner");
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

    /// Two bars end to end on one pipe unite into one bar however the pipe is
    /// written and whichever goes first.
    ///
    /// The upper bar writes its wall with the axis pointing down. The circle
    /// between the two walls then has two spellings of one pipe either side of
    /// it, and two things went wrong there (§231): it was read as two pipes
    /// crossing, which `rcad_geom::walk` walks as a straight line with no
    /// points, and each wall read which way it walks the circle against its own
    /// spelling, so both came out walking it the same way.
    ///
    /// Now the two walls are joined into one face, and there is no circle
    /// between them. The face is on the lower spelling by bits, which is the
    /// one pointing down, so whichever bar's wall was the other spelling has
    /// both its cap circles walked the other way round
    /// ([`Assembly::turn_round_each_closed_edge_the_joined_spelling_stores_the_other_way`]).
    /// `Model::validate` checks those, and the volume checks the rest.
    #[test]
    fn two_bars_on_one_pipe_answer_the_same_however_the_pipe_is_written() {
        let up = Cylinder::new(Axis::z(), 1.0).expect("a radius");
        let down = Cylinder::new(Axis::z().reversed(), 1.0).expect("a radius");
        assert_ne!(
            up.axis().direction(),
            down.axis().direction(),
            "the same pipe written two ways is different numbers, which is the difficulty"
        );

        let by_hand = 4.0 * core::f64::consts::PI;
        let how_far_it_gets = |wall, lower_first: bool| {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let policy = Policy::certified();
            let mut log = ProvenanceLog::new();
            let lower = a_bar_between(&mut model, 0.0, 2.0, up, &policy, &mut log);
            let upper = a_bar_between(&mut model, 2.0, 4.0, wall, &policy, &mut log);
            let [arguments, tools] = if lower_first {
                [lower, upper]
            } else {
                [upper, lower]
            };
            let fused = model
                .fuse(
                    &[arguments],
                    &[tools],
                    Rule::Union,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .expect("two bars end to end unite");
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .expect("the answer validates");
            let [solid] = fused.solids() else {
                panic!("two bars end to end are one bar: {:?}", fused.solids());
            };
            let walls: Vec<Surface> = model
                .solid(*solid)
                .expect("this model made the key")
                .faces()
                .map(|face| face.surface())
                .filter(|surface| surface.as_plane().is_none())
                .collect();
            let lowest = if wall == down { down } else { up };
            assert_eq!(
                walls,
                [Surface::Cylinder(lowest)],
                "the wall is one face on the lower spelling, lower first {lower_first}"
            );
            let measured = model
                .solid(*solid)
                .expect("this model made the key")
                .measure(
                    rcad_geom::Deflection::of(0.01).expect("a positive length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .expect("a bar measures");
            assert!(
                (measured.volume() - by_hand).abs() <= measured.volume_error(),
                "{} ± {} against {by_hand}, lower first {lower_first}",
                measured.volume(),
                measured.volume_error()
            );
        };

        for wall in [up, down] {
            for lower_first in [true, false] {
                how_far_it_gets(wall, lower_first);
            }
        }
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

    /// A ring counted against one spelling of a pipe, whose side names the pipe
    /// by the other spelling, still has the cap's plane for a partner.
    ///
    /// The count runs before the joined face is written onto one spelling, so
    /// a side cut from the upper bar still says its pipe is the upper bar's
    /// while the face is counted against the lower one. Read bit for bit, the
    /// pipe itself was taken for the partner, it is not a plane, and the count
    /// gave up — which leaves a wall that needed a seam in pieces.
    #[test]
    fn a_side_naming_its_pipe_by_the_other_spelling_is_still_counted_round() {
        let up = Surface::Cylinder(Cylinder::new(Axis::z(), 2.0).expect("a radius"));
        let down = Surface::Cylinder(Cylinder::new(Axis::z().reversed(), 2.0).expect("a radius"));
        let cap = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        );
        let assembly = Assembly {
            corners: vec![Corner {
                point: Point::new(2.0, 0.0, 4.0),
                wobble: 0.0,
                from: None,
            }],
            edges: vec![Edge {
                ends: [0, 0],
                carried_by: [up, cap],
                arc: WhichArc::TheOnlyOne,
                from: Vec::new(),
            }],
            runs: Vec::new(),
            faces: vec![Face {
                ancestors: Vec::new(),
                surface: down,
                used: Orientation::Forward,
                wires: vec![vec![Side {
                    edge: 0,
                    forward: true,
                    read_against: None,
                }]],
                is_a_whole_closed_surface: false,
            }],
            shells: Vec::new(),
            solids: Vec::new(),
        };
        let ring = [Dart {
            face: 0,
            wire: 0,
            position: 0,
        }];
        assert_eq!(
            assembly.a_ring_goes_right_round(&ring, &down),
            Some(true),
            "a whole cap circle goes right round the pipe once"
        );
    }

    /// A thin pipe across a wall spelled twice, whose bits sort between the two
    /// spellings, turns the circle between it and the wall round when the wall
    /// is joined onto its lower spelling — and one sorting above both does not.
    ///
    /// Worked out by hand at the corner `(2, 0, z + ½)` where the thin pipe,
    /// along `x` through `(0, 0, z)` with radius ½, meets the wall of radius 2
    /// about `z`: the wall's gradient is `(2, 0, 0)` and the thin pipe's
    /// `(0, 0, ½)`, so the crossing runs along `(0, −1, 0)`. The wall's
    /// longitude there is `z × (2, 0, 0) = (0, 2, 0)` and the thin pipe's is
    /// `x × (0, 0, ½) = (0, −½, 0)`, so the two store the circle running
    /// opposite ways. With the thin pipe between the spellings it named the
    /// circle before the join and the wall names it after; above both, the
    /// wall names it before and after, spelled two ways that wind alike.
    #[test]
    fn a_circle_the_joined_spelling_names_instead_of_a_thin_pipe_is_turned_round() {
        let about_z = |origin: f64| {
            Surface::Cylinder(
                Cylinder::new(
                    Axis::new(Point::new(0.0, 0.0, origin), Vector::new(0.0, 0.0, 1.0))
                        .expect("a direction"),
                    2.0,
                )
                .expect("a radius"),
            )
        };
        let (lower, upper) = (about_z(0.0), about_z(4.0));
        let turned_round = |across_at: f64| {
            let thin = Surface::Cylinder(
                Cylinder::new(
                    Axis::new(Point::new(0.0, 0.0, across_at), Vector::new(1.0, 0.0, 0.0))
                        .expect("a direction"),
                    0.5,
                )
                .expect("a radius"),
            );
            let face = |surface, forward| Face {
                ancestors: Vec::new(),
                surface,
                used: Orientation::Forward,
                wires: vec![vec![Side {
                    edge: 0,
                    forward,
                    read_against: None,
                }]],
                is_a_whole_closed_surface: false,
            };
            let mut assembly = Assembly {
                corners: vec![Corner {
                    point: Point::new(2.0, 0.0, across_at + 0.5),
                    wobble: 0.0,
                    from: None,
                }],
                edges: vec![Edge {
                    ends: [0, 0],
                    carried_by: [upper, thin],
                    arc: WhichArc::TheOnlyOne,
                    from: Vec::new(),
                }],
                runs: Vec::new(),
                faces: vec![face(upper, true), face(thin, false)],
                shells: Vec::new(),
                solids: Vec::new(),
            };
            let joined = [(0, lower), (1, thin)].map(|(member, surface)| JoinedFace {
                members: vec![member],
                surface,
                used: Orientation::Forward,
                is_a_whole_closed_surface: false,
                rings: Vec::new(),
            });
            let along = [[0, 1].map(|face| Dart {
                face,
                wire: 0,
                position: 0,
            })];
            assembly
                .turn_round_each_closed_edge_the_joined_spelling_stores_the_other_way(
                    &joined,
                    &along,
                    &[false],
                )
                .expect("both surfaces say which way the circle runs");
            [0, 1].map(|face| assembly.faces[face].wires[0][0].forward)
        };

        assert_eq!(
            turned_round(1.0),
            [false, true],
            "between the spellings, both sides turn round"
        );
        assert_eq!(
            turned_round(5.0),
            [true, false],
            "above both spellings, neither side turns round"
        );
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
            wires: vec![
                (0..3)
                    .map(|edge| Side {
                        edge,
                        forward,
                        read_against: None,
                    })
                    .collect(),
            ],
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

    /// A corner is the apex of a cone an edge there is carried by when it is
    /// the apex the cone stores, bit for bit but for the sign of a zero, and
    /// not when it is the next `f64` along from it (§347.4, Q862).
    #[test]
    fn only_a_corner_exactly_at_a_cones_apex_is_read_as_the_apex() {
        let apex = Point::new(0.0, -0.7, 3.3);
        let cone = Cone::new(
            Axis::new(apex, Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
            0.5,
        )
        .expect("a cone leaning a half");
        let through_the_apex =
            Plane::from_point_and_normal(apex, Vector::new(1.0, 0.0, 0.0)).expect("a unit normal");
        let corner = |point: Point| Corner {
            point,
            wobble: 0.0,
            from: None,
        };
        let assembly = Assembly {
            corners: vec![
                corner(apex),
                corner(Point::new(-0.0, -0.7, 3.3)),
                corner(Point::new(0.0, -0.7, f64::from_bits(3.3f64.to_bits() + 1))),
                corner(Point::new(0.0, 1.3, 7.3)),
            ],
            edges: vec![Edge {
                ends: [0, 3],
                carried_by: [Surface::Plane(through_the_apex), Surface::Cone(cone)],
                arc: WhichArc::TheOnlyOne,
                from: Vec::new(),
            }],
            runs: vec![EdgeRun::Straight],
            faces: Vec::new(),
            shells: Vec::new(),
            solids: Vec::new(),
        };
        let read: Vec<bool> = (0..4)
            .map(|corner| assembly.is_the_apex_of_a_cone_along(corner, 0))
            .collect();
        assert_eq!(read, [true, true, false, false]);
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

    /// Where a round bar started its cap circle is dropped when all a kept
    /// piece does there is pass along the circle.
    ///
    /// §252's case, on sides built by hand. The next two tests expect the
    /// vertex kept, so this is the one that shows it is kept for what they
    /// change and not because nothing is ever dropped.
    #[test]
    fn where_a_bar_started_its_cap_circle_is_dropped_when_the_kept_arc_only_passes_it() {
        let (model, vertex, circle) = a_cap_circle_started_at_five();
        assert_eq!(
            vertices_that_only_start_a_closed_edge(
                &model,
                [the_arc_through(vertex, &[circle])].iter()
            ),
            BTreeSet::from([vertex.slot_in(model.id())]),
        );
    }

    /// A vertex the kept sides pass along anything but the one closed edge
    /// that starts and ends there stays a corner, even where each wire passes
    /// straight through it.
    ///
    /// `vertices_that_only_start_a_closed_edge` reads the input's edges and
    /// nothing about the geometry, so every row walks the arc of the test
    /// above and changes only the edges its two halves run along. The halves
    /// still pass straight through the vertex, so the pairing test lets every
    /// row through, and each row fails one clause of the first test and no
    /// other. No fuse in the suites builds any of these, which is why §252.8
    /// could delete the first test with every suite still passing.
    #[test]
    fn a_vertex_passed_along_anything_but_the_closed_edge_that_starts_there_stays_a_corner() {
        let (mut model, vertex, circle) = a_cap_circle_started_at_five();
        let id = model.id();
        let here = vertex.slot_in(id);
        let elsewhere = model.add_vertex(Point::new(-5.0, 0.0, 0.0));
        let started_elsewhere = EdgeKey::new(model.add_closed_edge(elsewhere), id);
        let started_here_too = EdgeKey::new(model.add_closed_edge(here), id);
        let leaving = EdgeKey::new(model.add_edge(here, elsewhere), id);
        let arriving = EdgeKey::new(model.add_edge(elsewhere, here), id);

        for (named, wires) in [
            (
                "along no input edge, as two rulings cut through a cone's apex would be",
                vec![the_arc_through(vertex, &[])],
            ),
            (
                "along two input edges at once, as joining two faces on one plane makes",
                vec![the_arc_through(vertex, &[circle, leaving])],
            ),
            (
                "along two closed edges that both start here, one wire along each",
                vec![
                    the_arc_through(vertex, &[circle]),
                    the_arc_through(vertex, &[started_here_too]),
                ],
            ),
            (
                "along a closed edge that starts somewhere else",
                vec![the_arc_through(vertex, &[started_elsewhere])],
            ),
            (
                "along an edge that starts here and ends somewhere else",
                vec![the_arc_through(vertex, &[leaving])],
            ),
            (
                "along an edge that ends here and starts somewhere else",
                vec![the_arc_through(vertex, &[arriving])],
            ),
        ] {
            assert_eq!(
                vertices_that_only_start_a_closed_edge(&model, wires.iter()),
                BTreeSet::new(),
                "a vertex passed {named} was dropped"
            );
        }
    }

    /// A side that closes on itself keeps its one corner.
    ///
    /// §252.8 found the second test could be deleted with every suite
    /// passing, because the pairing test keeps the first row's vertex as well:
    /// a side that is a whole wire on its own is its own next side, and it
    /// comes back to where it started. The pairing test compares the two ends
    /// as points, though, and the second test compares the vertices they name.
    /// The two part where a closed side's two ends are one vertex at two
    /// roundings, and where a closed side's neighbours run along the same edge
    /// without ending where it starts. The arrangement builds neither today,
    /// which is why no suite noticed the second test gone.
    #[test]
    fn a_side_that_closes_on_itself_keeps_its_one_corner() {
        let (model, vertex, circle) = a_cap_circle_started_at_five();
        let at = |x: f64| MeetPoint::Existing {
            vertex,
            point: PredicatePoint::from([x, 0.0, 0.0]),
        };
        let round_the_cap = |ends| side([cap(), pipe()], ends, &[circle]);
        let half_way = MeetPoint::HalfWayRoundAClosedCurve {
            carried_by: [cap(), pipe()],
            picked_by: SmallVec::new(),
            point: PredicatePoint::from([-5.0, 0.0, 0.0]),
        };

        for (named, wire) in [
            (
                "the whole circle as the wire's one side",
                vec![round_the_cap([at(5.0), at(5.0)])],
            ),
            (
                "the whole circle with its two ends a rounding apart",
                vec![round_the_cap([at(5.0), at(5.0_f64.next_up())])],
            ),
            (
                "the whole circle, then round it again in two halves",
                vec![
                    round_the_cap([at(5.0), at(5.0)]),
                    round_the_cap([at(5.0), half_way.clone()]),
                    round_the_cap([half_way, at(5.0)]),
                ],
            ),
        ] {
            assert_eq!(
                vertices_that_only_start_a_closed_edge(&model, [wire].iter()),
                BTreeSet::new(),
                "{named} lost its one corner"
            );
        }
    }

    /// A model holding a round bar's cap circle the way `add_round_bar` builds
    /// it: one closed edge, starting and ending at the vertex `(5, 0, 0)`.
    fn a_cap_circle_started_at_five() -> (Model, VertexKey, EdgeKey) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let vertex = model.add_vertex(Point::new(5.0, 0.0, 0.0));
        let circle = model.add_closed_edge(vertex);
        let id = model.id();
        (model, VertexKey::new(vertex, id), EdgeKey::new(circle, id))
    }

    /// The wire of the piece of the cap inside a second bar standing beside
    /// the first: the arc of the cap circle from `(4, −3, 0)` through `vertex`
    /// to `(4, 3, 0)`, both halves running along `along`, and a cut back along
    /// the second bar's pipe.
    fn the_arc_through(vertex: VertexKey, along: &[EdgeKey]) -> Vec<FaceMeet> {
        let where_the_pipes_cross = |y: f64, which: u8| MeetPoint::OfThreeSurfaces {
            surfaces: [cap(), pipe(), the_other_pipe()],
            which,
            point: PredicatePoint::from([4.0, y, 0.0]),
        };
        let at_five = MeetPoint::Existing {
            vertex,
            point: PredicatePoint::from([5.0, 0.0, 0.0]),
        };
        vec![
            side(
                [cap(), pipe()],
                [where_the_pipes_cross(-3.0, 0), at_five.clone()],
                along,
            ),
            side(
                [cap(), pipe()],
                [at_five, where_the_pipes_cross(3.0, 1)],
                along,
            ),
            side(
                [cap(), the_other_pipe()],
                [
                    where_the_pipes_cross(3.0, 1),
                    where_the_pipes_cross(-3.0, 0),
                ],
                &[],
            ),
        ]
    }

    fn side(carried_by: [Surface; 2], ends: [MeetPoint; 2], along: &[EdgeKey]) -> FaceMeet {
        FaceMeet {
            carried_by,
            ends,
            material_side: Decision::Neg,
            along: along.iter().copied().collect(),
            approximation: None,
        }
    }

    /// **A point inside the other nappe is in no piece of a hyperbola's arc**
    /// (Q1289). The wall `x = −12` cuts the cone of slope 1 about `z` in the
    /// hyperbola `z² − y² = 144`, and the arc of its upper branch under its
    /// chord on `z = 18` bulges down, towards the lower branch. `(−12, 0, 15)`
    /// is between the arc and its chord. `(−12, 0, −15)` is inside the cone too,
    /// and on the side of the chord the arc bulges to, but on the lower nappe.
    #[test]
    fn a_point_inside_the_other_nappe_is_not_between_a_hyperbolas_arc_and_its_chord() {
        let cone = Cone::new(Axis::z(), 1.0).expect("a positive slope");
        let wall =
            Plane::from_point_and_normal(Point::new(-12.0, 0.0, 0.0), Vector::new(-1.0, 0.0, 0.0))
                .expect("a normal with length");
        let reach = 180.0_f64.sqrt();
        let ends = [
            Point::new(-12.0, reach, 18.0),
            Point::new(-12.0, -reach, 18.0),
        ];
        let nappe = the_nappe_an_arc_that_does_not_close_is_on(&cone, &wall, ends)
            .expect("both ends are up the axis from the apex");
        assert_eq!(nappe, Decision::Pos);
        let normal = wall.normal().components();
        let ends = ends.map(|end| end.coordinates());
        let left = winding_round(&[ends[0], ends[1], [-12.0, 0.0, 12.0]], normal)
            .expect("three corners off one line")
            == Decision::Pos;
        let arcs = [Some(Bulging {
            ends,
            surface: Surface::Cone(cone),
            left,
            nappe: Some((cone, nappe)),
        })];
        assert!(
            matches!(turns_round(&arcs, normal, [-12.0, 0.0, 15.0]), Some(1 | -1)),
            "between the arc and its chord"
        );
        assert_eq!(turns_round(&arcs, normal, [-12.0, 0.0, -15.0]), Some(0));
    }

    /// The plane `z = 0`, facing out of a bar that stands on it.
    fn cap() -> Surface {
        Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
                .expect("a normal with length"),
        )
    }

    /// The pipe of radius five round the `z` axis.
    fn pipe() -> Surface {
        Surface::Cylinder(Cylinder::new(Axis::z(), 5.0).expect("a pipe of radius five"))
    }

    /// The same pipe moved eight along `x`, which crosses the first at
    /// `(4, ±3, ·)`.
    fn the_other_pipe() -> Surface {
        let up = Axis::new(Point::new(8.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
            .expect("a direction with length");
        Surface::Cylinder(Cylinder::new(up, 5.0).expect("a pipe of radius five"))
    }
}
