//! A point on the floor of a half ball whose cap nothing can follow, against
//! the same point on the half ball alone.
//!
//! The half ball is `z ≥ 0` of radius 5: a flat floor on `z = 0`, a round disk
//! with the equator for its side, and the ball's skin above it. The holed one
//! is `a_half_ball_holed_through_its_pole_and_capped_on_the_same_ball.rs`'s,
//! with its rim's corner on the pole: the skin is holed along the plane
//! `y + 3z = 15`, and the hole is capped on the same ball. That cap's answer
//! about every point is `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`, a gap
//! of its own (`docs/decisions.md` §220.6), so a ray that meets the ball is
//! never counted on the holed half ball, and the answer there is
//! `CannotFollowAFacesTrimming`.
//!
//! **A face that holds the point puts it on the skin, whatever another face
//! says** (`SolidView::classify_point`). The floor holds every point of the
//! disk on it, on both half balls, so both answer `OnBoundary` there. Until
//! Q1722 the holed one answered `CannotFollowAFacesTrimming`, naming the cap,
//! whichever of the floor and the cap was listed first in the shell: the
//! ray's count stopped at the cap and the floor's answer went with it. Q1716
//! made the same change for a ring no ray can be fired at.
//!
//! A point inside is held by no face, and there the cap's refusal still
//! stands: the half ball alone places it inside.
//!
//! **A ray another face throws away is thrown away whatever the cap would
//! add to it** (Q1727). A box round a point of the floor, or of the skin, is
//! grazed by that face on every ray, and both half balls answer
//! `Undecidable`. Until Q1727 the holed one named the cap, on a box whose
//! answer following the cap could not have changed. A box on the cap itself
//! is grazed by the cap alone, and is still refused.
//!
//! **A graze for want of the cap's answer is the cap's refusal** (Q1740). A
//! box on the ball under the floor is grazed by the cap's own sign, since the
//! cap cannot say it holds none of the box, and until Q1740 that graze went in
//! as one from a face that answered: a box whose one ray meeting the ball
//! ahead was grazed by the skin and the floor came back `Undecidable`, where
//! the half ball alone places it outside.

use rcad_geom::{EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_predicates::{Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, DraftStep, FaceKey, Model, SolidBuilder, SolidKey, SolidView};

const RADIUS: f64 = 5.0;

/// Points on the floor, off its rim: its middle, which is the ball's centre,
/// and points under the skin and under the cap.
const ON_THE_FLOOR: [[f64; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [1.0, -2.0, 0.0],
    [0.5, 4.0, 0.0],
    [-3.0, 1.5, 0.0],
];

/// Points inside both half balls, held by no face.
const INSIDE: [[f64; 3]; 2] = [[1.0, -2.0, 1.0], [-0.5, 0.25, 2.0]];

/// Points on the ball, off the cap: `y + 3z < 15`.
const ON_THE_SKIN: [[f64; 3]; 2] = [[3.0, 0.0, 4.0], [0.0, -3.0, 4.0]];

/// A point on the ball and on the cap, `y + 3z > 15`, as near as `f64` has it.
const ON_THE_CAP: [f64; 3] = [0.0, 1.4, 4.8];

/// A point on the ball under the floor, `z < 0`, aimed so that the ray to the
/// third of the targets `SolidView::classify_point` fires at leaves the ball on
/// the equator, near `(3.786, -3.266, 0)`. The other two rays leave the ball
/// behind the point, so no ray from it meets the ball ahead.
///
/// No point of the ball has all three rays leave it behind: seen from the
/// ball's centre the second and third targets are 165.3° apart, and a ray
/// leaves the ball behind only from a point within 77.5° of the second, or
/// 79.4° of the third. So a box on the cap always has a ray that meets the
/// ball ahead, where the cap is asked and refuses.
const UNDER_THE_FLOOR: [f64; 3] = [2.615527281489544, 2.1172300591344544, -3.698155474893049];

#[test]
fn a_point_on_the_floor_of_a_half_ball_whose_cap_nothing_follows_is_on_its_skin() {
    let mut wrong = Vec::new();
    for floor_first in [false, true] {
        let mut model = a_model();
        let holed = a_half_ball(&mut model, Cap::OnThePole, floor_first);
        let view = model.solid(holed).expect("this model made the key");
        for policy in [Policy::certified(), Policy::tolerant(100.0)] {
            for point in ON_THE_FLOOR {
                let answer =
                    view.classify_point(Point::from(point), &policy, &mut ProvenanceLog::new());
                if answer != Containment::OnBoundary {
                    wrong.push(format!(
                        "{point:?}, floor first {floor_first}, {:?}: {answer:?}",
                        policy.strictness()
                    ));
                }
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "points on the floor not on the skin:\n{}",
        wrong.join("\n")
    );
}

/// The control: the half ball with no hole and no cap.
#[test]
fn a_point_on_the_floor_of_the_half_ball_alone_is_on_its_skin() {
    for floor_first in [false, true] {
        let mut model = a_model();
        let alone = a_half_ball(&mut model, Cap::None, floor_first);
        let view = model.solid(alone).expect("this model made the key");
        for policy in [Policy::certified(), Policy::tolerant(100.0)] {
            for point in ON_THE_FLOOR {
                assert_eq!(
                    view.classify_point(Point::from(point), &policy, &mut ProvenanceLog::new()),
                    Containment::OnBoundary,
                    "{point:?} on the floor, floor first {floor_first}, {policy:?}"
                );
            }
        }
    }
}

/// A point no face holds is placed by a ray, and every ray from inside meets
/// the ball, where the cap is asked and refuses: the holed half ball still
/// names the cap, and the half ball alone places the point inside.
#[test]
fn a_point_inside_a_half_ball_whose_cap_nothing_follows_is_still_refused() {
    for floor_first in [false, true] {
        let mut model = a_model();
        let holed = a_half_ball(&mut model, Cap::OnThePole, floor_first);
        let alone = a_half_ball(&mut model, Cap::None, floor_first);
        let holed = model.solid(holed).expect("this model made the key");
        let alone = model.solid(alone).expect("this model made the key");
        let cap = the_cap(holed);
        for policy in [Policy::certified(), Policy::tolerant(100.0)] {
            for point in INSIDE {
                let what = format!("{point:?}, floor first {floor_first}, {policy:?}");
                assert_eq!(
                    holed.classify_point(Point::from(point), &policy, &mut ProvenanceLog::new()),
                    Containment::CannotFollowAFacesTrimming { face: cap },
                    "{what}: the holed half ball"
                );
                assert_eq!(
                    alone.classify_point(Point::from(point), &policy, &mut ProvenanceLog::new()),
                    Containment::Inside,
                    "{what}: the half ball alone"
                );
            }
        }
    }
}

/// A box round a point of the floor holds points inside the half ball and
/// points under it, and the floor grazes every ray from it, whatever the cap
/// would add. So the holed half ball says what the half ball alone says,
/// `Undecidable` with the floor's width. Until Q1727 the cap's refusal came
/// first, on the first ray that met the ball.
#[test]
fn a_box_straddling_the_floor_of_a_half_ball_whose_cap_nothing_follows_is_undecidable() {
    let wrong = where_the_holed_half_ball_does_not_say_what_the_half_ball_alone_does(&ON_THE_FLOOR);
    assert!(
        wrong.is_empty(),
        "boxes on the floor not answered as on the half ball alone:\n{}",
        wrong.join("\n")
    );
}

/// The same on the ball, off the cap: the skin grazes every ray from the box,
/// so the cap's refusal gives way there too. The cap grazes the ray as well,
/// since nothing says it holds none of the box, but the skin's graze is one
/// it would throw the ray away for anyway.
#[test]
fn a_box_straddling_the_skin_beside_a_cap_nothing_follows_is_undecidable() {
    let wrong = where_the_holed_half_ball_does_not_say_what_the_half_ball_alone_does(&ON_THE_SKIN);
    assert!(
        wrong.is_empty(),
        "boxes on the skin not answered as on the half ball alone:\n{}",
        wrong.join("\n")
    );
}

/// On the cap itself the skin holds none of the box, being holed there, and
/// the only face that grazes a ray from it is the cap, for want of an answer
/// about its trimming. Nothing that answered threw the ray away, so the
/// refusal stands, though the half ball alone says `Undecidable`.
#[test]
fn a_box_straddling_a_cap_nothing_follows_is_still_refused() {
    for said in what_both_say_of_boxes_round(&[ON_THE_CAP]) {
        assert_eq!(
            said.holed,
            Containment::CannotFollowAFacesTrimming { face: said.cap },
            "{}: the holed half ball",
            said.what
        );
        assert!(
            matches!(said.alone, Containment::Undecidable { .. }),
            "{}: the half ball alone: {:?}",
            said.what,
            said.alone
        );
    }
}

/// A box on the ball under the floor, outside both half balls, is held by no
/// face. The skin and the floor say so, and the cap cannot, so it grazes every
/// ray from the box for want of that answer. The third ray is grazed where it
/// leaves the ball on the equator as well, by the skin and the floor, and is
/// thrown away whatever the cap would say; the other two are grazed by the cap
/// alone. So the answer names the cap, where the half ball alone places the
/// box outside. Until Q1740 the cap's graze from its own sign went in as one
/// from a face that answered, every ray was thrown away, and the holed half
/// ball said `Undecidable`, which a looser policy does nothing for.
#[test]
fn a_box_on_the_ball_under_the_floor_names_the_cap_nothing_follows() {
    for said in what_both_say_of_boxes_round(&[UNDER_THE_FLOOR]) {
        assert_eq!(
            said.holed,
            Containment::CannotFollowAFacesTrimming { face: said.cap },
            "{}: the holed half ball",
            said.what
        );
        assert_eq!(
            said.alone,
            Containment::Outside,
            "{}: the half ball alone",
            said.what
        );
    }
}

/// Each box round `points` where the half ball alone is not `Undecidable`, or
/// the holed half ball does not say the same.
fn where_the_holed_half_ball_does_not_say_what_the_half_ball_alone_does(
    points: &[[f64; 3]],
) -> Vec<String> {
    what_both_say_of_boxes_round(points)
        .into_iter()
        .filter(|said| {
            !matches!(said.alone, Containment::Undecidable { .. }) || said.holed != said.alone
        })
        .map(|said| {
            format!(
                "{}: holed {:?}, alone {:?}",
                said.what, said.holed, said.alone
            )
        })
        .collect()
}

/// What the holed half ball and the half ball alone said of one box.
struct Said {
    what: String,
    holed: Containment,
    alone: Containment,
    /// The cap of that holed half ball.
    cap: FaceKey,
}

/// What both half balls say of a box round each of `points`, with the floor
/// listed first and last, under a certified and a tolerant policy.
fn what_both_say_of_boxes_round(points: &[[f64; 3]]) -> Vec<Said> {
    let mut answers = Vec::new();
    for floor_first in [false, true] {
        let mut model = a_model();
        let holed = a_half_ball(&mut model, Cap::OnThePole, floor_first);
        let alone = a_half_ball(&mut model, Cap::None, floor_first);
        let holed = model.solid(holed).expect("this model made the key");
        let alone = model.solid(alone).expect("this model made the key");
        let cap = the_cap(holed);
        for policy in [Policy::certified(), Policy::tolerant(100.0)] {
            for &point in points {
                let ask = |solid: SolidView<'_>| {
                    solid.classify_uncertain_point(
                        a_box_round(point),
                        &policy,
                        &mut ProvenanceLog::new(),
                    )
                };
                answers.push(Said {
                    what: format!(
                        "{point:?}, floor first {floor_first}, {:?}",
                        policy.strictness()
                    ),
                    holed: ask(holed),
                    alone: ask(alone),
                    cap,
                });
            }
        }
    }
    answers
}

/// A box a few hundred roundings wide round `at`, which straddles every
/// face `at` is on.
fn a_box_round(at: [f64; 3]) -> PredicatePoint {
    let reach = 256.0 * f64::EPSILON * at.iter().fold(1.0f64, |most, each| most.max(each.abs()));
    PredicatePoint::Constructed(
        at.map(|coordinate| Interval::around(coordinate, reach).expect("a finite place")),
    )
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(125.0).expect("a positive size"))
}

/// Whether the half ball's skin is holed and capped.
#[derive(Clone, Copy)]
enum Cap {
    None,
    /// Holed along `y + 3z = 15`, with the rim's one corner on the pole.
    OnThePole,
}

/// The ball's face with one wire, which on the holed half ball is the cap.
fn the_cap(solid: SolidView<'_>) -> FaceKey {
    let mut caps = solid
        .faces()
        .filter(|face| face.surface().as_plane().is_none() && face.wire_count() == 1);
    let cap = caps.next().expect("the cap");
    assert!(caps.next().is_none(), "one face of the ball with one wire");
    cap.key()
}

/// The half ball `z ≥ 0` of radius 5, its skin holed and capped as `cap`
/// says, with the floor listed in the shell first or last.
fn a_half_ball(model: &mut Model, cap: Cap, floor_first: bool) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let on_the_equator = builder
        .add_vertex(Point::new(RADIUS, 0.0, 0.0))
        .expect("a point");
    let equator = builder
        .add_edge(
            on_the_equator,
            on_the_equator,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a closed edge");
    let walking = |builder: &mut SolidBuilder, edge, orientation| {
        let coedge = builder
            .add_coedge(edge, orientation)
            .expect("an edge of this builder");
        builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("a closed wire")
    };
    let equator_on_the_skin = walking(&mut builder, equator, Orientation::Forward);
    let equator_on_the_floor = walking(&mut builder, equator, Orientation::Reversed);
    let ball = Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball"));
    let floor = builder
        .add_face(
            Surface::Plane(
                Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
                    .expect("a plane"),
            ),
            &[equator_on_the_floor],
        )
        .expect("a face with one wire");
    let mut faces = match cap {
        Cap::None => {
            let skin = builder
                .add_face(ball, &[equator_on_the_skin])
                .expect("a face with one wire");
            vec![(skin, Orientation::Forward)]
        }
        Cap::OnThePole => {
            let corner = builder
                .add_vertex(Point::new(0.0, 0.0, RADIUS))
                .expect("a point");
            let rim = builder
                .add_edge(
                    corner,
                    corner,
                    EdgeRun::RightRoundWhereItCrosses {
                        and: Surface::Plane(
                            Plane::from_point_and_normal(
                                Point::new(0.0, 3.0, 4.0),
                                Vector::new(0.0, 1.0, 3.0),
                            )
                            .expect("a plane"),
                        ),
                    },
                )
                .expect("a closed edge");
            let hole = walking(&mut builder, rim, Orientation::Reversed);
            let rim_of_the_cap = walking(&mut builder, rim, Orientation::Forward);
            let skin = builder
                .add_face(ball, &[equator_on_the_skin, hole])
                .expect("a face with two wires");
            let cap = builder
                .add_face(ball, &[rim_of_the_cap])
                .expect("a face with one wire");
            vec![(skin, Orientation::Forward), (cap, Orientation::Forward)]
        }
    };
    if floor_first {
        faces.insert(0, (floor, Orientation::Forward));
    } else {
        faces.push((floor, Orientation::Forward));
    }
    builder.add_shell(&faces).expect("the faces close up");
    let solid = builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a half ball is a solid");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a half ball is a sound model");
    solid
}
