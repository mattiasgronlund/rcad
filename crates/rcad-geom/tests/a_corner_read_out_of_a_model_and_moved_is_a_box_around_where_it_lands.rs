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
//! - A cone's corner can sit further from its axis than the cone's `L` counts,
//!   so the cone gets a search of its own, aimed at the share no rounding
//!   makes: a turn stored off length one, turning the corner half a turn about
//!   the line halfway between its axis and it. The worst it finds is printed
//!   too (§464.38).

#![forbid(unsafe_code)]

use nalgebra as na;
use rcad_geom::{
    Axis, Cone, Cylinder, Isometry, Plane, Point, Sphere, Surface, Torus, Vector,
    where_a_corner_lands_after_moving,
};
use rcad_predicates::expansion::{
    compress, expansion_sum, product_is_exact, scale_expansion, two_diff, two_sum,
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
///
/// Since a turn of a cube and a shift that round nothing hand the corner back
/// exact (§34.6's `where_a_corner_lands_after_moving`), a wobble of zero shows
/// up here too — the identity and every whole-number shift among
/// [`every_way_of_placing_it`] give one. Dividing by that zero would be `0.0 / 0.0`, which is `NaN`, and
/// `f64::max` drops a `NaN` silently rather than counting it: the ratio would
/// still measure something, just not every reading. So the two kinds are kept
/// apart — an exact reading is asserted off its surface by exactly nothing,
/// and only an inexact one goes into the ratio.
#[test]
fn a_moved_corner_stays_inside_the_box_published_for_it() {
    let mut worst = 0.0f64;
    let mut exact_readings = 0usize;
    let mut inexact_readings = 0usize;
    for (corner, surfaces) in every_corner_with_two_surfaces_round_it() {
        for movement in every_way_of_placing_it() {
            let on = [&surfaces[0], &surfaces[1]];
            let (box_it_is_in, wobble) = where_a_corner_lands_after_moving(corner, on, &movement)
                .expect("these corners and placements are all finite");
            let moved = movement.apply_to_point(corner);
            for surface in on {
                let placed = surface.transformed(&movement);
                if wobble.magnitude() == 0.0 {
                    exact_readings += 1;
                    assert_eq!(
                        how_far_off(&placed, moved),
                        0.0,
                        "a move that rounds nothing left {placed:?} off by something"
                    );
                } else {
                    inexact_readings += 1;
                    worst = worst.max(how_far_off(&placed, moved) / wobble.magnitude());
                }

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
        "moved corners: {exact_readings} exact readings, {inexact_readings} inexact readings; the \
         furthest any inexact one sat off its own surface was {worst:.3} of the bound published \
         for it"
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

/// A turn of a cube moves each coordinate onto another and negates some, so a
/// corner turned by one, and lifted by whole numbers after, lands exactly where
/// it is turned to, still exactly on every surface it was on, and gets no box.
///
/// Every quarter turn about `x`, `y` and `z`, through the origin, and each
/// lifted the way `a_bar_turned_three_quarter_turns_and_halved.rs` places its
/// bar. The lift is kept to the corners made of whole numbers, whose sums it
/// cannot round.
#[test]
fn a_corner_turned_by_quarter_turns_lands_exactly_where_it_is_turned_to() {
    let lift = Isometry::translation(Vector::new(0.0, 0.0, -2.0)).expect("a finite shift");
    let mut readings = 0usize;
    for (corner, surfaces) in every_corner_with_two_surfaces_round_it() {
        let whole = corner.coordinates().iter().all(|it| it.fract() == 0.0);
        for axis in [Axis::x(), Axis::y(), Axis::z()] {
            for quarters in 1..4 {
                let turn =
                    Isometry::quarter_turns(&axis, quarters).expect("an axis along x, y or z");
                let lifted = lift.compose(&turn);
                for movement in [Some(turn), whole.then_some(lifted)].into_iter().flatten() {
                    let on = [&surfaces[0], &surfaces[1]];
                    let (landed, wobble) = where_a_corner_lands_after_moving(corner, on, &movement)
                        .expect("these corners and turns are all finite");
                    let what = format!("{corner:?} turned {quarters} quarters by {movement:?}");
                    assert_eq!(wobble.magnitude(), 0.0, "{what}");
                    assert_eq!(
                        landed,
                        PredicatePoint::from(movement.apply_to_point(corner).coordinates()),
                        "{what}"
                    );
                    for surface in on {
                        assert_eq!(
                            which_side_of(&surface.transformed(&movement), landed),
                            Decision::Zero,
                            "{what} is off {surface:?} turned with it"
                        );
                    }
                    readings += 1;
                }
            }
        }
    }
    assert_eq!(
        readings,
        8 * 9 + 6 * 9,
        "eight corners turned nine ways, and the six whole ones lifted too"
    );
}

/// A quarter turn about an axis that misses the origin carries a shift, and
/// where adding that shift to the turned corner rounds, the corner keeps its
/// box and is certified off neither surface it is a rounding from.
///
/// About the axis through `(1/10, 0, 0)` the bar's cap corner `(2, 0, 3)`
/// lands at `2 − 1/10` along `y`, which rounds. About the parallel axis through
/// `(1, 0, 0)` it lands at `(1, 1, 3)`, which does not, so what keeps the box is
/// the rounding and not the axis being somewhere else.
#[test]
fn a_corner_turned_about_an_axis_whose_shift_rounds_when_added_keeps_its_box() {
    let corner = Point::new(2.0, 0.0, 3.0);
    let wall = Surface::Cylinder(Cylinder::new(Axis::z(), 2.0).expect("a radius"));
    let cap = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 3.0), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal"),
    );
    let on = [&cap, &wall];
    let a_quarter_about_z_through = |x: f64| {
        let axis =
            Axis::new(Point::new(x, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0)).expect("a direction");
        Isometry::quarter_turns(&axis, 1).expect("an axis along z")
    };

    let rounds = a_quarter_about_z_through(0.1);
    assert_ne!(
        two_sum(2.0, -0.1).1,
        0.0,
        "2 − 1/10 rounds, so the corner's sum does"
    );
    let (boxed, wobble) =
        where_a_corner_lands_after_moving(corner, on, &rounds).expect("finite numbers");
    assert!(
        boxed.uncertainty() > 0.0 && wobble.magnitude() > 0.0,
        "{boxed:?}"
    );
    for surface in on {
        let placed = surface.transformed(&rounds);
        assert!(
            !matches!(which_side_of(&placed, boxed), Decision::Pos | Decision::Neg),
            "the turned corner was certified off {placed:?}: {boxed:?}"
        );
    }

    let whole = a_quarter_about_z_through(1.0);
    let (exact, wobble) =
        where_a_corner_lands_after_moving(corner, on, &whole).expect("finite numbers");
    assert_eq!(exact, PredicatePoint::from([1.0, 1.0, 3.0]));
    assert_eq!(wobble.magnitude(), 0.0);
}

/// A corner far out on a wide cone, turned the way that takes it furthest off
/// its cone, stays inside the box published for it.
///
/// A cone's `L` counts its apex and slope and not how far it reaches, so a
/// corner on one can sit up to `2√3 · L` from its axis where a pipe's sits
/// within `L`. A turn stored a little longer or shorter than length one moves
/// such a corner off its cone by about `2 · (q·q − 1)` times that distance, and
/// most when it is half a turn about the line halfway between the axis and the
/// corner. On paper that share, the slips and the tilt come to about `290u`,
/// past the `256u` the box is (§464.21's 3); this is the search for a corner
/// that gets there (Q2830).
///
/// So the corners here sit as far from their axes as `L` lets them, and each is
/// turned half a turn about that halfway line by the longest and the shortest
/// turn a movement keeps without dividing it, as well as every placement
/// [`every_way_of_placing_it`] makes. Not every corner is exactly on its cone,
/// since a slope of `2√2` is not an `f64`, so what is measured is how much
/// further off its cone the move took it, worked out exactly.
#[test]
fn a_moved_corner_far_out_on_a_wide_cone_stays_inside_the_box_published_for_it() {
    // The worst share a stretched half turn took, beside `2 · |q·q − 1| · ρ` for
    // the same reading, as a share of the same box.
    let mut worst_turned_halfway = (0.0f64, 0.0f64);
    let mut worst_placed = 0.0f64;
    let mut furthest_off_length_one = 0.0f64;
    let mut readings = 0usize;
    for (corner, cone) in corners_far_out_on_wide_cones() {
        let out = corner - cone.apex();
        let along = cone.axis().direction();
        let from_the_axis = out.cross(along).length() / along.length();
        let halfway = halfway_between_the_axis_and(corner, &cone);
        let through = cone.apex() + out * 0.5;
        let stretched = [true, false].map(|longer| {
            halfway
                .map(|about| half_a_turn_stored_off_length_one(about, through, longer))
                .to_vec()
        });
        for (off_length_one, _) in stretched.iter().flatten() {
            furthest_off_length_one = furthest_off_length_one.max(off_length_one.abs());
        }
        let movements = stretched
            .into_iter()
            .flatten()
            .map(|(off_length_one, it)| (Some(off_length_one), it));
        let placings = every_way_of_placing_it().into_iter().map(|it| (None, it));
        for (off_length_one, movement) in movements.chain(placings) {
            let surface = Surface::Cone(cone);
            let (box_it_is_in, wobble) =
                where_a_corner_lands_after_moving(corner, [&surface], &movement)
                    .expect("these corners and movements are all finite");
            let placed = cone.transformed(&movement);
            let moved = movement.apply_to_point(corner);
            let further_off = (how_far_off_a_cone_exactly(&placed, moved)
                - how_far_off_a_cone_exactly(&cone, corner))
            .abs();
            readings += 1;
            if wobble.magnitude() == 0.0 {
                assert_eq!(
                    further_off, 0.0,
                    "a move that rounds nothing took {corner:?} further off {cone:?}"
                );
                continue;
            }
            let share = further_off / wobble.magnitude();
            match off_length_one {
                Some(off) if share > worst_turned_halfway.0 => {
                    let drift = 2.0 * off.abs() * from_the_axis / wobble.magnitude();
                    worst_turned_halfway = (share, drift);
                }
                Some(_) => {}
                None => worst_placed = worst_placed.max(share),
            }

            let placed = Surface::Cone(placed);
            assert!(
                !matches!(
                    which_side_of(&placed, box_it_is_in),
                    Decision::Pos | Decision::Neg
                ),
                "a moved corner was certified off {placed:?}: {box_it_is_in:?}"
            );
            assert!(
                which_side_of(&moved_clear(&placed, wobble.magnitude()), box_it_is_in)
                    .is_certified(),
                "the same box against {placed:?} moved comfortably clear was not decided"
            );
        }
    }
    let (worst_turned_halfway, its_drift) = worst_turned_halfway;
    println!(
        "moved corners on cones: {readings} readings; the furthest a move took one off its \
         cone was {worst_turned_halfway:.3} of the bound published for it turned half a turn \
         about the halfway line by a stretched turn, where 2 |q.q - 1| rho is {its_drift:.3} of \
         it, and {worst_placed:.3} placed any other way; the stretched turns' q.q was up to \
         {:.2} eps off one, worked out exactly",
        furthest_off_length_one / f64::EPSILON
    );
    assert!(worst_turned_halfway < NO_CORNER_MAY_USE_ALL_OF_ITS_BOUND);
    assert!(worst_placed < NO_CORNER_MAY_USE_ALL_OF_ITS_BOUND);
    assert!(
        worst_turned_halfway > worst_placed,
        "half a turn by a stretched turn is built to take a corner furthest off its cone, and \
         took one {worst_turned_halfway} of the bound against {worst_placed} for the rest"
    );
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

/// Corners on cones, each about as far from its cone's axis as `L` lets it be.
///
/// Two kinds. A cone about `z` with the corner `(119, 120)` across from the
/// axis, `169` out, a Pythagorean triple, so the corner is exactly on it. And a
/// cube's long diagonal, corner to corner, on the cone about another of the
/// cube's diagonals, `2√2` times as far out as along: the corner is `√(32/3)`,
/// about `3.27`, times `L` from the axis, of the most `2√3`, and half a turn
/// about the line halfway between takes the cube onto itself, so the move
/// adds nothing to `L`.
fn corners_far_out_on_wide_cones() -> Vec<(Point, Cone)> {
    let mut all = Vec::new();
    for size in [1.0, 2f64.powi(30)] {
        for slope in [1.0, 4.0] {
            let up = 84.5 * size / slope;
            let axis = Axis::new(
                Point::new(-60.0 * size, -60.0 * size, -up),
                Vector::new(0.0, 0.0, 1.0),
            )
            .expect("a direction");
            all.push((
                Point::new(59.0 * size, 60.0 * size, up),
                Cone::new(axis, slope).expect("a slope"),
            ));
        }
    }
    for size in [4.0, 2f64.powi(30)] {
        for [x, y, z] in [[1.0, 1.0, -1.0], [1.0, -1.0, 1.0], [-1.0, 1.0, 1.0]] {
            let axis = Axis::new(Point::new(-size, -size, -size), Vector::new(x, y, z))
                .expect("a direction");
            all.push((
                Point::new(size, size, size),
                Cone::new(axis, 8f64.sqrt()).expect("a slope"),
            ));
        }
    }
    all
}

/// The two lines halfway between `cone`'s axis and the line from its apex
/// through `corner`, one each way round.
fn halfway_between_the_axis_and(corner: Point, cone: &Cone) -> [Vector; 2] {
    let out = corner - cone.apex();
    let out = (out * (1.0 / out.length())).components();
    let along = cone.axis().direction();
    let along = (along * (1.0 / along.length())).components();
    [1.0, -1.0].map(|way| {
        Vector::new(
            out[0] + way * along[0],
            out[1] + way * along[1],
            out[2] + way * along[2],
        )
    })
}

/// Half a turn about `about` through `through`, stored as far from length one
/// as a movement keeps a turn without dividing it by its length: as long as it
/// can be when `longer`, as short as it can be otherwise. Handed back with its
/// `q·q − 1`, worked out exactly and rounded once.
///
/// Read in from nalgebra, which keeps a turn's bits while `q·q` as `f64` works
/// it out is within `4ε` of one; `q·q` worked out exactly can sit further off.
fn half_a_turn_stored_off_length_one(
    about: Vector,
    through: Point,
    longer: bool,
) -> (f64, Isometry) {
    let unit = (about * (1.0 / about.length())).components();
    let [x, y, z] = through.coordinates();
    let mut best: Option<(f64, Isometry)> = None;
    for nudges in 0..17i64 * 17 * 17 {
        let by = [nudges % 17 - 8, nudges / 17 % 17 - 8, nudges / 289 - 8];
        let [i, j, k] = [0, 1, 2].map(|axis| nudged(unit[axis], by[axis]));
        let largest = i.abs().max(j.abs()).max(k.abs());
        if [i, j, k].iter().all(|it| *it == 0.0 || it.abs() == largest) {
            // A turn of a cube, which a movement does exactly.
            continue;
        }
        let turn = na::UnitQuaternion::new_unchecked(na::Quaternion::new(0.0, i, j, k));
        let movement = Isometry::try_from(na::Isometry3::rotation_wrt_point(
            turn,
            na::Point3::new(x, y, z),
        ))
        .expect("a finite turn");
        if movement.rotation_quaternion().map(f64::to_bits) != [0.0, i, j, k].map(f64::to_bits) {
            // Divided by its length on the way in.
            continue;
        }
        let off = [i, j, k]
            .iter()
            .fold(Exactly::of(-1.0), |sum, it| {
                sum.plus(&Exactly::of(*it).times(&Exactly::of(*it)))
            })
            .nearest();
        let further = |than: f64| if longer { off > than } else { off < than };
        if best.as_ref().is_none_or(|(most, _)| further(*most)) {
            best = Some((off, movement));
        }
    }
    best.expect("some nudge of the turn keeps its bits")
}

/// `value` moved `ulps` steps along the `f64`s, away from zero for a positive
/// count. Zero stays zero.
fn nudged(value: f64, ulps: i64) -> f64 {
    if value == 0.0 {
        value
    } else {
        f64::from_bits(value.to_bits().wrapping_add_signed(ulps))
    }
}

/// How far `point` sits off `cone`, outward positive, worked out exactly up to
/// the last division.
///
/// With `w` the stored axis direction, `|out × w|² − s²(out · w)²` is
/// `|w|²(ρ − s|t|)(ρ + s|t|)`, `ρ` how far the point is from the axis and `t`
/// how far along it, and `(ρ − s|t|) / √(1 + s²)` is how far it is off the
/// cone. The first is summed exactly; the rest only divides it by numbers
/// `f64` holds to a rounding.
fn how_far_off_a_cone_exactly(cone: &Cone, point: Point) -> f64 {
    let apex = cone.apex().coordinates();
    let at = point.coordinates();
    let out = [0, 1, 2].map(|axis| Exactly::difference(at[axis], apex[axis]));
    let w = cone.axis().direction().components();
    let across = [(1, 2), (2, 0), (0, 1)].map(|(a, b)| {
        out[a]
            .times(&Exactly::of(w[b]))
            .minus(&out[b].times(&Exactly::of(w[a])))
    });
    let across_squared = across
        .iter()
        .fold(Exactly::of(0.0), |sum, it| sum.plus(&it.times(it)));
    let up = (0..3).fold(Exactly::of(0.0), |sum, axis| {
        sum.plus(&out[axis].times(&Exactly::of(w[axis])))
    });
    let up_by_slope = up.times(&Exactly::of(cone.slope()));
    let gap = across_squared
        .minus(&up_by_slope.times(&up_by_slope))
        .nearest();
    let length_squared = cone.axis().direction().length_squared();
    let from_the_axis = (across_squared.nearest() / length_squared).sqrt();
    let along = up.nearest().abs() / length_squared.sqrt();
    gap / (length_squared * (from_the_axis + cone.slope() * along) * cone.slope().hypot(1.0))
}

/// A number held exactly, as `f64` entries that do not overlap, smallest first,
/// the way `rcad_predicates::expansion` keeps one.
struct Exactly(Vec<f64>);

impl Exactly {
    fn of(value: f64) -> Self {
        Self(vec![value])
    }

    fn difference(a: f64, b: f64) -> Self {
        let (rounded, lost) = two_diff(a, b);
        Self(vec![lost, rounded])
    }

    fn plus(&self, other: &Self) -> Self {
        let mut sum = vec![0.0; self.0.len() + other.0.len()];
        let used = expansion_sum(&self.0, &other.0, &mut sum);
        sum.truncate(used);
        Self(sum)
    }

    fn minus(&self, other: &Self) -> Self {
        self.plus(&Self(other.0.iter().map(|it| -it).collect()))
    }

    fn times(&self, other: &Self) -> Self {
        other.0.iter().fold(Self(Vec::new()), |sum, &factor| {
            assert!(
                self.0.iter().all(|&entry| product_is_exact(entry, factor)),
                "a product here fell below the normals, so it is not exact"
            );
            let mut scaled = vec![0.0; 2 * self.0.len()];
            let used = scale_expansion(&self.0, factor, &mut scaled);
            scaled.truncate(used);
            sum.plus(&Self(scaled))
        })
    }

    fn nearest(&self) -> f64 {
        let mut packed = vec![0.0; self.0.len()];
        let used = compress(&self.0, &mut packed);
        packed[..used].iter().sum()
    }
}
