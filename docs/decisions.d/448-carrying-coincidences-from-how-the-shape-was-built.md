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

*Being written. Q1954's first lane ran step 1 for all nine and stopped at its
budget; steps 2 to 4 are still to do for each row that refuses.*

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
