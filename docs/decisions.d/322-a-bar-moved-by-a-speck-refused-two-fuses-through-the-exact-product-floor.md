## 322. A bar moved by `2⁻¹⁰⁰⁷` refused two fuses through the exact level's product floor, and both are gaps — 2026-09-28

*Queue row Q100, from §246.6. Lane `lane-q100-a-tiny-move-reaches-a-predicate`,
off `e7b1894a`. **One cause for both refusals, and it is not the fuse.**
`2⁻¹⁰⁰⁷` squared is `2⁻²⁰¹⁴`, below the smallest subnormal, and `Exact`
refused every sum that product sat in. Neither answer is near a knife edge —
the plane-pipe room is `2⁻¹⁰⁰⁵` and the three-surface discriminant is `3` —
so both are **gaps**. Fixed in `rcad-geom` by keeping such a product with a box
round what it lost. The difference now fuses and measures the same as the
unmoved bar's, bit for bit. The union gets past every predicate and stops at
the triangulator, which is §246.5's refusal and Q79's row.*

### 322.1 The two refusals, reproduced

On `e7b1894a`, with the pair §246.1 builds — `Model::new(Scale::millimetres(8.0))`,
`add_round_bar(2.0, 8.0, identity)`, `place_solid` by `(0, 2⁻¹⁰⁰⁷, 0)`, and
`add_cube(2.0)` placed at `(10, 0, 0)` for the union and at `(1, −1, 3)` for
the difference — both refusals came back with §246.6's records to the digit. A
probe test, not committed, ran the same two fuses for five moves:

| move | union, cube far away | difference, cube through the wall |
|---|---|---|
| `0`, `2⁻¹⁰⁰`, `2⁻⁶⁰`, `2⁻⁵²` | fused, validates | fused, validates |
| `2⁻¹⁰⁰⁷` | "where a plane and a pipe cross", `Interval`, enclosure `8.9e-16` | "where three surfaces meet", `Exact`, enclosure `inf` |

**Where each is asked.** The union's cubes never touch the bar and are never
cut against it — `cuts::gather`'s box test keeps them apart. The refusal comes
from selection: `select::choose` → `place_the_piece` → `settle_against_each` →
`coincident_face` → `do_they_face_the_same_way`, which calls `intersect` on the
bar's wall and each of the cube's faces to learn whether they are one surface.
The cube's wall `y = 2` lies flat against the unmoved bar; moved, the bar pokes
`2⁻¹⁰⁰⁷` through it. The difference's refusal is `meet_of_three_surfaces` on
the cube's wall `x = 1`, the bar's cap plane `z = 0`, and the bar's wall, and
comes back through `an_exact_step_could_not_be_taken` — the refusal with an
infinite enclosure.

### 322.2 Why a move that small reaches either one

**The union.** The room a plane parallel to a pipe's axis leaves is
`n·n·r² − (n·o − d)²`. Here that is `4 − (2⁻¹⁰⁰⁷ − 2)²`, which is
`2⁻¹⁰⁰⁵ − 2⁻²⁰¹⁴`. The interval level rounds `2⁻¹⁰⁰⁷ − 2` to `−2` before it
squares anything and gets `[0, 8.9e-16]`. The exact level cannot form
`(2⁻¹⁰⁰⁷)²`: `product_is_exact` wants a product of at least `2⁻⁹⁶⁹`. It
cannot even form `1 · 2⁻¹⁰⁰⁷`, the reading's own product, for the same reason.

**The difference.** The line `x = 1, z = 0` against the moved pipe is
`s² − 2·2⁻¹⁰⁰⁷·s + (2⁻²⁰¹⁴ − 3)`, whose discriminant is exactly `3`: the
`2⁻²⁰¹⁴` cancels. But `how_a_pipe_cuts_the_line` builds the constant term out
of `m·2⁻¹⁰⁰⁷` and its square, `Term::times` refused both products, and a
refused product ended the whole meet before any sign was asked.

So one move reaches two predicates the same way, and the move's size is the
whole of it: `2⁻¹⁰⁰` squared is `2⁻²⁰⁰`, which the floor lets through.

### 322.3 Gaps, and the family that reaches them

A gap needs an answer that exists and a representable question that reaches
it. Both answers exist and neither is close: `2⁻¹⁰⁰⁵` of room, a discriminant
of `3`. Three families were weighed.

1. **The interval level.** Already tried by the cascade, and what it rounds away
   first is the move itself.
2. **A power-of-two slide**, `Exact::square_minus`. **It does not reach the
   union's room**, and `a_reading_no_slide_can_square_is_still_decided` in
   `exact.rs` is the demonstration. The reading `2⁻¹⁰⁰⁷ − 2` spans 1008
   binades. `WIDEST_SQUARABLE_SPREAD` is 995, and it is the published window
   with its own test (§29.6, §30.4). `square_minus` and `times` both return
   `None` on it. For the difference nothing here claims either way: the family
   below reaches it, and that is all the verdict needs.
3. **An expansion that keeps a product that falls off the bottom**, and carries
   a box round what it lost. This reaches both. Each product that rounds loses
   less than the smallest subnormal. The fused multiply-add makes the low half,
   and it rounds only when that low half is a subnormal itself. `two_sum` is
   exact everywhere. So the expansion is the true product give or take one
   `2⁻¹⁰⁷⁴` per product that rounded, and the error terms of a further product
   are worked out in intervals, which round outward. That is `Nearly` in
   `crates/rcad-geom/src/exact.rs`.

`the_box_holds_the_true_product_wherever_it_falls_off_the_bottom` checks that
bound on 2 700 products, two factors and a third after them. The truth is the
same product with both factors moved up `2⁶⁰⁰`, compared by exact signs.

### 322.4 What changed, and where it stops

- `Exact::times` is now `times_holding_on` plus "refuse if anything rounded".
  Where nothing rounds, a `Nearly` holds the very expansion `Exact` builds, so
  every question decided exactly before is decided by the same arithmetic.
- The plane-pipe room's enclosure is read off the `Nearly` expansion. Before, it
  came from the interval form.
- `three_surfaces.rs`'s `Term::Exactly(Exact)` is now `Term::Stored(Nearly)`.
  A product of stored numbers that falls off the bottom stays on the stored side.
  Its exact level cannot run, and its enclosure is the expansion and the box.

**A kept product is used only where what it lost is under one rounding of the
number it lands in** (`Nearly::is_within_a_rounding`). Otherwise the old
refusal stands. The first version had no such gate. It answered three surfaces
written `1e-159` across with a corner box a millionth of the shape wide:
every product there is near `1e-318` and loses bits. That box is sound, and
too loose for `every_point_where_three_surfaces_meet_somewhere_else_in_f64_is_on_all_three_of_them`
to tell the corner from a surface moved clear of it, and that test failed. The
uniformly tiny case is the scaling family's to answer, and it refuses as it did.
(§349 answers it; this stays as it was written.)
A line crossing one plane keeps refusing on any loss, because three planes
have Cramer's rule behind it. A ring keeps refusing too, because its quartic
needs its coefficients exactly.

### 322.5 A refusal rate that went down

`how_many_corners_two_bars_running_the_same_way_have_is_what_the_closed_form_says`
pins how many of its ninety tangent-and-turned rows go undecided. **51 before,
0 now.** A row can only move if some product on its way rounded, because a
`Nearly` that lost nothing is the `Exact` expansion entry for entry. So all 51
were this floor. The cause: the turned discriminant squares an expansion whose
lowest entries sit far down the range. The pin, `TANGENT_ROWS_A_TURN_LEAVES_UNDECIDED`,
is moved to 0 with that written beside it. §119.4's figure is history and is left
as it was.

### 322.6 Measured

- `mise exec -- just t -p rcad-geom -p rcad-topo --no-fail-fast`: 2091 tests
  run, 2091 passed, 12 skipped, with the gate of §322.4 and the pin of §322.5
  in.
- The tests that fail without the fix. Mutant: `Nearly::times` returns `None`
  whenever a product rounded, which is `Exact`'s refusal. With it,
  `a_plane_a_speck_inside_a_pipes_wall_cuts_it_in_two_lines` and
  `a_line_across_the_moved_pipe_meets_it_either_side_of_its_axis` fail in
  `crates/rcad-geom/tests/a_pipe_moved_by_a_speck_is_still_answered.rs`, with
  §246.6's own record for the second. So do
  `a_bar_moved_by_a_speck_less_a_cube_through_its_wall_holds_what_the_unmoved_one_does`
  and `no_predicate_refuses_a_bar_moved_by_a_speck_united_with_a_cube_far_away`
  in `crates/rcad-topo/tests/a_bar_moved_by_a_tiny_step_fuses_as_the_unmoved_bar.rs`.
  The controls pass either way.
- The difference moved by `2⁻¹⁰⁰⁷` measures `assert_eq!` the same volume as the
  unmoved one at deflection `0.01`, and validates.

### 322.7 What this leaves

- **The union far away still does not come back.** Past the predicates it
  stops at `CannotMeshAFace { why: CoordinateOutOfRange { position: 0, why: TooSmall } }`.
  That is spade refusing the bar's cap corner at `y = 2⁻¹⁰⁰⁷`: §246.5's
  refusal, and the flush queue row Q79 holds. **Not run here**: whether Q79's
  flush lets this union come back and validate. Once both are in,
  `no_predicate_refuses_a_bar_moved_by_a_speck_united_with_a_cube_far_away`
  should be tightened to fuse and validate.
- `select.rs` asks `intersect` for the whole crossing of a plane and a pipe
  when all it wants is whether the two are one surface, and they never are.
  Asking less there would not have hit this refusal at all. That file was the
  Q102 lane's while this ran, so it is named and not touched.
- The uniformly tiny triple §322.4 keeps refused is a gap for the scaling
  family. It was refused before this section and is refused after it. (§349
  answers it; this bullet stays as history.)

### 322.8 §39.4's refusals after `Nearly` — 2026-10-08

*Queue row Q329. Lane `lane-q329-the-refusal-catalogue-after-nearly`, off
`5f1bce87`, carried by two lanes in turn (§416).* §39.4 lists every refusal M4
can produce. It was written before `Nearly`, and nobody had asked which of its
rows a kept product moves. **One row moved, and `Nearly` did not move it.**
`Undecidable` out of `meet_of_three_surfaces`, the `2^k` family's 10.83%, now
refuses none of its 2051 triples. With the keep turned off it still refuses
none, and the row comes back the same to the last digit. §349's slide (Q327)
is what closed it. With the keep off, no test that pins a row of §39.4 fails.
§39.4's own figures stay as they were written.

**The two mutants.** Each is made on a scratch copy of `5f1bce87` (`git
archive 5f1bce87`, under `target/`), never on a tracked file:

- *keep off*: `if rounded > 0 { return None; }` straight after
  `times_holding_on` in `Nearly::times`. This is §322.6's mutant. A `Nearly`
  then never carries a box, and refuses wherever `Exact` refused.
- *slide off*: `the_meet_slid_into_range(surfaces).filter(|_| false)` in
  `meet_of_three_surfaces`. This is §349.6's mutant.

In the copy, the row is `cargo nextest run -p rcad-geom --test
every_point_where_three_surfaces_meet_is_on_all_three_of_them -E
'test(/somewhere_else/)' --no-capture`. The row prints before its `refused == 0`
pin fails, so every variant gives its figure. At the base it is `mise exec --
just t -p rcad-geom --test
every_point_where_three_surfaces_meet_is_on_all_three_of_them --no-capture`.
**Give each copy a `CARGO_TARGET_DIR` of its own.** The copies hold the same
files at the same paths inside each, and with one target dir shared, a re-run
of the *slide off* copy after the keep-off suite compiled nothing and printed
the *keep off* row: 0 refused where its own build refuses 180. Every figure
below was measured again with a target dir per copy.

| `somewhere else`, 2051 triples | corners | certified `Zero` | refused | widest box |
|---|---|---|---|---|
| as it stands | 806 | 254 | **0** (0.00%) | 3.0246243428348286e-16 |
| keep off | 806 | 254 | **0** (0.00%) | 3.0246243428348286e-16 |
| slide off | 670 | 240 | **180** (8.78%) | 3.0246243428348286e-16 |
| both off | 646 | 240 | **197** (9.61%) | 3.0246243428348286e-16 |
| §34.7 and §41.3, 2050 triples | 646 | 240 | 222 (10.83%) | 3.0246243428348286e-16 |

- **The keep answers 17 triples on its own**: 197 refused with both off, 180
  with only the slide off. The slide answers all 197 on its own. So nothing
  the keep answers on this row is left for it once the slide is in.
- *Slide off* is §349.6's row exactly: 180 refused, 670 corners.
- *Both off* has §34.7's corners and certified `Zero`s to the digit, and 25
  fewer refusals than §41.3's 222. **Those 25 are not traced.** The test file
  and its seeds have changed since §41.3 (`git log --` the file): one change
  kept another seed (`45386cd4`), and the row now draws 2051 triples where it
  drew 2050.
- **Where the keep shows in this file, it shows as a box, not a refusal.** The
  `turned` row's widest box is `2.410769996328908e-15` with the keep off, which
  is §41.3's figure to the digit, and `1.997652952092723e-15` with it. That row
  refuses none either way. With the keep off and the slide on, all eight tests
  in the file pass.

**Every test of `rcad-geom` and `rcad-topo`, with the keep off.** In the same
copy, `cargo nextest run -p rcad-geom -p rcad-topo --no-fail-fast`: 2822 tests
run, 2816 passed, 6 failed. All six are about `Nearly` itself:

- `a_reading_no_slide_can_square_is_still_decided` and
  `the_box_holds_the_true_product_wherever_it_falls_off_the_bottom`, the tests
  of `Nearly` in `exact.rs`;
- `a_plane_a_speck_inside_a_pipes_wall_cuts_it_in_two_lines`,
  `a_line_across_the_moved_pipe_meets_it_either_side_of_its_axis` and
  `a_bar_moved_by_a_speck_less_a_cube_through_its_wall_holds_what_the_unmoved_one_does`,
  three of §322.6's four;
- `how_many_corners_two_bars_running_the_same_way_have_is_what_the_closed_form_says`,
  the pin §322.5 moved to 0.

None of them pins a row of §39.4. §322.6's fourth test was rewritten by Q325
(`a54ecbde`) as
`a_bar_moved_by_a_speck_united_with_a_cube_far_away_holds_what_the_unmoved_one_does`,
and it passes with the keep off. Why is not traced. The plane-pipe room it
reached keeps its pin in `rcad-geom`, the first test above.

**The rows a kept product can reach, re-run at the base.** Outside tests,
`Nearly` is used only in `exact.rs`, `intersect.rs` and `three_surfaces.rs`.
It differs from `Exact` only where a product falls below `2⁻⁹⁶⁹`, the floor of
`product_is_exact`. Each test below asserts its refusal, so a pass means the row
stands as §39.4 wrote it. All seven pass, and none of them fails with the keep
off either:

| §39.4 row | then | now | why the keep does not move it |
|---|---|---|---|
| `Undecidable` out of `meet_of_three_surfaces` | gap, 10.83% | **answered**, 0 of 2051 | it does not: the slide moved it, as above |
| *order along a crossing*, a turned notched bar | gap | refused, certified and tolerant | its corner boxes come from `meet_of_three_surfaces`, but the refusal is §34.5's winding sum over them |
| `Undecidable` out of `intersect`, a direction with a subnormal part | gap | refused, and decided slid by `2^300` | the lean `n·w` is `Exact::dot_of` and is asked first; the room the keep serves is asked only after a `Zero` lean |
| `TheseTwoSurfacesDoNotCrossInAConic`, a dot that cancels in `f64` | gap | refused | `along_a_crossing.rs` has no `Nearly`, and `intersect`'s side is the same exact lean |
| `Undecidable` from `meet_of_three_planes`, past the top of the range | limit | refused | the keep still refuses a product that overflows |
| `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` | gap | refused | settled by the kinds of the three surfaces, before any product |
| `CannotWorkOutHowBigACrossingIs`, a bar too thin for its squares | gap | refused | the `f64` angle in `walk.rs`, with no `Nearly` |
| `ACrossingThisCannotBeOrderedAlong`, a ball one rounding off a bar's axis | gap | refused | refused by the kind of crossing, a quartic, before any order |

The seven tests, in one call: `mise exec -- just t -E
'test(a_turned_notched_bar_is_refused_for_want_of_an_order_along_a_crossing) |
test(a_ball_one_rounding_off_a_bars_axis_is_refused_before_any_order) |
test(a_direction_with_a_subnormal_component_cannot_be_told_from_square_to_a_plane)
|
test(three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere)
| test(a_bar_too_thin_for_its_own_squares_is_refused_rather_than_asserted_against)
|
test(a_conic_the_intersector_names_is_refused_here_because_the_two_ask_it_differently)
| test(three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved)'`.

> **2026-10-10 note (Q1554): the ball one rounding off a bar's axis is now
> refused meshing a face, not for want of an order.** Q1554 orders a ball's
> two loops round the bar's axis, so the last row of the table above no
> longer stops at `ACrossingThisCannotBeOrderedAlong`. Its test is now
> `a_ball_one_rounding_off_a_bars_axis_is_refused_meshing_every_fuse`, and
> that is the name to put in the command above in place of
> `a_ball_one_rounding_off_a_bars_axis_is_refused_before_any_order`. Every
> fuse is refused `CannotMeshAFace`: `CornersCoincide` with the ball first and
> for the bar less the ball, `ASideCutsOffOneCornerOfACurvedPatch` for the
> union and the intersection with the bar first, under a certified and a
> tolerant policy alike. Why the mesh refuses is not traced, and whether the
> keep moves the new refusal was not run.

**Two rows a kept product may reach, with nothing to re-run.** `Undecidable`
out of `face_holds_the_point` asks `intersect`, so the plane-pipe room is on its
way. *Which piece of a face a hole is in* follows rays along the crossings in
`fuse/arrange.rs`, and whether a kept product sits on that way is not traced.
§39.4 says no generator reaches either.

**The rows it does not reach**, one line each:

- `PlanesDoNotMeetInAPoint`: Cramer's rule is `rcad-predicates`' interval
  arithmetic, and the line through two planes still refuses any loss where it
  crosses the third (`where_a_line_crosses_one_plane`).
- `TheseTwoSurfacesDoNotCrossInAConic` for two planes, a plane down a pipe's
  axis, a ball against a pipe or two pipes, and
  `TheCrossingOfTheseTwoSurfacesIsNotClosed`: the kind of curve the pair makes.
- `DeflectionBelowWhatAMeshCanReach`: the mesher's floor, set by where the fuse
  puts the surface.
- The rows about faces in `rcad-topo` — what trims a face, which side of it a
  face lies on, poles, seams, arcs, cells, joins, cutting small enough, holes,
  which way an edge runs — and the rows struck through as closed: none of their
  tests fails with the keep off.
- Seven test names in §39.4 are not a `fn` at this base. Three *a point inside
  a piece* rows now answer: the bar cut off its axis and the two tools
  (`08883735`), and the bar and its copy (`647d8eed`). So does the ball cut
  down its polar axis (`c4c68524`), and the seam-twice test was renamed in
  `22f42553`. Those four commits are older than this section. The ball between
  two blocks was re-pinned by Q365 and Q508, and the Q1114 note's prism is now
  `the_prism_with_its_edge_a_rounding_into_the_blocks_top_answers_as_if_on_it`.
