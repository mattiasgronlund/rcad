## 243. A bend's wall between two planes through the ring's axis — 2026-09-25

*Row Q54, second lane (`lane-q54-torus-face-between-two-planes`, off
`ace6375f`). §232.5 named the classifier's refusal of a bend's torus face as
the next stop for both bend pins. `holds.rs` follows that face now, and both
pins move on to places §232.5 did not predict.*

  243.1   What refused
  243.2   The rule, and which legs held
  243.3   Which side a plane keeps is read off the wire, not a corner
  243.4   A puck's rounded rim was answered wrongly and is refused now
  243.5   Where both bends stop now
  243.6   What this leaves
  243.7   A bend whose axis is off the coordinate axes is refused

### 243.1 What refused

The first lane on this row put a temporary `eprintln!` in `classify.rs`'s two
`CannotFollowAFacesTrimming` arms. Both bend pins came through the ray-root
arm, from `what_this_face_holds` into `what_a_curved_face_holds`, where
`every_piece_of_that_crossing_is_an_edge(torus, cap plane, 1)` came back false
and the face was refused with `AFaceMissesPartOfWhereItsSurfacesCross`. A plane
through a ring's axis cuts it in **two** circles, and a bend's wall has one of
them as an edge. The root itself was found exactly. The answer exists, so this
is a gap (§16.10).

### 243.2 The rule, and which legs held

`is_a_ring_face_between_two_planes` sends a face on a ring with exactly two
trimming surfaces, both planes, one edge along each, to
`what_a_ring_face_between_two_planes_holds`. That reads the face where it is
stored (§168) and:

1. requires each plane to hold the ring's axis exactly — normal square to the
   axis direction and the centre on the plane, the two questions `intersect`
   asks before it names the two circles round the tube;
2. reads which side of each plane the face keeps off the way its wire walks the
   circle on that plane (243.3);
3. requires each edge circle to lie strictly in front of the other plane, read
   at the circle's corner — a circle round the tube sits at one angle round the
   axis, so its corner's side is the whole circle's;
4. then answers `point` with `held_in_front_of_all` against the planes as they
   read once moved into place.

The argument: with only those two circles as its edges, the face is one of the
two stretches of ring between them, and the side kept at each circle says
which. Step 3 holds exactly when that stretch is less than a half-turn, and
then the stretch is the part of the ring in front of both planes. A stretch of
more than a half-turn is the part in front of *either* plane, and step 3 is
what refuses it. A plane stored twice, or two stored planes that are one plane,
put a corner on the other plane and are refused by step 3 too.

The dispatched brief listed six legs. Legs 1, 3, 4 and 5 held as written. Leg
2 needed one more condition: **the planes must hold the axis**, because a plane
square to the axis also cuts a ring in two circles, or rests on it along one,
and 243.4 is what happens without it. Leg 6 — one plane with one edge along it
is the ring less one circle, not a half — holds because such a face never
reaches the new function; the general rule refuses it as before. No new test
pins leg 6.

Tests, in `holds.rs`: `a_quarter_bends_wall_holds_the_quarter_between_its_caps_and_nothing_else`
asks eight points exactly on the ring — held in the quarter, on the boundary on
either circle, not held in the other three quarters or on the cap plane's other
circle — and also checks the hand-built bend passes `Model::validate`.
`a_three_quarter_bends_wall_is_refused_rather_than_read_as_a_quarter` asks three
points and wants the refusal each time.

### 243.3 Which side a plane keeps is read off the wire, not a corner

The three-quarter bend is built as the quarter bend with its far end turned half
a turn round the axis: corner `(0, −3, 0)`, far cap facing `(1, 0, 0)`, seam
picked by the half-space in front of `(−1, 1, 0)·p = −3`, and every coedge
walked the way the quarter bend walks it. A turn about the axis keeps which way
a coedge runs against its surfaces, so this is a real bend and not a guess.

Read off a corner, both sides come out turned round. The near cap's plane is
stored facing `(0, −1, 0)`; the far corner `(0, −3, 0)` reads `Pos`, while the
wall leaves that cap at `y > 0`, which is `Neg`. The far cap's plane faces
`(1, 0, 0)`; the near corner `(3, 0, 0)` reads `Pos`, while the wall leaves
that cap at `x < 0`. With both turned round, both circles look in front of the
other plane, step 3 passes, and the answer is the one quarter the wall does not
cover. Put in place of the wire read, a corner read failed the three-quarter
test with

```
Exact([3.0, -4.0, 0.0]): Ok(Held)
```

— a wrong answer, not a refusal. The quarter test passed under the same
mutation, because for a quarter bend the two reads agree.

The wire read is `a_closed_crossing_is_walked_the_way_it_is_stored`, the rule
`Model::validate` checks every closed edge against, run backwards: asked what
the coedge's direction would be if the face kept the plane's `Neg` side, it
keeps `Neg` when the coedge runs that way and `Pos` when it does not. The
gradients carry each face's use orientation, as `validate` gives them, and the
answer is turned round when the plane's face is used reversed, because that is
what `validate` does to the side it hands in.

**`Model::validate` refuses that three-quarter bend** with
`AClosedEdgeIsWalkedAgainstItsSurface`. It hands the rule the side the wall's
corners read (`which_side_of_it_this_face_lies_on`), and both of the wall's
circles have the far corner on the other side of their plane. So the sweep
refuses a valid three-quarter bend, and a wire walked the wrong way round would
pass it. The three-quarter test asserts the refusal, so a sweep that learns
the bend breaks that test and points here. `validate.rs` was outside this
lane's write set.

### 243.4 A puck's rounded rim was answered wrongly and is refused now

A disk of radius four on `z = 1`, one on `z = −1`, and between them the outer
half of the ring of radius four and tube one. Each disk's plane rests on the
ring along one circle, and the rim has that circle as an edge. Before this lane
that went through the general rule: each plane crosses the ring in one curve,
each curve is an edge, and each disk's side, read off the other disk's corner,
leaves the rim as "the ring between the two planes" — the inner half too.
`(3, 0, 0)`, down in the hole and inside the puck, came back held, and a ray
through it would have counted a crossing that is not there.

It has the new function's shape — two planes, one edge along each — so it goes
there now and is refused, because neither plane holds the axis.
`a_pucks_rounded_rim_is_refused_rather_than_holding_the_inside_of_its_ring`
pins the refusal. The puck is built by hand and not put through the sweep;
nothing the test asks reads its wire. The rim is the part of the ring outside
the pipe of the ring's radius between the two planes, so answering it is a gap,
left open: nothing that has been dispatched builds a puck.

### 243.5 Where both bends stop now

**A bend and a bit-for-bit copy of itself** (`how_often_a_fuse_with_a_torus_refuses.rs`,
renamed `a_bend_fused_with_a_copy_of_itself_is_nothing_less_it_and_refused_otherwise`):

- The difference is answered, and it holds no solids — a solid less itself.
- The union and the intersection are refused with
  `TheFuseWroteABrokenAnswer { broken: CoedgesRunTheSameWay }`.

A temporary `eprintln!` in `select::choose` showed all six pieces placed `On`
the other bend, facing the same way. The union and the intersection kept the
first bend's two caps and its wall, not turned round, and dropped the copy's
three. Those are the right pieces. So the refusal is in writing them back out,
after selection. It was not traced further.

**The kernel's bend and the one built face by face, each less a copy turned an
eighth of a turn** (`building_a_solid_out_of_faces.rs`): both refuse alike, word
for word, still "a point in a piece that every solid could place". The piece is
the first bend's wall between `y = 0` and the turned bend's cap, with corners
`(3, 0, 0)` and a box round `(5/√2, 5/√2, 0)`. The caps are placed: the one on
`y = 0` outside the turned bend, the one on `x = 0` inside it. Selection tried
eight candidate points in that curved piece, each built from its corners — a
blend of them, or their middle stepped along an axis — so each is a box, and
each came back `CannotFireARayAtAFacesSurface` on the turned bend's torus face. `classify.rs` fires a ray at a torus only from an exact
point, and says so where it refuses. The pin fires from the middle of the two
corners and asserts that refusal, and that `(4, 0, 0)` is now placed outside.

§232.5 predicted that `select.rs`'s `a_face_of_the_input_holds` on the input
disk, or row Q55's `what_a_round_wire_encloses`, would be next. Neither pair
got there.

Both moved pins fail on the base source. With `crates/rcad-topo/src` checked
out at `ace6375f`, the copy pin failed with
`Union of a bend and a bit-for-bit copy of itself: Err(Undecidable { … "a point in a piece that every solid could place", inputs: [Exact([3.0, 0.0, 0.0])] … })`
and the turned one with
`the piece of the wall has two corners: Undecidable { … inputs: [Exact([3.0, 0.0, 0.0])] … }`.
With the new dispatch switched off, the quarter test failed with
`AFaceMissesPartOfWhereItsSurfacesCross { face: FaceKey { slot: FaceSlot(3v1), … } }`
and the puck test with `Exact([3.0, 0.0, 0.0]): Ok(Held)`. The three-quarter
test passes on the base, because the base refuses every bend wall. What it
guards against is the corner read, which 243.3 shows it catching.

### 243.6 What this leaves

- A bend unioned or intersected with a copy of itself keeps the right pieces and
  fails writing them back, `CoedgesRunTheSameWay` (243.5). Not traced.
- A curved piece's candidate points are built from its corners, so they are
  boxes, and a ray from a box cannot be fired at a torus (243.5). This is in
  `select.rs`'s `candidate_points` and `classify.rs`'s ray.
- `Model::validate` refuses a valid three-quarter bend and would pass one walked
  the wrong way (243.3). This is in `validate.rs`.
- A puck's rounded rim is refused, a gap with no consumer yet (243.4).
- §232.5's last item, `what_a_round_wire_encloses` asking the meridian circle's
  half-space, is untouched.
- A bend whose axis is off the coordinate axes is refused at the check that
  its planes hold the axis (243.7).

### 243.7 A bend whose axis is off the coordinate axes is refused

*Found by the row's verifier (`lane-q54-verify`, `6d28b7c0`) after the lane had
reported. The probe was a test the verifier deleted without committing, so the
numbers below are its own and were not run again here.*

The verifier built rings of radius 4 and tube 1 round four slanted axes —
`(0.6, 0.8, 0)`, `(1, 1, 0)`, `(0, 1, 1)` and `(1, 2, 3)` — and cut each with
planes through the origin whose normals were worked out square to the axis.

- `holds_the_axis_of` said `true` only where `dot(normal, axis)` came out
  exactly zero: `n = (0, 0, −1)` with the axis in the `xy` plane, and
  `n = (1, 0, 0)` with the axis `(0, 1, 1)`.
- On all five planes whose normal did not come out exactly square, the dot's
  enclosure held zero, its sign was `Undecidable`, and the function said
  `false`. The face is then refused by the `refuse()` after the call, not
  answered wrongly.
- `intersect(Torus, Plane)` named the two circles for four of those five. The
  fifth — axis `(1, 2, 3)`, `n = (w₂, 0, −w₀)`, dot in `[−1.1e-16, 0]` — it
  refused as `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`.

So on four planes the two functions this section calls "the same two
questions" disagree, and the `true` side pinned in `6d28b7c0` has only been
reached by shapes lined up with the axes.

Which reading is right decides gap or limit (§16.10), and it has not been
looked at. If the stored normal, rounded from one square to the axis, is not
square to it exactly, the plane cuts the ring in a quartic, `false` is the
true answer, and `intersect`'s two circles are the looser reading. If the dot
of the stored numbers is exactly zero and only the enclosure could not say so,
this is a gap: the exact sign of three products of `f64`s is always within
reach. Either way `holds_the_axis_of` returns a `bool`, so "no" and "cannot
tell" come out as the same refusal, and nothing records which it was.

*Looked at in §328: every plane `intersect` named two circles for is exactly
square to the stored axis, so those refusals were a gap and are answered now;
the one it refused leans off it, and "no" stays.*
