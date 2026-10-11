---
name: dispatch-lanes
description: Fill rcad's free lane slots from `docs/queue.md`, one `lanes` workflow per row — lane, verifier, merger — against a base you have watched go green, and read back what each run merged. Use when taking on the next work, or when a row's run returns and its slot is free.
argument-hint: "A row id to take first, or nothing to take what `just queue next` gives"
---

# Filling the lane slots from the queue

`docs/queue.md` holds every open item as a row, in the tiers of
`docs/decisions.md` §230.1 and §307. **Every free slot takes the highest ready row that
fits** (§230.3), and a slot is refilled when its own lane finishes, not when a
batch does (§230.4). The cap is four slots, with read-only rows, verifiers and
mergers each counting toward it (§397.7, up from §372's three once lane gates
left out the slow tier; §230.8 had six).

**`just queue` is what to dispatch next, and `just queue next` is what the free
slots take.** Every open item is a row of `docs/queue.md` — id, tier, state,
model, write set, and a line ending in the section behind it — in seven tiers:
1 defects of class A or B or with no class, 2 finding defects better, 2.2 how
the work is done, 2.5 slow runs, 3 corpus files, 4 new functionality, 5 defects
of class C or D (§311 put defects first, §403 the unlikely ones last). A row about the
queue and its verbs, the lanes workflow and its roles, hooks, skills,
`CLAUDE.md`, or a recipe that is neither a test nor a fuzz night is tier 2.2,
not tier 1 or 2 (§307.1); `just queue
tier <id> <tier>` moves one filed elsewhere. The dispatcher writes the file and
a lane never does. `check-queue` holds every row to its shape in the gate.
`docs/decisions.md` §230.

The rules below step 8 are the rest of what only a dispatcher needs: the
recipes the shared skills name, the plan, merging, CI and the Actions
allowance, the plugins, and lookup lanes.

`$ARGUMENTS` names a row to take first. Empty means take what `just queue next`
gives.

## 1. A worktree, and the queue

`EnterWorktree` first; nobody works in the hub. Then, in one call:

```
mise exec -- just status
mise exec -- just queue
```

`status` prints §0, the queue's ready rows, the board, `lanes`, the branches
carrying work `main` does not have, and how far the tip is past the last green.
`queue` adds what is running and what is waiting on which row.

The queue says what is next; §0 says only which unit the corpus rows serve.
Believe the file over any summary of it, including one in a handoff.

`status` is where to start rather than where to end up. Its first row is the
one a recipe cannot compute, so it is written down in `docs/decisions.md`
**§0** — `just decision 0` — and **that section is edited by hand when the work
moves**; `check-where-the-work-is` holds it to its shape but cannot tell you it
is stale, so `status` prints its age instead. `docs/decisions.md` §102.

Do not read the branch row off `just lanes`: `lanes` lists worktrees, and a
worktree is not a branch. Do not derive it from ahead-of-main either (§102.4).

## 2. Do not dispatch into a tree somebody holds

`status` already printed `lanes`. A tree reported `HELD` has an owner named in
its lock reason — message that owner, never send a second agent in. A lock date
does not prove the owner is gone.

**Take `just lanes` whole**: a truncated ask is re-asked more often than a
whole one. §157.3.

## 3. Pin the base, and watch it go green yourself

```
git rev-parse HEAD
mise exec -- just gate quiet
```

One plain Bash call for the gate, `timeout` 600000, no redirect and no pipe
through `tail` — a pipe hands you the pipe's exit code, so a failed gate reads
as 0. Never write a wait loop around it; a foreground call already waits. If the
tree is cold, pass `run_in_background: true` and let the harness wake you.

**Nothing checks that the pin was green but you.** GitHub's Actions allowance is
spent, so a red X there in a few seconds is billing and not a regression, and a
local green is the only green there is.

## 4. Pick the rows

```
mise exec -- just queue next        # or: just queue next N
```

It takes the first ready row of the highest tier whose write set meets no
running `lane:` row and no row taken above it, and names what each skipped row
met. N defaults to the free slots, counted from the trees agents hold on this
machine — lanes, verifiers and mergers alike, whoever sent them (§263.4).
Tiers 1 and 5 go in class order, not file order (§352.3, §403.3).

A line `take Q1,Q2,Q3 (a bundle of 3 small rows, one lane)` is **one
lane** doing all of those rows (§352.4), with each row's text indented under
it. Its model is opus if any of its rows says opus, else sonnet. Its write set
is the union of its rows' sets, leaving out `-`.

Before a brief, check the row's write set against what the work really
touches. A row whose write set is wrong is fixed in the file first: a lane that
needs a file its row does not name either stops or writes outside its set, and
the fill has already let another lane hold that file. A row that adds a
`check-*` recipe or an `#[ignore]` moves a count the gate pins —
`check-gate-runs-every-check`'s `check-*` count, `check-ignored`'s declared
count, `wildcard_arms` — so name the recipe that pins that count in its write
set too, or the merger has to bump it outside the write set on its own
(`.claude/agents/merger.md`).

Model per row, from its `model` field. Opus for anything reasoning about a
numerical refusal, a published bound, or whether a claim of impossibility
holds. Sonnet by name for: replaying a corpus, reproducing and classifying a
fuzz artifact, bisecting, running the gate and reporting, sweeping references
after a renumber, resolving a `decisions.md` conflict down to a marker
deletion, disk and worktree sweeps.

## 5. Dispatch: one `lanes` workflow per row

A row is one run of the `lanes` workflow: its lane, then the lane's verifier,
then a merger that lands it on `main` and hands back about twenty lines. You no
longer merge in your own context, which is where most of it went (§266).

A session reads its agent types once, when it starts. A session begun before its
tree held `.claude/agents/lane.md`, `verifier.md`, `merger.md` and `lander.md`
cannot resolve them, and every stage of the run throws — except the lander's: a
missing `lander` type does not throw the run, it sends the lane to the Opus
merger instead, on its own branch, `merge-<label>-after-land` (§384.3). Start a
new session rather than run it (§266.6).

A run takes the types the session has **when the run is launched**, and the
session can gain one later. On 2026-10-03 the harness told a session begun
without `lander.md` that `lander` was now available. Its run launched before
that note still sent Q796's merge to the Opus merger (`lander` not found,
`c5db0045`), though the merge stage ran after the note; its next run, launched
after it, merged Q801 through the lander (`67109a2e`). So once the note comes,
the next run you launch has the type, and a run already going does not.

For every row the fill took:

1. One `mise exec -- just queue apply take <id1> <branch1> + take <id2>
   <branch2> ...` (§309), commit that line, and push it to `main` — a docs
   commit goes straight there. The merger reads the queue
   from `origin/main`, so a `lane:` it finds there flips to `done:` on one side
   only. A take left on your branch meets that flip at your next merge of
   `main` as a row changed on both sides, and costs a `just queue pick`. A
   bundle is one `take` step per row, every one onto the same branch, in the
   same `apply`.
2. `Workflow` with `scriptPath: ".claude/workflows/lanes.js"`, one call per
   row or bundle, all of a fill's in one message. **Never `name: "lanes"`**: the
   name runs the copy the registry read earlier in the process, and neither a
   merge into your tree nor a `/clear` refreshes it. Q118's run by name got
   `lanes.js` as of Q182, from before Q186, and refused a merge its verifier
   had proven by three reproduced mutations. `scriptPath` runs the file in your
   tree as it is when you call.

The args are `{ base, lanes: [ { label, queueId, brief, writeSet, model,
sourcePaths, pinsOnly, sameBehaviour, report, verify } ] }`. The workflow appends the base,
the `switch -c` and its refusal to the brief itself, and sends each role with
its type, so `.claude/agents/lane.md`, `verifier.md` and `merger.md` carry the
standing rules and a brief carries only what is this row's:

- the work, and anything to report beyond the standard record;
- for a bundle, `queueId: "Q1,Q2,Q3"` as `queue next` printed it, and
  one run for the whole bundle;
- the write set, as `writeSet`. The branch defaults to `lane-<label>` and must
  not exist yet — this repo keeps its finished `lane-*` branches, so check `git
  branch --list 'lane-*'` first; a reused label stops the lane at its second
  command (§237);
- nothing about the context hook for a writing lane. One that sees "Lane
  budget passed" is past 150k of its own context; it commits, hands its branch
  over, and `lanes.js` starts a fresh lane on it, up to three times a row
  (§416). "Carry on past it" would undo that. A read-only lane has nothing to
  hand over and stops at the warning, so say "carry on past it" in a read-only
  brief that builds a probe or reads a listing;
- for a row that writes a section, `mise exec -- just claim-section '<what>'`
  before writing;
- the verbatim API signatures and exact-arithmetic facts that lane needs,
  because rediscovering those costs it twenty minutes;
- for a row that compares two fuzz setups on coverage, how far two identical
  runs drift apart. After one 9 736 s slice, measured with `RCAD_FUZZ_TIMINGS`
  unset, one pair ended 60 edges apart on the reader, 10 on the writer and 0
  to 3 on the other three targets (§272.9). A smaller gap says nothing, and
  one pair is a floor, so ask for several runs a side. One measuring pass per
  corpus is enough: two passes over one corpus agreed to within 2 edges.

`base` is a sha you have seen go green: step 3's, or the sha a merger pushed,
since it gated that tree. `sourcePaths` defaults to the `crates/<c>/src`
directories in the write set; set it by hand when that is not what the verifier
should revert — for a tooling row whose test is a recipe, the file the recipe
checks. A row whose `lines-that-run` print ends `lines that run: 0` — `.md`
files, comments, test code — names no failing test, and its verifier traces
every paragraph it moved instead; a row that adds a test, a `#[cfg(test)]` pin
in `src/` included, still names it. A bundle of such rows merges on that print
with no `pinsOnly` and no `sourcePaths` (§375, §382). A row that only pins a test
and writes a decisions section, with no `src/` path anywhere in its work,
needs `pinsOnly: true`: say so explicitly, never leave it to be inferred from
an empty `sourcePaths`, because that emptiness is also what a tooling row shows
when its dispatcher forgot to set `sourcePaths` by hand, and that must keep
refusing. Give a `pinsOnly` row a `src/` path too — in `writeSet` or
`sourcePaths` — and it is refused instead: there is something to revert then.
A row that changes lines that run and keeps what they compute — one of two
copies dropped for the other (Q854), a body lifted into a function (Q601), a
block moved — needs `sameBehaviour: true`, said explicitly for
`pinsOnly`'s reason: a lane naming no failing test is also what a lane that
changed behaviour and found no test looks like. Its lane names the tests that
reach the change, and its verifier shows they pass with the source reverted
and fail under a mutation of a line the change adds (§399). Never both marks;
the workflow throws at its start.

**Never do a row in your own tree because the workflow cannot merge it.** A
row the workflow has no road for is a missing road in `lanes.js`: file it as
a 2.2 row and dispatch the row once it is built. Q854's dispatcher did the
row itself, which is how §399 came to be written (§399.1).

An empty `writeSet` makes a read-only lane, which is neither verified nor
merged, and its answer comes back whole. A test-only row gets no write access
to `src/` and reports defects instead.

Several rows off one base can share a run, and their merges then wait for one
another. But a slot refills when its run returns (§230.4), so a shared run
holds every slot in it until the slowest row is merged.

**Never `SendMessage` an agent a run is running.** It does not reach the live
agent: it starts a second copy from the transcript, in the same tree and on the
same branch, and the two write over each other (§398). A brief found wrong
mid-run is corrected when the run returns, or the run is stopped with
`TaskStop` and the row sent again.

## 6. When a row's run returns

A merged row comes back as the sha the merger pushed, the gate's PASS line,
the rows it closed and added, and anything it left for you. A row that did not
merge comes back as why, the lane's summary and the verifier's findings, and
the trees to unlock. What the verifier checks is `verifier.md`'s to say, and how
the merger merges is `merger.md`'s; neither is repeated here.

A row whose lane wrote rows is merged by the lander, through `just land`, not
by the Opus merger — its result line reads `merged by just land, which the
lander ran`. A stop sends it to the Opus merger instead. After a stop, the
lander's own tree is unlocked but not marked finished, so it stays on disk
until the merger that follows marks it (§384.3).

- **Merged**: `git fetch origin` and `git rev-parse origin/main` — a reported
  sha can be invented — then merge `origin/main` into your branch, so the copy
  of the queue you fill from sees the row closed (§263.9). The slot is free:
  `mise exec -- just queue next 1`, and back to step 5.
- **Not merged**: decide it before anything else, and write the decision to
  the queue at once, in one `just queue apply` that also files the rows the
  lane left, read from its journal, each `from/<row>` (§432). The three ways
  on: take the row again under a new label with the fix in its brief; park it,
  with the section its reason belongs under; or `just queue drop` it. A row
  filed again after a drop says `was/<old row>`. `git worktree unlock` each
  tree the result named once the apply is pushed.

A lane's tree is unlocked by its verifier when checking starts, the verifier's
by the merger, and the merger's by itself at the end, so a row holds one slot
at a time. A merged row's merger then marks all three finished, and the run
removes them with `prune-worktrees`. The result's `prune:` lines say which
went and why any stayed (§315). A row whose run stopped early leaves its trees
locked, and the result names them.

**Close every returned run out this way before anything else.** Once nothing
is left that `just status`, `just queue` and `just lanes` cannot already show
— no row still reading `lane:` with its trees unlocked and no agent behind
it, no finding that lives only in a run's `journal.jsonl` — the close-out is
one message: tell Mattias to `/clear` in the same `claude` process and run
`/dispatch-lanes`. A run reports to the `claude` process that launched it, not
to the conversation, so a cleared conversation still receives its
completions; a new process loses those notifications outright. A run's
journal, and its args once it completes, stay reachable from the old
conversation's own directory, not the new one:
`~/.claude/projects/<project>/<launching conversation id>/subagents/workflows/<run
id>/journal.jsonl` and `.../workflows/<run id>.json`, `<project>` the tree's
path with each `/` turned to `-`. `docs/decisions.md` §388.

## 7. What is still yours

Whatever the merger lists as for you. Two cases come up:

- **It spent its push attempts,** or its catch-ups over docs alone (§367).
  Other merges reached `main` first each time,
  and its branch `merge-<label>` holds the merged tree. Merge `origin/main` into
  it in a tree of your own, gate, and push, or send the row again.
- **`ff-main` refused.** It refuses on purpose: the tree holding `main` is
  ahead, diverged or dirty, and that tree names no owner. Find out whose work
  it is before moving it.

A merged run removes its own trees. Reclaiming the rest of the disk is yours:
a run that did not merge, a read-only lane (its measurements may be in its
gitignored `target/`, so copy them out first), and any merged tree that a
`prune:` line says was kept. Do it as you go:

```
mise exec -- just clean-targets
mise exec -- just prune-worktrees
```

`prune-worktrees` judges by ancestry, so it keeps a branch that landed by
cherry-pick and every spike that is never merged by design. `git cherry -v main
<branch>` is the content test when you want to know which is which.

## 8. Write it down

```
mise exec -- just claim-section '<what it is about>'
```

Before writing the section, not after — another session takes the number
otherwise.

**The §0 update goes in the same commit as the section that closes it.** "§168.4's
pie slice closed by §169" is only true because §169 exists, so landing it a commit
later guarantees `main` carries a wrong first row for the length of a gate — the
staleness `status` prints an age for, manufactured on purpose. The code commit
landing separately is fine — that comes from a lane — but the section and the
pointer are one act by one session. `docs/decisions.md` §172.5.

## Recipes the shared skills name

`delegating` and `handoff` are written against this repo and say so; these are
the commands they name. `just lanes` lists the worktrees and who holds each.
`just check-hub` is the hub rule. `just clean-targets` then `just
prune-worktrees` reclaims disk, in that order, and `just release-lanes
<pattern>` unlocks a batch of finished lanes in one call. **`just ff-main`
catches the tree holding `main` up to `origin/main`** — it finds that tree by
the branch, so never type its path, and it refuses rather than merges on all
four shapes that are not a fast-forward (§210). **Reach for those three rather
than a sweep of your own.**

## The plan

**`just plan` is the roadmap with a box beside every item, and `just plan open`
is what is left** — the list a unit's lanes are cut from. `docs/roadmap.md`
still may not carry state and still has one commit; `docs/plan.md` holds a
line per item — the roadmap number, a state, the evidence — and the titles are
rendered from the roadmap, so there is one copy of them. `check-plan` derives
the item list from the roadmap on both sides and fails when they differ, and
holds every path a `done` names against the disk. Edit `docs/plan.md` when a
state moves; never add a checkbox to the roadmap. `docs/decisions.md` §106.

## Merging, and the tree that holds `main`

**A green `just gate` on the merged tree is the authorisation to merge.** Not a
sentence from Mattias. Gate the tree you actually merged. `docs/decisions.md`
§138.

That tree is `rcad-worktrees/main`, called `wave-1` in older sections and
memory notes (§210 is the forwarding address). **Do not type either name.**

## CI and the Actions allowance

**CI runs `just gate` — the same command you run**, so every `check-*` recipe is
enforced on the runner too.

There are two arms, and which one runs is decided by path. A push touching only
`docs/` and `*.md` goes to `.github/workflows/ci-docs.yml` and runs `just
gate-docs` — `gate_steps` minus `gate_metered_steps`. Anything touching source
runs the whole gate; a push touching both runs both. Both arms cancel a run
their own branch has already superseded, so **a green can now only be claimed
for a tip**, never for a commit in the middle of a push run.

The runner is metered: `rcad` is private on GitHub Free's 2000 minutes a month.
When a push to `main` shows a red X on a job with **zero steps** that failed in
two seconds, that is the allowance and not a regression — check the annotation
(`gh api /repos/mattiasgronlund/rcad/check-runs/<job>/annotations`) before
looking for one. `docs/decisions.md` §172.

**Three arms are not the same there, and they are the only three.**
`check-plugins`, the plugin arm of `check-hooks`, and `check-questions-selftest`
each ask about something a runner does not have — Claude Code's own config, a
**PreToolUse** hook, and the marketplace repo's tree — so they print that they
skipped and carry on, and say **"skipped, not passed"**. Every other `check-*`
runs on the runner exactly as it runs here, on whichever arm ran: the docs arm
leaves steps out, but it never runs one differently. The skip is in
`check-declared.sh` rather than in the `justfile`, so `rscene` gets the same
one. `check-plugins` keeps its place at the head of `gate`, because a skip is
not a reason to stop asking first. `docs/decisions.md` §77, and the argument in
full at the top of `.github/workflows/ci.yml`'s gate step.

**`just gate-fresh` is the gate with this machine's environment taken away** — a
container, a fresh clone, no `HOME` from here, no plugins, no warm `target/`.
It costs no Actions minutes, so it is the second opinion that survives a spent
allowance; run it before handing work over while the runner is down. It is cold
by construction, so pass `run_in_background: true`. `docs/decisions.md` §77.6,
§173.

**CI is green, and the commit it was green on matters more than the word.**
Check rather than believe — `gh run list --repo mattiasgronlund/rcad --status
success`. **Read that answer as a list and never write the tally down.** A count
goes stale on the next push; a named run does not. Say which commit.

`34266469454`, push of `e8ae455`, is the first whole-gate green on `main`
(§77.5). Every green before it ran `just test`, so check what a run actually
ran — `git show <sha>:.github/workflows/ci.yml` — before treating an older green
as covering the `check-*` recipes.

A green on the runner is not licence to skip `just gate` before handing work
over: the runner only sees what was pushed, and a merge you have not made is not
in it.

## The plugins

Three plugins from the `mattiasgronlund-local` marketplace, shared with
`rscene` (§64):

```
/plugin marketplace add ~/git/mattiasgronlund/claude-questions
/plugin install questions@mattiasgronlund-local
/plugin install guardrails@mattiasgronlund-local
/plugin install practices@mattiasgronlund-local
```

- **`questions`** — the parser, the status line, and the `asking-questions` skill.
- **`guardrails`** — the four hooks: no in-place shell patching, the context
  budget that asks for a handoff, the one that runs the question checker when
  `open-questions.md` is written, and the refusal itself (§152).
- **`practices`** — the nine shared skills, `code-quality` and `delegating`
  among them.

**A missing `guardrails` means the patching hook is not running at all** — the
rule in `CLAUDE.md`'s "Editing files" is then only prose, and it was written
because prose alone did not hold.

The version this repo asks for is the `ref` in `.claude/settings.json`, and
`check-declared.sh` fails a machine registered at a different one. Asking is all
it can do: a committed entry does not register itself, so what governs is the ref
`claude plugin marketplace add` was run with here. A **directory** source is named
rather than failed, because a working tree is the supported way to run a version
no tag ships yet.

The marketplace repo is **public** — the one public thing among these repos, and
deliberately so.

**That it is on GitHub is not a reason to repoint your own marketplace at it.**
`mattiasgronlund-local` is registered as a `directory` source, so it runs in
place out of `~/git/mattiasgronlund/claude-questions` and
`CLAUDE_PLUGINS_ROOT` resolves straight to that checkout. A `github` source
installs to a version-stamped cache path instead
(`~/.claude/plugins/cache/<marketplace>/<plugin>/<version>`), which moves on
every version bump and takes `CLAUDE_PLUGINS_ROOT` with it. Clone it, read it,
push to it — leave the source alone.

What stays here is what is this repo's alone: the hub and worktree-create hooks,
the status line **wrapper** (a plugin cannot supply a `statusLine`; only
`settings.json` can, and it names a path), and
`.claude/patching-cases.local.json` — the patching cases that name `rcad`
crates, `docs/decisions.md`, and a `just` recipe of ours. `just check-hooks`
appends that overlay to the plugin's own cases and fails loudly if it is named
and missing.

## Hunting: lookup lanes

**If you cannot name the file, it is a hunt — hand it to a lookup lane before
the first grep.** Ask the one question you can answer *before* the first probe:
can you name the file, the section or the recipe the answer is in? Then read it
yourself — one call inline beats a lane. Are you about to search for *where*?
Then dispatch `.claude/agents/lookup.md` — read-only, one question, and it dies
with the answer.

All the value is in delegating a hunt at its **first** probe. Never send a lane
for `just lanes`, `just status` or the question file — that is your own state,
and you need the answer in your own hands. `just spend <window> finding` is the
report that measures all of this; `docs/decisions.md` §158 is the argument.
