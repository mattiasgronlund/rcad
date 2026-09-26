## 269. The fuzz timing record loses its runs when a process dies between emptying `totals` and refilling it — 2026-09-26

*Q150. On 2026-09-26 `where_two_surfaces_cross`'s timing record held 298 642
runs, with `lost 0`, where libFuzzer counted 592 047 (§256.3). §262 ruled out
hung runs. This section measures one more cause, and it is the cause.
`merge_out` rewrites the one shared `totals` file with `std::fs::write`
(`crates/rcad-fuzz-timing/src/lib.rs:224`). That empties the file when it
opens it and refills it when it writes. `read_back` reads an empty file as
zero (`lib.rs:541-567`). So a process that ends between those two steps wipes
every run the record held, and the next merge starts again from its own few.
That window is about a tenth of a millisecond, and it is hit about once in
five target-nights, which is what the nights show.*

### 269.1 The 23-minute probe saw no reset, and was not expected to

The previous lane's probe, copied to `/tmp/q150-leg3/probe.sh` and run from
this lane's tree. It runs `cargo fuzz run where_two_surfaces_cross` with
`RCAD_FUZZ_TIMINGS` set, and reads `totals` every 0.15 s under the crate's own
lock (`flock -w 1 lock -c "cat totals"`). A reset shows as `runs` going down
between two reads, and it stays down, since nothing adds the lost runs back.

Its flags are the night's own, from the `Running` line at
`~/rcad-fuzz-logs/2026-09-26/fuzz.log:12`, with two differences. The probe
lacked `-max_len=71` and now has it. `-max_total_time` is 1 380 instead of
28 715. So: `-max_total_time=1380 -max_len=71 -fork=4 -timeout=10
-ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1`.

It ran from 10:54:26Z to 11:19:30Z. Read back by `/tmp/q150-leg3/analyse.py`:

- 9 346 reads, 8 815 of them after `totals` existed. `runs` went down **0**
  times. No read was empty, and in every read the bucket counts added up to
  `runs`.
- libFuzzer's last line: `#277195: ... oom/timeout/crash: 0/37/0 ... job: 99`.
  So 99 jobs ended: 37 on a timeout, none on a crash or out of memory, and 62
  normally. The merge processes libFuzzer runs between jobs (§262.7) are not in
  its log and were not counted.
- The record ended at `runs 275122`, `lost 0`. That is 2 073 below libFuzzer's
  count, −0.75%.

§269.4 puts the expected number of resets in a run this size at 0.04 at most.
Seeing none says nothing either way, which is why the window is measured
directly below.

### 269.2 How long a merge leaves `totals` empty: about a tenth of a millisecond

The same binary and the night's flags for 120 s, under strace, one trace file
per thread so no other thread's line splits an `openat` from its `write`:

```
strace -ff -ttt -T -y -s 24 --seccomp-bpf \
  -e trace=openat,write,close,rename,flock,exit_group -o <dir>/t \
  target/x86_64-unknown-linux-gnu/release/where_two_surfaces_cross \
  -artifact_prefix=<dir>/ -max_total_time=120 -max_len=71 -fork=4 -timeout=10 \
  -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 <corpus> corpus/where_two_surfaces_cross
```

`RCAD_FUZZ_TIMINGS` pointed under `/tmp`, which is the same ext4 filesystem
the night's `~/rcad-fuzz-logs` is on (`df -T`). Read back by
`/tmp/q150-leg3/windows.py`, over 82 merges from 68 processes:

- **From the start of the truncating `openat` to the end of the `write` that
  refills the file: median 133 µs, p90 170 µs, largest 254 µs.** From the
  `openat`'s return instead: median 72 µs, largest 191 µs.
- Tracing stops the process at each of those syscalls, so the real window is
  shorter than these figures, not longer.
- Merges per process per second: 82 in 221.5 process-seconds, 0.37. It is
  never above one. The merge thread sleeps `HOW_OFTEN_TO_MERGE`, one second,
  between merges (`lib.rs:125`), and returns early without writing when
  nothing is new (`lib.rs:208`).

**A timeout cannot end a process inside the window.** During a hang no run
finishes, so every tick after the first one in the hang returns at
`lib.rs:208` before opening anything. In the trace, the three processes that
ended on a timeout (exit 70, libFuzzer's timeout exit code) had last written
`totals` 11 002, 11 008 and 11 022 ms before they died. So §256.4's 951
timeout endings drop out of the count in §269.4.

The tracer changes one thing, and it is not the window. 65 of the 68 processes
ended with exit 1, and libFuzzer counted each as a crash (`0/0/1`, `0/0/2`, …,
one per job; the night had none). LeakSanitizer's check at exit needs
`ptrace`, and strace already holds it. That changes how a traced process ends,
not the merges it makes while alive. So no end times for normal endings are
taken from this run. None are needed: a normal ending is `exit(0)` from the
main thread when the job's time is up, and nothing ties that moment to the
merge thread's tick. Its chance of landing in the window is the window times
the merge rate, the same as a crash's.

### 269.3 One kill in the window, with the real target: the record restarts from zero and says `lost 0`

`/tmp/q150-leg3/reset-demo.sh` runs the target three times into one
`RCAD_FUZZ_TIMINGS` directory, without `-fork`:

| run | how it ended | libFuzzer | `totals` after it |
|---|---|---|---|
| 1 | normally, after 6 s | `Done 902 runs` | `runs 819`, 1 744 bytes |
| 2 | SIGKILL at its first write to `totals` | — | **0 bytes** |
| 3 | normally, after 3 s | `Done 903 runs` | `runs 834`, `lost 0` |

Run 2 ran under `strace -f -P <totals> -e trace=openat,write -e
inject=write:signal=SIGKILL:when=1`. Its trace is the whole mechanism in four
lines: the read-back, then `openat(... O_WRONLY|O_CREAT|O_TRUNC ...)`, then
`write(4, "runs 1453\nnanos ...", 1986) = ?` 133 µs later, then `+++ killed by
SIGKILL +++`. Run 3's record holds only run 3's own runs. Run 1's 819, and the
634 run 2 was about to add, are gone, and nothing in the file says they ever
existed.

Run 1's 83 missing runs (902 against 819) are a different, smaller loss. They
came in its last second and never got a tick. That is the design's own bound
(`lib.rs:114-125`), not a reset.

### 269.4 How many resets a night that predicts

The chance that one ending lands in the window is the window times the merges
per second. Timeouts count for nothing (§269.2). Normal endings, crashes,
running out of memory, and libFuzzer's merge processes all count.

- **09-26's `where_two_surfaces_cross`.** The last status line reads job
  1 102 with 951 timeouts, so there were 151 other job endings. There is at
  most one merge process per job, plus one at the start, which adds up to
  1 103 more. So at most 1 254 endings could have landed in a window.
  - 1 254 × 254 µs × 1/s = 0.32 a night.
  - 1 254 × 133 µs × 1/s = 0.17 a night.
  - 1 254 × 133 µs × 0.37/s = 0.06 a night.
  - Leaving the merge processes out, the 151 endings alone give 0.007 to 0.04.
- **The 23-minute probe.** 62 normal job endings plus at most 100 merge
  processes: at most 0.04.
- **Every target on every night in §269.5.** 42 975 jobs, 5 297 of them
  timeouts, which leaves 37 678 other endings, or 80 653 counting one merge
  process per job. Expected resets: 1.9 (37 678 × 133 µs × 0.37/s) to 20.5
  (80 653 × 254 µs × 1/s).

A reset on a given target on a given night is not rare. It is about what this
arithmetic says.

### 269.5 What the nights show: seven large one-off losses in 35 target-nights

`/tmp/q150-leg3/nights.py` compares, for every night in `~/rcad-fuzz-logs`
with a `timings/` directory (09-20 to 09-26) and every target, `totals`'s
`runs` against the last `#N` in that target's own log. The target is named by
the log's `Running` line.

28 of the 35 read between −0.5% and +2%, mostly above, for the reasons §256.3
and §262.7 give. Seven read far below:

| night | target | record | libFuzzer's last `#N` | gap | jobs | timeouts / crashes |
|---|---|---|---|---|---|---|
| 09-21 | cone-or-torus | 1 009 847 | 2 298 296 | −56.06% | 499 | 0 / 0 |
| 09-21 | fuse | 1 303 505 | 2 047 832 | −36.35% | 487 | 0 / 0 |
| 09-23 | surfaces-cross | 344 154 | 651 805 | −47.20% | 810 | 638 / 0 |
| 09-25 | reader | 424 741 863 | 448 741 142 | −5.35% | 528 | 0 / 0 |
| 09-25 | surfaces-cross | 9 349 | 676 854 | −98.62% | 956 | 837 / 0 |
| 09-26 | writer | 2 877 309 | 3 907 243 | −26.36% | 22 813 | 0 / 22 811 |
| 09-26 | surfaces-cross | 298 642 | 592 047 | −49.56% | 1 102 | 951 / 0 |

`lost` is 0 on all three 09-26 rows; the files before 09-26 predate the `lost`
line. Three things about these seven fit a reset, and nothing else found so
far fits them:

- **Each is a one-off, not a steady bleed per ending.** 09-21's cone-or-torus
  lost 56% in 499 jobs with no timeout and no crash. The design bound is under
  a second's runs per ending (§262.7), and the same target on its other six
  nights reads within half a percent.
- **The share lost is anywhere from 5% to 99%,** as "everything before a random
  moment" should be. 09-25's surfaces-cross kept 9 349 of 676 854 runs, so its
  last reset came in the night's last 1.4% of runs.
- **Seven in 35 sits inside §269.4's range of 1.9 to 20.5.**

Surfaces-cross has three of the seven, where the arithmetic's middle figure
(0.17 a night) expects about one in seven nights. That is at the high end of
the range, not outside it. The 09-26 writer, with 22 813 crash endings, is
expected to reset about three times that night (22 813 × 133 µs × 1/s). Its
−26.36% is part design bound (§256.4 counted about 45 runs per ending) and
part reset. One night cannot split the two.

### 269.6 Verdict

**Leg 3 holds.** A process that ends between `merge_out`'s truncating open and
its write leaves `totals` empty. The next merge reads that as zero and writes
back only its own runs, with `lost 0`. This happens about as often as the
window predicts, and it is what makes a target-night lose a large, random
share of its record. **The 293 405 runs `where_two_surfaces_cross` is missing
on 09-26 are the runs it recorded before that night's last reset.**

What this rests on, and what it does not:

- **Shown:** the mechanism, end to end, with the real target and the real
  crate (§269.3). **Measured:** the window (§269.2). **Counted:** how often
  large one-off losses happen across nights, against what the arithmetic
  predicts (§269.4, §269.5).
- **Not observed:** no reset was caught live on a real night. No night keeps a
  time series of `totals`, so that 09-26's gap is a reset is the best fit, not
  something seen.
- **Also possible:** a full disk would empty the file the same way, since
  `std::fs::write` truncates and then the write fails. No night here is known
  to have filled the disk, and the fix covers both.

The previous lane's two other leads cannot make a shortfall:

- **cargo-fuzz replaying artifacts after the run.** A replay goes through
  `timed` like any run, so it adds to the record. It can only take runs away as
  one more ending that lands in the window, and that is this section's cause,
  not a second one.
- **The fork's `min(300, JobId)` ramp of short jobs.** More endings early in
  the night, each losing at most one tick's runs, which is the design bound.
  The ramp is there on all 35 target-nights, and 28 of them lose nothing
  beyond that bound.

### 269.7 The fix, and what is left

The fix is to write `totals` to a temporary name in the same directory and
rename it over the real one. That is how `write_slow_input` has written slow
inputs since §236.6 (`lib.rs:286-290`). A rename lands whole or not at all, so
a process killed at any point leaves either the old record or the new one, and
never an empty one. **Q136-finish is making it.** This lane made no code
change.

Left:

- **The test this verdict still owes.** Once Q136-finish is on the nightly
  build, compare each night's `timings/<target>/totals` `runs` against the last
  `#N` in that target's log, as §269.5 does, for a week of nights. If this
  section is right, large one-off losses stop: none in about 35 target-nights,
  where there were seven before. The scripts under `/tmp/q150-leg3/` are
  scratch and do not survive a reboot. The comparison is small enough to redo
  from this paragraph.
- **`report` is written the same way,** but it is rebuilt whole from `totals`
  on every merge. So an empty one mends itself on the next tick, and it needs
  no fix.
