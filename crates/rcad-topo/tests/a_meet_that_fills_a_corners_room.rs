//! A thin wedge turned about an awkward axis, whose sharp corners are meets
//! wider than the room writing a corner down cost, and which the fuse still
//! folds into their corners (`docs/decisions.md` §301).
//!
//! `place_the_meets` folds a meet into a corner of the face when every point of
//! the meet's box lies inside the corner's box grown by that room, `16 × 4 × ε`
//! times the corner's size (§253). Where two walls of a wedge meet at a slope
//! of `1/256`, three turned planes cross at a shallow angle and the solve
//! comes back as a box over two rooms wide. **It is still folded**, because
//! what the fold measures is how far the box sticks out past the *corner's*
//! box, and `bake` solved the corner from the same three planes. The two boxes
//! hold one point and differ only by the order the solve took its rows in.
//! Across these wedges that is at most a fiftieth of the room.
//!
//! **So this file does not pin the room at half or at a tenth.** Shrunk to a
//! hundredth, some of these unions refuse. Those two are pinned in
//! `src/fuse/arrange.rs`'s `tests` module, where the places are written by
//! hand, and §301 says why no shape was found that needs them.
//!
//! **At a slope of `1/512` it does pin a room, the other one: the room `bake`
//! picks the sharp corner's place with.** Turning the planes rounds them, and
//! two planes that close carry the rounding about 512 times further, so the
//! only place the three turned planes meet is up to 1.68 rooms past where the
//! model stores the corner. A picker that granted the rounding alone refused
//! the union at 'where three surfaces meet'. It grants the room
//! `Model::validate` grants the wedge's corners as well (§368), and the place
//! is four millionths of that (§301.9).

use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule};

/// How steeply the wedge's long wall climbs from its sharp corner, from a
/// meet box a third of the room wide to one over two rooms wide, and then to
/// the slope where `bake` finds the corner's place past its rounding.
const SLOPES: [f64; 5] = [
    1.0 / 16.0,
    1.0 / 64.0,
    1.0 / 128.0,
    1.0 / 256.0,
    1.0 / 512.0,
];

const TURNS: [f64; 6] = [0.1, 0.3, 0.7, 1.1, 1.9, 2.5];

const AXES: [[f64; 3]; 3] = [[1.0, 2.0, 3.0], [3.0, -1.0, 2.0], [0.3, 1.0, -0.7]];

/// Every certified union of the wedge with a cube well clear of it answers
/// with the two solids, so every one of the wedge's corners was folded into
/// its own meet without asking the policy anything. A meet left standing
/// beside its corner overlaps it, and a certified policy refuses to say
/// whether two overlapping points are one.
#[test]
fn a_thin_turned_wedge_whose_corners_are_meets_wider_than_the_room_is_still_united() {
    for slope in SLOPES {
        for turn in TURNS {
            for axis in AXES {
                let (mut model, wedge, cube) = a_wedge_and_a_cube_clear_of_it(slope, turn, axis);
                let mut log = ProvenanceLog::new();
                let fused = model
                    .fuse(
                        &[wedge],
                        &[cube],
                        Rule::Union,
                        &Policy::certified(),
                        &mut log,
                    )
                    .unwrap_or_else(|why| {
                        panic!(
                            "a wedge climbing {slope} turned {turn} about {axis:?} was refused: \
                             {why}"
                        )
                    });
                assert_eq!(
                    fused.solids().len(),
                    2,
                    "two solids that do not touch unite as the two of them"
                );
                assert!(log.is_empty(), "a certified union recorded {log:?}");
            }
        }
    }
}

fn a_wedge_and_a_cube_clear_of_it(
    slope: f64,
    turn: f64,
    [x, y, z]: [f64; 3],
) -> (Model, rcad_topo::SolidKey, rcad_topo::SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [8.0, 0.0], [8.0, 8.0 * slope]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a triangle with three distinct corners");
    let turned = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(x, y, z)).expect("a direction with length"),
        turn,
    )
    .expect("a finite turn");
    let wedge = model
        .add_extrusion(&triangle, 1.0, &turned)
        .expect("a positive height");
    let cube = model
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a unit cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(100.0, 100.0, 100.0)).expect("a finite shift"),
        )
        .expect("this model made the key");
    (model, wedge, cube)
}
