## 47. The six arms, closed — recorded 2026-09-04

Ran 2026-09-03 19:37 to 2026-09-04 00:01, 4.4 hours of wall clock, 61 commits,
46 files, +7630/−640. **Twenty agents, about 6.2 agent-hours**, against a budget
of 9 with 1.5 reserved for the gate before the first lane started. The suite goes
from 1467 tests to **1508, none skipped** — the repo has no `#[ignore]` left for
the first time.

### 47.1 What it earned, and what it did not

**Earned, and falsifiable by counting:**

> **`where_a_plane_and_a_cone_cross` answers all six of its arms, and the kernel
> can draw every shape a plane cuts a cone in.**

Open `intersect.rs`, find the match on `(apex_is_on_the_plane, leaning)`, count
the arms that return `Err`: **zero**. Four variants were deleted outright and are
referenced nowhere in `crates/`. The parabola and each branch of the hyperbola
are walked into points on both surfaces; the ruling and the two lines through the
apex are straight runs told apart by pickers.

**Not earned: the fuse clause.** §46.1 said *and a frustum can be cut by a block*
would be added only if the fuse lane landed, and only against the pinned
assertion rather than a printed number. The fuse lane landed. The pin broke, 0
answered → 6 of 96. **And the clause is still false**, which is §47.2.

### 47.2 The pin was right and it was not enough

`AFrustumAgainstABlock` was pinned at `assert_eq!(answered, 0)` before any lane
was briefed, precisely so an improvement had to break a test rather than move a
`println!`. It broke, and the dispatcher re-derived the row at 6 answered and
wrote beside it: *"a block can now take a piece out of a frustum."*

**That sentence was false, and a lane sent to check it found out why.** The six
that answer are **all one row of the sixteen — the row where the block misses the
frustum altogether.** The block is always put down with its low corner at `x = 1`
and the frustum is never wider than `slope · far`, so the two are disjoint exactly
when `slope · far < 1`, and one row of the grid lands there. All six of its fuses
answer; all ninety fuses of the other fifteen rows refuse. **No block in that
family has ever cut a frustum.**

So the lesson is not "pin your numbers", which the unit already did. It is:

> **An assertion on a count is still a count.** A number moving in the right
> direction is not evidence that the thing the number stands for happened. What
> was missing was not a tighter pin — it was a check on what the answers *are*.

That check now exists: `Model::validate` on all six, volumes against closed forms
with the bound checked finite first, inclusion–exclusion across union and
intersection, both ways round, and a 40,625-point classification grid each answer
must match. All six pass it. They are right — they are just right about a case
nobody wanted to claim.

**And the ninety refusals moved, which is the real result.** They no longer say
*"a plane cuts this cone in a hyperbola"* — the geometry — but *"nothing can put
points in order along this curve"* — the arrangement. Demonstrated rather than
read off a label: `intersect` names the crossing, `where_a_closed_crossing_starts`
refuses both runs as `TheCrossingOfTheseTwoSurfacesIsNotClosed`, and the fuse
refuses after that. A hyperbola branch is a conic and is not closed, and *"round a
closed curve"* is the only conic order `arrange` has. **That is the next unit's
sentence, and this unit found it by moving a number and then asking what the
number meant.**

### 47.3 The first-refusal rule scores on its second branch

§46.6's rule, committed before the number was read, with the denominator named:
files that reach `SolidBuilder::build` — **265 of 323 KiCad, 11 of 44 NIST**.

The lane that stopped `brep.rs` overwriting the builder's own refusal measured
the curved-corner bucket at 168 → 84 on KiCad. **Two gates re-derived it at the
final tree and got 41% and 43.5% by independent routes** — because two later
lanes in the same unit moved it back up. So:

- the bucket does **not** shrink by more than half → **§46.6's second branch
  fires**, §45.3's relabelling is scored as noise, and option D's trigger is
  scored on the published number;
- **§46.13's tie-break is moot.** The unit spent a commit deciding which way 50.0%
  falls, and 50.0% was never the number. It is kept in the record because a rule
  read after the number is not a rule, and the discipline was right even though
  the boundary was irrelevant;
- no file-count projection is published, and the 68 B-spline files are not priced.

**§45.7 and §45.8 both score as right.** §45.8 predicted a lane's number would be
true when taken and false when merged, and that this is how the unit would lose.
It did happen, twice, and it cost nothing — because the plan forbade the lane
that measured it from publishing it, and made the gate generate it at the final
tree instead. **The prediction was correct and the countermeasure worked.**

What the corrected reporting bought, at the final tree: the 84 KiCad files that
stopped being called a corner name what actually stopped them — 44 crossings
where two surfaces are too nearly parallel to follow, 15 faces walking a closed
edge against the way its surface winds, 12 outer wires winding against the face
normal, 8 needing a half-space `f64` cannot write down. §43.5 and §44.6 cite 158
and 8 for these rows and are stale; the numbers above replace them.

### 47.4 Five published sentences were false, and two gates found them

The unit ran two closing gates in parallel — one re-deriving every load-bearing
number from the final tree, one auditing the claims. Between them, **five
sentences published in this repository were false at the tree that published
them**, none of which any lane caught:

1. **A volume "checked against the kernel's own published bound"** where the bound
   is `f64::INFINITY` for a pointed cone, because the radius of curvature goes to
   zero at the apex. `assert!(x <= inf)` cannot fail. The tree already forbade this
   exact pattern, in a file whose doc says to check against *a stated share of the
   deflection* rather than *dressing an unpublished number as a bound* — so the
   rule was written down and broken three hundred lines away.
2. **A winding rule justified by a case its own walker cannot produce.** The
   arithmetic was right (1.000 against 1.273); the pair is unreachable, because a
   bigger radius never gets fewer steps at one deflection. Confirmed twice, by
   different routes, 84,672 pairs and 24 × 64 pairs, zero sign flips.
3. **"This never refuses"**, five lines below the `# Errors` section documenting
   the two ways it refuses.
4. **"The length rule is the only thing that asks"** and **"every other check
   still runs over it"** — both measured by running one check in isolation and
   reporting its `Ok(())` as the fate of the model.
5. **"Reads as not straight, which is the safe side"** — where `false` is the
   two-arc branch, and two arcs where a ruling has one is a wrong answer, not a
   conservative one.

**Three of the five are one mistake**: a verdict measured on one function's return
value and reported as what the kernel does. That is now the standard those docs
are held to, written into them.

**And the number gate corrected two of its own verdicts an hour after publishing
them**, for the same reason: it had checked the number and stopped short of
checking the sentence the number was offered as evidence for. Recorded because it
is the cheapest available demonstration that the failure is structural and not a
lane being careless.

### 47.5 The unit's real yield was correctness, not the headline

The headline was a naming-and-drawing gap and it closed on schedule. What the
unit is actually worth is five holes nobody planned to find, four of them found
by asking a lane to justify a sentence it had already written.

| what | was |
|---|---|
| the winding of a flat face | certified **backwards** on a sliver — the walk comes back with zero points at pass 27, the margin collapses, and the corner polygon's sign is taken. Reachable under a tolerant policy, which is what the STEP door uses |
| a fuse of solids carrying a run the kernel cannot read | **committed an answer** — `Ok(fused)` built out of solids nothing can describe |
| a crossing shape with no row in the fuse | **crashed** the fuse, or was **skipped in silence** so a solid came back sound that nobody had looked at |
| a plane against a cone running straight | put **two arcs where a ruling has one** — the whole cone row, for three units |
| a model read back from bytes | took a face in no shell, a wire on no face, a coedge in no wire |

Each is now a refusal with a name, and each has a test. Four of the five are the
same shape: `EdgeRun` and `SurfaceMeet` are `#[non_exhaustive]` in `rcad-geom`, so
a new crossing kind arrives in `rcad-topo` **with zero compiler errors**, and this
is the unit that grew the vocabulary. Twenty-odd sites were swept and given
written verdicts; whether an edge closes now lives in **one** list rather than
six copies.

Two claims were re-classified in the direction §16.10 warns about. A hyperbola
whose apex plane does not round exactly was refused as a **limit**; the answer
existed — round the offset up, and check the near branch's clearance, which the
interval machinery already bounds from below — so it was a **gap**, and it is
closed. §43.6's row for two lines through a cone's apex argued no half-space can
pick one of two lines that cross; both halves true, conclusion false, and the
answer was one file away.

### 47.6 What was learned about running the work

- **Every lane committed at its halfway point and reported what was left.** No
  lane died holding work, which is what happened last unit.
- **Ownership was per file and it held.** Three lanes stopped at a file they did
  not own and reported instead of editing; one of them was right to, and the file
  it stopped at needed a change the dispatcher then made. One lane necessarily
  edited outside its list — a public signature change breaks its callers — and
  listed every file it touched and why.
- **Two lanes improved on their briefs and said so.** One was told to copy a rule
  into a second door and instead made both doors read one list, because *"the two
  had already drifted, which is the whole defect"*. One was told a defect existed
  in `rcad-geom/src/error.rs` — the dispatcher had the wrong crate — and fixed the
  right file rather than the named one.
- **The brief that produced the best work was the one that said what would count
  as failing.** The spike's rule was written before it ran and said in as many
  words that *"it looks tractable"* is a fail. It passed in five minutes.
- **A lane told to justify a sentence found the sentence wrong more often than a
  lane told to check code found a bug.** Four of the five false claims came from
  the claim gate, not the number gate.

### 47.7 What stands, and what the next unit picks up

**The named next thing**, and it comes with a demonstration rather than a guess:
*"nothing can put points in order along this curve"*, ninety times. Ordering
points along an endless bend is the whole of what stands between a cone and a
block now.

| open | where |
|---|---|
| **ordering along an endless bend** | the ninety refusals above. The next unit |
| **cones are absent from `fuses_that_must_agree.rs`** | commutativity, associativity, idempotence and absorption have **never** run over a cone |
| **the hyperbola's rounding chain is not derived** | the published `64·ε` rests on a 98-rounding **parabola** term count, and every measured worst case is a hyperbola |
| **a flat solid with a two-step face of no area** | passes `SolidBuilder::build` *and* `Model::validate`. Pinned by a test that asserts the gap |
| **two more copies of the closes-on-one-edge list** | `model.rs:591`, `builder.rs:366`. They fail safe today, so drift and not a hole |
| **`a_pair_that_crosses_in_a_conic` is a proxy for "closed conic"** | true for a hyperbola, which is not closed; the refusal arrives late instead of at the table |
| **the fuse drops the seam of a curved wall it never touched** | so `A ∪ B` for disjoint `A`, `B` does not hand `A` back at the topology level. Not new and not cone-specific — a round bar has done it since M2 |
| **the writer** | §46.8 item 3, unchanged. A ball that imports cannot be written out |
| **a tangent plane's single ruling** | still refused: `∇S × ∇Q` is exactly nothing where a plane rests on a cone's own side. The same limit as a plane on a pipe, one more surface pair |

### 47.8 The prediction

§45.8 predicted this unit would lose to a lane's number going stale. It did not,
because the plan forbade the measuring lane from publishing and made the gate
re-derive. **So the prediction for the next unit is about the thing that did get
through:** the six answered fuses were counted, asserted, re-derived and glossed
in a commit message — and the gloss was still false, because nobody looked at
*which* six until a lane was sent to. The next unit will publish a number that is
correct and a sentence about it that is not, and the countermeasure is the one
this unit found by accident: **for every rate that moves, name the cases that
moved and say what they have in common.**

