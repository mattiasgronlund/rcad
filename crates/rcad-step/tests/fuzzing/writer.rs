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

// Shared between a test binary and a fuzz target, and each uses a subset.
#![allow(dead_code)]

// `rcad-topo` owns the door, so the check on what it promised lives there and
// this reaches across to it. Both this file and `fuse.rs` carried a copy of it
// before, which is two places to update and one to forget.
#[path = "../../../rcad-topo/tests/fuzzing/the_door.rs"]
mod the_door;

use rcad_topo::Model;

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
        }
    }
}

/// **One draw of bytes, decoded the way a stranger's saved file is and written
/// out.**
///
/// The fuzz target and the suite's replay both call this and nothing narrower.
///
/// # Panics
///
/// If a model came through `Deserialize` carrying a number the door promised it
/// would refuse.
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
        Ok(_) => reached.written = 1,
        Err(_) => reached.refused = 1,
    }
    reached
}
