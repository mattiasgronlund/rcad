//! Pins the plain `f64` value `docs/decisions.md` §111.1 writes down as
//! `(∇S × ∇T) · (ẑ × ∇S) = 4.206230998539495e-30` for a dome sitting on a
//! pipe of the same radius, both from `kicad/LED_D5.0mm-3.step`.
//!
//! §111.1 names four points and one direction as the entire input, so no
//! STEP corpus is needed to recompute it — they are the literals below.
//!
//! This is **not** §111.2's boxed enclosure `[4.2062309985394852e-30,
//! 4.2062309985395062e-30]`. That range comes from re-running the same five
//! lines in interval arithmetic with, in §111.2's words, "a `nextafter` of
//! slack per operation" — a wider, ad hoc outer bound that was never checked
//! into the tree, so there is nothing here to call and no way to reproduce
//! its exact edges bit for bit. The plain `f64` value pinned below is inside
//! that range (`...4852 < ...4950 < ...5062`), which is all §111.2 claims of
//! it.

use rcad_geom::{Axis, Cylinder, Point, Sphere, Vector};

#[test]
fn the_dome_and_pipe_from_the_led_file_turn_by_a_real_amount() {
    let dome = Sphere::new(Point::new(2.54, -5.051_514_762_044e-15, 9.1), 2.5).expect("the dome");
    let pipe = Cylinder::new(
        Axis::new(
            Point::new(2.54, -3.030_908_857_227e-15, 9.1),
            Vector::new(-0.0, 3.330_669_073_875e-16, -1.0),
        )
        .expect("the pipe's axis"),
        2.5,
    )
    .expect("the pipe");
    let corner = Point::new(5.04, -5.051_514_762_044e-15, 9.1);

    let dome_gradient = corner - dome.centre();
    let out = corner - pipe.axis().origin();
    let along = pipe.axis().direction();
    let pipe_gradient = out * along.dot(along) - along * along.dot(out);

    let crossing = dome_gradient.cross(pipe_gradient);
    assert_eq!(
        crossing.components(),
        [0.0, 1.682_492_399_415_798e-30, -5.051_514_762_042_499_6e-15],
        "§111.1's own cross product, out of the file's own numbers"
    );

    let longitude = Vector::new(0.0, 0.0, 1.0).cross(dome_gradient);
    let turning = crossing.dot(longitude);

    assert_eq!(
        turning, 4.206_230_998_539_495e-30,
        "§111.1's own figure for (∇S × ∇T) · (ẑ × ∇S); a different value \
         means that citation does not reproduce"
    );
}
