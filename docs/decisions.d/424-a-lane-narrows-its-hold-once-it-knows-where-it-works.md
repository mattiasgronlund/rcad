## 424. A lane narrows its hold once it knows where it works, and two lanes may share arrange.rs or assemble.rs — 2026-10-09

*Mattias asked on 2026-10-09 what could be done about so many queue rows
needing `crates/rcad-topo/src/fuse/`, which more or less serialises the
dispatchers' work, and agreed to the two changes recommended. Measured in the
dispatcher session of `dispatch-2026-10-03`.*

In short: while one lane held the whole `fuse/` directory, it held back 79
ready rows, every tier 1 row among them. The work does not need that much room:
106 of 125 fuse merges changed one fuse file, and a lane makes its first fuse
edit a fifth of the way into its run. But narrowing the running lane alone
frees little, because the waiting rows are just as wide, and most of them name
`arrange.rs` or `assemble.rs` whole. So, first, a lane narrows its hold in the
claims file once it knows where it works, and again to its branch's diff
before it reports. Then two lanes may hold `arrange.rs` or `assemble.rs` at
once: of 34 pairs of fuse merges that shared a file, 24 would have merged clean
side by side.

  424.1   What was measured
  424.2   Why narrowing the running lane is not enough
  424.3   First: a lane narrows its hold (amends §230.3)
  424.4   Then: two lanes may share arrange.rs or assemble.rs (amends §230.3 and §230.6)
  424.5   What is not done, and why
  424.6   The rows

### 424.1 What was measured

The queue numbers are read from `docs/queue.md` at `16e0f176`, while Q1506 was
running with the whole directory in its write set. The merge and hold numbers
are over `origin/main` from 2026-09-29 16:00 to 2026-10-09 16:00 UTC. The lane
numbers are over transcripts whose first record falls between 2026-10-02 16:00
and 2026-10-09 16:00 UTC.

**What one directory hold blocks.** `just queue blocked` listed 79 ready rows
held back by Q1506. 70 of them are every ready row that touches `fuse/`, and
all 20 ready tier 1 rows are among them. The other 9 met its `error.rs` or its
test file. With four slots, verifiers counted (§397.7), nothing else in
`fuse/` could run beside it.

**What the ready rows name.** Of the 70:

- 10 name the whole directory, 5 of them in tier 1.
- 50 name a fuse file whole: `arrange.rs` (13 698 lines) 21 times,
  `assemble.rs` (8 962 lines) 11 times, `select.rs` 11 times.
- 10 name only items, as `path#item` (§230.6).
- Of the 15 tier 1 rows that do not name the directory, 14 name `arrange.rs`
  or `assemble.rs`. The 15th is Q1461, in `cuts.rs`.

**What the work needed.**

- **Merges.** 125 first-parent merges changed a file under `fuse/`
  (`git diff --name-only M^1 M`). 106 changed one fuse file, 15 two, 2 three
  and 2 four.
- **Lanes.** 188 lane transcripts edited a fuse file: each
  `subagents/**/agent-*.jsonl` whose `.meta.json` says `agentType` lane, with
  an `Edit` or `Write` on a path under `fuse/`. 148 edited one fuse file. The
  first fuse edit came a median 20% of the way through the transcript (p25
  11%, p75 37%), and the median transcript spans 20 minutes.
- **Holds.** 64 of the 125 merges could be paired by row id with the
  `queue: take` commit before them. Take to merge was a median 0.7 h (p25 0.6,
  p75 1.0). So the lane's own run is about half the hold, and the verifier and
  the land are the rest.

**Fuse lanes side by side.** Over the 82 hours in which at least one lane was
in flight (take to merge, 193 rows), one lane whose merge changed `fuse/` was
in flight 57% of the time, two 8%, three 1%, and none 34%.

**Would two lanes in one file collide?** For each pair of consecutive fuse
merges A then B, B's lane branch was merged onto A's base with
`git merge-tree --merge-base=B^1 B^2 A^1`: B's change on one side, A's undone
on the other. A conflict under `fuse/` means the two changed lines close
together.

- 34 of the 124 pairs shared a fuse file. 10 of them conflicted, and 24 merged
  clean.
- By git's own hunk labels (`diff=rust`), 11 pairs changed a function in
  common, and 9 of those conflicted. Of the 113 that changed none in common,
  1 conflicted.
- The method also undoes whatever landed between A and B, so it overstates
  conflicts a little.
- It cannot see two changes that merge clean and break each other's
  behaviour. That was not measured.

### 424.2 Why narrowing the running lane is not enough

Q1506's lane edited `arrange.rs` and `assemble.rs`. Had its hold named those
two files in place of the directory, it would still have held back 62 ready
rows, 19 of them tier 1. Only Q1461 would have been freed in tier 1. The
waiting rows are as wide as the running one, and they are wide in the two
largest files.

So two changes are needed, in this order. The first makes a directory hold
short. The second lets two lanes into one of the two files most rows name.
Without the first, the second gains little, since one directory hold still
keeps everything out.

### 424.3 First: a lane narrows its hold (amends §230.3)

**A row whose cause is not known yet may still name the directory.** That is
honest. What changes is how long it holds it.

- **The verb.** `just queue narrow <id> <write set>` appends
  `narrow <id> <time> <worktree> <write set>` to the claims file under its
  lock. It writes no `docs/queue.md`, so a lane may run it from its own tree,
  and §230.2's rule that a lane never writes the queue file stands.
- **What it refuses.** An id nobody holds. An entry outside the write set the
  row was taken with, where a directory holds the paths under it and a file
  holds its items. A widening back, to an entry inside the taken set but
  outside the current one, when that entry meets another running hold.
- **Who reads it.** The last `narrow` is the hold's write set for `next`,
  `blocked`, `running`, `elsewhere` and the queue page: for this copy's own
  `lane:` rows as well as rows held elsewhere. `release` clears it with the
  hold.
- **When a lane narrows.** At its first edit under `src/`, to the files or
  items it now knows it will change. Again before it reports, to the files its
  branch changed. A bundle lane narrows each of its rows.
- **Why the verifier needs no change.** It checks the branch against the write
  set the lane was sent with (`files_outside_write_set` in `lanes.js`). A
  widening back stays inside that set.

**What it buys.** On the medians in 424.1, a directory row holds the directory
for about the first fifth of its lane's run, some 4 of 20 minutes, instead of
the 0.7 h from take to merge. After that most holds sit on one file: 148 of
188 lanes edited one.

**Why not §230.5's two rows instead.** §230.5 sends a defect of unknown cause
to a read-only lane first, with write set `-`, and files the fix as a second
row. That holds nothing while tracing, but the fix lane reads again what the
trace lane read. And Q1476 then Q1506 shows the second row can come out as
wide as the first: the trace placed the panic at `assemble.rs:2725`, and the
fix row still named the directory. Narrowing keeps one lane and its reading.
§230.5 still stands for a defect nobody can place at all.

### 424.4 Then: two lanes may share arrange.rs or assemble.rs (amends §230.3 and §230.6)

Once 424.3 is built, the limit is one lane per file, and 14 of the 15 tier 1
rows outside the directory name one of these two files.

- **The rule.** A justfile list, `queue_shared_files`, names files two lanes
  may hold at once: `crates/rcad-topo/src/fuse/arrange.rs` and
  `crates/rcad-topo/src/fuse/assemble.rs` to start. A whole-file entry on such
  a file, or the same item in it, meets only when two running lanes, rows held
  elsewhere or rows taken above already hold it. A directory that holds the
  file still meets everything under it. Every other path keeps §230.3's rule.
- **What it costs.** On 424.1's pairs, 10 of 34 same-file pairs would have
  needed a conflict fixed at merge, and 9 of those 10 changed a function in
  common. §230.6 already judged a conflict cheaper than a row left waiting. A
  clash in behaviour shows up as a failed land gate, which runs the whole suite
  (§397.4), not as a wrong `main`.
- **Back to one lane per file** if either holds over the first 20 same-file
  pairs that run side by side:
  - more than half of them need a merger to fix a conflict;
  - more than 3 lands fail because the two changes break each other.
- **Both bars are judgement**, set where the re-runs would start to cost more
  than the waiting they save. Half is well above the measured 10 in 34.

### 424.5 What is not done, and why

- **Splitting `arrange.rs` and `assemble.rs`.** §230.6 kept them whole at
  7 820 and 4 380 lines; they are now 13 698 and 8 962. A split would narrow
  file-level holds without anyone guessing items. But it conflicts with every
  fuse branch in flight. Every open row naming the two files would have to be
  dropped and added again, because rows have no edit verb. And it only pays if
  the new files' borders fall where fixes land, which is not measured. If
  424.4's bars send it back, the split is next, starting from where in the two
  files the fixes of 424.1 landed. §441 measured that for `arrange.rs` and
  wrote down the ten files.
- **Narrowing rows to items before the work.** 59 of the 125 merges changed
  six or more items, by git's hunk labels. An item guessed before the trace
  would often be too narrow, and the lane would report
  `out_of_write_set_needs` and be sent again.
- **Narrowing Q1506 by hand now.** It would free 17 rows, 1 of them tier 1
  (424.2), and the verb does not exist yet.

### 424.6 The rows

Both are tier 2.2 (§307.1): they change the queue and the lanes' roles.

- **Q1530** builds 424.3. It is ranked first in tier 2.2, above Q281, which
  waits on the same `tooling/queue/` and `justfile#queue`. It is the row whose
  absence holds tier 1 back.
- **Q1531** builds 424.4, after Q1530.
- **Q1532** counts 424.4's two bars, after Q1531, once 20 same-file pairs have
  run side by side. Nothing in the queue can wait on a count, so its text says
  "not before", and a dispatcher that reaches it early passes it over.

Q1530 and Q1531 each record what they built here, as a new subsection. The
change to `lane.md` reaches only dispatchers started after it lands: `/clear`
does not reload agent types.
