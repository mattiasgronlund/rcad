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
