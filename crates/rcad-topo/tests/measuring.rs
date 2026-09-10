//! Volume, area and centroid, and the fact that moving a solid does not change
//! the first two (roadmap §11.2 says start here, and it costs nothing).
//!
//! Two halves, answering different questions. The first is about a
//! **polyhedron**, whose measurement is exact and has to stay bit for bit what
//! M0 handed back — `a_flat_solids_measurement_is_bit_for_bit_what_it_was` is
//! the whole of `docs/decisions.md` §23.7 in one test. The second is about a bar
//! and a ball, whose measurement is a sum over a mesh, and there the questions
//! are whether it closes in on the right answer from the side the geometry says
//! it must, and whether the published error bounds hold.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Measurement, Model, ProfileBuilder, SolidKey};

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
        .expect("this model made the key")
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures");

    let turn = a_third_of_a_turn_about_a_slanted_axis();
    model
        .place_solid(cube, &turn)
        .expect("this model made the key");
    let after = model
        .solid(cube)
        .expect("this model made the key")
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
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

    model
        .place_solid(cube, &a_third_of_a_turn_about_a_slanted_axis())
        .expect("this model made the key");

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
        .expect("this model made the key")
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures");

    model
        .place_solid(solid, &a_third_of_a_turn_about_a_slanted_axis())
        .expect("this model made the key");
    let after = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
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
    step_by_step
        .place_solid(first, &shift)
        .expect("this model made the key");
    step_by_step
        .place_solid(first, &turn)
        .expect("this model made the key");

    let (mut all_at_once, second) = a_cube_of_side_two();
    all_at_once
        .place_solid(second, &turn.compose(&shift))
        .expect("this model made the key");

    let stepped = step_by_step
        .solid(first)
        .expect("this model made the key")
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures");
    let together = all_at_once
        .solid(second)
        .expect("this model made the key")
        .measure(
            a_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures");
    assert_eq!(stepped, together);
}

#[test]
fn the_middle_of_a_placed_cube_moves_with_it() {
    let (mut model, cube) = a_cube_of_side_two();
    let shift = Isometry::translation(Vector::new(100.0, 0.0, 0.0)).expect("a finite shift");

    model
        .place_solid(cube, &shift)
        .expect("this model made the key");

    assert_eq!(
        model
            .solid(cube)
            .expect("this model made the key")
            .measure(
                a_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new()
            )
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
            .expect("this model made the key")
            .measure(
                a_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new()
            )
            .expect("a solid this size measures")
            .volume(),
        1.0
    );
    assert_eq!(
        model
            .solid(large)
            .expect("this model made the key")
            .measure(
                a_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new()
            )
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
            .expect("this model made the key")
            .measure(
                a_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
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
            .expect("this model made the key")
            .measure(
                a_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
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

/// The deflection the measurements above are asked for.
///
/// Every solid they measure is a polyhedron, and a polyhedron ignores it: its
/// own fan is the answer rather than a picture of one (`docs/decisions.md`
/// §25.4).
fn a_deflection() -> Deflection {
    Deflection::of(0.1).expect("a tenth of a millimetre is a length")
}

/// A shape, how to build it, and the five numbers M0 measured it as: volume,
/// area, and the three coordinates of the centroid, as bits.
type FrozenShape = (&'static str, fn() -> (Model, SolidKey), [u64; 5]);

/// **The test this whole change is judged by** (`docs/decisions.md` §23.7).
///
/// `measure` shipped in M0: exact, bit for bit repeatable, benchmarked, and read
/// by every metamorphic property in the fuse suite — inclusion–exclusion,
/// idempotence, absorption. Rebuilding it on an approximate mesh puts all of
/// that downstream of new arithmetic, and §23.7 named that as M2's schedule risk
/// before the work started.
///
/// **These bit patterns were read off the kernel before `measure` grew a
/// deflection.** If one of them moves by an ulp, a polyhedron has stopped taking
/// the exact path and the risk has arrived. There is no tolerance here on
/// purpose: "close enough" is exactly what this test exists not to accept.
///
/// The three error terms are exactly zero for the same reason, and the answer
/// does not move when the deflection does — which is what says the deflection is
/// ignored rather than merely small.
#[test]
fn a_flat_solids_measurement_is_bit_for_bit_what_it_was() {
    let shapes: [FrozenShape; 6] = [
        (
            "a cube of side two",
            a_cube_of_side_two,
            [
                4620693217682128896,
                4627448617123184640,
                4607182418800017408,
                4607182418800017408,
                4607182418800017408,
            ],
        ),
        (
            "a cube of side three, turned a third of a turn",
            a_turned_cube_of_side_three,
            [
                4628293042053316612,
                4632796641680687106,
                13603122674472583168,
                13826050856027422716,
                4617315517961601024,
            ],
        ),
        (
            "a plate with a hole",
            a_plate_with_a_hole,
            [
                4643774165772402688,
                4644618590702534656,
                4617315517961601024,
                4617315517961601024,
                4609434218613702656,
            ],
        ),
        (
            "a plate with a hole, turned",
            a_turned_plate_with_a_hole,
            [
                4643774165772402688,
                4644618590702534656,
                13605374474286268416,
                4613937818241073152,
                4620974692658839552,
            ],
        ),
        (
            "a notched extrusion",
            a_notched_extrusion,
            [
                4625759767262920704,
                4631952216750555136,
                4610184818551597739,
                4610184818551597739,
                4604930618986332160,
            ],
        ),
        (
            "a cube a long way out",
            a_cube_a_long_way_out,
            [
                4620693217682128896,
                4627448617123184640,
                4696837155274620928,
                13913128586354622464,
                4679718643993411584,
            ],
        ),
    ];

    for (what, build, in_m0) in shapes {
        let (model, solid) = build();
        // Two deflections five orders apart, because a polyhedron reads neither.
        for how_fine in [1e-6, 1e3] {
            let mut log = ProvenanceLog::new();
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(how_fine).expect("a length"),
                    &Policy::certified(),
                    &mut log,
                )
                .unwrap_or_else(|why| panic!("{what} measures: {why}"));
            let middle = measured.centroid().coordinates();
            assert_eq!(
                [
                    measured.volume().to_bits(),
                    measured.surface_area().to_bits(),
                    middle[0].to_bits(),
                    middle[1].to_bits(),
                    middle[2].to_bits(),
                ],
                in_m0,
                "{what} at a deflection of {how_fine} no longer measures what M0 \
                 measured, which is the risk §23.7 named"
            );
            assert_eq!(measured.volume_error(), 0.0, "{what}: a cubed unit");
            assert_eq!(measured.area_error(), 0.0, "{what}: a squared one");
            assert_eq!(measured.centroid_wobble(), 0.0, "{what}: and a length");
            assert!(
                log.is_empty(),
                "{what}: a polyhedron's measurement approximates nothing, got {:?}",
                log.records()
            );
        }
    }
}

fn a_turned_cube_of_side_three() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = model
        .add_cube(3.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side three");
    model
        .place_solid(cube, &a_third_of_a_turn_about_a_slanted_axis())
        .expect("this model made the key");
    (model, cube)
}

fn a_plate_with_a_hole() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with a hole");
    let ring = model
        .add_extrusion(&plate, 3.0, &Isometry::identity())
        .expect("a plate three thick");
    (model, ring)
}

fn a_turned_plate_with_a_hole() -> (Model, SolidKey) {
    let (mut model, ring) = a_plate_with_a_hole();
    model
        .place_solid(ring, &a_third_of_a_turn_about_a_slanted_axis())
        .expect("this model made the key");
    (model, ring)
}

fn a_notched_extrusion() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let notched = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 2.0],
        [2.0, 2.0],
        [2.0, 4.0],
        [0.0, 4.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("an L");
    let ell = model
        .add_extrusion(&notched, 1.5, &Isometry::identity())
        .expect("an L one and a half thick");
    (model, ell)
}

fn a_cube_a_long_way_out() -> (Model, SolidKey) {
    let (mut model, cube) = a_cube_of_side_two();
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(1e6, -3.5e5, 7.25e4)).expect("a shift"),
        )
        .expect("this model made the key");
    (model, cube)
}

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 5.0;

/// How coarse and how fine the curved measurements below are asked for.
///
/// Each step is a third of the one before, so a quantity falling with the
/// deflection falls by about a third each time, and one falling with its square
/// by a ninth. That is what makes "closing in" a claim a test can make.
const HOW_FINE: [f64; 6] = [1.0, 0.3, 0.1, 0.03, 0.01, 0.003];

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

fn measured(model: &Model, solid: SolidKey, how_fine: f64) -> Measurement {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(how_fine).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("measuring at a deflection of {how_fine}: {why}"))
}

/// A bar's mesh is drawn through points on the bar, so it is **inside** it, and
/// the volume and area it reports have to come up at `πr²h` and `2πr(r + h)`
/// from below.
///
/// From below is the half that could not be got by making the tolerance wide
/// enough. A mesh that strayed outside would pass a two-sided check at every
/// deflection here and still be wrong about what a mesh of a solid *is*.
#[test]
fn a_bars_meshed_volume_and_area_close_in_on_the_exact_ones_from_below() {
    let (model, bar) = a_round_bar();
    let exact_volume = core::f64::consts::PI * RADIUS * RADIUS * HEIGHT;
    let exact_area = 2.0 * core::f64::consts::PI * RADIUS * (RADIUS + HEIGHT);

    let mut closest = (f64::MAX, f64::MAX);
    for how_fine in HOW_FINE {
        let measured = measured(&model, bar, how_fine);
        let (missing_volume, missing_area) = (
            exact_volume - measured.volume(),
            exact_area - measured.surface_area(),
        );
        assert!(
            missing_volume > 0.0 && missing_area > 0.0,
            "a mesh inside the bar cannot report more than the bar has, and at a \
             deflection of {how_fine} it reported {} and {}",
            measured.volume(),
            measured.surface_area()
        );
        assert!(
            missing_volume < closest.0 && missing_area < closest.1,
            "a finer mesh has to get closer, and {how_fine} left {missing_volume} \
             and {missing_area} against {closest:?}"
        );
        closest = (missing_volume, missing_area);

        // The middle of a bar on the z axis is on the axis, whichever mesh it
        // was worked out over: the triangles round it are symmetric.
        let middle = measured.centroid().coordinates();
        assert!(
            middle[0].abs() < 1e-12 && middle[1].abs() < 1e-12,
            "the middle of a bar on the z axis is on it, and this one is at {middle:?}"
        );
        assert!((middle[2] - HEIGHT / 2.0).abs() < 1e-12, "half way up");
    }
    assert!(
        closest.0 < exact_volume / 1000.0 && closest.1 < exact_area / 1000.0,
        "the finest mesh here is still {closest:?} out"
    );
}

/// The same for a ball, against `4πr³/3` and `4πr²`.
#[test]
fn a_balls_meshed_volume_and_area_close_in_on_the_exact_ones_from_below() {
    let (model, ball) = a_ball();
    let exact_volume = 4.0 / 3.0 * core::f64::consts::PI * RADIUS * RADIUS * RADIUS;
    let exact_area = 4.0 * core::f64::consts::PI * RADIUS * RADIUS;

    let mut closest = (f64::MAX, f64::MAX);
    for how_fine in HOW_FINE {
        let measured = measured(&model, ball, how_fine);
        let (missing_volume, missing_area) = (
            exact_volume - measured.volume(),
            exact_area - measured.surface_area(),
        );
        assert!(
            missing_volume > 0.0 && missing_area > 0.0,
            "an inscribed mesh cannot report more ball than there is, and at a \
             deflection of {how_fine} it reported {} and {}",
            measured.volume(),
            measured.surface_area()
        );
        assert!(
            missing_volume < closest.0 && missing_area < closest.1,
            "a finer mesh has to get closer, and {how_fine} left {missing_volume} \
             and {missing_area} against {closest:?}"
        );
        closest = (missing_volume, missing_area);
    }
    assert!(
        closest.0 < exact_volume / 1000.0 && closest.1 < exact_area / 1000.0,
        "the finest mesh here is still {closest:?} out"
    );
}

/// §15.6: a published bound needs a test that could fail.
///
/// `volume_error`, `area_error` and `centroid_wobble` are each derived beside the
/// expression they bound, and a derivation is only a paragraph until something
/// executes it. Here the true answers are known in closed form, so what the mesh
/// really misses can be subtracted and compared against what the kernel says it
/// might.
///
/// Both shapes, and the bar is the one that matters: its **flat** caps are
/// meshed as polygons inscribed in their rims, so a bound that only charged the
/// curved faces would be short — by more than the bound has to spare once the
/// bar is short and wide.
#[test]
fn the_published_error_bounds_cover_what_a_mesh_of_a_bar_or_a_ball_misses() {
    let pi = core::f64::consts::PI;
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let mut shapes = Vec::new();
    // A tall bar, a short flat one where the caps carry most of the skin, a
    // ball, and one of each somewhere else entirely.
    for (radius, height) in [(3.0, 5.0), (4.0, 0.5), (0.25, 2.0)] {
        for placement in [
            Isometry::identity(),
            a_third_of_a_turn_about_a_slanted_axis(),
        ] {
            let bar = model
                .add_round_bar(radius, height, &placement)
                .expect("a bar");
            shapes.push((
                bar,
                pi * radius * radius * height,
                2.0 * pi * radius * (radius + height),
            ));
        }
        let ball = model
            .add_ball(radius, &Isometry::identity())
            .expect("a ball");
        shapes.push((
            ball,
            4.0 / 3.0 * pi * radius * radius * radius,
            4.0 * pi * radius * radius,
        ));
    }

    for (solid, exact_volume, exact_area) in shapes {
        for how_fine in HOW_FINE {
            let measured = measured(&model, solid, how_fine);
            assert!(
                exact_volume - measured.volume() <= measured.volume_error(),
                "the mesh at {how_fine} is {} short of {exact_volume} and only \
                 owned up to {}",
                exact_volume - measured.volume(),
                measured.volume_error()
            );
            assert!(
                exact_area - measured.surface_area() <= measured.area_error(),
                "the mesh at {how_fine} is {} short of {exact_area} and only owned \
                 up to {}",
                exact_area - measured.surface_area(),
                measured.area_error()
            );
        }
    }
}

/// The bounds are bounds and not the answer: they shrink with the mesh.
///
/// A bound that stayed put while the error fell would still pass the test above
/// and would be worth nothing.
#[test]
fn the_published_error_bounds_shrink_as_the_mesh_gets_finer() {
    for (model, solid) in [a_round_bar(), a_ball()] {
        let mut previous = (f64::MAX, f64::MAX, f64::MAX);
        for how_fine in HOW_FINE {
            let measured = measured(&model, solid, how_fine);
            let now = (
                measured.volume_error(),
                measured.area_error(),
                measured.centroid_wobble(),
            );
            assert!(
                now.0 < previous.0 && now.1 < previous.1 && now.2 < previous.2,
                "at a deflection of {how_fine} the bounds are {now:?} and were \
                 {previous:?}"
            );
            previous = now;
        }
        assert!(
            previous.0 > 0.0 && previous.1 > 0.0 && previous.2 > 0.0,
            "a curved solid is never measured exactly, and this one says it was"
        );
    }
}

/// The demonstration behind `measure`'s "the policy does not refuse this", and
/// it is a demonstration rather than a paragraph on purpose (`CLAUDE.md`).
///
/// `Policy::max_wobble` is 64 roundings at the working size — `1.4e-12` for a
/// 100 mm model. A mesh's centroid wobble falls roughly in step with the
/// deflection, so meeting that budget would need a deflection around `5e-13`,
/// and a ball asked for one is refused — that is under the floor a boundary
/// point can be placed to at all. So `wobble_is_acceptable(centroid_wobble)`
/// would be `false` for every curved solid at every deflection that can be met,
/// and judging on it would have replaced §25.5's blanket refusal with another.
#[test]
fn judging_a_meshed_centroid_against_the_policy_would_refuse_every_ball() {
    let budget = Policy::tolerant(100.0).max_wobble();
    let (model, ball) = a_ball();

    let how_fine = 0.1;
    let wobble = measured(&model, ball, how_fine).centroid_wobble();
    assert!(
        wobble > budget * 1e9,
        "a ball meshed to {how_fine} wobbles {wobble}, and the budget is {budget}"
    );

    let would_need = how_fine * budget / wobble;
    let refusal = model.solid(ball).expect("this model made the key").mesh(
        Deflection::of(would_need).expect("a length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(refusal, Err(Error::DeflectionBelowWhatAMeshCanReach { .. })),
        "a deflection of {would_need} is what the budget asks for, and the kernel \
         answered {refusal:?}"
    );
}

/// §19 for a measurement: a solid that has been moved and turned is the solid it
/// was, and its measured volume says so.
///
/// Not bit for bit, and it cannot be — a curved face is cut up in the frame it
/// is stored in and the points are carried into place afterwards, so a turned
/// bar's mesh is the turned mesh of a bar and rounds where the placement does.
/// What has to hold is that the two answers agree inside what they each admit to
/// being out by, which is what the error terms are for.
#[test]
fn a_bar_measured_where_it_was_put_agrees_with_one_at_the_origin() {
    let (mut model, bar) = a_round_bar();
    let at_the_origin = measured(&model, bar, 0.01);

    model
        .place_solid(bar, &a_third_of_a_turn_about_a_slanted_axis())
        .expect("this model made the key");
    let moved = measured(&model, bar, 0.01);

    let apart = (moved.volume() - at_the_origin.volume()).abs();
    assert!(
        apart <= at_the_origin.volume_error() + moved.volume_error(),
        "the same bar measured in two places is {apart} apart, and the two \
         answers only admit to {} and {}",
        at_the_origin.volume_error(),
        moved.volume_error()
    );
    let skin = (moved.surface_area() - at_the_origin.surface_area()).abs();
    assert!(
        skin <= at_the_origin.area_error() + moved.area_error(),
        "and {skin} apart on area"
    );
}
