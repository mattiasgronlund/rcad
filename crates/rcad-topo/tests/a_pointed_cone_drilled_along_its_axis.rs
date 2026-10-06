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
//! **Three of the six fuses answer, and three are refused a step later.** The
//! union and the bar less the cone keep the bar's band between the two circles,
//! and the corner each circle starts at is named by the two surfaces alone
//! (`MeetPoint::WhereAClosedCurveStarts`). Both circles are where the bar's wall
//! crosses the cone, so they get one corner and one edge, and that edge is
//! claimed four times: by the band's two wires, by the band below and by the cone.
//! `ResultIsNotManifold { count: 4 }`, a gap whose fix is in the name, outside
//! `arrange.rs`.
//!
//! The closed forms, from the slices of the shared piece: inside the bar the cone
//! holds a disc of radius `z` for `z` up to one and the whole unit disc above it,
//! so the two share `π/3 + 9π = 28π/3`. The cone holds `1000π/3`, and the bar
//! `40π`.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

mod cones;

use cones::a_pointed_cone;

const SLOPE: f64 = 1.0;
const HEIGHT: f64 = 10.0;

const CONE: f64 = 1000.0 * PI / 3.0;
const BAR: f64 = 40.0 * PI;
const SHARED: f64 = 28.0 * PI / 3.0;

/// The shared piece, whichever goes first, and the cone less the bar, each one
/// solid that the sweep accepts and that holds its closed form.
///
/// **The slack is what the two bare solids fall short by at this deflection**,
/// measured here rather than written down: a mesh is drawn through points on the
/// surfaces, and no answer holds more curved skin than the cone and the bar
/// together. The bare cone carries no published bound to use instead — a cone
/// face that reaches its apex has a radius of nothing in `measure.rs`.
#[test]
fn the_shared_piece_and_the_cone_less_the_bar_hold_their_closed_forms() {
    let slack = the_two_solids_fall_short_by();
    for (rule, cone_first, want) in [
        (Rule::Intersection, true, SHARED),
        (Rule::Intersection, false, SHARED),
        (Rule::Difference, true, CONE - SHARED),
    ] {
        let (model, answer) = fused(rule, cone_first)
            .unwrap_or_else(|why| panic!("{rule:?} (cone first: {cone_first}): {why:?}"));
        assert_eq!(
            answer.len(),
            1,
            "{rule:?} (cone first: {cone_first}): how many solids"
        );
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?} (cone first: {cone_first}): {why:?}"));
        let got = volume_of(&model, answer[0]);
        assert!(
            (got - want).abs() <= slack,
            "{rule:?} (cone first: {cone_first}) holds {got}, where the shapes say {want} \
             and the bare solids fall {slack} short"
        );
    }
}

/// The union, whichever goes first, and the bar less the cone are refused where
/// the two circles get one corner.
///
/// They would be `CONE + BAR − SHARED = 364π` and `BAR − SHARED = 92π/3`, the
/// second in two solids: the bar below `z = 1` with the cone's tip taken out of
/// its top, and the bar above `z = 10`.
#[test]
fn the_union_and_the_bar_less_the_cone_are_refused_where_two_circles_get_one_corner() {
    for (rule, cone_first) in [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Difference, false),
    ] {
        let refused = fused(rule, cone_first).map(|(_, answer)| answer);
        assert!(
            matches!(refused, Err(Error::ResultIsNotManifold { count: 4 })),
            "{rule:?} (cone first: {cone_first}): {refused:?}"
        );
    }
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
