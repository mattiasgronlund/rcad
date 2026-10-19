//! A surface the face across leaves out, and an edge both faces must split
//! alike (Q2364, `docs/decisions.md` §422).
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

/// A bar of radius 2 standing on `z ∈ [0, 2]` and the step of [`a_step`],
/// every rule both ways round, against the closed forms: the step is `76.5`,
/// the bar `8π`, and what they share is the part of the bar's disc with
/// `|y| ≤ 1.5`, `2 · 2 · (1.5 · √1.75 + 4 · asin 0.75)`.
#[test]
fn a_step_beside_a_bar_answers_every_rule_either_way_round() {
    let bar = PI * RADIUS * RADIUS * MEETING;
    let step = 3.0 * (6.0 * 4.0 + 1.5 * 1.0);
    let shared = MEETING
        * 2.0
        * (1.5 * (RADIUS * RADIUS - 2.25).sqrt() + RADIUS * RADIUS * (0.75f64).asin());
    let mut failed = Vec::new();
    let mut ran = 0;
    for rule in EVERY_RULE {
        for bar_first in [true, false] {
            let by_hand = match (rule, bar_first) {
                (Rule::Union, _) => bar + step - shared,
                (Rule::Intersection, _) => shared,
                (Rule::Difference, true) => bar - shared,
                (Rule::Difference, false) => step - shared,
                _ => unreachable!(),
            };
            let mut model = Model::new(Scale::millimetres(100.0).expect("a size"));
            let the_bar = model
                .add_round_bar(RADIUS, MEETING, &Isometry::identity())
                .expect("a bar");
            let the_step = a_step(&mut model);
            let (arguments, tools) = if bar_first {
                (vec![the_bar], vec![the_step])
            } else {
                (vec![the_step], vec![the_bar])
            };
            let asked = format!("step {rule:?} bar_first={bar_first}");
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

/// A block over `x ∈ [-3, 3]`, `y ∈ [-1.5, 1.5]`, `z ∈ [-1, 3]`, with a step
/// over `x ∈ [1.5, 3]` up to `z = 4`.
fn a_step(model: &mut Model) -> SolidKey {
    let profile = ProfileBuilder::new(vec![
        [-3.0, -1.0],
        [3.0, -1.0],
        [3.0, 4.0],
        [1.5, 4.0],
        [1.5, 3.0],
        [-3.0, 3.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a step");
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
