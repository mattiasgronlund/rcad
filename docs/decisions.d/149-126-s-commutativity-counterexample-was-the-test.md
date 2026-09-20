## 149. §126's commutativity counterexample was the test oracle, not the kernel, diagnosed 2026-09-19

*§126.5 saw a turned pair whose union differed by a face between the two orders,
put the tree back to make `main` safe, and threw the persisted seed away with
it; §126.6 worked out how the seed was lost and left the counterexample
unreproduced. **It is reproduced.** The two answers are the same solid. What
differs is how `fuses_that_must_agree.rs` writes a ring down, and the kernel was
never at fault. The fix this leaves is a test-suite change.*

### 149.1 The pair, and that it needs no seed

An **ell** two by two with a one-by-one bite, and a **comb** of two teeth three
high at `(-1, -4, 0)`, the pair turned together by thirty-two sixty-fourths —
half a turn — about `(0, 3, 2)`. On `ae35781c`, the tree §126.5 saw fail, that
pair fails the property **on its own, with no seed and no search**.

**It was reproduced fresh, not replayed.** 2 085 runs clean, then a first find at
≈2 300 runs, about 110 000 configurations, with proptest's save banner and the
*base* build's panic line before any rebuild. §126.5's "150 runs, 0 failures" is
consistent with a rate near 1 in 2 300 — **not enough search, not a different
tree**, which is the one thing §126.6 could not tell.

Five tests landed in
`crates/rcad-topo/tests/the_turned_pair_whose_union_lost_a_face_one_way_round.rs`,
none `#[ignore]`d.

### 149.2 Why one ring is written down two ways

`turned_to_its_smallest` (`fuses_that_must_agree.rs`) starts a ring at the
**first** corner equal to the smallest one. That is a canonical form only while
no ring repeats a corner.

The union has a face ring that goes through **the origin twice**. The origin is
the one point a turn holds still, and both inputs have a corner there, so the
turned pair meets itself at exactly one point of `f64`. The two orders walk into
it from different sides, so one ring is written down as its two halves in the
two orders — a cyclic rotation by 8 of 12 corners.

Compare the ring by its smallest **rotation** and the two orders agree bit for
bit, on `ae35781c` and on the tree this landed on. The intersection of the same
pair has no repeated corner, and it is the control.

### 149.3 What §126.6 got right, and the piece of its advice that does not work

Right, and still standing: the seed was persisted into a **tracked** file, so
the recovery that made `main` safe is the same operation that destroyed the
reproducer; §126 did not cause the failure; and the failure was real. All three
hold. The habit it drew — copy the regressions file out **before** resetting
anything — is the right habit and this section does not weaken it.

Wrong: *"this file already has `SOAK_FACTOR` and `SOAK_SEED` for exactly that."*
**They do not reach the test in question.**
`swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit` goes through
`over`, which is `run(TestRunner::new(configured(cases)), …)` with a literal
case count; only `measured` builds its runner with `new_with_rng` and
`fresh_seeded_soak`, and only `measured_cases` reads `RCAD_SOAK_FACTOR`. So
`over` ignores both knobs outright, and the only way to widen that search is to
run more processes. Checked here by reading the two helpers, not taken on
report.

That matters beyond the bookkeeping: **a rare property failure hunted with a
knob that does nothing looks unreproducible when it is merely under-searched**,
which is exactly what happened to §126.

### 149.4 The change this leaves, and the test that is built to die

Make `turned_to_its_smallest` take the smallest **rotation** instead of the
first smallest corner. It is a test-suite change with no kernel in it.

**Do it rather than special-case it.** The defect is family-agnostic: any answer
carrying a ring that revisits a corner can trip `same_answer` in any family, and
the lane's fifth test shows the same failure with no kernel involved at all — one
four-corner ring, written down from two of its own corners, coming back as two
different lists.

The lane's fourth test, `the_suites_ring_rule_still_tells_this_one_ring_from_itself`,
asserts **today's** behaviour on purpose, and its doc comment says the day the
rotation change lands this test fails and what replaces it is deleting it.

***That last claim is wrong, and §151.4 says why:*** the test holds its own copy
of both rules, because a test binary cannot call into another one, so changing
the suite would have left it passing and meaningless rather than red. The change
itself landed as §151.

`fuses_that_must_agree.rs` is untouched, and the regressions file was
deliberately left at its committed state: committing that seed would turn `just
gate` red over a false positive in the oracle.

