## 348. Why the queue grows: each close leaves 1.5 new rows, and nothing turns one away — 2026-09-28

*Mattias, 2026-09-28: "We need to do a root cause analysis, to figure out why
the job queue grow with about twice the amount of new items as we are able to
close." And then: "I would expect that most activities on rows would create
less then one new row so that unless new rows where added from something else
than an activity on a row, the number of rows should fade out relatively
fast." This section is the answer, measured on `origin/main` up to 8d72498c
(21:03 that day).*

### 348.1 The numbers

`docs/queue.md` was seeded at 9414e41c with 53 rows (§230.9). Since then 380
rows have been added and 216 closed: 197 done, 19 dropped, none parked. That
is 1.93 new rows per row done, which is the "about twice".

| day | added | closed | open at the end of the day |
|---|---|---|---|
| 09-25, from 21:29 | 58 | 27 | 49 |
| 09-26 | 111 | 67 | 100 |
| 09-27 | 85 | 50 | 131 |
| 09-28, to 21:03 | 126 | 72 | 186 |

How it was counted: each commit that changed the file, over the whole history
and not only first parents, is read against every parent it has. A row is
added at a commit when none of its parents has it, and closed there when every
parent that has it has it open. So a merge counts only what its own resolution
changed, not what it brought in. The open counts read the file as `main` had
it after each first-parent commit. The scripts were throwaway, in the session
scratchpad, and are not committed.

### 348.2 New rows are made by closing rows

331 of the 380 new rows (87%) were written in the same commit that closed
another row. The other 49 came in commits that closed nothing.

Over the thirteen six-hour windows the history covers, rows added follow rows
closed with a correlation of 0.95, at 1.64 new rows per close plus 2.0 a
window from elsewhere.

§311 swapped tiers 1 and 2 at 20:32Z on 09-27. That changed which rows closed
but not what a close leaves:

| | closes | rows added | per close | tooling and measuring closes |
|---|---|---|---|---|
| before §311 | 139 | 247 | 1.78 | 74 |
| after §311 | 77 | 133 | 1.73 | 7 |

Sorted by what the closed row wrote, every kind leaves more than one row
except docs. Rows added in a commit are split evenly over the rows it closed:

| closed row | closes | new rows per close |
|---|---|---|
| a fix in a crate's `src/` | 45 | 1.79 |
| a trace, write set `-` | 63 | 1.73 |
| a pin or a test only | 16 | 1.47 |
| measuring the tests or the nights | 36 | 1.40 |
| tooling: the queue, lanes, recipes, fuzz | 45 | 1.26 |
| docs | 9 | 0.97 |

### 348.3 Most closes do leave one row or none: the average comes from a tail

Of the 159 commits that closed exactly one row:

| new rows it left | commits | share | rows |
|---|---|---|---|
| 0 | 43 | 27% | 0 |
| 1 | 50 | 31% | 50 |
| 2 | 33 | 21% | 66 |
| 3 to 8 | 33 | 21% | 126 |

The middle close leaves one row. Without the tail the average is 0.92. With
it the average is 1.52, and the fifth of closes that leave three or more make
52% of the rows.

A close that leaves exactly one row is not the work shrinking. The one row is
usually the same problem's next step, because §230.5 makes a defect two rows,
a trace and then its fix. For the queue to fade, most closes would have to
leave nothing, and 27% do.

Two of the tail, as they were written:

- **Q54, a fix, left five rows, Q101–Q105.** The fix let a bend get further
  through the fuse, and five things further on failed: a broken write-back, a
  candidate point that is a box, a validate refusal, a puck's rim, and a bend
  off the coordinate axes that Q54's verifier found by trying other planes.
  Five real defects, each distinct.
- **Q147, a triage tool, left eight rows.** Seven of them, Q166–Q172, are the
  09-26 night's fuzz crashes, which could be counted once the tool grouped
  them. They are the fuzzer's finds coming in through a row that closed, not
  work the row made.

### 348.4 Why each new row exists

A subagent sorted all 331 rows born beside a close by why each one exists,
reading each against the rows its commit closed. I checked the tally against
its file and read ten labels at random against their rows; all ten read right,
and Q425's "coverage" could as well have been "pin". The labels are one
reader's judgement, not a measurement.

| why the row was born | rows | per close |
|---|---|---|
| **New kernel work found**: a defect seen in passing (38), the next refusal once a fix lets a shape further (22), the same finding in other families (28), a proof that a refusal is a limit (7) | 95 | 0.44 |
| **The same problem's next step**: the fix row a trace wrote (47), a replacement row (21), "once Qx lands" (12) | 80 | 0.37 |
| **Work handed on, or checking work**: a hypothesis the lane did not run (35), a claim or number the change made stale (27), code no test fails without (23) | 85 | 0.39 |
| **The machine**: the queue, lanes, recipes and fuzz tooling (39), measuring the tests and the nights (32) | 71 | 0.33 |

Asked whether doing the row would change what the kernel answers for some
input, the sort says yes for 87, no for 164, and "depends on what a trace
finds" for 80.

The first two groups alone come to 0.81 rows per close, under one. The last
two groups are what push it over.

Patterns the sort reported beside the labels. I have not checked these row
by row:

- **Rows repeat open rows.** Q331 repeated Q44 (both later folded into Q354).
  Q341 repeats Q101's `CoedgesRunTheSameWay`. Q418 asks Q118's question again.
  Q193 and Q194 both chase Q120's untested `NotHeld`. The only check at filing
  is "skip what is already a row", done by eye.
- **Too narrow a write set costs a replacement row.** Q202, Q338, Q345, Q350,
  Q354, Q368, Q410, Q412, Q417, Q426 and Q427 each replace a dropped row. Most
  of the 19 drops are these.
- **Chains that measure the fuzzing respawn and never reach a kernel answer.**
  The night slowdown: Q11 → Q46–Q48 → Q60–Q62 → Q135 → Q175 → Q243–Q245 →
  Q268, Q269 → Q296. Seeded runs that do not repeat: Q134 → Q136–Q138 → Q161 →
  Q236 → Q277–Q280. This is §311.2's "tier 1 refilled itself", seen from its
  rows.

### 348.5 Why nothing turns a row away

Every path a finding can take ends in the queue:

- The lane's report has a required field `remaining`, which
  `.claude/workflows/lanes.js` describes as "what is left, including what you
  learned about work you were told not to attempt".
- `merger.md` step 4 makes a row of every item of `remaining`, every item of
  `out_of_write_set_needs`, every verifier finding the merger did not fix, and
  every item of the "what this leaves" of a section the merge brings (§230.7).
  58 of the 107 section files from §230 on have a heading of that kind.
- The only filter is "skip what is already a row".
- The only way out is doing the row. 19 of the 216 closes were drops, and most
  of those were a row replaced by a wider copy of itself.
- Nothing ranks a finding by what it is worth. §13's "no named consumer, no
  build" governs features and exempts correctness gaps, and §16.10 says a gap
  is fixed, not deferred. In a certified kernel the refusals that have an
  answer run out only where the inputs do: every scale from 2^-1074 to 1e308,
  every placement, every family. So every one a lane sees becomes a row it
  owes.

And each row is cut small. A lane has about 45 minutes, a strict write set and
one deliverable. A file it needs but does not own is `out_of_write_set_needs`,
a new row. A doc or a pinned number its change makes stale is the next lane's
row. A hypothesis it had no time to run is a row, because `CLAUDE.md`'s
principle 5 forbids writing it down as a finding.

### 348.6 The root cause

**The queue records everything any lane saw, not the work we decided to do.**
A lane exploring this kernel sees more than it can fix. Every sighting is filed
as owed work, and nothing files one as seen but not owed. So each close leaves
1.5 to 1.8 rows, whatever kind of row it was and whichever tier comes first.

Three things follow:

- **More lanes make it worse.** Rows added are proportional to rows closed,
  so the open count grows by about 0.5 to 0.75 rows per close. A faster
  pipeline grows the queue faster.
- **About half the growth is real.** 95 of the 331 rows are kernel work
  nobody knew about before the lane looked. That is discovery, and a
  shorter queue that hid it would be worse.
- **The other half is the process's own.** Hypotheses handed on, stale claims
  and missing tests that a change caused, and follow-ups to measuring the
  machine: 156 rows, 0.72 per close. They are what lifts the rate from 0.81
  to 1.53.

### 348.7 What this leaves

The change this points at is to §230.7 and `merger.md` step 4. §311.4 set it
aside for Mattias to decide, so it is written here as a proposal and is not
adopted:

- **A leftover becomes a row only when it is new kernel work**: a wrong answer,
  a panic, or a refusal of an input that has an answer. **Or when it is the
  next step of the row's own problem**: a trace's fix, a replacement.
- **A stale claim or a missing test that the lane's own change causes is the
  lane's work**, inside its write set, and not a row for someone else. Its
  brief names the files that pin and document what it changes.
- **A hypothesis the lane did not run goes into its section as a hypothesis**,
  not into the queue. It becomes a row when someone chooses to run it.
- **A follow-up about the queue, the lanes or the nights is filed only when
  Mattias asks.** That is the one §311.4 named.
- **The queue page shows rows added per row closed**, over the last day, so
  the rate is a number anyone can watch rather than a feeling.

§226.1 is the reason §230.7 exists: two findings of 09-18 written only in a
section and a `/tmp` handoff were never picked up. Under the proposal nothing
is written only to `/tmp`. What is not a row is in a committed section, where
`just decisions-find` reaches it. What changes is that it is no longer owed.
