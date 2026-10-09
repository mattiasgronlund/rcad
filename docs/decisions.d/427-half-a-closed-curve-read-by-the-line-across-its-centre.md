## 427. Half a closed curve is read by the line across its centre — 2026-10-09

*Queue row Q1496. The new reading is
`into_the_arc_square_to_the_chord_from_the_centre` in `holds.rs`, the fifth way
`the_way_into_an_arc` reads an arc (`mise exec -- just src
into_the_arc_square_to_the_chord_from_the_centre`), and its doc comment carries
the argument. §330.3's 2026-10-09 note on Q1496 is the short form of 427.1.*

### 427.1 The case that needed it

A round bar of radius six from `z = −2` to `z = 2`, cut back to `x ≥ 0`, turned
about `z` by `0.3`, `π/4`, `0.7853`, `1` or either of Q555's two-part turns, and
fused with an unturned half bend of ring four and tube one, was refused
`AFaceTrimmedByACurveThisCannotFollow` on the bar's cap at `z = −2` in all six
fuses, either side first. Unturned, every fuse answered.

The refusal came from the fuse's arrangement, `which_way_an_edge_runs_round` in
`fuse/arrange.rs`, when `the_way_into_an_arc` gave no way into the cap's arc.
Q1496's lead, `what_a_flat_face_of_chords_and_arcs_holds` in `holds.rs`, was
not reached: the row's first lane put a print in each refusal in `holds.rs`,
and none printed. The arc is half
the cap's circle, so its chord runs through the circle's centre, and its picker
is the plane of the bar's flat face, which holds the chord. At the turn of
`0.3` that picker is `825.4107266045236x + 255.3294585553974y ≥ 0`, and the fuse
solves one end into `x ∈ [−1.7731212399680383, −1.7731212399680367]`,
`y ∈ [5.732018934753634, 5.732018934753639]`, a box the plane runs through.

The four ways there were each needed something this arc does not have. The
first needs each end certainly on the picker's plane, and the second each end
certainly on or in front of it; the plane runs through both boxes. The third
and the fourth need the centre certainly on one side of the chord line, or
certainly behind the picker, and the centre is on both. The arc is one half or
the other, so the refusal was a gap.

### 427.2 The fifth way

The line through the centre, square to the chord, meets the curve at two points.
When they are certainly on opposite sides of the chord line, and the one on the
side the picker's normal leans away from is certainly behind the picker, that
side's arc is not the one picked, so the arc is the half the normal leans to.
For half a curve both points are a long way from either end and the picker's
plane runs through the centre, so both hold with room to spare however the
plane runs through the end boxes. Each inequality is multiplied through by `(n·along)²`, so the
centre is never formed and no division is taken.

It runs after the third and the fourth for each picker, and it stays out of
`a_picker_that_tells_the_two_arcs_apart`: assemble lowers a picker until that
says yes, and the fifth way never asks whether an end is in front of the plane.

The check that the two points are on opposite sides of the chord is not
decoration. On a circle they always are, since the chord line is nearer the
centre than the curve is. On a long ellipse they need not be: the plane
`−0.96x + 0.28z = 0` cuts a pipe of radius one up `z` in an ellipse reaching
3.57 from its centre one way and 1 the other, and for the short arc from `t =
0.1` to `t = 1` of `(cos t, sin t, 24/7·cos t)`, the line across the centre
square to its chord meets the curve twice on the centre's side. The picker
through the centre with normal `(0.28, −1, 0.96)` holds that arc in front and
leans to the centre's side, so reading the arc as the half the normal leans to
would turn it round. `the_fifth_way_reads_nothing_when_the_line_across_the_centre_misses_the_chord`
in `holds.rs` pins that the check stops it.

### 427.3 The second way misreads that same ellipse

The same ellipse, arc and picker, handed to `the_way_into_an_arc` whole, are
read wrongly by the **second** way. Both ends are in front of the picker and the
centre is on its plane, so `a_picker_that_tells_the_two_arcs_apart` says yes,
and the way returned reads negative against the arc's middle at `t = 0.55` less
its start: it points to the centre's side, and the arc is on the other. That
was run, once, as a test written for the purpose and then taken out.

The doc comment's argument for an ellipse, under "Why the second way works, on a
circle and then on an ellipse", is where it goes wrong. It maps the ellipse to
a circle by an affine map and says which side of the chord `f` grows towards is
unchanged. That holds when the picker's plane holds the chord's direction, so
that `f` is the same all along the chord. When it does not, `f` grows towards
neither side as such, and the sign of the normal against the direction square
to the chord depends on what "square" means, which an affine map does not keep.
On a circle the argument does not need that step, since the arc's middle is a
radius from the centre along the square direction, and it stands.

A picker the fuse writes is the plane through the chord, lowered, so it holds
the chord's direction up to the rounding of the corners it was built from, and
this probe's picker is tilted far off it. No fuse is known to reach the
misreading, so it is class C. Arrange reads arcs through the same function, so
it shares the misreading.

### 427.4 What this leaves

- **The second way on an ellipse** (427.3): read an ellipse's arc by the second
  way only where that is sound, or by a reading that is, and pin the ellipse
  above through `the_way_into_an_arc`. Filed as a row.
- **Two half bars spelling the plane apart.** Two such half bars about the
  bend's axis, the near one over `z ∈ [−2, 2]` and the far one moved up by 6,
  turned onto `x + y = 0`, one by each of Q555's spellings, now get past the cap.
  In each pairing, the three fuses with the bars named first are refused
  `Undecidable` on `side of plane` at the corner near `(4.2426, −4.2426, −2)`,
  and the three with the bend named first are refused
  `CannotTellTheCellsOfAFaceApart`. With both turned by the shorter spelling
  every fuse answers within its error of the closed forms: `3π²` shared,
  `π² + 144π` for the union, `144π − 3π²` and `π²` for the two differences.
  This was run once, as a test written for the purpose and then taken out. Those are the two
  refusals the quarter bars of §330.3 meet, Q1539's and Q1538's, but whether
  their causes are those rows' causes has not been traced. Filed as a row
  that waits on both.
