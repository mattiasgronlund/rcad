//! A corner where two planes and a pipe meet, with the pipe's axis origin
//! written at different points of its own axis.
//!
//! §51.7's open row is *"one pipe, two names"*. Reversing the axis is one way
//! to write one pipe twice; **sliding the origin along it is the other**, and
//! `name_of_surface_bits` in `rcad-topo` reads the origin straight out, so the
//! two spellings are two names. The question this file answers is what that
//! costs in *arithmetic*: how far apart do the two spellings put the corner?
//!
//! **Not at all, and the reason is structural rather than lucky.**
//! `how_a_pipe_cuts_the_line` builds `‖(V + s·D − m·o) × w‖² − m²r²` out of
//! `Term`s, which are exact expansions and not rounded `f64`s. Sliding the
//! origin to `o + t·w` adds `−m·t·(w × w)` to the offset, and a cross product
//! of a vector with itself is **exactly** zero in exact arithmetic. So the
//! quadratic the corner is solved from is the same expression, and its two
//! roots come back in the same boxes, down to the bit.
//!
//! That is the demonstration §16.10 asks for, so the range below is wide enough
//! that a rounded route could not have survived it: the origin is slid from the
//! pipe's own centre out to `6.7e58`, which is far enough that a rounded
//! `V − m·o` would have nothing but cancellation left.
//!
//! **The slid origin has to land on the axis exactly, and that is the caller's
//! doing and not this file's.** `Axis::new` scales the direction to length one,
//! and no dyadic unit vector has more than one non-zero component — scaling
//! `(a, b, c)` with `a² + b² + c² = 4^k` down forces `(2^k, 0, 0)` — so a
//! slanted axis's stored direction is inexact and `p + t·w` is exact only for
//! well-chosen `t`. Here `p` is the origin and `t` is a power of two, which
//! makes every slid origin an exact multiple of the stored direction and so
//! exactly on the line. A caller who slides by an amount that *rounds* is not
//! writing one pipe twice at all: it is writing two pipes a rounding apart, and
//! `intersect` says so — which is the other half of the measurement and lives
//! in `rcad-topo`'s `one_pipe_whose_origin_sits_at_two_places.rs`.

use rcad_geom::{
    Axis, Cylinder, MeetOfThree, Plane, Point, Surface, Vector, meet_of_three_surfaces,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};

/// The pipe's radius. Nothing rests on the value.
const RADIUS: f64 = 5.0;

/// The direction the slanted pipe runs, before `Axis::new` scales it.
///
/// `(1, 2, 2)` has length three, so the stored direction is a rounding of
/// `(1/3, 2/3, 2/3)` and none of its three components is exact — which is the
/// case worth measuring. An axis-aligned pipe stores its direction exactly and
/// the slide cancels out of the cross product before any arithmetic runs at
/// all.
fn the_way_the_pipe_runs() -> Vector {
    Vector::new(1.0, 2.0, 2.0)
}

/// How far along its own axis each spelling writes the origin, in multiples of
/// the stored direction.
///
/// Every one is a power of two, so `t · w` is exact in each component and the
/// slid origin is exactly on the axis. The last two are `2^100` and `2^196`,
/// which put the origin at `4.2e29` and `3.3e58` — nowhere a rounded
/// subtraction against a corner five units from the axis could come back with
/// the same bits.
const HOW_FAR_ALONG_THE_AXIS: [f64; 7] = [
    0.0,
    two_to_the(2),
    two_to_the(10),
    two_to_the(20),
    two_to_the(40),
    two_to_the(100),
    two_to_the(196),
];

/// `2^power` exactly, built from the bits rather than written out.
///
/// `2^196` as a decimal literal wants more digits than an `f64` keeps, so the
/// reader cannot tell by eye whether what is written is the power of two this
/// file needs or a number a hair off it — and a hair off is a slid origin that
/// misses the axis, which is the one thing these steps must not be.
const fn two_to_the(power: u64) -> f64 {
    f64::from_bits((1023 + power) << 52)
}

/// The pipe, with its axis origin written `steps` of the stored direction along
/// its own axis.
fn a_pipe_naming_its_origin_at(steps: f64) -> Cylinder {
    let direction = Axis::new(Point::origin(), the_way_the_pipe_runs())
        .expect("a direction with a length")
        .direction()
        .components();
    let origin = Point::new(
        steps * direction[0],
        steps * direction[1],
        steps * direction[2],
    );
    Cylinder::new(
        Axis::new(origin, the_way_the_pipe_runs()).expect("a direction with a length"),
        RADIUS,
    )
    .expect("a positive radius")
}

/// Every slid origin is exactly on the axis, so every one of these is the same
/// pipe.
///
/// The whole measurement below rests on it: two pipes a rounding apart are two
/// pipes, and comparing where they cut something would be measuring a different
/// thing.
#[test]
fn every_slid_origin_lands_exactly_on_the_axis_it_was_slid_along() {
    let direction = a_pipe_naming_its_origin_at(0.0).axis().direction();
    for steps in HOW_FAR_ALONG_THE_AXIS {
        let origin = a_pipe_naming_its_origin_at(steps).axis().origin();
        for axis in 0..3 {
            assert_eq!(
                origin.coordinates()[axis],
                steps * direction.components()[axis],
                "the origin slid {steps} steps rounded on axis {axis}"
            );
        }
    }
    assert_ne!(
        a_pipe_naming_its_origin_at(0.0)
            .axis()
            .origin()
            .coordinates(),
        a_pipe_naming_its_origin_at(4.0)
            .axis()
            .origin()
            .coordinates(),
        "two spellings that stored the same numbers would not be two spellings"
    );
}

/// The corners two planes and the pipe meet at, boxes and all.
fn where_the_three_meet(pipe: Cylinder, planes: [Plane; 2]) -> Vec<PredicatePoint> {
    let found = meet_of_three_surfaces(
        [
            &Surface::Plane(planes[0]),
            &Surface::Plane(planes[1]),
            &Surface::Cylinder(pipe),
        ],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("two planes and a pipe is the triple §31.2 row 8 solves");
    let MeetOfThree::At(points) = found else {
        panic!("the two planes were meant to cut the pipe at points, and got {found:?}");
    };
    points.to_vec()
}

fn a_plane_through(through: [f64; 3], normal: [f64; 3]) -> Plane {
    Plane::from_point_and_normal(
        Point::new(through[0], through[1], through[2]),
        Vector::new(normal[0], normal[1], normal[2]),
    )
    .expect("a normal with a length")
}

/// **The measurement.** Not one ulp, over a slide of `6.7e58`.
///
/// Two plane pairs: one through the origin and axis-aligned, where every
/// coordinate of the answer is a round number, and one nowhere near either,
/// where all three coordinates of both corners are irrational and the boxes are
/// several ulps wide. Both give the same bits for every spelling.
#[test]
fn sliding_a_pipes_origin_along_its_own_axis_does_not_move_the_corner_at_all() {
    let pairs = [
        [
            a_plane_through([0.0; 3], [0.0, 1.0, 0.0]),
            a_plane_through([0.0; 3], [0.0, 0.0, 1.0]),
        ],
        [
            a_plane_through([1.0, 0.0, 0.0], [1.0, 2.0, 3.0]),
            a_plane_through([0.0, 0.0, 7.0], [4.0, -1.0, 1.0]),
        ],
    ];
    for planes in pairs {
        let unslid = where_the_three_meet(a_pipe_naming_its_origin_at(0.0), planes);
        assert_eq!(unslid.len(), 2, "a line cuts a pipe twice");
        for steps in HOW_FAR_ALONG_THE_AXIS {
            let slid = where_the_three_meet(a_pipe_naming_its_origin_at(steps), planes);
            let boxes = |points: &[PredicatePoint]| {
                points
                    .iter()
                    .map(|point| point.enclosure().map(|axis| (axis.lo(), axis.hi())))
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                boxes(&slid),
                boxes(&unslid),
                "writing the pipe's origin {steps} steps along its own axis moved the corner"
            );
        }
    }
}
