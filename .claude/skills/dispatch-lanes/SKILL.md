---
name: dispatch-lanes
description: Fill rcad's free lane slots from `docs/queue.md`, one `lanes` workflow per row — lane, verifier, merger — against a base you have watched go green, and read back what each run merged. Use when taking on the next work, or when a row's run returns and its slot is free.
argument-hint: "A row id to take first, or nothing to take what `just queue next` gives"
---

# Filling the lane slots from the queue

`docs/queue.md` holds every open item as a row, in the tiers of
`docs/decisions.md` §230.1 and §307. **Every free slot takes the highest ready row that
fits** (§230.3), and a slot is refilled when its own lane finishes, not when a
batch does (§230.4). The cap is four slots, with read-only rows, verifiers and
mergers each counting toward it (§447.3, which replaces §397.7's load bar with
a bar on what the load costs, and four slots pass it; §230.8 had six).

**`just queue` is what to dispatch next, and `just queue next` is what the free
slots take.** Every open item is a row of `docs/queue.md` — id, tier, state,
model, write set, and a line ending in the section behind it — in seven tiers:
1 defects of class A or B or with no class, 2 finding defects better, 2.2 how
the work is done, 2.5 slow runs, 3 corpus files, 4 new functionality, 5 defects
of class C or D (§311 put defects first, §403 the unlikely ones last). A row about the
queue and its verbs, the lanes workflow and its roles, hooks, skills,
`CLAUDE.md`, or a recipe that is neither a test nor a fuzz night is tier 2.2,
not tier 1 or 2 (§307.1); `just queue
tier <id> <tier>` moves one filed elsewhere. The dispatcher writes the file and
a lane never does. `check-queue` holds every row to its shape in the gate.
`docs/decisions.md` §230.

The rules below step 8 are the rest of what only a dispatcher needs: the
recipes the shared skills name, the plan, merging, CI and the Actions
allowance, the plugins, and lookup lanes.

`$ARGUMENTS` names a row to take first. Empty means take what `just queue next`
gives.

## 1. A worktree, and the queue

`EnterWorktree` first; nobody works in the hub. Then, in one call:

```
mise exec -- just status
mise exec -- just queue
```

**Before trusting any of that, check your own age against `.claude/agents`.**
A running `claude` process never re-reads `.claude/agents` (§266.6), so a
dispatcher started before an agent file last changed sends every lane,
verifier and merger it dispatches the rules that file had before the change,
without ever seeing the difference (§360.3). In one call:

```
ps -o lstart= -p $PPID
git log -1 --format=%cI origin/main -- .claude/agents
```

The first line is when this session's own process started — `$PPID` from a
Bash call is the `claude` process itself, not the shell running the command.
The second is when `.claude/agents` last changed on `origin/main`. If that
commit is later than the process start, hand off and tell Mattias to start a
fresh process before dispatching anything: this one is running `lane.md`,
`verifier.md` and `merger.md` as they were before that change, and nothing
short of a new process reads the new ones (§360).

`status` prints §0, the queue's ready rows, the board, `lanes`, the branches
carrying work `main` does not have, and how far the tip is past the last green.
`queue` adds what is running and what is waiting on which row.

The queue says what is next; §0 says only which unit the corpus rows serve.
Believe the file over any summary of it, including one in a handoff.

`status` is where to start rather than where to end up. Its first row is the
one a recipe cannot compute, so it is written down in `docs/decisions.md`
**§0** — `just decision 0` — and **that section is edited by hand when the work
moves**; `check-where-the-work-is` holds it to its shape but cannot tell you it
is stale, so `status` prints its age instead. `docs/decisions.md` §102.

Do not read the branch row off `just lanes`: `lanes` lists worktrees, and a
worktree is not a branch. Do not derive it from ahead-of-main either (§102.4).

## 2. Do not dispatch into a tree somebody holds

`status` already printed `lanes`. A tree reported `HELD` has an owner named in
its lock reason — message that owner, never send a second agent in. A lock date
does not prove the owner is gone.

**Take `just lanes` whole**: a truncated ask is re-asked more often than a
whole one. §157.3.

## 3. Pin the base, and watch it go green yourself

```
git rev-parse HEAD
mise exec -- just gate quiet
```

One plain Bash call for the gate, `timeout` 600000, no redirect and no pipe
through `tail` — a pipe hands you the pipe's exit code, so a failed gate reads
as 0. Never write a wait loop around it; a foreground call already waits. If the
tree is cold, pass `run_in_background: true` and let the harness wake you.

**Nothing checks that the pin was green but you.** GitHub's Actions allowance is
spent, so a red X there in a few seconds is billing and not a regression, and a
local green is the only green there is.

## 4. Pick the rows

```
mise exec -- just queue next        # or: just queue next N
```

`just queue help` prints every queue mode and verb with its usage.

It takes the first ready row of the highest tier whose write set meets no
running `lane:` row, no row taken above it, and no pinned row skipped above it
(§430.7), and names what each skipped row met. N defaults to the free slots,
counted from the trees agents hold on this machine — lanes, verifiers and
mergers alike, whoever sent them (§263.4).
Tiers 1 and 5 go in class order, not file order (§352.3, §403.3). While a
Sonnet row is ready and no running lane holds one, the fill keeps the last
free slot for it rather than giving every slot to Opus (§447.3).

**`next` already passes over a row waiting for its own date or for free
threads**, so it needs no second call to skip past one the way a `lane:`-held
write set does: a row whose `after:` names a date not yet come, or whose
`cores:` entry does not fit the machine's free threads (summed over the
running, held-elsewhere and already-taken rows, the night fuzz holding every
thread in its own window) is `waiting` or skipped like any other blocked row,
not offered for a second `next` to pass over by hand (§308).

**If `queue next` refuses, naming `just fuzz-triage-row`**, a fuzz night has
left groups that no open row names and no triage row is open yet (§418.2).
Run `just fuzz-triage-row` — it files the one triage row §418.3 describes, or
says why it filed nothing — then re-run `queue next`. It fills as usual, with
no refusal, when the latest night has no verdicts file; that line says so and
is not a reason to run `fuzz-triage-row`.

**If a row the fill took is in a family, trace the family first** (§450.3). A
family is three or more open rows, not parked, that name one `tests/*.rs` file
in their write sets. `just queue families` lists them, less the rows a trace
has named, a trace being a row, open or closed, starting "Trace the family on
<file>"; the plain `just queue` ends with how many. Do not send a row in a
family it lists alone. In the same `queue apply` as your takes, `add` the
trace with write set `-`, in the tier of the member the fill reached first. Put
every other member that is not pinned and not in a lane `after` it, keeping
what each already waits on (`after Q1 Q8,@1`). Then `take @1` in that slot.
450.3 has the trace's text. A family on `crates/rcad-topo/tests/merged.rs` is
no family, and is not traced (§373.7, below).

A line `take Q1,Q2,Q3 (a bundle of 3 small rows, one lane)` is **one
lane** doing all of those rows (§352.4), with each row's text indented under
it. A pinned row the fill takes alone gets a lane to itself. If it is small by
§352.4 (no trace, its place known, about ten minutes), run `just queue tag <id>
small` and then `next` again: the row then takes the small rows after it into
its lane (§430.8).

**Checking a row against the code is not yours.** Its `lanes` run sends the
row to prep first (`.claude/agents/prep.md`, §396.4). Prep reads the row and
the code at the base, and holds the rules this step held: the write set
against the work, the recipe that pins a count the row moves, the model per
row and per bundle, `sourcePaths`, the marks, and the facts the lane needs. A
row prep finds wrong comes back not merged before any lane starts, with a
`needs <file>: <why>` line for each file it is missing. When the write set is
the only thing wrong, fix it in place with `just queue writes <id> <write
set>` instead of re-filing (§230.3); any other finding still needs re-filing
with those files, drop and add, then `queue next` (§396.4).

**A row that adds a test file directly under `crates/rcad-topo/tests/`,
`crates/rcad-geom/tests/` or `crates/rcad-step/tests/` holds one item of
`crates/rcad-topo/tests/merged.rs`, `crates/rcad-geom/tests/merged.rs` or
`crates/rcad-step/tests/merged.rs`, never the whole file, and the new file
itself** (§373.7). The lane declares its own file there, in its place by
name, and the entry `crates/rcad-topo/tests/merged.rs#<module>` (or the
`rcad-geom` or `rcad-step` equivalent), the module being the file's name with
each `-` written `_`, lets rows adding different files run side by side. `merged.rs`
whole would make each of them wait on all the others. The write set needs the
new file's own path too — Q2353's row held `merged.rs` whole, its lane
narrowed to the item alone and dropped the file, and `unproven()` refused the
merge for a file outside the write set, so the row had to be re-filed as
Q2503. `just queue add` already asks for the item whenever the file is named;
asking for the file whenever the item is named is the other half, and still to
build. When prep stops such a row with `needs crates/rcad-topo/tests/merged.rs`,
re-file it with the item entry and the new file's path. At `ce88b698`, 45 open
rows named a new rcad-topo test file and
no `merged.rs` entry. They can be re-filed that way in one `queue apply`, drop
and add each, rather than one at a time as prep reaches them; so can a row that
holds `merged.rs` whole only to declare its one file. `just queue families`
counts `merged.rs` as a test file and so lists these rows as one family on it.
They share a binary's root, not a shape: trace no family on `merged.rs`.

**A row whose fix makes a corpus file import or stop importing names its two
corpus pins in its write set**, the same way a row that moves a `check-*`
count names the recipe that pins it: `KICAD_FILES_THAT_IMPORT` in
`crates/rcad-step/tests/how_often_a_real_file_imports.rs` and
`KICAD_FILES_THAT_ROUND_TRIP` in
`crates/rcad-step/tests/a_real_file_written_out_and_read_back_is_the_same_solid.rs`,
the KiCad corpus's only pair today. Naming them lets the row's own lane move
both counts together; left out, they are the merger's to re-pin instead, under
`merger.md` step 2's conditions (§391).

## 5. Dispatch: one `lanes` workflow per row

A row is one run of the `lanes` workflow: its lane, then the lane's verifier,
then a merger that lands it on `main` and hands back about twenty lines. You no
longer merge in your own context, which is where most of it went (§266).

A session reads its agent types once, when it starts. A session begun before its
tree held `.claude/agents/prep.md`, `lane.md`, `verifier.md`, `merger.md` and
`lander.md` cannot resolve them, and every stage of the run throws — except
two. A missing `lander` type does not throw the run, it sends the lane to the
Opus merger instead, on its own branch, `merge-<label>-after-land` (§384.3). A
missing `prep` type stops a short-form lane before it starts, and the row comes
back not merged with `prep threw` in its lines. Start a new session rather than
run it (§266.6). **A session begun before its tree held `prep.md` restarts
before it sends a short-form lane;** the full form, which sends no prep, works
meanwhile.

For every row the fill took:

1. One `mise exec -- just queue apply take <id1> <branch1> + take <id2>
   <branch2> ...` (§309), commit that line, and push it to `main` — a docs
   commit goes straight there. The merger reads the queue
   from `origin/main`, so a `lane:` it finds there flips to `done:` on one side
   only. A take left on your branch meets that flip at your next merge of
   `main` as a row changed on both sides, and costs a `just queue pick`. A
   bundle is one `take` step per row, every one onto the same branch, in the
   same `apply`. A take that never started a lane, or started one that died
   before it committed, leaves a hold with nothing running under it —
   `just queue give <id>` releases it and puts the row back to the state
   `take` replaced (todo, or its `after:` list). It refuses while the
   branch still has a tree or carries commits `main` does not have, so it
   never drops a hold a lane might still be running under (§263.3, §263.4).
2. `Workflow` with `scriptPath: ".claude/workflows/lanes.js"`, one call per
   row or bundle, all of a fill's in one message. **Never `name: "lanes"`**: the
   name runs the copy the registry read earlier in the process, and neither a
   merge into your tree nor a `/clear` refreshes it. Q118's run by name got
   `lanes.js` as of Q182, from before Q186, and refused a merge its verifier
   had proven by three reproduced mutations. `scriptPath` runs the file in your
   tree as it is when you call.

The args are `{ base, lanes: [ { label, queueId, notes } ] }`, the short form
(§396.4). A complete call looks like this:

```
{
  "base": "35cb5eaca58ccaa65ecd9d5712eabef3e350d633",
  "lanes": [
    { "label": "fix-the-corner-case", "queueId": "Q999" }
  ]
}
```

- `queueId` is the row's id, or `Q1,Q2,Q3` for a bundle as `queue next`
  printed it, in one run for the whole bundle.
- `notes` is text, and only what the row's text does not say: a finding from an
  earlier run, a file another running lane holds. Leave it out when the row
  says it all.
- The branch defaults to `lane-<label>` and must not exist yet.
  `prune-worktrees` deletes only the branches `main` holds (§458), so a lane
  branch that never landed is still there: check `git branch --list 'lane-*'`
  first; a reused label stops the lane at its second command (§237). Two lanes with
  one label in a run are refused at its start.

The run sends the row to prep, then builds the lane's brief from the row's
line, prep's marks and facts, your `notes`, and the two notes every brief
carries, on the steps in `rows` and on the test names in
`tests_that_fail_without_the_change`. The lane starts with the write set,
model, `sourcePaths` and marks prep answered, and a writing lane is asked for a
`for_the_verifier` field by default: how to lay back the `#[cfg(test)]` pins it
adds in `src/` when its verifier reverts the source. The workflow appends the
base, the `switch -c` and its refusal to the brief itself, and sends each role
with its type, so `.claude/agents/prep.md`, `lane.md`, `verifier.md` and
`merger.md` carry the standing rules.

**The full form is for a row sent again with a finding in its brief, a
`report` schema, or `verify: false`.** You then write what prep would have: the
brief, and every field by `prep.md`'s rules. The workflow gives a full-form
brief neither of the two notes nor the `for_the_verifier` field, so type both
notes into the brief and put the field in `report` yourself; `rowsNote`,
`testsNote` and `forTheVerifier` in `lanes.js` hold the words. A lane giving
`notes` beside a full-form field is refused at the run's start. A full-form
call:

```
{
  "base": "35cb5eaca58ccaa65ecd9d5712eabef3e350d633",
  "lanes": [
    {
      "label": "fix-the-corner-case",
      "queueId": "Q999",
      "brief": "Fix the off-by-one that drops the last edge of a closed loop.",
      "writeSet": ["crates/rcad-geom/src/walk.rs"],
      "model": "sonnet",
      "sourcePaths": ["crates/rcad-geom/src"],
      "pinsOnly": false,
      "sameBehaviour": false,
      "report": { "properties": { "finding": { "type": "string" } }, "required": ["finding"] },
      "verify": true
    }
  ]
}
```

`verify: false` skips the verifier, leaving the merger's own gate as the only
check.

`base` is a sha you have seen go green: step 3's, or the sha a merger pushed,
since it gated that tree.

**Never do a row in your own tree because the workflow cannot merge it.** A
row the workflow has no road for is a missing road in `lanes.js`: file it as
a 2.2 row and dispatch the row once it is built. Q854's dispatcher did the
row itself, which is how §399 came to be written (§399.1).

A row whose write set is `-`, or an empty `writeSet` in the full form, makes a
read-only lane, which is neither verified nor merged, and its answer comes back
whole. A test-only row gets no write access to `src/` and reports defects
instead.

Several rows off one base can share a run, and their merges then wait for one
another. But a slot refills when its run returns (§230.4), so a shared run
holds every slot in it until the slowest row is merged. `lanes.js` refuses more
than one entry in `args.lanes` unless the call also sets `args.shareOneRun` to
`true`: launch one `Workflow` call per row, and reach for that flag only when
sharing the run is the point.

**Never `SendMessage` an agent a run is running.** It does not reach the live
agent: it starts a second copy from the transcript, in the same tree and on the
same branch, and the two write over each other (§398). A brief found wrong
mid-run is corrected when the run returns, or the run is stopped with
`TaskStop` and the row sent again.

## 6. When a row's run returns

A merged row comes back as the sha the merger pushed, the gate's PASS line,
the rows it closed and added, and anything it left for you. A row that did not
merge comes back as why, the lane's summary and the verifier's findings, and
the trees to unlock. What the verifier checks is `verifier.md`'s to say, and how
the merger merges is `merger.md`'s; neither is repeated here.

A row whose lane wrote rows is merged by the lander, through `just land`, not
by the Opus merger — its result line reads `merged by just land, which the
lander ran`. A stop sends it to the Opus merger instead. After a stop, the
lander's own tree is unlocked but not marked finished, so it stays on disk
until the merger that follows marks it (§384.3).

A lane that reports `does_not_reproduce` ran every input its row names — crash
files, corpus files, a shape — on its base, saw none show the fault, and
committed nothing (§433.8). Its run sends it to the verifier even under
`verify: false`, never to the lander, and the verifier runs the same commands
on the base. On the verifier's confirm, a merger closes the row done at the
base sha with nothing merged, and its result line reads `closed done at the
base with nothing merged, its fault no longer showing (§433.8)`: that row is
merged, and the Merged bullet below is what you do. Anything but a confirm
comes back not merged, with `the verifier saw the fault still show: <command>`
or `the verifier said confirm, but ...` in its stop line, and the Not merged
bullet takes it: the fault is still there, so the row is taken again with
that command in its brief. A confirmed lane whose merger did not push reads
plain `not merged`, not `verified, not merged`, since `verified` there means
a verifier's `merge` or `fix_first`; the line after it, `a close at the base
that its verifier confirmed and its merger did not push (§433.8)`, says the
fault is gone and only the close is left: a `done <row> <base sha>` step in
the Not merged bullet's one `just queue apply`.

A writing row's branch can also sit at its own base for a reason other than
`does_not_reproduce`: the lane put in a probe, measured with it, and took it
out again before it stopped, so nothing is left to merge even though the row
is not reporting the fault gone. That comes back exactly as a read-only
lane's answer does — the report whole, no merger spent, the row still open —
and `lanes.js` logs `its branch is the base, so its report goes back whole,
not merged.` There is no verifier run and no `done` to apply: read the
report yourself, close the row at the base if what it found settles it or
re-take it if not, and file by hand, in one `just queue apply`, any rows the
report names.

- **Merged**: `git fetch origin` and `git rev-parse origin/main` — a reported
  sha can be invented — then merge `origin/main` into your branch, so the copy
  of the queue you fill from sees the row closed (§263.9). The slot is free:
  `mise exec -- just queue next 1`, and back to step 5.
- **Not merged**: decide it before anything else, and write the decision to
  the queue at once, in one `just queue apply` that also files the rows the
  lane left, read from its journal, each `from/<row>` (§432), or
  `from/fuzz:<target>` or `from/step:<file>` for a fault a crash file or a
  corpus file shows (§433.2). The three ways
  on: take the row again under a new label with the fix in its brief; park it,
  with the section its reason belongs under; or `just queue drop` it. A row
  filed again after a drop says `was/<old row>`. A drop that leaves rows
  waiting on it is refused, naming an `after` step for each: `after <id> -`
  frees one, and the steps go in the same apply (§451). `git worktree unlock` each
  tree the result named once the apply is pushed.

**A triage row's fault rows each name their fuzz target** (§433.3). The
triage row itself is `from/-`, and its lane starts each fault row it reports
with `from/fuzz:<target>`, the target whose crash file that row names;
`lane.md` tells it so. When you file a read-only lane's rows yourself, keep
that word. A fault row you file from a night's crash file or a corpus file
starts with `from/fuzz:<target>` or `from/step:<file>`, not `from/-` (§433.2);
`just queue add` checks that the target or the file is there.

A lane's tree is unlocked by its verifier when checking starts, the verifier's
by the merger, and the merger's by itself at the end, so a row holds one slot
at a time. A merged row's merger then marks all three finished, and the run
removes them with `prune-worktrees`. The result's `prune:` lines say which
went and why any stayed (§315). A row whose run stopped early leaves its trees
locked, and the result names them.

**Close every returned run out this way before anything else.** Once nothing
is left that `just status`, `just queue` and `just lanes` cannot already show
— no row still reading `lane:` with its trees unlocked and no agent behind
it, no finding that lives only in a run's `journal.jsonl` — the close-out is
one message: tell Mattias to `/clear` in the same `claude` process and run
`/dispatch-lanes`. A run reports to the `claude` process that launched it, not
to the conversation, so a cleared conversation still receives its
completions; a new process loses those notifications outright. A run's
journal, and its args once it completes, stay reachable from the old
conversation's own directory, not the new one:
`~/.claude/projects/<project>/<launching conversation id>/subagents/workflows/<run
id>/journal.jsonl` and `.../workflows/<run id>.json`, `<project>` the tree's
path with each `/` turned to `-`. `docs/decisions.md` §388.

## 7. What is still yours

Whatever the merger lists as for you. Two cases come up:

- **It spent its push attempts,** or its catch-ups over docs alone (§367).
  Other merges reached `main` first each time,
  and its branch `merge-<label>` holds the merged tree. Merge `origin/main` into
  it in a tree of your own, gate, and push, or send the row again.
- **`ff-main` refused.** It refuses on purpose: the tree holding `main` is
  ahead, diverged or dirty, and that tree names no owner. Find out whose work
  it is before moving it.

A merged run removes its own trees. Reclaiming the rest of the disk is yours:
a run that did not merge, a read-only lane (its measurements may be in its
gitignored `target/`, so copy them out first), and any merged tree that a
`prune:` line says was kept. Do it as you go:

```
mise exec -- just clean-targets
mise exec -- just prune-worktrees
```

`prune-worktrees` judges by ancestry, so it keeps a branch that landed by
cherry-pick and every spike that is never merged by design. `git cherry -v main
<branch>` is the content test when you want to know which is which.

## 8. Write it down

```
mise exec -- just claim-section '<what it is about>'
```

Before writing the section, not after — another session takes the number
otherwise.

**The §0 update goes in the same commit as the section that closes it.** "§168.4's
pie slice closed by §169" is only true because §169 exists, so landing it a commit
later guarantees `main` carries a wrong first row for the length of a gate — the
staleness `status` prints an age for, manufactured on purpose. The code commit
landing separately is fine — that comes from a lane — but the section and the
pointer are one act by one session. `docs/decisions.md` §172.5.

## Recipes the shared skills name

`delegating` and `handoff` are written against this repo and say so; these are
the commands they name. `just lanes` lists the worktrees and who holds each.
`just check-hub` is the hub rule. `just clean-targets` then `just
prune-worktrees` reclaims disk, in that order, and `just release-lanes
<pattern>` unlocks a batch of finished lanes in one call. **`just ff-main`
catches the tree holding `main` up to `origin/main`** — it finds that tree by
the branch, so never type its path, and it refuses rather than merges on all
four shapes that are not a fast-forward (§210). **Reach for those three rather
than a sweep of your own.**

`reference.md`, beside this file, holds what filling a slot does not use: the
plan, merging and the tree that holds `main`, CI and the Actions allowance,
the plugins, and the account of a run gaining the lander type on 2026-10-03.

## Hunting: lookup lanes

**If you cannot name the file, it is a hunt — hand it to a lookup lane before
the first grep.** Ask the one question you can answer *before* the first probe:
can you name the file, the section or the recipe the answer is in? Then read it
yourself — one call inline beats a lane. Are you about to search for *where*?
Then dispatch `.claude/agents/lookup.md` — read-only, one question, and it dies
with the answer.

All the value is in delegating a hunt at its **first** probe. Never send a lane
for `just lanes`, `just status` or the question file — that is your own state,
and you need the answer in your own hands. `just spend <window> finding` is the
report that measures all of this; `docs/decisions.md` §158 is the argument.
