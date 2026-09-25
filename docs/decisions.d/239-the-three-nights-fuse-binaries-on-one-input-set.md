## 239. The three nights' fuse binaries on one input set: the 09-23 rewrite runs the same inputs as fast as 09-22 — 2026-09-25

*Runs §234.5's first open item — leg (c) — by §227.4's method: build the fuse
fuzz binary at each of the three nights' commits, replay the same fixed inputs
on each, swap the order to cancel load. Read-and-measure only; nothing in the
kernel, the nightly tree or the corpus was written, and no binary of this lane
ran after 21:50Z.*

  239.1   What was built, and why the same bytes make the same solids on all three
  239.2   The input sets
  239.3   How it was timed
  239.4   The table
  239.5   File by file, 09-22 against 09-23
  239.6   What it shows for §234.4's leg
  239.7   What the 09-21 column says
  239.8   What was not run

### 239.1 What was built, and why the same bytes make the same solids on all three

Three binaries, built in this lane's worktree the way `fuzz-target` builds
them — `git checkout --detach <sha>`, then `cd fuzz && RUSTFLAGS="-C
target-cpu=x86-64-v3" mise exec -- cargo +nightly fuzz build
two_solids_fused_by_a_stranger`, then
`fuzz/target/x86_64-unknown-linux-gnu/release/two_solids_fused_by_a_stranger`
copied out to scratch space:

| night | commit | build id (`file <binary>`) |
|-------|--------|----------------------------|
| 09-21 | `ae124a29` | `1444c30c…` |
| 09-22 | `79eedce7` | `8e8efb1d…` |
| 09-23 | `055aa4a9` | `049fac09…` |

The night each commit belongs to is the nightly tree's own record, `git -C
/home/mattias/git/mattiasgronlund/rcad-nightly-fuzz reflog --date=iso-strict`:
`ae124a29` checked out at `2026-09-21T00:10:27+02:00`, `79eedce7` at
`2026-09-22T00:10:28+02:00`, `055aa4a9` at `2026-09-23T00:10:28+02:00` — the
same three §234.1 names.

**The same bytes build the same two solids on all three.** The target turns
its bytes into two models in `crates/rcad-topo/tests/fuzzing/fuse.rs` (and
`the_door.rs` beside it), reached by `#[path]`, and
`git diff --stat ae124a29 055aa4a9 -- fuzz crates/rcad-topo/tests/fuzzing`
prints nothing. So what differs between the three runs of one input is the
kernel under that code, not the solids it is handed.

### 239.2 The input sets

Copied — never moved — out of the two directories each night reads,
`/home/mattias/rcad-fuzz-corpus/two_solids_fused_by_a_stranger` and
`/home/mattias/git/mattiasgronlund/rcad-nightly-fuzz/fuzz/corpus/two_solids_fused_by_a_stranger`,
split by mtime at the three checkout times above, **all in UTC**:

- **A, before the 09-21 night**: `! -newermt '2026-09-20 22:10:27 UTC'` —
  9 986 files.
- **B, added during the 09-21 night**: `-newermt '2026-09-20 22:10:27 UTC' !
  -newermt '2026-09-21 22:10:28 UTC'` — 169 files.
- **C, added during the 09-22 night**: `-newermt '2026-09-21 22:10:28 UTC' !
  -newermt '2026-09-22 22:10:28 UTC'` — 923 files.

Every file was run: N = 1, no subsampling, because all three sets fitted the
time.

**A trap, checked rather than assumed.** The first split passed the times to
`find -newermt` without a zone, and this machine's zone is Europe/Stockholm, so
`find` read every edge two hours early (20:10Z). Rebuilt with the zone written
out, the three sets are identical name for name, because neither directory
holds a single file with an mtime between 20:10Z and 22:10Z on 09-20, 09-21 or
09-22 (`find <dir> -maxdepth 1 -type f -newermt '<day> 20:10:27 UTC' !
-newermt '<day> 22:10:28 UTC' | wc -l` — 0 for all six). Anyone repeating this
should write `UTC` into the cutoff anyway.

One file name is in both A and C — an old copy in one directory and a later
mtime in the other, §240's rewrite. §240.7 says the "added" sets B and C may
hold some older content for that reason. That moves which night a file is
credited to; it cannot move anything below, because every comparison here is
of two binaries on the **same** files.

### 239.3 How it was timed

Each binary is handed a batch of files by name, so libFuzzer runs each once
and prints `Executed <file> in N ms`; those lines are summed per batch
(`grep Executed | awk '{s+=$(NF-1)} END {print s}'`). Batches of 500 files for
A, 30 for B, 200 for C. Inside a batch all three binaries run the same files,
and **the order rotates every batch** — 09-21, 09-22, 09-23; then 09-22, 09-23,
09-21; then 09-23, 09-21, 09-22 — so each binary runs first, second and third
equally often and a change in machine load lands on all three alike. The
one-minute load average from `/proc/loadavg` is logged before every run.

Other sessions were building on this machine throughout: load read 2.3 to
16.3 during A, 10.1 to 30.5 during B, 10.9 to 14.3 during C. So, as in §227.4, **only the ratios mean anything**;
the absolute times are not comparable with a night's.

### 239.4 The table

Mean ms per input, each set's summed `Executed` times over its file count:

| input set | files | 09-21 (`ae124a29`) | 09-22 (`79eedce7`) | 09-23 (`055aa4a9`) | 09-22 / 09-21 | 09-23 / 09-22 |
|-----------|------:|------:|------:|------:|------:|------:|
| A, before 09-21 | 9 986 | 52.6 | 4.97 | 4.84 | 0.095 | 0.97 |
| B, added during 09-21 | 169 | 2 360 | 6.98 | 6.68 | 0.003 | 0.96 |
| C, added during 09-22 | 923 | 10.8 | 17.1 | 17.4 | 1.59 | 1.02 |

**The noise**, as the spread of the same ratio from one batch to the next
(every batch is the same files on each binary, so a ratio away from the
whole-set one is load, not code):

| input set | batches | 09-23 / 09-22, per batch | 09-22 / 09-21, per batch |
|-----------|--------:|--------------------------|--------------------------|
| A | 20 | 0.55 – 1.05, median 1.00 | 0.043 – 0.23, median 0.090 |
| B | 6 | 0.79 – 1.01, median 0.97 | 0.002 – 0.008, median 0.003 |
| C | 5 | 0.88 – 1.18, median 1.01 | 1.47 – 1.82, median 1.53 |

### 239.5 File by file, 09-22 against 09-23

A mean near 1 could hide a few inputs that got much slower, and §234 quotes
the nights' p99 moving from 48 ms to 67 ms. So every file of all three sets
was run again on the 09-22 and 09-23 binaries only, **twice**, keeping each
file's own `Executed` line rather than a batch sum — batches of 500, order
swapped every batch and swapped again between the two repeats. Load read 10.6
to 43.2 during this pass.

The tail, in ms per input:

| set | binary | repeat | mean | p90 | p99 | max |
|-----|--------|-------:|-----:|----:|----:|----:|
| A | 09-22 | 1 | 7.50 | 26 | 85 | 189 |
| A | 09-23 | 1 | 8.08 | 27 | 95 | 292 |
| A | 09-22 | 2 | 5.60 | 19 | 55 | 154 |
| A | 09-23 | 2 | 5.32 | 18 | 51 | 160 |
| B | 09-22 | 1 | 5.78 | 10 | 31 | 57 |
| B | 09-23 | 1 | 5.99 | 10 | 33 | 58 |
| B | 09-22 | 2 | 5.54 | 10 | 31 | 52 |
| B | 09-23 | 2 | 5.59 | 10 | 31 | 53 |
| C | 09-22 | 1 | 18.8 | 37 | 57 | 91 |
| C | 09-23 | 1 | 17.8 | 34 | 54 | 85 |
| C | 09-22 | 2 | 19.1 | 36 | 71 | 86 |
| C | 09-23 | 2 | 17.5 | 32 | 51 | 70 |

One binary moves further between its own two repeats than the two binaries
differ inside one repeat: A's p99 on the 09-22 binary is 85 ms, then 55 ms;
09-22 against 09-23 is 85 against 95, then 55 against 51.

**Files that moved.** A file was flagged if it was at least twice as slow on
one binary as the other in *both* repeats, and at least 10 ms on the slow side.
That flagged 12 of A's files as slower on 09-23 and none as faster; none either
way in B; in C none slower and 1 faster. For scale, one binary against itself —
repeat 2 against repeat 1 — puts 707 (09-22) and 785 (09-23) of A's files 2×
apart, so single runs here are loud.

The flagged files were then run ten more rounds on each binary, the order
alternating every round, beside 12 control files drawn from A with the same
range of 09-22 times (4 – 74 ms):

| files | 09-22, total of the per-file medians | 09-23, total of the per-file medians | per-file ratio |
|-------|------:|------:|---------------|
| the 12 flagged slower in A | 166 ms | 167 ms | 0.97 – 1.07 |
| the 1 flagged faster in C | 6 ms | 6 ms | 1.00 |
| 12 controls from A | 148 ms | 150 ms | 0.97 – 1.04 |

Every flag disappears over ten rounds. They also bunch by name — eight of the
twelve lie between `333e05d0…` and `3613794e…`, and the batches are cut from
the name-sorted list — which is what a load burst landing on the same batch in
both repeats looks like. **No input in any of the three sets runs slower on
the 09-23 binary.**

### 239.6 What it shows for §234.4's leg

**The fuse's own rewrite going into 09-23 did not make the same inputs
slower.** On the same files, the 09-23 binary ran 0.97, 0.96 and 1.02 times
as long as the 09-22 binary — each inside that set's own batch-to-batch spread
(0.55 – 1.05, 0.79 – 1.01, 0.88 – 1.18) — and file by file no input is slower
once rerun ten times.

So for the pair §234.4 found suspicious, leg (c) — the code — is **broken**:
09-23's 4.6 M runs against 09-22's 36.7 M is not the 09-23 kernel doing the
same work more slowly. What is left is what each night ran. A night runs
mutants of its corpus, not the corpus itself (§227.4's closing line), and the
09-23 night started from a corpus holding set C — the 923 files the 09-22 night
added, which take 17 ms each on either binary, against 5 – 7 ms for A and B,
the files the 09-22 night had started from. That points at leg (b), what the night ran, but it is
not tested here: no night's mutants were replayed.

### 239.7 What the 09-21 column says

Nobody asked for the 09-21 binary except as the first column, and it is the
one that moved:

- On **A**, the corpus the 09-21 night started from, the 09-21 binary took
  52.6 ms per input against the 09-22 binary's 4.97: **10.6 times slower**.
- On **B**, what the 09-21 night added, it took 2 360 ms against 6.98: **about
  340 times slower.**
- On **C**, what the 09-22 night added, it took 10.8 ms against 17.1:
  **1.6 times faster**, in every one of the 5 batches (1.47 – 1.82).

§234.4 read the diffs the other way round: the change going into 09-22 left
the fuse's own files alone, the change going into 09-23 rewrote them, and that
was "suggestive of a code explanation" on the 09-23 side, left "seen, not
confirmed". Measured, it is the reverse. The rewrite of `fuse/*.rs` changed
nothing on the same inputs (§239.6), while **the change going into 09-22,
outside `fuse/*.rs`, made the same fuse inputs ten to hundreds of times
faster** — a code explanation for 09-22's 36.7 M runs against 09-21's 1.3 M,
from the pair §234.4 had cleared. §234.4 lists what that change touched — `rcad-geom`'s `intersect.rs`,
`walk.rs`, `cone.rs`, `cylinder.rs`, `sphere.rs`, `torus.rs` and more, and
`rcad-topo`'s `mesh.rs` and `validate.rs`. Which of its 84 commits did it was
not looked for.

The 09-21 binary's time is lumpy rather than spread evenly. Thirty files drawn
at random from A (`shuf --random-source=<(yes 42) -n 30`) averaged 6.3 ms on
it, the slowest 32 ms, against 52.6 ms for the whole set — so most of A's time
on that binary sits in a few inputs the thirty missed, and its 500-file batches
ranged from 11.7 s to 49.8 s. Thirty random files from B fell on three levels:
13 at 0 – 75 ms, 7 at 553 – 597 ms, 10 at 4 552 – 4 791 ms. On the 09-22
binary all 169 of B averaged 7 ms, the slowest 57 ms. Three flat levels read
like a fixed amount of work being hit — a cap or a step limit — more than
inputs of graded difficulty, but that was not chased.

### 239.8 What was not run

- **The nights' own mutants.** Equal speed on the corpus files does not prove
  equal speed on the inputs each night generated from them.
- **No bisect** of the 84 commits from `ae124a29` to `79eedce7` for the one
  that made the fuse faster, and nothing on why the 09-22 night's finds (C)
  are slower on the binary that found them than on the one before it.
- **The 09-21 binary was not timed file by file** over all of A or B — only
  per-batch sums and the thirty-file samples — so the size of its tail is
  a sample's, not a census.
- **Load is not removed, only shared.** Other sessions built throughout
  (2.3 – 43.2); the order rotation spreads it and the ratios survive it, and
  the ten-round rerun is there because single runs did not.
