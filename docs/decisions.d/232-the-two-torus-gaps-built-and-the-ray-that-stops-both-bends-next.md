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

`holds.rs`'s `what_a_round_wire_encloses` has the same rule — the partner's
sign alone — and so the same defect for a flat face bounded by a meridian
circle. It is outside this lane's files and is not fixed or measured here.

### 232.3 A node on a crossing whose partner is the asked surface is on that surface

`arrange.rs`'s `which_side_of_everything` asked a node on the far circle of the
plane `x + y = 0` which side of the cap circle's plane it was on — the same
plane, bit for bit. The answer is exactly zero, and a box `5.7e-13` wide cannot
certify zero. It now passes over a node that lists a crossing whose partner
equals the asked one, as it passes over any node on the surface.

This is exact, not a tolerance. A crossing runs through a node only where the
node is on it, and every point of a crossing is on its partner. So a node
listing a crossing with partner `P` is on `P` by construction.

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
