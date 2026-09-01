//! §24.18's criterion 11: what one certified point along a marched crossing
//! costs, and how much of that is the interval divide.
//!
//! # Human-read, and there is no number it has to beat
//!
//! §24.16 is explicit, and §12.5 had to write the same caveat once already:
//! *"A figure is not a target. §1.4 and §11's bump-arena row both say an
//! optimisation with no number it must beat is the mistake, and the number the
//! marcher needs cannot be known until the marcher exists."* So **nothing here
//! asserts anything**. It prints, a person reads it, and §29 records what it
//! said on the day.
//!
//! # Which divide, and why it is a new one
//!
//! §24.12 predicted the money would go to `SideOfSurface` for a ball and a pipe
//! and to the interval divide beneath it. §27.4 read the code and found half of
//! that misaimed: neither sign predicate divides at any level, and the divide
//! §16.8 measured sits under the B-spline evaluators M3 does not call. So the
//! divide to measure is **the marcher's own**, and this is the first time
//! anybody could.
//!
//! It is the one in the certified deviation. Every settled point is measured
//! against both surfaces in interval arithmetic, and each of those measurements
//! divides: a plane's own normal by its length, a ball's outward direction by
//! its reach, a pipe's by its own, and the sum of the two distances by the sine
//! of the angle between the surfaces. **Nine divides for a ball against a pipe,
//! per certified point** — three for the ball, five for the pipe, one to put
//! them together.
//!
//! # Why the divide is written out again here
//!
//! `quotient_enclosing` is `pub(crate)`, so a benchmark cannot call it. What is
//! below is the same expression: four corner divisions, each widened by one
//! `f64` step at each end unless it came out exactly. Measuring the operation
//! rather than the call is what this row needs — the count per point comes from
//! the marcher's source and is written above.

use criterion::{Criterion, criterion_group, criterion_main};
use rcad_geom::{
    Axis, Cylinder, Deflection, Plane, Point, Sphere, Surface, SurfaceMeet, Vector, intersect, walk,
};
use rcad_predicates::{Interval, Policy, ProvenanceLog};
use std::hint::black_box;

/// How fine every walk here is asked for, as a share of the shapes' size.
const HOW_FINE: f64 = 1e-3;

/// A ball crossing a pipe off its axis, which is a genuine degree-four curve and
/// so goes to the marcher.
fn a_quartic() -> [Surface; 2] {
    [
        Surface::Sphere(Sphere::new(Point::new(2.0, 0.0, 0.0), 4.0).expect("a radius")),
        Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius")),
    ]
}

/// A plane cutting a ball, which is a conic and so is parametrized.
fn a_conic() -> [Surface; 2] {
    [
        Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 1.0), Vector::new(0.0, 0.0, 1.0))
                .expect("a unit normal"),
        ),
        Surface::Sphere(Sphere::new(Point::origin(), 4.0).expect("a radius")),
    ]
}

/// Walk the first run of a crossing, and hand back how many points it took.
fn walked(pair: &[Surface; 2]) -> usize {
    let Ok(SurfaceMeet::Curves(runs)) = intersect(
        &pair[0],
        &pair[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) else {
        panic!("this pair crosses in a curve");
    };
    let deflection = Deflection::of(4.0 * HOW_FINE).expect("a positive length");
    walk(
        &runs[0],
        [&pair[0], &pair[1]],
        deflection,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a shape this size is walked without complaint")
    .points()
    .len()
}

fn what_it_costs(bencher: &mut Criterion) {
    let quartic = a_quartic();
    let conic = a_conic();
    println!(
        "points per run: {} marched, {} parametrized. Divide the two run timings below by \
         these to get a cost per point, and multiply the divide's timing by nine to get what \
         one certified point spends inside it.",
        walked(&quartic),
        walked(&conic)
    );

    let mut group = bencher.benchmark_group("what a point along a crossing costs");
    group.bench_function("one marched run, start to finish", |run| {
        run.iter(|| black_box(walked(black_box(&quartic))));
    });
    group.bench_function("one parametrized run, start to finish", |run| {
        run.iter(|| black_box(walked(black_box(&conic))));
    });
    group.bench_function("naming the same crossing and drawing none of it", |run| {
        run.iter(|| {
            black_box(intersect(
                black_box(&quartic[0]),
                black_box(&quartic[1]),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            ))
            .ok();
        });
    });
    group.bench_function("one interval divide", |run| {
        let over = Interval::new(1.0, 1.0 + f64::EPSILON).expect("a range");
        let under = Interval::new(3.0, 3.0 + f64::EPSILON).expect("a range");
        run.iter(|| black_box(a_quotient(black_box(over), black_box(under))));
    });
    group.finish();
}

/// `over / under` as an enclosure, written the way `quotient_enclosing` writes
/// it, because that one is `pub(crate)` and a benchmark cannot reach it.
///
/// # Panics
///
/// If `under` touches zero, which the marcher's own copy turns into a refusal
/// and this one has no reason to.
fn a_quotient(over: Interval, under: Interval) -> Interval {
    assert!(!under.contains_zero(), "nothing here divides by zero");
    let mut lowest = f64::INFINITY;
    let mut highest = f64::NEG_INFINITY;
    for top in [over.lo(), over.hi()] {
        for bottom in [under.lo(), under.hi()] {
            let corner = top / bottom;
            lowest = lowest.min(corner.next_down());
            highest = highest.max(corner.next_up());
        }
    }
    Interval::new(lowest, highest).expect("a quotient runs from its lowest corner to its highest")
}

criterion_group!(benches, what_it_costs);
criterion_main!(benches);
