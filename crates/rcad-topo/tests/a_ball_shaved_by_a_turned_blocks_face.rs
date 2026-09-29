//! A ball meant to touch a block's face at one point, with the block and the
//! ball both turned about `z`, so that the face's stored plane cuts a sliver off
//! the ball instead (`docs/decisions.md` §346.6).
//!
//! The block is `[−2, 0] × [−2, 0] × [0, 2]` and the ball has radius 1 round
//! `(1, off, 1)`, one unit out from the face `x = 0`, before the turn. Turned,
//! the plane of that face is stored a rounding off, and on the stored numbers
//! it cuts the ball in a circle, of radius `5.2e-9` to `4.1e-8` and taking off
//! a cap `1.3e-17` to `8.5e-16` deep in the six draws whose exact numbers were
//! worked out (§346.6). With `off` negative the circle is on the block's
//! face, so the right union is one solid joined through it. With `off` positive
//! it is off the face, beside the block, and the union is the block and the
//! ball apart, the ball with the cap as a face of its own.
//!
//! Every one of the fourteen draws here refused `a point inside a piece of a
//! face` before §346.6: each point put back on the ball is a box `±5.7e-14`
//! wide, far thicker than the cap. Now `select` finds points inside the cap and
//! inside the disk the ball leaves on the face, and each draw gets past that
//! search. What each then answers is pinned below, by name. None answers yet.

use rcad_geom::{Axis, Isometry, Scale, Surface, SurfaceMeet, Vector, intersect};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, Rule, SolidKey};

/// What a draw is refused with, whichever policy and argument order.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Refused {
    /// A point was found inside the cap, and the block could not place it. The
    /// block's ray reads its face `x = 0` by the triangles of its stored
    /// corners and put the point outside, where the stored plane puts it
    /// inside, so `select` dropped it; or the ray could not say at all.
    NoPointEverySolidCouldPlace,
    /// Every piece was placed, and a face of the ball with a hole of radius
    /// about `1e-8` in it could not be meshed.
    AHoleGoesTooFarRound,
    /// Every piece was placed, and the answer's own check found two faces that
    /// do not agree which way they face.
    FacesDoNotAgreeWhichWayTheyFace,
}

/// `(off, turned, what it is refused with)`.
const DRAWS: [(f64, f64, Refused); 14] = [
    (-0.5, 1.0, Refused::NoPointEverySolidCouldPlace),
    (-0.5, 1.5, Refused::NoPointEverySolidCouldPlace),
    (-0.5, 2.5, Refused::NoPointEverySolidCouldPlace),
    (-0.5, 3.0, Refused::NoPointEverySolidCouldPlace),
    (-0.3, 0.3, Refused::NoPointEverySolidCouldPlace),
    (-0.3, 0.5, Refused::NoPointEverySolidCouldPlace),
    (-0.3, 1.5, Refused::NoPointEverySolidCouldPlace),
    (-0.3, 2.0, Refused::AHoleGoesTooFarRound),
    (-0.3, 2.5, Refused::NoPointEverySolidCouldPlace),
    (-0.3, 3.0, Refused::NoPointEverySolidCouldPlace),
    (0.3, 0.5, Refused::AHoleGoesTooFarRound),
    (0.3, 2.5, Refused::NoPointEverySolidCouldPlace),
    (0.3, 5.0, Refused::FacesDoNotAgreeWhichWayTheyFace),
    (0.5, 5.0, Refused::FacesDoNotAgreeWhichWayTheyFace),
];

#[test]
fn what_each_ball_shaved_by_a_turned_blocks_face_is_refused_with() {
    let mut wrong = Vec::new();
    for (off, turned, expected) in DRAWS {
        for (name, policy) in [
            ("certified", Policy::certified()),
            ("tolerant", Policy::tolerant(100.0)),
        ] {
            for ball_first in [false, true] {
                let what = format!("off {off}, turned {turned}, {name}, ball first {ball_first}");
                let got = what_the_union_is_refused_with(turned, off, &policy, ball_first);
                if got != Ok(expected) {
                    wrong.push(format!("{what}: {got:?} where {expected:?} is pinned"));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The premise: in every draw the stored plane of the block's face `x = 0`
/// really crosses the ball in a curve, rather than touching it at a point or
/// missing it.
#[test]
fn the_turned_face_cuts_the_ball_in_every_draw() {
    let policy = Policy::certified();
    for (off, turned, _) in DRAWS {
        let (model, block, ball) = a_block_and_a_ball_beside_it(turned, off, &policy);
        let skin = model
            .solid(ball)
            .expect("this model made the key")
            .faces()
            .next()
            .expect("a ball has a face")
            .surface();
        let face = model
            .solid(block)
            .expect("this model made the key")
            .faces()
            .map(|face| face.surface())
            .find(|surface| {
                let Surface::Plane(flat) = surface else {
                    return false;
                };
                let [a, b, c, _] = flat.coefficients();
                let (cos, sin) = (turned.cos(), turned.sin());
                (a - cos).abs() < 1e-12 && (b - sin).abs() < 1e-12 && c == 0.0
            })
            .expect("the block has its face x = 0, turned");
        let meet = intersect(&face, &skin, &policy, &mut ProvenanceLog::new());
        assert!(
            matches!(meet, Ok(SurfaceMeet::Curves(ref runs)) if runs.len() == 1),
            "off {off}, turned {turned}: the face's plane meets the ball as {meet:?}"
        );
    }
}

fn what_the_union_is_refused_with(
    turned: f64,
    off: f64,
    policy: &Policy,
    ball_first: bool,
) -> Result<Refused, String> {
    let (mut model, block, ball) = a_block_and_a_ball_beside_it(turned, off, policy);
    let (arguments, tools) = if ball_first {
        ([ball], [block])
    } else {
        ([block], [ball])
    };
    let fused = model.fuse(
        &arguments,
        &tools,
        Rule::Union,
        policy,
        &mut ProvenanceLog::new(),
    );
    match fused {
        Ok(fused) => Err(format!(
            "answered with {} solids; check it validates, is one solid where the circle is on \
             the face, and holds the block's volume and the ball's",
            fused.solids().len()
        )),
        Err(Error::Undecidable { records })
            if records.records().iter().any(|record| {
                record.predicate() == "a point in a piece that every solid could place"
            }) =>
        {
            Ok(Refused::NoPointEverySolidCouldPlace)
        }
        Err(Error::CurvedFaceWithAHoleGoesTooFarRound { .. }) => Ok(Refused::AHoleGoesTooFarRound),
        Err(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace)) => {
            Ok(Refused::FacesDoNotAgreeWhichWayTheyFace)
        }
        Err(other) => Err(format!("{other:?}")),
    }
}

/// A cube of side 2 from `(−2, −2, 0)` and a ball of radius 1 at `(1, off, 1)`,
/// both turned `turned` about `z`.
fn a_block_and_a_ball_beside_it(
    turned: f64,
    off: f64,
    policy: &Policy,
) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let everything = Isometry::rotation(&Axis::z(), turned).expect("a finite turn");
    let block = model
        .add_cube(2.0, policy, &mut ProvenanceLog::new())
        .expect("a cube of side 2");
    model
        .place_solid(
            block,
            &everything.compose(
                &Isometry::translation(Vector::new(-2.0, -2.0, 0.0)).expect("a finite shift"),
            ),
        )
        .expect("this model made the key");
    let ball = model
        .add_ball(
            1.0,
            &everything.compose(
                &Isometry::translation(Vector::new(1.0, off, 1.0)).expect("a finite shift"),
            ),
        )
        .expect("a ball");
    (model, block, ball)
}
