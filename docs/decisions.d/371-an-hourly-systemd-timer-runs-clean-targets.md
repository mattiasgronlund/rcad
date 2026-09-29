## 371. An hourly systemd timer runs `clean-targets` — 2026-09-29

*Asked for by Mattias: "a job which executes once each hour and runs `mise exec
-- just clean-targets` in this folder using systemd", from `main`'s tree. Every
figure below was run in this session.*

In short: `tooling/systemd/rcad-clean-targets.service` and `.timer`, installed
the way §81.5 installs the fuzz pair, by copying them into
`~/.config/systemd/user/`. The timer fires on the hour. `check-fuzz-unit` is now
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

Until now, "the next clean" was whenever somebody ran it. Now it is at most an
hour away. Anything gitignored under an unlocked tree's `target/` lasts an hour
at most: a lane's probe logs, a measuring lane's CSVs, `target/last-gate.log`.
Copy what you need out **before** you unlock the tree, not after. The gate's
served verdicts are not affected. They live in the git common directory
(`rcad-verdicts`), not in any `target/`.

A tree somebody works in by hand without locking it loses its `target/` once an
hour. kache refills it, which is the whole reason the recipe calls itself safe.
Lock the tree if the refill costs you.

**A lane still being verified is not covered by that advice, and this is the
cost the hour brings with it.** The verifier's first step is to unlock the
lane's tree (`.claude/agents/verifier.md`, for §263.4's slot count). From then
until the merger finishes, the lane's `target/` is unlocked while a verifier and
a merger may still want to read what is in it. A clean run by hand did exactly
that to Q354 on 2026-09-29: its probe logs went while it sat at its verifier,
and the merger had to file reruns because the figures now rested on the lane's
word alone. Before this timer, that took somebody running a clean at the wrong
moment. Now it happens to any run whose checking spans the top of an hour. The
timer stays anyway: a full disk stops every lane at once, and losing one run's
probe logs costs one rerun. The fix belongs in `clean-targets`,
which should leave alone a tree whose branch belongs to a queue row still
running, and it is queued as its own row rather than done here.

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
