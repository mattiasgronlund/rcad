## 303. The lattice pair's not-a-solid floor was too tight for a soak from the day it was set — 2026-09-27

*Queue row Q228, closing §290.4's finding A. Lane
`lane-q228-lattice-band-floor`, off `4066f85b`. Every run pinned to cores
6-13, eight at a time.*

**The band was too tight; the share did not move.** The lattice family's
not-a-solid rate is **2.23%**, and the floor `A_LATTICE_PAIR_IS_NOT_A_SOLID`
had was 2.2% — two thirds of one draw, the shipped corpus's 14 of 429 (3.3%),
which is a high draw. At `RCAD_SOAK_FACTOR=66` the floor failed **38 of 113**
fresh seeds. On the tree where the band was set, `e628f9fa`, every one of 32
soaked seeds gives **the same count** it gives today, 112's 555 and 115's 556
included. So the band failed a soak on the day it was written, and §290.4's two
failures were two ordinary draws. Both it and `A_TURNED_PAIR_IS_NOT_A_SOLID`
are now two thirds to three times a rate measured at a soak, and each end is
judged with the draw's own spread (§303.3).

### 303.1 How it was measured

`Tally::report` already prints the tally (`N fuses, R refused, M not a solid`),
so no scratch print was needed. Each run is the test binary called directly,
`RCAD_SOAK_FACTOR=<f> RCAD_SOAK_SEED=<s> <fuses_that_must_agree binary> --exact
<test> --nocapture --test-threads 1`, from `crates/rcad-topo`. The script is
`target/q228/one.sh <test> <factor|shipped> <seed|shipped>`, and
`taskset -c 6-13 xargs -P 8 -L 1 target/q228/one.sh < jobs.txt` runs eight at
a time. `python3 target/q228/summary.py <runs dir>` tabulates the runs. The old
tree is `git archive e628f9fa`, extracted under `target/q228/old` and built with
`cargo nextest run --manifest-path target/q228/old/Cargo.toml -p rcad-topo
--test fuses_that_must_agree --no-run`. Both trees use toolchain 1.96.0.

| tree | row | draw | runs | fuses a run | not a solid, pooled | spread per seed | under 2.2% |
|---|---|---|---|---|---|---|---|
| `4066f85b` | lattice | factor 66 | 113 seeds | 25 401 | 63 913 of 2 870 313, **2.227%** | 0.079 pts | **38** |
| `4066f85b` | lattice | factor 1 | 400 seeds | 441 | 4 591 of 176 400, 2.603% | 0.650 pts | **112** |
| `4066f85b` | lattice | shipped seed | 1 | 441 | 14, 3.175% | — | 0 |
| `4066f85b` | turned | factor 66 | 48 seeds | 25 401 | 29 847 of 1 219 248, **2.448%** | 0.121 pts | 0 |
| `4066f85b` | turned | factor 1 | 400 seeds | 441 | 6 987 of 176 400, 3.961% | 0.920 pts | **4** |
| `4066f85b` | turned | shipped seed | 1 | 441 | 14, 3.175% (93 refused) | — | 0 |
| `e628f9fa` | lattice | factor 66 | 32 seeds | 25 389 | 18 257 of 812 448, 2.247% | 0.052 pts | **5** |
| `e628f9fa` | lattice | factor 1 | 400 seeds | 429 | 4 591 of 171 600, 2.675% | 0.668 pts | **112** |
| `e628f9fa` | lattice | shipped seed | 1 | 429 | 14, 3.263% | — | 0 |
| `e628f9fa` | turned | factor 66 | 24 seeds | 25 389 | 15 019 of 609 336, 2.465% | 0.121 pts | 0 |
| `e628f9fa` | turned | factor 1 | 400 seeds | 429 | 6 956 of 171 600, 4.054% | 0.958 pts | **5** |
| `e628f9fa` | turned | shipped seed | 1 | 429 | 14, 3.263% (96 refused) | — | 0 |

The factor-66 seeds are 101-117, §290's own, then 1 upwards. The factor-1
seeds are 1-400. Seeds 112 and 115 reproduce §290.4 exactly: 555 and 556 of
25 401, and the base binary fails both with its own message.

**The lattice counts did not move between the two trees.** Seed for seed, all
32 factor-66 seeds and all 401 factor-1 runs, the shipped seed among them,
give the same count on `e628f9fa` as on `4066f85b` (`python3
target/q228/compare.py`: 32 of 32 and 401 of 401 identical). The denominators
differ only by the four seeds replayed since, 12 fuses. The turned counts did
move, by −29 to +27 a seed at factor 66, with 3 of 24 identical. They moved
both ways, and the pooled rate moved 0.02 points.

### 303.2 The hypothesis's three legs

The brief's hypothesis: at n = 25 401 a floor one or two spreads under the
rate fails a few to fifteen per cent of soaks.

1. **The denominator is fixed at 25 401 at factor 66** — held, on all 161
   soaked runs. It is `3 × (128 × 66 + 19)`. `fused` counts a fuse before it
   runs, `every_rule` makes three per configuration, and the nineteen
   `cc` lines in `fuses_that_must_agree.proptest-regressions` are replayed on
   top.
2. **The share's centre is near 0.023** — broken, and in the direction that
   made it worse. It is 0.02227, 0.34 spreads above the floor, so the floor
   fails about a third of soaks: 38 of 113. §290's 2 of 16 was a low draw of
   that. About 5 would have been expected.
3. **The draws are independent per seed** — not contradicted. Every seed is
   a different ChaCha key. The lattice spread per seed, 0.079 points, is
   *narrower* than the binomial's 0.093, which nothing here explains. The
   turned spread, 0.121 against 0.097, is wider, as it would be if one
   configuration's three fuses tended to fail together. Neither spread widens
   the band enough to matter (§303.3).

**Why factor 1 centres higher than factor 66.** Run from a directory outside
the repo, proptest cannot find the regression file ("Failed to find absolute
path of source file"), so no seed is replayed and the shipped seed draws 384
fuses. Lattice gives 11 of 384 and turned 6 of 384, against 14 of 441 for
both with the replays (`target/q228/no-replays.sh`). So the nineteen replayed
seeds give 3 of their 57 fuses not a solid for lattice and 8 for turned. That
weighs an eighth of a 441-fuse draw and a four-hundredth of a soak.

### 303.3 The band now

`band_for_a_draw_of(fuses, rate)` moves each end of the rate band out by
`A_DRAW_STRAYS_THIS_MANY_SPREADS` (three) of `sqrt(end × (1 − end) / fuses)`,
and never lets the floor under one fuse.
`Tally::rate_of_answers_that_were_not_solids_must_be_within` judges the lattice
and turned rows with it. The rate ends follow the file's convention, applied
to the soaked rate rather than to one draw:

| row | rate | ends | at 441 fuses | at 25 401 fuses |
|---|---|---|---|---|
| lattice | 2.227% | 1.5% – 6.7% | fires at 0 or over 45 | fires under 323 or over 1 821 |
| turned | 2.448% | 1.6% – 7.3% | fires at 0 or over 48 | fires under 347 or over 1 978 |

Every measured run on the current tree is inside its band: 113 + 401 lattice
runs and 48 + 401 turned runs (`python3 target/q228/newband.py
target/q228/runs`).

**The chance of a false fire, from the binomial at the measured rate, is under
1 in 20 000 at every factor from 1 to 660.** The worst is at factor 1: 4.9e-5
for lattice and 1.8e-5 for turned (`python3 target/q228/shipped-floor.py`).
**What it still catches**, same model: at factor 66, a halved lattice rate
fires 99 times in 100 and a halved turned rate 98. At factor 30 it is 69 and
60 times in 100, at factor 10 11 and 9.

**What it gives up is the floor at the shipped size, and no floor there
could have kept it.** Across the 400 fresh seeds at 441 fuses, a floor that
fires on a halved rate (binomial at half of 2.603%) often enough to matter
also fires on honest draws:

| fire under | honest lattice seeds that fire | halved lattice rate that fires |
|---|---|---|
| 6 of 441 | 0.7% | 49% |
| 7 of 441 | 2.8% | 65% |
| 8 of 441 | 7.2% | 78% |
| 10 of 441 (the old floor) | 28.0% | 93% |

The old floor caught a halving on the shipped corpus only because the fixed
seed's draw happened to sit high. The same floor fails 28% of the corpora the
shipped seed would draw after a change to the generator. That is §136.5's
lesson again, now at a count of fourteen, where §136.5 thought the convention
was still sound: *a floor inside the draw's own spread is a flake waiting for
the corpus to be reshuffled.* The alternative was a sharper floor on the
shipped corpus only, gated on `this_is_the_shipped_corpus`. It was not taken,
because it would be exactly that kind of floor. What the shipped suite still
asks is whether the family reaches four faces along one edge at all, and
whether the answers that are not solids have stopped being rare.

`the_lattice_band_takes_in_every_soaked_seed_and_still_fires_on_half_the_rate`
pins the arithmetic in the default suite, which never runs a soak. §290.4's
two draws and the lowest and highest of the 113 (519 and 615) are inside at
25 401. Half the rate, 283, is not. At 441, 14 and 1 are inside and 0 is not.

The failure message now prints its share and band to three decimals. §290.4
had to explain why "(2.2%), outside the 2.20%–9.80%" was not a contradiction.

### 303.4 The turned row: the same exposure, milder

It had the lattice band copied because the shipped counts matched, 14 and 14.
Its rate is higher, 2.448%, so at factor 66 its floor was 2.0 spreads down
rather than 0.34. None of 48 soaked seeds failed, but the lowest was 559 of
25 401, 2.2007%, one fuse clear. At factor 1, 4 of 400 fresh seeds failed.
Same cause, same treatment.

**An observation, not a finding: the turned family is not a solid more
often than the lattice family.** The rates are 2.448% and 2.227%, with
standard errors of 0.017 and 0.007 points. The turned family draws the same
lattice configurations and turns them, and 21% of its shipped fuses (93 of
441) refuse before they can find out whether the answer is a solid. If turning never made a solid
answer into one that is not, its rate would be *at most* the lattice rate. So
some turned fuses come back not a solid where the same configuration upright
is a solid. That may be an exact answer about the rounded input, since a
turned configuration is rounded and so a different configuration, or it may
be a gap. Nothing here tells the two apart, and `A_TURNED_PAIR_IS_NOT_A_SOLID`'s
doc no longer implies the ordering. The row that would settle it is below.

*Later, §351.6:* the turned band is 0.52% to 2.3% since Q711, from a rate of
0.78%, 726 of 92 844 fuses at `RCAD_SOAK_SEED` 11 to 22 and
`RCAD_SOAK_FACTOR=20`; it was 1.6% to 7.3% from 2.448%, as 303.3's table has
it. That rate is below the 2.227% this section measured for the lattice
family, which was not measured again, so on those figures the ordering this
observation reasoned from no longer holds. One cause of what it
suspected was a gap: a box and a taller one in the same place were a solid
upright and `count: 1` turned, and since §351.6 they unite to the taller box
turned too.

### 303.5 What is left

- **The other three bands were measured at a soak by Q258**, `6cafcef4`, and
  moved onto `band_for_a_draw_of` with their ends unchanged. Each was eight
  fresh seeds at factor 66, 25 407 fuses a seed, and 24 seeds at factor 1,
  447 fuses a seed, as each constant's doc comment records.
  `A_FLUSH_PAIR_IS_NOT_A_SOLID` pooled 24 570 of 203 256, 12.09%, each soaked
  seed between 11.83% and 12.52%, and the factor-1 seeds spread from 9.84% to
  14.09%; seeds 9 and 21, 44 of 447 each, were under the fixed 10% floor.
  `A_PAIR_ELSEWHERE_IS_NOT_A_SOLID` pooled 4 540 of 203 256, 2.23%, each seed
  between 2.01% and 2.38%, the factor-1 seeds 1.79% to 3.80%; seeds 3 and 11,
  8 of 447 each, were under the fixed 1.9% floor.
  `A_PAIR_LEANING_ALIKE_IS_NOT_A_SOLID` pooled 4 515 of 203 256, 2.22%, each
  seed between 2.10% and 2.31%, the factor-1 seeds 1.57% to 4.03%, every one
  inside its band. The flush band kept its narrow 10%–28% on purpose. The skew
  row keeps its own count floor (§136.5).
- **The turned-over-lattice gap in §303.4** is a row for whoever next works on
  §17.7: for a sample of turned fuses that come back not a solid, fuse the
  same configuration upright, and say for each whether it is an exact answer
  about the rounded input or a gap.
- §290.4's finding B, the turned bar refusing at seed 112, is untouched here.
