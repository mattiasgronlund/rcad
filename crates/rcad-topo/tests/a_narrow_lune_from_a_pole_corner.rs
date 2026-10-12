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
/// names. A tolerant policy only warns of the range, and there the seven are
/// still refused that name
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
/// further on.
///
/// **The seven Q1664 moved are refused
/// `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed`**, as all seven were under
/// every policy before Q1664: the arc of `x = 0` from a pole to a corner a
/// rounding off it has its chord along the tangent at the pole. **Q1614's two
/// are refused `CannotNameWhichPieceOfACrossingThisIs`**, where they were
/// refused before Q1614.
///
/// **Naming that arc by the tangent at the pole answered none of them** (Q1665,
/// commit `e5dcff60`, taken back), the shorter arc when the way the edge leaves
/// the pole and its chord point the same way. The four 90 degree differences
/// were then refused
/// `CannotMeshAFace` for corners that coincide, the 45 degree one at radius
/// 0.001 and the 10 degree one at radius 1000 were refused
/// `FacesDoNotAgreeWhichWayTheyFace`, and Q1614's two were refused as here.
/// The 10 degree intersection at radius 1 did not finish: `validate.rs`'s
/// `the_coarsest_mesh_whose_sides_do_not_cross` halved its deflection rung
/// after rung down towards half the closest corner pair, `1.28e−15`, and every
/// rung was refused `LoopsTouch` at the same corner, `(0, −6.4e−16, −1)`, where
/// the two knives' line meets the ball a rounding past the south pole.
#[test]
fn two_wedges_cut_by_two_knives_are_refused_by_name_under_a_tolerant_policy() {
    const UNNAMED_ARC: &str = "ACurvedEdgeThatIsPartOfACrossingCannotBeNamed";
    const UNNAMED_PIECE: &str = "CannotNameWhichPieceOfACrossingThisIs";
    for (degrees, radius, rule, refused) in [
        (90.0, 1.0, Rule::Difference, UNNAMED_ARC),
        (90.0, 0.001, Rule::Difference, UNNAMED_ARC),
        (90.0, 1000.0, Rule::Difference, UNNAMED_ARC),
        (90.0, 3.0, Rule::Difference, UNNAMED_ARC),
        (45.0, 0.001, Rule::Difference, UNNAMED_ARC),
        (45.0, 1000.0, Rule::Difference, UNNAMED_PIECE),
        (10.0, 1.0, Rule::Intersection, UNNAMED_ARC),
        (10.0, 0.001, Rule::Difference, UNNAMED_PIECE),
        (10.0, 1000.0, Rule::Difference, UNNAMED_ARC),
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
            which_refusal(&why),
            refused,
            "{degrees} degrees, radius {radius}, {rule:?}: {why:?}"
        );
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
    panic!("expected a refusal by name, got {why:?}")
}
