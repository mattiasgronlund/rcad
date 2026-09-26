//! Two overlapping parallel bars, united — and what still stops it.
//!
//! Ordinary mechanical CAD: two round bars that run the same way and share some
//! material. §48 pinned one reason the fuse could not do it — `arrange.rs`'s
//! straight gate swept the whole cylinder-cylinder row into `false`, so a
//! crossing that `rcad-geom` builds exactly as two endless lines fell past the
//! straight gate, past the conic gate, and out as
//! `ACrossingThisCannotBeOrderedAlong` with nothing having run out of
//! arithmetic. That gap is closed, and
//! `two_pipes_side_by_side_run_up_their_common_axis` in `arrange.rs` measures
//! the crossing being ordered up `z` where it used to be refused.
//!
//! **The second reason is closed too, and this file moved with it.** A corner of
//! the answer is where a bar's flat cap meets *both* pipes, which is one plane
//! and two quadrics, and `meet_of_three_surfaces` used to want two of its three
//! flat. It now answers this triple exactly: two pipes running the same way
//! square `p` the same way, so their difference is a flat surface and the triple
//! is two planes and a quadric after all. The corners come back as `(3, ±4, 0)`,
//! whole numbers in boxes of no width, and
//! `a_cap_plane_against_two_side_by_side_pipes_meets_them_at_three_and_four`
//! below is where that is checked.
//!
//! **So the fuse now stops one gate further in, and this file's job is to say
//! which one.** It is no longer geometry at all: with the corners placed, the
//! cap disc is cut into pieces by the other pipe's wall, and `select.rs` gives
//! up on one of those pieces, undecided with an unbounded enclosure.
//!
//! **Which way it gave up used to depend on which bar was the tool, and that is
//! closed.** `place_the_piece` raised one name for two different searches
//! (§216.4), so both orders read *"a point inside a piece of a face"* and the
//! pair looked symmetric; §217 split the names and they were not. §218.5 found
//! the cause and it was a gap, not the draw: the same region of the same plane
//! has three corners as one bar's cap piece and two as the other's, because a
//! bar's own rim start vertex is a corner of its own cap pieces and the other
//! bar's is not. Two corners is no polygon, so the winding count threw away
//! `(4, 0, 0)` — strictly inside both circles on whole numbers. Counting a
//! pipe's bulge closes it, and both orders now stop at the same next search.
//!
//! **Closing it does not make the bars fuse**, and nobody expected it to. A
//! piece bounded partly by an arc being hard to walk is still the missing
//! capability §48 names in `ACrossingThisCannotBeOrderedAlong`'s conic half and
//! not the quartic one.
//!
//! Three refusals, in order, each one closed by a different unit: the straight
//! gate, then the three-surface meet, then `select.rs` on a curved piece.
//! The first two are behind us and both regressions would show up here by name.

use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Cylinder, Deflection, Error as GeomError, Isometry, MeetOfThree, Plane, Point, Scale,
    Surface, Vector, meet_of_three_surfaces,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

const RADIUS: f64 = 5.0;
const HEIGHT: f64 = 10.0;
/// How far apart the two axes are: less than `2·RADIUS`, so the bars overlap,
/// and more than `0`, so they are two bars and not one.
const APART: f64 = 6.0;

/// `radius = 5` and `apart = 6` is a 3-4-5 either side of the plane of the two
/// axes, so the two lines the pipes cut run through `(3, ±4, z)` and land on
/// coordinates an `f64` holds exactly. Nothing that refuses below is refusing
/// because a number would not fit.
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

/// **Where the union of two overlapping parallel bars stops today, by name.**
///
/// Not `.is_err()`: the whole worth of this test is *which* refusal it is. Two
/// of them are behind us and both are named below, because either coming back is
/// a regression rather than a curiosity — `ACrossingThisCannotBeOrderedAlong`
/// was the answer before §48's gate flip, and
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` was the answer until the
/// three-surface meet learned one plane and two quadrics.
///
/// What is left is `select.rs`, and since §217 the record says **which** of its
/// two searches gave out. It is not the same one both ways round, which is what
/// the single name was hiding; [`the_walls_measured_when_217_split_them`] is the
/// pin and says what was found.
///
/// All three rules and both orders of the pair, because a refusal that depended
/// on which solid was the tool would be a different bug.
#[test]
#[ignore = "since 13d5abaa the union refuses ResultIsNotManifold { count: 1 } instead, undiagnosed, §231"]
fn two_overlapping_parallel_bars_stop_in_select_whichever_way_round() {
    let policy = Policy::certified();
    let (mut model, here, there) = two_bars_side_by_side();
    let mut stopped_at: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();

    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for (which, (arguments, tools)) in [
            ("here on there", ([here], [there])),
            ("there on here", ([there], [here])),
        ] {
            let mut log = ProvenanceLog::new();
            let refused = model.fuse(&arguments, &tools, rule, &policy, &mut log);
            assert!(
                !matches!(
                    refused,
                    Err(Error::Geometry(
                        GeomError::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
                    )) | Err(Error::ACrossingThisCannotBeOrderedAlong)
                ),
                "{rule:?} on two side-by-side bars went back to a refusal that is closed: \
                 {refused:?}"
            );
            let Err(Error::Undecidable { records }) = &refused else {
                panic!("{rule:?} on two side-by-side bars answered {refused:?}");
            };
            for record in records.records() {
                *stopped_at
                    .entry(format!("{rule:?} {which}"))
                    .or_default()
                    .entry(record.predicate().to_owned())
                    .or_default() += 1;
            }
        }
    }
    assert_eq!(
        stopped_at,
        the_wall_both_orders_stop_at(),
        "the side-by-side bars stopped somewhere else; re-derive this rather than widening it \
         to hold both names (§19.1)"
    );

    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("a refused fuse leaves the model exactly as it was");
}

/// Which of `select.rs`'s two searches gave out, per rule and per order.
///
/// §216.4 found `place_the_piece` raising one name for both searches and §217
/// split them. The pin is a map rather than a string because it was written
/// while this pair reached **both** halves.
///
/// **It reaches one half now, and that is what the asymmetry being closed looks
/// like from here.** §218.5 measured why the two orders differed: the refusing
/// cap piece is `disc@0 ∩ disc@6` on the same plane `z = 0` either way round,
/// but a bar's own rim start vertex is a corner of its own cap pieces and the
/// other bar's is not — three corners one way, two the other. Two corners is no
/// polygon, so the winding was 0 for every candidate and `(4, 0, 0)` was thrown
/// away although `16 < 25` and `4 < 25` put it strictly inside both circles.
/// Counting a pipe's bulge gives that piece an inside, and both orders now walk
/// on to the same next search and give out there.
///
/// **What is left is one wall, not the missing one.** Every rule and both orders
/// stop at "a point in a piece that every solid could place" — a different
/// search from the one they used to split on, and a separate piece of work. The
/// assertion above about a closed refusal coming back is untouched.
fn the_wall_both_orders_stop_at() -> BTreeMap<String, BTreeMap<String, usize>> {
    let every_solid_could_place = || {
        BTreeMap::from([(
            "a point in a piece that every solid could place".to_owned(),
            1,
        )])
    };
    ["Union", "Intersection", "Difference"]
        .into_iter()
        .flat_map(|rule| {
            [
                (format!("{rule} here on there"), every_solid_could_place()),
                (format!("{rule} there on here"), every_solid_could_place()),
            ]
        })
        .collect()
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
