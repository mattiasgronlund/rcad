## 284. A plane leaning through a bar's rim start: the turned one misses it, the exact one names two arcs as one — 2026-09-26

*Queue row Q127, from §252.8. Lane `lane-q127-rim-start-order`, off
`1a3fd648`. §252.8 found all six fuses of a bar and a plane leaning along
`z = y` through the bar's rim start `(5, 0, 0)` refusing `Undecidable` on
"order along a crossing". **That refusal is real, and it is a gap, but the
plane that meets it does not pass through the rim start.** A block turned by
`−π/4` about `x` misses it by `2.5e-15`, and what cannot be ordered is two
different points round the bar's cap circle, one of them in a box too wide
along `x`. **A plane on `z = y` to the bit is ordered and meets a second gap
instead**: an arc of the cap circle and an arc of the ellipse share both ends
on the wall and come out as one edge. No code changed. Both are pinned in
`crates/rcad-topo/tests/a_plane_leaning_through_a_bars_rim_start.rs`.*

### 284.1 The two planes

§252's tests build no plane through the rim start, so this lane built two.

- **A turned block**: a cube forty on a side, `[−20, 20] × [0, 40] × [−20, 20]`,
  turned by `−π/4` about `x`. Its face `y = 0` then leans along `z = y`, and
  the rest of the block is far from the bar. This reproduces §252.8: all six
  certified fuses refuse `Undecidable` on "order along a crossing".
- **A prism described face by face** with `SolidBuilder`: the part of
  `[−20, 20]³` with `z ≤ y`. Its leaning face is
  `Plane::from_point_and_normal(origin, (0, −1, 1))`, whose two components are
  equal, so it is `z = y` exactly and runs through `(5, 0, 0)` exactly.

The bar is `add_round_bar(5, 10)`. A temporary `eprintln!` in
`Arrangement::order_along` printed the face, the crossing's partner, the
walk's start and both nodes whenever the sweep came back undecided. A second
one, in `Assembly::darts_along_each_edge`, printed every edge claimed other
than twice. Both ran under
`mise exec -- just t -p rcad-topo --test a_plane_leaning_through_a_bars_rim_start --no-capture`
and both are removed.

### 284.2 What cannot be ordered, and against what budget

The fuse bakes the turned face as

```
normal (0.0, −0.7071067811865475, 0.7071067811865476), offset −1.7763568394002505e-15
```

so its two components differ by one ulp and its offset is `−2⁻⁴⁹`, not `0`.
It crosses the cap plane `z = 0` on the line `y = 2.512e-15`, not `y = 0`,
and the distance from `(5, 0, 0)` to it is `1.78e-15`.

All six fuses stop at the same place:

- **The face** is the bar's bottom cap, `z = 0`, with outward normal
  `(0, 0, −1)`.
- **The crossing** is the cap circle, where that plane meets the wall
  `x² + y² = 25`. It is closed, so it is ordered round from where the walk
  starts, `(0, −5, 0)`.
- **The two points** are the rim start, `Exact([5, 0, 0])`, the input vertex,
  and the meet of the turned face, the cap plane and the wall, which
  `meet_of_three_surfaces` hands back as
  `[4.999999999999997, 5.000000000000003] × [2.51214793389404e-15, 2.5121479338940407e-15] × [−0, 0]`.
- **The enclosure** of the sweep is `3.552713678800501e-14`, `Measures::Volume`.

Under `Policy::certified()` that refuses. Under `Policy::tolerant(100)` the
wobble is `3.55e-14 / 100 / 100 = 3.55e-18` against a budget of
`64·ε·100 = 1.42e-12`, so the policy reads it as `Zero` and folds the meet
into the rim start. The cap and the wall then end their edges at `(5, 0, 0)`,
the turned face at `(5, 2.512e-15, 0)`, and every fuse refuses
`ResultIsNotManifold`: `count: 3` for the union and the difference with the
bar as the argument, `count: 1` for the other four. The edges printed as
claimed once join the far meet `(−5, 2.512e-15, 0)` to one or the other of
those two near corners. The ones claimed three times are the cap circle's
arc, claimed by the cap and twice by the wall, which looks like 284.5's
collision again.

### 284.3 Why it cannot be ordered

The brief's hypothesis was that the two points might be one point, or two
meets whose boxes overlap. **Neither holds for the turned block.**

- They are two points. In exact rationals from the stored bits, the meet is
  `(5 − 6.31e-31, 2.5121479338940403e-15, 0)`. The box holds it, and its `y`
  side is one ulp wide and above zero, so the box does not hold the rim start.
- The boxes do not overlap, for the same reason.

What refuses is the box's width along `x`, as in §281.1. That side is `3` ulps
of `5` either way, `5.3e-15` in all, while the two points are `2.5e-15` apart
along the circle. The sweep round the circle from `(0, −5, 0)` is
`5·(x₁ − y₁ − 5)` for the meet `(x₁, y₁)` and the rim start. Its exact value is
`−1.256e-14`. Over the box as built it runs from `−2.59e-14` to `+7.6e-16`,
which straddles zero. The kernel's own sum in intervals adds its roundings and
reports `3.55e-14`.

`which_comes_first_round_a_crossing` has an exact level, but it needs exact
coordinates, and a meet of three surfaces is a `Constructed` box with none.
So the interval level is the only one that runs.

The numbers in this section came from a Python `Fraction` script over the
printed bits, `python3 scratchpad/q127.py`, which is not in the tree.

### 284.4 A gap

**The six answers exist.** The plane `z = y` cuts from the bar the wedge
`0 ≤ z ≤ y`, whose volume is `∫∫ y dA` over the half disc `y ≥ 0`, which is
`2·5³/3 = 250/3 = 83.333…`. The top cap plays no part, since `z ≤ y ≤ 5 < 10`.
The bar is `250π = 785.398…`. With `K` the knife's own volume, `64000` for the
turned cube and `32000` for the prism:

| fuse | volume by hand | turned block | prism |
|---|---|---|---|
| union, either way round | `K + 250π − 250/3` | `64702.065` | `32702.065` |
| intersection, either way round | `250/3` | `83.333` | `83.333` |
| bar less knife | `250π − 250/3` | `702.065` | `702.065` |
| knife less bar | `K − 250/3` | `63916.667` | `31916.667` |

Each is one solid. The turned plane is off `z = y` by `1.8e-15`, which moves
these by far less than the mesh's bound. The prism's intersection measures
`83.284 ± 0.496` against `83.333`, which is inside that bound.

**The order exists and exact arithmetic decides it.** The two points are
different points on the circle, and the sweep is `−1.26e-14`, not zero. So by
§16.10 the refusal is a **gap**. The function that refuses is `must_decide`
in `Arrangement::order_along` (`crates/rcad-topo/src/fuse/arrange.rs`),
reached from `nodes_along` placing the nodes of a closed crossing. What it is
handed is `round_a_closed_crossing`'s `Undecidable`.

The families of fix considered:

- **A narrower box for the meet.** The meet's `y` is known to one ulp, so
  `25 − y²` rounds to `25` and a correctly rounded square root gives
  `x ∈ [5 − 1 ulp, 5]`. Over that box the sweep runs from `−1.70e-14` to
  `−1.26e-14`, which decides it. Any half-width under `2.83` ulps of `5`
  decides it. This is §281.2's family, and §281.3's warning applies to it:
  narrowing a meet's box moves which nearby points are folded. This is the
  one that reaches the refusal where it is made.
- **A better-conditioned question.** Near the rim start the circle runs along
  `y`. Two points on it with `x > 0` come in the order of their `y`, and the
  meet's `y` side is one ulp wide and above zero. This needs a rule for when
  such a comparison stands in for the sweep, and that rule is not written.
- **An exact sign.** The meet is algebraic: its `y` is rational, and its `x`
  is `√(25 − y²)`. The sign of `5·(x − y − 5)` can be settled exactly by
  squaring. This is the largest of the three.
- **A tolerance.** Not a fix (§17). A tolerant policy already takes that route
  in 284.2, and it lands in a model it cannot assemble.

The fix belongs to whoever owns `arrange.rs` or `rcad-geom`'s
`meet_of_three_surfaces`. **It is not the whole of it**: 284.5 is what the
exact plane meets once the order is known, and the turned block will
probably meet it next. That is unmeasured: it cannot be run without the fix.

### 284.5 The exact plane: two arcs on one wall named as one edge

With the prism, the meet is `(5, 0, 0)` to the bit, the same point as the rim
start, and nothing refuses to order. Under both policies alike:

- both intersections answer, validate, and measure `83.284 ± 0.496` against
  `250/3`;
- the union and both differences refuse `ResultIsNotManifold { count: 4 }`.

The edge printed with four faces joins `(5, 0, 0)` to `(−5, 0, 0)` in all
four. With the bar as the argument it is carried by the cap plane and the
wall, from the bar's bottom circle, and is claimed by the cap, by the wall
twice and by the leaning face. With the prism as the argument it is carried
by the leaning face and the wall. The union then has it claimed by the
leaning face, the cap and the wall twice, and the difference by two leaning
faces, the flipped cap and the wall.

Two arcs run on the wall between those two corners: half the cap circle, and
half the ellipse the leaning plane cuts in the wall. The circle and the
ellipse cross there and nowhere else. **An arc of each comes out as one
edge.** The likely reason is in §245.4's fix, commit `52784689`. It names an
arc by its corners, by its curved surface either way round, and by which of
its circle's two arcs it is, and its message says "The plane is left out, as
`which_curve_it_is`'s doc requires". §245.4 closed the case of two
*different* curved surfaces through the same two corners. This is one curved
surface cut by two *different* planes. This lane read that commit and did not
trace `Edges::of`, so "the plane is left out" is the likely cause and not a
measured one. It is a **gap**, since the table in 284.4 holds.

### 284.6 What this leaves

- **Order the meet and the rim start round the cap circle** (284.4). The
  first family is the most direct. Whoever does it answers for what §281.3
  found narrowing moves. `crates/rcad-topo/src/fuse/arrange.rs` or
  `crates/rcad-geom/src/three_surfaces.rs`. Its check is
  `a_block_turned_to_lean_through_a_bars_rim_start_cannot_order_two_points_round_its_cap`,
  whose messages carry the volumes.
- **Name an arc by its plane as well** where two planes cut one curved
  surface between the same corners (284.5). `crates/rcad-topo/src/fuse/assemble.rs`.
  Its check is
  `a_plane_exactly_through_a_bars_rim_start_answers_all_six_fuses_right_under_both_policies`.
  `which_curve_it_is`'s doc says why the plane is left out, and it has to be
  read before this is changed. **Done by §330**, which renamed the check from
  `a_plane_exactly_through_a_bars_rim_start_is_ordered_and_two_arcs_on_its_wall_get_one_name`
  when all six fuses of the exact plane came to answer.
- **§39.4's table** has one row for "order along a crossing", the turned
  notched bar. This is a second cause of that refusal, and the table should
  gain a row when either item above lands. `docs/decisions.d/039-*.md` is
  outside this lane's write set.
- §252.8's sentence "a plane leaning through a bar's rim start" describes the
  turned block. It stays as written, and this section corrects it.
