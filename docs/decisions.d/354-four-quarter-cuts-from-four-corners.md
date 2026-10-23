## 354. Four quarter cuts from four corners, and why a cone's apex within a rounding is not the apex yet — 2026-09-28

*Queue row Q483 (Q220 sent again with a wider write set). Lane
`lane-q483-four-quarter-cuts-from-four-corners`, off `9612b25f`. §285 found
sixteen caps of a pointed cone refused `CornersCoincide { position: 2 }`
because two of the four quarter cuts left from one corner, and showed three
experiments. **Experiment A is real code now**: the cuts leave from four
different corners, in the loop's order. Twelve of the sixteen answer right, and
the other four hold every grid point and do not measure. **Experiment C, a
cone's apex within a rounding read as the apex, was built, measured, and taken
out again**: it answers two answers A leaves unmeasured and takes two answers
that measured right without it. The fold it leaves is traced in 354.4, and its
cause is outside `mesh.rs`.*

### 354.1 What changed

`cut_a_cap_up_to_its_pole` asks `corners_the_quarter_cuts_leave_from` which
corners the four cuts leave from. That function takes the corner nearest each
quarter, as before. When those four do not run in the loop's order, counted
from the first, it picks again. Each quarter after the first gets the nearest
of the corners after the one before it, and leaves one corner for each quarter
still to come. That is §285.6's experiment A, with two differences. It also
picks again when the four are different but out of order, since stretches out
of order overlap as badly as a repeated corner leaves no width. And the pick
keeps `nearest_corner_off_the_rims_poles`'s tie-break for a corner off the pole.

**When the nearest corners already run in order, the choice is unchanged**,
and the constrained pick would give the same corners anyway: each one is the
first nearest corner overall, and it lies inside the range the constrained pick
searches. A cap of fewer than four corners has no four to give, and it goes on
to the triangulator as before.

### 354.2 The two tables

The eighteen of §285.1, by
`mise exec -- just t -p rcad-topo --test merged -E 'test(=meshing_a_plane_cut_along_a_cones_other_nappe::the_eighteen_held_to_the_familys_four_checks)' --no-capture`:

| tree | right | the rest |
|---|---|---|
| base `9612b25f` | 0 | 16 `CornersCoincide { position: 2 }`, 2 fuse folds `{4, 4}` |
| this lane | 12 | 2 measure at 0.02 and refuse `BoundaryCrossesItself` finer (lean 1/4, crossing 0, turn 1); 2 measure at no rung, `ASideCutsOffOneCornerOfACurvedPatch` (lean 3/8, crossing 0, turn 1); 2 fuse folds `{4, 4}` |

The base row is §285.6's "none" row exactly, so Q412, Q205 and Q457 moved
none of the eighteen. This lane's row is §285.6's "A" row exactly.

The family pin,
`a_pointed_cone_with_its_apex_near_a_steeper_face_answers_right_or_refuses`,
over all 96 fuses:

| tree | right | refused |
|---|---|---|
| base | 64 | 16 `CannotMeshAFace`, 16 `CannotNameWhichPieceOfACrossingThisIs` |
| this lane | 76 | 4 `the answer does not measure: CannotMeshAFace`, 16 `CannotNameWhichPieceOfACrossingThisIs` |

The four that do not measure are the intersection and the block less the cone
at crossing 0, turn 1, both leans. They are the four §285.3 says the steep
face misses the apex by a rounding. `fuse_and_check` panicked on a refusal it
did not know, so `REFUSALS` gains `the answer does not measure: CannotMeshAFace`.

### 354.3 Why the apex half does not land

Experiment C gave `the_pole_a_point_sits_within_a_rounding_of` a cone arm: a
point within one `POINT_ALONG_A_CROSSING_ROUNDOFF` of the apex is the apex. It
was written as real code, with a `match` over the surface, and measured
together with A. Experiment B, the coarse fold's retry in `validate.rs`, is
Q221's and was not applied, so this is the A + C row §285.6 never measured.

| | harness (the eighteen) | family (96) |
|---|---|---|
| A | 12 right, 4 do not measure, 2 fuse folds | 76 right, 4 do not measure |
| A + C | 14 right, 2 do not measure (fold `{4, 4}` at every rung), 2 fuse folds | 76 right, 4 do not measure (fold) |

The family row is the whole story. C answers the two blocks less the cone at
crossing 0, turn 1. **It also takes the two cones less the same two blocks**,
which answer right under A alone and on the base. Under A + C they refuse
every measure at 0.002 with `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 2,
used: 3 }`. So C trades two answers that measure for two others. The harness
could not show this, because it holds only the intersection and the block less
the cone. The trade was found by printing each answer that did not measure
inside `fuse_and_check`, with C switched off by a variable for the A row. Both
the print and the switch were taken out.

So `the_pole_a_point_sits_within_a_rounding_of` still answers only for a ball,
and its doc comment says why. C is not wrong: a point a rounding from the apex
is the apex. But landing it now loses two right answers. It should land with
the fix for the fold in 354.4, which is what makes both pairs fold.

**2026-10-04:** Q846 (Q441's branch, cap cuts leaving only from corners whose
meridian meets the rim nowhere else) answers the two lean-3/8 caps at crossing
0, turn 1 of `meshing_a_plane_cut_along_a_cones_other_nappe.rs`, without
experiment C. Their old first cut ran down the ruling the rim comes up by,
along the rim's own last side, which is not §285.3's quarter-turn strip: cuts
still land on the near-apex stretch and the mesh answers. The lean-1/4 pair now
refuses `ASideCutsOffOneCornerOfACurvedPatch` at 0.01, 0.005 and 0.002, not
`BoundaryCrossesItself`
(`mise exec -- just t the_eighteen_held_to_the_familys_four_checks --nocapture`).

### 354.4 The fold, traced

The case is the intersection at lean 3/8, crossing 0, turn 1, under A + C,
measured at 0.05. A throwaway test fused it and measured it. A print in
`finish` listed every side that was drawn inside a patch and not used exactly
twice, and a print in `hand_over` listed each patch's corners.

- **The answer has three faces**: the cap `z = 8`, the cone's face `10v1` and
  the steep face `11v1`. The cone's face is handed to the triangulator as
  **one** patch of about 728 corners. It is not a cap under C, and at 1.478
  radians it is narrower than the quarter turn `cut_off_what_goes_too_far_round`
  cuts at.
- **The reported side is `(226, 606)`**. Its two ends are at the same height,
  `z = 1.9153216989388e-8`, equal to 14 digits. One is on each arm of the
  crossing, where the steep face passes a rounding above the apex and the
  crossing turns back on itself round it. The cone's face draws triangles
  `[226, 227, 606]` and `[606, 853, 226]`. The steep face draws
  `[227, 226, 606]` and `[226, 225, 606]`. The first of each pair is **the same
  triangle**, facing opposite ways. That is §187.4's flat pocket.
- **It is not one side.** At 0.05, 209 sides fold. Every one has both ends
  below `z = 1.92e-8`, and the longest is `1.27e-8`, which is the top rung
  `(226, 606)`. At lean 1/4 it is 202 sides, all below `1.58e-8`. At 0.002 on
  lean 3/8 it is 932 sides, all below `3.5e-9`. The folds are rungs of a ladder
  across the narrow strip between the two arms, just above the apex, and both
  faces draw the same rungs.
- **Where the rungs' ends come from.** The crossing is walked by
  `rcad_geom::walk`, called from `BuildingAMesh::walk` in `mesh.rs`. It puts hundreds of points down
  each arm towards the apex. They form a geometric run, each about 0.91 of the
  height of the one before: corners 227 to 295 of the cone's patch go from
  `2.1e-8` down to `4.2e-11`. The two arms are mirror images point for point.
  Across a strip that narrow, the cone's face lies within any deflection of
  the plane. So neither face gets a point of its own inside the strip, and
  each triangulates it from the arms alone.

**The step that makes the fold is the triangulator, once in each face**
(`rcad-mesh/src/triangulate.rs`, called from `hand_over`). It has a rule for
this class: `corners_cut_off_by_one_side` (§187.4) never lets a curved patch
draw a side that cuts off **one** corner inside an edge. A rung joins two
corners of one edge that are far apart along it, and nothing stops a curved
patch drawing that. Two families of fix would stop it, both outside `mesh.rs`.
One widens §187.4's rule, in `rcad-mesh`, from a side that cuts off one corner
to any side between two corners of one edge. That rule has to be told which
corners share an edge. The other stops the walk in `rcad-geom` putting mirror
points down both arms to `1e-16`. The first is the one aimed at the fold
itself. The second only makes it less likely. Neither is in this lane's
files, and neither was tried.

The frustum's two fuse folds (§285.4) are a different fold. They happen at the
coarsest mesh, where each arc is two chords, and experiment B retries them.
They are Q221's.

### 354.5 What else moved, and what did not

The whole `rcad-topo` suite ran under A + C before the pins moved. It was
1139 tests, and the only five that failed were in
`a_plane_cut_along_a_cones_other_nappe.rs` and
`meshing_a_plane_cut_along_a_cones_other_nappe.rs`. The six other files that
pin `CornersCoincide` did not move. That includes the ball caps, which
`cut_a_cap_up_to_its_pole` also cuts, and `a_block_face_through_a_cones_apex.rs`,
whose `CornersCoincide` on a pointed cone's union is Q468's (§347.3, §347.4).
The nearest-corner choice is unchanged wherever it already ran in order, so
this is what 354.1's argument predicts. With A alone and the pins below
moved, `just gate quiet` passed on `5824843f`: 2705 tests run, 2705 passed.

Pins moved, before → after:

- `a_pointed_cone_with_its_apex_near_a_steeper_face_answers_right_or_refuses`:
  64 right, 16 `CannotMeshAFace` → 76 right, 4 `the answer does not measure:
  CannotMeshAFace`. The 16 `CannotNameWhichPieceOfACrossingThisIs` did not move.
- `the_mesher_refuses_sixteen_caps_of_the_pointed_cone_and_two_folds_on_the_frustum`
  → `the_mesher_refuses_two_folds_on_the_frustum_and_no_cap_of_the_pointed_cone`:
  18 mesher refusals → the frustum's 2.

  **2026-09-29:** Q221 (`14ba0254`) renamed this test again, to
  `the_mesher_refuses_no_cap_of_the_pointed_cone_and_no_fold_on_the_frustum`,
  since the frustum's two folds now answer too.
- `two_quarter_cuts_share_a_corner_exactly_when_the_kept_rim_is_under_a_third_of_a_turn`
  → `two_quarters_are_nearest_one_corner_when_the_kept_rim_is_under_a_third_of_a_turn_and_every_cap_answers`.
  The prediction is unchanged. The six lean-3/8 intersections were each
  `CornersCoincide`, and all twelve are now `right` under the four checks.
- `which_side_of_the_face_meant_through_the_apex_it_is_on_does_not_say_which_refuse`
  → `…_does_not_say_which_do_not_measure`: the same two of six, now failing to
  measure at 0.002 rather than refused by the mesher.
- `the_eighteen_held_to_the_familys_four_checks`: 16 `CornersCoincide` and 2
  folds → the "this lane" row of 354.2.

### 354.6 What this leaves

- **The rungs (354.4)**: the fix is in `rcad-mesh`'s triangulator, or less
  directly in `rcad-geom`'s walk. With it, experiment C should land, and the
  two cones less the block would check whether it still costs them anything.
  The two intersections at crossing 0, turn 1 fold under A + C for the same
  reason.
- **Under A alone, the four at crossing 0, turn 1 were not traced**: 2 ×
  `BoundaryCrossesItself` finer than 0.02, and 2 × `ASideCutsOffOneCornerOfACurvedPatch`
  at every rung. The two at lean three eighths answer since Q441 (Q846). The
  two at lean one quarter were traced by Q2037 and answer since Q2040: a cut
  left from a corner on the ruling the rim runs down, a few ulps of longitude
  off the corners below it, and the sliver between was what the triangulator
  refused (354.7).
- **The frustum's two folds**: experiment B, Q221.

### 354.7 A rim side a few ulps off a cut's meridian runs along it (Q2040)

*Queue row Q2040, from Q2037's trace. Lane
`lane-q2040-a-side-a-few-ulps-off-a-cones-ruling`, off `f24a8d05`, in four
runs, relays 0 to 3.*

**What was refused.** A block whose steep face is meant through the pointed
cone's apex is stored with that face's plane an ulp off it: at lean one quarter
turned an eighth of a turn, the offset is `8.881784197001252e-16`. So the piece
of the cone inside the block is a cap whose rim runs down near one ruling, round
the apex a rounding above it, and up near another. Down the ruling the rim is a
hyperbola, not a line, and its corners' longitudes drift by ulps from one to the
next. Q2037 printed them at lean one quarter, crossing 0, turn 1, deflection
0.01: `-0.04719755119659771` at `z = 8` and `-0.04719755119659906` at
`z = 0.78`, over the first 899 corners of a rim of 1848.

`corners_the_quarter_cuts_leave_from` takes a corner as clear when the cut up its
meridian to the apex meets the rim nowhere else, and `a_side_meets_a_cut` read a
rim side as running along a cut only when both its ends had the cut's longitude
bit for bit. The sides below a corner on the ruling lay a few ulps to one side
of its meridian and crossed it nowhere, so the corner was taken clear. The cut
from it ran down the ruling a few ulps off the rim, and the patch between was a
sliver. The triangulator refused it `ASideCutsOffOneCornerOfACurvedPatch`, since
the cut's own points already sat where it would put a point in (Q2037: corner 7,
and the patch from 7 to 899), or, at other turns and deflections, `LoopsTouch`.

**The fix.** A side whose two ends are each within
`the_walks_rounding_of_a_longitude` of the cut's longitude runs along the cut:
four ulps of the larger of the longitude and a whole turn, `3.55e-15` for a
longitude under a turn. The argument is the walk's own arithmetic.
`WalkingAWire::arrive` takes each longitude as the one before it plus the short
way round to `atan2`'s answer. `atan2` rounds by half an ulp of an answer under
half a turn, and the difference, the turns taken off and the sum each round by
half an ulp of the larger of the turn and the longitude. So one longitude
carries about two such ulps, and the difference of two about four. Within that
the chart cannot say which side of a cut's meridian a corner lies on, and a cut
that close to the rim leaves a sliver whose points are the rim's own.

**The argument is for the walk's rounding, and the cone's drift is the
plane's.** It grows as the rim nears the apex, as about `2.1e-15 / z` turned an
eighth: relay 0 printed that cap's corner 200 at `z = 9.0e-8` with longitude
`-0.2617994112445287`, `2.3e-8` off the corners at the top. So the drift passes
four ulps of a turn somewhere down the ruling, and the bound reads as along the
cut only the run of the ruling above that. On every shape here that run holds
the corners a cut was taken from. That is measured, not argued.

**The bound alone only ever says "meets" more often.** A side that crosses the
cut's longitude at a latitude on the cut has its low end on the cut, its high
end on it, or the cut's far end strictly between them, and that is what the
branch for a side along the cut asks.

**The cap's closing side moves in one addition.** `the_sides_of_a_rim` moved
the side that closes a cap's rim by `climbed` and then by the turn, so its copy
a turn back could miss the first corner by an ulp. Q1879 moved it in one
addition for `a_leg_meets_the_rim` (`296b3d05`) and not here, since three cone
tests moved under it. Q2037 found why at Q1905's shape, lean three eighths,
crossing 0, turn 1, deflection 0.002. Its first corner is the top of the rim's
run down the ruling. With the copy an ulp off, the cut from that corner met the
rim where it set off, which kept the corner from being taken clear, by accident.
Under one addition with no bound it was taken clear, and the face was refused
`LoopsTouch { corner: 1887, side: 2014 }`. With the bound the run below it reads
as along the cut, and `the_sides_of_a_rim` takes the one addition.

**What it answers.** In `meshing_a_plane_cut_along_a_cones_other_nappe.rs`,
all eighteen of `the_eighteen_held_to_the_familys_four_checks` answer right,
where the two at lean one quarter, crossing 0, turn 1 measured at `0.02` and
no finer; all six blocks of the which-side test measure at `0.002`, `(0.25,
1.0)` the one that did not; and the lean-one-quarter block turned an eighth,
refused at every deflection, and turned 2, refused finer than `0.01`, both
measure at `0.002`. In `a_plane_cut_along_a_cones_other_nappe.rs` the lean-one-quarter
family round the pointed cone is 48 right, where it was 46 and 2
`CannotMeshAFace`, so all ninety-six of that family answer.

**What else moved.** Three of the slow tier's leaning-block pins, in the same
file, each lost its last refusals: the steep cone's block leaned 1 and unturned
is 16 right, where crossing 0's union and cone less the block refused
`CannotMeshAFace`, and the unit cone's blocks leaned a quarter and three
eighths are 48 right each, where crossing 0, turn 1's intersection and cone
less the block refused `CannotMeshAFace`. Which of the fix's two steps, the
bound or the one addition, moved those six fuses is not traced. §350.5's
tallies of the two leaning-block families count them as refused, so they are
stale; a row is filed to bring them up to date. Run whole on `5d338451`,
before those pins turned, `rcad-topo` ran 2149 tests, 2146 passed, and these
three failed on their old tallies; every other test answered as it did, the
island callers of `a_side_meets_a_cut` included.

**Not taken.** Q2037's probe read a side within `1e-12` of a cut's longitude as
along it and answered the same tests, but `1e-12` was a guess, not a bound.
Relay 0 tried reading the ruling off the edge, and a bound per latitude in
space, and took neither. Relay 1 described snapping each end within the bound
onto the cut's longitude before the bit-for-bit test, which also reads a corner
a few ulps past the cut as touching it; it was never run, so it was not taken.
