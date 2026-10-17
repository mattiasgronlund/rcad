## 249. A turned bar comes back from its STEP file a few ulps off: the frame, not the digits — 2026-09-26

*Queue row Q69, from §241.5 item 2. Lane `lane-q69-turned-bar-bits`, off
`05401ebe`. Three temporary test files did the measuring:
`crates/rcad-step/tests/zz_q69_probe.rs`, `zz_q69_collide.rs` beside it, and
`crates/rcad-geom/tests/zz_q69_quat.rs`. All three are deleted again. This
section is the only change. **The writer and the reader lose no bit.** Every
number in the file comes back exact. The bits move in `measure`, because the
bar going in is stored unturned with a turn on it, and the bar coming back is
stored turned with no turn on it, and a curved face is cut into triangles in
the frame it is stored in. It is a limit: two different turns write the same
placement and measure differently.*

  249.1   What was compared
  249.2   Every number in the file comes back bit for bit
  249.3   The first number that differs is a mesh point, and the frame puts it there
  249.4   The other two cases: a move alone passes, and the third had a turn
  249.5   A limit, and the demonstration
  249.6   What should change instead is the comparison
  249.7   What this leaves
  249.8   A circle drawn on a flat face moves with no placement: a gap

### 249.1 What was compared

The bar is built the way the mutator builds it:
`Model::new(Scale::millimetres(100.0))`, then
`add_round_bar(3.0, 5.0, &Isometry::identity())`, then `place_solid` with
`Isometry::rotation(&Axis::z(), FRAC_PI_4)`. Its `postcard` bytes go into
`one_model_decoded_and_written`, and the panic that comes back is §241.5's,
number for number:

```
a sound solid came back from its own file with a different volume or area: Measurement { volume: 141.3130858480765, volume_error: 0.23448492890640857, surface_area: 150.76324910914616, area_error: 0.2501042991108888, centroid: Point([-1.7763568394002505e-15, -5.551115123125783e-17, 2.499999999999997]), centroid_wobble: 0.006484013005436979 } went in, Measurement { volume: 141.3130858480765, volume_error: 0.2344849289064086, surface_area: 150.7632491091462, area_error: 0.2501042991108888, centroid: Point([-8.881784197001252e-16, 6.245004513516506e-17, 2.4999999999999987]), centroid_wobble: 0.006484013005436977 } came back
```

`postcard` keeps the placement exactly (`postcard keeps the placement bit for
bit: true`), so the model the harness sees is the one that was built.

The probe then wrote the model, read it back, and compared the two sides at the
harness's own deflection, `Deflection::for_scale(there.scale())`, which is
`0.005` here. It compared:

- every face's located surface, and for every coedge its two corners and its
  edge's run, in walk order;
- the shell's placement on each side;
- the mesh `SolidView::mesh` makes on each side, point by point and triangle by
  triangle.

Everything is printed with `{:?}`. That is the shortest text that reads back as
the same `f64`, so two printouts are equal exactly when the bits are.

### 249.2 Every number in the file comes back bit for bit

The writer's `real` prints `{:?}`, and the reader's tokenizer reads a number
with Rust's own `parse`, which rounds correctly. Some of the reals the bar's
file holds:

```
  file: #11=DIRECTION('',(0.0,0.0,1.0));
  file: #13=CIRCLE('',#12,3.0);
  file: #14=CARTESIAN_POINT('',(2.121320343559643,2.1213203435596424,0.0));
  file: #37=DIRECTION('',(0.0,0.0,1.0));
  file: #39=CYLINDRICAL_SURFACE('',#38,3.0);
  file: #41=CARTESIAN_POINT('',(2.121320343559643,2.1213203435596424,0.0));
```

And what came back:

```
every located number the same bits: true (9 lines)
placement going in Isometry(Isometry { rotation: [0.0, 0.0, 0.3826834323650898, 0.9238795325112867], translation: [0.0, 0.0, 0.0] }), coming back Isometry(Isometry { rotation: [0.0, 0.0, 0.0, 1.0], translation: [0.0, 0.0, 0.0] })
```

Nine lines: three surfaces and six coedges. So the writer does not change a
number on the way out, and the reader does not change one on the way in. The
reader does rebuild two things instead of taking them as written: `Axis::new`
scales the pipe's axis to length one, and `Plane::from_point_and_normal`
works each cap's offset out again from a point and a normal
(`crates/rcad-step/src/geometry.rs`). Neither rebuilt anything differently
here: the surface lines are among the nine that match. **The writer and the
reader are both cleared.**

What differs is the placement. The bar going in is stored as `add_round_bar`
built it: corner at `(3, 0, 0)`, pipe along `z` through the origin, and the
eighth turn on its shell (`place_solid` composes the movement onto the shell's
`loc`, `crates/rcad-topo/src/model.rs`). The bar coming back is stored at the
turned numbers, with no placement at all. The writer writes located numbers
(§191.2), and the reader bakes them in (§42.8). **So the first stored number
that differs is the placement itself.** Every stored corner differs after it,
but not by ulps: the two sides store the bar in different frames.

### 249.3 The first number that differs is a mesh point, and the frame puts it there

```
mesh points 252 vs 252, triangles 500 vs 500
mesh points differing: 214 (214 of the 252 on an edge or corner), worst 211 ulps, worst by 1.4641066137244252e-15
first differing point #0: going in [2.0129442839579097, 2.224422466547481, 0.0], coming back [2.0129442839579097, 2.2244224665474808, 0.0], ulps [0, 1, 0]
  as bits: ["0x40001a82886227a6", "0x4001cb9e01927902", "0x0000000000000000"] vs ["0x40001a82886227a6", "0x4001cb9e01927901", "0x0000000000000000"]
triangles with different corner indices: 330
volume 141.3130858480765 vs 141.3130858480765 (0 ulps); area 150.76324910914616 vs 150.7632491091462 (1 ulps); centroid Point([-1.7763568394002505e-15, -5.551115123125783e-17, 2.499999999999997]) vs Point([-8.881784197001252e-16, 6.245004513516506e-17, 2.4999999999999987])
```

Every point of a bar's mesh is on an edge. The 252 are the two corners and the
points walked round the two cap circles. 214 of them differ. The first is the
first point walked round the bottom circle, one ulp off in `y`. The 211 ulps
are on a coordinate near zero, where an ulp is small; by distance the most any
point moved is `1.46e-15`. The triangles differ as well: 330 of the 500 join
different points.

**Why, in the code.** `SolidView::mesh` (`crates/rcad-topo/src/mesh.rs`) cuts
each face up "in the frame it is stored in, and the points are moved into place
afterwards". For a placed face, that frame is the placement's inverse
(`the_frame_this_face_is_meshed_in`). The points are then carried out on the
placement itself (`back_out_of_the_frame_this_face_is_meshed_in`). So going
in, a point on the circle is worked out on the unturned pipe and then turned.
Coming back there is no placement, so the same point is worked out on the
turned pipe directly. Those are two roundings of the same true point, and they
land one ulp apart. `measure` adds up the mesh, so it adds up different
numbers.

The third hypothesis was that `measure` adds the same mesh up in a different
order. It does not: the mesh itself differs. And where the mesh does not
differ, `measure` does not either (249.4).

**A side finding.** The placed mesh is not even the unplaced bar's mesh,
turned:

```
the unplaced bar's mesh moved point by point differs from the placed mesh at 75 of 252 points, and from the read-back mesh at 209
corner Point([3.0, 0.0, 0.0]) moved out and back: Point([3.0, -4.440892098500626e-16, 0.0])
```

`BuildingAMesh::walk` does not give the walk the corners the arena stores. It
gives it `back.apply_to_point(end.point())`: the located corner, moved back
into the stored frame. The eighth turn out and back moves `(3, 0, 0)` by
`4.4e-16`. `the_surface_this_face_is_meshed_on` reads the surface straight out
of the arena for exactly this reason ("rounded twice for nothing"), and the
corners do not get the same treatment. It does not change this finding:
walking from the stored corner would still be a different frame from the one
the read-back meshes in.

### 249.4 The other two cases: a move alone passes, and the third had a turn

Each row is one model through `one_model_decoded_and_written` and the same
comparison. "Volume" and "area" are the ulps between going in and coming back.
In every row, every located number came back the same bits.

| model | harness | mesh points differing | triangles differing | volume | area |
|---|---|---|---|---|---|
| bar r3 h5, moved `(3, −5, 7)`, no turn | passes | 0 of 164 | 0 | 0 | 0 |
| bar r3 h5, turned an eighth about `z`, then moved `(3, −5, 7)` | panics | 141 of 164 | 273 | 4 | 1 |
| bar r3 h5, moved `(0.1, 0.2, 0.3)`, no turn | panics | 0 of 246 | 40 | 1 | 0 |
| cube 2 + bar r6 h4, moved `(9, −7, −15)`, no turn | passes | 0 of 206 | 0 | 0 | 0 |
| the same, turned 1/8 … 7/8 about `z` first | all seven panic | 138–202 of 206 | 272–340 | 0–7 | 0–5 |

Three things follow.

1. **A move by whole units, alone, does not fail.** §241.5 item 2 says "the
   same bar also moved by (3, −5, 7) fails the same way". Moved by `(3, −5, 7)`
   and nothing else, it passes. Turned and then moved, it fails, for the same
   reason as 249.3. The failing case must have been turned as well.
2. **Neither a turn nor a changed point is needed.** Moved by `(0.1, 0.2, 0.3)`,
   every mesh point comes back the same bits, but 40 triangles join different
   points and the volume is one ulp off. Deciding which way to split a face into
   triangles is done on the points in the frame the face is meshed in. That is
   the stored frame going in and the world coming back, so the same frame
   difference can flip a choice without moving a point. **Traced since, by
   Q108, and fixed in §362:** the flip was on the two flat caps, 20 of the 121
   triangles on each, and the decision was the triangulator's Delaunay test on
   four points of the cap circle, a true tie decided on the mesh points moved
   back into the stored frame. A flat face is now cut up where the solid sits,
   and this bar comes back cut and measured the same bits.
3. **The cube-and-bar crash had a turn in it, or a changed byte.** With no turn
   at `(9, −7, −15)`, the bar comes back the same bits; that row goes in at
   `452.10882097631367` and comes back at the same. Every eighth about `z` fails.
   None of them matches §241.5's pair exactly: the eighth goes in at
   `452.10882097631367` and comes back `452.1088209763138`, not
   `452.1088209763134`. The crash's bytes are gone.
   `fuzz/artifacts/no_model_makes_the_writer_panic/` is empty in all four
   worktrees that have one, so the turn cannot be read back out of them.

### 249.5 A limit, and the demonstration

§16.10 asks for the family of fixes and why none of it works. Three families
were considered:

1. **The writer's digits.** Nothing to fix. `{:?}` is already the shortest
   round trip, and 0 of the 9 located lines differ (249.2).
2. **The reader's parsing and rebuilding.** Nothing to fix, by the same nine
   lines.
3. **Put the frame in the file.** The one thing the file does not carry is the
   placement, and STEP can carry one. The writer would write the solid in its
   stored frame, plus the placement as an `AXIS2_PLACEMENT_3D` (an axis and a
   reference direction) or as the three axes of a
   `CARTESIAN_TRANSFORMATION_OPERATOR_3D`. The reader would then put that
   placement back on the shell instead of baking it in. That would reverse
   §42.8, and it still does not work. A placement is stored as a unit quaternion.
   A STEP placement is written from that quaternion's turned axes, and two
   different quaternions can turn the axes to the same numbers:

```
turn 0.7853981633974483 about Vector([[0.0, 0.0, 1.0]]): [0.9238795325112867, 0.0, 0.0, 0.3826834323650898] and [0.9238795325112868, 0.0, 0.0, 0.3826834323650898] write the same three columns
  a bar placed by each measures (141.3130858480765, 150.76324910914616, [-1.7763568394002505e-15, -5.551115123125783e-17, 2.499999999999997])
                             and (141.3130858480765, 150.7632491091462, [4.440892098500626e-16, -3.469446951953614e-17, 2.499999999999998]); same: false
870 of 4000 mutator turns have a different quaternion within two ulps that writes all three columns the same (564 of 1981 eighth-multiples)
```

The pair is this finding's own turn, and the same turn with `w` one ulp higher.
Both went through `Isometry::try_from`, which scales them to length one, so
both are placements a model can hold. "The same three columns" means that
`apply_to_vector` turns the three unit axes to the same nine numbers, bit for
bit. That is how the kernel turns a vector, and how every other located number
in the file is made. Written that way — the bar in the frame it is stored in,
which the two share, and the placement as those nine numbers — both turns give
the same file, so a reader handed it cannot tell which of the two it came from.
(Today's writer is not written that way: it turns every corner into the file,
and writes the corner `(3, 0, 0)` one ulp apart for the two turns.) The two bars
measure differently, so whichever turn the reader rebuilds, one of the two
models comes back with different bits. The search only nudged one of the four
numbers, by at most two ulps, so 870 of 4000 is a lower bound.

The obvious reader also misses most of the turns that have no twin. nalgebra's
`UnitQuaternion::from_rotation_matrix`, given the axes, rebuilds the eighth
about `z` exactly, but gets only two turns in five right:

```
eighth about z: source [0.9238795325112867, 0.0, 0.0, 0.3826834323650898]
  from_rotation_matrix: [0.9238795325112867, 0.0, 0.0, 0.3826834323650898] same bits true
from_rotation_matrix: 7918 of 20000 mutator turns came back the same bits (4467 of 10035 eighth-multiples)
```

**Two families not tried, and why.** One is to make the file carry the
quaternion's last bits some other way. That could be a property string, the
length of a `DIRECTION`'s ratios (a `DIRECTION` means only which way it
points), or turning the axes by a formula chosen so that this pair writes
different last bits. Each is a private channel between this writer and this
reader, because the reader would have to know how the writer rounded. The
second reader, in `a_second_reader_reads_what_this_writer_wrote.rs`, and every
other program would see nothing there. It would make the file carry rcad's
quaternion, not STEP's geometry. The other is to mesh a placed face in the world instead of in
its stored frame, so that a placed solid and its baked copy measure the same.
`mesh.rs`'s "Where the work is done" says the stored frame is the only thing
that works for a turned ball, and this lane did not test that claim either way.

**So this is a limit.** The comparison asks that a measurement worked out in
the stored frame survive, to the last bit, a file that cannot say which frame
that was. For some sound placed solids that answer does not exist, and the pair
above is one.

### 249.6 What should change instead is the comparison

Two kinds of solid measure the same whatever frame they are stored in:

- **A wholly flat solid** is measured over the triangles its faces fan into
  (`measured_over_its_own_fan`), and those use the located corners. Both sides
  have the same located corners, so the frame never enters. In every cube row of
  249.4 the cube came back `volume 8.0 vs 8.0 (0 ulps); area 24.0 vs 24.0 (0 ulps)`.
- **A curved solid at no placement** is stored in the same frame on both sides.

So `read_back_and_compared`, in `crates/rcad-step/tests/fuzzing/writer.rs`,
should check what the writer and the reader actually own. For every solid, that
is every located number of every face, corner and run, bit for bit, in walk
order — the comparison 249.2 made. That checks the file harder than three sums
do, because two different solids can share a volume, an area and a centroid.
The bit-for-bit `measure` comparison should stay for every solid that is wholly
flat or at no placement.

That leaves one thing uncompared: the measurement of a placed curved solid. The
way to keep it is to compare the read-back with a baked copy of the source.
Outside the fuse, the kernel has no bake. The tests in
`a_fuse_read_back_from_rounded_text.rs` bake by uniting with nothing, and
§129.5 kept the fuse out of this target. So that route needs a bake with this
comparison as its consumer (§13).

Neither route is taken here. §129.5 item 3 made the comparison bit for bit on
purpose, so changing it is a change to that decision.

### 249.7 What this leaves

- Change the comparison: every located number bit for bit for every solid, and `measure` bit for bit only for a solid that is wholly flat or at no placement, in `read_back_and_compared`, `crates/rcad-step/tests/fuzzing/writer.rs`, with §129.5 item 3 amended to match.
- Hold the limit in place with a test: the eighth turn about `z` and its twin with `w` one ulp higher write the same three columns and place a bar that measures differently, in a new `crates/rcad-step/tests/two_turns_that_write_the_same_placement.rs`.
- Decide whether `BuildingAMesh::walk` should give the walk a placed face's stored corners rather than its located corners moved back (249.3's side finding), in `crates/rcad-topo/src/mesh.rs`.
- ~~Find which triangle choice flips when a bar is moved by `(0.1, 0.2, 0.3)` and every mesh point stays the same (249.4 item 2), in `BuildingAMesh::cut_up`, `crates/rcad-topo/src/mesh.rs`.~~ Done: the caps' Delaunay tie, fixed in §362.
- Correct §241.5 item 2's "the same bar also moved by (3, −5, 7) fails the same way": moved alone it passes, and turned then moved it fails. The fix goes in `docs/decisions.d/241-the-writer-fuzz-target-mutates-models-rather-than-bytes.md`.

### 249.8 A circle drawn on a flat face moves with no placement: a gap

*Queue row Q2151, from Q2135, 2026-10-16. Three lanes: the first on
`lane-q2151-a-leaning-pipe-read-back-to-the-bit`, then two on
`lane-q2151-b-class-the-leaning-pipes-centroid`. Every probe was a scratch
test, deleted again.*

**249.6's second bullet is wrong for a solid with a circle drawn on a flat
face.** Such a solid at no placement is stored in the same frame on both
sides, as the bullet says, but its measurement can still change on the way
through the file. Two unplaced solids, every located number the same bits on
both sides:

| solid | kernel's circle centre | read-back centre | volume | area | centroid |
|---|---|---|---|---|---|
| a bar's top at `z = 4`, a disc on it drawn by a pipe up `z` from `z = −15.9` | `z = 3.9999999999999982` | `z = 4.0`, 4 ulps off | 1 ulp off | same bits | `z` 6 ulps off |
| a face square to `(1, 1, 1)` through `(4, 4, 4)`, a disc on it drawn by a pipe along `(1, 1, 1)` from the origin | `3.9999999999999996` in each | `4.000000000000001` in each, 2 ulps off | same bits | same bits | 16, 2 and 3 ulps off |

The first is `a_disc_drawn_by_a_pipe_from_far_below_its_top_comes_back_from_its_own_file`,
which measured `50.240477976741204` going in and `50.2404779767412` coming
back. The second is
`an_unplaced_leaning_disc_drawn_by_a_pipe_off_its_own_plane_comes_back_from_its_own_file`,
whose centroid went in at `(2.0000000000000084, 2.0000000000000013,
2.0000000000000018)` and came back at `(2.0000000000000155,
2.000000000000002, 2.000000000000003)`; 78 of its 304 mesh points differ.
Both are in `crates/rcad-step/tests/fuzzing/writer.rs`. A pipe along a
coordinate axis is no safer than one that leans: of the 400 origins
`z = k/10`, `k` from −200 to 199, 40 walk to a centre other than `4.0`.

**Where it moves.** The kernel stores the drawn pipe with its own origin.
`rcad_geom::walk`'s `where_a_plane_cuts_a_pipe` centres the circle at that
origin moved up the axis to the face, and that point rounds. The writer's
`Writing::an_arc_of_a_circle_on_a_flat_face` (`crates/rcad-step/src/write.rs`)
writes the `CIRCLE` at the same point, worked out the same way by
`where_a_plane_crosses_a_pipe`, whenever the pipe's own origin is not on the
face. The reader's `Walk::the_surface_that_names_the_curve`
(`crates/rcad-step/src/brep.rs`) names the run by a pipe whose origin is the
`CIRCLE`'s centre (`Axis::as_written(place.origin, …)`), and the walk centres
the circle again from there. That second walk need not land on the first. On
the `z` case, `4 − 3.9999999999999982` is exact and so is the sum, so it lands
on `4.0`. The writer's own doc already says the reader's walk "can land a few
ulps away"; what it did not say is that the measurement moves with it.

**Families of fix, each run on the probe in scratch code.**

1. **The writer writes the pipe's own origin.** Written literally, as the
   `CIRCLE`'s centre, the reader refuses the file: "a sound model was written
   as a file the reader refuses: what #80 describes is not a solid this kernel
   can hold", since the faces must hold the circle at its centre. Nothing
   else in the file can carry the origin today. There is no
   `CYLINDRICAL_SURFACE` for a drawn pipe (the one in the `z` bar's file,
   `#24`, is the bar's own wall), and `rcad-step`'s reader reads no
   `SURFACE_CURVE` that could tie one to the circle. Writing the pipe as a
   `CYLINDRICAL_SURFACE` beside the circle, at its own origin, is standard
   geometry, unlike §249.5's private channels, but it needs a reader for it,
   and it is not tried.
2. **The writer writes another centre the faces hold.** On the `z` case none
   works. Every one of the 2 097 153 origins within `2^20` ulps of `z = 4`
   walks to exactly `4.0`, never to `3.9999999999999982`: for `c` from 2 to 8,
   `4 − c` is exact and so is `c + (4 − c)`.
3. **The reader searches for an origin whose walk lands on the file's
   centre**, in a box of 4 ulps round it and in steps of `2^k` ulps along the
   axis, and builds the pipe there. On the leaning probe it moved the pipe's
   origin from `(3.9999999999999996, …)` to `(3.9999999999999982, …)` and the
   probe came back the same bits with no narrowing. So an origin other than
   the kernel's measures the same as the kernel's when it walks to the same
   centre. With `the_same_run` also calling such a pipe the same, a run of
   `rcad-step` and `rcad-fuzz-explain` stopped at its first failures, 212 of
   473 tests in, with 210 passed and 2 failed. The lane read the failures as
   the pin `how_often_a_turned_disc_drawn_on_a_bar_comes_back_from_its_own_file`,
   which now counted 84 of 372 axes moved; the other 261 tests were not run
   with the search. It is incomplete: of 4000 random
   leaning pipes, 2474 have a centre that does not walk back to itself, and
   the search found an origin for 2138 of them and none for 336; of the 56
   origins `z = k/10`, `k` from −400 to 399, that do not, it found 32 and
   none for 24. A box of 4 ulps alone found 986 of the 2474.
4. **Narrow the comparison.** The first lane left out every solid with a
   drawn pipe that does not run along a coordinate axis. That is not enough:
   the `z` case runs along `z` and fails. `every_drawn_circle_comes_back_round_the_same_centre` leaves out a
   solid only where the reader's pipe walks a circle on a flat face to other
   bits than the kernel's, and `read_back_and_compared` keeps it as a stopgap,
   not as the answer.
5. **Not tried: make the walk a fixed point**, so that a circle's centre,
   walked again from itself, lands on itself. Then the file's centre would
   read back as the kernel's. It changes how `rcad_geom::walk` centres every
   circle a plane cuts from a pipe.

**So this is a gap, class wrong/B.** The answer exists: the kernel's own
origin walks to the kernel's centre, and on the probe an origin the reader
found did too and measured the same bits. §249.5's argument does not carry
over. There, two placements write one file and measure differently, so no
reader can tell which one to rebuild. Here, two origins that walk to one
centre measured the same, so a file that names the centre names the
measurement, if the reader can find an origin that walks there or the file
says which. The `z` shape, a disc drawn on a bar's top by a pipe up its axis
from below, is a shape a designer makes on purpose, so it is B.

**What this leaves.**

- ~~Make a circle drawn on a flat face come back walked round the centre the kernel walks it round, by family 1 with a reader for it, or by a search that always finds an origin, in `crates/rcad-step/src/write.rs`, `brep.rs` and `geometry.rs`. Then take `every_drawn_circle_comes_back_round_the_same_centre` out of `read_back_and_compared`, and the two tests above are its failing tests. Filed as a row from Q2151.~~ Done: family 1, landed as Q2162. `Writing::an_arc_of_a_circle_on_a_flat_face` writes the model's own pipe as a `CYLINDRICAL_SURFACE` beside the circle and names the `CIRCLE` after it; the reader's `Reading::pipe_a_circle_names`, called from `Walk::the_surface_that_names_the_curve`, takes that named pipe rather than searching the file, because a search can find another solid's pipe instead — `two_discs_drawn_by_pipes_from_two_origins_on_one_line_come_back_from_their_own_file` names one solid's circle by the other's pipe 6 times in 301 placements when the first matching pipe is taken. `every_drawn_circle_comes_back_round_the_same_centre` is gone from `read_back_and_compared`.
- ~~§129.5 item 3 says `read_back_and_compared` keeps the bit-for-bit measurement for every solid wholly flat or at no placement; it now also asks that every circle drawn on a flat face come back walked round the same centre. The fix goes in `docs/decisions.d/129-the-night-used-one-core-of-24-changed-2026-09-18.md`, filed as a row from Q2151.~~ Done: below, in this row.
