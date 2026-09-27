## 304. Why two claims outlived their queue rows — 2026-09-27

*Queue row Q235. Q124's and Q128's claims holds outlived their rows by about
five and a half hours: the lanes workflow that ran them never started a Merge
phase for either, because the fix that would have let them merge landed two
minutes after both workflows had already read the old script into memory. A
dispatcher's hand merge dropped both holds, late. No code changed here; a
follow-up row is proposed in 304.4.*

### 304.1 What "outlived" means here

At 2026-09-27T00:00:19Z / 00:00:20Z, a dispatcher re-took Q124 and Q128 under
new branches after their first lanes were killed by the weekly usage limit
(commit `1d36734b`, "queue: re-take Q208, Q124, Q128 and Q148 under new
branches after the usage limit killed their lanes"; claims file lines 164-165:
`take Q124 2026-09-27T00:00:19Z ... lane-q124-other-nappe-mesher-2` and `take
Q128 2026-09-27T00:00:20Z ... lane-q128-thin-corner-2`). Both lanes ran,
finished, and were verified with verdict `merge` by 2026-09-27T00:41:39Z
(Q128) and 00:44:13Z (Q124) — the completion timestamps of
`workflows/wf_5f9420b1-6d7.json` and `workflows/wf_878d38d9-ca1.json` in the
dispatcher session `593f4204-93bb-48ec-938d-802a7278dee2`. §299.1's "at 00:40Z
on 09-27, Q124 and Q128 were both stuck exactly there" is this same moment:
the verifier's detached tree was still the one actually locked when Q217
observed the double count, because the row was still — correctly — held.

The hold did not become wrong at 00:40Z. It became wrong afterward: the row's
actual work was done and verified, yet nothing released it. Both rows stayed
`lane:<branch>`, held, until `release` events at 06:16:07Z (Q128) and
06:16:22Z (Q124) — claims file lines 182 and 185, both `how` = `done` —
matching two hand-merge commits, `ecc2feb9` (2026-09-27T08:16:13+02:00 =
06:16:13Z) and `4dd301b2` (2026-09-27T08:16:30+02:00 = 06:16:30Z). Between
00:41Z/00:44Z and 06:16Z, the row was "done" in every sense that matters
except the one the claims file and `docs/queue.md` record.

### 304.2 Why the lanes workflow's own Merge phase never ran

Both lanes' write sets were a test file plus a decisions section, with nothing
in `src/` — the shape `docs/decisions.d/287-a-lane-that-only-pins.md` (Q208)
built `lane.pinsOnly` for, after Q126, Q127 and Q203 were stuck the same way.
§287's fix landed as commit `cbe8b56e`, "lanes: let a pins-only lane merge
without a revert to demonstrate", at 2026-09-27T00:03:20Z — two minutes
*after* both of these workflows had already started
(`workflows/wf_878d38d9-ca1.json` and `wf_5f9420b1-6d7.json` both give
`"startTime":1790467281037` / `1790467281219`, i.e. 2026-09-27T00:01:21Z).
`cbe8b56e` is not an ancestor of the base sha these lanes read
(`ff78f99f...`); `ff78f99f` is an ancestor of `cbe8b56e` instead — `cbe8b56e`
is the very next commit after that base, landed while these two lanes were
already mid-flight.

A `lanes` workflow embeds the full text of `.claude/workflows/lanes.js` at
launch (`scriptPath` is read once, into the JSON's own `"script"` field) and
never re-reads it. Both workflows' embedded scripts are the pre-§287
`unproven()`, with no `pinsOnly` branch at all:

```
if (!sourcePathsOf(lane).length) why.push('the lane declared no source directory to revert')
```

So when each verifier came back with `"verdict":"merge"` (both journals show
this), the stale `unproven()` still refused: each workflow's own log records
"not merged -- the verifier said merge, but the tests were never seen to fail
with the source reverted, and no reproduced mutation stands in for it; the
lane declared no source directory to revert", `merged:false`, and neither ever
started a `Merge` phase agent. This is confirmed by absence as well as by the
log line: no `merge-q124-other-nappe-mesher-2` or `merge-q128-thin-corner-2`
branch, and no `"phase":"Merge"` entry for either label, appears in any
journal or workflow file on this machine — `mergeBranchOf()`'s naming and the
`MERGER` agent are what an automated merge always leaves behind, and neither
row left one. Both workflows then ended normally (`"status":"completed"`),
having handed each row back unmerged — the exact pattern §287.1 recorded for
Q126/Q127/Q203, reproduced here because this pair of runs was already in
flight when §287's own fix landed.

### 304.3 What actually dropped the hold

Since no merger ever ran, `just queue done` was not called by an automated
merger for either row. The two `release` events (304.1) coincide exactly with
two ordinary two-parent merge commits, `ecc2feb9` ("Merge
lane-q128-thin-corner-2: §286, ...; Q128 done, Q218 and Q219 added") and
`4dd301b2` ("Merge lane-q124-other-nappe-mesher-2: §285, ...; Q124 done, Q220
and Q221 added after Q204 and Q205") — the same commit-message shape other
dispatcher hand merges in this history use (e.g. "Merge lane-q110-...: ...",
"Merge remote-tracking branch 'origin/main' into merge-q225-..."), not the
`merge-<label>` branch an automated merger creates and reports through
`rows_closed`. So the step that dropped each hold, five and a half hours
late, was a dispatcher's hand merge: someone noticed both lanes had come back
"verifier said merge" but unmerged, merged each by hand onto main, and ran
`just queue done` directly.

### 304.4 Whether it can recur

Yes, on the same mechanism, independent of `pinsOnly` specifically.
`tooling/queue/queue.py`'s `take`/`done`/`close` behaved exactly as designed
throughout — there is no bug in the claims file or its verbs to fix here.
The gap is in the lanes workflow: it captures `.claude/workflows/lanes.js` as
literal text at launch and runs on that frozen copy for its whole duration,
however long that is. Any future change to `verified`, `unproven` or `merged`
can land while another workflow is already mid-flight, and that workflow will
decide every remaining row's fate on the old rules regardless. A row shaped
exactly wrong for the stale copy comes back `merged:false` with nothing further:
no retry, no flag, no signal beyond the workflow's own returned lines — and its
claims hold simply waits for a dispatcher to read `just queue running` or
`just status`, notice the stray "verifier said merge" result among the ones it
expected, and merge it by hand. That noticing is not automatic and this run
shows it can take hours.

**Proposed follow-up** (not built here; `tooling/queue/queue.py` is Q234's
write set right now):

`2 | sonnet | .claude/workflows/lanes.js | when a lanes workflow ends a row
with a verifier verdict of merge but merged: false (the row was never handed
to a Merge phase, or a merger declined for a non-content reason), have the
workflow say so more loudly than a log line among many -- carry it into the
returned row's own text so a dispatcher reading the twenty-line result, not
just the log, sees "verified, not merged" and can hand-merge it promptly
rather than by chance. §304`
