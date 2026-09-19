//! Two refusals that had the answer in hand and dropped it — `docs/decisions.md`
//! §142.8's sites 1 and 3.
//!
//! Both are the same bug written twice. A question is settled exactly, the
//! function that acts on the answer then works the very same quantity out again
//! in intervals, and the enclosure straddles zero because the true value is
//! zero and nothing in an interval can cancel. What the exact arm knew is
//! thrown away one line after it was learned.
//!
//! # Site 1 — the two lines a plane cuts out of a pipe it runs down
//!
//! `where_a_plane_and_a_pipe_cross` decides `n·n·r² − (n·o − d)² > 0` and only
//! then calls `two_lines_along_a_pipe`, which needs the same number back: the
//! two lines stand `√((n·n·r² − (n·o−d)²)/n·n)` either side of the picking
//! plane. It recomputed it as `√(r² − ((n·o−d)/‖n‖)²)`, and for a plane the
//! axis lies *in*, far from the origin, `n·o − d` encloses a window far wider
//! than the pipe is thick. The subtraction goes negative, the root clamps to
//! zero, and neither line is provably on its own side.
//!
//! # Site 3 — the two loops a ball cuts out of a pipe
//!
//! `where_a_ball_and_a_pipe_cross` asks whether the centre sits on the axis by
//! summing `A = ‖(c−o)×w‖²` exactly. When the answer is "it does", it hands the
//! pair to the centred-on-the-axis case, which works out how far apart the two
//! loops are — and starts by computing `A` again in intervals. `2r√(AW)` then
//! comes back nearly twice the whole of what it is taken off.
//!
//! In both the fix is the same: read the quantity off the exact expansion the
//! file already builds, and fall back to the enclosure only where the exact
//! arithmetic declines.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, Plane, Point, Sphere, Surface, SurfaceMeet, Vector, intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// How far out the two pairs are put: far enough that a dot product of the
/// place against a unit normal rounds by a thousand times the shapes' own size.
const FAR_OUT: f64 = 1e10;

/// A plane along a pipe's axis cuts it in two lines wherever the pipe is put.
///
/// The axis lies in the plane exactly — `x + y = 0` and a direction of
/// `(1, −1, 0)` through a point with `x = −y` — so the two lines stand at
/// exactly `±r` either side of the picking plane, and that is true at the
/// origin and `1e10` from it alike. §142.8's site 1 named the pair at the
/// origin and refused the far one.
#[test]
fn a_plane_running_down_a_pipe_names_two_lines_far_from_the_origin_too() {
    for place in [Point::origin(), Point::new(FAR_OUT, -FAR_OUT, 0.0)] {
        let pipe = Cylinder::new(
            Axis::new(place, Vector::new(1.0, -1.0, 0.0)).expect("a direction"),
            1e-8,
        )
        .expect("a pipe");
        let flat = Plane::from_coefficients([1.0, 1.0, 0.0, 0.0]).expect("a plane");
        let pair = [Surface::Plane(flat), Surface::Cylinder(pipe)];
        for first in 0..2 {
            assert_eq!(
                name_the_crossing(&pair, first),
                "curves 2",
                "a plane down the axis of a pipe at {place:?}, asked about surface {first} first"
            );
        }
    }
}

/// A ball centred on a pipe's axis cuts it in two loops wherever the pair is
/// put.
///
/// The centre is exactly on the axis — `(1, 1, 0)` through the origin, and a
/// centre with equal `x` and `y` — and the ball is wider than the pipe, so the
/// two loops stand `√(R² − r²) = 3.16e-5` either side of the centre. §142.8's
/// site 3 named the pair at the origin and refused the far one.
#[test]
fn a_ball_on_a_pipes_axis_names_two_loops_far_from_the_origin_too() {
    for place in [Point::origin(), Point::new(FAR_OUT, FAR_OUT, 0.0)] {
        let pipe = Cylinder::new(
            Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0)).expect("a direction"),
            1e-3,
        )
        .expect("a pipe");
        let ball = Sphere::new(place, 1.0005e-3).expect("a ball");
        let pair = [Surface::Sphere(ball), Surface::Cylinder(pipe)];
        for first in 0..2 {
            assert_eq!(
                name_the_crossing(&pair, first),
                "curves 2",
                "a ball at {place:?} on a pipe's axis, asked about surface {first} first"
            );
        }
    }
}

/// What the kernel calls the crossing, as a short word a failure can print.
fn name_the_crossing(pair: &[Surface; 2], first: usize) -> String {
    match intersect(
        &pair[first],
        &pair[1 - first],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Ok(SurfaceMeet::Curves(runs)) => format!("curves {}", runs.len()),
        Ok(other) => format!("{other:?}"),
        Err(why) => format!("refused: {why:?}"),
    }
}
