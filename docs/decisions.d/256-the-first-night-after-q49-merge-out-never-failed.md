## 256. The first night after Q49: `merge_out` never failed, and the two real undercounts are elsewhere — 2026-09-26

*§236.6's row, followed up: with Q49's fix landed (`a4238c5a`, "hand a merge's
runs back on a write failure, count them", cargo-fmt'd at `9f76894b`), read the
first night that ran it and set its new `lost` count — how many runs a merge
could not write and had to hand back — against libFuzzer's own count, to say
whether a failing `merge_out` is what §236 found, and why a slow night would
fail more.*

  256.1   Which night, and that it carries Q49
  256.2   `lost` is zero, on every target, all night
  256.3   The report against libFuzzer's own count, target by target
  256.4   The writer's gap fits the old per-job bound; the surfaces-cross gap
          does not, and 951 of its 1 104 jobs died to a timeout
  256.5   What is left

### 256.1 Which night, and that it carries Q49

The nightly-fuzz worktree re-parks itself on `main` once a night, recorded in
its own reflog. The run logged under `~/rcad-fuzz-logs/2026-09-25` was checked
out at `2026-09-25 00:10:29 +0200`, onto `6664fef8` — before Q49 landed
(`9f76894b`, 2026-09-25 22:37:21 +0200) — confirmed with
`git merge-base --is-ancestor 9f76894b 6664fef8`, which answers false. The next
run, logged under `~/rcad-fuzz-logs/2026-09-26`, was checked out at
`2026-09-26 00:10:44 +0200` onto `c92b8fe5`, and
`git merge-base --is-ancestor 9f76894b c92b8fe5` answers true. So
`~/rcad-fuzz-logs/2026-09-26` — 2026-09-25 22:10 UTC to 2026-09-26 06:1x UTC,
all five targets run with `-fork=4 -timeout=10 -ignore_crashes=1
-ignore_timeouts=1 -ignore_ooms=1 -max_total_time=28715` — is the first night
this row can read.

### 256.2 `lost` is zero, on every target, all night

Q49 added a `lost N` line to `written_out()`'s on-disk format and to the report
when `N>0`. Read whole (`cat ~/rcad-fuzz-logs/2026-09-26/timings/*/totals`),
every one of the five targets' `totals` files reads `lost 0`:
`no_model_makes_the_writer_panic`, `no_step_file_makes_the_reader_panic`,
`two_solids_fused_by_a_stranger`, `where_a_cone_or_a_torus_crosses`,
`where_two_surfaces_cross`. `hand_back_what_could_not_be_written` was never
called for any of them: on this night, `create_dir_all` and the shared lock
never failed once, on any target, across the whole run. Q49's own unit test
(`runs_a_failed_merge_could_not_write_survive_and_are_counted`) is the only
place that mechanism has ever actually fired.

### 256.3 The report against libFuzzer's own count, target by target

Following §236.3's method — the report's `runs` against the last `#N` a
target's own libFuzzer log printed before `INFO: exiting` — read from
`~/rcad-fuzz-logs/2026-09-26/fuzz-{cones-and-tori,fuse,reader,writer}.log` and
`fuzz.log` (`where_two_surfaces_cross`'s log; it has no `fuzz-` prefix of its
own):

| target | report `runs` | libFuzzer's last `#N` | gap | job (last seen) |
|---|---|---|---|---|
| cone-or-torus | 2 246 738 | 2 237 264 | **+9 474** (+0.42%) | 486 |
| fuse | 2 296 832 | 2 282 657 | **+14 175** (+0.62%) | 507 |
| reader | 426 788 358 | 425 652 406 | **+1 135 952** (+0.267%) | 528 |
| writer | 2 877 309 | 3 907 243 | **−1 029 934** (−26.35%) | 22 813 |
| surfaces-cross | 298 642 | 592 047 | **−293 405** (−49.56%) | 1 104 |

The first three read *higher* than libFuzzer's own last-printed line, not
lower — expected and not an undercount: the report is updated on every
`record()` while libFuzzer's status line is a periodic snapshot that lags the
true count between prints, and by design (§236.4) that lag is where a night's
individual runs stay fast. Both `lost=0` for these three and their small,
positive gap say the same thing: nothing was lost here at all.

The last two are large, genuine shortfalls — and `lost=0` for both of them too.
**So on this night, in every one of the five cases where the report and
libFuzzer disagree, a write failure Q49 can now count is not the cause,
because none happened.** Whatever produces the writer's and the
surfaces-cross's gap, it happens before `merge_out` is ever called, not inside
it.

### 256.4 The writer's gap fits the old per-job bound; the surfaces-cross gap does not, and 951 of its 1 104 jobs died to a timeout

The writer's last cumulative line reads `oom/timeout/crash: 0/0/22811` at
`job: 22813` — almost every one of its 22 813 job-endings was a crash
(`-ignore_crashes=1` respawns rather than stopping). Its gap per job is
1 029 934 / 22 813 ≈ **45 runs**, against a tail exec rate of 33–44/s on its
last three printed lines. That is squarely the module's own documented bound —
"a process only ever loses what it has seen since its last merge," under a
second's worth, once per job-ending (`crates/rcad-fuzz-timing/src/lib.rs:114-125`)
— just multiplied up by having ~45× the job-endings a quiet night has (~500).
Nothing here needed Q49 at all: a crash kills its process fast, close to
whatever the merge thread last flushed, and the design bound already covered
it.

`where_two_surfaces_cross`'s last cumulative line reads
`oom/timeout/crash: 0/951/0` at `job: 1102`, and the last job its log names is
1104 — **86% of its job-endings were a
10-second `-timeout` kill**, not a crash and not ordinary fork rotation. Its
gap per job-ending is 293 405 / 1 104 ≈ **266 runs**; its gap per *timeout*
job-ending is 293 405 / 951 ≈ **309 runs** — close to the night's own overall
average of 298 642 / 1 104 ≈ **270 runs per job**. That is not "under a
second's worth"; it is close to a whole job's worth, on almost every job. A job
that hangs the full 10 seconds before libFuzzer kills it loses not the last
second since its merge thread's last tick, but something close to everything
that job ever recorded.

This is a mechanism `merge_out`'s fix does not touch and Q49 was never asked
to: the loss happens because the merge thread's own tick never lands, not
because it lands and fails to write. The crate's merge thread is a second OS
thread that sleeps one second and then calls `merge_out` (`lib.rs:99-108`); if
the main thread is pinned on the one core its fork child has and spends up to
ten seconds inside a single hanging `run()` before libFuzzer's own `-timeout`
watchdog kills the process, whether the merge thread gets scheduled during
that hang at all is a question about the OS scheduler under this box's
fork-degree (4 children sharing however many cores `nproc`/5 leaves each
target), not something these logs can answer. **That link — a full ten-second
hang starves the merge thread rather than merely outrunning its last tick — is
this row's hypothesis, not its finding**; the arithmetic above (per-job loss ≈
per-job average, on the target whose job-endings are almost all timeouts) is
what supports it, and confirming it would need an instrumented run, not a log.

**So: merge_out's failures are not the §236 undercount, on this night — that
path never fired. The undercount that is still real and large sits on the
target whose jobs mostly die to a hard `-timeout` kill, and it is close to a
whole job's content per timeout, not the bounded "last second" the design
priced in. Why a slow night fails more is, on this evidence, "because a slow
night is one where a run runs long enough to be killed by `-timeout` instead of
ending on its own" — not "because create_dir_all or the lock fails more often."
§236.6's question about the write path specifically is left exactly where it
was: unfired, and so unanswered, on this night.**

*Since §262 (2026-09-26), the hypothesis in bold above is broken. A hung run
does not starve the merge thread. On a probe where nearly every job ended on a
`-timeout` kill, each kill lost about six runs, and the record read above
libFuzzer's count. The 1 104 jobs were counted correctly (§262.7). Why this
target's record holds about half its runs is still open, as Q150.*

### 256.5 What is left

- **The starved-merge-thread hypothesis in §256.4 is not run down.** It would
  need an instrumented fuzz run — a target that sleeps deliberately past
  `-timeout` under `-fork=N` with `RCAD_FUZZ_TIMINGS` set, checked for whether
  its merge thread's tick counter advances during the hang — not another
  night's logs.
- **No night yet has run Q49 while also being a 09-21-style slow night for the
  cone or fuse targets** (this night's cone p95 was 1.114 ms and fuse's was
  46.137 ms, both far below 09-21's seconds-long p99s in §236.3). §236.6's
  literal question — does a failing merge tick happen more on a slow night —
  still has no night where it was tested, because no merge tick has failed
  since Q49 landed.
- **`where_two_surfaces_cross`'s 951-of-1104 timeout rate this night is itself
  worth a queue row** — it is a much higher share than 09-21's fuse target
  carried, and nothing here asks whether that is this target's new normal or
  this one night's corpus.
