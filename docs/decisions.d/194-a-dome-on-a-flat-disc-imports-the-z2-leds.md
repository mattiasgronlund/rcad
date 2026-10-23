## 194. A dome on a flat disc imports, a rim through both poles measures, a ball-and-block union cut at a pole answers, and the Z2 LEDs are a limit — 2026-09-22

*Three batches, dispatched by §175's skill. Each lane had a verifier.
`just gate quiet` run here: on `79eedce7` before the first batch, PASS, 2205
run, 2205 passed, 6 skipped; on `699704fe` after it, PASS, 2210 run; on
`dddbe240` after the second, PASS, 2214 run; on the third's merge, see §194.4.
A number below is a lane's unless it names a command run here.*

### 194.1 The four Z2 LEDs are a limit: the file gives one face two radii

§193.3 left the four `LED_D3.0mm_Horizontal_O1.27mm_Z2.0mm*` files refused on a
corner 888 spacings off its curved face. In
`LED_D3.0mm_Horizontal_O1.27mm_Z2.0mm.step`, face `#302` is the LED's neck, a
`CYLINDRICAL_SURFACE` `#324` of radius 1.5. Its rim `CIRCLE` `#316` has radius
1.499999999999, and the rim's one corner `#308` lies exactly on that rim, so it
sits 1.0000889e-12 inside the cylinder. "888 spacings" is that distance over
5.07 × `f64::EPSILON`, the largest coordinate's spacing (`brep.rs`). The other
three files have the same shape at face `#17`, corner `#22`.

The lane ruled out every reading that would make it our gap: a wrong surface
(the face has one), the axis's sense (a distance from a line has none), unit or
placement (both radii move together), the axis's `−5.55e-17` tilt (the distance
is 1.499999999999 either way), and the text's rounding (the widest reading of
`3.499999999999` is still 5e-13 short of 3.5). The Z6 and Z10 siblings write
every radius as 1.5 and import. Importing the Z2 files would need a vertex
tolerance, which is the healing §42.9 refuses. So under §16.10 this is a
**limit**, not a gap.

I checked the radii myself:
`grep -oE "(CIRCLE|CYLINDRICAL_SURFACE|SPHERICAL_SURFACE)\('[^']*',#[0-9]+,[0-9.E+-]+\)"`
over the Z2 file gives `CIRCLE` 1.5 ×6, 1.499999999999 ×1, 1.499999999998 ×1,
1.9 ×4; `CYLINDRICAL_SURFACE` 1.5 ×1, 1.9 ×2; `SPHERICAL_SURFACE`
1.499999999998. If these files are ever healed, the sphere at 1.499999999998
against rims of 1.5 is probably the next blocker.

`a_z2_led_corner_off_its_face.rs` pins the refusal on all four files with the
face and corner ids, and checks the refused distance is 1.5 − 1.499999999999. It
also pins that Z6 and Z10 import. Nothing in `src/` changed, so it has no
failing-at-base run. Lane `lane-z2-leds`, `7a57fa9e`.

### 194.2 A rim whose one corner is a pole is cut in two across from it

The half ball on a flat disc that §193.3 named is now read. The first lane
found that the pole was not the blocker. The rim's direction was. At a corner on
a pole, `mesh.rs`'s two rules for which way a closed edge runs both answer
nothing, and so does `brep.rs`'s own hint. The mesher then walked the rim
backwards and refused with `AWireWillNotLieFlatOnItsFace`. The same file with
its rim written already cut at the far pole imported with no change at all.

So `describe_the_halves_of_a_circle` in `brep.rs` now also cuts a closed circle
at the opposite point when the curved face beside the corner has no direction
there (`the_way_its_first_coordinate_runs_at` is `None`). That is the narrower
of the two rules offered. Cutting every ball-and-plane rim would add a corner to
faces that import today. `a_dome_on_a_flat_disc_imports.rs` holds three half
balls on a disc, and all three measure half a ball within the published error.
The verifier reverted `brep.rs` and saw the one-corner case refused again.
KiCad stays 160 of 323 with an identical refused list. Lanes `lane-dome-on-disc`
(`381ab037`) and `lane-rim-cut-at-pole` (`2f83c5bf`).

### 194.3 An edge through a pole holds the pole as one of its points

A rim whose one corner is off the poles passes both poles in the middle of its
edge. `381ab037` taught the wire walk that a walked point landing exactly on a
pole is a pole, and opens its side the way a corner's is opened. That point
lands there only by luck. It did at the import's deflection, but not at 0.001.

`lane-pole-on-edge` (`c12323fc`) makes it certain. When an edge runs right round
where a ball meets another surface, each of the ball's poles is added to the
edge's walked points, but only when `side_of_surface` certifies `Zero` on both
surfaces. The pole goes between the two neighbouring points that straddle the
plane square to the crossing there. A walked point that already reads as the
pole, by `the_surface_pinches_at`'s own latitude equality, is moved onto it
rather than given a twin, and the distance moved is added to the mesh's worst
deviation. There is no new tolerance.

`a_rim_through_both_poles_measures.rs` measures the half ball at 0.05, 0.0137,
0.001 and 0.0003. The verifier put `mesh.rs` back to base and all four failed
with `ACurvedFaceGoesRightRoundWithNoSeam`. Edges that are only part of a
crossing get no pole, because deciding whether a pole is on such an edge needs
`picked_by`.

### 194.4 A ball-and-block union whose crossing starts at a pole answers

`lane-pole-on-edge`'s random search turned up a seed in
`fuses_that_must_agree.rs`: a ball of radius 1 on a block of side 5 at `[3, 3]`,
turned 16 steps about `x`. The union refused with
`AClosedEdgeOfTheAnswerWouldStartAtAPole`, which `a_turned_curved_refusal` does
not allow. I replayed the seed on `699704fe`, before that lane's change: all
three `ball_and_block` properties failed the same way
(`PROPTEST_CASES=1 just t -p rcad-topo --test merged -E
'test(~fuses_that_must_agree::ball_and_block)'`, 0 of 3). So it was already on `main`, and I did not treat it as
the lane's regression.

It is §194.2's shape inside the fuse. Turned, the floor meets the ball in a
circle whose start is exactly the ball's lower pole, so the closed edge has no
direction. `lane-ball-block-pole` (`f777d3e8`) cuts such a circle across from
its start in `arrange.rs` (`across_a_circle_that_starts_at_a_pole`), with a new
meet point `HalfWayRoundAClosedCurve` and corner name `HalfWayRoundACurve`. The
two halves join the same two corners, so an edge's name now also says which
half it is, read off the material side (`which_half_of_a_circle_cut_at_a_pole`).
`select.rs` reads a two-sided wire along one ball as round.

It does not cut when the point across is the other pole, which is a plane
exactly down the polar axis. Cut there, the fuse's own mesh refuses a wire whose
two corners are both poles, so
`degenerate_curved_fuses::a_ball_cut_by_a_plane_down_its_polar_axis_is_refused_by_name`
stays as it is. `a_turned_ball_and_block_closes_an_edge_at_a_pole.rs` failed at
base and passes. The verifier reverted `fuse/` and saw it and the three seeded
properties fail. The seed is committed.

Its branch failed the gate on `check-wildcards`, 92 against 88. The new
`panic!` on a material side that is not `Neg` or `Pos` is one. The other three
are the three kinds of `Surface` that `across_a_circle_that_starts_at_a_pole`
leaves to `_`, where `None` leaves the fuse refusing by name as before. Under
§40.8 the first is loud, and the other three fall back to a named refusal, so I
moved the ledger to 92 with that note.

`just gate quiet` here on `36f43c9e` with this section beside it: PASS, 2215
run, 2215 passed, 6 skipped.

### 194.5 What is left

- **A wire whose two corners are both poles** will not walk in `mesh.rs`
  (`walk_a_wire_round`, `AWireWillNotLieFlatOnItsFace`). This is a gap: the
  lune between two meridians plainly exists. It is why §194.4 does not cut a
  plane down the polar axis. Closing it would flip
  `a_ball_cut_by_a_plane_down_its_polar_axis_is_refused_by_name`.
- `AClosedEdgeOfTheAnswerWouldStartAtAPole` is still raised for a pole start on
  a ball against a ball, pipe, cone or torus, and for the polar-axis plane.
  §39.4's row for it was written before the ball-and-plane case closed and has
  not been edited.
- The box round the point across the circle (§194.4) allows
  2 × `POINT_ALONG_A_CROSSING_ROUNDOFF` of the largest coordinate. The doc argues
  that bound, and no test measures it. It needs a claim-audit pass.
- When neither closed-edge direction rule answers, `mesh.rs` still keeps the
  walker's order, and its own comment says that can give a wrong mesh rather
  than a refusal. Here it was caught as a refusal. A named refusal there would
  be safer.
- Not measured: how often §194.3's move-onto-the-pole case fires against a
  plain insert.
- Carried over from §193.3: the slit cut's limits, a closed run carrying `and`,
  the plain-f64 circle check in `brep.rs`, `builder.rs`'s two-step hole,
  `squared` onto `Interval`, the turned-bend band and `fuse::bake`'s corner test.

Commands: `just decision 193.3`, `16.10`, `40.8`, `42.9`; the `lanes` workflow
runs `wf_d3cdcfe4-228`, `wf_f46714e2-b61` and `wf_96528fab-147`.
