## 276. Q160: floors for the frustum family, the leaning bar, and the two comparison-share laws a halving no longer trips — 2026-09-26

*Queue row Q160, from §242.8's closing line: "Whether the two worst laws want a
floor of their own is Q160." Off `main` at `64cc4848`. `uptime`'s one-minute
load read 28.98 before the first measurement run and stayed in the high 20s
throughout, because Q148 was fuzzing on this machine the whole time — none of
the shares below depend on load, only wall time does.*

  276.1   What was re-measured, and how
  276.2   Turned-pair commutativity and the depth-four chain
  276.3   The frustum family, and why its old floor is retired
  276.4   The leaning bar is not a fresh-seed law at all
  276.5   Showing each halving

### 276.1 What was re-measured, and how

`fuses_that_must_agree.rs` guards two things per property: a rate band on the
shipped, fixed-seed corpus, and a **comparison floor** — how much of what a
property set out to compare it actually got both sides of. Five properties'
comparison floors were flagged as stale:

- `swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit` (turned-pair
  commutativity) and `uniting_four_at_once_is_uniting_them_one_at_a_time` (the
  depth-four chain), both under the shared `SOME_COMPARISONS_MUST_HAPPEN =
  0.3`, which §242.8 already showed no longer catches a halving of either
  (65% → about 32%, and 32% clears 30%).
- `swapping_the_two_sides_of_a_turned_frustum_and_block_changes_not_one_bit`,
  `turning_a_frustum_and_block_together_changes_no_answer_s_volume`, and
  `swapping_the_two_sides_of_a_block_and_a_frustum_changes_not_one_bit`, all
  three under `A_FRUSTUM_FAMILY_STILL_COMPARES_THIS_MUCH = 0.2`, whose own doc
  said the boundary it was sized against ("eighteen fresh-seed runs … between
  30% and 46%") is gone and the family compares far more now.
- `swapping_the_two_sides_of_a_bar_leaning_through_a_block_changes_not_one_bit`
  and
  `what_a_block_and_a_leaning_bar_take_up_adds_up_inside_a_stated_share_of_what_they_declare`,
  both under `A_LEANING_BAR_STILL_COMPARES_THIS_MUCH = 0.1`.

The first five run from a fresh seed every time (`over`), so each was measured
over twenty runs, `PROPTEST_RNG_SEED` 1 through 20, at the shipped case count
(`CASES = 48`, `LONGER_CASES = 24`), through
`mise exec -- cargo nextest run -p rcad-topo --test merged -E
'test(fuses_that_must_agree::<name>)' --no-capture`. The last two turned out to run from a fixed seed
regardless of `PROPTEST_RNG_SEED` (§276.4) and needed a different treatment.

### 276.2 Turned-pair commutativity and the depth-four chain

| law | worst | best | half the worst |
|---|---|---|---|
| turned-pair commutativity | 64.93% (87/134) | 82.09% (110/134) | 32.46% |
| the depth-four chain | 65.12% (28/43) | 81.40% (35/43) | 32.56% |

Both reproduce §242.8's own numbers almost to the decimal (65.12% and 64.93%
were its two low bounds too), which is itself a check on the method: twenty
independent runs, a different day, the same corpus, the same two numbers.

Both windows — above half the worst, below the worst itself — overlap widely:
(32.46%, 64.93%) and (32.56%, 65.12%). A new constant,
`A_THINNER_LAW_STILL_COMPARES_THIS_MUCH = 0.5`, replaces
`SOME_COMPARISONS_MUST_HAPPEN` at both call sites
(`report_what_it_compared_at_least`, not the plain `report_what_it_compared`).
`SOME_COMPARISONS_MUST_HAPPEN` itself stays at three tenths for the other
dozen-odd properties it still guards, which the doc already measured at 74% to
100% — nowhere near needing to move, and moving it up to serve two laws would
have blunted it for the rest, the same argument the frustum floor was already
built on (§49.3, restated in the doc at `A_THINNER_LAW_STILL_COMPARES_THIS_MUCH`).

### 276.3 The frustum family, and why its old floor is retired

Three properties share what was `A_FRUSTUM_FAMILY_STILL_COMPARES_THIS_MUCH`,
not two — the turned commutativity and volume-under-turn rows the queue row
named, and the *upright* commutativity row
(`swapping_the_two_sides_of_a_block_and_a_frustum_changes_not_one_bit`), found
by grepping every call site rather than trusting the row's own list:

| law | worst | best | half the worst |
|---|---|---|---|
| turned, commutativity | 76.12% (102/134) | 93.28% (125/134) | 38.06% |
| turned, volume under a turn | 76.12% (153/201) | 93.03% (187/201) | 38.06% |
| upright, commutativity | 95.52% (128/134) | 100% (134/134) | 47.76% |

The upright row's own window, (47.76%, 95.52%), still contains 0.5, so
`A_THINNER_LAW_STILL_COMPARES_THIS_MUCH` serves all five properties from
§276.2 and this section with one constant — margins ranging from 2.24 points
(the upright row, which has so little room below "all of it" that even a small
absolute margin is proportionally generous) to 17.5 points. The old
`A_FRUSTUM_FAMILY_STILL_COMPARES_THIS_MUCH = 0.2` is deleted rather than left
unused: nothing calls it once the three sites above move to the shared
constant, and a dead floor nobody reads is worse than none.

The old doc's "eighteen fresh-seed runs … 30% to 46%" measurement predates the
boundary's removal (`arrange`'s third `HowItRuns` variant) and was for the
upright row specifically, going by its language about the cone above the
frustum's cap. It is not re-derived, only superseded: the three rows above are
a direct re-measurement of what stands today.

### 276.4 The leaning bar is not a fresh-seed law at all

Both leaning-bar properties call `over_a_fixed_corpus`, not `over`, which
constructs `TestRunner::new_with_rng(_, TestRng::deterministic_rng(...))`.
`TestRng::deterministic_rng` (proptest 1.11.0,
`src/test_runner/rng.rs:504`) hard-codes its seed and never reads
`PROPTEST_RNG_SEED` or any other environment variable — confirmed by running
both properties once under `PROPTEST_RNG_SEED=1` and once under `=2` and
diffing the output: bit-identical, down to the refusal-cause breakdown.
**One run measures this family for good**, and twenty would have measured the
same thing twenty times.

That one run gives commutativity **60 of 134**, 44.78%, and inclusion and
exclusion **18 of 67**, 26.87% — both risen since the floor's own doc was
written against 28/126 (22%) and 11/63 (17%); the generator has moved. Half of
44.78% is 22.39%; half of 26.87% is 13.43%. The old floor, a tenth, clears
neither. A shared floor in **(22.39%, 26.87%)** — above both halved-worsts,
below both measured values — catches a halving of either law: `0.25` sits
1.87 points under the tighter measurement and 2.61 points over the tighter
half. A narrow window is safe to sit inside here specifically because there is
no sampling noise on a fixed-seed corpus: the number only moves when the code
or the generator does, which is exactly what this floor exists to catch.
`A_LEANING_BAR_STILL_COMPARES_THIS_MUCH` moves from `0.1` to `0.25`.

### 276.5 Showing each halving

Each fix is demonstrated with the same scratch mechanism: a temporary edit to
`Tally::comparing` (not committed) that counts only every other call —

```rust
fn comparing(&self) {
    if self.q160_scratch_skip_next.replace(!self.q160_scratch_skip_next.get()) {
        return;
    }
    self.compared.set(self.compared.get() + 1);
}
```

— with a matching `Cell<bool>` field added to `Tally`. This halves whatever a
property compares without touching its generator, so the "before" run shows
what the old floor does with today's law cut in half, and the "after" run
shows what the new floor does with the same cut. Six runs, one per property, at
a seed near that property's own worst so the halved share sits near the
predicted half:

| property | seed | halved share | old floor | old result | new floor | new result |
|---|---|---|---|---|---|---|
| turned-pair commutativity | 8 | 44/134 = 32.84% | 0.3 | pass | 0.5 | fail |
| depth-four chain | 15 | 14/43 = 32.56% | 0.3 | pass | 0.5 | fail |
| frustum, turned commutativity | 9 | 51/134 = 38.06% | 0.2 | pass | 0.5 | fail |
| frustum, volume under a turn | 9 | 77/201 = 38.31% | 0.2 | pass | 0.5 | fail |
| leaning bar, commutativity | any | 30/134 = 22.39% | 0.1 | pass | 0.25 | fail |
| leaning bar, inclusion and exclusion | any | 9/67 = 13.43% | 0.1 | pass | 0.25 | fail |

Commands, with the scratch patch applied:

```text
PROPTEST_RNG_SEED=8 mise exec -- cargo nextest run -p rcad-topo \
  --test merged \
  -E 'test(fuses_that_must_agree::swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit)' \
  --no-capture

PROPTEST_RNG_SEED=15 mise exec -- cargo nextest run -p rcad-topo \
  --test merged \
  -E 'test(fuses_that_must_agree::uniting_four_at_once_is_uniting_them_one_at_a_time)' --no-capture

PROPTEST_RNG_SEED=9 mise exec -- cargo nextest run -p rcad-topo \
  --test merged \
  -E 'test(fuses_that_must_agree::swapping_the_two_sides_of_a_turned_frustum_and_block_changes_not_one_bit) \
      or test(fuses_that_must_agree::turning_a_frustum_and_block_together_changes_no_answer_s_volume)' \
  --no-capture

PROPTEST_RNG_SEED=1 mise exec -- cargo nextest run -p rcad-topo \
  --test merged \
  -E 'test(fuses_that_must_agree::swapping_the_two_sides_of_a_bar_leaning_through_a_block_changes_not_one_bit) \
      or test(fuses_that_must_agree::what_a_block_and_a_leaning_bar_take_up_adds_up_inside_a_stated_share_of_what_they_declare)' \
  --no-capture
```

Run once with the scratch patch and today's floors (`main` at `64cc4848`), all
six report "pass" as the table says — the old floors genuinely let a halving
through. Re-run with only the floor constants changed to this section's new
values (still with the scratch patch, still the same seeds), all six fail with
`only N of M comparisons had an answer on both sides`. The scratch patch is
never committed; §276.1's `mise exec -- cargo nextest run -p rcad-topo --test
fuses_that_must_agree` (71 passed, 2 skipped, no scratch, default seeds) is
what confirms the real change leaves every property green today.

**What is not re-derived here**: `TURNED_REFUSALS_MUST_STAY_BELOW`,
`SKEW_REFUSALS_MUST_STAY_BELOW`, `LEANING_BAR_REFUSALS_MUST_STAY_BELOW`, and
the six not-a-solid bands are refusal-rate and not comparison-share guards,
judged on the shipped fixed-seed corpus by a separate convention
(`this_is_the_shipped_corpus`), and this row did not touch them.
`THE_WIDER_GRID_COMPARES_THIS_MANY` is an exact count and not a floor and was
likewise left alone.
