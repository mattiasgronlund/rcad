//! A wholly flat solid's measurement owns up to its own rounding.
//!
//! `docs/decisions.md` §25.4 said a polyhedron is "summed exactly" and all
//! three of its errors are exactly `0.0`. The sums were `f64` sums, and an
//! `f64` cannot even hold the answer in general: six times a volume is a sum
//! of products of `f64`, and a sixth of that is not a dyadic fraction. So a
//! turned cube, whose corners the turn rounded, measured `8.000000000000002`
//! with a published error of nothing.
//!
//! **The oracle here is the written polyhedron itself**, worked out exactly in
//! this file: every triangle [`rcad_topo::FaceView::for_each_triangle`] hands
//! out, taken from the world origin rather than from a corner, in
//! [`rcad_predicates::expansion`] arithmetic that throws no bit away. Nothing
//! here reads a number the kernel worked out and compares it with itself.
//!
//! The check is that the truth lies inside `value ± error`. A published error
//! of `0.0` then asks the value to be the truth on the nose, which is what a
//! cube on whole numbers still gets.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::expansion::{compress, expansion_sum, scale_expansion, two_diff, two_sum};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{FaceView, Model, Rule, SolidKey};

/// The two cubes `docs/queue.md` Q81 fused: side two, `b` moved out by
/// `(2, 0.5, 0.25)` so the two touch face to face, and both turned `0.7` about
/// the axis through `(1, 0, 0)` along `(1, 2, 3)`. `a` less `b` is `a` and
/// `b` less `a` is `b`; each came back measuring `8.000000000000002` with an
/// error of nothing.
#[test]
fn a_turned_cube_less_the_cube_it_touches_owns_up_to_its_rounding() {
    let (model, answers) = two_turned_cubes_each_less_the_other();
    for (what, solid) in answers {
        let measured = model
            .solid(solid)
            .expect("the fuse made the key")
            .measure(
                any_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a turned cube measures");
        let exact = six_times_the_volume(&model, solid);
        println!(
            "{what}: {} ± {:e}, exactly about {}",
            measured.volume(),
            measured.volume_error(),
            roughly(&exact) / 6.0
        );
        assert!(
            the_volume_is_inside(measured.volume(), measured.volume_error(), &exact),
            "{what} measures {} ± {:e}, and the polyhedron it is encloses {} exactly",
            measured.volume(),
            measured.volume_error(),
            roughly(&exact) / 6.0
        );
    }
}

/// A cube of side two turned a radian and put a million units out, which
/// `measuring_one_face.rs` found has a face measured `1.25e-10` off `4` with
/// an error of nothing. The placement rounded its corners, so its faces are no
/// longer quite squares of side two; this checks each face against the face it
/// really holds, and the solid against the polyhedron it really is.
#[test]
fn a_cube_turned_and_far_away_owns_up_to_what_each_face_rounds_to() {
    let mut model = a_model();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    model
        .place_solid(cube, &turned_and_a_million_units_out())
        .expect("this model made the key");
    let solid = model.solid(cube).expect("this model made the key");

    let (mut skin_at_least, mut skin_at_most) = (Vec::new(), Vec::new());
    for face in solid.faces() {
        let measured = solid
            .measure_one_face(
                face.key(),
                any_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a face of a turned cube measures");
        let exact = four_times_the_area_squared(face);
        println!(
            "a face: {} ± {:e}, exactly about {}",
            measured.area(),
            measured.area_error(),
            roughly(&exact).sqrt() / 2.0
        );
        assert!(
            the_area_is_inside(measured.area(), measured.area_error(), &exact),
            "a face measures {} ± {:e}, and the face it is has an area of {} exactly",
            measured.area(),
            measured.area_error(),
            roughly(&exact).sqrt() / 2.0
        );
        let (low, high) = the_area_between(&exact);
        skin_at_least = plus(&skin_at_least, &low);
        skin_at_most = plus(&skin_at_most, &high);
    }

    let measured = solid
        .measure(
            any_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a turned cube measures");
    let exact = six_times_the_volume(&model, cube);
    println!(
        "the cube: {} ± {:e}, exactly about {}; skin {} ± {:e}, exactly about {}",
        measured.volume(),
        measured.volume_error(),
        roughly(&exact) / 6.0,
        measured.surface_area(),
        measured.area_error(),
        roughly(&skin_at_least)
    );
    assert!(
        the_volume_is_inside(measured.volume(), measured.volume_error(), &exact),
        "the cube measures {} ± {:e}, and the polyhedron it is encloses {} exactly",
        measured.volume(),
        measured.volume_error(),
        roughly(&exact) / 6.0
    );
    let skin = measured.surface_area();
    let error = measured.area_error();
    assert!(
        at_least_zero(&plus(
            &skin_at_least,
            &negated(&exactly(two_diff(skin, error)))
        )) && at_least_zero(&plus(
            &exactly(two_sum(skin, error)),
            &negated(&skin_at_most)
        )),
        "the cube's skin measures {skin} ± {error:e}, and the faces it holds add up to {} \
         give or take {:e}",
        roughly(&skin_at_least),
        roughly(&plus(&skin_at_most, &negated(&skin_at_least)))
    );
}

/// Where nothing rounds, nothing is owed: a cube on whole numbers is measured
/// on the nose, and its three errors stay exactly `0.0`. This is what says the
/// fix charges the rounding the sums did and not a forward bound on what they
/// might have done.
#[test]
fn a_cube_on_whole_numbers_still_owes_nothing() {
    let mut model = a_model();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let solid = model.solid(cube).expect("this model made the key");
    let measured = solid
        .measure(
            any_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a cube measures");
    assert_eq!(
        [
            measured.volume(),
            measured.volume_error(),
            measured.surface_area(),
            measured.area_error()
        ],
        [8.0, 0.0, 24.0, 0.0]
    );
    for face in solid.faces() {
        let measured = solid
            .measure_one_face(
                face.key(),
                any_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a face of a cube measures");
        assert_eq!([measured.area(), measured.area_error()], [4.0, 0.0]);
    }
}

fn two_turned_cubes_each_less_the_other() -> (Model, Vec<(&'static str, SolidKey)>) {
    let policy = Policy::certified();
    let mut model = a_model();
    let a = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let b = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let turn = Isometry::rotation(
        &Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length"),
        0.7,
    )
    .expect("a rotation");
    model
        .place_solid(a, &turn)
        .expect("this model made the key");
    model
        .place_solid(
            b,
            &turn.compose(&Isometry::translation(Vector::new(2.0, 0.5, 0.25)).expect("a shift")),
        )
        .expect("this model made the key");

    let mut answers = Vec::new();
    for (what, left, right) in [("a less b", a, b), ("b less a", b, a)] {
        let fused = model
            .fuse(
                &[left],
                &[right],
                Rule::Difference,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} fuses: {why}"));
        let [only] = fused.solids() else {
            panic!(
                "{what} is one solid, and came back {}",
                fused.solids().len()
            );
        };
        answers.push((what, *only));
    }
    (model, answers)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A polyhedron reads no deflection; this one is only here because the
/// signature asks for one.
fn any_deflection() -> Deflection {
    Deflection::of(0.1).expect("a length")
}

fn turned_and_a_million_units_out() -> Isometry {
    let axis = Axis::new(Point::new(0.5, -1.5, 2.0), Vector::new(1.0, 2.0, 3.0))
        .expect("a direction with length");
    Isometry::translation(Vector::new(1e6, -1e6, 1e6))
        .expect("a shift")
        .compose(&Isometry::rotation(&axis, 1.0).expect("a rotation"))
}

/// Six times the volume the solid's triangles enclose, exactly: the sum over
/// every triangle of `a · (b × c)`, hung from the world origin.
fn six_times_the_volume(model: &Model, solid: SolidKey) -> Vec<f64> {
    let mut total = Vec::new();
    for face in model.solid(solid).expect("a key the model made").faces() {
        face.for_each_triangle(|[a, b, c]| {
            let [a, b, c] = [a, b, c].map(|corner| corner.coordinates());
            for (first, second, third) in [(0, 1, 2), (1, 2, 0), (2, 0, 1)] {
                let across = plus(
                    &times(&[b[second]], &[c[third]]),
                    &negated(&times(&[b[third]], &[c[second]])),
                );
                total = plus(&total, &times(&[a[first]], &across));
            }
        });
    }
    total
}

/// Four times the square of one face's area, exactly: the face's vector area
/// is half the sum of `a × b + b × c + c × a` over its triangles, whatever
/// corner they fan from, and its area is that vector's length.
fn four_times_the_area_squared(face: FaceView<'_>) -> Vec<f64> {
    let mut twice_the_vector_area: [Vec<f64>; 3] = Default::default();
    face.for_each_triangle(|[a, b, c]| {
        let [a, b, c] = [a, b, c].map(|corner| corner.coordinates());
        for (from, to) in [(a, b), (b, c), (c, a)] {
            for (axis, (second, third)) in [(1, 2), (2, 0), (0, 1)].into_iter().enumerate() {
                let across = plus(
                    &times(&[from[second]], &[to[third]]),
                    &negated(&times(&[from[third]], &[to[second]])),
                );
                twice_the_vector_area[axis] = plus(&twice_the_vector_area[axis], &across);
            }
        }
    });
    twice_the_vector_area
        .iter()
        .fold(Vec::new(), |total, component| {
            plus(&total, &times(component, component))
        })
}

/// Whether `value ± error` holds a sixth of `six_volumes`.
fn the_volume_is_inside(value: f64, error: f64, six_volumes: &[f64]) -> bool {
    let low = times(&exactly(two_diff(value, error)), &[6.0]);
    let high = times(&exactly(two_sum(value, error)), &[6.0]);
    at_least_zero(&plus(six_volumes, &negated(&low)))
        && at_least_zero(&plus(&high, &negated(six_volumes)))
}

/// Whether `value ± error` holds half the square root of `four_areas_squared`.
fn the_area_is_inside(value: f64, error: f64, four_areas_squared: &[f64]) -> bool {
    let low = exactly(two_diff(value, error));
    let high = exactly(two_sum(value, error));
    let low_enough = !at_least_zero(&low)
        || at_least_zero(&plus(
            four_areas_squared,
            &negated(&times(&times(&low, &low), &[4.0])),
        ));
    low_enough
        && at_least_zero(&plus(
            &times(&times(&high, &high), &[4.0]),
            &negated(four_areas_squared),
        ))
}

/// Two numbers either side of half the square root of `four_areas_squared`,
/// each checked against it exactly.
///
/// **Each is two `f64` and not one**, because the solid's published error is
/// a few roundings of its skin and a bracket one rounding wide per face would
/// be wider than that over six faces. One Newton step from the `f64` root puts
/// the second part within a rounding of its own, some thirty orders further
/// down, and the loops only nudge it until the exact check agrees.
fn the_area_between(four_areas_squared: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let squared = times(four_areas_squared, &[0.25]);
    let root = roughly(&squared).sqrt();
    let short_by = plus(&squared, &negated(&times(&[root], &[root])));
    let nudge = roughly(&short_by) / (2.0 * root);
    let square_of = |side: &[f64]| times(side, side);

    let mut low = nudge;
    while !at_least_zero(&plus(
        &squared,
        &negated(&square_of(&plus(&[low], &[root]))),
    )) {
        low = low.next_down();
    }
    let mut high = nudge;
    while !at_least_zero(&plus(
        &square_of(&plus(&[high], &[root])),
        &negated(&squared),
    )) {
        high = high.next_up();
    }
    (plus(&[low], &[root]), plus(&[high], &[root]))
}

fn exactly((rounded, lost): (f64, f64)) -> Vec<f64> {
    plus(&[lost], &[rounded])
}

fn times(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut total = Vec::new();
    let mut piece = vec![0.0; 2 * left.len()];
    for &factor in right {
        let used = scale_expansion(left, factor, &mut piece);
        total = plus(&total, &piece[..used]);
    }
    total
}

fn plus(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut sum = vec![0.0; left.len() + right.len()];
    let used = expansion_sum(left, right, &mut sum);
    let mut packed = vec![0.0; used];
    let kept = compress(&sum[..used], &mut packed);
    packed.truncate(kept);
    assert!(
        packed.iter().all(|entry| entry.is_finite()),
        "nothing here is big enough to overflow"
    );
    packed
}

fn negated(expansion: &[f64]) -> Vec<f64> {
    expansion.iter().map(|entry| -entry).collect()
}

/// The largest entry settles the sign, because the entries never overlap.
fn at_least_zero(expansion: &[f64]) -> bool {
    expansion.last().is_none_or(|&largest| largest >= 0.0)
}

fn roughly(expansion: &[f64]) -> f64 {
    expansion.iter().sum()
}
