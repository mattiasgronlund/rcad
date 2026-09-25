## 223. Associativity over a cone: item 1 was the cut list, and the fuse cannot take back its own cone answers — 2026-09-25

§0's next row, §222.3's five trios, went to three `fuse-debugging` lanes off
`33c59743`, which the dispatching session gated green first: **PASS, 2369
tests, 11 skipped.** Each lane had its own source files, and a fix in a file it
did not own came back as a written diagnosis and patch. One case is fixed. Four
stay `#[ignore]`d, and their doc comments now name what actually refuses. The
merged tree `b2a3ba12` gated **PASS, 2371 tests, 11 skipped**.

### 223.1 Item 1 was a missing cut in `cuts.rs`, not the cone

`cuts::gather` skipped any two solids whose boxes do not overlap. In
`block ∪ [frustum, cube]`, the two tools never touch each other, but the
block's box reaches both. The inside cube's wall `x=3.5` cuts the block floor,
and the floor meets the frustum's wall along the frustum's outline. So the floor
side of that edge was split at `x=3.5` and the wall side was not, which left
six edges with only one face along each.

**A trio of boxes does the same.** `block9 ∪ [box2 at (2,4,-1), box3 at
(0.5,0.5,4.5)]` is `ResultIsNotManifold` under union, intersection and
difference. Swap the inside box for one far away and it answers. That trio is
now the test
`a_block_with_one_box_through_its_floor_and_one_inside_it_unites_to_the_block_and_the_first`.

The fix cuts every pair of solids that a chain of overlapping boxes joins,
using `Sameness`, and the module header's "that changes no answer" now says the
same. With the fix, one trio in `TRIOS_THAT_AGREE_ABOUT_ANSWERING` answered in
the grouping that used to refuse for this reason. Its other grouping refuses at
the first union on "a point inside a piece of a face", which is item 2's
refusal on a frustum. It moved to its own `#[ignore]`d test,
`a_frustum_a_block_and_a_cube_beside_the_frustum_…`, so the count stays at 11.

The lane was not allowed to edit `cuts.rs`, so it left the fix as a patch. The
dispatching session applied it in `b2a3ba12`. The verifier applied the same
patch to the lane's branch and ran the rcad-topo tests: 926 of 926 passed.

### 223.2 Items 4 and 5: two gaps, one after the other, and neither is `holds.rs`

The hypothesis handed out was "the fuse cannot take back its own hyperbola
arc". Its first two legs broke.

- **The first refusal is on a circle arc, and it comes from `arrange.rs`.**
  `which_way_an_edge_runs_round` (reached from `cuts::gather` through
  `check_it_does_not_cross_itself`) refuses on the answer's block floor at
  `z=5`. The edge is the cone's circle of radius 2.5. The first fuse's
  `the_half_space_both_ends_are_on` lowered the picker just far enough for the
  stored f64 corners to read on or in front of it. The second fuse's bake then
  re-solves each corner from three surfaces into a box one ulp wide, and both
  boxes read `Undecidable` (enclosure `2.66e-15`) against that picker. A patch
  tested only on this file makes the refusal go away: lower the picker against
  each corner grown by `bake::what_writing_the_corner_down_cost`, and require
  `Pos | Zero`. It touches `assemble.rs` and `bake.rs` and is not landed, and
  the whole suite has not been run with it. The lane named the risk: lowering
  the picker by a few `1e-14` could flip the centre-behind test on an arc
  that is nearly a semicircle.
- **The next refusal is the hyperbola after all.** With that patch, the second
  union refuses `AFaceMeetsItsNeighbourOnItsOwnSurface` (`arrange.rs:1009`).
  The answer to `block ∪ frustum` leaves the frustum's cone as several faces.
  The block's wall planes `x=1` and `y=1` cut it apart below the block, and
  one face has the same cone across three of its four sides. **The fuse merges
  pieces that lie on one plane into one face, but it never does that for pieces
  on one curved surface.** That is probably also the ×12 of that name on `U∪U'`
  in §222.3, but nobody has checked.

### 223.3 Items 2 and 3: the same unmerged cone pieces, reached another way

- **Item 2** refused on a piece of the cube wall `x=3.5` inside the cone. The
  piece is bounded by a hyperbola arc and the cap's chord, and every candidate
  point landed on the chord. The lane changed `select.rs` to count the bulge of
  a cone arc when the point is shown to be on the same half of the cone as both
  of its ends. That fix is a gap closed: `cone ∪ cube` answers, and union plus
  intersection add up by volume. The trio still refuses at the next union
  (`AFaceTrimmedByACurveThisCannotFollow` on the answer's cone face), which is
  §223.2's family. **The change breaks 9 other rcad-topo tests.** Several of
  them pin a refusal as a limit, which the same-half test now shows is a gap.
  One, `a_wall_lying_on_a_cones_side_leaves_that_crossing_with_no_direction`,
  now reads `ResultIsNotManifold{count:4}` and has not been analysed. So it is
  **not merged**. It stays on `lane-swallowed-pointed-cone` at `184962b9`, and
  the verifier's gate on that branch confirmed the failures.
- **Item 3** refuses at `mesh.rs:895`. The union keeps the cone cut by the
  cube's planes, past the cube itself, as separate faces. The patch round the
  apex then has three edges with the cone on both sides, and no partner
  surface to mesh against. A mesher patch that takes the partner from the
  edge's run gets the union meshed. But then the second union refuses
  `AFaceMeetsItsNeighbourOnItsOwnSurface`, and one absorption case gains a
  corner, `(3,4.5,6)`, that the cone does not have. That patch was reverted.

### 223.4 What this leaves

Four of the five cases, and §223.1's moved trio, come down to one fact: **a
fuse answer can carry one curved surface as several faces, and the fuse cannot
take such an answer back in.** That is the next unit. Merging pieces on one
curved surface into one face, which the fuse already does for a plane, belongs
in `fuse/assemble.rs` or bake's joiner. §223.2's picker margin is the smaller
gap in front of it, and its patch is written out there.
`lane-swallowed-pointed-cone`'s `select.rs` change should be re-run once the
faces are merged, since most of its 9 failures are the same pattern.

The verifier on `not-manifold-one-way` saw `check-fuzz-overnight-selftest` fail
four times on the untouched base, with four different messages, while the load
average was 49–54. The dispatching session's gate on `b2a3ba12` passed that
step. §222's merged tree had stopped there too.
