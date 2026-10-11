//! A closed edge whose corner sits a hair off the place its loop is walked
//! from, so the walk leaves a marched point beside the corner (Q1690).
//!
//! A fuse puts a closed loop's corner where the loop is walked from
//! (`rcad_geom::where_a_closed_crossing_starts`). A mesh shifts a face stored
//! far from the origin until its numbers are small and walks the loop again
//! there, so the corner, rounded where it was stored, lands `2.2e-14` off the
//! place the walk starts from in the shifted frame, for a ball and a bar a
//! thousand from the origin. `rcad-geom`'s `begun_beside` kept that start
//! beside the corner, and the mesh the fuse checks its answer with refused it
//! as two corners in one place (`CornersCoincide`). The corner is now judged by
//! how far off the crossing it can be, which reaches the start, and the start
//! is left out.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule};

/// A ball of radius 5 half a unit off the axis of a bar of radius 4 that
/// reaches past both loops they cross in, at the origin and a thousand from it:
/// what the two share measures the same at both, within the error the
/// measurement gives.
#[test]
fn a_ball_off_a_bars_axis_a_thousand_from_the_origin_is_cut_as_it_is_at_the_origin() {
    let [ball, bar, off] = [5.0, 4.0, 0.5];
    let shared = what_a_ball_off_a_bars_axis_shares(ball, bar, off);
    for far in [0.0, 1000.0] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let the_bar = model
            .add_round_bar(bar, 12.0, &at(far, far, far - 1.0))
            .expect("a bar");
        let the_ball = model
            .add_ball(ball, &at(far + off, far, far + 5.0))
            .expect("a ball");
        let answer = model
            .fuse(
                &[the_bar],
                &[the_ball],
                Rule::Intersection,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{far} from the origin: {why:?}"));
        let [solid] = answer.solids()[..] else {
            panic!("one solid {far} from the origin, got {:?}", answer.solids());
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
        assert!(
            (measured.volume() - shared).abs() <= measured.volume_error(),
            "{far} from the origin: {} against {shared}, error {}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// What a ball of radius `ball` shares with a pipe of radius `bar` whose axis
/// is `off` from its centre, when both loops they cross in are on the pipe.
///
/// Round the line through the ball's centre along the pipe, the pipe's edge is
/// `ρ(θ) = −off·cos θ + √(bar² − off²·sin² θ)` out, and what the ball holds
/// inside that, above and below its centre, is `⅔(ball³ − (ball² − ρ²)^{3/2})`
/// for each `dθ`. The sum round `θ` is of a smooth periodic function, so the
/// trapezium rule closes in on it as fast as its points grow.
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

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
