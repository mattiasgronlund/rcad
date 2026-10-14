//! A cube set corner to corner on another cube's top corner, turned a little
//! short of a half turn about an axis leaning off `z` (queue rows Q2007, Q2008,
//! Q2009 and Q2019).
//!
//! The block is `add_cube(10.0)`, `[0, 10]³`. The turned cube is a second
//! `add_cube(10.0)` placed by `Isometry::try_from(([0, 0, 10], q))` with
//! `q = [sin p, 0, cos p · sin t, cos p · cos t]`: a turn `2p` short of a half
//! turn about the axis `(0, sin t, cos t)`, which leans `t` from `z` towards
//! `y`, then a move by `(0, 0, 10)`. Its corner at the origin lands on the
//! block's top corner `(0, 0, 10)`, and its three edges there run off along
//! `−x`, `−y` and `+z`, each a little off. A point of the turned cube with
//! `x ≥ 0` must be far up its `+z` edge, and there it is above the block's
//! top, so the two cubes meet only at that corner. So every answer here is
//! worked by hand: each cube is `1000`, and what they share has no volume.
//!
//! **Each of these panicked under `Policy::tolerant(100.0)`**, at three
//! different assertions in `fuse/assemble.rs`. The turned corner is stored
//! `1.2e−17` to `3.8e−17` from `(0, 0, 10)`, and all six planes there pass
//! within a rounding of it. So one cube's plane cuts the other cube's three
//! edges a rounding from that cube's corner, and
//! `fold_each_corner_one_plane_cuts_a_rounding_from_it` took the three meets
//! as that corner, while the other cube's own corner, on one or none of the
//! meets' planes, was as near. The two corners were then named apart at one
//! point, and joining the faces round them panicked. The fold is now left
//! alone where any other input corner is within the policy's wobble of every
//! meet (`docs/decisions.md` §417.7).

use rcad_geom::{Deflection, Isometry, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

/// `p = 0.005` and `t = 0.02`, as `f64::sin` and `f64::cos` give them:
/// a turn `0.01` short of a half turn about an axis leaning `0.02` (Q2008).
/// The union panicked "an input edge a parted line is a piece of runs along
/// one of its faces", both ways round.
const A_HUNDREDTH_SHORT_LEANING_TWO_HUNDREDTHS: [f64; 4] = [
    0.004_999_979_166_692_708,
    0.0,
    0.019_998_416_710_520_21,
    0.999_787_509_192_530_9,
];

/// The turn of a fuzz crash file's second half, its parts under `1e-100`
/// left out (Q2007): `7.5e−5` short of a half turn about an axis leaning
/// `0.0024`. Every fuse of it below panicked "a piece of a face has a straight
/// side with no length". `Policy::certified()` answered all of them.
const FROM_A_FUZZ_CRASH_FILE: [f64; 4] = [
    3.756_056_093_936_948_4e-5,
    0.0,
    0.002_401_491_721_205_327,
    0.999_997_115_709_199_1,
];

/// `p = 0.0005` and `t = 0.003` and `0.01`, as `f64::sin` and `f64::cos` give
/// them: a turn `0.001` short of a half turn about an axis leaning `0.003` or
/// `0.01` (Q2009). The turned cube less the block panicked "every face joined
/// here brought an outline of its own", the panic of Q1780's fuzz group.
const A_THOUSANDTH_SHORT: [[f64; 4]; 2] = [
    [
        0.000_499_999_979_166_666_9,
        0.0,
        0.002_999_995_125_002_595_6,
        0.999_995_375_003_940_2,
    ],
    [
        0.000_499_999_979_166_666_9,
        0.0,
        0.009_999_832_084_187_523,
        0.999_949_875_422_917_9,
    ],
];

#[test]
fn the_union_of_two_cubes_corner_to_corner_is_both_cubes_either_way_round() {
    for turn in [
        A_HUNDREDTH_SHORT_LEANING_TWO_HUNDREDTHS,
        FROM_A_FUZZ_CRASH_FILE,
    ] {
        for order in [Order::BlockFirst, Order::TurnedFirst] {
            let asked = format!("{turn:?}, union, {order:?}");
            let volume = volume_of_a_fuse(turn, Rule::Union, order, &asked);
            assert_about(volume, 2000.0, &asked);
        }
    }
}

/// Given together as the arguments, with no tools, the two cubes are both
/// cubes whether the rule is a union or a difference: what is in the
/// arguments and not in the tools is all of them.
#[test]
fn two_cubes_corner_to_corner_given_together_as_the_arguments_are_both_cubes() {
    for (turn, rule) in [
        (A_HUNDREDTH_SHORT_LEANING_TWO_HUNDREDTHS, Rule::Difference),
        (FROM_A_FUZZ_CRASH_FILE, Rule::Union),
        (FROM_A_FUZZ_CRASH_FILE, Rule::Difference),
    ] {
        let asked = format!("{turn:?}, {rule:?}, both as arguments");
        let volume = volume_of_a_fuse(turn, rule, Order::BothAsArguments, &asked);
        assert_about(volume, 2000.0, &asked);
    }
}

#[test]
fn the_turned_cube_less_the_block_is_the_turned_cube() {
    for turn in std::iter::once(FROM_A_FUZZ_CRASH_FILE).chain(A_THOUSANDTH_SHORT) {
        let asked = format!("{turn:?}, the turned cube less the block");
        let volume = volume_of_a_fuse(turn, Rule::Difference, Order::TurnedFirst, &asked);
        assert_about(volume, 1000.0, &asked);
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
    BothAsArguments,
}

/// The volume of the answer under `Policy::tolerant(100.0)`, once the model it
/// is written into validates.
fn volume_of_a_fuse(turn: [f64; 4], rule: Rule, order: Order, asked: &str) -> f64 {
    let policy = Policy::tolerant(100.0);
    let mut model = Model::new(Scale::millimetres(100.0).expect("a scale"));
    let block = a_cube(&mut model);
    let turned = a_cube(&mut model);
    let placement = Isometry::try_from(([0.0, 0.0, 10.0], turn)).expect("a placement");
    model.place_solid(turned, &placement).expect("a placement");
    let (arguments, tools): (&[SolidKey], &[SolidKey]) = match order {
        Order::BlockFirst => (&[block], &[turned]),
        Order::TurnedFirst => (&[turned], &[block]),
        Order::BothAsArguments => (&[block, turned], &[]),
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
