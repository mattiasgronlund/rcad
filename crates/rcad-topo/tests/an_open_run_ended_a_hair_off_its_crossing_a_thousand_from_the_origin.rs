//! An open edge whose end sits a hair off the place its crossing is walked
//! from, so the walk leaves a marched point beside that end (Q1697).
//!
//! A ball and a bar off its axis cross in two loops, and each is walked from a
//! seed on the plane through the bar's axis and the ball's centre
//! (`rcad-geom`'s `where_a_ball_reaches_across_a_pipe`). A block's wall on that
//! plane cuts each loop there, so the open edge the cut leaves ends where its
//! crossing is walked from. A mesh shifts a face stored far from the origin
//! until its numbers are small and walks the edge again there, so the end,
//! rounded where it was stored, lands `2.1e-14` and `2.2e-14` off the place the
//! walk starts from in the shifted frame, a thousand from the origin, where the
//! march bounds its points to `8.0e-15` and `6.0e-15`. `rcad-geom`'s
//! `where_on_the_loop_this_end_is` judged the end by its own box, kept the
//! start beside it, and the mesh the fuse checks its answer with refused the
//! two as two corners in one place (`CornersCoincide`). The end is now judged
//! by how far off the crossing it can be, which reaches the start, and the
//! start is left out, as Q1690 did for a closed edge's corner.
//!
//! Of the cut at the origin and a thousand from it, with the bar cut by the
//! block first or by the ball, only the cut a thousand from the origin by the
//! block first answers. At the origin it is refused ordering two ends of a loop
//! along it, and by the ball first a thousand from the origin it is refused a
//! face it cannot follow; the last two tests pin both. So the answer is checked
//! against its closed form, and not against the origin's.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

const BALL: f64 = 5.0;
const BAR: f64 = 4.0;
const OFF: f64 = 0.5;

/// The two blocks, one each side of the wall through the bar's axis and the
/// ball's centre, measured from the bar's axis: `low` and `high` corners.
const EITHER_SIDE: [(&str, [f64; 3], [f64; 3]); 2] = [
    ("y ≥ 0", [-20.0, 0.0, -20.0], [20.0, 20.0, 20.0]),
    ("y ≤ 0", [-20.0, -20.0, -20.0], [20.0, 0.0, 20.0]),
];

/// A bar of radius 4 that reaches past both loops it crosses a ball of radius 5
/// in, half a unit off the ball's centre, cut by a block whose wall is the
/// plane through the bar's axis and the ball's centre, a thousand from the
/// origin: what the three share measures half what the ball and the bar share,
/// within the error the measurement gives, whichever side of the wall the block
/// is on.
#[test]
fn a_ball_and_a_bar_cut_by_a_wall_through_where_their_loops_are_walked_from_measure_half_of_what_they_share()
 {
    let half = what_a_ball_off_a_bars_axis_shares(BALL, BAR, OFF) / 2.0;
    for (side, low, high) in EITHER_SIDE {
        let (model, answer) = cut(1000.0, low, high, Cuts::BlockFirst);
        let answer = answer.unwrap_or_else(|why| panic!("{side}: {why:?}"));
        let [solid] = answer[..] else {
            panic!("one solid on {side}, got {answer:?}");
        };
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a fine mesh"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("the answer measures");
        eprintln!(
            "{side}: {} against {half}, error {}",
            measured.volume(),
            measured.volume_error()
        );
        assert!(
            (measured.volume() - half).abs() <= measured.volume_error(),
            "{side}: {} against {half}, error {}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// The same cut at the origin is refused ordering two corners the wall puts on
/// one loop, `(−4, 0, 7.18)` and `(4, 0, 8.57)`, whichever side of the wall the
/// block is on and whichever the bar is cut by first. The answer exists: a
/// thousand from the origin it measures half of what the ball and the bar
/// share. Pinned as it stands, to move with the fix.
#[test]
fn the_same_cut_at_the_origin_is_refused_ordering_two_ends_of_a_loop_along_it() {
    for (side, low, high) in EITHER_SIDE {
        for cuts in [Cuts::BlockFirst, Cuts::BallFirst] {
            let (_, said) = cut(0.0, low, high, cuts);
            let Err(Error::Undecidable { records }) = said else {
                panic!("{side}, {cuts:?}: {said:?}");
            };
            let first = &records.records()[0];
            eprintln!("{side}, {cuts:?}: {first:?}");
            assert_eq!(
                first.predicate(),
                "order along a crossing",
                "{side}, {cuts:?}"
            );
        }
    }
}

/// With the bar cut by the ball first, the cut a thousand from the origin is
/// refused a face it cannot follow, whichever side of the wall the block is
/// on. The answer exists: cut by the block first, it measures half of what the
/// ball and the bar share. Pinned as it stands, to move with the fix.
#[test]
fn the_same_cut_with_the_ball_first_is_refused_a_face_it_cannot_follow() {
    for (side, low, high) in EITHER_SIDE {
        let (_, said) = cut(1000.0, low, high, Cuts::BallFirst);
        eprintln!("{side}: {said:?}");
        assert!(
            matches!(
                said,
                Err(Error::AFaceTrimmedByACurveThisCannotFollow { .. })
            ),
            "{side}: {said:?}"
        );
    }
}

/// Which of the ball and the block cuts the bar first.
#[derive(Clone, Copy, Debug)]
enum Cuts {
    BlockFirst,
    BallFirst,
}

/// The bar of radius 4 up `z` from `z = −1` to `11`, the ball of radius 5 at
/// `(0.5, 0, 5)`, and a block from `low` to `high`, each moved `far` along
/// every axis, and the three intersected, the bar with the one `cuts` names
/// first and the answer with the other.
fn cut(
    far: f64,
    low: [f64; 3],
    high: [f64; 3],
    cuts: Cuts,
) -> (Model, rcad_topo::Result<Vec<SolidKey>>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(BAR, 12.0, &at(far, far, far - 1.0))
        .expect("a bar");
    let ball = model
        .add_ball(BALL, &at(far + OFF, far, far + 5.0))
        .expect("a ball");
    let block = block(&mut model, low, high, far);
    let [first, then] = match cuts {
        Cuts::BlockFirst => [block, ball],
        Cuts::BallFirst => [ball, block],
    };
    let mut intersected = |arguments: &[SolidKey], tool| {
        model
            .fuse(
                arguments,
                &[tool],
                Rule::Intersection,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .map(|fused| fused.solids().to_vec())
    };
    let answer = intersected(&[bar], first).and_then(|cut| intersected(&cut, then));
    (model, answer)
}

/// What a ball of radius `ball` shares with a pipe of radius `bar` whose axis
/// is `off` from its centre, when both loops they cross in are on the pipe.
///
/// Round the line through the ball's centre along the pipe, the pipe's edge is
/// `ρ(θ) = −off·cos θ + √(bar² − off²·sin² θ)` out, and what the ball holds
/// inside that, above and below its centre, is `⅔(ball³ − (ball² − ρ²)^{3/2})`
/// for each `dθ`. The sum round `θ` is of a smooth periodic function, so the
/// trapezium rule closes in on it as fast as its points grow. The plane through
/// the pipe's axis and the ball's centre is a mirror of both, so either side of
/// it holds half.
fn what_a_ball_off_a_bars_axis_shares(ball: f64, bar: f64, off: f64) -> f64 {
    let points = 4096;
    let step = 2.0 * PI / f64::from(points);
    let left_out: f64 = (0..points)
        .map(|at| {
            let angle = f64::from(at) * step;
            let reach = -off * angle.cos() + (bar * bar - (off * angle.sin()).powi(2)).sqrt();
            (ball * ball - reach * reach).powf(1.5)
        })
        .sum::<f64>()
        * step;
    4.0 * PI * ball.powi(3) / 3.0 - 2.0 * left_out / 3.0
}

/// A block from `low` to `high`, both measured from `(far, far, far)`, a
/// rectangle extruded up `z` and placed there.
fn block(model: &mut Model, low: [f64; 3], high: [f64; 3], far: f64) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, high[2] - low[2], &at(far, far, far + low[2]))
        .expect("a block")
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
