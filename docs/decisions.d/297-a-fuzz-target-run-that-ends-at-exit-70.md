## 297. A `fuzz-target` run that ends at exit 70, on its own — 2026-09-27

*Queue row Q188, closing what Q180 left open in §275.5: rerun `mise exec --
just fuzz-target where_two_surfaces_cross 120 <scratch-root> "" 3` against the
corpus root Q180 built (388 files, 322 of them `timeout-*`), and this time let
it run to its own end instead of killing it. Lane
`lane-q188-a-run-that-ends-at-70`, off `531ca665`.*

Q180's corpus root (`/tmp/.../4554e28e.../scratchpad/q180/corpus-root`)
survived the reboot the row worried about: still there, still 388 files, 322
of them `timeout-*` (`find ... -name 'timeout-*' | wc -l`). Copied to this
lane's own scratchpad rather than pointed at directly, so Q180's copy stays
exactly as it was. `where_two_surfaces_cross` was rebuilt once, unpinned
(`cargo +nightly fuzz build`, `Finished release profile ... in 1m 08s`), so
the measured run below never pays a build cost inside its `taskset`. No night
was running (`uptime`: `load average: 1.09, 0.46, 0.27`); the run itself was
pinned throughout to cores 14-17 (`taskset -pc` on the orchestrator's pid
during the run: `current affinity list: 14-17`), launched detached
(`setsid nohup ./target/q188-run.sh 1 &`) and never killed.

**One run was enough.** Wall time, from the log file's birth to its last
write (`stat`): 17:30:43.18 to 18:28:10.41 local, 57 minutes 27 seconds. Its
tail, verbatim:

```
INFO: -fork=3: 580 seed inputs, starting to fuzz in /tmp/libFuzzerTemp.FuzzWithFork672222.dir
#22: cov: 3139 ft: 9153 corp: 580 exec/s: 0 oom/timeout/crash: 0/0/0 time: 3400s job: 3 dft_time: 0
INFO: fuzzed for 3400 seconds, wrapping up soon
INFO: exiting: 70 time: 3430s

────────────────────────────────────────────────────────────────────────────────
[111 "Failing input:" blocks, one per file already sitting in
fuzz/artifacts/where_two_surfaces_cross/ — cargo-fuzz's own post-mortem
listing of that gitignored directory, the same listing §275.5 saw after its
SIGKILL, this time following a real exit rather than an external kill]
────────────────────────────────────────────────────────────────────────────────

Error: Fuzz target exited with exit status: 70
error: Recipe `fuzz-target` failed with exit code 1
RECIPE_EXIT=1
```

`INFO: exiting: 70` printed on its own, from the process's own `exit(70)`,
with no kill anywhere in this run's history — `Error: Fuzz target exited with
exit status: 70` names an exit status, not a signal, unlike §275.5's `exited
with signal: 9 (SIGKILL)`. `RECIPE_EXIT=1`, matching §275.2's "always 1 on any
target failure, never the child's own code": the 70 reaches the log, never the
shell.

The 580 seed inputs are the merge survivors — up from the 497 that §275.5's
first run, seeded with 25 slow files rather than 322, kept (`Merge kept 497 of
the 834+25 candidates`); its second run was killed before printing any such
count — and the single
stats line's `time: 3400s` covers the whole span from "starting to fuzz" to
the wrap-up call, not one job in isolation; this run's 322 slow files, each
capped at the `-timeout=10` alarm rather than skipped, are the likely reason
that span is nearly the entire 57 minutes rather than anywhere near the
recipe's nominal `-max_total_time=120`, in the same direction as §275.5's own
reading but not reconfirmed by a fresh mechanism check here — this section
only claims the number it measured.

**Q188, settled**: a natural `fuzz-target` run does end with `INFO: exiting:
70` in its own log and the recipe exiting 1, exactly as §275.5 predicted it
would take "one or two more attempts" to catch. It took one.
