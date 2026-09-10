//! The second of the two pass criteria decisions.md §9.2 sets for M0: the
//! parallel classification must come back bit for bit identical to the serial
//! one.
//!
//! §9.2 calls this "the one that will actually find a bug", so it lives here as
//! a test the normal test run executes, not only as something a human sees when
//! they remember to run `cargo bench`. A pass criterion nobody runs is not a
//! pass criterion.

#![forbid(unsafe_code)]

mod plate_with_holes;

use rayon::ThreadPoolBuilder;
use rcad::{Policy, ProvenanceLog};

use plate_with_holes::{
    classify_across_threads, classify_one_at_a_time, first_difference, grid_over, plate_with_holes,
    tally,
};

/// Small enough to run on every test run, large enough that every worker gets
/// several hundred queries.
const POINTS_PER_SIDE: usize = 8;

/// More threads than a modest CI box has cores, on purpose. Oversubscribing
/// forces the workers to interleave, and interleaving is what would shake an
/// order-dependence loose.
const THREADS: usize = 8;

#[test]
fn classifying_a_grid_of_points_across_threads_gives_the_same_bits_as_one_thread() {
    let (model, plate) = plate_with_holes();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the plate is a valid solid");

    let solid = model.solid(plate).expect("this model made the key");
    let points = grid_over(solid, POINTS_PER_SIDE);

    let serial = classify_one_at_a_time(solid, &points);

    let pool = ThreadPoolBuilder::new()
        .num_threads(THREADS)
        .build()
        .expect("a thread pool of a fixed size");
    let parallel = pool.install(|| classify_across_threads(solid, &points));

    if let Some(index) = first_difference(&serial, &parallel) {
        panic!(
            "point {index} of {} classified differently on {THREADS} threads: \
             serial {:?}, parallel {:?}",
            points.len(),
            serial[index],
            parallel[index]
        );
    }
}

#[test]
fn the_grid_holds_points_inside_the_plate_outside_it_and_on_its_skin() {
    let (model, plate) = plate_with_holes();
    let solid = model.solid(plate).expect("this model made the key");
    let points = grid_over(solid, POINTS_PER_SIDE);

    let [inside, on_boundary, outside, undecided] = tally(&classify_one_at_a_time(solid, &points));

    assert!(inside > 0, "no sample point landed in the material");
    assert!(outside > 0, "no sample point landed in the air");
    assert!(on_boundary > 0, "no sample point landed on the skin");
    assert_eq!(
        inside + on_boundary + outside + undecided,
        points.len(),
        "the tally lost a point"
    );
}
