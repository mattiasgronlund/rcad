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
//!
//! **A solid with a curved face owes the same rounding**, over the mesh it is
//! measured on, and the last two tests hold one's volume and centroid against
//! that mesh worked out exactly the same way.

use std::cell::Cell;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::expansion::{
    compress, expansion_sum, product_is_exact, scale_expansion, two_diff, two_sum,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{FaceView, Measurement, Mesh, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_pointed_cone;

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
        let exact = six_times_the_volume(&model, solid, 0);
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
        let exact = four_times_the_area_squared(face, 0);
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
    let exact = six_times_the_volume(&model, cube, 0);
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
        || {
            a_plate_with_a_hole(
                "a plate with a hole turned and a million units out",
                &turned_and_a_million_units_out(),
            )
        },
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
            let (moment, weight) = where_the_solid_balances(&model, key, 0);
            let share = the_share_of_its_wobble(
                what,
                measured.centroid(),
                measured.centroid_wobble(),
                (&moment, &weight),
                0,
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
                let (moment, weight) = where_the_face_balances(face, 0);
                let share = the_share_of_its_wobble(
                    what,
                    measured.centroid(),
                    measured.centroid_wobble(),
                    (&moment, &weight),
                    0,
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
    for speck in the_specks() {
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

/// `docs/queue.md` Q573: the same cubes publish a wobble no wider than the
/// speck, for the solid and for each face, however far down the speck goes.
///
/// The solid's middle is `√3 · speck / 2`, about `0.87 · speck`, from where it
/// was measured, and a face's `√2 · speck / 2`, about `0.71 · speck`. The test
/// above holds that each wobble reaches that far, and this one that it reaches
/// no further than `speck`: under `1.16` times the truth for the solid and
/// `1.42` times for a face.
///
/// **What this pins is the scaling in `measure.rs`'s `length_at_most`.** A
/// wobble is the length of three distances, one per axis, and a distance under
/// about `1e-162` squares to below the smallest subnormal. Squared as they
/// were, each of those squares was stepped up to `2^-1074`, so a cube moved by
/// `1e-300` published a wobble of `5.4e-162` for a middle `0.87e-300` off, and
/// so did every smaller speck here. Scaling the three distances by a power of
/// two so the longest is between one and two, and the length back, is exact
/// wherever the square would have dropped.
///
/// **The last speck gets seven spacings of the subnormals, `3.5 · speck`.** At
/// `2^-1073` a spacing is half the speck, and the distance along each axis is
/// one spacing, but it reaches `length_at_most` as four: the division before
/// it works in the solid's units of two, where that distance is half the
/// smallest subnormal, and rounds it up by whole spacings. Three sides of four
/// spacings are `6.93` spacings long, which is seven once rounded up.
#[test]
fn a_cube_moved_by_a_speck_publishes_a_wobble_no_wider_than_the_speck() {
    let mut too_wide = Vec::new();
    for speck in the_specks() {
        let ceiling = speck.max(f64::from_bits(7));
        let (model, cube) = a_cube_of_side_two_moved_by(speck);
        let solid = model.solid(cube).expect("this model made the key");
        let measured = solid
            .measure(
                any_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a moved cube measures");
        let mut wobbles = vec![("the solid", measured.centroid_wobble())];
        for face in solid.faces() {
            let measured = solid
                .measure_one_face(
                    face.key(),
                    any_deflection(),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a face of a moved cube measures");
            wobbles.push(("a face", measured.centroid_wobble()));
        }
        println!(
            "a cube moved by {speck:e}: the solid's middle within {:e}, the widest face's within \
             {:e}",
            measured.centroid_wobble(),
            wobbles[1..]
                .iter()
                .map(|&(_, wobble)| wobble)
                .fold(0.0, f64::max)
        );
        for (what, wobble) in wobbles {
            if wobble > ceiling {
                too_wide.push(format!(
                    "a cube moved by {speck:e}: {what} publishes a middle within {wobble:e}, \
                     wider than {ceiling:e}"
                ));
            }
        }
    }
    assert!(
        too_wide.is_empty(),
        "wobbles wider than their speck:\n{}",
        too_wide.join("\n")
    );
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

/// `docs/queue.md` Q571: solids turned by half and quarter turns carry corner
/// parts far smaller than anything that was asked for, and those reach below
/// the subnormals in the exact sums behind a face's middle. **Every published
/// error still holds the truth**, of the solid and of each face, and each is
/// still as tight as it was.
///
/// A half turn about `x` is kept as the quaternion `(cos(π/2), 1, 0, 0)`, and
/// `cos(π/2)` in `f64` is `c = 6.1e-17`, not zero. Half turns about `x` and `y`
/// in turn multiply those `c`s together wherever every other term is exactly
/// zero: after four the turn is `(−1, 2c³, 0, −2c²)`, which puts the corner
/// `(2, 0, 0)` of a cube at `(2, 8c², −16c⁵)`. So a cube of side two turned
/// `x`, `y`, `x`, `y` is back where it started to within `8c² ≈ 3e-32`, with
/// corner parts down to `16c⁵ ≈ 2^-265`, which is `2^-266` in a face's units.
/// Eight quarter turns, `x x y y x x y y`, do the same with other small numbers
/// and reach `2^-257.7`. A search over every run of up to five of the 24
/// quarter, half and three-quarter turns either way about the three axes,
/// through the origin and through `(1, 1, 1)`, and over 300 000 of the runs of
/// six and of seven, left no corner of a cube of side two smaller than the
/// first.
///
/// **This is inside `docs/decisions.md` §3.5's envelope**: the solids are two,
/// 7.7 and ten across, at the origin, and the small parts are what the turns
/// rounded to rather than a feature anyone drew. `Model::validate` accepts
/// every one.
///
/// **What they reach**, counted once with a tally, not kept, of every product
/// `measure.rs`'s `multiplied` takes that `product_is_exact` refuses: nothing
/// in the volume, the skin, the solid's middle or any face's area; in a face's
/// middle, 13 to 18 products in each of four faces of the cube turned by half
/// turns, 4 to 6 in three faces of the one turned by quarter turns, 18 to 27
/// in four faces of the L and 13 to 23 in four of the plate, and on the half
/// turns some of them fall below the smallest subnormal outright.
///
/// **They are dropped and nothing is published wrong**, because each dropped
/// product loses less than `2^-1075` in the face's units, and the smallest
/// wobble a face publishes here is `6.4e-79`. What would break is a face whose
/// rounded middle landed exactly on the middle the lossy sums stand for, owing
/// `0.0`, while the dropped products moved the true one; on these solids the
/// parts that move a middle are the ones the turns left, and those the sums
/// keep. The truth used `1.000` of every solid's wobble and of the closest
/// face's, and of the volume's error on three of the four, so none of the
/// errors has grown loose either.
///
/// **The truth does not reach the subnormals itself.** Every corner is scaled
/// up by `2^80` before the sums, which is exact, and lifts the smallest product
/// of five of the parts above to `2^-927`; every product the sums take is
/// checked with `product_is_exact` and must pass. Comparing a middle squares
/// those sums, and the squares' tails do fall below the subnormals, so that
/// comparison counts them and asks the room between the squared distance and
/// the squared wobble to be wider than all of them could have lost.
#[test]
fn a_solid_turned_by_half_and_quarter_turns_owns_up_to_what_the_turns_rounded() {
    let solids = [
        turned_by(
            &four_half_turns(),
            a_cube_of_side_two("a cube turned by four half turns"),
        ),
        turned_by(
            &eight_quarter_turns(),
            a_cube_of_side_two("a cube turned by eight quarter turns"),
        ),
        turned_by(
            &four_half_turns(),
            an_uneven_l(
                "an uneven L turned by four half turns",
                &Isometry::identity(),
            ),
        ),
        turned_by(
            &four_half_turns(),
            a_plate_with_a_hole(
                "a plate with a hole turned by four half turns",
                &Isometry::identity(),
            ),
        ),
    ];
    let mut closest = Closest::default();
    for (model, named) in solids {
        for (what, key) in named {
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what} validates: {why}"));
            let solid = model.solid(key).expect("a key the model made");
            let smallest = the_smallest_part_of_a_corner(&model, key);
            assert!(
                smallest < 2f64.powi(-257),
                "{what}: the turns left no corner part below 2^-257; the smallest is {smallest:e}"
            );
            let measured = solid
                .measure(
                    any_deflection(),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what} measures: {why}"));

            let six_volumes = with_nothing_lost(what, || six_times_the_volume(&model, key, UP));
            let (volume, volume_error) = (
                lifted(measured.volume(), 3 * UP),
                lifted(measured.volume_error(), 3 * UP),
            );
            assert!(
                the_volume_is_inside(volume, volume_error, &six_volumes),
                "{what} measures {} ± {:e}, and the polyhedron it is encloses {} exactly",
                measured.volume(),
                measured.volume_error(),
                lifted(roughly(&six_volumes) / 6.0, -3 * UP)
            );
            let off = plus(&six_volumes, &negated(&times(&[volume], &[6.0])));
            let volume_share = share(roughly(&off).abs() / 6.0, volume_error);

            let (moment, weight) =
                with_nothing_lost(what, || where_the_solid_balances(&model, key, UP));
            let middle_share = the_share_of_its_wobble(
                what,
                measured.centroid(),
                measured.centroid_wobble(),
                (&moment, &weight),
                UP,
            );

            let (mut skin_at_least, mut skin_at_most) = (Vec::new(), Vec::new());
            let (mut face_area_share, mut face_middle_share): (f64, f64) = (0.0, 0.0);
            let mut smallest_face_wobble = f64::INFINITY;
            for face in solid.faces() {
                let one = solid
                    .measure_one_face(
                        face.key(),
                        any_deflection(),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| panic!("a face of {what} measures: {why}"));
                let squared = with_nothing_lost(what, || four_times_the_area_squared(face, UP));
                let (area, area_error) =
                    (lifted(one.area(), 2 * UP), lifted(one.area_error(), 2 * UP));
                assert!(
                    the_area_is_inside(area, area_error, &squared),
                    "a face of {what} measures {} ± {:e}, and the face it is has an area of {} \
                     exactly",
                    one.area(),
                    one.area_error(),
                    lifted(roughly(&squared).sqrt() / 2.0, -2 * UP)
                );
                let off = plus(&squared, &negated(&times(&times(&[area], &[area]), &[4.0])));
                let distance =
                    roughly(&off).abs() / (2.0 * (roughly(&squared).sqrt() + 2.0 * area));
                face_area_share = face_area_share.max(share(distance, area_error));
                let (low, high) = the_area_between(&squared);
                skin_at_least = plus(&skin_at_least, &low);
                skin_at_most = plus(&skin_at_most, &high);

                let (moment, weight) =
                    with_nothing_lost(what, || where_the_face_balances(face, UP));
                smallest_face_wobble = smallest_face_wobble.min(one.centroid_wobble());
                face_middle_share = face_middle_share.max(the_share_of_its_wobble(
                    what,
                    one.centroid(),
                    one.centroid_wobble(),
                    (&moment, &weight),
                    UP,
                ));
            }

            let (skin, skin_error) = (
                lifted(measured.surface_area(), 2 * UP),
                lifted(measured.area_error(), 2 * UP),
            );
            assert!(
                at_least_zero(&plus(
                    &skin_at_least,
                    &negated(&exactly(two_diff(skin, skin_error)))
                )) && at_least_zero(&plus(
                    &exactly(two_sum(skin, skin_error)),
                    &negated(&skin_at_most)
                )),
                "{what}'s skin measures {} ± {:e}, and the faces it holds add up to about {}",
                measured.surface_area(),
                measured.area_error(),
                lifted(roughly(&skin_at_least), -2 * UP)
            );
            let halfway = times(&plus(&skin_at_least, &skin_at_most), &[0.5]);
            let skin_share = share(
                roughly(&plus(&halfway, &negated(&[skin]))).abs(),
                skin_error,
            );

            println!(
                "{what}: the smallest corner part is 2^{:.1} and the smallest wobble of a face \
                 {smallest_face_wobble:e}; of its published errors the truth used \
                 {volume_share:.3} of the volume's, {skin_share:.3} of the skin's, \
                 {middle_share:.3} of the middle's, and at most {face_area_share:.3} of a face's \
                 area's and {face_middle_share:.3} of a face's middle's",
                smallest.log2()
            );
            closest.volume = closest.volume.max(volume_share);
            closest.skin = closest.skin.max(skin_share);
            closest.middle = closest.middle.max(middle_share);
            closest.face_area = closest.face_area.max(face_area_share);
            closest.face_middle = closest.face_middle.max(face_middle_share);
        }
    }
    println!("the closest each came, over all four: {closest:.3?}");
    assert!(
        closest.volume > 0.99 && closest.middle > 0.99 && closest.face_middle > 0.99,
        "a published error grew loose on a turned solid: {closest:?}"
    );
}

/// `docs/queue.md` Q1845: **a solid with a curved face owns up to the rounding
/// of the sum over its mesh** as well as to the space its curved faces hide.
///
/// The solid is the block of `a_cone_cap_whose_rim_runs_through_its_apex.rs`
/// turned onto a ruling of the pointed cone of slope one half and height eight,
/// less the cone. It keeps a hair of cone skin along the ruling and nothing
/// else curved, so the space hidden under its mesh of twenty triangles is
/// `5.7e-19`, and the sum over them rounds by far more than that. It measured
/// `1152.0000000000007` with an error of `5.7e-19`, while its mesh, summed
/// exactly, encloses `8.4e-14` less: `7.9e-15` of that is the sum rounding,
/// and the rest a sixth of the sum rounding.
///
/// **The oracle is the mesh the measure is taken over**, summed exactly here
/// from the world origin in the same arithmetic as the flat solids above.
/// [`FaceView::for_each_triangle`] cannot stand in for it, since a curved face
/// fans into its own corners and never into the mesh. The solid's true volume
/// is within the hidden space of the mesh's, so an error that does not reach
/// the mesh's own volume has left out the rounding.
///
/// The hidden space is five orders under the rounding, so the error is nearly
/// all rounding, and a rounding worked out rather than bounded in advance is
/// used nearly whole: the check floors that too.
#[test]
fn a_curved_solid_owns_up_to_the_rounding_of_the_sum_over_its_mesh() {
    let (measured, mesh) = the_block_less_the_cone_measured_and_meshed();
    let exact = six_times_the_volume_of(&mesh);
    let less = roughly(&plus(
        &times(&[measured.volume()], &[6.0]),
        &negated(&exact),
    )) / 6.0;
    let used = share(less.abs(), measured.volume_error());
    println!(
        "the block less the cone: {} ± {:e} over {} triangles, and its mesh encloses {less:e} \
         less exactly; {used:.6} of the error used",
        measured.volume(),
        measured.volume_error(),
        mesh.triangles().len(),
    );
    assert!(
        the_volume_is_inside(measured.volume(), measured.volume_error(), &exact),
        "the block less the cone measures {} ± {:e}, and the mesh it was measured over encloses \
         {less:e} less exactly",
        measured.volume(),
        measured.volume_error(),
    );
    assert!(
        used > 0.99,
        "the block less the cone's mesh used {used} of its published error"
    );
}

/// The same block less the cone, held to where its mesh balances.
///
/// The centroid is two rounded sums over the mesh divided, as a flat solid's
/// is, and its wobble owes their rounding too, on top of the share of the
/// solid's reach the hidden space can move it. That share is `5.1e-21` here,
/// and the centroid came out about `1.2e-15` from where the mesh's triangles
/// balance exactly. The floor holds the wobble to the rounding worked out, not
/// bounded in advance.
#[test]
fn a_curved_solid_owns_up_to_the_rounding_of_where_its_mesh_balances() {
    let (measured, mesh) = the_block_less_the_cone_measured_and_meshed();
    let (moment, weight) = where_the_mesh_balances(&mesh);
    let used = the_share_of_its_wobble(
        "the block less the cone",
        measured.centroid(),
        measured.centroid_wobble(),
        (&moment, &weight),
        0,
    );
    println!(
        "the block less the cone: middle {:?} within {:e}, {used:.6} of it used",
        measured.centroid(),
        measured.centroid_wobble(),
    );
    assert!(
        used > 0.99,
        "the block less the cone's mesh used {used} of its published wobble"
    );
}

/// The largest share of its published error the truth used, over the solids
/// measured, for each of the five errors.
#[derive(Debug, Default)]
struct Closest {
    volume: f64,
    skin: f64,
    middle: f64,
    face_area: f64,
    face_middle: f64,
}

/// How far up every corner is scaled before the truth is summed: far enough
/// that the smallest product of five of the turned solids' corner parts is
/// still exact, and not so far that the square of a face's moment overflows.
const UP: i32 = 80;

/// `value · 2^by`, which is exact for every value this file lifts.
fn lifted(value: f64, by: i32) -> f64 {
    value * 2f64.powi(by)
}

fn share(distance: f64, error: f64) -> f64 {
    if distance == 0.0 {
        return 0.0;
    }
    distance / error
}

/// `sums()`, and a panic if any product it took was one
/// [`product_is_exact`] refuses — so a truth that fell below the subnormals
/// itself is never what a published error is held against.
fn with_nothing_lost<T>(what: &str, sums: impl FnOnce() -> T) -> T {
    products_not_exact_so_far();
    let summed = sums();
    let lost = products_not_exact_so_far();
    assert_eq!(
        lost, 0,
        "{what}: {lost} products of the truth itself fell below the subnormals"
    );
    summed
}

/// Half turns about `x` and `y` through the origin, in turn: `x`, `y`, `x`,
/// `y`. Each is [`Isometry::rotation`] by `π`, so each carries `cos(π/2)`.
fn four_half_turns() -> Vec<Isometry> {
    let [x, y] = turns_about_x_and_y(core::f64::consts::PI);
    vec![x, y, x, y]
}

/// Quarter turns about `x` and `y` through the origin, two of each in turn.
fn eight_quarter_turns() -> Vec<Isometry> {
    let [x, y] = turns_about_x_and_y(core::f64::consts::FRAC_PI_2);
    vec![x, x, y, y, x, x, y, y]
}

fn turns_about_x_and_y(angle: f64) -> [Isometry; 2] {
    [Vector::new(1.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0)].map(|direction| {
        Isometry::rotation(
            &Axis::new(Point::new(0.0, 0.0, 0.0), direction).expect("a direction with length"),
            angle,
        )
        .expect("a rotation")
    })
}

/// Every solid of `named` moved by each of `turns` in order, one
/// [`Model::place_solid`] each.
fn turned_by(turns: &[Isometry], (mut model, named): NamedSolids) -> NamedSolids {
    for (_, key) in &named {
        for turn in turns {
            model
                .place_solid(*key, turn)
                .expect("this model made the key");
        }
    }
    (model, named)
}

fn a_cube_of_side_two(what: &'static str) -> NamedSolids {
    let mut model = a_model();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    (model, vec![(what, cube)])
}

/// The smallest coordinate of any corner of the solid that is not zero.
fn the_smallest_part_of_a_corner(model: &Model, solid: SolidKey) -> f64 {
    let mut smallest = f64::INFINITY;
    for face in model.solid(solid).expect("a key the model made").faces() {
        face.for_each_triangle(|corners| {
            for at in corners.iter().flat_map(|corner| corner.coordinates()) {
                if at != 0.0 {
                    smallest = smallest.min(at.abs());
                }
            }
        });
    }
    smallest
}

/// The moves a cube of side two is checked under: the first speck where each
/// product behind its errors drops below the subnormals, `1e-300`, and on down
/// to `2^-1073`, the last one its errors still hold.
fn the_specks() -> [f64; 8] {
    [
        2f64.powi(-100),
        2f64.powi(-220),
        2f64.powi(-270),
        2f64.powi(-400),
        1e-300,
        2f64.powi(-1000),
        2f64.powi(-1060),
        f64::from_bits(2),
    ]
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

fn a_plate_with_a_hole(what: &'static str, placement: &Isometry) -> NamedSolids {
    let mut model = a_model();
    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with a hole");
    let plate = model
        .add_extrusion(&plate, 3.0, placement)
        .expect("a plate three thick");
    (model, vec![(what, plate)])
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

/// [`a_block_turned_onto_the_ruling_less_the_cone`] measured, and the mesh the
/// measure was taken over.
fn the_block_less_the_cone_measured_and_meshed() -> (Measurement, Mesh) {
    let (model, solid) = a_block_turned_onto_the_ruling_less_the_cone();
    let solid = model.solid(solid).expect("the fuse made the key");
    let deflection = Deflection::of(0.01).expect("a positive length");
    let measured = solid
        .measure(deflection, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the block less the cone measures");
    let mesh = solid
        .mesh(deflection, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the block less the cone meshes");
    (measured, mesh)
}

/// The block `[−6, 6] × [−12, 4] × [0, 6]` turned about `x` until its floor
/// lies on `2y − z = 0`, as near as the turn's rounding lets it, less the
/// pointed cone of slope one half and height eight whose ruling that plane
/// touches.
fn a_block_turned_onto_the_ruling_less_the_cone() -> (Model, SolidKey) {
    let mut model = a_model();
    let cone = a_pointed_cone(&mut model, 0.5, 8.0);
    let rectangle = ProfileBuilder::new(vec![[-6.0, -12.0], [6.0, -12.0], [6.0, 4.0], [-6.0, 4.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    let turn = Isometry::rotation(&Axis::x(), f64::atan2(-2.0, -1.0)).expect("a finite turn");
    let block = model
        .add_extrusion(&rectangle, 6.0, &turn)
        .expect("a rectangle extrudes");
    let fused = model
        .fuse(
            &[block],
            &[cone],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the block less the cone fuses: {why}"));
    let [only] = fused.solids() else {
        panic!(
            "the block less the cone is one solid, and came back {}",
            fused.solids().len()
        );
    };
    let only = *only;
    (model, only)
}

/// Six times the volume a mesh's triangles enclose, exactly: the sum over
/// every triangle of `a · (b × c)`, hung from the world origin.
fn six_times_the_volume_of(mesh: &Mesh) -> Vec<f64> {
    let points = mesh.points();
    mesh.triangles().iter().fold(Vec::new(), |total, triangle| {
        let corners = triangle
            .corners()
            .map(|corner| scaled(points[corner as usize], 0));
        plus(&total, &six_times_the_volume_under(corners))
    })
}

/// Where the space a mesh's triangles enclose balances, exactly, hung from the
/// world origin as [`where_the_solid_balances`] hangs a solid's own triangles.
fn where_the_mesh_balances(mesh: &Mesh) -> ([Vec<f64>; 3], Vec<f64>) {
    let points = mesh.points();
    let mut balance = Default::default();
    for triangle in mesh.triangles() {
        let corners = triangle
            .corners()
            .map(|corner| scaled(points[corner as usize], 0));
        weigh_in(corners, &mut balance);
    }
    balance
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
/// every triangle of `a · (b × c)`, hung from the world origin — with every
/// corner scaled by `2^up`, so in units `2^-3up` of the model's.
fn six_times_the_volume(model: &Model, solid: SolidKey, up: i32) -> Vec<f64> {
    let mut total = Vec::new();
    for face in model.solid(solid).expect("a key the model made").faces() {
        face.for_each_triangle(|corners| {
            total = plus(
                &total,
                &six_times_the_volume_under(corners.map(|corner| scaled(corner, up))),
            );
        });
    }
    total
}

/// `corner`'s coordinates times `2^up`.
fn scaled(corner: Point, up: i32) -> [f64; 3] {
    corner.coordinates().map(|at| lifted(at, up))
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
/// corner they fan from, and its area is that vector's length. Every corner is
/// scaled by `2^up`, so this is in units `2^-4up` of the model's.
fn four_times_the_area_squared(face: FaceView<'_>, up: i32) -> Vec<f64> {
    let mut twice_the_vector_area: [Vec<f64>; 3] = Default::default();
    face.for_each_triangle(|corners| {
        let turned = twice_the_vector_area_of(corners.map(|corner| scaled(corner, up)));
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
/// times the sum of `a · (b × c)`. Every corner is scaled by `2^up`, and the
/// middle with it.
fn where_the_solid_balances(model: &Model, solid: SolidKey, up: i32) -> ([Vec<f64>; 3], Vec<f64>) {
    let mut balance = Default::default();
    for face in model.solid(solid).expect("a key the model made").faces() {
        face.for_each_triangle(|corners| {
            weigh_in(corners.map(|corner| scaled(corner, up)), &mut balance);
        });
    }
    balance
}

/// One triangle's tetrahedron with the world origin added to a moment and a
/// weight: its six volumes times the sum of its corners, and four times its
/// six volumes.
fn weigh_in(corners: [[f64; 3]; 3], (moment, weight): &mut ([Vec<f64>; 3], Vec<f64>)) {
    let six_volume = six_times_the_volume_under(corners);
    *weight = plus(weight, &times(&six_volume, &[4.0]));
    for (axis, total) in moment.iter_mut().enumerate() {
        let summed = plus(
            &plus(&[corners[0][axis]], &[corners[1][axis]]),
            &[corners[2][axis]],
        );
        *total = plus(total, &times(&six_volume, &summed));
    }
}

/// Where one face's fan balances exactly under `docs/decisions.md` §338.5's
/// definition, as a moment and a weight whose ratio is the middle along each
/// axis: every triangle weighed by its shadow on the fan's exact vector area,
/// `(a × b + b × c + c × a) · N`, and standing at `(a + b + c) / 3`. Every
/// corner is scaled by `2^up`, and the middle with it.
fn where_the_face_balances(face: FaceView<'_>, up: i32) -> ([Vec<f64>; 3], Vec<f64>) {
    let mut triangles = Vec::new();
    let mut twice: [Vec<f64>; 3] = Default::default();
    face.for_each_triangle(|corners| {
        let corners = corners.map(|corner| scaled(corner, up));
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
///
/// `moment` and `weight` were summed over corners scaled by `2^up`, so the
/// middle and the wobble are scaled the same way first. Squaring doubles how
/// far the sums reach down, and the smallest products of the squares can fall
/// below the subnormals where the sums did not; each loses less than half the
/// smallest subnormal, so the room left between the two sides must be more than
/// all of them could have lost, or the comparison has not decided anything.
fn the_share_of_its_wobble(
    what: &str,
    middle: Point,
    wobble: f64,
    (moment, weight): (&[Vec<f64>; 3], &[f64]),
    up: i32,
) -> f64 {
    assert!(
        wobble.is_finite(),
        "{what}: nothing is worth asserting against an infinite bound (§47.4)"
    );
    let middle = scaled(middle, up);
    let wobble = lifted(wobble, up);
    products_not_exact_so_far();
    let squared = moment
        .iter()
        .zip(middle)
        .fold(Vec::new(), |total, (moment, at)| {
            let off = plus(&times(weight, &[at]), &negated(moment));
            plus(&total, &times(&off, &off))
        });
    let allowed = times(&times(weight, weight), &times(&[wobble], &[wobble]));
    let lost = products_not_exact_so_far();
    let room = plus(&allowed, &negated(&squared));
    let distance = roughly(&squared).sqrt() / roughly(weight).abs();
    assert!(
        at_least_zero(&room),
        "{what}: a middle measured at {middle:?} is {:e} from where the triangles balance, and \
         the kernel owned up to {:e}",
        lifted(distance, -up),
        lifted(wobble, -up)
    );
    assert!(
        lost == 0 || more_than(&room, lost),
        "{what}: {lost} products of comparing a middle with its wobble fell below the \
         subnormals, and the room between the two is too narrow to say they did not decide it"
    );
    if distance == 0.0 {
        return 0.0;
    }
    distance / wobble
}

/// Whether `room`, which is at least zero, is more than `lost` products that
/// fell below the subnormals could have taken from it: each of those
/// [`two_product`](rcad_predicates::expansion::two_product) rounds once, into
/// a multiple of the smallest subnormal, so it is out by at most half of one.
fn more_than(room: &[f64], lost: usize) -> bool {
    let Some((largest, rest)) = room.split_last() else {
        return false;
    };
    let rest = rest
        .iter()
        .fold(0.0, |total: f64, entry| (total + entry.abs()).next_up());
    let lost = f64::from(u32::try_from(lost).expect("a count that fits")) * f64::from_bits(1);
    (largest - rest).next_down() > lost
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

thread_local! {
    /// How many products [`times`] took that [`product_is_exact`] refuses,
    /// since [`products_not_exact_so_far`] was last asked.
    static PRODUCTS_NOT_EXACT: Cell<usize> = const { Cell::new(0) };
}

/// How many products [`times`] took that may have lost something below the
/// subnormals since this was last asked, and starts the count again.
fn products_not_exact_so_far() -> usize {
    PRODUCTS_NOT_EXACT.with(|count| count.replace(0))
}

fn times(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut total = Vec::new();
    let mut piece = vec![0.0; 2 * left.len()];
    for &factor in right {
        let refused = left
            .iter()
            .filter(|&&entry| !product_is_exact(entry, factor))
            .count();
        PRODUCTS_NOT_EXACT.with(|count| count.set(count.get() + refused));
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
