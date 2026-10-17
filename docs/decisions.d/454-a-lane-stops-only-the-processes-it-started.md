## 454. A lane stops only the processes it started — 2026-10-17

*Dispatch 161 (session `clean-up-dead-sessions-6f`), after Q1626's lane killed
Q2240's land gate in another tree. Decided under `CLAUDE.md`'s "Decide it
yourself". Adds to the rule on detached runs in `.claude/agents/lane.md`
(§302).*

**In short.** Before you start a run you may have to stop, write down its pid.
Stop it with `kill <pid>`, or `kill -- -<pgid>` for a `setsid` group. Never use
`pkill`, `killall`, or any kill that picks its targets by matching a pattern.
Every tree on this machine runs the same recipes with the same command lines,
so a pattern written for your run matches other trees' runs too. A 2.2 row puts
a refusal of those forms in the Bash hook and this rule in the three role files.

### 454.1 What happened

At 19:11:14.216Z on 2026-10-17, Q1626's lane (run `wf_ba5ca889-895`, a
read-only Sonnet lane in `wf_ba5ca889-895-2`) wanted to stop its own
`-p rcad-topo` test run. It ran:

```
kill %1 2>/dev/null; pkill -9 -f "cargo-nextest nextest run --workspace.*rcad-topo" 2>/dev/null; pkill -9 -f "a_plane_cut_along_a_cones_other_nappe" 2>/dev/null
```

At that moment Q2240's lander was gating in another tree,
`wf_ff47a015-7eb-4`. The gate's `test` step runs `cargo nextest run
--workspace -E "not ($filter)"`, and `_slow-tests-filter` puts every name in
`slow_tests` into `$filter` as
`binary_id(=rcad-topo::a_plane_cut_along_a_cones_other_nappe) & test(=…)`.
Both patterns matched that command line. The lander's `62-test.log` ends at
test 4280 of 4281:

```
… 2698847 Killed                  cargo nextest run --workspace -E "not ($filter)"
error: Recipe `test` failed with exit code 137
```

`just land` stopped at `gate: FAIL`. The run then sent the row to the Opus
merger, which gated again (PASS, 4281 tests) and merged it at `95868d0f`.

How the cause was confirmed:

- The log was last written at 19:11:14Z, the same second as the `pkill`.
- `journalctl` has no kernel OOM kill and no systemd-oomd kill between 19:09Z
  and 19:14Z.
- The `test` recipe sets no timeout.
- The lane's command is in its transcript,
  `ec412a46-2223-4302-b037-6644204eccc3/subagents/workflows/wf_ba5ca889-895/agent-a1da7d9537ff5df4f.jsonl`
  under this project's transcript directory.

### 454.2 Why one lost land is not the cost

A lost land costs one more gate, and the Opus merger paid it. The expensive
case is a verifier. A verifier reverts the source and runs a test to see it
fail. A test killed from another tree also comes back failed. A mutation then
reads as caught, a revert as proving the test, and nothing in the verdict looks
wrong.

Up to four slots run at once (§447.3), beside Mattias's own sessions and the
fuzz night. They all run the same recipes, so they all have the same command
lines. A pattern that names a test, a binary or a recipe will match more than
one tree some of the time.

### 454.3 The rule

- **Write down the pid of any run you may stop.** `lane.md` already starts a
  run that cannot be chunked with `setsid nohup … &`; add `echo $!` and keep
  the number. `setsid` makes that pid the leader of a process group of its
  own, so the group id is the same number.
- **Stop it by number:** `kill <pid>`, or `kill -- -<pgid>` for the whole
  group. Send SIGKILL only to the same pid or group, and only after SIGTERM.
- **When the pid is lost**, run `pgrep -af <your tree's absolute path>`, read
  the list, and kill the pids you read, by number. A process whose command line
  does not name your tree is not yours.
- **Never** `pkill`, `killall`, `kill $(pgrep …)` or `pgrep … | xargs kill`:
  any form where a pattern picks the targets at the moment of the kill.

A misread pid can still kill the wrong process, and no hook can catch that.
What the rule takes away is the kill that hits another tree without anyone
reading a list.

### 454.4 What the row builds

- **The Bash hook refuses those forms.** The judgement goes in
  `tooling/queue/guard.py`, which `.claude/hooks/queue-write-guard.sh` already
  hands every Bash call to. A second hook would need a `settings.json` entry,
  and this one already sees every call. The refusal names §454 and the forms
  that are allowed. The hook's cases, `.claude/hooks/queue-write-cases.json`
  (run by `check-hooks`), hold each refused form. They also hold the forms that
  must pass: `kill <pid>`, `kill -- -<pgid>`, `pgrep -af <path>` on its own,
  and the word `pkill` inside a grep pattern or a quoted string. A guard that
  refuses a mere mention gets routed round, as `guard.py`'s own docstring says
  of §263.
- **`lane.md`, `verifier.md` and `merger.md` carry 454.3**, in `lane.md` next
  to the `setsid nohup` rule.

Mattias can still type `pkill` in a `!` command, which no hook sees.
