//! **A bend cut where two arcs of one circle on its tube matter** (Q1327,
//! Q1387).
//!
//! A plane cuts a ring (a torus) in up to two circles, and a fuse keeps arcs of
//! them. `which_arc_of_a_circle_this_is` in `fuse/assemble.rs` names such an
//! arc by its two corners and, where two arcs of one crossing run between the
//! same two corners, by which of them it runs round. Until Q1387 it told the
//! two apart for a plane against a ball, a cone or a pipe, and not against a
//! ring.
//!
//! Q1327 asked for two shapes, and this file and its neighbour hold them:
//!
//! - **(a) a bend under two planes whose meeting line crosses the tube
//!   twice,** with an arc of each plane kept between those two points: the
//!   half bend under a half bar here.
//! - **(b) a cut that keeps both arcs of one crossing of a plane and a ring.**
//!   That is the union of the top halves of one ring's two half bends, pinned
//!   in `a_joined_flat_face_whose_rings_enclose_no_area.rs`
//!   (`the_top_halves_of_two_half_bends_fuse_by_every_rule_to_what_pappus_gives`):
//!   its floor is the whole flat ring between the circles of radius 3 and 5,
//!   and each circle is two halves between the same two corners. Until Q1387
//!   that union was refused `ResultIsNotManifold { count: 4 }`, both ways
//!   round, which is what Q1327 guessed.
//!
//! The quarter bend and the round bar of its own radius here is a second
//! (b): the bar's wall cuts each cap's circle at the top and the bottom of the
//! tube into two halves between the same two corners, so the fuse meets both
//! and has to keep the right one. Until Q1387 it kept the wrong one.
//!
//! Every answer is checked against Pappus: a flat piece turned round the
//! ring's axis holds its area times the length its centroid travels.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod rings;

/// **A half bend under the corner of a half bar.** The half bend of ring 4 and
/// tube 1.25 runs from `+x` round through `+y` to `−x`. The half bar is a round
/// bar of radius 10 standing on `z = 0.75`, cut back to `x ≤ 0`, so its floor
/// and its cut face meet along the line `x = 0, z = 0.75`. That line crosses
/// the tube at `(0, 3, 0.75)` and `(0, 5, 0.75)`, both exactly. The floor cuts
/// the tube in the circles of radius 3 and 5, and the cut face cuts it in the
/// circle round `(0, 4, 0)`. Of each, the part they share keeps an arc ending
/// at one of those two points or running between them.
///
/// The bar is cut at `x = 0` and not at `y = 0` because a round bar's seam lies
/// on `y = 0`, and a cut along it is another shape.
///
/// What they share is the piece of the tube's cross-section over
/// `z = 0.75`, a segment of a disk of radius 1.25 cut 0.75 from its centre,
/// turned a quarter turn from `+y` to `−x`. The segment is even about the
/// tube's middle, so its centroid is 4 from the axis and Pappus gives its area
/// times `4 · π/2`. The half bend holds `π² · 4 · 1.25²` and the half bar
/// `π · 10² · 4.25 / 2`. **Every rule, both ways round, answers that and leaves
/// a model that validates.**
#[test]
fn a_half_bend_under_two_planes_meeting_across_its_tube_fuses_by_every_rule_to_what_pappus_gives() {
    let (ring, tube, floor): (f64, f64, f64) = (4.0, 1.25, 0.75);
    let half_chord = (tube * tube - floor * floor).sqrt();
    let segment = tube * tube * (floor / tube).acos() - floor * half_chord;
    let shared = segment * ring * PI / 2.0;
    let bend = PI * PI * ring * tube * tube;
    let half_bar = PI * 100.0 * (5.0 - floor) / 2.0;

    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let half_bend = rings::a_bend_turned_and_written_out(&mut model, ring, tube, 2, 0)
        .unwrap_or_else(|why| panic!("a half bend: {why:?}"));
    let bar = a_half_bar_over(&mut model, floor);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the half bend and the half bar are sound");
    assert_measures(&model, half_bend, bend, "the half bend");
    assert_measures(&model, bar, half_bar, "the half bar");

    for (rule, bend_first, bar_first) in [
        (
            Rule::Union,
            bend + half_bar - shared,
            bend + half_bar - shared,
        ),
        (Rule::Intersection, shared, shared),
        (Rule::Difference, bend - shared, half_bar - shared),
    ] {
        for (one, other, volume, what) in [
            (half_bend, bar, bend_first, format!("{rule:?}, bend first")),
            (bar, half_bend, bar_first, format!("{rule:?}, bar first")),
        ] {
            assert_fuses_to(&mut model.clone(), one, other, rule, volume, &what);
        }
    }
}

/// A round bar of radius 10 from `z = floor` to `z = 5`, less a block over
/// `x ≥ 0`: the half of it on `x ≤ 0`.
fn a_half_bar_over(model: &mut Model, floor: f64) -> SolidKey {
    let bar = model
        .add_round_bar(10.0, 5.0 - floor, &at(0.0, 0.0, floor))
        .expect("a round bar of radius 10");
    let square = ProfileBuilder::new(vec![[0.0, -20.0], [20.0, -20.0], [20.0, 20.0], [0.0, 20.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square is a profile");
    let block = model
        .add_extrusion(&square, 7.0, &at(0.0, 0.0, -1.0))
        .expect("a block over x ≥ 0");
    let cut = model
        .fuse(
            &[bar],
            &[block],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the bar less the block: {why:?}"));
    let [half] = cut.solids() else {
        panic!(
            "the bar less the block is one solid, not {:?}",
            cut.solids()
        );
    };
    *half
}

/// **A quarter bend and the round bar of its own radius on its axis.** The
/// bar's wall is the pipe of radius `R` the ring runs round, so it cuts the
/// tube along the circles over its top and under its bottom, `z = ±r`, and
/// cuts each cap's circle at the top and the bottom of the tube, `(R, 0, ±r)`
/// or `(0, R, ±r)`: two halves between the same two corners, the inner one
/// inside the bar and the outer one outside it.
///
/// What they share is the inner half of the tube's cross-section turned a
/// quarter turn: a half disk of area `πr²/2` whose centroid is `R − 4r/3π` from
/// the axis, so Pappus gives `(π²Rr² − 4πr³/3)/4`. The bend holds `π²Rr²/2`,
/// and the bar, four tubes tall, `4πR²r`. Ring 4 and tube 1, and ring 6 and
/// tube 2. **Every rule, both ways round, answers that and leaves a model that
/// validates.**
///
/// Until Q1387 every one was refused `ACurvedFaceGoesRightRoundWithNoSeam`
/// while `assemble` measured the answer: a piece of the bend's wall was built
/// with the start cap's inner half and the end cap's outer half, so its one
/// wire climbed a whole turn round the tube (Q1402).
#[test]
fn a_quarter_bend_and_the_bar_of_its_own_radius_fuse_by_every_rule_to_what_pappus_gives() {
    for (ring, tube) in [(4.0, 1.0), (6.0, 2.0)] {
        let bend = PI * PI * ring * tube * tube / 2.0;
        let bar = 4.0 * PI * ring * ring * tube;
        let shared = (PI * PI * ring * tube * tube - 4.0 * PI * tube * tube * tube / 3.0) / 4.0;

        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let quarter_bend = rings::a_quarter_bend(&mut model, ring, tube);
        let round_bar = model
            .add_round_bar(ring, 4.0 * tube, &at(0.0, 0.0, -2.0 * tube))
            .expect("a round bar of the ring's radius");
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("the bend and the bar are sound");
        assert_measures(&model, quarter_bend, bend, "the quarter bend");
        assert_measures(&model, round_bar, bar, "the bar");

        for (rule, bend_first, bar_first) in [
            (Rule::Union, bend + bar - shared, bend + bar - shared),
            (Rule::Intersection, shared, shared),
            (Rule::Difference, bend - shared, bar - shared),
        ] {
            for (one, other, volume, first) in [
                (quarter_bend, round_bar, bend_first, "bend"),
                (round_bar, quarter_bend, bar_first, "bar"),
            ] {
                let what = format!("ring {ring}, tube {tube}, {rule:?}, {first} first");
                assert_fuses_to(&mut model.clone(), one, other, rule, volume, &what);
            }
        }
    }
}

/// That `one` fused with `other` under `rule` answers, leaves a model that
/// validates, and is one solid holding `volume`.
fn assert_fuses_to(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    volume: f64,
    what: &str,
) {
    let answer = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} refused: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    let [solid] = answer.solids() else {
        panic!("{what} is one solid, not {:?}", answer.solids());
    };
    assert_measures(model, *solid, volume, what);
}

fn assert_measures(model: &Model, solid: SolidKey, volume: f64, what: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::for_scale(model.scale()),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{what} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
