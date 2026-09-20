## Core Principles

1. **Correctness paramount.** Production CAD kernel, not a toy. Crash > corrupt
2. **Every change needs a test.** Must fail without change, pass with it
3. **Assert invariants.** Don't silently fail. Don't hedge with if-statements
4. **Own your regressions.** If tests fail after your change, they are your regressions. Debug them directly. Never stash/revert to "check if they fail on main" — that wastes time and is categorically banned.
5. **Validate your hypotheses.**: If you suspect a given cause for a bug, validate it and provide incontrovertible evidence. NEVER make unearned assumptions.
6. **Follow the decisions.** `docs/decisions.md` records what was settled and why. `docs/roadmap.md` is the long-range plan, **not** a description of this codebase — where the two disagree, the disagreement is deliberate and `docs/decisions.md` §1 says why. Do not "fix" the code back to match the roadmap. If you think a decision is wrong, argue it and change the record; don't route around it.

Always use the code-quality skill when coding. Big work goes to lanes in a
worktree — the `delegating` skill has the mechanics, and only whoever is
dispatching needs them.

## Three plugins, and what this repo keeps

`.claude/` used to hold the hooks, the shared skills and a copy of the questions
parser. All three moved into the `mattiasgronlund-local` marketplace, shared with
`rscene`, because a copy in each repo is a copy that goes stale — and one did:
`rscene`'s parser still dropped the second label of a paired entry months after
the fix landed here.

```
/plugin marketplace add ~/git/mattiasgronlund/claude-questions
/plugin install questions@mattiasgronlund-local
/plugin install guardrails@mattiasgronlund-local
/plugin install practices@mattiasgronlund-local
```

- **`questions`** — the parser, the status line, and the `asking-questions` skill.
- **`guardrails`** — the four hooks: no in-place shell patching, the context
  budget that asks for a handoff, the one that runs the question checker when
  `open-questions.md` is written — **which is why `just question-check` is
  refused; the write already ran it, and a clean file now says so in one line**
  (§152) — and the refusal itself.
- **`practices`** — the nine shared skills, `code-quality` and `delegating`
  among them.

`just check-plugins` fails, first thing in `just gate`, when one is missing,
and names the command that installs it. **A missing `guardrails` means the
patching hook is not running at all** — the rule in "Editing files" below is
then only prose, and it was written because prose alone did not hold.

CI has no Claude Code to install a plugin into, and since 2026-09-13 it does not
clone the marketplace either. Each of the clone's three users turned out to be
checking something a runner does not have, so those three arms skip there and say
**"skipped, not passed"**: `check-plugins`, which asks whether Claude Code has
been *told about* the plugins; the plugin arm of `check-hooks`, whose subject is a
**PreToolUse** hook that fires in a session and never in a job; and
`check-questions-selftest`, which ran the marketplace repo's own cases out of the
clone and so tested that repo's tree rather than this one's — that repo runs them
itself on every push now. `check-plugins` keeps its place at the head of `gate`,
because a skip is not a reason to stop asking first.
`.github/workflows/ci.yml` carries the whole argument, and `docs/decisions.md` §77.

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

**A bare `just` lists the recipes, one line each** — `just` runs the first
recipe in the file, and that is `list`. Every listed recipe carries a real
sentence, so start there rather than reading the `justfile`.

**Recipes the shared skills refer to.** `delegating` and `handoff` are written
against this repo and say so; these are the commands they name. `just lanes`
lists the worktrees and who holds each. **`just gate` is the whole gate**, and
what CI runs — it was called `just default` until 2026-09-08, so an older
document or a session's memory may still say that. `just check-hub` is the hub
rule. `just clean-targets` then `just prune-worktrees` reclaims disk, in that
order, and `just release-lanes <pattern>` unlocks a batch of finished lanes in
one call. **Reach for those three rather than a sweep of your own** — and they
are reached for now: the hand-rolled sweep this paragraph was written against is
under a fifth of what lane bookkeeping costs, against nearly all of it a
fortnight ago.

What replaced it is asking, not doing. Lane and worktree admin is the second
largest ritual, and over half of it is 262 separate asks of "who holds what"
across 86 sessions, more than three quarters of which ask twice or more. That
part is mostly irreducible — other sessions really do lock and unlock while you
work — and §157.3 says why no recipe is proposed for it. **Take `just lanes`
whole**: 58% of those asks cut it through `head` or `tail`, and a truncated ask
is re-asked more often than a whole one. It is one screen for fourteen
worktrees. The compile cache is wired through mise's `[env]`
as `RUSTC_WRAPPER = "kache"`, which a non-interactive shell does not inherit —
so every `cargo` and `just` in a lane's brief needs a `mise exec --` prefix.

**`just status` answers "where does this repo stand" in one call**, and it is
where to start rather than where to end up: what unit the work is in, who holds
which lane, which **branches** carry work `main` does not have, how far the tip
is past the last green, and how far `decisions.md` has got. Assembling that by
hand took twelve tool calls and a lookup lane on 2026-09-14, and every
ingredient was already in the justfile. The first row is the one a recipe
cannot compute, so it is written down in `docs/decisions.md` **§0** — `just
decision 0` — and **that section is edited by hand when the work moves**;
`check-where-the-work-is` holds it to its shape but cannot tell you it is
stale, so `status` prints its age instead. `docs/decisions.md` §102.

**The §0 update goes in the same commit as the section that closes it.** "§168.4's
pie slice closed by §169" is only true because §169 exists, so landing it a commit
later guarantees `main` carries a wrong first row for the length of a gate — the
staleness `status` prints an age for, manufactured on purpose. It was also half
the runner bill: 5 of 6 §0 commits sat directly beside the section they announced,
and 27 of 44 sections cost two commits or more. The code commit landing separately
is fine — that comes from a lane — but the section and the pointer are one act by
one session. `docs/decisions.md` §172.5.

Do not read the branch row off `just lanes`: `lanes` lists worktrees, and a
worktree is not a branch. Do not derive it from ahead-of-main either — of the
five branches that count called unmerged on 2026-09-14, three carried nothing,
their commits having reached `main` by a cherry-pick or a split (§102.4).

**`just plan` is the roadmap with a box beside every item, and `just plan open`
is what is left** — the list a unit's lanes are cut from. `docs/roadmap.md`
still may not carry state and still has one commit; `docs/plan.md` holds a
line per item — the roadmap number, a state, the evidence — and the titles are
rendered from the roadmap, so there is one copy of them. `check-plan` derives
the item list from the roadmap on both sides and fails when they differ, and
holds every path a `done` names against the disk. Edit `docs/plan.md` when a
state moves; never add a checkbox to the roadmap. `docs/decisions.md` §106.

## Nobody works in the hub

`/home/mattias/git/mattiasgronlund/rcad` is the **hub**. It owns `.git` and
nothing else happens there. Every session gets a worktree of its own — call
`EnterWorktree` first thing — and `main` lives checked out in a worktree too,
never in the hub.

This is not tidiness. `git worktree lock` is this repo's record of who owns a
tree (§55), and it answers *"fatal: The main working tree cannot be locked or
unlocked"* for the hub, with no flag to override. So the hub is the one tree
whose owner cannot be written down, and it is the tree every session lands in by
default. The rule deletes that case instead of inventing a second record beside
the lock: a tree nobody works in needs no owner, and `just lanes` printing
nothing beside it becomes true rather than a guess.

`main` leaving the hub is the half that does the work. Git will not let a lane
move a branch that is checked out elsewhere, so a hub holding `main` drags every
merge back into the unlockable tree however carefully the lanes were dispatched.
Parked on a detached HEAD, `main` is checkable out — and lockable — in a
worktree like any lane.

`just check-hub` fails when the hub has a branch checked out or work left in it,
and it is in `just gate`. A repository with one worktree is exempt, because a
lone checkout is not a hub. `docs/decisions.md` §60.

**A hub nobody works in goes stale, and `check-hub` cannot see it.** Its files
match its own old commit, so `status` is empty and the check says "parked and
clean" — while `just decision 144.5` typed there answers *"no section 144.5"*,
which is true of that checkout and false of this repo. Since 2026-09-19 any
`just` command run in the hub re-parks it on `main` first: `just` evaluates
top-level assignments before every recipe, and `hub_aligned` spends three git
calls anywhere else. It refuses to move a hub holding a branch, or anything not
already on `main` byte for byte, because the re-park runs `clean -fd`. It aligns
to local `main` and never fetches — a network round trip on every `just t` is
§144's cost in another currency. `docs/decisions.md` §146.

`docs/decisions.md` is 27 000 lines: `just decisions-toc` for the map, then
`just decision 49` or `just decision 49.6`. Never guess a line range.

**Search it with `just decisions-find 'refusal rate'`, not with `grep`.** Grep
answers in line numbers, which is the one address this file does not take, so
every hit then costs a walk back up to the heading above it. `decisions-find`
answers in section numbers, which is what `just decision` wants next.

**Take a new section's number with `just claim-section 'what it is about'`
before writing it,** not by checking `git log --all -S` for a free one. The
check was right every time and lost anyway — another session took the number
during the hour it took to write the section. The recipe writes the claim down
under a lock, in a file every worktree shares, so the number is yours from the
moment it is printed. `docs/decisions.md` §159.

## Rules that come from the design

These are the ones that are expensive to notice late.

- **A predicate returns `Decision`, never a bare sign.** `Undecidable` is a
  result, not an error. Every call site matches all four cases; that is the point.
- **`Undecidable` at the operation level is a failure** (`Error::Undecidable`).
  At the predicate level it is not. Keep the two apart.
- **Panic on broken internal invariants. Return `Err` on bad caller input.**
  A corrupt solid is worse than a crash; a bad parameter is data, not a bug.
- **Never expose `&FaceData` or any other raw arena payload.** Public access goes
  through `FaceView`/`EdgeView`/`VertexView`, which carry the composed location.
  Handing out raw payload hands out un-located geometry.
- **Strictness travels as an explicit `&Policy` argument.** Never a field on
  `Model`, never thread-local. Model state that silently changes results is the
  footgun this kernel exists to avoid.
- **Provenance is recorded per decision, not per coordinate.** Detailed records
  only on non-certified outcomes; a fully certified operation logs nothing.
- **`#![forbid(unsafe_code)]`** in every crate here.
- **`#[non_exhaustive]` on every public enum.** They all grow each milestone. The
  cost of that is a wildcard arm at every match outside the owning crate, and a
  wildcard is where the next variant goes to be ignored — so when you write one,
  name the variants that exist and keep `_` for the ones that do not. An enum
  that is deliberately exhaustive says **"Exhaustive on purpose"** in its doc
  comment; `just check-enums` accepts nothing else. `docs/decisions.md` §40.
- **No LGPL runtime dependencies.** `cargo-deny` enforces it; don't work around it.
- Validity checks after a mutation are **local to touched entities**. The full
  sweep is `Model::validate()`, called at test and import boundaries. A full sweep
  per operation makes the test suite quadratic.
- **A part with no named consumer is not built.** Defer it to the milestone where
  the consumer appears. `docs/decisions.md` §13 says why. This governs **features,
  never correctness gaps** — see the next rule.
- **A claim that something cannot be done, or cannot be reached, needs a
  demonstration and not a paragraph.** Same rule as a published bound needing a
  test (§15.6), applied to two more kinds of claim. "No ordering of the arithmetic
  helps" was true of orderings and silent about scalings, and a representable
  answer was refused for it. Before writing such a sentence, name the family of
  fixes you considered and say why that family is the only one; for "no caller
  hits this", check the **public API**, not today's callers. A refusal is a
  *limit* when the answer does not exist and a *gap* when it does and the
  arithmetic could not reach it — defer a limit, fix a gap. `docs/decisions.md`
  §16.10.

## Decide it yourself, and land it

**A green `just gate` on the merged tree is the authorisation to merge.** Not a
sentence from Mattias — he has never blocked one, and a merge that turns out
wrong is a revert on a branch that is still on disk. Waiting costs a round trip
that lands at the end of a session with the prompt cache cold, which is the most
expensive turn there is. Gate the tree you actually merged, and never write down
a gate you have not read.

**Where you would have asked a question carrying a recommendation, take the
recommendation.** Say which way you went and why, and carry on. Ask only where a
wrong answer is expensive to undo, or where what Mattias wants is not derivable
from the record. A decision taken alone is still written down — the commit
message for a small one, `docs/decisions.md` for one with an argument behind it,
the handoff for one still running.

**End finished, or end written down.** Run the work to complete, or write it
into the handoff as postponed with what is done and what remains. Do not park a
session on a question. `docs/decisions.md` §138.

Still not yours to take: an action another session was **refused** permission
for. Being the hands for someone else's refusal is not a decision, at any level
of trust.

### The questions that are still questions

**Every question goes in `open-questions.md` in this session's scratchpad, the
same turn you ask it in the terminal.** A question asked only in the terminal
scrolls away, and recovering it costs a full-context turn — which happened, and
is why the file exists.

**Read it back with `just question <hash>`, never with `!cat`.** The status line
carries the hash and the open labels — `❓ 766405 K1-K4, Q2, Q4-Q6`. `just
question` prints whole bodies because it is written for you; `just question-tui`
is the one Mattias types. `docs/decisions.md` §100.

End any turn that asks for a decision with a `## Waiting on you` block, last,
after the prose.

The grammar, how a round is written, what breaks the parser and how to read a
file back cheaply are the **`asking-questions` skill's**, in the `questions`
plugin. They are not repeated here: a copy in each repo is a copy that goes
stale, and `rscene` carried one that still dropped the second label of a paired
entry months after the fix landed.

## Tooling

`rust-toolchain.toml` pins the compiler. `mise.toml` pins everything else
(`cargo-nextest`, `cargo-deny`, `taplo`, `just`). Run things through
`just`. Never install a tool ad hoc.

**CI runs `just gate` — the same command you run.** One gate step since
2026-09-07, so every `check-*` recipe is enforced on the runner too. Before that
it ran four recipes and the `check-*` ones gated nothing but your own machine.

**Since 2026-09-20 there are two arms, and which one runs is decided by path.**
A push touching only `docs/` and `*.md` goes to `.github/workflows/ci-docs.yml`
and runs `just gate-docs` — `gate_steps` minus `gate_metered_steps`, 1m07s
against 9m46s. Anything touching source runs the whole gate as before; a push
touching both runs both. Both arms cancel a run their own branch has already
superseded, so **a green can now only be claimed for a tip**, never for a commit
in the middle of a push run.

This is because the runner is metered and **it ran out**: `rcad` is private on
GitHub Free's 2000 minutes a month, and 171 runs spent 2271 of them in
seventeen days. When a push to `main` shows a red X on a job with **zero steps**
that failed in two seconds, that is the allowance and not a regression — check
the annotation (`gh api /repos/mattiasgronlund/rcad/check-runs/<job>/annotations`)
before looking for one. `docs/decisions.md` §172.

**`just gate` is still what you run locally, and `gate-docs` is not a shortcut
for it.** The steps it drops are the ones that cost money, not the ones that are
unlikely to fail.

**`just gate-fresh` is the gate with this machine's environment taken away** — a
container, a fresh clone, no `HOME` from here, no plugins, no warm `target/`.
That is §77.6's manoeuvre, which found all three of the red runs of 2026-09, as
a recipe rather than something retyped each time. It costs no Actions minutes,
so it is the second opinion that survives a spent allowance; run it before
handing work over while the runner is down. It is cold by construction, so pass
`run_in_background: true`.

**On its first working run it found a fourth.** The container's `awk` is mawk,
not gawk, and the composite half of `check-wildcards` was opening its matches on
a GNU-only word boundary — so it read every file and counted 0 of 4, silently,
here green and there red on the same commit. `docs/decisions.md` §173. A check
that answers a property of the environment rather than of the tree is the class,
and taking the environment away is still the only way anyone has found one.

**Run it as `just gate quiet`, in one plain Bash call, with `timeout` set to
600000 and no redirect.** `quiet` prints one line per step, the failing step's
own last sixty lines, and the summary block — about fifty lines in all — and
sends the rest to `target/last-gate.log`. That is the whole answer, in the tool
result, for one call. Bare `just gate` stays the terminal mode and CI's: it
streams every line, which is 300 KB, and 300 KB is 75 000 tokens you cannot put
in a result. **Do not redirect `quiet` to a file**; redirecting is what made the
summary block useless. In the week of 2026-09-12, 262 of 272 gate runs sent
stdout to a file — correctly, for 300 KB — and 122 of them then paid a second
whole-context call to read back the eight lines the recipe had already printed.

**Never pipe it through `tail` or `head`**: the exit code you get back is the
pipe's last command's, so a failed gate reads as 0.

**Never write a wait loop around it** — no `until ! pgrep -f 'just gate'`, no
`sleep 240; tail`. A foreground call already waits, and the measurement says it
fits: over 56 passing runs that week the median was 98 seconds and the slowest
244, and not one of 171 foreground runs hit its timeout. Foreground `sleep` is
**not** blocked in this harness, so this paragraph is the only thing stopping
you. The one case that can reach the nine minutes the recipe's `[doc]` names is
a cold tree — a fresh worktree, or just after `just clean-targets`. There, pass
`run_in_background: true` and let the harness wake you when it exits; that costs
two calls instead of one, which is still fewer than polling. `docs/decisions.md`
§147.

**Three arms are not the same there, and they are the only three.**
`check-plugins`, the plugin arm of `check-hooks`, and `check-questions-selftest`
each ask about something a runner does not have — Claude Code's own config, a
**PreToolUse** hook, and the marketplace repo's tree — so they print that they
skipped and carry on. Every other `check-*` runs on the runner exactly as it runs
here, on whichever arm ran: the docs arm leaves steps out, but it never runs one
differently. The skip is in `check-declared.sh` rather than in the `justfile`, so
`rscene` gets the same one; `docs/decisions.md` §77, and the argument in full at
the top of `.github/workflows/ci.yml`'s gate step.

It was one arm until 2026-09-13, when the marketplace clone came out of CI and
the other two lost the directory they had been reading — so an older document
saying "one arm, and it is the only one" is describing the job before that.

**`34266469454` is the first whole-gate green on `main`**, push of `e8ae455`
(the recipe was still called `default` when it ran). Every green before it ran `just test`, so check what a run actually
ran — `git show <sha>:.github/workflows/ci.yml` — before treating an older green
as covering the `check-*` recipes.

**Three red runs came first, and only the first was about plugins.** The other
two counted colour codes instead of clippy warnings: `ci.yml` sets
`CARGO_TERM_COLOR: always` and `check-wildcards` anchored its grep at `^`. All
three are the same shape — a check passing for a property of the environment it
ran in rather than of the tree — and the way to find one is to take away what
the environment supplies and run it again. `docs/decisions.md` §77.6.

**CI is green, and the commit it was green on matters more than the word.**
Check rather than believe — `gh run list --repo mattiasgronlund/rcad --status
success` — because the empty answer that command used to give is what this
paragraph said before.

**Read that answer as a list and never write the tally down here.** A count goes
stale on the next push, which is exactly why the sentence this replaced said
"never succeeded" rather than "26 of 26" — and the count that stood beside it
*was* stale within a day, twice. A named run does not go stale. Three to know,
all pushes to `main`, each reporting the same three numbers as `just gate` on
the same tree: `34155345448` at 1787 tests, `34255349804` at 1806 after the fuse
fuzz work landed, and `34266469454` at 1811 — the last of those the first run
that put the whole gate on a runner rather than `just test`.

The other green is a pull request build that contained almost none of the work
queued behind it. For a day "CI is green" and "the work is covered" were both
true statements about this repo and both useless without a sha. Say which
commit.

Three faults kept every earlier run red — the tool install, a rustup race, and
the missing STEP corpus, cleared in that order, the last by `corpus/fetch.sh`.
`.github/workflows/ci.yml` carries all three and the evidence for each fix.

So there is a second opinion now, and it agreed the first time it could. That is
not licence to skip `just gate` before handing work over: the runner only
sees what was pushed, and a merge you have not made is not in it.

**`just t <args>` runs one test or a few.** Everything after the name goes to
nextest: `just t how_often_a_fuse_with_a_cone_refuses`, `just t -p rcad-geom`,
`just t -E 'test(/walk/)'`. Reaching past `just` for a bare `cargo nextest run`
is what makes a lane hand-type `RCAD_STEP_CORPUS`; the recipe already has it.
Seven test files in `rcad-step` read that variable and **fail** rather than skip
when the corpus is missing (§44.10), so the bare command costs you seven
failures that `just` would not have shown you.

**`just src <name>` prints one item of Rust source**, with its doc comment and
attributes, from wherever it is defined: `just src Decision`, or `just src new
crates/rcad-geom` when the name has 43 definitions. A name defined more than
once lists the places rather than printing all of them. Reach for it instead of
grepping for a definition and then guessing a `Read` offset and limit — in
`arrange.rs`, 330 509 characters, the mean item comes back in 1 961.

**`just recipe <name>` does the same for this justfile** — and never grep it
either. `just --show` exists and is not the thing: it prints **one line** of the
comment block, because that is just's own doc-comment rule, while the argument
above a recipe here runs to paragraphs, and `just --show gate_steps` answers
`error: Justfile does not contain recipe` — the `:=` assignments are reachable
by no other reader at all. A name that matches nothing fails; one that is not
exact lists the names containing it, or prints the one it is contained in; **no
name at all is the index**, the only listing that shows the `[private]`
selftests. The justfile is 307 404 characters over 105 definitions and the mean
answer is 2 934 of them — a hundredth of the file, in one call.
`docs/decisions.md` §153.

**If you cannot name the file, it is a hunt — hand it to a lookup lane before
the first grep.** Finding things is about a third of a week's bill, and most of
that third is nobody's to delegate: probes made inside lanes, which are cheap
already, and the ritual of asking about this session's own state. What is left
is a sixth. Ask the one question you can answer *before* the first probe: can
you name the file, the section or the recipe the answer is in? Then read it
yourself — one call inline beats a lane. Are you about to search for *where*?
Then dispatch `.claude/agents/lookup.md` — read-only, one question, and it dies
with the answer.

**"Past two probes" was the rule here until 2026-09-19, and it is measured to
lose money.** Dispatching costs $0.13, the lane itself $0.25, carrying its
answer a few cents more — against $0.10 for a probe you make yourself and $0.38
for the median hunt. By the time a hunt is two probes old, what is left of it is
worth less than the lane, so a trigger that fires mid-hunt is negative at every
threshold. All the value is in delegating a hunt at its **first** probe, which
is why the rule now asks something you can answer there. Never send a lane for
`just lanes`, `just status` or the question file — that is your own state, and
you need the answer in your own hands — and never dispatch one from inside a
lane. `just spend <window> finding` is the report that measures all of this;
`docs/decisions.md` §158 is the argument.

**A call is billed the context it is made at, not what it prints.** That is the
one rule the lookup lane, the batching rule under "Editing files" and `just
question`'s whole-file default all come out of, and `docs/decisions.md` §100 has
the arithmetic: a round trip you did not have to take costs about what carrying
2 200 extra resident tokens for 73 more turns costs. So print the whole thing
and read it once.

## Editing files

`Read` first, then `Edit` or `Write`. **Never patch a tracked file through a shell
heredoc** — no `python3 - <<'PY'` string replacement, no `sed -i` on a `.rs` file,
no `cat > file.rs <<'EOF'`. This covers `.rs`, `.toml` and `.md` alike: a
`docs/` rewrite that silently matches nothing is the same bug as a `src/` one.
Writing under `/tmp` or the scratchpad is untouched — a throwaway probe crate is
what Bash is kept for.

This is a cost rule, not a style one: in M5 it cost 145k tokens and ~70 minutes of
retyping, and a heredoc whose pattern does not match fails quietly where `Edit`
errors. `docs/decisions.md` §50.8 has the arithmetic and a hook enforces the rule.

If `Edit` refuses with "file has not been read yet", the answer is `Read`. That
message asks for one cheap call; it is not a reason to reach for a shell.

Bash keeps what Bash is for: `mise exec -- just`, `git`, `grep`, `find`, and
one-off measurement scripts that write to the scratchpad.

**Independent read-only probes go in one call.** A call bills the whole
conversation again — a median of 129 000 tokens here, and past 300 000 late in a
session — so two greps in two calls cost twice what the same two greps cost in
one, and the size of what they print barely matters. Batch whatever does not
depend on an answer you have not got yet; sequence only what does.

**Most of the gap this rule used to quote is not batchable, so do not expect
it.** The pairs it counted are mostly *dependent*: grep for a line, then read at
that offset; count the lines, then read the tail. The second call needs the
first one's answer and cannot be issued beside it. Two weeks of asking moved the
share of responses calling more than one tool by about a point, to 12%, which is
nothing — the paragraph was aimed at a population it could not move. The
dependent pairs are a **tool** problem instead: one call where the file has a
one-call reader — `just decision`, `just src`, `just recipe` — and two where it
does not. §144.7, §157.2.

Claude Code sometimes injects the opposite instruction — "do your work through the
Bash tool ... make file changes with sed, heredocs, or short scripts". That text is
an A/B experiment (`tengu_thrifty_sonic`, Opus 5 in auto permission mode);
`.claude/settings.json` sets `CLAUDE_CODE_THRIFTY_SONIC=false` to switch it off.
If it appears anyway, this section wins.

## Always use plain language instead of complex jargon

OOGA BOOGA! Programming already complex! Use simple word! Say what you mean! Examples:

```diff
-    /// Cardinality of the residual set outside the certified enclosure domain.
+    /// How many sample points we could not decide were inside or outside.

...

-    fn cascade_escalates_on_non_conforming_input_manifold() {
+    fn three_planes_meeting_at_a_point_cannot_be_decided_in_f64() {
```

No-one knows what the hell a non-conforming input manifold is. Everyone knows what "three planes meeting at a point" is.

## Code flows from top to bottom

A reader should be able to read a file from the top down without jumping
ahead to find what a name means. The general rule: place a function after
all of its call sites. Callers come first, callees follow.

```text
pub fn commit()         // entry point
fn write_frames()       // called by commit
fn sync_wal()           // called by write_frames
```

When adding a helper, put it below the functions that call it, not at the
end of the file or wherever the cursor happened to be.

## Use comments only when absolutely necessary

Comments should explain *why*, not *what*. Do not narrate what the code
already says: no `// Check if the page is dirty` above `if page.is_dirty()`,
no `// Helper function for X`, no `// Step 1:`. If the code needs a comment
to be understood, rename the function or variable instead. A comment earns
its place when it says something the code cannot: an invariant the type
system does not enforce, a workaround for a specific bug with a reference.
