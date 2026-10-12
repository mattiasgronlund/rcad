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
you on the base — never skip it and never assume you are there already. A relay,
a lane carrying on a branch another lane handed over (§416), gets other first
commands in its brief: a plain `git switch` onto that branch and the checks after
it.

**A row naming the inputs that show its fault runs them on the base first**
(§433.8). Those are the crash files, the corpus files and any shape the row
says to build. Run each one before you read any code:

- A crash file: `mise exec -- cargo run -q --profile test -p rcad-fuzz-explain
  -- <target> <crash file>`, where the target is the directory under
  `artifacts/` that holds the file. It shows its fault when the print says
  `outcome: panicked`, as `Outcome`'s `Display` in
  `crates/rcad-fuzz-explain/src/lib.rs` writes it.
- A corpus file: the test or recipe the row names for it, run as it stands.
- A shape: a probe test you write and do not commit, run with `mise exec --
  just t <name>`, then remove.

If none of them shows the fault, stop there. Commit nothing, so `git status
--porcelain` prints nothing, and report outcome `does_not_reproduce`, with
`landed_sha` the base; `files_changed`, `tests_that_fail_without_the_change`
and `rows` empty; `for_the_verifier` `none`; and `base_runs` holding each
input, the command, the probe's path and whole text for a shape, the lines of
its print that say whether the fault shows, and `shows_fault`. Do not look for
why it is gone: your verifier runs the same commands on the base, and on its
confirm the row closes done at the base. If any of them shows the fault, do
the row as usual.

**You own the files your brief names and no others.** A change you find you need
outside them goes in `out_of_write_set_needs`, and you do not make it. A brief
that names no files makes you read-only: write no tracked file and make no
commit.

**Narrow your hold once you know where you work** (§424.3, §424.7). Your row
holds every file its write set names until it lands, and the fill holds back
each ready row that meets it, so a row on a whole directory blocks that
directory for your whole run. At your first edit under `src/`, run `mise exec
-- just queue narrow <id> <entries>` to the files or items you now know you
will change, and again before you report, to the entries your branch changed.
Write each entry inside the set your row was taken with and the way that set
names it: `justfile#queue`, not `justfile`, for a row taken on
`justfile#queue`. A bundle narrows each of its rows. `narrow` writes the claims
file alone and never `docs/queue.md`, so it is yours to run from your own tree.
It refuses a widening back, to an entry inside the taken set but outside the
current one, while another lane's hold meets that entry: you may still change
it, since your write set is still the one you were sent with, but name the
entry and the hold it quotes in `remaining`, since the two branches may meet
at merge.

Run `mise exec -- just fmt` before each commit. It reformats the whole tree,
but staging by explicit path already keeps that harmless — only your own
files get added. The gate checks formatting but does not fix it: skip
`just fmt` and `pre-gate` stops you in seconds, but only after the commit,
so the fix lands as a rustfmt-only commit on top of the real change, the
way Q574's tip did.

Gate with `mise exec -- just pre-gate gate-lane`. Commit first: the gate
refuses a tree with uncommitted or untracked files, since no sha names what it
would read and your verifier would run it all again (§415). `pre-gate` runs the gate's
cheapest steps at once — formatting, clippy and the wildcard count — and
prints the first 60 lines of each one that fails. `just` starts the gate only
when they all pass, so those failures come back in seconds instead of after a
whole gate: 4 of 24 lane gates failed on them alone, with every test passing
(§380, §381).

`gate-lane` is the whole gate but `test-slow`, the slow tier, which the land
gate runs (§397.4). The slow tier is listed in `slow_tests` in the justfile:
run the ones that cover what you changed with `mise exec -- just t <name>`.

A base older than §397.4 has `pre-gate` but no `gate-lane`, and there you gate
with `mise exec -- just pre-gate gate quiet`. A base older than §381 has no
`pre-gate` recipe at all — `just` says
`Justfile does not contain recipe pre-gate` — and there you gate with the
plain `mise exec -- just gate quiet` instead, as Q753's second run did on
`0c8a44fc`. A run that carries on an earlier run of its own row keeps that
run's branch and base, so it meets whatever base that was, not whatever
`main` has moved to since (§381).

Stage with an explicit `git add <path>`, never `-A`, and commit as you go.
Report `landed_sha` from `git rev-parse HEAD`, not from memory: a merger lands
the branch by that sha.

**Commit the failing test first, on its own, then the fix.** Once a test fails
the way your row says it should, commit it before you change the code it tests,
then commit each part of the fix as its tests pass. What you commit is what a
lane after you starts from. From 10-01 to 10-08 a lane's first commit came at a
median of 203k, the same as the median lane's peak, so a lane that stopped
earlier usually handed on nothing. A
second lane re-read 14–26k tokens when the first had committed code, and 60k and
105k when it had committed nothing (§416).

A change to anything but a `.md` file names the tests that fail without it,
and a lane that adds a test — a `#[cfg(test)]` pin in `src/` included — still
names it. A lane whose `lines-that-run` print ends `lines that run: 0` (`.md`
files, comments, test code) may name none, and its verifier then traces every
paragraph it moved instead.

A lane your brief marks `sameBehaviour` changes lines that run and keeps what
they compute — one of two copies dropped for the other, a body lifted into a
function, a block moved — so no test fails without it (§399). Leave
`tests_that_fail_without_the_change` empty, and name in
`tests_that_reach_the_change` the tests that run the lines you changed and pass
before and after; your verifier breaks one of those lines and needs one of them
to fail. Your commit message argues, case by case, that the old and the new give
the same answer on every input, as `30a22f66`'s does; the verifier reads it
against the diff and rejects a case it leaves out. If the change turns out to
alter an answer, report `blocked` and name the input.

**What your own change leaves stale is yours to fix, not to file** (§352.5): a
doc comment or a section's claim your change makes false, a test it leaves
missing. Fix it inside your write set before you report. Only what lies outside
the write set goes in `out_of_write_set_needs`.

**In `remaining`, give each defect you leave its class** (§352.3): `wrong`,
`crash` or `refuses`, then `/A` when a real input met it, `/B` for a shape
designers make on purpose, `/C` when only a probe builds it, `/D` when it lies
outside §3.5's scale envelope, with the number that puts it there. A hypothesis
nobody has run gets no class at all (§360.5). A fault from a fuzz crash file is
C or D only once you have run its shape again with every out-of-range number
made ordinary and every relation kept: a scale by a power of two, a part near
zero set to zero. The same failure makes it at least C. Name the ordinary shape
you ran beside the crash file. Untried, it has no class (§440.2). When your
work finds which feature of a shape a fault needs, build a shape designers
make with that feature and run it. If it fails the same way, the rows you leave
for that fault are B (§440.3). Mark an item `small` when it
needs no trace, its place is known, and it takes about ten minutes. A defect is
never small.

**You write the rows your work leaves, in `rows`** (§383.4): the steps of one
`just queue apply`, each a list of words. A merge on your branch closes your row
and runs them, and when your verifier says `merge`, a recipe does it with
nobody reading your report again. So every item of `remaining` and
`out_of_write_set_needs` that is work for later, and every item of the "what
this leaves" of a section you wrote (§230.7), gets a step. `remaining` keeps
what is not a row.

- `["add", "<tier>", "<model>", "<write set>", "<what, ending in §N>"]`, with
  §N your section or the one your row cites. The write set is one word, paths
  joined by commas, or `-` for none. A row whose fix adds a refusal, or turns a
  panic into one, names its crate's `error.rs`, since the new variant goes
  there: Q574, Q604 and Q605 were each dropped and filed again for want of it
  (§360.6). Every row says which row it came from (§432), and `just land`
  puts `from/<your row>` in front of a `<what>` that does not say. In a
  bundle, start `<what>` with `from/<id>` for an item from a row that is not
  the first; start it with `was/<id>` for a row that files `<id>` again.
  A row for a fault that a crash file of a fuzz target, or a file of the STEP
  corpus, shows starts with `from/fuzz:<target>` or `from/step:<file>`, not
  `from/<your row>`, even when your row's work turned it up (§433.2).
  `<target>` is the target's file name under `fuzz/fuzz_targets/` without
  `.rs`. `<file>` is the corpus file's own name with each space written `_`,
  or with one directory in front where `just queue add` says the name ends two
  paths (§433.1). A triage lane starts each fault row it reports with the
  target whose crash file that row names: the directory under `artifacts/`,
  and the first column of `fuzz-triage-members.tsv` (§433.3). A row your
  work leads to that is not such a fault still comes from your row.
- `["tag", "@1", "<class>"]` or `["tag", "@1", "small"]`, one tag at most per
  row. A defect's row takes its class from `remaining`: `wrong`, `crash` or
  `refuses`, then `/A` to `/D` as above. The class names the tier (§403): a row
  of class A or B is `add 1`, and one of class C or D is `add 5`. A row in any
  tier is `small` by the rule above.
- `["after", "@1", "<id>[,<id>]"]` for a row that waits on another. A row
  with no class goes down to the latest tier among the rows it waits on, so
  `add 1` and then `after` a tier 5 row lands it in tier 5 (§435.6).

`@1`, `@2` are the first and second `add` of your own list. Ids come from a
shared counter, so a step never names a new id itself. Before you add a row,
look for it among the open rows of `docs/queue.md` in your tree, which is the
base's copy; `grep -n` a word of it. A row already there is not added again;
name it in `remaining` instead. An empty list means nothing to add.

Tiers are §230.1's in §311's order: 1 a new defect of class A or B, or one with
no class, 2 a follow-up that measures or improves what the tests and the nights
search, with §307's 2.2 between 2 and 2.5, and §403's 5, below 4, for every
defect of class C or D. A row that changes how the work is done — the queue and its verbs, the
lanes workflow and its roles, hooks, skills, `CLAUDE.md`, a recipe that is
neither a test nor a fuzz night — goes in 2.2, never in 1 or 2. Opus for a
numerical refusal, a published bound or a claim of impossibility; Sonnet for a
replay, a fuzz artifact, a bisect or a sweep. What your own change leaves stale
inside your write set is not a row: fix it, under §352.5 above.

**A brief naming several rows is a bundle of small rows** (§352.4): one branch,
one report. Do each, and commit each on its own so its verifier and merger can
tell them apart. One that turns out not to be small you leave undone, and say so
in `remaining` with its id and why.

Budget about 45 minutes. If the work is bigger, commit what is done, report
what is left, and stop.

**When "Lane budget passed" comes and your row is not done, hand over** (§416).
This holds even if your brief says to carry on past the warning: that sentence
is older than the relay. Finish the call you are on, then:

1. Commit everything you have, each file by an explicit path, a half-done fix
   included, with a message that says what is unfinished.
2. Run `git switch --detach`, so the lane after you can check your branch out.
3. Report `outcome: relay`, with `landed_sha` your last commit and `handoff`
   saying what is done, by commit; what is next, in order; the places, as file,
   function and line range; what you measured, with its command; and what you
   tried that did not work.

The workflow starts a fresh lane on your branch with your brief and your
handoff. A handoff that names places saves that lane the reading you did; one
that says "see the code" costs it all again. Fill in the rest of the report as
usual, since the lane after you carries it on. A row that is done when the
warning comes is reported done, not handed over. A read-only lane has nothing
to commit, so it reports what it has, as the warning says.

Never restate a number you did not run yourself; every number you report
carries the command that produced it. Its `value` is text that command prints,
exactly as it prints it, since your verifier's `just verify-lane` runs the
command and looks for the value in its print (§385). Write "164 of 164" only
when the command prints that; a value you worked out from its print goes in
`what` or `summary`, with the printed text as the value.

**Do not merge anything into `main` and do not push.** A merger does that after
your verifier has checked the branch (§266).

**Never dispatch a lookup lane from inside a lane.** `docs/decisions.md` §158.
