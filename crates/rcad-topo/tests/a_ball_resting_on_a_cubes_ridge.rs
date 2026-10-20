//! A ball resting exactly on the ridge of a turned cube.
//!
//! The cube is the 2 by 2 square extruded 2, turned `3π/4` about `y` through
//! the origin, so `(1, 0, 1)` goes below `z = 0` and its edge along `y` at the
//! origin is a ridge between two faces leaning 45 degrees either way. A ball of
//! radius 1 centred at `(0, 1, 1)` touches that ridge at `(0, 1, 0)`, which is
//! the ball's south pole, and shares nothing else with the cube: each face's
//! plane cuts the ball in a circle through the pole, and both circles lie
//! outside the faces. The union is the two of them, `8 + 4π/3` between them,
//! and the ball less the cube is the ball whole.
//!
//! The fuse cuts the ball's skin along both whole circles, into two caps and
//! the face between them, whose one wire is a figure eight through the pole.
//! Each circle is a closed edge with a piece of the ball on both sides and no
//! flat face along it. At the pole there is no longitude to order the edge by,
//! so each piece reads its way round against the flat lid that would close it
//! (`the_lid_a_circle_from_a_pole_bounds` in `fuse/assemble.rs`). The cap lies
//! on the cube's side of the plane and the middle face on the other, so their
//! lids point opposite ways, and **read each against its own lid, both pieces
//! walked the circle the same way round.** Meshing the middle face then drew
//! the side from the pole along the first circle as the cap does, and both
//! fuses that keep the ball were refused `TwoPatchesOfTheMeshDrewTheSameSide {
//! drawn: 2, used: 4 }` at either size, the union either way round (Q2485).
//!
//! Now `every_closed_edge_read_off_one_spelling` reads each piece again against
//! the ball and the cube's face as the cube stores it, which is the cap's lid,
//! so the middle face is turned and the two walk each circle opposite ways.
//! Where a flat face lies along such a circle, as in
//! `a_ball_cut_through_its_pole.rs`, the lid each ball piece reads is that flat
//! face, and nothing is read again.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Fused, Measurement, Model, ProfileBuilder, Rule};

/// The union is the cube and the ball apart: two solids that validate, one
/// measuring 8 and the other `4π/3`, each within the error its own mesh
/// declares, at either working size and either way round.
#[test]
fn uniting_them_keeps_the_cube_and_the_ball_as_two_solids() {
    for size in [100.0, 1.0] {
        for ball_first in [true, false] {
            let what = format!("size {size}, ball first {ball_first}");
            let measured = measure_each(fuse(size, Rule::Union, ball_first), &what);
            let [ball, cube] = measured.as_slice() else {
                panic!("{what}: two solids, not {}", measured.len());
            };
            assert_holds(ball, 4.0 * PI / 3.0, 0.0, &format!("{what}: the ball"));
            // The cube's corners were turned by 3π/4 and rounded, so its own
            // triangles hold 8 only to that rounding. Its declared error is the
            // rounding of the sum over them alone, so the corners' is added.
            assert_holds(cube, 8.0, 8.0 * 1e-12, &format!("{what}: the cube"));
        }
    }
}

/// The ball less the cube is the ball, still cut along both circles: one solid
/// that validates and measures `4π/3` within the error its mesh declares, at
/// either working size.
#[test]
fn the_ball_less_the_cube_is_the_whole_ball() {
    for size in [100.0, 1.0] {
        let what = format!("size {size}");
        let measured = measure_each(fuse(size, Rule::Difference, true), &what);
        let [ball] = measured.as_slice() else {
            panic!("{what}: one solid, not {}", measured.len());
        };
        assert_holds(ball, 4.0 * PI / 3.0, 0.0, &what);
    }
}

/// A ball of radius 5 about `(3, 1, 4)` rests on the same ridge at `(0, 1,
/// 0)`, which is no pole of it, and the union is the cube and the ball apart:
/// two solids that validate, measuring 8 and `500π/3`, at either working size
/// and either way round. It was refused, `CornersCoincide` with the ball first
/// and `BoundaryCrossesItself` with the cube first (Q2492).
///
/// The face `x = z` cuts the ball in a circle whose lowest point is the ridge
/// point, and the walk round that circle starts a rounding below it, at `z =
/// −4.4e-16`, outside the circle. Seen from there the ridge point lies straight
/// up, as the far side of the circle at `y = 1` does, so ordered round the
/// circle by the sweep it sat on the far side, between the corners at `y = 2`
/// and those at `y = 0`. The ball's edges along that circle then ran the wrong
/// way round it: one from `(0.072, 2, 0.072)` to `(0.072, 0, 0.072)` passed
/// the ridge point, and the two that ended there overlapped the whole circle
/// between them, so the walk round the ball's face beside the cube's small
/// circle visited the ridge point twice at one place of the chart. Now the
/// ridge point is read as the place the start was rounded off, and comes
/// first (Q2499).
#[test]
fn uniting_the_ball_off_its_poles_keeps_the_cube_and_the_ball_as_two_solids() {
    for size in [100.0, 1.0] {
        for ball_first in [true, false] {
            let what = format!("size {size}, ball first {ball_first}");
            let measured = measure_each(
                fuse_the_ball_off_its_poles(size, Rule::Union, ball_first),
                &what,
            );
            let [cube, ball] = measured.as_slice() else {
                panic!("{what}: two solids, not {}", measured.len());
            };
            assert_holds(cube, 8.0, 8.0 * 1e-12, &format!("{what}: the cube"));
            assert_holds(ball, 500.0 * PI / 3.0, 0.0, &format!("{what}: the ball"));
        }
    }
}

/// The ball of radius 5 less the cube it rests on is the ball: one solid that
/// validates and measures `500π/3` within the error its mesh declares, at
/// either working size. It was refused as the union with the ball first was
/// (Q2492, Q2499).
#[test]
fn the_ball_off_its_poles_less_the_cube_is_the_whole_ball() {
    for size in [100.0, 1.0] {
        let what = format!("size {size}");
        let measured = measure_each(
            fuse_the_ball_off_its_poles(size, Rule::Difference, true),
            &what,
        );
        let [ball] = measured.as_slice() else {
            panic!("{what}: one solid, not {}", measured.len());
        };
        assert_holds(ball, 500.0 * PI / 3.0, 0.0, &what);
    }
}

/// Checks the model a fuse answered in validates, and measures each solid of
/// the answer, smallest volume first.
fn measure_each((model, fused): (Model, rcad_topo::Result<Fused>), what: &str) -> Vec<Measurement> {
    let fused = fused.unwrap_or_else(|why| panic!("{what}: the fuse is answered: {why}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the answer validates: {why}"));
    let mut measured: Vec<_> = fused
        .solids()
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.001).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what}: a solid measures: {why:?}"))
        })
        .collect();
    measured.sort_by(|one, other| one.volume().total_cmp(&other.volume()));
    measured
}

/// `measured` lands within its own declared error, and `slack` more, of
/// `closed_form`.
fn assert_holds(measured: &Measurement, closed_form: f64, slack: f64, what: &str) {
    assert!(
        (measured.volume() - closed_form).abs() <= measured.volume_error() + slack,
        "{what} measures {}, closed form {closed_form}, declared {}",
        measured.volume(),
        measured.volume_error()
    );
}

fn fuse(size: f64, rule: Rule, ball_first: bool) -> (Model, rcad_topo::Result<Fused>) {
    fuse_a_ball(1.0, Vector::new(0.0, 1.0, 1.0), size, rule, ball_first)
}

fn fuse_the_ball_off_its_poles(
    size: f64,
    rule: Rule,
    ball_first: bool,
) -> (Model, rcad_topo::Result<Fused>) {
    fuse_a_ball(5.0, Vector::new(3.0, 1.0, 4.0), size, rule, ball_first)
}

fn fuse_a_ball(
    radius: f64,
    centre: Vector,
    size: f64,
    rule: Rule,
    ball_first: bool,
) -> (Model, rcad_topo::Result<Fused>) {
    let mut model = Model::new(Scale::millimetres(size).expect("a working size"));
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let turn =
        Isometry::rotation(&Axis::y(), 3.0 * core::f64::consts::FRAC_PI_4).expect("a finite turn");
    let cube = model
        .add_extrusion(&square, 2.0, &turn)
        .expect("a positive height");
    let ball = model
        .add_ball(
            radius,
            &Isometry::translation(centre).expect("a finite shift"),
        )
        .expect("a positive radius");
    let (one, other) = if ball_first {
        (ball, cube)
    } else {
        (cube, ball)
    };
    let policy = Policy::tolerant(model.scale().nominal_size());
    let fused = model.fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new());
    (model, fused)
}
