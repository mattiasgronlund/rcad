//! A ball cut into a wedge by planes that all hold its polar axis.
//!
//! What is left of the ball's skin is a **lune**: a piece of the sphere between
//! two half great circles, with a corner at each pole. §195.1 made the half-turn
//! lune answer — `mesh.rs`'s `arrive_back_at_the_pole_the_walk_set_off_from`
//! writes the pole corner the walk was missing when it arrives back at the pole
//! it set off from — and said what it did not measure: a lune **narrower than
//! half a turn** that sets off from a pole corner now gets that corner written
//! twice. Every test here aims at that lune.
//!
//! Since §200 some wedges are answered, and each of those is measured against
//! its closed form. Of the 48 cases of each family, wedge by size by rule, 39
//! answer cut by two knives with `x >= 0` first, 45 with the turned knife first
//! (since Q1024 reads the side of a half ball's skin off its wire), and all 48
//! cut by one prism. The rest are refused by name. A case that moves either way
//! fails, naming itself.
//!
//! The ball's seam is the `+x` half of the meridian in `y = 0`, so a knife
//! turned a quarter turn about the axis lies along the seam.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_mesh::CannotTriangulate;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// How wide the wedges are, in degrees. All narrower than half a turn.
const WEDGES: [f64; 4] = [90.0, 45.0, 10.0, 170.0];

/// The radii of the balls.
const RADII: [f64; 4] = [1.0, 0.001, 1000.0, 3.0];

/// A wedge's width in degrees and the ball's radius, to the volume a rule's
/// answer should have.
type ClosedForm = fn(f64, f64) -> f64;

/// How fine a mesh every volume here is read off, as a share of the radius.
///
/// A deflection is an absolute length, so it is scaled with the ball: every
/// size is then meshed with the same number of steps round a great circle, and
/// one bound holds for all four.
const DEFLECTION_PER_RADIUS: f64 = 0.001;

/// How far under the closed form a measured volume may fall.
///
/// Every point of an inscribed mesh sits within the deflection `δ` of the
/// sphere, so the shell it leaves out is at most the curved area `A` times `δ`.
/// A slice of a ball bounded by planes through its centre has volume `A·r/3`,
/// so the share missing is at most `3δ/r`. The flat faces are meshed exactly.
/// For the union the missing shell is the same and the whole is larger, so the
/// same bound is looser still there.
const HOW_FAR_UNDER: f64 = 3.0 * DEFLECTION_PER_RADIUS;

/// A wedge cut by two separate knives, each a block with one face through the
/// polar axis: first the half ball `x >= 0`, then a knife turned so that what
/// the two keep between them is `degrees` wide.
///
/// This is the lune §195.1 did not measure, built the way its own half-turn
/// test builds the half-turn one, one plane at a time. The first cut answers at
/// every size. The turned knife's stored plane misses the poles by a rounding
/// (see [`a_knife_through_the_axis`]). At 90 degrees that sliver was where a
/// flat piece of `x = 0` was placed on the wrong side of the knife, which ended
/// in `ResultIsNotManifold` (§200.1); see
/// [`the_quarter_turned_knife_holds_the_axis_by_its_corners_and_places_a_point_by_its_plane`].
/// Since the fuse drops a point the ray and the stored planes disagree about,
/// the 90 degree wedge answers at every size as an intersection and a union,
/// and so does the 10 degree one at radii 0.001 and 1000. The union needed one
/// more step: the knife's face then has a half disk hole, two corners that
/// wind round nothing, and `assemble.rs`'s `rings_round` panicked on it. The
/// difference is refused at 90 degrees, for the range since Q1664. It panicked
/// in `arrange.rs`'s `cut_the_crossings_into_arcs` until that walked round a
/// closed crossing, §197.
///
/// Since the fuse steps a curved piece's point towards every cut's material
/// (§200), the cases in `answered` get past the curved pieces and answer, and
/// each is measured against its closed form. Since §330, the 170 degree
/// wedge's difference and union answer at every size, and so do the 10 degree
/// wedge's at radii 1 and 3. Most of the rest
/// now reach the flat pieces and assembly and are refused there instead. A case
/// that moves either way fails, naming itself.
///
/// **No case stops at `order along a crossing` since Q901**, which orders two
/// meets of three planes that share two by the planes when their boxes cannot
/// say. None answers for it: all twelve 45 degree cases, every size and every
/// rule, are refused at *a point inside a piece of a face* instead, and that
/// name takes the old one's place in [`moved_from_its_pin`].
///
/// **Ten of those twelve answer since Q1019**, which places a piece no point
/// was found in by a corner the other solid's ray puts strictly inside or
/// outside it (`docs/decisions.md` §365.7). Each is one solid that validates
/// and measures its closed form. The difference at radii 0.001 and 1000 is
/// refused for the range.
///
/// **Q1614 moved two differences**, the 45 degree wedge's at radius 1000 and
/// the 10 degree wedge's at radius 0.001, from
/// `CannotNameWhichPieceOfACrossingThisIs` to `DynamicRangeTooWide`, by asking
/// the range before nesting the answer's shells (`docs/decisions.md` §87.3).
/// **Q1664 moved seven more**, from
/// `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` to `DynamicRangeTooWide`, by
/// asking the range before the answer's edges are named, so no case is refused
/// that name now and it is no longer one this file accepts.
/// [`two_wedges_cut_by_two_knives_are_refused_their_difference_for_the_range`]
/// pins all nine by ratio.
#[test]
fn a_wedge_cut_from_a_ball_by_two_knives_through_its_polar_axis_is_answered_or_refused_by_name() {
    let answered = [
        (90.0, 1.0, "Intersection"),
        (90.0, 1.0, "Union"),
        (90.0, 0.001, "Intersection"),
        (90.0, 0.001, "Union"),
        (90.0, 1000.0, "Intersection"),
        (90.0, 1000.0, "Union"),
        (90.0, 3.0, "Intersection"),
        (90.0, 3.0, "Union"),
        (45.0, 1.0, "Intersection"),
        (45.0, 1.0, "Difference"),
        (45.0, 1.0, "Union"),
        (45.0, 0.001, "Intersection"),
        (45.0, 0.001, "Union"),
        (45.0, 1000.0, "Intersection"),
        (45.0, 1000.0, "Union"),
        (45.0, 3.0, "Intersection"),
        (45.0, 3.0, "Difference"),
        (45.0, 3.0, "Union"),
        (10.0, 1.0, "Difference"),
        (10.0, 1.0, "Union"),
        (10.0, 0.001, "Intersection"),
        (10.0, 0.001, "Union"),
        (10.0, 1000.0, "Intersection"),
        (10.0, 1000.0, "Union"),
        (10.0, 3.0, "Intersection"),
        (10.0, 3.0, "Difference"),
        (10.0, 3.0, "Union"),
        (170.0, 1.0, "Intersection"),
        (170.0, 1.0, "Difference"),
        (170.0, 1.0, "Union"),
        (170.0, 0.001, "Intersection"),
        (170.0, 0.001, "Difference"),
        (170.0, 0.001, "Union"),
        (170.0, 1000.0, "Intersection"),
        (170.0, 1000.0, "Difference"),
        (170.0, 1000.0, "Union"),
        (170.0, 3.0, "Intersection"),
        (170.0, 3.0, "Difference"),
        (170.0, 3.0, "Union"),
    ];
    let mut moved = Vec::new();
    for degrees in WEDGES {
        for radius in RADII {
            for (rule, closed_form) in what_two_knives_leave() {
                let mut model = a_model(radius);
                let ball = model
                    .add_ball(radius, &Isometry::identity())
                    .expect("a ball");
                let first = a_knife_through_the_axis(&mut model, radius, 0.0);
                let half = fused(&mut model, &[ball], &[first], Rule::Intersection);
                let second = a_knife_through_the_axis(&mut model, radius, 180.0 - degrees);
                let case = (degrees, radius, rule);
                let outcome = model.fuse(
                    &half,
                    &[second],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                if let Some(how) = moved_from_its_pin(
                    &mut model,
                    outcome,
                    is_pinned_as_answered(&answered, case),
                    radius,
                    closed_form(degrees, radius),
                ) {
                    moved.push(format!("{case:?}: {how}"));
                }
            }
        }
    }
    assert!(moved.is_empty(), "{}", moved.join("\n"));
}

/// Nine of the cases [`a_wedge_cut_from_a_ball_by_two_knives_through_its_polar_axis_is_answered_or_refused_by_name`]
/// lets through as refused by name are refused for the range: the 90 degree
/// wedge's difference at every radius, the 45 degree wedge's at radii 0.001
/// and 1000, and the 10 degree wedge's intersection at radius 1 and its
/// difference at radii 0.001 and 1000.
///
/// **Q1614 moved two of them**, the 45 degree difference at radius 1000 and
/// the 10 degree one at radius 0.001. Until then the fuse nested the answer's
/// shells before it asked the range, and both were refused
/// `CannotNameWhichPieceOfACrossingThisIs` there (`docs/decisions.md` §87.3).
///
/// **Q1664 moved the other seven.** The arc where the ball crosses the first
/// knife's plane `x = 0` runs from a pole to a corner a rounding off it, such
/// as `(0, 0, −1)` to `(0, 4.4e−16, −1)`, so its chord lies along the tangent
/// at the pole and nothing could say which way the arc leaves it. Each was
/// refused `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` while the edges
/// were named, before the range was asked. The fuse now asks the range first,
/// of the answer's corners and edges as they are once its pieces are joined,
/// and a chord a rounding long beside a coordinate of 1 is the limit §87.3
/// names. A tolerant policy only warns of the range, and there the arc is named
/// since Q1867 and the seven are refused further on
/// ([`two_wedges_cut_by_two_knives_are_refused_by_name_under_a_tolerant_policy`]).
#[test]
fn two_wedges_cut_by_two_knives_are_refused_their_difference_for_the_range() {
    for (degrees, radius, rule, ratio) in [
        (90.0, 1.0, Rule::Difference, 1.351_079_888_211_149e16),
        (90.0, 0.001, Rule::Difference, 1.290_583_773_813_634_8e16),
        (90.0, 1000.0, Rule::Difference, 1.419_201_563_247_005_2e16),
        (90.0, 3.0, Rule::Difference, 4.053_239_664_633_447e16),
        (45.0, 0.001, Rule::Difference, 6_917_529_027_641_085.0),
        (45.0, 1000.0, Rule::Difference, 1.319_413_953_331_200_8e16),
        (10.0, 1.0, Rule::Intersection, 5_435_766_241_772_876.0),
        (10.0, 0.001, Rule::Difference, 5_566_224_631_575_426.0),
        (10.0, 1000.0, Rule::Difference, 5_308_365_470_481_324.0),
    ] {
        let mut model = a_model(radius);
        let ball = model
            .add_ball(radius, &Isometry::identity())
            .expect("a ball");
        let first = a_knife_through_the_axis(&mut model, radius, 0.0);
        let half = fused(&mut model, &[ball], &[first], Rule::Intersection);
        let second = a_knife_through_the_axis(&mut model, radius, 180.0 - degrees);
        let mut log = ProvenanceLog::new();
        let outcome = model.fuse(&half, &[second], rule, &Policy::certified(), &mut log);
        let Err(Error::Invalid(rcad_topo::Invalid::DynamicRangeTooWide { ratio: found })) = outcome
        else {
            panic!("{degrees} degrees, radius {radius}, {rule:?}: {outcome:?}");
        };
        assert_eq!(found, ratio, "{degrees} degrees, radius {radius}, {rule:?}");
        assert!(
            log.records().is_empty(),
            "{degrees} degrees, radius {radius}, {rule:?}: a certified fuse logs nothing: {:#?}",
            log.records()
        );
    }
}

/// The same nine cases under a tolerant policy, where a range too wide is only
/// a warning, so each goes on past the range check and is refused by name
/// further on. Each is past the range §87.3 names: the arc of `x = 0` from a
/// pole to a corner a rounding off it is a chord `4.4e−16` long beside a
/// coordinate of 1.
///
/// **The seven Q1664 moved were refused
/// `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` until Q1867**, as all
/// seven were under every policy before Q1664: that arc has its chord along
/// the tangent at the pole. **Q1614's two were refused
/// `CannotNameWhichPieceOfACrossingThisIs` until Q2081**, as they were before
/// Q1614. Their arc from a pole to the corner a rounding off it has its
/// chord along the tangent too, and the crossing at the pole, squared off
/// against that chord, came out not nought but a rounding along it:
/// `(0, ±4.7e−38, 0)` at radius 1000 and `(0, ±3.3e−55, 0)` at radius 0.001.
/// The fuse took that rounding as the way into the arc, and a side of the
/// chord read along the chord names no one piece of the circle. Since Q2081
/// `Edge::the_way_across_the_chord_into_the_arc` faces an arc of a closed
/// curve whose two ends square off the same way by which way it sets off
/// along its chord, here away from the ball's centre.
///
/// **Q1867 names that arc** (`Edge::the_way_into_an_arc_along_its_own_chord`
/// in `fuse/assemble.rs`) and none of the seven answers for it, nor does
/// either of Q1614's two for Q2081. Each answer's corners, edges and faces are
/// right — the 90 degree difference at radius 1 and the 10 degree one at
/// radius 0.001 were dumped, the arcs from each pole to the corner beside it
/// picked as the short ones — and each is refused further on:
///
/// - **the 90 degree differences at radii 1 and 1000 and the 45 degree one at
///   radius 1000 for corners that coincide**, by the mesher, which sees the
///   pole and the corner a rounding from it as one point on the face's own map.
///   Until Q2290 all four 90 degree ones were refused so on the sphere's lune.
///   Since then `mesh.rs` cuts a loop that names one place twice into lobes
///   (`what_the_triangulator_can_cut`), so the lune is meshed, and the 90 degree
///   ones stop further on, at the flat face of `x = 0`: at radii 1 and 1000 for
///   corners that coincide there;
/// - **the 90 degree difference at radius 0.001 for loops that touch**, at that
///   flat face, at `(0, 4.6e−19, 0.001)` beside the north pole;
/// - **the 90 degree difference at radius 3 for two patches drawing one
///   side**. The lune and the flat face of `x = 0` meet at the south pole `P`
///   along two sides, from `P` to a second entry `P′` of the pole and from
///   `P′` to the corner `C` `4.4e−16` from it, and each face names that place
///   twice in its loop. Each cuts a lobe off there, and the side `P C` that
///   closes the lobe it keeps lies inside its patch: drawn by both patches, it
///   is used four times (`TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4,
///   used: 4 }`);
/// - **the 45 degree one at radius 0.001 and the 10 degree ones at radii 0.001
///   and 1000 for faces that do not agree which way they face**. The faces are
///   right and the name is not. The walk puts a point on the arc of `x = 0`
///   from the north pole to the corner beside it, and the flat face of
///   `x = 0` goes through that point, but the coarsest mesh of the sphere's
///   face draws the pole straight to the corner and skips it, so the sides
///   there are not each used once both ways;
/// - **the 10 degree intersection at radius 1 for loops that touch** at
///   `(0, −6.4e−16, −1)`, where the two knives' line meets the ball a rounding
///   past the south pole. Before Q1866 `validate.rs`'s
///   `the_coarsest_mesh_whose_sides_do_not_cross` walked its ladder down
///   endlessly there, a rung refused at the same corner each time; it now stops
///   at the second.
#[test]
fn two_wedges_cut_by_two_knives_are_refused_by_name_under_a_tolerant_policy() {
    const COINCIDE: &str = "CornersCoincide";
    const FACE_APART: &str = "FacesDoNotAgreeWhichWayTheyFace";
    const TOUCH: &str = "LoopsTouch";
    const ONE_SIDE_TWICE: &str = "TwoPatchesOfTheMeshDrewTheSameSide";
    for (degrees, radius, rule, refused) in [
        (90.0, 1.0, Rule::Difference, COINCIDE),
        (90.0, 0.001, Rule::Difference, TOUCH),
        (90.0, 1000.0, Rule::Difference, COINCIDE),
        (90.0, 3.0, Rule::Difference, ONE_SIDE_TWICE),
        (45.0, 0.001, Rule::Difference, FACE_APART),
        (45.0, 1000.0, Rule::Difference, COINCIDE),
        (10.0, 1.0, Rule::Intersection, TOUCH),
        (10.0, 0.001, Rule::Difference, FACE_APART),
        (10.0, 1000.0, Rule::Difference, FACE_APART),
    ] {
        let mut model = a_model(radius);
        let ball = model
            .add_ball(radius, &Isometry::identity())
            .expect("a ball");
        let first = a_knife_through_the_axis(&mut model, radius, 0.0);
        let half = fused(&mut model, &[ball], &[first], Rule::Intersection);
        let second = a_knife_through_the_axis(&mut model, radius, 180.0 - degrees);
        let policy = Policy::tolerant(model.scale().nominal_size());
        let outcome = model.fuse(&half, &[second], rule, &policy, &mut ProvenanceLog::new());
        let Err(why) = outcome else {
            panic!("{degrees} degrees, radius {radius}, {rule:?}: answered {outcome:?}");
        };
        assert_eq!(
            what_stops_the_mesher_or_which_refusal(&why),
            refused,
            "{degrees} degrees, radius {radius}, {rule:?}: {why:?}"
        );
    }
}

/// A half ball `x ≥ 0` cut by a block whose face `y = 1e−10·r` misses the
/// polar axis by a hair answers at every size and under every rule, and
/// measures its closed form.
///
/// The difference keeps the arc of `x = 0` from each pole to the corner where
/// the block's face crosses it, `1e−10·r` round. The `z` of that corner is
/// `r` to every bit an `f64` has, so the chord lies exactly along the tangent
/// at the pole, and the crossing at the pole runs exactly along the chord: the
/// two directions the fuse tells the arcs apart by both square off to nothing.
/// That difference was refused `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed`
/// at all four sizes until Q1867, under a certified policy and with the range
/// well inside §87.3's. The arc sets off from the pole towards the corner, so
/// it is the short one, on the side of the chord away from the ball's centre.
///
/// With the slab `0 ≤ y ≤ t` of the half ball, `t = 1e−10·r`, of volume
/// `(π/2)(r²t − t³/3)`, the difference is the quarter ball and the slab, the
/// intersection is the quarter ball less the slab, and the union is the block
/// and the half ball less that intersection.
#[test]
fn a_half_ball_cut_by_a_block_a_hair_off_its_polar_axis_is_answered() {
    a_half_ball_cut_by_a_block_off_its_polar_axis_is_answered(1e-10);
}

/// The same half ball and block with the block's face at `y = 1e−8·r`, which
/// answers at every size and under every rule and measures its closed form.
///
/// Q2012 found the difference refused `CannotNameWhichPieceOfACrossingThisIs`
/// under a certified policy at radii 0.001, 1000 and 3, where at `1e−10·r` and
/// `1e−7·r` it answered.
#[test]
fn a_half_ball_cut_by_a_block_a_hundred_hairs_off_its_polar_axis_is_answered() {
    a_half_ball_cut_by_a_block_off_its_polar_axis_is_answered(1e-8);
}

/// The half ball `x ≥ 0` of every radius cut by every rule with a block whose
/// face is `y = off·r`, each answer one solid that validates and measures its
/// closed form.
fn a_half_ball_cut_by_a_block_off_its_polar_axis_is_answered(off: f64) {
    for radius in RADII {
        let t = off * radius;
        let slab = PI / 2.0 * (radius * radius * t - t * t * t / 3.0);
        let quarter = a_balls_volume(radius) / 4.0;
        for (rule, closed_form) in [
            (Rule::Difference, quarter + slab),
            (Rule::Intersection, quarter - slab),
            (
                Rule::Union,
                12.0 * 6.0 * 12.0 * radius.powi(3) + quarter + slab,
            ),
        ] {
            let mut model = a_model(radius);
            let ball = model
                .add_ball(radius, &Isometry::identity())
                .expect("a ball");
            let first = a_knife_through_the_axis(&mut model, radius, 0.0);
            let half = fused(&mut model, &[ball], &[first], Rule::Intersection);
            let knife = block(
                &mut model,
                [12.0 * radius, 6.0 * radius, 12.0 * radius],
                Vector::new(-6.0 * radius, t, -6.0 * radius),
            );
            let answer = model
                .fuse(
                    &half,
                    &[knife],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("radius {radius}, {rule:?}: refused {why:?}"))
                .solids()
                .to_vec();
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("radius {radius}, {rule:?}: {why:?}"));
            assert_eq!(answer.len(), 1, "radius {radius}, {rule:?}");
            measures(volume_of(&model, &answer, radius), closed_form);
        }
    }
}

/// Why the 90 degree wedge was refused before it answered: the knife turned a
/// quarter turn holds the axis by its corners and misses it by its stored
/// plane, so a point in between was inside by the corners and is outside by
/// the plane.
///
/// `(0, 1.1e-16, -0.5)` is the first point the fuse tried inside the half disk
/// of `x = 0` that the knife does not cover. The fuse cuts along the stored
/// plane, which puts that half disk on the outside, and the point with it. The
/// ray fans the face's corners, `(∓6, ±8.9e-16, z)`, whose triangles hold the
/// axis exactly, and it once certified the point inside by them. Since Q668
/// the ray asks the stored plane too where the two part, and puts the point
/// outside with it (§346). Since Q996, `select.rs` no longer drops such a
/// point itself: its own copy of the guard,
/// `the_ray_and_the_stored_planes_disagree_about`, is gone, and `select.rs`
/// reads classify.rs's copies instead.
#[test]
fn the_quarter_turned_knife_holds_the_axis_by_its_corners_and_places_a_point_by_its_plane() {
    let mut model = a_model(1.0);
    let knife = a_knife_through_the_axis(&mut model, 1.0, 90.0);
    let view = model.solid(knife).expect("this model made the key");
    let point = rcad_predicates::Point::from([0.0, 1.110_223_024_625_156_5e-16, -0.5]);
    let through_the_axis = view
        .faces()
        .find(|face| {
            let mut corners = face
                .outer_wire()
                .points()
                .map(|corner| corner.coordinates());
            corners.all(|[_, y, _]| y.abs() < 1.0)
        })
        .expect("the knife has a face through the axis");
    let corners: Vec<[f64; 3]> = through_the_axis
        .outer_wire()
        .points()
        .map(|corner| corner.coordinates())
        .collect();
    assert!(
        corners
            .iter()
            .all(|corner| corner[1] == -corner[0] * 8.881_784_197_001_252e-16 / 6.0),
        "the corners hold the axis exactly: {corners:?}"
    );
    assert_eq!(
        through_the_axis.plane().side_of_point(point),
        rcad_predicates::Decision::Pos,
        "the stored plane puts the point outside the knife"
    );
    assert_eq!(
        view.classify_uncertain_point(point, &Policy::certified(), &mut ProvenanceLog::new()),
        rcad_topo::Containment::Outside,
        "the ray puts it outside, as the stored plane does"
    );
}

/// The same two knives, the turned one first.
///
/// For a wedge of 90 degrees the turned knife lies along the seam. The first
/// cut answers at every angle and size here, as
/// [`a_ball_cut_down_its_polar_axis_off_its_seam_is_answered_or_refused_by_name`]
/// pins, and that half ball is then cut again by `x >= 0`.
///
/// **The second cut was refused at every angle and size until Q1024**, naming
/// the half ball's skin as a face whose trimming nothing here follows. That
/// skin is a lune whose only corners are the poles, and both lie on the turned
/// knife's face, so no corner said which side of that face the skin keeps; a
/// point inside a piece of the second knife was found, and the ray that was to
/// place it against the half ball met the skin. The skin's side is now read
/// off the way its wire walks the rim (`holds.rs`'s
/// `the_side_its_wire_keeps_along`), and every case in `answered` is one solid
/// that validates and measures its closed form, the same three
/// [`what_two_knives_leave`] gives the other order.
///
/// **The 90 degree wedge of the ball of radius 0.001 is refused
/// `ResultIsNotManifold { count: 1 }` under every rule**, where the same wedge
/// answers at radii 1, 3 and 1000. Not traced.
#[test]
fn a_wedge_cut_with_the_turned_knife_first_is_answered_or_refused_by_name() {
    let answered = [
        (90.0, 1.0, "Intersection"),
        (90.0, 1.0, "Difference"),
        (90.0, 1.0, "Union"),
        (90.0, 1000.0, "Intersection"),
        (90.0, 1000.0, "Difference"),
        (90.0, 1000.0, "Union"),
        (90.0, 3.0, "Intersection"),
        (90.0, 3.0, "Difference"),
        (90.0, 3.0, "Union"),
        (45.0, 1.0, "Intersection"),
        (45.0, 1.0, "Difference"),
        (45.0, 1.0, "Union"),
        (45.0, 0.001, "Intersection"),
        (45.0, 0.001, "Difference"),
        (45.0, 0.001, "Union"),
        (45.0, 1000.0, "Intersection"),
        (45.0, 1000.0, "Difference"),
        (45.0, 1000.0, "Union"),
        (45.0, 3.0, "Intersection"),
        (45.0, 3.0, "Difference"),
        (45.0, 3.0, "Union"),
        (10.0, 1.0, "Intersection"),
        (10.0, 1.0, "Difference"),
        (10.0, 1.0, "Union"),
        (10.0, 0.001, "Intersection"),
        (10.0, 0.001, "Difference"),
        (10.0, 0.001, "Union"),
        (10.0, 1000.0, "Intersection"),
        (10.0, 1000.0, "Difference"),
        (10.0, 1000.0, "Union"),
        (10.0, 3.0, "Intersection"),
        (10.0, 3.0, "Difference"),
        (10.0, 3.0, "Union"),
        (170.0, 1.0, "Intersection"),
        (170.0, 1.0, "Difference"),
        (170.0, 1.0, "Union"),
        (170.0, 0.001, "Intersection"),
        (170.0, 0.001, "Difference"),
        (170.0, 0.001, "Union"),
        (170.0, 1000.0, "Intersection"),
        (170.0, 1000.0, "Difference"),
        (170.0, 1000.0, "Union"),
        (170.0, 3.0, "Intersection"),
        (170.0, 3.0, "Difference"),
        (170.0, 3.0, "Union"),
    ];
    let mut moved = Vec::new();
    for degrees in WEDGES {
        for radius in RADII {
            for (rule, closed_form) in what_two_knives_leave() {
                let mut model = a_model(radius);
                let half = the_ball_cut_by_the_turned_knife(&mut model, radius, degrees);
                let second = a_knife_through_the_axis(&mut model, radius, 0.0);
                let case = (degrees, radius, rule);
                let outcome = model.fuse(
                    &half,
                    &[second],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                if let Some(how) = moved_from_its_pin(
                    &mut model,
                    outcome,
                    is_pinned_as_answered(&answered, case),
                    radius,
                    closed_form(degrees, radius),
                ) {
                    moved.push(format!("{case:?}: {how}"));
                }
            }
        }
    }
    assert!(moved.is_empty(), "{}", moved.join("\n"));
}

/// The half ball the quarter-turned knife leaves places a point on the second
/// knife's face `x = 0` inside it, and a point past its skin outside it.
///
/// This is the point the quarter wedge stopped at until Q1024: a point inside
/// a piece of the second knife **was** found, and the ray that was to place it
/// met the half ball's skin, whose corners are both on the turned knife's face
/// and say nothing about which side of it the skin keeps. `classify` came back
/// `CannotFollowAFacesTrimming` naming that skin, and the fuse refused for it.
/// The skin's side is now read off its wire, and the point is inside.
#[test]
fn the_half_ball_places_a_point_on_the_second_knifes_face_inside_it() {
    let mut model = a_model(1.0);
    let half = the_ball_cut_by_the_turned_knife(&mut model, 1.0, 90.0);
    let [half] = half[..] else {
        panic!("the first cut is one solid, not {half:?}");
    };
    let view = model.solid(half).expect("this model made the key");
    for (point, placed) in [
        ([0.0, 0.5, 0.25], rcad_topo::Containment::Inside),
        ([0.0, 0.75, 0.75], rcad_topo::Containment::Outside),
    ] {
        assert_eq!(
            view.classify_uncertain_point(
                rcad_predicates::Point::from(point),
                &Policy::certified(),
                &mut ProvenanceLog::new()
            ),
            placed,
            "the half ball y >= 0 places {point:?}"
        );
    }
}

/// What the two knives leave of the ball under each rule, whichever cuts
/// first: the wedge `degrees` wide, the rest of the half ball, and that rest
/// with the whole knife `x >= 0`.
fn what_two_knives_leave() -> [(Rule, ClosedForm); 3] {
    [
        (Rule::Intersection, |degrees, radius| {
            degrees / 360.0 * a_balls_volume(radius)
        }),
        (Rule::Difference, |degrees, radius| {
            (180.0 - degrees) / 360.0 * a_balls_volume(radius)
        }),
        (Rule::Union, |degrees, radius| {
            (180.0 - degrees) / 360.0 * a_balls_volume(radius) + a_knifes_volume(radius)
        }),
    ]
}

/// The ball of `radius` in `model`, less the knife turned `180 − degrees`: a
/// half ball, which [`a_wedge_cut_with_the_turned_knife_first_is_answered_or_refused_by_name`]
/// then cuts by the knife `x >= 0`.
fn the_ball_cut_by_the_turned_knife(model: &mut Model, radius: f64, degrees: f64) -> Vec<SolidKey> {
    let ball = model
        .add_ball(radius, &Isometry::identity())
        .expect("a ball");
    let turned = a_knife_through_the_axis(model, radius, 180.0 - degrees);
    fused(model, &[ball], &[turned], Rule::Intersection)
}

/// A wedge cut in one fuse, by a prism whose edge **is** the polar axis.
///
/// This is not the lune §195.1 is about: the poles sit on the knife's edge
/// rather than on its face, which the fuse sees before it sees any lune. The
/// intersections in `answered` answer and measure since §200; a case left out
/// of it would have to be refused by name. The quarter wedge of the ball of radius 1000 answers since
/// the triangulator takes a corner under spade's floor as zero (§321.6). The
/// difference and union answer wherever the intersection does since §330,
/// which names an arc of the ball's crossing with a prism face by that face's
/// plane as well.
///
/// **Every case answers since §340.10.** Eighteen refused
/// `ResultIsNotManifold`: the ball meets the prism's edge at its poles, and the
/// box round a prism face stopped a rounding short of those two meets — at
/// radius `0.001` the face's box began at `y = 0` and both poles came out at
/// `y = −1.78e−35` — so the ball's cut across that face was dropped as missing
/// it.
#[test]
fn a_wedge_cut_by_one_prism_whose_edge_is_the_polar_axis_is_answered_or_refused_by_name() {
    let answered = [
        (90.0, 1.0, "Intersection"),
        (90.0, 1.0, "Difference"),
        (90.0, 1.0, "Union"),
        (90.0, 0.001, "Intersection"),
        (90.0, 0.001, "Difference"),
        (90.0, 0.001, "Union"),
        (90.0, 1000.0, "Intersection"),
        (90.0, 1000.0, "Difference"),
        (90.0, 1000.0, "Union"),
        (90.0, 3.0, "Intersection"),
        (90.0, 3.0, "Difference"),
        (90.0, 3.0, "Union"),
        (45.0, 1.0, "Intersection"),
        (45.0, 1.0, "Difference"),
        (45.0, 1.0, "Union"),
        (45.0, 0.001, "Intersection"),
        (45.0, 0.001, "Difference"),
        (45.0, 0.001, "Union"),
        (45.0, 1000.0, "Intersection"),
        (45.0, 1000.0, "Difference"),
        (45.0, 1000.0, "Union"),
        (45.0, 3.0, "Intersection"),
        (45.0, 3.0, "Difference"),
        (45.0, 3.0, "Union"),
        (10.0, 1.0, "Intersection"),
        (10.0, 1.0, "Difference"),
        (10.0, 1.0, "Union"),
        (10.0, 0.001, "Intersection"),
        (10.0, 0.001, "Difference"),
        (10.0, 0.001, "Union"),
        (10.0, 1000.0, "Intersection"),
        (10.0, 1000.0, "Difference"),
        (10.0, 1000.0, "Union"),
        (10.0, 3.0, "Intersection"),
        (10.0, 3.0, "Difference"),
        (10.0, 3.0, "Union"),
        (170.0, 1.0, "Intersection"),
        (170.0, 1.0, "Difference"),
        (170.0, 1.0, "Union"),
        (170.0, 0.001, "Intersection"),
        (170.0, 0.001, "Difference"),
        (170.0, 0.001, "Union"),
        (170.0, 1000.0, "Intersection"),
        (170.0, 1000.0, "Difference"),
        (170.0, 1000.0, "Union"),
        (170.0, 3.0, "Intersection"),
        (170.0, 3.0, "Difference"),
        (170.0, 3.0, "Union"),
    ];
    let mut moved = Vec::new();
    let rules: [(Rule, ClosedForm); 3] = [
        (Rule::Intersection, |degrees, radius| {
            degrees / 360.0 * a_balls_volume(radius)
        }),
        (Rule::Difference, |degrees, radius| {
            (360.0 - degrees) / 360.0 * a_balls_volume(radius)
        }),
        (Rule::Union, |degrees, radius| {
            (360.0 - degrees) / 360.0 * a_balls_volume(radius) + a_prisms_volume(radius, degrees)
        }),
    ];
    for degrees in WEDGES {
        for radius in RADII {
            for (rule, closed_form) in rules {
                let mut model = a_model(radius);
                let ball = model
                    .add_ball(radius, &Isometry::identity())
                    .expect("a ball");
                let prism = a_prism_with_its_edge_on_the_axis(&mut model, radius, degrees);
                let case = (degrees, radius, rule);
                let outcome = model.fuse(
                    &[ball],
                    &[prism],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                if let Some(how) = moved_from_its_pin(
                    &mut model,
                    outcome,
                    is_pinned_as_answered(&answered, case),
                    radius,
                    closed_form(degrees, radius),
                ) {
                    moved.push(format!("{case:?}: {how}"));
                }
            }
        }
    }
    assert!(moved.is_empty(), "{}", moved.join("\n"));
}

/// How a case differs from its pin, or `None` when it answers as pinned.
///
/// A case pinned as answered must be one solid, valid, and measure against its
/// closed form. Any other case must be refused by one of the names listed here.
fn moved_from_its_pin(
    model: &mut Model,
    outcome: Result<rcad_topo::Fused, Error>,
    pinned_as_answered: bool,
    radius: f64,
    closed_form: f64,
) -> Option<String> {
    match (outcome, pinned_as_answered) {
        (Ok(fused), true) => {
            let answer = fused.solids().to_vec();
            assert_eq!(answer.len(), 1);
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("a fuse writes a sound model or writes nothing");
            measures(volume_of(model, &answer, radius), closed_form);
            None
        }
        (Ok(fused), false) => Some(format!("answered with {} solids", fused.solids().len())),
        (Err(why), true) => Some(format!("refused: {why:?}")),
        (Err(why), false) => {
            assert!(
                matches!(
                    which_refusal(&why),
                    "a point inside a piece of a face"
                        | "CannotTellTheCellsOfAFaceApart"
                        | "ResultIsNotManifold"
                        | "CannotMeshAFace"
                        | "CannotNameWhichPieceOfACrossingThisIs"
                        | "DynamicRangeTooWide"
                ),
                "{why:?}"
            );
            None
        }
    }
}

fn is_pinned_as_answered(
    answered: &[(f64, f64, &str)],
    (degrees, radius, rule): (f64, f64, Rule),
) -> bool {
    let rule = format!("{rule:?}");
    answered.contains(&(degrees, radius, rule.as_str()))
}

/// The half-turn cut §195.1 answers, with the knife turned about the axis to
/// angles other than the two `degenerate_curved_fuses.rs`'s
/// `a_ball_cut_by_a_plane_down_its_polar_axis_is_answered` takes, which are
/// along the seam and square to it.
///
/// This is the first cut of every wedge above, so it is pinned here case by
/// case: whether it answers depends on the angle **and** the size. Where it
/// answers it is one solid, valid, and measures as half the ball.
///
/// **Every one answers now, and both lists are empty.** 135 degrees at radius 1
/// was the one answer the mesher refused, as a face going right round with no
/// seam: the rim's two pole sides are half a turn each, and summed the short
/// way round a half turn is a coin toss. It measures since
/// `WalkingAWire::finish` decides "right round" from the walk's own longitude.
/// The fuse refused 45 degrees at radii 1, 3 and 1000 and 170 degrees at radius
/// 1, the same way, from the coarse mesh it checks with: there the knife's
/// stored plane misses both poles by a rounding on the same side, so the kept
/// half held both and could not be meshed (§197.3). The mesher now puts a pole
/// in wherever the circle passes within one rounding of it, so both halves are
/// lunes. The lists stay, so the next refusal has somewhere to be pinned.
#[test]
fn a_ball_cut_down_its_polar_axis_off_its_seam_is_answered_or_refused_by_name() {
    let refused_by_the_fuse: [(f64, f64); 0] = [];
    let refused_by_the_mesher: [(f64, f64); 0] = [];
    for turn in [10.0, 45.0, 90.0, 135.0, 170.0] {
        for radius in RADII {
            let mut model = a_model(radius);
            let ball = model
                .add_ball(radius, &Isometry::identity())
                .expect("a ball");
            let knife = a_knife_through_the_axis(&mut model, radius, turn);
            let answer = match model.fuse(
                &[ball],
                &[knife],
                Rule::Intersection,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            ) {
                Err(why) => {
                    assert!(
                        refused_by_the_fuse.contains(&(turn, radius)),
                        "a ball of radius {radius} cut {turn} degrees round: {why:?}"
                    );
                    assert_eq!(which_refusal(&why), "ACurvedFaceGoesRightRoundWithNoSeam");
                    continue;
                }
                Ok(fused) => fused.solids().to_vec(),
            };
            assert!(
                !refused_by_the_fuse.contains(&(turn, radius)),
                "a ball of radius {radius} cut {turn} degrees round is answered now; \
                 take it off the list"
            );
            assert_eq!(answer.len(), 1);
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("a fuse writes a sound model or writes nothing");
            let measured = model
                .solid(answer[0])
                .expect("this model made the key")
                .measure(
                    Deflection::of(DEFLECTION_PER_RADIUS * radius).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
            match measured {
                Ok(measured) => {
                    assert!(
                        !refused_by_the_mesher.contains(&(turn, radius)),
                        "a ball of radius {radius} cut {turn} degrees round measures now; \
                         take it off the list"
                    );
                    measures(measured.volume(), 2.0 / 3.0 * PI * radius.powi(3));
                }
                Err(why) => {
                    assert!(
                        refused_by_the_mesher.contains(&(turn, radius)),
                        "half a ball of radius {radius} cut {turn} degrees round: {why:?}"
                    );
                    assert_eq!(which_refusal(&why), "ACurvedFaceGoesRightRoundWithNoSeam");
                }
            }
        }
    }
}

fn a_model(radius: f64) -> Model {
    Model::new(Scale::millimetres(25.0 * radius).expect("a positive size"))
}

/// The block `0 <= x <= 6r`, `|y|, |z| <= 6r`, turned `degrees` about the
/// polar axis. Its face `x = 0` holds the axis, and no edge of it touches the
/// ball.
///
/// A turn about the `z` axis leaves `(0, 0, z)` exactly where it was, but the
/// **stored** plane does not hold it exactly: `Plane::transformed` keeps the
/// offset as a rounded dot product, which is not zero. At radius 1 it is 0 at
/// 10 and 135 degrees, 8.88e-16 at 45, −4.44e-16 at 90 and 2.22e-16 at 170, so
/// most of these knives miss the poles by a rounding.
fn a_knife_through_the_axis(model: &mut Model, radius: f64, degrees: f64) -> SolidKey {
    let knife = block(
        model,
        [6.0 * radius, 12.0 * radius, 12.0 * radius],
        Vector::new(0.0, -6.0 * radius, -6.0 * radius),
    );
    let axis = Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("the z axis");
    model
        .place_solid(
            knife,
            &Isometry::rotation(&axis, degrees.to_radians()).expect("a finite turn"),
        )
        .expect("this model made the key");
    knife
}

fn a_balls_volume(radius: f64) -> f64 {
    4.0 / 3.0 * PI * radius.powi(3)
}

fn a_knifes_volume(radius: f64) -> f64 {
    6.0 * 12.0 * 12.0 * radius.powi(3)
}

/// Two triangles with sides `2r` either side of the wedge's middle, `4r` tall.
fn a_prisms_volume(radius: f64, degrees: f64) -> f64 {
    let far = 2.0 * radius;
    far * far * (degrees / 2.0).to_radians().sin() * 4.0 * radius
}

/// A prism `4r` tall standing on the wedge `degrees` wide that starts at `+y`,
/// its point at the origin and its far side `2r` out.
fn a_prism_with_its_edge_on_the_axis(model: &mut Model, radius: f64, degrees: f64) -> SolidKey {
    let far = 2.0 * radius;
    let middle = (90.0 + degrees / 2.0).to_radians();
    let end = (90.0 + degrees).to_radians();
    let outline = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [0.0, far],
        [far * middle.cos(), far * middle.sin()],
        [far * end.cos(), far * end.sin()],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a wedge-shaped outline");
    model
        .add_extrusion(
            &outline,
            4.0 * radius,
            &Isometry::translation(Vector::new(0.0, 0.0, -2.0 * radius)).expect("a shift"),
        )
        .expect("a prism")
}

fn block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3], at: Vector) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            along_z,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a box needs no decision")
}

/// Fuse under a certified policy and insist the model it leaves is valid.
fn fused(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Vec<SolidKey> {
    let answer = model
        .fuse(
            arguments,
            tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("this {rule:?} is answered: {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");
    answer
}

fn volume_of(model: &Model, solids: &[SolidKey], radius: f64) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(DEFLECTION_PER_RADIUS * radius).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("the answer measures")
                .volume()
        })
        .sum()
}

/// Insist a volume is under the closed form, by no more than [`HOW_FAR_UNDER`].
fn measures(found: f64, closed_form: f64) {
    let under = (closed_form - found) / closed_form;
    assert!(
        (0.0..HOW_FAR_UNDER).contains(&under),
        "measured {found}, closed form {closed_form}, {:.4}% under",
        100.0 * under
    );
}

/// The name of a refusal: the last predicate given up on, or the variant.
///
/// Read with `if let` rather than a `match`, because `Error` is
/// `#[non_exhaustive]` and a catch-all arm here would join the wildcard ledger
/// (§40) for a test helper that panics on anything else anyway.
fn which_refusal(why: &Error) -> &str {
    if let Error::Undecidable { records } = why {
        return records
            .records()
            .last()
            .expect("a refusal names at least one decision")
            .predicate();
    }
    if let Error::CannotTellTheCellsOfAFaceApart { .. } = why {
        return "CannotTellTheCellsOfAFaceApart";
    }
    if let Error::ACurvedFaceGoesRightRoundWithNoSeam { .. } = why {
        return "ACurvedFaceGoesRightRoundWithNoSeam";
    }
    if let Error::ResultIsNotManifold { .. } = why {
        return "ResultIsNotManifold";
    }
    if let Error::CannotMeshAFace { .. } = why {
        return "CannotMeshAFace";
    }
    if let Error::TwoPatchesOfTheMeshDrewTheSameSide { .. } = why {
        return "TwoPatchesOfTheMeshDrewTheSameSide";
    }
    if let Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed = why {
        return "ACurvedEdgeThatIsPartOfACrossingCannotBeNamed";
    }
    if let Error::Geometry(rcad_geom::Error::CannotNameWhichPieceOfACrossingThisIs) = why {
        return "CannotNameWhichPieceOfACrossingThisIs";
    }
    if let Error::AFaceTrimmedByACurveThisCannotFollow { .. } = why {
        return "AFaceTrimmedByACurveThisCannotFollow";
    }
    if let Error::Invalid(rcad_topo::Invalid::DynamicRangeTooWide { .. }) = why {
        return "DynamicRangeTooWide";
    }
    if let Error::Invalid(rcad_topo::Invalid::FacesDoNotAgreeWhichWayTheyFace) = why {
        return "FacesDoNotAgreeWhichWayTheyFace";
    }
    panic!("expected a refusal by name, got {why:?}")
}

/// What stopped the mesher, for a face it could not mesh, and otherwise
/// [`which_refusal`].
fn what_stops_the_mesher_or_which_refusal(why: &Error) -> &str {
    if let Error::CannotMeshAFace {
        why: CannotTriangulate::CornersCoincide { .. },
        ..
    } = why
    {
        return "CornersCoincide";
    }
    if let Error::CannotMeshAFace {
        why: CannotTriangulate::LoopsTouch { .. },
        ..
    } = why
    {
        return "LoopsTouch";
    }
    which_refusal(why)
}
