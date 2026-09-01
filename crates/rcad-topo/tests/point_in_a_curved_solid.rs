//! Inside a bar, inside a ball, or on the skin of one.
//!
//! `point_in_solid.rs` asks the same questions of polyhedra and has since M0.
//! What is new here is that the ray meets a *curved* surface: where it does is a
//! quadratic in one unknown, and whether that place is on the face is
//! [`SolidView::face_holds_the_point`](rcad_topo::SolidView::face_holds_the_point).
//!
//! `docs/decisions.md` §31.7's criterion 10 names the first two tests below and
//! criterion 9 names `whether_a_curved_face_holds_a_point_agrees_with_a_mesh_of_the_same_face`.

use rcad_geom::{Deflection, Isometry, Point, Scale, Surface};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, FaceKey, Mesh, Model, OnAFace, SolidKey};

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 5.0;

fn a_round_bar() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar three across and five tall");
    (model, bar)
}

fn a_ball() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball of radius three");
    (model, ball)
}

/// §31.7's criterion 10, first half.
///
/// A point exactly on a bar's wall, away from both caps and away from the seam.
/// `x² + y² − r²` is `0 + 9 − 9` and cancels to exactly zero, so the surface
/// itself is certified; what the answer then rests on is that the wall's two cap
/// circles put the point between them, which is the whole of what a curved face
/// holding a point comes to here.
#[test]
fn the_middle_of_a_bars_wall_is_on_its_skin() {
    let (model, bar) = a_round_bar();
    let mut log = ProvenanceLog::new();
    let on_the_wall = Point::new(0.0, RADIUS, HEIGHT / 2.0);

    assert_eq!(
        model
            .solid(bar)
            .classify_point(on_the_wall, &Policy::certified(), &mut log),
        Containment::OnBoundary
    );
    assert!(
        log.records().is_empty(),
        "a bar on the z axis needs nothing approximated, got {:?}",
        log.records()
    );
}

/// §31.7's criterion 10, second half.
///
/// One `f64` step inside the skin, which is the tightest a point can be asked
/// about without being on it. The ray meets the sphere twice, both places ahead
/// of the point, and the two crossings leave a winding of one.
#[test]
fn a_point_a_hair_inside_a_balls_skin_is_inside_it() {
    let (model, ball) = a_ball();
    let mut log = ProvenanceLog::new();

    assert_eq!(
        model.solid(ball).classify_point(
            Point::new(0.0, 0.0, RADIUS.next_down()),
            &Policy::certified(),
            &mut log
        ),
        Containment::Inside
    );
    assert_eq!(
        model.solid(ball).classify_point(
            Point::new(0.0, 0.0, RADIUS.next_up()),
            &Policy::certified(),
            &mut log
        ),
        Containment::Outside
    );
    assert_eq!(
        model.solid(ball).classify_point(
            Point::new(0.0, 0.0, RADIUS),
            &Policy::certified(),
            &mut log
        ),
        Containment::OnBoundary,
        "a pole is a corner of the one face, and the face holds it"
    );
    assert!(
        log.records().is_empty(),
        "a ball about the origin needs nothing approximated, got {:?}",
        log.records()
    );
}

/// The bulge is what a box round the corners misses, and reading a point against
/// that box would have called this one outside without firing a ray.
///
/// A bar's two corners are both on its seam at `x = 3, y = 0`, so the box round
/// them is a line. The point below is a whole radius away from it and squarely
/// inside the bar.
#[test]
fn a_point_in_the_bulge_a_bar_makes_away_from_its_corners_is_inside_it() {
    let (model, bar) = a_round_bar();
    let mut log = ProvenanceLog::new();

    assert_eq!(
        model.solid(bar).classify_point(
            Point::new(-2.0, 0.0, HEIGHT / 2.0),
            &Policy::certified(),
            &mut log
        ),
        Containment::Inside
    );
    assert_eq!(
        model.solid(bar).classify_point(
            Point::new(-4.0, 0.0, HEIGHT / 2.0),
            &Policy::certified(),
            &mut log
        ),
        Containment::Outside
    );
    assert!(log.records().is_empty(), "{:?}", log.records());
}

/// Level with the wall but past the end of it: on the cylinder, off the face.
///
/// This is the one a face's trimming decides and nothing else does. Both points
/// are certified onto the same surface; what tells them apart is which side of
/// the cap planes they are on.
#[test]
fn a_point_on_a_bars_cylinder_past_its_cap_is_outside_the_bar() {
    let (model, bar) = a_round_bar();
    let mut log = ProvenanceLog::new();

    for beyond in [-1.0, HEIGHT + 1.0] {
        assert_eq!(
            model.solid(bar).classify_point(
                Point::new(0.0, RADIUS, beyond),
                &Policy::certified(),
                &mut log
            ),
            Containment::Outside,
            "at height {beyond}"
        );
    }
    assert!(log.records().is_empty(), "{:?}", log.records());
}

/// The middle of a cap is on the skin, and the middle of the *plane* it sits on
/// is not.
///
/// A round cap is a flat face whose whole outline is one closed curve, so there
/// is no polygon to wind and the winding count has to read the circle as a
/// circle. Before that rule existed the count was taken over a single corner and
/// came out zero everywhere, which would have put the middle of every cap
/// outside the bar.
#[test]
fn the_middle_of_a_round_cap_is_on_the_skin_and_the_rest_of_its_plane_is_not() {
    let (model, bar) = a_round_bar();
    let mut log = ProvenanceLog::new();

    assert_eq!(
        model.solid(bar).classify_point(
            Point::new(0.5, 0.5, 0.0),
            &Policy::certified(),
            &mut log
        ),
        Containment::OnBoundary,
        "inside the bottom cap's circle"
    );
    assert_eq!(
        model.solid(bar).classify_point(
            Point::new(4.0, 4.0, 0.0),
            &Policy::certified(),
            &mut log
        ),
        Containment::Outside,
        "on the cap's plane and well outside its circle"
    );
    assert!(log.records().is_empty(), "{:?}", log.records());
}

/// §31.7's criterion 16 in miniature, for the one thing this lane decides: a
/// lattice-aligned curved control is decided everywhere and records nothing.
///
/// A grid rather than a handful, because what is being checked is that nothing
/// **anywhere** falls through to `Undecidable` or to a face nobody can follow.
/// The grid is offset by an eighth so that no sample lands on a cap plane or on
/// the seam, which are the boundary cases the tests above pin one at a time.
#[test]
fn a_grid_of_points_against_a_bar_and_a_ball_is_decided_and_records_nothing() {
    let mut counted = 0;
    for (model, solid, what) in [
        (a_round_bar().0, a_round_bar().1, "bar"),
        (a_ball().0, a_ball().1, "ball"),
    ] {
        let mut log = ProvenanceLog::new();
        for i in -8..=8 {
            for j in -8..=8 {
                for k in -8..=8 {
                    let at = Point::new(
                        f64::from(i) / 2.0 + 0.125,
                        f64::from(j) / 2.0 + 0.125,
                        f64::from(k) / 2.0 + 0.125,
                    );
                    let placed = model
                        .solid(solid)
                        .classify_point(at, &Policy::certified(), &mut log);
                    assert!(
                        matches!(
                            placed,
                            Containment::Inside | Containment::Outside | Containment::OnBoundary
                        ),
                        "the {what} could not place {at:?}: {placed:?}"
                    );
                    counted += 1;
                }
            }
        }
        assert!(
            log.records().is_empty(),
            "the {what} recorded {:?}",
            log.records()
        );
    }
    assert_eq!(counted, 2 * 17 * 17 * 17);
}

/// §31.7's criterion 9: one authority on whether a curved face holds a point,
/// and it agrees with a fine mesh of that same face wherever both can tell.
///
/// **The mesh is the oracle and not the answer.** A mesh of one face is a set of
/// flat pieces standing in for it, everywhere within `Mesh::deviation` of the
/// surface, so a point *on* the surface is within that of the mesh exactly when
/// it is on the face — up to a band round the boundary where the mesh cannot
/// tell either. Samples in that band are skipped, which is what "wherever both
/// can tell" means, and the count that is compared is asserted so that a band
/// swallowing the whole test would fail rather than pass.
#[test]
fn whether_a_curved_face_holds_a_point_agrees_with_a_mesh_of_the_same_face() {
    let fine = Deflection::of(0.002).expect("a fine mesh");
    let mut compared = 0;
    let mut held = 0;

    for (model, solid, what) in [
        (a_round_bar().0, a_round_bar().1, "bar"),
        (a_ball().0, a_ball().1, "ball"),
    ] {
        let mut log = ProvenanceLog::new();
        let mesh = model
            .solid(solid)
            .mesh(fine, &Policy::certified(), &mut log)
            .expect("a bar and a ball both mesh");
        // The mesh's own deviation, plus the longest chord a triangle of it can
        // have — a point is that far from a mesh point even when it is squarely
        // on the face.
        let near = mesh.deviation().magnitude() + longest_chord(&mesh);
        let far = 20.0 * near;

        for face in model.solid(solid).faces() {
            let surface = face.surface();
            if surface.as_plane().is_some() {
                continue;
            }
            for sample in samples_over(&surface) {
                let by_mesh = how_far_from_the_mesh_of(&mesh, face.key(), sample);
                let by_the_face = model
                    .solid(solid)
                    .face_holds_the_point(
                        face.key(),
                        sample.coordinates().into(),
                        &Policy::certified(),
                        &mut log,
                    )
                    .unwrap_or_else(|why| panic!("the {what}'s face at {sample:?}: {why}"));
                if by_mesh <= near {
                    assert_ne!(
                        by_the_face,
                        OnAFace::NotHeld,
                        "the mesh of the {what} covers {sample:?} and the face says it does not"
                    );
                    compared += 1;
                    held += 1;
                } else if by_mesh >= far {
                    assert_eq!(
                        by_the_face,
                        OnAFace::NotHeld,
                        "the mesh of the {what} is {by_mesh} from {sample:?} and the face holds it"
                    );
                    compared += 1;
                }
            }
        }
    }

    assert!(
        compared > 400,
        "only {compared} samples were far enough from the boundary to compare"
    );
    assert!(
        held > 100 && held < compared,
        "{held} of {compared} were held, which is not a two-sided comparison"
    );
}

/// Points on a surface to ask about, spread well past whatever the face covers.
///
/// A pipe is sampled a long way up and down its axis so that the samples run off
/// both caps of any bar drawn on it; a ball has no off, so it is sampled all over
/// and every sample should be held.
fn samples_over(surface: &Surface) -> Vec<Point> {
    let mut points = Vec::new();
    for round in 0..24 {
        let u = f64::from(round) * core::f64::consts::TAU / 24.0;
        match surface {
            Surface::Cylinder(_) => {
                for step in -8..=24 {
                    points.push(surface.point_at(u, f64::from(step) * 0.25));
                }
            }
            _ => {
                for step in -11..=11 {
                    points.push(surface.point_at(u, f64::from(step) * core::f64::consts::PI / 24.0));
                }
            }
        }
    }
    points
}

/// How far `at` is from the nearest triangle of one face of `mesh`.
fn how_far_from_the_mesh_of(mesh: &Mesh, face: FaceKey, at: Point) -> f64 {
    mesh.triangles()
        .iter()
        .filter(|triangle| triangle.face() == face)
        .map(|triangle| {
            let [a, b, c] = triangle.corners().map(|corner| mesh.points()[corner as usize]);
            how_far_from_a_triangle(at, a, b, c)
        })
        .fold(f64::INFINITY, f64::min)
}

/// The distance from `at` to the nearest point of the triangle `a b c`.
///
/// The nearest point is inside the triangle or on one of its three sides, so
/// this takes the smaller of the two: the drop onto the plane when that lands
/// inside, and the nearest point of each side otherwise.
fn how_far_from_a_triangle(at: Point, a: Point, b: Point, c: Point) -> f64 {
    let normal = (b - a).cross(c - a);
    let along_sides = [(a, b), (b, c), (c, a)]
        .into_iter()
        .map(|(from, to)| how_far_from_a_side(at, from, to))
        .fold(f64::INFINITY, f64::min);
    if normal.length() == 0.0 {
        return along_sides;
    }
    let inside = [(a, b), (b, c), (c, a)]
        .into_iter()
        .all(|(from, to)| (to - from).cross(at - from).dot(normal) >= 0.0);
    if inside {
        (at - a).dot(normal).abs() / normal.length()
    } else {
        along_sides
    }
}

/// The distance from `at` to the nearest point of the piece from `from` to `to`.
fn how_far_from_a_side(at: Point, from: Point, to: Point) -> f64 {
    let side = to - from;
    let squared = side.dot(side);
    if squared == 0.0 {
        return (at - from).length();
    }
    let part = ((at - from).dot(side) / squared).clamp(0.0, 1.0);
    (at - (from + side * part)).length()
}

/// The longest side of any triangle in the mesh.
fn longest_chord(mesh: &Mesh) -> f64 {
    mesh.triangles()
        .iter()
        .flat_map(|triangle| {
            let [a, b, c] = triangle.corners().map(|corner| mesh.points()[corner as usize]);
            [
                (b - a).length(),
                (c - b).length(),
                (a - c).length(),
            ]
        })
        .fold(0.0, f64::max)
}
