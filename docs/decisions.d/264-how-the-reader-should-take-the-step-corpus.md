## 264. With `-max_len=65536` the reader keeps nine tenths of the STEP corpus's edges at a tenth of its rate — 2026-09-26

*Queue row Q143, from §258.8 and §258.9. Lane `lane-q143-reader-arms-d-and-c`,
off `ae612730`. It runs §258.8's arms C and D beside arm B, twice, with the
reader binary §258.1 built at `8c76f406`. `git diff --stat 8c76f406 ae612730 --
fuzz/ crates/rcad-step/src` prints nothing, so the reader has not changed since.
Every number below was run on this machine on 2026-09-26. A figure from §258 is
quoted as §258's. The source citations are to `libfuzzer-sys-0.4.13`, as in
§258.*

  264.1   What was run
  264.2   Two rounds, four arms
  264.3   -max_len=5110 keeps almost nothing the STEP files buy
  264.4   -max_len=65536 keeps most of it, at a tenth of the rate
  264.5   Without -max_len, the merge itself times out on the biggest files
  264.6   What each arm writes into the night's corpus
  264.7   How T7 should pass the corpus
  264.8   What is left

### 264.1 What was run

Four arms at the same moment, side by side, for 900 s each; then the same four
again. Every arm had the night's flags as §258.1 lists them: `-fork=4
-timeout=10 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1
-dict=step.dict -max_total_time=900`. Each got a fresh scratch copy of
`fuzz/corpus/no_step_file_makes_the_reader_panic` (909 files, largest 5 110
bytes) as its first, writable directory, its own `-artifact_prefix`, and its
own `RCAD_FUZZ_TIMINGS` directory. Every arm set that variable, because §259.8
found it changes how many inputs libFuzzer runs.

- **B**: nothing else. The reader as it runs today.
- **C**: the 516 STEP files that are not zips, as a second, read-only
  directory, last. This is §258's `step-no-zip/` directory of symlinks.
- **D1**: `RCAD_STEP_CORPUS` (520 files, 392 005 383 bytes) as a second,
  read-only directory, last, and `-max_len=5110`.
- **D2**: the same, with `-max_len=65536`.

Round 1 ran 09:03:49 to 09:19:29Z, round 2 09:20:35 to 09:35:56Z. The
one-minute load average read 5.25 before round 1, 22.49 before round 2 and
30.77 after it, because other lanes were running. **So a rate here compares
with the other arms of its own round, and with nothing else.** That includes
§258.2: arm B ran at 25 148 execs/s there and at 10 580 and 9 845 here. The
load differed, and §258's runs did not set `RCAD_FUZZ_TIMINGS`. Their launch
commands, in that lane's transcript, carry no such variable.

The rate is §258's measure: the fork parent's own execution count, from the
first job line past 600 s to the last, as Q133's `summarize.sh` prints it. A
new file is a name in the writable directory after the run that was not in it
before.

### 264.2 Two rounds, four arms

Each cell is round 1 / round 2.

| | B | C | D1 | D2 |
|---|---|---|---|---|
| seed inputs after the merge | 874 / 874 | 1 185 / 1 176 | 903 / 905 | 989 / 998 |
| first job line | 2 s / 2 s | 354 s / 476 s | 3 s / 3 s | 11 s / 23 s |
| `cov` at the first job line | 3 971 / 3 971 | 11 856 / 11 857 | 3 978 / 4 019 | 10 905 / 10 918 |
| `ft` at the first job line | 7 916 / 7 919 | 50 149 / 49 726 | 7 987 / 7 986 | 33 838 / 33 952 |
| execs/s, from about 600 s to the end | 10 580 / 9 845 | 36 / 76 | 10 386 / 10 200 | 959 / 937 |
| the same, as a share of B | — | 0.34% / 0.77% | 98% / 104% | 9.1% / 9.5% |
| jobs | 82 / 82 | 51 / 47 | 82 / 82 | 80 / 79 |
| jobs ended on `-timeout` | 0 / 0 | 16 / 3 | 0 / 0 | 0 / 0 |
| `timeout-` files | 0 / 0 | 15 / 14 | 0 / 0 | 0 / 0 |
| `slow-unit-` files | 0 / 0 | 7 / 1 | 0 / 0 | 0 / 0 |
| `cov` at the last job line | 4 508 / 4 501 | 11 969 / 12 006 | 4 542 / 4 574 | 11 664 / 11 568 |
| `ft` at the last job line | 12 259 / 11 003 | 50 781 / 50 643 | 12 200 / 12 239 | 39 867 / 40 661 |
| exit status | 0 / 0 | 70 / 0 | 0 / 0 | 0 / 0 |

The two rounds agree closely on everything except arm C's merge and rate.
§264.5 is why the merge moved.

### 264.3 -max_len=5110 keeps almost nothing the STEP files buy

D1's first job line reported 3 978 and 4 019 edges, and B's 3 971. So the 520
STEP files, run at 5 110 bytes, added 7 and 48 edges, and 29 and 31 kept seeds.
Past the merge, D1 ran at B's rate and ended where B did. Six of the 520 files
are 5 110 bytes or shorter. The merge ran the other 514 cut to their first
5 110 bytes, as §258.7 read from the source. **D1 is arm B with a three-second
merge in front of it.**

### 264.4 -max_len=65536 keeps most of it, at a tenth of the rate

Against arm C, which ran every file whole:

| D2 as a share of C | round 1 | round 2 |
|---|---|---|
| `cov` at the first job line | 92.0% | 92.1% |
| `ft` at the first job line | 67.5% | 68.3% |
| `cov` at the last job line | 97.5% | 96.4% |
| `ft` at the last job line | 78.5% | 80.3% |
| execs/s past 600 s | 26.6 times | 12.3 times |

Against §258.6's arm A (`cov: 11159 ft: 48170`), D2's first job line has 97.7%
and 97.8% of the edges, and 70.2% and 70.5% of the features. 124 of the 520
files fit whole in 65 536 bytes, and the other 396 were cut. How D2's coverage
splits between those two groups was not measured.

D2 also found more in its fifteen minutes than any other arm, in both rounds:

| gained from the first job line to the last | B | C | D1 | D2 |
|---|---|---|---|---|
| edges | 537 / 530 | 113 / 149 | 564 / 555 | 759 / 650 |
| features | 4 343 / 3 084 | 632 / 917 | 4 213 / 4 253 | 6 029 / 6 709 |

So D2 starts at about 2.7 times B's edges and runs at a tenth of B's rate, and
still gains more than B does per quarter hour. §258.4's warning applies
unchanged: no job here had a budget over 82 s, and a night reaches 300 s jobs.

Arm C's first job line was above §258.6's arm A in both rounds, by about 700
edges and 1 600 to 2 000 features, although C has four fewer files. Why was not
looked into. Arm A ran once, under another load.

### 264.5 Without -max_len, the merge itself times out on the biggest files

Arm C's merge took 354 s in round 1 and 476 s in round 2. Its `timeout-` files
say why the two differ. 4 of round 1's were written before its first job line,
and 12 of round 2's. Every one of them is a file of 0.71 to 1.01 MB. Round 2
ran at the higher load, more of the biggest files took longer than
`-timeout=10`, and its merge ran 122 s longer. A night runs five targets at
once, so C's merge time is set by what else the machine is doing.

Round 1's arm C also **exited with status 70**, libFuzzer's timeout code,
though `-ignore_timeouts=1` was set. In fork mode the parent takes its exit
code from each job as the job ends (`FuzzerFork.cpp:388`), counts a timeout
and carries on (`:438-439`), and then exits with "the exit code from the last
child process" (`:488-491`). So a fork run whose last job ended on `-timeout`
exits 70. §258's arm A had six timeouts and exited 0, so it depends on which
job ends last. `fuzz-target` ends in `cargo +nightly fuzz run` under `set -euo
pipefail`. Whether cargo-fuzz passes the 70 on, and what `fuzz-overnight` does
with it, was not checked.

### 264.6 What each arm writes into the night's corpus

| new files in the writable directory | B | C | D1 | D2 |
|---|---|---|---|---|
| count | 518 / 507 | 50 / 90 | 520 / 501 | 692 / 652 |
| bytes in all | 1 749 611 / 1 879 065 | 5 512 553 / 11 932 758 | 1 888 130 / 1 756 171 | 28 115 880 / 23 911 345 |
| median bytes | 3 826 / 3 860 | 90 740 / 76 732 | 4 062 / 4 000 | 45 159 / 40 838 |
| largest bytes | 5 110 / 5 110 | 539 038 / 1 048 576 | 5 110 / 5 110 | 65 536 / 65 536 |

Each arm's largest new file is its `-max_len`. D1 and D2 pinned theirs. B
derives 5 110 from the tracked corpus. C got 1 MiB, the cap §258.4 read from
`FuzzerLoop.cpp:806-807`, reached in round 2.

**D2 writes 24 to 28 MB per quarter hour into the first directory.** That is
§258.5's 30 MB again, cut into files of at most 64 KiB. D2 fixes the rate and
the merge. It does not fix what the night keeps.

### 264.7 How T7 should pass the corpus

- **Not as written.** As written, T7 is §258's arm A, and C here is A without
  the four zips. C holds all the coverage, but it runs at 0.3% to 0.8% of B's
  rate after a merge of six to eight minutes that grows with the load. It also
  ends jobs on `-timeout`, writes finds of up to 1 MiB, and can exit 70.
- **Not with -max_len=5110.** D1 costs nothing and buys nothing, because it is
  arm B.
- **With -max_len=65536, as D2 ran it.** That is the whole corpus, last and
  read-only, which is where `8689e498` already puts it. It keeps 92% of C's
  edges at the first job line and 96% to 97.5% by the end. It runs at 9% to 10%
  of B's rate, which is 12 to 27 times C's, with a merge of 11 to 23 s and no
  timeouts in either round. It gained more edges and features than any other
  arm in both rounds. §264.9 reads the same four things off a real night's
  logs instead of this lane's fifteen-minute, small-corpus arms.

**The pin belongs in `fuzz-target`, in the branch that appends `step_corpus`,
and not in `fuzz-reader`'s arguments.** `fuzz-reader` always passes
`RCAD_STEP_CORPUS`, and `fuzz-target` hands it on only when the directory
exists. A night with no STEP corpus fetched runs as arm B and derives 5 110.
Pinning 65 536 there as well would let arm B's mutations grow to 64 KiB, and
no arm here ran that.

**T7 should merge after Q144, not before.** D2 still puts 24 to 28 MB per
quarter hour into the directory the next night starts from (§264.6). Those
finds join every later night's merge and seed pool, and the size order in
§258.3 draws seeds from the largest. This lane measured that cost for fifteen
minutes and not for a night. Q144's long pair is the run that measures it.

### 264.8 What is left

1. **Q144's long pair, run as D2 against B.** §258.9 parked it until this row
   picked an arm, and this is the arm. Besides the steady rate, it should
   report how many bytes the first directory gains per hour, because §264.6
   leaves that cost open. T7 waits on it (§264.7).
2. **Where the reader's finds go when the STEP corpus is passed.** None of
   these was run: a writable directory the next night does not start from; a
   `-merge=1` of the finds back into the corpus at a smaller `-max_len`; or
   keeping 64 KiB finds as D2 does.
3. **A -max_len between the two, or above 65 536.** 46 files fit whole in
   32 768 bytes, and 124 in 65 536. Neither 32 768 nor 131 072 was run, so
   where the coverage falls off between D1 and D2 is not known. *Answered by
   §274 (Q158): coverage is still rising at 131 072.*
4. **Which part of D2's coverage comes from the 124 files that fit whole.** A
   merge-only run over those 124 alone would say. *Answered by §274 (Q158):
   almost all of it.*
5. **The exit status 70** (§264.5): whether cargo-fuzz passes it on, and what
   `fuzz-overnight` does when a target exits non-zero.
6. **Why C's first job line is above §258.6's arm A** by about 700 edges,
   although it has four fewer files. `RCAD_FUZZ_TIMINGS` is not most of it:
   §258.6's arm B, without the variable, ended at 4 463 edges, and this
   section's arm B, with it, at 4 508 and 4 501. *(Added at merge, by the
   dispatcher.)* Nothing waits on this answer, so it has no queue row.

`RCAD_STEP_CORPUS` still held 520 files afterwards, and none was newer than the
start of round 1. §255.6's placement holds.

### 264.9 What a real night's logs show, against the lab's figures

*Queue row Q247, replacing Q246 (which cited §264.4 for the merge time; the
right figure is §264.7's). Read-only: no fuzz binary was run for this row. Q156
merged `1b344a33` at 2026-09-27 18:52:44 +0200 (`git log -1 --format='%ad'
1b344a33`); `$HOME/rcad-fuzz-logs` is where `fuzz-overnight` writes
(`fuzz_log_root` in the justfile, unset here, so its default). Every file
named below is under that directory.*

**The first night after the merge is gone.** `fuzz-overnight` names a night's
directory by the date its own `date +%Y-%m-%d` reads at the top of the run,
and every surviving night in this directory started at 00:10 and ran to about
08:1x — so the night that began the evening of 2026-09-27 (after the 18:52
merge) is the one dated 2026-09-28, and `ls $HOME/rcad-fuzz-logs/2026-09-28`
answers "No such file or directory". `2026-09-27` itself is the night
*before* the merge: it ran 00:10–08:15 that same morning, hours before Q156
landed, and its own directory carries none of this section's machinery
(no `corpus-names/`, no `reader-corpus-merge.log`, no
`timing-hooked-calls.log`) — all four are written unconditionally by the
current recipe once the build succeeds, so their absence says that night ran
an older revision of `fuzz-overnight`, from before the scratch-and-merge
mechanism (§264.7, §295.6) existed, not that the mechanism misfired.

**The earliest night that remains, 2026-09-29, has nothing to read.** Its
directory holds only `build.log` (a clean `cargo +nightly fuzz build`,
"Finished `release` profile ... in 1m 32s") and an empty `watermark.log` — no
`fuzz-reader.log`, no `corpus-names/`, no `timing-hooked-calls.log`, nothing
under `artifacts/` or `timings/`. The run never reached the point where any
target starts fuzzing, so none of the four things this row asks about —
the reader's command line, a merge time, a timeout verdict, a byte count —
can be read off it. `2026-09-30` is the same shape. `2026-10-01` and
`2026-10-02` fail earlier still: `build.log` ends "error: could not compile
`cc` (lib) due to 11 previous errors" — the fuzz targets never built at all.

**The earliest remaining night with a completed run is 2026-10-03.** Its
`fuzz-reader.log` carries this line, unedited:

```
Running `target/x86_64-unknown-linux-gnu/release/no_step_file_makes_the_reader_panic -artifact_prefix=/home/mattias/git/mattiasgronlund/rcad-nightly-fuzz/fuzz/artifacts/no_step_file_makes_the_reader_panic/ -max_total_time=28712 -max_len=65536 -fork=4 -timeout=10 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 -dict=step.dict /home/mattias/rcad-fuzz-corpus/no_step_file_makes_the_reader_panic-scratch /home/mattias/rcad-fuzz-corpus/no_step_file_makes_the_reader_panic corpus/no_step_file_makes_the_reader_panic /home/mattias/git/mattiasgronlund/rcad-step-corpus`
```

It carries `-max_len=65536`, and its last argument,
`/home/mattias/git/mattiasgronlund/rcad-step-corpus`, is exactly what
`RCAD_STEP_CORPUS`'s default expands to in that worktree
(`parent_directory(parent_directory(git rev-parse --path-format=absolute
--git-common-dir)) / "rcad-step-corpus"`, read off the hub's shared
`.git`) — so the line ends in `RCAD_STEP_CORPUS`, not in a path that merely
looks like it. `2026-10-04` through `2026-10-06` carry the same command line
with only `-max_total_time` changed night to night.

**The merge before the first job line runs longer than §264.7's lab figure.**
Reading the `time:` field off each night's own first `job: 1` line —
the same column §264.2 reports as "first job line" — gives 31 s
(2026-10-03), 36 s (2026-10-04), 42 s (2026-10-05) and 32 s (2026-10-06): all
above §264.7's 11–23 s. By 2026-10-03 the persistent reader corpus already
held 33 735 files (`wc -l
rcad-fuzz-logs/2026-10-03/corpus-names/no_step_file_makes_the_reader_panic`,
and the same count in that night's own `reader-corpus-merge.log`, "33735
files before merging") against the few hundred to two thousand files each
lab arm started from in §264.2 — a corpus thirty-plus times the size would
plausibly take fork mode longer to load before job 1, but that cause is not
measured here, only the four nights' consistency with each other.

**No job ended on `-timeout` in any of the four nights.** Every `job:` line
in each night's `fuzz-reader.log` reads `oom/timeout/crash: 0/0/0`; each night
ends `INFO: exiting: 0`; and `find
rcad-fuzz-logs/<night>/artifacts/no_step_file_makes_the_reader_panic
-name 'timeout-*'` is empty for 2026-10-03 through 2026-10-06 alike.

**The writable corpus directory gained well under §291.3's ≈18.5 MB/h.** The
reader's first, writable directory is
`$RCAD_FUZZ_CORPUS/no_step_file_makes_the_reader_panic-scratch`
(`fuzz-target`'s `$scratch`), and `fuzz-overnight` empties it with `rm -rf`
only after a *successful* end-of-night merge (`reader-corpus-merge.log`:
"persistent has N files after merging"). Three consecutive nights' own logs
confirm their scratch directory started empty, because each night's
"persistent has N files before merging" matches the previous night's own
"after merging" count (33796 → 33801 → 33806 → 33839 across
2026-10-03/04/05/06):

| night | scratch bytes at merge | run seconds | MB/h |
|---|---|---|---|
| 2026-10-04 | 58 088 615 | 28 764 | 7.27 |
| 2026-10-05 | 54 118 692 | 28 727 | 6.79 |
| 2026-10-06 | 56 392 898 | 28 800 | 7.05 |

(bytes from each night's `reader-corpus-merge.log`, "scratch is N bytes";
seconds from that night's `fuzz-reader.log`, "INFO: exiting: 0 time: Ns";
MB/h = bytes ÷ (seconds/3600) ÷ 1 000 000.) All three sit at 37–39% of
§291.3's rate. 2026-10-03's own scratch held 54 409 439 bytes over 28 713 s
(≈6.82 MB/h), in the same range, but whether *its* scratch started empty
cannot be read off these logs — the only earlier complete night,
2026-09-27, predates the scratch mechanism (above), so there is no earlier
"after merging" count to compare it against. A non-empty start would only
lower 2026-10-03's true rate, so this does not weaken the finding that a real
night gains bytes well below §291.3's figure, measured on D2/65536 for
fifteen minutes rather than a night.
