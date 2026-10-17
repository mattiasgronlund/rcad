//! A ball of radius 1 less a prism lying wholly inside it is the ball with a
//! hollow: the ball's skin in one face and the prism's faces facing in
//! (`docs/decisions.md` §400.2).
//!
//! A cube of side 0.9 square to the axes came back so, 7 faces. The same cube
//! turned an eighth of a turn about `z` came back with the ball's face in 26
//! pieces, and a 12-sided prism of radius 0.81 and height 1 with it in 98,
//! under union as well as difference. The fuse cut the ball against the prism
//! because nothing proved the ball's skin misses the prism's top: the box round
//! the top's corners reaches `(r, r, h/2)`, outside the ball, though every
//! corner of the top is inside it. Every plane of the prism then cut the ball's
//! face in a circle.
//!
//! A flat face whose every side is straight lies in the hull of its corners,
//! and a ball holds that hull once it holds every corner. That is now a proof
//! that the ball's skin misses the face.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const BALL: f64 = 4.0 * PI / 3.0;

/// The row's cube: corners `(±0.636, 0)` and `(0, ±0.636)`, from `z = -0.45`
/// to `0.45`. The box round its top reaches `(0.636, 0.636, 0.45)`, 1.0 out
/// and a little more, while every corner is 0.78 out.
#[test]
fn a_ball_less_a_cube_turned_an_eighth_turn_inside_it_is_seven_faces() {
    let half = 0.45;
    let square = vec![[-half, -half], [half, -half], [half, half], [-half, half]];
    let turned = vec![[0.636, 0.0], [0.0, 0.636], [-0.636, 0.0], [0.0, -0.636]];
    for (what, outline, area) in [
        ("the cube square to the axes", square, 0.81),
        (
            "the cube turned an eighth turn",
            turned,
            2.0 * 0.636 * 0.636,
        ),
    ] {
        assert_the_ball_with_a_hollow(outline, 2.0 * half, area * 2.0 * half, what);
    }
}

/// The row's 12-sided prism, radius 0.81 and height 1, whose every corner is
/// `√(0.81² + 0.5²)`, 0.95, out.
#[test]
fn a_ball_less_a_twelve_sided_prism_inside_it_is_fifteen_faces() {
    let what = "a 12-sided prism of radius 0.81";
    assert_the_ball_with_a_hollow(twelve_sides(0.81, 0.0), 1.0, 3.0 * 0.81 * 0.81, what);
}

/// A 12-sided prism of height 0.6 has its top's box leave the ball at
/// `2r² + 0.09 = 1`, `r = 0.6745`, started at a corner on `x`, and at
/// `r = 0.6983` started a twenty-fourth of a turn round. Its corners stay in
/// until `r² + 0.09 = 1`, `r = 0.9539`. Each radius here came back in 136
/// faces past the first two marks.
#[test]
fn a_ball_less_a_twelve_sided_prism_keeps_one_face_until_the_corners_leave_it() {
    for (radius, start) in [
        (0.674, 0.0),
        (0.675, 0.0),
        (0.95, 0.0),
        (0.698, PI / 12.0),
        (0.699, PI / 12.0),
    ] {
        let what = format!("a 12-sided prism of radius {radius} started at {start}");
        let area = 3.0 * radius * radius;
        assert_the_ball_with_a_hollow(twelve_sides(radius, start), 0.6, area * 0.6, &what);
    }
}

/// The ball and the prism inside it united, either way round, are the ball.
#[test]
fn a_ball_united_with_a_turned_cube_or_a_prism_inside_it_is_the_ball() {
    let turned = vec![[0.636, 0.0], [0.0, 0.636], [-0.636, 0.0], [0.0, -0.636]];
    for (what, outline, height) in [
        ("the turned cube", turned, 0.9),
        ("the 12-sided prism", twelve_sides(0.81, 0.0), 1.0),
    ] {
        for ball_first in [true, false] {
            let what = format!("{what}, ball first {ball_first}");
            let (mut model, ball, prism) = a_ball_and_a_prism(outline.clone(), height);
            let (arguments, tools) = if ball_first {
                (ball, prism)
            } else {
                (prism, ball)
            };
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
}

fn assert_the_ball_with_a_hollow(outline: Vec<[f64; 2]>, height: f64, hollow: f64, what: &str) {
    let sides = outline.len();
    let (mut model, ball, prism) = a_ball_and_a_prism(outline, height);
    let [solid] = fused(&mut model, ball, prism, Rule::Difference, what)[..] else {
        panic!("{what}: one solid");
    };
    let view = model.solid(solid).expect("this model made the key");
    let on_the_ball = view
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Sphere(_)))
        .count();
    let flat = view
        .faces()
        .filter(|face| face.surface().as_plane().is_some())
        .count();
    assert_eq!(
        (
            on_the_ball,
            flat,
            view.faces().count(),
            view.shells().count()
        ),
        (1, sides + 2, sides + 3, 2),
        "{what}: the ball's skin in one face, the prism's faces facing in, two shells"
    );
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

fn twelve_sides(radius: f64, start: f64) -> Vec<[f64; 2]> {
    (0..12)
        .map(|corner| {
            let angle = start + 2.0 * PI * f64::from(corner) / 12.0;
            [radius * angle.cos(), radius * angle.sin()]
        })
        .collect()
}

/// A ball of radius 1 at the origin, and a prism on `outline` from
/// `z = -height/2` to `height/2`.
fn a_ball_and_a_prism(outline: Vec<[f64; 2]>, height: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let ball = model
        .add_ball(1.0, &Isometry::identity())
        .expect("a ball of radius 1");
    let profile = ProfileBuilder::new(outline)
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a polygon");
    let at = Isometry::translation(Vector::new(0.0, 0.0, -height / 2.0)).expect("a finite shift");
    let prism = model.add_extrusion(&profile, height, &at).expect("a prism");
    (model, ball, prism)
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
