## 76. Two ways `prune-worktrees` removed the wrong tree, recorded 2026-09-08

`just prune-worktrees` (§50.8) removes a worktree once it is clean and its
branch is an ancestor of `main`. Two ways that test picked the wrong tree, both
found in the same afternoon, and it had no selftest at all — `clean-targets`,
the sibling recipe right above it in the `justfile`, has had one since it was
written (`check-clean-targets-selftest`); `prune-worktrees` did not.

### 76.1 The tree holding `main` is an ancestor of itself

`main` is trivially an ancestor of itself, so the worktree that had `main`
checked out fell through the same test as any merged lane branch and was
force-removed. This is not a hypothetical: it happened on 2026-09-08. The
sweep removed `rcad-worktrees/wave-1`, where `main` was checked out, leaving
`main` checked out nowhere — the exact state §60 exists to prevent. It was put
back with `git worktree add …/wave-1 main`.

The fix is a check ahead of the ancestry test: a worktree whose branch is
literally `main` is kept, with its own reason in the output, because §60
already settled that `main` lives in a worktree and never the hub — this
recipe agreeing with that rule is not a judgement call left to ancestry.

### 76.2 A detached worktree was judged by whichever tree you stood in

`git rev-parse --abbrev-ref HEAD` returns the literal string `HEAD` for a
worktree checked out detached, not a branch name. The old recipe then asked
`git -C "$repo" merge-base --is-ancestor "HEAD" main`, and `$repo` came from
`git rev-parse --show-toplevel` with no `-C`, which from a linked worktree
returns *that* worktree, not the hub. Standing in the tree holding `main` — the
natural place right after a merge — `"HEAD"` resolves there to `main` itself,
trivially its own ancestor. Every clean detached worktree would have been
removed, whatever commit it actually held.

This had not happened yet when it was found, and it is worse than §76.1: a
branch survives a worktree's removal because the branch still exists and still
names its commits (the sweep never deletes branches, only worktrees). A
detached tree has no branch. Its commits are anchored only by that worktree's
`HEAD`, and removing the worktree removes the anchor — a commit not otherwise
reachable becomes collectible. `rcad-worktrees/bend-s-slide`, detached and
clean, was exactly this shape the day this was found.

The fix judges a worktree by its own commit, not a name resolved through
whatever directory the command happens to run from: `git -C "$tree" rev-parse
HEAD` reads it straight from the tree in question. That also retires the old
`[ -z "$branch" ]` guard — it was meant to catch exactly this case and never
did, because `HEAD` is not empty. A detached tree is now recognized directly
(`branch = "HEAD"`) and kept unconditionally, the same way a locked or dirty
one is, rather than run through the ancestry test at all — a commit reachable
from `main` today does not make the anchor disposable, since nothing but that
worktree's `HEAD` names it as safe to keep. `prune-worktrees` also gained a
`dir="."` argument, matching `clean-targets`, so a selftest can point it at a
fixture instead of the repo it runs in.

### 76.3 What let both survive

Neither defect was reasoned out from reading the recipe; §76.1 was read off the
worktree list after the fact, and §76.2 was found by working out git's
worktree semantics by hand, not by a test failing. `check-prune-worktrees-selftest`
now pins five outcomes against one fixture: the tree holding `main` survives,
a locked tree survives, a dirty tree survives, a clean detached tree survives,
and — because a guard that keeps everything passes the first four for free and
proves nothing — a clean lane branch actually merged into `main` is removed.
It is wired into `default` next to `check-clean-targets-selftest`, so a
`prune-worktrees` change without a passing selftest is a red `just default`,
not a quiet gap noticed only when it next deletes the wrong tree.

### 76.4 Most trees carry no lock, and that is not a hook failure

`rcad-51` lost its worktree twice, and the obvious suspects — `prune-worktrees`
above, or the hook that is supposed to write the lock — are both innocent. The
cause is arithmetic about dates.

The lock became the ownership record in `925669a`, 2026-09-06 02:09. It is
written by `.claude/hooks/worktree-create.sh`, and `git show
925669a^:.claude/hooks/worktree-create.sh` has no `worktree lock` line at all
where `925669a`'s has one. So every worktree older than that morning was
created by a hook that could not have locked it, and 15 of the 23 here are:
from `rcad-fuzz` on 2026-09-01 through `spike-floor` at 20:24 on 2026-09-05.
None of them carries a lock, none of them ever could have, and a clean one of
them sitting on a branch already merged is removed correctly by a recipe with
no way to know anyone wanted it. **A missing lock on a tree older than
2026-09-06 is expected**; it is evidence of nothing.

Dates read off the mtime of each tree's `gitdir` file, which is a proxy and
worth one check rather than an assumption: for all five locked trees the ISO
timestamp inside the lock reason equals that mtime to the second — `fuzz-fuse-soak`
locked `2026-09-07T09:39:42Z` against an mtime of 11:39:42.48 +0200, and the
same for `dispatch-2`, `hub-rule`, `fuse-skin-validation` and `questions-grammar`.
Five for five is not a proof, but a proxy that agrees with the only independent
record available is worth more than one that was never compared to anything.

**Three trees younger than the rule carry no lock either, and they are the
interesting ones.** `agent-aa03cabfcfa3d9101` (2026-09-06 09:13),
`fuzz-verify-main` (2026-09-07 14:43) and `wave-1` (2026-09-08 18:56). The
mechanism is that the hook is a *tracked file, read from the session's own
checkout*: a session working in a tree whose HEAD predates `925669a` runs the
version with no lock in it, however new the worktree it is creating. And a tree
made by hand with `git worktree add` never runs the hook at all — which is what
the two named lanes are. So the rule's start date is not one instant across the
repo; it arrives in each session as that session's checkout catches up.

`wave-1` is the one to notice: it holds **`main`**. §76.1 is why that is safe
today — the recipe keeps the tree holding `main` — but the ownership record and
the guard now disagree about which trees are protected, and only the guard is
tested. That is a gap and not a limit: it is reachable, and the fix is a lock,
not a paragraph.

Not written down: which session left `agent-aa03cabfcfa3d9101` unlocked. The
mechanism above explains it and one bad hook run explains it equally well, and
nothing here distinguishes them. `rcad [a57470]` declined to claim it, and so
does this.

