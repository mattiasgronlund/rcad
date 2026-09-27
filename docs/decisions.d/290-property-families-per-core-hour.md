## 290. What the property families find per core-hour, on six cores for about two hours — 2026-09-27

*Row Q149 (`lane-q149-families-per-core-hour`, off `566db6f1`). Ran on cores
0-5 under `taskset`, nextest held to six threads, seventeen chunks of
`fuses_that_must_agree.rs` and `fuses_that_touch.rs` at `RCAD_SOAK_FACTOR=66`
with a fresh `RCAD_SOAK_SEED` and `PROPTEST_RNG_SEED` per chunk: 2.233 wall
hours, 8.830 CPU-hours in total — short of the three hours asked for, and
§290.6 says why and what finishing it needs. Two distinct findings, both a
known shape rather than a new kind of wrong answer. The third family the row
was asked to run, `a_fuse_read_back_from_rounded_text.rs` (§191.3), turned out
not to take a fresh seed at all: §290.2 proves it by running it twice with
different seeds and getting byte-identical counts.*

*Finished by Q227 (`lane-q227-soak-to-three-hours`, off `8a49bc9d`,
2026-09-27): seven more chunks, seeds 118-124, reach 3.067 wall-hours and
11.917 CPU-hours over all 24 chunks — the three hours asked for are reached.
§290.3, §290.4, §290.5 and §290.6 carry the new rows, the recomputed totals
and rates, and a third occurrence of finding A.*

  290.1   What "fresh seeds" actually reaches, file by file
  290.2   The step-rounding family does not soak — proof
  290.3   The chunks: seed, factor, wall time, CPU time, uptime
  290.4   The two findings
  290.5   Core-hours two ways, and failures per core-hour
  290.6   What is short of three hours, and how to finish it
  290.7   Set beside the fuzz targets (§265.6, §265.7, §226.3)
  290.8   What the comparison can and cannot tell

### 290.1 What "fresh seeds" actually reaches, file by file

The brief named `PROPTEST_RNG_SEED` and `PROPTEST_CASES`. Reading the three
files' own runners shows those are only part of the story, and for one file
no part of it:

- **`fuses_that_must_agree.rs`** has three ways a property gets its draw.
  Thirty-four call sites use `over()`, which calls `TestRunner::new()` on a
  `Config` whose `cases` field is always set to a literal written at the call
  site — so `PROPTEST_CASES` cannot change how many cases a call to `over()`
  draws, but `PROPTEST_RNG_SEED` does move its seed, because `TestRunner::new`
  takes the seed proptest's own `Config::default()` reads from that variable
  and the struct literal never overrides it. Fourteen call sites use
  `measured()`, which calls `TestRunner::new_with_rng()` directly with a seed
  from `fresh_seeded_soak()` (reads `RCAD_SOAK_SEED`, a name local to this
  codebase, not proptest's) and a case count from `measured_cases()`
  (`MEASURED_CASES = 128`, multiplied by `RCAD_SOAK_FACTOR` when it is set) —
  `PROPTEST_RNG_SEED` and `PROPTEST_CASES` do nothing to these. Two call sites
  use `over_a_fixed_corpus()`, `TestRng::deterministic_rng()`, no environment
  variable moves them at all.
- **`fuses_that_touch.rs`** (the touching family, §191.5) defines its own
  `SOAK_FACTOR` and `SOAK_SEED` constants, and both happen to name the same
  two variables, `RCAD_SOAK_FACTOR` and `RCAD_SOAK_SEED` — so the one pair of
  settings that moves `measured()` in the first file also moves this one.
  `PROPTEST_RNG_SEED` has no effect on it: its runner is built with
  `TestRunner::new_with_rng(config, seeded())`, never `TestRunner::new`.
- **`a_fuse_read_back_from_rounded_text.rs`** (the step-rounding family,
  §191.3) reads no environment variable anywhere in the file. §290.2.

So the two knobs that actually reach "fresh seeds, for hours" are
`RCAD_SOAK_SEED` and `RCAD_SOAK_FACTOR`, run together — not
`PROPTEST_CASES`, which this row found no effect for in any of the three
files it read (a narrower claim than "in the codebase": these three files are
what was checked). `PROPTEST_RNG_SEED` was set on every chunk too, for the
sake of the thirty-four `over()`-based properties, but it only moves which
fixed-size draw they get, not how large a draw — those thirty-four barely got
more exposure per chunk than one default run of `just t` gives them, no
matter how many chunks ran.

### 290.2 The step-rounding family does not soak — proof

`a_family_read_back_from_rounded_text` builds its `TestRunner` with:

```rust
let mut runner = TestRunner::new_with_rng(
    Config { cases: CASES, failure_persistence: None, ..Config::default() },
    TestRng::from_seed(RngAlgorithm::ChaCha, &[191; 32]),
);
```

`CASES` is the literal `48`. Nothing in the file calls `std::env::var`. So the
48 configurations it draws, and the order it draws them in, cannot change
between runs.

Run once with no environment variables set, and once with
`PROPTEST_RNG_SEED=999999 RCAD_SOAK_SEED=999999 RCAD_SOAK_FACTOR=5`, both
under `taskset -c 0-5`, `mise exec -- just t -p rcad-step --test
a_fuse_read_back_from_rounded_text --test-threads 6 --no-capture`: the two
runs print identical per-family counts, digit for digit —

| family | as built | 11 digits | 12 digits | 13 digits |
|---|---|---|---|---|
| lattice pair | 144 decided, 0 refused | 144/0 | 144/0 | 144/0 |
| skew pair | 122 decided, refused `{ResultIsNotManifold:1, Undecidable:21}` | 122/{same}, 0 rounding-introduced, 3 rescued | 122/{same}, 3 rescued | 45 decided, refused `{Undecidable:99}`, 77 rounding-introduced, 3 rescued |
| turned pair | 109 decided, refused `{ResultIsNotManifold:2, Undecidable:33}` | 0 decided, 144 refused, 109 rounding-introduced | same | same |
| curved pair | 144 decided, 0 refused | 144/0 | 144/0 | 144/0 |

Both runs finished in 4.4-11.4 seconds (`taskset -c 0-5 mise exec -- just t
-p rcad-step --test a_fuse_read_back_from_rounded_text --test-threads 6`), the
first including a warm build. These numbers also match §191.3's own published
ones exactly, which is a second confirmation from a different run months
apart.

So this family is a single fixed regression check on one baked-in draw, not a
property that explores more the longer or more variously it runs. Every
`just gate` already runs it once; running it for hours finds nothing `just
gate` did not already show, by construction. It is out of scope for a
per-core-hour comparison — there is no "per additional core-hour" to measure,
only "per run," and a run is a few seconds. The row's soak budget went to the
two families that do take a fresh seed: the `over()`/`measured()` properties in
`fuses_that_must_agree.rs` (which includes §191.4's turned curved families)
and `fuses_that_touch.rs` (§191.5's touching family).

### 290.3 The chunks: seed, factor, wall time, CPU time, uptime

Every chunk: `PROPTEST_RNG_SEED=<seed> RCAD_SOAK_SEED=<seed>
RCAD_SOAK_FACTOR=<factor> taskset -c 0-5 /usr/bin/time -v mise exec -- just t
--test fuses_that_must_agree --test fuses_that_touch --test-threads 6
--no-fail-fast`, bracketed by `uptime`. Chunk 1 calibrated the factor (75 came
out at 10:17.62, too long for one call's 600s ceiling and the harness moved it
to the background); chunks 2-17 used 66, which lands close to nine minutes.
Two short calibration runs before chunk 1 (factor 1 with all three families,
13.42s; factor 20 with two families, 139.40s) are not counted in the totals
below — they are a different, non-comparable draw size, run only to find a
factor.

| # | seed | factor | wall (s) | CPU (s) | uptime start (1m,5m,15m load) | uptime end (1m,5m,15m load) | result |
|---|---|---|---|---|---|---|---|
| 1 | 101 | 75 | 617.62 | 2172.79 | 08:35:03, 23.02/13.33/11.37 | 08:45:20, 28.03/27.93/20.94 | pass |
| 2 | 102 | 66 | 515.62 | 1917.23 | 08:47:38, 30.34/29.72/22.61 | 08:56:14, 23.76/26.85/24.47 | pass |
| 3 | 103 | 66 | 551.83 | 1926.79 | 08:56:21, 24.82/27.02/24.53 | 09:05:33, 18.90/25.51/25.87 | pass |
| 4 | 104 | 66 | 508.18 | 1917.47 | 09:05:39, 17.84/25.06/25.72 | 09:14:07, 18.49/23.35/25.08 | pass |
| 5 | 105 | 66 | 472.91 | 1917.03 | 09:14:13, 18.05/23.18/25.02 | 09:22:06, 13.88/18.69/22.52 | pass |
| 6 | 106 | 66 | 450.60 | 1910.22 | 09:22:20, 13.53/18.38/22.36 | 09:29:50, 13.76/16.16/19.93 | pass |
| 7 | 107 | 66 | 451.09 | 1908.53 | 09:29:58, 13.70/16.11/19.90 | 09:37:29, 13.67/15.66/18.42 | pass |
| 8 | 108 | 66 | 451.85 | 1909.26 | 09:37:36, 13.41/15.54/18.35 | 09:45:08, 13.70/15.56/17.49 | pass |
| 9 | 109 | 66 | 453.08 | 1914.22 | 09:45:20, 13.24/15.37/17.40 | 09:52:53, 13.87/15.53/16.90 | pass |
| 10 | 110 | 66 | 452.95 | 1915.05 | 09:52:59, 13.58/15.41/16.85 | 10:00:32, 13.63/15.45/16.53 | pass |
| 11 | 111 | 66 | 449.73 | 1907.17 | 10:00:49, 13.59/15.35/16.48 | 10:08:19, 13.82/15.57/16.37 | pass |
| 12 | 112 | 66 | 404.37 | 919.77 | 10:08:22, 13.67/15.51/16.35 | 10:15:07, 13.11/14.24/15.57 | **2 fail** (fail-fast: only 29/90 tests ran) |
| 13 | 113 | 66 | 451.38 | 1906.85 | 10:16:46, 12.34/13.64/15.22 | 10:24:17, 13.61/15.04/15.52 | pass (`--no-fail-fast` from here on) |
| 14 | 114 | 66 | 452.15 | 1913.02 | 10:24:21, 13.48/14.99/15.50 | 10:31:53, 14.07/15.50/15.76 | pass |
| 15 | 115 | 66 | 452.03 | 1911.57 | 10:32:01, 13.91/15.42/15.73 | 10:39:33, 13.72/15.48/15.85 | **1 fail** (90/90 ran) |
| 16 | 116 | 66 | 451.33 | 1908.37 | 10:39:44, 13.46/15.37/15.81 | 10:47:16, 13.82/15.68/16.02 | pass |
| 17 | 117 | 66 | 451.71 | 1913.04 | 10:47:37, 13.30/15.44/15.93 | 10:55:09, 13.68/15.46/15.96 | pass |

Totals for chunks 1-17 (Q149's own run): wall 8038.43s (2.233 hours), CPU
31788.38s (8.830 CPU-hours) (`python3 -c "print(sum([...]))"` over the
columns above).

Chunk 12 ran under nextest's default fail-fast and was cut short at 29 of 90
tests once two failed, so its CPU time (919.77s) is genuinely smaller work
done, not a faster chunk — it is included in the totals as what actually ran,
not padded to a full chunk's worth.

Chunks 18-24 (seeds 118-124) were run by Q227 (`lane-q227-soak-to-three-hours`,
off `8a49bc9d`) on 2026-09-27, 16:43-19:35 UTC, to close the shortfall §290.6
named — same command line, same `RCAD_SOAK_FACTOR=66`, `taskset -c 0-5`,
`--no-fail-fast` throughout. Other lanes and gates ran concurrently on cores
6-23 for the whole stretch, which the recorded loads reflect.

| # | seed | factor | wall (s) | CPU (s) | uptime start (1m,5m,15m load) | uptime end (1m,5m,15m load) | result |
|---|---|---|---|---|---|---|---|
| 18 | 118 | 66 | 345.75 | 1404.37 | 18:43:37, 9.86/9.91/12.59 | 18:49:23, 11.63/12.10/12.84 | pass |
| 19 | 119 | 66 | 404.56 | 1514.46 | 18:49:34, 10.39/11.82/12.74 | 18:56:18, 20.44/19.09/15.98 | pass |
| 20 | 120 | 66 | 463.38 | 1629.08 | 18:56:27, 21.61/19.40/16.12 | 19:04:10, 18.77/23.78/20.25 | **1 fail** (90/90 ran) |
| 21 | 121 | 66 | 404.21 | 1615.36 | 19:05:08, 12.34/21.02/19.52 | 19:11:52, 12.75/19.34/19.86 | pass |
| 22 | 122 | 66 | 391.18 | 1573.99 | 19:12:01, 12.02/18.96/19.73 | 19:18:32, 18.71/20.04/20.12 | pass |
| 23 | 123 | 66 | 479.29 | 1657.44 | 19:18:41, 21.31/20.55/20.29 | 19:26:41, 15.84/27.24/24.90 | pass |
| 24 | 124 | 66 | 514.64 | 1717.97 | 19:26:50, 14.86/26.65/24.73 | 19:35:25, 22.34/29.61/28.08 | **1 fail** (90/90 ran) |

Totals for chunks 18-24: wall 3003.01s, CPU 11112.67s
(`python3 -c "print(345.75+404.56+463.38+404.21+391.18+479.29+514.64); print(1404.37+1514.46+1629.08+1615.36+1573.99+1657.44+1717.97)"`).

Grand totals, all 24 chunks: wall 11041.44s (3.067 hours), CPU 42901.05s
(11.917 CPU-hours) (`python3 -c "print(8038.43+3003.01); print(31788.38+11112.67)"`)
— the three wall-hours asked for are reached, with 241.44s (0.067 hours) to
spare. §290.6 says so and carries the two new test failures; §290.5 carries
the recomputed core-hours and rates.

### 290.4 The two findings

Neither is a proptest per-case counterexample — both assertions run after the
whole draw, over a tally, so proptest's own shrinker and
`.proptest-regressions` persistence never engage. `git status --short` was
empty after every chunk, including the two that failed: there was no tracked
file to copy out before a reset, because nothing wrote one. The reproducer for
each is the seed and factor pair recorded above.

**A. `every_certified_fuse_of_a_lattice_pair_is_decided_and_records_nothing`**
— `tally.share_of_answers_that_were_not_solids_must_be_within(0.022..=0.098)`
(`fuses_that_must_agree.rs:511,1456`). Chunk 12: "555 of 25401 fuses came back
not a solid (2.2%), outside the 2.20%-9.80%". Chunk 15: "556 of 25401 …
(2.2%) …". `555/25401 = 0.021850`, `556/25401 = 0.021889`
(`python3 -c "print(555/25401); print(556/25401)"`) — both a hair under the
0.022 floor, which is why the message's own rounded "(2.2%)" looks like it
contradicts "outside the 2.20%-9.80%": the display rounds to the boundary, the
float is just under it. The same denominator, 25401, came back both times, so
the case count itself is not what varied. Reproduce with `RCAD_SOAK_SEED=112`
or `115`, `RCAD_SOAK_FACTOR=66`.

**B. `every_fuse_a_turned_curved_fixture_refuses_has_an_answer_turned_from_the_upright_one`**
— `assert_eq!((bar.0, ball.0), (0, 0), …)` (`fuses_that_must_agree.rs:3281`).
Chunk 12 only: `left: (1, 0)`, `right: (0, 0)` — one turned-bar-through-a-block
configuration refused a fuse, where the same configuration turned from its
upright answer validates (exactly §191.4's known gap shape: "every refusal of
the turned bar and ball is a gap"). The comment on the assertion says "Since
§202.2 the turned bar and ball refuse nothing on the shipped corpus, so only
the frustum still has refusals to demonstrate" — this run drew
`measured_cases() = 128 * 66 = 8448` cases (`python3 -c "print(128*66)"`) from
`RCAD_SOAK_SEED=112`, a corpus far past the shipped one, and found one
turned-bar case the shipped corpus never draws. It is not a new *kind* of
wrong answer — it is the same gap §191.4 already names, just not one the
shipped corpus had shown for the bar. §202.2's "refuses nothing" needs
scoping to the shipped corpus size, or needs re-checking against a larger one.

**Grouping.** Two distinct findings: different test, different assertion,
different code path (a statistical tally band vs. a turned-vs-upright
agreement check). A recurred twice (chunks 12 and 15) at nearly the same
fraction both times, which reads as one systematic near-boundary condition,
not two independent rare draws — counted once as a finding, with both
occurrences kept as evidence in §290.4 rather than folded away. B occurred
once, in fifteen chunks that exercised it. Three occurrences, two distinct
findings.

**Q227's seven chunks (seeds 118-124, §290.3) add two more failures, both the
same shape as A, one of them literally A.**

Chunk 24 (seed 124) is a third occurrence of finding A itself, same test,
same call site: `543 of 25401 fuses came back not a solid (2.1%), outside the
2.20%-9.80%` — `543/25401 = 0.021377`
(`python3 -c "print(543/25401)"`), again a hair under the 0.022 floor, at the
same 25401 denominator as chunks 12 and 15. §290.6's own prediction — "a
third would make 'the band's lower edge is too tight at this sample size' the
better-supported reading over 'a rare fluke'" — is what this recurrence bears
out; the test and the band are Q228's to change, not this row's.

Chunk 20 (seed 120) is not finding A — it fails a different named test,
`how_often_a_certified_fuse_of_a_turned_pair_refuses` — but the shape is
identical down to the denominator. That test's band constant,
`A_TURNED_PAIR_IS_NOT_A_SOLID` (`fuses_that_must_agree.rs:563`), is numerically
the same range as A's, `0.022..=0.098`
(`grep -n "A_LATTICE_PAIR_IS_NOT_A_SOLID\|A_TURNED_PAIR_IS_NOT_A_SOLID" crates/rcad-topo/tests/fuses_that_must_agree.rs`),
and the panic comes from the same shared method,
`share_of_answers_that_were_not_solids_must_be_within`
(`fuses_that_must_agree.rs:4575`), just called from a different site
(`fuses_that_must_agree.rs:1528` rather than `:1456`): `558 of 25401 fuses
came back not a solid (2.2%)`, `558/25401 = 0.021968`
(`python3 -c "print(558/25401)"`) — again just under the same 0.022 floor, at
the same 25401 denominator. Whether this counts as a fourth occurrence of A,
a new finding of its own, or evidence that the 0.022 floor is generally too
tight rather than tight for the lattice-pair family specifically, is left to
whoever next owns the test and the band (Q228) and the two core-hour measures
(Q230); this row only reports the count and the reproducer
(`RCAD_SOAK_SEED=120`, `RCAD_SOAK_FACTOR=66`).

No occurrence of finding B in these seven chunks.

### 290.5 Core-hours two ways, and failures per core-hour

Wall-time-based: `2.233 hours × 6 cores = 13.397 core-hours`
(`python3 -c "print(8038.43/3600*6)"`). CPU-time-based:
`31788.38s / 3600 = 8.830 core-hours` (already core-seconds summed across
however many of the six cores were busy at once, so no `×6`).

Two distinct findings over that budget:

- wall-based: `2 / 13.397 = 0.1493` distinct failures per core-hour
- CPU-based: `2 / 8.830 = 0.2265` distinct failures per core-hour
- (counting all three occurrences instead of two distinct findings: `0.2239`
  wall-based, `0.3397` CPU-based — given for completeness, not used below)

The two disagree by 34% because the chunks never used all six allotted cores:
`/usr/bin/time`'s "Percent of CPU" ranged 349-423% (never near 600%) across
every chunk, calibration runs included. It correlates loosely with the system
load average recorded alongside it — chunks 1-5, while the system-wide 1-minute
load average was 17-30, ran at 349-405% CPU; chunks 6-17, once it settled to
13-15, ran at 422-424% consistently. That is a correlation this run measured,
not a proven cause: `taskset -c 0-5` pins this process's own threads to cores
0-5, but nothing here checked whether other processes were also scheduled onto
those same six cores during the busier early chunks, or whether the gap is
instead this test workload's own uneven parallelism (a handful of long-running
properties, such as chunk 15's 74s lattice-pair check or the turned-frustum
property that ran into the SLOW markers past 300s, dominating wall time alone
while the other five threads sit idle). Both are live explanations; this
measurement does not separate them.

**Over all 24 chunks** (adding Q227's seven, seeds 118-124, §290.3): wall-based
`11041.44s / 3600 × 6 = 18.402` core-hours
(`python3 -c "print(11041.44/3600*6)"`), CPU-based `42901.05s / 3600 = 11.917`
core-hours (`python3 -c "print(42901.05/3600)"`). The two now disagree by
35.2% (`python3 -c "print((18.402-11.917)/18.402*100)"`), against 34% at 17
chunks — a small further widening, not a reversal. This row does not chase why
(Q230 owns that); it only adds the seven new chunks' own "Percent of CPU"
figures — 406%, 374%, 351%, 399%, 402%, 345%, 333% — which widen the range
already named above from 349-423% to 333-423%, still never near 600%, and do
not by themselves separate the two live explanations already given.

Findings over the full 24-chunk budget: three occurrences of finding A
(chunks 12, 15, 24), one of finding B (chunk 12), and one occurrence of the
same near-floor shape on a different test (chunk 20, turned-pair rather than
lattice-pair; §290.4 has the detail and does not resolve how it groups).
Counting that last one as its own third distinct finding (different named
test, same shared assertion) gives three distinct findings and five
occurrences:

- wall-based: `3 / 18.402 = 0.1630` distinct failures per core-hour
  (`python3 -c "print(3/18.402)"`)
- CPU-based: `3 / 11.917 = 0.2517` distinct failures per core-hour
  (`python3 -c "print(3/11.917)"`)
- counting all five occurrences instead of three distinct findings:
  `5 / 18.402 = 0.2717` wall-based, `5 / 11.917 = 0.4196` CPU-based
  (`python3 -c "print(5/18.402); print(5/11.917)"`)
- if chunk 20 is instead folded into finding A (same assertion, same band
  value, only the family differs), two distinct findings and five occurrences
  give `2 / 18.402 = 0.1087` wall-based, `2 / 11.917 = 0.1678` CPU-based
  (`python3 -c "print(2/18.402); print(2/11.917)"`) — the occurrence-based
  rates above are unchanged either way.

Every one of these rates is higher than the 17-chunk numbers (0.1493/0.2265
for two distinct findings, three occurrences), because the seven new chunks
added a third distinct-or-folded finding and two more occurrences on only a
37% increase in core-hours. Whether that means the rate is settling upward or
is still noise from a small sample is not something seven more chunks
settles either — it is the same open question §290.8 already named, now with
one more data point on each side of it.

### 290.6 What is short of three hours, and how to finish it

2.233 of the requested 3 wall-hours ran in Q149's own session (74%), 17
chunks. The shortfall was that session's own time budget, not a limit in the
method: chunk 1's calibration overshot to 617.62s deciding the factor, and
running the remaining ~46 minutes (6-7 more chunks at the same settings) was
not reached before that row had to close.

**Closed by Q227** (`lane-q227-soak-to-three-hours`, off `8a49bc9d`,
2026-09-27 16:43-19:35 UTC): seven more chunks, seeds 118-124, run exactly to
§290.3's command line — same `RCAD_SOAK_FACTOR=66`, same `taskset -c 0-5`,
`--no-fail-fast` throughout. Wall total across all 24 chunks: 11041.44s =
3.067 wall-hours (`python3 -c "print(11041.44/3600)"`) — **the three hours
asked for are reached**, 0.067 hours (241.44s) past it. §290.3 carries the
seven new rows; §290.5 carries the recomputed core-hours and rates.

Finding A recurred a third time, in the last of these seven chunks (seed 124,
chunk 24 in §290.3's table): `543 of 25401 fuses came back not a solid
(2.1%), outside the 2.20%-9.80%` — `543/25401 = 0.021377`
(`python3 -c "print(543/25401)"`), again a hair under the 0.022 floor, at the
same 25401 denominator as both earlier occurrences. This row's own prediction
— "a third would make 'the band's lower edge is too tight at this sample
size' the better-supported reading over 'a rare fluke'" — is what this
recurrence bears out: three for three of the chunks that have drawn this
sample size and landed near the floor at all. Q228 owns the test and the
band; this row only reports the count.

One more data point outside what was asked for: chunk 20 (seed 120) produced
the same near-floor shape, but on the *turned*-pair test
(`how_often_a_certified_fuse_of_a_turned_pair_refuses`), whose band constant
(`A_TURNED_PAIR_IS_NOT_A_SOLID`, `fuses_that_must_agree.rs:563`) is
numerically the same range as A's, `0.022..=0.098`, and whose panic comes
from the same shared method (`share_of_answers_that_were_not_solids_must_be_within`,
`fuses_that_must_agree.rs:4575`) as A, just called from a different site:
`558 of 25401 fuses came back not a solid (2.2%)`, `558/25401 = 0.021968`
(`python3 -c "print(558/25401)"`). §290.4 has the full detail and does not
decide how it groups with A; that is Q228's and Q230's to settle, not this
row's.

No new occurrence of finding B in these seven chunks, and no other test
failed.

### 290.7 Set beside the fuzz targets (§265.6, §265.7, §226.3)

§226.3's table gives the only fuzzing stretch this row can compare against
fairly: 09-22 to 09-25, four nights, each already running the "new way" (five
targets, `-fork=4`, eight hours, about 160 core-hours a night per §226.3's own
figure) — nothing new was found in those four nights, after §140's and
§181's families had already been found and, mostly, fixed. That is
`0 distinct faults / (4 × 160) core-hours = 0` per core-hour, over roughly
640 core-hours, in a "mature, nothing new being written" state — the same
shape this row measured for the property families: existing code, fresh
seeds, no new test or fixture written between chunks.

Against that: `fuses_that_must_agree.rs` and `fuses_that_touch.rs`, unchanged
since before this row started, found two distinct issues in 8.830 CPU-hours
(0.2265/core-hour) to 13.397 wall-core-hours (0.1493/core-hour) — a
nonzero rate where the comparable fuzzing stretch measured zero, on a budget
two orders of magnitude smaller (9-13 core-hours against ~640).

### 290.8 What the comparison can and cannot tell

**What it can tell.** In a state where nothing new is being written, these
two already-existing property families kept finding at a measurably nonzero
rate over a budget of single-digit core-hours, where the comparable fuzzing
stretch found nothing over roughly two orders of magnitude more core-hours.
§265.6 already named the likely reason: the writer and reader fuzz targets
check only that nothing panicked, so a wrong-but-not-crashing answer — which
is what both of this row's findings are — is invisible to them by
construction, no matter how long they run. A property family that knows the
right answer (a band a family has always kept to, an upright answer a turned
one must agree with) can catch exactly that kind of miss, and this row is a
first measurement of how often, not a guess.

**What it cannot tell.** The rate sits on two events, both surfacing in the
same one of seventeen chunks (12) and nowhere in the other sixteen —
consistent with a rare or clustered condition rather than a steady one, so
0.15-0.34 per core-hour is a point estimate with wide uncertainty, not a rate
to plan an allocation around. §290.1's split matters here too: most of a
chunk's added core-hours (the `RCAD_SOAK_FACTOR`-scaled `measured()`
properties and the seventeen touching cells) bought a bigger draw, but the
thirty-four `over()`-based properties — many of §191.4's and §279's turned and
leaning-bar laws among them — kept the same fixed case count every chunk, so
more chunks bought them a different draw, never a bigger one; a rate computed
over "hours run" is not the same thing as a rate computed over "cases drawn"
for that half of the suite. The fuzz targets' zero, on the other side, is
partly an artifact of what they check (§265.6) and partly of which code they
reach — cone and torus crossings, the STEP reader and writer — which is mostly
disjoint from what these two topology-only test files exercise, so a
zero-versus-nonzero comparison is measuring different attack surfaces as much
as it is measuring different techniques. And §290.5's 34% gap between the
two core-hour measures, and its correlation with system load that this run
did not resolve into a cause, means neither "0.15" nor "0.23" is the row's own
settled number for "per core-hour" until that is pinned down.
