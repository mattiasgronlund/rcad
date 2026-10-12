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
//! (`docs/decisions.md` §16.10): its answer exists, and another way of asking
//! for the same union reaches it.
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

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
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
}

fn unite(the_box: TheBox, wall: Wall, call: Call) -> (Model, Result<Vec<SolidKey>, Error>) {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let [lower, upper] = the_two_bars(&mut model, wall);
    let third = a_box(&mut model, the_box.bounds());
    let union = |model: &mut Model, arguments: &[SolidKey], tools: &[SolidKey]| {
        model
            .fuse(
                arguments,
                tools,
                Rule::Union,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .map(|fused| fused.solids().to_vec())
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
    };
    (model, said)
}

fn is_the_union_by_hand(model: &Model, united: &[SolidKey], the_box: TheBox, asked: &str) {
    let policy = Policy::certified();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|broken| panic!("{asked}: {broken:?}"));
    let (solids, by_hand) = the_box.union_by_hand();
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

#[test]
fn probe_q1406() {
    let policy = Policy::certified();
    let turn = core::f64::consts::FRAC_PI_4;
    let along = |y: f64| y * (4.0 - y * y).sqrt() / 2.0 + 2.0 * (y / 2.0).asin();
    let moment = |y: f64| -(4.0 - y * y).powf(1.5) / 3.0;
    let (k, r) = (0.1 * 2.0_f64.sqrt(), 1.75_f64.sqrt());
    let inside = moment(r) - moment(k) - k * (along(r) - along(k)) - 0.75 * (r - k) * (r - k);
    let the_box_outside = 2.5 * 1.9 * 3.6 - inside;
    eprintln!(
        "by hand: union {} ; lower∪box {} ; upper {}",
        16.0 * PI + the_box_outside,
        8.0 * PI + the_box_outside,
        8.0 * PI
    );
    let rules = [
        ("union", Rule::Union),
        ("intersection", Rule::Intersection),
        ("difference", Rule::Difference),
    ];
    for wall in EVERY_WALL {
        for (box_arg, upper_arg) in EVERY_ORDER_OF_THE_TWO_FUSES {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let [lower, upper] = the_two_bars(&mut model, wall);
            let third = a_turned_box(&mut model, turn);
            let first = if box_arg {
                model.fuse(
                    &[third],
                    &[lower],
                    Rule::Union,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
            } else {
                model.fuse(
                    &[lower],
                    &[third],
                    Rule::Union,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
            };
            let first = match first {
                Ok(f) => f.solids().to_vec(),
                Err(e) => {
                    eprintln!("{wall:?} box_arg={box_arg}: first refused {e:?}");
                    continue;
                }
            };
            eprintln!(
                "{wall:?} box_arg={box_arg}: first {}",
                volumes(&model, &first)
            );
            if !box_arg && !upper_arg {
                for face in model.solid(first[0]).expect("key").faces() {
                    let kind = match face.surface() {
                        Surface::Plane(_) => "plane",
                        Surface::Cylinder(_) => "pipe",
                        _ => "other",
                    };
                    for wire in face.wires() {
                        let points: Vec<String> = wire
                            .points()
                            .map(|p| format!("({:.4}, {:.4}, {:.4})", p.x(), p.y(), p.z()))
                            .collect();
                        eprintln!("    {kind} {:?} wire: {}", face.key(), points.join(" "));
                    }
                }
            }
            for (name, rule) in rules {
                for upper_first in [false, true] {
                    if upper_first != upper_arg && rule == Rule::Union {
                        continue;
                    }
                    let said = if upper_first {
                        model.fuse(&[upper], &first, rule, &policy, &mut ProvenanceLog::new())
                    } else {
                        model.fuse(&first, &[upper], rule, &policy, &mut ProvenanceLog::new())
                    };
                    match said {
                        Ok(f) => {
                            let solids = f.solids().to_vec();
                            let valid = model.validate(&policy, &mut ProvenanceLog::new());
                            eprintln!(
                                "  {wall:?} box_arg={box_arg} {name} upper_first={upper_first}: {} valid={:?}",
                                volumes(&model, &solids),
                                valid.is_ok()
                            );
                        }
                        Err(e) => eprintln!(
                            "  {wall:?} box_arg={box_arg} {name} upper_first={upper_first}: REFUSED {e:?}"
                        ),
                    }
                }
            }
        }
        for one_call in [false, true] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let [lower, upper] = the_two_bars(&mut model, wall);
            let third = a_turned_box(&mut model, turn);
            let said = if one_call {
                model.fuse(
                    &[lower],
                    &[upper, third],
                    Rule::Union,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
            } else {
                model.fuse(
                    &[upper, third],
                    &[lower],
                    Rule::Union,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
            };
            match said {
                Ok(f) => eprintln!(
                    "  {wall:?} one call lower_first={one_call}: {}",
                    volumes(&model, &f.solids().to_vec())
                ),
                Err(e) => eprintln!("  {wall:?} one call lower_first={one_call}: REFUSED {e:?}"),
            }
        }
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let [lower, upper] = the_two_bars(&mut model, wall);
        let third = a_turned_box(&mut model, turn);
        let said = model
            .fuse(
                &[lower],
                &[upper],
                Rule::Union,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .and_then(|bars| {
                model.fuse(
                    bars.solids(),
                    &[third],
                    Rule::Union,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
            });
        match said {
            Ok(f) => eprintln!(
                "  {wall:?} bars first: {}",
                volumes(&model, &f.solids().to_vec())
            ),
            Err(e) => eprintln!("  {wall:?} bars first: REFUSED {e:?}"),
        }
    }
    panic!("probe");
}

fn volumes(model: &Model, solids: &[SolidKey]) -> String {
    let policy = Policy::certified();
    solids
        .iter()
        .map(|&solid| {
            match model.solid(solid).expect("key").measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            ) {
                Ok(m) => format!("{} ± {}", m.volume(), m.volume_error()),
                Err(e) => format!("measure refused {e:?}"),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
        + &format!(" ({} solids)", solids.len())
}

fn a_turned_box(model: &mut Model, turn: f64) -> SolidKey {
    if turn > 0.5 {
        let rectangle =
            ProfileBuilder::new(vec![[1.5, -2.0], [4.0, -2.0], [4.0, -0.1], [1.5, -0.1]])
                .build(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("a rectangle");
        let about_x = Isometry::rotation(&Axis::x(), turn).expect("a finite turn");
        let down = Isometry::translation(Vector::new(0.0, 0.0, -0.4)).expect("a finite shift");
        return model
            .add_extrusion(&rectangle, 3.6, &about_x.compose(&down))
            .expect("a positive height");
    }
    let rectangle = ProfileBuilder::new(vec![[1.5, -0.5], [4.0, -0.5], [4.0, 0.5], [1.5, 0.5]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let centre = Isometry::translation(Vector::new(0.0, 0.0, 1.0)).expect("a finite shift");
    let about_x = Isometry::rotation(&Axis::x(), turn).expect("a finite turn");
    let half_down = Isometry::translation(Vector::new(0.0, 0.0, -0.5)).expect("a finite shift");
    model
        .add_extrusion(
            &rectangle,
            1.0,
            &centre.compose(&about_x).compose(&half_down),
        )
        .expect("a positive height")
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
