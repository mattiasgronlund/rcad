## 362. A placed flat cap is cut like its read-back: a flat face is cut into triangles where the solid sits — 2026-09-29

*Queue row Q645, re-filed from Q358, from §249.4 item 2 and Q108's trace. Lane
`lane-q645-a-placed-flat-cap-cut-like-its-read-back`, off `65facd7e`. The fix
is in `crates/rcad-topo/src/mesh.rs`, and
`crates/rcad-step/tests/a_flat_cap_is_cut_up_the_same_after_its_own_file.rs`
holds it. **A gap, and fixed**: the answer the read-back gives existed, and the
placed cap missed it by handing the triangulator other numbers. A flat face is
now cut up on its plane where the solid sits. A curved face is still cut up in
the frame it is stored in.*

  362.1   What flipped, and on which numbers
  362.2   The fix, and why only a flat face
  362.3   What else moves for a placed flat face
  362.4   A turned flat face, on a cube
  362.5   What this leaves

### 362.1 What flipped, and on which numbers

§249.4 item 2's bar: radius 3, height 5, built at the origin and moved by
`(0.1, 0.2, 0.3)` with no turn. Every mesh point comes back from its STEP file
the same bits, and the caps still came back cut differently. On the base, the
new test says:

```
the cap on the plane [0.0, 0.0, 1.0, -5.3] is cut into 121 triangles going in and 121 coming back, and 20 of them join other points
volume 141.31019364025335, area 150.76161009924724, centroid Point([0.09999999999999876, 0.1999999999999999, 2.7999999999999976])
volume 141.31019364025337, area 150.76161009924724, centroid Point([0.09999999999999787, 0.1999999999999999, 2.7999999999999967])
```

The decision that flipped is the triangulator's Delaunay test: spade's
`incircle`, reached from `BuildingAMesh::hand_over` through
`rcad_mesh::triangulate`. Q108's probe, copied and run again against this base,
replays spade one point at a time on both sides and stops at the first
insertion that differs:

```
bottom cap: first differing insertion: corner 6 (mesh point 116), only going in [[3, 4, 5], [3, 5, 6]], only coming back [[3, 4, 6], [4, 5, 6]]
  going in: corner 3 against the circle through [4, 5, 6] (ccw-ordered): incircle 1.3509359314543423e-19 (inside)
  coming back: corner 3 against the circle through [4, 5, 6] (ccw-ordered): incircle -4.6817410868991425e-20 (outside)
  world xy: corner 3 against the circle through [4, 5, 6] (ccw-ordered): incircle -4.6817410868991425e-20 (outside)
top cap: first differing insertion: corner 15 (mesh point 136), only going in [[0, 9, 15], [9, 11, 15]], only coming back [[0, 9, 11], [0, 11, 15]]
  going in: corner 0 against the circle through [9, 11, 15] (ccw-ordered): incircle -8.58020621386197e-18 (outside)
  coming back: corner 0 against the circle through [9, 11, 15] (ccw-ordered): incircle 9.870714279828575e-17 (inside)
  world xy: corner 0 against the circle through [9, 11, 15] (ccw-ordered): incircle 9.870714279828575e-17 (inside)
corners handed over the same bits: 0 of 123   (each cap)
```

On the bottom cap the four points are corners 3 to 6 of the loop, four points
in a row round the circle. On the top cap they are the model's corner and three
walked points that are not in a row. Either way all four lie on the cap circle,
so in true geometry the test is exactly zero. Only the last bits of the four
points decide it.

The two sides handed over different last bits. `walk_a_wire_round` moved every
point back into the frame the face is meshed in before reading its plane
coordinates. For the placed cap that frame is the placement's inverse, a shift
by `(−0.1, −0.2, −0.3)`, and each shifted coordinate is rounded. So the placed
cap was cut up on points one rounded subtraction off the mesh points. The cap
read back has no placement, and a plane is never shifted
(`a_shift_that_loses_nothing`), so it was cut up on the located plane's
coordinates of the mesh points themselves. The predicate is exact on both
sides; it was handed different numbers. The world points agree with the
read-back, in sign and in value, so the read-back's answer is the true answer
for the mesh points the triangles are made of.

### 362.2 The fix, and why only a flat face

Three functions in `mesh.rs`, beside the ones they fall back to:
`the_surface_this_face_is_cut_up_on`, `the_frame_this_face_is_cut_up_in` and
`back_out_of_the_frame_this_face_is_cut_up_in`. For a flat face they give the
located plane (`FaceView::surface`) and no movement either way. For a curved face
they give what the mesher used before: `the_surface_this_face_is_meshed_on`,
`the_frame_this_face_is_meshed_in` and
`back_out_of_the_frame_this_face_is_meshed_in`. `cut_up` reads the first,
`walk_a_wire_round` the second, and `hand_over` the third.

Q108's replay of this fix, run again at the base, hands the triangulator the
located plane's coordinates of the mesh points and gives
`located-frame corners are the read-back's bits: true; triangulated there, going in gives the read-back's triangles: true`
for both caps. With the fix in, the new test gives the same triangles on both
caps and the same volume, area and centroid, bit for bit, going in and coming
back. Both of its tests fail on the base `mesh.rs` with the messages in 362.1.

The wall is still cut up in a different frame on each side: the bar's stored
frame going in, and a shift that loses nothing coming back. For this bar that
costs nothing. The same probe reads
`wall points whose coordinates are the same bits both sides: 246 of 246`. So
once the caps agree, nothing is left to tell the two bars apart. The error
bounds `measure` prints beside the three sums still differ
(`volume_error: 0.2460690134578304` going in, `0.24606901345931106` coming
back), because they are charged for how far each side's frame can round, and
the test does not compare them.

**Why a flat face can move and a curved one cannot.** A curved face's frame
decides where its points go. The triangulator puts points inside a curved
patch, and the walk puts points along its edges. `SolidView::mesh` says why a
ball has to be cut in its stored frame, and §249.5 shows that a placed curved
solid measuring differently from its read-back is a limit. A flat face makes
no points. Its corners are the model's. The points round a circle on it were
walked by the curved face along that circle, or by the plane the circle's run
names. And the triangulator adds nothing to a plane: `how_far_a_point_lands_off_its_surface`
in `crates/rcad-mesh/src/triangulate.rs` says so, and `a_step_that_stays_within`
gives a plane an infinite step. So the only thing a flat face's frame decides
is how a tie goes. Where the solid sits is the one frame every copy of a flat
face shares: the placed solid, its read-back (the reader bakes the placement
in, §42.8), and a fuse's answer (a fuse bakes it in too, §17.3).

**Why not change `the_frame_this_face_is_meshed_in`.** `walk` reads it for a
circle imprinted on a flat face, which only that flat face can place. That walk
makes points, and moving its frame would move them. The floor and the room a
mesh works to (`how_far_moving_this_face_rounds` and the functions beside it)
read it as well. So the change is confined to cutting a face up.

**What it gives up.** A flat face is no longer cut the same wherever it is put:
move a flat face, and a tie can go the other way. `SolidView::mesh`'s "where a
solid sits must not change how it is cut up" now says so of curved faces only,
and says what a flat face does instead. The triangles of a flat face cover the
same flat region whichever way a tie goes, so what can move with the placement
is the last bits of a sum over them, not the shape.

### 362.3 What else moves for a placed flat face

By the code, nothing but the triangles:

- **The point table.** A corner goes in through `remember_the_corner` from
  `VertexView::point`, and a walked point through `walk`. Neither changed, and
  `hand_over` adds no point to a plane.
- **`Mesh::deviation`.** A flat patch strays by zero
  (`how_far_a_flat_piece_strays` gives a plane `0.0`), and
  `how_far_moving_this_face_rounds` is untouched.
- **A refusal.** Not measured. The triangulator's refusals that read the last
  bits of a corner — `CornersCoincide`, `BoundaryCrossesItself`,
  `CoordinateOutOfRange` — now see a placed flat face's world coordinates. That
  is what its read-back has always been handed, so a placed flat face refused
  where its read-back is not would be a new disagreement. No shape of one was
  built.

The whole workspace, run with `mise exec -- just t --workspace --no-fail-fast`
on the fix before the cube test was added: `2804 tests run: 2804 passed (3
slow), 11 skipped`. No pin on a volume, a triangle count or a mesh moved.

### 362.4 A turned flat face, on a cube

The row said a turned flat face was not demonstrated, and a bar cannot show
one: turned, its wall comes back cut in another frame (§249.3), its mesh points
differ, and comparing its caps by the points they join says nothing. A cube can.
Its mesh points are its eight corners, which the writer writes where they sit,
so they come back the same bits. Each face is a square, and a square's four
corners are a tie for the Delaunay test, so which diagonal a face is cut along
is the last bits' choice.

`a_turned_cube_has_every_face_cut_up_the_same_after_its_own_file` turns a cube
of side 2 by each of 1 to 7 eighths about `z`, `(1, 2, 3)` and `(−2, 0.5, 1)`,
moves it by `(3, −5, 7)`, and reads it back. On the base, 17 of the 21 came
back with faces cut along the other diagonal, from one face to four:

```
turned cubes cut up differently after their own file: ["1 eighths about Vector([[0.0, 0.0, 1.0]]): 2 faces", "2 eighths about Vector([[0.0, 0.0, 1.0]]): 2 faces", "4 eighths about Vector([[0.0, 0.0, 1.0]]): 2 faces", "1 eighths about Vector([[1.0, 2.0, 3.0]]): 1 faces", ...]
```

With the fix, none do. Every one measured the same bits on both sides even on
the base, because a wholly flat solid is measured over its own fan and not its
mesh (§249.6). So on a cube the flip showed only in the mesh.

### 362.5 What this leaves

- §249's limit stands for curved faces: a placed curved solid can still
  measure differently from its read-back (§249.5), and
  `read_back_and_compared` still compares a curved solid's measurement only
  when it has no placement (§249.6).
