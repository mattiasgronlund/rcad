//! A ball centred on a cube's face, beside the edge the face shares with the
//! next one, so the next face's plane cuts it in a circle that the edge
//! crosses through its middle (Q2403, from corpus seed `0fb776ff` of
//! `two_solids_fused_by_a_stranger`).
//!
//! The cube is `[0, 2]³` and the ball of radius `r` is centred at `(d, 0, 1)`,
//! on the face `y = 0`. The plane `x = 0` cuts the ball in a circle round
//! `(0, 0, 1)`, and the edge `x = y = 0` crosses that circle at its bottom and
//! its top: two places on the circle, half a turn apart. The circle is walked
//! from its bottom, which is a rounding off the bottom place, so the start and
//! the two places lie on one line and sweep no area. Nothing the boxes or the
//! planes ask put the two in order, and the policy was asked. Read as wide as
//! the wider box, a certified policy refused the order and a tolerant one took
//! the two places, two apart, as one point, and every fuse was refused
//! `CannotTellTheCellsOfAFaceApart`.
//!
//! Two places on a loop cut it into the same two arcs whichever comes first,
//! and boxes clear of each other hold two points, so the pair is now left in
//! the order it is numbered and nothing is asked.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, Rule, SolidKey};

/// Each ball centred on the face `y = 0` whose union with the cube was refused
/// under both policies, both ways round, and now answers with its volume. The
/// ball at a tenth from the edge reaches the cube's floor and roof; the one of
/// radius `0.9` reaches neither.
#[test]
fn a_ball_centred_on_a_cubes_face_by_its_edge_unites_with_its_volume() {
    for (d, r) in [(0.1, 1.0), (-0.1, 1.0), (1e-3, 1.0), (1e-3, 0.9)] {
        for certified in [false, true] {
            for ball_first in [false, true] {
                let what = format!(
                    "a ball of radius {r} at ({d}, 0, 1), certified {certified}, ball first \
                     {ball_first}"
                );
                let (mut model, cube, ball) = a_cube_and_a_ball([d, 0.0, 1.0], r);
                let policy = the_policy(certified);
                let answer = united(&mut model, cube, ball, ball_first, &policy)
                    .unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
                model
                    .validate(&policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                let [solid] = answer[..] else {
                    panic!("{what}: one solid, got {answer:?}");
                };
                let measured = model
                    .solid(solid)
                    .expect("this model made the key")
                    .measure(
                        Deflection::of(0.01).expect("a positive length"),
                        &policy,
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| panic!("{what}: the union measures: {why:?}"));
                let truth = the_union_of_the_cube_and_the_ball(d, r);
                assert!(
                    (measured.volume() - truth).abs() <= measured.volume_error(),
                    "{what}: {} measured, {} off, against {truth}",
                    measured.volume(),
                    measured.volume_error()
                );
            }
        }
    }
}

/// The seed itself, the ball of radius 1 centred `2⁻⁶⁴` off the face `x = 0`.
/// Its circle on that face crosses the edge at `z = 2⁻¹²⁹` and `z = 2`, walked
/// from the corner `(0, 0, 0)`, and its answer has a point `2⁻¹²⁹` from a
/// corner. That is `1.4e39` times smaller than the answer is wide, past the
/// `1e11` a model may span (§3.5), so a certified policy refuses it by name
/// where it refused the order before.
#[test]
fn the_seeds_ball_a_hair_off_the_face_spans_more_than_a_model_may() {
    for ball_first in [false, true] {
        let (mut model, cube, ball) = a_cube_and_a_ball([2f64.powi(-64), 0.0, 1.0], 1.0);
        let refused = united(&mut model, cube, ball, ball_first, &Policy::certified());
        assert!(
            matches!(
                refused,
                Err(Error::Invalid(Invalid::DynamicRangeTooWide { .. }))
            ),
            "ball first {ball_first}: {refused:?}"
        );
    }
}

/// The cube `[0, 2]³` and a ball of radius `r` centred at `at`, in a model of
/// working size 1, as the seed is.
fn a_cube_and_a_ball(at: [f64; 3], r: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(1.0).expect("a working size"));
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    let [x, y, z] = at;
    let ball = model
        .add_ball(
            r,
            &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        )
        .expect("a ball");
    (model, cube, ball)
}

fn the_policy(certified: bool) -> Policy {
    if certified {
        Policy::certified()
    } else {
        Policy::tolerant(1.0)
    }
}

fn united(
    model: &mut Model,
    cube: SolidKey,
    ball: SolidKey,
    ball_first: bool,
    policy: &Policy,
) -> Result<Vec<SolidKey>, Error> {
    let (arguments, tools) = if ball_first {
        (ball, cube)
    } else {
        (cube, ball)
    };
    model
        .fuse(
            &[arguments],
            &[tools],
            Rule::Union,
            policy,
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec())
}

/// The cube and the ball of radius `r ≤ 1` centred at `(d, 0, 1)`, with
/// `|d| < r`. The ball's height lies inside the cube's, so the part inside the
/// cube is the half with `y ≥ 0` of the cap with `x ≥ 0`, of height `r + d`.
fn the_union_of_the_cube_and_the_ball(d: f64, r: f64) -> f64 {
    let height = r + d;
    let cap = PI * height * height * (3.0 * r - height) / 3.0;
    8.0 + 4.0 * PI * r.powi(3) / 3.0 - cap / 2.0
}
