## 270. `fuzz-overnight`'s own mtime tally, not `fuzz-yield`'s filename diff, is the wrong count for `where_two_surfaces_cross` on 2026-09-26

*Q165. The row: on 2026-09-26 `fuzz-yield` counted 413 new timeouts and 188 new
slow units for `where_two_surfaces_cross`, where the night's own summary line
(`fuzz-overnight`'s mtime-based tally, `~/rcad-fuzz-logs/latest`) counted 484
and 230 — on every other target the two agree to within a file. Read off the
files, not the code: `fuzz-yield`'s 413/188 is right, and `fuzz-overnight`'s
484/230 is inflated by exactly the 71 timeout files and 42 slow-unit files
that this target rewrites, byte-for-byte unchanged, on nights it does not find
anything new at all. No fix belongs in `tooling/fuzz-yield/`; the bug is
`fuzz-overnight`'s and is named at the end for a new row.*

  270.1   The two counts and where each is computed
  270.2   `fuzz-yield`'s 413/188 is a plain before/after file-list count
  270.3   `fuzz-overnight`'s 484/230 double-counts 71 + 42 files that already existed, byte for byte
  270.4   Why this target and not the other four
  270.5   Verdict, and what is left

### 270.1 The two counts and where each is computed

`fuzz-yield`'s per-target "new files" line (`justfile`'s `fuzz-yield` recipe)
takes `comm -13` between the nearest earlier night's own copy of
`fuzz/artifacts/<target>/` and tonight's, both under `~/rcad-fuzz-logs`, and
counts what tonight's listing has that the earlier one does not — a filename
comparison, with the trailing 40-hex-character hash stripped before counting
by kind. Running it directly:

```
mise exec -- just fuzz-yield 2026-09-26 2026-09-26
```

prints, for `where_two_surfaces_cross`: `new files since 2026-09-25: 188
slow-unit, 413 timeout`.

`fuzz-overnight`'s own tally is a different computation over a different
directory: at the end of the night it counts, per kind, how many files under
the *live* fuzzing worktree's `fuzz/artifacts/<target>/` have an mtime newer
than `$night/.started` (`find "$target_dir" -maxdepth 1 -type f -name
"$kind-*" -newer "$since"`), where `$since` was touched once, at the night's
start. That figure is what landed in `~/rcad-fuzz-logs/latest`:

```
$ cat ~/rcad-fuzz-logs/latest
2026-09-26 08:15  c92b8fe5 (0 days old, 2 behind main)  5 targets, 2 TO LOOK AT
(fuzz-writer:overran crashed:22623), new tonight: ... where_two_surfaces_cross:
484 timeout, 230 slow-unit  27070 artifacts in /home/mattias/rcad-fuzz-logs/2026-09-26/artifacts
/home/mattias/rcad-fuzz-logs/2026-09-26
```

Re-running the exact `find … -newer` the recipe runs, against the live
fuzzing worktree (`~/git/mattiasgronlund/rcad-nightly-fuzz/fuzz/artifacts/where_two_surfaces_cross`,
the tree `rcad-fuzz.service` actually fuzzes in) and `~/rcad-fuzz-logs/2026-09-26/.started`
as `$since`, reproduces the summary line's numbers exactly: 484 timeout files,
230 slow-unit files.

### 270.2 `fuzz-yield`'s 413/188 is a plain before/after file-list count

Counting the two nights' directories directly:

```
$ ls ~/rcad-fuzz-logs/2026-09-26/artifacts/where_two_surfaces_cross | grep -c '^timeout-'
3066
$ ls ~/rcad-fuzz-logs/2026-09-25/artifacts/where_two_surfaces_cross | grep -c '^timeout-'
2653
$ ls ~/rcad-fuzz-logs/2026-09-26/artifacts/where_two_surfaces_cross | grep -c '^slow-unit-'
1359
$ ls ~/rcad-fuzz-logs/2026-09-25/artifacts/where_two_surfaces_cross | grep -c '^slow-unit-'
1171
```

3066 − 2653 = 413, and 1359 − 1171 = 188, matching `fuzz-yield`'s line exactly.
That simple subtraction only equals the `comm -13` count when nothing was
removed between the two nights; `comm -23` (names in 09-25's list but not in
09-26's) is empty, confirming 09-26's directory is a strict superset of
09-25's, so `fuzz-yield`'s number is not a coincidence of two counts that
happen to subtract to the same thing — it is the same set.

### 270.3 `fuzz-overnight`'s 484/230 double-counts 71 + 42 files that already existed, byte for byte

The gap is 484 − 413 = 71 timeouts and 230 − 188 = 42 slow units. Comparing,
by name, the files the live `-newer` check flags as "new tonight" against the
names already present in 09-25's copy:

```
$ comm -12 <(find …/where_two_surfaces_cross -name 'timeout-*' -newer .started -printf '%f\n' | sort) \
           <(ls ~/rcad-fuzz-logs/2026-09-25/artifacts/where_two_surfaces_cross | grep '^timeout-' | sort) | wc -l
71
$ comm -12 <(find …/where_two_surfaces_cross -name 'slow-unit-*' -newer .started -printf '%f\n' | sort) \
           <(ls ~/rcad-fuzz-logs/2026-09-25/artifacts/where_two_surfaces_cross | grep '^slow-unit-' | sort) | wc -l
42
```

71 and 42 — exactly the gap. `md5sum` on the live copy of each of the 113
files against 09-25's copy of the same name (a small script, not a heredoc,
run from this lane's tree) finds **0 content mismatches out of 113**: every
one of the files the mtime check counted as new tonight is a byte-for-byte
copy of a file already sitting in the previous night's own artifact listing.
Their mtimes are not clustered at the night's start (a one-time corpus
replay): `stat` on all 113 puts them between `00:14` and `08:10` on
2026-09-26, against a `.started` mtime of `00:11`, with 24 in the first hour
and 84 from `05:00` on. `fuzz-overnight`'s `-newer $since` cannot tell
"written again tonight, unchanged" from "genuinely new tonight" — a
content-addressed name that already exists gets its mtime bumped every time
libFuzzer's `-fork` workers rewrite it, and the check counts every bump.

`fuzz-yield`'s filename diff has no such blind spot: a name already present
in the previous night's listing is excluded regardless of what its mtime did
in between, which is exactly why its 413/188 matches the actual new-file
population and `fuzz-overnight`'s 484/230 does not.

### 270.4 Why this target and not the other four

The row itself says the two counts agree to within a file everywhere else,
and the same comparison bears that out: for `two_solids_fused_by_a_stranger`
that night, the mtime tally read 2 timeout/3 slow-unit against `fuzz-yield`'s
1/3, and for `where_a_cone_or_a_torus_crosses` they matched exactly (1
timeout, 3 slow-unit). Nothing there approaches a 71- or 42-file gap.

`where_two_surfaces_cross` is the one target `docs/decisions.d/135-*.md` and
`docs/decisions.d/260-*.md` already establish as carrying a standing
population of slow, timeout-triggering inputs in its own corpus — §135.4's "a
`timeout-` file on `where_two_surfaces_cross` is a nightly certainty and not
news," measured again in §260 as 78–87% of every night's job-endings, every
night since `-fork` mode began. A target whose corpus keeps re-finding the
same slow inputs every night is exactly the target whose already-written
artifact files keep getting rewritten (same name, same bytes, new mtime) —
the population the other four targets, without that standing tail, mostly
lack.

### 270.5 Verdict, and what is left

`fuzz-yield`'s count is correct and needs no change: `tooling/fuzz-yield/`,
`justfile`'s `fuzz-yield` recipe and `check-fuzz-yield-selftest` are untouched
by this section. There is no fix to add a selftest arm for.

`fuzz-overnight`'s own-night summary line is the wrong count, on this target,
by exactly the recurring-file population above. `fuzz-overnight` is outside
this lane's write set (`tooling/fuzz-yield/`, `justfile#fuzz-yield`,
`justfile#check-fuzz-yield-selftest`, `docs/decisions.d/`), and the recipe's
`fuzz/artifacts` section is also touched by the currently-running Q148
core-split lane, so this section fixes nothing there. The merger should add a
new row: `fuzz-overnight`'s per-kind `-newer "$since"` tally needs to exclude
names that already existed before tonight (e.g. diff against a listing taken
at `$since` rather than trust mtime alone), or drop the live tally in favour
of the copied-out directory's own filename diff that `fuzz-yield` already
computes correctly.
