## 448. Carrying coincidences from how the shape was built — 2026-10-13

*Mattias, 2026-10-13, after 446.7 sorted the class-B rows by kind: "I would
like us to try out the "carry coincidences from how the shape was built"
theory, if it holds it would probably help us a lot." Q1954, filed with this
section, tests it on nine rows and writes 448.5 onward.*

**In short.** Seven class-B rows refuse where two things touch exactly but the
numbers miss the touch by a rounding. Two more refuse after a rounding has
already made several things of one (446.7). The theory is that whatever built
the shape knew each coincidence when it made it, and that carrying the fact to
the question answers it under `Policy::certified()`, with nothing tolerated.
The kernel already does this for the faces round a corner
(`BakedCorner::around`). Q1954 traces the nine rows, says for each where the
fact was known and where it was lost, and files one writing row if one way of
carrying it covers three or more of them.

**Result.** The theory holds for (c), one movement both solids share. It
knew the coincidence on three open rows, Q1765, Q1842 and Q1936, and on
Q1958's shape, and Q1954 files one row to carry it to the fuse, after Q1962
(448.6). (b), a crossing's own surfaces, knew it for one row, Q1854. (a), one
solid's own faces, was lost in the build itself for Q1839, which stores a
face proved off its own corner. A fifth place the theory did not name, the
fuse's own choice of a test point, an aim or where a walk starts, explains
Q1834 and is how Q1958 was fixed. Q1867 and Q1734 touch only where the user
put them, (d), which the theory does not reach.

### 448.1 The theory

A refusal of 446.7's first or third kind asks a question whose exact answer is
that two things coincide: a point lies on a plane, four planes meet in one
point, a crossing runs through a corner. The numbers cannot show it, because a
turn, a division or a crossing rounded them. Where the same shape is built
with numbers that come out exact, it answers: Q1834 cut through the axis,
Q1765 at a quarter turn, Q1842 shifted.

The theory: whatever made the coincidence knew it was making one. If that
knowledge reaches the question, the question is answered from the model and
not from the arithmetic. That would help the third kind as much as the first,
and no tolerance reaches the third kind (446.7).

Three places the knowledge could come from. They are where to look, not
findings:

- **(a) One solid's own faces.** A corner lies on the surface of every face
  round it, and an edge on the surfaces of the faces either side of it.
- **(b) A crossing's own surfaces.** A point or a curve the fuse makes where a
  surface of one solid crosses a surface of the other lies on each surface it
  was made from.
- **(c) One movement for both solids.** A plane and its reverse stay exact
  opposites under any movement
  (`two_faces_that_touch_stay_touching_when_one_movement_moves_both` in
  `crates/rcad-geom/tests/isometry_algebra.rs`), and §41.13 found that two
  solids at two placements do not stay touching. So a touch between the two
  solids that was exact before a movement they share could be asked about in
  the frame before it.

And one that would mean the theory does not reach a row:

- **(d) Where the user put the two solids.** A touch that only the numbers the
  user gave make is known to nothing the kernel built.

### 448.2 What the kernel already carries

- `BakedCorner::around` (`crates/rcad-topo/src/fuse/bake.rs`) lists every face
  surface that meets at a corner, so that "which crossings run through a
  corner is then a question about the model rather than about arithmetic".
  That is (a), for corners, within one solid.
- `BakedCorner::solved_from` (Q1911, §294.20) names the three planes a corner
  was solved from when four or more meet there. Q1911 used it to put the
  pyramid's apex in order along each edge. That took the pyramid past one
  refusal and left Q1936's specks.
- §17.3 bakes placement into world coordinates when a fuse starts, and calls
  the rounding representation wobble. It says the fuse "cannot decide anything
  about two solids until their coordinates are in one frame, so one of them
  has to move". Two solids under one movement are already in one frame before
  it. That is the argument (c) would have with §17.3. The trace says only
  whether (c) would answer a row; whether to change §17.3 is a decision for
  after it.

### 448.3 What it must not become

- **Not a number that carries its history** (§1.1). A fact is kept on a
  corner, an edge, a face or a crossing, as `around` is, and never on a
  coordinate.
- **Not symbolic perturbation** (§1.5). A carried fact is true of the shape in
  exact arithmetic. It never picks a side for a point that is not on the
  surface.
- **Not a tolerance.** Nothing is taken as near enough. Where the numbers can
  still be asked, they are asked as a check, the way `derive_a_corner` asks
  each surface in `around` about its corner, and a surface that proves the
  corner off it is still a refusal.
- **An answer read from a carried fact is a certified answer**, so it adds no
  provenance record (§5). The rounding in the stored numbers stays
  representation wobble, as §17.3 has it.

### 448.4 What the row measures

**The rows**: the seven of 446.7's first kind, Q1834, Q1854, Q1839, Q1765,
Q1842, Q1737 and Q1867, and the two of its third, Q1936 and Q1734. Run every
one at the base, whatever its row's state.

Two of them moved while this was written. Q1734 closed at `45f6bca5`:
`zip_up_a_band` in `rcad-mesh` now meshes the band of torus wall a rounding
wide rather than refusing it. That is the other way to meet the third kind,
coping with the sliver instead of not making it. So for Q1734, step 1 will
find it answers; say as well whether the coincidence behind the band was known
from the build, since the two ways can then be set side by side. Q1936 was
taken into a lane at `743fbe6f`. Q1737 was dropped and filed again as Q1958 at
`c6962861`, on the same shape: Q1737's lane had fixed it at `a53f0a15` with
`Arrangement::the_node_round_the_start` in `fuse/arrange.rs`, and stopped on a
stale pin outside its write set, which Q1958 adds. Run the shape both rows
name. If it answers at the base, say, as for Q1734, which of (a) to (d) that
fix used.

**Each row** is run on the shape it names, under `Policy::certified()`, at the
lane's base. For each, write down:

1. **Whether it still refuses.** A row that answers is written down as
   answering, and steps 2 to 4 are skipped for it.
2. **The question that refuses**, in words — "is this point on that plane",
   "which of these comes first along that line" — with the function that asks
   it at the base (446.5 names most of them) and its record.
3. **Its exact answer, shown**: for example the same shape unplaced, the
   planes' coefficients in exact rationals, or the point built again from its
   surfaces in exact arithmetic. A refusal whose exact answer is not a
   coincidence is outside the theory. Say so, and stop there for that row.
4. **Which of (a) to (d) knew the coincidence, and where it was lost**: the
   line that rounded it, and whether the fact was still in hand when the
   question was asked. "`around` held it and the question asked the numbers"
   is a different finding from "nothing held it after the bake".

Q1839 and Q1867 refuse a range check, `DynamicRangeTooWide`, not a sign. For
them, say what small feature the check read, and whether it is a coincidence
the numbers missed. Q1839's row says its exact answer has no small feature.

**Probes** go under `/tmp`, or in a scratch test deleted before the commit, as
in 446.3. Nothing in `src/` changes, and nothing is fixed.

**What holds means.** The theory holds for a row when its exact answer is a
coincidence and (a), (b) or (c) knew it. If one of the three covers three or
more of the nine rows, the lane files one writing row, tier 1, `opus`: carry
that fact to the question that refuses, with the rows it should close named
one by one, each with the pin it turns round. If none of them reaches three,
the lane files nothing and says so. Then the theory is not what this tail
needs.

If 446.8, the trace of the two-thin-bars family, has landed by then and finds
that family's cause is a coincidence a turn lost, the writing row names those
rows as well.

**What the lane writes**: 448.5, the nine rows with steps 1 to 4 for each;
448.6, the count by (a) to (d), and the row it filed or why it filed none; and
a **Result.** paragraph under the In short above. The table should be committed
as it grows, so that a relay picks up from the file.

### 448.5 The nine rows at the base

**Step 1, run at `283ddff8`.** Q1930's probes from `/tmp/q1930-probes/` were
copied into `crates/rcad-topo/tests/` unchanged, but for a report that prints
every input of each record still `Undecidable`, run with `mise exec -- just t
-p rcad-topo -E 'test(/q1930_probe_q(1834|1854|1765|1737|1734|1842|1936|1839|1867)$/)'
--no-capture`, and deleted. 9 tests passed and printed 202 lines. The probes,
the report and the print (`q1954-base.txt`) are kept at `/tmp/q1954-probes/`;
`python3 q1954-short.py q1954-base.txt` gives the counts below. "Site" is the
predicate of the record still `Undecidable`, or the `Error` variant.

| Row | Runs | Certified at the base | Site and record |
|---|---|---|---|
| Q1834 | 5 cuts, 3 bars, both ways: 30 | refuses 30 | orient3d, `1.9e-16` to `8.9e-16`, at `(c, -s/2, z0 + h/4)` where `s` is `sqrt(1 - c²)`: `(0.5, -0.4330127018922194 to -0.43301270189221924, 2.5)` at `c = 0.5`, `z0 = 2`, `h = 2` |
| Q1854 | six fuses: 6 | refuses 6 | orient3d, `8.5e-16`, of the turned block's bottom normal `(0, -0.479425538604203, -0.8775825618903728)`, the side's `(-1, 0, 0)` and the box `(-1, ±0, ±8.9e-16)` |
| Q1839 | four placements, D and I: 8 | answers 3, refuses 5 | `Invalid(DynamicRangeTooWide)`, ratio `1.3e15` to `4.5e16` |
| Q1765 | pi/6 and 1 radian, wall 4 and 2 high, U and D both ways: 16 | refuses 16 | side of plane, `1.45e-13` at pi/6 on the box `[-1.0000000000000533, -0.9999999999999472] × [1.7320508075688241, 1.7320508075689303] × [0.9999999999999469, 1.000000000000053]`, `1.75e-13` at 1 radian on `[-1.6829419696158563, -1.6829419696157297] × [1.080604611736216, 1.0806046117363426] × [0.9999999999999367, 1.0000000000000633]` |
| Q1842 | four placements, D and I: 8 | answers 4 (unplaced and shifted), refuses 4 | turned: orient3d, `5.8e-13`, at `[1.4999999999999982, 1.5000000000000016] × [-2.920637567314879, -2.920637567314877] × [6.907957455021635, 6.907957455021639]`; turned then shifted: side of plane, `3.6e-15`, on `[9.999999999999993, 10.000000000000005] × [-5.0, -4.999999999999997] × [2.4999999999999982, 2.4999999999999996]` |
| Q1737 / Q1958 | 5 mm model, both notches, U, I and D: 6 | **answers 6** | — |
| Q1867 | nine two-knife wedges: 9 | refuses 9 | `Invalid(DynamicRangeTooWide)` |
| Q1936 | shifted, turned, turned then shifted, D and I: 6 | refuses 6 | a point inside a piece of a face, enclosure infinite; shifted, three corners whose boxes each hold the apex `(7, -5, 12.5)`, the first `[6.999999999999998, 7.000000000000003] × [-5.000000000000001, -4.999999999999998] × [12.499999999999996, 12.500000000000005]` |
| Q1734 | quarter and half turn, six fuses, each validated and measured: 12 | **answers 12** | — |

So Q1737's shape, which Q1958 closed, and Q1734's answer at the base, and the
seven others refuse as 446.4 found them.

**Steps 2 to 4.** Where each refusal is asked comes from the stacks Q1930
kept (`/tmp/q1930-probes/gdb-all-*.txt`, read with `lasthit.py`). The rest
comes from probes added to the copied files and run at `283ddff8`
(`q1954_probe_q1834`, `_q1765`, `_q1854` in `zz_q1930_simple.rs`;
`_q1842`, `_q1936`, `_q1839` in `zz_q1930_corner.rs`; `_q1867` in
`zz_q1930_lune.rs`). They are kept in `/tmp/q1954-probes/`, with the last
relay's `q1936_apex.py`, which works the pyramid's planes out in exact
rationals, and `relay3/zz_q1954_tetra.rs`.

- **Q1834.** *The question*: does the other half's cut face hold the point
  chosen inside a piece of the cut? `select::coincident_face` asks
  `holds::walk_one_outline`, and that asks orient3d of each corner against the
  plane through the point, the aim and a target off the face. *Its exact
  answer*: both halves' cut faces have the corners `(0.5, ±s, 2)` and
  `(0.5, ±s, 4)`, with `s = 0.8660254037844386`, and the first corner,
  `A = (0.5, -s, 2)`, is the aim (`holds.rs` 260-265). The point
  `(0.5, -s/2, 2.5)` is `A + (C - A)/4`, with `C = (0.5, s, 4)` the opposite
  corner. That is where `candidate_points`' first family puts it:
  `midway(corner, midway(before, after))`, a quarter of the way along the
  piece's diagonal. So the point lies on the line from the aim through `C`,
  and orient3d at `C` is exactly zero when the point is built from the stored
  corners. Built from the crossings, it is the box in the record. In exact
  real numbers it misses that line only by which way `sqrt(0.75)` rounded
  (`s*s - 0.75 = -8.7e-17` by `fma`). Either sign gives the same winding,
  since `C` is behind the point. *Which of (a) to (d)*: none. The fuse made
  the coincidence itself, by choosing a test point on a diagonal and an aim
  at one end of it. Cut through the axis, `s = 1` is exact, the zero is
  exact, and the union answers. The cure is a test point or an aim off the
  diagonal, not a carried fact.
- **Q1765.** *The question*: is the bar's seam corner on the cube's wall?
  `Arrangement::passes_through` asks it (`fuse/arrange.rs` 1795-1841, from
  `place_the_corners`) as `side_of_surface(wall, corner box)`. *Its exact
  answer*: before the turn, the corner is `(0, 2, 1)` and the wall is
  `x = 0`, so `Zero`. *Which of (a) to (d)*: **(c)**. The cube's faces carry
  the location `(R, 0)` and the bar's `(R, (-1.8660254037844388,
  1.2320508075688772, 1))`, with the same bits of `R`
  (`[0, 0, 0.2588190451025208, 0.9659258262890683]`). So the bar's placement
  is the cube's turn with a shift after it, and the shift was rounded into
  the bar's translation by `Isometry::compose`. Then
  `where_a_corner_lands_after_moving` gives the bar's corner as a box
  `1.06e-13` wide, because pi/6 is not one of the 24 turns of a cube it can
  check exactly. The fact was in hand until `compose`. After that nothing
  keeps the shared turn apart. (The corner as the view rounds it,
  `(-1.0000000000000002, 1.7320508075688772, 1)`, happens to be exactly on
  the wall as the view rounds it. That is a rounding that went the right
  way, not something the model knows.)
- **Q1854.** *The question*: which way do the slab's side and the pipe turn
  at a node on the slab's bottom? It is asked in
  `Arrangement::turns_anticlockwise` (`arrange.rs` 5181-5241) as orient3d of
  three gradients. *Its exact answer*: the side `x = -17` is stored exactly
  (`[-1, 0, 0, -17]`), because a turn about `x` keeps it, and the pipe's axis
  is `x = -16`, `y = 0`, with radius `1`. So the side rests along the pipe,
  the pipe's gradient on the touching line is exactly `(-1, 0, 0)`, two rows
  are parallel, and the determinant is `Zero`. `rcad_geom::intersect` already
  says so: the side meets the pipe in a `Straight` run, and every other face
  of the slab meets it in `RightRoundWhereItsFacesCross`. *Which of (a) to
  (d)*: the touch was made by the user's numbers (d). It stays exact in the
  two stored surfaces, so it is the crossing's own surfaces, **(b)**, that
  know it. Lines 5222-5227 read `Zero` for a plane resting along a cone and
  for two pipes that touch, but not for a plane resting along a pipe. The
  fact was in hand and was not read.
- **Q1842 turned.** *The question*: as for Q1834, in
  `holds::walk_one_outline`, about the point `(1.5, 0, 7.5)` turned 0.4 about
  `x`, which the probe puts at `(1.5, -2.920637567314879,
  6.907957455021638)`, inside the record's box. Unturned, that point is
  `C + (A - C)/4` on the half bar's flat face `y = 0`, with `C = (3, 0, 10)`
  and `A = (-3, 0, 0)` its opposite corners. It is on no diagonal of the
  block's face. *Which*: none, the same as Q1834: the fuse's own test
  point. The half bar's flat face, stored as `[0, 0.9210609940028851,
  0.38941834230865047, 0]`, has the wire `(-3, 0, 0)`, `(-3, -3.894…,
  9.2106…)`, `(3, -3.894…, 9.2106…)`, `(3, 0, 0)`, so its aim is `A`, and
  the point lies on the line from `A` through `C`. From the block's first
  corner, `(-5, 0, -1)` turned, it lies on no line through another corner,
  so the orient3d that refuses is the one at `C`. Unplaced and shifted,
  every number is exact and it answers. Both solids are turned by the one
  turn, so (c) would answer it too: in the frame before the turn it is the
  unplaced shape.
- **Q1842 turned then shifted.** *The question*: is the half bar's corner
  `(3, 0, 0)`, placed at `(10, -5, 2.5)`, on the block's face `y = 0`? It is
  asked in `Arrangement::passes_through`, as for Q1765. *Its exact answer*:
  before the placement, the corner is on `y = 0` in both solids, so `Zero`.
  As stored, the half bar's flat face is `[0, 0.9210609940028851,
  0.38941834230865047, 3.6317591142427994]` and the block's face is
  `[0, -0.9210609940028851, -0.38941834230865047, -3.631759114242799]`, one
  ulp apart in offset. So `is_on_by_construction` does not take them as one
  plane, and the box straddles both. *Which*: **(c)**. Both are at the one
  placement `(R, (7, -5, 2.5))`, and the block also carries its own shift
  `(-5, 0, -1)`, which lies along `y = 0`. `Isometry::compose` rounds
  `R·(-5, 0, -1) + t` into the block's translation,
  `(2, -4.610581657691349, 1.578939005997115)`, and the plane's offset
  rounds from that. Turned alone, both offsets come out `0` and the corner
  is `Zero` on both.
- **Q1839.** *The small feature*: under
  `Policy::certified().with_dynamic_range_check(Severity::Warn)`, every fuse
  that refuses at the base answers with an edge between the apex and a point
  a rounding from it. Unplaced, intersection: `2.46e-16`, from
  `(0, 2.458e-16, 10)` to `(0, 0, 10)`, and `11 / 2.458e-16` is the record's
  `4.47e16`. Shifted: `8.9e-16` to `2.2e-15`, a speck at `(7, -5, 12.5)`.
  Turned: `3.8e-15`. Turned then shifted: `1.1e-14`. The three that answer
  have no edge shorter than `3`. *Its exact answer*: in the shape the user
  built, the block's face runs exactly through the apex, so the
  tetrahedron's three side planes and that face meet in the one point, and
  no edge is there. *Which*: **(a)**, and it was lost in the build itself.
  The apex is a corner of all three side faces (`BakedCorner::around`), and
  the block's face is exactly through the stored apex unplaced and shifted.
  But `Plane::through_three_points` builds the `−y` side face as `[0,
  -0.9578262852211514, 0.28734788556634544, -2.8734788556634543]`, which is
  `Pos` at `(0, 0, 10)`, and the other two side faces are `Zero` there
  (`zz_q1954_tetra.rs`, kept in `/tmp/q1954-probes/relay3/` and run from
  `crates/rcad-geom/tests/`). So the face is stored proved off its own
  corner, and 448.3 makes that a refusal: the fact (a) would carry is false
  of the stored face, so it cannot be carried. (c) would answer 3 of the 5
  refusals: the difference shifted, turned, and turned then shifted, since
  the difference unplaced answers. It does not answer the intersection
  unplaced, which no movement placed, nor shifted, which before the shift is
  the unplaced intersection.
- **Q1867.** *The small feature*: the second knife's face through the axis
  is stored with offset `4.440892098500626e-16` at 90 degrees and radius 1
  (`4.5e-13` at 45 degrees and radius 1000, `-2.2e-16` at 10 degrees and
  radius 1), not `0`. So it misses the poles by that much, and
  `6 / 4.440892098500626e-16` is exactly the record's ratio
  `1.351079888211149e16`, with `6` the knife's reach. *Its exact answer*: the
  plane holds the axis, so the lune's tip is the pole and there is no small
  feature. *Which*: **(d)**. The ball is at the origin and the knife's face
  holds the `z` axis because of the numbers the user gave. Only the knife
  moves: `place_solid` composes its turn about `z` with its own shift
  `(0, -6r, -6r)`, and the offset rounds there. A turn about `z` keeps
  every point of the axis exactly, so the question could still be asked in
  the knife's frame before its turn. That is close to (c), but the movement
  belongs to one solid only. With the range check at `Warn`, certified then
  refuses by name (`ACurvedEdgeThatIsPartOfACrossingCannotBeNamed`, and
  `CannotNameWhichPieceOfACrossingThisIs` at 45 degrees and radius 1000), so
  no answer was there to measure.
- **Q1936.** *The question*: which side of the pyramid is a piece of the
  block's face on? At the base `select::choose` finds no point inside piece
  31 of 53 of the block's face, shifted: a triangle bounded by the `−x`,
  `+y` and `+x` side planes, with every corner within `5e-15` of the apex
  (§294.21). *Its exact answer*: shifted, the stored apex `(7, -5, 12.5)`
  is exactly on the block's face `y = -5` (`Zero`), and the four side
  planes, `−y`, `+x`, `+y` and `−x`, are `Pos`, `Neg`, `Pos` and `Pos`
  there. The shift rounds each plane's offset on its own, exact minus
  stored `-1.14e-15`, `6.4e-16`, `-2.5e-16` and `-2.5e-16`, so the four
  triples of side planes meet at `y + 5 = 4.6e-16` (two of them),
  `-4.6e-16` and `1.39e-15`, four points and not one, and those are the
  specks (`python3 /tmp/q1954-probes/q1936_apex.py`). Unplaced, each side
  plane is `1.11e-16` off `(0, 0, 10)`, because `10 × 0.28734788556634544`
  rounds, yet all four triples meet in one point with `x = y = 0` exactly.
  So the cut `y = 0` holds it, and unplaced the pyramid answers 60
  (§294.21). *Which*: **(c)**. The block is built at `(-5, 0, -1)`, and
  both solids are moved by one placement, which composes with the block's
  shift exactly to `(2, -5, 1.5)`: shifted, the block's face stays exact
  and only the pyramid's planes round. Turned, and turned then shifted, the
  block's face is `Pos` at the turned apex as well. In the frame before the
  movement the shape is the unplaced one, which answers. (a) alone cannot
  answer: `around` holds the four planes at the apex, but the stored
  shifted planes prove the stored apex off them, and 448.3 makes that a
  refusal. The fact is lost where the placement rounds each plane again,
  §17.3's bake into world coordinates. *After the base*: Q1936 landed at
  `3c4dbce5`, where `by_a_plane` reads `BakedCorner::solved_from` and
  places every speck. The fuse now refuses `DynamicRangeTooWide` on an edge
  a rounding long, which §294.21 calls the right answer for the placed
  numbers, and Q1962, after Q1954, carries (c) for it.
- **Q1737 / Q1958.** Answers at the base: Q1737's lane fixed it at
  `a53f0a15`, and Q1958 landed that at `e30afa64`. *The question it
  refused*: the order round the bar's bottom cap circle of the bar's seam
  corner against the nodes where the notch's sides cross that circle. Both
  solids are turned by `Isometry::rotation(y, π)`, and `sin π` rounds to
  `1.2e-16`, so the seam corner is a box `±8.5e-14` round `(-3, 0, 0)`. The
  circle is walked from the marcher's seed (`where_the_crossing_starts`,
  `fuse/arrange.rs` 8476 at `e68881ca`), at `(-3, 0, -3.7e-16)`, inside
  that box (the doc of `count_each_loop_from_the_node_round_its_start`,
  3326-3333). *The fix*: `the_node_round_the_start` (3349-3393) counts the
  loop from the one node whose box holds the start, only where the ellipse
  a plane cuts in a pipe turns on a radius at least 4 times the diagonal of
  the box round both and no other node is near. It reads the two surfaces
  only to bound the curve. *Which*: the fix carries no fact from the build.
  It makes the fuse's own choice of where the walk starts not matter, the
  fifth place in 448.6, as in Q1834. (c) would have answered it too: one
  `rotation(y, π)` moves both solids, and Q1737's row has the shape answer
  when two quarter turns move it instead.
- **Q1734.** Answers at the base: `zip_up_a_band` in `rcad-mesh`
  (`c35c5eb3`, `e43c8b14`, `45f6bca5`) meshes the band of torus wall a
  rounding wide instead of refusing it. *The coincidence behind the band*:
  the first bend is unplaced, its caps exactly on `y = 0` and `x = 0`
  (`a_quarter_bend` and `WhereABendSits::quarter` in
  `crates/rcad-topo/tests/rings/mod.rs`). The copy is placed by
  `Isometry::rotation(z, FRAC_PI_2)` or `rotation(z, PI)`, so the two bends
  are at two placements, and `FRAC_PI_2` is not `π/2`. In the stored input
  the copy's cap is a rounding off the first bend's. *Which*: **(d)**. The
  touch is only in what the user meant, and nothing the kernel built knew
  it. So here the two ways of 448.4 do not meet: the theory has nothing to
  carry, and coping with the band answers.

### 448.6 The count, and the row it files

One more place turned up that 448.1 did not list: **the fuse's own choice**.
The fuse makes a coincidence itself, by where it puts a test point, an aim or
the start of a walk, and the numbers then miss it. It is no fact of the
shape, so there is nothing to carry. The cure is to choose so that the
coincidence is not made, or so that it does not matter.

| Where the coincidence was known | Rows |
|---|---|
| (a) one solid's own faces | Q1839's intersection, unplaced and so shifted. It cannot be carried: the build stored a face proved off its own corner |
| (b) a crossing's own surfaces | Q1854 |
| (c) one movement for both solids | Q1765; Q1842 at both placements that refuse; Q1936 at all six; Q1958's shape, fixed another way; 3 of Q1839's 5 |
| (d) where the user put the two solids | Q1867, Q1734 |
| the fuse's own choice | Q1834; Q1842 turned; Q1958, fixed by making it not matter |

(c) covers three rows still open, Q1765, Q1842 and Q1936, and Q1958's shape
besides, so by 448.4 Q1954 files one writing row, tier 1, `opus`. Q1962,
filed by §294.21 to wait on Q1954, already carries (c) for Q1936's pyramid.
So the row filed here carries it for the other two: it waits on Q1962 and
reuses what that builds, closes Q1765 with a pin in
`a_bar_lying_along_a_plane_or_a_bar.rs` and Q1842 by turning round
`a_sound_half_bar_cut_flush_along_its_flat_face_refuses_when_turned`, and
says what Q1839's difference gives at the three placements (c) answers.
Q1765 and Q1842 wait on it, so that neither is taken alone with another
cure.

(b) covers one row, Q1854, where the fact is in hand when the question is
asked, and (a) none that can be carried; neither reaches three, and neither
gets a row. (d) is where the theory does not reach. The fuse's own choice
gets no row either: Q1834's own row keeps its cure, an aim or a test point
off the piece's diagonal (`holds.rs` 260-265, or `select::candidate_points`),
and Q1958 is fixed. 446.8, the trace of the two-thin-bars family, has not
landed, so the row names none of that family.
