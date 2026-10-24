## 372. Three lane slots, and a gate that runs its steps at once — 2026-09-29

*Mattias decided this on 09-29, and the number three is session rcad-cf's,
taken on his reasoning. It replaces §230.8's six. Every figure below was
measured by rcad-cf on 09-29, on 24 cores with a warm tree. The timing scripts
and their per-step CSVs are in the `ci-verdict-cache` tree's `target/`
(`gate-step-times.sh`, `gate-parallel.sh`, `gate-ordered.sh`, `served-both.sh`,
and `target/gate-times/<run>/steps.csv`), which is gitignored.*

In short: at six slots the machine was past its cores and half the mergers
had to merge `main` again and re-gate before they could push. Both come from
the same product: how many mergers are gating, times how long a gate takes.
So the cap drops to three, and the gate learns to run its steps at once
(**Q713**, re-filed from Q710), which takes a full gate from 308 s to about
91 s on idle cores. Q713 goes before Q693.

### 372.1 What Mattias decided

His words: "we must enable parallel tests, as it would be better to limit the
number of parallel lanes and make sure that the gate is faster, as that would
minimize the risk for a queue where merges collide and are required to re-base
and re-gate".

Three parts: the gate runs its steps at once, fewer lanes run, and the faster
gate comes first, because a faster gate is what makes merges collide less.

### 372.2 How often merges collide

Every merger result in every lanes run's journal
(`~/.claude/projects/*rcad*/*/subagents/workflows/*/journal.jsonl`, the
`result` records that carry `push_attempts`). A result has no timestamp, so a
merged one is dated by the commit date of its `merged_sha`, and the 52 that did
not merge by their journal's mtime.

| day | mergers | tried to push | needed more than one try | all three tries |
|---|---|---|---|---|
| 09-26 | 22 | 19 | 5 | 3 |
| 09-27 | 50 | 42 | 19 | 14 |
| 09-28 | 55 | 42 | 25 | 13 |
| 09-29 | 49 | 43 | 23 | 12 |
| all | 176 | 146 | 72 | 42 |

"Tried to push" is `push_attempts` of one or more. Every day ran under
§230.8's six slots, so this is the baseline and not a comparison: the journals
do not say how many lanes were running when each merger gated.

A try that loses the race merges `origin/main` again and gates again. When the
other merge brought code, that is a whole gate: 308 s in turn on idle cores
(§372.3), and more on busy ones.

### 372.3 What running the steps at once buys

| run | steps | wall | CPU-seconds | load at start |
|---|---|---|---|---|
| in turn (`serial-1`) | 56 | 308.3 s | 1384 | |
| all started together (`parallel-1`) | 56 | 110.0 s | 1320 | 14.5 |
| cargo steps chained, the rest beside them (`ordered-1`) | 56 | 91.2 s | 921 | 8.0 |
| a served gate in turn (`served-serial-1..3`) | 14 | 18.5–19.2 s | 21 | |
| a served gate at once (`served-parallel-1..3`) | 14 | 6.4 s | 22–23 | 8.6–15.0 |
| after Q693, in turn (`post-q693-serial`) | 6 | 9.0 s | 9 | |
| after Q693, at once (`post-q693-parallel-1..3`) | 6 | 3.6–3.8 s | 9 | 52.7–56.5 |

The chain in `ordered-1` is the eight steps that run cargo, `test` first:
`test check-ignored check check-fuzz-builds check-release-build
check-t-selftest check-wildcards lint`. `test`'s log shows seven waits for a
cargo file lock when all 56 started together, and four when chained.

In `ordered-1` the last step to finish was `check-fuzz-overnight-selftest`, at
91.1 s while spending 12 CPU-seconds, so it is asleep for most of it (**Q709**).
`test` came next at 73.5 s. Its slowest tests, the two
`a_corner_up_cube_round_…_apex_answers_right` tests in
`a_plane_cut_along_a_cones_other_nappe`, took 58 to 70 s each across the runs.

No step failed in the nine runs at once (`parallel-1`, `ordered-1`,
`ordered-2-after-touch`, and the six served ones). That is not proof it is
safe, and Q713 has to say what stopping at the first failure becomes.

**On busy cores it buys nothing.** A second chained run, `ordered-2-after-touch`,
took 224.5 s, and its `test` 183.8 s. It started at 21:15:49Z at a load of 11.8,
just after the `dispatch-next` dispatcher's fill locked three lane trees
(21:15:10Z), and as it ended the one-minute load was 56.5, read by the next run
under a second later. Its `test` also spent 1114 user CPU-seconds against
`ordered-1`'s 683, so it rebuilt more and is no measure of the layout. What it
shows is the load.

### 372.4 Why three

§230.8's own rule moves it: up while the one-minute load stays below the core
count with two gates overlapping, down when it does not. At about six slots
the load reached 56.5 on 24 cores.

Three is where §230.8 started, the `practices:delegating` rule of thumb. It is
also about what 24 cores hold once the gate runs its steps at once. `ordered-1`
averaged about ten cores (921 CPU-seconds over 91.2 s), so two such gates fill
most of the machine, and a third lane is usually editing or being verified
rather than gating. **That is reasoning, not a measurement.**

**Amended by §397.7 on 2026-10-03: four slots once the slow-tier row has
landed.** `sar` shows a busy window averaging 8 cores of 24 even at six slots;
spikes, not busy lanes, held the machine back. The counting below stands.

Read-only rows, verifiers and mergers still count toward the three, as they did
toward six: each builds and runs tests.

### 372.5 Q713 before Q693

Q710 waited on Q693, because both change the `gate` recipe. That order is
reversed. Q713 alone takes a full gate from 308 s to about 91 s. Q693 trims
only a served gate, and once Q713 is in, that is 6.4 s down to about 3.7 s. So
Q713 goes first, at the head of tier 2.2, and Q693 waits on it. Q713 is Q710
re-filed with the sentence about Q693 turned round, the count of runs at once
corrected from eight to nine, and this section as the one behind it.

### 372.6 What changed

- `queue_slots` in the `justfile` is `"3"`, and `check-queue-selftest`'s two
  lines that print the cap expect three.
- The `dispatch-lanes` skill says three and cites this section.
- The dispatcher holding `dispatch-next` (session rcad-12) agreed on 09-29 to
  take no new rows until fewer than three slots are held. Its five runs in
  flight then (Q667, Q419, Q705, Q661 and Q482) carry on.

### 372.7 What would move it

The table in §372.2, counted again over the days after this lands. Two things
confound it:

- §367 made a catch-up over docs alone cost no push attempt. It counts in
  `docs_catch_ups` instead, which only one of the 176 results carries so far.
  Compare `push_attempts` plus `docs_catch_ups` against this baseline, not
  `push_attempts` alone, or §367's change reads as the cap working.
- Q713 shortens the gate at the same time. That is Mattias's intent, and the
  two are not meant to be told apart.

Up to four if the rate of mergers needing more than one try falls well below
this baseline and the one-minute load stays under 24 with three lanes gating.
Down to two if the load still goes past the core count with the gate running
its steps at once.

**Amended by §397.7 on 2026-10-03: the load bar is replaced.** One `just t`
with no gate reached a run queue of 62, so no slot count could keep the load
under 24. The bar is now the share of busy daytime `sar` samples at a load of
24 or more, and catch-ups take the place of the push-attempt count.

### 372.8 What Q713 built

*Lane Q713, 2026-09-30. The figures in this subsection are its own, on 24
cores, each from `time mise exec -- just gate quiet …` with `uptime` read just
before.*

`gate` starts every step as soon as a slot is free, one slot per core. The
steps that build with cargo take one slot between them and run one after
another, in `gate_steps` order (§372.10). Each step writes its own log under
`target/last-gate.log.steps/`, and a step's output goes into `last-gate.log`
whole when the step ends, under a `===` line with its name and time.

| run | load at start | wall | CPU-seconds |
|---|---|---|---|
| fresh, this worktree's first build | 0.42 | 120.9 s | 1132 |
| fresh, warm | 8.55 | 91.2 s | 904 |
| served right after | 7.43 | 6.6 s | 23 |

In the warm run the chain's eight steps took 85.4 s together, `test` 69.9 s of
that, and the last step to end was `check-fuzz-overnight-selftest` at 89.7 s
(**Q709**). §372.3's `ordered-1` ran `test` first and took 91.2 s, so the order
inside the chain moves nothing on a warm tree. The first run was slower because
`check`, `lint` and `check-ignored` compiled: 8.6, 20.5 and 7.0 s against 2.9,
5.0 and 2.6 s in the warm one.

A third word sets the slot count: `just gate quiet fresh 1` runs the steps in
turn, the way to rule the change out if two steps are ever suspected of
disturbing each other.

### 372.9 A failure stops nothing

Every step runs to its end, and the block names every step that failed. `first
failure:` is the first failed step in `gate_steps`, not the first to end, so
the name does not depend on which step was quicker, and the exit code is that
step's. The rest are on an `also failed:` line. `quiet` prints the first
failure's last 60 lines, out of its own log.

The two other choices, and why not:

- **Stop starting new steps, and wait for the running ones.** This saves almost
  nothing. The two longest pieces, the cargo chain and
  `check-fuzz-overnight-selftest`, start in the first second, so a failure found
  at five seconds still waits about 90 for them.
- **Kill the running ones.** This makes a failure quick, but the killed steps
  report nothing. A lane that broke a test and a lint would then find the lint
  one gate later. Making merges re-gate less often is the point of this whole
  section. The serial loop never killed a step either, so no step was written to
  expect it.

What it costs: a failing gate now takes as long as a passing one, about 91 s on
idle cores. Under the serial loop a failure cost anything up to 308 s,
depending on where in the list it came.

One thing moves with it. `check-plugins` no longer runs by itself before the
rest, so a gate with no plugins runs every step. It is still what the block
names first, with its install command in the tail, because it comes first in
`gate_steps` and `check-gate-runs-every-check` keeps it there.

### 372.10 Which steps are chained

The list is in `gate`: `cargo_steps`, eight names. Its order comes from
`gate_steps`, so `check-t-selftest`, which needs the binaries `test` builds,
still runs after `test`.

It is a list, held by a check, and not worked out on each run. The simple
test, "the recipe mentions cargo", puts `check-fuzz-overnight-selftest` in the
chain, because its body builds a stub `cargo`, and `deny`, whose `cargo deny`
builds nothing. It also misses `check-t-selftest`, which reaches cargo through
`just t`. A wrong answer at run time is silent. The first would add that
selftest's 90 s to the chain. So `check-gate-runs-every-check` got an arm that
compares the list both ways with a stricter test: a line of the recipe that is
not a comment, where `cargo` starts a command and takes a subcommand that
builds (`build check clippy nextest test run bench doc llvm-cov fuzz`), or a
line that calls the `t` recipe. On today's 56 steps that test finds exactly the
eight. Taking `check-t-selftest` off the list and putting `deny` on fails the
arm, and its message names both. A new cargo step that the test cannot see
(one that calls some other recipe that builds) still gets past it. That makes
the gate slower, never wrong.

### 372.11 What loud and quiet print

`quiet` prints one line per step as the step ends, `· <step> (<seconds> s)`,
with `FAILED, exit <n>` added on a failure. It prints at the end rather than
the start because every step now starts at once: a start line would list them
all in the first second, and a result cut short would say nothing about how far
the run got. The first failure's tail and the block come after, and the block
is still last.

`loud`, which is bare `just gate` and what CI runs, prints each step's whole
output as one piece when the step ends, under its `===` line. Steps finish out
of list order, so that line is what says which step the text below it belongs
to. Nothing prints while a step runs, and on CI that means minutes of silence
while `test` runs. GitHub Actions has no timeout for a job that prints nothing,
so this is only a matter of taste. Streaming the chain and holding the rest
back would interleave the two.

### 372.12 CI's runner

`ubuntu-latest` has 4 cores, so the gate runs four steps at once there: the
chain and three others. This was not measured on a runner; the Actions
allowance is spent (§172). The case for it on four cores: cargo already uses
every core for the chain, and the other 48 steps are shell selftests that
mostly wait. In §372.3's `ordered-1` they used 128 of the 921 CPU-seconds,
while the eight chain steps used 794 (summed from its `steps.csv`). Running the
rest beside the chain costs the chain at most their CPU time. Running them in
turn instead adds their whole wall time, and that includes
`check-fuzz-overnight-selftest`'s 90 s of sleep. A job count from the cores
covers both machines and needs no threshold. A 1-core machine gets 1 slot,
which means running in turn.

**Amended 2026-10-24: the four-core case, measured.** Still not on a runner,
but on this machine, pinned: `taskset -c 0-3 mise exec -- just gate quiet
fresh 4` (the `taskset` matters — `jobs=4` only caps the recipe's own
step-slots, not which physical cores cargo's own threads land on) took
`real 43m2.138s` on a warm tree. `uptime` just before it read `load average:
65.18, 42.78, 34.64`, and three other lanes (Q2711, Q2633 and Q2665) were
building and testing rcad in their own worktrees on this machine for the whole
run — confirmed by `ps`, not assumed. The same tree, same commit, run again
fresh with every core instead (`mise exec -- just gate quiet fresh 24`, no
`taskset`) took `real 15m43.195s`; `uptime` just before read `load average:
12.91, 32.05, 34.14`, with the same three lanes still running. So four cores
cost about 2.7x the wall of 24 here — consistent with the case above, that
cargo already saturates every core for the chain and the rest mostly wait —
but the two runs did not hold the machine's own load fixed (it fell from 65 to
13 in the gap, for a reason not traced here), so the 2.7x is this machine's
figure, not a bound for `ubuntu-latest`'s dedicated 4. What the runs do settle
is that four job-slots finish and pass rather than deadlock or time out:
`gate: PASS` both times.

On `wait -n -p` (§372's recipe, and a fixture at `justfile:680`): this
machine's bash is `5.2.21(1)-release` (`bash --version`), the only version
checked until now. `ubuntu-latest` was mid-migration on the day of this
measurement — GitHub's changelog has it rolling from Ubuntu 24.04 to 26.04
between 2026-10-19 and 2026-11-19, so either image could answer the label.
`actions/runner-images`'s own readme for each lists Bash `5.2.21(1)-release`
for 24.04 and `5.3.9(1)-release` for 26.04. Both are past 5.1, so `wait -n -p`
runs on either.

### 372.13 A busy machine

The gate limits how many steps run at once to the number of cores, and only
that. It does not look at the load average, for two reasons:

- The one-minute load describes the last minute, and a gate takes about a
  minute and a half. `ordered-2-after-touch` (§372.3) started at a load of 11.8,
  under 24, so a limit based on load would have started every step, and it
  ended at 56.5.
- Running fewer steps at once does not reduce the work. It spreads the same CPU
  time over a longer wall and holds the merger's slot longer. The lever for a
  busy machine is how many gates run at once, which is §372.4's three slots.

No serial gate was measured under load, so it is not known whether in turn
would beat at once on a busy machine. The case for "no" is the second reason
above, not a measurement. If three lanes gating still push the load past the
core count, §372.7 already says what moves, and it is the cap, not this.

**Amended by §397.6 on 2026-10-03: the order of the levers.** The gate still
does not throttle itself by load. The slot count is no longer the first lever
for a busy machine: first what each gate runs (§397.4), then how many gates
landing needs (§397.5), and then the slot count, by §397.7's bar.
