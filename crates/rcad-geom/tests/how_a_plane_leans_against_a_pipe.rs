//! Whether a plane holds a pipe's axis direction, asked of the one question
//! `intersect` asks it with.
//!
//! [`how_a_plane_leans_against_a_pipe`] is the sign of `n·w` for the plane's
//! stored normal and the pipe's stored axis direction. `rcad-topo`'s door used to
//! spell that sign a second time, as a hand-written sum of exact products; it
//! now asks this function, so what has to hold is that the cascade behind it
//! decides every pair that sum decided, and gives the same sign.

use rcad_geom::{Axis, Cylinder, Plane, Point, Vector, how_a_plane_leans_against_a_pipe};
use rcad_predicates::Decision;
use rcad_predicates::expansion::{expansion_sign, expansion_sum, product_is_exact, two_product};

fn plane_square_to(normal: Vector) -> Plane {
    Plane::from_point_and_normal(Point::origin(), normal).expect("a plane")
}

fn pipe_along(direction: Vector) -> Cylinder {
    Cylinder::new(Axis::new(Point::origin(), direction).expect("an axis"), 1.0)
        .expect("a pipe of radius one")
}

#[test]
fn a_plane_holding_the_axis_direction_is_flush_with_it() {
    let flat = plane_square_to(Vector::new(0.0, 0.0, 1.0));
    for along in [
        Vector::new(1.0, 0.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        Vector::new(0.3, -0.7, 0.0),
    ] {
        assert_eq!(
            how_a_plane_leans_against_a_pipe(&flat, &pipe_along(along)),
            Decision::Zero,
            "every product in `n·w` is exactly nothing for {along:?}"
        );
    }
}

#[test]
fn a_plane_cutting_across_the_axis_leans_the_way_the_dot_says() {
    let flat = plane_square_to(Vector::new(0.0, 0.0, 1.0));
    assert_eq!(
        how_a_plane_leans_against_a_pipe(&flat, &pipe_along(Vector::new(0.0, 0.0, 1.0))),
        Decision::Pos
    );
    assert_eq!(
        how_a_plane_leans_against_a_pipe(&flat, &pipe_along(Vector::new(0.0, 0.0, -1.0))),
        Decision::Neg
    );
    assert_eq!(
        how_a_plane_leans_against_a_pipe(&flat, &pipe_along(Vector::new(1.0, 2.0, -3.0))),
        Decision::Neg
    );
}

/// The pair `rcad-topo`'s `a_plane_a_rounded_dot_calls_flush_with_an_axis_is_not`
/// writes out: the `f64` dot is exactly `0.0` and the true one is above zero.
#[test]
fn a_plane_a_rounded_dot_calls_flush_with_the_axis_is_not() {
    let normal = Vector::new(
        0.082_988_092_356_092_5,
        -0.977_481_762_337_168_9,
        0.194_016_444_729_097_85,
    );
    let along = Vector::new(
        0.292_520_127_841_236_6,
        -0.162_218_223_338_325_75,
        -0.942_399_714_996_085_8,
    );
    let flat = plane_square_to(normal);
    let pipe = pipe_along(along);
    assert_eq!(flat.normal(), normal, "the plane stores the normal it was given");
    assert_eq!(pipe.axis().direction(), along, "and the axis its direction");
    assert_eq!(flat.normal().dot(pipe.axis().direction()), 0.0);

    assert_eq!(how_a_plane_leans_against_a_pipe(&flat, &pipe), Decision::Pos);
    assert_eq!(
        sum_of_exact_products(normal.components(), along.components()),
        Some(Decision::Pos),
        "the oracle below agrees, so this pair is a witness and not a disagreement"
    );
}

/// **The cascade decides every pair the hand-written exact sum decided, with the
/// same sign.** The sum is the one `rcad-topo` carried before it asked
/// [`how_a_plane_leans_against_a_pipe`]; it is kept here as the oracle.
///
/// Draws of three kinds: random normals and directions, normals built square to
/// the direction in `f64` and nudged by one ulp so `n·w` is at or near nothing
/// and only the exact level can tell, and components scaled down far enough
/// that a product lands among the subnormals, which is where the sum gives up.
/// There the cascade's interval level can still settle a pair whose other
/// products dominate, so the cascade may decide **more** — never fewer.
#[test]
fn the_cascade_decides_every_pair_the_exact_sum_did_and_agrees_on_the_sign() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    let (mut compared, mut near_flush, mut only_the_cascade, mut neither) = (0, 0, 0, 0);
    for draw in 0..60_000_u64 {
        let mut unit = || (next() >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0;
        let mut along = [unit(), unit(), unit()];
        let mut normal = match draw % 3 {
            0 => [unit(), unit(), unit()],
            1 => {
                let [x, y, z] = Vector::from(along)
                    .cross(Vector::new(unit(), unit(), unit()))
                    .components();
                let mut built = [x, y, z];
                let which = (draw / 3 % 3) as usize;
                built[which] = f64::from_bits(built[which].to_bits() + 1);
                built
            }
            _ => [unit(), unit(), unit()],
        };
        if draw % 3 == 2 {
            let shift = (next() % 700) as i32;
            normal[(next() % 3) as usize] *= 2f64.powi(-shift);
            along[(next() % 3) as usize] *= 2f64.powi(-shift);
        }

        let Ok(flat) = Plane::from_point_and_normal(Point::origin(), Vector::from(normal)) else {
            continue;
        };
        let Ok(axis) = Axis::new(Point::origin(), Vector::from(along)) else {
            continue;
        };
        let pipe = Cylinder::new(axis, 1.0).expect("a pipe of radius one");

        let asked = how_a_plane_leans_against_a_pipe(&flat, &pipe);
        match sum_of_exact_products(
            flat.normal().components(),
            pipe.axis().direction().components(),
        ) {
            Some(summed) => {
                compared += 1;
                if flat.normal().dot(pipe.axis().direction()).abs() < 1e-15 {
                    near_flush += 1;
                }
                assert_eq!(
                    asked,
                    summed,
                    "the exact sum decided {summed:?} and the cascade said {asked:?} for \
                     n = {:?}, w = {:?}",
                    flat.normal(),
                    pipe.axis().direction()
                );
            }
            None => match asked {
                Decision::Pos | Decision::Neg | Decision::Zero => only_the_cascade += 1,
                Decision::Undecidable { .. } => neither += 1,
                other => panic!("a sign question answered {other:?}"),
            },
        }
    }

    println!(
        "{compared} pairs both decided and agreed on ({near_flush} of them within 1e-15 of \
         flush); {only_the_cascade} only the cascade decided; {neither} neither did"
    );
    assert!(compared > 30_000, "the draws reach the comparison: {compared}");
    assert!(near_flush > 1_000, "and a good share of them sit at the knife edge: {near_flush}");
    assert!(
        only_the_cascade > 0,
        "and some reach the subnormals, where the sum gives up: {only_the_cascade}"
    );
}

/// `n·w` as three exact products summed with no rounding thrown away, or
/// [`None`] when a product falls into the subnormals and is no longer exact.
fn sum_of_exact_products(one: [f64; 3], other: [f64; 3]) -> Option<Decision> {
    let mut total = [0.0; 6];
    let mut len = 0;
    for axis in 0..3 {
        if !product_is_exact(one[axis], other[axis]) {
            return None;
        }
        let (high, low) = two_product(one[axis], other[axis]);
        let mut grown = [0.0; 6];
        len = expansion_sum(&total[..len], &[low, high], &mut grown);
        total[..len].copy_from_slice(&grown[..len]);
    }
    Some(expansion_sign(&total[..len]))
}
