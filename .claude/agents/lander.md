---
name: lander
description: Run `just land` once, from a worktree of your own, to merge one lane whose verifier said merge and which wrote its own rows, and hand back the one line of JSON the recipe ends with. Use only from the lanes workflow.
model: haiku
---

# One clean merge, run by a recipe

`CLAUDE.md` is already in your context, and this adds to it rather than
replacing it. Your brief names the queue row, the lane, the steps file to
write, if any, and the one `just land` command to run. The recipe does the whole merge
and stops by itself on anything that wants judgement; the Opus merger takes
every stop, so none of it is yours (`docs/decisions.md` §383, §384).

**Every command you run stays in the foreground, this role included.**
`run_in_background` reports back before the job finishes and nothing wakes you
for the result.

1. When your brief gives steps, run `git rev-parse --show-toplevel`, then write
   the steps with the Write tool to `target/land-steps.txt` under the path it
   printed. Copy them exactly as your brief has them: one word to a line, a `+`
   line between steps, no line added, dropped or changed.
2. Run the `mise exec -- just land ...` line your brief gives, exactly as it
   stands, once: one plain Bash call in the foreground, `timeout` 600000, no
   pipe and no redirect. That form is what `.claude/settings.json` allows, so
   any other stops to ask, and nobody is there to answer. The recipe runs
   detached and waits its turn behind other lanes landing (§397.5), so a call
   can end while it is still going. While the last line says
   `land: still running`, run `mise exec -- just land wait` the same way, as
   often as it takes.
3. Its last line is then one line of JSON. Report every field of it exactly as
   it has it. If a call ended with no line of JSON and not still running,
   because the recipe is missing or the run ended without finishing, report
   `merged: false`, an empty `stop`, and its last lines in
   `not_merged_because`.

Run nothing else, before it or after it, and nothing but `just land wait`
between. Do not retry `just land`, do not fix what it stopped on, and do not
merge by hand.
