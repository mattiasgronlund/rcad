## 414. A verifier `merge` on a red gate goes to the Opus merger, not the Haiku lander — 2026-10-08

### 414.1 The measurement

Window 2026-10-01 to 10-08, re-run for this section (not copied from the
handoff that proposed it, `/tmp/rcad-handoff-lanes-js-merge-on-a-red-gate-2026-10-08.md`,
written by session `main-ec`):

```
python3 /tmp/claude-1000/-home-mattias-git-mattiasgronlund-rcad-worktrees-main/d82cdb29-95f8-4f99-a902-30ba28d53418/scratchpad/red_lanes.py
python3 /tmp/claude-1000/-home-mattias-git-mattiasgronlund-rcad-worktrees-main/d82cdb29-95f8-4f99-a902-30ba28d53418/scratchpad/land_stops.py
```

Both read the run records through `tooling/row-time/row_time.py`'s `load()`.
If these scripts are gone by the time this is read, the handoff above is the
only other record of the same numbers.

- **33 lanes handed over a red last gate.** Confirmed: `red_lanes.py` prints
  "33 lanes handed over with a red last gate" and lists exactly 33 entries.
- **5 of those came back `merge` with `fixes: []`: Q503, Q563, Q930, Q1033,
  Q1094.** Confirmed by scanning the same output for `verdict=merge` paired
  with a literal `fixes: []` line — exactly these five plus three more
  (Q1162, Q1309, Q1332) whose gate failed on a tooling selftest
  (`check-fuzz-overnight-selftest` twice, `check-verify-lane-selftest` once)
  unrelated to any file outside the write set, which is the handoff's other
  bucket, below.
- **The handoff's "30 red on an outside-write-set pin, 3 red on an unrelated
  tooling selftest" split is not a field in the report — it is a reading of
  free text**, as the handoff itself says ("I read that by eye from report
  text cut at about 220 characters; it is not a computed field"). Re-reading
  the same 33 entries by eye here agrees with the handoff's own split: the
  three it calls unrelated are Q1162 (`check-verify-lane-selftest`), Q1309 and
  Q1332 (`check-fuzz-overnight-selftest`) — the same three, by the same two
  recipe names, in the same 2:1 split the handoff reports. A fourth lane,
  Q453, also fails first on `check-fuzz-overnight-selftest`, but its own
  `out_of_write_set_needs` names the exact pin (`every_model_the_fuse_fuzzer_kept_is_fused_again.rs`,
  line 435 and its module table) that recipe checks, so it belongs in the
  30-bucket despite the recipe name; Q1332's `out_of_write_set_needs` names an
  unrelated `src/view.rs` finding that does not explain its gate failure, so
  it stays in the 3-bucket. This agrees with the handoff; it is written out
  here because the distinction is not visible from the recipe name alone.
- **The lander's stops on `gate`: `land_stops.py` lists 8 lanes, 6 of them
  stopped `gate`** (Q1143, Q1094, Q930, Q1230, Q503, Q563) **and 2 stopped
  `push`** (Q1176, Q1311). That matches the handoff's "Q503, Q563, Q930 and
  Q1094 are 4 of the week's 6 lander stops on `gate`. The other two are Q1143
  and Q1230, not examined."

So this run's numbers agree with the handoff's throughout; nothing measured
here disagrees with it.

### 414.2 The change

`landable(prior)` (`.claude/workflows/lanes.js`) decided a verifier's `merge`
was ready for the Haiku lander (`just land`) once the verdict was `merge` and
the lane's reported rows were steps `just land` could take. It never looked at
whether the verifier's own gate had passed, so a `merge` whose branch itself
failed its last gate — the five above — went to the lander anyway, and
`just land`'s shared gate (§397.5) went red with it: a cost beyond that one
lane's own gate, since a red gate there sends every other lane sharing the
run back to `origin/main` and lands its own lane alone.

`landable()` now also requires `prior.check.gate_summary` to hold a line that
reads exactly `gate: PASS` — a whole-line match, not a substring, since
`gate: FAIL  first failure: <step> (exit N)` is the other shape
`_gate-summary` prints (`just recipe _gate-summary`) and a served gate's
summary still opens with `gate: PASS` too. `gate_summary`'s own schema
description is "the summary block, verbatim, or why you could not run it"
(§383.4 item 4, §385) — a verifier that could not run the gate at all holds
neither line, so it reads as not passed here and goes to the Opus merger the
same as an outright `FAIL`, per the brief: a `merge` the verifier could
not gate at all is no safer to hand the lander than one it gated red.

The check lives in `landable()`, not in `unproven()`: a reason `unproven()`
returns goes through `stopped()` and `handedBack()` (lanes.js), and the row
comes back unmerged to the dispatcher. That is wrong for this case — the
lane's work was right, and it was the Opus merger, not another lane run, that
landed four of the five after their gate went red. `landable()` returning
false already routes to `byMerger()` (`merging()`), which is the path these
five needed.

The Opus merger's brief (`MERGE()`) said, for every row that never reached the
lander, "The lander did not run: the verifier asked for fixes, or the lane
wrote no rows just land takes." That is now a third reason too: "...or the
gate the verifier ran did not read gate: PASS." A merger reading the old text
on one of these five would have read a cause that was not true.

### 414.3 Out of scope

The handoff's item 3 — teaching `verifier.md` that a red gate together with
`out_of_write_set_needs` means `fix_first`, with those edits copied in as
`fixes`, which would make all five ordinary `fix_first` rows instead of
exercising this new path — is not done here. It is a second, independent
change to `verifier.md` and the prompt the verifier reads, not to `lanes.js`;
whether it is worth a queue row of its own is for whoever reads `remaining`
on this row's report.

See also §383.4 item 3 and §383.5 (which agent lands what), §384 (`just
land`), and §397.5 (the shared land gate landers run inside one detached
run).

### 414.4 The verdict says `fix_first` (Q1416)

Item 3 became Q1416. `verifier.md`'s "The verdict" now says that a gate red
only on pins the lane named in `out_of_write_set_needs` is a `fix_first`,
with each edit copied into `fixes`, and never a `merge` with `fixes` empty.
`landable()` already sends a `fix_first` to the Opus merger, so §414.2's
routing does not change. What changes is that the merger is handed the edits,
where before it was handed a `merge` and found the edits only when its gate
went red.

`merger.md` had to move with it. Its step 2 made a `fix_first` fix only when
the fix was inside the lane's write set, and told the merger not to merge on
any other. With only the verifier changed, these lanes would have gone from
merged after a red gate to not merged at all. A pin now counts as inside when
the lane named it and the verifier copied it. Nothing in the record argued for
the old limit: §383.3 counts fixes and does not bound them. The merger had
also made exactly these edits already, on two of the five rows above: Q503's
merge `f8b4cc67` re-pins `a_lens_whose_hole_is_half_a_disk.rs`, and Q563's
`be0ddf78` re-pins two KiCad corpus counts in `rcad-step`'s tests, each
outside its lane's write set (`git show --cc` on each). `merger.md` is outside
Q1416's write set, and widening the change to it was this session's call.

`lanes.js` is in the row's write set and did not change. The `fixes` schema
text ("each fix fix_first asks for: the file, the change, and why") and the
`VERIFY` and `MERGE` briefs say nothing the rule contradicts, so all of the
change is in `.md` files and `lines that run` counts 0 (§375).
