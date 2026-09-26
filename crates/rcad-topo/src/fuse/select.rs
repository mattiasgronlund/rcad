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

use rcad_geom::{
    Cone, EdgeRun, Picker, Plane, PlaneAgreement, Point, Surface, SurfaceMeet, Torus, Vector,
    intersect,
};
use rcad_predicates::{
    Decision, Interval, Measures, Point as PredicatePoint, Policy, ProvenanceLog,
    orient3d_of_points,
};

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
    assert_eq!(
        scratch.pieces.len(),
        scratch.cut_from.len(),
        "every piece is cut from one operand"
    );

    let mut chosen = Vec::new();
    for (index, (piece, &mine)) in scratch.pieces.iter().zip(&scratch.cut_from).enumerate() {
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
/// but buried in another is inside the union of them. So is a piece lying on
/// two of them that face opposite ways — see [`LyingOn::placement`].
///
/// # Errors
///
/// [`Error::Undecidable`], under one of **two** names, because two searches
/// happen here and a caller that cannot tell them apart is told the wrong
/// thing. [`INSIDE_A_PIECE`] when no candidate was inside the piece at all;
/// [`PLACED_AGAINST_EVERY_SOLID`] when one was and it could not be classified
/// against every solid in the set. The first says to look at the piece's
/// trimming, the second says the piece is fine and a surface further down gave
/// up — the lens in §216.4 is the second wearing the first's name.
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
    let also_picked_by = what_else_each_round_wire_asks(piece, policy, log)?;
    let mut one_was_inside = false;
    for &candidate in candidates {
        if the_piece_holds(piece, in_plane, &also_picked_by, candidate) != Some(true) {
            continue;
        }
        one_was_inside = true;
        let mut trial = ProvenanceLog::new();
        let settled = settle_against_each(
            model, solids, against, candidate, piece, in_plane, policy, &mut trial,
        )?;
        if let Some(placement) = settled {
            trial.drain_into(log);
            return Ok(placement);
        }
    }
    let which_search_gave_up = if one_was_inside {
        PLACED_AGAINST_EVERY_SOLID
    } else {
        INSIDE_A_PIECE
    };
    Err(gave_up(
        which_search_gave_up,
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
///
/// A round wire's partner's inside is the region it bounds only where the
/// partner leaves one region on the plane. A torus leaves two, so
/// `also_picked_by` — one entry per wire, from [`what_else_each_round_wire_asks`]
/// — says what else must hold the point, or `None` where nothing can say.
fn the_piece_holds(
    piece: &Piece,
    in_plane: Option<&InPlane>,
    also_picked_by: &[Option<Vec<Picker>>],
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
    let flat = piece
        .surface
        .as_plane()
        .expect("a piece measured from inside a plane is a flat piece");
    let mut winding = 0;
    for (position, wire) in piece.wires.iter().enumerate() {
        let inwards = if position == 0 { 1 } else { -1 };
        match round_wire_of(wire) {
            Some(partner) => match side_of_surface(&partner, point) {
                Decision::Neg => {
                    let pickers = also_picked_by[position].as_deref()?;
                    if every_picker_keeps(pickers, point)? {
                        winding += inwards;
                    }
                }
                Decision::Pos => (),
                Decision::Zero | Decision::Undecidable { .. } | _ => return None,
            },
            None => {
                let corners: Vec<PredicatePoint> =
                    wire.iter().map(|side| side.ends[0].point()).collect();
                winding += in_plane.winding_around(core::slice::from_ref(&corners), point)?;
                for side in wire {
                    winding += in_the_bulge_of(flat, side, in_plane, point)?;
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
/// **A pipe is counted the same way, where the plane crosses it**, which is
/// [`Cylinder::is_cut_in_a_closed_curve_by`](rcad_geom::Cylinder::is_cut_in_a_closed_curve_by)
/// — the sign of `n·a`, and the only thing a pipe can leave on a plane that is
/// not one closed curve is a pair of endless lines. What the ellipse's inside
/// leaves is one convex region, exactly as a circle's is, so the ball's
/// paragraph above carries over word for word.
///
/// **The cost this used to warn of is real, and it is one line in `assemble`.**
/// The warning was that counting the pipe turns two faces into four along one
/// edge, because `which_of_them_a_closed_crossing_runs_round` read a ball's
/// circle and a cone's closed crossing and not a pipe's, so two arcs of one
/// ellipse between the same corners were named as one edge. Measured: with the
/// pipe counted here and not there, two rows of `degenerate_curved_fuses.rs`
/// came back `ResultIsNotManifold { count: 4 }` and nothing else in the suite
/// moved. Naming the pipe there as well turned those two into answers, so the
/// warning was about a missing row and not about a shape the fuse cannot hold.
///
/// What the pair bought, all of it measured: three pinned refusals over bars
/// become answers — a bar cut by one plane off its axis, the control of the
/// tangent-wall row, and two tools meeting on a plane that cuts a round wall —
/// and `fusing_two_bars_side_by_side.rs`'s two orders, which used to give out in
/// different searches, now give out in the same one. Two side-by-side bars still
/// do not fuse; they die at the next rule, which is what §218.5 predicted.
///
/// **A cone is counted where the plane cuts it in a curve that closes**, which
/// is a circle for a cap's own plane and an ellipse for every plane leaned off
/// the axis by less than the cone's own half angle. A cap cut by a block's plane
/// is a piece bounded by an arc of the rim and the chord across it — two
/// corners, no polygon, and no point inside it at all until the bulge is counted
/// (`docs/decisions.md` §200.3's pair refused there), and a block's own face cut
/// by a leaning cone is the same piece bounded by an arc of an ellipse.
///
/// **A shallower plane is counted too, on the side's own half of the cone.** It
/// cuts the double cone in two branches, so "inside the cone" by the sign is two
/// regions on it and not one, and the point where the cone's axis crosses such a
/// plane — the one [`half_way_into_either_arc`] builds round — can be in the
/// other nappe's region: inside by the sign, on the bulge side of every chord of
/// the near branch, and outside the solid. `tests/a_plane_across_a_cone.rs`
/// shows each step on a frustum and a leaning block. That used to be read as
/// the limit, and it is not one: the two regions are told apart by the sign of
/// `(x − apex)·w`, which is one per nappe, and
/// [`is_on_the_nappe_the_side_is_on`] asks it of the point and of both ends of
/// the side. On one nappe the inside is a solid convex cone, so what it leaves
/// on any plane is one convex region — bounded by a hyperbola branch, a
/// parabola or an ellipse — and the chord between two points of its edge is
/// inside it. Everything the ball's paragraph says then carries over. A point
/// on the other half, or one whose half cannot be settled, counts nothing,
/// which can only lose a candidate.
///
/// **The test this asks is one-sided, and it is
/// [`Cone::is_cut_in_a_closed_curve_by`](rcad_geom::Cone::is_cut_in_a_closed_curve_by).**
/// What this count needs is that the cone leave *one* region on the plane and
/// that its boundary bulge the same way all along; a closed crossing is exactly
/// that, and an undecided or open one is read as no bulge, which costs a refusal
/// and never a wrong answer.
///
/// It used to compare the stored normal against the stored axis direction, so
/// only a cap's own plane counted, and `holds.rs` and `assemble.rs` each spelled
/// the question again (§23.8, §208.6). There is one spelling now, in
/// `rcad-geom`, and the ellipse is admitted: the sign
/// `(n·a)²(1 + s²) − s²|n|²|a|² > 0` is exact, and the region an ellipse's
/// inside leaves on a plane is one convex piece just as a circle's is.
fn in_the_bulge_of(
    flat: &Plane,
    side: &FaceMeet,
    in_plane: &InPlane,
    point: PredicatePoint,
) -> Option<i32> {
    let one_round_region = match side.carried_by[1] {
        Surface::Sphere(_) => true,
        Surface::Cylinder(pipe) => pipe.is_cut_in_a_closed_curve_by(flat) == Some(true),
        Surface::Cone(cone) => match cone.is_cut_in_a_closed_curve_by(flat) {
            Some(true) => true,
            Some(false) => is_on_the_nappe_the_side_is_on(&cone, side, point),
            None => false,
        },
        Surface::Plane(_) | _ => false,
    };
    if !one_round_region || side.material_side != Decision::Neg {
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

/// Whether `point` is, provably, on the same half of the double cone as both
/// ends of `side`: the sign of `(x − apex) · w` agreeing for all three.
///
/// `false` whenever that is not proved — the ends straddle the apex, one of the
/// three is on the plane through the apex square to the axis, or a sign could
/// not be settled. `false` only stops a bulge being counted, which can drop a
/// candidate and can never keep a point the piece does not hold.
fn is_on_the_nappe_the_side_is_on(cone: &Cone, side: &FaceMeet, point: PredicatePoint) -> bool {
    let apex = cone.apex().coordinates().map(Interval::exactly);
    let up = cone.axis().direction().components().map(Interval::exactly);
    let how_far_up = |at: PredicatePoint| {
        let at = at.enclosure();
        [0, 1, 2]
            .map(|axis| (at[axis] - apex[axis]) * up[axis])
            .into_iter()
            .fold(Interval::exactly(0.0), |sum, term| sum + term)
            .sign()
    };
    let decided = |sign: &Decision| matches!(sign, Decision::Pos | Decision::Neg);
    let signs = [
        how_far_up(side.ends[0].point()),
        how_far_up(side.ends[1].point()),
        how_far_up(point),
    ];
    signs.iter().all(decided) && signs[0] == signs[1] && signs[1] == signs[2]
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

/// For each wire of a flat piece, the pickers that must also hold a point
/// inside the wire's partner before the point is inside what the wire bounds:
/// none for a ball, a pipe or a cone, which each leave one region on a plane,
/// and `None` where nothing can say.
///
/// **A torus leaves two.** A plane through its axis cuts it in two meridian
/// circles, and the torus's inside on that plane is both their disks, so the
/// torus's sign alone holds a point in the other disk as surely as one in this
/// one. What tells them apart is the flat half-space [`intersect`] names each
/// circle by, having proved it clears both circles: each disk is exactly
/// the part of the tube on its own circle's side, so the wire's own corner
/// says which side is this wire's. A plane square to the axis cuts two
/// concentric circles, named by pipes, and neither circle's disk is the tube's
/// inside on one side of a pipe — so that family, and any wire whose corner no
/// picker certainly keeps, is `None`.
///
/// Asked once per piece rather than once per candidate, because the answer is
/// about the wire and not about the point.
///
/// # Errors
///
/// Whatever [`intersect`] gives for the piece's plane against the torus.
fn what_else_each_round_wire_asks(
    piece: &Piece,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<Option<Vec<Picker>>>> {
    if piece.surface.as_plane().is_none() {
        return Ok(Vec::new());
    }
    let mut asks = Vec::with_capacity(piece.wires.len());
    for wire in &piece.wires {
        asks.push(match round_wire_of(wire) {
            Some(ring @ Surface::Torus(_)) => the_pickers_of_the_meridian_circle_through(
                &piece.surface,
                &ring,
                wire[0].ends[0].point(),
                policy,
                log,
            )?,
            Some(_) | None => Some(Vec::new()),
        });
    }
    Ok(asks)
}

/// The pickers of the one meridian circle `flat` cuts `ring` in that passes
/// through `corner`, or `None` when the two do not cross in meridian circles or
/// no one circle certainly keeps the corner.
///
/// # Errors
///
/// Whatever [`intersect`] gives for the two.
fn the_pickers_of_the_meridian_circle_through(
    flat: &Surface,
    ring: &Surface,
    corner: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<Vec<Picker>>> {
    let SurfaceMeet::Curves(runs) = intersect(flat, ring, policy, log)? else {
        return Ok(None);
    };
    let mut keeping = runs.iter().filter_map(|run| match run.run() {
        EdgeRun::PartOfWhereItCrosses { picked_by, .. }
            if picked_by.iter().all(|picker| {
                matches!(picker, Picker::InFrontOf(_))
                    && picker.side_of_point(corner) == Decision::Pos
            }) =>
        {
            Some(picked_by.to_vec())
        }
        EdgeRun::PartOfWhereItCrosses { .. }
        | EdgeRun::Straight
        | EdgeRun::RightRoundWhereItsFacesCross
        | EdgeRun::RightRoundWhereItCrosses { .. }
        | _ => None,
    });
    Ok(match (keeping.next(), keeping.next()) {
        (Some(only), None) => Some(only),
        _ => None,
    })
}

/// Whether every one of `pickers` certainly keeps `point`: `Some(false)` when
/// one certainly rules it out, `None` when one cannot say or has it on its
/// boundary.
fn every_picker_keeps(pickers: &[Picker], point: PredicatePoint) -> Option<bool> {
    let mut kept = true;
    for picker in pickers {
        match picker.side_of_point(point) {
            Decision::Pos => (),
            Decision::Neg => kept = false,
            Decision::Zero | Decision::Undecidable { .. } | _ => return None,
        }
    }
    Some(kept)
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
    let mut lying_on = LyingOn::default();
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
            && let Some(same_way) =
                coincident_face(model, solids[index], piece, point, policy, log)?
        {
            lying_on.add(index, same_way);
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
        if matches!(placed, Containment::Inside | Containment::Outside)
            && the_ray_and_the_stored_planes_disagree_about(solid, point)
        {
            return Ok(None);
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
                let Some(same_way) =
                    coincident_face(model, solids[index], piece, point, policy, log)?
                else {
                    return Ok(None);
                };
                lying_on.add(index, same_way);
            }
        }
    }
    Ok(Some(lying_on.placement()))
}

/// The solids of a group a piece lies flush on, the first of each way round.
///
/// **The first, because "keep once" reads it.** Of several coincident faces
/// facing the same way, [`Fate::KeepIfItComesFirst`] keeps the piece only when
/// it comes before the solid named here, and `against` runs in the operands'
/// order — so naming the lowest-numbered one is what leaves exactly one of them.
#[derive(Default)]
struct LyingOn {
    same_way: Option<usize>,
    opposite_way: Option<usize>,
}

impl LyingOn {
    fn add(&mut self, solid: usize, same_way: bool) {
        let first = if same_way {
            &mut self.same_way
        } else {
            &mut self.opposite_way
        };
        first.get_or_insert(solid);
    }

    /// Where the piece sits against the group, once none of its solids had the
    /// piece inside it.
    ///
    /// **Flush on one solid facing the same way and on another facing against it
    /// is inside.** The first has its material behind the piece and the second
    /// in front of it, so the group's union has material on both sides and the
    /// piece is buried in it, exactly as it would be inside one solid. Reading
    /// only whichever of the two came first made the answer depend on the order
    /// the solids were named in: `s0 \ (s1 ∪ s2)` was refused where
    /// `s0 \ (s2 ∪ s1)` answered (`tests/a_face_between_two_tools_facing_opposite_ways.rs`).
    fn placement(self) -> Placement {
        match (self.same_way, self.opposite_way) {
            (Some(_), Some(_)) => Placement::Inside,
            (Some(solid), None) => Placement::On {
                solid,
                same_way: true,
            },
            (None, Some(solid)) => Placement::On {
                solid,
                same_way: false,
            },
            (None, None) => Placement::Outside,
        }
    }
}

/// Whether the ray and the arrangement put `point` on opposite sides of one of
/// `solid`'s flat faces.
///
/// **The two read one face from two descriptions.** The arrangement cuts along
/// the face's stored plane, which is the face (§17.1). The ray fans the face's
/// corners into triangles and asks which side of *those* the point is on
/// (`classify.rs`'s `count_a_polygon`), and a turned solid's corners are not on
/// its stored planes: turning a block a quarter turn about the `z` axis leaves
/// the face through the axis with corners `(∓6, ±8.9e-16, z)`, whose triangles
/// hold the axis exactly, while its stored plane misses the axis by 4.4e-16
/// (`a_knife_through_the_axis` in `tests/a_narrow_lune_from_a_pole_corner.rs`).
/// A point between the two is certified on one side by the ray and was proved
/// on the other side by the arrangement, and the piece it stands for is then
/// placed on the wrong side of the solid (§200.1).
///
/// So a candidate either description could be wrong about is abandoned, as one
/// the ray cannot honestly place, and the next is tried. Nothing here is a
/// tolerance: both sides are signs, and a candidate is dropped only when two
/// signs that were both decided say opposite things. A sign that could not be
/// decided, or a point on the plane, is left to the ray's own answer, which
/// already says `OnBoundary` or `Undecidable` wherever it cannot tell.
fn the_ray_and_the_stored_planes_disagree_about(
    solid: SolidView<'_>,
    point: PredicatePoint,
) -> bool {
    solid.faces().any(|face| {
        let Some(plane) = face.surface().as_plane().copied() else {
            return false;
        };
        let every_side_is_straight = face.wires().all(|wire| {
            wire.coedges()
                .all(|coedge| coedge.edge().stored_run() == &EdgeRun::Straight)
        });
        if !every_side_is_straight {
            return false;
        }
        let Some(off_the_plane) = InPlane::of(&plane) else {
            return false;
        };
        let on_the_side_of_the_reference = match plane.side_of_point(point) {
            Decision::Pos => off_the_plane.turned_round,
            Decision::Neg => !off_the_plane.turned_round,
            Decision::Zero | Decision::Undecidable { .. } => return false,
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        };
        let mut disagree = false;
        face.for_each_triangle(|corners| {
            let [a, b, c] = corners.map(|corner| PredicatePoint::from(corner.coordinates()));
            let reference = orient3d_of_points(a, b, c, off_the_plane.from);
            let this_one = orient3d_of_points(a, b, c, point);
            let decided = |turn: &Decision| matches!(turn, Decision::Pos | Decision::Neg);
            if decided(&reference)
                && decided(&this_one)
                && (reference == this_one) != on_the_side_of_the_reference
            {
                disagree = true;
            }
        });
        disagree
    })
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
/// A flat face with straight sides is wound, the way it always was. A **curved**
/// one is [`SolidView::face_holds_the_point`], which is the one authority on that
/// question (§35) and which the fuse can call here because what it is asked
/// about is a face of an *input* — a face the model holds, with corners it holds
/// — rather than a piece the arrangement solved for.
///
/// **So is a flat face with a curved side.** A cone's cap is a disk bounded by
/// one circle with one corner on it, and the polygon of its corners is that one
/// point, which winds round nothing: every point of the cap read as off it. So a
/// cap piece lying on a copy of the same cap was never on anything, and a cone
/// united or shared with a copy of itself refused on
/// [`PLACED_AGAINST_EVERY_SOLID`] (`docs/decisions.md` §222.3,
/// `tests/a_cone_and_its_copy.rs`). A ball has no flat face, which is why a ball
/// and its copy answered. Only [`OnAFace::Held`](crate::OnAFace::Held) counts
/// here, as a zero winding does not count above: a point on the face's rim says
/// nothing about the piece around it.
fn a_face_of_the_input_holds(
    model: &Model,
    solid: &BakedSolid,
    face: &BakedFace,
    piece: &Piece,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    let view = model.solid_at(solid.key.slot_in(model.id()));
    if let Some(flat) = piece.surface.as_plane() {
        if face
            .wires
            .iter()
            .flatten()
            .any(|side| side.across.as_plane().is_none())
        {
            for &key in &face.keys {
                if view.face_holds_the_point(key, point, policy, log)? == crate::OnAFace::Held {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
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
    /// 37.9% — 72 fuses that [`place_the_piece`] gave up on. That 72 was counted
    /// when one name covered both of its searches, so **how it splits between
    /// [`INSIDE_A_PIECE`] and [`PLACED_AGAINST_EVERY_SOLID`] has not been
    /// measured**; the total and the two rates still stand, the breakdown has to
    /// be re-run before anyone quotes one.
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
                candidates.extend(the_middles_of_a_round_wire(flat, &partner));
            }
        }
        for side in piece.wires.iter().flatten() {
            candidates.extend(half_way_into_either_arc(flat, side));
        }
        for side in piece.wires.iter().flatten() {
            candidates.extend(half_way_across_the_chord_to_a_cone(flat, side));
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
    if let Some(towards) = towards_every_flat_sides_material(piece) {
        candidates.push(PredicatePoint::from(
            (middle + towards * reach).coordinates(),
        ));
    }
    candidates
        .into_iter()
        .filter_map(|candidate| back_onto_the_surface(&piece.surface, candidate))
        .collect()
}

/// Which way from a curved piece's corners its material lies, as far as the
/// planes that cut it say: the sum of the way into the material across each of
/// them, each plane counted once. `None` when no plane cut it, or when the ways
/// cancel.
///
/// **A piece whose corners all lie on one plane is what needs it.** A half
/// ball's skin cut by a plane that just crosses its flat face has every corner
/// on that face — the two poles and the two places the cut crosses it — so
/// every blend of them lands back on the face, where the face's own plane
/// cannot say which side it is on, and of the six steps from their middle the
/// ones square to the face go into the cap the cut took off or out of the half
/// ball. What is kept is the part of the skin inside the half ball *and* on
/// the kept side of the cut, and the way to it is the two ways added up: one
/// step each, not the half ball's once for every side it bounds, or the
/// face's three sides outvote the cut's one and the step goes into the cap
/// again.
fn towards_every_flat_sides_material(piece: &Piece) -> Option<Vector> {
    let mut planes: Vec<(Plane, Decision)> = Vec::new();
    for side in piece.wires.iter().flatten() {
        if let Some(flat) = side.carried_by[1].as_plane() {
            let this = (*flat, side.material_side);
            if !planes.contains(&this) {
                planes.push(this);
            }
        }
    }
    let mut towards = Vector::zero();
    for (flat, material_side) in &planes {
        let into = if *material_side == Decision::Neg {
            -1.0
        } else {
            1.0
        };
        towards = towards + flat.normal() * into;
    }
    towards.normalized().ok()
}

/// Two points a quarter of the way across the circle a curved side of a flat
/// piece runs round, one either side of the chord between its ends: half way
/// from the middle of the chord to where the circle crosses the line square to
/// it. One of them is between the side and its chord, which is where a half
/// disk has all of its inside and its corners none (see [`in_the_bulge_of`]).
///
/// Which one is not asked, because nothing about a candidate is trusted: each
/// is placed and proved by the caller. The circle's centre is one of
/// [`the_middles_of_a_round_wire`], each tried, and its radius comes from the
/// right angle at the chord's middle, so for an ellipse the points are near
/// rather than on that line, which is all a candidate needs. Nothing for a
/// straight side, and for a side with nothing to go round or no length to go
/// square to.
fn half_way_into_either_arc(flat: &Plane, side: &FaceMeet) -> Vec<PredicatePoint> {
    let [start, end] = [0, 1].map(|which| middle_of(side.ends[which].point()));
    let chord = end - start;
    let Ok(across) = flat.normal().cross(chord).normalized() else {
        return Vec::new();
    };
    let middle = start + chord * 0.5;
    let mut candidates = Vec::new();
    for centre in the_middles_of_a_round_wire(flat, &side.carried_by[1]) {
        let off = across.dot(middle_of(centre) - middle);
        let reach = (off * off + chord.length_squared() / 4.0).sqrt();
        let points = [off + reach, off - reach].map(|to| middle + across * (to / 2.0));
        if points.iter().all(Point::is_finite) {
            candidates.extend(points.map(|point| PredicatePoint::from(point.coordinates())));
        }
    }
    candidates
}

/// Points half way from the middle of a cone side's chord to where the line
/// square to that chord, in the plane, crosses the cone.
///
/// **A hyperbola has no middle to build round.** [`half_way_into_either_arc`]
/// aims at where the cone's axis crosses the plane, and a plane parallel to the
/// axis has no such point, while for one leaning less than that the point can be
/// in the other nappe's region. So a flat piece bounded by a chord and an arc of
/// a hyperbola — a cube's wall cutting through a cone the way `x = 3.5` cuts a
/// cone about `x = 3` — had no candidate off its chord at all.
///
/// The line through the chord's middle square to it meets the cone where
/// `(1 + s²)(v·w)² − |w|²|v|² = 0` for `v = middle + t·across − apex`, which is a
/// quadratic in `t`. Half way to each root is a candidate, and one of them is
/// between the arc and its chord whenever the arc is part of the boundary of a
/// convex region that holds the chord. Nothing is trusted: the caller proves
/// each candidate is inside or drops it.
fn half_way_across_the_chord_to_a_cone(flat: &Plane, side: &FaceMeet) -> Vec<PredicatePoint> {
    let Surface::Cone(cone) = side.carried_by[1] else {
        return Vec::new();
    };
    let [start, end] = [0, 1].map(|which| middle_of(side.ends[which].point()));
    let chord = end - start;
    let Ok(across) = flat.normal().cross(chord).normalized() else {
        return Vec::new();
    };
    let middle = start + chord * 0.5;
    let up = cone.axis().direction();
    let widening = 1.0 + cone.slope() * cone.slope();
    let from_apex = middle - cone.apex();
    let (along_now, along_per_step) = (from_apex.dot(up), across.dot(up));
    let square = widening * along_per_step * along_per_step - up.length_squared();
    let linear =
        2.0 * (widening * along_now * along_per_step - up.length_squared() * from_apex.dot(across));
    let constant =
        widening * along_now * along_now - up.length_squared() * from_apex.length_squared();
    let steps: Vec<f64> = if square == 0.0 {
        vec![-constant / linear]
    } else {
        let discriminant = linear * linear - 4.0 * square * constant;
        if discriminant < 0.0 {
            return Vec::new();
        }
        let root = discriminant.sqrt();
        vec![
            (-linear + root) / (2.0 * square),
            (-linear - root) / (2.0 * square),
        ]
    };
    steps
        .into_iter()
        .map(|step| middle + across * (step / 2.0))
        .filter(Point::is_finite)
        .map(|point| PredicatePoint::from(point.coordinates()))
        .collect()
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

/// Where the ball, pipe, cone or torus that cut a round wire has its middle, on
/// the plane that wire is drawn on.
///
/// The centre of a circle cut on a plane, without ever saying the word: for a
/// pipe it is where the axis meets the plane, and for a ball it is the foot of
/// the perpendicular from the centre. Both are at distance nothing from the
/// axis or centre, so both are provably inside — which is what the winding
/// count then checks, in exact arithmetic and with no trust placed here.
///
/// **A torus has two answers, and both are offered.** A plane cuts a torus in
/// up to four circles, and where it goes through the torus's axis it cuts two,
/// one round each place the ring crosses the plane. Which of the two a given
/// wire goes round is not known here, and does not need to be: nothing about a
/// candidate is trusted, so the one that is not inside is ruled out by the same
/// count that proves the other in.
///
/// Nothing where the division has nothing to divide by, which is a pipe running
/// along the plane rather than through it — and then the wire it cut is not a
/// closed curve and this was not asked — or a ring that never reaches the plane.
fn the_middles_of_a_round_wire(flat: &Plane, partner: &Surface) -> Vec<PredicatePoint> {
    let (normal, offset) = (flat.normal(), flat.offset());
    let froms = match partner {
        Surface::Sphere(ball) => vec![ball.centre()],
        Surface::Cylinder(pipe) => {
            let axis = pipe.axis();
            let along = normal.dot(axis.direction());
            if along == 0.0 {
                return Vec::new();
            }
            vec![
                axis.origin()
                    + axis.direction()
                        * ((offset - normal.dot(axis.origin() - Point::origin())) / along),
            ]
        }
        Surface::Torus(ring) => where_the_ring_crosses_the_plane(normal, offset, ring),
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
                return Vec::new();
            }
            vec![
                axis.origin()
                    + axis.direction()
                        * ((offset - normal.dot(axis.origin() - Point::origin())) / along),
            ]
        }
        Surface::Plane(_) | _ => return Vec::new(),
    };
    froms
        .into_iter()
        .map(|from| {
            from + normal * ((offset - normal.dot(from - Point::origin())) / normal.dot(normal))
        })
        .filter(Point::is_finite)
        .map(|there| PredicatePoint::from(there.coordinates()))
        .collect()
}

/// The points where a torus's ring — the circle down the middle of its tube —
/// crosses the plane `normal · p = offset`: none, one or two.
///
/// The ring is `centre + ring_radius · w` for every unit `w` square to the
/// axis, so the crossings are the `w` whose part along the plane's normal is
/// `(offset − normal · centre) / ring_radius`. Only the part of the normal
/// square to the axis can see `w`; a plane square to the axis has none, and
/// its ring either misses the plane or lies in it, so it has no crossing to
/// name.
fn where_the_ring_crosses_the_plane(normal: Vector, offset: f64, ring: &Torus) -> Vec<Point> {
    let axis = ring.axis().direction();
    let across = normal - axis * normal.dot(axis);
    let how_much = across.length_squared().sqrt();
    let Ok(towards) = across.normalized() else {
        return Vec::new();
    };
    let Ok(sideways) = axis.cross(across).normalized() else {
        return Vec::new();
    };
    let reach =
        (offset - normal.dot(ring.centre() - Point::origin())) / (ring.ring_radius() * how_much);
    if reach.is_nan() || reach.abs() > 1.0 {
        return Vec::new();
    }
    let aside = (1.0 - reach * reach).sqrt();
    [aside, -aside]
        .map(|way| ring.centre() + (towards * reach + sideways * way) * ring.ring_radius())
        .to_vec()
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
///
/// It means what it says and nothing more: not one of the candidates could be
/// proved to be inside the piece. It is **not** the name for a point that was
/// inside and could not then be classified — that is
/// [`PLACED_AGAINST_EVERY_SOLID`], and the two were one string until §216.4
/// caught it naming the search that had worked.
const INSIDE_A_PIECE: &str = "a point inside a piece of a face";

/// The name the search for a point the solids can all classify goes by.
///
/// A point *was* found inside the piece; what ran out is somewhere to stand
/// where every solid in the set could say inside, outside or on.
const PLACED_AGAINST_EVERY_SOLID: &str = "a point in a piece that every solid could place";

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

#[cfg(test)]
mod tests {
    use rcad_geom::{Axis, Orientation, Plane, Surface, Torus};
    use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
    use smallvec::SmallVec;

    use super::{
        InPlane, Piece, candidate_points, corners_of, every_picker_keeps,
        the_middles_of_a_round_wire, the_pickers_of_the_meridian_circle_through, the_piece_holds,
        what_else_each_round_wire_asks,
    };
    use crate::fuse::{FaceMeet, MeetPoint};

    /// The torus `a_quarter_bend(4, 1)` is drawn on, and the plane `y = 0` its
    /// end cap lies on.
    fn a_bend_and_its_end_cap_plane() -> (Surface, Plane) {
        let ring = Torus::new(Axis::z(), 4.0, 1.0).expect("a ring wider than its tube");
        let flat = Plane::from_coefficients([0.0, 1.0, 0.0, 0.0]).expect("a normal");
        (Surface::Torus(ring), flat)
    }

    #[test]
    fn a_plane_through_a_bends_axis_offers_both_places_the_ring_crosses_it() {
        let (ring, flat) = a_bend_and_its_end_cap_plane();
        let mut middles: Vec<[f64; 3]> = the_middles_of_a_round_wire(&flat, &ring)
            .iter()
            .map(|middle| middle.enclosure().map(|axis| axis.lo()))
            .collect();
        middles.sort_by(|one, other| one[0].total_cmp(&other[0]));
        assert_eq!(middles, [[-4.0, 0.0, 0.0], [4.0, 0.0, 0.0]]);
    }

    /// The torus's inside on `y = 0` is two disks, and the sign of the torus
    /// holds both ring crossings. The half-space the circle through the cap's
    /// corner is named by holds only the one on the cap's side.
    #[test]
    fn a_bends_end_cap_holds_the_ring_crossing_on_its_own_side_and_not_the_other() {
        let (ring, flat) = a_bend_and_its_end_cap_plane();
        let policy = Policy::certified();
        for (corner, own, other) in [([3.0, 0.0, 0.0], 4.0, -4.0), ([-3.0, 0.0, 0.0], -4.0, 4.0)] {
            let pickers = the_pickers_of_the_meridian_circle_through(
                &Surface::Plane(flat),
                &ring,
                PredicatePoint::from(corner),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a plane through a torus's axis crosses it")
            .expect("one of the two circles keeps the corner");
            let keeps = |x: f64| every_picker_keeps(&pickers, PredicatePoint::from([x, 0.0, 0.0]));
            assert_eq!(keeps(own), Some(true), "the circle through {corner:?}");
            assert_eq!(keeps(other), Some(false), "the circle through {corner:?}");
        }
    }

    /// A plane square to the axis cuts two concentric circles, and neither's
    /// disk is the tube's inside on one side of the pipe that names it.
    #[test]
    fn a_plane_square_to_a_bends_axis_names_no_meridian_circle() {
        let (ring, _) = a_bend_and_its_end_cap_plane();
        let across = Plane::from_coefficients([0.0, 0.0, 1.0, 0.5]).expect("a normal");
        let pickers = the_pickers_of_the_meridian_circle_through(
            &Surface::Plane(across),
            &ring,
            PredicatePoint::from([4.0 + 0.75_f64.sqrt(), 0.0, 0.5]),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a plane square to a torus's axis crosses it");
        assert_eq!(pickers, None);
    }

    /// The torus's sign is inside at both disks on `y = 0`, so without the
    /// half-space the circle through the cap's corner is named by, the cap
    /// holds the other disk too.
    #[test]
    fn a_bends_end_cap_holds_the_disk_its_own_corner_is_in_and_not_the_other() {
        let (ring, flat) = a_bend_and_its_end_cap_plane();
        let cap = a_flat_piece_inside_the_ring(ring, flat, &[[3.0, 0.0, 0.0]]);
        let holds = |x: f64| {
            the_piece_holds_asked_as_selection_asks(&cap, PredicatePoint::from([x, 0.0, 0.0]))
        };
        assert_eq!(
            holds(4.0),
            Some(true),
            "the disk round (4, 0, 0) is the cap"
        );
        assert_eq!(holds(-4.0), Some(false), "the disk round (-4, 0, 0) is not");
    }

    /// Of the points offered for a bend's end cap, the first one it holds is in
    /// its own disk. The ring's crossing at `(−4, 0, 0)` is offered before the
    /// one at `(4, 0, 0)`, so a cap that held both would be placed from the
    /// other disk.
    #[test]
    fn the_first_point_a_bends_end_cap_offers_and_holds_is_in_its_own_disk() {
        let (ring, flat) = a_bend_and_its_end_cap_plane();
        let cap = a_flat_piece_inside_the_ring(ring, flat, &[[3.0, 0.0, 0.0]]);
        let first_held = candidate_points(&cap, &corners_of(&cap))
            .into_iter()
            .find(|&candidate| {
                the_piece_holds_asked_as_selection_asks(&cap, candidate) == Some(true)
            })
            .expect("the cap holds one of the points offered for it");
        assert_eq!(first_held, PredicatePoint::from([4.0, 0.0, 0.0]));
    }

    /// A plane square to the axis cuts the tube in a ring-shaped piece, and
    /// what the torus's sign holds on it is not the piece on either side of a
    /// pipe, so nothing here can say.
    #[test]
    fn a_piece_on_a_plane_square_to_a_bends_axis_cannot_say_what_it_holds() {
        let (ring, _) = a_bend_and_its_end_cap_plane();
        let across = Plane::from_coefficients([0.0, 0.0, 1.0, 0.5]).expect("a normal");
        let reach = 0.75_f64.sqrt();
        let piece = a_flat_piece_inside_the_ring(
            ring,
            across,
            &[[4.0 + reach, 0.0, 0.5], [4.0 - reach, 0.0, 0.5]],
        );
        assert_eq!(
            the_piece_holds_asked_as_selection_asks(&piece, PredicatePoint::from([4.0, 0.0, 0.5])),
            None
        );
    }

    /// [`the_piece_holds`] with the pickers worked out the way
    /// `place_the_piece` works them out.
    fn the_piece_holds_asked_as_selection_asks(
        piece: &Piece,
        point: PredicatePoint,
    ) -> Option<bool> {
        let flat = piece.surface.as_plane().expect("a flat piece");
        let in_plane = InPlane::of(flat).expect("a plane with a point off it");
        let also_picked_by =
            what_else_each_round_wire_asks(piece, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a plane crosses a torus in circles");
        the_piece_holds(piece, Some(&in_plane), &also_picked_by, point)
    }

    /// A piece of `flat` bounded by whole circles it cuts `ring` in, one wire
    /// through each corner, the first the outer one, with its material inside
    /// the tube.
    ///
    /// It names no ancestor: the two functions it is handed to read the
    /// surface and the wires and nothing else.
    fn a_flat_piece_inside_the_ring(ring: Surface, flat: Plane, corners: &[[f64; 3]]) -> Piece {
        let carried_by = [Surface::Plane(flat), ring];
        let wires = corners
            .iter()
            .map(|&corner| {
                let starts = MeetPoint::WhereAClosedCurveStarts {
                    carried_by,
                    point: PredicatePoint::from(corner),
                };
                vec![FaceMeet {
                    carried_by,
                    ends: [starts.clone(), starts],
                    material_side: Decision::Neg,
                    along: SmallVec::new(),
                    approximation: None,
                }]
            })
            .collect();
        Piece {
            ancestors: Vec::new(),
            surface: Surface::Plane(flat),
            orientation: Orientation::Forward,
            wires,
            is_the_whole_surface: false,
        }
    }
}
