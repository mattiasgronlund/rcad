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
//! (queue row Q1053), and every fuse holds the volume its doc works out. **So
//! is the one on a ball**, since a ball's skin is opened out flat from a point
//! of the walk and wound round there (queue row Q1242), and so is a second
//! shape on a ball that reached it, a square rod holed down its own axis. The
//! one on a cone still reaches it.
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
//!   it was refused before that, and answers now (two shapes below, queue row
//!   Q1176); described face by face it reaches it.
//!
//! The other shapes built on the way are pinned beside them, with what each
//! gives instead.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Sphere, Surface, Torus, Vector,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, Error, FaceKey, Model, OnAFace, ProfileBuilder, Rule, SolidBuilder, SolidKey,
};

mod cones;

// Shared with `fuses_that_must_agree.rs` and `fuzzing/fuse.rs`, which each
// reach the same file by `#[path]` from their own directory (§129.5 item 3).
#[path = "fuzzing/comparison.rs"]
mod comparison;

use comparison::{answer_of, same_answer};

/// A ball of radius 2 at the origin, less a ball of radius `0.3` at
/// `(0.5, 0.25, 1.9)`, against the square rod `|x|, |y| < 1`, `|z| < 5`.
///
/// The rod's four walls cut the big ball in four circles that cross each other
/// where its edges go through the ball, which is one lump with a patch inside
/// all four at each end. The dimple's circle is a lump of its own in the top
/// patch. Its outer walk agrees with both patches about every circle, the face
/// is a ball's, and the dimple's lump has the face's own boundary in it, so it
/// is wound once and not left out. That outer walk was refused
/// `CannotTellTheCellsOfAFaceApart` under every rule both ways round until a
/// ball's skin was opened out flat from a node of the dimple's circle, where
/// the patch holding it is the one whose walk goes round the far patch's
/// corners (queue row Q1242).
///
/// **The answer.** With `Q` [`the_square_part_of_the_ball`] and `L` the lens
/// the two balls share, the dimpled ball is `32π/3 − L`, the rod `40`, and the
/// two share `Q − L`, since the small ball lies inside the rod. Each answer
/// is one solid but the rod less the ball, which leaves the rod's two ends,
/// the top one holding the lens as well.
#[test]
fn a_dimpled_ball_through_a_square_rod_is_answered_under_every_rule() {
    use std::f64::consts::PI;
    let (ball, square) = (32.0 * PI / 3.0, the_square_part_of_the_ball());
    let lens = the_lens_two_balls_share(2.0, 0.3, (0.5f64 * 0.5 + 0.25 * 0.25 + 1.9 * 1.9).sqrt());
    let end = (40.0 - square) / 2.0;
    for (rule, ball_first) in every_rule_both_ways() {
        let mut model = a_model();
        let big = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
        let small = model
            .add_ball(0.3, &shift(0.5, 0.25, 1.9))
            .expect("a small ball");
        let dimpled = fuse_one(&mut model, big, small, Rule::Difference);
        let rod = block(&mut model, [-1.0, -1.0, -5.0], [1.0, 1.0, 5.0]);
        let what = format!("{rule:?}, the dimpled ball first {ball_first}");
        let answer = fuse(&mut model, dimpled, rod, rule, ball_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let volumes = match (rule, ball_first) {
            (Rule::Union, _) => vec![ball + 40.0 - square],
            (Rule::Intersection, _) => vec![square - lens],
            (Rule::Difference, true) => vec![ball - square],
            (Rule::Difference, false) => vec![end, end + lens],
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_each_holds(&model, &answer, &volumes, &what);
    }
}

/// The part of the ball of radius 2 at the origin inside the square rod
/// `|x|, |y| < 1`: `∫∫ 2√(4 − x² − y²)` over the square, which works out to
/// `76π/3 − 72·atan √2 + 8√2/3`.
fn the_square_part_of_the_ball() -> f64 {
    use std::f64::consts::PI;
    let root_two = 2.0f64.sqrt();
    76.0 * PI / 3.0 - 72.0 * root_two.atan() + 8.0 * root_two / 3.0
}

/// The lens two balls of radius `big` and `small` share, their centres
/// `apart` from each other: `π (big + small − apart)² (apart² + 2·apart·small
/// − 3 small² + 2·apart·big + 6 small·big − 3 big²) / (12 apart)`.
fn the_lens_two_balls_share(big: f64, small: f64, apart: f64) -> f64 {
    use std::f64::consts::PI;
    let overlap = big + small - apart;
    PI * overlap
        * overlap
        * (apart * apart + 2.0 * apart * small - 3.0 * small * small
            + 2.0 * apart * big
            + 6.0 * small * big
            - 3.0 * big * big)
        / (12.0 * apart)
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
/// fuse was refused later, in selection, by a ray that met the face its arcs
/// trim (§35), until `holds.rs` followed a ball's skin cut by any two planes
/// (queue row Q1162). Every rule, both ways round, now answers one solid.
///
/// The small ball, `π/6`, lies in `x ≤ −0.7`, `y ≤ −1.1`, clear of both cuts,
/// with its centre on the big ball's skin. The two skins cross on the plane
/// `1.9375` along the line of centres, so they share the big ball's cap past it,
/// `π · 0.0625² · (6 − 0.0625)/3`, and the small ball's cap on the near side,
/// `π · 0.4375² · (1.5 − 0.4375)/3`: `29π/384` in all. The cut ball is
/// [`a_ball_cut_to_x_and_y_at_one`].
#[test]
fn a_ball_face_bounded_by_two_arcs_places_a_hole_by_the_arcs_own_circles() {
    use std::f64::consts::PI;
    let cut = a_ball_cut_to_x_and_y_at_one();
    let small = PI / 6.0;
    let shared = 29.0 * PI / 384.0;
    for (rule, cut_ball_first) in every_rule_both_ways() {
        let mut model = a_model();
        let ball = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
        let corner = block(&mut model, [-10.0, -10.0, -10.0], [1.0, 1.0, 10.0]);
        let cut_ball = fuse_one(&mut model, ball, corner, Rule::Intersection);
        assert_holds(&model, cut_ball, cut, "the cut ball");
        let small_ball = model
            .add_ball(0.5, &shift(-1.2, -1.6, 0.0))
            .expect("a small ball");
        let what = format!("{rule:?}, the cut ball first {cut_ball_first}");
        let answer = fuse(&mut model, cut_ball, small_ball, rule, cut_ball_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let [solid] = answer[..] else {
            panic!("{what}: one solid, got {answer:?}");
        };
        let volume = match (rule, cut_ball_first) {
            (Rule::Union, _) => cut + small - shared,
            (Rule::Intersection, _) => shared,
            (Rule::Difference, true) => cut - shared,
            (Rule::Difference, false) => small - shared,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(&model, solid, volume, &what);
    }
}

/// A ball of radius 2 at the origin cut to `x ≤ 1`, `y ≤ 1`, worked out slice
/// by slice up `z`.
///
/// The slice at height `z` is a disc of radius `ρ = √(4 − z²)` less the part
/// past `x = 1`, less the part past `y = 1`, plus the corner past both, which
/// was taken away twice. Past a line `p` from the centre is
/// `ρ² acos(p/ρ) − p √(ρ² − p²)` while `p < ρ`, and the corner past both is
/// `∫ (√(ρ² − x²) − 1) dx` from `x = 1` to `√(ρ² − 1)` while `ρ² > 2`. The
/// slice area is smooth between `ρ = √2` and `ρ = 1`, where the corner and then
/// the parts past each line run out, so each stretch is summed on its own.
fn a_ball_cut_to_x_and_y_at_one() -> f64 {
    use std::f64::consts::PI;
    let past = |radius: f64, from: f64| {
        if from >= radius {
            0.0
        } else {
            radius * radius * (from / radius).acos() - from * (radius * radius - from * from).sqrt()
        }
    };
    let corner = |radius: f64| {
        if radius * radius <= 2.0 {
            return 0.0;
        }
        let up_to = |x: f64| {
            (x * (radius * radius - x * x).sqrt() + radius * radius * (x / radius).asin()) / 2.0
        };
        let far = (radius * radius - 1.0).sqrt();
        up_to(far) - up_to(1.0) - (far - 1.0)
    };
    let slice = |z: f64| {
        let radius = (4.0 - z * z).max(0.0).sqrt();
        PI * radius * radius - 2.0 * past(radius, 1.0) + corner(radius)
    };
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
    let [two, three] = [2.0f64.sqrt(), 3.0f64.sqrt()];
    2.0 * (simpson(0.0, two) + simpson(two, three) + simpson(three, 2.0))
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
/// placed by the walks round it instead. Every fuse was refused later, in
/// selection, naming the plate's top (queue row Q1165): a ray from a point of
/// the pin's wall met the top's plane at `(14.19, 7.04, 2)`, and `holds.rs`
/// read the top's sides to wind them, the tab's arc among them, and refused the
/// hole's circle, a side with no two ends to read a chord between. It counts
/// that circle as a whole now, the way it counts the hole of a top with
/// straight sides.
///
/// The plate is `200` and the tab's half disc past `x = 10` is `9π`, less the
/// hole's `2π`: `200 + 7π`. The pin is `π`, and it lies inside the tab's
/// circle (`1.5 + 0.5 < 3`), so they share its stretch from `z = 0` to `2`,
/// `π/2`. So `200 + 7.5π` united, `π/2` in common, `200 + 6.5π` the plate less
/// the pin, and the pin less the plate is its two ends, `π/4` each.
#[test]
fn a_pin_through_the_round_tab_of_a_plate_with_a_round_hole_is_answered_under_every_rule() {
    use std::f64::consts::PI;
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
        assert_holds(&model, holed, 200.0 + 7.0 * PI, "the plate");
        let what = format!("{rule:?}, the plate first {plate_first}");
        let answer = fuse(&mut model, holed, pin, rule, plate_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let volumes: &[f64] = match (rule, plate_first) {
            (Rule::Union, _) => &[200.0 + 7.5 * PI],
            (Rule::Intersection, _) => &[PI / 2.0],
            (Rule::Difference, true) => &[200.0 + 6.5 * PI],
            (Rule::Difference, false) => &[PI / 4.0, PI / 4.0],
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_each_holds(&model, &answer, volumes, &what);
    }
}

/// The top of the plate of
/// `a_pin_through_the_round_tab_of_a_plate_with_a_round_hole_is_answered_under_every_rule`
/// holds what its outline, tab and hole say, the hole's circle counted as a
/// whole beside the tab's arc (queue row Q1165). Each point is on the top's
/// plane `z = 2`. The hole's centre is inside the outline and inside the hole,
/// so it is held only if the hole is taken away; `(12, 5)` is in the tab and
/// past the square; `(14.19, 7.04)` is where the fuse's ray met the plane, past
/// the tab; `(4, 5)` is on the hole's circle.
#[test]
fn the_top_of_a_plate_with_a_round_tab_and_a_round_hole_holds_neither_the_hole_nor_past_the_tab() {
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
    let solid = model.solid(holed).expect("this model made the key");
    let top = solid
        .faces()
        .find(|face| {
            matches!(face.surface(), Surface::Plane(flat) if flat.origin().coordinates()[2] == 2.0)
        })
        .expect("the plate has a top")
        .key();
    for (x, y, held) in [
        (3.0, 5.0, OnAFace::NotHeld),
        (6.0, 5.0, OnAFace::Held),
        (12.0, 5.0, OnAFace::Held),
        (14.19, 7.04, OnAFace::NotHeld),
        (4.0, 5.0, OnAFace::OnItsBoundary),
    ] {
        let answer = solid
            .face_holds_the_point(
                top,
                PredicatePoint::from([x, y, 2.0]),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("({x}, {y}): {why:?}"));
        assert_eq!(answer, held, "({x}, {y})");
    }
}

/// The chip of
/// `a_chip_of_a_cone_beside_a_tent_cannot_tell_the_tents_two_walks_apart`, cut
/// from a frustum of slope 1 from `z = 1/2` to `8` instead of described: the
/// frustum, seamed down `−x` so that the seam stays clear of the chip, against
/// the prism along `y` whose face lies on `2x − z = 1` and then the one whose
/// face lies on `2x + z = 11`, the rest of each prism clear of the frustum.
///
/// The first cut answers one solid, the half chip. The second was refused
/// `AFaceTrimmedByACurveThisCannotFollow` in selection, both ways round (queue
/// row Q1177): a ray from a point of the second prism's face met the half
/// chip's skin, and `holds.rs` would not follow a cone face cut by one plane
/// the apex is behind that does not close, `2x − z ≥ 1`, beside the top
/// `z ≤ 8`. It does now, since a cut square to the axis keeps the rulings
/// nearest the branch's vertex. The probe then printed selection keeping
/// exactly the chip's three pieces, the lens on the cone and one piece on each
/// plane, each with the two corners `(3, ±4, 5)`, and the fuse was refused
/// `ResultIsNotManifold { count: 4 }` past selection, both ways round, for the
/// reason cutting with one prism holding both faces was
/// (`a_frustum_under_two_steep_planes_is_answered_under_every_rule`, queue row
/// Q1176): the lens's two arcs were one edge.
///
/// **The answers.** The half chip is the frustum past `2x − z = 1`,
/// [`the_half_chips_volume`]. The second prism is the triangle `30.5 · 27 / 2`
/// along `|y| ≤ 10`, `8235`. The two share [`the_chips_volume`]. Each fuse is
/// one solid.
#[test]
fn a_chip_cut_from_a_frustum_in_two_steps_is_answered_under_every_rule() {
    let (half_chip, falling_prism, chip) = (the_half_chips_volume(), 8235.0, the_chips_volume());
    for (rule, half_first) in every_rule_both_ways() {
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
        let what = format!("{rule:?}, the half chip first {half_first}");
        let answer = fuse(&mut model, half, falling, rule, half_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let volume = match (rule, half_first) {
            (Rule::Union, _) => half_chip + falling_prism - chip,
            (Rule::Intersection, _) => chip,
            (Rule::Difference, true) => half_chip - chip,
            (Rule::Difference, false) => falling_prism - chip,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_each_holds(&model, &answer, &[volume], &what);
    }
}

/// A frustum of slope 1 from `z = 1/2` to `16`, and the pointed cone of slope 1
/// to `16`, each against the prism along `y` (`|y| ≤ 20`) whose faces lie on
/// `2x + z = 3` and `8x + 3z = −3`; and the frustum from `1/2` to `8` against
/// the prism whose faces lie on `2x − z = 1` and `2x + z = 11`, the chip of
/// `a_chip_of_a_cone_beside_a_tent_cannot_tell_the_tents_two_walks_apart`.
///
/// Each plane leans more steeply than the cone's side and cuts a hyperbola.
/// The first pair meet in a ridge at `(−6, y, 15)` and cut away the whole top
/// of the cone, dipping to `z = 1` and `z = 0.6` below its two ends; the second
/// meet in a chord at `(3, y, 5)` inside it. So the line each pair meets along
/// crosses the cone twice, at `(−6, ±√189, 15)` and at `(3, ±4, 5)`, and every
/// fuse keeps an arc of each plane's hyperbola between those two points. Named
/// by their two corners alone the two arcs were one edge, with the cone's face
/// along it twice and each plane's face once, and every fuse was refused
/// `ResultIsNotManifold { count: 4 }`, both ways round (queue row Q1176). The
/// same roof over a frustum from `z = 2`, which both hyperbolas dip below,
/// never met it: there the floor's circle cuts each hyperbola in two, and no
/// two arcs share both their corners
/// (`a_roofed_frustum_beside_a_plate_with_a_triangular_hole_is_answered_under_every_rule`).
///
/// **The answers.** The roof's prism is the triangle `2 · 16 / 2` along
/// `|y| ≤ 20`, `640`, and the chip's `7 · 7 / 2` along `|y| ≤ 10`, `490`. The
/// cone is `π(far³ − near³)/3`. What the two share is
/// [`the_roofed_frustum_between`] the cone's floor and the ridge, and
/// [`the_chips_volume`] for the chip, which lies between `z = 1` and `5`, inside
/// the frustum and inside `|y| ≤ 10`. Each fuse is one solid.
#[test]
fn a_frustum_under_two_steep_planes_is_answered_under_every_rule() {
    use std::f64::consts::PI;
    let roof = [[-6.0, 15.0], [0.0, -1.0], [2.0, -1.0]];
    let chip = [[3.0, 5.0], [-0.5, -2.0], [6.5, -2.0]];
    for (what, cone_is_pointed, near, far, prism, half, prism_holds, shared) in [
        (
            "a frustum under the roof",
            false,
            0.5,
            16.0,
            roof,
            20.0,
            640.0,
            the_roofed_frustum_between(0.5, 15.0),
        ),
        (
            "a pointed cone under the roof",
            true,
            0.0,
            16.0,
            roof,
            20.0,
            640.0,
            the_roofed_frustum_between(0.0, 15.0),
        ),
        (
            "a frustum cut to the chip",
            false,
            0.5,
            8.0,
            chip,
            10.0,
            490.0,
            the_chips_volume(),
        ),
    ] {
        let cone_holds = PI * (far * far * far - near * near * near) / 3.0;
        for (rule, cone_first) in every_rule_both_ways() {
            let mut model = a_model();
            let cone = if cone_is_pointed {
                cones::a_pointed_cone(&mut model, 1.0, far)
            } else {
                cones::a_frustum(&mut model, 1.0, near, far)
            };
            let prism = a_prism_along_y(&mut model, prism, half);
            let what = format!("{what}, {rule:?}, the cone first {cone_first}");
            let answer = fuse(&mut model, cone, prism, rule, cone_first)
                .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            let volume = match (rule, cone_first) {
                (Rule::Union, _) => cone_holds + prism_holds - shared,
                (Rule::Intersection, _) => shared,
                (Rule::Difference, true) => cone_holds - shared,
                (Rule::Difference, false) => prism_holds - shared,
                (other, _) => panic!("not a rule this probes: {other:?}"),
            };
            assert_each_holds(&model, &answer, &[volume], &what);
        }
    }
}

/// The frustum cut to the chip
/// (`a_frustum_under_two_steep_planes_is_answered_under_every_rule`'s third
/// row), union'd with its prism cone first and prism first, write out the
/// same solid bit for bit — compared the way `fuses_that_must_agree.rs`'s
/// `swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit` compares its
/// two orders. Both answers hold the volume that test already checks, `give
/// or take` a mesh error of its own; what this pins is that **the two give or
/// take different amounts, by about `1.4e-6`, because each run's mesh starts
/// from a different corner of the cone face**, not because the two runs built
/// different solids (queue row Q1328).
#[test]
fn a_frustum_and_a_prism_fused_both_ways_write_out_the_same_solid() {
    let chip = [[3.0, 5.0], [-0.5, -2.0], [6.5, -2.0]];
    let build = || {
        let mut model = a_model();
        let cone = cones::a_frustum(&mut model, 1.0, 0.5, 8.0);
        let prism = a_prism_along_y(&mut model, chip, 10.0);
        (model, cone, prism)
    };
    let measure = |model: &Model, solid: SolidKey| {
        model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.001).expect("a fine mesh"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a frustum fused with a prism measures")
    };

    let (mut cone_first_model, cone, prism) = build();
    let cone_first = fuse_one(&mut cone_first_model, cone, prism, Rule::Union);
    let cone_first_measured = measure(&cone_first_model, cone_first);

    let (mut prism_first_model, cone, prism) = build();
    let prism_first = fuse_one(&mut prism_first_model, prism, cone, Rule::Union);
    let prism_first_measured = measure(&prism_first_model, prism_first);

    if let Err(why) = same_answer(
        &answer_of(&cone_first_model, &[cone_first]),
        &answer_of(&prism_first_model, &[prism_first]),
        Rule::Union,
        "swapping the cone and the prism",
    ) {
        panic!("one solid fused both ways wrote out as two different answers: {why}");
    }

    let apart = (cone_first_measured.volume() - prism_first_measured.volume()).abs();
    assert!(
        apart > 0.0,
        "the two meshes agreed about the volume exactly; the mesh no longer starts from a \
         different corner each way round, so the doc comment above is stale"
    );
    assert!(
        apart <= cone_first_measured.volume_error() && apart <= prism_first_measured.volume_error(),
        "the two meshes' volumes differ by {apart}, more than either run's own \
         volume_error ({}, {}) says the mesh could be out",
        cone_first_measured.volume_error(),
        prism_first_measured.volume_error(),
    );
}

/// The frustum of slope 1 from `z = 2` to `16` under the roof of
/// `a_frustum_under_two_steep_planes_is_answered_under_every_rule`, which
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
/// asked.
///
/// Every fuse was refused `AFaceTrimmedByACurveThisCannotFollow` in selection,
/// naming one of the two pieces of skin (queue row Q1177): a ray from a point of
/// the plate's floor met it, and `holds.rs` would not follow a cone face cut by
/// a floor and two planes neither of which is side by side with it. The part
/// of the cone in front of all three is two pieces, one each side of `y = 0`:
/// the roof's two planes meet in a ridge along `y`, square to the axis, and
/// each crosses the floor's circle. Which of them a face is, its corners say,
/// as they say which run of a bar's wall between two flats a face is.
///
/// **The answer.** The hole is nowhere near the roofed frustum: a point of it
/// has `y ≥ 10`, so on the frustum `z ≥ 10`, and under `2x + z ≤ 3` with
/// `x ≥ 0`, `z ≤ 3`. The roofed frustum reaches no higher than its ridge at
/// `z = 15` and no further than 15 from the axis, so the plate, `17.75 ·
/// (3600 − 1/100)`, holds all of it above `z = 9/4`. The roofed frustum is the
/// integral over `2 ≤ z ≤ 15` of the disc `r ≤ z` between `x = −(3 + 3z)/8`
/// and `x = (3 − z)/2`, which [`the_roofed_frustum_between`] works out slice
/// by slice, below `9/4` and above it. So the union is the plate and the part
/// below, the intersection the part above, the roofed frustum less the plate
/// the part below, and the plate less it the plate less the part above: one
/// solid each.
#[test]
fn a_roofed_frustum_beside_a_plate_with_a_triangular_hole_is_answered_under_every_rule() {
    let plate = 17.75 * (3600.0 - 0.01);
    let below = the_roofed_frustum_between(2.0, 2.25);
    let above = the_roofed_frustum_between(2.25, 15.0);
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
        let holed = model
            .add_extrusion(&profile, 17.75, &shift(0.0, 0.0, 2.25))
            .expect("a plate");
        let what = format!("{rule:?}, the roofed frustum first {roofed_first}");
        let answer = fuse(&mut model, roofed, holed, rule, roofed_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let volume = match (rule, roofed_first) {
            (Rule::Union, _) => plate + below,
            (Rule::Intersection, _) => above,
            (Rule::Difference, true) => below,
            (Rule::Difference, false) => plate - above,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_each_holds(&model, &answer, &[volume], &what);
    }
}

/// The skins `holds.rs` would not follow before queue row Q1177 hold their own
/// points, asked through the public `face_holds_the_point`: the half chip of
/// `a_chip_cut_from_a_frustum_in_two_steps_is_answered_under_every_rule`, the
/// frustum cut to `2x − z ≥ 1` under its top `z ≤ 8`, and the two pieces of
/// skin of the roofed frustum of
/// `a_roofed_frustum_beside_a_plate_with_a_triangular_hole_is_answered_under_every_rule`.
///
/// Every point is on the cone `r = z` exactly. `(0, 5/2, 5/2)` and its mirror
/// `(0, −5/2, 5/2)` are in front of the roofed frustum's floor and of both its
/// roof's planes, one in each piece of skin, so each piece holds one of them
/// and not the other: read as the part in front of every cut, each piece would
/// hold both. `(−3, 4, 5)` is past the roof's plane `8x + 3z ≥ −3` and
/// `(3, 4, 5)` past `2x + z ≤ 3`, so neither piece holds them.
#[test]
fn the_skins_of_a_half_chip_and_a_roofed_frustum_hold_their_own_points() {
    let held = |model: &Model, solid: SolidKey, face, [x, y, z]: [f64; 3]| {
        model
            .solid(solid)
            .expect("this model made the key")
            .face_holds_the_point(
                face,
                PredicatePoint::from([x, y, z]),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("({x}, {y}, {z}): {why:?}"))
    };
    let skins = |model: &Model, solid: SolidKey| -> Vec<FaceKey> {
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .filter(|face| matches!(face.surface(), Surface::Cone(_)))
            .map(|face| face.key())
            .collect()
    };

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
    let [skin] = skins(&model, half)[..] else {
        panic!("the half chip has one face on the cone");
    };
    for (point, answer) in [
        ([4.0, 3.0, 5.0], OnAFace::Held),
        ([-4.0, 3.0, 5.0], OnAFace::NotHeld),
        ([6.0, 8.0, 10.0], OnAFace::NotHeld),
        ([4.0, 3.0, -5.0], OnAFace::NotHeld),
    ] {
        assert_eq!(
            held(&model, half, skin, point),
            answer,
            "the half chip at {point:?}"
        );
    }

    let mut model = a_model();
    let cone = cones::a_frustum(&mut model, 1.0, 2.0, 16.0);
    let roof = a_prism_along_y(&mut model, [[-6.0, 15.0], [0.0, -1.0], [2.0, -1.0]], 20.0);
    let roofed = fuse_one(&mut model, cone, roof, Rule::Intersection);
    let [one, other] = skins(&model, roofed)[..] else {
        panic!("the roofed frustum has two faces on the cone");
    };
    for skin in [one, other] {
        for point in [[-3.0, 4.0, 5.0], [3.0, 4.0, 5.0]] {
            assert_eq!(
                held(&model, roofed, skin, point),
                OnAFace::NotHeld,
                "the roofed frustum at {point:?}"
            );
        }
    }
    let answers = [one, other].map(|skin| {
        [[0.0, 2.5, 2.5], [0.0, -2.5, 2.5]].map(|point| held(&model, roofed, skin, point))
    });
    let either_way = [
        [
            [OnAFace::Held, OnAFace::NotHeld],
            [OnAFace::NotHeld, OnAFace::Held],
        ],
        [
            [OnAFace::NotHeld, OnAFace::Held],
            [OnAFace::Held, OnAFace::NotHeld],
        ],
    ];
    assert!(
        either_way.contains(&answers),
        "each piece of skin holds the point on its own side of y = 0: {answers:?}"
    );
}

/// The skin of [`a_chip_of_a_cone`] is the part of the nappe `r = z, z ≥ 0` in
/// front of both its cuts, `2x − z ≥ 1` and `2x + z ≤ 11`, and holds what that
/// part does (queue row Q1230).
///
/// The first cut has the apex behind it and cuts the nappe in a hyperbola's
/// branch, running off along `(1, 0, 2)`, the axis seen in its plane. The
/// second leans across it, and its kept side falls along that direction,
/// `−(2 + 2) < 0`, so it keeps one stretch of the branch and the part in front
/// of both is one piece: the lens between the two hyperbolas. Every point
/// below is on the cone exactly:
///
/// - `(3, ±9/4, 15/4)`: `2x − z = 9/4` and `2x + z = 39/4`, in front of both,
///   so held. Seen down the axis these are on the chord `x = 3` both arcs share,
///   where reading the skin's sides down the axis refused them before.
/// - `(2, 0, 2)`: `2x − z = 2`, `2x + z = 6`, held.
/// - `(1, 0, 1)`: `2x − z = 1`, the first branch's vertex, on the boundary.
/// - `(4, 3, 5)`: `2x + z = 13`, past the second cut, not held.
/// - `(0, 1, 1)`: `2x − z = −1`, behind the first cut, not held.
/// - `(3, 4, −5)`: `2x − z = 11` and `2x + z = 1`, in front of both cuts, but on
///   the nappe below the apex, not held.
#[test]
fn the_skin_of_a_chip_between_two_hyperbolas_holds_the_lens_between_them() {
    let mut model = a_model();
    let chip = a_chip_of_a_cone(&mut model);
    let solid = model.solid(chip).expect("this model made the key");
    let skin = solid
        .faces()
        .find(|face| matches!(face.surface(), Surface::Cone(_)))
        .expect("the chip has a face on the cone")
        .key();
    for (point, answer) in [
        ([3.0, 2.25, 3.75], OnAFace::Held),
        ([3.0, -2.25, 3.75], OnAFace::Held),
        ([2.0, 0.0, 2.0], OnAFace::Held),
        ([1.0, 0.0, 1.0], OnAFace::OnItsBoundary),
        ([4.0, 3.0, 5.0], OnAFace::NotHeld),
        ([0.0, 1.0, 1.0], OnAFace::NotHeld),
        ([3.0, 4.0, -5.0], OnAFace::NotHeld),
    ] {
        let held = solid
            .face_holds_the_point(
                skin,
                PredicatePoint::from(point),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("the chip's skin at {point:?}: {why:?}"));
        assert_eq!(held, answer, "the chip's skin at {point:?}");
    }
}

/// The top of [`a_ball_dented_by_another_and_cut_flat`] is a crescent: the disc
/// `x² + y² ≤ 25` on `z = 0` less the disc `(x − 6)² + y² < 25`. Both its arcs
/// run between `(3, 4, 0)` and `(3, −4, 0)`, so their chords are one segment,
/// and a point on it was refused `AFaceTrimmedByACurveThisCannotFollow` (queue
/// row Q1258): a ball kept outside is no family the steps follow, so the face
/// is read off its sides, and the point was on the chords of two arcs at once.
///
/// Every point of the segment is strictly inside the second disc, the one the
/// face leaves out, so none is held. Off it, `(0, 0, 0)` and `(−4, 2, 0)` are in
/// the first disc and outside the second, so held; `(2, 0, 0)` and `(4, 0, 0)`
/// are inside both, and `(3, 5, 0)`, on the chord's line past its end, is
/// outside both; `(3, 4, 0)` is a corner.
#[test]
fn the_top_of_a_ball_dented_by_another_holds_no_point_of_the_chord_its_two_arcs_share() {
    let mut model = a_model();
    let dented = a_ball_dented_by_another_and_cut_flat(&mut model);
    let solid = model.solid(dented).expect("this model made the key");
    let top = solid
        .faces()
        .find(|face| matches!(face.surface(), Surface::Plane(_)))
        .expect("the dented ball has a flat face")
        .key();
    for (point, answer) in [
        ([3.0, 0.0, 0.0], OnAFace::NotHeld),
        ([3.0, 2.0, 0.0], OnAFace::NotHeld),
        ([3.0, -3.5, 0.0], OnAFace::NotHeld),
        ([0.0, 0.0, 0.0], OnAFace::Held),
        ([-4.0, 2.0, 0.0], OnAFace::Held),
        ([2.0, 0.0, 0.0], OnAFace::NotHeld),
        ([4.0, 0.0, 0.0], OnAFace::NotHeld),
        ([3.0, 5.0, 0.0], OnAFace::NotHeld),
        ([3.0, 4.0, 0.0], OnAFace::OnItsBoundary),
    ] {
        let held = solid
            .face_holds_the_point(
                top,
                PredicatePoint::from(point),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("the dented ball's top at {point:?}: {why:?}"));
        assert_eq!(held, answer, "the dented ball's top at {point:?}");
    }
}

/// A cone face asked at points on two chords at once, which
/// `what_a_cone_face_seen_down_its_axis_holds` in `holds.rs` refused
/// `AFaceTrimmedByACurveThisCannotFollow` once the solid built (queue rows
/// Q1258, Q1395 and Q2172).
///
/// **A lens alone never gets there.** A face on one nappe bounded by an arc of
/// each of two planes between the same two corners is crossed by each ruling
/// it holds in one piece, from the plane the apex is behind out to the plane
/// the apex is in front of, so it is the part of the nappe in front of both
/// cuts. It is bounded, so the second plane is reached along both rulings the
/// first plane's branch runs off along, and its kept side falls along both, so
/// along the way the branch runs off, their sum: `is_a_family_this_follows`
/// takes the pair, and the steps answer the lens before it is read off its
/// sides. [`a_chip_of_a_cone`] is that lens, and
/// `the_skin_of_a_chip_between_two_hyperbolas_holds_the_lens_between_them`
/// asks it on its chord. A chip whose second cut rises the way the branch runs
/// off keeps the branch's far end, so it is not bounded.
///
/// **So the skin needs a second wire whose cuts the steps do not follow**, and
/// [`a_chip_of_a_cone_holed_through`] gives it a square hole along the axis,
/// whose walls `x = 2.8125` and `y = ±1.5` each have the apex behind them.
/// Seen down the axis its two hyperbolas' arcs still share the chord `x = 3`.
/// Building it meshes its faces, and a cone face with a hole that goes round
/// nothing was refused there, `CurvedFaceWithAHoleGoesTooFarRound`, until a
/// hole was cut out to its outline on a cone face clear of its apex
/// ([`a_chip_of_a_cone_holed_through_holds_the_chip_less_the_hole`]).
///
/// **Each point is on the cone exactly.** `(3, ±9/4, 15/4)` and `(3, 0, 3)` are
/// on the chord `x = 3` seen down the axis, in front of both cuts and clear of
/// the hole, so held. They are read a little way towards a corner of the hole,
/// which is off that chord's line. `(12/5, 0, 12/5)` is inside the hole, so not
/// held.
#[test]
fn the_skin_of_a_chip_holed_through_holds_the_lens_less_the_hole() {
    let mut model = a_model();
    let chip = a_chip_of_a_cone_holed_through(&mut model)
        .unwrap_or_else(|why| panic!("the holed chip: {why:?}"));
    let solid = model.solid(chip).expect("this model made the key");
    let skin = solid
        .faces()
        .find(|face| matches!(face.surface(), Surface::Cone(_)))
        .expect("the holed chip has a face on the cone")
        .key();
    for (point, answer) in [
        ([3.0, 2.25, 3.75], OnAFace::Held),
        ([3.0, -2.25, 3.75], OnAFace::Held),
        ([3.0, 0.0, 3.0], OnAFace::Held),
        ([2.4, 0.0, 2.4], OnAFace::NotHeld),
    ] {
        let held = solid
            .face_holds_the_point(
                skin,
                PredicatePoint::from(point),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("the holed chip's skin at {point:?}: {why:?}"));
        assert_eq!(held, answer, "the holed chip's skin at {point:?}");
    }
}

/// [`a_chip_of_a_cone_holed_through`] builds, validates and holds the chip less
/// its hole (queue row Q2172).
///
/// Its skin is a face on one nappe clear of the apex, with an outline that goes
/// more than a quarter turn round the axis and a hole, neither loop round the
/// axis. Meshing it was refused `CurvedFaceWithAHoleGoesTooFarRound`, since a
/// cone's hole was cut out to its outline only where the surface pinches
/// nowhere, and a cone pinches at its apex.
///
/// **The answer.** The chip is [`the_chips_volume`]. The hole is the column over
/// the square `[2, 2.8125] × [−1.5, 1.5]` from the skin `z = √(x² + y²)` up to
/// the top `z = 2x − 1`, whose slice at `x` is `3(2x − 1)` less
/// `1.5 √(x² + 2.25) + x² asinh(1.5/x)`, the integral of `√(x² + y²)` over
/// `|y| ≤ 1.5`.
#[test]
fn a_chip_of_a_cone_holed_through_holds_the_chip_less_the_hole() {
    let mut model = a_model();
    let chip = a_chip_of_a_cone_holed_through(&mut model)
        .unwrap_or_else(|why| panic!("the holed chip: {why:?}"));
    let hole = by_simpsons_rule(2.0, 2.8125, |x| {
        3.0 * (2.0 * x - 1.0) - (1.5 * (x * x + 2.25).sqrt() + x * x * (1.5 / x).asinh())
    });
    assert_holds(&model, chip, the_chips_volume() - hole, "the holed chip");
}

/// The frustum of slope 1 from `z = 1` to `5`, cut to `x ≥ 0`, less the slot
/// `|y| ≤ 0.5`, `2.5 ≤ z ≤ 3.5` straight through it along `x` (queue row Q2172).
///
/// The slot cuts a window in the half frustum's skin, clear of the apex, inside
/// an outline that goes half a turn round the axis. Fusing it was refused
/// `CurvedFaceWithAHoleGoesTooFarRound` when the answer was meshed, since a
/// cone's hole was cut out to its outline only where the surface pinches
/// nowhere, and a cone pinches at its apex.
///
/// **The answer.** The half frustum is `π(5³ − 1³)/6 = 124π/6`. The slot takes
/// the slice at `z` of the half disc `r ≤ z` within `|y| ≤ 0.5`, which is the
/// integral of `√(z² − y²)` over `|y| ≤ 0.5`,
/// `0.5 √(z² − 0.25) + z² asin(0.5/z)`. The answer is one solid.
#[test]
fn a_halved_frustum_less_a_slot_through_its_skin_holds_the_half_less_the_slot() {
    use std::f64::consts::PI;
    let mut model = a_model();
    let frustum = cones::a_frustum(&mut model, 1.0, 1.0, 5.0);
    let half = block(&mut model, [0.0, -10.0, 0.0], [10.0, 10.0, 10.0]);
    let half = fuse_one(&mut model, frustum, half, Rule::Intersection);
    let slot = block(&mut model, [-10.0, -0.5, 2.5], [10.0, 0.5, 3.5]);
    let what = "the halved frustum less the slot";
    let answer = fuse(&mut model, half, slot, Rule::Difference, true)
        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
    let slot = by_simpsons_rule(2.5, 3.5, |z| {
        0.5 * (z * z - 0.25).sqrt() + z * z * (0.5 / z).asin()
    });
    assert_each_holds(&model, &answer, &[124.0 * PI / 6.0 - slot], what);
}

/// The part of the cone `r ≤ z` under the roof of
/// `a_roofed_frustum_beside_a_plate_with_a_triangular_hole_is_answered_under_every_rule`
/// and `a_frustum_under_two_steep_planes_is_answered_under_every_rule` from
/// `z = low` to `high`, by Simpson's rule over its slices.
///
/// The slice at height `z` is the disc `r ≤ z` between `x = −(3 + 3z)/8` and
/// `x = (3 − z)/2`. The first line cuts the disc from `z = 0.6` up and the
/// second from `z = 1`, and the two meet at `x = −6` at `z = 15`, so the slice
/// area is smooth between those heights and each stretch is summed on its own.
fn the_roofed_frustum_between(low: f64, high: f64) -> f64 {
    let slice = |z: f64| {
        let (left, right) = (-(3.0 + 3.0 * z) / 8.0, (3.0 - z) / 2.0);
        if left >= right {
            return 0.0;
        }
        the_part_of_a_disc_past(z, left) - the_part_of_a_disc_past(z, right)
    };
    let mut heights = vec![low];
    heights.extend([0.6, 1.0].into_iter().filter(|&at| low < at && at < high));
    heights.push(high);
    heights
        .windows(2)
        .map(|stretch| by_simpsons_rule(stretch[0], stretch[1], slice))
        .sum()
}

/// The part of a disc of radius `radius` about the origin past `x = from`:
/// `r² acos(a/r) − a √(r² − a²)` where the line cuts it.
fn the_part_of_a_disc_past(radius: f64, from: f64) -> f64 {
    if from >= radius {
        0.0
    } else if from <= -radius {
        std::f64::consts::PI * radius * radius
    } else {
        radius * radius * (from / radius).acos() - from * (radius * radius - from * from).sqrt()
    }
}

/// The integral of `slice` from `low` to `high` by Simpson's rule over 20 000
/// steps.
fn by_simpsons_rule(low: f64, high: f64, slice: impl Fn(f64) -> f64) -> f64 {
    let steps = 20_000;
    let step = (high - low) / f64::from(steps);
    let inner: f64 = (1..steps)
        .map(|at| {
            let weight = if at % 2 == 1 { 4.0 } else { 2.0 };
            weight * slice(low + f64::from(at) * step)
        })
        .sum();
    (slice(low) + inner + slice(high)) * step / 3.0
}

/// The ball of radius 2 at the origin against the square rod
/// `[−1, 1]² × [−5, 5]` holed down its own axis by a bar of radius `0.3`.
///
/// The rod's four walls cut the ball's skin in four circles that are one lump,
/// with a patch inside all four at each end, and the hole cuts it in two
/// circles square to the axis, one in each patch. Each circle's outer walk is
/// on the same side of every circle as both patches, so which patch holds it
/// is asked, and it was refused `CannotTellTheCellsOfAFaceApart` under every
/// rule both ways round from Q1140 on. Opened out flat from a node of the
/// circle, the patch holding it is the one whose walk goes round the far
/// patch's corners. Then each outer walk faced the other's, since neither
/// circle has the other inside it, and joining them glued both patches into
/// one cell, which was refused the same way; two walks each one cell with a
/// different patch are not joined now (queue row Q1242).
///
/// **The answer.** The rod holds the whole square inside the ball's radius, so
/// the two share [`the_square_part_of_the_ball`] less the hole's
/// `(4π/3)(8 − 3.91^(3/2))`, the part of the ball within `0.3` of the axis. The
/// holed rod is `40 − 0.9π`. The ball less the rod leaves the ball outside the
/// square and the core inside the hole, apart; the rod less the ball leaves its
/// two ends, alike.
#[test]
fn a_ball_against_a_square_rod_holed_down_its_axis_is_answered_under_every_rule() {
    use std::f64::consts::PI;
    let (ball, square) = (32.0 * PI / 3.0, the_square_part_of_the_ball());
    let hole = 4.0 * PI / 3.0 * (8.0 - 3.91f64.powf(1.5));
    let holed = 40.0 - 0.9 * PI;
    let shared = square - hole;
    for (rule, ball_first) in every_rule_both_ways() {
        let mut model = a_model();
        let sphere = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
        let rod = block(&mut model, [-1.0, -1.0, -5.0], [1.0, 1.0, 5.0]);
        let bar = model
            .add_round_bar(0.3, 12.0, &shift(0.0, 0.0, -6.0))
            .expect("a bar");
        let rod = fuse_one(&mut model, rod, bar, Rule::Difference);
        let what = format!("{rule:?}, the ball first {ball_first}");
        let answer = fuse(&mut model, sphere, rod, rule, ball_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let volumes = match (rule, ball_first) {
            (Rule::Union, _) => vec![ball + holed - shared],
            (Rule::Intersection, _) => vec![shared],
            (Rule::Difference, true) => vec![hole, ball - square],
            (Rule::Difference, false) => vec![(holed - shared) / 2.0; 2],
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_each_holds(&model, &answer, &volumes, &what);
    }
}

/// The ball of radius 2 at the origin against the square rod
/// `[−1, 1]² × [−5, 5]` holed off its axis, down `(0.4, 0.3)`, by a bar of
/// radius `0.3`.
///
/// The hole's axis is `0.5` off the ball's centre and the ball swallows the
/// hole's whole cross-section, `2 > 0.3 + 0.5`, so the two cross in two quartic
/// loops either side of the centre rather than in two circles. Each loop runs
/// once round the hole's axis, and is ordered round it since Q1554; until then
/// every fuse was refused `ACrossingThisCannotBeOrderedAlong` before any cell.
///
/// **The answer** is the one the rod holed down its own axis has, with the
/// part of the ball inside the hole [`the_ball_inside_a_hole`] rather than
/// `(4π/3)(8 − 3.91^(3/2))`: the hole lies inside the square, and the rod is
/// alike either side of `z = 0`, so the rod less the ball is still two alike
/// ends.
#[test]
fn a_ball_against_a_square_rod_holed_off_its_axis_is_answered_under_every_rule() {
    use std::f64::consts::PI;
    let (ball, square) = (32.0 * PI / 3.0, the_square_part_of_the_ball());
    let hole = the_ball_inside_a_hole(0.3, 0.5);
    let holed = 40.0 - 0.9 * PI;
    let shared = square - hole;
    for (rule, ball_first) in every_rule_both_ways() {
        let mut model = a_model();
        let sphere = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
        let rod = block(&mut model, [-1.0, -1.0, -5.0], [1.0, 1.0, 5.0]);
        let bar = model
            .add_round_bar(0.3, 12.0, &shift(0.4, 0.3, -6.0))
            .expect("a bar");
        let rod = fuse_one(&mut model, rod, bar, Rule::Difference);
        let what = format!("{rule:?}, the ball first {ball_first}");
        let answer = fuse(&mut model, sphere, rod, rule, ball_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let volumes = match (rule, ball_first) {
            (Rule::Union, _) => vec![ball + holed - shared],
            (Rule::Intersection, _) => vec![shared],
            (Rule::Difference, true) => vec![hole, ball - square],
            (Rule::Difference, false) => vec![(holed - shared) / 2.0; 2],
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_each_holds(&model, &answer, &volumes, &what);
    }
}

/// [`the_ball_inside_a_hole`] down the ball's own axis is the closed form
/// `(4π/3)(8 − 3.91^(3/2))`, to the `1e-7` its doc gives it.
#[test]
fn the_ball_inside_a_hole_down_its_axis_is_the_closed_form() {
    use std::f64::consts::PI;
    let closed = 4.0 * PI / 3.0 * (8.0 - 3.91f64.powf(1.5));
    let ruled = the_ball_inside_a_hole(0.3, 0.0);
    assert!(
        (ruled - closed).abs() < 1e-7,
        "{ruled} by the rule, {closed} closed"
    );
}

/// The part of the ball of radius 2 at the origin within `radius` of a line up
/// `z` whose foot is `off` from the centre: `∫∫ 2√(4 − ρ² − off² − 2·off·ρ·cos
/// θ) ρ dρ dθ` over the hole's section, `ρ` out from the hole's axis.
///
/// A midpoint rule of 1000 steps out from the axis by 64 round it, as
/// `two_curves_of_one_pair_that_meet.rs`'s `ball_in_bar`: the integrand is
/// smooth and goes round periodically, so the error is the step out's, and down
/// the ball's axis it is within `1e-7` of the closed form
/// ([`the_ball_inside_a_hole_down_its_axis_is_the_closed_form`]), far below the
/// `0.0078` a mesh of `0.001` gives the smallest piece here, the part of the
/// ball inside the hole, as its error. The hole must lie
/// inside the ball's equator, `radius + off < 2`.
fn the_ball_inside_a_hole(radius: f64, off: f64) -> f64 {
    use std::f64::consts::PI;
    let (out, round) = (1000, 64);
    let (step_out, step_round) = (radius / f64::from(out), 2.0 * PI / f64::from(round));
    let mut sum = 0.0;
    for i in 0..out {
        let rho = step_out * (f64::from(i) + 0.5);
        for j in 0..round {
            let theta = step_round * (f64::from(j) + 0.5);
            let squared = 4.0 - rho * rho - off * off - 2.0 * off * rho * theta.cos();
            sum += 2.0 * squared.sqrt() * rho;
        }
    }
    sum * step_out * step_round
}

/// A ball of radius `0.5` centred on the wall of a bar of radius 2, height 10,
/// is answered under every rule both ways round, with the ball on the bar's
/// seam at `(2, 0, 5)` and a quarter turn round at `(0, 2, 5)`. Each answer is
/// one solid.
///
/// **The ball's poles lie on the wall.** A sphere stores only its centre and
/// radius, and is parametrised about world `z` (`validate.rs`'s
/// `the_way_it_is_parametrised_about`), so turning the ball moves neither pole.
/// On the seam they are `(2, 0, 4.5)` and `(2, 0, 5.5)`, both on the wall. The
/// ball crosses the wall in one loop, `16 sin²(θ/2) + (z − 5)² = 0.25` with `θ`
/// round the bar from the ball's centre, whose top and bottom are those poles,
/// and the loop starts at the top one. Until Q2171 its one closed edge started
/// there, and every rule was refused `AClosedEdgeOfTheAnswerWouldStartAtAPole`,
/// on the bar's pipe with the bar first and on the ball with the ball first:
/// `fuse/arrange.rs`'s `across_a_circle_that_starts_at_a_pole` cut a loop a
/// second time only for a ball against a plane. It now cuts this one at the
/// other pole too, so it is two edges with a direction each.
///
/// The loop runs back and forth round the bar's axis, which no determinant
/// orders (§34.10). Q1554 ordered only the two loops of a ball that swallows a
/// pipe's whole cross-section, which a ball of `0.5` against a bar of 2 cannot:
/// the rod holed off its axis,
/// [`a_ball_against_a_square_rod_holed_off_its_axis_is_answered_under_every_rule`],
/// was pinned here beside it until then. Until Q1661 every rule was refused
/// `ACrossingThisCannotBeOrderedAlong` before any cell, by `fuse/arrange.rs`'s
/// `how_a_crossing_runs`, which now lets the one loop through, and since Q1767
/// `rcad-geom` orders it by angle round the line through the ball's centre
/// square to the bar's axis. The fuse then got as far as the cells and was
/// refused `CannotTellTheCellsOfAFaceApart` on the pipe, until Q1777's
/// `the_one_on_its_side_of_every_cut` told those cells apart.
///
/// **The answer.** The bar is `40π` and the ball `π/6`, and the two share
/// [`the_small_ball_inside_the_bar`].
///
/// Named `a_small_ball_on_a_bars_wall_is_refused_before_any_cell` until Q1766,
/// `a_small_ball_on_a_bars_seam_is_refused_telling_the_pipes_cells_apart`
/// until Q1985, and
/// `a_small_ball_centred_on_a_bars_wall_is_refused_where_its_poles_lie_on_the_wall`
/// until Q2171.
#[test]
fn a_small_ball_centred_on_a_bars_wall_is_answered_where_its_poles_lie_on_the_wall() {
    use std::f64::consts::PI;
    let (bar, ball, shared) = (40.0 * PI, PI / 6.0, the_small_ball_inside_the_bar());
    for (what, [x, y, z]) in [
        ("on its seam", [2.0, 0.0, 5.0]),
        ("a quarter turn from its seam", [0.0, 2.0, 5.0]),
    ] {
        for (rule, first) in every_rule_both_ways() {
            let mut model = a_model();
            let pipe = model
                .add_round_bar(2.0, 10.0, &Isometry::identity())
                .expect("a bar");
            let small = model.add_ball(0.5, &shift(x, y, z)).expect("a small ball");
            let what = format!("a ball {what}, {rule:?}, the bar first {first}");
            let answer = fuse(&mut model, pipe, small, rule, first)
                .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            let volumes = match (rule, first) {
                (Rule::Union, _) => vec![bar + ball - shared],
                (Rule::Intersection, _) => vec![shared],
                (Rule::Difference, true) => vec![bar - shared],
                (Rule::Difference, false) => vec![ball - shared],
                (other, _) => panic!("not a rule this probes: {other:?}"),
            };
            assert_each_holds(&model, &answer, &volumes, &what);
        }
    }
}

/// The ball and bar of
/// [`a_small_ball_centred_on_a_bars_wall_is_answered_where_its_poles_lie_on_the_wall`],
/// the ball on the seam, with the block `|x| < 1`, `|y| < 5`, `−1 < z < 11`
/// beside the ball and through the bar, which was to make a walk going round
/// nothing on the pipe. It is fused with the ball, against the bar, under every
/// rule both ways round, and was refused `AClosedEdgeOfTheAnswerWouldStartAtAPole`
/// with the ball and bar alone until Q2171.
///
/// **The answer.** The block misses the ball, and holds the strip of the bar
/// between `x = ±1`, `10(2√3 + 4π/3)`, which leaves the bar's two sides
/// `10(4π/3 − √3)` each. So the bar and the block less the bar are each one
/// solid, and the bar less the block is two, its side at `x > 1` dented by the
/// ball.
#[test]
fn a_small_ball_on_a_bars_wall_beside_a_block_through_the_bar_is_answered_under_every_rule() {
    use std::f64::consts::PI;
    let (bar, ball, shared) = (40.0 * PI, PI / 6.0, the_small_ball_inside_the_bar());
    let (slab, strip) = (240.0, 10.0 * (2.0 * 3f64.sqrt() + 4.0 * PI / 3.0));
    let side = 10.0 * (4.0 * PI / 3.0 - 3f64.sqrt());
    for (rule, first) in every_rule_both_ways() {
        let mut model = a_model();
        let pipe = model
            .add_round_bar(2.0, 10.0, &Isometry::identity())
            .expect("a bar");
        let others = vec![
            block(&mut model, [-1.0, -5.0, -1.0], [1.0, 5.0, 11.0]),
            model
                .add_ball(0.5, &shift(2.0, 0.0, 5.0))
                .expect("a small ball"),
        ];
        let (arguments, tools) = if first {
            (vec![pipe], others)
        } else {
            (others, vec![pipe])
        };
        let what = format!("{rule:?}, the bar first {first}");
        let answer = model
            .fuse(
                &arguments,
                &tools,
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: {why:?}"))
            .solids()
            .to_vec();
        let mut volumes = match (rule, first) {
            (Rule::Union, _) => vec![bar + slab - strip + ball - shared],
            (Rule::Intersection, _) => vec![shared, strip],
            (Rule::Difference, true) => vec![side - shared, side],
            (Rule::Difference, false) => vec![ball - shared, slab - strip],
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        volumes.sort_by(f64::total_cmp);
        assert_each_holds(&model, &answer, &volumes, &what);
    }
}

/// The part of the ball of radius `0.5` at `(2, 0, 5)` inside the bar of
/// radius 2 up `z`, by Simpson's rule over its slices.
///
/// The slice at height `z` is the lens the ball's disc of radius
/// `s = √(0.25 − (z − 5)²)` shares with the bar's disc, two centres 2 apart:
/// `s² acos(s/4) + 4 acos(1 − s²/8) − (s/2)√(16 − s²)`.
fn the_small_ball_inside_the_bar() -> f64 {
    by_simpsons_rule(4.5, 5.5, |z| {
        let s = (0.25 - (z - 5.0) * (z - 5.0)).max(0.0).sqrt();
        s * s * (s / 4.0).acos() + 4.0 * (1.0 - s * s / 8.0).acos()
            - s / 2.0 * (16.0 - s * s).sqrt()
    })
}

/// A ball of radius `3/8` at the origin against a ball of radius `5/8` whose
/// centre is `1/2` away along `x` or `y`, under every rule both ways round.
///
/// The two cross in the circle of radius `3/8` on the plane through the small
/// ball's centre square to the line between the centres, which is a great
/// circle of the small ball through both its poles, `(0, 0, ±3/8)`. The walk
/// starts that circle at one of the poles, and each fuse was refused
/// `AClosedEdgeOfTheAnswerWouldStartAtAPole` until Q2171 cut it at the other
/// pole as well, the point across. Probed at eight times this size, the big
/// ball along `−x` was answered before, so it is not run here.
///
/// **The answer.** The balls are `9π/128` and `125π/384`, and they share
/// [`the_lens_two_balls_share`]: the small ball's half past the circle and a
/// cap `1/8` deep of the big one, `17π/384`.
#[test]
fn two_balls_crossing_in_a_circle_through_the_poles_of_one_are_answered_under_every_rule() {
    use std::f64::consts::PI;
    let (small, big) = (9.0 * PI / 128.0, 125.0 * PI / 384.0);
    let lens = the_lens_two_balls_share(0.625, 0.375, 0.5);
    for (what, [x, y]) in [("x", [0.5, 0.0]), ("y", [0.0, 0.5])] {
        for (rule, small_first) in every_rule_both_ways() {
            let mut model = a_model();
            let one = model
                .add_ball(0.375, &Isometry::identity())
                .expect("a ball");
            let other = model.add_ball(0.625, &shift(x, y, 0.0)).expect("a ball");
            let what =
                format!("the big ball along {what}, {rule:?}, the small first {small_first}");
            let answer = fuse(&mut model, one, other, rule, small_first)
                .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            let volumes = match (rule, small_first) {
                (Rule::Union, _) => vec![small + big - lens],
                (Rule::Intersection, _) => vec![lens],
                (Rule::Difference, true) => vec![small - lens],
                (Rule::Difference, false) => vec![big - lens],
                (other, _) => panic!("not a rule this probes: {other:?}"),
            };
            assert_each_holds(&model, &answer, &volumes, &what);
        }
    }
}

/// A whole ring about `x`, 4 round a tube of 1, against a ball of radius 4 at
/// `(1, 0, 0)`, under every rule both ways round. The three tests after this
/// one put the ball at `(−1, 0, 0)`, and a ball of 3 at `(±3/4, 0, 0)` pressed
/// into an O-ring lying on its side, 4 round a tube of `5/4`.
///
/// For the ball of 4 at `(1, 0, 0)`, the two cross in two circles round `x`:
/// one at `x = 1` of radius 4, the top of the tube and a great circle of the
/// ball through both its poles, and one at `x = −15/17` of radius `60/17`. The
/// ball of 3 at `(3/4, 0, 0)` is the same shape: its great circle at `x = 3/4`
/// lies on the ring's inner wall, and its other circle is at `x = −357/1060`.
/// The walk starts the first circle at one of the poles, and each fuse was
/// refused `AClosedEdgeOfTheAnswerWouldStartAtAPole` until Q2171 cut it at the
/// point across as well, the ball's other pole. Each was then refused for want
/// of a point inside a piece whose corners were the ball's two poles, until
/// Q2212 put the ring's two bands in order round the tube. Each was then
/// refused for want of a point inside the ring's band outside the ball, whose
/// every candidate came back onto the inner half of the tube, inside the ball,
/// until Q2205 offered the point half way round the tube between two corners.
/// Each was then refused `AClosedEdgeOfTheAnswerWouldStartAtAPole`, because
/// the ring and the ball's two circles went by one name, so the other circle
/// closing on itself had the two halves of the circle through the poles joined
/// back into one closed side starting at a pole, until Q2260 named each circle
/// by its own plane and each half of the circle through the poles by its sign
/// (`which_of_the_pairs_closed_crossings_it_is` and
/// `which_arc_of_a_circle_walked_this_way` in `fuse/assemble.rs`).
///
/// The union both ways round and the ring less the ball were then refused
/// `TheFuseWroteABrokenAnswer { AClosedEdgeIsWalkedAgainstItsSurface }`, by
/// the sweep the fuse runs on its answer, and the answer was sound. Each keeps
/// the ring's whole band outside the ball as one face, bounded by the circle
/// through the poles in two arcs and by the other circle, at `x = −15/17` for
/// the ball of 4, stored with its own plane. So `Model::validate` finds one
/// circle of the band round the axis, not two, and read which side of that
/// circle's plane the band lies on off the band's corners. Those are the
/// ball's poles, in front of the plane; next to the circle the band lies
/// behind it, since the plane cuts the band again round the tube's outer half,
/// where there is no edge. Worked by hand at `(−15/17, 0, 60/17)`,
/// `∇ring × ∇plane` and `−(∇ring × ∇ball)` both run along `−y`, the way the
/// fuse walks the circle. The band inside the ball has the same two curves for
/// its edges and the same corners, so read off them one of the two bands was
/// always going to be refused. Since Q2287, `validate.rs` reads a ring face's
/// side of the ball across such a circle instead; every corner of either band
/// is on the ball, so nothing says, and the circle is passed over.
///
/// The ball at `(−1, 0, 0)` is the same shape mirrored, and so is the ball of 3
/// at `(−3/4, 0, 0)`. Each ball is its own test, so the four run side by side.
///
/// **The answer.** The ring is `8π²` and the ball `256π/3`, and the part of the
/// ring inside the ball at `(1, 0, 0)` is `π(2π + 4 atan(15/8) − 32/17)`. Its
/// slice at `x` is the ring's annulus between radii `4 ± √(1 − x²)` inside the
/// ball's disc of radius `√(16 − (x − 1)²)`, which never reaches past the
/// annulus, so the slice is `π(8√(1 − x²) + 2x − 2)` wherever that is above
/// nothing: from `x = −15/17`, where `4√(1 − x²) = 1 − x`, to `x = 1`. The
/// O-ring is `25π²/2` and its ball `36π`, and the slice is
/// `π(8√(25/16 − x²) + 3x/2 − 73/8)`, from `x = −357/1060` to `x = 3/4`.
#[test]
fn a_ball_of_four_crossing_a_ring_through_its_poles_from_plus_x_is_answered_under_every_rule() {
    a_ball_crossing_a_ring_through_its_poles_is_answered_under_every_rule(
        1.0,
        4.0,
        1.0,
        the_ring_inside_the_ball_of_four(),
    );
}

/// The ball of 4 at `(−1, 0, 0)`: the test above, mirrored.
#[test]
fn a_ball_of_four_crossing_a_ring_through_its_poles_from_minus_x_is_answered_under_every_rule() {
    a_ball_crossing_a_ring_through_its_poles_is_answered_under_every_rule(
        1.0,
        4.0,
        -1.0,
        the_ring_inside_the_ball_of_four(),
    );
}

/// The ball of 3 at `(3/4, 0, 0)` pressed into the O-ring, 4 round a tube of
/// `5/4`: the ball of 4's shape, its great circle on the ring's inner wall.
#[test]
fn a_ball_of_three_pressed_into_an_o_ring_from_plus_x_is_answered_under_every_rule() {
    a_ball_crossing_a_ring_through_its_poles_is_answered_under_every_rule(
        1.25,
        3.0,
        0.75,
        the_o_ring_inside_the_ball_of_three(),
    );
}

/// The ball of 3 at `(−3/4, 0, 0)`: the test above, mirrored.
#[test]
fn a_ball_of_three_pressed_into_an_o_ring_from_minus_x_is_answered_under_every_rule() {
    a_ball_crossing_a_ring_through_its_poles_is_answered_under_every_rule(
        1.25,
        3.0,
        -0.75,
        the_o_ring_inside_the_ball_of_three(),
    );
}

/// The whole ring about `x`, 4 round a tube of `tube`, and the ball of
/// `radius` at `(x, 0, 0)`, sharing `shared`, under every rule both ways round.
fn a_ball_crossing_a_ring_through_its_poles_is_answered_under_every_rule(
    tube: f64,
    radius: f64,
    x: f64,
    shared: f64,
) {
    use std::f64::consts::PI;
    let ring = 2.0 * PI * PI * 4.0 * tube * tube;
    let ball = 4.0 * PI * radius * radius * radius / 3.0;
    for (rule, ring_first) in every_rule_both_ways() {
        let mut model = a_model();
        let one = a_whole_ring_about_x(&mut model, tube);
        let other = model.add_ball(radius, &shift(x, 0.0, 0.0)).expect("a ball");
        let what = format!(
            "the ring round a tube of {tube} and the ball of {radius} at x = {x}, {rule:?}, \
             the ring first {ring_first}"
        );
        let answer = fuse(&mut model, one, other, rule, ring_first)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let volumes = match (rule, ring_first) {
            (Rule::Union, _) => vec![ring + ball - shared],
            (Rule::Intersection, _) => vec![shared],
            (Rule::Difference, true) => vec![ring - shared],
            (Rule::Difference, false) => vec![ball - shared],
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_each_holds(&model, &answer, &volumes, &what);
    }
}

/// `π(2π + 4 atan(15/8) − 32/17)`, worked out in the ball of 4's test's doc.
fn the_ring_inside_the_ball_of_four() -> f64 {
    use std::f64::consts::PI;
    PI * (2.0 * PI + 4.0 * (15.0_f64 / 8.0).atan() - 32.0 / 17.0)
}

/// The O-ring's slice inside the ball of 3, from the ball of 4's test's doc,
/// summed along `x`.
fn the_o_ring_inside_the_ball_of_three() -> f64 {
    use std::f64::consts::PI;
    by_simpsons_rule(-357.0 / 1060.0, 0.75, |x| {
        PI * (8.0 * (25.0 / 16.0 - x * x).sqrt() + 1.5 * x - 73.0 / 8.0)
    })
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

/// [`a_chip_of_a_cone`] with the square `[2, 2.8125] × [−1.5, 1.5]` cut out of
/// it along the axis, described face by face. The hole runs from the skin up to
/// the face on `2x − z = 1`, which is the top of the chip everywhere over the
/// square, and has a flat wall on each of `x = 2`, `x = 2.8125` and `y = ±1.5`.
///
/// The hole's corners are exact on the cone: `2² + 1.5² = 2.5²` and
/// `2.8125² + 1.5² = 3.1875²`. Above them, on `z = 2x − 1`, are `z = 3` and
/// `z = 4.625`. What building it gives is handed back.
fn a_chip_of_a_cone_holed_through(model: &mut Model) -> Result<SolidKey, Error> {
    let skin = Cone::new(Axis::z(), 1.0).expect("a positive slope");
    let rising =
        Plane::from_point_and_normal(Point::new(3.0, 4.0, 5.0), Vector::new(-2.0, 0.0, 1.0))
            .expect("a plane");
    let falling =
        Plane::from_point_and_normal(Point::new(3.0, 4.0, 5.0), Vector::new(2.0, 0.0, 1.0))
            .expect("a plane");
    let wall = |through: [f64; 3], facing: [f64; 3]| {
        Plane::from_point_and_normal(
            Point::new(through[0], through[1], through[2]),
            Vector::new(facing[0], facing[1], facing[2]),
        )
        .expect("a plane")
    };
    let near_wall = wall([2.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    let far_wall = wall([2.8125, 0.0, 0.0], [-1.0, 0.0, 0.0]);
    let left_wall = wall([0.0, 1.5, 0.0], [0.0, -1.0, 0.0]);
    let right_wall = wall([0.0, -1.5, 0.0], [0.0, 1.0, 0.0]);
    let upper = HalfSpace::through_a_point_facing(Point::origin(), Vector::new(0.0, 0.0, 1.0))
        .expect("a half-space");
    let on_the_upper_nappe = |plane| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(plane),
        picked_by: smallvec::smallvec![Picker::from(upper)],
    };

    let mut builder = SolidBuilder::new();
    let mut corner = |at: [f64; 3]| {
        builder
            .add_vertex(Point::new(at[0], at[1], at[2]))
            .expect("a corner")
    };
    let left = corner([3.0, 4.0, 5.0]);
    let right = corner([3.0, -4.0, 5.0]);
    let [b00, b10, b11, b01] = [
        [2.0, -1.5, 2.5],
        [2.8125, -1.5, 3.1875],
        [2.8125, 1.5, 3.1875],
        [2.0, 1.5, 2.5],
    ]
    .map(&mut corner);
    let [t00, t10, t11, t01] = [
        [2.0, -1.5, 3.0],
        [2.8125, -1.5, 4.625],
        [2.8125, 1.5, 4.625],
        [2.0, 1.5, 3.0],
    ]
    .map(&mut corner);
    let mut edge = |from, to, run| builder.add_edge(from, to, run).expect("an edge");
    let low = edge(right, left, on_the_upper_nappe(rising));
    let high = edge(left, right, on_the_upper_nappe(falling));
    let chord = edge(left, right, EdgeRun::Straight);
    let round_right = edge(b00, b10, on_the_upper_nappe(right_wall));
    let round_far = edge(b10, b11, on_the_upper_nappe(far_wall));
    let round_left = edge(b11, b01, on_the_upper_nappe(left_wall));
    let round_near = edge(b01, b00, on_the_upper_nappe(near_wall));
    let [up00, up10, up11, up01] = [(b00, t00), (b10, t10), (b11, t11), (b01, t01)]
        .map(|(from, to)| edge(from, to, EdgeRun::Straight));
    let top_right = edge(t00, t10, EdgeRun::Straight);
    let top_far = edge(t10, t11, EdgeRun::Straight);
    let top_left = edge(t11, t01, EdgeRun::Straight);
    let top_near = edge(t01, t00, EdgeRun::Straight);

    let wire = |builder: &mut SolidBuilder, steps: &[(_, Orientation)]| {
        let steps: Vec<DraftStep> = steps
            .iter()
            .map(|&(edge, way)| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")))
            .collect();
        builder.add_wire(&steps).expect("a wire")
    };
    use Orientation::{Forward, Reversed};
    let outline = wire(&mut builder, &[(low, Forward), (high, Forward)]);
    let hole = wire(
        &mut builder,
        &[
            (round_right, Forward),
            (round_far, Forward),
            (round_left, Forward),
            (round_near, Forward),
        ],
    );
    let lens = builder
        .add_face(Surface::Cone(skin), &[outline, hole])
        .expect("a face");
    let outline = wire(&mut builder, &[(low, Reversed), (chord, Reversed)]);
    let hole = wire(
        &mut builder,
        &[
            (top_far, Reversed),
            (top_right, Reversed),
            (top_near, Reversed),
            (top_left, Reversed),
        ],
    );
    let under = builder
        .add_face(Surface::Plane(rising), &[outline, hole])
        .expect("a face");
    let outline = wire(&mut builder, &[(high, Reversed), (chord, Forward)]);
    let over = builder
        .add_face(Surface::Plane(falling), &[outline])
        .expect("a face");
    let mut walls = Vec::new();
    for (plane, steps) in [
        (
            far_wall,
            [
                (round_far, Reversed),
                (up10, Forward),
                (top_far, Forward),
                (up11, Reversed),
            ],
        ),
        (
            near_wall,
            [
                (round_near, Reversed),
                (up01, Forward),
                (top_near, Forward),
                (up00, Reversed),
            ],
        ),
        (
            left_wall,
            [
                (round_left, Reversed),
                (up11, Forward),
                (top_left, Forward),
                (up01, Reversed),
            ],
        ),
        (
            right_wall,
            [
                (round_right, Reversed),
                (up00, Forward),
                (top_right, Forward),
                (up10, Reversed),
            ],
        ),
    ] {
        let outline = wire(&mut builder, &steps);
        walls.push(
            builder
                .add_face(Surface::Plane(plane), &[outline])
                .expect("a face"),
        );
    }
    let shell: Vec<_> = [lens, under, over]
        .into_iter()
        .chain(walls)
        .map(|face| (face, Forward))
        .collect();
    builder.add_shell(&shell).expect("seven faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The part of the ball of radius 5 at the origin below `z = 0` and outside the
/// ball of radius 5 at `(6, 0, 0)`, described face by face. Its top on `z = 0`
/// is a crescent between the arc of the first ball's circle through
/// `(−5, 0, 0)` and the arc of the second's through `(1, 0, 0)`, both from
/// `(3, 4, 0)` to `(3, −4, 0)`. Below, a face on each ball meets the other along
/// the lower half of the circle `x = 3`, `y² + z² = 16`; the second ball's face
/// is used turned round, since the material is outside that ball.
///
/// The two corners are exact on all three surfaces: `3² + 4² = 5²` and
/// `(3 − 6)² + 4² = 5²`.
fn a_ball_dented_by_another_and_cut_flat(model: &mut Model) -> SolidKey {
    let first = Sphere::new(Point::origin(), 5.0).expect("a ball");
    let second = Sphere::new(Point::new(6.0, 0.0, 0.0), 5.0).expect("a ball");
    let top =
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("a plane");
    let picked = |through: Point, facing: Vector| {
        smallvec::smallvec![Picker::from(
            HalfSpace::through_a_point_facing(through, facing).expect("a half-space")
        )]
    };
    let on_the_top = || EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(top),
        picked_by: picked(Point::new(3.0, 0.0, 0.0), Vector::new(-1.0, 0.0, 0.0)),
    };

    let mut builder = SolidBuilder::new();
    let near = builder
        .add_vertex(Point::new(3.0, 4.0, 0.0))
        .expect("a corner");
    let far = builder
        .add_vertex(Point::new(3.0, -4.0, 0.0))
        .expect("a corner");
    let rim = builder.add_edge(near, far, on_the_top()).expect("an arc");
    let dent_rim = builder.add_edge(far, near, on_the_top()).expect("an arc");
    let seam = builder
        .add_edge(
            near,
            far,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Sphere(second),
                picked_by: picked(Point::origin(), Vector::new(0.0, 0.0, -1.0)),
            },
        )
        .expect("an arc");
    let face = |builder: &mut SolidBuilder, surface, steps: [_; 2]| {
        let steps: Vec<DraftStep> = steps
            .into_iter()
            .map(|(edge, way)| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")))
            .collect();
        let wire = builder.add_wire(&steps).expect("a wire");
        builder.add_face(surface, &[wire]).expect("a face")
    };
    let crescent = face(
        &mut builder,
        Surface::Plane(top),
        [
            (rim, Orientation::Forward),
            (dent_rim, Orientation::Forward),
        ],
    );
    let outside = face(
        &mut builder,
        Surface::Sphere(first),
        [(rim, Orientation::Reversed), (seam, Orientation::Forward)],
    );
    let dent = face(
        &mut builder,
        Surface::Sphere(second),
        [
            (dent_rim, Orientation::Reversed),
            (seam, Orientation::Reversed),
        ],
    );
    builder
        .add_shell(&[
            (crescent, Orientation::Forward),
            (outside, Orientation::Forward),
            (dent, Orientation::Reversed),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a dented ball cut flat")
}

/// The volume of [`a_chip_of_a_cone`], by Simpson's rule over its slices.
///
/// The slice at height `z` is the disc `r ≤ z` between `x = (1 + z)/2` and
/// `x = (11 − z)/2`, nonempty for `1 ≤ z ≤ 5`, and the second bound cuts the
/// disc only from `z = 11/3` up. The part of a disc of radius `r` past `x = a`
/// is `r² acos(a/r) − a √(r² − a²)`. The slice area is smooth on each side of
/// `11/3`, so each side is summed on its own.
fn the_chips_volume() -> f64 {
    let slice = |z: f64| {
        the_part_of_a_disc_past(z, (1.0 + z) / 2.0) - the_part_of_a_disc_past(z, (11.0 - z) / 2.0)
    };
    by_simpsons_rule(1.0, 11.0 / 3.0, slice) + by_simpsons_rule(11.0 / 3.0, 5.0, slice)
}

/// The half chip of
/// `a_chip_cut_from_a_frustum_in_two_steps_is_answered_under_every_rule`: the
/// frustum of slope 1 from `z = 1/2` to `8` past `2x − z = 1`, by Simpson's rule
/// over its slices. The slice at height `z` is the disc `r ≤ z` past
/// `x = (1 + z)/2`, which the line cuts from `z = 1` up.
fn the_half_chips_volume() -> f64 {
    by_simpsons_rule(1.0, 8.0, |z| the_part_of_a_disc_past(z, (1.0 + z) / 2.0))
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

/// The whole ring 4 round a tube of `tube`, about `z` and turned a quarter
/// about `y` onto `x`. Its two faces are the ring's halves either side of
/// `z = 0`, joined along the two circles that plane cuts it in, as
/// `a_ring_crossed_in_two_closed_curves.rs` writes it.
fn a_whole_ring_about_x(model: &mut Model, tube: f64) -> SolidKey {
    let cut = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let skin = Surface::Torus(Torus::new(Axis::z(), 4.0, tube).expect("a tube narrower"));
    let mut builder = SolidBuilder::new();
    let [outer, inner] = [4.0 + tube, 4.0 - tube].map(|reach| {
        let corner = builder
            .add_vertex(Point::new(reach, 0.0, 0.0))
            .expect("a corner on the ring and on the plane");
        builder
            .add_edge(
                corner,
                corner,
                EdgeRun::RightRoundWhereItCrosses {
                    and: Surface::Plane(cut),
                },
            )
            .expect("a rim is a whole closed curve, so one corner twice is an edge")
    });
    let face = |builder: &mut SolidBuilder, walks: [(_, Orientation); 2]| {
        let wires = walks.map(|(edge, turn)| {
            let coedge = builder
                .add_coedge(edge, turn)
                .expect("an edge of this build");
            builder
                .add_wire(&[DraftStep::Along(coedge)])
                .expect("one closed edge is a wire")
        });
        builder
            .add_face(skin, &wires)
            .expect("a band between two circles")
    };
    let top = face(
        &mut builder,
        [
            (outer, Orientation::Reversed),
            (inner, Orientation::Forward),
        ],
    );
    let bottom = face(
        &mut builder,
        [
            (outer, Orientation::Forward),
            (inner, Orientation::Reversed),
        ],
    );
    builder
        .add_shell(&[(top, Orientation::Forward), (bottom, Orientation::Forward)])
        .expect("two halves close");
    let ring = builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole ring in two faces is a solid");
    model
        .place_solid(
            ring,
            &Isometry::quarter_turns(&Axis::y(), 1).expect("y is a whole direction"),
        )
        .expect("a solid of this model");
    ring
}
