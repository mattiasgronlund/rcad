//! **A bound wider than the pipe the crossing is drawn on says nothing.**
//!
//! The marcher's step used to be judged against the caller's deflection budget
//! and against nothing else. A caller may hand over a budget larger than the
//! whole crossing — `every_point_of_every_crossing_is_on_both_surfaces` in
//! `tests/fuzzing/mod.rs` asks `f64::MAX / 4.0` on purpose, as the cheapest way
//! to find out how big a crossing is before asking for the real one — and
//! against a budget like that the question decides nothing at all. A step of
//! `2.2e307` along the tangent was then accepted wherever the settling happened
//! to put it, which is somewhere else on the crossing entirely.
//!
//! # Why the acceptance oracle could not see this
//!
//! It is worth saying, because the fuzzer ran over these pairs for nights
//! without calling them wrong. The oracle asks whether each point is on both
//! surfaces **to within the bound the walk published for it** — and the
//! jumped-across walk published `2.6e46`. Every point passed, enormously. A walk
//! that gives up its accuracy and says so is behaving exactly as §24.1 asks it
//! to; what nothing checked was whether the number it published was any use.
//!
//! So this suite asserts the other half. The crossing of two pipes lies **on**
//! both of them, so it is drawn on a cylinder of the smaller radius: at most
//! `2r` across. A point announced as being within `4.7e77` of a curve that fits
//! inside a pipe `2^20` wide has not been located on it, whatever the arithmetic
//! did to arrive there. `docs/decisions.md` §89.
//!
//! # The family
//!
//! Two rows, reported separately (§17.7).
//!
//! **The corpus seed** is the pair as the fuzzer drew it, literals and all —
//! `fuzz/corpus/where_two_surfaces_cross/6f501cc2...`, one of nineteen seeds
//! that between them took 99.8% of the time the whole corpus cost.
//!
//! **On the lattice** is the same shape with every number an exact power of two,
//! which is the control: nothing here needs a coordinate `f64` cannot hold, so a
//! refusal in this row is a refusal of arithmetic that was never in danger. It
//! also says the defect is not exotic — a lean of `2^-20` is an ordinary
//! engineering tilt, not a denormal.

use rcad_geom::{Axis, Cylinder, Deflection, Point, Surface, SurfaceMeet, Vector, intersect, walk};
use rcad_predicates::{Policy, ProvenanceLog};

/// The budget the harness asks its first walk with: the coarsest there is.
const THE_COARSEST_ASK_THERE_IS: f64 = f64::MAX / 4.0;

/// A pipe, which is all this suite draws.
fn a_pipe(along: [f64; 3], through: [f64; 3], radius: f64) -> Surface {
    Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::from(through), Vector::from(along)).expect("a direction"),
            radius,
        )
        .expect("a radius"),
    )
}

/// The pipe every row crosses its leaning pipe against: along `y`, radius `2^19`.
///
/// It is the smaller of every pair here, so it is the one that confines the
/// crossing and the one the bound is judged against.
fn the_pipe_across() -> Surface {
    a_pipe([0.0, 1.0, 0.0], [256.0, 0.0, 2560.0], 524_288.0)
}

/// What one row found.
#[derive(Default)]
struct Walked {
    /// Runs that came back with points rather than a refusal.
    answered: usize,
    /// Runs that refused, which is an answer a caller can read.
    refused: usize,
    /// The widest bound any of them published.
    widest: f64,
}

impl Walked {
    /// Every run of one pair, walked at the coarsest ask there is.
    fn and_this_pair(mut self, leaning: &Surface, across: &Surface) -> Self {
        let mut log = ProvenanceLog::new();
        let Ok(SurfaceMeet::Curves(runs)) =
            intersect(leaning, across, &Policy::certified(), &mut log)
        else {
            return self;
        };
        let coarsest = Deflection::of(THE_COARSEST_ASK_THERE_IS).expect("a positive length");
        for run in &runs {
            match walk(
                run,
                [leaning, across],
                coarsest,
                &Policy::certified(),
                &mut log,
            ) {
                Ok(points) => {
                    self.answered += 1;
                    self.widest = self.widest.max(points.deviation().magnitude());
                }
                Err(_) => self.refused += 1,
            }
        }
        self
    }

    /// The row's verdict, printed and then asserted.
    ///
    /// `across` is the pipe that confines the crossing. The floor on `answered`
    /// is the anti-vacuity half: a regression that refused every pair here would
    /// otherwise leave `widest` at zero and this green.
    fn report(self, what: &str, across_the_radius: f64, at_least: usize) {
        println!(
            "{what}: {} runs answered, {} refused, the widest bound published was {:e}",
            self.answered, self.refused, self.widest
        );
        assert!(
            self.answered >= at_least,
            "{what}: only {} of its runs came back with points, so this row asserted nothing \
             about any bound",
            self.answered
        );
        assert!(
            self.widest > 0.0,
            "{what}: every bound came back zero, which is not a marcher measuring anything"
        );
        assert!(
            self.widest <= 2.0 * across_the_radius,
            "{what}: a walk published {:e} as how far its points may sit off a crossing that is \
             drawn on a pipe {:e} across, so the bound does not say where on it they are",
            self.widest,
            2.0 * across_the_radius
        );
    }
}

/// The pair the fuzzer drew, as it drew it.
#[test]
fn the_corpus_seed_that_ran_to_the_cap_publishes_a_bound_its_own_pipe_contains() {
    let across = the_pipe_across();
    Walked::default()
        .and_this_pair(
            &a_pipe(
                [1.8336565722544443e-96, 4.577639102365705e-96, 1.0],
                [0.0, 0.0, 1_124_073_473.0],
                50_331_898.0,
            ),
            &across,
        )
        .report("the corpus seed", 524_288.0, 2);
}

/// The control: the same shape with every number a power of two.
#[test]
fn a_pipe_leaning_by_any_power_of_two_publishes_a_bound_its_own_pipe_contains() {
    let across = the_pipe_across();
    let mut walked = Walked::default();
    for lean in [-20i32, -80, -160, -318, -319] {
        for radius in [20i32, 24, 25] {
            walked = walked.and_this_pair(
                &a_pipe(
                    [2f64.powi(lean), 2f64.powi(lean + 1), 1.0],
                    [0.0, 0.0, 2f64.powi(30)],
                    2f64.powi(radius),
                ),
                &across,
            );
        }
    }
    walked.report("on the lattice", 524_288.0, 20);
}
