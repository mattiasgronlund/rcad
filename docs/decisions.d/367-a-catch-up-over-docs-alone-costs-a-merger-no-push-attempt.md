## 367. A catch-up over docs alone costs a merger no push attempt — 2026-09-29

*Mattias agreed the rule on 09-29. This amends §266.4's bound of three
attempts, and closes Q256, which asked for the same thing by another route.
The figures below were measured by session rcad-cf from the merger transcripts
and were not re-run for this section.*

In short: a merger that loses the race for `main` merges `origin/main` again,
gates, and tries again, and §266.4 counts every such try against three
attempts. When everything `main` gained is under `docs/` or ends in `.md`, the
code the gate reads has not moved, the gate serves its slow steps (§337), and
the try costs about a minute. Such a try no longer counts as a push attempt. It
counts in `docs_catch_ups` instead, which has a bound of five, so a merger still
cannot loop.

### 367.1 What it cost

Q574's merger, in run `wf_01845d34-6f7`, gated green three times on 09-29 and
never pushed:

| gate | tree | UTC | result |
|---|---|---|---|
| 1 | `a73941ae` | 13:13 | passed, and Q373's merge reached `main` meanwhile |
| 2 | `99282553` | 13:20 | killed at 13:21:45 by a `/clear` |
| 3 | `99282553` | 13:21 | passed, and Q350's merge reached `main` meanwhile |
| 4 | `809f833a` | 13:30 | passed, and six `docs/queue.md` commits reached `main` meanwhile; out of attempts |

The third catch-up was over the queue file alone, and its gate would have been
served in about a minute. It cost the last attempt instead, and the dispatcher
landed the row by hand as `b6aa30e2`. Q571's merger failed the same way that
afternoon. Q256 recorded the same on 09-27: Q175, Q232 and Q235 each spent all
three attempts, mostly on queue takes and row adds.

### 367.2 How often

Over every merger transcript there were 154 mergers that gated and 354 gate
starts: 251 passed, 3 failed, 13 were killed with exit 137, and the ending of
87 could not be read. `main` had moved under a gate 141 times. For 49 of those
the merger printed what `main` gained: 27 were over docs alone and 22 over
code. The docs-only ones are queue commits and decisions sections both, not
split further.

### 367.3 Why a catch-up over docs alone is cheap

The gate names a tree by HEAD's files without `docs/` and `*.md`
(`_code-fingerprint`, §337.3). A merge that brings only such paths leaves that
name as it was, so the gate after the catch-up serves every step in
`gate_reusable_steps` from the gate the merger has just passed.

The rest still run. 46 of the 55 `gate_steps` are not on that list (§337.2),
and timed one by one on 09-29 they took 55.6 s:

- about 7.5 s read docs or a `.md`: `check-decisions` 3.8 s, `check-queue` 1.5,
  `check-lanes-workflow` 1.3, `check-where-the-work-is` 0.5, `check-plan` 0.4,
  `check-skills` 0.1;
- about 8.5 s read something outside the tree: `check-hooks` 6.2 s,
  `check-plugins` 1.2, `deny` 0.9, and `check-corpus`, `check-hub` and
  `check-fuzz-unit`;
- about 40 s are tooling selftests that §337.2 never looked at, because a step
  is not served until someone adds it. The largest are `check-queue-selftest`
  13.0 s, `check-gate-verdicts-selftest` 4.2, `check-questions-selftest` 3.7
  and `check-status-selftest` 3.3.

Two served gates seen that day took 1m33s (Q574's verifier) and 1m13s (the
dispatcher's). A tree whose sha starts with `0` runs every step (§337), so
about one catch-up in sixteen is a whole gate anyway.

### 367.4 The rule

Before it merges `origin/main` again, the merger runs

```
git diff --name-only HEAD...origin/main
```

When that prints paths and every one is under `docs/` or ends in `.md`, the try
after the catch-up counts in `docs_catch_ups` and not in `push_attempts`. An
empty list is not docs alone: it means the merger has not fetched since its
push was refused, and `merger.md` tells it to fetch first.

`DOCS_CATCH_UPS` in `lanes.js` bounds these at five, and the brief hands the
merger that number. Past it the merger stops, as when its attempts are spent.
The merger's schema requires `docs_catch_ups`, so every merger says how many it
used and the count can be measured, and the script logs a count past the bound.
`check-lanes-workflow`'s fourteenth arm holds the diff line, the bound and the
schema field.

**Five is a judgement, not a measurement.** At a served gate's minute and a
half, five tries cost under ten minutes. Six queue commits reached `main` during
one of Q574's gates, so queue commits come in runs, and five tries is room to
outlast one. A merger that loses five catch-ups over docs in a row is racing
something the dispatcher should look at, the same argument §266.4 made for
three.

**The gate stays whole.** Q256 asked for `gate-docs` on such a tree instead.
§337 has since made the whole gate serve its slow steps there, which left the
attempt as the only real cost. The steps a served gate still runs include the
ones that read prose, such as `check-queue` and `check-decisions`, and those
are the ones a docs change can fail.

### 367.5 What it does not reach

A race of code against code, 22 of the 49 above. The other merge's code moves
the tree's name, so the catch-up is a whole gate and rightly an attempt. Only a
lock on merging to `main` would stop those, and that would reverse §266.4; it
is left to Mattias.

A running `claude` process does not re-read `.claude/agents/` (§266.6), so the
new `merger.md` reaches a merger only from a process started after this lands,
and the new `lanes.js` only from such a process or when launched by
`scriptPath`.

### 367.6 What this leaves

- **Q687**: audit the tooling selftests for serving, the 40 s of §367.3's
  third bullet. It speeds every served gate, a catch-up's included.
- **Q688**: a lane runs `just fmt` before each commit. Q574's tip, which is
  the sha its row records as done, was a rustfmt-only commit, because its first
  gate failed at `lint`.
- **Q689**: a verifier's step 6 runs the lane's tests right after its own gate
  ran them on the same code, about five minutes for Q574.
