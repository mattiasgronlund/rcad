//! §140's crossing that disagrees with itself: a hair-thin pipe lying along the
//! skin of a pipe 161 decades fatter.
//!
//! The night of 2026-09-19 drew the pair below and
//! `naming_a_pair_does_not_depend_on_which_way_round_it_is_asked` caught it —
//! the fat pipe asked first refuses, the thin pipe asked first names two
//! straight runs. In exact arithmetic the two cross, so the refusal is §16.10's
//! gap and the answer is the right one.
//!
//! # The cross-section, and why the answer exists
//!
//! Both axes run along `x`, so the whole question is two circles in `(y, z)`:
//! one of radius `r1` at the origin, one of radius `r2` at `(r1, z)`. They cut
//! each other twice exactly when `(r1 − r2)² < d² < (r1 + r2)²` for `d² =
//! r1² + z²`. Subtracting `r1²` off both sides leaves `z² + 2·r1·r2 − r2² > 0`
//! and `2·r1·r2 + r2² − z² > 0`, and in exact rational arithmetic both sides
//! come out at `5.4531438193293825e146` — positive by 131 decades, not a knife
//! edge. The thin pipe's axis misses the fat pipe's skin by
//! `1.712674258532773e-139`, which is `3.17e-132` of its own radius, so the fat
//! pipe cuts it near enough through the middle.
//!
//! # Where the refusing order used to lose it
//!
//! `two_lines_told_apart` was handed each line as a whole coordinate and asked
//! which side of the picking plane it fell. Asked the fat pipe first the step
//! along the axis-to-axis line is `5.047645337156084e153` against a step off it
//! of `5.4016709327697754e-8` — a ratio of `9.34e160` — so the short step was
//! gone before the dot product started, and the two products that then had to
//! cancel were `2.0988704429280249e161` apiece against a wanted reading of
//! `2.7265719096646913e146`. Asked the thin pipe first the same step is
//! `1.712674258532773e-139` and there is nothing to cancel.
//!
//! The two lines now arrive as a point, a step along a stored direction and a
//! step either side of it, and the two readings that used to be subtracted out
//! of `2.1e161` are summed exactly instead. `docs/decisions.md` §140.

#![forbid(unsafe_code)]

mod fuzzing;

use fuzzing::{bytes_for, two_surfaces_from};
use rcad_geom::{SurfaceMeet, intersect};
use rcad_predicates::{Policy, ProvenanceLog};

/// The fat pipe's radius, and the thin pipe's offset off the fat axis.
const R1: f64 = 5.047645337156084e153;

/// The thin pipe's radius.
const R2: f64 = 5.4016709327697754e-8;

/// How far the thin pipe's axis sits along the fat axis' own direction.
const Z: f64 = 41581179.0;

/// The fourteen numbers the fuzzer drew, as the decoder reads them: a place and
/// a direction and a radius apiece.
const SEED: [f64; 14] = [
    0.0,
    0.0,
    0.0,
    127.0,
    0.0,
    0.0,
    R1,
    R1,
    R1,
    Z,
    142606335.0,
    0.0,
    0.0,
    R2,
];

/// Two side-by-side pipes that cross in two straight runs, whichever is asked
/// about first.
///
/// `docs/decisions.md` §140.
#[test]
fn a_thin_pipe_on_a_fat_pipes_skin_crosses_it_twice_either_way_round() {
    let pair = two_surfaces_from(&bytes_for([2, 2], &SEED)).expect("two pipes");
    for [first, second] in [[0, 1], [1, 0]] {
        let met = intersect(
            &pair[first],
            &pair[second],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let Ok(SurfaceMeet::Curves(runs)) = met else {
            panic!("asking pipe {first} first did not name a crossing: {met:?}");
        };
        assert_eq!(
            runs.len(),
            2,
            "asking pipe {first} first named the wrong number of runs"
        );
    }
}
