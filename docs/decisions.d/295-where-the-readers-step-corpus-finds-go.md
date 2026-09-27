## 295. Where the reader's STEP-corpus finds go: merging them back at a smaller max_len keeps almost all of the speed and a fifth of the coverage gain, for a hundredth of the bytes — 2026-09-27

*Queue row Q157, from §264.8 item 2. Lane `lane-q157-where-reader-finds-go`,
off `531ca665`. Answers which of three destinations the reader's
`-max_len=65536`, STEP-corpus-seeded finds (§264, §291) should go to: a
directory the next night does not start from, a `-merge=1` of them back into
the tracked corpus at a smaller `-max_len`, or keeping the 64 KiB finds as the
writable directory itself, as arm D2 does today. It reuses §291's own D2/65536
output (job 303, a snapshot of 2,407 files, 61,354,396 bytes — the same
`du -sb` figure §291.3 reports) as "the finds," copied out of the Q155 lane's
scratchpad rather than rerun, and never written into. Every number below was
run on this machine on 2026-09-27, `taskset -c 0-5`, sequentially. Cores 6-17
ran other lanes' gates throughout, and the one-minute load average climbed
from 5.83 to 37.94 over the course of these runs (295.1's table) — high enough
that the *absolute* exec/s figures here are not comparable to §291's quieter
three-hour ones, but the three policies were run back to back under the same
rising contention, so the figures compare fairly to each other.*

  295.1   What was run
  295.2   Bytes carried to the next night
  295.3   What a night starts with and ends with
  295.4   Steady exec/s
  295.5   A second cycle under the merge policy
  295.6   The choice
  295.7   What the choice asks of fuzz-overnight, fuzz-reader and fuzz-target

### 295.1 What was run

Three writable directories, each seeded, then each given one 420 s
`-max_total_time` burst (`-fork=4 -max_len=65536 -timeout=10
-ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 -dict=step.dict`, the
reader target, `RCAD_STEP_CORPUS` — 520 files, 392,005,383 bytes, matching
§291.1 exactly — passed as the second, read-only directory, same as arm D2):

- **A, discard**: seeded with a fresh copy of the tracked 909-file corpus
  (215,292 bytes). Its burst's output is never kept; a repeat of this policy
  reseeds from the same 909 files every night.
- **B, merge at a smaller max_len**: seeded with a fresh copy of the same
  909 files, then folded with D2's finds by running the target with
  `-merge=1 -max_len=5110` — 5,110 because that is arm B's own derived cap
  (§264.7), so a night with no STEP corpus fetched still derives the same
  small mutation ceiling it does today, which was §264.7's objection to
  pinning 65,536 anywhere but `fuzz-target`'s STEP-corpus branch. The merge
  kept 84 of D2's files (of 2,407 offered, plus the 909 already there —
  `MERGE-OUTER: 3316 files, 909 in the initial corpus`) for 301 new edges and
  2,301 new features, landing at 993 files, 499,607 bytes. The merge's own
  libFuzzer phase finished in about a second (`DONE exec/s: 3316` on 3,316
  files); the whole command, including a cold build, took 1m08s.
- **C, keep the finds**: seeded with a copy of D2's 2,407-file, 61,354,396-byte
  output directly, matching what arm D2 already hands the next night today.

Uptime and load (`uptime`), before and after each burst:

| | UTC | uptime | load average (1/5/15 min) |
|---|---|---|---|
| before A | 15:33:33Z | up 1 day, 23:21 | 5.83 / 2.89 / 1.32 |
| before B (= after A) | 15:41:41Z | up 1 day, 23:29 | 16.48 / 14.87 / 7.81 |
| after B | 15:48:56Z | up 1 day, 23:36 | 37.94 / 22.36 / 13.13 |
| before C | 15:49:34Z | up 1 day, 23:37 | 35.23 / 23.55 / 13.88 |
| after C | 15:56:55Z | up 1 day, 23:44 | 16.03 / 23.30 / 18.32 |

No crash, timeout or OOM in any burst (`oom/timeout/crash: 0/0/0` throughout;
no files under any burst's `artifacts/`). Each burst is one night's worth of
signal at 420 s, not §291's three hours to job 300 — long enough to compare
the three policies' shape, not to re-derive their multi-hour steady state.

### 295.2 Bytes carried to the next night

What each policy's writable directory holds **before** a night's fuzzing
starts — the cost the next night's corpus load pays, and what a copy of it
weighs if it needs to move with the corpus (backup, another machine, CI):

| | A, discard | B, merge at 5110 | C, keep the finds |
|---|---|---|---|
| files | 909 | 993 | 2,407 |
| bytes | 215,292 | 499,607 | 61,354,396 |
| bytes beyond the tracked corpus | 0 | 284,315 | 61,139,104 |

A never grows past the tracked corpus. B grows by a few hundred KB per night
that runs a merge (295.5 measures a second cycle). C already carries 61 MB
after one three-hour run and, per §291.3, keeps adding **≈18.5 MB/h** to it,
averaged over three hours and front-loaded — more than half of that in the
first hour alone.

### 295.3 What a night starts with and ends with

libFuzzer's `-fork` mode reports its own seed count and coverage on the first
job line, after an internal cross-pollination pass across every directory
given it (both the writable one and the read-only STEP corpus) — the
closest fork-mode equivalent to a single-process run's `LOADED` line:

| | A, discard | B, merge at 5110 | C, keep the finds |
|---|---|---|---|
| seed inputs (fork's own count) | 983 | 1,056 | 2,036 |
| starts at: cov / ft | 10,253 / 32,689 | 10,439 / 33,975 | 11,672 / 50,986 |
| ends at (420 s later): cov / ft | 10,978 / 39,278 | 10,813 / 37,843 | 11,677 / 51,045 |
| corp at end | 1,556 | 1,577 | 2,058 |
| directory bytes at end | 20,787,292 | 19,454,145 | 62,354,015 |

Two things this table shows directly:

- **C's own 2,407 files fold down to 2,036 seeds** — about 15% of them add no
  coverage the fork-mode merge doesn't already find elsewhere in the same
  set. Keeping the raw finds keeps that redundancy too; a `-merge=1` (as B
  does deliberately, and as C's own fork-mode startup does for free but
  without shrinking the directory on disk) is the only one of the three that
  removes it.
- **A single 420 s night, from either A's or B's small starting point, closes
  most of the gap to C's three-hour-compounded state**: A ends at 10,978/39,278
  against C's *starting* 11,672/50,986, and C's own 420 s of further fuzzing
  from there only adds 5 edges and 59 features (11,677/51,045). C has already
  spent its easy gains; A and B are still finding theirs quickly.

### 295.4 Steady exec/s

Cumulative executions from the first job line to the last, divided by the
elapsed time between them (same construction as §258/§264/§291.2, over this
run's shorter span):

| | A, discard | B, merge at 5110 | C, keep the finds |
|---|---|---|---|
| first job | #2,218 at 11s | #1,454 at 15s | #46 at 119s |
| last job | #1,007,970 at 430s | #1,000,248 at 421s | #338,581 at 425s |
| execs/s | 2,400 | 2,460 | 1,107 |
| share of A | — | 102% | 46% |

C's first job alone takes 119 s — most of it the cost of loading and
replaying a 61 MB directory before any real mutation starts, the same cost
§258.4 and §291.2 describe. A and B, starting under 500 KB, are fuzzing
within 11-15 s and run at better than twice C's rate throughout. B costs
nothing measurable next to A for keeping its 84 extra files.

### 295.5 A second cycle under the merge policy

To see whether the merge-at-5110 policy keeps finding new small files on a
second night, or exhausts them after one, B's own 420 s burst output (1,512
files, 19,454,145 bytes — some of them up to 64,501 bytes, since the burst
itself still ran at `-max_len=65536`) was merged again at `-max_len=5110`.
**This second merge target was seeded fresh from the 909-file tracked corpus,
not chained onto the first merge's 993-file result**, so its count is not a
clean net-new figure over 295.1's 84 files — it can only be read as "what a
repeat of this cycle costs and finds," not as proof of accumulation:

`MERGE-OUTER: 131 new files with 2692 new features added; 358 new coverage edges`

Result: 1,036 files, 637,253 bytes — 131,646 bytes and 43 more files than the
first cycle's 993/499,607. Whether some of those 131 files duplicate the
first cycle's 84 is not measured here; what is measured is that a second
cycle costs the same shape (low hundreds of KB, under 150 files) as the
first, not an escalating one.

### 295.6 The choice

**Merge the finds back at a smaller `-max_len` (B).** Discarding (A) is
close behind B on every number in 295.3 and 295.4, but it throws away real,
reproducible small-input coverage for nothing: B keeps 84 files (284 KB) that
A does not, at a measured exec/s cost of zero (295.4: 2,460 against 2,400,
inside this measurement's noise) and a `-merge=1` cost of about a second of
libFuzzer time. Keeping the finds (C) is the one that costs something real —
61 MB today and every 18.5 MB/h after (§291.3) — for a per-night speed a
fraction of A's or B's (295.4: 1,107 execs/s, 46% of A's) and a coverage lead
that a single fresh night nearly closes anyway (295.3). §264.7's own worry
about C — "those finds join every later night's merge and seed pool, and the
size order in §258.3 draws seeds from the largest" — is confirmed directly
here: C's fork-mode startup already treats 15% of its own kept files as
redundant (295.3), and its per-hour byte growth falls by more than half
between a fresh corpus and a three-hour-old one (295.2 against this section's
own 420 s deltas: ≈172 MB/h fresh, ≈8.5 MB/h from C's already-grown start —
arithmetic in 295.2's bytes-at-end row). A large kept corpus is not just
slower to fuzz from; it is slower to grow, too.

### 295.7 What the choice asks of fuzz-overnight, fuzz-reader and fuzz-target

Today `fuzz-target`'s `corpus_root` directory is both what a night writes to
*and* what the next night starts from (§264.7 already puts the STEP-corpus
branch's `-max_len=65536` there). Under this choice that directory becomes a
**scratch** one, thrown away after each night once merged, and a second,
small, tracked-style directory becomes what nights actually start from:

1. **`fuzz-target`** needs a nightly writable directory that is not the one
   handed back next time — a fresh or truncated `corpus_root` per invocation,
   the same fresh-start `-max_len=65536` behavior this section's A and B
   arms used, not C's reused one.
2. It then needs a `-merge=1 -max_len=5110` step, reader-target-specific
   (5,110 is arm B's own derived cap, not a general one — §264.7), from that
   night's scratch output into the persistent small corpus, after the run and
   before the scratch directory is discarded.
3. **`fuzz-reader`** does not need its own `-max_len` flag (§264.7's
   conclusion still holds); it needs to pass the new small persistent
   directory as the seed and the scratch directory as where the night writes.
4. **`fuzz-overnight`** needs to run that merge step once per night that
   fetched the STEP corpus, and to make sure the scratch directory it discards
   is actually discarded — not left to accumulate as an unpinned, growing
   `corpus_root` under a different name.

This is a `justfile` change, and Q156 and Q161 already hold `fuzz-reader`,
`fuzz-target`, `fuzz-overnight` and `check-fuzz-overnight-selftest`; it is not
made here. The row for it goes in `rows_to_add`.
