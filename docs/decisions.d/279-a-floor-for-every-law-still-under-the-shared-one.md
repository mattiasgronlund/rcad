## 279. Q185: one shared floor for the rest of the curved family, not one each — 2026-09-26

*Queue row Q185, continuing Q160 (§276): the curved laws under
`SOME_COMPARISONS_MUST_HAPPEN` "have no measured share written in the file
[...] give each a floor a halving falls under, as §276 did for five, or show
one shared floor can." Off `main` at `b0623d41`. Built on `08c5d6a3`, a
partial commit from a dead lane (`lane-q185-floors-measured`) that only fixed
`report_what_it_compared_at_least`'s stale doc comment. Two earlier lanes did
the measuring below; this lane did not repeat it, only re-derived the
aggregates from their CSVs and ran the one demonstration §279.4 describes.*

  279.1   What was measured, and how
  279.2   The worst, the half, and the floor between them
  279.3   §276's two floors: still needed, not folded in
  279.4   Demonstrating the halving

### 279.1 What was measured, and how

`SOME_COMPARISONS_MUST_HAPPEN` guards every property in this file that calls
`Tally::report_what_it_compared` (not `..._at_least` with a floor of its
own) — 24 call sites, all of them running from a fresh seed (`over`, not
`over_a_fixed_corpus`). Two earlier lanes ran each of the 24 under
`PROPTEST_RNG_SEED` 1 through 20, at the shipped case counts, through

```text
PROPTEST_RNG_SEED=<seed> mise exec -- cargo nextest run -p rcad-topo \
  --test merged -E 'test(fuses_that_must_agree::<name>)' --test-threads 4 --no-capture
```

That is **25 series**, not 24, because
`a_solid_with_a_copy_of_itself_taken_away_is_nothing_at_all` reports two:
a lattice one and a turned one. The 15 flat-family series are in
`q185-results.csv` (300 `report_what_it_compared` lines over 280 test runs);
the 10 curved-family series are in `q185-results-curved-first-run.csv` (200
test runs, one series each). Both live in the scratchpad this lane was
handed, alongside `q185-measure.sh` (the script that produced the flat CSV)
and `q185-measure.log` (its 300-line log); this lane read them but did not
copy them into the tree.

This lane re-derived every figure below straight from those two CSVs with

```text
awk -F, 'NR>1 { law=$1; pct=$5+0;
  if (!(law in lo) || pct < lo[law]) lo[law]=pct;
  if (!(law in hi) || pct > hi[law]) hi[law]=pct; }
  END { for (law in lo) printf "%s lowest=%.4f highest=%.4f\n", law, lo[law], hi[law]; }' \
  q185-results.csv q185-results-curved-first-run.csv | sort
```

rather than trusting the row's own arithmetic — which caught one mistake:
the queue row says "100% for eleven series, among them every curved law
except the skew pair", but the awk output above lists **thirteen** series
whose lowest-of-twenty share is exactly 100%, nine of the ten curved series
(every one but the skew pair) and four of the fifteen flat ones
(`a_solid_united_with_a_copy_of_itself_is_itself`,
`a_solid_united_with_what_it_shares_with_another_is_itself`, the lattice
series of `a_solid_with_a_copy_of_itself_taken_away_is_nothing_at_all`, and
`turning_a_ball_and_block_together_changes_no_answer_s_volume`). Every other
figure this section uses matched the row's own arithmetic exactly.

### 279.2 The worst, the half, and the floor between them

The number that matters for a floor is each series' own *worst* run over the
twenty seeds, not its average or its best. From lowest to highest:

- **76.7442%** (33/43), tied, for `uniting_three_at_once_is_uniting_two_and_then_the_third`
  and `uniting_the_first_two_and_then_the_third_is_uniting_the_last_two_and_then_the_first`
  — the two trio laws, and the floor-setters,
- 79.1045% (106/134) for `swapping_the_two_sides_of_a_degenerate_pair_changes_not_one_bit`,
- 82.0895% (110/134) for `swapping_the_two_sides_of_a_skew_pair_changes_not_one_bit`,
- 85.0746% (57/67) for the turned series of the copy-taken-away law,
- 86.05% to 95.35% for the remaining seven flat series,
- **100%** for the other thirteen (§279.1).

Half of the lowest, 76.7442%, is **38.3721%**. Half of a 100% series is
**50%**. A floor sitting anywhere in **(50%, 76.7442%)** clears every series'
own halved worst and falls below every series' own measured worst at once —
it catches a halving of every one of the twenty-five series, and no series
under `SOME_COMPARISONS_MUST_HAPPEN` needs a floor of its own.

`SOME_COMPARISONS_MUST_HAPPEN` moves from `0.3` to **`0.6`**: 16.74 points
below the lowest series' worst (76.74%), 10 points above the highest series'
halved worst (50%). Three tenths itself no longer did this job — half of
76.74% is 38.37%, which clears three tenths, so a law that halved what it
compared from 76.74% down to 38.37% passed quietly under the old floor. This
is what actually broke, not the "a quarter of turned fuses refuse" style of
argument the old doc gave for why three tenths was safe: that argument was
about two specific laws (turned-pair commutativity and the depth-four chain,
pulled out in §276), and said nothing about the trio laws' own margin, which
had shrunk to nothing without anyone re-measuring it.

### 279.3 §276's two floors: still needed, not folded in

§276 gave two families their own lower floor rather than moving
`SOME_COMPARISONS_MUST_HAPPEN` for everyone:
`A_THINNER_LAW_STILL_COMPARES_THIS_MUCH = 0.5` (five properties) and
`A_LEANING_BAR_STILL_COMPARES_THIS_MUCH = 0.25` (two more). Raising the
shared floor to 0.6 does not make either redundant — neither is now above or
equal to it (0.5 < 0.6, 0.25 < 0.6) — so this lane changed neither.

The margin makes the case as well as the inequality does. Two of
`A_THINNER_LAW_STILL_COMPARES_THIS_MUCH`'s five properties, turned-pair
commutativity and the depth-four chain, measure 64.93% and 65.12% worst
(§276.2) — only 4.9 and 5.1 points above the new 0.6, about one of their own
standard deviations (4.246 and 4.919, §276.2's own figures). Every one of
this row's twenty-five series sits at least 16.74 points clear of 0.6, an
order of magnitude more headroom. Folding those two properties into the
raised shared floor would have put them within a sigma of their own
historical worst, which is the same mistake §276 already turned down when it
gave them a floor of their own instead of raising `SOME_COMPARISONS_MUST_HAPPEN`
to meet them. The two leaning-bar properties are further still (44.78% and
26.87%, on a fixed seed with no run-to-run noise to begin with, §276.4) and
were never in question.

`report_what_it_compared_at_least`'s doc comment needed a second correction
beyond `08c5d6a3`'s: it still said this row would add new per-law floors "for
the rest of the curved family's thin laws". It does not — raising
`SOME_COMPARISONS_MUST_HAPPEN` covers the rest of the curved family along
with the flat one, so the doc now says that instead.

### 279.4 Demonstrating the halving

Demonstrated on `turning_a_ball_and_block_together_changes_no_answer_s_volume`
(worst 100% over the twenty seeds, curved, `report_what_it_compared`, no
floor of its own). The same scratch mechanism as §276.5 — a temporary,
uncommitted edit to `Tally::comparing` that counts only every other call:

```rust
fn comparing(&self) {
    if self.q185_scratch_skip_next.replace(!self.q185_scratch_skip_next.get()) {
        return;
    }
    self.compared.set(self.compared.get() + 1);
}
```

with a matching `Cell<bool>` field added to `Tally`. This halves what the
property compares without touching its generator. With the scratch patch
applied, the same command run twice, changing only the floor constant in the
source between runs:

```text
PROPTEST_RNG_SEED=1 mise exec -- cargo nextest run -p rcad-topo \
  --test merged \
  -E 'test(fuses_that_must_agree::turning_a_ball_and_block_together_changes_no_answer_s_volume)' \
  --no-capture
```

Both runs report "101 of 201 comparisons reached (50%)" — the scratch patch
halves 201 fuses's worth of comparisons to 101, not to exactly 100, because
`comparing` is called once per comparison and the toggle starts `false`. At
`SOME_COMPARISONS_MUST_HAPPEN = 0.3` (the value before this row) the run
passes. At `0.6` (this row's value) the same run fails:

```text
volume under a turn, a ball in a block: 402 fuses, 0 refused (0.0%), 0 not a solid (0.0%)
    101 of 201 comparisons reached (50%)

thread 'turning_a_ball_and_block_together_changes_no_answer_s_volume' panicked at
crates/rcad-topo/tests/fuses_that_must_agree.rs:4480:9:
volume under a turn, a ball in a block: only 101 of 201 comparisons had an
answer on both sides, so this property barely asserted anything
```

The scratch patch was never committed; `just t -p rcad-topo --test
fuses_that_must_agree` (71 passed, 2 skipped, no scratch, default seeds, at
`0.6`) is what confirms the real change leaves every property green today.
