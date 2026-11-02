## 475. The fuzz night pauses from 02:50 to 04:00, so inari can measure on an idle host — 2026-11-01

*Mattias, 2026-10-31, in inari's spec review (item X4, "Rough benchmarks on
loaded hosts"): "I suppose we should update the read nightly fuzz runs to stop
for an hour somewhere between 3AM and 4AM in the morning allowing inari a slot
for nightly idle host measurements." Asked in the inari session how, he answered
"Yes, option (a)" to: a templated `rcad-fuzz@.service`, run at 00:10 for 9600 s
and at 04:00 for 14400 s, with a note in §81. On 2026-11-01 he asked that
session to pass X4 to this one. The way it is done is decided by session
`fuzz-split-for-inari` under `CLAUDE.md`'s "Decide it yourself". Amends §81.*

**In short.** The hour is kept, and so are 02:50 and 04:00. Option (a)'s two
runs are not: the night stays one run of `fuzz-overnight`, started by the one
timer at 00:10, and the recipe stops every target at 02:50 and starts them again
at 04:00. Two runs would write one night's directory twice, and the second would
hide what the first found.

### 475.1 Why not two runs

A night's directory is named by its date:
`night="{{fuzz_log_root}}/$(date +%Y-%m-%d)"`. Runs at 00:10 and 04:00 have the
same date, so the 04:00 run writes into the directory the 00:10 run used. Then:

- `>"$night/$recipe.log"` empties each target's log, and the first run's lines
  are gone.
- `artifact-names-before/` and `corpus-names/` are listed again at 04:00. What
  the first run found is on those lists, so the summary does not count it as
  new. A crash found before 02:50 would not be in `latest` and would not fail
  the night, which is what §134 made a crash file do.
- Each run writes `latest`, so the 08:30 triage reads only the second run's line.

Fixing that means teaching the recipe that one night can have two runs. That is
most of the work a pause inside one run needs, and the template's own costs come
on top of it:

- **The time limit cannot come from the instance name.** systemd does not expand
  `%i` in `TimeoutStartSec=`. On this machine (systemd 255),
  `systemd-analyze --user verify` on a template holding `TimeoutStartSec=%i`
  says `Failed to parse TimeoutStartSec= parameter, ignoring: %i`. Ignored, the
  setting falls back to a oneshot's default, which is no limit at all: §81.5's
  failure again. Each instance would need a drop-in file of its own.
- **Two instances are two units.** systemd would start the 04:00 run beside a
  00:10 run that had not stopped, in the same tree.
- **Four checks read the unit by name.** `check-systemd-units` pairs
  `<name>.service` with `<name>.timer`. `check-triage-waits-for-the-night` reads
  `rcad-fuzz.timer` and `rcad-fuzz.service`. `clean-targets` and
  `prune-worktrees` read `WorkingDirectory=` out of `rcad-fuzz.service`. Each
  would change, with its selftest.
- **The 04:00 run would build again and could move the tree** onto a newer tip
  (§186), so one night would fuzz two commits. `fuzz-yield` finds a night's
  commit by the one `checkout` in the reflog between 00:10 and 00:16, and would
  miss the second.

With a pause inside one run, the timer, the time limit, those four checks and
the night's one commit all stay as they are.

### 475.2 How much of the hour is idle

The last five nights before this section, 2026-10-27 to 10-31, show how a
window ends. Each started at 00:10:45, so the window ended at 08:10:45. The
targets' logs were last written between 08:10:51 and 08:15:46: libFuzzer
finishes the job it is in, and on 10-31 `fuzz-writer` overran and was killed by
its `timeout`, which allows 300 s past the target's share and then 60 s more.
After the targets, linking the artifacts took 28 to 56 s and the reader's corpus
merge about 20 s.

So a target told to stop at 02:50 has stopped by 02:56 at the latest, because
its `timeout` kills it then. Linking and merging wait for the end of the night
and do not run in the hour. In the pause, only the recipe's own shell is left,
sleeping, and the five-minute `copy_out` stops with the targets.

Option (a)'s first run would have linked and merged inside the hour too, and
ended at about 02:58. systemd then took up to two more minutes to call the unit
stopped: on 10-30 and 10-31 the unit stopped 2 minutes after the recipe exited.

The night loses 4200 s of its 28 800. Option (a) would have given 9600 + 14 400
= 24 000 s, about the same, less a second build.

### 475.3 What the pause does not cover

It keeps the fuzz night out of the hour, and nothing else. A lane that a
session sends at 03:00 still runs. inari's job starts only when `load1` is below
1.5, and that check is the only thing that keeps a lane out of its measurement.

### 475.4 What gets built

- `fuzz-overnight` takes the pause as two clock times. A night whose window
  does not reach the first one runs as it does today, and so does a hand run
  given no pause.
- Every target gets until 02:50 as its share, and the same `timeout` as now.
  When all of them have stopped, `copy_out` runs once and the copier stops. At
  04:00 the copier starts again and every target gets the time left before the
  night's deadline.
- What is listed once at the start of a night is still listed once:
  `artifact-names-before/`, `corpus-names/`, `.started`. So the summary counts
  a crash from before the pause as new tonight. The build, the move to the tip
  (§186) and the hooked-calls check run once.
- A target's log holds both of its runs. Each run is checked for its own
  `INFO: exiting:` line, so a second run that never got fuzzing is not hidden
  by the first run's line. Every reader of a night's per-target logs counts both
  runs, `fuzz-yield` among them.
- `rcad-fuzz.service` passes the pause once the dispatcher installs it, as
  `ExecStart=%h/.local/bin/mise exec -- just fuzz-overnight 28800 02:50 04:00`.
  Its `TimeoutStartSec=` stays at 29400, because the night's deadline is still
  08:10.
- §81 gets a line saying the night now pauses, pointing here.

### 475.5 The first night with the pause fuzzed nothing

The unit was installed on 2026-11-01 (3e770449f). At 00:10 on 11-02 it failed
in the same second it started, and the journal holds one line from it:
``error: Justfile does not contain recipe `02:50` ``.

The night runs the `justfile` of its own tree, not `main`'s, and that tree was
still on `d0a1458d`, the 10-31 night's commit, where `fuzz-overnight` takes
only `window`. So `just` read `02:50` as a second recipe name and stopped before
the recipe ran. The recipe is also what moves the tree onto `main` (§186), so
the tree cannot get past this by itself. It will fail the same way every night
until somebody moves it.

The fix is the hand move that §81.4 keeps for this case, run once:
`git -C ~/git/mattiasgronlund/rcad-nightly-fuzz checkout --detach origin/main`.
After that the tree is on a commit whose recipe takes the pause, and that
recipe passes the pause on when it moves to a newer tip and runs again
(`pause_args` in `fuzz-overnight`). So the next night works, and so does every
night after it.

**The rule this leaves:** a new argument in `rcad-fuzz.service` waits until
the night's tree is on a commit whose recipe takes that argument. The gate
cannot see this. `check-systemd-units` compares the unit with `tooling/systemd/`,
not with the night's tree, so both commits here passed it. This is §108's
shape again: a night that fuzzed nothing because of what it found on its PATH,
this time because of what it found in its own tree.
