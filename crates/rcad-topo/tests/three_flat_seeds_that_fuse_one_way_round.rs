//! Two boxes on the same six planes, one with a corner stored a hair off where
//! its planes meet, fused both ways round.
//!
//! `docs/decisions.md` §316 and §351. Three fuzz seeds (`2e9c4138…`,
//! `65452b22…`, `1422656e…`) each hold two cubes on the same six planes, the
//! second with one stored corner moved by an amount no plane can see. Each fused
//! to one cube one way round and refused `ResultIsNotManifold` the other. The
//! planes decide where a solid is and a stored corner is only a copy (§17.1),
//! so both orders are the one cube and the refusal was wrong.
//!
//! The smallest shape that shows it is at unit size: two `[0, 2]³` boxes, the
//! second with its corner `(0, 0, 0)` stored as `(0, 1e-30, 0)`. That corner
//! starts the wire of the moved box's `y = 0` face, so both triangles fanned
//! from it tilt, and the ray called the clean box's `y = 0` piece `Outside` the
//! moved box when it lies on its face. Selection now asks whether a piece lies
//! on a face before it fires the ray.

use rcad_geom::{Deflection, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, Rule, SolidBuilder, SolidKey};

/// The seeds' own size, `s` in §316.1.
const THE_SEEDS_SIDE: f64 = 2.361_183_241_434_822_6e21;

#[test]
fn a_box_with_its_origin_stored_1e_30_off_and_a_clean_box_are_one_cube_whichever_comes_first() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for rule in [Rule::Union, Rule::Intersection] {
            for moved_first in [false, true] {
                let what = format!("{policy:?} {rule:?}, moved box first {moved_first}");
                let (mut model, clean, moved) = two_boxes(2.0, 0, Point::new(0.0, 1e-30, 0.0));

                let answer = fused(&mut model, clean, moved, moved_first, rule, &policy, &what);

                let [cube] = answer[..] else {
                    panic!("{what}: {} solids where one cube was due", answer.len());
                };
                assert_eq!(
                    model.solid(cube).expect("an answer's key").faces().count(),
                    6,
                    "{what}"
                );
                let volume = model
                    .solid(cube)
                    .expect("an answer's key")
                    .measure(
                        Deflection::of(0.1).expect("a tenth of a millimetre is a length"),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .expect("a box measures")
                    .volume();
                assert_eq!(volume, 8.0, "{what}: a flat cube's volume is exact");
            }
        }
    }
}

/// The same two boxes, one taken out of the other: nothing is left, whichever
/// is taken out of which. The clean box less the moved one refused
/// `ResultIsNotManifold { count: 1 }` before §351.
#[test]
fn a_box_less_a_copy_with_its_origin_stored_1e_30_off_is_nothing_whichever_comes_first() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for moved_first in [false, true] {
            let what = format!("{policy:?}, moved box first {moved_first}");
            let (mut model, clean, moved) = two_boxes(2.0, 0, Point::new(0.0, 1e-30, 0.0));

            let answer = fused(
                &mut model,
                clean,
                moved,
                moved_first,
                Rule::Difference,
                &policy,
                &what,
            );

            assert!(answer.is_empty(), "{what}: {answer:?}");
        }
    }
}

/// Every other move measured to refuse one way round before §351: §316.3's
/// rows at unit size and at the seeds' own size, the seeds' own moves, and two
/// that §316.3 did not have.
///
/// - At unit size a move of `1e-300` answered both ways when §316.3 measured
///   it, because the ray's `orient3d` could not decide a sign that small. Since
///   §324 it can, and it refused one way round like the rest.
/// - A move survives on a coordinate that is not zero when it is a few ulps
///   rather than under half of one. The far corner `(2, 2, 2)` stored four
///   ulps of `2` up is the far end of the diagonal each of its three faces is
///   fanned along, so both triangles of each tilt, and it refused too.
#[test]
fn every_move_of_a_stored_corner_that_refused_one_way_round_is_one_cube_both_ways() {
    let moves = [
        (2.0, 0, Point::new(0.0, 0.0, 1e-17)),
        (2.0, 0, Point::new(0.0, -1e-17, 0.0)),
        (2.0, 0, Point::new(0.0, 1e-300, 0.0)),
        (2.0, 7, Point::new(2.0, 2.0 + 4.0 * f64::EPSILON, 2.0)),
        (THE_SEEDS_SIDE, 0, Point::new(0.0, 1e-30, 0.0)),
        (THE_SEEDS_SIDE, 0, Point::new(0.0, 0.0, 1e-17)),
        (THE_SEEDS_SIDE, 0, Point::new(0.0, -1e-17, 0.0)),
        (THE_SEEDS_SIDE, 0, Point::new(0.0, 1e-300, 0.0)),
        (THE_SEEDS_SIDE, 0, Point::new(0.0, 1.358e-312, 0.0)),
        (THE_SEEDS_SIDE, 0, Point::new(0.0, 0.0, 3.2379e-319)),
    ];
    for (side, corner, stored_at) in moves {
        for rule in [Rule::Union, Rule::Intersection] {
            for moved_first in [false, true] {
                let what = format!(
                    "side {side}, corner {corner} stored at {stored_at:?}, {rule:?}, \
                     moved box first {moved_first}"
                );
                let (mut model, clean, moved) = two_boxes(side, corner, stored_at);
                let answer = fused(
                    &mut model,
                    clean,
                    moved,
                    moved_first,
                    rule,
                    &Policy::certified(),
                    &what,
                );
                let faces: Vec<usize> = answer
                    .iter()
                    .map(|&solid| model.solid(solid).expect("an answer's key").faces().count())
                    .collect();
                assert_eq!(faces, [6], "{what}");
            }
        }
    }
}

/// A model `100 · side` millimetres across holding the box `[0, side]³` twice,
/// the second with corner number `corner` stored at `stored_at`.
fn two_boxes(side: f64, corner: usize, stored_at: Point) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0 * side).expect("a working size"));
    let clean = a_box(&mut model, side, corner, None);
    let moved = a_box(&mut model, side, corner, Some(stored_at));
    (model, clean, moved)
}

/// The answer of the fuse of the two boxes, once the model has been checked to
/// validate. A certified answer records nothing (§5), so its log has to come
/// back empty too.
fn fused(
    model: &mut Model,
    clean: SolidKey,
    moved: SolidKey,
    moved_first: bool,
    rule: Rule,
    policy: &Policy,
    what: &str,
) -> Vec<SolidKey> {
    let (first, second) = if moved_first {
        (moved, clean)
    } else {
        (clean, moved)
    };
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&[first], &[second], rule, policy, &mut log)
        .unwrap_or_else(|refused| panic!("{what}: {refused:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|invalid| panic!("{what}: {invalid:?}"));
    if policy == &Policy::certified() {
        assert!(log.is_empty(), "{what}: {log:?}");
    }
    answer
}

/// The box `[0, side]³`, face by face. Corner number `n` is at `x = side` when
/// bit 1 of `n` is set, `y = side` for bit 2 and `z = side` for bit 4, and
/// corner number `moved` is stored at `stored_at` instead — which the six
/// planes do not see.
///
/// Each face's wire runs anticlockwise seen from outside, and the wires of the
/// three faces through the origin all start there, so every triangle fanned
/// from their first corner holds corner `0`.
fn a_box(model: &mut Model, side: f64, moved: usize, stored_at: Option<Point>) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let corners: Vec<_> = (0..8)
        .map(|index| {
            let at = |bit: usize| if index & bit == 0 { 0.0 } else { side };
            let point = match stored_at {
                Some(point) if index == moved => point,
                _ => Point::new(at(1), at(2), at(4)),
            };
            builder.add_vertex(point).expect("a finite point")
        })
        .collect();
    let low = Point::new(0.0, 0.0, 0.0);
    let high = Point::new(side, side, side);
    let faces: [([usize; 4], Point, Vector); 6] = [
        ([0, 4, 6, 2], low, Vector::new(-1.0, 0.0, 0.0)),
        ([1, 3, 7, 5], high, Vector::new(1.0, 0.0, 0.0)),
        ([0, 1, 5, 4], low, Vector::new(0.0, -1.0, 0.0)),
        ([2, 6, 7, 3], high, Vector::new(0.0, 1.0, 0.0)),
        ([0, 2, 3, 1], low, Vector::new(0.0, 0.0, -1.0)),
        ([4, 5, 7, 6], high, Vector::new(0.0, 0.0, 1.0)),
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
                DraftStep::Along(
                    builder
                        .add_coedge(edge, way)
                        .expect("an edge of this build"),
                )
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
