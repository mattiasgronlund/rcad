//! A cube of side 2 at the origin and a unit ball near one of its corners,
//! intersected. The ball's face of the answer holds its north pole, and its rim
//! runs round the three circles the cube's faces cut. Where it goes round the
//! side of one circle it bends back past a meridian it has already crossed.
//!
//! The mesher cuts such a cap into four by meridians up to the pole, and it
//! used to cut from the rim corner nearest each quarter turn. A meridian the rim
//! crosses twice took the cut out of the face and back in, and both pairs here
//! were refused at every deflection (`docs/queue.md` Q441). The cut now leaves
//! from a corner whose meridian meets the rim nowhere else
//! (`mesh.rs`, `corners_the_quarter_cuts_leave_from`).
//!
//! **The answer is checked against the same pair turned.** A rigid movement
//! changes neither the volume nor the faces, so the upright pair has to hold
//! what a turned one holds, within the two measurements' own bounds added.

#![forbid(unsafe_code)]

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Measurement, Model, Rule};

/// The cut up longitude `2.636` from where the cube's `y` edge leaves the
/// ball crossed the `x = 0` circle at `x = -0.175`, outside the cube, and the
/// triangulator refused the patch `BoundaryCrossesItself`.
#[test]
fn a_ball_whose_rim_goes_round_the_side_of_a_circle_meshes_as_it_does_turned() {
    let spot = [0.7, 0.65, 0.6];
    let upright = intersected_and_measured(None, spot);
    assert_eq!(
        upright.0, 5,
        "three flat faces, the ball's cap and the small piece of it at the corner"
    );
    for turn in [([2.0, -1.0, 1.0], 1.0), ([1.0, 2.0, 3.0], 1.0)] {
        holds_the_same(upright, intersected_and_measured(Some(turn), spot), turn);
    }
}

/// Both points where the cube's `z` edge pierces the ball lie on the rim, at
/// one longitude, since that edge runs along the ball's polar axis. The cut up
/// from the lower ran through the upper, and the mesh was refused
/// `TwoPatchesOfTheMeshDrewTheSameSide`.
#[test]
fn a_ball_whose_rim_meets_one_meridian_at_two_corners_meshes_as_it_does_turned() {
    let spot = [0.75, 0.6, 0.7];
    let upright = intersected_and_measured(None, spot);
    assert_eq!(upright.0, 4, "three flat faces and the ball's cap");
    for turn in [([1.0, 1.0, 1.0], 1.0), ([1.0, 2.0, 3.0], 1.0)] {
        holds_the_same(upright, intersected_and_measured(Some(turn), spot), turn);
    }
}

fn holds_the_same(
    (faces, upright): (usize, Measurement),
    (turned_faces, turned): (usize, Measurement),
    turn: ([f64; 3], f64),
) {
    assert_eq!(faces, turned_faces, "the same faces turned by {turn:?}");
    let apart = (upright.volume() - turned.volume()).abs();
    let bound = upright.volume_error() + turned.volume_error();
    assert!(
        apart <= bound,
        "upright {} and turned by {turn:?} {} are {apart} apart, over the {bound} the two \
         measurements declare between them",
        upright.volume(),
        turned.volume()
    );
}

/// The cube turned by `turn` about the origin, if any, and the ball at `spot`
/// in the cube's own frame, intersected, validated, and measured at a
/// deflection of `0.1`: how many faces, and the measurement.
fn intersected_and_measured(turn: Option<([f64; 3], f64)>, spot: [f64; 3]) -> (usize, Measurement) {
    let turn = turn.map_or_else(Isometry::identity, |([x, y, z], angle)| {
        Isometry::rotation(
            &Axis::new(Point::origin(), Vector::new(x, y, z)).expect("an axis"),
            angle,
        )
        .expect("a finite turn")
    });
    let mut model = Model::new(Scale::millimetres(1.0).expect("1 mm is a working size"));
    let block = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(block, &turn)
        .expect("this model made the key");
    let centre = turn.apply_to_point(Point::new(spot[0], spot[1], spot[2]));
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(centre - Point::origin()).expect("a finite placement"),
        )
        .expect("a ball");
    let fused = model
        .fuse(
            &[block],
            &[ball],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the cube and the ball at {spot:?} intersect: {why}"));
    let [solid] = fused.solids()[..] else {
        panic!("one piece of the ball, not {:?}", fused.solids());
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the intersection is a sound model");
    let view = model.solid(solid).expect("the fuse's own key");
    let measured = view
        .measure(
            Deflection::of(0.1).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the intersection with the ball at {spot:?} meshes: {why}"));
    (view.faces().count(), measured)
}
