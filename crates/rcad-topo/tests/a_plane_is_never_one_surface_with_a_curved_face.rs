//! A cube and a quarter bend twenty units apart, which a fuse could not
//! answer because it asked where a plane crosses a ring (`docs/decisions.md`
//! §322.7, queue row Q326).
//!
//! A fuse places each curved piece against the other solid, and first asks
//! each of that solid's faces whether the piece lies on it. For a flat face
//! that question went to `intersect`, which works out the **whole crossing**
//! of the two surfaces. A plane crosses a ring in circles only when it is
//! square to the ring's axis or holds it, and `intersect` refuses every other
//! plane as a quartic, however far away it is. Three of the cube's six planes
//! are neither, so every fuse of the two refused, although they share nothing.
//!
//! A plane and a curved surface are never one surface, so the fuse no longer
//! asks. `do_they_face_the_same_way` in `fuse/select.rs` says why that holds,
//! and [`no_curved_surface_can_be_built_flat`] is the constructors' half of
//! it.
//!
//! Of the five fuses of this pair, the three whose answer leaves the bend out
//! answer now and are pinned here. The union and the bend less the cube keep
//! the bend and are refused writing it back, `TheFuseWroteABrokenAnswer` with
//! `CoedgesRunTheSameWay`: the stop a bend fused with a copy of itself reaches
//! (`how_often_a_fuse_with_a_torus_refuses.rs`), and not this file's.

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, Isometry, Point, Scale, Sphere, Surface, Torus, Vector,
    intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

mod rings;

use rings::a_quarter_bend;

/// A cube of side two, twenty units out along `x`, and a quarter bend of ring
/// four and tube one about `z` through the origin. The bend reaches no
/// further than five from the origin.
fn a_bend_and_a_cube_far_away() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bend = a_quarter_bend(&mut model, 4.0, 1.0);
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(20.0, 0.0, 0.0)).expect("a shift"),
        )
        .expect("this model made the key");
    (model, bend, cube)
}

/// `x = 20`, `x = 22` and `y = 2` run along the bend's axis and off it, and
/// `intersect` refuses each against the bend's ring. `y = 0` holds the axis,
/// and `z = 0` and `z = 2` are square to it, so those three are answered.
/// This is the question the fuse asked before it placed the bend's wall.
fn three_of_the_cubes_planes_cross_the_bends_ring_in_a_quartic(
    model: &Model,
    bend: SolidKey,
    cube: SolidKey,
) {
    let ring = model
        .solid(bend)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .find(|surface| matches!(surface, Surface::Torus(_)))
        .expect("a bend's wall is on a ring");
    let mut refused = Vec::new();
    for face in model.solid(cube).expect("this model made the key").faces() {
        let flat = face.surface();
        let Surface::Plane(plane) = flat else {
            panic!("a cube is all planes: {flat:?}");
        };
        match intersect(&flat, &ring, &Policy::certified(), &mut ProvenanceLog::new()) {
            Ok(_) => (),
            Err(rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic) => {
                refused.push((plane.normal().components(), plane.offset()));
            }
            Err(other) => panic!("{plane:?} against the ring: {other:?}"),
        }
    }
    refused.sort_by(|one, other| one.partial_cmp(other).expect("no NaN in a cube"));
    assert_eq!(
        refused,
        [
            ([-1.0, 0.0, 0.0], -20.0),
            ([0.0, 1.0, 0.0], 2.0),
            ([1.0, 0.0, 0.0], 22.0),
        ],
        "the planes of the cube intersect refuses against the bend's ring"
    );
}

#[test]
fn a_cube_less_a_bend_far_away_is_the_cube() {
    let (mut model, bend, cube) = a_bend_and_a_cube_far_away();
    three_of_the_cubes_planes_cross_the_bends_ring_in_a_quartic(&model, bend, cube);

    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(&[cube], &[bend], Rule::Difference, &policy, &mut log)
        .expect("a cube less a bend it does not touch");
    assert!(log.is_empty(), "a certified fuse logs nothing: {log:?}");
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the answer is sound");
    let [left] = fused.solids() else {
        panic!("one solid: {:?}", fused.solids());
    };
    let measured = model
        .solid(*left)
        .expect("the fuse made the key")
        .measure(
            Deflection::for_scale(model.scale()),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a cube measures");
    assert_eq!(measured.volume(), 8.0, "the whole cube");
}

#[test]
fn a_bend_and_a_cube_far_away_share_nothing_either_way_round() {
    let policy = Policy::certified();
    for bend_first in [false, true] {
        let (mut model, bend, cube) = a_bend_and_a_cube_far_away();
        three_of_the_cubes_planes_cross_the_bends_ring_in_a_quartic(&model, bend, cube);
        let (argument, tool) = if bend_first {
            (bend, cube)
        } else {
            (cube, bend)
        };
        let mut log = ProvenanceLog::new();
        let fused = model
            .fuse(&[argument], &[tool], Rule::Intersection, &policy, &mut log)
            .expect("a cube and a bend that do not touch, shared");
        assert!(fused.solids().is_empty(), "bend first {bend_first}: {fused:?}");
        assert!(log.is_empty(), "a certified fuse logs nothing: {log:?}");
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .expect("the model is sound");
    }
}

/// Every surface that could be a plane and still be called a ball, a pipe, a
/// cone or a ring is refused at the door, from a call or from bytes.
///
/// A ball or a pipe of infinite radius, and a ring of infinite ring radius,
/// are the limits that flatten out. A cone of a right-angled half-angle, which
/// is an infinite slope, **is** the plane square to its axis through its apex.
/// The argument in `do_they_face_the_same_way` rests on each of these being
/// finite, and this is where it is shown that they are.
#[test]
fn no_curved_surface_can_be_built_flat() {
    let infinite = f64::INFINITY;
    assert!(
        matches!(
            Cone::from_half_angle(Axis::z(), core::f64::consts::FRAC_PI_2),
            Err(rcad_geom::Error::HalfAngleIsNotACone { .. })
        ),
        "a cone opened out to a right angle"
    );
    assert!(
        matches!(
            Cone::new(Axis::z(), infinite),
            Err(rcad_geom::Error::SlopeNotPositive { .. })
        ),
        "a cone of infinite slope"
    );
    assert!(
        matches!(
            Sphere::new(Point::origin(), infinite),
            Err(rcad_geom::Error::RadiusNotPositive { .. })
        ),
        "a ball of infinite radius"
    );
    assert!(
        matches!(
            Cylinder::new(Axis::z(), infinite),
            Err(rcad_geom::Error::RadiusNotPositive { .. })
        ),
        "a pipe of infinite radius"
    );
    assert!(
        matches!(
            Torus::new(Axis::z(), infinite, 1.0),
            Err(rcad_geom::Error::RadiusNotPositive { .. })
        ),
        "a ring of infinite ring radius"
    );

    let written = |what: &str, bytes: postcard::Result<Vec<u8>>| {
        bytes.unwrap_or_else(|why| panic!("{what} writes out: {why}"))
    };
    let cone = written("a cone", postcard::to_allocvec(&(Axis::z(), infinite)));
    assert!(postcard::from_bytes::<Cone>(&cone).is_err(), "a cone of infinite slope, read");
    let ball = written("a ball", postcard::to_allocvec(&(Point::origin(), infinite)));
    assert!(postcard::from_bytes::<Sphere>(&ball).is_err(), "a ball of infinite radius, read");
    let pipe = written("a pipe", postcard::to_allocvec(&(Axis::z(), infinite)));
    assert!(postcard::from_bytes::<Cylinder>(&pipe).is_err(), "a pipe of infinite radius, read");
    let ring = written("a ring", postcard::to_allocvec(&(Axis::z(), infinite, 1.0)));
    assert!(postcard::from_bytes::<Torus>(&ring).is_err(), "a ring of infinite ring radius, read");
}

/// **Two curved surfaces of different kinds still go to `intersect`**, and a
/// ring against a ball is refused there whatever the distance between them.
/// So a ball twenty units from a bend refuses every fuse this one makes. The
/// answers exist — a ball less a bend it does not touch is the ball — so this
/// is a gap. Q326 did not change that arm; skipping `intersect` there for two
/// kinds made both fuses below answer, when it was tried and taken out again.
#[test]
fn a_ball_far_from_a_bend_is_still_refused_where_a_ring_meets_a_ball() {
    let policy = Policy::certified();
    for rule in [Rule::Difference, Rule::Intersection] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let bend = a_quarter_bend(&mut model, 4.0, 1.0);
        let ball = model
            .add_ball(
                1.0,
                &Isometry::translation(Vector::new(20.0, 0.0, 0.0)).expect("a shift"),
            )
            .expect("a ball of radius one");
        let refused = model.fuse(&[ball], &[bend], rule, &policy, &mut ProvenanceLog::new());
        assert!(
            matches!(
                refused,
                Err(Error::Geometry(
                    rcad_geom::Error::ACrossingBetweenATorusAndAnotherCurvedSurface
                ))
            ),
            "{rule:?}: {refused:?}"
        );
    }
}
