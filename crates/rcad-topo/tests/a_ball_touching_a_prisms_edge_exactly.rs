//! A ball that touches a prism's upright edge, exactly.
//!
//! The prism is the triangle `(0, 0)`, `(4, 3)`, `(−3, 3)` stood 32 tall, so
//! one of its upright edges is the `z` axis, with a wall leaning away from it on
//! either side. A ball of radius `r` centred at `(r, 0, z)` touches that edge at
//! `(0, 0, z)` from `+x`, and a ball of radius 1 centred at `(0, −1, z)` touches
//! it there from `−y`, outside both walls.
//!
//! The two walls cross in the `z` axis and the axis is tangent to the ball, so
//! where the three surfaces meet is a double root. `meet_of_three_surfaces`
//! decided the discriminant zero exactly and then divided `−B` by `A` as
//! intervals, which put the touching point in a box with width in `z`. What a
//! fuse asks next at that point is which way a wall turns against the ball,
//! `orient3d` of three vectors in one plane, whose true value is zero, and a
//! box round a zero decides nothing: every one of the 72 fuses here refused
//! `Undecidable` under a certified policy. The step and the point are now
//! checked to land on an `f64` and handed back exactly where they do.
//!
//! Past that, union and difference stop at meshing a face, and those two
//! refusals are pinned as they answer below. A tangent point whose `−B/A` is
//! not an `f64`, which a slanted edge touched exactly would make, is not in
//! this file: nobody has run one.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_mesh::CannotTriangulate;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

/// The heights the ball's centre is put at: on the lattice, off it, and near
/// both ends of the prism.
const HEIGHTS: [f64; 8] = [2.5, 3.0, 7.25, 10.1, 16.0, 16.3, 24.7, 29.0];

/// A ball touching the edge from `+x`: its radius, and where its centre is
/// across the floor.
const FROM_PLUS_X: [(f64, [f64; 2]); 2] = [(1.0, [1.0, 0.0]), (2.0, [2.0, 0.0])];

/// A ball touching the edge from `−y`, outside both walls.
const FROM_MINUS_Y: (f64, [f64; 2]) = (1.0, [0.0, -1.0]);

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The regression: no fuse of a ball touching the edge refuses under a
/// certified policy, whichever side it touches from and whatever the rule.
#[test]
fn no_touch_of_the_edge_is_refused_under_a_certified_policy() {
    let mut refused = Vec::new();
    let mut asked = 0;
    for (radius, across) in FROM_PLUS_X.into_iter().chain([FROM_MINUS_Y]) {
        for height in HEIGHTS {
            for rule in EVERY_RULE {
                asked += 1;
                if let (_, Err(Error::Undecidable { records })) =
                    fuse_a_ball_touching_the_edge(radius, across, height, rule)
                {
                    refused.push((radius, across, height, rule, records));
                }
            }
        }
    }
    assert_eq!(asked, 72);
    assert!(
        refused.is_empty(),
        "{} of {asked} touches refused: {refused:#?}",
        refused.len()
    );
}

/// From `+x` the two share a cap of the ball: the wall through `(4, 3)` cuts
/// it `0.6·r` from the centre, so the cap is `0.4·r` deep, and the other wall
/// and both ends of the prism are clear of it. Every one comes back as one
/// solid whose measured volume is within its own published error of that cap.
#[test]
fn from_plus_x_the_intersection_is_the_cap_the_wall_cuts_off_the_ball() {
    for (radius, across) in FROM_PLUS_X {
        let depth = 0.4 * radius;
        let cap = core::f64::consts::PI * depth * depth * (3.0 * radius - depth) / 3.0;
        for height in HEIGHTS {
            let (model, fused) =
                fuse_a_ball_touching_the_edge(radius, across, height, Rule::Intersection);
            let fused = fused.unwrap_or_else(|refused| {
                panic!("radius {radius} at height {height}: {refused:?}")
            });
            let [solid] = fused.solids()[..] else {
                panic!(
                    "radius {radius} at height {height}: one cap, not {:?}",
                    fused.solids()
                );
            };
            let measured = model
                .solid(solid)
                .expect("the fuse made it")
                .measure(
                    Deflection::of(0.001).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a cap measures");
            assert!(
                (measured.volume() - cap).abs() <= measured.volume_error(),
                "radius {radius} at height {height}: {} ± {} is not the cap's {cap}",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
}

/// From `−y` the ball is outside both walls and shares only the touching
/// point with the prism, which holds no volume: the intersection is nothing.
#[test]
fn from_minus_y_the_intersection_is_empty() {
    let (radius, across) = FROM_MINUS_Y;
    for height in HEIGHTS {
        let (_, fused) = fuse_a_ball_touching_the_edge(radius, across, height, Rule::Intersection);
        let fused = fused.unwrap_or_else(|refused| panic!("height {height}: {refused:?}"));
        assert!(
            fused.solids().is_empty(),
            "height {height}: a point is not a solid, and this made {:?}",
            fused.solids()
        );
    }
}

/// Pinned as it answers: union and difference from `+x` stop at meshing a
/// face, `CornersCoincide { position: 2 }` — queue row Q659.
#[test]
fn from_plus_x_union_and_difference_cannot_mesh_a_face() {
    for (radius, across) in FROM_PLUS_X {
        for height in HEIGHTS {
            for rule in [Rule::Union, Rule::Difference] {
                assert_corners_coincide_at(2, radius, across, height, rule);
            }
        }
    }
}

/// Pinned as it answers: union and difference from `−y` stop at meshing a
/// face, `CornersCoincide { position: 4 }`. Nobody has traced it.
#[test]
fn from_minus_y_union_and_difference_cannot_mesh_a_face() {
    let (radius, across) = FROM_MINUS_Y;
    for height in HEIGHTS {
        for rule in [Rule::Union, Rule::Difference] {
            assert_corners_coincide_at(4, radius, across, height, rule);
        }
    }
}

fn assert_corners_coincide_at(
    position: usize,
    radius: f64,
    across: [f64; 2],
    height: f64,
    rule: Rule,
) {
    let (_, fused) = fuse_a_ball_touching_the_edge(radius, across, height, rule);
    match fused {
        Err(Error::CannotMeshAFace {
            why: CannotTriangulate::CornersCoincide { position: found },
            ..
        }) => assert_eq!(
            found, position,
            "radius {radius} from {across:?} at height {height}, {rule:?}"
        ),
        other => panic!(
            "radius {radius} from {across:?} at height {height}, {rule:?}: \
             expected CornersCoincide {{ position: {position} }}, got {other:?}"
        ),
    }
}

/// The model the fuse ran in, and what the fuse said.
fn fuse_a_ball_touching_the_edge(
    radius: f64,
    across: [f64; 2],
    height: f64,
    rule: Rule,
) -> (Model, rcad_topo::Result<Fused>) {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let prism = the_prism(&mut model);
    let ball = model
        .add_ball(
            radius,
            &Isometry::translation(Vector::new(across[0], across[1], height))
                .expect("a finite shift"),
        )
        .expect("a positive radius");
    let fused = model.fuse(
        &[ball],
        &[prism],
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    (model, fused)
}

fn the_prism(model: &mut Model) -> SolidKey {
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 3.0], [-3.0, 3.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a triangle");
    model
        .add_extrusion(&triangle, 32.0, &Isometry::identity())
        .expect("a positive height")
}
