//! A ball of radius 1 and a cube of side 4 turned so that one face plane
//! tilts 30 degrees off `z` and passes through the ball's north pole
//! `N = centre + z`, or misses it by less than one rounding. The cube sits
//! beside the ball, clear of it, with the two boxes overlapping.
//!
//! Q375 found this refused `ACurvedFaceGoesRightRoundWithNoSeam` under union
//! either way round and ball less block, at every tilt direction it tried,
//! while one rounding further off answered. Its fuse cut the ball along the
//! face plane although the cube never reached the ball, and the rim it cut ran
//! through `N` or within a rounding of it.
//!
//! On `656b9f09` every case here answers: the ball comes back as one face, not
//! cut, so there is no rim near `N` for the mesher to read. Commit `3830c0e5`
//! (Q456, a ball clear of a block stays whole) is the change that says so by
//! its title; it was not bisected. This file pins that answer: union either
//! way round measures the ball and the cube, ball less block measures the
//! ball, and each answer validates.
//!
//! The ball is placed at the origin and at `(0, 0, 7)`, since how far one
//! rounding reaches depends on where the solid sits.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Deflection, Isometry, POINT_ALONG_A_CROSSING_ROUNDOFF, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

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
                let placement = where_the_block_goes(centre, direction, through);
                for (name, rule, ball_first, volume) in [
                    ("ball and block", Rule::Union, true, ball + cube),
                    ("block and ball", Rule::Union, false, ball + cube),
                    ("ball less block", Rule::Difference, true, ball),
                ] {
                    let what = format!(
                        "{name}, ball at {centre:?}, tilted towards {direction} degrees, plane {through:?}"
                    );
                    fuses_validates_and_measures(
                        centre, &placement, rule, ball_first, volume, &what,
                    );
                }
            }
        }
    }
}

/// Fuse, validate the model, and check that the answer's solids together
/// measure `volume` within the error the measure publishes, at two
/// deflections.
fn fuses_validates_and_measures(
    centre: Point,
    placement: &Isometry,
    rule: Rule,
    ball_first: bool,
    volume: f64,
    what: &str,
) {
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
    let policy = Policy::certified();
    let fused = model
        .fuse(
            &[argument],
            &[tool],
            rule,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: fuses: {why:?}"));
    let solids: Vec<SolidKey> = fused.solids().to_vec();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: validates: {why:?}"));
    for deflection in [0.05, 0.01] {
        let mut measured = 0.0;
        let mut error = 0.0;
        for &solid in &solids {
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
/// starting `GAP` beyond the circle the plane cuts the ball in and running away
/// from `N`.
///
/// Through the pole exactly is found by moving the shift a few ulps at a time
/// until the stored plane holds `N` by the certified predicate.
fn where_the_block_goes(centre: Point, direction: f64, through: PlaneThrough) -> Isometry {
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
    let shift =
        (middle - Point::origin()) + away_from_the_pole * (1.0 + GAP) - across * (SIDE / 2.0);

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
