## 403. C and D defects go to a tier 5 below everything — 2026-10-04

*Mattias, 2026-10-04, asked what had happened to "judge how likely a failure is
to happen in reality, and fix the likely ones first". That is §352, his words of
09-28, as §360.5 refined it, and it works. But it ranks rows only inside tier
1, and §230.1's order is strict, so a D row — a ball 2^-528 across, a tower
1e308 tall — sat ahead of every row of tiers 2, 2.2, 2.5 and 3. Tier 3 is the
STEP corpus files that fail, which are "A, seen" by §352.3's own definition. His
decision: "Yes, we should create a new tier, tier 5. Any class C or class D
failure should be moved to tier 5." This section is how that was built. It
amends §230.1, §352.2, §352.3 and §352.6.*

### 403.1 The tiers

1, 2, 2.2, 2.5, 3, 4, 5. Tier 5 is the lowest, below tier 4, and the order
stays strict, as §230.1 has it: a tier 5 row gets a slot only when no row above
it can use one. Tier 5 holds every open defect of class C or D.

### 403.2 The class decides an open row's tier

A row of class A or B sits in tier 1, and a row of class C or D sits in tier 5.

- **A class may sit only on a tier 1 or a tier 5 row.** §352.2 said tier 1
  only.
- **Every open tier 5 row has a class.** The class is what puts a row in tier
  5, so a row there without one has no reason to be there. Parked rows count as
  open.
- **A closed row keeps the tier it closed in**, whatever its class. 38 closed
  tier 1 rows carry C or D. Every verb refuses a closed row, and moving them
  would rewrite history for nothing.
- **A row with no class stays in tier 1.** That includes a guess nobody has run
  (§360.5). Mattias said C and D, so only those move; see 403.10.

`check-queue` holds all four. When it refuses a class in the wrong tier, it
names the verb that fixes it: `just queue tier <id> <tier>`.

*§435 amended this on 2026-10-11, at Mattias's word. A row with no class that
waits on a tier 5 row goes down to tier 5 with it, so an open tier 5 row has a
class, or its `after:` names a tier 5 row. And a row with no class stays in
tier 1 only while it waits on nothing outside tier 1. Q1736 builds the check.*

### 403.3 The fill walks tiers 1 and 5 in class order

§352.3's list of classes is unchanged, and the fill now walks it in tier 5 as
well as tier 1. Each tier holds only its own classes, so tier 1 goes

`wrong/A, wrong/B, crash/A, crash/B, refuses/A, refuses/B, <no class>`

and tier 5 goes

`wrong/C, crash/C, refuses/C, wrong/D, crash/D, refuses/D`.

Both of §352.3's rules survive: how bad comes first inside A to C, and D comes
last whatever its symptom. What changes is that C now waits below tiers 2 to 4.
§352.3 had C above the rows with no class.

*§435.5 adds a place to tier 5's walk on 2026-10-11: `<no class>` between
`refuses/C` and `wrong/D`, for the rows that wait on a tier 5 row. The fill
already sorted a row with no class there, so only the walk as written changed.*

### 403.4 `tag` moves the row to the tier its class names

Setting a class on a tier 1 or tier 5 row moves the row to the foot of the tier
the class names. If the file has no heading for that tier, `tag` writes one. The
message says the row moved: `Q9: tag wrong/A -> refuses/C, tier 1 -> 5, at the
foot of ## Tier 5`. A class on any other tier is still refused: "a class ranks
tier 1 and tier 5 rows only".

Moving was chosen over refusing because of the lanes already running. `just
land` applies a lane's `add` and `tag` steps with **main's** `queue.py` at land
time (`tooling/land/land.py`, `apply_queue`). A lane that started before this
change carries the old `lane.md`, so it writes `add 1 …` and then `tag @1
refuses/C`. If `tag` refused, each such lane would stop at land time and fall to
an Opus merger. Because `tag` moves the row, those lanes land cleanly and their
C and D rows end up in tier 5.

Removing a class with `tag <id> -` moves nothing. `check-queue` refuses a tier
5 row with no class, so to remove a class, put the `tier` step that takes the
row elsewhere in the same `apply`.

### 403.5 `tier` and `add`

- **`tier` on a row with a class** goes only to the tier the class names. Any
  other target is refused, and the refusal still names `tag <id> -` as the way
  out. The migration (403.9) is this verb: `tier Qn 5` on a C row in tier 1.
- **`add` gets no rule of its own.** `check-queue` runs at the end of the batch,
  so it refuses `add 1 … refuses/C …`, or `add 5` with no class, and the refusal
  carries its reason. Since `apply` checks only at the end, `add 5 …` followed
  by `tag @1 refuses/C` is fine, and that is what `lane.md` now tells a lane to
  write.

### 403.6 The measure line

§352.6's line becomes

```
tier 1, open and not parked: N in A or B (w wrong, c crash, r refuses), N with no class; tier 5: N in C, N in D; N small rows ready, and a bundle is 5
```

C and D are counted by class wherever the row sits, so a copy that has not been
migrated still counts them right. "No class" is counted in tier 1 only.

Before the migration, the old recipe (the justfile at `b4e95b52`) read this on
the queue at `35eb8e30`:

```
tier 1, open and not parked: 33 in A or B (1 wrong, 0 crash, 32 refuses), 54 in C, 10 in D, 44 with no class; 2 small rows ready, and a bundle is 5
```

After it, the new recipe read the same counts in the new form:

```
tier 1, open and not parked: 33 in A or B (1 wrong, 0 crash, 32 refuses), 44 with no class; tier 5: 54 in C, 10 in D; 2 small rows ready, and a bundle is 5
```

What moved is the fill's order, not the counts. The 64 C and D rows not parked
had been taken ahead of every row of tiers 2 to 4, and now they come after them.

The handoff that brought this to the session said the A-or-B count was 13 that
morning. Neither recipe reproduces 13: on the queue at `b4e95b52`, the old one
and the new one both read 34 in A or B, and 22 of those were ready.

### 403.7 A heading written above the row it moves

`tier_step` asked `block_for_tier` for the target block. That call writes a
missing heading, which is five lines, and parses the file again. `tier_step`
then removed the row at the line number it had read **before** the call. When
the new heading lands above the row, the number points five lines too high and
the wrong line is deleted. That happens whenever a row moves up into a tier
the file has no heading for.

`check-queue-api-selftest` has an arm for it now: in a file with no `## Tier
2.2`, it adds a tier 3 row and moves it to tier 2.2. Run before the fix, the
verb refused, because `check-queue` found the result broken:

```
docs/queue.md (as it would be):27: ##: a row is five fields and a description, and this one has 3 fields
docs/queue.md (as it would be):33: ##: a row is five fields and a description, and this one has 3 fields
docs/queue.md (as it would be):38: ##: a row is five fields and a description, and this one has 2 fields
docs/queue.md (as it would be):42: this fence is never closed, so every line after it reads as a row
docs/queue.md (as it would be): heading "## Closed" is missing
```

The deleted line was a fence. `check-queue` caught it, so nothing was written
wrong, but the verb refused a move it should have made. `tag` moves rows now
too, so both verbs move through one helper, `move_to_tier`, which finds the row
again by its id after the heading is written. The arm passes with it.

### 403.8 The queue page

`tooling/queue-page/render.py` has a label and a colour for five tiers, and
raises `KeyError` on any other (§307.4). So `rows.py`'s `TIERS` leaves out 5,
as it leaves out 2.2. The page's `tally()` counts each open tier 5 row in the
parked band, and the page leaves those rows out of its ready list. That is a
known wrong page. Q284 was to teach the page tier 2.2. It is dropped and filed
again as Q982 to cover tier 5 too, ranked where Q284 stood, and Q501, which
waited on Q284, now waits on Q982.

*Q982 taught the page all seven tiers on 2026-10-04. `rows.py`'s `TIERS` is
gone, and the page reads `TIERS_IN_ORDER`. Its eight bands take the eight
colours of the `dataviz` palette in that palette's own order, which passes the
skill's validator in light and dark. Keeping each older tier's colour and
adding violet and red for 2.2 and 5 fails it: red sits beside magenta, too
close to tell apart even with full colour vision. So every band after tier 2
has a new colour. The page's cache held 838 tallies taken under five tiers, and
the new page would have raised `KeyError` on the first of them. It now tallies
such an entry again, once: 19 s on the real history, then 1.7 s warm.*

### 403.9 The migration

One `just queue apply` moved the 65 open C and D rows from tier 1 to tier 5, in
the same push as the tooling. With the new rule, `check-queue` refuses the file
while they sit in tier 1. 50 were `todo`, 14 were `after:`, and one, Q22, was
`parked:`. None was in `lane:`, and no other worktree held any of them.

### 403.10 What this leaves

- **Guesses with no class stay in tier 1.** By §360.5's reasoning a guess
  nobody has run is less likely than C, so it may belong in tier 5 too.
  Mattias said C and D, and widening that is his decision. *He answered on
  2026-10-04: they stay. "I think it is right as it is now, unless we can
  decide that the possible failure actually is C or D, as if it is A or B we
  need to fix it as it is equally likely that a user run into that one as that
  it would run into the other A or B failures we have already identified." A
  guess moves to tier 5 only when a lane shows it is C or D.*
- **`just land` reports an old lane's C row as tier 1.** `rows_added` takes the
  tier from the `add` step (`tooling/land/land.py`, `apply_queue`). A lane that
  writes `add 1` and then `tag @1 refuses/C` lands its row in tier 5, and the
  report calls it tier 1. It is not fixed, because the land selftest's stub
  `queue apply` writes no real rows to read the tier back from. New lanes write
  `add 5`, by `lane.md`, so only lanes started before this change misreport.
- **A running process keeps the `lane.md` and `verifier.md` it started with**
  (§360.3), so its lanes write `add 1` for C rows until it restarts. 403.4 is
  what lets those land.
- **Other dispatchers run the old `queue.py` from their own trees** until they
  merge main. A C row one of them adds to tier 1 with the old code is refused by
  the new `check-queue` at their next merge of main, and the refusal names `just
  queue tier <id> 5`. That is intended.
- **Tier 3's corpus refusals**, such as Q347 and Q559, are "A, seen" by §352.3
  and carry no class (§352.7). Mattias asked whether they should become tier 1
  `refuses/A` rows. *He answered on 2026-10-04: they stay in tier 3. "It is my
  way of prioritizing fuzz issues before new work, to ensure that we do not
  just rush and fix the 'known' files first. Ideally the fuzzing should be able
  to fix the tier 3's STEP corpus before we reach there." A corpus file that
  fails is a tier 3 row with no class. The fuzzing is meant to find the same
  defect first, and a defect it finds goes to tier 1 or 5 by its class.*
- **Q500** asked for the queue file's header to name "the six tiers of §307".
  It is dropped and filed again as Q983 to name seven, ranked where Q500 stood.
