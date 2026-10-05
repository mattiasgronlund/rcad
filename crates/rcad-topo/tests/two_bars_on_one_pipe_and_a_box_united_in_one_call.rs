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
//! - **The lower bar and a box through its wall, then the upper bar**
//!   ([`uniting_the_lower_bar_with_the_box_first_refuses_the_upper_bar_for_two_boxes`]).
//!   The first union answers and validates. The second is refused
//!   `AFaceTrimmedByACurveThisCannotFollow` on that union's wall, where
//!   `select.rs` asks the wall, through `SolidView::face_holds_the_point` in
//!   `holds.rs`, whether it holds a point of the upper bar's wall on the same
//!   pipe. A window cut opposite the seam is refused the same, so the seam is
//!   not what stops it
//!   ([`a_window_cut_opposite_the_seam_is_refused_the_same`]).

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

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

/// **The lower bar united with the box first, and the upper bar after, is
/// refused for two of the seven boxes**, by every spelling of the wall, where
/// the other two ways of asking answer: the box across the meeting and the box
/// on the lower wall, at the second, `AFaceTrimmedByACurveThisCannotFollow`
/// naming the first union's wall. Each refusal is a gap: the same three solids
/// unite in one call, and with the bars first.
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
fn uniting_the_lower_bar_with_the_box_first_refuses_the_upper_bar_for_two_boxes() {
    for wall in EVERY_WALL {
        for the_box in [
            TheBox::BlockOverNegativeX,
            TheBox::OnTheUpperWallOnly,
            TheBox::FarAway,
            TheBox::TopSlicedOffAbove,
            TheBox::SlicedAtTheMeeting,
        ] {
            let asked = format!("{the_box:?}, {wall:?}");
            let (model, said) = unite(the_box, wall, Call::LowerWithTheBoxThenUpper);
            let united = said.unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
            is_the_union_by_hand(&model, &united, the_box, &asked);
        }

        for the_box in [TheBox::AcrossTheMeeting, TheBox::OnTheLowerWallOnly] {
            refuses_on_the_first_unions_wall(the_box, wall);
        }
    }
}

/// **A window cut through the lower bar's wall opposite its seam is refused
/// the same** as the box on the lower wall, which cuts one through the seam:
/// so the seam running through the window is not what `holds.rs` cannot
/// follow. The same three united in one call, or with the bars first, are the
/// mirror image of the box on the lower wall's union, and hold as much.
#[test]
fn a_window_cut_opposite_the_seam_is_refused_the_same() {
    let the_box = TheBox::OnTheLowerWallOppositeTheSeam;
    for wall in EVERY_WALL {
        refuses_on_the_first_unions_wall(the_box, wall);
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

/// `(lower ∪ box) ∪ upper` refused `AFaceTrimmedByACurveThisCannotFollow`,
/// naming the first union's wall: a face of it on a pipe.
fn refuses_on_the_first_unions_wall(the_box: TheBox, wall: Wall) {
    let asked = format!("{the_box:?}, {wall:?}");
    let asked = asked.as_str();
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let [lower, upper] = the_two_bars(&mut model, wall);
    let third = a_box(&mut model, the_box.bounds());
    let first = model
        .fuse(
            &[lower],
            &[third],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{asked}, the lower bar and the box: {refused:?}"))
        .solids()
        .to_vec();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|broken| panic!("{asked}, the first union: {broken:?}"));
    let [solid] = first.as_slice() else {
        panic!("{asked}: the lower bar and the box are one solid: {first:?}");
    };

    let refused = model
        .fuse(
            &first,
            &[upper],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect_err(asked);
    let Error::AFaceTrimmedByACurveThisCannotFollow { face } = refused else {
        panic!("{asked}: {refused:?}");
    };
    let named = model
        .solid(*solid)
        .expect("this model made the key")
        .faces()
        .find(|used| used.key() == face)
        .unwrap_or_else(|| panic!("{asked}: {face:?} is not a face of the first union"));
    assert!(
        matches!(named.surface(), Surface::Cylinder(_)),
        "{asked}: {:?}",
        named.surface()
    );
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
    /// `fuse([lower], [box])`, then that with the upper bar.
    LowerWithTheBoxThenUpper,
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
        Call::LowerWithTheBoxThenUpper => union(&mut model, &[lower], &[third])
            .and_then(|first| union(&mut model, &first, &[upper])),
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
