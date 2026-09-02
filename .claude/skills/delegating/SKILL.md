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

When two lanes must write the same file — M5's cone and torus lanes both lived in
`triangulate.rs` — you have two choices, and taking the first by default is what cost
that run its day:

- **Same worktree, disjoint writes.** Cheap: no merge. Needs the `git add <path>`
  discipline below.
- **A worktree per lane** (`isolation: "worktree"` on the Agent call). No discipline
  needed, and `kache` still shares the compile cache across them, but you own the
  merge. Right when the overlap is real and the lanes are long.

Serialising is the third option and the worst one. Take it only when lane B needs
lane A's *output*, not merely its file.

## Dispatch them in one message

Concurrency comes from putting several Agent calls in a **single** assistant message.
One call per message runs them one after another no matter how independent they are.
Minimum three unless the table says otherwise.

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

## While they run

Stay small. The dispatcher's job is to receive reports and dispatch the next batch,
not to read the code itself — that is what burns the context that makes the lanes
expensive in the first place.

## Closing

Run `mise exec -- just default` yourself after every batch, more than once. Then one
gate agent that re-derives every load-bearing number in the lane reports from the
tree, and only then the merge.
