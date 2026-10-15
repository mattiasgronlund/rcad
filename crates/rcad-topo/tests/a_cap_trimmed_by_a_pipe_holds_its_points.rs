//! Flat faces cut by a bar's pipe, and a bar's wall cut by flat faces: which
//! of them say what points they hold, and which are refused by name.
//!
//! A cap keeping the **inside** of the pipe is answered: the inside of a pipe is
//! every point within its radius of the axis, which is convex, and a convex
//! piece is what `holds.rs`'s `is_a_family_this_follows` needs. A wall cut by
//! two planes along the axis is answered when the two meet inside the pipe — a
//! quarter of a bar — and when they are parallel and keep the strip between
//! them, a bar with two flats, whose two walls are told apart by the half of
//! the pipe each is in. A flat face keeping the **outside** of the pipe is
//! answered by its cuts when the pipe's circle crosses the edge of the face's
//! square only at the two ends of its one arc — a block with a bar bored
//! through the middle of one edge. Otherwise the part of its plane in front of
//! every cut can be two pieces, and the face winds its own sides instead, each
//! arc read as its chord with the piece between the two added or taken away
//! (row Q1002): the two halves of a block's top a bar bored into it splits, the
//! floor and top of a block bored through one corner, and the caps of a slab
//! with a round scoop out of its top. Where two pieces are in front of every
//! cut, the test shows a point of each on the same side of every cut and the
//! face holding only its own.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, FaceKey, Model, OnAFace, ProfileBuilder, Rule, SolidKey};

/// A bar of radius 5 from `z = 0` to `z = 4`, less the part outside `x ≥ 0` and
/// `y ≥ 0`: its wall is cut by two planes along the axis that meet on the
/// axis, and is the quarter of the pipe between them and the caps.
#[test]
fn a_quarter_of_a_bar_holds_the_points_of_its_wall() {
    for bar_first in [true, false] {
        let (model, solid) = a_quarter_of_a_bar(bar_first);
        let wall = the_only_face(&model, solid, |surface| {
            matches!(surface, Surface::Cylinder(_))
        });
        let mut held = 0;
        let mut not_held = 0;
        for degrees in [-15.0, 15.0, 45.0, 75.0, 105.0, 180.0, 255.0] {
            for z in [-0.5, 0.5, 2.0, 3.5, 4.5] {
                let on_the_wall = 0.0 < degrees && degrees < 90.0 && 0.0 < z && z < 4.0;
                let expected = if on_the_wall {
                    held += 1;
                    OnAFace::Held
                } else {
                    not_held += 1;
                    OnAFace::NotHeld
                };
                assert_eq!(
                    holds(&model, solid, wall, on_the_pipe(5.0, degrees, z)),
                    Ok(expected),
                    "the quarter's wall, bar first {bar_first}, at {degrees}° and z = {z}"
                );
            }
        }
        assert_eq!((held, not_held), (9, 26));
    }
}

/// The whole quarter, asked point by point: the ray caster crosses the wall,
/// so a wall that refuses makes the solid refuse.
#[test]
fn a_quarter_of_a_bar_says_which_points_are_inside_it() {
    for bar_first in [true, false] {
        let (model, solid) = a_quarter_of_a_bar(bar_first);
        let view = model.solid(solid).expect("this model made the key");
        for (point, expected) in [
            ([1.0, 1.0, 2.0], Containment::Inside),
            ([3.0, 3.0, 1.0], Containment::Inside),
            ([4.5, 1.0, 3.0], Containment::Inside),
            ([-1.0, 1.0, 2.0], Containment::Outside),
            ([1.0, -1.0, 2.0], Containment::Outside),
            ([4.0, 4.0, 2.0], Containment::Outside),
            ([1.0, 1.0, 4.5], Containment::Outside),
        ] {
            let mut log = ProvenanceLog::new();
            let [x, y, z] = point;
            assert_eq!(
                view.classify_point(Point::new(x, y, z), &Policy::certified(), &mut log),
                expected,
                "the quarter, bar first {bar_first}, at {point:?}"
            );
            assert!(log.is_empty(), "a certified answer logs nothing: {log:?}");
        }
    }
}

/// The quarter's roof is a flat face cut by the pipe, keeping its inside, and
/// by the two planes: a quarter of a disc.
#[test]
fn a_cap_trimmed_by_a_pipe_holds_its_points() {
    for bar_first in [true, false] {
        let (model, solid) = a_quarter_of_a_bar(bar_first);
        let roof = the_only_face(&model, solid, |surface| {
            surface
                .as_plane()
                .is_some_and(|flat| flat.normal().components() == [0.0, 0.0, 1.0])
        });
        for (x, y) in [(1.0, 1.0), (3.0, 1.0), (1.0, 3.0), (2.5, 2.5), (4.5, 0.5)] {
            assert_eq!(
                holds(&model, solid, roof, [x, y, 4.0]),
                Ok(OnAFace::Held),
                "the roof, bar first {bar_first}, at ({x}, {y})"
            );
        }
        for (x, y) in [(-1.0, 1.0), (1.0, -1.0), (4.0, 4.0), (-2.0, -2.0)] {
            assert_eq!(
                holds(&model, solid, roof, [x, y, 4.0]),
                Ok(OnAFace::NotHeld),
                "the roof, bar first {bar_first}, at ({x}, {y})"
            );
        }
    }
}

/// A bar of radius 5 with two flats, `|x| ≤ 3`. Each wall is one run of the
/// pipe's rulings, round `+y` or round `−y`, cut by both flats and both caps —
/// and the other wall is in front of all four cuts too, so "the part of the
/// pipe in front of every cut" is both walls. Each wall holds its own points
/// and none of the other's: those on the pipe between the flats and the caps,
/// on its own side of `y = 0`. Most points asked are on the pipe exactly, at
/// the corners of 3-4-5 triangles, so the rulings the flats cut at `(±3, ±4)`
/// are asked too; the rest are on it to within rounding and well clear of every
/// edge.
#[test]
fn a_bar_with_two_flats_holds_the_points_of_each_wall() {
    let (model, solid) = a_bar_with_two_flats();
    let walls = the_faces(&model, solid, |surface| {
        matches!(surface, Surface::Cylinder(_))
    });
    assert_eq!(walls.len(), 2, "one wall each side of the flats");
    let on_the_pipe_exactly = [
        [5.0, 0.0],
        [4.0, 3.0],
        [3.0, 4.0],
        [0.0, 5.0],
        [-3.0, 4.0],
        [-4.0, 3.0],
        [-5.0, 0.0],
        [-4.0, -3.0],
        [-3.0, -4.0],
        [0.0, -5.0],
        [3.0, -4.0],
        [4.0, -3.0],
    ];
    let on_the_pipe_rounded = [75.0, 100.0, 260.0, 285.0].map(|degrees| {
        let [x, y, _] = on_the_pipe(5.0, degrees, 0.0);
        [x, y]
    });
    let mut counted = [0; 3];
    for wall in walls {
        assert_eq!(
            sides_of_every_neighbour(&model, solid, wall, [0.0, 5.0, 2.0]),
            sides_of_every_neighbour(&model, solid, wall, [0.0, -5.0, 2.0]),
            "the two walls are on the same side of every cut of one of them"
        );
        let round = which_way_round(&model, solid, wall);
        for [x, y] in on_the_pipe_exactly.into_iter().chain(on_the_pipe_rounded) {
            for z in [-0.5, 0.0, 0.5, 2.0, 3.5, 4.0, 4.5] {
                let expected = on_a_wall_of_the_bar_with_two_flats(round, [x, y, z]);
                counted[match expected {
                    OnAFace::Held => 0,
                    OnAFace::OnItsBoundary => 1,
                    OnAFace::NotHeld => 2,
                    other => unreachable!("the test's own answer is one of three: {other:?}"),
                }] += 1;
                assert_eq!(
                    holds(&model, solid, wall, [x, y, z]),
                    Ok(expected),
                    "the wall round {round} times y, at ({x}, {y}, {z})"
                );
            }
        }
    }
    assert_eq!(counted, [18, 32, 174]);
}

/// The whole bar with two flats, asked point by point: the ray caster crosses
/// the walls, so a wall that refuses makes the solid refuse.
#[test]
fn a_bar_with_two_flats_says_which_points_are_inside_it() {
    let (model, solid) = a_bar_with_two_flats();
    let view = model.solid(solid).expect("this model made the key");
    for (point, expected) in [
        ([0.0, 0.0, 2.0], Containment::Inside),
        ([2.0, 4.0, 1.0], Containment::Inside),
        ([0.0, 4.5, 3.0], Containment::Inside),
        ([-1.0, -4.5, 1.0], Containment::Inside),
        ([3.5, 0.0, 2.0], Containment::Outside),
        ([0.0, 5.5, 2.0], Containment::Outside),
        ([2.0, -4.8, 2.0], Containment::Outside),
        ([0.0, 0.0, 4.5], Containment::Outside),
        ([0.0, -4.5, -0.5], Containment::Outside),
    ] {
        let mut log = ProvenanceLog::new();
        let [x, y, z] = point;
        assert_eq!(
            view.classify_point(Point::new(x, y, z), &Policy::certified(), &mut log),
            expected,
            "the bar with two flats, at {point:?}"
        );
        assert!(log.is_empty(), "a certified answer logs nothing: {log:?}");
    }
}

/// A bar of radius 5 from `z = 0` to `z = 4`, less the part outside `x ≤ 4` and
/// `y ≤ 4`. The two flats are not parallel, and meet where `x = y = 4`,
/// `√32 ≈ 5.66` from the axis — outside the pipe. `is_a_wedge_of_a_pipe` in
/// `holds.rs` refuses this (the two meet outside, not inside), and so does
/// `one_of_two_runs_between_two_flats` (the two are not parallel), but
/// `one_of_two_runs_between_two_cuts_meeting_off_the_pipe` answers it: each
/// flat crosses the circle twice, the point they meet at is strictly outside
/// it, and each flat's own chord keeps the other flat's side, so the wall is
/// two runs told apart by where the tangents from `(4, 4)` touch the circle —
/// a short run between the flats' own crossings, `(4, 3)` to `(3, 4)`, and the
/// rest of the bar. Neither wall is refused by name.
#[test]
fn a_bar_with_two_flats_meeting_outside_the_pipe_holds_the_points_of_each_wall() {
    let (model, solid) = a_bar_with_two_flats_meeting_outside_the_pipe();
    let walls = the_faces(&model, solid, |surface| {
        matches!(surface, Surface::Cylinder(_))
    });
    assert_eq!(walls.len(), 2, "one wall for each run the flats leave");
    let on_the_pipe_exactly = [
        [5.0, 0.0],
        [4.0, 3.0],
        [3.0, 4.0],
        [0.0, 5.0],
        [-3.0, 4.0],
        [-4.0, 3.0],
        [-5.0, 0.0],
        [-4.0, -3.0],
        [-3.0, -4.0],
        [0.0, -5.0],
        [3.0, -4.0],
        [4.0, -3.0],
    ];
    let mut counted = [0; 3];
    for wall in walls {
        assert_eq!(
            sides_of_every_neighbour(&model, solid, wall, [3.5, 3.5, 2.0]),
            sides_of_every_neighbour(&model, solid, wall, [-5.0, 0.0, 2.0]),
            "the short run and the long run are on the same side of every cut"
        );
        let near = is_the_near_run_of_two_flats_meeting_outside_the_pipe(&model, solid, wall);
        for [x, y] in on_the_pipe_exactly {
            for z in [-0.5, 0.0, 0.5, 2.0, 3.5, 4.0, 4.5] {
                let expected = on_a_wall_of_two_flats_meeting_outside_the_pipe(near, [x, y, z]);
                counted[class_of(expected)] += 1;
                assert_eq!(
                    holds(&model, solid, wall, [x, y, z]),
                    Ok(expected),
                    "the wall whose short run is {near}, at ({x}, {y}, {z})"
                );
            }
        }
    }
    assert_eq!(counted, [18, 32, 118]);
}

/// A block `2 × 12 × 4` with a bar of radius 3 bored down into its middle as
/// far as `z = 2`. The bar's circle runs into the top at one long side and out
/// at the other, so the top is two faces, each keeping the outside of the pipe
/// — and the other piece is in front of every one of this piece's cuts, which
/// is why no reading of the cuts answers it. Each half winds its own sides
/// instead, its arc read as the chord at `y = ±√8` with the piece of the bore's
/// disc past that chord taken away, and holds its own side of the bore and
/// none of the other's: every answer worked out from the coordinates
/// ([`on_one_half_of_the_narrow_top`]). Until row Q1002 both halves were
/// refused by name.
#[test]
fn a_block_with_a_bar_bored_into_it_has_each_half_of_its_top_hold_its_own_side() {
    let mut model = a_model();
    let block = a_block(&mut model, [-1.0, -6.0, 0.0], [2.0, 12.0, 4.0]);
    let bar = a_bar(&mut model, 3.0, [0.0, 0.0, 2.0]);
    let solid = the_one_solid(&mut model, block, bar, Rule::Difference);
    let tops = the_faces(&model, solid, |surface| {
        surface.as_plane().is_some_and(|flat| {
            flat.normal().components() == [0.0, 0.0, 1.0] && flat.offset() == 4.0
        })
    });
    assert_eq!(tops.len(), 2, "the top either side of the bore");
    let mut counted = [0; 3];
    for top in tops {
        assert_eq!(
            sides_of_every_neighbour(&model, solid, top, [0.0, 4.5, 4.0]),
            sides_of_every_neighbour(&model, solid, top, [0.0, -4.5, 4.0]),
            "the two pieces of the top are on the same side of every cut of one of them"
        );
        let side = which_side_of_y_is_zero(&model, solid, top);
        for x in [-1.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5] {
            for y in [
                -6.5, -6.0, -5.0, -4.5, -3.0, -2.5, 0.0, 2.5, 3.0, 4.5, 5.0, 6.0, 6.5,
            ] {
                let expected = on_one_half_of_the_narrow_top(side, [x, y]);
                counted[class_of(expected)] += 1;
                assert_eq!(
                    holds(&model, solid, top, [x, y, 4.0]),
                    Ok(expected),
                    "the half of the top on the side {side} of y = 0, at ({x}, {y})"
                );
            }
        }
    }
    assert_eq!(counted, [16, 24, 142]);
}

/// A block `10 × 10 × 4` with a bar of radius 3 bored down through the middle
/// of one edge. Its floor and its top each keep the outside of the pipe and are
/// **one piece**, the square less half a disc: the circle crosses the edge of
/// the square twice, where the block above crosses it four times. Each says
/// which points it holds, across the square and past it, every answer worked
/// out from the coordinates against the square and the disc
/// ([`on_the_square_less_half_a_disc`]).
#[test]
fn a_block_with_a_bar_bored_through_one_edge_holds_the_points_of_its_floor_and_its_top() {
    let (model, solid) = a_block_with_a_bar_bored_through_one_edge();
    let mut counted = [0; 3];
    for (z, facing) in [(0.0, -1.0), (4.0, 1.0)] {
        let face = the_only_face(&model, solid, |surface| {
            surface
                .as_plane()
                .is_some_and(|flat| flat.normal().components() == [0.0, 0.0, facing])
        });
        for x in [-1.0, 0.0, 0.5, 1.0, 2.0, 3.0, 4.0, 9.5, 10.0, 11.0] {
            for y in [-1.0, 0.0, 1.0, 2.0, 3.0, 5.0, 7.0, 8.0, 9.0, 10.0, 11.0] {
                let expected = on_the_square_less_half_a_disc([x, y]);
                counted[match expected {
                    OnAFace::Held => 0,
                    OnAFace::OnItsBoundary => 1,
                    OnAFace::NotHeld => 2,
                    other => unreachable!("the test's own answer is one of three: {other:?}"),
                }] += 1;
                assert_eq!(
                    holds(&model, solid, face, [x, y, z]),
                    Ok(expected),
                    "the face at z = {z}, at ({x}, {y})"
                );
            }
        }
    }
    assert_eq!(counted, [64, 56, 100]);
}

/// A block `10 × 10 × 4` with a bar of radius 3 bored down through one corner.
/// Its floor and its top are each **one piece**, the square less a quarter of
/// a disc. No reading of the cuts says so — each side through the bite holds
/// one end of the arc and crosses the pipe again outside the square, as the
/// narrow top's long sides do where its other piece begins — so each face winds
/// its own sides, the arc read as the chord `x + y = 3` with the piece of the
/// disc past it taken away. Every answer is worked out from the coordinates
/// against the square and the disc ([`on_the_square_less_a_quarter_disc`]), and
/// the points on the chord, inside the disc, are turned away. Until row Q1002
/// the floor was refused by name.
#[test]
fn a_block_with_a_bar_bored_through_one_corner_holds_the_points_of_its_floor_and_its_top() {
    let mut model = a_model();
    let block = a_block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let bar = a_bar(&mut model, 3.0, [0.0, 0.0, -1.0]);
    let solid = the_one_solid(&mut model, block, bar, Rule::Difference);
    let mut counted = [0; 3];
    for (z, facing) in [(0.0, -1.0), (4.0, 1.0)] {
        let face = the_only_face(&model, solid, |surface| {
            surface
                .as_plane()
                .is_some_and(|flat| flat.normal().components() == [0.0, 0.0, facing])
        });
        let along = [
            -1.0, 0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0, 9.5, 10.0, 11.0,
        ];
        for x in along {
            for y in along {
                let expected = on_the_square_less_a_quarter_disc([x, y]);
                counted[class_of(expected)] += 1;
                assert_eq!(
                    holds(&model, solid, face, [x, y, z]),
                    Ok(expected),
                    "the face at z = {z}, at ({x}, {y})"
                );
            }
        }
    }
    assert_eq!(counted, [96, 54, 138]);
}

/// The whole block with the bar bored through one edge, asked point by point:
/// the ray caster crosses the floor and the top, so a face that refuses makes
/// the solid refuse.
#[test]
fn a_block_with_a_bar_bored_through_one_edge_says_which_points_are_inside_it() {
    let (model, solid) = a_block_with_a_bar_bored_through_one_edge();
    let view = model.solid(solid).expect("this model made the key");
    for (point, expected) in [
        ([5.0, 5.0, 2.0], Containment::Inside),
        ([5.0, 1.0, 2.0], Containment::Inside),
        ([2.0, 8.5, 1.0], Containment::Inside),
        ([9.0, 9.0, 3.0], Containment::Inside),
        ([1.0, 5.0, 2.0], Containment::Outside),
        ([1.0, 2.5, 3.0], Containment::Outside),
        ([-1.0, 5.0, 2.0], Containment::Outside),
        ([5.0, 5.0, 4.5], Containment::Outside),
        ([5.0, 5.0, -0.5], Containment::Outside),
        ([11.0, 5.0, 2.0], Containment::Outside),
    ] {
        let mut log = ProvenanceLog::new();
        let [x, y, z] = point;
        assert_eq!(
            view.classify_point(Point::new(x, y, z), &Policy::certified(), &mut log),
            expected,
            "the block bored through one edge, at {point:?}"
        );
        assert!(log.is_empty(), "a certified answer logs nothing: {log:?}");
    }
}

/// The two caps of a slab with a round scoop out of its top, `|x| ≤ 4`,
/// `−6 ≤ y ≤ −3`, four tall, outside the pipe of radius 5 about `z`, built face
/// by face and fused as a block less a bar. The pipe's circle meets `x = ±4` at
/// `y = −3`, so the arc's ends are corners of the cap and the cap has no side
/// along `y = −3`.
///
/// **On each cap the part in front of every cut is two pieces**: the cap, and
/// the strip of `|x| ≤ 4` past the far side of the circle, `(0, 5.5)` among it.
/// Each cap winds its own sides instead, the arc read as the chord along
/// `y = −3` with the piece of the disc past it taken away, and every answer is
/// worked out from the coordinates ([`on_the_slab_cap`]): the strip is turned
/// away, so are the points on the chord, inside the disc, and the points on the
/// circle's far arc, which is the pipe but not this face's side. Until row
/// Q1002 every point was refused by name, on both caps of both slabs.
#[test]
fn a_slab_with_a_round_scoop_out_of_its_top_holds_the_points_of_its_caps() {
    let mut counted = [0; 3];
    for (made, (model, solid)) in [
        ("built face by face", {
            let mut model = a_model();
            let solid = a_slab_with_a_round_scoop_out_of_it_built_face_by_face(&mut model);
            (model, solid)
        }),
        ("fused", a_slab_with_a_round_scoop_out_of_it_fused()),
    ] {
        for (z, facing) in [(0.0, -1.0), (4.0, 1.0)] {
            let cap = the_only_face(&model, solid, |surface| {
                surface
                    .as_plane()
                    .is_some_and(|flat| flat.normal().components() == [0.0, 0.0, facing])
            });
            assert_eq!(
                sides_of_every_neighbour(&model, solid, cap, [0.0, -5.5, z]),
                sides_of_every_neighbour(&model, solid, cap, [0.0, 5.5, z]),
                "the cap and the strip past the circle are on the same side of every cut"
            );
            for x in [
                -5.0, -4.0, -3.5, -3.0, -2.0, 0.0, 0.5, 2.0, 3.0, 3.5, 4.0, 4.5,
            ] {
                for y in [
                    -7.0, -6.0, -5.5, -5.0, -4.5, -4.0, -3.0, -2.0, 0.0, 3.0, 4.0, 4.5, 5.0, 5.5,
                    7.0,
                ] {
                    let expected = on_the_slab_cap([x, y]);
                    counted[class_of(expected)] += 1;
                    assert_eq!(
                        holds(&model, solid, cap, [x, y, z]),
                        Ok(expected),
                        "the slab {made}, its cap at z = {z}, at ({x}, {y})"
                    );
                }
            }
        }
    }
    assert_eq!(counted, [84, 92, 544]);
}

/// The slab built face by face places points, and less the block
/// `2 × 1.5 × 2` at `(−1, −7, 1)` it is one solid of the volume worked out by
/// hand: the slab's `4 · (24 − (12 + 25 asin 0.8 − 24))` — four tall over the
/// rectangle `8 × 3` less the part of the disc of radius 5 inside it, which is
/// the sector `25 asin 0.8` and the two triangles either side, `12`, less the
/// rectangle's part above `y = −3`, `24` — less the block's overlap with the
/// slab, `2 × 0.5 × 2`. Before row Q1002 the caps refused, so the slab placed
/// no point inside it and the cut was refused `Undecidable`, naming a point in
/// a piece no solid could place.
#[test]
fn a_slab_with_a_round_scoop_out_of_it_places_its_points_and_takes_a_cut() {
    let policy = Policy::certified();
    let mut model = a_model();
    let slab = a_slab_with_a_round_scoop_out_of_it_built_face_by_face(&mut model);
    let view = model.solid(slab).expect("this model made the key");
    for (point, expected) in [
        ([0.0, -5.5, 2.0], Containment::Inside),
        ([3.5, -4.5, 1.0], Containment::Inside),
        ([0.0, -4.0, 2.0], Containment::Outside),
        ([0.0, 5.5, 2.0], Containment::Outside),
        ([0.0, -5.5, 4.5], Containment::Outside),
    ] {
        let mut log = ProvenanceLog::new();
        let [x, y, z] = point;
        assert_eq!(
            view.classify_point(Point::new(x, y, z), &policy, &mut log),
            expected,
            "the slab at {point:?}"
        );
        assert!(log.is_empty(), "a certified answer logs nothing: {log:?}");
    }

    let knife = a_block(&mut model, [-1.0, -7.0, 1.0], [2.0, 1.5, 2.0]);
    let cut = the_one_solid(&mut model, slab, knife, Rule::Difference);
    let measured = model
        .solid(cut)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the cut slab measures");
    let by_hand = 4.0 * (24.0 - (12.0 + 25.0 * 0.8_f64.asin() - 24.0)) - 2.0;
    assert!(
        (measured.volume() - by_hand).abs() <= measured.volume_error(),
        "the cut slab measures {} ± {} against {by_hand} by hand",
        measured.volume(),
        measured.volume_error()
    );
}

/// The slab with a round scoop out of its top as a block `|x| ≤ 4`,
/// `−6 ≤ y ≤ −2`, four tall,
/// less a bar of radius 5 from `z = −1` to `z = 5`: the bar takes all of the
/// block above `y = −3`, so its side at `y = −2` goes with it.
fn a_slab_with_a_round_scoop_out_of_it_fused() -> (Model, SolidKey) {
    let mut model = a_model();
    let block = a_block(&mut model, [-4.0, -6.0, 0.0], [8.0, 4.0, 4.0]);
    let bar = model
        .add_round_bar(5.0, 6.0, &at([0.0, 0.0, -1.0]))
        .expect("a bar of positive size");
    let solid = the_one_solid(&mut model, block, bar, Rule::Difference);
    (model, solid)
}

/// The slab `|x| ≤ 4`, `−6 ≤ y ≤ −3`, four tall, less the pipe of radius five
/// about `z`, built face by face as
/// `crates/rcad-step/tests/a_solid_written_out_and_read_back_is_the_same_solid.rs`
/// builds it: bottom cap, scoop, top cap, then the three flat sides.
fn a_slab_with_a_round_scoop_out_of_it_built_face_by_face(model: &mut Model) -> SolidKey {
    use rcad_geom::{Axis, Cylinder, EdgeRun, HalfSpace, Orientation, Picker, Plane};
    use rcad_topo::{DraftStep, SolidBuilder};

    let height = 4.0;
    let a_plane = |at: Point, facing: Vector| {
        Plane::from_point_and_normal(at, facing).expect("an axis direction has length")
    };
    let bottom = a_plane(Point::origin(), Vector::new(0.0, 0.0, -1.0));
    let top = a_plane(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0));
    let the_short_arc_on = |cap: Plane| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(cap),
        picked_by: [Picker::from(
            HalfSpace::through_a_point_facing(
                Point::new(0.0, -3.0, 0.0),
                Vector::new(0.0, -1.0, 0.0),
            )
            .expect("an axis direction has length"),
        )]
        .into(),
    };

    let mut builder = SolidBuilder::new();
    let corners = [
        (4.0, -3.0, 0.0),
        (-4.0, -3.0, 0.0),
        (4.0, -3.0, height),
        (-4.0, -3.0, height),
        (4.0, -6.0, 0.0),
        (-4.0, -6.0, 0.0),
        (4.0, -6.0, height),
        (-4.0, -6.0, height),
    ]
    .map(|(x, y, z)| {
        builder
            .add_vertex(Point::new(x, y, z))
            .expect("a finite corner")
    });
    let edges = [
        (0, 1, the_short_arc_on(bottom)),
        (2, 3, the_short_arc_on(top)),
        (0, 2, EdgeRun::Straight),
        (1, 3, EdgeRun::Straight),
        (4, 5, EdgeRun::Straight),
        (6, 7, EdgeRun::Straight),
        (0, 4, EdgeRun::Straight),
        (1, 5, EdgeRun::Straight),
        (2, 6, EdgeRun::Straight),
        (3, 7, EdgeRun::Straight),
        (4, 6, EdgeRun::Straight),
        (5, 7, EdgeRun::Straight),
    ]
    .map(|(from, to, run)| {
        builder
            .add_edge(corners[from], corners[to], run)
            .expect("an edge between two corners")
    });

    let (forward, back) = (Orientation::Forward, Orientation::Reversed);
    let faces = [
        (Surface::Plane(bottom), forward),
        (
            Surface::Cylinder(Cylinder::new(Axis::z(), 5.0).expect("a positive radius")),
            back,
        ),
        (Surface::Plane(top), forward),
        (
            Surface::Plane(a_plane(
                Point::new(4.0, 0.0, 0.0),
                Vector::new(1.0, 0.0, 0.0),
            )),
            forward,
        ),
        (
            Surface::Plane(a_plane(
                Point::new(-4.0, 0.0, 0.0),
                Vector::new(-1.0, 0.0, 0.0),
            )),
            forward,
        ),
        (
            Surface::Plane(a_plane(
                Point::new(0.0, -6.0, 0.0),
                Vector::new(0.0, -1.0, 0.0),
            )),
            forward,
        ),
    ];
    let wires: [&[(usize, Orientation)]; 6] = [
        &[(4, forward), (7, back), (0, back), (6, forward)],
        &[(0, forward), (3, forward), (1, back), (2, back)],
        &[(8, back), (1, forward), (9, forward), (5, back)],
        &[(6, back), (2, forward), (8, forward), (10, back)],
        &[(7, forward), (11, forward), (9, back), (3, back)],
        &[(4, back), (10, forward), (5, forward), (11, back)],
    ];
    let mut skin = Vec::with_capacity(faces.len());
    for ((surface, used), steps) in faces.into_iter().zip(wires) {
        let walked: Vec<DraftStep> = steps
            .iter()
            .map(|&(which, way)| {
                DraftStep::Along(builder.add_coedge(edges[which], way).expect("a coedge"))
            })
            .collect();
        let wire = builder.add_wire(&walked).expect("a wire that closes");
        skin.push((builder.add_face(surface, &[wire]).expect("a face"), used));
    }
    builder.add_shell(&skin).expect("a shell");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the slab with a round scoop out of it is not sound: {why}"))
}

fn a_quarter_of_a_bar(bar_first: bool) -> (Model, SolidKey) {
    let mut model = a_model();
    let bar = a_bar(&mut model, 5.0, [0.0, 0.0, 0.0]);
    let block = a_block(&mut model, [0.0, 0.0, -1.0], [10.0, 10.0, 6.0]);
    let [one, other] = if bar_first {
        [bar, block]
    } else {
        [block, bar]
    };
    let solid = the_one_solid(&mut model, one, other, Rule::Intersection);
    (model, solid)
}

/// A bar of radius 5 from `z = 0` to `z = 4`, less the part outside `|x| ≤ 3`.
fn a_bar_with_two_flats() -> (Model, SolidKey) {
    let mut model = a_model();
    let bar = a_bar(&mut model, 5.0, [0.0, 0.0, 0.0]);
    let slab = a_block(&mut model, [-3.0, -10.0, -1.0], [6.0, 20.0, 6.0]);
    let solid = the_one_solid(&mut model, bar, slab, Rule::Intersection);
    (model, solid)
}

/// A bar of radius 5 from `z = 0` to `z = 4`, less the part outside `x ≤ 4`
/// and `y ≤ 4`.
fn a_bar_with_two_flats_meeting_outside_the_pipe() -> (Model, SolidKey) {
    let mut model = a_model();
    let bar = a_bar(&mut model, 5.0, [0.0, 0.0, 0.0]);
    let block = a_block(&mut model, [-10.0, -10.0, -1.0], [14.0, 14.0, 6.0]);
    let solid = the_one_solid(&mut model, bar, block, Rule::Intersection);
    (model, solid)
}

/// A block `10 × 10 × 4` less a bar of radius 3 standing on `(0, 5)`, which is
/// the middle of the block's edge along `x = 0`, and running out through the
/// floor and the top.
fn a_block_with_a_bar_bored_through_one_edge() -> (Model, SolidKey) {
    let mut model = a_model();
    let block = a_block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let bar = a_bar(&mut model, 3.0, [0.0, 5.0, -1.0]);
    let solid = the_one_solid(&mut model, block, bar, Rule::Difference);
    (model, solid)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A bar of `radius`, 4 tall from `foot` for a radius of 5 and 6 tall for the
/// thinner bores, standing up `z`.
fn a_bar(model: &mut Model, radius: f64, foot: [f64; 3]) -> SolidKey {
    let height = if radius == 5.0 { 4.0 } else { 6.0 };
    model
        .add_round_bar(radius, height, &at(foot))
        .expect("a bar of positive size")
}

fn a_block(model: &mut Model, low: [f64; 3], size: [f64; 3]) -> SolidKey {
    let [x, y, z] = size;
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [x, 0.0], [x, y], [0.0, y]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    model
        .add_extrusion(&rectangle, z, &at(low))
        .expect("a positive height")
}

fn at(shift: [f64; 3]) -> Isometry {
    let [x, y, z] = shift;
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&[one], &[other], rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("{rule:?} is refused: {why:?}"));
    assert!(log.is_empty(), "a certified fuse logs nothing: {log:?}");
    let [solid] = answer.solids()[..] else {
        panic!("{rule:?} is one solid, not {:?}", answer.solids());
    };
    solid
}

fn the_faces(model: &Model, solid: SolidKey, wanted: impl Fn(&Surface) -> bool) -> Vec<FaceKey> {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .filter(|face| wanted(&face.surface()))
        .map(|face| face.key())
        .collect()
}

fn the_only_face(model: &Model, solid: SolidKey, wanted: impl Fn(&Surface) -> bool) -> FaceKey {
    let faces = the_faces(model, solid, wanted);
    let [face] = faces[..] else {
        panic!("one face is wanted, and {} are", faces.len());
    };
    face
}

fn on_the_pipe(radius: f64, degrees: f64, z: f64) -> [f64; 3] {
    let turn = degrees * PI / 180.0;
    [radius * turn.cos(), radius * turn.sin(), z]
}

/// What `face` says of `point`, or the face a refusal by name names.
fn holds(
    model: &Model,
    solid: SolidKey,
    face: FaceKey,
    point: [f64; 3],
) -> Result<OnAFace, FaceKey> {
    let mut log = ProvenanceLog::new();
    let answer = model
        .solid(solid)
        .expect("this model made the key")
        .face_holds_the_point(
            face,
            rcad_predicates::Point::from(point),
            &Policy::certified(),
            &mut log,
        );
    assert!(log.is_empty(), "a certified answer logs nothing: {log:?}");
    match answer {
        Ok(found) => Ok(found),
        Err(Error::AFaceTrimmedByACurveThisCannotFollow { face: named }) => Err(named),
        Err(other) => panic!("{point:?} on {face:?} is refused some other way: {other:?}"),
    }
}

/// Which way round the axis a wall of the bar with two flats runs, read off its
/// corners: `1.0` round `+y`, `-1.0` round `−y`.
fn which_way_round(model: &Model, solid: SolidKey, wall: FaceKey) -> f64 {
    let face = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .find(|used| used.key() == wall)
        .expect("a face of this solid");
    let ys: Vec<f64> = face
        .wires()
        .flat_map(|wire| wire.points().collect::<Vec<_>>())
        .map(|corner| corner.coordinates()[1])
        .collect();
    let round = ys[0].signum();
    assert!(
        ys.iter().all(|y| y.signum() == round && y.abs() > 3.0),
        "a wall's corners are where the flats cross the pipe, all on one side: {ys:?}"
    );
    round
}

/// Where a point on the pipe of radius 5 is against the wall of the bar with
/// flats at `|x| ≤ 3` that runs `round` times `+y`, worked out from its
/// coordinates: between the caps, between the flats and on the wall's own side
/// of `y = 0`, and on a cap or a flat when it is on its boundary. No point on
/// the pipe between the flats is on `y = 0`.
fn on_a_wall_of_the_bar_with_two_flats(round: f64, point: [f64; 3]) -> OnAFace {
    let [x, y, z] = point;
    if !(0.0..=4.0).contains(&z) || x.abs() > 3.0 || round * y < 0.0 {
        OnAFace::NotHeld
    } else if z == 0.0 || z == 4.0 || x.abs() == 3.0 {
        OnAFace::OnItsBoundary
    } else {
        OnAFace::Held
    }
}

/// Whether a wall of the bar with two flats meeting outside the pipe is the
/// short run between the flats' own crossings, `(4, 3)` to `(3, 4)`, read off
/// its corners: both have `x ≥ 3` and `y ≥ 3` for the short run, and neither
/// does for the long run that is the rest of the bar.
fn is_the_near_run_of_two_flats_meeting_outside_the_pipe(
    model: &Model,
    solid: SolidKey,
    wall: FaceKey,
) -> bool {
    let face = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .find(|used| used.key() == wall)
        .expect("a face of this solid");
    let corners: Vec<[f64; 3]> = face
        .wires()
        .flat_map(|wire| wire.points().collect::<Vec<_>>())
        .map(|corner| corner.coordinates())
        .collect();
    let near = corners[0][0] >= 3.0 && corners[0][1] >= 3.0;
    assert!(
        corners
            .iter()
            .all(|corner| (corner[0] >= 3.0 && corner[1] >= 3.0) == near),
        "a wall's corners are where the flats cross the pipe, all the same run: {corners:?}"
    );
    near
}

/// Where a point on the pipe of radius 5 is against the wall of the bar with
/// flats at `x ≤ 4` and `y ≤ 4` that is the short run between the flats' own
/// crossings when `near` is `true`, and the rest of the bar otherwise: between
/// the caps, kept by both flats, in that run, and on its boundary on a cap or
/// on a flat, `x = 4` or `y = 4`. No point of the pipe in one run is on the
/// other flat's crossing: `(−4, 3)` and `(3, −4)`, on the circle but not on
/// either flat, are interior to the long run.
fn on_a_wall_of_two_flats_meeting_outside_the_pipe(near: bool, point: [f64; 3]) -> OnAFace {
    let [x, y, z] = point;
    let kept_by_both_flats = x <= 4.0 && y <= 4.0;
    let in_this_run = if near {
        x >= 3.0 && y >= 3.0
    } else {
        x < 3.0 || y < 3.0
    };
    if !kept_by_both_flats || !in_this_run || !(0.0..=4.0).contains(&z) {
        OnAFace::NotHeld
    } else if z == 0.0 || z == 4.0 || x == 4.0 || y == 4.0 {
        OnAFace::OnItsBoundary
    } else {
        OnAFace::Held
    }
}

/// Where a point of the plane of the floor or the top of the block bored
/// through one edge is against that face, worked out from its coordinates: the
/// square `0 ≤ x, y ≤ 10` less the inside of the circle of radius 3 round
/// `(0, 5)`, on its boundary on the edge of the square or on the circle. Every
/// coordinate asked is a whole number or a half, so the distance is exact.
fn on_the_square_less_half_a_disc(point: [f64; 2]) -> OnAFace {
    let [x, y] = point;
    let from_the_axis = x * x + (y - 5.0) * (y - 5.0);
    let in_the_square = (0.0..=10.0).contains(&x) && (0.0..=10.0).contains(&y);
    if !in_the_square || from_the_axis < 9.0 {
        OnAFace::NotHeld
    } else if from_the_axis == 9.0 || x == 0.0 || x == 10.0 || y == 0.0 || y == 10.0 {
        OnAFace::OnItsBoundary
    } else {
        OnAFace::Held
    }
}

/// Which side of `y = 0` every corner of `face` is on: `1.0` or `-1.0`.
fn which_side_of_y_is_zero(model: &Model, solid: SolidKey, face: FaceKey) -> f64 {
    let face = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .find(|used| used.key() == face)
        .expect("a face of this solid");
    let ys: Vec<f64> = face
        .wires()
        .flat_map(|wire| wire.points().collect::<Vec<_>>())
        .map(|corner| corner.coordinates()[1])
        .collect();
    let side = ys[0].signum();
    assert!(
        ys.iter().all(|y| y.signum() == side && *y != 0.0),
        "every corner of a half of the top is on one side of y = 0: {ys:?}"
    );
    side
}

/// Where a point of the plane `z = 4` is against the half of the narrow top on
/// `side` of `y = 0`, worked out from its coordinates: `|x| ≤ 1` and `|y| ≤ 6`
/// on that side, outside the circle of radius 3 round the axis, and on its
/// boundary on the circle or on the block's edge. Every coordinate asked is a
/// whole number or a half, so the distance is exact.
fn on_one_half_of_the_narrow_top(side: f64, point: [f64; 2]) -> OnAFace {
    let [x, y] = point;
    let from_the_axis = x * x + y * y;
    let in_it = x.abs() <= 1.0 && side * y >= 0.0 && y.abs() <= 6.0 && from_the_axis >= 9.0;
    if !in_it {
        OnAFace::NotHeld
    } else if from_the_axis == 9.0 || x.abs() == 1.0 || y.abs() == 6.0 {
        OnAFace::OnItsBoundary
    } else {
        OnAFace::Held
    }
}

/// Where a point of the plane of the floor or the top of the block bored
/// through one corner is against that face, worked out from its coordinates:
/// the square `0 ≤ x, y ≤ 10` less the inside of the circle of radius 3 round
/// the origin, on its boundary on the edge of the square or on the circle.
fn on_the_square_less_a_quarter_disc(point: [f64; 2]) -> OnAFace {
    let [x, y] = point;
    let from_the_axis = x * x + y * y;
    let in_the_square = (0.0..=10.0).contains(&x) && (0.0..=10.0).contains(&y);
    if !in_the_square || from_the_axis < 9.0 {
        OnAFace::NotHeld
    } else if from_the_axis == 9.0 || x == 0.0 || x == 10.0 || y == 0.0 || y == 10.0 {
        OnAFace::OnItsBoundary
    } else {
        OnAFace::Held
    }
}

/// Where a point of a cap's plane is against a cap of the slab with a round
/// scoop out of its top, worked out from its coordinates: `|x| ≤ 4` and
/// `−6 ≤ y ≤ −3`, outside the circle of radius 5 round the axis, and on its
/// boundary on the circle, at `x = ±4` or at `y = −6`. The only points of
/// `y = −3` it holds are the two corners, which are on the circle.
fn on_the_slab_cap(point: [f64; 2]) -> OnAFace {
    let [x, y] = point;
    let from_the_axis = x * x + y * y;
    let in_it = x.abs() <= 4.0 && (-6.0..=-3.0).contains(&y) && from_the_axis >= 25.0;
    if !in_it {
        OnAFace::NotHeld
    } else if from_the_axis == 25.0 || x.abs() == 4.0 || y == -6.0 {
        OnAFace::OnItsBoundary
    } else {
        OnAFace::Held
    }
}

/// Where an answer is counted: held, on the boundary, not held.
fn class_of(answer: OnAFace) -> usize {
    match answer {
        OnAFace::Held => 0,
        OnAFace::OnItsBoundary => 1,
        OnAFace::NotHeld => 2,
        other => unreachable!("the test's own answer is one of three: {other:?}"),
    }
}

/// Which side of each face next to `face` the point is on, read off the
/// surfaces as plain numbers: the test's own reading, for points a long way
/// from every one of them.
fn sides_of_every_neighbour(
    model: &Model,
    solid: SolidKey,
    face: FaceKey,
    point: [f64; 3],
) -> Vec<bool> {
    let view = model.solid(solid).expect("this model made the key");
    let this = view
        .faces()
        .find(|used| used.key() == face)
        .expect("a face of this solid");
    let mut neighbours: Vec<FaceKey> = this
        .wires()
        .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
        .flat_map(|coedge| coedge.edge().neighbouring_faces().collect::<Vec<_>>())
        .filter(|other| *other != face)
        .collect();
    neighbours.sort_by_key(|key| format!("{key:?}"));
    neighbours.dedup();
    neighbours
        .into_iter()
        .map(|key| {
            let neighbour = view
                .faces()
                .find(|used| used.key() == key)
                .expect("a neighbour in this solid");
            let value = signed_value(&neighbour.surface(), point);
            assert!(value.abs() > 0.1, "{point:?} is clear of every neighbour");
            value > 0.0
        })
        .collect()
}

fn signed_value(surface: &Surface, point: [f64; 3]) -> f64 {
    match surface {
        Surface::Plane(flat) => {
            let normal = flat.normal().components();
            (0..3).map(|axis| normal[axis] * point[axis]).sum::<f64>() - flat.offset()
        }
        Surface::Cylinder(pipe) => {
            let origin = pipe.axis().origin().coordinates();
            let [x, y, _] = core::array::from_fn::<f64, 3, _>(|axis| point[axis] - origin[axis]);
            assert_eq!(
                pipe.axis().direction().components(),
                [0.0, 0.0, 1.0],
                "the bores here stand up z"
            );
            x * x + y * y - pipe.radius() * pipe.radius()
        }
        Surface::Sphere(_) | Surface::Cone(_) | Surface::Torus(_) | _ => {
            panic!("only planes and the bore are next to the faces asked about here")
        }
    }
}
