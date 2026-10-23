## 165. Two valid solids the corners could not tell apart, and the runs that now do — §163.4 answered, 2026-09-19

*§163.4 left open whether two whole valid answers that differ can write out the
same in the fuse oracle, because the written-out form records corners and not
what the edges between them run along. **They can**, and two pairs show it, both
built with `SolidBuilder`, both passing `Model::validate`, both run rather than
reasoned: a quarter of a ball and three quarters of it, and the lens where two
balls overlap and the two balls together. Each step of a wire now carries its
edge's run, and a closed edge the way it is walked; both pairs are told apart,
and none of the suite's comparisons of real answers splits. Neither pair has
been reached through the fuse. Building the first one also turned up a mesher
defect, reported here and not fixed (§165.4).*

### 165.1 The quarter and the three quarters

A unit ball cut by the planes `x = 0` and `y = 0`. The quarter is where `x ≥ 0`
**and** `y ≥ 0`; the three quarters are where `x ≥ 0` **or** `y ≥ 0`. Face by face:

| face | quarter | three quarters |
|---|---|---|
| sphere, used forwards | between the half-meridians at `+x` and `+y` | between those at `−y` and `−x` |
| half-disc on `x = 0` | the `y ≥ 0` half, facing `−x` | the `y ≤ 0` half, facing `−x` |
| half-disc on `y = 0` | the `x ≥ 0` half, facing `−y` | the `x ≤ 0` half, facing `−y` |

Both solids lie on the `x ≥ 0` side of `x = 0` where they touch it, so both flat
faces face the same way, and the plane's coefficients are identical. Every wire
is a ring of two corners, the poles `(0, 0, ±1)`, so the smallest rotation reads
it the same walked either way. Only the edges tell them apart: each half-meridian
is an `EdgeRun::PartOfWhereItCrosses` whose picker faces `+x`, `+y`, `−x` or `−y`.

`a_quarter_of_a_ball_and_three_quarters_of_it_are_two_answers` builds both in one
model, validates them, and asserts the two write out differently. Against the form
§163 left, it failed with the two answers written out identically:

```text
assertion `left != right` failed: a quarter of a ball and three quarters of it wrote out as the same answer
  left: [[[0, 0, 13830554455654793216, 0, 0, 0, 1, 6, 0, 0, 4607182418800017408, 0, 0, 13830554455654793216], [0, 13830554455654793216, 0, 0, 0, 0, 1, 6, 0, 0, 4607182418800017408, 0, 0, 13830554455654793216], [1, 0, 0, 0, 4607182418800017408, 0, 1, 6, 0, 0, 4607182418800017408, 0, 0, 13830554455654793216]]]
 right: (the same)
```

This is the answer-level version of §163.4's two ball caps. What §163.4 said
stopped the caps — a flat face beside them whose stored plane gives the side
away — does not stop this pair, because both flat faces here face the same way.

### 165.2 The lens and the union

Two balls of radius 5 about the origin and `(0, 0, 6)` cross in a circle of
radius 4 at `z = 3`, with a whole-number corner at `(4, 0, 3)`. Built as two
sphere faces, each used forwards, joined along one closed edge
(`EdgeRun::RightRoundWhereItsFacesCross`), there is exactly one choice left: which
of the two faces walks the circle forwards. One choice is the lens, the other is
the union. Measured at a deflection of `0.01`:

| first face walks the circle | measured volume | worked out by hand |
|---|---|---|
| forwards | 108.73 ± 1.24 | the lens, 108.91 |
| backwards | 937.47 ± 5.03 | the union, 938.29 |

Every face has one corner, `(4, 0, 3)`, on a round wire, so the corners are
identical. `where_two_balls_overlap_and_the_two_balls_together_are_two_answers`
checks both volumes against those two numbers and asserts the answers write out
differently. With the direction taken out of the written form, it fails with
two identical vectors.

`Model::validate` accepts **both** orientations, and it is right to. The closed-edge
check in `validate.rs` reads which side of the partner a face's material is on
from a corner of the face that is off the partner. A cap's only corner is on the
circle, so the check is skipped, as its own comment says. §163.4 called this
"the closed-edge direction `validate` already enforces". That is true only where
the check can run. What makes the direction canonical is what "forwards" means
for a closed edge: along the curved surface's own longitude, which is the mesher's
order and the fuse's rule (`a_closed_crossing_is_walked_the_way_it_is_stored`).
The mesher reads it that way, and that is why the two measured volumes differ.

### 165.3 What the written form holds now, and that it splits nothing

Each wire is a ring of **steps**: a step is the corner a coedge starts at,
followed by its run, and the smallest rotation turns whole steps.

- `Straight`: a tag and nothing else.
- `RightRound`: a tag and whether the wire walks the edge forwards. That is the
  edge's orientation as the view composes it, so a face used reversed counts.
- `PartOf`: a tag, the `and` surface in the same numbers a face's surface uses,
  and the pickers written out and **sorted**, because a piece is on or inside
  every picker and the list's order means nothing.

`Answer::moved_by` moves the runs too: a surface moves as a face's surface does,
and a flat picker `n · p ≥ d` becomes `n' · p ≥ d + n' · t`. Every step
encoding starts with a tag that fixes its length, so two different rings cannot
write out the same numbers.

Checked by taking each piece out and running
`just t -p rcad-topo --test merged -E 'test(/^fuses_that_must_agree::/)' --no-fail-fast`, as §163.3 did:

| taken out | tests that fail |
|---|---|
| the run, from every step | the quarter pair, the lens pair, `a_ring_of_two_corners_walked_the_other_way_is_another_ring` |
| a closed edge's direction | the lens pair |
| the picker sort | `a_run_picked_by_the_same_two_half_spaces_in_either_order_is_one_run` |

**Nothing splits, and that is not an empty claim.** All 54 tests pass with the
runs in, and the properties comparing real fuse answers do see curved runs.
Making the reader panic on each kind in turn shows it:

- **Closed edges** reach seven real-answer comparisons: swapping and shifting a
  block and a ball, and a block and a bar; swapping a block and a frustum;
  swapping a curved pair; and the frustum on the wider grid. None of them splits
  on which way a closed edge is walked. So the fuse follows the longitude rule
  both ways round and under a whole-number move.
- **Part-of runs** reach one real-answer comparison, the ball and its copy
  (`a_ball_and_a_copy_of_it_unite_share_and_cancel_the_way_a_solid_should`),
  where the ball's seam survives. That is thin. If a family that cuts a ball's
  seam, or crosses two curved surfaces in pieces, ever splits on the pickers,
  look there first. The picker is how the kernel stores a piece, not a canonical
  description of the curve, and two runs of the fuse could name one piece with
  two pickers. No run has been seen doing so.

§163.4's pin, `a_ring_of_two_corners_reads_the_same_walked_either_way`, is gone:
what it pinned is no longer true of a ring whose two edges differ. It is replaced
by `a_ring_of_two_corners_walked_the_other_way_is_another_ring`.

### 165.4 What was not shown, and a mesher defect found on the way

**Neither pair was reached through the fuse.** The kernel's ball has its poles on
`z` and its seam in `y = 0` on the `x ≥ 0` side (`Model::add_ball`). Four fuses of
a unit ball against a block whose vertical edge runs through both poles were run:
∩ with the `x ≤ 0, y ≥ 0` block, \ with the `x ≥ 0, y ≤ 0` block, ∩ with the
`x ≤ 0, y ≤ 0` block, and \ with the `x ≥ 0, y ≥ 0` block. All four came back
`Error::Undecidable` from "a point inside a piece of a face", with the two poles
as inputs and an unbounded enclosure. The rest of this paragraph is **reasoned,
not run**. Placing the ball so that its poles are off the cut should not help.
The seam and the poles then fall inside one of the two complementary sphere faces
and not the other, so they become corners one answer has and the other does not.
The same holds for the lens and the union of two kernel balls: each ball's poles
lie on opposite caps in the two answers. So for pairs built by complementing a
region of a kernel ball, the seam is what should keep the corners apart. Whether
some other fuse-reachable pair is confused by corners alone is not shown in
either direction. The written form now covers both kinds of pair built here
either way.

**The mesher gets the quarter wrong, and `validate` accepts it.** A quarter of
a unit ball, built as in §165.1, measures volume `5.209 ± 0.158` and area
`18.80`, where the true values are `π/3 ≈ 1.047` and `2π ≈ 6.283`. The sphere
face alone is meshed at about `5π`, which is more than the whole ball's skin: a
whole turn too many. It comes out the same with the quarter at longitudes
`[0°, 90°]`, `[−45°, 45°]` and `[135°, 225°]`. The three quarters measure right
every time (`3.141 ± 0.095`, area `12.59` against `4π ≈ 12.57`). `classify_point`,
asked at five points with the quarter at `[0°, 90°]`, refuses on both solids
with `CannotFollowAFacesTrimming`, which is a named refusal and not a wrong
answer. So a model that passes `Model::validate` gets a confidently wrong
volume. Either the mesher unwraps a lune's pole steps a turn too far, or
`validate` should refuse a wedge built this way. **Kernel source was out of this
lane's scope, so this is reported and not fixed.** `a_wedge_of_a_ball` in the
oracle file builds the shape for whoever takes it.

Write set: `crates/rcad-topo/tests/fuses_that_must_agree.rs` (the written-out
form, `Step` / `Run` / `Picking`, four tests replacing one, and the two builders),
this section, and a pointer in §163.4.

**Answered by §166**: the mesher, not `validate` — a lune's pole side was charged
a whole turn, and is now charged the angle between its two meridians.

---
