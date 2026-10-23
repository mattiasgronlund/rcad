## 280. Why the builder refuses a quarter bend turned half a turn — 2026-09-26

*Row Q119 (`lane-q119-turned-bend`, off `b0744a1e`). §250.4 found that
`SolidBuilder::build` refuses a quarter bend turned half a turn with
`TwoPatchesOfTheMeshDrewTheSameSide { drawn: 1, used: 1 }`. It guessed the
torus's `atan2` cut at `π` and did not trace it. It is traced here. The mesher
unwraps a torus's first coordinate, the angle round the tube, and takes the
second one, the angle round the ring, exactly as `atan2` returns it. So a
wall whose ring angles run past `π` is read as the other three quarters of the
ring, with its seam folded back on itself. That is a gap, and 280.6 gives the
fix and its test.*

  280.1   The probe
  280.2   Leg 1: the ring angles the mesher reads
  280.3   Leg 2: which side is refused, and where
  280.4   Leg 3: nothing else about the turned copy differs
  280.5   The cause
  280.6   A gap, and the fix
  280.7   Where the numbers come from

### 280.1 The probe

This was a scratch test in `crates/rcad-topo/tests/`, never committed. It
starts from the working bend of `rings::a_quarter_bend` (ring four, tube one)
and applies the exact quarter turn `(x, y) → (−y, x)`, `k` times, to every
number of it: both corners, both cap normals, and the seam's half-space
normal. The edges, the coedge directions and the shell's orientations stay as
they are. Zeros are written `+0.0`, the way a person writing it out would. The
half turn was also run keeping the `−0.0` the map produces, and it refused the
same way (280.4).

Three eighths of a turn cannot be written exactly. A corner on the inner
equator, radius three, that is not on an axis needs `a² + b² = 9·4ⁿ` in whole
numbers, and the only solutions put one of them at zero. A cap plane that is
not on an axis also gets a rounded normal. So the exact stand-in for "a wall
that crosses `π`" is a **half bend**: from direction `a`, round through
`(−a_y, a_x)`, to `−a`, with both caps on the one plane through the axis
(outward normal `(a_y, −a_x)`) and the seam's half-space normal `(−a_y, a_x)`.
That shape was turned the same four ways.

Every row is `SolidBuilder::build` under `Policy::certified()`:

| shape, turned `k` quarter turns | ring angles the wall covers | `build` |
|---|---|---|
| quarter, `k = 0` (the working bend) | `0 → π/2` | `Ok` |
| quarter, `k = 1` | `π/2 → π` | `Ok` |
| quarter, `k = 2` (the §250.4 copy) | `π → 3π/2` | `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 1, used: 1 }` |
| quarter, `k = 3` | `3π/2 → 2π` | `Ok` |
| half, `k = 0` | `0 → π` | `Ok` |
| half, `k = 1` | `π/2 → 3π/2`, across `π` | the same refusal |
| half, `k = 2` | `π → 2π` | the same refusal |
| half, `k = 3` | `−π/2 → π/2`, across `0` | `Ok` |

Every wall that runs past `π` is refused. Every wall that stops at `π`, or
crosses `0` instead, builds. The brief's cheap test gets the answer that
implicates the cut: the quarter turn, which touches `π` only at its end,
builds. The one crossing `π` is refused, and so is the half turn. **Which way
the wall leaves `π` is what decides it.** A corner at `(−3, 0, 0)` reads `+π`
(280.4). A wall that goes on from there has every other point near `−π`,
while a wall that ends there has every point on the `+π` side.

### 280.2 Leg 1: the ring angles the mesher reads

`BuildingAMesh::cut_up` was given a temporary `eprintln!`, since reverted,
printing the walked loop of the torus face and each patch handed to the
triangulator. `BuildingAMesh::finish` got one printing the refused side and the
triangles that use it. The half-turned bend's wall loop, as `(tube angle, ring
angle)`:

- corner `(−3, 0, 0)`: `(π, 3.141592653589793)`
- seam midpoint `(−2.121320343559643, −2.1213203435596424, 0)`:
  `(π, −2.356194490192345)`
- corner `(0, −3, 0)`: `(π, −1.5707963267948966)`
- the end circle: ring `−1.5707963267948966`, tube `4.712…` to `9.424…`
- back down the seam: `−2.356194490192345`, then `3.141592653589793`
- the start circle: ring `3.141592653589793` at every point

So one corner reads `+π` and everything else reads in `(−π, −π/2]`. The loop
spans `[−π/2, π]` in ring angle. That is `3π/2` wide, and it is the three
quarters of the ring where the bend is **not**. The seam side is folded
besides: along one line of the chart it runs from `π` down past `−π/2` to
`−3π/4`, then back up to `−π/2`. The four patches that `cut_each_narrow_enough`
makes cut across that three-quarter span. Their cut points sit at ring angles
`−0.898`, `−0.224`, `0.449`, `1.122`, `1.795` and `2.468`, all of them on the
far side of the torus from the bend.

The quarter turn builds, and read the same way it gives: seam `1.5708`,
`2.3562`, `3.1416`, and ring `3.141592653589793` at every point of the end
circle. That is all of `[π/2, π]`, with no fold. A cap circle on `y = 0` with
`x < 0` reads `+π` at every one of its points in both probes, so a wall that
**ends** at `π` reads as one piece.

**Leg 1 holds, with one correction.** The range is not read backwards from the
cut. Its one corner at `π` reads `+π` while the rest reads on the `−π` side,
so the loop is read the long way round with a fold in its seam.

### 280.3 Leg 2: which side is refused, and where

The refused side joins mesh points `0` and `3`: the seam midpoint
`(−2.121320343559643, −2.1213203435596424, 0)` and the end circle's point
`(0, −4, 1)`. **One triangle uses it**, `[0, 3, 8]`, where `8` is the corner
`(0, −3, 0)`. That triangle is on the wall face. **No cap is involved, and
neither is a second patch.** In the chart the triangle is `(7.854, −π/2)`,
`(3π, −π/2)`, `(3π, −3π/4)`. It is in the patch that holds the far seam side,
and it sits below the fold, outside the region the loop really bounds. Nothing
is on the other side of its side `(0, 3)`, so that side is used once.

It is raised in `BuildingAMesh::finish`, `mesh.rs:1841`. The filter
`drawn != 2 || used != 2` is at 1853 and the `Err` at 1856. `drawn` is counted
in `hand_over` at 1676. The probe printed one mesh and one refusal, so the
build stopped at its first mesh.

**So leg 2 breaks as the brief put it.** The side is not the wall's patch
against a cap's. `drawn: 1, used: 1` is one triangle hanging past a folded
boundary. The error's name, "two patches drew the same side", does not
describe that case. That is worth knowing the next time `{ drawn: 1, used: 1 }`
turns up, and it is not changed here.

### 280.4 Leg 3: nothing else about the turned copy differs

The probe does not write the turned copy by hand. It maps the working bend's
own numbers exactly, and the same construction builds at one quarter turn and
at three. The half turn's printed inputs are corners `(−3, 0, 0)` and
`(0, −3, 0)`, start cap facing `(0, 1, 0)`, end cap facing `(1, 0, 0)` and seam
half-space `(−1, −1, 0)`. Those are the working bend's `(3, 0, 0)`,
`(0, 3, 0)`, `(0, −1, 0)`, `(−1, 0, 0)` and `(1, 1, 0)` with `x` and `y`
negated. Keeping the signed zeros (`(−3, −0, 0)` and so on) gives the same
refusal. The corner reads `+π` either way, because the dot product in
`Torus::coordinates_of` adds `−0` and `+0` and gets `+0`.

**Leg 3 holds.** §250.4's own copy was never committed, so this does not check
that file. It builds the same shape and gets the same refusal, word for word.

### 280.5 The cause

`WalkingAWire::arrive`, `mesh.rs:2569`, splits the corner's chart position
with `let (raw, up) = sits_at;` at 2570. It unwraps `raw` by whole turns
(`the_short_way_round`, 3489, by the turn that `how_far_the_longitude_wraps`,
3478, gives for the **first** coordinate). It stores `up` exactly as it came,
`self.corners.push([longitude, up])` at 2618 and at 2639. On a sphere, a pipe
or a cone the second coordinate is a latitude or a height, and it never wraps.
**On a torus it is the angle round the ring**, `sideways.atan2(out)` at
`torus.rs:273`, which lies in `(−π, π]`. A wall whose ring angles run past `π`
is split there into one corner at `+π` and the rest near `−π`, and nothing puts
them back into one turn.

**Why a placed bend never meets it.** A face with a placement is meshed in the
frame it is stored in (`the_frame_this_face_is_meshed_in`). A bend put through
`place_solid` is therefore meshed as the unturned `0 → π/2`, whatever the
turn. The eighth-turned partner in `building_a_solid_out_of_faces.rs` is
placed, so it never reaches the cut. Only a bend written out where it sits
reaches it, and that is how a file or the builder writes one.

### 280.6 A gap, and the fix

**A gap in §16.10's sense.** The answer exists: the solid is the working bend
turned by an exact rotation, and the builder's sweep refuses it only at the
mesh. The arithmetic does not run out anywhere. The chart is read on the
wrong branch of `atan2`.

The fix is small and belongs in `mesh.rs`. It is described here so that it
can become a row, and it is not made here.

- Beside `how_far_the_longitude_wraps`, add a rule that says whether a
  surface's **second** coordinate wraps. It gives `Some(TAU)` for
  `Surface::Torus`, `None` for plane, sphere, pipe and cone, and the same panic
  on a kind it does not know.
- `WalkingAWire` remembers the last second coordinate. After the first corner,
  `arrive` stores `last + the_short_way_round(up − last, whole)` in place of the
  raw `up`. A torus has no pole, so only the branch for an ordinary corner
  needs it.
- `finish` measures how far the second coordinate climbed round the loop, the
  way it already does for the first. A loop that comes back a whole turn on in
  ring angle is a torus face that goes right round the ring. This mesher has
  no seam for that, so it refuses it as `AWireWillNotLieFlatOnItsFace` rather
  than meshing it.
- A torus face with more than one wire needs each later loop moved into the
  first loop's turn of ring angle, as `in_the_same_turn_as` does for the first
  coordinate. This lane did not check whether such a face reaches the mesher
  today. The fix either moves them or refuses the face, and does not leave it
  to the branch cut.

**The test that fails without it**, in `building_a_solid_out_of_faces.rs`
beside the working bend, or in `rings/mod.rs` as a turned copy of
`a_quarter_bend`. It writes out the working bend with `(x, y) → (−x, −y)` on
its corners, cap normals and seam half-space normal, as in 280.4. It asserts
that `SolidBuilder::build` is `Ok` and that `Model::validate` passes. Then it
checks that the volume agrees with the unturned bend's to within the deviation
both measurements report. Bit for bit is not claimed: the charts differ by
`π`, so `point_at` takes different sines. The half bend across `π` (`k = 1` in
280.1) builds too. Today both return
`Err(TwoPatchesOfTheMeshDrewTheSameSide { drawn: 1, used: 1 })`.

Once this lands, the fuse that §250.4 was waiting on can be tried: the bend
with its half-turned copy, the one partner that reaches the other disk of a
cap. It was not tried here.

### 280.7 Where the numbers come from

Every result and every chart position above comes from `mise exec -- just t -p
rcad-topo --test q119_scratch_turned_bend --no-capture`, run with the scratch
test and the temporary `eprintln!`s in `cut_up` and `finish` described in
280.1 and 280.2. Both were removed before this commit. The line numbers are
from `git show HEAD:crates/rcad-topo/src/mesh.rs | grep -n` and `git show
HEAD:crates/rcad-geom/src/torus.rs | grep -n`, at `b0744a1e`.

### 280.8 What Q190 made of 280.6

Q190 (`lane-q190-ring-angle-unwrapped`, 87c3ed43) made the fix 280.6 names.
`how_far_the_second_coordinate_wraps` gives `Some(TAU)` for a torus. The walk
unwraps the ring angle as it does the tube angle, and a wire that never
crosses `π` keeps its numbers bit for bit. A loop that climbs a whole turn
round the ring is refused as `AWireWillNotLieFlatOnItsFace`, and
`with_its_second_coordinate_in_the_turn_of` moves a face's later loops into
the first loop's turn. The quarter bend turned half a turn and the half bend
across `π` build and validate. Each matches the unturned bend's volume within
both measurements' errors.

**One refusal goes past 280.6.** A torus face whose one wire has a seam across
the tube, so that one point is two corners a whole turn apart round the ring,
is refused by the same name. The lane turned that refusal off and back on.
With it off, the crown of a ring folded (`TwoPatchesOfTheMeshDrewTheSameSide
{ drawn: 4, used: 4 }`), and a seamed half ring measured right. One fold
against one right answer, on a path where nothing cuts a patch round the ring,
so the refusal stays. The crown's pin in
`a_mesh_that_folds_at_one_deflection_and_not_the_next.rs` moves from
`CornersCoincide` to `AWireWillNotLieFlatOnItsFace`. It is still a gap: the
crown is an ordinary solid, and a mesher that cut round the ring as well as
round the tube would reach it. That cut is also what a fillet ring written
with a seam needs, which real files have. It is Q37's gap, now refused under
this name.

**Not settled.** Two loops side by side on a torus, each going round the tube
and neither inside the other, bound two bands. The move puts the later loop
the nearer way round, which is the wrong band when the face is the wider one.
Only the loops' directions can tell the two apart. The base chose by position
too, so this is not new. It is reasoned, not run, and is Q196. §343 ran it: it
was wrong, and the later loop is now placed by which way the first one runs.

**How it was checked.** The run's verifier read the diff and found the pin
above, but it ran out of room before its gate finished, and said fix first.
The dispatcher merged the branch, moved the pin, and ran the three new tests
in `building_a_solid_out_of_faces.rs` with `mesh.rs` checked out at the base
`b0623d41`. All three failed, and the turned bend was refused. With the fix
restored, all three passed (`./target/q190-revert.sh`, which runs `just t -p
rcad-topo --test merged -E 'test(/^building_a_solid_out_of_faces::/)'` on the three names). Then
`just gate quiet` ran on the merged tree.

Q40's relay file still does not import: `assert_pinned_kicad_imports` passes
unchanged. Whether its refusal moved was not checked; that is Q40's.
