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
//! search. Each now answers, and the answer holds ([`the_answer_holds`]).
//!
//! Four of them answer since §402. The plane of the block's face `y = 0` cuts
//! the ball too, beside the block, in a circle `|off|` from its middle, so the
//! ball's skin is left with a face holding both poles and two holes: that wide
//! circle and the tiny one. The mesher had no cut for a ball face whose loops
//! are all holes, and it now opens one into strips from pole to pole, here a
//! hole in each (§346.6, §402.5).
//!
//! **Where the union is one solid that face is joined now** (§229.2). The wide
//! circle has the ball on both sides of it there and is no edge, so the
//! joiner's ball arm folds the cap past `y = 0` back in, and the ball's skin is
//! one face with one hole, the tiny circle the block's face cuts. Where the ball
//! stands beside the block every piece of its skin is kept, which is the whole
//! ball and is left in its three pieces: the tiny cap, the cap past `y = 0`,
//! and the face with both holes.
//!
//! Three more answer since Q668. A point in the cap is between the block's
//! stored plane `x = 0` and the triangles of its stored corners, and the ray
//! that counted those triangles could not settle which side of them it was.
//! The ray now names such a face, and the block's stored plane places the
//! point when no other face of the block is near it (§346.6).
//!
//! The other seven answer since Q996: `-0.5` turned 1.5, 2.5 and 3, `-0.3`
//! turned 1.5, 2.5 and 3, and `+0.3` turned 2.5. Each found its point and
//! `select` dropped it, because some triangle of the block's corners and its
//! face's stored plane put the point on opposite sides, whatever the ray then
//! said. In `+0.3` turned 2.5 the point was `0.3` past the face `y = 0`, and
//! `Outside` by either reading. `select` no longer drops such a point: the ray
//! asks the plane itself wherever the two part (§346.6).

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Surface, SurfaceMeet, Vector, intersect};
use rcad_predicates::{Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, Rule, SolidKey};

/// `(off, turned)`.
const DRAWS: [(f64, f64); 14] = [
    (-0.5, 1.0),
    (-0.5, 1.5),
    (-0.5, 2.5),
    (-0.5, 3.0),
    (-0.3, 0.3),
    (-0.3, 0.5),
    (-0.3, 1.5),
    (-0.3, 2.0),
    (-0.3, 2.5),
    (-0.3, 3.0),
    (0.3, 0.5),
    (0.3, 2.5),
    (0.3, 5.0),
    (0.5, 5.0),
];

/// The block and the ball together. The cap they share, or the ball loses, is
/// a rounding deep and `1e-8` wide, so its volume is below `1e-29` and the
/// closed form leaves it out.
const THE_BLOCK_AND_THE_BALL: f64 = 8.0 + 4.0 * PI / 3.0;

#[test]
fn what_each_ball_shaved_by_a_turned_blocks_face_comes_to() {
    let mut wrong = Vec::new();
    for (off, turned) in DRAWS {
        for (name, policy) in [
            ("certified", Policy::certified()),
            ("tolerant", Policy::tolerant(100.0)),
        ] {
            for ball_first in [false, true] {
                let what = format!("off {off}, turned {turned}, {name}, ball first {ball_first}");
                if let Err(why) = the_union_holds(turned, off, &policy, ball_first, &what) {
                    wrong.push(format!("{what}: {why}"));
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
    for (off, turned) in DRAWS {
        let (model, block, ball) = a_block_and_a_ball_beside_it(turned, off, &policy);
        let skin = model
            .solid(ball)
            .expect("this model made the key")
            .faces()
            .next()
            .expect("a ball has a face")
            .surface();
        let face = the_plane_of_the_turned_face_x_0(&model, block, turned);
        let meet = intersect(&face, &skin, &policy, &mut ProvenanceLog::new());
        assert!(
            matches!(meet, Ok(SurfaceMeet::Curves(ref runs)) if runs.len() == 1),
            "off {off}, turned {turned}: the face's plane meets the ball as {meet:?}"
        );
    }
}

/// Segments `select` found up through the cap, as `(off, turned, [x, y], top)`:
/// one double across in `x` and `y`, from `z = 1` up to `top`.
///
/// In the first two the ray that counted the triangles of the block's stored
/// corners could not settle which side of them the segment was, and the block
/// answered `Undecidable` certified and `OnBoundary` tolerant. In the third
/// those triangles certainly put the segment outside the block, and it
/// answered `Outside`. Printed by a throwaway `eprintln` in `select.rs`,
/// taken out (§346.6).
const SEGMENTS_UP_THROUGH_THE_CAP: [(f64, f64, [f64; 2], f64); 3] = [
    (
        -0.3,
        0.3,
        [0.088_656_061_998_401_81, -0.286_600_946_737_681_7],
        1.000_000_021_073_424_3,
    ),
    (
        -0.5,
        1.0,
        [0.420_735_492_403_948_14, -0.270_151_152_934_069_8],
        1.000_000_007_450_580_6,
    ),
    (
        -0.5,
        1.5,
        [0.498_747_493_302_027_2, -0.035_368_600_833_851_085],
        1.000_000_059_604_644_8,
    ),
];

/// Each segment up through the cap is inside the block, under both policies.
///
/// The block's stored plane `x = 0` puts every point of it on the block's
/// side, a fraction of a spacing in, and that plane is the face (§17.1). The
/// block's other faces are a long way off, so the plane alone says where the
/// segment is (§346.6).
#[test]
fn a_segment_up_through_the_cap_is_inside_the_block() {
    for (off, turned, [x, y], top) in SEGMENTS_UP_THROUGH_THE_CAP {
        let segment = PredicatePoint::Constructed([
            Interval::exactly(x),
            Interval::exactly(y),
            Interval::new(1.0, top).expect("a segment upwards"),
        ]);
        for (name, policy) in [
            ("certified", Policy::certified()),
            ("tolerant", Policy::tolerant(100.0)),
        ] {
            let (model, block, _) = a_block_and_a_ball_beside_it(turned, off, &policy);
            let Surface::Plane(face) = the_plane_of_the_turned_face_x_0(&model, block, turned)
            else {
                unreachable!("the face x = 0 was found by its plane");
            };
            assert_eq!(
                face.side_of_point(PredicatePoint::from([x, y, 1.0])),
                Decision::Neg,
                "off {off}, turned {turned}: the segment is on the block's side of its stored plane"
            );
            let placed = model
                .solid(block)
                .expect("this model made the key")
                .classify_uncertain_point(segment, &policy, &mut ProvenanceLog::new());
            assert_eq!(
                placed,
                Containment::Inside,
                "off {off}, turned {turned}, {name}"
            );
        }
    }
}

fn the_union_holds(
    turned: f64,
    off: f64,
    policy: &Policy,
    ball_first: bool,
    what: &str,
) -> Result<(), String> {
    let (mut model, block, ball) = a_block_and_a_ball_beside_it(turned, off, policy);
    let (arguments, tools) = if ball_first {
        ([ball], [block])
    } else {
        ([block], [ball])
    };
    let answer = model
        .fuse(
            &arguments,
            &tools,
            Rule::Union,
            policy,
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("refused: {why:?}"))?
        .solids()
        .to_vec();
    the_answer_holds(&model, &answer, off, policy, what)
}

/// The union validates; it is one solid where the circle is on the block's
/// face and two, the block and the ball, where it is beside it; it measures
/// within the error it publishes of [`THE_BLOCK_AND_THE_BALL`]; and the ball's
/// skin in it is one face with one hole where the union is one solid, and
/// where it is two, the three pieces of the whole ball, one of them the face
/// with two holes that §402 cuts into strips.
///
/// **Measured summed over the solids, not one by one.** The turn rounds the
/// block's corners, so the block on its own measures a few `1e-15` off `8`,
/// which the error it publishes for its sums does not cover (§338.1). Summed,
/// the error is the ball's mesh's, about `0.13`.
fn the_answer_holds(
    model: &Model,
    answer: &[SolidKey],
    off: f64,
    policy: &Policy,
    what: &str,
) -> Result<(), String> {
    model
        .validate(policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("the answer does not validate: {why:?}"))?;
    let solids = if off < 0.0 { 1 } else { 2 };
    if answer.len() != solids {
        return Err(format!("{} solids where {solids} are right", answer.len()));
    }
    let (mut volume, mut error) = (0.0, 0.0);
    let mut wires_of_each_ball_face = Vec::new();
    for &solid in answer {
        let view = model.solid(solid).expect("the fuse's own key");
        let measured = view
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                policy,
                &mut ProvenanceLog::new(),
            )
            .map_err(|why| format!("the answer does not measure: {why:?}"))?;
        volume += measured.volume();
        error += measured.volume_error();
        wires_of_each_ball_face.extend(
            view.faces()
                .filter(|face| matches!(face.surface(), Surface::Sphere(_)))
                .map(|face| face.wires().count()),
        );
    }
    println!(
        "{what}: {solids} solids measuring {volume} within {error} of {THE_BLOCK_AND_THE_BALL}"
    );
    if (volume - THE_BLOCK_AND_THE_BALL).abs() > error {
        return Err(format!(
            "measures {volume}, {error} off at most, against {THE_BLOCK_AND_THE_BALL}"
        ));
    }
    let wanted: &[usize] = if off < 0.0 { &[1] } else { &[1, 1, 2] };
    let mut wires = wires_of_each_ball_face.clone();
    wires.sort_unstable();
    if wires != wanted {
        return Err(format!(
            "the ball's faces have {wires_of_each_ball_face:?} wires, not {wanted:?}"
        ));
    }
    Ok(())
}

/// The stored plane of the block's face `x = 0`, turned `turned` about `z`,
/// with its normal out of the block.
fn the_plane_of_the_turned_face_x_0(model: &Model, block: SolidKey, turned: f64) -> Surface {
    model
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
        .expect("the block has its face x = 0, turned")
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
