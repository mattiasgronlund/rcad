## 262. A hung run does not starve the timing crate's merge thread — the loss measured is a few percent of one process's own runs, not close to a whole job — 2026-09-26

*§256.4's hypothesis, tested with a run rather than argued from logs: "while the
main thread hangs in one run until `-timeout` kills the process, the merge
thread's ticks do not land, so the job's runs since its start are lost." A
scratch fuzz target, built against the real, unmodified `rcad-fuzz-timing`
crate by path, checks this directly — first by watching whether the merge
thread is still being scheduled while the main thread hangs, then by measuring
the real loss under `-fork`, against ground truth this row wrote itself,
independent of both the crate and of libFuzzer's own periodic status line.*

### 262.1 The probe, and what "ground truth" means here

A one-target scratch `cargo-fuzz` project, outside the tree, depending on
`crates/rcad-fuzz-timing` by an ordinary path dependency — the same crate this
row measures, unchanged. Every input goes through `rcad_fuzz_timing::timed`,
exactly as a real target does. Two byte-triggered hangs, everything else
returns at once: first byte `0xFF` sleeps (yields the CPU — cannot itself
starve anything); first byte `0xFE` spins in a tight loop with no syscall in
it, the shape a real hung geometry predicate actually takes.

Outside `timed`, immediately after it returns, the probe also appends one line
to a per-process file (`<pid>.log`) with a single `write(2)` per line, flushed
so the count already on disk survives that process being killed mid-hang. This
is **ground truth**: how many runs this specific process actually completed,
measured independently of anything `rcad-fuzz-timing` does. Comparing it
against the crate's own `totals` file answers "how much did the crate lose",
without trusting the crate's own accounting to grade itself.

One quirk found and pinned down before it could confuse the rest of this row:
replaying explicit files with `cargo fuzz run TARGET file1 file2 ...`
executes each file's callback **twice**, not once — confirmed by dropping to
`-detect_leaks=0`, which brought a single file's ground-truth count from 2
lines to 1. This is cargo-fuzz's own leak-detection re-run on file replay, has
nothing to do with `rcad-fuzz-timing`, and does not apply to `-fork`'s
continuous fuzzing loop (§262.3's numbers are not halved anywhere).

### 262.2 Does the merge thread tick during a hang? Watched directly: yes, every second, straight through a 16-second CPU-bound spin

One process, no `-fork`: 20 ordinary inputs (each answering at once) followed
by one `0xFE` spin, replayed with `cargo fuzz run` and `-timeout=15` (long
enough that nothing kills the process — this run watches a live hang, not a
kill). `RCAD_FUZZ_TIMINGS`'s `totals` file was polled every 0.2s throughout.

The burst flushed on schedule — `runs 24` at t=1.07s, `runs 40` at t=2.17s
(the 20 files, each run twice per §262.1's quirk) — and then held at 40
**unchanged** for the whole spin, only moving to 41 once the spin itself
returned (a spin actually ran *twice* too, 8s + 8s = the 16014ms libFuzzer
printed for that one file). A frozen `totals` file during a hang proves
nothing by itself: with no new run completing, there is nothing to write
either way, tick or no tick.

So the process's two non-main threads were watched directly instead, via
`/proc/<pid>/task/<tid>/status`'s `voluntary_ctxt_switches` polled every
0.2s — a count that only rises when the kernel actually schedules that
thread, regardless of whether it finds anything to do. **Both threads' counts
climbed by very close to one per second, without a single gap, for the entire
17.8-second run — including all 16 seconds of the CPU-bound spin.** One of
the two is almost certainly `rcad-fuzz-timing`'s own merge thread (it sleeps
exactly `HOW_OFTEN_TO_MERGE` = 1s, `lib.rs:92-99,125`); the other is very
likely libFuzzer's own internal per-unit timeout watchdog, which polls on a
similar cadence. Either way: on this 24-core, otherwise idle machine, a
background thread in the same process is scheduled every second right through
a hang that pins the main thread at 100% CPU with no syscalls in it at all.
**This is the direct answer to whether the tick lands: it does.**

### 262.3 Under `-fork`, against ground truth: the loss is a few percent per process, not close to a whole job

Real `-fork` runs, matching the night's own flags shortened for a few
minutes' walk-clock instead of hours: `-fork=4 -timeout=3 -ignore_crashes=1
-ignore_timeouts=1 -ignore_ooms=1 -max_total_time=120`, `RCAD_FUZZ_TIMINGS`
and a per-process side-log both set.

Two corpora, both 120s:

- **"control"** — four short ordinary seeds, no `0xFE`/`0xFF` anywhere in it.
  libFuzzer's own mutations found a hang-triggering leading byte on their own
  regardless (unsurprising: a byte-flip lands on either value about 1 time in
  128, and this ran 24 000+ executions) — its last printed line read
  `oom/timeout/crash: 0/116/0` at `job: 117`.
- **"poison"** — the same four seeds plus one seeded with a leading `0xFE`.
  Last printed line: `oom/timeout/crash: 0/112/0` at `job: 113`.

| corpus | ground truth (side-log sum) | distinct processes (side-log files) | crate's own `totals` `runs` | gap | gap as a share of ground truth | ground truth ÷ processes |
|---|---|---|---|---|---|---|
| control | 25 009 | 229 | 24 184 (`lost 0`) | 825 | 3.3% | ≈109 |
| poison | 19 059 | 215 | 18 391 (`lost 0`) | 668 | 3.5% | ≈89 |

`lost 0` in both — the same finding as §256.2, on this box too: no merge ever
failed to write. The gap is real work this row's own side-log proved happened,
that the crate's shared `totals` never saw — genuine loss, not a write
failure, not an artifact of not looking. But it is **≈3.3–3.5% of a process's
own runs**, not "close to everything that job ever recorded": on the poison
corpus a killed process lost on average about 3 runs against roughly 89 it
actually completed. That is squarely the design's own stated bound — "a
process only loses what it has seen since its last merge," under a second's
worth — not the much larger loss §256.4 read out of the real night's
arithmetic.

One more thing ground truth caught: the side-log's distinct-process count
(229, 215) is **about twice** the last `job:` number libFuzzer's own status
line printed (117, 113) at exit. libFuzzer's status line is a periodic
snapshot and not a running total (§256.3 already said so about `record()`
lagging it the other way); this row's side-log now puts a number on the size
of that lag for the job counter specifically, on this run: roughly 2×.

### 262.4 Heavier oversubscription does not change it

The night's `-fork=4` is not much contention on a 24-core box. To stress the
scheduling side of the hypothesis harder than the night's own flags do, the
poison corpus was run again at `-fork=48` — twice this machine's core count —
for another 120s.

Last printed line: `oom/timeout/crash: 0/1241/0` at `job: 1242`. Side-log:
2 436 distinct processes summing to 258 794 completed runs. Crate's `totals`:
`runs 250774`, `lost 0`. Gap = 8 020, **3.1%** of ground truth — indistinguishable
from the lighter runs in §262.3, and again about half the side-log's own
process count against the printed job number (2 436 vs 1 242).

Doubling the machine's own oversubscription past what the night actually ran
did not move the loss rate. Whatever caused the real night's much larger gap
on `where_two_surfaces_cross`, this row found no evidence, at three different
concurrency levels on this box, that a hung run stops the merge thread from
being scheduled.

### 262.5 What this leaves of §256.4's hypothesis

**Broken, on this box, at three concurrency levels (no `-fork`, `-fork=4`
matching the night, `-fork=48` well past it): the merge thread's ticks do
land, every second, through a full CPU-bound hang, and the measured loss is a
few percent of one process's own runs — the design's own bound — not
"something close to everything that job ever recorded."**

This does not by itself explain the real night's arithmetic (§256.4:
≈266–309 lost runs per timeout-ending job, against a ≈270 average). Two
things this row measured point at a different explanation, offered as
inference and not as a re-measurement of that specific night:

- §262.3 measured libFuzzer's own printed job counter under-reporting the
  true process count by roughly 2× on this probe. If a similar lag held on
  the night of 2026-09-25/26, the true job count behind
  `where_two_surfaces_cross`'s 1 104 could have been substantially higher,
  which alone would roughly halve the per-job-ending average §256.4 computed
  — still large, but a good deal less than "close to everything".
- Dividing a night's total gap by its job count assumes each job would
  otherwise have produced the *average* job's worth of runs. A job whose
  fork-child happens to draw a hanging input very early in its life — before
  it has done much of anything — genuinely produces very few runs, full
  stop; nothing was lost from it because there was almost nothing to lose.
  That job still counts once in the denominator. Neither this row nor §256
  measured *when in a job's life* `where_two_surfaces_cross` tends to draw its
  hanging inputs, so this is a plausible shape for the gap, not a confirmed
  one.

**If a machine were found where the merge thread genuinely does starve** —
this row did not find one, but did not run on the night's own fuzzing
machine either — the fix would not touch how a run is timed, only how often
the backlog is exposed to loss: shorten `HOW_OFTEN_TO_MERGE` (`lib.rs:125`)
so the maximum possible unflushed backlog shrinks (more lock/write overhead
per process, traded for a tighter bound), or raise the merge thread's
scheduling priority (e.g. `libc::setpriority`) so a starved scheduler favours
it over the thread doing the hanging work. Neither is built here — the
mechanism this row could reproduce did not need it.

### 262.6 What is left

- **This ran on an idle 24-core dev box, never on the real overnight-fuzzing
  machine, and never on `where_two_surfaces_cross` itself.** A trivial probe
  that returns at once is not that target: a slower per-run target could pack
  fewer runs into the same "under a second" window, which would mean *less*
  absolute loss for a real target than this row measured for the probe, not
  more — but this is reasoning, not something run.
- **The ~2× lag this row found between libFuzzer's printed `job:` counter and
  the true process count is itself only measured on this probe, at this
  corpus size.** Whether a similar ratio held for `where_two_surfaces_cross`
  on the night of 2026-09-25/26 is not established; it would need the same
  side-log approach run against an instrumented copy of that target, or
  another way to count real job-endings independent of libFuzzer's own status
  line.
- **Whether a hanging input's position in a job's life (early vs. late)
  actually explains most of the real night's gap is not measured anywhere.**
  §262.5's second point is offered as a plausible shape, not a finding; it
  would need per-job run counts from a real night, which the current log
  format does not carry.

### 262.7 Corrected at merge: the extra processes are libFuzzer's merges, and §256's own method finds no shortfall on the probe

Read at merge against the libFuzzer source that `libfuzzer-sys` 0.4.13 builds
(`~/.cargo/registry/src/*/libfuzzer-sys-0.4.13/libfuzzer/`), and against the
lane's own run output. The logs are deleted, but the lane's transcript still
holds what it printed. §262.2's finding stands, and so does the loss §262.3 and
§262.4 measure between the side-log and `totals`. Four things change.

**The "2× job-counter lag" is not a lag.** In fork mode the parent prints the
status line after every job it collects. The guard that would skip some is
commented out (`FuzzerFork.cpp:222-227`), and job numbers go up by one per job
(`:381`, `:478`). So `job:` is the number of the job just collected, and it
counts jobs correctly. When a job has left inputs with new features, the parent
then runs `CrashResistantMerge` over them (`:229-237`). That is a second process,
and it runs the same target on those inputs, through `timed` and the side-log
like any other. The side-log's 229, 215 and 2 436 files are the jobs, plus these
merge processes, plus the one merge over the seed corpus at start (`:348`). This
row did not measure how they split. §262.3's last paragraph, §262.4's "about
half", §262.5's first bullet and §262.6's second are withdrawn. 1 104 is
`where_two_surfaces_cross`'s job count, and §256.4 was dividing by the right
number.

**Each kill lost about six runs on the probe.** §262.3's last column divides by
processes, and many of those are merges, not jobs. Divided by timeout
job-endings instead, the gap is 825 / 116 ≈ 7.1 (control), 668 / 112 ≈ 6.0
(poison) and 8 020 / 1 241 ≈ 6.5 (`-fork=48`), against §256.4's ≈ 309.

**§256.3's method, run on the probe, gives the healthy shape.** The lane never
put `totals` against libFuzzer's last `#N`, which is the comparison §256.3 made.
Its transcript holds both numbers for every run:

| run | `totals` `runs` | libFuzzer's last `#N` | gap |
|---|---|---|---|
| control | 24 184 | 23 923 | **+261** (+1.09%) |
| poison | 18 391 | 17 949 | **+442** (+2.46%) |
| poison, `-fork=48` | 250 774 | 242 558 | **+8 216** (+3.39%) |

On all three runs, 99% of jobs ended on a `-timeout` kill (112 of 113, 116 of
117, 1 241 of 1 242). Yet the record reads *above* libFuzzer's count, like the
cone, fuse and reader rows of §256.3's table, and nothing like
`where_two_surfaces_cross`'s −49.56%. Some of every positive gap is built in.
`#N` adds up only the jobs' own final stats (`FuzzerFork.cpp:200-201`). The merge
processes inherit `RCAD_FUZZ_TIMINGS`, so `totals` counts their runs and `#N`
does not. By how far the side-log beats `#N` (1 086, 1 110 and 16 236 runs), the
merges add a count of their own. So do the jobs still running at exit. §256.3
put its three positive gaps down to the status line lagging between prints.
That is one of two causes, and those two gaps say nothing about lost runs either
way.

**At the design bound, the kills cannot account for `where_two_surfaces_cross`'s
gap. Neither of §262.5's two leads explains it.** The night's own log
(`~/rcad-fuzz-logs/2026-09-26/fuzz.log`) shows `-fork=4 -timeout=10`, `-max_total_time=28715`,
and 592 047 as the last `#N`. Take the 951 kills' 10 s of hanging out of
28 715 s × 4 children. What is left is 105 350 child-seconds, about 5.6 runs a
second. For the design bound to cover the 293 405-run gap, each of the 1 104
job-endings would have to lose 266 runs in its last second. That is 47 times the
average rate, and the last two status lines print `exec/s` of 3 and 1. §262.5's
second bullet says a job that hangs early had little to lose. That is about how
§256.4 averaged, not about where runs went missing. A run libFuzzer counted is
in `#N` whichever job ran it. So what takes about half of this target's runs
out of its record, with `lost 0`, is not known here. §269 answers it: a
process that ends between `merge_out`'s truncating open and its write leaves
`totals` empty.

One smaller correction. The second thread §262.2 watched is libFuzzer's RSS
thread, which sleeps a second between checks (`FuzzerDriver.cpp:314-316`).
libFuzzer's `-timeout` is a `SIGALRM` timer (`FuzzerUtilPosix.cpp:110-114`), not
a thread. Nothing in §262.2's reading depends on which thread it was.
