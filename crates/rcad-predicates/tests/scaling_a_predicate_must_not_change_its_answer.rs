//! A search for level 1 filters that certify a sign they have not earned.
//!
//! Every level 1 error bound is a *relative* one: some small multiple of a sum
//! of magnitudes. That only bounds anything while each rounding on the way
//! costs a fraction of the number being rounded, which stops being true at both
//! ends of the `f64` range — in the subnormals a product loses an absolute
//! amount instead, and past the top it loses everything.
//!
//! # How this catches one without an exact oracle
//!
//! A determinant is linear in each of its rows, so multiplying a row by a
//! positive number multiplies the determinant by that same positive number and
//! leaves its sign alone. Here `d` is the origin, so the rows *are* the other
//! points, and scaling a point by a power of two scales its row.
//!
//! So each sample is run twice: once with the rows where they were drawn, in
//! the middle of the range where every product is an ordinary normal number,
//! and once with each row pushed down into the subnormals or up towards
//! infinity. The two runs describe the same geometry up to a positive factor,
//! so a certified answer from one must match a certified answer from the other.
//! A disagreement is a defect whichever of the two is wrong — and the middle
//! run is the one Shewchuk's bounds were derived for, and the one the
//! `robust` differential suites next door already check.
//!
//! The sample count is deliberately modest so this stays a test. Set
//! `RCAD_SEARCH_SAMPLES` to run the same search as hard as you like:
//! `RCAD_SEARCH_SAMPLES=10000000 cargo test --release -- --nocapture` prints
//! what each run found.

use rcad_predicates::{
    Decision, Level, orient2d_reporting_level, orient3d, orient3d_reporting_level,
};

/// How many samples each search draws unless `RCAD_SEARCH_SAMPLES` says
/// otherwise. Enough to find every known bad case within the first few
/// thousand, and quick enough to run on every commit.
const SAMPLES: usize = 200_000;

/// The rows are shifted in both directions at once, because that is what makes
/// a subnormal product dangerous rather than merely inaccurate: two rows pushed
/// down until their products fall off the bottom, and a third left high enough
/// to scale the mistake back up into a number that clears the error bound.
const INTO_THE_SUBNORMALS: core::ops::RangeInclusive<i32> = -580..=580;

/// Far enough up that the products, and then the whole value, run past what
/// `f64` can hold.
const UP_TOWARDS_INFINITY: core::ops::RangeInclusive<i32> = 300..=400;

#[test]
fn pushing_a_determinant_into_the_subnormals_does_not_change_its_sign() {
    report(
        "orient3d, subnormal",
        search_orient3d(1, INTO_THE_SUBNORMALS),
    );
}

#[test]
fn pushing_a_determinant_up_towards_infinity_does_not_change_its_sign() {
    report(
        "orient3d, overflow",
        search_orient3d(2, UP_TOWARDS_INFINITY),
    );
}

#[test]
fn pushing_a_turn_into_the_subnormals_does_not_change_its_sign() {
    report(
        "orient2d, subnormal",
        search_orient2d(3, INTO_THE_SUBNORMALS),
    );
}

#[test]
fn pushing_a_turn_up_towards_infinity_does_not_change_its_sign() {
    report(
        "orient2d, overflow",
        search_orient2d(4, UP_TOWARDS_INFINITY),
    );
}

/// The known wrong answer, drawn straight from the family this search covers,
/// so a search that stopped finding anything would still have this to fail on.
#[test]
fn the_determinant_this_search_was_written_for_is_in_the_family_it_covers() {
    let s = 2f64.powi(-538);
    let scaled = orient3d(
        [-3.0, 0.0, 2f64.powi(1000)],
        [11.0 * s, s, 0.0],
        [9.0 * s, s, 2f64.powi(462)],
        [0.0, 0.0, 0.0],
    );
    let middle = orient3d(
        [-3.0, 0.0, 2f64.powi(1000)],
        [11.0 * 2f64.powi(62), 2f64.powi(62), 0.0],
        [9.0 * 2f64.powi(-238), 2f64.powi(-238), 2f64.powi(762)],
        [0.0, 0.0, 0.0],
    );
    assert_eq!(middle, Decision::Neg);
    match scaled {
        Decision::Neg | Decision::Zero | Decision::Pos => {
            panic!("the scaled run certified {scaled:?} where the middle run says {middle:?}")
        }
        Decision::Undecidable { .. } => (),
        other => panic!("unhandled decision: {other:?}"),
    }
}

/// Print what a run found, then insist that it found no disagreement and that
/// it certified enough samples to have been looking at anything at all.
fn report(what: &str, found: Found) {
    println!("{what}: {found:?} out of {} samples", samples());
    assert_eq!(
        found.disagreements, 0,
        "{what}: level 1 certified a sign the same geometry contradicts, \
         {} times in {} certified samples",
        found.disagreements, found.certified
    );
    assert!(
        found.certified > samples() / 100,
        "{what}: a search that certifies almost nothing is not testing the filter: {found:?}"
    );
}

fn samples() -> usize {
    std::env::var("RCAD_SEARCH_SAMPLES")
        .ok()
        .and_then(|count| count.parse().ok())
        .unwrap_or(SAMPLES)
}

/// What a search run came to.
#[derive(Debug, Default)]
struct Found {
    /// Samples where the scaled run certified a sign at level 1.
    certified: usize,
    /// Of those, how many disagreed with the same geometry in the middle of the
    /// range. Must be zero.
    disagreements: usize,
}

fn search_orient3d(seed: u64, shifts: core::ops::RangeInclusive<i32>) -> Found {
    let mut rolls = Rolls::new(seed);
    let mut found = Found::default();

    for _ in 0..samples() {
        let middle = [rolls.point(), rolls.point(), rolls.point()];
        let shifted = middle.map(|point| {
            let shift = rolls.shift(&shifts);
            point.map(|coordinate| coordinate * 2f64.powi(shift))
        });
        if shifted.iter().flatten().any(|c| !c.is_finite()) {
            continue;
        }

        let (scaled, level) = orient3d_reporting_level(
            shifted[0].into(),
            shifted[1].into(),
            shifted[2].into(),
            [0.0, 0.0, 0.0].into(),
        );
        if level != Level::Float || !is_certified(scaled) {
            continue;
        }
        found.certified += 1;

        let truth = orient3d(middle[0], middle[1], middle[2], [0.0, 0.0, 0.0]);
        if truth != scaled {
            found.disagreements += 1;
        }
    }
    found
}

fn search_orient2d(seed: u64, shifts: core::ops::RangeInclusive<i32>) -> Found {
    let mut rolls = Rolls::new(seed);
    let mut found = Found::default();

    for _ in 0..samples() {
        let middle = [rolls.turn(), rolls.turn()];
        let shifted = middle.map(|point| {
            let shift = rolls.shift(&shifts);
            point.map(|coordinate| coordinate * 2f64.powi(shift))
        });
        if shifted.iter().flatten().any(|c| !c.is_finite()) {
            continue;
        }

        let (scaled, level) = orient2d_reporting_level(shifted[0], shifted[1], [0.0, 0.0]);
        if level != Level::Float || !is_certified(scaled) {
            continue;
        }
        found.certified += 1;

        let (truth, _) = orient2d_reporting_level(middle[0], middle[1], [0.0, 0.0]);
        if truth != scaled {
            found.disagreements += 1;
        }
    }
    found
}

fn is_certified(decision: Decision) -> bool {
    match decision {
        Decision::Neg | Decision::Zero | Decision::Pos => true,
        Decision::Undecidable { .. } => false,
        other => panic!("unhandled decision: {other:?}"),
    }
}

/// A reproducible source of random numbers, so the search is the same search
/// every time it runs. Splitmix64, which is four lines and needs no dependency.
struct Rolls(u64);

impl Rolls {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    fn between(&mut self, low: i64, high: i64) -> i64 {
        let width = (high - low + 1) as u64;
        low + (self.next() % width) as i64
    }

    fn point(&mut self) -> [f64; 3] {
        [self.coordinate(), self.coordinate(), self.coordinate()]
    }

    fn turn(&mut self) -> [f64; 2] {
        [self.coordinate(), self.coordinate()]
    }

    /// A small whole number times a power of two, so the coordinate is exact
    /// and its magnitude ranges over a hundred binades. Small mantissas make
    /// near-degenerate configurations common, which is where a filter is asked
    /// to work.
    fn coordinate(&mut self) -> f64 {
        let mantissa = self.between(-8, 8) as f64;
        let exponent = self.between(-20, 20) as i32;
        mantissa * 2f64.powi(exponent)
    }

    fn shift(&mut self, shifts: &core::ops::RangeInclusive<i32>) -> i32 {
        self.between(i64::from(*shifts.start()), i64::from(*shifts.end())) as i32
    }
}
