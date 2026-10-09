//! A fuse over faces whose sides are arcs of the **ellipses** two leaning
//! planes cut a cone in (`docs/decisions.md` §211.8's loose end).
//!
//! §211 gave a cone's closed crossing a centre and taught `holds.rs`'s
//! `the_way_into_an_arc` to read a picker off it. `fuse/arrange.rs` carried a
//! second copy of that function whose second line asked for a `Surface::Sphere`
//! by name, so a plane and a cone fell out of it before any other test was
//! reached and every arc of a cone's rim — circle or ellipse — came back
//! `AFaceTrimmedByACurveThisCannotFollow` from the arrangement. The two are one
//! function now, and this is the fuse asking it about an ellipse.
//!
//! # The solid: a lens of a cone between two leaning lids
//!
//! Both lids hold the line `x = 0, z = 4`: the lower one is `x + z = 4` and the
//! upper one is `z − x = 4`, and the solid is the part of the inside of a cone
//! of slope a half — apex at the origin, about `+z` — between them. Each lid
//! leans a quarter turn off the axis, far more than the cone's own half angle,
//! so each cuts an **ellipse** and not a circle: `(1 + s²)(n·w)² − s²|n|²|w|²`
//! comes to `0.75c²` for both, with `c` the lid normal's repeated component,
//! which is §211.2's door standing open.
//!
//! **Neither lid goes through the apex**, and that is deliberate. A plane
//! through the apex cuts the cone in two straight rulings rather than in a
//! closed curve, which is a different question from this file's. The
//! arrangement used to fold two corners on those two rulings into one node and
//! refuse `EdgeHasNoLength`; it does not since `f2fc7d1d`, and
//! `a_frustum_cut_in_half_down_its_axis.rs` measures what such a cut does now:
//! it answers where the cutting face stays clear of the apex, and refuses where
//! the apex is on that face.
//!
//! Three faces, three edges, two corners: the two lids are each half of their
//! own ellipse, the piece of the cone between them is bounded by both arcs, and
//! the chord `x = 0, z = 4` closes each lid.
//!
//! # Why every number in it is exact
//!
//! Each lid's unit normal has a repeated `1/√2` in it, which is irrational, so a
//! corner is only exactly on a lid by construction and never by luck. Each lid is
//! written through `(0, 0, 4)` on the axis, so its stored offset is `±4c` with no
//! sum to round, and each corner `(0, ±2, 4)` dots to `(∓c)·0 + 0·(±2) + (±c)·4`
//! — the same `±4c` to the bit, because the lid normals' `y` components are
//! exactly zero. The corners sit on the cone for `cones/mod.rs`'s reason: a
//! power-of-two slope makes `slope · height` exact, and `0.5 · 4 = 2`.
//!
//! # Which of the four ways into an arc each lid asks
//!
//! [`the_way_into_an_arc`] reads a picker one of four ways — a third,
//! `into_the_arc_on_the_centres_side`, was added for a frustum's floor disc
//! (Q971) and is `arrange.rs`'s own since Q1033, rather than a second copy of
//! the first two; a fourth, `into_the_arc_away_from_the_centre`, was added in
//! `holds.rs` for a window arc whose ends' boxes straddle its picker (Q1093) —
//! and the two lids here ask one of the first two each, on purpose, because the
//! second is the one §211's arithmetic is for.
//!
//! The **upper lid**'s arc is the bigger half of its ellipse, so the ellipse's
//! centre lies on the arc's own side of the chord and no half-space can hold the
//! arc in front of it and the centre behind. Its picker is the chord's own plane
//! `x ≥ 0`, which holds both corners: that is the first way, and it never asks
//! where the centre is.
//!
//! The **lower lid**'s arc is the smaller half — its ellipse runs from `x = −4`
//! to `x = 4/3` and the arc kept is `x ≥ 0` — so the centre is at `x = −4/3` and
//! `x ≥ −0.5` holds the whole arc strictly in front of it with the centre
//! strictly behind. That is the second way, and it is the one that walks
//! §211.3's expression over an ellipse: `Cone::which_way_a_closed_crossings
//! _centre_lies` hands back the pole's direction and the apex, and the sign is
//! cleared of its division. On the base the fuse never got that far, because the
//! `Surface::Sphere` two lines above had already refused.
//!
//! [`the_way_into_an_arc`]: rcad_topo
//!
//! # A block big enough to hold the lens still cuts its own wall
//!
//! `docs/decisions.md` §215.2 named three walls a block can cut the lens with:
//! a wall straight through the lens, one along its edge, and a third that
//! stands clear of the lens in `x` and `y` alike. The tests below pin the first
//! two kinds against five blocks and the third against one: `(-3, -6, 0)` to
//! `(7, 6, 9)`. That block's own wall `x = -3` never meets the lens's two lids —
//! the lower one, `x + z = 4`, crosses it at `z = 7`, and the upper one,
//! `z − x = 4`, crosses it at `z = 1`, both well outside the block's own faces
//! at `y = ±6` — but the lens's **cone** still reaches out that far, since the
//! cone's radius at `z = 7` is `3.5`, past `3`. A plane parallel to the cone's
//! axis cuts it in a hyperbola rather than an ellipse, and the piece of the
//! wall inside the lens is bounded by an arc of that hyperbola on one side and
//! the straight line `z = 7` on the other, closing at its own two corners,
//! `(-3, ±1.803, 7)` — a two-edge, two-corner face the same shape as the lens's
//! own lids. §215.2's third wall answers today, under every rule and both
//! orders, the same as the other five.

use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Vector,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    Containment, DraftStep, Error, FaceKey, Model, OnAFace, ProfileBuilder, Rule, SolidBuilder,
    SolidKey, SolidView, WireStepView,
};

/// Slope a half, so `slope · height` rounds to nothing at the corners.
const SLOPE: f64 = 0.5;

/// Where both lids cross the axis, and how far up the cone the corners sit.
const LIDS_MEET_THE_AXIS_AT: f64 = 4.0;

/// How far behind the lower lid's corners its picker's plane stands.
///
/// Anything between nothing and `4/3` puts the corners strictly in front of it
/// and that ellipse's centre strictly behind; a half is a round number in the
/// middle of that range.
const THE_LOWER_LIDS_PICKER_STANDS_BACK: f64 = 0.5;

/// A bar driven right through the lens, in through the lower lid and out through
/// the upper one: the rectangle `x ∈ [0.5, 1]`, `y ∈ [−0.25, 0.25]` run from
/// `z = 2.75` to `z = 5.5`.
///
/// **The two lids are the only faces of the lens it crosses.** Its furthest
/// corner is `1.031` from the axis and the cone is already `1.375` wide where the
/// bar starts, so it never reaches the cone's skin. The lower lid crosses it
/// between `z = 3` and `z = 3.5` and the upper one between `z = 4.5` and `z = 5`,
/// all four well inside its ends — so both of its own ends are clear of the lens
/// and each lid cuts it rather than grazing it.
const BAR_CORNERS: [[f64; 2]; 4] = [[0.5, -0.25], [1.0, -0.25], [1.0, 0.25], [0.5, 0.25]];
const BAR_FROM: f64 = 2.75;
const BAR_HEIGHT: f64 = 2.75;

/// The lens is a solid, and the two ellipse arcs are its own sides.
///
/// The fixture in its own right: the corners are certified on the cone and on
/// both lids, and each arc is named by a half-space that tells it from its
/// sibling. This passes on the base too — the arrangement is what the base
/// cannot do, not the description.
#[test]
fn a_lens_of_a_cone_between_two_leaning_lids_is_a_solid() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let lens = a_lens_of_a_cone(&mut model);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a lens described this way is a sound model");
    let mut surfaces: Vec<Surface> = model
        .solid(lens)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .collect();
    let mut want = vec![
        Surface::Plane(the_lower_lid()),
        Surface::Plane(the_upper_lid()),
        Surface::Cone(the_cone()),
    ];
    let key = |surface: &Surface| format!("{surface:?}");
    surfaces.sort_by_key(key);
    want.sort_by_key(key);
    assert_eq!(
        surfaces, want,
        "a lens is two leaning lids and a piece of a cone"
    );
    assert_eq!(
        how_many_faces_along_each_edge(&model, lens).len(),
        3,
        "a lens has three edges: the two arcs and the chord"
    );
    for (edge, how_many) in how_many_faces_along_each_edge(&model, lens) {
        assert_eq!(how_many, 2, "the lens has {how_many} faces along {edge}");
    }
}

/// The arrangement no longer gives up where an arc of a cone's rim is an
/// **ellipse**, under every rule.
///
/// On the base this asks a question the fuse cannot answer at all:
/// `arrange.rs`'s `the_way_into_an_arc` looked for a `Surface::Sphere` among the
/// pair, found a cone, and returned `None` — and its one caller,
/// `which_way_an_edge_runs_round`, turns that `None` into
/// `AFaceTrimmedByACurveThisCannotFollow` by an `ok_or`. Run against
/// `3ce21631` in a clone of its own, this test comes back
/// `Err(AFaceTrimmedByACurveThisCannotFollow { face: FaceSlot(1v1) })` under
/// `Rule::Union` while the fixture above passes; with the two copies folded
/// into one it passes.
///
/// The centre is really asked, and of an ellipse. Instrumenting
/// `where_the_curves_centre_sits` with the lens in hand prints 45 calls, every
/// one of them `Neg` and every one of them measured from `Point([0, 0, 0])` —
/// the cone's apex, which is the cone arm of `where_a_closed_crossing_is
/// _centred` and not a ball's centre.
///
/// **Every rule now answers.** This test used to pin `Undecidable` under all
/// three: once the arc was read, the lens's own cone face could not be asked
/// what it held, so nothing could be classified against the lens. The cone arm
/// of `holds.rs`'s `is_a_family_this_follows` takes that face since `0552f4d1`,
/// and what each rule's answer is worth is the business of the volume tests
/// below; this one says only that none of the three refuses.
#[test]
fn the_arrangement_reads_the_way_into_a_cones_ellipse_arc() {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let (mut model, lens, bar) = a_lens_and_a_bar();
        let said = model.fuse(
            &[lens],
            &[bar],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            !matches!(
                said,
                Err(Error::AFaceTrimmedByACurveThisCannotFollow { .. })
            ),
            "under {rule:?} the arrangement gave up on an ellipse arc it can read: {said:?}"
        );
        assert!(said.is_ok(), "under {rule:?} the fuse refused: {said:?}");
    }
}

/// A point plainly on the piece of cone between the two lids is held by it.
///
/// `(2, 0, 4)` is on the cone to the bit — the slope is a half and `0.5 · 4 = 2`
/// — and it is strictly between the lids: `x + z` is `6`, at least the lower
/// lid's `4`, and `z − x` is `2`, at most the upper lid's `4`. So the answer is
/// `Held`, worked out from the shape alone.
///
/// This used to pin `AFaceTrimmedByACurveThisCannotFollow`: the cone arm of
/// `is_a_family_this_follows` refused any cone face with its apex behind a cut,
/// and the apex is behind the lower lid. Since `0552f4d1` one such cut that
/// closes, beside one other, is followed.
#[test]
fn the_lenss_cone_face_holds_a_point_between_its_lids() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let lens = a_lens_of_a_cone(&mut model);
    let view = model.solid(lens).expect("this model made the key");
    let skin = the_piece_of_cone(view);
    let said = view.face_holds_the_point(
        skin,
        PredicatePoint::from(Point::new(2.0, 0.0, 4.0).coordinates()),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(said, Ok(OnAFace::Held)),
        "the piece of cone said something else about a point on it: {said:?}"
    );
}

/// The middle of the bar's own bottom end is outside the lens.
///
/// `(0.625, −0.125, 2.75)` has `x + z = 3.375`, short of the lower lid's `4`, so
/// it is on the far side of the lower lid from the lens — `Outside`, from the
/// shape alone. It is the candidate the fuse used to give up on: classifying it
/// stopped at the lens's cone face and came back `CannotFollowAFacesTrimming`,
/// which is what this test pinned before `0552f4d1` let that face answer.
#[test]
fn the_middle_of_the_bars_bottom_end_is_outside_the_lens() {
    let (model, lens, _bar) = a_lens_and_a_bar();
    let view = model.solid(lens).expect("this model made the key");
    let said = view.classify_point(
        Point::new(0.625, -0.125, 2.75),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(said, Containment::Outside),
        "the lens placed the bar's own bottom end somewhere else: {said:?}"
    );
}

/// The lens and the bar united hold the lens, and the bar less what the two
/// share.
///
/// **The lens, worked out by hand.** Only `x ≥ 0` has room between the lids, and
/// there a column at `(x, y)` runs from `max(4 − x, 2r)` up to `4 + x`, with
/// `r = √(x² + y²)`. That height is `(4 + x − 2r)₊ − (4 − x − 2r)₊`, and
/// mirroring the second term through `x = 0` makes the whole volume
/// `∬ sign(x)·(4 + x − 2r)₊`. Round the axis, `x = r cos θ`, and the inner
/// integral `∫ (4 − r(2 − cos θ)) r dr` up to `r = 4 / (2 − cos θ)` is
/// `32 / (3(2 − cos θ)²)`. Put `t = tan(θ/2)` and
/// `∫ dθ / (2 − cos θ)² = (4 / 3√3) atan(√3 t) + 2t / (3(1 + 3t²))`, so the
/// quarter turns either side of `x = 0` give `4π/9√3 + 1/6` and
/// `2π/9√3 − 1/6`, and the lens is `(64/3)(2π/9√3 + 1/3)`:
///
/// ```text
/// lens = 128π / (27√3) + 64/9 ≈ 15.709864
/// ```
///
/// A midpoint sum on an 8000 × 16000 grid agrees to `1e−7`. The bar is
/// `½ · ½ · 11/4 = 11/16`, and what they share is `3/8`
/// (`a_bar_through_both_lids_intersected_with_the_lens_is_three_eighths`), so
/// the union is `lens + 5/16`.
///
/// **This refused `CannotMeshAFace { why: CoordinateOutOfRange }` on
/// `0552f4d1`**, and not because of the cone face that commit taught to answer.
/// The arrangement cuts the lens's cone along the planes of the bar's walls,
/// which run along the axis and so cross the cone in hyperbolas, and the fuse
/// named each arc of one by the half-space through its chord. That holds
/// the arc and the whole other branch, and `rcad_geom::walk` asks it at each
/// branch's vertex, so it walked the arc on the nappe below the apex — a face
/// reaching across both nappes, whose domain the triangulator could not scale.
/// The arc is named by the nappe it is on now (`fuse/assemble.rs`,
/// `the_branch_of_a_hyperbola_this_is_on`).
#[test]
fn the_lens_and_the_bar_unite_to_the_closed_form() {
    let the_lens = 128.0 * std::f64::consts::PI / (27.0 * 3.0_f64.sqrt()) + 64.0 / 9.0;
    what_the_fuse_measures(Rule::Union, the_lens + 5.0 / 16.0);
}

/// The bar taken out of the lens leaves the lens less the `3/8` they share —
/// the lens's volume is worked out in the test above.
///
/// It went through the same hyperbola arcs on the lens's cone and refused the
/// same way on `0552f4d1`.
#[test]
fn the_bar_taken_out_of_the_lens_leaves_the_closed_form() {
    let the_lens = 128.0 * std::f64::consts::PI / (27.0 * 3.0_f64.sqrt()) + 64.0 / 9.0;
    what_the_fuse_measures(Rule::Difference, the_lens - 3.0 / 8.0);
}

/// A block from `(-1, -6, 2)` to `(7, 6, 9)` holds the lens whole, so the union
/// is the block itself, the intersection is the lens, and the difference of
/// the lens less the block is empty.
#[test]
fn a_block_that_holds_the_whole_lens_fuses_exactly() {
    pin_block_against_the_lens(
        "a block holding the whole lens",
        [-1.0, -6.0, 2.0],
        [7.0, 6.0, 9.0],
    );
}

/// The block's wall `x = 1` runs straight through the lens.
#[test]
fn a_blocks_wall_at_x_one_cuts_the_lens() {
    pin_block_against_the_lens(
        "a block whose wall is x = 1",
        [1.0, -6.0, 2.0],
        [7.0, 6.0, 9.0],
    );
}

/// The block's wall `x = 2` runs straight through the lens, from the other
/// side of it.
#[test]
fn a_blocks_wall_at_x_two_cuts_the_lens_from_the_other_side() {
    pin_block_against_the_lens(
        "a block whose wall is x = 2, from the other side",
        [-1.0, -6.0, 2.0],
        [2.0, 6.0, 9.0],
    );
}

/// The block's two walls `y = -1` and `y = 1` both cut the lens.
#[test]
fn a_blocks_walls_either_side_in_y_cut_the_lens() {
    pin_block_against_the_lens(
        "a block whose walls are y = -1 and y = 1",
        [-1.0, -1.0, 2.0],
        [7.0, 1.0, 9.0],
    );
}

/// The block's wall `y = 0.5` cuts the lens off-centre.
#[test]
fn a_blocks_wall_at_y_a_half_cuts_the_lens() {
    pin_block_against_the_lens(
        "a block whose wall is y = 0.5",
        [-1.0, 0.5, 2.0],
        [7.0, 6.0, 9.0],
    );
}

/// Two of the block's walls, `x = 1` and `y = 1`, meet in a corner that cuts
/// the lens.
#[test]
fn a_blocks_corner_in_x_and_y_cuts_the_lens() {
    pin_block_against_the_lens(
        "a block whose corner is x = 1, y = 1",
        [1.0, 1.0, 2.0],
        [7.0, 6.0, 9.0],
    );
}

/// A block from `(-3, -6, 0)` to `(7, 6, 9)` holds the lens whole too, but its
/// own wall `x = -3` is far enough out that only the cone reaches it, in a
/// piece bounded by a hyperbola and a straight line — see the module doc's "A
/// block big enough to hold the lens still cuts its own wall".
#[test]
fn a_block_big_enough_for_the_lens_still_cuts_its_own_wall_in_a_hyperbola() {
    pin_block_against_the_lens(
        "a block big enough for the lens, whose own wall is x = -3",
        [-3.0, -6.0, 0.0],
        [7.0, 6.0, 9.0],
    );
}

/// Fuse the lens against a block from `lo` to `hi` under every rule and both
/// orders, and check each answer against the exact volume: the union is the
/// block plus the lens where the block does not hold it, the intersection is
/// what the two share, and the difference is whichever argument came first
/// less that shared part.
///
/// The shared part is [`lens_within`]'s closed form, not a mesh measurement, so
/// this is the same kind of check as `the_lens_and_the_bar_unite_to_the_closed_form`
/// above, just against a block instead of a bar.
fn pin_block_against_the_lens(name: &str, lo: [f64; 3], hi: [f64; 3]) {
    let lens_volume = 128.0 * std::f64::consts::PI / (27.0 * 3.0_f64.sqrt()) + 64.0 / 9.0;
    let block_volume = (hi[0] - lo[0]) * (hi[1] - lo[1]) * (hi[2] - lo[2]);
    let common = lens_within(lo[0], hi[0], lo[1], hi[1]);
    let union = block_volume + lens_volume - common;
    let lens_less_common = lens_volume - common;
    let block_less_common = block_volume - common;
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for lens_first in [true, false] {
            let want = match (rule, lens_first) {
                (Rule::Union, _) => union,
                (Rule::Intersection, _) => common,
                (Rule::Difference, true) => lens_less_common,
                (Rule::Difference, false) => block_less_common,
                _ => panic!("{name}: this pin only knows union, intersection and difference"),
            };
            let order = if lens_first {
                "lens,block"
            } else {
                "block,lens"
            };
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let lens = a_lens_of_a_cone(&mut model);
            let block = a_block(&mut model, lo, hi);
            let (first, second) = if lens_first {
                (lens, block)
            } else {
                (block, lens)
            };
            let fused = model
                .fuse(
                    &[first],
                    &[second],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{name} under {rule:?} ({order}) refused: {why:?}"));
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| {
                    panic!(
                        "{name} under {rule:?} ({order}) left a model that does not validate: {why}"
                    )
                });
            let (volume, error) = volume_of_each(&model, fused.solids());
            // `want` is itself a sum of an exact constant and a quadrature
            // result (`lens_within`), so even where the true answer is exactly
            // `0.0` — the lens entirely inside the block — `want` can miss it
            // by a couple of ulps though the fuse's own measured volume does
            // not. `1e-9` is far inside the mesh's own declared error whenever
            // that error is nonzero, and covers this when it is zero.
            assert!(
                (volume - want).abs() <= error + 1e-9,
                "{name} under {rule:?} ({order}) is {volume} against {want}, and the mesh may miss by {error}"
            );
        }
    }
}

/// A rectangular block from `lo` to `hi`, extruded the same way the lens and
/// the bar's own fixture is.
fn a_block(model: &mut Model, lo: [f64; 3], hi: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [lo[0], lo[1]],
        [hi[0], lo[1]],
        [hi[0], hi[1]],
        [lo[0], hi[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle on the grid");
    model
        .add_extrusion(
            &rectangle,
            hi[2] - lo[2],
            &Isometry::translation(Vector::new(0.0, 0.0, lo[2])).expect("a finite shift"),
        )
        .expect("a block")
}

/// The volume several solids hold together, and how far off it may be.
fn volume_of_each(model: &Model, solids: &[SolidKey]) -> (f64, f64) {
    let (mut volume, mut error) = (0.0, 0.0);
    let fine = Deflection::of(0.001).expect("a positive length");
    for &solid in solids {
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(fine, &Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("a solid does not measure: {why:?}"));
        volume += measured.volume();
        error += measured.volume_error();
    }
    (volume, error)
}

/// The lens's own closed form integrates `x` outward with `y` closed form; this
/// checks the same volume the other way round, `y` outward with `x` closed
/// form, so it can also answer for a box that clips `x` as well as `y`.
#[test]
fn the_x_closed_form_agrees_with_the_lenss_own_closed_form() {
    let lens_exact = 128.0 * std::f64::consts::PI / (27.0 * 3.0_f64.sqrt()) + 64.0 / 9.0;
    let numeric = lens_within(-10.0, 10.0, -10.0, 10.0);
    assert!(
        (numeric - lens_exact).abs() < 1e-9,
        "the two closed forms disagree: {numeric} against {lens_exact}"
    );
}

/// The lens's volume within `x ∈ [xa, xb]`, `y ∈ [ya, yb]`, integrated over `y`
/// with the `x` integral in closed form.
///
/// At a fixed `y`, the column at `(x, y)` runs from `max(4 − x, 2r) up to 4 + x`
/// (see `the_lens_and_the_bar_unite_to_the_closed_form`'s comment, with `r =
/// √(x² + y²)`): `2x` while `2r ≤ 4 − x` and `4 + x − 2r` from there up to where
/// `2r = 4 + x`. Both crossings solve a quadratic in `x` for a given `y`:
/// `3x² + 8x + 4y² − 16 = 0` for `2r = 4 − x`, whose non-negative root is
/// `x_a = (−4 + 2s) / 3`, and `3x² − 8x + 4y² − 16 = 0` for `2r = 4 + x`, whose
/// roots are `(4 ± 2s) / 3`, with `s = √(16 − 3y²)` in both. The lower root of
/// the second quadratic is `−x_a`, so past `|y| = 2` the first regime never
/// starts and the whole column is the second. `height_integral_x` does one `y`;
/// a panel sum with a break at `y = ±2` carries it over `[ya, yb]`.
fn lens_within(xa: f64, xb: f64, ya: f64, yb: f64) -> f64 {
    let max_y = (16.0f64 / 3.0).sqrt();
    let y0 = ya.max(-max_y);
    let y1 = yb.min(max_y);
    if y1 <= y0 {
        return 0.0;
    }
    let mut breaks = vec![y0, y1];
    for break_at in [-2.0, 2.0] {
        if break_at > y0 && break_at < y1 {
            breaks.push(break_at);
        }
    }
    breaks.sort_by(f64::total_cmp);
    let nodes = [
        (0.0, 128.0 / 225.0),
        (
            -(5.0 - 2.0 * (10.0f64 / 7.0).sqrt()).sqrt() / 3.0,
            (322.0 + 13.0 * 70.0f64.sqrt()) / 900.0,
        ),
        (
            (5.0 - 2.0 * (10.0f64 / 7.0).sqrt()).sqrt() / 3.0,
            (322.0 + 13.0 * 70.0f64.sqrt()) / 900.0,
        ),
        (
            -(5.0 + 2.0 * (10.0f64 / 7.0).sqrt()).sqrt() / 3.0,
            (322.0 - 13.0 * 70.0f64.sqrt()) / 900.0,
        ),
        (
            (5.0 + 2.0 * (10.0f64 / 7.0).sqrt()).sqrt() / 3.0,
            (322.0 - 13.0 * 70.0f64.sqrt()) / 900.0,
        ),
    ];
    let mut total = 0.0;
    for window in breaks.windows(2) {
        let (lo, hi) = (window[0], window[1]);
        let panels = 20_000;
        let step = (hi - lo) / panels as f64;
        for i in 0..panels {
            let mid = lo + (i as f64 + 0.5) * step;
            for (t, w) in nodes {
                total += w * step / 2.0 * height_integral_x(mid + t * step / 2.0, xa, xb);
            }
        }
    }
    total
}

/// The lens's cross-section height integrated over `x ∈ [xlo, xhi]` at one `y`,
/// in closed form — see [`lens_within`]'s doc comment for the two regimes and
/// where they split.
fn height_integral_x(y: f64, xlo: f64, xhi: f64) -> f64 {
    let disc = 16.0 - 3.0 * y * y;
    if disc <= 0.0 {
        return 0.0;
    }
    let s = disc.sqrt();
    let x_upper = (4.0 + 2.0 * s) / 3.0;
    let x_lower = (4.0 - 2.0 * s) / 3.0;
    let lo = xlo.max(x_lower.max(0.0));
    let hi = xhi.min(x_upper);
    if hi <= lo {
        return 0.0;
    }
    if x_lower < 0.0 {
        let x_a = -x_lower;
        let mut total = 0.0;
        let lid_hi = hi.min(x_a);
        if lid_hi > lo {
            total += lid_hi * lid_hi - lo * lo;
        }
        let cone_lo = lo.max(x_a);
        if hi > cone_lo {
            total += cone_regime_antiderivative(hi, y) - cone_regime_antiderivative(cone_lo, y);
        }
        total
    } else {
        cone_regime_antiderivative(hi, y) - cone_regime_antiderivative(lo, y)
    }
}

/// An antiderivative, in `x` at fixed `y`, of `4 + x − 2√(x² + y²)`.
fn cone_regime_antiderivative(x: f64, y: f64) -> f64 {
    4.0 * x + x * x / 2.0 - 2.0 * x_times_r_antiderivative(x, y)
}

/// An antiderivative, in `x` at fixed `y`, of `√(x² + y²)`.
fn x_times_r_antiderivative(x: f64, y: f64) -> f64 {
    if y == 0.0 {
        x * x / 2.0
    } else {
        (x * (x * x + y * y).sqrt() + y * y * (x / y.abs()).asinh()) / 2.0
    }
}

/// Fuse the lens and the bar under `rule`, check the model, and hold the one
/// solid that comes out to `want` within what its mesh says it may miss by.
///
/// Every point of the mesh is also held above `z = 8/3`, the lowest the lens
/// reaches — where `x + z = 4` meets the cone's ruling along `+x` — and the bar
/// starts higher. A point on the nappe below the apex is what the misnamed
/// arcs produced.
fn what_the_fuse_measures(rule: Rule, want: f64) {
    let (mut model, lens, bar) = a_lens_and_a_bar();
    let fused = model
        .fuse(
            &[lens],
            &[bar],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("under {rule:?} the fuse refused: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("what the fuse made is a sound model");
    let [piece] = fused.solids() else {
        panic!("one piece under {rule:?}: {:?}", fused.solids())
    };
    let view = model.solid(*piece).expect("a solid the fuse made");
    let fine = Deflection::of(0.001).expect("a positive length");
    let mesh = view
        .mesh(fine, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer meshes");
    let lowest = mesh
        .points()
        .iter()
        .map(|point| point.coordinates()[2])
        .fold(f64::INFINITY, f64::min);
    assert!(
        lowest > 2.6,
        "under {rule:?} a mesh point sits at z = {lowest}"
    );
    let measured = view
        .measure(fine, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer measures");
    assert!(
        (measured.volume() - want).abs() <= measured.volume_error(),
        "under {rule:?} the volume is {} against {want}, and the mesh may miss by {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The one face of the lens drawn on a cone.
fn the_piece_of_cone(view: SolidView<'_>) -> FaceKey {
    let mut cones = view
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Cone(_)));
    let skin = cones.next().expect("the lens has a piece of a cone in it");
    assert!(cones.next().is_none(), "the lens has one piece of a cone");
    skin.key()
}

/// The lens, and a bar poking out through its upper lid, in one model.
fn a_lens_and_a_bar() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let lens = a_lens_of_a_cone(&mut model);
    let square = ProfileBuilder::new(BAR_CORNERS.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle on the grid");
    let bar = model
        .add_extrusion(
            &square,
            BAR_HEIGHT,
            &Isometry::translation(Vector::new(0.0, 0.0, BAR_FROM)).expect("a finite shift"),
        )
        .expect("a rectangle extruded straight up is a solid");
    (model, lens, bar)
}

/// The cone the lens is cut out of.
fn the_cone() -> Cone {
    Cone::new(Axis::z(), SLOPE).expect("a positive slope")
}

/// The lid `x + z = 4`, facing out of the lens, which is on its `≥` side.
fn the_lower_lid() -> Plane {
    the_lid(-1.0, -1.0)
}

/// The lid `z − x = 4`, facing out of the lens, which is on its `≤` side.
fn the_upper_lid() -> Plane {
    the_lid(-1.0, 1.0)
}

/// A lid through `(0, 0, 4)` on the axis, leaning `across` in `x` and `up` in
/// `z`, which is what keeps the two corners exactly on it — see the module doc.
fn the_lid(across: f64, up: f64) -> Plane {
    Plane::from_point_and_normal(
        Point::new(0.0, 0.0, LIDS_MEET_THE_AXIS_AT),
        Vector::new(across, 0.0, up),
    )
    .expect("a vector with a length")
}

/// The plane the chord lies in, which is the upper lid's picker and the second
/// surface both arcs are named against.
fn the_chord_plane() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal")
}

/// A lens of a cone between two leaning lids, described face by face through the
/// public builder.
fn a_lens_of_a_cone(model: &mut Model) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let reach = SLOPE * LIDS_MEET_THE_AXIS_AT;
    let corner = |builder: &mut SolidBuilder, y: f64| {
        builder
            .add_vertex(Point::new(0.0, y, LIDS_MEET_THE_AXIS_AT))
            .expect("a finite point")
    };
    let positive = corner(&mut builder, reach);
    let negative = corner(&mut builder, -reach);

    let arc = |and: Plane, picker: HalfSpace| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(and),
        picked_by: smallvec::smallvec![Picker::from(picker)],
    };
    let facing_the_arcs = |from_x: f64| {
        HalfSpace::through_a_point_facing(Point::new(from_x, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
            .expect("the x axis has length")
    };
    let lower_arc = builder
        .add_edge(
            positive,
            negative,
            arc(
                the_chord_plane(),
                facing_the_arcs(-THE_LOWER_LIDS_PICKER_STANDS_BACK),
            ),
        )
        .expect("the smaller half of the lower lid's ellipse");
    let upper_arc = builder
        .add_edge(
            negative,
            positive,
            arc(the_chord_plane(), facing_the_arcs(0.0)),
        )
        .expect("the bigger half of the upper lid's ellipse");
    let chord = builder
        .add_edge(negative, positive, EdgeRun::Straight)
        .expect("the line both lids hold");

    let wire_of = |builder: &mut SolidBuilder, steps: &[(_, Orientation)]| {
        let along: Vec<DraftStep> = steps
            .iter()
            .map(|(edge, turn)| {
                DraftStep::Along(
                    builder
                        .add_coedge(*edge, *turn)
                        .expect("an edge of this build"),
                )
            })
            .collect();
        builder.add_wire(&along).expect("a closed wire")
    };
    let lower_wire = wire_of(
        &mut builder,
        &[
            (lower_arc, Orientation::Forward),
            (chord, Orientation::Forward),
        ],
    );
    let upper_wire = wire_of(
        &mut builder,
        &[
            (upper_arc, Orientation::Forward),
            (chord, Orientation::Reversed),
        ],
    );
    let skin_wire = wire_of(
        &mut builder,
        &[
            (lower_arc, Orientation::Reversed),
            (upper_arc, Orientation::Reversed),
        ],
    );

    let lower = builder
        .add_face(Surface::Plane(the_lower_lid()), &[lower_wire])
        .expect("half of the lower lid's ellipse");
    let upper = builder
        .add_face(Surface::Plane(the_upper_lid()), &[upper_wire])
        .expect("half of the upper lid's ellipse");
    let skin = builder
        .add_face(Surface::Cone(the_cone()), &[skin_wire])
        .expect("the piece of the cone between the two lids");

    builder
        .add_shell(&[
            (lower, Orientation::Forward),
            (upper, Orientation::Forward),
            (skin, Orientation::Forward),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a lens described this way is a solid")
}

/// How many coedges — which is how many faces — walk each edge of a solid.
///
/// An edge is named by the `Debug` spelling of its key, because a key carries no
/// order of its own and a `BTreeMap` needs one. It is exact all the same: two
/// different keys never print alike.
fn how_many_faces_along_each_edge(model: &Model, solid: SolidKey) -> BTreeMap<String, usize> {
    let view = model.solid(solid).expect("a solid the fuse made");
    let mut along: BTreeMap<String, usize> = BTreeMap::new();
    for face in view.faces() {
        for wire in face.wires() {
            for step in wire.steps() {
                let WireStepView::Along(coedge) = step else {
                    panic!("these answers are made of coedges and nothing else: {step:?}")
                };
                *along
                    .entry(format!("{:?}", coedge.edge().key()))
                    .or_default() += 1;
            }
        }
    }
    along
}
