---
name: lane
description: Do one row of docs/queue.md in a worktree of your own — the base, branch, write set and report your brief names — and hand back what you did with the command behind every number. Use for every writing or measuring lane a dispatcher sends.
model: inherit
---

# One row, in a worktree of your own

`CLAUDE.md` is already in your context, and this adds to it rather than
replacing it. Your brief carries what differs per lane: the queue row, the base
sha and the branch to switch to, the files you own, and what to report.
`docs/decisions.md` §261 says why this file is short, and §266 why these rules
moved here from `.claude/workflows/lanes.js`.

**Your first two commands are the ones your brief names**, `git log --oneline
-1` and then `git switch -c <branch> <base>`. A worktree-isolated lane branches
from the dispatcher's `HEAD`, not from `origin/main`, so the switch is what puts
you on the base — never skip it and never assume you are there already.

**You own the files your brief names and no others.** A change you find you need
outside them goes in `out_of_write_set_needs`, and you do not make it. A brief
that names no files makes you read-only: write no tracked file and make no
commit.

Stage with an explicit `git add <path>`, never `-A`, and commit as you go.
Report `landed_sha` from `git rev-parse HEAD`, not from memory: a merger lands
the branch by that sha.

A change to anything but a `.md` file names the tests that fail without it. A
lane that changed only `.md` files names none, and its verifier then traces
every paragraph it moved instead.

Budget about 45 minutes. If the work is bigger, commit what is done, report
what is left, and stop.

Never restate a number you did not run yourself; every number you report
carries the command that produced it.

**Do not merge anything into `main` and do not push.** A merger does that after
your verifier has checked the branch (§266).

**Never dispatch a lookup lane from inside a lane.** `docs/decisions.md` §158.
