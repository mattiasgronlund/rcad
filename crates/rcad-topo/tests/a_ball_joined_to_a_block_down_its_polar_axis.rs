//! A ball joined to a block whose face `x = 0` holds the ball's polar axis.
//!
//! The block's face keeps the square `|y|, |z| <= 6r` with the unit disk taken
//! out where the ball's near half went in. That hole is one circle walked as
//! two edges from pole to pole, and a ray casting through it meets the face at
//! a point it worked out — a box, a hair either side of `x = 0`.

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Interval, Point, Policy, ProvenanceLog};
use rcad_topo::{Model, OnAFace, ProfileBuilder, Rule, SolidKey};

const RADIUS: f64 = 1.0;

/// The ray caster asks the face about the point where the ray met its plane,
/// and that point is a box. The walk round the block's square aims its
/// half-line at one of the square's corners and then asks which side of that
/// half-line every corner is on — the aimed-at corner too, whose answer is
/// zero whatever the point is. Asked with a box it used to come back with no
/// sign, so every ray through this face gave up.
#[test]
fn a_point_worked_out_on_the_flat_face_is_placed_in_the_hole_or_on_the_block() {
    let mut model = a_model();
    let joined = half_a_ball(&mut model);
    let solid = model.solid(joined[0]).expect("this model made the key");
    let flat = solid
        .faces()
        .find(|face| {
            face.surface()
                .as_plane()
                .is_some_and(|plane| plane.offset() == 0.0)
        })
        .expect("the block's face x = 0 is kept, with the ball's hole in it");
    let asked = |[y, z]: [f64; 2]| {
        let a_hair = f64::EPSILON;
        let at = Point::Constructed([
            Interval::new(-a_hair, a_hair).expect("an ordered pair"),
            Interval::exactly(y),
            Interval::exactly(z),
        ]);
        solid
            .face_holds_the_point(
                flat.key(),
                at,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("({y}, {z}) is answered: {why:?}"))
    };
    assert_eq!(asked([0.3, 0.7]), OnAFace::NotHeld);
    assert_eq!(asked([3.0, 3.0]), OnAFace::Held);
    assert_eq!(asked([-0.8, -0.7]), OnAFace::Held);
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(25.0 * RADIUS).expect("a positive size"))
}

/// A ball joined to the block `0 <= x <= 6r`, `|y|, |z| <= 6r`.
fn half_a_ball(model: &mut Model) -> Vec<SolidKey> {
    let ball = model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball");
    let block = block(
        model,
        [6.0 * RADIUS, 12.0 * RADIUS, 12.0 * RADIUS],
        Vector::new(0.0, -6.0 * RADIUS, -6.0 * RADIUS),
    );
    let answer = model
        .fuse(
            &[ball],
            &[block],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a ball joined to a block down its polar axis is answered")
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");
    answer
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
