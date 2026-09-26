## 281. A meet of three planes boxed tight enough to order — 2026-09-26

*Queue row Q122, from §253.7. Lane `lane-q122-meet-boxed-tight`, off
`689fe3a3`. The question was whether tightening the box
`meet_of_three_surfaces` puts round three planes would make the tolerant union
§253 now refuses answer again, with two solids. **The box can be tightened, and
soundly: from `2.9e-10` wide to `8.9e-16`, and the exact meet is inside it.
But tightening it inside `meet_of_three_surfaces` does not give the union back.
It moves the refusal to another face and breaks nine other tests.** Tightening
only the meets `place_the_meets` asks for gives the union back, and still moves
eight tests. So the fix is in `rcad-topo`, which was not this lane's to change.
No code landed. The pin in
`a_meet_folds_into_a_corner_only_as_far_as_rounding_reaches.rs` is where §253
left it.*

*Landed by the dispatcher, not a merger. The lane stopped as blocked, so no
verifier re-ran any number below; each is the lane's own, with the command or
probe it names. Q122 is dropped here and its work goes on as Q202, whose write
set is in `rcad-topo`.*

### 281.1 The box, and where its width comes from

The draw is the one beside that test. The face that refuses on the base is
face 7. Its three planes, as stored, are:

| plane | normal | offset |
|---|---|---|
| a floor | `(7.08e-40, 7.10e-40, 1.0)` | `1.41e-309` |
| a turned wall | `(0.6001586505258283, −1.5617e-6, 0.7998809875203796)` | `3.0007932526291414` |
| a second turned wall | `(0.7998809875204285, 3.0927e-6, −0.6001586505198264)` | `3.9994049376021423` |

`meet_of_three_surfaces` hands back Cramer's box from `Plane::meet_of_three`:
`[4.9999999999999964, 5.000000000000004] × [−1.4300850306439927e-10,
1.4300850306439922e-10] × [−3.53841824347351e-39, −3.5384182428431114e-39]`.
Its widest side is `2.860170061287985e-10`. That was measured with a temporary
`eprintln!` in `where_three_planes_meet` while the test ran.

Solved in exact rationals from the stored bits (a Python `Fraction` script),
the three normals have a determinant of `3.105334300647015e-6`. The meet is
`(5 + 1.75e-39, −1.1394645151489238e-33, −3.538418243283311e-39)`, which is
`1.14e-33` from `(5, 0, 0)`. §253.7's `1.54e-6` was not derived again here.

**Both legs of the brief's hypothesis held.**

- (a) The planes are stored `f64`, so the meet is an exact rational point, and
  the script found it.
- (b) The box is Cramer's rule summed in intervals
  (`meet_of_three_planes_at_this_scale` in `rcad-predicates/src/planes.rs`).
  Nothing checks the answer against the planes. Each numerator is a small
  number built out of products near `1`, and every one of those products
  rounds. So the box is those roundings divided by `3.1e-6`. That is how badly
  the rounding *could* have gone, and it says nothing about where in the box
  the meet is.

### 281.2 The fixes weighed, and the one that works

- **An exact rational solve**: the Cramer numerators and the determinant as
  `Exact` expansions, with one division at the end. Not taken. The floor's
  offset is a subnormal, `1.41e-309`. A product of that entry is under
  `product_is_exact`'s `2⁻⁹⁶⁹` threshold, so `Exact::times` refuses on the very
  draw this row is for.
- **A better-conditioned formula**: the fallback in `where_three_planes_meet`
  already crosses the line of two planes with the third, picking the best
  minor. It divides by the same small determinant, so the same width comes
  back.
- **One checked step from the middle of the box: the one built.** Take
  `guess`, the middle of Cramer's box. For each plane, work out
  `r = offset − normal·guess` **exactly** (a sum of products of two stored
  numbers, with the subnormal offset entering as an entry and not as a
  factor). Then solve for the step
  `(r₁ (n₂ × n₃) + r₂ (n₃ × n₁) + r₃ (n₁ × n₂)) / det`. The cross products and
  `det` are exact and then read as enclosures, and the sum is in intervals.
  Every term of the numerator now carries a factor of `r`, and `r` is only as
  large as the guess is off, so the rounding that made the first box wide is
  now a rounding of something small. The answer is the overlap of
  `guess + step` with Cramer's box.

  **It is sound.** `r` is exact and its enclosure holds it. Everything else is
  interval arithmetic over values that hold their true numbers. So
  `guess + step` holds the meet, and so does Cramer's box, and the two overlap
  at least there.

  On this draw it gives `[5.0, 5.000000000000001] × [−1.1394645277447636e-33,
  −1.1394645047858895e-33] × [−3.5384182432833117e-39, −3.538418243283311e-39]`,
  with sides `8.88e-16`, `2.30e-41` and `6.5e-55` wide. The exact meet is
  inside it, which the script checked in rationals.

No tolerance is involved (§17): the step is a proof, not a threshold.

### 281.3 Why it did not land

**Wired into `where_three_planes_meet`, which is every caller:** the draw no
longer refuses on face 7. That meet now folds into the corner. Then face 12
refuses with `CannotTellTheCellsOfAFaceApart`. `bake.rs` derives face 12's
corner at `(5, 0, 0)` through `meet_of_three_surfaces` too. With the narrower
box, `passes_through` decides that corner is off the fifth and seventh
crossings, where before the tolerant policy read an `Undecidable` as on them.
So face 12 is left with six distinct nodes within `1.3e-16` of each other, a
cluster the arrangement cannot sort into cells. Those nodes are the meets of
crossings 0–1, 0–5, 0–7, 1–5, 1–7 and 5–7, printed with a temporary
`eprintln!` in `place_the_meets`.

`just t -p rcad-geom -p rcad-topo --no-fail-fast`, 2030 tests: **9 failed.**

- `fuse::bake::tests::re_deriving_the_corners_of_a_turned_cube_gives_boxes_that_hold_them`:
  "the box 1.1139430368449108..1.113943036844911 does not hold the stored
  1.1139430368449106". The narrowed box holds the meet of the three stored
  planes and not the stored corner, which lies ulps off them. Cramer's slack
  was covering that gap, and the test leans on it.
- `fuses_that_must_agree::a_solid_with_a_copy_of_itself_taken_away_is_nothing_at_all`
- `an_edge_with_one_face_along_it::how_often_one_pair_of_turned_boxes_leaves_such_an_edge`
- `the_turned_pair_whose_union_lost_a_face_one_way_round`: both tests
- `a_narrow_lune_from_a_pole_corner::a_wedge_cut_with_the_turned_knife_first_is_refused_by_name`
- `a_plane_cut_along_a_cones_other_nappe`: both tallies
- the Q122 pin itself, on the face 12 refusal above

**Wired only into `place_the_meets`** (a probe: a flag set around that one
call, in a temporary build): the draw answers, `ok: 1, solids_made: 2`, and
the fuse's own sweep accepts it under the tolerant policy. The same run: **8
failed**.

- The Q122 pin, now answering.
- `an_edge_with_one_face_along_it`'s count went from 104 to 207.
- `fuses_that_must_agree::turning_a_pair_makes_what_the_two_share_depend_on_which_one_comes_first`
  moved from `Solids(2)` to `NotASolid`.
- `the_turned_pair_whose_union_lost_a_face_one_way_round`: both tests.
- `a_narrow_lune_from_a_pole_corner`'s wedge.
- The two frustum tallies. One of them improved: right answers went from 40
  to 48 of 96.

Neither the total volume of 2000 nor a sweep under `Policy::certified` was
checked on that answer. The sweep that ran was the harness's, under the
policy the fuse ran with.

A third probe narrowed only boxes wider than `1e-12`, and the draw answered
the same way, `ok: 1, solids_made: 2`. **That one is not a fix.** A width
threshold is a tolerance, and it was run only to show that the answer comes
back when this one meet folds while face 12's corner keeps its Cramer box.

### 281.4 What this leaves

The refusal is still a **gap**. The meet exists, a sound box a few ulps wide
holds it, and the old answer comes back when that box reaches the fold. What
stops it is not `meet_of_three_surfaces` but how `rcad-topo` uses a box.

- **A narrower box of a meet is not a narrower box of a stored corner.** The
  bake test above is that fact, measured. So narrowing has to stay out of
  `bake.rs`'s derivation of corners, or that derivation has to grow its box
  to hold the stored corner as well.
- **Under a tolerant policy, whether nearby points are one point is decided by
  whether their boxes overlap.** Narrow the boxes and fewer points are merged,
  even at `1e-22` apart, against a budget of `1.42e-12`. That is what face 12
  shows. `merge_points_that_are_one_point` asks the policy only about boxes
  that touch, so a cluster of distinct points far inside the budget is never
  asked about.
- The next row belongs to whoever owns `crates/rcad-topo/src/fuse/arrange.rs`.
  It would narrow the meet in `place_the_meets`, or only inside
  `a_corner_is_this_place`'s containment test, and then answer for the eight
  tests that move. The step in 281.2 is about thirty lines and is written
  down there in enough detail to rebuild. It belongs in `three_surfaces.rs`
  as its own entry point, or `pub` in `rcad-geom`, once that caller exists
  (§13).
- `a_corner_is_this_place`'s doc comment says the fuse refuses "with the meet
  kept as a node". It is still true, and it should move with the row that
  closes this.
