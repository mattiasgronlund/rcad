//! A plane leaning through a bar's rim start: `z = y` through `(5, 0, 0)`.
//!
//! `add_round_bar` starts its bottom cap circle at `(5, 0, 0)`. The plane
//! `z = y` cuts the bottom cap along the diameter from `(−5, 0, 0)` to
//! `(5, 0, 0)` and the wall in a half ellipse up to `(0, 5, 5)`, so it passes
//! through the rim start, and the meet of the cap plane, the wall and the
//! leaning plane is that very point. §252.8 found all six fuses refusing
//! `Undecidable` on "order along a crossing" there, and `docs/decisions.md`
//! §284 is what this file measured.
//!
//! **That refusal belongs to a block turned to lean, and the turn misses the
//! rim start.** Turning a block by `−π/4` about `x` leaves its face on
//! `−0.7071067811865475·y + 0.7071067811865476·z = −2⁻⁴⁹`, which crosses the
//! cap plane at `y = 2.512e-15` and not at `y = 0`. So the meet is a second
//! point on the cap circle, `2.5e-15` round from the rim start, in a box `±3`
//! ulps wide along `x`. Going round the cap circle from `(0, −5, 0)`, the two
//! cannot be put in order from that box, although the exact answer is a sweep
//! of `−1.26e-14`. That was a **gap**, and all six fuses refused on it. Since
//! Q209 the order asks a box one checked step narrower, `[5 − 1 ulp, 5]` along
//! `x`, which decides it, and all six answer under both policies.
//!
//! **A plane exactly on `z = y` is ordered, and all six fuses answer.** Built
//! face by face so its face is `z = y` to the bit, the meet is `(5, 0, 0)`
//! exactly. The union and both differences then keep an arc of the cap circle
//! and an arc of the ellipse that join the same two corners on the one wall.
//! Until §330 named such an arc by its plane as well as by the wall, the two
//! came out as one edge with four faces along it, and those four fuses refused
//! `ResultIsNotManifold { count: 4 }`. All six are measured now.
//!
//! **A cube stood on its corner with a face through the bar's top rim start
//! reaches the fold the turned block no longer does.** On the faces that have
//! the rim start, the meet of that face, the cap plane and the wall is the
//! rim start, within what writing it down cost, and makes no node; the cube's
//! face has no rim start and keeps the meet. Until the arrangements handed on
//! the names they folded (§284.8), the answer got two corners there and all
//! six fuses refused `ResultIsNotManifold { count: 1 }`.
//!
//! Every fuse here has its volume worked out by hand: measured against it where
//! the fuse answers, and carried in the message where it refuses. The bar is
//! `π·5²·10`. The plane cuts it into a wedge, the part with `0 ≤ z ≤ y`, whose
//! volume is `∫∫ y dA` over the half disc `y ≥ 0`, which is `2·5³/3 = 250/3`.
//! The top cap plays no part, because `z ≤ y ≤ 5 < 10`. The block is a cube
//! forty on a side, and the prism is the half of one that `z = y` leaves, with
//! the whole bar inside it.

use std::f64::consts::PI;

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftStep, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;
const HEIGHT: f64 = 10.0;
/// Half the side of the turned block and of the prism, far enough out that
/// their only face near the bar is the leaning one.
const REACH: f64 = 20.0;
const THE_WEDGE: f64 = 2.0 * RADIUS * RADIUS * RADIUS / 3.0;
const A_CUBE_FORTY_ON_A_SIDE: f64 = (2.0 * REACH) * (2.0 * REACH) * (2.0 * REACH);
/// A right triangle with legs of 40, forty long: `z = y` through the cube's
/// middle cuts it in half.
const A_PRISM_UNDER_Z_EQUALS_Y: f64 = A_CUBE_FORTY_ON_A_SIDE / 2.0;

/// **The six fuses of §252.8, each one solid of the volume worked out by
/// hand, under `Policy::certified()` and with nothing in the log.**
///
/// The face that refused them all until Q209 is the bar's bottom cap, `z = 0`,
/// and the crossing is the cap circle, where that plane meets the wall. On it
/// lie the meet of the cap plane, the wall and the turned face, round
/// `(5 − 6.3e-31, 2.512e-15, 0)`, and the rim start `(5, 0, 0)` itself. They
/// are two points: the meet's `y` side is one ulp wide and above zero. The
/// order round the circle from `(0, −5, 0)` is the sign of a sweep whose exact
/// value is `−1.26e-14`. The box the quadratic along the two planes' line puts
/// round the meet is `[5 − 3 ulps, 5 + 3 ulps]` along `x`, and over it the
/// sweep straddles zero with an enclosure of `3.55e-14`. That refused all six.
///
/// One checked step onto the wall narrows that side to `[5 − 1 ulp, 5]`
/// (`MeetOfThree::boxed_tight`), and over it the sweep is proved negative.
/// The order round the circle asks the narrowed box only where the meet's own
/// box cannot say. The meet then stays a node of its own, `2.5e-15` round from
/// the rim start, on the cap, on the wall and on the turned face alike, and
/// each fuse comes back as one solid.
#[test]
fn a_block_turned_to_lean_through_a_bars_rim_start_answers_all_six_fuses_with_one_solid_each() {
    let policy = Policy::certified();
    let (model, bar, block) = a_bar_and(a_block_turned_to_lean_along_z_equals_y);

    for fuse in the_six_fuses(bar, block, A_CUBE_FORTY_ON_A_SIDE) {
        let log = one_solid_measured_right(&model, fuse, &policy);
        assert!(
            log.records().is_empty(),
            "{:?}: a certified fuse logs nothing, not {log:?}",
            fuse.0
        );
    }
}

/// **Under a tolerant policy the six answer the same way**, and for the same
/// reason: the order round the cap circle is proved, so nothing is left for the
/// policy to read, and the log stays empty.
///
/// Before Q209 the policy was asked. The enclosure of `3.55e-14` is a volume,
/// so its wobble at a working size of 100 is `3.55e-18`, against a budget of
/// `64·ε·100 = 1.42e-12`, and the policy read it as zero. The meet was folded
/// into the rim start on the cap and on the wall, while the turned face, which
/// has no rim start to fold it into, kept it as a corner of its own (§284.7).
/// The faces then disagreed on where their edges end, and every fuse was
/// refused `ResultIsNotManifold { count: 1 }`. With the order proved, the meet
/// is a node on all three faces and they agree.
///
/// This pair no longer reaches the fold §284.7 found. Since §284.8 a meet
/// folded into an input corner is handed on with the corner, so the faces
/// would agree even if it did, and
/// `a_cube_on_its_corner_leaning_through_a_bars_top_rim_start_answers_all_six_fuses`
/// is a shape that does reach it, under a certified policy.
#[test]
fn under_a_tolerant_policy_the_turned_block_answers_all_six_fuses_with_one_solid_each_too() {
    let (model, bar, block) = a_bar_and(a_block_turned_to_lean_along_z_equals_y);
    let policy = Policy::tolerant(model.scale().nominal_size());

    for fuse in the_six_fuses(bar, block, A_CUBE_FORTY_ON_A_SIDE) {
        let log = one_solid_measured_right(&model, fuse, &policy);
        assert!(
            log.records().is_empty(),
            "{:?}: nothing is left for the policy to read, not {log:?}",
            fuse.0
        );
    }
}

/// **A plane exactly on `z = y` puts the two points in order, and all six
/// fuses answer and measure right under both policies.**
///
/// The meet is the rim start to the bit, so nothing is left to order. The cap
/// circle and the ellipse both run from `(5, 0, 0)` to `(−5, 0, 0)` along the
/// wall, and the union and both differences keep an arc of each. The two arcs
/// share their corners and their wall, and only their planes, `z = 0` and
/// `z = y`, tell them apart. Since §330 an arc's name carries its plane, so
/// they are two edges; before, they were one with four faces along it, and
/// those four fuses refused `ResultIsNotManifold { count: 4 }`. The
/// intersections keep only one of the arcs, and answered before too.
#[test]
fn a_plane_exactly_through_a_bars_rim_start_answers_all_six_fuses_right_under_both_policies() {
    let (model, bar, prism) = a_bar_and(a_prism_under_z_equals_y);

    for policy in [
        Policy::certified(),
        Policy::tolerant(model.scale().nominal_size()),
    ] {
        for fuse in the_six_fuses(bar, prism, A_PRISM_UNDER_Z_EQUALS_Y) {
            one_solid_measured_right(&model, fuse, &policy);
        }
    }
}

/// **A cube stood on its corner, with a face leaning through the top rim start
/// of a bar, answers all six fuses under `Policy::certified()`.**
///
/// The bar has radius 4 and stands from `z = 3` to `z = 8`, so its top cap
/// circle starts at `(4, 0, 8)`. The cube is eight on a side, stood on its
/// corner at `(0, 0, 4)`, and one of its faces passes through the rim start
/// to within rounding: the meet of that face, the top cap plane and the wall
/// comes out at `(4 + 1 ulp, −3.8e-16, 8)`. That is inside what writing the
/// rim start down cost, so on the cap and on the wall the meet is the rim
/// start and makes no node of its own. The cube's face has no rim start, keeps
/// the meet as a node, and names it by its three surfaces.
///
/// Until the arrangements handed on what they folded (§284.8), the answer got
/// both, the rim start by its vertex and the meet by its three surfaces, a few
/// ulps apart, and all six fuses refused `ResultIsNotManifold { count: 1 }`.
/// It is the fifteenth draw of the corner-up cubes that
/// `a_bar_answers_every_corner_up_cube_so_what_a_frustum_refuses_there_is_the_cones`
/// stands against a bar, which answers every one of its 96 fuses there, so
/// these six are the frustum's alone.
///
/// Nothing here is worked out by hand. The volumes are held to one another:
/// union and intersection make up the two solids, and each difference is its
/// solid less the intersection, within the measured errors.
#[test]
fn a_cube_on_its_corner_leaning_through_a_bars_top_rim_start_answers_all_six_fuses() {
    let policy = Policy::certified();
    let (model, bar, cube) = a_bar_and_a_cube_on_its_corner();
    let one_bar = PI * 4.0 * 4.0 * 5.0;
    let one_cube = 8.0 * 8.0 * 8.0;

    let measured = |rule, arguments, tools| {
        let (volume, error, log) = one_solid_measured(&model, rule, arguments, tools, &policy);
        assert!(
            log.records().is_empty(),
            "{rule:?}: a certified fuse logs nothing, not {log:?}"
        );
        (volume, error)
    };
    let bar_less = measured(Rule::Difference, bar, cube);
    let cube_less = measured(Rule::Difference, cube, bar);
    let within = |what: &str, sum: f64, error: f64, of: f64| {
        assert!(
            (sum - of).abs() <= error,
            "{what}: {sum} ± {error} against {of}"
        );
    };
    for (one, other) in [(bar, cube), (cube, bar)] {
        let union = measured(Rule::Union, one, other);
        let both = measured(Rule::Intersection, one, other);
        within(
            "union and intersection make up the bar and the cube",
            union.0 + both.0,
            union.1 + both.1,
            one_bar + one_cube,
        );
        within(
            "the bar less the cube, and the intersection, make up the bar",
            bar_less.0 + both.0,
            bar_less.1 + both.1,
            one_bar,
        );
        within(
            "the cube less the bar, and the intersection, make up the cube",
            cube_less.0 + both.0,
            cube_less.1 + both.1,
            one_cube,
        );
        assert!(
            both.0 > both.1,
            "the two overlap, or this measures nothing: {both:?}"
        );
    }
}

/// One of the six fuses on a copy of `model`, held to one solid that
/// validates and measures within its own `volume_error` of the volume by hand.
/// Hands back what the fuse logged.
fn one_solid_measured_right(
    model: &Model,
    (rule, arguments, tools, by_hand): (Rule, SolidKey, SolidKey, f64),
    policy: &Policy,
) -> ProvenanceLog {
    let (volume, error, log) = one_solid_measured(model, rule, arguments, tools, policy);
    println!("{rule:?} under {policy:?}: measured {volume} ± {error} against {by_hand} by hand");
    assert!(
        (volume - by_hand).abs() <= error,
        "{rule:?}: measured {volume} ± {error} against {by_hand} by hand (§284.4)"
    );
    log
}

/// One fuse on a copy of `model`, held to one solid that validates, with its
/// volume, its `volume_error` and what the fuse logged.
fn one_solid_measured(
    model: &Model,
    rule: Rule,
    arguments: SolidKey,
    tools: SolidKey,
    policy: &Policy,
) -> (f64, f64, ProvenanceLog) {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(&[arguments], &[tools], rule, policy, &mut log)
        .unwrap_or_else(|refused| panic!("{rule:?} is one solid: {refused:?}"));
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|broken| panic!("{rule:?} came back broken: {broken:?}"));
    let [solid] = fused.solids() else {
        panic!("{rule:?} is one solid: {:?}", fused.solids());
    };
    let measured = model
        .solid(*solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a solid measures");
    (measured.volume(), measured.volume_error(), log)
}

/// A bar of radius 4 from `z = 3` to `z = 8`, and a cube eight on a side stood
/// on its corner at `(0, 0, 4)`: turned about the line through the origin
/// along `(1, −1, 0)` until its long diagonal stands upright, as
/// `how_often_a_fuse_with_a_cone_refuses.rs` stands its cubes.
fn a_bar_and_a_cube_on_its_corner() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(
            4.0,
            5.0,
            &Isometry::translation(Vector::new(0.0, 0.0, 3.0)).expect("a finite shift"),
        )
        .expect("a bar of radius four, five tall");
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let cube = model
        .add_extrusion(&square, 8.0, &Isometry::identity())
        .expect("a cube");
    let diagonal_up = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(1.0, -1.0, 0.0)).expect("a direction"),
        (1.0f64 / 3.0).sqrt().acos(),
    )
    .expect("a finite turn");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(0.0, 0.0, 4.0))
                .expect("a finite shift")
                .compose(&diagonal_up),
        )
        .expect("this model made the key");
    (model, bar, cube)
}

fn a_bar_and(knife: fn(&mut Model) -> SolidKey) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar of radius five, ten tall");
    let knife = knife(&mut model);
    (model, bar, knife)
}

/// Union, intersection and difference, each both ways round, with the volume
/// of each answer worked out by hand. Every answer is one solid.
fn the_six_fuses(
    bar: SolidKey,
    knife: SolidKey,
    knife_volume: f64,
) -> [(Rule, SolidKey, SolidKey, f64); 6] {
    let one_bar = PI * RADIUS * RADIUS * HEIGHT;
    let together = knife_volume + one_bar - THE_WEDGE;
    [
        (Rule::Union, bar, knife, together),
        (Rule::Union, knife, bar, together),
        (Rule::Intersection, bar, knife, THE_WEDGE),
        (Rule::Intersection, knife, bar, THE_WEDGE),
        (Rule::Difference, bar, knife, one_bar - THE_WEDGE),
        (Rule::Difference, knife, bar, knife_volume - THE_WEDGE),
    ]
}

/// A cube forty on a side, `[−20, 20] × [0, 40] × [−20, 20]`, turned by `−π/4`
/// about `x`. Its face `y = 0` then leans along `z = y` to within the turn's
/// rounding, and the rest of it is far from the bar.
fn a_block_turned_to_lean_along_z_equals_y(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![
        [-REACH, 0.0],
        [REACH, 0.0],
        [REACH, 2.0 * REACH],
        [-REACH, 2.0 * REACH],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let lean = Isometry::rotation(&Axis::x(), -std::f64::consts::FRAC_PI_4)
        .expect("a finite turn")
        .compose(&Isometry::translation(Vector::new(0.0, 0.0, -REACH)).expect("a finite shift"));
    model
        .add_extrusion(&square, 2.0 * REACH, &lean)
        .expect("a block")
}

/// The part of `[−20, 20]³` with `z ≤ y`, a prism whose end is a right
/// triangle, described face by face so that its leaning face is `z = y` to the
/// bit: the plane through the origin along `(0, −1, 1)`, whose normal has two
/// equal components.
fn a_prism_under_z_equals_y(model: &mut Model) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let mut corner = |x: f64, y: f64, z: f64| {
        builder
            .add_vertex(Point::new(x, y, z))
            .expect("a finite corner")
    };
    let low_left = corner(-REACH, -REACH, -REACH);
    let high_left = corner(-REACH, REACH, REACH);
    let back_left = corner(-REACH, REACH, -REACH);
    let low_right = corner(REACH, -REACH, -REACH);
    let high_right = corner(REACH, REACH, REACH);
    let back_right = corner(REACH, REACH, -REACH);

    let mut edge = |start, end| {
        builder
            .add_edge(start, end, EdgeRun::Straight)
            .expect("a straight edge")
    };
    let low = edge(low_left, low_right);
    let high = edge(high_left, high_right);
    let back = edge(back_left, back_right);
    let left_up = edge(low_left, high_left);
    let left_down = edge(high_left, back_left);
    let left_across = edge(back_left, low_left);
    let right_up = edge(low_right, high_right);
    let right_down = edge(high_right, back_right);
    let right_across = edge(back_right, low_right);

    use Orientation::{Forward, Reversed};
    let leaning = a_flat_face(
        &mut builder,
        Point::origin(),
        Vector::new(0.0, -1.0, 1.0),
        &[
            (low, Forward),
            (right_up, Forward),
            (high, Reversed),
            (left_up, Reversed),
        ],
    );
    let behind = a_flat_face(
        &mut builder,
        Point::new(0.0, REACH, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        &[
            (left_down, Reversed),
            (high, Forward),
            (right_down, Forward),
            (back, Reversed),
        ],
    );
    let under = a_flat_face(
        &mut builder,
        Point::new(0.0, 0.0, -REACH),
        Vector::new(0.0, 0.0, -1.0),
        &[
            (left_across, Reversed),
            (back, Forward),
            (right_across, Forward),
            (low, Reversed),
        ],
    );
    let left = a_flat_face(
        &mut builder,
        Point::new(-REACH, 0.0, 0.0),
        Vector::new(-1.0, 0.0, 0.0),
        &[
            (left_up, Forward),
            (left_down, Forward),
            (left_across, Forward),
        ],
    );
    let right = a_flat_face(
        &mut builder,
        Point::new(REACH, 0.0, 0.0),
        Vector::new(1.0, 0.0, 0.0),
        &[
            (right_across, Reversed),
            (right_down, Reversed),
            (right_up, Reversed),
        ],
    );
    builder
        .add_shell(&[
            (leaning, Forward),
            (behind, Forward),
            (under, Forward),
            (left, Forward),
            (right, Forward),
        ])
        .expect("one shell");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a prism that validates")
}

fn a_flat_face(
    builder: &mut SolidBuilder,
    through: Point,
    outward: Vector,
    walk: &[(DraftEdge, Orientation)],
) -> rcad_topo::DraftFace {
    let steps: Vec<DraftStep> = walk
        .iter()
        .map(|&(edge, turn)| DraftStep::Along(builder.add_coedge(edge, turn).expect("a coedge")))
        .collect();
    let wire = builder.add_wire(&steps).expect("a closed wire");
    let plane = Plane::from_point_and_normal(through, outward).expect("a plane");
    builder
        .add_face(Surface::Plane(plane), &[wire])
        .expect("a flat face")
}
