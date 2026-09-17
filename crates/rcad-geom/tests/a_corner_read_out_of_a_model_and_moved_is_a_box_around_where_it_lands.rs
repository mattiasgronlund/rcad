//! §31.3's corner with two surfaces round it, and §31.10 item 6's number.
//!
//! A round bar's cap corner has the cap's plane and the wall's cylinder round it
//! and nothing else, and the whole cap circle satisfies both — so §17.1's
//! "re-derive every corner as the meet of the planes around it" has nothing to
//! solve and the corner is read out of the model instead. Read, it is exactly on
//! both. **Moved, it is not**, because the placement rounds the point and the
//! surface separately.
//!
//! This file is the demonstration of both halves, and it is written §19's way
//! round — it asserts what the arithmetic does, so a change that made the moved
//! corner exact again would fail it loudly rather than quietly stop mattering.
//!
//! - The moved corner **handed over as an exact triple** gets a confident `Pos`
//!   or `Neg` about a rounding. That is §15.5's mistake and §27.7's road, and it
//!   is counted here rather than argued.
//! - The same corner **handed over as its box** is never certified either way.
//! - The box against the same surface moved comfortably clear **is** decided, so
//!   the first two halves are not passing for a box a mile wide.
//! - The worst share of [`CORNER_AFTER_MOVING_ROUNDOFF`] any of it actually uses
//!   is measured and printed (§15.6).

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cone, Cylinder, Isometry, Plane, Point, Sphere, Surface, Torus, Vector,
    where_a_corner_lands_after_moving,
};
use rcad_predicates::{Decision, Point as PredicatePoint, SideOfSurface};

/// How far a surface is moved to make a control that must be decided.
///
/// A millionth of the size. Nine orders wider than the published bound, so a
/// failure here is a box that has stopped meaning anything.
const COMFORTABLY_CLEAR: f64 = 1e-6;

/// The most of its own published bound a moved corner was ever found to use.
///
/// [`CORNER_AFTER_MOVING_ROUNDOFF`](rcad_geom::CORNER_AFTER_MOVING_ROUNDOFF) is
/// what that bound is a multiple of.
///
/// **One is the failure**: at one the bound is no longer a bound. Measured over
/// the shapes and turns below, across sixteen orders of magnitude of size, the
/// worst is about a twenty-fifth — so a search finds about `5 · f64::EPSILON` of
/// the size against a published `128`. That is the ordinary gap between a worst
/// case and a drawn one, because roundings mostly cancel, and it is the same
/// shape `Surface::ROUND_TRIP_ROUNDOFF` records: `2.4 · f64::EPSILON` found
/// against `256` published.
const NO_CORNER_MAY_USE_ALL_OF_ITS_BOUND: f64 = 1.0;

/// The bound holds, and the box is not so wide that holding it means nothing.
#[test]
fn a_moved_corner_stays_inside_the_box_published_for_it() {
    let mut worst = 0.0f64;
    let mut checked = 0usize;
    for (corner, surfaces) in every_corner_with_two_surfaces_round_it() {
        for movement in every_way_of_placing_it() {
            let on = [&surfaces[0], &surfaces[1]];
            let (box_it_is_in, wobble) = where_a_corner_lands_after_moving(corner, on, &movement)
                .expect("these corners and placements are all finite");
            let moved = movement.apply_to_point(corner);
            for surface in on {
                let placed = surface.transformed(&movement);
                checked += 1;
                worst = worst.max(how_far_off(&placed, moved) / wobble.magnitude());

                assert!(
                    !matches!(
                        which_side_of(&placed, box_it_is_in),
                        Decision::Pos | Decision::Neg
                    ),
                    "a moved corner was certified off {placed:?} it is a rounding from: \
                     {box_it_is_in:?}"
                );
                assert!(
                    which_side_of(&moved_clear(&placed, wobble.magnitude()), box_it_is_in)
                        .is_certified(),
                    "the same box against {placed:?} moved comfortably clear was not decided"
                );
            }
        }
    }
    println!(
        "moved corners: {checked} readings, the furthest any of them sat off its own surface was \
         {worst:.3} of the bound published for it"
    );
    assert!(worst < NO_CORNER_MAY_USE_ALL_OF_ITS_BOUND);
}

/// A corner read out of the model is exactly on both its surfaces, and the same
/// corner moved is provably off at least one of them.
///
/// **This is the reason the box exists**, and it is a measurement rather than a
/// paragraph. Handed over as an exact triple the moved corner buys a certified
/// sign — an exact answer to the wrong question (§2.5) — and if that never
/// happened there would be nothing here to fix.
#[test]
fn a_corner_that_was_exactly_on_its_surfaces_is_certified_off_them_once_it_is_moved() {
    let mut exactly_on_before = 0usize;
    let mut certified_off_after = 0usize;
    let mut readings = 0usize;

    for (corner, surfaces) in every_corner_with_two_surfaces_round_it() {
        for surface in &surfaces {
            assert_eq!(
                which_side_of(surface, PredicatePoint::from(corner.coordinates())),
                Decision::Zero,
                "{corner:?} is read out of the model as exactly on {surface:?}"
            );
            exactly_on_before += 1;
        }
        for movement in every_way_of_placing_it() {
            let moved = movement.apply_to_point(corner);
            for surface in &surfaces {
                readings += 1;
                let placed = surface.transformed(&movement);
                if matches!(
                    which_side_of(&placed, PredicatePoint::from(moved.coordinates())),
                    Decision::Pos | Decision::Neg
                ) {
                    certified_off_after += 1;
                }
            }
        }
    }

    println!(
        "{exactly_on_before} corner readings are exactly Zero before the move; after it, \
         {certified_off_after} of {readings} are certified off the surface as an exact triple"
    );
    assert!(
        certified_off_after > readings / 2,
        "if a moved corner were still on its surface there would be no box to publish, and only \
         {certified_off_after} of {readings} came back certified off"
    );
}

/// A corner or a placement made of numbers that are not numbers is refused
/// rather than turned into a box round nowhere.
#[test]
fn a_corner_that_is_not_a_place_gets_no_box_at_all() {
    let (corner, surfaces) = every_corner_with_two_surfaces_round_it()
        .into_iter()
        .next()
        .expect("there is at least one");
    let on = [&surfaces[0], &surfaces[1]];

    assert!(
        where_a_corner_lands_after_moving(
            Point::new(f64::NAN, 0.0, 0.0),
            on,
            &Isometry::identity()
        )
        .is_err()
    );
    // A corner at the top of the range shifted by the top of the range lands
    // nowhere `f64` can name, and a box round nowhere would read as a place.
    let far = Isometry::translation(Vector::new(f64::MAX, f64::MAX, f64::MAX))
        .expect("f64::MAX is finite");
    assert!(
        where_a_corner_lands_after_moving(Point::new(f64::MAX, f64::MAX, f64::MAX), on, &far)
            .is_err()
    );
    assert!(where_a_corner_lands_after_moving(corner, on, &Isometry::identity()).is_ok());
}

/// A round bar's cap corner and a ball's pole, at sizes across sixteen orders of
/// magnitude.
///
/// Every corner here is a whole number of a power-of-two size, so it starts
/// exactly on both its surfaces and the only thing that puts it off them is the
/// move.
fn every_corner_with_two_surfaces_round_it() -> Vec<(Point, [Surface; 2])> {
    let mut all = Vec::new();
    for scale in [1e-8, 1.0, 4.0, 1e8] {
        let (radius, height) = (2.0 * scale, 3.0 * scale);

        // A bar's cap corner: the cap's plane and the wall's cylinder, and the
        // whole cap circle satisfies both.
        let wall = Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a radius"));
        let cap = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        );
        all.push((Point::new(radius, 0.0, height), [cap, wall]));

        // A ball's pole against the plane through it, which is the same shape
        // one surface kind along.
        let ball = Surface::Sphere(Sphere::new(Point::origin(), radius).expect("a radius"));
        let through_the_pole = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.0, 0.0, radius), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        );
        all.push((Point::new(0.0, 0.0, radius), [through_the_pole, ball]));
    }
    all
}

/// Every placement a face's composed location can be, in miniature: no move, a
/// shift, a turn, and both.
fn every_way_of_placing_it() -> Vec<Isometry> {
    let mut all = vec![Isometry::identity()];
    for shift in [
        Vector::new(1.0, 0.0, 0.0),
        Vector::new(-7.0, 11.0, 0.5),
        Vector::new(1e7, 1e7, 1e7),
    ] {
        all.push(Isometry::translation(shift).expect("a finite shift"));
    }
    for direction in [
        Vector::new(1.0, 2.0, 3.0),
        Vector::new(0.0, 0.0, 1.0),
        Vector::new(-1.0, 1.0, 0.0),
    ] {
        let axis = Axis::new(Point::origin(), direction).expect("a direction");
        for steps in 1..32 {
            let angle = core::f64::consts::TAU * f64::from(steps) / 32.0;
            let turn = Isometry::rotation(&axis, angle).expect("a finite angle");
            all.push(turn);
            all.push(
                Isometry::translation(Vector::new(-7.0, 11.0, 0.5))
                    .expect("a finite shift")
                    .compose(&turn),
            );
        }
    }
    all
}

/// The same surface moved comfortably clear of where it was.
fn moved_clear(surface: &Surface, reach: f64) -> Surface {
    let gap = reach / COMFORTABLY_CLEAR;
    match surface {
        Surface::Plane(flat) => {
            let [a, b, c, d] = flat.coefficients();
            Surface::Plane(
                Plane::from_coefficients([a, b, c, d - gap]).expect("a plane moved along itself"),
            )
        }
        Surface::Sphere(ball) => Surface::Sphere(
            Sphere::new(ball.centre(), ball.radius() + gap).expect("a larger radius"),
        ),
        Surface::Cylinder(pipe) => Surface::Cylinder(
            Cylinder::new(pipe.axis(), pipe.radius() + gap).expect("a larger radius"),
        ),
        // A cone has no radius to grow; sliding its apex up its own axis moves
        // every one of its points off the cone it came from.
        Surface::Cone(cone) => Surface::Cone(
            Cone::new(
                Axis::new(
                    cone.apex() + cone.axis().direction() * gap,
                    cone.axis().direction(),
                )
                .expect("a direction with length"),
                cone.slope(),
            )
            .expect("the same slope"),
        ),
        // A torus cannot grow its tube without running through its own ring, so
        // this slides its centre up its own axis instead, exactly as a cone's
        // apex is slid.
        Surface::Torus(ring) => Surface::Torus(
            Torus::new(
                Axis::new(
                    ring.centre() + ring.axis().direction() * gap,
                    ring.axis().direction(),
                )
                .expect("a direction with length"),
                ring.ring_radius(),
                ring.tube_radius(),
            )
            .expect("the same two radii"),
        ),
        other => panic!("nothing here can move {other:?} clear"),
    }
}

/// Which side of a surface a point is on, whichever kind it is.
fn which_side_of(surface: &Surface, point: PredicatePoint) -> Decision {
    match surface {
        Surface::Plane(flat) => flat.side_of_point(point),
        Surface::Sphere(ball) => ball.side_of(point),
        Surface::Cylinder(pipe) => pipe.side_of(point),
        Surface::Cone(cone) => cone.side_of(point),
        Surface::Torus(ring) => ring.side_of(point),
        other => panic!("nothing here can place a point against {other:?}"),
    }
}

/// How far a point really sits off a surface, in the metric that surface's own
/// stored numbers name.
fn how_far_off(surface: &Surface, point: Point) -> f64 {
    match surface {
        Surface::Plane(flat) => {
            (flat.normal().dot(point - Point::origin()) - flat.offset()).abs()
                / flat.normal().length()
        }
        Surface::Sphere(ball) => ((point - ball.centre()).length() - ball.radius()).abs(),
        Surface::Cylinder(pipe) => {
            let along = pipe.axis().direction();
            ((point - pipe.axis().origin()).cross(along).length() / along.length() - pipe.radius())
                .abs()
        }
        Surface::Cone(cone) => {
            let along = cone.axis().direction();
            let out = point - cone.apex();
            (out.cross(along).length() - cone.slope() * out.dot(along).abs()).abs()
                / (along.length() * cone.slope().hypot(1.0))
        }
        Surface::Torus(ring) => {
            let along = ring.axis().direction();
            let out = point - ring.centre();
            let across = out.cross(along).length() / along.length() - ring.ring_radius();
            let axial = out.dot(along) / along.length();
            (across.hypot(axial) - ring.tube_radius()).abs()
        }
        other => panic!("nothing here can measure against {other:?}"),
    }
}
