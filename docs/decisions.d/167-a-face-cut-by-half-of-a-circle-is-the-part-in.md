## 167. A face cut by half of a circle is the part in front of every cut, or of any — §166.4's classify refusal fixed, 2026-09-19

*`classify_point` answered `CannotFollowAFacesTrimming` on every point of every
wedge of a ball built face by face. Slot 1 is the sphere face, and it is named
only because it is the first face the ray caster walks: the two half-discs
refuse too, the sphere face because its meridians have two different ends and
each half-disc because one of its sides is not straight. So nothing about the
wedges was ever asked; §35.11 files both refusals as gaps, and the answer exists
and is exact. The fix follows one family of faces bounded by half of a circle: a
face that is the part of its surface in front of every surface that cuts it —
or, on a sphere, in front of any — which is proved of the face in hand before it
is used, with the side each cut keeps read off the way the wire runs because
every corner of a lune sits on every cut. The four wedges now answer forty
points inside and out with an empty log.*

### 167.1 The cause, printed

A probe beside the test, since removed, built the quarter at `[0°, 90°]` and
asked each face whether it held four points, then classified `(0.3, 0.3, 0)`.
Abridged, with the keys shortened:

```text
face FaceKey { slot: FaceSlot(1v1), .. } surface Sphere(Sphere { centre: Point([0.0, 0.0, 0.0]), radius: 1.0 }) reversed false
   holds Point([0.3, 0.3, 0.0]): Err(AFaceTrimmedByACurveThisCannotFollow { face: .. FaceSlot(1v1) .. })
   (and the same for the other three points)
face FaceKey { slot: FaceSlot(2v1), .. } surface Plane(Plane { normal: Vector([[0.0, -1.0, 0.0]]), offset: 0.0 }) reversed false
   holds ...: Err(AFaceTrimmedByACurveThisCannotFollow { face: .. FaceSlot(2v1) .. })   (all four)
face FaceKey { slot: FaceSlot(3v1), .. } surface Plane(Plane { normal: Vector([[-1.0, 0.0, 0.0]]), offset: 0.0 }) reversed false
   holds ...: Err(AFaceTrimmedByACurveThisCannotFollow { face: .. FaceSlot(3v1) .. })   (all four)
classify: CannotFollowAFacesTrimming { face: FaceKey { slot: FaceSlot(1v1), .. } }
```

Every face refuses whatever point it is asked about, which is why the answers
did not move with §166's fix and did not depend on the point. The two lines that
give up, both in `holds.rs`:

- **The sphere face**: `what_trims_this_face` returns
  `AFaceTrimmedByACurveThisCannotFollow` for any edge whose two ends are
  different corners. A meridian runs north pole to south pole. §35.1's route
  reads a curved face as one of the two pieces a *whole* closed crossing cuts
  its surface into, and a meridian is half of one.
- **Each half-disc**: `walk_one_outline` returns the same error for any side
  that is not `EdgeRun::Straight`. The half-disc's rim is half a circle.

`count_crossings` walks faces in stored order and returns the first refusal, so
the sphere face at slot 1 is what `Containment` names.

**A gap and not a limit.** A point clear of the skin is inside or outside and
nothing about it is near a boundary; the answer exists in the stored numbers.
What was missing was a way to say which side of a *part* of a crossing a face
lies on, and §35.11 already filed both refusals as gaps.

### 167.2 The fix

`what_a_face_cut_by_part_of_a_crossing_holds`, in `holds.rs`, taken by any face
with a side that is part of a crossing and is not a seam. It is §35.1's route
with the precondition it can no longer assume shown instead:

1. **Which side of each neighbour the face keeps is read off the way its wire
   runs.** §35.2's corners cannot say: both corners of a lune sit on both walls.
   A wire keeps its material on its left, and left of `∇S × ∇T` is always `T`'s
   negative side (§36.6), so the sign of the side's direction against
   `∇S × ∇T` at its start says which side. A straight side runs from start to
   end; an arc leaves its start into the front of its picker, whose plane holds
   that start, so the picker's normal stands in for the direction. Interval
   arithmetic over stored numbers, with `Error::Undecidable` on a straddle.
2. **Every side must then lie on that side of every other neighbour** — the
   face is *convex* against its cuts. A corner of an edge along a cut is on it
   by topology (§35.2's rule). An arc whose two ends are both on a plane lies
   strictly one side of it, and which side is a sign: the arc's own plane meets
   both that plane and its picker's in the line through the two ends, so
   across the arc's plane the two read the same sign or opposite ones
   everywhere, and `(n × p) · (h × p)` says which.
3. **Then the face is the part of its surface in front of every cut.** The
   region behind a cut is connected, holds none of the face's boundary, and
   borders the face's own side along that cut on the side the face does not
   keep — so the face does not reach into it. And the part in front of every cut
   is connected with the face's boundary on its boundary, so the face is all of
   it.
4. **On a sphere, the other way round works too.** If every side lies *behind*
   every other cut, the rest of the sphere is the part in front of none, and the
   face is the part in front of any. That is the three quarters.

Connectedness is what limits the family, and it is checked, not assumed: a flat
face's cuts must be planes or balls it lies inside (half-planes and discs meet
in one convex piece), and a sphere face's cuts must be planes through the
centre (halves of a sphere meet in one piece, where caps can meet in two). Each
arc must be picked by a half-space whose plane holds both its ends exactly, so
the arc is the whole of its circle in front of that plane. Anything else is
refused by name as before.

The route is only taken by faces that were refused before — a side with two
different ends that is part of a crossing and not a seam — so no face that was
answered is answered differently.

### 167.3 The test, and what it printed before the fix

`a_wedge_of_a_ball_says_which_points_are_inside_it_wherever_it_sits`, in
`crates/rcad-topo/tests/a_quarter_of_a_ball_built_face_by_face.rs`: ten points,
two of them outside the ball, asked of the quarter at `[0°, 90°]`,
`[−45°, 45°]` and `[135°, 225°]` and of the three quarters. The oracle is where
the point is — inside the unit ball and swept from the first wall by less than
the wedge's turn — and it refuses any point within `0.05` of the skin, so
reading it off `atan2` decides nothing near a boundary. The certified log must
stay empty. It counts both answers and insists on ten of each.

§166.4's own points were not all usable. `(0.3, 0.3, 0)`, `(−0.3, −0.3, 0)` and
`(0.3, −0.3, 0.1)` lie exactly on a wall of the diagonal wedges, which the oracle
refused on its first full run, so they moved to `(0.35, 0.2, 0)`,
`(−0.2, −0.35, 0)` and `(0.35, −0.15, 0.1)`.

With the new route switched off, the test fails on its first point:

```text
assertion `left == right` failed: the wedge between [[1.0, 0.0], [0.0, 1.0]] asked about [0.35, 0.2, 0.0]
  left: CannotFollowAFacesTrimming { face: FaceKey { slot: FaceSlot(1v1), model: ModelId(1) } }
 right: Inside
```

After: `15 points inside a wedge and 25 outside, every one answered`. The three
quarters are the case that tells steps 3 and 4 apart. Reading them through step
3 would call `(−0.4, 0.3, 0.2)`, which is inside them, outside.

`check-wildcards` moved while this was written and is back at `87`. The first
draft matched on tuples of `Surface` and an array of `Decision`, which the
composite scan counts. The matches were rewritten to name what they match, and
the ledger was not raised. No other declared count moved.

`just gate quiet` on the code commit, read back: `gate: PASS`, `2099 tests run:
2099 passed, 1 skipped`, `3 tests run: 3 passed, 0 skipped`, `doc tests: 25
passed, 0 failed, across 9 crate(s)`.

### 167.4 What is left

- **A placed wedge still refuses.** A picker's plane must hold both ends of its
  arc *exactly*, and an `Isometry` rounds both the corners and the picker, so a
  turned or moved wedge will usually miss that and be refused by name. The same
  choice between topology and a predicate that §19.1 describes, with nothing
  topological to fall back on yet: nothing records that an arc's ends are on its
  picker's plane.
- **A pie slice is refused.** A sector of a disc less than a half-turn is
  convex, but its arc has only one end on each radius's plane, and step 2 cannot
  tell which side of that plane the arc bulges without building a point on it.
- **Fused arcs are mostly outside the family.** The fuse lowers a picker's
  offset until the predicate agrees both ends are in front
  (`the_half_space_both_ends_are_on`), so its planes seldom hold the ends
  exactly. No fuse test moved, and none was expected to.
- **Cylinders, cones and planes that miss a ball's centre** are refused by name.
  Each one breaks either a step's sign or the connectedness step 3 rests on.

Write set: `crates/rcad-topo/src/holds.rs`, the test file, this section, and the
pointer at the end of §166.4.

**§168 answers the first item**: the picker check and the centre check both
broke under a movement, and both are now asked where the face is stored. A
turned or moved wedge says which points are inside it.

