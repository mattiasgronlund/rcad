//! A ball with a bar along a line through its centre, and a thin bar through a
//! fat one, fused under every rule both ways round, and each answer fused
//! again with a cube far from it.
//!
//! **Each pair crosses in two closed loops.** The ball of radius 5 at the
//! origin and a bar of radius 3 through its centre cross in two circles, where
//! the bar's axis is `±4` from the centre. The bar of radius 1 along `x` at
//! `z = 5` and the bar of radius 4 up `z` cross in two loops, one either side
//! of the plane `x = 0`. `intersect` picks each loop out by a flat half-space:
//! through the ball's centre square to the bar's axis, or through the fat bar's
//! axis square to the thin one's.
//!
//! **Every answer that kept a face along both loops was refused fused again
//! with the cube**, `ResultIsNotManifold { count: 4 }` in every order, as were
//! the union and the ball less the bar where the bar ends at the ball's centre.
//! `fuse/assemble.rs` named a closed crossing by its two surfaces alone for
//! these pairs, so the two loops were one curve (row Q1394). The cone of
//! `tests/a_pointed_cone_drilled_along_its_axis.rs` was refused the same way
//! for the same reason (Q1375). A **gap** (§16.10): each loop is now named by
//! the side it is on of the half-space `intersect` picks it by, and every one
//! of these answers.
//!
//! **A thin bar that ends at the fat one's axis was refused before any of
//! that**, in its first fuse, under every rule both ways round, because the
//! plane of its cap parts the fat wall in two halves that the loop's outside
//! could not be told between (row Q1777). Another **gap**: a surface that cuts
//! a face is drawn wherever it crosses the inside of the face, so it has one
//! side over each region of the inside, and the halves are on its two sides.
//!
//! The closed forms. Inside the bar of radius 3 the ball holds the whole disk
//! of radius 3 out to `4` from its centre, and a disk of radius `√(25 − x²)`
//! from there to `5`, so the two share `36π + 14π/3 = 122π/3` either side of
//! the centre. The thin bar and the fat one share
//! `8∫₀¹ √((16 − y²)(1 − y²)) dy` ([`what_a_thin_bar_through_a_fat_one_shares`]).

use std::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

const BALL: f64 = 500.0 * PI / 3.0;
/// What the ball shares with the bar of radius 3 on one side of its centre.
const HALF_SHARED: f64 = 122.0 * PI / 3.0;

/// The bar of radius 3 along `x` from the ball's centre out to `x = 10`,
/// `90π`.
///
/// The union `500π/3 + 90π − 122π/3 = 216π` keeps one circle, at `x = 4`, and
/// so does the ball less the bar, `500π/3 − 122π/3 = 126π`; the ball's skin
/// holds the other circle, at `x = −4`, whole. Both were refused fused again
/// with the cube. The intersection, `122π/3`, and the bar less the ball,
/// `90π − 122π/3 = 148π/3`, were not.
#[test]
fn a_bar_ending_at_a_balls_centre_and_each_answer_beside_a_far_cube_hold_their_closed_forms() {
    every_rule_and_a_far_cube_hold_their_closed_forms(
        "a bar ending at a ball's centre",
        |model| [ball(model), bar_along_x(model, 3.0, 0.0, 10.0, 0.0)],
        [
            &[216.0 * PI],
            &[HALF_SHARED],
            &[BALL - HALF_SHARED],
            &[90.0 * PI - HALF_SHARED],
        ],
    );
}

/// The bar of radius 3 up `z` from the ball's centre out to `z = 10`, which
/// runs through the ball's pole at `z = 5`: the same closed forms as
/// [`a_bar_ending_at_a_balls_centre_and_each_answer_beside_a_far_cube_hold_their_closed_forms`],
/// with the ball's other pole, at `z = −5`, inside the circle the ball's skin
/// holds whole.
#[test]
fn a_bar_ending_at_a_balls_centre_through_its_pole_and_each_answer_beside_a_far_cube_hold_their_closed_forms()
 {
    every_rule_and_a_far_cube_hold_their_closed_forms(
        "a bar ending at a ball's centre through its pole",
        |model| {
            let bar = model
                .add_round_bar(3.0, 10.0, &Isometry::identity())
                .expect("a bar of radius three");
            [ball(model), bar]
        },
        [
            &[216.0 * PI],
            &[HALF_SHARED],
            &[BALL - HALF_SHARED],
            &[90.0 * PI - HALF_SHARED],
        ],
    );
}

/// The bar of radius 3 along `x` from `x = −10` to `x = 10`, `180π`, right
/// through the ball.
///
/// Every answer keeps both circles: the union `500π/3 + 180π − 244π/3 =
/// 796π/3`, the intersection `244π/3`, the ball less the bar `256π/3`, and the
/// bar less the ball two ends of `148π/3`. Every one was refused fused again
/// with the cube.
#[test]
fn a_bar_right_through_a_ball_and_each_answer_beside_a_far_cube_hold_their_closed_forms() {
    let one_end = 90.0 * PI - HALF_SHARED;
    every_rule_and_a_far_cube_hold_their_closed_forms(
        "a bar right through a ball",
        |model| [ball(model), bar_along_x(model, 3.0, -10.0, 20.0, 0.0)],
        [
            &[796.0 * PI / 3.0],
            &[2.0 * HALF_SHARED],
            &[BALL - 2.0 * HALF_SHARED],
            &[one_end, one_end],
        ],
    );
}

/// The bar of radius 1 along `x` at `z = 5` from `x = −10` to `x = 10`, `20π`,
/// right through the bar of radius 4 up `z` from `z = 0` to `z = 10`, `160π`.
///
/// Every answer keeps both loops: the union `180π − I`, the intersection `I`,
/// the fat bar less the thin one `160π − I`, and the thin bar less the fat one
/// two ends of `(20π − I)/2`, with `I` what the two share. Every one was
/// refused fused again with the cube.
#[test]
fn a_thin_bar_right_through_a_fat_one_and_each_answer_beside_a_far_cube_hold_their_closed_forms() {
    let shared = what_a_thin_bar_through_a_fat_one_shares();
    let one_end = (20.0 * PI - shared) / 2.0;
    every_rule_and_a_far_cube_hold_their_closed_forms(
        "a thin bar right through a fat one",
        |model| [fat_bar(model), bar_along_x(model, 1.0, -10.0, 20.0, 5.0)],
        [
            &[180.0 * PI - shared],
            &[shared],
            &[160.0 * PI - shared],
            &[one_end, one_end],
        ],
    );
}

/// The bar of radius 1 along `x` at `z = 5` from `x = −10` to the fat bar's
/// axis at `x = 0`, `10π`, crosses the fat bar's wall in one of the two loops,
/// its cap inside the fat bar.
///
/// The thin bar keeps the half of `I` on its side of `x = 0`, so the two share
/// `I/2`: the union `170π − I/2`, the intersection `I/2`, the fat bar less the
/// thin one `160π − I/2`, and the thin bar less the fat one `10π − I/2`, one
/// solid.
///
/// Every rule both ways round was refused `CannotTellTheCellsOfAFaceApart` in
/// the first fuse (Q1777). The plane of the thin bar's cap cuts the fat wall in
/// two straight lines that part it into the half at `x > 0` and the half at
/// `x < 0`, and the walk round the outside of the loop at `x = 4` agreed with
/// both halves about every closed curve.
#[test]
fn a_thin_bar_ending_at_a_fat_ones_axis_and_each_answer_beside_a_far_cube_hold_their_closed_forms()
{
    let half = what_a_thin_bar_through_a_fat_one_shares() / 2.0;
    every_rule_and_a_far_cube_hold_their_closed_forms(
        "a thin bar ending at a fat one's axis",
        |model| [fat_bar(model), bar_along_x(model, 1.0, -10.0, 10.0, 5.0)],
        [
            &[170.0 * PI - half],
            &[half],
            &[160.0 * PI - half],
            &[10.0 * PI - half],
        ],
    );
}

/// Fuses the pair `build` makes under every rule both ways round, then each
/// answer with the cube of side two at `(20, 20, 20)` both ways round, and
/// checks every answer passes [`Model::validate`] and holds `want`: the union,
/// the intersection, the first less the second and the second less the first,
/// one volume per solid, with the cube's `8` beside them after the cube.
fn every_rule_and_a_far_cube_hold_their_closed_forms(
    what: &str,
    build: fn(&mut Model) -> [SolidKey; 2],
    want: [&[f64]; 4],
) {
    for (rule, swapped) in every_rule_both_ways() {
        let want = match (rule, swapped) {
            (Rule::Union, _) => want[0],
            (Rule::Intersection, _) => want[1],
            (Rule::Difference, false) => want[2],
            (Rule::Difference, true) => want[3],
            (other, _) => panic!("a rule this file does not fuse: {other:?}"),
        };
        let what = format!("{what}, {rule:?} (swapped: {swapped})");
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let [first, second] = build(&mut model);
        let (arguments, tools) = if swapped {
            ([second], [first])
        } else {
            ([first], [second])
        };
        let answer = fused(&mut model, &arguments, &tools, rule, &what);
        holds(&model, &answer, want, &what);
        let cube = model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a whole-number cube");
        model
            .place_solid(cube, &at(20.0, 20.0, 20.0))
            .expect("this model made the key");
        let with_the_cube: Vec<f64> = want.iter().copied().chain([8.0]).collect();
        for answer_first in [true, false] {
            let (arguments, tools) = if answer_first {
                (answer.clone(), vec![cube])
            } else {
                (vec![cube], answer.clone())
            };
            let what = format!("{what} and a far cube, answer first: {answer_first}");
            let mut again = model.clone();
            let both = fused(&mut again, &arguments, &tools, Rule::Union, &what);
            holds(&again, &both, &with_the_cube, &what);
        }
    }
}

fn every_rule_both_ways() -> impl Iterator<Item = (Rule, bool)> {
    [Rule::Union, Rule::Intersection, Rule::Difference]
        .into_iter()
        .flat_map(|rule| [false, true].map(move |swapped| (rule, swapped)))
}

/// The solids `arguments` fused with `tools` under `rule` gives, once the model
/// validates.
fn fused(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
    what: &str,
) -> Vec<SolidKey> {
    let answer = model
        .fuse(
            arguments,
            tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
    answer.solids().to_vec()
}

/// Checks `solids` hold `want` between them, one volume each, in any order,
/// each within what [`Model`]'s measure says its volume may be off by.
fn holds(model: &Model, solids: &[SolidKey], want: &[f64], what: &str) {
    let mut got: Vec<(f64, f64)> = solids
        .iter()
        .map(|&solid| {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what}: a solid measures: {why:?}"));
            (measured.volume(), measured.volume_error())
        })
        .collect();
    got.sort_by(|one, other| one.0.total_cmp(&other.0));
    let mut want = want.to_vec();
    want.sort_by(f64::total_cmp);
    assert_eq!(
        got.len(),
        want.len(),
        "{what}: solids holding {got:?}, where the shapes say {want:?}"
    );
    for (&(volume, error), want) in got.iter().zip(&want) {
        assert!(
            (volume - want).abs() <= error,
            "{what}: holds {volume} give or take {error}, where the shapes say {want}"
        );
    }
}

/// `8∫₀¹ √((16 − y²)(1 − y²)) dy`, what the bar of radius 1 along `x` at
/// `z = 5` shares with the bar of radius 4 up `z` it runs right through: at
/// `y` the thin bar is `2√(1 − y²)` tall and the fat one `2√(16 − y²)` long.
///
/// Worked by the midpoint rule in `t` with `y = sin t`, as
/// `tests/two_arcs_where_two_curved_surfaces_cross.rs` works the half of it:
/// the integrand `cos²t·√(16 − sin²t)` is smooth and its odd derivatives
/// vanish at both ends of `[0, π/2]`, so 2000 steps settle it.
fn what_a_thin_bar_through_a_fat_one_shares() -> f64 {
    let steps = 2000_u32;
    let step = PI / 2.0 / f64::from(steps);
    (0..steps)
        .map(|at| {
            let t = (f64::from(at) + 0.5) * step;
            t.cos().powi(2) * (16.0 - t.sin().powi(2)).sqrt()
        })
        .sum::<f64>()
        * step
        * 8.0
}

/// The ball of radius 5 at the origin.
fn ball(model: &mut Model) -> SolidKey {
    model
        .add_ball(5.0, &Isometry::identity())
        .expect("a ball of radius five")
}

/// The bar of radius 4 up `z` from `z = 0` to `z = 10`.
fn fat_bar(model: &mut Model) -> SolidKey {
    model
        .add_round_bar(4.0, 10.0, &Isometry::identity())
        .expect("a bar of radius four")
}

/// The bar of `radius` along `x` at `z`, from `x = from` for `length`.
fn bar_along_x(model: &mut Model, radius: f64, from: f64, length: f64, z: f64) -> SolidKey {
    model
        .add_round_bar(
            radius,
            length,
            &at(from, 0.0, z)
                .compose(&Isometry::quarter_turns(&Axis::y(), 1).expect("y is an axis")),
        )
        .expect("a bar along x")
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
