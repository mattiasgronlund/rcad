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
//! Without the block, the bars alone were refused `ResultIsNotManifold` at the
//! thick bar's bottom rim, which went with the wrong piece of the bottom, until
//! `side_off_a_curve_of_a_pipe_along_it` in `fuse/arrange.rs` read the rim's
//! side of a thin bar's pipe off the circle that pipe cuts the bottom in
//! (Q3244).

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

        let with_the_block = fused(&mut model, &[thick], &[block], Rule::Union, || {
            format!("turned {turn}, the block")
        });
        let with_the_left_bar = fused(&mut model, &with_the_block, &[left], Rule::Union, || {
            format!("turned {turn}, the bar at x = −1")
        });
        let last = || format!("turned {turn}, the bar at x = 1");
        let [whole] = fused(&mut model, &with_the_left_bar, &[right], Rule::Union, last)[..] else {
            panic!("{}: the four unite into one solid", last());
        };
        let deflection = Deflection::of(0.01).expect("a fine mesh");
        valid_and_of_volume(&model, whole, the_volume(), deflection, last);
    }
}

/// The union is the thick bar and the four ends of length 1, `16π + 4π`.
#[test]
fn a_thick_bar_unites_with_two_thin_bars_overlapping_by_a_rounding_inside_it() {
    the_bars_alone_fused_by(Rule::Union, 20.0 * PI, &TURNS);
}

/// The intersection is the thin bar at `x = 1` between the thick bar's ends,
/// `4π`. At the turn `4.69` it is refused
/// `Invalid(FacesDoNotAgreeWhichWayTheyFace)` once the rim's side is read off
/// the thin circle, so that turn is left out (Q3244).
#[test]
fn a_thick_bar_and_a_thin_one_inside_it_meet_another_overlapping_it_by_a_rounding() {
    the_bars_alone_fused_by(Rule::Intersection, 4.0 * PI, &[0.3, 3.0, 4.71]);
}

/// The difference is the thick bar and the two ends of the bar at `x = −1`,
/// less the bar at `x = 1` between them, `16π + 2π − 4π`.
#[test]
fn a_thick_bar_and_a_thin_one_inside_it_less_another_overlapping_it_by_a_rounding() {
    the_bars_alone_fused_by(Rule::Difference, 14.0 * PI, &TURNS);
}

/// Panics unless, at each of `turns`, the thick bar united with the thin bar
/// at `x = −1` and then fused with the one at `x = 1` by `rule`, with no
/// block, is one solid that validates and measures `meant`.
///
/// The thin bars are inside the thick one from `z = −2` to 2. Where they share
/// their sliver past its ends there is no volume a measure can see.
///
/// The thick bar's bottom rim is a circle of radius 2 round `z`, and the bar
/// at `x = 1` cuts the bottom's plane in the circle of radius 1 round `c`,
/// which is inside the rim and clear of it by `1 − |c|`, about `5e−17`. The
/// rim's one node is the thick bar's seam corner, stored at exactly `2c`,
/// which is `|c|` from that circle's centre and so inside it. Read off that
/// corner, the rim went with the piece of the bottom inside the bar at
/// `x = 1`, the piece outside both thin bars was left without it, and every
/// rule was refused `ResultIsNotManifold` at the four turns (Q3244).
///
/// **Measured at a deflection of `0.01`.** There, and at `0.03`, the mesh of a
/// cap of the union and of the difference was refused `BoundaryCrossesItself`,
/// while `0.013`, `0.02`, `0.05` and `0.1` measured (Q3244). Each thin circle's
/// walk puts a point where it touches the rim, a rounding outside the rim's
/// walk and a few ulp from its corner at `2c` or `−2c`. Both measure since a
/// hole point within a rounding of the rim is pinched onto it (Q3248).
fn the_bars_alone_fused_by(rule: Rule, meant: f64, turns: &[f64]) {
    let deflection = Deflection::of(0.01).expect("a fine mesh");
    for &turn in turns {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let thick = a_bar(&mut model, 2.0, 4.0, [0.0, 0.0, -2.0], turn);
        let left = a_bar(&mut model, 1.0, 6.0, [-1.0, 0.0, -3.0], turn);
        let right = a_bar(&mut model, 1.0, 6.0, [1.0, 0.0, -3.0], turn);
        let with_the_left_bar = fused(&mut model, &[thick], &[left], Rule::Union, || {
            format!("turned {turn}, the bar at x = −1")
        });
        let last = || format!("turned {turn}, {rule:?} with the bar at x = 1");
        let [whole] = fused(&mut model, &with_the_left_bar, &[right], rule, last)[..] else {
            panic!("{}: one solid", last());
        };
        valid_and_of_volume(&model, whole, meant, deflection, last);
    }
}

/// Panics, saying `which`, unless the model validates and `solid` measures
/// `meant` at `deflection` to within the measure's own error.
fn valid_and_of_volume(
    model: &Model,
    solid: SolidKey,
    meant: f64,
    deflection: Deflection,
    which: impl Fn() -> String,
) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{}: {why}", which()));
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(deflection, &Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|refused| panic!("{}: {refused:?}", which()));
    println!(
        "{}: {} ± {} against {meant}",
        which(),
        measured.volume(),
        measured.volume_error()
    );
    assert!(
        (measured.volume() - meant).abs() <= measured.volume_error(),
        "{}: {} ± {} against {meant}",
        which(),
        measured.volume(),
        measured.volume_error()
    );
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

/// `one` and `other` fused by `rule`, panicking with `which` when it is
/// refused.
fn fused(
    model: &mut Model,
    one: &[SolidKey],
    other: &[SolidKey],
    rule: Rule,
    which: impl Fn() -> String,
) -> Vec<SolidKey> {
    model
        .fuse(
            one,
            other,
            rule,
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
