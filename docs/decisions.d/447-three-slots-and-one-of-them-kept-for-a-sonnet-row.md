## 447. Four slots, one kept for a Sonnet row, and a bar on what the load costs — 2026-10-13

*Mattias, 2026-10-13. The session asked whether to let Sonnet rows from tier 2
on into the fill beside tier 1. He answered "Yes and Yes. Could we increase the
number of slots when we add sonnet rows? How many sonnet rows should we let
in?" The next session read the nine days `sar` keeps: at four slots the machine
passed §397.7's load bar on every one of them, so it first wrote this section
as three slots. Mattias then asked "Why is the bar so low, isn't there 24
hyper-threads on this CPU, what do you see/think is the consequence of pushing
to many simultanious slots?" 447.3 is the answer the session gave, and he
answered "Yes!". This amends §397.7, and is the record Q853 asked for. The
file keeps the name it was first written under, because rows name it.*

In short: four slots stay, and one of them is kept for a Sonnet row while one
is ready. §397.7's load bar is replaced by a bar on what the load costs: how
often the gate's test step runs past the 600 s a call is given. There is no
fifth slot.

  447.1   Why the Sonnet rows were not reached
  447.2   The load at four slots
  447.3   What is decided
  447.4   The rows

### 447.1 Why the Sonnet rows were not reached

The fill takes tier 1 first (§230.3), and tier 1 is mostly Opus rows. At
`84ddac4a`, `just queue` listed these ready rows:

| tier | Opus | Sonnet |
|---|---|---|
| 1 | 36 | 6 |
| 2 | 46 | 39 |
| 2.2 | 7 | 54 |
| 2.5 | 2 | 8 |

So 101 ready Sonnet rows sat in tiers 2 to 2.5, behind 42 ready rows in tier 1.
Every lane that closed a tier-1 row filed about one and a half new ones (§446.2),
so tier 1 never ran dry and the fill never reached them.

Q853 is the case in point. It was filed on 2026-10-03 to measure §397.7's two
bars once four slots had run for 100 busy daytime samples. It was ready from
the day Q852 set the four slots, 10-04, and `git log -S'Q853 ' --
docs/queue.md` showed no commit after the one that added it. The bar it was to
read had been passed since the first day.

### 447.2 The load at four slots

Read as §397.7 reads it, from `/var/log/sysstat/saDD` with `sadf -d -- -q` and
`sadf -d -- -u` (the `-1` row, all CPUs). A sample counts when its time is
06:10 to 21:40 UTC and its `%user` is at least one core of the 24
(`%user * 24 / 100 >= 1`). `sar` keeps nine days, `sa05` to `sa13`, so 10-04,
the day the cap moved, is gone, and the three-slot days cannot be read again.
10-13 runs to 18:00 UTC.

| slots | busy daytime samples | user cores, mean | 90th percentile | 99th percentile | load ≥ 24 | load ≥ 48 |
|---|---|---|---|---|---|---|
| six, until 09-29 12:00 (§397.7) | 259 | 8.1 | 14.9 | 19.0 | 28% | 3% |
| three, since (§397.7) | 121 | 6.0 | 11.0 | 15.2 | 12% | 1% |
| four, 10-05 to 10-13 | 716 | 11.7 | 17.2 | 19.8 | 43% | 11% |

By day:

| day | busy samples | load ≥ 24 | load ≥ 48 |
|---|---|---|---|
| 10-05 | 85 | 29 (34%) | 6 |
| 10-06 | 83 | 43 (52%) | 10 |
| 10-07 | 65 | 33 (51%) | 10 |
| 10-08 | 81 | 38 (47%) | 7 |
| 10-09 | 86 | 20 (23%) | 5 |
| 10-10 | 75 | 38 (51%) | 9 |
| 10-11 | 88 | 38 (43%) | 8 |
| 10-12 | 83 | 23 (28%) | 5 |
| 10-13 | 70 | 44 (63%) | 17 |

The percentiles are the sample at rank `p * (n - 1)` of the sorted user cores.

- **§397.7's first bar is passed**: 43% against 20%, over 716 samples where it
  asks for 100. No day was under 23%.
- **The second bar, catch-ups, was not counted.**
- **The fuzzer does not explain it.** The mean `%nice` over a day's busy
  samples was 0.0 to 1.7%, and 4.0% on 10-08. At 18:01 UTC on 10-13 no niced
  process was using CPU (`ps -eo ni,pcpu`), and the one-minute load was 51
  with four lanes out.
- **Today alone says the same.** Mattias asked whether the changes of 10-12
  had brought the load down, so that only 10-13 should count. Each busy sample
  was put beside the `lanes` rows in flight over its ten minutes, read from
  every run record under `~/.claude/projects/*/*/workflows/` (a row is in
  flight from its first agent's start to its last agent's end). On 10-13, to
  19:50 UTC, rows in flight never passed 4.0, so the cap was kept. With 3.5 to
  4 rows in flight the load was 24 or more in 39 of 55 samples (71%), and with
  fewer in 9 of 25 (36%). The same split on 10-12 was 16 of 58 (28%) and 7 of
  25 (28%); on 10-05 to 10-11 the share at 3.5 to 4 rows ran from 33% to 68%.
- **Four slots load the machine more than six did.** The mean user cores went
  from 8.1 at six slots to 11.7 at four. So something other than the slot count
  moved too. What moved is not measured here.

### 447.3 What is decided

**A load of 24 means the machine is full, not that it is overloaded.** The CPU
is an AMD Ryzen 9 3900X (`lscpu`): 24 hardware threads on 12 cores. At a load
of 24 every thread has a task, and two threads share each core. Past 24, tasks
queue for a thread. Nothing else runs short: on 10-13 `%iowait` averaged 0.19%
of the daytime samples, 5 GB of 125 GB of memory was in use, and
`/proc/pressure/memory` and `/proc/pressure/io` read 0. §397.6 found the same:
the load is the run queue.

**Past full, the work runs slower and does not fail.** There is no nextest
configuration, so no test is stopped for running long. Every fresh gate's main
test step since 10-05 (a gate summary line `tests: Summary [ Ns] M tests run`
with M over 3000 and no `served from`, found in the transcripts under
`~/.claude/projects`, deduplicated on its duration and count), against the mean
one-minute load of the `sar` samples over its run, widened by ten minutes each
side:

| load while it ran | runs | median | 90th percentile |
|---|---|---|---|
| under 12 | 130 | 218 s | 305 s |
| 12 to 24 | 326 | 252 s | 364 s |
| 24 to 36 | 304 | 283 s | 405 s |
| 36 to 48 | 169 | 303 s | 502 s |
| 48 and up | 92 | 365 s | 667 s |
| all, at four slots | 1 021 | 266 s | 426 s |

Four times the load makes the median about 1.7 times slower. 29 of the 1 021
steps, 2.8%, ran past 600 s.

**What too many slots costs**, in the order it is expected to bite:

- **Each lane's builds and tests run slower.** On Q1818 (`just row-time
  Q1818`), about two thirds of the agents' time went to `just land`, `just gate
  quiet`, `just verify-lane` and `just t`. A row takes longer, and holds its
  files longer, which keeps other rows out of the fill.
- **A gate outgrows the 600 s a call is given.** At load 48 and up, one test
  step in ten ran past it, and a gate is longer than its test step. The agent
  then spends calls following the gate, and each call bills its whole context
  again.
- **More push races.** A longer gate gives `main` longer to move, so more lands
  catch up and gate again, which adds load. §397.7's catch-up bar is there for
  this.
- **More dispatcher context per hour**, since every lane costs the dispatcher a
  brief, a report and a merge (§230.8).

**Whether a fourth slot still pays is estimated, not measured.** From load 12
to 24 up to 36 to 48, the median test step slows about 20%. If two thirds of a
lane's time is that kind of work, each lane slows about 13%, and four lanes
then close about 18% more rows than three. The fourth slot would stop paying
only if it slowed that work by about half, and nothing in the table comes near.

**So four slots stay, and the load bar is replaced.** §397.7's bar counted how
often the machine was full, and being full is not the harm. The new bar counts
what the load costs:

- **Back to three** if either holds over at least 100 fresh main test steps at
  four slots:
  - more than 10% of them run past 600 s (2.8% from 10-05 to 10-13);
  - catch-ups pass 35% of lane merges over the same days, counted as §397.5
    counts them (§397.7's second bar, kept).
- **The median is watched, not a bar**: 266 s from 10-05 to 10-13. A rise with
  the share past 600 s still low says the load grew without yet costing calls.
- **Both thresholds are judgement**, as §397.7's were. 10% is between today's
  2.8% and the 90th percentile reaching 600 s, which happens at load 48 and up.
- **A fifth slot needs a measurement of rows closed per slot**, which nobody
  has taken. Until then four is the cap.

**One of the four is kept for a Sonnet row.** This is Mattias's yes.

- While a Sonnet row is ready and no running `lane:` row is a Sonnet row, the
  fill puts no Opus row in the last free slot. It puts the first Sonnet row in
  fill order there.
- Pins come first (§430.3). A pinned row takes any free slot, whatever its
  model.
- A bundle counts by its model, which is Sonnet only when all its rows are
  (§352.4).
- Tier 1's Opus rows get three slots, not four.

Why one, and not none: by the asking session's figures (§446.2), a closed small
Sonnet row leaves 0.05 new rows, and a closed Opus row about 1.5. The rows that
keep the machinery honest, Q853 among them, are Sonnet rows in tier 2.2, and
they are the ones that went unread. Why not two: Mattias asked to let Sonnet
rows in beside tier 1, not instead of it. How many rows a Sonnet slot closes a
day against an Opus slot was estimated, not measured; Q1947 counts it.

**No fifth slot for rows that write only docs.** Two reasons:

- No measurement says a fifth slot pays.
- A write set of docs alone does not mean the row builds nothing. At
  `84ddac4a`, 20 open rows that are not parked write only under `docs/` or to
  `.md` files. By their own text, Q942 replays 30 fuzz draws, Q1427 runs a
  soak, Q733 runs a gate pinned to four cores, and Q550 measures what a sum
  costs. §446's sweep is docs-only too, and ran 282 fuses. The fill reads the
  write set, and the write set cannot tell these from a row that edits prose.

### 447.4 The rows

- **Q1931** set `queue_slots` from 4 to 3 under the first reading of this
  section, and landed as 53298595 before Mattias's "Yes!".
- **Q1946**, pinned at his word, sets it back to 4, with the selftest lines and
  the skill sentence Q1931 moved.
- **Q1932** builds the kept Sonnet slot of 447.3 in the fill, held by
  `check-queue-selftest` and `check-queue-api-selftest`, and adds 447.5,
  saying what it built.
- **Q1947**, after Q1932, reads 447.3's bar and the closes per day by model
  two days after Q1932 lands, and adds 447.6. It replaces Q1933, which read
  §397.7's load bar and was dropped with this amendment.

Q853 is closed by this section.
