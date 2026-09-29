## 343. Two loops side by side on a torus: the band is read off which way they run — 2026-09-28

*Row Q196 (`lane-q196-two-loops-side-by-side-on-a-torus`, off `893b2b94`).
§280.8 left one thing reasoned and not run: two circles round the tube of one
ring bound two bands, and the mesher put the second circle the nearer way
round the ring from the first, whichever band the face was. It was run, it was
wrong, and the second circle is now placed by which way the first one runs.*

  343.1   What was run
  343.2   The rule
  343.3   What this leaves

### 343.1 What was run

Every solid here is a band of the ring `R = 4`, `r = 1` about `+z` between the
circle round `(4, 0, 0)` and a second circle round the tube, with a flat cap
on each circle's plane through the axis. Each is built the way
`tests/rings/mod.rs` builds a bend, with the wall used reversed and no seam:
the wall is two wires, one circle each. The other band between the same two
circles has both caps facing the other way and every coedge on a circle
turned over. By Pappus a band over the ring angle `φ` holds `φ·R·π·r²`.

With the old placement, through `SolidBuilder`:

- **Circles a quarter turn apart.** The quarter band built, validated, and
  measured `2π²` within its stated error at deflections `0.1`, `0.03` and
  `0.01`. The three-quarter band was refused
  `AClosedEdgeIsWalkedAgainstItsSurface` by the sweep, before anything was
  meshed. That is §243.3's refusal of a valid three-quarter bend and Q103's to
  fix: the sweep reads which side of each cap's plane the wall keeps off the
  other circle's corner, and past half the ring that corner is on the side the
  wall does not keep.
- **Circles half a turn apart.** Both bands are halves, and the nearer way
  round is a tie. After the move the second circle sat at ring angle `−π` for
  both faces, so both were meshed as the lower half. The lower half built, validated, and
  measured `4π²` with its centroid at `y = −2.586`. The upper half was refused
  `FacesDoNotAgreeWhichWayTheyFace`: the lower half's wall between the upper
  half's caps.

The three-quarter band was also built without the sweep, straight into the
arenas, and put to `SolidView::mesh`, which is public in its own right. At
deflection `0.1` it measured `13.07` where Pappus gives `6π² = 59.22`, allowed
`4.07`. Nothing in that mesh was folded, so nothing refused it. That is the
wrong answer §280.8 predicted.

### 343.2 The rule

A probe printed each loop of the quarter band as the mesher walks it. The
circle at ring angle `0` ran down the tube angle (it climbed `−2π`), and the
circle at `π/2` ran up. The quarter band validates and measures right, so the
face lies **above a loop running down the tube angle and below one running
up**. The lower half, which also measured right, agrees: its circle at `0`
runs up and the face is below it.

`with_its_second_coordinate_in_the_turn_of` now reads that when both loops go
right round the tube. The first loop stays where it is, bit for bit. The
second goes in the nearest whole turn on the first loop's face side: at or
above its middle for a first loop running down, at or below for one running
up. A face with a hole inside its outer loop is placed as before, the nearer
way round.

**Middles are enough to order two such loops.** Unwrapped round the ring, two
disjoint loops that each go once round the tube are one above the other. A
line straight up the ring angle from the lower loop's highest corner never
meets the lower loop again, so it crosses the upper one, and the upper loop's
highest corner is higher. Its lowest corner is higher by the same argument
from below. So the upper loop's middle is strictly higher. Two middles that
come out equal belong to loops that touch, which no valid face has.

**Q346 should not move this, reasoned and not run.** Q346 turns the bends'
walls to be used forwards and turns every coedge on a circle over. A view walks
a reversed face's wires backwards (`WireView::coedges`), so the mesher should
see each loop running the way it does now. This file's bands are built the
old way and are not in Q346's write set, so Q346 has to rebuild them too.

The tests that fail without the change:
`both_halves_of_a_ring_between_two_circles_half_a_turn_apart_are_the_half_they_are`
in `two_loops_side_by_side_on_a_torus.rs`, where the upper half is refused, and
`a_band_between_two_circles_meshes_the_way_round_its_circles_are_walked` in
`mesh.rs`'s tests, where the three-quarter band measures `13.07`. Both were run
with the new branch switched off in place and the tests kept. With the change,
the three-quarter band measures `6π²` within its stated error at all three
deflections, and so does each half at `4π²`, with its centroid on its own side
of `y = 0`.

### 343.3 What this leaves

- **Through the builder the three-quarter band is still refused**, by the
  sweep and not the mesher.
  `the_three_quarter_band_between_the_same_two_circles_is_refused_by_the_sweep`
  pins that refusal. When Q103 moves the sweep's reading onto the wire, that
  test is the one to move, to `6π²`.
- **Two loops that run the same way round the tube are not refused here.** The
  second loop goes where the first loop's direction says, and
  `cut_a_ring_across` turns both to run up before it cuts, as it did before, so
  nothing in the mesher reads the second loop's direction. Whether the sweep
  refuses a wall whose two circles run the same way was not run. Read from the
  code, `cut_a_ring_across` did the same before this change, on a cylinder's
  ring as well as a torus's.
- The torus orientation §329 found turned round was turned by §361, which
  rebuilt this section's bands with the wall used forwards; they pass as
  they did, as 343.2 reasoned. Q372's cutting in the ring angle is
  untouched. A three-quarter band spans `3π/2` in the ring
  angle in one patch per quarter of the tube, and it measured right at all
  three deflections without cuts in the ring angle.
