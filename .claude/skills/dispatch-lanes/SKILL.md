---
name: dispatch-lanes
description: Dispatch the next batch of rcad lanes through the `lanes` workflow, against a base you have watched go green. Use when starting a batch of lanes, when asked to take on the next unit of work, or when a previous batch has merged and the next one is due.
argument-hint: "Which unit, or nothing to take §0's next unclaimed item"
---

# Dispatching a batch of lanes

This skill's instructions tell you to call the `Workflow` tool, which is the
opt-in it needs. Do not stop to ask whether orchestration was authorised — being
invoked is the authorisation. Three or four writing lanes, each drawing its own
Sonnet verifier, is six to eight agents and inside the under-10 guideline.

`$ARGUMENTS` names the unit. Empty means take §0's next unclaimed item.

## 1. A worktree, and the board

`EnterWorktree` first; nobody works in the hub. Then, in one call:

```
mise exec -- just status
```

`status` prints §0, the board, `lanes`, the branches carrying work `main` does
not have, and how far the tip is past the last green — all five things this
skill needs, which is why it is one call and not five. Add `mise exec -- just
plan open` when the argument is empty and §0's next item does not obviously map
to a roadmap row.

§0's **"Next and unclaimed"** is the unit unless `$ARGUMENTS` says otherwise.
Believe §0 over any summary of it, including one in a handoff: §0 is edited by
hand and `status` prints its age, so check that age before leaning on it.

## 2. Do not dispatch into a tree somebody holds

`status` already printed `lanes`. A tree reported `HELD` has an owner named in
its lock reason — message that owner, never send a second agent in. A lock date
does not prove the owner is gone.

## 3. Pin the base, and watch it go green yourself

```
git rev-parse main
mise exec -- just gate quiet
```

One plain Bash call for the gate, `timeout` 600000, no redirect and no pipe
through `tail` — a pipe hands you the pipe's exit code, so a failed gate reads
as 0. Never write a wait loop around it; a foreground call already waits.

The workflow checks in code that every lane started from the pin. **Nothing in
it checks that the pin was green** — that half is yours. GitHub's Actions
allowance is spent, so a red X there in a few seconds is billing and not a
regression, and a local green is the only green there is. `just status` will
usually show `main` some commits past the last green; expect to be the one who
establishes it.

If the tree is cold, pass `run_in_background: true` and let the harness wake
you.

## 4. The write-set table, before any brief

| Lane | Deliverable (one sentence) | May write | Reads |
|---|---|---|---|

Two lanes are concurrent only when their **May write** columns are disjoint;
reads never conflict. If you cannot fill this in, you do not yet know what the
lanes are and dispatching is guessing.

Model per lane. Opus for anything reasoning about a numerical refusal, a
published bound, or whether a claim of impossibility holds. Sonnet by name for:
replaying a corpus, reproducing and classifying a fuzz artifact, bisecting,
running the gate and reporting, sweeping references after a renumber, resolving
a `decisions.md` conflict down to a marker deletion, disk and worktree sweeps.

## 5. Dispatch

`Workflow` with `name: "lanes"` — that is `meta.name` in
`.claude/workflows/lanes.js`, never the filename. If it answers *`Workflow
"lanes" not found`*, the session's registry was memoised before the file
existed; pass `scriptPath: ".claude/workflows/lanes.js"` instead, which bypasses
the registry.

```
args: { base: "<full sha from step 3>",
        lanes: [ { label, brief, writeSet, model, sourcePaths, report, verify } ] }
```

`label` and `brief` are required. `writeSet` absent or empty makes a read-only
lane: no verifier, no commit, and it is told to write no tracked file.
`branch` defaults to `lane-<label>`, and **it must name a branch that does not
exist yet.** The lane creates it with `git switch -c`, which refuses a name
already taken, and the lane then stops as blocked — rather than resetting a
branch that an older worktree may still have checked out, which is what `git
checkout -B` did to two of rscene's lanes (§237). This repo keeps its finished
lane branches, so a label reused from an earlier batch is taken; check with
`git branch --list 'lane-*'` before dispatching. `sourcePaths` is what the verifier reverts
to base to prove the tests fail without the change, defaulting to the
`crates/<c>/src` directories in the write set — set it by hand when that default
is wrong. `report` merges extra properties into that lane's schema for findings
that are its own. `verify: false` skips the check.

**Do not repeat the shared rules in a brief.** The script already appends the
switch onto the base with both shas reported back, the `mise exec --` prefix, the Bash
guard's no-variables rule, the 45-minute budget, every number carrying the
command that produced it, and no merge and no push. A brief carries only what is
that lane's alone — including verbatim API signatures and the exact-arithmetic
facts it needs, because rediscovering those costs a lane twenty minutes.

A test-only lane gets no write access to `src/` and reports defects instead.

## 6. Read the verdicts, not the lanes

A lane reporting its own result is reporting what it believes. The script
cross-examines a `merge` verdict and says so beside it when the verifier said
merge while `tests_fail_without_fix` is not true, when the lane declared no
source directory to revert, or when the branch touched files outside its write
set. Treat any of those three as unverified, whatever the verdict says.

A lane is finished when its worktree unlocks, not when it reports.

## 7. Close it yourself

Merge in a tree that is yours alone, never the hub and never one where another
session has `main` checked out. Run `mise exec -- just gate quiet` on the tree
you actually merged — `main` moves while lanes run, so gate the merge, not the
branch. **A green gate on that tree is the authorisation to merge**; do not wait
to be told.

Then, in order:

```
git worktree unlock <path>        # each tree as you merge it
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
otherwise. If the batch closes something §0 names, the §0 edit goes in **the
same commit** as the section that closes it.
