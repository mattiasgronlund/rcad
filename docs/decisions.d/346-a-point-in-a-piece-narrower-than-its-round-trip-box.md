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
placed with, and its radius. `rcad-geom`'s
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
fused the small ball at seven sizes, once with the old box and once with the
new one (`two_meets_reach_one_corner.rs`, a throwaway `q430_probe` test and an
environment switch between the two boxes, both taken out):

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

The row named nine more test files that name this refusal. Whether any pin in
them moved is below, from `mise exec -- just t -p rcad-topo --no-fail-fast` on
the fix commit.

PENDING

### 346.5 What this leaves

- **The small ball's answers are refused by their own check.** Whether
  `CannotMeshAFace`, `AWireWillNotLieFlatOnItsFace` and
  `ACurvedFaceGoesRightRoundWithNoSeam` there are limits of meshing a ball
  `1e-14` across in a unit model or signs of a wrong answer is the next
  question. `s = 2⁻⁴⁰` shows it without this fix.
- **The grazing block needs family 5 or 6 of 346.3.**
- **The fold is still shown only by `arrange.rs`'s two unit tests** (§288.5).
  Neither test in `two_meets_reach_one_corner.rs` can show it through a fuse
  yet, and each panics, saying so, on the day its fuse answers.
