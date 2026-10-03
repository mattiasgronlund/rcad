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

**Every command you run stays in the foreground, this role included.**
`run_in_background` reports back before the job finishes and nothing wakes you
for the result: Q206's first lane came back empty because the harness forced
its report before a backgrounded test run had printed anything, and Q148's
merger reported three steps into the gate and never pushed. Run the gate as
one plain Bash call, `timeout` 600000. A job that runs past ten minutes goes
in chunks of under ten, never in the background. A run that cannot be chunked
starts detached with `setsid nohup ... &`, and you wait on it with `timeout 540
tail --pid=<pid> -f /dev/null`, one foreground call at a time, until it returns
0 rather than 124. The harness refuses a foreground `sleep`, and the background
wait it offers instead ends your turn and forces your report (§302).

**Your first two commands are the ones your brief names**, `git log --oneline
-1` and then `git switch -c <branch> <base>`. A worktree-isolated lane branches
from the dispatcher's `HEAD`, not from `origin/main`, so the switch is what puts
you on the base — never skip it and never assume you are there already.

**You own the files your brief names and no others.** A change you find you need
outside them goes in `out_of_write_set_needs`, and you do not make it. A brief
that names no files makes you read-only: write no tracked file and make no
commit.

Run `mise exec -- just fmt` before each commit. It reformats the whole tree,
but staging by explicit path already keeps that harmless — only your own
files get added. The gate checks formatting but does not fix it: skip
`just fmt` and `pre-gate` stops you in seconds, but only after the commit,
so the fix lands as a rustfmt-only commit on top of the real change, the
way Q574's tip did.

Gate with `mise exec -- just pre-gate gate quiet`. `pre-gate` runs the gate's
cheapest steps at once — formatting, clippy and the wildcard count — and
prints the first 60 lines of each one that fails. `just` starts the gate only
when they all pass, so those failures come back in seconds instead of after a
whole gate: 4 of 24 lane gates failed on them alone, with every test passing
(§380, §381).

Stage with an explicit `git add <path>`, never `-A`, and commit as you go.
Report `landed_sha` from `git rev-parse HEAD`, not from memory: a merger lands
the branch by that sha.

A change to anything but a `.md` file names the tests that fail without it,
and a lane that adds a test — a `#[cfg(test)]` pin in `src/` included — still
names it. A lane whose `lines-that-run` print ends `lines that run: 0` (`.md`
files, comments, test code) may name none, and its verifier then traces every
paragraph it moved instead.

**What your own change leaves stale is yours to fix, not to file** (§352.5): a
doc comment or a section's claim your change makes false, a test it leaves
missing. Fix it inside your write set before you report. Only what lies outside
the write set goes in `out_of_write_set_needs`.

**In `remaining`, give each defect you leave its class** (§352.3): `wrong`,
`crash` or `refuses`, then `/A` when a real input met it, `/B` for a shape
designers make on purpose, `/C` when only a probe builds it, `/D` when it lies
outside §3.5's scale envelope, with the number that puts it there. A hypothesis
nobody has run gets no class at all (§360.5). Mark an item `small` when it
needs no trace, its place is known, and it takes about ten minutes. A defect is
never small.

**A brief naming several rows is a bundle of small rows** (§352.4): one branch,
one report. Do each, and commit each on its own so its verifier and merger can
tell them apart. One that turns out not to be small you leave undone, and say so
in `remaining` with its id and why.

Budget about 45 minutes. If the work is bigger, commit what is done, report
what is left, and stop.

Never restate a number you did not run yourself; every number you report
carries the command that produced it.

**Do not merge anything into `main` and do not push.** A merger does that after
your verifier has checked the branch (§266).

**Never dispatch a lookup lane from inside a lane.** `docs/decisions.md` §158.
