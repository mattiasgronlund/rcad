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
- **A pin outside the write set moved.** In
  `a_frustum_cut_in_half_down_its_axis.rs`,
  `a_block_whose_floor_runs_through_the_apex_is_refused_there` pins all six
  refused `Undecidable` on "which way a crossing runs". That was the direction
  read at the apex, which part 2 removes. Now the intersection, either order,
  and the frustum less the block each answer one solid that validates and
  measures `58.6392 ± 0.0280` against `56π/3 ≈ 58.6431`. The union, either
  order, and the block less the frustum refuse `ResultIsNotManifold { count: 1 }`.
  A likely cause, **not shown**: the apex is on the block's bottom edge, and
  the block's bottom face `z = 0` touches the cone at the apex alone, which
  `intersect` names as no curve. So that face gets no corner at the apex while
  the face `x = 0` does. The module comment there and the test's name and
  message need rewriting to match. That file is not this row's.
- **One ruling through an apex with no node there.** A face whose edge runs
  along one ruling from one nappe to the other through the apex has no second
  ruling to cross it, so it gets no node at the apex. Its arc across the apex
  has the partner's inside on both sides, and `runs_against_its_gradients`
  refuses it `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`. No test builds
  one.
- **A plane resting along a cone**, one ruling touched all along, is left as
  it was: `∇S × ∇Q` is nothing along the whole line, it carries no picker to
  read a line off, and `how_a_crossing_runs` still refuses it.

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
The plane-and-torus row refuses the two bitangent planes' Villarceau circles,
which do cross, as a quartic. The rest are two curves apart by what the pair
is. **None of that was tried here** by building two curves of one pair that
meet and running an arrangement on them, so the claim is "not reached by any
shape this row built", not "cannot be reached".

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
- The frustum's floor in 347.4: the same scratch test, with the frustum
  `cones::a_frustum(model, 0.5, 4.0, 8.0)` and the block
  `[−10, 0] × [−10, 10] × [0, 12]`, run with `--no-capture`.
- What each part is for, 347.2: temporary `eprintln!`s in `arrange.rs`, run
  through the same file and through `a_frustum_cut_in_half_down_its_axis`,
  all removed.
- The suite: `mise exec -- just t --workspace --no-fail-fast` on the branch's
  last code, `2695 tests run: 2694 passed (1 slow), 1 failed, 11 skipped`. The
  one is the frustum floor of 347.4. `mise exec -- just gate quiet` fails on
  that same test and on nothing before it.
- By hand: `64π/3 = 67.0206`, `511π/24 = 66.8894`, `513π/24 = 67.1512`,
  `648 − 511π/24 = 581.1106`, `56π/3 = 58.6431` (the last from `python3 -c`).
