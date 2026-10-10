//! A part turned a step at a time — a degree, then another, all the way round —
//! is placed by composing each step onto the turn it already has, which is what
//! `Model::place_solid` does with every call: `movement.compose(&used.loc)`.
//!
//! Composing two quaternions rounds, and a quaternion whose length is not one
//! stretches what it turns. **Nothing used to scale the product back**, so the
//! turn's length walked off one with every step, a plane's normal turned by it
//! walked past [`Plane::NORMAL_LENGTH_ROUNDOFF`], and
//! [`Plane::from_point_and_normal_as_written`] — what the STEP reader builds
//! every `PLANE` with — scaled it on the way back in. The plane came back an
//! ulp moved from the one its file wrote, as every turned plane did before
//! `docs/decisions.md` §355.
//!
//! The oracle is the bits the plane was written with. No tolerance appears in
//! this file.

use rcad_geom::{Axis, Isometry, Plane, Point, Vector};

/// Three slanted axes, three step sizes, 360 steps each: 3240 placements, every
/// one a plane written down and read back.
#[test]
fn a_plane_turned_a_step_at_a_time_reads_back_with_the_normal_it_was_written_with() {
    let plane = Plane::from_point_and_normal(Point::new(1.0, 2.0, 3.0), Vector::new(1.0, 2.0, 3.0))
        .expect("a finite point and a normal with a length");
    let mut written = 0;
    let mut moved_on_read = 0;
    let mut fewest_steps_that_moved_one = None;
    for direction in [
        Vector::new(1.0, 2.0, 3.0),
        Vector::new(-2.0, 1.0, 5.0),
        Vector::new(3.0, -1.0, 1.0),
    ] {
        let axis =
            Axis::new(Point::new(1.0, -2.0, 0.5), direction).expect("a finite axis with a length");
        for step_in_degrees in [1.0_f64, 5.0, 7.5] {
            let step = Isometry::rotation(&axis, step_in_degrees.to_radians())
                .expect("a finite angle about a finite axis");
            let mut placed = Isometry::identity();
            for steps in 1..=360 {
                placed = step.compose(&placed);
                let normal = plane.transformed(&placed).normal();
                let read = Plane::from_point_and_normal_as_written(Point::origin(), normal)
                    .expect("a turned unit normal is a normal");
                written += 1;
                let same_bits = read
                    .normal()
                    .components()
                    .into_iter()
                    .zip(normal.components())
                    .all(|(read, wrote)| read.to_bits() == wrote.to_bits());
                if !same_bits {
                    moved_on_read += 1;
                    fewest_steps_that_moved_one = Some(
                        fewest_steps_that_moved_one.map_or(steps, |fewest: i32| fewest.min(steps)),
                    );
                }
            }
        }
    }
    println!(
        "{moved_on_read} of {written} planes came back with a normal they were not written with, \
         the first after {fewest_steps_that_moved_one:?} steps"
    );
    assert_eq!(
        moved_on_read, 0,
        "{moved_on_read} of {written} planes turned a step at a time came back moved"
    );
}
