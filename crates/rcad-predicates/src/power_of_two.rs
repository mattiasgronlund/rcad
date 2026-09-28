/// The exponent `k` for which multiplying every coordinate by `2^k` puts the
/// arithmetic back in the middle of the `f64` range.
///
/// `differences` are the differences the predicate is going to form. Those are
/// what has to land in range, not the raw coordinates: these predicates
/// subtract first and multiply the differences afterwards, so a model sitting a
/// long way from the origin can have perfectly ordinary differences and a model
/// at the origin can have unusable ones.
///
/// The largest of them is brought into `[1, 2)`, which leaves every smaller one
/// as far above the bottom of the range as it can be while nothing risks the
/// top.
///
/// `None` when they are all zero, because then there is nothing to bring
/// anywhere and the determinant is exactly zero whatever scale it is read at.
///
/// A difference that overflowed to infinity gets exponent 1024 — one past the
/// largest an `f64` carries — which is the right answer: the true difference is
/// at least `2^1024`, so that is how far down it has to come.
pub(crate) fn exponent_that_brings_into_range(
    differences: impl IntoIterator<Item = f64>,
) -> Option<i32> {
    let largest = differences.into_iter().map(f64::abs).fold(0.0, f64::max);
    if largest == 0.0 {
        return None;
    }
    Some(-binary_exponent(largest))
}

/// The exponent `k` for which multiplying every coordinate by `2^k` puts the
/// largest of them in `[2^top, 2^(top + 1))`.
///
/// This is the second exponent a predicate tries, and it reads the raw
/// coordinates rather than the differences because level 3 does: it expands
/// the determinant over the coordinates themselves, so what has to stay clear
/// of the bottom of the range is every product of two or three of them. Each of
/// those products grows as `k` grows, and the only thing that stops `k` growing
/// is the largest product running off the top. So the `k` that takes the
/// largest coordinate as high as the predicate's `top` allows is the one that
/// lifts every smaller product furthest off the bottom. A predicate that asks
/// for it passes its own `top`, worked out from its own degree.
///
/// `None` when every coordinate is zero.
pub(crate) fn exponent_that_lifts_the_largest_to(
    coordinates: impl IntoIterator<Item = f64>,
    top: i32,
) -> Option<i32> {
    let largest = coordinates.into_iter().map(f64::abs).fold(0.0, f64::max);
    if largest == 0.0 {
        return None;
    }
    Some(top - binary_exponent(largest))
}

/// The largest `k` for which multiplying every coordinate by `2^k` leaves the
/// binades of each product's factors adding up to no more than `top`, and every
/// coordinate still an `f64`.
///
/// This is the third exponent a predicate tries.
/// [`exponent_that_lifts_the_largest_to`] assumes the largest product level 3
/// forms is the largest coordinate times itself, and for a spike at the origin
/// under a unit square that is false: its determinants hold `tiny * tiny` and
/// `tiny * 1` and never `1 * 1`, so the second exponent stops far below where
/// anything would overflow and leaves `tiny * tiny` under
/// [`product_is_exact`](crate::expansion::product_is_exact)'s floor
/// (`docs/decisions.md` §324.7). Here the products are read one by one instead,
/// and the one closest to the top is what stops `k`.
///
/// A product of `D` factors in binades `e_1 ... e_D` is below
/// `2^(e_1 + ... + e_D + D)`, and scaling moves each factor up by `k`, so the
/// sum moves up by `D * k`. A product with a zero factor is zero at every scale
/// and sets no limit. The largest coordinate goes no higher than the top binade
/// `f64` has, whatever the products allow.
///
/// `None` when every coordinate is zero.
pub(crate) fn exponent_that_lifts_every_product_to<const D: usize>(
    coordinates: impl IntoIterator<Item = f64>,
    products: impl IntoIterator<Item = [f64; D]>,
    top: i32,
) -> Option<i32> {
    let as_high_as_f64_goes = exponent_that_lifts_the_largest_to(coordinates, f64::MAX_EXP - 1)?;
    let degree = i32::try_from(D).expect("a product has a handful of factors");
    let as_high_as_the_products_go = products
        .into_iter()
        .filter(|factors| factors.iter().all(|&factor| factor != 0.0))
        .map(|factors| {
            let binades: i32 = factors.into_iter().map(binary_exponent).sum();
            (top - binades).div_euclid(degree)
        })
        .min();
    Some(as_high_as_the_products_go.map_or(as_high_as_f64_goes, |k| k.min(as_high_as_f64_goes)))
}

/// `exponents` in the order given, each once, leaving out zero and the missing
/// ones.
///
/// Zero is the input as it came, which has already been refused, and an
/// exponent already tried has already given its answer.
pub(crate) fn each_once_and_never_zero<const N: usize>(
    exponents: [Option<i32>; N],
) -> impl Iterator<Item = i32> {
    exponents
        .into_iter()
        .enumerate()
        .filter_map(move |(place, exponent)| {
            let exponent = exponent?;
            (exponent != 0 && !exponents[..place].contains(&Some(exponent))).then_some(exponent)
        })
}

/// `coordinates`, each multiplied by `2^exponent` — or `None` when that
/// multiplication is not exact for one of them.
///
/// Exactness is the whole value of the technique: a scaled copy that lost bits
/// describes different geometry, and a sign proved about it says nothing about
/// the input. Multiplying by a power of two is exact while the result stays a
/// normal number, and stops being exact at both ends — past the top it is
/// infinity, and below the bottom it is rounded onto the subnormal grid or
/// flushed to zero outright. So each result is asked whether it is normal, and
/// one that is not sinks the whole attempt.
///
/// That check is what tells a limit from a gap. Coordinates far from the origin
/// whose differences are far below it cannot be brought into range together:
/// lifting the differences off the bottom pushes the coordinates past the top,
/// and no single `exponent` does both. Such an input has no scaled copy and
/// keeps the refusal it earned.
///
/// A coordinate of exactly zero scales to zero exactly, at any exponent.
///
/// The answer does not depend on where `exponent` came from. Any exponent at
/// all either yields an exact copy or is refused here.
pub(crate) fn scaled_by_a_power_of_two<const N: usize>(
    coordinates: [f64; N],
    exponent: i32,
) -> Option<[f64; N]> {
    let mut scaled = coordinates;
    for coordinate in &mut scaled {
        if *coordinate == 0.0 {
            continue;
        }
        let moved = multiplied_by_a_power_of_two(*coordinate, exponent);
        if !moved.is_normal() {
            return None;
        }
        *coordinate = moved;
    }
    Some(scaled)
}

/// The `e` with `2^e <= |value| < 2^(e+1)`, read straight out of the bits.
///
/// Read rather than computed, because `log2` rounds and this has to be the
/// exact exponent — one binade out and a scaling that would have worked is
/// refused, or one that overflows is tried.
///
/// `value` must not be zero; zero has no such exponent.
fn binary_exponent(value: f64) -> i32 {
    assert!(value != 0.0, "zero has no binary exponent");
    let bits = value.abs().to_bits();
    let stored = (bits >> 52) as i32;
    if stored > 0 {
        return stored - 1023;
    }
    // Subnormal: the exponent field says nothing and the leading one is
    // somewhere in the mantissa, which stands for a multiple of `2^-1074`.
    let mantissa = bits & ((1 << 52) - 1);
    63 - mantissa.leading_zeros() as i32 - 1074
}

/// `value * 2^exponent`, in two steps so the factor itself never leaves the
/// range.
///
/// `2f64.powi(1074)` is infinity, and a single multiplication by that throws
/// away products that would have been ordinary numbers. Halving the exponent
/// keeps both factors well inside what `f64` holds. Each step moves the
/// magnitude the same way, so neither intermediate is further out than the
/// answer, and an answer that is normal was reached exactly.
fn multiplied_by_a_power_of_two(value: f64, exponent: i32) -> f64 {
    let half = exponent / 2;
    value * 2f64.powi(half) * 2f64.powi(exponent - half)
}

#[cfg(test)]
mod tests {
    use super::{
        binary_exponent, each_once_and_never_zero, exponent_that_brings_into_range,
        exponent_that_lifts_every_product_to, exponent_that_lifts_the_largest_to,
        multiplied_by_a_power_of_two, scaled_by_a_power_of_two,
    };

    #[test]
    fn the_exponent_of_an_ordinary_number_is_the_binade_it_sits_in() {
        assert_eq!(binary_exponent(1.0), 0);
        assert_eq!(binary_exponent(1.9999), 0);
        assert_eq!(binary_exponent(2.0), 1);
        assert_eq!(binary_exponent(-8.0), 3);
        assert_eq!(binary_exponent(0.5), -1);
        assert_eq!(binary_exponent(f64::MIN_POSITIVE), -1022);
        assert_eq!(binary_exponent(f64::MAX), 1023);
    }

    /// Down here the exponent field has run out and the leading one is inside
    /// the mantissa, so the bits have to be counted instead of read.
    #[test]
    fn a_subnormal_still_has_an_exponent() {
        assert_eq!(binary_exponent(f64::from_bits(1)), -1074);
        assert_eq!(binary_exponent(f64::from_bits(1) * 4.0), -1072);
        assert_eq!(binary_exponent(f64::MIN_POSITIVE / 2.0), -1023);
    }

    /// One past the top, which is what an overflowed difference is worth.
    #[test]
    fn an_overflowed_difference_counts_as_one_binade_past_the_largest_f64() {
        assert_eq!(binary_exponent(f64::INFINITY), 1024);
    }

    #[test]
    fn the_largest_difference_is_what_lands_in_range() {
        let exponent = exponent_that_brings_into_range([1e-200, 0.0, -3e-201]).unwrap();
        let brought_back = multiplied_by_a_power_of_two(1e-200, exponent);
        assert!((1.0..2.0).contains(&brought_back), "{brought_back}");
    }

    #[test]
    fn differences_that_are_all_zero_have_nothing_to_normalise() {
        assert_eq!(exponent_that_brings_into_range([0.0, -0.0, 0.0]), None);
    }

    #[test]
    fn the_largest_coordinate_is_what_lands_in_the_binade_asked_for() {
        let exponent = exponent_that_lifts_the_largest_to([1e-300, -2.0, 0.5], 338).unwrap();
        let lifted = multiplied_by_a_power_of_two(-2.0, exponent);
        assert_eq!(lifted, -(2f64.powi(338)));
        assert_eq!(exponent_that_lifts_the_largest_to([0.0, -0.0], 338), None);
    }

    /// `2^-10 * 2^-20` has binades adding up to `-30`, so `k = 20` takes the sum
    /// to `10`, and `2^-10 * 2^-20` is the product closest to the top. The
    /// product with a zero factor sets nothing.
    #[test]
    fn the_product_closest_to_the_top_is_what_stops_the_lift() {
        let tiny = [2f64.powi(-10), -(2f64.powi(-20)), 0.0];
        let products = [[tiny[0], tiny[1]], [tiny[1], tiny[1]], [tiny[0], tiny[2]]];
        assert_eq!(
            exponent_that_lifts_every_product_to(tiny, products, 10),
            Some(20)
        );
        assert_eq!(
            exponent_that_lifts_every_product_to(tiny, products, 11),
            Some(20),
            "a sum that is odd rounds down, never up past the top"
        );
    }

    /// With no product to stop it, the largest coordinate is lifted into the
    /// top binade `f64` has, and no further.
    #[test]
    fn with_every_product_zero_the_lift_stops_at_the_top_of_f64() {
        let coordinates = [0.5, 0.0];
        let exponent =
            exponent_that_lifts_every_product_to(coordinates, [[0.5, 0.0]], 1000).unwrap();
        assert_eq!(multiplied_by_a_power_of_two(0.5, exponent), 2f64.powi(1023));
        assert_eq!(
            exponent_that_lifts_every_product_to(coordinates, [[0.5, 0.5]], 5000),
            Some(exponent)
        );
        assert_eq!(
            exponent_that_lifts_every_product_to([0.0, -0.0], [[0.0, 0.0]], 1000),
            None
        );
    }

    #[test]
    fn each_exponent_is_tried_once_in_order_and_zero_never() {
        let tried: Vec<i32> =
            each_once_and_never_zero([Some(3), None, Some(0), Some(3), Some(-7), Some(3)])
                .collect();
        assert_eq!(tried, [3, -7]);
    }

    #[test]
    fn scaling_by_a_power_of_two_only_changes_the_exponent() {
        let scaled = scaled_by_a_power_of_two([1.5, -0.75, 0.0], 600).unwrap();
        assert_eq!(scaled, [1.5 * 2f64.powi(600), -0.75 * 2f64.powi(600), 0.0]);
    }

    /// The factor `2^1074` is itself infinity, so a single multiplication would
    /// throw this away.
    #[test]
    fn a_scaling_larger_than_any_f64_still_works_on_a_small_enough_number() {
        let scaled = scaled_by_a_power_of_two([f64::from_bits(1)], 1074).unwrap();
        assert_eq!(scaled, [1.0]);
    }

    #[test]
    fn a_coordinate_pushed_past_the_top_of_the_range_sinks_the_whole_scaling() {
        assert_eq!(scaled_by_a_power_of_two([1.0, 1e300], 100), None);
    }

    /// Flushing to zero is the quiet one: the number is still finite, and every
    /// bit of it is gone.
    #[test]
    fn a_coordinate_pushed_below_the_bottom_of_the_range_sinks_it_too() {
        assert_eq!(scaled_by_a_power_of_two([1.0, 1e-300], -100), None);
        assert_eq!(scaled_by_a_power_of_two([2f64.powi(-1000)], -100), None);
    }

    /// A coordinate that arrived subnormal is not exact where it is, and
    /// lifting it into the normal range is exact — subnormals carry fewer
    /// significant bits, not more.
    #[test]
    fn a_subnormal_coordinate_can_be_lifted_into_the_normal_range() {
        let subnormal = f64::MIN_POSITIVE / 4.0;
        let scaled = scaled_by_a_power_of_two([subnormal], 100).unwrap();
        assert_eq!(scaled, [subnormal * 2f64.powi(100)]);
        assert!(scaled[0].is_normal());
    }
}
