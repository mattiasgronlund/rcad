## 126. The closed-crossing rule reads a crossing written too small on a copy scaled up, fixed 2026-09-18

*§122.4 booked one fix and named one thing it would not reach. Both are here:
the crossing that was written far too small to multiply has its sign back, and
the case that survives the fix has a test of its own with a number in it — 26
exponents tried, 26 refusals, which is every exact copy that vector has.*

### 126.1 What was built

`which_way_a_closed_crossing_turns` was three functions and is now five. The two
levels moved into `both_levels_of_the_rule` — the enclosure, then the exact
level wherever the enclosure could not tell — because the public function now
runs that pair twice: once on the numbers the caller wrote, and, where they
defeat both levels, once on a copy of the four vectors with **each vector
multiplied by its own power of two**.

Each vector on its own, rather than one shared exponent, is what §122.4's
multilinearity buys and it is not a refinement: a gradient of ordinary size
beside an axis written at `2^-900` is two separate problems, and one exponent
read off all four would have to solve both at once. Both references are degree
one in `outward[0]`, `outward[1]` and `axis` and degree one or two in
`gradient`, so scaling any one of them by `s > 0` multiplies the answer by a
power of `s` — positive, so the sign is the input's sign.

The exponent per vector brings that vector's **largest** component into
`[1, 2)`: the largest is the one that would overflow, and every smaller
component then sits as far above the bottom of the range as the vector's own
spread allows. `brought_into_range` returns `None` rather than a rounded copy if
any component leaves the normal range on the way, because a copy that lost bits
is different geometry and a sign proved about it says nothing about the input.
`each_vector_brought_into_range` also answers `None` when no vector moved, so a
retry that would re-run the same numbers is not taken.

A refusal out of the retry is **discarded and the original one kept**. Both are
`Decision::Undecidable`, but only the first carries a width the caller can read:
an enclosure measured on the scaled copy is a width in somebody else's units.

**No new arithmetic.** `binary_exponent` and `multiplied_by_a_power_of_two` were
already `pub(crate)` in `rcad-geom`'s `exact.rs`, used by `isometry.rs` and
`point.rs`. `rcad-predicates` carries its own pair in `power_of_two.rs`, also
`pub(crate)`, and that duplication is older than this lane and was left alone:
merging them means widening one crate's public surface for two functions of four
lines each, and that is a decision with an owner rather than a lane's aside.

### 126.2 The witness has its sign

`a_crossing_written_far_too_small_to_multiply_is_still_refused` is now
`a_crossing_written_far_too_small_to_multiply_is_settled_on_a_copy_scaled_up`.
The input is unchanged — §122.3's witness crossing with both gradients scaled by
`2^-600`, axis left at `(1, 0, 0)` — and it answered
`Decision::Undecidable { .. }`; it answers `Decision::Pos`.

The test holds the retry to its story rather than taking the answer on its own:
the enclosure level is asserted to refuse and the exact level is asserted to
return `None`, both called directly on the numbers as written, so the retry
cannot be quietly credited with work either of those did. And it asserts the
answer equals `which_way_a_closed_crossing_turns` on the **unscaled** witness,
which is what "a positive factor cannot change a sign" means as a test rather
than as a sentence.

### 126.3 What is still refused, and how many scales were tried

`a_vector_spread_across_the_whole_range_has_no_scale_that_works` is the case
§122.4 named and would not assume: `axis = (2^1000, 0, 2^-1020)`, against the
ordinary witness crossing. `Exact` multiplies a pair only where the product
stays exact, which is the window from `2^-969` to the top of the range — about
1992 binades wide. This axis spans `2^2020`, wider than the window, so against a
gradient of order one there is no exponent at all: lift the small component clear
of the subnormals and the large one overflows.

**The test walks the family rather than arguing it.** Exponents `-1100..=1100`
are tried on the axis, each copy kept only if every component that was nonzero is
still normal, and the exact level is asked for a sign on each one that survives.
**26 exact copies exist — `2^-2` through `2^23` — and all 26 decline.** That is
the whole of the family §122.4's fix belongs to, so what is missing for this
input is arithmetic wider than a pair of `f64`s per product and not a better
scale to read it at (§16.10, §29).

The exponents whose copy is *not* exact are skipped, and the test would be
worthless without that: at `2^-1100` the small component flushes to zero and the
rule answers `Decision::Pos` — for an axis with one component gone, which is a
different crossing. That answer was measured, not supposed, and it is the reason
`brought_into_range` checks normality instead of trusting the exponent it picked.

### 126.4 Measured

`mise exec -- just gate` is exit 0 at **1976 tests**, 3 integration tests and 24
doc tests. The lane's diff adds exactly one `#[test]` and removes none — the
witness was renamed, not replaced — so that is `d1e573a4`'s count plus one,
derived from the diff rather than measured on the base.

The corpus did not move: `mise exec -- just t how_often_a_real_file_imports
--no-capture` prints `nist: 1 / 44 imported = 2.3%` and `kicad: 114 / 323
imported = 35.3%`, the same two rates §122.3 recorded, and
`assert_pinned_kicad_imports` passes, which is what actually says no file changed
side. That is the expected answer rather than a disappointing one: a STEP file
whose gradients come out at `2^-600` is not what the corpus holds, and this fix
is for the numbers the public API accepts rather than for the ones today's
importer happens to produce (§16.10).

One wildcard arm was avoided rather than added. The first cut of the retry
matched `Pos | Neg | Zero` and sent `_` to the old refusal, which took
`check-wildcards` from 87 to 88; it is written as a `matches!` against
`Decision::Undecidable` instead, and the count is back at 87 (§40).

Commands run to reach this: `git log --oneline -1`; `git checkout -B
lane-scale-and-retry d1e573a4`; `mise exec -- just decision 122`, `122.1`–`122.4`;
`mise exec -- just src product_is_exact`; direct `Read` of
`which_way_a_closed_crossing_turns.rs`, `power_of_two.rs`; `mise exec -- just t
-E 'test(/closed_crossing/)'`; `mise exec -- just t
a_vector_spread_across_the_whole_range_has_no_scale_that_works`; `mise exec --
just gate`; `mise exec -- just t how_often_a_real_file_imports --no-capture`.

### 126.5 The turned-pair failure blamed on this is not this, and the way to see that is a panic

A session merging this lane onto `main` at `ef787559` saw
`swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit` fail, read the
diff, and guessed a cause: each of the four vectors is scaled by a power of two
read off its own largest component, swapping the two sides of a fuse swaps
`outward[0]` and `outward[1]`, so a different pair of scalings is chosen and the
predicate's answer comes to depend on the order of its arguments.

**That guess is wrong twice, and the second way is the one that settles it.**

It is wrong on its own terms, because `brought_into_range` reads one vector and
nothing else. Swapping `outward[0]` and `outward[1]` swaps their two scalings
along with them, so the scaled copy is the same two vectors the other way round
and the cross product at the top of the rule negates exactly, which is what the
rule does on the numbers as written too.
`swapping_the_two_sides_flips_the_answer_on_the_scaled_copy_too` is that sentence
as a test: the §122.3 witness at `2^-600`, which only the retry reaches — the
test asserts both lower levels decline on the swapped order as well — answers
`Decision::Pos` one way round and `Decision::Neg` the other.

That test passes without any change to the rule, and that is the point of it:
there was no asymmetry to fix, and the claim is pinned rather than argued.

**And it is wrong because the rule is never called at all by that family.**
`a_turned_pair` draws from `lattice_solids`, which draws from `a_shape`, and
`a_shape` is `Block`, `Ell`, `Comb` and `Frame` — four extruded polygons and no
curved outline, exactly as §126's own note at `a_block_with_a_bar_through_it`
says it deliberately is. A pair of those has no curved surface, so it has no
closed edge, so `a_closed_crossing_is_walked_the_way_it_is_stored` — the one
caller — is not reached.

That is an unreachability claim, so it was demonstrated rather than written
(CLAUDE.md, §16.10). `which_way_a_closed_crossing_turns`'s body was temporarily
replaced by `panic!`, and on that build:

- `mise exec -- just t swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit`
  is **exit 0**, `1 test run: 1 passed`, 0.735 s.
- `mise exec -- just t a_crossing_with_room_in_it_is_read_the_same_both_ways_round`
  is **exit 100**, `panicked at which_way_a_closed_crossing_turns.rs:93:5:
  TEMPORARY PROBE`.

The second run is what makes the first one evidence: the panic is compiled in
and does fire when the predicate is reached, and the turned-pair family does not
reach it. The probe was reverted; it is written down here rather than left in the
tree.

**The failure did not reproduce anywhere.** 440 runs of that test, each 48
configurations from a fresh random seed — `over` uses `TestRunner::new`, so every
run is a new search and one green run of it is weak evidence either way:

| tree | runs | failures |
|---|---|---|
| `91423951` + this lane, merged | 40 | 0 |
| `91423951`, no §126 | 250 | 0 |
| `ef787559` + this lane's `which_way_a_closed_crossing_turns.rs` | 150 | 0 |

The last row is the configuration the report describes, built by checking out
`ef787559` and taking one file off the branch — the merged file is byte-identical
to the branch's, so that is the same code. 21 120 configurations, nothing.

The 0.42 s in the report suggested a seed replayed out of
`fuses_that_must_agree.proptest-regressions`, which would fail before the 48
random cases and so look reproducible. ~~That was checked and is not it: the
file is 21 lines and 5 249 bytes in every worktree of this repo, the same as the
committed one, so no session has added a seed to it.~~ **That check does not
show that, and the seed is the best explanation there is:** §126.6.

**So this lane is not blocked, and nothing downstream is named either.** What is
recorded is a failure one session saw once and no run since has seen, in a
property whose search is random, in a family this lane's code cannot enter. `mise
exec -- just gate` on the merge is **exit 0, PASS, 2 000 tests** — `91423951`'s
1 999 plus §126.2's one new test — and **exit 0, PASS, 2 001 tests** once this
section's test is in. Both read off the exit code, written to a file and echoed,
and not off the tail of a pipe.

Commands run to reach this: `git log --oneline -1`; `git checkout -B
lane-126-symmetry 91423951`; `git merge --no-ff lane-scale-and-retry`; `mise exec
-- just t swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit
--no-capture`, 440 times across three trees; `mise exec -- just t
a_crossing_with_room_in_it_is_read_the_same_both_ways_round` with a `panic!` in
the rule; `git diff ef787559 91423951 -- crates/rcad-topo/tests/pairs/mod.rs
crates/rcad-topo/tests/fuses_that_must_agree.rs` (empty); `wc -l` on every
worktree's `fuses_that_must_agree.proptest-regressions`; `mise exec -- just gate`.

### 126.6 The seed was not absent, it was thrown away — and a turned-pair counterexample went with it

§126.5 ruled out a replayed seed by reading
`crates/rcad-topo/tests/fuses_that_must_agree.proptest-regressions` today and
finding it identical everywhere. **A file being identical now says nothing about
whether a seed was ever written to it, because the file is tracked.** These three
are each checked rather than argued:

- `configured` in `fuses_that_must_agree.rs` sets `failure_persistence:
  Some(Box::new(FileFailurePersistence::WithSource("proptest-regressions")))`,
  and every runner in that file is built from it. A failure there **writes its
  seed** to that file. That is the point of the setting.
- The file is in `git ls-files`. So an unstaged line appended to it is undone by
  any `git reset --hard` or `git checkout` of the path.
- §126.5's own account records the recovery: the session that saw the failure
  put `origin/main` and `wave-1`'s `main` **back to `ef787559`**, having kept the
  merge on a branch. That is the operation that restores the file.

So the sequence that fits every observation is: the property failed on a random
case, proptest persisted the seed, the second run replayed it **before** the 48
random cases and failed identically in 0.42 s — shorter than the 0.735 s a
passing run of the same test takes, which is the tell — and the reset then threw
the seed away.

**What follows is not about §126.** §126.5's unreachability demonstration
stands, and is independently true: `a_turned_pair` draws from `lattice_solids`
and `a_shape`, whose four variants `Block`, `Ell`, `Comb` and `Frame` are
rectilinear outlines extruded, so every face is a plane, no closed edge exists,
and the rule §126 changes is never entered. Both halves hold at once. **§126 did
not cause that failure, and the failure was real.**

What is left is a commutativity counterexample for turned lattice pairs —
the answer differing by a face between the two orders, not by a refusal — that
was seen, was reproducible while its seed existed, and is now unreproduced in
21 120 configurations since. That bounds how often it is drawn; it does not make
it absent, and a property that must hold always is not answered by a rate.
**Whoever hunts it should raise the case count rather than repeat the default
one**: this file already has `SOAK_FACTOR` and `SOAK_SEED` for exactly that.

***Both halves of that last sentence are superseded by §149.*** The
counterexample **is reproduced** — an ell and a comb turned half a turn about
`(0, 3, 2)`, which fails on `ae35781c` with no seed and no search — and the two
answers turn out to be the same solid: `turned_to_its_smallest` writes one ring
down two ways when the ring repeats a corner. The kernel was never at fault.
And the knobs named here **do not reach this test**: it goes through `over`,
which builds `TestRunner::new(configured(cases))`, so only running more
processes widens that search. §149.3.

**The habit this costs:** a failing property run is evidence that expires. The
seed is written into a **tracked** file, so the recovery that makes `main` safe
is the same operation that destroys the reproducer. Commit the regressions file,
or copy it out, **before** resetting anything.

Commands run to reach this: `grep -rn 'failure_persistence\|FileFailurePersistence'`
over `crates/`; `sed -n '3596,3620p'` on `fuses_that_must_agree.rs` for
`configured`; `git ls-files | grep proptest-regressions`; `mise exec -- just src
a_turned_pair crates/rcad-topo`, `lattice_solids`, and `sed -n '160,200p'` on
`pairs/mod.rs` for `Shape`; `git diff --stat ef787559 merge-126-fails-symmetry --
'*.proptest-regressions'` (empty, so the seed was never committed either);
`mise exec -- just gate`.

