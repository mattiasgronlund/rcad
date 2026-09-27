## 293. What a timing merge costs inside a draw — 2026-09-27

*Queue row Q162, from §267.6: `HOW_OFTEN_TO_MERGE` moved the timing crate's
merge off its own thread and into a draw, on libFuzzer's own thread (§267).
The review left a plan and did not run it: `strace -f -ttt -e
trace=openat,flock` on a short `RCAD_FUZZ_TIMINGS` run, taking the time from
opening `lock` to releasing it as one merge, counted per process per second.*

  293.1   The answer
  293.2   The command and the script
  293.3   The measurement
  293.4   What share of a draw's time a merge costs
  293.5   How much of that is strace's own cost

### 293.1 The answer

A merge — from the `openat` that opens `the_shared_report_locked`'s `lock`
file to the `flock(fd, LOCK_UN)` that releases it, both in the same process —
took a mean of 13.0 ms and a median of 8.5 ms over 51 merges across 22
processes of `where_two_surfaces_cross`, with the longest at 62.1 ms. Against
a draw's own median time of 7 μs, one merge costs about 1200 draws' worth of
time. Against the process's own wall clock, merging cost about 0.3% of it —
the "rounding error next to the fuzzing" §267 already argued for, now
measured. `strace` itself inflates the 13.0/8.5/62.1 ms figures — see 293.5 —
so they are an upper bound on the real cost, not the real cost.

### 293.2 The command and the script

Built once, unstraced, so the timed run pays no compile time:

```
cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" cargo +nightly fuzz build where_two_surfaces_cross
```

Then, from `fuzz/`, on cores 18-21 (§162's cores; the night and two other
measuring lanes held 0-17):

```
ASAN_OPTIONS=detect_leaks=0 RCAD_FUZZ_TIMINGS=<dir> \
RUSTFLAGS="-C target-cpu=x86-64-v3" \
taskset -c 18-21 strace -f -ttt -o strace.log -e trace=openat,flock,rename \
cargo +nightly fuzz run where_two_surfaces_cross -- \
-max_total_time=90 -fork=4 -timeout=10 \
-ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1
```

`rename` is added to `-e trace=` beyond what the review's plan named, because
`the_shared_report_locked` frames only the wait for the lock and the
exclusive hold — `replace_whole`'s renames of `totals`, of every slow input,
and of `report` are what actually happens inside that hold, and the review's
own two-syscall list would have traced the bracket and missed everything it
brackets.

`ASAN_OPTIONS=detect_leaks=0` is not optional. Without it, LeakSanitizer
aborts the run: "LeakSanitizer does not work under ptrace (strace, gdb,
etc)" and asks for `/proc/sys/kernel/yama/ptrace_scope` to be `0`, which this
box has at `1` and which is not this lane's to change. Leak checking plays no
part in what is being measured here, so turning it off costs nothing this
section needs.

The script that reads `strace.log`, kept here in full since it is what turns
the trace into the numbers in 293.3:

```python
#!/usr/bin/env python3
"""Read a strace -f -ttt -e trace=openat,flock,rename log of a
RCAD_FUZZ_TIMINGS run and report, per process, how many merges it made and
how long each merge held the shared lock.

A merge starts at the openat() that opens rcad-fuzz-timing's "lock" file
(the_shared_report_locked in crates/rcad-fuzz-timing/src/lib.rs) and ends at
the flock(fd, LOCK_UN) that releases it, both in the same process. Everything
between — the exclusive flock(), reading "totals", the renames of "totals",
of every not-yet-written slow input, and of "report" — happens while that
fd is held, so the gap between those two lines is the whole merge, not just
the wait for the lock.
"""
import re
import sys
from collections import defaultdict

OPENAT = re.compile(r'^(\d+)\s+(\d+\.\d+)\s+openat\(AT_FDCWD, "(.*?)", [^)]*\)\s*=\s*(-?\d+)')
FLOCK = re.compile(r'^(\d+)\s+(\d+\.\d+)\s+flock\((\d+), ([A-Z_|]+)\)\s*=\s*(-?\d+)')


def read_trace(path):
    """(merges, lifetimes): merges[pid] is a list of merge durations in
    seconds, lifetimes[pid] is (first_ts, last_ts) over every traced line for
    that pid — a lower bound on the process's own lifetime, since neither its
    execve nor its exit was traced."""
    open_fd = {}
    merges = defaultdict(list)
    lifetimes = {}
    with open(path) as trace:
        for line in trace:
            pid = int(line.split(None, 1)[0])
            ts = float(line.split()[1])
            if pid in lifetimes:
                first, last = lifetimes[pid]
                lifetimes[pid] = (first, max(last, ts))
            else:
                lifetimes[pid] = (ts, ts)

            m = OPENAT.match(line)
            if m:
                _, _, filepath, ret = m.groups()
                ret = int(ret)
                if filepath.endswith("/lock") and ret >= 0:
                    open_fd[pid] = (ret, ts)
                continue
            m = FLOCK.match(line)
            if m:
                _, _, fd, flags, ret = m.groups()
                fd, ret = int(fd), int(ret)
                pending = open_fd.get(pid)
                if pending is None or pending[0] != fd:
                    continue
                if "UN" in flags and ret == 0:
                    _, open_ts = pending
                    merges[pid].append(ts - open_ts)
                    del open_fd[pid]
    return merges, lifetimes


def percentiles(values):
    values = sorted(values)
    n = len(values)
    mean = sum(values) / n
    median = values[n // 2] if n % 2 else (values[n // 2 - 1] + values[n // 2]) / 2
    return mean, median, values[-1]


def main():
    trace_path = sys.argv[1]
    merges, lifetimes = read_trace(trace_path)

    print(f"{'pid':>8} {'merges':>7} {'lifetime s':>11} {'merges/s':>9} "
          f"{'mean ms':>9} {'median ms':>10} {'max ms':>9}")
    all_durations = []
    for pid, durations in sorted(merges.items()):
        first, last = lifetimes[pid]
        lifetime = last - first
        mean, median, largest = percentiles(durations)
        all_durations.extend(durations)
        print(f"{pid:>8} {len(durations):>7} {lifetime:>11.3f} "
              f"{len(durations) / lifetime if lifetime > 0 else float('nan'):>9.3f} "
              f"{mean * 1e3:>9.3f} {median * 1e3:>10.3f} {largest * 1e3:>9.3f}")

    mean, median, largest = percentiles(all_durations)
    total_lifetime = sum(lifetimes[pid][1] - lifetimes[pid][0] for pid in merges)
    print(f"\n{len(all_durations)} merges total across {len(merges)} processes")
    print(f"mean   {mean * 1e3:.3f} ms")
    print(f"median {median * 1e3:.3f} ms")
    print(f"max    {largest * 1e3:.3f} ms")
    print(f"pace   {len(all_durations) / total_lifetime:.3f} merges per process per second, "
          f"summed over {total_lifetime:.1f} process-seconds")


if __name__ == "__main__":
    main()
```

A merge's fd is opened and closed fresh every time (`the_shared_report_locked`
returns a new `File` each call, and it is dropped at the end of `merge_out`),
so the same fd number gets reused by the same process's next merge. Pairing
the *next* `flock(fd, LOCK_UN)` after an `openat(".../lock")` on that fd,
within the same pid, is therefore unambiguous — nothing else in this process
opens a file literally named `lock`, and no other flock on that fd intervenes
between one merge's open and its own unlock. `flock` calls on *other* fds
appear throughout the trace (`cargo`'s own build-directory lock, taken once
near the start under `LOCK_SH` then `LOCK_EX`) and are ignored by construction,
since the script only follows a flock whose fd matches a `lock`-file open it
is still waiting to see closed.

### 293.3 The measurement

Both the strace run above and its unstraced twin used `where_two_surfaces_cross`,
`-fork=4`, `-max_total_time=90`, on `taskset -c 18-21`, `RCAD_FUZZ_TIMINGS`
pointed at a fresh scratch directory each time so the two runs' merges could
not interleave. Neither run was pointed at `fuzz/corpus/where_two_surfaces_cross`
directly — a plain `cargo fuzz run` writes newly interesting inputs straight
into that tracked directory, which the first attempt did before this was
noticed; the added files were removed with `git clean -fd fuzz/corpus/`
before anything was committed, and no corpus file is part of this section's
write set.

The script's output on the strace run's `strace.log`:

```
     pid  merges  lifetime s  merges/s   mean ms  median ms    max ms
 1908168       6      37.745     0.159     8.619      4.745    30.080
 1913581       2       2.077     0.963    21.240     21.240    24.793
 1913582       1      11.552     0.087     9.125      9.125     9.125
 1913583       2       4.082     0.490    12.507     12.507    19.092
 1913584       3       5.065     0.592    13.323     13.439    23.030
 1913846       1       0.113     8.863    11.071     11.071    11.071
 1913887       5       6.147     0.813    18.677      9.654    56.944
 1914083       1       0.088    11.383     6.026      6.026     6.026
 1914086       3      12.988     0.231    23.460     29.144    33.808
 1914211       1       0.078    12.812     5.420      5.420     5.420
 1914214       2       9.960     0.201     4.390      4.390     4.950
 1914568       1       0.129     7.741     5.177      5.177     5.177
 1914608       4      11.829     0.338     8.417      7.607    15.531
 1914943       1       9.427     0.106     3.001      3.001     3.001
 1916383       3      21.981     0.136    16.928     14.475    31.311
 1916386       1      12.117     0.083    14.137     14.137    14.137
 1916389       2      10.964     0.182     6.108      6.108     8.490
 1918756       2      18.131     0.110    18.067     18.067    22.618
 1918876       2      14.726     0.136     8.245      8.245    11.481
 1924761       6      18.021     0.333    18.740      9.516    62.068
 1924796       1       0.270     3.701    10.048     10.048    10.048
 1927529       1       0.083    12.064     5.897      5.897     5.897

51 merges total across 22 processes
mean   13.007 ms
median 8.490 ms
max    62.068 ms
pace   0.246 merges per process per second, summed over 207.6 process-seconds
```

The pace column is well under the nominal one merge a second `HOW_OFTEN_TO_MERGE`
sets: a process with only one draw running gets no chance to check whether a
merge is due until that draw returns, and `where_two_surfaces_cross` is the
target whose own doc comment already names a heavy tail — "twenty seeds of
996 taking 99.8% of the time" — so several of the 22 processes here spent
most of their traced lifetime inside one draw several seconds long (the
`report` this same run wrote lists slow units up to 13.0 s) and got far fewer
than one merge-due check per second of wall clock. The pace figure is an
average over a target whose draw lengths span nearly six orders of magnitude
(a 7 μs median against a 13 s tail), not a rate this target holds steadily.

### 293.4 What share of a draw's time a merge costs

The same run's own `report` (written by the crate being measured, so its
percentiles are what the crate itself thinks a draw costs, not what `strace`
adds — see 293.5) gives a median draw of 0.007 ms. Against that, the median
merge (8.490 ms) costs about 1200 draws' worth of time, and the longest merge
seen (62.068 ms) about 8900. That sounds large read as a multiple of the
fastest draws, and is: a merge is a lock, a file read, and three renames,
where most draws are a handful of predicate calls. But it is rare — one a
second at most, against however many thousand draws a busy process runs in
that second — so summed over the same 22 processes' 207.6 traced
process-seconds, the 51 merges' 0.663 s total is 0.32% of it. That is the
number `HOW_OFTEN_TO_MERGE`'s own doc comment predicted without measuring it:
"a rounding error next to the fuzzing."

### 293.5 How much of that is strace's own cost

`strace` only intercepts syscalls; a draw's own arithmetic runs unwatched
between them; a merge's open/flock/read/rename sequence is the one thing on
this crate's hot path that is *all* syscalls, so it is the one number this
section reports that `strace` can inflate.

The review's own suggested check — run the same target the same length with
and without `strace` and compare `exec/s` — was run: the unstraced twin
(same command, no `strace`, no `taskset`, `-max_total_time=90`, `-fork=4`)
logged 7 741 runs in about 101 s of fuzzing; the straced run logged 34 793 in
about 98 s. Read naively that says `strace` made the target *faster*, which
is absurd, and the reason is the same heavy tail 293.3 names: which of the
several-second inputs a run's mutations happen to explore in a given 90 s
window swings the total run count by an order of magnitude on its own,
swamping anything `strace` could cost a target whose typical draw makes no
syscall at all. **This comparison does not isolate `strace`'s cost on this
target, and the honest report is that it does not, not a number dressed up
to look like one** — the family of fix that would isolate it is a fixed,
non-mutating replay so both runs execute the identical sequence of inputs
(`-seed` fixes libFuzzer's mutator, but only a `-runs=0` corpus replay or an
equivalent fixed-content harness removes the tail's influence entirely), and
that was not built here.

What the two runs' own `report` files do support: their median draw times
were 0.007 ms (straced) and 0.015 ms (unstraced) — the same order of
magnitude, in the direction opposite to what tracing overhead would produce
if it reached the draw itself. That is consistent with the reasoning above
rather than a measurement of it: a draw that makes no syscall is not
`strace`'s to slow, so `strace`'s cost lands on the merges (293.3's numbers)
and not on 293.4's draw baseline. The 13.0/8.5/62.1 ms merge figures are
therefore an upper bound on the real, unstraced merge cost — some of that
time is `strace` stopping and restarting the process at each of the roughly
half-dozen syscalls a merge makes — and no unstraced measurement of the same
quantity exists, because nothing short of tracing the syscalls can see where
one merge begins and ends.
