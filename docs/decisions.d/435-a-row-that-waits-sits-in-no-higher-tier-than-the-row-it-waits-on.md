## 435. A row that waits sits in no higher tier than the row it waits on — 2026-10-11

*Mattias, 2026-10-11: "How come, we have rows in Tier 1 which are blocked by
rows at a higher Tier, e.g. Tier 5?" The answer below ended in a proposal: a
check that refuses it, the waiting row moved down as the usual fix, and §403.2
changed to let a row with no class sit in tier 5. He answered "yes, write the
section and file the row". This amends §403.2 and §403.3. Q1736 built the
tooling; 435.10 says what it built and where it went another way.*

### 435.1 What is there

On `main` at `0280634f`, 12 open rows wait on an open row in a tier the fill
reaches after their own:

| row | its tier | waits on | that row's tier and class |
|---|---|---|---|
| Q343 | 1 | Q484 | 5, `refuses/D` (Q484 itself waits on Q489) |
| Q1398 | 1 | Q913 | 2, no class (ready, since Q912 is done) |
| Q1466 | 1 | Q1465 | 5, `refuses/C` |
| Q1541 | 1 | Q1538, Q1539 | 5, both `refuses/C` |
| Q333 | 2 | Q1315 | 5, `refuses/C` |
| Q486 | 2 | Q485 | 5, `refuses/D` |
| Q594 | 2 | Q570 | 5, `wrong/D` |
| Q608 | 2 | Q570 | 5, `wrong/D` |
| Q653 | 2 | Q557 | 5, `refuses/C` |
| Q779 | 2 | Q652 | 5, `refuses/C` |
| Q1564 | 2 | Q654 | 5, `refuses/C` |
| Q1565 | 2 | Q758 | 5, `refuses/C` (Q758 itself waits on Q756) |

None of the 12 has a class, and none is pinned. Moving each one down to the
tier of the row it waits on creates no new case, because nothing waits on any
of them. The count is from a scratch script that reads every open row's
`after:` and compares tiers. The check 435.3 asks for is the command that gives
it again.

### 435.2 How they got there

A row's tier comes from its own class (§403.2): A and B in tier 1, C and D in
tier 5, and a row with no class in tier 1. Nothing compares that tier with the
tier of the rows it waits on:

- `just queue after` checks only that each id exists somewhere.
- `check-queue` checks only that an `after:` names rows that can still finish.
- `tag` and `tier` move one row and never look at the rows waiting on it.

Rows got into this state two ways:

- **§403.9's migration** (`0380e2d8`, 2026-10-04) moved 65 C and D rows to tier
  5 and left the rows waiting on them where they were. Q343 was filed on 09-28
  (`33920981`), and its blocker Q484 went to tier 5 in that commit.
- **Lanes since then** file a C row with `add 5`, as `lane.md` says, and file
  the row that follows it with `add 1`, because the follow-up has no class.
  Q1541 was filed on 10-09 (`b2e9c056`), waiting on Q1538 and Q1539.

### 435.3 What it costs, and the rule

**No slot is lost.** A row is ready only when every row it waits on is done
(§230.2), and the fill takes only ready rows. Q1541 cannot go out before Q1538
and Q1539 are done, and the fill reaches those only when tiers 1 to 4 have
nothing ready that fits. Q1541 is a tier 5 row in everything but its label.

**The label is what is wrong.** Tier 1 reads as holding work it does not hold:
three of the 24 tier 1 rows with no class on today's measure line (§403.6) are
rows that cannot start before a tier 5 row is done, and a fourth waits on a
tier 2 row. Worse, it hides the one case that does need a decision: a row worth
having soon that waits on a row the fill takes last. Nothing says so today.

**The rule:** an open row sits in no tier the fill reaches before the tier of an
open row its `after:` names. The tiers in the order the fill reaches them are
1, 2, 2.2, 2.5, 3, 4, 5 (§230.1, §307, §403). `check-queue` holds it. An
`after:` naming a `Q` row this copy does not hold passes, as it does today
(§263.5), because its tier cannot be seen.

The rule is about tiers only. Inside tier 1 or tier 5 the fill also walks by
class, and a row can wait on a row its walk reaches later; 435.9 counts those
and leaves them.

### 435.4 Which of the two rows moves

When the rule is broken, one of the two rows is in the wrong place: the waiting
row, which should go down to its blocker's tier, or the blocker, which should
come up to the waiting row's tier.

**A waiting row with no class goes down.** The reasons:

- It cannot run before its blocker anyway, so moving it down costs it nothing
  while it waits.
- Moving a C or D blocker up would undo a judgement already made. §403 put it
  in tier 5 at Mattias's word. If any row filed in tier 1 that waits on a C row
  pulled that row up, then every C row with a follow-up would leave tier 5. A
  lane that works on a C row usually files one, so §403 would be undone one
  follow-up at a time.
- All 12 rows of 435.1 work on their blocker's shape or build on its fix:
  "Once Q328 lands, point …" (Q343), "Once the walk names the long arc of the
  R = 38613965 bar" (Q1466), the counts Q486 commits "once the row this waits
  on has landed". Each is worth what its blocker is worth.

**This reads Mattias's §403.10 answer, and it is a reading.** He kept rows with
no class in tier 1 because one may turn out to be A or B, "as it is equally
likely that a user run into that one as that it would run into the other A or B
failures". A row that waits on a C or D row and works on that row's shape is
not equally likely. If a lane shows one of them is A or B, `tag` takes it back
to tier 1, and then the next paragraph applies.

**A waiting row with a class stays where its class puts it, and the check
refuses.** A class A or B row waiting on a tier 5 row is the real inversion: a
defect a user meets that cannot be fixed until a row the fill takes last is
done. No verb settles that alone. The refusal names both rows and says the
order is Mattias's to set. The blocker sits in tier 5 at his word (§403), only
he pins (§430.5), and `pin` refuses a C or D row (§430.4), so the answer may
need that rule changed too. None of the 12 has a class, so this case does not
occur today.

**A pinned waiting row is the same case.** A pinned row sits in tier 1 and no
verb but `take`, `done` and `pick` moves it (§430.4), so it cannot go down.

### 435.5 A row with no class may sit in tier 5 (amends §403.2 and §403.3)

§403.2 says every open tier 5 row has a class. It now reads: **every open tier
5 row has a class, or its `after:` names a tier 5 row**, open or closed. A
closed row keeps the tier it closed in (§403.2), so a waiting row whose blocker
is done still says why it is in tier 5.

**Once its blocker is done, the row stays in tier 5**, ready where it is. Its
worth was its blocker's shape, and that did not change when the blocker closed.
Moving it back up would also need the tier it came from, which the file does
not keep: 8 of the 12 came from tier 2, not tier 1.

**Where tier 5 walks it.** The fill already sorts a row with no class between
`refuses/C` and `wrong/D` in every tier (`_queue-rows`, the `classes` list), so
tier 5's walk becomes `wrong/C, crash/C, refuses/C, <no class>, wrong/D,
crash/D, refuses/D`. That is the place §352.3 first gave such a row, "just
above D", for §360.5's reason: a row nobody has shown is less likely than any
row a run has shown. No change to the fill; its comment and §403.3's walk
change.

### 435.6 What the verbs do

- **`after <id> <ids>`**: when a row it names sits in a later tier than `<id>`,
  a row with no class moves to the latest such tier, at its foot, and the
  message says so, as `tag` does when it moves a row (§403.4): `Q9: todo ->
  after:Q8, tier 1 -> 5, beside Q8, which it waits on`. A row with a class is
  refused (435.4). A pinned row is refused already (§430.4).
- **`tier` and `tag`, moving a row to a later tier**: each open row with no
  class that waits on it, and would now sit above it, moves with it, and so on
  along the chain. The message names each row moved. If one of them has a class
  or is pinned, the step is refused and names it.
- **`tier` and `tag`, moving a row to an earlier tier**: refused if the row
  would then sit above a row it waits on, naming that row. That includes `tag
  Q9 refuses/B` on a moved row whose blocker is still in tier 5, which is
  435.4's case for Mattias.
- **`tag`, moving a tier 5 row up**: a row with no class in tier 5 whose
  `after:` names no other tier 5 row has lost what keeps it there, so it moves
  with its blocker, to the blocker's new tier. That may be above the tier it
  first had, which the file does not keep (435.5).
- **`pin`**: refuses a row that waits on an open row outside tier 1, naming
  that row.
- **`add`** gets no rule of its own, as in §403.5. `check-queue` runs at the
  end of the batch, so `add 1 …` followed by `after @1 Q8` lands the new row in
  Q8's tier, and a lane's land steps need no change. `lane.md` says so in one
  sentence beside its `after` step.
- **`done`, `drop` and `park` of a blocker** move nothing (435.5).

### 435.7 The measure line

§403.6 counts "no class" in tier 1 only, so a row moved to tier 5 would drop out
of the line altogether. The tier 5 half gains a count: `tier 5: N in C, N in D,
N with no class`. With the 12 rows of 435.1 moved, today's line, `tier 1, open
and not parked: 19 in A or B (0 wrong, 0 crash, 19 refuses), 24 with no class;
tier 5: 150 in C, 22 in D; …`, reads 20 with no class in tier 1 once the 12
are moved. That was measured with `just --set queue_file <copy> queue` on a
scratch copy whose tier column was changed for those 12 rows and nothing else.
The tier 5 half would gain 11 with no class (Q1398 goes to tier 2, which the
line does not count).

### 435.8 The migration

The rows of 435.1 move in the same push as the check, as §403.9 moved the C and
D rows with §403's: without the move, the new `check-queue` refuses `main`'s
file. The lane counts again on its own base and moves what it finds there, with
`tier` steps in one `apply`. They went as `after` steps instead; 435.10 says
why.

### 435.9 What this leaves

- **The row that builds it is Q1736**, tier 2.2 (§307.1), filed with this
  section. Until it lands, nothing holds the rule, and rows filed meanwhile can
  break it. One already has: on `main` at `14d126b3`, committed 35 minutes
  after `0280634f`, Q1731 (tier 2) waits on Q1728 (tier 2.2). The lane counts again on its
  base for that reason.
- **Inside one tier, 9 rows of tier 5 wait on a row the class walk reaches
  after theirs**, counted with the 12 already moved: four moved rows with no
  class wait on D rows (Q343, Q486, Q594, Q608), because the walk puts no class
  above D; five C rows wait on a later C or D row (Q1098 on Q618, Q1547 on
  Q1471, Q1552 and Q1553 on Q484, Q1689 on Q652). Tier 1 has none. The fill
  reaches both rows of each pair only after tiers 1 to 4, so the cost is the
  same label error on a smaller scale. Not checked.
- **Q249, not built, would have `done` take a closed id out of every `after:`
  that names it.** On a tier 5 row with no class, that removes the only thing
  that says why the row is there, and `check-queue` would refuse it. Q249 must
  leave the id on such a row or move the row. Since the check catches it, it
  cannot land wrong unnoticed.
- **`just land` will report a moved row's tier from its `add` step**, as §403.10
  found for C rows: `rows_added` takes the tier from the step
  (`tooling/land/land.py`, `apply_queue`), so a lane that writes `add 1` and
  then `after @1 Q8` lands its row in tier 5 and the report calls it tier 1.
  Same cause, not fixed here.

### 435.10 What Q1736 built — 2026-10-12

**`check-queue` holds 435.3's rule.** It refuses an open row with `after:` that
sits in a tier the fill reaches before an open row it names, and names both
rows and both tiers. A row with no class and no pin is told the verb that moves
it: `move it down with just queue tier <id> <tier> (§435)`. That is `tier`, as
§403.2's refusals name it, and not `after`, which would need the row's whole
list typed again. A row whose class or pin keeps it where it is is told the
order is Mattias's to set (435.4). A closed blocker passes, and so does a `Q`
row this copy does not hold (§263.5).

**It holds 435.5 too.** An open tier 5 row with no class passes only if its
`after:` names a tier 5 row, open or closed, or a `Q` row this copy does not
hold.

**The verbs keep the rule as 435.6 says.** `after`, `tier`, `tag` and `pin` all
move a row through one function, `move_row` in `tooling/queue/queue.py`. The
measure line gains `N with no class` in its tier 5 half (435.7), and the
`queue` recipe's comment gives tier 5's walk with those rows between `refuses/C`
and `wrong/D`. `lane.md` says in one sentence beside its `after` step where
`after` lands a new row. `check-queue-selftest` pins each refusal and each pass
of the check. `check-queue-api-selftest` pins each move and refusal of the verbs,
and the merged queue of two copies refused for a row whose blocker only the
other copy held, until `tier` moves it.

The lane decided these itself:

- **The 12 rows of 435.1 move as 12 `after` land steps, not as `tier` steps on
  the branch** (435.8 said `tier`). A lane's land steps take only `add`, `after`
  and `tag` (`STEP_ARGS` in `tooling/land/land.py`), and a lane does not edit
  `docs/queue.md` (§230.2). Each step names the row's own `after:` list again,
  and the new `after` moves the row down. So the branch's own gate fails on
  `check-queue` alone, with the 12 rows in 13 refusals (Q1541 waits on two),
  until `just land` runs the steps: `land_one` runs `queue apply` on the merge
  before its `check-queue`. The rows, counted again by the new check on the base
  `789396ef`: Q343, Q1398 (to tier 2), Q1466, Q1541, Q333, Q486, Q594, Q608,
  Q653, Q779, Q1564 and Q1565. That is 435.1's list. 435.9's thirteenth pair,
  Q1731 on Q1728, no longer counts, because Q1728 is done.
- **Each step names the row's whole list, not only the blocker 435.1 names.**
  `after` writes the list it is given, so Q653's step names Q557 and Q639,
  where 435.1's table names only Q557, and Q1541's names Q1539 and Q1538 in
  the row's own order.
- **A row that a lane holds, or a parked one, passes in tier 5 with no class.**
  `take` and `park` write over the `after:` that said why the row is there.
  Refusing it would stop every `take` of such a row.
- **A row taken up out of tier 5 goes to the latest tier among the open rows it
  still waits on**, not to "the blocker's new tier" (435.6). A row that waits on
  two rows must sit no higher than either of them. This holds for every verb
  that moves a row out of tier 5, not only `tag`. So `pin` of a tier 5 row with
  no class takes up with it the rows that wait on it, and 435.6 did not say
  that.
