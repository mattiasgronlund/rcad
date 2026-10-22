//! A blind hole drilled into the top of a round bar, off its axis.
//!
//! The bar has radius 1 and height 4. The hole is a bar of radius 0.1 and
//! height 1 standing at `z = 3.5`, so it reaches half a unit down into the bar
//! and half a unit out above it. Its axis stands at radius 0.8 from the bar's.
//! The answer is the bar with a pocket in its top: the top, the wall and the
//! bottom of the bar, and the wall and the floor of the pocket, five faces.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

#[test]
fn a_blind_hole_at_an_eighth_turn_round_the_bars_axis() {
    the_bar_less_a_blind_hole(1.0, 4.0, 0.1, 0.8, PI / 4.0);
}

#[test]
fn a_blind_hole_on_the_bars_seam_side() {
    the_bar_less_a_blind_hole(1.0, 4.0, 0.1, 0.8, 0.0);
}

#[test]
fn a_blind_hole_at_a_sixteenth_turn_round_the_bars_axis() {
    the_bar_less_a_blind_hole(1.0, 4.0, 0.1, 0.8, PI / 8.0);
}

#[test]
fn a_blind_hole_at_a_quarter_turn_round_the_bars_axis() {
    the_bar_less_a_blind_hole(1.0, 4.0, 0.1, 0.8, PI / 2.0);
}

#[test]
fn a_smaller_blind_hole_nearer_the_bars_wall() {
    the_bar_less_a_blind_hole(1.0, 4.0, 0.0625, 0.875, PI / 8.0);
}

#[test]
fn a_tiny_blind_hole_close_to_the_wall_of_a_shorter_bar() {
    the_bar_less_a_blind_hole(0.99, 3.9, 0.02, 0.92, PI / 4.0);
}

#[test]
fn a_tiny_blind_hole_close_to_the_wall_of_a_shorter_bar_at_a_sixteenth_turn() {
    the_bar_less_a_blind_hole(0.99, 3.9, 0.02, 0.92, PI / 8.0);
}

fn the_bar_less_a_blind_hole(
    bar_radius: f64,
    bar_height: f64,
    hole_radius: f64,
    off_axis: f64,
    angle: f64,
) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(bar_radius, bar_height, &Isometry::identity())
        .expect("the bar");
    let hole = model
        .add_round_bar(
            hole_radius,
            1.0,
            &at(
                off_axis * angle.cos(),
                off_axis * angle.sin(),
                bar_height - 0.5,
            ),
        )
        .expect("the hole");
    let answer = model
        .fuse(
            &[bar],
            &[hole],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the bar less the hole refused: {why}"));
    let [solid] = answer.solids() else {
        panic!(
            "the bar less the hole is one solid, not {:?}",
            answer.solids()
        );
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the bar less the hole does not validate: {why}"));
    let view = model.solid(*solid).expect("this model made the key");
    let faces: usize = view.shells().map(|shell| shell.face_count()).sum();
    assert_eq!(faces, 5, "the bar's three faces and the pocket's two");
    let volume = PI * (bar_radius * bar_radius * bar_height - hole_radius * hole_radius * 0.5);
    the_volume_is(&model, *solid, volume);
}

fn the_volume_is(model: &Model, solid: SolidKey, volume: f64) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the bar less the hole does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "the bar less the hole holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
