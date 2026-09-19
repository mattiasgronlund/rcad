//! §142.3's three unmeasured sites, measured: a drawable pair for each, and the
//! enclosure that reaches zero when it should not.
//!
//! §142.2's family is the functions that decide **which of two already-named
//! pieces a point falls into**. None of them calls `ask()`, so there is no exact
//! level underneath and a straddling enclosure is a refusal with nothing to
//! escalate to. §140 was the first measured casualty; the sweep named four more
//! sites and only one of those had a number behind it.
//!
//! Each test here builds an ordinary pair of surfaces, asserts that
//! [`intersect`] refuses it with
//! [`Error::CannotNameWhichPieceOfACrossingThisIs`], and prints the enclosure
//! that did it. **All three are §16.10 gaps and not limits**: in each case the
//! answer exists and the test says in what sense, so the refusal is arithmetic
//! that could not reach an answer rather than an answer that is not there.
//!
//! # Why the printed enclosure is the one the kernel computed
//!
//! `intersect.rs` is private, so the numbers below are rebuilt out of
//! [`Interval`]'s public arithmetic rather than read out of it. Interval
//! multiplication and addition round outwards exactly as they do inside, so the
//! rebuilt value is the same one — except where a square root is involved, and
//! there the test never needs the root: it shows the value *under* the root
//! already straddles zero, which is what makes `root_enclosing` clamp its low
//! end to zero and hand back an enclosure that reaches nothing.

#![forbid(unsafe_code)]

use rcad_geom::{Axis, Cylinder, Error, Plane, Point, Sphere, Surface, Vector, intersect};
use rcad_predicates::{Interval, Policy, ProvenanceLog};

/// How far along its own direction the pipe's axis origin sits, for site one.
///
/// Anywhere the two products `n·o` has to add up cancel is enough; this is one
/// ten-billionth of nothing special.
const ALONG_THE_PLANE: f64 = 1.0e10;

/// Site one, `intersect.rs:440`: a pipe lying exactly in a plane loses both of
/// its lines when the place it lies at is far enough along the plane.
///
/// The plane is `x + y = 0` and the pipe's axis is the line through
/// `(L, −L, 0)` pointing along `(1, −1, 0)`. That axis is **exactly** in the
/// plane: `n·o` is `u·L + u·(−L)` for the one stored `u` both components round
/// to, so the two products are the same number with a sign moved and the true
/// sum is zero. The plane therefore cuts the pipe in two straight lines, `r`
/// either side of the axis, and there is nothing marginal about the answer.
///
/// What is marginal is `n·o` **as an enclosure**. Each product is enclosed to
/// its own last bit and the two do not cancel bit for bit, so the reading comes
/// out as `±ulp(u·L)` — here about `1.6e-6`, which is a hundred times the
/// pipe's own width. `out = √(r² − in_front²)` then has nothing left of `r²`
/// and reaches down to zero, and the two lines are no longer provably either
/// side of the picking plane.
#[test]
fn a_pipe_lying_in_a_plane_far_along_it_loses_both_of_its_lines() {
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 1.0, 0.0))
        .expect("a plane through the origin at forty-five degrees");
    let normal = flat.normal().components();
    let axis = Axis::new(
        Point::new(ALONG_THE_PLANE, -ALONG_THE_PLANE, 0.0),
        Vector::new(1.0, -1.0, 0.0),
    )
    .expect("an axis at forty-five degrees the other way");
    let along = axis.direction().components();

    assert_eq!(
        normal[0], -along[1],
        "both directions must round to the same stored component, or the axis is \
         not exactly in the plane"
    );

    let radius = 1.0e-8;
    let pipe = Cylinder::new(axis, radius).expect("a hair-thin pipe");

    let reading =
        dot_enclosing(normal, axis.origin().coordinates()) - Interval::exactly(flat.offset());
    let skin = Interval::exactly(radius);
    let under_the_root = skin * skin - reading * reading;
    println!("site 1: n·o − d encloses {reading:?}");
    println!("site 1: r² − in_front² encloses {under_the_root:?}");

    assert!(
        reading.contains_zero() && reading.width() > radius,
        "the reading has to be wider than the pipe for this to be site 440's shape"
    );
    assert!(
        under_the_root.lo() < 0.0,
        "the value under the root must straddle zero for `out` to reach zero"
    );

    let met = intersect(
        &Surface::Plane(flat),
        &Surface::Cylinder(pipe),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(met, Err(Error::CannotNameWhichPieceOfACrossingThisIs)),
        "site 440 was expected to refuse to tell the two lines apart, and said {met:?}"
    );
}

/// The same pipe at the origin, which is the control: nothing about the shape
/// refuses, only about where it was written down.
#[test]
fn the_same_pipe_lying_in_the_same_plane_at_the_origin_keeps_its_two_lines() {
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 1.0, 0.0))
        .expect("a plane through the origin at forty-five degrees");
    let axis = Axis::new(Point::origin(), Vector::new(1.0, -1.0, 0.0)).expect("an axis");
    let pipe = Cylinder::new(axis, 1.0e-8).expect("a hair-thin pipe");

    let met = intersect(
        &Surface::Plane(flat),
        &Surface::Cylinder(pipe),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(met, Ok(rcad_geom::SurfaceMeet::Curves(ref runs)) if runs.len() == 2),
        "the same pair at the origin should be named as two lines, and was {met:?}"
    );
}

/// How far apart the two axes are in site two, exactly.
const AXES_APART: f64 = 1.0;

/// The thin pipe's width in site two.
const THIN: f64 = 3.0e-17;

/// Site two, `intersect.rs:1198`: a fat pipe only just wide enough to swallow a
/// thin one's whole cross-section loses both of its loops.
///
/// The thin pipe runs along `x` through the origin, the fat one along `y`
/// through `(0, 0, 1)`, so the two axes are exactly one apart — the cross
/// product of the two stored directions is `(0, 0, 1)` on the nose and the
/// division by `√K` has nothing to spoil. Two closed loops are what a crossing
/// is when `r_fat > δ + r_thin`, and that comparison is asked through `ask` with
/// an exact level under it, so reaching
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] at all **proves the exact
/// level already said two loops**. The answer exists; `clear` cannot reach it.
///
/// `clear = √(r_fat² − (δ + r_thin)²)` is where it is lost. With the thin pipe
/// far narrower than one bit of `δ`, `δ + r_thin` is not a number and the two
/// squares are within a rounding of each other, so the value under the root
/// straddles zero. The test walks the first few `f64`s above `δ` and reports how
/// many of them refuse.
#[test]
fn a_fat_pipe_only_just_wide_enough_to_swallow_a_thin_one_loses_both_of_its_loops() {
    let thin = Cylinder::new(Axis::x(), THIN).expect("a pipe a rounding wide");
    let mut refused = Vec::new();
    let mut named = Vec::new();

    let mut fat_radius = AXES_APART;
    for _ in 0..12 {
        fat_radius = next_f64_up(fat_radius);
        let fat = Cylinder::new(
            Axis::new(Point::new(0.0, 0.0, AXES_APART), Vector::new(0.0, 1.0, 0.0))
                .expect("an axis along y"),
            fat_radius,
        )
        .expect("a fat pipe");
        let met = intersect(
            &Surface::Cylinder(thin),
            &Surface::Cylinder(fat),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let skin = Interval::exactly(fat_radius);
        let reach = Interval::exactly(AXES_APART) + Interval::exactly(THIN);
        let under_the_root = skin * skin - reach * reach;
        match met {
            Err(Error::CannotNameWhichPieceOfACrossingThisIs) => {
                assert!(
                    under_the_root.lo() <= 0.0,
                    "a refusal here has to be `clear` reaching zero, and at {fat_radius:e} \
                     the value under its root was {under_the_root:?}"
                );
                refused.push(fat_radius);
            }
            Ok(_) => {
                assert!(
                    under_the_root.lo() > 0.0,
                    "every width `clear` cannot certify has to refuse, and {fat_radius:e} \
                     did not: {under_the_root:?}"
                );
                named.push(fat_radius);
            }
            other => panic!("neither named nor site 1198's refusal: {other:?}"),
        }
    }

    let reach = Interval::exactly(AXES_APART) + Interval::exactly(THIN);
    let widest = *refused.last().expect("at least one width must refuse");
    let skin = Interval::exactly(widest);
    println!("site 2: δ + r_thin encloses {reach:?}");
    println!(
        "site 2: r_fat² − reach² encloses {:?} at the widest refusing r_fat {widest:e}",
        skin * skin - reach * reach
    );
    println!(
        "site 2: {} of the first 12 f64 widths above δ refuse, the first named one is {:e}",
        refused.len(),
        named.first().copied().unwrap_or(f64::NAN)
    );

    assert!(
        !refused.is_empty(),
        "site 1198 was expected to refuse a fat pipe a rounding wider than δ + r_thin"
    );
    assert!(
        !named.is_empty(),
        "a fat pipe a few roundings wider must still be named, or this is measuring \
         something other than the width of the refusing band"
    );
}

/// How far along the axis the ball's centre sits in site three.
const ALONG_THE_AXIS: f64 = 1.0e10;

/// The pipe's width in site three.
const PIPE: f64 = 1.0e-3;

/// The ball's radius in site three, a twentieth of a percent wider than the
/// pipe.
const BALL: f64 = 1.0005e-3;

/// Site three, `intersect.rs:677-689`: a ball sitting on a pipe's axis a long
/// way along it loses both of its loops.
///
/// The pipe runs along `(1, 1, 0)` through the origin and the ball's centre is
/// at `(M, M, 0)`, which is **exactly** on that axis — the cross product
/// `(c − o) × w` is `(0, 0, M·u − M·u)`, zero as real arithmetic, and the
/// kernel's own on-the-axis question says so through its exact level, because
/// that is the only way this pair reaches the ball-centred-on-the-axis row at
/// all.
///
/// So the crossing is two circles at `±√(R² − r²)` from the centre, each the
/// pipe's own width, and `R > r` decides it. What refuses is
/// `D₋ = W(R² − r²) − A − 2r√(AW)`, where `A = ‖(c − o) × w‖²` is **zero and
/// enclosed as `±ulp(M·u)²`**. That last number is `9.1e-13` here against a
/// `W(R² − r²)` of `1.0e-9`, so the `2r√(AW)` term alone is nearly twice the
/// whole of `D₋` and the enclosure reaches below zero.
///
/// **`reach` starting at nothing is on its own enough to refuse.** The margin
/// the caller then tests is `(n·ŵ)·reach − ‖n̂ × ŵ‖·R − offset_slop`, and
/// neither of the last two terms is ever below zero — so a `reach` whose low
/// end is zero leaves a margin whose low end cannot be above zero either. The
/// half-space's own offset rounding grows with the centre's distance as well,
/// and is not zero here; it does not have to be for this refusal to happen.
#[test]
fn a_ball_on_a_pipes_axis_far_along_it_loses_both_of_its_loops() {
    let axis = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0)).expect("an axis");
    let along = axis.direction().components();
    let centre = Point::new(ALONG_THE_AXIS, ALONG_THE_AXIS, 0.0);
    let pipe = Cylinder::new(axis, PIPE).expect("a millimetre-wide pipe");
    let ball = Sphere::new(centre, BALL).expect("a ball a shade wider");

    let off_the_axis = squared_length_enclosing(cross_enclosing(
        gap_enclosing(axis.origin().coordinates(), centre.coordinates()),
        exactly(along),
    ));
    let squared_direction = dot_enclosing(along, along);
    let room = squared_direction
        * (Interval::exactly(BALL) * Interval::exactly(BALL)
            - Interval::exactly(PIPE) * Interval::exactly(PIPE));
    let from_the_middle = Interval::exactly(2.0)
        * Interval::exactly(PIPE)
        * root_no_wider_than_the_kernels(off_the_axis * squared_direction);
    let d_minus = room - off_the_axis - from_the_middle;
    println!("site 3: A = ‖(c − o) × w‖² encloses {off_the_axis:?}");
    println!("site 3: W(R² − r²) encloses {room:?}");
    println!("site 3: 2r√(AW) encloses {from_the_middle:?}");
    println!("site 3: D₋ = W(R² − r²) − A − 2r√(AW) encloses {d_minus:?}");

    assert!(
        d_minus.lo() < 0.0,
        "D₋ must reach below zero for `reach` to come back as an enclosure that \
         starts at nothing: {d_minus:?}"
    );

    let met = intersect(
        &Surface::Sphere(ball),
        &Surface::Cylinder(pipe),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(met, Err(Error::CannotNameWhichPieceOfACrossingThisIs)),
        "site 677 was expected to refuse to tell the two loops apart, and said {met:?}"
    );
}

/// The same ball and pipe at the origin, which is the control.
#[test]
fn the_same_ball_on_the_same_pipes_axis_at_the_origin_keeps_both_of_its_loops() {
    let axis = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0)).expect("an axis");
    let pipe = Cylinder::new(axis, PIPE).expect("a millimetre-wide pipe");
    let ball = Sphere::new(Point::origin(), BALL).expect("a ball a shade wider");

    let met = intersect(
        &Surface::Sphere(ball),
        &Surface::Cylinder(pipe),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(met, Ok(rcad_geom::SurfaceMeet::Curves(ref runs)) if runs.len() == 2),
        "the same pair at the origin should be named as two loops, and was {met:?}"
    );
}

/// The next `f64` above `value`, for walking the band a refusal lives in.
fn next_f64_up(value: f64) -> f64 {
    f64::from_bits(value.to_bits() + 1)
}

/// `u · v` over two stored triples, enclosed the way `enclosure.rs` encloses it.
fn dot_enclosing(u: [f64; 3], v: [f64; 3]) -> Interval {
    (0..3).fold(Interval::exactly(0.0), |so_far, axis| {
        so_far + Interval::exactly(u[axis]) * Interval::exactly(v[axis])
    })
}

/// The same triple as three zero-width enclosures.
fn exactly(u: [f64; 3]) -> [Interval; 3] {
    u.map(Interval::exactly)
}

/// `there − here`, componentwise, as enclosures.
fn gap_enclosing(here: [f64; 3], there: [f64; 3]) -> [Interval; 3] {
    core::array::from_fn(|axis| Interval::exactly(there[axis]) - Interval::exactly(here[axis]))
}

/// `u × v` as enclosures.
fn cross_enclosing(u: [Interval; 3], v: [Interval; 3]) -> [Interval; 3] {
    core::array::from_fn(|axis| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        u[next] * v[after] - u[after] * v[next]
    })
}

/// `√value`, clamped at zero, and **never wider than `root_enclosing`'s answer**.
///
/// `enclosure.rs` rounds both ends outwards; this leaves them where `sqrt` puts
/// them. So a `D₋` built on this one starts no lower than the kernel's does, and
/// showing that this one reaches below zero shows the kernel's does too.
fn root_no_wider_than_the_kernels(value: Interval) -> Interval {
    Interval::new(value.lo().max(0.0).sqrt(), value.hi().sqrt())
        .expect("a root grows with what it is taken of")
}

/// `u · u` as an enclosure.
fn squared_length_enclosing(u: [Interval; 3]) -> Interval {
    (0..3).fold(Interval::exactly(0.0), |so_far, axis| {
        so_far + u[axis] * u[axis]
    })
}
