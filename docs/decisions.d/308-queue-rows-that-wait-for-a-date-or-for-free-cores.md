## 308. Queue rows that wait for a date, or for free cores — 2026-10-19

*Queue row Q281. Lane `lane-q281-c-pin-after-a-date-and-cores-and-write-308`,
off `23e20496`, carrying on `lane-q281-b-after-a-date-and-cores-in-the-queue`'s
work. Q281's row: `just queue next` is offered a row the dispatcher cannot
take yet, and has no way to say so in the file — Q174, which needs a week of
nights and cannot run before 2026-10-03, was offered first and passed over ten
times across four sessions on 09-26 and 09-27. This section gives the file the
two words it lacked: a date `after:` can wait on, and a `cores:` entry a row's
write set can hold.*

### 308.1 `after:` takes a date as well as a row id

§230.2's `state` field already read `after:<id>[,<id>]`. It now reads
`after:<id|date>[,<id|date>]`, a date written `YYYY-MM-DD`
(`tooling/queue/rows.py`'s `DATE`). `just queue after <id> <id|date>[,...]`
accepts one of each kind mixed in the same list, in `queue.py`'s
`after_step`, and refuses a second date in the same `after:`
(`T9 cannot wait on two dates`) — one row waits on one calendar date, not a
range. `_queue-check` reads the same grammar
(`^after:([TDQ][0-9]+|[0-9]{4}-[0-9]{2}-[0-9]{2})(,(...))*$`) and refuses two
dates in one `after:` the same way.

A row in `after:` is **ready** once every id it names is `done:` and the UTC
date, if it names one, has come — "come" meaning the fill's own notion of
today is on or past that date, never before it. `just queue next`'s fill
(`_queue-rows`'s awk) marks it `waiting` instead when a named id is not yet
`done:` or the date has not yet come, comparing the date as text against
`now_date`, both `YYYY-MM-DD`, which sorts correctly as long as both are real
calendar dates. `just queue` without `next` prints a waiting row's line as
`waits on 2026-10-03 (not yet come)` or `(come)` beside each id it also waits
on, read the same way `render.py` reads it for the queue page.

A date needs no row to exist; it is Mattias's own estimate of when a row
becomes worth trying again, not a claim that any row finishes by then. Nothing
makes a dated `after:` come true on its own — the row still needs a `-` to
free it, or whoever wrote the date to take it, if waiting on the date was not
enough.

### 308.2 `cores:<n>` or `cores:all` pins hardware threads, not a path

A write-set entry `cores:<n>` (a positive integer) or `cores:all` says the row
pins that many of the machine's hardware threads while it runs — a fuzz
target under `-fork=`, or anything else that keeps cores busy for a measured
time. `CORES_ENTRY` in `queue.py` is `^cores:([1-9][0-9]*|all)$`, so
`cores:0` is refused (a row that pins no threads carries no `cores:` entry at
all, the same as a write set that names no file); `_queue-check`'s `entry()`
accepts the same pattern before it falls through to `path#item`.

`just queue cores <id> <n>|all|-` sets, replaces or (with a dash for the
count) removes the one `cores:` entry of an open row's write set (`cores_step`); it never conflicts with any
other entry, and a row keeps at most one.

A `cores:` entry **never meets a path**, a directory or another `cores:`
entry — `entries_meet` in the justfile's fill returns false the moment either
side starts `cores:`, and `entry_holds` in `queue.py` does the same for the
narrow/claims side. Two rows each holding `cores:12` on a 24-thread machine
are not held back from running together by `sets_meet`; only the thread sum
below can skip one of them.

### 308.3 The fill sums `cores:` against the machine's free threads

`_queue-rows`'s fill reads a thread count, `queue_threads`, empty outside a
selftest meaning the real `nproc`. `cores_fit_skip(i, say)` is checked first
in `blocked`, ahead of the ordinary path-meets walk:

- it sums `cores:<n>` (or `all` as the whole thread count) over every row
  `kind[j] == "running"` here, every row held `elsewhere`, and every row
  `taken` earlier in the same fill, naming each one as it goes;
- if the clock (§308.4) says this moment sits in the night fuzz's window, the
  sum starts already holding every thread, named `the night fuzz`;
- a `cores:all` row fits only when that sum is exactly zero; a `cores:<n>` row
  fits when the sum plus `<n>` is at most `threads`.

A row that does not fit is skipped with a line naming every holder:

```
skip Q92 (tier 1): its cores:2 does not fit the 24 threads of the machine: Q90 (held by lane:lane-q90) holds them
```

— the same shape as the path-meets skip line, so a dispatcher reading `queue
blocked` sees one kind of reason, not two. A row with no `cores:` entry holds
no threads and is never named in another row's skip line; it also never skips
on account of one, since `cores_fit_skip` returns `0` the moment `cores_of`
finds no entry.

### 308.4 The night window is derived from the unit files, not typed in

`cores_fit_skip` needs to know whether *now* sits inside the night fuzz's own
window, because that run holds every thread without a `cores:` entry of its
own — it is a `systemd` timer, not a queue row. The fill derives the window
from the two files that already say what the night does, rather than writing
a time of day into the justfile a second time (the way a typed-in window
could drift from the unit that actually runs):

- `tooling/systemd/rcad-fuzz.timer`'s `OnCalendar=*-*-* 00:10:00` carries no
  time zone, so `systemd` reads it in the host's own zone, Europe/Stockholm;
- `fuzz-overnight window="28800"` is the recipe's own default run length, eight
  hours, which is also what the timer unit's `[Unit] Description` names as
  "00:10 to 08:10".

So the window other than its length is "00:10 to 08:10, Europe/Stockholm,
every day", and the fill computes today's and yesterday's version of that
window in UTC and checks whether `queue_now` (empty outside a selftest, meaning
`date -u`) sits inside either one — yesterday's in case the window crosses
midnight UTC while it is still running. `TZ=Europe/Stockholm date` does the
zone conversion, so the window moves with Stockholm's own clock: during CEST
(summer time, UTC+2) it is 22:10–06:10Z, and once Stockholm leaves summer time
— 2026-10-25 this year — it is 23:10–07:10Z, an hour later in UTC for the same
local 00:10–08:10. `check-queue-selftest`'s `corenight` fixture drives three
timestamps through this with `queue_now` set directly (`driveclock()`):
2026-10-19T02:00:00Z and 2026-10-19T12:00:00Z (summer time, in and out of the
window), and 2026-10-26T06:30:00Z, after the 10-25 changeover — outside the
old, fixed summer-time window (which would have ended at 06:10Z) but inside
the shifted one (23:10–07:10Z), so it only passes when the code reads the
zone rather than keeping summer time's offset. A fourth point, 07:30:00Z the
same day (outside even the shifted window), was checked by hand in a
throwaway script, `target/check-night.sh`, and was not part of the committed
fixture until §308.8 pinned it for a night row.

### 308.5 Why this is not §254's and §296's 21:50–06:00Z

Several measurement sections from late September — §239, §254, §268, §272,
§275, §296 — say their own fuzz runs stayed outside "21:50–06:00Z". That is
not a second, looser derivation of the same window; it is each lane's own
round-number safety margin, picked by eye at the time so a short foreground
run would not overlap the night. §239's is the clearest instance: "no binary
of this lane ran after 21:50Z" is that lane's own deadline for finishing its
own replay, not a reading of the timer files. None of those sections runs
`TZ=Europe/Stockholm date` or reads `rcad-fuzz.timer`; a fixed UTC pair chosen
under CEST stays fixed once CEST ends, so by 2026-10-25 it would sit an hour
off the night fuzz's own window, the same way `queue_now` would be wrong an
hour a day if this fill used a constant instead of deriving it per run. The
two are measuring different things — a human margin against being caught
running something foreground during a night that happened to exist, against a
machine-checked "do not hand a row the threads the night is already holding"
— and this section's window is the one the fill acts on.

### 308.6 The tests

`check-queue-selftest`'s `_queue-check` fixtures, built on `good.md` with its
existing `refused()`/`passes()` helpers: `mixed-come` and `mixed-wait` give a
row's `after:` a done id and a date, already come and not yet come, and both
are read as `_queue-check` says the row count should (ready in the first
case, waiting in the second); `two-dates` refuses a second date in the same
`after:`; `cores-zero` refuses a `cores:0` entry the same way as a bad path;
`cores-all` accepts `cores:all` in a write set.

The fill gets its own fixtures, driven with `drive()`'s new sibling
`driveclock()`, which also `--set`s `queue_threads` and `queue_now`:

- `afterdate`: a date not yet come (`queue next` leaves it waiting, and the
  waiting line says `(not yet come)`) and one that has come (`queue next`
  takes it);
- `cores`: a `lane:` row holding `cores:3` beside two `todo` rows needing
  `cores:1` and `cores:2` on a four-thread machine — the `cores:2` row is
  skipped naming the holder, the `cores:1` row is taken;
- `coresall` and `coresallalone`: a `cores:all` row beside a row holding some
  threads (skipped) and beside none (taken);
- `corenight`: the three `queue_now` timestamps of §308.4 above, each driven
  through `driveclock()` against a `cores:` row, confirming the night fuzz
  holds every thread in its own window and none outside it, and that the
  window itself moves across the 2026-10-25 change.

Checked against the base this row started from, `23e20496`, in a `git
archive` scratch copy rather than by stashing or checking the base out over
the lane's own branch, the first of these, `mixed-come`, fails outright: the
base's grammar refuses a dated `after:` (`state "after:T9,2020-01-01" is not
todo, lane:<branch>, after:<id>[,<id>], done:<sha>, dropped:§N or
parked:§N`), which stops the scratch run's own `fail()` before the later
cases run. The base justfile and `queue.py` plainly have no `cores:` entry,
no `queue_now` and no `queue_threads` either, so the rest of the new cases
would fail the same way, but only `mixed-come` was observed failing there.

### 308.7 What this leaves

- **The queue page** does not read `cores:` or a dated `after:` specially; a
  waiting row's page entry is unchanged by this section. Nobody has asked for
  one yet (§13).
- **The rows that should carry `cores:`** were not this section's to pick; the
  lane that wrote it reported them in its own `rows`, and its merge applied
  them with `just queue cores`: `cores:all` on Q60, Q62, Q245, Q269, Q296 and
  Q733, `cores:2` on Q187, `cores:8` on Q243, `cores:1` on Q2390. Q12 has
  none, since it must run beside the night fuzz to see its load. Q174, the
  row a date was first meant for, was done before the merge.

### 308.8 A row that runs only by night

*Queue row Q3150, 2026-11-04. Lane `lane-q3150-a-row-that-runs-only-by-night`
off `21aa4650`, in three relays. Q12 must run beside the night fuzz to see its
load, which is why §308.7 left it without a `cores:` entry, but nothing in the
file said so: a daytime fill offered it first for its Sonnet slot, and the
dispatcher passed it over by hand (`8f35c06a`).*

**The entry and its verb.** A write set may hold the word `night`
(`NIGHT_ENTRY` in `queue.py`). It says when the row runs, not what it writes.
`just queue night <id> on|off` sets it on an open row or takes it off
(`night_step`, built the way `cores_step` is). `on` refuses a row that has it
already, `off` a row that has none, and both refuse a closed row. Taking it off
a write set that holds nothing else leaves `-`. `add` and `writes` accept it
through `parsed_write_set`, and `_queue-check`'s `entry()` accepts it before
it falls through to `path#item`, so it is never looked for among the tracked
files.

**It never meets a path.** Like `cores:`, `night` matches only itself.
`entry_holds` in `queue.py` compares it as exact text, so `narrow` keeps it,
and `entries_meet` in the fill returns false when either side is `night`, so
two night rows never hold each other back on that word.

**It is refused beside `cores:`.** The night fuzz holds every thread in its
window (§308.3), so a row holding both could never fit and would never be
offered. `refuse_night_beside_cores` refuses the pair in `cores`, `night`,
`add` and `writes`, and `_queue-check` refuses it in the file in the same
words:

```
T1: night and cores:2 cannot share a write set: a night row runs beside the night fuzz, which holds every thread then (§308.8)
```

**The fill passes a night row over by day.** `night_skip(i, say)` is checked
first in `blocked`, ahead of `cores_fit_skip`. While `night` (§308.4) says the
clock sits outside the window, a row whose write set holds `night` is skipped
with a line that names the next window in UTC:

```
skip Q12 (tier 2.5): its night entry fits only inside the night fuzz's window, 23:10Z to 07:10Z
```

The next window starts at today's local 00:10 when the clock has not reached
it yet, and at tomorrow's otherwise. Tomorrow's local date is read from today's
local noon plus 86400 seconds, not from now plus 86400 seconds: the day of the
spring change is 23 hours long, so at 23:30 local on 2027-03-27 now plus a day
lands at 00:30 on 03-29, and the line would name the window after the one that
comes next. The window ends at its start plus `fuzz-overnight`'s 28800
seconds, as §308.4 has it, since that is how long the timer's run lasts.

**`meets` is exempt.** `take`, `narrow` and `writes` ask the fill's `meets`
mode whether a write set meets another hold (`the_hold_it_meets` in
`queue.py`). That question is about paths, and the clock must not refuse it: a
lane that took a night row at 05:00Z narrows its hold after the window has
closed. So `night_skip` returns 0 in `meets` mode.

**The queue page reads the line.** `render.py`'s `NIGHT_LINE` and
`parse_blocked_line` read it as `shared` `"night"`, with the window and no
holders, and `meeting_reason` words it as "waits for the night: `night` runs
only inside the night fuzz's window, ...". Without them `parse_blocked_line`'s
assert would fire on every daytime render with a night row open.

**The tests.**

- `check-queue-selftest`: `passes night` and `refused night-cores` in the
  shape check. The `corenight` fixture gains Q2, a `todo` night row, and Q3, a
  running night row whose entry Q2 must not meet, and drives them at eight
  times. 2026-10-19T02:00Z is night, so Q2 is taken, and 12:00Z is day, so it
  is skipped naming 22:10Z to 06:10Z. 2026-10-26T06:30Z sits inside only the
  window the 10-25 change moved, so Q2 is taken, and 07:30Z is just after it,
  so Q2 is skipped naming 23:10Z to 07:10Z: the point §308.4 checked only by
  hand. 2026-10-25T22:30Z is day after the change, though summer time's offset
  would put it in the window, and 23:20Z is night. 2026-10-24T22:05Z, local
  00:05 on the change day, names the window that starts five minutes later,
  22:10Z to 06:10Z, and not the one after it. 2027-03-27T22:30Z, the evening
  before the spring change, names that night's window, 23:10Z to 07:10Z, which
  now plus a day would step past. Then `queue meets` by day: a write set of
  `night` alone against a hold on `night,docs/a.md` meets nothing, and
  `night,docs/a.md` meets it on `docs/a.md` alone.
- `check-queue-api-selftest`'s Q3150 block, in a worktree of its own: `night
  on` and `off`, with the write set each reports and writes; each refusal
  above (a second `on`, a second `off`, a switch that is neither, a closed row,
  `cores` beside `night` and `night` beside `cores:`, in `add` and in
  `writes`); and `queue blocked` by day and by night on the copy it wrote.
- `check-queue-page-selftest`'s `night_check`: the line read by
  `parse_blocked_line` and worded by `meeting_reason`. Run against the base's
  `render.py`, in a `git archive` copy, it fails on the assert:
  `` `just queue blocked` printed a line this cannot read ``.

**What this leaves.**

- Q12 is the row this was built for. A lane runs no queue verb on the queue
  (Q983), so the dispatcher runs `just queue night Q12 on` once this is on
  main.
- `cores` cannot take a `cores:` entry off, though §308.2, `cores_step`'s
  docstring and `queue.py`'s usage say it removes one: it refuses everything
  but a count or `all`, so only `writes` can. Filed as a row of its own.
- `narrowed` in `queue.py` hands the fill a todo row's latest `writes` from
  the claims file in place of the file's column whenever the two differ. So a
  `night on` or a `cores` run on a todo row after a `writes` would, by reading
  the code, be hidden from the fill. Nobody has run it; filed as a row of its
  own. Q12 has no `writes`, `narrow` or take event in the claims file.
