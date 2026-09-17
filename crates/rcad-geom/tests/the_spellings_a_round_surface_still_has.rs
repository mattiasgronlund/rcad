//! Every way of writing one round surface twice, and which ones are still open.
//!
//! §51.2 item 2 fixed one of these — a signed zero — in `Plane`, `Axis` and
//! `Sphere`, and §51.7 books a second: an axis written either way round. Two
//! more were named while measuring the second (sliding a pipe's origin along
//! its own axis, and whatever a fifth might be). This file is the search for
//! the fifth, written as the closures it found rather than as a paragraph
//! saying there is none (§16.10).
//!
//! **The count.** A surface's stored numbers minus its real degrees of freedom
//! is how many ways there are to write it twice:
//!
//! | surface | stored | degrees of freedom | what is left over |
//! |---|---|---|---|
//! | plane | 4 | 3 | the normal's length, and its sign |
//! | ball | 4 | 4 | nothing |
//! | pipe | 7 | 5 | the axis direction's length, its sign, and where along the axis the origin sits |
//! | cone | 7 | 6 | the axis direction's length, and its sign |
//! | torus | 8 | 7 | the axis direction's length, and its sign |
//!
//! The difference of the two columns counts the *continuous* leftovers — one
//! for every kind but the pipe, which has two — and the sign rides on top of
//! them as the one discrete way round.
//!
//! Every "length" row is folded at construction and every test below is one of
//! those foldings. What is left over after them is the sign — §51.7's booked
//! row — and the pipe's sliding origin, which
//! `how_far_two_spellings_of_one_pipe_move_a_corner.rs` measures as costing no
//! arithmetic at all. **There is no fifth.**
//!
//! Two things the table says that are worth spelling out. A ball has no spare
//! number, which is why §51.2's signed-zero fix was the whole of its problem. And
//! a plane's leftover is its *normal*, never its point: `from_point_and_normal`
//! turns the point into one offset number, so two points of one plane either
//! round to the same offset — one spelling — or to different ones, which are two
//! planes a rounding apart and not two names for one. A pipe keeps a whole
//! three-number origin of which only the part across the axis matters, and that
//! is exactly why the sliding freedom exists there and nowhere else.

use rcad_geom::{Axis, Cone, Cylinder, Error, Plane, Point, Sphere, Torus, Vector};

fn along(x: f64, y: f64, z: f64) -> Vector {
    Vector::new(x, y, z)
}

/// A direction written at another scale is the same axis, bit for bit.
///
/// `Vector::scaled_to_length_one_from_anywhere` divides by the largest
/// component before it divides by the length, so two proportional directions
/// reach the division as the same three numbers. §22.6 records the same fix for
/// a plane's coefficients; this is the axis half of it, which nothing had
/// asked.
#[test]
fn an_axis_written_at_any_scale_is_the_same_axis() {
    let through = Point::new(1.0, -2.0, 0.5);
    for direction in [
        along(1.0, 2.0, 3.0),
        along(-4.0, 0.0, 7.0),
        along(0.0, 0.0, 5.0),
    ] {
        let plain = Axis::new(through, direction).expect("a direction with a length");
        for scale in [2.0, 3.0, 7.0, 0.5, 1e9, 1e-9] {
            let scaled = Axis::new(
                through,
                along(
                    direction.components()[0] * scale,
                    direction.components()[1] * scale,
                    direction.components()[2] * scale,
                ),
            )
            .expect("a scaled direction still has a length");
            assert_eq!(
                scaled.direction().components(),
                plain.direction().components(),
                "writing {direction:?} {scale} times as long gave a different axis"
            );
        }
    }
}

/// The same for a plane, which is where the fold was first written (§22.6).
#[test]
fn a_planes_coefficients_written_at_any_scale_are_the_same_plane() {
    let written = [1.0, 2.0, 3.0, -4.0];
    let plain = Plane::from_coefficients(written).expect("a normal with a length");
    for scale in [2.0, 3.0, 7.0, 0.5, 1e9, 1e-9] {
        let scaled = Plane::from_coefficients(written.map(|number| number * scale))
            .expect("scaling all four keeps the plane");
        assert_eq!(
            scaled.coefficients(),
            plain.coefficients(),
            "writing the plane's coefficients {scale} times as big gave a different plane"
        );
    }
}

/// **A plane handed back its own coefficients is a different plane**, and
/// [`Plane::moved_to`] is how to scale one without that.
///
/// The fold above is about *proportional* coefficients, and a stored normal is
/// not proportional to anything: it is the rounded normalizing of whatever built
/// it. Reducing and normalizing it a second time therefore moves it, the same
/// second scaling [`rcad_geom::Axis::moved_to`] exists to avoid for a direction.
///
/// This is the demonstration behind the number in [`Plane::moved_to`]'s doc
/// comment, and behind the plane arm of the fuzzing harness's `scaled`. The
/// night of 2026-09-16 and the night of 2026-09-17 both crashed on it: a cone of
/// slope `7.8e28` against a plane square to its axis is one circle, and the
/// plane rebuilt for the scaled run leaned `2.8e-17` off the axis — far more
/// than the `1/slope = 1.3e-29` that the cone's own side leans — so the scaled
/// pair really was a hyperbola in two branches and the kernel was right about
/// both.
#[test]
fn rebuilding_a_plane_from_its_own_coefficients_tilts_its_normal() {
    let mut state = 0x243f_6a88_85a3_08d3_u64;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        f64::from(u32::try_from(state >> 33).expect("33 bits shifted off a u64 leave 31"))
            / f64::from(u32::MAX >> 1)
            - 1.0
    };

    let mut tilted = 0;
    let mut moved_by_moved_to = 0;
    let draws = 100_000;
    for _ in 0..draws {
        let Ok(plane) =
            Plane::from_point_and_normal(Point::new(1.0, -2.0, 0.5), along(next(), next(), next()))
        else {
            continue;
        };
        let rebuilt =
            Plane::from_coefficients(plane.coefficients()).expect("a unit normal has a length");
        if rebuilt.normal().components() != plane.normal().components() {
            tilted += 1;
        }
        let slid = plane
            .moved_to(plane.offset() * 0.5)
            .expect("a finite offset");
        if slid.normal().components() != plane.normal().components() {
            moved_by_moved_to += 1;
        }
    }

    println!(
        "{tilted} of {draws} plane normals moved when the plane was rebuilt from its own coefficients"
    );
    assert_eq!(
        moved_by_moved_to, 0,
        "moved_to is supposed to hand the stored normal straight back"
    );
    assert!(
        tilted > draws / 5,
        "only {tilted} of {draws} rebuilt normals moved, so the number in Plane::moved_to's doc \
         comment no longer describes this arithmetic"
    );
}

/// A negative radius names the same points as a positive one, and every round
/// surface refuses it rather than folding it.
///
/// The sign is squared out of all four sign expressions, so `-r` and `r` would
/// be two names for one surface. Refusing at the door is the cheaper fix and it
/// is the one taken.
#[test]
fn no_round_surface_accepts_a_radius_written_negative() {
    let axis = Axis::z();
    assert!(matches!(
        Cylinder::new(axis, -2.0),
        Err(Error::RadiusNotPositive { .. })
    ));
    assert!(matches!(
        Sphere::new(Point::origin(), -2.0),
        Err(Error::RadiusNotPositive { .. })
    ));
    assert!(matches!(
        Torus::new(axis, -4.0, 1.0),
        Err(Error::RadiusNotPositive { .. })
    ));
    assert!(matches!(
        Torus::new(axis, 4.0, -1.0),
        Err(Error::RadiusNotPositive { .. })
    ));
}

/// A cone's slope is squared too, and a negative one is refused for the same
/// reason. [`Cone::new`]'s own doc comment says so; this is the test it never
/// had.
#[test]
fn a_cone_does_not_accept_a_slope_written_negative() {
    assert!(matches!(
        Cone::new(Axis::z(), -0.5),
        Err(Error::SlopeNotPositive { .. })
    ));
}

/// **The one that is left over and is nobody's lane.** A pipe's origin written
/// at another point of its own axis is still two spellings.
///
/// Here so that folding it breaks this test rather than passing unnoticed.
/// §51.7's other row — the axis written either way round — is deliberately not
/// asserted on: it is being closed as this is written, and a test that says it
/// is still open would fail the moment it stops being.
#[test]
fn a_pipes_origin_written_further_along_its_axis_is_still_a_second_spelling() {
    let pipe = Cylinder::new(Axis::z(), 2.0).expect("a positive radius");
    let slid = Cylinder::new(
        Axis::new(Point::new(0.0, 0.0, 2.0), along(0.0, 0.0, 1.0)).expect("a direction"),
        2.0,
    )
    .expect("a positive radius");
    assert_ne!(
        pipe.axis().origin().coordinates(),
        slid.axis().origin().coordinates(),
        "sliding the origin stopped changing the numbers, so the row is closed"
    );
}
