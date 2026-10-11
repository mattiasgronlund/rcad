## 434. A crash file that passes is folded into the corpus — 2026-10-11

*Mattias, 2026-10-11, on the crash files the night carries forward: "If the
fusser found 10000 files which was fixed by one change, why should we keep
regression-testing all these files. In other words what make us think that only
a subset of them would start to fail again if the fix was altered? Or why do we
think these files are "so much more important to regression test" than any
other file? Would it not be enough to keep one of the files that where fixed by
one single root cause fix?" Offered "strictly one per group" or "one per path
through the code", he chose: "One per path through the code, write it down".*

In short: once a crash file passes on `main`, the morning triage stops carrying
it. It folds the file into its target's persistent corpus with libFuzzer's
`-merge=1`, which keeps it only if it takes the code down a path no kept file
takes, and removes it from the night's tree. Every night replays its corpus
before it fuzzes, so the files kept are tested again each night for nothing. A
file that adds no path is dropped. Not built yet: Q1731 builds it.

### 434.1 Nobody decided to keep every crash file

The night copies its tree's `fuzz/artifacts/` into its own directory
(`copy_out` in `fuzz-overnight`: `cp -Rn fuzz/artifacts/. "$night/artifacts/"`).
That directory is gitignored and never cleaned, so every crash file ever found
is copied again every night, and every morning replays every one. §410.3 relies
on this only so that a missed morning costs nothing. That needs a file kept
until one replay has seen it, not kept for ever.

The pile:

- 22 633 crash files on 09-26 (§410.3), 69 879 on 10-06, 79 359 on 10-11.
- On 10-11's replayed `main` (a5d31701), 68 158 of the 79 359 pass and 11 201
  fail. Of the 68 158, 68 013 are the writer's
  (`no_model_makes_the_writer_panic`), 135 the fuse's, 8 are
  `where_two_surfaces_cross`'s and 2 `where_a_cone_or_a_torus_crosses`'s. The
  reader has none. Summed from the `files` column of
  `~/rcad-fuzz-logs/2026-10-11/fuzz-triage-verdicts.tsv`.
- 68 157 of those 68 158 already passed on 10-09, the first night whose triage
  wrote a `fuzz-triage-members.tsv`: they were in that night's `artifacts/` and
  not in its members file. Three mornings have replayed them to say the same
  thing.
- `rcad-fuzz-triage.service` started at 08:30 and ended at 08:45:49, 08:48:40
  and 08:46:33 on 10-09, 10-10 and 10-11, using 14m35s, 16m59s and 15m10s of
  CPU, build included (`journalctl --user`).
- 10-11's `artifacts/` is 398 MB and 10-06's was 326 MB. All of
  `~/rcad-fuzz-logs` is 3.0 GB, because each night's directory holds a full
  copy of its own.

### 434.2 One per path through the code

Mattias's question is why the 68 013 writer files that a few fixes made pass are
worth replaying every morning, when a fix that broke would show on some of them
as surely as on all of them. Two answers were offered: keep strictly one file
per group a fix closed, or keep one per path through the code. He chose the
second.

`-merge=1` is what gives it. Handed a corpus and some new files, it runs every
new file once, and keeps one only when it reaches a branch of the code, or takes
a branch a number of times, that nothing already kept does. So every branch a
dropped file reaches, some kept file reaches too. "Strictly one per group" would
also have needed groups the replay does not make: a file that passes has no
panic to group it by, and all of a target's passing files land in one bucket.

**The kept file goes into the corpus, not back into `artifacts/`.** libFuzzer
never reads `fuzz/artifacts/` back as a seed. On 2026-09-13 a night reported
"nothing found" on the very commit the night before had crashed on, with nothing
fixed in between (the comment over `copy_out`). A file kept only in
`artifacts/` is replayed by the morning and never by the night. A file in the
corpus is replayed by every night before it fuzzes, and fuzzed from. This
corrects what Mattias was told in chat first: that a fix that broke would likely
be found again the first night. Not from `artifacts/` it would not.

**What this gives up.** Today the morning replays every file, so a fix that
broke would fail on every one of the 68 158 that reaches the broken code. After
this, nothing replays the dropped files. A crash can need several branches taken
together that no one kept file takes, so a fix that broke need not fail on any
kept file. Every corpus makes the same bet for every input it drops, and
Mattias's argument is why it is a fair one here: nothing makes these files more
worth testing than any other input down the same paths. The fault's own
regression test is the test the fixing row landed, which fails without the fix
(CLAUDE.md, rule 2).

### 434.3 What the morning does

After the replay:

1. **Only in a tree standing on the night's commit.** That tree has the night's
   fuzz binaries in it, and its `fuzz/artifacts/` is what the night copies out.
   A hand run anywhere else, a run naming an older night, and the morning after
   a night whose build failed do not fold, and say why.
2. **The fold.** Each target's passing crash files are merged into
   `~/rcad-fuzz-corpus/<target>` (`fuzz_corpus_root`) with `-merge=1
   -timeout=10`, by the night's own binary under
   `fuzz/target/x86_64-unknown-linux-gnu/release/`, which needs no build. The
   merge writes into a copy of the corpus, and the copy replaces it only when
   the merge succeeded, as the reader's nightly merge does (§295.7, the comment
   over `merge_tmp`). The tracked `fuzz/corpus/<target>` is not counted as
   kept, so a file can be kept for a path only a tracked file takes.
3. **A file leaves the tree's `fuzz/artifacts/` only when the replay saw it pass
   and the merge read it to the end.** The night's binary is older than the
   `main` the replay ran: on 10-11 the night stood on cc97ce85 and the replay on
   a5d31701, 24 commits later. A file fixed in between still crashes the night's binary, and one
   that runs past ten seconds is not read either. Both stay, and the next
   morning tries again with the next night's binary.
4. **Nothing is cut on the way in.** libFuzzer names a corpus file for the
   SHA-1 of its bytes, as it named the crash file, so a kept file whose name is
   not its crash file's hash was cut and is a different input. The fold checks
   it.
5. **The fold adds nothing to `fuzz/artifacts/`, and removes no target's
   directory.** What libFuzzer writes for an input that crashes or hangs the
   merge goes to a scratch directory; in `fuzz/artifacts/` it would be carried
   as if a night had found it. An emptied target directory stays, for 434.5's
   last bullet.
6. **A record of what left.** Each removed file gets one line in
   `~/rcad-fuzz-logs/removed-crash-files.tsv`, appended and never rewritten: the
   night, the target, the file's name, the replayed `main` it passed on, and
   whether the merge kept it. Each night's directory keeps its own copy of every
   file, so a row naming a removed file can still replay it from
   `~/rcad-fuzz-logs/<night>/artifacts/<target>/`.
7. **The triage says what it did:** per target, how many files it folded, how
   many the merge kept, and how many stay for the next morning.

### 434.4 When a fault comes back

A fix that breaks shows one of two ways.

- **A kept file crashes again.** The night replays it and writes it back under
  its crash name, which is the same hash by 434.3's fourth rule. The name was
  removed from `fuzz/artifacts/`, so the night counts it as tonight's find and
  fails (§134). The morning replays it, it fails on `main`, and no row names its
  group: the row whose fix closed it is done, its sha is in the replayed `main`,
  and §418.4 counts a done row as naming its group only while its sha is not.
  So the triage raises it.
- **Only a dropped file would crash.** Nothing replays it. If the night finds
  the fault again, from the kept files down the same branches, its crash file
  has a new name, which no row names, and it is raised the same way.

The first is not yet shown. Q1731 shows it once: a crash file that
fails on `main`, put into a scratch corpus and fuzzed briefly with the night's
flags, comes back under its own crash name.

### 434.5 What still holds

- **§410.3.** A file leaves only after a replay has seen it pass, so a missed
  morning carries every file one more night. "Every night's `artifacts/` holds
  every crash file of the nights before it" now reads: every crash file not yet
  seen to pass and read by the merge.
- **§418.4.** Only files that pass leave, so a failing group keeps every file,
  and a file a row names stays in its group as long as the group fails.
- **§433.8's close without a lane's fix.** Mattias's argument settles it as
  well: the row whose fix closed the fault already landed a test that fails
  without that fix, so the second row has nothing left to pin.
- **Q1728's prep fact.** As filed, prep calls a row's crash files passing when
  each is under the latest night's `artifacts/` and not in its
  `fuzz-triage-members.tsv`. A removed file is in neither, and looks like a file
  that never existed. So prep reads `removed-crash-files.tsv` too: a file named
  there, and not under the latest night's `artifacts/`, passed on the `main` its
  line names. That is Q1731's, after Q1728.
- **The night's own summary** diffs `fuzz/artifacts/` against the names it held
  when the night began (`artifact-names-before`). A name removed in the morning
  and found again that night counts as new. That is right: it crashed again.
- **`fuzz-yield`'s new files** (`tooling/fuzz-yield/nights.py`) are the names in
  a night's `artifacts/<target>/` that the nearest earlier night with that
  directory lacked. Removing files adds no name, and a removed file found again
  counts as new that night, as it should. That needs every night to keep the
  target's directory: `nearest_earlier` skips a night without one, and would
  then diff against an older night that still held the removed name. 434.3's
  fifth rule keeps it. No change there.

### 434.6 What this leaves

- **Q1731**, after Q1728, builds 434.3 in `fuzz-triage`, pins each rule in
  `check-fuzz-triage-selftest`, extends Q1728's prep fact, and shows 434.4's
  first case once. It measures what is not measured here: how many passing
  files the first fold reads and keeps, per target, and how long it takes. The
  first fold is the largest there will be, about 68 000 files. The triage's
  `TimeoutStartSec=14400` and `check-triage-waits-for-the-night` bound the
  morning, and the lane may run the first fold once by hand rather than leave
  it to 08:30.
- **13 092 timeout files and 6 718 slow-unit files** are carried the same way
  and never replayed (10-11's `artifacts/`). Not this section's.
- **The night directories' own copies.** 3.0 GB, and nothing prunes them. Not
  this section's either, and 434.3's record needs them.
