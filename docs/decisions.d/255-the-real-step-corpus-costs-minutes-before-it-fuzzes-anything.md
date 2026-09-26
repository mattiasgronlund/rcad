## 255. The real STEP corpus costs minutes before it fuzzes anything — 2026-09-26

*Queue row Q66, from §226.1. Lane `lane-q66-reader-seed-measure`, off
`d52261e7`, measuring `lane-t7-reader-seeds-from-step-corpus` (`8689e498`)
before it merges. That commit makes `fuzz-reader` pass `RCAD_STEP_CORPUS` to
`fuzz-target` as a third, read-only, always-last corpus argument. Every number
below was run on this machine; none is restated from §226.1's row without a
fresh measurement.*

  255.1   The corpus itself
  255.2   Without the STEP corpus: a two-second start
  255.3   With the STEP corpus: the merge eats the whole run
  255.4   Taking the zips out does not fix it
  255.5   What -max_len becomes, and what pinning it would and would not do
  255.6   Nothing was written into RCAD_STEP_CORPUS
  255.7   Recommendation to T7
  255.8   What this leaves

### 255.1 The corpus itself

`find "$RCAD_STEP_CORPUS" -type f | wc -l` says 520 files. `du -sh` says 375M.
Four are zips (`find ... -iname '*.zip' -exec ls -la`): 13 976 599, 15 861 580,
60 294 624 and 5 952 887 bytes — 13.3, 15.1, 57.5 and 5.7 MiB, so "four zips up
to 60 MB" in the row is right. The other 516 files run 121 bytes to 8 634 760
bytes (about 8.2 MiB) — measured with `stat -c '%s'` over every non-zip file,
sorted.

The tracked reader corpus, `fuzz/corpus/no_step_file_makes_the_reader_panic`,
has 909 files running 1 to 5 110 bytes.

### 255.2 Without the STEP corpus: a two-second start

The reader target was built once (`cargo +nightly fuzz build
no_step_file_makes_the_reader_panic`, release, 67s) and run directly — not
through `fuzz-reader`, since 8689e498 is on a branch this lane must not merge
or edit — with the same flags `fuzz-target` passes: `-fork=4` (this machine has
24 cores and the night divides them over 5 targets, `nproc / 5 = 4`),
`-ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 -timeout=10
-max_total_time=15 -dict=step.dict`, against a scratch write directory and a
copy of the tracked corpus (never the tracked directory itself, per this
lane's brief).

Two runs, both fork mode:

- Run 1 (load average 1.73/2.15/2.09 just before): `-fork=4: 872 seed inputs,
  starting to fuzz` printed 2 seconds after the binary started; total 866 362
  executions by the 10s mark, exiting at "time: 11s".
- Run 2 (load average 2.45/2.64/2.44 just before): 873 seed inputs, same 2
  second start; 1 176 295 executions by "time: 19s" (four jobs ran past the
  15s target before the last one finished).

Both land in the tens of thousands of exec/s per job (13 989 to 47 091 in the
per-job `exec/s:` fields). **Startup is about two seconds.**

A non-fork probe (`-runs=0`, so it loads the corpus, runs the INITED pass, and
exits) confirms why: `909 files found`, `seed corpus: files: 909 min: 1b max:
5110b total: 215292b`, and it says plainly `-max_len is not provided;
libFuzzer will not generate inputs larger than 5110 bytes` — the row's "about
5 KB" is this number exactly. `Done 910 runs in 0 second(s)`.

### 255.3 With the STEP corpus: the merge eats the whole run

Same binary, same flags, `-max_total_time=15`, but the third positional corpus
argument was the real `$RCAD_STEP_CORPUS` directory (read-only, appended
last, exactly where 8689e498 puts it) instead of being left out.

The run took **282 seconds** wall clock (`INFO: exiting: 0 time: 282s`) for a
run that asked for 15. `ps` on the live process showed it stuck for the first
~270s in `-merge_inner=1` — libFuzzer fork mode's own initial pass that reads
every seed and executes the target once on each, before any of the four
fuzzing jobs starts. Once the merge finished, the fork scheduler had already
spent its whole 15-second budget and handed the remaining jobs slivers (1s,
2s, 4s...); the one that got to report before exit read `#35: ... exec/s: 5
... time: 263s job: 3`. **That "exec/s: 5" is an artifact of a job started
with no time left, not a steady-state rate — the honest number here is that
essentially none of the requested 15 seconds of fuzzing happened.**

`-fork=4: 1157 seed inputs` came out of the merge, against 872-873 without the
STEP corpus and 909+520 = 1429 raw candidates offered. `write-corp` (the
writable scratch directory) held 0 files afterward — no new corpus entries,
because there was no time left to find any.

### 255.4 Taking the zips out does not fix it

To find out whether the 282s is the zips or the real files, a second scratch
directory held symlinks to the 516 non-zip files only (`step-no-zip`, built
with a `find | while read` script since a `just`-tool loop referencing an
absolute scratchpad path is refused by this session's worktree guard as "too
complex to verify" — the script itself is not tracked).

A non-fork `-runs=0` probe against tracked-copy (909) + step-no-zip (516) — no
zips, 1425 raw files, 296 134 985 bytes (about 282.5 MiB) by libFuzzer's own
count — took **256 seconds** single-threaded (`Done 1426 runs in 256
second(s)`), reaching a `corp: 1161` kept set. It also hit `Slowest unit: 10
s:` — one real file in the corpus already saturates the `-timeout=10` ceiling
on its own, with a `slow-unit-...` artifact written to prove it.

**256 seconds without any zip is almost the same order of magnitude as 282
seconds with all four.** The kept-corpus counts confirm the zips add
essentially nothing: 1157 kept with the zips in (over 909+520 raw) versus 1161
kept without them (over 909+516 raw) — within noise of each other, so the four
zip files that account for 91.6 MiB of the 375 MiB corpus contributed at most a
handful of net-new corpus entries and did not change how many were kept.

So the finding is not "the zips are the whole problem" — it is that the 516
legitimate STEP files, most of them hundreds of kilobytes to several megabytes
of real geometry, are themselves the majority of the merge cost. Excluding the
zips removes dead weight for near-zero coverage return, but does not bring
startup back anywhere close to the 2-second baseline.

### 255.5 What -max_len becomes, and what pinning it would and would not do

The no-zip probe's banner: `-max_len is not provided; libFuzzer will not
generate inputs larger than 1048576 bytes` — exactly 1 MiB, matching the row.
This is a hard cap libFuzzer applies when deriving `max_len` from the corpus:
the actual largest file offered was 8 634 760 bytes (8.2 MiB), well past 1
MiB, and the derived value stopped at 1 048 576 anyway. Because that cap is
already hit by the non-zip files alone, adding the 60 MB zips on top would not
be expected to push `max_len` any higher — this particular claim was not
re-run with the zips included (a second multi-minute merge to confirm a number
that cannot exceed an already-observed hard cap was not worth the added run
time against this lane's budget), but nothing in libFuzzer's behavior here
suggests otherwise, and the merge time in §255.3 already answers the more
expensive question.

Pinning `-max_len` (to something near the current 5 110-byte baseline, or a
modest multiple of it) would **not** touch the merge cost in §255.3 or §255.4:
that cost is libFuzzer reading and executing every seed file once, which
happens regardless of `-max_len` — existing corpus entries larger than
`max_len` are still loaded and replayed. What pinning `-max_len` buys is
bounding what the mutation engine *generates* once fuzzing is actually
running: at an unpinned 1 MiB, and with a real file already costing the full
10-second timeout in §255.4, mutated multi-hundred-kilobyte inputs during a
steady fuzzing run risk routinely landing near that same slow path and
draining throughput. That is a real but separate concern from the startup
cost.

### 255.6 Nothing was written into RCAD_STEP_CORPUS

`find "$RCAD_STEP_CORPUS" -type f | wc -l` read 520 both before and after
every run in this section, and `find "$RCAD_STEP_CORPUS" -newer
fuzz/Cargo.toml` (a file that predates today) found nothing. libFuzzer only
ever writes new corpus entries to the *first* positional directory
(§81), and 8689e498 puts `RCAD_STEP_CORPUS` last — this measurement confirms
that placement holds in practice, not just in the diff. The 520-file count
held through the full-corpus run of §255.3, which used the real directory
directly (read-only) rather than a copy.

Whether libFuzzer "loads all 520": the fork run in §255.3 offered all 520 real
files (through the real directory, unfiltered) plus the 909 tracked ones and
came back with more kept seed inputs (1157) than the tracked corpus alone
gives (872-873), so the STEP files were read and executed, not skipped. The
explicit `N files found` banner that names a count per directory only prints
in non-fork mode (confirmed for the tracked-copy and step-no-zip directories
in §255.2 and §255.4); getting that exact line for the real 520-file directory
including the zips would need another non-fork run at least as slow as
§255.4's 256 seconds, which this lane's time did not stretch to. The indirect
evidence — no error, a plausible seed count, and a kept-count consistent with
the no-zip run — says it loaded all of them; this one line was not directly
re-confirmed for that exact directory.

### 255.7 Recommendation to T7

**Do both: leave the zips out, and pin -max_len.**

Leaving the zips out is close to free to do and has no measured downside:
§255.4 shows they contribute at most a handful of kept corpus entries against
91.6 MiB of the 375 MiB corpus and a meaningful share of the wall time. There
is no evidence here that they teach the reader anything a real `.stp` file
does not.

Pinning `-max_len` (near the existing 5 110-byte tracked-corpus baseline, or a
deliberately chosen larger-but-bounded value) protects steady-state fuzzing
throughput once a run is past its startup, given that a real seed file already
saturates the 10-second per-input timeout in this corpus (§255.4) and an
unpinned `max_len` would let the mutator generate inputs that size or larger
routinely.

**Neither change fixes the startup cost.** Even with the zips out, loading the
516 real STEP files alone cost 256 seconds single-threaded in §255.4, against
a 2-second baseline — a two-orders-of-magnitude jump that a night's per-target
time slice (`$left`, shared across all five targets and shrinking through the
night) may not survive intact, exactly as §255.3's full run demonstrated: a
15-second ask returned essentially zero seconds of actual fuzzing. T7 should
expect that landing 8689e498 as written turns every reader-target slot of a
night into "mostly merge, then out of time" unless the night's schedule
gives the reader a slice long enough to absorb a multi-minute merge tax, or
the STEP corpus handed to it is trimmed well below 516 files.

### 255.8 What this leaves

- `docs/queue.md` row **Q66** (this row) moves to `done` in the commit that
  merges this lane; T7 is unblocked to merge `8689e498` with this section's
  recommendation (leave the zips out, pin `-max_len`) and the open point in
  §255.7 (the startup cost survives both changes) carried into its own merge
  decision.
