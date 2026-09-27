## 291. The reader to job 300, at two max_lens: three hours costs far less than fifteen minutes predicted, and 131072 roughly halves the rate for a small coverage gain — 2026-09-27

*Queue rows Q155 and Q178, from §258.8 item 2 and §274.4. Lane
`lane-q155-reader-to-job-300`, off `566db6f1`. It runs §264's arm B and arm D2
(the whole STEP corpus, read-only, last) at `-max_len=65536` and again at
`-max_len=131072`, each `-fork=4`, each to job 300 (about three hours), side by
side on this machine's cores 6-17, each with its own copy of both corpus
directories so no two arms write one directory. The reader binary is unchanged
since the base commit (`git diff --stat 566db6f1 -- fuzz/
crates/rcad-step/src` prints nothing). Every number below was run on this
machine on 2026-09-27.*

  291.1   What was run
  291.2   Steady exec/s to job 300
  291.3   What each arm's first, writable directory gains per hour
  291.4   Coverage at job 300
  291.5   Q178: is 131072's extra coverage worth its rate

### 291.1 What was run

Three arms, launched at the same moment (06:33:11Z, `ps -o lstart` on all
three), each `-fork=4 -timeout=10 -ignore_crashes=1 -ignore_timeouts=1
-ignore_ooms=1 -dict=step.dict`, each with `RCAD_FUZZ_TIMINGS` set to its own
directory, each pinned with `taskset` to its own four cores, each given a
fresh copy of the tracked 909-file reader corpus (215,292 bytes, `du -sb
fuzz/corpus/no_step_file_makes_the_reader_panic`) as its first, writable
directory:

- **B**: `taskset -c 6-9`. Nothing else — the reader as it runs today.
- **D2 at 65536**: `taskset -c 10-13`. Its own copy of `RCAD_STEP_CORPUS`
  (520 files, 392,005,383 bytes — `du -sb`, matching §264.1's count exactly)
  as a second, read-only directory, last, and `-max_len=65536`.
- **D2 at 131072**: `taskset -c 14-17`. The same, with `-max_len=131072`.

Each ran to past job 300 and was then stopped (`kill -TERM` on its process
group, confirmed dead and no artifacts left behind): B at job 301 (11,609 s on
its own clock), D2/65536 at job 303 (11,849 s), D2/131072 at job 302
(11,863 s). No arm found a crash, a timeout, or an OOM
(`oom/timeout/crash: 0/0/0` on every line; no files under any arm's
`artifacts/`). Cores 0-5 and 18-21 held two other measuring lanes throughout;
the one-minute load average read 7.18 at launch, climbed as high as 25.88
around the one-hour mark (other lanes gating) and was back to 1.21 by the end
— recorded hourly (`uptime`):

| | uptime | load average (1/5/15 min) |
|---|---|---|
| start, 06:32:48Z | up 1 day, 14:20 | 7.18 / 5.22 / 8.76 |
| +1h, 07:30:05Z | up 1 day, 15:17 | 14.36 / 16.17 / 19.88 |
| +2h, 08:34:25Z | up 1 day, 16:22 | 17.69 / 16.43 / 16.07 |
| +3h, 09:37:21Z | up 1 day, 17:25 | 12.06 / 12.11 / 12.83 |
| end, 09:53:04Z | up 1 day, 17:40 | 1.21 / 6.50 / 9.99 |

### 291.2 Steady exec/s to job 300

Same measure as §258 and §264: the fork parent's own cumulative execution
count, from the first job line past 600 s to the last.

| | B | D2/65536 | D2/131072 |
|---|---|---|---|
| execs, 600 s line to last | 275,172,416 | 15,267,659 | 7,678,315 |
| time, 600 s line to last | 11,002 s | 11,245 s | 11,253 s |
| execs/s | 25,013 | 1,358 | 682 |
| share of B | — | 5.4% | 2.7% |
| share of D2/65536 | — | — | 50.2% |

§264.4's fifteen-minute figure for D2/65536 was 9.1%-9.5% of B's rate. Over
three hours to job 300 it is 5.4% — lower, not higher, than the short
measurement suggested. §258.4 expected the opposite: a 300 s job's seed replay
is a smaller share of its own time than a 60 s job's, so a long run should
look relatively better for the STEP-seeded arm, not worse. It does not, here.
A likely reason: D2's own kept corpus keeps growing across three hours (2,501
and 2,425 files by the end, against 989-998 in §264.2's fifteen-minute round),
and libFuzzer's `SkewTowardsLast` (§258.3) draws seeds more often from the
largest, most recently-kept files — so a later job's replay and mutation both
skew toward bigger inputs than an early job's did. That is a plausible
mechanism, not a traced one; nothing here isolated it from the alternative
that later 300 s jobs simply spend a bigger fraction of their budget on the
one slow input that saturates `-timeout=10`.

131072 costs about half of 65536's already-slow rate: 682 execs/s against
1,358, for the reason §258.4 gives directly — mutations at 131072 can grow to
twice the size, and a real STEP file at that length costs more to parse and
validate per execution.

### 291.3 What each arm's first, writable directory gains per hour

Baseline for every arm: 909 files, 215,292 bytes, from the same process start,
06:33:11Z. Measured at each arm's stop (`du -sb`):

| | B | D2/65536 | D2/131072 |
|---|---|---|---|
| stopped at | 09:46:41Z | 09:50:57Z | 09:50:57Z |
| elapsed | 3.225 h | 3.296 h | 3.296 h |
| files | 1,989 | 2,407 | 2,259 |
| bytes | 3,545,044 | 61,354,396 | 90,512,572 |
| bytes gained | 3,329,752 | 61,139,104 | 90,297,280 |
| **bytes/hour** | **≈1.03 MB/h** | **≈18.5 MB/h** | **≈27.4 MB/h** |

§264.6 measured D2/65536 writing 24 to 28 MB in its first quarter hour, which
extrapolates to roughly 96-112 MB/h if that rate held. Over the full three
hours to job 300 it instead averages about 18.5 MB/h — a sixth of that
extrapolation. The writes are heavily front-loaded: by the one-hour mark it had
already written about 41 MB (07:30:05Z checkpoint), well over half of its
three-hour total of 61 MB. §264.6's fifteen-minute figure was correct for its
own fifteen minutes; it overstates what a night keeps once the run is long
enough to reach job 300, in the direction §258.4 predicted for the byte cost,
even though §291.2's rate moved the other way.

D2/131072 gained bytes at about 1.5 times D2/65536's rate (27.4 against
18.5 MB/h) — unsurprising, since every kept find can be up to twice as large.

### 291.4 Coverage at job 300

| at job 300 | B | D2/65536 | D2/131072 |
|---|---|---|---|
| cov (edges) | 4,681 | 12,246 | 12,485 |
| ft (features) | 13,528 | 52,228 | 57,720 |

D2/131072 over D2/65536 at job 300: 239 more edges (2.0%) and 5,492 more
features (10.5%). §274.2's merge-only sweep (no fuzzing at all) found a bigger
gap at the same two `-max_len` values: 495 more edges (4.8%) and 10,464 more
features (34.7%). Three hours of fuzzing narrows the gap 131072 opens at the
merge, in both edges and features, without closing it.

### 291.5 Q178: is 131072's extra coverage worth its rate

**No — the pin should stay at 65536.** 131072 costs about half of 65536's
already-slow rate (291.2: 682 against 1,358 execs/s, 2.7% of B against 5.4%)
and gains bytes at about 1.5 times its rate (291.3: 27.4 against 18.5 MB/h —
more of the night's corpus budget spent on inputs the next night must replay).
What it buys, measured the way this row measured everything else — after
three hours of real fuzzing, not a merge alone — is 2.0% more edges and 10.5%
more features than 65536 already has at job 300 (291.4). §274.2's merge-only
sweep advertised a bigger gain (4.8% edges, 34.7% features) because it
measured the two `-max_len` values before any fuzzing had a chance to narrow
the difference. Paying half the rate for two points of edges and ten of
features is not a good trade, and it is a worse one than §274.4 already
thought it might be. §264.7's 65,536 pin stands.
