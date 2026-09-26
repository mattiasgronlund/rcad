## 258. For fifteen minutes past its STEP corpus merge, the reader runs at 1.5% of its rate — 2026-09-26

*Queue row Q133, from §255.9. Lane `lane-q133-reader-steady-rate`, off
`8c76f406`. It measures what `8689e498` (T7, not merged) would hand libFuzzer,
by running the reader binary by hand with that command line. One round of two
arms was run before this lane hit its context budget; §258.8 lists what was
not. Every number below was run on this machine on 2026-09-26; the source
citations are to `libfuzzer-sys-0.4.13`, the version `fuzz/Cargo.lock` pins.*

  258.1   What was run
  258.2   Past the merge: 374 execs/s against 25 148
  258.3   Why: every job replays seeds, and they lean to the biggest files
  258.4   Fifteen minutes is not a night's steady state
  258.5   What the night would keep: 30 MB of big inputs in fifteen minutes
  258.6   The coverage the STEP files buy
  258.7   -max_len, read from the source and not yet run
  258.8   What this leaves, and what T7 should do meanwhile

### 258.1 What was run

`cargo +nightly fuzz build no_step_file_makes_the_reader_panic` at `8c76f406`,
with the `RUSTFLAGS` `fuzz-target` sets. Then two arms at the same moment, side
by side, so whatever else the machine was doing hit both alike. The load
average read 13.80 just before they started. Both ran with the flags
`fuzz-target` passes a night: `-fork=4` (24 cores over five targets),
`-timeout=10 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1`,
`-dict=step.dict`, `-max_total_time=900`. Each got a fresh scratch copy of
`fuzz/corpus/no_step_file_makes_the_reader_panic` (909 files) as its first,
writable directory, and its own `-artifact_prefix`.

- **Arm A** also got `RCAD_STEP_CORPUS` (520 files, 375 MB) as a second,
  read-only directory, last — where `8689e498` puts it.
- **Arm B** got nothing else: the reader as it runs today.

Rates below are the fork parent's own cumulative execution count (the `#N`
on each job line) over the time between two job lines. That count includes
each job's seed replay, which is the point: it is what the night gets done.

### 258.2 Past the merge: 374 execs/s against 25 148

| | arm A, STEP corpus in | arm B, without |
|---|---|---|
| seed inputs after the merge | 1 159 | 873 |
| first job line | 304 s | 2 s |
| jobs finished | 63 | 82 |
| executions in all | 163 693 | 24 152 604 |
| execs/s, first job line to last | 261 | 26 519 |
| execs/s, from 600 s to the end | 374 | 25 148 |
| jobs ended on `-timeout` | 6 | 0 |
| `timeout-` / `slow-unit-` files | 6 / 3 | 0 / 0 |

The merge took about 300 s, which is §255.3's 282 s again. Past it, arm A ran
at **about 1.5% of arm B's rate**, 67 times slower, over the last five minutes
of the run. Its per-job `exec/s` for the last ten jobs was 22 to 194; arm B's
was 5 488 to 7 416.

### 258.3 Why: every job replays seeds, and they lean to the biggest files

Fork mode does not fuzz one long process. It starts one short job after
another, and each job begins by running every one of its seeds once:

- `FuzzerFork.cpp:139-160`: a job's seeds are `sqrt(N + 2)` files from the
  kept corpus — 34 of 1 159 here.
- They are picked with `SkewTowardsLast` (`FuzzerRandom.h:27-31`, the square
  root of a uniform draw over `n²`). The chance of an index past `m` is
  `1 - m²/n²`.
- The kept corpus is in ascending size order, because the merge sorts by size
  before it keeps anything (`FuzzerMerge.cpp:164-172`) and new finds are
  appended.

So the biggest files are the likeliest seeds. With 1 159 kept and 873 of them
being what the tracked corpus alone keeps, about 286 kept files came from the
STEP corpus, and they are the largest. The chance a given seed is one of the
top 286 is `1 - (873/1159)²`, about 43%. Every job replays around fifteen real
STEP files before it mutates anything. A seed that takes longer than
`-timeout=10` ends the job, which is what 6 of the 63 did.

The replay does not count against the job's time. `TimedOut()` is checked in
the fuzzing loop, after `ReadAndExecuteSeedCorpora` has finished
(`FuzzerLoop.cpp:872`). A job whose replay outlasts its budget ends the moment
the replay does, having mutated nothing. Arm A's first five jobs each ran
exactly 35 inputs (34 seeds and the empty one) and stopped.

### 258.4 Fifteen minutes is not a night's steady state

A job's budget is `min(300, JobId)` seconds (`FuzzerFork.cpp:130`, under the
comment "Start from very short runs and gradually increase them"). Arm A
reached job 63 and arm B job 82, so no job in either arm had a budget over 82
seconds. A night reaches 300 s jobs: at four jobs at a time
that takes roughly `300²/8` seconds, about three hours of its eight.

In a 300 s job the replay is a smaller share than in a 60 s one. So the 1.5%
in §258.2 is the reader's first fifteen minutes, and **overstates** what the
STEP corpus costs across most of a night. By how much was not measured; it
needs a run long enough to reach job 300 (§258.8). Note too that the mutations
themselves are slower in arm A, not just the replay. A job whose seeds include
a file over 1 MiB derives `-max_len` as 1 MiB (`FuzzerLoop.cpp:806-807`), so it
mutates inputs up to that size, and one of those can take the whole
`-timeout` on its own (§255.4).

### 258.5 What the night would keep: 30 MB of big inputs in fifteen minutes

New finds go to the first corpus directory. In a night that is
`fuzz_corpus_root/no_step_file_makes_the_reader_panic`, which the next night
starts from. In fifteen minutes arm A wrote **245 new files there, 30 324 162
bytes in all, median 85 430 bytes, largest 926 390**. Arm B wrote 639, 2 119 541
bytes in all, largest 5 110 — the tracked corpus's own largest, since that is
what its `-max_len` derives from.

Those big finds are not a one-off cost. They join every later night's startup
merge and every later job's seed pool, and the size ordering in §258.3 puts
them near the end, where the seeds are drawn from. Nothing here measured how
that compounds over nights.

`RCAD_STEP_CORPUS` itself still held 520 files afterwards, and none was newer
than the start of the runs. §255.6's placement holds.

### 258.6 The coverage the STEP files buy

Arm A's first job line reported `cov: 11159 ft: 48170`: that is the merge's
coverage before any fuzzing at all. Arm B, fuzzing flat out, ended at `cov:
4463 ft: 12449`. So the STEP files start the reader at two and a half times
the edges, and four times the features, that fifteen minutes of fuzzing
without them reaches. Arm A still gained 227 edges and 1 987 features in its
slow fuzzing. That is what §226.1 wanted, and it is why this section does not
recommend dropping T7. The question is only how to pass the corpus.

### 258.7 -max_len, read from the source and not yet run

§255.5 said that pinning `-max_len` "would not touch the merge cost", because
the merge still reads and runs every seed. The source says the merge runs a
seed **truncated** to `-max_len`: `FuzzerMerge.cpp:228-232` resizes each input
to `MaxInputLen` before running it. Seed loading does the same
(`FuzzerLoop.cpp:832`). So with `-max_len` pinned, the merge still reads 375 MB
from disk but runs only each file's first `max_len` bytes. For a real STEP file
that is the header and the first entities. Pinning would bound the finds of
§258.5 at `max_len` too.

**None of that was run.** It is a reading of the source, and §16.10 asks for a
demonstration. It also cuts both ways: a STEP file cut to its first 5 KB may
keep little of the coverage in §258.6. That is arm D in §258.8, and it is the
arm that decides T7.

### 258.8 What this leaves, and what T7 should do meanwhile

Not run, in order of what they would decide:

1. **Arm D, the full STEP corpus with `-max_len` pinned**, beside arm B. It
   should report the merge time, the rate past it, the starting `cov`/`ft` and
   the size of the finds. 5 110 is the tracked corpus's own largest; a larger
   value, such as 65 536, keeps more of each STEP file, and the run should say
   which value holds on to §258.6's coverage.
2. **A long pair, to job 300** (about three hours at `-fork=4`), for the
   steady rate §258.4 says fifteen minutes cannot give.
3. **Arm C, the 516 non-zip files only.** §255.4 already found the zips add
   almost nothing to what the merge keeps; the symlink directory is built and
   the run was not.
4. **A second A/B round**, for how much the numbers in §258.2 move between
   runs.

For T7 in the meantime: **do not merge `8689e498` as written.** As written, the
reader runs at 1.5% of its rate for at least its first fifteen minutes and
writes big inputs into the night's own corpus at 30 MB a quarter hour. The
likely fix is to pin `-max_len` for the reader when the STEP corpus is passed,
and to leave the zips out as §255.7 said. The value, and whether it keeps the
coverage, waits on arm D.
