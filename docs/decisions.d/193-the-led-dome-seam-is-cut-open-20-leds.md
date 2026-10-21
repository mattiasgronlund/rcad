## 193. The LED dome's seam is cut open, 20 LEDs import, and a mesh refusal names the face that stops it — 2026-09-21

*One batch of two lanes, dispatched by §175's skill off `8201bbb0` (`just gate
quiet` here: PASS, 2164 run, 2164 passed, 2 skipped). Both had a verifier, and
both verifiers said merge. Merged as `81f58380`: `lane-dome-slit` (`7f0ff912`),
`lane-finer-refusal` (`a6d9071a`), and two commits of mine. `just gate quiet`
here on `81f58380`: PASS, 2166 run, 2166 passed, 2 skipped. A number below is a
lane's unless it names a command run here.*

  193.1   The dome's slit is cut open, and KiCad goes from 140 to 160
  193.2   A mesh refusal names the finest face that stopped it, and `CornersCoincide` is not a crossing
  193.3   What is left

### 193.1 The dome's slit is cut open, and KiCad goes from 140 to 160

§192.2 found the LED domes refused with `CornersCoincide`: the seam is used
twice, and in the chart both uses land on the same points. The brief assumed the
two uses should land a whole turn apart. **That premise was wrong**, and the
lane showed why with printed chart corners. The file's ball has its axis along
`−y`, so in the file's own chart the seam runs from the pole (the dome's top) to
the rim, and the two uses are a turn apart. An rcad `Sphere` always has its axis
along `z`. In that chart the dome's top is an ordinary point on the equator,
`(−π/2, 0)`, and the seam is a line from there to the pole, walked out and back
on the same points. On the `O1.27 Z10` dome, corners 1 and 14 are both point 24,
and corners 2 and 13 are both point 45. Walking round an ordinary point gains no
longitude, so no shift by whole turns can separate the two uses.

`mesh.rs` now cuts such a face open. `the_inner_end_of_a_slit` finds a corner
whose two neighbours are the same point. `where_a_slit_carries_on_to` runs a line
from it along its own longitude, away from the slit, to the first side of the
loop it reaches, ending there by `where_to_cut`'s rule. Each half then holds one
side of the slit. No tolerance is involved.

`an_led_dome_seam_is_cut_once.rs` builds the LED dome on a bar of its own radius
by hand. It fails at base with `CornersCoincide` on `#57`, and with the fix it
imports within the published volume bound. The verifier reverted `mesh.rs` and
saw that failure.

**What it bought**: `just t -p rcad-step -E 'binary(merged) &
test(how_often_a_real_file_imports)' --no-capture`, run here on `81f58380`,
gives **160 of 323**, up from 140. The
lane diffed the refused lists and the verifier re-ran the diff: 20 files newly
import (8 `LED_D3.0mm_Horizontal`, 12 `LED_D5.0mm_Horizontal`), and 0 newly
refuse. So the true blocker of these LEDs was the slit, not `#223`. The four
`LED_D3.0mm_Horizontal_O1.27mm_Z2.0mm*` files still stop earlier, on a corner
888 spacings off the curved face the file puts it on, as they did before. Five
LED tests are re-pinned to "imports and validates": four by the lane, and
`a_circle_imprinted_on_a_flat_face_keeps_its_bend.rs` by me, since the gate
caught it outside the lane's write set.

**Limits of the cut.** It handles only a one-loop face on a surface that wraps.
It gives up, and the triangulator refuses as before, when the slit leaves along
the tip's own latitude.

### 193.2 A mesh refusal names the finest face that stopped it, and `CornersCoincide` is not a crossing

`the_coarsest_mesh_whose_sides_do_not_cross` now hands back the refusal from the
**finest** mesh it tried. A face that meshed at a finer deflection is not what
stopped the solid. `a_mesh_refusal_names_the_face_that_stops_it.rs` builds a
half disc whose hole leans out through the rim. Its ground face crosses only at
the coarse deflection, and its lid crosses at every deflection. At base the
refusal named the ground (face 0), and the verifier reproduced that.

`CornersCoincide` is out of `its_sides_cross`, so the halving stops at the first
one. The argument is in the doc comment: two corners of one loop at one chart
point are fixed by the solid, and the only accidental case needs two edges
within a rounding of each other, far below the halving's floor. The measurement
beside it was taken with a print that was then removed, **before §193.1**: of
1364 finer meshes across the suite, 886 refused `CornersCoincide`, and none was
followed by a mesh. The verifier rebuilt the print and got 1365, 884 and 4
successes, with no success after a `CornersCoincide`. The counts are off by one
or two, and the conclusion is the same. The lane's second figure, 24 of 24 LED
sweeps refusing again at 0.005, was not re-run by the verifier. Those LEDs now
import, so it cannot be re-run as stated. No builder-made solid makes
`CornersCoincide` happen. The "cannot be cured" claim therefore rests on the
argument plus that rebuilt measurement, not on a standing test.

*A fuse-made solid did (§364): a ball touching the edge between two of a
prism's walls leaves a wire that sets off from one corner twice. The mesher
cuts such a wire apart now, so that case no longer reaches the triangulator.*

### 193.3 What is left

- **A dome whose rim crosses a plane**, the §192.4 item "a rim through a pole
  with no corner there", **reachable through the public API**: the lane's
  hand-built half ball on a flat disc is refused with
  `AWireWillNotLieFlatOnItsFace`. That is why its test puts the dome on a bar.
  No committed test shows the refusal yet. It is a gap under §16.10; the answer
  plainly exists.
- **The four `LED_D3.0mm_Horizontal_O1.27mm_Z2.0mm*` files**: a corner 888
  spacings off its curved face.
- **The slit cut's limits**, §193.1.
- Carried over from §192.4 and §190.6: a closed run carrying `and`, the plain-f64
  circle check in `brep.rs`, `builder.rs`'s two-step hole, `squared` onto
  `Interval`, the turned-bend band and `fuse::bake`'s corner test.

Commands: `just decision 192.2`, `16.10`; the `lanes` workflow run
`wf_6d9f4bf3-142` (`wf_a67fa78d-b35` was a launch with a wrong base sha,
stopped before any lane worked); `just gate quiet` on `8201bbb0` and `81f58380`.
