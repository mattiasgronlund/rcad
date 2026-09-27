## 306. Chaining the second merge onto the first, instead of reseeding fresh, cuts cycle 2's reported new files from 131 to 54 — most of §295.5's gain was already in cycle 1 — 2026-09-27

*Queue row Q238, off `e82dec41` (`main` at `e82dec4130d2ed8743fa4a031a430c46cd852981`). Chains §295.5's second
merge onto §295.1's first instead of a fresh 909-file copy, and measures how
much of cycle 2's reported gain is genuinely new over cycle 1 rather than
rediscovered. §295.1's own 993-file cycle-1 result and §295.5's own chained-in
figure were not preserved in any scratchpad this machine still holds (only the
row's verifier's re-run of the cycle-1 merge, 990 files/486,212 bytes, and
§295.5's own unchained merge target, 1,036 files/637,253 bytes, survived) —
`grep -rl "499607\|993 files\|MERGE-OUTER"` across every `q157`/`q295`
scratchpad this machine has found neither the original 993-file directory nor
a merge log naming it. Cycle 1 was therefore rebuilt from its two byte-verified
inputs, as the brief's fallback allows, and B's second-cycle burst output was
reused unrun (it was intact and matched §295.5 exactly). Both merges ran
`taskset -c 20-23`, foreground, sequentially, on this machine between 19:51Z
and 20:00Z on 2026-09-27, using the `no_step_file_makes_the_reader_panic`
binary already built for the Q157 lane's own arms (its scratchpad's
`target-no-standin/.../release/no_step_file_makes_the_reader_panic`, so no
rebuild was needed).*

  306.1   The inputs, and how each was checked
  306.2   Cycle 1, rebuilt
  306.3   Cycle 2, chained onto cycle 1 instead of onto a fresh copy
  306.4   Net-new two ways
  306.5   Against §295.5's unchained number
  306.6   What this says about §295.2 and about Q237
  306.7   Which inputs the count names but the directory does not keep, and why the kept count moves

### 306.1 The inputs, and how each was checked

Three source directories, each copied (hardlinked, `cp -al`) into this lane's
own scratchpad and never written into directly — only the copies were merged:

| source | found at | files | bytes | matches |
|---|---|---|---|---|
| tracked corpus | this worktree's `fuzz/corpus/no_step_file_makes_the_reader_panic` | 909 | 215,292 | §295.1 exactly |
| D2's finds | session `3fa99194`'s scratchpad, `q155/D65536/write` | 2,407 | 61,354,396 | §295.1/§291 exactly |
| B's second burst | session `58223c20`'s scratchpad, `q157/B/write` | 1,512 | 19,454,145 | §295.5 exactly |

All three matched their decision's own count and `du -sb` figure exactly, so
none of them drifted or partially decayed since 295 was written. What did
*not* survive: a directory holding exactly 993 files / 499,607 bytes (§295.1's
own cycle-1 result) or exactly 1,036 files / 637,253 bytes as a *chained*
result — only the verifier's 990/486,212 re-run of the first and §295.5's own
*unchained* 1,036/637,253 target (found as `q157/B2_target`, byte-for-byte).
Neither is what this row needs to chain onto, so cycle 1 was rebuilt.

### 306.2 Cycle 1, rebuilt

A fresh copy of the 909-file tracked corpus, merged with D2's finds, same
command and flags as §295.1's arm B:

```
taskset -c 20-23 no_step_file_makes_the_reader_panic -merge=1 -max_len=5110 <cycle1> <d2finds>
```

```
MERGE-OUTER: 3316 files, 909 in the initial corpus, 0 processed earlier
MERGE-OUTER: attempt 1
MERGE-INNER: 3316 total files; 0 processed earlier; will process 3316 files now
#3316	DONE   cov: 4187 ft: 10120 exec/s: 1658 rss: 324Mb
MERGE-OUTER: successful in 1 attempt(s)
MERGE-OUTER: 85 new files with 2307 new features added; 303 new coverage edges
```

Result: **991 files, 492,033 bytes** — 909 + 82 by directory count, not the
85 `MERGE-OUTER` reports (the same few-file slack §295.1 already flagged
between the lane's 84 and the verifier's 81; this run adds a third point, 82,
to that spread). Close to but not identical to §295.1's 993/499,607 — expected,
since §295.1 itself says the kept count is not reproducible to the file and
"nothing turns on three files." This 991-file, 492,033-byte directory is what
306.3 chains onto, standing in for the lost 993-file one.

### 306.3 Cycle 2, chained onto cycle 1 instead of onto a fresh copy

B's second burst output (1,512 files, 19,454,145 bytes — unrun, reused as
found) merged into a copy of 306.2's result, not into a fresh 909-file copy as
§295.5 did:

```
taskset -c 20-23 no_step_file_makes_the_reader_panic -merge=1 -max_len=5110 <chained = copy of cycle1> <bwrite>
```

```
MERGE-OUTER: 2503 files, 991 in the initial corpus, 0 processed earlier
MERGE-OUTER: attempt 1
MERGE-INNER: 2503 total files; 0 processed earlier; will process 2503 files now
#2503	DONE   cov: 4242 ft: 10510 exec/s: 2503 rss: 180Mb
MERGE-OUTER: successful in 1 attempt(s)
MERGE-OUTER: 54 new files with 390 new features added; 55 new coverage edges
```

Result: **1,043 files, 671,957 bytes** — 991 + 52 by directory count, again a
few files under `MERGE-OUTER`'s reported 54.

### 306.4 Net-new two ways

Both ways the brief asked for, both against 306.2's 991-file cycle-1 result:

- **`MERGE-OUTER`'s own count**: 54 new files.
- **Filename diff** (`comm -13` on sorted `ls` listings — libFuzzer names a
  file by its content hash, so a name in both directories is the same input,
  not a coincidence): **52 new names**, and `comm -23` in the other direction
  is empty — every one of cycle 1's 991 filenames is still present, so the
  merge only ever adds, confirming the 52 is a clean net-new count, not a mix
  of adds and replacements.

The two-file gap (54 vs 52) matches the same direction and rough size as
306.2's own gap (85 vs 82) and as §295.5's own numbers reread this way (131
reported against 1,036 − 909 = 127 actual) — three independent merges here all
have `MERGE-OUTER` over-report the directory's real growth by two to four
files. That pattern held every time this session ran the merge; the cause was
not chased further, matching the "not looked into" the decision already
accepted for the kept-count noise.

### 306.5 Against §295.5's unchained number

§295.5 merged the same 1,512-file burst output into a **fresh** 909-file copy
and reported `131 new files with 2692 new features added; 358 new coverage
edges`, landing at 1,036 files, 637,253 bytes (909 + 127 by directory count,
inheriting the same few-file `MERGE-OUTER` slack).

Chaining onto cycle 1 instead of reseeding fresh:

| | §295.5, unchained (fresh 909) | this row, chained (onto cycle 1) |
|---|---|---|
| reported new files | 131 | 54 |
| new features | 2,692 | 390 |
| new coverage edges | 358 | 55 |
| bytes added this cycle | 421,961 (215,292 → 637,253) | 179,924 (492,033 → 671,957) |

Chaining cuts the reported new-file count to 41% of the unchained figure, and
new features/edges to about 15% of it. Read the other way: of the 131 files
§295.5's fresh reseed called "new," roughly three in five (77-79 of them, by
the 131−54 and 131−52 gaps) were already sitting in cycle 1's result —
rediscovered, not novel. §295.5 said this explicitly could not be answered
without chaining ("its count is not a clean net-new figure... it can only be
read as what a repeat of this cycle costs and finds, not as proof of
accumulation") — this row answers it: **about 52-54 of cycle 2's would-be 131
files are net new over cycle 1, not 131.**

### 306.6 What this says about §295.2 and about Q237

§295.2's "a few hundred KB per night" rests on §295.5's unchained delta
(421,961 bytes for cycle 2, on top of cycle 1's own 284,315-byte-beyond-tracked
growth). This row's chained delta — the shape production will actually see,
since §295.7 already designs the persistent corpus to accumulate across
nights rather than reseed — is smaller: **179,924 bytes for cycle 2**, not
escalating from cycle 1's 276,741, but well under half of what the unchained
measurement implied a night costs. §295.2's estimate is likely an
overstatement of steady per-night growth once merges chain as designed; on
this two-cycle sample, real growth looks closer to 150-280 KB/night than "a
few hundred KB" on the high end. Two cycles is a thin sample for a steady-state
rate, though, and this does not replace Q237's nightly recipe measuring its
own real growth after it has run for a few nights.

Q237 is building `fuzz-overnight`'s merge step (§295.7) alongside this row.
What it should take from here: the merge step it writes chains onto the
persistent corpus by construction already (that was §295.7's whole point,
items 1-2), so it will observe something closer to this row's 52-54
new-files/180 KB shape per night than §295.5's 131-files/422 KB one — the
smaller number is not a regression to explain, it is what correctly chaining
was expected to do.

### 306.7 Which inputs the count names but the directory does not keep, and why the kept count moves

*Queue row Q261, off `bf765d2b` (`main` at
`bf765d2be9c5b82725b43ba1c05c6e360953e556`). The three source directories
306.1 checked were still on this machine, byte-for-byte, so this row reused
them rather than re-verifying: the tracked corpus, D2's 2,407 finds and B's
1,512-file second burst. The `no_step_file_makes_the_reader_panic` binary is
the same one 306.2-306.3 used, still sitting in Q157's scratchpad. Every merge
below ran `taskset -c 0-5`, in a script under this lane's own `target/q261/`,
each launched detached and waited out with `tail --pid`. `uptime` before the
first run and after the last: load average 8.18 → 5.54 (this machine, other
lanes on other cores), between 19:25:06Z and 19:26:11Z on 2026-09-27 — the
whole reproduction (cycle 1 rebuild, three chained cycle-2 merges) took 65
seconds start to finish.*

**Part 1: what "N new files" counts.** Read in libFuzzer's own source
(`libfuzzer-sys-0.4.13`'s vendored copy, under
`~/.cargo/registry/src/.../libfuzzer/`), not inferred:

- **`Merger::Merge`** (`FuzzerMerge.cpp:139-194`) walks the files handed to it
  and does `NewFiles->push_back(Files[i].Name)` (line 188) for every one that
  adds a feature not already claimed — `Name` is the **input file's own path**,
  on disk, in whichever corpus directory it came from. This is the number
  `MERGE-OUTER: %zd new files` prints (line 534-536). **Leg (a) holds**: N is a
  count of picks, not a count of anything written.
- **`Merge()`** in `FuzzerDriver.cpp` (lines 517-544) is what turns those picks
  into files: `for (auto &Path : NewFiles) F->WriteToOutputCorpus(FileToVector(Path, Options.MaxLen))`
  (line 537-538). Two things happen to each pick before it becomes a file:
  - `FileToVector(Path, MaxSize)` (`FuzzerIO.cpp:41-58`) truncates to the first
    `MaxSize` bytes when the file is longer — `-max_len=5110` here, and B's
    burst was written at `-max_len=65536`, so a majority of its files are
    candidates for truncation.
  - `Fuzzer::WriteToOutputCorpus` (`FuzzerLoop.cpp:632-642`) names the output
    file `DirPlusFile(Options.OutputCorpus, Hash(U))` — content hash, always —
    and `WriteToFile` writes it with no existence check. Two picks whose bytes
    agree after truncation, or a pick whose truncated bytes match a file
    already sitting in the output directory, land at the same path and the
    second write is a no-op overwrite. **Leg (b) holds**, read directly off
    this call chain, not inferred from the count mismatch it explains.
- **Leg (c)**, checked by hashing rather than assumed: `sha1sum` over B's 1,512
  files, full length, found only 1 duplicate pair — but B and the tracked
  corpus share 909 identical files by full-length hash (every one of the
  tracked corpus's 909 files has a byte-identical twin somewhere in B).
  Truncated to 5110 bytes, B alone collapses from 1,512 distinct files to
  1,244 — **66 duplicate-hash groups**, 268 files' worth, purely from cutting
  files that were fuzzed at a larger `-max_len` down to this merge's smaller
  one. So there is real duplicate material for leg (b) to collapse — the
  question 306.7's second half answers is whether that duplicate material, or
  something else, is what actually produces the 2-4 file gap and the
  run-to-run movement.
  (`hashcheck.sh`, this lane's `target/q261/`.)

**Reproducing 306.2-306.3 to watch it happen.** A fresh cycle-1 rebuild (909
tracked + 2,407 from D2, same command as 306.2) landed at **989 files** off
`MERGE-OUTER: 81 new files` — 80 by directory count, one file under 306.2's own
991/85-vs-82 run, which is itself the point: this command has never landed on
the same number twice. Chaining B onto a copy of that 989-file result three
times in a row, same binary, same flags, same input bytes:

| | reported (`MERGE-OUTER`) | new features | new cov. edges | kept (directory count) |
|---|---|---|---|---|
| c2run1 | 54 | 391 | 54 | 51 |
| c2run2 | 53 | 386 | 55 | 51 |
| c2run3 | 55 | 396 | 54 | 52 |

Commands (`target/q261/run_cycle2.sh`):
`taskset -c 0-5 no_step_file_makes_the_reader_panic -merge=1 -max_len=5110 -merge_control_file=<cf> <copy of cycle1real> <B>`,
run three times over fresh `cp -al` copies of the same 989-file directory.
`/usr/bin/time -v` on each: 1.26-1.28 s wall, well under a second of that
running 2,501 inputs through the target — nowhere near this run's (unset,
so libFuzzer's 1200 s default) `-timeout`, and none of the three logs contains
a `fail`, `crash`, `timeout`, `stumbl(ed)` or `skip` line. **The near-timeout
leg of the "moving count" hypothesis is ruled out for this reproduction**: no
input in this merge took long enough, or failed, for a timeout to be in play.

Both the reported count and the kept count move, run to run, on byte-identical
inputs, binary and flags — matching Q261's own framing of the problem, not
just 306's. Filename diffs (`comm` on sorted directory listings) show it is
real membership that moves, not just arithmetic: c2run1 and c2run2 keep the
identical 51-file set (`comm` empty both directions) despite reporting 54 vs
53; c2run3 keeps one file neither of the others does —
`693eec976b5723a5fe252d81d39c2dc5d8c73353`, 5,110 bytes, confirmed by
`sha1sum` on the file `run_cycle2.sh` wrote it to and cross-checked against
`B.trunc.sha1` as B's own file truncated to this merge's `-max_len` — a file
whose coverage was judged novel in one run of three and not in the other two,
on the same bytes.

A second, independent reproduction makes the same point more sharply. Merging
B directly into three fresh copies of the plain 909-file tracked corpus (no
chaining — this is 295.5's shape, not 306's) reported the **same** count all
three times, `128 new files`, yet kept 128, 127 and 127 files respectively, and
the new-coverage-edge count moved under the constant headline number (357,
354, 358). Filename diffs confirm real churn hiding under that stable "128":
run1 and run2 differ by 3 files each direction (`344ca1c0...`,
`693eec97...`, `fd3d3c61...` in run1 only; `2d1896d5...`, `b16f61dd...` in run2
only — net -1, matching 128 → 127), and run2 vs run3 differ by one file each
way. **`693eec97...`** — the same file 306.7's chained reproduction flagged —
shows up flaky here too, present in the unchained run1 and absent from run2
and run3. One file's coverage was not settled by three separate merge
processes of the identical bytes.

**Conclusion.** Leg (a) and leg (b) are confirmed from libFuzzer's own source,
not inferred from the count gap they explain: `MERGE-OUTER`'s N is a count of
which *input* files the merge's greedy pass judged to add a feature, and each
one that survives is written under the SHA-1 of its (possibly `-max_len`
-truncated) bytes, so two picks that agree after truncation, or a pick that
agrees with something already in the output directory, are one file, not two.
Leg (c) is true of these inputs (66 duplicate-hash groups inside B alone, once
truncated) but is raw material for leg (b), not the moving part: the 2-4 file
gap and the run-to-run movement both trace to the same upstream cause —
**which files the merge's inner process judges to add a new feature is not
stable across runs of the identical bytes**, evidenced directly by
`693eec97...` and its neighbours appearing and disappearing across otherwise
identical merges, and by the new-coverage-edge count moving under a constant
headline number in the unchained reproduction. This machine's
`no_step_file_makes_the_reader_panic` links `rcad-step`, whose STEP graph code
(`crates/rcad-step/src/graph.rs`, `read.rs`) uses `std::collections::HashMap`,
whose default hasher is randomized per process — consistent with a run's
traversal order, and so its coverage trace, differing from the next run's on
identical bytes. That is offered as what is consistent with the finding, not
as a traced cause: this row did not instrument the target to confirm
hash-iteration order is the specific mechanism, only that no timeout or
crash explains the movement and that the movement is real, file by file.

**For the night's merge log**: report the kept count, not `MERGE-OUTER`'s own
number. The gap is 2-4 files on merges this size and is not noise to average
away — it is `MERGE-OUTER` counting picks that never became files, and the
kept count (`find $OUTPUT -type f | wc -l`, before and after, as
`fuzz-overnight`'s own reader-corpus-merge block already does at
`persistent_files_before`/`persistent_files_after`) is the number that
describes what the corpus actually gained. Changing `fuzz-overnight` to use it
is not this row's write set.
