//! A pointed cone drilled right through along its axis: the cone of slope one
//! with its apex at the origin and its base at `z = 10`, and a bar of radius one
//! from `z = −15` to `z = 25`.
//!
//! **The cone's surface has two nappes, and the bar crosses both.** Its wall
//! meets the cone in two circles, at `z = 1` on the solid's own nappe and at
//! `z = −1` on the other, picked apart by the flat half-space through the apex
//! square to the axis. Only the first is on the cone's face, but the bar's wall
//! is arranged against the whole surface, so it is cut at `−15`, `−1`, `1`, `10`
//! and `25` into four bands.
//!
//! **Every fuse of the pair was refused `CannotTellTheCellsOfAFaceApart` on the
//! bar's wall**, in `the_walk_of_each_other_lump_each_walk_faces`, and the sign
//! rule behind it would have glued two bands into one if it had got that far.
//! A walk's side of a curve was read as the cone's sign at one of its points,
//! and past the second circle that sign turns back: the band from `−15` to `−1`
//! and the band from `1` to `10` are both inside the cone, so they agreed about
//! every curve and each was the one walk of the other's lump it faced. A walk
//! along one circle had no sign at all against the other, since its points are
//! on the cone, and that agreed with both sides of it — which is where it
//! refused. A **gap** (§16.10): every answer exists and is worked out below. A
//! point past the half-space that picks a circle out is now on the circle's side
//! towards it, and the sign there is the one a step down the pipe meets.
//!
//! **The union and the bar less the cone keep the bar's band between the two
//! circles**, and were refused twice over before they answered. While the
//! corner each circle starts at was named by the two surfaces alone
//! (`MeetPoint::WhereAClosedCurveStarts`), both circles got one corner and one
//! edge, claimed four times — by the band's two wires, by the band below and by
//! the cone — and the three were refused `ResultIsNotManifold { count: 4 }`.
//! The corner is now named by what picks its circle out of the two as well
//! (Q1148).
//!
//! **Then the sweep refused the answer, and the answer was sound.** The bar's
//! wall from `z = −15` up to the circle at `z = 1` is one face of the answer,
//! the two bands either side of `z = −1` joined into it, and that face crosses
//! the cone's other nappe at `z = −1`. The sweep read which side of the cone the
//! face lies on at the circle `z = 1` off the one corner it has off the cone, the
//! seam corner at `z = −15`, which is inside the other nappe; next to `z = 1` the
//! face is outside the cone. So the edge read as walked against its surfaces,
//! and all three were refused
//! `TheFuseWroteABrokenAnswer { broken: AClosedEdgeIsWalkedAgainstItsSurface }`.
//! A **gap** (Q1190): a corner past the flat half-space that picks the circle
//! out now says the side a step from the circle towards that half-space meets,
//! which is the side the wall keeps next to the circle.
//!
//! **Then the union could not place a point its wall or its cone face reached.**
//! Each has one edge on where the cone and the pipe cross, and they cross in two
//! circles, so `holds.rs` refused both `AFaceMissesPartOfWhereItsSurfacesCross`
//! and every such point came back `CannotFollowAFacesTrimming`. A **gap**
//! (Q1218): a point past the same half-space now says the same side there too.
//!
//! **Then a fuse of the union could not walk its wall.** Fused again with a far
//! cube, the wall's edge on the circle at `z = 1` was read off the seam corner
//! too, this time in the fuse's arrangement, and every order was refused
//! `CannotTellTheCellsOfAFaceApart`. A **gap** (Q1354), closed the same way.
//!
//! **Then the fuse with the cube named both circles one curve.** It draws the
//! wall's circle on the cone again, the other nappe's circle with it, and the
//! two were one closed curve named by the pipe and the cone alone, so both got
//! one edge with four faces along it and every order was refused
//! `ResultIsNotManifold { count: 4 }`. A **gap** (Q1375): a circle round a
//! cone's axis is now named by which side it is on of the flat half-space that
//! picks it out.
//!
//! The closed forms, from the slices of the shared piece: inside the bar the cone
//! holds a disc of radius `z` for `z` up to one and the whole unit disc above it,
//! so the two share `π/3 + 9π = 28π/3`. The cone holds `1000π/3`, and the bar
//! `40π`.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, Rule, SolidKey};

mod cones;

use cones::a_pointed_cone;

const SLOPE: f64 = 1.0;
const HEIGHT: f64 = 10.0;

const CONE: f64 = 1000.0 * PI / 3.0;
const BAR: f64 = 40.0 * PI;
const SHARED: f64 = 28.0 * PI / 3.0;

/// Every rule, whichever goes first, answers with solids the sweep accepts, each
/// holding its closed form.
///
/// The union is `CONE + BAR − SHARED = 364π` and the bar less the cone
/// `BAR − SHARED = 92π/3` in two solids: the bar above `z = 10`, `15π`, and the
/// bar below `z = 1` with the cone's tip taken out of its top, `16π − π/3`. The
/// union and the bar less the cone keep the bar's wall from `z = −15` up to the
/// circle at `z = 1` in one face, which crosses the cone's other nappe at
/// `z = −1`.
///
/// **The slack is what the two bare solids fall short by at this deflection**,
/// measured here rather than written down: a mesh is drawn through points on the
/// surfaces, and no answer holds more curved skin than the cone and the bar
/// together. The bare cone carries no published bound to use instead — a cone
/// face that reaches its apex has a radius of nothing in `measure.rs`.
#[test]
fn every_rule_either_way_round_holds_its_closed_form_across_the_cones_other_nappe() {
    let slack = the_two_solids_fall_short_by();
    let the_bar_less_the_cone = [15.0 * PI, 16.0 * PI - PI / 3.0];
    for (rule, cone_first, want) in [
        (Rule::Union, true, &[CONE + BAR - SHARED][..]),
        (Rule::Union, false, &[CONE + BAR - SHARED][..]),
        (Rule::Intersection, true, &[SHARED][..]),
        (Rule::Intersection, false, &[SHARED][..]),
        (Rule::Difference, true, &[CONE - SHARED][..]),
        (Rule::Difference, false, &the_bar_less_the_cone[..]),
    ] {
        let (model, answer) = fused(rule, cone_first)
            .unwrap_or_else(|why| panic!("{rule:?} (cone first: {cone_first}): {why:?}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?} (cone first: {cone_first}): {why:?}"));
        let mut got: Vec<f64> = answer
            .iter()
            .map(|&solid| volume_of(&model, solid))
            .collect();
        got.sort_by(f64::total_cmp);
        assert_eq!(
            got.len(),
            want.len(),
            "{rule:?} (cone first: {cone_first}): solids holding {got:?}, where the shapes say \
             {want:?}"
        );
        for (got, want) in got.iter().zip(want) {
            assert!(
                (got - want).abs() <= slack,
                "{rule:?} (cone first: {cone_first}) holds {got}, where the shapes say {want} \
                 and the bare solids fall {slack} short"
            );
        }
    }
}

/// Each point round the union, whichever went first, is placed where the two
/// shapes put it.
///
/// The cone holds `0 ≤ z ≤ 10` within `|x, y| ≤ z`, and the bar `−15 ≤ z ≤ 25`
/// within one of the axis. So `(1, 0, −5)` is on the bar's wall below the apex,
/// `(0, 0, −5)` is inside the bar, and `(2, 0, −5)` is outside both, though
/// inside the cone's other nappe; `(3, 0, 3)` is on the cone's skin beside the
/// bar, and `(1.5, 0, 3)` inside the cone beside it.
///
/// **Every point a ray reached the bar's wall or the cone's face from was
/// [`Containment::CannotFollowAFacesTrimming`]**: the first runs along the
/// circle at `z = 1` and across the other nappe at `z = −1`, the second along
/// the circle at `z = 1` alone, and each was refused
/// `AFaceMissesPartOfWhereItsSurfacesCross` for having one edge where the cone
/// and the pipe cross in two circles (row Q1218).
#[test]
fn every_point_round_the_union_is_placed_where_the_shapes_say_across_the_other_nappe() {
    for cone_first in [true, false] {
        let (model, union) = the_union(cone_first);
        let solid = model.solid(union).expect("this model made the key");
        for (at, want) in [
            (Point::new(1.0, 0.0, -5.0), Containment::OnBoundary),
            (Point::new(0.0, 0.0, -5.0), Containment::Inside),
            (Point::new(2.0, 0.0, -5.0), Containment::Outside),
            (Point::new(3.0, 0.0, 3.0), Containment::OnBoundary),
            (Point::new(1.5, 0.0, 3.0), Containment::Inside),
        ] {
            let got = solid.classify_point(at, &Policy::certified(), &mut ProvenanceLog::new());
            assert_eq!(
                got, want,
                "the union (cone first: {cone_first}) places {at:?} {got:?}, where the shapes say \
                 {want:?}"
            );
        }
    }
}

/// The union fused again with a cube far from it is the two solids apart, the
/// union's `364π` and the cube's `8`, every way round, and the sweep accepts
/// them.
///
/// **By hand.** The cone of slope one up to `z = 10` has a base of radius ten,
/// so it holds `π · 10² · 10 / 3 = 1000π/3`, and the bar of radius one and
/// length forty holds `40π`. Inside the bar, the cone holds a disc of radius
/// `z` for `z` up to one, `∫₀¹ πz² dz = π/3`, and the whole unit disc from there
/// to ten, `9π`, so the two share `28π/3`. The union holds
/// `1000π/3 + 40π − 28π/3 = 972π/3 + 40π = 324π + 40π = 364π`. The cube of
/// side two moved to `(20, 20, 20)` holds `8` and spans `20` to `22` along each
/// axis, so every point of it is more than `28` from the cone's axis, and the
/// union reaches ten from it at most, at the cone's rim.
///
/// **The two circles were named one curve, and every order was refused
/// `ResultIsNotManifold { count: 4 }`.** The cube cuts nothing of the bar's
/// wall from `z = −15` up to the circle at `z = 1`, but the fuse draws the
/// wall's own edge on the cone again, and the cone's other nappe with it, so
/// the wall comes apart at `z = −1` into two bands. While `assemble.rs` named a
/// closed curve by its two surfaces alone wherever they were not a plane
/// through a ring's axis (`name_of_the_closed_crossing_through`), the new
/// circle at `z = −1` and the wall's circle at `z = 1` were one curve, and
/// `Corners::with_each_closed_curve_started_once` handed the start of the one
/// at `z = −1` to the input corner at `z = 1`. Both circles were then one edge,
/// along which the cone face and the two bands' three wires ran. A **gap**
/// (Q1375): a cone and a pipe on its axis now name a circle by the side it is
/// on of the flat half-space through the apex that `intersect` picks it by.
///
/// **Before that, the bands could not be told apart** (Q1354). The arrangement
/// read which side of the circle at `z = 1` the wall lies on off its one corner
/// off the cone, the seam corner at `z = −15`, which is past the half-space
/// `z ≥ 0` that picks the circle out and inside the other nappe. So the wall was
/// walked inside out and the fuse was refused `CannotTellTheCellsOfAFaceApart`
/// on it. That corner is now read as a node past that half-space is.
#[test]
fn the_union_and_a_far_cube_fuse_to_the_two_apart_in_every_order() {
    let slack = the_two_solids_fall_short_by();
    for cone_first in [true, false] {
        let (model, union) = the_union(cone_first);
        with_a_far_cube_it_is_the_two_apart(
            &model,
            union,
            CONE + BAR - SHARED,
            slack,
            &format!("the union with the bar (cone first: {cone_first})"),
        );
    }
}

/// A ball and a shallower cone on the cone's axis cross both its nappes too,
/// and each one's union with the cone, fused again with the far cube, is the
/// two solids apart, every way round.
///
/// **The ball** of radius five at `(0, 0, 1)` crosses the cone where
/// `(z − 1)² + z² = 25`, which is `(z − 4)(z + 3) = 0`: at `z = 4`, and at
/// `z = −3` on the other nappe. Up to `z = 4` the ball is the wider, so the two
/// share `∫₀⁴ πz² dz = 64π/3` there, and above it the ball's cap up to `z = 6`,
/// `π(2 · 25 − ∫₃⁵ u² du) = π(50 − 98/3) = 52π/3`: `116π/3` in all. The ball
/// holds `500π/3`, so the union holds `(1000 + 500 − 116)π/3 = 1384π/3`.
///
/// **The shallower cone**, of slope one half from its apex at `z = −3` up to
/// `z = 9`, crosses the cone where `|z| = (z + 3)/2`: at `z = 3`, and at
/// `z = −1` on the other nappe. It holds `π · 6² · 12/3 = 144π`. Up to `z = 3`
/// it is the wider, so the two share `∫₀³ πz² dz = 9π` there and
/// `∫₃⁹ π(z + 3)²/4 dz = π(12³ − 6³)/12 = 126π` above it, `135π` in all, and the
/// union holds `1000π/3 + 144π − 135π = 1027π/3`.
///
/// **Named by their two surfaces alone, both circles were one curve**, as the
/// bar's two were, and every order of both was refused
/// `ResultIsNotManifold { count: 4 }`. The fix that tells the bar's two apart
/// tells these apart too: `intersect` picks each circle with a flat half-space
/// square to the axis, the ball's halfway between the apex and its centre, the
/// shallower cone's through the steeper cone's apex.
#[test]
fn a_ball_or_a_shallower_cone_across_both_nappes_and_a_far_cube_fuse_to_the_two_apart() {
    type Pair = fn() -> (Model, SolidKey, SolidKey);
    let pairs: [(&str, Pair, f64, f64); 2] = [
        (
            "a ball",
            the_cone_and_a_ball,
            500.0 * PI / 3.0,
            1384.0 * PI / 3.0,
        ),
        (
            "a shallower cone",
            the_cone_and_a_shallower_cone,
            144.0 * PI,
            1027.0 * PI / 3.0,
        ),
    ];
    for (partner, the_pair, partner_holds, union_holds) in pairs {
        let slack = {
            let (model, cone, other) = the_pair();
            let short = [
                CONE - volume_of(&model, cone),
                partner_holds - volume_of(&model, other),
            ];
            assert!(
                short.iter().all(|&short| short > 0.0),
                "a mesh through points on a convex solid's skin falls inside it: the cone and \
                 {partner} are {short:?} short"
            );
            short[0] + short[1]
        };
        for cone_first in [true, false] {
            let (mut model, cone, other) = the_pair();
            let (arguments, tools) = if cone_first {
                ([cone], [other])
            } else {
                ([other], [cone])
            };
            let what = format!("the union with {partner} (cone first: {cone_first})");
            let answer = model
                .fuse(
                    &arguments,
                    &tools,
                    Rule::Union,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            let [union] = answer.solids()[..] else {
                panic!("{what} is one solid, not {:?}", answer.solids());
            };
            with_a_far_cube_it_is_the_two_apart(&model, union, union_holds, slack, &what);
        }
    }
}

/// Fuses `union` with the cube of side two at `(20, 20, 20)` both ways round,
/// and checks each answer passes [`Model::validate`] and is two solids, holding
/// the cube's `8` and `holds`, each within `slack`.
fn with_a_far_cube_it_is_the_two_apart(
    model: &Model,
    union: SolidKey,
    holds: f64,
    slack: f64,
    what: &str,
) {
    let mut model = model.clone();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole-number cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(20.0, 20.0, 20.0)).expect("a finite move"),
        )
        .expect("this model made the key");
    for (arguments, tools) in [([union], [cube]), ([cube], [union])] {
        let order = format!(
            "{what} and a far cube, union first: {}",
            arguments[0] == union
        );
        let mut fused = model.clone();
        let answer = fused
            .fuse(
                &arguments,
                &tools,
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{order}: {why:?}"));
        fused
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{order}: {why:?}"));
        let mut got: Vec<f64> = answer
            .solids()
            .iter()
            .map(|&solid| volume_of(&fused, solid))
            .collect();
        got.sort_by(f64::total_cmp);
        let want = [8.0, holds];
        assert_eq!(
            got.len(),
            want.len(),
            "{order}: solids holding {got:?}, where the shapes say {want:?}"
        );
        for (got, want) in got.iter().zip(want) {
            assert!(
                (got - want).abs() <= slack,
                "{order} holds {got}, where the shapes say {want} and the bare solids fall \
                 {slack} short"
            );
        }
    }
}

fn the_union(cone_first: bool) -> (Model, SolidKey) {
    let (model, answer) = fused(Rule::Union, cone_first)
        .unwrap_or_else(|why| panic!("the union (cone first: {cone_first}): {why:?}"));
    let [union] = answer[..] else {
        panic!("the union (cone first: {cone_first}) is one solid, not {answer:?}");
    };
    (model, union)
}

fn fused(rule: Rule, cone_first: bool) -> Result<(Model, Vec<SolidKey>), Error> {
    let (mut model, cone, bar) = the_cone_and_the_bar();
    let (arguments, tools) = if cone_first {
        ([cone], [bar])
    } else {
        ([bar], [cone])
    };
    let answer = model.fuse(
        &arguments,
        &tools,
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )?;
    let solids = answer.solids().to_vec();
    Ok((model, solids))
}

fn the_cone_and_the_bar() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
    let bar = model
        .add_round_bar(
            1.0,
            40.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -15.0)).expect("a finite move"),
        )
        .expect("a bar of radius one");
    (model, cone, bar)
}

fn the_cone_and_a_ball() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
    let ball = model
        .add_ball(
            5.0,
            &Isometry::translation(Vector::new(0.0, 0.0, 1.0)).expect("a finite move"),
        )
        .expect("a ball of radius five");
    (model, cone, ball)
}

fn the_cone_and_a_shallower_cone() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
    let shallower = a_pointed_cone(&mut model, 0.5, 12.0);
    model
        .place_solid(
            shallower,
            &Isometry::translation(Vector::new(0.0, 0.0, -3.0)).expect("a finite move"),
        )
        .expect("this model made the key");
    (model, cone, shallower)
}

fn the_two_solids_fall_short_by() -> f64 {
    let (model, cone, bar) = the_cone_and_the_bar();
    let cone_short = CONE - volume_of(&model, cone);
    let bar_short = BAR - volume_of(&model, bar);
    assert!(
        cone_short > 0.0 && bar_short > 0.0,
        "a mesh through points on a convex solid's skin falls inside it: \
         the cone is {cone_short} short and the bar {bar_short}"
    );
    cone_short + bar_short
}

fn volume_of(model: &Model, solid: SolidKey) -> f64 {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(how_fine(), &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a solid measures")
        .volume()
}

fn how_fine() -> Deflection {
    Deflection::of(0.002).expect("a positive length")
}
