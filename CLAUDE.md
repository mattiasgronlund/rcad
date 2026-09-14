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
- **`guardrails`** — the two hooks: no in-place shell patching, and the context
  budget that asks for a handoff.
- **`practices`** — the nine shared skills, `code-quality` and `delegating`
  among them.

`just check-plugins` fails, first thing in `just gate`, when one is missing,
and names the command that installs it. **A missing `guardrails` means the
patching hook is not running at all** — the rule in "Editing files" below is
then only prose, and it was written because prose alone did not hold.

CI has no Claude Code to install a plugin into, so it takes the other route
`check-plugins` names: it clones the marketplace to `$RUNNER_TEMP` at a **pinned
commit** and sets `CLAUDE_PLUGINS_ROOT`. Pinned so a push there cannot turn this
repo red without a commit here; bump it in `.github/workflows/ci.yml`
deliberately. That repo is **public** — the one public thing among these repos,
settled deliberately so the clone needs no secret. Nothing checks that your
*installed* plugins match the pin.

**That CI clones from GitHub is not a reason to repoint your marketplace at
it.** `mattiasgronlund-local` is registered as a `directory` source, so it runs
in place out of `~/git/mattiasgronlund/claude-questions` and
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
order. The compile cache is wired through mise's `[env]`
as `RUSTC_WRAPPER = "kache"`, which a non-interactive shell does not inherit —
so every `cargo` and `just` in a lane's brief needs a `mise exec --` prefix.

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

`docs/decisions.md` is 17 000 lines: `just decisions-toc` for the map, then
`just decision 49` or `just decision 49.6`. Never guess a line range.

**Search it with `just decisions-find 'refusal rate'`, not with `grep`.** Grep
answers in line numbers, which is the one address this file does not take, so
every hit then costs a walk back up to the heading above it. `decisions-find`
answers in section numbers, which is what `just decision` wants next.

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

## Asking the user a decision

Ask in prose, numbered, batched — `Q11`, `Q12` — with the recommendation and the
narrowing spelled out. That works: 51 user messages on 2026-09-05, one of which
was a correction. Do not reach for `AskUserQuestion` for a design batch; it caps
at four questions of four options and would split one message into seven.

**Every question also goes in `open-questions.md` in this session's scratchpad,
the same turn you ask it.** A question asked only in the terminal scrolls away,
and recovering it costs a full-context turn — which happened, and is why the file
exists. Write each one the way the `grilling` skill formats a round — a heading
line, the body under it, the recommendation last:

```
- [ ] **Q11** — Short heading, in one line?

  The body, as many paragraphs as it takes. Backticks, `§` refs and file:line
  belong here; this is the argument, not a summary of it.

  - **(a)** the first option, on a line of its own
  - **(b)** the second

  ➡️ *Recommend (a), because …*
```

The checkbox line carries the label and the heading, and everything indented
under it belongs to that question. `- [x]` and `→ answered: …` when the answer
comes.

**Options go on their own lines, lettered from `(a)` with no gaps.** Nothing can
lift out of prose an option that was never marked, and until 2026-09-07 nothing
was: 17 of 227 entries offered options and not one of them wrote the options as
lines, which is why finding what `(a)` meant took reading the paragraph it was
buried in. The heading, the option lines and the `➡️` line are the grammar; the
body between them is free prose that no tool interprets.

**`just question-check <hash>` names every violation, and it is your own file you
run it on.** There is no list of grandfathered sessions: `/tmp` is swept at 30
days, so a list of session ids would dangle within a month, and it would be a
second record beside the thing itself (§60). A dead session's file cannot be
fixed, so nothing judges it; yours can, so run this in the turn you write a
question. `just question <hash>` names every violation it finds under the
questions themselves, so the check and the read are one call.

Nothing is ever dropped: an entry whose label does not parse is listed as `??`
with its line number, and shows in the status line as `??×1`. The `asking-questions`
skill has the full grammar and the shapes that break it.

**The parser and that skill are not in this repo.** They live in the `questions`
plugin, because a copy in each repo is a copy that goes stale — `rscene` carried
one that still dropped the second label of a paired entry months after the fix
landed here. So do the hooks and the shared skills; see "Three plugins" below.

**Read them back with `just question`, never with `!cat`.** The status line shows
the open labels collapsed, after the session's hash — `❓ 766405 K1-K4, Q2,
Q4-Q6` — and that hash is the address. `just question 766405` prints **every open
question in full**; `just question 766405 Q11 Q19` prints those two, answered or
not; `--labels` is one line per entry, `--all` includes the answered, `--json` is
for a program. `!cat` puts the whole file in the conversation and charges for
every line of it on every later turn.

**`just question` is written for you; `just question-tui` is the one Mattias
types.** That split is why the default is whole bodies rather than a listing to
drill into: a person has a finite screen and cheap keystrokes, and you have the
reverse. §100.

**A label is a letter and a number, and the letter carries meaning.** `Q` for a
grilling round or any plain decision — that is what the `grilling` skill's format
already uses. A lane's findings take **the lane's own letter**, which is where
`K1`-`K4` came from: they were Lane K's. Keep it, because "which lane is this
from" is the first thing asked about a finding. The label goes straight after the
checkbox; the status line reads it there and nowhere else, so a body is free to
mention `M5` or `§52.11` without becoming a question.

**One entry may carry two labels when one answer settles both** — `**Q7 / Q11**
— …`, addressable as either. Both are counted and both are shown. Reading only
the first is a bug that has been fixed once already: on 2026-09-06 the line
showed fourteen of sixteen open questions and looked complete (`docs/decisions.md`
§59.2).

**Never in the tree.** Four sessions were live across these two repos on
2026-09-05; a tracked file would have had all four writing to it. Per session
also means the numbering is the session's own, so `Q3` means what it says in the
conversation the user is looking at.

The cost of that is the file dies with the session, so **carry every unanswered
question into the handoff document.** Long argument still belongs in
`docs/decisions.md` under the section it will become; the scratchpad file is what
is outstanding, not why.

End any turn that asks for a decision with a `## Waiting on you` block, last,
after the prose.

## Tooling

`rust-toolchain.toml` pins the compiler. `mise.toml` pins everything else
(`cargo-nextest`, `cargo-deny`, `taplo`, `just`). Run things through
`just`. Never install a tool ad hoc.

**CI runs `just gate` — the same command you run.** One gate step since
2026-09-07, so every `check-*` recipe is enforced on the runner too. Before that
it ran four recipes and the `check-*` ones gated nothing but your own machine.

**One arm is not the same there, and it is the only one.** `check-plugins` asks
Claude Code whether the plugins are *declared* and not merely on disk; a runner
has no config directory for that to be a question about, so it prints that it
skipped and carries on. Everything else — the three directories, the parser
selftest, every other `check-*` — runs on the runner exactly as it runs here.
The skip is in `check-declared.sh` rather than in the `justfile`, so `rscene`
gets the same one; `docs/decisions.md` §77.

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
