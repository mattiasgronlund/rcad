## 345. A tolerant union of two blocks apart answers again — 2026-10-04

*Queue row Q438 (Q122, then Q202 and Q429). Lane
`lane-q438-a-tolerant-union-of-two-blocks-apart`, off `aa08c916`, after §395.
Q429's lane, off `82f20827`, drafted this section and stopped on a pin
outside its write set. This is the section rewritten from what was measured on
the newer base. **The union answers again, with the two blocks, their volume
and a certified sweep that passes, and no other test in the workspace
moves.** §281.2's checked step narrows the box round a meet of three planes,
and the narrowed box is asked **one question only**: whether every point of
the meet lies inside the corner's room, in `a_corner_is_this_place`. The node
a meet becomes when it does not fold keeps Cramer's box. Wired in as the
node's own box, as §281.3's probe and Q429's commit had it, the narrowed box
moved 13 tests on this base, most of them to a refusal or to no solid. So it
is not wired in there.*

### 345.1 The narrowed box

`MeetOfThree::boxed_tight` in `crates/rcad-geom/src/three_surfaces.rs`, Q429's
code (`472f7420`, cherry-picked). Since then it is a method that narrows a
meet `meet_of_three_surfaces` already worked out, so the fold's caller solves
Cramer once and keeps both boxes. Where the three surfaces are planes, it
takes the middle of Cramer's box as a guess and works out
`offset − normal · guess` per plane, and the cross products and the
determinant, as `Nearly`. That is exact, or exact but for what falls off the
bottom of the range, which the floor's subnormal offset in §281.1 needs. It
sums the step in intervals and hands back where the stepped box and Cramer's
box overlap. Both hold the meet, so the overlap is sound, and an empty one
panics. Where a product overflows, or the determinant's enclosure reaches
zero, Cramer's box comes back as it was. The planes are narrowed in
`meet_of_three_surfaces`'s fixed order, so the box does not depend on argument
order (§31.3).

`a_meet_of_two_walls_nearly_parallel_is_boxed_to_a_few_ulps_and_still_holds_it`
takes a floor and two walls `1e-6`, `1.5e-6` and `3e-6` off parallel. Cramer's
box is wider than `1e-12`, and the narrowed one is narrower than `1e-14` and
inside it. Each side of the narrowed box is checked exactly to hold the meet.
Along an axis the meet is `D / det`, so it is inside `[lo, hi]` when
`D − lo·det` and `hi·det − D` both have `det`'s sign. A box `1e-12` past the
narrowed one fails that check, so the check can fail.

On the Q122 draw the narrowed box is `[5.0, 5.000000000000001] ×
[−1.1394645277447636e-33, −1.1394645047858895e-33] × [−3.5384182432833117e-39,
−3.538418243283311e-39]`, bit for bit §281.2's. Cramer's box there is
`2.860170061287985e-10` wide and the corner's room is `7.37e-14` either side
of it. That was read with a temporary log in `a_corner_is_this_place`, taken
back out (345.3).

**One step is enough at these angles and not at every angle.** The step's
error is the first box's width times about `ε / det`. At `det ≈ 3e-6` that
factor is `3e-11`. At `det ≈ 1e-12` the first box is about `1e-4` wide and
one step leaves about `1e-8`. That is worked from the formula and was not
measured. Nothing here needs more.

### 345.2 Where the narrowed box is asked, and where it is not

Q429's commit asked `boxed_tight` in `place_the_meets` in place of
`meet_of_three_surfaces`, so every node of three planes carried the narrowed
box. On `aa08c916`, with §395's fold, `mise exec -- just t -p rcad-geom -p
rcad-topo --no-fail-fast` gave `2311 tests run: 2298 passed, 13 failed, 9
skipped`:

| Test | On the base | With the node narrowed |
|---|---|---|
| `a_meet_further_from_a_corner_than_rounding_reaches_is_not_folded_into_it` (Q122) | refused | `ok: 1`, two solids |
| `a_pyramid_whose_two_faces_fold_by_a_sixty_fourth_is_united_…` | united | refused, two regions of one face |
| `a_pyramid_whose_apex_meets_lie_inside_its_room_but_miss_its_box_…` | united | refused at 1/16, two regions of one face |
| `a_pyramid_whose_base_corner_lies_past_its_own_rounding_…` | united | refused at 1/4096, two regions of one face |
| `turning_a_pair_makes_what_the_two_share_depend_on_…` | `Solids(2)` | `NotASolid` |
| `how_often_one_pair_of_turned_boxes_leaves_such_an_edge` | 104 | 207 |
| `an_ell_and_a_comb_turned_half_a_turn_unite_…` | answers | refused |
| `the_union_of_this_pair_has_a_ring_that_goes_through_one_corner_twice` | the ring | no ring |
| `a_wedge_cut_with_the_turned_knife_first_is_refused_by_name` | by name | `Undecidable`, a point inside a piece |
| `a_comb_standing_in_a_block_fuses_turned_as_it_does_upright` | answers | not a solid, all six |
| `a_frame_with_a_block_over_one_corner_fuses_turned_as_it_does_upright` | answers | not a solid, all six |
| the two frustum tallies in `a_plane_cut_along_a_cones_other_nappe.rs` | as pinned | moved |

The last two `a_turned_piece_…` tests are in a file this row does not own,
and they came after §281.3's list of eight.

**The three pyramids move because the fold's reach reads the narrowed box.**
A temporary switch made the reach test alone read the narrowed box, with every
node keeping Cramer's. All three pyramid tests failed the same way. §356's
branch folds a meet of three of the corner's own planes as long as its box
reaches the corner's room. At a fold of 1/64 the meet of the three stored
planes is 1.6 rooms past the apex's box (§356). Cramer's box round it reaches
the room, and a box a few ulps wide round it does not. So the meet stays a
node, and the face refuses. That is §395.4's gap made wider, and Q848 is the
row for it.

**The other ten move only when the node itself keeps the narrowed box.** With
the narrowed box asked by the room test alone (345.3), none of them moves.
Why each one moves when the node is narrowed was not traced. §281.4's reason
was that a tolerant policy merges only points whose boxes touch, so narrower
boxes merge fewer. That reason cannot be the whole story: all but the Q122
draw run under `Policy::certified()`, where two touching boxes are not merged
but refused (`merge_points_that_are_one_point` refuses on an undecided order).
**So no pin moves to a worse answer here, and none needs the argument §16.10
asks of one.** The node keeps Cramer's box.

**The room test is where the narrowing belongs.** §253's argument for the
fold is that every point of the meet's box lies within the corner's
rounding, so the meet and the corner are one point written down two ways. A
narrower box that soundly holds the meet makes that argument for more meets
and no different one. Since the narrowed box lies inside Cramer's, the one
condition that reads it can only become true where it was false, and never
the other way: the narrowing only adds folds. The reach and the count of places
reaching the room still read Cramer's box, so §356's branch and §288's two
places are exactly as §395 left them.

### 345.3 What moves with the room test narrowed

`mise exec -- just t -p rcad-geom -p rcad-topo --no-fail-fast`, with the room
test reading the narrowed box: `2311 tests run: 2310 passed (2 slow), 1
failed, 9 skipped`. The one is the Q122 pin, now answering. `mise exec -- just
t --workspace --no-fail-fast`: `2955 tests run: 2954 passed (2 slow), 1
failed, 9 skipped`, the same one.

A temporary log in `a_corner_is_this_place` recorded every call where the
fold with the narrowed room test and the fold as on the base disagree, over
the whole workspace run. There were 26, every one a fold the base did not
make and none the other way. 16 were in
`a_fused_draw_hands_its_provenance_back_through_the_door`, 8 in
`every_seed_in_the_checked_in_corpus_is_fused_without_panicking` and 2 in the
Q122 test. Cramer's box in them was from 1.3 times the half-room wide up to
1.9 million times it, and in 6 of them the coordinates are near `2.4e21`.

The corpus replay prints its tallies. The same binary with the fold switched
back to the base's gave `ok: 349, refused: 341, solids_made: 356`, and with
the narrowed room test `ok: 350, refused: 340, solids_made: 358`. One more
fuse answers, with two solids. The replay's both-ways check (`asked: 108`,
`the same: 36`) did not move.

The pyramid pins all pass. They are the 324 unions at folds of 1/4, 1/16 and
1/64, the 36 at 1/64 and height one, and the base-corner test. So does
`a_pyramid_whose_apex_meets_miss_even_its_room_is_still_refused`: the
narrowing closes none of §395.4's 5. The row's "12 of 36" and "36 of 36" were
measured off `82f20827`, with the touch test still in, and do not describe
this base.

### 345.4 The Q122 draw's answer

`the_two_blocks_unite_with_their_volumes_and_pass_a_certified_sweep` reads
the half of the draw the fuzz harness fuses, the same way the harness reads
it. The fuse is a union under `Policy::tolerant` with working size 100. It
answers with two solids. Their volume is `1999.9999999999993`, against
`1999.9999999999995` for the two blocks before the fuse. That was read with a
temporary print and taken back out, and the test holds them to within `1e-9`
of each other, relatively. A sweep of the answer under
`Policy::certified()` passes and records nothing.

### 345.5 The tests, and what fails without the change

- `a_tolerant_union_of_two_blocks_apart_answers_with_the_two_of_them` takes
  the place of `a_meet_further_from_a_corner_than_rounding_reaches_is_not_folded_into_it`,
  in the same file, and holds the harness's `ok: 1, refused: 0, solids_made:
  2`.
- `the_two_blocks_unite_with_their_volumes_and_pass_a_certified_sweep`, 345.4.
- `a_meet_is_folded_when_its_tighter_box_lies_inside_the_corners_room_however_far_cramers_reaches`,
  in `arrange.rs`'s `tests` module. On the cut `x = y` at the unit cube's
  corner, a meet whose Cramer box reaches a thousand rooms is folded when the
  tighter box reaches the room's edge, and is not when it reaches one step
  past it, or when it is Cramer's.
- `a_meet_of_two_walls_nearly_parallel_is_boxed_to_a_few_ulps_and_still_holds_it`,
  345.1.

With the room test reading Cramer's box again (`place.enclosure()` in
place of `boxed_tight.enclosure()`), `mise exec -- just t -p rcad-topo
--no-fail-fast -E 'test(/a_meet_is_folded_when_its_tighter_box|a_tolerant_union_of_two_blocks_apart_answers|the_two_blocks_unite_with_their_volumes/)'`
gave `3 tests run: 0 passed, 3 failed`.

### 345.6 What this leaves

- **Q209** waits on this row and wants a narrower box to *order* two points
  round a cap, not to fold one. Narrowing the node's box is what moved the
  ten tests in 345.2. So that row should narrow only the comparison it needs,
  and answer for those ten if it narrows more.
- **Q848** holds §395.4's 5 pyramid unions. The narrowing does not reach them,
  and narrowing the reach test as well would add more of the same kind
  (345.2).
