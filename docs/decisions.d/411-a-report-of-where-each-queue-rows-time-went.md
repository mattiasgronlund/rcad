## 411. `just row-time`: where each queue row's time went — 2026-10-07

*Designed with Mattias in a grilling on 2026-10-07: twenty-two questions, every
recommendation taken, and the whole confirmed with "yes". Two of its rules were
wrong and are corrected here, each with its measurement (§411.4, §411.6).
The rows filed with it build it (§411.9). Q1311 built `row-time` itself, and
§411.10 says what it built and the rules it chose where the design was
silent.*

In short: `just row-time` reads the `lanes` run records and the transcripts of
their agents. For each queue row, it says how long the row took from `take` to
merged and pruned, and how that time divides between the agents' steps and the
waits between them. Inside each agent it divides the time again: the model
writing, each kind of command, and each tool. The buckets must add up, or the
script fails rather than print. The point is to find what to speed up next.

### 411.1 What it covers

Only agents inside `lanes` workflow runs. Rows worked with a plain `Agent` call
are counted and named in one line, not broken down: 89 of them, 2026-09-25 to
09-29, none since. The dispatcher's own time is not counted against any row.

Rows are picked by Q id, or by `--since`/`--until`, the last 7 days by default.
Trends come from picking dates; nothing is stored between runs. Under the rows
comes a summary of the ones picked: total, median and p90 of each step and each
bucket.

It reports time and nothing else: no tokens, no cost. Times are UTC. Tables go
to the terminal, and `--json` prints the same numbers. A table shows its top
15 entries, and the rest are added together under "other".

### 411.2 A row

- **Elapsed**: from the row's `take` in the claims file to whichever comes
  later, its `release` or the end of its run's last agent, the pruner. The
  claims file starts on 2026-09-26. A row taken before then starts at its
  run's first agent, and is marked that way.
- **Agent time**: the running times of its steps, added up.
- **Slot time**: the time it held trees locked, counted once per tree, by the
  rule in §411.6. §263.4 makes a locked tree what fills a slot, so this is the
  row's cost against the cap of six.
- **Attempts**: a failed lane can be taken again with no release in between
  (`take_step` in `tooling/queue/queue.py`). A row worked in several runs adds
  them all up, says how many there were, and lists each one in a detail mode.
  119 of 601 Q ids ran more than once.

Elapsed time divides with no gaps and no time counted twice:

    elapsed = waiting to start + the agents' running times + handoffs
              + earlier merge in the same run + between attempts + after the run

The script checks that this holds for every row, and for every attempt within
it, and fails if it does not.

### 411.3 A step

A step is one agent: the lane, the verifier, the lander, the merger or the
pruner. Its running time runs from `startedAt` to `startedAt + durationMs`, and
it is split into buckets that add up to exactly that. If they do not, the
script fails.

**The model.** Every content block (thinking, prose, a tool call) is its own
record, stamped when that block finished streaming. A block's time runs from
the block before it to its own stamp. The first block of a response runs from
the moment its request was sent, and nothing records that moment. It is taken
as the later of two times: the previous response's last block, and the last
tool result that response got. So the first block also holds the wait for the
first token, and the legend says so. Thinking is redacted, but its block still
has a time. Model time divides as:

- thinking;
- prose;
- writing a tool call's input, divided by tool: code (`Edit`, `Write`),
  commands (`Bash`), briefs (`Agent`, `Workflow`), and other.

**Tools.** While the model is writing, the time is the model's, even when a
tool has already started. A tool is charged from its response's last block to
its own result. When several tools run at once, each moment is split equally
among the tools still waiting for a result. Some tools start the moment their
block is written; others wait until the response ends. So a tool whose result
comes before its response's last block is charged nothing. Each command also
gets an **own time** column: from its tool call record to its result. Those
columns overlap and are not added up.

**What a `Bash` call is filed under:**

- A call that runs `just` is filed under its recipe, plus its first argument
  when that argument is a plain lowercase word: `queue take`, `gate quiet`,
  `land wait`. `just t how_often_…` is filed as `t`. A call that runs several
  recipes is filed under all of them joined with `+`, each named once.
- A background job's start and its `wait` calls are one item: `verify-lane`,
  `land`. The land lock wait is taken out of `land` and goes to the wait in
  §411.4.
- Any other call is filed under its first program, after stripping
  `mise exec --`, `timeout N`, `cd X &&` and variable assignments. `git` and
  `cargo` are filed with their subcommand. Programs it pipes into are ignored.

Of 21 024 `Bash` calls over the 7 days to 2026-10-07, 49% hold a pipe, 26% a
`;` and 13% a `&&`, and 773 run `just` more than once. So a call has to be
filed under one key, and the rules above are how.

**Other tools** get one bucket per tool name.

**Gate steps are not broken out.** `target/last-gate.log` already has them,
and Mattias chose this in the grilling.

### 411.4 The waits

Each wait is a bucket of its own:

- **waiting to start**: from `take` to the run's first agent;
- **handoff**: from one agent's end to the next one's start, worktree
  creation included. Median 2 s;
- **earlier merge in the same run**: a run's merges go one at a time
  (`lastMerge` in `.claude/workflows/lanes.js`). This is from the verifier's
  end to the moment the first merge-stage agent is queued. p90 565 s;
- **land lock**: the part of the lander's `land` time before
  `land: holding the land lock`. When another run's lander landed this lane
  under its own gate, the whole wait goes here, and the row names that run;
- **between attempts**: from one attempt's last agent to the next `take`, or
  to the next run's first agent when the row was not taken again;
- **after the run**: from the last agent's end to the `release`, when the
  release comes later. That happens to a row whose run did not merge and which
  was dropped or parked afterwards.

**The grilled design had a "tail", from `release` to the end of prune. It is
dropped, because it counts the lander's time twice.** The `release` is written
by the `done` in the lander's `queue apply`, which `land_one` runs before the
gate and the push. Over the 185 landers since 2026-10-04, 171 released their
row during their own run, a median 20 s in. A median 320 s of the run was still
to come (p90 608 s), and that time is already the lander's step time. A row's
time after its release is the rest of the lander, the handoff and the pruner,
and each of those is counted where it belongs.

**The land lock without a timestamp.** Today `land.py`'s `say()` prints no
time. The lander sees the line only in the result of the `land` call that
printed it, which can take up to 540 s (`WAIT` in `tooling/land/land.py`). So
for a call without timestamps, the wait is taken as lasting until that call
started. That is a lower bound, and those rows are counted and marked. Once row
A of §411.9 has landed, the line carries its own time and the bound is not
needed.

The measurements in §411.4 and §411.6 were taken with two scripts,
`unlock.py` and `release.py`. They read the run records and the claims file
named in §411.8, and were left in that session's scratchpad, which a reboot
clears.

### 411.5 Shared work

Every step says how many rows it served, and has two columns: its full time,
and its share (its time divided by that number).

- **Bundles**: one lane for 2 to 5 rows, `queueId: "Q1,Q2"`. 44 lanes so far.
- **A shared gate**: a lander that landed other runs' lanes under its own gate
  (§397.5) shares that gate with them. The link is exact. `finish` in
  `land.py` writes each landed lane's result with the holder's `merged_sha`
  and `merge_branch`.

### 411.6 When a tree stops holding a slot

**What Mattias was told in the grilling, that a lane waiting for the land lock
holds three slots (the lane's tree, the verifier's and the lander's), was
wrong. It holds two.** The lane's tree is unlocked as soon as its verifier
starts its first `just verify-lane` call. Unlocking it is that recipe's first
step (`verify` in `tooling/verify-lane/verify_lane.py`, §385), and before §385
it was the first step a verifier did by hand. Over the 232 verifiers that ran
from 2026-10-04 to 2026-10-07, 230 made that call, a median 19 s into their
run (p90 31 s, longest 130 s). Their runs took a median 569 s. The other two
have no transcript at all. `just lanes` at about 13:05Z on 2026-10-07 showed
the same thing live: two runs had their lane trees (`-1`) free while their
verifier trees (`-2`) were held.

So a tree's slot time runs from its agent's start until:

- **the lane's tree**: the start of the verifier's first `just verify-lane`
  call, or of its `git worktree unlock` of that tree (runs before §385). With
  no verifier, the merger's unlock of it;
- **the verifier's tree**: `land: holding the land lock`, since `hold_batch`
  unlocks it straight after the switch that follows. When another run's
  lander landed the lane, that lander unlocked it before letting the lock go,
  so the same moment is a bound and is used. When the merger lands the lane
  instead, the time is the merger's `git worktree unlock` of it;
- **the lander's tree**: the lander's end. `land.py` unlocks its own tree
  after the push, and only `ff-main` and `mark-finished` come after that;
- **the merger's tree**: the merger's own `git worktree unlock` call, the last
  of its unlocks (`merger.md`), or its end if it never made that call.

The pruner runs in the dispatcher's tree and holds nothing. A row taken with
no tree yet holds one slot from `take` until its lane starts (§263.4).

A tree the transcripts never show being unlocked is counted until its run's
last agent ends, and that row is marked "still locked when the run ended". Two
cases do this: a lane handed back, whose trees the dispatcher unlocks later,
and a killed run. Neither later unlock is in any transcript.

So while a row waits for the land lock, it holds the verifier's tree and the
lander's. Between the verifier's end and the lander's start, it holds the
verifier's tree alone.

### 411.7 Gaps are counted, not dropped

- Lanes with no `queueId` (129 so far, all read-only), and runs with no lanes
  it can read, go under "unattributed".
- An agent with no transcript (a killed run) shows its `durationMs` as one
  bucket, "no transcript".
- 7 run records are 0 bytes long. They are counted by name.

### 411.8 What it reads

These facts were measured on 2026-10-07. The rows that build this must measure
again whatever they rely on, not quote it from here.

**Run records** are at
`~/.claude/projects/<project>/<session>/workflows/<runId>.json`: 614 files
across 39 project folders, 605 of them `lanes` runs, from 2026-09-20.
`workflowProgress` entries of type `workflow_agent` give each agent's `label`,
`agentId`, `agentType`, `queuedAt`, `startedAt`, `durationMs` and `state`. 229
early entries have no `agentType`, so the label's prefix decides: no prefix is
the lane, then `verify:`, `land:`, `merge:`, `prune:`. To find the row, strip
the prefix, match `args.lanes[].label`, and split its `queueId` on `,`.

**Agent transcripts** are at
`<session>/subagents/workflows/<runId>/agent-<agentId>.jsonl`. 210 agents are
split into 2 to 6 pieces under different session folders of the same project,
so the loader looks in every session folder and joins the pieces in time order.
Only the earliest piece has a `meta.json` beside it. A transcript's first record
is at `startedAt` and its last at `startedAt + durationMs`, to within 0.00 s.
`journal.jsonl` has no timestamps.

**Responses and tools.** One response is one `(requestId, message.id)`. No
record marks a request's start. `toolUseResult` carries no timing, so a
command's time is from its tool call record to its result record, and nothing
finer. Over the 7 days, a tool call that was not its response's last got its
result before the response ended for `Read` 729 of 735 times, `Skill` 84/91,
`Bash` 156/1239, `Write` 22/191, and `Edit` 25/3726.

**The claims file** is `rcad-queue-claims` in the git common directory: one
tab-separated event per line, `add`, `take` or `release`, with UTC times. It
starts on 2026-09-26.

**Why not reuse `just spend`'s loader.** `load()` in
`tooling/session-spend/spend.py` keeps only the first record of each response,
none of the tool result times, and no run record. `row-time` gets a loader of
its own.

### 411.9 What is built, and by which rows

- **Q1310** (row A): `say()` in `tooling/land/land.py` starts every line the detached
  run writes to its log with its UTC time. Nothing reads the time of
  `verify_lane.py`'s lines, since the lane tree's unlock is the start of the
  call (§411.6) and gate steps are not broken out. So the grilled design's
  timestamps there have no consumer, and §13 defers them.
- **Q1311** (row B): `tooling/row-time/row_time.py`, `just row-time`, and
  `just check-row-time-selftest` in `gate`. It reads `land` lines with and
  without timestamps, so it does not wait for row A. The selftest builds
  made-up fixtures, not copies of real transcripts. Between them they cover:
  a transcript split across two session folders, a bundle, a shared gate, a
  retried row, two tools running at once, a tool that starts before its
  response ends, a land wait with timestamps and one without, and a tree
  still locked when the run ended. A fixture whose buckets do not add up has
  to make it fail.
- **Q1312** (row C): `just spend` undercounts output tokens. `load()` in `spend.py`
  keeps the first record of each response, and `output_tokens` grows from one
  block to the next. Take the largest count in the group instead. That is
  still a lower bound, since 74% of responses never write a record with a
  `stop_reason`. Not part of `row-time`. It was found while measuring for
  this design.

### 411.10 What Q1311 built, the rules it chose, and the first week

*Built and measured on 2026-10-07 at base `b7435392`. Q1310 had not landed,
so no `land` line carried a time yet.*

**What exists.** `tooling/row-time/row_time.py`, run as `just row-time`, with
Q ids or `--since`/`--until` dates (the last seven days when given neither),
`--json` and `--detail`. `check-row-time-selftest` is in `gate_steps` and
`gate_reusable_steps` beside `check-spend-selftest`. It builds a made-up week
and holds every bucket of it, worked out by hand: every case of §411.9's list,
plus a row whose verifier starts before its lane ends, which has to stop the
script with nothing printed. Eleven mutations of `row_time.py`, each
breaking one of the rules below, were all caught by it.

**What it reads, measured again.** Over the `lanes` runs started before
2026-10-07, and the transcripts of the agents started from 2026-09-30:

- 598 runs: completed 573, killed 20, failed 5. 7 run record files are 0
  bytes long. 1 run has no `args`, and none has `args` as a string, so the
  loader takes no string form.
- 2010 agents. 18 have no `startedAt` and no `agentId`: they failed before
  they started (a `WorktreeCreate` hook, or an agent type the session could
  not resolve). They are listed as "agents that never started" and hold no
  time. 56 more have no `durationMs`, all in killed runs; such an agent ends
  when its run ends, since its last record is 11 to 59 ms past the run's end
  in all 24 that have a transcript.
- Prefixes: none 736, `verify:` 549, `prune:` 282, `merge:` 264, `land:` 179.
  `agentType` never disagrees with the prefix; 229 have none. A retried agent
  is labelled `<label> (retry N)`, and that suffix is dropped too.
- Transcript pieces per agent: 0: 18, 1: 1777, 2: 206, 3: 8, 6: 1. **The
  brief's correction to §411.8 overcounted**: 215 agents are in two pieces
  or more, but only 6 have pieces in two project folders. So the loader
  indexes every project folder, as `transcript_files` in `spend.py` does, and
  joins the pieces in time order; no two overlap.
- Of 53 069 assistant records, 3 hold two blocks and 2 hold three. Each block
  is its own event at the record's time.
- A write cut off mid-record leaves the rest of the line holding the next
  record: 22 lines in 12 transcripts over every run on record, none of them
  in the week. Such a line is read again from each record start in it, and
  the step is marked with the records lost.
- 2321 hook records (`hook_success`, `hook_system_message`,
  `hook_additional_context`) against 31 894 tool calls. A hook leaves a
  record only when it says something, and the record has one time, its end.
  **So hook time cannot be split out, and stays in the time of the tool it
  ran around.**
- The claims file starts at 2026-09-26T09:29:46Z, with 709 takes before
  2026-10-07, 30 of them on a branch no `lanes` run has: the rows worked
  outside a workflow.

The counts came from two scratch scripts, `facts.py` and `waits.py`, left in
the lane's scratchpad like §411.4's.

**The two agents far off their running time.** The first record of
`q803-… (retry 1)` in `wf_da94ef96-f56` is 1185 s after its `startedAt`. The
record keeps the first try's start but names only the retry, whose
transcript starts when the retry did; the first try's transcript is there
under an agent id no entry names. That time is the bucket "before the first
record". The last record of q437's lane in `wf_2154d68c-293` is 395 s before
its end: a `just pre-gate gate quiet` call had no result when it ended. From
the last record to the end, the tools still waiting share the time, as
anywhere else, and only when none waits is it "after the last record".

**Two copies of one agent.** Over the week, 16 responses began before the
previous response's last block, and 37 tool results came after the next
response's first block. Every one is in that same q437 lane: the dispatcher
sent it a message while it ran, which started a second copy writing into the
same transcript. So the sweep goes by time, not by response:

- while any response is being written, the moment is the model's, charged to
  the next block to finish;
- otherwise, the tools still waiting for a result share it equally, in whole
  milliseconds, the remainder to the earliest calls;
- otherwise, if a user message comes before the next block, it is "waiting
  for a message";
- otherwise it is the next response's first block, which so holds the wait
  for the first token.

Each moment is counted once whatever the order, and the step is marked "two
copies ran at once".

**Waiting for a message** came in three kinds over the week, each one gap
before a user message with no tool waiting: 14 `[Request interrupted by
user]`, 17 s in all, which is a run being killed; 2 `Your response above was
cut off mid-stream`, 104 s, where the stream stalled about 50 s before the
harness asked again; and 1 message from the dispatcher, 0.8 s. They share the
one bucket.

**A take belongs to the first attempt after it.** Q668 was taken once, its
first run killed 13 s in, and a second run started a minute later on the same
branch. Matching each attempt to the last take before it gave both the same
take, and the second attempt's wait ran backwards: the script stopped on it,
which is how it was found. The release is the first one at or after the last
attempt's first agent.

**The land lock.** It ends at the hold line's own time, or, with no time on
the line, at the start of the call that printed it, a lower bound: 136 rows
in the week carry that mark. A lander that printed only `land: waiting for
the land lock`, killed while it waited, spent all its `land` time on the lock
(2 in the week). So did a lander whose lane another run's lander landed (4).
Landers from before §397.5, the first at 2026-10-05T06:02Z with a hold line,
had no lock at all. They print no hold line, so `land.py`'s echo of each
`+ git worktree unlock <tree>` it ran is what frees the verifier's tree for
§411.6, at the start of the call that printed it.

**What the sums catch.** A step's buckets add up by how the sweep works, so
that check guards the code: a mutation that adds 1 ms to one bucket fails 18
cases. A row's parts come from separate definitions, the take, the agents'
starts and ends, and when they are queued, and those can disagree on real
input, as Q668 did.

**The week.** `mise exec -- just row-time --since 2026-09-30 --until
2026-10-07`: 320 rows.

    over the rows picked                         total share    median       p90
    row: elapsed                                     228h29m    38m41s     1h12m
    row: slot                                        199h22m    38m34s     1h14m
    wait: between attempts                            27h53m        0s       57s
    wait: waiting to start                            21h51m       38s     2m01s
    wait: after the run                                3h31m        0s       33s
    wait: land lock (in the lander's time)             1h31m        0s        0s
    wait: handoff                                     25m44s        5s        9s
    wait: earlier merge in the same run                9m18s        2s        3s
    step: lane                                       103h19m    21m04s    40m42s
    step: verifier                                    44h59m     7m43s    20m17s
    step: lander                                      18h25m     3m37s     9m28s
    step: merger                                       7h55m        0s     5m31s
    step: pruner                                       1h19m       21s       29s
    bucket: model: thinking                           52h41m    10m46s    20m48s
    bucket: just t                                    21h21m     3m22s     9m47s
    bucket: just verify-lane                          18h54m     2m16s     9m29s
    bucket: just land                                 15h38m     3m02s     8m48s
    bucket: just pre-gate gate-lane                   15h13m     2m25s     8m10s
    bucket: model: writing commands                   11h20m     2m24s     4m50s
    bucket: model: writing code                       10h25m     2m00s     3m54s

**What to speed up first: the gate, run three times for each merged row.**
The lane's own (`just pre-gate gate-lane` 15h13m, `just gate quiet` 5h35m,
`just pre-gate gate` 1h31m), the verifier's `just verify-lane` 18h54m, and
the lander's `just land` 15h38m with its land lock 1h31m come to about 58 h
of the 176 h the agents ran. That is more than the model's thinking, 52h41m,
and it is the largest share a change to the tooling can move. Gate steps are
not broken out here (§411.3), so the next step is to measure which of them
the verifier's and the lander's runs spend that time in, and how often they
are served from an earlier gate. `just t`, the lanes' own test runs, is next
at 21h21m.

The waits are not where the time is for a typical row. Waiting to start and
between attempts add up to 49h44m, but their medians are 38 s and 0 s, and
ten rows hold 90% of the two: rows taken hours before their run, like Q753's
17h10m, and bundles retried hours later, like Q1078's 7h32m. Handoffs and
earlier merges are 35 minutes together.

What this leaves: the gate measurement above is a 2.2 row, filed with this
section. Q1310's timestamps make the land lock exact rather than a bound,
with no change here.
