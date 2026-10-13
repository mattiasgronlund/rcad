## 445. A row that waits lends its place in the fill to the rows it waits on — 2026-10-13

*Mattias, 2026-10-13, after a dispatcher's `after` was refused: "Should this
really be blocked, would it not be better that the dispatcher could get a
rejection, with two options: 1. Force the relationship into the lower Tier,
which would recursivly move the rows addressed by the new dependency to the
same tier as the row which were to get the new dependency. 2. Change tire of
the row before adding the dependency. The rule should probably be like in real
time kernels, where a high priority task increases the priority of tasks it is
waiting for, or?" The session answered with the reading below: the fill lends
the place, and the file does not move. He answered "yes". This amends §435.4,
§435.6 and §435.10.*

### 445.1 What happened

At 10:17 UTC a dispatcher re-filing Q1116's rows added Q1903 (`refuses/B`,
tier 1) with `after @4 Q1014`. Q1014 is in tier 2 with no class, and builds the
pins of the same shape. `after` refused it: "Q1903 carries the class refuses/B,
which keeps it in tier 1, and it would wait on Q1014, which is tier 2: which of
the two moves is Mattias's to set (§435.4)". The dispatcher then filed Q1903
without the wait, and wrote "Q1014, tier 2, asks for the pins of this same
shape" into its text. The order became prose that the fill does not read. Q1903
was ready in tier 1 and could go out before Q1014.

**The refusal said a move was Mattias's when it was not.** On a scratch copy of
`main` at `90600d02`, `just queue apply tier Q1014 1 + after Q1903 Q1014`
applied, and `check-queue` passed. §435.4 leaves the order to Mattias because
the blocker it had in mind sits in tier 5 at his word (§403). Q1014 has no
class, and the message did not tell the two cases apart.

**This is the second case, and §435.4 said a second case would be the rule's
consumer (§13).** The first was Q1845 (`wrong/B`) on Q1870, on 2026-10-12.
Mattias settled it by moving the blockers up, and `tier` would only take Q1868
and Q1869 to tier 1 once their C class was taken off. In the session
transcripts since 2026-10-11, these two are the only refusals of 435.4's kind
outside the selftests' fixtures (Q7, Q77).

### 445.2 The rule: a waiting row lends its place

**The fill walks each open row at the earliest place of itself and of every
open row that waits on it, along the whole chain.** A place is what the fill
orders by: a pin first, then the tier, then the class (`_queue-rows`). A row
lends its place when that place is a judgement: it has a class, it is pinned,
or it was lent a place itself.

The file does not change. A row that is lent a place keeps its own tier and
class. When the row that lent it is done, dropped, parked, taken or graded
again, the lent place goes too, because nothing was written.

This is priority inheritance as a real-time kernel does it. A task waiting on a
lower one lends that one its priority while it waits; the lower task keeps its
own priority and gets it back. A row waiting on a row the fill takes later is
the same inversion: Q1903, in tier 1, waiting on Q1014, in tier 2, would go out
after every ready row of tier 1 and so is a tier 2 row in all but its label.

### 445.3 Why the file does not move

Mattias's option 1 moves the rows waited on, down the chain, into the waiting
row's tier. Writing that move into the file was turned down for three reasons:

- **A C or D row cannot sit in tier 1.** `tier` refuses a row with a class
  outside its class's tier (§403.2). On 2026-10-12 Q1868 and Q1869 had to lose
  their class to move, and their grade now lives only in 435.4's note.
- **A written move is never undone.** When the waiting row is dropped, graded
  again or filed again, the moved row stays up, and the file does not say why.
  The file does not keep the tier a row came from either (435.5 meets the same
  gap the other way round).
- **§403 put C and D rows in tier 5 at Mattias's word.** A lent place leaves
  that in the file, and changes only when the row goes out, and only while a
  row that needs it waits.

Option 2, moving the row first, already exists for a row with no class: `tier`
does it, and 445.5 keeps it.

### 445.4 A row with no class does not lend

A row with no class sits in tier 1 by default, not by grade. §403.10 kept it
there because it may turn out to be A or B. If it lent its place, every C row
with a follow-up filed in tier 1 would go out at tier 1, and §403 would be
undone one follow-up at a time. That is 435.4's reason, and it still holds. So
a row with no class still moves down to the tier of the row it waits on, as
435.6 has `after`, `tier` and `tag` do, and `check-queue` keeps 435.3's rule for
it. The four tier 5 rows with no class that wait on a D row (435.9) stay as
435.9 left them.

A row that was lent a place lends it on, with or without a class. Q1870 has no
class, and Q1845 (`wrong/B`) cannot go out before Q1868 and Q1869, which Q1870
waits on.

### 445.5 What the verbs and the check do

- **`after <id> <ids>`** on a row with a class is accepted, and its message says
  what it lends: `Q1903: todo -> after:Q1014, lends tier 1 refuses/B to Q1014`.
  A row with no class moves down as it does now. A pinned row is refused as it
  is now (§430.4).
- **`tier` and `tag`, moving a row to a later tier:** a row with a class or a
  pin that waits on it no longer stops the move; it lends. Rows with no class
  are still taken along (435.6).
- **`tier` and `tag`, moving a row with a class to an earlier tier** past a row
  it waits on: accepted, and it lends. A row with no class is still refused.
- **`pin`** no longer refuses a row that waits on an open row outside tier 1
  (435.6); the pin is lent.
- **`check-queue`** refuses an open row with no class and no pin that sits in a
  tier the fill reaches before an open row it waits on, as 435.3 says. It passes
  a row with a class or a pin.
- **No message says "Mattias's to set (§435.4)" any more.**
- **`just queue`, `ready`, `blocked` and `next`** walk the lent place. The
  listing names it beside the row, for example `lent tier 1 refuses/B by
  Q1903`, so that tier 2's rows do not hide one that goes out with tier 1. The
  measure line (§403.6, 435.7) still counts the tiers in the file. The queue
  page reads the listing and the `blocked` lines and asserts on any line it
  cannot read, which Q1856 shows. So it learns the new form in the same push.

### 445.6 What it changes today

Measured on `main` at `df8acbb4` with a scratch script that lends places as
445.2 says over the open rows of `docs/queue.md`. Nine open rows would be walked
earlier than their own place:

| row | its own place | walked at | lent by |
|---|---|---|---|
| Q1866 | tier 1, no class | tier 1 `refuses/B` | Q1867 |
| Q1868 | tier 1, no class | tier 1 `wrong/B` | Q1870, lent by Q1845 |
| Q1869 | tier 1, no class | tier 1 `wrong/B` | Q1870, lent by Q1845 |
| Q1870 | tier 1, no class | tier 1 `wrong/B` | Q1845 |
| Q489 | tier 5 `refuses/D` | tier 5 `refuses/C` | Q484, lent by Q1552 |
| Q484 | tier 5 `refuses/D` | tier 5 `refuses/C` | Q1552 |
| Q618 | tier 5 `refuses/C` | tier 5 `crash/C` | Q1098 |
| Q652 | tier 5 `refuses/C` | tier 5 `wrong/C` | Q1689 |
| Q1471 | tier 5 `refuses/C` | tier 5 `wrong/C` | Q1547 |

Tier 1 holds 24 ready `refuses/B` rows and 20 ready rows with no class, and
nothing else ready. So Q1868, Q1869 and Q1870 move from the head of the rows
with no class, where 435.4's note ranked them on 2026-10-12, to ahead of all 24
`refuses/B` rows. That follows from the rule: Q1845 is a wrong answer a user
meets, and it cannot be fixed before those three.

The five tier 5 rows are the pairs 435.9 counted and left unchecked. The rule
puts them in order without a check of its own.

### 445.7 Q1903 now, and what moves when the rule is built

Q1903's wait was put back the same day with today's verbs: `tier Q1014 1 +
after Q1903 Q1014`. Once the rule is on `main`, and if Q1014 is still open,
`tier Q1014 2` takes it back to tier 2, and Q1903 lends it tier 1. A lane's
land steps take no `tier` (`STEP_ARGS` in `tooling/land/land.py`, 435.10), so
that step is the dispatcher's. Q1903's text says "Q1014, tier 2"; no verb edits
a row's text, and that reads true again once Q1014 is back.

**Left alone: giving Q1868 and Q1869 back `refuses/C`.** Under the rule they
could carry their class again and still go out at `wrong/B`'s place. But `tag`
would take Q1870 down with them, and Q1846 with it, since Q1846 has no class and
waits on Q1870 (435.6). Q1846 would then be walked in tier 5. Mattias placed
these rows on 2026-10-12, so the row that builds this leaves them.

### 445.8 The row

Q1906, tier 2.2 (§307.1), filed with this section, builds 445.2 to 445.5 and
adds 445.9, saying what it built. Until it lands, a row with a class still
cannot wait on a row the fill takes later; `tier` on a blocker with no class,
as 445.7 did for Q1014, is the way round it meanwhile.
