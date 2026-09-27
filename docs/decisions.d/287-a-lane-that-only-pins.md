## 287. A lane that only pins can merge — 2026-09-26

*Queue row Q208. `lanes.js`'s `unproven` refused to merge a writing lane that
named a test but had no `src/` path to revert -- a verdict, a pinned test and
a decisions section, nothing in `src/` in the write set. Three rows shaped
exactly that way, Q126, Q127 and Q203, came back "verifier said merge" and
were merged by hand each time. `lane.pinsOnly: true` is now the explicit way
past that refusal, checked in `.claude/workflows/lanes.js`, in the verifier's
prompt it builds, and in `.claude/agents/verifier.md`. It is never inferred
from an empty `sourcePaths`, and a `pinsOnly` lane that does declare a `src/`
path -- in its write set or in `sourcePaths` -- is refused too, since there is
then something to revert. No `src/` changed; this is a lane of the seventh
kind, a change to `.claude/workflows/lanes.js`, `.claude/agents/verifier.md`,
`.claude/skills/dispatch-lanes/SKILL.md` and the `justfile`'s
`check-lanes-workflow` recipe.*

  287.1   The three rows that got stuck
  287.2   Why a field, and not an inference
  287.3   The change in `unproven`
  287.4   The verifier's prompt and `verifier.md`
  287.5   The test
  287.6   What this leaves

### 287.1 The three rows that got stuck

`unproven` (`.claude/workflows/lanes.js`), for a lane whose report names
failing tests, asked for two things: a verifier's word that the tests fail
without the fix (`tests_fail_without_fix`, or a `reproduced_mutation`
`mutationProven` accepts), and a non-empty `sourcePathsOf(lane)` -- the
directory the verifier reverted to get that word. `sourcePathsOf` defaults to
the `crates/<c>/src` dirs in the lane's write set, or an explicit
`sourcePaths`.

Q126 (`lane-q126-apex-cells`), Q127 (`lane-q127-rim-start-order`) and Q203
(`lane-q203-plate-glue-sweep`) each wrote a `tests/*.rs` file pinning a
refusal and a `docs/decisions.d/*.md` section, and each said in its brief not
to touch `src/`: the fix belongs to whoever owns the file next (283.5, 284.6).
Their write sets held no `src/` path, so `sourcePathsOf` returned `[]` for all
three, `unproven` always added "the lane declared no source directory to
revert", and the script refused every one of them regardless of what the
verifier found. All three verifiers said `merge`; Mattias merged all three by
hand. `docs/queue.md`'s Q208 row names this pattern.

### 287.2 Why a field, and not an inference

The brief this section answers considered inferring the pins-only case from
the write set holding no `src/` path, and rejected it. That inference reads
the same shape a *mistake* produces: a tooling lane whose dispatcher forgot to
pass `sourcePaths` by hand -- exactly the case `sourcePaths defaults to the
crates/<c>/src dirs in writeSet; set it by hand when that is not what the
verifier should revert` (the `dispatch-lanes` skill, step 5) already covers
for a row like this one, where the file to revert is `justfile` or
`lanes.js` itself and no `crates/<c>/src` default exists. Today that lane is
refused, and refusing is the safe side: a merge that never watched anything
fail is a merge on the verifier's word, which is the thing `unproven` exists
to keep from happening (`lanes.js`'s own comment above `unproven`). Reading an
empty `sourcePathsOf(lane)` as license to skip the revert would wave through
both shapes alike and lose that safety for the one that still needs it.

So `lane.pinsOnly` is a field the dispatcher sets by hand, once, for a row it
already knows has nothing under `src/` to revert -- the same judgement it
already makes for `sourcePaths` itself. Setting `pinsOnly` and also declaring
a `src/` path -- in `writeSet` or in `sourcePaths` -- is refused rather than
accepted: there is something to revert then, so the lane is not pins-only, and
the field does not override that.

### 287.3 The change in `unproven`

```js
function unproven(lane, kind, check) {
  const why = []
  if (check.files_outside_write_set && check.files_outside_write_set.length) why.push(...)
  if (kind === 'tests') {
    if (check.tests_pass_on_branch !== true) why.push('the tests do not pass on the branch')
    const sources = sourcePathsOf(lane)
    if (lane.pinsOnly === true && sources.length) why.push('the lane is marked pinsOnly, but it declares a source directory to revert -- there is something to revert, so pinsOnly does not apply')
    const pinsOnly = lane.pinsOnly === true && !sources.length
    if (!pinsOnly) {
      if (check.tests_fail_without_fix !== true && !mutationProven(check.reproduced_mutation)) why.push(...)
      if (!sources.length) why.push('the lane declared no source directory to revert')
    }
  }
  return why
}
```

`files_outside_write_set` and `tests_pass_on_branch` are checked exactly as
before, for every lane. The local `pinsOnly` is `true` only when the
dispatcher set the field *and* there is nothing to revert; only then are the
revert-or-mutation check and the no-source refusal skipped. Every other lane
-- `pinsOnly` unset, or set beside a declared source -- goes through the same
two checks the script always ran, so a lane that does not name the field gets
exactly the refusal Q126, Q127 and Q203 got.

### 287.4 The verifier's prompt and `verifier.md`

`VERIFY`'s prompt (`.claude/workflows/lanes.js`) now appends a paragraph when
`lane.pinsOnly === true`, right after "Source paths to revert:", telling the
verifier to skip the revert, run the lane's new tests on the branch, and
report `tests_fail_without_fix: false` with no `reproduced_mutation` -- so the
schema's required fields are still satisfied, honestly, rather than by a
revert that has nothing to act on. `.claude/agents/verifier.md`'s step 7 and
its frontmatter description say the same: a declared-empty source list is
expected and not a defect only when the brief marks the lane `pinsOnly`, and
is the lane's own gap otherwise.

### 287.5 The test

`check-lanes-workflow`'s tenth arm checks four exact lines survive in
`.claude/workflows/lanes.js`: the one that gates the local `pinsOnly` on
`sourcePathsOf(lane)` being empty (so the field cannot be swapped for that
inference), the refusal a lane with no field and no source still gets, the
refusal a `pinsOnly` lane with a declared source gets instead, and the line
in `VERIFY`'s prompt that carries the field to the verifier. Four cuts prove
each: `pins-inferred` replaces the gate with `!sources.length` alone (no
field read at all) and is caught by the first; `pins-no-refusal`,
`pins-no-conflict` and `pins-verify-missing` each remove one of the other
three lines and are caught in turn. `mise exec -- just check-lanes-workflow`
reports "17 cuts, each caught" on this branch. Since none of the four lines
existed in `.claude/workflows/lanes.js` before this row, the same arm fails on
`main` at `ff78f99f` -- the base this row starts from -- with "does not gate a
pins-only merge on there being nothing to revert", which is the "fails
without the change" half of `CLAUDE.md`'s second principle for a lane that
touched no `src/`: every paragraph and every line this row moved or added is
what the arm now depends on, and `check-lanes-workflow` traces them the same
way the eight arms before it are traced (§237, §266.7).

A textual arm cannot run the workflow, so it cannot show a real Q126-shaped
lane merging end to end; the first row dispatched with `pinsOnly: true` is
that demonstration, the way §266.6 named the first real `lanes` run as the
proof that `agentType` loads the agent files.

### 287.6 What this leaves

- The first row actually dispatched with `pinsOnly: true` is the end-to-end
  proof this section's arm cannot be (287.5).
- `docs/queue.md`'s Q208 row is closed by this lane's merger, in the same
  commit as this section, per §172.5.
