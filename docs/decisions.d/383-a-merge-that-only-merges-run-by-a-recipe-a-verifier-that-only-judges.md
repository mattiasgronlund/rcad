## 383. A merge that only merges, run by a recipe; a verifier that only judges — 2026-10-03

*Mattias asked why the Q774 run took 27 turns to merge and 22 to verify. He
also asked whether a script could do most of both, and whether a Sonnet or
Haiku session could merge the common case and call in an Opus one when the
merge is not clean. This section measures what those turns are and settles
how. It changes no code; Q792 and Q793 do.*

In short: since 10-01 the git half of every merge has been clean. But 14 of 15
merges added rows and 6 made a verifier's fix, and that work is half the
merger's time. A Haiku agent running a script can only merge once someone else
has written the rows. So the lane writes them, as `add` steps, while it still
has the context. A recipe, `just land`, then does the whole of a clean merge,
and `lanes.js` hands any merge the recipe stops on to today's Opus merger. A
verifier's fixed steps and its re-runs of the lane's numbers go to a second
recipe, and the verifier keeps the reading.

### 383.1 What the turns are

Every merger and verifier transcript from a lanes run started since 2026-10-01
was measured: 15 mergers and 19 verifiers, under
`~/.claude/projects/*/*/subagents/workflows/wf_*/`. A turn is one model
response that calls at least one tool. A turn is *fixed* when every call in it
is a step its role's file names in a fixed form. For the merger that is fetch,
switch, unlock, `rev-parse`, the merge, `queue apply`, `check-queue`, `git
add`, commit, status, gate, the ancestor check, the push, `ff-main` and
`mark-finished`. For the verifier it is unlock, `rev-parse`, `log`, `diff
--name-only`, `lines-that-run`, checkout, gate and status. A verifier turn
counts as a *number* when each call in it is one of the commands in the lane's
`numbers`. Every other turn is *reading*. A turn's time runs from the turn
before's last result to its own last result, so the thinking before a call
counts with that call. The scripts were `mergers.py` and `split.py` in session
`c3fce9b6-5564-425f-a370-dd4d47b0dce0`'s scratchpad under `/tmp`, which a
reboot clears. This paragraph is enough to rebuild them.

Per run, on average:

| | fixed | numbers | reading | report |
|---|---|---|---|---|
| merger, 15 runs | 13.5 turns, 124 s | — | 14.1 turns, 114 s | 1.0 turn, 10 s |
| verifier, 19 runs | 7.7 turns, 91 s | 2.3 turns, 58 s | 12.2 turns, 125 s | 1.1 turns, 57 s |

- **The merger's reading calls**, per run: grep 5.1, `just decision` 2.3,
  `just queue` 1.9, Read 1.8, Edit 1.3. The grep calls read the queue and the
  lane's code. Each merger's `queue apply` came 70 to 229 s into a run of
  about 220 s; in most it came after about 130 s.
- **The verifier's reading calls**: grep 3.1, Read 1.6, `git diff` 1.5, `just
  t` 0.9, `ls` 0.6, Edit 0.6. The `just t` runs are mostly the test run after
  the revert. That is fixed work, but the count above files it as reading.

Q774 (`wf_cfa0186e-82d`) shows both:
- **Its merger** spent from 13 s to 124 s reading `docs/queue.md`, §0,
  `walk.rs` and `write.rs` to write two rows and their `after`s. The lane had
  already read all four. One turn, 37 s, went on composing the `queue apply`
  line.
- **Its verifier** spent five turns working out that the 164 in "164 of 164"
  is the length of the pinned list, a test the gate had already passed. It
  also ran the 19 s corpus test three times.

### 383.2 The merges since 10-01 were clean at git

Of the 15, none hit a `git merge` conflict, a failing gate or a lost push
race. One needed a catch-up over docs alone. One returned nothing (Q753,
`wf_2a30358f-f81`). Over all 202 mergers since 09-26 the picture is different:
many of the older ones needed two or three push attempts. §367's free
catch-up over docs and the served gate (§337) both landed between the two
periods. Whether they are why was not measured.

**There is no merge lock.** Merges inside one run wait for one another
(`lastMerge` in `lanes.js`). Merges from different runs race, and the ancestor
check before the push and `ff-main` settle it (§266.4). The clean merges are a
measurement, not a guarantee, and `just land` keeps the retry loop.

### 383.3 What a merge needs judgement for

Going through `merger.md`'s steps, a script cannot do four things:

1. **Turn what the lane leaves into rows.** That covers the lane's `remaining`
   and `out_of_write_set_needs`, the verifier's unfixed findings, and a new
   section's "what this leaves". Each row needs a tier, a model, a write set,
   a class, maybe `small` or an `after`, and must not repeat an open row. 14
   of 15 merges did this.
2. **Make a verifier's `fix_first` fixes.** 6 of 15.
3. **Fix a decisions note the lane's change made stale, or edit §0 when the
   merge closes what §0 names.** Not counted.
4. **Settle a conflict, a failing gate or a moved count.** 0 of 15.

Item 1 is the only one nearly every merge needs, and the merger is the wrong
place for it. It has to read back what the lane already knew.

### 383.4 What changes

1. **The lane writes its rows.** Its report gains `rows`. Each is an `add`
   step for `just queue apply`: tier, model, write set, and what, ending in §N.
   A row that waits on another names it as `after`. The rules for tier, class
   and `small` in `merger.md` step 3.4 move to `lane.md`. The lane looks for
   a row already in the queue at its base before it adds one. `remaining`
   stays, for what is not a row. §230.7 still holds, since the rows land in the
   merge commit. So does §266.3's "a lane still never writes the file": the
   lane writes the steps and the recipe writes the file. The verifier checks
   the rows as it checks any claim. Each must match what `remaining` says, and
   none may repeat an open row on `origin/main`.
2. **`just land` does a clean merge.** It takes the row or rows, the lane's
   branch, the sha the lane reports, the trees to unlock, and the `add` steps.
   It runs `merger.md`'s steps 1, 3, 4 and 5 as they stand:
   - fetch, branch from `origin/main`, unlock, `rev-parse`;
   - `merge --no-ff --no-commit`, then one `queue apply` with `done` and
     every `add`;
   - `check-queue`, `git add`, commit, `gate quiet`;
   - the fetch, ancestor check and push loop, with §367's free catch-ups and
     the same bounds;
   - `ff-main`, unlock, `mark-finished`.

   It ends with one line of JSON holding `mergedSchema`'s fields. It stops,
   and says why, on any of these: the sha differs from the lane's report, any
   conflict, §0 names the row, `check-queue` fails, the gate fails, or the
   attempts run out.
3. **`lanes.js` picks the merger.** When the verifier says `merge` and the lane
   reported its rows, a Haiku agent, `lander`, runs `just land` once and passes
   its JSON back. That is how the pruner runs `prune-worktrees`. A `fix_first`
   goes to today's Opus merger. So does every merge `just land` stopped on,
   and the stop line goes in its brief. That merger needs a branch name of its
   own, since the lander's `merge-<label>` already exists.
4. **`just verify-lane` does a verifier's fixed steps.** It runs, in order:
   - unlock the lane's tree, `git log` from the base to the branch, and the
     files outside the write set;
   - `lines-that-run`, before the checkout as now, then the checkout;
   - `gate quiet`, and the revert, test and restore of step 7. It skips the
     revert when the count is 0 or the lane is `pinsOnly`. When a source path
     also holds test code the branch changed, it leaves that revert to the
     verifier.

   It prints every `verdictSchema` field it can fill. For each of the lane's
   `numbers`, it runs the command and says whether `value` appears in what
   the command printed. For that, `value` must be text the command prints, and
   `numberSchema` says so. The verifier then does the reading:
   - what the branch takes out of a test;
   - the section's claims, under §16.10;
   - any number the recipe did not find;
   - a revert by hand;
   - the rows.

### 383.5 Why `lanes.js` escalates, not the cheap agent

Mattias proposed a Sonnet or Haiku merger that starts an Opus agent itself when
the merge is not clean. Nobody here checked whether an agent a workflow started
can start one of its own. The choice does not depend on it:

- The recipe's stop line decides, not a cheaper model's reading of what went
  wrong.
- The Opus merger starts with a clean context and a brief that names the stop.
  It does not inherit the Haiku agent's context.
- The run record shows the escalation as its own agent, with its own time.

`lanes.js` already picks the next agent by the verifier's verdict. This adds one
more case to the same code.

### 383.6 What it saved, measured once both rows had run

The paragraph below replaced an estimate. Measured 2026-10-24, over every
`lander`, `merger` and `verifier` agent of a `lanes` run started from
2026-10-03 (when Q792 landed) to 2026-10-24, joined to its row and lane by
the same `(run, lane)` key `row_time.py` uses, and read with its own loader
(`load()`, `read_transcript()` in `tooling/row-time/row_time.py`) rather than
a new one. The turn-counting itself — grouping `tool_use` blocks by the
streamed message id they share — is the kind of script §383.1 built and
named, run fresh from this session's scratchpad; a reboot clears it, the same
as the originals.

- **A clean merge is not about a minute.** Of 793 landers, 723 never needed a
  merger. Their turns are as estimated — 4 at the median (mean 4.6; 230 of
  793 took exactly 4, 195 took 5): `git rev-parse`, a `Write` of the steps
  file when the row has any, one `mise exec -- just land ...`, and the
  report. But their wall time is a median **10 m 56 s** (mean 12 m 02 s, p10
  3 m 08 s, p90 21 m 10 s), not a minute. The commands' own time — fetch,
  merge, gate, push, `ff-main` — is still a minute or two of that; most of
  the rest is the land lock (§384.1): lanes now land far more often, with no
  Opus turn in the way, and queue behind one another to do it (§397.5). A
  bundle of 24 rows landed by one recent lane (`just row-time Q2545 Q2540
  Q799 Q1240 Q1756 Q1964 Q1347 Q1378 Q728 Q1060 Q1995 Q2393 Q2550 Q1435 Q1448
  Q1542 Q1847 Q1720 Q2532 Q643 Q1691 Q1695 Q1301 Q1997`) shows a lander step
  of 24 m 11 s at the median with 10 m 50 s of it land lock; `just row-time
  --since 2026-10-03` puts the lander step at 6 m 40 s median, 19 m 46 s p90,
  over all 1351 rows it picked (most with no lander at all, which pulls the
  median down). Against the roughly 220 s-to-250 s the Opus merger cost
  before (§383.1's table: 124 s fixed + 114 s reading + 10 s report), the
  lander's own turns are far cheaper, but its wall time, with the queue it now
  sits in, is longer, not shorter.
- **A verifier's fixed and numbers turns did collapse to about one call**,
  but its total did not fall. `just verify-lane` (with an occasional `wait`
  poll while it runs detached) is 1.37 turns on average, 1 at the median,
  replacing the 7.7 fixed and 2.3 numbers turns of §383.1's table — a drop of
  about 8.6 turns on the step itself, close to the "about 10" estimated. Yet
  959 verifiers since 10-03 average 22.5 turns (median 21), against the 23.3
  total of 7.7+2.3+12.2+1.1 before. Their "other" (reading and report) turns
  average 21.1, median 20 — well above the 13.3 (12.2+1.1) they were. Some of
  the rise is a wider mix of rows reaching a verifier than §383.1's 19-run
  sample saw: one sampled run, `verify:q1428-a-fuzz-group-named-by-any-crash-
  file`, a fuzz-triage verifier, ran 78 turns on its own, almost all reading.
  The saving §383.6 named is real and lands on the step it named; it does not
  show up in the verifier's total turns, because something else grew at the
  same time.
- **What writing its rows costs a lane** is a handful of extra reading turns,
  not nothing, only on a run that has something to add. Of 40 sampled lanes
  (2026-10-10 to 10-18), 11 (27.5%) wrote a non-empty `rows` list; widening
  the sample to 60 found 16. On those, the calls that look for an open row
  before adding one — `grep` over `docs/queue.md`, `just queue show`, `just
  decision` — average 3.36 turns near the end of the run (median 3, max 9),
  costing a mean 9.7 s, median 6.8 s, of their own command time. This does not
  count the model's own thinking and writing of the `add` steps, which was
  not isolated.

### 383.7 What this leaves

- **Q792, the merge.** The lane writes its rows; add `just land` and
  `lander`; `lanes.js` picks the merger. Tier 2.2, Opus.
- **Q793, the verifier.** `just verify-lane`, `verifier.md` and
  `numberSchema`. It waits on Q792, since both change `lanes.js`. Done
  2026-10-03 (§385).
- **A `.claude/settings.json` line allowing `mise exec -- just land *`.** That
  line is Mattias's to commit. Without it the lander's push may stop on a
  permission prompt. The pruner has the same kind of line for
  `prune-worktrees`.
- **A new agent type reaches only a new `claude` process.**
