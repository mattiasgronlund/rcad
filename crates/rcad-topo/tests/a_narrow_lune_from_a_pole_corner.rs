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
//! its closed form: 17 cut by two knives, 13 intersections and 4 unions, and 10
//! by one prism, all intersections. The rest are still refused by name, most
//! now at the flat pieces or in assembly rather than at the curved pieces. A
//! case that moves either way fails, naming itself.
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
/// [`the_quarter_turned_knife_holds_the_axis_by_its_corners_and_not_by_its_plane`].
/// Since the fuse drops a point the ray and the stored planes disagree about,
/// the 90 degree wedge answers at every size as an intersection and a union,
/// and so does the 10 degree one at radii 0.001 and 1000. The union needed one
/// more step: the knife's face then has a half disk hole, two corners that
/// wind round nothing, and `assemble.rs`'s `rings_round` panicked on it. The
/// difference is refused at 90 degrees, by name. It panicked in `arrange.rs`'s
/// `cut_the_crossings_into_arcs` until that walked round a closed crossing,
/// §197.
///
/// Since the fuse steps a curved piece's point towards every cut's material
/// (§200), the cases in `answered` get past the curved pieces and answer, and
/// each is measured against its closed form. Most of the rest now reach the
/// flat pieces and assembly and are refused there instead. A case that moves
/// either way fails, naming itself.
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
        (10.0, 0.001, "Intersection"),
        (10.0, 0.001, "Union"),
        (10.0, 1000.0, "Intersection"),
        (10.0, 1000.0, "Union"),
        (10.0, 3.0, "Intersection"),
        (170.0, 1.0, "Intersection"),
        (170.0, 0.001, "Intersection"),
        (170.0, 1000.0, "Intersection"),
        (170.0, 3.0, "Intersection"),
    ];
    let mut moved = Vec::new();
    let rules: [(Rule, ClosedForm); 3] = [
        (Rule::Intersection, |degrees, radius| {
            degrees / 360.0 * a_balls_volume(radius)
        }),
        (Rule::Difference, |degrees, radius| {
            (180.0 - degrees) / 360.0 * a_balls_volume(radius)
        }),
        (Rule::Union, |degrees, radius| {
            (180.0 - degrees) / 360.0 * a_balls_volume(radius) + a_knifes_volume(radius)
        }),
    ];
    for degrees in WEDGES {
        for radius in RADII {
            for (rule, closed_form) in rules {
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

/// Why the 90 degree wedge was refused before it answered: the knife turned a
/// quarter turn holds the axis by its corners and misses it by its stored
/// plane, so a point in between is inside by the ray and outside by the plane.
///
/// `(0, 1.1e-16, -0.5)` is the first point the fuse tried inside the half disk
/// of `x = 0` that the knife does not cover. The fuse cuts along the stored
/// plane, which puts that half disk on the outside, and the point with it. The
/// ray fans the face's corners, `(∓6, ±8.9e-16, z)`, whose triangles hold the
/// axis exactly, and certifies the point inside. The fuse now drops a point
/// the two disagree about (`select.rs`'s
/// `the_ray_and_the_stored_planes_disagree_about`).
#[test]
fn the_quarter_turned_knife_holds_the_axis_by_its_corners_and_not_by_its_plane() {
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
        rcad_topo::Containment::Inside,
        "the ray puts it inside"
    );
}

/// The same two knives, the turned one first.
///
/// For a wedge of 90 degrees the turned knife lies along the seam. The first
/// cut is answered everywhere here but once — the 10 degree wedge's knife,
/// turned 170, at radius 1 (see
/// [`a_ball_cut_down_its_polar_axis_off_its_seam_is_answered_or_refused_by_name`]).
/// Whenever it is answered, cutting that half ball again by `x >= 0` is
/// refused by name, on one of three refusals that depend on the angle and the
/// size.
#[test]
fn a_wedge_cut_with_the_turned_knife_first_is_refused_by_name() {
    for degrees in WEDGES {
        for radius in RADII {
            let mut model = a_model(radius);
            let ball = model
                .add_ball(radius, &Isometry::identity())
                .expect("a ball");
            let turned = a_knife_through_the_axis(&mut model, radius, 180.0 - degrees);
            let half = match model.fuse(
                &[ball],
                &[turned],
                Rule::Intersection,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            ) {
                Ok(fused) => fused.solids().to_vec(),
                Err(why) => {
                    assert_eq!(
                        (degrees, radius, which_refusal(&why)),
                        (10.0, 1.0, "ACurvedFaceGoesRightRoundWithNoSeam"),
                        "only one first cut here is refused"
                    );
                    continue;
                }
            };
            let second = a_knife_through_the_axis(&mut model, radius, 0.0);
            let why = match model.fuse(
                &half,
                &[second],
                Rule::Intersection,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            ) {
                Err(why) => why,
                Ok(fused) => panic!(
                    "a {degrees} degree wedge of a ball of radius {radius} was answered \
                     with {} solids; this test records a refusal, so move it to the \
                     answered test",
                    fused.solids().len()
                ),
            };
            assert!(
                matches!(
                    which_refusal(&why),
                    "a point in a piece that every solid could place"
                        | "CannotTellTheCellsOfAFaceApart"
                        | "order along a crossing"
                ),
                "a {degrees} degree wedge of a ball of radius {radius}: {why:?}"
            );
        }
    }
}

/// The quarter wedge names the second of `place_the_piece`'s two searches.
///
/// The test above lets three names through, so it cannot say which of the two
/// searches gave up. This one says it for one case: a point **was** found
/// inside the piece, and what ran out is somewhere to stand that every solid
/// could place. Until §216.4 both searches wrote *a point inside a piece of a
/// face*, so this fuse wrote down the name of the search that had worked.
#[test]
fn a_quarter_wedge_stops_after_a_point_is_found_inside_the_piece_and_not_before() {
    let radius = 1.0;
    let mut model = a_model(radius);
    let ball = model
        .add_ball(radius, &Isometry::identity())
        .expect("a ball");
    let turned = a_knife_through_the_axis(&mut model, radius, 90.0);
    let half = model
        .fuse(
            &[ball],
            &[turned],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the first cut answers at every size")
        .solids()
        .to_vec();
    let second = a_knife_through_the_axis(&mut model, radius, 0.0);
    let why = model
        .fuse(
            &half,
            &[second],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("a 90 degree wedge of a ball of radius 1 is refused");
    assert_eq!(
        which_refusal(&why),
        "a point in a piece that every solid could place"
    );
}

/// A wedge cut in one fuse, by a prism whose edge **is** the polar axis.
///
/// This is not the lune §195.1 is about: the poles sit on the knife's edge
/// rather than on its face, which the fuse sees before it sees any lune. The
/// cases in `answered` answer and measure since §200; the rest are refused by
/// name. The quarter wedge of the ball of radius 1000 answers since the
/// triangulator takes a corner under spade's floor as zero (§321.6).
#[test]
fn a_wedge_cut_by_one_prism_whose_edge_is_the_polar_axis_is_answered_or_refused_by_name() {
    let answered = [
        (90.0, 1.0, "Intersection"),
        (90.0, 1000.0, "Intersection"),
        (45.0, 1.0, "Intersection"),
        (45.0, 1000.0, "Intersection"),
        (45.0, 3.0, "Intersection"),
        (10.0, 1.0, "Intersection"),
        (10.0, 0.001, "Intersection"),
        (10.0, 1000.0, "Intersection"),
        (10.0, 3.0, "Intersection"),
        (170.0, 1.0, "Intersection"),
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
                    "order along a crossing"
                        | "CannotTellTheCellsOfAFaceApart"
                        | "ResultIsNotManifold"
                        | "CannotMeshAFace"
                        | "ACurvedEdgeThatIsPartOfACrossingCannotBeNamed"
                        | "CannotNameWhichPieceOfACrossingThisIs"
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
    panic!("expected a refusal by name, got {why:?}")
}
