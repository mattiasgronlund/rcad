## 471. A corner where two fans of faces meet gave no wrong answer, and the LEDs' fuses refuse at a pipe touching a plane there — 2026-10-28

*Answers Q2911, the first question §469.6 filed: whether anything rcad computes
on a solid assumes one fan of faces round each corner. Measured by Q2911's
read-only lane at `1085ae50`, with two probe test files in a scratch worktree,
not kept (§471.5).*

**Settled, as far as it was run: nothing rcad computes gave a wrong answer or
panicked at a corner in two fans.** The volume, the area, which side of the skin
a point is on, the mesh, the STEP writer and every fuse that answers are right
on §469.4's plate paired across the air, against the same plate paired across
the plate and moved apart (§471.2). The fuse writes such corners itself, and
those answers validate and measure right. On the 21 KiCad solids the mesh, the
corner's own side of the skin and the STEP round trip are right (§471.3).

Two refusals were found and filed (§471.6). Every fuse tried on the 21 KiCad
solids refuses at such a corner, but because a pipe touches a plane there, not
because the faces round it form two fans. And `classify_point` refuses the
whole LED, not just its corner. A third refusal is about §469.4's line and not
its corners.

**Not shown:** that nothing can go wrong. No fuse answers on a KiCad solid with
such a corner, and no point near one could be checked, so those two readers have
not been tried there.

### 471.1 What the corners are

The fans are counted as §469.6 counts them: the corners of faces at each vertex,
one per step along a loop, joined wherever two share an edge.

The LED's two such corners are not two pieces of material touching. On the
front of the flange, `y = -1.27`, the round hole of radius 2.5 touches the flat
chord `x = -1.23` at `#74`, and on the back, `y = -2.27`, it does the same at
`#123`. The disc face `#223` inside the hole lies on the same plane as the ring
face `#59`. Seen from the geometry, `#74` is one point on the straight
right-angle edge between `y = -1.27` and `x = -1.23`: one fan, which `#59`
enters as two points of zero angle. Seen from the loops, it is two fans. The
plate of §469.4 paired across the air is the other kind: two fans at `(2, 2, 0)`
and `(2, 2, 1)` where two pieces of plate really touch along the line.

### 471.2 §469.4's plate

Built three ways: paired across the air (two fans at each end of the line),
paired across the plate (one fan), and with the hole moved to `[0.5, 1.5]²` so
nothing touches.

- Volume `11` and area `42`, error `0`, for all three.
- `classify_point` is right at 2527 of 2527 points, a grid at quarter steps,
  against the exact answer, for each.
- The mesh's volume and area are right. For both pairings that touch, the side
  up `(2, 2)` is drawn by four triangles, two each way, and the mesh does not
  refuse. That is the line seen through a mesh with one table of points, the
  same for both pairings, so it says nothing about corners, and it is no defect.
- STEP written, read back and validated: 18 vertices before and after, volume
  `11`, and the corners in two fans come back in two fans.
- The fuse: 12 boxes by union, difference and intersection, both ways round,
  and copies of the plate itself, stacked one up and half up. 170 answers, every
  one valid with the exact volume. A stack of two plates paired across the plate
  comes back with corners in two fans that the fuse wrote itself.
- Of the refusals, every one but one kind is `ResultIsNotManifold`, count 4, on
  an answer where two solids touch along an edge of both, which §80.9 decided
  not to build. Those are alike for both pairings, and for the moved-apart plate
  where a box covers the notch.
- The one other kind: the box `[2, 2.5] × [1.5, 2] × [-0.5, 1.5]`, or its mirror
  `[1.5, 2] × [2, 2.5] × [-0.5, 1.5]`, united with the plate paired across the
  air answers `11.25` with the plate first and refuses `ResultIsNotManifold`,
  count 4, with the box first. Paired across the plate, both orders answer. The
  plate the fuse makes, the L less `[1, 2]² × [-1, 2]`, refuses the same way and
  has no corner in two fans, so it is the line and not the corner. It is class
  C (§471.6).
- Two cubes touching at one corner unite into two solids, so the fuse does not
  write a corner in two fans where two pieces touch at one point only.

### 471.3 The 21 KiCad solids

The sweep imports all 165 KiCad files that import and keeps the solids with a
corner in two fans: 21, as §469.6 counted, the 20 `LED_D3.0` and `LED_D5.0`
horizontal files and `LED_D5.0mm-3.step`.

- The mesh is closed on all 21. In
  `LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm.step` its volume is
  `162.827470914589` against `162.82747091458947` measured, and its area
  `199.12876532147382` against `199.1287653214738`.
- The corner itself is on the skin, 41 of 41 times.
- Written to STEP and read back, all 21 keep their volume and their number of
  corners in two fans. In the LED file one vertex stays at `#74`'s point.
- Every fuse refuses. A box of 0.2 centred on each of the 41 corners refuses
  `Undecidable` on *where three surfaces meet*, under intersection and under
  difference, and so did turned boxes and a box of 0.6 in the LED file. A print
  put in `derive_a_corner` (`fuse/bake.rs`) in the scratch tree names the corner
  it stops at: `#123` at `(-1.23, -2.27, 3.0)`, handed the plane `x = -1.23`,
  the plane `y = -2.27`, and the body's pipe of radius 2.5, which touches
  `x = -1.23` there. `place_a_corner` gathers every face round the vertex, so
  the pipe is there only because the corner holds the body's fan as well as the
  flange's. Each fan alone has two surfaces, and its corner would be read off
  the model. The fuse stops at the first corner it cannot place, so whether
  `#74` refuses as well is not known, and in the other 20 files which corner
  refuses was not traced. Class A (§471.6).
  - Since Q2922 (`f037591f2`) bake places these corners, and the 41 boxes are
    refused further on, in five ways, each a row of its own:
    `AFaceTrimmedByACurveThisCannotFollow` on the upper half of the flange's
    rim, 14 corners in 7 `LED_D5.0` files (Q2926); `Undecidable` on *side of
    plane*, 14 in 7 files (Q2929); `AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve`,
    6 in 3 `LED_D3.0` files (Q2930); `AFaceTrimmedByACurveThisCannotFollow` on
    face slot 2, 3 in 2 files (Q2931); and still *where three surfaces meet*,
    4 in 2 files (Q2928). The probe that ran the sweep is kept outside git, on
    this machine, in `~/rcad-probes/q2911/`.
    - The *where three surfaces meet* item stopped at the front corner of
      two fans in each of its two files: `(-1.23, -1.27, 15)` in
      `LED_D5.0mm_Horizontal_O1.27mm_Z15.0mm.step`, and
      `(-0.23, -3.81, 5.999999956464)` in
      `LED_D3.0mm_Horizontal_O3.81mm_Z6.0mm.step`. There the flange's front
      plane and the body's front disc plane are one plane written twice, a
      rounding apart, so the corner has three planes round it and failed
      only `place_a_corner`'s planes condition. Q2928 dropped that
      condition: the corner read out of the model is now asked about by
      every surface round it, with the room kept, and the two files now
      refuse `AFaceTrimmedByACurveThisCannotFollow` on the body's pipe and
      `AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve` on the front disc.
- `classify_point` on the LED answers `CannotFollowAFacesTrimming`, naming face
  slot 2, at every point tried: the body's centre `(1.27, -5, 3)`, points in the
  flange, outside it and far off. So no point near the corner could be checked,
  and the refusal is the whole solid's. Class A (§471.6).
  - Since Q2923 (`a93fcfbda`) it answers. Two curved sides were read against
    the wrong surface. The front `y = -1.27` is two faces, the flange's (slot
    2) and the body's (slot 7), stored a rounding apart, and the three arcs of
    the circle between them were read against slot 7's plane, not the pipe of
    radius 2.5 their runs name: two planes cross in no arc. Past that, the rim
    where the dome meets the body's pipe (slot 8) was read against the ball,
    not the rim's plane its runs name. `every_reading_of_the_sides_of`
    (`holds.rs`) now reads such a side against its run's `and` between two
    flat faces or two curved ones, and `where_a_point_is_in_a_led.rs` pins
    five points and 2879 of a grid behind the front.
- There is no LED with `#74` written as two vertices to hold up against it: the
  reader refuses that file, *what #15 describes is not a solid this kernel can
  hold*.

### 471.4 What would reopen this

A fuse that answers on one of the 21, or a point the classifier answers near
one of their corners, once the two class A rows close. Either is a reader not
yet run on a real corner in two fans.

### 471.5 What ran this

Two probe test files, `probe_q2911_plate.rs` and `probe_q2911_led.rs`, under
`crates/rcad/tests/` of a scratch worktree made with `git worktree add
--detach` at `1085ae50`, and removed after. Copies are in the dispatcher's
scratchpad for this session, which a reboot wipes.

- `just t -p rcad --test probe_q2911_plate --no-capture`: §471.2, all of it.
- `just t -p rcad --test probe_q2911_led --no-capture
  probe_q2911_every_kicad_file_with_a_corner_in_two_fans`: §471.3's sweep.
- `just t -p rcad --test probe_q2911_led --no-capture probe_q2911_the_led` and
  `probe_q2911_the_led_away`: the LED's mesh, its round trip, the file with two
  vertices, and the classifier's refusals. The corner `derive_a_corner` gave up
  at came from the second with an `eprintln` added in `fuse/bake.rs`.
- `just t -p rcad --test probe_q2911_led --no-capture
  probe_q2911_a_plate_the_fuse_cut_its_hole_into`: the union that refuses one
  way round on the plate the fuse makes. The probe printed `exact 11.75`, a slip
  in the probe: the answer is 11 + 0.5 - 0.25 = `11.25`, which is what the fuse
  gave.

### 471.6 The rows

- **Q2922**, tier 1, refuses/A: every fuse of the 21 KiCad solids refuses where
  the body's pipe touches the flange's flat at a corner in two fans (§471.3).
- **Q2923**, tier 1, refuses/A: `classify_point` refuses every point of the
  LED (§471.3).
- **Q2924**, tier 5, refuses/C: §469.4's plate united with a box beside its
  line answers one way round and refuses the other (§471.2).
- Q2911 is closed at `1085ae50`, the base its lane read.
