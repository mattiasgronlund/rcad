## 331. A second fuse of side-by-side bars stops at a pipe arc with no centre — 2026-09-28

*Queue row Q78, from §245.7's last item and §252.8. Lane
`lane-q78-a-second-fuse-of-side-by-side-bars`, off `6301f5eb`. Fusing any of
the six side-by-side answers once more refuses
`AFaceTrimmedByACurveThisCannotFollow` at the answer's **bottom cap**, whatever
it is fused with, a block that touches nothing included. The refusal is raised
in `fuse/arrange.rs`, `Arrangement::which_way_an_edge_runs_round`, because
`where_a_closed_crossing_is_centred` in `holds.rs` has no arm for a pipe. That
is the same missing arm §325.2 found on half bars, and Q330 is already the row
that adds it. **A gap** (§16.10). No `src/` changed.
`crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs` pins the refusal and
reaches one of the refused answers by another route.*

### 331.1 Where it stops

A temporary test fused each of the six first answers again, by every rule and
both ways round, with two blocks 30 by 20 and 10 tall: one from `z = 5` up,
over the answer's top half, and one from `z = 50` up, touching nothing. It was
`mise exec -- just t -p rcad-topo --test merged -E 'test(~fusing_two_bars_side_by_side::zz_temp_probe_second_fuse)' --no-capture`,
and it is deleted again. All 72 fuses refused the same way, for example:

```
PROBE   Union with far block answer first: Err(AFaceTrimmedByACurveThisCannotFollow { face: FaceKey { slot: FaceSlot(7v1), model: ModelId(1) } })
```

`FaceSlot(7v1)` is the answer's face on the plane `z = 0`, normal `−z`, in
all six answers. The same probe printed it: one wire of two edges, each from
`(3, −4, 0)` to `(3, 4, 0)` or back, one with the pipe round `(0, 0, ·)` on
its other side and one with the pipe round `(6, 0, ·)`.

A temporary `eprintln!` in the `ok_or` at `fuse/arrange.rs:2001` printed
once for each of the 72 fuses, 72 lines in all, each naming `FaceSlot(7v1)`,
the plane `z = 0` as `surface`, and one of the two pipes as `partner`:

```
TEMPTRACE which_way_an_edge_runs_round refuses face [FaceKey { slot: FaceSlot(7v1), model: ModelId(1) }] surface Plane(Plane { normal: Vector([[0.0, 0.0, -1.0]]), offset: 0.0 }) partner Cylinder(Cylinder { axis: Axis { origin: Point([6.0, 0.0, 0.0]), direction: Vector([[0.0, 0.0, 1.0]]) }, radius: 5.0 }) ends [Exact([3.0, 4.0, -0.0]), Exact([3.0, -4.0, -0.0])]
```

So every second fuse stops in the arrangement of the answer's bottom cap. The
block touching nothing shows that a cut plays no part: the arrangement runs on
that face with nothing cutting it, and stops reading the face's own
boundary.

`classify_point` on the same answers gives a second refusal that is not the
fuse's. `(3, 0, 5)`, `(0, 0, 5)` and `(8, 0, 5)` all come back
`CannotFollowAFacesTrimming { face: FaceSlot(8v1) }`, the top cap, and
`(20, 0, 5)` comes back `Outside`. Which line of `holds.rs` refuses the top cap
was not traced. §325.2 names `is_a_family_this_follows`, which admits no pipe
as a flat face's cut, and that is a guess here, not a finding.

### 331.2 Why the first fuse never met it

`Arrangement::arcs_an_edge_covers` asks `which_way_an_edge_runs_round` only for
an edge whose two ends are different nodes of a closed crossing. An edge that
starts and ends at one corner covers every arc and returns before the question
(`fuse/arrange.rs:1930`). A bar's cap circle is one closed edge, so no first
fuse of two bars asks it. The first fuse cuts both cap circles at `(3, ±4, ·)`,
so every answer's caps are bounded by **arcs**, and the next fuse asks about
each arc.

To answer, `which_way_an_edge_runs_round` needs a direction from the chord into
the arc. It gets it from `the_way_into_an_arc` (`fuse/arrange.rs:3740`), which
needs `where_a_closed_crossing_is_centred` (`holds.rs:1356`) to place the
curve's centre. That function has an arm for a ball and one for a cone. A pipe
falls to `Surface::Plane(_) | Surface::Cylinder(_) | Surface::Torus(_) | _ => None`,
so `the_way_into_an_arc` answers `None`, and the `ok_or` turns that into the
refusal, naming the face.

The face refused is the first one whose arrangement meets such an edge, and in
all six answers that is the bottom cap. How far round an arc runs never comes
into it: the unions' caps are bounded by the two arcs of about 254° (§252.5)
and the lenses' by the two short ones, and both refuse at the same `ok_or`.
That is why §252.8 found the lens refusing too.

### 331.3 What a trial pipe centre gets

The arm §325.2 describes was put in `where_a_closed_crossing_is_centred` for a
trial and taken out again: the axis origin as `from`, and the axis direction,
turned to read positive against the plane's normal, as `along`. A plane
parallel to the axis gets `None`, since that plane cuts the pipe in lines. With
it the same probe gave, for each answer:

- **The block touching nothing**: both unions and both lenses answer every
  rule both ways round. The union gives two solids, the intersection none, and
  the difference gives the answer (with the answer first) or the block (with
  the block first). Every solid validates. The answer that comes back has 4
  faces, the corners `(3, ±4, 0)` and `(3, ±4, 10)`, and the same volume as
  the first fuse: `1346.5972099829235 ± 2.207736708953189` for the union
  with `here` as the argument, and `223.41772414475454 ± 0.9202470039325203`
  for the lens with `here` as the argument.
- **The block touching nothing, with a crescent** (the two differences): the
  union, and the difference with the crescent first, refuse
  `ACurvedFaceGoesRightRoundWithNoSeam`, on `FaceSlot(5v1)` or `FaceSlot(11v1)`
  of a model id that is not the test's own. The intersection and the
  difference with the block first answer. Not traced.
- **The block over the top half**: every fuse refuses `Undecidable`. The
  unions and lenses both ways round, and the crescents with the block first,
  name the predicate `"a point in a piece that every solid could place"`
  (`fuse/select.rs`, `PLACED_AGAINST_EVERY_SOLID`), with inputs `(3, 4, 5)`
  and `(3, −4, 5)`. The crescents with the crescent first name
  `"a point inside a piece of a face"` instead. Not traced. The top-cap
  refusal of 331.1 may be the cause, and nothing here shows it.
- `classify_point` refuses the top cap as before.

With the trial arm, `mise exec -- just t -p rcad-topo --no-fail-fast` ran
1084 tests, and 1083 passed. The one that failed was
`a_union_or_a_lens_fused_with_a_block_it_does_not_touch_comes_back_as_it_was`,
the new pin, failing on its first fuse, the union with `here` as the
argument, then the union with the top block. The error was the
`Undecidable` above. No other pin moved.

**Does a corner with two curved edges survive a second fuse?** Yes, where the
second fuse gets that far. Each of `(3, ±4, 0)` and `(3, ±4, 10)` is where two
arcs, one of each pipe's circle, meet the straight line the two pipes cross
in. With the trial arm, both unions and both lenses come back from the fuse
with the block touching nothing with the same four corners, four faces, and
their first volume. With the top block, the fuse refuses before any answer is
built, so this lane saw nothing about the corners there.

### 331.4 Why it is a gap and not a limit

The refusal says the kernel cannot tell which of a circle's two arcs an edge
is. The edge is one arc or the other. Every refused fuse has an answer, and
the kernel builds it by another route:

- With the block touching nothing, the answer is the first answer and the
  block, untouched, or one of the two, or nothing. The trial arm of 331.3
  returned exactly that for the unions and lenses.
- With the block `T` over the top half, the three rules give
  `(A ∪ B) − T = (A − T) ∪ (B − T)`, `(A ∩ B) − T = (A − T) ∩ (B − T)` and
  `(A − B) − T = (A − T) − (B − T)`. Each half bar's caps are whole circles
  again, so no arc is ever an input edge.
  `cutting_the_top_half_off_each_bar_first_answers_what_the_second_fuse_should`
  fuses them that way: all three rules, both ways round, give one solid that
  validates, with corners `(3, ±4, 0)` and `(3, ±4, 5)`, within the mesh's bound
  of half the first answer's volume by hand. That is the answer the refused
  `Difference` with the block second should give.

The families of fixes considered:

1. **Place a pipe's centre in `where_a_closed_crossing_is_centred`.** A plane
   not parallel to a pipe's axis cuts it in a circle or an ellipse. Turning
   the pipe half a turn about the point where the axis meets the plane maps
   the pipe to itself and the plane to itself, so the curve is symmetric
   about that point, and that point is its centre. The centre always exists
   for a crossing that closes, and the trial arm is that family. It keeps the
   one spelling both callers share (§23.8, §211.5), so the arrangement and
   `holds.rs` cannot disagree about a pipe.
2. **Tell the two arcs apart by a point in the middle of the edge.** Ruled
   out by the record. An edge stores no curve (§22), and
   `which_way_an_edge_runs_round`'s doc says why a tangent is read instead.
3. **Keep a bar's cap circle as one closed edge in the answer.** Not
   possible. The answer's cap really is bounded by two arcs of two circles.

The arithmetic plays no part. Every corner here is exact (§245.5), the refusal
is a `None` and not an `Undecidable`, and the trial arm turned it into answers
without any other change.

### 331.5 What this leaves

- **The pipe arm, and the pin it breaks.** Q330 already adds the arm, for the
  half bars of §325. `a_union_or_a_lens_fused_with_a_block_it_does_not_touch_comes_back_as_it_was`
  fails as soon as it lands, so Q330's write set needs
  `crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs`, and the pin gets
  rewritten there to what the second fuses do next: the unions and lenses
  against a block that touches nothing answer, with the checks 331.3 lists.
  `holds.rs` is Q105's file, so Q330 waits on Q105.

  **Closed by Q350 (`8f9750e2`).** The arm lands, and the pin is rewritten as
  `a_union_or_a_lens_fused_with_a_block_it_does_not_touch_comes_back_as_it_was`,
  the name §394 (Q586) now carries this test under.
- **The top half, after the arm.** `"a point in a piece that every solid could
  place"` in `fuse/select.rs` for all six answers, and `"a point inside a piece
  of a face"` for the crescents with the crescent first. `select.rs` is Q102's
  file. Whether it is the top cap `classify_point` refuses is the first thing
  to find out.

  *2026-09-29, Q596, off `8d42d3ab`.* It was, and it was `holds.rs`, not
  `select.rs`. The record's inputs `(3, ±4, 5)` are the corners of the piece
  (`corners_of(piece)`), the block's floor inside the two pipes, and not the
  point that was tried, so the third leg of the brief (a point on a trimming
  curve) was not it. A temporary trace in
  `what_a_face_cut_by_part_of_a_crossing_holds` showed the top cap `8v1`
  refusing at two places. For the unions and lenses, `where_a_side_lies` read
  an arc against a pipe as `Lies::Across`, because its arc arm answered only a
  flat cut. For the crescents, `is_a_family_this_follows` refused a face
  keeping a pipe's outside, with `kept` of `[Neg, Pos]`. Past the cap, each
  wall refused next (`9v1`): `leaves_a_pipe_in_one_piece` took only planes, and
  the wall is cut by the other pipe. Past that, each crescent's inner wall
  (`10v1`, used reversed) read every cut as keeping the far side. The printed
  `kept` was `Pos` for the floor, the roof and the other pipe.
  `which_side_of_its_neighbour_a_side_keeps` turned `∇S` round for a reversed
  face, and the view's walk already does that. `fuse/assemble.rs`'s `sides_of`
  stores a reversed curved piece's wire the used way round, so that every
  edge's two coedges run opposite ways in the arena, and `WireView::coedges`
  walks it back. Printed through the view, the forward outer wall and the
  reversed inner wall both walk the ruling at `(3, 4)` from `z = 10` down.

  What `holds.rs` now follows, each argued in its doc comment. Two pipes whose
  axes run the same way cut a plane in two curves that meet in at most two
  points, because one sign less the other, scaled, is a plane's sign. So an
  arc of one with both ends on the other lies on one side of it, the side it
  leaves its start into. The cap of the union is the two insides taken
  together (`is_two_discs_taken_together`), and a disc less a disc is one
  piece. On a wall, a pipe running the same way is a cut along the axis. The
  reversed-face turn is gone. With it, all 24 second fuses of the unions and
  lenses with the top block answer, one solid each, validated, at the volume by
  hand: `V/2 + 6000`, `V/2`, `V/2` and `6000 − V/2`.
  `a_block_over_the_top_half_of_a_union_or_a_lens_cuts_it_as_cutting_each_bar_first_does`
  checks them, and `every_first_answer_says_which_points_it_holds` checks four
  points on each of the six answers. Both fail with `holds.rs` at the base. The
  second also fails with only the reversed-face turn put back:
  `CannotFollowAFacesTrimming` on `10v1` at `(−3, 0, 5)` of the crescent.
  `mise exec -- just t -p rcad-topo --no-fail-fast` ran 1180 tests and all 1180
  passed, with 11 skipped.

  **Still refusing: the twelve fuses of the two crescents with the top
  block**, every rule both ways round, `"a point inside a piece of a face"`
  (`INSIDE_A_PIECE`). The piece has corners `(3, ±4, 15)` and `(3, ±4, 10)`,
  on the inner wall's pipe, and the inner wall stops at `z = 10`. So the
  arrangement cut a piece off the part of that pipe the face does not hold.
  `classify_point` now answers on the crescents, so this is the fuse's own
  reading of the reversed wall. What is known: `bake_one_face` reads the wire
  through `model.face_at`, which is `Orientation::Forward`, so it gets the
  stored walk, the used way round. It then records `orientation: Reversed`
  beside it. Whether `fuse/arrange.rs` turns that walk round once more, as
  `holds.rs` did, is **not traced**. It is a hypothesis on two legs, bake's
  forward read and arrange's reading of `orientation`, and both files are
  outside Q596's write set. It is a gap: every one of the twelve has an
  answer, and cutting each bar first builds the crescent less the block.

  **Closed by Q609 (331.6).** The first leg holds and the second does not.
- **The crescents against a block that touches nothing, after the arm.**
  `ACurvedFaceGoesRightRoundWithNoSeam`, not traced.

  **Closed by Q609 too (331.6).** The same cause, and the same fix.

### 331.6 A face used reversed reached the arrangement walked the wrong way round

*Queue row Q609, lane `lane-q609-crescents-with-a-block-over-their-top`, off
`2c349f2b`. One `src/` change, in `fuse/bake.rs`.*

**What `bake_one_face` gets.** A temporary unit test in `bake.rs` baked the
crescent `here − there` and printed each face's wire two ways. Through the
solid's view, the inner wall `10v1` (pipe round `(6, 0, ·)`, used reversed)
walks `(3, 4, 0) → (3, −4, 0) → (3, −4, 10) → (3, 4, 10) → (3, 4, 0)`, down the
ruling at `(3, 4)`, which is the walk with the wall on its left about the
pipe's own outward gradient. Baked, through `model.face_at`, it walked
`(3, 4, 0) → (3, 4, 10) → (3, −4, 10) → (3, −4, 0) → (3, 4, 0)`, up that
ruling: the stored walk, which `fuse/assemble.rs`'s `sides_of` writes the way
the solid uses the face. The three faces used forwards baked the same walk as
their view. So the first leg holds.

**Whether the arrangement turns it round again.** It does not.
`fuse/arrange.rs` reads `self.face.orientation` only to copy it onto each
piece (`the_whole_surface` and `piece_from`), and `split_a_face` on the baked
inner wall with nothing cutting it gave back one piece walked exactly as it was
baked. The second leg does not hold. The arrangement reads the face on the
left of its boundary about `∇S`, the face's own surface
(`count_the_windings_inside_each_lump`; `material_is_left_of_forward` takes
`o = ∇S`), and nothing turns the stored walk into that.

**Where the piece comes from.** A temporary trace in `split_a_face` showed what
the fuse cut the inner wall by, five surfaces: the planes `z = 0`, `z = 5`,
`z = 10` and `z = 15`, and the other pipe. Handed the stored walk and those
five, the same unit test got four pieces: `[0, 5]`, `[5, 10]`, a piece of two
sides at `z = 0`, and a piece of six sides from `(3, ±4, 10)` to
`(3, ±4, 15)` that walks each ruling there and back, which is the piece
`INSIDE_A_PIECE` named. Handed the walk turned round, it got exactly
`[0, 5]` and `[5, 10]`. With only the block's floor and roof as cuts, both
walks gave `[0, 5]` and `[5, 10]`, so the wrong walk does not always show.
Why the face's own neighbours' surfaces are among its cuts here was not
traced.

**The fix.** `bake_one_face` reads the stored walk as before, and for a face
used reversed turns each wire round, the order and each side's two ends, which
is what `WireView::coedges` does. `inside_across` is still read off the stored
walk, since the rule it reads is checked by `Model::validate` against the
coedge's stored turn and the faces' outward gradients. `fuse/assemble.rs`
already expected this: it turns a kept piece's wire round where the piece's
use and the rule's turn differ, so a reversed input piece kept as it is comes
out stored the used way round again, as the arena wants.

**What it changes.** All twelve fuses of the crescents with the block over the
top half answer, one solid each, validated, at the volumes cutting each bar
first gives: `a_block_over_the_top_half_of_a_crescent_cuts_it_as_cutting_each_bar_first_does`.
And Q352's shape answers too. With the fix switched off, six of the twelve
fuses of the crescents with the block that touches nothing, the unions and
the differences with the crescent first, refused
`ACurvedFaceGoesRightRoundWithNoSeam`; with it, all twelve answer and
validate, and `a_crescent_fused_with_a_block_it_does_not_touch_comes_back_as_it_was`
checks them. `a_face_used_reversed_is_baked_walked_the_way_its_view_walks_it`
in `bake.rs` pins the walk itself. All three fail with the fix switched off,
and `mise exec -- just t -p rcad-topo --no-fail-fast` ran 1333 tests and all
1333 passed, with 9 skipped.

**What this leaves.** Nothing in this shape. A reversed face that a first fuse
writes and a second fuse cuts is not rare (a bore through a block, then a
second cut across it), and why no test met this before was not looked into;
a row pins such a bore. Q772's slab with a round scoop, its scoop a pipe face
used reversed and built face by face, still refuses its cut on this branch,
`could not settle 1 decisions the answer depends on`, so this is not its
cause.
