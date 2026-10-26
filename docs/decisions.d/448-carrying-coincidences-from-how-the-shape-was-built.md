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
put them, (d), which the theory does not reach. Then 448.7 found that the
shared movement (c) needs was known to the code calling the kernel and not to
the kernel, which keeps one composed placement per shell. A caller that fuses
first and places after gets every answer exactly today, so §17.3 stands, and
Q1962 and Q1963 are dropped. Since Q2575 the fuse carries (a) after all, for
the faces round a corner whose planes miss one point: not as a sign of the
stored planes, but as the planes those faces are drawn in (448.8).

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

### 448.7 Who knew the movement the two solids share — 2026-10-14

*Session `dispatch-2026-10-03-a8`, reading 448.5 and 448.6 against the code at
`469ec5c5`, before Q1962 and Q1963 were taken. The decision below is the
session's, by `CLAUDE.md`'s "Decide it yourself". A consumer that needs the
kernel to carry (c) turns it round.*

**What the kernel holds.** A solid keeps one placement per shell,
`ShellUse::loc` in `crates/rcad-topo/src/model.rs`, and `Model::place_solid`
composes each movement onto it (`used.loc = movement.compose(&used.loc)`). No
movement is kept apart from the placement it lands on. Some of the tests go
further and compose the shared movement themselves before the kernel sees
it: `a_block` in
`crates/rcad-topo/tests/a_broken_three_surface_corner_read_back_from_bytes.rs`
builds the block at `placement.compose(&own)`, while the pyramid and the half
bar are built where they stand and then moved by `place_solid`. Q1954's probe
for Q1765 builds the bar at `turn.compose(&shift(-1, 2, 1))`.

So in every row 448.6 counts under (c), what knew that one movement placed
both solids was the code calling the kernel. The kernel was handed the
composed numbers. For the kernel as it stands, that is (d), "known to nothing
the kernel built" (448.1). 448.5's "the fact was in hand until `compose`" is
true of the caller.

**What a caller can do today.** Fuse the two solids where they were built, and
move the answer after.
`a_cut_made_where_the_solids_were_built_and_then_placed_answers_at_every_placement`,
in the same test file, does that for the pyramid and for the half bar, each
with `THE_HALF_PAST_THE_SEAM`, by difference and intersection, at the four
placements of §294.8, under `certified`. All 16 answer, validate, hold what
the two hold, and leave nothing in the log, and each half pyramid is 5 faces
and 16 sides. Placed first, the same pyramid refuses `DynamicRangeTooWide` at
all six placed fuses
(`a_sound_pyramid_placed_and_cut_through_its_apex_keeps_a_speck_of_the_cut`).
`mise exec -- just t -p rcad-topo -E
'test(/^a_broken_three_surface_corner_read_back_from_bytes::/)'`: 14 tests run,
14 passed.

**What carrying (c) inside the kernel would take.** The model would keep each
movement apart on the shell: `ShellUse`, and everything that composes, writes
or reads a shell's placement. The fuse would work in the frame of the
movements both solids share and give the answer that movement, where §17.3
settled that it "bakes placement into world coordinates at entry". And it
would help only a caller that moves both solids by `place_solid` and fuses
after, not one that composes its own placements, as these tests do. None of
that is in Q1962's write set, which is `fuse/`, one test file and §294's file.

**Settled: §17.3 stands, and the kernel does not carry (c) until a consumer
needs it.** For the numbers the kernel is handed, the pyramid's refusal is a
limit (§294.21). The answer of those numbers holds an edge a rounding long,
which `f64` cannot hold beside coordinates near 10. The answer the designer
meant is the answer of other numbers, which the kernel is not given. Giving
it takes a new input, the movement handed in apart, so it is a feature, and
§13's rule 3 defers a feature to the milestone where its consumer appears.
This is not "no caller hits it": any caller can move two solids by one
movement and fuse them after. The point is that no answer the kernel gives to
the numbers it is handed is wrong, and a caller who knows the movement is
shared already has an exact road, which the test above pins.

**What it does to the rows.**

- **Q1962** is dropped. Fused first, its pyramid answers. Placed first, it
  refuses with the limit §294.21 named, and its pin stays as it is.
- **Q1963** is dropped. Q1765 and Q1842 stop waiting on it and go back to
  what their own rows ask: for the numbers the kernel is handed, is the
  refusal a gap or a limit? Q1842 turned is refused at the fuse's own test
  point on a diagonal (448.5), whose cure is in Q1834's row, so Q1842 waits
  on Q1834. Q1765 is ready.
- **Q1839** keeps its row. Its own finding in 448.5 is (a), a face the build
  stored proved off its own corner. The 3 of its 5 that 448.5 gave to (c) are
  (d) for the kernel, as above.
- **Q1969** keeps its row. It names (c) as its pattern, but the cure it asks
  for is `Arrangement::the_node_round_the_start` taking a cone as well as a
  pipe. That is the fuse's own choice, as Q1958's cure was.

**Mattias, 2026-10-14**, asked whether a consumer moves several parts by one
movement and fuses them after: "Probably not yet, but would cloning and moving
one part using an matrix or polar pattern. e.g. FreeCAD:s PolarPattern
functionality match the description?"

**A pattern is not (c).** A pattern moves one part by several movements, one
for each copy, and leaves the part the copies are cut into where it was. (c)
is several parts moved by one movement. A copy and that part share no
movement. Two copies share one only in that each neighbouring pair is the
first pair turned, and the first pair is one part left where it was beside a
turned copy of it, which is (d): the touch is made by the step the user chose
and the shape of the part. 448.5 already has a polar pattern of two, Q1734,
whose second quarter bend is the first turned by `rotation(z, FRAC_PI_2)`. It
is filed under (d), and it answers at the base because `zip_up_a_band` meshes
the band rather than refusing it. A pattern makes a consumer for (c) only where
the patterned part is then moved together with another part by one movement
and the two are fused after, and there the caller can fuse first, as above. So
nothing turns the decision round yet.

**Where (c) would be needed.** Two pads sketched side by side on one datum
plane turned 30 degrees, `0..20 × 0..10` and `20..30 × 0..10`, touch exactly
along `x = 20` in the sketch's frame. A caller that builds each pad there,
turns it, and fuses it gives the kernel two solids under one turn that touch,
which is the shape Q1765 and Q1842 refuse. This example itself has not been
run. Fusing the two pads in the sketch's frame and turning the answer after
is the road above. A caller that models one feature at a time cannot take it:
when the second pad arrives, the first has already been fused into a body in
world coordinates. That caller is (c)'s consumer. For it the movement would
have to stay on the first pad's faces through the fuse that put them in the
body, and not only on the shell as above. That is a reading, not measured.

*Mattias, 2026-10-14: "Ok, so that is absolutley something we must solve, but
not just now."* So it is a tier 4 row, parked on this section, which §230.1
keeps parked until `docs/consumers.md` names a caller that builds features on
a tilted plane one at a time.

### 448.8 A corner whose planes miss one point, read as the point they all run through — 2026-10-25

*Queue row Q2746, from Q2575, which built the reading (merged at
`955f07d6`). Written down from the code at `2fbc7423`. Amended by Q2777, read
at `d89061f2`, for what Q2744 changed (lane tip `1c5a029b1`, merged at
`615ef838`): a face that a face of another solid cuts is read round that
face's crowded corners too, and `by_a_plane` reads a piece inside the room of
such a corner of the solid it is placed against. The doc comments of
`as_read_round_a_corner`, `ACornerReadAsOnePoint`, `by_a_plane` and
`BakedFace::corners_of_the_solids_cutting_it` say how. Amended by Q2815, read
at `c3b3afeff`, for which three planes `bake` solves such a corner from (the
paragraphs "Since Q2815" below); the doc comments of `derive_a_corner` and
`ACornerReadAsOnePoint::lets_a_crossing_in_through_a_face_it_does_not_cut`
say how.*

**What the reading is.** Take a corner of the input with four or more planes
round it (`BakedCorner::around`), one of which misses the point `bake` solved
the corner at. It is read as one point that every plane round it runs
through. Each plane round the corner is read as its own copy, moved without
turning, so that it runs through that point. Every other plane is read where
it is. This is `ACornerReadAsOnePoint` in `fuse/arrange.rs`. It reads only a
node whose whole box lies inside the corner's room, which is the corner's box
grown by the room `bake` asked each plane round it about the corner with
(`BakedCorner::room_its_planes_were_asked_with`). Two places read it:

- `Arrangement::as_read_round_a_corner` puts two nodes in order along a
  crossing of a face round the corner. The face's plane must be round the
  corner, and each node must be the corner or the meet of the crossing's two
  planes and one third plane. It is asked first, before the boxes, in
  `as_proved_before_any_policy` and in `which_comes_first`. The boxes hold the
  true points, and the order they prove can be one that no straight drawing of
  the face keeps beside the others.
- `by_a_plane` in `fuse/select.rs` reads a flat piece with straight sides the
  same way when the piece's own solid has such a corner, the piece's plane is
  round it, and every corner of the piece is inside its room
  (`TheCornersOfAPiece::ReadRoundACorner`). The corner need not be a corner of
  the piece.

`ACornerReadAsOnePoint::of` gives no reading, and the fuse asks as it did
before, for a corner with fewer than four planes round it, a curved surface
among them, no `solved_from` or no room (a corner read out of the model), a
determinant of the three solved-from normals not proved nonzero, or every
plane round it running exactly through the point solved.

**Since Q2744, a face of the other solid is read round the corner too.**
`gather` in `fuse/cuts.rs` hands each face that a face of another solid cuts
every corner of that face with four or more surfaces round it
(`hand_on_its_crowded_corners`), and the face keeps them as
`BakedFace::corners_of_the_solids_cutting_it`. A wedge whose edge runs up
through the placed pyramid's apex has no corner there, but each wedge face
that a side face cuts is handed the apex. The same two places read it:

- `as_read_round_a_corner` tries the face's own corners first, then the ones
  handed on. Round a corner handed on, the face's own plane is not round it,
  so it is read where it is, and no node is the corner: each must be the meet
  of the crossing's two planes and a third, inside the corner's room. The room
  is asked before the reading is made, so a face whose nodes all lie far from
  the corner never reads it
  (`a_cube_top_a_pyramid_pokes_through_is_split_without_reading_the_apex_far_above_it`,
  in `fuse/arrange.rs`).
- `TheCornersOfAPiece::of` tries such a corner of the piece's own solid
  first, then one of the solid the piece is placed against (`theirs`), and
  only then the meets of three planes in the true planes. Round a corner of
  the solid placed against, every corner of the piece must be a meet of three
  planes, the piece's among them, inside the room, and the piece's plane need
  not be round the corner. That solid is read as moved too:
  `placed_by_the_planes_of` reads each of its corners as the meet of the three
  planes `bake` solved it from, with the planes round the corner moved, and a
  plane of it round the corner places the piece as moved.

So the speck by the apex that the lines where side planes cross a wedge face
cut out of it is drawn with the side planes moved, as the side faces are, and
placed against the pyramid as drawn.

**Why.** Four planes written in `f64` need not meet in one point (§294.19).
Placed by a shift, the sound pyramid of
`a_broken_three_surface_corner_read_back_from_bytes.rs` has its four side
planes meet three by three in four apexes a rounding apart. `bake` solves the
apex from three of them (`BakedCorner::solved_from`, Q1911, §294.20). So on
each side face the corner is on two of the lines the other side planes cross
it in, and a rounding off the third. On the `−y` face at the plain shift, as
the test below found it, that third line is the face's own edge with the `−x`
face. The face's outline ends that edge at the apex, while the planes put the
apex off its line. A wedge plane run between
two of the apexes then crosses those lines in an order that no straight
drawing with the corner on all three keeps. Each order was proved exactly, of
the true points, and together they are not the orders of one drawing, so the
face could not be drawn.
`a_placed_pyramid_reaches_the_range_check_where_a_wedge_plane_runs_between_two_apexes`
runs 39 fuses: `Difference` under `certified`, with each of three wedges at
13 placements. Before Q2575, 23 of them were refused
`CannotTellTheCellsOfAFaceApart`, and each of the 23 has a wedge plane between
two apexes. The test's doc walks through the four lines. Q2333's lane let the
planes overrule the boxes on that face, and then turned two cuts round as
well, and each still refused. The orders of the true points are what no
drawing keeps. So Q2575 changed which planes the face is drawn in, not how the
orders are asked.

At `2fbc7423` none of the 39 is refused that way. Of the 27 with a wedge
plane between two apexes, 21 are refused at the range check 448.7 settled for
the cut through the apex, 4 answer, and 2 are refused `Undecidable` on a
speck of a wedge face (below). Of the 12 with none, 9 are refused at the range
check and 3 answer. At `d89061f2`, since Q2744, the test's doc says those 2
are refused at the range check, and its tally is 21 refused there and 6
answering of the 27, and 9 and 3 of the 12. `mise exec -- just t -p rcad-topo -E
'test(/a_placed_pyramid_reaches_the_range_check_where_a_wedge_plane_runs_between_two_apexes|a_far_apex_whose_side_planes_miss_one_point|keeps_a_speck_of_the_cut$|tells_its_cells_apart$/)'`:
4 tests run, 4 passed. The second of these is the reading's own pin, in
`fuse/select.rs`. It reads the apex of a far pyramid whose near face misses
it by `1e-8`, against planes a few roundings round it in every direction.

**The arithmetic, all exact.** Write `K` for the point `bake` solved the
corner at, where its three solved-from planes meet, and `D` for the
determinant of their three normals. The reading works in the corner's own
frame. A point `Y` is `Z = D·(Y − K)` there, and a plane `n·Y + d = 0` is
`n·Z + ω = 0`, with `ω = D·(n·K + d)`: `D` times the plane's value at `K`. That
is the four by four determinant of the three solved-from planes and this one,
which `three_planes_against_a_fourth` works out in
`rcad_predicates::expansion` without rounding. No coordinate of `K` is ever
rounded. A plane round the corner is read with `ω = 0`, so through `K`, and
every other plane with its own `ω`.

- The corner is read at `Z = 0`. Its side of a plane is the sign of that
  plane's `ω` as read.
- A meet of three planes is read where the three meet at their `ω` as read.
  Its side of a fourth plane is the sign of the four by four determinant of
  the four at their `ω` as read, times the sign of the determinant of the
  three normals.
- Either side is turned back into a side in space by the sign of `D`, which
  the reading keeps as `handed`.
- Of two nodes on the line where planes `S` and `Q` cross, which comes first
  is read from the side this node is on of the plane that makes the other
  node, by the rule `where_a_corner_off_a_line_comes_along_it` follows
  (§294.20).

The struct holds `solved_from`, the three planes; `around`, every plane round
the corner; `handed`; and `room`. Where a product of the expansions would
round, the reading gives `None` and the fuse asks as it did before.

**Why it is not a tolerance (448.3).** Two things need saying: whether the
reading takes anything as near enough, and whether it overrules a surface that
proves the corner off it.

- Every sign it gives is exact, of real planes. Each moved plane is a real
  plane, so each node read is a real point. The corner is `K`, and the meet of
  two planes round the corner and a third is where the third crosses the line
  through `K` along the two. So the orders along every crossing of a face
  round the corner are the orders of one straight drawing. On a face of the
  other solid read round the corner, the face's own plane is not moved, and
  each node is the meet of it and two other real planes, moved or not, so a
  real point too. No sign of a stored
  plane is taken as near enough to be another sign. Inside the room, a stored
  plane round the corner is asked only once, in `of`, to find whether it
  misses `K`.
- The width it works within is not its own. It is the room `bake` already
  asked each plane round the corner about the corner with: what
  `Model::validate_solid` grants a corner off a plane, plus what writing the
  corner down cost. `as_read_round_a_corner`'s doc says the moved planes are a
  rounding from the true ones, which is that room. It also says a node further
  off is put in order by its true point and is as far from every node read as
  the room is wide.
- 448.3's check still stands, in `bake`. 448.3's example of it is
  `derive_a_corner` asking each surface round the corner about the corner,
  grown by that room, and there a surface that proves the corner off it is
  still a refusal, before any reading is made. So the reading moves only
  planes that passed the check. 448.5's Q1936 entry asked more narrowly, of
  the stored planes at the stored apex itself, found them off, and said that
  (a) alone cannot answer. That is still true of the stored planes. The
  reading does not say they run through `K`. It draws the faces round the
  corner in planes that do.

So the reading carries 448.1's (a), that a corner lies on the surface of every
face round it, in a form 448.5 did not try. It is carried not as a sign of the
stored planes, which exact arithmetic denies, but as the drawing that the
faces round the corner are cut in. The fact comes from how the shape was
built: one vertex with four faces round it, the pyramid's apex, which the
outline of each of those faces states. The placement rounded each plane's
offset on its own, and the reading carries the one vertex past that rounding.
It comes from the solid's own faces, so it is (a) and not (c). Read on a face
of the other solid since Q2744, it still carries only that fact of the
pyramid's: the face's own plane is not moved, and nothing says it runs
through `K`. It needs no
movement kept apart, which 448.7 settled the kernel does not keep. Its answer
is a certified answer and adds no provenance record (448.3), and the rounding
in the stored offsets stays representation wobble (§17.3).

**What Q2744 closed.** Two items this section first left are done, both at
`1c5a029b1`:

- The 2 of the 39 that were refused `Undecidable` on a speck of a wedge face
  by the apex. That face had no corner there, so it was drawn in the true
  planes. Q2744 draws it round the apex and places the speck against the
  pyramid read round it, as above, and both are refused at the range check
  under `certified` now.
- The 5 of the 8 placements where the order looped by the apex that
  `tolerant(100)` refused `ResultIsNotManifold`: Q2745, which closed with
  Q2744. Since Q2744 all five answer by every rule
  (`a_sound_pyramid_placed_and_cut_by_a_wedge_through_its_apex_tells_its_cells_apart`,
  run above).

**Since Q2815, which three planes the corner is solved from is a choice.**
`bake` solved a corner with four or more planes round it from the three whose
meet has the least box, and the reading moves the others through that meet.
So which three it takes decides where the planes of the other solid cross the
moved ones, and that can change the answer. Turned a tenth of a radian about
`(−2, 1, 0.5)`, the sound pyramid cut through its apex by the wedge on one
side of the `x` axis shows it. Exact fractions on the printed planes put the
point where the wedge's edge crosses the `+x` side plane behind each of the
other three, behind `+y` by `1.8e−17` of that plane's value. So in the true
planes the edge comes into the pyramid through `+x`, and the intersection is
four faces whose top corner is that point. The least box is where `−y`, `+x`
and `−x` meet. Read round it, `+y` is moved, the point lands `2.6e−16` in
front of the moved `+y`, and the edge comes in through `+y` instead. Neither
wedge face cuts `+y` further down, so each cuts off the top of `+y` a triangle
whose corners lie within `2e−15` of one another. The intersection kept it as a
fifth face, 5 faces and 18 sides under `tolerant(100)`, and the difference
kept a stretch of it, 8 faces and 34 sides. `certified` refused both at the
range check, where before Q2575's merge `955f07d6` it answered them with
nothing in the log, in 4 faces and 12 sides and 7 and 28.

`derive_a_corner` now keeps the least box unless its reading lets a line where
two planes of the fuse cross come into the corner through a face that one of
the two does not cut further from the corner. A plane cuts a face further from
the corner when it parts the far ends of the face's two edges at the corner.
Then `derive_a_corner` takes the next triple by box whose reading does not let
such a line in, and the least box when every triple does. Each triple is
asked about every surface round the corner, as the least box is (448.3), and
one a surface proves off is passed over. Under the pyramid above it takes `+x`,
`+y` and `−x`, with `−y` moved. Read so, the edge comes in through `+x`, as in
the true planes, and both wedge faces cut `+x` further down. The difference
and the intersection answer in 7 faces and 28 sides and 4 and 12, with nothing
in the log, under `certified` and `tolerant(100)` alike
(`a_pyramid_turned_and_cut_on_one_side_of_the_x_axis_answers_as_the_true_planes_do`).
The union there answers with nothing in the log too, which `certified` refused
at the range check before
(`a_placed_pyramid_cut_by_a_wedge_through_its_apex_answers_under_tolerant_where_q2583_found_it_refused`).

**Why a line let in that way is a speck.** Two planes of the other solid
whose line runs within a rounding of the corner meet the face that line comes
in through in two lines from where it comes in. A plane that cuts the face
further from the corner runs its line across the face. Where both do, the two
lines part where the edge comes in, and the answer's corner is that point. A
plane that does not cut it keeps its line within a rounding of the corner. It
cuts off a piece of the face whose sides are a rounding long, which the answer
keeps and `certified` refuses at the range check.

**What it reads.** `the_plane_cuts_the_face_away_from_the_corner` in
`fuse/arrange.rs` works from the normals alone, as exact determinants, so its
answer is the same in every reading and in the true planes. Where the line
comes in is asked of the reading, `ACornerReadAsOnePoint::side_of_a_meet`,
exact as above. Any sign that cannot be worked out exactly, a face round the
corner with other than two edges, or a nought determinant counts as letting a
line in, so that triple is passed over. Three things narrow where it is asked:

- Facing out of the solid is what tells a line coming into the corner from one
  going past it. So it is asked only round a corner whose every surface is the
  plane of one of its faces, and which `the_planes_round_a_convex_corner` in
  `fuse/bake.rs` proves convex from the stored corners and planes. That holds
  when the far end of every edge from it lies behind every face plane round it
  but the two the edge runs between. Any other corner keeps the least box.
- The planes it asks about are those of every flat face of every solid in the
  fuse (`the_plane_of_every_face`) that run through the corner's room, less
  the ones round the corner, either way round. So `Scratch::split_both_groups`
  bakes the arguments and the tools in one call, and each solid's corners see
  the other's planes.
- The joined faces of a wholly flat solid, `corner_of_every_vertex_left`, make
  no choice and keep the least box, as before Q2815.

**Why it asks the reading and not the true planes.** The reading is what the
face is drawn in. Shifted to `(7, −5, 2.5)` after a tenth of a radian about
`(1, 2, 3)`, the edge of the wedge round the `x` axis comes in through `−y` in
the true planes, and neither wedge face cuts `−y` further down: a speck the
true planes make. Read round the least box, it lets in no such line, and the
difference answers under `certified` with nothing in the log, as at the base.
Q2815's lane asked of the true planes at `2f0c46e2e`. There `bake` took
`−y`, `+x` and `−x`, the edge came in through `−y` as read too, the answer kept
the true speck, and `certified` refused that difference at the range check
(`a_sound_pyramid_placed_and_cut_by_a_wedge_through_its_apex_tells_its_cells_apart`
failed there).

**Why it is not a tolerance (448.3).** It takes no sign as near enough to
another. It chooses only among triples whose corner every surface round it
already passed 448.3's check for, and each corner read is the meet of three
real planes. The answer is a certified answer and adds no provenance record.

`mise exec -- just t -E
'test(/a_broken_three_surface_corner_read_back_from_bytes::/)'`, at
`c3b3afeff`: 28 tests run, 28 passed.

**What it leaves.** A face of the other solid is read round a corner only
where a face round that corner cuts it. Read round a corner of the piece's own
solid, a plane of the solid it is placed against that is one of the planes
round the corner, either way round, places nothing in `by_a_plane`, since the
reading moves it and that solid as read does not
(`TheCornersOfAPiece::sides_of`, and the Q2575 paragraph of `by_a_plane`'s
doc). Read round a corner of the solid it is placed against, that solid is
read as moved too, so its planes round the corner do place it. Each of the
following already has an open row or a settled place, so this section files
nothing:

- The cut through the apex, Q1936's shape. Its speck lies on the block's face,
  which belongs to the other solid. Since Q2744 the side faces that cut that
  face hand it the apex, but the apex `bake` solved is a rounding off the cut
  (the test's doc), so the side planes, moved through it or as they stand,
  cross the block's face in lines that do not all meet in one point. So it
  stays where 448.7 settled it. Under `certified` it is refused at the range
  check for the numbers the kernel is handed, and under `tolerant(100)` it
  answers
  (`a_sound_pyramid_placed_and_cut_through_its_apex_keeps_a_speck_of_the_cut`,
  run above).
- The second of Q2744's two, the `−y` wedge shifted after the turn 0.7 about
  `(0.3, −1, 2)`, one of the 8 placements: `certified` refuses it at the range
  check by every rule, and `tolerant(100)` answers the intersection and
  refuses the difference and the union `TheFuseWroteABrokenAnswer`, a face
  whose outline winds against its normal: Q2783.
- An order along a crossing by a node outside every corner's room, or by a
  corner with exactly three planes round it a rounding off the crossing's
  line, is still asked by `as_the_planes_say`, against each cut by that cut's
  plane alone: Q2750.
- 21 fuses of Q2583's 672-fuse sweep answered at `f7bde708` and refuse at
  `5afdb494`. The fuse changes between those include Q2575's merge, and they
  are not bisected, so nothing here says the reading made them: Q2758. Q2758
  has closed since. Its test says `tolerant(100)` answers all 21, and that
  the two `certified` answered at `f7bde708` were refused at the range check
  from Q2575's merge on. Those two are the difference and the intersection of
  the pyramid above, and since Q2815 all three rules there answer with nothing
  in the log under both policies. The other 18 are refused at the range check
  under `certified`, as at `f7bde708`, and are not traced
  (`a_placed_pyramid_cut_by_a_wedge_through_its_apex_answers_under_tolerant_where_q2583_found_it_refused`).
- A corner of the joined faces of a wholly flat solid makes no choice. Whether
  one needs it is not measured, and no shape found so far does.
