## 399. A change that keeps the behaviour has a road to a merge — 2026-10-04

*Queue row Q837. Session `rcad-b1`, finished after a handoff. Some rows change
lines that run and keep what they compute: one of two copies dropped for the
other, a body lifted into a function, a block moved. Such a row had no road to
a merge through the `lanes` workflow, so Q854's dispatcher did its row in its
own tree. Now the dispatcher marks such a lane `sameBehaviour`. The lane names
the tests that reach its change, and it merges when three things hold: those
tests pass on the branch, they pass with the source put back to the base, and
one of them fails when the verifier breaks a line the change adds. This
changed `.claude/workflows/lanes.js`, `tooling/verify-lane/verify_lane.py`,
`.claude/agents/verifier.md` and `lane.md`, the `dispatch-lanes` skill, and the
justfile's `check-lanes-workflow` and `check-verify-lane-selftest`.*

### 399.1 Why Q854 had no road

Q854 (`30a22f66`) dropped the exact sum in `validate.rs`'s
`holds_the_axis_of` and asked `holds.rs`'s `how_the_axis_lies_on` instead.
That sum was the second spelling of one question that §392.4 found. Its
dispatcher reported: "Q854 done by me, not a lane (30a22f66, full gate passed).
Its fix changes no behaviour, so no test can fail without it, and the lanes
workflow can't merge that kind of change." Mattias asked why, and said a
dispatcher should never have to merge a lane.

The dispatcher was right about the workflow. `verified()` in `lanes.js` chose
the steps from the report alone. A lane naming a test that fails without its
change got the tests steps. Every other lane got the moved-text steps, and
`unproven` lets moved text merge only when `just lines-that-run` counts 0.
`just lines-that-run 7ba29e21 30a22f66` prints `lines that run: 25`. So an
honest report, naming no failing test, ends in "the branch changes 25 lines
that run and names no test that fails without them". A report that names a
test anyway is refused at the revert, because the test passes with the source
put back.

The record already knew. §375.5 says a code move is printed as lines that run.
§375.1 says Q601's lift waited for Mattias's hand on 09-29. Q837 asked for this
road, and Q833 was waiting on it.

A probe showed it end to end. It loads `lanes.js` with every agent stubbed to
give a scripted answer, and prints whether the lane merged. Run on `main`'s
`lanes.js`, it refuses a Q854-shaped lane with that message, even when the
verifier says `merge` and reports every proof in 399.2.

### 399.2 What such a change can show

Say the old and the new give the same answer on every input. Then a test that
checks answers passes on both, and a test that fails at the base shows that
the two differ. A test that fails without the change exists only when the
claim is false, so asking for one is asking for the wrong evidence. Here is
what can be shown instead:

1. **The tests pass on the branch.** That is the verifier's gate, as for every
   lane.
2. **The same tests pass with the source put back to the base.** `just
   verify-lane` reverts the source paths as it does for the tests steps, and
   reports `tests_pass_without_change`. Here a failure is the finding: the base
   answered differently on that test's input, so the change altered behaviour,
   and the verdict is `reject`.
3. **One of them fails when a line the change adds is broken.** Points 1 and 2
   hold just as well for a test that never runs the changed lines: every test
   in the workspace passes both. So the verifier changes one line the diff
   adds, so that it still builds and computes a wrong answer, runs one of the
   lane's tests, and reports `mutation_caught`. The test must fail on an
   assertion or a panic. A build error fails every test built against the
   broken code, whether it runs the line or not.
4. **An argument, case by case, that the two agree on every input.** The
   tests show it only for the inputs they feed. `30a22f66`'s commit message is
   the model: it takes each way the old sum and the new `Decision` can answer —
   a product below 2^-969, a sum that overflows, a centre that is not Zero —
   and says what each side gives. The verifier reads the argument against the
   diff. A case it leaves out, or one where the two differ, is a `reject`.

`unproven` holds the printed evidence, points 1 to 3. Point 4 is the
verifier's reading, like its reading of what a branch takes out of a test
(§375.5). When `lines-that-run` counts 0, points 2 and 3 are skipped, as §375
skips the revert: nothing that runs changed.

### 399.3 Why a field, and not an inference

§287.2's argument, unchanged. A lane that names no failing test looks the same
as a lane that changed behaviour and found no test for it. Reading the road
off the report would open it to that lane too. So `lane.sameBehaviour` is set
by the dispatcher from the row, and only the field opens the road.

Two marked lanes are stopped before a verifier is spent on them. One that
names a test that fails without its change has said the behaviour changed. One
that names no test that reaches its change gives points 2 and 3 nothing to
run. `sameBehaviour` beside `pinsOnly` throws when the run starts, and
`verify_lane.py` refuses an input with both: a pins-only lane has nothing in
`src/` whose behaviour it keeps.

An unmarked lane is refused as before, but the refusal now names the road:
"(a row that keeps what they compute goes again marked sameBehaviour, §399)".
`lanes.js` is read fresh at every run, so that message reaches a dispatcher
whose copy of the `dispatch-lanes` skill is older than this section.

### 399.4 What was built

- **`lanes.js`.** `sameBehaviour` joins the header's list of lane fields.
  Both marks on one lane throw. `verified()` picks the kind `same_behaviour`
  from the field alone, with its two stops before the verifier. `unproven`
  asks for `tests_pass_on_branch`, and when lines that run is not 0, for a
  source path, `tests_pass_without_change`, and a mutation that failed
  (`mutationCaught`). A marked lane's report must carry
  `tests_that_reach_the_change`. The verifier's prompt sends the
  `same-behaviour` record and those tests to `just verify-lane`, and spells out
  the steps. The verdict schema gains `tests_pass_without_change` and
  `mutation_caught`. And the unmarked refusal names the road.
- **`verify_lane.py`.** It reads the `same-behaviour` record, and refuses it
  beside `pins-only`. Its revert reports `tests_pass_without_change`: true
  only when every test ran and passed, false when one failed, and left to the
  verifier otherwise. A test the revert took out, or one that did not build, is
  not a pass: Q231's rule, turned round.
- **`verifier.md`.** A `## Same behaviour` section with steps 6 to 9, and the
  `reject` cases in the verdict.
- **`lane.md`.** What a marked lane reports, its argument in the commit
  message, and `blocked`, naming the input, when the change turns out to alter
  an answer.
- **The `dispatch-lanes` skill.** The field, and when to set it. And the rule
  Mattias asked for: **never do a row in your own tree because the workflow
  cannot merge it.** That is a road missing from `lanes.js`, to be filed as a
  2.2 row.
- **§396.4** gets a dated note: Q840's prep answers `sameBehaviour` beside
  `pinsOnly`.

### 399.5 The tests

- **`check-lanes-workflow`'s twentieth arm.** It checks that the steps are
  picked from the field, the three refusals before the verifier, the two
  `unproven` lines for the revert and the mutation, `mutationCaught`'s body,
  that `verifier.md` has a line `## Same behaviour`, and that the unmarked
  refusal names the road. Ten cuts break each in turn: `same-inferred` picks
  the kind from the reaching tests instead of the field, `same-with-pins`,
  `same-names-a-failing-test` and `same-names-nothing` each take out a
  refusal, `same-no-revert` and `same-no-mutation` an `unproven` line,
  `same-any-mutation` lets a mutation that passed count, `same-no-steps` takes
  the heading out of `verifier.md`, and `same-unpointed` takes the road out of
  the refusal. The nineteenth arm's records gain `same-behaviour`, and the cut
  `same-record-unread` renames it in `verify_lane.py`. `mise exec -- just
  check-lanes-workflow` prints "78 cuts, each caught".
- **`check-verify-lane-selftest`** gains three arms. A marked lane whose test
  passes with the source reverted reports `tests_pass_without_change: true`
  and no `tests_fail_without_fix`. One whose test fails there reports `false`
  and says the behaviour changed. An input with both marks is refused before a
  run starts. It prints "in 12 arms". Run from a copy of this tree with
  `origin/main`'s `verify_lane.py` (`5ccdea3c`) put in, it fails at the first
  new arm: "record 6 of target/verify-lane.txt is 'same-behaviour', and the
  records are base, branch, lane-tree, write-set, source-paths, pins-only,
  tests, number".
- **The probe of 399.1.** Neither recipe runs the workflow; the probe does. On
  this branch 11 cases come out as designed. A marked lane with points 1 to 3
  shown merges. It is refused without the mutation, with a mutation that
  passed, with a test that fails reverted, and with no source path. It merges
  at lines that run 0. One that names a failing test, or no reaching test,
  stops before its verifier. An unmarked lane is refused even with every proof
  shown. Both marks throw. On `main`'s `lanes.js` every marked case is
  refused, except the count of 0, which §375 already let through. The probe is
  not in the gate: it needs `node`, which `mise.toml` does not pin, and
  `CLAUDE.md` says never to install a tool ad hoc. It was a throwaway in the
  session's scratchpad.

The end-to-end proof is the first row sent with `sameBehaviour: true`, as
§287.6 said of `pinsOnly`.

### 399.6 What was not chosen

- **The verifier's reading alone** (Q837's first option): the verifier
  compares the bodies, finds them the same, and the gate is the test.
  `unproven` refuses a merge on the verifier's word; see `lanes.js`'s comment
  above it. A reading that two spellings agree is the claim most worth
  checking, not taking. The reading stays, as point 4, beside the printed
  evidence.
- **A differential test**: keep the old body, run the old and the new on many
  inputs, and assert they agree. Where it fits, it is stronger than points 2
  and 3 on the inputs it draws. But it needs the old code kept whole in a
  test: a dropped copy as it was, a lifted body inside a copy of its old
  caller. That copy is the second spelling the row was written to remove. A
  moved block compared with itself shows nothing. And it still covers only the
  inputs it draws, so point 4 is needed beside it. A lane may still add one
  when its argument has a case no test feeds.
- **A refactor rides only with the change that needs it** (Q837's other
  option, which drops Q833). Then two spellings of one question stay until
  some change needs one of them, and they can drift apart meanwhile. That
  drift is what §392.4 filed Q854 against. And a lift so a function reads from
  the top down (Q601) is never needed by any change.
- **Letting a byte-identical move through `git diff --color-moved`** (Q786).
  A lane that only moves code would then count no line that runs. That saves
  one mutation and one reading, for one shape of the three, and no row waits
  on it. Q786 is dropped on this section: a move goes by `sameBehaviour`.

### 399.7 What this leaves

- **One mutation shows that the tests reach one changed line, not each.** A
  change of 25 lines whose tests reach only one of them gets through on that
  one line and the argument. A mutation per changed block would close that,
  at one test run per block. Nothing here measured how many blocks a refactor
  row has, so it is not filed.
- **Q833** waited on Q837 for this road. It is filed again, with the mark in
  its text, so a dispatcher or Q840's prep reads it from the row. Its run is
  the end-to-end proof of 399.5.
- **A session started before this landed has the old `verifier.md` and
  `lane.md`.** A process reads its agent files once, when it starts (§360.3).
  `lanes.js` is read at every run, and its prompt and schemas carry the steps,
  so such a run still does them. Even so, send the first marked row from a
  session started after this landed.
