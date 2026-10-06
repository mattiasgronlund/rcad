## 406. A block leaned along a cone ruling — 2026-10-05

*Q902, lane `lane-q902-a-block-leaned-along-a-ruling` off `db185a87`, after
Q866. It carries on from Q696's trace for §340.4, and its three stages are the
row's, measured again on this base.*

**What refused, measured first.**
`a_block_below_a_frustum_is_cut_where_the_frustum_has_no_face_and_every_answer_is_right`
in `a_plane_cut_along_a_cones_other_nappe.rs` was right 72, `a point inside a
piece of a face` 16 and `order along a crossing` 8 on the base, as Q866's lane
said; Q866 moved nothing there. The eight are every rule of two blocks: leaned
`2`, turned `1`, from `−3` up to `1` and up to `1.5`. The block is turned by
`atan 2` about `y`, which leaves its top and bottom faces, and the side edges
along its local `x`, parallel to a ruling of the frustum's cone (slope one
half) up to how the turn rounded. So the top and the bottom cut the cone in
ellipses `4.5e16` long, and each side edge meets the cone a second time far out.

The three rules with the cone first refused on the cone's face, round the
ellipse the top cuts, between two meets `3.9` apart by the frustum, swept from
where the ellipse starts, `2.2e16` out: enclosure `1.87e17` for the block up to
`1`, `2.63e17` up to `1.5`. The block less the cone refused on the bottom face,
round the ellipse the bottom cuts, between the two meets of its side edges with
the cone `2.26e16` out: boxes `11`, `16` and `32` wide on the three axes, `~35`
apart, enclosure `8.0e17`. `how_often_a_certified_fuse_of_a_skew_pair_refuses`
was 441 fuses, 39 refused, every one `a point inside a piece of a face`.

### 406.1 Swept from the start, then from `one`

`HowThreeOnACurveWind::on_intervals` sweeps `(one − from) × (other − from)`.
Each step from a start `2.2e16` out rounds by an ulp of `2.2e16`, and that
rounding is multiplied by the other long step, so two points `3.9` apart
straddle nothing. `(one − from) × (other − one)` is the same number for every
three points, since the two differ by `(one − from) × (one − from)`; so both
enclosures hold every value the boxes allow, and so does their overlap, which
is what it answers now. The second subtracts two boxes close together and
rounds by an ulp of the short step. A **gap** (§16.10): the sign exists, and
the overlap proves it.
`two_points_close_together_far_from_the_start_are_ordered_round_a_long_ellipse`
pins it on the stored numbers, against the exact sign at the two boxes'
middles. Every one of the three then refused one stage on, on the same face
and ellipse, between two meets `4.48e16` out whose boxes overlap.

### 406.2 A node where nothing drawn crosses is left out of the order

The two meets `4.48e16` out are where the top's ellipse meets two side walls
whose own crossings with the cone `work_out_how_each_crossing_runs` dropped,
every node on them being clear of the box round the face. So nothing drawn
crosses the ellipse there. `the_nodes_on_each_crossing` leaves such a node out
of the crossing's list, which both the fold and the cut into arcs read, unless
every node on the crossing is one.

**Why no piece changes.** The node is no corner (`vertex` is `None`) and was
placed on a crossing since dropped, so its box is clear of the box round the
face, and the face is inside that box (`bake.rs`'s `Bounds`, grown by §340.10).
No drawn curve crosses the crossing between two neighbouring nodes, so the
arcs either side of it cannot reach the face's boundary to get in, and the
cells either side of them are outside the face, with winding nothing. The node
has two ways out, both along the one crossing. Left out, the drawing is the
same graph with one bend smoothed away: the same cells, the same windings, and
no piece of the face is bounded through it. A crossing whose every node is
such a one keeps them all, because with none it would draw no arc, and a
closed crossing drawn in full is read for which side of it a walk is on.

That rule left the skew-pair pin as it was, 441 fuses and 39 refused, every
one `a point inside a piece of a face`. Q696's probe of the rule, on an older
base and without the last clause, moved it to 21 `order along a crossing` and
21 `which side of a plane`; neither reproduces here, so that pin is not moved
and the rule is not narrowed for it. The eight then all refused on the bottom
face, 406.0's last case.

### 406.3 A far meet boxed to an ulp, and the turn read off the root

**The order.** The two meets of the bottom's side edges with the cone are
where a line a rounding off parallel to a ruling meets the cone again, at a
graze: the cone's gradient there is `4e16` long and its dot with the line is
a few units. `MeetOfThree::boxed_tight` takes one checked step onto the cone,
whose row is that gradient worked out over the whole box; over a box `30` wide
the determinant straddles nothing and the step gives up. Along the line the
root is simple. Where the line's direction `n₁ × n₂` has a part `E` along an
axis, its point with coordinate `s` there is `N(s)/E`, and `E² Q(N(s)/E)` is a
polynomial of degree two at most in `s` whose every coefficient is exact in
the stored numbers. Opposite strict signs at the two ends of the box's side
mean one root between them, which is the meet's coordinate; halving keeps it
between two `f64` where the signs still differ. The meet is then boxed
between two neighbouring `f64` on each axis, and the sweep round the bottom's
ellipse decides it.
`the_far_meet_of_an_edge_leaned_along_a_ruling_is_boxed_between_two_neighbouring_f64`
pins it on the stored numbers, with the checked step shown to leave the box as
it was.

**The turn.** The next refusal was `orient3d` at the same meet: which way the
side edge and the ellipse leave it, `det(∇S, ∇Q₁, ∇Q₂)` for the bottom's plane,
the side wall and the cone. It is the gradient's dot with the edge, a few
units out of terms near `1e16`, and over the corner's box it straddles nothing
by `104`, over a box an ulp wide by `17`: no box at that size settles it, since
a product of a coordinate near `1e16` rounds by an ulp of it. The same two
exact signs do. `E² Q(N(s)/E)` climbs through the root exactly when it is
above nothing at the high end, and its slope there is `E · ∇Q · (n₁ × n₂)`, so
the sign of `∇Q · (n₁ × n₂)` is the sign at the high end times the sign of
`E`. That is the determinant up to which row the curved surface is in
(`the_turn_where_two_planes_cross_a_quadric`), and `gradient_at` is half the
gradient of the expression `three_surfaces.rs` writes for a ball, a pipe and a
cone. `a_line_runs_into_a_ball_where_it_meets_it_first_and_out_where_it_meets_it_last`
and `the_sign_along_two_planes_line_is_inside_on_or_outside_the_ball_where_it_is`
pin the two signs on a ball where nothing rounds. It is
`MeetOfThree::which_way_a_line_runs_through_a_quadric` so as not to widen
`rcad-geom`'s list of exports in `lib.rs`.

**Only at a node that stands for one meet.** A tolerant policy may fold two
meets into one node, which keeps one of their boxes; the sign proved there is
that meet's, and not the fold's. Asked at every node,
`a_tolerant_policy_asked_about_the_same_two_meets_folds_them` in
`a_block_edge_grazing_a_ball.rs` failed: its two meets `2⁻⁵¹` apart are folded
under `Policy::tolerant(25.0)`, the turn there was read off one of them, and
the fuse refused `CannotTellTheCellsOfAFaceApart` instead of `Undecidable`. So
the route is asked only where the node's `also` is empty, and under a
certified policy no two meets are ever folded.

**And only on a flat face, with the curved surface a partner.** Asked on a
curved face too, it decides the turn §317 found undecided at the ball of
`a_block_edge_grazing_a_ball_at_cx_16.rs`, the gap that file's doc names: the
place below `16` turns as minus its gradient's `x`. Every certified fuse there
then refuses one step on, `order along a crossing` between the two places
along the block's edge, whose boxes `[15.999999999999998, 16]` and
`[16, 16.000000000000004]` touch at `16` (enclosure `5.3e-15`). That file pins
the turn and is not this lane's, so the route stopped at flat faces, and 406.5
left the rest as a row. Q914 has since dropped the stop and ordered the two
places by the quadric at the end they share; §317.7 says why both are sound.

### 406.4 What the pin says now

None of the eight stops at the order. The block up to `1.5` answers right for
the intersection and for the block less the cone, each held to the file's four
checks: it validates, it is one solid, its volume measured at `0.002` is within
the bound of the closed forms' figure, and each decides 30172 grid points and
holds every one. The other six refuse inside the fuse, meshing what it built:
the block up to `1.5`'s union and cone less the block
`AWireWillNotLieFlatOnItsFace`, the block up to `1`'s union and cone less the
block `TwoPatchesOfTheMeshDrewTheSameSide`, and its intersection and block less
the cone `CannotMeshAFace` with `BoundaryCrossesItself`. Printed for the block
up to `1`'s intersection, every piece of every face cut lies within `8` of the
origin, so the far meets are in no piece; what the mesher refuses is not
traced. The pin is right 74, `a point inside a piece of a face` 16 and two of
each of the three. One run of it took 25.6 s on the lane's commit against
21.4 s on the base; two runs in between, with the machine busier, took 59 s and
48 s. Q1074 has since traced and closed the six, and the pin is right 80
(§406.5).

### 406.5 What this leaves

- **The six refusals in meshing what the fuse built**, by name above, were
  traced by Q1058 and closed by Q1074, and all six answer right. They shared
  one cause: `rcad_geom::walk` placed the points of an arc of the top's
  ellipse by the frustum by `Conic::at`, from a centre `2.2e16` out, so the
  arc's one middle point rounded to `(−3, 0, 8)`, `(0, 0, 4)` or `(−2, 0, 8)`,
  one or two off the cone. A **gap** (§16.10): the ruling at the arc's middle
  angle, cut by the plane, gives a point on both surfaces to a rounding. Where
  the ellipse reaches more than `PointsAlongACrossing::WALKED_ROUND_THE_CONE_PAST`
  (a million) times further up the axis than a run's ends, the arc that misses
  the ellipse's far end is placed along the rulings, picked at its own middle,
  charged by its own size and counted by a bound on its own bend
  (`AnArcRoundACone`); and `mesh.rs` charges such an edge's floor by its ends
  past twice that line, where it charged the ellipse's far end, `2.2e4` as a
  floor. `an_arc_of_a_plane_cut_nearly_along_a_cones_ruling.rs` pins the arc
  on the fuse's own plane, and the pin is right 80 and `a point inside a piece
  of a face` 16.
- **An edge whose half-spaces pick the arc past the far end** is charged its
  ends in `mesh.rs` and walked round the ellipse in `walk`. Unless another edge
  of its face reaches as far, the walk is then handed a deflection under what
  it can place a point to: refused under a certified policy, and waved through
  with a `room left to refine a walk` record under a tolerant one. No shape
  that reaches it has been built; a face on a pointed cone whose only curved
  edge runs round the far side from two ends a millionth of the way up is the
  one to try.
  *2026-10-06, Q1082 (`593ecb26`): the shape is built, the sliver in
  `a_cone_arc_past_the_far_end_of_a_long_ellipse.rs`. It is also closed:
  `mesh.rs` now asks
  `PointsAlongACrossing::are_placed_round_the_cone_from_the_ends_of` which arc
  an edge's walk will place before charging the edge by its ends, rather than
  always by the ends, and the sliver is refused
  `DeflectionBelowWhatAMeshCanReach` at a floor of `6.1e-4` under every
  policy asked. Two rows are left: the tip's
  `TooManyPointsAlongOneCrossing { most: 1048576 }` at the floor the mesher
  picks for a tolerant caller, `9.1e-11` (Q1186), and the sliver's fold,
  `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }` (Q1187).*
- **The turn read off the root on a curved face** was Q914's, and §317.7
  closed it: the `curved == 0` stop in `the_turn_where_two_planes_cross_a_quadric`
  is gone, the two places along the edge, `16 ∓ 2⁻⁵¹`, are ordered, and the
  pin is now `every_fuse_refuses_on_a_point_inside_a_piece_between_the_two_places`.
- §340.4's "four refuse `ResultIsNotManifold`" is out of date since §340.10
  and this; it is §340's to say, and its owner's.
