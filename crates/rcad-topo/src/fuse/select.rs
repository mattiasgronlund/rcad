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

use rcad_geom::{Plane, PlaneAgreement};
use rcad_predicates::{
    Decision, Interval, Measures, Point as PredicatePoint, Policy, ProvenanceLog,
    orient3d_of_points,
};

use crate::key::FaceSlot;
use crate::{Containment, Error, Model, Result, SolidView};

use super::arrange::{Piece, agreement_of_two_planes};
use super::bake::{BakedFace, BakedSolid};
use super::{Scratch, as_exact_as_it_is, gave_up, the_other_way};

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
        let facing = outward_plane_of(piece);
        let corners = corners_of(piece);
        let Some(in_plane) = InPlane::of(&piece.plane()) else {
            return Err(nothing_to_measure_from(piece, log));
        };
        let candidates = candidate_points(&corners);

        let own_group: Vec<usize> = (0..solids.len())
            .filter(|&other| other != mine && (other < arguments) == (mine < arguments))
            .collect();
        let against_own = place_the_piece(
            model,
            &solids,
            &own_group,
            &candidates,
            &corners,
            &in_plane,
            &facing,
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
            &corners,
            &in_plane,
            &facing,
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
    corners: &[Vec<PredicatePoint>],
    in_plane: &InPlane,
    facing: &Plane,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Placement> {
    if against.is_empty() {
        return Ok(Placement::Outside);
    }
    for &candidate in candidates {
        if in_plane.winding_around(corners, candidate) != Some(1) {
            continue;
        }
        let mut trial = ProvenanceLog::new();
        let settled = settle_against_each(
            model, solids, against, candidate, in_plane, facing, policy, &mut trial,
        )?;
        if let Some(placement) = settled {
            trial.drain_into(log);
            return Ok(placement);
        }
    }
    Err(gave_up(
        INSIDE_A_PIECE,
        &corners.concat(),
        Measures::Length,
        log,
    ))
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
    in_plane: &InPlane,
    facing: &Plane,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<Placement>> {
    let mut on = None;
    for &index in against {
        let solid: SolidView<'_> = model.solid(solids[index].key);
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
            Containment::CannotFollowAFacesTrimming { .. } => return Ok(None),
            Containment::OnBoundary | Containment::Undecidable { .. } => {
                let Some(face) = coincident_face(solids[index], in_plane, facing, point, log)?
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
    solid: &BakedSolid,
    in_plane: &InPlane,
    facing: &Plane,
    point: PredicatePoint,
    log: &mut ProvenanceLog,
) -> Result<Option<bool>> {
    for face in &solid.faces {
        let agreement = agreement_of_two_planes(facing, &outward_plane_of_face(face), log)?;
        let same_way = match agreement {
            PlaneAgreement::SameWay => true,
            PlaneAgreement::OppositeWay => false,
            PlaneAgreement::Different => continue,
            other => {
                panic!("plane agreement came back with an answer nothing here knows: {other:?}")
            }
        };
        let corners: Vec<Vec<PredicatePoint>> = face
            .wires
            .iter()
            .map(|wire| wire.iter().map(|edge| edge.start.point).collect())
            .collect();
        // Either sign means the point is in this face's material. Which one
        // comes out says only which way that face's outline runs round *this*
        // piece's normal, and a face pressed against this one from the other
        // side runs the other way — which is what `OppositeWay` above already
        // said. Zero is the only answer that puts the point outside it.
        if in_plane
            .winding_around(&corners, point)
            .is_some_and(|winding| winding != 0)
        {
            return Ok(Some(same_way));
        }
    }
    Ok(None)
}

/// The plane a piece's material is *behind*: the piece's own plane, turned round
/// when its solid uses the face the other way.
///
/// Everything the ON columns need is here. Two coincident faces face the same
/// way exactly when these two planes do, and comparing the stored planes alone
/// would call a face and the face pressed against it from the other side the
/// same thing.
fn outward_plane_of(piece: &Piece) -> Plane {
    if piece.orientation.is_reversed() {
        piece.plane().reversed()
    } else {
        piece.plane()
    }
}

/// The plane of an input face, turned to point out of the material.
///
/// # Panics
///
/// If the face is curved. Selection is still gated on
/// [`Error::FuseNeedsFlatFaces`](crate::Error::FuseNeedsFlatFaces), which lane 5
/// takes off; until then a curved face never reaches here.
fn outward_plane_of_face(face: &BakedFace) -> Plane {
    let plane = *face
        .surface
        .as_plane()
        .expect("selection is still gated on every face being flat");
    if face.orientation.is_reversed() {
        plane.reversed()
    } else {
        plane
    }
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
                _ => (),
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
                _ => return None,
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
    fn turn(
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
/// Two families, and between them they cover any polygon that is not a straight
/// line. A corner and the middle of the side facing it is inside every triangle
/// and every convex corner, which is most pieces on the first try. A diagonal's
/// middle is what is left for a piece with a dent in it, and a polygon with more
/// than three corners always has a diagonal.
fn candidate_points(wires: &[Vec<PredicatePoint>]) -> Vec<PredicatePoint> {
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
    candidates
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
