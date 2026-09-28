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

### 297.1 Where the 3400 s went

*Queue row Q239, checking the hypothesis §297 only stated: that the startup
merge, paying up to `-timeout=10` on each slow seed, is the 3400 s. Lane
`lane-q239-where-the-3400s-went`, off `0fcd3e3c`.*

Q180's corpus root (388 files, 322 `timeout-*`) is unchanged since §297
(`find ... -name 'timeout-*' | wc -l`: still 322 of 388). `where_two_surfaces_cross`
was built the way `fuzz-target` builds it (`CI=1 RUSTC=fuzz/rustc-stand-in.sh
RUSTFLAGS="-C target-cpu=x86-64-v3" cargo +nightly fuzz run`), then run once
directly with `-merge=1 -timeout=10` — no `-fork` — against that corpus root
alone, merging into a fresh, empty `target/q239/merge-out` and never writing
into Q180's copy:

```
taskset -c 0-5 cargo +nightly fuzz run where_two_surfaces_cross \
    target/q239/merge-out <q180-corpus-root>/where_two_surfaces_cross \
    -- -merge=1 -timeout=10
```
under `/usr/bin/time -v`, pinned throughout to cores 0-5 (other lanes hold
6-23), launched detached and waited out in `timeout 540 tail --pid=<pid> -f
/dev/null` segments. `uptime` before: `load average: 2.83, 4.86, 11.61`;
after: `load average: 11.21, 9.37, 10.48` (a neighbor lane, not this one,
which held 99% of one core throughout).

**The run took 54:50.76** (`/usr/bin/time -v`'s `Elapsed (wall clock) time`;
user 3226.33s, sys 37.60s, 99% CPU, exit status 0) — 3290.76 s, against
§297's single stats-line `time: 3400s`, a 109 s (3.2%) difference.

**Leg (a), confirmed and sharpened.** No `-fork` was passed here, yet the log
shows exactly the two-level process §255.3 only inferred from `ps` showing
`-merge_inner=1`: an outer supervisor (`MERGE-OUTER: attempt N`) that, on
every crash, relaunches a fresh child which reads a persistent control file
and skips the entries already marked done (`MERGE-INNER: 388 total files; 69
processed earlier; will process 319 files now`, and so on through 62
attempts). This is what actually walks the seed corpus in both plain
`-merge=1` and fork mode's own startup pass — the same mechanism, not two
different ones. This run makes no fresh claim about `-max_total_time` itself
(no `-fork`, no `-max_total_time` flag were passed); that half of the leg
still rests on §297's own log, where the `time: 3400s` stats line only
appears after `-fork=3: 580 seed inputs, starting to fuzz` — i.e. after this
merge phase, not counted against it.

**Leg (b), broken as stated, held in substance.** Exactly 61 of 388 files
(16%; 61 of the 322 `timeout-*`-tagged files, 19%) actually tripped the
`-timeout=10` alarm — one crash (`ERROR: libFuzzer: timeout after 11
seconds`) per `MERGE-OUTER` attempt beyond the first, 61 crashes over 62
attempts (`grep -c "ERROR: libFuzzer: timeout"`: 61; every one of the 61
distinct failing paths is itself named `timeout-*`, none of the 66
non-tagged files crashed:
`grep -o "...' caused a failure" | sed ... | sort -u | wc -l`: 61, all
`^timeout-`). So "most... hit the per-input timeout" is false read literally
— most did not crash. But the run's own pace, sampled at each poll (elapsed
seconds from `ps -p <pid> -o etimes`, "processed earlier" count from the log
at that moment): 0→545s/0→69 files (7.90 s/file), 545→1116s/69→131
(9.21 s/file), 1116→1665s/131→187 (9.80 s/file), 1665→2210s/187→252
(8.38 s/file), 2210→2756s/252→307 (9.93 s/file), 2756→3290.76s/307→388
(6.60 s/file) — a steady 7-10 s per processed file for the *entire* run, not
a two-speed mix of near-zero fast files and 10s-capped slow ones. The other
261 `timeout-*` files that never crossed the alarm still cost seconds each to
replay, nowhere near the target's documented p99 of 7.9ms (`justfile`'s own
comment on the `-timeout=10` choice). The wall time is not built from a
handful of literal alarm trips; it is built from nearly the whole
`timeout-*` population being seconds-expensive to replay, alarm or no alarm.
The log gives no true per-file timing (only cumulative "pulse" markers with
no timestamp), so the 7-10 s/file figures above are this run's own polling
checkpoints, not something read out of the log directly.

**Leg (c), held.** 388 files at a measured, steady 7-10 s each sums to
exactly the right order of magnitude, and the direct measurement confirms it
closer than the arithmetic needs to: 3290.76 s against §297's 3400 s, 3.2%
apart, on a different day, different core pinning (0-5 here, 14-17 there),
and rising ambient load during this run. **This pass is §297's 3400 s.** The
merge phase alone, before any `-fork` job gets a slice of `-max_total_time`,
already accounts for the whole reported span.

**A night's seed pool can hold that many slow inputs, easily — it already
does.** §297's `-fork=3` run merged two sources: the tracked
`fuzz/corpus/where_two_surfaces_cross` (834 files, `ls | wc -l`, unrelated to
this lane's write set so not fuzzed further here) plus Q180's 388-file
corpus root, 1222 raw candidates, kept down to the 580 §297 already reported.
This lane's own merge, of the 388-file corpus root *alone* into an empty
output directory, kept 193 of them as sufficiently novel
(`MERGE-OUTER: 193 new files with 5872 new features added; 1886 new coverage
edges`) — a different merge (one source, not two, into an empty rather than
partly-primed output) from the one that produced §297's 580, so this number
is a proxy for "how many of the slow files survive a merge" and not a count
of how many of the 580 are themselves slow. Getting that exact split would
mean rerunning §297's own two-source merge and diffing filenames — out of
this row's scope, and not attempted here rather than guessed.

Measured: `taskset -c 0-5 /usr/bin/time -v cargo +nightly fuzz run
where_two_surfaces_cross target/q239/merge-out <q180-corpus-root> --
-merge=1 -timeout=10`, log at this lane's `target/q239/run-merge.log`
(gitignored, not part of this commit).

### 297.2 Splitting §297's 580 kept seeds by source

*Queue row Q270, off `6f3c62e0` (`main` at
`6f3c62e09f8625023ca129b2c0105b25da6defc3`). Lane
`lane-q270-two-source-split`. `where_two_surfaces_cross` was rebuilt the way
`fuzz-target` builds it (`CI=1 RUSTC=fuzz/rustc-stand-in.sh
RUSTFLAGS="-C target-cpu=x86-64-v3" cargo +nightly fuzz run`, checked against
`just recipe fuzz-target` rather than copied from §297.1). At this lane's
base the tracked corpus still holds 834 files (`ls
fuzz/corpus/where_two_surfaces_cross | wc -l`) — unchanged from §297 and
§297.1, so the split below compares against the same two source sizes §297's
580 came from, with nothing to correct for. Q180's corpus root, copied into
this lane's own `target/q270/root-copy` rather than read from the scratchpad
directly, is still 388 files, 322 of them `timeout-*`
(`find ... -name 'timeout-*' | wc -l`).*

**One run, as asked.** §297's own two-source `-merge=1 -timeout=10` pass was
rerun once, of a copy of the tracked corpus plus a copy of Q180's root, into
a fresh empty `target/q270/merge-out`, pinned to cores 16-19, launched
detached and waited out in `timeout 540 tail --pid` segments. `uptime`
before: `load average: 7.78, 6.09, 8.58`; after: `load average: 9.56, 8.21,
10.33`. No `-max_len` flag was passed (matching §297.1's own direct-merge
command); the log says why the truncation step of §306.7's leg (b) never
fires here: `-max_len is not provided; libFuzzer will not generate inputs
larger than 1048576 bytes`. Checked, not assumed: every one of the 1222
input files, in both sources, is at most 71 bytes
(`find ... -printf '%s\n' | sort -n | tail -1`), so hashing each one whole
(`sha1sum`) and cut to 1,048,576 bytes (`head -c 1048576 | sha1sum`) landed
on identical digests for all 1222 — the two hash sets this section needed are
one set.

**The run took 54:30.10** (`/usr/bin/time -v`'s `Elapsed (wall clock) time`;
user 3201.61s, sys 37.81s, 99% CPU, exit status 0) — 3270.10 s, close to
§297.1's 3290.76 s for the *single*-source root-alone merge, despite this
merge walking 1222 candidates against that run's 388: the extra 834 tracked
files are cheap next to the root's `timeout-*` files, which is also where
this run's cost concentrates. 71 of this run's 72 `MERGE-OUTER` attempts
crashed on the per-input alarm (`grep -c 'ERROR: libFuzzer: timeout'`), and
every one of the 71 distinct failing paths is a `root-copy/timeout-*` file
(`grep -o "...' caused a failure" | sed ... | sort -u`, cross-checked
against `^timeout-`); none is from the tracked copy. Same shape as §297.1's
leg (b), one source further in.

**MERGE-OUTER: 586 new files with 9152 new features added; 3121 new
coverage edges** (`successful in 72 attempt(s)`) — the kept directory count
(`find target/q270/merge-out -type f | wc -l`) is also 586, an exact match
this run rather than the 2-4 file gap §306.7 measured elsewhere: this run's
kept set and its reported count happen to agree. 586 against §297's 580 is 6
more, on the identical two source directories and the same flags; §306.7
already found a two-source merge's kept set moves by a few files between
runs of identical bytes, and this section runs it once rather than a second
time to explain the 6.

**The split, by hashing every kept name against both sources'
SHA-1s** (`comm -12` of the sorted kept-name list against each source's
sorted digest list): **454 of the 586 kept files came from the tracked
corpus, 132 from Q180's root** — 454 + 132 = 586 exactly, with no kept name
matching both sources' digests and none matching neither (`comm` on the two
splits, and on the kept list against the union of both digest lists, are all
empty). Mapping the 132 root-sourced digests back to the root's own
filenames (`sha1sum` over `root-copy`, joined on digest, since the root's
388 files hash to 388 distinct digests — `sort -u | wc -l` — so the join is
unambiguous): **70 of the 132 are `timeout-*` files, 62 are not.**

So of the 580-590-ish seeds a two-source merge of this pair keeps, most are
the tracked corpus's own (454, 78%), the root contributes about a fifth
(132), and only a little over half of what the root contributes is itself
one of its slow files (70 of 132) — the root's other 62 kept files survived
the merge on coverage that had nothing to do with being slow. §297.1's 193
(root merged alone) was already flagged as a proxy and not this count; this
row is the count itself, not a proxy for it.

Scripted at this lane's `target/q270/run-merge.sh`
(`/usr/bin/time -v` into `target/q270/run-merge.log`, both gitignored, not
part of this commit).
