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

`just queue cores <id> <n>|all` sets, replaces or removes the one `cores:`
entry of an open row's write set (`cores_step`); it never conflicts with any
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
throwaway script, `target/check-night.sh`, and is not part of the committed
fixture.

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
- **The rows that should carry `cores:`** are not this section's to pick; the
  lane that wrote it reports them in its own `rows`, for the merger to apply
  with `just queue cores`.
