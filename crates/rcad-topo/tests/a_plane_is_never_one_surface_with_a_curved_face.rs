//! Two surfaces of different kinds are never one surface, so a fuse no longer
//! asks `intersect` whether they are (`docs/decisions.md` §322.7, queue rows
//! Q326 and Q463). The file holds every pair of the five kinds.
//!
//! A fuse places each curved piece against the other solid, and first asks
//! each of that solid's faces whether the piece lies on it. That question went
//! to `intersect`, which works out the **whole crossing** of the two surfaces,
//! and `intersect` refuses some pairs however far apart they are. A refusal
//! there refused the fuse, although the two solids shared nothing.
//!
//! **A plane and a curved face**, Q326: a cube and a quarter bend twenty units
//! apart. A plane crosses a ring in circles only when it is square to the
//! ring's axis or holds it, and `intersect` refuses every other plane as a
//! quartic. Three of the cube's six planes are neither, so every fuse of the
//! two refused. `do_they_face_the_same_way` in `fuse/select.rs` says why no
//! curved surface is a plane, and [`no_curved_surface_can_be_built_flat`] is
//! the constructors' half of it.
//!
//! Of the five fuses of this pair, the three whose answer leaves the bend out
//! are pinned here. Since Q667 the other two also answer, with the certified
//! log empty and `Model::validate` passing: the union is two solids, the cube
//! at 8 exactly and the bend at 19.7061 give or take 0.3921 at deflection
//! 0.01, and the bend less the cube is one solid, the bend. No test here pins
//! that pair yet.
//!
//! **Two curved faces of different kinds**, Q463: `intersect` refuses a ring
//! against a ball, a pipe or a cone, and until Q2788 it refused a cone against
//! a ball off its axis too. A ball twenty units from a bend refused all six
//! fuses of the pair for it, and so did a ball twenty units from a frustum;
//! each now answers with the two solids as they were. For each of the six pairs of curved kinds a test below builds,
//! from any two surfaces of those kinds, one plane that `intersect` cuts the
//! two in differently, and runs it over drawn pairs and over the pairs most
//! alike. One plane cuts one surface in one way.

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, Isometry, Plane, Point, Scale, Sphere, Surface, SurfaceMeet,
    Torus, Vector, intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

mod cones;
mod rings;

use cones::a_frustum;
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
        match intersect(
            &flat,
            &ring,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) {
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
        assert!(
            fused.solids().is_empty(),
            "bend first {bend_first}: {fused:?}"
        );
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
    assert!(
        postcard::from_bytes::<Cone>(&cone).is_err(),
        "a cone of infinite slope, read"
    );
    let ball = written(
        "a ball",
        postcard::to_allocvec(&(Point::origin(), infinite)),
    );
    assert!(
        postcard::from_bytes::<Sphere>(&ball).is_err(),
        "a ball of infinite radius, read"
    );
    let pipe = written("a pipe", postcard::to_allocvec(&(Axis::z(), infinite)));
    assert!(
        postcard::from_bytes::<Cylinder>(&pipe).is_err(),
        "a pipe of infinite radius, read"
    );
    let ring = written("a ring", postcard::to_allocvec(&(Axis::z(), infinite, 1.0)));
    assert!(
        postcard::from_bytes::<Torus>(&ring).is_err(),
        "a ring of infinite ring radius, read"
    );
}

/// **A ball twenty units from a bend, fused every way, is the two as they
/// were.** `intersect` refuses a ring against a ball whatever the distance
/// between them, and the fuse used to ask it whether the ball's face was the
/// bend's wall, so all six fuses refused,
/// `ACrossingBetweenATorusAndAnotherCurvedSurface`.
#[test]
fn a_ball_far_from_a_bend_fuses_to_the_two_as_they_were() {
    every_fuse_of_a_ball_far_from(
        |model| a_quarter_bend(model, 4.0, 1.0),
        &Err(rcad_geom::Error::ACrossingBetweenATorusAndAnotherCurvedSurface),
    );
}

/// **The same for a frustum**, which the row that found the bend's case named
/// and did not run. All six fuses used to refuse,
/// `ACrossingBetweenAConeAndAnotherCurvedSurface`, when `intersect` refused
/// every cone against a ball off its axis. Since Q2788 it names this one: the
/// ball meets neither nappe of the cone the frustum's wall is cut from.
#[test]
fn a_ball_far_from_a_frustum_fuses_to_the_two_as_they_were() {
    every_fuse_of_a_ball_far_from(
        |model| a_frustum(model, 0.5, 2.0, 4.0),
        &Ok(SurfaceMeet::Nothing),
    );
}

/// A ball of radius one twenty units out along `x`, and `partner` round the
/// origin, fused under every rule both ways round. Each answer is certified,
/// sound, and holds the solids it should keep, each measuring as that solid
/// did on its own.
///
/// `said_by_intersect` is what `intersect` says to the ball against the
/// partner's curved wall, whether it refuses or not: the fuse no longer asks.
fn every_fuse_of_a_ball_far_from(
    partner: fn(&mut Model) -> SolidKey,
    said_by_intersect: &Result<SurfaceMeet, rcad_geom::Error>,
) {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let other = partner(&mut model);
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(Vector::new(20.0, 0.0, 0.0)).expect("a shift"),
        )
        .expect("a ball of radius one");

    let curved_wall_of = |solid: SolidKey| {
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .map(|face| face.surface())
            .find(|surface| surface.as_plane().is_none())
            .expect("a curved face")
    };
    let asked = intersect(
        &curved_wall_of(ball),
        &curved_wall_of(other),
        &policy,
        &mut ProvenanceLog::new(),
    );
    assert_eq!(&asked, said_by_intersect);

    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for (argument, tool) in [(ball, other), (other, ball)] {
            let kept: &[SolidKey] = if rule == Rule::Union {
                &[argument, tool]
            } else if rule == Rule::Intersection {
                &[]
            } else {
                &[argument]
            };
            let mut answer = model.clone();
            let mut log = ProvenanceLog::new();
            let fused = answer
                .fuse(&[argument], &[tool], rule, &policy, &mut log)
                .unwrap_or_else(|why| panic!("{rule:?}, ball first {}: {why}", argument == ball));
            assert!(log.is_empty(), "a certified fuse logs nothing: {log:?}");
            answer
                .validate(&policy, &mut ProvenanceLog::new())
                .expect("the answer is sound");
            let mut got: Vec<_> = fused
                .solids()
                .iter()
                .map(|solid| what_it_measures(&answer, *solid))
                .collect();
            let mut want: Vec<_> = kept
                .iter()
                .map(|solid| what_it_measures(&model, *solid))
                .collect();
            for list in [&mut got, &mut want] {
                list.sort_by(|one, other| one.0.total_cmp(&other.0));
            }
            assert_eq!(got.len(), want.len(), "{rule:?}: {got:?} against {want:?}");
            for (answered, alone) in got.iter().zip(&want) {
                assert!(
                    answered.2 == alone.2 && (answered.0 - alone.0).abs() <= answered.1 + alone.1,
                    "{rule:?}: the answer's solid is {answered:?} and the one it keeps is {alone:?}, \
                     as volume, the error measure publishes with it, and faces"
                );
            }
        }
    }
}

/// A solid's volume, the error `measure` publishes with it, and its face count.
fn what_it_measures(model: &Model, solid: SolidKey) -> (f64, f64, usize) {
    let view = model.solid(solid).expect("this model made the key");
    let measured = view
        .measure(
            Deflection::of(0.01).expect("a hundredth of the ball"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a ball, a bend and a frustum measure");
    (
        measured.volume(),
        measured.volume_error(),
        view.faces().count(),
    )
}

/// **A ball and a pipe.** A plane square to a pipe's axis cuts it in one
/// circle wherever along the axis it stands, and a plane further from a ball's
/// centre than its radius misses the ball. The plane square to the pipe's
/// axis one more than the radius from the ball's centre is both.
#[test]
fn a_ball_and_a_pipe_are_never_one_surface() {
    let mut draw = Draw(0xBA11);
    let snug = (
        Sphere::new(Point::origin(), 2.0).expect("a radius"),
        Cylinder::new(Axis::z(), 2.0).expect("a radius"),
    );
    let mut pairs = vec![snug];
    pairs.extend((0..DRAWS).map(|_| (draw.ball(), draw.pipe())));
    for (ball, pipe) in pairs {
        let along = pipe.axis().direction();
        let flat = a_plane(ball.centre() + along * (ball.radius() + 1.0), along);
        assert_eq!(
            [
                how_a_plane_cuts(flat, Surface::Cylinder(pipe)),
                how_a_plane_cuts(flat, Surface::Sphere(ball)),
            ],
            [Cut::Curves(1), Cut::Nothing],
            "{ball:?} and {pipe:?}"
        );
        intersect_never_calls_them_one(Surface::Sphere(ball), Surface::Cylinder(pipe));
    }
}

/// **A ball and a cone.** A plane square to a cone's axis cuts it in one
/// circle anywhere but through its apex, and a plane further from a ball's
/// centre than its radius misses the ball. The plane square to the cone's axis
/// one more than the radius from the ball's centre, on the side of the centre
/// the apex is not, is both.
#[test]
fn a_ball_and_a_cone_are_never_one_surface() {
    let mut draw = Draw(0xC0DE);
    let resting_inside = (
        Sphere::new(Point::new(0.0, 0.0, 4.0), 2.0).expect("a radius"),
        Cone::new(Axis::z(), 1.0).expect("a slope"),
    );
    let round_the_apex = (
        Sphere::new(Point::origin(), 1.0).expect("a radius"),
        Cone::new(Axis::z(), 1.0).expect("a slope"),
    );
    let mut pairs = vec![resting_inside, round_the_apex];
    pairs.extend((0..DRAWS).map(|_| {
        let ball = draw.ball();
        let apex = draw.point();
        (ball, draw.cone(apex))
    }));
    for (ball, cone) in pairs {
        let along = cone.axis().direction();
        let apex_is_above = (cone.apex() - ball.centre()).dot(along) > 0.0;
        let away = if apex_is_above { -1.0 } else { 1.0 };
        let flat = a_plane(
            ball.centre() + along * (away * (ball.radius() + 1.0)),
            along,
        );
        assert_eq!(
            [
                how_a_plane_cuts(flat, Surface::Cone(cone)),
                how_a_plane_cuts(flat, Surface::Sphere(ball)),
            ],
            [Cut::Curves(1), Cut::Nothing],
            "{ball:?} and {cone:?}"
        );
        intersect_never_calls_them_one(Surface::Sphere(ball), Surface::Cone(cone));
    }
}

/// **A ball and a ring.** A plane holding a ring's axis cuts it in two
/// circles, one either side of the axis, and a plane through a ball's centre
/// cuts it in one. The plane holding the ring's axis and the ball's centre is
/// both. The ball most like the ring is the one in its bend: centred on the
/// circle the tube runs round, with the tube's radius.
#[test]
fn a_ball_and_a_ring_are_never_one_surface() {
    let mut draw = Draw(0x0B0E);
    let ring = Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring");
    let in_the_bend = (
        Sphere::new(Point::new(4.0, 0.0, 0.0), 1.0).expect("a radius"),
        ring,
    );
    let through_the_middle = (Sphere::new(Point::origin(), 4.0).expect("a radius"), ring);
    let mut pairs = vec![in_the_bend, through_the_middle];
    pairs.extend((0..DRAWS).map(|_| (draw.ball(), draw.ring())));
    for (ball, ring) in pairs {
        let [x, y, _] = ball.centre().coordinates();
        let across = if x == 0.0 && y == 0.0 {
            Vector::new(0.0, 1.0, 0.0)
        } else {
            Vector::new(-y, x, 0.0)
        };
        let flat = a_plane(ring.centre(), across);
        assert_eq!(
            [
                how_a_plane_cuts(flat, Surface::Torus(ring)),
                how_a_plane_cuts(flat, Surface::Sphere(ball)),
            ],
            [Cut::Curves(2), Cut::Curves(1)],
            "{ball:?} and {ring:?}"
        );
        intersect_never_calls_them_one(Surface::Sphere(ball), Surface::Torus(ring));
    }
}

/// **A pipe and a cone.** The plane square to a cone's axis through its apex
/// touches the cone there and nowhere else. No plane touches a pipe at one
/// point: through every point of a pipe runs a straight line of it, parallel
/// to the axis, and a plane either holds that line or crosses the pipe in a
/// closed curve round it. The pipe most like that is one lying on the plane.
#[test]
fn a_pipe_and_a_cone_are_never_one_surface() {
    let mut draw = Draw(0x0C0E);
    let cone = Cone::new(Axis::z(), 1.0).expect("a slope");
    let round_the_axis = (Cylinder::new(Axis::z(), 2.0).expect("a radius"), cone);
    let lying_on_the_plane = (
        Cylinder::new(
            Axis::new(Point::new(0.0, 0.0, 2.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis"),
            2.0,
        )
        .expect("a radius"),
        cone,
    );
    let mut pairs = vec![round_the_axis, lying_on_the_plane];
    pairs.extend((0..DRAWS).map(|_| (draw.pipe(), draw.cone(Point::origin()))));
    for (pipe, cone) in pairs {
        let flat = a_plane(cone.apex(), cone.axis().direction());
        let cone_cut = how_a_plane_cuts(flat, Surface::Cone(cone));
        let pipe_cut = how_a_plane_cuts(flat, Surface::Cylinder(pipe));
        assert_eq!(cone_cut, Cut::OnePoint, "{cone:?}");
        assert_ne!(pipe_cut, Cut::OnePoint, "{pipe:?}");
        intersect_never_calls_them_one(Surface::Cylinder(pipe), Surface::Cone(cone));
    }
}

/// **A pipe and a ring.** The plane square to a ring's axis through its
/// centre cuts it in two circles, the inner and the outer edge, and so does
/// any plane holding its axis. A plane not parallel to a pipe's axis cuts it
/// in one closed curve. The first plane is not parallel to the pipe's axis
/// unless that axis is square to the ring's, and then the plane holding the
/// ring's axis square to the pipe's is not either.
#[test]
fn a_pipe_and_a_ring_are_never_one_surface() {
    let mut draw = Draw(0x919E);
    let ring = Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring");
    let through_the_middle = (Cylinder::new(Axis::z(), 4.0).expect("a radius"), ring);
    let along_the_tube = (
        Cylinder::new(
            Axis::new(Point::new(4.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0)).expect("an axis"),
            1.0,
        )
        .expect("a radius"),
        ring,
    );
    let mut pairs = vec![through_the_middle, along_the_tube];
    pairs.extend((0..DRAWS).map(|_| (draw.pipe(), draw.ring())));
    for (pipe, ring) in pairs {
        let along = pipe.axis().direction();
        let flat = if along.z() == 0.0 {
            a_plane(ring.centre(), along)
        } else {
            a_plane(ring.centre(), Vector::new(0.0, 0.0, 1.0))
        };
        assert_eq!(
            [
                how_a_plane_cuts(flat, Surface::Torus(ring)),
                how_a_plane_cuts(flat, Surface::Cylinder(pipe)),
            ],
            [Cut::Curves(2), Cut::Curves(1)],
            "{pipe:?} and {ring:?}"
        );
        intersect_never_calls_them_one(Surface::Cylinder(pipe), Surface::Torus(ring));
    }
}

/// **A cone and a ring.** A plane square to a ring's axis further from its
/// centre than the tube's radius misses the ring. No plane misses a cone,
/// because a cone here is both nappes: of its lines through the apex, at most
/// two run parallel to a plane, and every other one crosses it.
#[test]
fn a_cone_and_a_ring_are_never_one_surface() {
    let mut draw = Draw(0xC019);
    let ring = Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring");
    let round_the_middle = (Cone::new(Axis::z(), 1.0).expect("a slope"), ring);
    let mut pairs = vec![round_the_middle];
    pairs.extend((0..DRAWS).map(|_| {
        let apex = draw.point();
        (draw.cone(apex), draw.ring())
    }));
    for (cone, ring) in pairs {
        let flat = a_plane(
            ring.centre() + Vector::new(0.0, 0.0, ring.tube_radius() + 1.0),
            Vector::new(0.0, 0.0, 1.0),
        );
        let ring_cut = how_a_plane_cuts(flat, Surface::Torus(ring));
        let cone_cut = how_a_plane_cuts(flat, Surface::Cone(cone));
        assert_eq!(ring_cut, Cut::Nothing, "{ring:?}");
        assert_ne!(cone_cut, Cut::Nothing, "{cone:?}");
        intersect_never_calls_them_one(Surface::Cone(cone), Surface::Torus(ring));
    }
}

/// How many pairs each sweep above draws, beside the ones it names.
const DRAWS: usize = 64;

/// What `intersect` cuts a surface in with a plane, in the words the sweeps
/// above compare.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cut {
    Nothing,
    OnePoint,
    Curves(usize),
}

fn how_a_plane_cuts(flat: Plane, surface: Surface) -> Cut {
    let met = intersect(
        &Surface::Plane(flat),
        &surface,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .unwrap_or_else(|why| panic!("{flat:?} against {surface:?}: {why}"));
    match met {
        SurfaceMeet::Nothing => Cut::Nothing,
        SurfaceMeet::TouchAtOnePoint { .. } => Cut::OnePoint,
        SurfaceMeet::Curves(runs) => Cut::Curves(runs.len()),
        SurfaceMeet::SameSurface => panic!("{surface:?} was called the plane {flat:?}"),
        other => panic!("a shape these sweeps have no word for: {other:?}"),
    }
}

/// `intersect` says something other than "one surface" of the two, either way
/// round, or refuses to say anything.
fn intersect_never_calls_them_one(one: Surface, other: Surface) {
    for (first, second) in [(one, other), (other, one)] {
        let met = intersect(
            &first,
            &second,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            !matches!(met, Ok(SurfaceMeet::SameSurface)),
            "{first:?} and {second:?} were called one surface"
        );
    }
}

fn a_plane(through: Point, normal: Vector) -> Plane {
    Plane::from_point_and_normal(through, normal).expect("a normal with a length")
}

/// A deterministic sequence, so that each sweep is the same pairs every run.
struct Draw(u64);

/// Directions an axis is drawn along: the three of the frame, some leaning
/// ones, and some square to `z`, which [`a_pipe_and_a_ring_are_never_one_surface`]
/// needs both of.
const DIRECTIONS: [[f64; 3]; 12] = [
    [0.0, 0.0, 1.0],
    [0.0, 0.0, -1.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [1.0, 1.0, 0.0],
    [3.0, -4.0, 0.0],
    [1.0, -1.0, 1.0],
    [1.0, 2.0, 2.0],
    [0.0, 3.0, 4.0],
    [-2.0, 1.0, 2.0],
    [3.0, 0.0, -4.0],
    [1.0, 1.0, 8.0],
];

impl Draw {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.0 >> 33
    }

    /// A whole number between `low` and `high`, both ends included.
    fn whole(&mut self, low: i64, high: i64) -> f64 {
        let span = u64::try_from(high - low + 1).expect("a range that runs upwards");
        let picked = i64::try_from(self.next() % span).expect("inside the range") + low;
        picked as f64
    }

    fn point(&mut self) -> Point {
        Point::new(self.whole(-6, 6), self.whole(-6, 6), self.whole(-6, 6))
    }

    fn direction(&mut self) -> Vector {
        let count = u64::try_from(DIRECTIONS.len()).expect("a short list");
        let [x, y, z] = DIRECTIONS[usize::try_from(self.next() % count).expect("inside the list")];
        Vector::new(x, y, z)
    }

    fn ball(&mut self) -> Sphere {
        Sphere::new(self.point(), self.whole(1, 4)).expect("a radius")
    }

    fn pipe(&mut self) -> Cylinder {
        let axis = Axis::new(self.point(), self.direction()).expect("an axis");
        Cylinder::new(axis, self.whole(1, 4)).expect("a radius")
    }

    fn cone(&mut self, apex: Point) -> Cone {
        let axis = Axis::new(apex, self.direction()).expect("an axis");
        Cone::new(axis, self.whole(1, 4) / 2.0).expect("a slope")
    }

    /// A ring about `z` with its centre on `z`. A plane holding its axis or
    /// square to it is then written exactly, which `intersect` needs to cut
    /// it in circles rather than refuse the plane as a quartic. Every other
    /// placement is this one moved, and moving both surfaces one way moves
    /// what a plane cuts them in with them.
    fn ring(&mut self) -> Torus {
        let ring = self.whole(3, 8);
        let tube = self.whole(1, 2);
        let centre = Point::new(0.0, 0.0, self.whole(-3, 3));
        Torus::new(
            Axis::new(centre, Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
            ring,
            tube,
        )
        .expect("a tube narrower than its ring")
    }
}
