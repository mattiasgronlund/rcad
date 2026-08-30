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
//! Lines are gathered between **every pair of distinct input solids**, whether
//! the two are in the same group or not. `A` is the union of the arguments and
//! `B` the union of the tools (§17.4), and a union is only real if the solids
//! making it up have been cut against each other: otherwise two overlapping
//! arguments keep the halves of each other's faces that are buried inside the
//! other one, and the answer is a self-overlapping shell rather than a union.
//!
//! # Every face of a solid is cut by every plane of the other
//!
//! Two solids whose boxes miss each other are skipped whole, and that changes
//! no answer: solids that cannot touch leave every face of both alone. Inside a
//! pair that might touch, though, **no face pair is skipped**, and a box test
//! there would be wrong rather than merely keen.
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

use rcad_geom::{Plane, PlaneAgreement};
use rcad_predicates::{Policy, ProvenanceLog};

use crate::{Error, Result};

use super::arrange::{agreement_of_two_planes, planes_are_parallel, stretches_of_a_line_inside};
use super::bake::{BakedFace, BakedSolid};

/// Work out every cut line, and hang each on the face it cuts.
///
/// Every pair of distinct solids is cut against every other, whichever group
/// each came from.
///
/// # Errors
///
/// [`Error::ArgumentIntersectsItself`] when a solid has two faces that really do
/// run through each other, [`Error::Undecidable`] when a decision that takes
/// could not be settled under `policy`.
pub(crate) fn gather(
    solids: &mut [BakedSolid],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<()> {
    for solid in solids.iter() {
        check_it_does_not_cross_itself(solid, policy, log)?;
    }

    let mut found: Vec<(usize, usize, Plane)> = Vec::new();
    for one in 0..solids.len() {
        for other in (one + 1)..solids.len() {
            if !solids[one].bounds.overlap(solids[other].bounds) {
                continue;
            }
            for mine in 0..solids[one].faces.len() {
                for theirs in 0..solids[other].faces.len() {
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

/// The lines each of two faces cuts in the other, each as the second plane of
/// the pair — the face's own plane is the first and is not repeated.
fn cuts_between(
    face: &BakedFace,
    against: &BakedFace,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<(Vec<Plane>, Vec<Plane>)> {
    match agreement_of_two_planes(&face.plane, &against.plane, log)? {
        PlaneAgreement::SameWay | PlaneAgreement::OppositeWay => {
            Ok((boundary_of(against), boundary_of(face)))
        }
        PlaneAgreement::Different => {
            if planes_are_parallel(&face.plane, &against.plane, policy, log)? {
                return Ok((Vec::new(), Vec::new()));
            }
            Ok((vec![against.plane], vec![face.plane]))
        }
        other => panic!("plane agreement came back with an answer nothing here knows: {other:?}"),
    }
}

/// One cut line per edge of a face, for cutting a face coplanar with it.
fn boundary_of(face: &BakedFace) -> Vec<Plane> {
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
            match agreement_of_two_planes(&face.plane, &against.plane, log)? {
                PlaneAgreement::SameWay | PlaneAgreement::OppositeWay => continue,
                PlaneAgreement::Different => (),
                other => {
                    panic!("plane agreement came back with an answer nothing here knows: {other:?}")
                }
            }
            if planes_are_parallel(&face.plane, &against.plane, policy, log)? {
                continue;
            }
            let mine = stretches_of_a_line_inside(face, &against.plane, policy, log)?;
            let theirs = stretches_of_a_line_inside(against, &face.plane, policy, log)?;
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

#[cfg(test)]
mod tests {
    use rcad_geom::{Isometry, Plane, Scale, Vector};
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
        model.place_solid(
            other,
            &Isometry::translation(shift).expect("a finite shift"),
        );
        bake(&model, &[one, other], &mut ProvenanceLog::new()).expect("two cubes bake")
    }

    fn facing(solid: &BakedSolid, way: Vector) -> &BakedFace {
        solid
            .faces
            .iter()
            .find(|face| face.plane.normal() == way)
            .expect("a cube has a face pointing every way")
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
        let mut cutting: Vec<[f64; 4]> = up.cuts.iter().map(Plane::coefficients).collect();
        cutting.sort_by(|one, other| one.partial_cmp(other).expect("no nans here"));
        assert_eq!(
            cutting,
            [
                [-1.0, 0.0, 0.0, 1.0],
                [0.0, -1.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, -3.0],
                [1.0, 0.0, 0.0, -3.0]
            ],
            "all four walls of the upper cube, the two far ones included: a cut is by a plane, \
             and a plane reaches past the face carrying it"
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
        let mut cutting: Vec<[f64; 4]> = down.cuts.iter().map(Plane::coefficients).collect();
        cutting.sort_by(|one, other| one.partial_cmp(other).expect("no nans here"));
        assert_eq!(
            cutting.len(),
            4,
            "the four walls of the upper cube, none of which reaches down here"
        );
    }

    /// Two coplanar faces cut each other with the four edges of the other, not
    /// with a plane that is the same plane.
    ///
    /// The upper cube's four walls also reach down to the face they share and
    /// cut it directly, and they lie on the very same four planes — so eight cut
    /// lines arrive and four distinct planes carry them. Without the coplanar
    /// branch only the four direct ones would.
    #[test]
    fn one_cube_sitting_on_another_is_cut_once_per_edge_of_the_face_they_share() {
        let mut solids = two_cubes(Vector::new(0.0, 0.0, 2.0));

        gathered(&mut solids);

        for (solid, way) in [(0, 1.0), (1, -1.0)] {
            let shared = facing(&solids[solid], Vector::new(0.0, 0.0, way));
            assert_eq!(shared.cuts.len(), 8);
            let mut distinct: Vec<[f64; 4]> = shared.cuts.iter().map(Plane::coefficients).collect();
            distinct.sort_by(|one, other| one.partial_cmp(other).expect("no nans here"));
            distinct.dedup();
            assert_eq!(distinct.len(), 4);
        }
    }

    #[test]
    fn two_cubes_that_are_nowhere_near_each_other_cut_nothing() {
        let mut solids = two_cubes(Vector::new(10.0, 0.0, 0.0));

        gathered(&mut solids);

        for solid in &solids {
            for face in &solid.faces {
                assert!(face.cuts.is_empty());
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
        model.place_solid(
            other,
            &Isometry::translation(Vector::new(1.0, 1.0, 1.0)).expect("a finite shift"),
        );
        let crossed = both_solids_as_one(&mut model, one, other);

        let mut solids = bake(&model, &[crossed], &mut ProvenanceLog::new()).expect("it bakes");
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
        assert_eq!(
            up.cuts.len(),
            4,
            "the top of the lower cube is cut by the four walls of the upper one"
        );
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
