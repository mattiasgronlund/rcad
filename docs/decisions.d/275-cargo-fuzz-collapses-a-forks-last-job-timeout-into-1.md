## 275. `cargo-fuzz` collapses a `-fork` run's last-job timeout (70) into a flat 1, so `fuzz-overnight` never sees the 70 that `FuzzerFork.cpp` gives

*Q159. The row: "A -fork run exits with its last job's code, so arm C of §264
exited 70 with -ignore_timeouts=1 set (FuzzerFork.cpp:488-491). Find out
whether cargo-fuzz passes that 70 on, and what fuzz-target under set -euo
pipefail and fuzz-overnight then do with it: whether a night that timed out
last is reported as a failure or stops the targets after it." §264.5 §264.8.*

  275.1   `cargo fuzz run` always exits 1 on any target failure, never the child's own code
  275.2   `fuzz-target`'s own exit code is exactly cargo-fuzz's, unchanged by `set -euo pipefail`
  275.3   What reaches `fuzz-overnight`'s `wait`, and what it does with it
  275.4   Verdict: §264.5's "not checked" is now checked, and it changes the picture
  275.5   Q180: witnessing `fuzz-target` itself end on a last-job timeout — close, not clean

### 275.1 `cargo fuzz run` always exits 1 on any target failure, never the child's own code

`cargo-fuzz`'s `exec_fuzz` (installed version 0.13.2,
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cargo-fuzz-0.13.2/src/project.rs:538-598`)
spawns the built fuzz binary as a child, `wait()`s on it, and when
`!status.success()` calls `bail!("Fuzz target exited with {}", status)` at
line 598 — an `Err`, not a passthrough of `status.code()`. `main()`
(`src/main.rs:169-171`) is a plain `fn main() -> Result<()>`, so any `Err`
becomes Rust's default `Termination` for `Result`: the message is printed and
the process exits **1**, whatever `status` held.

Run, not read: built `where_two_surfaces_cross`
(`mise exec -- cargo +nightly fuzz build where_two_surfaces_cross`), then, with
a scratch corpus root so nothing tracked was touched:

```
cargo +nightly fuzz run where_two_surfaces_cross <scratch-corpus>/where_two_surfaces_cross corpus/where_two_surfaces_cross -- \
    -max_total_time=30 -fork=1 -timeout=1 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1
```

`-fork=1` makes its one job trivially the "last" job. Fuzzing found several
inputs over the 1-second `-timeout`; the log shows libFuzzer's own line
`INFO: exiting: 70 time: 35s` — the exact `FuzzerFork.cpp:488-491` exit —
followed by cargo-fuzz's `Error: Fuzz target exited with exit status: 70`.
The whole `cargo fuzz run` *process* nonetheless exited **1**, confirmed by
echoing `$?` right after the redirected run: `EXIT_STATUS=1`.

### 275.2 `fuzz-target`'s own exit code is exactly cargo-fuzz's, unchanged by `set -euo pipefail`

`fuzz-target` (`justfile:4144`) ends with that same
`cargo +nightly fuzz run ...` invocation as its last statement — nothing
after it, no pipe — so under `set -euo pipefail` the recipe's own exit status
is simply whatever that last command returned; `-e` has nothing left to act
on. Confirmed generically with a throwaway recipe of the same shape
(`target/exit-code-check.justfile`, script body `set -euo pipefail; exit
{{code}}`): `mise exec -- just -f target/exit-code-check.justfile
last-stmt-fails 70` returns `$? = 70` through `just` unchanged. Combined with
§275.1: whenever the built target fails for any reason libFuzzer can raise —
crash, oom, or a `-fork` run whose last job timed out — `fuzz-target`'s own
exit code is always cargo-fuzz's flat **1**, never the specific 70 (or any
other libFuzzer code) the child gave.

### 275.3 What reaches `fuzz-overnight`'s `wait`, and what it does with it

`fuzz-overnight`'s per-target `case "$status" in 124|137) ...; *) ...esac`
(the loop after `recipes=({{fuzz_all}})`) is commented as if a real 70 can
land in that `*` arm alongside other values; §275.2 says it never can —
`wait "${pids[$i]}"` observes `just <recipe>`'s own exit code, which is
cargo-fuzz's flat 1 (or 0) for every one of those cases, indistinguishably.
That does not matter to the arm's own logic, which does not switch on the
number at all: it greps the target's log for libFuzzer's own trailer line,
`grep -q '^INFO: exiting:'`. That line is written by the *child* process
before cargo-fuzz decides its own exit code, so it is present exactly when
the fork run finished on its own (cleanly, or via an ignored last-job
timeout) and absent when the target's launch failed outright — which is what
the arm is actually distinguishing.

Verified with a stand-in rather than a real night, so as not to touch
`rcad-nightly-fuzz` or spend a night's window (`target/fuzz-overnight-standin.sh`
and `target/fake-fuzz-recipe.sh`, both scratch scripts under `target/`,
not part of this row's write set and not committed). It copies
`fuzz-overnight`'s own start-loop / wait-loop / case-statement verbatim, with
three fake "recipes" in place of the real five: `timeout-70` (exits 70 having
printed `INFO: exiting: 70`, modelling the last-job-timed-out case), `clean`
(exits 0), and `silent-crash` (exits 1, printing nothing of the sort,
modelling a launch failure). Running `./target/fuzz-overnight-standin.sh`
gives:

```
--- all 3 stand-in targets started before any wait: timeout-70 clean silent-crash
--- timeout-70 finished with raw exit status 70
--- clean finished with raw exit status 0
--- silent-crash finished with raw exit status 1
silent-crash: never got fuzzing (exited 1) — see ./target/standin-night/silent-crash.log
=== to_look_at: silent-crash:no-run
```

Only `silent-crash` — the one with no `INFO: exiting:` line — lands in
`to_look_at`. `timeout-70` does not, and would not either if its status were
1 instead of 70: both values fall into the same `*` arm, and the grep never
inspects the number. And because all three are started in the same `for`
loop, before any `wait` runs, one target's exit code cannot stop the targets
after it — `fuzz-overnight` already launches all five before it waits on the
first.

### 275.4 Verdict: §264.5's "not checked" is now checked, and it changes the picture

§264.5's and §264.8's open question — "whether cargo-fuzz passes the 70 on" —
is answered: **no**. `cargo fuzz run` never returns the libFuzzer child's own
code; `exec_fuzz`'s `bail!` on any non-zero child status turns every failure
into an `anyhow::Error`, and `fn main() -> Result<()>` turns every `Err` into
a flat process exit of 1. So the `FuzzerFork.cpp` "exit with the last job's
own code" fact that made arm C's *child* process exit 70 is real and
reproducible (§275.1), but invisible one layer up: `fuzz-target`, and hence
`fuzz-overnight`, see cargo-fuzz's own 1, never 70, for that case, for a
crash, or for an oom alike.

This does not misreport a night that timed out last. `fuzz-overnight`'s `*`
arm never actually keyed on the number 70 — only on `grep -q '^INFO:
exiting:'` — and that line survives cargo-fuzz's translation unchanged
(§275.3), because it is the child's own stdout, written before cargo-fuzz
decides its own exit code. A night whose last `-fork` job timed out is
**not** reported as a failure, and the targets after it are unaffected —
they already started before any `wait` ran.

What is inaccurate is the *reasoning* written down, not the *behaviour*:
`fuzz-overnight`'s own comment above the case statement ("70 is a timeout in
the last job of forty thousand ... **Under `-fork` the exit status says
nothing about the night**") and §264.5's framing both describe a value — the
literal 70 — reaching a layer of the pipeline where it can never actually
arrive. The classification the code performs is still the right one; the
comment justifying it names a number `$status` cannot hold at that point.

Left for whoever next touches `fuzz-overnight` (Q148, while it runs, per this
row's brief — nothing here was fixed): reword that comment to say `$status`
there is cargo-fuzz's own 0/1, not libFuzzer's fork exit code, so a future
reader does not go looking for a 70 that cannot land in that variable.

### 275.5 Q180: witnessing `fuzz-target` itself end on a last-job timeout — close, not clean

*Q180. §275.2 composed its `-timeout=10` claim from `exec_fuzz`'s source plus
a `-timeout=1` run (§275.1's, done by hand, not through the recipe). The row
asked for the thing itself: run `just fuzz-target`, with its hardcoded
`-timeout=10` under `-fork`, end on a last job that timed out, and show the
recipe exiting 1 with `INFO: exiting: 70` in its log — off 21:50–06:00Z.*

A prior Q180 lane found the obstacle first: a real `just fuzz-target
where_two_surfaces_cross 150 <scratch> 71 1` with two slow (`timeout-*`)
seeds in the scratch corpus ran ten fork jobs at oom/timeout/crash 0/0/0 and
exited 0. The dispatcher's hypothesis for why: a `-fork` run's first step
merges the seed corpus and keeps only inputs that add features (a); the
tracked 834-file corpus already covers what the slow inputs reach, so the
merge drops them (b); a job only ever sees inputs that survived that merge
(c). If all three held, seeding more slow files would not help.

**(a) and (c), read from `FuzzerFork.cpp` (libfuzzer-sys 0.4.13, the copy
under `~/.cargo/registry`), both hold.** `FuzzWithFork`'s startup calls
`CrashResistantMerge(Env.Args, {}, SeedFiles, &Env.Files, Env.Features, ...)`
with `Env.Features` freshly empty (`FuzzerFork.cpp:346-354`) — a coverage
minimizing merge over every file in every `CorpusDirs` entry, `SizedFile`
sorted ascending by byte size (`FuzzerIO.h:90`), keeping a file only when it
adds a feature no earlier (smaller) file already claimed. That is (a).
`CreateNewJob` (`FuzzerFork.cpp:120-197`) then draws each job's
`-seed_inputs` only from `Env.Files` — the survivors of that merge and of
each later `RunOneMergeJob` — never straight from `CorpusDirs` again
(`Files[Rand->SkewTowardsLast(Files.size())]`, `FuzzerFork.cpp:156`,
`FuzzerRandom.h:27-31`: `SkewTowardsLast` draws `sqrt(rand(n²))`, a density
rising *linearly* towards the last index, not excluding early ones). That is
(c): a job can only ever be seeded with something the merge let through.

**(b) is false, checked directly and not inferred.** Built
`where_two_surfaces_cross` (`mise exec -- cargo +nightly fuzz build
where_two_surfaces_cross`) and ran the merge (b) describes by hand:

```
./fuzz/target/x86_64-unknown-linux-gnu/release/where_two_surfaces_cross \
    -merge=1 -timeout=30 <scratch>/merge-out <scratch>/slow-seeds fuzz/corpus/where_two_surfaces_cross
```

with `<scratch>/slow-seeds` holding two files copied read-only from
`~/rcad-fuzz-logs/2026-09-26/artifacts/where_two_surfaces_cross/timeout-*`
(the row's own two). `MERGE-OUTER: 502 new files with 8812 new features
added` — and both slow files' own content hashes are among the 502: `ls
<scratch>/merge-out` contains `d5f895da...` and `ca050275...` by name
(libFuzzer names a merge survivor after its own hash). The 834 tracked files
do **not** already cover what these two reach; the merge keeps them. (b) was
the leg that broke the hypothesis, not (a) or (c) — the tracked corpus isn't
what stands between a slow seed and a job, low odds of the *particular* slow
file being drawn is.

**First real run, following "find a slow input that adds features the
tracked corpus lacks, however many": 25 slow files, still not enough.**
Seeded 25 `timeout-*` files (distinct from the row's two) into a fresh
scratch corpus root and ran the recipe itself, unedited:

```
mise exec -- just fuzz-target where_two_surfaces_cross 120 <scratch-root> "" 3
```

(`-artifact_prefix` is the recipe's own default here — `fuzz/artifacts/`,
gitignored — never `fuzz/corpus/`.) Merge kept 497 of the 834+25 candidates.
System load was heavy throughout (a nightly fuzz row, Q148, pinned to 20 of
24 cores the whole time) and inflated the run's wall time far past its
nominal 120s budget: `#6080: cov: 3134 ft: 8829 corp: 497 exec/s: 1520
oom/timeout/crash: 0/0/0 time: 306s job: 3`, `INFO: exiting: 0 time: 313s`,
recipe exit 0. Only 3 fork jobs completed in that time (contention, not the
merge) — too few draws against 497 candidates, of which perhaps 15 were
slow, to expect a hit; not a repeat of the first lane's finding so much as
the same low-odds draw landing wrong again.

**Second run, seeding far more: the mechanism fires, but this lane's own
kill overwrote the trailer line.** Copied 300 more `timeout-*` files into the
same scratch root (322 slow files total, plus corpus growth from the first
run, 388 entries against the 834 tracked) and reran the same recipe
invocation. This time real `-timeout=10` alarms fired repeatedly during the
run: `fuzz/artifacts/where_two_surfaces_cross/` — gitignored, `git status
--porcelain fuzz` stayed clean throughout — ended up holding 23 `timeout-*`
files, written between 16:22:57 and 16:31:35 local time, and every one
matches by content hash a file this lane had seeded (confirmed for
`timeout-0fec3f722efe6e298f84ce4bec175f214d698549`,
`sha1sum` against the scratch copy). That is 23 separate confirmations that
a slow seed **was** drawn into a job and **did** hit the alarm, under the
recipe's own hardcoded `-timeout=10`, under `-fork` — the exact mechanism
`fuzz-overnight` relies on, happening for real, repeatedly, once enough slow
seeds are in the pool.

It did not end cleanly. This lane's context budget warning landed while the
run was still going (wall time again inflated far past the nominal 120s by
system load), and the run was killed (`kill -9` on the fork orchestrator
process) rather than left to finish on its own. The log's tail is cargo-fuzz
reporting the artifacts directory's contents after detecting the kill, not
libFuzzer's own trailer:

```
Error: Fuzz target exited with signal: 9 (SIGKILL)
error: Recipe `fuzz-target` failed with exit code 1
RECIPE_EXIT=1
```

`RECIPE_EXIT=1` is consistent with §275.2 but proves nothing beyond it here,
because a `SIGKILL` from outside is a different failure than a `-fork` run
finishing with its last job's timeout. **`INFO: exiting: 70` was never
printed in this run** — the last artifact (`timeout-0fec3f722e...`) was
written at 16:31:35.147 local, one seventh of a second before this lane's own
kill landed, so the orchestrator may well have been about to print that line
and exit 70 on its own; that is a plausible reading of the timing, not a
witnessed one. The row's exact ask — `fuzz-target` exiting 1 *with* `INFO:
exiting: 70` in its own log, from a natural finish — is not yet demonstrated.

**What is settled and what is not.** Settled: the dispatcher's hypothesis is
wrong, and (b) is the leg that breaks it, not (a) or (c) — a slow input that
adds coverage is never filtered out by the merge, and enough of them in the
seed pool do get drawn into jobs and do hit the timeout, repeatedly, under
`-fork` with `-timeout=10`. Not settled: the clean trailer line, because this
run was killed rather than let finish. Left for whoever picks this up: the
scratch corpus root this lane built already has 388 files (322 of them
slow) and is reusable as-is — rerun the same `fuzz-target` invocation above
against it (off 21:50–06:00Z, and only when Q148 or another night isn't
already using most of the cores) and let it run to its own completion this
time; on this evidence it should not need more than one or two more
attempts to catch `INFO: exiting: 70` on the last job before the process's
own `exit(ExitCode)`.
