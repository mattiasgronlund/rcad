//! A plane through a torus's axis cuts two circles, and the flat half-space that
//! tells them apart passes through the torus's centre only to within the
//! rounding of its own offset. Each circle is `R − r` clear of the centre, so
//! the two are told apart only when that rounding is provably smaller than
//! `R − r`.
//!
//! Most tori here lie on the plane `z = 0`, centred `x` out along `+x`, with
//! their axis along `(1, 1, 0)`. The stored axis is `(h, h, 0)` and the picking
//! normal `w × n` is `(h, −h, 0)` with nothing rounded, so the one way the
//! picker misses the centre is its offset: `fl(h·x)` where the true dot is
//! `h·x`. With `x` near `2⁵³` that misses by up to half a unit, which is as
//! wide as the room between a tube and its ring when both are under one.
//!
//! `docs/decisions.md` §314 says why a torus this far out is the only kind on
//! which the rounding of `R − r` could matter. §314.8 and §314.9 are the two
//! tori on which the margin's rounding down costs a whole `f64` of room.

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
    let ring = a_torus_out_along_x(Vector::new(1.0, 1.0, 0.0), 1.0, 0.4);
    let flat = the_plane_through_its_axis(&ring);
    let off = how_far_the_picker_misses(&ring, &flat);
    assert!(
        0.3 < off.hi && off.hi < 0.6,
        "the picker is more than half of 0.6 off the centre and less than all of it: {off:?}"
    );

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
    let along = Vector::new(1.0, 1.0, 0.0);
    let probe = a_torus_out_along_x(along, 1.0, 0.4);
    let flat = the_plane_through_its_axis(&probe);
    let off = how_far_the_picker_misses(&probe, &flat);
    let rounded_room = off.hi;
    assert!(
        off.lo < 0.0,
        "the distance is just under the f64 it rounds to: {off:?}"
    );
    let ring_radius = rounded_room + 2f64.powi(-10);
    let tube_radius = 2f64.powi(-10) + 109.0 * 2f64.powi(-62);
    let ring = a_torus_out_along_x(along, ring_radius, tube_radius);

    let (room, room_rounding) = two_sum(ring_radius, -tube_radius);
    assert_eq!(
        room, rounded_room,
        "R − r rounds to the f64 just past the distance"
    );
    assert!(room_rounding < 0.0, "and the true R − r is under that f64");
    assert!(
        off.lo - room_rounding > 2f64.powi(-90),
        "the picker is further off the centre than the true R − r: {off:?} against {room_rounding:e}"
    );

    assert_refused(&flat, &ring);
}

/// A torus whose room `R − r` is a quarter of a step under the `f64` one past
/// the picker's distance is refused, although both circles clear the picker.
/// It is refused with `R − r` enclosed, and named with `R − r` rounded.
///
/// On this torus nothing in `w × n` rounds, so the turn is exactly zero, and
/// the margin charges the picker `up`, the `f64` at or above its true
/// distance: a room of exactly the `f64` past `up` is named. The room here is
/// a quarter step under that `f64` and rounds to it, but
/// `Interval::exactly(R) − Interval::exactly(r)` has the `f64` below as its low
/// end, which is `up` itself, and `up − up` is not above zero. The true room
/// clears the true distance by three quarters of a step and nothing is turned,
/// so the answer exists and this refusal is a gap (`docs/decisions.md` §314.8).
#[test]
fn rounding_the_ring_less_the_tube_down_takes_a_whole_f64_off_the_room() {
    let along = Vector::new(1.0, 1.0, 0.0);
    let probe = a_torus_out_along_x(along, 1.0, 0.4);
    let flat = the_plane_through_its_axis(&probe);
    let off = how_far_the_picker_misses(&probe, &flat);
    assert!(
        off.lo < 0.0,
        "the distance is just under the f64 it rounds to: {off:?}"
    );
    let up = off.hi;
    let room = up.next_up();
    assert_both_circles_named(&flat, &a_torus_with_room(along, room));

    let ring_radius = room + 2f64.powi(-10);
    let tube_radius = 2f64.powi(-10) + 2f64.powi(-56);
    let (rounded_room, room_rounding) = two_sum(ring_radius, -tube_radius);
    assert_eq!(
        (rounded_room, room_rounding),
        (room, -2f64.powi(-56)),
        "R − r is a quarter of a step under the f64 past up, and rounds to it"
    );
    assert_eq!(room - up, 2f64.powi(-54), "one step at 0.34 is 2⁻⁵⁴");
    assert!(
        (room - up) + room_rounding - off.lo > 2f64.powi(-55),
        "the true room clears the true distance by three quarters of a step: {off:?}"
    );

    assert_refused(&flat, &a_torus_out_along_x(along, ring_radius, tube_radius));
}

/// A torus whose room is one `f64` past the picker's distance is refused,
/// because the margin takes the turn of a rounded `w × n` off the room rounding
/// down, and the turn's bound costs about `2·10⁻³⁵` but not zero. It is named
/// with the turn taken out of the margin.
///
/// The axis leans along `(1, 5, 0)` and the plane's normal is `(0, 0, z)` with
/// `z` three `f64` under one, kept as written, so `w × n` rounds. The margin
/// charges the picker `up`, the `f64` at or above its true distance: a room
/// two `f64` past `up` is named. A room one `f64` past `up` is refused, because
/// that room less the turn, rounded down, is `up`, and `up − up` is not above
/// zero. That room clears the true distance by more than a step and the turn
/// costs under `2⁻¹⁰⁰` of it, so the answer exists and this refusal is a gap
/// (`docs/decisions.md` §314.9).
#[test]
fn subtracting_a_turn_worth_ten_to_the_minus_35_takes_a_whole_f64_off_the_room() {
    let along = Vector::new(1.0, 5.0, 0.0);
    let lean = 1.0 - 3.0 * 2f64.powi(-53);
    let probe = a_torus_out_along_x(along, 1.0, 0.4);
    let flat = Plane::from_point_and_normal_as_written(probe.centre(), Vector::new(0.0, 0.0, lean))
        .expect("a normal");
    assert_eq!(
        flat.normal().components(),
        [0.0, 0.0, lean],
        "the normal is kept as written"
    );

    let off = how_far_the_picker_misses(&probe, &flat);
    assert!(
        off.lo > 0.0,
        "the distance is just over the f64 nearest it: {off:?}"
    );
    let up = off.hi.next_up();
    assert_eq!(up, 0.155_354_594_472_641_85);
    let room = up.next_up();
    assert!(
        (room - off.hi) - off.lo > 2f64.powi(-55),
        "a room one f64 past up clears the true distance by more than a step: {off:?}"
    );

    let [w1, w2, _] = probe.axis().direction().components();
    let [q1, q2, _] = probe.axis().direction().cross(flat.normal()).components();
    assert_eq!(
        (q1, q2),
        (w2 * lean, -(w1 * lean)),
        "w × n is two products, each rounded once"
    );
    let rounding = [w2.mul_add(lean, -q1), w1.mul_add(lean, q2)];
    let turned =
        rounding[0].mul_add(rounding[0], rounding[1] * rounding[1]) / q1.mul_add(q1, q2 * q2);
    let ring_radius = room + 2f64.powi(-10);
    assert!(
        (1e-35..1e-34).contains(&(ring_radius * turned)),
        "the turn's bound costs about 2·10⁻³⁵, and not zero: {turned:e}"
    );

    assert_refused(&flat, &a_torus_with_room(along, room));
    assert_both_circles_named(&flat, &a_torus_with_room(along, room.next_up()));
}

/// A plane through the axis leaning off every coordinate plane, where `w × n`
/// rounds, still names both circles.
///
/// The axis is stored as `(h, h, 0)` and the normal as `(a, −a, a)`, so
/// `w × n` comes out `(p, −p, −2p)` with `p = fl(h·a)`. That is parallel to the
/// true product `h·a·(1, −1, −2)`, so the picker is not turned at all, but the
/// bound on the turn reads only the rounding: `((h·a − p)/p)²`, about
/// `7·10⁻³⁵`, which costs `R = 4` times that of the room. What this guards is
/// the bound being worked
/// out against the true `w × n` and not against something a whole picker away
/// from it, which would refuse every plane that is not a coordinate plane.
#[test]
fn a_leaning_plane_through_the_axis_still_names_both_circles() {
    let axis = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0)).expect("an axis");
    let ring = Torus::new(axis, 4.0, 1.0).expect("a tube narrower than its ring");
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, -1.0, 1.0))
        .expect("a normal");

    let [h, _, _] = axis.direction().components();
    let [a, _, _] = flat.normal().components();
    assert_eq!(flat.normal().components(), [a, -a, a]);
    let p = h * a;
    assert_eq!(
        axis.direction().cross(flat.normal()).components(),
        [p, -p, -2.0 * p],
        "w × n as stored is parallel to the true product"
    );
    let turned = (h.mul_add(a, -p) / p).powi(2);
    assert!(
        (1e-34..1e-33).contains(&(4.0 * turned)),
        "w × n rounds, and the bound on a turn it does not have costs about 3·10⁻³⁴: {turned:e}"
    );

    assert_both_circles_named(&flat, &ring);
}

/// A torus out along `x` with its axis along `along`, whose `R − r` is `room`
/// with nothing rounded.
fn a_torus_with_room(along: Vector, room: f64) -> Torus {
    let ring = a_torus_out_along_x(along, room + 2f64.powi(-10), 2f64.powi(-10));
    assert_eq!(ring.ring_radius() - ring.tube_radius(), room);
    ring
}

fn a_torus_out_along_x(along: Vector, ring_radius: f64, tube_radius: f64) -> Torus {
    let axis = Axis::new(Point::new(OUT_ALONG_X, 0.0, 0.0), along).expect("an axis");
    Torus::new(axis, ring_radius, tube_radius).expect("a tube narrower than its ring")
}

fn the_plane_through_its_axis(ring: &Torus) -> Plane {
    Plane::from_point_and_normal(ring.centre(), Vector::new(0.0, 0.0, 1.0)).expect("a normal")
}

fn assert_both_circles_named(flat: &Plane, ring: &Torus) {
    let Ok(SurfaceMeet::Curves(runs)) = crossed(flat, ring) else {
        panic!(
            "a plane through the axis cuts two circles, told apart: {:?}",
            crossed(flat, ring)
        );
    };
    assert_eq!(runs.len(), 2);
    assert_eq!(crossed_the_other_way(flat, ring), crossed(flat, ring));
}

fn assert_refused(flat: &Plane, ring: &Torus) {
    assert_eq!(
        crossed(flat, ring),
        Err(Error::CannotNameWhichPieceOfACrossingThisIs)
    );
    assert_eq!(
        crossed_the_other_way(flat, ring),
        Err(Error::CannotNameWhichPieceOfACrossingThisIs)
    );
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

/// How far the picker's flat surface sits off the torus's centre, as a length,
/// for a plane whose `w × n` is `(q₁, q₂, 0)` and a centre `(x, 0, 0)`.
///
/// The reading at the centre is `q₁·x − fl(q₁·x)`, which one fused
/// multiply-add gives exactly, and the length is that reading over
/// `√(q₁² + q₂²)`.
fn how_far_the_picker_misses(ring: &Torus, flat: &Plane) -> Wide {
    let [q1, q2, zero] = ring.axis().direction().cross(flat.normal()).components();
    assert_eq!(zero, 0.0, "the plane's normal is square to the xy plane");
    let [x, y, z] = ring.centre().coordinates();
    assert_eq!((y, z), (0.0, 0.0), "the centre is out along x");
    let reading = q1.mul_add(x, -(q1 * x)).abs();

    let [first_square, second_square] = [product(q1, q1), product(q2, q2)];
    let (hi, lo) = two_sum(first_square.hi, second_square.hi);
    let length = root(Wide {
        hi,
        lo: lo + first_square.lo + second_square.lo,
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
