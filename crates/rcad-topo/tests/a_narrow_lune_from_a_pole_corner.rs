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
//! None of them gets as far as the mesher. The wedge cannot be built by any of
//! the three roads tried here without the fuse refusing first, so what §195.1
//! left unmeasured is still unmeasured: these tests stand in front of it, and
//! each fails with the measurement in hand the day its road is answered.
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

const RULES: [Rule; 3] = [Rule::Intersection, Rule::Difference, Rule::Union];

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
/// every size. The second is refused by name at every angle, size and rule:
/// "order along a crossing" at 45 degrees, "a point inside a piece of a face"
/// at the other three. The turned knife's stored plane misses the poles by a
/// rounding (see [`a_knife_through_the_axis`]), and the refusals' boxes sit that
/// far beside them. It panicked in `arrange.rs`'s `cut_the_crossings_into_arcs`
/// until that walked round a closed crossing, §197.
///
/// The day a case is answered, it is measured against its closed form and then
/// fails, asking to be moved into a test that insists on the answer.
#[test]
fn a_wedge_cut_from_a_ball_by_two_knives_through_its_polar_axis_is_refused_by_name() {
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
                match model.fuse(
                    &half,
                    &[second],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                ) {
                    Err(why) => assert_eq!(
                        which_refusal(&why),
                        if degrees == 45.0 {
                            "order along a crossing"
                        } else {
                            "a point inside a piece of a face"
                        },
                        "the {rule:?} of a half ball of radius {radius} and a knife \
                         {degrees} degrees round"
                    ),
                    Ok(fused) => {
                        let answer = fused.solids().to_vec();
                        assert_eq!(answer.len(), 1);
                        measures(
                            volume_of(&model, &answer, radius),
                            closed_form(degrees, radius),
                        );
                        panic!(
                            "the {rule:?} of a half ball of radius {radius} and a knife \
                             {degrees} degrees round is answered and measures; move it \
                             into a test that insists on the answer"
                        );
                    }
                }
            }
        }
    }
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
                    "a point inside a piece of a face"
                        | "CannotTellTheCellsOfAFaceApart"
                        | "order along a crossing"
                ),
                "a {degrees} degree wedge of a ball of radius {radius}: {why:?}"
            );
        }
    }
}

/// A wedge cut in one fuse, by a prism whose edge **is** the polar axis.
///
/// This is not the lune §195.1 is about: the poles sit on the knife's edge
/// rather than on its face, which the fuse sees before it sees any lune. It is
/// refused by name at every angle, size and rule, on one of two refusals.
#[test]
fn a_wedge_cut_by_one_prism_whose_edge_is_the_polar_axis_is_refused_by_name() {
    for degrees in WEDGES {
        for radius in RADII {
            for rule in RULES {
                let mut model = a_model(radius);
                let ball = model
                    .add_ball(radius, &Isometry::identity())
                    .expect("a ball");
                let prism = a_prism_with_its_edge_on_the_axis(&mut model, radius, degrees);
                let why = match model.fuse(
                    &[ball],
                    &[prism],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                ) {
                    Err(why) => why,
                    Ok(fused) => panic!(
                        "the {rule:?} of a ball of radius {radius} and a {degrees} degree \
                         prism was answered with {} solids; this test records a refusal",
                        fused.solids().len()
                    ),
                };
                assert!(
                    matches!(
                        which_refusal(&why),
                        "a point inside a piece of a face" | "CannotTellTheCellsOfAFaceApart"
                    ),
                    "the {rule:?} of a ball of radius {radius} and a {degrees} degree prism: \
                     {why:?}"
                );
            }
        }
    }
}

/// The half-turn cut §195.1 answers, with the knife turned about the axis to
/// angles other than the two `degenerate_curved_fuses.rs`'s
/// `a_ball_cut_by_a_plane_down_its_polar_axis_is_answered` takes, which are
/// along the seam and square to it.
///
/// This is the first cut of every wedge above, so it is pinned here case by
/// case: whether it answers depends on the angle **and** the size. Where it
/// answers it is one solid, valid, and measures as half the ball — except at
/// 135 degrees and radius 1, where the answer passes `Model::validate` and the
/// mesher then refuses to measure it.
#[test]
fn a_ball_cut_down_its_polar_axis_off_its_seam_is_answered_or_refused_by_name() {
    let refused_by_the_fuse = [(45.0, 1.0), (45.0, 1000.0), (45.0, 3.0), (170.0, 1.0)];
    let refused_by_the_mesher = [(135.0, 1.0)];
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
    panic!("expected a refusal by name, got {why:?}")
}
