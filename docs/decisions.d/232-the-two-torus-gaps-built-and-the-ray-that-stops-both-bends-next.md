## 232. The two torus gaps of §229.3 built, and the ray that stops both bends next — 2026-09-25

*§229.3 named two gaps in the torus pins and built neither. Lane
`lane-torus-gaps`, off `1aec994c`, built both. Each pin now refuses one step
further on, and both at the same place: a ray fired from inside a bend's end
cap cannot follow the other bend's torus face.*

  232.1   A flat piece on a meridian circle offers both places the ring crosses its plane
  232.2   A torus leaves two disks on a plane through its axis, and the piece now asks which
  232.3   A node on a crossing whose partner is the asked surface is on that surface
  232.4   What both bends do past it
  232.5   What this leaves
  232.6   The second item of 232.5 was already fixed when it was written

### 232.1 A flat piece on a meridian circle offers both places the ring crosses its plane

`select.rs`'s `the_middle_of_a_round_wire` had no torus arm, so a bend's end
cap — a disk with one corner, on its rim — had nothing inside it to try. It is
now `the_middles_of_a_round_wire` and returns a list: one point for a ball, a
pipe or a cone as before, and for a torus the places its ring crosses the
plane, none, one or two. A plane through the axis gets two, `(4, 0, 0)` and
`(−4, 0, 0)` for the cap on `y = 0`. Which of the two circles a wire goes round
is not known there and does not need to be, because no candidate is trusted.
`half_way_into_either_arc` tries each middle it is given.

With it, the cap holds `(4, 0, 0)`, certified, and a bend fused with its copy
no longer refuses "a point inside a piece of a face".

### 232.2 A torus leaves two disks on a plane through its axis, and the piece now asks which

The piece's own test counted a round wire as `±1` wherever the point was inside
the wire's partner. That is right for a ball, a pipe and a cone, which each
leave one region on a plane. A torus cut through its axis leaves two, so the
torus's sign held `(−4, 0, 0)` — in the other circle's disk — as inside the cap
too. Shown by switching the new check off: that point was then held and sent to
be placed against the copy.

`the_piece_holds` now also asks the flat half-space `intersect` names the
wire's circle by. `intersect` has already proved that half-space clears both
circles (`two_meridian_circles`), so each disk is exactly the part of the tube
on its own circle's side, and the wire's corner says which side is this one.
It is asked once per piece. A plane square to the axis cuts two circles one
inside the other, named by pipes, and neither circle's disk is the tube's
inside on one side of a pipe, so that family says nothing and a disk bounded
by it stays unplaced. That is a refusal where a wrong answer was possible.
`a_plane_square_to_a_bends_axis_names_no_meridian_circle` holds it; whether any
fuse reaches it is not measured.

Until three tests in `select.rs` were added, switching the check off left
every test passing. They are on a cap of the torus `(4, 1)` on `y = 0` with
its corner at `(3, 0, 0)`: `a_bends_end_cap_holds_the_disk_its_own_corner_is_in_and_not_the_other`,
`the_first_point_a_bends_end_cap_offers_and_holds_is_in_its_own_disk` and
`a_piece_on_a_plane_square_to_a_bends_axis_cannot_say_what_it_holds`. The first
two fail however the check is switched off, and the third when the whole check
is a no-op (`7f921572`).

`holds.rs`'s `what_a_round_wire_encloses` has the same rule — the partner's
sign alone — and so the same defect for a flat face bounded by a meridian
circle. It is outside this lane's files and is not fixed or measured here.

**2026-10-09 (Q1512).** The pin named above as the refusal,
`a_piece_on_a_plane_square_to_a_bends_axis_cannot_say_what_it_holds`, was
renamed by row Q1501 to
`a_band_on_a_plane_square_to_a_bends_axis_holds_its_own_points`
(`fuse/select.rs:3880`): the band is no longer unplaced.  `the_piece_holds`
now answers it there, asking `runs_round_the_outer_circle`
(`fuse/select.rs:1831`, also asked at `:1335`, `:1689` and `:1730`), the
disk's own circle rather than its partner's sign. `holds.rs` already answered
the matching case for a flat face — not the meridian circles of 232.1 but the
circles round the axis a plane square to it cuts — by
`the_picker_naming_the_circle_through` (`holds.rs:569`), as §250.9 says.

### 232.3 A node on a crossing whose partner is the asked surface is on that surface

`arrange.rs`'s `which_side_of_everything` asked a node on the far circle of the
plane `x + y = 0` which side of the cap circle's plane it was on — the same
plane, bit for bit. The answer is exactly zero, and a box `5.7e-13` wide cannot
certify zero. It now passes over a node that lists a crossing whose partner
equals the asked one, as it passes over any node on the surface.

This is not a tolerance, and it is exactly as exact as the node's list of
crossings. Every point of a crossing is on its partner, so a node listing a
crossing with partner `P` is taken to be on `P`; the skip reads the list and
never the node's box. Under `Policy::certified` a crossing is on a node's list
because the model says so (a corner's own faces and the edge it starts,
`place_the_corners`), because the node was made for it (a meet, a pole, where a
closed crossing starts), or because a sign was proved zero — with two
exceptions. `round_a_closed_crossing` answers `Zero` for two nodes whose boxes
are equal bit for bit (`if one == other`) without proving they are one point,
and `merge_points_that_are_one_point` folds on that `Zero`. `fold_the_nodes`
then gives the node two nodes fold into every crossing either had, so a wrong
fold lists a crossing only one of them was on. Under `Policy::tolerant`,
`passes_through` and `order_along` send a narrow undecided sign through
`must_decide`, and `resolve_under_policy` reads it as `Zero`: a corner can list
a crossing it is only near, and two nodes only near each other fold. The box
`where_the_crossing_starts` gives a closed crossing's start is a guess at its
width, but it decides no list: that node gets its own crossing by construction,
and `round_a_closed_crossing` tells it from the others by name, not by box.

None of that is a new way to be wrong. Skipping a node says no side; the loop
goes on to the walk's next node, and a walk with no node left to answer leaves
the side unsaid. `group_the_cycles_into_cells` ignores an unsaid side, so a
wrong skip can hide a disagreement — but only at a node whose list is wrong,
and a crossing on a node's list is already drawn through it, its arcs ending
there. A wrong list is a wrong drawing before it is a wrong skip, and the skip
only takes the call the arrangement made when it put the crossing on the list.
The old question was no check on the list either: it refused at a right one,
which is the case this section fixes.

§229.3 passed on a worry: grouping assumes a closed curve cuts the surface in
two, and one circle round the tube does not cut a torus in two. Here it does
not matter. The quarter bend's torus face runs from angle 0 to 90°, and both
circles of `x + y = 0` lie at 135° and 315°, outside it, so the whole face is
on one side of the plane and its sign stands in for the pair. Past the gap the
arrangement runs to the end and cuts every face. **Whether those cells are
right is not measured**, because selection refuses before there is a result
to check.

### 232.4 What both bends do past it

Both pins now refuse "a point in a piece that every solid could place", on the
first bend's end cap on `y = 0` (one corner, `(3, 0, 0)`):

- **A bend fused with its copy**, under all three rules.
- **A bend built face by face** and the kernel's bend, less a copy turned an
  eighth of a turn round the ring, alike word for word.

In both, the point found inside the cap is `(4, 0, 0)`. The ray fired from it
at the other bend answers `CannotFollowAFacesTrimming`, and the face it names
is that bend's torus face. Both tests pin that ray answer and the face's
surface kind. `settle_against_each` reads that answer as "cannot place" and
has no other candidate inside the cap. Which `holds.rs` error the classifier
turned into `CannotFollowAFacesTrimming` is not traced.

Each pin fails without its change. The copy's pin was "a point inside a piece
of a face" on `1aec994c`, and the first run with 232.1 in place moved it. With
232.3's `arrange.rs` change taken out and everything else kept, the
built-face-by-face pin fails with `left: "side of plane"`.

`just t -p rcad-topo`: 935 passed, 15 skipped. Of the 15 ignored, the one that
passes when run (`a_bar_inside_a_bar_touching_its_wall`) passes on the base
source too, so nothing moved. `ignored_tests` stays 15.

### 232.5 What this leaves

- The classifier cannot follow a bend's torus face (232.4). That is the next
  refusal for both bends, and it is in `classify.rs`/`holds.rs`.
- Once that is past, a flat piece coincident with an input disk still cannot be
  settled `ON`: `select.rs`'s `a_face_of_the_input_holds` winds a flat input
  face's corners, and a disk has one. `holds.rs`'s round-wire rule would answer
  instead, and it has 232.2's defect.
- `holds.rs`'s `what_a_round_wire_encloses` should ask the meridian circle's
  half-space as `the_piece_holds` now does (232.2).

### 232.6 The second item of 232.5 was already fixed when it was written

*Added 2026-09-26, from queue row Q56.*

`16a9d4c4` (§222.3, the cone and its copy) sends a flat input face through
`SolidView::face_holds_the_point` whenever any of its sides has a neighbour
that is not a plane. It no longer winds its corners. `a_cone_and_its_copy.rs`
pins it: a cone's cap is a disk with one corner, the same shape as a bend's.
This section was written off `1aec994c`, which does not have that commit.

Q56's lane, off `441f359e`, printed what `a_face_of_the_input_holds` answered
in every fuse of two bends where a flat piece lies on an input disk. It
answered `Held` every time: at `(0, 4, 0)` for two quarter bends on one torus,
in both orders, and at `(−4, 0, 0)` and `(4, 0, 0)` for a half bend with either
quarter bend. The verifier re-ran the ancestry check. Q56 is closed as done at
`16a9d4c4`, not at a commit of its own.

Those fuses still refuse, somewhere else:

- **Two quarter bends on one torus.** The wall piece's points on the inner
  equator are fired at the other bend's torus face. The ray answers
  `CannotFireARayAtAFacesSurface`, and `settle_against_each` reads that as
  "cannot place". A torus takes a ray only along an exact line (`classify.rs`),
  so this is probably Q102's cause: a curved piece's points are boxes. That is
  not traced. The verifier reproduced 6 of 6 fuses refusing, but not the lane's
  count of refusal lines, because `select.rs` swallows the answer into
  `Ok(None)` without logging it. *2026-10-02, Q754: no longer reached.
  Selection places every piece in all six fuses; four answer, and the two
  unions refuse later, `ResultIsNotManifold { count: 4 }` (§378).*
- **A half bend with a quarter bend.** `classify_uncertain_point` answers
  `CannotFollowAFacesTrimming` on the half bend's torus face at every point
  the lane tried, the centre of its own cap included. A half bend's two end
  circles lie in one plane through the axis, and §243's step 3 refuses a wall
  whose two stored planes are one plane. That it is this step and not another
  is not traced. The wall is the ring on one side of that plane, so an answer
  exists and the refusal is a gap (§16.10). *2026-10-03, Q753: it was not step
  3. Both caps are stored as one plane, so the wall had one trimming and the
  general rule refused it (§377.2). The wall is now the half of the ring its
  wire keeps; of 24 fuses with a quarter bend, 14 answer and 10 refuse later,
  `ResultIsNotManifold { count: 4 }` (§377). *2026-10-04, Q756: all 24 now
  answer, pinned in `a_half_bend_fused_with_a_quarter_bend_on_one_torus.rs`
  (§377.5).*

The lane also claimed that only a second bend on the same torus can put a flat
piece on a torus disk through the public API. The verifier checked one leg of
that: a plane parallel to the ring's axis but off it is refused as a quartic,
even when it misses the torus (`intersect.rs`, lines 3313–3314). The other
legs, that a pipe, a ball and a cone are each refused against a torus, were
not checked. Nothing here builds on that claim.
