## 426. Two faces along an edge on parallel planes are one face — 2026-10-09

*Queue row Q1446. The rule is in `edges_with_one_face_on_both_sides` and
`facing_the_same_way`, and the pass that runs it a second time is in
`join_up_what_is_one_face`, all in `fuse/assemble.rs` (`mise exec -- just src
edges_with_one_face_on_both_sides`). It extends §17.4's amendment of
2026-08-30, which joins two faces of an answer on one plane facing one way.*

### 426.1 The case that needed it

A frame five by three and one high, with a block two by one and one high flush
beneath one end of its front wall, both turned together by 27/64 of a turn
(`a_turned_frame_and_a_block_flush_beneath_it.rs`, queue row Q1426). Over the
turned planes the block's front and the frame's front have the same normal
bits, with offsets `2.7755575615628914e-17` and `0.0`. `Plane::agreement_with`
calls them `Different`, and they are: two parallel planes a rounding apart.

Their union came back with the strip of the block's top between the two fronts
left out, by Q1426's rule for a ring of two straight sides, and that left the
frame's front and the block's front as two faces along one edge, from the
turned `(3, 0, 0)` to the turned `(5, 0, 0)`. Nothing joined them, since the
joining asked only for one plane. At the turned `(5, 0, 0)` the only faces
round that corner were the two fronts and the wall at `x = 5`: two plane
directions, where a corner needs three. Taking the block back out of that union
was refused in `bake`'s `derive_a_corner`, at "where three surfaces meet", with
no bound at all. The lane before this one named the corner with a print in
`bake` and listed the three planes; its handoff has them.

### 426.2 The rule

`edges_with_one_face_on_both_sides` dissolves an edge whose two flat faces are
on one plane facing one way, as before, **and also one whose two planes have
parallel normals facing the same way, wherever each plane sits.** That is
decided exactly, with no tolerance: both planes are slid to the origin with
`Plane::moved_to(0.0)`, which keeps the normal's bits, and
`agreement_of_two_planes` is asked whether the two are then one plane facing
one way. Its six minors are then a question about the two normals alone.

The joined face goes on `plane_of_the_joined_face`, the smallest of its pieces'
planes read as four numbers, as every joined face already did. Here that is the
block's front, whose fourth coefficient is `-2.78e-17` against the frame's
`0.0`, so the frame's corners on that face sit `2.8e-17` off the plane they are
written on. `Model::validate` accepts the answer.

**The joining runs a second time when the first pass left a ring out.** The
edge between the two fronts does not exist until
`with_each_ring_of_two_straight_sides_left_out` has left the strip out and made
its two edges one, which is after the joining has read every edge. So that step
now says whether it left a ring out, `rebuild_from` passes the answer on, and
`join_up_what_is_one_face` runs again when it did. It runs again only after two
or more edges were made one, so there are fewer edges each time and it ends.

**Why it is right:**

- **Two parallel planes apart meet nowhere.** An edge lies on both of its
  faces' planes, so in exact geometry no edge has one of two such planes on
  each side. One that is written there is the rounding's: the two faces are
  pieces of one flat region whose two solids worked their planes out
  separately.
- **Kept apart, the edge's corners cannot be read back.** At an end of such an
  edge the two faces give one plane direction between them, and a corner with
  only one more face round it has two directions, so no three of its planes
  meet in a point. That is §17.4's amendment of 2026-08-30 again: an answer this
  kernel can write and the next fuse cannot read.
- **Joining moves no corner.** The corners of the piece whose plane the joined
  face does not keep sit off the kept plane by the gap between the two planes,
  `2.8e-17` here, and the next fuse writes them as meets with the kept plane.

### 426.3 Rejected

**Reading the edge's line off other planes at its corners, in `bake`.** The
lane before this one weighed it and did not take it. It patches `bake` alone,
and leaves an edge whose two faces' planes never meet for every later stage
that reads an edge's line off its two faces.

**Joining planes whose offsets are within a tolerance.** Not needed: the edge
between the two faces is the evidence that they are one region, and the
question left, whether the normals are parallel, has an exact answer.

### 426.4 The tests

`the_block_taken_back_out_of_the_union_is_the_frame`
(`a_turned_frame_and_a_block_flush_beneath_it.rs`) was refused before the rule.
It now gives one solid of `12`, and each face of the frame as built matches
exactly one face of the answer: the same normal bits, the same way round, and
an offset and every corner within `1e-12`. Not bit for bit, because a fuse
writes each corner as the meet of three of its planes where the frame has its
built corners turned: the corners round the frame's hole, which the block never
comes near, come back an ulp or two off, and the joined front sits on the
block's front plane.

### 426.5 Its place among the others

It is the fifth rule of its kind. §365.7's two rules and §417.4's third shorten
or drop pieces within one face's own ring, and §423's fourth takes a ring out of
one face with the stretch of another face that walks it back. This one is the
first to join two faces whose planes `agreement_of_two_planes` calls
`Different`. Like the other four it runs under every policy, and for an input
too, through `joined_faces_of_an_input`.
