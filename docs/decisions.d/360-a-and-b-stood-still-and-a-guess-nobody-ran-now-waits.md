## 360. The queue on 09-29: A and B stood still, and a guess nobody has run now waits below every refusal — 2026-09-29

*Mattias, 2026-09-29: "The trend seems to have continued today, that for each
completed row there it has created more than one new row." This section
measures 09-29 the way §348 measured the days before it, and checks what
§352 changed. Measured on `origin/main` up to 04a469ad (16:40) unless a figure
says otherwise; §360.1 and §360.5 run to 81270468 (17:39).*

In short: 09-29 filed 1.66 rows per close, about what 09-26 to 09-28 did.
§352 made the number of rows stop mattering and made the A-or-B count the one
to watch, and that count did not fall. The walk works `wrong/C` and `crash/C`
before any refusal a user meets, and at 17:39 ten of the eleven open rows in
those two classes were guesses nobody had run. Nine guesses came back on
09-29. One found the defect it guessed at, and between them they filed 39 new
rows, 8 of them more guesses. §360.5 moves a guess nobody has run out of the
classes, so it waits below every refusal a user can meet, and re-tags the 19
open ones.

### 360.1 The numbers

Counted as §348.1 counted them: by author date, with the seed commit left out.
Open counts are first-parent.

| day | added | done | dropped | added per close | open at the end |
|---|---|---|---|---|---|
| 09-25 | 58 | 25 | 2 | 2.15 | 49 |
| 09-26 | 111 | 64 | 3 | 1.66 | 100 |
| 09-27 | 85 | 49 | 1 | 1.70 | 131 |
| 09-28 | 188 | 77 | 32 | 1.72 | 207 |
| 09-29 to 16:40 | 118 | 51 | 31 | 1.44 | 245 |
| 09-29 to 17:39 | 148 | 57 | 32 | 1.66 | 268 |

`flow.py <repo>` replays every commit that touched `docs/queue.md` and writes
`events.json`; the table is its last lines. It and the other scripts named
here (`today.py`, `measure.py`, `takes.py`, `lane_births.py`, `stale_ws.py`,
`c_rows.py`, `since.py`) were throwaway, in two session scratchpads, and are
not committed.

### 360.2 Where 09-29's first 118 rows came from

| what closed rows and added them | closes | rows added |
|---|---|---|
| a writing lane, one row per merge | 21 | 63 |
| a trace answered, and the dispatcher filed its fix rows | 13 | 28 |
| a row dropped and filed again with a wider write set | 13 | 12 |
| a bundle of small rows | 28 | 5 |
| a lane of several rows that were not small | 5 | 8 |
| a drop as moot or a duplicate (Q353, Q556) | 2 | 0 |
| rows §352 filed, closing nothing (Q500, Q501) | 0 | 2 |

- **A one-row writing lane left 3.00 rows per merge.** On 09-27 it was 1.56
  and on 09-28 2.42. 09-25 and 09-26 do not compare: lanes were closed in
  separate queue commits then.
- **The traces** were Q361, Q362, Q386, Q387, Q388, Q375, Q319, Q381, Q382,
  Q374, Q376, Q385 and Q510.
- **The re-files**: Q355→Q502, Q369→Q509, Q434→Q538, Q401→Q539, Q251→Q540,
  Q544→Q574, Q356→Q575, Q567→Q576, Q351→Q596, Q588→Q604, Q599→Q605, and Q452
  with Q552→Q585. Three of them only added the crate's `error.rs`: Q574, Q604
  and Q605. Two added the tests that pin the number the fix moves: Q502 and
  Q509. §360.6 answers the first three.
- **Bundles work.** 28 small rows closed and 5 were added, which is what
  §352.4 was for.

`lane_births.py 2026-09-29` prints each one-row merge with the rows born
beside it; `takes.py` and `today.py` gave the other lines.

### 360.3 What a writing lane's 63 rows were

One reader's labels, as §348.4's were, read against the row each merge closed:

| why the row was born | rows |
|---|---|
| new kernel work | 22 |
| the same problem's next step | 4 |
| a guess the lane did not run | 11 |
| something the lane's own change left | 20 |
| measuring | 5 |
| a refactor (Q601) | 1 |

The 20 the lane's own change left:

- **10 stale claims**: Q515, Q516, Q519, Q521, Q526, Q553, Q551, Q568, Q580
  and Q593.
- **8 missing tests or pins**: Q517, Q518, Q520, Q528, Q577, Q590, Q592 and
  Q495.
- **2 decisions sections the lane owed**: Q527 and Q586.

§352.5 says a lane fixes what its own change leaves stale, inside its write
set. That could have caught only 5 of the 20: Q517, Q519, Q521, Q528 and Q577.
The other 15 were outside the write set: 10 in `docs/decisions.d/`, 4 in other
or new test files, and Q551 in a crate's docs. `stale_ws.py` sorted them.

**The 5 inside the write set were filed because the lanes never saw §352.5.**
A running process keeps the agent files it started with, and an edit made
after that does not reach its lanes:

- §266.6 had shown this for an agent file *added* after a process starts. This
  session showed it for one *edited*. Its process started on 09-28 at 17:58.
  0930c94a added three paragraphs to `lane.md` on 09-28 at 23:24, §352.3's
  classes and §352.5 among them, and the hub and this tree both have them. A
  `lane` subagent spawned on 09-29 at about 18:00, asked to quote its own
  instructions, said it had no class paragraph. Asked for the paragraph after
  the one ending "every paragraph it moved instead.", it quoted "Budget about
  45 minutes.", which is what came next before 0930c94a.
- The live dispatcher's process (`dispatch-next`) started on 09-26 at 14:03,
  so its lanes and mergers have run 09-26's `lane.md` and `merger.md` all
  along. That is the same mechanism on another process, not a second
  observation.
- `lanes.js` is read fresh on every run. It carries §352.3's classes in the
  `remaining` description and §352.4's bundles, and that is the likeliest way
  rows filed on 09-29 got their classes. It does not carry §352.5.

Q530 and Q532 are one finding that Q354's merge filed twice. Q530 reads
`dropped:§16.10`, and Q532 is `todo` with no class. `git log -G` finds no
commit for the drop, probably because it happened inside a merge. Neither is
touched here.

### 360.4 The measure did not move

§352.6's line counts the open tier 1 rows by bucket:

| at | A or B | C | D |
|---|---|---|---|
| d523d224, 08:40, when the tags landed | 68 | 23 | 11 |
| 04a469ad, 16:40 | 68 | 39 | 12 |
| 81270468, 17:39 | 71 | 49 | 12 |

Between 08:40 and 16:40, 17 A-or-B rows were born. 5 were done (Q359, Q509,
Q373, Q350, Q539) and 11 were dropped, 7 of them to be filed again as A or B.
At 17:39 the 71 are 3 `wrong`, no `crash` and 68 `refuses`. `measure.py <repo>
2026-09-29` prints the line after each first-parent commit, and `c_rows.py`
lists the C rows.

The count stood still for three reasons that feed each other:

- **The walk takes `wrong/C` and `crash/C` first.** §352.3's order is
  `wrong/A, wrong/B, wrong/C, crash/A, crash/B, crash/C, refuses/A, ...`, and
  68 of the 71 A-or-B rows are `refuses`. So the walk reaches them only after
  every ready `wrong/C` and `crash/C` row. After the tags landed, one-row tier 1
  lanes took 15 C rows and 10 A-or-B rows. Q408 (`refuses/A`) still waits: the
  writer refuses every ball, about 21% of the writer target's input.
- **A C row leaves more C rows.** `rings_round`'s panic is one chain: Q359 →
  Q567 → Q576 → Q597 → Q598, Q599, Q600 → Q616 → Q633, Q634, Q636, Q641, Q642.
  No input has reached that panic yet.
- **Every guess comes in as C.** §352.3 said "a hypothesis nobody has run is
  C at most", and the eight guesses filed beside §360.5's nine all came in
  as C.

### 360.5 A guess nobody has run has no class until a lane runs it

Nine guesses came back on 09-29. A guess here is a row whose defect no run had
shown: it asks a lane to build a shape and see.

| row | class | what the run found | rows it filed |
|---|---|---|---|
| Q319 | wrong/C | refusals, no wrong answer | 2 |
| Q549 | wrong/C | a wrong answer, but only at 2^-1074 (D) | 4 |
| Q571 | wrong/C | the bound held | 2 |
| Q604 | wrong/C | **a wrong answer inside the envelope**: the volume depended on face order | 4 |
| Q597 | crash/C | two torus probes refuse before the panic | 5 |
| Q600 | wrong/C | the arc is never read bulging the wrong way | 4 |
| Q598 | crash/C | no panic; the lens is refused earlier | 4 |
| Q606 | wrong/C | no wrong mesh; the rim builds only the way the walk runs | 4 |
| Q616 | crash/C | no panic in any of five families | 10 |

One of the nine found the defect it guessed at, inside the envelope. Together
they filed 39 rows, and 8 of those were new `crash/C` or `wrong/C` guesses
(Q571, Q616, Q628, Q633, Q634, Q636, Q641, Q642). Each of those went ahead of
the 68 refusals. `since.py events.json 2026-09-29T00:00` prints each close with
the rows born beside it; Q597 and Q616 count as guesses by my reading, since
the panic they chase is in the code but no input has reached it.

The runs did find real things, just not the ones they guessed: Q616's probes
found Q635 and Q637 (`refuses/B`: a plate with an off-centre hole is refused),
and Q598's found three `refuses/C`. That is the cost of what follows. Those
refusals would have been found later.

Three ways were weighed, and put to Mattias as Q1:

- **(a)** Keep the order, and change the measure to count what the walk works
  first: `wrong` or `crash` in A to C, then `refuses` in A and B. The
  refusals keep waiting while guesses keep arriving.
- **(b)** A guess nobody has run has no class until a lane runs it, so it
  sorts just above D, where untagged rows already go. Rows shown real keep
  "how bad first". No tool changes.
- **(c)** A and B come before C whatever the symptom. The watched number
  falls, but a wrong answer a probe has shown waits behind 68 refusals.

**(b) is taken.** The session took it without an answer, on §138, since the
question carried a recommendation, and a re-tag undoes it. The reasons:

- **It keeps "how bad first" for every row shown real.** Mattias adopted that
  order (§352.3), and (b) does not touch it.
- **It ranks a guess by how likely it is,** which was Mattias's first
  instruction in §352: "prioritise them after the probability ... that a user
  runs into the problem". A guess nobody has run is less likely than any row a
  run has shown, and on 09-29 one guess in nine was right.
- **§352.3 already allowed it.** "C at most" allows less than C.

What changed with it:

- **§352.3 and §352.5** now say a guess nobody has run gets no class, and a
  note in each points here.
- **`lane.md` and `merger.md`** say the same. So does the `remaining`
  description in `.claude/workflows/lanes.js`, which is read fresh on every
  run. That is the only copy a running dispatcher sees before it restarts
  (§360.3).
- **19 open tier 1 rows lost their class** in one `just queue apply`, all of
  them guesses by the rule above: the `wrong/C` rows Q191, Q194 and Q428; the
  `crash/C` rows Q605, Q628, Q633, Q634, Q636, Q641 and Q642; and the
  `refuses/C` rows Q304, Q305, Q418, Q424, Q511, Q523, Q555, Q612 and Q613.
  Q560 (`wrong/C`, shown by Q510's run) was in a lane and was left alone, as
  §352.7 leaves every row in `lane:`. The 29 other C rows each name a run that
  showed the refusal, and they keep their class.

After the re-tag the walk has no ready `wrong/C` or `crash/C` row, so the next
tier 1 lane goes to `refuses/A`. §352.6's line then counts only shown rows in
C, and the guesses among "no class".

### 360.6 Two changes to merger.md

Both follow from §352.5, so they are taken without asking.

- **Step 2 gets a bullet after the one about counts.** A stale citation or a
  dated note in a decisions section, made stale by the lane's own change, is
  the merger's to fix outside the write set. That holds when the fix is a few
  lines and no running row's write set holds that file, and the merge commit
  says so. On 09-29 this would have fixed Q515, Q516, Q526, Q553, Q568, Q580
  and Q593 instead of filing them.
- **Step 3's fourth item gets a sentence:** a row whose fix adds a refusal, or
  turns a panic into one, names its crate's `error.rs`. On 09-29 that would
  have saved the three re-files that only added it: Q574, Q604 and Q605.

An edited agent file does not reach a running process (§360.3), so neither
change reaches the live dispatcher's mergers until that process restarts. The
change to `lanes.js` in §360.5 does.

### 360.7 What this leaves

- **The live dispatcher runs 09-26's role files** until its process restarts:
  no §352.3 or §352.5 in `lane.md` or `merger.md`, and none of this section.
  A restart is the whole fix for today. Nothing tells a dispatcher that its
  agent files changed under it: Q643, tier 2.2.
- **Q530 and Q532** hold one finding. Q530 is dropped and Q532 is open with no
  class, and why Q530 was dropped is not known. Whoever takes Q532 should read
  Q530's drop first.
- **The A-or-B count is still the one to watch.** If it does not fall once the
  walk reaches `refuses/A`, the cause is somewhere other than the order, and
  §360.4's figures are what to compare against.
