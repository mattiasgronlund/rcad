## 346. A point in a piece narrower than its round-trip box — 2026-09-28

*Queue row Q430, which replaces Q226, from §288.5 and §248.3 (Q112). Lane
`lane-q430-a-piece-narrower-than-its-round-trip-box`, off `82f20827`. §288.5
named two shapes where `select` refuses `a point inside a piece of a face` with
an infinite enclosure, and called both one gap. **They are two different
pieces with two different answers.** The small ball on a cube's corner was a
gap in `select` and is fixed: a point put back onto a curved face got a box
sized by the point and floored at one unit, and the box is now sized by the
numbers its round trip goes through. The fuse then gets past `select` and is
refused by the check of its answer, which cannot mesh the ball's faces
(346.2). The grazing block's piece **holds no point with `f64` coordinates at
all**, which a test shows. No point `select` could build is inside it, however
it is built, so that refusal stays. The fuse's answer still exists, so it is
still a gap of the fuse, and what could close it is outside `select`'s point
search (346.3). Neither shape shows which place `a_corner_is_this_place`
folds yet.*

  346.1   Which piece refused
  346.2   The small ball: the box was floored at one unit
  346.3   The grazing block: no double is inside the piece
  346.4   The pins
  346.5   What this leaves
  346.6   A second shape: a cap a fraction of a spacing deep and `1e-8` wide

### 346.1 Which piece refused

A throwaway probe in `place_the_piece` printed the piece, its sides and every
candidate whenever it gave up. It was run on the base over
`a_block_edge_grazing_a_ball.rs` and `two_meets_reach_one_corner.rs`, then
taken out.

- **The small ball** (`s = 2⁻⁴⁸`, radius `4.6e-14`): the piece of the ball's
  skin inside the cube, bounded by the three walls `x = 0`, `y = 0` and
  `z = 0`. Every candidate was a box reaching `5.68e-14` from its middle,
  which is `256 ε`, so each box was wider than the ball. The piece held none of
  them.
- **The grazing block with the ball first** (`a_block_edge_grazing_a_ball.rs`
  at `h = 0`): the piece of the ball's skin inside the block, bounded by the
  block's faces `x = −1` and `y = 2⁻²⁵`. Its boxes were the same `5.68e-14`.
- **The grazing block with the block first** (`two_meets_reach_one_corner.rs`):
  the piece of the block's face `y = 2⁻²⁵` inside the ball. Its candidates are
  exact and all lie on its straight side, so the winding cannot say.

### 346.2 The small ball: the box was floored at one unit

`back_onto_the_surface` puts a candidate back onto a curved face through the
surface's own two coordinates, and gives the result a box of
`Surface::ROUND_TRIP_ROUNDOFF` times a size. That constant's own doc names the
size: the largest of the point's coordinates, the numbers the surface is
placed with, and its radius. The point is the one on the surface, after the
trip, and not the candidate before it (346.4 has what that cost). `rcad-geom`'s
`a_round_trip_through_a_surfaces_own_two_coordinates.rs` measures the bound
against exactly that. `select` used the point's coordinates alone and floored
them at one. That is wrong both ways:

- **Too wide for a small surface.** A ball of radius `13s` near the origin got
  a box `2 × 256 ε` wide whatever `s` was.
- **Too narrow for a surface placed with big numbers.** A pipe whose axis
  origin is far from the point was sized by the point only.

The derivation beside the constant has no unit in it: every term is a fraction
of the size. It needs each rounding to cost a fraction of the number rounded,
which fails only for subnormal numbers, where a rounding can cost an absolute
`2⁻¹⁰⁷⁵`. So the size is floored at `f64::MIN_POSITIVE / f64::EPSILON`,
`2⁻⁹⁷⁰`, where the box reaches `2⁻¹⁰¹⁴`, `2⁶¹` times one such rounding.

**Measured, three rules and both policies at each size.** The same probe
fused the small ball at six sizes, once with the old box and once with the
new one (`two_meets_reach_one_corner.rs`, a throwaway `q430_probe` test and an
environment switch between the two boxes, both taken out). The new box here is
`52eb56fb`'s, which also sized by the candidate before its trip (346.4). On
`3e0abbe7`, the committed test below sees one of the same three refusals at
`2⁻⁴⁸`, `2⁻⁵⁰` and `2⁻⁵²` for every rule and policy; the other rows were not
run again.

| `s` | old box | new box |
|---|---|---|
| `2⁻⁴⁰` | Union `ACurvedFaceGoesRightRoundWithNoSeam`; the other two `CannotMeshAFace` (`BoundaryCrossesItself`) | the same |
| `2⁻⁴⁴` | refused, `a point inside a piece of a face` | as `2⁻⁴⁰` |
| `2⁻⁴⁶` | refused, `a point in a piece that every solid could place` | Union and Intersection answer, one solid, empty log; Difference `CannotMeshAFace` |
| `2⁻⁴⁸` | refused, `a point inside a piece of a face` | Union `CannotMeshAFace` (`CornersCoincide`); the other two `AWireWillNotLieFlatOnItsFace` |
| `2⁻⁵⁰`, `2⁻⁵²` | refused, `a point inside a piece of a face` | `CannotMeshAFace` (`CornersCoincide`) |

Both policies gave the same row every time. So `select` places every piece of
these shapes now, and the fuse is refused later, by the check of its answer,
exactly as it was at `2⁻⁴⁰` where `select` never refused. **That check is not
in this row's write set**, and whether each of those refusals is a limit of
meshing a face `1e-14` across in a model a unit wide, or a broken answer, was
not looked at. Until it is, no fuse of the small ball shows the fold either.

`a_ball_smaller_than_the_room_on_a_cube_corner_is_placed_and_its_answer_cannot_be_meshed`
replaces the test that pinned the refusal. It fails on the base: every fuse
there is `Error::Undecidable`.

### 346.3 The grazing block: no double is inside the piece

The ball has radius `R = 1 + 2⁻⁵¹`, so `R² = 1 + 2⁻⁵⁰ + 2⁻¹⁰²`. The two places,
`(−1, 2⁻²⁵, ±2⁻⁵¹)`, fence off three pieces: the ball's skin inside the block,
the block's face `x = −1` inside the ball, and its face `y = 2⁻²⁵` inside the
ball. A point inside any of them has `x ≤ −1` and `y ≥ 2⁻²⁵`, with at least
one of the two strict, and is on the ball or in it.

- The double nearest past `x = −1` is `−1 − 2⁻⁵²`. With `y = 2⁻²⁵` and `z = 0`
  that point is `1 + 2⁻⁵¹ + 2⁻⁵⁰ + 2⁻¹⁰⁴` from the centre, squared. Past `R²`.
- The double nearest past `y = 2⁻²⁵` is `2⁻²⁵ + 2⁻⁷⁷`. With `x = −1` and
  `z = 0` that is `1 + 2⁻⁵⁰ + 2⁻¹⁰¹ + 2⁻¹⁵⁴`. Past `R²` too.
- Every other such point is further out still.

So **no point with `f64` coordinates is inside any of the three pieces**. The
pieces are about `2⁻⁵⁰` long, `2⁻⁷⁸` across in `y` and `2⁻¹⁰³` in `x`, at
coordinates the size of one. A box does no better: a box proved past one of
the two planes has every point past that plane's nearest double, so it holds
no point of the ball either.
`no_point_with_f64_coordinates_is_inside_the_piece_between_the_two_places` in
`a_block_edge_grazing_a_ball.rs` classifies both nearest doubles against the
ball, through `SolidView::classify_point`, and both are `Outside`.

**The families of fix considered, and why none of them is inside `select`:**

1. **A better candidate**: one that never round-trips through the face's two
   coordinates, a point exactly on the ball, a blend with other weights. Each
   is still a `rcad_predicates::Point`, which is three doubles or a box with
   double ends, and none of those is inside the piece. This family is empty
   for this shape however the point is built.
2. **A power-of-two scale.** Scaling the model by `2ᵏ` scales the pieces and
   the spacing of doubles alike, so the piece is still a thousandth of a
   spacing across. Scaling fixes coordinates that are all too big or all too
   small, and here they are the size of one and their differences are not.
3. **Exact arithmetic.** The predicates are already exact on exact input. The
   trouble is before them: the input cannot be written down.
4. **Moving the origin to the corner.** Here that move is exact, and in the
   moved frame the pieces do hold doubles. But the point is then classified
   against the model's solids in the model's own frame, and there it is two
   doubles per coordinate. That is the next family.
5. **A point with more than one double per coordinate**: a `Point` variant in
   `rcad-predicates` that is the sum of two doubles, read by every predicate
   and by the ray classification. That is outside `select`.
6. **Placing a piece without a point in it.** Both sides of the ball's piece
   lie on the block's faces, with the piece on the block's side of each, at a
   convex edge. Or a piece takes its placement from a neighbour across a side
   that no solid of the other group made. Either replaces "classifying a piece
   is classifying one point inside it", which is `select`'s module doc and
   §17.4's shape, and the first needs the arrangement to say which face of
   which solid a side lies on. That is a decision to take, not a fix.

The first four are the ones `select` can reach, and 1 covers every point it
could hand on. So for placing by a point this is a **limit**: the point does
not exist. For the fuse it is still a **gap**, because the fuse's answer
exists. It is deferred to a row that owns 5 or decides 6.

### 346.4 The pins

The row named nine more test files that name this refusal. **No pin in any of
them moved.** `mise exec -- just t -p rcad-topo --no-fail-fast` on `3e0abbe7`:
1129 tests run, 1129 passed, 11 skipped.

**One did move on the way, and it was the fix's own mistake.** The first
version sized the box by the candidate before its trip as well as by the point
after it. On `52eb56fb` the same command ran 1129 and failed one:
`the_named_control_seeds_still_land_either_side_of_the_fuse` in
`every_model_the_fuse_fuzzer_kept_is_fused_again.rs`, where the seed
`a_curved_face_with_a_hole_that_goes_too_far_round` refused on both halves
instead of one. A probe showed why. A candidate is a step of one radius from
the middle of the corners, so before its trip it can be three units out where
the point it comes back as is two. That grew one ball's box from `1.14e-13` to
`1.71e-13`, and it lost the only candidate inside a cap near the ball's top.
The candidate before its trip is off the surface, and the box only has to hold
the true point at the two coordinates the trip came back with, so it is not
one of the numbers the bound is about. `3e0abbe7` takes it out.

**Q335 does not move, and it is not this piece.**
`a_fuse_that_needs_a_decision_nothing_can_prove_refuses` in `fusing_solids.rs`
passes on `3e0abbe7` as it did on the base (`mise exec -- just t -p rcad-topo
--test fusing_solids a_fuse_that_needs_a_decision_nothing_can_prove_refuses`).
Its piece is flat, the quarter of a tower's top at `z = 1e308`, half a unit
across, with four exact corners. A flat piece's candidates are exact and never
go through `back_onto_the_surface`, and a square half a unit wide holds plenty
of doubles. So it gives up for another reason, which was not traced here.

### 346.5 What this leaves

- **The small ball's answers are refused by their own check.** Whether
  `CannotMeshAFace`, `AWireWillNotLieFlatOnItsFace` and
  `ACurvedFaceGoesRightRoundWithNoSeam` there are limits of meshing a ball
  `1e-14` across in a unit model or signs of a wrong answer is the next
  question. `s = 2⁻⁴⁰` shows it without this fix.
- **The grazing block needs family 5 or 6 of 346.3.**
- **Q335's refusal is a different one** (346.4), still untraced.
- **The fold is still shown only by `arrange.rs`'s two unit tests** (§288.5).
  Neither test in `two_meets_reach_one_corner.rs` can show it through a fuse
  yet, and each panics, saying so, on the day its fuse answers.

### 346.6 A second shape: a cap a fraction of a spacing deep and `1e-8` wide

*Queue row Q658, which replaces Q398's trace. Lane
`lane-q658-a-ball-shaved-by-a-turned-blocks-face`, off `5268282c`.*

**The shape.** A block `[−2, 0] × [−2, 0] × [0, 2]` and a ball of radius 1
round `(1, off, 1)`, both turned about `z`, united. Unturned the ball touches the
plane `x = 0` at one point. Turned, that face's plane is stored a rounding off
and cuts the ball: `the_turned_face_cuts_the_ball_in_every_draw` in
`tests/a_ball_shaved_by_a_turned_blocks_face.rs` has `intersect` answer one
curve for each of the fourteen draws. Q398's `q398_room.py` worked out six of
them exactly: the circle has a radius of `5.2e-9` to `4.1e-8` and the cap is
`1.3e-17` to `8.5e-16` deep. With `off` negative the circle is on the block's
face and the union is one solid joined through it; with `off` positive it is
beside the block, and the union is the two apart, the ball with the cap as a
face of its own.

**It is not §346.3's piece.** That sliver holds no double at all. This cap is
as thin, but `1e-8` long. At the height of the ball's centre it is a strip
along the cut, a fraction of a spacing deep and at least `2 × 5.2e-9`, about
`1e8` spacings of `1.1e-16`, long, and
each spacing along it is another chance for a double to fall inside.

**What reaches it, in `select.rs`.** Tried only when no point
`candidate_points` offered was inside the piece:

1. `doubles_in_a_sliver` walks each cutting plane from the foot of the ball's
   centre, one spacing at a time, up to 64 each way, and at each step tries the
   six doubles next to the plane in the other coordinate. It keeps up to four
   points proved inside the ball and on the given side of every cut. Nothing
   found is trusted without those signs.
2. For a piece of the ball, `segments_up_through_a_cap` needs an axis no
   cutting plane's normal has a part along (`z` here), holds that coordinate at
   the centre's, and runs a segment from each point up to the first double
   proved outside the ball. The segment is a box that holds a point of the ball,
   and every point of it is on the same side of each cut.
3. For a flat piece bounded by the ball's circle, `points_beyond_a_plane_in_a_ball`
   keeps only points on the far side of the plane from the centre. A point
   there inside the ball has its foot nearer the centre, so inside the disk.
4. `side_of_a_plane` asks a plane about a box with each coordinate its normal
   has no part along narrowed to one end. Asked as a box, the segment through
   the cap turned by `1` (`3.6e-17` deep) was never held: the interval sum
   rounds by more than the cap is deep. Narrowed, it is asked exactly. The same
   narrowing is used where `select` compares the ray with the stored planes.

**Measured, on the base and after.** A throwaway probe fused every draw with
both policies and both argument orders (`crates/rcad-topo/tests/q658_probe.rs`,
taken out). On the base all 56 refused `a point inside a piece of a face`.
After, each draw answers alike under both policies and both orders. The third
column is since §402 (Q904), and the last since Q668; the paragraphs on each
row below say why:

| draws (`off`, turned) | refused with, after Q658 | since §402 | since Q668 |
|---|---|---|---|
| −0.5: 1; −0.3: 0.3, 0.5 | `a point in a piece that every solid could place` | the same | answers |
| −0.5: 1.5, 2.5, 3; −0.3: 1.5, 2.5, 3; +0.3: 2.5 | `a point in a piece that every solid could place` | the same | the same |
| −0.3: 2; +0.3: 0.5 | `CurvedFaceWithAHoleGoesTooFarRound` | answers | answers |
| +0.3: 5; +0.5: 5 | `Invalid::FacesDoNotAgreeWhichWayTheyFace` | answers | answers |

`what_each_ball_shaved_by_a_turned_blocks_face_comes_to` pins that table, its
last column now, and was `…_is_refused_with` until Q904. Q658's pin failed on
its base. `mise exec -- just t -p rcad-topo --no-fail-fast`: 1209 run, 1209
passed, 11 skipped. No other pin moved.

A draw that answers is checked to validate, to be one solid where the circle is
on the block's face and two where it is beside it, to measure within the error
it publishes of `8 + 4π/3` summed over its solids, and to keep a face on the
ball with two holes. The cap is below `1e-29` in volume and the closed form
leaves it out. The volumes are measured summed because the turn rounds the
block's corners: on its own, the block measured `7.999999999999997` with an
error of `1.4e-15` in `off 0.3` turned 0.5 (a throwaway probe, taken out),
which is the solid's own distance from `8` and not its sums' (§338.1). `mise
exec -- just t -p rcad-topo --test a_ball_shaved_by_a_turned_blocks_face
--no-capture` prints each of the 16 answers, four draws in two orders under two
policies: from `12.16826` to `12.16910`, against `12.18879`, with published
errors from `0.12615` to `0.12655`. The two policies give the same volume, and
the two orders give it to within `3e-14`.

**Why the first row still refuses: the block's ray and its stored plane
disagree about the cap.** A trace printed each segment's placement against the
block. In `off −0.5` turned 1.5, 2.5 and 3, and `off −0.3` turned 1.5 and 2.5,
the ray says `Outside` and `the_ray_and_the_stored_planes_disagree_about` says
the two disagree: the segment is proved on the block's side of the stored plane
`x = 0`, which is inside it by §17.1, and the ray fans that face from its
stored corners, whose triangles leave the segment outside. `select` drops such
a point, as §200.1 has it, and every point of the cap is within `8.5e-16` of
the plane. In `off −0.3` turned 0.3 the ray answers `Undecidable` (certified)
and `OnBoundary` (tolerant). **The guard is right to drop these, and the cause
is not in `select.rs`**: it is `classify.rs`'s reading of a flat face by its
corner triangles rather than its stored plane (`count_a_polygon`). Why the
other first-row draws refuse was not traced one by one.

*2026-10-04, Q668: traced one by one; three of the ten answer.* *Queue row
Q668, lane `lane-q668-a-ball-shaved-by-a-turned-blocks-face`, off
`2c349f2b`.* An `eprintln` in `settle_against_each` (taken out) printed each
segment's placement against the block and the guard's verdict, in all four
runs of each draw. In seven, `off −0.5` turned 1.5, 2.5, 3, `off −0.3` turned
1.5, 2.5, 3 and `off +0.3` turned 2.5, the ray said `Outside` and the guard
said the two disagree. In three, `off −0.5` turned 1 and `off −0.3` turned 0.3
and 0.5, the ray said `Undecidable` certified and `OnBoundary` tolerant, and
the guard was silent: the segment is a box `2e-8` tall, and its sign against
the corners' triangle is not settled, where the stored plane, asked with the
height it cannot see narrowed away, settles it exactly. Q658's reading of
`off −0.3` turned 0.3 is reproduced; `off −0.5` turned 1 was not among the six
it traced. That `off +0.3` turned 2.5 refuses for the same reason as the
first six was a guess and is wrong: its cap is beside the block, `0.3` past its
face `y = 0`, the ray's `Outside` is right by either reading, and the guard
drops the point because it asks the plane of every triangle of the face `x = 0`
however far from that face the point is.

**What changed, in `classify.rs`.** Where a ray's count turns on which side of
a triangle the point is, the face's stored plane is asked too: the ray crosses
the plane ahead when the point and the target are on opposite sides of it.
Where the triangle's certified sign says otherwise, or cannot say and the plane
can, no count is trusted and the ray names the face. The point's box is then
stretched along an axis to the plane, as `select` stretches one (item 4 above,
Q671's box). If the face holds every point of the stretched box and no other
face holds any, the skin inside the box is that face alone, so the point is
inside exactly when it is on the face's inner side of the plane. Otherwise the
answer is `Undecidable`. A point exactly on a triangle's plane still goes to
the outline as before, since every stored corner is one.

**Why not let the ray cross the face by its stored plane outright.** The ray
finds the triangle it passes through by the triangle's edges, which are the
corners'. Reading only the side by the plane mixes the two: a point in the
sliver within a sliver's depth of the face's edge could be counted through the
face where the plane meets the ray beyond that edge, past a second face that
puts the point outside. That was reasoned and not run, so it is a guess with no
class (§360.5), and it is why the plane is asked alone only over a box no other
face holds.

**What it does to the draws.** The three answer, in all four runs each, and
hold the test file's checks. `a_segment_up_through_the_cap_is_inside_the_block`
pins one segment from each of three draws against the block alone: `Inside`
under both policies, where it was `Undecidable` and `OnBoundary` in two and
`Outside` in the third. The seven still refuse: the ray now says `Inside` for
the six whose cap is on the block's face, the plane's answer, and `select`'s
guard still drops them, since it reads every triangle and not the ray's answer.
With the guard switched off by a throwaway check in `select.rs` (taken out),
all forty runs of the ten draws came back `Ok(Answers)` against the pin that
still had them refused, so each validated, was one solid or two as the side of
the circle says, measured within its error of `8 + 4π/3`, and kept a face with
two holes on the ball.

**What else moved.** `mise exec -- just t --workspace --no-fail-fast`: 3006
run, 3004 passed, 2 failed, 9 skipped. `mise exec -- just test-slow`: 40 run,
40 passed. Each of the two is a point the ray read by corners and the plane now
reads, and each new answer is the stored plane's exact sign, printed by a
throwaway `eprintln` in the new rule (taken out):

- `the_quarter_turned_knife_holds_the_axis_by_its_corners_and_not_by_its_plane`
  in `a_narrow_lune_from_a_pole_corner.rs`: `(0, 1.1e-16, −0.5)` against the
  knife, `Inside` before and `Outside` now. The stored plane, normal
  `(−2.2e-16, −1, 0)` and offset `−4.4e-16`, puts it `3.3e-16` outside, which
  that test's own second assertion says. Renamed at the merge to
  `the_quarter_turned_knife_holds_the_axis_by_its_corners_and_places_a_point_by_its_plane`.
- `which_side_of_the_face_meant_through_the_apex_it_is_on_does_not_say_whether_it_measures`
  in `meshing_a_plane_cut_along_a_cones_other_nappe.rs`: the apex against the
  block at lean one quarter, turn one, `Outside` before and `Inside` now. The
  steep face's stored plane has offset `4.4e-16`, so the origin is `4.4e-16`
  on the block's side of it. The test's claim survives the move: `(0.25, 1.0)`
  and `(0.375, 1.0)` are both inside, and only the second measures.

**The second row gets past `select`**, and the mesher refuses a face of the
ball with a hole of radius about `1e-8`. The refusal is `cut_up`'s fallback in
`mesh.rs`, after `cut_a_ring_across` and `cut_a_hole_up_to_both_poles` both
declined. Whether the face is refused for the hole's size or for its shape — a
ball face whose outer loop and hole reach no pole, which neither cut takes —
was not traced; that second reading is a guess nobody has run, so it carries no
class.

*2026-10-04, Q904: traced, and it was the shape, though not that one.* The
plane of the block's face `y = 0` cuts the ball too, beside the block, in a
circle `|off|` from the ball's middle and `2 acos |off|` round in longitude:
`2.532` for `off ±0.3`, `2.094` for `0.5`. So the ball's skin is a face that
holds both poles and has two holes, that wide circle and the tiny one of
`x = 0`, about `1.3e-8` round. An `eprintln` in the base's
`cut_off_what_goes_too_far_round` (taken out) printed that face in all eight
runs of this row, two draws in two orders under two policies: two loops, both
enclosing less than nothing of the chart, so both holes, neither going right
round, and the wide one first. Only a face of one wire was opened along a seam
(§202.1), and the first loop went more than a quarter turn round, so the face
went to `cut_a_ring_across`, which declined because no loop goes right round,
and to `cut_a_hole_up_to_both_poles`, which declined because no loop encloses
any of the chart, so there is no outer loop to cut the hole up to. **A gap**:
the skin exists. §402 fills it by opening such a face into strips from pole to
pole. A second `eprintln`, in the branch's strip cut and taken out too, printed
two strips of one hole each for the face of every answer, and each such strip
is cut exactly as the one hole of a ball less a disc is (§402.3). With the
base's `mesh.rs` put back, all eight runs refuse
`CurvedFaceWithAHoleGoesTooFarRound` again.

**The third row gets past `select` and the answer's own check refuses it.** In
`off +0.5` turned 5 the segment through the cap was placed `Outside` the block,
which is right: the cap is beside the block. What `validate` then found two
faces disagreeing about was not traced.

*2026-10-04, Q904: traced. The answer was built wrong and the check was
right.* The ball's skin is the same face with two holes, and here the tiny hole
came first, `1.1e-8` and `2.0e-8` round for `off 0.5` and `0.3`. A first loop
no more than a quarter turn round was handed over whole, so the triangulator
had the tiny hole for the face's outer loop and filled it, not the skin round
it. The check is `every_side_of_the_skin_is_used_both_ways` in `validate.rs`,
over the coarsest mesh. An `eprintln` there (taken out) printed, in all eight
runs, the sides that were not used once each way and the faces whose triangles
used them: four sides drawn by the face with two holes and one cap of the ball,
and four drawn by it and the other cap. The ball, beside the block, is three
faces there: those two caps of one wire each, the tiny one beyond `x = 0` and
the one beyond `y = 0`, and the face with both their circles for holes. The
strips cut that face as they do the second row's, and all eight runs answer.
With the base's `mesh.rs` put back, all eight refuse
`Invalid(FacesDoNotAgreeWhichWayTheyFace)` again.

**A guess Q658 named, demonstrated.** A flat piece's candidate is three doubles
off its plane by a rounding, and a round wire's partner is asked about the point
where it is, not about its foot. So a disk the ball is wider than just off the
plane can hold a point whose foot is outside it. A unit test in `select.rs`
showed it on a disk of radius `0.661`, with the point a tenth of a unit off; it
is `a_point_inside_a_ball_whose_foot_is_outside_its_disk_is_not_held` now. A
real candidate is a rounding off, so it reaches only a disk of radius about
`1e-8`, and no candidate `candidate_points` offers was seen to land there. The
new flat candidates in item 3 are built on the far side for this reason. Q671
closed it for the rest, below.

**Closed for every candidate, by two readings (Q671).** *Queue row Q671, lane
`lane-q671-a-flat-pieces-candidate-asked-at-its-foot`, off `86be3100`.*
`the_piece_holds` now asks each curved partner of a flat piece, a round wire's
and each curved side's alike, through `side_of_a_partner_by_its_plane`:

- **Over the point's box stretched to the plane.** `stretched_to_the_plane`
  moves the box's two ends along the axis the plane's normal has most of, until
  the plane's own sign puts each end's face on its own side of the plane. The
  box then holds a point of the plane as well as the point. A sign over a box
  is the sign of every point in it, so it says the same thing of the point and
  of a point of the plane beside it, whatever the partner is. **That is why the
  box, and not the far-side test alone**: the far-side test is an argument
  about a ball's centre, and it covers a point *inside* the ball only. A point
  outside a ball on the far side can have its foot in the hole the ball cuts,
  because the ball is narrower there than on the plane
  (`a_point_outside_a_ball_whose_foot_is_in_its_hole_is_not_held`). A pipe, a
  cone and a ring have no centre to be on the far side of.
- **And, for a ball, at the point alone where the point is inside it on the far
  side of the plane from its centre.** Its foot is nearer the centre then, and
  so is every point on the way down to it, so the reading holds there too. The
  box cannot read the cap: over it the ball's sign is too near zero for the
  interval sum to settle. With this reading switched off,
  `a_point_a_rounding_off_a_thin_disks_middle_is_held_from_either_side` failed
  at `+1e-16`. Q658's pin moved too: draws went back to refusing `a point inside
  a piece of a face`, among them `off −0.5` turned 1 and `off −0.3` turned 0.3,
  0.5 and 2.

Switching the box off instead failed the three `…_is_not_held` tests in
`select.rs`: the one above, the hole's, and the half disk's, whose arc is a
curved side rather than a round wire. The partner was read at the point, with
the far-side reading kept. Nothing else in `fuse::select` or Q658's pin failed.
All three tests fail on the base with the hold they assert against.

**What it costs, measured.** A probe ran `the_piece_holds` both ways over
`mise exec -- just t -p rcad-topo --no-fail-fast` (1214 tests; the probe is
taken out). Every answer that changed became `None`. 111 of them had been
`Some(false)`, which the caller reads the same way. 9 had been `Some(true)`:
the three new unit tests, and six in
`what_each_ball_shaved_by_a_turned_blocks_face_is_refused_with` (now
`…_comes_to`), on the disks of
`off −0.3` turned 0.3 and turned 2. The probe also wrote two lines that were cut
by two test processes writing at once. In each of the six, the point was within
`2.1e-17` of the disk's centre, on the ball's side of the plane. Exact
rational arithmetic (`q671_foot.py`, in the lane's scratchpad) put each point's
foot inside its disk. So the old answers were right. At the foot the ball's sign
is only `-1.7e-16` and `-4.2e-17`, which the stretched box cannot settle. Those
draws then find their point in item 3's search, and their pinned answers do not
move. `mise exec -- just t --workspace --no-fail-fast`: 2810 run, 2810 passed,
11 skipped. **So no fuse in the suite reached the wrong reading**, and the fix
costs one fuse some correct first candidates without changing its answer.

**What it does not change.** A flat piece's corners are still wound round the
point itself. `InPlane` reads a point that is off the plane at the place where
the line from its reference point through that point meets the plane. Whether
that place can be far enough from the point's foot to matter was not traced.
It is a guess nobody has run, so it carries no class.

**What this leaves**, each for a row of its own:

- The first row needs `classify.rs` to read a flat face's crossing by its stored
  plane, or `select` to place a piece some other way (§346.3 family 6).
  *Q668: done above for three of its draws. The other seven need `select.rs`'s
  `the_ray_and_the_stored_planes_disagree_about` narrowed or dropped, now that
  the ray names a face where the two readings part; with it off, all ten
  answer. Two pins outside Q668's files moved to the plane's answer, and its
  merge re-pinned both.*
- The second row needs `mesh.rs`'s `cut_up` traced for a ball face with a
  small hole. *Q904: done above; it answers since §402.*
- The third row needs the answer's check traced for which two faces disagree.
  *Q904: done above; it answers since §402.*
- The winding of a point off a flat piece's plane, above, if anyone runs it.
