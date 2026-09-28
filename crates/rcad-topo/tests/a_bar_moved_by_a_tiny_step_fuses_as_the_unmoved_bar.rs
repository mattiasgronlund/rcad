//! A round bar moved `2⁻¹⁰⁰⁷` along `y`, fused with a cube, against the same
//! fuse of the bar that has not moved (`docs/decisions.md` §246.6, §322).
//!
//! §246 found the move in a fuzzer seed: one changed byte is the top byte of the
//! `y` of the bar's placement. Unmoved, both fuses below go through. Moved, both
//! refused on a predicate, and for one reason: `2⁻¹⁰⁰⁷` squared is `2⁻²⁰¹⁴`,
//! below the smallest subnormal, and the exact level refused any sum with that
//! product in it.
//!
//! - **Through the wall**, the line where the cube's wall `x = 1` crosses the
//!   bar's cap plane `z = 0` meets the bar's wall where `(y − 2⁻¹⁰⁰⁷)² = 3`.
//!   The `2⁻²⁰¹⁴` cancels out of that discriminant, and "where three surfaces
//!   meet" refused it at the exact level with an enclosure of infinity.
//! - **Far away**, selection asks whether the bar's wall is the same surface as
//!   the cube's wall `y = 2`, and `intersect` works out how the two cross. The
//!   unmoved bar touches that plane; moved, it pokes `2⁻¹⁰⁰⁷` through it and
//!   leaves `2⁻¹⁰⁰⁵` of room. "Where a plane and a pipe cross" refused that at
//!   the interval level, `[0, 8.9e-16]`.
//!
//! Both are gaps, and `crates/rcad-geom/tests/a_pipe_moved_by_a_speck_is_still_answered.rs`
//! asks the two questions on their own. `rcad-geom`'s exact arithmetic now keeps
//! a product that falls off the bottom of the range and carries a box round
//! what it lost.
//!
//! **The union far away now comes back.** Past the predicates it used to reach
//! the triangulator, which refused the bar's cap corner at `y = 2⁻¹⁰⁰⁷` as too
//! small for spade: §246.5's refusal. Queue row Q79's flush stops that refusal,
//! and queue row Q325 found it: the union now validates, and comes back bit
//! for bit the same — same face, edge and vertex counts, same volumes — as the
//! unmoved pair's.
//! `a_bar_moved_by_a_speck_united_with_a_cube_far_away_holds_what_the_unmoved_one_does`
//! checks that.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

/// `2⁻¹⁰⁰⁷`, from its bits (§246.3 says why not `powi`).
const SPECK: f64 = f64::from_bits(0x0100_0000_0000_0000);

/// A bar of radius two and eight tall, moved `step` along `y`, and a cube of
/// side two with its low corner at `cube_at` — §246's own pair.
fn bar_and_cube(step: f64, cube_at: Vector) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(8.0).expect("8 mm is a working size"));
    let bar = model
        .add_round_bar(2.0, 8.0, &Isometry::identity())
        .expect("a bar of radius two, eight tall");
    model
        .place_solid(
            bar,
            &Isometry::translation(Vector::new(0.0, step, 0.0)).expect("a finite move"),
        )
        .expect("this model made the key");
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    model
        .place_solid(
            cube,
            &Isometry::translation(cube_at).expect("a finite move"),
        )
        .expect("this model made the key");
    (model, bar, cube)
}

/// The cube's low corner for the difference: it spans `1 ≤ x ≤ 3`, so the bar's
/// wall at radius two runs through it.
fn through_the_wall() -> Vector {
    Vector::new(1.0, -1.0, 3.0)
}

/// The bar with the cube taken out of its wall, and how much it holds.
fn the_bar_less_the_cube(step: f64) -> f64 {
    let (mut model, bar, cube) = bar_and_cube(step, through_the_wall());
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(
            &[bar],
            &[cube],
            Rule::Difference,
            &Policy::certified(),
            &mut log,
        )
        .unwrap_or_else(|refusal| panic!("moved by {step:e}, the difference refused: {refusal:?}"));
    assert!(
        log.is_empty(),
        "a certified fuse writes nothing down: {log:?}"
    );
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the difference validates");
    let [answer] = fused.solids() else {
        panic!("one bar with a notch in it, got {:?}", fused.solids());
    };
    model
        .solid(*answer)
        .expect("the answer is in this model")
        .measure(
            Deflection::of(0.01).expect("a hundredth is a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the answer measures")
        .volume()
}

#[test]
fn a_bar_moved_by_a_speck_less_a_cube_through_its_wall_holds_what_the_unmoved_one_does() {
    let moved = the_bar_less_the_cube(SPECK);
    let unmoved = the_bar_less_the_cube(0.0);
    assert_eq!(
        moved, unmoved,
        "a move of 2^-1007 moves no mesh point by a bit"
    );
}

/// The bar and the cube ten along `x`, united: each answer solid's own face,
/// edge and vertex count and volume, smaller solid first. The cube touches
/// nothing, so the union keeps them as two solids, not one.
fn union_far_away(step: f64) -> Vec<(usize, usize, usize, f64)> {
    let (mut model, bar, cube) = bar_and_cube(step, Vector::new(10.0, 0.0, 0.0));
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(&[bar], &[cube], Rule::Union, &Policy::certified(), &mut log)
        .unwrap_or_else(|refusal| panic!("moved by {step:e}, the union refused: {refusal:?}"));
    assert!(
        log.is_empty(),
        "a certified fuse writes nothing down: {log:?}"
    );
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the union validates");
    let mut shapes: Vec<_> = fused
        .solids()
        .iter()
        .map(|&key| {
            let solid = model.solid(key).expect("the answer is in this model");
            let (edges, vertices) = edges_and_vertices(solid);
            let volume = solid
                .measure(
                    Deflection::of(0.01).expect("a hundredth is a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("the answer measures")
                .volume();
            (solid.faces().count(), edges, vertices, volume)
        })
        .collect();
    shapes.sort_by_key(|&(faces, ..)| faces);
    shapes
}

/// The edges and vertices a solid's faces reach, each counted once no matter
/// how many of the solid's faces share it.
fn edges_and_vertices(solid: rcad_topo::SolidView) -> (usize, usize) {
    let mut edges = std::collections::HashSet::new();
    let mut vertices = std::collections::HashSet::new();
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                edges.insert(coedge.edge().key());
                vertices.insert(coedge.start().key());
                vertices.insert(coedge.end().key());
            }
        }
    }
    (edges.len(), vertices.len())
}

/// The cube ten along `x` touches nothing, and its wall `y = 2` lies flat
/// against where the unmoved bar's wall reaches. Since Q79's flush, the union
/// comes back and validates, bit for bit the same as the unmoved pair's.
#[test]
fn a_bar_moved_by_a_speck_united_with_a_cube_far_away_holds_what_the_unmoved_one_does() {
    let moved = union_far_away(SPECK);
    let unmoved = union_far_away(0.0);
    assert_eq!(
        moved, unmoved,
        "a move of 2^-1007 moves no mesh point by a bit"
    );
}
