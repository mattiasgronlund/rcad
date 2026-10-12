## 418. A fuzz group no row names stops the fill until a row is filed to triage it — 2026-10-08

*Mattias, 2026-10-08: "systemd is collecting the information from the nightly
fuzz session, but there is never anything that converts the findings into rows.
And I would be good if something could recognise that there are findings from
the fuzz run that would require rows to fix, and then make sure that there is a
row created to analyze and prioritize fixes for the findings." Decided in the
`dispatch-red-merge-and-dirty-gate` session; built by the two rows of §418.5.*

In short: when the latest night has groups still failing on `main` that no row
names, and no triage row is open, `just queue next` refuses and names the
command that files one. That one row's lane reads the night's verdicts when it
starts and reports one row per fault, with its class. A row names a group by
one of its crash files, so `fuzz-triage` writes every file of each failing
group, not only a sample.

  418.1   What happens today
  418.2   The row is filed at the fill, not at 08:30
  418.3   One row triages, and it reads the night it starts on
  418.4   A row names a group by one of its crash files
  418.5   What is built, and in what order

### 418.1 What happens today

The night fuzzes from 00:10 to 08:10, and at 08:30 a timer replays what it
found against `main` and writes `fuzz-triage-verdicts.tsv` (§410). Q1217
(9d078c4e) made `just status` count the groups still failing that no open row
names. Nothing files a row. Every row about a fuzz group so far was written by a
session somebody asked: Q1204 to Q1216 on 10-06 at 21:14 (673780eb), and Q1364
on 10-08 at 09:30 (5d803c9e).

The timer itself failed both mornings it ran. `journalctl --user -u
rcad-fuzz-triage.service` shows 10-07 and 10-08 at 08:30 each ending in `mise:
command not found` and exit 127. 9cc229f0 fixed that at 09:01 on 10-08, and the
verdict files of 10-07 and 10-08 were written by hand at 09:27 and 09:28.
Nothing said so in between: a unit's failure lands in a journal nobody reads.

### 418.2 The row is filed at the fill, not at 08:30

Three places were weighed.

**The 08:30 unit files the row.** It runs in the night's own tree, on the
night's commit. To file a row it would need a checkout of `main`, a commit, and
a push to `origin` with nobody watching, every morning. When that failed, it
would fail where the two mornings of §418.1 failed: in the journal.

**A step in the `dispatch-lanes` skill.** Q1028 records five fills in three
days that did the opposite of what the skill, the handoff and a memory note all
said. Prose in a skill is not a mechanism.

**`just queue next` refuses.** This one. A row does nothing until a fill takes
it, so a row filed at the fill is as early as one filed at 08:30. Every fill
runs `queue next`, so the refusal is read by whoever can act on it, and the fix
is one command that takes seconds.

It does not refuse when the latest night has no verdicts file. It says so on a
line of its own and fills. A missed morning loses nothing, because the next
morning's replay carries it (§410.3), and holding a fill up for a
fifteen-minute replay costs more than it buys.

### 418.3 One row triages, and it reads the night it starts on

What gets filed is one row, not a row per group. Turning groups into rows is
judgment: is this a new fault, one an open row already describes under another
place, or one not worth fixing? On 10-06 a session turned 17 failing groups
into 13 rows; on 10-08 one found that 16 of 17 already had a row.

The row is tier 1 with no class, Opus, write set `-`. Its lane reads the latest
night's verdicts when it starts, not when the row was filed, so one open triage
row is enough: while one is `todo`, `after:` or `lane:`, `queue next` does not
ask for another. Groups that turn up while it runs are the next fill's to see.

The lane reports one row per fault, with its class (§352.3, §403), naming one of
the group's crash files. For an open row that already describes a group but
names none of its files, it reports that row filed again with one: a drop and an
add, carrying any row that waits on it.

Tier 1 with no class walks after every A and B row. That is §352's order: a row
whose likelihood nobody has judged waits behind the ones judged likely. The
faults the nights have found so far mostly came out D: Q1204, Q1205, Q1206,
Q1283, Q1313 and Q1364 are all in tier 5.

*§440.2, 2026-10-12: before the triage lane grades a fault C or D, it runs the
shape again with each out-of-range number made ordinary and every relation
kept. The same failure makes it at least C, and a row it files without that try
has no class.*

### 418.4 A row names a group by one of its crash files

Q1217 counts a group as named when an open row names its sample's file name, or
its panic's place. Neither holds still.

**The sample changes as the group grows.** `fuzz-triage` hands the decoder the
night's files in sorted order, and `triage` in `crates/rcad-fuzz-explain`
keeps the first file of each group it reads. Every night adds files. Of the 14
groups failing on both 10-07 and 10-08, 2 had a different sample the second
morning: a writer group that went from 4 files to 5, and the `mod.rs:571` group
that went from 16 to 21. A row naming 10-07's sample stopped naming its group on
10-08.

**A place is shared.** `writer.rs:226` holds four of 10-08's failing groups,
because it is the assertion every reader refusal in the writer fuzzer reaches.
Q1217 counts those apart, as needing a look.

So **a group is named when a row names any crash file in it.** Every night's
`artifacts/` carries every crash file of the nights before (§410.3), so a file a
row names stays in its group for as long as the group exists. For that,
`fuzz-triage` writes every file of each failing group, not one sample. Then:

- an open row naming one of its files names the group;
- a dropped row naming one names it too, so a fault decided not worth fixing is
  not raised again every morning;
- a done row names it only while its merge is not yet in the `main` the night
  was replayed on. A group still failing on a `main` that holds the fix was not
  fixed, and needs a row again. That needs the replayed `main`'s sha in the
  night's directory; today it goes only to the journal.

A group named only by its place counts as not named, for the refusal. So the
first triage after this lands files again the rows written before it, which
name places.

> **2026-10-12 note (Q1731): only files that pass leave (§434).** The morning
> now removes from the tree each crash file that passed on `main` (§434.3), so
> "every crash file of the nights before" above means every one not yet seen
> to pass. A file of a failing group does not pass, so it stays, and a file a
> row names stays in its group for as long as the group fails (§434.5). A file
> that has left is listed in `~/rcad-fuzz-logs/removed-crash-files.tsv` with
> the `main` it passed on, and prep reads that list as well as the members
> file.

### 418.5 What is built, and in what order

Two rows, Q1429 after Q1428.

1. **Q1428.** `fuzz-triage` writes each failing group's files and the `main` it replayed
   on. The count `status` prints matches by file, with the dropped and done
   rules of §418.4, and keeps the place-only count apart.
2. **Q1429.** `just fuzz-triage-row` files the triage row of §418.3 when that count is not
   zero and no triage row is open. `queue next` refuses as §418.2 says, naming
   that command, and the `dispatch-lanes` skill says what the refusal means.

Not built: recurring rows, and rows that wait for a date. Mattias asked about
both the same day. This needs neither, because the fill runs every time a slot
frees, and that is the clock it rides on.
