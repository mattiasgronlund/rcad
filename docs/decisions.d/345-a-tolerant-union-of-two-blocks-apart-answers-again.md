## 345. A tolerant union of two blocks apart answers again — 2026-09-28

*Queue row Q429 (Q122, then Q202). Lane
`lane-q429-meets-boxed-tight-in-place-the-meets`, off `82f20827`. **Not
finished: the lane stopped where its brief says to stop, on a pin outside its
write set.** §281.2's checked step is rebuilt and wired into
`place_the_meets` alone, as §281.3's second probe was. It also moves both pins
in `a_corner_with_four_planes_round_it.rs`, a file §281 predates and this row
does not own. One of the two moves is a right answer lost, and a change to the
fold in `arrange.rs` gives it back. The other is the gap that file pins
closing, and it cannot be turned round from here. So the row goes out again
with that file in its write set. The eight moved tests §281.3 lists have not
been answered for yet.*

### 345.1 What is built

`MeetOfThree::boxed_tight` in `crates/rcad-geom/src/three_surfaces.rs`. It
calls `meet_of_three_surfaces` and, where all three surfaces are planes,
narrows the box round their meet by §281.2's one step. The step starts from
the middle of the box, takes `offset − normal · guess` per plane and the cross
products and determinant as `Nearly` (exact, or exact but for what falls off
the bottom of the range, which the floor's subnormal offset needs). The sum
is in intervals, and what comes back is where the stepped box and the first
box overlap. An empty overlap panics, because both hold the meet. Where a
product overflows, or the determinant's enclosure reaches zero, the first box
comes back as it was. The planes are narrowed in `meet_of_three_surfaces`'s
fixed order, so the box does not depend on argument order (§31.3).

It is an associated function on `MeetOfThree` and not a free function beside
`meet_of_three_surfaces` because `rcad-geom/src/lib.rs`, which re-exports the
free ones, is not in this row's write set. `MeetOfThree` is already exported.

`a_meet_of_two_walls_nearly_parallel_is_boxed_to_a_few_ulps_and_still_holds_it`
takes a floor and two walls `1e-6`, `1.5e-6` and `3e-6` off parallel. Cramer's
box is wider than `1e-12` and the narrowed one narrower than `1e-14` and inside
it. Each side of the narrowed box is checked to hold the meet exactly: along
an axis the meet is `D / det`, so it is inside `[lo, hi]` when `D − lo·det` and
`hi·det − D` both have `det`'s sign. A box `1e-12` past the narrowed one fails
that check, so the check can fail. `just t -p rcad-geom
a_meet_of_two_walls_nearly_parallel`: passes.

**One step is enough at these angles and not at every angle.** The step's
error is the first box's width times about `ε / det`. At `det ≈ 3e-6` that
factor is `3e-11`. At `det ≈ 1e-12` the first box is about `1e-4` wide and
one step leaves about `1e-8`. That is worked from the formula, not measured.
Nothing here needs more, and a second step is the same thirty lines again.

### 345.2 The pin outside the write set

`crates/rcad-topo/tests/a_corner_with_four_planes_round_it.rs` came after §281
(Q240, `034111e9`, `4b6e3e2e`). Its two tests unite a pyramid with a cube
clear of it, under `Policy::certified()`, 36 ways each. The pyramid's base is
bent at one corner by a fold, so four planes meet at its apex.

- **A fold of 1/16**, pinned as all 36 answered with two solids.
- **A fold of 1/64**, pinned as all 36 refused on "order along a crossing". Its
  doc calls that a gap and says the change that closes it turns the test
  round.

Measured with a probe: a copy of that file, untracked and deleted after, with
a test that runs both spellings, the six turns and the three axes at height
one and counts. An answer counts as right when it has two solids, a volume
within `1e-9` of the pyramid's plus one, an empty log, and passes
`Model::validate` under `Policy::certified()`. Every answer in both rows below
was right.

| Fold | Narrowed, fold as on `main` | Narrowed, fold without the overlap |
|---|---|---|
| 1 | 33 of 36 | 36 of 36 |
| 1/4 | 20 | 36 |
| 1/16 | 12 | 36 |
| 1/64 | 2 | 36 |
| 1/256 | 1 | 25 |
| 1/4096 | 0 | 4 |

The file's own table, from §301.6's probe over 108 per fold with three
heights, has `main` at 108, 103, 99, 0, 0 and 0. That table was not re-run
here.

**The first column is a right answer lost, 24 times of 36 at the pinned fold.**
The refusals named "two regions of one face could not be told apart" (10 at
1/16) and "order along a crossing" (14). `a_corner_is_this_place` folded a meet
into a corner only when the meet's box **touched the corner's own box** and
lay inside that box grown by `what_writing_the_corner_down_cost`. Cramer's box
was wide enough to touch whatever it lay near. A box a few ulps wide round the
meet of one face's three planes need not touch the apex's box, which `bake`
solved from another triple. So the meet stayed a node beside the corner.

That reading is inferred from the tally moving and was not traced meet by
meet. What was measured is its consequence: with the touch taken out of the
fold, and the room test left as it was, the second column. `just t -p
rcad-geom -p rcad-topo --no-fail-fast` then passes the 1/16 pin again, and
moves nothing that the narrowing alone did not already move.

The touch is not what §253 argued for. §253's argument is that inside the
room a meet and a corner are one point written down two ways. The touch is
left over from the fold before §253, which folded on a touch alone. So this
lane keeps the change, and `a_corner_is_this_place`'s doc says why.

**The second column turns the 1/64 pin round: 36 of 36 answer, all right.**
That is the gap closing, which the file asks for. But the file is not this
row's, so the pin stays where it was and the lane stops.

### 345.3 What moves, and what is not answered for

With both changes, `just t -p rcad-geom -p rcad-topo --no-fail-fast` fails
nine tests: the 1/64 pyramid above and §281.3's eight.

- the Q122 pin in `a_meet_folds_into_a_corner_only_as_far_as_rounding_reaches.rs`
- `an_edge_with_one_face_along_it::how_often_one_pair_of_turned_boxes_leaves_such_an_edge`
- `fuses_that_must_agree::turning_a_pair_makes_what_the_two_share_depend_on_which_one_comes_first`
- both tests in `the_turned_pair_whose_union_lost_a_face_one_way_round.rs`.
  The first fails with "the ell against the comb answers", so the ell and the
  comb now refuse where they answered.
- `a_narrow_lune_from_a_pole_corner::a_wedge_cut_with_the_turned_knife_first_is_refused_by_name`
- both frustum tallies in `a_plane_cut_along_a_cones_other_nappe.rs`

**None of the eight was answered for**, and no pin in them moved. §281.4's
explanation, that narrower boxes merge fewer points, was not checked against
any of them. The Q122 draw's volume and a certified sweep of its answer were
not run either. Only the pyramid rows above are measured.

The fold change bears on the eight. A test §281.3 saw move might now move
back, or differently. §281.3 measured without it, and this lane did not
measure them one by one.

### 345.4 What the next row needs

- `a_corner_with_four_planes_round_it.rs` in the write set. Its 1/64 test
  turns round, and its doc's table and paragraph move with it.
- A per-test answer for the eight, as Q429's brief asks: check first whether
  each moved test's change comes from two points that used to merge now
  staying apart, and then whether those points really are distinct.
  `turning_a_pair_…`'s `NotASolid` and `an_edge_…`'s 104 → 207 are the two
  that look worst.
- Whether the 1/256 and 1/4096 refusals are the step running out (§345.1) or
  something else. The narrowing helps them from 0 to 25 and 4 of 36, and does
  not finish them.
