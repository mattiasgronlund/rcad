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
//! comparison is `round_trip_one`'s in
//! `a_real_file_written_out_and_read_back_is_the_same_solid.rs`, at the level
//! that test compares: the written text parses, every reference in it points
//! somewhere, `Import::read` turns it back into a model that validates, the
//! same number of solids comes back, and each solid's volume, surface area and
//! centroid are the same bit for bit. That test's last leg, a `Rule::Difference`
//! against a third solid, is left out: it is a fuse, and a fuse that panics on
//! a sound model here would be a fuse defect reported by the writer's target.
//!
//! What the source explains is counted and not failed. A model that does not
//! pass `Model::validate` was never a solid, so a file of it owes nothing on
//! the way back; a solid that does not measure has no answer to compare.
//! Everything else that differs panics, because it is a sound model this
//! writer and this reader disagree about.

// Shared between a test binary and a fuzz target, and each uses a subset.
#![allow(dead_code)]

// `rcad-topo` owns the door, so the check on what it promised lives there and
// this reaches across to it. Both this file and `fuse.rs` carried a copy of it
// before, which is two places to update and one to forget.
#[path = "../../../rcad-topo/tests/fuzzing/the_door.rs"]
mod the_door;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Model, SolidKey};

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
    /// Written files the reader turned back into a model.
    pub read_back: usize,
    /// Read-back models not compared because a solid of the model that went in
    /// does not measure.
    pub did_not_measure: usize,
    /// Solids whose volume, surface area and centroid were compared bit for bit
    /// with the solid that went in, and matched.
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
    let file = StepFile::read(text.as_bytes())
        .unwrap_or_else(|why| panic!("this writer wrote a file its own reader cannot parse: {why}"));
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
        let Ok(here) = source
            .solid(here_key)
            .expect("this model made the key")
            .measure(deflection, &policy, &mut log)
        else {
            reached.did_not_measure = 1;
            return reached;
        };
        let back = there
            .solid(back_key)
            .expect("this model made the key")
            .measure(deflection, &policy, &mut log)
            .unwrap_or_else(|why| {
                panic!("a solid that measured does not measure after its own file: {why}")
            });
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
        reached.solids_the_same += 1;
    }
    reached.came_back_the_same = 1;
    reached
}
