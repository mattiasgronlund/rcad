//! A plane through a torus's axis cuts two circles, and the flat half-space that
//! tells them apart passes through the torus's centre only to within the
//! rounding of its own offset. Each circle is `R − r` clear of the centre, so
//! the two are told apart only when that rounding is provably smaller than
//! `R − r`.
//!
//! Every torus here lies on the plane `z = 0`, centred `x` out along `+x`, with
//! its axis along `(1, 1, 0)`. The stored axis is `(h, h, 0)` and the picking
//! normal `w × n` is `(h, −h, 0)` with nothing rounded, so the one way the
//! picker misses the centre is its offset: `fl(h·x)` where the true dot is
//! `h·x`. With `x` near `2⁵³` that misses by up to half a unit, which is as
//! wide as the room between a tube and its ring when both are under one.
//!
//! `docs/decisions.md` §314 says why a torus this far out is the only kind on
//! which the rounding of `R − r` could have mattered, and why it never did.

use rcad_geom::{
    Axis, EdgeRun, Error, HalfSpace, Picker, Plane, Point, Surface, SurfaceMeet, Torus, Vector,
    intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// `2⁵³ + 8`, where the picker for an axis along `(1, 1, 0)` misses the centre
/// by `0.3431457505…`, in the upper half of the gap between the two `f64`
/// either side of it.
const OUT_ALONG_X: f64 = 9_007_199_254_741_000.0;

/// A picker `0.343` off the centre of a torus whose tube leaves `0.6` of room
/// either side of it: more than half the room, and less than all of it, so the
/// two circles are apart and each lies wholly on its own side.
///
/// Read through an interval dot, the offset's rounding came back as a whole
/// unit — the interval ends at the two `f64` either side of `h·x`, and the
/// offset is the nearer, so the far end is at least twice the true rounding
/// away — and a whole unit is more than `0.6`, so this pair was refused. Read
/// exactly, it is `0.343` and the pair is named.
#[test]
fn a_picker_off_the_centre_by_more_than_half_the_room_still_names_both_circles() {
    let ring = a_torus_out_along_x(1.0, 0.4);
    let off = how_far_the_picker_misses_the_centre(&ring);
    assert!(
        0.3 < off.hi && off.hi < 0.6,
        "the picker is more than half of 0.6 off the centre and less than all of it: {off:?}"
    );

    let flat = the_plane_through_its_axis(&ring);
    let SurfaceMeet::Curves(runs) = crossed(&flat, &ring).expect("two circles, told apart") else {
        panic!("a plane through the axis cuts two circles");
    };
    let [h, _, _] = ring.axis().direction().components();
    let between = HalfSpace::new(Vector::new(h, -h, 0.0), h * OUT_ALONG_X).expect("a half-space");
    let pickers: Vec<Picker> = runs
        .iter()
        .map(|run| {
            let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                panic!("each circle is one piece of the crossing");
            };
            assert_eq!(and, &Surface::Torus(ring));
            assert_eq!(picked_by.len(), 1);
            picked_by[0]
        })
        .collect();
    assert_eq!(
        pickers,
        vec![
            Picker::InFrontOf(between),
            Picker::InFrontOf(between.reversed())
        ],
        "the picker is w × n as stored, through the centre to within one rounding of h·x"
    );
    assert_eq!(crossed_the_other_way(&flat, &ring), crossed(&flat, &ring));
}

/// A picker off the centre by **all** of the room, on a torus whose `R − r`
/// rounds up past how far off the picker is: `fl(R − r)` is more than the
/// distance and the true `R − r` is not.
///
/// This is the torus whose offset's rounding uses up `R − r` and not
/// `fl(R − r)`, and it is refused, with `R − r` rounded as it used to be and
/// enclosed as it is now alike. What was compared against `fl(R − r)` was one
/// `f64` bound on the distance, and rounding to nearest never carries a number
/// past an `f64`: no `f64` lies between the true `R − r` and its rounding, so
/// `fl(R − r)` above that bound means `R − r` above it too.
#[test]
fn a_picker_off_the_centre_by_all_of_the_room_is_refused_although_the_room_rounds_up_past_it() {
    let off = how_far_the_picker_misses_the_centre(&a_torus_out_along_x(1.0, 0.4));
    let rounded_room = off.hi;
    assert!(
        off.lo < 0.0,
        "the distance is just under the f64 it rounds to: {off:?}"
    );
    let ring_radius = rounded_room + 2f64.powi(-10);
    let tube_radius = 2f64.powi(-10) + 109.0 * 2f64.powi(-62);
    let ring = a_torus_out_along_x(ring_radius, tube_radius);

    let (room, room_rounding) = two_sum(ring_radius, -tube_radius);
    assert_eq!(room, rounded_room, "R − r rounds to the f64 just past the distance");
    assert!(room_rounding < 0.0, "and the true R − r is under that f64");
    assert!(
        off.lo - room_rounding > 2f64.powi(-90),
        "the picker is further off the centre than the true R − r: {off:?} against {room_rounding:e}"
    );

    let flat = the_plane_through_its_axis(&ring);
    assert_eq!(
        crossed(&flat, &ring),
        Err(Error::CannotNameWhichPieceOfACrossingThisIs)
    );
    assert_eq!(
        crossed_the_other_way(&flat, &ring),
        Err(Error::CannotNameWhichPieceOfACrossingThisIs)
    );
}

/// A plane through the axis leaning off every coordinate plane, where `w × n`
/// rounds and the picker is turned a little off the true surface between the
/// two circles, still names both of them.
///
/// The turn costs `R·sin²θ` of the room, which is about `10⁻³²` of the ring
/// here. What this guards is the bound on it being worked out against the true
/// `w × n` and not against something a whole picker away from it, which would
/// refuse every plane that is not a coordinate plane.
#[test]
fn a_leaning_plane_through_the_axis_still_names_both_circles() {
    let axis = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0)).expect("an axis");
    let ring = Torus::new(axis, 4.0, 1.0).expect("a tube narrower than its ring");
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, -1.0, 1.0))
        .expect("a normal");

    let [h, _, _] = axis.direction().components();
    let [_, _, lean] = flat.normal().components();
    assert_ne!(
        h.mul_add(lean, -(h * lean)),
        0.0,
        "w × n rounds, so the picker is turned off the true surface"
    );

    let SurfaceMeet::Curves(runs) = crossed(&flat, &ring).expect("two circles, told apart") else {
        panic!("a plane through the axis cuts two circles");
    };
    assert_eq!(runs.len(), 2);
    assert_eq!(crossed_the_other_way(&flat, &ring), crossed(&flat, &ring));
}

fn a_torus_out_along_x(ring_radius: f64, tube_radius: f64) -> Torus {
    let axis = Axis::new(
        Point::new(OUT_ALONG_X, 0.0, 0.0),
        Vector::new(1.0, 1.0, 0.0),
    )
    .expect("an axis");
    Torus::new(axis, ring_radius, tube_radius).expect("a tube narrower than its ring")
}

fn the_plane_through_its_axis(ring: &Torus) -> Plane {
    Plane::from_point_and_normal(ring.centre(), Vector::new(0.0, 0.0, 1.0)).expect("a normal")
}

fn crossed(flat: &Plane, ring: &Torus) -> Result<SurfaceMeet, Error> {
    intersect(
        &Surface::Plane(*flat),
        &Surface::Torus(*ring),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

fn crossed_the_other_way(flat: &Plane, ring: &Torus) -> Result<SurfaceMeet, Error> {
    intersect(
        &Surface::Torus(*ring),
        &Surface::Plane(*flat),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

/// A number held as the sum of two `f64`, good to about a hundred bits.
#[derive(Clone, Copy, Debug)]
struct Wide {
    hi: f64,
    lo: f64,
}

/// How far the picker's flat surface sits off the torus's centre, as a length.
///
/// The reading at the centre is `h·x − fl(h·x)`, which one fused multiply-add
/// gives exactly, and the picking normal is `(h, −h, 0)`, so the length is that
/// reading over `h·√2`.
fn how_far_the_picker_misses_the_centre(ring: &Torus) -> Wide {
    let [h, also_h, zero] = ring.axis().direction().components();
    assert_eq!((also_h, zero), (h, 0.0), "the axis is stored along (1, 1, 0)");
    let [x, _, _] = ring.centre().coordinates();
    let reading = h.mul_add(x, -(h * x)).abs();

    let squared = product(h, h);
    let length = root(Wide {
        hi: 2.0 * squared.hi,
        lo: 2.0 * squared.lo,
    });
    let first = reading / length.hi;
    let step = product(first, length.hi);
    let left = ((reading - step.hi) - step.lo) - first * length.lo;
    Wide {
        hi: first,
        lo: left / length.hi,
    }
}

fn product(a: f64, b: f64) -> Wide {
    let hi = a * b;
    Wide {
        hi,
        lo: a.mul_add(b, -hi),
    }
}

fn root(of: Wide) -> Wide {
    let first = of.hi.sqrt();
    let square = product(first, first);
    Wide {
        hi: first,
        lo: ((of.hi - square.hi) - square.lo + of.lo) / (2.0 * first),
    }
}

/// `a + b` and what rounding it lost, exactly.
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let sum = a + b;
    let b_part = sum - a;
    let a_part = sum - b_part;
    (sum, (a - a_part) + (b - b_part))
}
