## 206. The cone cap's bulge, the pole cut's sliver, the closed-run limit, and a miss test that ignored `y` — 2026-09-22

*Three lanes off `9405cb32`, each verified by a Sonnet lane that re-derived its
claims from the branch. Where a number below was measured by a lane and not by
me, it says so. The four counts in 206.5, the row table, the `√2` argument and
every test result are from runs made here on the merged tree.*

### 206.1 §200.3's pair was neither of §204.3's two candidates

§204.2 called the refusal a **gap**: the solids share volume, so every rule has
an answer to give. §204.3 named two fixes and took neither — the turning-point
box learning which arc a curved side keeps, or the point test learning to follow
the frustum's cap outline.

It was a third thing, in `fuse/select.rs`. The cone's big cap, cut by a corner-up
cube's plane, leaves pieces bounded by an arc of the rim and the chord across it.
That is two corners, so the corner polygon winds 0, and `in_the_bulge_of`
returned 0 for a cone **by name**. Every candidate came back `Some(false)` and
`the_piece_holds` had nothing to settle against.

The fix counts the bulge for a cone whose crossing with the piece's plane is a
circle — the plane square to the axis, an exact one-sided test. §200.3's pair
goes from 0 right / 8 refused to **4 right / 4 refused**; the two neighbouring
families go 16 right to 24 and 16 to 20, and every *"a point inside a piece of a
face"* refusal in the nappe file is gone.

**The box was not involved at all**, and that is the demonstration §16.10 asks
for rather than a paragraph: the refusing candidates were all on the cap plane
`z = 8` and were rejected by `the_piece_holds` before any ray was fired, so no
box and no ray target took part. §200.3's *second* fault, the pointed cone's
`classify_point` answering `Outside` inside itself, was already fixed on the
base — `a_pointed_cone_knows_its_inside.rs` pins 991 inside / 29634 outside /
0 wrong / 0 refused over the grid.

§204.3's contested figure **reproduces exactly**: 16040 `CannotFollowAFacesTrimming`
on `Plane(normal (0,0,1), offset 8.0)`, the frustum's big cap. Lane's measurement,
by a temporary per-variant tally in `fuse_and_check`.

### 206.2 The floor is bounded by `holds.rs`, not by the box

The nappe file's floor went 8 000 → **8 800**; the smallest `decided` count over
the file's 104 answers is 8 879 and **did not move with the fix at all**.

So whatever §204.3's two candidates are about, the floor is not it. The cap is a
flat face whose one curved side is an arc of the cone's rim, and
`what_a_face_cut_by_part_of_a_crossing_holds`'s `is_a_family_this_follows`
admits *"a flat face whose curved sides are arcs of a ball it lies inside"* and
nothing else. Teaching it a cone's rim arc is what raises the floor, and the
family argument has to be re-made for a cone — the region inside a cone on a
plane square to its axis is one disc. Not attempted; the lane ran out of budget.

### 206.3 The pole cut lost 72 answers, not 9, and all 72 are one gap

§202.2's "9" is a **net loss against a pre-§202 base**, not the number
outstanding. What the base test actually pinned was **72** refusals: 36 by
intersection (30 `TwoPatchesOfTheMeshDrewTheSameSide`, 6 `VolumeIsNotPositive`)
and 36 by difference (18 and 18), the same (lean, turn) pattern at all three
radii, every one at lean 0.3 or 0.7 and none at 1.1. Re-deriving the figure
rather than working to it is what turned this up.

All 72 are a gap. A coarse walk gives the sliver's rim four corners, one of them
the pole. The piece spans 1.6165 rad, past the quarter turn a patch may span, so
`cut_in_two` cuts it from the pole to the corner opposite — and `how_many_steps`
answered 1, so the cut carried no point and was a bare side between two points
the model already holds. The flat face closing the sliver drew that same pair as
its diagonal, and the side ended up used by four triangles. Where the flat face
picked the other diagonal, the curved face had no point off its own rim's plane,
the solid meshed flat, and §41.6's retry could not rescue the zero because every
finer mesh was refused.

The fix: **a cut across a piece that is still one whole closed curve takes at
least two steps.** The sliver now answers 126 of 126, where it answered 54.

Narrowed to that case deliberately. Charging every cut a point makes a moved
round bar's volume slip 3.6e-4 where it slips 1.8e-13 today, which
`a_solid_with_a_torus_face.rs` pins. **The narrowing is a cost bound and not a
proof**: nothing shows a bare cut across a piece bounded by more than one closed
curve can never meet a neighbour's diagonal, only that no solid the gate builds
has shown one. `cut_in_two`'s doc comment says so out loud.

### 206.4 §204.1's limit holds for the family we can reach, and two families are unknown

The nearest way past §204.1's *"only runs whose pieces all come from one input
edge"* is a closed edge shared by **both** solids, cut at each solid's own
vertex: every piece then carries two input edges, the join keeps no corner, and
`assemble.rs:1081` panics. Reaching it needs both cut points seam-free, and a
pointed cone's side is the only face here with a closed wire and no seam — so
both solids must be pointed cones on one circle. Built by hand through
`SolidBuilder`, every rule both ways round is refused
`ACrossingBetweenAConeAndAnotherCurvedSurface` before the fuse assembles
anything, and the sphere-for-cone variant is the existing dome-on-a-shaft cell,
0 of 7116 touching-row fuses answered. **That family is a limit**, and the test
is the pin.

Two families are **not** settled, and the new test's doc comment says so rather
than hiding it: a run whose pieces come from two *different* input edges, and a
run of a fresh crossing curve carrying no input edge. Both would panic at
`assemble.rs:1081` today, so neither is theoretical safety — they are unknowns.
`assemble.rs` is unchanged, because the case to make it handle could not be
built.

Worth carrying forward: a ball is one face, one seam edge and two poles, so a
pole's `leaving` list is that one seam edge written down twice and the pole stays
a corner. The "ball's meridian of two different edges" in `f108acbe`'s message is
something the arrangement makes *after* cutting, not a structure the input has.

**The soak nobody had run could not have been run.** `fuses_that_touch.rs` had
`CASES` as a `const` and its seed fixed; it now reads `RCAD_SOAK_FACTOR` and
`RCAD_SOAK_SEED`. The factor-100 soak is green and wrote no new seed: 16 proptest
cells, 2402 shapes and 14412 fuses per cell, 38432 shapes and 230592 fuses. The
lane timed it at 362.955s and its verifier at 324.523s on a machine running three
other lanes; neither is a clean figure.

### 206.5 A miss test that never looked at `y`, and three counts re-derived

Merging 206.1 moved four pinned counts in files that lane could not write. Three
are re-derivations. The fourth was not a count problem at all.

`what_the_six_answered_block_fuses_actually_say.rs` asserted that no fuse of a
**meeting** block and frustum may answer unchecked, and nine began to answer. Its
own message read that as *"the arrangement learning to order points along a
hyperbola, which is good news"*. It is nothing of the kind. Its
`the_block_misses_the_frustum` compared the block's nearest `x` against the
frustum's widest radius and never looked at `y`, so it read `slope · far = 1` as
touching.

The block's low corner is at `(1, 1, near + 1)` and it only runs further out in
both `x` and `y`, so no point of it is nearer the axis than `√2`; the frustum is
about that axis with its apex at the origin and never passes `slope · far`. So
`slope · far < √2` is **exact** for disjointness, needing no `z` reasoning — and
the three rows at `slope · far = 1` are disjoint for a reason the old test could
not see. Five of the sixteen rows clear the frustum, not one.

Re-pinning the count would have recorded the opposite of what happened. The
file's headline claim survives intact — *no block in this family has ever taken a
piece out of a frustum* — and the file and its test are renamed off "six", which
is now fifteen.

The other three, all from `just t --no-fail-fast` on the merged tree:

| Where | Was | Is |
|---|---|---|
| `fuses_that_must_agree.rs` comparisons made | 106 | 110 |
| …of 336 fuses, refused / unholdable | 124 / 0 | 108 / 8 |
| `how_often_a_fuse_with_a_cone_refuses.rs` answered | 6 of 96 | 15 of 96 |
| …inside a piece / corner too far / boundary crosses / four faces | 60 / 14 / 7 / 9 | 30 / 18 / 9 / 24 |
| `ordering_points_along_an_endless_bend.rs` answered | 53 | 53 |
| …refusals, by kind | 31 / 0 | 27 / 4 |

Every comparison that ran still agreed, either side. The answered floor moves
6 → 15 rather than staying where it was, per §19.1.

**The three rises have one cause and it is not a regression.** Fuses that used to
stop at `select.rs` now get far enough to reach `assemble`'s edge naming, where
`which_arc_of_a_circle_this_is` reads a ball-and-plane circle only and so calls
two arcs of a cone's rim circle between one pair of corners a single edge. That
same gap is what would let the ellipse case — a pipe, and a cone cut by a
non-square plane — be counted at all. The ordering grid's refusals are asserted
as a counted map now, so a third kind cannot arrive unnoticed.

### 206.6 What is left

- The four of §200.3's eight that still refuse are a **mesher** gap: the
  intersection's cone face is a ring of three arcs at `z = 2.5592648499956177`
  running right round the apex with no seam, and `cut_up`/`walk_a_wire_round`
  give `AWireWillNotLieFlatOnItsFace`.
- The nappe floor back to 10 000 needs `is_a_family_this_follows` to admit a
  cone's rim arc, with the family argument re-made for a cone (206.2).
- `assemble.rs`'s `which_arc_of_a_circle_this_is` handles only a ball-and-plane
  circle (206.5). Three counts sit on that gap now.
- The two unsettled closed-run families (206.4), either of which panics today.
- `validate.rs`'s `how_much_space_a_curved_solid_encloses` breaks its halving
  loop on any refusal from a finer mesh, so a coarse mesh that folds is handed
  straight on and §41.6's rescue never runs. Retrying
  `TwoPatchesOfTheMeshDrewTheSameSide` the way a crossing is retried would be a
  second, independent guard against 206.3's whole family.
- Still deferred, unchanged: §124, §136.4, §125.6, §191.6.

Commands: `just decision 204.1`, `204.2`, `204.3`, `202.2`, `200.3`, `16.10`.
