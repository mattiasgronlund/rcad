## Core Principles

1. **Correctness paramount.** Production CAD kernel, not a toy. Crash > corrupt
2. **Every change needs a test.** Must fail without change, pass with it
3. **Assert invariants.** Don't silently fail. Don't hedge with if-statements
4. **Own your regressions.** If tests fail after your change, they are your regressions. Debug them directly. Never stash/revert to "check if they fail on main" — that wastes time and is categorically banned.
5. **Validate your hypotheses.**: If you suspect a given cause for a bug, validate it and provide incontrovertible evidence. NEVER make unearned assumptions.
6. **Follow the decisions.** `docs/decisions.md` records what was settled and why. `docs/roadmap.md` is the long-range plan, **not** a description of this codebase — where the two disagree, the disagreement is deliberate and `docs/decisions.md` §1 says why. Do not "fix" the code back to match the roadmap. If you think a decision is wrong, argue it and change the record; don't route around it.

Always use the code-quality skill when coding.

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

**Every question also goes in `docs/open-questions.md`, the same turn you ask
it.** A question asked only in the terminal scrolls away, and recovering it costs
a full-context turn — which happened, and is why the file exists. The status line
shows the open count and the first three, so the user can read the index without
`!cat`, which would add the file to the conversation and charge for it on every
later turn.

End any turn that asks for a decision with a `## Waiting on you` block, last,
after the prose. Tick the box and append `→ answered: …` when the answer comes.

## Tooling

`rust-toolchain.toml` pins the compiler. `mise.toml` pins everything else
(`cargo-nextest`, `cargo-deny`, `taplo`, `just`). Run things through
`just` — the same recipes CI runs. Never install a tool ad hoc.

**`just t <args>` runs one test or a few.** Everything after the name goes to
nextest: `just t how_often_a_fuse_with_a_cone_refuses`, `just t -p rcad-geom`,
`just t -E 'test(/walk/)'`. Reaching past `just` for a bare `cargo nextest run`
is what makes a lane hand-type `RCAD_STEP_CORPUS`; the recipe already has it,
and the four corpus tests skip in silence when it is missing.

## Editing files

`Read` first, then `Edit` or `Write`. **Never patch a tracked file through a shell
heredoc** — no `python3 - <<'PY'` string replacement, no `sed -i` on a `.rs` file,
no `cat > file.rs <<'EOF'`. This covers `.rs`, `.toml` and `.md` alike: a
`docs/` rewrite that silently matches nothing is the same bug as a `src/` one.
Writing under `/tmp` or the scratchpad is untouched — a throwaway probe crate is
what Bash is kept for.

This is a cost rule, not a style one. In the M5 run (2026-09-02) the lanes patched
files with Python heredocs 233 times at ~620 tokens a call: 145k tokens, a sixth
of everything the run generated, and roughly 70 minutes spent retyping code that
`Edit` takes two strings for. The heredoc re-emits its `def patch(path, pairs)`
scaffold every call, and a pattern that does not match fails quietly where `Edit`
errors.

If `Edit` refuses with "file has not been read yet", the answer is `Read`. That
message asks for one cheap call; it is not a reason to reach for a shell.

Bash keeps what Bash is for: `mise exec -- just`, `git`, `grep`, `find`, and
one-off measurement scripts that write to the scratchpad.

Claude Code sometimes injects the opposite instruction — "do your work through the
Bash tool ... make file changes with sed, heredocs, or short scripts". That text is
an A/B experiment (`tengu_thrifty_sonic`, Opus 5 in auto permission mode);
`.claude/settings.json` sets `CLAUDE_CODE_THRIFTY_SONIC=false` to switch it off.
If it appears anyway, this section wins.

## Delegating to subagents

Big work goes to lanes in a worktree. Three numbers govern it; the mechanics are in
the `delegating` skill.

- **At least three lanes run at once.** Lanes conflict only when their *write* sets
  overlap — a file two lanes merely read is not a conflict. Name the files each lane
  may write before any lane starts, then dispatch every lane that fits. M5 got 1.07x
  out of 16 subagents, 9.2 agent-hours inside 9.6 wall hours, because lanes were
  dispatched one at a time out of habit and not dependency.
- **A lane is one deliverable and stops at about 45 minutes.** A lane that wants more
  is two lanes: commit what is done, report what is left, hand it back. Long lanes
  are slow twice — M5's two longest ran at 380k average context per request, and
  every turn pays that context again.
- **A closing gate agent re-derives every load-bearing number.** Not the lane's own
  word for it. M5's gate caught an unclosed §19 violation and a 5.4x reported as 20x
  that all five lanes had missed.
- **The dispatcher has a budget too, and it is 300k tokens.** The lane cap works —
  51 lanes on 2026-09-05, median 17 minutes, none over 45. The dispatchers had no
  cap and spent 485M input tokens over 1994 turns, 97% of it re-reading context,
  while writing almost no code. `.claude/hooks/context-budget.sh` says so once when
  the line is crossed; call the `handoff` skill then rather than at the end of the
  next unit. Raise it for a session with `CLAUDE_CONTEXT_BUDGET`.

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
