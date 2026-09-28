//! Two `[0, 2]³` boxes built face by face, the second with its corner
//! `(2, 0, 0)` stored as `(2, 1e-300, 0)`, fused both ways round by union and
//! by intersection.
//!
//! `docs/decisions.md` §324, from §316.6. Both boxes sit on the same six
//! planes, so each fuse is one box of six faces. Until §324 every certified
//! fuse here refused `Undecidable`, on one `orient3d` call: the moved box's
//! `y = 0` triangle `(0,0,0)`, `(2, 1e-300, 0)`, `(2, 0, 2)` against the point
//! `(0.5, -0, 0.5)`, which is exactly on its plane. Exact arithmetic could
//! have said so and declined to run, because products near `1e-300` sit below
//! its floor; the predicate's rescale now lifts them above it.
//!
//! The log has to come back empty as well: a certified answer records nothing
//! (§5), and before §324 the tolerant fuse only answered by writing four
//! approximations into it.

use rcad_geom::{EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, Rule, SolidBuilder, SolidKey};

#[test]
fn each_fuse_is_the_one_box_whichever_comes_first() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for rule in [Rule::Union, Rule::Intersection] {
            for moved_first in [false, true] {
                let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                let clean = a_box(&mut model, Point::new(2.0, 0.0, 0.0));
                let moved = a_box(&mut model, Point::new(2.0, 1e-300, 0.0));
                let (first, second) = if moved_first {
                    (moved, clean)
                } else {
                    (clean, moved)
                };

                let mut log = ProvenanceLog::new();
                let fused = model
                    .fuse(&[first], &[second], rule, &policy, &mut log)
                    .unwrap_or_else(|refused| {
                        panic!("{policy:?} {rule:?}, moved box first {moved_first}: {refused:?}")
                    });

                let faces: Vec<usize> = fused
                    .solids()
                    .iter()
                    .map(|&solid| model.solid(solid).expect("an answer's key").faces().count())
                    .collect();
                assert_eq!(
                    faces,
                    [6],
                    "{policy:?} {rule:?}, moved box first {moved_first}"
                );
                assert!(
                    log.is_empty(),
                    "{policy:?} {rule:?}, moved box first {moved_first}: {log:?}"
                );
            }
        }
    }
}

/// The box `[0, 2]³`, face by face, with its corner `(2, 0, 0)` stored at
/// `corner` — which the six planes do not see.
///
/// Each face's wire runs anticlockwise seen from outside, and the `y = 0`
/// face's starts at the origin, so its first triangle is the one the refusal
/// named.
fn a_box(model: &mut Model, corner: Point) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let corners: Vec<_> = (0..8)
        .map(|index| {
            let at = |bit: usize| if index & bit == 0 { 0.0 } else { 2.0 };
            let point = if index == 1 {
                corner
            } else {
                Point::new(at(1), at(2), at(4))
            };
            builder.add_vertex(point).expect("a finite point")
        })
        .collect();
    let origin = Point::new(0.0, 0.0, 0.0);
    let far = Point::new(2.0, 2.0, 2.0);
    let faces: [([usize; 4], Point, Vector); 6] = [
        ([0, 4, 6, 2], origin, Vector::new(-1.0, 0.0, 0.0)),
        ([1, 3, 7, 5], far, Vector::new(1.0, 0.0, 0.0)),
        ([0, 1, 5, 4], origin, Vector::new(0.0, -1.0, 0.0)),
        ([2, 6, 7, 3], far, Vector::new(0.0, 1.0, 0.0)),
        ([0, 2, 3, 1], origin, Vector::new(0.0, 0.0, -1.0)),
        ([4, 5, 7, 6], far, Vector::new(0.0, 0.0, 1.0)),
    ];

    let mut edges = std::collections::BTreeMap::new();
    let mut shell = Vec::new();
    for (ring, on, outwards) in faces {
        let steps: Vec<_> = (0..4)
            .map(|turn| {
                let (from, to) = (ring[turn], ring[(turn + 1) % 4]);
                let ends = (from.min(to), from.max(to));
                let edge = *edges.entry(ends).or_insert_with(|| {
                    builder
                        .add_edge(corners[ends.0], corners[ends.1], EdgeRun::Straight)
                        .expect("two corners apart")
                });
                let way = if from < to {
                    Orientation::Forward
                } else {
                    Orientation::Reversed
                };
                DraftStep::Along(builder.add_coedge(edge, way).expect("an edge of this build"))
            })
            .collect();
        let wire = builder.add_wire(&steps).expect("four steps round a square");
        let plane = Plane::from_point_and_normal(on, outwards).expect("a unit normal");
        let face = builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a square on its plane");
        shell.push((face, Orientation::Forward));
    }
    builder.add_shell(&shell).expect("six faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a box described this way is a solid")
}
