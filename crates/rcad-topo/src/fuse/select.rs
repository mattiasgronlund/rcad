//! Which of the pieces the answer keeps, and which way round.
//!
//! §17.4 settles the shape: **one table read as data and one `match` on
//! [`Rule`]**, not three algorithms wearing one signature. [`UNION`],
//! [`INTERSECTION`] and [`DIFFERENCE`] are that table, [`row_for`] is that
//! `match`, and everything below them is the same code whichever rule was asked
//! for.
//!
//! # Two passes, and why the first one exists
//!
//! `A` is the *union* of the arguments and `B` the union of the tools, so a
//! piece of one argument that is buried inside another argument is not on the
//! boundary of `A` at all and no rule can want it. Every piece is therefore
//! classified twice: once against the rest of its own group, which is
//! [`Rule::Union`]'s row whatever the caller asked for, and then against the
//! other group, which is the caller's row. One pass would answer the wrong
//! question for `fuse([A, A_moved], [], Union)`.
//!
//! # Classifying a piece is classifying one point inside it
//!
//! [`SolidView::classify_uncertain_point`] does the work — §1.7's third exit
//! criterion says classification is *reused*, not reimplemented. What is needed
//! here is a point that is provably inside the piece to hand it, and a piece is
//! a polygon whose corners are boxes rather than numbers.
//!
//! A candidate point is a fixed blend of corners with weights that are powers of
//! two, so a piece of a lattice-aligned model gives an exactly representable
//! candidate and the cascade can still prove things about it. Nothing about a
//! candidate is trusted: [`InPlane::winding_around`] *proves* it is inside
//! before it is used, and a candidate that cannot be proved either way is
//! dropped and the next one tried, without a word in the log. Choosing which
//! point to look at decides nothing (§5).
//!
//! The one predicate this needs is which way three points on the face turn about
//! its normal, and that is [`orient3d_of_points`] against a fourth point off the
//! plane whose side has been proved — see [`InPlane`]. No new predicate, and
//! nothing is projected into two dimensions, which is §15.5's rule.

use std::collections::BTreeMap;

use rcad_geom::{Plane, PlaneAgreement, Point, Surface, SurfaceMeet, intersect};
use rcad_predicates::{
    Decision, Interval, Measures, Point as PredicatePoint, Policy, ProvenanceLog,
    orient3d_of_points,
};

use crate::key::FaceSlot;
use crate::validate::side_of_surface;
use crate::{Containment, Error, Model, Result, SolidView};

use super::arrange::{Piece, agreement_of_two_planes};
use super::bake::{BakedFace, BakedSolid};
use super::{FaceMeet, Scratch, as_exact_as_it_is, gave_up, middle_of, the_other_way};

/// Which pieces of the two groups a fuse keeps.
///
/// `#[non_exhaustive]` because this is an enum that grows: imprint and symmetric
/// difference are the obvious next two. The contrast with
/// [`Orientation`](rcad_geom::Orientation), which §3.7 makes deliberately
/// exhaustive, is the whole test — there is no third way round a face, and there
/// are plenty more selection rules.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// Everything in either.
    Union,
    /// Only what is in both.
    Intersection,
    /// What is in the arguments and not in the tools.
    Difference,
}

/// One piece the answer keeps.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Chosen {
    /// Which of [`Scratch::pieces`] it is.
    pub(crate) piece: usize,
    /// Whether the rule turns it round. Difference keeps a tool's faces
    /// reversed: what was the outside of the tool becomes the inside of the
    /// bite it took.
    pub(crate) reversed: bool,
}

/// Work out which pieces the rule keeps.
///
/// # Errors
///
/// [`Error::Undecidable`] when no point inside a piece could be placed against
/// the solids it has to be placed against.
pub(crate) fn choose(
    model: &Model,
    scratch: &Scratch,
    rule: Rule,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<Chosen>> {
    let solids: Vec<&BakedSolid> = scratch.arguments.iter().chain(&scratch.tools).collect();
    let arguments = scratch.arguments.len();
    let owner = which_solid_owns_each_face(model, &solids);

    let mut chosen = Vec::new();
    for (index, piece) in scratch.pieces.iter().enumerate() {
        // Every ancestor of a piece is a face of one solid, so the first of them
        // names which one as well as any other.
        let mine = owner[&piece.ancestors[0].slot_in(model.id())];
        let side = if mine < arguments {
            Side::Argument
        } else {
            Side::Tool
        };
        let corners = corners_of(piece);
        let in_plane = match piece.surface.as_plane() {
            Some(flat) => {
                Some(InPlane::of(flat).ok_or_else(|| nothing_to_measure_from(piece, log))?)
            }
            None => None,
        };
        let candidates = candidate_points(piece, &corners);

        let own_group: Vec<usize> = (0..solids.len())
            .filter(|&other| other != mine && (other < arguments) == (mine < arguments))
            .collect();
        let against_own = place_the_piece(
            model,
            &solids,
            &own_group,
            &candidates,
            piece,
            in_plane.as_ref(),
            policy,
            log,
        )?;
        if keeps(
            row_for(Rule::Union).row(side).fate(against_own),
            against_own,
            mine,
        )
        .is_none()
        {
            continue;
        }

        let other_group: Vec<usize> = (0..solids.len())
            .filter(|&other| (other < arguments) != (mine < arguments))
            .collect();
        let against_other = place_the_piece(
            model,
            &solids,
            &other_group,
            &candidates,
            piece,
            in_plane.as_ref(),
            policy,
            log,
        )?;
        let fate = row_for(rule).row(side).fate(against_other);
        if let Some(reversed) = keeps(fate, against_other, mine) {
            chosen.push(Chosen {
                piece: index,
                reversed,
            });
        }
    }
    Ok(chosen)
}

/// §17.4's selection table, as data.
///
/// The rows the section prints are the two halves of each entry here. `ON` is
/// split into same-facing and opposite-facing, decided by
/// [`Plane::agreement_with`], and "keep once" is [`Fate::KeepIfItComesFirst`]:
/// of two coincident pieces exactly one survives, and the one that survives is
/// the one from the earlier solid. Arguments come before tools, so for
/// Difference that is the argument's own face — which is the one the answer
/// wants, since the tool's would have been turned round onto it.
const UNION: Selection = Selection {
    arguments: Row {
        inside: Fate::Drop,
        outside: Fate::Keep,
        on_same_way: Fate::KeepIfItComesFirst,
        on_opposite_way: Fate::Drop,
    },
    tools: Row {
        inside: Fate::Drop,
        outside: Fate::Keep,
        on_same_way: Fate::KeepIfItComesFirst,
        on_opposite_way: Fate::Drop,
    },
};

const INTERSECTION: Selection = Selection {
    arguments: Row {
        inside: Fate::Keep,
        outside: Fate::Drop,
        on_same_way: Fate::KeepIfItComesFirst,
        on_opposite_way: Fate::Drop,
    },
    tools: Row {
        inside: Fate::Keep,
        outside: Fate::Drop,
        on_same_way: Fate::KeepIfItComesFirst,
        on_opposite_way: Fate::Drop,
    },
};

const DIFFERENCE: Selection = Selection {
    arguments: Row {
        inside: Fate::Drop,
        outside: Fate::Keep,
        on_same_way: Fate::Drop,
        on_opposite_way: Fate::KeepIfItComesFirst,
    },
    tools: Row {
        inside: Fate::KeepReversed,
        outside: Fate::Drop,
        on_same_way: Fate::Drop,
        on_opposite_way: Fate::KeepIfItComesFirst,
    },
};

/// The one `match` on [`Rule`] in the whole fuse.
///
/// [`Rule`] is `#[non_exhaustive]`, which stops a *caller* matching it
/// exhaustively and leaves this match exhaustive here, where the rows live. A
/// new rule is a new row and a new arm, and the compiler asks for both.
fn row_for(rule: Rule) -> &'static Selection {
    match rule {
        Rule::Union => &UNION,
        Rule::Intersection => &INTERSECTION,
        Rule::Difference => &DIFFERENCE,
    }
}

/// What one rule does with a piece of each group.
struct Selection {
    arguments: Row,
    tools: Row,
}

impl Selection {
    fn row(&'static self, side: Side) -> &'static Row {
        match side {
            Side::Argument => &self.arguments,
            Side::Tool => &self.tools,
        }
    }
}

/// What one rule does with a piece of one group, by where that piece sits.
struct Row {
    inside: Fate,
    outside: Fate,
    on_same_way: Fate,
    on_opposite_way: Fate,
}

impl Row {
    fn fate(&self, placement: Placement) -> Fate {
        match placement {
            Placement::Inside => self.inside,
            Placement::Outside => self.outside,
            Placement::On { same_way: true, .. } => self.on_same_way,
            Placement::On {
                same_way: false, ..
            } => self.on_opposite_way,
        }
    }
}

/// What becomes of one piece.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fate {
    Drop,
    Keep,
    /// Kept with its outward direction turned round.
    KeepReversed,
    /// Kept only by the earlier of the two solids that share this face, so that
    /// two coincident pieces leave one face behind rather than two.
    KeepIfItComesFirst,
}

/// Which group a piece came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Argument,
    Tool,
}

/// Where a piece sits relative to a group of solids.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Placement {
    Inside,
    Outside,
    /// Lying on a face of one of them.
    On {
        /// Which solid, by its place in the two groups laid end to end.
        solid: usize,
        /// Whether that face and this piece point the same way.
        same_way: bool,
    },
}

/// Whether the piece survives, and whether it is turned round.
///
/// # Panics
///
/// If a "keep once" lands on a piece that is not coincident with anything. That
/// cell only ever appears in the two `ON` columns, so reaching it otherwise
/// means the table and this function have stopped agreeing.
fn keeps(fate: Fate, placement: Placement, mine: usize) -> Option<bool> {
    match fate {
        Fate::Drop => None,
        Fate::Keep => Some(false),
        Fate::KeepReversed => Some(true),
        Fate::KeepIfItComesFirst => match placement {
            Placement::On { solid, .. } => (mine < solid).then_some(false),
            other => panic!("keep-once asked about a piece that is on nothing: {other:?}"),
        },
    }
}

/// Where one piece sits relative to a set of solids.
///
/// Inside beats on, and on beats outside: a piece lying on one solid of a group
/// but buried in another is inside the union of them.
///
/// # Errors
///
/// [`Error::Undecidable`] when no candidate point inside the piece could be
/// placed against every solid in the set.
#[expect(
    clippy::too_many_arguments,
    reason = "every one of these is a different thing the placement depends on, and bundling \
              them into a struct would only move the list"
)]
fn place_the_piece(
    model: &Model,
    solids: &[&BakedSolid],
    against: &[usize],
    candidates: &[PredicatePoint],
    piece: &Piece,
    in_plane: Option<&InPlane>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Placement> {
    if against.is_empty() {
        return Ok(Placement::Outside);
    }
    for &candidate in candidates {
        if the_piece_holds(piece, in_plane, candidate) != Some(true) {
            continue;
        }
        let mut trial = ProvenanceLog::new();
        let settled = settle_against_each(
            model, solids, against, candidate, piece, in_plane, policy, &mut trial,
        )?;
        if let Some(placement) = settled {
            trial.drain_into(log);
            return Ok(placement);
        }
    }
    Err(gave_up(
        INSIDE_A_PIECE,
        &corners_of(piece).concat(),
        Measures::Length,
        log,
    ))
}

/// Whether one piece of a face holds a point on its surface, or `None` when
/// nothing here can say.
///
/// **A flat piece is wound and a curved one is not**, and §35.2 is why rather
/// than this being two answers to one question. A closed curve cut on a *plane*
/// by a ball or a pipe bounds the part of the plane inside that ball or pipe —
/// one of the two sides is bounded and the other runs on for ever, so which is
/// which is settled without asking anything, and that is what lets a round wire
/// join a crossing count as `±1`. On a **closed** surface both sides are
/// bounded, a circle on a sphere leaves two caps and neither is the outer one,
/// so there is nothing to wind and what answers instead is which side of each
/// bounding curve the piece's own material lies on — which the arrangement
/// already worked out and put on [`FaceMeet::material_side`].
///
/// The one thing a *flat* piece may not have any more is a polygon: the disk a
/// bar takes out of a block's face is bounded by one closed curve and has one
/// corner. Its wire contributes to the count exactly as a round hole does, which
/// is `holds.rs`'s rule read the same way round.
fn the_piece_holds(
    piece: &Piece,
    in_plane: Option<&InPlane>,
    point: PredicatePoint,
) -> Option<bool> {
    // A piece that is the **whole** of a closed surface is trimmed by nothing.
    // Its one wire is a seam — where the surface's own two coordinates were cut
    // open, not where the face stops — so every point of the surface is in it,
    // and reading the seam as a trimming would keep half a ball (§38.1).
    if piece.is_the_whole_surface {
        return Some(true);
    }
    let Some(in_plane) = in_plane else {
        return Some(
            piece
                .wires
                .iter()
                .flatten()
                .all(|side| side_of_surface(&side.carried_by[1], point) == side.material_side),
        );
    };
    // **A flat piece's corners are a polygon and its sides need not be**
    // (§38.5). Where a bar's wall cuts a plane the side between two corners is an
    // arc, and the chord across it is on the other side of the crossing: the
    // midway point of the chord is inside the bar where the piece is outside it,
    // so a winding count over the corners alone hands classification a point the
    // piece does not hold. What rules it out is the sign the arrangement already
    // wrote down — the side of the partner this piece's material is on — read
    // here for a flat piece exactly as the curved branch above reads it.
    for side in piece.wires.iter().flatten() {
        if side.carried_by[1].as_plane().is_some() {
            continue;
        }
        match side_of_surface(&side.carried_by[1], point) {
            found if found == side.material_side => (),
            Decision::Neg | Decision::Pos | Decision::Zero => return Some(false),
            Decision::Undecidable { .. } | _ => return None,
        }
    }
    let mut winding = 0;
    for (position, wire) in piece.wires.iter().enumerate() {
        let inwards = if position == 0 { 1 } else { -1 };
        match round_wire_of(wire) {
            Some(partner) => match side_of_surface(&partner, point) {
                Decision::Neg => winding += inwards,
                Decision::Pos => (),
                Decision::Zero | Decision::Undecidable { .. } | _ => return None,
            },
            None => {
                let corners: Vec<PredicatePoint> =
                    wire.iter().map(|side| side.ends[0].point()).collect();
                winding += in_plane.winding_around(core::slice::from_ref(&corners), point)?;
                for side in wire {
                    winding += in_the_bulge_of(side, in_plane, point)?;
                }
            }
        }
    }
    Some(winding == 1)
}

/// One when `point` is between a curved side of a flat piece and the chord
/// across it, where the corners alone wind nothing round it; zero when it is
/// not; `None` when the chord cannot say.
///
/// **A half disk has two corners**, so the polygon of its corners has no
/// inside at all and every point of the half disk is between its arc and its
/// chord. `z ≤ 0.5` cut out of a half ball leaves one on the plane `z = 0.5`.
///
/// Only a side whose material is **inside** a ball is counted. What the ball
/// leaves on the plane is then one disk, and the material is on the left of the
/// side, so the disk is too: the chord is inside it, and the side is the part
/// of its rim on the chord's right. So the part of the disk right of the chord
/// is exactly the bulge, and `point` is already known to be in the disk,
/// because [`the_piece_holds`] asks every curved side's partner first. A side
/// whose material is outside its ball bites into the piece instead, and that
/// bite is what the ball's own sign already took out.
///
/// **A pipe the plane crosses leaves an ellipse and the same argument holds**,
/// but it is not counted yet. Counting it answers two pinned refusals over bars
/// (`degenerate_curved_fuses.rs`) and turns two more into four faces along one
/// edge, because two arcs of one ellipse between the same corners are then
/// named as one edge (`assemble`'s `which_arc_of_a_circle_this_is` reads a
/// ball's circle only).
///
/// **A cone is not counted either, because a plane steeper than its side cuts
/// it in two branches and "inside the cone" is then two regions, not one.** A
/// cone face is on one nappe, but the fuse cuts a plane along the whole double
/// cone and the cone's sign is inside in both nappes. Where the cone's axis
/// crosses such a plane — the point [`half_way_into_either_arc`] builds round —
/// can be in the other nappe's region: inside by the sign, on the bulge side of
/// every chord of the near branch, and outside the solid.
/// `tests/a_plane_across_a_cone.rs` shows each step on a frustum and a leaning
/// block. Counting a cone cut in an ellipse would still meet the pipe's
/// edge-naming gap: with cones counted, two corner-up-cube fuses in
/// `how_often_a_fuse_with_a_cone_refuses.rs` become four faces along one edge.
fn in_the_bulge_of(side: &FaceMeet, in_plane: &InPlane, point: PredicatePoint) -> Option<i32> {
    let a_ball = matches!(side.carried_by[1], Surface::Sphere(_));
    if !a_ball || side.material_side != Decision::Neg {
        return Some(0);
    }
    // Compared rather than matched, so that `Decision`'s wildcard arm is not
    // written once more (§40): a zero or an undecided turn is `None` either way.
    let turn = in_plane.turn(side.ends[0].point(), side.ends[1].point(), point);
    if turn == Decision::Neg {
        Some(1)
    } else if turn == Decision::Pos {
        Some(0)
    } else {
        None
    }
}

/// The surface that cut a wire made of one closed curve, or `None` for a wire
/// that is a polygon.
///
/// A wire with one side whose two ends are the same corner goes right round a
/// closed curve. On a plane that curve was cut by a ball or a pipe, because two
/// planes cross in a straight line and a straight line does not close.
///
/// **So does a wire of two sides along one ball**, which is the same circle
/// cut in two because it started at the ball's pole (§194). Its two corners
/// are a polygon of no area, so read as one it winds round nothing and the disk
/// it bounds would hold no point at all.
fn round_wire_of(wire: &[FaceMeet]) -> Option<Surface> {
    match wire {
        [only] if only.ends[0].point() == only.ends[1].point() => Some(only.carried_by[1]),
        [one, other]
            if matches!(one.carried_by[1], Surface::Sphere(_))
                && one.carried_by[1] == other.carried_by[1] =>
        {
            Some(one.carried_by[1])
        }
        _ => None,
    }
}

/// Where one point sits relative to each solid in the set, or `None` when this
/// point cannot say.
///
/// A point that lands exactly on an edge or a corner of one of the solids is a
/// point that says nothing about the piece around it: the piece is on that
/// solid's skin only if it is coincident with a whole face of it. So a boundary
/// answer with no coincident face to go with it abandons this candidate rather
/// than being read as `ON`.
///
/// An undecided answer asks the same question rather than giving up on the
/// spot, and that is what settles `A \ A` on a turned solid. Every corner there
/// is a box, so a candidate inside a piece is a box too, and a box sitting on
/// the very plane it was solved from straddles that plane: no ray can be
/// trusted and classification honestly says so. The ray was never what settles
/// `ON`. §17.1 makes the stored plane authoritative, [`coincident_face`] proves
/// two planes are the same plane from their coefficients alone, and the winding
/// proves the point is within the face's outline — neither of which cares how
/// wide the box is.
#[expect(
    clippy::too_many_arguments,
    reason = "same list as `place_the_piece`, one point further in"
)]
fn settle_against_each(
    model: &Model,
    solids: &[&BakedSolid],
    against: &[usize],
    point: PredicatePoint,
    piece: &Piece,
    in_plane: Option<&InPlane>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<Placement>> {
    let mut on = None;
    for &index in against {
        // A **curved** piece is asked about coincidence before the ray and not
        // after it. A candidate on a plane is exactly on it and a ray fired at a
        // solid whose face is that plane comes back `OnBoundary`; a candidate on
        // a *sphere* is worked out through the surface's own two coordinates and
        // lands a rounding off it, so the same ray comes back confidently Inside
        // or Outside about a face the piece is lying on. Coincidence is settled
        // by the surfaces and the outlines and never by the ray (§17.1), so
        // asking it first costs nothing and reaching for it afterwards would be
        // too late.
        if in_plane.is_none()
            && let Some(face) = coincident_face(model, solids[index], piece, point, policy, log)?
        {
            if on.is_none() {
                on = Some(Placement::On {
                    solid: index,
                    same_way: face,
                });
            }
            continue;
        }
        let solid: SolidView<'_> = model.solid_at(solids[index].key.slot_in(model.id()));
        let mut ray = ProvenanceLog::new();
        let placed = solid.classify_uncertain_point(point, policy, &mut ray);
        // A classification nobody could make is not a decision anybody took, so
        // it says nothing in the log (§5) — and criterion 7 would otherwise read
        // it as slop this fuse had accepted, which is exactly the conflation
        // §17.2 spends a section keeping apart.
        if !matches!(placed, Containment::Undecidable { .. }) {
            ray.drain_into(log);
        }
        match placed {
            Containment::Inside => return Ok(Some(Placement::Inside)),
            Containment::Outside => (),
            // A face nobody can follow is not a piece that could be placed
            // somewhere else, and reading it as `Outside` would drop material.
            // This was `unreachable!`, on the grounds that the fuse refuses
            // curved solids at entry; `docs/decisions.md` §35 records why that
            // argument stops holding and why what replaces it is a refusal
            // about a caller's solid rather than a panic.
            Containment::CannotFollowAFacesTrimming { .. }
            | Containment::CannotFireARayAtAFacesSurface { .. } => return Ok(None),
            Containment::OnBoundary | Containment::Undecidable { .. } => {
                let Some(face) = coincident_face(model, solids[index], piece, point, policy, log)?
                else {
                    return Ok(None);
                };
                if on.is_none() {
                    on = Some(Placement::On {
                        solid: index,
                        same_way: face,
                    });
                }
            }
        }
    }
    Ok(Some(on.unwrap_or(Placement::Outside)))
}

/// Whether the face of `solid` that `point` lies on faces the same way as
/// `facing`, or `None` when no face of it holds that point.
///
/// # Errors
///
/// [`Error::Undecidable`] when two planes cannot be told apart at all.
fn coincident_face(
    model: &Model,
    solid: &BakedSolid,
    piece: &Piece,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<bool>> {
    let mine = outward_plane_of(piece);
    for face in &solid.faces {
        let Some(same_way) = do_they_face_the_same_way(piece, mine, face, policy, log)? else {
            continue;
        };
        if a_face_of_the_input_holds(model, solid, face, piece, point, policy, log)? {
            return Ok(Some(same_way));
        }
    }
    Ok(None)
}

/// Whether a piece and an input face are on one surface, and if so whether they
/// face the same way — or `None` when they are not on one surface.
///
/// §31.3 splits this in two and the split is what makes one rule cover all three
/// surface kinds. **Is it one surface** is geometry, and for a plane
/// [`Plane::agreement_with`] answers it and the direction together, because a
/// reversed plane is a different four numbers. A sphere has no reversed form, so
/// for a curved pair the surfaces settle the first half and the two faces' own
/// **orientations** settle the second.
fn do_they_face_the_same_way(
    piece: &Piece,
    mine: Option<Plane>,
    face: &BakedFace,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<bool>> {
    if let (Some(mine), Some(theirs)) = (mine, outward_plane_of_face(face)) {
        return Ok(match agreement_of_two_planes(&mine, &theirs, log)? {
            PlaneAgreement::SameWay => Some(true),
            PlaneAgreement::OppositeWay => Some(false),
            PlaneAgreement::Different => None,
            other => {
                panic!("plane agreement came back with an answer nothing here knows: {other:?}")
            }
        });
    }
    if !matches!(
        intersect(&piece.surface, &face.surface, policy, log)?,
        SurfaceMeet::SameSurface
    ) {
        return Ok(None);
    }
    Ok(Some(piece.orientation == face.orientation))
}

/// Whether the material of one face of an input holds a point already known to
/// be on that face's surface.
///
/// A flat face is wound, the way it always was. A **curved** one is
/// [`SolidView::face_holds_the_point`], which is the one authority on that
/// question (§35) and which the fuse can call here because what it is asked
/// about is a face of an *input* — a face the model holds, with corners it holds
/// — rather than a piece the arrangement solved for.
fn a_face_of_the_input_holds(
    model: &Model,
    solid: &BakedSolid,
    face: &BakedFace,
    piece: &Piece,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    if let Some(flat) = piece.surface.as_plane() {
        let Some(in_plane) = InPlane::of(flat) else {
            return Ok(false);
        };
        let corners: Vec<Vec<PredicatePoint>> = face
            .wires
            .iter()
            .map(|wire| wire.iter().map(|edge| edge.start.point).collect())
            .collect();
        // Either sign means the point is in this face's material. Which one
        // comes out says only which way that face's outline runs round *this*
        // piece's normal, and a face pressed against this one from the other
        // side runs the other way — which is what the agreement above already
        // said. Zero is the only answer that puts the point outside it.
        return Ok(in_plane
            .winding_around(&corners, point)
            .is_some_and(|winding| winding != 0));
    }
    let view = model.solid_at(solid.key.slot_in(model.id()));
    Ok(view.face_holds_the_point(face.keys[0], point, policy, log)? != crate::OnAFace::NotHeld)
}

/// The plane a piece's material is *behind*: the piece's own plane, turned round
/// when its solid uses the face the other way.
///
/// Everything the ON columns need is here. Two coincident faces face the same
/// way exactly when these two planes do, and comparing the stored planes alone
/// would call a face and the face pressed against it from the other side the
/// same thing.
fn outward_plane_of(piece: &Piece) -> Option<Plane> {
    let flat = *piece.surface.as_plane()?;
    Some(if piece.orientation.is_reversed() {
        flat.reversed()
    } else {
        flat
    })
}

/// The plane of an input face, turned to point out of the material, or `None`
/// when the face is curved and has no such thing.
fn outward_plane_of_face(face: &BakedFace) -> Option<Plane> {
    let plane = *face.surface.as_plane()?;
    Some(if face.orientation.is_reversed() {
        plane.reversed()
    } else {
        plane
    })
}

fn corners_of(piece: &Piece) -> Vec<Vec<PredicatePoint>> {
    piece
        .wires
        .iter()
        .map(|wire| wire.iter().map(|side| side.ends[0].point()).collect())
        .collect()
}

fn which_solid_owns_each_face(model: &Model, solids: &[&BakedSolid]) -> BTreeMap<FaceSlot, usize> {
    let mut owner = BTreeMap::new();
    for (index, solid) in solids.iter().enumerate() {
        for face in &solid.faces {
            for key in &face.keys {
                owner.insert(key.slot_in(model.id()), index);
            }
        }
    }
    owner
}

/// A face's plane with a point off it whose side has been proved.
///
/// That point is the whole trick. Three points on the plane turn anticlockwise
/// about its normal exactly when `det(q - p, r - p, n)` is positive, and for any
/// point `s = p - t·n` with `t > 0` the determinant `orient3d(p, q, r, s)` is
/// `t` times that — the terms carrying `n` twice drop out. So an ordinary
/// [`orient3d_of_points`] against a point *below* the plane is the in-plane
/// turn, exactly, on corners that are boxes rather than numbers. Below is what
/// [`Plane::side_of_point`] returning [`Decision::Neg`] proves, and a point
/// above works just as well with the sign turned round.
///
/// Nothing is projected into the plane's own two coordinates, which would round
/// exact corners into inexact ones and manufacture the undecidability M2.5
/// exists to measure (§15.5).
pub(crate) struct InPlane {
    from: PredicatePoint,
    /// Whether [`InPlane::from`] came out above the plane rather than below it.
    turned_round: bool,
}

impl InPlane {
    /// A reference point off `plane`, or `None` when none of the tries could be
    /// proved to be off it.
    ///
    /// **The reach is a length that scales with the plane, and never shorter
    /// than one.** A step of a fixed size vanishes into a plane far enough out —
    /// past an offset of about `2^72` the `f64` spacing there is wider than a
    /// step of `2^20`, so every candidate lands back exactly on the plane and a
    /// perfectly ordinary fuse has no answer. The plane's own offset is the only
    /// length in sight that scales with it. The floor of one is not tidiness:
    /// dropping it and stepping off a plane `0.05` from the origin by `0.05`
    /// leaves the reference point so close to the face that the winding fan
    /// cannot be decided, and §17.7's headline refusal rate goes from 20.7% to
    /// 37.9% — 72 fuses that could not place a point inside a piece of a face.
    ///
    /// The two reaches are only tries: each is *proved* to be on one side or
    /// abandoned, so which one lands decides nothing. **The second is the same
    /// distance the other way**, and it is what a plane out at `1e308` needs,
    /// where stepping further from the origin overflows and stepping towards it
    /// cannot. `a_fuse_that_needs_a_decision_nothing_can_prove_refuses` is where
    /// that runs: with only the first try the fuse there stops before reaching
    /// any arithmetic. Stepping towards the origin *first* was measured too, and
    /// it is the 37.9% again — a face far from the origin ends up with a
    /// reference point a whole offset away from it, and the fan degenerates.
    pub(crate) fn of(plane: &Plane) -> Option<Self> {
        let origin = plane.origin().coordinates();
        let normal = plane.normal().components();
        let out_here = plane.offset().abs().max(1.0);
        for reach in [out_here, -out_here] {
            let from = PredicatePoint::from([
                origin[0] + normal[0] * reach,
                origin[1] + normal[1] * reach,
                origin[2] + normal[2] * reach,
            ]);
            match plane.side_of_point(from) {
                Decision::Pos => {
                    return Some(Self {
                        from,
                        turned_round: true,
                    });
                }
                Decision::Neg => {
                    return Some(Self {
                        from,
                        turned_round: false,
                    });
                }
                Decision::Zero | Decision::Undecidable { .. } | _ => (),
            }
        }
        None
    }

    /// How many times the wires wind round `point`: one inside the material,
    /// zero outside it and inside a hole. `None` when the point sits on the
    /// outline itself, or when a turn could not be decided.
    ///
    /// Every corner of the outer wire is tried as the apex in turn, and the
    /// first fan that can answer does. The count does not depend on which
    /// corner the fan starts from, but where its chords run does, and a point
    /// sitting exactly on a chord is on the boundary of two of the triangles
    /// with neither able to say whether it is in it. Moving the apex moves the
    /// chords off the point.
    ///
    /// A rectangle is what forces this rather than a stray case: every point
    /// [`candidate_points`] can offer for a rectangular piece lies on one of
    /// its two diagonals or on an edge, so a fan from one fixed corner leaves
    /// two rectangles pressed together with nothing to say about each other.
    pub(crate) fn winding_around(
        &self,
        wires: &[Vec<PredicatePoint>],
        point: PredicatePoint,
    ) -> Option<i32> {
        wires
            .first()?
            .iter()
            .find_map(|&apex| self.winding_fanned_from(apex, wires, point))
    }

    /// The count from one apex, exactly the way
    /// [`FaceView::for_each_triangle`](crate::FaceView::for_each_triangle) fans
    /// a face: the signed triangles add up to the polygon, holes included,
    /// because a hole is wound the other way and subtracts itself.
    fn winding_fanned_from(
        &self,
        apex: PredicatePoint,
        wires: &[Vec<PredicatePoint>],
        point: PredicatePoint,
    ) -> Option<i32> {
        let mut winding = 0;
        for wire in wires {
            for (index, &from) in wire.iter().enumerate() {
                let to = wire[(index + 1) % wire.len()];
                if from == apex || to == apex {
                    continue;
                }
                winding += self.inside_triangle(apex, from, to, point)?;
            }
        }
        Some(winding)
    }

    /// `1` or `-1` when `point` is strictly inside the triangle, signed the way
    /// the triangle turns; `0` when it is outside; `None` when it is on the
    /// triangle's own boundary or a turn could not be decided.
    fn inside_triangle(
        &self,
        first: PredicatePoint,
        second: PredicatePoint,
        third: PredicatePoint,
        point: PredicatePoint,
    ) -> Option<i32> {
        let mut positives = 0;
        let mut negatives = 0;
        let mut zeros = 0;
        for [from, to] in [[first, second], [second, third], [third, first]] {
            match self.turn(from, to, point) {
                Decision::Pos => positives += 1,
                Decision::Neg => negatives += 1,
                Decision::Zero => zeros += 1,
                Decision::Undecidable { .. } | _ => return None,
            }
        }
        if positives > 0 && negatives > 0 {
            return Some(0);
        }
        if zeros > 0 {
            return None;
        }
        Some(if positives > 0 { 1 } else { -1 })
    }

    /// Which way `first`, `second`, `third` turn about the plane's normal:
    /// [`Decision::Pos`] anticlockwise.
    pub(crate) fn turn(
        &self,
        first: PredicatePoint,
        second: PredicatePoint,
        third: PredicatePoint,
    ) -> Decision {
        let decision = orient3d_of_points(first, second, third, self.from);
        if self.turned_round {
            the_other_way(decision)
        } else {
            decision
        }
    }
}

/// Points that might be inside the piece, in a fixed order.
///
/// Nothing here is trusted: the caller proves a candidate is inside before using
/// it. The weights are powers of two so that a candidate from an exactly
/// representable piece is exactly representable itself, which is what lets the
/// cascade prove anything about it at all.
///
/// Four families now, and the two new ones are what a piece bounded by a closed
/// curve needs. A corner and the middle of the side facing it is inside every
/// triangle and every convex corner, which is most pieces on the first try. A
/// diagonal's middle is what is left for a piece with a dent in it, and a
/// polygon with more than three corners always has a diagonal.
///
/// **A disk has one corner and no diagonal**, so neither of those reaches it:
/// every blend of its one corner is that corner, which is on the boundary. What
/// does reach it is the middle of the curve that bounds it — where the ball or
/// pipe that cut it has its axis, which is a point of the plane provably inside
/// that ball or pipe.
///
/// **A curved piece's corners are not on its surface once they are blended**,
/// and a point off the surface says nothing about a region drawn on it. Every
/// blend is therefore put back onto the surface through its own two coordinates.
/// That is `atan2` and it is allowed here for the reason §5 gives: *choosing*
/// which point to look at decides nothing, and what follows proves the point is
/// inside the piece with signs that never see a coordinate.
fn candidate_points(piece: &Piece, wires: &[Vec<PredicatePoint>]) -> Vec<PredicatePoint> {
    let mut candidates = Vec::new();
    for wire in wires {
        for (index, &corner) in wire.iter().enumerate() {
            let before = wire[(index + wire.len() - 1) % wire.len()];
            let after = wire[(index + 1) % wire.len()];
            candidates.push(midway_between(corner, midway_between(before, after)));
        }
    }
    let corners: Vec<PredicatePoint> = wires.concat();
    for (index, &one) in corners.iter().enumerate() {
        for &other in &corners[index + 1..] {
            candidates.push(midway_between(one, other));
        }
    }
    if let Some(flat) = piece.surface.as_plane() {
        for wire in &piece.wires {
            if let Some(partner) = round_wire_of(wire) {
                candidates.extend(the_middle_of_a_round_wire(flat, &partner));
            }
        }
        for side in piece.wires.iter().flatten() {
            candidates.extend(half_way_into_either_arc(flat, side).into_iter().flatten());
        }
        return candidates;
    }
    // **A piece bounded by one closed curve has one corner**, and every blend of
    // one corner is that corner, which is on the boundary. A cap a plane cuts
    // off a ball is exactly that. What reaches it is six points spread round the
    // surface from the middle of what corners there are — nothing is trusted
    // about them either, and the six are a fixed set so which one answers does
    // not depend on anything but the piece.
    let middle = middle_of_the_corners(wires);
    let reach = how_big_the_surface_is(&piece.surface, middle);
    for axis in 0..3 {
        for way in [1.0, -1.0] {
            let mut step = [0.0; 3];
            step[axis] = way * reach;
            candidates.push(PredicatePoint::from(
                [0, 1, 2].map(|which| middle.coordinates()[which] + step[which]),
            ));
        }
    }
    candidates
        .into_iter()
        .filter_map(|candidate| back_onto_the_surface(&piece.surface, candidate))
        .collect()
}

/// Two points a quarter of the way across the circle a curved side of a flat
/// piece runs round, one either side of the chord between its ends: half way
/// from the middle of the chord to where the circle crosses the line square to
/// it. One of them is between the side and its chord, which is where a half
/// disk has all of its inside and its corners none (see [`in_the_bulge_of`]).
///
/// Which one is not asked, because nothing about a candidate is trusted: each
/// is placed and proved by the caller. The circle's centre is
/// [`the_middle_of_a_round_wire`], and its radius comes from the right angle
/// at the chord's middle, so for an ellipse the points are near rather than on
/// that line, which is all a candidate needs. `None` for a straight side, and
/// for a side with nothing to go round or no length to go square to.
fn half_way_into_either_arc(flat: &Plane, side: &FaceMeet) -> Option<[PredicatePoint; 2]> {
    let centre = the_middle_of_a_round_wire(flat, &side.carried_by[1])?;
    let [start, end] = [0, 1].map(|which| middle_of(side.ends[which].point()));
    let chord = end - start;
    let across = flat.normal().cross(chord).normalized().ok()?;
    let middle = start + chord * 0.5;
    let off = across.dot(middle_of(centre) - middle);
    let reach = (off * off + chord.length_squared() / 4.0).sqrt();
    let points = [off + reach, off - reach].map(|to| middle + across * (to / 2.0));
    points
        .iter()
        .all(Point::is_finite)
        .then(|| points.map(|point| PredicatePoint::from(point.coordinates())))
}

/// The middle of a piece's corners, which is somewhere near it and nowhere in
/// particular on it.
fn middle_of_the_corners(corners: &[Vec<PredicatePoint>]) -> Point {
    let mut summed = [0.0; 3];
    let mut how_many = 0.0;
    for corner in corners.iter().flatten() {
        let at = middle_of(*corner).coordinates();
        for axis in 0..3 {
            summed[axis] += at[axis];
        }
        how_many += 1.0;
    }
    if how_many == 0.0 {
        return Point::origin();
    }
    Point::from(summed.map(|total| total / how_many))
}

/// How far across a curved surface is near `there`: the radius of the ball or
/// the pipe, and for a cone the radius it has where the piece sits.
///
/// **A cone is the reason this takes a point.** A ball and a pipe are the same
/// size everywhere, so a step of "one radius" means the same wherever it is
/// taken; a cone widens as it goes, so a step sized on the surface as a whole
/// would be meaningless — there is no such number, and
/// [`Surface::furthest_a_unit_step_goes`] says so by handing back infinity. What
/// a step wants is how big the cone is *here*, which is one multiplication.
fn how_big_the_surface_is(surface: &Surface, there: Point) -> f64 {
    match surface {
        Surface::Sphere(ball) => ball.radius(),
        Surface::Cylinder(pipe) => pipe.radius(),
        Surface::Cone(cone) => cone
            .radius_at(cone.axis().direction().dot(there - cone.apex()))
            .max(f64::MIN_POSITIVE),
        Surface::Plane(_) | _ => 1.0,
    }
}

/// Where the ball or pipe that cut a round wire has its middle, on the plane
/// that wire is drawn on.
///
/// The centre of a circle cut on a plane, without ever saying the word: for a
/// pipe it is where the axis meets the plane, and for a ball it is the foot of
/// the perpendicular from the centre. Both are at distance nothing from the
/// axis or centre, so both are provably inside — which is what the winding
/// count then checks, in exact arithmetic and with no trust placed here.
///
/// `None` where the division has nothing to divide by, which is a pipe running
/// along the plane rather than through it — and then the wire it cut is not a
/// closed curve and this was not asked.
fn the_middle_of_a_round_wire(flat: &Plane, partner: &Surface) -> Option<PredicatePoint> {
    let (normal, offset) = (flat.normal(), flat.offset());
    let from = match partner {
        Surface::Sphere(ball) => ball.centre(),
        Surface::Cylinder(pipe) => {
            let axis = pipe.axis();
            let along = normal.dot(axis.direction());
            if along == 0.0 {
                return None;
            }
            axis.origin()
                + axis.direction()
                    * ((offset - normal.dot(axis.origin() - Point::origin())) / along)
        }
        // **The same point, and it takes a proof rather than a picture.** Where
        // the cone's axis meets the plane is inside the ellipse they cut, and
        // not obviously: the ellipse's centre is *not* the axis point, it is
        // `s_c = −(1 + t²)hAk/E` along the plane's slope from it. The gap
        // between the two works out to `|h|kt²/(|A|E)` against a semi-axis of
        // `|h|t/E`, so the axis point sits `tk/|A|` of the way out — and `|A| >
        // tk` is exactly the condition that made this an ellipse in the first
        // place. So it is strictly inside, always, for the same one sign the
        // intersector already decided.
        Surface::Cone(cone) => {
            let axis = cone.axis();
            let along = normal.dot(axis.direction());
            if along == 0.0 {
                return None;
            }
            axis.origin()
                + axis.direction()
                    * ((offset - normal.dot(axis.origin() - Point::origin())) / along)
        }
        Surface::Plane(_) | _ => return None,
    };
    let there =
        from + normal * ((offset - normal.dot(from - Point::origin())) / normal.dot(normal));
    there
        .is_finite()
        .then(|| PredicatePoint::from(there.coordinates()))
}

/// A point put back onto the surface it is meant to be a point of.
///
/// Through the surface's own two coordinates and out again, which rounds — so
/// what comes back is a **box** [`Surface::ROUND_TRIP_ROUNDOFF`] wide rather
/// than a triple. That width is not slop anybody accepted: a candidate a
/// rounding off a curved face would be classified confidently against a solid
/// whose own face it is lying on, and `ON` is the one column a ray was never
/// what settled (§17.1).
fn back_onto_the_surface(surface: &Surface, candidate: PredicatePoint) -> Option<PredicatePoint> {
    let at = middle_of(candidate);
    let (u, v) = surface.coordinates_of(at);
    let there = surface.point_at(u, v);
    if !there.is_finite() {
        return None;
    }
    let reach = Surface::ROUND_TRIP_ROUNDOFF * there.largest_coordinate_magnitude().max(1.0);
    Some(PredicatePoint::Constructed(there.coordinates().map(
        |coordinate| {
            Interval::around(coordinate, reach)
                .expect("the check just above sent every point that is not a place back as None")
        },
    )))
}

/// Halfway between two points, which is exact whenever both ends are.
fn midway_between(one: PredicatePoint, other: PredicatePoint) -> PredicatePoint {
    let (one, other) = (one.enclosure(), other.enclosure());
    let half = Interval::exactly(0.5);
    as_exact_as_it_is(PredicatePoint::Constructed(
        [0, 1, 2].map(|axis| (one[axis] + other[axis]) * half),
    ))
}

/// The name the search for a point inside a piece goes by in provenance records.
const INSIDE_A_PIECE: &str = "a point inside a piece of a face";

/// The name the search for a point off a face's plane goes by.
const OFF_A_PLANE: &str = "a point off a face's plane";

fn nothing_to_measure_from(piece: &Piece, log: &mut ProvenanceLog) -> Error {
    let normal = piece.plane().normal().components();
    gave_up(
        OFF_A_PLANE,
        &[PredicatePoint::from(normal)],
        Measures::Length,
        log,
    )
}
