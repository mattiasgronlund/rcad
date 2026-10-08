//! A cube's corner poking into a ball, and the ball's skin left inside the
//! cube: the whole ball less a triangle of three arcs.
//!
//! The cube is turned off the axes, so none of its faces is square to the
//! ball's polar axis, and its corner stands just outside the ball while the
//! three edges from it pass through. Each of the three faces at the corner
//! crosses the ball in a circle and cuts off a cap, and the three caps overlap
//! two at a time but never all three. So the skin inside the cube is two
//! faces: a small triangle of three short arcs between the caps, pointing at
//! the corner, and the rest of the ball, which holds both poles and whose one
//! wire is three long arcs, with the material outside.
//!
//! The mesher opened a ball along a seam and cut it up to both poles only for
//! a rim that is one closed curve. Three arcs are not, so the skin fell through
//! to be filled inside the triangle, the opposite of the face, and the
//! intersection and the cube less the ball were refused
//! `Invalid(FacesDoNotAgreeWhichWayTheyFace)` in both orders. Any rim that
//! passes each of its points once is an island in the opened ball the same
//! way, and is cut the same way now (`docs/decisions.md` §402).
//!
//! The volumes are the closed forms: the ball inside the cube is the ball less
//! the three caps the cube's faces cut off, plus the three pieces beyond two
//! faces at once, which the caps took off twice. The corner is outside the
//! ball, so nothing is beyond all three.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

const BALL: f64 = 4.0 * PI / 3.0;

/// Q443's draw: a cube of side 2 turned one radian about `(1, 1, 1)`, and a
/// ball of radius 1 at `(0.62, 0.71, 0.66)` in the cube's frame.
#[test]
fn a_turned_cubes_corner_in_a_ball_measures_its_closed_form_under_every_rule() {
    let centre = [0.62, 0.71, 0.66];
    let shared = what_the_cube_and_the_ball_share(centre);
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for ball_first in [false, true] {
            let truth = match (rule, ball_first) {
                (Rule::Union, _) => 8.0 + BALL - shared,
                (Rule::Difference, false) => 8.0 - shared,
                (Rule::Difference, true) => BALL - shared,
                (Rule::Intersection, _) => shared,
                (other, _) => panic!("no closed form written for {other:?}"),
            };
            let what = format!("{rule:?}, ball first {ball_first}");
            let (model, answer) = fused(centre, rule, ball_first, &what);
            assert_measures(&model, &answer, truth, &what);
        }
    }
}

/// What makes the test above reach the cut it is about: the intersection's
/// skin is two faces on the ball, each bounded by one wire of three arcs. The
/// three caps the cube's faces cut off overlap two at a time and never all
/// three, so between them they leave a small triangle of skin pointing at the
/// corner, and the rest of the ball, which is the face that holds both poles.
#[test]
fn the_intersections_skin_is_two_faces_each_bounded_by_three_arcs() {
    let centre = [0.62, 0.71, 0.66];
    let (model, answer) = fused(centre, Rule::Intersection, false, "the intersection");
    let [solid] = answer[..] else {
        panic!("one piece of the ball, got {answer:?}");
    };
    let wires: Vec<Vec<usize>> = model
        .solid(solid)
        .expect("the fuse's own key")
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Sphere(_)))
        .map(|skin| skin.wires().map(|wire| wire.coedges().count()).collect())
        .collect();
    assert_eq!(wires, [[3], [3]], "two faces of one wire of three arcs");
    let shared = what_the_cube_and_the_ball_share(centre);
    assert!(shared > BALL / 2.0, "{shared} is more than half the ball");
}

fn fused(centre: [f64; 3], rule: Rule, ball_first: bool, what: &str) -> (Model, Vec<SolidKey>) {
    let turn = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(1.0, 1.0, 1.0)).expect("an axis"),
        1.0,
    )
    .expect("a finite turn");
    let mut model = Model::new(Scale::millimetres(1.0).expect("1 mm is a working size"));
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(cube, &turn)
        .expect("this model made the key");
    let at = turn.apply_to_point(Point::new(centre[0], centre[1], centre[2]));
    let ball = model
        .add_ball(
            1.0,
            &Isometry::translation(at - Point::origin()).expect("a finite placement"),
        )
        .expect("a ball");
    let (arguments, tools) = if ball_first {
        (ball, cube)
    } else {
        (cube, ball)
    };
    let answer = model
        .fuse(
            &[arguments],
            &[tools],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: refused {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
    (model, answer)
}

/// The answer's volume, summed over its solids, lands within the bound it
/// publishes for itself of `truth`.
fn assert_measures(model: &Model, answer: &[SolidKey], truth: f64, what: &str) {
    let (mut volume, mut error) = (0.0, 0.0);
    for &solid in answer {
        let measured = model
            .solid(solid)
            .expect("the fuse's own key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: the answer measures: {why:?}"));
        volume += measured.volume();
        error += measured.volume_error();
    }
    assert!(
        (volume - truth).abs() <= error,
        "{what}: {volume} measured, {error} off at most, against {truth}"
    );
}

/// The ball of radius 1 at `centre` inside the cube `[0, 2]³`, for a centre
/// whose ball reaches none of the planes `x = 2`, `y = 2`, `z = 2` and leaves
/// the corner at the origin outside.
fn what_the_cube_and_the_ball_share([x, y, z]: [f64; 3]) -> f64 {
    BALL - cap(x) - cap(y) - cap(z) + beyond_two(x, y) + beyond_two(x, z) + beyond_two(y, z)
}

/// The cap of the unit ball beyond a plane `h` from its centre.
fn cap(h: f64) -> f64 {
    PI * (1.0 - h).powi(2) * (2.0 + h) / 3.0
}

/// The piece of the unit ball at the origin with `u > a` and `v > b`, for
/// `a, b > 0` with `a² + b² < 1`: the slice at `u` is the segment of a disc of
/// radius `√(1 − u²)` beyond `b`, summed from `a` out.
fn beyond_two(a: f64, b: f64) -> f64 {
    if a * a + b * b >= 1.0 {
        return 0.0;
    }
    let s = (1.0 - a * a - b * b).sqrt();
    2.0 / 3.0 * (s / (a * b)).atan() + 2.0 / 3.0 * a * b * s
        - (a - a.powi(3) / 3.0) * (b / (1.0 - a * a).sqrt()).acos()
        - (b - b.powi(3) / 3.0) * (a / (1.0 - b * b).sqrt()).acos()
}

/// The closed form above, against the slices it sums added up one by one.
#[test]
fn the_piece_beyond_two_planes_is_the_sum_of_its_slices() {
    for [a, b] in [[0.62_f64, 0.71], [0.4, 0.4], [0.6, 0.3], [0.1, 0.9]] {
        let end = (1.0 - b * b).sqrt();
        let steps = 200_000;
        let width = (end - a) / f64::from(steps);
        let summed: f64 = (0..steps)
            .map(|step| {
                let u = a + (f64::from(step) + 0.5) * width;
                let r = (1.0 - u * u).sqrt();
                (r * r * (b / r).acos() - b * (r * r - b * b).sqrt()) * width
            })
            .sum();
        assert!(
            (summed - beyond_two(a, b)).abs() < 1e-9,
            "{a}, {b}: {summed} summed, {} closed",
            beyond_two(a, b)
        );
    }
}
