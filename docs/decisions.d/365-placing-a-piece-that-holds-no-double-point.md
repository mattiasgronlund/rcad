## 365. Placing a piece that holds no double point — 2026-09-29

*Queue row Q661. Lane `lane-q661-placing-a-piece-with-no-f64-point`, off
`6273ce5e`. §346.3 left the grazing block of `a_block_edge_grazing_a_ball.rs`
to one of two families: 5, a point that is the sum of two doubles, or 6,
placing a piece without a point. This weighs the two and takes 6, in one
narrow form: **a piece that shares a side with a piece of another solid's face
is on the side of that face its own `material_side` says** (365.3). It is tried
only where the point search found no point inside the piece, so no fuse that
answers today can answer differently. What each family costs was read from the
code. Which pieces the other refusals stop on was run, for four of them, with a
throwaway probe that is taken out (365.2); everything else in 365.2 was read.*

  365.1   What each family has to change
  365.2   Which other refusals each one reaches
  365.3   The choice: a piece placed from a side it shares with another solid's face
  365.4   What this leaves
  365.5   What building the rule ran

### 365.1 What each family has to change

**Family 5, a point that is two doubles per coordinate.** Read, not built.

- `rcad_predicates::Point` (`point.rs`) gains a variant. Its own matches in
  `point.rs`, `planes.rs`, `real_numbers.rs`, `provenance.rs` and `orient3d.rs`
  take an arm. Outside the crate, 73 lines in 19 `src` files name
  `Point::Exact` or `Point::Constructed`, `arrange.rs` (19) and `select.rs` (9)
  the most; every `match` among them has a last arm, because the enum is
  `#[non_exhaustive]`, and that arm is where the new variant would land unread.
- **Eight exact levels start with `exact_coordinates()?`**, so each needs a
  second exact path: `orient3d_exactly` / `exact_determinant` (`orient3d.rs`,
  whose twelve terms assume each coordinate is one double), and the `exactly`
  of the side of a plane (`plane.rs`), a ball (`sphere.rs`), a pipe
  (`cylinder.rs`), a cone (`cone.rs`), a ring (`torus.rs`, the quartic), and
  the predicates in `along_a_crossing.rs` and `walk.rs`. `holds.rs` and
  `validate.rs` implement `Predicate` too and were not read.
- **Exact points are not enough for a curved piece.** `select` builds a curved
  piece's candidates as boxes: `back_onto_the_surface` and
  `segments_up_through_a_cap` in `select.rs`. The ball's sliver is `2⁻¹⁰³`
  across in `x` at `x = −1`, and a box with `f64` ends is at least one spacing,
  `2⁻⁵³`, wide there. So family 5 also needs boxes whose ends are two doubles,
  and a level 2 over them in every predicate: interval arithmetic that is not
  `Interval`.
- **The ray needs it too.** `count_where_the_ray_meets_it` in `classify.rs`
  starts from `point.enclosure()`, which rounds to `f64`, and hands that to
  `where_a_line_meets_a_surface` (rcad-geom), `where_it_lands`, `gradient_at`
  and `dot_enclosing`. For the block's piece `x = −1` inside the ball, a
  two-double point such as `(−1, 2⁻²⁵ + 2⁻⁷⁹, 0)`, which is inside the ball,
  rounds to a box whose far corner `(−1, 2⁻²⁵ + 2⁻⁷⁷, 0)` is the double that
  `no_point_with_f64_coordinates_is_inside_the_piece_between_the_two_places`
  shows is outside. That box straddles the ball, so the root next to the start
  is not known to be ahead or behind, whichever target is aimed at (read, not
  run). So the curved route of the ray, and `where_a_line_meets_a_surface`
  under it, need two-double boxes as well. `what_a_point_beside_it_says`
  builds an `f64` point beside the box, and no such point is in the piece.
- `select.rs` then builds two-double candidates, and `the_piece_holds`,
  `side_of_a_plane`, `stretched_to_the_plane`, `InPlane::winding_around`,
  `a_face_of_the_input_holds` (through `face_holds_the_point` in `holds.rs`)
  and `the_ray_and_the_stored_planes_disagree_about` all read them.

That is the numeric layer built a second time, which §1.6 says is "a separate
build of the numeric layer, not a type parameter dragged through topology".
A `Point` variant is not a type parameter, but it is dragged through topology
all the same: `classify.rs`, `holds.rs` and `select.rs` read it.

**Family 6, placing a piece without a point.** Read, not built. What the
grazing block needs is one rule, 365.3's:

- **A way to find, for a side of a piece, the pieces of other faces that have
  the same side.** The same two surfaces, the same two ends, and the same arc
  where two arcs join those ends. `assemble.rs` already names both across
  faces, for the pieces it keeps: `Corners::of` names an end by its
  `MeetPoint` (three surfaces in a fixed order and `which`), and `Edges::of`
  names a side by its ends, its two surfaces and `WhichArc`
  (`which_arc_this_is`). Family 6 needs that naming over every piece before
  `select::choose`, not only over the kept ones after it.
- **Which solid's face each piece came from is already recorded.**
  `Scratch::cut_from` names the solid and `Piece::ancestors` the faces. So
  "which solid's face a side lies on" is read off the piece across the side,
  and `arrange.rs` and `cuts.rs` need no new field. §346.3 said the
  arrangement would have to say it, which was true of reading the side alone:
  `BakedFace::cuts` is a list of surfaces with no owner, deduplicated by their
  bits in `canonical_cut_surfaces`.
- **`place_the_piece` in `select.rs` gets one more try** before it gives up
  (365.3), and the module doc's "Classifying a piece is classifying one point
  inside it" gains its exception. No predicate changes and no number is built.

### 365.2 Which other refusals each one reaches

A throwaway test fused each shape under every rule, both ways round, certified,
and printed the corners the refusal record names (`mise exec -- just t -p
rcad-topo --test q661_probe --no-capture`, the file `q661_probe.rs` taken
out). The point named inside each piece was worked by hand from those
corners, not classified.

| row | shape as run | refused with, all six fuses | the piece's corners | holds an `f64` point? |
|---|---|---|---|---|
| Q661 | the grazing block at `h = 0` | `a point inside a piece of a face` | `(−1, 2⁻²⁵, ±2⁻⁵¹)` | **no** (§346.3, a test) |
| Q635 | the pinned cone and block | the same | `(−12, ±13.416…, 18)` | yes, run |
| Q637 | bar of radius 10 at `(−6, 0, 0)`, height 4, and bar of radius 5 at `(−3, 0, −1)`, height 6 | the same | `(4, 0, 0)`, `(−3, −5, 0)` | yes, run |
| Q602 | bar of radius 39, height 4, and bar of radius 25 at `z = −1`, height 6 | the same | `(39, 0, 0)`, `(0, −25, 0)` | yes, run |
| Q621 | the tube of `a_sliver_arc_on_a_joined_flat_face.rs` and a bar of radius 3.6 at `(−3.5, 0)`, height 4 | the same | `(−6.8986, ±1.1873, 0)` | yes, run |

- **Q635.** The corners are where the wall `x = −12` meets the top `z = 18` on
  the cone. Three pieces have exactly those two corners, and each holds an
  exact double: `(−12, 0, 15)` on the wall inside the cone, `(0, 0, 18)` on the
  top inside the cone, `(0, 16, 16)` on the cone inside the block.
- **Q637.** Every rule and both orders refuse, where the row had run only
  Difference. The piece is the plate's bottom cap, the ring between the circle
  through `(4, 0, 0)` and the hole's circle through `(−3, −5, 0)`, and
  `(−14, 0, 0)` is inside it.
- **Q602.** Reproduced with those heights, which the row did not know. The
  piece is the same form as Q637's: a flat cap bounded by one circle with one
  corner, with a hole that is one circle with one corner. `(30, 0, 0)` is inside
  it. The tube of radius 7 less 1 that Q621 builds answers.
- **Q621.** Reproduced without waiting on Q602. The piece is the lens of the
  bar's cap outside the tube, `0.1` deep and `2.37` wide; `(−7.05, 0, 0)` is
  inside both the bar and the lens, to the nearest double.

**So family 5 reaches none of these four.** Each piece already holds a point
with `f64` coordinates, and each is a gap in `select`'s search for one, which
is family 1 of §346.3 and not a new number type.

The rest were read, not run:

- **Q398** is dropped, and Q658, which replaced it, showed its caps hold many
  `f64` points (§346.6). Ten of its draws now refuse `a point in a piece that
  every solid could place` because the block's ray reads its face by corner
  triangles and not by the stored plane (Q668). 365.3's rule reads the stored
  plane, through `material_side`, and never fires the ray, so it would reach
  them if it were tried after that refusal too. It is not, yet (365.4).
- **Q609** stops on a piece the arrangement may have cut wrongly (its row's
  hypothesis about `bake_one_face` and a reversed pipe). Neither family is the
  fix. A rule that places pieces by their sides would place a wrongly cut
  piece too, which is why 365.3 runs only after the point search has failed and
  never instead of it.
- **Q603**'s predicate and heights were never recorded, and **Q464**'s 56
  fuses need their sweep rebuilt. Neither was run here, so neither is placed in
  the table. Q464's own guess, §350.5's, is candidates stepping past the apex,
  which is a search gap and not a piece without a double.
- **365.3's rule would reach** Q635, Q637 and Q602 on paper: each piece has a
  side that is a cut of the other solid's face (the hyperbola or circle on the
  cone, the hole's circle on the bar's wall). Q621's lens does not: its side
  on the tube's circle lies along the tube's bottom rim, an edge of the tube,
  where the tube near the side is a wedge and not one face, and the tube's cap
  beside it is on the lens's own plane. None of that was run.

### 365.3 The choice: a piece placed from a side it shares with another solid's face

**Family 6, as one rule:** a piece `P` of one solid, and a solid `S` it must be
placed against. If a side `s` of `P` is also a side of a piece `Q` cut from a
face `G` of `S`, and

- `s` is a cut of `G`, not along any of `G`'s own input edges (`Q`'s side has
  an empty `along`),
- `P`'s side `s` names `G`'s surface as `carried_by[1]`, bit for bit, and
- `P` is not on `G`'s surface (that is the ON case, which `coincident_face`
  already answers),

then `P` is inside `S` when its `material_side` on `s` is `S`'s inner side of
`G` (read from `G`'s orientation), and outside it otherwise.

**Why it holds.** `P` was cut by every surface of `S`, so its inside crosses no
face of `S`, and it is all inside `S` or all outside. That is the premise one
point already rests on (§17.4). Next to a point in the middle of `s`, `S`'s
skin is `G` alone: `s` lies inside `G`, not on its edges, and a sound solid is
one face thick there. `P` crosses `G`'s surface along `s` and lies on the side
`material_side` names, which the walk that made `s` decided for free (§36.6).
So next to `s`, `P` is on `S`'s side of `G` or off it, and so is all of `P`.
Nothing here is a tolerance, and every sign it reads was decided when the
arrangement was built.

**It places all three slivers of the grazing block**, read and not run. The
ball's piece shares its arc on `x = −1` with the block's piece on that face,
which is a cut of it, and lies on `x < −1`, the block's inside, so it is
inside the block. The block's pieces on `x = −1` and `y = 2⁻²⁵` each share
their arc with the ball's piece, the ball's one face covers its whole surface,
and each lies inside the ball, so each is inside the ball.

**Why 6 over 5:**

- **6 builds no numbers.** 5 is §1.6's second numeric layer, in ten predicates,
  in `Interval`, and in the ray, for one known shape.
- **5 reaches nothing else that is open** (365.2); 6's rule reaches Q635, Q637
  and Q602 on paper, and Q668's draws if it is also tried after the second
  refusal. Those pieces all hold doubles, so each of their rows should still
  mend the search that missed them: this is a floor under them, not their fix.
- **5 moves the limit rather than removing it.** A piece narrower than two
  doubles' spacing, away from any double, is out of its reach as this piece is
  out of `f64`'s. Nobody built one, so that is a guess with no class
  (§360.5). 6's rule does not look at how wide the piece is at all.
- **6 is where §17.1 already points.** The arrangement cut `P` against `G`'s
  stored surface, and the rule reads that same sign back. The ray reads a flat
  face by its corner triangles, which is the disagreement §346.6 drops points
  over.

**Why only this rule, and only after the point search.** Two more forms of 6
were weighed and left: a piece taking its neighbour's placement across a side
no surface of `S` carries (a flood through the solid's own edges), and a piece
whose corner sits on a convex edge of `S` with a side along each of its two
faces. The grazing block needs neither, since the rule above places all three
slivers, so §13 leaves both until a piece that the rule cannot reach asks for
one; Q621's lens may be that piece. And **the rule runs only where no point
inside the piece was found** (the `INSIDE_A_PIECE` exit of `place_the_piece`).
A fuse that answers today reached a point for every piece, never gets to the
rule, and answers exactly as before; only refusals under that name can move.
The point search also stays a check on the arrangement (Q609 above), which a
rule tried first would silence.

**What it does not settle.** Past `select`, the grazing block's answer has a
face with a hole `2⁻⁵⁰` long and `2⁻⁷⁸` wide, and whether assembling and
meshing that answers or refuses was not run. §346.2's ball `1e-14` across got
past `select` and was refused by its own mesh check, so the row that builds
this pins whatever the six fuses give, without assuming they answer.

### 365.4 What this leaves

- **The row that builds 365.3's rule** (the lane's report carries it): the
  naming of sides across every piece, the rule in `place_the_piece`, and pins
  on what the grazing block's fuses give, both in
  `a_block_edge_grazing_a_ball.rs` and with the block first in
  `two_meets_reach_one_corner.rs`, with every other pin naming
  `a point inside a piece of a face` that moves.
- **Trying the rule after `a point in a piece that every solid could place` as
  well** is Q668's to decide; its ten draws are the first consumer.
- **Q637 and Q602 look like one stop**: a flat cap with one-corner circles for
  its outline and its hole, all six fuses, both reproduced above. Q637's row
  asked for that to be said there.
- **Q621 reproduces now**, without Q602, with the corners above.

### 365.5 What building the rule ran

*Queue row Q712. Lane `lane-q712-a-piece-placed-by-a-side-it-shares`, off
`9599c4d0`. Everything here was run, with a throwaway probe test that is taken
out, unless it says otherwise.*

**Built as 365.3 says, in one place.** `SidesItShares` in `fuse/select.rs` is
tried only where `place_the_piece` found no point inside the piece, which now
answers `None` there instead of refusing. Which sides are one is
`the_edge_each_side_of_every_piece_is` in `fuse/assemble.rs`: the kept
pieces' own naming (`Corners::with_each_closed_curve_started_once`,
`Corners::of`, `Edges::of`, so `which_arc_this_is`), run over every piece,
once per fuse, and only for a fuse that reaches the rule. Inside one solid of
the group is inside, outside every one is outside, and anything else leaves the
refusal as it was. A naming that gives up, a plane agreement that cannot be
told, or two shared sides that disagree leave the piece unplaced, so the rule
adds no refusal. What the naming asked the policy goes into the log only when a
piece is placed by it.

**One assumption was wrong and cost a panic.** `Edges::of` asserts a straight
side has two different corners, and that holds for the kept pieces. Over every
piece it does not: `Corners::of` makes two corners of two names on one `f64`
triple one corner, and a piece no rule keeps can have a straight side between
two such. A seed of the fuse fuzzer's corpus and the draw of
`a_fused_draw_hands_its_provenance_back_through_the_door.rs` panicked there.
The naming now gives up at such a side, before `Edges::of` is asked.

**Before the rule**, on `9599c4d0`, all thirty fuses below (three rules, both
policies) refused `a point inside a piece of a face` on `(−1, 2⁻²⁵, ±2⁻⁵¹)`.

**The grazing block at `h = 0`, ball first and block first.** All three
slivers are placed, in all six fuses and under both policies: the ball's
sliver inside the block, the block's slivers on `x = −1` and `y = 2⁻²⁵` inside
the ball. A temporary print of each placement showed it, and
`the_three_slivers_of_a_block_edge_grazing_a_ball_are_placed_by_the_sides_they_share`
in `select.rs` pins which pieces each rule then keeps. None of the six
answers, so there is no volume to check:

| fuse | refused with |
|---|---|
| union, either order | `Geometry(CannotNameWhichPieceOfACrossingThisIs)` |
| ball less block | `Geometry(CannotNameWhichPieceOfACrossingThisIs)` |
| intersection, either order | `ACurvedFaceGoesRightRoundWithNoSeam` |
| block less ball | `ACurvedFaceGoesRightRoundWithNoSeam` |

Each logs two `side of a half-space` records, under either policy; neither
was traced. The first refusal is raised only in `rcad-geom`'s `walk.rs`, and
it is the two answers whose ball face keeps the hole 365.3 named that meet it;
that the hole is why was not traced. The second is raised only in `mesh.rs`,
as §346.2's small ball's was. Which of them are limits is not settled here.

**The block-first corner forms of `two_meets_reach_one_corner.rs`.** With the
block above the top place, the ball's sliver is still refused: it lies below
the block's floor and touches the block only at the corner, so no side of it
is a side of a piece of the block's faces. This is the first piece the rule
cannot reach, and it is the shape of the second form 365.3 left (a corner on
the solid). With the block reaching the other place, the slivers are placed and
the refusal moves to the ball's lens beyond `x = −1` on the far side of the
block's roof or floor, between `(−1, ±2⁻²⁵, ±2⁻⁵¹)`: both its corners are on
the plane of that face, so every candidate blended from them is on that plane
and none is in the lens. Whether the lens holds a double point was read, not
run.

**Pins that moved.** In the row's files: `a_block_edge_grazing_a_ball.rs`,
`two_meets_reach_one_corner.rs` (above), and
`a_plane_cut_along_a_cones_other_nappe.rs`, whose quarter-lean family went from
`right: 32` and sixteen `a point inside a piece of a face` to `right: 48`, held
by that file's own checks against the shapes' volume. Also moved, and not
repinned by that run (the second run, below, did):

- `a_joined_flat_face_whose_rings_enclose_no_area.rs`,
  `a_cone_cut_by_a_block_wall_off_its_axis_is_refused_while_a_piece_is_placed`:
  the union, not swapped, answers one solid. Its volume was not checked.
- Outside the row's files, so not touched:
  `a_block_edge_grazing_a_ball_at_cx_0.rs` (the first case it checks, the
  union, ball first, certified, is now
  `CannotNameWhichPieceOfACrossingThisIs` where `a point inside a piece of a
  face` was pinned; the rest were not reached);
  `a_broken_two_surface_corner_read_back_from_bytes.rs` (the frustum with its
  corner off the cap, less the block over its top, now gives the sound
  frustum's answer, and the intersection blames itself
  `VertexOffItsFacePlane { distance: 1.0 }`; the sound frustum less the block
  over its rim by the seam now answers one solid, unchecked); and
  `a_mesh_that_folds_at_one_deflection_and_not_the_next.rs` (four of ten
  `Undecidable` rungs now answer and mesh, 278 to 282 answered). Each is its
  own row, with its volume to check. The gate was not green, so `test-slow`
  was not run.

**The second run** (lane `lane-q712-a-piece-placed-by-a-side-it-shares-2`)
merged the first run's branch at `30f8b67c` onto `45593a1c` and ran the whole
suite there before changing anything: 3041 run, 4 failed, the four pins above
and no other. Each was repinned from a run under both policies where the file
has two, and each new answer checked against a volume worked out apart from
the fuse. To tell what the rule moved from what `main` moved, the same probe
was run again with the rule switched off by hand: every case below refused `a
point inside a piece of a face` that way, so the rule is what moved each one.

- **The cone and the block wall** (Q635's shape): all six fuses answer, not
  only the union. Each validates and measures what the cone up to `z = 18`,
  `1944π`, less the part past the wall, `1944 acos(2/3) − 144√180 + 576
  ln((18 + √180)/12)`, gives; the intersection measures `5849.749 ± 1.281`
  against `5849.825`. Q635's search for a point in that piece is still
  unmended. The rule just answers first.
- **The edge along `x` at `cx = 0`**: two slivers, not three. The ball's skin
  past `y = 1` is outside the block, and the block's side is inside the ball. A
  print of each placement showed both placed by their sides in all twelve
  fuses, and the right way round. All twelve then refuse
  `CannotNameWhichPieceOfACrossingThisIs`, each with one `side of a
  half-space` record. That includes the intersection and the block less the
  ball, which along `z` refuse `ACurvedFaceGoesRightRoundWithNoSeam`.
- **The frustum read back from bytes**, under both policies: the corner off
  the cap less the block over the top gives the sound answer, and the two
  intersected blame themselves with `VertexOffItsFacePlane { distance: 1.0 }`.
  Against the block over the rim by the seam, the sound frustum and every
  moved corner give the sound answer. The sound frustum's two answers measure
  `39π/4` and `52π/3 − 1.98370` within their errors, held by a new test.
- **The chopped slivers**: 282 answer and 6 refuse, where 278 answered and 10
  refused. The four the rule answers are at lean `0.9`, chopped at `0.3` of
  the rim, turned 2, 4, 8 and 10 twelfths. A Simpson sum over slices matched
  all 282 answers, measured at `0.001`, within their declared error, and a
  new test holds the four to it. Four more, at lean `0.9` turned 2, 4 and 10
  twelfths and at `1.2` turned 4, all chopped at `0.5`, now refuse `side of a
  half-space`. That was not traced.

`just pre-gate gate-lane` then passed with 3003 tests run, and `just
test-slow` with 40 run and 40 passed.
