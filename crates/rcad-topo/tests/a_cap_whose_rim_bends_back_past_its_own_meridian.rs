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
//! Turned, the cube can leave both of the ball's poles outside it, and the
//! ball's face is then no cap but a piece holding neither pole, cut across the
//! middle of how far round it goes instead. Where its rim bends back past that
//! middle, the cut has to go round the bend (`docs/queue.md` Q847; `mesh.rs`,
//! `cut_in_two`).
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
    let upright = intersected_and_measured(None, spot, CUBE_FIRST, 0.1);
    assert_eq!(
        upright.0, 5,
        "three flat faces, the ball's cap and the small piece of it at the corner"
    );
    for turn in [([2.0, -1.0, 1.0], 1.0), ([1.0, 2.0, 3.0], 1.0)] {
        holds_the_same(
            upright,
            intersected_and_measured(Some(turn), spot, CUBE_FIRST, 0.1),
            turn,
        );
    }
}

/// Both points where the cube's `z` edge pierces the ball lie on the rim, at
/// one longitude, since that edge runs along the ball's polar axis. The cut up
/// from the lower ran through the upper, and the mesh was refused
/// `TwoPatchesOfTheMeshDrewTheSameSide`.
#[test]
fn a_ball_whose_rim_meets_one_meridian_at_two_corners_meshes_as_it_does_turned() {
    let spot = [0.75, 0.6, 0.7];
    let upright = intersected_and_measured(None, spot, CUBE_FIRST, 0.1);
    assert_eq!(upright.0, 4, "three flat faces and the ball's cap");
    for turn in [([1.0, 1.0, 1.0], 1.0), ([1.0, 2.0, 3.0], 1.0)] {
        holds_the_same(
            upright,
            intersected_and_measured(Some(turn), spot, CUBE_FIRST, 0.1),
            turn,
        );
    }
}

/// Turned 1 rad about `(−1, 3, 2)`, the ball's large face is shaped like a
/// `C` in its own chart: both poles are outside the cube, the rim runs within
/// six degrees of one and eleven of the other, and the `y = 0` circle reaches
/// in between the two horns to longitude `−0.13`. The face is cut across the
/// middle of how far round it goes, and the second cut, across `0.176`, was
/// pulled back to the horns' tips at `−1.18` and `1.61`, ran across that
/// circle and was refused `BoundaryCrossesItself`, at every deflection here
/// and fused either way round (`docs/queue.md` Q847). A cut that leaves the
/// piece is now replaced by one that stays in it (`mesh.rs`, `cut_in_two`).
#[test]
fn a_ball_face_shaped_like_a_c_in_its_chart_meshes_as_it_does_turned() {
    let spot = [0.7, 0.65, 0.6];
    let turn = ([-1.0, 3.0, 2.0], 1.0);
    for deflection in [0.3, 0.1, 0.03] {
        let upright = intersected_and_measured(None, spot, CUBE_FIRST, deflection);
        for cube_first in [CUBE_FIRST, !CUBE_FIRST] {
            let turned = intersected_and_measured(Some(turn), spot, cube_first, deflection);
            holds_the_same(upright, turned, turn);
        }
    }
}

/// The other turns `docs/queue.md` Q847 found refused after the cut up to a
/// cap's pole first left only from a corner clear of the rim: three with the
/// ball at `(0.75, 0.6, 0.7)` refused `TwoPatchesOfTheMeshDrewTheSameSide`
/// after that cut, three more without reaching it, and three with the ball at
/// `(0.7, 0.65, 0.6)` were refused `Invalid(FacesDoNotAgreeWhichWayTheyFace)`.
/// Every one of them meshes now, fused either way round, at every deflection
/// here, and holds what the upright pair holds.
#[test]
fn the_turns_that_folded_after_the_cut_from_a_clear_corner_mesh_as_they_do_upright() {
    let turns_both_spots_folded_at = [
        ([-1.0, 3.0, 2.0], 2.0),
        ([3.0, 1.0, -2.0], 2.0),
        ([3.0, 1.0, -2.0], 3.0),
    ];
    let cases = [
        ([0.75, 0.6, 0.7], &turns_both_spots_folded_at[..]),
        (
            [0.75, 0.6, 0.7],
            &[
                ([2.0, -1.0, 1.0], 3.0),
                ([-1.0, 3.0, 2.0], 1.0),
                ([3.0, 1.0, -2.0], 1.0),
            ][..],
        ),
        ([0.7, 0.65, 0.6], &turns_both_spots_folded_at[..]),
    ];
    for (spot, turns) in cases {
        for deflection in [0.3, 0.1, 0.03] {
            let upright = intersected_and_measured(None, spot, CUBE_FIRST, deflection);
            for &turn in turns {
                for cube_first in [CUBE_FIRST, !CUBE_FIRST] {
                    let turned = intersected_and_measured(Some(turn), spot, cube_first, deflection);
                    holds_the_same(upright, turned, turn);
                }
            }
        }
    }
}

/// The cube is the first argument of the fuse and the ball the second.
const CUBE_FIRST: bool = true;

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
/// in the cube's own frame, intersected with the cube first or the ball
/// first, validated, and measured at `deflection`: how many faces, and the
/// measurement.
fn intersected_and_measured(
    turn: Option<([f64; 3], f64)>,
    spot: [f64; 3],
    cube_first: bool,
    deflection: f64,
) -> (usize, Measurement) {
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
    let (first, second) = if cube_first {
        (block, ball)
    } else {
        (ball, block)
    };
    let fused = model
        .fuse(
            &[first],
            &[second],
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
            Deflection::of(deflection).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| {
            panic!(
                "the intersection with the ball at {spot:?}, turned by {turn:?}, meshes at \
                 {deflection}: {why}"
            )
        });
    (view.faces().count(), measured)
}
