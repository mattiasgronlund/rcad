//! A thick round bar holding two thin bars that each touch its wall from
//! inside, all turned about `z`, at the turns where rounding pushes each thin
//! bar a hair out through the wall.
//!
//! At `Scale::millimetres(100.0)` the thick bar has radius 2 and stands on the
//! `z` axis from `z = −2` to 2. The thin bars have radius 1 and stand from
//! `z = −3` to 3 with their axes at `(−1, 0)` and `(1, 0)`, so each touches
//! the thick bar's wall along one line and the two touch each other along the
//! `z` axis. All three are turned about `z` by one angle and united, the thick
//! bar with the thin bar at `x = −1` first and that with the one at `x = 1`
//! second. The same is done with the block `[0, 5] × [−1.5, 1.5] × [−2, 2]`,
//! turned with them, united with the thick bar before either thin bar.
//!
//! Turned, the thin bars' axes are at `c` and `−c` exactly, and at every
//! turn here `|c|² − 1` is above nought (`4.8e−17` at turn 1). So the two
//! thin bars stand apart by a hair, and each pokes out of the thick bar's
//! wall by about `2.4e−17`: the `x = −1` pipe crosses the thick bar's rims at
//! two corners about `2e−8` apart. The first union's cap at `z = −2` is the
//! disk of the thick bar less the `x = −1` disk, and its rim edge runs the
//! long way round between those two corners.
//!
//! Q3241 found the second union refused `AFaceTrimmedByACurveThisCannotFollow`
//! (Q3245): arranging that cap, nothing could tell which arc of the rim its
//! rim edge is, though its ends are `2e−8` apart and each is held in a box
//! `1.6e−15` wide.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::expansion::{expansion_sign, expansion_sum, scale_expansion};
use rcad_predicates::{Decision, Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The turns at which Q3241 found the second union refused, and turn 4.9,
/// which is `49.0 * 0.1`. There the block's second union was refused until
/// Q3264 (Q3265), and after it all three went through and the measure was
/// refused `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 1, used: 1 }` until
/// Q3266 (Q3274).
const TURNS: [f64; 5] = [1.0, 2.0, 4.9, 5.0, 6.0];

/// [`TURNS`], turn 0.2, where `|c|² − 1` is `1.8e−18` and the corners are
/// `3.8e−9` apart, and the turns `n · 0.1` for `n` = 4, 31, 49, 54, 61 and
/// 62, at which Q3262 found the second union of the bars alone refused (rows
/// Q3264 and Q3265). Turn 61 is written as that product, which is
/// `6.1000000000000005` and not the `f64` nearest 6.1.
const TURNS_OF_THE_BARS_ALONE: [f64; 11] =
    [0.2, 0.4, 1.0, 2.0, 3.1, 4.9, 5.0, 5.4, 6.0, 61.0 * 0.1, 6.2];

#[test]
fn a_thick_bar_unites_with_two_thin_bars_each_a_hair_out_of_its_wall() {
    for turn in TURNS_OF_THE_BARS_ALONE {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let thick = a_bar(&mut model, 2.0, 4.0, [0.0, 0.0, -2.0], turn);
        let [left, right] = the_thin_bars(&mut model, turn);

        let with_the_left_bar = united(&mut model, &[thick], &[left], || {
            format!("turned {turn}, the bar at x = −1")
        });
        let whole = the_last_union(&mut model, &with_the_left_bar, right, turn);
        it_measures(&model, whole, turn, 20.0 * PI);
    }
}

#[test]
fn a_block_on_a_thick_bar_unites_with_two_thin_bars_each_a_hair_out_of_its_wall() {
    for turn in TURNS {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let thick = a_bar(&mut model, 2.0, 4.0, [0.0, 0.0, -2.0], turn);
        let block = the_block(&mut model, turn);
        let [left, right] = the_thin_bars(&mut model, turn);

        let with_the_block = united(&mut model, &[thick], &[block], || {
            format!("turned {turn}, the block")
        });
        let with_the_left_bar = united(&mut model, &with_the_block, &[left], || {
            format!("turned {turn}, the bar at x = −1")
        });
        let whole = the_last_union(&mut model, &with_the_left_bar, right, turn);
        it_measures(&model, whole, turn, the_volume_with_the_block());
    }
}

/// The thin bars at `x = −1` and `x = 1`, turned by `turn`, checked to stand
/// apart by a hair.
fn the_thin_bars(model: &mut Model, turn: f64) -> [SolidKey; 2] {
    let left = a_bar(model, 1.0, 6.0, [-1.0, 0.0, -3.0], turn);
    let right = a_bar(model, 1.0, 6.0, [1.0, 0.0, -3.0], turn);
    let c = where_its_axis_crosses_z_nought(model, right);
    assert_eq!(
        where_its_axis_crosses_z_nought(model, left),
        c.map(|x| -x),
        "turned {turn}: the thin bars' axes are at c and −c"
    );
    assert_eq!(
        squared_length_less_one(c),
        Decision::Pos,
        "turned {turn}: the thin bars' axes at {c:?} are over 2 apart, so each pokes out of \
         the thick bar's wall"
    );
    [left, right]
}

/// The union of `so_far` with the thin bar at `x = 1`, which must be one
/// solid in a model that validates.
fn the_last_union(model: &mut Model, so_far: &[SolidKey], right: SolidKey, turn: f64) -> SolidKey {
    let last = || format!("turned {turn}, the bar at x = 1");
    let [whole] = united(model, so_far, &[right], last)[..] else {
        panic!("{}: the bars unite into one solid", last());
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("turned {turn}: {why}"));
    whole
}

fn it_measures(model: &Model, whole: SolidKey, turn: f64, meant: f64) {
    let measured = model
        .solid(whole)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("turned {turn}: {refused:?}"));
    assert!(
        (measured.volume() - meant).abs() <= measured.volume_error(),
        "turned {turn}: {} ± {} against {meant}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The thick bar and the block, `4` tall: the disk of radius 2 and the
/// rectangle `5 × 3`, less the part of the disk with `x ≥ 0` and `|y| ≤ 1.5`,
/// whose area is `1.5 √1.75 + 4 asin 0.75`. Each thin bar adds its two ends
/// of length 1 past `z = ±2`; between them it is inside the thick bar but for
/// a sliver no measure can see.
fn the_volume_with_the_block() -> f64 {
    let disk = 4.0 * PI;
    let rectangle = 15.0;
    let both = 1.5 * 1.75_f64.sqrt() + 4.0 * 0.75_f64.asin();
    4.0 * (disk + rectangle - both) + 4.0 * PI
}

/// The union of `one` and `other`, panicking with `which` when it is refused.
fn united(
    model: &mut Model,
    one: &[SolidKey],
    other: &[SolidKey],
    which: impl Fn() -> String,
) -> Vec<SolidKey> {
    model
        .fuse(
            one,
            other,
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{}: {refused:?}", which()))
        .solids()
        .to_vec()
}

fn a_bar(model: &mut Model, radius: f64, height: f64, foot: [f64; 3], turn: f64) -> SolidKey {
    model
        .add_round_bar(radius, height, &turned(turn, foot))
        .expect("a bar")
}

fn the_block(model: &mut Model, turn: f64) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[0.0, -1.5], [5.0, -1.5], [5.0, 1.5], [0.0, 1.5]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    model
        .add_extrusion(&rectangle, 4.0, &turned(turn, [0.0, 0.0, -2.0]))
        .expect("a block")
}

/// A move to `at`, then the turn about `z`.
fn turned(turn: f64, at: [f64; 3]) -> Isometry {
    Isometry::rotation(&Axis::z(), turn)
        .expect("a finite angle")
        .compose(&Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite move"))
}

/// `x` and `y` of the axis of `bar`'s pipe, as the model stores it.
fn where_its_axis_crosses_z_nought(model: &Model, bar: SolidKey) -> [f64; 2] {
    let pipe = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .find_map(|face| {
            let Surface::Cylinder(pipe) = face.surface() else {
                return None;
            };
            Some(pipe)
        })
        .expect("a bar has a pipe");
    let axis = pipe.axis();
    assert_eq!(
        axis.direction().components(),
        [0.0, 0.0, 1.0],
        "a bar turned about z still stands along it"
    );
    let origin = axis.origin();
    [origin.x(), origin.y()]
}

/// The sign of `|c|² − 1`, decided exactly.
fn squared_length_less_one(c: [f64; 2]) -> Decision {
    let mut sum = vec![-1.0];
    for x in c {
        let mut square = [0.0; 2];
        let squared = scale_expansion(&[x], x, &mut square);
        let mut wider = vec![0.0; sum.len() + squared];
        let len = expansion_sum(&sum, &square[..squared], &mut wider);
        wider.truncate(len);
        sum = wider;
    }
    expansion_sign(&sum)
}
