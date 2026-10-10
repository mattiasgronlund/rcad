## 428. A flat face past the pipe between two circles of a ring, and a ring face read off its other circle — 2026-10-09

*Queue row Q1513, under §191.5. The naming is
`which_of_the_pairs_closed_crossings_it_is` in `fuse/assemble.rs`; the two
rules are `whether_past_the_pipe_between_two_circles_of_a_ring` and
`side_off_the_other_circle_a_plane_cuts_a_ring_in` in `fuse/arrange.rs`. Each
doc comment carries its own argument, and `mise exec -- just src <name>`
prints it.*

### 428.1 The case

The top half of a ring of 4 round a tube of 1 about `z`, with its seam or
without, beside a round bar of radius 23/8 that stands clear of it, was refused
`ResultIsNotManifold { count: 4 }` for the union both ways round and for the
half ring less the bar. Every fuse that kept the half ring's own faces was
refused, so a half ring could not be fused with anything at all.
`tests/a_half_ring_beside_a_bar_clear_of_it.rs` pins all six fuses, with the
seam and without. Three causes stood in the way, each found once the one before
it was gone.

### 428.2 Two circles under one name

`z = 0` cuts the ring in its outer and its inner equator. A closed crossing was
named by its two surfaces, and by the flat half-space `intersect` picks it out
by where there is one. A plane square to a ring's axis picks its two circles by
a pipe instead: the pipe of the ring's own radius about the ring's own axis,
`OutsideAPipe` for the outer circle and `InsideAPipe` for the inner. So both
equators had one name, became one edge with four faces along it, and the fuse
was refused as not a manifold.

They are named by whatever one picker picks the circle out now
(`which_curve_of_the_pair`). The slot ahead of the numbers says which kind of
picker it is: 0 for a flat half-space, 1 outside a ball, 2 outside a pipe and 3
inside one. A flat picker keeps 0 there, so every name that existed before
sorts as it did. The pipe is built from the ring alone, so the plane facing up
or down never reaches it, and the ring's axis written either way round is
folded; `each_circle_a_plane_square_to_a_rings_axis_cuts_is_one_name_however_the_two_are_spelled`
in `assemble.rs` asks every spelling, and fails with the base's naming.

### 428.3 The flat face reads the ring's sign past the pipe

With the two edges told apart the fuse was refused
`ACurvedFaceGoesRightRoundWithNoSeam`: the arrangement handed back each of the
half ring's two faces as two pieces of one wire each. On the flat face both
wires are crossings with the one ring. Every node of a walk round one circle is
on that ring, so `is_on_the_partner_of_a_crossing_through` passes over it and
the walk learns nothing about the other circle. No two walks agreed about any
curve, and nothing was joined.

`whether_past_the_picker` now answers a flat face against a ring whose crossing
is picked by that pipe, through
`whether_past_the_pipe_between_two_circles_of_a_ring`. On the plane, the pipe
is a circle between the two. Past the outer circle's picker is the disc inside
the pipe, wholly inside the outer circle; past the inner circle's picker is the
plane outside the pipe, wholly outside the inner circle. Either way it is the
band between the two circles, where the ring is negative. In front of either
picker the ring cuts the plane in that one circle and nothing else, so the
ring's own sign at the point is its side. A point whose box straddles the pipe
is answered only where the two readings agree.

The same rule tells apart the two pieces of a face that keeps both the outside
of the outer circle and the inside of the inner one. Both are outside the ring,
so the ring's sign alone would join them (428.6).

### 428.4 The ring's face reads one circle's side off the other, with one plane alone

On the ring's own face both wires are crossings with `z = 0`, and the walks
were stuck the same way. `side_off_the_other_circle_a_plane_cuts_a_ring_in` is
asked after `side_off_a_curve_it_also_cuts` finds nothing, and reads the walk's
side of the plane off a half-edge along the other circle: the region left of a
forward half-edge along either circle is the plane's negative side. That is
sound because a plane cuts a ring in two circles only square to its axis or
through it, and either way the two circles together cut the ring into two
pieces, one each side of the plane. With nothing else drawn those two pieces
are the only regions, so the plane's sign names the one a walk bounds.

**It holds only where that one plane is all that is drawn on the face.** Two
planes square to the axis cut a ring into four bands, and the band between them
outside the ring's radius and the band between them inside it have the same
sign against both planes. `f075b3d7` first read the side off the other circle
wherever a ring's face had both. With a probe in `group_the_cycles_into_cells`,
since removed, `f34da049`'s lane measured
`a_bar_in_a_rings_hole_touching_nothing_cannot_tell_the_cells_of_a_face_apart`
in `a_plate_cut_by_a_whole_bend.rs`, where a bar's caps at `z = 1/4` and `z =
3/4` cut the ring: every walk round either middle band read `[Neg, Neg, Neg,
Neg]`, walks 0, 2, 4 and 6 all faced each other and were joined into one cell,
and the union came back `ACurvedFaceGoesRightRoundWithNoSeam` in place of the
pinned `CannotTellTheCellsOfAFaceApart`. So the rule returns `None` unless every
arc drawn on the face runs along one of that plane's two circles.

**The general case stays refused.** With two or more planes square to the axis
on one ring's face, no reading by sign tells the outer and the inner band
apart; that needs the bands put in order round the tube, which nothing here
does. The fuse is refused `CannotTellTheCellsOfAFaceApart`, which is row
Q1500's refusal, and the measurement above is what stands behind it. A round
plate of radius 6 and one thick, `z` in `[−1/2, 1/2]`, holding the whole ring's
middle, has two such planes on each of the ring's faces and is refused that
way by every fuse, both ways round:
`a_plate_holding_the_middle_of_the_ring_is_refused_by_every_fuse` in
`tests/a_block_with_a_whole_ring_through_its_middle.rs`. That this is the cause
there has not been traced. No row is filed for it: Q1500 is the row.

### 428.5 The join of a plane and a ring takes flat-picked circles only

`the_name_each_closed_curve_goes_by` joins the curve a plane cuts a ring in with
a ball's, a pipe's or a cone's that is the same circle (Q1470). It gathered
every circle whose name says which, `curve.1.is_some()`. On `30a1b8da` a
pipe-picked circle had no such name and was never gathered; after 428.2 it has
one, so the join would have widened with nothing to show it right. `9764e426`
narrows it to names with 0 in the first slot, a flat picker, which gathers
exactly what the base gathered.

It was narrowed rather than tested because no test can reach it. The one shape
known that would is the top half of the ring standing on a round bar of its own
outer radius, `add_round_bar(5, 2)` moved by `(0, 0, −2)`: the bar's top rim is
the half ring's outer equator, and the bar's wall, `z = 0` and the ring all
meet along that circle. Every fuse of the two, both ways round, with the seam
and without, is refused `Geometry(WhereThreeSurfacesMeetNeedsTwoOfThemFlat)`,
which `rcad-geom`'s `three_surfaces.rs` raises. That is pinned as it stands,
`on_a_bar_of_its_outer_radius_every_fuse_is_refused_where_three_surfaces_meet`
in `tests/a_half_ring_beside_a_bar_clear_of_it.rs`, and it passes with
`30a1b8da`'s `fuse/` as well. The answers are
plain: the union is one solid of `50π + 4π²`, the intersection nothing, the half
ring less the bar `4π²`, and the bar less the half ring `50π`.

A bead or a rounded rim standing on a shaft of its own outer radius is a shape
designers draw on purpose, so the refusal is class B, and it is filed as a row.
Q988 counts what `three_surfaces.rs` still refuses for a ring, a pipe and a
plane square to their axis; it is a measurement, not this fix, so the row
stands beside it. When the shape answers, whether a pipe-picked circle joins
here is that row's to decide, with the shape as its test.

### 428.6 The lead: the base refused, it did not answer wrong

The first run's handoff guessed that a flat face on `z = 0` that a whole ring
crosses, keeping both the outside of the outer equator and the inside of the
inner one, had been joined into one cell on `30a1b8da`: a wrong answer. Run, it
was a refusal. A round plate of radius 6 with `z` in `[−2, 0]` and the whole
ring cut along its equators was refused `CannotTellTheCellsOfAFaceApart` on the
plate's face by all six fuses on `30a1b8da`. On this branch all six answer, one
valid solid each, with the faces and volumes counted by hand: the union `72π +
4π²`, the intersection `4π²`, the plate less the ring `72π − 4π²`, and the ring
less the plate `4π²`. Three tests in
`tests/a_block_with_a_whole_ring_through_its_middle.rs` pin them, and they fail
with `30a1b8da`'s `fuse/`.

A block, as the lead named it, never reaches the face. Its side planes are
parallel to the ring's axis and off it, and `where_a_plane_and_a_torus_cross`
refuses every such plane, even 20 from the axis against a reach of 5:
`ACrossingBetweenAPlaneAndATorusThatIsAQuartic`, row Q618. So the pins use a
round plate on the ring's own axis.

### 428.7 What this leaves

- **The half ring on a bar of its own outer radius** (428.5), refused
  `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` by every fuse. Filed as a row.
- **Two or more planes square to the axis on one ring's face** (428.4). Q1500
  holds it; not filed again.
- **A block with a ring through it** (428.6). Q618 holds it; not filed again.
