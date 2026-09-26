## 260. `where_two_surfaces_cross`'s 86% timeout share on 2026-09-26 is not new — it has been 78–87% every night since `-fork` mode began

*§256.5's last item, followed up: is the night of 2026-09-26, where 951 of
`where_two_surfaces_cross`'s 1,104 job-endings died to the 10-second
`-timeout`, this target's normal, or did it jump, and if it jumped, when and
with what. Read from the nights' own logs, going back as far as the log
format supports an answer: it never jumped. The first night this can be
measured at all — 2026-09-20, the night `-fork` mode itself landed — already
carried an 81% share, and every night since sits in the same 78–87% band. This
is the target `docs/decisions.d/135-*.md` already named as having a slow tail
so extreme that the 10-second bound was written specifically to let it keep
losing to `-timeout` rather than exclude those inputs from the corpus.*

  260.1   Why 2026-09-20 is as far back as this question can go
  260.2   Reproducing 09-26's 951 of 1,104, from the file
  260.3   The same read, six more nights back to the mechanism's first night
  260.4   Nothing else moved with it: same flags, same target file, no tracked
          corpus change
  260.5   Why this target's tail was already named, before fork mode existed
  260.6   What is left

### 260.1 Why 2026-09-20 is as far back as this question can go

`~/rcad-fuzz-logs/` holds a directory per night back to 2026-09-09
(`ls ~/rcad-fuzz-logs/`). `where_two_surfaces_cross`'s log (`fuzz.log`, no
`fuzz-` prefix) exists for every night except 2026-09-15, whose directory
holds only `build.log` and an empty `watermark.log` —
`tail ~/rcad-fuzz-logs/2026-09-15/build.log` reads `cargo: command not found`,
so nothing ran that night.

But a "job" and a per-job `-timeout` kill are `-fork`-mode concepts, and
`-fork` was not always on. `grep -m1 '^     Running' ~/rcad-fuzz-logs/2026-09-19/fuzz.log`
prints a command line with no `-fork` and no `-timeout` at all — a single
long-lived process, same as every night from 09-09 through 09-19. The same
grep against `~/rcad-fuzz-logs/2026-09-20/fuzz.log` prints
`-fork=4 -timeout=10 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1` for
the first time. Those nights' own reflog checkouts (below) confirm why: the
night of 09-19 ran commit `5336ff31`, the night of 09-20 ran `146d0d62`, and
`git -C ~/git/mattiasgronlund/rcad-nightly-fuzz log --oneline 5336ff31..146d0d62`
contains `22684dad feat(fuzz): the night forks, and stops reading a crash off
the exit status` and `52ea7984 perf(fuzz): one input gets ten seconds, where
it used to get twenty`, both dated 2026-09-18 (`git log -1 --format=%ad
22684dad` / `52ea7984`) — written the day before they first ran overnight.
Before a single-process night has no job boundaries and no `-timeout` kill to
count; **2026-09-20 is the first night the question in this section's title
can be asked of the logs at all**, not a choice of window.

### 260.2 Reproducing 09-26's 951 of 1,104, from the file

`grep -E 'oom/timeout/crash' ~/rcad-fuzz-logs/2026-09-26/fuzz.log | tail -1`
prints:

```
#592047: cov: 4002 ft: 14455 corp: 1161 exec/s: 1 oom/timeout/crash: 0/951/0 time: 28715s job: 1102 dft_time: 0
```

— `0/951/0` for oom/timeout/crash, matching the row and §256.4 exactly.
`grep -oE 'job: [0-9]+' ~/rcad-fuzz-logs/2026-09-26/fuzz.log | awk '{print $2}' | sort -n | tail -1`
prints `1104` — the job field is not printed in strict order across the four
`-fork` workers (the five lines immediately above the last one read job 1101,
1093, 1103, 1104, 1102 in that order), so the last-printed line's own job
number (1102) understates the total by two; 1104 is the count that matches
what the row names and what §256.3's "job (last seen)" column used, so this
section uses **max job number in the file**, not the last line's job field,
as the per-night job-ending total throughout. `951 / 1104` (`echo
'951/1104*100' | bc -l`) is 86.14%.

### 260.3 The same read, six more nights back to the mechanism's first night

Same two greps, one per night, 2026-09-20 through 2026-09-25 — the last
`oom/timeout/crash` line's timeout figure, and the max `job:` value in the
file:

| night | commit | job-endings (max `job:`) | timeout job-endings | share |
|---|---|---:|---:|---:|
| 2026-09-20 | `146d0d62` | 835 | 677 | **81.08%** |
| 2026-09-21 | `ae124a29` | 797 | 623 | **78.17%** |
| 2026-09-22 | `79eedce7` | 907 | 791 | **87.21%** |
| 2026-09-23 | `055aa4a9` | 810 | 638 | **78.77%** |
| 2026-09-24 | `afdff061` | 908 | 775 | **85.35%** |
| 2026-09-25 | `6664fef8` | 969 | 837 | **86.38%** |
| 2026-09-26 | `c92b8fe5` | 1104 | 951 | **86.14%** |

Each row's job-endings and timeout count came from
`grep -E 'oom/timeout/crash' ~/rcad-fuzz-logs/2026-09-<DD>/fuzz.log | tail -1`
and `grep -oE 'job: [0-9]+' ~/rcad-fuzz-logs/2026-09-<DD>/fuzz.log | awk
'{print $2}' | sort -n | tail -1`; each share from `echo
'<timeout>/<job>*100' | bc -l`. The commit for each night is the one its
`~/git/mattiasgronlund/rcad-nightly-fuzz` reflog shows checked out nearest
that night's 00:10 (`git -C ~/git/mattiasgronlund/rcad-nightly-fuzz reflog
show --date=iso HEAD`) — the one mid-evening checkout in that log
(`a3f8ffe9` at 2026-09-20 21:11:06) is not a night's own run and is excluded.

**86.14% on 09-26 is the second-highest of the seven, not a new high and not
a jump**: 09-22, five nights earlier, already reached 87.21%, and the very
first night the mechanism ran (09-20) was already at 81.08% — well inside the
range every later night stayed in. The seven nights bounce between 78.17%
and 87.21% with no monotonic climb (day-by-day: 81.08, 78.17, 87.21, 78.77,
85.35, 86.38, 86.14) — the last three nights do sit in a tighter 85.35–86.38%
band than the first four's wider 78.17–87.21% spread, but 09-22 alone already
matches that band's top, so seven points is not enough to call that tightening
a trend rather than noise, and this section does not claim one.

### 260.4 Nothing else moved with it: same flags, same target file, no tracked corpus change

For every one of the seven nights, `grep -m1 '^     Running'
~/rcad-fuzz-logs/2026-09-<DD>/fuzz.log` prints the same
`-fork=4 -timeout=10 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1`, and
`-max_total_time` within 28715–28804 — the same nightly window and the same
fuzzer settings throughout; the share is not moving because a flag moved.

`git -C ~/git/mattiasgronlund/rcad-nightly-fuzz log --oneline
146d0d62..c92b8fe5 -- fuzz/fuzz_targets/where_two_surfaces_cross.rs` and the
same command against `crates/rcad-geom/tests/fuzzing/mod.rs` (the file the
target `#[path]`-includes, per that file's own header) both print nothing:
across all seven nights, no commit touched the target's own harness. `git -C
~/git/mattiasgronlund/rcad-nightly-fuzz log --oneline 5336ff31..c92b8fe5 --
fuzz/corpus/where_two_surfaces_cross` (the tracked seed corpus) also prints
nothing — it did not change across the whole window either.

The fuzzer's own persistent corpus (`/home/mattias/rcad-fuzz-corpus/where_two_surfaces_cross`,
untracked, separate from the seed corpus above) did keep growing:
`find /home/mattias/rcad-fuzz-corpus/where_two_surfaces_cross -type f
-printf '%TY-%Tm-%Td\n' | sort | uniq -c` shows 172, 75, 192, 153, 98, 91 and
92 new files landing on the nights of 09-20 through 09-26 respectively — a
steady trickle, not a step. Nothing here is a change that lines up with any
of the share's night-to-night moves.

### 260.5 Why this target's tail was already named, before fork mode existed

`docs/decisions.d/135-*.md`, written the day before `-fork` mode's first
overnight run, measured this exact target on 2026-09-18's single-process
night: 99% of its inputs cost at most 428 of 5,774 seconds, and the remaining
1% — 2,592 runs — cost the other 93%, ending in 64 runs every one of which
ran 17.06 to 19.59 seconds (§135.1). §135.2 chose `-timeout=10` over a tighter
bound *because* a tight bound would silently exclude from the corpus "the
nearly-parallel crossings" that are this target's whole reason to exist,
and said so: "the bound is deliberately loose: it is there to stop one input
eating twenty seconds, not to make the fuzzer fast." §135.4 went further —
"a `timeout-` file on `where_two_surfaces_cross` is a nightly certainty and
not news."

This section did not re-run that instrumentation; it takes §135's own numbers
as read and reports them as the standing explanation for why this target,
alone among the five, has run at 78–87% timeout job-endings every measured
night rather than a defect that appeared on one of them. **That the corpus
already held (and keeps growing) inputs in the 10-second range explains why
a large, fairly stable share of `-fork` job-endings hit the timeout; it does
not, by itself, explain why the share sits specifically in the high-70s to
high-80s rather than some other band** — that finer claim is not run down
here and would need knowing what fraction of the corpus's ~1,700 files
(`ls /home/mattias/rcad-fuzz-corpus/where_two_surfaces_cross | wc -l`) are
individually slow enough to time a job out, which these logs do not record
per-input.

### 260.6 What is left

- **Why the share sits at roughly 80%, rather than some other stable value,
  is not answered here.** It would need per-input timing inside a `-fork`
  job — which input each job drew and whether it was one of the slow ones —
  not another night's cumulative log line.
- **The mild tightening noted in §260.3** (last three nights 85–86%, versus
  the first four's wider 78–87% spread) **is not established as a trend.**
  Seven points and one early outlier on the high side (09-22) are not enough
  to separate a real narrowing from noise; a few more nights' cumulative
  lines, read the same way, would settle it cheaply.
- The two `crates/rcad-geom/src` commits named in this window that touch
  surface-crossing code (`3a853568`, `65785037`, plus `73e5863a` and
  `a7df89e2`) were checked for landing on the target's own harness file and
  found not to touch it (§260.4); whether any of them changed the *speed* of
  the surface-crossing path itself, as opposed to its harness, was not
  checked, and nothing in §260.3's table lines up with any of their dates
  closely enough to suggest it matters.
