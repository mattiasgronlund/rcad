## 224. The picker margin, a cone and its copy, and the joiner still unbuilt — 2026-09-25

§0's next row, §223.4, went to three lanes off `8c064e83`, which the
dispatching session gated green first: **PASS, 2371 tests, 11 skipped.** One
lane took §223.2's picker margin and then the joiner. One took the 46 fuses of
a cone with its copy. One, read-only, checked what §222.3's `U∪U'` refusals
are. The margin landed. The joiner did not. The cone-and-copy fix is real but
moves eleven other tests, so only its reproducers landed. The merged tree
`b86e7bc9` gated **PASS, 2374 tests, 16 skipped**.

### 224.1 The picker margin, and the risk §223.2 named was real

§223.2's patch as written broke 10 of rcad-topo's tests, all half-ball ones,
for example `a_half_ball_cut_across_its_polar_axis_measures_half_a_ball_without_its_cap`
(`mise exec -- just t -p rcad-topo --no-fail-fast`: 916 passed, 10 failed).
An exact semicircle has both ends on the chord's own plane, and the patch always
lowered the picker below them. With a guard for that case alone, the patch still
moved the circle's centre in front of the floor arc's picker once the arc came
within `t=2^-44` of a semicircle. Base keeps the centre behind down to `2^-50`.

What landed in `02154238`: if both stored ends read exactly Zero on the chord
plane, keep that plane. Otherwise lower the picker against each end grown by
`bake::what_writing_the_corner_down_cost`, but only while the grown ends read
`Pos | Zero` **and** the centre still reads `Neg | Zero`. If that never holds,
fall back to base's picker. So no arc is named worse than on base. The test
`a_near_semicircle_keeps_its_centre_behind_its_picker` in
`tests/pieces_of_one_curved_surface.rs` pins it. It passes on base and fails
on the margin without the centre guard.

`(block ∪ frustum) ∪ a cube that touches nothing` now gets past the picker. It
reaches `AFaceMeetsItsNeighbourOnItsOwnSurface`, the unjoined cone pieces, and
two new tests pin that. The two frustum trios in `fuse_laws_over_a_cone.rs` now
refuse there too, and their ignore strings say so. Where the margin cannot be
granted without moving the centre (`t` in `[2^-45, 2^-44]` for a radius-2.5
circle), `AFaceTrimmedByACurveThisCannotFollow` still stands, as on base.

### 224.2 The joiner is not built, and the design is written down

`a_block_and_frustum_united_can_be_united_again` is `#[ignore]`d on
`AFaceMeetsItsNeighbourOnItsOwnSurface`. It is a gap: the one face those pieces
make up exists. The lane read `assemble.rs:564-880` and reached this design:

1. **Same surface, same way round.** The fuse already names surfaces bit for
   bit (`name_of_surface_bits`, `name_of_the_surface_either_way_round`), and
   `SurfaceMeet::SameSurface` is what raises the refusal. The same bits and the
   same `used` orientation give Pos. The same surface the other way round gives
   Neg, so do not join. A surface equal only up to how it is written gives
   Undecidable, and those faces are left apart.

   > **2026-10-05 (§407): closed for pipes.** `one_cone_or_pipe_the_same_way_round`
   > now folds a pipe's spelling before comparing, so a pipe joins however its
   > axis is turned or its origin slid. A cone still compares raw bits here.
2. **Rings without a normal.** The ring walk (`ring_from`/`onwards`) never
   uses the plane. Only `rings_round`'s winding about `plane.normal()` does. On
   a cone or cylinder, classify rings in `(u, v)` instead, with `u` the angle
   about the axis unwrapped along each edge. A ring whose net `u` turn is 0
   is an outline or a hole, by the sign of its area. A ring whose net turn is
   ±2π goes right round. For that one, keep one crossing edge as a seam (the
   lowest index, so the choice does not depend on argument order), or refuse.

   > **2026-10-04 note (Q869): a seam is not always one edge.** A seam is one
   > dissolved edge from ring to ring where there is one, and otherwise the
   > chain of rulings from ring to ring with the fewest edges and the lowest
   > corners (`a_seam_across_the_band` in `fuse/assemble.rs`, Q503's
   > `f8155020`): a box standing inside a bar cuts the wall into 24 pieces, 8
   > round by 3 up, and no one edge runs from rim to rim. Chains are kept to
   > rulings, which keep their angle round the axis, because the mesher
   > refused a chain of two arcs where a pointed cone or frustum is united
   > with a cube stood on its corner round the apex
   > (`TwoPatchesOfTheMeshDrewTheSameSide`, seven tests across four files, not
   > traced). The corners in the middle of a chain stay corners, as every
   > corner where pieces of one cone or pipe met does.
   >
   > **2026-10-15 note (Q2092): a seam is a chain of any edges, read off by
   > where its corners lie.** Chains are no longer kept to rulings: Q868 found
   > the mesher's `TwoPatchesOfTheMeshDrewTheSameSide` gone at `548b0dd1`. A
   > box standing inside a frustum off its axis crosses the wall in hyperbola
   > arcs and no ruling, and left the wall in 28 pieces; a cube stood on its
   > corner round a pointed cone's apex left 9. With a seam of any dissolved
   > edges each wall is one face that validates and measures right, and the
   > cone's union meshes from deflection 0.5 down to 0.0005
   > (`a_box_inside_a_frustum.rs`, `a_wire_that_rings_a_cones_apex.rs`). The
   > seam's corners now go by where they lie alone
   > (`Assembly::which_lies_lower`), not input vertex first as the corner a
   > closed run keeps does, since which corners are input vertices changes
   > when an answer is fused again. At `BesideABlock` slope one half from
   > `z = 2` to `z = 10` in `fuse_laws_over_a_cone.rs`, the union `A` has one
   > input vertex on a ring, `(1, 0, 2)`, the frustum's own seam corner, and
   > in `A ∪ A` every ring corner is one of `A`'s. Input first kept the chain
   > of two arcs on `x = 1` through `(1, −1.118, 3)` in `A` and its mirror on
   > `y = 1` through `(−1.118, 1, 3)` in `A ∪ A`, and idempotence failed. By
   > where they lie both keep the chain on `y = 1`, through `(0, 1, 2)`,
   > `(−1.118, 1, 3)` and `(−4.899, 1, 10)`.
3. **Corners that stop being corners.** This is the hard part. After a join, a
   corner with only the cone and one other surface round it sits in the middle
   of an arc, and §17.1 cannot re-derive it. The two arcs either side must
   become one `EdgeRun::PartOfWhereItCrosses`, with a new picker and a new
   `towards`. `edges_after_joining` does this for straight edges only.
4. `plane_of_the_joined_face` and `rebuild_from` assume `Surface::Plane` and
   `Orientation::Forward` for any group of more than one face. They must carry
   the shared surface and `used` instead.

### 224.3 All twenty `U∪U'` refusals are the unjoined pieces

§223.2 guessed that the ×12 `AFaceMeetsItsNeighbourOnItsOwnSurface` on `U∪U'`
was the unjoined cone. The read-only lane rebuilt all 23 of the test's pairs in
a scratch crate and counted faces per curved surface of `U` through
`FaceView::surface()`. It got exactly 8 `AFaceTrimmedByACurveThisCannotFollow`
and 12 `AFaceMeetsItsNeighbourOnItsOwnSurface`, the same counts as §222.3. **In
all 20 of them, `U` carries its cone as 2, 5, 7 or 8 faces.** So the ×8 is the
same fact met one step earlier. The other refusals in that test are
`Undecidable`, on a point that every solid could place or on a point inside a
piece of a face. §224.4 is the first of those.

### 224.4 A cone and its copy: the cap, not the wall

All 46 fuses of a cone with its copy refuse on "a point in a piece that every
solid could place", and the cause is the flat cap. In `select.rs`,
`a_face_of_the_input_holds` decides whether a flat face holds a point by
winding the polygon of that face's corners. A cone's cap is one circle with one
corner, so that polygon winds round nothing. The cap piece's point, for example
`(0,0,2)`, was held by no face of the copy. A ball has no flat face, which is
why a ball and its copy answer. This is a **gap**.

The fix, on `lane-cone-and-its-copy` at `13d5abaa`: a flat face with any side
whose neighbour is not a plane is asked through `SolidView::face_holds_the_point`
instead. With it, all 46 cases compare with no findings. **It is not merged**,
because it moves 11 other rcad-topo tests (930 run: 919 passed, 11 failed):

- Seven are refusals pinned as limits that now answer. They include a bar and
  its copy (`degenerate_curved_fuses`, §37's table row), two
  `ordering_points_along_an_endless_bend` counts, and
  `one_pipe_whose_origin_sits_at_two_places`. So §36.16's "two copies of one
  curved face" row was the flat caps of bars as well as cones.
- `fuses_that_must_agree` over `FrustumAndBlock{side:7, up_per_unit_wider:8,
  at:[3,3]}` now answers one way round and gives `ResultIsNotManifold{count:1}`
  the other. The frustum's cap sits flush with the block's roof, and each order
  keeps a different one of the two coincident disks, whose circles carry
  different corners. The lane traced that far and no further.
- `two_bars_on_one_pipe_are_refused_the_same_however_the_pipe_is_written` now
  answers for one spelling and refuses `AWireWillNotLieFlatOnItsFace` for the
  other.
- Absorption with A the cone now answers, but with one extra corner, such as
  `[3.0,1.5,0.0]`, where the block floor crossed the cone wall. That looks like
  §224.2's fact again. Nobody has shown it.

What landed is `0a2cad50`, which adds the four reproducers in
`tests/a_cone_and_its_copy.rs` as `#[ignore]`d tests. With the margin lane's
one, `ignored_tests` goes from 11 to 16.

### 224.5 What this leaves

The joiner (§224.2) is still the next unit, and it now has a design. Once it
lands, re-run `13d5abaa` and `lane-swallowed-pointed-cone` (`184962b9`). Many
of their failures should be the same pattern. §224.4's flush-cap asymmetry and
two-spellings asymmetry are separate, and each is a `fuse-debugging` lane.
