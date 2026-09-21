## 189. The pole check lands, the far pipes are answered, and the LED body meets its dome — 2026-09-21

*Three lanes off `f792e20c`, dispatched by §175's skill against a base gated
green here first (`just gate quiet`: PASS, 2139 run, 2139 passed, 3 skipped).
Two wrote code and were verified; the third was read-only. `validate` now
refuses §188.2's pole off the pinch. The six far-pipe refusals from §188.4 are
answered. The two LED files that "will not lie flat" turn out to be a gap
§188.3 did not predict: the body cylinder touches its dome ball round a circle,
and the coarse retry did not cause it.*

Merged as `633223d6` (`lane-pole-off-the-pinch` at `2e38ab1e`,
`lane-far-pipe-normals` at `07166706`). `just gate quiet` run here on
`633223d6`: PASS, 2146 run, 2146 passed, 2 skipped. Every other number below is
a lane's. Where the verifier re-ran a number, this section says so.

### 189.1 A pole must stand where its surface pinches

`Model::validate` refuses a `WireStep::Pole` whose vertex is provably away from
every point where its face's surface pinches. The refusal is
`Invalid::APoleStandsWhereTheSurfaceDoesNotPinch { at, pinch }`. A sphere
pinches at centre ± radius along z, the same fixed frame `add_ball` and the fuse
use. A cone pinches at its apex. A plane, a cylinder and a torus pinch nowhere,
so a pole on one of them is always refused. No caller can build such a pole
today: the verifier traced every place a `Pole` is made, and that part of the
check is untested. The cone arm is untested as well.

"Provably away" gets the room `not_provably_off` uses: the larger of the
corner's recorded slop and `what_writing_a_point_down_costs`. The corner and
each pinch point become boxes, because centre + radius rounds. The corner is
refused only if its box misses every pinch box. Comparing interval ends is
exact, so no Undecidable case exists. The check runs after the on-surface check,
so a pole slid off a ball still gets `VertexOffItsFaceSurface`.

**§188.2 described the seed wrongly.** The first half of fuse seed `3482a32e`
has a corner at x = 4.74e226, not at the pole. The verifier reproduced this. The
on-surface check let that corner through because squaring its widened box
overflows (`Undecidable { enclosure: inf }`), and `not_provably_off` reads
Undecidable as "not provably off". The corner at the centre sits inside the
room there, which is about 1.7e212. The new check refuses the seed on the far
corner. **Open:** should an overflowed enclosure count as "not provably on"?
Nothing was changed there.

Fuse corpus: 132 of 926 decoded halves are now refused for this. 131 were
already refused on base. The one new refusal is `e8fe8935` half 0, a pole
4.66e-10 off the pinch of a radius-5 ball: a real error that validate used to
pass. The replay counts go 682 → 681 validated, and every floor is still met.
The verifier re-ran all of these and matched them. `the_fuse_corpus_holds_one_pole_off_its_pinch_that_nothing_refused_before`
pins it. The rcad-step tests, corpus imports included, all pass, but nobody has
counted STEP files per refusal for this check. `TwoPatchesOfTheMeshDrewTheSameSide`
stays in `finish()` as the mesher's own guard.

### 189.2 The six far pipes are answered

§188.4's fix is in `two_lines_between_two_pipes`, and it runs only after the
existing 3 anchors × 4 normals have all refused. `normals_made_of_two_square_ones`
adds one exactly-square normal plus another × ±2^p, p in -80..=80. It keeps a
candidate only if `two_sum` shows nothing rounded and `Exact::dot_of` confirms
it square. `offsets_around_the_middle` tries every f64 inside the exact middle's
enclosure, nearest the centre first.

Two corrections to §188.4, both found by building the fix. **Each kept normal
must be turned toward `asked_for`.** Without that, one pair asked the second way
round had its plane facing the wrong way, and the lines sat on opposite sides in
reversed order. **The ±2 ulp margin is not needed.** The f64 nearest the true
middle lies inside the enclosure, so if any f64 separates the lines, that one
does. A window wider than 16 f64s is given up on and stays refused, never
answered wrongly. The sweep saw windows of 4 to 6 (lane's probe, since removed).

The sweep went from 7994 of 8000 answered (6 refused) to 8000 of 8000, with 0
wrong either way. The verifier re-ran both and matched. With only half 1 there
are still 2 refusals; with only half 2, all 6 remain. That split was the lane's
own, and the switch it used has been removed. Cost, also the lane's and not
repeatable: about +0.45 ms per refused asking. The common case did not change
measurably (6000 ordinary askings: 216–228 ms before, 220–232 ms after).
`the_kernel_names_the_three_far_pairs_it_refuses` is no longer ignored, and
`ignored_tests` went 3 → 2.

### 189.3 The LED body meets its dome, and the walk misses it

Read-only lane. `LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm` and `_Z6.0mm` refuse
face 8, the LED body: a cylinder of radius 1.5 on the axis x = 1.27, z = 10.
Its closed edge 28 is where the body meets its dome, a ball of the **same
radius, centred on the cylinder's axis**. The two surfaces touch all the way
round that circle and do not cut through each other.

The reader names edge 28 `RightRoundWhereItsFacesCross`, so it goes to
`follow_the_quartic`. The follower starts at its own seed, a quarter turn from
the file's corner (index 202 of 206 at deflection 0.869). It also goes the wrong
way round: `the_way_a_closed_edge_is_stored_running` gives None at every
deflection, because the surfaces touch and their crossing has no direction.
The wire walk climbs 2π along the bottom and 2π again along the top: from
longitude -1.5708 to 10.2102. `WalkingAWire::finish` then returns None, and
`mesh.rs` turns that into `AWireWillNotLieFlatOnItsFace`.

**The coarse retry did not cause it.** It only chooses which of two refusals
is reported. At deflections from 0.869 down to 0.0068, face 8 refuses either as
"will not lie flat" or with edge 28 not followable, and it meshes at none of
them. **It is a gap:** the circle exists and the file names it as a CIRCLE. A
temporary hack that started the walked points at the corner and ran them
backwards got face 8 through. Starting at the corner alone did not. The next
refusal is face 17, `CornersCoincide`, which nobody has identified. It is
probably the dome's wire through the ball's pole.

The proposed fix has two halves. In `rcad-geom/src/walk.rs`, a ball centred on
a pipe's axis goes in the row that already handles a cone and a pipe on one
axis: they meet in circles, so work the circle out directly and start at the
run's own corner. In `rcad-topo/src/mesh.rs`, when the direction rule gives
None because the faces touch, run the points the way the pipe's longitude
increases. The proposed test is `a_bar_capped_by_a_dome_of_its_own_radius_imports`
in `crates/rcad-step/tests`. Whether some of §188.3's seven "50 shortenings"
files are this same dome joint has not been measured.

### 189.4 What is left

- **The LED dome joint**, §189.3: `walk.rs` and `mesh.rs`, plus the test file.
- **`validate` passing a flat-only bent edge**, §187.1's sweep half, in
  `validate.rs`. It is free now that §189.1 has landed.
- **An overflowed enclosure in `not_provably_off`**, §189.1: is it "not
  provably off", or "not provably on"?
- **The cone arm of §189.1**, which has no test.
- **The turned-bend band**, §187.4 item 2, and **fuse::bake's corner test**,
  the two ignored tests left.

Commands: `just decision 188.2`, `188.4`, `188.3`, `16.10`; the `lanes`
workflow run `wf_da610ea3-f3f`; `git merge lane-pole-off-the-pinch
lane-far-pipe-normals`; `just gate quiet` on `f792e20c` and `633223d6`.
