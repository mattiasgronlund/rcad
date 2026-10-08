//! A cone's skin cut by planes whose open curves never reach each other's
//! plane, read at the apex, where the winding seen down the axis refuses and
//! only the steps that read the skin as the part in front of its cuts answer.
//!
//! # Two walls along the axis (row Q1235)
//!
//! The pointed cone of slope 1, apex at the origin and base at `z = 20`, is
//! cut by the block `[−12, 24] × [−24, 15] × [−1, 18]`. What is left of its
//! skin is the upper nappe below `z = 18` with `x > −12` and `y < 15`: one
//! piece, since the two walls' parts of the disc of radius 18 seen down the
//! axis are apart, their corner `(−12, 15)` being `√369` from the axis. Its
//! sides are the hyperbola on `x = −12` from `(−12, ±√180, 18)` down through
//! `(−12, 0, 12)`, the one on `y = 15` from `(±√99, 15, 18)` down through
//! `(0, 15, 15)`, and two arcs of the top circle between them.
//!
//! **Each hyperbola never reaches the other wall**: `x = −12`'s runs over
//! `|y| ≤ √180`, short of `15`, and `y = 15`'s over `|x| ≤ √99`, short of `12`.
//! The line where the two walls meet runs up the axis, steeper than the cone's
//! side, so it meets each wall's hyperbola once on each branch. `holds.rs` read
//! such a line only where the curve's part of degree two is positive along it,
//! so each hyperbola read as crossing the other wall, and the steps that read
//! the skin as the part in front of its three cuts said nothing. The winding
//! seen down the axis answers every other point of the skin, but not the apex,
//! which is where every ruling is seen, so the apex was refused
//! `AFaceTrimmedByACurveThisCannotFollow` and `classify_point` there answered
//! `CannotFollowAFacesTrimming`, though the apex is on the skin.
//!
//! # A wall and a slope meeting under the apex (row Q1391)
//!
//! The same cone cut by the prism `x ≥ −12`, `z ≥ 2x − 12`, `z ≤ 24`,
//! `|y| ≤ 24` keeps the upper nappe below `z = 20` with `x ≥ −12` and
//! `z ≥ 2x − 12`. Its sides are the hyperbola on `x = −12` from
//! `(−12, ±16, 20)` down through `(−12, 0, 12)`, the one on `z = 2x − 12` from
//! `(16, ±12, 20)` down through `(12, 0, 12)`, and two arcs of the base circle.
//! The two planes meet in the line `x = −12`, `z = −36`, square to the axis
//! and under the apex. On either plane that line misses the skin's branch of
//! the hyperbola and meets the other nappe's branch twice, at
//! `(−12, ±√1152, −36)`, both in front of each arc's chord, so each hyperbola
//! read as crossing the other plane and the apex was refused the same way.
//!
//! # A wall and a plane meeting it along a ruling (row Q1390)
//!
//! The same cone kept to `x ≥ −12` and to the origin's side of a plane whose
//! line with `x = −12` runs `(0, 1, −1)`, along a ruling, so parallel to an
//! asymptote of each plane's hyperbola: it meets each once. The plane is a face
//! of a tetrahedron built through `SolidBuilder`, since no builder turns a
//! block's face that way exactly. The part of degree two along that line is
//! zero exactly only when the plane's normal is stored in numbers whose squares
//! are exact, `(√2, 1, 1)` stored as `(0.7071…, 0.5, 0.5)`; there the apex was
//! refused until such a line was read as meeting the curve once. With the
//! normal `(2, 1, 1)` that part is not settled and the apex is still refused,
//! which the last test pins.

use std::collections::HashMap;

use rcad_geom::{Axis, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    Containment, DraftStep, Error, Model, OnAFace, ProfileBuilder, Rule, SolidBuilder, SolidKey,
    SolidView,
};

mod cones;

/// **The skin of a cone cut by two walls along its axis holds its own points,
/// the apex among them**, which is on its boundary as the uncut cone's apex is
/// (`a_pointed_cone_knows_its_inside.rs`). Every point asked is on the cone
/// exactly. The apex was refused until each wall's hyperbola was read against
/// the other wall's plane along a line steeper than the cone's side (Q1235);
/// the rest were answered before, seen down the axis, and are answered the
/// same now.
#[test]
fn the_skin_of_a_cone_cut_by_two_walls_along_its_axis_holds_its_apex() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cut = a_cone_cut_by_two_walls_along_its_axis(&mut model);
    let solid = model.solid(cut).expect("this model made the key");
    let corners = corners_of_the_skin(solid);
    assert_eq!(
        corners
            .iter()
            .filter(|corner| corner[0] == -12.0 && corner[2] == 18.0)
            .count(),
        2,
        "{corners:?}"
    );
    assert_eq!(
        corners
            .iter()
            .filter(|corner| corner[1] == 15.0 && corner[2] == 18.0)
            .count(),
        2,
        "{corners:?}"
    );
    assert_eq!(corners.len(), 4, "{corners:?}");
    let asked = |at| what_the_skin_holds(solid, at);
    assert_eq!(asked([0.0, 0.0, 0.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([3.0, 4.0, 5.0]), OnAFace::Held);
    assert_eq!(asked([-9.0, 12.0, 15.0]), OnAFace::Held);
    assert_eq!(asked([9.0, 12.0, 15.0]), OnAFace::Held);
    assert_eq!(asked([-12.0, 9.0, 15.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([0.0, 15.0, 15.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([-13.0, 0.0, 13.0]), OnAFace::NotHeld);
    assert_eq!(asked([0.0, 16.0, 16.0]), OnAFace::NotHeld);
    assert_eq!(asked([12.0, 16.0, 20.0]), OnAFace::NotHeld);
    assert_eq!(asked([0.0, 5.0, -5.0]), OnAFace::NotHeld);
}

/// **The apex of the cut cone is on its boundary**, where `classify_point`
/// answered `CannotFollowAFacesTrimming` naming the skin until Q1235. A point
/// up the axis inside it and one below the apex are answered too.
#[test]
fn the_apex_of_a_cone_cut_by_two_walls_along_its_axis_is_on_its_boundary() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cut = a_cone_cut_by_two_walls_along_its_axis(&mut model);
    let solid = model.solid(cut).expect("this model made the key");
    assert_the_apex_is_on_the_boundary(solid);
}

/// **The skin of a cone cut by a wall and a slope meeting under its apex holds
/// its apex**, on its boundary. The corners are where each plane meets the base
/// circle, `(−12, ±16, 20)` exactly and `(16, ±12, 20)` to a rounding of the
/// slope's stored normal. The apex was refused
/// `AFaceTrimmedByACurveThisCannotFollow` until each hyperbola was read as
/// never reaching the other plane because both its meets with the line the two
/// planes share are on the other nappe (Q1391). The other points are worked out
/// by hand against `x ≥ −12` and `z ≥ 2x − 12`, each a whole unit or more off
/// either plane or on one exactly.
#[test]
fn the_skin_of_a_cone_cut_by_a_wall_and_a_slope_meeting_under_its_apex_holds_its_apex() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cut = a_cone_cut_by_a_wall_and_a_slope_meeting_under_its_apex(&mut model);
    let solid = model.solid(cut).expect("this model made the key");
    let corners = corners_of_the_skin(solid);
    assert_eq!(corners.len(), 4, "{corners:?}");
    for corner in &corners {
        let near = |want: [f64; 3]| (0..3).all(|axis| (corner[axis] - want[axis]).abs() < 1e-12);
        assert!(
            [[-12.0, 16.0], [-12.0, -16.0], [16.0, 12.0], [16.0, -12.0]]
                .iter()
                .any(|&[x, y]| near([x, y, 20.0])),
            "{corners:?}"
        );
    }
    let asked = |at| what_the_skin_holds(solid, at);
    assert_eq!(asked([0.0, 0.0, 0.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([3.0, 4.0, 5.0]), OnAFace::Held);
    assert_eq!(asked([11.0, 0.0, 11.0]), OnAFace::Held);
    assert_eq!(asked([-12.0, 9.0, 15.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([0.0, 20.0, 20.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([13.0, 0.0, 13.0]), OnAFace::NotHeld);
    assert_eq!(asked([-13.0, 0.0, 13.0]), OnAFace::NotHeld);
    assert_eq!(asked([0.0, 5.0, -5.0]), OnAFace::NotHeld);
}

/// **The apex of the cone cut by a wall and a slope meeting under it is on its
/// boundary**, where `classify_point` answered `CannotFollowAFacesTrimming`
/// naming the skin until Q1391.
#[test]
fn the_apex_of_a_cone_cut_by_a_wall_and_a_slope_meeting_under_it_is_on_its_boundary() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cut = a_cone_cut_by_a_wall_and_a_slope_meeting_under_its_apex(&mut model);
    let solid = model.solid(cut).expect("this model made the key");
    assert_the_apex_is_on_the_boundary(solid);
}

/// **The skin of a cone cut by a wall and a plane meeting it along a ruling
/// holds its apex**, on its boundary. The plane's normal `(√2, 1, 1)` is
/// stored as `(0.7071…, 0.5, 0.5)`, so the part of degree two along the line
/// it meets `x = −12` in is `0.5² − 0.5²`, zero exactly. Its corners on the
/// base circle solve `x/√2 + y/2 = 1.5` and `x² + y² = 400`:
/// `x = (6√2 ± √4764)/6`, about `12.918` and `−10.090`. The apex was refused
/// `AFaceTrimmedByACurveThisCannotFollow` until a line along a ruling was read
/// as meeting the curve once (Q1390). The other points are worked out by hand
/// against `x ≥ −12` and `0.7071…x + y/2 + z/2 ≤ 11.5`, each half a unit or
/// more off either plane or on the wall exactly.
#[test]
fn the_skin_of_a_cone_cut_by_a_wall_and_a_plane_along_a_ruling_holds_its_apex() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cut = a_cone_cut_by_a_wall_and_a_plane_along_a_ruling(&mut model);
    let solid = model.solid(cut).expect("this model made the key");
    let corners = corners_of_the_skin(solid);
    assert_eq!(corners.len(), 4, "{corners:?}");
    for corner in &corners {
        let near = |want: [f64; 3]| (0..3).all(|axis| (corner[axis] - want[axis]).abs() < 1e-3);
        assert!(
            [
                [-12.0, 16.0],
                [-12.0, -16.0],
                [12.918, -15.269],
                [-10.089, 17.269]
            ]
            .iter()
            .any(|&[x, y]| near([x, y, 20.0])),
            "{corners:?}"
        );
    }
    let asked = |at| what_the_skin_holds(solid, at);
    assert_eq!(asked([0.0, 0.0, 0.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([3.0, 4.0, 5.0]), OnAFace::Held);
    assert_eq!(asked([0.0, 11.0, 11.0]), OnAFace::Held);
    assert_eq!(asked([-12.0, 9.0, 15.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([0.0, -20.0, 20.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([0.0, 12.0, 12.0]), OnAFace::NotHeld);
    assert_eq!(asked([-13.0, 0.0, 13.0]), OnAFace::NotHeld);
    assert_eq!(asked([0.0, 5.0, -5.0]), OnAFace::NotHeld);
}

/// **The apex of the cone cut by a wall and a plane meeting it along a ruling
/// is on its boundary**, where `classify_point` answered
/// `CannotFollowAFacesTrimming` naming the skin until Q1390.
#[test]
fn the_apex_of_a_cone_cut_by_a_wall_and_a_plane_along_a_ruling_is_on_its_boundary() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cut = a_cone_cut_by_a_wall_and_a_plane_along_a_ruling(&mut model);
    let solid = model.solid(cut).expect("this model made the key");
    assert_the_apex_is_on_the_boundary(solid);
}

/// **The same shape with a plane whose normal is not stored in short numbers
/// still refuses its apex.** The plane `2x + y + z = 16` meets `x = −12` along
/// `(0, 1, −1)`, a ruling's direction, as the plane above does, but its normal
/// is stored as `(0.8165…, 0.4082…, 0.4082…)`, each square in the part of
/// degree two along that line rounds, and their difference is not settled.
/// Every other point of the skin is answered seen down the axis. This pins the
/// refusal a sign settled exactly would turn into `OnItsBoundary`.
#[test]
fn a_cone_cut_by_a_wall_and_a_plane_along_a_ruling_stored_inexactly_still_refuses_its_apex() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_cone_kept_to_x_at_least_minus_twelve(&mut model);
    let leaning =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 16.0), Vector::new(2.0, 1.0, 1.0))
            .expect("a direction with length");
    let wedge = a_tetrahedron(
        &mut model,
        [
            Point::new(-100.0, -60.0, 276.0),
            Point::new(100.0, -60.0, -124.0),
            Point::new(0.0, 100.0, -84.0),
            Point::new(-245.0, -122.0, -112.0),
        ],
        leaning,
    );
    let cut = the_one_solid_both_keep(&mut model, cone, wedge);
    let solid = model.solid(cut).expect("this model made the key");
    assert_eq!(corners_of_the_skin(solid).len(), 4);
    let refused = solid.face_holds_the_point(
        the_skin(solid).key(),
        PredicatePoint::from([0.0, 0.0, 0.0]),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(
            refused,
            Err(Error::AFaceTrimmedByACurveThisCannotFollow { .. })
        ),
        "{refused:?}"
    );
    assert_eq!(what_the_skin_holds(solid, [3.0, 4.0, 5.0]), OnAFace::Held);
}

/// The apex is on the boundary, a point up the axis inside and one below the
/// apex outside: every solid here keeps the axis from the apex to `z = 9`.
fn assert_the_apex_is_on_the_boundary(solid: SolidView<'_>) {
    let classified =
        |at: Point| solid.classify_point(at, &Policy::certified(), &mut ProvenanceLog::new());
    assert_eq!(classified(Point::origin()), Containment::OnBoundary);
    assert_eq!(classified(Point::new(0.0, 0.0, 9.0)), Containment::Inside);
    assert_eq!(classified(Point::new(0.0, 0.0, -1.0)), Containment::Outside);
}

/// What the solid's one face on the cone answers for `at`, which must be
/// answered.
fn what_the_skin_holds(solid: SolidView<'_>, at: [f64; 3]) -> OnAFace {
    solid
        .face_holds_the_point(
            the_skin(solid).key(),
            PredicatePoint::from(at),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{at:?} is answered: {why:?}"))
}

fn corners_of_the_skin(solid: SolidView<'_>) -> Vec<[f64; 3]> {
    the_skin(solid)
        .wires()
        .flat_map(|wire| wire.points().collect::<Vec<_>>())
        .map(|corner| corner.coordinates())
        .collect()
}

fn the_skin(solid: SolidView<'_>) -> rcad_topo::FaceView<'_> {
    solid
        .faces()
        .find(|face| matches!(face.surface(), Surface::Cone(_)))
        .expect("the cut cone has a skin")
}

/// The pointed cone of slope 1 with its base at `z = 20`, apex at the origin,
/// less all but the block `[−12, 24] × [−24, 15] × [−1, 18]`, whose walls
/// `x = −12` and `y = 15` and top `z = 18` cut it and whose other faces are
/// clear of it.
fn a_cone_cut_by_two_walls_along_its_axis(model: &mut Model) -> SolidKey {
    let cone = cones::a_pointed_cone(model, 1.0, 20.0);
    let profile = ProfileBuilder::new(vec![
        [-12.0, -24.0],
        [24.0, -24.0],
        [24.0, 15.0],
        [-12.0, 15.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a simple outline");
    let block = model
        .add_extrusion(
            &profile,
            19.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -1.0)).expect("a whole-number shift"),
        )
        .expect("a block");
    the_one_solid_both_keep(model, cone, block)
}

/// The same cone less all but the prism `x ≥ −12`, `z ≥ 2x − 12`, `z ≤ 24`,
/// `|y| ≤ 24`: the triangle `(−12, −36)`, `(18, 24)`, `(−12, 24)` in `x` and
/// `z`, drawn in `x` and `y` and turned a quarter about `x`, which takes `y`
/// to `z` exactly. Its top and ends are clear of the cone.
fn a_cone_cut_by_a_wall_and_a_slope_meeting_under_its_apex(model: &mut Model) -> SolidKey {
    let cone = cones::a_pointed_cone(model, 1.0, 20.0);
    let profile = ProfileBuilder::new(vec![[-12.0, -36.0], [18.0, 24.0], [-12.0, 24.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a simple outline");
    let laid_along_y = Isometry::quarter_turns(&Axis::x(), 1)
        .expect("x is an axis")
        .compose(
            &Isometry::translation(Vector::new(0.0, 0.0, -24.0)).expect("a whole-number shift"),
        );
    let prism = model
        .add_extrusion(&profile, 48.0, &laid_along_y)
        .expect("a prism");
    the_one_solid_both_keep(model, cone, prism)
}

/// The same cone less all but `x ≥ −12` and the side of the origin of the
/// plane through `(0, 0, 23)` with normal `(√2, 1, 1)`. That plane is stored as
/// `0.7071…x + y/2 + z/2 = 11.5`, and it is a face of a tetrahedron whose three
/// corners on it satisfy that exactly (`r` the stored `0.7071…`, `256r` exact),
/// whose fourth corner is far on the origin's side, and whose other three faces
/// are clear of the cone.
fn a_cone_cut_by_a_wall_and_a_plane_along_a_ruling(model: &mut Model) -> SolidKey {
    let cone = a_cone_kept_to_x_at_least_minus_twelve(model);
    let leaning = Plane::from_point_and_normal(
        Point::new(0.0, 0.0, 23.0),
        Vector::new(core::f64::consts::SQRT_2, 1.0, 1.0),
    )
    .expect("a direction with length");
    let [r, half, also_half] = leaning.normal().components();
    assert_eq!([half, also_half, leaning.offset()], [0.5, 0.5, 11.5]);
    let wedge = a_tetrahedron(
        model,
        [
            Point::new(-128.0, 40.0, -17.0 + 256.0 * r),
            Point::new(128.0, 40.0, -17.0 - 256.0 * r),
            Point::new(0.0, -64.0, 87.0),
            Point::new(0.0, 0.0, -256.0),
        ],
        leaning,
    );
    the_one_solid_both_keep(model, cone, wedge)
}

/// The cone less all but the block `[−12, 40] × [−40, 40] × [−8, 30]`, whose
/// wall `x = −12` alone reaches it.
fn a_cone_kept_to_x_at_least_minus_twelve(model: &mut Model) -> SolidKey {
    let cone = cones::a_pointed_cone(model, 1.0, 20.0);
    let profile = ProfileBuilder::new(vec![
        [-12.0, -40.0],
        [40.0, -40.0],
        [40.0, 40.0],
        [-12.0, 40.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a simple outline");
    let block = model
        .add_extrusion(
            &profile,
            38.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -8.0)).expect("a whole-number shift"),
        )
        .expect("a block");
    the_one_solid_both_keep(model, cone, block)
}

/// The tetrahedron with these corners, its face through the first three on
/// `first`, which faces away from the fourth, and each other face on the plane
/// through its three corners facing away from the corner it leaves out.
fn a_tetrahedron(model: &mut Model, corners: [Point; 4], first: Plane) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let vertices = corners.map(|corner| builder.add_vertex(corner).expect("a finite point"));
    let mut edges = HashMap::new();
    for from in 0..4 {
        for to in from + 1..4 {
            let edge = builder
                .add_edge(vertices[from], vertices[to], EdgeRun::Straight)
                .expect("two corners of this build");
            edges.insert((from, to), edge);
        }
    }
    let at = |which: usize| Vector::from(corners[which].coordinates());
    let mut faces = Vec::with_capacity(4);
    for left_out in [3, 0, 1, 2] {
        let mut three: Vec<usize> = (0..4).filter(|which| *which != left_out).collect();
        let mut facing = (at(three[1]) - at(three[0])).cross(at(three[2]) - at(three[1]));
        if facing.dot(at(left_out) - at(three[0])) > 0.0 {
            three.swap(1, 2);
            facing = -facing;
        }
        let steps: Vec<DraftStep> = (0..3)
            .map(|step| {
                let (from, to) = (three[step], three[(step + 1) % 3]);
                let (edge, turn) = if from < to {
                    (edges[&(from, to)], Orientation::Forward)
                } else {
                    (edges[&(to, from)], Orientation::Reversed)
                };
                DraftStep::Along(
                    builder
                        .add_coedge(edge, turn)
                        .expect("an edge of this build"),
                )
            })
            .collect();
        let wire = builder
            .add_wire(&steps)
            .expect("three sides round a triangle");
        let plane = if left_out == 3 {
            first
        } else {
            Plane::from_point_and_normal(corners[three[0]], facing)
                .expect("a direction with length")
        };
        faces.push((
            builder
                .add_face(Surface::Plane(plane), &[wire])
                .expect("a triangle on its plane"),
            Orientation::Forward,
        ));
    }
    builder.add_shell(&faces).expect("four triangles close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a tetrahedron")
}

fn the_one_solid_both_keep(model: &mut Model, one: SolidKey, other: SolidKey) -> SolidKey {
    let answer = model
        .fuse(
            &[one],
            &[other],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the intersection refused: {why}"));
    let [solid] = answer.solids() else {
        panic!("the intersection is one solid, not {:?}", answer.solids());
    };
    *solid
}
