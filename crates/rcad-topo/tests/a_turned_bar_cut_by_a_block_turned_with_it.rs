//! A round bar of radius 3 and height 5, notched by a block of side 2, with
//! bar and block turned together by `Model::place_solid`. Q765's probe found
//! every such fuse refused under `Policy::certified()` at *where a plane and a
//! pipe cross*, turned half a turn about `y` or one radian about `(1, 1, 1)`,
//! and kept nothing but the doc of `a_turned_bar_comes_back_the_same` in
//! `rcad-step`. Q1732 rebuilt the fuses and pins what they answer here
//! (`docs/decisions.md` §246.6).
//!
//! **The shapes.** The bar is `add_round_bar(3, 5)`, with its seam on the
//! ruling `(3, 0, z)`. A notch is a block `2 × 2 × 2` from `z = 1` to `z = 3`:
//! the *seam notch* `[2, 4] × [−1, 1]` across the seam, or the *wall notch*
//! `[−1, 1] × [2, 4]` a quarter turn round the wall from it. Four sides of
//! either run along the bar's axis.
//!
//! **Unturned, and turned an exact half turn** about `y` by
//! `Isometry::quarter_turns`, every fuse answers what it holds by hand under
//! both policies and logs nothing.
//!
//! **Half a turn about `y` by `Isometry::rotation(y, π)`: answered, though the
//! bottom cap circle starts inside a box.** Every side runs *exactly* along the
//! turned bar: how it leans against the pipe comes out zero. But `sin π`
//! rounds to `1.2e-16`, so the turn is not one of a cube's, and the fuse reads
//! the bar's seam corner through `where_a_corner_lands_after_moving` as a box
//! `± CORNER_AFTER_MOVING_ROUNDOFF · 3`. The bottom cap circle starts inside
//! that box. Every certified fuse was refused *order along a crossing* between
//! that box and where a side's line crosses the bottom circle — Q885's case,
//! met there on a shape only a probe builds, and here on one a designer draws.
//! Every tolerant fuse was refused `CannotTellTheCellsOfAFaceApart`. The seam
//! notch tilted off the axis, so that no side runs along it, was refused at the
//! same box when certified, and as two curves touching when tolerant, so what
//! refused was the corner and not a side along the axis. `fuse/arrange.rs` now
//! counts the bottom circle from the seam corner's box, since no other node
//! can lie between that box and where the circle starts
//! (`Arrangement::count_each_loop_from_the_node_round_its_start`, Q1737).
//! Every fuse now answers what it holds by hand under both policies and logs
//! nothing, and the tilted notch answers with the volume it has unturned.
//!
//! **One radian about `(1, 1, 1)`: answered, though a side lies a rounding off
//! parallel to the axis.** The turn rounds each side's normal and the bar's
//! axis apart, so every side leans against the pipe by a rounding. Its plane's
//! crossing with the pipe is then named a closed one, an ellipse whose sweep
//! starts more than `1e15` from the bar. Every fuse, under either policy, was
//! refused *order along a crossing*: first between the two points where one
//! side's plane crosses the wall at the two caps, `5` apart, with an enclosure
//! past `100`, and then, once those were ordered, between a cap's and where
//! the line two sides of the notch cross in leaves the pipe, `1e16` off.
//! `fuse/arrange.rs` orders both now (`along_the_stretch_between_the_two` and
//! `Arrangement::as_the_pipes_axis_says`). Every fuse was then refused
//! `CannotNameWhichPieceOfACrossingThisIs` on a wall edge along a side's
//! ellipse (Q1885), until Q1882 walked such an arc up the pipe. Every fuse now
//! answers what it holds by hand under both policies and logs nothing, and so
//! does the seam notch tilted off the axis, with the volume it has unturned.
//!
//! **The half turn by rotation was a gap, not a limit** (§16.10), and it is
//! closed. A turn moves no material, so each answer exists and is the unturned
//! one. The exact half turn reached it for the same shape, and so did the
//! one-radian turn, whose rounding is in the turn itself.
//!
//! *Where a plane and a pipe cross*, the predicate Q765 named, refuses none of
//! these fuses.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, CORNER_AFTER_MOVING_ROUNDOFF, Cylinder, Deflection, Isometry, Plane, Point, Scale,
    Surface, SurfaceMeet, Unit, Vector, how_a_plane_leans_against_a_pipe, intersect,
    where_a_closed_crossing_starts, where_a_corner_lands_after_moving,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_frustum;

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 5.0;
const SIDE: f64 = 2.0;
const RULES: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// A notch's corner nearest the origin, before any turn: the seam notch is
/// `[2, 4] × [−1, 1] × [1, 3]`.
const THE_SEAM_NOTCH: [f64; 3] = [2.0, -1.0, 1.0];
/// `[−1, 1] × [2, 4] × [1, 3]`.
const THE_WALL_NOTCH: [f64; 3] = [-1.0, 2.0, 1.0];
const BOTH_NOTCHES: [[f64; 3]; 2] = [THE_SEAM_NOTCH, THE_WALL_NOTCH];

/// The controls: the bar unturned, and turned half a turn about `y` by two
/// quarter turns, which round nothing.
#[test]
fn a_notched_bar_unturned_or_turned_an_exact_half_turn_answers_every_fuse_by_hand() {
    answers_every_fuse_by_hand("unturned", &Isometry::identity());
    answers_every_fuse_by_hand("an exact half turn", &half_a_turn_exactly());
}

/// Every side of either notch, turned by `rotation(y, π)` with the bar, runs
/// exactly along the bar's axis: the turn rounds a side's normal and the axis
/// the same way, and `n · w` comes out zero.
#[test]
fn turned_half_a_turn_by_rotation_every_notch_side_runs_exactly_along_the_bar() {
    let turn = half_a_turn_by_rotation();
    for notch in BOTH_NOTCHES {
        let (model, bar, notch_key) = a_notched_bar(&turn, notch, &Isometry::identity());
        let pipe = the_pipe(&model, bar);
        for side in the_sides(&model, notch_key, &turn) {
            assert_eq!(
                how_a_plane_leans_against_a_pipe(&side, &pipe),
                Decision::Zero,
                "notch at {notch:?}, side {side:?}"
            );
        }
    }
}

/// What tells the two half turns apart is the bar's seam corner on its bottom
/// cap. Two quarter turns hand it back as the exact point it lands on;
/// `rotation(y, π)` hands it back as a box `± CORNER_AFTER_MOVING_ROUNDOFF · 3`
/// round where it lands.
#[test]
fn the_seam_corner_is_a_box_after_rotation_by_pi_and_a_point_after_two_quarter_turns() {
    let exactly = half_a_turn_exactly();
    let (corner, wobble) = the_seam_corner_as_the_fuse_reads_it(&exactly);
    assert_eq!(wobble, 0.0);
    assert_eq!(corner, PredicatePoint::from([-RADIUS, 0.0, 0.0]));

    let rounded = half_a_turn_by_rotation();
    let (corner, wobble) = the_seam_corner_as_the_fuse_reads_it(&rounded);
    assert_eq!(wobble, CORNER_AFTER_MOVING_ROUNDOFF * RADIUS);
    let lands = rounded
        .apply_to_point(Point::new(RADIUS, 0.0, 0.0))
        .coordinates();
    for (side, at) in corner.enclosure().into_iter().zip(lands) {
        assert!(side.lo() < at && at < side.hi(), "{at} outside {side:?}");
        assert!(side.width() >= wobble, "{side:?} is narrower than {wobble}");
    }
}

/// Q885's case on a designer's shape: the bottom cap circle starts inside the
/// seam corner's box, and a side's line crosses the circle a little way round.
/// The circle is counted from that box, so every fuse of either notch, under
/// either policy, answers what it holds by hand and logs nothing, as the exact
/// half turn does.
#[test]
fn turned_half_a_turn_by_rotation_every_fuse_answers_what_it_holds_by_hand() {
    answers_every_fuse_by_hand("half a turn by rotation", &half_a_turn_by_rotation());
}

/// The seam notch tilted off the bar's axis, so that no side of it runs along
/// the bar, answers every fuse under the same turn and both policies with the
/// volume it has unturned. It was refused at the seam corner's box when
/// certified, so what refused was the corner and not a side along the axis.
#[test]
fn turned_half_a_turn_by_rotation_a_notch_tilted_off_the_axis_answers_every_fuse() {
    answers_every_tilted_fuse_as_unturned(&half_a_turn_by_rotation());
}

/// Turned one radian about `(1, 1, 1)`, every side of either notch leans
/// against the bar's axis by a rounding. Its plane's crossing with the pipe is
/// named a closed one, and the sweep round it starts more than `1e15` from the
/// bar.
#[test]
fn turned_one_radian_about_a_diagonal_every_notch_side_leans_by_a_rounding_and_crosses_far_off() {
    let turn = one_radian_about_a_diagonal();
    for notch in BOTH_NOTCHES {
        let (model, bar, notch_key) = a_notched_bar(&turn, notch, &Isometry::identity());
        let pipe = the_pipe(&model, bar);
        let wall = Surface::Cylinder(pipe);
        for side in the_sides(&model, notch_key, &turn) {
            let what = format!("notch at {notch:?}, side {side:?}");
            let leans = how_a_plane_leans_against_a_pipe(&side, &pipe);
            assert!(
                matches!(leans, Decision::Pos | Decision::Neg),
                "{what}: {leans:?}"
            );
            let flat = Surface::Plane(side);
            let SurfaceMeet::Curves(runs) = intersect(
                &flat,
                &wall,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a plane and a pipe meet in something named") else {
                panic!("{what}: no curves");
            };
            let [run] = runs.as_slice() else {
                panic!("{what}: {runs:?}");
            };
            let [x, y, z] =
                where_a_closed_crossing_starts(run, [&flat, &wall], &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"))
                    .coordinates();
            let how_far = (x * x + y * y + z * z).sqrt();
            assert!(how_far > 1e15, "{what}: starts {how_far} from the bar");
        }
    }
}

/// Every fuse of either notch, under either policy, answers what it holds by
/// hand and logs nothing. It puts the nodes round each side's long ellipse in
/// order: where a side meets the caps and the notch's ends, near the bar, and
/// where the line two sides of the notch cross in leaves the pipe, `1e16` off.
/// It then walks each wall edge on such an ellipse up the pipe (Q1882), where
/// it was refused `CannotNameWhichPieceOfACrossingThisIs` (Q1885).
#[test]
fn turned_one_radian_about_a_diagonal_every_fuse_answers_what_it_holds_by_hand() {
    answers_every_fuse_by_hand("one radian about (1, 1, 1)", &one_radian_about_a_diagonal());
}

/// The control for the one-radian turn: the seam notch tilted so that no side
/// runs along the bar answers every fuse under both policies, with the volume
/// the same fuse has unturned.
#[test]
fn turned_one_radian_about_a_diagonal_a_notch_tilted_off_the_axis_answers_every_fuse() {
    answers_every_tilted_fuse_as_unturned(&one_radian_about_a_diagonal());
}

const BALL_RADIUS: f64 = 3.0;

/// A box straddling the north pole: `x, y ∈ [−1, 1]`, `z ∈ [2, 4]`, so its
/// centre line `x = y = 0` runs through the pole at `(0, 0, 3)` and its
/// farthest corner is `√2` from that line — inside the ball down to where the
/// ball's own radius at that height, `√(9 − z²)`, falls to `√2` at
/// `z = √7 ≈ 2.6458`. Below that the whole square footprint is inside the
/// ball, so the overlap's cross-section is simply `√(9 − x² − y²) − 2`, never
/// clipped by the ball's own curve, for every `(x, y)` the box reaches.
const POLAR_NOTCH: [f64; 3] = [-1.0, -1.0, 2.0];

/// `BALL_RADIUS³ · 4π / 3`.
fn the_balls_own_volume() -> f64 {
    4.0 / 3.0 * PI * BALL_RADIUS.powi(3)
}

/// The part of the ball inside [`POLAR_NOTCH`]'s box, by a composite Simpson's
/// rule over `x, y ∈ [−1, 1]` of `√(9 − x² − y²) − 2` — a method the kernel's
/// own `measure`, which integrates a triangulated mesh, never touches. 100
/// panels in each direction converges `f64`-tight: doubling to 200 moves the
/// answer by under `1e-12`, far below the volume error the fuse itself
/// reports.
fn the_polar_notchs_overlap_with_the_ball() -> f64 {
    simpsons_rule_over_a_square(-1.0, 1.0, -1.0, 1.0, 100, |x, y| {
        (BALL_RADIUS * BALL_RADIUS - x * x - y * y).sqrt() - 2.0
    })
}

/// Composite Simpson's rule for `∫∫ f(x, y) dx dy` over `[x0, x1] × [y0, y1]`,
/// `panels` panels (even) along each axis.
fn simpsons_rule_over_a_square(
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    panels: usize,
    f: impl Fn(f64, f64) -> f64,
) -> f64 {
    assert_eq!(panels % 2, 0, "Simpson's rule needs an even panel count");
    let weight = |i: usize| {
        if i == 0 || i == panels {
            1.0
        } else if i % 2 == 1 {
            4.0
        } else {
            2.0
        }
    };
    let step_x = (x1 - x0) / panels as f64;
    let step_y = (y1 - y0) / panels as f64;
    let mut total = 0.0;
    for i in 0..=panels {
        let x = x0 + step_x * i as f64;
        let mut row = 0.0;
        for j in 0..=panels {
            let y = y0 + step_y * j as f64;
            row += weight(j) * f(x, y);
        }
        total += weight(i) * row;
    }
    total * step_x * step_y / 9.0
}

/// A ball of radius 3 and a block of side 2 straddling its north pole
/// ([`POLAR_NOTCH`]), both turned half a turn about `y` by `rotation`, fused
/// by every rule both ways round (the ball as argument and as tool), under
/// both policies. Q1737's fix to `Arrangement::the_node_round_the_start`
/// matches a plane and a pipe only — this is a plane and a *sphere*, the one
/// other surface whose loop is counted from a start point the half turn
/// rounds (the moved pole, not a moved corner, but the same
/// `CORNER_AFTER_MOVING_ROUNDOFF`-sized box).
#[test]
fn a_turned_ball_notched_near_its_pole_by_a_turned_block_answers_every_fuse_both_ways_round() {
    let turn = half_a_turn_by_rotation();
    let overlap = the_polar_notchs_overlap_with_the_ball();
    let ball_volume = the_balls_own_volume();
    for tolerant in [false, true] {
        let policy = if tolerant {
            Policy::tolerant(5.0)
        } else {
            Policy::certified()
        };
        for rule in RULES {
            for ball_is_argument in [true, false] {
                let what =
                    format!("tolerant {tolerant}, {rule:?}, ball is argument {ball_is_argument}");
                let (mut model, ball, notch) = a_turned_ball_and_polar_notch(&turn);
                let (first, second) = if ball_is_argument {
                    (ball, notch)
                } else {
                    (notch, ball)
                };
                let mut log = ProvenanceLog::new();
                let fused = model
                    .fuse(&[first], &[second], rule, &policy, &mut log)
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"))
                    .solids()
                    .to_vec();
                assert!(log.is_empty(), "{what}: logged {log:?}");
                let by_hand = what_the_ball_and_notch_hold_by_hand(
                    rule,
                    ball_is_argument,
                    ball_volume,
                    overlap,
                );
                match by_hand {
                    Some(expected) => {
                        let (volume, error) = the_volume_of_the_one_solid(&model, Ok(fused), &what);
                        assert!(
                            (volume - expected).abs() <= error,
                            "{what}: measured {volume} ± {error}, by hand {expected}"
                        );
                    }
                    None => assert!(fused.is_empty(), "{what}: expected no solid, got {fused:?}"),
                }
            }
        }
    }
}

/// What fusing the ball with the polar notch holds, by rule and by which one
/// is the argument. `None` means the rule removes the whole block or the
/// whole ball, leaving no solid at all.
fn what_the_ball_and_notch_hold_by_hand(
    rule: Rule,
    ball_is_argument: bool,
    ball_volume: f64,
    overlap: f64,
) -> Option<f64> {
    let block_volume = SIDE * SIDE * SIDE;
    match rule {
        Rule::Union => Some(ball_volume + block_volume - overlap),
        Rule::Intersection => Some(overlap),
        Rule::Difference if ball_is_argument => Some(ball_volume - overlap),
        Rule::Difference => Some(block_volume - overlap),
        other => panic!("no volume worked out by hand for {other:?}"),
    }
}

/// The ball and the block of [`POLAR_NOTCH`], both turned by `turn`.
fn a_turned_ball_and_polar_notch(turn: &Isometry) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 5.0).expect("5 mm is a working size"));
    let ball = model
        .add_ball(BALL_RADIUS, turn)
        .expect("a ball turned half a turn");
    let [x, y, z] = POLAR_NOTCH;
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [SIDE, 0.0], [SIDE, SIDE], [0.0, SIDE]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square is a profile");
    let notch = model
        .add_extrusion(
            &square,
            SIDE,
            &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        )
        .expect("a block");
    model
        .place_solid(notch, turn)
        .expect("a turn moves a block");
    (model, ball, notch)
}

/// The frustum's radius at its bottom cap: `1`, where the block's planes
/// `y = ±1` touch the bottom rim, or `0.5`, where none of the block's planes
/// meets it.
const FRUSTUM_NEARS: [f64; 2] = [1.0, 0.5];
const FRUSTUM_FAR: f64 = 10.0;
/// `slope = 1`, a power of two, so `slope · height` rounds to nothing at
/// either cap (cones/mod.rs's own discipline).
const FRUSTUM_SLOPE: f64 = 1.0;
/// `[2, 4] × [−1, 1] × [5, 7]`: every corner's distance off the axis is at
/// most `√(4² + 1²) = √17 ≈ 4.123`, and the frustum's radius at the lowest
/// corner height, `z = 5`, is `5` (`slope = 1`) — so the whole block sits
/// inside the frustum, corner to corner.
const FRUSTUM_NOTCH: [f64; 3] = [2.0, -1.0, 5.0];

/// `(π / 3) · height · (r_near² + r_near · r_far + r_far²)`.
fn the_frustums_own_volume(near: f64) -> f64 {
    let height = FRUSTUM_FAR - near;
    let r_near = FRUSTUM_SLOPE * near;
    let r_far = FRUSTUM_SLOPE * FRUSTUM_FAR;
    PI / 3.0 * height * (r_near * r_near + r_near * r_far + r_far * r_far)
}

/// A frustum from radius 1 or 0.5 to radius 10 and a block of side 2 that
/// sits wholly inside it ([`FRUSTUM_NOTCH`]), unturned or turned an exact half
/// turn about `y` (two quarter turns, which round nothing), fused by every
/// rule both ways round, under both policies: every combination answers what
/// it holds by hand, with nothing logged.
#[test]
fn a_block_wholly_inside_a_frustum_unturned_or_turned_an_exact_half_turn_answers_every_fuse_both_ways_round()
 {
    for near in FRUSTUM_NEARS {
        answers_every_frustum_fuse_both_ways_round("unturned", &Isometry::identity(), near, None);
        answers_every_frustum_fuse_both_ways_round(
            "an exact half turn",
            &half_a_turn_exactly(),
            near,
            None,
        );
    }
}

/// Turned half a turn about `y` by `rotation`, every combination still answers
/// **except** `Difference` with the frustum as the argument — cutting the block
/// out of the frustum. That one is refused
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] on the frustum's lateral
/// face, under both policies, though the block never touches that face: its
/// farthest corner is `0.88` inside it, at the tightest end.
///
/// It is refused where the fuse asks which shell the block's hollow sits in.
/// A ray from a corner of the hollow crosses the frustum's wall, and the wall
/// keeps the side of its bottom circle's plane away from the apex, read off
/// every corner of the wall off that plane. `sin π` rounds to `1.2e-16`, so the
/// turned circle's own corner, `(−0.9999999999999998, 0, −1)`, is stored a
/// rounding off the turned plane, `normal (−1.2e-16, 0, 1), offset −1`, on the
/// apex's side, while the far rim's corners are on the other. So the corners
/// disagree, and `holds.rs`'s `the_side_away_from_the_apex_a_face_keeps` gives
/// up. Leaving the circle's own corner out of that reading, as a probe did,
/// answers both radii with the volume by hand (Q1969).
#[test]
fn turned_half_a_turn_by_rotation_cutting_the_block_out_of_the_frustum_is_refused() {
    for near in FRUSTUM_NEARS {
        answers_every_frustum_fuse_both_ways_round(
            "half a turn by rotation",
            &half_a_turn_by_rotation(),
            near,
            Some(Rule::Difference),
        );
    }
}

/// The block cuts the frustum's wall nowhere, so the fuse gives the wall back
/// as one face, a band between the bottom circle and the far rim, with the two
/// caps beside it: three faces for the union, and those three and the block's
/// six for cutting the block out. Before, wherever no plane of the block
/// touched the bottom rim (`near = 0.5`), and under the turn by `rotation`,
/// the wall came back in 25 pieces: 27 faces for the union and 33 for the
/// difference (Q1969). Turned by `rotation`, the difference is still refused
/// ([`turned_half_a_turn_by_rotation_cutting_the_block_out_of_the_frustum_is_refused`]).
#[test]
fn a_block_wholly_inside_a_frustum_leaves_the_frustums_wall_one_face() {
    let both: &[(Rule, usize)] = &[(Rule::Union, 3), (Rule::Difference, 9)];
    for (which, turn, rules) in [
        ("unturned", Isometry::identity(), both),
        ("an exact half turn", half_a_turn_exactly(), both),
        (
            "half a turn by rotation",
            half_a_turn_by_rotation(),
            &both[..1],
        ),
    ] {
        for near in FRUSTUM_NEARS {
            for &(rule, faces) in rules {
                let what = format!("{which}, near {near}, {rule:?}");
                let (mut model, frustum, notch) = a_turned_frustum_and_inside_notch(&turn, near);
                let fused = model
                    .fuse(
                        &[frustum],
                        &[notch],
                        rule,
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                let [solid] = fused.solids() else {
                    panic!("{what}: {} solids", fused.solids().len());
                };
                let view = model.solid(*solid).expect("the fuse made the key");
                assert_eq!(view.faces().count(), faces, "{what}");
            }
        }
    }
}

/// Fuses the frustum of [`FRUSTUM_NOTCH`] from radius `near` with the block
/// wholly inside it, both placed by `turn`, by every rule both ways round,
/// under both policies, and checks each answers what it holds by hand with
/// nothing logged — except `refused_rule` with the frustum as the argument,
/// which it checks is refused [`Error::AFaceTrimmedByACurveThisCannotFollow`]
/// instead.
fn answers_every_frustum_fuse_both_ways_round(
    which: &str,
    turn: &Isometry,
    near: f64,
    refused_rule: Option<Rule>,
) {
    let frustum_volume = the_frustums_own_volume(near);
    let block_volume = SIDE * SIDE * SIDE;
    for tolerant in [false, true] {
        let policy = if tolerant {
            Policy::tolerant(5.0)
        } else {
            Policy::certified()
        };
        for rule in RULES {
            for frustum_is_argument in [true, false] {
                let what = format!(
                    "{which}, tolerant {tolerant}, {rule:?}, frustum is argument {frustum_is_argument}"
                );
                let (mut model, frustum, notch) = a_turned_frustum_and_inside_notch(turn, near);
                let (first, second) = if frustum_is_argument {
                    (frustum, notch)
                } else {
                    (notch, frustum)
                };
                let mut log = ProvenanceLog::new();
                let result = model.fuse(&[first], &[second], rule, &policy, &mut log);
                if Some(rule) == refused_rule && frustum_is_argument {
                    let why = result.expect_err(&format!("{what}: expected a refusal"));
                    assert!(
                        matches!(why, Error::AFaceTrimmedByACurveThisCannotFollow { .. }),
                        "{what}: {why:?}"
                    );
                    continue;
                }
                let fused = result
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"))
                    .solids()
                    .to_vec();
                assert!(log.is_empty(), "{what}: logged {log:?}");
                let by_hand = match rule {
                    Rule::Union => Some(frustum_volume),
                    Rule::Intersection => Some(block_volume),
                    Rule::Difference if frustum_is_argument => Some(frustum_volume - block_volume),
                    Rule::Difference => None,
                    other => panic!("no volume worked out by hand for {other:?}"),
                };
                match by_hand {
                    Some(expected) => {
                        let (volume, error) = the_volume_of_the_one_solid(&model, Ok(fused), &what);
                        assert!(
                            (volume - expected).abs() <= error,
                            "{what}: measured {volume} ± {error}, by hand {expected}"
                        );
                    }
                    None => assert!(fused.is_empty(), "{what}: expected no solid, got {fused:?}"),
                }
            }
        }
    }
}

/// The frustum from radius `near` and the block of [`FRUSTUM_NOTCH`], both
/// turned by `turn`.
fn a_turned_frustum_and_inside_notch(turn: &Isometry, near: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 5.0).expect("5 mm is a working size"));
    let frustum = a_frustum(&mut model, FRUSTUM_SLOPE, near, FRUSTUM_FAR);
    model
        .place_solid(frustum, turn)
        .expect("a turn moves a frustum");
    let [x, y, z] = FRUSTUM_NOTCH;
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [SIDE, 0.0], [SIDE, SIDE], [0.0, SIDE]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square is a profile");
    let notch = model
        .add_extrusion(
            &square,
            SIDE,
            &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        )
        .expect("a block");
    model
        .place_solid(notch, turn)
        .expect("a turn moves a block");
    (model, frustum, notch)
}

/// Fuses the seam notch tilted off the bar's axis with the bar, unturned and
/// both placed by `turn`, by every rule under both policies, and checks that
/// each comes back one solid with nothing logged, and the turned one with the
/// unturned one's volume.
fn answers_every_tilted_fuse_as_unturned(turn: &Isometry) {
    for tolerant in [false, true] {
        for rule in RULES {
            let what = format!("tolerant {tolerant}, {rule:?}");
            let (model, fused, log) = notched_and_fused(
                &Isometry::identity(),
                THE_SEAM_NOTCH,
                &a_tilt(),
                rule,
                tolerant,
            );
            assert!(log.is_empty(), "{what}, unturned: logged {log:?}");
            let (unturned, unturned_error) =
                the_volume_of_the_one_solid(&model, fused, &format!("{what}, unturned"));

            let (model, fused, log) =
                notched_and_fused(turn, THE_SEAM_NOTCH, &a_tilt(), rule, tolerant);
            assert!(log.is_empty(), "{what}, turned: logged {log:?}");
            let (turned, turned_error) =
                the_volume_of_the_one_solid(&model, fused, &format!("{what}, turned"));
            assert!(
                (turned - unturned).abs() <= turned_error + unturned_error,
                "{what}: turned {turned} ± {turned_error}, unturned {unturned} ± {unturned_error}"
            );
        }
    }
}

/// Fuses either notch with the bar, both placed by `turn`, by every rule under
/// both policies, and checks that each comes back one sound solid holding what
/// [`what_it_holds_by_hand`] says, with nothing logged.
fn answers_every_fuse_by_hand(which: &str, turn: &Isometry) {
    for notch in BOTH_NOTCHES {
        for tolerant in [false, true] {
            for rule in RULES {
                let what = format!("{which}, notch at {notch:?}, tolerant {tolerant}, {rule:?}");
                let (model, fused, log) =
                    notched_and_fused(turn, notch, &Isometry::identity(), rule, tolerant);
                let (volume, error) = the_volume_of_the_one_solid(&model, fused, &what);
                let by_hand = what_it_holds_by_hand(rule);
                assert!(
                    (volume - by_hand).abs() <= error,
                    "{what}: measured {volume} ± {error}, by hand {by_hand}"
                );
                assert!(log.is_empty(), "{what}: logged {log:?}");
                model
                    .validate(&Policy::certified(), &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            }
        }
    }
}

fn half_a_turn_exactly() -> Isometry {
    Isometry::quarter_turns(&Axis::y(), 2).expect("two quarter turns about y")
}

fn half_a_turn_by_rotation() -> Isometry {
    Isometry::rotation(&Axis::y(), PI).expect("half a turn about y")
}

fn one_radian_about_a_diagonal() -> Isometry {
    let diagonal =
        Axis::new(Point::origin(), Vector::new(1.0, 1.0, 1.0)).expect("a finite direction");
    Isometry::rotation(&diagonal, 1.0).expect("one radian about (1, 1, 1)")
}

/// `0.3` radians about an axis through `(3, 0, 2)`, on the seam inside the
/// notch, that runs along no side of it.
fn a_tilt() -> Isometry {
    let through_the_notch =
        Axis::new(Point::new(3.0, 0.0, 2.0), Vector::new(1.0, 2.0, 0.5)).expect("a finite axis");
    Isometry::rotation(&through_the_notch, 0.3).expect("a tilt")
}

/// Builds the bar and a notch with its corner at `notch`, and fuses them by
/// `rule`, bar first.
fn notched_and_fused(
    turn: &Isometry,
    notch: [f64; 3],
    tilt: &Isometry,
    rule: Rule,
    tolerant: bool,
) -> (Model, Result<Vec<SolidKey>, Error>, ProvenanceLog) {
    let (mut model, bar, notch_key) = a_notched_bar(turn, notch, tilt);
    let policy = if tolerant {
        Policy::tolerant(model.scale().nominal_size())
    } else {
        Policy::certified()
    };
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(&[bar], &[notch_key], rule, &policy, &mut log)
        .map(|fused| fused.solids().to_vec());
    (model, fused, log)
}

/// The bar, placed by `turn`, and the notch, placed by `turn` after `tilt`.
fn a_notched_bar(
    turn: &Isometry,
    [x, y, z]: [f64; 3],
    tilt: &Isometry,
) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 5.0).expect("5 mm is a working size"));
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar");
    model.place_solid(bar, turn).expect("a turn moves a bar");
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [SIDE, 0.0], [SIDE, SIDE], [0.0, SIDE]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square is a profile");
    let notch = model
        .add_extrusion(
            &square,
            SIDE,
            &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        )
        .expect("a block");
    model
        .place_solid(notch, &turn.compose(tilt))
        .expect("a turn moves a block");
    (model, bar, notch)
}

fn the_volume_of_the_one_solid(
    model: &Model,
    fused: Result<Vec<SolidKey>, Error>,
    what: &str,
) -> (f64, f64) {
    let fused = fused.unwrap_or_else(|why| panic!("{what}: {why:?}"));
    let [solid] = fused.as_slice() else {
        panic!("{what}: {} solids", fused.len());
    };
    let measured = model
        .solid(*solid)
        .expect("the fuse made the key")
        .measure(
            Deflection::for_scale(model.scale()),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid the fuse made can be measured");
    (measured.volume(), measured.volume_error())
}

/// The bar is `π · 3² · 5 = 45π`. Of it, either notch holds its height `2`
/// times the part of `[2, 4] × [−1, 1]` inside the circle of radius 3, which is
/// `∫ (√(9 − y²) − 2) dy` over `y` from `−1` to `1`, or `√8 + 9 asin(1/3) − 4`.
/// The wall notch is the seam notch a quarter turn round the axis.
fn what_it_holds_by_hand(rule: Rule) -> f64 {
    let the_bar = PI * RADIUS * RADIUS * HEIGHT;
    let the_notch = SIDE * SIDE * SIDE;
    let held = SIDE * (8.0_f64.sqrt() + RADIUS * RADIUS * (1.0_f64 / 3.0).asin() - 4.0);
    match rule {
        Rule::Union => the_bar + the_notch - held,
        Rule::Intersection => held,
        Rule::Difference => the_bar - held,
        other => panic!("no volume worked out by hand for {other:?}"),
    }
}

fn the_pipe(model: &Model, bar: SolidKey) -> Cylinder {
    let mut pipes = model
        .solid(bar)
        .expect("this model made the bar")
        .faces()
        .map(|face| face.surface())
        .filter(|surface| matches!(surface, Surface::Cylinder(_)));
    let (Some(Surface::Cylinder(pipe)), None) = (pipes.next(), pipes.next()) else {
        panic!("a round bar has one wall");
    };
    pipe
}

/// The planes of the notch's four sides: the faces whose normals, moved back
/// by the turn, lie across the bar's axis.
fn the_sides(model: &Model, notch: SolidKey, turn: &Isometry) -> Vec<Plane> {
    let back = turn.inverse();
    let sides: Vec<Plane> = model
        .solid(notch)
        .expect("this model made the notch")
        .faces()
        .map(|face| {
            let Surface::Plane(flat) = face.surface() else {
                panic!("a block is flat");
            };
            flat
        })
        .filter(|flat| back.apply_to_vector(flat.normal()).components()[2].abs() < 0.5)
        .collect();
    assert_eq!(sides.len(), 4, "a block has four sides");
    sides
}

/// What `where_a_corner_lands_after_moving` makes of the bar's seam corner on
/// its bottom cap, `(3, 0, 0)`, with the cap and the wall round it as the bar
/// stores them, and how far it says the corner may be from the box's middle.
fn the_seam_corner_as_the_fuse_reads_it(turn: &Isometry) -> (PredicatePoint, f64) {
    let bottom = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
            .expect("a unit normal"),
    );
    let wall = Surface::Cylinder(Cylinder::new(Axis::z(), RADIUS).expect("a positive radius"));
    let (corner, wobble) =
        where_a_corner_lands_after_moving(Point::new(RADIUS, 0.0, 0.0), [&bottom, &wall], turn)
            .expect("a finite corner and a finite move");
    (corner, wobble.magnitude())
}
