---
name: delegating
description: Split work into lanes, brief them, and run at least three at once. Use when handing rcad work to subagents, when planning a milestone's lanes, when a lane runs long, or when merging lane work back.
---

# Running lanes

## The write-set table comes first

Before dispatching anything, write this out:

| Lane | Deliverable (one sentence) | May write | Reads |
|---|---|---|---|

Two lanes are concurrent when their **May write** columns are disjoint. Reads never
conflict. If you cannot fill the table, you do not yet know what the lanes are, and
dispatching is guessing.

**Dispatch with `isolation: "worktree"` on the Agent call unless you have a reason
not to**, letting the harness make the worktree. `kache` still shares the compile
cache across them, so the cost is the merge and nothing else.

The alternative — `git worktree add` by hand and the path pasted into the lane's
prompt — was the default here and it is the worse one, for a reason measured on
2026-09-06: **a hand-made worktree does not change the lane's working directory.**
Every subagent in the 2026-09-05 transcripts started in the primary tree, including
the ones whose brief named a worktree, so reaching the right tree depended on a `cd`
the lane had to remember. One lane forgot and wrote the shared tree. Under
`isolation: "worktree"` the harness sets the working directory to the worktree —
verified by a probe whose own `pwd` was its worktree — and the create hook locks it,
so it also arrives with an owner on record. Neither is true of the hand-made kind.

Same worktree with disjoint writes is still right when the lanes are short and the
write sets genuinely do not overlap; it needs the `git add <path>` discipline below.
Serialising is the third option and the worst one. Take it only when lane B needs
lane A's *output*, not merely its file.

## Never dispatch into a worktree somebody holds

`just lanes` before you dispatch. A worktree reported `HELD` has an owner named in
its lock reason; **message that owner instead of sending a second agent in.** Two
agents in one worktree is not a hypothetical: on 2026-09-05 it cost a spike both of
its remaining answers, a probe SIGTERMed mid-run and an output file filled with the
other agent's output, over 27 minutes.

The lock is the record because it is the only thing that survives the session that
made it. `ps aux | grep claude` cannot do this job — a subagent runs inside its
parent's process, so two agents colliding in one worktree are one pid with one
working directory.

## Dispatch them in one message

Concurrency comes from putting several Agent calls in a **single** assistant message.
One call per message runs them one after another no matter how independent they are.
Minimum three unless the table says otherwise.

This is the rule that gets broken by habit rather than by reasoning. M5 got **1.07x
out of 16 subagents** — 9.2 agent-hours inside 9.6 wall hours — because the lanes
were dispatched one at a time when nothing in the write-set table said they had to
be. Lanes conflict only when their *write* sets overlap; a file two lanes merely
read is not a conflict.

Pick the model per lane: a mechanical sweep or an audit is Sonnet work; a lane that
has to reason about the arithmetic is Opus.

## The brief

Paste this into every lane, filled in. The lane cannot see your memory, this skill,
or the other lanes' briefs — anything you leave out, it will invent.

> You own these files and no others: `<paths>`. If you need a change outside them,
> stop and report it; do not make it.
>
> Deliverable: `<one sentence>`. Done when: `<the exact command that proves it>`.
>
> - `Read` then `Edit`. Never patch a file through a python/sed/cat heredoc.
> - Prefix every cargo/just with `mise exec --`. `RUSTC_WRAPPER = "kache"` lives in
>   mise's `[env]`, which a non-interactive shell does not inherit; without the
>   prefix every build is uncached.
> - Stage with explicit `git add <path>`. Never `git add -A` — another lane is
>   working in this worktree and `-A` sweeps its half-finished files into your commit.
> - Commit as you go. If you are killed, you keep only what you committed.
> - Budget ~45 minutes. If the work is bigger than that, commit what is done, report
>   what is left, and stop. Do not grow the lane.
> - Report: what changed, the test that fails without it, and each number you measured
>   with the command that produced it. Never restate a number you did not run yourself.
>
> `<verbatim API signatures and the exact-arithmetic facts this lane needs>` —
> rediscovering these costs a lane twenty minutes.

A test-only lane gets **no write access to `src/`** and reports defects instead. A
metamorphic suite that is allowed to edit the kernel until it goes green is worthless.

Do not tell a lane to read CLAUDE.md. It is injected into every subagent's context
already — verified by a probe subagent that quoted it back with zero tool calls — so
the instruction buys nothing and cost 8 KB a lane across 16 lanes on 2026-09-05. If
one rule is load-bearing for that lane, quote the rule in the brief.

The 45 minutes is not a scheduling preference. **A long lane is slow twice**: M5's two
longest ran at **380k average context per request**, and every turn pays that context
again. A lane that wants more is two lanes — commit what is done, report what is left,
hand it back. Against that, a fresh lane costs ~20k tokens to start (system prompt,
tool schemas, CLAUDE.md, memory), so splitting is worth it well before the second
lane's own context reaches the first's.

## While they run

Stay small. The dispatcher's job is to receive reports and dispatch the next batch,
not to read the code itself — that is what burns the context that makes the lanes
expensive in the first place.

**A lane is finished when its worktree unlocks, not when it reports.** A report is
the lane answering the question in front of it, and an agent you have sent a
follow-up to has more of them queued. That is exactly how the 27 minutes happened:
three questions went to one spike by `SendMessage`, it answered the first, the
dispatcher read that answer as the end, and a second agent went into the same
worktree 100 seconds later — while the first woke to question two. Nothing about
the report said which of the three it was.

**The dispatcher has a budget too, and it is 250k tokens.** The lane cap works — 51
lanes on 2026-09-05, median 17 minutes, none over 45. The dispatchers had no cap and
spent 485M input tokens over 1994 turns, 97% of it re-reading context, while writing
almost no code. `.claude/hooks/context-budget.sh` says so once when the line is
crossed; call the `handoff` skill then, rather than at the end of the next unit.
Raise it for a session with `CLAUDE_CONTEXT_BUDGET`.

## Closing

Run `mise exec -- just default` yourself after every batch, more than once. Then one
gate agent that re-derives every load-bearing number in the lane reports from the
tree, and only then the merge.

**Not the lane's own word for any number.** M5's gate caught an unclosed §19 violation
and a 5.4x reported as 20x, both of which all five lanes had missed. A lane reporting
its own result is reporting what it believes.

**Run it in a tree that is yours alone**, and not one where anybody else has `main`
checked out. On 2026-09-05 the merge tree was shared with a live session: a "green
twice" was measured over somebody else's uncommitted work, and a clean fast-forward
had to be handed back to Mattias undone, because landing it would have moved the
other session's HEAD underneath it.

**Unlock each worktree as you merge it** — `git worktree unlock <path>`. Nothing
releases a lock on its own, so a lock left behind is rubbish that only a person
clears, and `just prune-worktrees` will keep the worktree for ever rather than
reclaim it. The ISO date in the reason is what tells a later reader that a lock is
abandoned rather than live.
