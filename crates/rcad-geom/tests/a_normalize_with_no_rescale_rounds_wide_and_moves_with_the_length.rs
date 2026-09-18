//! What the six bare `normalized_as` sites of `docs/decisions.md` §121.1 cost,
//! measured rather than argued.
//!
//! §121.1 names six normalizes that run no rescale first — `Plane::basis`,
//! `Cylinder::basis`, `Cone::basis`, `Torus::basis`, `Segment::new`, and the
//! plane a circle is written in — and says they carry two defects where
//! `Plane`'s constructors carry one: the answer is not the nearest `f64`, and
//! it is not a function of the ratios between the components alone. It leaves
//! open whether the second one reaches anything. These four tests answer that.
//!
//! **The correctly rounded answer is reached through `Axis::new`.**
//! `Vector::scaled_to_length_one_from_anywhere` is private to the crate, but
//! `Axis::new` is its only public caller and does nothing else to the direction
//! beyond folding `-0.0` to `0.0`, which `==` on `f64` does not see. So
//! `Axis::new(Point::origin(), v)?.direction()` is that function, and the bound
//! it publishes — every component above `2^-1012` is the nearest `f64` to its
//! exact ratio — is what makes it usable as a reference here. That bound is
//! gated by `a_unit_component_is_the_nearest_f64_to_the_exact_ratio` in
//! `point.rs`, so nothing is assumed: this file measures against a published,
//! tested bound instead of building a second exact reference beside it.
//!
//! # What came out, and where it disagrees with §121.1
//!
//! **The two defects do not come as a pair, and the six sites split three ways.**
//!
//! - A bare normalize misses the nearest `f64` on **8.94%** of components, and
//!   the constructor path §121.2 calls the same arithmetic misses on **16.33%**
//!   of the same components. §121.1 reads the bare one as the constructor path
//!   minus a step and so as at least as bad; on this count it is half as bad,
//!   because dividing by the largest component is itself a rounding and lands
//!   on every component that is not the largest.
//! - The ratio defect reaches **`Segment::new` and nothing else**. A caller
//!   chooses how far along a ray to put the far end, so the same run written at
//!   two lengths is a thing a caller writes; 38.1% of such pairs store two
//!   directions. Every other site takes its vector from what the surface stores,
//!   and every public route into that store scales to length one first.
//! - Dividing by the largest component **keeps the ratios** over the same
//!   210 816 pairs: 0 moved. So `Segment::new` gave up a guarantee the two
//!   `Plane` constructors were already keeping, which is the opposite way round
//!   from "the constructor path plus one more defect".
//!
//! **What correct rounding buys the five surface sites is smaller than it
//! looks.** A basis direction is meant to be of length one, and as written a
//! pipe's first one is not, 5 905 times in 20 000. Correctly rounded it is still
//! not, 4 686 times. The gap closes by a fifth and does not shut: no `f64`
//! triple is of length exactly one in general, so that part is a limit and not
//! a gap (§16.10).

use rcad_geom::{Axis, BSplineCurve, Cone, Cylinder, Plane, Point, Segment, Torus, Vector};

/// **A bare normalize misses the nearest `f64` on about one component in
/// eleven, and the constructor path misses on nearly twice as many.**
///
/// §121.1 says the bare normalize "rounds twice like the old path". It rounds
/// twice, but not like: dividing by the largest component is a third rounding,
/// and it lands on every component that is not the largest. So the site §121.1
/// calls the worse of the two is the better of the two on this count, and the
/// case for moving these six is the *other* defect and not this one.
///
/// The population is §120.6's: four hand-written directions, every integer
/// triple in `-6..=6`, nine hundred directions `(1, 2^-e, 0)` that claim a
/// binade each, and three thousand drawn from mantissas and exponents four
/// hundred binades either way. Both older arithmetics run beside the reference,
/// so this is a comparison and not a memory.
#[test]
fn how_often_a_bare_normalize_misses_the_nearest_f64() {
    let mut checked = 0usize;
    let mut bare_missed = 0usize;
    let mut reducing_first_missed = 0usize;
    let mut smallest_checked = f64::INFINITY;

    for direction in the_population() {
        let Ok(correct) = correctly_rounded(direction) else {
            continue;
        };
        let Ok(bare) = direction.normalized() else {
            continue;
        };
        let Ok(reduced) = divided_by_its_largest_component(direction).normalized() else {
            continue;
        };
        for axis in 0..3 {
            let answer = correct.components()[axis];
            if answer == 0.0 || answer.abs() < 2f64.powi(-1012) {
                continue;
            }
            checked += 1;
            smallest_checked = smallest_checked.min(answer.abs());
            if bare.components()[axis] != answer {
                bare_missed += 1;
            }
            if reduced.components()[axis] != answer {
                reducing_first_missed += 1;
            }
        }
    }

    println!(
        "{checked} components with a correctly rounded answer above 2^-1012: a bare normalize \
         missed it on {bare_missed} ({:.2}%), dividing by the largest component first on \
         {reducing_first_missed} ({:.2}%). The smallest answer checked was {smallest_checked:e}.",
        100.0 * bare_missed as f64 / checked as f64,
        100.0 * reducing_first_missed as f64 / checked as f64,
    );

    assert!(
        checked >= 10_000,
        "only {checked} components, which is not the measurement this claims to be"
    );
    assert!(
        smallest_checked < 2f64.powi(-800),
        "the population is meant to span binades the way §120.6's does, and the smallest answer \
         it reached was {smallest_checked:e}"
    );
    assert!(
        bare_missed * 20 > checked,
        "a bare normalize is meant to miss the nearest f64 on a large share of these, and here \
         it missed on only {bare_missed} of {checked}"
    );
    assert!(
        bare_missed <= reducing_first_missed,
        "dividing by the largest component first adds a rounding to every component that is not \
         the largest, so it cannot come out ahead of the bare normalize — yet here the bare one \
         missed {bare_missed} times against {reducing_first_missed}"
    );
}

/// **Two segments along one ray store two different directions**, which is the
/// second defect reaching the public API at `Segment::new` and nowhere else.
///
/// `Segment::new` is handed `end - start`, and a caller choosing how far along
/// the ray to put `end` chooses the length of that vector. So the same run,
/// written twice at two lengths, is a thing a caller writes rather than a thing
/// the arithmetic has to be provoked into.
///
/// The same pairs through `Axis::new` never differ. That is §22.6's property,
/// and it is what a rescale first buys.
#[test]
fn two_segments_along_one_ray_run_different_ways() {
    let mut drawn = 0usize;
    let mut segments_differed = 0usize;
    let mut first_witness = None;

    for x in -6i32..=6 {
        for y in -6i32..=6 {
            for z in -6i32..=6 {
                let short = Vector::new(f64::from(x), f64::from(y), f64::from(z));
                if short == Vector::zero() {
                    continue;
                }
                for scale in 2..=97 {
                    let long = short * f64::from(scale);
                    drawn += 1;
                    let here = Segment::new(Point::origin(), Point::origin() + short)
                        .expect("a lattice direction has a length");
                    let further = Segment::new(Point::origin(), Point::origin() + long)
                        .expect("a multiple of it has one too");
                    if here.direction() != further.direction() {
                        segments_differed += 1;
                        first_witness.get_or_insert((short, scale));
                    }
                    assert_eq!(
                        correctly_rounded(short).expect("a lattice direction has a length"),
                        correctly_rounded(long).expect("a multiple of it has one too"),
                        "an axis along {short:?} and one along the same ray {scale} times over \
                         are meant to be the same axis"
                    );
                }
            }
        }
    }

    println!(
        "{drawn} pairs of collinear segments: {segments_differed} of them ({:.1}%) stored a \
         different direction for one ray. The first was {first_witness:?}.",
        100.0 * segments_differed as f64 / drawn as f64
    );
    assert!(
        drawn >= 200_000,
        "only {drawn} pairs, which is not the measurement this claims to be"
    );
    assert!(
        segments_differed * 4 > drawn,
        "the point of this test is that a bare normalize moves with the length, and here it \
         moved for only {segments_differed} of {drawn} pairs"
    );
}

/// **One named pair, so the finding survives the rates.**
///
/// The run from the origin to `(6, 6, 5)` and the run from the origin to
/// `(18, 18, 15)` are the same ray. `Axis` gives them one direction; `Segment`
/// gives them two.
///
/// Three and not two: doubling a vector scales its length by exactly two and
/// the division by that length gives the same answer back, so every
/// power-of-two multiple of a ray is a pair this defect cannot be caught on.
/// That is the whole of what the rescale in
/// `Vector::scaled_to_length_one_from_anywhere` does, arrived at from the side
/// that fails.
#[test]
fn the_run_to_six_six_five_and_the_run_three_times_as_far_point_different_ways() {
    let short = Vector::new(6.0, 6.0, 5.0);
    let long = short * 3.0;

    let here = Segment::new(Point::origin(), Point::origin() + short).expect("has a length");
    let further = Segment::new(Point::origin(), Point::origin() + long).expect("has a length");
    println!(
        "{:?} against {:?}",
        here.direction().components(),
        further.direction().components()
    );

    assert_ne!(
        here.direction(),
        further.direction(),
        "this is the defect §121.1 names; if it has been fixed, this test is the one to delete"
    );
    assert_eq!(
        Segment::new(Point::origin(), Point::origin() + short * 2.0)
            .expect("has a length")
            .direction(),
        here.direction(),
        "a power-of-two multiple rounds the same way even without a rescale, which is why the \
         witness above is three times and not twice"
    );
    assert_eq!(
        correctly_rounded(short).expect("has a length"),
        correctly_rounded(long).expect("has a length"),
        "a rescale first makes the answer a function of the ratios, and 6:6:5 is 18:18:15"
    );
}

/// **No surface's `basis()` can be handed the same ray at two lengths**, so the
/// second defect reaches none of the five sites that are not `Segment::new`.
///
/// This is the §16.10 demonstration and not a paragraph. The family of routes
/// is every way a caller supplies the vector that reaches those normalizes, and
/// there is only one shape of route: `basis()` and `BSplineCurve::circle` take
/// the vector from what the surface stores, and what the surface stores came
/// through `Axis::new` or through a `Plane` constructor, both of which scale to
/// length one before storing. So the length a caller wrote is gone by the time
/// the bare normalize runs, and the only way to reach it with two lengths of one
/// ray would be a route that stores an unnormalized direction — which is
/// `Plane::transformed`, and that one changes the ray as well as the length and
/// so is a different question (§17.1).
///
/// Written the other way round: every one of these surfaces is built here from
/// the same ray at ninety-six lengths, and every one of them comes back bit for
/// bit the same.
#[test]
fn a_surface_built_from_one_ray_at_any_length_has_one_basis() {
    let mut drawn = 0usize;

    for x in -4i32..=4 {
        for y in -4i32..=4 {
            for z in -4i32..=4 {
                let short = Vector::new(f64::from(x), f64::from(y), f64::from(z));
                if short == Vector::zero() {
                    continue;
                }
                let at = |scale: i32| {
                    Axis::new(Point::origin(), short * f64::from(scale))
                        .expect("a lattice direction has a length")
                };
                let first = at(1);
                for scale in 2..=97 {
                    let again = at(scale);
                    drawn += 1;
                    assert_eq!(
                        Cylinder::new(first, 1.0)
                            .expect("a pipe of radius one")
                            .basis(),
                        Cylinder::new(again, 1.0)
                            .expect("a pipe of radius one")
                            .basis(),
                        "a pipe on {short:?} and one on the same ray {scale} times over"
                    );
                    assert_eq!(
                        Cone::new(first, 0.5)
                            .expect("a cone of slope a half")
                            .basis(),
                        Cone::new(again, 0.5)
                            .expect("a cone of slope a half")
                            .basis(),
                        "a cone on {short:?} and one on the same ray {scale} times over"
                    );
                    assert_eq!(
                        Torus::new(first, 3.0, 1.0).expect("a bend").basis(),
                        Torus::new(again, 3.0, 1.0).expect("a bend").basis(),
                        "a bend on {short:?} and one on the same ray {scale} times over"
                    );
                    assert_eq!(
                        BSplineCurve::circle(first, 1.0).expect("a unit circle"),
                        BSplineCurve::circle(again, 1.0).expect("a unit circle"),
                        "a circle round {short:?} and one round the same ray {scale} times over"
                    );
                }
            }
        }
    }

    println!("{drawn} pairs of surfaces built from one ray at two lengths, no basis moved");
    assert!(
        drawn >= 40_000,
        "only {drawn} pairs, which is not the demonstration this claims to be"
    );
}

/// **The divide the six sites skip is what buys the ratios**, so skipping it
/// loses something `Plane`'s constructors have rather than sharing a defect with
/// them.
///
/// §121.1 reads the bare normalize as the constructor path minus a step, and so
/// as carrying that path's defect plus one more. Over the same two hundred
/// thousand proportional pairs the previous test breaks on 38.1% of the time,
/// dividing by the largest component first breaks on **none**: the divide puts
/// a `±1.0` in the largest slot whatever length the ray was written at, and the
/// remaining components are the rounded ratios, which do not care either.
///
/// It is not the same guarantee as the rescale's. A power of two moves no
/// mantissa bit and the divide does, so this is a measurement over one lattice
/// and not a property — but it is enough to say the two defects §121.1 names do
/// not come as a pair, and that `Segment::new` gave up a guarantee the two
/// constructors were already keeping.
#[test]
fn dividing_by_the_largest_component_keeps_the_ratios_where_normalizing_alone_does_not() {
    let mut drawn = 0usize;
    let mut normals_differed = 0usize;

    for x in -6i32..=6 {
        for y in -6i32..=6 {
            for z in -6i32..=6 {
                let short = Vector::new(f64::from(x), f64::from(y), f64::from(z));
                if short == Vector::zero() {
                    continue;
                }
                for scale in 2..=97 {
                    let long = short * f64::from(scale);
                    drawn += 1;
                    let here = Plane::from_point_and_normal(Point::origin(), short)
                        .expect("a lattice normal has a length");
                    let further = Plane::from_point_and_normal(Point::origin(), long)
                        .expect("a multiple of it has one too");
                    if here.normal() == further.normal() {
                        assert_eq!(
                            here.basis(),
                            further.basis(),
                            "two planes holding one normal are meant to have one basis"
                        );
                    } else {
                        normals_differed += 1;
                    }
                }
            }
        }
    }

    println!(
        "{drawn} pairs of planes written from one ray at two lengths: {normals_differed} stored \
         two different normals"
    );
    assert!(
        drawn >= 200_000,
        "only {drawn} pairs, which is not the measurement this claims to be"
    );
    assert_eq!(
        normals_differed, 0,
        "dividing by the largest component is meant to leave the ratios alone on a lattice this \
         small, and here it moved {normals_differed} of {drawn} planes"
    );
}

/// **The first defect does reach every basis, and it reaches it about as often
/// as the sweep says.**
///
/// The vector `Cylinder::basis` normalizes is `e - along·(along·e)` for the
/// coordinate direction `e` the axis leans on least. That is a vector like any
/// other, and normalizing it without a rescale first misses the nearest `f64`
/// at the rate the first test measures. Here it is measured at the site rather
/// than in the abstract, over axes drawn across the sphere.
///
/// The cone and the bend run the same three lines as the pipe, so one count
/// covers all three; the plane crosses instead of projecting and the circle
/// crosses the other way, and each gets its own.
#[test]
fn how_often_a_surfaces_basis_is_not_the_nearest_f64() {
    let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut a_mantissa = || {
        f64::from(u32::try_from(next() >> 32).expect("thirty-two bits")) / f64::from(u32::MAX) - 0.5
    };

    let mut drawn = 0usize;
    let mut a_pipes_basis_missed = 0usize;
    let mut a_planes_basis_missed = 0usize;
    let mut a_circles_plane_missed = 0usize;
    let mut bare_is_not_of_length_one = 0usize;
    let mut correct_is_not_of_length_one = 0usize;

    for _ in 0..20_000 {
        let written = Vector::new(a_mantissa(), a_mantissa(), a_mantissa());
        let (Ok(axis), Ok(plane)) = (
            Axis::new(Point::origin(), written),
            Plane::from_point_and_normal(Point::origin(), written),
        ) else {
            continue;
        };
        drawn += 1;

        let along = axis.direction();
        let leans_on_least = the_coordinate_direction_it_leans_on_least(along);
        let across = leans_on_least - along * along.dot(leans_on_least);
        let bare = Cylinder::new(axis, 1.0)
            .expect("a pipe of radius one")
            .basis()
            .0;
        let correct = correctly_rounded(across).expect("the axis is not along that direction");
        if bare != correct {
            a_pipes_basis_missed += 1;
        }
        // Everything that treats a basis direction as unit — `point_at` putting
        // a mesh point on the surface, `plane_coordinates_of` reading a
        // distance off one — pays for whatever this is not.
        bare_is_not_of_length_one += usize::from(bare.length() != 1.0);
        correct_is_not_of_length_one += usize::from(correct.length() != 1.0);

        let normal = plane.normal();
        let crossed = normal.cross(the_coordinate_direction_it_leans_on_least(normal));
        if plane.basis().0 != correctly_rounded(crossed).expect("the normal is not along it") {
            a_planes_basis_missed += 1;
        }

        // The circle's third control point is `centre + a_quarter_on * radius`,
        // so a unit circle round the origin writes that direction down
        // untouched and it can be read back off the curve.
        let a_quarter_on = correctly_rounded(along.cross(leans_on_least))
            .expect("the axis is not along the direction it leans on least");
        if BSplineCurve::circle(axis, 1.0)
            .expect("a unit circle")
            .control_point(2)
            .coordinates()
            != a_quarter_on.components()
        {
            a_circles_plane_missed += 1;
        }
    }

    println!(
        "{drawn} drawn axes: a pipe's basis was not the correctly rounded direction \
         {a_pipes_basis_missed} times ({:.1}%), a plane's {a_planes_basis_missed} times \
         ({:.1}%), and the direction a unit circle starts a quarter-turn on \
         {a_circles_plane_missed} times ({:.1}%). A pipe's first basis direction \
         was not of length one {bare_is_not_of_length_one} times as it is written \
         and {correct_is_not_of_length_one} times correctly rounded.",
        100.0 * a_pipes_basis_missed as f64 / drawn as f64,
        100.0 * a_planes_basis_missed as f64 / drawn as f64,
        100.0 * a_circles_plane_missed as f64 / drawn as f64,
    );
    assert!(
        drawn >= 15_000,
        "only {drawn} axes, which is not the measurement this claims to be"
    );
    for (missed, what) in [
        (a_pipes_basis_missed, "a pipe's basis"),
        (a_planes_basis_missed, "a plane's basis"),
        (a_circles_plane_missed, "the plane a circle is written in"),
    ] {
        assert!(
            missed * 4 > drawn,
            "a bare normalize is meant to miss the nearest f64 often enough to matter at every \
             one of these sites, and at {what} it missed on only {missed} of {drawn}"
        );
    }
}

/// The reference: `Vector::scaled_to_length_one_from_anywhere`, reached through
/// the only public caller that runs it and nothing else.
fn correctly_rounded(direction: Vector) -> rcad_geom::Result<Vector> {
    Ok(Axis::new(Point::origin(), direction)?.direction())
}

/// What `Plane`'s two constructors do before they normalize, so the first test
/// measures the constructor path beside the bare one.
fn divided_by_its_largest_component(direction: Vector) -> Vector {
    let [x, y, z] = direction.components();
    let widest = x.abs().max(y.abs()).max(z.abs());
    Vector::new(x / widest, y / widest, z / widest)
}

/// The tie-breaking the four `basis()` sites and the circle all share.
fn the_coordinate_direction_it_leans_on_least(direction: Vector) -> Vector {
    let [x, y, z] = direction.components().map(f64::abs);
    if x <= y && x <= z {
        Vector::new(1.0, 0.0, 0.0)
    } else if y <= z {
        Vector::new(0.0, 1.0, 0.0)
    } else {
        Vector::new(0.0, 0.0, 1.0)
    }
}

/// §120.6's population, rebuilt here: the same four hand-written directions, the
/// same lattice, the same nine hundred binades and the same generator, so the
/// rates this file prints sit beside the ones that section prints.
fn the_population() -> Vec<Vector> {
    let mut directions = vec![
        Vector::new(1.0, 1.0, 0.0),
        Vector::new(1.0, 1.0, 1.0),
        Vector::new(3.0, 4.0, 5.0),
        Vector::new(2f64.powi(-483), 2f64.powi(516), 0.0),
    ];
    for x in -6i32..=6 {
        for y in -6i32..=6 {
            for z in -6i32..=6 {
                let triple = Vector::new(f64::from(x), f64::from(y), f64::from(z));
                if triple != Vector::zero() {
                    directions.push(triple);
                }
            }
        }
    }
    for spread in 1..=900 {
        directions.push(Vector::new(1.0, times_two_to_the(1.0, -spread), 0.0));
    }

    let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for _ in 0..3_000 {
        let mut drawn = || {
            let bits = next();
            let mantissa = f64::from(u32::try_from(bits >> 32).expect("thirty-two bits"))
                / f64::from(u32::MAX)
                - 0.5;
            let exponent = i32::try_from(bits % 801).expect("under a thousand") - 400;
            times_two_to_the(mantissa, exponent)
        };
        let triple = Vector::new(drawn(), drawn(), drawn());
        if triple != Vector::zero() {
            directions.push(triple);
        }
    }
    directions
}

/// `value * 2^exponent`, in two steps so the factor itself never leaves the
/// range — the crate's `multiplied_by_a_power_of_two`, which is private to it.
fn times_two_to_the(value: f64, exponent: i32) -> f64 {
    let half = exponent / 2;
    value * 2f64.powi(half) * 2f64.powi(exponent - half)
}
