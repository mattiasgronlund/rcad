## 447. Three slots, and one of them kept for a Sonnet row — 2026-10-13

*Mattias, 2026-10-13. The session asked whether to let Sonnet rows from tier 2
on into the fill beside tier 1. He answered "Yes and Yes. Could we increase the
number of slots when we add sonnet rows? How many sonnet rows should we let
in?" Its answer: no slot more for rows that build, because the load had passed
24 in 25 of 45 samples that day; one of the four slots kept for a Sonnet row; a
fifth slot for rows that write only docs; and read the load again after a day.
It had read one day. The next session read the nine days `sar` keeps, and the
answer changed: at four slots the machine passed §397.7's bar on every one of
them. This amends §397.7, and is the record Q853 asked for.*

In short: three slots, as §397.7 says once its bar is passed. One of the three
is kept for a Sonnet row while one is ready. There is no fifth slot.

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
bars once four slots had run for 100 busy daytime samples. It has been ready
since Q852 set the four slots on 10-04, and `git log -S'Q853 ' --
docs/queue.md` shows no commit since the one that added it. The bar it was to
read has been passed since the first day.

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

- **The first bar is passed**: 43% against 20%, over 716 samples where §397.7
  asks for 100. No day was under 23%.
- **The second bar, catch-ups, was not counted.** Either bar sends the cap back
  (§397.7), so it cannot change the answer.
- **The fuzzer does not explain it.** The mean `%nice` over a day's busy
  samples was 0.0 to 1.7%, and 4.0% on 10-08. At 18:01 UTC on 10-13 no niced
  process was using CPU (`ps -eo ni,pcpu`), and the one-minute load was 51
  with four lanes out.
- **Four slots load the machine more than six did.** The mean user cores went
  from 8.1 at six slots to 11.7 at four. So something other than the slot count
  moved too, and three slots may not bring the share back to 12%. What moved is
  not measured here; the measuring row of 447.4 reads it again.

### 447.3 What is decided

**Three slots.** §397.7 says back to three when either bar holds, and the first
one holds. Until the row that sets it lands, `queue next` still counts four, and
a dispatcher fills to three.

**One of the three is kept for a Sonnet row.** This is Mattias's yes.

- While a Sonnet row is ready and no running `lane:` row is a Sonnet row, the
  fill puts no Opus row in the last free slot. It puts the first Sonnet row in
  fill order there.
- Pins come first (§430.3). A pinned row takes any free slot, whatever its
  model.
- A bundle counts by its model, which is Sonnet only when all its rows are
  (§352.4).
- Tier 1's Opus rows get two slots, not four.

Why one, and not none: by the asking session's figures (§446.2), a closed small
Sonnet row leaves 0.05 new rows, and a closed Opus row about 1.5. The rows that
keep the machinery honest, Q853 among them, are Sonnet rows in tier 2.2, and
they are the ones that went unread. Why not two: that leaves tier 1 one slot,
and Mattias asked to let Sonnet rows in beside tier 1, not instead of it. How
many rows a Sonnet slot closes a day against an Opus slot was estimated, not
measured, and the measuring row of 447.4 counts it.

**No fifth slot for rows that write only docs.** There are two reasons:

- The machine is past its bar at four, so no slot is added.
- A write set of docs alone does not mean the row builds nothing. At
  `84ddac4a`, 20 open rows that are not parked write only under `docs/` or to
  `.md` files. By their own text, Q942 replays 30 fuzz draws, Q1427 runs a
  soak, Q733 runs a gate pinned to four cores, and Q550 measures what a sum
  costs. §446's sweep is docs-only too, and runs 23 shapes. The fill reads the
  write set, and the write set cannot tell these from a row that edits prose.

### 447.4 The rows

Filed with this section:

- **Q1931**, tier 2.2, Sonnet, small: `queue_slots` from 4 back to 3, with the
  comment above it, and every selftest line and skill sentence that pins four.
  Q852's write set, which moved it the other way.
- **Q1932**, tier 2.2, Sonnet, after Q1931: the kept Sonnet slot of 447.3 in
  the fill, held by `check-queue-selftest` and `check-queue-api-selftest`. It
  adds 447.5, saying what it built.
- **Q1933**, tier 2.2, Sonnet, after Q1932: once there are 100 busy daytime
  samples at three slots with the kept Sonnet slot, read both §397.7 bars again
  as 447.2 reads them, and count closes per day by the closed row's model. It
  adds 447.6.

Q853 is closed by this section.
