## 347. A node where a plane through a cone's apex crosses its two rulings — 2026-09-28

*Row Q207 (`lane-q207-a-node-at-the-apex`, off `f254b984`). §283 found every
fuse of a pointed cone and a block whose face `x = 0` runs through its apex
refused `CannotTellTheCellsOfAFaceApart`, because `place_the_meets` puts no
node where the plane's two rulings cross, and named a fix in two parts
(§283.5). Both parts are built, and a third was needed that §283.5 did not
see. Three of the six fuses now answer right, and four of six with the apex
just below the face. What refuses now refuses in the mesher.*

  347.1   What was built
  347.2   What each part is for, measured by taking it out
  347.3   What the fuses do now
  347.4   What is left
  347.5   Which pairs still get no node where their two curves meet
  347.6   §39.4's row and `CannotTellTheCellsOfAFaceApart`
  347.7   Where the numbers come from
  347.8   The frustum's floor through the apex (Q457)
  347.9   Two curves of one pair, brought together (Q471)

### 347.1 What was built

All in `crates/rcad-topo/src/fuse/arrange.rs`.

1. **A node at the apex.** `place_the_meets` hands a pair of crossings with
   one partner to `place_where_two_rulings_cross`. When the pair is a plane
   and a cone that `a_plane_cuts_a_cone_in_rulings` says cut in rulings, that
   puts a node at `Cone::apex`. The apex is stored, so the node is exact. Both
   rulings' pickers hold it: they are opposite half-spaces whose flat surface
   passes exactly through the apex, which `rcad-geom` proves before it names
   the pair, and an assert says so. The node is named by the plane and the
   cone whichever of them the face is, so both faces name the apex alike.
2. **A ruling runs along its own line.** `Crossing` carries `through_an_apex`
   for each of the two rulings. `along` is `n × p`, the plane's normal crossed
   with the normal of the half-space that picks the ruling. The order along the
   crossing, `way_out_of` and the turn between two ways out all read `along`,
   not `∇S × ∇Q`. The turn is `N · (t₁ × t₂)` over the two ways out themselves
   (`turn_between_two_ways_out`), which is the old determinant before
   `(a×b)×(a×c) = a·((a×b)·c)` took it apart. Pairs with no ruling in them
   keep the old determinant bit for bit.
3. **The partner's side turns round at the apex.** Along `apex + t·d` the
   cone's gradient is `t` times one vector, so `∇S × ∇Q` is `t·V`: it points
   away from the apex on both sides, or towards it on both. That is
   `towards_the_apex`, read once a step out. Each arc then says whether
   forward runs against `∇S × ∇Q` along it (`against_its_gradients`), and
   `wire_from` turns the material side round for one that does. Two arcs either
   side of the apex are never joined into one.
4. **On a face on the cone, the apex is one node per nappe**
   (`part_the_two_nappes_at_the_apex`). This is the part §283.5 did not see.
   The rulings run on through the apex into the nappe the solid does not have,
   so the skin's drawing has ways out of the apex into both nappes. The node
   keeps the nappe the face's boundary leaves into and a new node at the same
   point takes the other. The other nappe's arcs are then a lump of straight
   cuts with no boundary in it, and `drop_the_lumps_of_straight_cuts_alone`
   throws that away.
5. **At a cone face's own apex the turn is about the axis**
   (`what_a_turn_at_the_apex_is_about`): `−w` on the nappe along `+w`, and `+w`
   on the other. On the `+w` nappe, `det(t·G(d), d, t·d′(θ))` works out to
   `−t²·slope²·(1 + slope²)` for a cone about `+z`, so the rulings go round
   clockwise about the gradient as `θ` climbs. `leaves_the_node_the_same_way`
   compares ways out there as they run round the axis (`round_the_axis`).

### 347.2 What each part is for, measured by taking it out

Each of these came up while building, on the union with the cone first — the
first fuse either pin asks — unless it says otherwise.

- **Part 1 alone** refuses `TwoCurvesTouchWithoutCrossing` on face `2v1`,
  **the cone's skin**, not on the block's face `8v1`, both for the apex on the
  face and for the raised-floor control. §283.5 said part 1 alone might only
  swap the refusal; it does, on the other face.
- **Everything but the split** refuses the same, at the skin's apex. A
  temporary print showed the two ways out there as `(0, 0.5, 1)` and
  `(0, 0.5, −1)`, one up each nappe. It also showed the drawing: the skin is cut
  by all six of the block's faces, and the rulings meet `y = ±6` and `z = −2`
  on the other nappe at `(0, ∓6, −12)` and `(0, ∓1, −2)`, so each ruling has
  arcs there.
- **With the split but ways out compared in space** it refuses the same, in
  `sorted_anticlockwise`'s tie-break. A plane through the cone's axis cuts two
  rulings half a turn apart round the axis. So the turn about the axis is
  nothing, and in space the two go the same way up the nappe.
- **With `along` turned to agree with `∇S × ∇Q` a step out**, which was the
  first form of part 3, the frustum clear of the apex in
  `a_frustum_cut_in_half_down_its_axis.rs` went from six answers to six
  refusals on "a point inside a piece of a face". A print of the skin's pieces
  showed one ruling side of the `x > 0` half carrying `Neg` where every point
  of that half is on the partner's `Pos` side. Turning `d` round turns round
  which side of the apex it points to as well, so that sign cannot be chosen
  by orienting `d`. It is the crossing's, and `towards_the_apex` holds it.

### 347.3 What the fuses do now

`a_block_face_through_a_cones_apex.rs`, the apex on the face (floor `z = −2`):

| fuse | now | right answer |
|---|---|---|
| union, cone first | `CannotMeshAFace`, `CornersCoincide` | `864 + 64π/3 ≈ 931.0206` |
| union, block first | `AWireDoesNotFanIntoALoopToMesh`, `TooFewCorners { count: 2 }` | the same |
| intersection, either order | one solid, validates, `67.0165 ± 0.0374` | `64π/3 ≈ 67.0206` |
| cone less block | one solid, validates, `67.0165 ± 0.0374` | `64π/3` |
| block less cone | `AWireDoesNotFanIntoALoopToMesh`, `TooFewCorners { count: 2 }` | `864 − 64π/3 ≈ 796.9794` |

The raised-floor control (floor `z = 1`, the apex just below the face):

| fuse | now | right answer |
|---|---|---|
| union, either order | `CannotMeshAFace`, `CornersCoincide` | `648 + 128π/3 − 511π/24` |
| intersection, either order | one solid, validates, `66.8852 ± 0.0554` | `511π/24 ≈ 66.8894` |
| cone less block | one solid, validates, `67.1469 ± 0.0571` | `513π/24 ≈ 67.1512` |
| block less cone | one solid, validates, `581.1148 ± 0.0554` | `648 − 511π/24 ≈ 581.1106` |

Every answer is inside the mesh's own error of the volume worked by hand. The
two pins are renamed to what they show:
`a_block_face_through_the_apex_cuts_the_cone_in_half_and_the_mesher_refuses_the_rest`
and `the_same_block_with_the_apex_just_below_its_face_answers_all_but_the_unions`.
Each asserts the answers — one solid, validated, measured — and each refusal
by the mesher's own name, so a mesher fix moves them. The raised floor moved
because §283.3 was right that it was likely the same missing node. It took
the node and the rulings' direction together, so the refusal was not only the
node.

**2026-10-04 (Q862, `70e6c705`):** every fuse in both tables answers now,
one solid, validated, inside the mesh's own error of the right answer. The
four refusals were not the mesher's. `Assembly::what_is_left` read the apex
as a point walked straight through where only the two rulings left it, and
folded them into one straight edge between their far ends, beside the cap's
chord between the same two corners. It now keeps every corner that is the
apex of a cone an edge there is carried by (347.4's note). The pins are
renamed `a_block_face_through_the_apex_cuts_the_cone_in_half_and_every_fuse_answers`
and `the_same_block_with_the_apex_just_below_its_face_answers_every_fuse`,
and each pins the face count as well. With the floor at `z = 1` the cone's
skin outside the block comes back as three faces of one cone where one would
do: ten faces for the unions where eight would do, and six for the cone less
the block where four would. `Assembly::edges_a_curved_face_keeps` leaves the
three apart, because joined they are one ring of four sides that goes right
round the apex and is not a band between two rings. A temporary print there
showed that group in both unions and the cone less the block.

### 347.4 What is left

- **The mesher.** Every refusal left in 347.3 is the check a fuse makes by
  meshing a curved answer before it writes it. The answers refused are the
  ones that keep the cone's skin cut down to the apex against the block, or the
  block's face with the apex on its edge. Nothing here says which of
  `mesh.rs`'s steps meets which: the triangulator saying two corners land on
  one point of the chart, and a wire fanning into two corners, are what the
  errors say and no more. §285 has a mesher refusal of the same name on a
  pointed cone's cap cut round its apex. `mesh.rs` is outside this row's write
  set.

  **2026-10-04 (Q862, `70e6c705`):** the mesher was right to refuse. Q468
  traced every refusal here to the shell it was handed, the two rulings
  folded into one straight edge through the apex by `Assembly::what_is_left`
  in `fuse/assemble.rs`, and a probe that kept the apex answered all twelve.
  The rule is built: a corner that is the apex of a cone an edge there is
  carried by is never walked straight through. It is an exact match of the
  corner's point against `Cone::apex` by `exactly_at`, the same bits
  `Corners::of` already joins two corners by, since the arrangement puts the
  node where the two rulings cross at the apex as the cone stores it. A
  corner one `f64` off the apex is not kept on that ground
  (`only_a_corner_exactly_at_a_cones_apex_is_read_as_the_apex`). A direction
  test at the corner was the other way, and was not taken: it would read two
  rounded directions, and on a nearly flat cone the two rulings a plane
  through its axis cuts are nearly opposite. Keeping the apex is never wrong,
  because a face on a cone lies on one nappe and two rulings of one nappe are
  never opposite. What it leaves is the bullet below on pieces of one cone
  round its apex.
- **Pieces of one cone round its apex stay apart** (2026-10-04, Q862). With
  the floor at `z = 1`, the cone's skin outside the block is cut along the
  rulings below the floor and round the cone at `z = 1` into three pieces of
  the one cone. Joined they are one face whose one ring, of four sides, goes
  right round the apex, with the apex inside it as on the input's own skin.
  `Assembly::edges_a_curved_face_keeps` leaves such a group apart, since a
  seam is only looked for across a band between two rings. The doc comment
  on `Assembly::the_corners_each_closed_run_keeps` records a ring of arcs
  right round a pointed cone with no seam refused by the writer
  (`AWireWillNotLieFlatOnItsFace`), measured on a pointed cone less a block
  whose walls touch its rim, so joining these is more than dropping the
  rule. The answer is the right solid and validates, with two faces too
  many and edges with one cone on both sides.

  **2026-10-05 (Q989):** joined. `Assembly::needs_no_seam` now takes one
  ring right round a cone, and nothing else, as a cap round its apex: the
  other piece that ring cuts the nappe into is unbounded, so the face holds
  the apex, and the mesher cuts it up to the apex
  (`cut_a_cap_up_to_its_pole`) with no seam kept. The unions answer 8 faces
  and the cone less the block 4, validated and inside the mesh's own error
  of the volumes in 347.3; `mise exec -- just t -p rcad-topo --test
  a_block_face_through_a_cones_apex` pins them. Over the workspace, a
  temporary print in the new arm showed it reached in three more tests, all
  passing: `fuses_that_touch.rs`'s
  `a_pointed_cone_standing_on_its_apex_on_a_block` (rings of 4 and 8 sides),
  and `how_often_a_fuse_with_a_cone_refuses.rs`'s
  `how_often_a_certified_fuse_with_a_cone_in_it_refuses` and
  `uniting_a_cone_with_a_cube_gives_the_same_answer_whichever_way_round_it_is_asked`
  (rings of 3, 6 and 9 sides, seven pieces each, the corner-up cube round a
  pointed cone's apex).

  **2026-10-16 (Q999):** compared, by running each with `--no-capture` at
  `8ac9f1c8` (the tree right before this change) and at `78c8df3a` (this
  change's merge). None of the three tests' printed counts moved.
  `how_often_a_certified_fuse_with_a_cone_in_it_refuses` printed the same five
  lines both times: a frustum against a corner-up cube 96 fuses, 48 answered,
  48 refused; a pointed cone against a corner-up cube 96 fuses, 84 answered,
  12 refused; a frustum against a block 96 fuses, 96 answered, 0 refused; a
  frustum against a bar 96 fuses, 0 answered, 96 refused; two frustums on one
  axis 96 fuses, 0 answered, 96 refused.
  `uniting_a_cone_with_a_cube_gives_the_same_answer_whichever_way_round_it_is_asked`
  printed the same line both times: a cone and a corner-up cube, both ways
  round, 44 pairs compared, 20 refused both ways.
  `a_pointed_cone_standing_on_its_apex_on_a_block` printed the same counts
  both times: touching, 15 shapes, 98 laws checked, 28/28/28 answered and
  2/2/2 refused for difference/intersection/union; ClearByOne, 11 shapes, 77
  laws checked, 22/22/22 answered for difference/intersection/union. The rim
  case on `Assembly::the_corners_each_closed_run_keeps` was not measured
  again.
- **A pin outside Q207's write set moved.** In
  `a_frustum_cut_in_half_down_its_axis.rs`,
  `a_block_whose_floor_runs_through_the_apex_is_refused_there` pinned all six
  refused `Undecidable` on "which way a crossing runs". That was the direction
  read at the apex, which part 2 removes. Row Q457 rewrote that file: 347.8
  says what the floor through the apex does now, and what a probe showed of
  why three of the six are not a manifold.
- **One ruling through an apex with no node there.** A face whose edge runs
  along one ruling from one nappe to the other through the apex has no second
  ruling to cross it, so it gets no node at the apex. Its arc across the apex
  has the partner's inside on both sides, and `runs_against_its_gradients`
  refuses it `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`. No test builds
  one.
- **A plane resting along a cone**, one ruling touched all along, no longer
  refuses by `how_a_crossing_runs`: §39.4's 2026-10-04 note (Q473) reads the
  ruling instead.

### 347.5 Which pairs still get no node where their two curves meet

`place_the_meets` still skips every other pair of crossings with one partner.
`where_the_ray_crosses`'s doc comment used to say every place two crossings
meet has a node. It now says which pairs it is not true for. The pairs
`rcad-geom`'s `intersect` names two curves for:

| pair | the two curves |
|---|---|
| plane and pipe, along the axis | two parallel lines |
| two pipes side by side | two parallel lines |
| ball centred on a pipe's axis | two circles in parallel planes |
| cone and pipe about one axis | two circles in parallel planes |
| plane square to a torus's axis | two circles round the axis, one inside the other |
| plane through a torus's axis | the two meridian circles, either side of the axis |
| plane and cone, apex off the plane, `E < 0` | two hyperbola branches, one per nappe |
| ball and pipe, off the axis | two loops either side of a plane |
| two leaning pipes | two loops |
| plane and cone, apex on the plane, `E < 0` | two rulings through the apex — **the node of 347.1** |

Where two of those loops touch, `intersect` refuses by name,
`ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo`. `rcad-geom` pins that
for the ball and the pipe
(`a_ball_resting_on_the_far_inside_of_a_pipe_crosses_it_in_a_curve_with_no_name`)
and for two pipes (`two_leaning_pipes_of_the_same_thickness_whose_axes_meet_cross_in_a_curve_with_no_name`).
*Since 2026-10-10 (Q1603, landed with Q1607) two pipes of one width whose
axes meet are named, as four half ellipses that meet where the walls touch,
and that pin is
`two_leaning_pipes_of_the_same_thickness_whose_axes_meet_cross_in_four_half_ellipses`;
see the note at the end of 347.9.*
The plane-and-torus row refuses the two bitangent planes' Villarceau circles,
which do cross, as a quartic. The rest are two curves apart by what the pair
is. **None of that was tried here** by building two curves of one pair that
meet and running an arrangement on them, so the claim is "not reached by any
shape this row built", not "cannot be reached".

**2026-10-09 (Q471):** measured. 347.9 builds every pair above and runs
each family to where its two curves would meet. Outside the apex's rulings,
the two curves of a pair meet only where `intersect` refuses the shape by
name, `ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo` for the two loop
pairs, or where it names one curve, or where the surface cannot be made. So
no shape reaches `place_the_meets` with two curves of one pair that meet
today, and the two loop pairs will once their refusals are lifted.

### 347.6 §39.4's row and `CannotTellTheCellsOfAFaceApart`

§283.6 asked for two edits, neither made here: drop "undemonstrated", and stop
saying the fix for this cause is "a point inside each".

- **This fix takes away §283's demonstration.** The block face through the apex
  no longer refuses `CannotTellTheCellsOfAFaceApart`. That leaves §49.6's
  demonstration alone: `a_ball_and_a_bar_in_one_block.rs`'s
  `the_same_three_one_pair_at_a_time_stop_somewhere_else`, a rim wire of one
  closed edge. "Undemonstrated" can still go, on that one.
- **The second edit is moot for this cause**, which no longer reaches the
  variant. Whether "a point inside each" is the fix for §49.6's cause was not
  looked at.
- The classification stays **gap**.

`039-*.md` and `error.rs` are not this row's, and neither was edited.

### 347.7 Where the numbers come from

- The fuses in 347.3, both tables: `mise exec -- just t -p rcad-topo --test
  a_block_face_through_a_cones_apex`. The volumes were printed by a scratch
  test in the same file that measured each answer at a deflection of `0.001`
  and was removed before the commit. The committed pins assert the same
  answers against the same bound.
- The frustum's floor, first measured by Q207 and now in 347.8: the same
  scratch test, with the frustum `cones::a_frustum(model, 0.5, 4.0, 8.0)`
  and the block `[−10, 0] × [−10, 10] × [0, 12]`, run with `--no-capture`.
- What each part is for, 347.2: temporary `eprintln!`s in `arrange.rs`, run
  through the same file and through `a_frustum_cut_in_half_down_its_axis`,
  all removed.
- The suite: `mise exec -- just t --workspace --no-fail-fast` on the branch's
  last code, `2695 tests run: 2694 passed (1 slow), 1 failed, 11 skipped`. The
  one is the old frustum floor pin of 347.4, which Q457 rewrote (347.8).
  `mise exec -- just gate quiet` fails on that same test and on nothing before
  it.
- By hand: `64π/3 = 67.0206`, `511π/24 = 66.8894`, `513π/24 = 67.1512`,
  `648 − 511π/24 = 581.1106`, `56π/3 = 58.6431` (the last from `python3 -c`).

### 347.8 The frustum's floor through the apex (Q457)

*Row Q457 merged this branch into its own (a merge, so the shas above hold)
and rewrote `a_frustum_cut_in_half_down_its_axis.rs`, whose floor pin 347.4
moved.* The frustum is `cones::a_frustum(model, 0.5, 4.0, 8.0)`, cut by the
block `[−10, 0] × [−10, 10] × [floor, 12]`.

**Before §347**, with the floor at `z = 0`, all six fuses refused
`Undecidable` on "which way a crossing runs", pinned by
`a_block_whose_floor_runs_through_the_apex_is_refused_there`.

**Now, floor at `z = 0`**, the apex a point on the block's bottom edge:

| fuse | now | right answer |
|---|---|---|
| intersection, either order | one solid, validates, `58.639165886922406 ± 0.028014994530896106` | `56π/3 = 58.64306286700947` |
| frustum less block | one solid, validates, `58.63916588692237 ± 0.028014994530896103` | `56π/3` |
| union, either order | `ResultIsNotManifold { count: 1 }` | `2400 + 56π/3` |
| block less frustum | `ResultIsNotManifold { count: 1 }` | `2400 − 56π/3` |

`a_floor_through_the_apex_still_gives_the_half_frustum_under_intersection_and_difference`
and `a_floor_through_the_apex_is_not_a_manifold_under_the_fuses_that_keep_the_floor`
pin the two halves.

**Floor at `z = −2`**, the apex inside the block's face `x = 0`: all six
answer, one solid each, and the model validates. The union, frustum first,
measures `2858.639165886922 ± 0.028014994530896103`, block first
`2858.6396935349926 ± 0.04206920874423824`, against `2800 + 56π/3`; the block
less the frustum `2741.3603148433963 ± 0.04210757287410164` against
`2800 − 56π/3`; the intersections and the frustum less the block the same
`58.6392` as at `z = 0`. Pinned by
`a_block_face_with_the_apex_inside_it_cuts_the_frustum_in_half_under_every_rule`.
The file's old comment said this floor refused `CannotTellTheCellsOfAFaceApart`
before §347, as the pointed cone of 347.3 did. That was never pinned, and
Q457 did not measure it again.

**Why the floor at `z = 0` is not a manifold, as far as it was shown.** A
temporary print in `split_a_face`, on the union with the frustum first:

- The block's face `x = 0` walks its bottom edge as two sides,
  `(0, −10, 0) → (0, 0, 0)` and `(0, 0, 0) → (0, 10, 0)`: the node of 347.1 is
  on that edge.
- The floor `z = 0` is cut by the planes `x = −10`, `y = ±10` and `x = 0` and
  by no cone. Its nodes are its four corners only, and it walks the same edge
  as one side, `(0, 10, 0) → (0, −10, 0)`.
- `intersect` of the plane `z = 0` facing `−z` with the cone is
  `TouchAtOnePoint { at: Exact([0, 0, 0]) }`, and `cuts.rs`'s
  `how_two_faces_meet` reads a touch at one point as two faces that meet
  nowhere. So the floor never learns of the apex.

Every leg of the guess in 347.4 held. **Not shown**: that the edge
`assemble.rs` finds claimed by one face is this one; nothing printed it. It
is the likely cause, not a shown one. Every fuse that keeps the floor beside
the pieces of `x = 0` either side of the apex is refused, and every fuse that
keeps neither answers. **A gap, not a limit**: every answer exists. The fix
is queued, with two candidates: the floor learns of the apex as a point on
its boundary, or `assemble` drops a vertex that is no corner on a straight
edge, as §245.3 does for a closed edge.

**The first candidate landed in Q1337** (§31.3): `cuts.rs`'s
`how_two_faces_meet` hands a touch at one point to both faces as a cut, and
`arrange.rs`'s `place_the_touches` puts a node at the point once it is proved
to lie on exactly one crossing drawn on the face, so the floor now learns of
the apex. **Q474**
(`lane-q1337-q474-a-floor-through-a-cones-apex-answers`) then answered the
floor-through-the-apex fuses under every rule, merging the two floor `z = 0`
pins above,
`a_floor_through_the_apex_still_gives_the_half_frustum_under_intersection_and_difference`
and
`a_floor_through_the_apex_is_not_a_manifold_under_the_fuses_that_keep_the_floor`,
into one: `a_floor_through_the_apex_cuts_the_frustum_in_half_under_every_rule`
in `tests/a_frustum_cut_in_half_down_its_axis.rs`. The table's union and
block-less-frustum rows for floor `z = 0` are stale along with those two test
names — all six fuses now give one solid each, the model validates, and every
volume is the closed form `56π/3`, or `2400 ± 56π/3`, within what the mesh
says it may miss by.

`a_half_frustums_corners_on_two_rulings_stay_two_corners` did not move.

Where the numbers come from: a scratch test in
`a_frustum_cut_in_half_down_its_axis.rs` run with `--no-capture` at floors
`0` and `−2`, and the temporary print in `arrange.rs`, all removed before the
commit; `56π/3` by hand. Q457's verifier reverted `crates/rcad-topo/src` to
`93768fd3` with the tests kept: the three new floor tests and both pins in
`a_block_face_through_a_cones_apex.rs` fail, and all pass on the branch.

### 347.9 Two curves of one pair, brought together (Q471) — 2026-10-09

*Row Q471 (`lane-q471-two-curves-of-one-pair-that-meet`, off `5d65cf3f`),
tests only, in three relays (§416). 347.5 said only that no shape Q207 built
brought two curves of one of its pairs together.
`crates/rcad-topo/tests/two_curves_of_one_pair_that_meet.rs` builds each pair
through the public API, runs every fuse on it, three rules each way round,
certified, in a model of 100 mm, and takes each family to where its two curves
would meet. No source changed.*

**Where two curves of one pair can meet.** Where two surfaces cross, each has
a normal there, and the two normals are not parallel, the crossing near that
point is one smooth curve and nothing else. So two curves of one pair can share
a point only where the surfaces are tangent, or where one of them has no
normal. A plane, a pipe and a ball have a normal everywhere. A ring has one
everywhere unless its tube is at least as wide as the ring, and `Torus::new`
refuses that ring, built or read from a file
(`a_ring_whose_tube_is_as_wide_as_the_ring_cannot_be_made`). That leaves a
cone's apex. So in each pair below, the two curves come together only as the
shape is pushed to a tangency or through an apex, and the last column is that
limit. At a tangency `intersect` names one curve, or refuses the shape by name
before any arrangement, or one of the two surfaces cannot be made.

Faces are listed union, intersection, first less second, second less first;
`3+3` is two solids of three faces each.

| pair | built | the six fuses, curves apart | where the two would meet |
|---|---|---|---|
| plane and pipe, along the axis | bar r 2 up `z` from 0 to 4, block `[1, 5] × [−5, 5] × [1, 3]`: lines at `y = ±√3` | all answer, faces 14, 4, 10, 8 | a plane resting on the pipe: one line |
| two pipes side by side | bars r 2 up `z` at `(0, 0)` and `(2, 0)`: lines at `x = 1`, `y = ±√3` | all answer, faces 10, 4, 10, 4 | axes `r₁ + r₂` or `r₁ − r₂` apart: one line |
| ball centred on a pipe's axis | ball r 2, bar r 1 down its axis from `z = −3` to 3: circles at `z = ±√3` | all answer, faces 5, 3, 2, 3+3 | a ball as wide as the pipe: one circle |
| cone and pipe about one axis | pointed cone of slope 1 up to `z = 4`, bar r 1 from `z = −2` to 6: circles at `z = ±1` | all answer, faces 6, 3, 3, 3+3 | a pipe of no width or a flat cone: neither can be made |
| plane square to a torus's axis | top half of a ring of 4 round a tube of 1, bar r 6 about its axis from `z = −2` to `1/2`: circles of radius `4 ± √3/2` | all answer since Q1514, faces 5, 4, 2, 7; refused `CannotTellTheCellsOfAFaceApart`, 6 of 6, before it | the plane on top of the tube: one circle |
| plane through a torus's axis | the same half ring, the `x ≥ 0` half of a bar r 10 from `z = −2` to 5: both meridian circles on its face `x = 0` | all answer, faces 6, 4, 4, 6 | a tube as wide as the ring: cannot be made |
| plane and cone, apex off the plane | the same cone, block `[1, 6] × [−6, 6] × [−2, 6]`: a hyperbola branch on each nappe | all answer, faces 8, 3, 3, 8 | a plane through the apex: 347.1's rulings and node |
| ball and pipe, off the axis | ball r 2, bar r 1 from `z = −3` to 3 at `x = 1/2`, and three more placements: two loops | refused `ACrossingThisCannotBeOrderedAlong`, 6 of 6 in each placement | the bar at `x = 1`, touching the ball from inside (Viviani): refused in `intersect`, `ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo`, 6 of 6 |
| two leaning pipes | bar r 2 up `z` from −2 to 2, bar r 1 along `x` at `y = 1/2`: two loops | all answer, faces 7, 3, 4, 3+3 | the thin bar at `y = 1`, resting inside, and two bars of r 1 whose axes meet (the Steinmetz cross): both refused in `intersect` the same way, 6 of 6 each |
| plane and cone, apex on the plane | 347.3 | 347.3 | the node of 347.1 |

Every answer is the solids its face counts list, `Model::validate` passes, and
their volumes add up to the closed form within what the mesh at deflection
`0.01` says it may miss by. Each test's doc comment gives its closed form; the
two leaning pipes' is a 20 000-step midpoint rule.

The pins, pair by pair, the answer first and the limit after:

- Plane and pipe:
  `a_block_along_a_bar_cuts_its_pipe_in_two_lines_and_every_fuse_answers`,
  `a_plane_resting_on_a_pipe_touches_it_along_one_line`.
- Two pipes side by side:
  `two_bars_side_by_side_cross_in_two_lines_and_every_fuse_answers`,
  `two_pipes_resting_side_by_side_touch_along_one_line`.
- Ball on a pipe's axis:
  `a_ball_on_a_bars_axis_crosses_it_in_two_circles_and_every_fuse_answers`,
  `a_ball_on_a_pipes_axis_as_wide_as_the_pipe_touches_it_round_one_circle`.
- Cone and pipe:
  `a_cone_and_a_bar_on_one_axis_cross_in_two_circles_and_every_fuse_answers`,
  `a_pipe_of_no_width_and_a_flat_cone_cannot_be_made`.
- Plane square to a torus's axis:
  `a_half_ring_cut_by_a_round_plate_crosses_it_in_two_circles_and_every_fuse_answers`,
  `a_plane_resting_on_top_of_a_ring_touches_it_round_one_circle`.
- Plane through a torus's axis:
  `a_half_bar_on_a_rings_axis_cuts_the_tube_in_two_circles_and_every_fuse_answers`,
  `a_ring_whose_tube_is_as_wide_as_the_ring_cannot_be_made`, and the floor
  below, `a_half_bar_whose_floor_the_whole_ring_would_rest_on_is_refused_ordering_that_circle`.
- Plane and cone, apex off the plane:
  `a_block_beside_a_cones_apex_cuts_both_nappes_and_every_fuse_answers`,
  `a_plane_cuts_a_cones_two_nappes_apart_unless_it_runs_through_the_apex`.
- Ball and pipe off the axis:
  `a_ball_with_a_bar_through_it_off_its_axis_is_refused_ordering_the_loops`,
  `a_bar_touching_a_ball_from_inside_is_refused_before_any_arrangement`.
- Two leaning pipes:
  `a_thin_bar_through_a_thick_one_crosses_it_in_two_loops_and_every_fuse_answers`,
  `a_thin_bar_resting_on_the_inside_of_a_thick_one_is_refused_before_any_arrangement`,
  `two_bars_of_one_radius_whose_axes_meet_are_refused_before_any_arrangement`,
  since 2026-10-10
  `two_bars_of_one_radius_whose_axes_meet_are_refused_telling_the_cells_of_a_pipe_apart`
  (the note at the end of this section).

**What it settles.**

- With the two curves apart, seven pairs answer every fuse right. Two are
  refused before anything is asked about a meet: the plane square to a
  torus's axis by Q1514's `CannotTellTheCellsOfAFaceApart`, and the ball and
  pipe off the axis by `ACrossingThisCannotBeOrderedAlong`, the gap §39.4
  lists for a ball's loops, still open after Q1483 ordered two pipes' loops.
  *Since Q1514 (2026-10-10) the plane square to a torus's axis answers too,
  so eight pairs answer and only the ball and pipe off the axis is refused.*
- Where the two curves would meet: for six pairs no shape the kernel can make
  gets there, since the limit is one curve or a surface that cannot be made.
  For the plane and cone it is the apex, 347.1's node. For the two loop pairs
  shapes do get there, and `intersect` refuses them by name before any
  arrangement. So nothing hands `place_the_meets` two curves of one pair that
  meet, other than the rulings at an apex. That holds until the loop pairs are
  named, and their rows ask what `place_the_meets` needs then.

> **2026-10-10 note (Q1554): the ball and pipe off the axis answer with the
> ball first, and are refused meshing with the bar first.** Q1554 orders the
> two loops round the bar's axis (`the_way_a_loop_round_a_pipe_through_a_ball_winds`
> in `along_a_crossing.rs`), so the row above no longer stops at
> `ACrossingThisCannotBeOrderedAlong`. With the ball first, all 12 fuses of the
> four placements answer, each held to the ball, the bar and the ball inside
> the bar by a midpoint rule (`ball_in_bar`):
> `a_ball_with_a_bar_through_it_off_its_axis_crosses_it_in_two_loops_and_every_fuse_with_the_ball_first_answers`.
> With the bar first, all 12 are refused `CannotMeshAFace { why: CornersCoincide }`:
> `a_ball_with_a_bar_through_it_off_its_axis_is_refused_meshing_every_fuse_with_the_bar_first`.
> Traced for the union, the marcher's first step on the bar's wall
> (`one_step` in `march.rs`) settles back one rounding off the loop's corner and
> is kept as a step of `5.9e-17`, and the steps after it double from there;
> that is a row of its own. The two replace
> `a_ball_with_a_bar_through_it_off_its_axis_is_refused_ordering_the_loops` in
> the pins above. Where the two loops meet, the bar touching the ball from
> inside, is still refused in `intersect`
> (`a_bar_touching_a_ball_from_inside_is_refused_before_any_arrangement`), so
> what this section settles about `place_the_meets` stands: nothing yet hands
> it two loops that meet.

> **2026-10-24 note (Q2649): the bar touching the ball from inside answers
> too.** The pin above, `a_bar_touching_a_ball_from_inside_is_refused_before_any_arrangement`,
> no longer holds under that name. Since Q2649 `place_the_meets` puts a node
> where the two loops meet, at `(2, 0, 0)`, and every fuse answers, each held
> to a corner there:
> `a_bar_touching_a_ball_from_inside_crosses_it_in_two_loops_that_meet_and_every_fuse_answers`
> in `two_curves_of_one_pair_that_meet.rs`. So `place_the_meets` is now handed
> two loops that meet, for this pair.

**The coaxial pairs 347.5 leaves out.** A ring with a pipe, a ball or a cone
on its axis, a cone and a ball, and two cones, each on one axis, cross in
circles round the axis at heights apart.
`coaxial_circles_come_together_only_where_their_surfaces_touch` takes each to
its limit. A pipe or a ball round the tube or in the hole, a cone resting on
the tube (slope `3/4`, apex at `z = −7`, touching at `ρ = 4.8`, `z = −0.6`),
and a ball resting in a cone each give one circle. Two cones of slopes
`s₁ ≠ s₂` with apexes `d` apart cross at `s₂·d/(s₂ ± s₁)` from the first apex,
one height only for `d = 0`, where they touch at the apex and nowhere else
(`TouchAtOnePoint`, in `where_two_cones_cross`'s table; the test counts it as
no curve). None of these was run through a fuse.

**Why a block cannot reach a ring.** A block's sides parallel to the ring's
axis and off it are planes `where_a_plane_and_a_torus_cross` refuses as
`ACrossingBetweenAPlaneAndATorusThatIsAQuartic`, by its own table, even where
the plane misses the tube: relay 1's probe, the block `[0, 10] × [−10, 10] ×
[−1, 5]` against the half ring, was refused that way in all six fuses. That is
Q618's refusal. So the plane through the axis is reached through a bar, which
has no flat side parallel to its axis: a first fuse cuts the bar in half with a
block, and the half bar's face `x = 0` holds both meridian circles. Its floor
is at `z = −2` because at `z = −1` it is the plane the whole ring rests on.
Every fuse is then refused `CannotFollowACrossingWhereItsSurfacesAreTangent`
(`OrderingTheCircleAPlaneRestsOnARingAlong`), ordering points round the circle
of radius 4 at `z = −1`, which lies on no face of the half ring. That is a gap
— every answer exists — and is filed as a row.

**Rows this leaves**, filed by the merge: a ball's two loops apart refused
ordering (B); the Steinmetz cross and the thin bar resting inside a thick one,
refused in `intersect` (B); Viviani's figure of eight, refused in `intersect`
(C); the half bar whose floor the whole ring rests on (C). Q1514 already holds
the half ring under a round plate.

**Where the numbers come from.** `mise exec -- just t -p rcad-topo --test
two_curves_of_one_pair_that_meet --no-fail-fast`, with each test's time read
from `target/last-test.log`. The closed forms are worked by hand in each
test's doc comment. Relay 1's quartic probe is described here and not kept.

> **2026-10-10 note (Q1603, Q1607, Q1611): the Steinmetz cross is named and
> gets its nodes, and is refused telling the cells of a pipe apart.**
> `intersect` names two pipes of one width whose axes meet as four half
> ellipses, each picked by a corner of the two flat surfaces through where the
> axes meet (`four_half_ellipses_where_two_pipes_of_one_width_meet`). It does
> so only where those surfaces pass exactly through that point, and refuses
> the rest `CannotNameWhichPieceOfACrossingThisIs`, a gap: 1192 of the 1224
> pairs of `the_namer_names_this_family_in_four_halves_and_the_walker_draws_every_piece`,
> which prints `32 of 1224 pairs named in four halves`. `place_the_meets` now
> gives two halves of one pair the two places the walls touch, `c ± r n̂`
> (`place_where_two_halves_meet`), and each half a node half way along
> (`place_the_middle_of_each_half`), so this is the second pair, after the
> apex's rulings, to get a node where two curves of one pair cross. Each half
> is ordered as a bend along the cross product of its pickers' normals, and
> leaves an end the way `the_way_a_half_ellipse_runs_at_its_end` says. Two
> things above no longer hold: the table's last cell for the two leaning
> pipes, which has the Steinmetz cross refused in `intersect`, and the
> sentence of "What it settles" that nothing hands `place_the_meets` two
> curves of one pair that meet other than the rulings at an apex. The thin
> bar resting inside a thick one is still refused in `intersect`.
>
> The cross still does not answer: all six fuses are refused
> `CannotTellTheCellsOfAFaceApart`.
> `the_one_that_goes_round_the_axis_as_well` counts how often a walk crosses
> a half-plane from a pipe's axis towards `up × across`, for `across` each of
> `±x`, `±y` and `±z`. On either pipe two of the six are along its own axis
> and give no half-plane, since `up × up` is nought, and each of the other
> four runs through `(0, ±1, 0)` or through a middle at `(±1, 0, ±1)`. A node
> on the half-plane's line, either side of the axis, leaves the side
> `how_often_an_arc_crosses` asks of it neither left nor right, so that
> half-plane gives no count. That is read off the code here, and 15b04034's
> probe saw the same. Three
> diagonal half-planes got every fuse past the arrangement (e7d4da47), to
> `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` in `assemble.rs`, where the
> two pipes' gradients are parallel at `(0, ±1, 0)` and their cross product
> is nought. They were taken out because they answered three blocks of
> `a_cone_shared_with_its_union_with_a_block_is_the_cone` wrongly where
> `main` refuses (15b04034). Both are rows. The turned-pair property skips
> the pair as a tangency a turn moves off (`is_on_a_knife_edge`): 10 of its
> 438 skipped draws are this pair, counted by Q1611 with a print in that
> branch that is not kept.
>
> The pins: `the_halves_of_the_steinmetz_cross_meet_at_two_nodes_and_leave_each_into_its_own_corner`
> in `fuse/arrange.rs` holds the two nodes, the four middles and the way a
> half leaves `(0, 1, 0)`, and fails on an assertion with `arrange.rs` as it
> was before Q1607;
> `two_bars_of_one_radius_whose_axes_meet_are_refused_telling_the_cells_of_a_pipe_apart`
> holds the refusal.

> **2026-10-10 note (Q1598): a trial that settles back to within its two
> ends' own bounds is tried again shorter, and the ball and pipe off the axis
> answer with the bar first too.** The trace in the Q1554 note holds. Measured
> in the mesh's frame for the union, bar first, first placement, with prints in
> `one_step` that are not kept: the first trial asks `4.105e153` and settles at
> `(−0.49999999999999994, 0, 2.1e-17)`, a chord of `5.948e-17` and nothing at
> all along the loop, between two ends that `how_far_off_the_crossing` puts
> within `4.59e-16` and `6.88e-16` of the crossing. That trial had not moved
> anywhere the walk could tell apart from its start, and it was kept because
> its chord was not exactly nought.
>
> `what_a_trial_of_this_length_measures` in `march.rs` now calls a trial
> settled back when its chord is nought, or when it covered less than half of
> its ask and its chord is no longer than its two ends' bounds from
> `how_far_off_the_crossing` added together. `one_step` takes the square root
> of such a trial's ask, as it already did for a chord of nought, and never
> keeps it. The bound is the walk's own enclosure of how far the settling's
> roundings may have left each end off the crossing, and it is not a tolerance
> on the answer: it decides only whether a trial is tried again shorter, and no
> point the walk hands back moves for it. A walk whose steps really are that
> short, asked and covered, never meets it, since it is asked only of a trial
> that came back short of half its ask.
>
> With the bar first, all 12 fuses of the four placements now answer, each
> validated, with the ball-first answers' faces and within the mesh's volume
> error of `ball_in_bar`:
> `a_ball_with_a_bar_through_it_off_its_axis_crosses_it_in_two_loops_and_every_fuse_with_the_bar_first_answers`
> replaces the Q1554 note's
> `a_ball_with_a_bar_through_it_off_its_axis_is_refused_meshing_every_fuse_with_the_bar_first`.
> So the ball and pipe off the axis answer every fuse both ways round, and the
> Q1554 note's sentence that the bar-first fuses are refused no longer holds.
> The march is pinned on its own in `march.rs`:
> `a_trial_that_settles_a_rounding_off_its_start_is_shortened_and_not_kept`
> walks the upper loop from its top with the coarse walk's room, and its first
> kept step lands `0.78` along the loop; the walk closes in six points, each
> known to `1.3e-15`, every step longer than twice that.
>
> How often the rule is met, over `rcad-geom`'s tests, counted by relay 1 with
> a print in `one_step` that is not kept: 2702 trials settled back. 1391 had a
> chord of nought, which were rooted before as well; the 263 of those asked 4
> or more were all in coarse walks, with `2.2e307` or `4.5e307` of room. The
> other 1311 were short trials within their ends' bounds, and the old rule kept
> 900 of them: 440 in coarse walks, and 460 in walks of `0.5` of room or less
> with each end known only to between `8e-9` and `3e-2`. Those 460 came from
> six tests:
> `a_surface_written_twice_a_few_ulps_apart_names_the_crossing_its_run_names`
> (190), `every_seed_in_the_checked_in_corpus_names_its_crossing_honestly`
> (148),
> `every_point_along_a_quartic_turned_crossing_is_on_both_the_surfaces_that_named_it`
> (51),
> `every_point_along_a_crossing_a_hair_from_tangency_is_on_both_the_surfaces_that_named_it`
> (35), `the_hair_thin_pipes_are_refused_at_the_finest_ask_too` (28) and
> `a_crossing_thinner_than_one_step_of_the_grid_crawls_to_the_point_cap` (8).
> Another 413 short trials, which the old code measured, now have an end whose
> bound refuses; that makes each a trial that could not be measured, and it is
> rooted too. A lazier rule, judging such a trial by the old rules, was tried
> and dropped: it left the thin-pipe pin below refused all the same, by the
> march's own bound on how far its points are off.
>
> Four pins outside this row's files move, and are left to a row that holds
> them. Three are in `rcad-geom`.
> `a_crossing_thinner_than_one_step_of_the_grid_crawls_to_the_point_cap`
> (`what_makes_a_huge_pipe_against_a_thin_one_slow_to_walk.rs`) was refused
> `TooManyPointsAlongOneCrossing` after a million points, and is now refused
> at once `CannotFollowACrossingWhereItsSurfacesAreTangent { at:
> BoundingHowFarOffTheCrossingAPointIs }`, the refusal
> `the_hair_thin_pipes_are_refused_at_the_finest_ask_too` already pins.
> `the_zero_on_the_pipe_tilted_by_a_hundred_and_twenty_two_orders_is_the_sizing_walk`
> and its sibling at a hundred and twenty-nine orders
> (`why_the_walked_point_oracle_checks_nothing_on_the_slow_pipes.rs`) say the
> sizing walk refuses, and it now walks: `Walked { drawn: 5, checked: 5 }`.
> The fourth is in `rcad-topo`: the ball of 4 one rounding off a bar's axis,
> `a_ball_one_rounding_off_a_bars_axis_is_refused_meshing_every_fuse` in
> `degenerate_curved_fuses.rs` and Q1599's row, now answers all six fuses both
> ways round, under a certified and a tolerant policy alike. Each answer
> validates, has faces 5, 3, 2 or 3+3, and measures within its volume error
> and 0.07% to 0.11% under the closed form of
> `what_a_ball_with_a_bar_through_its_centre_leaves`, by a probe that is not
> kept. §39.4's and §322's notes on the two old refusals say what Q1554 left.
>
> The same row, its files widened, then moved all four, each after checking
> it. The thread through the huge pipe is
> `a_crossing_thinner_than_one_step_of_the_grid_is_refused_at_its_first_step`,
> pinning the tangent refusal at `BoundingHowFarOffTheCrossingAPointIs`. That
> is a limit by §16.10 and not a gap: the loop is `4.34e-19` across where one
> step of the grid is `5.82e-11`, so no answer exists to reach, and the same
> pair walks once its thin pipe spans a grid step. The two tilted pipes are
> `the_pipe_tilted_by_a_hundred_and_twenty_two_orders_has_every_walked_point_checked`
> and its sibling at a hundred and twenty-nine orders. Each pins that both runs
> walk and every point they draw is checked against both pipes, in a box
> reaching `8.94e-8` each way round a loop of radius `1.375`. Those two and
> `the_zero_on_the_moved_hair_thin_pair_is_the_sizing_walk_refusing` leave the
> slow tier, since each now runs in hundredths of a second. The ball of 4 is
> `a_ball_one_rounding_off_a_bars_axis_answers_every_fuse_as_one_on_it_does`:
> 24 fuses off the axis, with the 24 on it beside them. Each one validates and
> has the centred ball's faces, and each measures within its volume error of
> the closed form. The union and both differences add up against the ball and
> the bar measured alone. So Q1599, that ball's row, is done by this row's
> change alone, with the trial it kept not traced. §39.4 and §322 each carry a
> Q1598 note saying so.
>
> **Where the numbers come from.** The trace: prints in `one_step`, run as
> `mise exec -- cargo nextest run -p rcad-topo --test
> two_curves_of_one_pair_that_meet` at `72cc2f6c`. The counts: an `eprintln`
> in `one_step` on each trial that settled back, and one where an end's bound
> refused, run as `mise exec -- just t -p rcad-geom --no-capture
> --no-fail-fast` and tallied. The moved pins: `mise exec -- just t
> --workspace --no-fail-fast`, which prints `3808 tests run: 3804 passed (19
> slow), 4 failed, 3 skipped`, the four being those above.

> **2026-10-23 note (Q2628): the thin bar resting inside a thick one is
> renamed, not refused before any arrangement.** The pin above,
> `a_thin_bar_resting_on_the_inside_of_a_thick_one_is_refused_before_any_arrangement`,
> no longer holds under that name. Since Q2628 `intersect` names the two
> loops where the thin bar touches the thick one from inside
> (`a_thin_bar_through_a_thick_one_crosses_it_in_two_loops_and_every_fuse_answers`'s
> loops, pushed together until they cross like an X at `(0, 2, 0)`), and
> `place_the_meets` puts no node where they meet, so the arrangement refuses
> the two loops rather than `intersect` refusing them by name. The pin is now
> `a_thin_bar_resting_on_the_inside_of_a_thick_one_is_refused_where_its_two_loops_meet`
> in `two_curves_of_one_pair_that_meet.rs`.

> **2026-10-24 note (Q2649): the pin above is renamed again, to the pair
> answering.** `place_the_meets` now puts a node where the two loops meet, so
> the pin above no longer holds under that name either. Every fuse answers,
> held to a corner at `(0, 2, 0)`:
> `a_thin_bar_resting_on_the_inside_of_a_thick_one_crosses_it_in_two_loops_that_meet_and_every_fuse_answers`
> in `two_curves_of_one_pair_that_meet.rs`.
