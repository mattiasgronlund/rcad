## 268. Under `-fork=4`, set C costs 1.7 times over five minutes; the hour's run lost its binary after that — 2026-09-26

*Runs queue row Q135, the next measurement §254.6 named: the same two corpora,
with C and without it, under `-fork=4` for much longer than two minutes. It
was meant to run an hour. The fuzz binary was deleted from under it about five
minutes in, so only those five minutes are fuzzing. This lane only read and
measured. Nothing in the kernel, the nightly tree or the corpus was written,
and the run fell between 10:34:40Z and 11:35:40Z, outside 21:50–06:00Z.*

  268.1   What ran
  268.2   Why the counts go flat at five minutes
  268.3   What the five minutes show
  268.4   What it shows for §254.5
  268.5   What the next run must change

### 268.1 What ran

**The binary and the corpora are §254's.** The binary is
`two_solids_fused_by_a_stranger` built at `055aa4a9` in
`/tmp/q61-build-055aa4a9` (§254.1). Each arm started from a fresh hard-linked
copy of §254's own arm: 11 077 files with C, 10 154 without.

**Both arms ran at the same time**, so they shared whatever load the machine
had:

```
RCAD_FUZZ_TIMINGS=<arm's own dir> <binary> -fork=4 -max_total_time=3600 -timeout=10 \
  -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 -print_final_stats=1 <copy>
```

**Runs were counted by the timing crate, not by libFuzzer.** Under `-fork` the
parent prints no `stat::number_of_executed_units` line, which is what §254
counted; a 60 s smoke run under `-fork=4` printed nine lines and none of them
a `stat::` line. `rcad-fuzz-timing` merges every child's runs into one `totals`
file (§262 found the loss to be a few percent), and a loop read its `runs` line
every 300 s. The 1-minute load average was 7.59 at the start and 10 – 27 during
the run, from other sessions.

### 268.2 Why the counts go flat at five minutes

| t | runs with C | runs without C |
|--:|------------:|---------------:|
| 300 s | 175 297 | 301 308 |
| 600 s | 217 431 | 369 916 |
| 900 s – 3 600 s | 217 431 | 369 916 |

**The fuzzers stopped fuzzing.** `/tmp/q61-build-055aa4a9` was removed at about
10:40Z, 315 s into the run, and is gone now. This lane did not remove it. From
that moment every new job failed to start. The log shows it: `exec/s: 0`, and
the crash count in `oom/timeout/crash` goes up by one per job, to 8 381 618 with
C and 8 485 304 without by t = 3 600 s. Both parents then exited with status
127, the shell's "command not found". The parents lived on because each had
already loaded the binary; `-ignore_crashes=1` made each failed start a crash
to step past.

**It was not a hang, and not the timing merge.** Both `totals` files were last
written at 10:40:32Z and 10:40:29Z, when the last jobs that had started before
the removal finished. They still hold those counts. §269's failure, a process
that ends mid-merge and leaves `totals` empty, would show as a drop to zero,
and neither file dropped.

So everything below comes from the first 315 s.

### 268.3 What the five minutes show

| | with C | without C | without ÷ with |
|--|-------:|----------:|---------------:|
| seeds the parent kept from the corpus | 3 518 | 3 062 | |
| first job finished at | 100 s | 77 s | |
| runs at t = 300 s | 175 297 | 301 308 | 1.72 |
| runs in all | 217 431 | 369 916 | 1.70 |
| time spent inside the target, in all | 717.9 s | 730.1 s | |
| mean time per run | 3.30 ms | 1.97 ms | 1.67 |
| p90 | 9.437 ms | 1.180 ms | |
| p99 | 52.429 ms | 39.846 ms | |

"Runs in all" includes the jobs that were still running when the binary went.
The seed counts and job times are from the parent's own lines, the rest from
each arm's `timings/.../report`.

**Both arms spent about the same time inside the target**: 718 s against
730 s. So the two arms got about the same CPU, and the gap is how long each run
took, not a difference in share. With C, the mean run is 1.67 times as long,
and p90 is eight times as long.

**After the first job, the gap is wider than §254 found for one process.**
From the parent's `#N` lines, with C went from `#118` at 100 s to `#179821` at
318 s, 824 runs a second. Without C went from `#172` at 77 s to `#307033` at
315 s, 1 289 a second. That is 1.56 times. §254.3 measured 1.15 times for the
mutants of one process. The two setups differ in more than `-fork`: this run
had four children, heavier load, and the parent's first pass over the corpus
before any job, so this does not say which of those makes the difference.

### 268.4 What it shows for §254.5

**Under `-fork=4`, C costs 1.7 times over five minutes**, against §254's 1.4
times over two minutes in one process. It is still far from the nights' 8
times. The shape moves the way §254.5's hypothesis says it would: with C in the
corpus, a much larger share of the runs are slow ones (p90 at 9.4 ms against
1.2 ms). But this is one pair of runs, with unpinned seeds, under heavy load.

**It says nothing about whether the gap grows over an hour**, which is what
Q135 asked. There is one clean sample, at 300 s, so there is no trend to read.
That question is still open.

### 268.5 What the next run must change

- **Run a binary the lane owns.** Build it in the lane's own scratchpad, or copy
  it there before starting. This run used another lane's `/tmp` build tree and
  lost it.
- **Stop the round when the jobs stop.** The sampling loop should also read the
  crash count and `exec/s` from each parent's last line and end the round once
  jobs fail to start. This one spent 55 minutes and two 8.4 M-line logs failing
  to start jobs.
- **Keep the rest**: both arms at once, counts from the timing crate, samples
  every 300 s, an hour, outside 21:50–06:00Z.
- **Read how libFuzzer sizes a job under `-fork`** (how many seeds each job gets
  and how long it runs) before reading a trend. It was not read here.
