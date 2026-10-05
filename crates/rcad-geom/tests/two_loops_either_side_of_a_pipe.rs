//! A thin pipe right through a fat one leaves two loops, one either side of a
//! flat surface through the fat pipe's axis. The half-space that picks them
//! passes through that axis only to within the rounding of its own offset, so
//! the two are told apart only when the room the nearer loop leaves is provably
//! more than that rounding.
//!
//! §314 tightened how the rounding is read: exactly, where it had been read
//! through an interval dot whose far end sits at least twice the rounding away.
//! On a torus that factor of two had been paying for a turn of the picker which
//! the torus's own margin never charged (§314.4). These tests ask the same of
//! the pipe's margin (Q304), at the pairs where it is tightest.
//!
//! # The pairs
//!
//! Every fat pipe here is `2` wide, along `(1, 1, 0)`, through `(0, 0, 2⁵³ + 8)`,
//! and every thin pipe goes through the same point, so the two axes meet. A
//! picking normal with a `z` part then has an offset near `2⁵²` that rounds by a
//! third of a unit, which is a real share of the room between the loops. Each
//! test widens the thin pipe until the pair is refused, one `f64` at a time, and
//! measures the room at the widest pipe still named.
//!
//! # What the room is
//!
//! Write `n̂ = A·â + B·b̂` for the picker's normal, with `â` the part of the thin
//! pipe's direction across the fat axis and `b̂` square to both axes; the normal
//! is exactly square to the fat axis, so those two are all of it. A point of the
//! crossing sits `a` along `â` and `b` along `b̂` from the fat axis, with
//! `a² + b² = r₂²`, and because the axes meet, `|b|` reaches `r₁` and no
//! further. So the nearer loop comes within `A·√(r₂² − r₁²) − |B|·r₁` of the
//! flat surface through the axis, **and gets that near**, and the picker's
//! surface sits `e` off that one. The room is
//!
//! ```text
//! A·√(r₂² − r₁²) − |B|·r₁ − e
//! ```
//!
//! and above zero the nearer loop is wholly on its own side of the picker.
//! Below zero it crosses it, and naming the pair would hand back a loop cut in
//! two. It is worked out here to about a hundred bits from the numbers the
//! picker and the two pipes hold, which is what lets it be read at the
//! threshold, where it is a few roundings wide.

use rcad_geom::{
    Axis, Cylinder, EdgeRun, Error, HalfSpace, Picker, Point, Surface, SurfaceMeet, Vector,
    intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// `2⁵³ + 8`, where `h·z` for `h = fl(1/√2)` misses the nearest `f64` by
/// `0.3431457505…` (the torus of §314 sits at the same number).
const OUT_ALONG_Z: f64 = 9_007_199_254_741_000.0;

/// How wide every fat pipe here is.
const FAT_RADIUS: f64 = 2.0;

/// Below this the room is certainly above zero: it is worked out to about a
/// hundred bits from numbers no larger than four.
const SURELY_ABOVE_ZERO: f64 = 1.0e-27;

/// A thin pipe along `(1, 0, 2)`, whose part across the fat axis does not land
/// exactly square to it in `f64`, so the picker is one of the fallback normals,
/// `(0, 0, h)`, leaning `19.5°` off the direction that separates the loops:
/// `A = 2√2/3` and `B = 1/3`.
///
/// This is where the margin is tightest. Where `B` is near zero the margin's
/// bound on it is `√(1 − A²)` of an `A` a few roundings low, which is about
/// `10⁻⁸`, and that hides anything smaller. With `B` a third, the bound is good
/// to a few roundings, so the arithmetic is the only slack left. The offset
/// `fl(h·z)` misses by `0.343`, which is `0.485` of a length along the normal,
/// against `1.414` as an interval dot read it.
///
/// The widest thin pipe named leaves room above zero, so every loss the margin
/// charges, it charges in full. The next `f64` up is refused with under `10⁻¹³`
/// of room: the margin charges the offset its true rounding, not twice it.
#[test]
fn the_widest_thin_pipe_named_through_a_leaning_picker_leaves_its_near_loop_clear() {
    let thin_way = Vector::new(1.0, 0.0, 2.0);
    let (named, refused) = the_widest_thin_pipe_named(thin_way);
    let picker = the_picker_of(named, thin_way);
    let at_named = the_room(named, picker, thin_way);
    let at_refused = the_room(refused, picker, thin_way);
    eprintln!("leaning: {picker:?}\nnamed: {at_named:#?}\nrefused: {at_refused:#?}");

    assert!(
        (at_named.leaning_into - 8f64.sqrt() / 3.0).abs() < 1e-12,
        "the picker is a fallback, 19.5° off: {at_named:?}"
    );
    assert!(
        at_named.off_as_an_interval_read_it > 2.0 * at_named.off,
        "an interval dot read the offset as more than twice its rounding: {at_named:?}"
    );
    assert!(
        at_named.room.hi > SURELY_ABOVE_ZERO,
        "the near loop of the widest pipe named is clear of the picker: {at_named:?}"
    );
    assert!(
        at_refused.room.hi < 1e-13,
        "the margin gives up within roundings of the loop touching the picker: {at_refused:?}"
    );
}

/// A thin pipe along `(1, −2, 2)`, whose part across the fat axis lands exactly
/// square to it once rounded, so the picker is that rounded direction — turned
/// `5.5·10⁻¹⁷` off the true one, as the torus's `w × n` is turned (§314.4).
///
/// The torus's margin never charged its turn. This one does: `A` and `B` are
/// read off the normal as it is stored, so the turn is in `B`. The widest pipe
/// named leaves room above zero, and a hundred million times what the turn
/// costs.
#[test]
fn the_widest_thin_pipe_named_through_a_turned_picker_leaves_its_near_loop_clear() {
    let thin_way = Vector::new(1.0, -2.0, 2.0);
    let (named, _) = the_widest_thin_pipe_named(thin_way);
    let picker = the_picker_of(named, thin_way);
    let at_named = the_room(named, picker, thin_way);
    eprintln!("turned: {picker:?}\nnamed: {at_named:#?}");

    let turn_costs = at_named.leaning_away * named;
    assert!(
        0.0 < turn_costs && turn_costs < 1e-15,
        "the picker is turned a rounding off the direction that separates the loops: {at_named:?}"
    );
    assert!(
        at_named.room.hi > 1e8 * turn_costs,
        "the near loop of the widest pipe named is clear of the picker by far more than the turn: {at_named:?}"
    );
}

/// Every thin pipe through the fat one's axis point along a direction with whole
/// components from `−4` to `4` and a `z` part from `1` to `4` — 324 directions,
/// among them fallback pickers, turned ones and ones that land square untouched
/// — widened to the widest one named, leaves its near loop clear of the picker.
#[test]
fn every_widest_thin_pipe_named_leaves_its_near_loop_clear_of_the_picker() {
    let mut smallest: Option<Room> = None;
    for x in -4..=4 {
        for y in -4..=4 {
            for z in 1..=4 {
                let thin_way = Vector::new(f64::from(x), f64::from(y), f64::from(z));
                let (named, _) = the_widest_thin_pipe_named(thin_way);
                let room = the_room(named, the_picker_of(named, thin_way), thin_way);
                assert!(
                    room.room.hi > SURELY_ABOVE_ZERO,
                    "the near loop crosses the picker along {thin_way:?}: {room:?}"
                );
                if smallest
                    .as_ref()
                    .is_none_or(|least| room.room.hi < least.room.hi)
                {
                    smallest = Some(room);
                }
            }
        }
    }
    eprintln!("smallest room at the widest pipe named: {smallest:#?}");
}

fn the_widest_thin_pipe_named(thin_way: Vector) -> (f64, f64) {
    let mut named = 1.0;
    let mut refused = FAT_RADIUS;
    assert!(is_named(named, thin_way), "a pipe half as wide is named");
    while named.next_up() < refused {
        let between = f64::from_bits(u64::midpoint(named.to_bits(), refused.to_bits()));
        if is_named(between, thin_way) {
            named = between;
        } else {
            refused = between;
        }
    }
    (named, refused)
}

fn is_named(thin_radius: f64, thin_way: Vector) -> bool {
    match crossed(thin_radius, thin_way) {
        Ok(SurfaceMeet::Curves(runs)) => {
            assert_eq!(runs.len(), 2, "a thin pipe right through leaves two loops");
            true
        }
        Err(Error::CannotNameWhichPieceOfACrossingThisIs) => false,
        other => panic!("two loops, named or refused: {other:?}"),
    }
}

fn the_picker_of(thin_radius: f64, thin_way: Vector) -> HalfSpace {
    let Ok(SurfaceMeet::Curves(runs)) = crossed(thin_radius, thin_way) else {
        panic!("this pair is named");
    };
    let pickers: Vec<HalfSpace> = runs
        .iter()
        .map(|run| {
            let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                panic!("each loop is one piece of the crossing");
            };
            let [Picker::InFrontOf(half_space)] = picked_by.as_slice() else {
                panic!("each loop is picked by one half-space: {picked_by:?}");
            };
            *half_space
        })
        .collect();
    assert_eq!(
        pickers[1],
        pickers[0].reversed(),
        "the two loops are picked by one surface"
    );
    pickers[0]
}

fn crossed(thin_radius: f64, thin_way: Vector) -> Result<SurfaceMeet, Error> {
    let (thin, fat) = the_pair(thin_radius, thin_way);
    intersect(
        &Surface::Cylinder(thin),
        &Surface::Cylinder(fat),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

fn the_pair(thin_radius: f64, thin_way: Vector) -> (Cylinder, Cylinder) {
    let through = Point::new(0.0, 0.0, OUT_ALONG_Z);
    let fat = Cylinder::new(
        Axis::new(through, Vector::new(1.0, 1.0, 0.0)).expect("an axis"),
        FAT_RADIUS,
    )
    .expect("a pipe");
    let thin =
        Cylinder::new(Axis::new(through, thin_way).expect("an axis"), thin_radius).expect("a pipe");
    (thin, fat)
}

/// What the room is made of, for one thin pipe and one picker.
#[derive(Debug)]
struct Room {
    /// `A`, how much of the picker's normal lies along the direction that
    /// separates the loops.
    leaning_into: f64,
    /// `|B|`, how much does not.
    leaning_away: f64,
    /// `e`, how far the picker's surface truly sits off the fat axis.
    off: f64,
    /// How far an interval dot read `e` as: one unit of the offset, over the
    /// normal's length.
    off_as_an_interval_read_it: f64,
    /// `A·√(r₂² − r₁²) − |B|·r₁ − e`, to about a hundred bits.
    room: Wide,
}

/// The room the nearer loop leaves, worked out from the numbers the two pipes
/// and the picker hold.
///
/// With `w` and `t` the two axes' stored directions and `n` the picker's
/// normal, `A = ‖w‖·(n·t) / (‖n‖·‖t × w‖)` and `B = n·(w × t) / (‖n‖·‖t × w‖)`,
/// which is right because `n·w` is exactly zero — checked here, not assumed.
fn the_room(thin_radius: f64, picker: HalfSpace, thin_way: Vector) -> Room {
    let (thin, fat) = the_pair(thin_radius, thin_way);
    let normal = picker.normal().components();
    let fat_way = fat.axis().direction().components();
    let thin_way = thin.axis().direction().components();
    let through = fat.axis().origin().coordinates();

    let square = dot(normal, fat_way);
    assert_eq!(
        (square.hi, square.lo),
        (0.0, 0.0),
        "the picker's normal is exactly square to the fat axis"
    );

    let across = [
        Wide::product(fat_way[1], thin_way[2]).minus(Wide::product(fat_way[2], thin_way[1])),
        Wide::product(fat_way[2], thin_way[0]).minus(Wide::product(fat_way[0], thin_way[2])),
        Wide::product(fat_way[0], thin_way[1]).minus(Wide::product(fat_way[1], thin_way[0])),
    ];
    let sideways = across
        .iter()
        .fold(Wide::of(0.0), |sum, part| sum.plus(part.times(*part)))
        .root();
    let leaning_away = normal
        .iter()
        .zip(across)
        .fold(Wide::of(0.0), |sum, (n, part)| {
            sum.plus(Wide::of(*n).times(part))
        })
        .abs();
    let along = dot(normal, thin_way).abs();
    let normal_length = dot(normal, normal).root();
    let fat_length = dot(fat_way, fat_way).root();
    let off = dot(normal, through).minus(Wide::of(picker.offset())).abs();
    let clear = Wide::product(FAT_RADIUS, FAT_RADIUS)
        .minus(Wide::product(thin_radius, thin_radius))
        .root();

    let below = normal_length.times(sideways);
    let room = fat_length
        .times(along)
        .times(clear)
        .minus(leaning_away.times(Wide::of(thin_radius)))
        .minus(off.times(sideways))
        .over(below);
    let one_unit = picker.offset().abs().next_up() - picker.offset().abs();
    Room {
        leaning_into: fat_length.times(along).over(below).hi,
        leaning_away: leaning_away.over(below).hi,
        off: off.over(normal_length).hi,
        off_as_an_interval_read_it: one_unit / normal_length.hi,
        room,
    }
}

fn dot(one: [f64; 3], other: [f64; 3]) -> Wide {
    one.iter()
        .zip(other)
        .fold(Wide::of(0.0), |sum, (mine, theirs)| {
            sum.plus(Wide::product(*mine, theirs))
        })
}

/// A number held as the sum of two `f64`, good to about a hundred bits.
#[derive(Clone, Copy, Debug)]
struct Wide {
    hi: f64,
    lo: f64,
}

impl Wide {
    fn of(value: f64) -> Self {
        Self { hi: value, lo: 0.0 }
    }

    fn product(a: f64, b: f64) -> Self {
        let hi = a * b;
        Self {
            hi,
            lo: a.mul_add(b, -hi),
        }
    }

    fn plus(self, other: Self) -> Self {
        let (sum, sum_lost) = two_sum(self.hi, other.hi);
        let (low, low_lost) = two_sum(self.lo, other.lo);
        let (sum, lost) = quick_two_sum(sum, sum_lost + low);
        let (hi, lo) = quick_two_sum(sum, lost + low_lost);
        Self { hi, lo }
    }

    fn minus(self, other: Self) -> Self {
        self.plus(Self {
            hi: -other.hi,
            lo: -other.lo,
        })
    }

    fn times(self, other: Self) -> Self {
        let first = Self::product(self.hi, other.hi);
        let (hi, lo) = quick_two_sum(
            first.hi,
            first.lo + (self.hi * other.lo + self.lo * other.hi),
        );
        Self { hi, lo }
    }

    fn over(self, other: Self) -> Self {
        let first = self.hi / other.hi;
        let left = self.minus(other.times(Self::of(first)));
        let second = left.hi / other.hi;
        let left = left.minus(other.times(Self::of(second)));
        let (hi, lo) = quick_two_sum(first, second);
        Self { hi, lo }.plus(Self::of(left.hi / other.hi))
    }

    fn root(self) -> Self {
        let first = self.hi.sqrt();
        let (hi, lo) = quick_two_sum(
            first,
            self.minus(Self::product(first, first)).hi / (2.0 * first),
        );
        Self { hi, lo }
    }

    fn abs(self) -> Self {
        if self.hi < 0.0 {
            Self {
                hi: -self.hi,
                lo: -self.lo,
            }
        } else {
            self
        }
    }
}

/// `a + b` and what rounding it lost, exactly.
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let sum = a + b;
    let b_part = sum - a;
    let a_part = sum - b_part;
    (sum, (a - a_part) + (b - b_part))
}

/// `a + b` and what rounding it lost, exactly, for `|a| ≥ |b|`.
fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
    let sum = a + b;
    (sum, b - (sum - a))
}
