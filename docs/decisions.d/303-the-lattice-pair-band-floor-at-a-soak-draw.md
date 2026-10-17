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
and never lets the floor under one fuse. *Later, §303.6:* that spread is now
widened by how the answers that are not solids come together in a
configuration, which on this tree leaves every band as it was.
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

### 303.6 Why a draw's spread is not the binomial's

*Queue row Q260, lane `lane-q260-q1417-why-a-band-spreads-wider-than-binomial`,
off `5bcd7c40`. The runs are in that lane's tree under `target/q260/`:
`one.sh <test> <factor> <seed>` runs a row with the regression seeds replayed,
`norep.sh` the same from outside the repo so nothing is replayed (§303.2),
`old.sh` the same on `4066f85b`'s tree, and `python3 target/q260/summary.py
[runs dir]` tabulates them. Both trees carry the count this section adds, so
the counts are the shipped code's.*

**A draw is a sum over configurations, not over fuses.** `every_rule` fuses
each configuration three ways, union, overlap and difference, and the
configurations come one after another from the seed's stream. So a draw's
count is a sum of independent counts out of three, and the binomial, which
takes each fuse as a draw of its own, is the right spread only if a
configuration's three fuses come back not a solid independently. They do not,
and the two families went opposite ways for that one reason. `Tally` now
counts configurations by how many of their three fuses were not a solid, and
`report` prints it.

**On this tree, almost no configuration gives more than one.** At factor 1 with
nothing replayed (128 configurations a draw, seeds 1 to 200), and at factor 66
with the seeds replayed (seeds 1 to 8):

| family | draw | configurations with none / one / two / three | rate | spread per seed | binomial | configurations predict |
|---|---|---|---|---|---|---|
| lattice | factor 1, 200 | 23932/1668/0/0 | 2.172% | 0.7441 pts | 0.7438 pts | 0.7271 pts |
| lattice | factor 66, 8 | 63139/4615/0/0 | 2.270% | 0.0580 pts | 0.0935 pts | 0.0913 pts |
| flush | factor 1, 200 | 16371/9229/0/0 | 12.017% | 1.5519 pts | 1.6593 pts | 1.4146 pts |
| flush | factor 66, 8 | 43529/24224/0/0 | 11.918% | 0.1966 pts | 0.2033 pts | 0.1736 pts |
| elsewhere | factor 1, 200 | 23856/1744/0/0 | 2.271% | 0.7193 pts | 0.7602 pts | 0.7423 pts |
| elsewhere | factor 66, 8 | 63214/4539/0/0 | 2.233% | 0.0905 pts | 0.0927 pts | 0.0906 pts |
| leaning alike | factor 1, 200 | 23897/1703/0/0 | 2.217% | 0.7705 pts | 0.7514 pts | 0.7342 pts |
| leaning alike | factor 66, 8 | 63218/4536/0/0 | 2.232% | 0.0743 pts | 0.0927 pts | 0.0905 pts |
| turned | factor 1, 200 | 24937/661/1/1 | 0.867% | 0.4678 pts | 0.4732 pts | 0.4719 pts |
| turned | factor 66, 4 | 33047/829/0/0 | 0.816% | 0.0404 pts | 0.0564 pts | 0.0560 pts |

A configuration that gives at most one has a count of variance `3p(1 − 3p)`
where the binomial has `3p(1 − p)`, so its spread is `sqrt((1 − 3p) / (1 −
p))` of the binomial's: 0.977 for the lattice family and 0.85 for the flush.
That is the whole of what the per-configuration counts say about the narrow
lattice spread. §303.2's 0.079 against 0.093, over 113 seeds, is 0.85 of the
binomial, and the eight seeds here are 0.62 of it, both below the 0.977 the
configurations give. Within one seed the configurations are independent: on
`4066f85b`, lattice seed 1 at factor 66 split into 66 blocks of 128
configurations gives block sums whose variance is 0.969 of independent
configurations', and turned seeds 1 and 2 give 1.223 and 0.862, against the
0.18 a ratio from 66 blocks strays by chance, one way in one seed and the
other way in the next (`python3 target/q260/blocks.py target/q260/seq/*.txt`,
from `target/q260/seq.sh`, which prints each configuration's count). At
factor 1, 200 seeds a family, the five spreads are 0.97 to 1.10 of what the
configurations predict. So the narrow factor-66 spreads are draws of few seeds
below a spread that is itself a little under the binomial's, not a cause still
to be found; the run that would settle it is the row below.

**On `4066f85b`, the turned family's configurations failed together.** Over
seeds 1 to 8 at factor 66, 2 286 configurations gave one answer that was not a
solid, 537 two and 539 all three (`python3 target/q260/summary.py
target/q260/oldruns`). A configuration's count then varies more than the
binomial allows, and the configurations predict 0.1315 points against the
binomial's 0.0970 at 25 401 fuses, 1.36 times as wide. §303.2's 0.121 over 48
seeds is 0.92 of that. At factor 1 with nothing replayed, 200 seeds of that
tree spread 0.9588 points against a binomial 0.7824, 1.23 times as wide, and
0.91 of the 1.0514 its configurations predict. Which change since took the
failing together away was not traced. §351.6's is the one that fits, a corner
of one solid on an edge of the other named twice, leaving edges with one face
each, which every rule over that pair would meet, but that is not checked. On
this tree the turned spread is 0.99 of the binomial's.

**The band now.** `band_for_a_draw_of(fuses, together, rate)` multiplies the
binomial's variance by `together`, which `Tally::how_many_not_solids_come_together`
counts from the draw itself: for each answer that is not a solid, how many of
its configuration's three were, itself included, on average. With `X` a
configuration's count, `E[X²]` is `together` times `E[X] = 3p`, so `Var X =
together · 3p − 9p²`, at most `together` times the binomial's `3p(1 − p)`
because `together` is at most three. It is one on four families of this
tree, so their bands are the ones they were. Turned is 674 over 666, 1.012,
pooled over its 200 draws at factor 1, and one only in a draw without the two
configurations that gave more than one. On `4066f85b`'s turned draws it would
have been 9 285 over 4 977, 1.87, and the band's spread 1.37 times as wide.
`answers_that_are_not_solids_coming_three_to_a_configuration_widen_the_band`
pins that arithmetic on those counts.

The measured spread was taken from the draw rather than written into each
band as a constant because a constant goes stale with the kernel, as this one
would have at §351.6. It is never under one: the narrowing above comes from
the three rules excluding each other at the measured rate, and a band end is
a rate no draw has had, so the binomial is the spread it keeps there.

**Every family still passes the shipped draw and the soaks.** Each row's shipped
draw passes, and so does every soaked seed above at factor 66. With each rate
above and the replayed seeds' own answers that are not solids (3, 7, 3, 2 and
3 of 63, replayed less fresh at factor 1, seeds 1 and 2), the chance an honest
draw fires is at most 4.8e-07, elsewhere at factor 10, at every factor from 1
to 660 (`python3 target/q260/falsefire.py lattice=0.02270 flush=0.11918
elsewhere=0.02233 turned=0.00867 "leaning alike=0.02232"`). What a halved
rate costs differs by row: lattice 97 times in 100 at factor 66, flush and
elsewhere 100, turned 8, and leaning alike **never**, at any factor to 660.

What this leaves:

- **The leaning-alike band cannot catch its rate halving.** Its floor, 1.1%,
  is half its soaked rate of 2.232%, because Q258 kept the ends derived from
  one draw of 1.6%. Two thirds to three times 2.232% is 1.5% to 6.7%. Turned
  at factor 66 catches a halving 8 times in 100 for the same kind of reason,
  its floor of 0.52% against a rate of 0.867%. Re-deriving both is Q1425.
- **The turned row panics at factor 66, seed 2.** `RCAD_SOAK_FACTOR=66
  RCAD_SOAK_SEED=2` stops in `fuse/assemble.rs` with "a face has at least
  three sides, one that closes, or two not both straight; this one has 2",
  shrunk to a 5 × 3 frame one high at the origin and a 2 × 1 block one high
  at `[3, 0, -1]`, turned by 27 steps about `[-2, -3, -2]`. Proptest wrote
  `cc 495585801e746d7c89c5aceca5ce3a319ee33917cafbeaa1c4186d04c1c51140` into
  the regression file, and this lane took it back out: the file is not in its
  write set. Q1426.
- **Nothing replayed, the turned floor of one fuse fails 5 of 200 honest draws
  of 384 fuses**, seeds 67, 94, 108, 162 and 196, which give none. The
  binomial over configurations puts that at 3.4%. With the regression seeds
  replayed, 2 of the 63 replayed fuses are not solids, so it cannot happen;
  only a run from outside the repo reaches it, and §303.3's floor is kept.
- **The narrow factor-66 spreads**, eight seeds a family and §303.2's 113,
  sit under what the configurations predict by more than one would expect
  together. A run of 48 seeds a family at factor 66, with the per-block counts
  of each, would say whether they are chance. Q1427.

### 303.7 The narrow factor-66 spread was chance

*Queue row Q1427. Seeds 1 to 48, `RCAD_SOAK_FACTOR=66 RCAD_SOAK_SEED=<1..48>
<fuses_that_must_agree binary> --exact <test> --nocapture --test-threads 1`,
from `crates/rcad-topo`, 20 at a time. The binary is built with `cargo nextest
run --cargo-profile test --no-run -p rcad-topo --test fuses_that_must_agree`.
Scripts are under `target/q1427/` (gitignored): `one.sh <family> <seed>` runs
one, `run-<family>.sh` the batch of 48, `extract.py <family>` the table below.
`fuses_that_must_agree.proptest-regressions` gained the turned family's own
`cc` lines below and lost them again before this file was committed, the same
as §303.6's own seed-2 line.*

**Lattice, chance.** All 48 complete, no refusals, and `together` is 1.0000:
no configuration gave more than one of its three fuses not a solid (18 486 of
406 560). The pooled rate is 1.5156% (18 486 of 1 219 680 fuses) — well under
§303.2's 2.227%, consistent with the gaps fixed since 09-27 moving answers
from not a solid to a solid. The denominator moved too: 25 410 fuses a seed,
not §303.2's 25 401, since the regression file now replays 22 lines against
this strategy rather than 19, `3 × (128 × 66 + 22)`; held on every one of the
96 runs below that completed.

The per-seed spread over the 48 is 0.0656 points against a
configuration-predicted 0.0755 (the binomial's own 0.0766 is close, since
`together` is 1), a ratio of variances of 0.76. A chi-square test of the
sample variance against that predicted variance, 35.546 on 47 degrees of
freedom, lands at its 11th percentile: an eleven-in-a-hundred draw, not a
one-in-twenty one. **The narrow lattice spread §303.6 asked about is chance.**

One side effect of the rate's fall is worth naming without being a finding:
1.5156% sits only 0.016 points above `A_LATTICE_PAIR_IS_NOT_A_SOLID`'s raw
floor of 1.5%, where §303.2's 2.227% sat 0.73 points above it.
`band_for_a_draw_of`'s three-spread widening still took in every one of the
48 seeds, so nothing failed here — but the floor is worth a fresh look if the
rate keeps falling, the way Q228 looked at it the first time.

**Turned, too few seeds left to tell.** Of seeds 1 to 48, 24 panic with
Q2236's own message (`fuse/assemble.rs:2572`, "every face joined here brought
an outline of its own..."), needing no new row, and 12 more fail a different
way (below), leaving 12 complete: seeds 3, 4, 7, 10, 12, 13 and 15 to 20. Over
those, the pooled rate is 0.6776% (2 066 of 304 920 fuses, same 25 410 a
seed), `together` is 1.0116 (4 configurations of 101 640 gave three of three,
none gave two), the per-seed spread is 0.0461 against a configuration-predicted
0.0514 (ratio 0.80), and the chi-square on 11 degrees of freedom lands at its
36th percentile — also inside chance, but with two in three seeds thrown out
before they can answer, this says little either way.

**A new failure, 12 of the 48 turned seeds (1 and 38 to 48): `Union went
wrong: this kernel built an answer that does not validate, and wrote nothing:
a hole in a face winds the same way as the outer wire.`** That is
`Error::TheFuseWroteABrokenAnswer { broken: Invalid::HoleWindsWithTheOuterWire
}`, which `fused`'s `a_flat_refusal` does not allow for
`Coordinates::AreUnderAGeneralRotation`, so the test counts it a failure
rather than a refusal. Unlike §417's family, these are small, ordinary pairs
near the origin, not far-apart turns with huge coordinate ranges. Seed 1
shrinks to `Ell { width: 2, depth: 5, bite_width: 1, bite_depth: 2 }` height 3
at the origin and `Ell { width: 2, depth: 3, bite_width: 1, bite_depth: 2 }`
height 1 at `[0, 1, 1]`, turned by `Turn { direction: [-1, 2, -3], steps: 36
}`; proptest's line was `cc
082ed6006b7346baf41062500b56e870dfe4db56b24fbee9d3ad8fd965c2e125`, and this
lane took it back out, the same as §303.6's own. A row to say whether each of
the 12 is §417's gap reached by ordinary numbers or a new one, and fix or pin
it by name, is below.
