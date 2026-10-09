## 423. A flat ring another plane walks back — 2026-10-09

*Queue row Q1526. The rule itself was built and merged under queue row Q1506,
by lane `lane-q1506-the-turn-13-pin-repinned`, in `c7760e33` ("a flat ring
another plane walks back is left out, so a cube tipped onto its edge along
another cube's floor fuses without a panic; turn 13 answers its union; Q1506
done"). This section writes the argument down and names the pins; nothing
here changes the code. `leave_out_each_flat_ring_another_plane_walks_back` and
`a_flat_ring_another_plane_walks_back`, both in `fuse/assemble.rs`
(`mise exec -- just src leave_out_each_flat_ring_another_plane_walks_back`,
`mise exec -- just src a_flat_ring_another_plane_walks_back`).*

### 423.1 The case that needed it

A cube tipped onto one of its own edges, standing with that edge along a
second cube's floor
(`a_cube_tipped_onto_one_edge_along_another_cubes_floor.rs`). The floor cuts
across the tipped cube's bottom face a rounding off the tipped cube's own
edge: at the tilt `t = 0.038` the meet of the floor, the bottom face and the
tipped cube's side `x = 5` rounds to `(4.999999999999999, 5, 0)`, a rounding
short of the cube's own corner at `(5, 5.000000000000001, 0)`
(`leave_out_each_flat_ring_another_plane_walks_back`'s doc comment). Joining
the union kept a sliver triangle of the bottom face between those two points
and `(0, 5, 0)` — every one of its three sides has the floor on the other
side of it — and the floor, joined up, walks those same three corners back
the other way, as a notch in its own outline. Before the rule, two of the
notch's corners had only those two edges, so the walk that writes each face's
boundary went straight through both and folded the sliver's three edges into
one: an edge from the third corner back to itself, with no length, which
panicked `edges_after_joining` ("a piece of a face has a straight side with no
length").

The same shape turns up in Q1208's random turns of a cube with its corner on
a block's face: turns 8, 21 and 49 of
`random_turns_whose_planes_meet_outside_the_block_answer_or_are_refused_by_name_tolerant`
(`a_turned_cube_with_its_corner_on_a_face.rs`) were refused `NoSpaceInside`
the same way — the tip's triangle a sliver kept with nothing to close it over
— until this rule left it out (queue row Q1506's commit `2b91988e`: "turns 8,
21 and 49 ... now answer the union ... where the pin has them refused
NoSpaceInside"). Turn 13 of the same test is the other case this fixed: its
union was refused `HoleWindsWithTheOuterWire` before, and now answers,
`1383.97`; its intersection is still refused `OuterWireWindsAgainstTheNormal`,
so with no intersection to hold it to, the pin instead holds the union to
`1000` more than the turned cube less the block, which answers `383.97`
(`mise exec -- just decision 417.3`, which records both figures).

### 423.2 The rule

`leave_out_each_flat_ring_another_plane_walks_back` looks, after every joined
face's rings are built, for a flat face with one ring of straight sides that
some other flat face — on a plane the two are not the same as
(`agreement_of_two_planes` says `Different`) — walks back as one unbroken
stretch of one of its own rings. When it finds one, both the face's own ring
and the matching stretch of the other face's ring are taken out: the ring's
edges are marked dissolved, the stretch is removed from the other face's
ring, and a face or a ring left empty by that is dropped too.

It is not taken when the stretch is a face's whole outer ring and that face
has holes: dropping the only ring that bounds the holes would leave them with
nothing round them (`a_flat_ring_another_plane_walks_back`'s
`would_leave_holes_alone` guard).

**Why it is right**, in plain words:

- **The shared sides sit on the line the two planes cross.** A straight side
  that two faces on different planes share has to lie on the one line where
  the planes meet, so in exact arithmetic a ring built only from such sides
  goes out along that line and back, closing on itself without ever stepping
  off it: it encloses nothing. Only rounding — moving each corner to the
  nearest representable point — can make such a ring look like it bounds some
  area.
- **The two walks cancel.** The left-out ring and the stretch taken out of
  the other face's ring step through the same corners in opposite
  directions, so as the answer's corners are written the two add no area and
  no volume between them: no measured volume changes.
- **Every edge left still has two faces.** Each edge of the left-out ring
  loses both of the faces it bordered — the ring's own face and the
  stretch's — so it is marked dissolved rather than left with one side open.
  Every edge the rule leaves alone keeps the two faces it already had.

### 423.3 The tests

- `a_cube_tipped_onto_one_edge_along_another_cubes_floor.rs`, all 7 tests
  (`mise exec -- just t -p rcad-topo --test
  a_cube_tipped_onto_one_edge_along_another_cubes_floor` → "7 tests run: 7
  passed, 0 skipped"): the union and the tipped cube less the block, which
  panicked before the rule, now answer and validate at both tilts, both
  orders and both policies.
- `a_turned_cube_with_its_corner_on_a_face.rs`, all 21 tests
  (`mise exec -- just t -p rcad-topo --test
  a_turned_cube_with_its_corner_on_a_face` → "21 tests run: 21 passed, 0
  skipped"):
  `random_turns_whose_planes_meet_outside_the_block_answer_or_are_refused_by_name_tolerant`
  carries turns 8, 21 and 49 (now answering, where they were refused
  `NoSpaceInside`) and turn 13 (its union now answering, where it was
  refused `HoleWindsWithTheOuterWire`).

### 423.4 How it differs from the other three, and what it leaves in

It is the fourth rule of its kind. §365.7's two rules and §417.4's third all
shorten or drop pieces *within one face's own ring* — a side whose two ends
round to one corner (`sides_of`), the wire that then walks out and back along
one edge once such a side is gone (`encloses_something`), and an edge only
such left-out wires ever walked, with the corners it leaves with nothing
attached (`leave_out_what_no_wire_walks`). This rule is the first to reach
*across* two faces at once: it only fires where a pair of flat faces on
different planes retrace the same corners, and it removes a whole ring from
one face together with the matching stretch of the other's ring, rather than
shortening either ring on its own.

**The face it drops can be part of the exact answer.** For the tipped cube,
the cube's own edge — where its two stored planes meet — lies about `6e-18`
below the floor, so in exact geometry a wedge of the cube that thin hangs
below the floor there. No representable point lies inside a wedge that thin,
so nothing written could ever mark it, and its corners round onto the very
three corners the dropped ring walks
(`leave_out_each_flat_ring_another_plane_walks_back`'s doc comment). §417.4
already took such roundings — an answer that may say less than the exact
geometry by a sliver no double can hold — as right rather than as a defect to
fix; this rule runs under every policy for the same reason the first three
do, since the dropped ring could never have held a nonzero area or volume to
begin with.
