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
//! Lines are gathered between the two groups and not within one of them. A face
//! of an argument is never cut by another argument, so two argument solids that
//! overlap each other are not decomposed against each other. The signature calls
//! `A` the union of the arguments (§17.4), and making that union real is work
//! that has no consumer yet.

use rcad_geom::{Plane, PlaneAgreement};
use rcad_predicates::{Policy, ProvenanceLog};

use crate::{Error, FaceKey, Result};

use super::arrange::{agreement_of_two_planes, planes_are_parallel, stretches_of_a_line_inside};
use super::bake::{BakedFace, BakedSolid};

/// One line cutting one face: the second plane of the pair.
///
/// The face's own plane is the first, so it is not stored here.
#[derive(Clone, Debug)]
pub(crate) struct Cut {
    /// The plane that, met with the face's own, carries the line.
    pub(crate) partner: Plane,
    /// The face of the other group that put this line here.
    pub(crate) from: FaceKey,
}

/// Work out every cut line, and hang each on the face it cuts.
///
/// # Errors
///
/// [`Error::ArgumentIntersectsItself`] when a solid has two faces that really do
/// run through each other, [`Error::Undecidable`] when a decision that takes
/// could not be settled under `policy`.
pub(crate) fn gather(
    arguments: &mut [BakedSolid],
    tools: &mut [BakedSolid],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<()> {
    for solid in arguments.iter().chain(tools.iter()) {
        check_it_does_not_cross_itself(solid, policy, log)?;
    }

    let mut onto_arguments: Vec<(usize, usize, Cut)> = Vec::new();
    let mut onto_tools: Vec<(usize, usize, Cut)> = Vec::new();
    for (argument, one) in arguments.iter().enumerate() {
        for (tool, other) in tools.iter().enumerate() {
            if !one.bounds.overlap(other.bounds) {
                continue;
            }
            for (mine, face) in one.faces.iter().enumerate() {
                for (theirs, against) in other.faces.iter().enumerate() {
                    if !face.bounds.overlap(against.bounds) {
                        continue;
                    }
                    let (onto_face, onto_against) = cuts_between(face, against, policy, log)?;
                    onto_arguments.extend(onto_face.into_iter().map(|cut| (argument, mine, cut)));
                    onto_tools.extend(onto_against.into_iter().map(|cut| (tool, theirs, cut)));
                }
            }
        }
    }

    for (solid, face, cut) in onto_arguments {
        arguments[solid].faces[face].cuts.push(cut);
    }
    for (solid, face, cut) in onto_tools {
        tools[solid].faces[face].cuts.push(cut);
    }
    Ok(())
}

/// The lines each of two faces cuts in the other.
fn cuts_between(
    face: &BakedFace,
    against: &BakedFace,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<(Vec<Cut>, Vec<Cut>)> {
    match agreement_of_two_planes(&face.plane, &against.plane, log)? {
        PlaneAgreement::SameWay | PlaneAgreement::OppositeWay => {
            Ok((boundary_of(against), boundary_of(face)))
        }
        PlaneAgreement::Different => {
            if planes_are_parallel(&face.plane, &against.plane, policy, log)? {
                return Ok((Vec::new(), Vec::new()));
            }
            Ok((
                vec![Cut {
                    partner: against.plane,
                    from: against.key,
                }],
                vec![Cut {
                    partner: face.plane,
                    from: face.key,
                }],
            ))
        }
        other => panic!("plane agreement came back with an answer nothing here knows: {other:?}"),
    }
}

/// One cut line per edge of a face, for cutting a face coplanar with it.
fn boundary_of(face: &BakedFace) -> Vec<Cut> {
    face.wires
        .iter()
        .flatten()
        .map(|edge| Cut {
            partner: edge.across,
            from: face.key,
        })
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
    use rcad_geom::{Isometry, Scale, Vector};
    use rcad_predicates::{Policy, ProvenanceLog};

    use super::gather;
    use crate::fuse::bake::{BakedFace, BakedSolid, bake};
    use crate::{Error, Model, SolidKey};

    /// Two cubes of side two, the second one moved by `shift`.
    fn two_cubes(shift: Vector) -> (Vec<BakedSolid>, Vec<BakedSolid>) {
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
        (
            bake(&model, &[one]).expect("a cube bakes"),
            bake(&model, &[other]).expect("a cube bakes"),
        )
    }

    fn facing(solid: &BakedSolid, way: Vector) -> &BakedFace {
        solid
            .faces
            .iter()
            .find(|face| face.plane.normal() == way)
            .expect("a cube has a face pointing every way")
    }

    fn gathered(arguments: &mut [BakedSolid], tools: &mut [BakedSolid]) {
        let mut log = ProvenanceLog::new();
        gather(arguments, tools, &Policy::certified(), &mut log)
            .expect("two lattice-aligned cubes leave nothing undecided");
        assert!(log.is_empty(), "nothing here had to be approximated");
    }

    #[test]
    fn two_cubes_that_overlap_cut_each_other_where_their_faces_cross() {
        let (mut arguments, mut tools) = two_cubes(Vector::new(1.0, 1.0, 1.0));

        gathered(&mut arguments, &mut tools);

        let up = facing(&arguments[0], Vector::new(0.0, 0.0, 1.0));
        let mut cutting: Vec<[f64; 4]> = up
            .cuts
            .iter()
            .map(|cut| cut.partner.coefficients())
            .collect();
        cutting.sort_by(|one, other| one.partial_cmp(other).expect("no nans here"));
        assert_eq!(
            cutting,
            [[-1.0, 0.0, 0.0, 1.0], [0.0, -1.0, 0.0, 1.0]],
            "the top of the lower cube is cut by the two walls of the upper one that reach it"
        );
    }

    #[test]
    fn a_face_the_other_cube_cannot_reach_is_cut_by_nothing() {
        let (mut arguments, mut tools) = two_cubes(Vector::new(1.0, 1.0, 1.0));

        gathered(&mut arguments, &mut tools);

        let down = facing(&arguments[0], Vector::new(0.0, 0.0, -1.0));
        assert!(
            down.cuts.is_empty(),
            "the bottom of the lower cube is nowhere near the upper one"
        );
    }

    #[test]
    fn one_cube_sitting_on_another_is_cut_once_per_edge_of_the_face_they_share() {
        let (mut arguments, mut tools) = two_cubes(Vector::new(0.0, 0.0, 2.0));

        gathered(&mut arguments, &mut tools);

        let up = facing(&arguments[0], Vector::new(0.0, 0.0, 1.0));
        let down = facing(&tools[0], Vector::new(0.0, 0.0, -1.0));
        assert_eq!(
            up.cuts.iter().filter(|cut| cut.from == down.key).count(),
            4,
            "two coplanar faces cut each other with the four edges of the other, not with a plane \
             that is the same plane"
        );
        assert_eq!(down.cuts.iter().filter(|cut| cut.from == up.key).count(), 4);
    }

    #[test]
    fn two_cubes_that_are_nowhere_near_each_other_cut_nothing() {
        let (mut arguments, mut tools) = two_cubes(Vector::new(10.0, 0.0, 0.0));

        gathered(&mut arguments, &mut tools);

        for solid in arguments.iter().chain(&tools) {
            for face in &solid.faces {
                assert!(face.cuts.is_empty());
            }
        }
    }

    #[test]
    fn a_cube_does_not_run_through_itself() {
        let (mut arguments, mut tools) = two_cubes(Vector::new(1.0, 1.0, 1.0));
        gathered(&mut arguments, &mut tools);
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

        let mut arguments = bake(&model, &[crossed]).expect("it bakes");
        let refusal = gather(&mut arguments, &mut [], &policy, &mut log);

        assert!(
            matches!(refusal, Err(Error::ArgumentIntersectsItself)),
            "expected a refusal, got {refusal:?}"
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
