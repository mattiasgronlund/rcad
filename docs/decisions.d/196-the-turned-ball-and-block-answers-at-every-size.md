## 196. The turned ball-and-block answers at every size, and a narrow lune from a pole corner is still out of reach — 2026-09-22

*One batch of three lanes, dispatched by §175's skill, workflow run
`wf_8ecf5c94-756`. `just gate quiet` run here: on `4b27349e` before the batch,
PASS, 2221 run, 2221 passed, 6 skipped; on the merge `d96d247a`, PASS, 2226
run, 2226 passed, 7 skipped. A number below is a lane's unless it names a
command run here.*

### 196.1 Two spellings of "is this a pole" disagreed, and the fuse now asks the mesh's

§195.2's 11 refusals were a gap under §16.10, and it is closed.
`lane-turned-ball-block` (`d6bef345`, `771fba09`) found one cause behind both
refusals. Turned about `x` at a size that is not a power of two, the rounded
start of the circle sits a hair off the ball's axis: 4.4e-16 at radius 3,
1.04e-20 at radius 0.001. `atan2` still reads its latitude as exactly −π/2, so
`mesh.rs`'s `the_surface_pinches_at` calls it a pole. `arrange.rs`'s
`across_a_circle_that_starts_at_a_pole` only cut the circle when
`the_way_its_first_coordinate_runs_at` had no answer, and it had one, read off
that hair. So the fuse left the circle as one closed edge, and the mesh walked
it from a "pole" with no pole side to close on. At radius 3 that was a loop
whose corners coincide. At 0.001 it was a face going right round with no seam.
The lane found this with temporary probes that printed the points and the code
that put them there. §195.2's guess, a point across a few ulps from the other
pole, was not the cause.

The fix has two parts, and neither adds a tolerance:

1. The fuse asks `the_surface_pinches_at` itself, which is now `pub(crate)`.
   A start the mesh will call a pole always gets the cut.
2. Once cut, the start's node is a box, because the rounded start is not
   certified on the ball. `round_a_closed_crossing` compared that box against
   the exact start as a sum, which straddles zero, so all 11 then refused
   "order along a crossing". It now knows the start by either of the two names
   it is stored under, the walked point or its node.

All 36 answer and pass `Model::validate`, and the sweep asserts 36 where it
asserted a floor of 25. Two new tests check radius 3 and radius 0.001 for one
solid and for volume against 125·s³ + ⅔πs³. Both come in under, by 6.63e-6 and
9.77e-6 of the volume. The verifier reverted `crates/rcad-topo/src` to base and
saw exactly those three tests fail. The lane also took out each part of the fix
on its own, and each time the tests failed.

What the lane left, and what I have not checked:

- The two pole tests still run in two frames: the fuse reads latitude in the
  model's frame, the mesh in its shifted meshing frame. They agree on every
  case here. Nobody has shown they always agree.
- At radius 3 the stored plane puts the two poles on opposite sides, so the
  exact circle winds round the axis about 1e-15 from each pole. The answer
  stores a cut at a near-pole, not a cap round one pole. It validates and
  measures as described above.
- No `RCAD_SOAK_FACTOR=100` soak was run. Only the gate's default cases ran.

### 196.2 The narrow lune §195.1 left unmeasured is still unmeasured

`lane-narrow-lune-pole` (`0d79b850`) was test only. It could not build a wedge
narrower than half a turn that gets as far as the mesher.
`a_narrow_lune_from_a_pole_corner.rs` pins what happens instead:

- **Two knives, the half ball `x ≥ 0` first:** the second fuse **panics** at
  `fuse/arrange.rs:1743`, "two edges of one face run along the same stretch of
  one crossing". This happens in all 48 cases (angles 90, 45, 10 and 170°,
  radii 1, 0.001, 1000 and 3, every rule). The panic is on valid public input,
  so it breaks "panic only on a broken invariant". Its test is `#[ignore]`d, so
  `ignored_tests` is 7. The lane guessed that the pole-to-pole edges left by
  the first cut lie on the second great circle and get counted as running
  along it. Nobody has checked that guess.
- **The turned knife first:** refused by name every time.
- **One prism whose edge is the polar axis:** refused by name every time.
- **The half-turn cut of §195.1 with the knife turned off the seam** depends on
  both the angle and the size. The fuse refuses it with
  `ACurvedFaceGoesRightRoundWithNoSeam` at 45° (r = 1, 1000, 3) and 170°
  (r = 1). At 135° and r = 1 the fuse answers a solid that validates, and then
  `measure` refuses with the same variant. That is a probable gap, because the
  answer exists. The test pins each case, so each will fail and ask to come
  off the list once it answers.

§196.1's fix changed none of these outcomes: `just t -p rcad-topo --test
a_narrow_lune_from_a_pole_corner` on the merge, 3 passed, 1 skipped.

### 196.3 How often §194.3's move onto the pole fires

`pole-move-rate` was read only and ran on the base, before §196.1. It counted
`put_in_a_pole` over `just t -p rcad-topo` and `just t -p rcad-step`. The step
ran 19 668 times, inserted a pole 10 times, and moved a walked point onto one
28 times. The largest move was 8.0e-12, in the turned ball-and-block sweep.
20 of the 28 moves were outside `a_rim_through_both_poles_measures.rs`: 18 in
that sweep and 2 in `a_dome_on_a_flat_disc_imports.rs`. §194.5's item is
closed as measured. §196.1 changes the sweep's path, so the counts for that
file may differ now.

### 196.4 What is left

- **The panic cutting a half ball a second time through its polar axis**,
  §196.2. It is the next item, because it is a crash on valid input. When it
  is fixed, un-ignore
  `a_wedge_cut_from_a_ball_by_two_knives_through_its_polar_axis_is_answered`,
  move `ignored_tests` down, and then the narrow lune can be measured.
- The half-turn cut off the seam refusing at 45° and 170°, and the 135° answer
  that will not measure, §196.2.
- The two frames of the pole test, §196.1.
- Still open from §195.4: a named refusal in `mesh.rs` for a closed edge that
  neither direction rule answers; `AClosedEdgeOfTheAnswerWouldStartAtAPole`
  and §39.4's row for it; §193.3's carry-overs.

Commands: `just decision 195.2`, `195.1`, `194.3`, `16.10`.
