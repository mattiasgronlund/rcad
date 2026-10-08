## 229. The joiner lands, the torus pins take their real refusal, and five relay files each name a defect — 2026-09-25

*§0's next row was the joiner (§224.2). Two batches of lanes went out from
worktree `dispatch-next`: the first off `da8073ae` built the joiner and
§222.2's two reader gaps, and diagnosed §224.4's two asymmetries without
committing; the second off `e68aa416` moved the torus pins, re-ran the cone and
its copy on the joiner, tested the reader's new checks from STEP files, and read
five relay files whose refusal moved. This section records what landed and what
each lane left.*

  229.1   What landed, and the gate it landed on
  229.2   The joiner: three of §224.2's four items, and why not the third
  229.3   The two torus pins were refusing at a corner put on the wrong curve
  229.4   The reader checks a closed ellipse on a pipe and a circle on a cone or torus
  229.5   Five relay files, each stopped by a defect that was already there
  229.6   A cone and its copy on the joiner: two steps done, not merged
  229.7   What this leaves

### 229.1 What landed, and the gate it landed on

Onto `main`: `ff78ab00` and `f2fc7d1d` (the joiner and a corner rule),
`0ceae266` (their tests) and `e68aa416` (a wildcard named), `825f15c4` through `ab032853` (the
reader's new checks and their tests), `ce16afdd` (the torus pins) and
`283d8dd3`/`e5f2c109` (a ball resting in a cone). `ignored_tests` is 15.

The merged code, `ba401e5d` (with `main`'s `0c1346f9` merged in), gated
**PASS, 2382 tests, 15 skipped**, with this section beside it uncommitted.

`lane-land-cone-and-copy` is **not** in it (§229.6). Neither is
`lane-step-closed-circles-blocked` (`f886614e`), which holds two STEP tests that
fail for reasons outside the reader (§229.4).

### 229.2 The joiner: three of §224.2's four items, and why not the third

`ff78ab00` builds items 1, 2 and 4. Two curved faces with the same surface bits,
used the same way round, are one face.

> **2026-10-05 (§407): closed for pipes.** The rule above is now the folded
> one for a pipe — the same bits are no longer required, only the same point
> set and the same `used`. A cone still needs the same bits, as this section
> built it.

> **2026-10-08 (Q1380): a ball is joined too.** §224.2 item 2 designed the
> curved joiner for a cone or a cylinder and this section built only that, so
> a ball's pieces stayed apart by omission, not by a decision. Now two pieces
> of one ball used the same way round are one face, and a ball's group keeps
> no seam: the mesher opens a ball's face along meridians of its own (§202.1,
> §402). A group that makes up the whole ball is left apart, since a joined
> face is never written as a whole closed surface. A ring's pieces are still
> never joined.
>
> A ball of radius 2 united with the box `[0.5, 3]³`, either way round, and
> the ball less the box, keep its skin as one face where they kept seven
> (`a_ball_kept_to_a_corner_of_a_box.rs`). Three pins of the split moved to the
> joined answer, each with its volume checked: a block poking into a ball
> (one face of one wire), a ball cut through both poles (one face, where the
> union kept two and `bake` joined them for the next fuse), and a ball shaved
> by a turned block's face (one hole where it kept two).
>
> Two refusals moved one step on, and the row asked for neither: it asked to
> keep those groups apart so neither refuses. **Both refused before the join
> too**, by other names, so keeping them apart makes neither answer. A block
> edge grazing a ball: the union and the ball less the block were refused
> `CannotNameWhichPieceOfACrossingThisIs` writing the answer, and now all six
> fuses, three rules both ways round, are refused
> `ACurvedFaceGoesRightRoundWithNoSeam` meshing it, as the other three were
> already. The joined face is a ball less a sliver `2⁻⁵⁰` long, the same
> shape as the box's corner, so no rule of the joiner's tells them apart. A
> half frustum against a ball: the unions and the ball less the half were
> refused `TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing`, the ball on both
> sides of an edge between two of its own pieces, and are now refused
> `AWireWillNotLieFlatOnItsFace` on the joined face, whose one ring runs over
> the ball's north pole. A rule that kept a ring through a pole apart would
> tell that one apart, and would only bring back the refusal the split itself
> caused.

A band that goes right round the axis
keeps one dissolved edge as its seam — the one with the lowest corners from ring
to ring, so the choice does not depend on argument order — and a group with a
ring it cannot count, or with no such edge, is left apart. No ring of a curved
face is taken as its outline (§31.3).

It departs from item 3 on purpose: **a corner where pieces of one cone met stays
a corner.** Folding the two arcs either side into one made a rim arc longer than
a half circle, whose centre no picker can keep behind it (§224.1). Measured: the
second union refused `AFaceTrimmedByACurveThisCannotFollow` on the frustum's top
rim.

With it, block ∪ frustum and block ∪ bar each carry their curved surface as one
face, validate, and measure within their own error of the volume worked out by
hand. `a_block_and_frustum_united_can_be_united_again` answers and is no longer
ignored; so does the far-cube associativity trio. `(block ∪ bar) ∪ cube` refuses
`AFaceTrimmedByACurveThisCannotFollow` and is ignored as a gap (`0ceae266`).

`f2fc7d1d` was needed on the way. A plane along a cone's axis crosses it in two
lines, and a wall's corner on the cone was counted as on both, because the model
lists the cone round that corner. Two ways out along one cone then made a turn
nobody could decide: the joined `(block ∪ frustum) ∪ cube` refused `Undecidable`
on `orient3d` at (2.29, 1, 5). Now only the picker of each crossing is asked.

### 229.3 The two torus pins were refusing at a corner put on the wrong curve

`f2fc7d1d` moved two torus tests from `TwoCurvesTouchWithoutCrossing` to
`Undecidable`, and left them failing. The lane that took them (`ce16afdd`) showed
the old name was wrong, with debug output rather than a guess: the plane y = 0
cuts the torus in two circles; the end cap's corner (3, 0, 0) is on the near one;
the far circle's picker answers Neg there, in all eight corner-and-crossing pairs
of both bends. Under the old rule, put back behind a debug switch, both tests
refused at exactly that corner, between the cap edge's own circle and the far
one. The verifier re-ran the base commit whole and got the new refusals' exact
figures.

Both new refusals are gaps (§16.10), named and not built:

- **A bend fused with its copy**: "a point inside a piece of a face" on the end
  cap disk. The only point tried is the disk's one corner, on its rim. (4, 0, 0)
  is inside the piece, certified. `the_middle_of_a_round_wire` in `select.rs`
  has no torus arm; a plane through the axis crosses the ring at two points, and
  both should be offered.
- **A bend built face by face**: "side of plane" in `which_side_of_everything`,
  at a point on the far circle of the plane x + y = 0, asked about the cap circle
  of that same plane, bit for bit. The answer is exactly zero, which a box
  5.7e-13 wide cannot certify — but the point's own list of crossings already
  says it. Untested worry passed on: `group_the_cycles_into_cells` assumes a
  closed curve cuts the surface in two, and one circle round the tube does not
  cut a torus in two.

### 229.4 The reader checks a closed ellipse on a pipe and a circle on a cone or torus

`825f15c4` closes §222.2's two gaps. `every_face_holds_the_curve` now bounds an
ELLIPSE against a pipe in closed form, and a CIRCLE against a cone and a torus.
A torus asks its two families of circles the way the walk does. Each bound is
held by a test (§15.6): 150 draws of each kind of curve, moved, grown, tilted
and turned, 720 points each, none past the bound plus four roundings. The corpus
classifier's floor moved from 265 to 516 of a measured 688. The corpus counts
did not move: nist 1 of 44, KiCad 161 of 323.

A lane then wrote the STEP files §222.2 asked for (`lane-step-closed-circles`).
One lands: **a ball resting in a cone** reads, cut in two round the circle it
touches, each half named by that circle's plane; on `da8073ae`'s reader the file
is refused `ACrossingBetweenAConeAndAnotherCurvedSurface`. The other two are on
`lane-step-closed-circles-blocked` and fail on this tree for reasons outside the
reader, both shown by the lane and reproduced by its verifier:

- **A cone written twice, one ulp apart.** The reader now passes the circle and
  names it by its plane, but `rcad-topo`'s closed-edge check
  (`check_this_solids_closed_edges_are_walked_the_way_their_surfaces_wind`) takes
  the face across the edge as the partner, not the third surface the run names.
  The two cones' gradients cross to (0, 1.07e-14, 0), which is rounding, and the
  sign read off it is wrong: with the plane as the partner the rule answers
  Neg, which matches the lower band; with the two cones it answers Pos. It fails
  the same way on `da8073ae`, so it cannot yet tell the old reader from the new.
  §341 makes the check read the plane; the file now validates and measures
  `312π`.
- **A torus written twice.** Refused before the reader's change matters: the
  mesher answers `CornersCoincide` on a band that goes right round the ring,
  and `tests/rings/mod.rs` says the only torus face the kernel builds is a bend.

### 229.5 Five relay files, each stopped by a defect that was already there

Five KiCad relay files refused in a new place once `825f15c4` let the reader past
their first torus edge. A read-only lane checked whether any of the reader's new
plane-named curves was drawn wrongly. **None was.** Each file now stops on an
older defect. This is the lane's reading, with the probes it ran; nobody re-ran
it:

- **`Relay_SPDT_Finder_34.51_Vertical`** — the mesher's chart of a torus face. A
  corner at (-4.85, -3.45, 14.0) sits exactly on the torus's seam round the ring
  and gets the angle +π, while both its neighbours are on the -π side, so the
  boundary jumps across the chart and crosses itself. Which mesher step picks
  the branch is not traced.
- **`Relay_DPDT_Fujitsu_FTR-F1C`, `Relay_DPST_Fujitsu_FTR-F1A`** — a mitre
  ELLIPSE between two pipes of radius 0.1 fails the new ellipse check. The file
  writes the half-axis as 0.141421356237, and one pipe's axis leans 3.7e-15, so
  the edge is named by its pipe, as the old reader named every ellipse, and the
  pipe-against-pipe walk refuses `CannotNameWhichPieceOfACrossingThisIs`. Whether
  the check should pass it is not measured: by the lane's own arithmetic the two
  axes miss each other by about 1.9e-14, which is about the size of the
  allowance.
- **`Relay_1-Form-A` and `-B_Schrack-RYII_RM5mm`** — the file's only two torus
  faces, both written `.T.`, come out meshed the wrong way round: all 16
  mismatched sides touch them. A copy with both changed to `.F.` imports. The
  lane's own geometry says `.T.` is right, so the kernel turns these faces over.

### 229.6 A cone and its copy on the joiner: two steps done, not merged

§224.5 said to re-run `13d5abaa` once the joiner landed. `lane-land-cone-and-copy`
(`63281be7`) did two of three steps:

1. Took `13d5abaa` (`16a9d4c4`) and re-pinned what it moved (`647d8eed`): a bar
   and its copy answer with the bar; a bar less itself is nothing; 59 of §49.6's
   84 answer, up from 57; `ignored_tests` 15 → 11.
2. `63281be7`: a closed curve's one corner is chosen once, by what it is — an
   input vertex another kept side ends at, then any input vertex, then the
   lowest point — and every piece on that curve uses it. The flush cap (§224.4)
   answers the same either way round; the wider grid compares 118, up from 114;
   a frustum flush inside a turned taller one answers all six fuses.
3. The two spellings of one pipe: **not started.** The earlier diagnosis lane
   found two places in `assemble.rs`: `the_surface_that_cut_each_closed_rim`
   compares surfaces bit for bit, and `Edges::of` takes each side's direction
   round a closed edge from that side's own pair of surfaces. Fixing both made
   all four cases answer with the right volume, in a trial.

Not merged, because its tip is not green: rustfmt fails on its own code; the
three two-spellings tests stay red; `two_overlapping_parallel_bars_stop_in_select_whichever_way_round`
is red (two bars side by side now refuse `ResultIsNotManifold` in five of six
fuses); and absorption with A the cone finds one extra corner, four times, for
example at (3, 1.5, 0), where the block's floor cut the cone. The verifier found
that test passes on `e68aa416` and fails on the lane's tip; **whether the lane's
change made the corner or the old refusal hid it is not shown.** One lead: the
earlier flush-cap lane found the extra corner is the one corner of a closed edge
between two pieces of the same cone, and §229.2's joiner keeps such a corner on
purpose.

### 229.7 What this leaves

- Finish `lane-land-cone-and-copy`: rustfmt, the two spellings (§229.6 item 3),
  the absorption corner, the two bars. Then `lane-swallowed-pointed-cone`
  (`184962b9`), which nobody has re-run.
- §229.3's two torus gaps: a torus arm for `the_middle_of_a_round_wire`, and
  reading zero off a point's own crossings in `which_side_of_everything`.
- §229.4's cone written twice: the closed-edge check should take the surface
  the run names as the partner. The mesher and the fuse share that rule.
- §229.5's three relay defects: a torus seam the chart puts on the wrong side,
  a mitre ellipse the check misses by about its allowance, and a torus face
  meshed the wrong way round.
- §39.4's catalogue should say the two torus pins now refuse `Undecidable` and
  why; `§37`, `§39` and `§31` cite test names `lane-land-cone-and-copy` renamed.
