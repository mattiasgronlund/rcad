//! Two overlapping parallel bars, fused by every rule and both ways round.
//!
//! Ordinary mechanical CAD: two round bars that run the same way and share some
//! material. **Every fuse of the pair answers**, and
//! [`two_overlapping_parallel_bars_fuse_by_every_rule_whichever_way_round`]
//! checks all six — three rules, each in both orders — against what the shape
//! says: the volume, the corners and the faces, all worked out by hand.
//!
//! The pair used to refuse, and in this order, each refusal closed in a
//! different place. A closed one coming back is a regression, and the six fuses
//! would name it:
//!
//! 1. **The straight gate** (§48). `arrange.rs` swept the whole
//!    cylinder-cylinder row into `false`, so a crossing `rcad-geom` builds
//!    exactly as two endless lines fell out as
//!    `ACrossingThisCannotBeOrderedAlong` with nothing having run out of
//!    arithmetic. `two_pipes_side_by_side_run_up_their_common_axis` in
//!    `arrange.rs` measures the crossing being ordered up `z`.
//! 2. **The three-surface meet.** A corner of the answer is where a bar's flat
//!    cap meets *both* pipes, one plane and two quadrics, and
//!    `meet_of_three_surfaces` used to want two of the three flat. Two pipes
//!    running the same way square `p` the same way, so their difference is flat
//!    and the triple is two planes and a quadric after all.
//!    [`a_cap_plane_against_two_side_by_side_pipes_meets_them_at_three_and_four`]
//!    checks the corners it places.
//! 3. **`select.rs`, on a cap piece bounded by an arc.** The same region of the
//!    same plane had three corners as one bar's cap piece and two as the
//!    other's, and two corners is no polygon, so the two orders gave out in
//!    different searches until a pipe's bulge was counted (§218.5). Both then
//!    gave out placing the piece against the other solid, until a flat face
//!    with a curved side was asked what it holds rather than wound by its
//!    corners (`13d5abaa`, §224.4, §231.4).
//! 4. **Two arcs of two circles named as one edge** (§245.4, hole B), in
//!    `assemble.rs`. Both cap circles pass through `(3, ±4, ·)`, so two good
//!    edges came out as one with four faces along it:
//!    [`an_arc_of_each_bar_between_the_same_two_corners_is_an_edge_of_its_own`].
//! 5. **A bar's rim start taken for a corner** (§245.3, hole A; §252), in
//!    `assemble.rs`. `add_round_bar` starts each cap circle at `(5, 0, ·)`, and
//!    one bar's pieces walked their arc through that point as two sides where
//!    the other bar's walked it as one:
//!    [`where_a_bar_started_its_cap_circle_is_not_a_corner_of_the_answer`].
//!
//! Selection kept the right pieces for the last two (§245.2); both holes were in
//! how the kept pieces were named, and both refused `ResultIsNotManifold`.
//!
//! **Fusing any of the six answers once more still refuses**
//! `AFaceTrimmedByACurveThisCannotFollow` (§252.8). Nothing here asks that yet.

use rcad_geom::{
    Axis, Cylinder, Deflection, Isometry, MeetOfThree, Plane, Point, Scale, Surface, Vector,
    meet_of_three_surfaces,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

const RADIUS: f64 = 5.0;
const HEIGHT: f64 = 10.0;
/// How far apart the two axes are: less than `2·RADIUS`, so the bars overlap,
/// and more than `0`, so they are two bars and not one.
const APART: f64 = 6.0;

/// `radius = 5` and `apart = 6` is a 3-4-5 either side of the plane of the two
/// axes, so the two lines the pipes cut run through `(3, ±4, z)` and land on
/// coordinates an `f64` holds exactly. No fuse below can refuse because a
/// number would not fit, and every corner is compared as the exact number it is.
fn two_bars_side_by_side() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let here = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar of radius five, ten tall");
    let there = model
        .add_round_bar(
            RADIUS,
            HEIGHT,
            &Isometry::translation(Vector::new(APART, 0.0, 0.0)).expect("a shift along `x`"),
        )
        .expect("a second bar of the same size, six along `x`");
    (model, here, there)
}

/// **Every fuse of two overlapping parallel bars answers, whichever bar is the
/// tool.**
///
/// The three rules and both orders of the pair, each fuse on its own copy of the
/// model. A union or an intersection is one solid whichever way round it is
/// asked for, and the two differences are mirror images of each other in the
/// plane `x = 3`, so each rule has one answer by hand for both orders. A fuse
/// whose answer depended on which solid was the tool fails here.
///
/// Each answer is checked against the shape, worked out without the kernel:
///
/// - **Its volume.** A bar is `π·5²·10`, and the two share a lens of two discs
///   of radius 5 whose centres are 6 apart, ten tall. The lens's half-angle is
///   `acos(3/5)` and its chord is 8 long, so its area is `2·5²·acos(3/5) − 3·8`.
///   The union is two bars less the lens, the intersection is the lens, and a
///   difference is one bar less it.
/// - **Its corners**, `(3, ±4, 0)` and `(3, ±4, 10)`: where the two pipes cross
///   each other, on the two cap planes. Both ends of every edge are read, so a
///   bar's rim start at `(5, 0, ·)` or `(11, 0, ·)` left in as a corner fails.
/// - **Its faces: two flat and two round.** Every answer is one piece of each
///   cap plane and one piece of each pipe. The two pipes cut each other along
///   the lines through `(3, ±4, ·)`, so each pipe's wall is two pieces, the part
///   inside the other bar and the part outside, and every rule keeps exactly one
///   of the two. What a rule keeps of one cap plane is one region, and pieces of
///   it that lie side by side are joined into one face (§17.4).
#[test]
fn two_overlapping_parallel_bars_fuse_by_every_rule_whichever_way_round() {
    let policy = Policy::certified();
    let (model, here, there) = two_bars_side_by_side();
    let one_bar = std::f64::consts::PI * RADIUS * RADIUS * HEIGHT;
    let half_apart = APART / 2.0;
    let lens = 2.0 * RADIUS * RADIUS * (half_apart / RADIUS).acos()
        - half_apart * (4.0 * RADIUS * RADIUS - APART * APART).sqrt();
    let shared = lens * HEIGHT;
    let four_corners = vec![
        [3.0, -4.0, 0.0],
        [3.0, -4.0, HEIGHT],
        [3.0, 4.0, 0.0],
        [3.0, 4.0, HEIGHT],
    ];

    for (rule, by_hand) in [
        (Rule::Union, 2.0 * one_bar - shared),
        (Rule::Intersection, shared),
        (Rule::Difference, one_bar - shared),
    ] {
        for (which, arguments, tools) in [
            ("here on there", here, there),
            ("there on here", there, here),
        ] {
            let mut model = model.clone();
            let fused = model
                .fuse(
                    &[arguments],
                    &[tools],
                    rule,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|refused| panic!("{rule:?} {which}: {refused:?}"));
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| panic!("{rule:?} {which} came back broken: {broken:?}"));
            let [solid] = fused.solids() else {
                panic!(
                    "{rule:?} {which} of two overlapping bars is one solid: {:?}",
                    fused.solids()
                );
            };
            let solid = model.solid(*solid).expect("this model made the key");

            let surfaces: Vec<Surface> = solid.faces().map(|face| face.surface()).collect();
            let flat = surfaces
                .iter()
                .filter(|surface| matches!(surface, Surface::Plane(_)))
                .count();
            let round = surfaces
                .iter()
                .filter(|surface| matches!(surface, Surface::Cylinder(_)))
                .count();
            assert_eq!(
                (surfaces.len(), flat, round),
                (4, 2, 2),
                "{rule:?} {which}: the faces of the answer lie on {surfaces:?}"
            );

            let mut corners: Vec<[f64; 3]> = solid
                .faces()
                .flat_map(|face| face.wires().collect::<Vec<_>>())
                .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
                .flat_map(|coedge| [coedge.edge().start(), coedge.edge().end()])
                .map(|corner| corner.point().coordinates())
                .collect();
            corners.sort_by(|one, other| one.partial_cmp(other).expect("no corner is NaN"));
            corners.dedup();
            assert_eq!(
                corners, four_corners,
                "{rule:?} {which}: the corners of the answer"
            );

            let measured = solid
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .expect("a solid measures");
            assert!(
                (measured.volume() - by_hand).abs() <= measured.volume_error(),
                "{rule:?} {which}: measured {} ± {} against {by_hand} by hand",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
}

/// **An arc of one bar's circle and an arc of the other's are two edges, even
/// where they join the same two corners.**
///
/// Both circles pass through `(3, ±4, 0)` on the bottom cap plane, and through
/// `(3, ±4, 10)` on the top one, so four arcs join each pair of corners: two on
/// each circle. `assemble.rs` used to name an arc by its two corners and by
/// which of its own circle's two arcs it is — which tells the two arcs of one
/// circle apart and not an arc of one circle from an arc of the other. Two good
/// edges then came out as one edge with four faces along it, and these two
/// fuses refused `ResultIsNotManifold { count: 4 }` (§245.4).
///
/// These are the two of the six fuses that met only that hole (§245.6). The
/// check is the volume worked out by hand: a bar is `π·5²·10`, and the two
/// share a lens of two discs of radius 5 whose centres are 6 apart, ten tall.
/// The lens's half-angle is `acos(3/5)` and its chord is 8 long, so its area is
/// `2·5²·acos(3/5) − 3·8`.
#[test]
fn an_arc_of_each_bar_between_the_same_two_corners_is_an_edge_of_its_own() {
    let policy = Policy::certified();
    let (model, here, there) = two_bars_side_by_side();
    let one_bar = std::f64::consts::PI * RADIUS * RADIUS * HEIGHT;
    let half_apart = APART / 2.0;
    let lens = 2.0 * RADIUS * RADIUS * (half_apart / RADIUS).acos()
        - half_apart * (4.0 * RADIUS * RADIUS - APART * APART).sqrt();
    let shared = lens * HEIGHT;

    for (rule, arguments, tools, by_hand) in [
        (Rule::Union, there, here, 2.0 * one_bar - shared),
        (Rule::Difference, here, there, one_bar - shared),
    ] {
        let mut model = model.clone();
        let fused = model
            .fuse(
                &[arguments],
                &[tools],
                rule,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refused| panic!("{rule:?} of two side-by-side bars: {refused:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|broken| panic!("{rule:?} came back broken: {broken:?}"));
        let [solid] = fused.solids() else {
            panic!(
                "{rule:?} of two overlapping bars is one solid: {:?}",
                fused.solids()
            );
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

/// **Where a bar started its own cap circle is not a corner of the answer.**
///
/// `add_round_bar` starts each cap circle at `(5, 0, ·)`, and nothing meets the
/// circle there: the point is only where the builder began it. The faces cut
/// from that bar walked its circle as two sides through the point, while the
/// other bar's cap had the same arc as one cut. Both were kept, so three edges
/// had one face each, and the three fuses below refused
/// `ResultIsNotManifold { count: 1 }` (§245.3, hole A).
///
/// The fourth is the intersection that answered already, with `(5, 0, 0)` and
/// `(5, 0, 10)` as corners in the middle of a curved edge. Now it has the four
/// corners the other three have.
///
/// The check is the volume worked out by hand, as in
/// [`an_arc_of_each_bar_between_the_same_two_corners_is_an_edge_of_its_own`],
/// and the corners, which are whole numbers in boxes of no width
/// ([`a_cap_plane_against_two_side_by_side_pipes_meets_them_at_three_and_four`]).
#[test]
fn where_a_bar_started_its_cap_circle_is_not_a_corner_of_the_answer() {
    let policy = Policy::certified();
    let (model, here, there) = two_bars_side_by_side();
    let one_bar = std::f64::consts::PI * RADIUS * RADIUS * HEIGHT;
    let half_apart = APART / 2.0;
    let lens = 2.0 * RADIUS * RADIUS * (half_apart / RADIUS).acos()
        - half_apart * (4.0 * RADIUS * RADIUS - APART * APART).sqrt();
    let shared = lens * HEIGHT;
    let four_corners = vec![
        [3.0, -4.0, 0.0],
        [3.0, -4.0, HEIGHT],
        [3.0, 4.0, 0.0],
        [3.0, 4.0, HEIGHT],
    ];

    for (rule, arguments, tools, by_hand) in [
        (Rule::Union, here, there, 2.0 * one_bar - shared),
        (Rule::Intersection, there, here, shared),
        (Rule::Difference, there, here, one_bar - shared),
        (Rule::Intersection, here, there, shared),
    ] {
        let mut model = model.clone();
        let fused = model
            .fuse(
                &[arguments],
                &[tools],
                rule,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refused| panic!("{rule:?} of two side-by-side bars: {refused:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|broken| panic!("{rule:?} came back broken: {broken:?}"));
        let [solid] = fused.solids() else {
            panic!(
                "{rule:?} of two overlapping bars is one solid: {:?}",
                fused.solids()
            );
        };
        let solid = model.solid(*solid).expect("this model made the key");

        let mut corners: Vec<[f64; 3]> = solid
            .faces()
            .flat_map(|face| face.wires().collect::<Vec<_>>())
            .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
            .map(|coedge| coedge.edge().start().point().coordinates())
            .collect();
        corners.sort_by(|one, other| one.partial_cmp(other).expect("no corner is NaN"));
        corners.dedup();
        assert_eq!(corners, four_corners, "{rule:?}: the corners of the answer");

        let measured = solid
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

/// **Where a bar started its cap circle stays a corner when the other bar's
/// circle crosses it there.**
///
/// The other side of [`where_a_bar_started_its_cap_circle_is_not_a_corner_of_the_answer`].
/// A second bar centred at `(5, 5, 0)` has a circle through `(5, 0, 0)`, so
/// the two circles cross exactly where the first bar began its own. Subtracting
/// one circle from the other leaves `x + y = 5`, so they cross at `(5, 0, ·)`
/// and `(0, 5, ·)`, whole numbers again. There a third surface meets the
/// circle, the cut along it runs along no input edge, and the vertex is a
/// corner of every answer.
///
/// The centres are `5·√2` apart, so the lens's half-angle is `acos(√2/2) = π/4`
/// and its area is `2·5²·π/4 − (5√2/2)·5√2 = 12.5·π − 25`.
#[test]
fn where_a_bar_started_its_cap_circle_stays_a_corner_where_the_other_circle_crosses_it() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let here = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar of radius five, ten tall");
    let there = model
        .add_round_bar(
            RADIUS,
            HEIGHT,
            &Isometry::translation(Vector::new(RADIUS, RADIUS, 0.0)).expect("a shift"),
        )
        .expect("a second bar whose circle runs through the first one's start");
    let one_bar = std::f64::consts::PI * RADIUS * RADIUS * HEIGHT;
    let shared = (12.5 * std::f64::consts::PI - 25.0) * HEIGHT;
    let four_corners = vec![
        [0.0, 5.0, 0.0],
        [0.0, 5.0, HEIGHT],
        [5.0, 0.0, 0.0],
        [5.0, 0.0, HEIGHT],
    ];

    for (rule, by_hand) in [
        (Rule::Union, 2.0 * one_bar - shared),
        (Rule::Intersection, shared),
        (Rule::Difference, one_bar - shared),
    ] {
        for (arguments, tools) in [(here, there), (there, here)] {
            let mut model = model.clone();
            let fused = model
                .fuse(
                    &[arguments],
                    &[tools],
                    rule,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|refused| panic!("{rule:?} of two crossing bars: {refused:?}"));
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| panic!("{rule:?} came back broken: {broken:?}"));
            let [solid] = fused.solids() else {
                panic!(
                    "{rule:?} of two crossing bars is one solid: {:?}",
                    fused.solids()
                );
            };
            let solid = model.solid(*solid).expect("this model made the key");

            let mut corners: Vec<[f64; 3]> = solid
                .faces()
                .flat_map(|face| face.wires().collect::<Vec<_>>())
                .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
                .map(|coedge| coedge.edge().start().point().coordinates())
                .collect();
            corners.sort_by(|one, other| one.partial_cmp(other).expect("no corner is NaN"));
            corners.dedup();
            assert_eq!(corners, four_corners, "{rule:?}: the corners of the answer");

            let measured = solid
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

/// **The corner the fuse could not place, and now can, asked for on its own.**
///
/// A bar's cap plane against the two pipes: one flat surface and two quadrics.
/// `radius = 5` and `apart = 6` gives `x² + y² = 25` against
/// `(x − 6)² + y² = 25`, and subtracting one from the other leaves `12x = 36` —
/// so the corners are `(3, ±4, 0)`, a 3-4-5 with every number whole. Checked
/// against those numbers and against boxes of no width, so what is compared is
/// the whole answer and not a rounding of it.
///
/// This is the call `Arrangement::place_the_meets` makes and hands straight
/// back — the only place in this crate that lets that error out, since
/// `bake.rs`'s corner search swallows it and tries the next triple.
#[test]
fn a_cap_plane_against_two_side_by_side_pipes_meets_them_at_three_and_four() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let cap = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
            .expect("a bar's bottom cap"),
    );
    let here = Surface::Cylinder(Cylinder::new(Axis::z(), RADIUS).expect("the first pipe"));
    let there = Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::new(APART, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
                .expect("an axis six along `x`"),
            RADIUS,
        )
        .expect("the second pipe"),
    );

    let answer = meet_of_three_surfaces([&cap, &here, &there], &policy, &mut log)
        .expect("two pipes running the same way and a cap");
    let MeetOfThree::At(corners) = &answer else {
        panic!("expected two corners, got {answer:?}");
    };
    let mut placed: Vec<[f64; 3]> = corners
        .iter()
        .map(|corner| {
            corner.enclosure().map(|side| {
                assert_eq!(side.lo(), side.hi(), "{corner:?} is a box with width");
                side.lo()
            })
        })
        .collect();
    placed.sort_by(|one, other| one[1].total_cmp(&other[1]));
    assert_eq!(placed, vec![[3.0, -4.0, 0.0], [3.0, 4.0, 0.0]]);
    assert!(
        log.is_empty(),
        "a certified answer wrote {log:?} into the log"
    );
}
