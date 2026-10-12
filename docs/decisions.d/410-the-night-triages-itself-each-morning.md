## 410. The night triages itself each morning — 2026-10-06

*Asked for by Mattias: "Shouldn't the nightly job end with `just fuzz-triage`?",
and then to triage every night that had not been. Two sessions, one after the
other; every figure below was read off the files named beside it.*

In short: `tooling/systemd/rcad-fuzz-triage.service` and `.timer` run `just
fuzz-triage` at 08:30 every day, from the night's own tree, with no arguments.
The night's own unit is not touched. `check-triage-waits-for-the-night` adds up
the night's hard stop out of the unit files and fails if the morning could start
before it, or still be running when the next night starts.

### 410.1 Nobody was running it

§186.2 rests on one sentence: "`fuzz-triage` replays every finding against
`main` the next morning". The recipe did that when asked. Nothing asked it. No
skill, no agent file and no CLAUDE.md names `fuzz-triage`; `grep -rl` over all
three, and over the plugins, finds nothing.

So the morning replay happened when somebody remembered it. Eighteen nights up
to 2026-10-05 left inputs to replay. Before 2026-10-06, one of them had a
`fuzz-triage-verdicts.tsv`: 2026-09-26, written at 13:09 that day. Every other
verdict file's modification time is 2026-10-06, from the two sessions that
asked for this.

### 410.2 A timer of its own, not the end of the night

Three ways to run it after the night were weighed.

**At the end of `fuzz-overnight`.** The night's `TimeoutStartSec=29400` kills
the whole run at 08:20, and the replay is not short: the backfill read
2026-10-04's 64 420 crash files in 13.5 minutes and 2026-10-05's 68 845 in 16.3,
going by when each night's verdict file was written after the one before it.
Inside the night, the replay would eat the fuzzing it reports on, or be killed
by the backstop meant for the fuzzer.

**`OnSuccess=` on `rcad-fuzz.service`.** That edits the tracked night unit, and
`check-systemd-units` runs in every gate on this machine, lanes' included. From
the push until Mattias copied the new file in, every gate here would fail on
"installed `rcad-fuzz.service` is not `tooling/systemd/rcad-fuzz.service`", for
a change no lane made. A new pair has no such window: it passes as "not
installed" until it is installed, then as "matches". And a service with no timer
beside it is what arm seven of `check-systemd-units-selftest` calls half a pair.

**A timer pair of its own.** This one.

### 410.3 Why 08:30, and why a missed morning is fine

The night's hard stop is a sum: 00:10, plus up to a minute of `AccuracySec`
(the timer fired at 00:10:34 on 2026-10-06), plus `TimeoutStartSec=29400`, plus
`TimeoutStopSec=120`. That is 08:23:00 at the latest. 08:30 is after it.

It has to be after it because of `latest`. `fuzz-triage` with no argument
replays the night `~/rcad-fuzz-logs/latest` names, and the night writes that
when it finishes. A replay that starts before then reads the night before, and
does so every morning: the verdicts would always be a day behind, and nothing
in them would say so.

`Persistent=false`. A catch-up run fires at boot, at whatever hour the machine
comes up, and that hour is bounded by nothing above. Losing a morning costs
nothing, because **every night's `artifacts/` holds every crash file of the
nights before it.** Checked: each crash file in the `artifacts/` of every night
from 2026-09-16 to 2026-10-05 is also in 2026-10-06's 69 879. The next morning
replays the missed night's inputs along with its own. The exceptions are the
three hand-rescued directories, `artifacts-rescued-2026-09-11`, `-13` and `-14`,
which are not carried and were replayed once by hand in the backfill (§410.5).

The pile is cumulative, so the replay grows with it: 22 633 crash files on
2026-09-26, 69 879 ten nights later. `TimeoutStartSec=14400` is four hours,
about fifteen times today's run, and ends at 12:33, far from the next night.

> **2026-10-12 note (Q1731): a crash file now leaves the pile once it has
> passed (§434).** After the replay, the morning folds each crash file that
> passed on `main`, and that the merge read to the end, into its target's
> corpus, and removes it from the tree's `fuzz/artifacts/` (§434.3). So the
> sentence in bold above now reads: every night's `artifacts/` holds every
> crash file of the nights before it that has not yet been seen to pass and
> been read by the merge. A missed morning still costs nothing, since a file
> leaves only after a morning has replayed it. The pile is no longer
> cumulative, and the replay shrinks with it. Each removed file has a line in
> `~/rcad-fuzz-logs/removed-crash-files.tsv`, and each night's directory keeps
> its own copy.

### 410.4 The check reads the files and refuses what it cannot read

`check-triage-waits-for-the-night` reads the two timers and two services in
`tooling/systemd/` and does the sum of §410.3 for each pair: the timer's hour,
`AccuracySec` (systemd's minute when unset), `RandomizedDelaySec`,
`TimeoutStartSec`, `TimeoutStopSec`. Both are counted from the night's start, so
a night across midnight needs no case of its own; the selftest has one.

It refuses, naming the line, an `OnCalendar=` that is not one time every day, a
span written any way but a number of seconds, minutes or hours (`infinity` and
`8h 10min` among them), a missing timeout, a key written twice, and
`Persistent=true` on either timer. Each is a case where reading on would mean
guessing, and a guess here is a pass that proves nothing.

Without this change the check fails, on `rcad-fuzz-triage.timer is not there`.
With the `AccuracySec` minute left out of the sum, four selftest arms fail,
among them 08:22, which is inside that minute and passes without it.

It reads the files itself rather than calling `systemd-analyze`, so it runs on a
runner. §81.5's reason for keeping `systemd-analyze --user verify` out of the
gate still holds, and both new files passed it before they were committed.

### 410.5 What the backfill found

Every night with inputs and no verdicts was replayed against `main`, one at a
time, and each now has its `fuzz-triage-verdicts.tsv`. The latest night is the
one that matters, because it carries all the others: 2026-10-06 replayed 69 879
crash files into 29 groups, **17 of them still failing on `main`**. Five are in
the predicate fuzzers, two in the fuse (`arrange.rs:2282`, `assemble.rs:1549`)
and ten in the STEP writer's round trip.

Of the rescued nights only 2026-09-11 left anything failing, and that one is
"bytes did not decode to a case": 247 inputs written in an older encoding, not a
fault in the kernel.

A verdict file is not a queue row. Triage files nothing, and nothing yet puts
the morning's count where a dispatcher looks; that gap is a row of its own,
beside one row per failing group.

*Q1217 put the count in `just status` on 2026-10-08. For the night `latest`
names, it prints how many groups still fail on `main`, how many no open row
names, and how many a row names only by a panic place that other failing groups
share too. That last kind needs a look to say which group the row is about,
and `_status-triage` says why it is counted apart. On 2026-10-08 the counts
were 17, 2 and 8. Filing the rows is still done by hand.*

### 410.6 Installing it

```
cp tooling/systemd/rcad-fuzz-triage.service tooling/systemd/rcad-fuzz-triage.timer ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now rcad-fuzz-triage.timer
```

Then `just check-systemd-units` says the installed pair matches. To see one run
now rather than at 08:30, `systemctl --user start rcad-fuzz-triage.service`
replays whichever night `latest` names, and `journalctl --user -u
rcad-fuzz-triage.service` has what it printed.
