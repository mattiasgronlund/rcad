# What dispatch-lanes does not need to fill a slot

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

A run takes the types the session has **when the run is launched**, and the
session can gain one later. On 2026-10-03 the harness told a session begun
without `lander.md` that `lander` was now available. Its run launched before
that note still sent Q796's merge to the Opus merger (`lander` not found,
`c5db0045`), though the merge stage ran after the note; its next run, launched
after it, merged Q801 through the lander (`67109a2e`). So once the note comes,
the next run you launch has the type, and a run already going does not.
