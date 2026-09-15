//! Where a crossing's direction goes when its two surfaces come parallel, and
//! what does and does not get it back.
//!
//! The marcher asks which way a crossing runs by crossing the two unit normals
//! together and scaling the answer to length one
//! (`which_way_the_crossing_runs` in `march.rs`), and refuses with
//! `CannotFollowACrossingWhereItsSurfacesAreTangent` when that cannot be scaled.
//! Its doc comment says the refusal is for normals that "are parallel". These
//! tests say at what angle that happens and why nothing moves it.
//!
//! # The construction
//!
//! Two unit vectors square to each other, `u` and `t`. Lean `u` towards `t` by
//! an angle: `v = u·cos θ + t·sin θ`. Then `u × v = sin θ · (u × t)` **exactly**,
//! so the true direction of the crossing is `u × t` — and that one is a cross of
//! two vectors at right angles, where no cancellation can happen, so `f64` gets
//! it right to a rounding. That gives every test below a direction to be
//! measured against that is not itself the thing under test.

use rcad_geom::{Error, Vector};

/// Angle between two directions, in radians, counting a flip as no angle: a
/// tangent that points the other way is still the same line to follow.
fn angle_between(one: Vector, other: Vector) -> f64 {
    one.dot(other).abs().clamp(-1.0, 1.0).acos()
}

/// `acos` near one loses half the digits, so no angle measured this way can come
/// back smaller than about the root of a rounding. Nothing below this is a
/// measurement.
const SMALLEST_ANGLE_THIS_CAN_MEASURE: f64 = 3e-8;

struct Spinner(u64);

impl Spinner {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 11) as f64) / ((1u64 << 53) as f64) * 2.0 - 1.0
    }

    /// A direction in no particular place, well away from the axes so that no
    /// component is accidentally zero and no product cancels for free.
    fn a_direction(&mut self) -> Vector {
        loop {
            let tried = Vector::new(self.next(), self.next(), self.next());
            let length = tried.length();
            if (0.3..1.0).contains(&length) {
                return tried
                    .normalized()
                    .expect("a length between a third and one");
            }
        }
    }

    /// A pair of unit directions square to each other.
    fn two_at_right_angles(&mut self) -> (Vector, Vector) {
        loop {
            let one = self.a_direction();
            let other = self.a_direction();
            let across = other - one * other.dot(one);
            if across.length() > 0.3 {
                return (one, across.normalized().expect("a third of a unit"));
            }
        }
    }
}

/// `u` leaned towards `t` by `angle`, scaled to length one the way a surface's
/// normal comes back from `how_far_off_and_which_way`.
fn leaned(u: Vector, t: Vector, angle: f64) -> Vector {
    (u * angle.cos() + t * angle.sin())
        .normalized()
        .expect("leaning a unit vector leaves a unit vector")
}

/// `a·b − c·d`, correctly rounded, by Kahan: the product that is subtracted is
/// taken back exactly with a fused multiply-add, so the difference is not a
/// difference of two rounded numbers. This is the "compute the cross product
/// without the cancellation" fix, and the tests below are what it is worth.
fn difference_of_products(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let taken_away = c * d;
    let what_that_rounding_lost = c.mul_add(d, -taken_away);
    let difference = a.mul_add(b, -taken_away);
    difference - what_that_rounding_lost
}

fn crossed_without_the_cancellation(one: Vector, other: Vector) -> Vector {
    let [ax, ay, az] = one.components();
    let [bx, by, bz] = other.components();
    Vector::new(
        difference_of_products(ay, bz, az, by),
        difference_of_products(az, bx, ax, bz),
        difference_of_products(ax, by, ay, bx),
    )
}

/// **The boundary, measured.** Two normals leaning closer than about one
/// rounding apart cross to exactly nothing, and that is the whole of the
/// refusal.
///
/// It is cancellation and not underflow. Each component of the cross product is
/// a difference of two products of numbers no bigger than one, so each is
/// computed with an error of about a rounding *however small the answer is*.
/// Once `sin θ` drops to a rounding the components are nothing but that error,
/// and they cancel to exactly zero often enough to refuse. Underflow would want
/// `sin θ` near `1e-308`, which is nineteen powers of ten further down and never
/// gets a chance to happen.
#[test]
fn two_normals_leaning_closer_than_one_rounding_apart_cross_to_exactly_nothing() {
    let mut spinner = Spinner(0x5eed_0001);
    let orientations: Vec<(Vector, Vector)> =
        (0..500).map(|_| spinner.two_at_right_angles()).collect();

    let refuses_at = |angle: f64| {
        orientations
            .iter()
            .filter(|(u, t)| {
                let v = leaned(*u, *t, angle);
                u.cross(v).normalized() == Err(Error::ZeroLength { what: "vector" })
            })
            .count()
    };

    let a_rounding = f64::EPSILON / 2.0;
    assert_eq!(
        refuses_at(a_rounding * 16.0),
        0,
        "sixteen roundings of lean still leaves a direction in the cross product"
    );

    let far_below = refuses_at(a_rounding / 16.0);
    assert!(
        far_below > orientations.len() / 2,
        "a sixteenth of a rounding of lean is below the noise, so most orientations \
         cross to exactly nothing, not {far_below} of {}",
        orientations.len()
    );

    let smallest_lean_that_always_answers = (0..80)
        .map(|halvings| a_rounding * 16.0 * 0.5_f64.powi(halvings))
        .take_while(|angle| refuses_at(*angle) == 0)
        .last()
        .expect("the first one answers");
    // A boundary set by underflow would sit near `1e-308`. This range puts it
    // within a million of a rounding, which is two hundred and ninety powers of
    // ten clear of that, so the test really does tell the two apart.
    assert!(
        (f64::EPSILON / 1e6..=f64::EPSILON).contains(&smallest_lean_that_always_answers),
        "the boundary is where a rounding is, not where underflow is: \
         {smallest_lean_that_always_answers:e}"
    );
    println!(
        "every orientation still answers down to a lean of {smallest_lean_that_always_answers:e}, \
         which is {:.2} roundings",
        smallest_lean_that_always_answers / a_rounding
    );
}

/// **Scaling is the fix that cannot work, and here is why rather than a
/// paragraph saying so.** Scaling both normals by a power of two is *exact* —
/// every component's exponent moves and no bit of any significand changes — so
/// the cross product comes back with the same significands and a moved exponent,
/// and a direction that was lost is lost by exactly the same bits.
///
/// This is the claim §16.10 was written about: "no ordering of the arithmetic
/// helps" was once true of orderings and silent about scalings. So the scaling
/// is tried, at a scale big enough to lift every product clear of the subnormals
/// and one small enough to push them into them, and it is bit-for-bit the same
/// answer both ways.
#[test]
fn scaling_both_normals_by_a_power_of_two_leaves_the_same_bits_behind() {
    let mut spinner = Spinner(0x5eed_0002);
    let a_rounding = f64::EPSILON / 2.0;

    for _ in 0..500 {
        let (u, t) = spinner.two_at_right_angles();
        for lean in [a_rounding * 4.0, a_rounding, a_rounding / 8.0] {
            let v = leaned(u, t, lean);
            let plain = u.cross(v);
            for power in [-500i32, -60, 60, 500] {
                let scale = 2.0_f64.powi(power);
                let scaled = (u * scale).cross(v * scale);
                let [x, y, z] = scaled.components();
                let [px, py, pz] = plain.components();
                let back = scale * scale;
                assert_eq!(
                    [x / back, y / back, z / back],
                    [px, py, pz],
                    "scaling by 2^{power} is exact, so it can only move the exponent"
                );
            }
        }
    }
}

/// **Taking the cancellation out gets the arithmetic's own answer back, and it
/// is worth nothing.**
///
/// A cross product computed with fused multiply-adds is correctly rounded, so it
/// hands back the direction the two `f64` normals really span at any angle at
/// all. Measured against the *true* direction of the crossing, it is no better
/// than the plain one — because by then the error is in the normals and not in
/// the cross product.
///
/// That is the demonstration that the family is exhausted. Anything that reads
/// only the two unit normals — a different order, a compensated sum, an exactly
/// rounded determinant, a basis where the subtraction is not there — is a
/// function of those two vectors, and this test says that function's input no
/// longer carries the answer.
#[test]
fn taking_the_cancellation_out_of_the_cross_product_buys_nothing_against_the_truth() {
    let mut spinner = Spinner(0x5eed_0003);

    println!("lean          plain      no cancellation");
    let mut worst_ratio = 0.0_f64;
    for halvings in 0..14 {
        let lean = f64::EPSILON * 256.0 * 0.5_f64.powi(halvings);
        let mut plain_worst = 0.0_f64;
        let mut compensated_worst = 0.0_f64;
        for _ in 0..500 {
            let (u, t) = spinner.two_at_right_angles();
            let truth = u
                .cross(t)
                .normalized()
                .expect("two vectors at right angles");
            let v = leaned(u, t, lean);
            if let Ok(plain) = u.cross(v).normalized() {
                plain_worst = plain_worst.max(angle_between(plain, truth));
            }
            if let Ok(compensated) = crossed_without_the_cancellation(u, v).normalized() {
                compensated_worst = compensated_worst.max(angle_between(compensated, truth));
            }
        }
        println!("{lean:9.3e}   {plain_worst:9.3e}   {compensated_worst:9.3e}");
        if compensated_worst > SMALLEST_ANGLE_THIS_CAN_MEASURE {
            worst_ratio = worst_ratio.max(plain_worst / compensated_worst);
        }
    }
    assert!(
        worst_ratio < 30.0,
        "taking the cancellation out never bought more than a rounding or two of \
         direction, and it bought {worst_ratio:.1} times here"
    );
}

/// **And the reason it buys nothing: the answer is not in the normals.**
///
/// One rounding of either normal — the least either of them can be wrong by,
/// since each arrives scaled to length one by a division — swings the exactly
/// computed cross product by about a rounding divided by the lean. By the time
/// the two surfaces lean a rounding apart that swing is a quarter turn, and the
/// direction the marcher would follow is whichever way the last bit of a
/// division happened to fall.
///
/// So this is a **limit and not a gap** (§16.10): below a lean of about
/// `1e-16` the crossing's direction does not exist in the two `f64` normals, and
/// no arithmetic performed on them can put it there.
#[test]
fn one_rounding_of_either_normal_swings_the_direction_a_quarter_turn() {
    let mut spinner = Spinner(0x5eed_0004);
    let nudged = |v: Vector| {
        let [x, y, z] = v.components();
        Vector::new(f64::from_bits(x.to_bits() ^ 1), y, z)
    };

    let mut swing_at_a_rounding_of_lean = 0.0_f64;
    let mut swing_at_a_million_roundings = 0.0_f64;
    for _ in 0..2000 {
        let (u, t) = spinner.two_at_right_angles();
        for (lean, worst) in [
            (f64::EPSILON / 2.0, &mut swing_at_a_rounding_of_lean),
            (f64::EPSILON * 5e5, &mut swing_at_a_million_roundings),
        ] {
            let v = leaned(u, t, lean);
            let here = crossed_without_the_cancellation(u, v).normalized();
            let there = crossed_without_the_cancellation(nudged(u), nudged(v)).normalized();
            if let (Ok(here), Ok(there)) = (here, there) {
                *worst = worst.max(angle_between(here, there));
            }
        }
    }

    println!(
        "a lean of one rounding swings by {swing_at_a_rounding_of_lean:9.3e} rad, \
         a lean of a million roundings by {swing_at_a_million_roundings:9.3e} rad"
    );
    assert!(
        swing_at_a_rounding_of_lean > 0.5,
        "one rounding of the normals must be able to turn the answer right round, \
         not {swing_at_a_rounding_of_lean:e}"
    );
    assert!(
        swing_at_a_million_roundings < 1e-4,
        "and a million roundings of lean must leave the direction settled, \
         not {swing_at_a_million_roundings:e}"
    );
}
