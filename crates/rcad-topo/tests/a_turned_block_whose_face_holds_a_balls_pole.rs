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
//! within about a rounding of it. Most cases answer, measure the cap worked
//! out from the plane's distance to the centre and hold the points their rule
//! says; the rest are pinned by name.
//!
//! **Through the pole exactly, with the square centred on the circle.** The
//! third test is Q888's shape with the face plane through `N` exactly rather
//! than a rounding off it, at every tilt direction and both the origin and
//! `(0, 0, 7)`, all six fuses measured and validated. Q2495's fix is what
//! answers it. The cap's height `h` is `1 − |n_z|`, the placed face plane's
//! own normal's `z`, rather than the second test's distance worked out from
//! the ball's centre: since the plane is pinned to `N = centre + z` by
//! construction, `h` does not move with the centre, while that distance does.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Deflection, Isometry, POINT_ALONG_A_CROSSING_ROUNDOFF, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_mesh::CannotTriangulate;
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Mesh, Model, Rule, SolidKey};

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
                    validates_and_measures(&model, &solids, volume, &[0.05, 0.01], &what);
                }
            }
        }
    }
}

/// What a case of the cut ball does today.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Today {
    Answers,
    /// `CannotMeshAFace { LoopsTouch }`. The rim's one corner sits within
    /// one rounding of `N` — measured at `2.7e-14` to `5.5e-14`, depending
    /// on the case — so it is walked as the pole, but at its own latitude
    /// and not the pinch. The ball face's loop then runs up the meridian
    /// through that corner, out to the rim and round it, and back down the
    /// same meridian to close on the pole corner again. The side closing the
    /// loop runs along that same meridian, so the rim corner the loop passed
    /// on its way out sits on that closing side, and `triangulate` refuses
    /// the loop as `CannotTriangulate::LoopsTouch`.
    CornerReadAsThePoleOffThePinch,
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
/// the cube less that. Each is checked within the error the measure publishes,
/// measured at deflections of `0.3`, `0.1`, `0.05` and `0.01`.
/// With the plane through `N` tilted 30 degrees, `d` is `cos 30° = √3/2` to
/// within a rounding, so the cap is `π (1 − √3/2)² (2 + √3/2) / 3 ≈ 0.0539`.
///
/// **And each answer's mesh winds once round the points its rule says, and
/// not round the others.** Four points, each well clear of both skins: the
/// centre and `(0, 0, −0.95)` near the south pole, which are the ball below
/// the plane; `n (1 − h/2)` along the plane's normal `n`, which is in the cap;
/// and `−1.5 n`, which is in the block clear of the ball. The union holds all
/// four, ball less block only the cap's, block less ball only the block's,
/// and the two kept in common the centre and the south pole's.
///
/// **Twelve of the 144 are refused, each by name** (see [`Today`]). Ball less
/// block at direction 30 and `−0.5` roundings and at 250 and `−0.5` and
/// `−0.99` were refused `TwoPatchesOfTheMeshDrewTheSameSide` until Q964: the
/// cap above the plane, `N` on its rim, was cut in two and its half cut again
/// from a rim point straight to `N` with no point between, and the disc drew
/// the same chord across the same rim. The two at 250 were still refused it
/// measured at `0.05` until Q1026: the half holding the circle's one corner,
/// next to `N` on the rim, was handed over with that corner marked as where
/// the rim passes from one edge to another, though the rim is one closed edge,
/// so the cap cut the corner off with a side from `N` to the rim point past it
/// and the disc cut off the same three rim points. All three were still
/// refused it measured at `0.3` and `0.1` until Q1027, and so was ball less
/// block at 180 and `−0.5` at `0.3`. In the half whose pole side runs from
/// where the cut reaches `N` to where the rim does, the triangulator cut the
/// side from the rim's `N` to the rim point two on, which cuts off the one
/// between, and drew the side from the cut's `N` to that point instead, the
/// same two points; the disc, fanned from `N`, drew it too. That was traced at
/// 30 and at 250 and `−0.5`, at `0.3`; the rest answer with the same change.
/// Twelve
/// more, at direction 30 with the block less the ball or either intersection,
/// were refused `Invalid(FacesDoNotAgreeWhichWayTheyFace)` until Q963: the
/// piece below the plane is a cap round the south pole whose rim runs through
/// `N`, `N` is the last point its walk reaches, and `WalkingAWire::finish`
/// wrote that pole side's far corner at the rim corner's longitude as read
/// rather than the whole turn on that the walk had counted.
#[test]
fn a_block_whose_face_plane_misses_a_balls_pole_by_a_rounding_cuts_off_a_cap_or_is_refused_by_name()
{
    let ball = 4.0 / 3.0 * PI;
    let cube = SIDE * SIDE * SIDE;
    for direction in [0.0, 30.0, 180.0, 250.0] {
        let n = tilted_towards(direction);
        for roundings in [-1.01, -0.99, -0.5, 0.5, 0.99, 1.01] {
            let placement = where_the_block_goes(
                Point::origin(),
                direction,
                PlaneThrough::RoundingsOff(roundings),
                -SIDE / 2.0,
            );
            let high = how_high_the_cap_the_plane_cuts_off_is(&placement);
            let cap = PI * high * high * (3.0 - high) / 3.0;
            let points = [
                Point::origin(),
                Point::new(0.0, 0.0, -0.95),
                Point::origin() + n * (1.0 - high / 2.0),
                Point::origin() + n * -1.5,
            ];
            for (name, rule, ball_first, volume, holds) in [
                (
                    "ball and block",
                    Rule::Union,
                    true,
                    cube + cap,
                    [true, true, true, true],
                ),
                (
                    "block and ball",
                    Rule::Union,
                    false,
                    cube + cap,
                    [true, true, true, true],
                ),
                (
                    "ball less block",
                    Rule::Difference,
                    true,
                    cap,
                    [false, false, true, false],
                ),
                (
                    "block less ball",
                    Rule::Difference,
                    false,
                    cube - (ball - cap),
                    [false, false, false, true],
                ),
                (
                    "ball with block",
                    Rule::Intersection,
                    true,
                    ball - cap,
                    [true, true, false, false],
                ),
                (
                    "block with ball",
                    Rule::Intersection,
                    false,
                    ball - cap,
                    [true, true, false, false],
                ),
            ] {
                let what = format!(
                    "{name}, tilted towards {direction} degrees, plane {roundings} roundings off"
                );
                let said = fused(Point::origin(), &placement, rule, ball_first);
                match what_it_does_today(direction, roundings, name) {
                    Today::Answers => {
                        let (model, solids) =
                            said.unwrap_or_else(|why| panic!("{what}: fuses: {why:?}"));
                        validates_and_measures(
                            &model,
                            &solids,
                            volume,
                            &[0.3, 0.1, 0.05, 0.01],
                            &what,
                        );
                        winds_round_just(&model, &solids, &points, holds, &what);
                    }
                    Today::CornerReadAsThePoleOffThePinch => assert!(
                        matches!(
                            said,
                            Err(Error::CannotMeshAFace {
                                why: CannotTriangulate::LoopsTouch { .. },
                                ..
                            })
                        ),
                        "{what}: moved from its pin. Got {:?}",
                        said.map(|(_, solids)| solids)
                    ),
                }
            }
        }
    }
}

#[test]
fn a_turned_blocks_face_through_a_balls_pole_cuts_off_a_cap_and_measures() {
    let cube = SIDE * SIDE * SIDE;
    let ball = 4.0 / 3.0 * PI;
    for centre in [Point::origin(), Point::new(0.0, 0.0, 7.0)] {
        for direction in [0.0, 30.0, 180.0, 250.0] {
            let placement =
                where_the_block_goes(centre, direction, PlaneThrough::ThePole, -SIDE / 2.0);
            let high = 1.0
                - the_cubes_face_plane_on_y_zero()
                    .transformed(&placement)
                    .normal()
                    .z()
                    .abs();
            let cap = PI * high * high * (3.0 - high) / 3.0;
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
                    "{name}, ball at {centre:?}, tilted towards {direction} degrees, plane through the pole"
                );
                let (model, solids) = fused(centre, &placement, rule, ball_first)
                    .unwrap_or_else(|why| panic!("{what}: fuses: {why:?}"));
                validates_and_measures(&model, &solids, volume, &[0.3, 0.1, 0.05, 0.01], &what);
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
    Today::Answers
}

/// How high the cap of the unit ball at the origin that lies beyond the
/// cube's face plane is.
fn how_high_the_cap_the_plane_cuts_off_is(placement: &Isometry) -> f64 {
    let plane = the_cubes_face_plane_on_y_zero().transformed(placement);
    1.0 - plane.offset().abs()
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
/// `volume` within the error the measure publishes, at each of `deflections`.
fn validates_and_measures(
    model: &Model,
    solids: &[SolidKey],
    volume: f64,
    deflections: &[f64],
    what: &str,
) {
    let policy = Policy::certified();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: validates: {why:?}"));
    for &deflection in deflections {
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
        println!("{what}: at {deflection} measures {measured}, {volume} wanted within {error}");
        assert!(
            (measured - volume).abs() <= error,
            "{what}: at {deflection} measures {measured}, {volume} wanted within {error}"
        );
    }
}

/// Check that the answer's meshes, taken together, wind once round each of
/// `points` that `holds` says and not at all round the others.
///
/// **Counted on the mesh, not asked of `classify_point`**, which answers
/// `CannotFollowAFacesTrimming` for the cut ball's or the cube's face on
/// almost every answer here. The mesh is what Q963 changed, and a mesh that
/// does not close, or folds back over itself, winds round a point a number of
/// times that is not a whole one, or the wrong whole one.
fn winds_round_just(
    model: &Model,
    solids: &[SolidKey],
    points: &[Point],
    holds: [bool; 4],
    what: &str,
) {
    let meshes: Vec<Mesh> = solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("the fuse made the key")
                .mesh(
                    Deflection::of(0.01).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what}: meshes: {why:?}"))
        })
        .collect();
    for (&point, held) in points.iter().zip(holds) {
        let turns: f64 = meshes
            .iter()
            .map(|mesh| how_many_times_it_winds_round(mesh, point))
            .sum();
        let wanted = f64::from(u8::from(held));
        assert!(
            (turns - wanted).abs() < 1e-9,
            "{what}: the answer winds {turns} times round {point:?}, {wanted} wanted"
        );
    }
}

/// The solid angle the mesh's triangles cover seen from `point`, in whole
/// spheres: van Oosterom and Strackee's angle for each triangle, added up.
fn how_many_times_it_winds_round(mesh: &Mesh, point: Point) -> f64 {
    let covered: f64 = mesh
        .triangles()
        .iter()
        .map(|triangle| {
            let [a, b, c] = triangle
                .corners()
                .map(|at| mesh.points()[at as usize] - point);
            let (la, lb, lc) = (a.length(), b.length(), c.length());
            2.0 * a
                .dot(b.cross(c))
                .atan2(la * lb * lc + a.dot(b) * lc + a.dot(c) * lb + b.dot(c) * la)
        })
        .sum();
    covered / (4.0 * PI)
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
    let tilt = TILT_DEGREES.to_radians();
    let n = tilted_towards(direction);
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

/// The unit normal tilted `TILT_DEGREES` off `z` towards `direction`, in
/// degrees round `z` from `x`.
fn tilted_towards(direction: f64) -> Vector {
    let (tilt, turn_towards) = (TILT_DEGREES.to_radians(), direction.to_radians());
    Vector::new(
        tilt.sin() * turn_towards.cos(),
        tilt.sin() * turn_towards.sin(),
        tilt.cos(),
    )
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
