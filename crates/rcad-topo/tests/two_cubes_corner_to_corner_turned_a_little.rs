//! A cube set corner to corner on another cube's top corner, turned `0.05`
//! short of a half turn about an axis leaning `0.01` off `z` (queue rows
//! Q2300 and Q2302, `docs/decisions.md` §437.6).
//!
//! The block is `add_cube(10.0)`, `[0, 10]³`. The turned cube is a second
//! `add_cube(10.0)` placed by `Isometry::try_from(([0, 0, 10], q))` with
//! `q = [sin(p/2), 0, cos(p/2) · sin t, cos(p/2) · cos t]` at `t = 0.01` and
//! `p = 0.05`: a turn `p` short of a half turn about the axis
//! `(0, sin t, cos t)`, then a move by `(0, 0, 10)`. Its corner at the origin
//! lands on the block's top corner `(0, 0, 10)`, and its three edges there run
//! off along `−x`, `−y` and `+z`, each a little off. Near that corner the
//! turned cube keeps to `z ≥ 10 − 0.02 y`, `x ≤ 0.05 y` and
//! `y ≤ 0.02 (z − 10) − 0.05 x`, to first order, and a point of the block,
//! with `x ≥ 0`, `y ≥ 0` and `z ≤ 10`, meets all three only at `(0, 0, 10)`.
//! So every answer here is worked by hand: each cube is `1000`, and what they
//! share has no volume.
//!
//! **Every fuse of the two refused under `Policy::certified()`**, on the order
//! of two points along the turned cube's bottom edge on its `x` face. One is
//! the turned corner, solved from the turned cube's bottom, `x` and `y` faces
//! and stored `1e−16` round `(0, 0, 10)`. The other is the block's top corner,
//! where the block's `x = 0` and `y = 0` faces cross the turned bottom exactly
//! at `(0, 0, 10)`, on that edge's line exactly too. The boxes could not part
//! them, and the planes were asked only when each point was the meet of the
//! edge's two planes and one more, or a corner of the face: the block's corner
//! is neither on this face, so `Arrangement::as_the_planes_say` gave nothing.
//! It now asks where the turned `y` face, which puts the turned corner on the
//! edge, cuts the line, and which side of that face the block's corner lies
//! on, which exact arithmetic says. The turned `y` face runs `7e−18` past
//! `(0, 0, 10)`, so the two are two points, and the refusal was a gap.

use rcad_geom::{Deflection, Isometry, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

/// `p = 0.05` and `t = 0.01`, as `f64::sin` and `f64::cos` give them.
const A_TWENTIETH_SHORT_LEANING_A_HUNDREDTH: [f64; 4] = [
    0.024_997_395_914_712_332,
    0.0,
    0.009_996_708_549_004_05,
    0.999_637_532_316_423_9,
];

#[test]
fn the_union_of_two_cubes_turned_a_little_corner_to_corner_is_both_cubes() {
    for order in [Order::BlockFirst, Order::TurnedFirst] {
        let asked = format!("union, {order:?}");
        assert_about(volume_of_a_fuse(Rule::Union, order, &asked), 2000.0, &asked);
    }
}

#[test]
fn two_cubes_turned_a_little_corner_to_corner_share_no_volume() {
    for order in [Order::BlockFirst, Order::TurnedFirst] {
        let asked = format!("intersection, {order:?}");
        assert_about(
            volume_of_a_fuse(Rule::Intersection, order, &asked),
            0.0,
            &asked,
        );
    }
}

#[test]
fn either_cube_less_the_other_turned_a_little_corner_to_corner_is_the_whole_cube() {
    for order in [Order::BlockFirst, Order::TurnedFirst] {
        let asked = format!("difference, {order:?}");
        assert_about(
            volume_of_a_fuse(Rule::Difference, order, &asked),
            1000.0,
            &asked,
        );
    }
}

fn assert_about(got: f64, worked: f64, asked: &str) {
    assert!(
        (got - worked).abs() <= 1e-9 * worked,
        "{asked}: measured {got}, worked by hand {worked}"
    );
}

#[derive(Clone, Copy, Debug)]
enum Order {
    BlockFirst,
    TurnedFirst,
}

/// The volume of the answer under `Policy::certified()`, once the model it is
/// written into validates.
fn volume_of_a_fuse(rule: Rule, order: Order, asked: &str) -> f64 {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a scale"));
    let block = a_cube(&mut model);
    let turned = a_cube(&mut model);
    let placement = Isometry::try_from(([0.0, 0.0, 10.0], A_TWENTIETH_SHORT_LEANING_A_HUNDREDTH))
        .expect("a placement");
    model.place_solid(turned, &placement).expect("a placement");
    let (arguments, tools): (&[SolidKey], &[SolidKey]) = match order {
        Order::BlockFirst => (&[block], &[turned]),
        Order::TurnedFirst => (&[turned], &[block]),
    };
    let answer = model
        .fuse(arguments, tools, rule, &policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{asked}: {why}"));
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{asked}: {why}"));
    answer
        .solids()
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.1).expect("a deflection"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{asked} does not measure: {why}"))
                .volume()
        })
        .sum()
}

fn a_cube(model: &mut Model) -> SolidKey {
    model
        .add_cube(10.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube")
}
