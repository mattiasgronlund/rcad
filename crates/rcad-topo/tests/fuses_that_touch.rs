//! Fusing two solids that touch without crossing, under every rule, beside the
//! same two solids moved one unit apart. `docs/decisions.md` §191.
//!
//! `pairs/mod.rs` keeps every curved solid a unit clear of every wall so that
//! nothing it draws is tangent to anything, and hands tangency to
//! `degenerate_curved_fuses.rs` one hand-written case at a time. But every
//! fillet in a real part is a round face tangent to two flat ones, so a touch is
//! the ordinary case in a file and not the edge case. `touching/mod.rs` draws a
//! catalogue of them, each cell twice: touching, and moved one unit clear.
//!
//! # What is asked of every drawn pair
//!
//! Under `Policy::certified()`, all three rules, both ways round:
//!
//! - **no panic**, and an answer passes `Model::validate` and logs nothing;
//! - **inclusion and exclusion** — `A ∪ B + A ∩ B = A + B`, `A \ B + A ∩ B = A`,
//!   `B \ A + A ∩ B = B` — against the share of the declared mesh error that
//!   `fuses_that_must_agree.rs` measured, never a tolerance of this file's own;
//! - where the insides are apart, `A ∩ B` is **no solid at all**, and where one
//!   is inside the other, `B \ A` is none — exact, not measured;
//! - **union and intersection both ways round** agree on how many solids and on
//!   the volume to within what the two meshes declare.
//!
//! A refusal is a result: it is counted by name, per row, and the table below is
//! the count. A cell whose control never answers has measured nothing, and
//! fails.
//!
//! # The table
//!
//! `mise exec -- just t -p rcad-topo --test fuses_that_touch --no-capture`, from
//! the fixed seed: 24 drawn shapes per cell, split between the two rows, six
//! fuses each. "answered" is out of every fuse that row made.
//!
//! | cell | touching | clear by one | what the touch alone refuses |
//! |---|---|---|---|
//! | two blocks, one flush on the other | 66 of 66 | 84 of 84 | — |
//! | a bar beside a block's wall | 78 of 78 | 78 of 78 | — |
//! | a bar in an inside corner (a fillet) | 84 of 84 | 72 of 72 | — |
//! | a bar's cap flush on a block | 0 of 66 | 84 of 84 | *a point inside a piece of a face* 66 |
//! | a ball resting on a block | 54 of 72 | 78 of 78 | `NotFinite` 9, `ResultIsNotManifold` 9 |
//! | a ball against a block's wall | 57 of 78 | 72 of 72 | `ResultIsNotManifold` 21 |
//! | two bars touching along a line | 60 of 60 | 96 of 96 | — |
//! | a bar inside a bar, touching | panics on its control | — | see below |
//! | two balls touching at a point | 54 of 54 | 96 of 96 | — |
//! | a ball inside a ball, touching once | 54 of 54 | 96 of 96 | — |
//! | the dome on a shaft of its radius | 0 of 66 | 12 of 84 | `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` 66 |
//! | a ball against a bar's wall | 54 of 54 | 96 of 96 | — |
//! | a ball inside a bar, touching | 54 of 54 | 96 of 96 | — |
//! | a cone's apex on a block | 33 of 90 | 66 of 66 | `CannotNameWhichPieceOfACrossingThisIs` 21, *a point inside a piece of a face* 36 |
//! | a frustum's cap flush on a block | 0 of 84 | 66 of 66 | *a point inside a piece of a face* 84 |
//! | a bend lying on a block | 0 of 66 | 0 of 90 | nothing: both rows refuse as a quartic |
//! | a bend's cap flush against a block | 0 of 66 | 90 of 90 | `ACrossingBetweenAPlaneAndATorusThatIsAQuartic` 66 |
//!
//! Every answer that came back passed every law above. The two failures are a
//! **panic** on a bar standing inside a bar and touching nothing —
//! [`a_bar_standing_wholly_inside_a_thicker_bar_is_united_and_hollowed_without_a_panic`]
//! — which the property meets on its control row before its touching row, so
//! that cell's table is not printed until the panic is fixed. By hand, before
//! queue row Q649, the touching row (radius three, radius one at `(0, 2)`)
//! refused every fuse on *which way a crossing runs*; it has not been asked
//! again since.
//!
//! # Which refusals are gaps and which are limits
//!
//! §16.10: a limit where the answer does not exist, a gap where it does. **For
//! every cell whose insides are apart the answer exists and this kernel can
//! hold it**: the union is the two inputs as they stand, the intersection is
//! nothing, and the difference is the first input — three things the model
//! already holds, since both inputs were built and validated. The rows that do
//! answer say the fuse can reach it: two balls touching at a point, a ball
//! against a bar, a ball of radius one on a block, and every control. So every
//! refusal in the "touch alone" column of an apart cell is a **gap**:
//!
//! - A wall or a second pipe lying along a pipe's ruling: **closed**, every
//!   fuse of those three cells answered, 2026-10-03. Every union used to refuse
//!   `ResultIsNotManifold`, because both solids keep both sides of the touching
//!   line and it came out one edge with four faces round it, two pieces of the
//!   wall and the skin walking it both ways (queue row Q760 printed them).
//!   `fuse/assemble.rs` now parts such an edge into one per input face, and the
//!   union is the two solids as they stand, two lumps;
//!   `a_bar_lying_along_a_plane_or_a_bar.rs` pins it against the volumes. Until
//!   queue row Q649 every fuse of these three cells refused before that, on
//!   *which way a crossing runs*: the two surfaces' normals are parallel along
//!   the line, so their cross product is exactly nothing. The line runs along
//!   the pipe's stored axis, and `fuse/arrange.rs`'s `AlongATouch` reads it now.
//!   Handed over first, a bar touching on its seam used to stop earlier still,
//!   on *side of plane* or *side of cylinder*, asked about a box round its seam
//!   corner that a whole-number move had put there without rounding anything;
//!   `a_bar_moved_by_whole_numbers_keeps_its_seam_corner_exact.rs` is that gap,
//!   closed.
//! - *a point inside a piece of a face*, a flat cap flush on a flat face with a
//!   circle round it: known, §39.4's coincident-face row and §38.9.
//! - `ResultIsNotManifold` and `NotFinite` on a ball resting on a block:
//!   **new**, unclassified before this file, and demonstrated a gap by
//!   [`a_ball_resting_on_a_block_it_overhangs_is_refused_though_the_answer_is_both_solids`].
//!   A `NotFinite` out of whole-number input is also a coordinate the kernel
//!   worked out as infinity, which is worth a look on its own.
//! - `CannotNameWhichPieceOfACrossingThisIs` on a cone's apex: a plane through a
//!   cone's apex square to its axis crosses it in the apex alone. Unclassified
//!   here beyond the argument above, that the answer is the two inputs.
//!
//! **The dome is known: §124.** A ball on a bar of its own radius, centred on
//! the bar's axis, shares exactly one circle with it **only** because it is
//! tangent (§128.3: `(z − h)² = R² − r²` is two circles for `R > r` and none for
//! `R < r`), so it has no transversal control at equal radius; the control here
//! widens the bar by one, where the ball sits wholly inside the pipe and crosses
//! nothing but the cap. **Of every cell here it is the only one with that
//! property**: every other touch has a crossing twin one unit the other way.
//! Its control still refuses 72 of 84, on *a point inside a piece of a face*;
//! what in it refuses is not worked out here.
//!
//! **A bend's touch is not reached.** Every fuse of a bend lying on a block is
//! refused as a quartic, touching or one unit clear, so that cell is pinned at
//! *nothing answers* and fails the day that changes. A bend's cap against a
//! block refuses its touching row the same way. Its control, whose block is
//! clear of the bend, answers all 90. The 45 that keep the bend were refused
//! writing it back, `TheFuseWroteABrokenAnswer` with `CoedgesRunTheSameWay`,
//! the stop §243.5 and §323.5 record, until the bend's wall was used forwards
//! (§361). A torus against any other curved surface is refused by name.
//!
//! # What is not built, and why
//!
//! - **A bar lying on its side** on a block or another bar needs a quarter turn,
//!   and a quarter turn of a round solid cannot be asked for exactly
//!   (`fuses_that_must_agree.rs`'s
//!   `a_quarter_turn_of_a_round_solid_cannot_be_asked_for_exactly`). An upright
//!   bar beside a wall is the same touch turned, and is built instead.
//! - **A plane lying along a cone's ruling** needs a slanted face, which no
//!   whole-number block has; `degenerate_curved_fuses.rs` builds it by hand as
//!   `a_wedge_lying_on_a_cones_side_shares_and_leaves_what_is_worked_out_by_hand`.
//! - **A cone against a ball or a bar**: not built yet, and nothing here says
//!   whether they would reach their touch. A cone's side against another cone
//!   *is* built now, as one hand-written pair rather than a cell —
//!   [`two_cones_sharing_one_rim_circle_are_refused_before_the_join_could_see_them`]
//!   — and every rule of it is refused by name before the fuse assembles
//!   anything. A torus against anything curved is refused by name before it could
//!   (`how_often_a_fuse_with_a_torus_refuses.rs`'s last three rows).

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use proptest::test_runner::{
    Config, FileFailurePersistence, RngAlgorithm, TestCaseError, TestRng, TestRunner,
};
use rcad_geom::{Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, Rule, SolidKey};

mod touching;

use touching::{Apart, Drawn, Insides, Shape, Slope, both_rows};

/// How many shapes each cell draws, touching and clear together.
///
/// [`SOAK_FACTOR`] multiplies it and [`SOAK_SEED`] replaces the fixed seed, the
/// way `fuses_that_must_agree.rs` does. With neither set — which is how the gate
/// runs it — this is 24 shapes from the one fixed seed, and the table at the top
/// of this file is what that prints.
const CASES: u32 = 24;

/// The environment variable that multiplies [`CASES`] for a soak.
///
/// ```text
/// RCAD_SOAK_FACTOR=100 cargo nextest run -p rcad-topo --test fuses_that_touch
/// ```
const SOAK_FACTOR: &str = "RCAD_SOAK_FACTOR";

/// The environment variable that reseeds every cell.
///
/// These tests are deterministic on purpose, so running the same soak twice
/// draws the same shapes twice. A *spread* needs this varied as well.
const SOAK_SEED: &str = "RCAD_SOAK_SEED";

#[test]
fn two_blocks_one_on_the_other() {
    every_rule_over(
        "two blocks, one flush on the other",
        touching::a_block_on_a_block(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_bar_standing_beside_a_blocks_wall() {
    every_rule_over(
        "a bar beside a block's wall",
        touching::a_bar_beside_a_wall(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_bar_in_an_inside_corner_touching_both_walls() {
    every_rule_over(
        "a bar in an inside corner, the fillet",
        touching::a_bar_in_an_inside_corner(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_bar_standing_on_a_block() {
    every_rule_over(
        "a bar's cap flush on a block",
        touching::a_bar_standing_on_a_block(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_ball_resting_on_a_block() {
    every_rule_over(
        "a ball resting on a block",
        touching::a_ball_on_a_block(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_ball_against_a_blocks_wall() {
    every_rule_over(
        "a ball against a block's wall",
        touching::a_ball_against_a_wall(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn two_bars_touching_along_a_line() {
    every_rule_over(
        "two bars touching along a line",
        touching::two_bars_touching(),
        Expect::TheControlAnswers,
    );
}

#[test]
#[ignore = "open: §191, its control row is a bar wholly inside a bar, which panics in fuse/assemble.rs"]
fn a_bar_inside_a_bar_touching_its_wall() {
    every_rule_over(
        "a bar inside a bar, touching",
        touching::a_bar_inside_a_bar(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn two_balls_touching_at_a_point() {
    every_rule_over(
        "two balls touching at a point",
        touching::two_balls_touching(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_ball_inside_a_larger_ball_touching_once() {
    every_rule_over(
        "a ball inside a ball, touching once",
        touching::a_ball_inside_a_ball(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_ball_on_a_bar_of_the_same_radius() {
    every_rule_over(
        "the dome on a shaft of its radius",
        touching::a_dome_on_a_shaft(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_ball_against_a_bars_wall() {
    every_rule_over(
        "a ball against a bar's wall",
        touching::a_ball_against_a_bar(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_ball_inside_a_bar_touching_its_wall() {
    every_rule_over(
        "a ball inside a bar, touching",
        touching::a_ball_inside_a_bar(),
        Expect::TheControlAnswers,
    );
}

#[test]
fn a_pointed_cone_standing_on_its_apex_on_a_block() {
    every_rule_over(
        "a cone's apex on a block",
        touching::a_cones_apex_on_a_block(),
        Expect::TheControlAnswers,
    );
}

/// A pointed cone of slope a half and height two, its apex on the top of a
/// 4-block whose walls `x = −1` and `y = 1` are tangent to the cone's rim
/// circle at `(−1, 0, 2)` and `(0, 1, 2)`, far above the block.
///
/// The wall planes cross the cone's surface in curves that touch the rim at
/// those two points, so the rim is cut there as well as at its own one corner
/// `(1, 0, 2)`. Joining the cap's pieces back into one face then walked
/// straight through all three corners, and the rim came back as one edge with
/// no ends: a panic in `fuse/assemble.rs`, *"a run of edges that became one
/// edge has two ends left"*, found by the property above from seed
/// `9bb0dbcd…3910` once the cone's box reached its apex (§202.3).
///
/// The inputs meet only at the apex, so every answer is known without the
/// fuse: the union is both solids as they stand, the intersection is nothing,
/// and each difference is its first solid. Checked by solid count, by how many
/// faces each solid keeps, and by where points fall against the two closed
/// forms.
#[test]
fn a_pointed_cone_on_its_apex_whose_rim_touches_two_walls_fuses_by_every_rule() {
    let built = Drawn {
        shape: Shape::AConesApexOnABlock {
            slope: Slope::Half,
            at: [1, 3],
        },
        apart: Apart::Touching,
    }
    .build();
    let (block, cone) = (built.first, built.second);
    let in_block = |p: Point| {
        let [x, y, z] = p.coordinates();
        (-1.0..=3.0).contains(&x) && (-3.0..=1.0).contains(&y) && (-4.0..=0.0).contains(&z)
    };
    let in_cone = |p: Point| {
        let [x, y, z] = p.coordinates();
        (0.0..=2.0).contains(&z) && x.hypot(y) <= 0.5 * z
    };
    let samples = [
        Point::new(0.1, 0.07, 1.03),
        Point::new(0.3, -0.2, 1.9),
        Point::new(0.5, -1.5, -2.0),
        Point::new(2.9, 0.9, -0.1),
        Point::new(0.9, 0.9, 1.9),
        Point::new(-0.9, 0.3, 1.5),
        Point::new(0.2, 0.1, 0.1),
        Point::new(3.5, 0.0, -1.0),
    ];
    let union: fn(bool, bool) -> bool = |a, b| a || b;
    let intersection: fn(bool, bool) -> bool = |a, b| a && b;
    let difference: fn(bool, bool) -> bool = |a, b| a && !b;
    for (one, other, rule, solids, faces, holds) in [
        (block, cone, Rule::Union, 2, vec![2, 6], union),
        (cone, block, Rule::Union, 2, vec![2, 6], union),
        (block, cone, Rule::Intersection, 0, vec![], intersection),
        (cone, block, Rule::Intersection, 0, vec![], intersection),
        (block, cone, Rule::Difference, 1, vec![6], difference),
        (cone, block, Rule::Difference, 1, vec![2], difference),
    ] {
        let mut model = built.model.clone();
        let policy = Policy::certified();
        let answer = model
            .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?} refused: {why}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?} does not validate: {why}"));
        let pieces = answer.solids().to_vec();
        assert_eq!(pieces.len(), solids, "{rule:?}: how many solids");
        let mut got: Vec<usize> = pieces
            .iter()
            .map(|&piece| model.solid(piece).expect("an answer's key").faces().count())
            .collect();
        got.sort_unstable();
        assert_eq!(got, faces, "{rule:?}: how many faces each solid has");

        let first_is_block = one == block;
        for p in samples {
            let (a, b) = if first_is_block {
                (in_block(p), in_cone(p))
            } else {
                (in_cone(p), in_block(p))
            };
            let want = holds(a, b);
            let inside = pieces.iter().any(|&piece| {
                match model.solid(piece).expect("an answer's key").classify_point(
                    p,
                    &policy,
                    &mut ProvenanceLog::new(),
                ) {
                    Containment::Inside => true,
                    Containment::Outside => false,
                    said @ (Containment::OnBoundary
                    | Containment::Undecidable { .. }
                    | Containment::CannotFollowAFacesTrimming { .. }
                    | Containment::CannotFireARayAtAFacesSurface { .. }) => {
                        panic!("{rule:?}: {p:?} is clear of every face and came back {said:?}")
                    }
                    other => panic!("{rule:?}: {p:?} came back {other:?}"),
                }
            });
            assert_eq!(
                inside, want,
                "{rule:?} (block first: {first_is_block}) at {p:?}"
            );
        }
    }
}

/// Two pointed cones base to base on one circle, their rim vertices half a turn
/// apart — the shape that would hand §204.1's join a run of pieces carrying two
/// input edges — refused before the fuse assembles anything.
///
/// §204.1 joins a closed curve that was cut into pieces back into one edge only
/// **when every piece came from one input edge**. A run whose pieces are an
/// edge of *both* solids is the nearest way past that: cut the shared closed
/// edge at each solid's own vertex and every piece carries two input edges, so
/// the join keeps no corner and `assemble.rs` panics with *"a run of edges that
/// became one edge has two ends left"* the way it did before §204.1.
///
/// **Both cut points have to have nothing else leaving them**, and a rim vertex
/// has something else leaving it wherever the face inside the rim carries a
/// seam ending there — a bar, a frustum, a ball and a bend all do, so one of
/// the two cut points stays a corner and the run keeps it. A pointed cone's
/// side is the one face here with a closed wire and no seam, so both solids
/// have to be pointed cones on one circle, which is two cone faces meeting one
/// curve: refused by name. The same family with a sphere in place of the second
/// cone is [`a_ball_on_a_bar_of_the_same_radius`], whose touching row answers
/// none of its fuses either.
///
/// So *this* family is a **limit and not a gap** (§16.10) and the pin is here:
/// the day either refusal goes, the closed-run join has a second case to
/// answer. What is **not** settled is a closing run of pieces of two *different*
/// input edges, or of a fresh crossing curve carrying no input edge at all;
/// neither has been built and neither has been ruled out.
#[test]
fn two_cones_sharing_one_rim_circle_are_refused_before_the_join_could_see_them() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let lower = a_pointed_cone_with_its_vertex_at(
        &mut model,
        Point::new(0.0, 0.0, 0.0),
        2.0,
        Point::new(2.0, 0.0, 2.0),
    );
    let upper = a_pointed_cone_with_its_vertex_at(
        &mut model,
        Point::new(0.0, 0.0, 4.0),
        -2.0,
        Point::new(-2.0, 0.0, 2.0),
    );
    let policy = Policy::certified();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for (one, other) in [(lower, upper), (upper, lower)] {
            let why = model
                .clone()
                .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
                .expect_err("a cone against a cone is refused");
            assert!(
                matches!(
                    why,
                    Error::Geometry(rcad_geom::Error::ACrossingBetweenAConeAndAnotherCurvedSurface)
                ),
                "{rule:?}: refused {why} instead"
            );
        }
    }
}

/// A pointed cone of slope one, its apex at `apex`, its base cap `far` up the
/// `z` axis from there — `far` may point down — and its one vertex wherever
/// `rim_at` says on the rim circle.
fn a_pointed_cone_with_its_vertex_at(
    model: &mut Model,
    apex: Point,
    far: f64,
    rim_at: Point,
) -> SolidKey {
    use rcad_geom::{Axis, Cone, EdgeRun, Orientation, Plane, Surface};
    use rcad_topo::{DraftStep, SolidBuilder};

    let up = Vector::new(0.0, 0.0, 1.0);
    let skin = Cone::new(Axis::new(apex, up).expect("z is a direction"), 1.0).expect("a slope");
    let mut builder = SolidBuilder::new();

    let rim = builder.add_vertex(rim_at).expect("a finite point");
    let circle = builder
        .add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the cap plane cuts the cone in one closed curve");

    let [outward, inward] = if far > 0.0 {
        [Orientation::Forward, Orientation::Reversed]
    } else {
        [Orientation::Reversed, Orientation::Forward]
    };
    let at = apex + up * far;
    let plane = Plane::from_point_and_normal(at, up * far.signum()).expect("a unit normal");
    let cap_coedge = builder.add_coedge(circle, outward).expect("an edge built");
    let cap_wire = builder
        .add_wire(&[DraftStep::Along(cap_coedge)])
        .expect("one closed edge is a wire");
    let cap = builder
        .add_face(Surface::Plane(plane), &[cap_wire])
        .expect("a cap is a plane trimmed by its circle");

    let side_coedge = builder.add_coedge(circle, inward).expect("an edge built");
    let side_wire = builder
        .add_wire(&[DraftStep::Along(side_coedge)])
        .expect("one closed edge is a wire");
    let side = builder
        .add_face(Surface::Cone(skin), &[side_wire])
        .expect("a face on the cone, bounded by its base circle and pinched at the apex");

    builder
        .add_shell(&[(cap, Orientation::Forward), (side, Orientation::Forward)])
        .expect("two faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a pointed cone described this way is a solid")
}

#[test]
fn a_frustums_cap_flush_on_a_block() {
    every_rule_over(
        "a frustum's cap flush on a block",
        touching::a_frustums_cap_on_a_block(),
        Expect::TheControlAnswers,
    );
}

/// Pinned at nothing answered, touch and control alike: every fuse in both
/// rows is refused `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`, so the
/// touch is never reached. This fails the day that changes, which is the day
/// this cell starts measuring its tangency.
#[test]
fn a_bend_of_pipe_lying_on_a_block() {
    every_rule_over(
        "a bend lying on a block",
        touching::a_bend_lying_on_a_block(),
        Expect::NothingAnswers,
    );
}

/// A bend's cap flush against a block, and the block slid one unit clear along
/// the cap's plane.
///
/// The touching row refuses every fuse as a quartic: the block touches the
/// bend, so the two are cut against each other, and three of the block's walls
/// run along the bend's axis and off it. The control's block is clear of the
/// bend, so nothing is cut, and since Q326 no fuse asks whether a block's wall
/// is the bend's ring (`do_they_face_the_same_way` in `fuse/select.rs`). Until
/// then that question was the quartic, and the control answered nothing.
///
/// Now the control answers all six fuses: both intersections are nothing, the
/// block less the bend is the whole block, its volume exact, and the union both
/// ways round and the bend less the block keep the bend. Those three were
/// refused writing it back, `TheFuseWroteABrokenAnswer` with
/// `CoedgesRunTheSameWay`, where a bend fused with a copy of itself stopped
/// (§243.5, §323.5), until the bend's wall was used forwards (§361).
#[test]
fn a_bends_cap_flush_against_a_block() {
    every_rule_over(
        "a bend's cap flush against a block",
        touching::a_bends_cap_against_a_block(),
        Expect::TheControlAnswers,
    );
}

/// A thin bar standing wholly inside a thick one, touching nothing, is united
/// and hollowed out without a panic.
///
/// Found by the control row of
/// [`a_bar_inside_a_bar_touching_its_wall`] and shrunk to this: a bar of radius
/// one and height two standing on the axis of a bar of radius two and height
/// four, one unit up, so every face of the thin bar is clear of every face of
/// the thick one. Off the axis it panics the same way — radius three with the
/// thin bar at `(0, 1)`, `(1, 0)` or `(−1, 0)`. `A ∩ B` and `B \ A` answer;
/// `A ∪ B` and `A \ B` panic in `fuse/assemble.rs` with *"a ring of a joined
/// face came back winding neither way: Zero"*, both ways round for the union.
///
/// The answer is plain: the union is the thick bar and the difference is the
/// thick bar with a bar-shaped hole sealed inside it. A ball standing inside
/// the same bar is answered under every rule, so it is the two pipes and not
/// the hole.
#[test]
#[ignore = "open: §191, a bar wholly inside a thicker bar panics under union and difference"]
fn a_bar_standing_wholly_inside_a_thicker_bar_is_united_and_hollowed_without_a_panic() {
    for rule in [Rule::Union, Rule::Difference] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let thick = model
            .add_round_bar(2.0, 4.0, &Isometry::identity())
            .expect("a bar of radius two");
        let thin = model
            .add_round_bar(
                1.0,
                2.0,
                &Isometry::translation(Vector::new(0.0, 0.0, 1.0)).expect("a whole-number shift"),
            )
            .expect("a bar of radius one");
        let answer = model
            .fuse(
                &[thick],
                &[thin],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{rule:?} of a bar inside a bar refused: {why}"));
        assert_eq!(answer.solids().len(), 1, "{rule:?} is one solid");
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?} left a model that does not validate: {why}"));
    }
}

/// A ball of radius two resting on a block it hangs over the edge of is
/// refused, though its answer is the two solids as they stand — a **gap**
/// (§16.10), and this is the demonstration.
///
/// A cube of side two and a ball of radius two whose south pole rests on the
/// middle of the cube's top, at `(1, 1, 2)`. Every number is whole and the touch
/// is exact. The union is refused `ACurvedFaceGoesRightRoundWithNoSeam`. It has
/// been refused all along and the name has moved twice: `ResultIsNotManifold`
/// first, in assembly, where the two arcs of one ball-and-plane circle between
/// the same two corners were given one edge name, until §198 told them apart;
/// then `AWireWillNotLieFlatOnItsFace`, out of the mesher, until §208.4 let a
/// ring of arcs be walked.
///
/// **The ball moved over to `(1, 2)` on a cube of side four was refused too, and
/// answers since §217.** It was `Geometry(NotFinite)` — a coordinate worked out
/// as infinity or not a number out of whole-number input, raised in
/// `Arrangement::the_whole_surface` and pinned then as measured rather than as
/// understood. What moved it is the only thing §217 changed here: the fuse now
/// writes the surface that cut a closed rim. That block's wall plane cuts the
/// ball in a circle whose two faces are the one sphere, and a rim whose two
/// faces name one surface is exactly what the new run exists for.
///
/// **Why that makes this particular coordinate finite was not chased**, so the
/// row is kept and asserts the two solids rather than being deleted: a row that
/// refused once and answers now is the cheapest guard there is against it
/// refusing again, and the mechanism is a question for whoever wants it.
///
/// Three rows say the answer exists and this kernel can hold it:
///
/// - the same ball **one unit higher** unites into two solids;
/// - a ball of **radius one** resting on the same point unites into two solids;
/// - the same ball resting on a cube of **side four**, where it no longer hangs
///   over an edge, unites into two solids.
///
/// So what refuses is not the touch alone and not the overhang alone: a side
/// wall's plane crossing the ball while the top touches it. A fuse crosses
/// planes and not faces, so a wall nowhere near the ball still cuts it.
#[test]
fn a_ball_resting_on_a_block_it_overhangs_is_refused_though_the_answer_is_both_solids() {
    let refused = |side: f64, centre: [f64; 3], radius: f64| {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cube = model
            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a whole-number cube");
        let [x, y, z] = centre;
        let ball = model
            .add_ball(
                radius,
                &Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift"),
            )
            .expect("a whole-number ball");
        match model.fuse(
            &[cube],
            &[ball],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) {
            Ok(united) => {
                assert_eq!(united.solids().len(), 2, "two solids that touch at a point");
                None
            }
            Err(why) => {
                println!("a ball of {radius} at {centre:?} on a cube of {side}: {why:?}");
                Some(named(&why))
            }
        }
    };

    assert_eq!(
        refused(2.0, [1.0, 1.0, 4.0], 2.0).as_deref(),
        Some("ACurvedFaceGoesRightRoundWithNoSeam")
    );
    assert_eq!(
        refused(4.0, [1.0, 2.0, 6.0], 2.0),
        None,
        "refused Geometry(NotFinite) until §217 named the plane that cut the rim"
    );
    assert_eq!(refused(2.0, [1.0, 1.0, 5.0], 2.0), None, "one unit higher");
    assert_eq!(refused(4.0, [1.0, 2.0, 7.0], 2.0), None, "one unit higher");
    assert_eq!(refused(2.0, [1.0, 1.0, 3.0], 1.0), None, "radius one");
    assert_eq!(refused(4.0, [1.0, 2.0, 5.0], 1.0), None, "radius one");
    assert_eq!(refused(4.0, [2.0, 2.0, 6.0], 2.0), None, "over the middle");
}

/// What a cell's rows must have done for the run to have measured anything.
enum Expect {
    /// The control answered and at least one law was checked on it, so the
    /// oracle ran on this cell.
    TheControlAnswers,
    /// No fuse answered in either row: the cell is pinned at a refusal that
    /// comes before its touch.
    NothingAnswers,
}

fn every_rule_over(cell: &str, shapes: impl Strategy<Value = Shape>, expect: Expect) {
    let tally = Tally::default();
    // From one fixed seed, so the table at the top of this file is what a run
    // prints rather than one draw of it.
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases: cases_to_draw(),
            source_file: Some(file!()),
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
                "proptest-regressions",
            ))),
            ..Config::default()
        },
        seeded(),
    );
    if let Err(why) = runner.run(&both_rows(shapes), |drawn| every_rule_of(&drawn, &tally)) {
        panic!("{cell}: {why}");
    }
    tally.report(cell);
    match expect {
        Expect::TheControlAnswers => assert!(
            tally.laws_checked(Apart::ClearByOne) > 0,
            "{cell}: not one law was checked on the control, so nothing was compared"
        ),
        Expect::NothingAnswers => assert_eq!(
            tally.answered.get(),
            0,
            "{cell}: a fuse answered, so this cell reaches its touch now and should be \
             measured rather than pinned"
        ),
    }
}

/// [`CASES`], multiplied by whatever [`SOAK_FACTOR`] says.
///
/// # Panics
///
/// If the variable is not a whole number above zero, or if the product is more
/// than a run can count. A soak that silently ran the shipped corpus while the
/// operator read the number as the wide one is worse than one that stopped
/// (§48.2, §50.9).
fn cases_to_draw() -> u32 {
    let Ok(factor) = std::env::var(SOAK_FACTOR) else {
        return CASES;
    };
    let factor: u32 = factor
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_FACTOR} has to be a whole number, not {factor:?}"));
    assert!(factor > 0, "{SOAK_FACTOR} has to be at least one");
    CASES
        .checked_mul(factor)
        .unwrap_or_else(|| panic!("{CASES} cases times {factor} is more than one run can count"))
}

/// The fixed seed, or the one [`SOAK_SEED`] names.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number.
fn seeded() -> TestRng {
    let Ok(seed) = std::env::var(SOAK_SEED) else {
        return TestRng::deterministic_rng(RngAlgorithm::ChaCha);
    };
    let seed: u64 = seed
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_SEED} has to be a whole number, not {seed:?}"));
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    TestRng::from_seed(RngAlgorithm::ChaCha, &bytes)
}

/// Every rule both ways round, and every law that has both its sides.
fn every_rule_of(drawn: &Drawn, tally: &Tally) -> Result<(), TestCaseError> {
    let built = drawn.build();
    let (a, b) = (built.first, built.second);
    let apart = drawn.apart;
    tally.drawn(apart);

    let mut got = BTreeMap::new();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for (one, other, way) in [(a, b, Way::Forwards), (b, a, Way::Backwards)] {
            let outcome = fused(&built.model, one, other, rule)?;
            tally.outcome(apart, rule, &outcome);
            if let Outcome::Answered(held) = outcome {
                got.insert((name_of(rule), way), held);
            }
        }
    }

    let [first, second] = [a, b].map(|solid| how_much_it_holds(&built.model, &[solid]));
    let answer = |rule, way| got.get(&(name_of(rule), way));

    if let (Some(united), Some(shared)) = (
        answer(Rule::Union, Way::Forwards),
        answer(Rule::Intersection, Way::Forwards),
    ) {
        tally.law(apart);
        adds_up(
            &[united, shared],
            &[&first, &second],
            "A ∪ B + A ∩ B = A + B",
        )?;
    }
    if let (Some(left), Some(shared)) = (
        answer(Rule::Difference, Way::Forwards),
        answer(Rule::Intersection, Way::Forwards),
    ) {
        tally.law(apart);
        adds_up(&[left, shared], &[&first], "A \\ B + A ∩ B = A")?;
    }
    if let (Some(left), Some(shared)) = (
        answer(Rule::Difference, Way::Backwards),
        answer(Rule::Intersection, Way::Forwards),
    ) {
        tally.law(apart);
        adds_up(&[left, shared], &[&second], "B \\ A + A ∩ B = B")?;
    }

    for rule in [Rule::Union, Rule::Intersection] {
        if let (Some(forwards), Some(backwards)) =
            (answer(rule, Way::Forwards), answer(rule, Way::Backwards))
        {
            tally.law(apart);
            prop_assert_eq!(
                forwards.solids,
                backwards.solids,
                "{:?} of {:?} gave {} solids one way round and {} the other",
                rule,
                drawn,
                forwards.solids,
                backwards.solids
            );
            let between = (forwards.volume - backwards.volume).abs();
            prop_assert!(
                between <= forwards.error + backwards.error,
                "{rule:?} of {drawn:?} holds {} one way round and {} the other, further apart \
                 than the {} the two meshes declare between them",
                forwards.volume,
                backwards.volume,
                forwards.error + backwards.error
            );
        }
    }

    let nothing_left = match built.insides {
        Insides::Apart => vec![
            (Rule::Intersection, Way::Forwards),
            (Rule::Intersection, Way::Backwards),
        ],
        Insides::SecondInsideFirst => vec![(Rule::Difference, Way::Backwards)],
        Insides::Overlap => vec![],
    };
    for (rule, way) in nothing_left {
        if let Some(held) = answer(rule, way) {
            tally.law(apart);
            prop_assert_eq!(
                held.solids,
                0,
                "{:?} ({:?}) of {:?} holds no volume and came back with {} solids",
                rule,
                way,
                drawn,
                held.solids
            );
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Way {
    Forwards,
    Backwards,
}

enum Outcome {
    Answered(Held),
    Refused(String),
}

/// How much an answer holds, how much of that its mesh may be hiding, and how
/// many solids it is.
struct Held {
    volume: f64,
    error: f64,
    solids: usize,
}

/// Fuse a copy of `model` under a certified policy.
///
/// An answer must pass `Model::validate` and must have recorded nothing it
/// could not prove. A refusal is a result: it is named and handed back.
fn fused(
    model: &Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
) -> Result<Outcome, TestCaseError> {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    match model.fuse(&[one], &[other], rule, &Policy::certified(), &mut log) {
        Ok(fused) => {
            prop_assert!(
                log.is_empty(),
                "a certified {rule:?} recorded {} decisions it could not prove",
                log.len()
            );
            let solids = fused.solids().to_vec();
            if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
                return Err(TestCaseError::fail(format!(
                    "a {rule:?} answered with a model that does not validate: {why}"
                )));
            }
            let (volume, error) = volume_and_error(&model, &solids);
            Ok(Outcome::Answered(Held {
                volume,
                error,
                solids: solids.len(),
            }))
        }
        Err(why) => Ok(Outcome::Refused(named(&why))),
    }
}

/// An error's variant; for an undecided predicate which ones gave up, and for
/// a broken answer what the sweep refused it for.
fn named(why: &Error) -> String {
    let written = format!("{why:?}");
    let name: String = written.chars().take_while(|c| *c != '{').collect();
    let open = name.matches('(').count() - name.matches(')').count();
    let name = format!("{}{}", name.trim_end(), ")".repeat(open));
    if let Error::TheFuseWroteABrokenAnswer { broken } = why {
        let broken: String = format!("{broken:?}")
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect();
        return format!("{name} ({broken})");
    }
    let Error::Undecidable { records } = why else {
        return name;
    };
    let gave_up: BTreeSet<_> = records.records().iter().map(|r| r.predicate()).collect();
    format!(
        "{name} ({})",
        gave_up.into_iter().collect::<Vec<_>>().join(", ")
    )
}

fn how_much_it_holds(model: &Model, solids: &[SolidKey]) -> Held {
    let (volume, error) = volume_and_error(model, solids);
    Held {
        volume,
        error,
        solids: solids.len(),
    }
}

/// The deflection every volume here is measured at.
///
/// One of the deflections the share below was measured over.
const A_DEFLECTION: f64 = 0.1;

/// How much of what the measurements declare a volume law may be out by.
///
/// **Not a tolerance chosen here.** It is
/// `A_CURVED_LAW_MAY_USE_THIS_MUCH_OF_WHAT_THE_FOUR_DECLARE` in
/// `fuses_that_must_agree.rs`, whose doc says where it comes from: each side of
/// a law is out by at most its own `Measurement::volume_error`, and the worst
/// share of the summed errors that family ever used is 0.025. Copied rather
/// than shared because that file is a test binary and not a module; a change
/// there should be made here too.
const A_LAW_MAY_USE_THIS_MUCH_OF_WHAT_ITS_SIDES_DECLARE: f64 = 0.05;

fn volume_and_error(model: &Model, solids: &[SolidKey]) -> (f64, f64) {
    let deflection = Deflection::of(A_DEFLECTION).expect("a length");
    solids
        .iter()
        .map(|&solid| {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(deflection, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a solid this size measures");
            (measured.volume(), measured.volume_error())
        })
        .fold((0.0, 0.0), |(volume, error), (more, worse)| {
            (volume + more, error + worse)
        })
}

/// What `added` holds together is what `against` holds together.
///
/// When every side is flat the measurements declare nothing and the volumes are
/// whole numbers, so the law holds exactly.
fn adds_up(added: &[&Held], against: &[&Held], law: &str) -> Result<(), TestCaseError> {
    let out_by = added.iter().map(|held| held.volume).sum::<f64>()
        - against.iter().map(|held| held.volume).sum::<f64>();
    let declared: f64 = added.iter().chain(against).map(|held| held.error).sum();
    if declared == 0.0 {
        prop_assert_eq!(
            out_by,
            0.0,
            "{} is out by {} with every side flat",
            law,
            out_by
        );
    } else {
        prop_assert!(
            out_by.abs() < A_LAW_MAY_USE_THIS_MUCH_OF_WHAT_ITS_SIDES_DECLARE * declared,
            "{law} is out by {out_by}, which is {} of the {declared} its sides declare",
            out_by.abs() / declared
        );
    }
    Ok(())
}

fn name_of(rule: Rule) -> &'static str {
    match rule {
        Rule::Union => "union",
        Rule::Intersection => "intersection",
        Rule::Difference => "difference",
        _ => unreachable!("only the three rules above are asked"),
    }
}

/// What each row of a cell came back with.
#[derive(Default)]
struct Tally {
    drawn: RefCell<BTreeMap<Apart, u32>>,
    outcomes: RefCell<BTreeMap<(Apart, String), u32>>,
    laws: RefCell<BTreeMap<Apart, u32>>,
    fuses: Cell<u32>,
    answered: Cell<u32>,
}

impl Tally {
    fn drawn(&self, apart: Apart) {
        *self.drawn.borrow_mut().entry(apart).or_default() += 1;
    }

    fn outcome(&self, apart: Apart, rule: Rule, outcome: &Outcome) {
        self.fuses.set(self.fuses.get() + 1);
        let what = match outcome {
            Outcome::Answered(_) => {
                self.answered.set(self.answered.get() + 1);
                format!("{} answered", name_of(rule))
            }
            Outcome::Refused(why) => format!("{} refused: {why}", name_of(rule)),
        };
        *self.outcomes.borrow_mut().entry((apart, what)).or_default() += 1;
    }

    fn law(&self, apart: Apart) {
        *self.laws.borrow_mut().entry(apart).or_default() += 1;
    }

    fn laws_checked(&self, apart: Apart) -> u32 {
        self.laws.borrow().get(&apart).copied().unwrap_or(0)
    }

    fn report(&self, cell: &str) {
        println!("{cell}: {} fuses", self.fuses.get());
        for apart in [Apart::Touching, Apart::ClearByOne] {
            println!(
                "  {apart:?}: {} shapes, {} laws checked",
                self.drawn.borrow().get(&apart).copied().unwrap_or(0),
                self.laws_checked(apart)
            );
            for ((row, what), count) in self.outcomes.borrow().iter() {
                if *row == apart {
                    println!("    {count:>3}  {what}");
                }
            }
        }
    }
}
