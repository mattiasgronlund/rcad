//! Everything M2 added, reached with `rcad` as the only dependency.
//!
//! A re-export list with nothing consuming it is a list that goes stale
//! quietly, and `docs/decisions.md` §13 rule 3 is about exactly that. This file
//! names [`Mesh`], [`MeshTriangle`], [`Deflection`] and [`CannotTriangulate`]
//! from the facade and nowhere else, so a name dropped from it stops the build
//! here rather than in somebody's project.

use rcad::{
    CannotTriangulate, Deflection, Mesh, MeshTriangle, Model, Plane, Point, Policy, ProvenanceLog,
    Scale, TopologyError, Vector,
};

const RADIUS: f64 = 2.0;
const HEIGHT: f64 = 6.0;

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A bar, meshed and measured with nothing but the front door.
#[test]
fn a_bar_meshes_and_measures_from_the_front_door() {
    let mut model = a_model();
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, &rcad::Isometry::identity())
        .expect("a bar two across and six tall");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar is a sound solid");

    let how_fine = Deflection::of(0.01).expect("a hundredth of a millimetre");
    let mesh: Mesh = model
        .solid(bar)
        .expect("this model made the key")
        .mesh(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar this size meshes");
    assert!(mesh.deviation().magnitude() <= how_fine.length());

    let faces: std::collections::HashSet<_> = mesh
        .triangles()
        .iter()
        .map(|triangle: &MeshTriangle| triangle.face())
        .collect();
    assert_eq!(faces.len(), 3, "two caps and a wall");
    assert!(mesh.points().len() > 3 * faces.len());

    let measured = model
        .solid(bar)
        .expect("this model made the key")
        .measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("and measures");
    let exact = core::f64::consts::PI * RADIUS * RADIUS * HEIGHT;
    assert!(
        measured.volume() < exact && exact - measured.volume() <= measured.volume_error(),
        "a mesh inside the bar reports {} of {exact} and admits to {}",
        measured.volume(),
        measured.volume_error()
    );
    // Three quantities in three units, and the units are the whole point.
    assert!(measured.area_error() > 0.0, "a squared unit");
    assert!(measured.centroid_wobble() > 0.0, "a length");
    assert!(
        measured
            .centroid()
            .distance_to(Point::new(0.0, 0.0, HEIGHT / 2.0))
            <= measured.centroid_wobble(),
        "the middle of a bar on the z axis is half way up it"
    );
}

/// A cube is a polyhedron, so the deflection is ignored and nothing is
/// approximated — the same call, the other path.
#[test]
fn a_cube_measured_from_the_front_door_approximates_nothing() {
    let mut model = a_model();
    let mut log = ProvenanceLog::new();
    let cube = model
        .add_cube(4.0, &Policy::certified(), &mut log)
        .expect("a cube of side four");

    let measured = model
        .solid(cube)
        .expect("this model made the key")
        .measure(
            Deflection::of(1e9).expect("a length nothing here reads"),
            &Policy::certified(),
            &mut log,
        )
        .expect("a cube measures");
    assert_eq!(measured.volume(), 64.0);
    assert_eq!(measured.volume_error(), 0.0);
    assert_eq!(measured.area_error(), 0.0);
    assert_eq!(measured.centroid_wobble(), 0.0);
    assert!(log.is_empty(), "a polyhedron's fan approximates nothing");
}

/// Why [`CannotTriangulate`] is out here at all: a caller handed one has to be
/// able to say which refusal it was without naming `rcad-mesh`.
#[test]
fn a_caller_can_read_which_refusal_the_triangulator_gave() {
    let mut model = a_model();
    let ball = model
        .add_ball(1.0, &rcad::Isometry::identity())
        .expect("a unit ball");

    // Fine enough that the inside of the patch asks for more points than one
    // patch will hold, and coarse enough that its boundary does not — which is
    // the band where the refusal a caller has to read comes from the
    // triangulator rather than from the walk or the floor.
    let refusal = model.solid(ball).expect("this model made the key").mesh(
        Deflection::of(1e-6).expect("a length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    match refusal {
        Err(TopologyError::CannotMeshAFace { why, .. }) => match why {
            CannotTriangulate::MoreTrianglesThanAPatchWillHold { would_need } => {
                assert!(would_need > 0.0);
            }
            other => panic!("a millionth of the ball asks for triangles, not {other:?}"),
        },
        other => panic!("a ball meshed to a millionth gave {other:?}"),
    }
}

/// §24.18's criterion 8 from outside the workspace: a solid whose edges were
/// **computed** meshes and weighs something.
///
/// The whole of M3 is behind this one call. `add_bar_cut_off_by` asks
/// [`intersect`](rcad::intersect) where a leaning plane and a pipe cross, puts
/// the answer on an edge, and the mesher hands that edge back to `walk` to be
/// followed — and the shape that comes out is one no builder in M2 could
/// declare, because which conic a leaning plane cuts out of a pipe is a
/// question rather than a fact.
///
/// §24.15's argument for exporting any of this is that it has users who are not
/// kernel users, and a user is what this file is.
#[test]
fn a_bar_cut_off_at_an_angle_is_built_and_meshed_from_the_front_door() {
    let mut model = a_model();
    let bar = model
        .add_bar_cut_off_by(
            RADIUS,
            [
                Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
                    .expect("the z axis is a unit normal"),
                Plane::from_point_and_normal(
                    Point::new(0.0, 0.0, HEIGHT),
                    Vector::new(1.0, 0.0, 3.0),
                )
                .expect("a direction with length"),
            ],
            &rcad::Isometry::identity(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a leaning plane cuts one closed curve out of a pipe");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar cut off at an angle is a sound solid");

    let how_fine = Deflection::of(0.01).expect("a hundredth of a millimetre");
    let mesh: Mesh = model
        .solid(bar)
        .expect("this model made the key")
        .mesh(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("and meshes");
    assert!(mesh.deviation().magnitude() <= how_fine.length());
    assert_eq!(
        mesh.triangles()
            .iter()
            .map(|triangle: &MeshTriangle| triangle.face())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        3,
        "two caps and a wall"
    );

    // A bar cut off at an angle holds what an upright one of the average height
    // holds: the slice taken off one side is the slice added to the other.
    let measured = model
        .solid(bar)
        .expect("this model made the key")
        .measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("and weighs something");
    let upright = core::f64::consts::PI * RADIUS * RADIUS * HEIGHT;
    assert!(
        (measured.volume() - upright).abs() <= upright / 100.0 + measured.volume_error(),
        "a bar cut off at an angle holds {} and an upright one holds {upright}",
        measured.volume()
    );
}
