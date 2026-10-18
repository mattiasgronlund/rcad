//! A cube from `(3, 3, 0)` to `(7, 7, 4)` and the ball of radius 2 centred at
//! `(5, 5, 2)`, which touches each of the cube's six faces at its middle.
//!
//! **Every point offered on the ball's skin was on the cube's.** The ball's one
//! face has its two poles for corners, and their middle is the ball's centre.
//! The six steps of one radius from there along the axes, put back onto the
//! ball, land on the six points where it touches the cube, and a point on the
//! cube's skin places nothing about a face that does not lie on it. So every
//! fuse of the two, either way round and under a certified policy or a
//! tolerant one, was refused `Undecidable` at *a point in a piece that every
//! solid could place*, naming the two poles (queue row Q2116). A gap and not a
//! limit (§16.10): every point of the ball off those six is inside the cube.
//! Since that row a ball's face no other rule places is asked at the eight
//! steps from the middle of its corners towards the corners of the box round
//! it, and the first of them, `(5 + 2/√3, 5 + 2/√3, 2 + 2/√3)`, is inside the
//! cube.
//!
//! Each answer is checked against what the two hold, worked out by hand: the
//! cube's 64 and the ball's `32π/3`.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The cube and the ball united, either way round, are the cube.
#[test]
fn a_cube_united_with_the_ball_inscribed_in_it_is_the_cube() {
    for cube_first in [true, false] {
        the_one_solid(Rule::Union, cube_first, &[6], 64.0);
    }
}

/// Their intersection, either way round, is the ball.
#[test]
fn a_cube_and_the_ball_inscribed_in_it_have_the_ball_in_common() {
    for cube_first in [true, false] {
        the_one_solid(Rule::Intersection, cube_first, &[1], 32.0 * PI / 3.0);
    }
}

/// The cube less the ball is one solid of two shells, the cube's six faces and
/// the ball's one face as a hollow touching each of them at one point.
#[test]
fn a_cube_less_the_ball_inscribed_in_it_keeps_the_ball_as_a_hollow() {
    the_one_solid(Rule::Difference, true, &[1, 6], 64.0 - 32.0 * PI / 3.0);
}

/// The ball less the cube is nothing.
#[test]
fn the_ball_inscribed_in_a_cube_less_the_cube_is_nothing() {
    for (name, policy) in policies() {
        let mut model = a_model();
        let (cube, ball) = the_cube_and_its_ball(&mut model);
        let answer = model
            .fuse(
                &[ball],
                &[cube],
                Rule::Difference,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{name}: refused {why:?}"));
        assert!(
            answer.solids().is_empty(),
            "{name}: the ball less the cube is nothing, not {:?}",
            answer.solids()
        );
    }
}

/// The cube and the ball fused by `rule`, the cube first or the ball, under a
/// certified policy and a tolerant one: one solid, its shells holding `faces`
/// faces from fewest to most, whose volume is `by_hand` to within what the
/// mesh could be hiding.
fn the_one_solid(rule: Rule, cube_first: bool, faces: &[usize], by_hand: f64) {
    for (name, policy) in policies() {
        let mut model = a_model();
        let (cube, ball) = the_cube_and_its_ball(&mut model);
        let (one, other) = if cube_first {
            (cube, ball)
        } else {
            (ball, cube)
        };
        let answer = model
            .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| {
                panic!("{name}: {rule:?}, cube first {cube_first}, refused {why:?}")
            });
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{name}: {rule:?}: the model does not validate: {why:?}"));
        let [solid] = answer.solids()[..] else {
            panic!(
                "{name}: {rule:?}, cube first {cube_first}, is one solid, not {:?}",
                answer.solids()
            );
        };
        let view = model.solid(solid).expect("this model made the key");
        let mut counted: Vec<usize> = view.shells().map(|shell| shell.face_count()).collect();
        counted.sort_unstable();
        assert_eq!(
            counted, faces,
            "{name}: {rule:?}, cube first {cube_first}: the faces of each shell"
        );
        let measured = view
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a solid measures");
        assert!(
            (measured.volume() - by_hand).abs() <= measured.volume_error(),
            "{name}: {rule:?}, cube first {cube_first}: measured {} ± {} against {by_hand} by hand",
            measured.volume(),
            measured.volume_error()
        );
    }
}

fn policies() -> [(&'static str, Policy); 2] {
    [
        ("certified", Policy::certified()),
        ("tolerant", Policy::tolerant(100.0)),
    ]
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// The cube from `(3, 3, 0)` to `(7, 7, 4)` and the ball of radius 2 at its
/// middle.
fn the_cube_and_its_ball(model: &mut Model) -> (SolidKey, SolidKey) {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let cube = model
        .add_extrusion(&square, 4.0, &at(3.0, 3.0, 0.0))
        .expect("a positive height");
    let ball = model
        .add_ball(2.0, &at(5.0, 5.0, 2.0))
        .expect("a ball of positive radius");
    (cube, ball)
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
