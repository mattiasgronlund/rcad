## 168. A placed wedge is proved where it is stored and asked where it sits — §167.4's first item fixed, 2026-09-19

*Turned or moved, a wedge of a ball answered `CannotFollowAFacesTrimming` for
every point, because §167's route asked the face as it sits for two facts only
exact arithmetic can hold: that each arc's picker plane holds both of the arc's
ends, and that each wall runs through the sphere's centre. A movement rounds
both. Every fact that route proves is one a movement keeps, so it now proves them
of the face where it is stored — its stored surface, the stored runs, the stored
corners — where they are exact, and asks only the point against the cuts as they
read once moved. A gap closed, not a limit: the answer existed, and no new
topological record was needed. The placed wedges answer the same forty points as
the unplaced ones, with an empty log.*

### 168.1 The cause, printed

The test below, run on `3b3949b2` before the fix:

```text
assertion `left == right` failed: the wedge between [[1.0, -1.0], [1.0, 1.0]], placed by Isometry(Isometry { rotation: [0.0, 0.0, 0.0, 1.0], translation: [0.1, -0.3333333333333333, 2.7] }), asked about [0.35, 0.2, 0.0]
  left: CannotFollowAFacesTrimming { face: FaceKey { slot: FaceSlot(1v1), model: ModelId(3) } }
 right: Inside
```

The quarter at `[0°, 90°]` passed under the same shift — its walls and pickers
are axis-aligned — and the diagonal one is the first to fail. A shift alone is
enough. A probe beside the test, since removed, read the placed diagonal
quarter's two meridians through the public views (`EdgeView::run`,
`VertexView::point`) and asked each picker about its arc's ends and each wall
about the ball's centre:

```text
moved: picker reads the ends [Neg, Neg]; wall reads the centre Pos
moved: picker reads the ends [Zero, Zero]; wall reads the centre Pos
turned: picker reads the ends [Neg, Pos]; wall reads the centre Pos
turned: picker reads the ends [Neg, Pos]; wall reads the centre Neg
```

Where the wedge is stored every one of those reads `Zero` (§167's own test, which
needs them to, passes). So both of §167.2's exactness preconditions break — the
picker in `the_sides_of` and the centre in `is_a_family_this_follows` — and the
first one asked refuses. §167.4 named the first and not the second.

### 168.2 The fix

**What had to be exact is shape, and a movement does not change shape.**
§167.2's steps ask whether a plane holds two points, whether a plane holds a
centre, which way a wire runs against a cross product, and which side of a plane
an arc lies on. A turn and a shift — an `Isometry` is a rotation and a
translation, never a mirror — keep every one of those answers in exact
arithmetic. What loses them is *computing* the moved surface, corner and picker,
each rounded on its own.

So `what_a_face_cut_by_part_of_a_crossing_holds` now reads everything it proves
from the frame the face is stored in: `FaceView::stored_surface` for the face
and for each face across a side, `EdgeView::stored_run` for the picker, and each
corner's stored point, read from the arena by `where_it_is_stored` because
`VertexView` only hands out the moved one and moving it back would round it a
second time. Only the last step — which side of each cut the *point* is on —
uses the cuts as they read once moved (`Cut::placed`), because the point was
asked in the world.

**Why the answer is still about the placed face.** The stored face is proved to
be the part of its surface in front of every stored cut, or of any. The placed
face is its image under the movement, so it is exactly the part in front of
every moved cut. The moved cuts as computed are rounded, which is §17.3's
representation wobble and the same wobble every other face of a placed solid is
classified against: this face is held to its neighbours' standard, and no
longer to one it cannot meet.

**One precondition is new, and checked.** The face across every side must be
stored at the face's own placement, or the two stored frames are not one frame.
Faces of one shell always are. A side whose neighbour is placed differently is
refused by name, `AFaceTrimmedByACurveThisCannotFollow`, rather than compared
across frames.

§167.4 said there was "nothing topological to fall back on yet: nothing records
that an arc's ends are on its picker's plane". The stored numbers record it
exactly; the route only had to stop reading the moved copies of them. The family
of fix §167.4 pointed at — a topological record — was not needed.

### 168.3 The test, and what it printed before the fix

`a_moved_or_turned_wedge_of_a_ball_says_which_points_are_inside_it`, in
`crates/rcad-topo/tests/a_quarter_of_a_ball_built_face_by_face.rs`. §167's test
body became `wedges_say_which_points_are_inside_them(movement)`, which places
each of the four wedges with `Model::place_solid` and asks the ten points moved
by the same movement, against the same unmoved oracle. §167's test calls it with
`Isometry::identity()`; the new one calls it three times:

- **moved** by `(0.1, −1/3, 2.7)`, which no power of two holds, so the corners,
  the pickers' offsets and the walls' offsets all round;
- **turned** by `0.7` about an axis through `(0.3, −0.2, 0.5)` along `(1, 2, 3)`,
  which misses the ball's centre and lies along no coordinate axis;
- **both**, the turn and then the shift.

Every point is at least `0.05` clear of the skin before it is moved, and moving
it rounds it by an ulp or two, so the oracle's answer stands. Before the fix the
test failed on the moved diagonal quarter's first point, printed in §168.1.
After, each of the three movements prints `15 points inside a wedge and 25
outside, every one answered`, with the log empty.

`mise exec -- just t -p rcad-topo` on the code commit `544d63ed`: `756 tests
run: 756 passed, 1 skipped`.

`just gate quiet` on the code commit `544d63ed`, read back: `gate: PASS`,
`2100 tests run: 2100 passed, 1 skipped`, `3 tests run: 3 passed, 0 skipped`,
`doc tests: 25 passed, 0 failed, across 9 crate(s)`. One test more than §167's
2099, the new one. No declared count moved.

### 168.4 What is left of §167.4

- **A placed wedge** — done, above.
- **A pie slice is still refused**, and the stored frame makes nothing new exact
  for it. The radius's plane holds one end of the arc by topology — the corner
  the two edges share — but step 2 asks where the whole arc lies against it, and
  with one end on it that turns on which way the arc leaves that end and whether
  it runs more than a half-turn. The direction it leaves in is the same
  `∇S × ∇T` sign `which_side_of_its_neighbour_a_side_keeps` already reads; how
  far it runs is what the arc's own picker would have to say. A sketch of the
  next lane, not a proof.
- **Fused arcs are still mostly outside the family**, and this does not help
  them. The fuse bakes placement into world coordinates at entry (§17.3), so a
  fused face's stored frame *is* the rounded world frame, and
  `the_half_space_both_ends_are_on` lowers the picker's offset until both ends
  read strictly in front, so they are off its plane where it is stored too.
  Admitting them needs steps 1 and 2 to work from ends *in front of* the picker
  rather than on it.
- **Cylinders, cones and planes that miss a ball's centre** are refused by name
  as before; nothing here touched them.

Write set: `crates/rcad-topo/src/holds.rs`, the test file, this section, and the
pointer at the end of §167.4.

**§169 answers the second item**, and the sketch above was half right: the
direction the arc leaves in is the `∇S × ∇T` sign, as it says, but how far the
arc runs did not need the picker to say — the far end's own side of the cut says
it, because the side can change only where the cut meets the arc's circle again.

