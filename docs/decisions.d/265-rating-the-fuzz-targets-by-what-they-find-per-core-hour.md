## 265. Rating the fuzz targets by what they find per core-hour — 2026-09-26

*Asked by Mattias on 2026-09-26: "Would it be possible to rate each separate
test task which we allocate resources to by the amount of faults it finds
divided by the amount of resources (time/CPUs) it was allocated / used? My
feeling is that giving more resources to test tasks which finds more per
allocated resource would help us find as much problems as possible each night,
or what have I missed?" This section is the answer, measured from
`~/rcad-fuzz-logs/2026-09-20` to `2026-09-26`, and the four rows it left:
Q146 to Q149.*

  265.1   The answer
  265.2   A crash file is not a fault
  265.3   Too few finds to rank
  265.4   Coverage creeps all night, and jumps only when a target runs fast
  265.5   A core is not a unit of search
  265.6   What the tests check moves more than where the cores go
  265.7   What is queued

### 265.1 The answer

The idea is sound, and it is how a fuzzing farm is usually scheduled: treat
each target as a slot machine and give the next hour to the one most likely to
pay. Two things stop us doing it on the numbers we have. The count of faults
does not exist yet — what the night counts is crash files, which are not
faults (§265.2). And the faults that do exist are too few to tell any target
from any other (§265.3). What the logs do support is a cheaper signal and a
different lever: runs and coverage per core-hour, which every log already
holds (§265.4, §265.5), and the checks the tests make (§265.6).

Today every target gets the same share: `fuzz-overnight` starts all five at
once for the whole window with `-fork` set to `nproc / 5`, 4 on this machine,
so about 32 core-hours each.

### 265.2 A crash file is not a fault

The night of 2026-09-26 wrote **22 528** new crash files for the writer target
and **95** for the fuse (`~/rcad-fuzz-logs/latest`). A ratio built on those
would call the writer about 240 times the better target. The writer crashed in
its second job, 19 s into the night, and kept on.

How many faults those files are could not be read off the log. In fork mode the
parent's log keeps `ERROR: libFuzzer: deadly signal` for each crash and not the
panic message, so nothing in `fuzz-writer.log` tells one crash from another.
`fuzz-triage` does decode each file and prints the panic's message and place,
but one `cargo run` per file, and it keeps no record of what it decided. So the
number the question needs — distinct faults that still fail on `main`, per
target per night — lives today only in the prose of sections like §181 and
§228. That is Q147.

### 265.3 Too few finds to rank

What the seven nights found, from §226.3 and the `latest` line: one fault on
09-20 (four crashes, §181), one crash of an already-fixed family on 09-21, and
nothing from 09-22 to 09-25. 09-26 is not triaged.

A target that finds nothing in a period is consistent, at 95%, with a true rate
anywhere from 0 to 3.7 faults per period; one find is consistent with 0.025 to
5.6. Those two ranges overlap almost entirely, so "0 per 32 core-hours" and
"1 per 32 core-hours" are not a difference anyone can act on. (Exact Poisson
bounds, `P(X ≤ k; μ) = 0.025` solved by bisection.) A ranking by faults needs
either many more faults or many more nights than a week.

### 265.4 Coverage creeps all night, and jumps only when a target runs fast

Read from each log's `#N: cov: …` lines: the coverage at or before 30 minutes,
at the end, and the hour of the last line where it rose. Thirty-five
target-nights:

- On **30 of 35** the coverage gained after the first half hour was under 1% of
  the night's total. The median target-night gained 0.37%.
- It is not flat: on 26 of 35 the last rise came after hour 6. So a target
  does keep finding new ground all night, slowly.
- The three largest late gains:

| night | target | runs | cov at 0.5 h | cov at end | gained |
|---|---|---|---|---|---|
| 09-20 | cones and tori | 160 M | 2 423 | 3 005 | 19.4% |
| 09-22 | fuse | 36.7 M | 17 112 | 17 778 | 3.8% |
| 09-26 | writer | 3.9 M | 11 133 | 11 352 | 1.9% |

The first two are the two nights those targets ran fast. The cone-or-torus
target made 2.0 M to 2.3 M runs on every other night, so 09-20 was 70 to 80
times its usual; the fuse makes 2.0 M to 4.6 M, so 09-22 was 8 to 18 times its usual.
The third is the first night of a rewritten writer target (§241).

So for the fuse and the cone-or-torus target, the evidence points at **speed
per input** as what limits a night, not a search space that has run out. That
makes the slow-input rows already in tier 2.5 (Q10, Q12, Q60, Q62) the cheaper
way to buy those targets more search. Two nights are not a demonstration, and
§265.7 does not treat them as one.

The large steps between nights — the fuse from 18 359 at the end of 09-23 to
19 009 at the start of 09-24, and from 19 130 to 20 306 into 09-26 — are new
code arriving with a new commit. New code brings new edges to count, so those
steps are not finds. Whether *faults* also arrive with new code is the question
Q146's report, fed by Q147's verdicts, has to answer before anything allocates
on it.

### 265.5 A core is not a unit of search

At the same four cores, the writer made 2 957 M runs on 09-25 and the fuse
2.0 M, about 1 400 times fewer. "Resources used" has to be core-seconds, and
"search done" has to be runs or coverage — never one standing for the other.

More cores for one target is also not more search in proportion. Four `-fork`
jobs on a target that is gaining under 1% mostly search the same ground. And
load is the cause §233 left standing for the fast timeouts, so moving cores
onto one target can add timeouts that are not findings.

### 265.6 What the tests check moves more than where the cores go

§226.3 measured that the night went from about 8 core-hours to 160, twenty
times, and found nothing new from 09-22 to 09-25. In the same week new
property-test families found twelve wrong answers: §191's five, §187's two and
§223's five. The writer and reader targets check that nothing panicked; the
wrong answers needed a check that knows the right answer.

So the ratio Mattias asked for is worth more across **kinds** of night work
than across the five fuzz targets alone: a property family run with fresh seeds
for a night is a candidate for the same cores. Whether it keeps finding when
nothing new has been written is not measured. Those families found their
twelve when they were new, which is the same shape as §265.4's steps. That is
Q149. This is §171.5's kind C.

### 265.7 What is queued

Four tier-1 rows, in the order they depend on each other:

- **Q146** — `just fuzz-yield`: per night and per target, core-hours, runs,
  coverage at the start, at 30 minutes and at the end, the hour of the last
  rise, and new files by kind; plus distinct faults once Q147 records them.
  The script behind §265.4 is most of the first half.
- **Q147** — `fuzz-triage` groups a night's crash-class inputs by the panic's
  place, decodes one input per group rather than every file, and writes the
  night's verdicts to a file Q146 reads. 09-26's 22 528 writer files are the
  case it has to handle.
- **Q148** — after Q146 and Q147: a split of the night's cores that moves them
  between targets from one slice of the night to the next. It is built only if
  Q146's numbers, over nights the report covers, show it beating the equal
  split on runs and coverage. libFuzzer cannot change `-fork` mid-run, so a
  slice is a restart, and a restart reloads the corpus — for the reader,
  256 to 282 s once the STEP corpus is passed in (§255). Every target keeps at
  least one fork: the one that found nothing is the one new code will wake up.
- **Q149** — measure what `fuses_that_must_agree.rs` and §191's families find
  per core-hour when run with fresh seeds for hours, so a property family and a
  fuzz target can be compared on one scale.

None of the four sets an allocation by past faults. That waits for Q146 to have
enough triaged nights to separate one target from another (§265.3).
