//! The writer's checked-in corpus, replayed on every commit.
//!
//! `fuzz/fuzz_targets/no_model_makes_the_writer_panic.rs` is the `cargo-fuzz`
//! target, needs nightly, and does not run in `just gate`. This file does, over
//! the same harness — `tests/fuzzing/writer.rs` — which the target reaches
//! across to rather than copying.
//!
//! # Sixteen of 1180, and all sixteen empty
//!
//! Measured 2026-09-13, before the two control seeds below were added: of 1180
//! seeds, **16 decoded into a `Model`**, and every one of the sixteen held at
//! most one solid and no faces, no edges and no vertices at all. So
//! `rcad_step::write` was reached 16 times out of 1180 and turned away every
//! time at `TheModelHasNoSolid` or `ASolidWithNoShell`, the two shallowest
//! refusals it has, and `every_number_the_door_promised_is_real` walked three
//! empty iterators 1180 times over.
//!
//! The fuse corpus decodes 910 models out of 3480 halves off the **same**
//! `postcard` door, so the door is not what separates them. What is different
//! is that the fuse target splits its input and takes two draws per seed. Why
//! that is worth 910 against 16 is not measured here and this file does not
//! guess at it.
//!
//! # The two control seeds
//!
//! `two_cubes_that_write` is two cubes of side 2, one at the origin and one at
//! `(1, 1, 1)` — 12 faces, 16 corners — which writes 10 435 bytes of Part 21.
//! `a_ball_whose_pole_the_writer_refuses` is one ball of radius 2, which is
//! refused `AWireThatStandsStillAtAPole`: a named gap deep inside
//! `solid_from_shell`, not one of the two doorstep refusals. One either side,
//! and the second one reaches past where the corpus had ever been.
//!
//! **These two seeds are `postcard` bytes, so they stop decoding if `Model`'s
//! serialized shape changes.** That is the alarm working, not a fragility: a
//! change to the door that silently reinterprets a stranger's saved file is
//! precisely what §44.5 is about, and the floors here go red for it.
//!
//! # The curved control seed
//!
//! Q42 (§129.5) added `1bf987c187b377606ff669042712474c52227800`: a round bar
//! of radius two, eight tall, `add_round_bar`'s own shape from
//! `a_solid_written_out_and_read_back_is_the_same_solid.rs`. Before it, no
//! seed here had ever decoded into a model with a curved face — the sixteen
//! empty decodes above have no faces at all, and of the two control seeds,
//! `two_cubes_that_write` is planar and `a_ball_whose_pole_the_writer_refuses`
//! is refused before it is ever measured. Mutating `postcard` bytes almost
//! never lands on a shape whose internal keys are even self-consistent (16 of
//! 1180), let alone one whose curved corners also certify onto their surface
//! the way `Model::validate` asks — every curved shape in this corpus, cube or
//! cylinder alike, was built and checked in by hand, none found by mutation.
//!
//! # Where the floors below come from
//!
//! Measured on this tree, 2026-09-13, by
//!
//! ```text
//! just t -p rcad-step \
//!     --test every_model_the_writer_fuzzer_kept_is_written_again --no-capture
//! ```
//!
//! which reported, with the two control seeds in:
//!
//! ```text
//! Reached { draws: 1182, decoded: 18, faces: 13, written: 1, refused: 17 }
//! ```
//!
//! in 19 ms. The `draws`, `decoded` and `refused` floors are three quarters of
//! that. `written` and `faces` come from one seed, so their floor is what that
//! seed brings — and if the corpus ever grows a second model with faces in it,
//! the floor stays honest by being too low rather than by being re-cut.
//!
//! # The read-back floors
//!
//! Measured 2026-09-25, once the harness read each written file back
//! (`docs/decisions.md` §129.5 item 3), by the same command:
//!
//! ```text
//! Reached { draws: 1182, decoded: 18, faces: 13, written: 1, refused: 17,
//!           written_but_unsound: 0, read_back: 1, did_not_measure: 0,
//!           solids_the_same: 2, came_back_the_same: 1 }
//! ```
//!
//! **`came_back_the_same` and `solids_the_same` are the only thing that sees
//! the comparison go.** Every other counter is untouched if the read-back is
//! deleted, and a comparison that is deleted passes every seed. Both come from
//! `two_cubes_that_write` alone, so their floor is what that one seed brings,
//! for the same reason as `written`'s.

#![forbid(unsafe_code)]

mod fuzzing;

use std::path::PathBuf;

use rcad_geom::Surface;
use rcad_topo::Model;

use fuzzing::writer::{Reached, one_model_decoded_and_written};

/// Where the checked-in corpus lives.
fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus/no_model_makes_the_writer_panic")
}

#[test]
fn every_seed_in_the_checked_in_corpus_is_written_without_panicking() {
    let mut reached = Reached::default();
    let listing = std::fs::read_dir(corpus()).expect("the corpus is checked in beside the target");
    for entry in listing {
        let path = entry.expect("a readable directory entry").path();
        if !path.is_file() {
            continue;
        }
        let bytes = std::fs::read(&path).expect("a readable seed");
        reached = reached.and(one_model_decoded_and_written(&bytes));
    }

    println!("{reached:?}");
    assert!(
        reached.draws >= 900,
        "only {} seeds are checked in, so the corpus has shrunk",
        reached.draws
    );
    assert!(
        reached.decoded >= 13,
        "only {} seeds decoded into a model, so the door has moved under the \
         corpus and almost nothing reaches the writer at all",
        reached.decoded
    );
    assert!(
        reached.refused >= 12,
        "only {} models were refused by name, so the writer's refusal paths \
         are no longer reached",
        reached.refused
    );
    assert!(
        reached.written >= 1,
        "no model was written out, so every seed in this corpus stops at a \
         refusal — which is what was true before `two_cubes_that_write` was \
         checked in"
    );
    assert!(
        reached.faces >= 12,
        "only {} faces across every model that decoded, so the corpus is back \
         to holding nothing but empty models",
        reached.faces
    );
    assert!(
        reached.came_back_the_same >= 1,
        "no written model was read back and found the same, so nothing in this \
         corpus reaches the comparison: {reached:?}"
    );
    assert!(
        reached.solids_the_same >= 2,
        "only {} solids were compared with the solid that went in, where \
         `two_cubes_that_write` alone brings two: {reached:?}",
        reached.solids_the_same
    );
}

/// The two named seeds, each asserted to be the thing it was checked in as.
#[test]
fn the_two_named_control_seeds_still_land_either_side_of_the_writer() {
    let writes = std::fs::read(corpus().join("two_cubes_that_write"))
        .expect("the control seed is checked in");
    let reached = one_model_decoded_and_written(&writes);
    assert_eq!(reached.written, 1, "{reached:?}");
    assert_eq!(
        reached.faces, 12,
        "two cubes have twelve faces: {reached:?}"
    );
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 2),
        "two cubes written out come back as the same two cubes: {reached:?}"
    );

    let pole = std::fs::read(corpus().join("a_ball_whose_pole_the_writer_refuses"))
        .expect("the control seed is checked in");
    let reached = one_model_decoded_and_written(&pole);
    assert_eq!(reached.decoded, 1, "{reached:?}");
    assert_eq!(
        reached.refused, 1,
        "a ball's pole is a wire step that stands still, which this writer \
         refuses by name: {reached:?}"
    );
}

/// A seed with a curved face — a cylinder, a cone, a sphere or a torus — is
/// checked in and reaches the volume/area/centroid comparison.
///
/// `Reached` has no counter of its own for this (it counts faces, not their
/// kind), so this test decodes each seed itself and asks its faces directly.
/// Without a curved seed checked in, this fails rather than silently shrinking
/// back to the two planar cubes `two_cubes_that_write` brings.
#[test]
fn a_curved_solid_in_the_corpus_reaches_the_comparison() {
    let listing = std::fs::read_dir(corpus()).expect("the corpus is checked in beside the target");
    let mut curved_seeds = 0;
    let mut curved_seeds_that_came_back_the_same = 0;
    for entry in listing {
        let path = entry.expect("a readable directory entry").path();
        if !path.is_file() {
            continue;
        }
        let bytes = std::fs::read(&path).expect("a readable seed");
        let Ok(model) = postcard::from_bytes::<Model>(&bytes) else {
            continue;
        };
        if !model.faces().any(|face| surface_is_curved(face.surface())) {
            continue;
        }
        curved_seeds += 1;
        if one_model_decoded_and_written(&bytes).came_back_the_same >= 1 {
            curved_seeds_that_came_back_the_same += 1;
        }
    }

    assert!(
        curved_seeds >= 1,
        "no seed in the corpus decodes into a model with a curved face at all"
    );
    assert!(
        curved_seeds_that_came_back_the_same >= 1,
        "{curved_seeds} seed(s) decode into a model with a curved face, but none of them \
         reached the volume/area/centroid comparison — a curved solid stops short of it again"
    );
}

/// A cylinder, a cone, a sphere or a torus — anything that is not a plane.
fn surface_is_curved(surface: Surface) -> bool {
    match surface {
        Surface::Plane(_) => false,
        Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_) => true,
        _ => true,
    }
}
