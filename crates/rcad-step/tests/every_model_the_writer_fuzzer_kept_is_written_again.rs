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
//! `a_ball_of_radius_two` is one ball of radius 2. Before Q730 it was refused
//! `AWireThatStandsStillAtAPole`: a named gap deep inside `solid_from_shell`,
//! not one of the two doorstep refusals, one either side of
//! `two_cubes_that_write`, and the one that reached furthest past where the
//! corpus had ever been. Since Q730 the writer leaves a ball's poles
//! unwritten, the reader puts them back where the seam turns, and the ball
//! comes back the same, so both control seeds now land on the written side.
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
//! `two_cubes_that_write` is planar and `a_ball_of_radius_two` was refused
//! before it was ever measured. Mutating `postcard` bytes almost
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
//!     -E 'binary(merged) & test(every_model_the_writer_fuzzer_kept_is_written_again)' \
//!     --no-capture
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
//!
//! # The seven shape seeds
//!
//! Q58 (`docs/decisions.md` §241) gave the target a mutator that keeps every
//! input a model, and checked in seven models beside it: a round bar, the same
//! bar moved by whole units, the same bar turned a quarter about `x`, a ball, a
//! frustum, a pointed cone and a quarter bend of pipe. The same command then
//! reported
//!
//! ```text
//! Reached { draws: 1190, decoded: 26, faces: 34, written: 4, refused: 22,
//!           written_but_unsound: 0, read_back: 4, did_not_measure: 0,
//!           solids_the_same: 5, came_back_the_same: 4 }
//! ```
//!
//! Only the bar and the moved bar were written then. The other five were
//! refused by name — the ball at its pole, and the frustum, the cone, the bend
//! and the turned bar at a curve between two faces the writer cannot name yet —
//! and §241 lists what that costs the fuzzer. Since Q730 the ball is written
//! and comes back the same, since Q405 the turned bar, since Q662 the frustum
//! and the pointed cone (`docs/decisions.md` §366), and since Q407 the quarter
//! bend, so all seven come back the same.

#![forbid(unsafe_code)]

mod fuzzing;

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use rcad_geom::Deflection;
use rcad_geom::Surface;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::Model;

use fuzzing::writer::{Reached, one_model_decoded_and_written, still_a_model};

/// Mutator seeds checked, seed by seed against every corpus entry with a
/// `byte_mutate` stand-in that panics, to never draw `Draw::below(8)` as 0, 1
/// or 2 on the first draw — the one branch that calls `byte_mutate` rather than
/// drawing only from the seed (§241.1). Found by that check, not chosen by hand.
///
/// Was `[0, 3, 7, 16, 34]` before Q1011 added a seventh shape to
/// `a_drawn_solid`: that shape draws nothing of its own, but the extra arm
/// shifts how many numbers `a_drawn_solid` draws for some seeds, which moves
/// where the later `below(8)` draw lands for any corpus entry that does not
/// decode (`still_a_model` draws a fresh model, and so a solid, before that
/// draw). Re-run after the shape was added: `0` no longer avoids the
/// byte-mutation branch on every entry, so it dropped out; `16` and `34` also
/// dropped, and `8`, `11` and `12` took their place, alongside `3` and `7`,
/// which still avoid it. Now `[3, 7, 8, 11, 12]`.
const MUTATOR_SEEDS: [u32; 5] = [3, 7, 8, 11, 12];

/// Every checked-in seed, mutated by each of [`MUTATOR_SEEDS`], still decodes
/// into a `Model`.
///
/// `byte_mutate` here panics if it is ever called. `still_a_model` only calls
/// it from the branch [`MUTATOR_SEEDS`] was chosen to avoid, so this also
/// proves those seeds take the same branch — one more solid, one moved, or a
/// fresh model — on every run, decoded input or not.
#[test]
fn every_seed_mutated_by_a_fixed_seed_still_decodes_into_a_model() {
    fn byte_mutate_must_not_run(_: &mut [u8], _: usize, _: usize) -> usize {
        panic!(
            "still_a_model reached libFuzzer's own byte mutation, which \
             MUTATOR_SEEDS is chosen to avoid"
        );
    }

    for (name, bytes) in every_seed() {
        for &seed in &MUTATOR_SEEDS {
            let mut buffer = [0u8; 4096];
            let max_size = buffer.len();
            buffer[..bytes.len()].copy_from_slice(&bytes);
            let written = still_a_model(
                &mut buffer,
                bytes.len(),
                max_size,
                seed,
                byte_mutate_must_not_run,
            );
            postcard::from_bytes::<Model>(&buffer[..written]).unwrap_or_else(|why| {
                panic!(
                    "mutating {name} with seed {seed} gave {written} byte(s) that do \
                     not decode into a model: {why}"
                )
            });
        }
    }
}

/// Mutator seeds checked, seed by seed against every corpus entry, that draw
/// `Draw::below(8)` as 0, 1 or 2 on every entry — the byte-mutation branch
/// [`MUTATOR_SEEDS`] is chosen to miss. Found by running the same check the
/// other way: for each of these five, all 1190 corpus entries take that
/// branch, decoded or not.
///
/// Re-run after Q1011's seventh shape, for the same reason [`MUTATOR_SEEDS`]
/// was — the shape draws nothing of its own, but the extra arm can still
/// shift how many numbers an undecoded entry's `a_drawn_solid` draws before
/// the later `below(8)` draw. Checked unchanged at `[4, 6, 10, 33, 51]`: for
/// every one of these five, every corpus entry still lands `below(8)` on 0, 1
/// or 2, decoded or not.
const BYTE_MUTATION_SEEDS: [u32; 5] = [4, 6, 10, 33, 51];

/// How many times [`one_bit_flipped`] has run since it was last reset.
///
/// `still_a_model` only calls `byte_mutate` from inside the byte-mutation
/// branch, so resetting this before a call and reading it after tells the
/// test whether that call actually reached the branch — not just that
/// whatever `still_a_model` produced happens to decode, which every branch
/// guarantees on its own and so proves nothing about which one ran.
static BYTE_MUTATION_BRANCH_RUNS: AtomicUsize = AtomicUsize::new(0);

/// A deterministic stand-in for libFuzzer's own byte mutation: flip one fixed
/// bit and touch nothing else, so a replayed seed reaches the same bytes every
/// time.
///
/// `still_a_model` cannot be handed the real mutation here — it draws from
/// libFuzzer's own generator, not from `seed` (`docs/decisions.md` §241.1) —
/// so this is what the replay passes instead, on the seeds chosen to reach the
/// branch that would call it.
fn one_bit_flipped(data: &mut [u8], size: usize, _max_size: usize) -> usize {
    BYTE_MUTATION_BRANCH_RUNS.fetch_add(1, Ordering::Relaxed);
    if size == 0 {
        return size;
    }
    data[size / 2] ^= 0x01;
    size
}

/// Every checked-in seed, mutated by each of [`BYTE_MUTATION_SEEDS`], reaches
/// the byte-mutation branch and still decodes into a `Model`.
///
/// [`every_seed_mutated_by_a_fixed_seed_still_decodes_into_a_model`] replays
/// `still_a_model` on seeds chosen to skip its byte-mutation branch; this
/// replays it on seeds chosen to take that branch every time, so
/// `one_byte_changed_and_still_a_model`'s own decode check is what keeps this
/// test passing. Every other branch also decodes on its own, so the run count
/// is what proves [`BYTE_MUTATION_SEEDS`] still does what its doc comment
/// says rather than the assertion below passing for a reason that has nothing
/// to do with byte mutation.
#[test]
fn every_seed_mutated_by_a_seed_that_flips_a_byte_still_decodes_into_a_model() {
    for (name, bytes) in every_seed() {
        for &seed in &BYTE_MUTATION_SEEDS {
            let mut buffer = [0u8; 4096];
            let max_size = buffer.len();
            buffer[..bytes.len()].copy_from_slice(&bytes);
            BYTE_MUTATION_BRANCH_RUNS.store(0, Ordering::Relaxed);
            let written = still_a_model(&mut buffer, bytes.len(), max_size, seed, one_bit_flipped);
            assert!(
                BYTE_MUTATION_BRANCH_RUNS.load(Ordering::Relaxed) > 0,
                "mutating {name} with seed {seed} never reached the byte-mutation \
                 branch, so BYTE_MUTATION_SEEDS no longer takes it and this test is \
                 not checking what its name says"
            );
            postcard::from_bytes::<Model>(&buffer[..written]).unwrap_or_else(|why| {
                panic!(
                    "mutating {name} with seed {seed} gave {written} byte(s) that do \
                     not decode into a model: {why}"
                )
            });
        }
    }
}

/// Where the middle of a quarter of a torus of ring 6 and tube 2 lies, along
/// each of the two axes it runs between: the whole torus's middle circle,
/// `6 + 2² / (4 · 6)`, pulled in by a quarter arc's `sin(π/4) / (π/4)`, then
/// split along the diagonal.
const QUARTER_BEND_MIDDLE: f64 = 3.925_7;

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

/// The two named control seeds, each asserted to be the thing it was checked
/// in as: `two_cubes_that_write` writes two cubes, and `a_ball_of_radius_two`
/// writes a ball whose poles the writer leaves unwritten and the reader puts
/// back, so both come back the same.
#[test]
fn the_two_named_control_seeds_are_written_and_come_back() {
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

    let pole = std::fs::read(corpus().join("a_ball_of_radius_two"))
        .expect("the control seed is checked in");
    let reached = one_model_decoded_and_written(&pole);
    assert_eq!(reached.decoded, 1, "{reached:?}");
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "a ball's poles are left unwritten and put back by the reader, so the \
         ball comes back the same: {reached:?}"
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

/// At least ten seeds decode into a model with a face in it.
///
/// Before Q58 three did — `two_cubes_that_write`, the ball and the round bar
/// — and the other sixteen that decoded were empty. The fuzz target no longer
/// needs a seed to decode, because its mutator turns bytes that do not into a
/// fresh model; this file does, because it replays the corpus and nothing else.
/// So the corpus is all of the writer `just gate` ever sees, and this is its
/// floor: the three there were, and the seven shapes Q58 checked in beside
/// them.
#[test]
fn at_least_ten_seeds_decode_into_a_model_with_a_face() {
    let with_a_face: Vec<String> = every_seed()
        .into_iter()
        .filter(|(_, bytes)| one_model_decoded_and_written(bytes).faces > 0)
        .map(|(name, _)| name)
        .collect();
    assert!(
        with_a_face.len() >= 10,
        "only {} seeds decode into a model with a face — {with_a_face:?} — so the \
         corpus is back to reaching the writer with almost nothing to write",
        with_a_face.len()
    );
}

/// The seven shapes Q58 checked in, each the shape its name says, and each
/// either written and read back the same or refused by name.
///
/// They were drawn by the target's own `a_frustum`, `a_pointed_cone` and
/// `a_quarter_bend` and by `add_round_bar` and `add_ball`, and none of them is
/// one the comparison panics on: the turned bar that does is kept out of the
/// corpus and reported in `docs/decisions.md` §241 instead.
#[test]
fn every_shape_seed_is_the_shape_it_is_named_for() {
    struct Shape {
        seed: &'static str,
        faces: &'static [&'static str],
        centroid: [f64; 3],
        comes_back: bool,
    }
    let shapes = [
        Shape {
            seed: "a_round_bar_of_radius_three",
            faces: &["cylinder", "plane", "plane"],
            centroid: [0.0, 0.0, 2.5],
            comes_back: true,
        },
        Shape {
            seed: "a_round_bar_moved_by_whole_units",
            faces: &["cylinder", "plane", "plane"],
            centroid: [3.0, -5.0, 9.5],
            comes_back: true,
        },
        Shape {
            seed: "a_round_bar_turned_a_quarter_about_x",
            faces: &["cylinder", "plane", "plane"],
            centroid: [0.0, -2.5, 0.0],
            comes_back: true,
        },
        Shape {
            seed: "a_ball_of_radius_three",
            faces: &["sphere"],
            centroid: [0.0, 0.0, 0.0],
            comes_back: true,
        },
        Shape {
            seed: "a_frustum_of_a_half_slope_cone",
            faces: &["cone", "plane", "plane"],
            centroid: [0.0, 0.0, 60.0 / 13.0],
            comes_back: true,
        },
        Shape {
            seed: "a_pointed_cone",
            faces: &["cone", "plane"],
            centroid: [0.0, 0.0, 4.5],
            comes_back: true,
        },
        Shape {
            seed: "a_quarter_bend_of_pipe",
            faces: &["plane", "plane", "torus"],
            centroid: [QUARTER_BEND_MIDDLE, QUARTER_BEND_MIDDLE, 0.0],
            comes_back: true,
        },
    ];
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    for shape in shapes {
        let bytes = std::fs::read(corpus().join(shape.seed)).expect("the shape seed is checked in");
        let model = postcard::from_bytes::<Model>(&bytes).expect("a shape seed decodes");
        let solid = model.solids().next().expect("a shape seed holds a solid");
        assert_eq!(model.solid_count(), 1, "{}", shape.seed);
        let mut faces: Vec<&str> = solid
            .faces()
            .map(|face| surface_kind(face.surface()))
            .collect();
        faces.sort_unstable();
        assert_eq!(faces, shape.faces, "{}", shape.seed);

        let centroid = solid
            .measure(Deflection::for_scale(model.scale()), &policy, &mut log)
            .expect("every shape seed measures")
            .centroid();
        let off = [
            centroid.x() - shape.centroid[0],
            centroid.y() - shape.centroid[1],
            centroid.z() - shape.centroid[2],
        ];
        assert!(
            off.iter().all(|coordinate| coordinate.abs() < 0.05),
            "{} has its middle at {centroid:?}, not near {:?}",
            shape.seed,
            shape.centroid
        );

        let reached = one_model_decoded_and_written(&bytes);
        let outcome = (reached.came_back_the_same, reached.refused);
        let expected = if shape.comes_back { (1, 0) } else { (0, 1) };
        assert_eq!(outcome, expected, "{}: {reached:?}", shape.seed);
    }
}

/// The quarter bend seed, written and read back, measures the same volume to
/// the bit, with the mesh taken at the read-back model's scale as the target
/// takes it.
///
/// Q70's probe of 2026-09-28 saw it come back `118.38670869374307` against
/// `118.38685376813798` built, with its wall's loop starting one coedge later.
/// On a base with the bends built the way a file writes them (§361) and the
/// cap circles and the seam written as Q407 writes them, neither is so: the
/// loop starts where it did, and this prints the one volume twice.
#[test]
fn the_quarter_bend_seed_measures_the_same_read_back_from_its_own_file() {
    let bytes =
        std::fs::read(corpus().join("a_quarter_bend_of_pipe")).expect("the seed is checked in");
    let model = postcard::from_bytes::<Model>(&bytes).expect("a shape seed decodes");
    let text = rcad_step::write(&model, "a quarter bend").expect("the bend is written");
    let file = rcad_step::StepFile::read(text.as_bytes()).expect("this writer's text parses");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let back = rcad_step::Import::read(&file, &policy, &mut log).expect("the bend is read back");
    let deflection = Deflection::for_scale(back.model().scale());
    let [built, read_back] = [&model, back.model()].map(|holding| {
        holding
            .solids()
            .next()
            .expect("one solid")
            .measure(deflection, &policy, &mut log)
            .expect("the bend measures")
            .volume()
    });
    println!("built {built:?}, read back {read_back:?}");
    assert_eq!(built.to_bits(), read_back.to_bits());
}

/// Q166's seed (`docs/decisions.md` §313), the 2026-09-26 crash file
/// `crash-001e0849a7f44ed74619db24f364721fa6dd90a5`: a round bar of radius
/// five and length eleven, turned about `x` and moved to `(−13, 1, 8)`. Its
/// cap at `18.060941075367033` came back from its own file at
/// `18.06094107536703`, because the writer wrote the foot of the
/// perpendicular and the reader dotted it back an ulp short. It comes back
/// the same now.
#[test]
fn a_turned_round_bar_comes_back_with_its_caps_on_the_same_planes() {
    let bytes = std::fs::read(
        corpus().join("a_turned_round_bar_whose_cap_the_foot_of_its_perpendicular_misses"),
    )
    .expect("the seed is checked in");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}"
    );
}

/// How often a model this target's own mutator draws is refused, and by what
/// name: four hundred draws, each from a checked-in seed or an earlier draw,
/// the way a fuzzing run grows its corpus. Q70 ran the same loop three
/// thousand times (`docs/decisions.md` §366).
///
/// Every model written is read back and compared, so a cap this writer has
/// learned to name and the reader does not take back panics here. Those
/// panics are counted, except one on an edge's run, which fails: seven turned
/// balls whose seam came back a different run were Q822, and none is left.
/// Nor may a ball's seam be refused, which is what two of the seven became
/// before the writer stepped two coordinates of the seam's centre together,
/// nor a sound model's flat cap on a cone (Q662) or on a bend (Q407), nor a
/// ball at its pole: 38 draws were (§366.4), and none has been since the
/// writer writes a whole ball turned off `z` cut open on the `z` line
/// (`docs/decisions.md` §404).
#[test]
fn how_often_a_model_the_mutator_draws_is_refused() {
    fn one_byte_bumped(data: &mut [u8], size: usize, _max_size: usize) -> usize {
        if size == 0 {
            return 0;
        }
        let at = (data
            .iter()
            .take(size)
            .map(|&byte| usize::from(byte))
            .sum::<usize>()
            * 31
            + size)
            % size;
        data[at] = data[at].wrapping_add(1 + u8::try_from(size % 7).expect("below seven"));
        size
    }

    let mut pool: Vec<Vec<u8>> = std::fs::read_dir(corpus())
        .expect("the corpus is checked in beside the target")
        .map(|entry| std::fs::read(entry.expect("an entry").path()).expect("a seed"))
        .collect();
    pool.sort();
    let mut refusals: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut comparisons_that_panicked: std::collections::BTreeMap<String, (usize, String)> =
        std::collections::BTreeMap::new();
    std::panic::set_hook(Box::new(|_| {}));
    let (draws, mut refused, mut a_sound_cone_cap_refused) = (400_u32, 0, 0);
    let mut state: u64 = 0x5170;
    for draw in 0..draws {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let pick = usize::try_from(state >> 33).expect("31 bits fit") % pool.len();
        let mut data = pool[pick].clone();
        let size = data.len();
        data.resize(4096, 0);
        let size = still_a_model(&mut data, size, 4096, draw, one_byte_bumped);
        data.truncate(size);
        pool.push(data.clone());
        let model = postcard::from_bytes::<Model>(&data).expect("the mutator keeps a model");
        match rcad_step::write(&model, "fuzz") {
            Ok(_) => {
                let compared = std::panic::catch_unwind(|| one_model_decoded_and_written(&data));
                if let Err(why) = compared {
                    let said = why
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| why.downcast_ref::<&str>().map(|said| (*said).to_owned()))
                        .unwrap_or_default();
                    let whole: String = said.chars().take(400).collect();
                    let said = said.lines().next().unwrap_or_default();
                    let said = said.trim_start_matches("assertion `left == right` failed: ");
                    let said = said.split(':').next().unwrap_or_default().to_owned();
                    let seen = comparisons_that_panicked.entry(said).or_insert((0, whole));
                    seen.0 += 1;
                }
            }
            Err(why) => {
                refused += 1;
                let named = format!("{why:?}");
                if named.contains("\"plane\", second: \"cone\"")
                    || named.contains("\"cone\", second: \"plane\"")
                    || named.contains("\"plane\", second: \"ring\"")
                    || named.contains("\"ring\", second: \"plane\"")
                {
                    let sound = model
                        .validate(&Policy::certified(), &mut ProvenanceLog::new())
                        .is_ok();
                    a_sound_cone_cap_refused += usize::from(sound);
                }
                *refusals.entry(named).or_default() += 1;
            }
        }
    }
    let _ = std::panic::take_hook();
    println!("{refused} of {draws} drawn models refused");
    for (named, count) in &refusals {
        println!("{count:6}  {named}");
    }
    for (said, (count, first)) in &comparisons_that_panicked {
        println!("{count:6}  written, and the comparison panicked: {said}");
        println!("        the first: {first}");
    }
    println!(
        "{a_sound_cone_cap_refused} sound models refused where a flat face meets a cone or a ring"
    );
    assert_eq!(
        a_sound_cone_cap_refused, 0,
        "a sound model was refused where its flat cap meets its cone or its ring"
    );
    let runs_that_came_back_different: usize = comparisons_that_panicked
        .iter()
        .filter(|(said, _)| said.contains("with a different edge run"))
        .map(|(_, (count, _))| count)
        .sum();
    assert_eq!(
        runs_that_came_back_different, 0,
        "a sound solid's edge came back from its own file as a different run"
    );
    assert!(
        !refusals
            .keys()
            .any(|named| named.contains("first: \"ball\", second: \"ball\"")),
        "a ball's seam was refused: {refusals:?}"
    );
    assert!(
        !refusals.contains_key("AWireThatStandsStillAtAPole"),
        "a ball was refused at its pole: {refusals:?}"
    );
}

/// Every file in the checked-in corpus, by name.
fn every_seed() -> Vec<(String, Vec<u8>)> {
    let listing = std::fs::read_dir(corpus()).expect("the corpus is checked in beside the target");
    let mut seeds = Vec::new();
    for entry in listing {
        let path = entry.expect("a readable directory entry").path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .expect("a file has a name")
            .to_string_lossy()
            .into_owned();
        seeds.push((name, std::fs::read(&path).expect("a readable seed")));
    }
    seeds
}

/// What kind of surface a face lies on, in a word.
fn surface_kind(surface: Surface) -> &'static str {
    match surface {
        Surface::Plane(_) => "plane",
        Surface::Cylinder(_) => "cylinder",
        Surface::Cone(_) => "cone",
        Surface::Sphere(_) => "sphere",
        Surface::Torus(_) => "torus",
        _ => "a kind this test does not know",
    }
}

/// A cylinder, a cone, a sphere or a torus — anything that is not a plane.
fn surface_is_curved(surface: Surface) -> bool {
    match surface {
        Surface::Plane(_) => false,
        Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_) => true,
        _ => true,
    }
}
