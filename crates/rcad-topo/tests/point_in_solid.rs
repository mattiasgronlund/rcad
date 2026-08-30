//! Inside, outside, on the skin, or honestly not sure.

use rcad_geom::{Isometry, Point, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
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
