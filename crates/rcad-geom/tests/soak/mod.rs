//! The two variables that turn a shipped measurement into a soak.
//!
//! A file that runs from a fixed seed reproduces its number, which is what makes
//! the number worth writing down. It also means running it again explores
//! nothing at all: the same seed draws the same corpus however many times it is
//! asked. So a search for the case the shipped corpus missed needs two things —
//! a wider corpus and a seed that is not the fixed one — and these are the two
//! variables that give it them.
//!
//! ```text
//! for seed in $(seq 1 500); do
//!   RCAD_SOAK_FACTOR=100 RCAD_SOAK_SEED=$seed cargo nextest run -p rcad-geom \
//!     --release --no-capture
//! done
//! ```
//!
//! They live here rather than in each file because a sweep across several of
//! them at once is only one command if every file spells the variables the same
//! way, and four copies of a name are four chances to spell one of them
//! differently. `rcad-topo` keeps its own copies: sharing across crates needs a
//! dev-dependency crate to hang the module on, and `docs/decisions.md` §13 says
//! not to build a part until a consumer names it.
//!
//! # Every threshold in this crate is judged on every corpus
//!
//! Nothing here gates an assertion on the corpus being the shipped one, and that
//! is deliberate rather than an omission. The four files that take these
//! variables publish either a **worst case** — no marched point strays further
//! than this share of its bound, every walked point is on both its surfaces,
//! turning a pair never changes what it crosses in — or a **share** of a family
//! that may refuse. Both mean the same thing over a wide corpus as over a narrow
//! one, so both are judged over whatever ran, and finding the draw that breaks
//! one is the whole reason to spend the cycles.

// Four test binaries share this, and each takes the part it needs.
#![allow(dead_code)]

use proptest::test_runner::{RngAlgorithm, TestRng};

/// The environment variable that multiplies a file's corpus.
pub const SOAK_FACTOR: &str = "RCAD_SOAK_FACTOR";

/// The environment variable that replaces a file's fixed seed.
pub const SOAK_SEED: &str = "RCAD_SOAK_SEED";

/// How many cases to run: `shipped`, times whatever [`SOAK_FACTOR`] says.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number above zero, or
/// to one whose product is more than a run can count. A product that wrapped
/// would run **fewer** cases than the shipped corpus while the operator read the
/// number as the wide one, which is a green run that says nothing (§48.2).
pub fn cases(shipped: u32) -> u32 {
    multiplied(shipped, std::env::var(SOAK_FACTOR).ok().as_deref())
}

/// The fixed seed, or the one [`SOAK_SEED`] names.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number.
pub fn seeded() -> TestRng {
    match seed_bytes(std::env::var(SOAK_SEED).ok().as_deref()) {
        Some(bytes) => TestRng::from_seed(RngAlgorithm::ChaCha, &bytes),
        None => TestRng::deterministic_rng(RngAlgorithm::ChaCha),
    }
}

fn multiplied(shipped: u32, factor: Option<&str>) -> u32 {
    let Some(factor) = factor else {
        return shipped;
    };
    let by: u32 = factor
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_FACTOR} has to be a whole number, not {factor:?}"));
    assert!(by > 0, "{SOAK_FACTOR} has to be at least one");
    shipped
        .checked_mul(by)
        .unwrap_or_else(|| panic!("{shipped} cases times {by} is more than one run can count"))
}

/// The seed [`SOAK_SEED`] names, laid into ChaCha's thirty-two bytes, or `None`
/// where it names none.
fn seed_bytes(seed: Option<&str>) -> Option<[u8; 32]> {
    let seed = seed?;
    let seed: u64 = seed
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_SEED} has to be a whole number, not {seed:?}"));
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{multiplied, seed_bytes};

    #[test]
    fn a_run_that_names_no_factor_gets_the_shipped_corpus() {
        assert_eq!(multiplied(2048, None), 2048);
    }

    #[test]
    fn a_factor_multiplies_the_shipped_corpus() {
        assert_eq!(multiplied(2048, Some("100")), 204_800);
    }

    /// The failure the operator cannot see from the outside.
    ///
    /// A wrapped product runs fewer cases than the shipped corpus and prints
    /// nothing to say so, so the honest answer is to stop (§50.9).
    #[test]
    #[should_panic(expected = "is more than one run can count")]
    fn a_factor_too_big_to_multiply_stops_the_run_instead_of_narrowing_it() {
        multiplied(2048, Some("2097153"));
    }

    #[test]
    fn a_run_that_names_no_seed_gets_the_fixed_one() {
        assert!(seed_bytes(None).is_none());
    }

    /// The other failure a sweep cannot see from the outside.
    ///
    /// Five hundred seeds that all landed on the same bytes would draw the same
    /// corpus five hundred times, print five hundred identical numbers, and look
    /// exactly like a soak that found nothing.
    #[test]
    fn every_seed_a_sweep_names_lays_down_different_bytes() {
        let laid: BTreeSet<[u8; 32]> = (0..500)
            .map(|seed| seed_bytes(Some(&seed.to_string())).expect("a named seed"))
            .collect();
        assert_eq!(laid.len(), 500);
    }
}
