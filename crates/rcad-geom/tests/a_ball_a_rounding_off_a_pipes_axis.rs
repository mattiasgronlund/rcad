//! A ball wider than a pipe, centred a rounding off the pipe's axis, as a round
//! bar and a ball on its axis leave them once both are turned.
//!
//! A pipe of radius 5 down `z` and a ball centred on `z` cross in two circles,
//! `√(R² − 5²)` either side of the centre along the axis. Turned by `1.1` about
//! the line through `(1, 0, −1)` along `(1, 2, 3)`, the stored centre sits a
//! rounding off the stored axis, so to the walker the pair is no longer two
//! circles but a degree-four crossing a rounding off them, picked up at the two
//! places the ball reaches furthest along the axis. Those two places are taken
//! along a way across the pipe, and the one worked out as `o − w(o·w)`, with
//! `o` running from the axis's origin to the centre, is rounding: `4.4e−15`
//! long and pointing `0.99997` of the way back down the axis. Both seeds then
//! sat inside the pipe near neither loop, and each loop was refused
//! `CannotNameWhichPieceOfACrossingThisIs` (Q2129, Q2330).
//!
//! These are the four balls of `rcad-topo`'s
//! `a_bar_and_a_ball_on_its_cap_turned.rs`, each turned in one step and in two
//! halves, where the bar is turned in one.

use rcad_geom::{
    Axis, Cylinder, Deflection, Isometry, NamedRun, Point, Sphere, Surface, SurfaceMeet, Vector,
    intersect, walk, where_a_closed_crossing_starts,
};
use rcad_predicates::{Policy, ProvenanceLog};

const PIPE_RADIUS: f64 = 5.0;

/// Each ball as `(radius, how far up z its centre is)`.
const BALLS: [(f64, f64); 4] = [(6.0, 10.0), (6.0, 12.0), (5.5, 10.0), (6.0, 15.0)];

/// How far off both surfaces a start may sit: a few roundings of numbers
/// about twenty long, and a million times less than the smallest of the loops.
const ON_BOTH: f64 = 1e-12;

/// Each loop starts on both surfaces, `√(R² − 5²)` from the centre along the
/// axis, on the side the same loop of the pair unturned starts on, and walks
/// round that side and no other.
#[test]
fn each_loop_of_a_ball_a_rounding_off_a_pipes_axis_starts_where_the_unturned_one_does() {
    for (radius, height) in BALLS {
        let unturned = Pair::of(radius, height, &[], &[]);
        let unturned_sides = unturned.loops().map(|run| {
            let up = unturned.how_far_up(unturned.start(&run, false));
            assert!(
                up != 0.0,
                "the unturned {radius} at {height} starts on its centre"
            );
            up.signum()
        });
        let turn = turning(1.1);
        let half = turning(0.55);
        for ball_turns in [vec![turn], vec![half, half]] {
            let turned = Pair::of(radius, height, &[turn], &ball_turns);
            let what = format!(
                "a ball of {radius} at {height} turned in {} steps",
                ball_turns.len()
            );
            let reach = (radius * radius - PIPE_RADIUS * PIPE_RADIUS).sqrt();
            for (run, side) in turned.loops().into_iter().zip(unturned_sides) {
                for pipe_first in [false, true] {
                    let start = turned.start(&run, pipe_first);
                    let [off_the_ball, off_the_pipe] = turned.how_far_off(start);
                    let up = turned.how_far_up(start);
                    assert!(
                        off_the_ball <= ON_BOTH && off_the_pipe <= ON_BOTH,
                        "{what}: {start:?} is {off_the_ball} off the ball and {off_the_pipe} off the pipe"
                    );
                    assert!(
                        (up - side * reach).abs() <= ON_BOTH,
                        "{what}: the loop the unturned pair starts {side} up starts {up} up"
                    );
                }
                let walked = walk(
                    &run,
                    [&turned.ball, &turned.pipe],
                    Deflection::of(0.01).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what}: the loop {side} up is walked: {why:?}"));
                let strays = walked.deviation().magnitude() + ON_BOTH;
                for point in walked.points() {
                    let [off_the_ball, off_the_pipe] = turned.how_far_off(*point);
                    assert!(
                        off_the_ball <= strays && off_the_pipe <= strays,
                        "{what}: {point:?} walked is {off_the_ball} off the ball and {off_the_pipe} off the pipe"
                    );
                    assert!(
                        (turned.how_far_up(*point) - side * reach).abs() <= strays,
                        "{what}: {point:?} walked is off the loop {side} up"
                    );
                }
            }
        }
    }
}

/// The turn `rcad-topo`'s file places both solids by, or a part of it.
fn turning(angle: f64) -> Isometry {
    let axis =
        Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    Isometry::rotation(&axis, angle).expect("a finite angle")
}

/// A ball and the pipe of radius 5 it is centred on the axis of, before either
/// is turned.
struct Pair {
    ball: Surface,
    pipe: Surface,
    centre: Point,
    radius: f64,
    axis: Axis,
}

impl Pair {
    /// The pipe down `z` turned by each of `pipe_turns` in turn, and the ball
    /// of `radius` at `height` up `z` by each of `ball_turns`.
    fn of(radius: f64, height: f64, pipe_turns: &[Isometry], ball_turns: &[Isometry]) -> Self {
        let pipe = pipe_turns.iter().fold(
            Cylinder::new(Axis::z(), PIPE_RADIUS).expect("a radius"),
            |pipe, turn| pipe.transformed(turn),
        );
        let ball = ball_turns.iter().fold(
            Sphere::new(Point::new(0.0, 0.0, height), radius).expect("a radius"),
            |ball, turn| ball.transformed(turn),
        );
        Self {
            ball: Surface::Sphere(ball),
            pipe: Surface::Cylinder(pipe),
            centre: ball.centre(),
            radius,
            axis: pipe.axis(),
        }
    }

    /// The two loops `intersect` names.
    fn loops(&self) -> [NamedRun; 2] {
        let named = intersect(
            &self.ball,
            &self.pipe,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a ball and a pipe are named");
        let SurfaceMeet::Curves(runs) = named else {
            panic!("a ball wider than a pipe on its axis crosses it: {named:?}");
        };
        runs.try_into()
            .unwrap_or_else(|runs| panic!("two loops and not {runs:?}"))
    }

    /// Where `run` starts, asked with the pipe or the ball first.
    fn start(&self, run: &NamedRun, pipe_first: bool) -> Point {
        let along = if pipe_first {
            [&self.pipe, &self.ball]
        } else {
            [&self.ball, &self.pipe]
        };
        where_a_closed_crossing_starts(run, along, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{run:?} starts somewhere: {why:?}"))
    }

    /// How far `point` is along the pipe's axis from the ball's centre.
    fn how_far_up(&self, point: Point) -> f64 {
        (point - self.centre).dot(self.along())
    }

    /// How far `point` is off the ball and off the pipe.
    fn how_far_off(&self, point: Point) -> [f64; 2] {
        let out = point - self.axis.origin();
        let across = out - self.along() * out.dot(self.along());
        [
            ((point - self.centre).length() - self.radius).abs(),
            (across.length() - PIPE_RADIUS).abs(),
        ]
    }

    fn along(&self) -> Vector {
        self.axis
            .direction()
            .normalized()
            .expect("an axis has a direction")
    }
}
