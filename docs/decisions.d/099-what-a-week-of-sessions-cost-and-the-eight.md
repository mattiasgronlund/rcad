## 99. What a week of sessions cost, and the eight things that were paying for nothing, 2026-09-14

Every session transcript in this repo and its worktrees with activity between
2026-09-07 and 2026-09-14 was read and priced: 13 573 API calls, 1 805 M billed
input tokens, 6 M output. At list rates — Opus 5 at $5/$25 per MTok, cache reads
at $0.50 — that is **$1 163 for the week**.

The method matters, because the naive reading is wrong by 1.85x. A transcript
writes one JSONL entry per content block and **repeats the same `usage` object
on each of them**, so a thinking block, a text block and three tool calls from
one API response appear as five entries all claiming the same token counts.
Dedupe by `message.id` before summing anything.

### 99.1 It is round-trips that cost, not output

**72.5% of the bill is cache reads**: the same context read again on every turn.
Raw tool output for the entire week was 4.3 M tokens. The bill is **420x the
information**. A `grep` that returns forty bytes at 250 000 tokens of context
costs a quarter of a million tokens, and would have cost the same returning
nothing.

Everything below follows from that one ratio. The lever is never "print less";
it is "take fewer turns", or "take them somewhere the context is small".

Three numbers set the scale. The median call bills 117 000 tokens and the mean
133 000. **Calls above 150 000 are 34% of calls and 55% of the bill**; above
200 000, 19% of calls and 35%; above 250 000, 7% and 16%. And **only 11% of
calls issue more than one tool call**, so the commonest shape in the corpus is a
quarter-million-token call that asks one question.

### 99.2 Where it went

By what the call was doing, as a share of the $1 163:

- finding things — `grep`/`sed`/`cat`/`find` $228, `Read` $62, `git log`/`show`/
  `diff` $90, `just decision*` $30, `just src` $22 — **$481, 41%**
- `Edit` of source, $134, at a mean context of 142 000
- answering the user with no tool at all, $70
- running one test or the suite, $53; `gate` runs, $39, plus a further 145 calls
  that grepped the redirected gate log for three numbers the recipe already had
- `docs/decisions.md` reads and edits, $48
- the `open-questions.md` ritual, $53 — 263 `Edit`s against 57 `Write`s, and 128
  `question-check` calls whose **median result was 31 bytes**, because the
  checker prints nothing when the file is clean
- lane, worktree and disk bookkeeping, $38

By where the work ran: main sessions on Opus, 66%, at a mean context of 166 000.
Subagents on Opus, 26%, at 97 000. Subagents on Sonnet, 3%. Six Haiku calls all
week.

### 99.3 The refusals that cost a turn each

333 calls were refused by the worktree-isolation guard, which rejects a `git`
command it cannot prove stays inside the worktree — a `~/git/...` path inside a
loop or a `$( )` is the usual shape. Each refusal buys a retry at full context:
**~40 M tokens, 1.2% of the week**, for commands that were never run. A further
37 were refused by the permission classifier and 16 by the patching hook.

The patching hook's 16 are the system working: that is 16 heredocs that did not
silently fail to match. The guard's 333 are not, and the fix is in how commands
are written rather than in the guard.

### 99.4 The eight changes

Approved as a batch by Mattias on 2026-09-14, each with the measurement that
argues it:

1. **`just fuzz-triage`** — the night already keeps its artifacts and writes
   `latest`; nothing said what the bytes *meant*, so every morning re-derived it
   by hand through `cargo +nightly fuzz run`. The fuzz worktrees were 30% of the
   week. The recipe decodes an artifact through the target's own harness and
   prints the reconstruction line a regression test needs — which is what the
   20-to-40 `Edit` round-trips at the tail of a bug session were spent writing.
2. **A `lookup` lane** (`.claude/agents/lookup.md`), read-only and on Sonnet, for
   any question needing more than two probes. Against $481 of finding things at
   150 000 tokens a call, a lane that starts at 34 000 and bills at a fifth the
   rate costs about a tenth for the same answer.
3. **Batch independent read-only probes into one call.** 1 655 read-only calls
   directly followed another: 9% of the bill spent on the split alone.
4. **`gate` ends with its own summary**, so the 145 follow-up greps become none.
5. **One `Write` per round of questions**, and a hook that runs the checker on
   write and speaks only when it finds something.
6. **`just release-lanes`**, and disk work through `clean-targets`. One session
   unlocked fifteen worktrees in fifteen calls at 137 000 tokens each.
7. **A lower context budget from a single source, and a `SessionStart` hook that
   says when `guardrails` is missing.** On 2026-09-08 session 83dcf00a ran to
   322 000 tokens with the budget hook not installed; the first anyone knew was
   being asked why the nagging had stopped. A handoff costs one call to write
   and a 46 000-token floor to resume, so it repays itself after about ten
   calls — the economic threshold is far below where sessions actually stop.
8. **Mechanical lanes default to Sonnet**, named by kind in the `delegating`
   skill. Subagent-Opus was $300 against subagent-Sonnet's $35.

### 99.5 What this section is not

It is not a budget, and the numbers in it are a week's measurement rather than a
bound. **Do not update them in place.** A count goes stale on the next push —
§77's tally of CI runs went stale twice inside a day — and a figure here that
drifts is worse than no figure, because the argument it was carrying keeps
reading as measured. Re-measure and write a new section; leave this one as the
record of what was true on 2026-09-14.

