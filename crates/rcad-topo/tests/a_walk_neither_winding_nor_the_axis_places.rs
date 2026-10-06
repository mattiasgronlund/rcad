//! Shapes that reach the refusal of `docs/decisions.md` §340.2: a walk on a
//! face whose signs match several walks of one other lump, where neither
//! winding (a flat face) nor going round the axis (a cone or a pipe) says which
//! of them holds it. The fuse refuses `CannotTellTheCellsOfAFaceApart` in
//! `arrange.rs`'s `the_walk_of_each_other_lump_each_walk_faces`.
//!
//! **Three shapes reached it: one on a ball, one on a plane and one on a
//! cone**, each under every rule and with the arguments both ways round. A
//! printed probe at each of the five places `arrange.rs` raises that refusal,
//! since removed, named that one place for every fuse. Each answer exists — two
//! ordinary solids, whose volumes each test's doc works out by hand — so each
//! refusal was a gap (§16.10). **The one on a plane is answered now**, since
//! winding on a flat face follows each curved side as well as the corners
//! (queue row Q1053), and every fuse holds the volume its doc works out. The
//! ones on a ball and on a cone still reach it.
//!
//! **The one on a cone is the axis rule giving up on a walk that goes round
//! nothing** (queue row Q1056): `the_one_that_goes_round_the_axis_as_well`
//! returns `None` at `its_own == 0`. The face is a chip of a cone bounded by
//! two arcs of hyperbolas alone, described face by face, and the other lump is
//! a circle with two hyperbola arcs on it.
//!
//! **What makes two walks of one lump agree** is that only a curve drawn in
//! full has a side, and a closed curve is drawn in full only when it is a cut.
//! Every face is cut by every other face of its own solid as well as the other
//! one's (`cuts.rs`'s header), so a face's own closed boundary curves are drawn
//! in full too. The ball face bounded by two arcs below shows it: its hole is
//! placed by those arcs' circles. So the two walks have to be told apart by
//! nothing at all, or by a curve that runs on:
//!
//! - **On a ball every curve closes**, so it takes two regions with the same
//!   side of every circle: a square rod through a ball leaves a patch at each
//!   end, both inside all four of its circles.
//! - **On a plane** a straight cut runs on, and two halves of a face either
//!   side of it agree about every circle. Winding by corners alone placed a
//!   walk there only when every side was straight, so a face with a round
//!   notch was not placed.
//! - **On a cone** a plane cuts an ellipse or a circle, which goes once round
//!   the axis, or a hyperbola or a parabola, which runs on away from the apex,
//!   or lines through the apex. A walk that goes round nothing has no closed
//!   curve in it. A curve that runs on is drawn only between its outermost
//!   nodes, and it meets every closed curve that is anywhere above it, so a
//!   lump of such arcs lies above every closed curve on the face. A lump of
//!   them with no side of the face in it is thrown away before any walk is
//!   compared (`drop_the_lumps_of_straight_cuts_alone`): a plate's triangular
//!   hole drawn above a roofed frustum is, below. So the route found to the
//!   axis rule's `its_own == 0` is a face whose own sides are such arcs alone,
//!   above every closed curve: the chip. Built by fusing a frustum with planes
//!   it is refused before that (two shapes below); described face by face it
//!   reaches it.
//!
//! The other shapes built on the way are pinned beside them, with what each
//! gives instead.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

mod cones;

/// A ball of radius 2 at the origin, less a ball of radius `0.3` at
/// `(0.5, 0.25, 1.9)`, against the square rod `|x|, |y| < 1`, `|z| < 5`.
///
/// The rod's four walls cut the big ball in four circles that cross each other
/// where its edges go through the ball, which is one lump with a patch inside
/// all four at each end. The dimple's circle is a lump of its own in the top
/// patch. Its outer walk agrees with both patches about every circle, the face
/// is a ball's, and the dimple's lump has the face's own boundary in it, so it
/// is wound once and not left out.
///
/// **The answer exists.** With `Q = ∫∫ 2√(4 − x² − y²)` over the square
/// `|x|, |y| < 1`, the part of the ball in the rod, and `L` the lens the two balls
/// share, the dimpled ball is `32π/3 − L`, the rod `40`, and the two share
/// `Q − L`, since the small ball lies inside the rod.
#[test]
fn a_dimpled_ball_through_a_square_rod_cannot_tell_the_two_ends_of_the_rod_apart() {
    every_rule_both_ways_refuses_on_a(
        "a dimpled ball through a square rod",
        || {
            let mut model = a_model();
            let ball = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
            let small = model
                .add_ball(0.3, &shift(0.5, 0.25, 1.9))
                .expect("a small ball");
            let dimpled = fuse_one(&mut model, ball, small, Rule::Difference);
            let rod = block(&mut model, [-1.0, -1.0, -5.0], [1.0, 1.0, 5.0]);
            (model, dimpled, rod)
        },
        |surface| matches!(surface, Surface::Sphere(ball) if ball.radius() == 2.0),
    );
}

/// A chip of the cone `r = z` (slope 1, apex at the origin), described face by
/// face: the part of the solid cone where `2x − z ≥ 1` and `2x + z ≤ 11`.
/// Against it, a tent: the prism along `y`, `|y| ≤ 10`, over the triangle
/// `(−11/16, 3/4)`, `(−11/8, 7/4)`, `(−13/16, 3/4)` in `xz`.
///
/// The chip's face on the cone is bounded by one arc of the hyperbola each of
/// its planes cuts, both from `(3, 4, 5)` to `(3, −4, 5)`, so its walk goes
/// round nothing and its lump has no closed curve in it. The tent stands on the
/// far side of the axis, clear of the chip, and §400 cannot prove the two apart,
/// so the tent's planes are drawn on the chip's face as well. Its floor
/// `z = 3/4` cuts a circle round the axis, below the chip. Its two walls lean
/// more steeply than the cone's side and cut hyperbolas round the `−x` side
/// only: the one through `(−11/16, 3/4)` dips below the circle and crosses it,
/// and the other crosses the first twice above the circle. Above the circle that
/// lump has two walks, the patch between the three and the rest, and the chip's
/// walk is above the circle like both of them. The printed probe said:
///
/// ```text
/// axis rule: walk 0: its_own == 0, candidates [2, 3]
/// site the_walk_of_each_other_lump_each_walk_faces: walk 0 (lump 0, winding Some(1), 2 sides)
///     against [2, 3] of lump 14 on Cone(..)
/// ```
///
/// **The answer exists**: the two do not meet. The tent is `20 · (1/8 · 1)/2 =
/// 5/4`, and the chip is the integral over `1 ≤ z ≤ 5` of the part of the disc
/// `r ≤ z` between `x = (1 + z)/2` and `x = (11 − z)/2`, which
/// [`the_chips_volume`] works out slice by slice. So the union is the two of
/// them, the intersection nothing, and each less the other is itself. The walk
/// that goes round nothing bounds a disc, the other lump holds a circle round
/// the axis and so is in no disc, and the chip's walk faces neither of its
/// walks: a gap (§16.10).
#[test]
fn a_chip_of_a_cone_beside_a_tent_cannot_tell_the_tents_two_walks_apart() {
    let mut model = a_model();
    let chip = a_chip_of_a_cone(&mut model);
    let tent = a_tent(&mut model);
    assert_holds(&model, chip, the_chips_volume(), "the chip");
    assert_holds(&model, tent, 1.25, "the tent");

    every_rule_both_ways_refuses_on_a(
        "a chip of a cone beside a tent",
        || {
            let mut model = a_model();
            let chip = a_chip_of_a_cone(&mut model);
            let tent = a_tent(&mut model);
            (model, chip, tent)
        },
        |surface| matches!(surface, Surface::Cone(_)),
    );
}

/// A plate `10 × 10 × 2` less a bar of radius 3 standing on its corner
/// `(0, 0)`, against a slab `[5, 20] × [−5, 15] × [1, 5]` less a bar of radius 1
/// at `(7.5, 5)`.
///
/// The slab's wall `x = 5` cuts the plate's top in two, and the slab's hole
/// cuts a whole circle in the half `x > 5`. The circle's outer walk agrees with
/// both halves, and the other half's side is the notch's arc. Its corners are
/// not its shape, so this was refused here until the winding followed each arc
/// as well (queue row Q1053): the notch's arc bulges towards `(0, 0)`, away from
/// the circle, so the half `x < 5` winds no times round it and the half `x > 5`
/// once.
///
/// The plate is `200 − 4.5π`, the slab `1200 − 4π`, and they share
/// `[5, 10] × [0, 10] × [1, 2]` less the hole, `50 − π`: so `1350 − 7.5π`
/// united, `50 − π` in common, `150 − 3.5π` and `1150 − 3π` either way round
/// less the other. Every rule, both ways round, answers one solid holding that.
#[test]
fn a_plate_with_a_round_notch_cut_in_two_beside_a_hole_is_answered_under_every_rule() {
    use std::f64::consts::PI;
    for (rule, plate_first) in every_rule_both_ways() {
        let mut model = a_model();
        let plate = block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
        let notch = model
            .add_round_bar(3.0, 4.0, &shift(0.0, 0.0, -1.0))
            .expect("a bar");
        let notched = fuse_one(&mut model, plate, notch, Rule::Difference);
        let slab = block(&mut model, [5.0, -5.0, 1.0], [20.0, 15.0, 5.0]);
        let hole = model
            .add_round_bar(1.0, 6.0, &shift(7.5, 5.0, 0.0))
            .expect("a bar");
        let holed = fuse_one(&mut model, slab, hole, Rule::Difference);
        let what = format!("{rule:?}, the plate first {plate_first}");
        let answer = fuse(&mut model, notched, holed, rule, plate_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let [solid] = answer[..] else {
            panic!("{what}: one solid, got {answer:?}");
        };
        let volume = match (rule, plate_first) {
            (Rule::Union, _) => 1350.0 - 7.5 * PI,
            (Rule::Intersection, _) => 50.0 - PI,
            (Rule::Difference, true) => 150.0 - 3.5 * PI,
            (Rule::Difference, false) => 1150.0 - 3.0 * PI,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(&model, solid, volume, &what);
    }
}

/// A ball of radius 2 cut down to `x ≤ 1`, `y ≤ 1`, against a ball of radius
/// `0.5` at `(−1.2, −1.6, 0)` on its skin.
///
/// The ball's face is bounded by an arc of each of two circles, and the small
/// ball's circle is a lump of its own inside it. The probe printed both arcs'
/// circles drawn in full, cut by the solid's own planes, and the circle's outer
/// walk agreeing with the one walk inside the face: §340.2 is not asked. Every
/// fuse is refused later, in selection, by a ray that meets the face its arcs
/// trim (§35).
#[test]
fn a_ball_face_bounded_by_two_arcs_places_a_hole_by_the_arcs_own_circles() {
    for (rule, cut_ball_first) in every_rule_both_ways() {
        let mut model = a_model();
        let ball = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
        let corner = block(&mut model, [-10.0, -10.0, -10.0], [1.0, 1.0, 10.0]);
        let cut_ball = fuse_one(&mut model, ball, corner, Rule::Intersection);
        let small = model
            .add_ball(0.5, &shift(-1.2, -1.6, 0.0))
            .expect("a small ball");
        let refused = fuse(&mut model, cut_ball, small, rule, cut_ball_first);
        assert!(
            matches!(
                refused,
                Err(Error::AFaceTrimmedByACurveThisCannotFollow { .. })
            ),
            "{rule:?}, the cut ball first {cut_ball_first}: {refused:?}"
        );
    }
}

/// A plate `10 × 10 × 2` with a round tab, a bar of radius 3 at `(10, 5)`,
/// against a pin of radius `0.5` and height 4 through the tab at `(11.5, 5)`.
///
/// The face's own corners are all on the plate's square and the pin's circle
/// on the plate's floor is outside it, so counting the corners alone wound the
/// circle no times, left its lump out as outside the face, and refused every
/// fuse `ResultIsNotManifold` (queue row Q1054). The tab's arc bulges out past
/// the circle, and the count follows it now.
///
/// The plate is `200 + 9π` and the pin `π`, and they share `π/2`: so
/// `200 + 9.5π` united, `π/2` in common, `200 + 8.5π` the plate less the pin,
/// and the pin less the plate is its two ends, `π/4` each.
#[test]
fn a_pin_through_a_plates_round_tab_is_answered_under_every_rule() {
    use std::f64::consts::PI;
    for (rule, plate_first) in every_rule_both_ways() {
        let mut model = a_model();
        let plate = block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
        let tab = model
            .add_round_bar(3.0, 2.0, &shift(10.0, 5.0, 0.0))
            .expect("a bar");
        let tabbed = fuse_one(&mut model, plate, tab, Rule::Union);
        let pin = model
            .add_round_bar(0.5, 4.0, &shift(11.5, 5.0, -1.0))
            .expect("a bar");
        let what = format!("{rule:?}, the plate first {plate_first}");
        let answer = fuse(&mut model, tabbed, pin, rule, plate_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let volumes: &[f64] = match (rule, plate_first) {
            (Rule::Union, _) => &[200.0 + 9.5 * PI],
            (Rule::Intersection, _) => &[PI / 2.0],
            (Rule::Difference, true) => &[200.0 + 8.5 * PI],
            (Rule::Difference, false) => &[PI / 4.0, PI / 4.0],
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_each_holds(&model, &answer, volumes, &what);
    }
}

/// A plate `10 × 10 × 2` with a round notch, less a bar of radius 3 at
/// `(10, 5)`, against a pin of radius `0.5` and height 4 standing in the notch
/// at `(8.5, 5)`, clear of the plate.
///
/// The tab's shape the other way round: the notch's corners are the tab's, the
/// pin's circle on the plate's floor is inside their square, and the corners
/// alone wound it once. Its lump was taken for a hole in the floor and every
/// fuse was refused `ResultIsNotManifold` (queue row Q1054). The notch's arc
/// bulges in past the circle, and the count follows it now.
///
/// The plate is `200 − 9π` and the pin `π`, and they share nothing: so the two
/// of them united, nothing in common, and each one less the other is itself.
#[test]
fn a_pin_standing_in_a_plates_round_notch_is_answered_under_every_rule() {
    use std::f64::consts::PI;
    for (rule, plate_first) in every_rule_both_ways() {
        let mut model = a_model();
        let plate = block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
        let notch = model
            .add_round_bar(3.0, 4.0, &shift(10.0, 5.0, -1.0))
            .expect("a bar");
        let notched = fuse_one(&mut model, plate, notch, Rule::Difference);
        let pin = model
            .add_round_bar(0.5, 4.0, &shift(8.5, 5.0, -1.0))
            .expect("a bar");
        let what = format!("{rule:?}, the plate first {plate_first}");
        let answer = fuse(&mut model, notched, pin, rule, plate_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let volumes: &[f64] = match (rule, plate_first) {
            (Rule::Union, _) => &[PI, 200.0 - 9.0 * PI],
            (Rule::Intersection, _) => &[],
            (Rule::Difference, true) => &[200.0 - 9.0 * PI],
            (Rule::Difference, false) => &[PI],
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_each_holds(&model, &answer, volumes, &what);
    }
}

/// A plate `10 × 10 × 2` with a square hole `[3, 7] × [3, 7]`, against a pin
/// of radius `0.5` and height 4 standing in the hole at `(5, 5)`, clear of the
/// plate.
///
/// The hole's wire is walked clockwise, with the plate on its left, so it
/// subtracts itself from the outline's count. The count turned it round as
/// well, wound the pin's circle on the floor twice, and every fuse was refused
/// `CannotTellTheCellsOfAFaceApart` naming the floor (queue row Q1054).
///
/// The plate is `168` and the pin `π`, and they share nothing.
#[test]
fn a_pin_standing_in_a_plates_square_hole_is_answered_under_every_rule() {
    use std::f64::consts::PI;
    for (rule, plate_first) in every_rule_both_ways() {
        let mut model = a_model();
        let plate = block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
        let hole = block(&mut model, [3.0, 3.0, -1.0], [7.0, 7.0, 3.0]);
        let holed = fuse_one(&mut model, plate, hole, Rule::Difference);
        let pin = model
            .add_round_bar(0.5, 4.0, &shift(5.0, 5.0, -1.0))
            .expect("a bar");
        let what = format!("{rule:?}, the plate first {plate_first}");
        let answer = fuse(&mut model, holed, pin, rule, plate_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let volumes: &[f64] = match (rule, plate_first) {
            (Rule::Union, _) => &[PI, 168.0],
            (Rule::Intersection, _) => &[],
            (Rule::Difference, true) => &[168.0],
            (Rule::Difference, false) => &[PI],
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_each_holds(&model, &answer, volumes, &what);
    }
}

/// The plate with a round tab of
/// `a_pin_through_a_plates_round_tab_is_answered_under_every_rule`, with a
/// round hole of radius 1 at `(3, 5)` as well, against the same pin through
/// the tab.
///
/// The hole's circle is one side that closes on its own corner, with no chord,
/// so the floor's rim is not counted round the pin's circle and the lump is
/// placed by the walks round it instead. Every fuse is refused later, in
/// selection, naming the plate's top.
#[test]
fn a_pin_through_the_round_tab_of_a_plate_with_a_round_hole_is_refused_in_selection() {
    for (rule, plate_first) in every_rule_both_ways() {
        let mut model = a_model();
        let plate = block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
        let tab = model
            .add_round_bar(3.0, 2.0, &shift(10.0, 5.0, 0.0))
            .expect("a bar");
        let tabbed = fuse_one(&mut model, plate, tab, Rule::Union);
        let hole = model
            .add_round_bar(1.0, 4.0, &shift(3.0, 5.0, -1.0))
            .expect("a bar");
        let holed = fuse_one(&mut model, tabbed, hole, Rule::Difference);
        let pin = model
            .add_round_bar(0.5, 4.0, &shift(11.5, 5.0, -1.0))
            .expect("a bar");
        let refused = fuse(&mut model, holed, pin, rule, plate_first);
        let Err(Error::AFaceTrimmedByACurveThisCannotFollow { face }) = refused else {
            panic!("{rule:?}, the plate first {plate_first}: {refused:?}");
        };
        let surface = model
            .face(face)
            .expect("the refusal names a face of the model")
            .surface();
        assert!(
            matches!(surface, Surface::Plane(top) if top.origin().coordinates()[2] == 2.0),
            "{rule:?}, the plate first {plate_first}: refused on {surface:?}"
        );
    }
}

/// The chip of
/// `a_chip_of_a_cone_beside_a_tent_cannot_tell_the_tents_two_walks_apart`, cut
/// from a frustum of slope 1 from `z = 1/2` to `8` instead of described: the
/// frustum, seamed down `−x` so that the seam stays clear of the chip, against
/// the prism along `y` whose face lies on `2x − z = 1` and then the one whose
/// face lies on `2x + z = 11`, the rest of each prism clear of the frustum.
///
/// The first cut answers one solid. The second is refused
/// `AFaceTrimmedByACurveThisCannotFollow` naming a face on the cone, both ways
/// round; only the intersection is asked, since the chip is what it is for.
/// Cutting with one prism holding both faces is refused `ResultIsNotManifold`
/// (`a_frustum_under_two_steep_planes_is_refused_as_not_a_manifold`). Not
/// traced.
#[test]
fn a_chip_cut_from_a_frustum_in_two_steps_is_refused_on_its_cone_face_in_selection() {
    for chip_first in [true, false] {
        let mut model = a_model();
        let cone = cones::a_frustum_seamed_towards(
            &mut model,
            1.0,
            0.5,
            8.0,
            [-1.0, 0.0],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a frustum");
        let rising = a_prism_along_y(
            &mut model,
            [[-4.5, -10.0], [20.0, -10.0], [9.0, 17.0]],
            10.0,
        );
        let half = fuse_one(&mut model, cone, rising, Rule::Intersection);
        let falling = a_prism_along_y(
            &mut model,
            [[-20.0, -10.0], [10.5, -10.0], [-3.0, 17.0]],
            10.0,
        );
        let refused = fuse(&mut model, half, falling, Rule::Intersection, chip_first);
        let Err(Error::AFaceTrimmedByACurveThisCannotFollow { face }) = refused else {
            panic!("the half chip first {chip_first}: {refused:?}");
        };
        let surface = model
            .face(face)
            .expect("the refusal names a face of the model")
            .surface();
        assert!(
            matches!(surface, Surface::Cone(_)),
            "the half chip first {chip_first}: refused on {surface:?}"
        );
    }
}

/// A frustum of slope 1 from `z = 1/2` to `16`, and the pointed cone of slope 1
/// to `16`, each cut by the prism along `y` (`|y| ≤ 20`) whose faces lie on
/// `2x + z = 3` and `8x + 3z = −3`; and the frustum from `1/2` to `8` cut by
/// the prism whose faces lie on `2x − z = 1` and `2x + z = 11`, the chip of
/// `a_chip_of_a_cone_beside_a_tent_cannot_tell_the_tents_two_walks_apart`.
///
/// Each plane leans more steeply than the cone's side and cuts a hyperbola.
/// The first pair meet in a ridge at `(−6, y, 15)` and cut away the whole top
/// of the cone, dipping to `z = 1` and `z = 0.6` below its two ends; the second
/// meet in a chord at `(3, y, 5)` inside it. Every one of these intersections is
/// refused `ResultIsNotManifold { count: 4 }`, both ways round. The same roof
/// over a frustum from `z = 2`, which both hyperbolas dip below, answers
/// (`a_roofed_frustum_beside_a_plate_with_a_triangular_hole_never_draws_the_hole`).
/// Not traced.
#[test]
fn a_frustum_under_two_steep_planes_is_refused_as_not_a_manifold() {
    let roof = [[-6.0, 15.0], [0.0, -1.0], [2.0, -1.0]];
    let chip = [[3.0, 5.0], [-0.5, -2.0], [6.5, -2.0]];
    for (what, cone_is_pointed, near, far, prism, half) in [
        ("a frustum under the roof", false, 0.5, 16.0, roof, 20.0),
        ("a pointed cone under the roof", true, 0.0, 16.0, roof, 20.0),
        ("a frustum cut to the chip", false, 0.5, 8.0, chip, 10.0),
    ] {
        for cone_first in [true, false] {
            let mut model = a_model();
            let cone = if cone_is_pointed {
                cones::a_pointed_cone(&mut model, 1.0, far)
            } else {
                cones::a_frustum(&mut model, 1.0, near, far)
            };
            let prism = a_prism_along_y(&mut model, prism, half);
            let refused = fuse(&mut model, cone, prism, Rule::Intersection, cone_first);
            assert!(
                matches!(refused, Err(Error::ResultIsNotManifold { count: 4 })),
                "{what}, the cone first {cone_first}: {refused:?}"
            );
        }
    }
}

/// The frustum of slope 1 from `z = 2` to `16` under the roof of
/// `a_frustum_under_two_steep_planes_is_refused_as_not_a_manifold`, which
/// answers, against a plate from `z = 9/4` to `20` over `[−30, 30]²` with a
/// triangular hole `(3/10, 10)`, `(1/10, 53/5)`, `(0, 11)`.
///
/// Both of the roof's hyperbolas dip below the bottom cap, so the cut frustum's
/// cone face is two pieces, one each side of `y = 0`. The hole's three walls
/// lean towards `+x` by less than the roof's other plane leaves room for, so
/// each cuts a hyperbola on the cone wholly above the roof, and the three cut a
/// triangle there about `z = 10`, on the face's box and off the face: a walk that
/// goes round nothing in a lump of its own. The probe printed each wall's
/// crossing with two nodes and one arc, and then no walk of that lump: it has no
/// side of the face in it and no curve that closes, so
/// `drop_the_lumps_of_straight_cuts_alone` throws it away, and §340.2 is never
/// asked. Every fuse is refused later, in selection, naming a face on the cone.
/// Not traced.
#[test]
fn a_roofed_frustum_beside_a_plate_with_a_triangular_hole_never_draws_the_hole() {
    for (rule, roofed_first) in every_rule_both_ways() {
        let mut model = a_model();
        let cone = cones::a_frustum(&mut model, 1.0, 2.0, 16.0);
        let roof = a_prism_along_y(&mut model, [[-6.0, 15.0], [0.0, -1.0], [2.0, -1.0]], 20.0);
        let roofed = fuse_one(&mut model, cone, roof, Rule::Intersection);
        let profile = ProfileBuilder::new(vec![
            [-30.0, -30.0],
            [30.0, -30.0],
            [30.0, 30.0],
            [-30.0, 30.0],
        ])
        .with_hole(vec![[0.3, 10.0], [0.1, 10.6], [0.0, 11.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with a hole");
        let plate = model
            .add_extrusion(&profile, 17.75, &shift(0.0, 0.0, 2.25))
            .expect("a plate");
        let refused = fuse(&mut model, roofed, plate, rule, roofed_first);
        let Err(Error::AFaceTrimmedByACurveThisCannotFollow { face }) = refused else {
            panic!("{rule:?}, the roofed frustum first {roofed_first}: {refused:?}");
        };
        let surface = model
            .face(face)
            .expect("the refusal names a face of the model")
            .surface();
        assert!(
            matches!(surface, Surface::Cone(_)),
            "{rule:?}, the roofed frustum first {roofed_first}: refused on {surface:?}"
        );
    }
}

/// Three shapes built on the way that never reach the arrangement's cells: a
/// ball against a pipe crosses in a quartic, which nothing orders (§34.10).
///
/// - the ball of radius 2 against the square rod with a hole of radius `0.3`
///   down its axis, and down `(0.4, 0.3)`;
/// - a bar of radius 2, height 10, against a slab `|x| < 1` along it and a ball
///   of radius `0.5` on its wall at `(2, 0, 5)`, which was to be a walk going
///   round nothing on a pipe.
#[test]
fn a_ball_against_a_pipe_is_refused_before_any_cell() {
    for (rule, first) in every_rule_both_ways() {
        for hole in [[0.0, 0.0], [0.4, 0.3]] {
            let mut model = a_model();
            let ball = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
            let rod = block(&mut model, [-1.0, -1.0, -5.0], [1.0, 1.0, 5.0]);
            let bar = model
                .add_round_bar(0.3, 12.0, &shift(hole[0], hole[1], -6.0))
                .expect("a bar");
            let holed = fuse_one(&mut model, rod, bar, Rule::Difference);
            let refused = fuse(&mut model, ball, holed, rule, first);
            assert!(
                matches!(refused, Err(Error::ACrossingThisCannotBeOrderedAlong)),
                "the rod holed at {hole:?}, {rule:?}, the ball first {first}: {refused:?}"
            );
        }

        let mut model = a_model();
        let bar = model
            .add_round_bar(2.0, 10.0, &Isometry::identity())
            .expect("a bar");
        let slab = block(&mut model, [-1.0, -5.0, -1.0], [1.0, 5.0, 11.0]);
        let small = model
            .add_ball(0.5, &shift(2.0, 0.0, 5.0))
            .expect("a small ball");
        let (arguments, tools) = if first {
            (vec![bar], vec![slab, small])
        } else {
            (vec![slab, small], vec![bar])
        };
        let refused = model.fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            matches!(refused, Err(Error::ACrossingThisCannotBeOrderedAlong)),
            "the bar, the slab and the ball, {rule:?}, the bar first {first}: {:?}",
            refused.map(|fused| fused.solids().to_vec())
        );
    }
}

/// Each rule, with `true` for the first solid built named first.
fn every_rule_both_ways() -> Vec<(Rule, bool)> {
    [Rule::Union, Rule::Intersection, Rule::Difference]
        .into_iter()
        .flat_map(|rule| [(rule, true), (rule, false)])
        .collect()
}

/// Every rule both ways round refuses `CannotTellTheCellsOfAFaceApart`, on a
/// face of the first solid built whose surface `on` accepts.
fn every_rule_both_ways_refuses_on_a(
    what: &str,
    build: impl Fn() -> (Model, SolidKey, SolidKey),
    on: impl Fn(Surface) -> bool,
) {
    for (rule, first) in every_rule_both_ways() {
        let (mut model, one, other) = build();
        let refused = fuse(&mut model, one, other, rule, first);
        let Err(Error::CannotTellTheCellsOfAFaceApart { face }) = refused else {
            panic!("{what}, {rule:?}, the first built first {first}: {refused:?}");
        };
        let surface = model
            .face(face)
            .expect("the refusal names a face of the model")
            .surface();
        assert!(
            on(surface),
            "{what}, {rule:?}, the first built first {first}: refused on {surface:?}"
        );
    }
}

fn fuse(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    one_first: bool,
) -> Result<Vec<SolidKey>, Error> {
    let (arguments, tools) = if one_first {
        (one, other)
    } else {
        (other, one)
    };
    model
        .fuse(
            &[arguments],
            &[tools],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec())
}

/// `answer` is one solid per entry of `volumes`, smallest first, each holding
/// that volume.
fn assert_each_holds(model: &Model, answer: &[SolidKey], volumes: &[f64], what: &str) {
    assert_eq!(
        answer.len(),
        volumes.len(),
        "{what}: {} solids, got {answer:?}",
        volumes.len()
    );
    let mut by_size: Vec<(f64, SolidKey)> = answer
        .iter()
        .map(|&solid| (volume_of(model, solid, what), solid))
        .collect();
    by_size.sort_by(|one, other| one.0.total_cmp(&other.0));
    for ((_, solid), &volume) in by_size.into_iter().zip(volumes) {
        assert_holds(model, solid, volume, what);
    }
}

fn volume_of(model: &Model, solid: SolidKey, what: &str) -> f64 {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} does not measure: {why}"))
        .volume()
}

/// `solid` validates and measures `volume`, give or take what the mesh says.
fn assert_holds(model: &Model, solid: SolidKey, volume: f64, what: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{what} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The one solid a fuse that builds a shape gives.
fn fuse_one(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let answer = fuse(model, one, other, rule, true)
        .unwrap_or_else(|why| panic!("building the shape: {why:?}"));
    let [solid] = answer[..] else {
        panic!("one solid, got {answer:?}");
    };
    solid
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(1.0).expect("1 mm is a working size"))
}

fn shift(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

/// A block from `low` to `high`, a rectangle extruded up `z`.
fn block(model: &mut Model, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, high[2] - low[2], &shift(0.0, 0.0, low[2]))
        .expect("a block")
}

/// The part of the solid cone `r ≤ z` where `2x − z ≥ 1` and `2x + z ≤ 11`,
/// described face by face: a face on the cone bounded by an arc of the
/// hyperbola each plane cuts, from `(3, 4, 5)` to `(3, −4, 5)`, and a flat face
/// on each plane between its arc and the chord the two planes meet along.
///
/// The two corners are exact on the cone (`3² + 4² = 5²`) and on both planes.
/// Each arc is picked out of its plane's two branches, one on each nappe, by
/// the half-space `z ≥ 0`.
fn a_chip_of_a_cone(model: &mut Model) -> SolidKey {
    let skin = Cone::new(Axis::z(), 1.0).expect("a positive slope");
    let rising =
        Plane::from_point_and_normal(Point::new(3.0, 4.0, 5.0), Vector::new(-2.0, 0.0, 1.0))
            .expect("a plane");
    let falling =
        Plane::from_point_and_normal(Point::new(3.0, 4.0, 5.0), Vector::new(2.0, 0.0, 1.0))
            .expect("a plane");
    let upper = HalfSpace::through_a_point_facing(Point::origin(), Vector::new(0.0, 0.0, 1.0))
        .expect("a half-space");
    let on_the_upper_nappe = |plane| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(plane),
        picked_by: smallvec::smallvec![Picker::from(upper)],
    };

    let mut builder = SolidBuilder::new();
    let left = builder
        .add_vertex(Point::new(3.0, 4.0, 5.0))
        .expect("a corner");
    let right = builder
        .add_vertex(Point::new(3.0, -4.0, 5.0))
        .expect("a corner");
    let low = builder
        .add_edge(right, left, on_the_upper_nappe(rising))
        .expect("an arc");
    let high = builder
        .add_edge(left, right, on_the_upper_nappe(falling))
        .expect("an arc");
    let chord = builder
        .add_edge(left, right, EdgeRun::Straight)
        .expect("a chord");
    let face = |builder: &mut SolidBuilder, surface, steps: [_; 2]| {
        let steps: Vec<DraftStep> = steps
            .into_iter()
            .map(|(edge, way)| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")))
            .collect();
        let wire = builder.add_wire(&steps).expect("a wire");
        builder.add_face(surface, &[wire]).expect("a face")
    };
    let lens = face(
        &mut builder,
        Surface::Cone(skin),
        [(low, Orientation::Forward), (high, Orientation::Forward)],
    );
    let under = face(
        &mut builder,
        Surface::Plane(rising),
        [(low, Orientation::Reversed), (chord, Orientation::Reversed)],
    );
    let over = face(
        &mut builder,
        Surface::Plane(falling),
        [(high, Orientation::Reversed), (chord, Orientation::Forward)],
    );
    builder
        .add_shell(&[
            (lens, Orientation::Forward),
            (under, Orientation::Forward),
            (over, Orientation::Forward),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a chip of a cone")
}

/// The volume of [`a_chip_of_a_cone`], by Simpson's rule over its slices.
///
/// The slice at height `z` is the disc `r ≤ z` between `x = (1 + z)/2` and
/// `x = (11 − z)/2`, nonempty for `1 ≤ z ≤ 5`, and the second bound cuts the
/// disc only from `z = 11/3` up. The part of a disc of radius `r` past `x = a`
/// is `r² acos(a/r) − a √(r² − a²)`. The slice area is smooth on each side of
/// `11/3`, so each side is summed on its own.
fn the_chips_volume() -> f64 {
    let past = |radius: f64, from: f64| {
        if from >= radius {
            0.0
        } else {
            radius * radius * (from / radius).acos() - from * (radius * radius - from * from).sqrt()
        }
    };
    let slice = |z: f64| past(z, (1.0 + z) / 2.0) - past(z, (11.0 - z) / 2.0);
    let simpson = |low: f64, high: f64| {
        let steps = 20_000;
        let step = (high - low) / f64::from(steps);
        let inner: f64 = (1..steps)
            .map(|at| {
                let weight = if at % 2 == 1 { 4.0 } else { 2.0 };
                weight * slice(low + f64::from(at) * step)
            })
            .sum();
        (slice(low) + inner + slice(high)) * step / 3.0
    };
    simpson(1.0, 11.0 / 3.0) + simpson(11.0 / 3.0, 5.0)
}

/// The tent of
/// `a_chip_of_a_cone_beside_a_tent_cannot_tell_the_tents_two_walks_apart`: a
/// prism along `y`, `|y| ≤ 10`, over the triangle `(−11/16, 3/4)`,
/// `(−11/8, 7/4)`, `(−13/16, 3/4)`.
///
/// Its walls lie on `x + 11z/16 = −11/64` and `x + 9z/16 = −25/64`. Each leans
/// more steeply than the cone's side, and on the cone each is a hyperbola
/// round the `−x` side alone, `46.6°` and `55.8°` either side of it: neither
/// comes within `120°` of `+x`, where the chip's wider hyperbola reaches, so
/// neither crosses the chip's. The first dips to `z = 0.55`, below the floor's
/// circle at `3/4`; the second to `0.89`, above it.
fn a_tent(model: &mut Model) -> SolidKey {
    a_prism_along_y(
        model,
        [[-0.6875, 0.75], [-1.375, 1.75], [-0.8125, 0.75]],
        10.0,
    )
}

/// A prism along `y` from `−half` to `half` over a triangle in the `xz` plane,
/// its corners listed anticlockwise seen from `−y`.
fn a_prism_along_y(model: &mut Model, [a, b, c]: [[f64; 2]; 3], half: f64) -> SolidKey {
    let at = |[x, z]: [f64; 2], y: f64| [x, y, z];
    a_polyhedron(
        model,
        &[
            at(a, -half),
            at(b, -half),
            at(c, -half),
            at(a, half),
            at(b, half),
            at(c, half),
        ],
        &[
            &[0, 1, 2],
            &[3, 5, 4],
            &[0, 3, 4, 1],
            &[1, 4, 5, 2],
            &[2, 5, 3, 0],
        ],
    )
}

/// A solid with flat faces through `corners`, each face listed anticlockwise
/// seen from outside, so that its first three corners give its outward normal.
fn a_polyhedron(model: &mut Model, corners: &[[f64; 3]], faces: &[&[usize]]) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let vertices: Vec<_> = corners
        .iter()
        .map(|&[x, y, z]| builder.add_vertex(Point::new(x, y, z)).expect("a corner"))
        .collect();
    let mut edges = std::collections::BTreeMap::new();
    let mut drafted = Vec::new();
    for face in faces {
        let mut steps = Vec::new();
        for (at, &from) in face.iter().enumerate() {
            let to = face[(at + 1) % face.len()];
            let ends = (from.min(to), from.max(to));
            let edge = *edges.entry(ends).or_insert_with(|| {
                builder
                    .add_edge(vertices[ends.0], vertices[ends.1], EdgeRun::Straight)
                    .expect("an edge")
            });
            let way = if from < to {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            steps.push(DraftStep::Along(
                builder.add_coedge(edge, way).expect("a coedge"),
            ));
        }
        let wire = builder.add_wire(&steps).expect("a wire");
        let [a, b, c] = [face[0], face[1], face[2]].map(|index| corners[index]);
        let along = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let next = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
        let outward = Vector::new(
            along[1] * next[2] - along[2] * next[1],
            along[2] * next[0] - along[0] * next[2],
            along[0] * next[1] - along[1] * next[0],
        );
        let plane = Plane::from_point_and_normal(Point::new(a[0], a[1], a[2]), outward)
            .expect("three corners not in a line");
        drafted.push(
            builder
                .add_face(Surface::Plane(plane), &[wire])
                .expect("a face"),
        );
    }
    let shell: Vec<_> = drafted
        .into_iter()
        .map(|face| (face, Orientation::Forward))
        .collect();
    builder.add_shell(&shell).expect("the faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a polyhedron")
}
