//! A half ball cut a second time, by a plane that misses its poles.
//!
//! Cutting a ball by a plane through its polar axis leaves a skin whose
//! boundary is one great circle, walked as two edges that meet at the poles:
//! one runs south to north, the other north to south. Both lie on the same
//! closed crossing when a later fuse arranges that face, and the one running
//! against the crossing's direction covers the stretch that **wraps** past the
//! crossing's start. Cutting those edges into arcs by their end indices alone
//! gave both of them the same stretch, and the fuse panicked with "two edges of
//! one face run along the same stretch of one crossing".
//!
//! The second cut does not have to go anywhere near the axis for that: every
//! cut here misses both poles.
//!
//! The union twin is there for the direction. A half ball made by intersection
//! and one made by union, on opposite sides of the same plane, have skins bound
//! by the same great circle and every corner of both sits on the plane, so a
//! rule that read which arc an edge covers off the plane alone would answer the
//! same for both — and one of them would come out inside out.
//!
//! **One of them is answered**, and measured against its closed form. The other
//! two are still refused further on, and each test says where. The closed forms
//! are written down beside them, so the day one is answered it is measured
//! before it asks to be turned round.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

const RADIUS: f64 = 1.0;

/// How fine a mesh every volume here is read off.
const DEFLECTION: f64 = 0.001;

/// How far under the closed form a measured volume may fall, as a share.
///
/// Every point of an inscribed mesh sits within the deflection `δ` of the
/// sphere, so the shell it leaves out is at most the curved area `A` times `δ`.
/// The volume of a solid is the sum over its faces of a third of each face's
/// area times its distance from the centre: the sphere gives `A·r/3`, a plane
/// through the centre gives nothing, and a plane the solid lies on the centre's
/// side of gives something positive. So every solid here has volume at least
/// `A·r/3`, and the share missing is at most `3δ/r`. For the union the flat
/// block adds volume and no curved area, so the bound is looser still there.
const HOW_FAR_UNDER: f64 = 3.0 * DEFLECTION / RADIUS;

/// Where the second cut lies, measured from the centre.
const CUT_AT: f64 = 0.5;

/// `z <= 0.5` off the half ball `x >= 0`: half of a ball with its cap of height
/// `r − 0.5` taken off.
///
/// Three things stood in the way, one after the other. The circle `z = 0.5`
/// starts where the rim `x = 0` crosses it, so the two places the circles cross
/// were folded into one and the skin's cells could not be told apart. The top
/// of the answer is a half disk, whose two corners wind round nothing, so no
/// point inside it could be found. And its straight side and its round side
/// run between the same two corners, so they came out one edge with four faces
/// along it.
#[test]
fn a_half_ball_cut_across_its_polar_axis_measures_half_a_ball_without_its_cap() {
    let mut model = a_model();
    let half = half_a_ball(&mut model, Rule::Intersection);
    let below = a_block_below(&mut model, CUT_AT);
    let answer = model
        .fuse(
            &half,
            &[below],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a half ball cut across its polar axis is answered")
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");
    assert_eq!(answer.len(), 1);
    measures(
        volume_of(&model, &answer),
        (a_balls_volume() - a_caps_volume(RADIUS - CUT_AT)) / 2.0,
    );
}

/// `x <= 0.5` off the half ball `x >= 0`: half a ball with the cap of height
/// `r − 0.5` beyond `x = 0.5` taken off.
///
/// Its rim `x = 0` is two edges between the poles, which used to be named as
/// one. Then its skin would not mesh: a half ball's skin with a round hole where
/// the cap came off is a curved face with a hole that is not a ring of two
/// whole circles, and it is cut from the hole up to each pole now.
#[test]
fn a_half_ball_cut_square_to_its_flat_face_measures_half_a_ball_without_its_side_cap() {
    let mut model = a_model();
    let half = half_a_ball(&mut model, Rule::Intersection);
    let before = block(
        &mut model,
        [7.0 * RADIUS + CUT_AT, 14.0 * RADIUS, 14.0 * RADIUS],
        Vector::new(-7.0 * RADIUS, -7.0 * RADIUS, -7.0 * RADIUS),
    );
    let answer = model
        .fuse(
            &half,
            &[before],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a half ball cut square to its flat face is answered")
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");
    assert_eq!(answer.len(), 1);
    measures(
        volume_of(&model, &answer),
        a_balls_volume() / 2.0 - a_caps_volume(RADIUS - CUT_AT),
    );
}

/// The ball joined to the block `0 <= x <= 6r`, `|y|, |z| <= 6r`, then cut by
/// `z <= 0.5` inside a block that covers everything else.
///
/// What is left is the block's part below the cut, `6 × 12 × (6 + 0.5)`, and
/// the ball's part below the cut on the far side of `x = 0` — half of a ball
/// with its cap of height `r − 0.5` taken off. The ball's near half lies inside
/// the block and adds nothing.
///
/// Its cells are told apart now, as the intersection twin's are. What refuses
/// is placing the half disk the cut leaves on `z = 0.5` for `x ≤ 0`: the one
/// point found inside it is classified against the joined solid, whose flat
/// face `x = 0` has a round hole walked as two edges between the poles, and
/// the ray caster cannot follow that hole. The candidate is dropped, no other
/// is left, and the fuse gives up undecided rather than by name.
#[test]
fn a_ball_joined_to_a_block_down_its_polar_axis_then_cut_across_it_is_refused_rather_than_panicking()
 {
    let mut model = a_model();
    let joined = half_a_ball(&mut model, Rule::Union);
    let below = a_block_below(&mut model, CUT_AT);
    let the_blocks_part = 6.0 * 12.0 * (6.0 + CUT_AT) * RADIUS.powi(3);
    let the_balls_part = (a_balls_volume() - a_caps_volume(RADIUS - CUT_AT)) / 2.0;
    refused_or_measured(
        &mut model,
        [&joined, &[below]],
        "Undecidable",
        the_blocks_part + the_balls_part,
    );
}

/// The joined solid's flat face `x = 0`, which the ray caster has to cross, is
/// one the trimming of which it cannot follow — whatever point of the face it
/// is asked about.
#[test]
fn the_flat_face_of_a_ball_joined_to_a_block_down_its_polar_axis_cannot_be_followed() {
    let mut model = a_model();
    let joined = half_a_ball(&mut model, Rule::Union);
    let solid = model.solid(joined[0]).expect("this model made the key");
    let flat = solid
        .faces()
        .find(|face| {
            face.surface()
                .as_plane()
                .is_some_and(|plane| plane.offset() == 0.0)
        })
        .expect("the block's face x = 0 is kept, with the ball's hole in it");
    let asked = solid.face_holds_the_point(
        flat.key(),
        rcad_predicates::Point::from([0.0, 3.0, 3.0]),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(
            asked,
            Err(Error::AFaceTrimmedByACurveThisCannotFollow { .. })
        ),
        "the day this is followed, the joined-and-cut test above should move: {asked:?}"
    );
}

/// Intersect `arguments` with `tools` and insist it is refused as `refusal`.
///
/// An answer is measured against `closed_form` first, so that the day this is
/// answered the test says whether the answer is right before it asks to be
/// turned into one that expects an answer.
fn refused_or_measured(
    model: &mut Model,
    [arguments, tools]: [&[SolidKey]; 2],
    refusal: &str,
    closed_form: f64,
) {
    match model.fuse(
        arguments,
        tools,
        Rule::Intersection,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Err(why) => assert_eq!(which_refusal(&why), refusal, "{why:?}"),
        Ok(fused) => {
            let answer = fused.solids().to_vec();
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("a fuse writes a sound model or writes nothing");
            assert_eq!(answer.len(), 1);
            measures(volume_of(model, &answer), closed_form);
            panic!("answered, one solid, and it measures right: make this test expect an answer");
        }
    }
}

/// The name of a refusal. Read with `if let` rather than a `match`, because
/// `Error` is `#[non_exhaustive]` and a catch-all arm here would join the
/// wildcard ledger (§40) for a helper that panics on anything else anyway.
fn which_refusal(why: &Error) -> &'static str {
    if let Error::CannotTellTheCellsOfAFaceApart { .. } = why {
        return "CannotTellTheCellsOfAFaceApart";
    }
    if let Error::CurvedFaceWithAHoleGoesTooFarRound { .. } = why {
        return "CurvedFaceWithAHoleGoesTooFarRound";
    }
    if let Error::Undecidable { .. } = why {
        return "Undecidable";
    }
    panic!("expected a refusal by name, got {why:?}")
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(25.0 * RADIUS).expect("a positive size"))
}

/// A ball fused by `rule` with the block `0 <= x <= 6r`, `|y|, |z| <= 6r`,
/// whose face `x = 0` holds the polar axis.
fn half_a_ball(model: &mut Model, rule: Rule) -> Vec<SolidKey> {
    let ball = model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball");
    let knife = block(
        model,
        [6.0 * RADIUS, 12.0 * RADIUS, 12.0 * RADIUS],
        Vector::new(0.0, -6.0 * RADIUS, -6.0 * RADIUS),
    );
    fused(model, &[ball], &[knife], rule)
}

/// The block `|x|, |y| <= 7r`, `-7r <= z <= at`: everything the other solids
/// reach, below the plane `z = at`.
fn a_block_below(model: &mut Model, at: f64) -> SolidKey {
    block(
        model,
        [14.0 * RADIUS, 14.0 * RADIUS, 7.0 * RADIUS + at],
        Vector::new(-7.0 * RADIUS, -7.0 * RADIUS, -7.0 * RADIUS),
    )
}

fn a_balls_volume() -> f64 {
    4.0 / 3.0 * PI * RADIUS.powi(3)
}

/// The cap of height `h` off a ball: `π h² (3r − h) / 3`.
fn a_caps_volume(h: f64) -> f64 {
    PI * h * h * (3.0 * RADIUS - h) / 3.0
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

fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(DEFLECTION).expect("a positive length"),
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
