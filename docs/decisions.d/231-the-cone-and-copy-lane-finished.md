## 231. The cone-and-copy lane finished: a corner no piece needed, two spellings of one pipe, and a pin — 2026-09-25

*§229.6 left `lane-land-cone-and-copy` (`63281be7`) red for four reasons. Lane
F1 merged `main` (`1aec994c`) into it on `lane-cone-and-copy-finish` and
worked the four in order. This records what each turned out to be.*

  231.1   rustfmt
  231.2   The absorption corner was hidden by a refusal, and the joiner kept it
  231.3   Two spellings of one pipe: two places, both fixed
  231.4   Two bars side by side: pinned, not diagnosed
  231.5   What this leaves

### 231.1 rustfmt

`cargo fmt --all` on the merged tree touched `assemble.rs` and
`a_cone_and_its_copy.rs` and nothing else (`f47ed396`).

### 231.2 The absorption corner was hidden by a refusal, and the joiner kept it

`a_cone_united_with_what_it_shares_with_a_block_is_the_cone` found four extra
corners on the lane's tip. **It is not a regression of the lane's step 2.** Run
on `647d8eed` (step 1 only) it fails the same way: 9 compared, the same four
findings. On `e68aa416` it passes having compared 4, with *"a point in a piece
that every solid could place"* refused 9 times against 4 on `647d8eed`: the cap
fix `13d5abaa` answered five more cases, and four of those carry the corner.

Neither leg of the hypothesis §229.6 passed on was the cause as stated. The
answer, printed face by face for the first frustum, was the frustum's wall in
**two faces**: one band from the lower rim to the floor circle at `z = 0`, and
one from there to the upper rim, each between two whole circles with no seam.
The corner was the floor circle's one corner, kept because the circle was kept.
It was kept because the joiner asks a band that goes right round for a seam
from ring to ring (§229.2), and a closed edge runs round a ring, never across,
so it found none and left the group apart. §229.2's rule keeping a corner where
pieces of one cone met did not come into it, by a reading of `what_is_left`
rather than a probe: that rule only marks corners some staying edge leaves, and
once the circle is dissolved none does.

The arrangement already makes, writes and fuses again bands between two whole
circles with no seam — that is what `A ∩ B` is here. So a group whose every
ring that goes right round is one closed side needs no seam, and is now joined
(`71feed66`, `Assembly::needs_no_seam`). The test passes with 10 compared, and
the one `ACurvedFaceGoesRightRoundWithNoSeam` among its refusals went with it.
A ring of several arcs going right round still needs a seam, as before.

### 231.3 Two spellings of one pipe: two places, both fixed

The diagnosis lane's two legs held, and both are fixed (`68b81daf`):

1. `the_surface_that_cut_each_closed_rim` compared the two faces' surfaces bit
   for bit, so a circle between two spellings of one wall was left
   `RightRoundWhereItsFacesCross`, which the walk reads as two pipes crossing.
   It now asks `name_of_the_surface_either_way_round`, the fold `arrange.rs`
   already names corners and cut surfaces by. That is exact, has no tolerance,
   and covers a pipe, a cone and a bend turned round, and a pipe's origin slid
   along its axis where that folds exactly.
2. `Edges::of` reads each side's way round a closed edge against the curved
   surface of that side's own crossing pair, which for a curved piece is its own
   surface, and a surface turned round is wound about the other way. The side
   that counts is the one the **mesher** reads: with two parallel gradients the
   crossing gives no direction, and
   `the_way_a_closed_edge_where_its_faces_touch_runs` falls back to the winding
   of the lower of the two faces' surfaces by bits. So
   `every_closed_edge_read_off_one_spelling` turns round each side whose
   winding is opposite that one's. That is a property of the two faces and not
   of which solid was handed over first. The two windings are exact negatives,
   so their dot product has no rounding to be near zero with. **A flat piece is
   read against a curved surface too**, its own bar's pipe, and this pass
   skipped every edge with a flat face along it; §359 has the pass compare what
   each side was read against rather than its face's surface.

Shown needed: with the second pass switched off, the reversed-axis unit test
refuses `TheFuseWroteABrokenAnswer { broken: CoedgesRunTheSameWay }`. The
slid-origin tests do not need it, since sliding does not turn the winding.

The three tests now say what the fuse does. All four unions (two spellings ×
two orders, on both the reversed and the slid pipe) answer one bar, validate,
and measure within their own bound of the volume by hand. `Model::validate`
skips a closed edge between two faces on one pipe, which is why the volume is
the check. **Two spellings still leave the wall as two faces**, because the
joiner joins curved pieces by bits (§229.2).

Across the order, faces are compared and volumes are not: with **one**
spelling the two orders of the slid-origin union measure 50.20344246266881 and
50.20344246266873, both within the bound of 16π. Bit-identical volumes either
way round are not something this fuse gives even with nothing spelled twice.

### 231.4 Two bars side by side: pinned, not diagnosed

`two_overlapping_parallel_bars_stop_in_select_whichever_way_round` is ignored,
with its cause named: the first of its six fuses, the union with `here` as the
argument, now refuses `ResultIsNotManifold { count: 1 }` rather than stopping in
`select`. The lane's unshown guess, a bar's rim corner at (5, 0, 0) inside the
lens splitting one side's arc and not its neighbour's, was not tested.
`ignored_tests` is 12.

### 231.5 What this leaves

- Diagnose §231.4. It is a gap until shown otherwise, and it is the one
  item here whose cause is still a guess.
- The joiner could join two spellings of one curved surface. It would have to
  pick one spelling by bits, and then turn round every closed side of the
  joined face read off the other one — §231.3's second pass, once more.

  > **2026-10-05 (§407): closed for pipes.** `84538a4e`/Q354 build exactly
  > this, for a pipe: `rings_of_each_joined_face` picks the lowest spelling
  > by bits, and `turn_round_each_closed_edge_the_joined_spelling_stores_the_other_way`
  > is §231.3's second pass, generalised. A cone is not reached by it.
- `lane-swallowed-pointed-cone` (`184962b9`), as §229.7 says.
