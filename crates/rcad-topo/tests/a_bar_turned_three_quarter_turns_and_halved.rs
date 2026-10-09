//! A round bar turned three quarter turns about its own axis, then halved
//! through that axis by a block's face, answers every fuse the way the same bar
//! unturned does. Q1504 found it refused and pinned the refusal; Q1522 made it
//! answer (`docs/decisions.md` §352.3).
//!
//! **The shape** is the one a probe in Q1305's relay found and did not keep.
//! The bar has radius `11/2` and height `5/2`, and is placed with
//! `lift(0, 0, −2) ∘ quarter_turns(z, 3)`, so it stands from `z = −2` to
//! `z = 1/2`. The block is `[−6, 6] × [0, 6] × [−3, 1]`, and its face `y = 0`
//! cuts the bar through its axis.
//!
//! **What it holds, by hand.** Half the bar is `π r² h / 2 = 605π/16` and the
//! block is `12 · 6 · 4 = 288`. The block holds the half of the bar on `y ≥ 0`
//! and misses the rest, so the union is `288 + 605π/16`, the intersection
//! and the bar less the block are `605π/16`, and the block less the bar is
//! `288 − 605π/16`. A measured volume is right when it is within the mesh's own
//! `volume_error` of that.
//!
//! **What happens.** Turned by any number of quarter turns, every fuse answers
//! under both policies.
//!
//! **What was wrong before Q1522.** Turned one or three quarter turns, every
//! certified fuse was refused *order along a crossing* round a cap circle, and
//! every tolerant one as two curves touching on the bar's wall. A quarter turn
//! rounds nothing: the turned seam corner `(0, −11/2, −2)` and the turned cap
//! and wall come out exact. But the fuse reads that corner through
//! `rcad_geom::where_a_corner_lands_after_moving`, which then handed back an
//! exact point only for a move that was a shift, and boxed every other one
//! `± CORNER_AFTER_MOVING_ROUNDOFF · 11/2 = ± 1.56e-13`. The cap circle starts
//! at exactly that corner, so the start was inside the corner's box, and the
//! order round the circle cannot be read from a sweep that starts in a box:
//! Q885's case, with the box made by a move that did not need one. Now a turn
//! of a cube, then a shift, that round nothing hand the corner back exact
//! ([`the_turned_bars_seam_corner_is_read_exactly_where_its_cap_circle_starts`]),
//! and both refusals went with the box.
//!
//! **Why it was a gap and not a limit** (§16.10). A bar built in place, with
//! the same three surfaces bit for bit and its seam corner where the turn puts
//! it, was read exactly and answered every fuse while the turned bar was
//! refused
//! ([`a_bar_built_with_its_seam_where_the_turn_puts_it_answers_every_fuse_under_both_policies`]).

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface,
    SurfaceMeet, Vector, intersect, where_a_closed_crossing_starts,
    where_a_corner_lands_after_moving,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.5;
const HEIGHT: f64 = 2.5;
const LIFT: f64 = -2.0;
const RULES: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// A cap circle of the bar, and where a turn of an odd number of quarters puts
/// its seam corner.
#[derive(Clone, Copy, Debug)]
struct ACapCircle {
    quarters: i32,
    /// Where the bar stores its corner on this circle, before it is placed.
    stored_corner: [f64; 3],
    /// Which way out of the bar the cap faces.
    outward: f64,
    /// Where the placed bar's corner is, and where the circle starts.
    lands: [f64; 3],
}

const THE_BOTTOM_CIRCLE_TURNED_THREE_QUARTERS: ACapCircle = ACapCircle {
    quarters: 3,
    stored_corner: [RADIUS, 0.0, 0.0],
    outward: -1.0,
    lands: [0.0, -RADIUS, LIFT],
};

const THE_TOP_CIRCLE_TURNED_ONE_QUARTER: ACapCircle = ACapCircle {
    quarters: 1,
    stored_corner: [RADIUS, 0.0, HEIGHT],
    outward: 1.0,
    lands: [0.0, RADIUS, LIFT + HEIGHT],
};

const BOTH_CIRCLES: [ACapCircle; 2] = [
    THE_BOTTOM_CIRCLE_TURNED_THREE_QUARTERS,
    THE_TOP_CIRCLE_TURNED_ONE_QUARTER,
];

/// One and three quarter turns were refused under both policies before Q1522;
/// none and two answered.
#[test]
fn a_bar_turned_any_number_of_quarter_turns_and_halved_answers_every_fuse_under_both_policies() {
    for quarters in 0..4 {
        every_fuse_answers_with_what_it_holds_by_hand(
            |model| a_bar_turned(model, quarters),
            &format!("{quarters} quarter turns"),
        );
    }
}

/// The turn rounds nothing: the corner and both surfaces round it land exactly
/// where they belong, `where_a_corner_lands_after_moving` hands the corner back
/// as that exact point, and the cap circle starts on it.
#[test]
fn the_turned_bars_seam_corner_is_read_exactly_where_its_cap_circle_starts() {
    let wall = Surface::Cylinder(Cylinder::new(Axis::z(), RADIUS).expect("a positive radius"));
    let moved_wall = the_wall_where_the_turn_puts_it();
    for circle in BOTH_CIRCLES {
        let placement = placement(circle.quarters);
        let [x, y, z] = circle.stored_corner;
        let [lx, ly, lz] = circle.lands;
        assert_eq!(
            placement.apply_to_point(Point::new(x, y, z)),
            Point::new(lx, ly, lz)
        );
        let moved_cap = a_cap(lz, circle.outward);
        assert_eq!(a_cap(z, circle.outward).transformed(&placement), moved_cap);
        assert_eq!(wall.transformed(&placement), moved_wall);

        let (seam_corner, wobble) = where_its_seam_corner_lands(circle);
        assert_eq!(wobble, 0.0, "{circle:?}");
        assert_eq!(
            seam_corner,
            PredicatePoint::from(circle.lands),
            "{circle:?}"
        );

        let SurfaceMeet::Curves(runs) = intersect(
            &moved_cap,
            &moved_wall,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a cap across a pipe is named") else {
            panic!("a cap square to a pipe cuts it in a circle");
        };
        let [circle_run] = runs.as_slice() else {
            panic!("a cap square to a pipe cuts it in one circle, got {runs:?}");
        };
        let start = where_a_closed_crossing_starts(
            circle_run,
            [&moved_cap, &moved_wall],
            &mut ProvenanceLog::new(),
        )
        .expect("a circle has a start")
        .coordinates();
        assert_eq!(start, circle.lands, "{circle:?}");
    }
}

/// The control. This bar has the turned bar's three surfaces, bit for bit and
/// in the same order, and its seam corner where the turn puts it; it is built
/// there rather than moved there, so the fuse reads its corner exactly however
/// it reads a moved one. It answered every fuse while the turned bar was
/// refused, which showed the answer existed.
#[test]
fn a_bar_built_with_its_seam_where_the_turn_puts_it_answers_every_fuse_under_both_policies() {
    let mut model = a_model();
    let turned = a_bar_turned(&mut model, THE_BOTTOM_CIRCLE_TURNED_THREE_QUARTERS.quarters);
    let built = a_bar_built_with_its_seam_where_the_turn_puts_it(&mut model);
    let surfaces = |solid: SolidKey| -> Vec<Surface> {
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .map(|face| face.surface())
            .collect()
    };
    assert_eq!(surfaces(built), surfaces(turned));

    every_fuse_answers_with_what_it_holds_by_hand(
        a_bar_built_with_its_seam_where_the_turn_puts_it,
        "built in place",
    );
}

/// Every rule, either way round, under `Policy::certified()` and a tolerant
/// policy, gives one solid of the volume worked out by hand and logs nothing.
fn every_fuse_answers_with_what_it_holds_by_hand(
    add_the_bar: impl Fn(&mut Model) -> SolidKey,
    which_bar: &str,
) {
    for tolerant in [false, true] {
        for rule in RULES {
            for bar_first in [true, false] {
                let mut model = a_model();
                let policy = if tolerant {
                    Policy::tolerant(model.scale().nominal_size())
                } else {
                    Policy::certified()
                };
                let bar = add_the_bar(&mut model);
                let block = the_block(&mut model);
                let (fused, log) = fuse(&mut model, bar, block, rule, bar_first, &policy);
                let what =
                    format!("{which_bar}, tolerant {tolerant}, {rule:?}, bar first {bar_first}");
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
                let by_hand = what_it_holds_by_hand(rule, bar_first);
                assert!(
                    (measured.volume() - by_hand).abs() <= measured.volume_error(),
                    "{what}: measured {} ± {}, by hand {by_hand}",
                    measured.volume(),
                    measured.volume_error()
                );
                assert!(log.is_empty(), "{what}: logged {log:?}");
            }
        }
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn a_bar_turned(model: &mut Model, quarters: i32) -> SolidKey {
    model
        .add_round_bar(RADIUS, HEIGHT, &placement(quarters))
        .expect("a bar")
}

fn placement(quarters: i32) -> Isometry {
    let lift = Isometry::translation(Vector::new(0.0, 0.0, LIFT)).expect("a finite shift");
    lift.compose(&Isometry::quarter_turns(&Axis::z(), quarters).expect("a turn about z"))
}

/// `[−6, 6] × [0, 6] × [−3, 1]`.
fn the_block(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![[-6.0, 0.0], [6.0, 0.0], [6.0, 6.0], [-6.0, 6.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    model
        .add_extrusion(
            &square,
            4.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -3.0)).expect("a finite shift"),
        )
        .expect("a block")
}

fn fuse(
    model: &mut Model,
    bar: SolidKey,
    block: SolidKey,
    rule: Rule,
    bar_first: bool,
    policy: &Policy,
) -> (Result<Vec<SolidKey>, Error>, ProvenanceLog) {
    let (first, second) = if bar_first {
        (bar, block)
    } else {
        (block, bar)
    };
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(&[first], &[second], rule, policy, &mut log)
        .map(|fused| fused.solids().to_vec());
    (fused, log)
}

fn what_it_holds_by_hand(rule: Rule, bar_first: bool) -> f64 {
    let half_the_bar = PI * RADIUS * RADIUS * HEIGHT / 2.0;
    let the_block = 12.0 * 6.0 * 4.0;
    match (rule, bar_first) {
        (Rule::Union, _) => the_block + half_the_bar,
        (Rule::Intersection, _) | (Rule::Difference, true) => half_the_bar,
        (Rule::Difference, false) => the_block - half_the_bar,
        (other, _) => panic!("no volume worked out by hand for {other:?}"),
    }
}

/// What the fuse reads the turned bar's seam corner as: the stored corner,
/// moved, with the cap and the wall round it as the bar stores them.
fn where_its_seam_corner_lands(circle: ACapCircle) -> (PredicatePoint, f64) {
    let [x, y, z] = circle.stored_corner;
    let cap = a_cap(z, circle.outward);
    let wall = Surface::Cylinder(Cylinder::new(Axis::z(), RADIUS).expect("a positive radius"));
    let (corner, wobble) = where_a_corner_lands_after_moving(
        Point::new(x, y, z),
        [&cap, &wall],
        &placement(circle.quarters),
    )
    .expect("a finite corner and a finite move");
    (corner, wobble.magnitude())
}

fn a_cap(z: f64, outward: f64) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, outward))
            .expect("a unit normal"),
    )
}

/// The turned bar, built in place through the public [`SolidBuilder`] the way
/// `add_round_bar` builds an unplaced one: two corners on the seam, the seam,
/// two cap circles, and a wall that runs round the bottom, up the seam, back
/// round the top and down the seam.
fn a_bar_built_with_its_seam_where_the_turn_puts_it(model: &mut Model) -> SolidKey {
    let [x, y, bottom_z] = THE_BOTTOM_CIRCLE_TURNED_THREE_QUARTERS.lands;
    let top_z = bottom_z + HEIGHT;
    let mut builder = SolidBuilder::new();

    let bottom_corner = builder
        .add_vertex(Point::new(x, y, bottom_z))
        .expect("a finite point");
    let top_corner = builder
        .add_vertex(Point::new(x, y, top_z))
        .expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling of a pipe is straight");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the bottom cap cuts the pipe in one circle");
    let top_circle = builder
        .add_edge(
            top_corner,
            top_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the top cap cuts the pipe in one circle");

    let cap = |builder: &mut SolidBuilder, z: f64, outward: f64, edge, turn| {
        let coedge = builder
            .add_coedge(edge, turn)
            .expect("an edge of this build");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("one closed edge is a wire");
        builder
            .add_face(a_cap(z, outward), &[wire])
            .expect("a cap is a plane trimmed by its circle")
    };
    let bottom = cap(
        &mut builder,
        bottom_z,
        -1.0,
        bottom_circle,
        Orientation::Reversed,
    );
    let top = cap(&mut builder, top_z, 1.0, top_circle, Orientation::Forward);

    let round_the_wall: Vec<DraftStep> = [
        (bottom_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (top_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .into_iter()
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    })
    .collect();
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("round the bottom, up the seam, back round the top, down the seam");
    let wall = builder
        .add_face(the_wall_where_the_turn_puts_it(), &[wire])
        .expect("a face on the pipe");

    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (wall, Orientation::Forward),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar described this way is a solid")
}

/// The pipe of radius `11/2` about the `z` axis through `(0, 0, −2)`, which is
/// where any turn about `z` followed by the lift puts the bar's wall.
fn the_wall_where_the_turn_puts_it() -> Surface {
    Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::new(0.0, 0.0, LIFT), Vector::new(0.0, 0.0, 1.0))
                .expect("a unit direction"),
            RADIUS,
        )
        .expect("a positive radius"),
    )
}
