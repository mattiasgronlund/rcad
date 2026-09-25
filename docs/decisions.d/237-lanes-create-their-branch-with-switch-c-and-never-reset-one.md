## 237. Lanes create their branch with `switch -c` and never reset one — 2026-09-25

*rscene's `235ccaf` (2026-09-22) found that `git checkout -B`, one of the first
commands its lanes were given, had reset two of their branches while older
worktrees still had them checked out. `.claude/workflows/lanes.js` here gave
every lane the same line. This records the fix and how it is held.*

  237.1   What changed, and what it costs the dispatcher
  237.2   How it is held, and why not rscene's harness
  237.3   What this leaves

### 237.1 What changed, and what it costs the dispatcher

`lanes.js` now gives each lane `git switch -c <branch> <base>`, which refuses a
name already taken, and tells the lane to stop and report `blocked` when it
does, never to reach for `checkout -B` or `switch -C`. That sentence matters as
much as the command: without it, the nearest fix a lane knows for a refused
switch is the reset.

What a reset costs is not visible where it happens. The older worktree keeps its
files and its index, so `git status` there shows a staged revert of everything
since its own commit, and a commit made there undoes that on the branch. In
rscene it was round 21's lane-B and lane-C showing a revert of round 22, found
from the branches' reflogs.

The cost moves to the dispatcher: a label must now name a branch that does not
exist yet. This repo keeps its finished `lane-*` branches, so a label reused
from an earlier batch stops that lane at its second command. `dispatch-lanes`
step 5 says so and names `git branch --list 'lane-*'` as the check. A lane
stopped there costs one lane start; a reset branch costs a revert found late, or
never.

### 237.2 How it is held, and why not rscene's harness

`just check-lanes-workflow` reads `lanes.js` for three things: the `switch -c`
line exactly once, every line naming `checkout -B` or `switch -C` being the one
that warns against them, and the refusal sentence. Each is proven by a cut of
the real file that must be caught first by that arm. Run against `main`'s
`lanes.js` at `39b278cc`, it fails on the first arm, *"has the line '  git
switch -c ${branchOf(lane)} ${BASE}' 0 times, expected once"*; on the fix it
passes with all three cuts caught.

It reads the source, not the prompt a lane is actually sent, so it cannot tell
whether the line sits inside the text a lane receives or somewhere dead. rscene
closes that with a node harness, `.claude/lanes-selftest.mjs`, which renders
every prompt. It was not brought here, for two reasons. This repo pins no node,
and no other check here needs one. And the harness is written against rscene's
workflow, which has moved on from this one: by their names, 16 of its 24 cases
test something this repo's `lanes.js` does not do — nine for device lanes and
the goldens, two for recipe checks in the mutation sentence, three for
verifying by mutation, one for refusing a merge without a green gate, one for
refusing two lanes that share a file. None of the 24 checks `switch -c`. Porting
it would mean writing a harness for the eight cases left plus the one this
section needs, to hold a literal line that a grep already holds. The harness is
the right tool on the day this repo takes rscene's workflow whole, and that is a
separate decision not taken here.

### 237.3 What this leaves

- §124, §126, §127, §128, §130, §131 and §132 quote `git checkout -B` in the
  commands they record having run. They say what was run, and are left alone.
- The shared `delegating` skill in the `practices` plugin told a brief to open
  with `git checkout -B <lane-branch> <base sha>`. It lives in the marketplace
  repo, and got the same fix there in `82e46e3`. A probe for that commit, on
  this machine's git 2.43.0, is the mechanism measured rather than read off a
  report: `checkout -B lane-B main` exited 0 while another worktree held
  `lane-B`, that tree's `git status` then showed the newer round's file as a
  staged deletion, and `switch -c` on the same name answered *"fatal: a branch
  named 'lane-B' already exists"*.

---
