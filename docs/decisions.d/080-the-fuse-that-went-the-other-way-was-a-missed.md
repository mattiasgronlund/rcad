## 80. The fuse that went the other way was a missed refusal, recorded 2026-09-09

§78.10 left one fuse unattributed: a `Frame` and a `Comb` that answered before
`b4890f1` and are not a solid after it, against `b4890f1`'s stated direction of
fewer wrong answers. It refused to guess which of §16.10's two that refusal is,
and said the guess would need the same work §78.3 did for the thirty.

Seven lanes did that work. The answer is that the item is written the wrong way
round: **`b4890f1` fixed a missed refusal, and what replaced the old answer is a
limit.** The old answer was wrong — two solids touching along a segment that
neither of them recorded — and it survived because the two edges along that
segment were cut in places that did not line up, so the manifold check never saw
the four faces it exists to see (§80.4). Today's
refusal is §16.10's limit: no arithmetic gave up anywhere in it, and the answer
the caller asked for is not one a `Model` can hold. Only that half is one of
§16.10's two. The other half is not a refusal at all, so §16.10's test does not
reach it — §80.1 says why, and declines to stretch the word to cover it.

### 80.1 Both halves of the verdict, and only one of them is §16.10's

§16.9's test, which §16.10 quotes: a refusal is a **limit** when the answer does
not exist, and a **gap** when it does and the arithmetic could not reach it.
§78.10 asked which one today's refusal is. It is worth asking of the old answer
too, and asking both is what turns the item over — but only the first question
is the one §16.10 poses, because the old behaviour was not a refusal.

**The refusal is a limit.** Nothing was approximated on the way to it. The
certified fuse comes back with a `ProvenanceLog` holding **0 records** in every
argument order and both leans — asserted in the tree now, by
`refusal_nothing_was_undecidable_about` in
`crates/rcad-topo/tests/degenerate_fuses.rs`, which fails the test if the log is
not empty. So there is no better arithmetic being waited on. And the answer a
union would have to return does not exist here: four faces run along one
stretch, `Model` means an edge is used by exactly two coedges, and §17.4's first
row settles that a boolean does not get to change what a `Model` is on its way
past. A limit is deferred, not fixed, and §17.7 row 8 already carries **one**
other entry deferred for the same reason: "two cubes touching along part of an
edge", refused `ResultIsNotManifold`. Row 8's other two entries — touching on a
face, and touching at a corner — are answered, and the catalogue's second
`ResultIsNotManifold` is in row 2, "two cubes catty-corner, sharing one whole
edge". Both the table in §17.7 and `catalogue()` in
`crates/rcad-topo/tests/degenerate_fuses.rs` say so.

**What `b4890f1` fixed was a missed refusal.** Before it, this pair fused to two
separate solids that each looked fine on their own and together were not the
union of anything: the contact segment appears twice, once in each solid, as two
edges lying on top of each other with no record of the other. A caller asking for
`A ∪ B` got a `Fused` back and could not tell. The kernel wrote an answer to a
configuration §17.7 refuses, and it wrote it for one of the two shapes that
relation comes in while refusing the other in the same build (§80.4).

**§16.10 does not classify that half, and this section does not stretch it to.**
Its test sorts **refusals** — a refusal is a limit when the answer does not exist
and a gap when it does. The old behaviour was an *answer* where a refusal was
right, which is the mirror image of that test and not a case of it. Nothing is
lost by saying so. What §16.10's dichotomy is *for* is deciding which refusals
§13 rule 3 may defer, and a wrong answer was never deferrable under any rule
here — `CLAUDE.md`'s first principle is that correctness is paramount and a
corrupt solid is worse than a crash. It needs no slot in that partition to be a
defect, and giving it one would leave "gap" meaning nothing narrower than
"wrong".

§78.10's sentence "this one went the other way" is therefore true of the
*outcome* and wrong about the *direction*. It went from a wrong answer to a
refusal, which is the direction §78.8 credits `b4890f1` with; it is only the
not-a-solid tally that moved the unexpected way, because a tally counts refusals
and not mistakes.

### 80.2 What refuses today, and why no tolerance moves it

`Error::ResultIsNotManifold { count: 4 }`, raised at
`crates/rcad-topo/src/fuse/assemble.rs:562` — the only place in `crates/` that
variant is constructed, which is what makes this refusal traceable to one line.
`grep -rn ResultIsNotManifold crates/` names it in plenty of other places and
**none of them builds one**: the variant's own declaration in
`crates/rcad-topo/src/error.rs`, one string in the catalogue's table printer,
the patterns that match it in the topo test files, and doc comments everywhere.
That is the durable claim and the grep re-checks it in a second; **a count of
those lines is not durable, and this section proved it by carrying one that a
single new doc comment falsified.** The falsifying commit was written on a
sibling branch of the one that published the count, so the two were true apart
and false the moment they merged — which is the ordinary case in this repo, not
a mishap.

The stage is skin assembly: `Assembly::join_up_what_is_one_face`
(`assemble.rs:518`) asks `Assembly::darts_along_each_edge` (`assemble.rs:545`)
for the two darts on each edge, and the whole check is

    <[Dart; 2]>::try_from(by.as_slice())
        .map_err(|_| Error::ResultIsNotManifold { count: by.len() })

**There is no number in that for a tolerance to move.** It compares a dart
*count* against two. Lane A ran it under `Policy::tolerant` at `1e-12`, `1e-9`,
`1e-6`, `1e-3`, `1e-1` and `1.0` and got the same refusal every time, which is
what the code says should happen and is worth having been run rather than
argued (§16.10). The same lane reports the refusal is identical in both argument
orders, sheared and unsheared, and that both inputs validate under
`Policy::certified()` before the fuse starts — that last one on the lane's word
only, since nothing in the tree checks it (§80.10). `Assembly::commit` and
`Model::validate` are never reached, so nothing is written — which is why
§78.10 was right that leaving this open cost nothing but attribution.

Two edges of the would-be answer carry four darts each. Unsheared they run
`[0,3,2] → [0,2,2]` and `[0,1,2] → [0,2,2]`, which together are the whole
contact segment (Lane A). The four faces along them are the frame's top cap at
`z = 2`, the frame's left wall, the comb's bottom cap and the comb's right wall
— two coplanar pairs facing opposite ways.

The family harness maps this to `WhatCameBack::NotASolid`
(`crates/rcad-topo/tests/fuses_that_must_agree.rs`), which is how it reaches the
`*_IS_NOT_A_SOLID` bands §78.8 is about.

### 80.3 The pair, worked out on paper

Both shapes come out of the generators, and the first surprise is that the name
lies. `Shape::Comb { teeth: 1, tooth_height: 3 }` is **not toothed**: `comb()`
in `crates/rcad-topo/tests/pairs/mod.rs` computes `span = 2·1 − 1 = 1` and
`top = 1 + 3 = 4`, runs its loop once, and returns `[(0,0), (1,0), (1,4),
(0,4)]` — a plain 1×4 rectangle. `Shape::Frame { width: 5, depth: 3 }` is the
rectangle `[0,5] × [0,3]` with the hole `[1,4] × [1,2]`.

Extruded and placed as §78.10 records them, the frame is `z ∈ [0,2]` at the
origin and the comb is `z ∈ [2,5]` at `[-1, 1, 2]`, so the comb's footprint is
`x ∈ [-1,0]`, `y ∈ [1,5]`. Volumes follow: the frame is
`(5·3 − 3·1) · 2 = 24` and the comb is `1·4·3 = 12`. The shear is unimodular,
so neither changes.

**They share no volume at all.** The footprints meet only at `x = 0`, the
heights only at `z = 2`, the depths on `y ∈ [1,3]` — so the intersection is the
segment `x = 0, 1 ≤ y ≤ 3, z = 2` and nothing more. The volumes and the segment
are arithmetic anyone can redo, and the test in the tree asserts both ends of it.
Lane B checked the segment is not an artefact of how the region is written down:
exact membership stepped by ±1/8 about its midpoint gives two opposite closed
quadrants meeting along a line, both regular closed, so
`closure(interior(A ∪ B)) = A ∪ B` and no regularization argument deletes the
contact.

**The shear decides nothing here, and that is checkable rather than assumed.**
`x_per_y: -1` has determinant 1 and integer entries, so every corner stays whole,
and it fixes the direction `(1,0)`, so the y-normal faces and the z-caps keep
exact unit normals. The two touching walls do not — `x = 0` becomes `x + y = 0`,
whose unit normal `(1,1,0)/√2` is not an `f64`. But ON is settled by
`Plane::agreement_with` on the coefficient 4-vectors (§17.4), and Lane B reports
the two walls come out as `1x + 1y = 0` and `4x + 4y = 0`, exactly proportional,
so all six minors are exactly zero. The unsheared twin is axis-aligned
throughout and refuses identically, which is the demonstration rather than the
argument: the test runs both leans.

### 80.4 Why this overlap walked past a check the other one did not

This is the mechanism, and it is the part §78.10 said needed working out.

§17.7 row 8 already has a case of exactly this relation:
`two_cubes_that_only_touch_along_part_of_an_edge` in
`crates/rcad-topo/tests/degenerate_fuses.rs`, asserting `count: 4`. Lane E ran
it at `c66bdca` and at `cc959c1`, and **it passes at both**. So one build
refused row 8 for the cubes and answered it for the frame and the comb, off one
catalogue row, and the difference is the shape of the overlap.

In the cube pair the small cube's edge lies **wholly inside** the big cube's. A
vertex therefore lands in the interior of the big edge, that edge splits, and
the piece left over matches the small cube's edge end to end. One edge, four
faces, refused.

In the frame and the comb **neither edge contains the other**. The frame's edge
at `x = 0, z = 2` runs `y ∈ [0,3]`; the comb's runs `y ∈ [1,5]`. They overlap on
`y ∈ [1,3]` and each sticks out past the other, so the two have to be cut into
pieces that line up before four faces can share one edge. At `c66bdca` they were
cut — and not into pieces that line up. Lane K read the arrangement out of an
instrumented copy of that commit: 28 faces, and the line `x = 0, z = 2` carrying
five edges with two faces each. The frame's are `(0,0,2) → (0,1,2)` and
`(0,1,2) → (0,3,2)`; the comb's are `(0,1,2) → (0,2,2)`, `(0,3,2) → (0,2,2)` and
`(0,3,2) → (0,5,2)`. So the frame ran `[1,3]` in one piece against the comb's
`[1,2]` and `[2,3]`. One edge against two is two ordinary edges with two darts
each: the two shells never met in the adjacency walk, and the manifold check
answered correctly about the wrong question.

**The cut that was missing is at `y = 2`, and the arithmetic says so before any
instrument does.** The frame is `[0,5] × [0,3]` with the hole `[1,4] × [1,2]`
(§80.3), so its own planes in `y` are `0`, `1`, `2` and `3`, and the comb's are
`1` and `5`. Cutting a face by the *other* solid's planes was never off, so the
comb's wall at `y = 1` always cut the frame's top cap and the frame's outer wall
at `y = 3` always cut the comb's — which is why those two splits are already
there at `c66bdca`, and why neither of them can be the one `b4890f1` supplies.
`y = 2` is the far wall of the frame's **own** hole, and nothing of the comb
lies on it: the comb's planes are `z = 2`, `z = 5`, `y = 1`, `y = 5`, `x = 0`
and `x = -1`. That leaves `y = 0`, the frame's other own-only plane, and it
cuts nothing new because it is an endpoint of the frame's own edge rather than
a point inside it. So `y = 2` is the only plane on this line whose cut
other-solid cutting could not already have supplied — all four of the frame's
`y` planes accounted for, which is what §16.10 asks of a claim shaped like
this one.

`b4890f1` — "cuts: a solid's own planes cut its own faces" — is what supplies
it. At `b4890f1` and at today's `c4f29af` the picture is identical and it is not
the one above: 46 faces, the frame's top cap and its wall at `x = 0` carrying
twelve and ten distinct cut planes where they carried the comb's four, and among
them the new one that matters, `0,-1,0|-2`. The contact then carries
`(0,1,2) → (0,2,2)` and `(0,3,2) → (0,2,2)` with **four** faces each — the
frame's `z = 2` and `x = 0` against the comb's — and it refuses like the cubes
do. **Two edges, not one.** `count: 4` in the message is the dart count of the
first edge reached and says nothing about how many there are; §80.2's two
four-dart edges are these two.

**And the loop bound is what does it, shown on today's tree rather than argued
from an old one.** `git show b4890f1 -- crates/rcad-topo/src/fuse/cuts.rs`
changes **two** loop bounds, and both are needed. The outer
`for other in (one + 1)..solids.len()` becomes `for other in one..solids.len()`,
which lets a solid meet itself; the inner `for theirs in 0..` becomes
`for theirs in from..`, with `let from = if one == other { mine + 1 } else { 0 }`
above it, which stops a face being cut against itself and stops each pair inside
one solid arriving twice. The `let from` is dead without the inner change, so a
reader given only the outer one has to work the rest out. Put both behind an
environment switch in a copy of `c4f29af` and turn own-solid cuts off, and
today's tree does not merely answer again — **it builds `c66bdca`'s arrangement
exactly.** Lane K diffed the whole dumped block rather than the lines it cared
about: all 63 lines equal, edge for edge, same edge indices, both endpoint
fields and both face planes on every one. The 28 faces are the same 28, and
`x = 0, z = 2` carries the same five edges listed above. Two loop bounds, both
directions, on current code — and in the configuration Lane K probed, unsheared
and one argument order, nothing else that landed since `c66bdca` moves this
arrangement at all. §80.10 has the method and what it did not establish.

That is why §78.8's aggregate and this case are not in tension. `b4890f1` did
not make the kernel refuse more; it made it *notice* more, and one of the things
it noticed was a contact that had been invisible in one of the two shapes it
comes in.

### 80.5 What the old answer was, and why `validate` could not see it

Lane E's measurement, at `c66bdca` and at `cc959c1`, in all four cells
(sheared/unsheared × either argument order). The union returned **two solids**:

| | faces | edges | vertices | volume |
|---|---|---|---|---|
| the frame | 10 | 24 | 16 | 24 |
| the comb | 6 | 12 | 8 | 12 |

Every edge had exactly two coedges. The total volume was `36.000000000`, which
is the true volume of the union. `Model::validate(&Policy::certified(), …)`
returned `Ok` with an empty provenance log.

Those counts are the inputs' own. **Nothing survives in the answer of any cut or
any merge, and no edge in either solid records the other.** The contact appears
as two distinct unsplit edges lying on top of each other — the comb's
`(0,1,2) → (0,5,2)` and the frame's `(0,0,2) → (0,3,2)`. That is the *answer*,
after `join_up_what_is_one_face` has folded the pieces back and dropped the
corners that stopped being corners: the arrangement it was built from did carry
cuts, five edges' worth on this line, and §80.4 is where that matters. Reading
this table as the arrangement is the one way to misread it.
Each solid is locally manifold, so §4.3's rule
that validity checks are local to touched entities is exactly what stopped
`validate` from having an opinion: there is no entity you can point at that is
wrong. The wrongness is between two solids that share a `Model` and do not know
it.

A caller could not tell from the numbers either. The volume is right. The face
counts are right for each shape. The only symptom is the solid count, and "two"
is a legitimate answer for a union whose arguments do not touch.

### 80.6 The eleven that went the other way, checked against truth

`b4890f1` turned **eleven fuse-instances — seven distinct configurations** —
from not-a-solid into answers, and until Lane G nobody had checked whether those
answers were right. §78.8's sentence "fewer answers are not solids because fewer
answers are wrong" was the stated direction and not a measured one.

G measured it without asking the kernel. The shear is unimodular and applied to
both solids equally, so the boolean of the sheared footprints is the image of
the unsheared one, with the same volume, piece count and cavity count; the
shapes sit on the unit lattice, so every true volume is an exact integer. All
eleven match that ground truth on volume, surface area, centroid, solid count
and shell count, and `Model::validate` under a certified policy accepted every
one. Two of the eleven are worth naming because they are the ones a sloppy
implementation gets wrong: one is an intersection whose true answer is two
face-disconnected lumps of volume 2 and 1, and the kernel returns exactly those
two; and four are unions of inputs sharing zero volume, where merging into one
solid is right only because the contact is two-dimensional — three shared unit
squares face-on in each.

**The messages split cleanly, and this is sharper than the aggregate.** All
eleven that vanished said "an edge would have **1** faces along it" — a dangling
face. The six that remain and the one that arrived say **4**. So `b4890f1`
removed one symptom class outright and left the other untouched. §78.9 reports
the fall as per-family factors between 0.375 and 0.85, which is what an
aggregate can say; what actually happened is not a proportional thinning at all.

Neither §78.8 nor §78.9 is wrong. Both are reading a rate, and neither claims
anything per case. This section is the per-case reading they left for someone
else, and it says: eleven answers checked against exact truth and all correct,
one wrong answer replaced by a refusal, and no case anywhere in the audit that
went from right to wrong.

### 80.7 Seven instances, six configurations, and one seed

Two things about the number 7 that a later reader would otherwise re-derive.

**Seven fuse-instances are six distinct configurations**, because the strategy
draws one configuration twice (Lane D). All seven fail with the same text, "an
edge would have 4 faces along it"; all seven pair a `Frame` with something else;
six are `Union` and one is `Difference`.

**And the seven belongs to the shipped seed, not to the family.** Re-derived
here, three runs of `how_often_a_certified_fuse_of_a_pair_leaning_the_same_way_refuses`:

| seed | fuses | refused | not a solid |
|---|---|---|---|
| shipped | 429 | 24 (5.6%) | **7** (1.6%) |
| `RCAD_SOAK_SEED=42` | 429 | 36 (8.4%) | **8** (1.9%) |
| `RCAD_SOAK_SEED=999` | 429 | 30 (7.0%) | **7** (1.6%) |

What holds across seeds is the band, `A_PAIR_LEANING_ALIKE_IS_NOT_A_SOLID:
0.011..=0.049` (`crates/rcad-topo/tests/fuses_that_must_agree.rs:483`), which all
three of those rates sit inside. §78.8 spends a subsection on where a count sits
inside its band; the corollary here is that "the 7" is not a property of the
kernel to be attributed at all — six of it are, one for each configuration, and
the seventh is the strategy drawing one of them again.

### 80.8 `cc959c1` names a sibling, not the commit before

§78.10 says the pair "answered at `cc959c1`". It did — Lane E ran it there — but
`cc959c1` is not the commit before `b4890f1`, and a later reader diffing the two
will not find the change. Re-derived here:

    git rev-parse b4890f1^          → c66bdca
    git merge-base cc959c1 b4890f1  → c66bdca
    git rev-list --count cc959c1..b4890f1 → 1
    git rev-list --count b4890f1..cc959c1 → 9
    git merge-base --is-ancestor cc959c1 b4890f1 → false

So the two are **siblings off `c66bdca`**, nine commits apart the other way. The
`1` that makes `cc959c1..b4890f1` look adjacent is a set difference and not an
adjacency. The "before" witness is **`c66bdca`**, and it behaves the same as
`cc959c1` on this pair, so §78.10's sentence costs nothing and is left standing
with this correction beside it.

Lane C found the boundary by `git bisect start f95bc32 cc959c1`, which converged
in 9 steps with 0 skips on `b4890f1`, and then confirmed it directly rather than
by bisect's inference: `c66bdca` answers, `b4890f1` refuses. C spot-probed four
more commits across the untested half of the range plus HEAD — 14 direct probes
in all, one flip, no reversal anywhere sampled.

### 80.9 Two touching solids is a possibility with no consumer

This is the one thing left open, and it is left open deliberately.

`c66bdca` shows the kernel **can** deliver this union as two touching solids
carrying exactly the right total volume. It never chose to. It arrived there by
failing to notice a contact, in one of the two shapes that contact comes in, and
the same build refused the other shape. So the old behaviour is not a design
anybody could keep — it was inconsistent with itself off one catalogue row.

But it does raise a question the catalogue has never been asked. §17.4's first
row settles that a non-manifold *result* is refused, and §17.7 row 8 records the
one refusal that follows. Whether "two touching solids" should ever be an
**answer** for row 8 — a `Fused` holding two solids whose boundaries meet on a
set of measure zero, each individually manifold, and both aware of the contact
— is a different question from the one those rows settle. It would change
§17.4's first row, it would change §17.7's table, and it would have to cover
the cube pair too, because half an answer for one row-8 shape and a refusal for
the other is exactly the inconsistency `b4890f1` removed.

**It is not decided here, and nothing in this section should be read as
proposing it.** It has no named consumer (§13 rule 3), and §16.10's refinement of
that rule permits deferring a *feature* while forbidding the deferral of a
*gap* — this is the first of the two, because the refusal it would replace is a
limit and not a gap (§80.1).

**Asked and answered, 2026-09-09: the refusal stands.** Mattias's decision is
that the possibility is recorded and not built — it has no named consumer, and
§13 rule 3 says a part with no named consumer is not built. It is reserved
rather than closed: if a consumer ever appears, deciding it is a unit of its
own, and that unit inherits the whole of row 8, the cube pair included. What
this section owes a later reader is that the possibility exists and where the
evidence for it is: §80.5's table.

### 80.10 What this section re-derived, and what it took on a lane's word

§78.11's habit, kept.

**Re-derived from this tree while writing this section**, and re-derived by the
suite on every `just gate` — the volumes 24 and 12, the edge extents `y ∈ [0,3]`
and `y ∈ [1,5]`, the empty intersection, `ResultIsNotManifold { count: 4 }` in
both argument orders and both leans, and the empty provenance log. Lane F's
`two_solids_whose_touching_edges_each_run_past_the_other_cannot_be_united`
asserts all of them, built out of the generators' own `Shape`s so it is §78.10's
configuration and not a transcription of it. That test is the answer to §78.11's
complaint about itself: this pair's figures are not one lane's measurement on
one day.

**Re-derived here by reading or by running, and cheap to redo** — the commit
topology in §80.8; `assemble.rs:562` being the only construction site of the
variant; `comb(1, 3)` being a 1×4 rectangle and `Frame { 5, 3 }` its rings; the
volumes and the contact segment as arithmetic; the three seed rows in §80.7;
and two more that belong here rather than in the column below.

**§80.1's reading of row 8** — three entries, one of them refused, the
catalogue's other `ResultIsNotManifold` in row 2 — off §17.7's table and
`catalogue()` in `crates/rcad-topo/tests/degenerate_fuses.rs`. It is written
down because a miscount of exactly this row is the one thing this section
already got wrong once, and the two records that settle it disagree with
nothing.

**The `b4890f1` diff** — the two loop bounds, and the 96-and-37 that says the
rest of the file's change is prose and tests. `git show b4890f1 --
crates/rcad-topo/src/fuse/cuts.rs` is five seconds from this tree, so it is not
anybody's word. Only the *run* of the counterfactual below is.

**Taken on a lane's word, needing instrumentation that is not checked in or a
build of an archived commit:**

- The tolerance sweep at six `Policy::tolerant` values, and the four-dart edges
  `[0,3,2] → [0,2,2]` and `[0,1,2] → [0,2,2]` (Lane A).
- That both inputs validate under `Policy::certified()` before the fuse starts
  (Lane A, §80.2). **Nothing in the tree checks it**, and this bullet is here
  because the claim read as re-derived until somebody looked:
  `Configuration::build` and `Placed::built_after` in
  `crates/rcad-topo/tests/pairs/mod.rs` never call `Model::validate`, and
  neither does Lane F's test. The two `validate` calls in `degenerate_fuses.rs`
  are on other pairs and on the *answer*, not the inputs.
- The plane coefficients `1x + 1y = 0` and `4x + 4y = 0`, and the ±1/8
  membership sampling about the segment's midpoint (Lane B).
- The bisect: 9 steps, 0 skips, 14 direct probes (Lane C).
- `423 fuses, 24 refused, 17 not a solid` at `c66bdca`; the seven being six
  configurations drawn seven times; six `Union` and one `Difference` (Lane D).
- Everything in §80.5 — the two-solid answer, its counts and volumes, the two
  unsplit coincident edges, and `two_cubes_that_only_touch_along_part_of_an_edge`
  passing at `c66bdca` and `cc959c1` (Lane E).
- The whole of §80.6: eleven instances, seven configurations, the ground-truth
  comparison, and the 1-versus-4 split in the messages (Lane G).
- The whole of §80.4's mechanism — the arrangement at `c66bdca` and at
  `b4890f1`, the cut lists, and `y = 2` being the plane that arrives (Lane K).
  This one is a **demonstration** and not a report, but the instrumentation that
  makes it one is not in the tree, so it belongs in this column. The method:
  copy the worktree (`rsync -a --exclude target --exclude .git --exclude
  corpus`), `git archive` `c66bdca` and `b4890f1` into two more, and instrument
  **only those copies** in two places — the end of `gather` in
  `crates/rcad-topo/src/fuse/cuts.rs`, printing each baked face's plane and its
  distinct cut planes, and `darts_along_each_edge` in `assemble.rs`, printing
  every edge's endpoints and the planes of the faces claiming it, before the
  `ResultIsNotManifold` map. The probe is Lane C's minimal one.
- **The counterfactual was run on current code**, which is what makes §80.4 a
  demonstration and not a story about an archived commit (Lane K). The whole of
  `b4890f1`'s **executable** change to `cuts.rs` is two loop bounds — the file's
  diff is 96 insertions and 37 deletions, and the rest of it is the module
  header, `gather`'s doc comment, five rewritten unit tests and two new test
  helpers. Behind an environment switch in a copy of `c4f29af`, turning
  own-solid cuts off makes today's tree answer, drops the arrangement from 46
  faces to 28 and **reproduces `c66bdca`'s arrangement
  exactly** — all 63 lines of the dumped block equal, edge for edge, same edge
  indices, both endpoint fields and both face planes on every one. The probe
  prints four blocks and all four match: the two input solids at 10 and 6 faces,
  the arrangement at 28, and the assembly after joining at 16.

      LANE_K_DUMP=1 LANE_K_NO_OWN_CUTS=1 mise exec -- \
        cargo test -p rcad-topo --test lane_k_probe -- --nocapture

Two things Lane K did not establish, so nothing above should be read as saying
them. It ran the **unsheared** configuration only, in one argument order, taking
§78.10's word that the shear makes no difference and Lane F's test for the
orders. And it did not chase why the `y = 2` corner survives on the frame side
once it exists, noting only that the refusal fires straight out of the
arrangement, before any joining.

**The two stages are tied together by measurement, and it is worth keeping them
apart in the reading.** An earlier listing of "the contact edges" as the two
unsplit ones `(0,0,2) → (0,3,2)` and `(0,1,2) → (0,5,2)` was the post-join
block, not the arrangement: the label was wrong and the numbers were not.
Joining folds the frame's `[0,1]` and `[1,3]` into one edge and the comb's
`[1,2]`, `[2,3]` and `[3,5]` into one, dropping the corners at `y = 1`, `2` and
`3` because they stopped being corners — and what is left is §80.5's committed
answer. Five edges before, two after, **at `c66bdca` and at own-cuts-off
`c4f29af`** — the two trees that answer. At `b4890f1` and at today's tree as it
stands there is no "after" at all: the refusal comes out of
`darts_along_each_edge` before anything is joined. That is the distinction §80.5
warns about, now measured rather than inferred.

Lane E's and Lane G's are the ones worth re-running if this is ever reopened,
because §80.1's verdict rests on them: §80.5 is why the old answer was wrong,
and §80.6 is why `b4890f1` is not merely trading one wrong answer for another.

---

