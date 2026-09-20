## 169. A pie slice is where its arc leaves the corner it shares with a radius — §168.4's second item fixed, 2026-09-20

*An eighth of a ball answered `CannotFollowAFacesTrimming` about every point,
placed or not. Its three flat faces are pie slices — two radii and an arc — and
§167's step 2 could read an arc only against a cut holding **both** of its ends,
which is the half-disc a wedge has and not this. The cut meets the arc's own
circle in the end it holds and in one other point, so the arc's side of the cut
changes nowhere else: it leaves on one side and is still on it at its far end
unless it ran past that other point, and the far end's own side says whether it
did. Which way it leaves is `∇S × ∇T` turned to point into the arc, which is the
front of the arc's picker. So a sign already in hand answers how far the arc
runs, and no point on it is built. A gap closed, not a limit. The eighth answers
forty points, four placements, with an empty log.*

### 169.1 The cause, printed

The test below, run on `f34f8321` before the fix:

```text
assertion `left == right` failed: an eighth of a ball, placed by Isometry(Isometry { rotation: [0.0, 0.0, 0.0, 1.0], translation: [0.0, 0.0, 0.0] }), asked about [0.3, 0.3, 0.3]
  left: CannotFollowAFacesTrimming { face: FaceKey { slot: FaceSlot(1v1), model: ModelId(1) } }
 right: Inside
```

No movement is involved — this is the identity placement, and slot 1 is the
first pie slice, the quarter-disc on `z = 0`. §168's cause needed a shift or a
turn; this one is there where the solid is stored.

`where_a_side_lies` is the step. It asks, for each surface cutting the face,
where every *other* side of the face lies against it, and its arc arm read one
shape only:

```rust
let (Some(own_plane), [Decision::Zero, Decision::Zero]) = (own_plane, ends) else {
    return Lies::Across;
};
```

`Lies::Across` is a refusal at the call site. A wedge's flat face is a half-disc:
its one straight side is a diameter, both ends of its arc are on both walls, and
`[Zero, Zero]` is every pair it ever reads. A pie slice's arc has one end on each
radius's plane and the other end off it, so every pair it reads is `[Zero, Pos]`
or `[Pos, Zero]` — and so does every arc of the sphere face above it, because
each of those also runs from a cut to a cut.

### 169.2 The fix, and why the far end is enough

The arc runs on a circle, one of the two surfaces either side of it being a
plane and the other a sphere — `which_side_of_its_neighbour_a_side_keeps`
already refuses anything else before this step is reached. The cut is a plane
that holds one end of the arc, and it holds it by **topology**: that end is a
corner of an edge running along the cut, which §35.2's rule reads as on it
rather than asking a predicate.

A plane holding a point of a circle meets that circle in that point and in at
most one other. So along the circle the cut's sign changes only at those two
places, and the arc — a connected piece of the circle with that end as one of
its ends — has one sign from its end until it reaches the other place, and the
opposite sign after. Its far end is neither of those two places, since the far
end reads a settled `Pos` or `Neg`. So:

> the arc lies beside the cut exactly when the side it leaves its end on is the
> side its far end is on.

That is the whole of it. §168.4 guessed that how far the arc runs was "what the
arc's own picker would have to say"; it is not, and nothing needs to know where
the arc stops. The far end's sign is read already.

**Which way it leaves.** The way the crossing runs at that end is `∇S × ∇T`,
which is the same quantity `which_side_of_its_neighbour_a_side_keeps` reads, now
named once in `the_way_the_crossing_runs` and used by both. It points one way
along the circle or the other, and which of the two is the arc is settled by the
picker: the arc lies in front of its picker and this end is on the picker's
plane, so into the arc is whichever way reads positive against the picker's
normal. The side of the cut it leaves on is then

```text
sign( (∇S × ∇T · picker) · (∇S × ∇T · cut) )
```

— one interval product, and turning `∇S × ∇T` round turns both factors, so which
of the arc's two ends is the one on the cut never has to be worked out.

**Nothing an existing face answered moves.** The new arm fires only on end pairs
the old code returned `Lies::Across` for, and `Lies::Across` is a refusal. Where
the two signs disagree it still refuses — and there it is right to: the arc
really does cross the cut.

**Two grazes are refused rather than answered.** A `Zero` from either factor is
the cut touching the circle at that end rather than crossing it, or the picker's
plane doing so; both are sides that stay where they start, and both come back
`Lies::Across`. A gap, left for the face that needs it.

### 169.3 The test, and what it printed before the fix

`a_pie_slice_of_a_ball_says_which_points_are_inside_it_wherever_it_sits`, in
`crates/rcad-topo/tests/a_quarter_of_a_ball_built_face_by_face.rs`.
`an_eighth_of_a_ball` builds the eighth of the unit ball where every coordinate
is positive: four faces, six edges, four corners. Three of the faces are pie
slices of a quarter turn, on the three coordinate planes; the fourth is a
triangle of the sphere. Each arc is a quarter of a great circle, picked by a
half-space whose normal is ones and zeroes and whose offset is one, which reads
exactly `Zero` at both of its ends — `HalfSpace` keeps a caller's normal bit for
bit, so nothing there rounds.

Ten points, each at least `0.05` clear of all four faces, four each inside and
outside by the octant's own oracle, asked at four placements: unplaced, §168's
shift by `(0.1, −1/3, 2.7)`, §168's turn, and both. Before the fix the unplaced
eighth failed on its first point, printed in §169.1. After, the test prints
`16 points inside an eighth of a ball and 24 outside, every one answered`, with
the log empty at every one.

`mise exec -- just t -p rcad-topo` on the code commit `2ebe8554`: `757 tests
run: 757 passed, 1 skipped`. One test more than §168's 756, the new one.

`mise exec -- just gate quiet` on that tree with this section in it, read back:
`gate: PASS`, `2101 tests run: 2101 passed, 1 skipped`, `3 tests run: 3 passed,
0 skipped`, `doc tests: 25 passed, 0 failed, across 9 crate(s)`. One test more
than §168's 2100. No declared count moved.

### 169.4 What is left of §168.4

- **A pie slice** — done, above.
- **Fused arcs are still outside the family**, unchanged by this. The fuse bakes
  placement into world coordinates at entry (§17.3) and
  `the_half_space_both_ends_are_on` lowers the picker's offset until both ends
  read strictly in front, so an arc's ends are off its picker's plane where it is
  stored. This step needs that plane to hold the end it is asked about — it reads
  the direction into the arc as "the way that reads positive against the picker's
  normal", which is only the way along the circle when the end is on the plane.
  Admitting fused arcs still needs steps 1 and 2 rebuilt around ends *in front
  of* a picker rather than on it, which is the same work §168.4 named.
- **Cylinders and cones** are refused by name as before. Nothing here touched
  them, and they are a different shape of problem: their crossings with a plane
  are not circles, so "the cut meets the arc's circle twice" — the whole of
  §169.2 — says nothing about them. An ellipse would carry the argument over
  unchanged; a parabola or a hyperbola, which a cone also gives, would not,
  because the sign along an unbounded crossing can change once rather than twice.
- **Planes that miss a ball's centre** are refused by name as before.
- **An arc with neither end on the cut** is still refused, and is not this
  section's case: with nothing holding an end, there is no place along the circle
  where the argument can start.

Write set: `crates/rcad-topo/src/holds.rs`, the test file, this section, and the
pointer at the end of §168.4.

**§176 admits fused arcs, and the second item above was wrong in the same way
§168.4's sketch was.** Steps 1 and 2 did not need rebuilding: the fuse lowers the
picker's *offset*, and no step here ever read an offset. What had to go was the
acceptance test that asked for one.

