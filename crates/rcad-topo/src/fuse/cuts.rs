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
//! **A hollow frustum under that top, cones for pipes, needed two things
//! more** (Q1461). The band round a cone face reaches down to the apex, and
//! as a box in how far up and how far out it held the apex as far out as the
//! wide end, so a leaning plane is now asked at the band's two ends and at the
//! apex, where the cone says how far out the face is. And two cones on one
//! axis meet each other in circles, so where the top met both could not be
//! worked out and the cones were taken to meet on it. Where three surfaces
//! cannot be placed, the crossing of a plane and a cone is now asked whether
//! the band round the cone's axis that it lies in misses the third surface.
//! **The crossing of a plane and a pipe is asked the same way** (Q1583): a
//! pipe boring that frustum on its axis meets each cone in circles too, and
//! with the frustum's apex over the top, where its wall's band does not prove
//! the top clear, the ellipses where the top crosses the pipe and the cone
//! were both kept, and where the three meet could not be worked out.
//!
//! **A torus about the band's axis is asked too** (Q1588). A bar standing in
//! a ring's hole, its top level with the top of the tube, cut the ring's skin
//! along the circle where the top's plane touches it, though the top reaches
//! nowhere near that circle. The piece of skin past the circle, on the side of
//! the plane where the skin has no point, had no point inside to place, and
//! every fuse of the two was refused.
//!
//! **A cut whose faces meet the face only on its boundary is marked too**
//! (Q1470, `docs/decisions.md` §422). Where every face carrying a surface
//! lies on one closed side of a plane and the face it cuts lies on the other,
//! or the box round the face it cuts is strictly past that plane, the two
//! share no point inside that face ([`meets_it_only_on_its_boundary`]). Inside
//! it the crossing is then no edge of the answer, even where it meets other
//! crossings. A bar butted onto a bend's cap is the shape: its pipe meets the
//! ring in a loop whose halves lie past each other's cap. Such a crossing can
//! still split an edge where it meets one, so it is not dropped here: each face
//! carries the list ([`BakedFace::cuts_that_reach_only_its_boundary`]), each
//! edge learns its neighbour's, and the arrangement leaves the crossing out
//! only where no node of it splits a crossing that is kept.
//!
//! **Each face is told, too, the sides of planes every face carrying a cut
//! lies in** (Q1909, [`BakedFace::sides_the_faces_of_each_cut_lie_in`]). A
//! face's box can reach across such a plane where a crossing on it does not:
//! a pin slanting up out of a bar's axis has the pipe of its wall cross the
//! bar's end in an ellipse on the far side of the axis, wholly behind the
//! pin's own cap. No face of the pin reaches that ellipse, and that is what
//! lets the arrangement leave it out, whether or not it can place where the
//! ellipse meets the bar's rim (Q2201).
//!
//! # Each face is told the caps through the start of their axis that cut it
//!
//! `bake` reads off a round bar's numbers before its move that its near cap
//! holds the start of its axis, and hangs that on the bar's own near cap and
//! wall ([`BakedFace::caps_through_the_start_of_their_axis`]), since the moved
//! cap misses the moved start by a rounding. The face that needs it is the
//! other bar's (Q1820): a bar starting where its axis crosses another's of its
//! width cuts that other's wall with its near cap and its pipe, and the
//! arrangement names where those three meet only from the fact. So each face
//! is told, once, every such cap that a face cutting it carries. A fact told
//! is the cap's and pipe's own and true wherever it is read, and the
//! arrangement uses it only where the cap crosses the face and the pipe is
//! the face's own surface or crosses it too.
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
    EdgeRun, MeetOfThree, NamedRun, PlaneAgreement, Surface, SurfaceMeet, intersect,
    meet_of_three_surfaces,
};
use rcad_predicates::{Policy, ProvenanceLog};
use smallvec::SmallVec;

use crate::{EdgeKey, Error, Result};

use super::Sameness;
use super::arrange::stretches_of_a_line_inside;
use super::bake::{ACapThroughTheStartOfItsAxis, ASideOfAPlane, BakedFace, BakedSolid};

/// Work out every cut line, and hang each on the face it cuts.
///
/// Every face is cut against every other face, in its own solid and in every
/// other, whichever group each came from. The header says why the solid's own
/// planes are in there, why a crossing that splits nothing and lies where
/// its face never reaches is then left off, why each face is told which
/// cuts meet it only on its boundary, and each edge what the face across it
/// was told, why each face is told the sides of planes the faces of each cut
/// lie in, and why each face is told the caps through the start of their
/// axis that the faces cutting it carry.
///
/// **No face pair is skipped, but [`how_two_faces_meet`] is only ever asked
/// once per distinct pair of surfaces** ([`HowFacesMeetBySurface`]). The
/// header's "every pair" is about faces, not about calls: a connector's whole
/// row of pins holds the same cap height and the same two walls along the
/// row, so the same two surfaces arrive on this loop's doorstep from many
/// different pairs of faces, and the pair, not the geometry, is what used to
/// be new every time.
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
    let mut told: Vec<Vec<Vec<ACapThroughTheStartOfItsAxis>>> = solids
        .iter()
        .map(|solid| vec![Vec::new(); solid.faces.len()])
        .collect();
    let mut how_they_meet = HowFacesMeetBySurface::new();
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
                    let meet = how_they_meet.ask(&face.surface, &against.surface, policy, log)?;
                    let (onto_face, onto_against) = cuts_from_how_they_meet(meet, face, against);
                    if !onto_face.is_empty() {
                        told[one][mine].extend(&against.caps_through_the_start_of_their_axis);
                    }
                    if !onto_against.is_empty() {
                        told[other][theirs].extend(&face.caps_through_the_start_of_their_axis);
                    }
                    found[one][mine].extend(found_on(face, against, onto_face));
                    found[other][theirs].extend(found_on(against, face, onto_against));
                }
            }
        }
    }

    for (solid, (faces, told)) in found.into_iter().zip(told).enumerate() {
        for (face, (cuts, told)) in faces.into_iter().zip(told).enumerate() {
            let face = &mut solids[solid].faces[face];
            let kept = the_cuts_left_on(face, &cuts);
            face.cuts.extend(kept);
            face.cuts_that_reach_only_its_boundary = the_cuts_that_reach_only_its_boundary(&cuts);
            face.sides_the_faces_of_each_cut_lie_in = the_sides_the_faces_of_each_cut_lie_in(&cuts);
            for cap in told {
                if !face.caps_through_the_start_of_their_axis.contains(&cap) {
                    face.caps_through_the_start_of_their_axis.push(cap);
                }
            }
        }
    }
    for solid in solids.iter_mut() {
        tell_each_edge_what_its_neighbour_leaves_out(solid);
    }
    Ok(())
}

#[cfg(test)]
static SURFACE_PAIRS_REALLY_ASKED: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// Caches how two surfaces meet, keyed by the surfaces' own stored numbers,
/// since the answer depends on nothing else — [`how_two_faces_meet`] takes
/// two [`Surface`]s, not two faces — and a connector's whole row of pins
/// repeats the same cap plane and the same two walls along the row, on every
/// pin.
struct HowFacesMeetBySurface {
    seen: HashMap<[u64; 9], usize>,
    answers: HashMap<(usize, usize), HowTwoFacesMeet>,
}

impl HowFacesMeetBySurface {
    fn new() -> Self {
        Self {
            seen: HashMap::new(),
            answers: HashMap::new(),
        }
    }

    fn id_of(&mut self, surface: &Surface) -> usize {
        let bits = super::arrange::name_of_surface_bits(surface);
        let next = self.seen.len();
        *self.seen.entry(bits).or_insert(next)
    }

    /// Both orders land on the same key, because [`how_two_faces_meet`] gives
    /// the same answer whichever surface is handed first: `intersect`'s own
    /// test helper `crossing` (`rcad_geom::intersect`'s tests) checks exactly
    /// that for every pair it covers, and
    /// [`a_line_they_rest_on`]'s `matches!` lists both orders of every mixed
    /// pair it names, so the one place here that reads past the bare
    /// [`SurfaceMeet`] stays symmetric too.
    fn ask(
        &mut self,
        face: &Surface,
        against: &Surface,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<HowTwoFacesMeet> {
        let (a, b) = (self.id_of(face), self.id_of(against));
        let key = if a <= b { (a, b) } else { (b, a) };
        if let Some(&answer) = self.answers.get(&key) {
            return Ok(answer);
        }
        #[cfg(test)]
        SURFACE_PAIRS_REALLY_ASKED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let answer = how_two_faces_meet(face, against, policy, log)?;
        self.answers.insert(key, answer);
        Ok(answer)
    }
}

/// The crossings each of two faces cuts in the other, each as the second surface
/// of the pair — the face's own surface is the first and is not repeated.
///
/// [`intersect`] is the one authority on how two surfaces meet (§24.3), so
/// [`how_two_faces_meet`] asks it rather than working the cases out again.
/// That is also what makes the coincident case one rule for all three surface
/// kinds: two faces on one *cylinder* answer [`SurfaceMeet::SameSurface`]
/// exactly as two on one plane do, and the boundary of each is what cuts the
/// other (§31.3).
///
/// **Which way round the two faces came in is not asked here**, and §31.3
/// says why: "is it one surface" is geometry and this is where it is settled,
/// while "does it face the same way" is topology and the faces' own
/// orientations settle it, in selection. For a plane the two used to be welded
/// together in `Plane::agreement_with`; a sphere has no reversed form, so they
/// come apart.
fn cuts_from_how_they_meet(
    meet: HowTwoFacesMeet,
    face: &BakedFace,
    against: &BakedFace,
) -> (Vec<Surface>, Vec<Surface>) {
    match meet {
        HowTwoFacesMeet::OnOneSurface => (boundary_of(against), boundary_of(face)),
        HowTwoFacesMeet::Nowhere => (Vec::new(), Vec::new()),
        HowTwoFacesMeet::AtOnePoint
        | HowTwoFacesMeet::AlongALineTheyRestOn
        | HowTwoFacesMeet::AlongCurves => (vec![against.surface], vec![face.surface]),
    }
}

/// A cut found for a face, before it is hung on it.
#[derive(Clone, Debug)]
struct ACutFound {
    cut: Surface,
    /// The sides of planes the face that carries `cut` lies in
    /// ([`BakedFace::lies_in`]) when `cut` is that face's own surface, and
    /// none when it is a surface across that face's boundary.
    its_face_lies_in: SmallVec<[ASideOfAPlane; 2]>,
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
            its_face_lies_in: if cut == against.surface {
                against.lies_in.clone()
            } else {
                SmallVec::new()
            },
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
        on_the_other_side
            || anywhere_on_the_face
                .is_some_and(|anywhere| theirs.plane.side_of_point(anywhere) == theirs.past_it())
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

/// For each cut, the sides of planes that every face carrying it lies in, by
/// the first finding's sides that every other finding of that cut holds as
/// well; a cut left with none is not listed.
fn the_sides_the_faces_of_each_cut_lie_in(
    found: &[ACutFound],
) -> Vec<(Surface, SmallVec<[ASideOfAPlane; 2]>)> {
    let mut sides: Vec<(Surface, SmallVec<[ASideOfAPlane; 2]>)> = Vec::new();
    for one in found {
        if sides.iter().any(|(cut, _)| *cut == one.cut) {
            continue;
        }
        let every_face_lies_in: SmallVec<[ASideOfAPlane; 2]> = one
            .its_face_lies_in
            .iter()
            .filter(|side| {
                found
                    .iter()
                    .filter(|other| other.cut == one.cut)
                    .all(|other| {
                        other
                            .its_face_lies_in
                            .iter()
                            .any(|theirs| one_closed_side_of_one_plane(side, theirs))
                    })
            })
            .copied()
            .collect();
        if !every_face_lies_in.is_empty() {
            sides.push((one.cut, every_face_lies_in));
        }
    }
    sides
}

/// Whether the two are the same closed side of the same plane, however each
/// plane is written.
fn one_closed_side_of_one_plane(one: &ASideOfAPlane, other: &ASideOfAPlane) -> bool {
    match one.plane.agreement_with(&other.plane) {
        PlaneAgreement::SameWay => one.behind == other.behind,
        PlaneAgreement::OppositeWay => one.behind != other.behind,
        PlaneAgreement::Different | PlaneAgreement::CannotTell | _ => false,
    }
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
///
/// **Where the crossing is a plane's with a cone or a pipe, the band round
/// the axis that it lies in is asked as well** (Q1461, Q1583), when the three
/// cannot be worked out and the partners do meet. Two cones on one axis meet
/// in circles, and so do a cone and a pipe, and with a plane where they meet
/// needs two of the three flat. A hollow frustum under a leaning top kept the
/// ellipse where the top crosses each cone's other nappe, near neither circle,
/// and was refused for it; bored on its axis by a pipe, with its apex over the
/// top, it kept the ellipse where the top crosses the pipe as well.
fn the_crossing_meets_none_of(surface: &Surface, cut: &Surface, others: &[Surface]) -> bool {
    let policy = Policy::certified();
    let crossing = the_band_a_plane_crosses_a_cone_in(surface, cut)
        .or_else(|| the_band_a_plane_crosses_a_pipe_in(surface, cut));
    others.iter().filter(|other| *other != cut).all(|other| {
        let mut dropped = ProvenanceLog::new();
        match meet_of_three_surfaces([surface, cut, other], &policy, &mut dropped) {
            Ok(MeetOfThree::Nowhere) => true,
            Ok(MeetOfThree::At(_) | MeetOfThree::MoreThanAPoint) => false,
            Ok(_) => false,
            Err(_) => {
                matches!(
                    intersect(cut, other, &policy, &mut dropped),
                    Ok(SurfaceMeet::Nothing)
                ) || crossing.is_some_and(|band| a_surface_misses_a_band(other, band))
            }
        }
    })
}

/// The band round a cone's axis that the whole crossing of a plane and that
/// cone lies in, the two either way round, or `None` where this cannot bound
/// it.
///
/// Read round the cone's axis the way [`a_leaning_plane_misses_a_band`] reads
/// a plane, every point of the plane has `near(up)² ≤ |n × w|²·out`, and every
/// point of the cone has `out = slope²·up²`. So every point of the crossing
/// has `q(up) ≤ 0`, where `q(up) = near(up)² − |n × w|²·slope²·up²`. Where the
/// plane is nearer square to the axis than the cone's wall is,
/// `(n·w)² > |n × w|²·slope²`, `q` is a bowl, at or below nought only between
/// its two roots, and the crossing is an ellipse or the apex alone. A plane
/// leaning more than that crosses the cone in a curve that runs off for ever,
/// and no band holds it. The two roots bound the band, as
/// [`where_a_bowl_dips_to_nought`] proves them.
fn the_band_a_plane_crosses_a_cone_in(one: &Surface, other: &Surface) -> Option<ABandRoundAnAxis> {
    let ((Surface::Plane(flat), Surface::Cone(cone)) | (Surface::Cone(cone), Surface::Plane(flat))) =
        (one, other)
    else {
        return None;
    };
    let exactly = rcad_predicates::Interval::exactly;
    let is =
        |value: rcad_predicates::Interval, sign: rcad_predicates::Decision| value.sign() == sign;
    let (origin, along) = axis_numbers(cone.axis());
    let plane = APlaneRoundAnAxis::of(flat, origin, along);
    let spread = plane.leaning * squared(exactly(cone.slope()));
    if !is(
        squared(plane.rising) - spread,
        rcad_predicates::Decision::Pos,
    ) {
        return None;
    }
    let q = |up| squared(plane.near(up)) - spread * squared(up);
    let q_rising = |up| plane.rising * plane.near(up) - spread * up;

    let (at_the_origin, rising, spread_f64) = (
        middle(plane.at_the_origin),
        middle(plane.rising),
        middle(spread),
    );
    let bowl = rising * rising - spread_f64;
    let centre = -at_the_origin * rising / bowl;
    let half = (at_the_origin * at_the_origin * spread_f64).max(0.0).sqrt() / bowl;
    let up = where_a_bowl_dips_to_nought(centre, half, q, q_rising)?;
    Some(ABandRoundAnAxis {
        origin,
        along,
        up,
        out: squared(exactly(cone.slope())) * squared(up),
        slope: Some(cone.slope()),
    })
}

/// The band round a pipe's axis that the whole crossing of a plane and that
/// pipe lies in, the two either way round, or `None` where this cannot bound
/// it (Q1583).
///
/// Read round the pipe's axis the way [`a_leaning_plane_misses_a_band`] reads
/// a plane, every point of the plane has `near(up)² ≤ |n × w|²·out`, and every
/// point of a pipe of radius `r` has `out = r²·|w|²`. So every point of the
/// crossing has `q(up) ≤ 0`, where `q(up) = near(up)² − |n × w|²·r²·|w|²`.
/// Where `n·w` is not nought, `q` is a bowl, at or below nought only between
/// its two roots, and the crossing is an ellipse between them. The roots
/// bound the band, as [`where_a_bowl_dips_to_nought`] proves them.
///
/// **Where `n·w` is nought, or is not proved not to be, there is no band.**
/// The plane then runs along the axis, and crosses the pipe in two lines
/// along it, touches it along one, or misses it. A line along the axis has no
/// end, so no band holds it, and the crossing is asked as it was before: kept
/// unless the three surfaces are placed or the partners do not meet.
fn the_band_a_plane_crosses_a_pipe_in(one: &Surface, other: &Surface) -> Option<ABandRoundAnAxis> {
    let ((Surface::Plane(flat), Surface::Cylinder(pipe))
    | (Surface::Cylinder(pipe), Surface::Plane(flat))) = (one, other)
    else {
        return None;
    };
    let exactly = rcad_predicates::Interval::exactly;
    let (origin, along) = axis_numbers(pipe.axis());
    let plane = APlaneRoundAnAxis::of(flat, origin, along);
    if !matches!(squared(plane.rising).sign(), rcad_predicates::Decision::Pos) {
        return None;
    }
    let length = dot(along.map(exactly), along.map(exactly));
    let out = squared(exactly(pipe.radius())) * length;
    let reach = plane.leaning * out;
    let q = |up| squared(plane.near(up)) - reach;
    let q_rising = |up| plane.rising * plane.near(up);

    let (at_the_origin, rising) = (middle(plane.at_the_origin), middle(plane.rising));
    let centre = -at_the_origin / rising;
    let half = middle(reach).max(0.0).sqrt() / rising.abs();
    let up = where_a_bowl_dips_to_nought(centre, half, q, q_rising)?;
    Some(ABandRoundAnAxis {
        origin,
        along,
        up,
        out,
        slope: None,
    })
}

/// The range of `up` outside which the bowl `q` is proved above nought, given
/// its two roots as `centre ± half` in plain `f64`, or `None` where that is
/// not proved. `q_rising` has the sign of `q`'s slope, and the caller has
/// proved that `q` is a bowl.
///
/// Interval arithmetic has no square root, so the roots are found in plain
/// `f64` and each moved out by a millionth of how far out they are. What is
/// proved, in interval arithmetic on the numbers the two surfaces store, is
/// that `q` is above nought at each, falling at the lower and rising at the
/// upper. In a bowl that leaves no point at or below nought outside them, so
/// no point of the crossing either, wherever the `f64` roots landed.
fn where_a_bowl_dips_to_nought(
    centre: f64,
    half: f64,
    q: impl Fn(rcad_predicates::Interval) -> rcad_predicates::Interval,
    q_rising: impl Fn(rcad_predicates::Interval) -> rcad_predicates::Interval,
) -> Option<rcad_predicates::Interval> {
    let exactly = rcad_predicates::Interval::exactly;
    let is =
        |value: rcad_predicates::Interval, sign: rcad_predicates::Decision| value.sign() == sign;
    let margin = (centre.abs() + half) * 1e-6;
    let (low, high) = (centre - half - margin, centre + half + margin);
    let (low_up, high_up) = (exactly(low), exactly(high));
    if !(is(q(low_up), rcad_predicates::Decision::Pos)
        && is(q_rising(low_up), rcad_predicates::Decision::Neg)
        && is(q(high_up), rcad_predicates::Decision::Pos)
        && is(q_rising(high_up), rcad_predicates::Decision::Pos))
    {
        return None;
    }
    rcad_predicates::Interval::new(low, high).ok()
}

/// The middle of a range, in plain `f64`.
fn middle(value: rcad_predicates::Interval) -> f64 {
    (value.lo() + value.hi()) / 2.0
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
/// an axis that the other face lies in ([`band_a_face_is_inside`]); one is a
/// ball holding the other, a flat face, strictly inside it
/// ([`a_ball_holds_a_flat_face`]); the surfaces do not meet at all; or they
/// touch at one point outside that part of space.
///
/// **The corners are what settle a prism inside a ball.** A 12-sided prism of
/// radius 0.81 and height 1 inside a ball of radius 1: the box round its top
/// reaches `(0.81, 0.81, 0.5)`, 1.25 out, so the ball's skin runs through that
/// box although every corner of the top is 0.95 out (Q2247).
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
        || a_ball_holds_a_flat_face(&against.surface, face)
        || a_ball_holds_a_flat_face(&face.surface, against)
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

/// Whether `surface` is a ball holding every point of `face`, a flat face whose
/// every side is straight, strictly inside it, so that the ball's skin shares
/// no point with the face.
///
/// Such a face lies in the hull of its corners, a straight side lying between
/// its two, and a ball holds the hull of any points it holds. So each corner's
/// own box lying strictly inside the ball is enough, where the one box round all
/// the corners together can reach out of the ball at a corner of that box which
/// no corner of the face is near. A side on anything but a plane may bulge past
/// its corners, and a face with such a side is not asked about.
fn a_ball_holds_a_flat_face(surface: &Surface, face: &BakedFace) -> bool {
    let Surface::Sphere(ball) = surface else {
        return false;
    };
    if face.surface.as_plane().is_none() {
        return false;
    }
    let mut sides = face.wires.iter().flatten().peekable();
    sides.peek().is_some()
        && sides.all(|side| {
            side.across.as_plane().is_some()
                && [&side.start, &side.end].into_iter().all(|corner| {
                    let [_, furthest] = a_box_against_a_balls_skin(ball, corner.point.enclosure());
                    furthest.sign() == rcad_predicates::Decision::Neg
                })
        })
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
    let [nearest, furthest] = a_box_against_a_balls_skin(ball, room);
    matches!(nearest.sign(), rcad_predicates::Decision::Pos)
        || matches!(furthest.sign(), rcad_predicates::Decision::Neg)
}

/// The squared distance from `ball`'s centre to the nearest and to the furthest
/// point of `room`, each less the ball's squared radius: the nearest above
/// nought puts the box wholly outside the ball, the furthest below nought puts
/// it wholly inside.
fn a_box_against_a_balls_skin(
    ball: &rcad_geom::Sphere,
    room: [rcad_predicates::Interval; 3],
) -> [rcad_predicates::Interval; 2] {
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
    [nearest - skin, furthest - skin]
}

/// Where a face lies round one axis: how far up it and how far out from it.
/// The crossing of a plane and a cone or a pipe is banded the same way
/// ([`the_band_a_plane_crosses_a_cone_in`],
/// [`the_band_a_plane_crosses_a_pipe_in`]), and what is said of the face here
/// is said of the crossing there.
///
/// Both are measured the way the stored surfaces measure them, with the axis's
/// direction `w` as stored and not scaled to length one: `up` holds
/// `(p − origin)·w` and `out` holds `|(p − origin) × w|²` for every point `p`
/// of the face. A pipe of radius `r` about the line is then every point with
/// `out = r²·|w|²`, and a cone every point with `out = slope²·(up − its apex's
/// up)²`, whatever the length of `w` — the expressions the two side-of
/// predicates in `rcad-geom` take the sign of.
///
/// `out` is a range of its own, so the band is a box in `up` and `out`. Where
/// every point of the face is on one cone with its apex at `origin`, `slope`
/// says which, and `out` is then `slope²·up²` at every point of the face and
/// not merely somewhere in its range (Q1461).
#[derive(Clone, Copy, Debug)]
struct ABandRoundAnAxis {
    origin: [f64; 3],
    along: [f64; 3],
    up: rcad_predicates::Interval,
    out: rcad_predicates::Interval,
    slope: Option<f64>,
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
        slope: None,
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
            band.slope = Some(cone.slope());
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
/// [`a_leaning_plane_misses_a_band`], and a torus about the line by
/// [`a_torus_on_the_axis_misses_a_band`]; anything else is not asked.
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
        Surface::Torus(ring) => {
            return band.is_on_the_axis(ring.axis())
                && a_torus_on_the_axis_misses_a_band(ring, band);
        }
        _ => return false,
    };
    matches!(
        over_the_band.sign(),
        rcad_predicates::Decision::Pos | rcad_predicates::Decision::Neg
    )
}

/// Whether the torus `ring`, about the band's own axis, provably passes
/// through no point of `band` (Q1588).
///
/// Its side-of predicate takes the sign of `(v·v)·(x·x + R² − r²)² −
/// 4R²·|x × v|²`, with `v` its own direction and `x` the point less its
/// centre. The band's `w` runs along `v`, so the same expression with `w` for
/// `v` is a positive number times it; and the centre is on the band's line, so
/// `x × w` is the band's `(p − origin) × w`. Times `|w|²` once more, and with
/// `h = x·w`, which is `up` less the centre's `up`, Lagrange's identity
/// `|x|²·|w|² = h² + out` turns it into an expression in `up` and `out` alone:
///
/// ```text
/// F = (out + h² + (R² − r²)·|w|²)² − 4R²·|w|²·out
///   = (out + h² − (R² + r²)·|w|²)² + 4R²·|w|²·(h² − r²·|w|²)
/// ```
///
/// The two lines are one polynomial, so each worked over the band in interval
/// arithmetic holds every value `F` takes there, and either one keeping a
/// sign proves the band clear. Neither is enough alone. The first has `h`
/// once and is tight over a band at one `out`, such as a pipe's; the second
/// has `out` once and is tight over a band at one `up`, such as a flat face's.
/// A disk in the hole level with the top of the tube, where `h² = r²·|w|²`, is
/// proved clear only by the second: over `out` in `[0, (23/8)²]` the first is
/// `(out + 16)² − 64·out`, which interval arithmetic puts between `−273` and
/// `589`. A pipe through the tube's whole height is proved clear only by the
/// first.
fn a_torus_on_the_axis_misses_a_band(ring: &rcad_geom::Torus, band: ABandRoundAnAxis) -> bool {
    let exactly = rcad_predicates::Interval::exactly;
    let length = band.length();
    let height = squared(band.up - band.up_of(ring.centre().coordinates().map(exactly)));
    let ring_squared = squared(exactly(ring.ring_radius()));
    let tube_squared = squared(exactly(ring.tube_radius()));
    let four_r_squared_w = exactly(4.0) * ring_squared * length;
    let one_way = squared(band.out + height + (ring_squared - tube_squared) * length)
        - four_r_squared_w * band.out;
    let other_way = squared(band.out + height - (ring_squared + tube_squared) * length)
        + four_r_squared_w * (height - tube_squared * length);
    [one_way, other_way].iter().any(|value| {
        matches!(
            value.sign(),
            rcad_predicates::Decision::Pos | rcad_predicates::Decision::Neg
        )
    })
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
///
/// **On a cone, `out` is asked at three heights and not over its range**
/// (Q1461). There `out = slope²·up²` at every point, so the second test is
/// `|near| > √|n × w|²·slope·|up|`. Where `near` keeps one sign, both sides
/// are straight in `up` on either side of the apex, so the gap between them is
/// least at an end of the band or at the apex, and those three heights are
/// enough. Over the box `out` is as wide at the apex as at the far rim: a
/// frustum standing on its wide end, its apex just under a leaning top, was
/// not proved clear of it by that.
fn a_leaning_plane_misses_a_band(flat: &rcad_geom::Plane, band: ABandRoundAnAxis) -> bool {
    let exactly = rcad_predicates::Interval::exactly;
    let plane = APlaneRoundAnAxis::of(flat, band.origin, band.along);
    if !matches!(
        plane.near(band.up).sign(),
        rcad_predicates::Decision::Pos | rcad_predicates::Decision::Neg
    ) {
        return false;
    }
    let clear = |up, out| {
        matches!(
            (squared(plane.near(up)) - plane.leaning * out).sign(),
            rcad_predicates::Decision::Pos
        )
    };
    let Some(slope) = band.slope else {
        return clear(band.up, band.out);
    };
    [band.up.lo(), band.up.hi()]
        .into_iter()
        .chain(band.up.contains_zero().then_some(0.0))
        .all(|up| {
            let up = exactly(up);
            clear(up, squared(exactly(slope)) * squared(up))
        })
}

/// A plane read round the axis through `origin` along `w`, as
/// [`a_leaning_plane_misses_a_band`] reads it, in interval arithmetic on the
/// numbers the plane stores.
#[derive(Clone, Copy, Debug)]
struct APlaneRoundAnAxis {
    /// `(n·origin + d)·|w|²`.
    at_the_origin: rcad_predicates::Interval,
    /// `n·w`.
    rising: rcad_predicates::Interval,
    /// `|n × w|²`.
    leaning: rcad_predicates::Interval,
}

impl APlaneRoundAnAxis {
    fn of(flat: &rcad_geom::Plane, origin: [f64; 3], along: [f64; 3]) -> Self {
        let exactly = rcad_predicates::Interval::exactly;
        let [a, b, c, d] = flat.coefficients();
        let normal = [a, b, c].map(exactly);
        let along = along.map(exactly);
        let [x, y, z] = cross(normal, along);
        Self {
            at_the_origin: (dot(normal, origin.map(exactly)) + exactly(d)) * dot(along, along),
            rising: dot(normal, along),
            leaning: squared(x) + squared(y) + squared(z),
        }
    }

    /// The plane's equation scaled by `|w|²` at a point `up` along the axis,
    /// less the term that turns with the point round it.
    fn near(self, up: rcad_predicates::Interval) -> rcad_predicates::Interval {
        self.at_the_origin + self.rising * up
    }
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
/// **Two faces resting on each other along a line are skipped**, because they
/// only touch there ([`HowTwoFacesMeet::AlongALineTheyRestOn`]): a block's
/// wall and a bar standing against it, united into one solid touching itself
/// along that line (queue row Q1902). That is the answer a fuse now writes
/// (§39.4, the Q1901 note), and this used to refuse every fuse after it.
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

/// What two faces' surfaces do to each other, in the words the cutting here is
/// written in.
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
    /// They rest against each other along one straight line and cross nowhere:
    /// a plane or a pipe against a pipe's side, or a plane along a cone's
    /// ruling.
    ///
    /// Each is handed the other's surface as a cut, as for a crossing, and the
    /// arrangement runs the line along the pipe's axis or the cone's ruling
    /// (`AlongATouch`). **It is no crossing**, so two faces of one solid on
    /// such a pair are not checked for running through each other. Off the
    /// line the two surfaces share no point, and along it they are tangent,
    /// with a pipe wholly on one side of a plane or of another pipe and each
    /// nappe of a cone on one side of a plane. So they cross along no curve,
    /// and two faces that run through each other cross along one.
    AlongALineTheyRestOn,
    /// They cross along one curve, or two.
    AlongCurves,
}

/// Read a crossing in those words, and refuse a shape that has no word here.
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
        SurfaceMeet::Curves(runs) if a_line_they_rest_on(face, against, &runs) => {
            Ok(HowTwoFacesMeet::AlongALineTheyRestOn)
        }
        SurfaceMeet::Curves(_) => Ok(HowTwoFacesMeet::AlongCurves),
        _ => Err(Error::ACrossingThisCannotBeOrderedAlong),
    }
}

/// Whether `runs` is the one straight line a plane or a pipe rests on a pipe
/// along, or a plane on a cone.
///
/// **[`intersect`] has already told a touch from a cut**, and this reads its
/// answer the way the arrangement's `the_line_one_surface_rests_on_another_along`
/// does. Those three pairs come back as one bare [`EdgeRun::Straight`] where
/// they rest on each other, and as two lines a half-space picks apart, a closed
/// curve or nothing where they do not. Two planes cross in a bare straight run
/// too, and are none of these.
fn a_line_they_rest_on(face: &Surface, against: &Surface, runs: &[NamedRun]) -> bool {
    let [run] = runs else {
        return false;
    };
    *run.run() == EdgeRun::Straight
        && matches!(
            (face, against),
            (
                Surface::Plane(_) | Surface::Cylinder(_),
                Surface::Cylinder(_)
            ) | (Surface::Cylinder(_), Surface::Plane(_))
                | (Surface::Plane(_), Surface::Cone(_))
                | (Surface::Cone(_), Surface::Plane(_))
        )
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Axis, Cone, Cylinder, Isometry, Plane, Point, Scale, Surface, Torus, Vector};
    use rcad_predicates::{Interval, Policy, ProvenanceLog};

    use super::{
        ABandRoundAnAxis, SURFACE_PAIRS_REALLY_ASKED, a_face_misses_a_surface_by_its_band,
        a_surface_misses_a_band, gather, no_face_of_one_meets_the_other,
        the_band_a_plane_crosses_a_cone_in, the_band_a_plane_crosses_a_pipe_in,
        the_crossing_meets_none_of,
    };
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

    /// `n` cubes of side two, each one further along `z` by one unit than the
    /// last, so each pair overlaps by one unit, every cube's four side walls
    /// sit on the chain's own four planes — the way a connector's row of pins
    /// shares its two long walls along the row — and only the top and bottom
    /// caps differ from cube to cube.
    fn chain_of_cubes(n: usize) -> Vec<BakedSolid> {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let mut keys = Vec::with_capacity(n);
        for i in 0..n {
            let cube = model.add_cube(2.0, &policy, &mut log).expect("a cube");
            if i > 0 {
                model
                    .place_solid(
                        cube,
                        &Isometry::translation(Vector::new(0.0, 0.0, i as f64))
                            .expect("a finite shift"),
                    )
                    .expect("this model made the key");
            }
            keys.push(cube);
        }
        bake(&model, &keys, &policy, &mut ProvenanceLog::new()).expect("a chain of cubes bakes")
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

    /// The wall of a frustum of the cone about `−z` of slope `1/2` with its
    /// apex at `z = 22`, from `z = −4` up to `20`, as `band_a_face_is_inside`
    /// bands it: `2` to `26` down from the apex, widened to take the apex in.
    /// The plane `4z = x + 96` is `3.75` or more above it, and over the apex
    /// the band's box reaches out as far as the wide end (Q1461).
    #[test]
    fn a_leaning_top_over_a_frustums_narrow_end_is_proved_to_miss_its_wall_by_its_slope() {
        let top = leaning_plane_through(Point::new(0.0, 0.0, 24.0));
        let wall = |slope| ABandRoundAnAxis {
            origin: [0.0, 0.0, 22.0],
            along: [0.0, 0.0, -1.0],
            up: Interval::new(0.0, 26.0).expect("a range"),
            out: Interval::new(0.0, 169.0).expect("a range"),
            slope,
        };
        assert!(a_surface_misses_a_band(&top, wall(Some(0.5))));
        assert!(
            !a_surface_misses_a_band(&top, wall(None)),
            "the box alone holds a point 13 out at the apex, which the top passes over"
        );
    }

    /// `4z = x + 60` crosses that frustum's wall near `z = 15`.
    #[test]
    fn a_leaning_plane_across_a_frustums_wall_is_not_proved_to_miss_it() {
        let across = leaning_plane_through(Point::new(0.0, 0.0, 15.0));
        let wall = ABandRoundAnAxis {
            origin: [0.0, 0.0, 22.0],
            along: [0.0, 0.0, -1.0],
            up: Interval::new(0.0, 26.0).expect("a range"),
            out: Interval::new(0.0, 169.0).expect("a range"),
            slope: Some(0.5),
        };
        assert!(!a_surface_misses_a_band(&across, wall));
    }

    /// The top of a bar of radius `23/8` about `z`, standing in the hole of a
    /// ring of four round a tube of one, level with the top of the tube at
    /// `z = 1` (Q1588). There the tube is the one circle of radius four, so a
    /// disk reaching `23/8` out misses it and one reaching four out touches it.
    #[test]
    fn a_disk_in_a_rings_hole_level_with_the_top_of_its_tube_is_proved_to_miss_it() {
        let disk = |reach: f64| ABandRoundAnAxis {
            origin: [0.0, 0.0, -1.0],
            along: [0.0, 0.0, 1.0],
            up: Interval::exactly(2.0),
            out: Interval::new(0.0, reach * reach).expect("a range"),
            slope: None,
        };
        assert!(a_surface_misses_a_band(&ring_about_up(), disk(23.0 / 8.0)));
        assert!(
            !a_surface_misses_a_band(&ring_about_up(), disk(4.0)),
            "the disk reaches the circle the top of the tube runs round"
        );
    }

    /// The wall of that bar from `z = −1` to `1`, which is the tube's whole
    /// height, is clear of the tube's inner side, three out at `z = 0`; a wall
    /// three out touches it there; and a wall four out from `z = −1/2` to `1/2`
    /// is inside the tube all the way.
    #[test]
    fn a_pipe_through_a_rings_tube_is_proved_to_miss_it_only_where_it_does() {
        let wall = |radius: f64, low: f64, high: f64| ABandRoundAnAxis {
            origin: [0.0, 0.0, -1.0],
            along: [0.0, 0.0, 1.0],
            up: Interval::new(low + 1.0, high + 1.0).expect("a range"),
            out: Interval::exactly(radius * radius),
            slope: None,
        };
        assert!(a_surface_misses_a_band(
            &ring_about_up(),
            wall(23.0 / 8.0, -1.0, 1.0)
        ));
        assert!(a_surface_misses_a_band(
            &ring_about_up(),
            wall(4.0, -0.5, 0.5)
        ));
        assert!(
            !a_surface_misses_a_band(&ring_about_up(), wall(3.0, -1.0, 1.0)),
            "the wall touches the inner side of the tube at z = 0"
        );
    }

    /// The top `4z = x + 96` crosses the cone of slope `1/2` about `−z` with
    /// its apex at `z = 22` in an ellipse on the nappe above the apex, from
    /// `z = 214/9` over `x < 0` up to `170/7` over `x > 0`: `16/9` to `16/7`
    /// above the apex. A cone of slope `1/4` on the same axis with its apex at
    /// `z = 23` is `7/36` to `9/28` out from the axis at those heights and the
    /// first cone `8/9` to `8/7`, so the two never meet on the top. The two cones do meet,
    /// in circles at `z = 21` and `22⅓`, and the three surfaces have only the top
    /// flat, so the crossing is proved clear of the second cone only by the band
    /// round the axis it lies in (Q1461). Either cone crossing the top, with
    /// the other the third surface, either way round.
    #[test]
    fn the_ellipse_a_leaning_top_crosses_a_cone_in_meets_no_second_cone_on_its_axis() {
        let top = leaning_plane_through(Point::new(0.0, 0.0, 24.0));
        let (outer, bore) = (cone_about_down(22.0, 0.5), cone_about_down(23.0, 0.25));
        for (crossed, other) in [(outer, bore), (bore, outer)] {
            assert!(the_crossing_meets_none_of(&top, &crossed, &[other]));
            assert!(the_crossing_meets_none_of(&crossed, &top, &[other]));
        }
    }

    /// The cone of slope `1` about `−z` with its apex at `z = 23` is `0.78`
    /// out at the ellipse's lowest point, where the ellipse is `0.89` out, and
    /// `1.29` at its highest, where the ellipse is `1.14`. So it crosses the
    /// ellipse.
    #[test]
    fn the_ellipse_a_leaning_top_crosses_a_cone_in_is_not_proved_clear_of_a_cone_through_it() {
        let top = leaning_plane_through(Point::new(0.0, 0.0, 24.0));
        let (outer, through) = (cone_about_down(22.0, 0.5), cone_about_down(23.0, 1.0));
        assert!(!the_crossing_meets_none_of(&top, &outer, &[through]));
    }

    /// From `16/9` to `16/7` above the apex, and a millionth or so wider.
    #[test]
    fn the_band_round_the_ellipse_a_leaning_top_crosses_a_cone_in_is_its_height_and_no_more() {
        let top = leaning_plane_through(Point::new(0.0, 0.0, 24.0));
        let band = the_band_a_plane_crosses_a_cone_in(&top, &cone_about_down(22.0, 0.5))
            .expect("the top crosses the cone in an ellipse");
        let (lowest, highest) = (-16.0 / 9.0, -16.0 / 7.0);
        assert!(
            band.up.lo() < highest && band.up.lo() > highest - 1e-5,
            "{band:?}"
        );
        assert!(
            band.up.hi() > lowest && band.up.hi() < lowest + 1e-5,
            "{band:?}"
        );
        assert_eq!(band.slope, Some(0.5));
    }

    /// `x = 0` holds the axis and crosses the cone in two lines through the
    /// apex, and `4x = z` leans more than the wall and crosses it in a
    /// hyperbola. Neither ends, so no band holds them.
    #[test]
    fn a_plane_crossing_a_cone_in_a_curve_without_end_is_given_no_band() {
        let cone = cone_about_down(22.0, 0.5);
        for normal in [Vector::new(1.0, 0.0, 0.0), Vector::new(4.0, 0.0, -1.0)] {
            let plane = Surface::Plane(
                Plane::from_point_and_normal(Point::new(0.0, 0.0, 0.0), normal)
                    .expect("a normal with a length"),
            );
            assert!(
                the_band_a_plane_crosses_a_cone_in(&plane, &cone).is_none(),
                "{plane:?}"
            );
        }
    }

    /// The top `4z = x + 96` crosses the pipe of radius `1/2` about `z` in an
    /// ellipse from `z = 23.875` to `24.125`. The cones of slope `1/2` about
    /// `−z` with their apex at `z = 22` and `26` are each `15/16` to `17/16`
    /// out at those heights, so neither meets the ellipse, but each meets the
    /// pipe in two circles, and with only the top flat where the three meet
    /// cannot be worked out. So the crossing is proved clear of either cone
    /// only by the band round the axis it lies in (Q1583), the top and the
    /// pipe either way round.
    #[test]
    fn the_ellipse_a_leaning_top_crosses_a_pipe_in_meets_no_cone_on_its_axis() {
        let top = leaning_plane_through(Point::new(0.0, 0.0, 24.0));
        let pipe = pipe_about_up(0.5);
        for apex in [22.0, 26.0] {
            let cone = cone_about_down(apex, 0.5);
            assert!(the_crossing_meets_none_of(&top, &pipe, &[cone]), "{apex}");
            assert!(the_crossing_meets_none_of(&pipe, &top, &[cone]), "{apex}");
        }
    }

    /// The cone of slope `1/2` about `−z` with its apex at `z = 23` is `1/2`
    /// out at `z = 24`, where the top crosses the pipe at `(0, ±1/2, 24)`, so
    /// the three meet there.
    #[test]
    fn the_ellipse_a_leaning_top_crosses_a_pipe_in_is_not_proved_clear_of_a_cone_through_it() {
        let top = leaning_plane_through(Point::new(0.0, 0.0, 24.0));
        let through = cone_about_down(23.0, 0.5);
        assert!(!the_crossing_meets_none_of(
            &top,
            &pipe_about_up(0.5),
            &[through]
        ));
    }

    /// From `29.875` to `30.125` up the axis from the pipe's origin at
    /// `z = −6`, and a millionth of `30` or so wider. Every point of it is the
    /// pipe's own `1/4` out, and it is on no cone.
    #[test]
    fn the_band_round_the_ellipse_a_leaning_top_crosses_a_pipe_in_is_its_height_and_no_more() {
        let top = leaning_plane_through(Point::new(0.0, 0.0, 24.0));
        let band = the_band_a_plane_crosses_a_pipe_in(&top, &pipe_about_up(0.5))
            .expect("the top crosses the pipe in an ellipse");
        let (lowest, highest) = (29.875, 30.125);
        assert!(
            band.up.lo() < lowest && band.up.lo() > lowest - 4e-5,
            "{band:?}"
        );
        assert!(
            band.up.hi() > highest && band.up.hi() < highest + 4e-5,
            "{band:?}"
        );
        assert_eq!((band.out.lo(), band.out.hi()), (0.25, 0.25));
        assert_eq!(band.slope, None);
    }

    /// `x = 0` and `x = 1/4` run along the pipe's axis and cross the pipe in
    /// two lines along it, and `x = 1/2` touches it along one. None of them
    /// ends, so no band holds them, and the crossing is asked as it was before
    /// the band: `x = 1/4` crosses the cone of slope `1/2` with its apex at
    /// `z = 22` where that cone is `1/2` out, at `z = 21` and `23`, and is
    /// not proved clear of it.
    #[test]
    fn a_plane_along_a_pipes_axis_crosses_it_in_lines_without_end_and_is_given_no_band() {
        let pipe = pipe_about_up(0.5);
        for across in [0.0, 0.25, 0.5] {
            let plane = Surface::Plane(
                Plane::from_point_and_normal(
                    Point::new(across, 0.0, 0.0),
                    Vector::new(1.0, 0.0, 0.0),
                )
                .expect("a normal with a length"),
            );
            assert!(
                the_band_a_plane_crosses_a_pipe_in(&plane, &pipe).is_none(),
                "{plane:?}"
            );
            assert!(
                the_band_a_plane_crosses_a_pipe_in(&pipe, &plane).is_none(),
                "{plane:?}"
            );
        }
        let along = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.25, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
                .expect("a normal with a length"),
        );
        assert!(!the_crossing_meets_none_of(
            &along,
            &pipe,
            &[cone_about_down(22.0, 0.5)]
        ));
    }

    /// The pipe of radius `radius` about the `z` axis, through `(0, 0, −6)`.
    fn pipe_about_up(radius: f64) -> Surface {
        Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::new(0.0, 0.0, -6.0), Vector::new(0.0, 0.0, 1.0))
                    .expect("z is a direction"),
                radius,
            )
            .expect("a positive radius"),
        )
    }

    fn cone_about_down(apex: f64, slope: f64) -> Surface {
        Surface::Cone(
            Cone::new(
                Axis::new(Point::new(0.0, 0.0, apex), Vector::new(0.0, 0.0, -1.0))
                    .expect("z is a direction"),
                slope,
            )
            .expect("a positive slope"),
        )
    }

    /// The ring of four round a tube of one about the `z` axis.
    fn ring_about_up() -> Surface {
        Surface::Torus(Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring"))
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

    /// A bar standing against a block's wall touches it along a line and
    /// crosses it nowhere, so one solid made of both does not run through
    /// itself. It is what a union of the two answers (§39.4, the Q1901 note),
    /// and refusing it refused every fuse after that union (queue row Q1902).
    #[test]
    fn a_solid_whose_bar_rests_on_its_blocks_wall_is_not_refused() {
        let answer = a_block_and_a_bar_as_one(5.0);

        assert!(answer.is_ok(), "expected no refusal, got {answer:?}");
    }

    #[test]
    fn a_solid_whose_bar_reaches_into_its_blocks_wall_is_refused() {
        let answer = a_block_and_a_bar_as_one(4.75);

        assert!(
            matches!(answer, Err(Error::ArgumentIntersectsItself)),
            "expected a refusal, got {answer:?}"
        );
    }

    #[test]
    fn a_solid_of_two_bars_side_by_side_is_not_refused() {
        let answer = two_bars_as_one(2.0);

        assert!(answer.is_ok(), "expected no refusal, got {answer:?}");
    }

    #[test]
    fn a_solid_of_two_bars_that_overlap_is_refused() {
        let answer = two_bars_as_one(1.5);

        assert!(
            matches!(answer, Err(Error::ArgumentIntersectsItself)),
            "expected a refusal, got {answer:?}"
        );
    }

    /// A block of side four and a bar of radius one standing at `x`, `y = 2`
    /// from height one to three, wired into one solid by hand and gathered.
    /// At `x = 5` the bar's wall rests on the block's wall `x = 4`.
    fn a_block_and_a_bar_as_one(x: f64) -> Result<(), Error> {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let block = model.add_cube(4.0, &policy, &mut log).expect("a block");
        let bar = model
            .add_round_bar(
                1.0,
                2.0,
                &Isometry::translation(Vector::new(x, 2.0, 1.0)).expect("a finite shift"),
            )
            .expect("a bar of radius one");
        let both = both_solids_as_one(&mut model, block, bar);

        let mut solids =
            bake(&model, &[both], &policy, &mut ProvenanceLog::new()).expect("it bakes");
        gather(&mut solids, &policy, &mut log)
    }

    /// Two bars of radius one and two high, the second standing `apart` from
    /// the first along y, wired into one solid by hand and gathered. Two apart
    /// they rest on each other along a line, away from either bar's seam.
    fn two_bars_as_one(apart: f64) -> Result<(), Error> {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let one = model
            .add_round_bar(1.0, 2.0, &Isometry::identity())
            .expect("a bar of radius one");
        let other = model
            .add_round_bar(
                1.0,
                2.0,
                &Isometry::translation(Vector::new(0.0, apart, 0.0)).expect("a finite shift"),
            )
            .expect("another bar of radius one");
        let both = both_solids_as_one(&mut model, one, other);

        let mut solids =
            bake(&model, &[both], &policy, &mut ProvenanceLog::new()).expect("it bakes");
        gather(&mut solids, &policy, &mut log)
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

    /// A wall shared by every cube in the chain is cut by the chain's other
    /// two walls and, from every cube including its own, that cube's two
    /// caps — the memoized path and the one it replaced hang the same
    /// surfaces on a face, so this is unmoved by [`HowFacesMeetBySurface`]
    /// existing at all.
    ///
    /// Worked by hand from how [`cuts_from_how_they_meet`] and
    /// [`boundary_of`] fill a face's cuts: this wall's own four edges border
    /// the chain's other two walls and its own cube's two caps (not the wall
    /// directly opposite it, which it shares no edge with and which is
    /// parallel to it besides); every other cube contributes those same two
    /// walls again by the coplanar branch, and every cube's own two caps by
    /// the crossing branch. The two other walls arrive once each, no matter
    /// how many cubes repeat them; each cube's own two caps once each. Five
    /// cubes: two walls plus ten caps, twelve surfaces.
    #[test]
    fn a_wall_the_whole_chain_shares_is_cut_by_every_cubes_caps() {
        let mut solids = chain_of_cubes(5);

        gathered(&mut solids);

        let wall = facing(&solids[0], Vector::new(-1.0, 0.0, 0.0));
        assert_eq!(
            distinct_cuts(wall).len(),
            12,
            "the chain's other two walls and both caps of each of the five cubes"
        );
    }

    /// [`HowFacesMeetBySurface`] asks [`how_two_faces_meet`] once per distinct
    /// pair of surfaces, not once per pair of faces — a connector's row of
    /// pins repeats the same cap height and the same two walls along the
    /// row, and this chain of cubes repeats its four walls the same way.
    ///
    /// Worked by hand: five cubes carry four shared walls and two caps each
    /// (ten caps, all distinct from each other and from every wall). The
    /// distinct unordered surface pairs this chain's faces ever put in front
    /// of [`HowFacesMeetBySurface::ask`] are the ten wall-wall pairs (four
    /// walls, repeats allowed), the `4 * 10 = 40` wall-cap pairs, and the
    /// cap-cap pairs — `4` crossing pairs for each of the ten unordered pairs
    /// of distinct cubes plus one same-cube pair per cube, `4 * 10 + 5 = 45`
    /// — for `10 + 40 + 45 = 95` real calls. [`gather`]'s loop puts
    /// `36 * 10 + 15 * 5 = 435` face pairs in front of
    /// [`HowFacesMeetBySurface::ask`] in all, so this is what caches a little
    /// over four fifths of them away.
    #[test]
    fn a_chain_of_five_cubes_asks_how_two_faces_meet_ninety_five_times() {
        let mut solids = chain_of_cubes(5);
        SURFACE_PAIRS_REALLY_ASKED.store(0, std::sync::atomic::Ordering::Relaxed);

        gathered(&mut solids);

        assert_eq!(
            SURFACE_PAIRS_REALLY_ASKED.load(std::sync::atomic::Ordering::Relaxed),
            95,
            "five cubes' worth of shared walls and distinct caps, worked out by hand above"
        );
    }
}
