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
//! Two solids are skipped whole only when no chain of overlapping boxes joins
//! them: solids that nothing links leave every face of each other alone. Two
//! that miss each other but are both reached by a third are still cut against
//! each other, because the third's faces are cut by both and the edges it
//! shares with each must be split at the same points (§222.3 item 1). Inside a
//! pair that might touch, **no face pair is skipped**, and a box test there
//! would be wrong rather than merely keen.
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
//! [`assemble`](super::assemble)'s joining folds the extra pieces back into one
//! face and drops the corners that stopped being corners, so nothing of the
//! split reaches the answer.
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

use rcad_geom::{Surface, SurfaceMeet, intersect};
use rcad_predicates::{Policy, ProvenanceLog};

use crate::{Error, Result};

use super::Sameness;
use super::arrange::stretches_of_a_line_inside;
use super::bake::{BakedFace, BakedSolid};

/// Work out every cut line, and hang each on the face it cuts.
///
/// Every face is cut against every other face, in its own solid and in every
/// other, whichever group each came from. The header says why the solid's own
/// planes are in there.
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
    let mut found: Vec<(usize, usize, Surface)> = Vec::new();
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
                    found.extend(onto_face.into_iter().map(|cut| (one, mine, cut)));
                    found.extend(onto_against.into_iter().map(|cut| (other, theirs, cut)));
                }
            }
        }
    }

    for (solid, face, cut) in found {
        solids[solid].faces[face].cuts.push(cut);
    }
    Ok(())
}

/// Which solids are joined to each other by a chain of solids whose boxes
/// overlap.
///
/// **Two solids whose boxes miss are still cut against each other when a third
/// solid's box reaches both** (§222.3 item 1). A block with one box poking up
/// through its floor and another sitting inside it: the inside box's walls cut
/// the block's floor, and that floor meets the poking box's walls along the
/// footprint. Skip the poking box against the inside one and the floor side of
/// that edge is split where the inside box's walls cross it while the wall side
/// is not, so the edge comes back with one face along it. It is the header's
/// argument one solid further out: both sides of every edge of the answer have
/// to be diced by the one set of planes, and that set is everything in the
/// chain.
fn solids_within_reach_of_each_other(solids: &[BakedSolid]) -> Sameness {
    let mut within_reach = Sameness::of(solids.len());
    for one in 0..solids.len() {
        for other in one + 1..solids.len() {
            if solids[one].bounds.overlap(solids[other].bounds) {
                within_reach.join(one, other);
            }
        }
    }
    within_reach
}

/// The crossings each of two faces cuts in the other, each as the second surface
/// of the pair — the face's own surface is the first and is not repeated.
///
/// [`intersect`] is the one authority on how two surfaces meet (§24.3), so
/// [`how_two_faces_meet`] asks it rather than working the three cases out again.
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
        HowTwoFacesMeet::AlongCurves => Ok((vec![against.surface], vec![face.surface])),
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
    /// Nothing to cut along: they miss each other altogether, or they touch at
    /// one point and cross nowhere.
    ///
    /// A pair that touches at one point cuts nothing (§31.3): neither face is
    /// split and neither solid learns about the other. Two tangent balls united
    /// come back as two shells that share no vertex.
    Nowhere,
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
        SurfaceMeet::Nothing | SurfaceMeet::TouchAtOnePoint { .. } => Ok(HowTwoFacesMeet::Nowhere),
        SurfaceMeet::Curves(_) => Ok(HowTwoFacesMeet::AlongCurves),
        _ => Err(Error::ACrossingThisCannotBeOrderedAlong),
    }
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Isometry, Plane, Scale, Surface, Vector};
    use rcad_predicates::{Policy, ProvenanceLog};

    use super::gather;
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
