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
//! of `−1.26e-14` and a box one ulp wide decides it. A **gap**.
//!
//! **A plane exactly on `z = y` is ordered, and meets another wall.** Built
//! face by face so its face is `z = y` to the bit, the meet is `(5, 0, 0)`
//! exactly, both intersections answer right, and the union and both
//! differences refuse `ResultIsNotManifold { count: 4 }`: an arc of the cap
//! circle and an arc of the ellipse join the same two corners on the one wall,
//! and come out as one edge.
//!
//! Each assertion's message carries the volume by hand that the refusal stands
//! in for. The bar is `π·5²·10`. The plane cuts it into a wedge, the part with
//! `0 ≤ z ≤ y`, whose volume is `∫∫ y dA` over the half disc `y ≥ 0`, which is
//! `2·5³/3 = 250/3`. The top cap plays no part, because `z ≤ y ≤ 5 < 10`.

use std::f64::consts::PI;

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;
const HEIGHT: f64 = 10.0;
/// Half the side of the turned block and of the prism, far enough out that
/// their only face near the bar is the leaning one.
const REACH: f64 = 20.0;
const THE_WEDGE: f64 = 2.0 * RADIUS * RADIUS * RADIUS / 3.0;
const A_CUBE_FORTY_ON_A_SIDE: f64 = 64_000.0;
/// A right triangle with legs of 40, forty long.
const A_PRISM_UNDER_Z_EQUALS_Y: f64 = 32_000.0;

/// **The six fuses of §252.8, all refused on the order of two points round the
/// bar's cap circle.**
///
/// The refusing face is the bar's bottom cap, `z = 0`, and the crossing is the
/// cap circle, where that plane meets the wall. The two points are the meet of
/// the cap plane, the wall and the turned face, a box round
/// `(5 − 6.3e-31, 2.512e-15, 0)`, and the rim start `(5, 0, 0)` itself. They
/// are two points: the box's `y` side is one ulp wide and above zero. The
/// order round the circle from `(0, −5, 0)` is the sign of a sweep whose exact
/// value is `−1.26e-14`, and the box's `x` side, `[5 − 3 ulps, 5 + 3 ulps]`,
/// makes that sweep straddle zero with an enclosure of `3.55e-14`.
#[test]
fn a_block_turned_to_lean_through_a_bars_rim_start_cannot_order_two_points_round_its_cap() {
    let policy = Policy::certified();
    let (model, bar, block) = a_bar_and(a_block_turned_to_lean_along_z_equals_y);

    for (rule, arguments, tools, by_hand) in the_six_fuses(bar, block, A_CUBE_FORTY_ON_A_SIDE) {
        let mut model = model.clone();
        let refused = model.fuse(
            &[arguments],
            &[tools],
            rule,
            &policy,
            &mut ProvenanceLog::new(),
        );
        let Err(Error::Undecidable { records }) = &refused else {
            panic!(
                "{rule:?}: expected Error::Undecidable on \"order along a crossing\", which \
                 stands in for one solid of volume {by_hand} (§284); got {refused:?}"
            );
        };
        let [record] = records.records() else {
            panic!("{rule:?}: one record, not {records:?}");
        };
        assert_eq!(
            record.predicate(),
            "order along a crossing",
            "{rule:?}: Error::Undecidable stands in for one solid of volume {by_hand} (§284)"
        );
        let [meet, rim_start] = record.inputs() else {
            panic!("{rule:?}: two points to order, not {:?}", record.inputs());
        };
        assert_eq!(
            *rim_start,
            PredicatePoint::Exact([RADIUS, 0.0, 0.0]),
            "{rule:?}: the second point is the bar's rim start"
        );
        assert!(
            meet.enclosure()[1].lo() > 0.0,
            "{rule:?}: the meet {meet:?} is off the rim start, so the two are not one point"
        );
    }
}

/// **Under a tolerant policy the order is settled, and the fuse comes apart
/// one stage on.**
///
/// The enclosure of `3.55e-14` is a volume, so its wobble at a working size of
/// 100 is `3.55e-18`, against a budget of `64·ε·100 = 1.42e-12`. The policy
/// reads it as zero, and the meet is folded into the rim start on the cap and
/// on the wall. The turned face keeps the meet as a corner of its own, so the
/// faces no longer agree where their edges end.
#[test]
fn under_a_tolerant_policy_the_turned_block_is_ordered_and_its_faces_disagree_on_a_corner() {
    let (model, bar, block) = a_bar_and(a_block_turned_to_lean_along_z_equals_y);
    let policy = Policy::tolerant(model.scale().nominal_size());

    for ((rule, arguments, tools, by_hand), count) in
        the_six_fuses(bar, block, A_CUBE_FORTY_ON_A_SIDE)
            .into_iter()
            .zip([3, 1, 1, 1, 3, 1])
    {
        let mut model = model.clone();
        let refused = model.fuse(
            &[arguments],
            &[tools],
            rule,
            &policy,
            &mut ProvenanceLog::new(),
        );
        assert!(
            matches!(refused, Err(Error::ResultIsNotManifold { count: got }) if got == count),
            "{rule:?}: expected Error::ResultIsNotManifold {{ count: {count} }}, which stands in \
             for one solid of volume {by_hand} (§284); got {refused:?}"
        );
    }
}

/// **A plane exactly on `z = y` puts the two points in order, and two arcs on
/// the wall then get one name.**
///
/// Both policies alike. The meet is the rim start to the bit, so nothing is
/// left to order. The cap circle and the ellipse both run from `(5, 0, 0)` to
/// `(−5, 0, 0)` along the wall, and an arc of each comes out as one edge with
/// four faces along it. The intersections keep only one of those arcs, so they
/// answer, and right.
#[test]
fn a_plane_exactly_through_a_bars_rim_start_is_ordered_and_two_arcs_on_its_wall_get_one_name() {
    let (model, bar, prism) = a_bar_and(a_prism_under_z_equals_y);

    for policy in [
        Policy::certified(),
        Policy::tolerant(model.scale().nominal_size()),
    ] {
        for (rule, arguments, tools, by_hand) in the_six_fuses(bar, prism, A_PRISM_UNDER_Z_EQUALS_Y)
        {
            let mut model = model.clone();
            let fused = model.fuse(
                &[arguments],
                &[tools],
                rule,
                &policy,
                &mut ProvenanceLog::new(),
            );
            if rule != Rule::Intersection {
                assert!(
                    matches!(fused, Err(Error::ResultIsNotManifold { count: 4 })),
                    "{rule:?}: expected Error::ResultIsNotManifold {{ count: 4 }}, which stands \
                     in for one solid of volume {by_hand} (§284); got {fused:?}"
                );
                continue;
            }
            let fused = fused.unwrap_or_else(|refused| panic!("{rule:?}: {refused:?}"));
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| panic!("{rule:?} came back broken: {broken:?}"));
            let [solid] = fused.solids() else {
                panic!("{rule:?} is one wedge: {:?}", fused.solids());
            };
            let measured = model
                .solid(*solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .expect("a solid measures");
            assert!(
                (measured.volume() - by_hand).abs() <= measured.volume_error(),
                "{rule:?}: measured {} ± {} against {by_hand} by hand",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
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
