//! A ball of radius 1 less a round bar standing wholly inside it is the ball
//! with a hollow: the ball's skin in one face, and the bar's pipe and its two
//! caps facing in (`docs/decisions.md` §400.2).
//!
//! A bar of radius 0.2 and height 0.6 on the ball's axis came back so, 4
//! faces. The same bar moved off to `(0.4, 0)` came back with 8, the ball's
//! skin in 5 pieces, and united with the ball it left the ball in 5 faces.
//! Nothing proved the ball's skin misses the bar's faces: the band round the
//! bar's axis was asked only of a ball centred on that axis.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

const BALL: f64 = 4.0 * PI / 3.0;
const RADIUS: f64 = 0.2;
const HEIGHT: f64 = 0.6;

#[test]
fn a_ball_less_a_bar_standing_off_its_axis_inside_it_is_four_faces() {
    assert_the_ball_with_a_hollow(&centred(), "the bar on the ball's axis");
    assert_the_ball_with_a_hollow(&off(0.4).compose(&centred()), "the bar at (0.4, 0)");
}

/// At `(0.75, 0)` the rim of either cap reaches `(0.95, 0, ±0.3)`, `√0.9925`
/// out, and the ball still holds it. The ball holds the bar exactly where it
/// holds the rims of its caps, and the proof asks that and no more.
#[test]
fn a_ball_less_a_bar_whose_rims_nearly_reach_its_skin_is_four_faces() {
    assert_the_ball_with_a_hollow(&off(0.75).compose(&centred()), "the bar at (0.75, 0)");
}

#[test]
fn a_ball_united_with_a_bar_standing_off_its_axis_inside_it_is_the_ball() {
    assert_the_ball_alone(&centred(), "the bar on the ball's axis");
    assert_the_ball_alone(&off(0.4).compose(&centred()), "the bar at (0.4, 0)");
}

/// The bar centred on the ball's own centre, standing along `z`.
fn centred() -> Isometry {
    Isometry::translation(Vector::new(0.0, 0.0, -HEIGHT / 2.0)).expect("a shift")
}

fn off(x: f64) -> Isometry {
    Isometry::translation(Vector::new(x, 0.0, 0.0)).expect("a shift")
}

fn assert_the_ball_with_a_hollow(placement: &Isometry, what: &str) {
    let (mut model, ball, bar) = a_ball_and_a_bar(placement);
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
    assert_measured_near(&model, solid, BALL - PI * RADIUS * RADIUS * HEIGHT, what);
}

/// The ball and the bar united, either way round, are the ball.
fn assert_the_ball_alone(placement: &Isometry, what: &str) {
    for ball_first in [true, false] {
        let what = format!("{what}, ball first {ball_first}");
        let (mut model, ball, bar) = a_ball_and_a_bar(placement);
        let (arguments, tools) = if ball_first { (ball, bar) } else { (bar, ball) };
        let [solid] = fused(&mut model, arguments, tools, Rule::Union, &what)[..] else {
            panic!("{what}: one solid");
        };
        let view = model.solid(solid).expect("this model made the key");
        let surfaces: Vec<Surface> = view.faces().map(|face| face.surface()).collect();
        assert!(
            matches!(surfaces[..], [Surface::Sphere(_)]),
            "{what}: the ball in one face, got {} faces",
            surfaces.len()
        );
        assert_measured_near(&model, solid, BALL, &what);
    }
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

/// A ball of radius 1 at the origin, and a bar of radius 0.2 and height 0.6
/// placed with `placement`.
fn a_ball_and_a_bar(placement: &Isometry) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let ball = model
        .add_ball(1.0, &Isometry::identity())
        .expect("a ball of radius 1");
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, placement)
        .expect("a bar");
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
