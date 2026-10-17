//! A ball of radius 1 less a round bar standing wholly inside it is the ball
//! with a hollow: the ball's skin in one face, and the bar's pipe and its two
//! caps facing in (`docs/decisions.md` §400.2).
//!
//! A bar of radius 0.2 and height 0.6 on the ball's axis came back so, 4
//! faces. The same bar moved off to `(0.4, 0)` came back with 8, the ball's
//! skin in 5 pieces.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

const BALL: f64 = 4.0 * PI / 3.0;

#[test]
fn a_ball_less_a_bar_standing_off_its_axis_inside_it_is_four_faces() {
    for (what, centre) in [
        ("the bar on the ball's axis", [0.0, 0.0]),
        ("the bar at (0.4, 0)", [0.4, 0.0]),
    ] {
        assert_the_ball_with_a_hollow(centre, 0.2, 0.6, what);
    }
}

fn assert_the_ball_with_a_hollow(centre: [f64; 2], radius: f64, height: f64, what: &str) {
    let (mut model, ball, bar) = a_ball_and_a_bar(centre, radius, height);
    let [solid] = fused(&mut model, ball, bar, Rule::Difference, what)[..] else {
        panic!("{what}: one solid");
    };
    let view = model.solid(solid).expect("this model made the key");
    let on_the_ball = view
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Sphere(_)))
        .count();
    let on_the_pipe = view
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Cylinder(_)))
        .count();
    let flat = view
        .faces()
        .filter(|face| face.surface().as_plane().is_some())
        .count();
    assert_eq!(
        (
            on_the_ball,
            on_the_pipe,
            flat,
            view.faces().count(),
            view.shells().count()
        ),
        (1, 1, 2, 4, 2),
        "{what}: the ball's skin in one face, the bar's pipe and caps facing in, two shells"
    );
    let hollow = PI * radius * radius * height;
    assert_measured_near(&model, solid, BALL - hollow, what);
}

fn assert_measured_near(model: &Model, solid: SolidKey, truth: f64, what: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the solid measures: {why:?}"));
    assert!(
        (measured.volume() - truth).abs() <= measured.volume_error(),
        "{what}: {} measured, {} off, against {truth}",
        measured.volume(),
        measured.volume_error()
    );
}

/// A ball of radius 1 at the origin, and a round bar of `radius` whose axis
/// runs along `z` through `centre`, from `z = -height/2` to `height/2`.
fn a_ball_and_a_bar(centre: [f64; 2], radius: f64, height: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let ball = model
        .add_ball(1.0, &Isometry::identity())
        .expect("a ball of radius 1");
    let at = Isometry::translation(Vector::new(centre[0], centre[1], -height / 2.0))
        .expect("a finite shift");
    let bar = model.add_round_bar(radius, height, &at).expect("a bar");
    (model, ball, bar)
}

fn fused(
    model: &mut Model,
    arguments: SolidKey,
    tools: SolidKey,
    rule: Rule,
    what: &str,
) -> Vec<SolidKey> {
    let fused = model
        .fuse(
            &[arguments],
            &[tools],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
    fused.solids().to_vec()
}
