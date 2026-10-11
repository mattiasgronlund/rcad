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

### 433.8 A row that no longer reproduces is closed without a lane's fix

*Mattias, 2026-10-11, on which faults were fixed: "what I meant by a fault was
when a row identified a root cause and fixed that, and that was what I expected
that fixing such a root cause would fix a large set of failed files as they all
run into the same root problem. What I was after in my question was that if we
report rows for a set of files day one then add a new row for a new set of
files day two, it might be that both rows actually are created related to files
where the root cause was the same for all of them, i.e. fixing the root cause
for one of the rows implicitly fixed the other row. That is the second row
investigated would not reproduce using the latest checking on origin/main. In
which case I hope it would be possible to close the second row without any
deeper investigation."*

Q1708 and Q1709, filed first under this number, read the question as which
merge fixed each crash file. They were dropped.

It happens. On 10-11's replayed `main` (a5d31701), 28 groups of crash files
still fail. Rows name 25 of them by a crash file, and 12 of those 25 by more
than one row. Of the 14 open rows that name a crash file, 12 name only files
that still fail. Q1316 names one file, and it passes. Q1487 names seven, and
four of them pass. Q1487's own text says Q1316's file is its bar: a round bar
whose slanted cap crosses its top. Measured at 14:40Z with a script matching the
open rows' `crash-<hex>` words against the night's `artifacts/` and
`fuzz-triage-members.tsv`.

Nothing lets a lane stop there today. A lane reports `fixed`,
`limit_demonstrated`, `partial`, `blocked` or `relay`, and its verifier wants a
test that fails on the base. A lane on a row whose files pass has nowhere to
say so.

So:

- **A lane whose row names the inputs that show its fault runs them on its base
  first.** These are the crash files, the corpus files, and any shape the row
  says to build. If none of them shows the fault, the lane stops, makes no
  commit, and reports `does_not_reproduce`, giving each command and what it
  printed. It does not look for why.
- **Its verifier runs the same commands on the base**, and confirms only when
  none of them shows the fault.
- **The row is then closed `done:<base sha>`.** It is `done` because the fault
  is gone from `main`, and the sha is the base because that is where it was
  shown gone. The merge that fixed it lies between the last replay that failed
  its files and the base. Finding that merge is a bisect, and it is not filed
  until someone asks which fix closed which rows.
- **A row closed this way comes back if its fault does.** §418.4 counts a done
  row as naming its group only while its sha is not in the replayed `main`. If
  one of its files fails again on a later `main`, its group is unnamed, and the
  triage files a row for it.
- **Prep says so before the lane starts.** The run's prep stage (§396) reads the
  latest night. When every crash file the row names passes on that night's
  replayed `main`, the brief says so and names the sha.

The lane runs the row's own shape, and not only its files, because of rows like
Q1316. Its file passes, but the row also names a plate moved 2^-1007 that
`validate_solid` refused `TooSmall` at 3741c5a7, built through the public API.
That shape shows the fault as much as the file does. Whether it still fails
was not checked here.

What this leaves:

- **Q1728**: `does_not_reproduce` in the lane, verifier, merger and prep
  instructions, in `lanes.js` and in the dispatch-lanes skill, with
  `check-lanes-workflow` arms.
