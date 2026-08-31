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
//! # What keeps this from turning into a tautology
//!
//! decisions.md §14.1's second question: do the two runs take *different* paths
//! through the dangerous code? Two runs down the same branch prove nothing,
//! however pretty the algebra. Here the middle run evaluates level 1 with
//! Shewchuk's hypotheses intact and the shifted run evaluates it with them
//! broken, and that difference is the whole test.
//!
//! `orient2d` and `orient3d` now retry an [`Decision::Undecidable`] on a copy
//! of the input rescaled by an exact power of two, which is a change of exactly
//! the shape that could flatten those two paths into one. It does not — a
//! retry only ever follows a refusal, and level 1 certifying a sign it has not
//! earned is not a refusal — but "it does not" is a claim, so the counting
//! below measures it rather than assuming it. A run has to find samples where
//! level 1's products left the normal range in the shifted run and stayed
//! inside it in the middle one, or it reports that it was not testing anything.
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

/// Far enough up that `orient3d`'s products, and then the whole value, run past
/// what `f64` can hold. It reaches infinity in the height that scales each
/// minor rather than in the minor itself, which is why this range is enough
/// even though no single product of two coordinates overflows in it.
const UP_TOWARDS_INFINITY: core::ops::RangeInclusive<i32> = 300..=400;

/// `orient2d` multiplies two coordinates and stops, so it has to be pushed
/// nearly twice as far before anything leaves the range at all.
///
/// This is its own constant because sharing `UP_TOWARDS_INFINITY` meant the
/// turn search never overflowed: a coordinate is at most `2^23` here, so at a
/// shift of 400 the largest product is `2^846`, a perfectly ordinary number.
/// The search reported a healthy count of certified samples for it and was
/// checking nothing (decisions.md §14.1, question 2). The counting in [`Found`]
/// is what turned that up.
const A_TURN_UP_TOWARDS_INFINITY: core::ops::RangeInclusive<i32> = 480..=600;

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
        search_orient2d(4, A_TURN_UP_TOWARDS_INFINITY),
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

/// Print what a run found, then insist that it found no disagreement, that it
/// certified enough samples to have been looking at anything at all, and that
/// enough of those really did take the two runs down different paths.
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
    assert!(
        found.down_different_paths > samples() / 1000,
        "{what}: every certified sample evaluated the same arithmetic in both \
         runs, so this search is a tautology and proves nothing: {found:?}"
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
    /// Of those, how many had a level 1 product outside the normal range in the
    /// shifted run and every one inside it in the middle run.
    ///
    /// These are the samples where the two runs genuinely evaluate different
    /// arithmetic, which is the only reason the comparison is worth making
    /// (decisions.md §14.1, question 2). A run that finds none of these has
    /// stopped being a test.
    ///
    /// It counts both of the ways the shifted run can differ: certifying at
    /// level 1 with the hypotheses broken, which is the failure being hunted,
    /// and declining all the way down and settling a rescaled copy, which is
    /// still a different path and still not the middle run's.
    down_different_paths: usize,
}

/// Whether multiplying these two keeps the relative accuracy every level 1
/// error bound assumes.
///
/// Stricter than [`rcad_predicates::product_is_normal`] at the top of the
/// range, because infinity clears any comparison against a finite threshold and
/// is exactly one of the two ways this search breaks the hypothesis.
fn product_stays_in_range(a: f64, b: f64) -> bool {
    a == 0.0 || b == 0.0 || (a * b).is_normal()
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
        if !determinant_products_stay_in_range(shifted)
            && determinant_products_stay_in_range(middle)
        {
            found.down_different_paths += 1;
        }

        let truth = orient3d(middle[0], middle[1], middle[2], [0.0, 0.0, 0.0]);
        if truth != scaled {
            found.disagreements += 1;
        }
    }
    found
}

/// All nine products `orient3d`'s level 1 forms — the six inside its 2x2 minors
/// and the three that scale each minor by a height — with the fourth point at
/// the origin so the rows are the points themselves.
///
/// The last three matter as much as the first six and for the opposite reason.
/// The minors are where a product falls off the bottom; the heights are where
/// the whole term runs off the top, and `orient3d` reaches infinity there long
/// before any minor does.
fn determinant_products_stay_in_range(rows: [[f64; 3]; 3]) -> bool {
    let [a, b, c] = rows;
    let minors = [
        (b[0], c[1]),
        (c[0], b[1]),
        (c[0], a[1]),
        (a[0], c[1]),
        (a[0], b[1]),
        (b[0], a[1]),
    ];
    if !minors
        .iter()
        .all(|&(left, right)| product_stays_in_range(left, right))
    {
        return false;
    }

    let first = b[0] * c[1] - c[0] * b[1];
    let second = c[0] * a[1] - a[0] * c[1];
    let third = a[0] * b[1] - b[0] * a[1];
    product_stays_in_range(a[2], first)
        && product_stays_in_range(b[2], second)
        && product_stays_in_range(c[2], third)
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
        if !turn_products_stay_in_range(shifted) && turn_products_stay_in_range(middle) {
            found.down_different_paths += 1;
        }

        let (truth, _) = orient2d_reporting_level(middle[0], middle[1], [0.0, 0.0]);
        if truth != scaled {
            found.disagreements += 1;
        }
    }
    found
}

/// The two products `orient2d`'s level 1 takes the difference of, with the
/// third point at the origin so the rows are the points themselves.
fn turn_products_stay_in_range(rows: [[f64; 2]; 2]) -> bool {
    let [a, b] = rows;
    product_stays_in_range(a[0], b[1]) && product_stays_in_range(a[1], b[0])
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
