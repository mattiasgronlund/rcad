## 251. Q5: the hyperbola arc's bulge on a pointed cone, rerun on main — 2026-09-26

*Queue row Q5, from §229.7. Lane `lane-q5-finish`: `2d1d61ea` (the tip of
`lane-q5-swallowed-pointed-cone-rerun`, which had merged `184962b9` onto an
older main and pinned three cone trios in `78ba39a9`) with `50a3165e`, today's
main, merged into it. **`184962b9` lands.** Its own two tests fail without it,
eight rcad-topo pins move with it, and every new answer is held to a second
route. Nothing moved to a wrong answer.*

### 251.1 The merge

`git merge --no-edit 50a3165e` onto `2d1d61ea` resolved on its own, with no
conflict. `select.rs` carries both changes side by side: Q80's `Scratch::cut_from`
and `LyingOn` (a piece flush on two tools facing opposite ways is inside) are in
`choose` and `settle_against_each`, and `184962b9`'s change is in
`in_the_bulge_of`, the new `is_on_the_nappe_the_side_is_on`, and the new
candidate loop over `half_way_across_the_chord_to_a_cone` in `candidate_points`.
The one place the two touched the same lines is `candidate_points`, where main
had changed `half_way_into_either_arc` to return a `Vec`; the merged call
`candidates.extend(half_way_into_either_arc(flat, side))` is main's spelling.

### 251.2 Its own tests fail without it

With `select.rs` put back to `50a3165e`'s and the tests `184962b9` and
`78ba39a9` added run (`mise exec -- just t -p rcad-topo --test
fuse_laws_over_a_cone --no-fail-fast`), 12 run, 10 pass, 2 fail:

- `a_cube_wall_parallel_to_a_pointed_cones_axis_cuts_it_and_both_ways_answer`:
  *cone ∪ cube answers: "Undecidable on a point inside a piece of a face"*.
- `three_trios_that_refused_both_ways_now_answer_one_way_round`: the frustum
  8/2 trio stops at `[Some("the first union: Undecidable on a point inside a
  piece of a face"), Some("the first union: Undecidable on side of plane")]`
  where the pin says `[None, Some(…side of plane)]`.

With the merged `select.rs` all 12 pass. So the change is needed and stays.

### 251.3 Which half moves what

The change is two things: counting a hyperbola or parabola arc's bulge when
the point is on the arc's own nappe, and a new candidate half way from a cone
side's chord towards the cone. Run with each switched off in turn (a temporary
environment switch, removed again):

- **Candidates off, bulge on**: only the lane's own
  `a_cube_wall_parallel_…` fails. The frustum grid, the halved-slope control
  and the leaning-block frustum all go back to their old pins. So the bulge
  count alone reaches nothing; the pieces need a point to be counted at.
- **Bulge off, candidates on**: `a_cube_wall_parallel_…` fails, and the
  halved-slope control of `degenerate_curved_fuses.rs` still moves. That piece
  is bounded by an **ellipse** arc, whose bulge was counted before; it only
  lacked a candidate.

### 251.4 The argument for the bulge

`the_piece_holds` asks every curved side's partner first, so a point that
reaches `in_the_bulge_of` is inside the cone by its sign — on either nappe.
`is_on_the_nappe_the_side_is_on` then requires `(x − apex)·w` to have one
decided sign at the point and at both ends of the side. One nappe's inside is
a convex cone, so its part of any plane is one convex region `R`, bounded by
an ellipse, a parabola or one branch of a hyperbola. The side runs with `R` on
its left, and a line meets that boundary at most twice, so the chord between
the side's ends is inside `R` and cuts it in two: the part right of the chord
is bounded by the chord and the one finite arc of the boundary between them,
which is the side. So "inside `R` and right of the chord" is exactly between
the side and its chord — the ball's argument, needing only that `R` be convex.
Both ends on one nappe puts both on one branch, so the finite arc is the side
and not a piece of the other branch. A point on the other nappe, on the plane
through the apex square to the axis, or whose sign cannot be settled counts
nothing, which can only lose a candidate. A plane through the apex cuts
rulings, and a straight side is its own chord, so its bulge is empty and
`R` is on its left: nothing right of it is inside `R`.

### 251.5 Every pin that moved, and why the new one is right

Found with `mise exec -- just t -p rcad-topo --no-fail-fast` on the merged
tree: 1010 run, 8 fail. After re-deriving them: 1011 run, 1011 pass.

- **`ordering_points_along_an_endless_bend.rs`**,
  `fifty_nine_of_the_eighty_four_…` → `all_eighty_four_answer`: 59 → 84, no
  refusals left (was 25 × *a point inside a piece of a face*); and the rule's
  exceptions, renamed `…_is_wrong_about_all_it_turned_down`, 19 → 44. The 25
  are a block wall's piece inside the cone carried on above the frustum: one
  hyperbola arc and the roof's chord, two corners. **Right because**: the new
  `every_answer_on_the_grid_holds_what_the_shapes_hold_by_hand` fuses all 84
  under Union and Intersection, validates, and holds each volume to the
  frustum's closed form within `measure`'s own bound at deflection 0.01. On
  this grid the frustum's face never reaches a wall (its widest cap has radius
  2 and its axis is at least 3 from every wall), so what the two share is the
  frustum between the floor and the lower of its top cap and the roof,
  `π u (ρ(b)³ − ρ(a)³) / 3` with `ρ(z) = 1 + (z + 1)/u`. 168 answers, all
  within bound.
- **`fuses_that_must_agree.rs`**, `THE_WIDER_GRID_COMPARES_THIS_MANY`:
  118 → 168, every comparison made. The fifty new ones agree bit for bit both
  ways round, and their volumes are the previous item's.
- **`a_plane_cut_along_a_cones_other_nappe.rs`**, three families, renamed from
  `…_refuses_every_time` to `…_answers_right_or_refuses`. The file's own
  `fuse_and_check` holds every answer to `validate`, the solid count, the
  volume against closed forms and a column sum, and 9 300 grid points
  classified against the shapes; it panics on a wrong one and none did.
  - steeper plane across a frustum: 0 right / 84 point-in-piece / 12 order →
    **40 right** / 44 / 12.
  - frustum's apex inside the block: 0 / 24 point-in-piece / 8 naming →
    **22 right** / 8 naming / 2 `TwoPatchesOfTheMeshDrewTheSameSide`.
  - pointed cone, apex near a steeper face: 0 / 80 point-in-piece / 16 naming
    → **58 right** / 8 point-in-piece / 16 naming / 14 `CannotMeshAFace`.
  **A label in the first family was wrong.** Eight of its cases, lean 3/8 with
  crossing −3 or −6, were `OVERLAPPING` and miss the frustum: the plane
  `x = 3/8 (z − crossing)` meets the upper nappe first at `z = 9` or `z = 18`,
  above the top cap at 8, and clears the top rim by 0.125 in `x` for −3 and
  by 1.25 for −6. Their union
  came back as two solids, which is right; the case now computes the label
  from where the upper branch starts. It went unread while every case refused.
- **`a_plane_across_a_cone.rs`**,
  `a_block_leaning_through_a_frustum_is_cut_along_the_cone_off_the_frustums_own_face`:
  the refused piece was the block's `y = −10` wall at `z < −20`, one arc of
  the lower nappe's hyperbola and a chord. It is placed now — outside the
  frustum, kept — and the union refuses one step on, on *a point inside a
  piece of a face* for a piece of the **frustum's own face**, corners
  `(1, 0, 2)`, `(4, 0, 8)` and `(3.375, ±2.147, 8)`. A curved piece is asked
  its sides' signs and never wound, so this refusal is not `in_the_bulge_of`'s
  and was there before; it was simply never reached.
- **`degenerate_curved_fuses.rs`**, the halved-slope control of
  `a_wall_lying_on_a_cones_side_leaves_that_crossing_with_no_direction`:
  *a point inside a piece of a face* → `ResultIsNotManifold { count: 4 }`. The
  wedge's faces with normals `(−8, ±16, ·)` each cut the cone in an ellipse,
  both ellipses pass through the two points where the edge those faces share
  pierces the cone, and each face's piece inside the cone is an ellipse arc
  with that edge as its chord. The new candidate gets a point in both (§251.3),
  and `assemble` then names the arc on one plane and the arc on the other as
  one edge. Printed with a temporary `eprintln!` in `Edges::of`, since
  removed: corners 5 and 6 carry six sides, four of them sharing the key
  `(cone, false)` — the cone along the plane `(−0.431, −0.861, 0.269)` twice,
  the cone along `(−0.441, 0.882, 0.165)`, and that plane's own side. A
  refusal, not a wrong answer, and it is `assemble.rs`'s.
- **`fuse_laws_over_a_cone.rs`**: `184962b9`'s and `78ba39a9`'s pins hold
  unchanged on today's main.

### 251.6 What this leaves

- `crates/rcad-topo/src/fuse/assemble.rs`, `which_arc_of_a_circle_this_is`:
  two arcs of closed crossings on **two different planes** through the same
  two corners get one name, because the name holds the curved surface and the
  sign and not the plane. §245.4 added the surface for two crossings on one
  plane; this is the same gap across two. The halved-slope control of
  `degenerate_curved_fuses.rs` pins it.
- `crates/rcad-topo/src/fuse/select.rs`: *a point inside a piece of a face* on
  a curved piece of a frustum's face cut by a steeper plane —
  `a_plane_across_a_cone.rs`'s leaning block, now the first thing that union
  stops on, and the remaining 44 and 8 of the other-nappe families.
- The mesher: 14 `CannotMeshAFace` and 2 `TwoPatchesOfTheMeshDrewTheSameSide`
  in `a_plane_cut_along_a_cones_other_nappe.rs`, reached now that those fuses
  get past selection. Not looked into here.
- `crates/rcad-topo/tests/fuse_laws_over_a_cone.rs`: the two ignored
  swallowed-cone trios keep the reasons `184962b9` wrote for them; neither
  moved.

### 251.7 What the verifier found

An independent verifier read the argument, re-derived pins by hand, ran
mutations, and merged the lane onto `732a849a` (1014 rcad-topo tests passed).
Its verdict was to merge, with three changes that were made at merge.

- **Confirmed.** The nappe guard is proved and not assumed: it reads interval
  signs, and only `Pos` and `Neg` count. Convexity carries 251.4's argument
  over, so a point on the other nappe, or one whose nappe cannot be told, can
  only lose a candidate and never give a wrong answer. The apex cases are safe,
  but only because of the guard. Both of 251.2's tests fail on the base. The
  eight cases relabelled `APART` are right. Three grid shapes and the roof case
  were worked again by hand, all within the bound, and the new grid test's
  tolerance is exactly `volume_error()`. All 120 other-nappe answers match a
  Simpson-rule volume the verifier worked out, within the bound.
- **The guard had no test of its own.** Making it answer yes when it cannot
  tell **passed all of `rcad-topo`** (1011 of 1011), while a unit probe showed
  a triangle with a corner at the apex then holding points in the opposite
  wedge, which is a wrong answer. Counting every open cone side was caught only
  by a refusal changing its name in one test. So two unit tests went into
  `select.rs`'s `mod tests` at merge:
  `a_hyperbola_arcs_piece_holds_nothing_on_the_other_nappe` and
  `a_piece_with_a_corner_at_the_apex_holds_nothing_in_the_opposite_wedge`.
  Measured at merge: with `Some(false) => true` in `in_the_bulge_of`, which
  counts every open cone side, both fail; with the guard answering yes
  whenever a sign is not `Pos` or `Neg`, the second fails.
- `in_the_bulge_of`'s last arm names `Surface::Torus(_)` now (§40).
- The gap for a crossing of −6 is 1.25 and not 0.125; 251.5 and the test's
  doc comment say so now.
