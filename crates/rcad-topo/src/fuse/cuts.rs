//! Which lines cut which faces.
//!
//! A cut line is always a *pair of planes* and never a point and a direction
//! (§17.4). Two faces whose planes are not parallel contribute the pair
//! `{plane(A), plane(B)}`. Two coplanar faces contribute one line per edge of
//! `B`, each as `{plane(A), plane(B')}` for the face `B'` across that edge —
//! because the boundary of `B` is what cuts `A`, and each of its edges already
//! lies on two planes. That is the only place the coplanar case is special: the
//! arrangement sees one uniform kind of input.
//!
//! Lines are gathered between **every pair of input faces**, whether the two
//! belong to different solids or to one, and whether the two solids are in the
//! same group or not. `A` is the union of the arguments and `B` the union of
//! the tools (§17.4), and a union is only real if the solids making it up have
//! been cut against each other: otherwise two overlapping arguments keep the
//! halves of each other's faces that are buried inside the other one, and the
//! answer is a self-overlapping shell rather than a union.
//!
//! # Every face is cut by every plane, its own solid's included
//!
//! Two solids are skipped whole only when no chain of solids that may meet
//! joins them, and two may meet unless it is proved that no face of the one
//! shares a point with a face of the other (§400): solids that nothing links
//! leave every face of each other alone, whether they stand apart or one is
//! wholly inside the other. Two that are apart but are both reached by a third
//! are still cut against each other, because the third's faces are cut by both
//! and the edges it shares with each must be split at the same points (§222.3
//! item 1). Inside a pair that might touch, **no face pair is skipped**, and a
//! box test there would be wrong rather than merely keen.
//!
//! A cut is by a *plane*, and a plane reaches past the face that carries it. A
//! face whose box is nowhere near the other solid can still be crossed by one
//! of its planes, and if its neighbour is crossed by that plane too then the
//! two are split at the same point of the edge between them. Skip the pair the
//! box test calls hopeless and only one side of that edge gets split, so the
//! two no longer meet along one edge and the result is not a manifold. §17.4
//! throws the BVH out because a broad phase "changes no answer"; a face-level
//! box test here does change one, which is the same argument arriving from the
//! other side.
//!
//! **A solid's own planes cut its own faces for that same reason** (§48, W1).
//! Cutting a face only by the *other* solid's planes keeps the two sides of an
//! edge in step for as long as both sides come from one solid, and the answer
//! is under no obligation to do that. Where the two solids are flush a region
//! has a face over it in both, the rule keeps one copy, and which copy it keeps
//! is not the same everywhere on the plane — so a piece of `A`'s face ends up
//! along an edge from a piece of `B`'s. `A`'s piece is split where `B`'s planes
//! cross that edge and `B`'s piece where `A`'s do, and those are two different
//! sets of points.
//!
//! A block two wide against a frame with a one-wide hole is that pair, on
//! whole numbers, with nothing rounded anywhere: the frame's hole wall cuts the
//! block's wall at `x = 1`, nothing of the block sits on `x = 1`, and the
//! frame's own wall is left whole. United one way round it was a solid and the
//! other way round an edge with one face along it. Cutting every face by every
//! plane of both solids is the closure of the argument above: every face on
//! either side of any edge of the answer is then diced by the one set, so the
//! two sides agree about where the corners are wherever they come from.
//! [`assemble`](super::assemble)'s joining folds the extra pieces of a plane, a
//! cone, a pipe or a ball back into one face (§229.2), and on a plane drops
//! the corners that stopped being corners.
//!
//! **Some of the split still reaches the answer.** A corner where pieces of
//! one curved surface met stays a corner; a band round a cone or a pipe keeps
//! one seam, and a group on one whose rings cannot be counted, or that has no
//! seam to keep, stays in its pieces; pieces that make up a whole ball stay
//! apart; and a ring's pieces are never joined at all.
//!
//! # A crossing that splits nothing and is no edge is left off
//!
//! The argument above is about edges: a crossing has to be on a face because
//! it splits the face's edges where it splits its neighbours'. A crossing that
//! **shares no point with any other surface the face is bounded or cut by**
//! splits no edge, of this face or any other, so that argument asks nothing of
//! it. Every edge of the face's pieces lies along one of those other surfaces.
//! What such a crossing still does is cut the face's inside into the part on
//! either side of it, and that is wanted only where it is an edge of the
//! answer: where the face that carries it reaches it.
//!
//! **So it is left off when that face provably misses the whole surface it
//! cuts** (Q1090). A tube from `z = −10` to `20` and a block from `0` to `22`
//! about it: the pipes cross the block's top in circles of radius 4 and 12
//! that meet nothing else on it, and the tube's walls never come within two of
//! `z = 22`. Cut along them, the ring between was a piece that selection could
//! not find a point inside, and the fuse was refused. Left off, the circles
//! are no loss. No boundary of either solid passes within reach of them,
//! because any surface that did would share a point with them, so the top on
//! one side of each circle is inside or outside each solid exactly where the
//! other side is.
//!
//! The face missing the surface is read off the band round its axis, the test
//! [`two_faces_provably_apart`] uses, and that the crossing meets nothing else
//! is asked the way the arrangement places its nodes; both are proofs and
//! neither is a box. A surface carried by two faces stays on wherever one of
//! them does not say so. The band is asked of a plane leaning to its axis as
//! well (Q1174): the same block with its top leaning, `4z = x + 96`, clear of
//! the tube, was cut along the ellipses where the pipes cross it and refused
//! the same way, because the band answered only for a plane square to it.
//!
//! # Two gaps, left open on purpose
//!
//! A solid is only refused for running through itself when the overlap is
//! *proved*. Two faces whose stretches merely touch inside their enclosures have
//! not been shown to cross, and refusing them would refuse a sound solid whose
//! faces meet at a single corner.
//!
//! Two **coplanar** faces of one solid are not checked against each other at
//! all. The pair carries no cut line — a plane does not cut itself — so an
//! overlap between them would have to be found by comparing the two outlines in
//! their shared plane, and nothing here does that. A solid built by folding one
//! face back onto another is therefore not caught. Both gaps belong with roadmap
//! §7.4's catalogue of degenerate inputs, which is where the cases that would
//! close them get written.

use std::collections::HashMap;

use rcad_geom::{
    MeetOfThree, PlaneAgreement, Surface, SurfaceMeet, intersect, meet_of_three_surfaces,
};
use rcad_predicates::{Decision, Policy, ProvenanceLog};

use crate::{EdgeKey, Error, Result};

use super::Sameness;
use super::arrange::stretches_of_a_line_inside;
use super::bake::{BakedFace, BakedSolid};

/// Work out every cut line, and hang each on the face it cuts.
///
/// Every face is cut against every other face, in its own solid and in every
/// other, whichever group each came from. The header says why the solid's own
/// planes are in there, and why a crossing that splits nothing and lies where
/// its face never reaches is then left off.
///
/// # Errors
///
/// [`Error::ArgumentIntersectsItself`] when a solid has two faces that really do
/// run through each other, [`Error::Undecidable`] when a decision that takes
/// could not be settled under `policy`, and
/// [`Error::ACrossingThisCannotBeOrderedAlong`] when two surfaces meet in a
/// shape [`how_two_faces_meet`] has no word for.
pub(crate) fn gather(
    solids: &mut [BakedSolid],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<()> {
    for solid in solids.iter() {
        check_it_does_not_cross_itself(solid, policy, log)?;
    }

    let within_reach = solids_within_reach_of_each_other(solids);
    let mut found: Vec<Vec<Vec<ACutFound>>> = solids
        .iter()
        .map(|solid| vec![Vec::new(); solid.faces.len()])
        .collect();
    for one in 0..solids.len() {
        for other in one..solids.len() {
            if within_reach.leader(one) != within_reach.leader(other) {
                continue;
            }
            for mine in 0..solids[one].faces.len() {
                let from = if one == other { mine + 1 } else { 0 };
                for theirs in from..solids[other].faces.len() {
                    let face = &solids[one].faces[mine];
                    let against = &solids[other].faces[theirs];
                    let (onto_face, onto_against) = cuts_between(face, against, policy, log)?;
                    found[one][mine].extend(found_on(face, against, onto_face));
                    found[other][theirs].extend(found_on(against, face, onto_against));
                }
            }
        }
    }

    for (solid, faces) in found.into_iter().enumerate() {
        for (face, cuts) in faces.into_iter().enumerate() {
            let kept = the_cuts_left_on(&solids[solid].faces[face], &cuts);
            solids[solid].faces[face].cuts.extend(kept);
            solids[solid].faces[face].cuts_that_reach_only_its_boundary =
                the_cuts_that_reach_only_its_boundary(&cuts);
        }
    }
    for solid in solids.iter_mut() {
        tell_each_edge_what_its_neighbour_leaves_out(solid);
    }
    Ok(())
}

/// A cut found for a face, before it is hung on it.
#[derive(Clone, Copy, Debug)]
struct ACutFound {
    cut: Surface,
    /// Whether the face that carries `cut` provably shares no point with the
    /// whole surface of the face it cuts, so that no point of their crossing is
    /// on that face.
    its_face_misses_this_surface: bool,
    /// Whether `cut` is the surface of the face that carries it and that face
    /// provably meets the face it cuts on that face's boundary alone
    /// ([`meets_it_only_on_its_boundary`]).
    reaches_only_its_boundary: bool,
}

fn found_on(face: &BakedFace, against: &BakedFace, cuts: Vec<Surface>) -> Vec<ACutFound> {
    let its_face_misses_this_surface =
        !cuts.is_empty() && a_face_misses_a_surface_by_its_band(against, &face.surface);
    let reaches_only_its_boundary =
        !cuts.is_empty() && meets_it_only_on_its_boundary(face, against);
    cuts.into_iter()
        .map(|cut| ACutFound {
            cut,
            its_face_misses_this_surface,
            reaches_only_its_boundary: reaches_only_its_boundary && cut == against.surface,
        })
        .collect()
}

/// Whether every point `against` shares with `face` is on `face`'s boundary,
/// by a plane `against` lies on one side of: `face` lies on the other side,
/// so the two share points of the plane alone and `face`'s inside misses the
/// plane ([`BakedFace::lies_in`]); or the box round `face` is strictly on the
/// other side, so the two share nothing.
fn meets_it_only_on_its_boundary(face: &BakedFace, against: &BakedFace) -> bool {
    let anywhere_on_the_face = face.bounds.as_a_point_anywhere_inside();
    against.lies_in.iter().any(|theirs| {
        let on_the_other_side = face.lies_in.iter().any(|mine| {
            let agreement = mine.plane.agreement_with(&theirs.plane);
            (agreement == PlaneAgreement::SameWay && mine.behind != theirs.behind)
                || (agreement == PlaneAgreement::OppositeWay && mine.behind == theirs.behind)
        });
        let past_it = if theirs.behind {
            Decision::Pos
        } else {
            Decision::Neg
        };
        on_the_other_side
            || anywhere_on_the_face
                .is_some_and(|anywhere| theirs.plane.side_of_point(anywhere) == past_it)
    })
}

/// Each surface every finding of which reaches only the face's boundary.
fn the_cuts_that_reach_only_its_boundary(found: &[ACutFound]) -> Vec<Surface> {
    let mut left_out: Vec<Surface> = Vec::new();
    for one in found.iter().filter(|one| one.reaches_only_its_boundary) {
        if !left_out.contains(&one.cut)
            && found
                .iter()
                .filter(|other| other.cut == one.cut)
                .all(|other| other.reaches_only_its_boundary)
        {
            left_out.push(one.cut);
        }
    }
    left_out
}

/// Fill in [`BakedEdge::its_neighbour_leaves_out`](super::bake::BakedEdge::its_neighbour_leaves_out)
/// on every edge of `solid`: the cuts every other face of the solid along one
/// of the edge's input edges leaves out.
fn tell_each_edge_what_its_neighbour_leaves_out(solid: &mut BakedSolid) {
    if solid
        .faces
        .iter()
        .all(|face| face.cuts_that_reach_only_its_boundary.is_empty())
    {
        return;
    }
    let mut faces_along: HashMap<EdgeKey, Vec<usize>> = HashMap::new();
    for (index, face) in solid.faces.iter().enumerate() {
        for edge in face.wires.iter().flatten() {
            for key in &edge.keys {
                faces_along.entry(*key).or_default().push(index);
            }
        }
    }
    for index in 0..solid.faces.len() {
        for wire in 0..solid.faces[index].wires.len() {
            for position in 0..solid.faces[index].wires[wire].len() {
                let neighbours: Vec<usize> = solid.faces[index].wires[wire][position]
                    .keys
                    .iter()
                    .flat_map(|key| faces_along[key].iter().copied())
                    .filter(|&other| other != index)
                    .collect();
                let Some((&first, rest)) = neighbours.split_first() else {
                    continue;
                };
                let left_out: Vec<Surface> = solid.faces[first]
                    .cuts_that_reach_only_its_boundary
                    .iter()
                    .copied()
                    .filter(|cut| {
                        rest.iter().all(|&other| {
                            solid.faces[other]
                                .cuts_that_reach_only_its_boundary
                                .contains(cut)
                        })
                    })
                    .collect();
                solid.faces[index].wires[wire][position].its_neighbour_leaves_out = left_out;
            }
        }
    }
}

/// The cuts to hang on `face`: every one found, but a crossing that splits
/// nothing on the face and lies where the face carrying it never reaches
/// (the header's last section).
///
/// Such a crossing is one whose carrying face provably misses `face`'s whole
/// surface, and which provably shares no point with any other surface `face`
/// is bounded or cut by. One surface carried by two faces is left off only
/// where both say so, since each finding is weighed on its own.
fn the_cuts_left_on(face: &BakedFace, found: &[ACutFound]) -> Vec<Surface> {
    let others: Vec<Surface> = face
        .wires
        .iter()
        .flatten()
        .map(|edge| edge.across)
        .chain(found.iter().map(|one| one.cut))
        .collect();
    found
        .iter()
        .filter(|one| {
            !(one.its_face_misses_this_surface
                && the_crossing_meets_none_of(&face.surface, &one.cut, &others))
        })
        .map(|one| one.cut)
        .collect()
}

/// Whether the crossing of `surface` and `cut` provably shares no point with
/// any of `others` but `cut` itself.
///
/// Asked the way [`place_the_meets`](super::arrange) asks for a node: where
/// the three surfaces meet, and where that cannot be worked out, whether the
/// two partners meet at all. Under a certified policy, with its records
/// dropped, for the reason [`two_faces_provably_apart`] gives: a crossing left
/// off on a guess is an edge of the answer lost.
fn the_crossing_meets_none_of(surface: &Surface, cut: &Surface, others: &[Surface]) -> bool {
    let policy = Policy::certified();
    others.iter().filter(|other| *other != cut).all(|other| {
        let mut dropped = ProvenanceLog::new();
        match meet_of_three_surfaces([surface, cut, other], &policy, &mut dropped) {
            Ok(MeetOfThree::Nowhere) => true,
            Ok(MeetOfThree::At(_) | MeetOfThree::MoreThanAPoint) => false,
            Ok(_) => false,
            Err(_) => matches!(
                intersect(cut, other, &policy, &mut dropped),
                Ok(SurfaceMeet::Nothing)
            ),
        }
    })
}

/// Which solids are joined to each other by a chain of solids that may meet.
///
/// **Two solids whose boxes miss are still cut against each other when a third
/// solid reaches both** (§222.3 item 1). A block with one box poking up
/// through its floor and another sitting inside it: the inside box's walls cut
/// the block's floor, and that floor meets the poking box's walls along the
/// footprint. Skip the poking box against the inside one and the floor side of
/// that edge is split where the inside box's walls cross it while the wall side
/// is not, so the edge comes back with one face along it. It is the header's
/// argument one solid further out: both sides of every edge of the answer have
/// to be diced by the one set of planes, and that set is everything in the
/// chain.
///
/// **Two solids are joined unless it is proved that they do not meet** (§400).
/// Whole-solid boxes that touch were the test until then, and a block's plane
/// then cut a ball standing clear of the block in two, a ball nothing folds
/// back together. Two solids no face of which shares a point have no edge of
/// the answer between them, so the header's argument asks nothing of their
/// planes. Proved, not guessed: a pair this cannot settle is joined, so a
/// refusal or an answer moves only where the solids are really apart.
fn solids_within_reach_of_each_other(solids: &[BakedSolid]) -> Sameness {
    let mut within_reach = Sameness::of(solids.len());
    for one in 0..solids.len() {
        for other in one + 1..solids.len() {
            if solids[one].bounds.overlap(solids[other].bounds)
                && !no_face_of_one_meets_the_other(&solids[one], &solids[other])
            {
                within_reach.join(one, other);
            }
        }
    }
    within_reach
}

/// Whether it is proved that no face of `one` shares a point with any face of
/// `other`, edges and corners included.
///
/// That is enough for two solids to be left uncut against each other, whether
/// they stand apart or one lies wholly inside the other: either way no edge of
/// the answer has a face of the one on one side and a face of the other on the
/// other, and selection places each uncut face against the other solid as a
/// whole.
fn no_face_of_one_meets_the_other(one: &BakedSolid, other: &BakedSolid) -> bool {
    one.faces.iter().all(|face| {
        other
            .faces
            .iter()
            .all(|against| two_faces_provably_apart(face, against))
    })
}

/// Whether `face` and `against` provably share no point.
///
/// Every point the two share is on both surfaces and inside both faces' boxes,
/// so any one of these proves there is none: the two boxes miss; one surface
/// misses the part of space both boxes hold; one surface misses the band round
/// an axis that the other face lies in ([`band_a_face_is_inside`]); the
/// surfaces do not meet at all; or they touch at one point outside that part of
/// space.
///
/// **The band is what settles one round solid inside another about one axis.**
/// A frustum of radius 1 to 3 inside a bar of radius 4: the box round the
/// frustum's top cap reaches `√18` from the axis at its corners, so the bar's
/// wall runs through that box although it never comes within 1 of the cap. Round
/// the axis the cap reaches 3 out and the wall is 4 out, and nothing in between
/// is a guess.
///
/// **Where one of the two has no box here, `bake`'s own boxes are asked
/// whether they miss.** Those are read off stored corners in plain `f64`
/// (§35.4), grown by what writing a corner down costs (Q694, §340.10), but
/// still plain `f64`, so a miss between them is still not a proof. It is the
/// test the two whole solids' boxes, those same boxes taken together, have
/// always passed as one, and it is asked only of a face on a pipe, a cone or
/// a torus, or a flat face with a side on one, which nothing here bounds
/// (§400.3). Without it a block holding a cube clear of a frustum through its
/// floor cut the frustum along the cube's planes in one grouping of a fuse
/// and not in the other.
///
/// **Asked under a certified policy whatever the fuse runs under, and its
/// records dropped.** A tolerant policy settling a close call one way would
/// skip a cut on a guess. A pair this cannot settle is not refused here: it is
/// joined, and the cutting that follows asks the same surfaces again under the
/// fuse's own policy and log, as it did before there was a proof.
fn two_faces_provably_apart(face: &BakedFace, against: &BakedFace) -> bool {
    let (mine, theirs) = (box_a_face_is_inside(face), box_a_face_is_inside(against));
    if (mine.is_none() || theirs.is_none()) && !face.bounds.overlap(against.bounds) {
        return true;
    }
    let both = match (mine, theirs) {
        (Some(mine), Some(theirs)) => match where_two_boxes_overlap(mine, theirs) {
            Some(both) => Some(both),
            None => return true,
        },
        (Some(only), None) | (None, Some(only)) => Some(only),
        (None, None) => None,
    };
    if both.is_some_and(|both| {
        a_surface_misses_a_box(&face.surface, both)
            || a_surface_misses_a_box(&against.surface, both)
    }) {
        return true;
    }
    if a_face_misses_a_surface_by_its_band(face, &against.surface)
        || a_face_misses_a_surface_by_its_band(against, &face.surface)
    {
        return true;
    }
    let Ok(meet) = intersect(
        &face.surface,
        &against.surface,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) else {
        return false;
    };
    if let SurfaceMeet::TouchAtOnePoint { at } = meet {
        return both.is_some_and(|both| where_two_boxes_overlap(both, at.enclosure()).is_none());
    }
    matches!(meet, SurfaceMeet::Nothing)
}

/// Whether `face` provably shares no point with `surface`, read off the band
/// round an axis the face lies in.
fn a_face_misses_a_surface_by_its_band(face: &BakedFace, surface: &Surface) -> bool {
    band_a_face_is_inside(face).is_some_and(|band| a_surface_misses_a_band(surface, band))
}

/// A box the face is certainly inside, rounded outward, or `None` where this
/// cannot bound the face.
///
/// - A face on a **ball** is inside the box round the whole ball.
/// - A **flat** face is inside the hull of its boundary. A straight side lies
///   between its two corners, and a corner is inside the box it carries; a
///   side on a ball is inside that ball's box.
/// - A face on anything else, and a flat face with a side on anything else, is
///   not bounded here. `bake`'s own boxes are not used: they are read off
///   stored corners in plain `f64` (§35.4), and a surface missing the part of
///   one such box another holds could be a rounding of where the two touch.
fn box_a_face_is_inside(face: &BakedFace) -> Option<[rcad_predicates::Interval; 3]> {
    if let Surface::Sphere(ball) = &face.surface {
        return box_round_a_ball(ball);
    }
    face.surface.as_plane()?;
    let mut round_it: Option<[rcad_predicates::Interval; 3]> = None;
    for edge in face.wires.iter().flatten() {
        let side = if let Surface::Sphere(ball) = &edge.across {
            box_round_a_ball(ball)?
        } else {
            edge.across.as_plane()?;
            both_boxes(edge.start.point.enclosure(), edge.end.point.enclosure())
        };
        round_it = Some(round_it.map_or(side, |so_far| both_boxes(so_far, side)));
    }
    round_it
}

fn box_round_a_ball(ball: &rcad_geom::Sphere) -> Option<[rcad_predicates::Interval; 3]> {
    let [x, y, z] = ball
        .centre()
        .coordinates()
        .map(|centre| rcad_predicates::Interval::around(centre, ball.radius()));
    Some([x.ok()?, y.ok()?, z.ok()?])
}

/// The smallest box holding both. An axis whose ends are not numbers holds
/// everything, which is never too small.
fn both_boxes(
    one: [rcad_predicates::Interval; 3],
    other: [rcad_predicates::Interval; 3],
) -> [rcad_predicates::Interval; 3] {
    core::array::from_fn(|axis| {
        rcad_predicates::Interval::new(
            one[axis].lo().min(other[axis].lo()),
            one[axis].hi().max(other[axis].hi()),
        )
        .unwrap_or(rcad_predicates::Interval::ENTIRE)
    })
}

/// The part of space both boxes hold, or `None` when they provably hold none.
/// Boxes that touch share what they touch along, and an axis whose ends are not
/// numbers is taken to hold everything.
fn where_two_boxes_overlap(
    one: [rcad_predicates::Interval; 3],
    other: [rcad_predicates::Interval; 3],
) -> Option<[rcad_predicates::Interval; 3]> {
    let mut both = [rcad_predicates::Interval::ENTIRE; 3];
    for axis in 0..3 {
        let (low, high) = (
            one[axis].lo().max(other[axis].lo()),
            one[axis].hi().min(other[axis].hi()),
        );
        if low > high {
            return None;
        }
        both[axis] =
            rcad_predicates::Interval::new(low, high).unwrap_or(rcad_predicates::Interval::ENTIRE);
    }
    Some(both)
}

/// Whether `surface` provably passes through no point of `room`.
///
/// A plane misses a box when its equation keeps one sign over the whole box. A
/// ball's skin misses one that lies wholly outside the ball or wholly inside
/// it. Both are worked in interval arithmetic on the numbers the surface
/// stores, which are the surface (§17.1), so a sign proved here is a sign of
/// the surface itself. Any other surface proves nothing.
fn a_surface_misses_a_box(surface: &Surface, room: [rcad_predicates::Interval; 3]) -> bool {
    if let Surface::Plane(flat) = surface {
        let [a, b, c, d] = flat.coefficients();
        let over_the_box = rcad_predicates::Interval::exactly(a) * room[0]
            + rcad_predicates::Interval::exactly(b) * room[1]
            + rcad_predicates::Interval::exactly(c) * room[2]
            + rcad_predicates::Interval::exactly(d);
        return matches!(
            over_the_box.sign(),
            rcad_predicates::Decision::Pos | rcad_predicates::Decision::Neg
        );
    }
    let Surface::Sphere(ball) = surface else {
        return false;
    };
    let centre = ball.centre().coordinates();
    let mut nearest = rcad_predicates::Interval::exactly(0.0);
    let mut furthest = rcad_predicates::Interval::exactly(0.0);
    for axis in 0..3 {
        let gap = room[axis] - rcad_predicates::Interval::exactly(centre[axis]);
        let near = if gap.lo() > 0.0 {
            gap.lo()
        } else if gap.hi() < 0.0 {
            -gap.hi()
        } else {
            0.0
        };
        let far = gap.lo().abs().max(gap.hi().abs());
        nearest = nearest
            + rcad_predicates::Interval::exactly(near) * rcad_predicates::Interval::exactly(near);
        furthest = furthest
            + rcad_predicates::Interval::exactly(far) * rcad_predicates::Interval::exactly(far);
    }
    let skin = rcad_predicates::Interval::exactly(ball.radius())
        * rcad_predicates::Interval::exactly(ball.radius());
    matches!((nearest - skin).sign(), rcad_predicates::Decision::Pos)
        || matches!((furthest - skin).sign(), rcad_predicates::Decision::Neg)
}

/// Where a face lies round one axis: how far up it and how far out from it.
///
/// Both are measured the way the stored surfaces measure them, with the axis's
/// direction `w` as stored and not scaled to length one: `up` holds
/// `(p − origin)·w` and `out` holds `|(p − origin) × w|²` for every point `p`
/// of the face. A pipe of radius `r` about the line is then every point with
/// `out = r²·|w|²`, and a cone every point with `out = slope²·(up − its apex's
/// up)²`, whatever the length of `w` — the expressions the two side-of
/// predicates in `rcad-geom` take the sign of.
#[derive(Clone, Copy, Debug)]
struct ABandRoundAnAxis {
    origin: [f64; 3],
    along: [f64; 3],
    up: rcad_predicates::Interval,
    out: rcad_predicates::Interval,
}

/// The band round an axis the face certainly lies in, worked in interval
/// arithmetic, or `None` where this cannot bound the face that way.
///
/// - A face on a **pipe** or a **cone** whose every side is where a plane
///   square to the axis crosses it. Each such side keeps one `up`, the `up` of
///   any corner on it. Through every point of either surface but a cone's apex
///   runs a straight ruling along which `up` changes at a steady rate, so the
///   least and the most `up` on the face are on its sides, or at the apex: a
///   cone's band always reaches down to its apex, which only ever widens it. A
///   seam is not a side here, since the face lies on both sides of it, and
///   `bake` leaves seams out of the wires.
/// - A **flat** face square to an axis, whose every side is straight or where
///   the plane crosses a pipe or a cone about that axis. Such a side is a circle
///   about the axis, where `out` is the same all round, so the corners on it
///   say what it is; a straight side is no farther out anywhere than at its
///   ends. A flat face is inside the hull of its sides and `out` only grows away
///   from the axis, so the face is no farther out than its farthest corner.
/// - Anything else is not banded. A side running anywhere else could reach
///   past every corner on it.
fn band_a_face_is_inside(face: &BakedFace) -> Option<ABandRoundAnAxis> {
    let sides = || face.wires.iter().flatten();
    let corners =
        sides().flat_map(|side| [side.start.point.enclosure(), side.end.point.enclosure()]);
    let (origin, along) = match &face.surface {
        Surface::Cylinder(pipe) => axis_numbers(pipe.axis()),
        Surface::Cone(cone) => axis_numbers(cone.axis()),
        Surface::Plane(_) => sides().find_map(|side| match &side.across {
            Surface::Cylinder(pipe) => Some(axis_numbers(pipe.axis())),
            Surface::Cone(cone) => Some(axis_numbers(cone.axis())),
            Surface::Plane(_) | Surface::Sphere(_) | Surface::Torus(_) => None,
            _ => None,
        })?,
        Surface::Sphere(_) | Surface::Torus(_) => return None,
        _ => return None,
    };
    let mut band = ABandRoundAnAxis {
        origin,
        along,
        up: rcad_predicates::Interval::exactly(0.0),
        out: rcad_predicates::Interval::exactly(0.0),
    };
    let up = corners
        .clone()
        .map(|corner| band.up_of(corner))
        .reduce(hull)?;
    match &face.surface {
        Surface::Cylinder(pipe) => {
            if !sides().all(|side| band.is_square_to(&side.across)) {
                return None;
            }
            band.up = up;
            band.out = squared(rcad_predicates::Interval::exactly(pipe.radius())) * band.length();
        }
        Surface::Cone(cone) => {
            if !sides().all(|side| band.is_square_to(&side.across)) {
                return None;
            }
            band.up = hull(up, rcad_predicates::Interval::exactly(0.0));
            band.out = squared(rcad_predicates::Interval::exactly(cone.slope())) * squared(band.up);
        }
        Surface::Plane(_) => {
            if !band.is_square_to(&face.surface)
                || !sides().all(|side| match &side.across {
                    Surface::Plane(_) => true,
                    Surface::Cylinder(pipe) => band.is_on_the_axis(pipe.axis()),
                    Surface::Cone(cone) => band.is_on_the_axis(cone.axis()),
                    Surface::Sphere(_) | Surface::Torus(_) => false,
                    _ => false,
                })
            {
                return None;
            }
            band.up = up;
            band.out = corners
                .map(|corner| band.out_of(corner))
                .fold(rcad_predicates::Interval::exactly(0.0), hull);
        }
        Surface::Sphere(_) | Surface::Torus(_) => return None,
        _ => return None,
    }
    Some(band)
}

/// Whether `surface` provably passes through no point of `band`.
///
/// A surface about the band's own axis is asked as one expression: a pipe or a
/// cone on that line, a ball centred on it, or a plane square to it. Each is
/// one expression in `up` and `out`, worked over the band in interval
/// arithmetic on the numbers the surface stores (§17.1), and a sign proved for
/// it is a sign of the surface itself. A ball is `out + (up − its centre's
/// up)² = radius²·|w|²`, which is Lagrange's identity, and a plane square to
/// the line is `(n·w)·(up + w·origin) + d·|w|² = 0`, its own equation scaled
/// by `|w|²`. A plane leaning to the line is asked by
/// [`a_leaning_plane_misses_a_band`]; anything else is not asked.
fn a_surface_misses_a_band(surface: &Surface, band: ABandRoundAnAxis) -> bool {
    let exactly = rcad_predicates::Interval::exactly;
    let over_the_band = match surface {
        Surface::Plane(flat) => {
            if !band.is_square_to(surface) {
                return a_leaning_plane_misses_a_band(flat, band);
            }
            let [a, b, c, d] = flat.coefficients();
            let along = band.along.map(exactly);
            dot([a, b, c].map(exactly), along) * (band.up + dot(along, band.origin.map(exactly)))
                + exactly(d) * band.length()
        }
        Surface::Cylinder(pipe) => {
            if !band.is_on_the_axis(pipe.axis()) {
                return false;
            }
            band.out - squared(exactly(pipe.radius())) * band.length()
        }
        Surface::Cone(cone) => {
            if !band.is_on_the_axis(cone.axis()) {
                return false;
            }
            let apex = band.up_of(cone.apex().coordinates().map(exactly));
            band.out - squared(exactly(cone.slope())) * squared(band.up - apex)
        }
        Surface::Sphere(ball) => {
            let centre = ball.centre().coordinates().map(exactly);
            if !band.runs_along(difference(centre, band.origin)) {
                return false;
            }
            band.out + squared(band.up - band.up_of(centre))
                - squared(exactly(ball.radius())) * band.length()
        }
        Surface::Torus(_) => return false,
        _ => return false,
    };
    matches!(
        over_the_band.sign(),
        rcad_predicates::Decision::Pos | rcad_predicates::Decision::Neg
    )
}

/// Whether the plane `flat`, leaning to the band's axis, provably passes
/// through no point of `band` (Q1174).
///
/// Scaled by `|w|²`, the plane's equation at a point `p` of the band is
/// `(n·origin + d)·|w|² + (n·w)·up + (n × w)·((p − origin) × w)`, by
/// Lagrange's identity, and the last term is never farther from nought than
/// `√(|n × w|²·out)`. So the plane misses the band where the rest, `near`,
/// keeps one sign over the band and `near² > |n × w|²·out` everywhere in it.
/// Both are worked in interval arithmetic on the numbers the plane stores, as
/// the square case is.
fn a_leaning_plane_misses_a_band(flat: &rcad_geom::Plane, band: ABandRoundAnAxis) -> bool {
    let exactly = rcad_predicates::Interval::exactly;
    let [a, b, c, d] = flat.coefficients();
    let normal = [a, b, c].map(exactly);
    let along = band.along.map(exactly);
    let near = (dot(normal, band.origin.map(exactly)) + exactly(d)) * band.length()
        + dot(normal, along) * band.up;
    if !matches!(
        near.sign(),
        rcad_predicates::Decision::Pos | rcad_predicates::Decision::Neg
    ) {
        return false;
    }
    let [x, y, z] = cross(normal, along);
    let leaning = squared(x) + squared(y) + squared(z);
    matches!(
        (squared(near) - leaning * band.out).sign(),
        rcad_predicates::Decision::Pos
    )
}

impl ABandRoundAnAxis {
    fn up_of(self, point: [rcad_predicates::Interval; 3]) -> rcad_predicates::Interval {
        dot(
            difference(point, self.origin),
            self.along.map(rcad_predicates::Interval::exactly),
        )
    }

    fn out_of(self, point: [rcad_predicates::Interval; 3]) -> rcad_predicates::Interval {
        let [x, y, z] = cross(
            difference(point, self.origin),
            self.along.map(rcad_predicates::Interval::exactly),
        );
        squared(x) + squared(y) + squared(z)
    }

    /// `|w|²`.
    fn length(self) -> rcad_predicates::Interval {
        let along = self.along.map(rcad_predicates::Interval::exactly);
        dot(along, along)
    }

    /// Whether `surface` is a plane whose normal is provably parallel to the
    /// axis, exactly and not to within a rounding.
    fn is_square_to(self, surface: &Surface) -> bool {
        surface.as_plane().is_some_and(|flat| {
            let [a, b, c, _] = flat.coefficients();
            self.runs_along([a, b, c].map(rcad_predicates::Interval::exactly))
        })
    }

    /// Whether `axis` is provably the band's own line: parallel to it, and
    /// through a point of it.
    fn is_on_the_axis(self, axis: rcad_geom::Axis) -> bool {
        let (origin, along) = axis_numbers(axis);
        self.runs_along(along.map(rcad_predicates::Interval::exactly))
            && self.runs_along(difference(
                origin.map(rcad_predicates::Interval::exactly),
                self.origin,
            ))
    }

    /// Whether `direction × w` is exactly nought, which interval arithmetic
    /// says only when it is.
    fn runs_along(self, direction: [rcad_predicates::Interval; 3]) -> bool {
        cross(
            direction,
            self.along.map(rcad_predicates::Interval::exactly),
        )
        .iter()
        .all(|component| matches!(component.sign(), rcad_predicates::Decision::Zero))
    }
}

fn axis_numbers(axis: rcad_geom::Axis) -> ([f64; 3], [f64; 3]) {
    (axis.origin().coordinates(), axis.direction().components())
}

fn difference(
    point: [rcad_predicates::Interval; 3],
    origin: [f64; 3],
) -> [rcad_predicates::Interval; 3] {
    core::array::from_fn(|axis| point[axis] - rcad_predicates::Interval::exactly(origin[axis]))
}

fn dot(
    one: [rcad_predicates::Interval; 3],
    other: [rcad_predicates::Interval; 3],
) -> rcad_predicates::Interval {
    one[0] * other[0] + one[1] * other[1] + one[2] * other[2]
}

fn cross(
    one: [rcad_predicates::Interval; 3],
    other: [rcad_predicates::Interval; 3],
) -> [rcad_predicates::Interval; 3] {
    [
        one[1] * other[2] - one[2] * other[1],
        one[2] * other[0] - one[0] * other[2],
        one[0] * other[1] - one[1] * other[0],
    ]
}

/// The square of a range, which is never below nought even where the range
/// holds nought and the product of the range with itself would say it was.
fn squared(value: rcad_predicates::Interval) -> rcad_predicates::Interval {
    let product = value * value;
    if value.contains_zero() {
        rcad_predicates::Interval::new(0.0, product.hi())
            .unwrap_or(rcad_predicates::Interval::ENTIRE)
    } else {
        product
    }
}

/// The smallest range holding both.
fn hull(
    one: rcad_predicates::Interval,
    other: rcad_predicates::Interval,
) -> rcad_predicates::Interval {
    rcad_predicates::Interval::new(one.lo().min(other.lo()), one.hi().max(other.hi()))
        .unwrap_or(rcad_predicates::Interval::ENTIRE)
}

/// The crossings each of two faces cuts in the other, each as the second surface
/// of the pair — the face's own surface is the first and is not repeated.
///
/// [`intersect`] is the one authority on how two surfaces meet (§24.3), so
/// [`how_two_faces_meet`] asks it rather than working the cases out again.
/// That is also what
/// makes the coincident case one rule for all three surface kinds: two faces on
/// one *cylinder* answer [`SurfaceMeet::SameSurface`] exactly as two on one
/// plane do, and the boundary of each is what cuts the other (§31.3).
///
/// **Which way round the two face is not asked here**, and §31.3 says why: "is
/// it one surface" is geometry and this is where it is settled, while "does it
/// face the same way" is topology and the faces' own orientations settle it, in
/// selection. For a plane the two used to be welded together in
/// `Plane::agreement_with`; a sphere has no reversed form, so they come apart.
fn cuts_between(
    face: &BakedFace,
    against: &BakedFace,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<(Vec<Surface>, Vec<Surface>)> {
    match how_two_faces_meet(&face.surface, &against.surface, policy, log)? {
        HowTwoFacesMeet::OnOneSurface => Ok((boundary_of(against), boundary_of(face))),
        HowTwoFacesMeet::Nowhere => Ok((Vec::new(), Vec::new())),
        HowTwoFacesMeet::AtOnePoint | HowTwoFacesMeet::AlongCurves => {
            Ok((vec![against.surface], vec![face.surface]))
        }
    }
}

/// One cut per edge of a face, for cutting a face that is on the same surface.
fn boundary_of(face: &BakedFace) -> Vec<Surface> {
    face.wires
        .iter()
        .flatten()
        .map(|edge| edge.across)
        .collect()
}

/// Two faces of one solid that run through each other are the solid crossing
/// itself.
///
/// The decomposition already works out where a line meets a face, so this costs
/// nothing new (§17.4). It is deliberately not part of [`Model::validate`]:
/// every face against every other is quadratic in the face count, and §4.3 spent
/// a section refusing to make validation that.
///
/// Faces that share a corner are skipped: they meet along the edges between
/// them, and that is what a solid is. Two coplanar faces of one solid are
/// skipped too — the pair carries no line to compare, so an overlap between them
/// would have to be found some other way, and nothing here does.
///
/// Only a *proved* overlap is an error. Two stretches whose enclosures merely
/// touch have not been shown to run through each other, and refusing them would
/// refuse a solid whose faces meet at a single corner.
///
/// **Every skip above is a pair this has a reason to skip.** A pair whose
/// crossing has no word in [`HowTwoFacesMeet`] is refused there instead, because
/// a check that quietly does not run reports a solid sound that nobody looked
/// at.
///
/// [`Model::validate`]: crate::Model::validate
fn check_it_does_not_cross_itself(
    solid: &BakedSolid,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<()> {
    for (position, face) in solid.faces.iter().enumerate() {
        for against in &solid.faces[position + 1..] {
            if !face.bounds.overlap(against.bounds) || share_a_corner(face, against) {
                continue;
            }
            if how_two_faces_meet(&face.surface, &against.surface, policy, log)?
                != HowTwoFacesMeet::AlongCurves
            {
                continue;
            }
            let mine = stretches_of_a_line_inside(face, &against.surface, policy, log)?;
            let theirs = stretches_of_a_line_inside(against, &face.surface, policy, log)?;
            if mine.provably_overlap(&theirs) {
                return Err(Error::ArgumentIntersectsItself);
            }
        }
    }
    Ok(())
}

fn share_a_corner(face: &BakedFace, against: &BakedFace) -> bool {
    face.wires.iter().flatten().any(|edge| {
        against
            .wires
            .iter()
            .flatten()
            .any(|theirs| theirs.start.vertex == edge.start.vertex)
    })
}

/// What two faces' surfaces do to each other, in the three words the cutting
/// here is written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HowTwoFacesMeet {
    /// The two are one surface, so the boundary of each is what cuts the other.
    OnOneSurface,
    /// They miss each other altogether.
    Nowhere,
    /// They touch at one point and cross nowhere.
    ///
    /// A pair that touches at one point draws no curve on either face (§31.3).
    /// Each is still handed the other's surface as a cut, and the arrangement
    /// puts the point on a curve of the face that runs through it, because a
    /// face next door can cut the other surface in curves through that point
    /// and split the edge between them there (Q1337). Where no curve of the
    /// face runs through the point nothing is split: two tangent balls united
    /// come back as two shells that share no vertex.
    AtOnePoint,
    /// They cross along one curve, or two.
    AlongCurves,
}

/// Read a crossing in those three words, and refuse a shape that has no word
/// here.
///
/// The one place in this file that looks at a [`SurfaceMeet`]. There used to be
/// two, and each drew its own wrong conclusion from a shape it did not know: one
/// crashed the fuse, and the other — the self-crossing check — **skipped the
/// pair in silence**, so a solid nobody had looked at came back sound.
///
/// A sixth shape two surfaces can meet in is `rcad-geom` growing a word, not a
/// broken invariant here and not bad input: the two solids are as sound as they
/// were. So it refuses by name, and the name is the one its twin in
/// [`arrange`](super::arrange) already uses for the same arrival — a cut this
/// hangs on a face is ordered along there, and a shape with no word here has no
/// order there either.
///
/// [`SurfaceMeet`] is `#[non_exhaustive]` and belongs to another crate, so a
/// sixth shape arriving here is not a compile error. `a_fourth_kind_of_run.rs`
/// is what says so out loud.
fn how_two_faces_meet(
    face: &Surface,
    against: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<HowTwoFacesMeet> {
    match intersect(face, against, policy, log)? {
        SurfaceMeet::SameSurface => Ok(HowTwoFacesMeet::OnOneSurface),
        SurfaceMeet::Nothing => Ok(HowTwoFacesMeet::Nowhere),
        SurfaceMeet::TouchAtOnePoint { .. } => Ok(HowTwoFacesMeet::AtOnePoint),
        SurfaceMeet::Curves(_) => Ok(HowTwoFacesMeet::AlongCurves),
        _ => Err(Error::ACrossingThisCannotBeOrderedAlong),
    }
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Isometry, Plane, Point, Scale, Surface, Vector};
    use rcad_predicates::{Policy, ProvenanceLog};

    use super::{a_face_misses_a_surface_by_its_band, gather, no_face_of_one_meets_the_other};
    use crate::fuse::bake::{BakedFace, BakedSolid, bake};
    use crate::{Error, Model, SolidKey};

    /// Two cubes of side two, the second one moved by `shift`.
    fn two_cubes(shift: Vector) -> Vec<BakedSolid> {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let one = model.add_cube(2.0, &policy, &mut log).expect("a cube");
        let other = model
            .add_cube(2.0, &policy, &mut log)
            .expect("another cube");
        model
            .place_solid(
                other,
                &Isometry::translation(shift).expect("a finite shift"),
            )
            .expect("this model made the key");
        bake(&model, &[one, other], &policy, &mut ProvenanceLog::new()).expect("two cubes bake")
    }

    fn facing(solid: &BakedSolid, way: Vector) -> &BakedFace {
        solid
            .faces
            .iter()
            .find(|face| flat(face).normal() == way)
            .expect("a cube has a face pointing every way")
    }

    /// The plane of a baked face. Every face in this module's tests is flat.
    fn flat(face: &BakedFace) -> Plane {
        *face.surface.as_plane().expect("a cube is a polyhedron")
    }

    fn gathered(solids: &mut [BakedSolid]) {
        let mut log = ProvenanceLog::new();
        gather(solids, &Policy::certified(), &mut log)
            .expect("two lattice-aligned cubes leave nothing undecided");
        assert!(log.is_empty(), "nothing here had to be approximated");
    }

    #[test]
    fn two_cubes_that_overlap_cut_each_other_where_their_faces_cross() {
        let mut solids = two_cubes(Vector::new(1.0, 1.0, 1.0));

        gathered(&mut solids);

        let up = facing(&solids[0], Vector::new(0.0, 0.0, 1.0));
        assert_eq!(
            distinct_cuts(up),
            [
                [-1.0, 0.0, 0.0, -0.0],
                [-1.0, 0.0, 0.0, 1.0],
                [0.0, -1.0, 0.0, -0.0],
                [0.0, -1.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, -3.0],
                [0.0, 1.0, 0.0, -2.0],
                [1.0, 0.0, 0.0, -3.0],
                [1.0, 0.0, 0.0, -2.0]
            ],
            "all four walls of the upper cube, the two far ones included, and its own four \
             beside them: a cut is by a plane, and a plane reaches past the face carrying it"
        );
    }

    /// The bottom of the lower cube is nowhere near the upper one, and is cut by
    /// its walls all the same — because its own neighbours are, and a face split
    /// on one side of a shared edge and not the other leaves an edge with one
    /// face along it.
    #[test]
    fn a_face_the_other_cube_cannot_reach_is_cut_all_the_same() {
        let mut solids = two_cubes(Vector::new(1.0, 1.0, 1.0));

        gathered(&mut solids);

        let down = facing(&solids[0], Vector::new(0.0, 0.0, -1.0));
        assert_eq!(
            distinct_cuts(down).len(),
            8,
            "the four walls of the upper cube, none of which reaches down here, and the four \
             of its own cube"
        );
    }

    /// Two coplanar faces cut each other with the four edges of the other, not
    /// with a plane that is the same plane.
    ///
    /// The upper cube's four walls also reach down to the face they share and
    /// cut it directly, and its own four lie on those very same planes — so
    /// twelve cut lines arrive and four distinct planes carry them. Without the
    /// coplanar branch only eight would.
    #[test]
    fn one_cube_sitting_on_another_is_cut_once_per_edge_of_the_face_they_share() {
        let mut solids = two_cubes(Vector::new(0.0, 0.0, 2.0));

        gathered(&mut solids);

        for (solid, way) in [(0, 1.0), (1, -1.0)] {
            let shared = facing(&solids[solid], Vector::new(0.0, 0.0, way));
            assert_eq!(shared.cuts.len(), 12);
            assert_eq!(distinct_cuts(shared).len(), 4);
        }
    }

    /// Two cubes ten apart leave each other's faces alone. Each is still cut by
    /// its own four walls, which is what the header asks for and says nothing
    /// about the other cube.
    #[test]
    fn two_cubes_that_are_nowhere_near_each_other_cut_nothing_of_each_other() {
        let mut solids = two_cubes(Vector::new(10.0, 0.0, 0.0));

        gathered(&mut solids);

        for solid in &solids {
            let mine = walls_of(solid);
            for face in &solid.faces {
                for cut in distinct_cuts(face) {
                    assert!(
                        mine.contains(&cut),
                        "{cut:?} cuts a face of a cube that does not carry it, so it came \
                         from ten away"
                    );
                }
            }
        }
    }

    #[test]
    fn a_cube_does_not_run_through_itself() {
        let mut solids = two_cubes(Vector::new(1.0, 1.0, 1.0));
        gathered(&mut solids);
    }

    /// A ball of radius 5 at the origin and a bar of radius 3 standing from
    /// `−half` to `half` up its middle.
    fn a_ball_round_a_bar(half: f64) -> Vec<BakedSolid> {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let ball = model
            .add_ball(5.0, &Isometry::identity())
            .expect("a ball of radius five");
        let bar = model
            .add_round_bar(
                3.0,
                2.0 * half,
                &Isometry::translation(Vector::new(0.0, 0.0, -half)).expect("a finite shift"),
            )
            .expect("a bar of radius three");
        let policy = Policy::certified();
        bake(&model, &[ball, bar], &policy, &mut ProvenanceLog::new()).expect("both bake")
    }

    /// Nothing but the band round the bar's axis proves this: every face of
    /// the bar is on a pipe or bounded by one, so none has a box here, and the
    /// ball's skin meets every surface of the bar.
    #[test]
    fn a_bar_with_its_rims_inside_a_ball_is_proved_apart_from_it() {
        let solids = a_ball_round_a_bar(2.0);
        assert!(no_face_of_one_meets_the_other(&solids[0], &solids[1]));
    }

    /// Rims at `3² + 4² = 5²` are on the ball, exactly, so the two touch along
    /// both rims and must be cut against each other.
    #[test]
    fn a_bar_with_its_rims_on_the_ball_is_not_proved_apart_from_it() {
        let solids = a_ball_round_a_bar(4.0);
        assert!(!no_face_of_one_meets_the_other(&solids[0], &solids[1]));
    }

    #[test]
    fn a_bar_poking_out_of_a_ball_is_not_proved_apart_from_it() {
        let solids = a_ball_round_a_bar(4.5);
        assert!(!no_face_of_one_meets_the_other(&solids[0], &solids[1]));
    }

    /// The plane `4z = x + 12` leans to the bar's axis and is at `z = 2.25` or
    /// higher over the bar's wall, which ends at `z = 2` (Q1174).
    #[test]
    fn a_leaning_plane_clear_of_a_bars_rim_is_proved_to_miss_its_wall() {
        let solids = a_ball_round_a_bar(2.0);
        let clear = leaning_plane_through(Point::new(0.0, 0.0, 3.0));
        assert!(a_face_misses_a_surface_by_its_band(
            the_wall(&solids[1]),
            &clear
        ));
    }

    /// `4z = x + 11` is at `z = 2` over `(−3, 0)`, so it shares that one point
    /// of the rim with the wall; `4z = x` runs across the wall.
    #[test]
    fn a_leaning_plane_through_a_bars_rim_or_across_its_wall_is_not_proved_to_miss_it() {
        let solids = a_ball_round_a_bar(2.0);
        for through in [Point::new(-3.0, 0.0, 2.0), Point::new(0.0, 0.0, 0.0)] {
            let plane = leaning_plane_through(through);
            assert!(
                !a_face_misses_a_surface_by_its_band(the_wall(&solids[1]), &plane),
                "{plane:?}"
            );
        }
    }

    fn leaning_plane_through(point: Point) -> Surface {
        Surface::Plane(
            Plane::from_point_and_normal(point, Vector::new(-1.0, 0.0, 4.0))
                .expect("a normal with a length"),
        )
    }

    fn the_wall(bar: &BakedSolid) -> &BakedFace {
        bar.faces
            .iter()
            .find(|face| matches!(face.surface, Surface::Cylinder(_)))
            .expect("a bar has a wall")
    }

    /// Two cubes wired into one solid, overlapping. Nothing a profile can build
    /// crosses itself — `ProfileBuilder` refuses it — so the only way to get one
    /// is to put it together by hand.
    #[test]
    fn a_solid_made_of_two_cubes_that_overlap_is_refused() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let one = model.add_cube(2.0, &policy, &mut log).expect("a cube");
        let other = model
            .add_cube(2.0, &policy, &mut log)
            .expect("another cube");
        model
            .place_solid(
                other,
                &Isometry::translation(Vector::new(1.0, 1.0, 1.0)).expect("a finite shift"),
            )
            .expect("this model made the key");
        let crossed = both_solids_as_one(&mut model, one, other);

        let mut solids =
            bake(&model, &[crossed], &policy, &mut ProvenanceLog::new()).expect("it bakes");
        let refusal = gather(&mut solids, &policy, &mut log);

        assert!(
            matches!(refusal, Err(Error::ArgumentIntersectsItself)),
            "expected a refusal, got {refusal:?}"
        );
    }

    /// Two solids handed over on the *same* side still cut each other. `A` is
    /// the union of the arguments, and a union of two overlapping cubes that
    /// were never cut against each other keeps the halves buried inside the
    /// other one.
    #[test]
    fn two_cubes_in_one_group_cut_each_other_the_same_as_two_in_opposite_groups() {
        let mut solids = two_cubes(Vector::new(1.0, 1.0, 1.0));

        gathered(&mut solids);

        let up = facing(&solids[0], Vector::new(0.0, 0.0, 1.0));
        let cutting = distinct_cuts(up);
        for wall in walls_of(&solids[1]) {
            if wall[2] != 0.0 {
                continue;
            }
            assert!(
                cutting.contains(&wall),
                "the top of the lower cube is cut by {wall:?}, a wall of the upper one"
            );
        }
    }

    /// The planes cutting one face, sorted and with the repeats dropped, the way
    /// [`canonical_cut_surfaces`](super::super::arrange::name_of_surface_bits)
    /// takes them.
    fn distinct_cuts(face: &BakedFace) -> Vec<[f64; 4]> {
        let mut distinct: Vec<[f64; 4]> = face.cuts.iter().map(coefficients_of).collect();
        distinct.sort_by(|one, other| one.partial_cmp(other).expect("no nans here"));
        distinct.dedup();
        distinct
    }

    /// The plane of every face of one cube.
    fn walls_of(solid: &BakedSolid) -> Vec<[f64; 4]> {
        solid
            .faces
            .iter()
            .map(|face| coefficients_of(&face.surface))
            .collect()
    }

    fn coefficients_of(surface: &Surface) -> [f64; 4] {
        surface
            .as_plane()
            .expect("a cube is a polyhedron")
            .coefficients()
    }

    fn both_solids_as_one(model: &mut Model, one: SolidKey, other: SolidKey) -> SolidKey {
        let mut shells = model.data().solids[one.slot_in(model.id())].shells.clone();
        let theirs = model.data().solids[other.slot_in(model.id())]
            .shells
            .clone();
        shells.extend(theirs);
        let slot = model.add_solid(shells);
        model.key_for_solid(slot)
    }
}
