## 134. Under `-fork` the exit status stops being a crash oracle, built 2026-09-18

*§133 left the recipe half of §129's item 1 unwritten and one question open: what
exit 70 means. Read out of libFuzzer's source, the answer is that the fork
parent's exit code is whichever of its forty thousand jobs was collected last, so
it is not a crash **and zero is not a clean night either** — a crash in job 3 of
40 000 exits 0. The night's oracle moves off the status and onto the name
libFuzzer gives the file it writes.*

### 134.1 The flags, and which of them do anything

`-fork=N -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1`, added in
`fuzz-target` and passed only when its new `fork` argument is above zero.

Of those, **only `-ignore_crashes=1` changes libFuzzer's behaviour.**
`FuzzerFlags.def:66-67` gives `ignore_timeouts` and `ignore_ooms` a default of 1
already; `ignore_crashes` defaults to 0, and without it the first crash reaches
the `else` at `FuzzerFork.cpp:450`, calls `StopJobs()`, and ends that target's
night with however many hours were left.

The other two are passed anyway, and the reason is not decoration: with item 2's
per-input `-timeout` the night will *depend* on timeouts being survivable rather
than merely tolerate it, and a default that moved under us would end every target
in its first minute on §91.3's landed slow units. A default depended on is worth
one word of pinning.

### 134.2 The exit status is the last job's, in both directions

`FuzzerFork.cpp` keeps one `ExitCode` across the whole run. Each collected job
overwrites it (the `Job->ExitCode` assignment above line 437); nothing resets it;
the loop's `continue` paths for ignored timeouts, ignored OOMs and ignored
crashes all leave it holding that job's code. Line 488 comments *"Use the exit
code from the last child process"* and line 490 is `exit(ExitCode)`.

So the status the night reads is a fact about **one arbitrary job out of forty
thousand**, and every fate the night is built to survive leaves a mark on it:

- a timeout in the last job → **70** (`timeout_exitcode`, `FuzzerFlags.def:55`)
- a crash in the last job → **77** or a signal (`error_exitcode`, line 53)
- a clean last job → **0**, *no matter what happened in the other 39 999*

§133.7 measured the first of these: two runs of the same command on the same
corpus exited 70 and 0, differing only in which job landed last. The handoff that
carried this forward framed the job as "decide what 70 means", and that framing
is half the problem — it treats a false alarm as the whole defect. **The blind
spot is the larger one.** A status mapping that trusts zero reports a night as
clean while a crash sits in `fuzz/artifacts/`, which is exactly the failure §91
was written to prevent, and it is silent rather than noisy.

### 134.3 What fails a night instead: the name libFuzzer gave the file

libFuzzer names every input it writes for what happened to it —
`DumpCurrentUnit` and `WriteUnitToFileWithPrefix` in `FuzzerLoop.cpp` give
exactly five prefixes: `crash-` (lines 191, 236, 250, 574), `oom-` (132, 319),
`timeout-` (301), `leak-` (717) and `slow-unit-` (456).

Four of those five are a bug and fail the night. `slow-unit-` is not: it means an
input was slower than its neighbours, and the night of 2026-09-13 wrote three
while being a good night. That distinction already existed in prose in
`fuzz-overnight`'s artifact count; it is now load-bearing.

**`timeout-` is in the failing set on today's meaning, and item 2 has to revisit
it.** Today a timeout is libFuzzer's 1200-second default and genuinely wants
looking at. Under item 2's `-timeout=10` the six `slow-unit-*` seeds §91.3 landed
in `fuzz/corpus/where_two_surfaces_cross/` — one of them 63 seconds — would write
a `timeout-` file on the corpus replay every single night. A standing population
cannot be what fails a night, so item 2 owns that edit. The comment in the recipe
says so at the point of change.

The status keeps the one question no child answers: `timeout(1)` killing the
whole target from outside, which is still 124 or 137.

And a second, positive check replaces what the status used to carry. `INFO:
exiting:` is the last line `FuzzWithFork` prints before `exit`, so it is present
exactly when the fork loop finished of its own accord. Absent, the target never
got as far as fuzzing — a build or launch failure — and that is a night that
fuzzed four targets while reporting five.

### 134.4 Tonight's crash and not last week's

`fuzz/artifacts/` is gitignored and never cleaned, so it accumulates in the tree
the timer keeps checked out, and `copy_out` copies all of it. The artifact
**count** in the summary has always been carry-over for that reason, and it could
stay a wart while it was only ever reported.

It cannot stay one now that a crash file is what fails a night: a crash from a
week ago would fail every night from here on, nobody would read the summary after
the third, and that is §91's failure with the sign flipped — a night nobody
believes, reached by crying wolf rather than by staying quiet.

So the find is read against a marker file written just before the targets start,
and out of the worktree rather than out of the night's copy — `cp -Rn` gives the
copy a copy-time mtime while the original keeps libFuzzer's own. libFuzzer writes
an artifact once and never rewrites it, so mtime answers this question exactly.

### 134.5 A hand run does not fork

`fork` defaults to `0` on all five recipes and `fuzz-target`, and only
`fuzz-overnight` passes a number. A hand-run `just fuzz 60` wants the opposite of
what the night wants: one crash, one report, and a stop, so the person watching
can read it. Only the night would rather lose the report than lose eight hours.

The number is `nproc` divided by the five targets, which are already running at
once since §129 — 4 on this machine, 20 jobs against 24 cores. Computed and not
written down: a literal `4` would under-use a bigger machine and thrash a smaller
one, and this justfile is checked out on whatever the timer runs on.

### 134.6 The mutants

Five, each caught by one arm, each with a message naming the defect:

| mutation | arm | what it said |
| --- | --- | --- |
| non-zero → `crash`, no liveness check | One | a target never got fuzzing and the night reported success |
| liveness check kept, non-zero → `crash` as well | Eleven | every target exited 70 after fuzzing its whole window and the night reported 1 — `5 TO LOOK AT (fuzz:crash fuzz-reader:crash …)` |
| crash artifacts counted but not reported | Twelve | a target wrote a crash artifact and the night reported success |
| `-newer "$since"` dropped | Thirteen | a crash left over from an earlier night failed this one |
| `-ignore_crashes=1` dropped | One | a target forks without `-ignore_crashes=1`, so its first crash still ends its night |
| `fork=0` | One | no target was told to fork |

The second row is the one that matters most, because it is the fix the handoff
described — teach the status about 70 — and it still fails: it reports five
crashes on a night where nothing happened.

**Two arms were fixed on the way, both of the same shape.** The artifact oracle's
`find` and the fork assertion's `grep` each exit 1 when they match nothing, and
`pipefail` took the recipe down with no message before either could report. Arm
Ten of this selftest already carries that lesson in a comment from the last time
it was learned; it has now been learned twice more in the same file.

### 134.7 The fixture had to change with the contract

The stub `cargo` wrote `crash-<tag>` for every target on every run. Since a crash
file is now what fails a night, that left no fixture able to express a good night
at all — so it writes `slow-unit-` by default and `crash-` only when an arm asks.

Arm One's meaning moved with it. It used to say "the second target dies" with
`exit 1`, which under fork mode is a target that exited 70 and was fine; it now
says it by withholding `INFO: exiting:`, which is the failure the status used to
carry and no longer can. The assertion is `:no-run` where it was `:crash`, and
everything else about the arm is unchanged.

Commands run to reach this: `grep` of `FuzzerFlags.def`, `FuzzerFork.cpp` and
`FuzzerLoop.cpp` in `libfuzzer-sys-0.4.13`; `mise exec -- just
check-fuzz-overnight-selftest` against the tree and against six mutations; a real
`mise exec -- just fuzz 25 <corpus> 2`, which is the only thing here that says
libFuzzer accepts the argv rather than that the recipe emits it;
`mise exec -- just gate`.

