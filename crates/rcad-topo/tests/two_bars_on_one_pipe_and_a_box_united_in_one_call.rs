//! Two bars stacked on one pipe and a box, all three united: in one call,
//! `fuse([lower], [upper, box])`, and in two.
//!
//! The lower bar stands from `z = 0` to `z = 2` and the upper one from `z = 2`
//! to `z = 4`, both of radius 2, with the pipe they share written four ways
//! ([`Wall`]). The boxes are the seven of the Q44 probe's `the_third`
//! ([`TheBox`]), the ones queue row Q354's lane most likely drew from.
//!
//! # What the row said, and what this base does
//!
//! Q354's lane reported the one-call union refused `ResultIsNotManifold
//! { count: 1 }` in 24 of 24 runs, and did not say which boxes. On this base
//! **no fuse here is refused `ResultIsNotManifold`**: all seven boxes unite in
//! one call to the solid worked out by hand, under every spelling of the wall
//! and both orders of the call, and the same as two calls give
//! ([`one_call_and_two_calls_unite_the_bars_and_each_box_to_the_same_solid`]).
//! The seventh, the block standing on the plane the bars meet on, was refused
//! in one call for another reason until Q1062
//! ([`a_bars_top_cap_flush_on_the_blocks_floor_unites_with_it`]).
//!
//! # Whether the upper bar and the box overlap, and what fuse promises then
//!
//! The upper bar and the box share space for five of the seven boxes: the
//! block over `x < 0` holds half of it, the box across the meeting and the box
//! on the upper wall bite into its wall, the block from `z = 3` up holds its
//! top half, and the block from `z = 2` up holds the whole of it, its floor on
//! the bar's bottom cap. The far cube and the box on the lower wall do not
//! touch it.
//!
//! That is allowed. `Model::fuse`'s doc says so in its second paragraph:
//!
//! > `A` is the union of `arguments` and `B` the union of `tools`.
//!
//! and `fuse/select.rs`'s module doc says how: a piece of one solid buried in
//! another of its own group "is not on the boundary of `A` at all and no rule
//! can want it", so every piece is placed against the rest of its own group
//! first. Nothing asks that the solids of one group be apart, so no fuse here
//! breaks a precondition, and **every refusal in this file is a gap**
//! (`docs/decisions.md` §16.10): its answer exists. For the bars and a box,
//! another way of asking for the same union reaches it.
//!
//! # The refusals, and where each comes from
//!
//! - **The lower bar and the block standing on the meeting, then the upper
//!   bar, answers since Q1080.** The first union answers, `7200 + 8π`. The
//!   second was refused `Undecidable` at "a point in a piece that every solid
//!   could place", on the upper bar's wall, all of it inside the block. Every
//!   point of that wall lies on the pipe the first union's wall is on, which
//!   holds none of them, and the ray from each grazed that wall: the pipe's
//!   sign straddles the point, and so does one root of every ray. `classify.rs`
//!   no longer grazes a ray on a face that holds none of the box it is asked
//!   about. Until Q1062 the first union was refused under that name instead, on
//!   the lower bar's top cap flush on the block's floor, and so was every one
//!   call ([`a_bars_top_cap_flush_on_the_blocks_floor_unites_with_it`]).
//! - **The lower bar and a box through its wall, then the upper bar, answers
//!   since Q1302**
//!   ([`uniting_the_lower_bar_with_the_box_first_and_the_upper_bar_after_is_the_same_union`]).
//!   The first union answers and validates. Until Q1093 the second was refused
//!   `AFaceTrimmedByACurveThisCannotFollow` on that union's wall before any
//!   face was asked whether it holds a point: the arrangement cutting the wall
//!   (`which_way_an_edge_runs_round` in `fuse/arrange.rs`) could not say which
//!   of the two arcs of its circle the bottom edge of the window or notch is,
//!   on `z = 0.5`, or `z = 1` for the box across the meeting. The first union
//!   named that arc by a half-space through both its stored ends, which sit
//!   exactly on its plane at `x = ±1.9364916731037085`. The second fuse solves
//!   each end again from the pipe and two planes, into a box from
//!   `1.9364916731037083` to `1.9364916731037085` across that plane, and
//!   `a_picker_that_tells_the_two_arcs_apart` in `holds.rs`, which asks each
//!   end to be on or in front, cannot tell (Q1063). `holds.rs` now reads the
//!   arc by the middle of the other arc, which is behind the picker
//!   (`into_the_arc_away_from_the_centre`). Since Q1094 the notch answers by
//!   every spelling of the wall, and a window was refused `ResultIsNotManifold`
//!   at one edge by every spelling, opposite the seam too
//!   ([`a_window_cut_opposite_the_seam_unites_the_same_by_every_call`]).
//!
//!   That edge was a piece of the circle the bars meet on. The first union
//!   keeps a corner on that circle wherever two pieces of its wall met, which
//!   `assemble.rs` does on purpose (`what_is_left`), and two of them,
//!   `(1.5, ±1.3229, 2)`, are where the box's inner face `x = 1.5` crossed it.
//!   That face is inside the bar and gone from the first union, so nothing in
//!   the second fuse cuts the upper bar's bottom circle there. The first
//!   union's wall then walked the circle in arcs that end at those corners,
//!   and the upper bar's wall in arcs that pass them, and no edge had two
//!   faces. Each such corner has one wire through it and nothing else, so it
//!   is no corner of any answer, and since Q1302 `assemble.rs` reads that
//!   wire past it (`corners_one_wire_walks_past`).
//! - **The lower bar and Q1406's box, turned an eighth of a turn about `x`,
//!   then the upper bar, is refused** `Undecidable` at "order along a
//!   crossing", by union, intersection and difference alike
//!   ([`the_turned_box_and_the_lower_bar_cannot_be_fused_with_the_upper_bar`]).
//!   The box was built to ask whether `assemble.rs` reads past corners on the
//!   circle the bars meet on when nothing of the first union crosses that
//!   circle there, which the first union of it and the lower bar keeps at
//!   `(1.5, ±√1.75, 2)`. The second fuse gives up in `fuse/arrange.rs` first,
//!   on a corner of the first union on the bottom circle, so that is not
//!   answered yet. One call reaches the union
//!   ([`a_box_turned_an_eighth_of_a_turn_unites_with_the_lower_bar_and_with_both_bars`]).
//! - **A square bar standing on an edge, pushed through the lower bar's wall,
//!   unites with that bar alone lying along `x`**, either way round, since
//!   Q1817 ([`a_square_bar_standing_on_an_edge_along_x_unites_with_the_bar`]).
//!   It was refused `Undecidable` at "order along a crossing" before, on a
//!   corner a hair past where a loop round the pipe starts. **Lying along `y`
//!   it is refused** `CannotNameWhichPieceOfACrossingThisIs`, either way round,
//!   while the result is meshed
//!   ([`a_square_bar_standing_on_an_edge_along_y_is_refused_walking_its_long_ellipse`]).
//!   Its inner end's plane is stored a rounding off the pipe's axis and crosses
//!   the pipe in an ellipse `1.8e16` long. Until Q1818 two corners `0.38` apart
//!   up that ellipse could not be put in order; now they are, and walking a
//!   `1.23`-long arc of it is what fails. Each brace was built as a shape a
//!   designer draws with what Q1406's box has, side faces slanting across the
//!   wall.

use core::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, FRAC_PI_4, PI, SQRT_2};

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog, Record};
use rcad_topo::{DraftStep, Error, Model, OnAFace, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 2.0;

/// Where the two bars meet; each is this tall.
const MEETING: f64 = 2.0;

/// The seven boxes the union of all three answers in one call.
const ANSWERED_IN_ONE_CALL: [TheBox; 7] = [
    TheBox::BlockOverNegativeX,
    TheBox::AcrossTheMeeting,
    TheBox::OnTheUpperWallOnly,
    TheBox::OnTheLowerWallOnly,
    TheBox::FarAway,
    TheBox::TopSlicedOffAbove,
    TheBox::SlicedAtTheMeeting,
];

/// `(the box is the argument of the first fuse, the upper bar is the argument
/// of the second)`, every way.
const EVERY_ORDER_OF_THE_TWO_FUSES: [(bool, bool); 4] =
    [(false, false), (false, true), (true, false), (true, true)];

const EVERY_WALL: [Wall; 4] = [
    Wall::SpelledOnce,
    Wall::OriginSlidUp,
    Wall::AxisTurnedRound,
    Wall::TwoRoundBars,
];

/// **For all seven boxes, one call and two calls give the same union**, by every
/// spelling of the wall: `fuse([lower], [upper, box])`,
/// `fuse([upper, box], [lower])`, and `lower ∪ upper` first with the box after,
/// each the solid or solids [`TheBox::union_by_hand`] works out, and each
/// passing `Model::validate`.
#[test]
fn one_call_and_two_calls_unite_the_bars_and_each_box_to_the_same_solid() {
    for the_box in ANSWERED_IN_ONE_CALL {
        for wall in EVERY_WALL {
            for call in [
                Call::LowerAgainstUpperAndBox,
                Call::UpperAndBoxAgainstLower,
                Call::LowerWithUpperThenTheBox,
            ] {
                let asked = format!("{the_box:?}, {wall:?}, {call:?}");
                let (model, said) = unite(the_box, wall, call);
                let united = said.unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
                is_the_union_by_hand(&model, &united, the_box, &asked);
            }
        }
    }
}

/// **The lower bar and the block standing on its top cap unite**, by every
/// spelling of the wall: `7200 + 8π`, the block and the bar under it, which is
/// what [`TheBox::union_by_hand`] gives the block standing on the meeting,
/// since the block holds the upper bar.
///
/// Refused `Undecidable` at "a point in a piece that every solid could place"
/// until Q1062, on the bar's top cap flush on the block's floor, and every one
/// call with it. The one point found inside the cap is its middle, `(0, 0, 2)`,
/// which is the middle of the floor too and lies on both its diagonals, so the
/// fan of triangles `select.rs` wound the floor's corners in had the point on
/// a spoke from every corner and could not say the floor held it. The cap was
/// never read as lying on the floor, and the ray called its middle on the
/// block's skin. `select.rs` now asks `SolidView::face_holds_the_point` where
/// the fan cannot say.
#[test]
fn a_bars_top_cap_flush_on_the_blocks_floor_unites_with_it() {
    let the_box = TheBox::SlicedAtTheMeeting;
    for wall in EVERY_WALL {
        let asked = format!("{wall:?}, the lower bar and the block");
        let (model, said) = unite(the_box, wall, Call::LowerWithTheBoxAlone);
        let united = said.unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
        is_the_union_by_hand(&model, &united, the_box, &asked);
    }
}

/// **The lower bar united with the box first, and the upper bar after, gives
/// the union by hand for all seven boxes**, by every spelling of the wall: the
/// solid or solids one call gives, and the bars first.
///
/// **The box on the lower wall answers since Q1302**, with each of the two
/// fuses asked both ways round
/// ([`the_box_first_then_the_upper_bar_in_every_order`]): `16π + 4 −
/// (0.5·√3.75 + 4·asin(0.25))`, the bar four tall and the box's `x ∈ [1.5, 4]`,
/// `|y| ≤ 0.5`, one tall, less the strip of the disc of radius 2 it
/// overlaps ([`TheBox::union_by_hand`]). The second fuse was refused
/// `ResultIsNotManifold { count: 1 }`. The first union keeps a corner on the
/// circle the bars meet on wherever two pieces of its wall met, which
/// `assemble.rs` does on purpose (`what_is_left`), and two of them,
/// `(1.5, ±1.3229, 2)`, are where the box's inner face `x = 1.5` crossed it.
/// That face is inside the bar and gone from the first union, so nothing in
/// the second fuse cuts the upper bar's bottom circle there. The union drops
/// both caps on `z = 2`, and the first union's wall then walked the circle in
/// arcs ending at those corners where the upper bar's wall walked past them.
/// Each such corner has one wire through it and nothing else, so it cannot be
/// a corner of any answer, and `assemble.rs` now reads the wire past it
/// (`corners_one_wire_walks_past`), as it already did where a closed edge
/// only started.
///
/// Until Q1093 the box across the meeting was refused too, and both were
/// refused `AFaceTrimmedByACurveThisCannotFollow` naming the first union's
/// wall, from the arrangement that cuts it, which could not read the bottom arc
/// of the window or notch. That arc is read now, and the second fuse refused
/// later, by how the wall is written, until Q1094:
///
/// - with the upper wall's axis turned round, `Undecidable` at "side of
///   cylinder" on a bottom corner of the window or notch, for both boxes. The
///   corner was solved from the lower pipe and two planes and asked of the
///   upper pipe, the same pipe written the other way round, which `bake.rs`
///   took through it unasked only where the two were written alike. It takes
///   any spelling now (`BakedCorner::is_on_by_construction`).
/// - for the box across the meeting, `AFaceTrimmedByACurveThisCannotFollow`
///   naming the first union's notched wall, which no ray fired from the upper
///   bar's bottom cap could be placed past
///   ([`a_notched_wall_holds_no_point_of_its_pipe_above_its_top`]). It answers
///   now, by every spelling of the wall.
///
/// The other five answer by hand. Two of them were refused at the second fuse,
/// under "a point in a piece that every solid could place", because the ray
/// from every point of the upper bar's wall grazed the first union's wall on
/// the same pipe:
///
/// - the block over `x < 0` until Q1064, on the half of the upper wall at
///   `x ≥ 0`, which `select.rs` placed by its sides on `x = 0`, cuts of the
///   block's face there;
/// - the block standing on the meeting until Q1080, on the whole of the upper
///   wall, whose corners are the two ends of its seam and none of whose sides
///   is a cut of a face of the block. Until Q1062 it was refused at the first
///   union instead ([`a_bars_top_cap_flush_on_the_blocks_floor_unites_with_it`]).
///
/// The pipe's sign straddles each point of the upper wall, and one root of
/// every ray lands a rounding either side of the point, and the lower wall
/// holds neither. `classify.rs` grazed every ray on that until Q1080, and now
/// grazes only where the face may hold the box.
#[test]
fn uniting_the_lower_bar_with_the_box_first_and_the_upper_bar_after_is_the_same_union() {
    for wall in EVERY_WALL {
        for the_box in [
            TheBox::BlockOverNegativeX,
            TheBox::AcrossTheMeeting,
            TheBox::OnTheUpperWallOnly,
            TheBox::FarAway,
            TheBox::TopSlicedOffAbove,
            TheBox::SlicedAtTheMeeting,
        ] {
            let call = Call::LowerWithTheBoxThenUpper {
                the_box_is_the_argument: false,
                the_upper_bar_is_the_argument: false,
            };
            let asked = format!("{the_box:?}, {wall:?}");
            let (model, said) = unite(the_box, wall, call);
            let united = said.unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
            is_the_union_by_hand(&model, &united, the_box, &asked);
        }

        the_box_first_then_the_upper_bar_in_every_order(TheBox::OnTheLowerWallOnly, wall);
    }
}

/// **A window cut through the lower bar's wall opposite its seam unites the
/// same by every call**, the mirror image of the box on the lower wall's
/// union, and holding as much: in one call, with the bars first, and with the
/// box first and each fuse asked both ways round. It was refused
/// `ResultIsNotManifold { count: 1 }` at the second fuse until Q1302, the same
/// as the box on the lower wall, so the seam running through that window was
/// never what the second fuse could not follow
/// ([`uniting_the_lower_bar_with_the_box_first_and_the_upper_bar_after_is_the_same_union`]
/// says what it was).
#[test]
fn a_window_cut_opposite_the_seam_unites_the_same_by_every_call() {
    let the_box = TheBox::OnTheLowerWallOppositeTheSeam;
    for wall in EVERY_WALL {
        the_box_first_then_the_upper_bar_in_every_order(the_box, wall);
        for call in [
            Call::LowerAgainstUpperAndBox,
            Call::UpperAndBoxAgainstLower,
            Call::LowerWithUpperThenTheBox,
        ] {
            let asked = format!("{wall:?}, {call:?}");
            let (model, said) = unite(the_box, wall, call);
            let united = said.unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
            is_the_union_by_hand(&model, &united, the_box, &asked);
        }
    }
}

/// **The notched wall of the lower bar united with the box across the meeting
/// holds no point of its pipe above its top**, by every spelling of the wall:
/// `SolidView::face_holds_the_point` says `NotHeld` at `(2, 0, 3)`, `(0, 2, 3)`
/// and `(-2, 0, 3)`, where it refused `AFaceTrimmedByACurveThisCannotFollow`
/// until Q1094. The wall is one face, and its notch leaves no reading of its
/// sides in front of every cut, but every side of it is on or under its top
/// cap's plane, and a point of the pipe above that plane is not on a face
/// bounded below it (`in_front_of_a_plane_every_side_is_behind` in
/// `holds.rs`). That is what the rays fired at it from the upper bar's
/// bottom cap met, and the second union refused for it.
#[test]
fn a_notched_wall_holds_no_point_of_its_pipe_above_its_top() {
    let policy = Policy::certified();
    for wall in EVERY_WALL {
        let asked = format!("{wall:?}");
        let (model, said) = unite(TheBox::AcrossTheMeeting, wall, Call::LowerWithTheBoxAlone);
        let united = said.unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
        let [solid] = united.as_slice() else {
            panic!("{asked}: the lower bar and the box are one solid: {united:?}");
        };
        let solid = model.solid(*solid).expect("this model made the key");
        let walls: Vec<_> = solid
            .faces()
            .filter(|face| matches!(face.surface(), Surface::Cylinder(_)))
            .map(|face| face.key())
            .collect();
        let [notched] = walls.as_slice() else {
            panic!("{asked}: the wall is one face: {walls:?}");
        };
        for above in [[2.0, 0.0, 3.0], [0.0, 2.0, 3.0], [-2.0, 0.0, 3.0]] {
            let held = solid
                .face_holds_the_point(
                    *notched,
                    PredicatePoint::from(above),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|refused| panic!("{asked}, {above:?}: {refused:?}"));
            assert_eq!(held, OnAFace::NotHeld, "{asked}, {above:?}");
        }
    }
}

/// **Q1406's box, a block turned an eighth of a turn about `x`, unites with
/// the lower bar either way round, and with both bars in one call or with the
/// bars first**, by every spelling of the wall: `8π` and `16π` each, plus what
/// of the box is outside the bars ([`the_turned_box_outside_the_bars`]).
///
/// The union of the lower bar and the box is the shape Q1406 asked for. It
/// keeps two corners on the circle the bars meet on, `(1.5, ±√1.75, 2)`, where
/// the box's face `x = 1.5` crossed it, and that face is inside the bar and
/// gone from the union, so nothing of the union crosses that circle there. It
/// keeps a corner on the bottom circle at `(-√(4 - k²), -k, 0)`, `k = 0.1·√2`,
/// too, where the plane of the box's face `y + z = -k` crosses it on the side
/// of the pipe away from the box. Uniting the upper bar with it is refused
/// there ([`the_turned_box_and_the_lower_bar_cannot_be_fused_with_the_upper_bar`]),
/// so that refusal is a gap: the answer exists, and these calls reach it.
#[test]
fn a_box_turned_an_eighth_of_a_turn_unites_with_the_lower_bar_and_with_both_bars() {
    let one_bar = PI * RADIUS * RADIUS * MEETING;
    let outside = the_turned_box_outside_the_bars();
    let k = 0.1 * SQRT_2;
    let kept = [
        [1.5, 1.75_f64.sqrt(), MEETING],
        [1.5, -1.75_f64.sqrt(), MEETING],
        [-(RADIUS * RADIUS - k * k).sqrt(), -k, 0.0],
    ];
    for wall in EVERY_WALL {
        for call in [Call::LowerWithTheBoxAlone, Call::TheBoxWithTheLowerAlone] {
            let asked = format!("the turned box, {wall:?}, {call:?}");
            let (model, said) = unite_with(wall, call, a_box_turned_an_eighth_of_a_turn_about_x);
            let united = said.unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
            is_the_solid_worked_out(&model, &united, (1, one_bar + outside), &asked);
            let corners = its_corners(&model, united[0]);
            for corner in kept {
                assert!(
                    corners.iter().any(|&at| lies_within(
                        at.map(|along| [along, along]),
                        corner,
                        1e-12
                    )),
                    "{asked}: no corner at {corner:?} among {corners:?}"
                );
            }
        }
        for call in [
            Call::LowerAgainstUpperAndBox,
            Call::UpperAndBoxAgainstLower,
            Call::LowerWithUpperThenTheBox,
        ] {
            let asked = format!("the turned box, {wall:?}, {call:?}");
            let (model, said) = unite_with(wall, call, a_box_turned_an_eighth_of_a_turn_about_x);
            let united = said.unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
            is_the_solid_worked_out(&model, &united, (1, 2.0 * one_bar + outside), &asked);
        }
    }
}

/// **The lower bar united with Q1406's turned box cannot be fused with the
/// upper bar**: not by union, intersection or difference, with each fuse asked
/// both ways round, by any spelling of the wall. Each is refused `Undecidable`
/// at "order along a crossing", on two points within `10⁻¹²` of
/// `(-√(4 - k²), -k, 0)`, with `k = 0.1·√2`, and what it worked out to order
/// them has an enclosure under `10⁻¹⁴`. They are the first union's own corner
/// where the plane of the box's face `y + z = -k` crosses the bar's bottom
/// circle, on the far side of the pipe from the box, and a point worked out
/// round it.
///
/// The union exists, and one call reaches it
/// ([`a_box_turned_an_eighth_of_a_turn_unites_with_the_lower_bar_and_with_both_bars`]),
/// so this is a gap. It is refused in `fuse/arrange.rs`, before `assemble.rs`
/// runs. So this shape does not yet say what Q1406 asked: whether the corners
/// the first union keeps on the circle the bars meet on, at `(1.5, ±√1.75, 2)`
/// where the box's face `x = 1.5` crossed it, are read past
/// (`corners_one_wire_walks_past`) when nothing of the box crosses that circle.
#[test]
fn the_turned_box_and_the_lower_bar_cannot_be_fused_with_the_upper_bar() {
    let k = 0.1 * SQRT_2;
    let on_the_bottom_circle = [-(RADIUS * RADIUS - k * k).sqrt(), -k, 0.0];
    for wall in EVERY_WALL {
        for order in EVERY_ORDER_OF_THE_TWO_FUSES {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                let asked = format!("{wall:?}, {order:?}, {rule:?}");
                let second = the_turned_box_and_the_lower_bar_then_the_upper_bar(wall, order, rule);
                let Err(why) = &second else {
                    panic!("{asked}: answered {second:?}");
                };
                assert_eq!(
                    what_it_gave_up_on(why),
                    "order along a crossing",
                    "{asked}: {why:?}"
                );
                let points = the_points_it_could_not_order(why);
                assert_eq!(points.len(), 2, "{asked}: {why:?}");
                for point in points {
                    assert!(
                        lies_within(point, on_the_bottom_circle, 1e-12),
                        "{asked}: {point:?}"
                    );
                }
                let enclosure = the_last_record(why).enclosure();
                assert!(enclosure < 1e-14, "{asked}: {enclosure}");
            }
        }
    }
}

/// **A square bar standing on an edge along `x`, pushed through the lower
/// bar's wall, unites with it**, either way round, by any spelling of the wall
/// ([`a_diamond_brace_along_x`]): one solid, `8π` and what of the brace is
/// outside the bar ([`the_diamond_brace_outside_the_bar`]).
///
/// It was refused `Undecidable` at "order along a crossing" until Q1817. The
/// brace's top edge `y = 0, z = 1 + √½` lies on the bar's seam plane, and the
/// loop where the plane of its face `z - y = 1 + √½` cuts the pipe is walked
/// from `(2, 0, 1 + √½)`: exactly on the pipe, `6.8e-17` off that plane. The
/// top edge meets the pipe at `y = -1.3e-16`, a hair past the start, and the
/// area the start, that corner and the next one round, `(√3.5, -√½, 1)`, sweep
/// is about `3.5e-17`, which the corner's box, `2.2e-15` wide along `x`,
/// swamps. Read as an angle about the bar's axis instead, the corner is a
/// certified hair past the start, since the start's `y` is exactly nothing,
/// and the next corner is past it (`round_a_closed_crossing_by_its_angle` in
/// `fuse/arrange.rs`). The brace's face `z - y = 1 - √½`, with the brace as
/// the argument, was refused the same way, from `(-2, 0, 1 - √½)`.
///
/// Built as a shape a designer draws with what Q1406's box has, side faces
/// slanting across the wall, to see whether that box's refusal
/// ([`the_turned_box_and_the_lower_bar_cannot_be_fused_with_the_upper_bar`])
/// meets designers. It did not reach that fuse: it was refused one fuse
/// earlier.
#[test]
fn a_square_bar_standing_on_an_edge_along_x_unites_with_the_bar() {
    let one_bar = PI * RADIUS * RADIUS * MEETING;
    let outside = the_diamond_brace_outside_the_bar();
    for wall in EVERY_WALL {
        for call in [Call::LowerWithTheBoxAlone, Call::TheBoxWithTheLowerAlone] {
            let asked = format!("the diamond brace along x, {wall:?}, {call:?}");
            let (model, said) = unite_with(wall, call, a_diamond_brace_along_x);
            let united = said.unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
            is_the_solid_worked_out(&model, &united, (1, one_bar + outside), &asked);
        }
    }
}

/// **A square bar standing on an edge along `y`, pushed through the lower
/// bar's wall, gets its corners put in order and is then refused walking the
/// long ellipse its inner end cuts the pipe in**:
/// `CannotNameWhichPieceOfACrossingThisIs`, either way round, by any spelling
/// of the wall ([`a_diamond_brace_along_y`]). The answer exists, since
/// swapping `x` and `y` carries the brace onto [`a_diamond_brace_along_x`] and
/// the bar onto itself, and that brace unites: `8π` and
/// [`the_diamond_brace_outside_the_bar`]. So this is a gap.
///
/// The brace is turned a quarter turn about `x`, and `cos(π/2)` rounds to
/// `6.1e-17`, so the plane of its inner end is stored as `-y - 2.2e-16·z =
/// -1.5000000000000002`: not quite along the pipe's axis. It crosses the pipe
/// in an ellipse `1.8e16` long, where `y = 1.5` would cross it in two lines
/// `x = ±√1.75`, and the walk starts it at `(2, 0, 6755399441055745)`.
///
/// Until Q1818 it was refused `Undecidable` at "order along a crossing", on
/// two corners of that ellipse `0.38` apart up the axis, `(-√1.75, 1.5, 1 +
/// √1.75 - √½)` and `(-√1.75, 1.5, 2)`: the first where it meets the plane
/// `x + z = 1 - √½` of one of the brace's faces, the second on the bar's top
/// cap. Their `y` differ by `2.2e-16 · 0.38`, under an ulp of `1.5`, so seen
/// down the axis their boxes overlap, and the area they sweep with that start
/// came out with an enclosure of `15`. Read off how far apart they are along
/// the axis instead, they are proved in order
/// (`round_a_closed_crossing_by_its_angle` in `fuse/arrange.rs`).
///
/// The arrangement after that is the brace along `x`'s with `x` and `y`
/// swapped, `28` faces. Asked as `lower ∪ brace` with the wall spelled once,
/// the fuse gives up when `nest_the_shells` meshes it to measure how much
/// space it holds, in `Conic::the_stretch_this_run_covers` in `rcad-geom`'s
/// `walk.rs`, on the wall's edge from `(√1.75, 1.5, 1 - √1.75 + √½)` to
/// `(√1.75, 1.5, 1 + √1.75 - √½)`, `1.23` long on that ellipse. Its ends come
/// out at one angle round the ellipse, `0.8480620789814812` and
/// `0.8480620789814809`, so the end comes before the start. The middle of the
/// short arc between them, placed from the ellipse's centre `6.8e15` up the
/// axis, lands at `z = -1`, `2` below the edge's own middle. The plane
/// `assemble.rs` names the arc by, through its chord, has a normal of
/// rounding, `(-2.2e-16, -4.5e-16, 0)`, so both arcs of the ellipse are
/// picked.
#[test]
fn a_square_bar_standing_on_an_edge_along_y_is_refused_walking_its_long_ellipse() {
    for wall in EVERY_WALL {
        for call in [Call::LowerWithTheBoxAlone, Call::TheBoxWithTheLowerAlone] {
            let asked = format!("the diamond brace along y, {wall:?}, {call:?}");
            let (_, said) = unite_with(wall, call, a_diamond_brace_along_y);
            assert!(
                matches!(
                    said,
                    Err(Error::Geometry(
                        rcad_geom::Error::CannotNameWhichPieceOfACrossingThisIs
                    ))
                ),
                "{asked}: {said:?}"
            );
        }
    }
}

/// `(lower ∪ box) ∪ upper` with each fuse asked both ways round, the four
/// calls of [`EVERY_ORDER_OF_THE_TWO_FUSES`], each the union by hand.
fn the_box_first_then_the_upper_bar_in_every_order(the_box: TheBox, wall: Wall) {
    for (the_box_is_the_argument, the_upper_bar_is_the_argument) in EVERY_ORDER_OF_THE_TWO_FUSES {
        let call = Call::LowerWithTheBoxThenUpper {
            the_box_is_the_argument,
            the_upper_bar_is_the_argument,
        };
        let asked = format!("{the_box:?}, {wall:?}, {call:?}");
        let (model, said) = unite(the_box, wall, call);
        let united = said.unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
        is_the_union_by_hand(&model, &united, the_box, &asked);
    }
}

/// The two bars stacked on one pipe, each [`MEETING`] tall.
#[derive(Clone, Copy, Debug)]
enum Wall {
    /// Both walls name the axis at the origin, pointing up.
    SpelledOnce,
    /// The upper wall names its axis at `z = 2`.
    OriginSlidUp,
    /// The upper wall names its axis pointing down.
    AxisTurnedRound,
    /// Both from `Model::add_round_bar(2.0, 2.0)`, the upper one placed at
    /// `z = 2`, so its wall names its axis there.
    TwoRoundBars,
}

/// The seven boxes of the Q44 probe's `the_third`, and one more.
#[derive(Clone, Copy, Debug)]
enum TheBox {
    /// Not one of the seven: the box on the lower wall turned half way round
    /// the axis, so the window it cuts is opposite the seam.
    OnTheLowerWallOppositeTheSeam,
    /// `x ∈ [-10, 0]`, reaching past both bars every other way.
    BlockOverNegativeX,
    /// Two on a side across the meeting, through both walls at the seam.
    AcrossTheMeeting,
    /// Through the upper wall at the seam, clear of the lower bar.
    OnTheUpperWallOnly,
    /// Through the lower wall at the seam, clear of the upper bar.
    OnTheLowerWallOnly,
    /// A cube two on a side at `(20, 20, 20)`.
    FarAway,
    /// From `z = 3` up, reaching past both bars every other way.
    TopSlicedOffAbove,
    /// From `z = 2` up, its floor on the upper bar's bottom cap.
    SlicedAtTheMeeting,
}

impl TheBox {
    fn bounds(self) -> [[f64; 2]; 3] {
        let thin = [-0.5, 0.5];
        let wide = [-10.0, 10.0];
        match self {
            Self::BlockOverNegativeX => [[-10.0, 0.0], wide, [-3.0, 10.0]],
            Self::AcrossTheMeeting => [[1.0, 3.0], thin, [1.0, 3.0]],
            Self::OnTheUpperWallOnly => [[1.5, 4.0], thin, [2.5, 3.5]],
            Self::OnTheLowerWallOnly => [[1.5, 4.0], thin, [0.5, 1.5]],
            Self::OnTheLowerWallOppositeTheSeam => [[-4.0, -1.5], thin, [0.5, 1.5]],
            Self::FarAway => [[20.0, 22.0], [20.0, 22.0], [20.0, 22.0]],
            Self::TopSlicedOffAbove => [wide, wide, [3.0, 20.0]],
            Self::SlicedAtTheMeeting => [wide, wide, [MEETING, 20.0]],
        }
    }

    /// How many solids the union of the two bars and this box is, and how much
    /// it holds.
    ///
    /// The bars together are a bar four tall, `16π`. A thin box through the
    /// wall adds what of it is outside the pipe: its cross-section,
    /// `y ∈ [-0.5, 0.5]` out to `x = reach`, less the part of the disc of
    /// radius 2 in that strip, times its height.
    fn union_by_hand(self) -> (usize, f64) {
        let both_bars = PI * RADIUS * RADIUS * 2.0 * MEETING;
        let strip_of_the_disc =
            0.5 * (RADIUS * RADIUS - 0.25).sqrt() + RADIUS * RADIUS * (0.5 / RADIUS).asin();
        let outside_the_pipe = |reach: f64| reach - strip_of_the_disc;
        match self {
            Self::BlockOverNegativeX => (1, 10.0 * 20.0 * 13.0 + both_bars / 2.0),
            Self::AcrossTheMeeting => (1, both_bars + 2.0 * outside_the_pipe(3.0)),
            Self::OnTheUpperWallOnly
            | Self::OnTheLowerWallOnly
            | Self::OnTheLowerWallOppositeTheSeam => (1, both_bars + outside_the_pipe(4.0)),
            Self::FarAway => (2, both_bars + 8.0),
            Self::TopSlicedOffAbove => (1, 20.0 * 20.0 * 17.0 + PI * RADIUS * RADIUS * 3.0),
            Self::SlicedAtTheMeeting => (1, 20.0 * 20.0 * 18.0 + both_bars / 2.0),
        }
    }
}

/// Which fuse or fuses unite the three.
#[derive(Clone, Copy, Debug)]
enum Call {
    /// `fuse([lower], [upper, box])`.
    LowerAgainstUpperAndBox,
    /// `fuse([upper, box], [lower])`.
    UpperAndBoxAgainstLower,
    /// `fuse([lower], [upper])`, then that with the box.
    LowerWithUpperThenTheBox,
    /// `fuse([lower], [box])`, then `fuse([that], [upper])`, each the other way
    /// round where it says the second is the argument.
    LowerWithTheBoxThenUpper {
        the_box_is_the_argument: bool,
        the_upper_bar_is_the_argument: bool,
    },
    /// `fuse([lower], [box])` and no more, the upper bar left out.
    LowerWithTheBoxAlone,
    /// `fuse([box], [lower])` and no more, the upper bar left out.
    TheBoxWithTheLowerAlone,
}

fn unite(the_box: TheBox, wall: Wall, call: Call) -> (Model, Result<Vec<SolidKey>, Error>) {
    unite_with(wall, call, |model| a_box(model, the_box.bounds()))
}

/// The two bars and the solid `third` builds, united as `call` says.
fn unite_with(
    wall: Wall,
    call: Call,
    third: impl FnOnce(&mut Model) -> SolidKey,
) -> (Model, Result<Vec<SolidKey>, Error>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let [lower, upper] = the_two_bars(&mut model, wall);
    let third = third(&mut model);
    let union = |model: &mut Model, arguments: &[SolidKey], tools: &[SolidKey]| {
        fuse_by(model, arguments, tools, Rule::Union)
    };
    let said = match call {
        Call::LowerAgainstUpperAndBox => union(&mut model, &[lower], &[upper, third]),
        Call::UpperAndBoxAgainstLower => union(&mut model, &[upper, third], &[lower]),
        Call::LowerWithUpperThenTheBox => union(&mut model, &[lower], &[upper])
            .and_then(|bars| union(&mut model, &bars, &[third])),
        Call::LowerWithTheBoxThenUpper {
            the_box_is_the_argument,
            the_upper_bar_is_the_argument,
        } => {
            let first = if the_box_is_the_argument {
                union(&mut model, &[third], &[lower])
            } else {
                union(&mut model, &[lower], &[third])
            };
            first.and_then(|first| {
                if the_upper_bar_is_the_argument {
                    union(&mut model, &[upper], &first)
                } else {
                    union(&mut model, &first, &[upper])
                }
            })
        }
        Call::LowerWithTheBoxAlone => union(&mut model, &[lower], &[third]),
        Call::TheBoxWithTheLowerAlone => union(&mut model, &[third], &[lower]),
    };
    (model, said)
}

/// The lower bar united with [`a_box_turned_an_eighth_of_a_turn_about_x`],
/// then that fused with the upper bar by `rule`, each fuse the other way round
/// where `order` says the second is the argument.
fn the_turned_box_and_the_lower_bar_then_the_upper_bar(
    wall: Wall,
    (the_box_is_the_argument, the_upper_bar_is_the_argument): (bool, bool),
    rule: Rule,
) -> Result<Vec<SolidKey>, Error> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let [lower, upper] = the_two_bars(&mut model, wall);
    let the_box = a_box_turned_an_eighth_of_a_turn_about_x(&mut model);
    let first = if the_box_is_the_argument {
        fuse_by(&mut model, &[the_box], &[lower], Rule::Union)
    } else {
        fuse_by(&mut model, &[lower], &[the_box], Rule::Union)
    }
    .expect("the lower bar and the turned box unite");
    if the_upper_bar_is_the_argument {
        fuse_by(&mut model, &[upper], &first, rule)
    } else {
        fuse_by(&mut model, &first, &[upper], rule)
    }
}

fn fuse_by(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Result<Vec<SolidKey>, Error> {
    model
        .fuse(
            arguments,
            tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec())
}

fn is_the_union_by_hand(model: &Model, united: &[SolidKey], the_box: TheBox, asked: &str) {
    is_the_solid_worked_out(model, united, the_box.union_by_hand(), asked);
}

/// Whether `united` is `solids` solids holding `by_hand` between them, within
/// the bound the measure gives, and the model validates.
fn is_the_solid_worked_out(
    model: &Model,
    united: &[SolidKey],
    (solids, by_hand): (usize, f64),
    asked: &str,
) {
    let policy = Policy::certified();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|broken| panic!("{asked}: {broken:?}"));
    assert_eq!(united.len(), solids, "{asked}: {united:?}");
    let (mut volume, mut bound) = (0.0, 0.0);
    for &solid in united {
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
        volume += measured.volume();
        bound += measured.volume_error();
    }
    assert!(
        (volume - by_hand).abs() <= bound,
        "{asked}: {volume} ± {bound} against {by_hand}"
    );
}

/// Where every side of every face of `solid` starts.
fn its_corners(model: &Model, solid: SolidKey) -> Vec<[f64; 3]> {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .flat_map(|face| face.wires())
        .flat_map(|wire| wire.coedges())
        .map(|side| side.start().point().coordinates())
        .collect()
}

fn the_two_bars(model: &mut Model, wall: Wall) -> [SolidKey; 2] {
    let (upper_axis_at, upper_turned_round) = match wall {
        Wall::TwoRoundBars => {
            let placed_at =
                |z: f64| Isometry::translation(Vector::new(0.0, 0.0, z)).expect("a finite shift");
            return [0.0, MEETING].map(|bottom| {
                model
                    .add_round_bar(RADIUS, MEETING, &placed_at(bottom))
                    .expect("a bar")
            });
        }
        Wall::SpelledOnce => (0.0, false),
        Wall::OriginSlidUp => (MEETING, false),
        Wall::AxisTurnedRound => (0.0, true),
    };
    [
        a_bar(model, 0.0, 0.0, false),
        a_bar(model, MEETING, upper_axis_at, upper_turned_round),
    ]
}

/// A bar from `bottom` up [`MEETING`], whose wall names its axis at
/// `z = axis_at`, pointing down when `turned_round`.
///
/// Built face by face, since `Model::add_round_bar` places the whole solid and
/// so cannot write the axis anywhere but at its own bottom.
fn a_bar(model: &mut Model, bottom: f64, axis_at: f64, turned_round: bool) -> SolidKey {
    let top = bottom + MEETING;
    let round = |turn: Orientation| if turned_round { turn.reversed() } else { turn };
    let mut builder = SolidBuilder::new();
    let low = builder
        .add_vertex(Point::new(RADIUS, 0.0, bottom))
        .expect("a finite corner");
    let high = builder
        .add_vertex(Point::new(RADIUS, 0.0, top))
        .expect("a finite corner");
    let seam = builder
        .add_edge(low, high, EdgeRun::Straight)
        .expect("the seam up the wall");
    let low_circle = builder
        .add_edge(low, low, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the bottom cap circle");
    let high_circle = builder
        .add_edge(high, high, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the top cap circle");

    let cap = |builder: &mut SolidBuilder, z: f64, outward: f64, edge, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
        let coedge = builder.add_coedge(edge, turn).expect("one use of the rim");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("a cap is bounded by one whole circle");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a flat cap")
    };
    let bottom_cap = cap(
        &mut builder,
        bottom,
        -1.0,
        low_circle,
        round(Orientation::Reversed),
    );
    let top_cap = cap(
        &mut builder,
        top,
        1.0,
        high_circle,
        round(Orientation::Forward),
    );

    let round_the_wall = [
        (low_circle, round(Orientation::Forward)),
        (seam, Orientation::Forward),
        (high_circle, round(Orientation::Reversed)),
        (seam, Orientation::Reversed),
    ]
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("a step round the wall"),
        )
    });
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("the wall's one wire");
    let pipe = Cylinder::new(
        Axis::new(
            Point::new(0.0, 0.0, axis_at),
            Vector::new(0.0, 0.0, if turned_round { -1.0 } else { 1.0 }),
        )
        .expect("a direction with a length"),
        RADIUS,
    )
    .expect("a positive radius");
    let wall = builder
        .add_face(Surface::Cylinder(pipe), &[wire])
        .expect("the curved wall");

    builder
        .add_shell(&[
            (bottom_cap, Orientation::Forward),
            (top_cap, Orientation::Forward),
            (wall, Orientation::Forward),
        ])
        .expect("a closed skin");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar described face by face")
}

/// Q1406's box: `x ∈ [1.5, 4]`, `(y + z)/√2 ∈ [-2, -0.1]` and
/// `(z - y)/√2 ∈ [-0.4, 3.2]`, a block turned an eighth of a turn about `x`.
///
/// It pokes out of the lower bar's wall and bottom cap and comes nowhere near
/// the circle the bars meet on. Of the planes its faces lie on, only `x = 1.5`
/// crosses that circle, at `(1.5, ±√1.75, 2)`, where that face is not: on the
/// pipe, `y + z = -0.1·√2` reaches up to `z = 2 - 0.1·√2`, `y + z = -2·√2` to
/// `2 - 2·√2` and `z - y = -0.4·√2` to `2 - 0.4·√2`, and `z - y = 3.2·√2`
/// comes no lower than `3.2·√2 - 2`.
fn a_box_turned_an_eighth_of_a_turn_about_x(model: &mut Model) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[1.5, -2.0], [4.0, -2.0], [4.0, -0.1], [1.5, -0.1]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let about_x = Isometry::rotation(&Axis::x(), FRAC_PI_4).expect("a finite turn");
    let down = Isometry::translation(Vector::new(0.0, 0.0, -0.4)).expect("a finite shift");
    model
        .add_extrusion(&rectangle, 3.6, &about_x.compose(&down))
        .expect("a positive height")
}

/// How much of [`a_box_turned_an_eighth_of_a_turn_about_x`] is outside the
/// bars: all of it, `2.5 · 1.9 · 3.6`, less what it shares with the lower bar.
///
/// It shares nothing with the upper bar, since where it rises past `z = 2` it
/// is at `y ≤ -2 - 0.1·√2`. Inside the lower bar its other faces are out of
/// reach, so what it shares is where `x ≥ 1.5`, `x² + y² ≤ 4` and
/// `0 ≤ z ≤ -k - y`, with `k = 0.1·√2`: with `u = -y` running from `k` to
/// `r = √1.75`, the integral of `(√(4 - u²) - 1.5)·(u - k)`.
fn the_turned_box_outside_the_bars() -> f64 {
    let k = 0.1 * SQRT_2;
    let r = 1.75_f64.sqrt();
    let under_the_circle = |u: f64| u * (4.0 - u * u).sqrt() / 2.0 + 2.0 * (u / 2.0).asin();
    let its_moment = |u: f64| -(4.0 - u * u).powf(1.5) / 3.0;
    let shared = its_moment(r)
        - its_moment(k)
        - k * (under_the_circle(r) - under_the_circle(k))
        - 0.75 * (r - k) * (r - k);
    2.5 * 1.9 * 3.6 - shared
}

/// How much of [`a_diamond_brace_along_x`] is outside the lower bar: all of
/// it, `2.5`, less what it shares with the bar.
///
/// The brace stays between `z = 1 - √½` and `1 + √½`, inside the bar's height,
/// so what it shares is where `x ≥ 1.5` and `x² + y² ≤ 4`. At each `y` its
/// cross-section is `2·(h - |y|)` tall, with `h = √½`, and reaches into the
/// bar from `x = 1.5` to `√(4 - y²)`. So it shares the integral of
/// `(√(4 - y²) - 1.5)·2·(h - |y|)` over `|y| ≤ h`, which is four times that
/// of `(√(4 - y²) - 1.5)·(h - y)` from `0` to `h`.
fn the_diamond_brace_outside_the_bar() -> f64 {
    let h = FRAC_1_SQRT_2;
    let under_the_circle = h * (4.0 - h * h).sqrt() / 2.0 + 2.0 * (h / 2.0).asin();
    let its_moment = (8.0 - (4.0 - h * h).powf(1.5)) / 3.0;
    let shared = 4.0 * (h * under_the_circle - its_moment - 0.75 * h * h);
    2.5 - shared
}

/// A square bar of side 1 from `x = 1.5` to `4`, turned an eighth of a turn
/// about `x` so it stands on an edge, its middle on `z = 1`. Its four long
/// edges lie on the lines `y = 0, z = 1 ± √½` and `y = ±√½, z = 1`.
fn a_diamond_brace_along_x(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![[1.5, -0.5], [4.0, -0.5], [4.0, 0.5], [1.5, 0.5]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let up = Isometry::translation(Vector::new(0.0, 0.0, 1.0)).expect("a finite shift");
    let about_x = Isometry::rotation(&Axis::x(), FRAC_PI_4).expect("a finite turn");
    let half_down = Isometry::translation(Vector::new(0.0, 0.0, -0.5)).expect("a finite shift");
    model
        .add_extrusion(&square, 1.0, &up.compose(&about_x).compose(&half_down))
        .expect("a positive height")
}

/// The same square bar standing on an edge, from `y = 1.5` to `4`, its middle
/// on `x = 0, z = 1`: its faces are on the planes `|x| + |z - 1| = √½`.
fn a_diamond_brace_along_y(model: &mut Model) -> SolidKey {
    let diamond = ProfileBuilder::new(vec![
        [0.0, -FRAC_1_SQRT_2],
        [FRAC_1_SQRT_2, 0.0],
        [0.0, FRAC_1_SQRT_2],
        [-FRAC_1_SQRT_2, 0.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square standing on a corner");
    let up = Isometry::translation(Vector::new(0.0, 0.0, 1.0)).expect("a finite shift");
    let lying_along_y = Isometry::rotation(&Axis::x(), -FRAC_PI_2).expect("a finite turn");
    let out = Isometry::translation(Vector::new(0.0, 0.0, 1.5)).expect("a finite shift");
    model
        .add_extrusion(&diamond, 2.5, &up.compose(&lying_along_y).compose(&out))
        .expect("a positive height")
}

/// The name of the predicate a refusal gave up on.
fn what_it_gave_up_on(why: &Error) -> &str {
    the_last_record(why).predicate()
}

/// The points the refusal's last decision was asked about, each as a box.
fn the_points_it_could_not_order(why: &Error) -> Vec<[[f64; 2]; 3]> {
    the_last_record(why)
        .inputs()
        .iter()
        .map(|point| point.enclosure().map(|along| [along.lo(), along.hi()]))
        .collect()
}

fn the_last_record(why: &Error) -> &Record {
    let Error::Undecidable { records } = why else {
        panic!("expected `Undecidable`, got {why:?}");
    };
    records
        .records()
        .last()
        .expect("a refusal names the decision it gave up on")
}

/// Whether the box lies within `reach` of `point` along every axis.
fn lies_within(node: [[f64; 2]; 3], point: [f64; 3], reach: f64) -> bool {
    (0..3).all(|axis| node[axis][0] - reach <= point[axis] && point[axis] <= node[axis][1] + reach)
}

fn a_box(model: &mut Model, [x, y, z]: [[f64; 2]; 3]) -> SolidKey {
    let rectangle =
        ProfileBuilder::new(vec![[x[0], y[0]], [x[1], y[0]], [x[1], y[1]], [x[0], y[1]]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            z[1] - z[0],
            &Isometry::translation(Vector::new(0.0, 0.0, z[0])).expect("a finite shift"),
        )
        .expect("a positive height")
}
