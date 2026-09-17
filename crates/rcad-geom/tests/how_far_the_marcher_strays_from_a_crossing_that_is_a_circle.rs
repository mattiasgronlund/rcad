//! §24.18's criterion 7, and the reason §24.1 built a redundant marcher at all.
//!
//! # The oracle exists by construction
//!
//! §24.5: *"The planar special cases of the last two rows — coaxial, parallel
//! axes, equal radii with crossing axes — are exactly the configurations where
//! the marcher has an exact answer to be differenced against, so §24.1's oracle
//! exists by construction rather than by anyone arranging it."*
//!
//! A ball threaded on a pipe's axis is that configuration. It is a **ball and a
//! pipe**, so [`walk`] hands it to the marcher like any other degree-four
//! crossing and nothing in the evaluator knows it is special. And what the two
//! surfaces cross in is two circles whose radius and height anybody can write
//! down: the pipe's own radius `r`, at `±√(R² − r²)` along the axis from the
//! ball's centre. That radius is the number the caller wrote, whatever length
//! the stored direction rounded to (§118.11).
//!
//! So the marcher's points are differenced against **a formula and not against
//! another of our own implementations**, which is what makes this a ground truth
//! rather than a consistency check.
//!
//! # Why a ratio and not a length
//!
//! §16.8: the ratio stays comparable across scales where a count of steps does
//! not. Every pair here is drawn in whole numbers and then written at `2^k` for
//! `k` sweeping two hundred orders either way, and the number this file asserts
//! has to mean the same thing at both ends of that.
//!
//! **One is the failure.** At one the marcher's own published bound stops being
//! a bound on where its points are.
//!
//! # The closed form is measured against the same formula
//!
//! The identical circle is also where a *plane* cuts the ball, and named that
//! way [`walk`] parametrizes it in closed form instead. Both arms are therefore
//! differenced against one written-down answer, which is the only way the two
//! numbers below can be read side by side.

#![forbid(unsafe_code)]

mod soak;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, TestRunner};
use rcad_geom::{
    Axis, Cylinder, Deflection, Plane, Point, Sphere, Surface, SurfaceMeet, Vector, intersect, walk,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// How many threaded balls each row walks on the shipped corpus, which
/// `soak::cases` multiplies and `soak::seeded` reseeds.
///
/// Ninety-six, and that is already fifty thousand marched points: a marcher
/// settles two points onto both surfaces per step and works out a certified
/// enclosure for each, so it costs a great deal more per point than a
/// parametrization does. §24.16's benchmark is where that cost is measured.
const CASES: u32 = 96;

/// How fine each walk is, as a share of how big the shapes are.
const HOW_FINE: f64 = 1e-3;

/// **Criterion 7's ceiling, and it is recorded here.**
///
/// Three quarters. One would be the point at which the published bound stops
/// being a bound at all.
///
/// Measured on the shipped corpus: **0.577, and the same 0.577 across four
/// hundred binary orders of magnitude**, which is §16.8's reason for a ratio
/// arriving as a measurement rather than as an argument.
///
/// That number is high next to the closed form's, and it is high *by
/// construction* rather than by weakness. The parametrized arm publishes
/// `256·ε` of the shape's size against a derivation of about `34·ε`, so a drawn
/// case uses under a hundredth of it. The marcher publishes what it measured:
/// the two certified distances its point ended at, divided by the sine of the
/// angle between the surfaces, which is the radius of the region a point that
/// close to both of them can be in. The true distance to the crossing is
/// `√(d₁² + d₂² − 2d₁d₂cos ψ)/sin ψ` where the published bound is
/// `(d₁ + d₂)/sin ψ`, and for two surfaces meeting square on with equal
/// residuals the ratio between those is `1/√2`. **So a ratio near 0.58 is the
/// bound being nearly tight, and a ratio near zero would have meant the marcher
/// was publishing a number it had not earned.**
const NO_MARCHED_POINT_MAY_STRAY_MORE_THAN: f64 = 0.75;

/// A ball threaded on a pipe's axis, in whole numbers and written at `2^k`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Threaded {
    /// Which coordinate direction the axis runs along.
    along: usize,
    /// Whether it runs backwards along it.
    backwards: bool,
    /// Where on that axis the pipe's origin sits.
    origin: i32,
    /// Where on that axis the ball's centre sits.
    centre: i32,
    /// The pipe's radius.
    skin: u32,
    /// How much wider the ball is than the pipe.
    wider: u32,
    /// The power of two everything is written at.
    scale: i32,
}

impl Threaded {
    /// The two surfaces.
    fn surfaces(self) -> [Surface; 2] {
        let by = 2f64.powi(self.scale);
        let mut direction = [0.0; 3];
        direction[self.along] = if self.backwards { -1.0 } else { 1.0 };
        let mut origin = [0.0; 3];
        origin[self.along] = f64::from(self.origin) * by;
        let mut centre = [0.0; 3];
        centre[self.along] = f64::from(self.centre) * by;

        let pipe = Cylinder::new(
            Axis::new(Point::from(origin), Vector::from(direction)).expect("a direction"),
            f64::from(self.skin) * by,
        )
        .expect("a radius");
        let ball = Sphere::new(Point::from(centre), f64::from(self.skin + self.wider) * by)
            .expect("a radius");
        [Surface::Sphere(ball), Surface::Cylinder(pipe)]
    }

    /// How big the numbers here are, which is what a deflection is a share of.
    fn how_big(self) -> f64 {
        let by = 2f64.powi(self.scale);
        let widest = f64::from(self.origin.abs())
            .max(f64::from(self.centre.abs()))
            .max(f64::from(self.skin + self.wider));
        (widest * by).max(f64::MIN_POSITIVE)
    }

    /// How far `at` really is from the nearer of the two circles the two
    /// surfaces cross in.
    ///
    /// The circle is `r` from the axis and `√(R² − r²)` along it from the ball's
    /// centre, so the distance to it is the length of the two-vector made of how
    /// far off each of those the point is. Written in the pipe's own frame,
    /// where the axis is a coordinate direction and none of it rounds.
    fn how_far_off_the_circle(self, at: Point) -> f64 {
        let by = 2f64.powi(self.scale);
        let skin = f64::from(self.skin) * by;
        let radius = f64::from(self.skin + self.wider) * by;
        let up = (radius * radius - skin * skin).sqrt();

        let along = at.coordinates()[self.along];
        let sideways: f64 = (0..3)
            .filter(|axis| *axis != self.along)
            .map(|axis| at.coordinates()[axis] * at.coordinates()[axis])
            .sum::<f64>()
            .sqrt();
        let height = (along - f64::from(self.centre) * by).abs();
        f64::hypot(sideways - skin, height - up)
    }
}

fn a_threaded_ball() -> impl Strategy<Value = Threaded> {
    (
        0usize..3,
        prop::bool::ANY,
        -6i32..=6,
        -6i32..=6,
        1u32..=5,
        1u32..=5,
    )
        .prop_map(|(along, backwards, origin, centre, skin, wider)| Threaded {
            along,
            backwards,
            origin,
            centre,
            skin,
            wider,
            scale: 0,
        })
}

/// The same, written anywhere in `f64` a whole number times `2^k` still names
/// the shape it was drawn as.
fn a_threaded_ball_somewhere_else() -> impl Strategy<Value = Threaded> {
    (a_threaded_ball(), -200i32..=200).prop_map(|(threaded, scale)| Threaded { scale, ..threaded })
}

/// The headline row: drawn where it stands.
#[test]
fn a_marched_circle_stays_inside_the_bound_the_marcher_published_for_it() {
    let worst = march_them_all(a_threaded_ball(), "marched, drawn where they stand");
    assert!(
        worst < NO_MARCHED_POINT_MAY_STRAY_MORE_THAN,
        "the furthest a marched point strayed was {worst} of its own published bound"
    );
}

/// The same shapes across four hundred binary orders of magnitude, which is what
/// makes a ratio worth asserting rather than a length.
#[test]
fn a_marched_circle_written_anywhere_in_f64_stays_inside_the_same_share_of_its_bound() {
    let worst = march_them_all(
        a_threaded_ball_somewhere_else(),
        "marched, written at 2^k for k from -200 to 200",
    );
    assert!(
        worst < NO_MARCHED_POINT_MAY_STRAY_MORE_THAN,
        "the furthest a marched point strayed was {worst} of its own published bound"
    );
}

/// The closed form, on the same circles and against the same formula.
///
/// The circle a ball threaded on a pipe's axis cuts is also where a plane square
/// to the axis through that circle cuts the ball, and named *that* way the
/// evaluator parametrizes it instead of marching it. So the two arms of the
/// evaluator are measured against one written-down answer, and the numbers this
/// file prints can be read next to each other.
#[test]
fn the_same_circle_named_as_a_plane_cutting_a_ball_is_parametrized_and_not_marched() {
    let mut runner = TestRunner::new_with_rng(configured(), soak::seeded());
    let worst = std::cell::Cell::new(0.0f64);
    let walked = std::cell::Cell::new(0usize);

    let outcome = runner.run(&a_threaded_ball(), |threaded| {
        let [ball, pipe] = threaded.surfaces();
        let (Surface::Sphere(round), Surface::Cylinder(pipe)) = (ball, pipe) else {
            unreachable!("just built them")
        };
        let up = {
            let radius = round.radius();
            let skin = pipe.radius();
            (radius * radius - skin * skin).sqrt()
        };
        let cut = Plane::from_point_and_normal(
            round.centre() + pipe.axis().direction() * up,
            pipe.axis().direction(),
        )
        .expect("a coordinate direction is a unit normal");
        let flat = Surface::Plane(cut);

        let Ok(SurfaceMeet::Curves(runs)) = intersect(
            &flat,
            &ball,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) else {
            return Ok(());
        };
        let deflection = Deflection::of(threaded.how_big() * HOW_FINE).expect("a positive length");
        let Ok(along_it) = walk(
            &runs[0],
            [&flat, &ball],
            deflection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) else {
            return Ok(());
        };
        walked.set(walked.get() + along_it.points().len());
        let bound = along_it.deviation().magnitude();
        for point in along_it.points() {
            worst.set(
                worst
                    .get()
                    .max(threaded.how_far_off_the_circle(*point) / bound),
            );
        }
        Ok(())
    });
    if let Err(why) = outcome {
        panic!("{why}");
    }

    println!(
        "parametrized: {} points, the furthest off the circle it was named on sat {:.3} of the \
         bound published for it",
        walked.get(),
        worst.get()
    );
    assert!(walked.get() > 1000, "walked {} points", walked.get());
    assert!(worst.get() < 1.0);
}

/// March every threaded ball a strategy draws, and hand back the worst share of
/// its own published bound any point was found to stray from the circle.
fn march_them_all(strategy: impl Strategy<Value = Threaded>, what: &str) -> f64 {
    let mut runner = TestRunner::new_with_rng(configured(), soak::seeded());
    let worst = std::cell::Cell::new(0.0f64);
    let marched = std::cell::Cell::new(0usize);
    let runs = std::cell::Cell::new(0usize);

    let outcome = runner.run(&strategy, |threaded| {
        let surfaces = threaded.surfaces();
        let Ok(SurfaceMeet::Curves(named)) = intersect(
            &surfaces[0],
            &surfaces[1],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) else {
            return Ok(());
        };
        let deflection = Deflection::of(threaded.how_big() * HOW_FINE).expect("a positive length");

        for run in &named {
            let Ok(along_it) = walk(
                run,
                [&surfaces[0], &surfaces[1]],
                deflection,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            ) else {
                continue;
            };
            runs.set(runs.get() + 1);
            marched.set(marched.get() + along_it.points().len());
            let bound = along_it.deviation().magnitude();
            for point in along_it.points() {
                worst.set(
                    worst
                        .get()
                        .max(threaded.how_far_off_the_circle(*point) / bound),
                );
            }
        }
        Ok(())
    });
    if let Err(why) = outcome {
        panic!("{why}");
    }

    println!(
        "{what}: {} runs into {} points, the furthest any of them strayed from the circle it \
         was named on was {:.3} of the bound published for it",
        runs.get(),
        marched.get(),
        worst.get()
    );
    assert!(
        marched.get() > 1000,
        "{what} marched {} points, which is not enough to be measuring anything",
        marched.get()
    );
    worst.get()
}

fn configured() -> Config {
    Config {
        cases: soak::cases(CASES),
        source_file: Some(file!()),
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..Config::default()
    }
}
