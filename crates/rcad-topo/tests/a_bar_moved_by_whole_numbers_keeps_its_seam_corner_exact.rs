//! A bar moved by whole numbers keeps its seam corner exact.
//!
//! A round bar's cap corner is read out of the model and moved rather than
//! solved for, and the move used to cost it a box every time: half a width of
//! `CORNER_AFTER_MOVING_ROUNDOFF` times its size, whatever the move was. A shift
//! by whole numbers rounds nothing, so the box hid an exact answer. A bar whose
//! seam lies exactly on a block's wall then asked *side of plane* about a box
//! round `(0, 2, 1)` on `x = 0`, and refused, where the answer is `Zero`.
//!
//! This is a gap and not a limit (§16.10): the corner is exactly representable,
//! and only the box hid it.
//!
//! The corner is exact only when every surface round it moved exactly, so the
//! fuse asks about every one of them and not only the first and last face round
//! the corner (row Q1567). A tilted bar whose wall is two half pipes has the
//! wall first and last round the corner at its top, and its top between them.
//! Moved along `x`, the top's offset rounds and the wall does not, and the box
//! stays on that corner.

#![forbid(unsafe_code)]

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point,
    Scale, Surface, Vector, where_a_corner_lands_after_moving,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftStep, DraftWire, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey,
};

/// How far under the closed form a bar's meshed volume may be.
///
/// A mesh at [`A_DEFLECTION`] round a circle of radius one is 0.33% short of
/// it. A ceiling on a measurement, not a tolerance anything is decided by.
const HOW_FAR_A_MESH_MAY_BE: f64 = 0.005;

/// How fine the meshes the volumes are read off are.
const A_DEFLECTION: f64 = 0.02;

/// The seam corner of a bar of radius one, moved by whole numbers, lands on
/// the whole-number point it was moved to and on no box round it; moved by a
/// tenth, which rounds, it keeps its box.
#[test]
fn a_bar_moved_by_whole_numbers_lands_its_seam_corner_on_an_exact_point() {
    let (corner, [cap, wall_of_the_bar]) = the_seam_corner_of_a_bar_and_its_surfaces();
    let on = [&cap, &wall_of_the_bar];

    let whole = Isometry::translation(Vector::new(-1.0, 2.0, 1.0)).expect("a whole-number shift");
    let (landed, wobble) = where_a_corner_lands_after_moving(corner, on, &whole)
        .expect("a finite corner and a finite shift");
    assert_eq!(
        landed,
        PredicatePoint::from([0.0, 2.0, 1.0]),
        "a whole-number shift of (1, 0, 0) rounds nothing"
    );
    assert_eq!(wobble.magnitude(), 0.0);

    let wall = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal");
    assert_eq!(
        wall.side_of_point(landed),
        Decision::Zero,
        "the seam corner is on the wall x = 0"
    );

    let a_tenth = Isometry::translation(Vector::new(0.1, 2.0, 1.0)).expect("a finite shift");
    let (boxed, wobble) = where_a_corner_lands_after_moving(corner, on, &a_tenth)
        .expect("a finite corner and a finite shift");
    assert!(
        boxed.uncertainty() > 0.0 && wobble.magnitude() > 0.0,
        "1 + 0.1 rounds, so the corner is a box and not a point: {boxed:?}"
    );
}

/// A bar standing with its seam on a block's wall, handed over first: every
/// rule used to stop on *side of plane*, asked about the seam corner's box, and
/// none is refused on it now.
///
/// `ABarBesideAWall { side: 4, radius: 1, along: 2, on_its_seam: true }` in
/// `fuses_that_touch.rs`. The bar fills `x` from −2 to 0 and the cube `x` from 0
/// to 4, so they meet only along the seam `(0, 2, z)`: the union is the two
/// solids as they stand, two lumps of `π · 1² · 2` and 64, the intersection is
/// no solid, and the difference is the bar whole.
#[test]
fn a_bar_with_its_seam_on_a_blocks_wall_is_not_refused_on_which_side_of_the_wall_its_corner_is() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = a_cube(&mut model, 4.0);
    let bar = model
        .add_round_bar(
            1.0,
            2.0,
            &Isometry::translation(Vector::new(-1.0, 2.0, 1.0)).expect("a whole-number shift"),
        )
        .expect("a whole-number bar");
    let the_bar = PI * 1.0 * 1.0 * 2.0;

    let mut joined = model.clone();
    let both = answered(&mut joined, bar, cube, Rule::Union);
    let mut found: Vec<f64> = both
        .iter()
        .map(|&solid| volume_of(&joined, solid))
        .collect();
    found.sort_by(f64::total_cmp);
    assert!(
        matches!(found[..], [bar, cube]
            if bar <= the_bar && (the_bar - bar) / the_bar < HOW_FAR_A_MESH_MAY_BE
                && cube == 64.0),
        "the union is the bar and the cube as they stand, measured {found:?}"
    );

    let mut shared = model.clone();
    assert!(
        answered(&mut shared, bar, cube, Rule::Intersection).is_empty(),
        "a bar and a cube whose insides are apart share no solid"
    );

    let mut cut = model.clone();
    let what_is_left = answered(&mut cut, bar, cube, Rule::Difference);
    assert_eq!(what_is_left.len(), 1, "the cube takes nothing off the bar");
    let found = volume_of(&cut, what_is_left[0]);
    assert!(
        found <= the_bar && (the_bar - found) / the_bar < HOW_FAR_A_MESH_MAY_BE,
        "the bar less the cube measured {found}, and the bar is {the_bar}"
    );
}

/// Moved one along `x`, the corner `(3, 1, 4)` at the top of
/// [`a_tilted_bar`] is a box: the top's offset, 5, gains 0.6 and rounds,
/// though the wall and the corner itself move exactly. Asked about the wall
/// alone, it is a point.
///
/// The corner has a half pipe first and last round it and the top between
/// them, so asking about the first and the last face is asking about the wall
/// twice. Moved along `y`, nothing rounds, and it is a point whichever
/// surfaces are asked.
#[test]
fn the_top_of_a_tilted_bar_rounds_when_moved_along_x_and_its_wall_does_not() {
    let corner = Point::new(3.0, 1.0, 4.0);
    let wall = Surface::Cylinder(the_pipe());
    let top = Surface::Plane(the_top());

    let along_x = Isometry::translation(Vector::new(1.0, 0.0, 0.0)).expect("a finite shift");
    let (every_surface, _) =
        where_a_corner_lands_after_moving(corner, [&wall, &top, &wall], &along_x)
            .expect("a finite corner and a finite shift");
    assert!(
        every_surface.uncertainty() > 0.0,
        "5 + 0.6 rounds, so the corner on the moved top is a box: {every_surface:?}"
    );
    let (the_wall_twice, _) = where_a_corner_lands_after_moving(corner, [&wall, &wall], &along_x)
        .expect("a finite corner and a finite shift");
    assert_eq!(
        the_wall_twice,
        PredicatePoint::from([4.0, 1.0, 4.0]),
        "the wall moves exactly, so asked about the wall alone the corner is a point"
    );

    let along_y = Isometry::translation(Vector::new(0.0, 2.0, 0.0)).expect("a finite shift");
    let (every_surface, _) =
        where_a_corner_lands_after_moving(corner, [&wall, &top, &wall], &along_y)
            .expect("a finite corner and a finite shift");
    assert_eq!(
        every_surface,
        PredicatePoint::from([3.0, 3.0, 4.0]),
        "the top's normal has no y, so a shift along y leaves its offset alone"
    );
}

/// A tilted bar moved one along `x` and fused with a cube that bites its foot
/// comes back with a box on the corner at its top, `(4, 1, 4)`, because its top
/// rounded on the move. Moved along `y` instead, nothing rounds, and the corner
/// comes back with no box.
///
/// The cube is half a unit on a side, centred on the middle of the bar's foot,
/// so the fuse bakes the bar and writes it again, and the corner is five away
/// from anything the cube cuts. The foot's plane passes through the cube's
/// centre and cuts it in two equal halves, so the union is the bar and half the
/// cube, `5π + 0.5³ / 2`.
///
/// Asked about the first and last face round the corner only, which are the
/// two halves of the wall, the corner came back exact, and the fuse wrote it
/// with no box though the top it is on had rounded.
#[test]
fn a_tilted_bar_moved_along_x_keeps_the_box_its_top_puts_round_its_corner() {
    for (shift, the_top_rounds) in [([1.0, 0.0, 0.0], true), ([0.0, 2.0, 0.0], false)] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let bar = a_tilted_bar(&mut model);
        model
            .place_solid(
                bar,
                &Isometry::translation(Vector::from(shift)).expect("a finite shift"),
            )
            .expect("the bar is in this model");
        let cube = a_cube(&mut model, 1.0);
        model
            .place_solid(
                cube,
                &Isometry::translation(Vector::new(10.0, 10.0, 10.0)).expect("a finite shift"),
            )
            .expect("the cube is in this model");

        let union = answered(&mut model, bar, cube, Rule::Union);
        let mut found: Vec<f64> = union
            .iter()
            .map(|&solid| volume_of(&model, solid))
            .collect();
        found.sort_by(f64::total_cmp);
        let the_bar = 5.0 * PI;
        assert!(
            matches!(found[..], [cube, bar]
                if cube == 1.0 && bar <= the_bar && (the_bar - bar) / the_bar < HOW_FAR_A_MESH_MAY_BE),
            "{shift:?}: the union is the cube and the bar as they stand, measured {found:?}"
        );

        let corner = Point::new(3.0, 1.0, 4.0) + Vector::from(shift);
        let box_written = how_wide_a_box_is_written_on(&model, &union, corner);
        if the_top_rounds {
            assert!(
                box_written > 0.0,
                "{shift:?}: the top's offset rounded, so the corner {corner:?} keeps a box"
            );
        } else {
            assert_eq!(
                box_written, 0.0,
                "{shift:?}: nothing rounded, so the corner {corner:?} is exact"
            );
        }
    }
}

/// How wide the box the model writes on the corner of `solids` at `corner` is,
/// zero when it writes none.
///
/// # Panics
///
/// If none of `solids` has a corner there.
fn how_wide_a_box_is_written_on(model: &Model, solids: &[SolidKey], corner: Point) -> f64 {
    let mut seen = Vec::new();
    for &solid in solids {
        for face in model.solid(solid).expect("this model made the key").faces() {
            for wire in face.wires() {
                for coedge in wire.coedges() {
                    let vertex = coedge.start();
                    if vertex.point() == corner {
                        return vertex
                            .approximation()
                            .map_or(0.0, |approximation| approximation.magnitude());
                    }
                    seen.push(vertex.point());
                }
            }
        }
    }
    panic!("no solid has a corner at {corner:?}, only at {seen:?}")
}

/// Which way the axis of [`a_tilted_bar`] runs, from the origin to the middle
/// of its top: `(3, 0, 4)`, five long.
fn along_the_axis() -> Vector {
    Vector::new(3.0, 0.0, 4.0)
}

/// The pipe the wall of [`a_tilted_bar`] is on: radius one, round the axis
/// through the origin along [`along_the_axis`].
fn the_pipe() -> Cylinder {
    Cylinder::new(
        Axis::new(Point::origin(), along_the_axis()).expect("a direction with a length"),
        1.0,
    )
    .expect("a radius of one")
}

/// The plane the top of [`a_tilted_bar`] is on: square to its axis, through
/// `(3, 0, 4)`, so its offset is 5.
///
/// Its normal's `x` is the `f64` nearest 0.6, whose last bits fall below the
/// last bit of a number near 5, so a shift of one along `x` rounds the offset.
fn the_top() -> Plane {
    Plane::from_point_and_normal(Point::origin() + along_the_axis(), along_the_axis())
        .expect("a direction with a length")
}

/// A round bar of radius one and length five, lying along [`along_the_axis`]
/// from its foot at the origin, whose wall is two half pipes split along the
/// lines through `(0, ±1, 0)`.
///
/// Its faces are added in the order wall on one side, top, wall on the other
/// side, foot. The fuse lists the faces round a corner in the order they were
/// added, so the corner `(3, 1, 4)` at the top has a half pipe first and last
/// round it and the top between them. Its caps are square to its axis, so each
/// meets the wall in a circle.
fn a_tilted_bar(model: &mut Model) -> SolidKey {
    let wall = Surface::Cylinder(the_pipe());
    let top = the_top();
    let foot = Plane::from_point_and_normal(Point::origin(), -along_the_axis())
        .expect("a direction with a length");

    let mut draft = SolidBuilder::new();
    let mut corner_at = |x: f64, y: f64, z: f64| {
        draft
            .add_vertex(Point::new(x, y, z))
            .expect("a finite corner")
    };
    let (foot_a, foot_b, top_a, top_b) = (
        corner_at(0.0, 1.0, 0.0),
        corner_at(0.0, -1.0, 0.0),
        corner_at(3.0, 1.0, 4.0),
        corner_at(3.0, -1.0, 4.0),
    );
    let runs = [
        (foot_a, foot_b, half_way_round_on(foot, 1.0)),
        (foot_b, foot_a, half_way_round_on(foot, -1.0)),
        (top_a, top_b, half_way_round_on(top, 1.0)),
        (top_b, top_a, half_way_round_on(top, -1.0)),
        (foot_a, top_a, EdgeRun::Straight),
        (foot_b, top_b, EdgeRun::Straight),
    ];
    let mut edges = Vec::with_capacity(runs.len());
    for (from, to, run) in runs {
        edges.push(
            draft
                .add_edge(from, to, run)
                .expect("an edge between two corners"),
        );
    }
    let [foot_one, foot_other, top_one, top_other, seam_a, seam_b] = edges[..] else {
        unreachable!("six runs make six edges")
    };

    let (forward, reversed) = (Orientation::Forward, Orientation::Reversed);
    let one_side = a_wire(
        &mut draft,
        &[
            (foot_one, forward),
            (seam_b, forward),
            (top_one, reversed),
            (seam_a, reversed),
        ],
    );
    let round_the_top = a_wire(&mut draft, &[(top_one, forward), (top_other, forward)]);
    let other_side = a_wire(
        &mut draft,
        &[
            (foot_other, forward),
            (seam_a, forward),
            (top_other, reversed),
            (seam_b, reversed),
        ],
    );
    let round_the_foot = a_wire(&mut draft, &[(foot_other, reversed), (foot_one, reversed)]);

    let faces = [
        draft.add_face(wall, &[one_side]),
        draft.add_face(Surface::Plane(top), &[round_the_top]),
        draft.add_face(wall, &[other_side]),
        draft.add_face(Surface::Plane(foot), &[round_the_foot]),
    ]
    .map(|face| (face.expect("a face on its surface"), forward));
    draft.add_shell(&faces).expect("four faces close a shell");
    draft
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the tilted bar builds: {why}"))
}

/// Half of the circle where the pipe crosses the cap `plane`: the half on the
/// side `side` points to of the plane through the axis and `(0, ±1, 0)`.
fn half_way_round_on(plane: Plane, side: f64) -> EdgeRun {
    let across_the_axis = Vector::new(-4.0, 0.0, 3.0);
    EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(plane),
        picked_by: smallvec::smallvec![Picker::from(
            HalfSpace::through_a_point_facing(Point::origin(), across_the_axis * side)
                .expect("a direction with a length")
        )],
    }
}

fn a_wire(draft: &mut SolidBuilder, steps: &[(DraftEdge, Orientation)]) -> DraftWire {
    let steps: Vec<DraftStep> = steps
        .iter()
        .map(|&(edge, way)| {
            DraftStep::Along(draft.add_coedge(edge, way).expect("an edge of this draft"))
        })
        .collect();
    draft.add_wire(&steps).expect("the steps join end to end")
}

/// The corner a bar's seam starts from, and the two surfaces round it, as a bar
/// standing where it was built stores them.
fn the_seam_corner_of_a_bar_and_its_surfaces() -> (Point, [Surface; 2]) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(1.0, 2.0, &Isometry::identity())
        .expect("a whole-number bar");
    let corner = Point::new(1.0, 0.0, 0.0);
    assert!(
        model.vertices().any(|vertex| vertex.point() == corner),
        "a bar of radius one stands its seam at (1, 0, 0)"
    );
    let cap = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
            .expect("the z axis is a unit normal"),
    );
    let wall = Surface::Cylinder(Cylinder::new(Axis::z(), 1.0).expect("a radius of one"));
    let held: Vec<Surface> = model
        .solid(bar)
        .expect("the bar was just built")
        .faces()
        .map(|face| face.surface())
        .collect();
    assert!(
        held.contains(&cap) && held.contains(&wall),
        "the bar holds its bottom cap and its wall as these numbers: {held:?}"
    );
    (corner, [cap, wall])
}

fn a_cube(model: &mut Model, side: f64) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(&square, side, &Isometry::identity())
        .expect("building a box needs no decision")
}

/// The solids a certified fuse answers with, having checked that it proved every
/// decision and wrote a model that validates.
fn answered(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> Vec<SolidKey> {
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&[one], &[other], rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("{rule:?} of the bar and the cube refused: {why:?}"));
    assert!(
        log.records().is_empty(),
        "{rule:?}: every decision is proved, got {:?}",
        log.records()
    );
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{rule:?} wrote a model that does not validate: {why}"));
    answer.solids().to_vec()
}

fn volume_of(model: &Model, solid: SolidKey) -> f64 {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(A_DEFLECTION).expect("a fiftieth of a millimetre is a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures")
        .volume()
}
