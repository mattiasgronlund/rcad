## 447. Three slots again, one kept for a Sonnet row — 2026-10-13

*Mattias, 2026-10-13. The session asked whether to let Sonnet rows from tier 2
on into the fill beside tier 1. He answered "Yes and Yes. Could we increase the
number of slots when we add sonnet rows? How many sonnet rows should we let
in?" The next session read the nine days `sar` keeps: at four slots the machine
passed §397.7's load bar on every one of them, so it first wrote this section
as three slots. Mattias then asked "Why is the bar so low, isn't there 24
hyper-threads on this CPU, what do you see/think is the consequence of pushing
to many simultanious slots?" 447.3 is the answer the session gave, and he
answered "Yes!". This amends §397.7, and is the record Q853 asked for. 447.6
then read 447.3's own bar once four slots had run with the kept Sonnet slot,
and both halves of it were passed; 447.7 is Q2892 acting on that reading.
447.8 reads the same bar again after a few days at three slots, and weighs
447.7's suite-growth confound against what moved: the suite grew hardly at
all over that window, while the bar it was supposed to bring back under got
worse, not better. The file keeps the name it was first written under,
because rows name it.*

In short: three slots again, now that 447.3's bar is passed at four (§447.7).
One of the three is kept for a Sonnet row while one is ready. There is no
fifth slot.

  447.1   Why the Sonnet rows were not reached
  447.2   The load at four slots
  447.3   What is decided
  447.4   The rows
  447.5   What Q1932 built
  447.6   447.3's bars, read again with the kept Sonnet slot
  447.7   Back to three, by 447.3's bar
  447.8   A few days at three slots: the bar again, and the suite-growth confound weighed

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

### 447.5 What Q1932 built

The `queue` recipe's `next`-mode awk in `justfile` and `elsewhere()` in
`tooling/queue/queue.py`, held by `check-queue-selftest` and
`check-queue-api-selftest`:

- `_queue-rows` now carries each row's own model into the fill (`model[r]`),
  which it parsed but used to drop; `queue.py`'s `elsewhere()` prints a 12th
  field so a row another dispatcher holds carries its model too, with a `"-"`
  placeholder kept in `the_hold_it_meets()`'s synthetic record so the field
  count still agrees (that path never reads it).
- `take_at()` carries the guard: before taking a row for what would be the
  last free slot, if the row is not pinned, is not itself Sonnet, no running
  lane is Sonnet (`running_has_sonnet()`), and no Sonnet row has been taken yet
  this fill, it looks for the first ready Sonnet row still available
  (`sonnet_slot_pos()`) and takes that instead. A row that would be *blocked*
  anyway still falls through to the normal skip-and-print path, so a row
  meeting a running lane is still named in the fill's output, not swallowed by
  the guard.
- `take_full_bundle()`, which takes a full bundle of small rows ahead of tier
  order, carries the same guard: a full bundle that is not itself all Sonnet
  does not take the literal last free slot while a ready Sonnet row sits
  unblocked; `bundle_is_pure_sonnet()` and the inline check inside
  `take_full_bundle()` test purity before committing either path.
- `running_has_sonnet()` groups this copy's own running rows and every other
  dispatcher's by the branch (or lane) they share, matching 447.3's "a bundle
  counts by its model" for a lane that is itself a bundle: a branch counts as
  Sonnet only when every row held under it is.
- A pinned row is exempt, per 447.3's "pins come first".

`check-queue-selftest`'s `full` fixture had its `Q3` changed from Opus to
Sonnet, so its bundle (`Q1,Q2,Q3`) stays pure Sonnet; left mixed, the new rule
reads it as not a Sonnet take and diverts the fixture's own point (a full
bundle going ahead of tier 1) into exercising the new rule by accident
instead. `sonnetkept`, `nosonnetkept`, `sonnetrunning`, `sonnetfullbundle` and
`sonnetfullbundlepure` are the fixtures that hold 447.3 itself: each was
checked against the rule taken out (`if (0 && ...)` in place of the guard),
confirmed to fail, and checked again with the guard restored.

The one-sentence summary is in `.claude/skills/dispatch-lanes/SKILL.md`'s
step 4.

### 447.6 447.3's bars, read again with the kept Sonnet slot

Q1932 (5b9241d75, 2026-10-13T23:11:05+02:00, 21:11:05 UTC) built the kept
Sonnet slot. `queue_slots` was 3 at that moment (Q1931 had set it 19:22:14
UTC the same day); it went back to 4 only at Q1946 (072686e8d,
2026-10-13T23:35:11+02:00, 21:35:11 UTC). So "four slots with the kept Sonnet
slot" has held continuously only since 21:35:11 UTC on 10-13, and both bars
below are read from that moment to now, 2026-10-27.

**The test-step bar.** Every fresh gate's main test step since 21:35:11 UTC
on 10-13 (a gate summary line `tests: Summary [ Ns] M tests run` with M over
3000 and no `served from`, found in the transcripts under
`~/.claude/projects/*rcad*/**/*.jsonl`, deduplicated on its exact duration and
count, read from each record's own `timestamp` field):

| | since 10-13 21:35 UTC (four slots, kept Sonnet slot) | 10-05 to 10-13 (447.3) |
|---|---|---|
| fresh main test steps | 737 | 1 021 |
| median | 530.0 s | 266 s |
| share past 600 s | 303 of 737, 41.1% | 2.8% |

Both halves of 447.3's bar are now passed: 41.1% against the 10% bar, and the
median has nearly doubled against the 266 s it was watched at. The script is
`/tmp/.../scratchpad/gate_steps.py` of this row's session, run as `python3
gate_steps.py`; the scratchpad is not kept, so the method is written out in
full above rather than cited by path.

**The catch-up bar.** `git log origin/main --merges --since=2026-10-13T21:35:11+00:00`,
over the same window, counted the way §397.5 counts a catch-up (a merge
branch merging `origin/main` again after losing the push race, which names
its branch `merge-q<id>-...`) against every lane merge (`Merge lane-...`
commits, bundles counted once each as §397.5's own count did):

| catch-ups | lane merges | share |
|---|---|---|
| 225 | 487 | 46.2% |

```
git log origin/main --merges --since=2026-10-13T21:35:11+00:00 --format='%H|%s' | grep -c '^[^|]*|Merge lane-'
git log origin/main --merges --since=2026-10-13T21:35:11+00:00 --format='%H|%s' | grep -c 'into merge-q[0-9]'
```

46.2% is well past the 35% bar, and nearly double §397.5's own era (17 of 79,
21.5%, at three slots before the lock in item 1-9 of §397.5 existed). Both of
447.3's "back to three" conditions are independently passed: the test-step
share and the catch-up share. Neither the median nor the catch-up share were
cross-checked against the busy-machine `sar` reading §447.2 used; whether the
rise is the fourth slot, the kept Sonnet slot, or the much higher dispatch
rate visible in the rows-closed figures below (487 lane merges in under 14
days, against 79 in under 5 days at 447.2's reading) is not decided here.

**Rows closed per day by model.** Every "`Qnnn[, Qnnn...] done`" merge commit
on `origin/main` (a `Merge lane-...` subject) names which rows it closed and
when, by its own commit date; `docs/queue.md` keeps a `model` field on every
row, done or not, so each closed id's model is read off the line that names
it (`^Qnnn\s+\S+\s+\S+\s+(opus|sonnet)\s`). Dates are the commit's own
`%cI`, converted to UTC, since the window's own boundary is a UTC instant:

```
git log origin/main --merges --since=2026-10-13T21:35:11Z --until=2026-10-27T23:59:59Z --format='%H|%cI|%s'
git log origin/main --merges --since=2026-10-05T00:00:00Z --until=2026-10-13T21:35:11Z --format='%H|%cI|%s'
```

kept to subjects starting `Merge lane-`, with a `Qnnn[, Qnnn...] done` list
pulled out of each (about 40 relay-only "into lane-..." merges in each window
name no row done and are not counted):

| | since 10-13 21:35 UTC (now) | 10-05 00:00 UTC to 10-13 21:35 UTC |
|---|---|---|
| lane merges | 487 | 491 |
| merges with a done list | 447 | 452 |
| rows closed | 537 (299 Opus, 238 Sonnet) | 558 (374 Opus, 184 Sonnet) |

The two windows are not the same length (13 days plus a sliver on each end
here, 8 days plus a sliver there), so the rate is read over full UTC days
only: 10-14 to 10-26 (13 days) against 10-05 to 10-12 (8 days); 10-13 and
10-27 are partial on both ends of their windows (10-13 by the 21:35:11 UTC
cutover itself, 10-27 because it is still running) and are left out.

| | since 10-13 21:35 UTC, 13 full days | 10-05 to 10-12, 8 full days | ratio |
|---|---|---|---|
| Opus rows closed per day | 22.0 | 43.25 | 0.51 |
| Sonnet rows closed per day | 17.9 | 21.6 | 0.83 |
| total rows closed per day | 39.9 | 64.9 | 0.62 |

Rows close more slowly now for both models, not faster: the Opus rate is
about half the baseline's, the Sonnet rate about five sixths of it, and the
combined rate about three fifths. This runs the other way from the two bars
above, which both rose. Nothing here says why: the same caveat as the test-step
and catch-up bars applies, and more sharply — the queue may hold fewer easy
rows than it did on 10-05 to 10-12 (nobody has counted them), lanes now spend
part of each run on the family checks §450.3 added, and some of the rows closed
since are themselves slower-running (fuzz triage, soaks) than the median row of
two weeks ago. Any of those would
lower the rate on their own, without the slot count or the kept Sonnet slot
costing anything; this reading cannot tell them apart, and does not try to.

### 447.7 Back to three, by 447.3's bar

447.6 read both of 447.3's "back to three" conditions at four slots with the
kept Sonnet slot, and both were passed, independently: 303 of 737 fresh main
test steps since 10-13 21:35 UTC ran past 600 s, 41.1% against the 10% bar;
and catch-ups were 225 of 487 lane merges over the same window, 46.2% against
the 35% bar. 447.3 said either alone sends the cap back. Q2892 (this row) acts
on it: `queue_slots` goes from 4 to 3, with the comment above it rewritten to
argue from this section, as Q1931 (72b8321d) first moved it to three and Q1946
(072686e8d) moved it back; the same two selftest lines move again
(`check-queue-selftest`), and the dispatch-lanes skill's cap sentence with
them. The note `queue next` prints past the cap, and the comment in
`check-queue-api-selftest` that named where the real cap is pinned, both cited
§397.7 by then-habit after 447.3 replaced its load bar; both now cite this
section instead, since this is where the cap's current value is argued.

**The Sonnet slot stays kept, so tier 1's Opus rows now get two slots, not
three.** 447.3's "one of the four is kept for a Sonnet row" and its reasons
(§446.2's figures on what a closed Sonnet row versus an Opus row leaves
behind) did not depend on the total being four; nothing in 447.6's reading
touches them either. Three slots, one kept, leaves two for Opus.

**One thing 447.6 did not weigh: the suite has grown.** Gate summary lines in
the transcripts put the main test step at about 3 600 tests on 2026-10-10 (3
531 to 3 709 across that day's runs), against 4 615 at `4c41703a` today, a 27%
rise. A bigger suite runs longer at the same load, which pushes the test-step
bar (600 s) up for a reason that has nothing to do with how many slots are
filled or how costly the load is per §447.3's argument. 447.6 did not cross-
check its 41.1% and 46.2% against this growth, and neither does this section;
a later reading of the bar at three slots should weigh how much of any change
in the share past 600 s is the suite growing rather than the slot count.

### 447.8 A few days at three slots: the bar again, and the suite-growth confound weighed

`queue_slots` went back to 3 at Q2892 (`44564b230`,
2026-10-27T11:04:39+01:00, 10:04:39 UTC). This reads 447.3's two bars again
over the days since, from that moment to now, 2026-11-01, and weighs 447.7's
suite-growth confound against them.

**The test-step bar.** Read the same way as 447.6 (a gate summary line
`tests: Summary [ Ns] M tests run` with M over 3000 and no `served from`,
found in the transcripts under `~/.claude/projects/*rcad*/**/*.jsonl`,
deduplicated on its exact duration and count, read from each record's own
`timestamp` field), since 2026-10-27T10:04:39Z:

| | since 10-27 10:04 UTC (three slots, kept Sonnet slot) | since 10-13 21:35 UTC (447.6, four slots) | 10-05 to 10-13 (447.3) |
|---|---|---|---|
| fresh main test steps | 158 | 737 | 1 021 |
| median | 820.2 s | 530.0 s | 266 s |
| share past 600 s | 158 of 158, 100% | 303 of 737, 41.1% | 2.8% |

Both of 447.3's own thresholds are passed by more than they were at 447.6's
reading, not less: every one of the 158 steps ran past 600 s, against the 10%
bar, and the median has risen again, by more than half, against the 530.0 s
447.6 watched it at. Every day in the window shows the same 100% share on its
own: 27 of 27 on 10-27, 40 of 40 on 10-28, 49 of 49 on 10-29, 33 of 33 on
10-30, 6 of 6 on 10-31, 3 of 3 on 11-01. The script is
`/tmp/.../scratchpad/gate_steps_447_8.py` of this row's session, run as
`python3 gate_steps_447_8.py`; as with 447.6, the scratchpad is not kept, so
the method is written out in full above rather than cited by path. Checked
against 447.6's own window (since 2026-10-13T21:35:11Z to 2026-10-27T10:04:39Z)
the same script finds 885 steps, median 520.9 s, 357 of 885 (40.3%) past
600 s — close enough to 447.6's own 737/530.0 s/41.1% to trust the method
reads the same thing 447.6 did, the gap being more transcripts now on disk
for that past window than 447.6 had when it ran.

**The catch-up bar.** Counted the way 447.6 counted it, over the same window:

```
git log origin/main --merges --since=2026-10-27T10:04:39Z --format='%H|%s' | grep -c '^[^|]*|Merge lane-'
git log origin/main --merges --since=2026-10-27T10:04:39Z --format='%H|%s' | grep -c 'into merge-q[0-9]'
```

| catch-ups | lane merges | share |
|---|---|---|
| 28 | 76 | 36.8% |

36.8% is still past the 35% bar, but only just, against 46.2% at 447.6's
reading. This is the one figure that moved the way cutting the slot count
predicts.

**Weighing 447.7's suite-growth confound.** The transcripts' own test counts
run from 4 615–4 618 at the start of this window (10-27, matching `4c41703a`'s
4 615 that 447.7 already named) to 4 682 now — a 1.45% rise, not the 27% rise
447.7 flagged between 10-10 and 10-27. That 27% rise sits inside 447.6's own
reading window (10-13 to 10-27), not between 447.6's reading and this one:
by the time 447.6's window closed, the suite was already at the size it is
now, within 1.45%. So the 27% growth 447.7 raised cannot be what pushed the
share past 600 s from 41.1% to 100% or the median from 530.0 s to 820.2 s
between the two readings — the suite most of that growth happened under had
already stopped growing by then. A further 1.45% growth, on its own, is far
too small to explain a 59-point rise in the share past 600 s or a 55% rise in
the median: nothing in 447.3's load-versus-duration table shows anything
close to that sensitivity for a change of that size, and 447.2 itself already
flagged that something other than the slot count was moving the load even
before the suite had grown this much.

**The slot count does not explain it either, and in the direction that
matters it runs the wrong way.** Cutting from four slots to three is supposed
to lower the load and so shorten the test step, per 447.3's own argument; the
median instead grew by more than half. The one bar that moved the way the
cut predicts is the catch-up share, and even that may be riding a falling
merge rate rather than the lower cap as such: 76 lane merges over about 5.3
days here is roughly 14 a day, against roughly 36 a day over 447.6's 13.7-day
window — a further fall from the decline 447.6 itself already measured in
rows closed per day. Fewer merges racing each other also means fewer
catch-ups, with or without a lower cap.

A day/night split, checked because a running fuzzer with `nice`d CPU use is
known to slow lanes at night: 27 of the 158 steps fell between 00:00 and
08:00 UTC, median 1 282.5 s, all past 600 s; the other 131, in daytime hours,
had median 794.5 s, also all past 600 s. Night makes it worse, but daytime
alone already sits far past both 447.6's 530.0 s median and the 10% bar, so
time of day is not the whole story either.

**So this section rules out the two things it was asked to weigh, and
nothing else.** Neither the suite's growth (1.45% over this window, its 27%
already spent inside 447.6's own reading) nor the slot cut itself (which
should have shortened the step, not lengthened it by more than half)
accounts for the rise from 447.6's reading to this one. What does is not
measured here.
