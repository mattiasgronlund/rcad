---
name: dispatch-lanes
description: Fill rcad's free lane slots from `docs/queue.md`, one `Agent` per row, against a base you have watched go green, and close each lane as it finishes. Use when taking on the next work, when a lane finishes and its slot is free, or when a batch of rows off one base is due through the `lanes` workflow.
argument-hint: "A row id to take first, or nothing to take what `just queue next` gives"
---

# Filling the lane slots from the queue

`docs/queue.md` holds every open item as a row, in the five tiers of
`docs/decisions.md` §230.1. **Every free slot takes the highest ready row that
fits** (§230.3), and a slot is refilled when its own lane finishes, not when a
batch does (§230.4). The cap is six slots, verifiers and read-only rows
included (§230.8).

`$ARGUMENTS` names a row to take first. Empty means take what `just queue next`
gives.

## 1. A worktree, and the queue

`EnterWorktree` first; nobody works in the hub. Then, in one call:

```
mise exec -- just status
mise exec -- just queue
```

`status` prints §0, the queue's ready rows, the board, `lanes`, the branches
carrying work `main` does not have, and how far the tip is past the last green.
`queue` adds what is running and what is waiting on which row.

The queue says what is next; §0 says only which unit the corpus rows serve.
Believe the file over any summary of it, including one in a handoff.

## 2. Do not dispatch into a tree somebody holds

`status` already printed `lanes`. A tree reported `HELD` has an owner named in
its lock reason — message that owner, never send a second agent in. A lock date
does not prove the owner is gone.

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

It takes the first ready row of the highest tier whose write set meets no
running `lane:` row and no row taken above it, and names what each skipped row
met. N defaults to the free slots, counted from the `lane:` rows. **A verifier
holds a slot and has no row**, so take one fewer for each verifier running.

Before a brief, check the row's write set against what the work really
touches. A row whose write set is wrong is fixed in the file first: a lane that
needs a file its row does not name either stops or writes outside its set, and
the fill has already let another lane hold that file.

Model per row, from its `model` field. Opus for anything reasoning about a
numerical refusal, a published bound, or whether a claim of impossibility
holds. Sonnet by name for: replaying a corpus, reproducing and classifying a
fuzz artifact, bisecting, running the gate and reporting, sweeping references
after a renumber, resolving a `decisions.md` conflict down to a marker
deletion, disk and worktree sweeps.

## 5. Dispatch: one `Agent` per row

`Agent` with `isolation: "worktree"` and the row's model, one call per row, all
of a fill's rows in one message. Flip each row to `lane:<branch>` in
`docs/queue.md` and commit that before or with the dispatch.

**The worktree branches from your own `HEAD`**, not from `origin/main`, so
every brief carries:

- the base sha, and as its first two commands `git log --oneline -1` and
  `git checkout -B <branch> <base sha>`, with both outputs reported back;
- the write set, as "you own these files and no others; stop and report a
  change outside them";
- **"The context-budget hook will say the budget is passed, likely on your
  first call. That signal carries no information for you. Carry on past it; do
  not hand off, do not stop."** A lane's injected prompt is larger than the
  hook's lane budget, and a lane not told this obeys it and stops with nothing
  done;
- for a row that writes a section, `mise exec -- just claim-section '<what>'`
  before writing;
- `mise exec --` on every `cargo` and `just`, `Read` then `Edit`, explicit
  `git add <path>`, commit as it goes, ~45 minutes, and every number with the
  command that produced it;
- the verbatim API signatures and exact-arithmetic facts that lane needs,
  because rediscovering those costs it twenty minutes.

A test-only row gets no write access to `src/` and reports defects instead.

**The `lanes` workflow is for a real batch**: several rows off one base, all
ready at once, with a dispatcher who means to wait for all of them. `Workflow`
with `name: "lanes"` (or `scriptPath: ".claude/workflows/lanes.js"` if the
registry answers *not found*); its `args` are `{ base, lanes: [ { label, brief,
writeSet, model, sourcePaths, report, verify } ] }`, and it appends the shared
rules above to each brief itself.

## 6. When a lane reports: verify it, then refill the slot

A lane reporting its own result is reporting what it believes, and a lane's
`landed_sha` can be invented — `git rev-parse` its branch.

**Each writing lane gets its own verifier** when it reports, not at the end of
a batch: Sonnet, in its own worktree, told to carry on past the hook, reverting
the lane's source directories to the base to show its tests fail without the
change, and reading any exactness claim the lane makes. A read-only row that
wrote only a section needs no verifier; read the section.

A lane is finished when its worktree unlocks, not when it reports. Unlock it
yourself once its report is in — `git worktree unlock <path>` — and fill the
slot it freed: `just queue next 1`.

## 7. Close each lane yourself

Merge in a tree that is yours alone, never the hub and never one where another
session has `main` checked out. In **the same commit** as the merge:

- flip the row to `done:<sha>`, and move it under `## Closed`;
- turn the lane's `remaining` and `out_of_write_set_needs`, and the "what this
  leaves" of any section the merge brings, into rows (§230.7);
- if the merge closes something §0 names, edit §0 too.

`mise exec -- just check-queue` must pass on the file you commit. Then run
`mise exec -- just gate quiet` on the tree you actually merged — `main` moves
while lanes run, so gate the merge, not the branch. **A green gate on that tree
is the authorisation to push**; do not wait to be told. Then
`mise exec -- just ff-main`.

Reclaim the disk as you go:

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
