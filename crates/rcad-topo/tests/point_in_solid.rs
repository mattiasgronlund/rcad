//! Inside, outside, on the skin, or honestly not sure.

use rcad_geom::{Isometry, Plane, Point, Scale};
use rcad_predicates::{Cause, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, ProfileBuilder, SolidKey};

fn a_cube_of_side_two() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    (model, cube)
}

fn classify(model: &Model, solid: SolidKey, point: Point) -> Containment {
    let mut log = ProvenanceLog::new();
    let answer = model
        .solid(solid)
        .classify_point(point, &Policy::certified(), &mut log);
    assert!(
        log.is_empty(),
        "an exact classification should not have to approximate anything"
    );
    answer
}

#[test]
fn the_middle_of_a_cube_is_inside_it() {
    let (model, cube) = a_cube_of_side_two();

    assert_eq!(
        classify(&model, cube, Point::new(1.0, 1.0, 1.0)),
        Containment::Inside
    );
}

#[test]
fn a_point_well_clear_of_a_cube_is_outside_it() {
    let (model, cube) = a_cube_of_side_two();

    for point in [
        Point::new(-1.0, 1.0, 1.0),
        Point::new(1.0, 5.0, 1.0),
        Point::new(1.0, 1.0, 100.0),
        Point::new(-3.0, -3.0, -3.0),
    ] {
        assert_eq!(classify(&model, cube, point), Containment::Outside);
    }
}

#[test]
fn a_point_just_inside_a_face_is_inside_and_one_just_outside_is_not() {
    let (model, cube) = a_cube_of_side_two();

    assert_eq!(
        classify(&model, cube, Point::new(1.0, 1.0, 1.9)),
        Containment::Inside
    );
    assert_eq!(
        classify(&model, cube, Point::new(1.0, 1.0, 2.1)),
        Containment::Outside
    );
}

#[test]
fn the_middle_of_a_cube_face_is_on_its_skin() {
    let (model, cube) = a_cube_of_side_two();

    for point in [
        Point::new(1.0, 1.0, 0.0),
        Point::new(1.0, 1.0, 2.0),
        Point::new(0.0, 1.0, 1.0),
        Point::new(2.0, 1.0, 1.0),
        Point::new(1.0, 0.0, 1.0),
        Point::new(1.0, 2.0, 1.0),
    ] {
        assert_eq!(classify(&model, cube, point), Containment::OnBoundary);
    }
}

#[test]
fn the_middle_of_a_cube_edge_is_on_its_skin() {
    let (model, cube) = a_cube_of_side_two();

    assert_eq!(
        classify(&model, cube, Point::new(1.0, 0.0, 0.0)),
        Containment::OnBoundary
    );
    assert_eq!(
        classify(&model, cube, Point::new(0.0, 2.0, 1.0)),
        Containment::OnBoundary
    );
}

#[test]
fn a_corner_of_a_cube_is_on_its_skin() {
    let (model, cube) = a_cube_of_side_two();

    for corner in [
        Point::new(0.0, 0.0, 0.0),
        Point::new(2.0, 0.0, 0.0),
        Point::new(2.0, 2.0, 2.0),
        Point::new(0.0, 2.0, 0.0),
    ] {
        assert_eq!(classify(&model, cube, corner), Containment::OnBoundary);
    }
}

#[test]
fn a_point_in_the_hole_of_a_plate_is_outside_the_plate() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with a hole");
    let solid = model
        .add_extrusion(&plate, 3.0, &Isometry::identity())
        .expect("a plate three thick");

    assert_eq!(
        classify(&model, solid, Point::new(5.0, 5.0, 1.5)),
        Containment::Outside
    );
    assert_eq!(
        classify(&model, solid, Point::new(2.0, 2.0, 1.5)),
        Containment::Inside
    );
    assert_eq!(
        classify(&model, solid, Point::new(4.0, 5.0, 1.5)),
        Containment::OnBoundary
    );
}

#[test]
fn a_point_in_the_notch_of_an_l_shape_is_outside_it() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ell = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 2.0],
        [2.0, 2.0],
        [2.0, 4.0],
        [0.0, 4.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("an L");
    let solid = model
        .add_extrusion(&ell, 2.0, &Isometry::identity())
        .expect("an L two thick");

    assert_eq!(
        classify(&model, solid, Point::new(3.0, 3.0, 1.0)),
        Containment::Outside
    );
    assert_eq!(
        classify(&model, solid, Point::new(1.0, 3.0, 1.0)),
        Containment::Inside
    );
    assert_eq!(
        classify(&model, solid, Point::new(3.0, 1.0, 1.0)),
        Containment::Inside
    );
}

#[test]
fn classifying_a_grid_gives_the_same_answers_whatever_order_it_is_walked_in() {
    let (model, cube) = a_cube_of_side_two();
    let grid = sample_grid();

    let forwards: Vec<_> = grid
        .iter()
        .map(|&point| classify(&model, cube, point))
        .collect();
    let mut backwards: Vec<_> = grid
        .iter()
        .rev()
        .map(|&point| classify(&model, cube, point))
        .collect();
    backwards.reverse();

    assert_eq!(forwards, backwards);
}

#[test]
fn classifying_a_grid_on_several_threads_agrees_with_doing_it_one_at_a_time() {
    let (model, cube) = a_cube_of_side_two();
    let grid = sample_grid();

    let one_at_a_time: Vec<_> = grid
        .iter()
        .map(|&point| classify(&model, cube, point))
        .collect();

    let shared = &model;
    let in_parallel: Vec<Containment> = std::thread::scope(|threads| {
        let handles: Vec<_> = grid
            .chunks(16)
            .map(|chunk| {
                threads.spawn(move || {
                    let mut log = ProvenanceLog::new();
                    chunk
                        .iter()
                        .map(|&point| {
                            shared
                                .solid(cube)
                                .classify_point(point, &Policy::certified(), &mut log)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("a thread finished"))
            .collect()
    });

    assert_eq!(one_at_a_time, in_parallel);
}

/// A grid over and around the cube, deliberately landing on its faces, edges
/// and corners as well as in the middle of things.
fn sample_grid() -> Vec<Point> {
    let steps: Vec<f64> = (0..13).map(|step| -1.0 + f64::from(step) * 0.5).collect();
    let mut points = Vec::new();
    for &x in &steps {
        for &y in &steps {
            for &z in &steps {
                points.push(Point::new(x, y, z));
            }
        }
    }
    points
}

#[test]
fn every_point_of_the_grid_gets_a_straight_answer() {
    let (model, cube) = a_cube_of_side_two();

    for point in sample_grid() {
        let answer = classify(&model, cube, point);
        match answer {
            Containment::Inside | Containment::OnBoundary | Containment::Outside => {}
            Containment::Undecidable { enclosure } => {
                panic!("could not place {point:?} (enclosure {enclosure})")
            }
            other => panic!("unhandled answer: {other:?}"),
        }
    }
}

#[test]
fn a_point_built_from_three_planes_deep_inside_a_cube_is_inside_it() {
    let (model, cube) = a_cube_of_side_two();

    // Three planes square to the axes at halves and quarters, so Cramer's rule
    // divides by one and the box the meet comes back in has no width at all.
    // It is still a constructed point, so exact arithmetic is off the table and
    // every one of these decisions is proved by intervals alone.
    let middle = meet_of_three_planes(
        [1.0, 0.0, 0.0, -0.75],
        [0.0, 1.0, 0.0, -1.25],
        [0.0, 0.0, 1.0, -0.5],
    );
    assert_eq!(middle.uncertainty(), 0.0, "these divisions are all by one");

    assert_eq!(
        classify_uncertain(&model, cube, middle),
        Containment::Inside
    );
}

#[test]
fn a_point_built_from_three_planes_well_clear_of_a_cube_is_outside_it() {
    let (model, cube) = a_cube_of_side_two();

    let far_away = meet_of_three_planes(
        [1.0, 0.0, 0.0, -5.0],
        [0.0, 1.0, 0.0, 3.0],
        [0.0, 0.0, 1.0, -7.0],
    );

    assert_eq!(
        classify_uncertain(&model, cube, far_away),
        Containment::Outside
    );
}

/// The one above never fires a ray — the whole box is past the side of the
/// solid's bounding box, which settles it. This one is inside the bounding box
/// and outside the material, so the rays have to do the work.
#[test]
fn a_point_built_from_three_planes_in_the_notch_of_an_l_shape_is_outside_it() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ell = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 2.0],
        [2.0, 2.0],
        [2.0, 4.0],
        [0.0, 4.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("an L");
    let solid = model
        .add_extrusion(&ell, 2.0, &Isometry::identity())
        .expect("an L two thick");

    let in_the_notch = meet_of_three_planes(
        [1.0, 0.0, 0.0, -3.0],
        [0.0, 1.0, 0.0, -3.0],
        [0.0, 0.0, 1.0, -1.0],
    );

    assert_eq!(
        classify_uncertain(&model, solid, in_the_notch),
        Containment::Outside
    );
}

/// The answer decisions.md §12.5 says cannot be reached today.
///
/// The point is the meet of the cube's own top face plane `z = 2` with two
/// upright planes crossing above the middle of that face, so it is genuinely,
/// exactly on the skin. Cramer's rule still divides, and the quotient of two
/// ranges is a range: the box comes back straddling `z = 2`. There is no
/// ill-conditioning here and no nudge by a nanometre — the planes are square to
/// each other and the meet is exactly representable. The width is the division
/// alone (decisions.md §17.5), and it is enough that no precision can prove
/// which side of the top face the point is on.
#[test]
fn a_point_whose_box_straddles_a_face_of_a_cube_cannot_be_placed() {
    let (model, cube) = a_cube_of_side_two();

    let on_the_top_face = a_meet_straddling_the_top_of_the_cube();
    assert!(
        on_the_top_face.uncertainty() > 0.0,
        "the meet of three planes is a box, and this is the case where it matters"
    );

    let mut log = ProvenanceLog::new();
    let answer =
        model
            .solid(cube)
            .classify_uncertain_point(on_the_top_face, &Policy::certified(), &mut log);

    let Containment::Undecidable { enclosure } = answer else {
        panic!("a box straddling the top face cannot be placed, but got {answer:?}");
    };
    assert!(
        enclosure > 0.0,
        "the enclosure is what straddled zero, so it has width"
    );
    assert!(
        !log.is_empty(),
        "a decision that could not be certified is recorded"
    );
}

#[test]
fn a_tolerant_policy_puts_that_same_point_on_the_skin_and_says_so_in_the_log() {
    let (model, cube) = a_cube_of_side_two();
    let policy = Policy::tolerant(100.0);

    let mut log = ProvenanceLog::new();
    let answer = model.solid(cube).classify_uncertain_point(
        a_meet_straddling_the_top_of_the_cube(),
        &policy,
        &mut log,
    );

    assert_eq!(answer, Containment::OnBoundary);

    let resolved: Vec<_> = log
        .records()
        .iter()
        .filter_map(|record| record.approximation())
        .collect();
    assert!(
        !resolved.is_empty(),
        "resolving an undecided predicate leaves a record carrying what it accepted"
    );
    for approximation in resolved {
        assert_eq!(approximation.cause(), Cause::UndecidableResolved);
        assert!(policy.wobble_is_acceptable(approximation.magnitude()));
    }
    assert!(policy.wobble_is_acceptable(log.worst_wobble(100.0)));
}

/// The cube's top face plane, and two upright planes crossing over the middle
/// of it. Their meet is exactly on the face and the box around it is not.
fn a_meet_straddling_the_top_of_the_cube() -> PredicatePoint {
    // Upright through (0.5, 1.25), which is inside the top face and off its
    // triangulation diagonal, so the rays meet the face rather than an edge.
    meet_of_three_planes(
        [0.0, 0.0, 1.0, -2.0],
        [1.0, 2.0, 0.0, -3.0],
        [2.0, -1.0, 0.0, 0.25],
    )
}

/// Every exact point of the grid gets the same answer whichever way in it is
/// asked, which is decisions.md §1.7's third criterion — classification reused
/// and not reimplemented — made into something that can fail.
#[test]
fn the_exact_way_in_and_the_general_one_give_the_same_answers() {
    let (model, cube) = a_cube_of_side_two();

    for point in sample_grid() {
        let mut through_the_wrapper = ProvenanceLog::new();
        let wrapped =
            model
                .solid(cube)
                .classify_point(point, &Policy::certified(), &mut through_the_wrapper);

        let mut through_the_general_one = ProvenanceLog::new();
        let general = model.solid(cube).classify_uncertain_point(
            PredicatePoint::from(point.coordinates()),
            &Policy::certified(),
            &mut through_the_general_one,
        );

        assert_eq!(wrapped, general, "at {point:?}");
        assert_eq!(
            through_the_wrapper.len(),
            through_the_general_one.len(),
            "at {point:?}"
        );
    }
}

/// A coordinate that cannot be bounded is not a coordinate the classifier will
/// place, and that is a change from M0.
///
/// The short cut that used to answer this compared the coordinate against the
/// bounding box, and infinity is past every side of it. What that comparison
/// reads now is the point's enclosure, and there is no enclosure around
/// infinity — so nothing is proved there and every ray comes back undecided.
/// Deciding from a rigorous enclosure or not at all is the trade, and this is
/// the one input where it costs an answer.
#[test]
fn a_point_with_a_coordinate_that_cannot_be_bounded_cannot_be_placed() {
    let (model, cube) = a_cube_of_side_two();

    for point in [
        Point::new(f64::INFINITY, 1.0, 1.0),
        Point::new(f64::NEG_INFINITY, 1.0, 1.0),
        Point::new(f64::NAN, 1.0, 1.0),
    ] {
        let mut log = ProvenanceLog::new();
        let answer = model
            .solid(cube)
            .classify_point(point, &Policy::certified(), &mut log);
        assert!(
            matches!(answer, Containment::Undecidable { .. }),
            "{point:?} was placed as {answer:?}"
        );
    }
}

fn classify_uncertain(model: &Model, solid: SolidKey, point: PredicatePoint) -> Containment {
    let mut log = ProvenanceLog::new();
    let answer = model
        .solid(solid)
        .classify_uncertain_point(point, &Policy::certified(), &mut log);
    assert!(
        log.is_empty(),
        "a meet that lands on an f64 is still decided by intervals, with nothing approximated"
    );
    answer
}

fn meet_of_three_planes(first: [f64; 4], second: [f64; 4], third: [f64; 4]) -> PredicatePoint {
    let plane = |coefficients: [f64; 4]| {
        Plane::from_coefficients(coefficients).expect("four numbers describing a plane")
    };
    Plane::meet_of_three(&plane(first), &plane(second), &plane(third))
        .expect("three planes meeting at one point")
}
