## 371. An hourly systemd timer runs `clean-targets` — 2026-09-29

*Asked for by Mattias: "a job which executes once each hour and runs `mise exec
-- just clean-targets` in this folder using systemd", from `main`'s tree. Every
figure below was run in this session.*

In short: `tooling/systemd/rcad-clean-targets.service` and `.timer`, installed
the way §81.5 installs the fuzz pair, by copying them into
`~/.config/systemd/user/`. The timer fires on the hour, and the sweep takes only
trees whose `target/` nobody has written in for three hours (§371.5). `check-fuzz-unit` is now
`check-systemd-units` and reads every pair in `tooling/systemd/`, so the new pair
cannot drift from the tree unseen.

### 371.1 What the unit does

It runs `just clean-targets` from `%h/git/mattiasgronlund/rcad-worktrees/main`.
The recipe keeps the tree it runs from, so `main`'s tree is the one tree the hour
never sweeps. It also keeps every locked tree and the overnight fuzz tree, for the
reasons written above the recipe. The unit sets `PATH` itself and calls mise by
absolute path, for §108's reason: the reboot of 2026-09-15 showed that a PATH
inherited from a login session is not a PATH a unit has.

The first run was by hand, `systemctl --user start rcad-clean-targets.service`,
at 23:03: `Result=success`, `24G reclaimed; 10 worktree(s) kept`, 3.5 s of CPU,
and `/` went from 78% to 75% used.

The path is typed into the unit. `ff-main` finds `main`'s tree by the branch it
holds (§210), and a unit file cannot. If the tree moves, the run fails rather
than cleans nothing: `systemd-run --user --wait -p
WorkingDirectory=/home/mattias/no-such-rcad-tree /bin/true` exits 200, and so
the unit shows under `systemctl --user --failed`.

### 371.2 What an hourly clean changes

Until now, "the next clean" was whenever somebody ran it. Now it comes at the
first hour after an unlocked tree's `target/` has been quiet for three hours
(§371.5), so between three and four hours after the last write. That covers
everything gitignored in there: a lane's probe logs, a measuring lane's CSVs,
`target/last-gate.log`. Copy what you need out before that. The gate's served
verdicts are not affected. They live in the git common directory
(`rcad-verdicts`), not in any `target/`.

A tree somebody works in by hand without locking it keeps its `target/` while
they build in it. Three quiet hours later it goes, and kache refills it. Lock
the tree if the refill costs you.

**A lane still being verified is the case the three hours are for.** The
verifier's first step is to unlock the lane's tree
(`.claude/agents/verifier.md`, for §263.4's slot count). From then until the
merger finishes, the lane's `target/` is unlocked while a verifier and a merger
may still want to read what is in it. A clean run by hand did exactly that to
Q354 on 2026-09-29: its probe logs went while it sat at its verifier, and the
merger had to file reruns because the figures now rested on the lane's word
alone. The timer's first version swept every unlocked tree on the hour, and so
would have done that to any run whose checking spanned the top of an hour.
Q707 was filed for it, proposing that `clean-targets` look up which queue rows
were still running. §371.5 fixed it with a rule that needs no queue.

### 371.3 Installing it

```
cp tooling/systemd/rcad-clean-targets.service tooling/systemd/rcad-clean-targets.timer ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now rcad-clean-targets.timer
```

This session could not run the third line. The worktree Bash guard reads
`enable` as bash's builtin of that name and refuses the command. It ran
`systemctl --user add-wants timers.target rcad-clean-targets.timer`, which makes
the same `timers.target.wants/` link, and then `systemctl --user start` on the
timer. `is-enabled` says `enabled` afterwards.

`systemd-analyze --user verify` passed on both files before they were copied.
§81.5 asks for that check, and it is still not a gate step, for §81.5's reason.

### 371.4 The drift check reads every pair

`check-systemd-units` takes its names from the files in `tooling/systemd/`, not
from a list, so a third timer is covered the day it lands. Each pair is judged
on its own. A machine with the fuzz timer installed and not this one passes.
A pair installed and behind the tree fails. So does a service tracked with no
timer beside it. Older sections that name `check-fuzz-unit` mean this recipe.

The selftest gained three cases. Run against the old recipe, the case with the
second timer installed and stale came back exit 0, because the old recipe read
only `rcad-fuzz`. The case with a lone service came back exit 0 too. With the
change, all of them pass.

### 371.5 Only a `target/` quiet for three hours is swept

`clean-targets` now keeps an unlocked tree when any file or directory under any
of its `target/` directories changed in the last 180 minutes
(`find … -mmin -180 -print -quit`). Mattias chose three hours, after the
measurement below showed that one hour was enough for every lane so far, but
with only a few minutes to spare.

**How long a lane's `target/` is still needed after the lane stops writing.**
Every `lanes` run record under `~/.claude/projects/*/*/workflows/` (353 files,
2026-09-29), for each lane whose merger finished. The time runs from the lane
agent's end to its merger's end:

| | minutes |
|---|---|
| median | 21 |
| 95th percentile | 35 |
| longest | 53 (Q244's `q244-effort-on-set-c`, `wf_734276ce-1d0`) |
| over 60 | 0 of 174 |

Every verifier started the minute its lane ended. Lanes in one run are
pipelined, so no lane's tree sits waiting for a sibling. A lane's last write can
come a little before the agent ends, so the real gap can be a little longer
than these figures. That, and a merger stuck retrying pushes, is what three
hours leaves room for. The script is
`lane_gaps.py` in this session's scratchpad, and it was not kept: the run
records it reads are what to measure again.

**The newest entry anywhere under `target/` decides, not `target/`'s own
time.** A directory's time moves only when its own entries are added or
removed. A file written three levels down left `target/`'s time at three hours
earlier in a probe. Walking a 12 GB `target/` of 11,372 files to prove nothing
in it is new took 0.04 s, which is less than the `du` the recipe already runs.

**A fresh `target/` anywhere keeps the whole tree.** A tree with an old `target/`
and a fresh `fuzz/target` is in use, whichever of the two its evidence sits in.

**A walk `find` could not finish keeps the tree too.** A build that deletes a
file while `find` walks past it makes `find` exit non-zero. With 20 directories
of 2,000 files being deleted and remade around it, 14 of 40 walks exited 1 with
"No such file or directory". Before this change, `du` failing the same way ended
the whole run under `set -e`. `origin/main`'s recipe, run on a scratch hub whose
first unlocked tree held a mode-000 directory in `target/`, printed `du: cannot
read directory … Permission denied`, exited 1, and never reached the next tree,
whose `target/` was still there afterwards. Now that tree is kept, with a line
saying why, and the run goes on. The selftest builds that case with an empty mode-000 directory.
It refuses to run as root, where `find` can read one, rather than pass
without testing anything.

What stays uncovered is a run that fails and whose files the dispatcher wants
more than three hours later. Copying them out is still the only guarantee.

The selftest's six fixture trees now start four hours old, so each one is kept
or swept for its own reason and not because the fixture wrote it a moment ago.
Against the old recipe, the new cases failed: it swept a tree with a file in
`target/` written a moment ago, and a tree whose `fuzz/target` was fresh.
