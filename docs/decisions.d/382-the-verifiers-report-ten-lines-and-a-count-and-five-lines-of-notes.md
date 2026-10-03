## 382. The verifier's report: ten lines and a count, and five lines of notes — 2026-10-03

*Queue row Q755, with two changes Mattias asked for folded in. Each agent in a
lanes run ends on a turn that writes its report, and those turns are 6.5% of a
run. Mattias asked whether that could be cut. Once it was measured, he said
"Yes do it" to two changes: cap the lines-that-run print at ten lines, and
hold the verifier's `notes` to five. Q755 was already queued for the same
files: two verifiers had been refused over how they copied the lines-that-run
evidence back. This section changes `.claude/workflows/lanes.js`,
`.claude/agents/verifier.md` and the `justfile`. It also changes what counts as
§375's evidence: a count of 0, where it used to be an empty print.*

In short: the awk program that prints the lines that run has moved out of
`lanes.js` into `just lines-that-run <base> <branch>`. The recipe prints the
first ten lines, then always one last line, `lines that run: N`. `unproven`
reads N off that line, and N = 0 is what lets a lane skip the revert. A
verifier now types and copies back a command of about 110 characters instead of
about 1.6 KB, and at most eleven lines of print instead of up to 199. There is
no empty print left to report wrongly. `notes` is held to five lines for the
merger, in both the schema and `verifier.md`. The expected saving is about 28 s
per verifier, about 400 s over the 14 runs measured. That would take the report
turns from 6.5% of a run to about 4.8%. It is an estimate and has to be
measured again (§382.7).

### 382.1 What a report turn costs

The 14 lanes runs of 2026-10-03, Q740 to Q774 (`wf_4373c43c-235`,
`wf_958d31d2-1dc`, `wf_70e2e48c-3cb`, `wf_686fd35f-792`, `wf_47249d27-a88`,
`wf_e95cb3a9-ea6`, `wf_cabf2caf-0b3`, `wf_0965d880-86f`, `wf_4aa64a95-1c6`,
`wf_4dba94b5-1a1`, `wf_65d9ee08-4c2`, `wf_0feaf8af-cd7`, `wf_c84930ee-141`,
`wf_cfa0186e-82d`). For each agent, its last turn runs from its last tool
result to its `StructuredOutput` call. That turn is split into thinking and
writing by the timestamps of the records in it, and each field of the output is
sized. The script was `report_turn.py`, in session
`d3990082-da68-4d7a-a60e-7a4d1531bf89`'s scratchpad under `/tmp`, which a
reboot clears. This paragraph is enough to rebuild it.

- All 14 runs: 1,520 s of 392 minutes, 6.5%. Of that, 1,130 s is writing and
  390 s thinking. Writing runs at about 270 characters a second, so each KB of
  report costs about 4 s.
- Mean last turn per role, thinking plus writing, and report size: lane 41 s
  (9 + 32), 8.7 KB; verifier 50.5 s (10 + 40), 10.8 KB; merger 9 s, 1.8 KB;
  pruner 12.5 s, 11 of it thinking, 0.5 KB.
- The verifier's fields, mean size: `lines_that_run` 4,283 characters (40%),
  `notes` 3,858 (36%), `lines_that_run_command` 1,226 (11%), `gate_summary`
  662. `lines_that_run` ran from 0 to 199 lines, up to 13.7 KB, and was over 20
  lines in 8 of the 14 runs. The `notes` were step-by-step accounts of what the
  other fields already held: Q766's in `wf_0feaf8af-cd7` and Q767's in
  `wf_c84930ee-141` are typical.

### 382.2 The print: ten lines, then the count

Nothing read past the first line. `unproven` asked only whether the print was
empty, and quoted its first line in a refusal. The merger acts on the verdict.
The other 198 lines of a 199-line print reached nobody, and the verifier spent
about 4 s a KB writing them.

The recipe now prints the first ten lines that run, then `lines that run: N`
with the full count. The count line is always printed. The one exception is
when `git diff` printed nothing at all: then the recipe prints §375's warning
and exits 1. With no count line, `unproven` refuses the lane. `verifier.md`
still says never to drop a line the command printed. That stays true, since the
command now prints less.

Saving: about 3.3 KB a verifier, about 12 s.

### 382.3 `notes`: five lines, for the merger

`notes` reaches the merger inside the verdict, and the dispatcher in the row
`handedBack` writes. Neither needs the verifier's steps again, because the
other fields already hold them. The schema's description and `verifier.md`'s
section on the verdict now both say: at most five lines, what the merger must
act on and what no other field says. Each note that `verifier.md` asks for
still fits in a line: a line the command missed, the count of 0 as the reason
a revert was skipped, and pinsOnly as the reason a revert was skipped.

The bound is written in the description, not as a `maxLength`. A hard limit
would make the whole verdict fail validation over the length of a note, and the
verdict is what the run needs.

Saving: about 3.3 KB a verifier, about 12 s.

### 382.4 Q755: a recipe, and a count instead of an empty print

Two verifiers said `merge` and `lanes.js` refused them over how they copied the
evidence back. Q708's first run (`wf_70e2e48c-3cb`) reported an empty print as
the two characters `""`, which `unproven` read as a line that runs. Q753
(`wf_e95cb3a9-ea6`) put a paraphrase in place of the awk in
`lines_that_run_command`, so the comparison with the real command failed, even
though all five named tests had failed under its revert.

The command is now `mise exec -- just lines-that-run <base> <branch>`. That is
short enough to copy exactly, and `unproven` still compares it with the one it
handed over. §375 had put the program inline because the verifier's Bash guard
refuses `awk -f`. A `mise exec -- just` call is what the verifier already runs
for its gate. Not having to type the 1.6 KB command in its step-1 Bash call
saves more time again, outside the last turn and not counted in §382.1.

Q755 suggested a byte count reported beside the text. That was not built. The
verifier would have to work the number out itself, and that is one more thing
it can get wrong. The recipe's own count, copied with the print, says the same
thing. Because the count line is always there, the print is never empty, so
there is nothing for `""` to stand in for. `unproven` takes N only from the
print's last line, and only in the exact form `lines that run: N`. Anything
else is refused, the same as a print that was never reported.

The verifier runs the recipe in step 1, before step 2 checks the branch out. So
the recipe that judges a branch is the one in the dispatcher's tree, never a
copy the branch changed. A branch that does change the recipe shows up in the
print: `lines-that-run` is not a `check-*` recipe, so changes to its body count
as lines that run.

Saving: the command copy, about 1.1 KB, about 4 s.

### 382.5 What was left alone

- **The lane's report**, 8.7 KB and 41 s. The verifier checks every entry in
  `numbers` against its command, and `remaining` becomes queue rows. Both are
  read.
- **The merger**: already 9 s.
- **The pruner**: about 2 minutes over 11 runs, almost all of it thinking.

### 382.6 Tests

`check-lanes-workflow`, 50 cuts, each caught by the arm it was made for.

- **Arm 16** now cuts the awk program out of the recipe instead of `lanes.js`.
  After the fixture's seven lines it wants `lines that run: 7`. It pins the
  recipe's `git diff` line with every flag, the command `lanes.js` hands over,
  and the line in `unproven` that reads the count. New cuts: `runs-uncounted`,
  `runs-count-loose` and `runs-diff-narrow`. The four older cuts of the program
  now cut the recipe.
- **Arm 17** is new. On a second repo, whose one commit adds fifteen functions,
  it wants the first ten lines and then `lines that run: 15`. It also checks
  that the schema's `notes` and `verifier.md` both carry the five-line bound.
  Its cuts are `runs-uncapped`, `notes-unbounded` and `notes-unsaid`.

The recipe was also run by hand. On Q781's merge (`ef133a82..a6affd27`) it
printed `lines that run: 0`. On `21211e81`, the commit that added `pre-gate`,
it printed ten lines and `lines that run: 69`. On an empty diff it printed the
warning and exited 1. `linesThatRun` was checked under node 26.10 on ten
prints: 0, 1 and 15 counted, and `""`, an empty string, a missing field, the
warning, a count that is not the last line, and a count with words after it all
refused. The repo pins no node, so that check is not a gate step.

### 382.7 What this leaves

- A dispatcher gets the new `lanes.js` only by launching it with `scriptPath`.
  It gets the new `verifier.md` only in a `claude` process started after this
  lands, because neither `/clear` nor a merge reloads agent types. A verifier
  with the old `verifier.md` and the new `lanes.js` has two rules for skipping
  the revert: its brief says on a count of 0, and its file says on an empty
  print. `unproven` decides on the count either way, so the worst outcome is a
  revert it did not need. On a moved-text lane, though, the old file says the
  print must be empty to merge, and the print is never empty now. That verifier
  can refuse a lane that should merge. It refuses; it never merges one it
  should not.
- Measure the saving again over about ten runs with the method of §382.1. The
  28 s a verifier is an estimate from report sizes, not a measurement.
- Q741, Q742 and Q743 named `LINES_THAT_RUN` in `lanes.js` and had no write
  access to the recipe. They are dropped and added again as Q785, Q786 and
  Q787, with `justfile#lines-that-run` in their write sets.
