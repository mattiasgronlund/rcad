## 247. An arc is named by its circle as well as its corners — 2026-09-25

*Q75, which is §245.4's hole B. Lane `lane-q75-arc-edge-names-its-circle`, off
`81f18a4d`. `Edges::of` in `crates/rcad-topo/src/fuse/assemble.rs` now names
an arc of a closed crossing by its curved surface, named either way round,
beside its two corners and the bit that says which of the crossing's two
arcs it is. The two fuses that met hole B alone now answer and measure right.
The three that meet hole A still refuse `count: 1`, at the same edges as in
§245.1. No pin moved.*

  247.1   The test, and how it failed on the base
  247.2   The fix
  247.3   The three legs: all held, and the second only as far as anything reaches it
  247.4   What the six fuses do now
  247.5   What this leaves
  247.6   What the verifier found: two planes on one ball

### 247.1 The test, and how it failed on the base

`an_arc_of_each_bar_between_the_same_two_corners_is_an_edge_of_its_own`, in
`crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs`, fuses the two
hole-B-only fuses of §245.6: the union with `there` as the argument, and the
difference with `here` as the argument. It then validates the model, asks for
one solid, and checks the volume against the one worked out by hand. On the
base it failed at the first of them:

```
    thread 'an_arc_of_each_bar_between_the_same_two_corners_is_an_edge_of_its_own' (1423026) panicked at crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs:223:39:
    Union of two side-by-side bars: ResultIsNotManifold { count: 4 }
```

With the two fuses swapped round, the difference fails the same way on its own:

```
    Difference of two side-by-side bars: ResultIsNotManifold { count: 4 }
```

**The volume by hand.** A bar is `π·5²·10 = 785.398…`. The two bars share a
lens of two discs of radius 5 with centres 6 apart. Its half-angle is
`acos(3/5)` and its chord is 8 long, so its area is `2·5²·acos(3/5) − 3·8 =
22.3648…`, and the shared volume is ten times that, `223.648…`. With the fix,
a temporary print beside the assertion gave:

```
TEMP Union: faces 4 volume 1346.5972099829255 error 2.2077367089531887 by hand 1347.1487177940905
TEMP Difference: faces 4 volume 561.5897429190844 error 1.5661424928259258 by hand 561.7505543966422
```

Both are within the mesh's own bound, `0.55 ≤ 2.21` and `0.16 ≤ 1.57`. Keeping
or dropping the lens wrongly moves the answer by 223.6, a hundred times either
bound, so the check can fail. Four faces is right for both. The union has one
bottom cap, one top cap and two walls. The difference has two caps, its own
wall, and `there`'s wall turned inward. Both caps of the union are single
faces because the two bars' caps lie on one plane each and are joined.

### 247.2 The fix

The third part of `EdgeName` went from `Option<bool>` to
`Option<([u64; 9], bool)>`. `which_arc_of_a_circle_this_is` now returns
`name_of_the_surface_either_way_round(&meet.carried_by[round_one])` beside the
bit it returned before, and `None` in exactly the cases it did before. Only the
curved surface goes in and the plane does not. The plane is the same for every
arc on it, so it tells no two of them apart. `which_curve_it_is`'s doc records
what a plane cost in an open side's name: the skew family's share that came
out as not a solid rose from under three in a hundred to three and a half.

`which_arc_of_a_circle_this_is`'s doc has a new part, *"Which circle, as well
as which arc"*, that says the same.

### 247.3 The three legs: all held, and the second only as far as anything reaches it

**Leg 1 held, by construction.** `which_of_them_a_closed_crossing_runs_round`
tests each surface as `closes_round(round, flat)`, whose first line is
`let flat = flat.as_plane()?;`. So it answers `Some(i)` only when the other
surface is a plane. For a plane it answers `Some(false)`. For a ball it answers
`Some(true)`, and for a cone or a pipe it answers what
`is_cut_in_a_closed_curve_by` says. `carried_by[round_one]` is therefore
always a ball, a cone or a pipe, never a plane. A side between two curved
surfaces has no plane, so it gets `None` and keeps the name it had before.

**Leg 2 held where the suite can check it, and the suite does not reach the
case it is about.** The tests that pin §231.3 still pass:
`two_bars_on_one_pipe_answer_the_same_however_the_pipe_is_written` in
`assemble.rs`, and the three in `one_pipe_whose_origin_sits_at_two_places.rs`.
But their two walls share a **closed** circle, and
`which_arc_of_a_circle_this_is` answers `None` when `from == to`. So they never
read the new part of the key. To check that, the fold was replaced with
`name_of_surface_bits` for one run of the whole crate:

```
     Summary [ 165.424s] 951 tests run: 951 passed (4 slow), 12 skipped
```

So no test in `rcad-topo` puts an open arc between two spellings of one wall.
Two shapes were tried to make one, and neither got there:

- **Two half bars stacked on one pipe**, each made as a bar minus a block over
  `x < 0`. The union refuses before `assemble`, even with one spelling:

  ```
  two half bars end to end, lower first true: AFaceTrimmedByACurveThisCannotFollow { face: FaceKey { slot: FaceSlot(10v1), model: ModelId(1) } }
  ```

  That refusal is raised in `holds.rs`.
- **Two whole bars stacked with their seams on opposite sides**, at
  `(1, 0, ·)` and `(−1, 0, ·)`. All four runs answer the right volume, for both
  spellings and both orders, and they passed with the bits in place of the
  fold too. A print in `which_arc_of_a_circle_this_is` showed nothing, so no
  side of that fuse gets an arc's name at all. The circle between the walls
  stays closed.

Neither test was kept, because neither can fail for the reason it would be
named for. So the fold rests on an argument, not a test. (Since Q350 and Q532,
four tests fail without its pipe part: §247.6's note of 2026-10-10.) It is the same exact
fold, with no tolerance, that `names_of_the_three_surfaces` and
`names_of_the_two_surfaces` name an arc's corners and a closed crossing by.
Naming the arc by the stored numbers would let two faces agree on both
corners and still disagree on the edge between them.

**Leg 3 held.** Both hole-B-only fuses got past the manifold check. The joiner,
grouping into shells, nesting, the throwaway sweep and `Model::validate` all
accepted them, and both measure right (§247.1).

### 247.4 What the six fuses do now

A temporary test ran all six, each on a fresh copy of the model:

```
TEMP ===== Union here on there: Err ResultIsNotManifold { count: 1 }
TEMP ===== Union there on here: Ok, 1 solids
TEMP ===== Intersection here on there: Ok, 1 solids
TEMP ===== Intersection there on here: Err ResultIsNotManifold { count: 1 }
TEMP ===== Difference here on there: Ok, 1 solids
TEMP ===== Difference there on here: Err ResultIsNotManifold { count: 1 }
```

A temporary print in `darts_along_each_edge` of every edge without exactly two
faces gave:

```
TEMP edge 0: ends [0, 1] claimed by 1 faces
TEMP edge 1: ends [1, 2] claimed by 1 faces
TEMP edge 4: ends [3, 4] claimed by 1 faces
TEMP edge 5: ends [4, 5] claimed by 1 faces
TEMP edge 12: ends [0, 2] claimed by 1 faces
TEMP edge 15: ends [3, 5] claimed by 1 faces
TEMP ===== Union here on there: Err ResultIsNotManifold { count: 1 }
TEMP edge 1: ends [0, 1] claimed by 1 faces
TEMP edge 3: ends [2, 3] claimed by 1 faces
TEMP edge 6: ends [0, 4] claimed by 1 faces
TEMP edge 7: ends [1, 4] claimed by 1 faces
TEMP edge 8: ends [2, 5] claimed by 1 faces
TEMP edge 9: ends [3, 5] claimed by 1 faces
TEMP ===== Intersection there on here: Err ResultIsNotManifold { count: 1 }
TEMP edge 2: ends [0, 2] claimed by 1 faces
TEMP edge 5: ends [3, 5] claimed by 1 faces
TEMP edge 8: ends [2, 6] claimed by 1 faces
TEMP edge 9: ends [0, 6] claimed by 1 faces
TEMP edge 10: ends [5, 7] claimed by 1 faces
TEMP edge 11: ends [3, 7] claimed by 1 faces
TEMP ===== Difference there on here: Err ResultIsNotManifold { count: 1 }
```

No edge has four faces any more, so hole B is gone from all six. Each of the
three that still refuses has hole A's two triples of one-face edges, one triple
per cap and nothing else. For the intersection and the difference with
`there` as the argument, the edge numbers are §245.1's exactly. For the union
with `here` as the argument, §245.1 had edges 0, 1, 10 and 3, 4, 13, and it is
now 0, 1, 12 and 3, 4, 15. The shift is the two edges that §245 found sharing
one name, which each became two edges and are made before the third edge of
each triple.

`two_overlapping_parallel_bars_stop_in_select_whichever_way_round`, still
ignored for Q77, now stops at its first fuse with:

```
Union on two side-by-side bars answered Err(ResultIsNotManifold { count: 1 })
```

**No pin moved.** The whole crate passed with the fix before the probe tests
were taken out:

```
     Summary [  88.911s] 952 tests run: 952 passed (2 slow), 12 skipped
```

That count includes the temporary six-fuse test, which asserts nothing. With
the probes out, on the tree that is committed:

```
     Summary [  71.894s] 951 tests run: 951 passed (2 slow), 12 skipped
```

Both runs include `fuses_that_must_agree.rs`, which compares answers bit for
bit, and `turning_a_fuse.rs`. Against the base, the new test is the only one
added.

### 247.5 What this leaves

- Fix hole A (Q76): an input vertex that only splits one crossing is not a corner of the answer, in `Assembly::from_the_pieces` and `Corners`, `crates/rcad-topo/src/fuse/assemble.rs`.
- Once hole A is fixed (Q77), rewrite `two_overlapping_parallel_bars_stop_in_select_whichever_way_round` as six fuses that answer, and take off its `#[ignore]`. Its file header still says the pair stops in `select.rs`, which is wrong for every fuse since §245. `crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs`.
- Find a shape that puts an open arc between two spellings of one wall, so that the fold in an arc's name has a test that fails without it (§247.3). `crates/rcad-topo/src/fuse/assemble.rs`.
- Two half bars stacked on one pipe refuse `AFaceTrimmedByACurveThisCannotFollow` with one spelling, before `assemble`. That refusal is not in §245 and was not looked into here. `crates/rcad-topo/src/holds.rs`.

### 247.6 What the verifier found: two planes on one ball

The verifier re-ran the base failure (`ResultIsNotManifold { count: 4 }`),
checked that `None` comes back in the same cases as before, re-derived the
volumes and bounds, and found the new name strictly finer than the old one, so
the change cannot join two edges that were apart before. It said merge. It
also found where the name is still not fine enough, and wrote a probe for it
that was not committed:

- **Arcs of two circles on one ball, cut by two planes, still get one name.**
  A ball of radius 5 at the origin, less the block `x ∈ [−10, −3]`,
  `y ∈ [−10, 10]`, `z ∈ [0, 10]`: the planes `x = −3` and `z = 0` cut the ball
  in two circles through `(−3, ±4, 0)`, and an arc of each runs between those
  two corners on the same curved surface. `ball − wedge` refuses
  `ResultIsNotManifold { count: 4 }` on the base and on the tip. With the plane
  folded into the name as well, it moves on to
  `ACurvedFaceGoesRightRoundWithNoSeam { face: FaceSlot(2v1) }`. The other
  three fuses of the pair (`ball ∩ wedge` both ways round, `wedge − ball`)
  answer right. So 247.2's argument that the curved surface is named and the
  plane is not holds only for arcs that lie on one plane.
- **That shape might also reach a wrong answer.** Two spellings of one wall
  that the fold leaves apart could pair four faces crosswise into two edges
  that each look sound. Nobody built it.
- **The surface fold has no test that fails without it.** With the raw bits
  in its place, 951 of 951 tests pass, and 39 more fuses tried all refused in
  `holds.rs` first. That is 247.5's third item, measured.
- **Two half bars stacked on one pipe refuse before `assemble`** even with one
  spelling, as `AFaceTrimmedByACurveThisCannotFollow` and in some fuses
  `AFaceMeetsItsNeighbourOnItsOwnSurface`. That is 247.5's fourth item.

*2026-10-10, Q115:* the pipe part of the fold now has tests that fail without
it.

- **Four tests fail without it.** With `name_of_surface_bits` in place of
  `name_of_the_surface_either_way_round` in `which_arc_of_a_circle_this_is`,
  four of 1797 rcad-topo tests fail, all `ResultIsNotManifold { count: 1 }`:
  - `two_half_bars_fuse_directly_to_what_cutting_the_block_off_afterwards_gives`.
    This is the union of two half bars stacked on one pipe, with the upper
    wall's axis at z = 2, and it reaches `assemble` since Q350.
  - the three tests of `two_bars_on_one_pipe_and_a_box_united_in_one_call.rs`
    that run every wall, since Q532.
- **What was reproduced.** The dispatcher reproduced the four failures, as one
  line shadowing the fold in that function, over those two files: 10 run, 6
  passed, 4 failed.
- **What only the lane measured.** With only the axis origin left raw, all four
  fail. With only the direction left raw, the three whose upper bar's axis is
  turned round fail.
- **The cone and ring parts of the fold are still reached by no test.** Half
  frustums whose cone is spelled about −z cannot be cut today (Q654). The top
  halves of two quarter bends with one ring spelled about −z refuse
  `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` in all six fuses (Q758). Q1564
  and Q1565 wait on those two rows, to pin the unions once they answer.
