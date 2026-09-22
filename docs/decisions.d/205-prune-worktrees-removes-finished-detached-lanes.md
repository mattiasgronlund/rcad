## 205. `prune-worktrees` removes a finished lane even when it is detached — 2026-09-22

*The rule is rscene's (`eed893f`), ported at Mattias's request after §203.4
found the repo keeping 55 worktrees. Numbers are from runs made here.*

### 205.1 What §76 cost

§76 fixed two ways the recipe removed the wrong tree. The second was a detached
tree judged through the wrong tree's `HEAD`, and the fix kept every detached
tree whatever it held. Workflow lanes here end detached. On 2026-09-22, just
after `clean-targets`, the recipe removed 7 trees and kept 55, and most of
those were clean, unlocked and detached at a commit already on `main`.

§76 needed only half of what it did. The fault was *which* `HEAD` was asked
about. The recipe now reads each tree's commit from `git worktree list
--porcelain` and never resolves a name, so a detached tree can be judged like
any other.

### 205.2 The conditions

A tree is removed only when all of these hold. The recipe's comment says why
each one is there:

1. It is neither the hub, nor the tree the recipe runs in, nor the tree the
   overnight fuzz unit runs in (205.4).
2. It is unlocked.
3. It does not hold `main` (§60, §76).
4. Its `HEAD` is on `main` or on `origin/main`.
5. Nothing is uncommitted, untracked files included.
6. `fuzz/artifacts/` holds no files.
7. Its `logs/HEAD` is an hour old.
8. No process has its working directory inside it.

rscene's version has 1 (without the fuzz tree), 2, 4 (against `origin/main`
only), 5, 7 and 8. Three changes are rcad's besides 205.4:

- **3** is §76's first defect, which rscene does not have because its `main` is
  in the hub.
- **4** also accepts local `main`, because merges here land on local `main` in
  a worktree and are pushed afterwards. Either ref keeps every commit the tree
  reaches, so neither can lose work. rscene measures against `origin/main`
  because its hub's local `main` was 65 commits behind.
- **6** is §92.3: `fuzz/artifacts/` is gitignored, so a tree holding the only
  copy of a crash input passes 5.

Condition 7 is what makes the port safe here and not only in rscene. A
workflow lane that has just started sits at its base, which is on `main`,
clean, and unlocked, because the harness does not lock a workflow lane
(`worktree-create.sh` locks only the trees it creates). Without the hour, that
lane would be removed out from under its agent.

`git worktree remove` loses `--force`. Ignored files such as `target/` do not
make it refuse, and a tree that passed 5 has nothing else. So a refusal now
means one of the conditions is wrong, and the recipe fails instead of forcing.

### 205.3 The selftest, and the mutations that prove it

`check-prune-worktrees-selftest` builds this repo's layout: a detached hub,
`main` in a worktree of its own, and a runner. It adds one tree for each keep
reason and two finished lanes, one detached on a commit only `origin/main` has
and one on a branch merged into `main`. The detached one also holds an ignored
`target/`. Each keep case fails exactly one condition.

Each condition was then turned off in turn on a copy of the justfile, and the
selftest was run against that copy. The unchanged copy passed. Every one of the
nine mutants failed, and each named the tree it should have kept. Seven failed
the selftest's own checks. The locked and dirty mutants failed because
`git worktree remove` refused, which without `--force` is the point. The
`origin/main` mutant left the detached finished tree on disk.

Writing the selftest found one bug in the port before it landed. `read` with
`IFS=$'\t'` folds two tabs in a row into one, so a detached tree's empty branch
field moved the lock flag into the branch column, and a locked tree read as
unlocked. The branch is the last field now.

### 205.4 The overnight fuzz tree is kept by name

`rcad-nightly-fuzz` was kept only by its lock, whose reason reads "overnight
fuzzing", and by the 2103 files in its `fuzz/artifacts/`. Neither is meant to
protect it. A lock records which agent owns a tree, and `release-lanes` exists
to drop locks in bulk. The artifacts are triaged away, and a night that finds
nothing leaves none. With both gone, `prune-worktrees` would remove the tree
the timer runs in, and `clean-targets` would delete its `fuzz/target` mid-night.

Both recipes now read `WorkingDirectory` from
`tooling/systemd/rcad-fuzz.service`, with `%h` expanded. They keep that tree,
locked or not, and fail if the unit names no directory. They read the tracked
copy, not the installed one, because the tracked copy is what this tree
means. `check-fuzz-unit` already fails when the two drift apart. Each selftest
has an unlocked fixture tree that a fixture unit names. Turning the rule off in
either recipe fails that recipe's selftest on that tree.

rscene also has a `check` mode that `gate` runs, which fails while a removable
tree is on disk. That was **not** ported. What it checks is this machine's
worktrees, not the tree being gated, which is §173's class. It would also turn
the gate red for an hour after every lane finished.
