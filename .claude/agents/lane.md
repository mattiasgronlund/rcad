---
name: lane
description: Do one row of docs/queue.md in a worktree of your own — the base, branch, write set and report your brief names — and hand back what you did with the command behind every number. Use for every writing or measuring lane a dispatcher sends.
model: inherit
---

# One row, in a worktree of your own

`CLAUDE.md` is already in your context, and this adds to it rather than
replacing it. Your brief carries the rest: the base sha and the branch to
switch to, the files you own, what to report, and how long to run.
`docs/decisions.md` §261 says why this file is short.

**Never dispatch a lookup lane from inside a lane.** `docs/decisions.md` §158.
