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
  365.6   Rungs nearer each crossing, after every other point
  365.7   A piece placed by a corner, and a strip whose corners round to two
  365.8   A plane places a piece with corners on it
  365.9   A piece placed by the piece of its own face across a side

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

> **2026-10-05 note (Q998): `the_ray_and_the_stored_planes_disagree_about` is
> gone.** Since Q996 dropped `select.rs`'s own copy of the guard, this list's
> last name no longer exists there; `select.rs` reads classify.rs's copies
> instead, as `stretched_to_the_plane`, `an_end_on_its_side` and
> `side_of_a_plane` now do.

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

  **2026-10-05 note (Q905): answered since Q714.** `between_the_round_wires`
  in `fuse/select.rs` now offers points on the line through the two circles'
  own middles, and all six fuses answer, pinned in
  `a_plate_with_an_off_centre_hole.rs` against `300π`. The piece held an
  `f64` point all along; this was a gap in the search, not a reach for
  365.3's rule.
- **Q602.** Reproduced with those heights, which the row did not know. The
  piece is the same form as Q637's: a flat cap bounded by one circle with one
  corner, with a hole that is one circle with one corner. `(30, 0, 0)` is inside
  it. The tube of radius 7 less 1 that Q621 builds answers.

  **2026-10-05 note (Q905): answered since Q714 too**, by the same search
  fix, run in a deleted scratch test rather than pinned in a tracked file.
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
  **Q603's lane, and Q897 after it, ran it.** The tube is a bar of radius 3,
  height 4, less a bar of radius 1 from `z = −1` to `5`; the second bar has
  radius 1.5 and stands at `(2, 0)`, from `z = 0` to `4`, `−1` to `4` or `−1`
  to `3`. All 18 fuses, every rule both orders, refused `a point inside a
  piece of a face` (Q897 re-ran them on `dbf8eaa8`). The piece is a cap of the
  tube less the bar's disk, a C: the bottom cap for the flush bar, the top cap
  for the other two, with corners `(2.6875, ±1.3332)`, `(0.6875, ±0.7262)` and
  the hole's corner `(0, ∓1)`. The rungs of 365.6 answer all 18.
- **365.3's rule would reach Q635** on paper: its piece has a side that is a
  cut of the other solid's face, the hyperbola on the cone. Q637 and Q602 no
  longer need it, now that Q714 answers both through the point search
  instead (above). Q621's lens is not reached either way: its side on the
  tube's circle lies along the tube's bottom rim, an edge of the tube, where
  the tube near the side is a wedge and not one face, and the tube's cap
  beside it is on the lens's own plane. None of that was run.
- **A second tube and bar, a different shape from Q603's.** Q576 also found a
  tube of radius 29 less 13 united with a bar of radius 20 at `(21, 0)`
  refusing the same predicate, both argument orders, on a five-corner
  C-shaped piece — Q897's kind of gap, but not Q897's own numbers: Q603's
  tube and bar are radius 3, 1 and 1.5, so this is a separate piece and not
  simply that one reproduced at another scale. Whether 365.6's rungs reach it
  was not run here; Q715, running beside this row, replays it.

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
  (Q637 and Q602 answered through that search fix before the rule was ever
  tried on them, Q714; 365.2.)
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
one; Q621's lens may be that piece.

> **2026-10-05 note (Q1019): the piece this waited for is a strip of a KiCad
> pin header's wall**, which holds no `f64` point and shares no side with a
> piece of the other solid. Neither form above places it; a third does, a
> corner the other solid's ray puts strictly inside or outside it (365.7).

> **2026-10-08 note (Q1357): the first form, the flood, is built**, one step
> deep: a triangle a rounding across on a comb's top asked for it (365.9).

And **the rule runs only where no point
inside the piece was found** (the `INSIDE_A_PIECE` exit of `place_the_piece`),
or, since 365.6, where only a rung was found and none could be placed.
A fuse that answers today reached a point for every piece, never gets to the
rule, and answers exactly as before; only refusals under that name can move.
The point search also stays a check on the arrangement (Q609 above), which a
rule tried first would silence.

> **2026-10-05 note (Q1079): also tried, since Q1064, where a point was found
> but no ray placed it.** `choose` (`fuse/select.rs`) now tries the side rule
> and then the corner rule after `by_a_plane` as well, for
> `Search::NoPointInsidePlaced`. The piece that first needed this was half of a
> bar's wall, `x ≥ 0`, against a block over `x < 0` that was united first with
> a bar on the same pipe: every point of that half lies on the first bar's
> pipe, which holds none of it, and the ray grazed there. **Since Q1080 that
> half no longer reaches this rule**: the ray places it itself now, because
> `count_where_the_ray_meets_it` (`classify.rs`) no longer grazes a ray on a
> face that holds none of the point. A temporary print in `choose`, run one
> test at a time with `mise exec -- just t <name> --no-capture`, showed the
> rule still reached this way by `how_often_a_certified_fuse_of_a_turned_pair_refuses`,
> `swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit`,
> `how_often_a_certified_fuse_with_a_torus_in_it_refuses`,
> `every_seed_in_the_checked_in_corpus_is_fused_without_panicking` and
> `a_fused_draw_hands_its_provenance_back_through_the_door`.

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

  **2026-10-05 note (Q1079): decided, by Q1064.** `choose` tries the side rule
  and then the corner rule after `by_a_plane` for a piece a point was found in
  and no ray placed (`Search::NoPointInsidePlaced`); see the note under 365.3.
- **Q637 and Q602 look like one stop**: a flat cap with one-corner circles for
  its outline and its hole, all six fuses, both reproduced above. Q637's row
  asked for that to be said there.

  **2026-10-05 note (Q905): answered as one stop, by Q714.** Both are a gap
  in the point search, not a reach for this rule; see the notes under
  365.2.
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

### 365.6 Rungs nearer each crossing, after every other point

*Queue row Q897. Lane `lane-q897-a-ladder-of-points-into-a-c-shaped-piece`,
off `dbf8eaa8`. Everything here was run, with throwaway prints and a probe
test that are taken out, unless it says otherwise.*

**The gap.** Q603's C (365.2) has an outer arc that runs the long way round.
`half_way_into_either_arc` goes half way from the chord's middle,
`(2.6875, 0)`, to where the line square to the chord crosses the circle,
`(−3, 0)`, and that is `(−0.15625, 0)`, in the hole; the other way is in the
bar. The C holds plenty of doubles, so this is family 1 of §346.3.

**What was built.** `rungs_towards_each_crossing` in `fuse/select.rs`: for
each side of a flat piece, each place the two half-way functions aim at, and
`k` from 2 to 52, the point `2⁻ᵏ` of the way back from that place to the
chord's middle; and where the side's material is outside its partner, the
point as far past it, `k` from 1. The shallow rungs of every side come before
the deep ones of any. The two half-way functions now build their points
through one `Chord`, with the same expressions in the same order.

**Where they are tried, and why there.** In `place_the_piece`, after the old
candidates and after `points_in_a_sliver_of_a_ball`, which still runs only
where no old candidate was held, as before. A piece any earlier point placed is
placed by that same point. A piece only a rung was held in and no rung placed
goes to 365.3's rule, as it did before the rungs, and is refused `a point in a
piece that every solid could place` only if the rule fails too: a rung that
cannot be classified must not take away an answer the rule gave. A piece an
old point was held in and no point placed is refused under that name as
before, without the rule; the rungs are tried there too, so one of them can
now place it.

**Why 52.** A rung `2⁻⁵²` of the way from its crossing is within about one
rounding of it, and the next would not differ from it. A piece thinner than
that beside every crossing gets the points it got before, then
`points_in_a_sliver_of_a_ball`, then 365.3's rule, then the same refusal by
name. The C needs one rung: the first that places it is half as far again past
the bar's arc as the arc is from its chord's middle, `(1.5028, ±1.6382)`, a
`k = 1` rung past a side whose material is outside its circle. The row's
point worked by hand, `(−1.578125, 0)`, a quarter of the way back from
`(−3, 0)`, is rung 9 of the C's 926; the piece holds it and it settles
`Outside`, but the earlier rung places the piece first.

**What it answers.**

- **Q603's tube and bar**: all 18 fuses answer, and
  `a_tube_less_a_bar_united_with_a_second_bar.rs` holds each to the tube `32π`,
  the bar `2.25π` per unit of height, and the lens with radius 3 less the lens
  with radius 1 per unit of shared height. On `dbf8eaa8` all 18 refused `a
  point inside a piece of a face`, naming the C's five corners.
- **Q716's lens** (the tube of radius 7 less 1, a bar of radius 3.6 at
  `(−3.5, 0)`): all six fuses answer and hold the same kind of volume, pinned
  in the same file. With the rungs switched off by hand, all six refused `a
  point inside a piece of a face` at `(−6.8986, ±1.1873, 0)`, so the rungs are
  what answers it.
- **Q635's cone and block** (Q972's question): not reached. The piece 365.3's
  rule places there, in all six fuses, is the cone's skin inside the block,
  between `(−12, ±13.416, 18)`. It is curved, so it has no rungs, and its
  search is still the six steps of the curved branch.
- **Q714's closed circles** have no chord, so no rungs, as the row said
  (read, not run).

**Pins that moved.** The row's files and the crate's own tests were run with
their pins untouched, the slow tier among them: 525 tests, one failure.
`two_holed_bars_whose_corners_lie_on_one_line` in
`a_joined_flat_face_whose_rings_enclose_no_area.rs` had all six fuses
refuse `a point inside a piece of a face`, and now answers all six, each one
solid that validates and measures the area of the quarter-turned pair beside
it, times 4: union `200π − lens − 4π`, intersection `lens − 4π`, difference
`100π − lens`, with `lens = 200 acos(0.6) − 96`. Over the six,
`join_up_what_is_one_face` is reached 18 times and `holds_the_other_rings` 8
times (breakpoints), and `rings_round` does not panic; what that answered was
not read.

**What this leaves.** Q635's piece is a curved piece, and Q929's ladder of
shorter steps from the corners' middle is the curved branch's version of this
one; whether it reaches Q635's piece was not run. Q714's closed circles are
still the job of `between_the_round_wires`.

### 365.7 A piece placed by a corner, and a strip whose corners round to two

*Queue row Q1019. Lane `lane-q1019-a-kicad-strip-placed-by-its-corner`, off
`d5023489`. Everything here was run, with a scratch copy of Q690's
`q690_one_fuse` that is taken out, unless it says otherwise. Q985 traced the
stop and probed the rule first, on `84b34d0f`.*

**The piece.** Q690's cut of the first solid of KiCad's
`PinHeader_1x11_P2.54mm_Vertical.step`, and of `1x39`, less the box over the
middle half of its own box (`Rule::Difference`, certified). The body's walls
have a V notch between each pair of pins, every chamfer at 45°, so the plane
of a notch's chamfer on one wall, carried across the body, meets the other
wall at a notch corner by design: `x + y = −0.254` reaches `x = −1.27` at
`y = 1.016`, where that wall meets the chamfer `−x + y = 2.286`. Every face is
cut by every plane of its own solid too, so the wall is cut there, and over the
planes the file stores the two lines are `9.48e-17` apart, under the `2.2e-16`
between doubles at `y = 1.016` (Q985's exact arithmetic). The strip between
them, `z` from 0 to `0.064`, holds no `f64` point. On `d5023489` both cuts
still refused `a point inside a piece of a face` naming its four corners; the
rungs of 365.6 do not reach a piece no double is in. It shares no side with a
piece of the box, so 365.3's rule cannot place it either. It is outside the
box, which runs from `x = −0.635` to `0.635`, so the refusal is a gap.

**The rule, after 365.3's.** `by_a_corner` in `fuse/select.rs`: a piece that
neither a point nor a side it shares placed is placed against each solid by
the first of its corners that solid's ray calls `Inside` or `Outside`; inside
one solid of the group is inside, outside every one is outside, anything else
refuses by the old name. It follows the side rule wherever that rule runs, so
after the rungs too.

**Why it holds.** `P` was cut by every surface of `S`, so its inside lies all
inside `S`, all outside it or all on its skin (§17.4). A corner `c` is on
`P`'s outline, so every open neighbourhood of `c` meets `P`'s inside. If `c` is
strictly inside `S`, some neighbourhood of it is too, so part of `P`'s inside
is inside `S`, and so all of it is; that neighbourhood touches no skin, so `P`
is not on the skin either. Strictly outside is the same the other way. The
ray's signs are decided over the corner's whole box, and the true corner is in
that box.
**`OnBoundary` is never read**: a piece on either side of a face, or on it, can
have its corner there. **Nor is a corner on any stored surface of `S`**, which
`is_off_every_surface_of` asks first: the ray reads a flat face by its stored
corners, and of a point on the stored plane it can certify a side the plane
does not have (§316.3, §351). The point search keeps such points off the ray
with `coincident_face`, which a corner has no face of its own to ask.

**Then the answer panicked, as Q985 found**: `Edges::of`'s "a piece of a flat
face has a side with no length", on the bottom face `z = 0`, a side along
`x = 1.27` from the vertex at `(1.27, −1.524, 0)` to the meet of `x = 1.27`,
`z = 0` and `−x + y = −2.794`. **Where the two were made one**: `Corners::of`,
by coordinate. A print there showed the meet joined into corner 68, the
vertex's, twice, through `Corners::at`, which makes two corners on one `f64`
triple one: both round to `(1.27, −1.524, 0)` bit for bit. That join is only
reached when the two go by different names, so no arrangement had folded one
into the other through `names_of_one_point`. **The arrangement keeps them
apart**: a side of a piece runs between two nodes it ordered along a crossing,
here `x = 1.27`.

**Naming them apart was weighed and not built.** Two corners on one `f64`
triple, with an edge between them, is an edge of no length as the answer is
written, which `Invalid::EdgeHasNoLength` names; whether `Model::validate`
would raise it here was not run. And the three meets of `x = 1.27`,
`x + y = −0.254` and `−x + y = −2.794` on `z = 0` are `8e-18` to `1.6e-17`
apart (Q985's `corner.py`): there is no `f64` triple to put each one at.

**What was built: a side with no length is left out, in `assemble.rs`.**
`sides_of` leaves out a straight side whose ends `Corners::of` made one
corner. Sound because the corners are named through one table for the whole
answer, so every face that has that side leaves it out, and no edge is left
with one face along it. The strip on the wall then keeps two sides, along its
two long lines, and both name one edge, between the same two corners: a wire
that walks out and back along it. **A flat piece's wire that walks every edge
of it as often each way is left out** (`encloses_something`), and a piece with
no wire left is not written. Sound because such a wire encloses nothing as its
corners are written, and it takes one use off each side of every edge it
walks, so the other faces along each edge stay paired: the wall beside the
strip and the notch's chamfer meet along that edge, as the design has them.
Both are needed, each one alone was run against the new test: without the
first it panics as above, without the second it refuses
`ResultIsNotManifold { count: 4 }`.

**What it answers.** Q690's four cuts, through the scratch copy, under each
rule:

| file | rule | pieces | volume | validates |
|---|---|---|---|---|
| 1x11 | difference | 11 | `164.922737253333` | yes |
| 1x11 | intersection | 1 | `51.6794660000000` | yes |
| 1x39 | difference | 39 | `583.042240079997` | yes |
| 1x39 | intersection | 1 | `184.911025999999` | yes |

Each pair sums to its solid's own measure, `216.602203253333` and
`767.953266079999`. The intersections were also worked by hand: the box's
block of the body, `1.27 × 13.97 × 2.54` (and `× 49.53` for 1x39), plus each
pin the box holds, `0.64 × 0.64 × 3.23` above and below the body, five pins
and nineteen: `51.679466` and `184.911026`.

`a_pin_header_body_less_a_box.rs` builds the 1x11 body without its pins as an
extrusion of the notched outline, in the file's own decimals, and cuts it by
the same box. Its planes are the extrusion's and not the file's, so its stop is
a different piece: on the bottom face, between the top right chamfer, the line
of a left notch's chamfer `x − y = 0.254` and the box's plane `x = 0.635`,
with two corners a rounding apart at `(1.27, 1.016)`, which the refusal named.
Why no candidate was proved inside it, when it is `0.6` wide, was not traced. Its corner `(1.016, 1.27)`
is outside the box and places it, and both answers then validate and add up to
the body. With the corner rule off the difference refuses `a point inside a
piece of a face`.

**Pins that moved.** In `a_narrow_lune_from_a_pole_corner.rs`, ten of the
twelve 45° wedges answer, each one solid that validates and measures its closed
form; with the corner rule off all ten refuse `a point inside a piece of a
face`, so the rule is what moved them. The difference at radius `0.001`
refuses `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` and at `1000`
`CannotNameWhichPieceOfACrossingThisIs`. Every other test of `rcad-topo`
passed with its pins as they were, and every test of `rcad-step`.

**`every_kicad_file_that_imports_round_trips_to_the_same_solid` accepts a
refusal and fails on a panic.** It cuts each solid by the same box before and
after the round trip, counts the same refusal on both sides as nothing to
compare, and catches no panic. So landing the rule without the change to
`assemble.rs` would have failed it, on the panic.

**What this leaves.** `the_edge_each_side_of_every_piece_is`, the naming
365.3's rule reads, still gives up at a side with no length rather than
leaving it out as `sides_of` now does, so a fuse that has one places no piece
by a side. Leaving it out there too was not tried. Q993 traces the families
that may stop on the same kind of strip.

### 365.8 A plane places a piece with corners on it

*Queue rows Q1288 and Q1279. Q1288's lane changed the rule, on
`lane-q1288-a-turned-cube-cornered-on-a-face` (`63a3cce8`, off `924e2d98`);
lane `lane-q1279-q1288-a-corner-on-a-separating-plane`, off `bff35f27`,
merged it and moved the pins it moves. Each count below says which of the two
ran it.*

**The rule.** `by_a_plane` in `fuse/select.rs` places a flat piece with
straight sides outside a solid with only flat faces by the plane `σ` of one of
that solid's faces, turned to face out, that has every corner of the solid on
or behind it. Since Q991 it asked for every corner of the piece **strictly**
in front of `σ`. It now asks for every corner **on or in front** of `σ`, and
**one strictly** in front. Each corner is still asked as the meet of three
planes, exactly.

**Why it holds.** The solid lies in the hull of its corners, so all of it is on
or behind `σ`. The piece lies in the hull of its own corners, so all of it is
on or in front. One corner strictly in front puts the piece's plane off `σ`,
so the two planes cross in a line and the piece lies on one side of it; the
piece's inside, open in its plane, is then strictly in front of `σ` and shares
no point with the solid. A side of the piece may lie on `σ`, and a piece
placed outside is placed by its inside. A piece with every corner on `σ` is
not placed: it may lie in `σ`, on the solid's face.

**Why it was needed.** A sliver a rounding across, whose corners are where
planes of the other solid cross by design, holds no `f64` point, so the point
search finds none, and its corners lie on those planes rather than in front of
them:

- **A turned cube with its corner on a block's face** (Q1288). Turned by 1.5
  about `(1, 1, 0)` with the corner at `(0, 2, 4)`, the cube's three planes
  meet `1.03e-16` inside the block's face `x = 0`, and cross it in three lines
  that cut out a triangle a rounding across, in front of all three, two of
  its corners on each. Q1288's lane ran Q1208's 400 random turns once: the
  planes meet a rounding inside the face in 201, whose unions were all refused
  `Undecidable` *a point inside a piece of a face* under both policies; every
  rule both ways round now answers each of them, and their volumes agree. The
  other 199 meet outside the face and are unchanged.
  `a_turned_cube_with_its_corner_on_a_face.rs` pins the corner at `(0, 2, 4)`
  under each policy, at `(0, 2, 5)`, where the planes meet exactly on the face,
  and the first 60 random turns, 29 of which meet inside.
- **A prism standing a rounding above a block's top** (Q1255, dropped for
  Q1279). At `x = −2.5`, `−1.25` and `1.5` the piece was the sliver of the
  block's top between where the prism's walls meet it, from `z = 0` to `2`,
  two of its corners on the plane of the prism's end. All 18 fuses (three
  placements, three rules, both orders) now answer, and
  `a_prism_lying_on_one_edge.rs` holds them against volumes in `LAID`.
- **The skew family** (Q1279, from Q1146's list). Its headline row,
  `how_often_a_certified_fuse_of_a_skew_pair_refuses`, refused 30 of 447 fuses
  on `bff35f27`, all for want of a point inside a piece, and refuses 16 now:
  15 still for that, and one is the limit below. The turned family's row went
  from 45 refused of 447 to 39: 33 for a point, 6 the limit below. Both counts
  were run by this lane, on its base and on its branch.

**A limit some of them reach next: an edge too short.** A fuse that gets past
the sliver can then write an answer whose own shortest edge is a rounding long,
and `Assembly::check_dynamic_range` refuses it `Invalid::DynamicRangeTooWide`.
The skew pair `three_cut_lines_through_one_corner_of_a_skew_pair.rs` pins third
— a block two by two leaning one, and a block two by three leaning two the
other way at `(5, 1, 1)` — answers its intersection (`7/6`) and difference
(`41/6`), and its union is refused at a spread of `5 · 2^51` exactly. The
strip of the second block's wall `x + 2y = 5` from `z = 1` to `2` is placed
outside the first block, and its end runs from that block's corner on `y = 1`
to where the first block's stored wall `x − y = 2` crosses its floor edge; the
corner is strictly in front of that wall over the stored planes (the file's
second test), and the two ends round to `(3.0000000000000004, 1, 1)` and
`(3, 1, 1)`, an edge `2^-51` long beside a coordinate of 5. That is a length
over a length, so no scaling fits the two in `f64` together: a limit, class D
by §3.5's `1e11`. `fuses_that_must_agree.rs` counts it by name for the turned
and skew families (`a_flat_refusal`) instead of failing on it; the six turned
ones run at spreads of `7.7e15` to `2.9e16`.

**Pins that moved, and where a refusal moved to.** In
`two_turned_cubes_a_rounding_apart.rs` the pair overlapping by `2^-52` is still
refused, a limit; with the first cube as argument the stop moved off the strip
of its top from `y = 0` to `0.5`, now placed by the second cube's plane
`y = 0.5`, onto the rest of that strip, inside the second cube. In
`a_prism_lying_on_one_edge.rs` the prism a rounding into the top is still
refused, a limit; with the block first the stop moved off the sliver from
`z = 0` to `2`, now placed by the prism's end, onto the one from `z = 2` to
`8`, inside the prism. Every other `rcad-topo` test passed as it stood.

**What this leaves.** A turned cube whose three planes meet a rounding
**outside** the block's face is still refused in 148 of Q1208's 199 under a
tolerant policy (Q1288's lane's count); filed as a row of its own, Q1353,
and §417 places its triangle inside the cube. Q1280
traces the four skew pairs Q1146's probe of this rule left refused, each on a
sliver in the plane of a face of the non-convex solid.

> **2026-10-10 note (Q1034, Q1606): a flat piece is placed by an edge of a
> face it lies on, and the naming of every piece's sides leaves out a side of
> no length.** Lane `lane-q1034-q1606-a-sliver-along-an-edge-placed-by-its-side`,
> off `dcd8ed35`. Every count here was run, the two changes alone on the base
> as well as together.
>
> **Q1034.** `the_edge_each_side_of_every_piece_is` in `fuse/assemble.rs`,
> which 365.3's rule reads, now leaves out a straight side whose two ends come
> out one corner and names the rest, as `sides_of` has done since 365.7,
> rather than giving up the whole naming; with no give-up left, the `Option`
> is off its return type. It is sound for `sides_of`'s reason: every piece
> names its ends through the one table of corners, so every piece with that
> side leaves it out, and sides are compared by edge number alone, so the side
> just stops being one a piece can be placed across. This closes what 365.7
> left. Alone it moved nothing measured: the skew row refused 14 of 450 fuses
> on the base and 14 with it.
>
> **Q1606.** `by_an_edge_of_a_face_it_lies_on` in `fuse/select.rs` places a
> flat piece `P` lying on the plane of a flat face `G` of a solid `S`, with a
> straight side `s` carried by `G`'s plane and the plane `H` across an edge `e`
> of `G`, and both ends of `s` on `e` and not one point. Near the middle of
> `s`, `S`'s skin is `G` and the face across `e` and nothing else, and `P` was
> cut by every surface of `S` (§17.4), so where `P` is near there is where it
> is. On `G`'s side of `H`'s plane `P` lies on `G`, facing `G`'s way or the
> other; on the other side it is inside `S` behind `H`'s outward plane and
> outside in front of it, whether `e` is convex or reflex. Every question is
> `which_meet_comes_first_along_a_line` along the line of `e`, each point the
> meet of three stored planes, so every answer is exact and nothing goes into
> the log. `s` is the one side 365.9's rule passes over, since `H` holds both
> its ends, and the new rule is tried after every other, 365.9's included, so
> 365.9's "it is tried last" no longer holds and a piece any older rule placed
> is placed as before.
>
> **What the two moved.** `how_often_a_certified_fuse_of_a_skew_pair_refuses`
> refused 14 of 450 on the base, 12 for a point inside a piece and 2 the edge
> too short above, with 2 not a solid; the doc's 13 of 447 had drifted. With
> Q1606 alone it is 11, 9 for the point, 3 not a solid, and with both it is
> 8, 6 for the point, 5 not a solid, so three fuses need both. Which ones was
> not run fuse by fuse; Q1280's probe needed both for Frame 4x3's
> intersection. Frame 4x3 /
> Comb 3 meets in two unit cubes, in either order, and the comb less the frame
> holds 9; the union, either order, and the frame less the comb have two parts
> touching along a line, refused `ResultIsNotManifold { count: 4 }`, the true
> answer, checked by hand. Comb 2 / Ell 5x4 touch only across `z = 1`: the
> intersection is empty, each less the other is itself, 9 and 51, and the
> union, either order, is not a solid. `a_sliver_along_an_edge_of_a_face_it_lies_on.rs`
> pins them; with Q1034 alone, five of its six tests fail.
> `swapping_the_two_sides_of_a_skew_pair_changes_not_one_bit` refuses 20 of
> 280 with both. The fuse corpus, the fused draw,
> `three_cut_lines_through_one_corner_of_a_skew_pair.rs` and
> `a_sliver_on_the_plane_of_a_face_it_misses.rs` pass as they stood.
>
> **What this leaves: two of the four pairs.** Frame 3x3 / Ell 4x4 stops at
> `(3, 2, 1)` and Comb 3 / Comb 2 at `(4, 1, 1)`, every rule, either order,
> `a point inside a piece of a face`, pinned there. Each is a triangle a
> rounding across on the floor `z = 1` of one solid, cut by two walls of the
> other that meet at an upright edge and by the first solid's own leaning
> wall, stored rounded, through that edge. No face of the other solid is on
> `z = 1`, so the edge rule cannot reach it, and 365.3's rule got no named
> side for it, with the naming not refused; that every side of it comes out
> one corner is a lead, not run. By its sides' material it lies inside the
> other solid. Each is a row of its own, `refuses/A`.

> **2026-10-10 note (Q1608, Q1609): a flat piece is placed by a side that
> starts inside an edge of a face, and a shared side is read only off a piece
> whose side names the piece's own plane.** The fix and its pins are on
> `lane-q1608-q1609-a-sliver-under-a-rounded-wall` (`a6c7684c`); lane
> `lane-q1608-q1609-finish-the-sliver-under-a-rounded-wall` merged it onto
> `1bf7140b`, moved the pins it moves and ran every count here on its branch.
> The base was not run.
>
> **The rule.** `by_a_side_from_an_edge_it_starts_on` in `fuse/select.rs`,
> tried after every other rule, places a flat piece `P` against a solid `S`
> by a side `s` carried by `P`'s plane and the plane of a flat face `G` of
> `S`, one end of which is where `P`'s plane crosses the line of an edge `e`
> of `G`, strictly between `e`'s ends. Near that end `S`'s skin is `G` and
> `H`, the face across `e`, and nothing else. Where `s` runs to `G`'s side of
> `H`'s plane, `P` is inside `S` when its material is on `G`'s inner side;
> where it runs beyond `e`, `P` is inside when `s` there is behind `H`'s
> outward plane, whether `e` is convex or reflex. Every question is the meet
> of three stored planes, or two planes compared bit for bit, so every answer
> is exact and nothing goes into the log. It places the two slivers 365.8's
> last note left: the triangle on the ell's floor at `(3, 2, 1)` and the one
> on the second comb's floor at `(4, 1, 1)`.
>
> **The partner check.** `SidesItShares::is_inside`, 365.3's rule, now reads
> a side `P` shares with a piece cut from `G` only when that piece's side also
> names `P`'s own surface as its partner, as well as `P`'s side naming `G`'s.
> Two sides are one edge of the naming when their ends are named the same
> corners, and a corner can take in two places a rounding apart. In the combs
> the strip of the first comb's gap floor `y = 1`, from `x = 3` to where the
> second comb's wall `x + 2y = 5` crosses it `3.7e-16` along, lies on the
> second comb's gap floor. Its side on that wall was named one edge with the
> cut of that wall by the first comb's `x − 2y = 1`, `9.3e-17` above, read off
> that cut, and put outside the second comb, and the intersection kept neither
> copy of it. Now 365.3's rule does not place it, and Q1606's
> `by_an_edge_of_a_face_it_lies_on` places it on that floor, facing its way, in
> each rule's fuse (a temporary print, since removed).
>
> **The twelve answers**, under `Policy::certified`, volume give or take its
> error, pinned in `a_sliver_along_an_edge_of_a_face_it_lies_on.rs`:
>
> - Frame 3x3 / Ell 4x4, frame first: the intersection is two parts,
>   `1.6666666666666667 ± 0.0`; the union `27.333333333333325 ± 1.1e-14`; the
>   difference `14.333333333333337 ± 5.3e-15`.
> - Ell first: the intersection the same; the union
>   `27.333333333333332 ± 8.5e-16`; the difference three parts,
>   `11.33333333333333 ± 2.4e-15`, pinned bit for bit.
> - Comb 3 / Comb 2, comb 3 first: the intersection two parts,
>   `3.0 ± 8.3e-17`; the union and the difference refused
>   `ResultIsNotManifold { count: 4 }`. That the stored planes describe no four
>   faces along an edge there was reasoned and not run.
> - Comb 2 first: the intersection and the union refused the same way; the
>   difference four parts, `4.0 ± 4.0e-16`.
>
> **The ell less the frame is short, and is no defect.** Its `11.33333333333333`
> is `2.9e-15` under the ell's `13` less the intersection's
> `1.6666666666666667`, past its error. A throwaway probe worked out the
> triangles of all six frame and ell answers exactly. Every part of each
> measures within its own error of its own triangles, so the measure keeps its
> bound. Six times the union less the intersection and both differences is
> exactly nothing, either order, so no piece is lost
> (`no_answer_of_a_leaning_frame_and_a_leaning_ell_loses_a_piece` pins it).
> What takes it past its error is two things: its largest part measures
> `2.4e-15` under its own triangles, nearly all its error, and it and the union
> write the ell's corner `(−1, 4)`, where its wall `x + 2y = 7` meets `y = 4`,
> at `(−0.9999999999999998, 4)`, though the frame does not reach `y = 4`. That
> costs the triangles `2.6e-16` against the ell's. The stored wall crosses
> `y = 4` at `−1 + 1.24e-16`, worked out exactly from its stored coefficients,
> which rounds to `−0.9999999999999999`, so the corner is written one rounding
> past the stored planes' meet. The answer validates and every measure holds
> its own triangles, so no row is filed; why the corner moves was not traced.
>
> **What moved in `fuses_that_must_agree.rs`.**
> `how_often_a_certified_fuse_of_a_skew_pair_refuses` refuses 2 of 450, from
> the 8 365.8's last note gave, both the edge too short above, so its check
> that a point inside a piece is the most-named cause now names the edge too
> short. Not a solid went from that note's 5 to 7: the comb pair's union and
> difference with the first comb first, which stopped at the sliver before
> and now get past it. `swapping_the_two_sides_of_a_skew_pair_changes_not_one_bit`
> refuses 12 of 280, from that note's 20, and 2 of its comparisons disagree
> about whether the answer is a solid: both the comb pair's intersection,
> answered with the first comb first and refused with the second first. That
> count was not run on the base; the last note had this pair refused both ways
> round there.
>
> **What this leaves.** Two rows, filed by this lane's merge: the comb pair's
> intersection with the second comb first, refused with four faces along an
> edge while the other order answers, `refuses/B`; and the comb pair's unions
> and the first less the second, refused the same way, a row with no class
> since its cause was not run, with `assemble.rs`'s corner naming a lead.

### 365.9 A piece placed by the piece of its own face across a side

*Queue row Q1357. Lane `lane-q1357-a-triangle-on-the-plane-of-a-face-it-misses`,
off `d50273dc`. Everything here was run, with a temporary switch that turned
the rule off and is taken out, unless it says otherwise.*

**The piece.** A comb of two teeth two high, three high at the origin, and an
ell two by five with a bite one by four, leaning back one along `x` per step
along `y`, one high at `(1, −3, 3)`: one pair of
`how_often_a_certified_fuse_of_a_skew_pair_refuses`. The comb's walls `x = 2`
and `y = 1` and the ell's wall `x + y = 3` all pass through `(2, 1, 3)`, the
comb's corner at the foot of its right tooth, and the ell's wall is stored
rounded. So the three cut a triangle out of the comb's top, with corners
`(2, 1, 3)` exactly and two meets with the leaning wall in boxes
`[1, 1 + 2⁻⁵²]` along `y` and `[2, 2 + 2⁻⁵¹]` along `x`, as the refusal
record printed them. It holds no `f64` point. It lies in the plane of the
ell's floor, `z = 3`, but the floor at `y = 1` runs from `x = 0` to `1`, so
it is well outside the ell. Every rule stopped on it, `a point inside a piece
of a face`. `by_a_corner` passes over each corner, since each is on the ell's
plane `z = 3`, and `by_a_plane` finds no plane of the ell, which is not
convex, with the ell behind it and the triangle in front.

**The two roads.**

1. Place the triangle by its side along `y = 1`, a cut by the comb's own wall,
   taking the placement of the piece of the comb's top across that side.
2. Test exactly, in the plane, whether a corner lying on a plane with the
   whole ell behind it is in the ell's skin on that plane.

**The choice: 1.**

- **It asks nothing new.** It reads the arrangement's own sides and asks of
  each end of one side, as the meet of its three planes, which side of a plane
  it is on: `side_of_a_plane_where_three_planes_meet`, which `by_a_plane`
  already asks. Road 2 needs a crossing count in the plane over corners that
  are meets of three stored planes, with its own ties where a corner of the
  ell's floor lies on the line the count runs along, and the skin of a solid
  that is not convex, on such a plane, is its faces there and any edge or
  corner touching it as well. None of that exists.
- **It is the form 365.3 weighed and left** "until a piece the rule cannot
  reach asks for one". This piece asks.
- **It does not need a supporting plane.** Road 2 places only a corner on a
  plane with all of the solid behind it. Road 1 does not care where the
  piece is, only that its neighbour can be placed.

**The rule** (`by_the_piece_across_a_side` in `fuse/select.rs`). A flat piece
`P` of a face `F`, a solid `S` with only flat faces, and a straight side `s` of
`P` that `P′`, another piece of `F`, has too, run the other way. If no plane
of `S` other than `F`'s own (bit for bit, either way round) holds both ends of
`s`, each asked exactly, then `P` is placed against the group where `P′` is.

**Why it holds.** Each of `P` and `P′` was cut by every plane of `S`, so each
is all inside `S`, all outside or all on its skin (§17.4). A plane of `S` that
does not hold both ends of `s` meets the line of `s` in one point at most, so
all but a few points `m` of `s` lie on no plane of `S` but `F`'s. If `m` is off
`S`'s skin, a neighbourhood of it is all inside or all outside, and it meets
the inside of both pieces, so the two are placed alike. If `m` is on the skin,
it is on a face of `S` in `F`'s plane and on no other plane of `S`, so near
`m` the skin is a disk of that plane with `S` on one side; both pieces reach
into the disk, so both are on `S`'s skin, facing the same way. A plane of `S`
other than `F`'s holding the whole side is the one place the two can differ,
along an edge of `S`, and such a side is passed over. Per solid the two pieces
are placed alike, so they are against the group too.

**Its limits, as built.** The neighbour is placed by its own point search
(`place_the_piece`) and by nothing else, so a chain of two slivers is not
reached; nobody has built one, so that is a guess with no class (§360.5).
Under `Policy::tolerant` the arrangement may fold nodes it only found near
each other, so `s`'s ends are as exact as the drawing, which is true of every
rule here. **It is tried last**, after the sides, the corners and the plane,
for either search's failure, so a fuse that answered before answers as it did.

**What it answers.** All six fuses of the pair, both orders, certified:
the union holds 27, the comb's 21 and the ell's 6, the intersection nothing,
and each difference the first solid, each answer validating and logging
nothing. `a_sliver_on_the_plane_of_a_face_it_misses.rs` holds them; with the
rule off both its tests refuse `a point inside a piece of a face`. Q1280's
probe, which placed the triangle outside by hand, got the same three answers.

**Pins that moved.** `how_often_a_certified_fuse_of_a_skew_pair_refuses`:
16 of 447 refused with the rule off, 15 for the point and one the edge too
short of 365.8, and 13 with it, 12 for the point. Its threshold, 18%, was not
moved. `swapping_the_two_sides_of_a_skew_pair_changes_not_one_bit` refuses 22
of 276 either way.

**One pin outside the row's files moved, and is not moved here.**
`two_walls_a_hair_apart_are_caught_before_the_walk_gets_a_two_sided_piece`
in `a_piece_of_a_face_bounded_by_two_straight_sides.rs` pins the tolerant
union's last record as `a point inside a piece of a face`, on a sliver about
`1e-99` wide that it calls a limit. A temporary print showed the rule place
that sliver: the triangle on `z = 10` between the plain knife's line `y = 5`
and the leaning knife's line, from `x = 0` to `5`, takes the placement of the
square of the same face across `y = 5`, which the ray puts inside the leaning
knife. That is right: the leaning knife's plane holds `(0, 5, 10)` and not
`(5, 5, 10)`, and the sliver lies on the same side of it as the square. So the
sliver was a gap and not a limit. The fuse now stops a piece later, `a point
in a piece that every solid could place`, on a piece of a wall from
`(−5, 5, 2)` to `(0, 5, 5)`, which this did not trace. Every other test of
the workspace passed with its pins as they stood, and the slow tier passed 40
of 40.

**What this leaves.** Q1356's four pairs are not reached: the skew row moved by
this pair's three fuses and no more. Their slivers lie in the plane of a face
of the other solid with a side along an edge of that face, which is the one
side this rule passes over.
