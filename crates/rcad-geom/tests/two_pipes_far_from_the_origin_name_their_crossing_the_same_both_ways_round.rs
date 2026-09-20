//! §181's crossing that disagrees with itself: §140's pair — a thin pipe lying
//! along the skin of a pipe 153 decades fatter — moved away from the origin.
//!
//! The night of 2026-09-20 drew four of these and
//! `naming_a_pair_does_not_depend_on_which_way_round_it_is_asked` caught them.
//! Asking the fat pipe first names two straight runs; asking the thin pipe
//! first refuses with `CannotNameWhichPieceOfACrossingThisIs`.
//!
//! # Why the answer exists
//!
//! Both axes run along `x`, so the question is two circles in `(y, z)`: one of
//! radius `R1`, one of radius `R2` whose centre sits `R1` away in `y` and `Δz`
//! along the axis. The true axis separation is `√(R1² + Δz²) = R1 + Δz²/(2·R1)`,
//! and that excess is `1.4e-111` against `R2 = 1.984375`. So the thin pipe cuts
//! the fat pipe's skin in two endless straight lines, by 111 decades — not
//! tangentially and not at a knife edge.
//!
//! # What is different from §140, and it is not a radius
//!
//! §140's pair sits at the origin, so its picking plane passes through the
//! origin and `normal · anchor` is exactly `0` — an offset with no rounding to
//! lose. This pair sits `3.7e21` out, where the offset is `1.87e175` and one
//! ulp of it places the plane on a grid of `6.9e5`, while the two lines it has
//! to separate stand `3.97` apart. `docs/decisions.md` §181.
//!
//! # Why this asserts agreement rather than an answer
//!
//! §181.6 leaves open which way the pair should be made to agree: teaching both
//! orders to answer hands every consumer a plane that cannot resolve the slab,
//! and refusing both is the smaller change under "crash > corrupt". The
//! invariant the fuzzer caught is broken either way round, so this pins that and
//! not the direction.

#![forbid(unsafe_code)]

mod fuzzing;

use fuzzing::{bytes_for, two_surfaces_from};
use rcad_geom::{SurfaceMeet, intersect};
use rcad_predicates::{Policy, ProvenanceLog};

/// The fat pipe's radius, and the thin pipe's offset off the fat axis.
const R1: f64 = 5.047645337156084e153;

/// The thin pipe's radius. `127/64`, so `R2²` is exact.
const R2: f64 = 1.984375;

/// How far the thin pipe's axis sits along the fat axis' own direction.
const Z: f64 = 41581179.0;

/// How far the fat pipe's own axis sits from the origin along `z`.
const FAR: f64 = 3.705489715806406e21;

/// The fourteen numbers the fuzzer drew, as the decoder reads them: a place and
/// a direction and a radius apiece.
const SEED: [f64; 14] = [
    100663296.0,
    0.0,
    FAR,
    127.0,
    0.0,
    0.0,
    R1,
    R1,
    R1,
    Z,
    167772415.0,
    0.0,
    0.0,
    R2,
];

/// Two side-by-side pipes far from the origin name the same crossing whichever
/// is asked about first.
///
/// `docs/decisions.md` §181.
#[test]
fn a_pipe_on_a_fat_pipes_skin_is_named_the_same_way_round_far_from_the_origin() {
    let pair = two_surfaces_from(&bytes_for([2, 2], &SEED)).expect("two pipes");
    let named = [[0, 1], [1, 0]].map(|[first, second]| {
        how_it_was_named(intersect(
            &pair[first],
            &pair[second],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ))
    });
    assert_eq!(
        named[0], named[1],
        "the same pair of pipes was named differently by which one was asked about first"
    );
}

/// What a crossing was named, coarsely enough that the two orders' answers are
/// comparable.
///
/// The two orders build the same plane with negated normals, so the runs
/// themselves differ bit for bit even where the two orders agree completely.
fn how_it_was_named(met: rcad_geom::Result<SurfaceMeet>) -> String {
    match met {
        Ok(SurfaceMeet::Curves(runs)) => format!("{} run(s)", runs.len()),
        Ok(SurfaceMeet::Nothing) => "nothing".to_owned(),
        Ok(SurfaceMeet::TouchAtOnePoint { .. }) => "one touching point".to_owned(),
        Ok(SurfaceMeet::SameSurface) => "the same surface".to_owned(),
        Ok(_) => "an arm that did not exist when this test was written".to_owned(),
        Err(why) => format!("refused: {why:?}"),
    }
}
