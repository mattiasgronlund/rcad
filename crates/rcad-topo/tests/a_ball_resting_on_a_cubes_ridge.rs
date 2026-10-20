//! A ball resting exactly on the ridge of a turned cube.
//!
//! The cube is the 2 by 2 square extruded 2, turned `3π/4` about `y` through
//! the origin, so `(1, 0, 1)` goes below `z = 0` and its edge along `y` at the
//! origin is a ridge between two faces leaning 45 degrees either way. A ball of
//! radius 1 centred at `(0, 1, 1)` touches that ridge at `(0, 1, 0)`, which is
//! the ball's south pole, and shares nothing else with the cube: each face's
//! plane cuts the ball in a circle through the pole, and both circles lie
//! outside the faces. The union is the two of them, `8 + 4π/3` between them.
//!
//! The fuse cuts the ball's skin along both whole circles, into two caps and
//! the face between them, whose one wire is a figure eight through the pole.
//! Meshing that face to check which way it faces, its patches take in the
//! cap side of the first circle as well as its own, and draw the side from the
//! pole to the next point walked along that circle as the cap does: refused
//! `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 2, used: 4 }`. The solid has
//! one corner, so the ladder that would mesh it finer stops at once; forced on
//! in a probe, every rung from 1 down to `1.2e-4` folds on the same side.

use rcad_geom::{Axis, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule};

/// Pinned as it answers: the union is refused for a fold, at either working
/// size and either way round. The answer exists, so this is a gap.
#[test]
fn uniting_them_is_refused_for_a_fold_in_the_mesh_of_the_balls_middle_face() {
    for size in [100.0, 1.0] {
        for ball_first in [true, false] {
            let fused = unite(size, ball_first);
            assert!(
                matches!(
                    fused,
                    Err(Error::TwoPatchesOfTheMeshDrewTheSameSide { drawn: 2, used: 4 })
                ),
                "size {size}, ball first {ball_first}: {fused:?}"
            );
        }
    }
}

fn unite(size: f64, ball_first: bool) -> rcad_topo::Result<Fused> {
    let mut model = Model::new(Scale::millimetres(size).expect("a working size"));
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let turn =
        Isometry::rotation(&Axis::y(), 3.0 * core::f64::consts::FRAC_PI_4).expect("a finite turn");
    let cube = model
        .add_extrusion(&square, 2.0, &turn)
        .expect("a positive height");
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(Vector::new(0.0, 1.0, 1.0)).expect("a finite shift"),
        )
        .expect("a positive radius");
    let (one, other) = if ball_first {
        (ball, cube)
    } else {
        (cube, ball)
    };
    let policy = Policy::tolerant(model.scale().nominal_size());
    model.fuse(
        &[one],
        &[other],
        Rule::Union,
        &policy,
        &mut ProvenanceLog::new(),
    )
}
