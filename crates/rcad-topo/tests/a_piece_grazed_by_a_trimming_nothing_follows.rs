//! Fuses that reach Q1740's change, and what each of them answers.
//!
//! The half ball is `z ≥ 0` of radius 5, its skin holed along the plane
//! `y + 3z = 15` and the hole capped on the same ball, as in
//! `a_point_on_a_face_beside_a_trimming_nothing_follows.rs`. Nothing follows
//! that cap's trimming. Q1740 changed one arm of `classify.rs`'s
//! `Crossings::grazed_unless_it_holds_none`: a face whose trimming nothing
//! follows, asked about a box its own sign could not place, now marks the ray
//! as meeting that face as well as grazing it. Read from the code, that could
//! turn a fuse's answer into a refusal: `fuse/select.rs`'s
//! `settle_against_each` gives `MetAFaceNobodyFollows` where it gave
//! `Abandoned`, and `place_the_piece` then refuses
//! `AFaceTrimmedByACurveThisCannotFollow` before the rules tried after the
//! search (a shared side, a corner, a plane) get a go at the piece (Q1760).
//!
//! **No fuse found gave a different answer for it.** The arm is reached only
//! where a point of a piece is a rounding or so off the cap's ball, so its box
//! straddles the cap's sign: by a ball centred on the half ball's centre with a
//! radius one or four roundings off 5, and by a bowl, that ball kept below a
//! plane `z = top`. When Q1760 counted, the 288 fuses pinned here reached it
//! 1428 times, all of them with the capped half ball: 384 times in the 24
//! fuses with a ball, 1044 in the 120 with a bowl. Q1760's lanes turned the
//! arm back to what it was before Q1740, through a probe edit to `classify.rs`
//! that is not kept, and every fuse pinned here answered the same with it
//! either way. So did 372 more that are not pinned: balls of other radii and
//! centres, bowls of radius 5 and `5(1 ± 2⁻⁴⁰)`, and copies of the half ball,
//! capped or not, moved by up to `1e-15`. In the 288, with the arm off as with
//! it on, `settle_against_each` was handed `CannotFollowAFacesTrimming` 210
//! times and `place_the_piece` refused naming the cap 93 times, so the arm's
//! mark was never what made them refuse; `fuse/assemble.rs`'s
//! `one_shell_is_inside_another`, which reads the same refusal, met it in none
//! of them. Across the rest of `rcad-topo`'s tests the arm was reached only by
//! three tests of `a_point_on_a_face_beside_a_trimming_nothing_follows.rs`,
//! none of them a fuse, and 25 more shapes fused with the capped half ball
//! (blocks and balls over the cap and the floor, copies of it turned and placed
//! apart) never reached it. Those counts are Q1760's, and were not taken again
//! when Q2243 changed what 27 of the 288 answer. So the pins below are what
//! each fuse answers today; they are not a check that the arm is there.
//!
//! **The ring was not run.** The row asked for a ring whose wall nothing
//! follows too, and none was found that can be built: `holds.rs` follows every
//! ring the tests cut with planes, a ring between two planes square to its
//! axis among them.
//!
//! The half ball with no hole and no cap is beside each pin, to show which of
//! the refusals are the cap's. Against a ball just bigger the capped one said
//! `Undecidable` where the half ball alone answers, a refusal that did not
//! name the face that caused it. `fuse/select.rs`'s `by_a_corner` passed over
//! the one corner of the bigger ball's skin, which is on the stored plane of
//! the half ball's floor. It now asks a corner on the plane of a flat face with
//! a curved side, as the floor is (Q2243): the corner
//! `(0, -5.000000000000001, 0)` places the skin outside, and a later piece is
//! refused naming the cap, as against a ball just smaller. The bowls
//! a rounding off kept below the floor answer, or name the cap, for the same
//! reason. Against a bowl clear of the half ball, `z ≤ -1`, both say
//! `Undecidable`, though the two solids are 1 apart.

use core::f64::consts::PI;

use rcad_geom::{
    Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftStep, DraftWire, Error, FaceKey, Fused, Model, Rule, SolidBuilder, SolidKey,
    SolidView,
};

use Answer::{CannotMesh, NamesTheCap, NoMeshCanTell, Solids, Undecidable};

const RADIUS: f64 = 5.0;

/// What one fuse came to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Answer {
    /// This many solids.
    Solids(usize),
    /// `Error::Undecidable`.
    Undecidable,
    /// `Error::AFaceTrimmedByACurveThisCannotFollow`, naming the half ball's
    /// cap.
    NamesTheCap,
    /// `Error::CannotMeshAFace`.
    CannotMesh,
    /// `Error::CannotTellWhichWayASolidsFacesPoint`: a curved solid no mesh
    /// signs whose closest sum is not above zero.
    NoMeshCanTell,
}

/// What the half ball is fused with, centred where it is.
#[derive(Clone, Copy, Debug)]
enum Other {
    /// The ball of this radius.
    Ball(f64),
    /// The ball of this radius kept below the plane `z = top`.
    Bowl { radius: f64, top: f64 },
}

/// Whether the half ball's skin is holed and capped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Half {
    Capped,
    Alone,
}

/// The ball one rounding bigger is refused naming the cap under every rule
/// both ways round, and so is the one four roundings bigger, where the half
/// ball alone answers all six. Each shell they leave between the two balls is
/// under `4e-15` thick.
///
/// All twelve were `Undecidable` until `by_a_corner` asked a corner on the
/// plane of a flat face with a curved side (Q2243): it now places the bigger
/// ball's skin by its corner on the half ball's floor plane, and a later piece
/// is refused naming the cap.
#[test]
fn a_ball_just_bigger_names_the_cap() {
    for radius in roundings_up() {
        assert_eq!(
            six_answers(Other::Ball(radius), Half::Capped),
            [NamesTheCap; 6],
            "{radius}"
        );
        assert_eq!(
            six_answers(Other::Ball(radius), Half::Alone),
            [
                Solids(1),
                Solids(1),
                Solids(1),
                Solids(1),
                Solids(0),
                Solids(1)
            ],
            "{radius}"
        );
    }
}

/// A ball just smaller is refused naming the cap. The half ball alone answers
/// the one four roundings smaller, and cannot mesh what it writes for the one
/// a rounding smaller in three of the six.
#[test]
fn a_ball_just_smaller_names_the_cap() {
    let [one, four] = roundings_down();
    for radius in [one, four] {
        assert_eq!(
            six_answers(Other::Ball(radius), Half::Capped),
            [NamesTheCap; 6],
            "{radius}"
        );
    }
    assert_eq!(
        six_answers(Other::Ball(one), Half::Alone),
        [
            CannotMesh,
            CannotMesh,
            Solids(1),
            Solids(1),
            CannotMesh,
            Solids(1)
        ]
    );
    assert_eq!(six_answers(Other::Ball(four), Half::Alone), [Solids(1); 6]);
}

/// A bowl one or four roundings off the cap's ball. Kept below `z = 3` it is
/// refused naming the cap, and so is it kept below the floor or below
/// `z = -3`, clear of the half ball; kept below `z = 1`, or `z = -1`, it is
/// `Undecidable`.
///
/// The bowl a rounding smaller kept below the floor was `Undecidable` whenever
/// the half ball was listed first, until `by_a_corner` asked a corner on the
/// plane of a flat face with a curved side (Q2243).
#[test]
fn a_bowl_just_off_the_caps_ball_names_the_cap_or_is_undecidable() {
    let [one_up, four_up] = roundings_up();
    let [one_down, four_down] = roundings_down();
    for radius in [one_up, four_up, one_down, four_down] {
        for (top, said) in [
            (3.0, [NamesTheCap; 6]),
            (1.0, [Undecidable; 6]),
            (0.0, [NamesTheCap; 6]),
            (-1.0, [Undecidable; 6]),
            (-3.0, [NamesTheCap; 6]),
        ] {
            assert_eq!(
                six_answers(Other::Bowl { radius, top }, Half::Capped),
                said,
                "{radius} {top}"
            );
        }
    }
}

/// The same bowls against the half ball alone. Kept below `z = 1` or
/// `z = -1` they are still `Undecidable`, the second though the bowl is 1
/// clear of the half ball and is two solids with it. Kept below `z = -3` they
/// answer, and so do they kept below the floor, where the two are the halves
/// of one ball and hold what
/// `a_bowl_just_off_under_the_half_ball_alone_holds_what_two_half_balls_hold`
/// says. Kept below `z = 3`, the bowls bigger answer, the half ball less the
/// bowl four roundings smaller holds about `54` but is refused because no mesh
/// can tell which way its faces point, and the union with the bowl a rounding
/// smaller cannot be meshed.
///
/// The bowls a rounding off kept below the floor were `Undecidable` until
/// `by_a_corner` asked a corner on the plane of a flat face with a curved side
/// (Q2243).
#[test]
fn a_bowl_just_off_the_half_ball_alone_answers_or_is_undecidable() {
    let [one_up, four_up] = roundings_up();
    let [one_down, four_down] = roundings_down();
    let against = |radius, top| six_answers(Other::Bowl { radius, top }, Half::Alone);
    for radius in [one_up, four_up, one_down, four_down] {
        assert_eq!(against(radius, 1.0), [Undecidable; 6], "{radius}");
        assert_eq!(against(radius, -1.0), [Undecidable; 6], "{radius}");
        assert_eq!(
            against(radius, -3.0),
            [
                Solids(2),
                Solids(2),
                Solids(0),
                Solids(0),
                Solids(1),
                Solids(1)
            ],
            "{radius}"
        );
        assert_eq!(
            against(radius, 0.0),
            [
                Solids(1),
                Solids(1),
                Solids(0),
                Solids(0),
                Solids(1),
                Solids(1)
            ],
            "{radius}"
        );
    }
    for radius in [one_up, four_up] {
        assert_eq!(against(radius, 3.0), [Solids(1); 6], "{radius}");
    }
    assert_eq!(
        against(one_down, 3.0),
        [
            CannotMesh,
            CannotMesh,
            Solids(1),
            Solids(1),
            Solids(1),
            Solids(1)
        ]
    );
    assert_eq!(
        against(four_down, 3.0),
        [
            Solids(1),
            Solids(1),
            Solids(1),
            Solids(1),
            NoMeshCanTell,
            Solids(1)
        ]
    );
}

/// The half ball alone and a bowl one or four roundings off kept below its
/// floor are the two halves of one ball. Their union holds both halves, they
/// have nothing in common, and each less the other is itself.
#[test]
fn a_bowl_just_off_under_the_half_ball_alone_holds_what_two_half_balls_hold() {
    let half_a_ball = |radius: f64| 2.0 * PI * radius.powi(3) / 3.0;
    let mut wrong = Vec::new();
    let [one_up, four_up] = roundings_up();
    let [one_down, four_down] = roundings_down();
    for radius in [one_up, four_up, one_down, four_down] {
        let bowl = Other::Bowl { radius, top: 0.0 };
        each_fuse(bowl, Half::Alone, |rule, half_first, model, _, fused| {
            let should_hold = match (rule, half_first) {
                (Rule::Union, _) => Some(half_a_ball(RADIUS) + half_a_ball(radius)),
                (Rule::Intersection, _) => None,
                (Rule::Difference, true) => Some(half_a_ball(RADIUS)),
                (Rule::Difference, false) => Some(half_a_ball(radius)),
                (other, _) => panic!("no volume for {other:?}"),
            };
            let what = fused
                .map_err(|why| format!("refused {why:?}"))
                .and_then(|fused| what_it_holds(model, fused.solids(), should_hold));
            if let Err(why) = what {
                wrong.push(format!(
                    "{radius}, {rule:?}, half ball first {half_first}: {why}"
                ));
            }
        });
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// 5 one rounding up and four roundings up.
fn roundings_up() -> [f64; 2] {
    let one = RADIUS.next_up();
    [one, one.next_up().next_up().next_up()]
}

/// 5 one rounding down and four roundings down.
fn roundings_down() -> [f64; 2] {
    let one = RADIUS.next_down();
    [one, one.next_down().next_down().next_down()]
}

/// The union, the intersection and the difference of the half ball and
/// `other`, each with the half ball first and then second.
fn six_answers(other: Other, half: Half) -> [Answer; 6] {
    let mut said = [Undecidable; 6];
    let mut at = said.iter_mut();
    each_fuse(other, half, |rule, half_first, _, cap, fused| {
        *at.next().expect("six answers") = match fused {
            Ok(answer) => Solids(answer.solids().len()),
            Err(Error::Undecidable { .. }) => Undecidable,
            Err(Error::AFaceTrimmedByACurveThisCannotFollow { face }) if Some(face) == cap => {
                NamesTheCap
            }
            Err(Error::CannotMeshAFace { .. }) => CannotMesh,
            Err(Error::CannotTellWhichWayASolidsFacesPoint { .. }) => NoMeshCanTell,
            Err(other) => panic!("{rule:?}, half ball first {half_first}: {other:?}"),
        };
    });
    said
}

/// Each of the six fuses [`six_answers`] makes, handed to `look` with its
/// rule, whether the half ball was first, the model it was made in, the half
/// ball's cap if it has one, and what the fuse came to.
fn each_fuse(
    other: Other,
    half: Half,
    mut look: impl FnMut(Rule, bool, &Model, Option<FaceKey>, rcad_topo::Result<Fused>),
) {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for half_first in [true, false] {
            let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
            let half_ball = a_half_ball(&mut model, half);
            let cap = (half == Half::Capped)
                .then(|| the_cap(model.solid(half_ball).expect("this model made the key")));
            let other = match other {
                Other::Ball(radius) => model
                    .add_ball(radius, &Isometry::identity())
                    .expect("a ball"),
                Other::Bowl { radius, top } => a_bowl(&mut model, radius, top),
            };
            let (arguments, tools) = if half_first {
                ([half_ball], [other])
            } else {
                ([other], [half_ball])
            };
            let fused = model.fuse(
                &arguments,
                &tools,
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            look(rule, half_first, &model, cap, fused);
        }
    }
}

/// Whether `solids` is one solid that validates and measures `should_hold`
/// within the error its mesh declares, or none where `should_hold` is `None`.
fn what_it_holds(
    model: &Model,
    solids: &[SolidKey],
    should_hold: Option<f64>,
) -> Result<(), String> {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .map_err(|why| format!("does not validate: {why:?}"))?;
    let (&[solid], Some(should_hold)) = (solids, should_hold) else {
        return match (solids.len(), should_hold) {
            (0, None) => Ok(()),
            (count, _) => Err(format!("{count} solids")),
        };
    };
    let measured = model
        .solid(solid)
        .expect("the fuse made the key")
        .measure(
            Deflection::of(0.01).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("does not measure: {why:?}"))?;
    if (measured.volume() - should_hold).abs() > measured.volume_error() {
        return Err(format!(
            "measured {} ± {}, and it should hold {should_hold}",
            measured.volume(),
            measured.volume_error()
        ));
    }
    Ok(())
}

/// The half ball `z ≥ 0` of radius 5, its skin holed and capped or not.
fn a_half_ball(model: &mut Model, half: Half) -> SolidKey {
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
    let mut faces = match half {
        Half::Capped => {
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
        Half::Alone => {
            let skin = builder
                .add_face(ball, &[equator_on_the_skin])
                .expect("a face with one wire");
            vec![(skin, Orientation::Forward)]
        }
    };
    faces.push((floor, Orientation::Forward));
    builder.add_shell(&faces).expect("the faces close up");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a half ball is a solid")
}

/// The ball of `radius` about the origin, kept below the plane `z = top`.
fn a_bowl(model: &mut Model, radius: f64, top: f64) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let on_the_rim = builder
        .add_vertex(Point::new((radius * radius - top * top).sqrt(), 0.0, top))
        .expect("a point");
    let rim = builder
        .add_edge(
            on_the_rim,
            on_the_rim,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a closed edge");
    let rim_on_the_skin = walking(&mut builder, rim, Orientation::Reversed);
    let rim_on_the_lid = walking(&mut builder, rim, Orientation::Forward);
    let skin = builder
        .add_face(
            Surface::Sphere(Sphere::new(Point::origin(), radius).expect("a ball")),
            &[rim_on_the_skin],
        )
        .expect("a face with one wire");
    let lid = builder
        .add_face(
            Surface::Plane(
                Plane::from_point_and_normal(Point::new(0.0, 0.0, top), Vector::new(0.0, 0.0, 1.0))
                    .expect("a plane"),
            ),
            &[rim_on_the_lid],
        )
        .expect("a face with one wire");
    builder
        .add_shell(&[(skin, Orientation::Forward), (lid, Orientation::Forward)])
        .expect("the faces close up");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bowl is a solid")
}

/// A wire that walks once round the closed `edge`.
fn walking(builder: &mut SolidBuilder, edge: DraftEdge, orientation: Orientation) -> DraftWire {
    let coedge = builder
        .add_coedge(edge, orientation)
        .expect("an edge of this builder");
    builder
        .add_wire(&[DraftStep::Along(coedge)])
        .expect("a closed wire")
}

/// The face of the ball with one wire.
fn the_cap(solid: SolidView<'_>) -> FaceKey {
    let mut caps = solid
        .faces()
        .filter(|face| face.surface().as_plane().is_none() && face.wire_count() == 1);
    let cap = caps.next().expect("the cap");
    assert!(caps.next().is_none(), "one face of the ball with one wire");
    cap.key()
}
