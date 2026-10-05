//! A ball of radius 1 and a cube of side 4 turned so that one face plane
//! tilts 30 degrees off `z` and passes through the ball's north pole
//! `N = centre + z`, or misses it by about one rounding.
//!
//! **Clear of the ball.** The cube sits beside the ball, with the two boxes
//! overlapping. Q375 found this refused `ACurvedFaceGoesRightRoundWithNoSeam`
//! under union either way round and ball less block, at every tilt direction
//! it tried, while one rounding further off answered. Its fuse cut the ball
//! along the face plane although the cube never reached the ball, and the rim
//! it cut ran through `N` or within a rounding of it.
//!
//! On `656b9f09` every case of that kind answers: the ball comes back as one
//! face, not cut, so there is no rim near `N` for the mesher to read. Commit
//! `3830c0e5` (Q456, a ball clear of a block stays whole) is the change that
//! says so by its title; it was not bisected. The first test pins that answer:
//! union either way round measures the ball and the cube, ball less block
//! measures the ball, and each answer validates. The ball is placed at the
//! origin and at `(0, 0, 7)`, since how far one rounding reaches depends on
//! where the solid sits.
//!
//! **Cutting the ball.** The second test moves the face's square to be centred
//! on the circle the plane cuts (Q547's shape, Q889). The cube then holds all
//! of the ball below the plane, and the cap above it has `N` on its rim or
//! within about a rounding of it. Most cases answer and measure the cap worked
//! out from the plane's distance to the centre; the rest are pinned by name.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Deflection, Isometry, POINT_ALONG_A_CROSSING_ROUNDOFF, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_mesh::CannotTriangulate;
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, Rule, SolidKey};

const SIDE: f64 = 4.0;
const GAP: f64 = 0.1;
const TILT_DEGREES: f64 = 30.0;

#[derive(Clone, Copy, Debug)]
enum PlaneThrough {
    ThePole,
    RoundingsOff(f64),
}

#[test]
fn a_block_clear_of_a_ball_whose_face_plane_holds_its_pole_fuses_and_measures() {
    let ball = 4.0 / 3.0 * PI;
    let cube = SIDE * SIDE * SIDE;
    for centre in [Point::origin(), Point::new(0.0, 0.0, 7.0)] {
        for direction in [30.0, 250.0] {
            for through in [
                PlaneThrough::ThePole,
                PlaneThrough::RoundingsOff(-0.99),
                PlaneThrough::RoundingsOff(-0.5),
                PlaneThrough::RoundingsOff(0.5),
                PlaneThrough::RoundingsOff(0.99),
            ] {
                let placement = where_the_block_goes(centre, direction, through, 1.0 + GAP);
                for (name, rule, ball_first, volume) in [
                    ("ball and block", Rule::Union, true, ball + cube),
                    ("block and ball", Rule::Union, false, ball + cube),
                    ("ball less block", Rule::Difference, true, ball),
                ] {
                    let what = format!(
                        "{name}, ball at {centre:?}, tilted towards {direction} degrees, plane {through:?}"
                    );
                    let (model, solids) = fused(centre, &placement, rule, ball_first)
                        .unwrap_or_else(|why| panic!("{what}: fuses: {why:?}"));
                    validates_and_measures(&model, &solids, volume, &what);
                }
            }
        }
    }
}

/// What a case of the cut ball does on `4358cce4`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Today {
    Answers,
    /// `Invalid(FacesDoNotAgreeWhichWayTheyFace)`. The ball's piece below the
    /// plane is a cap round the south pole whose rim runs through `N`, and `N`
    /// is the last point its walk reaches before closing on the rim's corner.
    /// `WalkingAWire::finish` closes that pole side at the first corner's
    /// longitude as it was read, not a turn on, so the corners run the pole
    /// side across the cap's longitudes instead of the piece's. The walk's own
    /// count is right (`climbed` is a whole turn); only the corner written is
    /// not. Closing it a turn on answers all twelve, and nothing else moved.
    PoleSideClosedAtTheWrongTurn,
    /// `CannotMeshAFace { CornersCoincide }`. The rim's one corner sits
    /// `2.7e-14` from `N`, inside one rounding, so it is walked as the pole,
    /// but at its own latitude `π/2 − 2.7e-14` and not the pinch. The cap round
    /// the south pole is then cut into quarters, and the cutter knows a pole
    /// side only at the pinch's latitude, so the third quarter's pole side has
    /// no width.
    CornerReadAsThePoleOffThePinch,
    /// `TwoPatchesOfTheMeshDrewTheSameSide`. The cap above the plane has `N`
    /// on its rim and is cut narrower than a quarter turn; one cut runs from a
    /// rim point straight to `N` with nothing between, and the disc's
    /// triangulation of the same rim points draws that same chord.
    TheCapsCutToItsPoleIsTheDiscsChord,
}

/// The ball at the origin and the cube whose face's square is centred on the
/// circle the plane cuts, so the cube holds every bit of the ball below the
/// plane and the cap above is the one piece it leaves.
///
/// **What each answer measures is worked out from the plane, not the mesh.**
/// The stored plane's distance from the centre is `d`, the cap above it is
/// `h = 1 − d` high, and a cap of a unit ball holds `π h² (3 − h) / 3`. Ball
/// less block is the cap, ball and block together are the cube and the cap,
/// the two kept in common are the ball less the cap, and block less ball is
/// the cube less that. Each is checked within the error the measure publishes.
///
/// **Twenty-seven of the 144 are refused, each by name** (see [`Today`]),
/// measured on `4358cce4`. Counted with intersection one way round, as the
/// row did before Q847, that is the nineteen of Q889, unchanged by Q847.
#[test]
fn a_block_whose_face_plane_misses_a_balls_pole_by_a_rounding_cuts_off_a_cap_or_is_refused_by_name()
{
    let ball = 4.0 / 3.0 * PI;
    let cube = SIDE * SIDE * SIDE;
    for direction in [0.0, 30.0, 180.0, 250.0] {
        for roundings in [-1.01, -0.99, -0.5, 0.5, 0.99, 1.01] {
            let placement = where_the_block_goes(
                Point::origin(),
                direction,
                PlaneThrough::RoundingsOff(roundings),
                -SIDE / 2.0,
            );
            let cap = the_cap_the_plane_cuts_off(&placement);
            for (name, rule, ball_first, volume) in [
                ("ball and block", Rule::Union, true, cube + cap),
                ("block and ball", Rule::Union, false, cube + cap),
                ("ball less block", Rule::Difference, true, cap),
                (
                    "block less ball",
                    Rule::Difference,
                    false,
                    cube - (ball - cap),
                ),
                ("ball with block", Rule::Intersection, true, ball - cap),
                ("block with ball", Rule::Intersection, false, ball - cap),
            ] {
                let what = format!(
                    "{name}, tilted towards {direction} degrees, plane {roundings} roundings off"
                );
                let said = fused(Point::origin(), &placement, rule, ball_first);
                match what_it_does_today(direction, roundings, name) {
                    Today::Answers => {
                        let (model, solids) =
                            said.unwrap_or_else(|why| panic!("{what}: fuses: {why:?}"));
                        validates_and_measures(&model, &solids, volume, &what);
                    }
                    Today::PoleSideClosedAtTheWrongTurn => assert!(
                        matches!(
                            said,
                            Err(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace))
                        ),
                        "{what}: moved from its pin. Got {:?}",
                        said.map(|(_, solids)| solids)
                    ),
                    Today::CornerReadAsThePoleOffThePinch => assert!(
                        matches!(
                            said,
                            Err(Error::CannotMeshAFace {
                                why: CannotTriangulate::CornersCoincide { .. },
                                ..
                            })
                        ),
                        "{what}: moved from its pin. Got {:?}",
                        said.map(|(_, solids)| solids)
                    ),
                    Today::TheCapsCutToItsPoleIsTheDiscsChord => assert!(
                        matches!(said, Err(Error::TwoPatchesOfTheMeshDrewTheSameSide { .. })),
                        "{what}: moved from its pin. Got {:?}",
                        said.map(|(_, solids)| solids)
                    ),
                }
            }
        }
    }
}

fn what_it_does_today(direction: f64, roundings: f64, name: &str) -> Today {
    let keeps_the_piece_below = matches!(
        name,
        "block less ball" | "ball with block" | "block with ball"
    );
    if direction == 0.0 && roundings <= 0.5 && keeps_the_piece_below {
        return Today::CornerReadAsThePoleOffThePinch;
    }
    if direction == 30.0 && roundings >= -0.5 && keeps_the_piece_below {
        return Today::PoleSideClosedAtTheWrongTurn;
    }
    let folds = [(30.0, -0.5), (250.0, -0.99), (250.0, -0.5)];
    if name == "ball less block" && folds.contains(&(direction, roundings)) {
        return Today::TheCapsCutToItsPoleIsTheDiscsChord;
    }
    Today::Answers
}

/// How much of the unit ball at the origin lies beyond the cube's face plane.
fn the_cap_the_plane_cuts_off(placement: &Isometry) -> f64 {
    let plane = the_cubes_face_plane_on_y_zero().transformed(placement);
    let high = 1.0 - plane.offset().abs();
    PI * high * high * (3.0 - high) / 3.0
}

/// The ball of radius 1 at `centre` fused with the cube at `placement`, and
/// the model the answer is in.
fn fused(
    centre: Point,
    placement: &Isometry,
    rule: Rule,
    ball_first: bool,
) -> Result<(Model, Vec<SolidKey>), Error> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a positive size"));
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(centre - Point::origin()).expect("a finite shift"),
        )
        .expect("a ball");
    let block = model
        .add_cube(SIDE, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(block, placement)
        .expect("a turned cube is placed");
    let (argument, tool) = if ball_first {
        (ball, block)
    } else {
        (block, ball)
    };
    let solids = model
        .fuse(
            &[argument],
            &[tool],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )?
        .solids()
        .to_vec();
    Ok((model, solids))
}

/// Validate the model, and check that the answer's solids together measure
/// `volume` within the error the measure publishes, at two deflections.
fn validates_and_measures(model: &Model, solids: &[SolidKey], volume: f64, what: &str) {
    let policy = Policy::certified();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: validates: {why:?}"));
    for deflection in [0.05, 0.01] {
        let mut measured = 0.0;
        let mut error = 0.0;
        for &solid in solids {
            let measure = model
                .solid(solid)
                .expect("the fuse made the key")
                .measure(
                    Deflection::of(deflection).expect("a positive length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what}: measures at {deflection}: {why:?}"));
            measured += measure.volume();
            error += measure.volume_error();
        }
        assert!(
            (measured - volume).abs() <= error,
            "{what}: at {deflection} measures {measured}, {volume} wanted within {error}"
        );
    }
}

/// The cube turned so that its face `y = 0` faces along the normal tilted
/// `TILT_DEGREES` off `z` towards `direction`, and moved so that the face
/// plane holds `N` or misses it by the roundings asked, with the face's square
/// starting `away` from the middle of the circle the plane cuts the ball in,
/// measured away from `N`, and running on away from it. `1 + GAP` puts the
/// square clear of the circle; `−SIDE / 2` centres it there.
///
/// Through the pole exactly is found by moving the shift a few ulps at a time
/// until the stored plane holds `N` by the certified predicate.
fn where_the_block_goes(
    centre: Point,
    direction: f64,
    through: PlaneThrough,
    away: f64,
) -> Isometry {
    let (tilt, turn_towards) = (TILT_DEGREES.to_radians(), direction.to_radians());
    let n = Vector::new(
        tilt.sin() * turn_towards.cos(),
        tilt.sin() * turn_towards.sin(),
        tilt.cos(),
    );
    let pole = centre + Vector::new(0.0, 0.0, 1.0);
    let up = Vector::new(0.0, 0.0, 1.0);
    let away_from_the_pole = (up - n * up.dot(n)).normalized().expect("tilted") * -1.0;
    let across = n.cross(away_from_the_pole);

    // The face y = 0 faces out along -y, so +y is turned onto -n.
    let y_axis = Vector::new(0.0, 1.0, 0.0);
    let first = Isometry::rotation(
        &Axis::new(Point::origin(), y_axis.cross(n * -1.0)).expect("not parallel"),
        y_axis.dot(n * -1.0).clamp(-1.0, 1.0).acos(),
    )
    .expect("a turn");
    let x_axis = first.apply_to_vector(Vector::new(1.0, 0.0, 0.0));
    let spin = x_axis
        .cross(away_from_the_pole)
        .dot(n)
        .atan2(x_axis.dot(away_from_the_pole));
    let second = Isometry::rotation(&Axis::new(Point::origin(), n).expect("a unit normal"), spin)
        .expect("a turn");
    let turn = second.compose(&first);

    let rounding = POINT_ALONG_A_CROSSING_ROUNDOFF * pole.largest_coordinate_magnitude().max(1.0);
    let off = match through {
        PlaneThrough::ThePole => 0.0,
        PlaneThrough::RoundingsOff(roundings) => roundings * rounding * tilt.sin(),
    };
    let middle = centre + n * ((pole - centre).dot(n) + off);
    let shift = (middle - Point::origin()) + away_from_the_pole * away - across * (SIDE / 2.0);

    let stored = the_cubes_face_plane_on_y_zero();
    let placed = |shift: Vector| {
        Isometry::translation(shift)
            .expect("a finite shift")
            .compose(&turn)
    };
    if let PlaneThrough::RoundingsOff(_) = through {
        return placed(shift);
    }
    let nudged = |x: f64, ulps: i32| {
        (0..ulps.abs()).fold(x, |x, _| if ulps > 0 { x.next_up() } else { x.next_down() })
    };
    for dx in -6..=6 {
        for dy in -6..=6 {
            for dz in -6..=6 {
                let placement = placed(Vector::new(
                    nudged(shift.x(), dx),
                    nudged(shift.y(), dy),
                    nudged(shift.z(), dz),
                ));
                if stored
                    .transformed(&placement)
                    .side_of_point(PredicatePoint::from(pole.coordinates()))
                    == Decision::Zero
                {
                    return placement;
                }
            }
        }
    }
    panic!("no shift within six ulps puts the plane through the pole");
}

fn the_cubes_face_plane_on_y_zero() -> Plane {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a positive size"));
    let cube = model
        .add_cube(SIDE, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    let found: Vec<Plane> = model
        .solid(cube)
        .expect("the model made the key")
        .faces()
        .filter_map(|face| match face.surface() {
            Surface::Plane(plane) if plane.normal().y().abs() == 1.0 && plane.offset() == 0.0 => {
                Some(plane)
            }
            Surface::Plane(_)
            | Surface::Sphere(_)
            | Surface::Cylinder(_)
            | Surface::Cone(_)
            | Surface::Torus(_)
            | _ => None,
        })
        .collect();
    assert_eq!(found.len(), 1, "one face of the cube on y = 0");
    found[0]
}
