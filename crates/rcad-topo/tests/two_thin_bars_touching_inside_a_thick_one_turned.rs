//! A thick round bar with a block on one side, holding two thin bars that
//! touch each other along its axis, all turned about `z`.
//!
//! At `Scale::millimetres(100.0)` the thick bar has radius 2 and stands on the
//! `z` axis from `z = −2` to 2. The block is `x` 0 to 5, `y` −1.5 to 1.5, `z`
//! −2 to 2. The thin bars have radius 1 and stand from `z = −3` to 3 with
//! their axes at `(−1, 0)` and `(1, 0)`, so each touches the thick bar's wall
//! along one line and the two touch each other along the `z` axis. All four
//! are turned about `z` by one angle. The block is united with the thick bar
//! first, then the thin bar at `x = −1`, then the one at `x = 1`.
//!
//! Turned, the thin bars' axes are at `c` and `−c` exactly, and at the four
//! turns here `|c|² − 1` is below nought, so the two thin bars overlap by a
//! sliver a rounding thick along the `z` axis. The pipe of the bar at
//! `x = −1` cuts the wall of the bar at `x = 1` along two lines close beside
//! each other, from `z = −2` to 2, and the strip of wall between them lies
//! inside the thick bar. No face of either argument of the last fuse lies on
//! the `x = −1` pipe between `z = −2` and 2, so those two lines are no edges
//! of the answer, and the strip is a piece of its own.
//!
//! No point can be found inside the strip to place it by: Q3241 measured the
//! two lines `2e−8` apart, every box a point is held in `3.4e−13` wide, and
//! the `x = −1` pipe's sign over such a box spanning `7e−13` against a margin
//! near `1e−16`. So the last fuse was refused `Undecidable` (Q3243) until
//! `by_the_piece_across_a_side_clear_of_every_face` in `fuse/select.rs`
//! offered the middle of a side two pipes along one direction cross along,
//! as it already did the middle of a side two planes cross along. The piece
//! across such a side is the rest of the wall, which a point inside places,
//! and the strip is placed with it.
//!
//! Without the block, the bars alone are still refused, `ResultIsNotManifold`.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::expansion::{expansion_sign, expansion_sum, scale_expansion};
use rcad_predicates::{Decision, Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The turns at which Q3241 found the last fuse refused.
const TURNS: [f64; 4] = [0.3, 3.0, 4.69, 4.71];

#[test]
fn a_block_on_a_thick_bar_unites_with_two_thin_bars_overlapping_by_a_rounding_inside_it() {
    for turn in TURNS {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let thick = a_bar(&mut model, 2.0, 4.0, [0.0, 0.0, -2.0], turn);
        let block = the_block(&mut model, turn);
        let left = a_bar(&mut model, 1.0, 6.0, [-1.0, 0.0, -3.0], turn);
        let right = a_bar(&mut model, 1.0, 6.0, [1.0, 0.0, -3.0], turn);

        let c = where_its_axis_crosses_z_nought(&model, right);
        assert_eq!(
            where_its_axis_crosses_z_nought(&model, left),
            c.map(|x| -x),
            "turned {turn}: the thin bars' axes are at c and −c"
        );
        assert_eq!(
            squared_length_less_one(c),
            Decision::Neg,
            "turned {turn}: the thin bars' axes at {c:?} are under 2 apart, so the bars overlap"
        );

        let with_the_block = united(&mut model, &[thick], &[block], || {
            format!("turned {turn}, the block")
        });
        let with_the_left_bar = united(&mut model, &with_the_block, &[left], || {
            format!("turned {turn}, the bar at x = −1")
        });
        let last = || format!("turned {turn}, the bar at x = 1");
        let [whole] = united(&mut model, &with_the_left_bar, &[right], last)[..] else {
            panic!("{}: the four unite into one solid", last());
        };
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("turned {turn}: {why}"));

        let measured = model
            .solid(whole)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.001).expect("a fine mesh"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refused| panic!("turned {turn}: {refused:?}"));
        let meant = the_volume();
        assert!(
            (measured.volume() - meant).abs() <= measured.volume_error(),
            "turned {turn}: {} ± {} against {meant}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// The thick bar and the block, `4` tall: the disk of radius 2 and the
/// rectangle `5 × 3`, less the part of the disk with `x ≥ 0` and `|y| ≤ 1.5`,
/// whose area is `1.5 √1.75 + 4 asin 0.75`. Each thin bar adds its two ends
/// of length 1 past `z = ±2`; between them it is inside the thick bar, and the
/// sliver the two share holds no volume a measure can see.
fn the_volume() -> f64 {
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
