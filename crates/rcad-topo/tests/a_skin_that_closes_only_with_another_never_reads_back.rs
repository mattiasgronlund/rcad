//! A cube's six faces, or a round bar's three, shared out between two solids
//! and written into a file.
//!
//! Each half is an open skin. An edge along the cut is walked once from each
//! half's shell, so it still has two coedges running opposite ways, which is
//! what the door asked for. Every key resolves, every face is in one shell and
//! every shell is in one solid. So the door let in all 34 ways of cutting
//! these two skins in two. Once in, the halves gave wrong answers and a fuse of
//! a bar half panicked. A panic on a caller's file is the defect, and the file
//! is now refused at the door: a shell must close on its own, so an edge
//! walked from two shells is refused as
//! `Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells`. `docs/decisions.md`
//! §342 is the record.
//!
//! The halves are made by rewriting the bytes, because nothing in the public
//! API builds one. The shell and solid arenas are the last two things
//! `ModelData` writes, so the split rewrites the end of the file and leaves
//! every face, wire, edge and corner byte for byte as it was.
//!
//! The door's refusal is `postcard::Error::SerdeDeCustom`, and postcard drops
//! the message that says why. The unit tests in `validate.rs` pin the name,
//! asking the same check through `Model::check_shape_is_possible`, `validate`
//! and `validate_solid`.

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule};

/// The bytes of one `FaceUse` or `ShellUse`: a two-byte key, a placement of
/// seven `f64`s, and an orientation.
const ONE_USE: usize = 2 + 7 * 8 + 1;

#[test]
fn every_way_of_cutting_a_cubes_skin_in_two_is_refused_at_the_door() {
    let bytes = written(&a_cube());
    let mut refused = 0;
    for mask in 1u32..63 {
        if mask & 1 == 0 {
            continue;
        }
        let first: Vec<usize> = (0..6).filter(|face| mask & (1 << face) != 0).collect();
        let second: Vec<usize> = (0..6).filter(|face| !first.contains(face)).collect();
        assert_eq!(
            what_the_door_says(&with_the_faces_shared_out(&bytes, &[&first, &second])),
            Some(postcard::Error::SerdeDeCustom),
            "the cube cut into {first:?} and {second:?}"
        );
        refused += 1;
    }
    assert_eq!(refused, 31, "every way of cutting six faces in two");
}

#[test]
fn every_way_of_cutting_a_round_bars_skin_in_two_is_refused_at_the_door() {
    let bytes = written(&a_round_bar());
    for alone in 0..3 {
        let rest: Vec<usize> = (0..3).filter(|&face| face != alone).collect();
        assert_eq!(
            what_the_door_says(&with_the_faces_shared_out(&bytes, &[&[alone], &rest])),
            Some(postcard::Error::SerdeDeCustom),
            "the bar cut into [{alone}] and {rest:?}"
        );
    }
}

/// **The fuse that panicked on a bar half has nothing to run on any more.**
///
/// Before the door asked about shells, this file read back, and fusing the
/// half holding the wall and the top cap panicked with "no entry found for
/// key" in `fuse/bake.rs`, even when the cube it was fused with was nowhere
/// near. The fuse looked up the face across an edge of the cut among the faces
/// of the half, and that face is in the other half. A fuse takes keys into a
/// model, and the only model that holds this half is the one these bytes would
/// have become.
///
/// The same fuse of the whole bar answers, so what is refused is the file and
/// not the fuse.
#[test]
fn the_bar_half_a_fuse_panicked_on_never_becomes_a_model() {
    let bar = a_round_bar();
    let bytes = written(&bar);
    let with_its_wall_and_top_apart_from_its_bottom = [&[0][..], &[1, 2]];

    assert_eq!(
        what_the_door_says(&with_the_faces_shared_out(
            &bytes,
            &with_its_wall_and_top_apart_from_its_bottom
        )),
        Some(postcard::Error::SerdeDeCustom),
        "the half is refused before a fuse can be handed it"
    );

    let mut whole: Model = postcard::from_bytes(&bytes).expect("the whole bar reads back");
    let bar = whole.solids().next().expect("the bar").key();
    let cube = whole
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    whole
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(10.0, 0.0, 0.0)).expect("a shift"),
        )
        .expect("moved clear of the bar");
    let fused = whole
        .fuse(
            &[bar],
            &[cube],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the whole bar and a cube clear of it unite");
    assert_eq!(fused.solids().len(), 2, "two solids that do not touch");
}

/// Rewriting the end of the file with every face in one shell gives back the
/// file as it was written, byte for byte, so the cut is the only thing the
/// tests above change.
#[test]
fn sharing_the_faces_out_into_one_shell_rewrites_the_file_it_was_given() {
    for (bytes, faces) in [(written(&a_cube()), 6), (written(&a_round_bar()), 3)] {
        let every_face: Vec<usize> = (0..faces).collect();
        assert_eq!(with_the_faces_shared_out(&bytes, &[&every_face]), bytes);
        postcard::from_bytes::<Model>(&bytes).expect("the whole skin reads back");
    }
}

fn a_cube() -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    model
}

fn a_round_bar() -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    model
        .add_round_bar(1.0, 2.0, &Isometry::identity())
        .expect("a bar of radius one and height two");
    model
}

fn written(model: &Model) -> Vec<u8> {
    postcard::to_allocvec(model).expect("a model writes out")
}

/// Why these bytes did not become a model, or `None` when they did.
fn what_the_door_says(bytes: &[u8]) -> Option<postcard::Error> {
    postcard::from_bytes::<Model>(bytes).err()
}

/// The bytes of a model holding one solid with one shell, rewritten so each
/// group of that shell's faces is a shell of its own, used by a solid of its
/// own. A face is named by where it sits in the shell's list.
///
/// Each arena is written as its number of slots, then each slot: an empty
/// slot zero, then a `1` for "something here", the entity, and the slot's
/// version, which is `1` for everything written once. The asserts check that
/// layout before relying on it.
fn with_the_faces_shared_out(bytes: &[u8], groups: &[&[usize]]) -> Vec<u8> {
    let faces: usize = groups.iter().map(|group| group.len()).sum();
    let shells_len = 1 + 2 + 1 + 1 + faces * ONE_USE + 1;
    let solids_len = 1 + 2 + 1 + 1 + ONE_USE + 1;
    let (head, tail) = bytes.split_at(bytes.len() - shells_len - solids_len);
    let (shells, solids) = tail.split_at(shells_len);
    assert_eq!(
        &shells[..5],
        &[2, 0, 0, 1, faces as u8],
        "one shell of every face"
    );
    assert_eq!(shells[shells_len - 1], 1, "the shell's version");
    let face_uses: Vec<&[u8]> = (0..faces)
        .map(|at| &shells[5 + at * ONE_USE..5 + (at + 1) * ONE_USE])
        .collect();
    for used in &face_uses {
        assert!(
            (1..=faces as u8).contains(&used[0]) && used[1] == 1,
            "a face use starts with a face's key"
        );
    }
    assert_eq!(&solids[..5], &[2, 0, 0, 1, 1], "one solid of one shell");
    assert_eq!(solids[solids_len - 1], 1, "the solid's version");
    let shell_use = &solids[5..5 + ONE_USE];
    assert_eq!(&shell_use[..2], &[1, 1], "the shell's key");

    let slots = groups.len() as u8 + 1;
    let mut out = head.to_vec();
    out.extend([slots, 0, 0]);
    for group in groups {
        out.extend([1, group.len() as u8]);
        for &face in *group {
            out.extend(face_uses[face]);
        }
        out.push(1);
    }
    out.extend([slots, 0, 0]);
    for shell in 1..slots {
        out.extend([1, 1, shell, 1]);
        out.extend(&shell_use[2..]);
        out.push(1);
    }
    out
}
