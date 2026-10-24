//! A surface the face across leaves out, and an edge both faces must split
//! alike (Q2364, Q2382, `docs/decisions.md` §422).
//!
//! A face leaves out a cut whose faces meet it only on its boundary, unless a
//! node of the cut splits an edge the face across splits there too. The face
//! across says what it leaves out through `its_neighbour_leaves_out`, which
//! holds surfaces as that face spelled them. §422.8 feared a third spelling:
//! the face across leaves out one spelling of a pipe but keeps its run,
//! because another spelling it sees draws the run there, and this face drops
//! the run where the face across splits the edge.
//!
//! **That does not show here.** Two bars stacked on one pipe, spelled four
//! ways, and a box in seven places answer every rule both ways round. In this
//! shape every spelling that draws the face across's run through a node on
//! the shared edge passes through that node, so it cuts this face there too,
//! and this face draws it by the same spelling. A spelling that only touches
//! this face at the node is not built.
//!
//! **What did show is a step beside a bar.** The step's lower top crosses the
//! bar's pipe in a circle wholly above the bar and leaves it out, but still
//! splits its edge with the riser where the circle meets it, since the riser
//! crosses the pipe in two lines that run on up. The riser listed the pipe as
//! reaching it only on its boundary and left its lines out, so the edge was
//! split on one side alone, and three of the six fuses were refused
//! `ResultIsNotManifold`.
//!
//! **And a top that climbs past the bar in two leaning faces** (Q2382). Both
//! are wholly above the bar, so both list its pipe, and neither crosses the
//! pipe in a curve wholly above it. The lower one keeps its curve, since the
//! curve runs on into the block's side, which crosses the pipe for real. The
//! upper one was told that the lower one may leave the pipe out, took that for
//! what it does, and left its own curve out, so their edge was split on the
//! lower face's side alone, and three of the six fuses were refused
//! `ResultIsNotManifold`.
//!
//! **Two more cases Q2382 left unbuilt** (Q2541). Two spellings of the bars'
//! pipe beside the raised step or climbing top answer every rule, so a face
//! that weighs one spelling at the node alone answers as the face across
//! does. A rim crossing that is also a listed cut, drawn on past the rim, is
//! not reached: three slabs and blocks with a D-shaped hole beside a bar on
//! the hole's pipe answer every rule, and none has a face that lists the pipe
//! and draws it along its rim.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 2.0;
const MEETING: f64 = 2.0;

/// How the upper bar's wall writes the pipe both bars stand on.
#[derive(Clone, Copy, Debug)]
enum Wall {
    /// With the lower bar's numbers.
    SpelledOnce,
    /// With its axis starting at `z = 2` rather than `z = 0`.
    OriginSlidUp,
    /// With its axis pointing down.
    AxisTurnedRound,
    /// Both bars made by `add_round_bar`, the upper one moved up.
    TwoRoundBars,
}

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// Two bars of radius 2 stacked on one pipe, the upper one's wall spelled
/// four ways, and a box over `x ∈ [1, 3]`, `|y| ≤ 0.5` at seven heights,
/// every rule both ways round, against the closed forms: what the box and the
/// bars share is the part of the bars' disc with `x ≥ 1` and `|y| ≤ 0.5`, as
/// tall as their heights overlap.
#[test]
fn two_spellings_of_one_pipe_and_a_box_answer_every_rule_either_way_round() {
    let heights: [[f64; 2]; 7] = [
        [2.5, 5.0],
        [-1.0, 1.5],
        [2.5, 3.5],
        [3.0, 5.0],
        [1.0, 3.0],
        [4.5, 6.0],
        [-2.0, -0.5],
    ];
    let walls = [
        Wall::SpelledOnce,
        Wall::OriginSlidUp,
        Wall::AxisTurnedRound,
        Wall::TwoRoundBars,
    ];
    let (x0, x1, half) = (1.0, 3.0, 0.5);
    let strip = half * (RADIUS * RADIUS - half * half).sqrt()
        + RADIUS * RADIUS * (half / RADIUS).asin()
        - 2.0 * half * x0;
    let mut failed = Vec::new();
    let mut ran = 0;
    for z in heights {
        let shared = (z[1].min(2.0 * MEETING) - z[0].max(0.0)).max(0.0) * strip;
        let the_box = (x1 - x0) * 2.0 * half * (z[1] - z[0]);
        let bars = PI * RADIUS * RADIUS * 2.0 * MEETING;
        for wall in walls {
            for rule in EVERY_RULE {
                for bars_first in [true, false] {
                    let by_hand = match (rule, bars_first) {
                        (Rule::Union, _) => bars + the_box - shared,
                        (Rule::Intersection, _) => shared,
                        (Rule::Difference, true) => bars - shared,
                        (Rule::Difference, false) => the_box - shared,
                        _ => unreachable!(),
                    };
                    let mut model = Model::new(Scale::millimetres(100.0).expect("a size"));
                    let [lower, upper] = the_two_bars(&mut model, wall);
                    let boxed = a_box(&mut model, [[x0, x1], [-half, half], z]);
                    let (arguments, tools) = if bars_first {
                        (vec![lower, upper], vec![boxed])
                    } else {
                        (vec![boxed], vec![lower, upper])
                    };
                    let asked = format!("{z:?} {wall:?} {rule:?} bars_first={bars_first}");
                    ran += 1;
                    if let Some(wrong) = check(&mut model, &arguments, &tools, rule, by_hand) {
                        println!("{asked}: {wrong}");
                        failed.push(asked);
                    }
                }
            }
        }
    }
    println!("ran {ran}, failed {}", failed.len());
    assert!(failed.is_empty(), "{failed:#?}");
}

/// A block over `x ∈ [-3, 3]`, `z ∈ [-1, 3]`, with a step over `x ∈ [1.5, 3]`
/// up to `z = 4`, drawn in `x` and `z` for [`a_block_across_y`].
const A_STEP: [[f64; 2]; 6] = [
    [-3.0, -1.0],
    [3.0, -1.0],
    [3.0, 4.0],
    [1.5, 4.0],
    [1.5, 3.0],
    [-3.0, 3.0],
];

/// A block over `x ∈ [-3, 3]` from `z = -1`, its top flat at `z = 2.2` up to
/// `x = 0.5`, then leaning up to `z = 3` at `x = 1.5` and on, steeper, to
/// `z = 5` at `x = 2.1`, and flat again from there, drawn in `x` and `z` for
/// [`a_block_across_y`].
const A_TOP_CLIMBING_IN_TWO_LEANING_FACES: [[f64; 2]; 7] = [
    [-3.0, -1.0],
    [3.0, -1.0],
    [3.0, 5.0],
    [2.1, 5.0],
    [1.5, 3.0],
    [0.5, 2.2],
    [-3.0, 2.2],
];

/// A bar of radius 2 standing on `z ∈ [0, 2]` and the step of [`A_STEP`],
/// every rule both ways round, against the closed forms of
/// [`every_rule_beside_the_bar`].
#[test]
fn a_step_beside_a_bar_answers_every_rule_either_way_round() {
    let failed = every_rule_beside_the_bar(&A_STEP);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// A bar of radius 2 standing on `z ∈ [0, 2]` and the block of
/// [`A_TOP_CLIMBING_IN_TWO_LEANING_FACES`], every rule both ways round,
/// against the closed forms of [`every_rule_beside_the_bar`].
///
/// The two leaning faces are wholly above the bar, so each lists the bar's
/// pipe as reaching it only on its boundary, and each crosses the pipe in an
/// ellipse that dips below the bar, so neither leaves it out as reached by no
/// face. They meet along `x = 1.5`, `z = 3`, which the pipe crosses at
/// `y = ±√1.75`. The lower one keeps its ellipse, since it runs on into the
/// block's sides at `y = ±1.5`, which cross the pipe below the bar's top too
/// and do not list it. So it splits the edge at `y = ±√1.75`, and the upper
/// one must as well.
#[test]
fn a_top_climbing_past_a_bar_in_two_leaning_faces_answers_every_rule_either_way_round() {
    let failed = every_rule_beside_the_bar(&A_TOP_CLIMBING_IN_TWO_LEANING_FACES);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// Two bars of radius 2 stacked on one pipe over `z ∈ [0, 4]`, the upper
/// one's wall spelled four ways, and the blocks of [`A_STEP`] and
/// [`A_TOP_CLIMBING_IN_TWO_LEANING_FACES`] raised by 2, every rule both ways
/// round (Q2541).
///
/// The raised blocks stand from `z = 1` and their lowest top is at `z = 4.2`,
/// so each top, the step's riser and the two leaning faces are wholly above
/// both bars, and each lists both spellings of the pipe. The riser and the
/// lower leaning face cross the pipe in curves drawn by both spellings at
/// once, so a face weighs each spelling apart where it asks whether the face
/// across places a node alone. What the bars and a block share is the part of
/// the bars' disc with `|y| ≤ 1.5`, from `z = 1` to `z = 4`.
#[test]
fn two_spellings_of_the_bars_pipe_beside_a_raised_step_or_climbing_top_answer_every_rule() {
    let shared =
        3.0 * 2.0 * (1.5 * (RADIUS * RADIUS - 2.25).sqrt() + RADIUS * RADIUS * 0.75f64.asin());
    let bars = PI * RADIUS * RADIUS * 2.0 * MEETING;
    let walls = [
        Wall::SpelledOnce,
        Wall::OriginSlidUp,
        Wall::AxisTurnedRound,
        Wall::TwoRoundBars,
    ];
    let mut failed = Vec::new();
    let mut ran = 0;
    for (named, profile) in [
        ("step", &A_STEP[..]),
        ("climbing top", &A_TOP_CLIMBING_IN_TWO_LEANING_FACES[..]),
    ] {
        let raised: Vec<[f64; 2]> = profile.iter().map(|&[x, z]| [x, z + 2.0]).collect();
        let block = 3.0 * area_inside(&raised);
        for wall in walls {
            for rule in EVERY_RULE {
                for bars_first in [true, false] {
                    let by_hand = match (rule, bars_first) {
                        (Rule::Union, _) => bars + block - shared,
                        (Rule::Intersection, _) => shared,
                        (Rule::Difference, true) => bars - shared,
                        (Rule::Difference, false) => block - shared,
                        _ => unreachable!(),
                    };
                    let mut model = Model::new(Scale::millimetres(100.0).expect("a size"));
                    let [lower, upper] = the_two_bars(&mut model, wall);
                    let the_block = a_block_across_y(&mut model, &raised);
                    let (arguments, tools) = if bars_first {
                        (vec![lower, upper], vec![the_block])
                    } else {
                        (vec![the_block], vec![lower, upper])
                    };
                    let asked = format!("{named} {wall:?} {rule:?} bars_first={bars_first}");
                    ran += 1;
                    if let Some(wrong) = check(&mut model, &arguments, &tools, rule, by_hand) {
                        println!("{asked}: {wrong}");
                        failed.push(asked);
                    }
                }
            }
        }
    }
    println!("ran {ran}, failed {}", failed.len());
    assert!(failed.is_empty(), "{failed:#?}");
}

/// A block drawn in `y` and `z` across `x ∈ [-3, 3]`: flat on `z = -1` from
/// `y = -1.5` to `y = 2.5`, a side leaning in up to `y = 1.5` at `z = 3`, a
/// top on `z = 3` back to `y = -0.3`, a riser down to `z = 2`, and a lower top
/// on `z = 2` back to `y = -1.5`.
const A_STEP_WITH_A_LEANING_SIDE: [[f64; 2]; 6] = [
    [-1.5, -1.0],
    [2.5, -1.0],
    [1.5, 3.0],
    [-0.3, 3.0],
    [-0.3, 2.0],
    [-1.5, 2.0],
];

/// The radius of the pipe the D-shaped hole and the D-shaped bar share, and
/// where its axis crosses `x = 0`: along `z` through `y = 0.5`.
const THE_PIPES_RADIUS: f64 = 1.5;
const THE_PIPES_AXIS_AT: f64 = 0.5;

/// The block of [`A_STEP_WITH_A_LEANING_SIDE`] with a D-shaped hole through
/// it, the part of the pipe's disc with `x ≤ -1.3`, beside a D-shaped bar on
/// the same pipe, the part of its disc with `y ≤ -0.6`, from `z = -2` to
/// `z = 4`, every rule both ways round (Q2541).
///
/// Meant as the shape of Q2541's second case: the upper top's edge with the
/// hole is an arc of the circle the pipe crosses it in, and the rest of that
/// circle runs on through the upper top, across its edge with the leaning
/// side at `x = ±√1.25` and its edge with the riser at `x = ±√2.21`. **It does
/// not reach that case yet**: a probe in `the_listed_cuts_it_still_draws`
/// found no face here that lists the pipe and draws it only along its rim.
/// What the two share is the bar's part of the disc, from `z = -1` to `z = 2`.
#[test]
fn a_rim_drawn_on_past_the_hole_beside_a_d_shaped_bar_on_its_pipe_answers_every_rule() {
    let block = 6.0 * area_inside(&A_STEP_WITH_A_LEANING_SIDE);
    let hole = 4.0 * a_slice_of_the_disc(1.3);
    let bar = 6.0 * a_slice_of_the_disc(THE_PIPES_AXIS_AT + 0.6);
    let holed = block - hole;
    let shared = 3.0 * a_slice_of_the_disc(THE_PIPES_AXIS_AT + 0.6);
    let mut failed = Vec::new();
    let mut ran = 0;
    for rule in EVERY_RULE {
        for bar_first in [true, false] {
            let by_hand = match (rule, bar_first) {
                (Rule::Union, _) => bar + holed - shared,
                (Rule::Intersection, _) => shared,
                (Rule::Difference, true) => bar - shared,
                (Rule::Difference, false) => holed - shared,
                _ => unreachable!(),
            };
            let mut model = Model::new(Scale::millimetres(100.0).expect("a size"));
            let the_block = a_block_across_x(&mut model, &A_STEP_WITH_A_LEANING_SIDE);
            let the_hole = a_part_of_the_pipes_bar(&mut model, [[-2.0, -1.3], [-1.5, 2.5]]);
            let holed_block = one_solid(&mut model, &[the_block], &[the_hole], Rule::Difference);
            let the_bar = a_part_of_the_pipes_bar(&mut model, [[-2.0, 2.0], [-1.5, -0.6]]);
            let (arguments, tools) = if bar_first {
                (vec![the_bar], vec![holed_block])
            } else {
                (vec![holed_block], vec![the_bar])
            };
            let asked = format!("{rule:?} bar_first={bar_first}");
            ran += 1;
            if let Some(wrong) = check(&mut model, &arguments, &tools, rule, by_hand) {
                println!("{asked}: {wrong}");
                failed.push(asked);
            }
        }
    }
    println!("ran {ran}, failed {}", failed.len());
    assert!(failed.is_empty(), "{failed:#?}");
}

/// A slab over `x ∈ [-3, 3]`, `y ∈ [-0.3, 1.5]`, `z ∈ [2, 3]` with the D-shaped
/// hole of [`a_rim_drawn_on_past_the_hole_beside_a_d_shaped_bar_on_its_pipe_answers_every_rule`]
/// through it, beside a bar on the same pipe from `z = -3` up to a slanting
/// cut along `y + z/2 = 0.6`, every rule both ways round (Q2541). The two
/// share nothing: the slab is wholly past the cut.
///
/// Meant as the shape of Q2541's second case. **It does not reach it**: a
/// probe in `gather` found that the slab's top and bottom do not list the
/// pipe. Their boxes run over the whole circle the hole's arc is on, `y` from
/// `-1` to `2`, so neither is strictly past the cut the bar's wall lies
/// behind. The slab's two long sides and the hole's flat side do list it.
#[test]
fn a_slab_with_a_d_shaped_hole_over_a_bar_on_its_pipe_cut_on_a_slant_answers_every_rule() {
    let bar = PI * THE_PIPES_RADIUS * THE_PIPES_RADIUS * (3.0 + 1.2 - 2.0 * THE_PIPES_AXIS_AT);
    let failed = every_rule_for_a_holed_slab_over([2.0, 3.0], a_bar_on_the_pipe_under_a_slant, bar);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The slab of
/// [`a_slab_with_a_d_shaped_hole_over_a_bar_on_its_pipe_cut_on_a_slant_answers_every_rule`],
/// from `z = 2.3`, over a wedge cut from a bar on the same pipe by
/// `z = 0.5 + t` above and `z = 0.5 - 3t` below, with `t = y - 0.5 ≥ 0` the
/// way along the pipe's radius, every rule both ways round (Q2541). The two
/// share nothing: the wedge stays below `z = 2`.
///
/// Meant as the shape of Q2541's second case. **It does not reach it**: the
/// box round the wedge's wall runs up to `z = 5` from the whole ellipse its
/// lower edge is an arc of, so the wedge is within reach of the slab, but its
/// wall's two edges are arcs and not closed curves, so the wall lies in no
/// side of a plane and no face of the slab lists the pipe.
#[test]
fn a_slab_with_a_d_shaped_hole_over_a_wedge_cut_from_a_bar_on_its_pipe_answers_every_rule() {
    let wedge = 8.0 / 3.0 * THE_PIPES_RADIUS.powi(3);
    let failed =
        every_rule_for_a_holed_slab_over([2.3, 3.0], a_wedge_cut_from_a_bar_on_the_pipe, wedge);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// What goes wrong fusing a slab over `x ∈ [-3, 3]`, `y ∈ [-0.3, 1.5]` and
/// `z`, with the part of the pipe's disc with `x ≤ -1.3` cut through it, with
/// what `bar` makes, of volume `bar_volume` and sharing nothing with the slab,
/// every rule both ways round, as the fuses' names.
fn every_rule_for_a_holed_slab_over(
    z: [f64; 2],
    bar: fn(&mut Model) -> SolidKey,
    bar_volume: f64,
) -> Vec<String> {
    let holed = (6.0 * 1.8 - a_slice_of_the_disc(1.3)) * (z[1] - z[0]);
    let mut failed = Vec::new();
    let mut ran = 0;
    for rule in EVERY_RULE {
        for bar_first in [true, false] {
            let by_hand = match (rule, bar_first) {
                (Rule::Union, _) => bar_volume + holed,
                (Rule::Intersection, _) => 0.0,
                (Rule::Difference, true) => bar_volume,
                (Rule::Difference, false) => holed,
                _ => unreachable!(),
            };
            let mut model = Model::new(Scale::millimetres(100.0).expect("a size"));
            let the_slab = a_box(&mut model, [[-3.0, 3.0], [-0.3, 1.5], z]);
            let the_hole = a_part_of_the_pipes_bar(&mut model, [[-2.0, -1.3], [-1.5, 2.5]]);
            let holed_slab = one_solid(&mut model, &[the_slab], &[the_hole], Rule::Difference);
            let the_bar = bar(&mut model);
            let (arguments, tools) = if bar_first {
                (vec![the_bar], vec![holed_slab])
            } else {
                (vec![holed_slab], vec![the_bar])
            };
            let asked = format!("{rule:?} bar_first={bar_first}");
            ran += 1;
            if let Some(wrong) = check(&mut model, &arguments, &tools, rule, by_hand) {
                println!("{asked}: {wrong}");
                failed.push(asked);
            }
        }
    }
    println!("ran {ran}, failed {}", failed.len());
    failed
}

/// A bar of radius [`THE_PIPES_RADIUS`] about the pipe's axis from `z = -5`
/// up `8`, cut to the wedge between `z = 0.5 + t` and `z = 0.5 - 3t`, with
/// `t = y - 0.5`, drawn in `y` and `z` as the triangle those two planes make
/// with `y = 3`.
fn a_wedge_cut_from_a_bar_on_the_pipe(model: &mut Model) -> SolidKey {
    let placed =
        Isometry::translation(Vector::new(0.0, THE_PIPES_AXIS_AT, -5.0)).expect("a finite shift");
    let bar = model
        .add_round_bar(THE_PIPES_RADIUS, 8.0, &placed)
        .expect("a bar");
    let the_wedge = a_block_across_x(model, &[[0.5, 0.5], [3.0, -7.0], [3.0, 3.0]]);
    one_solid(model, &[bar], &[the_wedge], Rule::Intersection)
}

/// A bar of radius [`THE_PIPES_RADIUS`] about the pipe's axis from `z = -3`
/// up to where it crosses `y + z/2 = 0.6`, which is `z = 1.2 - 2y`.
fn a_bar_on_the_pipe_under_a_slant(model: &mut Model) -> SolidKey {
    let placed =
        Isometry::translation(Vector::new(0.0, THE_PIPES_AXIS_AT, -3.0)).expect("a finite shift");
    let bar = model
        .add_round_bar(THE_PIPES_RADIUS, 7.0, &placed)
        .expect("a bar");
    let under_the_slant = a_block_across_x(
        model,
        &[[-2.0, -4.0], [2.6, -4.0], [-1.9, 5.0], [-2.0, 5.0]],
    );
    one_solid(model, &[bar], &[under_the_slant], Rule::Intersection)
}

/// The area of the part of the pipe's disc further than `apart` from its
/// centre along one direction.
fn a_slice_of_the_disc(apart: f64) -> f64 {
    let r = THE_PIPES_RADIUS;
    r * r * (apart / r).acos() - apart * (r * r - apart * apart).sqrt()
}

/// `profile`, drawn in `y` and `z`, pushed across `x ∈ [-3, 3]`.
fn a_block_across_x(model: &mut Model, profile: &[[f64; 2]]) -> SolidKey {
    let profile = ProfileBuilder::new(profile.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a profile");
    let about_x = Isometry::quarter_turns(&Axis::x(), 1).expect("a quarter turn");
    let about_z = Isometry::quarter_turns(&Axis::z(), 1).expect("a quarter turn");
    let back = Isometry::translation(Vector::new(-3.0, 0.0, 0.0)).expect("a finite shift");
    model
        .add_extrusion(&profile, 6.0, &back.compose(&about_z.compose(&about_x)))
        .expect("a positive height")
}

/// The part over `x` and `y` of a bar of radius [`THE_PIPES_RADIUS`] about
/// the pipe's axis from `z = -2` to `z = 4`.
fn a_part_of_the_pipes_bar(model: &mut Model, [x, y]: [[f64; 2]; 2]) -> SolidKey {
    let placed =
        Isometry::translation(Vector::new(0.0, THE_PIPES_AXIS_AT, -2.0)).expect("a finite shift");
    let bar = model
        .add_round_bar(THE_PIPES_RADIUS, 6.0, &placed)
        .expect("a bar");
    let the_box = a_box(model, [x, y, [-3.0, 5.0]]);
    one_solid(model, &[bar], &[the_box], Rule::Intersection)
}

/// The one solid fusing `arguments` and `tools` by `rule` gives.
fn one_solid(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> SolidKey {
    let fused = model
        .fuse(
            arguments,
            tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a fuse that makes the shape");
    let [one] = fused.solids() else {
        panic!("{} solids where one was meant", fused.solids().len());
    };
    *one
}

/// What goes wrong fusing a bar of radius 2 on `z ∈ [0, 2]` with the block
/// of `profile` across `y ∈ [-1.5, 1.5]`, every rule both ways round, as the
/// fuses' names: the block covers `z ∈ [-1, 2.2]` over `x ∈ [-3, 3]` at
/// least, so the bar is `8π`, the block `3` times the profile's area, and
/// what they share is the part of the bar's disc with `|y| ≤ 1.5`,
/// `2 · 2 · (1.5 · √1.75 + 4 · asin 0.75)`.
fn every_rule_beside_the_bar(profile: &[[f64; 2]]) -> Vec<String> {
    let bar = PI * RADIUS * RADIUS * MEETING;
    let block = 3.0 * area_inside(profile);
    let shared = MEETING
        * 2.0
        * (1.5 * (RADIUS * RADIUS - 2.25).sqrt() + RADIUS * RADIUS * (0.75f64).asin());
    let mut failed = Vec::new();
    let mut ran = 0;
    for rule in EVERY_RULE {
        for bar_first in [true, false] {
            let by_hand = match (rule, bar_first) {
                (Rule::Union, _) => bar + block - shared,
                (Rule::Intersection, _) => shared,
                (Rule::Difference, true) => bar - shared,
                (Rule::Difference, false) => block - shared,
                _ => unreachable!(),
            };
            let mut model = Model::new(Scale::millimetres(100.0).expect("a size"));
            let the_bar = model
                .add_round_bar(RADIUS, MEETING, &Isometry::identity())
                .expect("a bar");
            let the_block = a_block_across_y(&mut model, profile);
            let (arguments, tools) = if bar_first {
                (vec![the_bar], vec![the_block])
            } else {
                (vec![the_block], vec![the_bar])
            };
            let asked = format!("{rule:?} bar_first={bar_first}");
            ran += 1;
            if let Some(wrong) = check(&mut model, &arguments, &tools, rule, by_hand) {
                println!("{asked}: {wrong}");
                failed.push(asked);
            }
        }
    }
    println!("ran {ran}, failed {}", failed.len());
    failed
}

/// The area inside `profile`, a polygon going round anticlockwise.
fn area_inside(profile: &[[f64; 2]]) -> f64 {
    let mut twice = 0.0;
    for (at, here) in profile.iter().enumerate() {
        let next = profile[(at + 1) % profile.len()];
        twice += here[0] * next[1] - next[0] * here[1];
    }
    twice / 2.0
}

/// `profile`, drawn in `x` and `z`, pushed across `y ∈ [-1.5, 1.5]`.
fn a_block_across_y(model: &mut Model, profile: &[[f64; 2]]) -> SolidKey {
    let profile = ProfileBuilder::new(profile.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a profile");
    let about_x = Isometry::quarter_turns(&Axis::x(), 1).expect("a quarter turn");
    let out = Isometry::translation(Vector::new(0.0, 1.5, 0.0)).expect("a finite shift");
    model
        .add_extrusion(&profile, 3.0, &out.compose(&about_x))
        .expect("a positive height")
}

/// What is wrong with fusing `arguments` and `tools` by `rule`: a refusal, a
/// result that does not validate, or a volume further from `by_hand` than the
/// measure's own bound. `None` when nothing is.
fn check(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
    by_hand: f64,
) -> Option<String> {
    let policy = Policy::certified();
    let fused = match model.fuse(arguments, tools, rule, &policy, &mut ProvenanceLog::new()) {
        Ok(fused) => fused.solids().to_vec(),
        Err(refused) => return Some(format!("refused {refused:?}")),
    };
    if let Err(broken) = model.validate(&policy, &mut ProvenanceLog::new()) {
        return Some(format!("does not validate {broken:?}"));
    }
    let (mut volume, mut bound) = (0.0, 0.0);
    for solid in fused {
        let measured = match model.solid(solid).expect("a key").measure(
            Deflection::of(0.01).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        ) {
            Ok(measured) => measured,
            Err(refused) => return Some(format!("measure refused {refused:?}")),
        };
        volume += measured.volume();
        bound += measured.volume_error();
    }
    ((volume - by_hand).abs() > bound).then(|| format!("{volume} ± {bound} against {by_hand}"))
}

/// Two bars of radius 2 on one pipe, one on `z ∈ [0, 2]` and one on
/// `z ∈ [2, 4]`, the upper one's wall written as `wall` says.
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

/// A bar of radius 2 about the `z` axis from `bottom` up `2`, described face by
/// face, its wall's axis starting at `z = axis_at` and pointing down where
/// `turned_round`.
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

/// The box over `x`, `y` and `z`, each a low and a high end.
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
