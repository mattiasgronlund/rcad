//! A round bar whose top corner is moved a millimetre in its file, read back,
//! and fused under `Policy::certified`. `docs/decisions.md` §294 is the record
//! and this file is its measurement.
//!
//! A round bar's top corner has two surfaces round it, the top cap's plane and
//! the wall's cylinder, so `place_a_corner` hands it to the fuse as the model
//! stored it and asks neither surface about it (§286). `Deserialize` does not
//! call `Model::validate`, so a file can carry that corner anywhere and the
//! model still reads back. §286.4 left open what a fuse then does with it: a
//! wrong `Ok`, `TheFuseWroteABrokenAnswer`, or a panic. That answer is what
//! decides whether option (c), asking the corner at the door with `validate`'s
//! own room, is owed.
//!
//! **No partner tried here gets a wrong `Ok` or a panic out of it.** What comes
//! back depends on where the partner's surfaces are:
//!
//! - **Where the answer keeps the moved corner**, the sweep of the answer finds
//!   it and the fuse refuses with `TheFuseWroteABrokenAnswer`, carrying the
//!   very refusal `validate` gave the bar. That error's own doc says it is "a
//!   bug here rather than bad input". Here it is bad input, so the fuse blames
//!   itself for the caller's file. The model is left as it was.
//! - **Where the partner cuts the moved corner away**, the answer is the one
//!   the sound bar gives: the same corners to the last bit, the same volume, and
//!   it validates on its own. A face is its surface (§17.1), and a corner that
//!   is cut away was never more than a cached sample of it.
//! - **Where a partner's face runs through where the corner should be**, the
//!   fuse puts a new corner there and cannot tell it from the moved one. It
//!   refuses, as `Undecidable` or as `CannotTellTheCellsOfAFaceApart`, which
//!   name the arithmetic and the arrangement and not the file.
//!
//! The corner is moved in the bytes because nothing in the public API writes a
//! corner off its surfaces: every builder validates what it built.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 10.0;

/// Where `add_round_bar` puts the top corner: on the seam, at the top.
const WHERE_THE_BAR_PUTS_IT: [f64; 3] = [RADIUS, 0.0, HEIGHT];

/// A millimetre out from the wall, still on the top cap's plane.
const OFF_THE_WALL: [f64; 3] = [RADIUS + 1.0, 0.0, HEIGHT];

/// A millimetre above the top cap, still on the wall's cylinder.
const OFF_THE_CAP: [f64; 3] = [RADIUS, 0.0, HEIGHT + 1.0];

/// A millimetre out from the wall and a quarter turn round it, so the corner
/// no longer sits where the seam meets the cap either.
const A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL: [f64; 3] = [0.0, RADIUS + 1.0, HEIGHT];

const EVERY_MOVE: [[f64; 3]; 3] = [OFF_THE_WALL, OFF_THE_CAP, A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL];

/// A block, as the two corners of its footprint, the height of its floor and
/// how tall it stands.
#[derive(Clone, Copy, Debug)]
struct Block {
    low: [f64; 2],
    high: [f64; 2],
    floor: f64,
    tall: f64,
}

/// Nowhere near the bar.
const FAR_AWAY: Block = Block {
    low: [20.0, 20.0],
    high: [22.0, 22.0],
    floor: 20.0,
    tall: 2.0,
};

/// Over the bottom two millimetres of the bar.
const OVER_THE_BOTTOM: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: -1.0,
    tall: 3.0,
};

/// Over the top two millimetres of the bar, and the moved corner with them.
const OVER_THE_TOP: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// Over the top two millimetres of the side the seam is on, `x > 2`.
const OVER_THE_RIM_BY_THE_SEAM: Block = Block {
    low: [2.0, -5.0],
    high: [5.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// Over the top two millimetres of the side away from the seam, `x < -2`.
const OVER_THE_RIM_AWAY_FROM_THE_SEAM: Block = Block {
    low: [-5.0, -5.0],
    high: [-2.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// A notch in the top of the rim a quarter turn round from the seam, where
/// [`A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL`] claims the corner is.
const A_NOTCH_A_QUARTER_TURN_ROUND: Block = Block {
    low: [-1.0, 2.0],
    high: [1.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// The whole `y > 0` half of the bar, so one face of the block runs along the
/// seam and through where the corner should be.
const THE_HALF_PAST_THE_SEAM: Block = Block {
    low: [-5.0, 0.0],
    high: [5.0, 5.0],
    floor: -1.0,
    tall: 12.0,
};

#[test]
fn a_bar_whose_top_corner_is_moved_a_millimetre_in_its_file_reads_back_and_does_not_validate() {
    let policy = Policy::certified();
    for (moved_to, refusal) in [
        (OFF_THE_WALL, Invalid::VertexOffItsFaceSurface),
        (OFF_THE_CAP, Invalid::VertexOffItsFacePlane { distance: 1.0 }),
        (
            A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL,
            Invalid::VertexOffItsFaceSurface,
        ),
    ] {
        let (model, _) = a_bar_read_back_with_its_top_corner_at(moved_to);
        match model.validate(&policy, &mut ProvenanceLog::new()) {
            Err(Error::Invalid(found)) => assert_eq!(found, refusal, "moved to {moved_to:?}"),
            other => panic!("a corner moved to {moved_to:?} validates as {other:?}"),
        }
    }
    let (sound, _) = a_bar_read_back_with_its_top_corner_at(WHERE_THE_BAR_PUTS_IT);
    sound
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the same bytes with the corner left alone are a sound bar");
}

/// **The error is the wrong one to hand a caller whose file is broken.**
/// `TheFuseWroteABrokenAnswer` is documented as "a bug here rather than bad
/// input", and what it carries here is the refusal `validate` gives the bar
/// before any fuse. No model is harmed: the refusal leaves it byte for byte
/// as it was, which is the fuse's "Atomic" promise, and nothing corrupt is
/// written. §294 says why this is not the gap option (c) closes.
#[test]
fn a_fuse_that_keeps_the_moved_corner_blames_itself_for_the_broken_file() {
    let policy = Policy::certified();
    for moved_to in EVERY_MOVE {
        for (partner, rule) in [(FAR_AWAY, Rule::Union), (OVER_THE_BOTTOM, Rule::Difference)] {
            let (mut model, bar) = a_bar_read_back_with_its_top_corner_at(moved_to);
            let Err(Error::Invalid(what_validate_says)) =
                model.validate(&policy, &mut ProvenanceLog::new())
            else {
                panic!("the bar with its corner at {moved_to:?} is broken");
            };
            let block = a_block(&mut model, partner);
            let before = postcard::to_allocvec(&model).expect("a model writes out");

            match model.fuse(&[bar], &[block], rule, &policy, &mut ProvenanceLog::new()) {
                Err(Error::TheFuseWroteABrokenAnswer { broken }) => assert_eq!(
                    broken, what_validate_says,
                    "{partner:?} {rule:?} with the corner at {moved_to:?}: the answer is \
                     refused for the bar's own fault"
                ),
                other => panic!(
                    "{partner:?} {rule:?} with the corner at {moved_to:?} came back {other:?}"
                ),
            }
            assert_eq!(
                postcard::to_allocvec(&model).expect("a model writes out"),
                before,
                "a refused fuse leaves the model as it was"
            );
        }
    }
}

#[test]
fn a_fuse_that_cuts_the_moved_corner_away_gives_the_answer_the_sound_bar_gives() {
    for moved_to in EVERY_MOVE {
        for (partner, rule) in [
            (OVER_THE_TOP, Rule::Difference),
            (OVER_THE_RIM_BY_THE_SEAM, Rule::Difference),
            (OVER_THE_RIM_AWAY_FROM_THE_SEAM, Rule::Difference),
            (OVER_THE_RIM_BY_THE_SEAM, Rule::Union),
            (OVER_THE_RIM_BY_THE_SEAM, Rule::Intersection),
            (A_NOTCH_A_QUARTER_TURN_ROUND, Rule::Difference),
        ] {
            the_same_answer_as_the_sound_bar(moved_to, partner, rule);
        }
    }
}

/// A block whose face runs along the seam, through the point where the top
/// corner should be.
///
/// The fuse puts a corner of its own at `(3, 0, 10)`, where that face crosses
/// the top circle, and the moved corner is still the start of the same circle.
/// Moved off the wall it sits in the block's face as well, and the arrangement
/// cannot tell the two regions of a face apart. Moved off the cap it sits on
/// the wall above the new corner, and ordering the two along the circle's
/// crossing comes out `Undecidable` with nothing to widen. Moved a quarter turn
/// round, it is inside the half the block takes, and the answer is the sound
/// bar's. **None of the three refusals names the file**, and neither is wrong
/// about what it could not do.
#[test]
fn a_cut_along_the_seam_refuses_a_corner_moved_in_the_cut_and_answers_one_moved_out_of_it() {
    let policy = Policy::certified();
    for rule in [Rule::Difference, Rule::Intersection] {
        let (mut model, bar) = a_bar_read_back_with_its_top_corner_at(OFF_THE_WALL);
        let block = a_block(&mut model, THE_HALF_PAST_THE_SEAM);
        let fused = model.fuse(&[bar], &[block], rule, &policy, &mut ProvenanceLog::new());
        assert!(
            matches!(fused, Err(Error::CannotTellTheCellsOfAFaceApart { .. })),
            "{rule:?} with the corner off the wall came back {fused:?}"
        );

        let (mut model, bar) = a_bar_read_back_with_its_top_corner_at(OFF_THE_CAP);
        let block = a_block(&mut model, THE_HALF_PAST_THE_SEAM);
        match model.fuse(&[bar], &[block], rule, &policy, &mut ProvenanceLog::new()) {
            Err(Error::Undecidable { records }) => assert!(
                records
                    .records()
                    .iter()
                    .any(|record| record.predicate() == "order along a crossing"),
                "{rule:?}: the refusal is the order of the two corners along the circle: \
                 {records:?}"
            ),
            other => panic!("{rule:?} with the corner off the cap came back {other:?}"),
        }

        the_same_answer_as_the_sound_bar(
            A_QUARTER_TURN_ROUND_AND_OFF_THE_WALL,
            THE_HALF_PAST_THE_SEAM,
            rule,
        );
    }
}

/// Fuse the bar with its corner moved, and the same bar with its corner left
/// alone, against the same block, and check the two answers are one solid.
///
/// "The same" is the same corners to the last bit, taken as a set because the
/// fuse may start a cut circle at a different corner, and the same volume to
/// within what that different start does to the mesh the volume is summed over.
fn the_same_answer_as_the_sound_bar(moved_to: [f64; 3], partner: Block, rule: Rule) {
    let policy = Policy::certified();
    let broken = fused_answer(moved_to, partner, rule);
    let sound = fused_answer(WHERE_THE_BAR_PUTS_IT, partner, rule);
    assert_eq!(
        broken.corners, sound.corners,
        "{partner:?} {rule:?} with the corner at {moved_to:?}: the corners of the answer"
    );
    assert!(
        (broken.volume - sound.volume).abs() <= 1e-12 * sound.volume,
        "{partner:?} {rule:?} with the corner at {moved_to:?}: volume {} against {}",
        broken.volume,
        sound.volume
    );
    broken
        .model
        .validate_solid(broken.solid, &policy, &mut ProvenanceLog::new())
        .expect("the answer on its own is sound");
}

struct Answer {
    model: Model,
    solid: SolidKey,
    corners: Vec<[u64; 3]>,
    volume: f64,
}

/// The one solid fusing the bar with its top corner at `moved_to` and
/// `partner` gives, its corners as bits, sorted, and its volume.
fn fused_answer(moved_to: [f64; 3], partner: Block, rule: Rule) -> Answer {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let (mut model, bar) = a_bar_read_back_with_its_top_corner_at(moved_to);
    let block = a_block(&mut model, partner);
    let fused = model
        .fuse(&[bar], &[block], rule, &policy, &mut log)
        .unwrap_or_else(|error| {
            panic!("{partner:?} {rule:?} with the corner at {moved_to:?} refused: {error:?}")
        });
    let [solid] = fused.solids() else {
        panic!(
            "{partner:?} {rule:?} with the corner at {moved_to:?} is one solid, not {}",
            fused.solids().len()
        );
    };
    let view = model.solid(*solid).expect("the fuse made this key");
    let mut corners: Vec<[u64; 3]> = view
        .faces()
        .flat_map(|face| face.wires())
        .flat_map(|wire| wire.coedges())
        .map(|coedge| coedge.start().point().coordinates().map(f64::to_bits))
        .collect();
    corners.sort_unstable();
    corners.dedup();
    let volume = view
        .measure(Deflection::for_scale(model.scale()), &policy, &mut log)
        .expect("the answer measures")
        .volume();
    Answer {
        solid: *solid,
        model,
        corners,
        volume,
    }
}

/// A bar of radius three and height ten, written out with postcard, its top
/// corner overwritten in the bytes with `moved_to`, and read back.
///
/// The corner is the only place the three numbers `(3, 0, 10)` sit side by
/// side in the file. That is asserted, so the overwrite cannot land on a
/// surface.
fn a_bar_read_back_with_its_top_corner_at(moved_to: [f64; 3]) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar of radius three and height ten");
    let mut bytes = postcard::to_allocvec(&model).expect("a model writes out");

    let stored = the_bytes_of(WHERE_THE_BAR_PUTS_IT);
    let places: Vec<usize> = bytes
        .windows(stored.len())
        .enumerate()
        .filter(|(_, window)| *window == stored.as_slice())
        .map(|(at, _)| at)
        .collect();
    let [at] = places.as_slice() else {
        panic!("the top corner is written once, and was found at {places:?}");
    };
    bytes[*at..*at + stored.len()].copy_from_slice(&the_bytes_of(moved_to));

    let read_back: Model =
        postcard::from_bytes(&bytes).expect("reading back does not ask where a corner is");
    let bar = read_back.solids().next().expect("the bar came back").key();
    (read_back, bar)
}

/// A point the way postcard writes one: three `f64`s, little-endian.
fn the_bytes_of(point: [f64; 3]) -> Vec<u8> {
    point
        .iter()
        .flat_map(|coordinate| coordinate.to_le_bytes())
        .collect()
}

fn a_block(model: &mut Model, block: Block) -> SolidKey {
    let [wide, deep] = [block.high[0] - block.low[0], block.high[1] - block.low[1]];
    let footprint = ProfileBuilder::new(vec![[0.0, 0.0], [wide, 0.0], [wide, deep], [0.0, deep]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    model
        .add_extrusion(
            &footprint,
            block.tall,
            &Isometry::translation(Vector::new(block.low[0], block.low[1], block.floor))
                .expect("a plain translation"),
        )
        .expect("a block")
}
