//! Volume, area and centroid, and the fact that moving a solid does not change
//! the first two (roadmap §11.2 says start here, and it costs nothing).

use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, SolidKey};

fn a_cube_of_side_two() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    (model, cube)
}

fn a_third_of_a_turn_about_a_slanted_axis() -> Isometry {
    let axis = Axis::new(Point::new(0.5, -1.5, 2.0), Vector::new(1.0, 1.0, 1.0))
        .expect("a direction with length");
    Isometry::rotation(&axis, core::f64::consts::TAU / 3.0).expect("a rotation")
}

#[test]
fn a_cube_rotated_by_a_third_of_a_turn_has_the_same_volume_and_area() {
    let (mut model, cube) = a_cube_of_side_two();
    let before = model
        .solid(cube)
        .measure()
        .expect("a solid this size measures");

    let turn = a_third_of_a_turn_about_a_slanted_axis();
    model.place_solid(cube, &turn);
    let after = model
        .solid(cube)
        .measure()
        .expect("a solid this size measures");

    assert!((after.volume() - before.volume()).abs() < 1e-12);
    assert!((after.surface_area() - before.surface_area()).abs() < 1e-12);
    assert!(
        after
            .centroid()
            .distance_to(turn.apply_to_point(before.centroid()))
            < 1e-12
    );
}

#[test]
fn a_rotated_cube_is_still_a_sound_solid() {
    let (mut model, cube) = a_cube_of_side_two();

    model.place_solid(cube, &a_third_of_a_turn_about_a_slanted_axis());

    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rotation cannot break a solid");
}

#[test]
fn moving_a_plate_with_a_hole_leaves_its_volume_alone() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with a hole");
    let solid = model
        .add_extrusion(&plate, 3.0, &Isometry::identity())
        .expect("a plate three thick");
    let before = model
        .solid(solid)
        .measure()
        .expect("a solid this size measures");

    model.place_solid(solid, &a_third_of_a_turn_about_a_slanted_axis());
    let after = model
        .solid(solid)
        .measure()
        .expect("a solid this size measures");

    assert!((after.volume() - before.volume()).abs() < 1e-11);
    assert!((after.surface_area() - before.surface_area()).abs() < 1e-11);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("still sound");
}

#[test]
fn moving_a_solid_twice_is_the_same_as_moving_it_once_with_both_movements() {
    let shift = Isometry::translation(Vector::new(3.0, -7.0, 11.0)).expect("a finite shift");
    let turn = a_third_of_a_turn_about_a_slanted_axis();

    let (mut step_by_step, first) = a_cube_of_side_two();
    step_by_step.place_solid(first, &shift);
    step_by_step.place_solid(first, &turn);

    let (mut all_at_once, second) = a_cube_of_side_two();
    all_at_once.place_solid(second, &turn.compose(&shift));

    let stepped = step_by_step
        .solid(first)
        .measure()
        .expect("a solid this size measures");
    let together = all_at_once
        .solid(second)
        .measure()
        .expect("a solid this size measures");
    assert_eq!(stepped, together);
}

#[test]
fn the_middle_of_a_placed_cube_moves_with_it() {
    let (mut model, cube) = a_cube_of_side_two();
    let shift = Isometry::translation(Vector::new(100.0, 0.0, 0.0)).expect("a finite shift");

    model.place_solid(cube, &shift);

    assert_eq!(
        model
            .solid(cube)
            .measure()
            .expect("a solid this size measures")
            .centroid(),
        Point::new(101.0, 1.0, 1.0)
    );
}

#[test]
fn a_bigger_cube_holds_the_cube_of_its_side() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let small = model
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side one");
    let large = model
        .add_cube(3.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side three");

    assert_eq!(
        model
            .solid(small)
            .measure()
            .expect("a solid this size measures")
            .volume(),
        1.0
    );
    assert_eq!(
        model
            .solid(large)
            .measure()
            .expect("a solid this size measures")
            .volume(),
        27.0
    );
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("two sound cubes");
}

/// A cube `add_cube` accepts is a cube `measure` answers about or refuses in
/// words — never one it panics on.
///
/// `measure` used to assert `six_volumes.is_finite() && six_volumes > 0.0` and
/// die on the first half of it. The justification written above the assert was
/// that a closed solid with its faces pointing outwards cannot enclose
/// non-positive space, "which makes this a broken model rather than bad input".
/// True, and about geometry; the check was about arithmetic. A cube of side
/// `1e300` points its faces outwards perfectly well and simply encloses more
/// than an `f64` holds. Bad input is data, not a bug (`CLAUDE.md`), so it is an
/// `Err`.
#[test]
fn a_cube_too_big_to_measure_is_refused_in_words_rather_than_by_panicking() {
    for side in [1e103, 1e140, 1e200, 1e300] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cube = model
            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("add_cube accepts a side of {side:e}: {why}"));

        let refusal = model
            .solid(cube)
            .measure()
            .expect_err("a cube of side {side:e} encloses more than an f64 holds");
        assert!(
            matches!(refusal, Error::VolumeTooLargeToMeasure),
            "a cube of side {side:e} runs out of room for its volume: {refusal:?}"
        );
    }
}

/// Area and centroid are answered, not quietly handed back as infinity.
///
/// The assert `measure` used to carry watched `six_volumes` and nothing else,
/// and `six_volumes` is the last of the three to overflow. A cube of side
/// `2^256` upwards came back with a real volume, a `surface_area` of infinity
/// and a centroid at infinity on all three axes, and said nothing — the exact
/// corrupt-output case the assert was there to prevent, escaping through the
/// two fields it did not look at. Both are §16.10 *gaps*: a `2^300` cube's
/// area and middle are ordinary `f64` numbers, and only the arithmetic on the
/// way to them overflowed.
#[test]
fn a_cube_whose_area_overflowed_the_old_sum_gets_its_area_and_middle_anyway() {
    for exponent in [255, 256, 257, 300, 340] {
        let side = f64::from_bits(u64::try_from(1023 + exponent).expect("normal") << 52);
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cube = model
            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube this size builds");

        let measured = model
            .solid(cube)
            .measure()
            .unwrap_or_else(|why| panic!("a cube of side 2^{exponent} measures: {why}"));
        assert_eq!(
            measured.surface_area(),
            6.0 * side * side,
            "six faces of side 2^{exponent}"
        );
        assert_eq!(
            measured.centroid(),
            Point::new(side / 2.0, side / 2.0, side / 2.0),
            "the middle of a cube of side 2^{exponent}"
        );
    }
}
