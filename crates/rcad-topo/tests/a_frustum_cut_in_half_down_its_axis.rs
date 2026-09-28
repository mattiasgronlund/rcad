//! A frustum cut in half by a block whose face `x = 0` runs down the frustum's
//! own axis. `docs/decisions.md` §215.2 named this as a fuse that could not be
//! done, and queue row Q19 asked for it to be measured again.
//!
//! The frustum is `cones::a_frustum(model, 0.5, 4.0, 8.0)`: a cone of slope a
//! half about `+z`, apex at the origin, between `z = 4` and `z = 8`. The plane
//! `x = 0` goes through the apex and cuts the cone in two straight rulings,
//! `y = z/2` and `y = −z/2`, so the half of the frustum on either side of it is
//! bounded by pieces of those two rulings, running from the corners `(0, ±2, 4)`
//! on the small rim to `(0, ±4, 8)` on the big one. The whole frustum is
//! `π/3 · (4² · 8 − 2² · 4) = 112π/3`, and each half is `56π/3`.
//!
//! # What §215.2 said, and what is true now
//!
//! §215.2 said the arrangement folded two corners on two different rulings into
//! one node and refused `Invalid::EdgeHasNoLength`. That was true when it was
//! written and is not true now. `f2fc7d1d` made `place_the_corners` take a
//! corner as lying on one curve of a surface only when that curve's picker
//! allows it. Before that, a corner of the hand-built half frustum at `(0, 2, 4)`
//! was put on **both** rulings, because the model lists the cone round it. A
//! ruling is ordered along `z`, the axis it climbs fastest, and `(0, 2, 4)` and
//! `(0, −2, 4)` are both at `z = 4`, so along the wrong ruling they came out as
//! one point and were folded together. Putting the old line back — the picker
//! question in `place_the_corners` replaced by `true` — brings the refusal back
//! in all six fuses of [`a_half_frustums_corners_on_two_rulings_stay_two_corners`].
//!
//! # Where the block's floor is decides the answer
//!
//! The block's face `x = 0` holds the two rulings as well, and on that face they
//! run on past the frustum and cross at the apex. Three floors, measured:
//!
//! - **Floor at `z = 2`, clear of the apex.** All six fuses answer, one solid
//!   each, the model validates, and every volume matches the closed form.
//!   [`a_block_clear_of_the_apex_cuts_the_frustum_in_half_under_every_rule`].
//! - **Floor at `z = −2`, the apex inside the block's face.** All six answer
//!   as they do clear of the apex.
//!   [`a_block_face_with_the_apex_inside_it_cuts_the_frustum_in_half_under_every_rule`].
//!   This is `a_block_face_through_a_cones_apex.rs`'s shape with a frustum in
//!   place of a pointed cone. This comment said before §347 that it refused
//!   `CannotTellTheCellsOfAFaceApart` on that face, as the pointed cone did;
//!   that was never pinned, and Q457 did not measure it again (§347.8).
//! - **Floor at `z = 0`, through the apex.** The apex is then a point on the
//!   block's own bottom edge. The three fuses that keep none of the block's
//!   floor answer, right:
//!   [`a_floor_through_the_apex_still_gives_the_half_frustum_under_intersection_and_difference`].
//!   The three that keep it, both unions and the block less the frustum, are
//!   refused `ResultIsNotManifold { count: 1 }`:
//!   [`a_floor_through_the_apex_is_not_a_manifold_under_the_fuses_that_keep_the_floor`].
//!
//! # Why the floor through the apex is not a manifold
//!
//! Before §347 all six fuses on this floor refused `Undecidable` naming *which
//! way a crossing runs*. Each ruling's direction was read as `∇S × ∇Q` at its
//! first node, which was the apex, and the cone has no gradient there. §347
//! reads a ruling's direction off its own line, and the fuses now get as far
//! as putting the answer's faces together.
//!
//! There the block's face `x = 0` and its floor `z = 0` disagree about the
//! edge they share. On `x = 0` the two rulings meet the bottom edge at the
//! apex, so that face has a node there and walks the bottom edge as two sides.
//! The floor is never cut by the cone: `intersect` says the plane `z = 0`
//! touches the cone at the apex and nowhere else, and `cuts.rs` reads a touch
//! at one point as two faces that meet nowhere. So the floor has no node at the
//! apex and walks the bottom edge as one side. A probe printed both faces'
//! corners and showed this (§347.8). It did not print which edge `assemble.rs`
//! finds claimed by one face. So the disagreement is the likely cause of the
//! refusal, but that has not been shown. Every fuse that keeps the floor next
//! to the pieces of `x = 0` on either side of the apex is refused, and every
//! fuse that keeps neither answers.
//!
//! **This is a gap and not a limit**: every answer exists and is written in the
//! refusals' messages.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, HalfSpace, Isometry, NamedRun, Orientation, Picker, Plane,
    Point, Scale, Surface, SurfaceMeet, Vector, intersect,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey, WireStepView,
};

mod cones;

/// Slope a half, so `slope · height` rounds to nothing at either rim.
const SLOPE: f64 = 0.5;

/// The heights of the frustum's two rims.
const LOW: f64 = 4.0;
const HIGH: f64 = 8.0;

/// Where the face `x = 0` meets the two rims: the ends of the two pieces of
/// ruling that bound either half of the frustum.
const CORNERS_DOWN_THE_AXIS: [[f64; 3]; 4] = [
    [0.0, -2.0, 4.0],
    [0.0, 2.0, 4.0],
    [0.0, -4.0, 8.0],
    [0.0, 4.0, 8.0],
];

/// Each of the six fuses of the frustum with the block
/// `[−10, 0] × [−10, 10] × [2, 12]` is one solid, the model validates, and the
/// volume is the closed form within what the mesh says it may miss by.
///
/// The block holds the frustum's `x ≤ 0` half whole, so what they share is
/// `56π/3`, and the block is `10 · 20 · 10 = 2000`. Every answer also has a
/// corner at each of the four points where `x = 0` meets the rims, which is
/// what §215.2 said the arrangement folded together.
#[test]
fn a_block_clear_of_the_apex_cuts_the_frustum_in_half_under_every_rule() {
    for (rule, frustum_first) in six_fuses() {
        is_one_solid_of_the_right_volume(2.0, rule, frustum_first);
    }
}

/// The block's floor lowered to `z = −2`, so the apex is inside its face
/// `x = 0`, where the two rulings cross. All six fuses answer as they do clear
/// of the apex. The block is `10 · 20 · 14 = 2800`.
#[test]
fn a_block_face_with_the_apex_inside_it_cuts_the_frustum_in_half_under_every_rule() {
    for (rule, frustum_first) in six_fuses() {
        is_one_solid_of_the_right_volume(-2.0, rule, frustum_first);
    }
}

/// The block's floor at `z = 0`, through the apex, so the apex is a point on
/// the block's bottom edge. The intersection, either order, and the frustum
/// less the block keep nothing of the block's floor, and each answers the half
/// frustum, `56π/3`.
#[test]
fn a_floor_through_the_apex_still_gives_the_half_frustum_under_intersection_and_difference() {
    for (rule, frustum_first) in [
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, true),
    ] {
        is_one_solid_of_the_right_volume(0.0, rule, frustum_first);
    }
}

/// The same floor under the three fuses that keep it: the union, either order,
/// and the block less the frustum. Each is refused
/// `ResultIsNotManifold { count: 1 }`, an edge of the answer claimed by one
/// face. The module doc says what a probe showed of why.
///
/// Every answer exists: the block is `10 · 20 · 12 = 2400` and holds the
/// frustum's `x ≤ 0` half whole. The message of each assertion is the answer
/// the refusal stands in for.
#[test]
fn a_floor_through_the_apex_is_not_a_manifold_under_the_fuses_that_keep_the_floor() {
    for (rule, frustum_first) in [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Difference, false),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let frustum = cones::a_frustum(&mut model, SLOPE, LOW, HIGH);
        let cutter = a_block(&mut model, [-10.0, -10.0, 0.0], [0.0, 10.0, 12.0]);
        let said = fuse(&mut model, frustum, cutter, rule, frustum_first);
        assert!(
            matches!(said, Err(Error::ResultIsNotManifold { count: 1 })),
            "{rule:?} (frustum first: {frustum_first}) should be one solid of volume {}; today \
             the floor has no corner at the apex where the face x = 0 has one. Got {said:?}",
            the_right_volume(0.0, rule, frustum_first)
        );
    }
}

/// The half frustum built face by face — its flat face on `x = 0`, its four
/// corners exactly where that face meets the rims — fused with the block
/// `[−10, 10] × [−10, 10] × [6, 12]` across its top.
///
/// **§215.2's defect was this pair's corners folding into one node.** Each
/// corner is on the cone, the cone crosses `x = 0` in two rulings, and each
/// corner was put on both of them. Now all six fuses get through the
/// arrangement and stop one stage later, in `select.rs`, for want of a point in
/// a piece that both solids can place. That is the half frustum's flat face:
/// it is bounded by rulings, which `holds.rs` cannot yet follow, and
/// `a_flat_cap_cut_by_a_cones_rim.rs`'s `the_face_down_the_axis_is_still_refused`
/// pins that on its own. With `place_the_corners` putting a corner on both
/// rulings again, all six come back `Invalid::EdgeHasNoLength`.
///
/// Every answer exists. The half frustum is `56π/3`, the block is
/// `20 · 20 · 6 = 2400`, and they share the half frustum above `z = 6`, which is
/// `½ · π/3 · ¼ · (8³ − 6³) = 37π/3`.
#[test]
fn a_half_frustums_corners_on_two_rulings_stay_two_corners() {
    let pi = core::f64::consts::PI;
    let (half, block, shared) = (half_the_frustum(), 20.0 * 20.0 * 6.0, 37.0 * pi / 3.0);
    for (rule, half_first) in six_fuses() {
        let want = match (rule, half_first) {
            (Rule::Union, _) => half + block - shared,
            (Rule::Intersection, _) => shared,
            (Rule::Difference, true) => half - shared,
            (Rule::Difference, false) => block - shared,
            (other, _) => panic!("{other:?} is not one of the six"),
        };
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let half_frustum = a_half_frustum(&mut model);
        let cutter = a_block(&mut model, [-10.0, -10.0, 6.0], [10.0, 10.0, 12.0]);
        let refused = fuse(&mut model, half_frustum, cutter, rule, half_first);
        assert_eq!(
            the_predicate_that_gave_up(&refused),
            Some("a point in a piece that every solid could place"),
            "{rule:?} (half frustum first: {half_first}) should be one solid of volume {want}; \
             today it gets through the arrangement and stops at the half frustum's flat face. \
             Got {refused:?}"
        );
    }
}

/// Fuse the frustum with the block `[−10, 0] × [−10, 10] × [floor, 12]` under
/// `rule`, and assert that the answer is one solid, that the model validates,
/// that the solid has a corner at each of the four points where `x = 0` meets
/// the rims, and that its volume is the closed form within what the mesh says
/// it may miss by.
fn is_one_solid_of_the_right_volume(floor: f64, rule: Rule, frustum_first: bool) {
    let policy = Policy::certified();
    let want = the_right_volume(floor, rule, frustum_first);
    let what = format!("{rule:?} (frustum first: {frustum_first}, floor at z = {floor})");
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let frustum = cones::a_frustum(&mut model, SLOPE, LOW, HIGH);
    let cutter = a_block(&mut model, [-10.0, -10.0, floor], [0.0, 10.0, 12.0]);
    let answer = fuse(&mut model, frustum, cutter, rule, frustum_first)
        .unwrap_or_else(|why| panic!("{what} should be one solid of volume {want}: {why:?}"));
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} does not validate: {why:?}"));
    let [solid] = answer[..] else {
        panic!("{what} is one solid: {answer:?}")
    };
    let corners = corners_of(&model, solid);
    for corner in CORNERS_DOWN_THE_AXIS {
        assert!(
            corners.contains(&corner),
            "{what} has no corner at {corner:?}: {corners:?}"
        );
    }
    let measured = model
        .solid(solid)
        .expect("a solid the fuse made")
        .measure(
            Deflection::of(0.001).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the answer measures");
    assert!(
        (measured.volume() - want).abs() <= measured.volume_error(),
        "{what} is {} against {want}, and the mesh may miss by {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The volume of the fuse of the frustum with the block
/// `[−10, 0] × [−10, 10] × [floor, 12]`. Every such block holds the frustum's
/// `x ≤ 0` half whole, so what the two share is half the frustum.
fn the_right_volume(floor: f64, rule: Rule, frustum_first: bool) -> f64 {
    let (half, block) = (half_the_frustum(), 10.0 * 20.0 * (12.0 - floor));
    match (rule, frustum_first) {
        (Rule::Union, _) => block + half,
        (Rule::Intersection, _) | (Rule::Difference, true) => half,
        (Rule::Difference, false) => block - half,
        (other, _) => panic!("{other:?} is not one of the six"),
    }
}

/// Half the frustum: `56π/3`.
fn half_the_frustum() -> f64 {
    core::f64::consts::PI / 3.0 * (4.0 * 4.0 * HIGH - 2.0 * 2.0 * LOW) / 2.0
}

fn six_fuses() -> [(Rule, bool); 6] {
    [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, true),
        (Rule::Difference, false),
    ]
}

/// Fuse `first` and `second` under `rule`, with `first` as the argument when
/// `first_goes_first` and as the tool otherwise.
fn fuse(
    model: &mut Model,
    first: SolidKey,
    second: SolidKey,
    rule: Rule,
    first_goes_first: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    let (arguments, tools) = if first_goes_first {
        ([first], [second])
    } else {
        ([second], [first])
    };
    model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}

/// The name of the predicate an `Undecidable` refusal gave up on last, and
/// `None` for any other outcome.
fn the_predicate_that_gave_up(said: &rcad_topo::Result<Vec<SolidKey>>) -> Option<&'static str> {
    match said {
        Err(Error::Undecidable { records }) => {
            records.records().last().map(|record| record.predicate())
        }
        _ => None,
    }
}

/// The block from `from` to `to`, extruded straight up from a rectangle on the
/// grid.
fn a_block(model: &mut Model, from: [f64; 3], to: [f64; 3]) -> SolidKey {
    let (across, deep) = (to[0] - from[0], to[1] - from[1]);
    let rectangle =
        ProfileBuilder::new(vec![[0.0, 0.0], [across, 0.0], [across, deep], [0.0, deep]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a rectangle is a profile");
    let placed =
        Isometry::translation(Vector::new(from[0], from[1], from[2])).expect("a finite move");
    model
        .add_extrusion(&rectangle, to[2] - from[2], &placed)
        .expect("a rectangle extrudes")
}

/// Every corner of a solid, as the point it stands at.
fn corners_of(model: &Model, solid: SolidKey) -> Vec<[f64; 3]> {
    let mut corners = Vec::new();
    for face in model.solid(solid).expect("a solid the fuse made").faces() {
        for wire in face.wires() {
            for step in wire.steps() {
                let WireStepView::Along(coedge) = step else {
                    panic!("these answers are made of coedges and nothing else: {step:?}")
                };
                corners.push(coedge.edge().start().point().coordinates());
            }
        }
    }
    corners
}

/// The cone the frustum is cut out of.
fn the_cone() -> Cone {
    Cone::new(Axis::z(), SLOPE).expect("a positive slope")
}

/// The half frustum's flat face's own plane, facing out of it.
fn the_cut_plane() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal")
}

/// The `x ≥ 0` half of the frustum, described face by face through the public
/// builder, with its two rulings taken from [`intersect`] rather than spelled
/// out here. The same solid `a_flat_cap_cut_by_a_cones_rim.rs` builds.
fn a_half_frustum(model: &mut Model) -> SolidKey {
    let runs = the_two_rulings();
    let up_the_positive_side = the_ruling_through(&runs, 2.0);
    let up_the_negative_side = the_ruling_through(&runs, -2.0);

    let mut builder = SolidBuilder::new();
    let corner = |builder: &mut SolidBuilder, y: f64, z: f64| {
        builder
            .add_vertex(Point::new(0.0, y, z))
            .expect("a finite point")
    };
    let bottom_negative = corner(&mut builder, -SLOPE * LOW, LOW);
    let bottom_positive = corner(&mut builder, SLOPE * LOW, LOW);
    let top_positive = corner(&mut builder, SLOPE * HIGH, HIGH);
    let top_negative = corner(&mut builder, -SLOPE * HIGH, HIGH);

    let round_the_front = || EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(the_cut_plane()),
        picked_by: smallvec::smallvec![Picker::from(
            HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
                .expect("the x axis has length")
        )],
    };
    let bottom_arc = builder
        .add_edge(bottom_positive, bottom_negative, round_the_front())
        .expect("half of the small rim");
    let top_arc = builder
        .add_edge(top_negative, top_positive, round_the_front())
        .expect("half of the big rim");
    let up_the_positive = builder
        .add_edge(bottom_positive, top_positive, up_the_positive_side)
        .expect("a piece of the ruling at y > 0");
    let up_the_negative = builder
        .add_edge(bottom_negative, top_negative, up_the_negative_side)
        .expect("a piece of the ruling at y < 0");
    let bottom_across = builder
        .add_edge(bottom_negative, bottom_positive, EdgeRun::Straight)
        .expect("the small cap's straight side");
    let top_across = builder
        .add_edge(top_positive, top_negative, EdgeRun::Straight)
        .expect("the big cap's straight side");

    let wire_of = |builder: &mut SolidBuilder, steps: &[(_, Orientation)]| {
        let along: Vec<DraftStep> = steps
            .iter()
            .map(|(edge, turn)| {
                DraftStep::Along(
                    builder
                        .add_coedge(*edge, *turn)
                        .expect("an edge of this build"),
                )
            })
            .collect();
        builder.add_wire(&along).expect("a closed wire")
    };
    let bottom_wire = wire_of(
        &mut builder,
        &[
            (bottom_arc, Orientation::Forward),
            (bottom_across, Orientation::Forward),
        ],
    );
    let top_wire = wire_of(
        &mut builder,
        &[
            (top_arc, Orientation::Forward),
            (top_across, Orientation::Forward),
        ],
    );
    let skin_wire = wire_of(
        &mut builder,
        &[
            (bottom_arc, Orientation::Reversed),
            (up_the_positive, Orientation::Forward),
            (top_arc, Orientation::Reversed),
            (up_the_negative, Orientation::Reversed),
        ],
    );
    let cut_wire = wire_of(
        &mut builder,
        &[
            (up_the_negative, Orientation::Forward),
            (top_across, Orientation::Reversed),
            (up_the_positive, Orientation::Reversed),
            (bottom_across, Orientation::Reversed),
        ],
    );

    let cap = |builder: &mut SolidBuilder, at: f64, outward: f64, wire| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, at), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a half-disc cap")
    };
    let bottom = cap(&mut builder, LOW, -1.0, bottom_wire);
    let top = cap(&mut builder, HIGH, 1.0, top_wire);
    let side = builder
        .add_face(Surface::Cone(the_cone()), &[skin_wire])
        .expect("half the cone's skin");
    let flat = builder
        .add_face(Surface::Plane(the_cut_plane()), &[cut_wire])
        .expect("the flat face down the axis");

    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
            (flat, Orientation::Forward),
        ])
        .expect("four faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("half a frustum described this way is a solid")
}

/// The two runs [`intersect`] names for the cut plane against the cone.
fn the_two_rulings() -> Vec<NamedRun> {
    let SurfaceMeet::Curves(runs) = intersect(
        &Surface::Plane(the_cut_plane()),
        &Surface::Cone(the_cone()),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a plane down a cone's axis cuts it in two straight lines") else {
        panic!("two rulings are two curves");
    };
    assert_eq!(runs.len(), 2);
    runs
}

/// Whichever of the two rulings holds `(0, y, 4)` and not its mirror, which is
/// the pickers' to say and not the order's.
fn the_ruling_through(runs: &[NamedRun], y: f64) -> EdgeRun {
    let holds = |run: &NamedRun, at: [f64; 3]| {
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
            panic!("a ruling through the apex is a piece of where the two surfaces cross");
        };
        picked_by
            .iter()
            .all(|picker| picker.side_of_point(PredicatePoint::from(at)) != Decision::Neg)
    };
    runs.iter()
        .find(|run| holds(run, [0.0, y, LOW]) && !holds(run, [0.0, -y, LOW]))
        .expect("one of the two runs holds that ruling and not the other")
        .run()
        .clone()
}
