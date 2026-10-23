## 401. A meet of two planes and a pipe narrows, and the order round a bar's rim start is decided — 2026-10-04

*Queue row Q209, from §284.6's first item. Lane
`lane-q209-a-block-leaning-through-a-bars-rim-start`, off `ef6c6fcd`, landed as
`164b8f73`. §284.6 asked for the order round the bar's cap circle to be
decided rather than refused, naming `arrange.rs` or `three_surfaces.rs` as the
owner. This closes it: the meet is narrowed by one checked step, the order
asks the narrowed box only where its own box cannot say, and all six fuses of
284's turned block answer, under both policies, within 284.4's volumes by
hand.*

### 401.1 Why §345's step missed this meet

§345's `MeetOfThree::boxed_tight` matched `[Plane, Plane, Plane]` only and
returned `self.clone()` for anything else. A temporary `eprintln!` in
`place_the_meets`, run as `PROBE_Q209=1 mise exec -- just t -p rcad-topo
--test merged -E 'test(~a_plane_leaning_through_a_bars_rim_start::a_block_turned)' --no-capture`,
printed `kinds ["plane", "pipe", "plane"]` or `["plane", "plane", "pipe"]`
with `boxed_tight` equal to `place` — "same true" — in 9 of 9 calls at this
meet: the turned face, the bar's cap plane and the bar's wall, two planes and
a pipe.

Even where §345's step does narrow a meet of three planes, the narrowed box
went only to `a_corner_is_this_place`'s room test (345.2). `Arrangement::
order_along` read `self.nodes[..].point`, the unnarrowed box, so the order
round the cap circle never saw a narrower box at all. Both gaps had to close
for this meet: it is not three planes, and narrowing alone does not reach the
order.

### 401.2 One checked step onto a quadric

`MeetOfThree::boxed_tight` now also takes one checked step where the third
surface is a ball, a pipe or a cone, with two planes ahead of it in
`meet_of_three_surfaces`'s fixed order (§31.3). The quadric goes into the
third plane's row of the same linear step §345.1 already took for three
planes, using `Q(x) − Q(g) = (x − g)·(M(x + g) + 2b)` for the quadric's own
`M` and `b` and a guess `g` at the middle of Cramer's box. Both the stepped
box and the one `meet_of_three_surfaces` first hands back hold the meet, and
the narrowed box is their overlap, sound for the reason §345.1's is: an empty
overlap panics rather than answers.

At the rim-start meet this narrows the box's `x` side from
`[4.999999999999997, 5.000000000000003]` (`5.33e-15` wide, ±3 ulps of 5) to
`[4.999999999999999, 5.0]`, measured by the new unit test
`a_meet_of_two_planes_and_a_pipe_is_boxed_to_one_ulp_by_one_step_and_still_holds_it`:

```
as placed  [[4.999999999999997, 5.000000000000003], [2.51214793389404e-15, 2.5121479338940407e-15], [-0.0, 0.0]]
narrowed   [[4.999999999999999, 5.0],                [2.5121479338940403e-15, 2.5121479338940407e-15], [-0.0, 0.0]]
```

(`mise exec -- just t -p rcad-geom --no-capture a_meet_of_two_planes_and_a_pipe_is_boxed`.)

### 401.3 The order asks the narrowed box only where its own box cannot say

Each arrangement node now keeps this narrowed box. `Arrangement::
as_the_boxes_say`, the order along a crossing, asks it only at a node whose
own (unnarrowed) box leaves the sweep undecided, and even then takes only a
box that proves `Pos` or `Neg` — an `Undecidable` from the narrowed box is no
better than the one already in hand, so it changes nothing. Round a closed
crossing the narrowed boxes are swept from the start and never matched back
to it by name; a crossing that is not closed does not reach this path at all.

At the rim-start meet, §284.3's exact sweep is `−1.256e-14` and the
unnarrowed box's sweep runs `[−2.59e-14, +7.6e-16]`, which straddles zero.
Narrowed to `[4.999999999999999, 5.0]` along `x`, the sweep is decided: this
is the "narrower box for the meet" family §284.4 named and did not build.

### 401.4 The three-plane exclusion

`MeetOfThree::boxed_tight`'s match still returns `self.clone()` for three
planes where the order asks it — it is excluded from §401.3's path, not just
left at Cramer's box for the room test. Asked there too, on a probe that
PROBE_Q209_NO_PLANES (this lane) kept out, the narrowed box moved four pins,
measured on the uncommitted change before the exclusion
(`mise exec -- just t -p rcad-geom -p rcad-topo --no-fail-fast`, 2313 run,
2309 passed, 4 failed):

1. `a_plane_cut_along_a_cones_other_nappe
   a_block_below_a_frustum_is_cut_where_the_frustum_has_no_face_and_every_answer_is_right`:
   right `64 → 68`, "order along a crossing" `20 → 8`, "a point inside a
   piece of a face" `8 → 16`, `ResultIsNotManifold` `4 → 4`.
2. `a_steeper_plane_across_a_frustum_answers_right_or_refuses`: right
   `68 → 80`, "order along a crossing" `12 → 0`.
3. `a_corner_with_four_planes_round_it
   a_pyramid_whose_apex_meets_miss_even_its_room_is_still_refused`: a union
   at fold 1/4096, 4 high, turned 0.3 about `(3, −1, 2)`, refused on the
   base, was united. Its volume and log were not checked.
4. `a_narrow_lune_from_a_pole_corner
   a_wedge_cut_with_the_turned_knife_first_is_refused_by_name`: from its
   refusal by name to `Undecidable` on "a point inside a piece of a face".
   Whether that is a worse answer needs a §16.10 argument this lane did not
   make.

With the three-plane case kept out of §401.3's path (`PROBE_Q209_NO_PLANES`,
the committed rule), all four pass, and the whole workspace gave `2953 of
2953`. Letting three-plane meets' narrowed boxes be asked by the order too is
row Q866, queued rather than built here, since checking each of the
four new answers against its volume by hand, and arguing the fourth under
§16.10, is more than this section's row.

**2026-10-05, queue row Q866 (`9b57019e`): the exclusion above is lifted.**
`Arrangement::as_the_boxes_say` now asks the narrowed box for a meet of three
planes too, worked out by `Arrangement::narrowed_box_of` from the node's own
box and its three surfaces (`made_of`) only when the order asks, rather than
kept on the node — keeping it on every three-plane meet was what Q990
(`bd081312`) had already taken out for the 1x39 pin header's cost. By then
Q901's `as_the_planes_say` and Q1008 had already moved three of the four pins
above on their own: `a_block_below_a_frustum_...` and both steeper-plane
tallies stay at the counts this lane measured for them, and the wedge still
refuses by name, so none needed the §16.10 argument this section deferred. The
fourth, the pyramid family, moves instead on a different case than item 3
measured: a pyramid folding 1/4096 through its apex is united, checked
against its volume by hand and against the corners it was built with.

### 401.5 The six fuses, both policies

On `164b8f73`, under `Policy::certified()`, each of 284's six fuses of the
turned block (union either way round, intersection either way round, bar
less block, block less bar) is one solid, `Model::validate` passes, the log
is empty, and the measured volume lies inside §284.4's `volume_error` of the
volume by hand:

| fuse | measured | by hand |
|---|---|---|
| union, bar first | `64701.78853110312 ± 2.64139200928866` | `64702.06483006411` |
| union, block first | `64701.788531103346 ± 2.64139200928866` | `64702.06483006411` |
| intersection, bar first | `83.28528001191573 ± 0.49632624874087766` | `83.33333333333333` |
| intersection, block first | `83.28528001191575 ± 0.49632624874087766` | `83.33333333333333` |
| bar less block | `701.7885311031195 ± 2.641392009288458` | `702.064830064115` |
| block less bar | `63916.7144984306 ± 0.49632664752785616` | `63916.666666666664` |

(`mise exec -- just t -p rcad-topo --test merged -E 'test(~a_plane_leaning_through_a_bars_rim_start::a_block_turned)' --no-capture`.)

The same six numbers come under `Policy::tolerant(100)`, with an empty log:
on this base the order round the cap circle is proved, so the meet is never
folded into the rim start, and the check renamed itself from
`under_a_tolerant_policy_the_turned_block_is_ordered_and_its_faces_disagree_on_a_corner`
to
`under_a_tolerant_policy_the_turned_block_answers_all_six_fuses_with_one_solid_each_too`
(`mise exec -- just t -p rcad-topo --test merged -E 'test(~a_plane_leaning_through_a_bars_rim_start::under_a_tolerant_policy)' --no-capture`).
On the base `ef6c6fcd`, all six certified fuses refused `Undecidable` on
"order along a crossing", and all six tolerant ones refused
`ResultIsNotManifold { count: 1 }`.

§284.8 later found this same base already clear of 284.7's fold for the
turned block: since the order is proved, the policy is never asked at this
meet, so 284.7's gap (a node carrying only one of the two names a fold
drops) cannot fire here and needed Q656's own fix for the shapes it still
reaches.

### 401.6 §284.6's first item is done

§284.6's first bullet, "order the meet and the rim start round the cap
circle", is done by `164b8f73`: the check it named,
`a_block_turned_to_lean_through_a_bars_rim_start_cannot_order_two_points_round_its_cap`,
is renamed to
`a_block_turned_to_lean_through_a_bars_rim_start_answers_all_six_fuses_with_one_solid_each`
and passes. §284.6's third bullet, that §39.4's table should gain a row when
this lands, is answered in §401.7 instead: the existing row is corrected
rather than a new one added, since the refusal itself is gone at this shape.

### 401.7 §39.4: this cause of "order along a crossing" is gone

§39.4's curved-arrangement table carried a row: "a block turned by −π/4 to
lean through a bar's rim start — a second cause, met ordering the meet and
the rim start round the cap circle | gap | ...(Q209, §284.6)". That row's
demonstration, `a_block_turned_to_lean_through_a_bars_rim_start_cannot_order_
two_points_round_its_cap`, is the test §401.6 renamed to an answer. This
cause of "order along a crossing" is closed by this section: a meet of two
planes and a quadric now narrows by one checked step, and the order reads
that narrower box exactly where the unnarrowed one could not decide it. The
row is struck from §39.4's live list by this note; the table's text itself is
`039-*.md`'s, outside this lane's write set, so the correction is recorded
here and §39.4 should drop the row rather than keep citing a test that no
longer refuses.
