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
//! Past that, union and difference stopped at meshing a face, because the
//! edge meets the ball at one vertex where both walls' circles start and end,
//! and the ball's face between them is a figure eight. The mesher cuts such a
//! wire into the loops that touch there now (`docs/decisions.md` §364), and
//! every rule is held to its closed form below. A tangent point whose `−B/A` is
//! not an `f64`, which a slanted edge touched exactly would make, is not in
//! this file: nobody has run one.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
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

/// From `+x` the union is the prism and the ball less the cap they share, and
/// the ball less the prism is the ball less that cap, each one solid.
///
/// Both were refused `CannotMeshAFace { CornersCoincide { position: 2 } }`
/// until the mesher cut a wire that sets off from one corner twice into the
/// loops that touch there (`docs/decisions.md` §364): the edge meets the ball
/// at one vertex, both walls' circles start and end at it, and the ball's face
/// between them was a figure eight.
#[test]
fn from_plus_x_union_and_difference_measure_the_prism_and_the_ball_less_the_cap() {
    for (radius, across) in FROM_PLUS_X {
        let depth = 0.4 * radius;
        let cap = core::f64::consts::PI * depth * depth * (3.0 * radius - depth) / 3.0;
        let ball = 4.0 / 3.0 * core::f64::consts::PI * radius.powi(3);
        for height in HEIGHTS {
            assert_measures(1, radius, across, height, Rule::Union, PRISM + ball - cap);
            assert_measures(1, radius, across, height, Rule::Difference, ball - cap);
        }
    }
}

/// From `−y` the ball shares only the touching point with the prism: the
/// union is the two of them, two solids, and the ball less the prism is the
/// ball whole.
///
/// Both were refused `CannotMeshAFace { CornersCoincide { position: 4 } }` for
/// the same figure eight as from `+x`, here with both walls' circles outside
/// the walls (`docs/decisions.md` §364).
#[test]
fn from_minus_y_union_and_difference_measure_the_prism_and_the_whole_ball() {
    let (radius, across) = FROM_MINUS_Y;
    let ball = 4.0 / 3.0 * core::f64::consts::PI * radius.powi(3);
    for height in HEIGHTS {
        assert_measures(2, radius, across, height, Rule::Union, PRISM + ball);
        assert_measures(1, radius, across, height, Rule::Difference, ball);
    }
}

/// The prism's volume: a triangle of area 10.5, 32 tall.
const PRISM: f64 = 336.0;

/// Assert the fuse makes `solids` solids that measure `should` between them,
/// within the errors they publish.
fn assert_measures(
    solids: usize,
    radius: f64,
    across: [f64; 2],
    height: f64,
    rule: Rule,
    should: f64,
) {
    let (model, fused) = fuse_a_ball_touching_the_edge(radius, across, height, rule);
    let fused = fused.unwrap_or_else(|refused| {
        panic!("radius {radius} from {across:?} at height {height}, {rule:?}: {refused:?}")
    });
    assert_eq!(
        fused.solids().len(),
        solids,
        "radius {radius} from {across:?} at height {height}, {rule:?}"
    );
    let (mut volume, mut error) = (0.0, 0.0);
    for &solid in fused.solids() {
        let measured = model
            .solid(solid)
            .expect("the fuse made it")
            .measure(
                Deflection::of(0.001).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a solid this size measures");
        volume += measured.volume();
        error += measured.volume_error();
    }
    assert!(
        (volume - should).abs() <= error,
        "radius {radius} from {across:?} at height {height}, {rule:?}: \
         {volume} ± {error} is not {should}"
    );
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
