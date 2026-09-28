## 315. A merged run removes its own worktrees — 2026-09-28

*At Mattias's request, after `/` filled. rcad-18 measured the disk and the trees
before a `/clear` and wrote them into its handoff. The numbers after the clear
are marked as re-run.*

### 315.1 Why the trees stayed

On 2026-09-28 `/` (913 GB) was full, with 4.2 GB free. 92 `wf_*` lane trees
held about 335 GB, most of it `target/`. Mattias removed 101 trees by hand. `df
-h /` then read 545 GB used and 322 GB free, re-run after the clear.

Three things kept them:

- **Nothing in a run removes a tree.** `lanes.js` only unlocks. The harness
  removes only a tree with no changes, and a lane's tree has a commit. Removal
  was a manual step in `dispatch-lanes` §7, `clean-targets` then
  `prune-worktrees`. It last ran on 2026-09-27 at 20:47, and runs went on
  finishing until 00:23.
- **`prune-worktrees` waits an hour** after a tree's `HEAD` last moved (§205.2,
  condition 7). So even a dispatcher that runs it straight after a merge
  removes nothing from that merge.
- **Selftest fakes.** Seven trees were kept only by files that
  `check-fuzz-overnight-selftest` left in their `fuzz/artifacts/`. 315.3 covers
  them.

### 315.2 The mark, and who removes

- **`just mark-finished <tree>...`** writes `rcad-run-over` into each tree's
  own git directory. So the mark goes when the tree goes and never shows in
  `git status`. It refuses the hub and any locked tree. It checks every tree
  before it marks any, so a refusal marks nothing.
- **`prune-worktrees` skips the hour for a marked tree, and nothing else.** A
  marked tree that is not on `main`, is dirty, holds a real find, or has a
  process in it is still kept.
- **The merger marks its run's three trees**, the lane's, the verifier's and its
  own, and only when its push went through (`merger.md` §5). The brief gives it
  the line with the first two paths filled in.
- **A `pruner` agent (Haiku) then runs `mise exec -- just prune-worktrees`** in
  the dispatcher's tree, and its lines go into the row as `prune:`. The merger
  cannot do this itself, because the recipe keeps the tree it runs in, and the
  merger's own tree is one of the three. The command is the exact text
  `.claude/settings.json` allows. It takes no argument because the allow rule
  matches the whole string, so the mark carries the information instead. This
  section does not widen the allowlist.
- **Left out on purpose.** A run that did not merge keeps its trees for the
  dispatcher to judge. A read-only lane keeps its tree, because its
  measurements may be in a gitignored `target/` (memory
  `a-failed-lanes-target-data-dies-at-the-next-clean`). A merged lane's
  `target/` scratch is lost, and that is accepted, because its conclusions are
  on `main`.
- **A pruner that throws does not unmerge the row.** A session started before
  `pruner.md` existed cannot resolve the type (§266.6). `pruned` catches that
  and adds a `prune: did not run` line.

**Tests.** `check-prune-worktrees-selftest` gains `marked`, a tree marked
minutes after it was made, which is removed. It also checks that
`mark-finished` refuses a locked tree and the hub, and that a refusal leaves
`just-started`, named ahead of the locked tree, unmarked. On a copy of the
justfile, the unchanged copy passed. Each of five mutants failed on its own
fixture:

- the skip removed;
- the lock refusal removed;
- the hub refusal removed;
- marking while still checking;
- 315.3's filter removed.

`check-lanes-workflow` gains a twelfth arm with three cuts, each caught by that
arm: the pruner started without the `row.merged` guard, its command drifting
from the allow rule, and the merger's mark line dropped. The arm reads the
files as text, like the other eleven.

**Not demonstrated.** A throwaway node harness ran `lanes.js` with a fake
`agent()`. That run showed the pruner is started only for a merged row, with
the three paths, and that a throwing pruner keeps `merged: true`. It is not a
gate step, because the repo pins no node. Two things wait for the first real
merged run: that the allow rule lets a workflow agent run the command without
the auto-mode classifier, and that the harness does not trip over a tree
removed after its agent ended. Until then, both are the documentation's word.

### 315.3 The selftest's leftovers

`check-fuzz-overnight-selftest` writes into the real `fuzz/artifacts/`, under a
per-run tag. Its trap removed only `crash-<tag>` and `slow-unit-<tag>`. Arms
thirteen, fourteen and twenty-three also write `crash-stale-`, `timeout-old-`
and `timeout-` files. Each arm removes its own file when it passes, and none
does when it fails.

- **The trap now goes by the tag alone**, `fuzz/artifacts/*/*-<tag>`. The trap
  is a function that the recipe also calls at its end, and the recipe then
  fails if any file with the tag is left. The last arm leaves its `timeout-` to
  that function. So the arm checks every kind an arm writes, not only the kinds
  somebody named. The recipe passed unchanged, in 83 s. With the old two-kind
  trap put back, it failed and named
  `fuzz/artifacts/where_two_surfaces_cross/timeout-overnight-selftest-…`.
- **The trap runs on TERM, INT and HUP, and not on KILL.** That was measured on
  this machine's bash with a script that sets an EXIT trap and is sent each
  signal (`target/trapprobe.sh`, not kept). So the handoff's 25 `slow-unit-`
  leftovers, which mean the trap never ran, point to a KILL or to the machine
  stopping. Which one it was is not known.
- **`prune-worktrees` ignores `*-overnight-selftest-*`** in its
  `fuzz/artifacts/` check. libFuzzer names an artifact
  `ArtifactPrefix + Prefix + Hash(U)` (`FuzzerLoop.cpp:647`, in
  `libfuzzer-sys` 0.4.13). `Prefix` is a fixed literal such as `crash-` or
  `slow-unit-`, and `Hash` is a SHA-1 in hex (`FuzzerSHA1.cpp:218`), so a real
  find's name never holds `overnight`. The `fakes` fixture holds only such a
  file and is removed. With the filter removed, the selftest fails on it.
