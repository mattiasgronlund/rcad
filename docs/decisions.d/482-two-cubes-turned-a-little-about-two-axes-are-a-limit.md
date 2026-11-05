## 482. Two cubes, one turned a little about two axes and moved onto the other's edge, are a limit

Queue row Q3205, from Q3195 (and Q2254, whose probe first met it).

**The shape.** A block `add_cube(10.0)` at `Scale::millimetres(100.0)`, and a
second `add_cube(10.0)` placed by `Isometry::try_from(([tx, 0, 0], q))`. `q` is
the turn `0.0043` about `z` and the turn `0.0081` about `x`, put together
either way round. `tx` is `5` or `5.0000001`. As meant, the turned cube's
corner sits on the block's bottom front edge, the two skins cross at that one
corner, and nothing in any answer is small.

**What happens.** Under `Policy::certified()` 18 of the 24 fuses are refused
`DynamicRangeTooWide`. The 6 that answer have the turn about `x` first and the
move by exactly `5`. Under `Policy::tolerant(100.0)` all 24 answer and
validate.

**Why: the numbers the model holds have the edge.** Each placed plane's offset
is `n . t`, rounded. The turned cube's bottom, front and left planes then meet
outside the block: past `y = 0` by `3.5e-19` to `4.8e-19` at three placements,
and below `z = 0` by `1.7e-21` at the fourth. Where a face of one cube crosses
a face of the other near that corner, the line they cross along runs between
two meets of planes a rounding apart, and that line is an edge of every
answer: the union, what the two share, and each difference. Worked out exactly
from the stored numbers, every answer's shortest edge is `1.7e-21` to
`1.1e-16` long. The cubes' reach is `15`. That is a spread of `1.4e17` to
`8.7e21`, where a model supports `1e11`.

**This is a limit, not a gap** (§16.10). The answer does not exist without the
edge, so a fuse of these numbers cannot give one without it. The families of
fix considered, and why each fails:

1. Another order of the arithmetic, or more precision. The edge is in the
   exact answer of the stored numbers, so no better arithmetic removes it.
2. Scaling the whole shape by a power of two. The edge and the coordinates
   scale alike, so the spread stays.
3. A tolerance, taking two points a rounding apart as one. That works, and it
   is `Policy::tolerant(100.0)`: all 24 answer. It is a different policy, not
   a fix of the certified one.
4. Keeping from the placement the fact that the turned corner lies on the
   block's edge. That answers the shape as meant, not the numbers stored.
   Carrying such a fact is §448's question, queue row Q1998, parked on §448.7.
   If it lands, these refusals are its first customers.

**Same limit elsewhere.** §294.21 is the same limit for a pyramid placed and
cut through its apex: the half pyramid keeps an edge a rounding long. Its
refusals under certified stand as the limit. Read §294.21 with its later
correction, which calls a different triangle case a gap, so the two are not
mixed up. Q3191's pin of two pointed cones both 8 high, one leaning, refused
`DynamicRangeTooWide` with corners shown `4.1e-16` apart, is a sibling case
(`crates/rcad-topo/tests/a_funnel_with_a_leaning_spout.rs`).

**Pinned in** `crates/rcad-topo/tests/two_cubes_one_turned_a_little_about_two_axes.rs`.
The test
`every_answer_of_the_stored_planes_has_an_edge_a_rounding_long_where_the_skins_cross`
works out the exact edge. Q2254's probe, not committed, counted 15 refusals
where the tests count 18; how it put the two turns together is not known.

**What this leaves.** Nothing to build. A designer who needs these answers
under certified places the second cube on the edge by a route that does not
round (§448's carrying), or uses the tolerant policy.
