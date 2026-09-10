//! M0's exit criterion 10: point classification must scale to at least 0.8 of
//! linear across the machine's physical cores, using nothing but the public API
//! (decisions.md §9.2, §7.3, roadmap §10).
//!
//! Nothing here reaches inside the kernel. The library has no `rayon`
//! dependency and no `parallel` feature; the whole benchmark is one `&Model`
//! shared by every worker. That is the point — if this scales, roadmap §10's
//! claim that parallel reads are free is demonstrated rather than asserted.
//!
//! The bit-identity half of §9.2 is checked here too, so a `cargo bench` run
//! reports it, but it lives properly in
//! `the_parallel_run_matches_the_serial_one.rs` where the normal test run
//! executes it.

#![forbid(unsafe_code)]

mod plate_with_holes;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rayon::ThreadPoolBuilder;

use plate_with_holes::{
    classify_across_threads, classify_one_at_a_time, first_difference, grid_over, plate_with_holes,
    tally,
};

/// 13³ is 2197 queries, a few thousand `orient3d` calls each. Big enough that
/// one worker's share still dwarfs the cost of handing it out, so what gets
/// measured is the kernel and not `rayon`'s fork and join.
const POINTS_PER_SIDE: usize = 13;

/// The sweep. The machine this was written against is a twelve-core AMD Ryzen 9
/// 3900X, and twelve is where the 0.8 of linear has to hold: the other twelve
/// hardware threads share those same twelve cores' arithmetic units, so
/// counting them would be measuring how well the kernel scales onto cores that
/// are not there.
const THREAD_COUNTS: [usize; 5] = [1, 2, 4, 8, 12];

fn classifying_a_grid_of_points(criterion: &mut Criterion) {
    let (model, plate) = plate_with_holes();
    let solid = model.solid(plate).expect("this model made the key");
    let points = grid_over(solid, POINTS_PER_SIDE);

    report_what_the_grid_hit_and_whether_the_threads_agree(&points, solid);

    let mut group = criterion.benchmark_group("classifying_a_grid_of_points");
    group.throughput(Throughput::Elements(points.len() as u64));
    group.sample_size(20);

    group.bench_function("one_thread_without_rayon", |bencher| {
        bencher.iter(|| classify_one_at_a_time(solid, &points));
    });

    for threads in THREAD_COUNTS {
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("a thread pool of a fixed size");
        group.bench_function(BenchmarkId::new("threads", threads), |bencher| {
            bencher.iter(|| pool.install(|| classify_across_threads(solid, &points)));
        });
    }

    group.finish();
}

/// Print the two things a timing report cannot show: that the grid really does
/// straddle the plate's skin, and that the parallel answers match the serial
/// ones bit for bit.
fn report_what_the_grid_hit_and_whether_the_threads_agree(
    points: &[rcad::Point],
    solid: rcad::SolidView<'_>,
) {
    let serial = classify_one_at_a_time(solid, points);
    let [inside, on_boundary, outside, undecided] = tally(&serial);
    println!(
        "grid of {} points: {inside} inside, {on_boundary} on the boundary, \
         {outside} outside, {undecided} undecided",
        points.len()
    );

    let most = *THREAD_COUNTS.iter().max().expect("the sweep is not empty");
    let pool = ThreadPoolBuilder::new()
        .num_threads(most)
        .build()
        .expect("a thread pool of a fixed size");
    let parallel = pool.install(|| classify_across_threads(solid, points));

    match first_difference(&serial, &parallel) {
        None => println!("bit-identical to the serial run on {most} threads"),
        Some(index) => panic!(
            "point {index} classified differently on {most} threads: \
             serial {:?}, parallel {:?}",
            serial[index], parallel[index]
        ),
    }
}

criterion_group!(benches, classifying_a_grid_of_points);
criterion_main!(benches);
