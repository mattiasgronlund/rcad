//! M1 exit criterion 13: how wide interval de Boor's enclosure actually is.
//!
//! This is a number that decides a later milestone's design rather than a
//! curiosity. If a degree-3 curve at a working size of 100 came back with a box
//! thousands of ulps across, M3's certified surface marching would not work and
//! its design would have to change — so it is measured now, not in eighteen
//! months when the change is expensive.
//!
//! Widths are reported in `f64` steps rather than in millimetres, because an
//! absolute width says nothing except how big the model is. Two readings, and
//! the second is the one to judge by:
//!
//! - **in ulps of the coordinate** is what you would ask for first, and it
//!   blows up wherever a coordinate passes near zero. The degree-5 patch below
//!   reaches twenty million of them at a coordinate of `-1.2e-7` whose box is
//!   `5.6e-16` across — five `f64` steps of the model it came from. Nothing is
//!   wrong there except the denominator.
//! - **in ulps of the model's own size** is the same fact
//!   `EVALUATION_ROUNDOFF`'s derivation opens with: a bound on de Boor is
//!   absolute in the size of the input and never relative to the answer,
//!   because a control net has signs in it and `(+1) + (−1)` comes out as zero
//!   with a rounding still attached. This is what the ceiling is set on.
//!
//! A `criterion` run is something a human reads, and `docs/decisions.md` §12.5
//! was honest that nobody notices one regressing. So the ceiling lives in the
//! normal test suite, in
//! `tests/an_enclosure_stays_within_a_few_ulps_of_the_point.rs`, and this file
//! is where the figure behind it comes from.

#![forbid(unsafe_code)]

#[path = "../tests/support/mod.rs"]
mod support;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use support::{
    a_curve_with_a_wide_dynamic_range, a_surface_with_a_wide_dynamic_range,
    a_well_conditioned_curve, a_well_conditioned_surface, widest_enclosure_of_a_curve,
    widest_enclosure_of_a_surface,
};

/// Every degree the generated families reach. The cap is 15, but a degree
/// nobody draws with tells us nothing about whether marching works.
const DEGREES: [usize; 5] = [1, 2, 3, 4, 5];

/// A millimetre part, a hand-sized part, a machine, and a building.
const WORKING_SIZES: [f64; 5] = [1.0, 100.0, 1e3, 1e4, 1e6];

fn how_wide_the_enclosure_gets(criterion: &mut Criterion) {
    report_the_widths();

    let mut group = criterion.benchmark_group("evaluating_with_an_enclosure");
    for degree in DEGREES {
        let curve = a_well_conditioned_curve(degree, 100.0);
        group.bench_function(BenchmarkId::new("curve_in_f64", degree), |bencher| {
            bencher.iter(|| curve.evaluate(0.3));
        });
        group.bench_function(BenchmarkId::new("curve_enclosed", degree), |bencher| {
            bencher.iter(|| curve.evaluate_with_enclosure(0.3));
        });

        let surface = a_well_conditioned_surface(degree, 100.0);
        group.bench_function(BenchmarkId::new("surface_in_f64", degree), |bencher| {
            bencher.iter(|| surface.evaluate(0.3, 0.7));
        });
        group.bench_function(BenchmarkId::new("surface_enclosed", degree), |bencher| {
            bencher.iter(|| surface.evaluate_with_enclosure(0.3, 0.7));
        });
    }
    group.finish();
}

/// The table that goes into the milestone's written record.
fn report_the_widths() {
    println!("\nwidest enclosure, in f64 steps of the model's own size");
    println!("(in brackets: the same box in steps of the coordinate itself)\n");
    println!(
        "{:>6}  {:>6}  {:>18}  {:>18}  {:>18}  {:>18}",
        "degree", "size", "curve well", "curve wide", "patch well", "patch wide"
    );

    for degree in DEGREES {
        for size in WORKING_SIZES {
            let readings = [
                widest_enclosure_of_a_curve(&a_well_conditioned_curve(degree, size)),
                widest_enclosure_of_a_curve(&a_curve_with_a_wide_dynamic_range(degree, size)),
                widest_enclosure_of_a_surface(&a_well_conditioned_surface(degree, size)),
                widest_enclosure_of_a_surface(&a_surface_with_a_wide_dynamic_range(degree, size)),
            ];
            print!("{degree:>6}  {size:>6.0e}");
            for reading in readings {
                print!(
                    "  {:>8.1} ({:>7.0e})",
                    reading.in_ulps_of_the_size, reading.in_ulps_of_the_coordinate
                );
            }
            println!();
        }
    }
    println!();
}

criterion_group!(benches, how_wide_the_enclosure_gets);
criterion_main!(benches);
