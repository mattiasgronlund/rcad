## 390. A torus band right round the ring is cut round the ring — 2026-10-03

*Queue row Q815, which was Q372 (refuses/B, §229.4, §280.8), off `b1136e72`.
The mesher cut a patch only round a torus's tube, so a torus face that goes
right round the ring was refused `AWireWillNotLieFlatOnItsFace`, a gap and
not a limit (§280.8). It now cuts round the ring too. This section is what
refused, what changed, and what the reader still refuses.*

### 390.1 What refused, run before anything changed

**The disc of §229.4, written face by face.** A disc of radius three and
thickness two whose rim is the outer half of the torus of ring three and
tube one, cut at its outer equator into two bands. Each band is one wire:
round one circle, up a seam across the tube, round the other circle, and
down the seam again. The flat faces touch the torus round its top and bottom
circles. `a_disc_with_a_round_rim_in_two_bands` in
`building_a_solid_out_of_faces.rs` writes it through `SolidBuilder`, with
either band first in the shell, and with the walls used forwards or turned
round.

- Walls forwards, on `b1136e72`: refused `AWireWillNotLieFlatOnItsFace`.
- Walls forwards, on `b1136e72` with only the seam half of
  `WalkingAWire::goes_right_round_the_second_way` switched off, as an
  uncommitted probe: both band orders built, validated and held what Pappus
  gives at deflections `0.1`, `0.03` and `0.01`. A line printed from the
  probe saw the seam half answer `true` twenty times over the run. So Q37's
  reading is confirmed: the seam half was the whole refusal.
- Walls turned round, with each band's loop walked the other way: refused
  `ADraftEdgeIsWalkedTheSameWayTwice` at the door, on the base and after the
  change alike. That rim walks the top and bottom circles the way the flat
  faces do; it is a wrong description, and the door says so before any mesh.

The same probe on the other two shapes: the top half of a ring with a seam
across the tube built and held Pappus, and the crown of a ring was refused
`TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }`, as §280.8
found. The top half of a ring with one wire per equator is refused
`AWireWillNotLieFlatOnItsFace` on the base too, and each of its wires is one
closed edge, which passes no point twice, so that refusal is the other half,
the climb.

**The disc read from §229.4's STEP file** (`f886614e`'s test) never reached
the mesher on the base: both spellings were refused
`AClosedEdgeIsWalkedAgainstItsSurface` at the equator. 390.4 is why.

### 390.2 What changed in `mesh.rs`

**The seam half of the refusal is gone.** A wire that passes one point at
two corners a whole turn apart round the ring closes in the chart; it is an
ordinary loop a whole turn wide. What was wrong with meshing it was the
width, and the width is now cut.

**The climb half follows the tube's rule.** A wire that comes back a whole
turn on round the ring may, where it may round the tube: one closed curve,
or a wire with no seam and no pole. A wire that comes back a whole turn on
both ways round is refused. The walked loop carries the climb round the
ring beside the climb round the tube.

**Cutting round the ring is cutting round the tube with the corners turned
on their side.** Everything that cuts a face cuts across the first number
of the corners it is handed. `Chart::OnItsSide` swaps the two numbers of
every corner, and the climbs with them, so the same `cut_each_narrow_enough`,
`cut_in_two`, `where_to_cut`, `how_far_round` and `longitudes_of` cut round
the ring; only where a point is put on the surface, in `points_along_a_cut`
and `cut_in_two`, are the two numbers given back in the surface's own order.
The row named those five functions for a ring-angle version each. Turning
the corners instead gave the ring-angle copy of `cut_a_ring_across` that Q194
needs for nothing: it is the same function, handed the loops on their side.

- **A patch that reaches a whole turn round the ring by a seam** is cut to a
  quarter turn that way (`cut_what_reaches_right_round_the_ring`). A patch
  of more than one loop that does is refused
  `CurvedFaceWithAHoleGoesTooFarRound`; nothing cuts a piece with a hole.
- **A patch that does not is left bit for bit as it was**, though it may be
  wider than a quarter turn round the ring. §343.3's band three quarters of
  the way round measured `6π²` uncut that way, and a chord stops growing past
  half a turn and comes to nothing only at a whole one. So no face the
  kernel meshed before is meshed differently: every one of them was either
  refused or is left as it was.
- **A face of two wires that each go right round the ring** is a band
  (`cut_a_band_right_round_the_ring`). Its second loop is moved by whole
  turns of the tube angle onto the side of the first the face lies on, then
  `cut_a_ring_across` cuts it at four quarters of the ring, and each quarter
  is cut round the tube as any piece is. A face with one such wire, or three,
  is refused `ACurvedFaceGoesRightRoundWithNoSeam`.

**Which side of the first loop the face lies on** is 343.2's rule asked the
other way round. A torus face keeps its material on its left and the chart's
turn `∂u × ∂v` points into the tube, so its loop runs clockwise in `(u, v)`:
up the ring angle along the low side of the band in `u`, down it along the
high side. So the face lies above a loop running up the ring and below one
running down it. The top half of a ring is the check: its outer equator, at
`u = 0`, runs up the ring, the inner one at `u = π` runs down, and the face
is the `u` between them over the top of the tube. With the other turn the
cuts would go under the tube and mesh the bottom half.

**A cut round the ring carries a point of its own.** `cut_in_two` charged a
bare cut only to a piece that is not one whole closed curve, and its comment
said that no solid had shown a bare cut across a piece bounded by two closed
curves meeting a neighbour's diagonal. The crown has. Each cut round it runs
from a point of the outer circle to a point of the inner one, and the flat
face across both circles is free to draw that pair. With bare cuts round the
ring the crown's build was refused `TwoPatchesOfTheMeshDrewTheSameSide
{ drawn: 4, used: 4 }`; with a point on each it builds. Every face cut that
way was refused before such a cut existed, so the point moves no earlier
answer.

### 390.3 What it answers now

Every volume is held within the error its own measurement states, against
Pappus, with the arithmetic in the test.

| solid | Pappus | deflections |
|---|---|---|
| top half of a ring `R = 4`, `r = 1`, one wire per equator | `π² r² R = 4π²` | `0.1`, `0.03`, `0.01` |
| the same with a seam across the tube | `4π²` | `0.1`, `0.03`, `0.01` |
| §229.4's disc, either band first | `18π + 3π² + 4π/3` | `0.1`, `0.03`, `0.01` |
| crown of a ring `R = 20`, `r = 5`, cut at `z = 4` | `2π·20·(25·acos(4/5) − 12)` | `0.5`, `0.1`, `0.05` |

**The crown walks the rescue's ladder with one fold.** At room five, the
tube's own radius and five times the crown's height, its mesh folds; room
two and a half meshes, and so does every rung after it. That is the fold
`a_mesh_that_folds_at_one_deflection_and_not_the_next.rs` counts for the
slivers, cured at the next rung. Its coarsest mesh encloses something,
because every cut round the ring carries a point over the top of the tube,
so the crown never reaches §208.3's arm. Run with the arm taken out as an
uncommitted edit, that file and `building_a_solid_out_of_faces.rs` ran
`17 tests run: 16 passed, 1 failed`, and the one failure was
`the_arm_removal_experiment`, which reads the arm's text.

**No answer moved elsewhere.** `just t -p rcad-topo` ran `1254 tests run:
1254 passed`, and `just t -p rcad-step` failed only the two tests of the
torus written twice before they were pinned (390.4).

**The tests that fail without the change**, each run against `b1136e72`'s
`mesh.rs` with the tests kept:
`the_top_half_of_a_ring_with_no_seam_is_a_band_round_the_ring_and_holds_what_pappus_gives`,
`the_top_half_of_a_ring_with_a_seam_across_the_tube_holds_what_pappus_gives`
and `a_disc_with_a_round_rim_in_two_bands_holds_what_pappus_gives` in
`building_a_solid_out_of_faces.rs`, and
`the_crown_of_a_ring_holds_what_pappus_gives_and_no_rung_of_its_ladder_folds`
in `a_mesh_that_folds_at_one_deflection_and_not_the_next.rs`. They replace
the two half-ring pins and the crown pin, which pinned the refusal.

### 390.4 The step that could not be done: the torus test of `f886614e`

The row asked for `f886614e`'s test to land. It is on this branch, and it
cannot pass from inside this row's files: the reader refuses the disc before
the mesher sees it, so it is pinned to that refusal, for both spellings
alike.

The reader says which way a closed edge is stored from the way its curved
surface's first coordinate runs, and on a torus that is round the tube,
square to every circle round the ring. The travel dots it to exactly zero,
so `which_way_round_a_closed_edge_is_stored` in `brep.rs` falls back to the
file's `ORIENTED_EDGE` flags. `Model::validate` reads a second reference
there, `∇C` crossed into the first, which stores the equator running down
the ring angle; the flags store it up the ring angle, the way its `CIRCLE`
runs, and the lower band's use of it is read as walking it against its
surface.

A probe in `brep.rs`, uncommitted, read that second reference between two
curved faces. The bit-for-bit spelling then got past the equator and was
refused `AWireWillNotLieFlatOnItsFace` on a band, and the one-ulp spelling
was refused too. Not traced, but the disc written through `SolidBuilder`
needed the lower band's use of the bottom circle, where the flat face
touches the torus, the other way round from the file's flag: the flags miss
the mesher's rule for faces that touch, `the_way_a_closed_edge_where_its_faces_touch_runs`,
as well. That is the reader's to fix, and it is the row 390.5 files.

### 390.5 What this leaves

- **The reader stores a circle round a torus's ring by the file's flags.**
  The equator between the two bands, and the circles where the flat faces
  touch the torus, are stored the way the flags happen to say, and the disc
  of §229.4 read from its STEP file is refused for it. A fillet round a disc
  is a shape files hold. Filed as a row on `brep.rs`, with this file's
  pinned test to turn back into `f886614e`'s, checked against Pappus.

  *2026-10-04, Q824 (`5f117641`):* where the travel is square to a torus's
  longitude, the reader now stores the circle by the rule `rcad-topo` uses —
  the second reference where the two surfaces cross, the mesher's rule where
  a flat face touches the torus. The bit-for-bit spelling reads and meets
  Pappus at three deflections. The one-ulp spelling is still refused
  `AClosedEdgeIsWalkedAgainstItsSurface`, now at the lid's circle and in
  `rcad-topo`, not the reader: Q946.
- **A torus face with a hole that reaches a whole turn round the ring by a
  seam is refused** `CurvedFaceWithAHoleGoesTooFarRound`. Reasoned, not run:
  no such solid was built here. Filed as a row to build one and see.
- Q194 waits on this row and can now build §250.7's half ring in its
  spellings: the two this section builds are its no-seam spelling and one
  seamed one.
