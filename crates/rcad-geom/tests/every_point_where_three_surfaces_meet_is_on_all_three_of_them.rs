//! §31.7's criteria 5, 6 and 7, and the figures §34 records.
//!
//! # "Certified ON" has to mean "never certified OFF", one dimension down
//!
//! Criterion 5 asks that every point where three surfaces meet come back as a
//! box "certified `Zero` on all three". Taken literally that is not something
//! any amount of precision reaches, and the reason is the one
//! `every_point_along_a_crossing_is_on_both_surfaces.rs` already argues at
//! length: where a line cuts a ball is at coordinates no `f64` can name, so a
//! box round it has real width and an interval evaluation of `‖p − c‖² − R²`
//! over that box straddles zero. `Decision::Zero` about it would take the box to
//! have no width at all, which happens only where the arithmetic rounded
//! nothing.
//!
//! So the oracle is the same one M3 used, with the same two halves:
//!
//! - **Nothing where three surfaces meet is ever certified off any of them.** A
//!   `Pos` or a `Neg` is a bug — an exact answer to the wrong question (§2.5).
//! - **The same box against a surface moved comfortably clear is decided.**
//!   Otherwise the first half would pass just as well for boxes a mile wide.
//!
//! And where the arithmetic *does* round nothing — a lattice triple whose
//! corner is a whole number — the answer is a box of no width and a certified
//! `Zero` on all three. That is measured too, because it is the half of
//! criterion 5 that can be met to the letter.
//!
//! # How wide the boxes are
//!
//! Reported as a share of the largest number the triple was written with, per
//! family and never blended (§17.7).

#![forbid(unsafe_code)]

mod crossings;

use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};
use rcad_geom::{
    Axis, Cone, Cylinder, Error, MeetOfThree, Plane, Point, Sphere, Surface, Torus, Vector,
    meet_of_three_surfaces,
};
use rcad_predicates::{
    Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface,
};

use crossings::{
    Corner, Elsewhere, Piece, a_corner_somewhere_else, a_lattice_corner, a_leaning_corner,
    a_turned_corner,
};

/// How many triples each measured row draws.
const CASES: u32 = 2048;

/// How far a surface is moved to make a control that must be decided.
///
/// A millionth of the size of the triple. Ten orders of magnitude wider than any
/// box measured below, so a row that fails this half is one whose boxes have
/// stopped meaning anything rather than one that was asked too much.
const COMFORTABLY_CLEAR: f64 = 1e-6;

/// The widest box any row is allowed to hand back, as a share of the largest
/// number the corner was worked out from.
///
/// **A floor under nonsense and not a derived bound**, and it is set where it is
/// because two real mistakes were caught by moving it there. Reading the
/// discriminant's size in intervals rather than out of the exact expansion put
/// the `somewhere else` row at `2.1e-8`; taking the two roots as `(−B ± √Δ)/A`
/// rather than through the stable pair put one turned corner's box at fifteen
/// hundred times the shape. The rows now measure between `2.2e-16` and
/// `1.8e-14`, so this leaves about five orders of headroom — enough that an
/// ordinary drawing does not trip it and far too little for either of those two
/// to come back unnoticed. §34 records what each row measures.
const NO_BOX_MAY_BE_A_BILLIONTH_OF_THE_SHAPE: f64 = 1e-9;

/// The control: whole numbers, planes square to an axis.
///
/// **§31.7's criterion 6.** Nothing on the lattice may refuse, and every corner
/// on it lands on a whole number, so the boxes have no width and the `Zero`
/// criterion 5 asks for is reached to the letter.
#[test]
fn a_certified_meeting_of_three_lattice_surfaces_never_refuses() {
    let measured = meet_every_triple(a_lattice_corner(), "the control");
    assert_eq!(
        measured.refused, 0,
        "the lattice control refused {} of {} triples",
        measured.refused, measured.drawn
    );
    assert!(
        measured.certified_zero > 0,
        "a lattice corner that rounds nothing is certified Zero on all three, and none was"
    );
}

/// The same shapes with the planes and pipes tilted, so a stored direction is a
/// rounding of the one that was asked for.
#[test]
fn every_point_where_three_leaning_surfaces_meet_is_on_all_three_of_them() {
    let measured = meet_every_triple(a_leaning_corner(), "leaning");
    assert!(measured.worst_box < NO_BOX_MAY_BE_A_BILLIONTH_OF_THE_SHAPE);
}

/// The headline: the control under one general rotation.
#[test]
fn every_point_where_three_turned_surfaces_meet_is_on_all_three_of_them() {
    let measured = meet_every_triple(a_turned_corner(), "turned");
    assert!(measured.worst_box < NO_BOX_MAY_BE_A_BILLIONTH_OF_THE_SHAPE);
}

/// The control written somewhere else in `f64`, where the exact arithmetic runs
/// out of range where the triple stands and the refusal is a **gap** rather
/// than a limit.
///
/// **So none of them is refused**, and that is pinned rather than printed. The
/// same triple slid by a power of two is answered (§349): 180 of these 2051
/// were refused before the slide, and 52 while it brought the largest length
/// to one instead of centring them all on it.
#[test]
fn every_point_where_three_surfaces_meet_somewhere_else_in_f64_is_on_all_three_of_them() {
    let measured = meet_every_triple(a_corner_somewhere_else(), "somewhere else");
    assert!(measured.worst_box < NO_BOX_MAY_BE_A_BILLIONTH_OF_THE_SHAPE);
    assert_eq!(
        measured.refused, 0,
        "{} of {} triples written somewhere else were refused",
        measured.refused, measured.drawn
    );
}

/// The triple the proptest seed `e0a2a205` shrank to, by name: the planes
/// `y = 2` and `z = 0` and a ball of radius three about `(0, 1, 0)`, every
/// number multiplied by `2⁻⁵²⁸`, so about `1e-159` across.
///
/// Every product of two of those numbers is near `2⁻¹⁰⁵⁶`, among the
/// subnormals, where each one loses bits. §322.4 kept the triple refused, because
/// the box round what was lost is a millionth of the shape wide. Slid up the
/// range by a power of two it is the same three surfaces at size one, and a slide
/// is exact both ways while every number stays normal.
///
/// **So the oracle is the triple at home**: the same three written at `2⁰` meet
/// at two corners, and these two are those, every end of every box moved down
/// `2⁻⁵²⁸` and not one bit different. Then the first oracle of this file on top:
/// on all three, and decided against each one moved clear.
#[test]
fn two_planes_and_a_ball_written_about_1e_159_across_meet_where_the_same_three_do_at_home() {
    let tiny = Corner {
        first: Piece::Flat {
            normal: [0, 1, 0],
            at: 2,
        },
        second: Piece::Flat {
            normal: [0, 0, 1],
            at: 0,
        },
        third: Piece::Ball {
            centre: [0, 1, 0],
            radius: 3,
        },
        turn: None,
        elsewhere: Elsewhere {
            scale: -528,
            shift: None,
        },
    };
    let at_home = Corner {
        elsewhere: Elsewhere::here(),
        ..tiny
    };
    let [answer, home] = [tiny, at_home].map(|corner| {
        let surfaces = corner.surfaces();
        meet_of_three_surfaces(
            [&surfaces[0], &surfaces[1], &surfaces[2]],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
    });
    let home = home.expect("the lattice triple is answered");
    let MeetOfThree::At(home_points) = &home else {
        panic!("a line through a ball's middle meets it twice, not {home:?}");
    };
    let answer = answer.expect("the same triple 2^-528 across is answered");
    let MeetOfThree::At(points) = &answer else {
        panic!("a line through a ball's middle meets it twice, not {answer:?}");
    };
    assert_eq!(
        points.iter().map(both_ends_of).collect::<Vec<_>>(),
        home_points
            .iter()
            .map(|point| both_ends_of(point).map(|side| side.map(|end| end * 2f64.powi(-528))))
            .collect::<Vec<_>>(),
        "the corners 2^-528 across are the corners at home moved down, bit for bit"
    );

    let surfaces = tiny.surfaces();
    for point in points {
        if let Err(why) = on_all_three_and_clear_of_each_moved(&surfaces, tiny.how_big(), *point) {
            panic!("{why}");
        }
    }
}

/// **The limit the slide runs into**: the same triple written `2⁻¹⁰⁶²` across,
/// among the subnormals, where a corner has no box a billionth of the shape
/// wide because `f64` has no numbers that close together.
///
/// The slide up is exact, because a subnormal moved up the range is a normal
/// number with every bit it had. The corner is found at size one. It is the
/// slide back that cannot be exact: `x = ±√8·2⁻¹⁰⁶²` lands between two
/// subnormals `2⁻¹⁰⁷⁴` apart, and one such step is already `8e-5` of the
/// ball's `3·2⁻¹⁰⁶²`. So the answer is a box at most two subnormal steps wide
/// on each side that still holds the corner. Measured: `1.6e-4` of the shape,
/// where `NO_BOX_MAY_BE_A_BILLIONTH_OF_THE_SHAPE` lets a row hand back `1e-9`.
/// `√8` is not a fraction, so no box of `x` in `f64` is narrower than one step.
/// A box a billionth of the shape does not exist there, so this is a limit and
/// not a gap (§16.10), and §349 says why the family's range stops short of it.
#[test]
fn three_surfaces_written_among_the_subnormals_meet_in_a_box_a_subnormal_step_or_two_wide() {
    let tiny = f64::from_bits(1 << 12);
    assert_eq!(tiny, 2f64.powi(-531) * 2f64.powi(-531));
    let [answer, home] = [tiny, 1.0].map(|size| {
        let surfaces = the_seeds_three_at(size);
        meet_of_three_surfaces(
            [&surfaces[0], &surfaces[1], &surfaces[2]],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
    });
    let home = home.expect("the lattice triple is answered");
    let MeetOfThree::At(home_points) = &home else {
        panic!("a line through a ball's middle meets it twice, not {home:?}");
    };
    let answer = answer.expect("the same triple among the subnormals is answered");
    let MeetOfThree::At(points) = &answer else {
        panic!("a line through a ball's middle meets it twice, not {answer:?}");
    };
    assert_eq!(points.len(), home_points.len());

    let size = 3.0 * tiny;
    let mut widest = 0.0f64;
    for (point, at_home) in points.iter().zip(home_points) {
        for (side, home_side) in point.enclosure().iter().zip(at_home.enclosure()) {
            let moved_up = |end: f64| end * 2f64.powi(531) * 2f64.powi(531);
            assert!(
                moved_up(side.lo()) <= home_side.lo() && home_side.hi() <= moved_up(side.hi()),
                "{side:?} does not hold {home_side:?} moved down 2^-1062"
            );
            assert!(
                side.hi() <= side.lo().next_up().next_up(),
                "{side:?} is wider than two subnormal steps"
            );
            widest = widest.max(side.width() / size);
        }
    }
    println!("among the subnormals: widest box {widest:e} of the size");
    assert!(
        widest > 1e-5,
        "a box among the subnormals is a subnormal step wide, {widest:e} of the shape"
    );
}

/// The triple the proptest seed `e0a2a205` shrank to, every number multiplied by
/// `size`: the planes `y = 2` and `z = 0` and a ball of radius three about
/// `(0, 1, 0)`.
fn the_seeds_three_at(size: f64) -> [Surface; 3] {
    let flat = |through: [f64; 3], normal: [f64; 3]| {
        Surface::Plane(
            Plane::from_point_and_normal(Point::from(through), Vector::from(normal))
                .expect("an axis direction"),
        )
    };
    [
        flat([0.0, 2.0 * size, 0.0], [0.0, 1.0, 0.0]),
        flat([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        Surface::Sphere(
            Sphere::new(Point::new(0.0, size, 0.0), 3.0 * size).expect("a radius above zero"),
        ),
    ]
}

/// **§31.7's criterion 7.** Three surfaces of which fewer than two are flat are
/// refused by name, and the refusal is a gap: the points are there and this is
/// the demonstration.
///
/// A pipe of radius three down `z`, a ball of radius five at the origin and a
/// plane. The pipe and the ball meet where `x² + y² = 9` and `x² + y² + z² =
/// 25`, so at `z = ±4` — and the point `(3, 0, 4)` is exactly on both, in whole
/// numbers, with nothing rounded anywhere. Add a third curved surface through it
/// and the corner still exists and is still exact; what changes is that no
/// discriminant this milestone writes can reach it.
#[test]
fn three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved() {
    let corner = Point::new(3.0, 0.0, 4.0);
    let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
    let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
    let second_ball =
        Surface::Sphere(Sphere::new(Point::new(3.0, 0.0, 0.0), 4.0).expect("a radius"));
    let cap = Surface::Plane(
        Plane::from_point_and_normal(corner, Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal"),
    );
    let wall = Surface::Plane(
        Plane::from_point_and_normal(corner, Vector::new(0.0, 1.0, 0.0))
            .expect("the y axis is a unit normal"),
    );

    // The corner is exactly on all five, with no rounding anywhere.
    for surface in [&pipe, &ball, &second_ball, &cap, &wall] {
        assert_eq!(
            which_side_of(surface, PredicatePoint::from(corner.coordinates())),
            Decision::Zero,
            "{corner:?} is exactly on {surface:?}"
        );
    }

    // One plane and two quadrics, and three quadrics. Both refuse by name. The
    // wall runs along the axis the ball and the pipe share and cuts the pipe in
    // two lines and the ball in a circle.
    for triple in [[&wall, &ball, &pipe], [&second_ball, &ball, &pipe]] {
        assert_eq!(
            meet_of_three_surfaces(triple, &Policy::certified(), &mut ProvenanceLog::new()),
            Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
        );
    }

    // The cap is square to that axis and cuts both in one circle of radius
    // three, so it is not the demonstration: two circles about one centre are
    // answered.
    assert_eq!(
        meet_of_three_surfaces(
            [&cap, &ball, &pipe],
            &Policy::certified(),
            &mut ProvenanceLog::new()
        ),
        Ok(MeetOfThree::MoreThanAPoint)
    );

    // Two planes and one of them is answered, so the refusal is about how many
    // curved surfaces there are and not about this corner.
    assert_eq!(
        meet_of_three_surfaces(
            [&cap, &wall, &pipe],
            &Policy::certified(),
            &mut ProvenanceLog::new()
        )
        .map(|answer| middles_of(&answer)),
        Ok(vec![[-3.0, 0.0, 4.0], [3.0, 0.0, 4.0]])
    );
}

/// The answer is about which three surfaces they are, and not about the order
/// they were handed over in.
///
/// §31.3 needs this: three surfaces can meet at more than one point, so a corner
/// is named by the three surfaces *and* by which of their meets it is, and that
/// count is only well defined if every caller counts the same way. All six
/// orders, bit for bit.
#[test]
fn the_same_three_surfaces_meet_at_the_same_points_whichever_order_they_are_asked_in() {
    let mut runner = a_runner(256);
    let outcome = runner.run(&a_turned_corner(), |corner| {
        let surfaces = corner.surfaces();
        let asked = |order: [usize; 3]| {
            meet_of_three_surfaces(
                [
                    &surfaces[order[0]],
                    &surfaces[order[1]],
                    &surfaces[order[2]],
                ],
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
        };
        let first = asked([0, 1, 2]);
        for order in [[0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
            prop_assert_eq!(
                asked(order),
                first.clone(),
                "asking in the order {:?} gave a different answer",
                order
            );
        }
        Ok(())
    });
    if let Err(why) = outcome {
        panic!("{why}");
    }
}

/// What one row measured.
struct Measured {
    drawn: usize,
    refused: usize,
    corners: usize,
    certified_zero: usize,
    worst_box: f64,
}

/// Draw a family of triples, meet each one, and check every point that comes
/// back.
fn meet_every_triple(strategy: impl Strategy<Value = Corner>, what: &str) -> Measured {
    let mut runner = a_runner(CASES);

    let drawn = std::cell::Cell::new(0usize);
    let refused = std::cell::Cell::new(0usize);
    let corners = std::cell::Cell::new(0usize);
    let certified_zero = std::cell::Cell::new(0usize);
    let worst_box = std::cell::Cell::new(0.0f64);

    let outcome = runner.run(&strategy, |corner| {
        drawn.set(drawn.get() + 1);
        let surfaces = corner.surfaces();
        let size = corner.how_big();
        let answer = meet_of_three_surfaces(
            [&surfaces[0], &surfaces[1], &surfaces[2]],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let MeetOfThree::At(points) = (match answer {
            Ok(answer) => answer,
            Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat) => {
                unreachable!("every triple here has two flat cuts in it")
            }
            Err(_) => {
                refused.set(refused.get() + 1);
                return Ok(());
            }
        }) else {
            return Ok(());
        };

        for point in &points {
            corners.set(corners.get() + 1);
            // **The largest number involved, and the corner's own coordinates
            // are among them.** Two planes a hair off parallel meet a long way
            // from either of them, and charging the box that lands out there
            // against the size of the shapes that named it would be measuring a
            // rounding of `1e16` against a `3` (§19, and
            // `Surface::ROUND_TRIP_ROUNDOFF`'s wording).
            let scale = the_largest_number_involved(size, *point);
            worst_box.set(worst_box.get().max(point.uncertainty() / scale));
            if on_all_three_and_clear_of_each_moved(&surfaces, size, *point)? {
                certified_zero.set(certified_zero.get() + 1);
            }
        }
        Ok(())
    });
    if let Err(why) = outcome {
        panic!("{why}");
    }

    let measured = Measured {
        drawn: drawn.get(),
        refused: refused.get(),
        corners: corners.get(),
        certified_zero: certified_zero.get(),
        worst_box: worst_box.get(),
    };
    println!(
        "{what}: {} triples, {} corners, {} of them certified Zero on all three, \
         {} refused ({:.2}%), widest box {:e} of the size",
        measured.drawn,
        measured.corners,
        measured.certified_zero,
        measured.refused,
        100.0 * measured.refused as f64 / measured.drawn as f64,
        measured.worst_box,
    );
    measured
}

/// Both halves of the oracle for one point, and whether it came out certified
/// on all three surfaces.
///
/// **Asked with the point and the three surfaces slid together to about size
/// one**, by a power of two, which is the same question at another size. That
/// is not the oracle being eased. Two planes and a ball `2⁻⁵²⁸` across that
/// touch at `(0, 2⁻⁵²⁸, 0)` meet in a box of no width at all, and the ball moved
/// a millionth of its size clear of it is `2·2⁻⁵²⁸·2⁻⁵⁴⁸` away in `‖p − c‖² − R²`,
/// under the smallest subnormal. No box is tight enough for the side question
/// to be decided there, and at size one this one is.
fn on_all_three_and_clear_of_each_moved(
    surfaces: &[Surface; 3],
    size: f64,
    point: PredicatePoint,
) -> Result<bool, TestCaseError> {
    let scale = the_largest_number_involved(size, point);
    let up = -(scale.log2().floor() as i32);
    let point = slid_outward(point, up);
    let mut on_all_three = true;
    for surface in surfaces {
        let surface = slid(surface, up);
        match which_side_of(&surface, point) {
            Decision::Zero => (),
            Decision::Undecidable { .. } => on_all_three = false,
            decided => prop_assert!(
                false,
                "a point where three surfaces meet was certified {:?} off {:?}: {:?}",
                decided,
                surface,
                point
            ),
        }
        prop_assert!(
            which_side_of(&moved_clear(&surface, slid_exactly(scale, up)), point).is_certified(),
            "the same box against {:?} moved comfortably clear was not decided: {:?}",
            surface,
            point
        );
    }
    Ok(on_all_three)
}

/// A box moved by `2^exponent`, each end exactly where it stays a normal number
/// and one step further out where it does not.
fn slid_outward(point: PredicatePoint, exponent: i32) -> PredicatePoint {
    let outward = |end: f64, step: fn(f64) -> f64| {
        let moved = end * 2f64.powi(exponent);
        if end == 0.0 || moved.is_normal() {
            moved
        } else {
            step(moved)
        }
    };
    PredicatePoint::Constructed(point.enclosure().map(|side| {
        Interval::new(
            outward(side.lo(), f64::next_down),
            outward(side.hi(), f64::next_up),
        )
        .expect("both ends moved the same way")
    }))
}

/// The same surface with space scaled by `2^exponent`: every length multiplied
/// and every direction, normal and slope left as its bits are.
fn slid(surface: &Surface, exponent: i32) -> Surface {
    let by = |value: f64| slid_exactly(value, exponent);
    let place = |at: Point| Point::from(at.coordinates().map(by));
    match surface {
        Surface::Plane(flat) => {
            Surface::Plane(flat.moved_to(by(flat.offset())).expect("a finite offset"))
        }
        Surface::Sphere(ball) => {
            Surface::Sphere(Sphere::new(place(ball.centre()), by(ball.radius())).expect("a radius"))
        }
        Surface::Cylinder(pipe) => Surface::Cylinder(
            Cylinder::new(
                pipe.axis()
                    .moved_to(place(pipe.axis().origin()))
                    .expect("a finite origin"),
                by(pipe.radius()),
            )
            .expect("a radius"),
        ),
        Surface::Cone(cone) => Surface::Cone(
            Cone::new(
                cone.axis()
                    .moved_to(place(cone.apex()))
                    .expect("a finite apex"),
                cone.slope(),
            )
            .expect("the same slope"),
        ),
        Surface::Torus(ring) => Surface::Torus(
            Torus::new(
                ring.axis()
                    .moved_to(place(ring.centre()))
                    .expect("a finite centre"),
                by(ring.ring_radius()),
                by(ring.tube_radius()),
            )
            .expect("the same two radii"),
        ),
        other => panic!("nothing here can slide {other:?}"),
    }
}

/// `value * 2^exponent`, which has to be exactly that.
fn slid_exactly(value: f64, exponent: i32) -> f64 {
    let moved = value * 2f64.powi(exponent);
    assert!(
        value == 0.0 || moved.is_normal(),
        "{value:e} does not slide by 2^{exponent} exactly"
    );
    moved
}

/// The same surface moved comfortably clear of where it was.
///
/// A plane pushed along its own normal, a ball and a pipe grown. A point that
/// was on the original is then provably inside the moved one, and the sign that
/// says so is what stops the first half of the oracle passing for a box a mile
/// wide.
fn moved_clear(surface: &Surface, size: f64) -> Surface {
    let gap = size * COMFORTABLY_CLEAR;
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

/// The largest magnitude any of this was worked out from: the numbers the three
/// surfaces are placed and sized with, and the corner's own coordinates.
fn the_largest_number_involved(size: f64, point: PredicatePoint) -> f64 {
    point
        .enclosure()
        .iter()
        .map(|side| side.lo().abs().max(side.hi().abs()))
        .fold(size, f64::max)
        .max(f64::MIN_POSITIVE)
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

/// The middle of each box, for a test that knows the answer is a whole number.
fn middles_of(answer: &MeetOfThree) -> Vec<[f64; 3]> {
    let MeetOfThree::At(points) = answer else {
        panic!("expected points, got {answer:?}");
    };
    points
        .iter()
        .map(|point| point.enclosure().map(|side| (side.lo() + side.hi()) / 2.0))
        .collect()
}

/// Both ends of each side of a box, as they are stored.
fn both_ends_of(point: &PredicatePoint) -> [[f64; 2]; 3] {
    point.enclosure().map(|side| [side.lo(), side.hi()])
}

fn a_runner(cases: u32) -> TestRunner {
    TestRunner::new_with_rng(
        Config {
            cases,
            source_file: Some(file!()),
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
                "proptest-regressions",
            ))),
            ..Config::default()
        },
        TestRng::deterministic_rng(RngAlgorithm::ChaCha),
    )
}
