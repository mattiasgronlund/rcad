## 160. The context ceiling is not a dial problem — §144.3 re-measured, applied 2026-09-19

*§144.3 called the context ceiling the largest single lever in that survey and
asked a lane to lower the budget until the long tail past 200k went away. The
premise did not survive the re-measurement it also asked for. The budget hook
fires on time — a median of 6 400 tokens past a line it is holding at 200 000,
which is 3% late — and 39 of the 40 sessions that crossed in the week of
2026-09-12 were warned, the exception dating from the week the hooks moved into
plugins. What it could not do was say it twice, or say it from inside a turn,
and that is where 22 of the 23 points of overspend are. The dial did not move.
Two things about the hook did, in the `guardrails` plugin, and a report was
built here so the next session can check rather than believe.*

### 160.1 What was measured, and the report that does it

`just spend <since> <until> warnings`, new here, beside the `ceiling` report
§144.3 reasons from. They are not the same question. `ceiling` prices a cap
**nothing enforces**: what the week would have cost if no response had ever been
billed above one. `warnings` prices the ceiling **that is installed**, by
splitting the same spend four ways — under the budget, inside the turn that
crossed it, after the session had been told, and in a session nothing told.

Each session is measured against the budget the hook itself named in its own
message, not against today's `context-budget.default`. The window of 2026-09-12
straddles the drop from 250k to 200k, and a fixed number would have misfiled
every session on the far side of it.

Run over 2026-09-12 to 2026-09-20, 77 dispatcher sessions and $841 between them:

```
  under the budget     $  640.65   76.2%    76 sessions
  before the warning   $   80.08    9.5%    39 sessions
  after the warning    $  110.49   13.1%    39 sessions
  never warned         $    9.37    1.1%     2 sessions

  39 first warnings: median 6,377 tokens past the budget, worst 106,521
  39 sessions kept going afterwards: median 13 further responses, worst 68
```

**These numbers move while you read them**, and one of the two sessions in the
last row is the session that ran the report. That is the point of §144.10: run
the command, do not quote this block.

### 160.2 §144.3's premise — "it fires too late" — is wrong

The brief says the mechanism works but fires too late, and reasons from a median
session billed at about 130k and expensive ones running to 250k and beyond. Both
halves of that are true and neither one is about lateness. The hook was already
at 200k when §144.3 was written — `3a27299`, 2026-09-14 — and it caught the
crossing within 3% in the median case. A session at 250k is not a session the
hook reached late; it is a session that was told at 206k and kept going.

This is §77.6's shape in a new place. The brief measured a property of the
outcome — how big sessions got — and attributed it to the one part of the
mechanism that has a number in it, because the number is the part that is easy
to look at.

### 160.3 Where the 23% actually goes

**13 points are billed after the warning was delivered.** A median of 13 further
responses, 68 of them in the worst session for $11.26, which is more than an
average whole session costs. Only 15% of that tail touches a handoff at all, so
it is not the cost of obeying; 34 of the 39 warned sessions did eventually write
one, having first finished what they were doing at the most expensive context
they would ever be at. The hook spoke once and was then silent by construction:
a sentinel file per session meant the session that ignored it — the expensive
case, by definition — was the one guaranteed never to hear again.

**9 points are billed inside the turn that crossed the budget**, where no `Stop`
and no `UserPromptSubmit` can run. A turn here is long: 496 turns of two
responses or more that week, median growth 9 473 tokens, and the worst grew from
57k to 246k across 105 responses without ever returning to the user. The worst
first warning in the table above — 106 521 tokens past the line — is that same
effect measured from the other end.

**1 point is a session nothing told.** Every silent session in the whole record
starts between `927eb580` on 2026-09-07, the day `.claude/` emptied into the
three plugins, and 09-11, alongside sessions on the same days that *were*
warned. `check-plugins` went into the gate the next day for exactly that failure,
and nothing since 09-12 has been silent. A missing plugin is a missing hook, and
a missing hook looks precisely like a hook with nothing to say — which is §152's
argument about the question file, arriving at the same conclusion from the
budget's side.

### 160.4 So the dial stays where it is

`ceiling` on the same week says a hard cap at 150k saves 8.9% of the bill and
125k saves 14.6%. Those remain the honest upper bounds for a cap **a session
cannot exceed**. Nothing here is that. Lowering an advisory line moves the
warning earlier and leaves the 13 points that follow it untouched, while buying
the extra handoffs §144.3's own last paragraph warns about — and §138 means each
of those is a real document, not a sentence.

The two leaks above are 22 of the 23 points, and closing them costs no handoffs
that were not already owed. That is the whole argument for leaving the number
alone, and it is why this section is not the lane §144.3 described.

### 160.5 What changed, in `guardrails` v0.3.2 through v0.3.5

**A rung, not a sentinel.** The warning repeats every 25 000 tokens of growth
past the first warning. On that week's tail that is one extra warning in the
median session past the budget and four in the worst — measured against $67 still
ahead of the first repeat, and against $30 still ahead of the second. Smaller
steps nag; the reason the original fired once was that nagging every turn is
worse than silence, and a rung keeps that while dropping the "once".

**"Past the first warning" is a correction, and v0.3.2 shipped it wrong.** The
rung was first counted from the budget, which nagged the session that wrote it
within two turns: warned at 272 565 tokens and again at 277 124, because the
first warning landed 72 000 past a 200 000 budget — already rung 2 — putting
rung 3 three thousand tokens away. That is not an edge case but **the** case, and
the same one §160.3's nine points are about: a session that crosses deep inside a
turn is first told a long way past the line, so counting from the budget puts the
next rung just ahead of wherever it happened to be standing, and the deeper the
crossing the sooner the nag. `v0.3.3` counts from where the first warning landed
and the case that proves it is in `check-context-budget.sh`, shown failing
against `v0.3.2` first.

**And v0.3.3 broke the upgrade, which is the third instance in one afternoon.**
It changed the state file from one number to two without saying what the
one-number shape means, so a v0.3.2 file holding the rung `2` reads as "first
warned at 2 tokens, and never warned". The same session, now warned twice, was
told at 299 056 tokens that it had **just** passed the budget and could raise it
with `CLAUDE_CONTEXT_BUDGET` — the wrong sentence at the worst possible size —
and its file became `2 10`, which puts every later rung 25 000 from zero. A live
session's state file outlives the version that warned it. `v0.3.4` restarts the
ladder from the current size and calls the next warning a repeat, that being the
one thing certainly true of a session whose file exists at all, and writes the
state **above** the quiet exit rather than below it, because the upgrading call
is a quiet one and a write that only happens when the hook speaks would re-decide
the baseline for ever. The case caught that second half; reading it did not.

**And fixing the reader did not undo what the bad reader had written — v0.3.5.**
v0.3.3 did not only misread the rung `3` as a baseline, it **persisted** it in the
new two-field format, so v0.3.4's careful handling of the old shape never looked
at it and the session was told `319,479 more than when you were first told`. The
close is an invariant rather than another migration rule: the only baseline this
script ever writes is one it has just found greater than the budget, so anything
smaller came from somewhere else and the ladder restarts. One comparison, on the
branch the upgrade already needed, and it closes the class rather than the
instance. **A data fix is not a code fix, and a code fix is not a data fix** —
three versions each corrected the reader while the file on disk stayed wrong.

Three things are worth keeping from how all this was found. It took **two turns**
each time, not a week, because the author was the subject — the fastest
measurement in this file so far, and available only because the thing being
changed acts on the session changing it. Every one of the four was in the
mechanism §160.5 describes rather than in anything §160.1 measures, which is the
division §77.6 draws: the numbers were right all afternoon and what was built on
them was wrong four times. And all four shipped with cases that passed — the
cases asserted what the code did on the paths its author was thinking about, so
each new one is worth more than the ten before it, and the count is not the
thing. A version that changes a file another version reads needs a case for
**reading the old file**, and one that has already shipped a bad write needs a
case for **reading what it wrote**; those are the general forms of the third and
fourth.

**`PostToolUse` beside `Stop` and `UserPromptSubmit`.** `tail -n 400 | jq` costs
14 ms on the largest transcript on this machine (3.4 MB), so the check can afford
to run after every tool call. Mid-turn the advice is deliberately narrower —
finish the call, do not abandon a merge or a gate run, hand off rather than start
the next thing — because a hook cannot see whether a merge is half done, and the
expensive turns are exactly the ones where it might be.

Four new cases in `check-context-budget.sh`, each demonstrated failing against
the old hook first: a step up warns again, the same size stays quiet, the
mid-turn wording is addressed to `PostToolUse`, and `hooks.json` routes all three
events — because a hook that answers correctly on an event nothing routes to it
is the silent point above, from the inside. The `delegating` skill said "fires
once when the line is crossed" and now says what it does.

### 160.6 The measurement had the same bug it was measuring

The first draft of the `warnings` report put four sessions in "never warned" and
three of them had been warned. A transcript records a hook two ways — the whole
stdout as `hook_success`, or just its summary line as `hook_system_message` —
and only the first names the budget. A parser that required the budget saw the
summary-only sessions as no warning at all, which reads as a broken hook rather
than as a broken parser, and would have put the finding's headline number 4x too
high in the one bucket that indicts the mechanism.

`warning_said` and `bucket_of` are pure functions for this reason and are held by
`check-spend-selftest` alongside `ritual_of`, which was wrong the same way for
§144.5. Two of the functions this file reasons from have now been wrong once
each; that is the argument for the cases, not a reason to trust the others.

### 160.7 What this does not claim

That the repeat and the mid-turn warning will be **heeded** is unmeasured and
unmeasurable in advance: the 13 points are what a session did after being told
once, and nothing here proves it behaves differently on being told twice. The
measurement that settles it is §144.3's own — cost per week at a steady amount of
work — re-run on a week that ran entirely under v0.3.5, with `0 repeat warnings`
in the report being the first thing to check, since a week with none means the
new path never ran.

Write set: the `guardrails` plugin (marketplace, `c57c714`, `5cc6465` and
`5eb22b9` and `3e476a4`, tagged `v0.3.2` to `v0.3.5`) and here `tooling/session-spend/`, its
cases, the `spend` recipe's doc, and the plugin ref in `.claude/settings.json`.

---
