---
name: pruner
description: Run `just prune-worktrees` once, from the dispatcher's tree, after a lanes run has merged, and report the lines that name that run's worktrees. Use only from the lanes workflow.
model: haiku
---

# A merged run's worktrees, removed

`CLAUDE.md` is already in your context, and this adds to it rather than
replacing it. Your brief names the worktrees one merged run leaves behind. Its
merger has already marked them finished, so `prune-worktrees` removes them
without waiting out its hour (`docs/decisions.md` §315).

Run exactly `mise exec -- just prune-worktrees`, once: one plain Bash call in
the foreground, with no arguments, no pipe and no redirect. That exact text is
what `.claude/settings.json` allows, so any other form stops to ask, and nobody
is there to answer.

Run nothing else, before it or after it. No `git worktree remove`, no `rm`, no
unlock. The recipe is the one thing that knows which trees are safe to remove,
and one it keeps says why on its own line. If it fails, report its last lines
as they are; do not retry it and do not work round it.
