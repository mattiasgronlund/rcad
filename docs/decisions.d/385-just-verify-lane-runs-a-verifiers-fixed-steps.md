## 385. `just verify-lane` runs a verifier's fixed steps in one detached run that the verifier follows — 2026-10-03

*Q793 builds what §383.4 item 4 settles: the recipe, the rule on a number's
`value`, `verifier.md` and `VERIFY`. This section records what was built, how
it keeps every call under the harness's ten minutes, and how it is tested.*

In short: the verifier writes the recipe's input to a file and runs `just
verify-lane` on it. That starts one python process, in a session of its own,
which does the verifier's fixed steps in order and ends with one line of JSON
holding every verdict field it could fill. Each call follows the run for at
most 540 s, and while the run goes on, `just verify-lane wait` follows it
again. The verifier takes the fields as they stand and keeps the reading.

### 385.1 What was built

- **`tooling/verify-lane/verify_lane.py`**, run by `just verify-lane`. In
  order, it:
  - unlocks the lane's tree;
  - checks that the branch exists and sits on the base, and prints `git log
    --oneline` between them, the first 12 commits;
  - lists the files outside the write set, a rename counted at both names;
  - runs `lines-that-run`, before the checkout;
  - checks the branch tip out and runs `gate quiet`, relaying all but the one
    line each passing step prints;
  - runs once each named recipe the gate does not run (step 6);
  - reverts the source paths with `git restore --source=<base>`, which also
    removes a file the branch added, runs each named test, and puts the
    branch's files back;
  - runs each number's command on the branch tip, with a bound of 600 s.
- **A named test is a recipe or a nextest name.** `just --show` says which
  names are recipes, and a name made of words and `::` goes to `just t`. It
  leaves anything else, such as a sentence, to the verifier. With the source
  reverted, a test counts as failing only when it ran and failed: nextest's
  exit 100, or a recipe still in the justfile that exits non-zero. Exit 4, no
  test ran, is the revert taking the test out, which Q231's verifier read as a
  failure. Exit 101 is a revert that does not build. Both go to `left_to_you`.
- **It skips the revert** when the lane is `pinsOnly` or lines-that-run counts
  0, and reports `tests_fail_without_fix: false` with the reason in `skipped`,
  as `verifier.md` step 7 asks. When a source path holds test code the branch
  changed, it reverts nothing and names the first such lines, so the verifier
  reverts by hand. Test code is what `lines-that-run` skips as test code: a
  file under `crates/*/tests/` or `fuzz/corpus/`, the items after a column-0
  `#[cfg(test)]`, and a `check-*` recipe's body. The script ports that awk's
  two state machines to python.
- **A number's `value` must be text its command prints.** The recipe looks for
  it in the print with colour codes taken off, and again with runs of
  whitespace squeezed to one space. A value it does not find goes in
  `number_mismatches` with the print's last line, and the verifier reads each
  and keeps or drops it. It does not run a command that names a verb that
  writes, such as a commit, a push, a checkout, a `queue` verb,
  `claim-section` or `mark-finished`, and puts it in `left_to_you`. The list
  is of names, so a write it does not name would run. A verifier re-running
  the numbers by hand had the same exposure, and still reads every command.
- **`numberSchema`** says `value` is text the command prints, exactly as it
  prints it. **`lane.md`** tells the lane the same, with §383.1's "164 of 164"
  as the case: a value only when the command prints it.
- **`verifier.md`** gains "First: `just verify-lane`". The verifier writes the
  input, runs the recipe and waits on it, and takes its fields. It keeps the
  reading: what the branch takes out of a test, the section's claims under
  §16.10, each number the recipe did not find or did not run, a revert by
  hand, and the rows (§384.1). Steps 1 to 7 stay. They describe what the
  recipe does, and what a verifier does by hand in a tree whose justfile has
  no `verify-lane`.
- **`lanes.js`**: `VERIFY` writes the input between two dashed lines, and
  hands over `mise exec -- just verify-lane target/verify-lane.txt` and `mise
  exec -- just verify-lane wait`. `verifyInput` builds the records. `oneLine`,
  now shared with the lander's `stepsProblem`, keeps out a word no line can
  carry: a number or a test name holding one is named in the prompt, for the
  verifier to check by hand.

The input takes §384.2's form: a `+` line between records, each record's name
on its first line and then its words, one to a line. The records are `base`,
`branch`, `lane-tree`, `write-set`, `source-paths`, `pins-only`, `tests`, and
one `number` per number, with its `what`, `value` and `command`. The script
refuses a file it cannot read before it starts anything.

### 385.2 The ten minutes: one detached run, followed in calls

The harness ends a Bash call at 600 s. A served gate takes about a minute. A
cold tree's gate can take the nine minutes the gate's `[doc]` names, and after
it come the revert's rebuild, its test run and every number's command. Two ways
to keep each call under 600 s were weighed:

- **Phases the verifier calls in turn**: start, gate, revert, numbers. No phase
  has a bound of its own. The gate alone can take nine minutes, a revert in
  `crates/` rebuilds everything that depends on the crate, and a lane's
  numbers can add up to more than ten. So a long phase would need chunking
  all the same. And every phase after the checkout would run the branch's copy
  of the script, so a lane that changed `tooling/verify-lane/` would be judged
  by its own change. §382.4 runs `lines-that-run` before the checkout for that
  reason.
- **One run, detached, which each call follows for a bounded time.** This was
  taken. The first call checks the input and the tree. It starts the run with
  `start_new_session`, the `setsid` of `lane.md`'s rule, and prints the run's
  log as it grows, for at most `VERIFY_LANE_WAIT` seconds. The default is 540,
  the same as `lane.md`'s `timeout 540 tail`. A call that runs out of time
  exits 3, and its last line says the run is still going. `just verify-lane
  wait` prints what the log gained since and follows for another 540 s. Once
  the run has ended, a `wait` prints its JSON again.

The run is one python process, started before the checkout. Python reads the
whole file before it runs, so the copy that judges a branch is never one the
branch changed. That is also why `land.py` is python (§384.1). A `wait` after
the checkout runs the branch's copy of the recipe, but a `wait` only prints the
log. If the branch's justfile has no `verify-lane`, the still-going line names
the pid and the log, for `timeout 540 tail --pid=<pid> -f /dev/null` and a
Read.

The run keeps its state in `target/verify-lane/` of the verifier's own tree.
That directory is gitignored, so the checkout leaves it alone. It holds the
pid, the log the calls print, how much of it has been printed, every command's
whole output in `full.log`, and an `exit` file written after the JSON. A second
start in the same tree refuses and points at `wait`. A run whose process is
gone with no `exit` file is reported as dead, with `full.log` named.

How many calls a real cold run takes was not measured.

### 385.3 What it fills, and what it leaves

Its JSON holds the `verdictSchema` fields it can decide, and two of its own:

| field | filled |
|---|---|
| `worktree_path`, `branch_ok` | always; the path is its own tree's toplevel |
| `files_outside_write_set` | once the branch sits on the base |
| `lines_that_run_command`, `lines_that_run` | the command as `linesThatRunCommand` builds it, and its print |
| `gate_summary` | from `=== gate summary ===` to the end |
| `tests_pass_on_branch` | `true` on a passing gate when it can run every name, `false` when a named recipe the gate does not run fails |
| `tests_fail_without_fix` | `true` when every named test ran and failed reverted; `false` when every one passed, or on `pinsOnly` or a count of 0 |
| `number_mismatches` | each value not in its command's print |
| `skipped` | what it skipped and why, for `notes` |
| `left_to_you` | what it could not decide |

`verdict`, `notes`, `fixes`, `unbacked_claims`, `reproduced_mutation`,
`paragraphs_checked` and `moved_text_problems` stay the verifier's.

### 385.4 How it is tested

- **`check-verify-lane-selftest`**, in `gate_steps` and `gate_reusable_steps`,
  has ten arms. Each is a throwaway repository with a lane's worktree on its
  branch and a verifier's at the base. A stub on `VERIFY_LANE_JUST` stands in
  for `just`. Its `gate` prints a summary block, and its `t <name>` runs
  `tests/<name>.sh`, exiting 100 when that fails and 4 when there is none. Its
  `lines-that-run` is the real recipe. The arms are:
  - a test and a recipe that fail with the source reverted, beside a number
    found, a number not found, and a number whose command commits;
  - a revert that takes the new test out;
  - tests that pass reverted;
  - `pinsOnly`;
  - a count of 0;
  - a source path holding changed test code;
  - a lane that names no test;
  - a run longer than one call, followed to its end by `wait`;
  - an input with an unknown record.

  The JSON's fields are held to `verdictSchema`'s, read out of `lanes.js`. Five
  breaks were run on the script, and each failed the selftest in its own arm:
  - exit 4 counted as a failure: `taken-out` says "a revert that took the test
    out gave tests_fail_without_fix true";
  - `lines-that-run` moved after the checkout: `fails-reverted` says both it
    and the gate ran at the lane's tip;
  - every value counted as found: `fails-reverted` says `number_mismatches` is
    `[]`;
  - the skip on a count of 0 removed: `zero` says `tests_fail_without_fix` is
    absent;
  - the skip on `pinsOnly` removed: `pins-only` says "true and (absent)".

  With `tooling/verify-lane/` reverted to the base it fails in its first arm,
  since the script is gone. `check-gate-runs-every-check` counts 56.
- **`check-lanes-workflow`** gains a nineteenth arm and six cuts, 68 in all.
  `VERIFY` holds the start and the wait as lines of their own, `numberSchema`
  and `lane.md` hold the rule on `value`, `verifier.md` holds both commands,
  and each record `lanes.js` writes is one `verify_lane.py` reads. With
  `lanes.js` at the base it fails: "does not hand the verifier just
  verify-lane and its wait".
- **`lanes.js` under node**, outside the gate: parsed inside an async function
  with `export const meta` made `const`, and run with `mise exec
  node@26.10.0 --` against stub agents. Three lanes, one with tests, one
  `pinsOnly` and one naming no test, each went lane, verifier, lander, pruner.
  The input each `VERIFY` wrote, cut from between its dashed lines, parsed with
  `verify_lane.py`'s own reader. A number with a two-line value, one with an
  empty value and a two-line test name were left out of the file and named in
  the prompt.

### 385.5 What this leaves

- **How many calls a real run takes**, cold and served, and how often a
  verifier still ran a step the recipe had filled. Q800 measures §383.6's
  estimates from the same transcripts but writes only §383 and §384.
- **A verifier sent by a `claude` process started before this merge** may
  read the old `verifier.md`, if agent files load when the process starts, as
  a new type does (§384.3). Not tested. Its `VERIFY` prompt still names the
  recipe and says to take its fields, so it runs the recipe all the same.
