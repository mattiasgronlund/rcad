## 236. Why the timing record undercounts on a slow fuzzing night — 2026-09-25

*§227.5's last item, followed up: the cone-or-torus target's timing record for
2026-09-21 held 1 009 847 runs where libFuzzer itself counted 2 298 296. Not a
cone-target bug and not a fork-mode bug on their own — the same night's fuse
target loses runs too, and the same fork mode on a fast night does not.*

  236.1   What was checked, and against what
  236.2   Timeout, OOM and crash are not it
  236.3   The designed per-job loss explains 09-20 almost exactly, and is far
          too small for 09-21
  236.4   The same night's fuse target loses runs too — not the cone
          target's code
  236.5   The shape of the loss, and the one code path that fits it
  236.6   What is left, and where a fix would go

### 236.1 What was checked, and against what

Four files, read only, for the night of 2026-09-21 and, for contrast,
2026-09-20: `~/rcad-fuzz-logs/<date>/fuzz-cones-and-tori.log` and
`~/rcad-fuzz-logs/<date>/fuzz-fuse.log` (libFuzzer's own log, one status line
every so often, format `#<execs>: cov: … oom/timeout/crash: A/B/C time: Ns
job: N`), and `~/rcad-fuzz-logs/<date>/timings/<target>/report`, written by
`crates/rcad-fuzz-timing/src/lib.rs` (read in full). Both nights ran
`-fork=4 -timeout=10 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1`,
confirmed from each log's `Running` line, so fork degree is held fixed
throughout.

### 236.2 Timeout, OOM and crash are not it

The candidate named in the queue row — a fork child killed by `-timeout`,
`-rss_limit` or a crash before it flushes its record — is the one thing the
log itself answers directly, because libFuzzer prints
`oom/timeout/crash: 0/0/0` (or a nonzero count, if one ever fired) on every
status line. Grepped whole:

- `2026-09-21/fuzz-cones-and-tori.log`: 499 status lines, 499 read
  `oom/timeout/crash: 0/0/0`. Zero.
- `2026-09-21/fuzz-fuse.log`: 487 status lines, 487 read `0/0/0`. Zero.

No process died of a hang, an OOM or a crash on the night the undercount is
worst. Every one of the 499 (cone) and 487 (fuse) child processes that night
ended some other way — libFuzzer's own per-job rotation, not a kill libFuzzer
itself is willing to call a fault.

### 236.3 The designed per-job loss explains 09-20 almost exactly, and is far too small for 09-21

The module's own doc comment says the loss this design accepts is bounded:
merging every second (not every thirty) means "a process only ever loses what
it has seen since its last merge" (`crates/rcad-fuzz-timing/src/lib.rs:114-125`),
and `-fork` kills children from outside with no exit to hook
(same file, lines 32-49). That bound is a **per-job-teardown** loss of at most
about a second's worth of runs, at whatever rate that job was running.

**09-20 fits that bound almost exactly.** The report reads 159 727 331 runs;
libFuzzer's own last status line reads `#160488342 … job: 512`. Missing:
761 011 runs, 0.47% of libFuzzer's count, across 512 job endings — about
1 486 runs per ending. The run rate at those endings ranges from a few
thousand a second early to 53/s at the very last line, so "under a second's
worth of runs, once per job" covers the loss with room to spare. The missing
*seconds* (115 092 process-seconds against 101 661.3 recorded, 11.7%) are
close to the fuse's own 5-9% overhead figure in §227.3 and are not, on their
own, evidence of anything beyond ordinary fork-mode bookkeeping.

**09-21 is not explained by the same bound.** The report reads 1 009 847
runs; libFuzzer's last line reads `#2298296 … job: 499`. Missing: 1 288 449
runs, 56.1% of libFuzzer's count, across 499 job endings — about 2 582 runs
per ending, which is *more* than the ~2 024 runs each ending's job
successfully recorded on average. 09-21's own exec rate never exceeds
double digits a second for the whole night (every status line reads `exec/s`
in the ones or tens), so "under a second's worth of runs, once per job"
bounds the loss at roughly 499 × 50 ≈ 25 000 runs at the most generous
reading — about 1.9% of what is actually missing. Missing seconds tell the
same story: 115 332 process-seconds (4 × 28 833s, from `INFO: fuzzed for
28833 seconds`) against 48 527.8 recorded is 66 804.2 missing, 57.9% —
matching the missing-run share almost exactly (56.1%), which the per-job
bound does not predict at all: a bound of "the last second of a ~58-second
job" should cost a couple of percent, not well over half.

### 236.4 The same night's fuse target loses runs too — not the cone target's code

If the cause were specific to `where_a_cone_or_a_torus_crosses` — its own
inputs, its own call into `every_oracle_holds_of` — a different target on the
same machine, same night, same fork degree should not show it. It does.
`two_solids_fused_by_a_stranger`'s 09-21 report
(`2026-09-21/timings/two_solids_fused_by_a_stranger/report`) reads 1 303 505
runs, 91 444.7 s. libFuzzer's own last line for that log reads `#2047832 …
job: 487`, `INFO: fuzzed for 28806 seconds`. Missing: 744 327 runs, 36.3% of
libFuzzer's count; missing process-seconds 23 779.3 of 115 224, 20.6%.

**This is the same "1.30 M" row §227.3's table reports for the fuse on
09-21** — read from this same file, but never checked there against
libFuzzer's own count the way §227.3 checked 09-18 and 09-25 (which it found
agreed within 2%, and leaned on for that reason). It does not agree here: the
fuse's true 09-21 activity was 2 047 832 runs, not 1 303 505. §227.3's
qualitative point — that night's run count is set by how much of it goes on
slow inputs — still stands even at the corrected count (2.05 M is still far
below 09-22's 36.7 M), but the exact figure in that row understates the
night's real work by more than a third.

So the loss is not the cone target's code: it hit two independent targets,
built from different fuzz harnesses, sharing only the same crate
(`rcad-fuzz-timing`) and the same kind of night — one where a run
frequently takes hundreds of milliseconds to seconds rather than
microseconds. The cone target's p99 that night was 2 282 ms (§227.2); the
fuse's was 1 007 ms — both far above either target's fast-night p99 (09-20
cone: 1.245 ms; 09-18/09-25 fuse, per §227.3, well under that too). The two
nights and targets that show large loss are exactly the two that spent the
night mostly inside slow individual runs; the two that do not are exactly
the two that did not.

### 236.5 The shape of the loss, and the one code path that fits it

The loss is not concentrated in a few catastrophically slow runs. For the
cone target on 09-21: mean duration of the runs the record *did* keep is
48 527.8 s / 1 009 847 = 48.1 ms; mean duration implied for the runs it
*missed*, if all 66 804.2 missing seconds went to the 1 288 449 missing runs,
is 66 804.2 s / 1 288 449 = 51.9 ms — close to the kept mean, not an order of
magnitude above it. A "one huge run lost per job, killed mid-flight" story
would show a missing-run mean far above the kept one; this shows almost the
same mean, which reads as an **ongoing** loss through a job's life rather
than a loss only at its last moment.

The one place in `crates/rcad-fuzz-timing/src/lib.rs` shaped like an ongoing,
silent loss is `merge_out` (lines 170-216). It takes this process's
unmerged delta *out* of the live accumulator unconditionally, at the top of
the locked block:

```
let mine = std::mem::replace(&mut watch.took, HowLongRunsTook::new());   // line 180
```

and only afterwards attempts the disk work that would actually preserve it —
`std::fs::create_dir_all(&slowest)` (line 198) and
`the_shared_report_locked(&into)` (lines 201-203, opening and `flock`-ing the
shared `lock` file). Both are written to fail closed: `if …is_err() {
return; }` and `let Some(lock) = … else { return; };`. On either failure,
`mine` — the delta already pulled out of `watch.took` — is simply dropped:
never added back to the in-memory accumulator, never retried on the next
tick. The module's own doc comment says failures here are deliberately
swallowed ("a fuzz target that stopped fuzzing because its measurements
could not be filed would be measuring the disk," lines 168-169), but
swallowing the *error* and discarding the *data* are two different things —
the code does both. A merge tick that fails this way costs the process
everything it had recorded since its previous successful tick, not just the
current one's delta, and it costs it permanently.

This is a candidate that fits the size and the shape of what the records
show — a loss that scales with how many ticks fail rather than with how many
jobs end, and that lands roughly evenly across fast and slow runs rather
than singling out the multi-second tail. It is not proven from the records
alone: nothing here shows *why* `create_dir_all` or the lock/open would fail
more often on a slow night than a fast one at the same fork degree, and nothing
instruments how often either return fires. `std::fs::File::lock()` is
documented to block rather than fail under ordinary contention, so a plain
contention story does not obviously explain a rising failure rate either.
That link — slow night to failing merge tick — is this row's open question,
not its finding.

### 236.6 What is left, and where a fix would go

Not run, per the brief: only described.

- **The `merge_out` early-return paths silently drop already-collected data**
  instead of returning it to `watch.took` for the next tick to retry. The
  fix is local to `crates/rcad-fuzz-timing/src/lib.rs`'s `merge_out`
  (lines 170-216): on either `create_dir_all` or
  `the_shared_report_locked` failing, merge `mine` back into `watch.took`
  (`watch.lock()...; watch.took.add(&mine);`, mirroring `HowLongRunsTook::add`
  at lines 351-358) before returning, rather than letting it fall out of
  scope. The slow-run bytes already collected in `to_write` would need the
  same treatment, or a note that they are still safe to drop once (a slow
  input is likely to be re-offered later, where a merged run-count never is).
- **Whether that fix actually explains the size of the loss is untested.**
  A next lane could add a counter of failed `create_dir_all`/lock attempts to
  the existing crate tests, or instrument one real fork run against a
  deliberately slow target, and compare the failure count against a night's
  known deficit the way this row compared run counts.
- **Why the failure rate would rise on a slow night is not established.**
  Named here as the open half of the mechanism, not answered.
- **The corpus files themselves were not touched.** This row read logs and
  the timing crate only, never built or ran anything in
  `rcad-nightly-fuzz`, per the lane's read-only limit.
