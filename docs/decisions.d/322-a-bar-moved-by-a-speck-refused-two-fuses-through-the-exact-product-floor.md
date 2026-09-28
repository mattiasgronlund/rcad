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
  family. It was refused before this section and is refused after it.
