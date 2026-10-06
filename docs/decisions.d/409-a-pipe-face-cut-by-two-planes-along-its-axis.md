## 409. A pipe face cut by two planes along its axis — Q505's wedge, 2026-10-06

*Queue row Q880, the section Q505 (`5fe97a23`, `3593f714`) owed. Q505 built
the wedge and pinned what stays refused; this writes down its argument, the
flat-face argument beside it, and the probe's count by site, read off
`5f38a030`. The row's own shape — Q361's 6 x 6 x 2 box inside a bar of radius
5 — stopped reaching `is_a_family_this_follows` once `8f9750e2` (Q350) and
Q503 landed, so Q505 probed ten other bar-and-block shapes to find what was
still refused there.*

  409.1   The wedge: two planes along the axis, strictly inside   47 lines
  409.2   A flat face against a pipe, closed by name for another reason
                                                                   28 lines
  409.3   The probe: 114 refusals by site, and the three that stay 52 lines

### 409.1 The wedge: two planes along the axis, strictly inside

`is_a_family_this_follows` (`crates/rcad-topo/src/holds.rs`) is the test
`what_a_face_cut_by_part_of_a_crossing_holds`'s whole argument rests on: that
the part of a face's surface in front of every cut is one piece. For
`Surface::Cylinder`, before Q505, it asked only `leaves_a_pipe_in_one_piece`,
which carries one plane along the pipe's axis (or a second pipe running the
same way) beside at most a floor and a roof. A second plane along the axis
fell through to `false` — refused by name — because two arcs of a circle can
meet in two pieces, and nothing told that case from one.

Q505's `is_a_wedge_of_a_pipe` adds the one shape of that case it can show is
one piece: two planes along the axis whose line of intersection passes
**strictly inside** the pipe. Looked down the axis, the pipe is a circle and
each plane a line. Both planes' normals are square to the axis
(`SquareToTheAxis` settles that), so their line of intersection is parallel
to the axis too, and every point on it is the same distance from the axis —
it projects to one point in the cross-section, the wedge's apex. What both
planes keep, in that view, is the intersection of two half-planes: a wedge
with its point at the apex, its angle one of the four a pair of crossing
lines cuts the view into, so under a half turn.

From a point strictly inside a circle, every ray out of it leaves the circle
exactly once. So the rulings the wedge keeps — those whose direction from the
apex falls inside the wedge's angle — are exactly one connected run, the same
shape `leaves_a_pipe_in_one_piece` already reads off a single plane along the
axis. `is_a_wedge_of_a_pipe` asks that function again with one of the two
planes left out, so its floor-and-roof argument for the caps carries over
word for word, and calls the result one piece only when the other, left-out
plane also meets this condition.

Where the two planes meet is read without a division. With `nᵢ·x = dᵢ` the
two planes, `o` the pipe's axis origin, `eᵢ = dᵢ − nᵢ·o`, and `m = n₁ × n₂`,
the point `o + U/|m|²` with `U = e₁(n₂ × m) + e₂(m × n₁)` lies on both planes
(`nᵢ·U = eᵢ|m|²`, by the triple product), and — because `m` lies along the
axis — it is `|U|/|m|²` from the axis everywhere along that line. "Strictly
inside" is then the sign of `|U|² − r²|m|⁴` over stored numbers in intervals
(`do_two_planes_along_the_axis_meet_inside`): negative is inside, and
parallel planes read `m = 0`, both sides zero, and are refused rather than
guessed at.

Two parallel planes are the case this leaves to a different function,
`one_of_two_runs_between_two_flats`: when they keep the strip between them
rather than meeting, the pipe's wall is two runs, one each side of the plane
through the axis square to the flats, and which run a face is is read off the
half its corners sit in (§409.3's first bullet).

### 409.2 A flat face against a pipe: convex either way, closed by name

The other arm `is_a_family_this_follows` touches here is `Surface::Plane` cut
by a pipe, which `8f9750e2` (Q350) and Q503 had already carried for one pipe
cutting the plane in a closed curve. Reading the two side by side, on a cone
and on a pipe, looks like one argument; `is_a_family_this_follows`'s own doc
comment (added to by Q505) says why it is two:

- **On a cone**, convexity of the region kept needs the crossing to close: an
  open branch of a hyperbola leaves two pieces with nothing between them, so
  only `Cone::is_cut_in_a_closed_curve_by` (a circle or an ellipse, whichever
  nappe) gives a convex inside.
- **On a pipe, convexity never needed that.** The inside of a pipe is every
  point within its radius of the axis, which is convex regardless of how a
  plane cuts it — a plane along the axis keeps a strip between two rulings,
  which is convex too. So "the crossing closes" is not what makes the region
  this argument keeps convex here.

What closing buys instead is somewhere to put the face's own sides: a
straight side of a flat face is read as inside a pipe only where
`where_a_side_lies` can place it against a closed crossing, and an arc of a
pipe's own wall is only read off a closed curve too
(`the_way_into_an_arc`). A plane along the pipe's axis crosses it in two
rulings, which never closes, so a flat face with a side along one of those
rulings is not this family's case at all — it is a seam, and
`what_a_flat_face_holds` answers it directly instead of through this
argument.

### 409.3 The probe: 114 refusals by site, and the three that stay

Q505's own probe — ten bar-and-block shapes, six rule orders each,
instrumented at every `cannot()` of
`what_a_face_cut_by_part_of_a_crossing_holds` — is not kept; its 114-refusal
count is read off the commit message of `5fe97a23`, counted on `cd52afea`,
and was not rerun for this section. By site:

- 27 at `is_a_family_this_follows`, in two kinds: a pipe face cut by two
  planes along the axis (the quarter bar's wall, both orders, and a bar with
  two flats), and a flat face keeping a pipe's outside (a box with a bar
  bored across an edge, and the notch's floor and roof);
- 54 at a function Q505's own commit calls `the_sides_of`, "no way into an
  arc"; on this base that function is `every_reading_of_the_sides_of`,
  renamed and changed to return a reading per neighbour instead of one
  (`a82d5a20`, Q975, 2026-10-05) — the same step, under a new name;
- 12 at `which_side_of_its_neighbour_a_side_keeps`;
- 15 at `Lies::Across`;
- 6 at neither in front nor behind.

The wedge of §409.1 takes the first kind where it is one piece: 114 becomes
111, for the quarter bar's wall (both orders) and the block less a bar's
quarter-round wall. Three shapes stay refused by `is_a_family_this_follows`,
and each has an answer anyway, through a different, special-cased path rather
than through this generic argument — `3593f714`'s "what cannot reach it is
reading the face as the part of its surface in front of every cut" — so each
is a **gap** under §16.10, not a limit, and each is pinned refused in the
test file:

- **A bar with two flats, `|x| ≤ 3`**: each wall is one run of rulings, and
  the other wall sits on the same side of all four cuts, so "in front of
  every cut" would hold a point on it too. `one_of_two_runs_between_two_flats`
  answers this wall directly, by which half of the pipe its corners are in,
  without going through `is_a_family_this_follows` at all.
- **A flat face keeping a pipe's outside**, a `2 × 12` block with a radius-3
  bore down into it: the top is two pieces, each on the same side of all the
  other's cuts. `what_a_flat_face_of_chords_and_arcs_holds` answers both
  halves, reading each arc as its chord with the piece between added back or
  taken away.
- **The floor of a block with a bar bored through one edge**: one piece, but
  `is_a_family_this_follows` cannot tell it from the two-piece top above, so
  it still refuses by name. `is_a_polygon_less_one_bite_of_a_pipe` answers it,
  reading the pipe's circle against the polygon's edge instead.

`is_a_family_this_follows`, `is_a_wedge_of_a_pipe`,
`do_two_planes_along_the_axis_meet_inside`, `leaves_a_pipe_in_one_piece`,
`one_of_two_runs_between_two_flats`, `where_a_side_lies`,
`what_a_flat_face_of_chords_and_arcs_holds` and
`is_a_polygon_less_one_bite_of_a_pipe` are checked present on this base
(`just src`, all in `crates/rcad-topo/src/holds.rs`). The one rename among the
names this section cites is `the_sides_of`, above; none of the rest has moved
since Q505.
