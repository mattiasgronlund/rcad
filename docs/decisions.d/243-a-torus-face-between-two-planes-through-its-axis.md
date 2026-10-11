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
  243.8   One plane square to the axis, along both its circles

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

*2026-10-03, Q753: neither is refused by step 3 now.* A plane stored twice is
one trimming with both edges along it, and on `0c8a44fc` that wall never
reached this rule: the general rule refused it with
`NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` (§377.2). Two stored planes
that are one plane did reach step 3. Both shapes come here now, and when the
planes hold the axis with normals along one line, the face is the half of the
ring on the side its wire keeps. It is refused, with
`AFaceMissesPartOfWhereItsSurfacesCross`, only when the wire keeps one side at
one circle and the other side at the other (§377.3).

The dispatched brief listed six legs. Legs 1, 3, 4 and 5 held as written. Leg
2 needed one more condition: **the planes must hold the axis**, because a plane
square to the axis also cuts a ring in two circles, or rests on it along one,
and 243.4 is what happens without it. *2026-10-04, Q826: one plane square to
the axis, with both of its circles as the face's only edges, is followed now
(243.8).* Leg 6 — one plane with one edge along it
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

*2026-10-04, Q826:* both planes are square to the axis, and since 243.8 that
is followed for one plane along both of its circles. The rim is still refused,
by the same name and earlier: each disk's plane rests on the tube, and at the
circle where it rests the closed-edge rule finds no side to read (243.8).

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

### 243.8 One plane square to the axis, along both its circles

*Row Q826 (`lane-q826-the-top-half-of-a-ring-holds-its-points`, off
`c11bf21f`).* The half-ring of §250.7 and §250.9, ring four and tube one
keeping `z ≥ 0`, read back through `Deserialize`. Its wall is one torus face
whose only edges are the two circles `z = 0` cuts the ring in, of radius five
and three.

**What refused, traced.** A temporary `eprintln!` in `holds.rs`, and a probe
test asking all eight orientation choices, both removed. Each leg the row named
held:

- `classify_point` at `(4, 0, 0)` fires a ray that meets the wall at a box
  round `(4.734, 0.746, 0.610)`, and the wall refuses it;
- the wall reaches `what_a_curved_face_holds` with one trimming and two edges
  along it, and `is_a_ring_face_between_two_planes` sends it to
  `what_a_ring_face_between_two_planes_holds` (§377.3);
- there step 1 refuses, because `how_the_axis_lies_on` gives `Neg` for the
  plane `(0, 0, −1)·p = 0`.

Asked directly at `(4, 0, 1)` and `(4, 0, −1)`, the wall refused
`AFaceMissesPartOfWhereItsSurfacesCross` in all eight choices, as Q664
measured.

**The rule.** 243.2's leg 2 asks for the axis because a plane square to it also
cuts a ring in two circles. With one such circle from each of two planes, the
planes' sides do not settle the face: between two planes square to the axis
the ring holds two bands, an inner and an outer, and 243.4's rim is one of
them. **One plane square to the axis, along both of its circles, is the other
case where one plane's side settles the face.** The plane `z = h`, with `|h|`
less than the tube, cuts the ring in two circles round the axis, at `R ± s`
with `s > 0`. The ring less those two circles is two pieces, the part above
the plane and the part below it, and each has both circles as its whole edge.
So a face whose only edges are those two circles is one of the two pieces, and
the side of the plane it keeps is the piece. The answer exists, so the refusal
was a gap (§16.10).

`what_a_ring_face_between_two_planes_holds` now takes a plane whose normal lies
exactly along the ring's axis (`lie_along_one_line`) as well as one that holds
the axis. It reads the side kept at each circle off the wire, by step 2's
closed-edge rule, as before. Then, when both planes are square to the axis:

1. both edges must be along the one stored plane — the two cuts' surfaces
   equal;
2. the two sides must be equal;
3. `point` is asked against that plane alone (`held_in_front_of_all`).

Anything else is refused `AFaceMissesPartOfWhereItsSurfacesCross`, the name
step 1 gave this shape before: one plane square to the axis and one not, two
planes square to it that are not one stored plane, or two sides that differ.

**When the two sides differ.** Both circles are stored running the same way
round the axis. Where a circle runs round the axis, the closed-edge rule's
longitude is square to it and the rule reads the latitude, `∇ × (a × ∇)` for
`a = ê × w`, the tangent of the centre circle. That is `a` times `|∇|²`, and
`a` is the same at the inner and the outer circle. Either half's wire walks its
two circles opposite ways round, so a wire that walks both the same way round
keeps one side at one circle and the other side at the other, and is no half.
Of the eight choices in the read-back test, four walk the flat face's circles
the same way round. In those four the wall is still refused, and so is
`classify_point` at `(4, 0, 0)`, naming the wall. The row asked for
`OnBoundary` in all eight. Answering those four would mean reading a side that
one of the wire's own circles contradicts.

**A plane resting on the tube** touches it along one circle. There the two
gradients are parallel, the closed-edge rule finds no side to read
(`Ok(None)`), and the face is refused at step 1 of the read. A temporary
`eprintln!` showed this for 243.4's rim, at `(4, 0, −1)`.
`a_pucks_rounded_rim_is_refused_rather_than_holding_the_inside_of_its_ring`
still passes.

**What is pinned**, in `tests/a_half_rings_flat_face_read_back_from_bytes.rs`.
`the_wall_holds_the_half_of_the_ring_its_wire_keeps`: in the four choices whose
flat face walks its circles opposite ways round, the wall holds three points on
the tube on the side its wire keeps, does not hold two on the other side, and
has four points of its two circles on its boundary; in the other four it
refuses. `classify_point_finds_the_hole_outside_and_the_flat_face_on_the_skin`:
`(4, 0, 0)` is `OnBoundary` in the first four and refused naming the wall in the
other four. Which side the wire keeps comes from the half-ring's own walk,
outer circle `Forward` on the flat face and wall used `Forward`, keeping
`z ≥ 0`. Turning the walk over, or the use round, turns that side over. With
`holds.rs` put back to `c11bf21f`, both tests fail, `classify_point` with
`CannotFollowAFacesTrimming` at choice `(Reversed, Forward, Forward)`.

> **2026-10-11 note (Q1730): `(4, 0, 0)` is `OnBoundary` in every choice
> asked, not just the first four.** Q1726 merged Q1722's fix to
> `count_crossings`, and `classify_point_finds_the_hole_outside_and_the_flat_face_on_the_skin`
> now pins `OnBoundary` on every choice it asks. The ray from `(4, 0, 0)`
> still meets the wall, which still refuses on the four choices whose wire
> keeps a different side of `z = 0` at each circle — that refusal is
> unchanged — but the flat face holds the point all the same, so
> `classify_point` answers `OnBoundary` regardless.

**`Model::validate` passes a half-ring whose hole is walked the wrong way.**
Asked of the eight read-back half-rings on `c11bf21f`, `validate` refused the
four that walk the outer circle `Reversed` on the flat face, with
`FacesDoNotAgreeWhichWayTheyFace`. It refused `(Forward, Forward, Reversed)`
and `(Forward, Reversed, Reversed)` with `VolumeIsNotPositive`, and passed
`(Forward, Reversed, Forward)` and `(Forward, Forward, Forward)`. The last one
walks the flat face's hole the same way round as its outline. 250.7 recorded
Q193's `validate` refusing the wall `AWireWillNotLieFlatOnItsFace` in all
eight. On this base it does not.

**What this leaves.**

- `Model::validate` passes the half-ring whose flat face walks its hole the
  same way round as its outline. Every corner of the flat face is on the ring,
  and every corner of the wall is on the plane, so no corner says which side
  either face keeps. Not traced. *2026-10-04, Q949: answered. `validate` now
  reads a ring face's side of a plane square to its axis off its wire at each
  circle, the way `holds.rs` does, and refuses two circles along one stored
  plane that keep different sides with `AClosedEdgeIsWalkedAgainstItsSurface`
  (`b4625fa8` traced it, `16ef0214` is the check in `validate.rs`, `346f8592`
  the pins, merged as `45593a1c`).*
- Two different planes square to the axis, one circle each, are refused. So is
  one such plane stored twice, two ways. Where the planes cut the tube, as on a
  washer's rounded rim between `z = ±h`, the band is told from its twin by the
  pipe of the ring's own radius, which `intersect` names each circle by. Where
  the planes rest on the tube, it is 243.4's rim, which Q104 holds.
  *2026-10-06, Q950: answered where the planes cut the tube.
  `what_a_ring_face_between_two_planes_holds` asks `intersect` whether two
  planes square to the axis are one plane. One plane takes the rule above as it
  stands; two take step 3, and then the side of the pipe of the ring's own
  radius must name both circles (`e43dbc55`, pinned by
  `a_washers_rounded_rim_holds_the_outer_band_between_its_planes_and_a_spools_waist_the_inner`
  and `a_half_rings_wall_holds_its_half_with_one_circles_cap_stored_turned_round`).
  `Model::validate` passes the outer and the inner band between `z = ±3` at
  their one sound walk. The part of a ring in front of either plane, not both,
  is still refused, and is Q1144.* *2026-10-06, Q1144: answered where each
  circle is behind the other plane. For planes through the axis the face is the
  part of the ring in front of either plane; for two planes square to it, the
  ring less the band (`c4183211`, pinned by
  `a_three_quarter_bends_wall_holds_the_three_quarters_between_its_caps_and_nothing_else`
  and `a_grooved_rings_wall_holds_the_ring_less_the_band_the_groove_took`). One
  circle in front of the other plane and one behind is still refused, Q1155.
  `Model::validate` refuses the sound grooved ring, Q1153, and passes a ring
  wall walked to keep the near side of both planes, Q1154.*
  *2026-10-06, Q1153: answered. `Model::validate` now holds a ring face's two
  circles round the axis to opposite ways round the tube, rather than reading
  a corner off the other circle, and so it passes the sound grooved ring
  (`a_grooved_rings_wall_holds_the_ring_less_the_band_the_groove_took`) and
  refuses the ring wall walked to keep the near side of both planes, Q1154's
  wall
  (`a_bands_wall_walking_both_circles_the_same_way_round_the_tube_is_refused`).*
  *2026-10-09, recording Q1335 (`54109536`, merged as `0383d43c` on
  2026-10-08): one circle in front of the other plane and one behind, Q1155's
  shape, is answered for two planes square to the axis with one outer circle
  and one inner, as where the cone `r = 9 − z` on the ring's axis cuts it at
  `z = −4` and `z = 3`. Round the tube's cross-section the face runs from the
  outer circle to the inner one, over the top or under the bottom: the part on
  each circle's side of the pipe of radius `R` in front of that circle's own
  plane. A point on that pipe is the top or bottom of the tube, on one side of
  both planes, so both parts give it the same answer. Pinned by
  `a_ring_face_from_an_outer_circle_of_one_plane_to_an_inner_of_another_runs_over_the_top_or_under`
  in `holds.rs` and
  `the_ring_face_of_each_piece_cut_by_the_cone_holds_its_side_of_the_cone` in
  `a_ring_face_bounded_by_a_circle_on_a_coaxial_surface.rs`. The test that
  pinned the refusal, `holds.rs`'s
  `a_ring_face_along_an_outer_circle_of_one_plane_and_an_inner_of_another_is_refused`
  (`e43dbc55`), asserted every walk refused; it is narrowed to the walks no
  piece keeps, with a wall along both outer circles added, as
  `a_ring_face_along_two_circles_keeping_sides_no_piece_keeps_is_refused`.*
- A wall whose two edges are one circle walked twice the same way round would
  pass the side check here as it would for a plane through the axis. That is
  Q790's question, not run for a plane square to the axis either.
  *2026-10-06, Q790: answered. The same wall on `z = 0`, walked the same way
  round twice, was read as a half too, and the check §377.3 puts back refuses
  it (`ca9c8434`, pinned by
  `a_wall_round_one_circle_round_the_axis_twice_the_same_way_is_refused`). This
  closes Q951. The lane's probe, since removed, found `Model::validate`
  refusing both such walls.*
- §377.3's paragraph "One refusal is renamed" says this shape is refused at
  step 1. It is answered now. §377 was outside this lane's write set.
- `Model::validate` read a ring face's circle round the axis off the wrong
  pair when the face across that circle was a pipe, a cone or a ball on the
  same axis rather than a plane: `which_two_surfaces_a_run_is_between` puts
  whichever curved face sorts first (a pipe, a cone and a ball each sort
  before a ring) beside the plane, so a circle between a ring and a curved
  partner never matched `[Torus, Plane]`, Q1153's band check was skipped, and
  the side kept was read off the other circle's corner instead. It refused a
  ring with its rim turned down to a pipe, and four other pieces of a ring
  cut by a pipe, a cone or a ball on its axis. *2026-10-07, Q1173: answered.
  `a_circle_round_the_axis` now also takes the face's own surface and returns
  `Option<(Torus, Plane)>`, rather than reading the pair alone (`463e76b1`, a
  one-ulp pin fix on top in `4cdda947`). Pinned in
  `crates/rcad-topo/tests/a_ring_face_bounded_by_a_circle_on_a_coaxial_surface.rs`:
  `intersect_names_no_crossing_of_a_ring_with_a_pipe_cone_or_ball_on_its_axis`,
  `a_circle_named_only_by_the_ring_and_its_partner_is_refused_for_the_pair`,
  `each_piece_walked_its_own_way_validates_and_holds_what_pappus_gives` and
  `every_other_description_of_each_piece_is_refused`.*
