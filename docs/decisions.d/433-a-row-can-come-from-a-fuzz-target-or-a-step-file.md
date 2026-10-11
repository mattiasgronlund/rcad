## 433. A row can come from a fuzz target or a STEP file — 2026-10-11

*Mattias, 2026-10-11, told that a row filed from a fuzz finding says `from/-`:
"You are right the row should have a parent, it should just not be a row. So we
can continue to measure fanout, and if we then identify the actual fuzz-type as
the parent, we could actually find out: What fan out rows from respective
fuzz-type have had over time. How many issues have been found by each
fuzz-type. And it should not just be fuzz-type we give as parent, it should
also include things as a STEP-file, so we could learn more about what fixing
specifc STEP-file:s ment."*

This amends §432.2's word, which had `from/-` cover "a fuzz night found it".

### 433.1 Two more words

- `from/fuzz:<target>`: a crash file or seed of fuzz target `<target>` shows
  the fault. `<target>` is a file name under `fuzz/fuzz_targets/` without its
  `.rs`, as in `from/fuzz:where_two_surfaces_cross`.
- `from/step:<file>`: a file of the STEP corpus shows the fault. `<file>` is
  its path in `corpus/manifest.sha256`, cut after a `/` as close to the end as
  still names one file, with each space written `_`. 345 of the 367 files need
  only their own name, as in `from/step:LED_D5.0mm.step`. The other 22 are
  eleven NIST names, each used once under `CTC Definitions` or `FTC
  Definitions` and once under `AP203 geometry only`, with different bytes, so
  they keep one directory: `from/step:CTC_Definitions/nist_ctc_01_asme1_rd.stp`.

`was/` still takes only a row. `from/-` keeps the rest: Mattias asked for it,
or a session found it by looking.

### 433.2 What found the fault beats the row that was running

A row has one parent. Q68, a sound model whose own written file the reader
refuses, was found by the writer target in the first minutes after Q58's lane
made its seeds reach the writer (c0b29166). Both Q58 and the target have a
claim on it. The word names the target.

Both of Mattias's questions are asked per target: how many faults it found, and
how far the work on them grew. A fault hung on the row that happened to be
running is missing from both counts. Q58 made the seeds reach the writer. It
did not write the faults the writer target then found.

A row that fixing a fault leads to stays `from/<the fault's row>`. So a target's
tree holds every fault it found, plus every row the work on them filed. The
same goes for a corpus file.

### 433.3 The triage row

§418.3's triage row is `from/-`. §418 files it because a night left groups no
row names. No row's run led to it.

Its lane starts each fault row it reports with `from/fuzz:<target>`, the target
whose crash file that row names. The file sits under `artifacts/<target>/` in
the night's directory, and `fuzz-triage-members.tsv` names the target in its
first column. A triage row then has no children, which fits: it sorts what the
night found and finds nothing itself.

If the lane leaves the word out, `just land` writes `from/<the triage row>`
(§432.4). The row is still counted, but under the wrong parent.

### 433.4 What is checked, and when

`just queue add` checks the name. The target's `.rs` must be under
`fuzz/fuzz_targets/`. For a corpus file, the name with its spaces put back must
be the end of exactly one path in `corpus/manifest.sha256`, starting just after
a `/`.

`check-queue` checks only the shape: a kind it knows, then a name with no space
in it. A target can be renamed, or a file can leave the corpus, after a row
names it. The row still says truly where it came from, and a gate that failed
on it would push someone to rewrite that.

The kinds are a fixed list, so a misspelt kind is refused rather than counted
as a new source.

### 433.5 Kinds not added yet

Other things than nights and corpus files have filed rows. The OCCT test corpus
run by §242's `rcad_testing` is one, a property suite's failing seed another.
Each gets a kind when a row needs it counted (§13). Until then such a row is
`from/-`, and its text names the source.

### 433.6 Older rows

Rows filed before §432 carry no word. §432.1's commit inference gives most fuzz
faults no parent at all. Of its 221 rows with no parent, 17 name a fuzz target
in the row or in the subject of the commit that added it, and 34 more mention
fuzzing, a night or a crash file there. That is a keyword match, not a reading
of each row.

An older row whose text names the input that shows its fault can give that
input back as its source: a crash file, a seed by its hash, or one corpus file.
A row that names a target or a file in passing cannot. Q58 names the writer
target and is not one of its faults. Telling those apart is a measurement, with
its own row.

### 433.7 What this leaves

- **Q1703**: the two words in `just queue add`, `check-queue`, their
  selftests, and in the lane, merger, verifier and dispatch-lanes
  instructions.
- **Q1704**, `was/Q1696`, after Q1703: §432.7's growth recipe, which now also
  prints each target's and each corpus file's tree by week.
- **Q1705**, `was/Q1429`, after Q1447 and Q1703: the triage row that
  `fuzz-triage-row` files says `from/-` and tells its lane to name each fault
  row's target. It waits on Q1703 because a triage lane that writes the word
  before `just queue add` takes it would stop `just land` at its queue step.
- **Q1707**, `was/Q1706`, after Q1704: reading the source of an older row back
  from its text, with 30 links checked by hand. Q1706 read it only from rows
  filed before this section. A fault row filed by hand between this section and
  Q1703's merge can say only `from/-` or `from/<the triage row>`, so Q1707 reads
  every row filed before Q1703 merged.

Mattias pinned Q1703, Q1447, Q1704, Q1705 and Q1707, in that order, on the same
day (a6cd6135). Q1447 is in the list because Q1705 waits on it, and a pinned row
still waits on its `after:` (§430.4).

### 433.8 A fault is not a row

Mattias asked next whether this tells which faults were fixed, not only which
rows were closed. It does not. Q1703 to Q1707 count rows.

A row is not a fault. On 10-11's main, 28 groups of crash files still fail. Rows
name 25 of them by a crash file, and 12 of those 25 are named by more than one
row: a row filed again, or a done row whose files still fail, as `writer.rs:226`
is named by Q1209 (done) and by Q1295 and Q1297 (open). Q1474 alone names 7
groups. A group is not a fault either. The triage puts crash files with one
panic place and message in one group, and a merge can fix part of a group.

So here a fault is **the set of inputs one merge turns from failing to
passing**. Passing is the replay's word: the input no longer panics, whether it
now succeeds or is refused. The merge is a first-parent commit of `main`. Two
faults fixed in one merge count as one, so the count is a lower bound.

What is on disk, measured at 14:10Z over `/home/mattias/rcad-fuzz-logs` with a
script reading the night directories:

- A night keeps every earlier night's crash files. None of the 69 879 crash
  files of 10-06 is missing from a later night, so an input that turned is
  still there to replay (§410.3).
- Since 10-09 the morning triage replays every crash file against that
  morning's `main`. It keeps which files fail (`fuzz-triage-members.tsv`) and
  against which commit (`fuzz-triage-replayed-main.txt`). 6 301, 8 507 and
  11 201 files failed on 10-09, 10-10 and 10-11.
- Between the 10-09 and 10-10 replays one file stopped failing, the only file
  of its group (`assemble.rs:2556`, a piece of a face with a straight side of no
  length). 131 first-parent commits lie between the two, and nothing records
  which of them turned it. Between 10-10 and 10-11 none stopped, over 76.
- 68 158 of 10-11's 79 359 crash files no longer fail, and 68 157 of those did
  not fail at the first replay on 10-09. Which merges fixed them is recorded
  nowhere.

For KiCad, git already holds the answer. `KICAD_FILES_THAT_IMPORT` names every
KiCad file that imports, and its test fails when one starts or stops. 11
first-parent commits of `main` changed it, adding 165 names and removing none.
`KICAD_FILES_THAT_ROUND_TRIP` was changed by 6 commits, adding 165. The first
commit of each list says what worked when the list was written, not what that
commit fixed, so it counts no fault. NIST files are not pinned by name.
`how_often_a_real_file_imports` prints how many import and asserts nothing about
which.

What this leaves:

- **Q1708**: each morning's triage names the merge that turned each file that
  stopped failing, by bisecting the first-parent commits between the two
  replays, and keeps one line per fault. `just faults` prints, per fuzz target
  and per KiCad file and by week, the faults fixed, the inputs they turned, and
  the rows closed in the merges that turned them.
- **Q1709**: the NIST files that import are pinned by name, as KiCad's are, so
  a merge that makes one import is seen in git.
- The 68 157 files that turned before 10-09 are not filed. Finding their merges
  means replaying them at merges a month old, against fuzz targets that have
  changed since, and no one has measured whether those merges still build the
  replay. Filed when someone asks for faults fixed before 10-09 (§13).
