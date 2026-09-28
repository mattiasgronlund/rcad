//! Arbitrary bytes into `rcad_step::write`, which must never panic.
//!
//! CLAUDE.md's rule that a caller's bytes may not panic, and the reader's own
//! fuzz target is why this one exists: that target found a real crash in its
//! first five minutes, and the writer had no target at all until this one.
//!
//! **`rcad_step::write` takes a `&Model`, not bytes, so this has to make one
//! first.** `Model` implements `serde`'s `Deserialize`
//! (`crates/rcad-topo/src/serialize.rs`) — that is precisely the route a
//! stranger's bytes take into a `Model`, whether from a saved file or from the
//! fuzzer — so arbitrary bytes go through `postcard::from_bytes` and whatever
//! comes out, valid or not, goes straight into the writer. A `Model` a fuzzer
//! built is under no obligation to be one `Model::validate` would let through,
//! and the writer owes an `Err` either way, never a crash.
//!
//! **A model that does come out is checked against what the door promised**,
//! not just handed on. `Deserialize` refuses a corner that is at no point and a
//! slop that is not a length (`docs/decisions.md` §44.5), and walking past a
//! model whose corner is `NaN` would leave that promise untested by the thing
//! best placed to break it.
//!
//! **What the corpus actually reaches is the reason [`Reached`] counts faces.**
//! Sixteen of 1180 checked-in seeds decoded at all and every one of the sixteen
//! was empty — at most one solid, no faces, no vertices — so the door check
//! walked three empty iterators 1180 times and the writer never got past
//! `TheModelHasNoSolid` or `ASolidWithNoShell`. `two_cubes_that_write` and
//! `a_ball_whose_pole_the_writer_refuses` are the two seeds that fix that, one
//! either side.
//!
//! **A file that was written is read back and compared with the model that went
//! in** (`docs/decisions.md` §129.5 item 3). Not panicking was all this target
//! asked before, and the writer is the highest-throughput target there is. The
//! written text parses, every reference in it points somewhere, `Import::read`
//! turns it back into a model that validates, and the same number of solids
//! comes back — the same door `round_trip_one` in
//! `a_real_file_written_out_and_read_back_is_the_same_solid.rs` checks. That
//! test's last leg, a `Rule::Difference` against a third solid, is left out: it
//! is a fuse, and a fuse that panics on a sound model here would be a fuse
//! defect reported by the writer's target.
//!
//! **What is compared, and why it stops short of the measurement for some
//! solids** (`docs/decisions.md` §249.6, amending §129.5 item 3). Every
//! solid's located numbers are checked bit for bit: each face's surface, and
//! for every coedge its two corners and its edge's run, in walk order. These
//! are the numbers the writer writes and the reader reads, and §249.2 found
//! every one of them come back the same bits for a turned round bar. Volume,
//! surface area and centroid are then also checked bit for bit, but only for a
//! solid that sits wholly on planes or that carries no placement at all. A
//! curved solid placed with a turn is meshed in the frame it is stored in on
//! the way out and in the frame the reader baked it into on the way back — two
//! different roundings of the same true surface — so its measurement can come
//! back a few ulps off with nothing wrong in either the writer or the reader
//! (§249.5, a limit, not a defect). `two_turns_that_write_the_same_placement.rs`
//! pins that limit down: two turns that turn the three axes to the same nine
//! numbers, so that STEP would write them as one placement, still leave a bar
//! measuring differently.
//!
//! What the source explains is counted and not failed. A model that does not
//! pass `Model::validate` was never a solid, so a file of it owes nothing on
//! the way back; a solid that does not measure has no answer to compare.
//! Everything else that differs panics, because it is a sound model this
//! writer and this reader disagree about.
//!
//! **A sound model is one whose every solid also passes
//! `Model::validate_solid`**, because that is the question the reader asks.
//! It builds each solid through `SolidBuilder::build`, which judges the solid
//! in a model of its own. `validate` can pass a model whose solid is refused
//! alone: a corner may sit off its plane by up to about `1e-9` times the
//! largest coordinate, and a far neighbour makes that room wider
//! (`docs/decisions.md` §242.6). Twelve of the 2026-09-26 crash files were
//! that, and nothing else (§326).

// Shared between a test binary and a fuzz target, and each uses a subset.
#![allow(dead_code)]

// `rcad-topo` owns the door, so the check on what it promised lives there and
// this reaches across to it. Both this file and `fuse.rs` carried a copy of it
// before, which is two places to update and one to forget.
#[path = "../../../rcad-topo/tests/fuzzing/the_door.rs"]
mod the_door;

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane,
    Point, Scale, Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey, SolidView, WireView};

use the_door::every_number_the_door_promised_is_real;

/// How far one draw of bytes got.
#[derive(Clone, Copy, Debug, Default)]
pub struct Reached {
    /// Byte strings handed to the decoder.
    pub draws: usize,
    /// Byte strings that decoded into a model.
    pub decoded: usize,
    /// Faces across every model that decoded.
    pub faces: usize,
    /// Models this writer wrote out as a Part 21 file.
    pub written: usize,
    /// Models this writer refused by name.
    pub refused: usize,
    /// Written models that do not pass `Model::validate`, so there is no solid
    /// to compare the file against.
    pub written_but_unsound: usize,
    /// Written models that pass `Model::validate` but hold a solid that
    /// `Model::validate_solid` refuses. The reader builds each solid through
    /// `SolidBuilder::build`, which judges it alone, and a neighbour can widen
    /// the room `validate` gives a corner (`docs/decisions.md` §242.6).
    pub written_but_a_solid_unsound_alone: usize,
    /// Written files the reader turned back into a model.
    pub read_back: usize,
    /// Read-back models not compared because a solid of the model that went in
    /// does not measure.
    pub did_not_measure: usize,
    /// Solids whose located numbers were compared bit for bit with the solid
    /// that went in, and matched — plus, for a solid wholly flat or at no
    /// placement, whose volume, surface area and centroid also matched bit
    /// for bit (§249.6).
    pub solids_the_same: usize,
    /// Models written, read back, and the same in every solid.
    pub came_back_the_same: usize,
}

impl Reached {
    /// What two draws reached between them.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        Self {
            draws: self.draws + other.draws,
            decoded: self.decoded + other.decoded,
            faces: self.faces + other.faces,
            written: self.written + other.written,
            refused: self.refused + other.refused,
            written_but_unsound: self.written_but_unsound + other.written_but_unsound,
            written_but_a_solid_unsound_alone: self.written_but_a_solid_unsound_alone
                + other.written_but_a_solid_unsound_alone,
            read_back: self.read_back + other.read_back,
            did_not_measure: self.did_not_measure + other.did_not_measure,
            solids_the_same: self.solids_the_same + other.solids_the_same,
            came_back_the_same: self.came_back_the_same + other.came_back_the_same,
        }
    }
}

/// **One draw of bytes, decoded the way a stranger's saved file is, written out,
/// and read back.**
///
/// The fuzz target and the suite's replay both call this and nothing narrower.
///
/// # Panics
///
/// If a model came through `Deserialize` carrying a number the door promised it
/// would refuse, or if a sound model was written as a file that does not come
/// back as the same model.
#[must_use]
pub fn one_model_decoded_and_written(bytes: &[u8]) -> Reached {
    let mut reached = Reached {
        draws: 1,
        ..Reached::default()
    };
    let Ok(model) = postcard::from_bytes::<Model>(bytes) else {
        return reached;
    };
    reached.decoded = 1;
    reached.faces = model.faces().count();
    every_number_the_door_promised_is_real(&model);
    match rcad_step::write(&model, "fuzz") {
        Ok(text) => {
            reached.written = 1;
            reached = reached.and(read_back_and_compared(&model, &text));
        }
        Err(_) => reached.refused = 1,
    }
    reached
}

/// Read `text` back and compare it with `source`, the model it was written
/// from — `round_trip_one`'s checks, in
/// `a_real_file_written_out_and_read_back_is_the_same_solid.rs`.
fn read_back_and_compared(source: &Model, text: &str) -> Reached {
    let mut reached = Reached::default();
    let file = StepFile::read(text.as_bytes()).unwrap_or_else(|why| {
        panic!("this writer wrote a file its own reader cannot parse: {why}")
    });
    let broken = file.references_that_point_nowhere();
    assert!(
        broken.is_empty(),
        "this writer wrote {} reference(s) to an entity it never wrote",
        broken.len()
    );

    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    if source.validate(&policy, &mut log).is_err() {
        reached.written_but_unsound = 1;
        return reached;
    }
    if source.solids().any(|solid| {
        source
            .validate_solid(solid.key(), &policy, &mut log)
            .is_err()
    }) {
        reached.written_but_a_solid_unsound_alone = 1;
        return reached;
    }
    let import = Import::read(&file, &policy, &mut log).unwrap_or_else(|why| {
        panic!("a sound model was written as a file the reader refuses: {why}")
    });
    let there = import.model();
    reached.read_back = 1;
    if let Err(why) = there.validate(&policy, &mut log) {
        panic!("a sound model came back from its own file as one that is not: {why}");
    }

    let source_solids: Vec<SolidKey> = source.solids().map(|solid| solid.key()).collect();
    let there_solids: Vec<SolidKey> = there.solids().map(|solid| solid.key()).collect();
    assert_eq!(
        source_solids.len(),
        there_solids.len(),
        "a sound model's solids changed in number on the way through its own file"
    );

    // The model that went in carries whatever scale its bytes named, and the one
    // read back carries the scale of its own vertices (`Import`). The mesh is
    // taken from the second so that a drawn scale of 1e-300 cannot ask for a
    // mesh finer than the solid, and it is the same mesh on both sides.
    let deflection = Deflection::for_scale(there.scale());
    for (&here_key, &back_key) in source_solids.iter().zip(&there_solids) {
        let here_solid = source.solid(here_key).expect("this model made the key");
        let back_solid = there.solid(back_key).expect("this model made the key");
        assert_located_numbers_the_same(here_solid, back_solid);

        let Ok(here) = here_solid.measure(deflection, &policy, &mut log) else {
            reached.did_not_measure = 1;
            return reached;
        };
        let back = back_solid
            .measure(deflection, &policy, &mut log)
            .unwrap_or_else(|why| {
                panic!("a solid that measured does not measure after its own file: {why}")
            });
        if here_solid.is_wholly_flat() || solid_has_no_placement(here_solid) {
            assert!(
                here.volume() == back.volume() && here.surface_area() == back.surface_area(),
                "a sound solid came back from its own file with a different volume or area: \
                 {here:?} went in, {back:?} came back"
            );
            assert!(
                here.centroid() == back.centroid(),
                "a sound solid came back from its own file with a different centroid: \
                 {here:?} went in, {back:?} came back"
            );
        }
        reached.solids_the_same += 1;
    }
    reached.came_back_the_same = 1;
    reached
}

/// Every located number of `here` and `back`, walked the way §249.1 walked it
/// by hand: each face's surface, and for every coedge its two corners and its
/// edge's run, in walk order.
///
/// Compared as `{:?}` text rather than with `==`, because derived float
/// equality treats `0.0` and `-0.0` as one number and this has to mean the
/// same bits (§249.1). A mismatch here is never a limit: the writer writes
/// every one of these numbers unchanged and the reader parses them back
/// exactly (§249.2), so a difference is a defect in one of the two, not a
/// frame the file cannot name.
///
/// A plane is the one surface whose numbers are not written as they are
/// stored. The writer picks the point it writes so that the reader's offset
/// comes back the same bits (§313). The normal it writes as stored, and the
/// reader scales it to length one again, so a normal a turn left an ulp off
/// length one still comes back moved: 1 509 of the 2026-09-26 crash files
/// stop here for that. It is a gap in the reader, not a limit — the file
/// carries the bits — and this check keeps failing on it until `rcad-geom`
/// gives the reader a way to keep them (§313.3).
///
/// # Panics
///
/// If a face, a coedge, a corner or a run differs between `here` and `back`.
fn assert_located_numbers_the_same(here: SolidView, back: SolidView) {
    let here_faces: Vec<_> = here.faces().collect();
    let back_faces: Vec<_> = back.faces().collect();
    assert_eq!(
        here_faces.len(),
        back_faces.len(),
        "a sound solid came back from its own file with a different number of faces"
    );
    for (here_face, back_face) in here_faces.iter().zip(&back_faces) {
        assert_eq!(
            format!("{:?}", here_face.surface()),
            format!("{:?}", back_face.surface()),
            "a sound solid's face came back from its own file with a different surface"
        );
        let here_coedges: Vec<_> = here_face.wires().flat_map(WireView::coedges).collect();
        let back_coedges: Vec<_> = back_face.wires().flat_map(WireView::coedges).collect();
        assert_eq!(
            here_coedges.len(),
            back_coedges.len(),
            "a sound solid's face came back from its own file with a different number of coedges"
        );
        for (here_coedge, back_coedge) in here_coedges.iter().zip(&back_coedges) {
            assert_eq!(
                format!("{:?}", here_coedge.start().point()),
                format!("{:?}", back_coedge.start().point()),
                "a sound solid's coedge came back from its own file starting at a different corner"
            );
            assert_eq!(
                format!("{:?}", here_coedge.end().point()),
                format!("{:?}", back_coedge.end().point()),
                "a sound solid's coedge came back from its own file ending at a different corner"
            );
            assert_eq!(
                format!("{:?}", here_coedge.edge().run()),
                format!("{:?}", back_coedge.edge().run()),
                "a sound solid's coedge came back from its own file with a different edge run"
            );
        }
    }
}

/// Whether `solid` carries no placement at all: every shell it uses sits at
/// exactly [`Isometry::identity`].
///
/// A move with no turn is still a placement here — §249.4 item 2 measured one
/// ulp off a bar moved by `(0.1, 0.2, 0.3)` and no turn, so only the identity
/// is exempt from the measurement check above.
///
/// # Why shells are enough, and a face's own location is not also checked
///
/// A face use carries a location of its own (`rcad_topo`'s `FaceUse::loc`),
/// composed with its shell's on the way to a face view — so in the type this
/// function does not look far enough. It does not have to: every solid this
/// function ever sees came from `postcard::from_bytes::<Model>`
/// (`one_model_decoded_and_written`, this file), and that is `Model`'s
/// `Deserialize` impl (`crates/rcad-topo/src/serialize.rs:167`), which calls
/// `check_shape_is_possible` before handing back a `Model`. That check refuses
/// the bytes outright — `Invalid::AFaceUseCarriesAPlacement`
/// (`crates/rcad-topo/src/validate.rs:1060-1074`) — if any face use's own
/// location is not exactly the identity. `crates/rcad-topo/src/model.rs`'s
/// `a_file_that_shifts_one_face_off_its_shell_never_becomes_a_model` pins that
/// down through this same decode call. So a face use's own location is always
/// the identity here, on every solid this function is asked about, and a
/// shell's location is the only placement such a solid can carry.
fn solid_has_no_placement(solid: SolidView) -> bool {
    solid
        .shells()
        .all(|shell| shell.location() == Isometry::identity())
}

/// The bar whose measurement §249.1 saw come back a few ulps off: a curved
/// solid turned, so its measurement is left out, and nothing else is.
///
/// Built without new `use` lines: this file is also compiled into the fuzz
/// target, where `#[test]` functions are dropped and an import only they used
/// would be unused.
#[test]
fn a_round_bar_turned_an_eighth_about_z_comes_back_from_its_own_file() {
    let eighth_turn = Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_4)
        .expect("a finite angle about a unit axis");
    let bytes = postcard::to_allocvec(&one_solid_placed_by(a_round_bar, &eighth_turn))
        .expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}"
    );
}

/// A cube of side two turned a tenth of a radian about `x` and moved by whole
/// units. Three of its faces sit on planes whose offset the foot of the
/// perpendicular, the point the writer used to write, reads back an ulp off,
/// and all six keep their normals through the file (§313).
#[test]
fn a_cube_turned_a_tenth_of_a_radian_comes_back_on_the_same_planes() {
    let turned = Isometry::translation(Vector::new(-13.0, 1.0, 8.0))
        .expect("a finite offset")
        .compose(&Isometry::rotation(&Axis::x(), 0.1).expect("a finite angle about a unit axis"));
    let bytes = postcard::to_allocvec(&one_solid_placed_by(a_cube, &turned))
        .expect("a model always encodes");
    let reached = one_model_decoded_and_written(&bytes);
    assert_eq!(
        (reached.came_back_the_same, reached.solids_the_same),
        (1, 1),
        "{reached:?}"
    );
}

/// Moved along its own axis, so that the first face compared, the bottom cap,
/// is on another plane.
#[test]
#[should_panic(expected = "with a different surface")]
fn a_round_bar_moved_one_unit_up_its_axis_is_told_apart_by_its_surfaces() {
    let moved = Isometry::translation(Vector::new(0.0, 0.0, 1.0)).expect("a finite offset");
    let here = one_solid_placed_by(a_round_bar, &Isometry::identity());
    let back = one_solid_placed_by(a_round_bar, &moved);
    assert_located_numbers_the_same(the_one_solid(&here), the_one_solid(&back));
}

/// A quarter turn about the bar's own axis leaves its three surfaces the same
/// bits and moves the corner on its seam.
#[test]
#[should_panic(expected = "at a different corner")]
fn a_round_bar_turned_a_quarter_about_its_axis_is_told_apart_by_its_corners() {
    let quarter_turn = Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_2)
        .expect("a finite angle about a unit axis");
    let here = one_solid_placed_by(a_round_bar, &Isometry::identity());
    let back = one_solid_placed_by(a_round_bar, &quarter_turn);
    assert_located_numbers_the_same(the_one_solid(&here), the_one_solid(&back));
}

/// A quarter turn about the line through a ball's poles leaves its sphere and
/// its two poles the same bits and turns the half-plane its seam is cut from.
#[test]
#[should_panic(expected = "with a different edge run")]
fn a_ball_turned_a_quarter_about_its_poles_is_told_apart_by_its_seam() {
    let quarter_turn = Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_2)
        .expect("a finite angle about a unit axis");
    let here = one_solid_placed_by(a_ball, &Isometry::identity());
    let back = one_solid_placed_by(a_ball, &quarter_turn);
    assert_located_numbers_the_same(the_one_solid(&here), the_one_solid(&back));
}

fn a_round_bar(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    model.add_round_bar(3.0, 5.0, &Isometry::identity())
}

fn a_ball(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    model.add_ball(2.0, &Isometry::identity())
}

fn a_cube(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    model.add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A model holding one shape, built at the origin and then placed by
/// `placement`, the order the mutator builds one in.
fn one_solid_placed_by(
    shape: fn(&mut Model) -> rcad_topo::Result<SolidKey>,
    placement: &Isometry,
) -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = shape(&mut model).expect("each shape fits in a model this size");
    model
        .place_solid(solid, placement)
        .expect("the key came from this model");
    model
}

fn the_one_solid(model: &Model) -> SolidView<'_> {
    model.solids().next().expect("the model holds one solid")
}

/// Change `data[..size]` into the `postcard` bytes of a model, and return how
/// many bytes that took.
///
/// # Every mutation is still a model
///
/// The input is `postcard` bytes of a [`Model`], the route a stranger's saved
/// file takes, and libFuzzer's own byte mutations almost never land on bytes
/// that still decode: 19 of the 1183 seeds checked in on 2026-09-25 did, and 16
/// of those were empty. So this replaces libFuzzer's mutator. It decodes the
/// input and makes one change a model can take — a byte changed where the
/// result still decodes, one more solid, one solid moved or turned, or a fresh
/// model — and encodes the result. In two 60-second runs on 2026-09-25, 11 145
/// of 11 179 and 12 272 of 12 309 mutated inputs decoded (`docs/decisions.md`
/// §241).
///
/// The shapes are the ones the writer has to spell differently: a cube, a round
/// bar, a ball, a frustum, a pointed cone and a quarter bend of pipe, which is
/// the torus a mechanical file holds. No builder in the kernel makes the last
/// three, so they are described face by face here, each with its corners exact
/// on its curved surface — a power-of-two slope on a cone, the inner equator on
/// a torus — because `Model::validate` asks that every corner of a curved face
/// be certified on it.
///
/// An input that does not decode becomes a fresh model, so the 1164 seeds that
/// never reached the writer each start a model rather than staying bytes.
///
/// The one input this leaves as it came is one where even a fresh one-solid
/// model does not fit in `max_size`. With no `-max_len`, libFuzzer's limit is
/// 4096 bytes — its own floor, since the corpus's longest seed is 4064 — and
/// the longest one-solid model drawn here, the quarter bend, is 681.
///
/// `byte_mutate` is libFuzzer's own mutation (`libfuzzer_sys::fuzzer_mutate`),
/// passed in rather than called directly, because this file is also compiled
/// into the ordinary test suite (`mod fuzzing` in
/// `every_model_the_writer_fuzzer_kept_is_written_again.rs`), which does not
/// depend on `libfuzzer-sys` and cannot safely call its mutation outside a real
/// fuzzing run (`docs/decisions.md` §241.1: it draws from libFuzzer's own
/// generator, not from `seed`). The fuzz target passes the real one; the
/// replay test passes a stand-in, over seeds chosen so it is never reached.
pub fn still_a_model(
    data: &mut [u8],
    size: usize,
    max_size: usize,
    seed: u32,
    byte_mutate: fn(&mut [u8], usize, usize) -> usize,
) -> usize {
    let mut draw = Draw::new(seed);
    let mut model = match postcard::from_bytes::<Model>(&data[..size]) {
        Ok(model) => model,
        Err(_) => a_fresh_model(&mut draw),
    };
    match draw.below(8) {
        0..=2 => {
            if let Some(nudged) =
                one_byte_changed_and_still_a_model(data, size, max_size, byte_mutate)
            {
                return nudged;
            }
            one_more_solid(&mut model, &mut draw);
        }
        3 | 4 => one_more_solid(&mut model, &mut draw),
        5 | 6 => one_solid_moved(&mut model, &mut draw),
        _ => model = a_fresh_model(&mut draw),
    }
    written_over(data, max_size, &model)
        .or_else(|| written_over(data, max_size, &a_fresh_model(&mut draw)))
        .unwrap_or(size)
}

/// Encode `model` over the front of `data`, if it fits in `max_size`.
fn written_over(data: &mut [u8], max_size: usize, model: &Model) -> Option<usize> {
    let bytes = postcard::to_allocvec(model).expect("a model always encodes");
    if bytes.len() > max_size {
        return None;
    }
    data[..bytes.len()].copy_from_slice(&bytes);
    Some(bytes.len())
}

/// libFuzzer's own mutation, kept only if the bytes still decode.
///
/// This is the half that reaches the writer with a model `Model::validate`
/// would refuse — a corner nudged off its face, a plane turned a little — which
/// no builder makes and which the writer owes an `Err` for rather than a crash.
fn one_byte_changed_and_still_a_model(
    data: &mut [u8],
    size: usize,
    max_size: usize,
    byte_mutate: fn(&mut [u8], usize, usize) -> usize,
) -> Option<usize> {
    let original = data[..size].to_vec();
    for _ in 0..4 {
        let nudged = byte_mutate(data, size, max_size);
        if postcard::from_bytes::<Model>(&data[..nudged]).is_ok() {
            return Some(nudged);
        }
        data[..size].copy_from_slice(&original);
    }
    None
}

/// A model of the size every shape here is drawn at, holding one of them.
fn a_fresh_model(draw: &mut Draw) -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_drawn_solid(&mut model, draw)
        .expect("every shape drawn here is sound in a model of this size");
    maybe_moved(&mut model, solid, draw);
    model
}

/// Add one drawn shape to `model`, or start again if the model will not take
/// it.
///
/// A model that came through the door can carry any scale at all, and a shape
/// a few units across is refused outright in a model whose working size is
/// `1e-300`. That refusal is the builder's answer to a caller, not a defect,
/// so the mutation moves on rather than keeping a model it could not change.
fn one_more_solid(model: &mut Model, draw: &mut Draw) {
    match a_drawn_solid(model, draw) {
        Ok(solid) => maybe_moved(model, solid, draw),
        Err(_) => *model = a_fresh_model(draw),
    }
}

/// Move or turn one of `model`'s solids.
///
/// Only a sound model is moved. `Model::place_solid` checks the solid it moved
/// and panics if the solid is broken, which is right for a solid the kernel
/// built and wrong to ask of one a changed byte broke on the way in.
fn one_solid_moved(model: &mut Model, draw: &mut Draw) {
    let sound = model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .is_ok();
    let count = model.solid_count() as u64;
    if !sound || count == 0 {
        one_more_solid(model, draw);
        return;
    }
    let which = usize::try_from(draw.below(count)).expect("below a usize count");
    let solid = model
        .solids()
        .nth(which)
        .expect("fewer than the model's own count")
        .key();
    model
        .place_solid(solid, &a_movement(draw))
        .expect("the key came from this model");
}

/// Leave the new solid where it was built a third of the time, move it a
/// third, and move and turn it the last third.
fn maybe_moved(model: &mut Model, solid: SolidKey, draw: &mut Draw) {
    if draw.below(3) == 0 {
        return;
    }
    model
        .place_solid(solid, &a_movement(draw))
        .expect("the key came from this model");
}

/// A move by whole units, and half the time a turn about one of the three axes
/// or a slanted one — by a multiple of an eighth of a turn, which is not exact
/// in `f64`, or by any angle at all.
fn a_movement(draw: &mut Draw) -> Isometry {
    let offset = Vector::new(
        draw.whole(-16, 16),
        draw.whole(-16, 16),
        draw.whole(-16, 16),
    );
    let moved = Isometry::translation(offset).expect("a finite offset");
    if draw.below(2) == 0 {
        return moved;
    }
    let axis = match draw.below(4) {
        0 => Axis::x(),
        1 => Axis::y(),
        2 => Axis::z(),
        _ => Axis::new(
            Point::origin(),
            Vector::new(draw.whole(1, 4), draw.whole(-4, 4), draw.whole(-4, 4)),
        )
        .expect("a direction whose first component is at least one"),
    };
    let angle = if draw.below(2) == 0 {
        draw.whole(1, 7) * core::f64::consts::FRAC_PI_4
    } else {
        draw.fraction() * core::f64::consts::TAU
    };
    let turned = Isometry::rotation(&axis, angle).expect("a finite angle about a unit axis");
    moved.compose(&turned)
}

/// One of the six shapes, with whole-number sizes and a power-of-two slope.
fn a_drawn_solid(model: &mut Model, draw: &mut Draw) -> rcad_topo::Result<SolidKey> {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    match draw.below(6) {
        0 => model.add_cube(draw.whole(1, 8), &policy, &mut log),
        1 => model.add_round_bar(draw.whole(1, 8), draw.whole(1, 16), &Isometry::identity()),
        2 => model.add_ball(draw.whole(1, 8), &Isometry::identity()),
        3 => {
            let near = draw.whole(1, 8);
            let far = near + draw.whole(1, 8);
            a_frustum(model, a_slope(draw), near, far)
        }
        4 => a_pointed_cone(model, a_slope(draw), draw.whole(1, 8)),
        _ => {
            let tube = draw.whole(1, 4);
            a_quarter_bend(model, tube + draw.whole(1, 8), tube)
        }
    }
}

/// A power of two, so a cone's corner at `(slope · height, 0, height)` is on
/// the cone exactly.
fn a_slope(draw: &mut Draw) -> f64 {
    [0.25, 0.5, 1.0, 2.0][usize::try_from(draw.below(4)).expect("below four")]
}

/// A frustum about `+z` with its apex at the origin and its caps at `near` and
/// `far` up the axis: two cap circles, a seam up the ruling over `+x`, three
/// faces — a round bar's structure on a cone.
fn a_frustum(model: &mut Model, slope: f64, near: f64, far: f64) -> rcad_topo::Result<SolidKey> {
    let skin = Cone::new(Axis::z(), slope).expect("a positive slope");
    let mut builder = SolidBuilder::new();
    let bottom_corner = builder
        .add_vertex(Point::new(slope * near, 0.0, near))
        .expect("a finite point");
    let top_corner = builder
        .add_vertex(Point::new(slope * far, 0.0, far))
        .expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling of a cone is straight");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the cone in one closed curve");
    let top_circle = builder
        .add_edge(
            top_corner,
            top_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the cone in one closed curve");

    let bottom = a_cap(
        &mut builder,
        Point::new(0.0, 0.0, near),
        Vector::new(0.0, 0.0, -1.0),
        bottom_circle,
        Orientation::Reversed,
    );
    let top = a_cap(
        &mut builder,
        Point::new(0.0, 0.0, far),
        Vector::new(0.0, 0.0, 1.0),
        top_circle,
        Orientation::Forward,
    );
    let side = a_wall(
        &mut builder,
        Surface::Cone(skin),
        bottom_circle,
        seam,
        top_circle,
    );
    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A cone that ends in a point: a base cap at `far` up the axis and one face on
/// the cone reaching down to the apex. Two faces, one edge, one vertex, and the
/// apex is not a vertex at all — it is where the cone's own chart runs out.
fn a_pointed_cone(model: &mut Model, slope: f64, far: f64) -> rcad_topo::Result<SolidKey> {
    let skin = Cone::new(Axis::z(), slope).expect("a positive slope");
    let mut builder = SolidBuilder::new();
    let rim = builder
        .add_vertex(Point::new(slope * far, 0.0, far))
        .expect("a finite point");
    let circle = builder
        .add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the cap plane cuts the cone in one closed curve");
    let cap = a_cap(
        &mut builder,
        Point::new(0.0, 0.0, far),
        Vector::new(0.0, 0.0, 1.0),
        circle,
        Orientation::Forward,
    );
    let side_coedge = builder
        .add_coedge(circle, Orientation::Reversed)
        .expect("an edge of this build");
    let side_wire = builder
        .add_wire(&[DraftStep::Along(side_coedge)])
        .expect("one closed edge is a wire");
    let side = builder
        .add_face(Surface::Cone(skin), &[side_wire])
        .expect("a face on the cone, bounded by its base circle");
    builder
        .add_shell(&[(cap, Orientation::Forward), (side, Orientation::Forward)])
        .expect("two faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A quarter of a torus about `+z`, from the cap in the plane `y = 0` to the
/// cap in the plane `x = 0`, with its seam along the inner equator.
///
/// The corners sit on the inner equator, `(ring − tube, 0, 0)` and
/// `(0, ring − tube, 0)`, where the torus's quartic is a difference of two
/// products that are equal number for number, so it cancels exactly for any
/// sizes. The seam names which of the two circles the plane `z = 0` cuts by
/// saying it is inside the pipe of the ring's own radius, and which quarter of
/// that circle by a half-space.
fn a_quarter_bend(model: &mut Model, ring: f64, tube: f64) -> rcad_topo::Result<SolidKey> {
    let skin = Torus::new(Axis::z(), ring, tube).expect("a tube narrower than its ring");
    let mut builder = SolidBuilder::new();
    let inner = ring - tube;
    let start_corner = builder
        .add_vertex(Point::new(inner, 0.0, 0.0))
        .expect("a finite point");
    let end_corner = builder
        .add_vertex(Point::new(0.0, inner, 0.0))
        .expect("a finite point");
    let flat_through_the_middle =
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let seam = builder
        .add_edge(
            start_corner,
            end_corner,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(flat_through_the_middle),
                picked_by: [
                    Picker::InsideAPipe(
                        Cylinder::new(Axis::z(), ring).expect("the ring's own radius"),
                    ),
                    Picker::from(
                        HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.0)
                            .expect("a normal with a length"),
                    ),
                ]
                .into_iter()
                .collect(),
            },
        )
        .expect("a quarter of the inner equator");
    let start_circle = builder
        .add_edge(
            start_corner,
            start_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("one whole circle of the two this plane cuts");
    let end_circle = builder
        .add_edge(
            end_corner,
            end_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("one whole circle of the two this plane cuts");

    let start_cap = a_cap(
        &mut builder,
        Point::origin(),
        Vector::new(0.0, -1.0, 0.0),
        start_circle,
        Orientation::Reversed,
    );
    let end_cap = a_cap(
        &mut builder,
        Point::origin(),
        Vector::new(-1.0, 0.0, 0.0),
        end_circle,
        Orientation::Forward,
    );
    let side = a_wall(
        &mut builder,
        Surface::Torus(skin),
        start_circle,
        seam,
        end_circle,
    );
    builder
        .add_shell(&[
            (start_cap, Orientation::Forward),
            (end_cap, Orientation::Forward),
            (side, Orientation::Reversed),
        ])
        .expect("three faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A flat face through `at`, facing `outward`, trimmed by one closed edge.
fn a_cap(
    builder: &mut SolidBuilder,
    at: Point,
    outward: Vector,
    circle: rcad_topo::DraftEdge,
    turn: Orientation,
) -> rcad_topo::DraftFace {
    let plane = Plane::from_point_and_normal(at, outward).expect("a coordinate direction");
    let coedge = builder
        .add_coedge(circle, turn)
        .expect("an edge of this build");
    let wire = builder
        .add_wire(&[DraftStep::Along(coedge)])
        .expect("one closed edge is a wire");
    builder
        .add_face(Surface::Plane(plane), &[wire])
        .expect("a cap is a plane trimmed by its circle")
}

/// The curved face between two closed edges: round the first, along the seam,
/// back round the second, and back down the seam — the seam's two coedges in
/// one wire, which is what makes it a seam.
fn a_wall(
    builder: &mut SolidBuilder,
    surface: Surface,
    first: rcad_topo::DraftEdge,
    seam: rcad_topo::DraftEdge,
    second: rcad_topo::DraftEdge,
) -> rcad_topo::DraftFace {
    let round_the_wall: Vec<DraftStep> = [
        (first, Orientation::Forward),
        (seam, Orientation::Forward),
        (second, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .into_iter()
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    })
    .collect();
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("round one end, along the seam, round the other, and back");
    builder
        .add_face(surface, &[wire])
        .expect("a face on the curved surface")
}

/// Numbers drawn from the seed libFuzzer hands the mutator, so a mutation is
/// the same every time its seed is — except libFuzzer's own byte mutation,
/// which draws from libFuzzer's generator and not from this seed.
struct Draw(u64);

impl Draw {
    fn new(seed: u32) -> Self {
        Self(u64::from(seed))
    }

    /// SplitMix64: every seed, even zero, gives a well-mixed stream.
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, count: u64) -> u64 {
        self.next() % count
    }

    /// A whole number from `low` to `high`, both included.
    fn whole(&mut self, low: i32, high: i32) -> f64 {
        let span = u64::try_from(high - low + 1).expect("low is not above high");
        f64::from(low) + self.below(span) as f64
    }

    /// A number from zero up to but not including one.
    fn fraction(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1_u64 << 53) as f64
    }
}
