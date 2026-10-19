## 458. `prune-worktrees` deletes the branches `main` holds — 2026-10-19

*Mattias asked why the repo keeps so many local branches, then asked for
`prune-worktrees` to delete the merged ones. This replaces the recipe's
"Branches are never deleted" (§205, §315).*

### 458.1 What was there

On 2026-10-19 the repo had 7,330 local branches:

- 4,412 `worktree-wf_*`, one for each workflow agent's tree;
- 1,621 `lane-*`;
- 984 `merge-*`;
- 245 other `worktree-*`, from sessions and plain agents.

`git for-each-ref --no-merged main` listed 69 of them. The other 7,261 had no
commit that `main` lacks. Nothing ever deleted a branch. `prune-worktrees`
removed trees and said *"Branches are never deleted, so a spike that is never
merged by design (§50.4) keeps its commits"*. That reason covers the 69 and
none of the rest.

Keeping them cost time. `_status-branches` runs one `git cherry` per branch,
and `just status` took 22 minutes on 2026-10-18 (Q2350). `claim-section` loops
over every ref twice too. Nobody has measured whether that is why it outlasts
two minutes.

By the time this was built, someone had deleted the merged branches by hand,
and 86 were left.

### 458.2 The rule

Once the trees are dealt with, the recipe deletes every local branch that
passes all three of these:

1. **`main` holds it** (`git for-each-ref --merged=main`).
2. **No worktree has it checked out.** This is read after the removals, so a
   tree removed in this run gives up its branch in the same run.
3. **No queue row holds it.** `queue.py held` prints the branch of every hold
   in the claims file.

The delete is `git branch -d`, run in the tree holding `main`. So at the
moment of deleting, git checks again that each branch is merged into that
tree's `HEAD` and checked out nowhere. A branch that moved, or was checked out,
after the recipe listed it is refused, and the recipe fails rather than
deleting it.

Why each rule:

- **Local `main`, not `origin/main`.** git's second check measures against the
  `HEAD` of the tree it runs in (or a branch's upstream, if it has one), so the
  list has to use the same ref. A branch that only `origin/main` holds waits
  for `ff-main`. Removing a tree accepts either ref (§205.2), because a tree
  holds nothing a ref does not keep. Here the safety comes from git's second
  check, and that check knows one ref.
- **Checked out nowhere.** git refuses a checked-out branch anyway. Leaving
  those out of the list means a refusal only ever comes from a branch that
  changed in the meantime, which is what the refusal is there to catch.
- **No queue hold.** A lane that commits nothing leaves its branch at its
  base, which is on `main`. Once its tree is detached (Q278 on 2026-10-09,
  §299.6) or removed, nothing has the branch checked out. The verifier of a
  `does_not_reproduce` lane still runs `git rev-parse` on it
  (`branch_at_base`, `.claude/workflows/lanes.js`). A hold lasts until the
  row's `done`, `drop` or `park`, so the branch goes at the first prune after
  that.
- **The recipe fails when no tree holds `main`.** §60 puts `main` in a
  worktree, and without one there is nowhere to run `branch -d`.

Deleting a branch loses two things: its name and its own reflog. A merged
lane's name is also in its merge commit's subject (`Merge lane-…`), and its tip
is that merge's second parent. The reflog §299.6 reads is the tree's `HEAD`
reflog, not the branch's, so it stays.

The test is ancestry, not content. A branch whose work was reworked or
cherry-picked onto `main` (§92.1) does not count as merged, so it stays. That
is the safe side to err on. §237.1's check that a label is free still matters
for lane branches that never landed, and `dispatch-lanes` now says so.

**Measured:** 7,000 packed branches, deleted with one `git branch -d` call,
took 1.7 s with git 2.43. `update-ref --stdin` took 0.5 s and `branch -D`
1.6 s. So the delete is not split up further.

### 458.3 The tests

`check-prune-worktrees-selftest` gains five branches that no tree has checked
out. `landed` and `released` are on `main`. `released` also has a queue row
that took it and later released it. `held` is on `main` with a row still
holding it. `unlanded` is on no landed ref, and `origin-only` is only on
`origin/main`. The test expects `merged` (whose tree is removed), `landed` and
`released` to be deleted. It expects `main`, `runner` (checked out),
`held`, `unlanded` and `origin-only` to stay. The tally line counts three.

The selftest was run against scratch copies of the justfile and `tooling/`.
The unchanged copy passed, and each of six mutants failed:

| mutant | how it failed |
|---|---|
| no delete (the recipe as it was before) | "the merged branch was not deleted" |
| hold filter removed | "the held branch was deleted" |
| checked-out filter removed | git refused `main` and `runner`; exit 123 |
| `--merged=origin/main` | git refused `origin-only`; exit 123 |
| `branch -d` in the recipe's own tree | git refused `landed` and `merged`; exit 123 |
| `held` ignores releases | "the released branch was not deleted" |

The checked-out mutant shows that git's own check keeps `main` safe even
without the filter.

### 458.4 What this leaves

Q2350, running `cherry` only over branches `main` lacks, is still right. Its
22 minutes came from 7,124 branches, nearly all of them merged. With those
gone, `just status` should be short again without it. Not measured.
