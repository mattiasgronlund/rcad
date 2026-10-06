//! A point in one face's sliver of a turned cube, within a rounding or two of
//! the edge that face shares with its neighbour (`docs/decisions.md` §346.6,
//! Q997).
//!
//! A turn leaves a flat face's stored corners a rounding off its stored plane,
//! and the plane is the face (§17.1). Between the plane and the triangles the
//! ray fans from the corners is a sliver. The ray asks the plane at every
//! triangle its line passes through, but the line from a point in one face's
//! sliver by its edge can pass through the neighbouring face's triangle
//! instead, or through none at all. The plane was then never asked, and the
//! corners' answer came back where the stored planes say the other. Q997's
//! probe found that at 180458 of 4320000 points within two spacings of the
//! edges of 48 turned cubes.
//!
//! Each point here is held to every stored plane by the kernel's exact signs,
//! and both were worked out again in exact rationals (`q997_exact.py`, in the
//! lane's scratchpad). Both were answered wrong, by the corners. Both are
//! refused now: the plane of the face whose sliver each is in cannot place it
//! alone so near the neighbour, whose own corners and plane part there too.

use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{
    Decision, Point as PredicatePoint, Policy, ProvenanceLog, orient3d_of_points,
};
use rcad_topo::{Containment, FaceView, Model, SolidKey, SolidView};

/// A point `2.5e-16` outside one face's stored plane, which that face's
/// triangle puts inside, `1.7e-16` inside its neighbour's. The line from it to
/// the ray's first target passes through the neighbour's triangle and through
/// none of its own face's, and the corners counted it inside.
#[test]
fn a_point_past_a_faces_plane_whose_line_crosses_the_neighbour_is_not_put_inside() {
    let (model, block) = a_turned_cube([-2.0, 1.0, 5.0], 3.0);
    let view = model.solid(block).expect("this model made the key");
    let point = [
        -0.340_011_261_769_737_84,
        -1.280_412_468_998_033_6,
        -1.086_479_387_957_468_6,
    ];
    let sliver = the_one_face_whose_corners_and_plane_part(view, point);
    let faces: Vec<FaceView<'_>> = view.faces().collect();
    assert_eq!(side_of_its_plane(faces[sliver], point), Side::Outer);
    let crossed = faces_its_line_passes_through(view, point);
    assert!(!crossed.contains(&sliver), "{crossed:?}");
    assert!(
        crossed
            .iter()
            .any(|&other| share_an_edge(faces[sliver], faces[other])),
        "the line passes through the neighbour's triangle: {crossed:?}"
    );
    assert_eq!(
        what_the_stored_planes_say(view, point),
        Containment::Outside
    );
    assert_refused(view, point);
}

/// A point `1.3e-17` inside one face's stored plane, which both its triangles
/// put outside, and `1.2e-17` inside its neighbour's. The line from it to the
/// ray's first target passes through no triangle at all, and the corners
/// counted it outside.
#[test]
fn a_point_short_of_a_faces_plane_whose_line_crosses_no_triangle_is_not_put_outside() {
    let (model, block) = a_turned_cube([-2.0, 1.0, 5.0], 5.0);
    let view = model.solid(block).expect("this model made the key");
    let point = [
        -0.738_667_994_669_924_3,
        0.866_516_401_762_128_7,
        -1.255_655_724_122_034_7,
    ];
    let sliver = the_one_face_whose_corners_and_plane_part(view, point);
    let faces: Vec<FaceView<'_>> = view.faces().collect();
    assert_eq!(side_of_its_plane(faces[sliver], point), Side::Inner);
    assert_eq!(
        faces_its_line_passes_through(view, point),
        Vec::<usize>::new()
    );
    assert_eq!(what_the_stored_planes_say(view, point), Containment::Inside);
    assert_refused(view, point);
}

/// Neither the corners' answer nor a guess, under either policy: the stored
/// planes' answer is not what the ray can show here.
fn assert_refused(view: SolidView<'_>, point: [f64; 3]) {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        assert_eq!(
            view.classify_point(
                Point::new(point[0], point[1], point[2]),
                &policy,
                &mut ProvenanceLog::new()
            ),
            Containment::Undecidable { enclosure: 0.0 },
            "{policy:?}"
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Side {
    Inner,
    Outer,
    On,
}

/// The cube is convex, so it holds a point exactly when every stored plane
/// puts the point on the cube's side.
fn what_the_stored_planes_say(view: SolidView<'_>, point: [f64; 3]) -> Containment {
    let sides: Vec<Side> = view
        .faces()
        .map(|face| side_of_its_plane(face, point))
        .collect();
    if sides.contains(&Side::Outer) {
        Containment::Outside
    } else if sides.contains(&Side::On) {
        Containment::OnBoundary
    } else {
        Containment::Inside
    }
}

fn side_of_its_plane(face: FaceView<'_>, point: [f64; 3]) -> Side {
    let outer = if face.orientation().is_reversed() {
        Decision::Neg
    } else {
        Decision::Pos
    };
    read(
        face.plane().side_of_point(PredicatePoint::from(point)),
        outer,
    )
}

/// The side an exact sign puts a point on, where `outer` is the sign of the
/// outer side.
fn read(side: Decision, outer: Decision) -> Side {
    assert!(
        matches!(side, Decision::Pos | Decision::Neg | Decision::Zero),
        "an exact point is always placed: {side:?}"
    );
    if side == Decision::Zero {
        Side::On
    } else if side == outer {
        Side::Outer
    } else {
        Side::Inner
    }
}

/// The index of the one face some triangle of which puts `point` on the other
/// side from that face's stored plane. A triangle's outer side is the side it
/// puts a point one unit out along the face's outward normal from its middle.
fn the_one_face_whose_corners_and_plane_part(view: SolidView<'_>, point: [f64; 3]) -> usize {
    let parting: Vec<usize> = view
        .faces()
        .enumerate()
        .filter(|&(_, face)| {
            let by_the_plane = side_of_its_plane(face, point);
            let outward = face.outward_normal().components();
            let mut parts = false;
            face.for_each_triangle(|corners| {
                let at = corners.map(|corner| corner.coordinates());
                let out: [f64; 3] = core::array::from_fn(|axis| {
                    (at[0][axis] + at[1][axis] + at[2][axis]) / 3.0 + outward[axis]
                });
                let [a, b, c] = at.map(PredicatePoint::from);
                let outer = orient3d_of_points(a, b, c, PredicatePoint::from(out));
                let by_the_triangle = read(
                    orient3d_of_points(a, b, c, PredicatePoint::from(point)),
                    outer,
                );
                parts |= by_the_triangle != Side::On
                    && by_the_plane != Side::On
                    && by_the_triangle != by_the_plane;
            });
            parts
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(parting.len(), 1, "faces parting: {parting:?}");
    parting[0]
}

/// The faces with a triangle the line from `point` to the ray's first target
/// passes through, by the three signs across its sides.
fn faces_its_line_passes_through(view: SolidView<'_>, point: [f64; 3]) -> Vec<usize> {
    let target = PredicatePoint::from(the_first_ray_target(view));
    let point = PredicatePoint::from(point);
    view.faces()
        .enumerate()
        .filter(|&(_, face)| {
            let mut through = false;
            face.for_each_triangle(|corners| {
                let [a, b, c] = corners.map(|corner| PredicatePoint::from(corner.coordinates()));
                let signs = [
                    orient3d_of_points(a, b, target, point),
                    orient3d_of_points(b, c, target, point),
                    orient3d_of_points(c, a, target, point),
                ];
                through |= signs.iter().all(|&sign| sign == Decision::Pos)
                    || signs.iter().all(|&sign| sign == Decision::Neg);
            });
            through
        })
        .map(|(index, _)| index)
        .collect()
}

/// The first of `classify.rs`'s three ray targets, worked out the way its
/// `ray_targets` does from the solid's box.
fn the_first_ray_target(view: SolidView<'_>) -> [f64; 3] {
    let (low, high) = view.box_around_it().expect("a cube has a box");
    let (low, high) = (low.coordinates(), high.coordinates());
    let span = (0..3)
        .map(|axis| high[axis] - low[axis])
        .fold(0.0, f64::max);
    [
        high[0] + span,
        high[1] + span * 0.618_033_988_749_895,
        high[2] + span * 0.414_213_562_373_095_1,
    ]
}

fn share_an_edge(one: FaceView<'_>, other: FaceView<'_>) -> bool {
    let corners: Vec<Point> = one.outer_wire().points().collect();
    other
        .outer_wire()
        .points()
        .filter(|corner| corners.contains(corner))
        .count()
        == 2
}

/// A cube of side 2 about the origin, turned `angle` about `axis` through it.
fn a_turned_cube(axis: [f64; 3], angle: f64) -> (Model, SolidKey) {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let cube = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side 2");
    let turn = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(axis[0], axis[1], axis[2])).expect("an axis"),
        angle,
    )
    .expect("a finite turn");
    let centred = Isometry::translation(Vector::new(-1.0, -1.0, -1.0)).expect("a finite shift");
    model
        .place_solid(cube, &turn.compose(&centred))
        .expect("this model made the key");
    (model, cube)
}
