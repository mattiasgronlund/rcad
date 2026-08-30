//! A search for surface predicates that certify a sign they have not earned.
//!
//! The companion to `rcad-predicates`' search of the same name. Every level 1
//! error bound is a *relative* one, and relative bounds stop meaning anything
//! once a product falls into the subnormals or runs past the top of the range.
//!
//! # How this catches one without an exact oracle
//!
//! Each of the three predicates here decides on a polynomial that is
//! homogeneous in everything it is handed, once the surface is placed at the
//! origin:
//!
//! - a plane through the origin decides on `normal · point`, which doubles when
//!   the point doubles;
//! - a ball at the origin decides on `|point|² - radius²`, which quadruples
//!   when the point and the radius both double;
//! - a pipe about an axis through the origin decides on
//!   `|point × direction|² - radius²`, likewise.
//!
//! So scaling by a power of two multiplies the value by a *positive* number and
//! cannot change its sign. Each sample is drawn at one end of the `f64` range
//! and then carried back to the middle of it, where every product is an
//! ordinary normal number, and the two runs must agree. Carrying a number
//! between scales by a power of two is exact as long as neither end overflows,
//! and both ends are checked, so the two runs really are the same geometry and
//! not merely a near copy of it.
//!
//! # What makes a sample worth drawing
//!
//! A point nowhere near the surface is settled by the first bit of the first
//! level and tests nothing. So the generators build configurations that sit as
//! close to the surface as whole numbers allow — a point exactly at right
//! angles to the normal a plane was asked for, or a radius whose square is the
//! nearest whole number to the point's squared distance — and then push them to
//! the end of the range, where the products lose their accuracy.
//!
//! # Why the counts do not say which level answered
//!
//! `rcad-geom`'s surface predicates hand back a decision, not the tier that
//! made it. They do not need to here: levels 2 and 3 cannot certify a wrong
//! sign, because an interval is a guaranteed enclosure and exact arithmetic is
//! exact. So every disagreement below is a level 1 disagreement, and the
//! `certified` count is an over-count of the samples level 1 answered.
//!
//! Set `RCAD_SEARCH_SAMPLES` to run the same search harder:
//! `RCAD_SEARCH_SAMPLES=10000000 cargo test --release -- --nocapture`.

use rcad_geom::{Axis, Cylinder, Plane, Point, Sphere, Vector};
use rcad_predicates::{Decision, Point as PredicatePoint, SideOfSurface};

/// How many samples each search draws unless `RCAD_SEARCH_SAMPLES` says
/// otherwise.
const SAMPLES: usize = 100_000;

/// Where a sample's biggest coordinate is put, as a power of two.
///
/// A plane multiplies a coordinate by a normal of about length one, so its
/// products only lose accuracy once the coordinate itself is near the bottom of
/// the range. A ball or a pipe squares its coordinates, so for those the damage
/// starts halfway down.
const A_PLANE_LOSES_ITS_PRODUCTS: core::ops::RangeInclusive<i32> = -1074..=-1020;
const A_PLANE_OVERFLOWS_ITS_PRODUCTS: core::ops::RangeInclusive<i32> = 1000..=1023;
const A_SQUARE_LOSES_ITS_PRODUCTS: core::ops::RangeInclusive<i32> = -545..=-515;
const A_SQUARE_OVERFLOWS_ITS_PRODUCTS: core::ops::RangeInclusive<i32> = 500..=530;

#[test]
fn a_point_at_the_bottom_of_the_range_is_on_the_same_side_of_a_plane() {
    report(
        "plane, subnormal",
        search_plane(1, A_PLANE_LOSES_ITS_PRODUCTS),
    );
}

#[test]
fn a_point_at_the_top_of_the_range_is_on_the_same_side_of_a_plane() {
    report(
        "plane, overflow",
        search_plane(2, A_PLANE_OVERFLOWS_ITS_PRODUCTS),
    );
}

#[test]
fn a_ball_at_the_bottom_of_the_range_holds_the_same_points() {
    report(
        "sphere, subnormal",
        search_sphere(3, A_SQUARE_LOSES_ITS_PRODUCTS),
    );
}

#[test]
fn a_ball_at_the_top_of_the_range_holds_the_same_points() {
    report(
        "sphere, overflow",
        search_sphere(4, A_SQUARE_OVERFLOWS_ITS_PRODUCTS),
    );
}

#[test]
fn a_pipe_at_the_bottom_of_the_range_holds_the_same_points() {
    report(
        "cylinder, subnormal",
        search_cylinder(5, A_SQUARE_LOSES_ITS_PRODUCTS),
    );
}

#[test]
fn a_pipe_at_the_top_of_the_range_holds_the_same_points() {
    report(
        "cylinder, overflow",
        search_cylinder(6, A_SQUARE_OVERFLOWS_ITS_PRODUCTS),
    );
}

/// Print what a run found, then insist that it found no disagreement and that
/// it built enough configurations to have been looking at anything at all.
///
/// It deliberately does *not* insist on a certified count. Down at the bottom
/// of the range these predicates are supposed to refuse, and the plane refuses
/// nearly every time — a floor under `certified` would be a floor under the
/// very thing the guards exist to remove.
fn report(what: &str, found: Found) {
    println!("{what}: {found:?} out of {} samples", samples());
    assert_eq!(
        found.disagreements, 0,
        "{what}: a certified sign contradicts the same geometry at a different scale, \
         {} times in {} certified samples",
        found.disagreements, found.certified
    );
    assert!(
        found.considered > samples() / 2,
        "{what}: most samples were thrown away before they were asked: {found:?}"
    );
}

fn samples() -> usize {
    std::env::var("RCAD_SEARCH_SAMPLES")
        .ok()
        .and_then(|count| count.parse().ok())
        .unwrap_or(SAMPLES)
}

/// What a search run came to.
#[derive(Debug, Default)]
struct Found {
    /// Samples that made it as far as being asked, both ends included.
    considered: usize,
    /// Of those, how many the run at the end of the range certified a sign for.
    certified: usize,
    /// Of those, how many disagreed with the same geometry back in the middle
    /// of the range. Must be zero.
    disagreements: usize,
}

fn search_plane(seed: u64, targets: core::ops::RangeInclusive<i32>) -> Found {
    let mut rolls = Rolls::new(seed);
    let mut found = Found::default();

    for _ in 0..samples() {
        let facing = rolls.whole_numbers();
        let Ok(plane) = Plane::from_point_and_normal(Point::origin(), vector(facing)) else {
            continue;
        };
        // Exactly at right angles to the normal the plane was asked for, and so
        // as close to the plane it actually stored as whole numbers get.
        let on_the_plane = cross(facing, rolls.whole_numbers());
        let Some(carry) = rolls.carry_for(on_the_plane, &targets) else {
            continue;
        };
        let (Some(sample), Some(middle)) = (
            carry.out(on_the_plane),
            carry.out(on_the_plane).and_then(|out| carry.back(out)),
        ) else {
            continue;
        };

        found.considered += 1;
        let at_the_end = plane.side_of_point(PredicatePoint::from(sample));
        if !is_certified(at_the_end) {
            continue;
        }
        found.certified += 1;

        if plane.side_of_point(PredicatePoint::from(middle)) != at_the_end {
            found.disagreements += 1;
        }
    }
    found
}

fn search_sphere(seed: u64, targets: core::ops::RangeInclusive<i32>) -> Found {
    let mut rolls = Rolls::new(seed);
    let mut found = Found::default();

    for _ in 0..samples() {
        let point = rolls.whole_numbers();
        let reach = point[0] * point[0] + point[1] * point[1] + point[2] * point[2];
        let radius = rolls.radius_nearly_reaching(reach);
        let Some(carry) = rolls.carry_for(point, &targets) else {
            continue;
        };
        let (Some(sample), Some(sample_radius)) = (carry.out(point), carry.out_one(radius)) else {
            continue;
        };
        let (Some(middle), Some(middle_radius)) =
            (carry.back(sample), carry.back_one(sample_radius))
        else {
            continue;
        };
        let (Ok(at_the_end), Ok(back_in_the_middle)) = (
            Sphere::new(Point::origin(), sample_radius),
            Sphere::new(Point::origin(), middle_radius),
        ) else {
            continue;
        };

        found.considered += 1;
        let decision = at_the_end.side_of(PredicatePoint::from(sample));
        if !is_certified(decision) {
            continue;
        }
        found.certified += 1;

        if back_in_the_middle.side_of(PredicatePoint::from(middle)) != decision {
            found.disagreements += 1;
        }
    }
    found
}

fn search_cylinder(seed: u64, targets: core::ops::RangeInclusive<i32>) -> Found {
    let mut rolls = Rolls::new(seed);
    let mut found = Found::default();
    // Along a coordinate axis the stored direction is exact, so a radius built
    // from whole numbers really does land on the skin.
    let axis = Axis::z();

    for _ in 0..samples() {
        let point = rolls.whole_numbers();
        let reach = point[0] * point[0] + point[1] * point[1];
        let radius = rolls.radius_nearly_reaching(reach);
        let Some(carry) = rolls.carry_for(point, &targets) else {
            continue;
        };
        let (Some(sample), Some(sample_radius)) = (carry.out(point), carry.out_one(radius)) else {
            continue;
        };
        let (Some(middle), Some(middle_radius)) =
            (carry.back(sample), carry.back_one(sample_radius))
        else {
            continue;
        };
        let (Ok(at_the_end), Ok(back_in_the_middle)) = (
            Cylinder::new(axis, sample_radius),
            Cylinder::new(axis, middle_radius),
        ) else {
            continue;
        };

        found.considered += 1;
        let decision = at_the_end.side_of(PredicatePoint::from(sample));
        if !is_certified(decision) {
            continue;
        }
        found.certified += 1;

        if back_in_the_middle.side_of(PredicatePoint::from(middle)) != decision {
            found.disagreements += 1;
        }
    }
    found
}

fn is_certified(decision: Decision) -> bool {
    match decision {
        Decision::Neg | Decision::Zero | Decision::Pos => true,
        Decision::Undecidable { .. } => false,
        other => panic!("unhandled decision: {other:?}"),
    }
}

fn vector([x, y, z]: [f64; 3]) -> Vector {
    Vector::new(x, y, z)
}

/// `left × right`, in whole numbers small enough to be exact.
fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

/// A power of two to carry a configuration out to one end of the `f64` range
/// and back again.
///
/// The trip out may round — a coordinate pushed into the subnormals loses bits,
/// and that is the whole point of the exercise. The trip back is exact, so
/// whatever the sample landed on is what the run in the middle of the range is
/// asked about.
#[derive(Clone, Copy)]
struct Carry {
    shift: i32,
}

impl Carry {
    fn out(self, point: [f64; 3]) -> Option<[f64; 3]> {
        self.every(point, self.shift)
    }

    fn back(self, point: [f64; 3]) -> Option<[f64; 3]> {
        self.every(point, -self.shift)
    }

    fn out_one(self, value: f64) -> Option<f64> {
        finite(shifted(value, self.shift)).filter(|scaled| *scaled > 0.0)
    }

    fn back_one(self, value: f64) -> Option<f64> {
        finite(shifted(value, -self.shift)).filter(|scaled| *scaled > 0.0)
    }

    fn every(self, point: [f64; 3], shift: i32) -> Option<[f64; 3]> {
        let scaled = point.map(|coordinate| shifted(coordinate, shift));
        scaled
            .iter()
            .all(|coordinate| coordinate.is_finite())
            .then_some(scaled)
    }
}

/// `value * 2^shift`, in two halves so that the factor itself never overflows.
///
/// A single `2f64.powi(1077)` is infinity, but the number being carried is at
/// the other end of the range and the answer is an ordinary one. Splitting the
/// shift keeps every intermediate between the two ends.
fn shifted(value: f64, shift: i32) -> f64 {
    let half = shift / 2;
    value * 2f64.powi(half) * 2f64.powi(shift - half)
}

fn finite(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}

/// A reproducible source of random numbers, so the search is the same search
/// every time it runs. Splitmix64, which is four lines and needs no dependency.
struct Rolls(u64);

impl Rolls {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    fn between(&mut self, low: i64, high: i64) -> i64 {
        let width = (high - low + 1) as u64;
        low + (self.next() % width) as i64
    }

    /// Three small whole numbers, at least one of them not zero.
    fn whole_numbers(&mut self) -> [f64; 3] {
        loop {
            let drawn = [0, 1, 2].map(|_| self.between(-9, 9) as f64);
            if drawn.iter().any(|&component| component != 0.0) {
                return drawn;
            }
        }
    }

    /// A whole-number radius whose square is about `reach`, so the point the
    /// caller drew sits within a unit or two of the skin.
    fn radius_nearly_reaching(&mut self, reach: f64) -> f64 {
        let nearest = reach.sqrt().round();
        (nearest + self.between(-1, 1) as f64).max(1.0)
    }

    /// The shift that puts `point`'s biggest coordinate at a power of two drawn
    /// from `targets`. `None` for the point at the origin, which has no
    /// magnitude to move.
    fn carry_for(
        &mut self,
        point: [f64; 3],
        targets: &core::ops::RangeInclusive<i32>,
    ) -> Option<Carry> {
        let biggest = point
            .iter()
            .fold(0.0f64, |so_far, coordinate| so_far.max(coordinate.abs()));
        let target = self.between(i64::from(*targets.start()), i64::from(*targets.end())) as i32;
        (biggest > 0.0).then(|| Carry {
            shift: target - biggest.log2().floor() as i32,
        })
    }
}
