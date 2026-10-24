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
//! **A point placed with a decision the policy would refuse the fuse for is
//! held back while the next one is tried.** A ray thrown away where it grazed
//! a face can leave what it could not decide in the point's log, though the
//! next ray placed the point. The first point that places the piece with
//! nothing the policy refuses places it, and only where none does is the point
//! held back taken, and the fuse refused for it as before (§35.6, Q2267).
//!
//! The one predicate this needs is which way three points on the face turn about
//! its normal, and that is [`orient3d_of_points`] against a fourth point off the
//! plane whose side has been proved — see [`InPlane`]. No new predicate, and
//! nothing is projected into two dimensions, which is §15.5's rule.
//!
//! # A piece no point places is placed by a side it shares
//!
//! **Only where the search above placed no point inside the piece**, whether
//! it found none or found some and no ray placed them. A fuse that placed a
//! point in every piece never gets here, and answers as it did.
//! A piece can be narrower than the spacing of `f64`, and then no point it
//! could be handed is inside it (§346.3). Or every point found inside it can
//! be one no ray places. Q1064 brought the first such piece here, half of a
//! bar's wall lying on a pipe that a wall of the other solid lies on too, and
//! holds none of the half; since Q1080 the ray places that piece itself, as
//! `classify.rs` no longer grazes on a face that holds none of the point. The
//! rule is still reached where a point was found, confirmed with a temporary
//! print in `choose` (Q1079): a turned pair
//! (`swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit`,
//! `how_often_a_certified_fuse_of_a_turned_pair_refuses`), a solid with a
//! torus face (`how_often_a_certified_fuse_with_a_torus_in_it_refuses`), the
//! fuzz corpus (`every_seed_in_the_checked_in_corpus_is_fused_without_panicking`)
//! and a fused draw (`a_fused_draw_hands_its_provenance_back_through_the_door`)
//! each reach it. Such a piece is placed against a solid by a side it shares with a
//! piece cut from one of that solid's faces, from which side of that face its
//! own material is on ([`SidesItShares`], §365.3).
//!
//! **And where no side it shares places it either, by a corner** that the
//! other solid's ray puts strictly inside or strictly outside it
//! ([`by_a_corner`], §365.7): every open neighbourhood of a corner meets the
//! piece's inside, and that inside is all on one side of the solid.
//!
//! # Or a flat piece outside a flat solid by a plane
//!
//! A flat piece with straight sides is placed outside a solid with only flat
//! faces when the plane of one of that solid's faces has the whole solid on or
//! behind it, every corner of the piece on or in front of it and one strictly
//! in front, each corner asked as the meet of three planes, exactly
//! ([`by_a_plane`], Q991, Q1288). Two turned cubes whose facing faces lie a
//! rounding apart have no point inside either face that a ray can place, and
//! nor does the triangle a rounding across that a turned cube's corner, just
//! inside a block's face, cuts out of it; this is what answers both.
//!
//! **The plane comes last where no point was found inside the piece, and first
//! where one was found and none placed.** That is the order each rule came in:
//! the sides and the corners were tried only where no point was found, or only
//! a rung of [`rungs_towards_each_crossing`] was (§365.6), until Q1064 tried
//! them after the plane where a point was found as well. So every piece one of
//! them placed before is placed by the same rule now.
//!
//! # Then by the piece of its own face across a side
//!
//! A flat piece none of those placed, against solids with only flat faces,
//! takes the placement of the piece of its own face across one of its straight
//! sides, where no plane of those solids but the face's own holds that side
//! ([`by_the_piece_across_a_side`], §365.9). Near most of such a side the
//! solids' skin is that plane or nothing, so the two pieces sit alike. It is
//! tried after the sides, the corners and the plane, so a piece any of them
//! placed is placed as before.
//!
//! # Then by an edge of a face it lies on
//!
//! A flat piece that lies on the plane of a flat face `G` of a solid, with a
//! straight side along an edge `e` of `G`, both ends exactly on `e`, is placed
//! against that solid by which side of the plane across `e` its material is on
//! ([`by_an_edge_of_a_face_it_lies_on`], §365.8, Q1606). Near the middle of
//! that side the solid's skin is `G` and the face across `e` and nothing else:
//! on `G`'s side the piece lies on `G`, and on the other it is inside or
//! outside by the outward plane across `e`. This is the one side the piece
//! across passes over, since the plane across `e` holds both its ends. It is
//! tried after every rule above, so a piece any of them placed is placed as
//! before.
//!
//! # Then by a side that starts inside an edge
//!
//! A flat piece with a straight side along the plane of a flat face `G` of a
//! solid, starting strictly inside an edge `e` of `G`, is placed against that
//! solid from where that side runs ([`by_a_side_from_an_edge_it_starts_on`],
//! §365.8, Q1608, Q1609). Near the start the solid's skin is `G` and the face
//! across `e`: where the side runs along `G`, the piece is inside when its
//! material is on `G`'s inner side, and where it runs on beyond `e`, off the
//! skin, the piece is inside when the side runs behind the outward plane
//! across `e`. It is tried after every rule above, so a piece any of them
//! placed is placed as before.
//!
//! # Then on a face it shares a side with
//!
//! A piece, flat or curved, that shares a side with a piece cut from a face
//! `G` of a solid on its own surface, with the same partner surface and its
//! material on the same side of it, lies on `G`
//! ([`SidesItShares::place_on_a_face_it_shares_a_side_with`], §35.6, Q1537):
//! near the middle of that side the two pieces are one part of one surface.
//! Two quarter bends placed a quarter turn apart by a float turn need it:
//! each bend's cap plane cuts the other bend's wall a rounding round from that
//! bend's own cap rim, and no `f64` point lies in the band between. It is
//! tried after every rule above, so a piece any of them placed is placed as
//! before.
//!
//! # Then by the piece across a side that starts clear of every face
//!
//! A piece, flat or curved, takes the placement of the piece of its own face
//! across one of its sides, when the point search places that piece inside
//! or outside and the start of the side is off every face of every solid it is
//! placed against ([`by_the_piece_across_a_side_clear_of_every_face`], §35.6,
//! Q1537): near that start the two pieces are on one side of the skin. A start
//! on the plane and the ring of a cap of a bend is shown off the cap by the
//! half-space that names the cap's circle ([`is_behind_the_meridian_disk`]).
//! Two quarter bends placed half a turn apart need it. It is tried after
//! every rule above, so a piece any of them placed is placed as before.
//!
//! # Then by a meridian disk beside it
//!
//! A flat piece inside a ring, on a plane through the ring's axis, is placed
//! against a solid made only of that ring and of planes through its axis by a
//! face of that solid that is a whole meridian disk: when no other plane of
//! the solid parts the piece's half of its plane from the disk's, the piece is
//! outside when it is in front of the disk's outward plane and inside when it
//! is behind, each side read off the planes' own numbers
//! ([`by_a_meridian_disk_beside_it`], §35.6, Q2056). Two quarter bends placed
//! round by `0.3` and `0.3 + π/2` need it: the first bend's end cap lies
//! strictly in front of the copy's start cap plane, an ulp off it, so neither
//! plane cuts the other bend's wall and the cap shares no side with anything.
//! It is tried after every rule above, so a piece any of them placed is placed
//! as before.
//!
//! # And last, by a step towards a corner of the box round it
//!
//! A curved piece is placed by a point one reach from the middle of its
//! corners towards each corner of the box round that middle, put back onto its
//! surface ([`by_a_step_towards_a_corner_of_the_box_round_it`], §35.6, Q2116).
//! The six steps the point search takes from there go along the axes, and a
//! ball inscribed in a cube touches the cube at all six: no point offered on
//! the ball's skin was off the cube's. It is tried after every other rule, so
//! a piece any of them placed is placed as before.
//!
//! # Then a piece of a flat shell both groups hold lies on the other group
//!
//! Once every piece is placed, under a tolerant policy, [`choose`] puts
//! together what the intersection would keep and asks of each of its shells
//! whether the policy takes it as flat, as the shell fold of `assemble.rs`
//! does ([`the_pieces_of_each_flat_shell_of_both_groups`], Q1710, §417).
//! Each piece of such a shell, where both groups have pieces, is read as
//! lying on the other group facing the other way: a shell with no inside is
//! two sides of one sheet, back to back. So the intersection keeps none of it,
//! and a difference keeps the argument's pieces and drops the tool's. Two
//! skewed combs touching along one line need it: the comb less the other kept
//! the prism of `3.1e−32` their rounded planes close as a notch in its own
//! shell, which the shell fold cannot reach, and the notch's floor rounded to
//! a triangle with no winding. A piece of the shell lying on the other group
//! facing the same way is read so too (Q2253): the rim of a slab where two
//! walls stand flush, which a difference keeps when it is the argument's, as
//! it keeps the rest of the face it is a strip of. A union drops a buried
//! piece and one lying the other way alike, and keeps that rim as part of its
//! outside, so it does not ask this; nor does a certified policy, and a piece
//! of no such shell is placed as above.
//!
//! # Or, for a union, a piece of a flat shell one group less the other leaves
//!
//! A union of two groups is the second group with the first less the second
//! added on, so where the first less the second is a shell the policy takes
//! as flat, the union there is the second group's own outside. So [`choose`]
//! puts together what each group less the other would keep, and reads each
//! piece of such a shell, where both groups have pieces, as that difference
//! would leave it out: a piece of the group the other is taken from as inside
//! the other, and a piece of the other as outside the first. A turned ell
//! inside a taller one, flush against its wall, needs it (Q2270): as stored,
//! the small ell's wall is `8.3e−17` outside the big one's, so the small ell
//! less the big one is a slab that thin, and the union kept the small ell's
//! wall pieces on it and dropped the big ell's under it, which the joining of
//! the two walls then could not tell an outline of. Read this way, the union
//! keeps the big ell's wall whole and none of the slab.

use rcad_geom::{
    Cone, Cylinder, Picker, Plane, PlaneAgreement, Point, Sphere, Surface, SurfaceMeet, Torus,
    Vector, intersect,
};
use rcad_predicates::{
    Decision, Interval, Measures, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface,
    orient3d_of_points, side_of_a_plane_where_three_planes_meet,
    which_meet_comes_first_along_a_line,
};

use crate::classify::{every_side_of_it_is_straight, side_of_a_flat_face, stretched_to_the_plane};
use crate::holds::{
    OnAFace, cross, dot, exactly, how_the_axis_lies_on, lie_along_one_line,
    the_half_space_naming_the_meridian_circle_through, the_meridian_half_space_keeps,
};
use crate::validate::side_of_surface;
use crate::{Containment, Error, FaceKey, FaceView, Model, Result, SolidView, VertexKey};

use super::arrange::{
    Piece, a_plane_rests_along_a_pipe, agreement_of_two_planes, name_of_surface_bits,
};
use super::assemble::{
    NamedSide, the_edge_each_side_of_every_piece_is, the_pieces_of_each_flat_shell_of_both_groups,
};
use super::bake::{BakedCorner, BakedEdge, BakedFace, BakedSolid};
use super::{
    FaceMeet, MeetPoint, Scratch, a_fuse_logging_this_is_refused, as_exact_as_it_is, gave_up,
    middle_of, the_other_way,
};

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
/// [`Error::Undecidable`] when a piece could not be placed against the solids
/// it has to be placed against, and neither a side it shares, nor a corner, nor
/// a plane ([`by_a_plane`]), nor the piece of its own face across a side
/// ([`by_the_piece_across_a_side`]), nor an edge of a face it lies on
/// ([`by_an_edge_of_a_face_it_lies_on`]), nor a side that starts inside an edge
/// ([`by_a_side_from_an_edge_it_starts_on`]), nor a face it shares a side with
/// ([`SidesItShares::place_on_a_face_it_shares_a_side_with`]), nor the piece
/// across a side that starts clear of every face
/// ([`by_the_piece_across_a_side_clear_of_every_face`]), nor a meridian disk
/// beside it ([`by_a_meridian_disk_beside_it`]), nor a step towards a corner of
/// the box round it ([`by_a_step_towards_a_corner_of_the_box_round_it`])
/// placed it: under
/// [`INSIDE_A_PIECE`] when no point inside it was found, and under
/// [`PLACED_AGAINST_EVERY_SOLID`] when a point was found and could not be
/// placed, or only a rung of [`rungs_towards_each_crossing`],
/// [`shorter_steps_across_a_curved_piece`],
/// [`between_the_middles_of_a_cone_pieces_sides`],
/// [`steps_from_a_reach_into_the_planes_material`] or
/// [`half_way_round_the_tube_between_two_corners`] was found.
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] instead of the second, when
/// a point offered before the rungs was found inside the piece and the ray
/// from a point tried met a face of a solid that nothing here can follow; none
/// of those rules is tried then. The same refusal, naming the same kind of
/// face, when only a rung was found inside and a rung's ray met such a face,
/// once none of those rules has placed the piece.
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

    let mut by_a_shared_side = SidesItShares::of(model, scratch, &solids);
    let mut placed = Vec::with_capacity(scratch.pieces.len());
    for (index, (piece, &mine)) in scratch.pieces.iter().zip(&scratch.cut_from).enumerate() {
        let corners = corners_of(piece);
        let in_plane = match piece.surface.as_plane() {
            Some(flat) => {
                Some(InPlane::of(flat).ok_or_else(|| nothing_to_measure_from(piece, log))?)
            }
            None => None,
        };
        let candidates = candidate_points(piece, &corners);
        let mut place = |against: &[usize], log: &mut ProvenanceLog| -> Result<Placement> {
            let (unplaced, the_plane_first, unfollowed) = match place_the_piece(
                model,
                &solids,
                against,
                &candidates,
                piece,
                in_plane.as_ref(),
                policy,
                log,
            )? {
                Search::Placed(placement) => return Ok(placement),
                Search::NoPointInside => (INSIDE_A_PIECE, false, None),
                Search::OnlyRungsInside { unfollowed } => {
                    (PLACED_AGAINST_EVERY_SOLID, false, unfollowed)
                }
                Search::NoPointInsidePlaced => (PLACED_AGAINST_EVERY_SOLID, true, None),
            };
            let mut by_its_sides_or_corners = |log: &mut ProvenanceLog| {
                by_a_shared_side
                    .place(index, against, policy, log)
                    .or_else(|| by_a_corner(model, &solids, against, &corners, policy, log))
            };
            let by_its_plane = || by_a_plane(&solids, against, mine, piece);
            let placed = if the_plane_first {
                by_its_plane().or_else(|| by_its_sides_or_corners(log))
            } else {
                by_its_sides_or_corners(log).or_else(by_its_plane)
            };
            let by_the_piece_across = |in_plane: &InPlane| {
                by_the_piece_across_a_side(
                    model, scratch, &solids, against, index, in_plane, policy, log,
                )
            };
            let placed = placed
                .or_else(|| in_plane.as_ref().and_then(by_the_piece_across))
                .or_else(|| by_an_edge_of_a_face_it_lies_on(&solids, against, mine, piece))
                .or_else(|| by_a_side_from_an_edge_it_starts_on(&solids, against, mine, piece))
                .or_else(|| {
                    by_a_shared_side
                        .place_on_a_face_it_shares_a_side_with(index, against, policy, log)
                })
                .or_else(|| {
                    by_the_piece_across_a_side_clear_of_every_face(
                        model, scratch, &solids, against, index, policy, log,
                    )
                })
                .or_else(|| by_a_meridian_disk_beside_it(&solids, against, piece))
                .or_else(|| {
                    by_a_step_towards_a_corner_of_the_box_round_it(
                        model, &solids, against, piece, policy, log,
                    )
                });
            placed.ok_or_else(|| match unfollowed {
                Some(face) => Error::AFaceTrimmedByACurveThisCannotFollow { face },
                None => gave_up(unplaced, &corners.concat(), Measures::Length, log),
            })
        };

        let own_group: Vec<usize> = (0..solids.len())
            .filter(|&other| other != mine && (other < arguments) == (mine < arguments))
            .collect();
        let against_own = place(&own_group, log)?;
        if keeps(
            row_for(Rule::Union)
                .row(Side::of(mine, arguments))
                .fate(against_own),
            against_own,
            mine,
        )
        .is_none()
        {
            placed.push(None);
            continue;
        }

        let other_group: Vec<usize> = (0..solids.len())
            .filter(|&other| (other < arguments) != (mine < arguments))
            .collect();
        placed.push(Some(place(&other_group, log)?));
    }

    let selection = row_for(rule);
    if selection.keeps_a_buried_piece_apart_from_one_lying_the_other_way() {
        let held_by_both = kept_by(row_for(Rule::Intersection), &placed, scratch);
        for shell in
            the_pieces_of_each_flat_shell_of_both_groups(model, scratch, &held_by_both, policy, log)
        {
            for &piece in &shell {
                let of_an_argument = scratch.cut_from[piece] < arguments;
                let solid = shell
                    .iter()
                    .map(|&other| scratch.cut_from[other])
                    .find(|&other| (other < arguments) != of_an_argument)
                    .expect("a shell taken as flat holds pieces of both groups");
                placed[piece] = Some(Placement::On {
                    solid,
                    same_way: false,
                });
            }
        }
    }
    if selection.keeps_the_outside_of_both_groups() {
        let mut left_out = Vec::new();
        for (less, of_the_arguments) in [(&DIFFERENCE, true), (&TOOLS_LESS_ARGUMENTS, false)] {
            let kept = kept_by(less, &placed, scratch);
            for shell in
                the_pieces_of_each_flat_shell_of_both_groups(model, scratch, &kept, policy, log)
            {
                left_out.extend(shell.into_iter().map(|piece| {
                    let taken_from = (scratch.cut_from[piece] < arguments) == of_the_arguments;
                    let placement = if taken_from {
                        Placement::Inside
                    } else {
                        Placement::Outside
                    };
                    (piece, placement)
                }));
            }
        }
        for (piece, placement) in left_out {
            placed[piece] = Some(placement);
        }
    }
    Ok(kept_by(selection, &placed, scratch))
}

/// The pieces `selection` keeps, from where each piece sits against the other
/// group, or `None` for a piece its own group buries, which no rule keeps.
fn kept_by(
    selection: &'static Selection,
    placed: &[Option<Placement>],
    scratch: &Scratch,
) -> Vec<Chosen> {
    let arguments = scratch.arguments.len();
    placed
        .iter()
        .zip(&scratch.cut_from)
        .enumerate()
        .filter_map(|(piece, (placement, &mine))| {
            let placement = (*placement)?;
            let fate = selection.row(Side::of(mine, arguments)).fate(placement);
            keeps(fate, placement, mine).map(|reversed| Chosen { piece, reversed })
        })
        .collect()
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

/// What the tools less the arguments keep: [`DIFFERENCE`] with the two groups
/// the other way round.
///
/// No [`Rule`] answers it. A union asks which shells of it a tolerant policy
/// takes as flat, as it asks of [`DIFFERENCE`] ([`choose`]). Where a tool's
/// piece lies on an argument's facing the other way, the tool's is the one
/// kept, as [`DIFFERENCE`] keeps the argument's.
const TOOLS_LESS_ARGUMENTS: Selection = Selection {
    arguments: Row {
        inside: Fate::KeepReversed,
        outside: Fate::Drop,
        on_same_way: Fate::Drop,
        on_opposite_way: Fate::Drop,
    },
    tools: Row {
        inside: Fate::Drop,
        outside: Fate::Keep,
        on_same_way: Fate::Drop,
        on_opposite_way: Fate::Keep,
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

    /// Whether a piece buried in the other group fares otherwise than one
    /// lying on it facing the other way, in either group: so whether this rule
    /// needs the pieces of a shell taken as flat read as lying on the other
    /// group. Not for a union, which drops both. Nor may a union have them read
    /// so: a piece of such a shell lying on the other group facing the same
    /// way is the union's outside where the two stand flush, kept once as it
    /// lies, and read the other way the union would drop it.
    fn keeps_a_buried_piece_apart_from_one_lying_the_other_way(&'static self) -> bool {
        [&self.arguments, &self.tools]
            .iter()
            .any(|row| row.inside != row.on_opposite_way)
    }

    /// Whether this rule keeps every piece of either group lying outside the
    /// other, and drops every piece buried in it: so whether it answers one
    /// group with the other less it added on, and needs the pieces of a shell
    /// of that difference taken as flat read as the difference leaves them
    /// out. Only a union.
    fn keeps_the_outside_of_both_groups(&'static self) -> bool {
        [&self.arguments, &self.tools]
            .iter()
            .all(|row| row.outside == Fate::Keep && row.inside == Fate::Drop)
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

impl Side {
    /// The group of the solid `mine`, by its place in the two groups laid end
    /// to end, of which the first `arguments` are the arguments.
    fn of(mine: usize, arguments: usize) -> Self {
        if mine < arguments {
            Self::Argument
        } else {
            Self::Tool
        }
    }
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

/// Where one piece sits relative to a set of solids, or which of the two ways
/// the search for a point to place it by ran out.
///
/// Inside beats on, and on beats outside: a piece lying on one solid of a group
/// but buried in another is inside the union of them. So is a piece lying on
/// two of them that face opposite ways — see [`LyingOn::placement`].
///
/// **Two searches happen here, and a caller that cannot tell them apart is told
/// the wrong thing.** [`Search::NoPointInside`] says no point inside the piece
/// was found, which is [`INSIDE_A_PIECE`] unless a side the piece shares places
/// it ([`SidesItShares`]), a corner does ([`by_a_corner`]), a plane does
/// ([`by_a_plane`]) or the piece across a side does
/// ([`by_the_piece_across_a_side`]); [`Search::NoPointInsidePlaced`] says one
/// was found and could not be classified against every solid in the set, which
/// is [`PLACED_AGAINST_EVERY_SOLID`] unless a plane, a side, a corner or the
/// piece across a side places the piece. The first says to look at the piece's
/// trimming, the second says the piece is fine and a surface further down gave
/// up — the lens in §216.4 is the second wearing the first's name. Neither is a
/// refusal here, so that the caller can try the plane, the sides, the corners,
/// the pieces across and the edges before it refuses.
///
/// **The rungs of [`rungs_towards_each_crossing`] come last**, after
/// [`points_in_a_sliver_of_a_ball`] too, so a piece any earlier point placed
/// is placed by that point as it was. A piece only a rung was found inside,
/// and that no rung could place, is [`Search::OnlyRungsInside`] and not a
/// refusal here: before the rungs it was a piece with no point found, which a
/// side it shares or a corner could place, and a rung that cannot be
/// classified must not take that away (`docs/decisions.md` §365.6). A curved
/// piece's rungs are [`shorter_steps_across_a_curved_piece`],
/// [`between_the_middles_of_a_cone_pieces_sides`],
/// [`steps_from_a_reach_into_the_planes_material`] and then
/// [`half_way_round_the_tube_between_two_corners`], offered only where no
/// earlier point was found inside the piece, so a piece one was found in and
/// not placed is refused or placed as it was.
///
/// **A point whose ray met a face nobody can follow was never placed, and the
/// refusal says which face**, as `assemble.rs`'s `one_shell_is_inside_another`
/// does for a hollow's corners and for the same reason (§35.6). This used to
/// be [`PLACED_AGAINST_EVERY_SOLID`], and a caller read "loosen the policy" off
/// a refusal no policy moves. It wins over another point left undecided, when
/// both happen, for the reason that function gives.
///
/// **A rung's ray that met such a face is named too, but by the caller**, and
/// only once every rule [`choose`] tries after the search has failed to place
/// the piece: [`Search::OnlyRungsInside`] carries the face, so §365.6's order
/// stands. Q1040
/// found the piece that reaches it, the skin of a ball poking out of a half
/// ball's rim, placed against a cut ball in its own group whose skin nobody
/// follows (`tests/a_rung_whose_ray_meets_a_face_nobody_follows.rs`); it was
/// refused under [`PLACED_AGAINST_EVERY_SOLID`].
///
/// **A point placed with a decision the policy would refuse the fuse for
/// places the piece only where no later point places it without one.** The
/// piece is all inside or all outside each solid (§17.4), so every point that
/// places it says the same, and which of them says it decides nothing (§5).
/// Two quarter bends of ring 4 and tube 1 placed round by `−0.3565` and by
/// `−0.3565 − π/2` have touching caps that lie apart. The second candidate of
/// the copy's end cap, a rounding inside its rim at the ring's inner equator,
/// was placed outside the first bend by its second ray, after the first grazed
/// that bend's start cap on its rim, where `side of torus` cannot be told. The
/// graze's two records stayed in the point's log, and every rule both ways
/// round was refused under a certified policy, until this held the point back
/// for the next candidate, which places the cap outside and asks nothing
/// undecided (`tests/a_piece_no_ray_can_place_names_its_refusal.rs`, Q2267,
/// §35.6). Since Q2489 a ray `classify_uncertain_point` throws away leaves no
/// record, so that point places the cap itself and none is held back, and
/// Q2617's probe found no other test that hands this check a record.
///
/// The point held back is taken where no later point places the piece, and
/// where a later point's search fails, before the refusals below: a piece some
/// point placed before is still placed, and a fuse that was refused for that
/// point's records is refused for the same ones.
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] naming the caller's face,
/// when a point offered before the rungs was inside the piece, no point placed
/// it, and one of them met that face. Such a point that none placed for any
/// other reason is [`Search::NoPointInsidePlaced`], which the caller refuses
/// under [`PLACED_AGAINST_EVERY_SOLID`] when no rule [`choose`] tries after the
/// search places the piece.
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
) -> Result<Search> {
    if against.is_empty() {
        return Ok(Search::Placed(Placement::Outside));
    }
    let also_picked_by = what_else_each_round_wire_asks(piece, policy, log)?;
    let meridians = the_meridian_circle_each_side_runs_round(piece, policy, log)?;
    let mut unfollowed = None;
    let mut held_back: Option<(Placement, ProvenanceLog)> = None;
    let mut try_one = |candidate: PredicatePoint,
                       log: &mut ProvenanceLog|
     -> Result<Option<Option<Placement>>> {
        if the_piece_holds(piece, in_plane, &also_picked_by, &meridians, candidate) != Some(true) {
            return Ok(None);
        }
        let mut trial = ProvenanceLog::new();
        let settled =
            match settle_against_each(model, solids, against, candidate, piece, policy, &mut trial)
            {
                Ok(settled) => settled,
                Err(why) => {
                    let Some((placement, mut kept)) = held_back.take() else {
                        return Err(why);
                    };
                    kept.drain_into(log);
                    return Ok(Some(Some(placement)));
                }
            };
        Ok(Some(match settled {
            Settled::At(placement) if a_fuse_logging_this_is_refused(model, policy, &trial) => {
                held_back.get_or_insert((placement, trial));
                None
            }
            Settled::At(placement) => {
                trial.drain_into(log);
                Some(placement)
            }
            Settled::MetAFaceNobodyFollows(face) => {
                unfollowed.get_or_insert(face);
                None
            }
            Settled::Abandoned => None,
        }))
    };
    let mut one_was_inside = false;
    for &candidate in candidates {
        let Some(settled) = try_one(candidate, log)? else {
            continue;
        };
        one_was_inside = true;
        if let Some(placement) = settled {
            return Ok(Search::Placed(placement));
        }
    }
    if !one_was_inside {
        for candidate in points_in_a_sliver_of_a_ball(piece) {
            let Some(settled) = try_one(candidate, log)? else {
                continue;
            };
            one_was_inside = true;
            if let Some(placement) = settled {
                return Ok(Search::Placed(placement));
            }
        }
    }
    let mut a_rung_was_inside = false;
    let mut rungs = rungs_towards_each_crossing(piece);
    if !one_was_inside {
        rungs.extend(shorter_steps_across_a_curved_piece(piece));
        rungs.extend(between_the_middles_of_a_cone_pieces_sides(piece));
        rungs.extend(steps_from_a_reach_into_the_planes_material(piece));
        rungs.extend(half_way_round_the_tube_between_two_corners(piece));
    }
    for rung in rungs {
        let Some(settled) = try_one(rung, log)? else {
            continue;
        };
        a_rung_was_inside = true;
        if let Some(placement) = settled {
            return Ok(Search::Placed(placement));
        }
    }
    if let Some((placement, mut kept)) = held_back {
        kept.drain_into(log);
        return Ok(Search::Placed(placement));
    }
    if one_was_inside {
        if let Some(face) = unfollowed {
            return Err(Error::AFaceTrimmedByACurveThisCannotFollow { face });
        }
        return Ok(Search::NoPointInsidePlaced);
    }
    Ok(if a_rung_was_inside {
        Search::OnlyRungsInside { unfollowed }
    } else {
        Search::NoPointInside
    })
}

/// What [`place_the_piece`] found.
enum Search {
    Placed(Placement),
    /// No point offered was proved inside the piece.
    NoPointInside,
    /// Only rungs of [`rungs_towards_each_crossing`],
    /// [`shorter_steps_across_a_curved_piece`],
    /// [`between_the_middles_of_a_cone_pieces_sides`],
    /// [`steps_from_a_reach_into_the_planes_material`] or
    /// [`half_way_round_the_tube_between_two_corners`] were, and none of them
    /// could be classified against every solid in the set.
    OnlyRungsInside {
        /// The first face a rung's ray met whose trimming nothing here
        /// follows, if one did.
        unfollowed: Option<FaceKey>,
    },
    /// A point offered before the rungs was proved inside the piece, and no
    /// point inside it could be classified against every solid in the set.
    NoPointInsidePlaced,
}

/// §365.3's rule: a piece no point inside which was placed, placed against each
/// solid by a side it shares with a piece cut from one of that solid's faces.
///
/// A piece `P` and a solid `S`. If a side of `P` is also a side of a piece cut
/// from a face `G` of `S`, the other piece's side runs along none of `G`'s own
/// edges or a piece `Y` cut from `S` on `P`'s surface bit for bit has the side
/// too with its material on the partner's other side, `P`'s side and `Y`'s
/// name `G`'s surface bit for bit as their partner and the other piece's side
/// names `P`'s, and `P` is not on `G`'s surface, then **`P` is inside `S` when
/// its material is on `S`'s inner side of `G`**, and outside it otherwise.
///
/// **Both partners are asked because one name does not make one line.** Two
/// sides are one edge of the naming when their ends are named the same corners,
/// and a corner can take in two places a rounding apart. Two combs of the skew
/// family (Q1609) had the strip of one's gap floor `y = 1` between `x = 3` and
/// the other's wall `x + 2y = 5`, `3.7e-16` along, lying on the other's gap
/// floor. Its side on that wall, which is the other comb's own edge there, was
/// named one edge with the cut of that wall by the first comb's `x − 2y = 1`,
/// `9.3e-17` above it, whose ends were named the same two corners. Read off
/// that cut, the strip came out outside the other comb, and the intersection
/// kept neither copy of it. The cut's partner is `x − 2y = 1`, not `y = 1`.
///
/// `P` was cut by every surface of `S`, so it is all inside `S` or all outside
/// it, which is what one point already rests on (§17.4). Where the side is a
/// cut of `G` and not one of its edges, `S`'s skin beside the middle of the
/// side is `G` alone, and `P` is on the side of `G`'s surface that
/// [`FaceMeet::material_side`] names, which the walk that made the side decided
/// for nothing (§36.6). No number is built and nothing is a tolerance.
///
/// **Along an edge of `G`, `Y` says that `P` carries on past `G`.** Take a
/// point `m` of the side inside an edge `e` of `G` it runs along. `S`'s skin
/// beside `m` is `G` and the face `H` across `e`. `Y`'s inside comes up to `m`
/// and lies on `S`'s skin, on `P`'s surface, which is not `G`'s, so it lies on
/// `H`: `H` is on `P`'s surface, and beside `m` it is on the other side of
/// `G`'s surface from `P`. So `P` is off `S`'s skin there, and leaves `m` the
/// way opposite to `H`. Cut across `e` at `m`, `G` and `H` leave `m` at an
/// angle `θ` through `S`, never `π`, since the side is a crossing of the two
/// surfaces and not a touching (§36.6). Where `θ < π`, `H` leaves on `G`'s
/// inner side, so `P` leaves on its outer side, out of the angle: outside.
/// Where `θ > π`, `H` leaves on `G`'s outer side and `P` on its inner side,
/// into the angle: inside. Both are the rule's answer. The shape that needs it
/// is two hollow quarters of one cone side by side, whose side planes round
/// into each other: the piece of the first quarter's cone between the two
/// planes, from `z = 4` to `z = 4.5`, has its top on the rim of the other
/// quarter's bottom cap, and nothing else places it. Turned a trillionth of a
/// radian further into each other, the same holds of the piece of the first
/// quarter's bore there
/// (`tests/two_quarters_of_a_hollow_cone_flush_along_a_plane.rs`, Q2551).
/// Where the two planes cross the rim at one `f64` triple, the other quarter's
/// side there is a stretch of the rim off its own end, which the answer
/// leaves out as having no length, and the naming keeps it for this, by its
/// two ends (Q2604).
///
/// **None of this asks whether a point was found inside `P`**, so the rule
/// holds where one was found and the ray could not place it. Nor can `P` lie on
/// another face of `S`: beside the side, `S`'s skin is `G` alone, or `G` and a
/// face on the far side of `G`'s surface from `P`.
/// The piece Q1064 brought here is
/// half of a bar's wall, `x ≥ 0`, against a block over `x < 0` that a bar on
/// the same pipe was united with first. Every point of the half lies on the
/// pipe, which that first bar's wall is on and holds none of, and the ray from
/// each grazed that wall. Its two sides on `x = 0` are cuts of the block's
/// face there, and its material is on the face's outer side. Since Q1080 the
/// ray places that half itself and it no longer comes here.
///
/// Which sides are one is [`the_edge_each_side_of_every_piece_is`], the
/// naming `assemble` gives the kept pieces, worked out once and only for a fuse
/// that reaches here, so a fuse that placed every piece by a point or a plane
/// never pays for it.
struct SidesItShares<'a> {
    model: &'a Model,
    scratch: &'a Scratch,
    solids: &'a [&'a BakedSolid],
    /// Every side of every piece with the edge it names: `None` until a piece
    /// first needs it, and `Some(None)` when the naming refused.
    named: Option<Option<Vec<Vec<NamedSide>>>>,
    /// What naming the sides asked the policy, kept out of the log until a
    /// piece is placed by them.
    asked: ProvenanceLog,
}

impl<'a> SidesItShares<'a> {
    fn of(model: &'a Model, scratch: &'a Scratch, solids: &'a [&'a BakedSolid]) -> Self {
        Self {
            model,
            scratch,
            solids,
            named: None,
            asked: ProvenanceLog::new(),
        }
    }

    /// Where piece number `piece` sits against the solids `against`, or `None`
    /// when some solid of them could not be placed this way and none had the
    /// piece inside it.
    ///
    /// **Nothing here refuses.** A naming that refuses, a plane agreement
    /// that cannot be told, or two shared sides that disagree each leave the
    /// piece where the point search left it, and unless a rule [`choose`]
    /// tries after this places it the caller refuses under the name that
    /// search ran out under, as it did before this was tried.
    fn place(
        &mut self,
        piece: usize,
        against: &[usize],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Option<Placement> {
        self.name_the_sides(policy);
        let named = self.named.as_ref().and_then(Option::as_ref)?;
        let mut trial = ProvenanceLog::new();
        let mut inside_one = false;
        let mut outside_every_one = true;
        for &solid in against {
            match self.is_inside(named, piece, solid, policy, &mut trial) {
                Some(true) => inside_one = true,
                Some(false) => (),
                None => outside_every_one = false,
            }
        }
        let placement = if inside_one {
            Placement::Inside
        } else if outside_every_one {
            Placement::Outside
        } else {
            return None;
        };
        self.asked.drain_into(log);
        trial.drain_into(log);
        Some(placement)
    }

    /// Name every side of every piece, the first time a piece needs it.
    fn name_the_sides(&mut self, policy: &Policy) {
        if self.named.is_none() {
            self.named = Some(
                the_edge_each_side_of_every_piece_is(
                    self.model,
                    self.scratch,
                    policy,
                    &mut self.asked,
                )
                .ok(),
            );
        }
    }

    /// Whether piece number `piece` is inside solid number `solid`, read off
    /// every side it shares with a piece cut from a face of that solid, or
    /// `None` when no side says or two say different things.
    fn is_inside(
        &self,
        named: &[Vec<NamedSide>],
        piece: usize,
        solid: usize,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Option<bool> {
        let mine = &self.scratch.pieces[piece];
        let my_surface = name_of_surface_bits(&mine.surface);
        let mut found = None;
        for shared in &named[piece] {
            let theirs = named
                .iter()
                .zip(&self.scratch.cut_from)
                .enumerate()
                .filter(|(_, (_, from))| **from == solid)
                .flat_map(|(other, (sides, _))| sides.iter().map(move |side| (other, side)))
                .filter(|(_, side)| {
                    side.edge == shared.edge
                        && name_of_surface_bits(&side.side.carried_by[1]) == my_surface
                        && (side.side.along.is_empty()
                            || self
                                .its_skin_on_my_surface_stops_at(named, solid, shared, my_surface))
                });
            for (other, _) in theirs {
                let cut_from = &self.scratch.pieces[other];
                let face = self.solids[solid]
                    .faces
                    .iter()
                    .find(|face| face.keys == cut_from.ancestors)
                    .expect("a piece is cut from a face of the solid it is credited to");
                if name_of_surface_bits(&shared.side.carried_by[1])
                    != name_of_surface_bits(&face.surface)
                {
                    continue;
                }
                let on_its_surface =
                    do_they_face_the_same_way(mine, outward_plane_of(mine), face, policy, log)
                        .ok()?;
                if on_its_surface.is_some() {
                    return None;
                }
                let inner = if face.orientation.is_reversed() {
                    Decision::Pos
                } else {
                    Decision::Neg
                };
                let inside = shared.side.material_side == inner;
                if found.is_some_and(|already| already != inside) {
                    return None;
                }
                found = Some(inside);
            }
        }
        found
    }

    /// Whether a piece cut from solid number `solid`, on the surface named
    /// `surface` bit for bit, has the side `shared` too, with the same partner
    /// bit for bit and its material on the partner's other side: the face of
    /// `solid` that piece is cut from ends at the side, and the piece with
    /// `shared` carries its surface on past it.
    fn its_skin_on_my_surface_stops_at(
        &self,
        named: &[Vec<NamedSide>],
        solid: usize,
        shared: &NamedSide,
        surface: [u64; 9],
    ) -> bool {
        let partner = name_of_surface_bits(&shared.side.carried_by[1]);
        named
            .iter()
            .zip(&self.scratch.cut_from)
            .zip(&self.scratch.pieces)
            .filter(|((_, from), piece)| {
                **from == solid && name_of_surface_bits(&piece.surface) == surface
            })
            .flat_map(|((sides, _), _)| sides)
            .any(|side| {
                side.edge == shared.edge
                    && side.side.material_side != shared.side.material_side
                    && name_of_surface_bits(&side.side.carried_by[1]) == partner
            })
    }

    /// The rule after [`by_a_side_from_an_edge_it_starts_on`]: piece number
    /// `piece` placed against the solids `against` by a side it shares with a
    /// piece cut from a face of one of them on its own surface, with the same
    /// partner and the same material side (`docs/decisions.md` §35.6, Q1537).
    ///
    /// **Why such a side puts a piece on the skin.** `P` is the piece, `S` the
    /// solid, and `Q` a piece cut from a face `G` of `S`, on `P`'s surface. Both
    /// have the side `s`, carried by that surface and one partner surface, and
    /// each has its material on the same side of the partner. So near the
    /// middle of `s` both are the part of the one surface on that side of the
    /// partner, and `P`'s inside meets `Q`'s in an open set. `Q` lies in `G`,
    /// which is on `S`'s skin. `P` was cut by every surface of `S`, so its inside
    /// lies all inside `S`, all outside it, or all on its skin (§17.4), and so
    /// it lies on the skin, on `G`, facing `G`'s way or the other, as
    /// [`do_they_face_the_same_way`] says. Nothing asks where `s` runs, and
    /// [`Self::is_inside`] passes this case over: it reads the side of a face
    /// `P` crosses, and `P` here crosses none.
    ///
    /// Both partners are asked bit for bit and the two material sides compared,
    /// as [`Self::is_inside`] asks them, since one name does not make one line.
    /// A solid [`Self::is_inside`] places is placed so, inside one solid of the
    /// group is inside, and lying on one or more is read as [`LyingOn`] reads
    /// it. A solid neither rule places leaves the piece unplaced.
    ///
    /// The pieces this was built for are the two bands of two quarter bends
    /// placed a quarter turn apart by a float turn: each bend's cap plane cuts
    /// the other's wall a rounding round from that bend's own cap rim, and the
    /// band between holds no `f64` point. Each band's own rim is the other
    /// band's cut, one circle named once, started at two places
    /// (`tests/a_piece_no_ray_can_place_names_its_refusal.rs`). Placed round by
    /// `0.3` and `0.3 + π/2` instead of `0` and `π/2`, there are no such bands:
    /// the first bend's end cap lies strictly in front of the copy's start cap
    /// plane, an ulp off it, so neither plane cuts the other bend's wall, and
    /// that cap is placed by [`by_a_meridian_disk_beside_it`] (Q1948, Q2056).
    fn place_on_a_face_it_shares_a_side_with(
        &mut self,
        piece: usize,
        against: &[usize],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Option<Placement> {
        self.name_the_sides(policy);
        let named = self.named.as_ref().and_then(Option::as_ref)?;
        let mut trial = ProvenanceLog::new();
        let mut inside_one = false;
        let mut each_one_placed = true;
        let mut lying_on = LyingOn::default();
        for &solid in against {
            match self.is_inside(named, piece, solid, policy, &mut trial) {
                Some(true) => inside_one = true,
                Some(false) => (),
                None => match self.lies_on(named, piece, solid, policy, &mut trial) {
                    Some(same_way) => lying_on.add(solid, same_way),
                    None => each_one_placed = false,
                },
            }
        }
        let placement = if inside_one {
            Placement::Inside
        } else if each_one_placed {
            lying_on.placement()
        } else {
            return None;
        };
        self.asked.drain_into(log);
        trial.drain_into(log);
        Some(placement)
    }

    /// Whether piece number `piece` lies on solid number `solid` facing the
    /// same way as the face it lies on, by
    /// [`Self::place_on_a_face_it_shares_a_side_with`]'s rule, or `None` when
    /// no side says or two say different things.
    fn lies_on(
        &self,
        named: &[Vec<NamedSide>],
        piece: usize,
        solid: usize,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Option<bool> {
        let mine = &self.scratch.pieces[piece];
        let mut found = None;
        for shared in &named[piece] {
            let partner = name_of_surface_bits(&shared.side.carried_by[1]);
            let theirs = named
                .iter()
                .zip(&self.scratch.cut_from)
                .enumerate()
                .filter(|(_, (_, from))| **from == solid)
                .flat_map(|(other, (sides, _))| sides.iter().map(move |side| (other, side)))
                .filter(|(_, side)| {
                    side.edge == shared.edge
                        && side.side.material_side == shared.side.material_side
                        && name_of_surface_bits(&side.side.carried_by[1]) == partner
                });
            for (other, _) in theirs {
                let cut_from = &self.scratch.pieces[other];
                let face = self.solids[solid]
                    .faces
                    .iter()
                    .find(|face| face.keys == cut_from.ancestors)
                    .expect("a piece is cut from a face of the solid it is credited to");
                let Some(same_way) =
                    do_they_face_the_same_way(mine, outward_plane_of(mine), face, policy, log)
                        .ok()?
                else {
                    continue;
                };
                if found.is_some_and(|already| already != same_way) {
                    return None;
                }
                found = Some(same_way);
            }
        }
        found
    }
}

/// The rule after §365.3's: a piece no point inside which was placed, and that
/// no side it shares placed, placed against each solid by the first of its own
/// corners the solid's ray puts strictly inside or strictly outside it.
///
/// **Why a corner can place a piece.** A piece `P` and a solid `S`. `P` was cut
/// by every surface of `S`, so its inside crosses no face of `S`, and that
/// inside lies all inside `S`, all outside it, or all on its skin — the premise
/// one point already rests on (§17.4). A corner `c` of `P` is on `P`'s
/// outline, so every open neighbourhood of `c` meets `P`'s inside. When `c` is
/// strictly inside `S`, a neighbourhood of it is inside `S` too, and the part of
/// `P`'s inside it meets is inside `S`, which puts all of `P` inside `S`; it
/// cannot be on `S`'s skin, which that neighbourhood does not touch. Strictly
/// outside is the same argument the other way. No number is built and nothing
/// is a tolerance: the ray decides the sign about the corner's whole box.
///
/// **A corner on `S`'s skin says nothing**, since a piece on either side of a
/// face, or on it, can have its corner there, so
/// [`Containment::OnBoundary`] is never read as a placement. Nor is a corner on
/// the stored surface of a face of `S`, wherever on that surface it is, unless
/// the face is flat with a curved side:
///
/// - The ray reads a flat face whose every side is straight by triangles
///   fanned from its stored corners, which a turned solid keeps a rounding off
///   the stored plane, and about a point on that plane it can certify a side
///   the plane does not have (§316.3, §351). The point search keeps such
///   points from the ray by [`coincident_face`], which a corner has no face of
///   its own to be asked of.
/// - From a corner on a curved surface the ray can place the corner and still
///   log a side of that surface it could not decide, and that record refuses
///   the whole fuse under a certified policy. Q2243 asked such corners too,
///   and the bend placed half a turn round from another, which answers, was
///   refused so: the ray from `(0, 3, 0)`, on the other bend's ring, placed it
///   outside and logged an undecided side of the ring beside it
///   (`tests/a_piece_no_ray_can_place_names_its_refusal.rs`).
///
/// A flat face with a curved side the ray reads by where it meets the stored
/// plane, and a point on that plane it hands to the face's outline, which says
/// whether the face holds it, so a corner on such a plane is asked. The corner
/// of a block that sits on a round tab's rim, on the plane of the plate's top
/// or floor, is placed so (Q2243, `tests/a_block_cornered_on_a_round_tabs_rim.rs`).
///
/// The piece this was built for is the strip of a KiCad pin header's wall
/// between two notch chamfers that meet it at one line by design, and a
/// rounding apart over the stored planes, so no `f64` lies between them
/// (`docs/decisions.md` §365.7). Inside one solid of the group is inside,
/// outside every one is outside, and anything else is `None`, so the caller
/// tries [`by_a_plane`] where it has not yet, and refuses under the name the
/// point search ran out under if nothing places the piece. What the ray asked
/// the policy goes into the log only when the piece is placed.
fn by_a_corner(
    model: &Model,
    solids: &[&BakedSolid],
    against: &[usize],
    corners: &[Vec<PredicatePoint>],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Option<Placement> {
    let mut trial = ProvenanceLog::new();
    let mut inside_one = false;
    let mut outside_every_one = true;
    for &index in against {
        let solid: SolidView<'_> = model.solid_at(solids[index].key.slot_in(model.id()));
        match is_inside_by_a_corner(solid, corners, policy, &mut trial) {
            Some(true) => inside_one = true,
            Some(false) => (),
            None => outside_every_one = false,
        }
    }
    let placement = if inside_one {
        Placement::Inside
    } else if outside_every_one {
        Placement::Outside
    } else {
        return None;
    };
    trial.drain_into(log);
    Some(placement)
}

/// Whether the first corner `solid`'s ray places strictly is inside it, or
/// `None` when no corner off every surface but a plane with a curved side was
/// placed strictly.
fn is_inside_by_a_corner(
    solid: SolidView<'_>,
    corners: &[Vec<PredicatePoint>],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Option<bool> {
    corners.iter().flatten().find_map(|&corner| {
        if !is_off_every_surface_but_a_plane_with_a_curved_side(solid, corner) {
            return None;
        }
        let mut ray = ProvenanceLog::new();
        let inside = match solid.classify_uncertain_point(corner, policy, &mut ray) {
            Containment::Inside => true,
            Containment::Outside => false,
            Containment::OnBoundary
            | Containment::Undecidable { .. }
            | Containment::CannotFollowAFacesTrimming { .. }
            | Containment::CannotFireARayAtAFacesSurface { .. } => return None,
        };
        ray.drain_into(log);
        Some(inside)
    })
}

/// Whether every face of `solid` puts `point` strictly on one side of its
/// stored surface, but a flat face with a curved side, whose plane `point` may
/// lie on.
fn is_off_every_surface_but_a_plane_with_a_curved_side(
    solid: SolidView<'_>,
    point: PredicatePoint,
) -> bool {
    solid
        .faces()
        .all(|face| is_flat_with_a_curved_side(face) || is_strictly_off_the_surface_of(face, point))
}

/// Whether `face` is flat and has a side that is not straight. The ray reads
/// such a face by where it meets the stored plane, and a point on that plane
/// it hands to the face's own outline; a flat face whose every side is
/// straight it reads by triangles fanned from its stored corners, the test
/// `classify.rs`'s `count_crossings` routes a face to `count_a_polygon` by.
fn is_flat_with_a_curved_side(face: FaceView<'_>) -> bool {
    face.surface().as_plane().is_some() && !every_side_of_it_is_straight(face)
}

/// A flat piece with straight sides placed outside each solid by a plane of
/// one of that solid's faces that has the whole solid on or behind it, every
/// corner of the piece on or in front of it, and one corner strictly in front.
/// Outside every solid of the group is outside; anything else is `None`, and
/// the caller tries the sides and the corners where it has not yet, then the
/// piece across a side ([`by_the_piece_across_a_side`]), or refuses.
///
/// **Why a plane can place a piece.** Every face of a solid `S` here is flat,
/// so `S`'s skin is polygons with straight sides, which lie in the hull of
/// their corners, and so does `S`. With every corner of `S` on or behind a
/// plane `σ`, all of `S` is. A piece `P` with straight sides lies in the hull
/// of its own corners too, so with each of them on or in front of `σ` all of
/// `P` is. One corner strictly in front puts `P`'s plane off `σ`, so the two
/// planes cross in a line, `P` lies on one side of that line, and `P`'s inside,
/// which is open in its plane, lies strictly in front of `σ`: it shares no
/// point with `S`, though a side of `P` may lie on `σ`. Unlike [`by_a_corner`],
/// this does not lean on `P` having been cut by every surface of `S`.
///
/// **A corner on `σ` is allowed because a piece a rounding across has them.**
/// A cube turned so that its corner touches a block's face from inside, with
/// its three planes meeting `1e−16` inside the face, crosses that face in
/// three lines cutting out a triangle a rounding across, in front of all
/// three planes. No point is found inside it, two of its corners are on each
/// of the three planes, and with every corner strictly in front asked for, all
/// of Q1208's 201 random turns of that kind were refused (Q1288,
/// `tests/a_turned_cube_with_its_corner_on_a_face.rs`).
///
/// **Each corner is asked as the one point where three planes meet**
/// ([`side_of_a_plane_where_three_planes_meet`]), never by its box, and the
/// answer is exact, so nothing goes into the log. The boxes are what cannot
/// say. Two cubes turned so that two faces lie a rounding apart face to face
/// have every corner of one within a box's width of the other's face, and a
/// point picked inside one face lies on its plane, a rounding off the other's,
/// where the ray cannot tell which side it starts on: every fuse of the two
/// was refused, though the cubes share no point
/// (`tests/two_turned_cubes_a_rounding_apart.rs`).
///
/// A corner of `S` is the meet of the three planes bake solved it from
/// ([`BakedCorner::solved_from`](super::bake::BakedCorner)). With exactly three
/// round it those are the three; with four or more, as at a square pyramid's
/// apex, they are the three whose meet the corner's box holds, and four planes
/// written in `f64` need not meet in one point. The arrangement asks those
/// same three where it cuts a face by the corner (`passes_through`) and where
/// it puts the corner in order along an edge (`as_the_planes_say`), §294.19
/// and §294.20, so the `S` it cut the pieces against has its corner at that
/// meet, and so does the hull above. A corner of `P` that is a corner of the
/// input is the meet of the surfaces round it when they are exactly three
/// planes ([`BakedCorner::around`](super::bake::BakedCorner)), and where there
/// are more, the meet of `P`'s plane and the two planes that cut it along `P`'s
/// sides there ([`where_the_lines_of_two_sides_meet`]); a meet the
/// arrangement placed is the meet of the three surfaces it names. A corner of
/// `S` read out of the model, which nothing solved, any other corner of `P`, a
/// corner of `P` not on `P`'s own plane, or a curved surface on `P` or on `S`
/// leaves this rule out. Until Q1936 a corner of `S` was read from the planes
/// round it too, so a solid with four planes round a corner was never asked,
/// and the pyramid placed and cut through its apex refused on a speck of the
/// cut a rounding across (§294.21). Until Q2334 a corner of `P` with four
/// planes round it was left out the same way, and the pyramid turned about `x`
/// and cut by a wedge whose edge runs up through its apex refused on a speck of
/// its `−x` face a rounding across by the apex.
///
/// **And inside a solid whose every face plane has all of it on or behind,
/// with every corner of the piece on or behind every one of them and, for
/// each, one corner strictly behind** (Q1353, `docs/decisions.md` §417). Such
/// a solid `S` is convex. Take `p` inside `P`, open in `P`'s plane, and a face
/// plane `σ` of `S`. All of `P` is on or behind `σ`, as above; were `p` on
/// `σ`, the distance to `σ` would be at most zero on a disk round `p` in `P`'s
/// plane and zero at its middle, so zero on all of that plane, and every
/// corner of `P` on `σ`, which one strictly behind rules out. So `p` is
/// strictly behind every face plane of `S`. A point `k` inside `S` is too,
/// since a ball round it is in `S`. Every point between `k` and `p` is then
/// strictly behind every face plane, so on no face of `S`, and the segment
/// never leaves `S`: `p` is inside `S`. Inside one solid of the group is
/// inside, as [`by_a_corner`] reads it. The piece this was built for is the
/// same triangle with the turned cube's planes meeting a rounding *outside*
/// the block's face, where the triangle is the cube's tip cut off by the face,
/// inside the cube: each of its corners is on two of the cube's planes there
/// and strictly behind the third.
fn by_a_plane(
    solids: &[&BakedSolid],
    against: &[usize],
    mine: usize,
    piece: &Piece,
) -> Option<Placement> {
    let corners = the_planes_meeting_at_each_corner_of_a_piece(piece, solids[mine])?;
    let mut outside_every_one = true;
    for &index in against {
        match placed_by_the_planes_of(solids[index], &corners) {
            Some(true) => return Some(Placement::Inside),
            Some(false) => (),
            None => outside_every_one = false,
        }
    }
    outside_every_one.then_some(Placement::Outside)
}

/// The three planes each corner of a flat piece with straight sides is the
/// one meeting point of, as coefficients, or `None` where the piece is not
/// that or some corner is not such a meet on the piece's own plane.
fn the_planes_meeting_at_each_corner_of_a_piece(
    piece: &Piece,
    own: &BakedSolid,
) -> Option<Vec<[[f64; 4]; 3]>> {
    let flat = *piece.surface.as_plane()?;
    piece
        .wires
        .iter()
        .flat_map(|wire| {
            (0..wire.len()).map(|at| (&wire[(at + wire.len() - 1) % wire.len()], &wire[at]))
        })
        .map(|(before, side)| {
            if !is_straight(side) {
                return None;
            }
            let three = match the_planes_meeting_at(&side.ends[0], own, flat) {
                Some(three) => three,
                None => where_the_lines_of_two_sides_meet(before, side, own, flat)?,
            };
            Some(three.map(|plane| plane.coefficients()))
        })
        .collect()
}

/// `flat` and the planes that cut it along `before` and `side`, where
/// `before` ends at the corner of the input `side` starts at and that corner
/// has more than three planes round it, or `None` where it is not that.
///
/// **A corner of a flat piece with straight sides is where the lines of its
/// two sides cross.** Each side runs along `flat` and its partner plane, so
/// the one point on both is the meet of `flat` and the two partners, and the
/// piece is the polygon with that corner. No other reading puts both sides on
/// their lines. With four or more planes round the corner, as at a square
/// pyramid's apex, bake solved it from three of them, and on the face whose
/// plane is not one of the three the corner it solved is a rounding off
/// `flat`. Placed by a turn, the pyramid's `−x` face is cut by a wedge's face
/// `y = 2x` into a triangle a rounding across by the apex, with no point
/// inside it, and its corner at the apex is the meet of `flat` and the `±y`
/// planes, the face's own top corner (§294).
fn where_the_lines_of_two_sides_meet(
    before: &FaceMeet,
    side: &FaceMeet,
    own: &BakedSolid,
    flat: Plane,
) -> Option<[Plane; 3]> {
    let MeetPoint::Existing { vertex, .. } = side.ends[0] else {
        return None;
    };
    if before.ends[1] != side.ends[0] || the_corner_at(own, vertex).around.len() <= 3 {
        return None;
    }
    let [one, other] = [before, side].map(|side| {
        let [carrier, partner] = &side.carried_by;
        is_one_plane_either_way(*carrier.as_plane()?, flat)
            .then(|| partner.as_plane().copied())
            .flatten()
    });
    Some([flat, one?, other?])
}

/// Whether `side` runs where two planes meet, and so along a straight line.
fn is_straight(side: &FaceMeet) -> bool {
    side.carried_by
        .iter()
        .all(|surface| surface.as_plane().is_some())
}

/// The three planes `end` is the one meeting point of, one of them `flat`
/// either way round, or `None` where it is no such meet.
fn the_planes_meeting_at(end: &MeetPoint, own: &BakedSolid, flat: Plane) -> Option<[Plane; 3]> {
    let three = match end {
        MeetPoint::Existing { vertex, .. } => three_planes(&the_corner_at(own, *vertex).around)?,
        MeetPoint::OfThreeSurfaces { surfaces, .. } => three_planes(surfaces)?,
        MeetPoint::WhereAClosedCurveStarts { .. }
        | MeetPoint::HalfWayRoundAClosedCurve { .. }
        | MeetPoint::APoleOfAClosedSurface { .. } => return None,
    };
    three
        .iter()
        .any(|plane| *plane == flat || *plane == flat.reversed())
        .then_some(three)
}

/// Whether the planes of `solid`'s faces, each turned to face out of it, put
/// a piece with `corners` outside it (`Some(false)`) or inside it
/// (`Some(true)`), by [`by_a_plane`]'s two rules, or `None` when neither
/// holds.
///
/// Outside: one plane has every corner of the solid on or behind it, every one
/// of `corners` on or in front of it, and one of `corners` strictly in front.
/// Inside: every plane has every corner of the solid on or behind it, every
/// one of `corners` on or behind it, and one of `corners` strictly behind.
fn placed_by_the_planes_of(solid: &BakedSolid, corners: &[[[f64; 4]; 3]]) -> Option<bool> {
    let its_own = the_planes_meeting_at_each_corner_of_a_solid(solid)?;
    let mut inside_every_one = true;
    for face in &solid.faces {
        let outward = outward_plane_of_face(face)
            .expect("every face of a solid whose corners all meet three planes is flat")
            .coefficients();
        let side =
            |three: &[[f64; 4]; 3]| side_of_a_plane_where_three_planes_meet(*three, outward).ok();
        let solid_behind = its_own
            .iter()
            .all(|three| matches!(side(three), Some(Decision::Neg | Decision::Zero)));
        let piece: Vec<Option<Decision>> = corners.iter().map(side).collect();
        let piece_in_front = piece
            .iter()
            .all(|side| matches!(side, Some(Decision::Pos | Decision::Zero)))
            && piece.contains(&Some(Decision::Pos));
        if solid_behind && piece_in_front {
            return Some(false);
        }
        let piece_behind = piece
            .iter()
            .all(|side| matches!(side, Some(Decision::Neg | Decision::Zero)))
            && piece.contains(&Some(Decision::Neg));
        inside_every_one &= solid_behind && piece_behind;
    }
    inside_every_one.then_some(true)
}

/// The three planes bake solved each corner of every face of `solid` from, as
/// coefficients, or `None` where a face is curved or a corner was read out of
/// the model and not solved, or solved from a curved surface.
fn the_planes_meeting_at_each_corner_of_a_solid(solid: &BakedSolid) -> Option<Vec<[[f64; 4]; 3]>> {
    if solid
        .faces
        .iter()
        .any(|face| face.surface.as_plane().is_none())
    {
        return None;
    }
    solid
        .faces
        .iter()
        .flat_map(|face| face.wires.iter().flatten())
        .map(|edge| {
            Some(three_planes(edge.start.solved_from.as_ref()?)?.map(|plane| plane.coefficients()))
        })
        .collect()
}

/// The rule after the sides, the corners and the plane: a flat piece that no
/// point, side it shares, corner or plane placed takes the placement of the
/// piece of its own face across one of its straight sides, where no plane of a
/// solid it is placed against holds that side, unless that plane is the
/// face's own. Where it places nothing, [`by_an_edge_of_a_face_it_lies_on`]
/// is tried next, then
/// [`by_a_side_from_an_edge_it_starts_on`],
/// [`SidesItShares::place_on_a_face_it_shares_a_side_with`], and
/// [`by_the_piece_across_a_side_clear_of_every_face`] last.
///
/// **Why a neighbour can place a piece.** `P` and `P′` are pieces of one flat
/// face `F` that share a side `s`, and `S` is a solid with only flat faces.
/// Each of the two was cut by every plane of `S`, so each lies all inside `S`,
/// all outside it or all on its skin (§17.4). A plane of `S` that does not hold
/// both ends of `s` meets the line of `s` in one point at most, so all but a
/// few points `m` of `s` lie on no plane of `S` but `F`'s own.
///
/// - Off `S`'s skin, `m` has a neighbourhood all inside `S` or all outside it,
///   and that neighbourhood meets the inside of `P` and of `P′`, so the two
///   are placed alike.
/// - On it, `m` is on a face of `S` in `F`'s plane and on no other plane of
///   `S`, so near `m` `S`'s skin is a disk of that plane, with `S` on one side.
///   `P` and `P′` both reach into that disk, so both lie on `S`'s skin, on the
///   same face plane, facing the same way.
///
/// A plane of `S` that holds both ends of `s` and is not `F`'s is where `P`
/// and `P′` can differ, along an edge of `S`, so such a side is passed over.
/// Each end is asked as the meet of its three planes, exactly, as
/// [`by_a_plane`] asks, so nothing goes into the log for it. `P′` is placed by
/// [`place_the_piece`], its own point search, and what that asked the policy
/// goes into the log only when it places `P`.
///
/// The piece this was built for is a triangle a rounding across on a comb's
/// top, cut out by two of the comb's walls and a leaning wall of an ell
/// standing on that top, all three through the comb's corner at the foot of a
/// tooth (`docs/decisions.md` §365.9). It holds no `f64` point, every corner
/// of it is on the plane of the ell's floor, though well off the floor, and
/// the ell is not convex, so no plane of the ell has it in front. The piece
/// across its side along the comb's own wall `y = 1` holds points, and the
/// ray places them.
#[expect(
    clippy::too_many_arguments,
    reason = "the neighbour is placed by place_the_piece, which asks for all of these"
)]
fn by_the_piece_across_a_side(
    model: &Model,
    scratch: &Scratch,
    solids: &[&BakedSolid],
    against: &[usize],
    piece: usize,
    in_plane: &InPlane,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Option<Placement> {
    let mine = &scratch.pieces[piece];
    let flat = *mine.surface.as_plane()?;
    let own = solids[scratch.cut_from[piece]];
    let theirs: Vec<Plane> = against
        .iter()
        .flat_map(|&index| &solids[index].faces)
        .map(|face| face.surface.as_plane().copied())
        .collect::<Option<_>>()?;
    let others: Vec<[f64; 4]> = theirs
        .iter()
        .filter(|plane| **plane != flat && **plane != flat.reversed())
        .map(Plane::coefficients)
        .collect();
    mine.wires.iter().flatten().find_map(|side| {
        if !no_plane_holds_both_ends(side, own, flat, &others) {
            return None;
        }
        let across = &scratch.pieces[the_piece_across(scratch, piece, side)?];
        let mut trial = ProvenanceLog::new();
        let candidates = candidate_points(across, &corners_of(across));
        let search = place_the_piece(
            model,
            solids,
            against,
            &candidates,
            across,
            Some(in_plane),
            policy,
            &mut trial,
        );
        let Ok(Search::Placed(placement)) = search else {
            return None;
        };
        trial.drain_into(log);
        Some(placement)
    })
}

/// Whether `side` is straight and every one of `planes` puts one end of it
/// or the other strictly on one side, each end asked exactly as the meet of
/// its three planes, one of them `flat`.
fn no_plane_holds_both_ends(
    side: &FaceMeet,
    own: &BakedSolid,
    flat: Plane,
    planes: &[[f64; 4]],
) -> bool {
    if !is_straight(side) {
        return false;
    }
    let Some(ends) = side
        .ends
        .iter()
        .map(|end| Some(the_planes_meeting_at(end, own, flat)?.map(|plane| plane.coefficients())))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    planes.iter().all(|&plane| {
        ends.iter().any(|&three| {
            matches!(
                side_of_a_plane_where_three_planes_meet(three, plane),
                Ok(Decision::Pos | Decision::Neg)
            )
        })
    })
}

/// The other piece cut from the same face as piece number `piece` that has
/// `side` as a side, run the other way, if there is one.
fn the_piece_across(scratch: &Scratch, piece: usize, side: &FaceMeet) -> Option<usize> {
    let mine = &scratch.pieces[piece];
    let from = scratch.cut_from[piece];
    scratch
        .pieces
        .iter()
        .zip(&scratch.cut_from)
        .enumerate()
        .find(|&(other, (theirs, &their_from))| {
            other != piece
                && their_from == from
                && theirs.ancestors == mine.ancestors
                && theirs.wires.iter().flatten().any(|their_side| {
                    their_side.carried_by == side.carried_by
                        && their_side.ends[0] == side.ends[1]
                        && their_side.ends[1] == side.ends[0]
                })
        })
        .map(|(other, _)| other)
}

/// The corner of `solid` that is `vertex`.
///
/// # Panics
///
/// If no face of `solid` has a corner there: a piece names as a corner of the
/// input only a corner of the face it was cut from.
fn the_corner_at(solid: &BakedSolid, vertex: VertexKey) -> &BakedCorner {
    &solid
        .faces
        .iter()
        .flat_map(|face| face.wires.iter().flatten())
        .find(|edge| edge.start.vertex == vertex)
        .expect("a corner of the input a piece names is a corner of its own solid")
        .start
}

/// The three planes `surfaces` are, or `None` when they are not three planes.
fn three_planes(surfaces: &[Surface]) -> Option<[Plane; 3]> {
    let [first, second, third] = surfaces else {
        return None;
    };
    Some([*first.as_plane()?, *second.as_plane()?, *third.as_plane()?])
}

/// Comes before [`by_a_side_from_an_edge_it_starts_on`]: a flat piece that
/// lies on the plane of a flat face `G` of a solid, with a straight side along
/// an edge `e` of `G`, placed against that solid by which side of the plane
/// across `e` its material is on (`docs/decisions.md` §365.8, Q1606). Where it
/// places nothing, [`by_a_side_from_an_edge_it_starts_on`] is tried.
///
/// **Why an edge can place a piece.** `P` is the piece, on `G`'s plane, `S` the
/// solid, and `H` the flat face across `e`. `s` is a side of `P` carried by
/// `G`'s plane and `H`'s, so it runs along the line `e` runs along, and both
/// its ends lie on `e` and are not one point. Take `m`, the middle of `s`: it
/// is strictly inside `e`, so near `m` the skin of `S` is `G` and `H` and
/// nothing else, `G` a half-plane on one side of `H`'s plane. Near `m` the
/// inside of `P` is the half-plane of `G`'s plane on its material side of
/// `H`'s plane. `P` was cut by every surface of `S`, so its inside lies all
/// inside `S`, all outside it, or all on its skin (§17.4), and it is enough to
/// know where it is near `m`.
///
/// - On `G`'s side of `H`'s plane, the inside of `P` near `m` lies on `G`, so
///   `P` lies on `S`'s skin, on `G`, and faces `G`'s way or the other.
/// - On the other side, it lies in `G`'s plane beyond `e`, and so in front of
///   `H`'s outward plane where the solid's corner at `e` is convex and behind
///   it where it is reflex: across a convex edge the solid is on `G`'s side of
///   `H`'s plane, and across a reflex one `H` faces towards `G`. Behind is
///   inside and in front is outside, with no need to ask which kind of edge it
///   is.
///
/// **`G`'s side of `H`'s plane is read off the way `G`'s wire walks `e`**, with
/// the face on its left about `G`'s own normal ([`BakedFace::wires`]). Left of
/// a walk along `n_G × n_H` is the negative side of `H`'s plane, since
/// `n_H · (n_G × (n_G × n_H)) = −‖n_G × n_H‖²`, as for
/// [`FaceMeet::material_side`]. Every question here is
/// [`which_meet_comes_first_along_a_line`] along that line, each point asked
/// as the meet of three planes, or two planes compared bit for bit, so every
/// answer is exact and nothing goes into the log. An end of `s` or of `e` that
/// is not the meet of `G`'s plane, `H`'s and one plane more, a question that
/// comes back undecided, or a plane that does not match leaves the side out.
///
/// Inside one solid of the group is inside. Lying on one or more with none
/// having it inside is read as [`LyingOn`] reads it, and outside every one is
/// outside. A solid no side of the piece lies along an edge of leaves the
/// piece unplaced, and the caller refuses under the name the point search ran
/// out under.
///
/// The piece this was built for is a sliver of a frame's wall `y = 1`, between
/// a comb's wall `x = 2` and the frame's own leaning wall a rounding past it,
/// along the edge where the comb's tooth top `y = 1` meets that wall. It holds
/// no `f64` point, and no plane of the comb has it in front, since it lies on
/// the tooth top (`tests/a_sliver_along_an_edge_of_a_face_it_lies_on.rs`).
fn by_an_edge_of_a_face_it_lies_on(
    solids: &[&BakedSolid],
    against: &[usize],
    mine: usize,
    piece: &Piece,
) -> Option<Placement> {
    let flat = *piece.surface.as_plane()?;
    let mut inside_one = false;
    let mut placed_against_every_one = true;
    let mut lying_on = LyingOn::default();
    for &index in against {
        match beside_an_edge_of(solids, index, solids[mine], piece, flat) {
            Some(Placement::Inside) => inside_one = true,
            Some(Placement::Outside) => (),
            Some(Placement::On { solid, same_way }) => lying_on.add(solid, same_way),
            None => placed_against_every_one = false,
        }
    }
    if inside_one {
        Some(Placement::Inside)
    } else {
        placed_against_every_one.then(|| lying_on.placement())
    }
}

/// Where `piece`, on the plane `flat`, sits against solid number `index` by
/// [`by_an_edge_of_a_face_it_lies_on`]'s rule, read off the first side of it
/// along an edge of a face of that solid on `flat`, or `None` where no side is.
fn beside_an_edge_of(
    solids: &[&BakedSolid],
    index: usize,
    own: &BakedSolid,
    piece: &Piece,
    flat: Plane,
) -> Option<Placement> {
    let solid = solids[index];
    solid
        .faces
        .iter()
        .filter(|face| {
            face.surface
                .as_plane()
                .is_some_and(|plane| is_one_plane_either_way(*plane, flat))
        })
        .find_map(|face| {
            face.wires.iter().flatten().find_map(|edge| {
                piece.wires.iter().flatten().find_map(|side| {
                    beside_one_edge(solid, index, own, piece, flat, face, edge, side)
                })
            })
        })
}

/// Where `piece` sits against solid number `index` when its side `side` lies
/// along the edge `edge` of the face `face`, on whose plane the piece lies, or
/// `None` where it does not, as [`by_an_edge_of_a_face_it_lies_on`] asks it.
#[expect(
    clippy::too_many_arguments,
    reason = "the piece, the face, the edge and the side are each asked about"
)]
fn beside_one_edge(
    solid: &BakedSolid,
    index: usize,
    own: &BakedSolid,
    piece: &Piece,
    flat: Plane,
    face: &BakedFace,
    edge: &BakedEdge,
    side: &FaceMeet,
) -> Option<Placement> {
    let g = *face.surface.as_plane()?;
    let h = *edge.across.as_plane()?;
    if !is_straight(side) || !is_one_plane_either_way(*side.carried_by[0].as_plane()?, g) {
        return None;
    }
    let across = *side.carried_by[1].as_plane()?;
    if !is_one_plane_either_way(across, h) {
        return None;
    }
    let line = [g.coefficients(), across.coefficients()];
    let start = the_third(three_planes(&edge.start.around)?, g, h)?;
    let end = the_third(three_planes(&edge.end.around)?, g, h)?;
    let [from, to] = &side.ends;
    let from = the_third(the_planes_meeting_at(from, own, flat)?, g, h)?;
    let to = the_third(the_planes_meeting_at(to, own, flat)?, g, h)?;
    let order = |first, second| which_meet_comes_first_along_a_line(line, first, second).ok();
    let along = order(start, end)?;
    let towards_g = match along {
        Decision::Pos => Decision::Neg,
        Decision::Neg => Decision::Pos,
        Decision::Zero | Decision::Undecidable { .. } | _ => return None,
    };
    let is_on_the_edge = |point| {
        [order(start, point), order(point, end)]
            .iter()
            .all(|step| *step == Some(along) || *step == Some(Decision::Zero))
    };
    let has_length = matches!(order(from, to)?, Decision::Pos | Decision::Neg);
    if !has_length || !is_on_the_edge(from) || !is_on_the_edge(to) {
        return None;
    }
    if side.material_side == towards_g {
        let mine = outward_plane_of(piece)?;
        let theirs = outward_plane_of_face(face)?;
        let same_way = if mine == theirs {
            true
        } else if mine == theirs.reversed() {
            false
        } else {
            return None;
        };
        return Some(Placement::On {
            solid: index,
            same_way,
        });
    }
    let behind = the_inner_side_of(the_face_across(solid, edge, h)?, across)?;
    Some(if side.material_side == behind {
        Placement::Inside
    } else {
        Placement::Outside
    })
}

/// The rule after [`by_an_edge_of_a_face_it_lies_on`]: a flat piece with a
/// straight side along the plane of a flat face `G` of a solid, one end of
/// that side strictly inside an edge `e` of `G`, placed against that solid by
/// where that side runs from that end (`docs/decisions.md` §365.8, Q1608,
/// Q1609).
///
/// **Why the start of a side can place a piece.** `P` is the piece, on a plane
/// that is neither `G`'s nor that of `H`, the flat face across `e`, and `S` is
/// the solid. `s` is a side of `P` carried by `P`'s plane and `G`'s, and `c`,
/// one end of it, is where `P`'s plane crosses the line of `e`, strictly
/// between `e`'s two ends. So near `c` the skin of `S` is `G` and `H` and
/// nothing else, `G` a half-plane on one side of `H`'s plane. `P` was cut by
/// every surface of `S`, so its inside lies all inside `S`, all outside it, or
/// all on its skin (§17.4), and it is enough to know where it is near one
/// point. Take `m` on `s`, near `c`: it is on `G`'s plane, and off `H`'s,
/// since the far end of `s` is off it.
///
/// - Where `s` runs to `G`'s side of `H`'s plane, `m` is on `G` and on no other
///   face, so near `m` the skin is a disk of `G`'s plane with `S` behind it.
///   The inside of `P` near `m` is a half-disk off that plane, on its material
///   side of it, and so inside `S` when that is `G`'s inner side and outside
///   it when it is not.
/// - Where it runs to the other side, `m` is on `G`'s plane beyond `e`, off
///   the skin, and the inside of `P` near `m` is where `m` is. That is in
///   front of `H`'s outward plane where the solid's corner at `e` is convex
///   and behind it where it is reflex: across a convex edge the solid is on
///   `G`'s side of `H`'s plane, and across a reflex one `H` faces towards `G`.
///   So `m` is inside when it is behind `H`'s outward plane, with no need to
///   ask which kind of edge it is.
///
/// `P` is never on the skin of `S`, since near `m` it is off it.
///
/// `G`'s side of `H`'s plane is read off the way `G`'s wire walks `e`, as
/// [`by_an_edge_of_a_face_it_lies_on`] reads it. Every question here is
/// [`which_meet_comes_first_along_a_line`] along the line of `e`, or
/// [`side_of_a_plane_where_three_planes_meet`] for the far end of `s`, each
/// point asked as the meet of three planes, or two planes compared bit for
/// bit, so every answer is exact and nothing goes into the log. An end of `s`
/// or of `e` that is not such a meet, a question that comes back undecided or
/// as a tie, or a plane that does not match leaves the side out.
///
/// Inside one solid of the group is inside, and outside every one is outside,
/// as [`by_a_plane`] reads it. A solid no side of the piece starts inside an
/// edge of leaves the piece unplaced, and the caller refuses under the name
/// the point search ran out under.
///
/// The pieces this was built for are two slivers a rounding wide
/// (`tests/a_sliver_along_an_edge_of_a_face_it_lies_on.rs`). One is cut from a
/// leaning ell's floor `z = 1` where the frame's hole corner `(3, 2)` is. The
/// other is cut from a comb's floor `z = 1` by the other comb's gap floor
/// `y = 1`, its tooth wall `x − 2y = 2` and the first comb's own wall
/// `x + 2y = 6`, and its side along the tooth wall starts at `(4, 1, 1)`,
/// inside the edge where that wall meets the gap floor, and runs up the wall.
/// Every corner of each is on the plane of a face of the other solid, so no
/// plane has it in front, and no side of it lies along an edge.
fn by_a_side_from_an_edge_it_starts_on(
    solids: &[&BakedSolid],
    against: &[usize],
    mine: usize,
    piece: &Piece,
) -> Option<Placement> {
    let flat = *piece.surface.as_plane()?;
    let mut outside_every_one = true;
    for &index in against {
        match from_an_edge_of(solids[index], solids[mine], piece, flat) {
            Some(true) => return Some(Placement::Inside),
            Some(false) => (),
            None => outside_every_one = false,
        }
    }
    outside_every_one.then_some(Placement::Outside)
}

/// Whether `piece`, on the plane `flat`, is inside `solid` (`Some(true)`) or
/// outside it (`Some(false)`) by [`by_a_side_from_an_edge_it_starts_on`]'s
/// rule, read off the first side of it that starts inside an edge of a face of
/// `solid` and runs along that face's plane, or `None` where no side does.
fn from_an_edge_of(
    solid: &BakedSolid,
    own: &BakedSolid,
    piece: &Piece,
    flat: Plane,
) -> Option<bool> {
    solid.faces.iter().find_map(|face| {
        face.wires.iter().flatten().find_map(|edge| {
            piece
                .wires
                .iter()
                .flatten()
                .find_map(|side| from_one_edge(solid, own, flat, face, edge, side))
        })
    })
}

/// Whether the piece with the side `side`, on the plane `flat`, is inside
/// `solid` when that side runs along the plane of `face` from a point strictly
/// inside its edge `edge`, or `None` where it does not, as
/// [`by_a_side_from_an_edge_it_starts_on`] asks it.
fn from_one_edge(
    solid: &BakedSolid,
    own: &BakedSolid,
    flat: Plane,
    face: &BakedFace,
    edge: &BakedEdge,
    side: &FaceMeet,
) -> Option<bool> {
    let g = *face.surface.as_plane()?;
    let h = *edge.across.as_plane()?;
    if !is_straight(side) || !is_one_plane_either_way(*side.carried_by[0].as_plane()?, flat) {
        return None;
    }
    let cut_by = *side.carried_by[1].as_plane()?;
    if !is_one_plane_either_way(cut_by, g) {
        return None;
    }
    if [(flat, g), (flat, h), (g, h)]
        .iter()
        .any(|&(one, other)| is_one_plane_either_way(one, other))
    {
        return None;
    }
    let holds =
        |three: &[Plane; 3], plane| three.iter().any(|one| is_one_plane_either_way(*one, plane));
    let [first, second] = &side.ends;
    let ends = [
        the_planes_meeting_at(first, own, flat)?,
        the_planes_meeting_at(second, own, flat)?,
    ];
    let far = match ends.map(|three| holds(&three, g) && holds(&three, h)) {
        [true, false] => ends[1],
        [false, true] => ends[0],
        [true, true] | [false, false] => return None,
    };
    if !holds(&far, g) {
        return None;
    }
    let start = the_third(three_planes(&edge.start.around)?, g, h)?;
    let end = the_third(three_planes(&edge.end.around)?, g, h)?;
    let line = [g.coefficients(), h.coefficients()];
    let order = |first, second| which_meet_comes_first_along_a_line(line, first, second).ok();
    let along = order(start, end)?;
    let towards_g = match along {
        Decision::Pos => Decision::Neg,
        Decision::Neg => Decision::Pos,
        Decision::Zero | Decision::Undecidable { .. } | _ => return None,
    };
    let at = flat.coefficients();
    if order(start, at)? != along || order(at, end)? != along {
        return None;
    }
    let runs_to = side_of_a_plane_where_three_planes_meet(
        far.map(|plane| plane.coefficients()),
        h.coefficients(),
    )
    .ok()?;
    match runs_to {
        Decision::Pos | Decision::Neg => (),
        Decision::Zero | Decision::Undecidable { .. } | _ => return None,
    }
    if runs_to == towards_g {
        Some(side.material_side == the_inner_side_of(face, cut_by)?)
    } else {
        Some(runs_to == the_inner_side_of(the_face_across(solid, edge, h)?, h)?)
    }
}

/// The one plane of `three` that is neither `g` nor `h`, either way round, as
/// coefficients, or `None` where there is not exactly one.
fn the_third(three: [Plane; 3], g: Plane, h: Plane) -> Option<[f64; 4]> {
    let mut rest = three
        .into_iter()
        .filter(|plane| !is_one_plane_either_way(*plane, g) && !is_one_plane_either_way(*plane, h));
    let third = rest.next()?;
    rest.next().is_none().then(|| third.coefficients())
}

/// The face of `solid` on the plane `h`, stored that way round, that shares an
/// input edge with `edge`: the face across it.
fn the_face_across<'a>(solid: &'a BakedSolid, edge: &BakedEdge, h: Plane) -> Option<&'a BakedFace> {
    solid.faces.iter().find(|other| {
        other.surface.as_plane() == Some(&h)
            && other
                .wires
                .iter()
                .flatten()
                .any(|theirs| theirs.keys.iter().any(|key| edge.keys.contains(key)))
    })
}

/// Which side of `plane` is inside the solid next to `face`, when `plane` is
/// that face's plane facing either way, or `None` when it is not.
fn the_inner_side_of(face: &BakedFace, plane: Plane) -> Option<Decision> {
    let outward = outward_plane_of_face(face)?;
    if plane == outward {
        Some(Decision::Neg)
    } else if plane == outward.reversed() {
        Some(Decision::Pos)
    } else {
        None
    }
}

/// Whether `one` and `other` are one plane, bit for bit, facing either way.
fn is_one_plane_either_way(one: Plane, other: Plane) -> bool {
    one == other || one == other.reversed()
}

/// Comes before [`by_a_meridian_disk_beside_it`]: a piece, flat or curved,
/// that nothing before placed takes the placement of the piece of its own
/// face across one of its sides, where the start of that side, or the middle
/// of a straight side, is clear of every face of every solid it is placed
/// against, when the point search places that piece inside or outside
/// (`docs/decisions.md` §35.6, Q1537, Q2551).
///
/// **Why a neighbour can place a piece here.** `P` and `P′` are pieces of one
/// face that share a side `s`, and `m` is where `s` starts, or the middle of
/// `s` where `s` runs where two planes meet and so is the straight segment
/// between its ends, whose middle lies in the box halfway between theirs
/// ([`midway_between`]). Each was cut by
/// every surface of a solid `S`, so each lies all inside `S`, all outside it
/// or all on its skin (§17.4). Every neighbourhood of `m` meets the inside of
/// `P` and of `P′`. When `m` is off every face of `S`, it is off `S`'s skin,
/// so a neighbourhood of it lies all inside `S` or all outside it, and the two
/// are placed alike, neither of them on the skin. So a `P′` the search puts on
/// the skin places nothing.
///
/// **`m` is a box, and every point of it has to be off every face.** A face
/// whose surface puts the whole box strictly on one side is off it
/// ([`is_strictly_off_the_surface_of`]). A flat face whose one wire is a
/// meridian circle of the ring across it is off the box when the half-space
/// naming that circle puts the whole box behind it
/// ([`is_behind_the_meridian_disk`]). Otherwise the face is asked whether it
/// holds the box ([`SolidView::what_this_face_holds`]), which takes the box as
/// on its surface and answers [`OnAFace::NotHeld`] only when no point of it
/// that is on the surface is on the face. Unlike
/// [`by_the_piece_across_a_side`], this does not need `S`'s faces flat: it asks
/// about one point and not about a whole side. What the faces and the search
/// asked the policy goes into the log only when the piece is placed.
///
/// The pieces this was built for are the bands of two quarter bends placed
/// half a turn apart by a float turn: each bend's cap plane cuts the other
/// bend's wall a rounding round from that bend's own cap rim, half a ring from
/// the first bend, and the band between holds no `f64` point. The cut starts
/// on the other bend's cap plane and on its ring, half a ring from that cap.
/// [`SolidView::what_this_face_holds`] reads the cap's rim against the ring
/// first and is undecided there, at `side of torus`, and the half-space naming
/// the rim puts the start off the cap
/// (`tests/a_piece_no_ray_can_place_names_its_refusal.rs`).
///
/// Placed round by `0.3` or `−2` and half a turn on, or by `0.3` and a quarter
/// turn on, the same kind of band is not placed. One bend's start cap plane
/// cuts the other bend's wall, and the box round the cut's start straddles
/// that plane. Asked whether that bend's wall holds the start,
/// [`SolidView::what_this_face_holds`] gives up on that plane, undecided,
/// before it asks the end cap's plane, which puts the start strictly off the
/// wall (Q1948, Q2056). Placed round by `0` and half a turn on, that plane is
/// `y = 0`, the start is on it, and the end cap's plane is asked next.
///
/// It placed a second kind of piece too, with no rounding in it, until the
/// point search learned to: a thick bar's cap where a thin bar stands inside
/// and touches the thick bar's wall along its seam. The thin bar's pipe cuts
/// the cap's plane in a circle that touches the rim at the seam corner, so the
/// piece between them has two sides, each from that corner back to it, and the
/// point search found no point inside it. The corner is on the thin bar's pipe
/// but past its caps, so clear of every face of the thin bar (Q1563,
/// `tests/fuses_that_touch.rs`). Since Q1748 [`between_the_round_wires`]
/// offers a point inside that piece and [`in_the_bulge_of`] holds it, so this
/// rule is no longer asked about it. Switched off, it costs no fuse in
/// `tests/fuses_that_touch.rs`, but tests elsewhere in this crate still fail
/// without it, the bends' among them.
///
/// The middle is asked for a sliver of a hollow quarter cone's cap at `z = 4`,
/// between its own side plane and another quarter's an ulp off it, from the
/// bore to the cone, the other quarter starting at `z = 4.5`. Each end of the
/// side along the other's plane lies on the line through a side of the other's
/// face on that plane, and that face cannot say it does not hold it; the
/// middle is on no such line, and the rest of the cap across that side is
/// placed outside (`tests/two_quarters_of_a_hollow_cone_flush_along_a_plane.rs`).
/// No face of the other quarter reaches `z = 4`, so the sliver shares no side
/// with a piece of it and [`SidesItShares`] cannot place it: without the
/// middle it is refused `a point inside a piece of a face` again (Q2562).
fn by_the_piece_across_a_side_clear_of_every_face(
    model: &Model,
    scratch: &Scratch,
    solids: &[&BakedSolid],
    against: &[usize],
    piece: usize,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Option<Placement> {
    let theirs: Vec<SolidView<'_>> = against
        .iter()
        .map(|&index| model.solid_at(solids[index].key.slot_in(model.id())))
        .collect();
    scratch.pieces[piece]
        .wires
        .iter()
        .flatten()
        .find_map(|side| {
            let across = &scratch.pieces[the_piece_across(scratch, piece, side)?];
            let start = side.ends[0].point();
            let middle = is_straight(side).then(|| midway_between(start, side.ends[1].point()));
            let mut trial = [Some(start), middle]
                .into_iter()
                .flatten()
                .find_map(|on_it| {
                    let mut asked = ProvenanceLog::new();
                    theirs
                        .iter()
                        .all(|&solid| is_clear_of_every_face_of(solid, on_it, policy, &mut asked))
                        .then_some(asked)
                })?;
            let in_plane = match across.surface.as_plane() {
                Some(flat) => Some(InPlane::of(flat)?),
                None => None,
            };
            let candidates = candidate_points(across, &corners_of(across));
            let search = place_the_piece(
                model,
                solids,
                against,
                &candidates,
                across,
                in_plane.as_ref(),
                policy,
                &mut trial,
            );
            let Ok(Search::Placed(placement @ (Placement::Inside | Placement::Outside))) = search
            else {
                return None;
            };
            trial.drain_into(log);
            Some(placement)
        })
}

/// Whether no face of `solid` holds any point of `point`'s box, each face
/// asked first by its surface ([`is_strictly_off_the_surface_of`]), then,
/// for a meridian disk, by the half-space naming its circle
/// ([`is_behind_the_meridian_disk`]), and then by its trimming.
fn is_clear_of_every_face_of(
    solid: SolidView<'_>,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> bool {
    solid.faces().all(|face| {
        is_strictly_off_the_surface_of(face, point)
            || is_behind_the_meridian_disk(solid, face, point, policy, log)
            || matches!(
                solid.what_this_face_holds(face, point, policy, log),
                Ok(OnAFace::NotHeld)
            )
    })
}

/// Whether `face`'s stored surface puts `point`'s whole box strictly on one
/// side of it.
fn is_strictly_off_the_surface_of(face: FaceView<'_>, point: PredicatePoint) -> bool {
    let surface = face.surface();
    let side = surface.as_plane().map_or_else(
        || side_of_surface(&surface, point),
        |flat| side_of_a_flat_face(flat, point),
    );
    matches!(side, Decision::Pos | Decision::Neg)
}

/// Whether `face` is the disk of one meridian circle of a ring, its only wire
/// that circle and the face across it the ring, and the half-space naming that
/// circle puts every point of `point`'s box behind it.
///
/// **Such a disk holds no point behind that half-space.** [`intersect`] names a
/// meridian circle by a half-space only once it has proved the half-space's
/// plane clears both circles a plane through the ring's axis cuts it in, so the
/// circle, and the disk inside it, lie in front of that plane
/// (`holds.rs`'s [`the_meridian_half_space_keeps`]). This asks no sign of the
/// ring, which [`SolidView::what_this_face_holds`] reads first, so a box on
/// the ring a long way from the disk is still put off it.
///
/// The circle is named where the face is stored, from the stored plane, ring
/// and corner, and the half-space moved into place, as `holds.rs` names it,
/// for its reason: a turn rounds the plane and the ring each on its own.
fn is_behind_the_meridian_disk(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> bool {
    let mut wires = face.wires();
    let (Some(wire), None) = (wires.next(), wires.next()) else {
        return false;
    };
    let Some(circle) = wire.coedges().next().filter(|_| wire.closes_on_one_edge()) else {
        return false;
    };
    let Ok(Some(across)) = crate::holds::the_face_across(solid, face, circle.edge()) else {
        return false;
    };
    let (flat, ring) = (face.stored_surface(), across.stored_surface());
    if flat.as_plane().is_none()
        || !matches!(ring, Surface::Torus(_))
        || across.location() != face.location()
    {
        return false;
    }
    let corner = PredicatePoint::from(circle.start().stored_point().coordinates());
    let Ok(Some(this_circle)) =
        the_half_space_naming_the_meridian_circle_through(&flat, &ring, corner, policy, log)
    else {
        return false;
    };
    matches!(
        the_meridian_half_space_keeps(&this_circle.transformed(&face.location()), point),
        Ok(false)
    )
}

/// Whether a flat piece inside a ring, on a plane through the ring's axis, is
/// inside or outside a group of solids each made of that ring and of planes
/// through its axis, read off a meridian disk of each
/// ([`is_inside_by_a_meridian_disk`]). Inside one solid of the group is
/// inside, outside every one is outside, and anything else is `None`, as
/// [`by_a_corner`] folds it.
///
/// The piece this was built for is a quarter bend's end cap disk, against a
/// copy of the bend placed a quarter turn on round the ring. Placed round by
/// `0.3` and `0.3 + π/2`, the end cap's plane and the copy's start cap plane
/// differ by an ulp, so the disk lies strictly in front of the copy's start
/// cap, a rounding off it, and no point found inside it is placed against the
/// copy. Its one side is its rim, along its own bend's wall, and no piece lies
/// across that, so no rule before this one places it
/// (`tests/a_piece_no_ray_can_place_names_its_refusal.rs`, Q2056).
///
/// Nothing here asks the policy: every question is settled exactly or left
/// unsettled, so nothing goes into the log.
fn by_a_meridian_disk_beside_it(
    solids: &[&BakedSolid],
    against: &[usize],
    piece: &Piece,
) -> Option<Placement> {
    let flat = piece.surface.as_plane()?;
    let mut inside_one = false;
    let mut outside_every_one = true;
    for &index in against {
        match is_inside_by_a_meridian_disk(solids[index], flat, piece) {
            Some(true) => inside_one = true,
            Some(false) => (),
            None => outside_every_one = false,
        }
    }
    if inside_one {
        Some(Placement::Inside)
    } else {
        outside_every_one.then_some(Placement::Outside)
    }
}

/// Whether `piece`, flat on `flat`, is inside `solid`, or `None` where this
/// cannot say. It says only where all of these hold, each asked of the numbers
/// as stored:
///
/// 1. `flat` holds the axis of a ring `T` ([`how_the_axis_lies_on`]), and the
///    piece has a side along `T` with its material inside the tube, on the
///    negative side of the ring.
/// 2. Every face of `solid` lies on `T` itself or on a plane holding its axis.
/// 3. One flat face `G` of `solid` is a whole meridian disk: its one wire is
///    one edge right round a circle, with `T` across it. Its outward plane is
///    `σ`.
/// 4. The plane of every other flat face of `solid` has the piece's half of
///    `flat` and `G`'s half of `σ` strictly on one side of it.
/// 5. The piece's half of `flat` is strictly in front of `σ`, and then the
///    piece is outside, or strictly behind it, and then it is inside.
///
/// **Why that places it.** Take a point `x` inside the piece near its side
/// along `T`, so in that side's half of `flat`. The piece's material is inside
/// the tube, so `x` is, and off the axis. The circle `C` round the axis
/// through `x` keeps `x`'s distance from the axis and its height, so it stays
/// inside the tube and meets no face on `T`. Let `y` be where `C` meets `G`'s
/// half of `σ`: inside the tube, so inside the disk `G`. Go round `C` from `y`
/// to `x` on `x`'s side of `σ`, less than half a turn. No point of that arc
/// past `y` is on `σ`, and a plane through the axis meets it only where the
/// plane parts `y` from `x`, which the fourth check rules out. So the arc
/// meets no face of `solid` past `y`, and near `y` the solid's skin is `G`
/// alone, since `y` is off every other face's plane. `x` is then where a point
/// just past `G` on `x`'s side of `σ` is: outside in front of `G` and inside
/// behind it. The piece was cut by every surface of `solid`, so it is placed
/// as `x` is (§17.4).
///
/// A plane that holds either half gives nought in the fourth check, and this
/// says nothing then: a second face on `σ` is one such plane.
fn is_inside_by_a_meridian_disk(solid: &BakedSolid, flat: &Plane, piece: &Piece) -> Option<bool> {
    let ring = solid.faces.iter().find_map(|face| {
        if let Surface::Torus(ring) = face.surface {
            Some(ring)
        } else {
            None
        }
    })?;
    let on_the_ring_or_through_its_axis = |surface: &Surface| {
        surface
            .as_plane()
            .map_or(*surface == Surface::Torus(ring), |cut| {
                how_the_axis_lies_on(cut, &ring) == Decision::Zero
            })
    };
    if how_the_axis_lies_on(flat, &ring) != Decision::Zero
        || !solid
            .faces
            .iter()
            .all(|face| on_the_ring_or_through_its_axis(&face.surface))
    {
        return None;
    }
    let along_the_ring = piece.wires.iter().flatten().find(|side| {
        side.carried_by[1] == Surface::Torus(ring) && side.material_side == Decision::Neg
    })?;
    let the_piece = HalfOfAPlaneThroughAnAxis::of(&ring, flat, along_the_ring.ends[0].point())?;
    solid.faces.iter().enumerate().find_map(|(disk, face)| {
        let outward = outward_plane_of_face(face)?;
        let rim = the_corner_of_a_whole_meridian_disk(face, ring)?;
        let the_disk = HalfOfAPlaneThroughAnAxis::of(&ring, &outward, rim)?;
        let both_on_one_side_of_every_other_plane = solid
            .faces
            .iter()
            .enumerate()
            .filter(|&(other, _)| other != disk)
            .filter_map(|(_, other)| other.surface.as_plane())
            .all(|cut| {
                let in_front = the_piece.is_in_front_of(cut);
                in_front.is_some() && in_front == the_disk.is_in_front_of(cut)
            });
        if !both_on_one_side_of_every_other_plane {
            return None;
        }
        the_piece.is_in_front_of(&outward).map(|in_front| !in_front)
    })
}

/// The corner of `face`'s one wire when that wire is one edge right round a
/// circle, with `ring` across it: a whole meridian disk of `ring` when the face
/// is flat on a plane through its axis.
fn the_corner_of_a_whole_meridian_disk(face: &BakedFace, ring: Torus) -> Option<PredicatePoint> {
    let [wire] = face.wires.as_slice() else {
        return None;
    };
    let [rim] = wire.as_slice() else {
        return None;
    };
    (rim.start.vertex == rim.end.vertex && rim.across == Surface::Torus(ring))
        .then_some(rim.start.point)
}

/// One half of a plane through a ring's axis, the plane cut along the axis.
///
/// Read as `arrange.rs`'s `side_of_a_plane_holding_a_rings_axis` reads it. A
/// point of the plane `P` is `c + a·u + b·w` for the ring's centre `c`, its
/// axis `w` and `u` along `w × n_P`, so which half it is in is the sign of
/// `a`, which is that of `det[p − c, w, n_P]`. On a circle the plane cuts the
/// ring in, `a` is `±R + r·cos φ`, at least `R − r` from nought, so the box
/// round a corner there tells it. Another plane `Q` through the axis is nought
/// along it, so its sign over the half is that of `a·(n_Q·u)`, and `n_Q·u` has
/// the sign of `det[n_Q, w, n_P]`. `orient3d` with one point at the origin
/// gives each of the two signs turned round the same way, so their product is
/// the side.
struct HalfOfAPlaneThroughAnAxis {
    axis: PredicatePoint,
    normal: PredicatePoint,
    /// Whether `det[p − c, w, n_P]`, as `orient3d` turns it, is positive.
    positive: bool,
}

impl HalfOfAPlaneThroughAnAxis {
    /// The half of `flat`, a plane holding `ring`'s axis, that `point` is in,
    /// or `None` where its box does not say.
    fn of(ring: &Torus, flat: &Plane, point: PredicatePoint) -> Option<Self> {
        let axis = PredicatePoint::from(ring.axis().direction().components());
        let normal = PredicatePoint::from(flat.normal().components());
        let (centre, box_) = (ring.centre().coordinates(), point.enclosure());
        let from_the_centre = PredicatePoint::Constructed(core::array::from_fn(|index| {
            box_[index] - Interval::exactly(centre[index])
        }));
        let origin = PredicatePoint::from([0.0; 3]);
        let positive =
            strictly_positive(orient3d_of_points(origin, from_the_centre, axis, normal))?;
        Some(Self {
            axis,
            normal,
            positive,
        })
    }

    /// Whether this half lies strictly on the positive side of `cut`, a plane
    /// holding the same axis, or `None` where `cut` holds it, either way
    /// round, or the sign is not settled.
    fn is_in_front_of(&self, cut: &Plane) -> Option<bool> {
        let origin = PredicatePoint::from([0.0; 3]);
        let across = orient3d_of_points(
            origin,
            cut.normal().components().into(),
            self.axis,
            self.normal,
        );
        Some(strictly_positive(across)? == self.positive)
    }
}

/// Whether `sign` is strictly positive, or `None` where it is nought or not
/// settled.
fn strictly_positive(sign: Decision) -> Option<bool> {
    (sign == Decision::Pos || sign == Decision::Neg).then_some(sign == Decision::Pos)
}

/// The last rule: a curved piece placed by a point one reach from the middle
/// of its corners towards each corner of the box round that middle, put back
/// onto its surface (`docs/decisions.md` §35.6, Q2116).
///
/// **The six steps [`candidate_points`] takes along the axes are where a box
/// touches the ball inscribed in it.** A cube from `(3, 3, 0)` to `(7, 7, 4)`
/// and the ball of radius 2 at `(5, 5, 2)` touch at the middle of each of the
/// cube's six faces. The ball's one face has its two poles for corners, so
/// their middle is the ball's centre, and each step of one radius from there,
/// put back onto the ball, is one of those six points: on the cube's skin,
/// where the ray places nothing about a face that does not lie on it. Its
/// corners are on that skin too, and it shares no side with any piece, so
/// every fuse of the two was refused at [`PLACED_AGAINST_EVERY_SOLID`]
/// (`tests/a_ball_inscribed_in_a_cube.rs`). The step towards `(1, 1, 1)`
/// comes back as `(5 + 2/√3, 5 + 2/√3, 2 + 2/√3)`, inside the cube.
///
/// Nothing is trusted: each point is proved inside the piece and placed by the
/// ray as a candidate is, and what that asked goes into the log only when the
/// piece is placed. A point the ray cannot place is dropped whatever the
/// reason, a face nobody follows included, so a piece none of these places is
/// refused as it was. Tried after every other rule, so a piece any of them
/// placed is placed as before.
fn by_a_step_towards_a_corner_of_the_box_round_it(
    model: &Model,
    solids: &[&BakedSolid],
    against: &[usize],
    piece: &Piece,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Option<Placement> {
    if piece.surface.as_plane().is_some() {
        return None;
    }
    let middle = middle_of_the_corners(&corners_of(piece));
    let reach = how_big_the_surface_is(&piece.surface, middle);
    TOWARDS_THE_CORNERS_OF_A_BOX
        .into_iter()
        .filter_map(|[x, y, z]| {
            let step = middle + Vector::new(x, y, z) * reach;
            back_onto_the_surface(&piece.surface, PredicatePoint::from(step.coordinates()))
        })
        .find_map(|point| {
            if the_piece_holds(piece, None, &[], &[], point) != Some(true) {
                return None;
            }
            let mut trial = ProvenanceLog::new();
            let Ok(Settled::At(placement)) =
                settle_against_each(model, solids, against, point, piece, policy, &mut trial)
            else {
                return None;
            };
            trial.drain_into(log);
            Some(placement)
        })
}

/// The ways from the middle of a box to its eight corners, in a fixed order.
const TOWARDS_THE_CORNERS_OF_A_BOX: [[f64; 3]; 8] = [
    [1.0, 1.0, 1.0],
    [1.0, 1.0, -1.0],
    [1.0, -1.0, 1.0],
    [1.0, -1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, -1.0, -1.0],
];

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
/// already worked out and put on [`FaceMeet::material_side`]. Where two
/// surfaces cross in two closed curves that side holds more than the piece, and
/// the side also asks what picks its curve out ([`what_else_a_curved_side_asks`]).
///
/// The one thing a *flat* piece may not have any more is a polygon: the disk a
/// bar takes out of a block's face is bounded by one closed curve and has one
/// corner. Its wire contributes to the count exactly as a round hole does, which
/// is `holds.rs`'s rule read the same way round.
///
/// A round wire's partner's inside is the region it bounds only where the
/// partner leaves one region on the plane. A torus leaves two. On a plane
/// through its axis they are the two meridian disks, and `also_picked_by` — one
/// entry per wire, from [`what_else_each_round_wire_asks`] — says what else
/// must hold the point, or `None` where nothing can say. An arc of one of
/// those two circles is read the same way: `meridians`, one entry per side of
/// every wire, from [`the_meridian_circle_each_side_runs_round`], says which
/// disk its bulge is in ([`in_the_bulge_of`]).
///
/// **On a plane square to a ring's axis what a whole circle bounds is read
/// from which of the two circles it is.** A point inside the ring is in the
/// band between them, so inside what the outer circle bounds and outside what
/// the inner one does ([`runs_round_the_outer_circle`]); no picker says that,
/// and asked for one the band could not say what it held
/// (`a_band_on_a_plane_square_to_a_bends_axis_holds_its_own_points`). A point
/// outside the ring can still be inside what either bounds, since both go
/// round the inner disk, which is outside the ring ([`in_the_inner_disk_of`]).
///
/// **A flat piece's curved partners are asked about the point and a point of
/// the plane beside it at once**, through [`side_of_a_partner_by_its_plane`]. A
/// candidate can be a rounding off the plane, and where a ball only just cuts
/// the plane it is wider a rounding off it than on it by far more than a
/// rounding: a point inside the ball beside a disk `1e-8` wide can have its
/// foot outside the disk, and asked about the point alone the disk held it
/// (`docs/decisions.md` §346.6).
///
/// **So are its corners and the chord across each bulge, over the same box.**
/// [`InPlane`] reads a point off its plane where the line from its reference
/// point through it meets the plane, and that is `|f − o|·h/(d − h)` from the
/// foot `f`: `h` the point's height, `d` the reference point's, at most
/// `max(|offset|, 1)`, and `o` the plane's point nearest the origin. On a plane
/// through the origin, `2²⁰` out, a point one spacing high is read `8.6e-5`
/// from its foot, and a half disk held a point whose foot was short of its
/// chord
/// (`a_point_a_rounding_off_a_far_half_disk_whose_foot_is_short_of_its_chord_is_not_held`).
/// A turn settled over the box is the same for the point and for a point of
/// the plane beside it, which the plane reads where it is.
///
/// **The corners are stretched to the plane too.** A corner is a box, and one
/// the bake solved from three planes that leave this one out can lie wholly off
/// it: a face put through a pyramid's apex `1e-8` beside where the other three
/// planes there meet is one the model takes, and `2²⁰` out the formula above
/// reads that apex about `1.5e-2` from where the face meets its two
/// neighbours. Wound round the corners as they were, the fan held points off
/// the face and missed points on it, `1e-3` and more from every edge
/// (`a_far_face_a_tolerance_off_its_apex_holds_no_point_its_neighbours_put_off_it`,
/// row Q1251). Stretched, a corner's box holds a point of the plane beside it
/// as well, so a turn settled over it is the turn about where the corner is,
/// and a point between the two readings is left undecided.
fn the_piece_holds(
    piece: &Piece,
    in_plane: Option<&InPlane>,
    also_picked_by: &[Option<Vec<Picker>>],
    meridians: &[Vec<Option<Picker>>],
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
        if let Surface::Cylinder(pipe) = &piece.surface
            && piece
                .wires
                .iter()
                .flatten()
                .any(|side| a_plane_rests_along_a_pipe(&piece.surface, &side.carried_by[1]))
        {
            return counted_along_the_ruling(piece, pipe, point);
        }
        if let Surface::Cone(cone) = &piece.surface
            && !is_on_the_nappe_the_piece_is_on(piece, cone, point)?
        {
            return Some(false);
        }
        for side in piece.wires.iter().flatten() {
            let found = match side.carried_by[1].as_plane() {
                Some(flat) => side_of_a_flat_face(flat, point),
                None => side_of_surface(&side.carried_by[1], point),
            };
            if found != side.material_side {
                return Some(false);
            }
            for picker in what_else_a_curved_side_asks(piece, side) {
                match picker.side_of_point(point) {
                    Decision::Pos => (),
                    Decision::Neg | Decision::Zero => return Some(false),
                    Decision::Undecidable { .. } | _ => return None,
                }
            }
        }
        return Some(true);
    };
    // **A flat piece's corners are a polygon and its sides need not be**
    // (§38.5). Where a bar's wall cuts a plane the side between two corners is an
    // arc, and the chord across it is on the other side of the crossing: the
    // midway point of the chord is inside the bar where the piece is outside it,
    // so a winding count over the corners alone hands classification a point the
    // piece does not hold. What rules it out is the sign the arrangement already
    // wrote down — the side of the partner this piece's material is on — read
    // here for a flat piece exactly as the curved branch above reads it.
    let flat = piece
        .surface
        .as_plane()
        .expect("a piece measured from inside a plane is a flat piece");
    let reaching_the_plane = stretched_to_the_plane(flat, point)?;
    for side in piece.wires.iter().flatten() {
        if side.carried_by[1].as_plane().is_some() {
            continue;
        }
        match side_of_a_partner_by_its_plane(flat, &side.carried_by[1], point, reaching_the_plane) {
            found if found == side.material_side => (),
            Decision::Neg | Decision::Pos | Decision::Zero => return Some(false),
            Decision::Undecidable { .. } | _ => return None,
        }
    }
    let mut winding = 0;
    let mut one_or_nothing = 0;
    for (position, wire) in piece.wires.iter().enumerate() {
        match round_wire_of(wire) {
            Some(partner) => {
                let inwards = if what_a_round_wire_bounds_holds_the_material(flat, &partner, wire)?
                {
                    1
                } else {
                    -1
                };
                match side_of_a_partner_by_its_plane(flat, &partner, point, reaching_the_plane) {
                    Decision::Neg => {
                        let in_its_disk = if let Surface::Torus(ring) = partner
                            && is_square_to_the_axis(flat, &ring)
                        {
                            runs_round_the_outer_circle(flat, &ring, &wire[0])?
                        } else {
                            let pickers = also_picked_by[position].as_deref()?;
                            every_picker_keeps(pickers, reaching_the_plane)?
                        };
                        if in_its_disk {
                            winding += inwards;
                        }
                    }
                    Decision::Pos => {
                        if in_the_inner_disk_of(flat, &partner, reaching_the_plane)? {
                            winding += inwards;
                        }
                    }
                    Decision::Zero | Decision::Undecidable { .. } | _ => return None,
                }
            }
            None => {
                let corners = wire
                    .iter()
                    .map(|side| stretched_to_the_plane(flat, side.ends[0].point()))
                    .collect::<Option<Vec<PredicatePoint>>>()?;
                winding +=
                    in_plane.winding_around(core::slice::from_ref(&corners), reaching_the_plane)?;
                for (index, side) in wire.iter().enumerate() {
                    match in_the_bulge_of(
                        flat,
                        side,
                        wire,
                        meridians[position][index].as_ref(),
                        in_plane,
                        reaching_the_plane,
                    )? {
                        Bulge::Counts(count) => winding += count,
                        Bulge::OneOrNothing => one_or_nothing += 1,
                    }
                }
            }
        }
    }
    // The wires wind once round each point of the piece and not at all round
    // any other. Counted with every unsettled bulge as nothing, a one cannot
    // really be a two, so the point is held; a count the unsettled bulges could
    // still lift to one says nothing.
    if winding == 1 {
        Some(true)
    } else if (winding..=winding + one_or_nothing).contains(&1) {
        None
    } else {
        Some(false)
    }
}

/// Whether a piece of `pipe` whose sides are rulings, arcs of planes across its
/// axis and whole curves such planes cut holds `point`, counted along the
/// ruling through it: how many arcs and whole curves that ruling meets above
/// `point`, odd being held. `None` for a piece with a side of any other kind,
/// and where a sign cannot be settled.
///
/// It is `holds.rs`'s count along a ruling for a solid's face
/// (`how_often_the_ruling_above_a_point_is_crossed`), read off a piece's sides.
/// The piece is bounded and the ruling runs on up the pipe for ever, so it
/// leaves the piece for good, and a plane across the axis meets each ruling
/// once and crosses it there.
///
/// **Which rulings an arc meets is read off its chord.** Seen down the axis
/// the arc is an arc of the pipe's circle, and `point` is on that circle, so
/// the ruling meets the arc when `point` is on the arc's side of the chord, as
/// `holds.rs`'s `whether_the_ruling_meets` reads it. Which side that is comes
/// from which way round the axis the side runs from its first end to its
/// second, and that is one sign: a side whose material is on its partner `P`'s
/// negative side runs along `∇S × ∇P` (§36.6). The pipe's gradient `r` is
/// square to its axis `w`, so `w · (r × (r × n)) = −(w·n)|r|²` for `P`'s normal
/// `n`, at every point of the arc: the side runs counterclockwise round `w`
/// where its material is on `P`'s negative side and `w·n` is negative, or on
/// its positive side and `w·n` positive, and clockwise otherwise. A
/// counterclockwise side keeps the sign `Neg` against its chord.
///
/// **A whole curve meets every ruling once**, since seen down the axis it goes
/// right round the circle, and it has no chord to read. A bar standing on a
/// block's top flush with the block's side has its wall cut along the ruling
/// where that side's plane rests on it, from the foot's circle to the top's,
/// and both circles are whole: read off a chord of no length, every point of
/// the wall was left undecided and every rule refused
/// (`a_bar_standing_on_a_blocks_top_flush_with_its_edge_answers_every_rule`).
///
/// **A ruling side is not counted.** The ruling through `point` meets it only
/// where `point` is on it, and then `point` is on the ruling through a corner
/// of each arc the side ends at, where that arc's chord reads nothing and this
/// says nothing; so does `point` on a corner's ruling anywhere else, and on an
/// arc.
fn counted_along_the_ruling(piece: &Piece, pipe: &Cylinder, point: PredicatePoint) -> Option<bool> {
    let up = exactly(pipe.axis().direction().components());
    let at = point.enclosure();
    let mut above = 0_usize;
    for side in piece.wires.iter().flatten() {
        let flat = side.carried_by[1].as_plane()?;
        let leans = dot(exactly(flat.normal().components()), up).sign();
        if leans == Decision::Zero {
            continue;
        }
        if !matches!(leans, Decision::Pos | Decision::Neg) {
            return None;
        }
        if !closes_on_itself(side) && !the_ruling_meets_the_arc(side, leans, up, at)? {
            continue;
        }
        let beyond = side_of_a_flat_face(flat, point);
        if !matches!(beyond, Decision::Pos | Decision::Neg) {
            return None;
        }
        if beyond != leans {
            above += 1;
        }
    }
    Some(above % 2 == 1)
}

/// Whether the ruling through `at` meets the arc `side` runs along, a plane
/// leaning `leans` against the axis `up` being its partner: whether `at` is on
/// the arc's side of its chord seen down the axis, as
/// [`counted_along_the_ruling`] sets out. `None` where that sign cannot be
/// settled.
fn the_ruling_meets_the_arc(
    side: &FaceMeet,
    leans: Decision,
    up: [Interval; 3],
    at: [Interval; 3],
) -> Option<bool> {
    let keeps = if (side.material_side == Decision::Neg) == (leans == Decision::Neg) {
        Decision::Neg
    } else {
        Decision::Pos
    };
    let [from, to] = side.ends.each_ref().map(|end| end.point().enclosure());
    let chord = core::array::from_fn(|axis| to[axis] - from[axis]);
    let off_the_start = core::array::from_fn(|axis| at[axis] - from[axis]);
    match dot(up, cross(chord, off_the_start)).sign() {
        reads @ (Decision::Pos | Decision::Neg) => Some(reads == keeps),
        Decision::Zero | Decision::Undecidable { .. } | _ => None,
    }
}

/// Whether a flat piece's material is inside what `wire`, one whole closed
/// curve along `partner`, bounds: then the wire winds once round each point
/// there, and otherwise it winds round each the other way. `None` where a ring
/// square to `flat` cannot say which of its two circles the wire is.
///
/// **The wire's material side says it, and where the wire comes in the piece
/// does not.** The arrangement puts a flat piece's outer wire first only where
/// that wire is in the lump of the face's own outline. A piece of a round
/// plate's top between a bar's circle and the inner circle of a half ring
/// standing round it has neither in that lump, and came with the bar's circle
/// first: read as the outer wire, it wound every point of the band zero times
/// and none was found inside it
/// (`tests/a_half_ring_standing_on_a_round_plate.rs`, row Q1580).
///
/// What a ball, a pipe or a cone bounds on a plane is the part inside it, and
/// so is what each meridian circle of a ring bounds on a plane through its
/// axis, so the material is in it where it is on the partner's negative side.
/// On a plane square to a ring's axis each of the two circles bounds the disk
/// inside it ([`in_the_inner_disk_of`]): the band inside the ring is in what
/// the outer circle bounds, and outside what the inner one does.
fn what_a_round_wire_bounds_holds_the_material(
    flat: &Plane,
    partner: &Surface,
    wire: &[FaceMeet],
) -> Option<bool> {
    let in_the_partner = wire[0].material_side == Decision::Neg;
    if let Surface::Torus(ring) = partner
        && is_square_to_the_axis(flat, ring)
    {
        return Some(in_the_partner == runs_round_the_outer_circle(flat, ring, &wire[0])?);
    }
    Some(in_the_partner)
}

/// Which side of `partner` both `point` and a point of `flat` beside it are
/// on, or undecided where that could not be shown.
///
/// It is read over `reaching_the_plane`, the point's box stretched until it
/// reaches the plane ([`stretched_to_the_plane`]), which holds both: a sign over
/// a box is the sign of every point in it. That holds whatever the partner is,
/// and it is what a hole needs as much as a disk. A point outside a ball on the
/// far side of the plane from its centre can have its foot in the hole the ball
/// cuts, because the ball is narrower there than on the plane
/// (`a_point_outside_a_ball_whose_foot_is_in_its_hole_is_not_held`).
///
/// **A ball is first asked about the point alone, where the far side makes that
/// enough.** A point there inside the ball has its foot nearer the centre, and
/// every point on the way down to it too, so all of them are inside. It is the
/// only reading that reaches a cap a fraction of a spacing deep: over the
/// stretched box the ball's sign is too near zero for the interval sum to
/// settle, and the points [`points_beyond_a_plane_in_a_ball`] builds for such a
/// disk are all on that side
/// (`a_point_a_rounding_off_a_thin_disks_middle_is_held_from_either_side`).
fn side_of_a_partner_by_its_plane(
    flat: &Plane,
    partner: &Surface,
    point: PredicatePoint,
    reaching_the_plane: PredicatePoint,
) -> Decision {
    if let Surface::Sphere(ball) = partner
        && is_beyond_the_plane_from_the_centre(flat, ball, point)
        && ball.side_of(point) == Decision::Neg
    {
        return Decision::Neg;
    }
    side_of_surface(partner, reaching_the_plane)
}

/// Whether `point` is certainly on one side of `flat` and the centre of `ball`
/// is on the other, or on the plane.
fn is_beyond_the_plane_from_the_centre(flat: &Plane, ball: &Sphere, point: PredicatePoint) -> bool {
    let centres_side = flat.side_of_point(PredicatePoint::from(ball.centre().coordinates()));
    matches!(
        (side_of_a_flat_face(flat, point), centres_side),
        (Decision::Pos, Decision::Neg | Decision::Zero)
            | (Decision::Neg, Decision::Pos | Decision::Zero)
    )
}

/// Whether `point`, taken as being on `cone`, is on the half of it `piece` is
/// on, or `None` when nothing here can say.
///
/// **A cone's surface is both its halves**, meeting at the apex, and a piece
/// lies on one of them. Its sides do not say which. A pointed cone's face is
/// trimmed by its cap alone, and the whole of the other half is on the side of
/// the cap the face keeps, so a point there passes every side of the piece.
/// [`candidate_points`] makes such points: a step of one radius towards the
/// apex goes past it wherever the cone's radius is more than its height, and a
/// cone whose radius is twice its height, swallowed by a block, had its face
/// placed by a point under the apex and every rule refused
/// (`docs/decisions.md` §350).
///
/// Which half is read off the piece's corners, as `holds.rs` reads a face's:
/// the sign of `(corner − apex) · w`. A point at the apex is on both halves, so
/// it is `None`, and so is one too near the apex's plane to tell.
fn is_on_the_nappe_the_piece_is_on(
    piece: &Piece,
    cone: &Cone,
    point: PredicatePoint,
) -> Option<bool> {
    let mut pieces_half = None;
    for side in piece.wires.iter().flatten() {
        // A corner at the apex, or too near its plane to tell, says nothing
        // about which half the piece is on; its other corners say.
        let found @ (Decision::Pos | Decision::Neg) =
            how_far_up_the_axis(cone, side.ends[0].point())
        else {
            continue;
        };
        if pieces_half.is_some_and(|already| already != found) {
            return None;
        }
        pieces_half = Some(found);
    }
    let found @ (Decision::Pos | Decision::Neg) = how_far_up_the_axis(cone, point) else {
        return None;
    };
    Some(found == pieces_half?)
}

/// Which side of the plane through the apex square to the axis `point` is on:
/// the sign of `(point − apex) · w`, over the point's box.
fn how_far_up_the_axis(cone: &Cone, point: PredicatePoint) -> Decision {
    let apex = cone.apex().coordinates();
    let up = cone.axis().direction().components();
    let at = point.enclosure();
    (0..3)
        .map(|axis| (at[axis] - Interval::exactly(apex[axis])) * Interval::exactly(up[axis]))
        .fold(Interval::exactly(0.0), |so_far, term| so_far + term)
        .sign()
}

/// What a side of a curved piece asks a point besides which side of its
/// partner the point is on: what picks the closed curve it runs along out of
/// the other one its two surfaces cross in, which must hold the point strictly
/// inside. Nothing where the piece has a side along that other curve too, or
/// where the side's ends do not say which curve it runs along.
///
/// **Two surfaces can cross in two closed curves, and then the partner's side
/// does not say which piece holds a point.** A ball on a cone's axis pushing
/// out through its side crosses the cone in two circles, and they cut each
/// surface in three: a band between them, bounded by both, and beyond each
/// circle a part bounded by that circle alone. Crossing a circle crosses to the
/// other side of the partner, so the two parts beyond are on one side of it and
/// the band is alone on the other. The ball's cap below the lower circle and
/// its cap above the upper one are both inside the cone, and a point of the
/// upper one, `(2.93, 0, 4.66)`, was taken as a point of the lower one and
/// found inside a frustum the lower cap is wholly below (Q1197).
///
/// What tells the two parts beyond apart is what `intersect` names each curve
/// by, the `picked_by` of [`MeetPoint::WhereAClosedCurveStarts`]. For a ball,
/// a pipe or a second cone on a cone's axis, and for a pipe through a ball,
/// that is a half-space square to the axis whose plane lies between the two
/// curves, facing its own. That plane meets each surface only inside its band
/// — on the cone and the ball a circle between the two, on a pipe through a
/// ball a circle inside the ball and the ball's equator outside the pipe — so
/// each part beyond a curve, which does not meet the plane, lies wholly on its
/// own curve's side of it: the half-space holds the cap beyond its circle, or
/// everything of the cone past it, the other nappe included, and nothing of the
/// part beyond the other curve. For two curves of another shape a picker can
/// still only rule points out, so asking it never holds a point that the
/// partner alone did not.
///
/// **A piece with sides along both curves asks neither.** It is the band, alone
/// on its side of the partner, so the partner already says, and the two
/// pickers hold opposite halves of it: asked both, the band of a pointed cone
/// inside the ball of `tests/a_cone_hollowed_along_its_axis.rs` held no point,
/// and was placed only by a side it shares. A side between two corners where a
/// third surface cuts its curve does not say which curve it is, so a piece with
/// such a side along the partner asks nothing of it, as before. Measured on
/// that file's ball cut in half by a wall through both circles (Q1248): against
/// the pointed cone each cap of the half ball has only such sides along the
/// cone, both caps are inside it, and every rule still answers.
///
/// **A piece with a side along one curve only can still be part of the band**,
/// where its face stops short of the other curve: the frustum from `z = 2` of
/// that file holds only the upper circle, and its piece inside the ball runs
/// from its own rim up to it. The picker then rules out the points of the piece
/// beyond the half-space's plane, which costs candidates and keeps nothing it
/// should not; there the plane is the frustum's rim, `z = 2`, and costs none. A
/// frustum from `z = 1.5` reaches below that plane, and its band is still
/// placed, by a point above it (Q1248).
fn what_else_a_curved_side_asks<'a>(piece: &'a Piece, side: &'a FaceMeet) -> &'a [Picker] {
    let Some(mine) = which_curve_it_runs_along(side) else {
        return &[];
    };
    let along_one_curve = piece
        .wires
        .iter()
        .flatten()
        .filter(|other| other.carried_by[1] == side.carried_by[1])
        .all(|other| which_curve_it_runs_along(other) == Some(mine));
    if along_one_curve { mine } else { &[] }
}

/// What picks the closed curve `side` runs along out of the others its two
/// surfaces cross in, as an end of it where the curve starts or half way round
/// carries it, or `None` when neither end is one.
fn which_curve_it_runs_along(side: &FaceMeet) -> Option<&[Picker]> {
    side.ends.iter().find_map(|end| match end {
        MeetPoint::WhereAClosedCurveStarts { picked_by, .. }
        | MeetPoint::HalfWayRoundAClosedCurve { picked_by, .. } => Some(picked_by.as_slice()),
        MeetPoint::Existing { .. }
        | MeetPoint::OfThreeSurfaces { .. }
        | MeetPoint::APoleOfAClosedSurface { .. } => None,
    })
}

/// One when `point` is between a curved side of a flat piece and the chord
/// across it, where the corners alone wind nothing round it; zero when it is
/// not; one or nothing when the turn about the chord cannot be settled; and
/// `None` when a side along a ring cannot say which of the ring's circles it
/// runs round, which side of a half-space the point is on, or what
/// [`across_the_inner_disk`] counts. A side along a ring with its material
/// outside the ring can count minus one there.
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
/// **The chord's two ends are stretched to the plane first**, as
/// [`the_piece_holds`] stretches the corners it winds round, and for its
/// reason: a corner whose box lies off the plane is read far along it. A half
/// disk's two corners wind round nothing, so its chord decides alone, and `2²⁰`
/// out with both corners `1e-8` off the plane it missed points `1e-3` inside it
/// (`a_far_half_disk_whose_corners_are_off_its_plane_holds_no_point_across_its_chord`).
///
/// **A turn about the chord that cannot be settled is one or nothing**, and
/// [`the_piece_holds`] holds the point where the count is one with it counted
/// nothing. A piece's wires wind once round each of its points and not at all
/// round any other, so that one cannot really be a two. A block's floor whose
/// corner is a rounding outside a pipe's rim has that corner cut off by an arc
/// between where the block's two walls cross the rim, and the two are boxes a
/// rounding wide that overlap at the corner, so the chord between them has no
/// direction and the turn about it is undecided for every point. While that
/// turn left the whole count undecided, the floor of a block cornered twice on
/// a round tab's rim held no point, a unit clear of the arc included, and every
/// certified fuse of the block under the tab was refused [`INSIDE_A_PIECE`]
/// (`a_rectangle_with_its_corner_cut_off_by_an_arc_a_rounding_long_holds_its_own_points`,
/// Q2297). Read as nothing, it would make a half disk's points near its chord
/// read as outside it rather than as undecided
/// (`a_far_half_disk_whose_corners_are_off_its_plane_holds_no_point_across_its_chord`).
/// Where another side of `wire` runs back along the chord on a plane, that
/// plane's sign settles the turn instead
/// ([`which_side_of_a_wall_back_along_the_chord`]): a disk whose two corners
/// are that close, cut by a straight side, has nothing else to wind round its
/// points.
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
/// different searches, now give out in the same one. Two side-by-side bars now
/// fuse by every rule, both ways round.
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
/// that, an open one is that once the point is held to the side's own nappe,
/// and an undecided one is read as no bulge, which costs a refusal and never a
/// wrong answer.
///
/// It used to compare the stored normal against the stored axis direction, so
/// only a cap's own plane counted, and `holds.rs` and `assemble.rs` each spelled
/// the question again (§23.8, §208.6). There is one spelling now, in
/// `rcad-geom`, and the ellipse is admitted: the sign
/// `(n·a)²(1 + s²) − s²|n|²|a|² > 0` is exact, and the region an ellipse's
/// inside leaves on a plane is one convex piece just as a circle's is.
///
/// **A torus is counted on a plane square to its axis, along the outer of the
/// two circles it cuts there** ([`runs_round_the_outer_circle`]). The torus's
/// inside on such a plane is the band between two circles round the axis, so
/// a point its sign holds is inside the outer circle, and a side along that
/// circle whose material is inside the torus has the outer disk on its left:
/// the ball's paragraph carries over. A side along the inner circle with its
/// material inside the torus bites into the piece, and the bite is part of the
/// inner disk, which the torus's sign already took out, so it counts nothing.
/// The top halves of the two half bends of one ring have their floors on
/// `z = 0` with all four corners on the `x` axis, so the corners wind round
/// nothing, and until the outer arc was counted no point was held and every
/// fuse of the two was refused [`INSIDE_A_PIECE`] (row Q1268). A side whose
/// material is outside the torus does not count nothing: its bulge holds the
/// part of the inner disk on its side of the chord, which the torus's sign
/// lets through, and [`across_the_inner_disk`] counts it (row Q1305).
///
/// **A torus is counted on a plane through its axis too, along either of the
/// two meridian circles it cuts there, on the point's own side of the
/// half-space `meridian` that names the side's circle.** The torus's inside on
/// such a plane is the two circles' disks, one each side of that half-space's
/// plane, which [`intersect`] proved clears both: so a point the torus's sign
/// holds and the half-space keeps is in the side's own disk, and the ball's
/// paragraph carries over. Without the half-space the bulge would count the
/// other disk's points on the same side of the chord, which the torus's sign
/// holds just as surely. A bend's start cap is a half disk with nothing but
/// its chord along the floor and half a meridian circle round it, so its
/// corners wind round nothing, and until its arc was counted no point was held
/// and every fuse of two bends' top halves meeting flush was refused
/// [`INSIDE_A_PIECE`] (row Q1373).
///
/// **A side that closes on itself has no chord, and its bulge is the whole
/// region it goes round**, which holds every point its partner's sign does.
/// Such a side is in a wire of more than one side where two whole circles
/// touch at a corner they share: a thin bar's cap flush with a thick bar's,
/// the two pipes touching on the thick bar's seam, leaves the thick cap a
/// crescent whose one wire is the rim and the thin circle, each from the seam
/// corner back to it. The corners are that one corner twice and wind round
/// nothing, so the rim counts one and the thin circle, whose material is
/// outside it, nothing. Asked of the chord, the turn about a chord with no
/// length was zero, no point of the crescent was held, and every fuse was
/// refused [`INSIDE_A_PIECE`] (row Q1748). A side along a ring with its
/// material outside goes to [`across_the_inner_disk`] first, which reads such
/// a side as undecided, and that only loses a candidate.
fn in_the_bulge_of(
    flat: &Plane,
    side: &FaceMeet,
    wire: &[FaceMeet],
    meridian: Option<&Picker>,
    in_plane: &InPlane,
    point: PredicatePoint,
) -> Option<Bulge> {
    if let Surface::Torus(ring) = side.carried_by[1]
        && meridian.is_none()
        && side.material_side == Decision::Pos
    {
        return across_the_inner_disk(flat, &ring, side, in_plane, point).map(Bulge::Counts);
    }
    let one_round_region = match side.carried_by[1] {
        Surface::Sphere(_) => true,
        Surface::Cylinder(pipe) => pipe.is_cut_in_a_closed_curve_by(flat) == Some(true),
        Surface::Cone(cone) => match cone.is_cut_in_a_closed_curve_by(flat) {
            Some(true) => true,
            Some(false) => is_on_the_nappe_the_side_is_on(&cone, side, point),
            None => false,
        },
        Surface::Torus(ring) => match meridian {
            Some(this_circle) => every_picker_keeps(core::slice::from_ref(this_circle), point)?,
            None => runs_round_the_outer_circle(flat, &ring, side)?,
        },
        Surface::Plane(_) | _ => false,
    };
    if !one_round_region || side.material_side != Decision::Neg {
        return Some(Bulge::Counts(0));
    }
    if closes_on_itself(side) {
        return Some(Bulge::Counts(1));
    }
    let turn = which_side_of_the_chord(flat, side, in_plane, point)
        .or_else(|| which_side_of_a_wall_back_along_the_chord(side, wire, point));
    Some(match turn {
        Some(turn) => Bulge::Counts(i32::from(turn == Decision::Neg)),
        None => Bulge::OneOrNothing,
    })
}

/// What one side of a flat piece adds to how often the piece's wires wind
/// round a point, over what its corners alone wind ([`in_the_bulge_of`]).
enum Bulge {
    /// One, minus one or nothing.
    Counts(i32),
    /// One where the point is between the side and its chord and nothing where
    /// it is not, and the turn about the chord could not say which.
    OneOrNothing,
}

/// What a side along `ring` whose material is outside it counts for `point`,
/// which [`the_piece_holds`] has already found outside the ring: on a plane
/// square to the ring's axis, minus one along the outer circle and one along
/// the inner where `point` is in the inner disk and on the arc's side of the
/// chord, and nothing on any other plane or anywhere else. `None` where the
/// pipe of the ring's own radius or the chord cannot say.
///
/// **The arc and its chord wind once round what is between them**, clockwise
/// for an arc that runs clockwise round the axis, which is the outer circle's
/// way with the material outside it on the left, and the other way round for
/// the inner circle's. What is between an arc of the outer circle and its chord
/// is outside the ring only where it is in the inner disk, and the inner disk
/// is all outside the ring; a point outside the ring is in the inner disk
/// exactly when it is inside the pipe of the ring's own radius, which runs
/// between the two circles. Counting nothing, the piece of `z = 0` beyond the
/// ring of four round a tube of one, between `x = 0` and a pipe of 10, held a
/// point of the inner disk
/// (`a_plate_beyond_a_rings_outer_circle_holds_no_point_of_the_inner_disk`),
/// and the half disk inside the circle of 3 held none of its own
/// (`a_half_disk_inside_a_rings_inner_circle_holds_its_own_points`).
fn across_the_inner_disk(
    flat: &Plane,
    ring: &Torus,
    side: &FaceMeet,
    in_plane: &InPlane,
    point: PredicatePoint,
) -> Option<i32> {
    if !is_square_to_the_axis(flat, ring) {
        return Some(0);
    }
    let outer = runs_round_the_outer_circle(flat, ring, side)?;
    match the_pipe_of_the_rings_own_radius(ring).side_of(point) {
        Decision::Neg => (),
        Decision::Pos => return Some(0),
        Decision::Zero | Decision::Undecidable { .. } | _ => return None,
    }
    let (where_the_arc_is, counts) = if outer {
        (Decision::Pos, -1)
    } else {
        (Decision::Neg, 1)
    };
    let turn = which_side_of_the_chord(flat, side, in_plane, point)?;
    Some(if turn == where_the_arc_is { counts } else { 0 })
}

/// Whether `point`, which [`the_piece_holds`] has already found outside the
/// partner of a whole round wire, is inside what the wire bounds all the same:
/// where the partner is a ring, `flat` is square to its axis, and `point` is in
/// the inner disk. `None` where the pipe of the ring's own radius cannot say.
///
/// **Both circles such a plane cuts a ring in go round the inner disk**, and
/// the inner disk is all outside the ring, so the ring's sign alone says
/// nothing about which side of either circle a point outside it is on. A point
/// outside the ring is in the inner disk exactly when it is inside the pipe of
/// the ring's own radius, which runs between the two circles
/// ([`across_the_inner_disk`] asks the same of an arc). Asked of the ring's
/// sign alone, a square less the disk inside a ring's whole outer circle held
/// a point of the inner disk
/// (`a_square_less_a_rings_whole_outer_circle_holds_no_point_of_the_inner_disk`),
/// and the disk inside its whole inner circle none of its own
/// (`a_disk_inside_a_rings_whole_inner_circle_holds_its_own_points`, row Q1501).
///
/// Every other partner leaves nothing outside it inside what it bounds: a ball,
/// a pipe or a cone leaves one region on a plane, and so does each meridian
/// circle of a ring on a plane through its axis.
fn in_the_inner_disk_of(flat: &Plane, partner: &Surface, point: PredicatePoint) -> Option<bool> {
    let Surface::Torus(ring) = partner else {
        return Some(false);
    };
    if !is_square_to_the_axis(flat, ring) {
        return Some(false);
    }
    match the_pipe_of_the_rings_own_radius(ring).side_of(point) {
        Decision::Neg => Some(true),
        Decision::Pos => Some(false),
        Decision::Zero | Decision::Undecidable { .. } | _ => None,
    }
}

/// Which side of the chord between `side`'s two ends, stretched to the plane,
/// `point` is on: `Neg` on the right, `Pos` on the left, and `None` on the
/// chord's line or where that could not be shown.
fn which_side_of_the_chord(
    flat: &Plane,
    side: &FaceMeet,
    in_plane: &InPlane,
    point: PredicatePoint,
) -> Option<Decision> {
    let [from, to] = side
        .ends
        .each_ref()
        .map(|end| stretched_to_the_plane(flat, end.point()));
    let turn = in_plane.turn(from?, to?, point);
    // Compared rather than matched, so that `Decision`'s wildcard arm is not
    // written once more (§40): a zero or an undecided turn is `None` either way.
    (turn == Decision::Neg || turn == Decision::Pos).then_some(turn)
}

/// Which side of the chord across `side` `point` is on, as
/// [`which_side_of_the_chord`] says it, read off a side of `wire` along a
/// plane that runs back between the same two ends instead: `None` where the
/// wire has no such side, where its ends are not both certainly on that plane
/// and on `flat`, or where the plane's sign over `point` cannot be settled.
///
/// **Two ends whose boxes overlap give the chord between them no direction**,
/// so no turn about it is settled, but where a plane carries a side back
/// between them the chord is that plane's line across `flat`. A meet of three
/// surfaces that names both planes lies exactly on both, so a chord between
/// two of them is on that line. The side back runs from the chord's end to its
/// start with its material on its left, which is the chord's right, and the
/// side of its plane that material is on says which half of `flat` that is. A
/// corner the input already had is not read: four planes round it need not
/// meet in one point, and it can be off the one asked (§294.19).
///
/// The floor `z = −1` of the round bar turned a quarter turn in the L plate is
/// cut by the plate's inner side, which crosses the bar's rim either side of
/// the seam in two boxes that overlap. The piece of the floor between that
/// side and the long arc held no point, its centre included, and every
/// certified fuse of the two was refused [`INSIDE_A_PIECE`]
/// (`a_disk_cut_a_rounding_inside_its_rim_holds_its_centre_and_the_sliver_does_not`,
/// `tests/a_bar_flush_with_an_l_plates_inner_side_turned.rs`, Q2655).
fn which_side_of_a_wall_back_along_the_chord(
    side: &FaceMeet,
    wire: &[FaceMeet],
    point: PredicatePoint,
) -> Option<Decision> {
    let back = wire.iter().find(|other| {
        other.ends[0] == side.ends[1]
            && other.ends[1] == side.ends[0]
            && other.carried_by[1].as_plane().is_some()
            && other.ends.iter().all(|end| {
                matches!(end, MeetPoint::OfThreeSurfaces { surfaces, .. }
                    if other.carried_by.iter().all(|carrier| surfaces.contains(carrier)))
            })
    })?;
    let wall = back.carried_by[1]
        .as_plane()
        .expect("the side back was picked for running along a plane");
    let found = side_of_a_flat_face(wall, point);
    if found == back.material_side {
        Some(Decision::Neg)
    } else if found == the_other_way(back.material_side) {
        Some(Decision::Pos)
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

/// Whether `side`, along `ring` on `flat`, runs round the outer of the two
/// circles a plane square to the ring's axis cuts it in: `false` on a plane
/// that is not square to the axis, and `None` where the side's end cannot be
/// shown off the pipe of the ring's own radius, which has one circle inside it
/// and the other outside.
///
fn runs_round_the_outer_circle(flat: &Plane, ring: &Torus, side: &FaceMeet) -> Option<bool> {
    if !is_square_to_the_axis(flat, ring) {
        return Some(false);
    }
    match the_pipe_of_the_rings_own_radius(ring).side_of(side.ends[0].point()) {
        Decision::Pos => Some(true),
        Decision::Neg => Some(false),
        Decision::Zero | Decision::Undecidable { .. } | _ => None,
    }
}

/// Whether `flat` is square to `ring`'s axis, asked exactly, as `holds.rs`
/// asks it ([`lie_along_one_line`]).
fn is_square_to_the_axis(flat: &Plane, ring: &Torus) -> bool {
    lie_along_one_line(
        flat.normal().components(),
        ring.axis().direction().components(),
    )
}

/// The pipe round `ring`'s axis through the middle of its tube, which a plane
/// square to the axis meets between the two circles it cuts the ring in.
fn the_pipe_of_the_rings_own_radius(ring: &Torus) -> Cylinder {
    Cylinder::new(ring.axis(), ring.ring_radius())
        .expect("a ring's radius is finite and above nothing, which is all a pipe's needs")
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
        [only] if closes_on_itself(only) => Some(only.carried_by[1]),
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
/// picker certainly keeps, is `None`. [`the_piece_holds`] does not ask this
/// for that family: which circle the wire is says it all
/// ([`runs_round_the_outer_circle`]).
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
/// That is one half-space, looked up by `holds.rs`'s
/// [`the_half_space_naming_the_meridian_circle_through`] for a face of a solid
/// too, and handed back as a list because every other wire's answer is one.
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
    Ok(
        the_half_space_naming_the_meridian_circle_through(flat, ring, corner, policy, log)?
            .map(|this_circle| vec![this_circle]),
    )
}

/// For every side of every wire of a flat piece, the half-space naming the
/// meridian circle of a ring the side runs along, which [`in_the_bulge_of`]
/// reads its bulge's disk by; `None` for a side along anything else, for one
/// on a plane that does not hold the ring's axis exactly
/// ([`how_the_axis_lies_on`]), and for one whose corner no one circle
/// certainly keeps. Nothing for a curved piece.
///
/// Worked out once per piece, as [`what_else_each_round_wire_asks`] is, since
/// the answer is about the side and not about the point.
///
/// # Errors
///
/// Whatever [`intersect`] gives for the piece's plane against a ring whose
/// axis it holds.
fn the_meridian_circle_each_side_runs_round(
    piece: &Piece,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<Vec<Option<Picker>>>> {
    let Some(flat) = piece.surface.as_plane() else {
        return Ok(Vec::new());
    };
    let mut each = Vec::with_capacity(piece.wires.len());
    for wire in &piece.wires {
        let mut sides = Vec::with_capacity(wire.len());
        for side in wire {
            sides.push(
                if let Surface::Torus(ring) = &side.carried_by[1]
                    && how_the_axis_lies_on(flat, ring) == Decision::Zero
                {
                    the_half_space_naming_the_meridian_circle_through(
                        &piece.surface,
                        &side.carried_by[1],
                        side.ends[0].point(),
                        policy,
                        log,
                    )?
                } else {
                    None
                },
            );
        }
        each.push(sides);
    }
    Ok(each)
}

/// Whether every one of `pickers` keeps `point`: `Some(false)` when one
/// certainly rules it out, `None` when one cannot say.
///
/// Each list is empty or holds the one half-space naming a ring's meridian
/// circle ([`what_else_each_round_wire_asks`],
/// [`the_meridian_circle_each_side_runs_round`]), so each picker is asked what
/// `holds.rs` asks it, [`the_meridian_half_space_keeps`], and a point on its
/// plane is kept.
fn every_picker_keeps(pickers: &[Picker], point: PredicatePoint) -> Option<bool> {
    let mut kept = true;
    for picker in pickers {
        kept &= the_meridian_half_space_keeps(picker, point).ok()?;
    }
    Some(kept)
}

/// Where one point sits relative to each solid in the set, or why this point
/// cannot say.
///
/// **Whether the piece lies on a face of the solid is asked first, and the ray
/// only after it**, for every piece. The ray was never what settles `ON`. §17.1
/// makes the stored plane authoritative, [`coincident_face`] proves two planes
/// are the same plane from their coefficients alone, and the winding proves the
/// point is within the face's outline through the corners the planes meet at —
/// none of which cares how wide a box is or where a stored corner was left.
///
/// The ray reads a face as triangles fanned from its **stored** corners, and a
/// valid solid may store a corner a hair off where its planes meet (§75.6). A
/// box with its corner `(0, 0, 0)` stored as `(0, 1e-30, 0)` tilts both
/// triangles of its `y = 0` face, and the ray certified a point lying on that
/// face as `Outside`: the same two boxes fused one way round and were not a
/// solid the other (§316.3, §351). A candidate on a *sphere* is worked out
/// through the surface's own two coordinates and lands a rounding off it, and
/// the ray is as confident and as wrong about it.
///
/// So what reaches the ray lies on no face of the solid. A point that lands
/// exactly on an edge or a corner of one of the solids says nothing about the
/// piece around it, and an undecided answer is a box straddling a plane — every
/// corner of a turned solid is one, which is why `A \ A` there needs coincidence
/// asked at all. Either abandons this candidate rather than being read as `ON`.
///
/// **A point off every face but between one face's stored plane and its
/// corners' triangles is the ray's to place.** The fuse cuts along the stored
/// plane, and a turned solid's corners are a rounding off it, so the two can
/// put the point on opposite sides of the face (§200.1). Wherever the ray's
/// count turns on such a triangle it asks the stored plane too, and where the
/// two part it places the point by the plane or says `Undecidable`
/// (`classify.rs`'s `count_a_polygon`, §346.6). `select` once dropped every
/// point any triangle and its face's plane disagreed about, whatever the ray
/// then said and however far the point was from that face, and with them every
/// point of a cap a turned block's face takes off a ball
/// (`tests/a_ball_shaved_by_a_turned_blocks_face.rs`).
fn settle_against_each(
    model: &Model,
    solids: &[&BakedSolid],
    against: &[usize],
    point: PredicatePoint,
    piece: &Piece,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Settled> {
    let mut lying_on = LyingOn::default();
    for &index in against {
        if let Some(same_way) = coincident_face(model, solids[index], piece, point, policy, log)? {
            lying_on.add(index, same_way);
            continue;
        }
        let solid: SolidView<'_> = model.solid_at(solids[index].key.slot_in(model.id()));
        let mut ray = ProvenanceLog::new();
        let placed = solid.classify_uncertain_point(point, policy, &mut ray);
        if matches!(placed, Containment::Inside | Containment::Outside)
            && a_face_may_run_between_it_and_the_plane(solid, piece, point, policy, &mut ray)
        {
            return Ok(Settled::Abandoned);
        }
        // A classification nobody could make is not a decision anybody took, so
        // it says nothing in the log (§5) — and criterion 7 would otherwise read
        // it as slop this fuse had accepted, which is exactly the conflation
        // §17.2 spends a section keeping apart. Nor does one on the skin, which
        // places nothing: the point is abandoned, or placed by the one face its
        // box meets, which asks nothing of the ray.
        if !matches!(
            placed,
            Containment::Undecidable { .. } | Containment::OnBoundary
        ) {
            ray.drain_into(log);
        }
        match placed {
            Containment::Inside => return Ok(Settled::At(Placement::Inside)),
            Containment::Outside => (),
            // A face nobody can follow is not a piece that could be placed
            // somewhere else, and reading it as `Outside` would drop material.
            // This was `unreachable!`, on the grounds that the fuse refuses
            // curved solids at entry; `docs/decisions.md` §35 records why that
            // argument stops holding and why what replaces it is a refusal
            // about a caller's solid rather than a panic. `solid` is the
            // caller's, so `face` is a key the caller can read.
            Containment::CannotFollowAFacesTrimming { face } => {
                return Ok(Settled::MetAFaceNobodyFollows(face));
            }
            // `Error::CannotFireARayAtAFacesSurface` exists since Q883, and
            // `assemble.rs`'s `one_shell_is_inside_another` returns it. Here it
            // still stays with the undecided points, because no fuse is known
            // to be refused for it: the point beside it answers every point
            // whose box no face of the solid meets (§323). Since Q1392 let the
            // pieces of two bends placed a quarter turn round by a float turn be
            // cut, their fuses reach this arm, but each piece whose point
            // reached it is placed by some other point or rule, and since
            // Q1537 each such fuse answers
            // (`tests/a_piece_no_ray_can_place_names_its_refusal.rs`). Since
            // Q2331 a wedge whose face is a plane through the axis a hair below
            // a bend's cap reaches it too, at hairs from `1e-15` down, in each
            // of its six fuses, and each answers
            // (`a_wedge_a_hair_below_a_bends_cap_answers_every_rule_with_their_volumes`).
            // A fuse refused for want of this ray is what changing this waits
            // for (§35.6).
            Containment::CannotFireARayAtAFacesSurface { .. } => return Ok(Settled::Abandoned),
            // Coincidence was asked above and found no face the piece lies on.
            Containment::OnBoundary | Containment::Undecidable { .. } => {
                match by_the_one_face_its_box_meets(solid, piece, point, policy, log) {
                    Some(true) => return Ok(Settled::At(Placement::Inside)),
                    Some(false) => (),
                    None => return Ok(Settled::Abandoned),
                }
            }
        }
    }
    Ok(Settled::At(lying_on.placement()))
}

/// Whether a face of `solid` may pass between `point` and the points of a flat
/// piece's plane beside it, so that where the ray put `point` need not be
/// where the piece is.
///
/// [`the_piece_holds`] holds a point a rounding off a flat piece's plane by
/// the point's box stretched to the plane ([`stretched_to_the_plane`]), so
/// what it shows is that the piece holds points of the plane in that box, not
/// that it holds `point`. The ray places `point`. The two answers are the same
/// unless the skin of `solid` runs between `point` and those points of the
/// plane. A flat face cannot where its plane leaves `point` and the four
/// corners of the plane's part of the box all strictly on one side
/// ([`the_corners_of_the_plane_in`]), and a curved one where its surface
/// leaves `point` and the whole box on one side. Any other face cannot where
/// it holds no part of the box.
///
/// **The plane is asked at its corners, not over the box.** The box runs past
/// the plane by as much as it took to put each end on its own side, and a
/// face's plane parallel to the piece's and a rounding from it cuts that
/// overshoot without coming between `point` and the piece: under
/// `Policy::tolerant(3.0)` a tall block through a turned flat one held a box
/// so by its own face, and the piece was placed by a later candidate through a
/// tolerant `orient3d` its log did not expect (`turning_a_fuse.rs`'s
/// `a_tall_block_through_a_flat_one_turned_answers_every_rule_under_a_tolerant_policy`).
///
/// **What a face that holds no part of the box was decided with goes into
/// `ray`**, which says why `point` was placed where it was, so a tolerance
/// spent on it is logged where the ray's own is, and dropped with it.
///
/// Two quarter bends placed round by `−2` and by `−2 − π/2` meet at two cap
/// planes through the ring's axis a rounding apart, and each cap's disk lies
/// `1.7e-17` inside the other bend. A point `8e-16` off the first bend's start
/// cap, beyond the copy's end cap plane, was held by that cap and placed
/// outside the copy, and every rule both ways round was refused
/// `ResultIsNotManifold` along the cap's rim (Q1949).
fn a_face_may_run_between_it_and_the_plane(
    solid: SolidView<'_>,
    piece: &Piece,
    point: PredicatePoint,
    policy: &Policy,
    ray: &mut ProvenanceLog,
) -> bool {
    let Some(flat) = piece.surface.as_plane() else {
        return false;
    };
    if side_of_a_flat_face(flat, point) == Decision::Zero {
        return false;
    }
    let reach = stretched_to_the_plane(flat, point)
        .expect("the_piece_holds stretched this point to the plane before it held it");
    let corners = the_corners_of_the_plane_in(flat, reach);
    for face in solid.faces() {
        let surface = face.surface();
        let clear = match surface.as_plane() {
            Some(plane) => {
                let where_the_point_is = side_of_a_flat_face(plane, point);
                matches!(where_the_point_is, Decision::Pos | Decision::Neg)
                    && corners.iter().all(|&three| {
                        side_of_a_plane_where_three_planes_meet(three, plane.coefficients())
                            == Ok(where_the_point_is)
                    })
            }
            None => {
                let where_the_point_is = side_of_surface(&surface, point);
                matches!(where_the_point_is, Decision::Pos | Decision::Neg)
                    && side_of_surface(&surface, reach) == where_the_point_is
            }
        };
        if clear {
            continue;
        }
        let mut asked = ProvenanceLog::new();
        match solid.face_holds_the_point(face.key(), reach, policy, &mut asked) {
            Ok(OnAFace::NotHeld) => asked.drain_into(ray),
            Ok(OnAFace::Held | OnAFace::OnItsBoundary) | Err(_) => return true,
        }
    }
    false
}

/// The four corners of the part of `flat` inside `reach`, each as the three
/// planes it is the one meet of: `flat` and one end of `reach` across each of
/// the two axes [`stretched_to_the_plane`] did not stretch along.
///
/// **That stretch is what makes these the corners.** It moved both ends of
/// `reach` along the axis `flat`'s normal has most of until each end's face
/// lies wholly on its own side of `flat`, so every line along that axis
/// through `reach` meets `flat` inside it, and the part of `flat` in `reach`
/// is the parallelogram these four meets span. A plane that leaves all four
/// strictly on one side leaves the whole parallelogram there. The axis is
/// picked here exactly as there, ties included; picked any other way, the
/// parallelogram could run out of `reach`.
fn the_corners_of_the_plane_in(flat: &Plane, reach: PredicatePoint) -> [[[f64; 4]; 3]; 4] {
    let normal = flat.normal().components();
    let along = (0..3)
        .max_by(|&one, &other| normal[one].abs().total_cmp(&normal[other].abs()))
        .expect("three axes");
    let [across, and_across] = [(along + 1) % 3, (along + 2) % 3];
    let boxed = reach.enclosure();
    let at = |axis: usize, end: f64| {
        let mut plane = [0.0; 4];
        plane[axis] = 1.0;
        plane[3] = -end;
        plane
    };
    let ends = |axis: usize| [boxed[axis].lo(), boxed[axis].hi()];
    let [one, other] = ends(across);
    let [first, second] = ends(and_across);
    [(one, first), (one, second), (other, first), (other, second)]
        .map(|(this, that)| [flat.coefficients(), at(across, this), at(and_across, that)])
}

/// Whether a flat piece is inside `solid`, read off the one face of `solid`
/// that the box of `point` meets, where that face's plane is parallel to the
/// piece's, number for number, and not the same plane: inside when the piece's
/// plane is behind the face's, outside when it is in front. `None` where the
/// box meets no such face, or meets another face too, or a sign is not proved.
///
/// **Inside a box that one face alone meets, the solid's skin is that face.**
/// The box is `point`'s stretched until it holds points of the piece's plane,
/// which [`the_piece_holds`] showed the piece holds, and then of the face's
/// plane `n · x = e`, turned out of the solid. Every other face has its surface
/// leave the box strictly on one side or holds no part of it, as
/// [`a_face_may_run_between_it_and_the_plane`] asks it, and the face holds the
/// whole box: every point `g` of its plane in the box is in its material, clear
/// of its boundary. The piece's plane, with the same normal `n`, is
/// `n · x = d`. Take a point `x` of the piece in the box. The segment from `g`
/// to `x` lies in the box, so it meets no other face, and none of it but `g` is
/// on the face's plane, which `d ≠ e` keeps `x` off. Beside `g` the solid is
/// the side of the face's plane its material is on, so the whole segment past
/// `g` is inside the solid where `d < e` and outside it where `d > e`, and so is
/// `x`. The piece was cut by every surface of the solid, so all of it lies
/// where `x` does (§17.4). `d` and `e` are two `f64`s compared exactly, and
/// nothing is a tolerance but what the policy lets the faces' own tests take.
///
/// The ray cannot see this. A box across a face's plane is one it cannot put
/// on either side, though every point of the piece in it is on one.
///
/// A half bar cut flush along its flat face by a block, the two turned 0.4
/// about `x` and then shifted by `(7, −5, 2.5)`, has the block's face one ulp
/// past the flat face, on the side away from the bar: the bar's material is
/// `n · x < −3.6317591142427994` and the block's `n · x > −3.631759114242799`.
/// Every point found inside the block's face across from the flat face is a
/// box `4e−15` across both planes, and every fuse of the two was refused
/// placing that piece against the bar (Q2043).
fn by_the_one_face_its_box_meets(
    solid: SolidView<'_>,
    piece: &Piece,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Option<bool> {
    let flat = piece.surface.as_plane()?;
    let reach = stretched_to_the_plane(flat, point)?;
    solid.faces().find_map(|face| {
        let Surface::Plane(plane) = face.surface() else {
            return None;
        };
        let outward = if face.orientation().is_reversed() {
            plane.reversed()
        } else {
            plane
        };
        let normal = outward.normal().components();
        let along = [*flat, flat.reversed()]
            .into_iter()
            .find(|one| one.normal().components() == normal)?;
        let (piece_at, face_at) = (along.offset(), outward.offset());
        if !(piece_at.is_finite() && face_at.is_finite()) || piece_at == face_at {
            return None;
        }
        let inside = piece_at < face_at;
        let reach = stretched_to_the_plane(&plane, reach)?;
        let mut asked = ProvenanceLog::new();
        if !matches!(
            solid.face_holds_the_point(face.key(), reach, policy, &mut asked),
            Ok(OnAFace::Held)
        ) {
            return None;
        }
        let only_this_one_meets_it =
            solid
                .faces()
                .filter(|other| other.key() != face.key())
                .all(|other| {
                    let surface = other.surface();
                    let clear = match surface.as_plane() {
                        Some(plane) => side_of_a_flat_face(plane, reach),
                        None => side_of_surface(&surface, reach),
                    };
                    matches!(clear, Decision::Pos | Decision::Neg)
                        || matches!(
                            solid.face_holds_the_point(other.key(), reach, policy, &mut asked),
                            Ok(OnAFace::NotHeld)
                        )
                });
        if !only_this_one_meets_it {
            return None;
        }
        asked.drain_into(log);
        Some(inside)
    })
}

/// What one point inside a piece says about where the piece sits.
enum Settled {
    /// Where the piece sits against every solid asked.
    At(Placement),
    /// The ray from the point met this face of a caller's solid, whose
    /// trimming nothing here follows, so the point was never placed. Not
    /// `Undecidable`: the question was never put (`docs/decisions.md` §35.6).
    MetAFaceNobodyFollows(FaceKey),
    /// The point says nothing, and the next one is tried.
    Abandoned,
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
    match (mine, outward_plane_of_face(face)) {
        (Some(mine), Some(theirs)) => {
            return Ok(match agreement_of_two_planes(&mine, &theirs, log)? {
                PlaneAgreement::SameWay => Some(true),
                PlaneAgreement::OppositeWay => Some(false),
                PlaneAgreement::Different => None,
                other => {
                    panic!("plane agreement came back with an answer nothing here knows: {other:?}")
                }
            });
        }
        // A plane and a curved surface are never one surface, so their whole
        // crossing is not asked for. `intersect` refuses a plane against a ring
        // as a quartic however far apart the two are, and that refused the
        // fuse (`docs/decisions.md` §322.7, Q326).
        //
        // No curved surface rcad-geom builds is a plane. A ball's radius and a
        // ring's two radii are finite and above zero, so a ball and a ring are
        // bounded and a plane is not. A pipe's radius is finite and above zero,
        // so every point of it is that far from its axis, and a plane runs
        // further than that from any line. A cone's slope is finite and above
        // zero, so on the plane square to its axis through its apex the cone
        // has one point, the apex, and a plane meets that plane in a line, in
        // the whole of it, or nowhere. The cone that is a plane, a half-angle
        // of a right angle, is refused by `Cone::from_half_angle`, and its
        // infinite slope by `Cone::new`. Each type checks again when it is read
        // from bytes. `intersect` never answered `SameSurface` for such a pair
        // either, so what skipping it drops is a refusal, or the records of a
        // crossing nothing here used.
        (Some(_), None) | (None, Some(_)) => return Ok(None),
        (None, None) => (),
    }
    if of_two_curved_kinds(&piece.surface, &face.surface) {
        return Ok(None);
    }
    if !matches!(
        intersect(&piece.surface, &face.surface, policy, log)?,
        SurfaceMeet::SameSurface
    ) {
        return Ok(None);
    }
    Ok(Some(piece.orientation == face.orientation))
}

/// Whether two curved surfaces are of two different kinds, which are never one
/// surface, so `intersect` is not asked whether they are.
///
/// `intersect` refuses a ring against a ball, a pipe or a cone, and a cone
/// against a ball, however far apart the two are, and a ball twenty units from
/// a bend refused every fuse for it (`docs/decisions.md` §322.7, Q463).
/// `tests/a_plane_is_never_one_surface_with_a_curved_face.rs` shows each pair
/// of kinds apart: one plane, built from the two, that `intersect` cuts each
/// of them in differently. `intersect` answers `SameSurface` only for a pair of
/// one kind, so what skipping it drops is a refusal, or the records of a
/// crossing nothing here used. Where two such faces really cross, the cutting
/// has already asked for that crossing.
fn of_two_curved_kinds(one: &Surface, other: &Surface) -> bool {
    // A plane is answered by the caller before this. A kind rcad-geom grows
    // later has not been shown apart from these four, so it is still asked
    // about.
    let shown_apart = |surface: &Surface| {
        matches!(
            surface,
            Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_)
        )
    };
    shown_apart(one)
        && shown_apart(other)
        && core::mem::discriminant(one) != core::mem::discriminant(other)
}

/// Whether the material of one face of an input holds a point already known to
/// be on that face's surface.
///
/// A flat face with straight sides is wound, the way it always was, and asked
/// of the model where the winding cannot say
/// ([`the_model_says_one_of_its_faces_holds`]).
///
/// **Its corners are wound round the point's box stretched to the plane**
/// ([`stretched_to_the_plane`]), as [`the_piece_holds`] winds a piece's.
/// [`InPlane`] reads a point off its plane where the line from its reference
/// point through it meets the plane, and `2²⁰` out on a plane through the
/// origin a point a spacing off is read about `1e-4` from its foot. Wound round
/// the point alone, a face held such a point whose foot was off it, and did not
/// hold one whose foot was on it
/// (`a_point_a_rounding_off_a_far_face_is_on_it_where_its_foot_is`,
/// `docs/decisions.md` §346.6). Over the box the fan cannot say for either, and
/// the model answers both. A point on the plane stretches to itself and is
/// wound as before.
///
/// **So are its corners**, each stretched to the plane on its own, for the
/// reason [`the_piece_holds`] gives: a corner the bake solved without this
/// face's plane can lie off it, and is read far from where the face meets its
/// neighbours. A corner that cannot be stretched leaves the point to the model,
/// as an undecided fan does.
///
/// A **curved**
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
///
/// **A face that cannot tell leaves the point off it**, for the next face and
/// then the ray, as [`the_model_says_one_of_its_faces_holds`] does for straight
/// sides. `holds.rs`'s `what_a_flat_face_of_chords_and_arcs_holds` asks which
/// side of every chord the point is on, the chord under each arc too, and a
/// point that is a box a rounding wide across the line of a chord has no side.
/// A block resting on a plate's round tab, with two opposite corners on the
/// line of the chord of the tab's arc, has such a point offered where its
/// diagonal crosses that line, and passed up, the refusal refused the whole
/// fuse under every rule for half of the block's outlines
/// (`tests/a_bar_cut_beside_its_axis_and_united_again.rs`, row Q2036).
///
/// **What the ray then makes of it.** On the face's plane every ray asks this
/// face again whether it holds the point (`classify.rs`'s
/// `ask_whether_the_face_holds_it`), gets the same refusal and takes it as a
/// graze, so no count of crossings places the point; only the plane of a
/// turned flat face beside it could, where that face's corners and plane part
/// about the ray. A rounding off the plane, a point the ray did place is still
/// abandoned unless this face says it holds none of the plane in the point's
/// box ([`a_face_may_run_between_it_and_the_plane`]). An abandoned point leaves
/// the piece to its next one, which on that block is on the other diagonal. In
/// row Q2036's measurement each of the 704 points dropped here, by that test
/// and by probes of blocks along each of the tab's chords, was on the plane
/// and the ray could not tell; no other test in `rcad-topo`, and no file of
/// the `two_solids_fused_by_a_stranger` fuzz corpus, dropped one. The other
/// refusals are about the face's trimming and not about one point, so they
/// still refuse the fuse by name.
///
/// **A curved face of the input can be several faces of the model**, which the
/// bake hands over as one because they are one surface used one way: a bar's
/// wall in two bands on one pipe is one face with a key per band. It holds a
/// point when any of them does. Asked of the first key alone, the wall of a
/// round bar lying on the upper band read as off the bar's wall whenever the
/// lower band was listed first, and every such fuse was refused `Undecidable`
/// at [`PLACED_AGAINST_EVERY_SOLID`] (row Q977).
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
                let mut trial = ProvenanceLog::new();
                match view.face_holds_the_point(key, point, policy, &mut trial) {
                    Ok(said) => {
                        trial.drain_into(log);
                        if said == OnAFace::Held {
                            return Ok(true);
                        }
                    }
                    Err(Error::Undecidable { .. }) => (),
                    Err(refused) => return Err(refused),
                }
            }
            return Ok(false);
        }
        let corners: Option<Vec<Vec<PredicatePoint>>> = face
            .wires
            .iter()
            .map(|wire| {
                wire.iter()
                    .map(|edge| stretched_to_the_plane(flat, edge.start.point))
                    .collect()
            })
            .collect();
        // Either sign means the point is in this face's material. Which one
        // comes out says only which way that face's outline runs round *this*
        // piece's normal, and a face pressed against this one from the other
        // side runs the other way — which is what the agreement above already
        // said. Zero is the only answer that puts the point outside it.
        let winding = InPlane::of(flat)
            .zip(stretched_to_the_plane(flat, point))
            .zip(corners)
            .and_then(|((in_plane, reaching_the_plane), corners)| {
                in_plane.winding_around(&corners, reaching_the_plane)
            });
        if let Some(winding) = winding {
            return Ok(winding != 0);
        }
        return Ok(the_model_says_one_of_its_faces_holds(
            view, face, point, policy, log,
        ));
    }
    for &key in &face.keys {
        if view.face_holds_the_point(key, point, policy, log)? != crate::OnAFace::NotHeld {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether [`SolidView::face_holds_the_point`] says one of the model's faces
/// that `face` was baked from holds `point`, for a flat face with straight
/// sides whose corners [`InPlane::winding_around`] could not wind round the
/// point's box stretched to the plane, or whose point or corners could not be
/// stretched to it.
///
/// **A point a rounding off the plane beside the face's edge comes here**, as
/// the fan over that box runs across the edge, and the model reads it beside
/// its foot: [`SolidView::face_holds_the_point`] counts from a point off the
/// face's first corner, as far off the plane as that corner's largest
/// coordinate, where [`InPlane`] counts from one beside the origin
/// (`a_point_a_rounding_off_a_far_face_is_on_it_where_its_foot_is`).
///
/// **What the fan cannot see.** Every triangle of the fan has its apex at a
/// corner of the face, and a point on a spoke from that apex is on the edge of
/// two triangles, neither of which can say it holds it. The middle of a
/// rectangle is on both its diagonals, so every apex leaves it on a spoke. A
/// bar's cap flush on a block's floor is that case: the one point found inside
/// the cap is its middle, under the middle of the floor, so the cap was never
/// read as lying on the floor, the ray called the point on the block's skin,
/// and the fuse was refused (Q1062). `holds.rs` counts the sides a half-line
/// crosses instead, reading a corner on that line as below it, so it has no
/// spokes to sit on.
///
/// **Why a `Held` puts the piece on the face**, as a winding that is not zero
/// does. The caller proved the piece's plane and the face's plane one plane,
/// and `point` was proved inside the piece. The piece was cut by every surface
/// of the solid, so its inside lies all on the face or all off it (§17.4), and
/// a point of that inside the face's material holds, away from its outline,
/// puts it on the face.
///
/// Only [`OnAFace::Held`](crate::OnAFace::Held) counts, as in the caller. Any
/// other answer, and a refusal, leave the point where the fan left it: off
/// this face, for the next face and then the ray to place. So a point the fan
/// placed is placed as it was, and what the model was asked goes into the log
/// only when it says the face holds the point.
fn the_model_says_one_of_its_faces_holds(
    view: SolidView<'_>,
    face: &BakedFace,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> bool {
    face.keys.iter().any(|&key| {
        let mut trial = ProvenanceLog::new();
        let held = matches!(
            view.face_holds_the_point(key, point, policy, &mut trial),
            Ok(crate::OnAFace::Held)
        );
        if held {
            trial.drain_into(log);
        }
        held
    })
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
///
/// **A point off the plane is read where the line from the reference point
/// through it meets the plane**, not at its foot, and the two can be far apart:
/// `M` out on a plane through the origin, a point a rounding high is read about
/// `M` times that height from its foot. A caller that needs the foot asks about
/// a box that reaches the plane, as [`the_piece_holds`] and
/// [`a_face_of_the_input_holds`] do. **A corner is read the same way**, and a
/// corner's box can lie off the plane as well, so those two stretch each corner
/// to the plane before they wind round it.
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
    /// cannot. `two_towers_1e308_tall_are_united_by_proof_alone` in
    /// `tests/fusing_solids.rs` is where that runs, and the fuse there now
    /// answers: with only the first try it refuses on [`OFF_A_PLANE`] before it
    /// reaches any arithmetic. Stepping towards the origin *first* was measured too, and
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
    /// triangle's own boundary, or when it might be inside and a turn could not
    /// be decided.
    ///
    /// **Two turns of opposite sign put the point outside, whatever the third
    /// one is**, so `0` does not wait for the third to be decided. A point
    /// inside the triangle or on its boundary turns the same way about all
    /// three sides, or not at all about some of them, and never one way about
    /// one side and the other way about another; and every sign the third
    /// turn could have gives `0` here once the other two disagree. That is the
    /// piece three cut lines meeting at one point leave (Q1145): one side of it
    /// is a rounding long, every fan from every corner holds a triangle on that
    /// side, and the turn about it is undecidable for every point there is.
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
        let mut undecided = 0;
        for [from, to] in [[first, second], [second, third], [third, first]] {
            match self.turn(from, to, point) {
                Decision::Pos => positives += 1,
                Decision::Neg => negatives += 1,
                Decision::Zero => zeros += 1,
                Decision::Undecidable { .. } => undecided += 1,
                _ => return None,
            }
        }
        if positives > 0 && negatives > 0 {
            return Some(0);
        }
        if zeros > 0 || undecided > 0 {
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
/// **A ring between two such curves has neither**, and the middles of both can
/// be in its hole. What reaches it is a point between the two curves along a
/// line through one of their middles ([`between_the_round_wires`]), offered
/// after every other candidate here, so a piece one of those placed is placed
/// as it was.
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
        candidates.extend(between_the_round_wires(flat, piece));
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
    candidates.extend(steps_from_the_middle(piece, middle, reach));
    // **A step of one radius towards a cone's apex goes past it** wherever the
    // cone's radius is more than its height, and lands on the other half of
    // the cone, where the piece is not (§350). Half way from the corners'
    // middle to the apex is on the corners' own half, whatever the slope: the
    // corners are on it, and so is the apex.
    if let Surface::Cone(cone) = &piece.surface {
        candidates.push(midway_between(
            PredicatePoint::from(middle.coordinates()),
            PredicatePoint::from(cone.apex().coordinates()),
        ));
    }
    candidates
        .into_iter()
        .filter_map(|candidate| back_onto_the_surface(&piece.surface, candidate))
        .collect()
}

/// The steps [`candidate_points`] takes across a curved piece from the middle
/// of its corners, taken again at a half, a quarter, an eighth and a sixteenth
/// of the length, each put back onto the surface. Nothing for a flat piece,
/// whose last points are [`rungs_towards_each_crossing`].
///
/// **A step of one radius can be too long for every way it goes.** A block
/// over a frustum's rim, its wall at `x = 2` and its floor at `z = 5`, has its
/// wall's plane cut the cone below the floor in a hyperbola whose lowest point
/// is `(2, 0, 4)`. The piece between that and the circle at `z = 5` has two
/// corners, `(2, ±1.5, 5)`, and is nowhere more than one unit deep, while the
/// cone there is `2.5` across: every step of `2.5` from `(2, 0, 5)`, put back
/// on the cone, lands on that circle or outside the piece. Half the step
/// towards the side of both planes the piece is on, `(2.88, 0, 4.12)`, comes
/// back as `(2.06, 0, 4.12)`, inside it. A single block finds its way round
/// that by a side the piece shares with it (§365.3), but placed against the
/// block and a second solid together, even a cube far away, it has no side to
/// share with the second, and its corners lie on the block's edge, where no
/// corner places it: such fuses were refused `"a point inside a piece of a
/// face"` (`tests/a_frustums_wall_cut_below_a_blocks_floor.rs`).
///
/// The lengths are the reach halved, which is exact, and the steps are tried
/// only where no other point was found inside the piece, after every other
/// point, by [`place_the_piece`]'s rule for a rung: so a piece any earlier
/// point placed is placed by that point as it was.
fn shorter_steps_across_a_curved_piece(piece: &Piece) -> Vec<PredicatePoint> {
    if piece.surface.as_plane().is_some() {
        return Vec::new();
    }
    let middle = middle_of_the_corners(&corners_of(piece));
    let reach = how_big_the_surface_is(&piece.surface, middle);
    [2.0, 4.0, 8.0, 16.0]
        .into_iter()
        .flat_map(|shorter| steps_from_the_middle(piece, middle, reach / shorter))
        .filter_map(|step| back_onto_the_surface(&piece.surface, step))
        .collect()
}

/// For a piece of a cone, the point half way between each two places where a
/// side cut by a plane is crossed square to its chord, put back onto the cone;
/// offered after [`shorter_steps_across_a_curved_piece`], under the same rule.
///
/// **Every corner of a piece can lie on one circle while the piece hangs below
/// it.** The nested cups of
/// `tests/a_joined_flat_face_whose_rings_enclose_no_area.rs` have their second
/// cone's skin cut by the cups' wall `x = −12` and a flush block's wall
/// `x = −11`: the piece between them is a band hanging off the circle of radius
/// 14 at `z = 16`, its four corners all on that circle and its two hyperbola
/// sides reaching down to `(−12, 0, 14)` and `(−11, 0, 13)`. Every blend of the
/// corners comes back onto that circle, which is the piece's own side, and the
/// steps from their middle go down the cone by `14/2ᵏ`, while the band at
/// `y = 0` is between `z = 13` and `14`, two to three below them: no point
/// offered was inside it, and every fuse with the cups first was refused
/// [`INSIDE_A_PIECE`] (row Q1224).
///
/// A side's curve is crossed square to its chord where
/// [`half_way_across_the_chord_to_a_cone`] aims, once on each nappe for a
/// hyperbola, and half way between the two sides' crossings on the band's own
/// nappe is `(−11.5, 0, 13.5)`, which is on the cone exactly and inside the
/// band. Nothing is trusted: the caller proves each point inside the piece or
/// drops it.
fn between_the_middles_of_a_cone_pieces_sides(piece: &Piece) -> Vec<PredicatePoint> {
    let Surface::Cone(cone) = &piece.surface else {
        return Vec::new();
    };
    let mut crossed = Vec::new();
    for side in piece.wires.iter().flatten() {
        let Some(flat) = side.carried_by[1].as_plane() else {
            continue;
        };
        let Some(chord) = Chord::of(flat, side) else {
            continue;
        };
        crossed.extend(
            chord
                .where_the_cone_crosses(cone)
                .into_iter()
                .map(|step| chord.middle + chord.across * step)
                .filter(Point::is_finite),
        );
    }
    let mut candidates = Vec::new();
    for (index, &one) in crossed.iter().enumerate() {
        for &other in &crossed[index + 1..] {
            candidates.push(PredicatePoint::from(
                (one + (other - one) * 0.5).coordinates(),
            ));
        }
    }
    candidates
        .into_iter()
        .filter_map(|candidate| back_onto_the_surface(&piece.surface, candidate))
        .collect()
}

/// For a curved piece a plane cut, the steps [`candidate_points`] takes from
/// the middle of its corners, taken again from one reach on from that middle
/// towards the side of every plane that cut it that it is on
/// ([`towards_every_flat_sides_material`]), each put back onto the surface;
/// offered after [`between_the_middles_of_a_cone_pieces_sides`], under the same
/// rule.
///
/// **A plane through a ball's centre keeps every step along it on the circle
/// it cuts the ball in.** The ball of radius 3 at `(0, 3, 5)` against the bar
/// of radius 2 up `z` capped at `z = 5` has its skin cut by the cap's plane and
/// by the one loop the bar's pipe crosses it in, into four pieces, each between
/// an arc of the circle at `z = 5` and an arc of the loop, and all four with
/// the same two corners, `(±4√2/3, 2/3, 5)`. Their middle is on the plane and so
/// is the ball's centre, so a step along `x` or `y`, put back onto the ball,
/// lands on that circle, which bounds every piece. The steps along `z`, and the
/// one towards the plane's material, land above or below the middle, inside
/// the pipe: in the two pieces inside it and in neither of the two outside it.
/// No point offered was inside those two, and every rule both ways round was
/// refused [`INSIDE_A_PIECE`], as it was for the bar from `z = 5` up and the
/// bar cut down to `z = 5` by a block (row Q1861,
/// `tests/two_arcs_where_two_curved_surfaces_cross.rs`). From one reach below
/// the middle the step along `x` comes back as `(1.86, 1.55, 3.14)`, outside the
/// pipe and below the plane, and from one reach above it as
/// `(1.86, 1.55, 6.86)`.
///
/// Nothing is trusted: the caller proves each point inside the piece or drops
/// it.
fn steps_from_a_reach_into_the_planes_material(piece: &Piece) -> Vec<PredicatePoint> {
    if piece.surface.as_plane().is_some() {
        return Vec::new();
    }
    let Some(towards) = towards_every_flat_sides_material(piece) else {
        return Vec::new();
    };
    let middle = middle_of_the_corners(&corners_of(piece));
    let reach = how_big_the_surface_is(&piece.surface, middle);
    steps_from_the_middle(piece, middle + towards * reach, reach)
        .into_iter()
        .filter_map(|step| back_onto_the_surface(&piece.surface, step))
        .collect()
}

/// For a piece of a ring, the point half way round the tube between each two
/// corners and the point across the tube from it, at the angle round the ring
/// half way between them, each put back onto the ring; offered after
/// [`steps_from_a_reach_into_the_planes_material`], under the same rule.
///
/// **Every step from the middle of a band's corners lands on the inner half
/// of the tube.** A whole ring about `x`, 4 round a tube of 1, against a ball
/// of radius 4 at `(1, 0, 0)` is cut in two circles round `x`, one at `x = 1`
/// through the ball's poles `(1, 0, ±4)` and one at `x = −15/17` through
/// `(−15/17, 0, 60/17)`. The band outside the ball runs from the second circle
/// round the outside of the tube to the first. The middle of its three corners
/// is near the axis, and each step from it, put back onto the ring, comes back
/// as the nearest point of the tube, on its inner half and inside the ball:
/// no point offered was inside the band, and every rule both ways round was
/// refused [`INSIDE_A_PIECE`] (row Q2205,
/// `tests/a_walk_neither_winding_nor_the_axis_places.rs`). So was a ball of 3
/// at `(0.75, 0, 0)` pressed into a ring 4 round a tube of 1.25.
///
/// Half way round the tube between two corners is one of the two places
/// between them on the circle round the tube, and the point across the tube
/// from it is the other, so whichever way the band runs one of the two is in
/// it: from `(1, 0, 4)` and `(−15/17, 0, 60/17)` that is `(−0.24, 0, 4.97)`.
/// Nothing is trusted: the caller proves each point inside the piece or drops
/// it.
fn half_way_round_the_tube_between_two_corners(piece: &Piece) -> Vec<PredicatePoint> {
    let Surface::Torus(ring) = &piece.surface else {
        return Vec::new();
    };
    let corners: Vec<(f64, f64)> = corners_of(piece)
        .concat()
        .into_iter()
        .map(|corner| ring.coordinates_of(middle_of(corner)))
        .collect();
    let mut candidates = Vec::new();
    for (index, &(over, round)) in corners.iter().enumerate() {
        for &(other_over, other_round) in &corners[index + 1..] {
            let half_way_over = (over + other_over) / 2.0;
            let half_way_round = (round + other_round) / 2.0;
            for across in [0.0, core::f64::consts::PI] {
                candidates.push(PredicatePoint::from(
                    ring.point_at(half_way_over + across, half_way_round)
                        .coordinates(),
                ));
            }
        }
    }
    candidates
        .into_iter()
        .filter_map(|candidate| back_onto_the_surface(&piece.surface, candidate))
        .collect()
}

/// Six steps of `reach` from `middle`, one each way along each axis, and one
/// more towards the side of every plane that cut the piece that the piece is
/// on ([`towards_every_flat_sides_material`]), none of them put back onto the
/// piece's surface yet.
fn steps_from_the_middle(piece: &Piece, middle: Point, reach: f64) -> Vec<PredicatePoint> {
    let mut steps = Vec::new();
    for axis in 0..3 {
        for way in [1.0, -1.0] {
            let mut step = [0.0; 3];
            step[axis] = way * reach;
            steps.push(PredicatePoint::from(
                [0, 1, 2].map(|which| middle.coordinates()[which] + step[which]),
            ));
        }
    }
    if let Some(towards) = towards_every_flat_sides_material(piece) {
        steps.push(PredicatePoint::from(
            (middle + towards * reach).coordinates(),
        ));
    }
    steps
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
    let Some(chord) = Chord::of(flat, side) else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    for both in chord.where_the_circles_cross(flat, &side.carried_by[1]) {
        let points = both.map(|to| chord.middle + chord.across * (to / 2.0));
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
///
/// **A plane through the apex makes the two roots one**, and a discriminant
/// below zero is read as that one root rather than as no crossing. Such a plane
/// cuts the cone in two rulings, and where both reach the same cap the piece
/// between them and the cap's chord is a triangle with the apex for its third
/// corner — a corner nothing names, so the triangle's two named corners wind
/// round nothing. The line square to that chord through its middle runs through
/// the apex, where it touches the cone twice over, and rounding puts the
/// discriminant either side of zero. Below it, the pointed cone of
/// `tests/a_plane_cut_along_a_cones_other_nappe.rs` had no candidate inside the
/// triangle and refused eight fuses on [`INSIDE_A_PIECE`] (§282). Where the
/// line misses the cone the step lands somewhere on it that the caller then
/// proves or drops, like any other.
fn half_way_across_the_chord_to_a_cone(flat: &Plane, side: &FaceMeet) -> Vec<PredicatePoint> {
    let Surface::Cone(cone) = side.carried_by[1] else {
        return Vec::new();
    };
    let Some(chord) = Chord::of(flat, side) else {
        return Vec::new();
    };
    chord
        .where_the_cone_crosses(&cone)
        .into_iter()
        .map(|step| chord.middle + chord.across * (step / 2.0))
        .filter(Point::is_finite)
        .map(|point| PredicatePoint::from(point.coordinates()))
        .collect()
}

/// Points ever nearer each place [`half_way_into_either_arc`] and
/// [`half_way_across_the_chord_to_a_cone`] go half way to: a quarter of the way
/// back from it towards the chord's middle, an eighth, and so on, and as far
/// again past it where the side's material is outside its partner. Nothing for
/// a curved piece, whose last points are [`shorter_steps_across_a_curved_piece`].
///
/// **Half way can be in a hole when the side is long.** A tube's cap less a bar
/// across its wall is a C (`tests/a_tube_less_a_bar_united_with_a_second_bar.rs`):
/// the outer arc runs the long way round from `(2.6875, ±1.3332)`, so half way
/// from its chord's middle `(2.6875, 0)` to `(−3, 0)` is `(−0.15625, 0)`, in the
/// tube's hole, and the other way it is in the bar. Every point offered before
/// was in the one or the other, and every fuse of the two refused. The first
/// rung in the C is half as far again past the bar's arc as its chord's middle
/// is from it, `(1.5028, ±1.6382)`; a quarter of the way back from `(−3, 0)`,
/// `(−1.578125, 0)`, is in it too.
///
/// Near a crossing on the side's own arc the piece is on the side
/// `material_side` names, so some rung is in it wherever the piece is wider
/// there than the rung is near. **The ladder stops at `2⁻⁵²` of the way**,
/// where a rung is within a rounding of the crossing it is built from and the
/// next could not be told from it; a piece thinner than that beside every
/// crossing gets the points it got before, and is refused by the same name.
/// The shallow rungs of every side come before the deep ones of any, since a
/// rung nearer a partner is harder to classify against the solid it bounds.
///
/// These are offered after every other point, even [`points_in_a_sliver_of_a_ball`],
/// so they place only pieces nothing else did (`docs/decisions.md` §365.6).
fn rungs_towards_each_crossing(piece: &Piece) -> Vec<PredicatePoint> {
    let Some(flat) = piece.surface.as_plane() else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    for side in piece.wires.iter().flatten() {
        let Some(chord) = Chord::of(flat, side) else {
            continue;
        };
        let mut crossings: Vec<f64> = chord
            .where_the_circles_cross(flat, &side.carried_by[1])
            .into_iter()
            .flatten()
            .collect();
        if let Surface::Cone(cone) = &side.carried_by[1] {
            crossings.extend(chord.where_the_cone_crosses(cone));
        }
        let material_is_outside = side.material_side == Decision::Pos;
        lines.push((chord, crossings, material_is_outside));
    }
    let mut rungs = Vec::new();
    let mut back = 1.0;
    for depth in 1..f64::MANTISSA_DIGITS {
        back /= 2.0;
        for (chord, crossings, material_is_outside) in &lines {
            for &crossing in crossings {
                if depth > 1 {
                    rungs.push(chord.middle + chord.across * (crossing * (1.0 - back)));
                }
                if *material_is_outside {
                    rungs.push(chord.middle + chord.across * (crossing * (1.0 + back)));
                }
            }
        }
    }
    rungs
        .into_iter()
        .filter(Point::is_finite)
        .map(|point| PredicatePoint::from(point.coordinates()))
        .collect()
}

/// The middle of a curved side's chord, and the way square to the chord in the
/// piece's plane.
struct Chord {
    middle: Point,
    across: Vector,
    length_squared: f64,
}

impl Chord {
    /// `None` for a side with no length to go square to.
    fn of(flat: &Plane, side: &FaceMeet) -> Option<Self> {
        let [start, end] = [0, 1].map(|which| middle_of(side.ends[which].point()));
        let chord = end - start;
        let across = flat.normal().cross(chord).normalized().ok()?;
        Some(Self {
            middle: start + chord * 0.5,
            across,
            length_squared: chord.length_squared(),
        })
    }

    /// How far along [`Chord::across`] from the middle the circle round each of
    /// [`the_middles_of_a_round_wire`] through the chord's two ends crosses,
    /// both ways.
    fn where_the_circles_cross(&self, flat: &Plane, partner: &Surface) -> Vec<[f64; 2]> {
        the_middles_of_a_round_wire(flat, partner)
            .into_iter()
            .map(|centre| {
                let off = self.across.dot(middle_of(centre) - self.middle);
                let reach = (off * off + self.length_squared / 4.0).sqrt();
                [off + reach, off - reach]
            })
            .collect()
    }

    /// How far along [`Chord::across`] from the middle the line crosses
    /// `cone`, as [`half_way_across_the_chord_to_a_cone`] reads it.
    fn where_the_cone_crosses(&self, cone: &Cone) -> Vec<f64> {
        let up = cone.axis().direction();
        let widening = 1.0 + cone.slope() * cone.slope();
        let from_apex = self.middle - cone.apex();
        let (along_now, along_per_step) = (from_apex.dot(up), self.across.dot(up));
        let square = widening * along_per_step * along_per_step - up.length_squared();
        let linear = 2.0
            * (widening * along_now * along_per_step
                - up.length_squared() * from_apex.dot(self.across));
        let constant =
            widening * along_now * along_now - up.length_squared() * from_apex.length_squared();
        if square == 0.0 {
            return vec![-constant / linear];
        }
        let discriminant = linear * linear - 4.0 * square * constant;
        let root = discriminant.max(0.0).sqrt();
        vec![
            (-linear + root) / (2.0 * square),
            (-linear - root) / (2.0 * square),
        ]
    }
}

/// Points between the round wires of a flat piece: along the line from each
/// round wire's middle through its corner, the middle of every stretch between
/// two places, one after the other, where that line crosses a round wire.
///
/// **A ring has no blend of corners inside it, and its middles can all be in
/// its hole.** The ring a hole leaves in a round cap is bounded by two whole
/// circles with one corner each, so its corner blends are its two corners and
/// the one point half way between them, and the middle of each circle is
/// inside the hole wherever the hole covers the cap's middle. A plate of radius
/// 10 round `(−6, 0, 0)` with a hole of radius 5 round `(−3, 0, 0)` had that
/// one blend, `(0.5, −2.5, 0)`, in the hole too, and every fuse of the two
/// refused (`tests/a_plate_with_an_off_centre_hole.rs`, `docs/decisions.md`
/// §365.2).
///
/// A line through a hole's middle leaves the hole and then the outline, once
/// each way along it, since a disk is convex and the hole is inside the
/// outline, so the stretch between the two is the ring. Each round wire is read
/// as the circle round its middle through its corner, which is exactly what a
/// ball or a pipe square to the plane cuts and only near what a leaning pipe or
/// a cone does; nothing about a candidate is trusted, so a wire read wrongly
/// only costs candidates.
///
/// **Two whole circles can be one wire**, where they touch at a corner they
/// share. A thin bar's cap flush with a thick bar's, the two pipes touching on
/// the thick bar's seam, leaves the thick cap a crescent whose one wire is the
/// rim and the thin circle, each from the seam corner back to it. That wire is
/// not round ([`round_wire_of`]), every blend of its corners is the seam
/// corner, and every side's chord has no length, so until each such side was
/// read here as a round wire of its own no point was offered inside it and
/// every fuse was refused [`INSIDE_A_PIECE`] (row Q1748). The line from the
/// rim's middle through the seam corner crosses the rim at the corner and
/// across from it, and the thin circle at the corner and one diameter in from
/// it, so the middle of the stretch from across the rim to that second
/// crossing is inside the crescent: `(−1, 0, 0)` for radii three and one. Such
/// sides come after the round wires, so a piece with none is offered what it
/// was. Nothing for a piece with fewer than two round wires and such sides
/// together, whose one middle is already offered.
fn between_the_round_wires(flat: &Plane, piece: &Piece) -> Vec<PredicatePoint> {
    let mut through_its_corner = Vec::new();
    for wire in &piece.wires {
        let Some(partner) = round_wire_of(wire) else {
            continue;
        };
        let corner = middle_of(wire[0].ends[0].point());
        for middle in the_middles_of_a_round_wire(flat, &partner) {
            through_its_corner.push((middle_of(middle), corner));
        }
    }
    let whole_circles_in_a_wire_of_more = piece
        .wires
        .iter()
        .filter(|wire| round_wire_of(wire).is_none())
        .flatten()
        .filter(|side| closes_on_itself(side));
    for side in whole_circles_in_a_wire_of_more {
        let corner = middle_of(side.ends[0].point());
        for middle in the_middles_of_a_round_wire(flat, &side.carried_by[1]) {
            through_its_corner.push((middle_of(middle), corner));
        }
    }
    if through_its_corner.len() < 2 {
        return Vec::new();
    }
    let mut candidates = Vec::new();
    for &(from, corner) in &through_its_corner {
        let Ok(along) = (corner - from).normalized() else {
            continue;
        };
        let mut crossings: Vec<f64> = through_its_corner
            .iter()
            .flat_map(|&(middle, on_it)| {
                let off = from - middle;
                let half_way = along.dot(off);
                let below =
                    half_way * half_way - off.length_squared() + (on_it - middle).length_squared();
                let reach = below.sqrt();
                [-half_way - reach, -half_way + reach]
            })
            .filter(|step| step.is_finite())
            .collect();
        crossings.sort_by(f64::total_cmp);
        candidates.extend(
            crossings
                .windows(2)
                .map(|both| from + along * ((both[0] + both[1]) / 2.0))
                .filter(Point::is_finite)
                .map(|point| PredicatePoint::from(point.coordinates())),
        );
    }
    candidates
}

/// Whether `side` runs right round a closed curve, from one corner back to it.
fn closes_on_itself(side: &FaceMeet) -> bool {
    side.ends[0].point() == side.ends[1].point()
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
/// **A torus has two answers on a plane through its axis, and both are
/// offered.** A plane cuts a torus in up to four circles, and where it goes
/// through the torus's axis it cuts two, one round each place the ring crosses
/// the plane. Which of the two a given wire goes round is not known here, and
/// does not need to be: nothing about a candidate is trusted, so the one that
/// is not inside is ruled out by the same count that proves the other in.
///
/// **On a plane square to its axis it has one**, the foot of the torus's
/// centre: the two circles it cuts there are both round the axis. The point is
/// in the hole and holds nothing, and what it is for is the arcs. Without it
/// the half ring a half bend's floor is on had no middle to step from, so
/// [`half_way_into_either_arc`] and [`rungs_towards_each_crossing`] offered
/// nothing off the `x` axis its four corners lie on (row Q1268).
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
        Surface::Torus(ring)
            if lie_along_one_line(normal.components(), ring.axis().direction().components()) =>
        {
            vec![ring.centre()]
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
///
/// **The box is sized by the numbers the round trip goes through, and by
/// nothing else** — [`Surface::ROUND_TRIP_ROUNDOFF`]'s own size: the point on
/// the surface, the numbers that place the surface, and its radius. The
/// candidate before its trip is not one of them. It is off the surface, and
/// the box only has to hold the true point at the two coordinates it came back
/// with, whatever those are. Sizing by it too grew one ball's box from
/// `1.14e-13` to `1.71e-13` and lost the only candidate inside a cap in
/// `every_model_the_fuse_fuzzer_kept_is_fused_again.rs`. It used to be the point's
/// size floored at one, which is wrong both ways. A ball of radius `4.6e-14` on
/// a cube's corner got a box `1.1e-13` wide, so no box fitted in the piece of
/// it inside the cube and every fuse of the two refused
/// (`tests/two_meets_reach_one_corner.rs`, `docs/decisions.md` §346). And a
/// pipe placed from an axis origin far from the point got a box sized by the
/// point alone.
///
/// Scaling a surface and a point by a power of two scales every step of the
/// round trip exactly, as long as nothing in it is subnormal, so the bound
/// holds at any size the way it holds at the sizes it was measured at. The
/// floor is where that stops: below [`SMALLEST_SIZE_A_ROUND_TRIP_SCALES_FROM`]
/// a rounding can cost an absolute `2⁻¹⁰⁷⁵` instead of a fraction of the number.
fn back_onto_the_surface(surface: &Surface, candidate: PredicatePoint) -> Option<PredicatePoint> {
    let at = middle_of(candidate);
    let (u, v) = surface.coordinates_of(at);
    let there = surface.point_at(u, v);
    if !there.is_finite() {
        return None;
    }
    let size = the_largest_number_placing(surface)?
        .max(there.largest_coordinate_magnitude())
        .max(SMALLEST_SIZE_A_ROUND_TRIP_SCALES_FROM);
    let reach = Surface::ROUND_TRIP_ROUNDOFF * size;
    Some(PredicatePoint::Constructed(there.coordinates().map(
        |coordinate| {
            Interval::around(coordinate, reach)
                .expect("the check just above sent every point that is not a place back as None")
        },
    )))
}

/// The size below which [`back_onto_the_surface`] stops shrinking its box.
///
/// `2⁻⁹⁷⁰`. At that size the box reaches `2⁻¹⁰¹⁴`, which is `2⁶¹` times the most
/// one rounding of a subnormal number can cost, and a round trip is a few dozen
/// roundings.
const SMALLEST_SIZE_A_ROUND_TRIP_SCALES_FROM: f64 = f64::MIN_POSITIVE / f64::EPSILON;

/// The largest of the numbers a curved surface is placed and sized with, which
/// is what [`Surface::ROUND_TRIP_ROUNDOFF`] is stated against together with
/// the point. The same list
/// `rcad-geom/tests/a_round_trip_through_a_surfaces_own_two_coordinates.rs`
/// measures the bound against. `None` for a plane, which is never put back
/// onto itself here.
fn the_largest_number_placing(surface: &Surface) -> Option<f64> {
    match surface {
        Surface::Sphere(ball) => Some(
            ball.centre()
                .largest_coordinate_magnitude()
                .max(ball.radius()),
        ),
        Surface::Cylinder(pipe) => Some(
            pipe.axis()
                .origin()
                .largest_coordinate_magnitude()
                .max(pipe.radius()),
        ),
        Surface::Cone(cone) => Some(cone.apex().largest_coordinate_magnitude().max(cone.slope())),
        Surface::Torus(ring) => Some(
            ring.centre()
                .largest_coordinate_magnitude()
                .max(ring.ring_radius() + ring.tube_radius()),
        ),
        Surface::Plane(_) | _ => None,
    }
}

/// Points inside a piece cut from a ball, or from a plane inside a ball, that
/// is far thinner than the box [`back_onto_the_surface`] puts a point in; tried
/// only once no point [`candidate_points`] offered was inside the piece.
///
/// **A plane that only just cuts a ball leaves two such pieces.** A block
/// turned about `z` beside a ball that should touch its face at one point has
/// that face's plane stored a rounding off, and it cuts the ball in a circle
/// of radius about `1e-8`, taking off a cap `1.3e-17` to `8.5e-16` deep
/// (`tests/a_ball_shaved_by_a_turned_blocks_face.rs`, `docs/decisions.md`
/// §346.6). A point put back on the ball comes back as a box `±5.7e-14` wide,
/// 67 to 4400 times as deep as the cap, so no side of the cap can place it and
/// the cap held none of them. The disk the ball leaves on the plane fares no
/// better: the one point offered in the middle of it, a double, came out a
/// rounding outside the ball.
///
/// Unlike §346.3's sliver, these hold points with `f64` coordinates: `1e-8`
/// long and a fraction of a spacing deep still crosses many doubles, and a
/// walk along the cut meets them. [`doubles_in_a_sliver`] finds a few, each
/// **proved** inside the ball and on the right side of every cut, and
/// [`segments_up_through_a_cap`] and [`points_beyond_a_plane_in_a_ball`] say
/// what each piece makes of them.
fn points_in_a_sliver_of_a_ball(piece: &Piece) -> Vec<PredicatePoint> {
    match &piece.surface {
        Surface::Sphere(ball) => segments_up_through_a_cap(piece, ball),
        Surface::Plane(flat) => points_beyond_a_plane_in_a_ball(piece, flat),
        Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_) | _ => Vec::new(),
    }
}

/// For a piece of a ball's skin whose every side a plane cut, segments along
/// an axis no cutting plane's normal has a part along, each from a double
/// point proved inside the ball to one proved outside it.
///
/// **A segment is a box that holds a point of the ball**: its two ends are on
/// either side of the ball's surface, so the surface crosses it. And every
/// point of it is on the same side of each cutting plane, exactly, because a
/// plane with no part along the segment cannot tell its points apart. So the
/// sides [`the_piece_holds`] asks decide on it as they would on one double,
/// where on a box as wide as the round trip's they decided nothing.
fn segments_up_through_a_cap(piece: &Piece, ball: &Sphere) -> Vec<PredicatePoint> {
    let mut cuts = Vec::new();
    for side in piece.wires.iter().flatten() {
        let Some(flat) = side.carried_by[1].as_plane() else {
            return Vec::new();
        };
        cuts.push((*flat, side.material_side));
    }
    let Some(up) = (0..3).find(|&axis| {
        cuts.iter()
            .all(|(flat, _)| flat.normal().components()[axis] == 0.0)
    }) else {
        return Vec::new();
    };
    let height = ball.centre().coordinates()[up];
    doubles_in_a_sliver(ball, &cuts, up, height)
        .into_iter()
        .filter_map(|inside| a_segment_out_of_the_ball(ball, inside, up))
        .collect()
}

/// The segment from `inside` along `up` to the first double point proved
/// outside `ball`, or `None` when no step reached one.
fn a_segment_out_of_the_ball(ball: &Sphere, inside: [f64; 3], up: usize) -> Option<PredicatePoint> {
    let centre = ball.centre().coordinates();
    let aside: f64 = (0..3)
        .filter(|&axis| axis != up)
        .map(|axis| (inside[axis] - centre[axis]).powi(2))
        .sum();
    let mut reach =
        (2.0 * (ball.radius().powi(2) - aside).max(0.0).sqrt()).max(ball.radius() * f64::EPSILON);
    for _ in 0..64 {
        let mut outside = inside;
        outside[up] += reach;
        if ball.side_of(PredicatePoint::from(outside)) == Decision::Pos {
            let mut segment = inside.map(Interval::exactly);
            segment[up] = Interval::new(inside[up], outside[up])
                .expect("an end with the other sign is a step above the one inside");
            return Some(PredicatePoint::Constructed(segment));
        }
        reach *= 2.0;
    }
    None
}

/// For a flat piece bounded by a ball's circle, double points on the far side
/// of the piece's plane from the ball's centre, proved inside the ball.
///
/// **The far side is what makes such a point stand for the disk.** The disk is
/// the part of the plane inside the ball, and a point's foot on the plane is
/// nearer the centre than the point itself whenever the point is on the far
/// side, so a point there inside the ball has its foot inside the ball too,
/// which is inside the disk. A point on the near side proves nothing of the
/// kind: the ball is wider there than on the plane, and a point inside it can
/// have its foot outside the disk
/// (`a_point_inside_a_ball_whose_foot_is_outside_its_disk_is_not_held`). The
/// same argument is what lets [`side_of_a_partner_by_its_plane`] read the ball
/// at such a point alone.
fn points_beyond_a_plane_in_a_ball(piece: &Piece, flat: &Plane) -> Vec<PredicatePoint> {
    let balls: Vec<Sphere> = piece
        .wires
        .iter()
        .filter_map(|wire| match round_wire_of(wire) {
            Some(Surface::Sphere(ball)) => Some(ball),
            Some(_) | None => None,
        })
        .collect();
    let [ball] = balls[..] else {
        return Vec::new();
    };
    let far_side = match flat.side_of_point(PredicatePoint::from(ball.centre().coordinates())) {
        Decision::Pos => Decision::Neg,
        Decision::Neg => Decision::Pos,
        Decision::Zero | Decision::Undecidable { .. } | _ => return Vec::new(),
    };
    let normal = flat.normal().components();
    let held = (0..3)
        .min_by(|&one, &other| normal[one].abs().total_cmp(&normal[other].abs()))
        .expect("three axes");
    let foot = flat.project(ball.centre()).coordinates();
    doubles_in_a_sliver(&ball, &[(*flat, far_side)], held, foot[held])
        .into_iter()
        .map(PredicatePoint::from)
        .collect()
}

/// Up to four double points proved inside `ball` and on the given side of each
/// of `cuts`, with coordinate `held` at `height`.
///
/// **Found by walking along each cut from where the ball's centre is nearest
/// it**, one spacing of doubles at a time in one of the other two coordinates,
/// and at each step trying the few doubles next to the cut in the third. The
/// sliver runs along the cut for its whole length and can be a fraction of a
/// spacing deep, so a step may hold no double inside it; the walk takes up to
/// 64 each way, and at the thinnest cap measured, `1.3e-17` deep, it found
/// points the cap holds. Nothing found is trusted without the two signs, so where the walk
/// goes decides nothing (§5).
fn doubles_in_a_sliver(
    ball: &Sphere,
    cuts: &[(Plane, Decision)],
    held: usize,
    height: f64,
) -> Vec<[f64; 3]> {
    const STEPS_EACH_WAY: usize = 64;
    const WANTED: usize = 4;
    let [one, other] = match held {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    let inside_everything = |at: [f64; 3]| {
        let point = PredicatePoint::from(at);
        ball.side_of(point) == Decision::Neg
            && cuts
                .iter()
                .all(|(flat, side)| flat.side_of_point(point) == *side)
    };
    let mut found = Vec::new();
    for (cut, _) in cuts {
        let normal = cut.normal().components();
        let [solved, walked] = if normal[one].abs() >= normal[other].abs() {
            [one, other]
        } else {
            [other, one]
        };
        if normal[solved] == 0.0 {
            continue;
        }
        let foot = cut.project(ball.centre()).coordinates();
        let mut ahead = foot[walked];
        let mut behind = foot[walked].next_down();
        for _ in 0..STEPS_EACH_WAY {
            for along in [ahead, behind] {
                let on_the_cut = (cut.offset() - normal[walked] * along - normal[held] * height)
                    / normal[solved];
                let mut at = [0.0; 3];
                at[held] = height;
                at[walked] = along;
                let mut near = on_the_cut.next_down().next_down();
                for _ in 0..6 {
                    at[solved] = near;
                    if inside_everything(at) {
                        found.push(at);
                        if found.len() == WANTED {
                            return found;
                        }
                    }
                    near = near.next_up();
                }
            }
            ahead = ahead.next_up();
            behind = behind.next_down();
        }
    }
    found
}

/// Halfway between two points, which is exact whenever both ends are and the
/// middle is a double.
///
/// **Two ways to halve, and each is exact where the other is not.** Adding
/// first overflows once the two ends add up past the largest double: two
/// corners at `z = 1e308` make `2e308`, and every point blended from a tower's
/// top came back as a box reaching up to infinity, which no winding could
/// place (`docs/decisions.md` §353). Halving first loses the last bit of a
/// subnormal end. Both enclose the same middle, so the part they share does
/// too, and it is one double whenever either of them was.
fn midway_between(one: PredicatePoint, other: PredicatePoint) -> PredicatePoint {
    let (one, other) = (one.enclosure(), other.enclosure());
    let half = Interval::exactly(0.5);
    as_exact_as_it_is(PredicatePoint::Constructed([0, 1, 2].map(|axis| {
        let added_first = (one[axis] + other[axis]) * half;
        let halved_first = one[axis] * half + other[axis] * half;
        Interval::new(
            added_first.lo().max(halved_first.lo()),
            added_first.hi().min(halved_first.hi()),
        )
        .expect("two enclosures of one middle both hold it")
    })))
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
    use rcad_geom::{
        Axis, Cone, Cylinder, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface,
        SurfaceMeet, Torus, Vector, intersect,
    };
    use rcad_predicates::{
        Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface,
    };
    use smallvec::SmallVec;

    use super::{
        InPlane, Piece, Rule, a_face_of_the_input_holds, by_the_one_face_its_box_meets,
        candidate_points, choose, corners_of, every_picker_keeps, midway_between,
        the_meridian_circle_each_side_runs_round, the_middles_of_a_round_wire,
        the_pickers_of_the_meridian_circle_through, the_piece_holds,
        what_else_each_round_wire_asks,
    };
    use crate::fuse::bake::{BakedEdge, BakedFace, BakedSolid, bake};
    use crate::fuse::{FaceMeet, MeetPoint, Scratch};
    use crate::{Model, ProfileBuilder, SolidKey};

    /// Adding first overflows here: `1e308 + 1.5e308` is past the largest
    /// double, and the middle, `1.25e308`, is not.
    #[test]
    fn the_middle_of_two_corners_past_half_the_largest_double_is_exact() {
        let middle = midway_between(
            PredicatePoint::from([0.5, -1e308, 1e308]),
            PredicatePoint::from([1.0, -1.5e308, 1.5e308]),
        );
        assert_eq!(middle, PredicatePoint::Exact([0.75, -1.25e308, 1.25e308]));
    }

    /// Halving first loses the last bit of the smallest subnormal: half of
    /// `2⁻¹⁰⁷⁴` is no double. The middle of it and `3 × 2⁻¹⁰⁷⁴` is `2⁻¹⁰⁷³`, which is.
    #[test]
    fn the_middle_of_two_subnormal_corners_is_exact() {
        let smallest = f64::from_bits(1);
        let middle = midway_between(
            PredicatePoint::from([smallest, 0.0, 0.0]),
            PredicatePoint::from([3.0 * smallest, 0.0, 0.0]),
        );
        assert_eq!(middle, PredicatePoint::Exact([2.0 * smallest, 0.0, 0.0]));
    }

    /// The triangle a fan of the top of
    /// `tests/three_cut_lines_through_one_corner_of_a_skew_pair.rs`'s two
    /// leaning blocks holds: from `(1, 1)` to the corner `(3, 1)`, and on to a
    /// second meet a rounding from that corner, all at `z = 2`. Its first and
    /// last sides run along one line, opposite ways, so a point off that line
    /// turns one way about the first and the other way about the last. Which
    /// way it turns about the short side nothing can say, and that is not
    /// needed to know the point is outside (Q1145).
    #[test]
    fn a_point_off_a_triangle_with_a_side_a_rounding_long_is_outside_it() {
        let top = Plane::from_coefficients([0.0, 0.0, 1.0, -2.0]).expect("a normal");
        let in_plane = InPlane::of(&top).expect("a plane with a point off it");
        let apex = PredicatePoint::from([1.0, 1.0, 2.0]);
        let corner = PredicatePoint::from([3.0, 1.0, 2.0]);
        let a_rounding_away = PredicatePoint::Constructed([
            Interval::around(3.0, 2e-15).expect("a finite box"),
            Interval::around(1.0, 1e-15).expect("a finite box"),
            Interval::exactly(2.0),
        ]);
        let well_inside_the_top = PredicatePoint::from([7.0 / 3.0, 4.0 / 3.0, 2.0]);
        assert!(
            matches!(
                in_plane.turn(corner, a_rounding_away, well_inside_the_top),
                Decision::Undecidable { .. }
            ),
            "the short side's turn is the one nothing can say"
        );
        assert_eq!(
            in_plane.inside_triangle(apex, corner, a_rounding_away, well_inside_the_top),
            Some(0)
        );
    }

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

    /// Both circles `z = 0.5` cuts the bend in are round the axis, so their
    /// middle is where the axis meets the plane.
    #[test]
    fn a_plane_square_to_a_bends_axis_offers_where_the_axis_meets_it() {
        let (ring, _) = a_bend_and_its_end_cap_plane();
        let across = Plane::from_coefficients([0.0, 0.0, 1.0, -0.5]).expect("a normal");
        assert_eq!(
            the_middles_of_a_round_wire(&across, &ring),
            [PredicatePoint::from([0.0, 0.0, 0.5])]
        );
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

    /// `(0, 4, 0)` sits exactly on the meridian half-space's own plane, not
    /// strictly in front of or behind it. `every_picker_keeps` still answers
    /// `Some(true)`, because a point on the half-space's own plane is kept
    /// (`holds.rs`'s `the_meridian_half_space_keeps`).
    #[test]
    fn every_picker_keeps_a_point_on_the_half_spaces_own_plane() {
        let (ring, flat) = a_bend_and_its_end_cap_plane();
        let pickers = the_pickers_of_the_meridian_circle_through(
            &Surface::Plane(flat),
            &ring,
            PredicatePoint::from([3.0, 0.0, 0.0]),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a plane through a torus's axis crosses it")
        .expect("one of the two circles keeps the corner");
        let on_the_plane = PredicatePoint::from([0.0, 4.0, 0.0]);
        assert_eq!(
            pickers[0].side_of_point(on_the_plane),
            Decision::Zero,
            "(0, 4, 0) must sit exactly on the half-space's own plane for this test to mean anything"
        );
        assert_eq!(every_picker_keeps(&pickers, on_the_plane), Some(true));
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

    /// `z = 0.5` cuts the tube in a band between two whole circles round the
    /// axis. A point the torus's sign holds there is inside the outer circle
    /// and outside the inner one, whichever it is, so the band holds it. No
    /// picker says that, and until the wire's own circle was asked the band
    /// could not say what it held (Q1501).
    #[test]
    fn a_band_on_a_plane_square_to_a_bends_axis_holds_its_own_points() {
        let (ring, _) = a_bend_and_its_end_cap_plane();
        let across = Plane::from_coefficients([0.0, 0.0, 1.0, -0.5]).expect("a normal");
        let reach = 0.75_f64.sqrt();
        let band = a_flat_piece_inside_the_ring(
            ring,
            across,
            &[[4.0 + reach, 0.0, 0.5], [4.0 - reach, 0.0, 0.5]],
        );
        let holds =
            |at: [f64; 3]| the_piece_holds_asked_as_selection_asks(&band, PredicatePoint::from(at));
        assert_eq!(holds([4.0, 0.0, 0.5]), Some(true), "in the band");
        assert_eq!(
            holds([0.0, -4.0, 0.5]),
            Some(true),
            "in the band, a quarter turn on"
        );
        assert_eq!(holds([2.0, 0.0, 0.5]), Some(false), "in the inner disk");
        assert_eq!(holds([6.0, 0.0, 0.5]), Some(false), "beyond the band");
    }

    /// A bend's start cap on `y = 0`: its chord along the floor `z = 0` from
    /// `(3, 0, 0)` to `(5, 0, 0)`, and half of the meridian circle round
    /// `(4, 0, 0)` over the top, as the top half of a quarter bend has it. Its
    /// two corners wind round nothing, so what holds a point is the arc's
    /// bulge. The torus's sign holds the other disk, round `(−4, 0, 0)`, as
    /// surely, and its upper half is on the bulge's side of the chord's line
    /// too: only the half-space naming the arc's circle keeps it out (Q1373).
    #[test]
    fn a_bends_start_cap_holds_its_own_half_disk_and_not_the_other_disks_half() {
        let (ring, _) = a_bend_and_its_end_cap_plane();
        let cap = Plane::from_coefficients([0.0, -1.0, 0.0, 0.0]).expect("a normal");
        let floor = Plane::from_coefficients([0.0, 0.0, -1.0, 0.0]).expect("a normal");
        let corner = |x: f64, which: u8| MeetPoint::OfThreeSurfaces {
            surfaces: [Surface::Plane(cap), ring, Surface::Plane(floor)],
            which,
            point: PredicatePoint::from([x, 0.0, 0.0]),
        };
        let half_disk = Piece {
            ancestors: Vec::new(),
            surface: Surface::Plane(cap),
            orientation: Orientation::Forward,
            wires: vec![vec![
                FaceMeet {
                    carried_by: [Surface::Plane(cap), Surface::Plane(floor)],
                    ends: [corner(3.0, 0), corner(5.0, 1)],
                    material_side: Decision::Neg,
                    along: SmallVec::new(),
                    approximation: None,
                },
                FaceMeet {
                    carried_by: [Surface::Plane(cap), ring],
                    ends: [corner(5.0, 1), corner(3.0, 0)],
                    material_side: Decision::Neg,
                    along: SmallVec::new(),
                    approximation: None,
                },
            ]],
            is_the_whole_surface: false,
        };
        let holds = |at: [f64; 3]| {
            the_piece_holds_asked_as_selection_asks(&half_disk, PredicatePoint::from(at))
        };
        assert_eq!(holds([4.0, 0.0, 0.5]), Some(true), "inside the half disk");
        assert_eq!(holds([4.0, 0.0, -0.5]), Some(false), "under its chord");
        assert_eq!(
            holds([-4.0, 0.0, 0.5]),
            Some(false),
            "in the other disk, on the bulge's side of the chord's line"
        );
    }

    /// A piece of a block's floor under a plate with a round tab, a pipe of 3
    /// about `(10, 10)`: the rectangle from `(7.6, 8.2)` to `(10, 10)`, whose
    /// corner `(7.6, 8.2)` is a rounding outside the pipe, so the pipe cuts
    /// that corner off by an arc between where the block's two walls cross
    /// the rim. Those two places are boxes a rounding wide, and they overlap at
    /// the corner, so the chord between them has no direction and the turn
    /// about it was undecided for every point. No point was held, a unit clear
    /// of the arc included, and every certified fuse of the block under the
    /// plate was refused "a point inside a piece of a face"
    /// (`tests/a_block_cornered_on_a_round_tabs_rim.rs`, Q2297).
    #[test]
    fn a_rectangle_with_its_corner_cut_off_by_an_arc_a_rounding_long_holds_its_own_points() {
        let flat = Plane::from_coefficients([0.0, 0.0, 1.0, 0.0]).expect("a normal");
        let pipe = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::from([10.0, 10.0, 0.0]), Vector::new(0.0, 0.0, 1.0))
                    .expect("an axis"),
                3.0,
            )
            .expect("a positive radius"),
        );
        let wall = |coefficients: [f64; 4]| {
            Surface::Plane(Plane::from_coefficients(coefficients).expect("a normal"))
        };
        let west = wall([-1.0, 0.0, 0.0, 7.6]);
        let south = wall([0.0, -1.0, 0.0, 8.2]);
        let east = wall([1.0, 0.0, 0.0, -10.0]);
        let north = wall([0.0, 1.0, 0.0, -10.0]);
        let corner =
            |one: Surface, other: Surface, [x, y]: [Interval; 2]| MeetPoint::OfThreeSurfaces {
                surfaces: [Surface::Plane(flat), one, other],
                which: 0,
                point: PredicatePoint::Constructed([x, y, Interval::exactly(0.0)]),
            };
        let at = |x: f64, y: f64| [Interval::exactly(x), Interval::exactly(y)];
        let on_the_south_wall = corner(
            south,
            pipe,
            [
                Interval::new(7.6, 7.600000000000001).expect("an interval"),
                Interval::exactly(8.2),
            ],
        );
        let on_the_west_wall = corner(
            west,
            pipe,
            [
                Interval::exactly(7.6),
                Interval::new(8.2, 8.200000000000001).expect("an interval"),
            ],
        );
        let sides = [
            (
                south,
                on_the_south_wall.clone(),
                corner(south, east, at(10.0, 8.2)),
            ),
            (
                east,
                corner(south, east, at(10.0, 8.2)),
                corner(east, north, at(10.0, 10.0)),
            ),
            (
                north,
                corner(east, north, at(10.0, 10.0)),
                corner(north, west, at(7.6, 10.0)),
            ),
            (
                west,
                corner(north, west, at(7.6, 10.0)),
                on_the_west_wall.clone(),
            ),
            (pipe, on_the_west_wall, on_the_south_wall),
        ];
        let piece = Piece {
            ancestors: Vec::new(),
            surface: Surface::Plane(flat),
            orientation: Orientation::Forward,
            wires: vec![
                sides
                    .into_iter()
                    .map(|(partner, from, to)| FaceMeet {
                        carried_by: [Surface::Plane(flat), partner],
                        ends: [from, to],
                        material_side: Decision::Neg,
                        along: SmallVec::new(),
                        approximation: None,
                    })
                    .collect(),
            ],
            is_the_whole_surface: false,
        };
        let holds = |at: [f64; 3]| {
            the_piece_holds_asked_as_selection_asks(&piece, PredicatePoint::from(at))
        };
        assert_eq!(
            holds([8.2, 9.55, 0.0]),
            Some(true),
            "a unit clear of the arc"
        );
        assert_eq!(
            holds([9.4, 8.65, 0.0]),
            Some(true),
            "a unit clear of the arc"
        );
        assert_ne!(
            holds([11.0, 9.0, 0.0]),
            Some(true),
            "past the east side, inside the pipe"
        );
        assert_eq!(holds([7.5, 8.1, 0.0]), Some(false), "outside the rim");
    }

    /// The floor `z = 0` of a pipe of `0.5` about the `z` axis, cut by the
    /// plane `y = c`, `c` the double just under `0.5`: the plane crosses the
    /// rim `2⁻²⁷` either side of `x = 0`, and leaves above it a sliver no
    /// double lies in. The two crossings are boxes that overlap, so the chord
    /// between them has no direction and the turn about it is undecided for
    /// every point. The disk less the sliver then held no point, its centre
    /// included, and every certified fuse of the bar turned a quarter turn in
    /// the L plate was refused "a point inside a piece of a face"
    /// (`tests/a_bar_flush_with_an_l_plates_inner_side_turned.rs`, Q2655).
    #[test]
    fn a_disk_cut_a_rounding_inside_its_rim_holds_its_centre_and_the_sliver_does_not() {
        let flat = Plane::from_coefficients([0.0, 0.0, 1.0, 0.0]).expect("a normal");
        let pipe = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::from([0.0, 0.0, 0.0]), Vector::new(0.0, 0.0, 1.0))
                    .expect("an axis"),
                0.5,
            )
            .expect("a positive radius"),
        );
        let just_under_a_half = 0.5 - f64::EPSILON / 4.0;
        let wall = Surface::Plane(
            Plane::from_coefficients([0.0, 1.0, 0.0, -just_under_a_half]).expect("a normal"),
        );
        let crossing = |which: u8, lo: f64, hi: f64| MeetPoint::OfThreeSurfaces {
            surfaces: [Surface::Plane(flat), wall, pipe],
            which,
            point: PredicatePoint::Constructed([
                Interval::new(lo, hi).expect("an interval"),
                Interval::new(0.499_999_9, 0.5).expect("an interval"),
                Interval::exactly(0.0),
            ]),
        };
        let east = crossing(0, -1e-8, 2e-8);
        let west = crossing(1, -2e-8, 1e-8);
        let side = |partner: Surface, from: &MeetPoint, to: &MeetPoint, material_side| FaceMeet {
            carried_by: [Surface::Plane(flat), partner],
            ends: [from.clone(), to.clone()],
            material_side,
            along: SmallVec::new(),
            approximation: None,
        };
        let piece = |wire: Vec<FaceMeet>| Piece {
            ancestors: Vec::new(),
            surface: Surface::Plane(flat),
            orientation: Orientation::Forward,
            wires: vec![wire],
            is_the_whole_surface: false,
        };
        let the_disk_less_the_sliver = piece(vec![
            side(wall, &east, &west, Decision::Neg),
            side(pipe, &west, &east, Decision::Neg),
        ]);
        let the_sliver = piece(vec![
            side(wall, &west, &east, Decision::Pos),
            side(pipe, &east, &west, Decision::Neg),
        ]);
        let holds = |piece: &Piece, at: [f64; 3]| {
            the_piece_holds_asked_as_selection_asks(piece, PredicatePoint::from(at))
        };
        assert_eq!(
            holds(&the_disk_less_the_sliver, [0.0, 0.0, 0.0]),
            Some(true),
            "the disk's centre"
        );
        assert_eq!(
            holds(&the_disk_less_the_sliver, [0.3, 0.3, 0.0]),
            Some(true),
            "a point of the disk under the wall"
        );
        assert_eq!(
            holds(&the_disk_less_the_sliver, [0.0, -0.6, 0.0]),
            Some(false),
            "outside the rim"
        );
        assert_eq!(
            holds(&the_sliver, [0.0, 0.0, 0.0]),
            Some(false),
            "the disk's centre, on the far side of the wall from the sliver"
        );
    }

    /// The half of the plate `z = 0` with `x ≥ 0` beyond the ring of four
    /// round a tube of one: between the circle of 5 the ring cuts it in and a
    /// pipe of 10, its four corners on the line `x = 0`, so they wind round
    /// nothing. The pipe's arc bulges over the whole half disk of 10, and the
    /// ring's arc bites the half disk of 5 back out of it. The ring's sign takes
    /// the tube out of that bite and not the inner disk, so a point there was
    /// held (Q1305).
    #[test]
    fn a_plate_beyond_a_rings_outer_circle_holds_no_point_of_the_inner_disk() {
        let plate = a_flat_piece_on_z_0(&[&[
            (
                a_pipe_about_z(10.0),
                [0.0, -10.0],
                [0.0, 10.0],
                Decision::Neg,
            ),
            (the_plane_x_0(), [0.0, 10.0], [0.0, 5.0], Decision::Neg),
            (a_ring_of_four(), [0.0, 5.0], [0.0, -5.0], Decision::Pos),
            (the_plane_x_0(), [0.0, -5.0], [0.0, -10.0], Decision::Neg),
        ]]);
        let holds = |at: [f64; 3]| {
            the_piece_holds_asked_as_selection_asks(&plate, PredicatePoint::from(at))
        };
        assert_eq!(holds([7.5, 0.0, 0.0]), Some(true), "on the plate");
        assert_eq!(holds([4.0, 0.0, 0.0]), Some(false), "in the tube");
        assert_eq!(holds([2.5, 0.0, 0.0]), Some(false), "in the inner disk");
        assert_eq!(holds([-7.5, 0.0, 0.0]), Some(false), "past the chord");
    }

    /// The half of the plate `z = 0` with `x ≥ 0` inside the circle of 3 the
    /// ring of four round a tube of one cuts it in: a half disk whose two
    /// corners wind round nothing, so what holds a point is the ring's arc, and
    /// until that was counted it held none (Q1305).
    #[test]
    fn a_half_disk_inside_a_rings_inner_circle_holds_its_own_points() {
        let half_disk = a_flat_piece_on_z_0(&[&[
            (a_ring_of_four(), [0.0, -3.0], [0.0, 3.0], Decision::Pos),
            (the_plane_x_0(), [0.0, 3.0], [0.0, -3.0], Decision::Neg),
        ]]);
        let holds = |at: [f64; 3]| {
            the_piece_holds_asked_as_selection_asks(&half_disk, PredicatePoint::from(at))
        };
        assert_eq!(holds([1.5, 0.0, 0.0]), Some(true), "in the half disk");
        assert_eq!(holds([-1.5, 0.0, 0.0]), Some(false), "past the chord");
        assert_eq!(holds([4.0, 0.0, 0.0]), Some(false), "in the tube");
        assert_eq!(holds([7.5, 0.0, 0.0]), Some(false), "beyond the tube");
    }

    /// The square of 20 round the axis on `z = 0`, less the disk inside the
    /// whole outer circle, of 5, that the ring of four round a tube of one cuts
    /// it in. The ring's sign takes the tube out of that disk and not the inner
    /// disk inside it, so read by that sign alone the circle took nothing out
    /// of the square there, and a point of the inner disk was held (Q1501).
    #[test]
    fn a_square_less_a_rings_whole_outer_circle_holds_no_point_of_the_inner_disk() {
        let side_of_the_square = |normal: [f64; 2]| {
            Surface::Plane(
                Plane::from_coefficients([normal[0], normal[1], 0.0, -10.0]).expect("a normal"),
            )
        };
        let plate = a_flat_piece_on_z_0(&[
            &[
                (
                    side_of_the_square([1.0, 0.0]),
                    [10.0, -10.0],
                    [10.0, 10.0],
                    Decision::Neg,
                ),
                (
                    side_of_the_square([0.0, 1.0]),
                    [10.0, 10.0],
                    [-10.0, 10.0],
                    Decision::Neg,
                ),
                (
                    side_of_the_square([-1.0, 0.0]),
                    [-10.0, 10.0],
                    [-10.0, -10.0],
                    Decision::Neg,
                ),
                (
                    side_of_the_square([0.0, -1.0]),
                    [-10.0, -10.0],
                    [10.0, -10.0],
                    Decision::Neg,
                ),
            ],
            &[(a_ring_of_four(), [5.0, 0.0], [5.0, 0.0], Decision::Pos)],
        ]);
        let holds = |at: [f64; 3]| {
            the_piece_holds_asked_as_selection_asks(&plate, PredicatePoint::from(at))
        };
        assert_eq!(holds([7.5, 0.0, 0.0]), Some(true), "on the plate");
        assert_eq!(holds([4.0, 0.0, 0.0]), Some(false), "in the tube");
        assert_eq!(holds([2.5, 0.0, 0.0]), Some(false), "in the inner disk");
        assert_eq!(
            holds([-1.0, 2.0, 0.0]),
            Some(false),
            "in the inner disk, across the axis"
        );
        assert_eq!(holds([12.5, 0.0, 0.0]), Some(false), "past the square");
    }

    /// The disk inside the whole inner circle, of 3, that the ring of four
    /// round a tube of one cuts `z = 0` in. Its material is outside the ring,
    /// and so is every point of the disk, so read by the ring's sign alone the
    /// circle bounded nothing and the disk held none of its own points (Q1501).
    #[test]
    fn a_disk_inside_a_rings_whole_inner_circle_holds_its_own_points() {
        let disk =
            a_flat_piece_on_z_0(&[&[(a_ring_of_four(), [3.0, 0.0], [3.0, 0.0], Decision::Pos)]]);
        let holds =
            |at: [f64; 3]| the_piece_holds_asked_as_selection_asks(&disk, PredicatePoint::from(at));
        assert_eq!(holds([0.0, 0.0, 0.0]), Some(true), "on the axis");
        assert_eq!(holds([-1.5, 1.5, 0.0]), Some(true), "in the disk");
        assert_eq!(holds([4.0, 0.0, 0.0]), Some(false), "in the tube");
        assert_eq!(holds([7.5, 0.0, 0.0]), Some(false), "beyond the tube");
    }

    /// The band of `z = 0` between a pipe of `23/8` about the axis and the
    /// inner circle, of 3, that the ring of four round a tube of one cuts it
    /// in, its material outside both. The arrangement handed row Q1580's plate
    /// over with the pipe's circle first, and read as the outer wire for coming
    /// first, that circle wound every point of the band zero times, so the band
    /// held none of its own points.
    #[test]
    fn a_band_inside_a_rings_inner_circle_holds_its_points_with_either_circle_first() {
        let the_pipe: SideOnZ0 = (
            a_pipe_about_z(23.0 / 8.0),
            [23.0 / 8.0, 0.0],
            [23.0 / 8.0, 0.0],
            Decision::Pos,
        );
        let the_ring: SideOnZ0 = (a_ring_of_four(), [3.0, 0.0], [3.0, 0.0], Decision::Pos);
        for (band, first) in [
            (
                a_flat_piece_on_z_0(&[&[the_pipe], &[the_ring]]),
                "pipe first",
            ),
            (
                a_flat_piece_on_z_0(&[&[the_ring], &[the_pipe]]),
                "ring first",
            ),
        ] {
            let holds = |at: [f64; 3]| {
                the_piece_holds_asked_as_selection_asks(&band, PredicatePoint::from(at))
            };
            assert_eq!(
                holds([0.0, 47.0 / 16.0, 0.0]),
                Some(true),
                "in the band, {first}"
            );
            assert_eq!(holds([1.0, 0.0, 0.0]), Some(false), "in the pipe, {first}");
            assert_eq!(holds([4.0, 0.0, 0.0]), Some(false), "in the tube, {first}");
            assert_eq!(
                holds([7.5, 0.0, 0.0]),
                Some(false),
                "beyond the tube, {first}"
            );
        }
    }

    /// One side of a piece of `z = 0`: its partner, its two ends in the plane,
    /// and which side of the partner the piece's material is on.
    type SideOnZ0 = (Surface, [f64; 2], [f64; 2], Decision);

    /// A piece of the plane `z = 0`, facing `+z`, walked wire by wire and side
    /// by side: each side's partner, its two ends in the plane, and which side
    /// of the partner the piece's material is on. A wire of one side whose two
    /// ends are the same goes right round a closed curve.
    fn a_flat_piece_on_z_0(wires: &[&[SideOnZ0]]) -> Piece {
        let flat = Plane::from_coefficients([0.0, 0.0, 1.0, 0.0]).expect("a normal");
        let corner = |[x, y]: [f64; 2], partner: Surface| MeetPoint::OfThreeSurfaces {
            surfaces: [Surface::Plane(flat), partner, the_plane_x_0()],
            which: 0,
            point: PredicatePoint::from([x, y, 0.0]),
        };
        let wires = wires
            .iter()
            .map(|sides| {
                sides
                    .iter()
                    .map(|&(partner, from, to, material_side)| FaceMeet {
                        carried_by: [Surface::Plane(flat), partner],
                        ends: [corner(from, partner), corner(to, partner)],
                        material_side,
                        along: SmallVec::new(),
                        approximation: None,
                    })
                    .collect()
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

    fn a_ring_of_four() -> Surface {
        Surface::Torus(Torus::new(Axis::z(), 4.0, 1.0).expect("a ring wider than its tube"))
    }

    fn a_pipe_about_z(radius: f64) -> Surface {
        Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a positive radius"))
    }

    /// The plane `x = 0`, facing `−x`, out of a plate on `x ≥ 0`.
    fn the_plane_x_0() -> Surface {
        Surface::Plane(Plane::from_coefficients([-1.0, 0.0, 0.0, 0.0]).expect("a normal"))
    }

    /// The block from `(0, 0, 0)` to `(4, 4, 4)` has its floor on `z = 0`,
    /// facing `−z`. A flat piece one step of an `f64` below the floor is
    /// outside the block, and one step above it is inside, though a box
    /// `2e−15` tall round the middle of the floor runs through both planes and
    /// the ray can put it on neither side. What the floor's offset and the
    /// piece's say is the answer, since no other face of the block comes near
    /// the box (Q2043). A piece on the floor's own plane is not this rule's
    /// to place: coincidence is asked of it first.
    #[test]
    fn a_flat_piece_a_step_off_a_blocks_floor_is_placed_by_the_floor() {
        let (model, block) =
            a_prism_four_tall_on(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]]);
        let solid = model.solid(block).expect("the block is in the model");
        let one_step = f64::from_bits(1);
        let mid_floor = PredicatePoint::Constructed([
            Interval::exactly(2.0),
            Interval::exactly(2.0),
            Interval::around(0.0, 1e-15).expect("a finite box"),
        ]);
        let placed = |height: f64| {
            by_the_one_face_its_box_meets(
                solid,
                &a_flat_piece_on([0.0, 0.0, 1.0, -height]),
                mid_floor,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
        };
        assert_eq!(placed(-one_step), Some(false), "below the floor");
        assert_eq!(placed(one_step), Some(true), "above the floor");
        assert_eq!(placed(0.0), None, "on the floor itself");
    }

    /// A wedge on the triangle `(0, 0)`, `(4, 0)`, `(0, ¼)` has a wall on
    /// `y = 0`, facing `−y`, and its long side `x + 16y = 4` leans over that
    /// wall towards `x = 4`, a sixteenth as high as it is far from that end.
    /// A box `2⁻²⁰` short of the end, `2⁻²²` tall, is held by the wall and is
    /// also run through by the side, `2⁻²⁴` to `2⁻²³` above the wall. The wall
    /// cannot place a piece by that box alone: a piece of the plane
    /// `y = 3·2⁻²⁵` is inside the wedge in one part of the box and outside it
    /// in the other, and the wall would call all of it inside. Nor a step off
    /// the wall either way, where the wall's word happens to be right.
    #[test]
    fn a_box_a_side_leaning_over_a_wall_runs_through_is_not_placed_by_the_wall() {
        let (model, wedge) = a_prism_four_tall_on(vec![[0.0, 0.0], [4.0, 0.0], [0.0, 0.25]]);
        let solid = model.solid(wedge).expect("the wedge is in the model");
        let short = f64::powi(2.0, -20);
        let under_the_side = PredicatePoint::Constructed([
            Interval::new(4.0 - 2.0 * short, 4.0 - short).expect("a finite box"),
            Interval::around(0.0, short / 8.0).expect("a finite box"),
            Interval::exactly(2.0),
        ]);
        let one_step = f64::from_bits(1);
        for height in [-one_step, one_step, 3.0 * short / 32.0] {
            assert_eq!(
                by_the_one_face_its_box_meets(
                    solid,
                    &a_flat_piece_on([0.0, 1.0, 0.0, -height]),
                    under_the_side,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                ),
                None,
                "the piece on y = {height:e}"
            );
        }
    }

    /// `outline` on `z = 0` drawn straight up to `z = 4`, its floor on `z = 0`.
    fn a_prism_four_tall_on(outline: Vec<[f64; 2]>) -> (Model, SolidKey) {
        let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
        let outline = ProfileBuilder::new(outline)
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("an outline");
        let prism = model
            .add_extrusion(&outline, 4.0, &Isometry::identity())
            .expect("a positive height");
        (model, prism)
    }

    /// A piece of the plane with these coefficients. It has no sides:
    /// [`by_the_one_face_its_box_meets`] reads the piece's plane and nothing
    /// else.
    fn a_flat_piece_on(coefficients: [f64; 4]) -> Piece {
        Piece {
            ancestors: Vec::new(),
            surface: Surface::Plane(Plane::from_coefficients(coefficients).expect("a normal")),
            orientation: Orientation::Forward,
            wires: Vec::new(),
            is_the_whole_surface: false,
        }
    }

    /// A piece of the wall `y = −10` inside the lower nappe of `x² + y² = (z/2)²`:
    /// one arc of the hyperbola and the chord `z = −52` across it, corners
    /// `(±24, −10, −52)`. The upper nappe's region on the same wall is inside the
    /// cone by its sign and on the bulge side of the chord, and is not the piece.
    #[test]
    fn a_hyperbola_arcs_piece_holds_nothing_on_the_other_nappe() {
        let cone = Cone::new(Axis::z(), 0.5).expect("a positive slope");
        let wall = Plane::from_coefficients([0.0, 1.0, 0.0, 10.0]).expect("a normal");
        let floor = Plane::from_coefficients([0.0, 0.0, 1.0, 52.0]).expect("a normal");
        assert_eq!(cone.is_cut_in_a_closed_curve_by(&wall), Some(false));
        let corner = |at: [f64; 3], which: u8| MeetPoint::OfThreeSurfaces {
            surfaces: [
                Surface::Plane(wall),
                Surface::Cone(cone),
                Surface::Plane(floor),
            ],
            which,
            point: PredicatePoint::from(at),
        };
        let [left, right] = [[-24.0, -10.0, -52.0], [24.0, -10.0, -52.0]];
        let piece = Piece {
            ancestors: Vec::new(),
            surface: Surface::Plane(wall),
            orientation: Orientation::Forward,
            wires: vec![vec![
                FaceMeet {
                    carried_by: [Surface::Plane(wall), Surface::Cone(cone)],
                    ends: [corner(left, 0), corner(right, 1)],
                    material_side: Decision::Neg,
                    along: SmallVec::new(),
                    approximation: None,
                },
                FaceMeet {
                    carried_by: [Surface::Plane(wall), Surface::Plane(floor)],
                    ends: [corner(right, 1), corner(left, 0)],
                    material_side: Decision::Pos,
                    along: SmallVec::new(),
                    approximation: None,
                },
            ]],
            is_the_whole_surface: false,
        };
        let holds = |at: [f64; 3]| {
            the_piece_holds_asked_as_selection_asks(&piece, PredicatePoint::from(at))
        };
        assert_eq!(
            holds([0.0, -10.0, -30.0]),
            Some(true),
            "between the arc and its chord"
        );
        assert_eq!(holds([0.0, -10.0, -60.0]), Some(false), "past the chord");
        assert_eq!(
            holds([0.0, -10.0, 30.0]),
            Some(false),
            "the upper nappe's region"
        );
        assert_eq!(
            holds([3.0, -10.0, 25.0]),
            Some(false),
            "the upper nappe's region"
        );
    }

    /// A triangle on the plane `y = 0`, which passes through the apex: one ruling
    /// from `(10, 0, 20)` down to the apex, then up `x = 0` and back along `z = 20`.
    /// The apex end's nappe sign is zero, and the opposite wedge `z < 0` is inside
    /// the cone by its sign and right of the ruling's line.
    #[test]
    fn a_piece_with_a_corner_at_the_apex_holds_nothing_in_the_opposite_wedge() {
        let cone = Cone::new(Axis::z(), 0.5).expect("a positive slope");
        let wall = Plane::from_coefficients([0.0, 1.0, 0.0, 0.0]).expect("a normal");
        let roof = Plane::from_coefficients([0.0, 0.0, 1.0, -20.0]).expect("a normal");
        let upright = Plane::from_coefficients([1.0, 0.0, 0.0, 0.0]).expect("a normal");
        assert_eq!(cone.is_cut_in_a_closed_curve_by(&wall), Some(false));
        let corner = |at: [f64; 3], which: u8| MeetPoint::OfThreeSurfaces {
            surfaces: [
                Surface::Plane(wall),
                Surface::Cone(cone),
                Surface::Plane(roof),
            ],
            which,
            point: PredicatePoint::from(at),
        };
        let [out, apex, up] = [[10.0, 0.0, 20.0], [0.0, 0.0, 0.0], [0.0, 0.0, 20.0]];
        let side =
            |from: [f64; 3], to: [f64; 3], by: Surface, kept: Decision, which: [u8; 2]| FaceMeet {
                carried_by: [Surface::Plane(wall), by],
                ends: [corner(from, which[0]), corner(to, which[1])],
                material_side: kept,
                along: SmallVec::new(),
                approximation: None,
            };
        let piece = Piece {
            ancestors: Vec::new(),
            surface: Surface::Plane(wall),
            orientation: Orientation::Forward,
            wires: vec![vec![
                side(out, apex, Surface::Cone(cone), Decision::Neg, [0, 1]),
                side(apex, up, Surface::Plane(upright), Decision::Pos, [1, 2]),
                side(up, out, Surface::Plane(roof), Decision::Neg, [2, 0]),
            ]],
            is_the_whole_surface: false,
        };
        let holds = |at: [f64; 3]| {
            the_piece_holds_asked_as_selection_asks(&piece, PredicatePoint::from(at))
        };
        assert_eq!(holds([2.0, 0.0, 10.0]), Some(true), "inside the triangle");
        assert_eq!(holds([0.0, 0.0, -10.0]), Some(false), "the opposite wedge");
        assert_eq!(holds([1.0, 0.0, -10.0]), Some(false), "the opposite wedge");
    }

    /// A ball of radius 3 four up a cone of slope 1 crosses it in two circles,
    /// at `z = 2 ∓ √2/2`, and `intersect` picks each out by a half-space
    /// either side of `z = 2`. The ball's cap below the lower circle and its cap
    /// above the upper one are both inside the cone, so the lower cap's side
    /// alone held `(2.93, 0, 4.66)`, a point of the upper cap (Q1197). The
    /// cone's band between the circles has a side along each, whose pickers
    /// hold opposite halves of it, and it holds points either side of `z = 2`.
    #[test]
    fn a_balls_cap_below_a_cone_holds_nothing_of_its_cap_above() {
        let cone = Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a positive slope"));
        let ball =
            Surface::Sphere(Sphere::new(Point::from([0.0, 0.0, 4.0]), 3.0).expect("a radius"));
        let SurfaceMeet::Curves(circles) = intersect(
            &ball,
            &cone,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a ball on a cone's axis crosses it") else {
            panic!("a ball pushing out through a cone's side crosses it in curves");
        };
        let side = |on: Surface, partner: Surface, height: f64| {
            let at = PredicatePoint::from([height, 0.0, height]);
            let picked_by = circles
                .iter()
                .find_map(|circle| {
                    let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = circle.run() else {
                        panic!("each of the two circles is picked out: {circle:?}");
                    };
                    picked_by
                        .iter()
                        .all(|picker| picker.side_of_point(at) == Decision::Pos)
                        .then(|| picked_by.clone())
                })
                .expect("one of the two circles holds the point");
            let starts = MeetPoint::WhereAClosedCurveStarts {
                carried_by: [on, partner],
                picked_by,
                point: at,
            };
            vec![FaceMeet {
                carried_by: [on, partner],
                ends: [starts.clone(), starts],
                material_side: Decision::Neg,
                along: SmallVec::new(),
                approximation: None,
            }]
        };
        let piece = |surface: Surface, wires: Vec<Vec<FaceMeet>>| Piece {
            ancestors: Vec::new(),
            surface,
            orientation: Orientation::Forward,
            wires,
            is_the_whole_surface: false,
        };
        let [low, high] = [-1.0, 1.0].map(|way| 2.0 + way * 2.0_f64.sqrt() / 2.0);
        let lower_cap = piece(ball, vec![side(ball, cone, low)]);
        let band = piece(cone, vec![side(cone, ball, low), side(cone, ball, high)]);
        let holds = |piece: &Piece, at: [f64; 3]| {
            the_piece_holds(piece, None, &[], &[], PredicatePoint::from(at))
        };
        assert_eq!(holds(&lower_cap, [0.75, 0.0, 1.1]), Some(true), "the cap");
        assert_eq!(
            holds(&lower_cap, [2.93, 0.0, 4.66]),
            Some(false),
            "the cap above"
        );
        assert_eq!(holds(&band, [1.5, 0.0, 1.5]), Some(true), "below z = 2");
        assert_eq!(holds(&band, [2.5, 0.0, 2.5]), Some(true), "above z = 2");
    }

    /// A disk on `z = 0` cut by a ball of radius 1 round `(0, 0, −0.75)`, so
    /// `√0.4375 ≈ 0.661` across from its middle. `(0.67, 0, −0.1)` is inside
    /// the ball, on the centre's side of the plane, and its foot `(0.67, 0, 0)`
    /// is outside the ball and so outside the disk. Asked about the point where
    /// it is, the ball held it, and so did the disk.
    ///
    /// **No fuse is known to reach this.** A real candidate is a rounding off
    /// its plane, not a tenth of a unit, so it reaches only a disk the ball is a
    /// rounding wider than just off the plane, one of radius about `1e-8`
    /// (`docs/decisions.md` §346.6), and no candidate `candidate_points` offers
    /// was seen to land there. The numbers here are big so that each step can
    /// be checked by hand.
    #[test]
    fn a_point_inside_a_ball_whose_foot_is_outside_its_disk_is_not_held() {
        let (ball, flat) = a_ball_cutting_z_0_round_the_origin(0.75);
        let disk = a_flat_piece_inside_the_ring(
            Surface::Sphere(ball),
            flat,
            &[[0.4375_f64.sqrt(), 0.0, 0.0]],
        );
        let off_the_plane = PredicatePoint::from([0.67, 0.0, -0.1]);
        let foot = PredicatePoint::from([0.67, 0.0, 0.0]);
        assert_eq!(ball.side_of(off_the_plane), Decision::Neg);
        assert_eq!(ball.side_of(foot), Decision::Pos);
        assert_eq!(
            the_piece_holds_asked_as_selection_asks(&disk, off_the_plane),
            None,
            "the ball cannot say one thing of the point and of its foot"
        );
        assert_eq!(
            the_piece_holds_asked_as_selection_asks(&disk, foot),
            Some(false)
        );
    }

    /// The same ball and point against the half of that disk on `x > 0`: an arc
    /// of the ball's circle and the chord `x = 0` across it. The arc is a curved
    /// side rather than a round wire, and its ball is asked before the
    /// corners are wound. No fuse is known to reach this either.
    #[test]
    fn a_point_inside_a_ball_whose_foot_is_outside_its_half_disk_is_not_held() {
        let (ball, flat) = a_ball_cutting_z_0_round_the_origin(0.75);
        let chord = Plane::from_coefficients([1.0, 0.0, 0.0, 0.0]).expect("a normal");
        let reach = 0.4375_f64.sqrt();
        let corner = |at: [f64; 3], which: u8| MeetPoint::OfThreeSurfaces {
            surfaces: [
                Surface::Plane(flat),
                Surface::Sphere(ball),
                Surface::Plane(chord),
            ],
            which,
            point: PredicatePoint::from(at),
        };
        let [low, high] = [[0.0, -reach, 0.0], [0.0, reach, 0.0]];
        let half_disk = Piece {
            ancestors: Vec::new(),
            surface: Surface::Plane(flat),
            orientation: Orientation::Forward,
            wires: vec![vec![
                FaceMeet {
                    carried_by: [Surface::Plane(flat), Surface::Sphere(ball)],
                    ends: [corner(low, 0), corner(high, 1)],
                    material_side: Decision::Neg,
                    along: SmallVec::new(),
                    approximation: None,
                },
                FaceMeet {
                    carried_by: [Surface::Plane(flat), Surface::Plane(chord)],
                    ends: [corner(high, 1), corner(low, 0)],
                    material_side: Decision::Pos,
                    along: SmallVec::new(),
                    approximation: None,
                },
            ]],
            is_the_whole_surface: false,
        };
        let holds = |at: [f64; 3]| {
            the_piece_holds_asked_as_selection_asks(&half_disk, PredicatePoint::from(at))
        };
        assert_eq!(holds([0.3, 0.0, 0.0]), Some(true), "inside the half disk");
        assert_eq!(holds([0.67, 0.0, 0.0]), Some(false), "past its arc");
        assert_eq!(
            holds([0.67, 0.0, -0.1]),
            None,
            "inside the ball, with its foot past the arc"
        );
    }

    /// A half disk a million units out on `x + z = 0`, a plane through the
    /// origin: the ball of radius 1 round `(2²⁰, 0, −2²⁰)` cuts the plane in a
    /// circle, and the chord `x = 2²⁰` across its middle keeps the half further
    /// out. Its two corners are exact, on the plane, the ball and the chord.
    ///
    /// The point is one spacing of `z` off the plane, on the side
    /// [`InPlane`]'s reference point is on, and its foot is `2⁻²⁰` short of the
    /// chord, outside the half disk. `InPlane` reads a point off its plane where
    /// the line from its reference point, one unit off the plane beside the
    /// origin, meets the plane, and for a point `2²⁰` out that is `2²⁰` times the
    /// point's height further out than its foot: about `8.6e-5` in `x`, so over
    /// the chord. Read there the chord's turn put the point in the half disk
    /// (`docs/decisions.md` §346.6, Q685).
    ///
    /// Over the point's box stretched to the plane the chord's turn cannot say,
    /// and until Q2655 the piece answered nothing. The chord side runs back along
    /// the plane `x = 2²⁰` between two corners that are meets of both planes, so
    /// the chord is that plane's line across `x + z = 0`, and the stretched box,
    /// `x = 2²⁰ − 2⁻²⁰` all through, is wholly short of it: the point and its
    /// foot are both off the half disk ([`which_side_of_a_wall_back_along_the_chord`]).
    #[test]
    fn a_point_a_rounding_off_a_far_half_disk_whose_foot_is_short_of_its_chord_is_not_held() {
        let out = f64::powi(2.0, 20);
        let flat = Plane::from_coefficients([1.0, 0.0, 1.0, 0.0]).expect("a normal");
        let ball = Sphere::new(Point::from([out, 0.0, -out]), 1.0).expect("a positive radius");
        let chord = Plane::from_coefficients([1.0, 0.0, 0.0, -out]).expect("a normal");
        let corner = |at: [f64; 3], which: u8| MeetPoint::OfThreeSurfaces {
            surfaces: [
                Surface::Plane(flat),
                Surface::Sphere(ball),
                Surface::Plane(chord),
            ],
            which,
            point: PredicatePoint::from(at),
        };
        let [low, high] = [[out, -1.0, -out], [out, 1.0, -out]];
        let half_disk = Piece {
            ancestors: Vec::new(),
            surface: Surface::Plane(flat),
            orientation: Orientation::Forward,
            wires: vec![vec![
                FaceMeet {
                    carried_by: [Surface::Plane(flat), Surface::Sphere(ball)],
                    ends: [corner(low, 0), corner(high, 1)],
                    material_side: Decision::Neg,
                    along: SmallVec::new(),
                    approximation: None,
                },
                FaceMeet {
                    carried_by: [Surface::Plane(flat), Surface::Plane(chord)],
                    ends: [corner(high, 1), corner(low, 0)],
                    material_side: Decision::Pos,
                    along: SmallVec::new(),
                    approximation: None,
                },
            ]],
            is_the_whole_surface: false,
        };
        let short = out - f64::powi(2.0, -20);
        let foot = PredicatePoint::from([short, 0.0, -short]);
        let spacing = f64::powi(2.0, -33);
        let point = PredicatePoint::from([short, 0.0, -short + spacing]);
        assert_eq!(flat.side_of_point(foot), Decision::Zero);
        assert_eq!(flat.side_of_point(point), Decision::Pos);
        assert_eq!(ball.side_of(point), Decision::Neg);
        assert_eq!(
            the_piece_holds_asked_as_selection_asks(&half_disk, foot),
            Some(false),
            "short of the chord"
        );
        assert_eq!(
            the_piece_holds_asked_as_selection_asks(&half_disk, point),
            Some(false),
            "the point and its foot are both short of the chord's plane"
        );
    }

    /// A flat face `2²⁰` out on `x + y = 0`, a plane through the origin, and
    /// two points a spacing off that plane in `x` and in `y` beside its near
    /// edge `x = 2²⁰`, the one nearest the origin.
    ///
    /// The first point's foot is `2⁻²⁰` short of that edge, off the face, and
    /// the point is on the side [`InPlane`]'s reference point is on, so it is
    /// read about `1.7e-4` further out than its foot, over the edge. The second
    /// point's foot is `2⁻²⁰` past the edge, on the face, and the point is on
    /// the other side, so it is read about `3.4e-4` nearer the origin, back
    /// over the edge. Wound round each point alone, the face held the first and
    /// not the second (`docs/decisions.md` §346.6, Q1107). Wound round each
    /// point's box stretched to the plane, the fan cannot say, and the model
    /// answers as each foot says.
    ///
    /// **No fuse is known to reach this.** A point is asked of a face of an
    /// input only once [`the_piece_holds`] has held its stretched box, and a
    /// piece beside this face, sharing its edge with the same corners, does not
    /// hold the first point
    /// (`a_square_beside_a_far_face_does_not_hold_a_point_read_over_their_edge`).
    #[test]
    fn a_point_a_rounding_off_a_far_face_is_on_it_where_its_foot_is() {
        let (model, solid, face) = a_far_face_on_a_plane_through_the_origin();
        let on_it = |point: PredicatePoint| {
            let piece = a_square_beside_the_far_face(face.surface);
            a_face_of_the_input_holds(
                &model,
                &solid,
                &face,
                &piece,
                point,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("the corners are exact")
        };
        let out = f64::powi(2.0, 20);
        let short = out - f64::powi(2.0, -20);
        let past = out + f64::powi(2.0, -20);
        let [below, above] = [f64::powi(2.0, -33), f64::powi(2.0, -32)];
        let short_of_the_edge = PredicatePoint::from([short - below, -short - below, 0.5]);
        let past_the_edge = PredicatePoint::from([past + above, -past + above, 0.5]);
        assert!(
            !on_it(PredicatePoint::from([short, -short, 0.5])),
            "its foot"
        );
        assert!(
            !on_it(short_of_the_edge),
            "read over the edge, its foot short of it"
        );
        assert!(on_it(PredicatePoint::from([past, -past, 0.5])), "its foot");
        assert!(
            on_it(past_the_edge),
            "read short of the edge, its foot past it"
        );
    }

    /// The square beside the far face of the test above, sharing its near
    /// edge, holds the first point's foot and not the point. [`the_piece_holds`]
    /// winds the square's corners over the point's box stretched to the plane,
    /// whose reading runs from beside the foot, in the square, to the point's,
    /// on the face, and so crosses that edge. Every candidate is asked that
    /// before any face of an input is, so a piece beside the face never hands
    /// this point to [`a_face_of_the_input_holds`].
    #[test]
    fn a_square_beside_a_far_face_does_not_hold_a_point_read_over_their_edge() {
        let (_, _, face) = a_far_face_on_a_plane_through_the_origin();
        let square = a_square_beside_the_far_face(face.surface);
        let short = f64::powi(2.0, 20) - f64::powi(2.0, -20);
        let below = f64::powi(2.0, -33);
        let foot = PredicatePoint::from([short, -short, 0.5]);
        let point = PredicatePoint::from([short - below, -short - below, 0.5]);
        assert_eq!(
            the_piece_holds_asked_as_selection_asks(&square, foot),
            Some(true)
        );
        assert_eq!(
            the_piece_holds_asked_as_selection_asks(&square, point),
            None
        );
    }

    /// A square on `z = 0` with the same ball's disk taken out of it. On the
    /// far side of the plane from the ball's centre the ball is narrower than
    /// the hole, so `(0.6, 0, 0.1)` is outside the ball while its foot
    /// `(0.6, 0, 0)` is in the hole. Reading the sign at the point where it is
    /// held it; the far side is where a point *inside* the ball can be trusted,
    /// and this one is outside. No fuse is known to reach this either.
    #[test]
    fn a_point_outside_a_ball_whose_foot_is_in_its_hole_is_not_held() {
        let (ball, flat) = a_ball_cutting_z_0_round_the_origin(0.75);
        let walls = [
            [1.0, 0.0, 0.0, -2.0],
            [0.0, 1.0, 0.0, -2.0],
            [1.0, 0.0, 0.0, 2.0],
            [0.0, 1.0, 0.0, 2.0],
        ]
        .map(|wall| Plane::from_coefficients(wall).expect("a normal"));
        let corners = [
            [2.0, 2.0, 0.0],
            [-2.0, 2.0, 0.0],
            [-2.0, -2.0, 0.0],
            [2.0, -2.0, 0.0],
        ];
        let corner = |which: usize| MeetPoint::OfThreeSurfaces {
            surfaces: [
                Surface::Plane(flat),
                Surface::Plane(walls[which]),
                Surface::Plane(walls[(which + 1) % 4]),
            ],
            which: 0,
            point: PredicatePoint::from(corners[which]),
        };
        let square = (0..4)
            .map(|which| FaceMeet {
                carried_by: [Surface::Plane(flat), Surface::Plane(walls[(which + 1) % 4])],
                ends: [corner(which), corner((which + 1) % 4)],
                material_side: Decision::Neg,
                along: SmallVec::new(),
                approximation: None,
            })
            .collect();
        let hole_starts = MeetPoint::WhereAClosedCurveStarts {
            carried_by: [Surface::Plane(flat), Surface::Sphere(ball)],
            picked_by: SmallVec::new(),
            point: PredicatePoint::from([0.4375_f64.sqrt(), 0.0, 0.0]),
        };
        let hole = vec![FaceMeet {
            carried_by: [Surface::Plane(flat), Surface::Sphere(ball)],
            ends: [hole_starts.clone(), hole_starts],
            material_side: Decision::Pos,
            along: SmallVec::new(),
            approximation: None,
        }];
        let with_a_hole = Piece {
            ancestors: Vec::new(),
            surface: Surface::Plane(flat),
            orientation: Orientation::Forward,
            wires: vec![square, hole],
            is_the_whole_surface: false,
        };
        let holds = |at: [f64; 3]| {
            the_piece_holds_asked_as_selection_asks(&with_a_hole, PredicatePoint::from(at))
        };
        assert_eq!(
            holds([1.5, 0.0, 0.0]),
            Some(true),
            "between the hole and the rim"
        );
        assert_eq!(holds([0.6, 0.0, 0.0]), Some(false), "in the hole");
        assert_eq!(
            ball.side_of(PredicatePoint::from([0.6, 0.0, 0.1])),
            Decision::Pos
        );
        assert_eq!(
            holds([0.6, 0.0, 0.1]),
            None,
            "outside the ball, with its foot in the hole"
        );
    }

    /// A ball that cuts `z = 0` in a disk about `1.5e-8` across, the width of
    /// the disk §346.6 is about: `(1 − 2⁻⁵³)` below the plane, one unit round.
    /// A point `1e-16` off the plane over the disk's middle is held from
    /// either side. Below it, the ball is certainly inside over the whole way
    /// up to the plane; above it, in the cap a fraction of a spacing deep, the
    /// ball's sign over that way is too close to zero to settle, and the point
    /// is held because it is on the far side of the plane from the centre and
    /// inside the ball, so its foot is nearer the centre and inside too.
    #[test]
    fn a_point_a_rounding_off_a_thin_disks_middle_is_held_from_either_side() {
        let (ball, flat) = a_ball_cutting_z_0_round_the_origin(1.0 - f64::EPSILON / 2.0);
        let disk = a_flat_piece_inside_the_ring(
            Surface::Sphere(ball),
            flat,
            &[[f64::EPSILON.sqrt(), 0.0, 0.0]],
        );
        for height in [-1e-16, 1e-16] {
            let point = PredicatePoint::from([0.0, 0.0, height]);
            assert_eq!(ball.side_of(point), Decision::Neg, "at {height}");
            assert_eq!(
                the_piece_holds_asked_as_selection_asks(&disk, point),
                Some(true),
                "at {height}"
            );
        }
    }

    /// The grazing block of `tests/a_block_edge_grazing_a_ball.rs` at `h = 0`.
    ///
    /// No point with `f64` coordinates is inside any of the three slivers its
    /// two places fence off (§346.3), so no point search places them. Each is
    /// placed by the side it shares with a piece of the other solid's face
    /// (§365.3): the ball's sliver is inside the block, and the block's two are
    /// inside the ball. Every other piece holds a point and is outside the other
    /// solid. So an intersection keeps the three slivers and nothing else, and a
    /// union keeps everything but them.
    #[test]
    fn the_three_slivers_of_a_block_edge_grazing_a_ball_are_placed_by_the_sides_they_share() {
        let place = f64::powi(2.0, -51);
        let edge = f64::powi(2.0, -25);
        let the_two_places = [
            PredicatePoint::Exact([-1.0, edge, -place]),
            PredicatePoint::Exact([-1.0, edge, place]),
        ];
        let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
        let ball = model
            .add_ball(1.0 + place, &Isometry::identity())
            .expect("a positive radius");
        let square = ProfileBuilder::new(vec![
            [-4.0, edge],
            [-1.0, edge],
            [-1.0, edge + 3.0],
            [-4.0, edge + 3.0],
        ])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
        let block = model
            .add_extrusion(
                &square,
                4.0,
                &Isometry::translation(Vector::new(0.0, 0.0, -2.0)).expect("a finite shift"),
            )
            .expect("a positive height");
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let scratch = Scratch::split_both_groups(&model, &[ball], &[block], &policy, &mut log)
            .expect("the two places are exact and apart, so the cutting decides everything");
        let mut kept = |rule: Rule| -> Vec<usize> {
            choose(&model, &scratch, rule, &policy, &mut log)
                .expect("every piece is placed, the three slivers by their sides")
                .iter()
                .map(|chosen| {
                    assert!(!chosen.reversed, "{rule:?} turns nothing round");
                    chosen.piece
                })
                .collect()
        };

        let inside_the_other = kept(Rule::Intersection);
        let surfaces: Vec<Surface> = inside_the_other
            .iter()
            .map(|&index| {
                let sliver = &scratch.pieces[index];
                assert!(
                    corners_of(sliver)
                        .concat()
                        .iter()
                        .all(|corner| the_two_places.contains(corner)),
                    "a sliver's corners are the two places: {sliver:?}"
                );
                sliver.surface
            })
            .collect();
        let the_ball = Sphere::new(Point::from([0.0, 0.0, 0.0]), 1.0 + place).expect("a radius");
        let the_wall_at = |coefficients| {
            Surface::Plane(Plane::from_coefficients(coefficients).expect("a normal"))
        };
        assert_eq!(
            surfaces,
            [
                Surface::Sphere(the_ball),
                the_wall_at([0.0, -1.0, 0.0, edge]),
                the_wall_at([1.0, 0.0, 0.0, 1.0]),
            ],
            "the ball's sliver, then the block's on y = 2^-25 and on x = -1"
        );
        let everything_else: Vec<usize> = (0..scratch.pieces.len())
            .filter(|index| !inside_the_other.contains(index))
            .collect();
        assert_eq!(kept(Rule::Union), everything_else);
        assert!(log.is_empty(), "a certified fuse asked nothing: {log:?}");
    }

    /// The grazing block of `tests/a_block_edge_grazing_a_ball_at_cx_16.rs`:
    /// the ball's sliver above `y = 1` and below `z = −2⁻²⁵` is outside the
    /// block, and its side on `y = 1` says so.
    ///
    /// Its two places are `16 ∓ 2⁻⁵¹` along the edge and come out one corner,
    /// so both arcs of the ball's circle on `z = −2⁻²⁵` run from that corner to
    /// itself. Named as one closed edge, the sliver's short arc, which lies
    /// outside the block's top, took the top's pieces along the long arc as
    /// pieces across it and read the sliver inside the block, against its side
    /// on `y = 1`; the two disagreed and the fuse refused on the sliver (Q1073).
    #[test]
    fn the_sliver_of_a_ball_between_two_places_one_corner_apart_is_placed_by_its_side_on_the_wall()
    {
        let top = -f64::powi(2.0, -25);
        for ball_first in [true, false] {
            let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
            let ball = model
                .add_ball(
                    1.0 + f64::powi(2.0, -51),
                    &Isometry::translation(Vector::new(16.0, 0.0, 0.0)).expect("a finite shift"),
                )
                .expect("a positive radius");
            let square =
                ProfileBuilder::new(vec![[14.0, -2.0], [18.0, -2.0], [18.0, 1.0], [14.0, 1.0]])
                    .build(&Policy::certified(), &mut ProvenanceLog::new())
                    .expect("a square");
            let block = model
                .add_extrusion(
                    &square,
                    2.0,
                    &Isometry::translation(Vector::new(0.0, 0.0, top - 2.0))
                        .expect("a finite shift"),
                )
                .expect("a positive height");
            let (first, second) = if ball_first {
                (ball, block)
            } else {
                (block, ball)
            };
            let policy = Policy::certified();
            let mut log = ProvenanceLog::new();
            let scratch =
                Scratch::split_both_groups(&model, &[first], &[second], &policy, &mut log)
                    .expect("the turn and both orders are decided (Q699, Q914)");
            let mut kept = |rule: Rule| -> Vec<usize> {
                choose(&model, &scratch, rule, &policy, &mut log)
                    .unwrap_or_else(|refused| panic!("ball first {ball_first}: {refused:?}"))
                    .iter()
                    .map(|chosen| chosen.piece)
                    .collect()
            };
            let inside_the_other = kept(Rule::Intersection);
            let has_a_side = |piece: &Piece, normal: [f64; 3], material: Decision| {
                piece.wires.iter().flatten().any(|side| {
                    side.carried_by[1]
                        .as_plane()
                        .is_some_and(|flat| flat.normal().components() == normal)
                        && side.material_side == material
                })
            };
            let slivers_of_the_ball: Vec<usize> = (0..scratch.pieces.len())
                .filter(|&index| {
                    let piece = &scratch.pieces[index];
                    matches!(piece.surface, Surface::Sphere(_))
                        && has_a_side(piece, [0.0, 1.0, 0.0], Decision::Pos)
                        && has_a_side(piece, [0.0, 0.0, 1.0], Decision::Neg)
                })
                .collect();
            let [sliver] = slivers_of_the_ball[..] else {
                panic!(
                    "ball first {ball_first}: one sliver above y = 1 and below the top, not {slivers_of_the_ball:?}"
                );
            };
            assert!(
                !inside_the_other.contains(&sliver),
                "ball first {ball_first}: the sliver is outside the block"
            );
            assert!(
                kept(Rule::Union).contains(&sliver),
                "ball first {ball_first}: a union keeps the sliver"
            );
        }
    }

    /// The side of a three-cornered bar one unit tall standing on
    /// `(2²⁰, −2²⁰)`, `(2²⁰ + 1, −2²⁰ − 1)` and `(2²⁰ + 1, −2²⁰)`: the face on
    /// `x + y = 0` between the first two, with the model and the baked bar it
    /// is a face of. Its four corners are exact and on its plane.
    fn a_far_face_on_a_plane_through_the_origin() -> (Model, BakedSolid, BakedFace) {
        let out = f64::powi(2.0, 20);
        let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
        let triangle = ProfileBuilder::new(vec![
            [out, -out],
            [out + 1.0, -out - 1.0],
            [out + 1.0, -out],
        ])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a triangle");
        let bar = model
            .add_extrusion(&triangle, 1.0, &Isometry::identity())
            .expect("a positive height");
        let [solid] = <[BakedSolid; 1]>::try_from(
            bake(
                &model,
                &[bar],
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a bar bakes"),
        )
        .expect("one solid");
        let face = solid
            .faces
            .iter()
            .find(|face| {
                face.surface.as_plane().is_some_and(|flat| {
                    let [x, y, z] = flat.normal().components();
                    x == y && z == 0.0
                })
            })
            .expect("the bar has a side on x + y = 0")
            .clone();
        let flat = face.surface.as_plane().expect("a flat face");
        assert_eq!(flat.offset(), 0.0, "through the origin");
        for edge in face.wires.iter().flatten() {
            assert_eq!(flat.side_of_point(edge.start.point), Decision::Zero);
        }
        (model, solid, face)
    }

    /// A square on the far face's plane, from `x = 2²⁰ − 1` to its near edge
    /// `x = 2²⁰` and one unit tall, its corners exact and wound the way its
    /// plane's [`InPlane`] counts as one.
    fn a_square_beside_the_far_face(flat: Surface) -> Piece {
        let out = f64::powi(2.0, 20);
        let corners = [
            [out - 1.0, 1.0 - out, 0.0],
            [out, -out, 0.0],
            [out, -out, 1.0],
            [out - 1.0, 1.0 - out, 1.0],
        ];
        let walls = [
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0, out],
            [0.0, 0.0, 1.0, -1.0],
            [1.0, 0.0, 0.0, 1.0 - out],
        ]
        .map(|wall| Surface::Plane(Plane::from_coefficients(wall).expect("a normal")));
        let corner = |which: usize| MeetPoint::OfThreeSurfaces {
            surfaces: [flat, walls[(which + 3) % 4], walls[which]],
            which: 0,
            point: PredicatePoint::from(corners[which]),
        };
        let square = (0..4)
            .map(|which| FaceMeet {
                carried_by: [flat, walls[which]],
                ends: [corner(which), corner((which + 1) % 4)],
                material_side: Decision::Neg,
                along: SmallVec::new(),
                approximation: None,
            })
            .collect();
        Piece {
            ancestors: Vec::new(),
            surface: flat,
            orientation: Orientation::Forward,
            wires: vec![square],
            is_the_whole_surface: false,
        }
    }

    /// A ball of radius 1 whose centre is `below` under the origin, and the
    /// plane `z = 0` it cuts.
    fn a_ball_cutting_z_0_round_the_origin(below: f64) -> (Sphere, Plane) {
        let ball = Sphere::new(Point::from([0.0, 0.0, -below]), 1.0).expect("a positive radius");
        let flat = Plane::from_coefficients([0.0, 0.0, 1.0, 0.0]).expect("a normal");
        (ball, flat)
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
        let meridians = the_meridian_circle_each_side_runs_round(
            piece,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a plane crosses a torus in circles");
        the_piece_holds(piece, Some(&in_plane), &also_picked_by, &meridians, point)
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
                    picked_by: SmallVec::new(),
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

    /// A face a tolerance off its apex, `2²⁰` out, holds no point its
    /// neighbours' planes put off it, and misses none they put on it.
    ///
    /// The face near the origin of
    /// [`a_far_pyramid_whose_near_face_misses_its_apex`] meets its two
    /// neighbours beside the apex the model stores, and the bake solves the
    /// apex from the three planes that leave this face out, so the apex's box
    /// is off this face's plane. [`InPlane`] reads a box off its plane where the
    /// line from its reference point through it meets the plane, which `1.5e6`
    /// out is about `1e-2` from where the face meets its neighbours. Wound round
    /// that box, the fan certified points in the sliver between the two the
    /// wrong way round, millimetres from any edge: the piece held points off
    /// it, and so did the face.
    ///
    /// The points run from each base corner towards a point beside where the
    /// face meets its neighbours, either way along the base. A point within
    /// `1e-6` of a neighbour's plane is passed over, as the face's edges are
    /// only known to a rounding or so.
    #[test]
    fn a_far_face_a_tolerance_off_its_apex_holds_no_point_its_neighbours_put_off_it() {
        let (model, solid) = a_far_pyramid_whose_near_face_misses_its_apex(1e-8);
        let face = solid
            .faces
            .iter()
            .find(|face| {
                face.surface
                    .as_plane()
                    .is_some_and(|flat| flat.offset().abs() < 0.5)
            })
            .expect("the face near the origin");
        let flat = *face.surface.as_plane().expect("a flat face");
        let wire = &face.wires[0];
        let middle_of = |point: PredicatePoint| {
            point
                .enclosure()
                .map(|side| side.lo() + (side.hi() - side.lo()) / 2.0)
        };
        let apex = wire
            .iter()
            .position(|edge| middle_of(edge.start.point)[2] > 0.5)
            .expect("an apex");
        assert!(
            matches!(
                flat.side_of_point(wire[apex].start.point),
                Decision::Pos | Decision::Neg
            ),
            "the apex's box is off the face's plane"
        );
        let neighbour = |edge: &BakedEdge| *edge.across.as_plane().expect("a flat neighbour");
        let where_the_neighbours_meet = middle_of(
            Plane::meet_of_three(
                &flat,
                &neighbour(&wire[(apex + wire.len() - 1) % wire.len()]),
                &neighbour(&wire[apex]),
            )
            .expect("three planes meet"),
        );
        let [first, second] = <[[f64; 3]; 2]>::try_from(
            wire.iter()
                .map(|edge| middle_of(edge.start.point))
                .filter(|corner| corner[2] < 0.5)
                .collect::<Vec<_>>(),
        )
        .expect("two base corners");
        let length = (0..3)
            .map(|axis| (second[axis] - first[axis]).powi(2))
            .sum::<f64>()
            .sqrt();
        let along_the_base = [0, 1, 2].map(|axis| (second[axis] - first[axis]) / length);
        let centre = [0, 1, 2]
            .map(|axis| (first[axis] + second[axis] + where_the_neighbours_meet[axis]) / 3.0);
        let is_above = |edge: &BakedEdge, point: [f64; 3]| match neighbour(edge)
            .side_of_point(PredicatePoint::from(point))
        {
            Decision::Pos => true,
            Decision::Neg => false,
            Decision::Zero | Decision::Undecidable { .. } | _ => {
                panic!("{point:?} is more than 1e-6 from every neighbour's plane")
            }
        };
        let is_off_it = |point: [f64; 3]| {
            wire.iter()
                .any(|edge| is_above(edge, point) != is_above(edge, centre))
        };
        let mut piece = Piece {
            ancestors: Vec::new(),
            surface: face.surface,
            orientation: face.orientation,
            wires: vec![
                wire.iter()
                    .map(|edge| FaceMeet {
                        carried_by: [face.surface, edge.across],
                        ends: [&edge.start, &edge.end].map(|end| MeetPoint::Existing {
                            vertex: end.vertex,
                            point: end.point,
                        }),
                        material_side: Decision::Neg,
                        along: SmallVec::new(),
                        approximation: None,
                    })
                    .collect(),
            ],
            is_the_whole_surface: false,
        };
        if the_piece_holds_asked_as_selection_asks(&piece, PredicatePoint::from(centre))
            != Some(true)
        {
            piece.wires[0].reverse();
            for side in &mut piece.wires[0] {
                side.ends.swap(0, 1);
            }
        }
        assert_eq!(
            the_piece_holds_asked_as_selection_asks(&piece, PredicatePoint::from(centre)),
            Some(true),
            "the piece holds its middle"
        );
        let mut asked = 0;
        let mut wrong = Vec::new();
        for (which, corner) in [first, second].into_iter().enumerate() {
            for beside in [1e-3, 3e-3, 1e-2, -1e-3, -3e-3, -1e-2] {
                let towards = [0, 1, 2]
                    .map(|axis| where_the_neighbours_meet[axis] + beside * along_the_base[axis]);
                for part in [0.3, 0.5, 0.7, 0.9] {
                    let point =
                        [0, 1, 2].map(|axis| corner[axis] + part * (towards[axis] - corner[axis]));
                    let nearest = wire
                        .iter()
                        .map(|edge| neighbour(edge).signed_distance_to(Point::from(point)).abs())
                        .fold(f64::INFINITY, f64::min);
                    if nearest <= 1e-6 {
                        continue;
                    }
                    asked += 1;
                    let is_off = is_off_it(point);
                    let piece_holds = the_piece_holds_asked_as_selection_asks(
                        &piece,
                        PredicatePoint::from(point),
                    );
                    let face_holds = a_face_of_the_input_holds(
                        &model,
                        &solid,
                        face,
                        &piece,
                        PredicatePoint::from(point),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .expect("a flat face with straight sides answers");
                    if piece_holds == Some(is_off) || face_holds == is_off {
                        wrong.push(format!(
                            "from base corner {which}, {beside:e} along the base, {part} of the way: off {is_off}, piece {piece_holds:?}, face {face_holds}"
                        ));
                    }
                }
            }
        }
        assert!(asked >= 40, "only {asked} points are clear of every edge");
        assert!(
            wrong.is_empty(),
            "{} of {asked} points answered against the neighbours' planes: {wrong:#?}",
            wrong.len()
        );
    }

    /// A pyramid on a square base `2²⁰` out, turned `2.5` about the `z` axis,
    /// whose face on `x + y = 0` (before the turn) passes through its two base
    /// corners and `miss` beside its apex: the plane is put through the apex
    /// moved sideways by `miss`, and the apex stored is not moved. The model's
    /// room at 100 mm is `1e-7`, so it takes a `miss` below that.
    fn a_far_pyramid_whose_near_face_misses_its_apex(miss: f64) -> (Model, BakedSolid) {
        let out = f64::powi(2.0, 20);
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let turned = Isometry::rotation(
            &Axis::new(Point::from([0.0, 0.0, 0.0]), Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
            2.5,
        )
        .expect("a turn");
        let sideways = miss / f64::sqrt(2.0);
        let apex_the_near_face_passes =
            turned.apply_to_point(Point::new(out + 0.5 + sideways, -out - 0.5 + sideways, 1.0));
        let points = [
            [out + 0.5, -out - 0.5, 1.0],
            [out, -out, 0.0],
            [out + 1.0, -out - 1.0, 0.0],
            [out + 2.0, -out, 0.0],
            [out + 1.0, -out + 1.0, 0.0],
        ]
        .map(|[x, y, z]| turned.apply_to_point(Point::new(x, y, z)));
        let faces: [&[usize]; 5] = [
            &[0, 1, 2],
            &[0, 2, 3],
            &[0, 3, 4],
            &[0, 4, 1],
            &[1, 4, 3, 2],
        ];
        let mut builder = crate::SolidBuilder::new();
        let vertices = points.map(|point| builder.add_vertex(point).expect("a finite corner"));
        let mut edges: Vec<([usize; 2], crate::DraftEdge)> = Vec::new();
        let drafted = faces.map(|face| {
            let steps: Vec<crate::DraftStep> = (0..face.len())
                .map(|position| {
                    let (from, to) = (face[position], face[(position + 1) % face.len()]);
                    let (edge, orientation) =
                        match edges.iter().find(|(ends, _)| *ends == [to, from]) {
                            Some(&(_, edge)) => (edge, Orientation::Reversed),
                            None => {
                                let edge = builder
                                    .add_edge(vertices[from], vertices[to], EdgeRun::Straight)
                                    .expect("a side");
                                edges.push(([from, to], edge));
                                (edge, Orientation::Forward)
                            }
                        };
                    crate::DraftStep::Along(builder.add_coedge(edge, orientation).expect("a use"))
                })
                .collect();
            let wire = builder.add_wire(&steps).expect("a closed boundary");
            let [first, second, third] = if face.len() == 4 {
                [face[0], face[1], face[2]]
            } else {
                [face[1], face[2], face[0]]
            };
            let at = |index: usize| {
                if face[..] == [0, 1, 2] && index == 0 {
                    apex_the_near_face_passes
                } else {
                    points[index]
                }
            };
            let plane = Plane::through_three_points(at(first), at(second), at(third))
                .expect("three corners");
            builder
                .add_face(Surface::Plane(plane), &[wire])
                .expect("a face")
        });
        builder
            .add_shell(&drafted.map(|face| (face, Orientation::Forward)))
            .expect("five faces");
        let pyramid = builder
            .build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a face less than the room off its apex is taken");
        let [solid] = <[BakedSolid; 1]>::try_from(
            bake(
                &model,
                &[pyramid],
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("the pyramid bakes"),
        )
        .expect("one solid");
        (model, solid)
    }

    /// A half disk `2²⁰` out on `x + y = 0`, cut from it by a ball round
    /// `(2²⁰, −2²⁰, 0)` and the plane square to it through that centre, holds
    /// no point across its chord when its two corners are points `1e-8` off its
    /// plane, either side.
    ///
    /// Two corners wind round nothing, so what holds a point is the bulge:
    /// which side of the chord between the corners it is on. [`InPlane`] reads
    /// a corner `1e-8` off the plane about `1.5e-2` along it, out there, and
    /// both corners the same way across the chord, so the chord as read held
    /// points short of it or missed points past it. The bulge stretches the
    /// chord's ends to the plane, as [`the_piece_holds`] stretches the corners
    /// it winds round.
    #[test]
    fn a_far_half_disk_whose_corners_are_off_its_plane_holds_no_point_across_its_chord() {
        let out = f64::powi(2.0, 20);
        let flat = Plane::from_coefficients([1.0, 1.0, 0.0, 0.0]).expect("a normal");
        let ball = Surface::Sphere(
            Sphere::new(Point::from([out, -out, 0.0]), 1.0).expect("a positive radius"),
        );
        let across = Surface::Plane(
            Plane::from_coefficients([1.0, -1.0, 0.0, -2.0 * out]).expect("a normal"),
        );
        for off in [1e-8, -1e-8] {
            let aside = off / f64::sqrt(2.0);
            let corner = |z: f64| MeetPoint::OfThreeSurfaces {
                surfaces: [Surface::Plane(flat), ball, across],
                which: 0,
                point: PredicatePoint::from([out + aside, -out + aside, z]),
            };
            let side = |partner: Surface, ends: [MeetPoint; 2], material_side: Decision| FaceMeet {
                carried_by: [Surface::Plane(flat), partner],
                ends,
                material_side,
                along: SmallVec::new(),
                approximation: None,
            };
            let half_disk = Piece {
                ancestors: Vec::new(),
                surface: Surface::Plane(flat),
                orientation: Orientation::Forward,
                wires: vec![vec![
                    side(ball, [corner(1.0), corner(-1.0)], Decision::Neg),
                    side(across, [corner(-1.0), corner(1.0)], Decision::Pos),
                ]],
                is_the_whole_surface: false,
            };
            let at = |beyond_the_chord: f64, z: f64| {
                PredicatePoint::from([out + beyond_the_chord, -out - beyond_the_chord, z])
            };
            assert_eq!(
                the_piece_holds_asked_as_selection_asks(&half_disk, at(0.5, 0.0)),
                Some(true),
                "{off:e} off: the half disk holds its middle"
            );
            assert_eq!(
                the_piece_holds_asked_as_selection_asks(&half_disk, at(-0.5, 0.0)),
                Some(false),
                "{off:e} off: the half disk holds nothing of the other half"
            );
            let mut wrong = Vec::new();
            for beyond_the_chord in [1e-3, 3e-3, 6e-3, -1e-3, -3e-3, -6e-3] {
                for z in [-0.3, 0.0, 0.3] {
                    let held = the_piece_holds_asked_as_selection_asks(
                        &half_disk,
                        at(beyond_the_chord, z),
                    );
                    if held == Some(beyond_the_chord < 0.0) {
                        wrong.push(format!(
                            "{beyond_the_chord:e} past the chord at z {z}: {held:?}"
                        ));
                    }
                }
            }
            assert!(wrong.is_empty(), "{off:e} off: {wrong:#?}");
        }
    }
}
