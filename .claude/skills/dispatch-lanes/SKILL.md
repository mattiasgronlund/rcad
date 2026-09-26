---
name: dispatch-lanes
description: Fill rcad's free lane slots from `docs/queue.md`, one `Agent` per row, against a base you have watched go green, and close each lane as it finishes. Use when taking on the next work, when a lane finishes and its slot is free, or when a batch of rows off one base is due through the `lanes` workflow.
argument-hint: "A row id to take first, or nothing to take what `just queue next` gives"
---

# Filling the lane slots from the queue

`docs/queue.md` holds every open item as a row, in the five tiers of
`docs/decisions.md` §230.1. **Every free slot takes the highest ready row that
fits** (§230.3), and a slot is refilled when its own lane finishes, not when a
batch does (§230.4). The cap is six slots, verifiers and read-only rows
included (§230.8).

**`just queue` is what to dispatch next, and `just queue next` is what the free
slots take.** Every open item is a row of `docs/queue.md` — id, tier, state,
model, write set, and a line ending in the section behind it — in five tiers:
finding defects better, defects, slow runs, corpus files, new functionality.
The dispatcher writes the file and a lane never does. `check-queue` holds every
row to its shape in the gate. `docs/decisions.md` §230.

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
met. N defaults to the free slots, counted from the `lane:` rows. **A verifier
holds a slot and has no row**, so take one fewer for each verifier running.

Before a brief, check the row's write set against what the work really
touches. A row whose write set is wrong is fixed in the file first: a lane that
needs a file its row does not name either stops or writes outside its set, and
the fill has already let another lane hold that file.

Model per row, from its `model` field. Opus for anything reasoning about a
numerical refusal, a published bound, or whether a claim of impossibility
holds. Sonnet by name for: replaying a corpus, reproducing and classifying a
fuzz artifact, bisecting, running the gate and reporting, sweeping references
after a renumber, resolving a `decisions.md` conflict down to a marker
deletion, disk and worktree sweeps.

## 5. Dispatch: one `Agent` per row

`Agent` with `isolation: "worktree"` and the row's model, one call per row, all
of a fill's rows in one message. Flip each row to `lane:<branch>` in
`docs/queue.md` and commit that before or with the dispatch.

The agent types: `subagent_type: "lane"` for a row's lane
(`.claude/agents/lane.md`), `subagent_type: "verifier"` for its verifier in
step 6 (`.claude/agents/verifier.md`), and `lookup` for a hunt
(`.claude/agents/lookup.md`). Each adds its own prompt to `CLAUDE.md`; none
replaces it.

**The worktree branches from your own `HEAD`**, not from `origin/main`, so
every brief carries:

- the base sha, and as its first two commands `git log --oneline -1` and
  `git switch -c <branch> <base sha>`, with both outputs reported back — and
  "if the switch says the branch already exists, stop and report it as
  blocked". A reset (`checkout -B`, `switch -C`) rewinds a branch an older
  worktree may still hold, and that tree's next commit reverts everything
  after it (§237). **The branch must not exist yet**: this repo keeps its
  finished `lane-*` branches, so check `git branch --list 'lane-*'` first;
- the write set, as "you own these files and no others; stop and report a
  change outside them";
- nothing about the context hook, unless you expect the lane to run long. A
  lane that sees "Lane budget passed" is past 150k of its own context and is
  told to commit and report; say "carry on past it" only in a brief whose work
  needs that much. (Until guardrails v0.3.7 the hook read the dispatcher's
  transcript, so it fired on a lane's first call and every brief had to
  countermand it. It no longer does.);
- for a row that writes a section, `mise exec -- just claim-section '<what>'`
  before writing;
- `mise exec --` on every `cargo` and `just`, `Read` then `Edit`, explicit
  `git add <path>`, commit as it goes, ~45 minutes, and every number with the
  command that produced it;
- the verbatim API signatures and exact-arithmetic facts that lane needs,
  because rediscovering those costs it twenty minutes.

A test-only row gets no write access to `src/` and reports defects instead.

**The `lanes` workflow is for a real batch**: several rows off one base, all
ready at once, with a dispatcher who means to wait for all of them. `Workflow`
with `name: "lanes"` (or `scriptPath: ".claude/workflows/lanes.js"` if the
registry answers *not found*); its `args` are `{ base, lanes: [ { label, brief,
writeSet, model, sourcePaths, report, verify } ] }`, and it appends the shared
rules above to each brief itself, the `switch -c` and its refusal included. A
lane's `branch` defaults to `lane-<label>`, so a label reused from an earlier
batch stops that lane at its second command. An empty `writeSet` makes a
read-only lane. `sourcePaths` defaults to the `crates/<c>/src` directories in the
write set; set it by hand when that is not what the verifier should revert.

## 6. When a lane reports: verify it, then refill the slot

A lane reporting its own result is reporting what it believes, and a lane's
`landed_sha` can be invented — `git rev-parse` its branch.

**Each writing lane gets its own verifier** when it reports, not at the end of
a batch: Sonnet, in its own worktree, reverting
the lane's source directories to the base to show its tests fail without the
change, and reading any exactness claim the lane makes. A read-only row that
wrote only a section needs no verifier; read the section.

A lane is finished when its worktree unlocks, not when it reports. Unlock it
yourself once its report is in — `git worktree unlock <path>` — and fill the
slot it freed: `just queue next 1`.

## 7. Close each lane yourself

Merge in a tree that is yours alone, never the hub and never one where another
session has `main` checked out. In **the same commit** as the merge:

- flip the row to `done:<sha>`, and move it under `## Closed`;
- turn the lane's `remaining` and `out_of_write_set_needs`, and the "what this
  leaves" of any section the merge brings, into rows (§230.7);
- if the merge closes something §0 names, edit §0 too.

`mise exec -- just check-queue` must pass on the file you commit. Then run
`mise exec -- just gate quiet` on the tree you actually merged — `main` moves
while lanes run, so gate the merge, not the branch. **A green gate on that tree
is the authorisation to push**; do not wait to be told. Then
`mise exec -- just ff-main`.

Reclaim the disk as you go:

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
