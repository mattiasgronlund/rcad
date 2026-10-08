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

/// The union fused again with a cube far from it is refused
/// `ResultIsNotManifold { count: 4 }`, every way round. **A gap**: the answer
/// is the two solids apart, `364π` and the cube's `8`.
///
/// **Its wall is now cut right, and the two circles are then named one
/// curve.** The cube cuts nothing of the bar's wall from `z = −15` up to the
/// circle at `z = 1`, but the fuse draws the wall's own edge on the cone again,
/// and the cone's other nappe with it, so the wall comes apart at `z = −1` into
/// two bands. In `assemble.rs` a closed curve is named by its two surfaces
/// alone wherever they are not a plane through a ring's axis
/// (`name_of_the_closed_crossing_through`), so the new circle at `z = −1` and
/// the wall's circle at `z = 1` are one curve, and
/// `Corners::with_each_closed_curve_started_once` hands the start of the one at
/// `z = −1` to the input corner at `z = 1`. Both circles are then one edge, along
/// which the cone face and the two bands' three wires run.
///
/// **Before that, the bands could not be told apart** (Q1354). The arrangement
/// read which side of the circle at `z = 1` the wall lies on off its one corner
/// off the cone, the seam corner at `z = −15`, which is past the half-space
/// `z ≥ 0` that picks the circle out and inside the other nappe. So the wall was
/// walked inside out and the fuse was refused `CannotTellTheCellsOfAFaceApart`
/// on it. That corner is now read as a node past that half-space is.
#[test]
fn the_union_and_a_far_cube_are_refused_for_naming_both_circles_one_curve() {
    for cone_first in [true, false] {
        let (mut model, union) = the_union(cone_first);
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
            let got = model.clone().fuse(
                &arguments,
                &tools,
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(
                matches!(got, Err(Error::ResultIsNotManifold { count: 4 })),
                "the union (cone first: {cone_first}) and a far cube, union first: {}, came back \
                 {got:?}, where the pin says four faces run along the two circles' one edge",
                arguments[0] == union
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
