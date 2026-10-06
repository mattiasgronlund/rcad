//! Shapes that reach the refusal of `docs/decisions.md` §340.2: a walk on a
//! face whose signs match several walks of one other lump, where neither
//! winding (a flat face) nor going round the axis (a cone or a pipe) says which
//! of them holds it. The fuse refuses `CannotTellTheCellsOfAFaceApart` in
//! `arrange.rs`'s `the_walk_of_each_other_lump_each_walk_faces`.
//!
//! **Two shapes reached it, one on a ball and one on a plane**, under every
//! rule and with the arguments both ways round. A printed probe at each of the
//! five places `arrange.rs` raises that refusal, since removed, named that one
//! place for all twelve fuses. Both answers exist — each is two ordinary
//! solids, and its volume is worked out by hand in each test's doc — so both
//! refusals were gaps (§16.10). **The one on a plane is answered now**, since
//! winding on a flat face follows each curved side as well as the corners
//! (queue row Q1053), and every fuse holds the volume its doc works out. The
//! one on a ball still reaches it.
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
//!
//! The other shapes built on the way are pinned beside them, with what each
//! gives instead.

use rcad_geom::{Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

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
