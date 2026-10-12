## 352. Rank tier 1 by how bad and how likely, and send small rows in bundles — 2026-09-29

*Mattias, 2026-09-28, after §348: "record them all, but prioritise them after
the probability we assign them for how probable we think it is that a user
runs into the problem and would report it as a defect." He had rejected filing
only what a real input reaches: the point is to find the edge cases before a
customer does. Then, on the rows that are not defects: "for missing tests, and
other smaller things, it would be better if we could merge them together into
a one row adding multiple tests of other smaller things. I suspect we pay alot
for them when we process them one by one instead of in a batch, as these rows
should othervise be fixed at a high pace." This section is both answers. It
amends §230.2, §230.3, §230.7 and §311.4, and it settles §348.7: that proposal
is not adopted.*

### 352.1 Every finding is still filed; the queue's length stops being the measure

§230.7 stands: what a lane leaves becomes a row. §348 found the queue growing
about twice as fast as it closes, and the fix is not to file less. It is to
stop judging the queue by how many rows it holds. What counts now is how many
open tier 1 rows are in buckets A and B (§352.3), and §352.6 prints that on
every `just queue`.

### 352.2 The tag is the first word of a row's description

A row can say how bad and how likely it is in three places. They were weighed
like this:

- **A new field in the row grammar.** Every row changes on one day. Every
  reader that picks fields by position breaks: the `_queue-rows` awk,
  `tooling/queue/rows.py`, the queue page's `render.py`, the merge driver's
  `merge.py`, and `guard.py`. How `add` is called changes too, while the live
  dispatcher's mergers carry the `merger.md` they read when their process
  started, and agent types do not reload in a running session (§266.6).
  Rejected.
- **Sub-tiers** like 2.2 and 2.5. Two measures squeezed into one tier number
  need eight more headings, and the tier number stops saying one thing.
  Rejected.
- **The first word of the description.** Every reader stays as it is. A row
  with no tag is still valid, so a merger that knows nothing of tags still
  writes a good row. Taken.

A tag is one of:

- **a class**, `wrong/A` to `refuses/D` (§352.3). Only a tier 1 row may carry
  one. *§403.2, 2026-10-04: a tier 1 or tier 5 row, and the class decides an
  open row's tier, A and B tier 1 and C and D tier 5.*
- **`small`** (§352.4). A row of any tier may carry it.

A row carries one tag at most. A defect needs a trace before anyone knows where
the fix goes, so a defect is never small.

`just queue tag <id> <class|small|->` sets, replaces or, given `-`, removes a
row's tag. It refuses a closed row, a class on a row outside tier 1, and a tag
the row carries already. `just queue tier` refuses to move a row that has a
class, and names `tag <id> -` as the way out, since the class would be wrong in
its new tier. `just check-queue` refuses a first word shaped like a class
(`^[a-z]+/[A-Z]$`) that is not one, which catches `wrong/E`, and it refuses a
class on a row outside tier 1. No open row started with the word `small` when
this was written, so the word was free.

### 352.3 The classes, and the order the fill walks tier 1

A class is how bad the symptom is, then `/`, then how likely a user's model
meets it.

How bad, worst first:

- **`wrong`**: a wrong answer. Nothing outside the kernel can tell it from a
  right one.
- **`crash`**: a panic, an abort, an overflow, or a hang past the timeout.
  CLAUDE.md's first principle says "Crash > corrupt", so a crash is the lesser
  harm and sits below `wrong`.
- **`refuses`**: a named refusal of an input that has an answer.

**The class goes by what the user sees.** `wrong` is only an answer returned
as `Ok` that is wrong. A named error is `refuses`, even when the kernel built
the answer wrong and caught it itself (`Error::TheFuseWroteABrokenAnswer`,
`Invalid(...)`), and so is a `Model::validate` that turns away a sound solid.
The tagging of the open rows (§352.7) moved thirteen rows from `wrong` to
`refuses` on this rule.

How likely, as buckets tied to evidence rather than a guessed percentage:

- **A, seen**: a real input met it, such as a file of the STEP corpus. *A
  corpus file that fails is still a tier 3 row and gets no class, so that the
  fuzzing has a chance to find its defect first (Mattias, 2026-10-04, §403.10).*
- **B, typical**: a shape designers make on purpose (tangent, flush, coaxial,
  a fillet), or one that a turn or a file written to 11–13 digits rounds apart.
- **C, possible**: inside §3.5's scale envelope, but only a probe builds it. A
  hypothesis nobody has run has no class until a lane runs it (§360.5).
- **D, never**: outside §3.5's envelope, with the number that puts it there,
  such as Q325's bar moved by 2^-1007 or Q335's towers 1e308 tall. That number
  is the demonstration §16.10 asks for.

The fill walks tier 1 in this order:

`wrong/A, wrong/B, wrong/C, crash/A, crash/B, crash/C, refuses/A, refuses/B, refuses/C, <no class>, wrong/D, crash/D, refuses/D`

- **How bad comes first** for A to C, as Mattias adopted. A wrong answer is
  silent, so "would a user report it" undercounts it.
- **D comes last whatever its symptom.** No user's model reaches it, so its
  chance of hurting one is taken as zero. It is still filed and still fixed,
  after everything a user can meet.
- **Rows with no class go just above D.** That is where `add` puts a new row
  today, at the foot of its tier, so a merger that knows nothing of classes
  changes nothing in the order.

*§403.3 split this walk on 2026-10-04. C and D rows moved to a new tier 5
below tier 4, so tier 1 walks only A, B and the rows with no class, and tier 5
walks `wrong/C, crash/C, refuses/C, wrong/D, crash/D, refuses/D`.*

*§360.5 changed the C bucket on 2026-09-29. It said "a hypothesis nobody has
run is C at most", and every guess came in as C, so the walk took guesses
ahead of every refusal a user meets and the A-or-B count stood still. A guess
now has no class until a lane runs it, and 19 open rows lost their class.*

*§440.2, 2026-10-12: a fault filed from a fuzz crash file is graded C or D only
after its shape has been run again with ordinary numbers. Until then it has no
class. The input the fuzzer keeps is the first to reach the code, and that is
often the least likely one.*

The file's line order breaks ties only inside one class. **The fill sorts by
class; the lines do not have to be in class order.** They could not be kept
that way: the merge driver (`tooling/queue/merge.py`, `assemble`) keeps the
frame side's order inside a block, so a merge across a re-tag would undo any
order the lines held.

This amends §230.3, whose rule was "the first ready row of the highest tier".
It is still that, with tier 1's rows in class order.

### 352.4 Small rows go out in bundles

A row is **small** when it needs no trace, its place is known, and the work
takes about ten minutes: one test, one stale claim, a fix of a few lines, one
more arm for a selftest.

- **Each item keeps its own row.** Its id stays, so an `after:` can still name
  it, and nothing is filed differently.
- **Only dispatch groups them.** One lane takes up to `queue_bundle` (5) ready
  small rows, on one branch, with one verifier and one merger, so the gate runs
  once for all of them. Five rows of about ten minutes fit the 45 minutes
  `lane.md` gives a lane. The merger closes each row `done:` with the same lane
  tip. A row the lane found was not small it leaves undone, and says so; that
  row stays held, the merger names it, and the dispatcher takes it again.
- **Not one row holding many items.** Two mergers folding items into the same
  row at once conflict on that row in the merge driver, and `just queue pick
  ours|theirs` keeps one side. An item is lost.

How `just queue next` takes them:

- **A full bundle goes first.** When at least 5 ready small rows meet no
  running lane and no row held elsewhere, the first free slot takes the first
  5, in walk order, ahead of tier 1. That is what "fixed at a high pace" asks
  for, and the cost is bounded: it happens only once 5 have piled up.
- **Only one full bundle per fill goes ahead of tier 1.** The design first said
  "the first free slot takes the first 5", and looping on that would let a pile
  of small rows take every slot from tier 1.
- **Otherwise a small row waits for the walk.** When the walk reaches a small
  row by its tier, it takes that row and the next ready small rows after it in
  the walk, whatever their tier, up to 5, as one lane. A small row is never
  taken alone; a bundle of 1 is still printed as a bundle.
- **Write sets.** Rows in one bundle may overlap each other, since one lane
  does them all. A small row that meets a running lane, a row held elsewhere,
  or a row already taken is skipped as any row is. The bundle's write set is
  the union of its rows' sets, and `-` adds nothing.
- **Its line** is `take Q1,Q2,Q3 (a bundle of 3 small rows, one lane)`,
  with each row's text indented under it. The id list is what the `lanes`
  workflow takes as `queueId`.
- **One bundle is one slot.** `just queue running` counts holds per branch,
  not per row, so the rows of a bundle, all taken onto one branch, hold one
  slot between them.
- **Its model** is opus when any of its rows says opus, else sonnet. The
  dispatcher chooses (`dispatch-lanes`, step 4).

### 352.5 Rows about the process

These rows have no user who could meet them, so the classes do not fit them.

- **A lane fixes what its own change leaves stale.** A doc comment or a
  section's claim its change makes false, or a test its change leaves missing,
  is fixed inside its write set before it reports, not filed. `lane.md` says
  so, and `merger.md` step 4 makes such a thing a step 2 fix when it is a few
  lines. This is §230.7's one exception.
- **Hypotheses are filed.** One that predicts a kernel defect is a tier 1 row
  with no class until someone runs it (§360.5). Any other goes in its tier.
  A row of the form "build a shape that reaches this code, or show none can"
  is such a hypothesis when reaching the code would be a defect. When the code
  there is already right and the row only asks for a test that fails without
  it, the row is tier 2 with no class.
- **A row that waits on a capability not built yet keeps its tier.** Its
  `after:` says what it waits on; the tier does not.
- **Follow-ups about the tooling and the measuring are filed in their tiers**
  (2, 2.2, 2.5), as they are now, and tagged `small` when they are small.

§348.7 proposed filing such follow-ups only when Mattias asks, and filing a
refusal only inside the scale envelope. **It is not adopted**, because:

- Mattias said "record them all".
- The number of rows is no longer the measure (§352.1).
- §226.1 found that a finding left only in a section is lost.
- The tier order already keeps these rows behind every defect, and D keeps
  what no user reaches behind everything a user can.

This settles what §311.4 left open: follow-ups about the fuzzer are still
filed, and they wait in tier 2.

### 352.6 The measure

The plain `just queue` output ends with one line:

```
tier 1, open and not parked: N in A or B (w wrong, c crash, r refuses), N in C, N in D, N with no class; N small rows ready, and a bundle is 5
```

The first number is the one to watch. The rows' ages are left to the queue
page (§352.7). Right after the open rows were tagged it read:

```
tier 1, open and not parked: 68 in A or B (4 wrong, 1 crash, 63 refuses), 23 in C, 11 in D, 16 with no class; 22 small rows ready, and a bundle is 5
```

"No class" counts the `small` rows of tier 1 and the rows in `lane:`, which
were left untagged, as well as the rows that carry no tag at all.

*§403.6, 2026-10-04: the line now counts C and D under tier 5, `tier 1, open
and not parked: N in A or B (…), N with no class; tier 5: N in C, N in D; …`.*

### 352.7 What this leaves

- **The queue file's own header** still says five tiers (§307 added 2.2) and
  names no tags. The write guard refuses every hand edit to `docs/queue.md`,
  its prose included, so only a row can fix it, and that row needs a checked
  way to change the header first. Q500, tier 2.2.
- **The queue page shows the measure and the rows' ages**, after Q284 teaches
  it tier 2.2. Q501, tier 2.2.
- **The open rows got their tags** in a separate commit on 2026-09-29, from a
  read-only sweep that labelled every open row from its text. Its labels are
  one reader's judgement, and two sessions checked them against the rules in
  §352.3 and §352.5 before they were applied. That check moved thirteen rows
  from `wrong` to `refuses` and Q378 from `wrong/D` to `wrong/B`; moved seven
  rows from tier 2 to tier 1 and two the other way; dropped the sweep's
  `small` on Q343 (it may need `intersect` to answer a new plane) and Q368 (it
  re-counts a table as well as un-ignoring two tests), and its class on two
  tier 3 rows, since only tier 1 carries one. It also tagged the rows filed
  after the sweep ran. Rows in `lane:` were left alone, so that no merger
  meets a row changed on both sides.
- **A merger started before this change** carries the old `merger.md` until
  its dispatcher's process restarts. The rows it adds have no class and go
  just above D, which is where they went before, so nothing breaks; they wait
  for a `tag`.
