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
use rcad_topo::{FaceView, Model, ProfileBuilder, Rule, SolidKey};

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

/// §15.6 for the centroid wobble, of a flat solid and of each of its faces:
/// where the solid's triangles and each face's fan really balance, worked out
/// exactly here, is within the published wobble of the measured centroid.
///
/// The shapes are ones whose middle cannot come out on the nose — turned, a
/// million units out, sides that are not whole numbers, a hole — and the check
/// is on the distance itself, squared and compared exactly, so a wobble of
/// `0.0` asks for the exact middle and fails on every one of them that is off.
/// At least one must be, or the test proves nothing.
///
/// **The exact middles are `docs/decisions.md` §338.5's, reached by another
/// route.** The solid's is every triangle's tetrahedron from the world origin
/// weighed by its signed volume, where the kernel hangs them from a corner; a
/// face's is every triangle weighed by its shadow on the fan's exact vector
/// area, with the weights summed one by one and each triangle's cross product
/// taken as `a × b + b × c + c × a` from the world origin, where the kernel
/// takes `|N|²` and the sides from the fan's apex.
///
/// It prints how close the closest one came, as a share of its wobble, and
/// floors that too, so a wobble grown loose fails here as well.
#[test]
fn the_middle_of_a_flat_solid_and_of_each_face_is_within_its_wobble() {
    let builders: [fn() -> NamedSolids; 5] = [
        two_turned_cubes_each_less_the_other,
        || a_cube_turned_and_far_away("a cube turned and a million units out"),
        || an_uneven_l("an uneven L", &Isometry::identity()),
        || {
            an_uneven_l(
                "an uneven L turned and a million units out",
                &turned_and_a_million_units_out(),
            )
        },
        a_plate_with_a_hole_turned_and_far_away,
    ];
    let (mut closest_solid, mut closest_face): (f64, f64) = (0.0, 0.0);
    let mut off_at_all = 0;
    for build in builders {
        let (model, solids) = build();
        for (what, key) in solids {
            let solid = model.solid(key).expect("a key the model made");
            let measured = solid
                .measure(
                    any_deflection(),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what} measures: {why}"));
            let (moment, weight) = where_the_solid_balances(&model, key);
            let share = the_share_of_its_wobble(
                what,
                measured.centroid(),
                measured.centroid_wobble(),
                &moment,
                &weight,
            );
            println!(
                "{what}: middle within {:e}, {share:.3} of it used",
                measured.centroid_wobble()
            );
            closest_solid = closest_solid.max(share);
            off_at_all += usize::from(share > 0.0);

            for face in solid.faces() {
                let measured = solid
                    .measure_one_face(
                        face.key(),
                        any_deflection(),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| panic!("a face of {what} measures: {why}"));
                let (moment, weight) = where_the_face_balances(face);
                let share = the_share_of_its_wobble(
                    what,
                    measured.centroid(),
                    measured.centroid_wobble(),
                    &moment,
                    &weight,
                );
                closest_face = closest_face.max(share);
                off_at_all += usize::from(share > 0.0);
            }
        }
    }
    println!(
        "the closest a solid's middle came to its wobble: {closest_solid:.3} of it; a face's: \
         {closest_face:.3}"
    );
    assert!(
        off_at_all > 0,
        "no middle here came out off, so nothing was checked"
    );
    assert!(
        closest_solid > 0.99 && closest_face > 0.99,
        "the closest a solid's middle came to its wobble was {closest_solid} of it, and a face's \
         {closest_face}"
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
            measured.area_error(),
            measured.centroid_wobble()
        ],
        [8.0, 0.0, 24.0, 0.0, 0.0]
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
        assert_eq!(
            [
                measured.area(),
                measured.area_error(),
                measured.centroid_wobble()
            ],
            [4.0, 0.0, 0.0]
        );
    }
}

/// `docs/queue.md` Q402 and Q549: a solid whose corners mix coordinates near
/// nothing with ones near two, so products in the exact sums behind its errors
/// fall below the smallest subnormal. **Every published error still holds the
/// truth.**
///
/// The solid is a cube of side two moved by `speck` along all three axes,
/// through the public API, and `Model::validate` accepts every one of them:
/// its dynamic-range check compares the largest coordinate with the shortest
/// edge, which is two here whatever the speck. So §338.3's "`Model::validate`'s
/// dynamic-range check refuses such a solid" does not hold for this one, and
/// no `Deserialize` is needed to reach it.
///
/// `2 + speck` rounds to `2`, so the cube's corners are `speck` and `2` along
/// each axis, and the truth is worked out by hand: a volume of `(2 − speck)³`,
/// faces of `(2 − speck)²`, and every middle `1 + speck / 2` along each axis
/// the face or solid spans. The kernel hangs its sums from the corner at
/// `speck`, so each offset to a far corner is `2 − speck`, kept as `2` and a
/// lost part of `−speck`, and in the solid's units `u = speck / 2` is what
/// multiplies. The products of lost parts that fall below `2^-1074`, and the
/// first speck where each does:
///
/// - `2^-220`: a face's moment, `u⁵ = 2^-1105`.
/// - `2^-270`: the solid's moment and a face's weights, `u⁴ = 2^-1084`.
/// - `2^-400` and `1e-300`, which is §338.3's own mix: the volume, `u³`.
///
/// **They are dropped and nothing is published wrong**, because each is below
/// `2^-1075` in the solid's units, and the published distance is at least the
/// lost parts' own first-order share, rounded up by more than that. What does
/// break is the next test.
#[test]
fn a_cube_moved_by_a_speck_owns_up_to_what_the_speck_moves() {
    for speck in [
        2f64.powi(-100),
        2f64.powi(-220),
        2f64.powi(-270),
        2f64.powi(-400),
        1e-300,
        2f64.powi(-1000),
        2f64.powi(-1060),
        f64::from_bits(2),
    ] {
        let (model, cube) = a_cube_of_side_two_moved_by(speck);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("a cube moved by {speck:e} validates: {why}"));
        let solid = model.solid(cube).expect("this model made the key");
        let measured = solid
            .measure(
                any_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a moved cube measures");
        assert_eq!(
            [
                measured.volume(),
                measured.surface_area(),
                measured.centroid().coordinates()[0],
                measured.centroid().coordinates()[1],
                measured.centroid().coordinates()[2],
            ],
            [8.0, 24.0, 1.0, 1.0, 1.0],
            "a cube moved by {speck:e}"
        );
        assert!(
            at_least(measured.volume_error(), 12.0, speck),
            "a cube moved by {speck:e} publishes a volume of 8 ± {:e}, and 8 − (2 − speck)³ is \
             about {:e}",
            measured.volume_error(),
            12.0 * speck
        );
        assert!(
            at_least(measured.area_error(), 24.0, speck),
            "a cube moved by {speck:e} publishes a skin of 24 ± {:e}, and 24 − 6·(2 − speck)² is \
             about {:e}",
            measured.area_error(),
            24.0 * speck
        );
        assert!(
            covers_a_middle_off_along(measured.centroid_wobble(), speck, 3.0),
            "a cube moved by {speck:e} publishes a middle within {:e}, and it is √3 · {:e} off",
            measured.centroid_wobble(),
            speck / 2.0
        );

        for face in solid.faces() {
            let measured = solid
                .measure_one_face(
                    face.key(),
                    any_deflection(),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a face of a moved cube measures");
            let middle = measured.centroid().coordinates();
            assert_eq!(
                (
                    measured.area(),
                    middle.iter().filter(|&&at| at == 1.0).count(),
                    middle
                        .iter()
                        .filter(|&&at| at == speck || at == 2.0)
                        .count()
                ),
                (4.0, 2, 1),
                "a face of a cube moved by {speck:e} measures {} with its middle at {middle:?}",
                measured.area()
            );
            assert!(
                at_least(measured.area_error(), 4.0, speck),
                "a face of a cube moved by {speck:e} publishes 4 ± {:e}, and 4 − (2 − speck)² is \
                 about {:e}",
                measured.area_error(),
                4.0 * speck
            );
            assert!(
                covers_a_middle_off_along(measured.centroid_wobble(), speck, 2.0),
                "a face of a cube moved by {speck:e} publishes a middle within {:e}, and it is \
                 √2 · {:e} off",
                measured.centroid_wobble(),
                speck / 2.0
            );
        }
    }
}

/// **The limit, pinned: a cube of side two moved by `2^-1074`, the smallest
/// subnormal, publishes all five errors as `0.0`, and every one of its
/// answers is off.** `docs/queue.md` Q402 and Q549, class D.
///
/// It is not a product that fails here but the step before any product: each
/// axis is measured in units of `2^1` for a cube two across, and the lost part
/// of an offset, `−2^-1074`, halves to `2^-1075`, which rounds to zero. So the
/// exact sums are the sums of the cube that was not moved, the rounded ones
/// are too, and the two agree to the bit. One step up, a move of `2^-1073`
/// halves to `2^-1074`, which is there to keep, and the test above holds it.
/// The truth is a volume of
/// `(2 − 2^-1074)³`, below `8` by about `3.5e-323`, and a middle `2^-1075` off
/// along each axis.
///
/// **What puts it outside `docs/decisions.md` §3.5's envelope is `2^1075`**:
/// the cube is two across and carries a move of `2^-1074`, a spread of
/// `2^1075`, about `4e323`, against §3.5's `1e11`. It is `docs/queue.md` Q325's
/// bar moved by `2^-1007`, one step further down. Any shape needs that
/// spread for a lost part to vanish this way, because the units are picked so
/// that the solid's widest offset along the axis is between one and two, and a
/// part vanishes only once it is under `2^-1075` of those. The check in
/// `Model::validate` passes it, as it passes the cube above: it reads edges,
/// and every edge is two long.
#[test]
fn a_cube_moved_by_the_smallest_subnormal_publishes_no_error_and_is_off() {
    let speck = f64::from_bits(1);
    let (model, cube) = a_cube_of_side_two_moved_by(speck);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("validate reads edges, and every edge is two long");
    let solid = model.solid(cube).expect("this model made the key");
    let measured = solid
        .measure(
            any_deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a moved cube measures");
    assert_eq!(
        [
            measured.volume(),
            measured.volume_error(),
            measured.surface_area(),
            measured.area_error(),
            measured.centroid().coordinates()[0],
            measured.centroid_wobble(),
        ],
        [8.0, 0.0, 24.0, 0.0, 1.0, 0.0],
        "the limit moved: a cube moved by 2^-1074 now publishes something other than an \
         unmoved cube's exact answers"
    );
    for face in solid.faces() {
        let measured = solid
            .measure_one_face(
                face.key(),
                any_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a face of a moved cube measures");
        assert_eq!(
            [
                measured.area(),
                measured.area_error(),
                measured.centroid_wobble()
            ],
            [4.0, 0.0, 0.0],
            "the limit moved for a face of a cube moved by 2^-1074"
        );
    }
}

/// A cube of side two, moved by `speck` along each axis.
fn a_cube_of_side_two_moved_by(speck: f64) -> (Model, SolidKey) {
    let mut model = a_model();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(speck, speck, speck)).expect("a shift"),
        )
        .expect("this model made the key");
    (model, cube)
}

/// Whether `error` is at least `by · speck`, checked exactly.
///
/// For a cube moved by `speck` that is enough to hold the truth: `8 −
/// (2 − speck)³` is `12·speck − 6·speck² + speck³`, which is under `12·speck`,
/// and the skin and a face go the same way with `24` and `4`.
fn at_least(error: f64, by: f64, speck: f64) -> bool {
    at_least_zero(&plus(&[error], &negated(&times(&[speck], &[by]))))
}

/// Whether `wobble` reaches a point `speck / 2` off along each of `axes` axes,
/// which is `√axes · speck / 2` away: `(2 · wobble)² ≥ axes · speck²`, checked
/// exactly.
///
/// Both are doubled up to where `speck` is at least one, which is exact, so
/// that their squares cannot fall into the subnormals and make the check round.
fn covers_a_middle_off_along(wobble: f64, speck: f64, axes: f64) -> bool {
    if wobble >= speck {
        return true;
    }
    if wobble < speck / 2.0 {
        return false;
    }
    let (mut wobble, mut speck) = (wobble, speck);
    while speck < 1.0 {
        wobble *= 2.0;
        speck *= 2.0;
    }
    at_least_zero(&plus(
        &times(&times(&[wobble], &[wobble]), &[4.0]),
        &negated(&times(&times(&[speck], &[speck]), &[axes])),
    ))
}

/// A model, and the solids in it each with what to call it.
type NamedSolids = (Model, Vec<(&'static str, SolidKey)>);

fn a_cube_turned_and_far_away(what: &'static str) -> NamedSolids {
    let mut model = a_model();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    model
        .place_solid(cube, &turned_and_a_million_units_out())
        .expect("this model made the key");
    (model, vec![(what, cube)])
}

/// An L whose sides are none of them whole numbers, nor the same, so its
/// middle is a ratio no `f64` holds even where it stands.
fn an_uneven_l(what: &'static str, placement: &Isometry) -> NamedSolids {
    let mut model = a_model();
    let ell = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [4.1, 0.0],
        [4.1, 0.3],
        [0.9, 0.3],
        [0.9, 7.7],
        [0.0, 7.7],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("an L");
    let ell = model
        .add_extrusion(&ell, 2.35, placement)
        .expect("an L 2.35 thick");
    (model, vec![(what, ell)])
}

fn a_plate_with_a_hole_turned_and_far_away() -> NamedSolids {
    let mut model = a_model();
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with a hole");
    let plate = model
        .add_extrusion(&plate, 3.0, &turned_and_a_million_units_out())
        .expect("a plate three thick");
    (
        model,
        vec![("a plate with a hole turned and a million units out", plate)],
    )
}

fn two_turned_cubes_each_less_the_other() -> NamedSolids {
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
        face.for_each_triangle(|corners| {
            total = plus(
                &total,
                &six_times_the_volume_under(corners.map(|corner| corner.coordinates())),
            );
        });
    }
    total
}

/// `a · (b × c)`, exactly: six times the signed volume of the tetrahedron the
/// triangle makes with the world origin.
fn six_times_the_volume_under([a, b, c]: [[f64; 3]; 3]) -> Vec<f64> {
    let mut total = Vec::new();
    for (first, second, third) in [(0, 1, 2), (1, 2, 0), (2, 0, 1)] {
        let across = plus(
            &times(&[b[second]], &[c[third]]),
            &negated(&times(&[b[third]], &[c[second]])),
        );
        total = plus(&total, &times(&[a[first]], &across));
    }
    total
}

/// Four times the square of one face's area, exactly: the face's vector area
/// is half the sum of `a × b + b × c + c × a` over its triangles, whatever
/// corner they fan from, and its area is that vector's length.
fn four_times_the_area_squared(face: FaceView<'_>) -> Vec<f64> {
    let mut twice_the_vector_area: [Vec<f64>; 3] = Default::default();
    face.for_each_triangle(|corners| {
        let turned = twice_the_vector_area_of(corners.map(|corner| corner.coordinates()));
        for (total, part) in twice_the_vector_area.iter_mut().zip(&turned) {
            *total = plus(total, part);
        }
    });
    twice_the_vector_area
        .iter()
        .fold(Vec::new(), |total, component| {
            plus(&total, &times(component, component))
        })
}

/// `a × b + b × c + c × a`, exactly: twice one triangle's vector area, which is
/// `(b − a) × (c − a)` without taking a side.
fn twice_the_vector_area_of([a, b, c]: [[f64; 3]; 3]) -> [Vec<f64>; 3] {
    let mut twice: [Vec<f64>; 3] = Default::default();
    for (from, to) in [(a, b), (b, c), (c, a)] {
        for (axis, (second, third)) in [(1, 2), (2, 0), (0, 1)].into_iter().enumerate() {
            let across = plus(
                &times(&[from[second]], &[to[third]]),
                &negated(&times(&[from[third]], &[to[second]])),
            );
            twice[axis] = plus(&twice[axis], &across);
        }
    }
    twice
}

/// Where the space the solid's triangles enclose balances, exactly, as a
/// moment and a weight whose ratio is the middle along each axis.
///
/// Every triangle makes a tetrahedron with the world origin, which weighs its
/// signed volume and stands at a quarter of the sum of its corners; so the
/// moment is the sum of `a · (b × c)` times `a + b + c`, and the weight four
/// times the sum of `a · (b × c)`.
fn where_the_solid_balances(model: &Model, solid: SolidKey) -> ([Vec<f64>; 3], Vec<f64>) {
    let mut moment: [Vec<f64>; 3] = Default::default();
    let mut weight = Vec::new();
    for face in model.solid(solid).expect("a key the model made").faces() {
        face.for_each_triangle(|corners| {
            let corners = corners.map(|corner| corner.coordinates());
            let six_volume = six_times_the_volume_under(corners);
            weight = plus(&weight, &times(&six_volume, &[4.0]));
            for (axis, total) in moment.iter_mut().enumerate() {
                let summed = plus(
                    &plus(&[corners[0][axis]], &[corners[1][axis]]),
                    &[corners[2][axis]],
                );
                *total = plus(total, &times(&six_volume, &summed));
            }
        });
    }
    (moment, weight)
}

/// Where one face's fan balances exactly under `docs/decisions.md` §338.5's
/// definition, as a moment and a weight whose ratio is the middle along each
/// axis: every triangle weighed by its shadow on the fan's exact vector area,
/// `(a × b + b × c + c × a) · N`, and standing at `(a + b + c) / 3`.
fn where_the_face_balances(face: FaceView<'_>) -> ([Vec<f64>; 3], Vec<f64>) {
    let mut triangles = Vec::new();
    let mut twice: [Vec<f64>; 3] = Default::default();
    face.for_each_triangle(|corners| {
        let corners = corners.map(|corner| corner.coordinates());
        let turned = twice_the_vector_area_of(corners);
        for (total, part) in twice.iter_mut().zip(&turned) {
            *total = plus(total, part);
        }
        triangles.push((corners, turned));
    });

    let mut moment: [Vec<f64>; 3] = Default::default();
    let mut weight = Vec::new();
    for (corners, turned) in triangles {
        let shadow = turned
            .iter()
            .zip(&twice)
            .fold(Vec::new(), |total, (part, of)| {
                plus(&total, &times(part, of))
            });
        weight = plus(&weight, &times(&shadow, &[3.0]));
        for (axis, total) in moment.iter_mut().enumerate() {
            let summed = plus(
                &plus(&[corners[0][axis]], &[corners[1][axis]]),
                &[corners[2][axis]],
            );
            *total = plus(total, &times(&shadow, &summed));
        }
    }
    (moment, weight)
}

/// How far `middle` is from `moment / weight`, as a share of `wobble` — and a
/// panic when it is further than `wobble`, which is checked exactly: the
/// squared distance times `weight²` against `wobble² · weight²`.
fn the_share_of_its_wobble(
    what: &str,
    middle: Point,
    wobble: f64,
    moment: &[Vec<f64>; 3],
    weight: &[f64],
) -> f64 {
    assert!(
        wobble.is_finite(),
        "{what}: nothing is worth asserting against an infinite bound (§47.4)"
    );
    let middle = middle.coordinates();
    let squared = moment
        .iter()
        .zip(middle)
        .fold(Vec::new(), |total, (moment, at)| {
            let off = plus(&times(weight, &[at]), &negated(moment));
            plus(&total, &times(&off, &off))
        });
    let allowed = times(&times(weight, weight), &times(&[wobble], &[wobble]));
    let distance = roughly(&squared).sqrt() / roughly(weight).abs();
    assert!(
        at_least_zero(&plus(&allowed, &negated(&squared))),
        "{what}: a middle measured at {middle:?} is {distance:e} from where the triangles \
         balance, and the kernel owned up to {wobble:e}"
    );
    if distance == 0.0 {
        return 0.0;
    }
    distance / wobble
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
