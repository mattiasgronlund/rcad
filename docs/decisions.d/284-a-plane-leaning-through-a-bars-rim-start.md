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
  whose messages carry the volumes. **Done by §401 (Q209, `164b8f73`)**, which
  renamed the check to
  `a_block_turned_to_lean_through_a_bars_rim_start_answers_all_six_fuses_with_one_solid_each`
  when a meet of two planes and a pipe came to narrow by one checked step and
  the order read that narrower box.
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
  outside this lane's write set. **Corrected by §401.7**: the second cause's
  row is struck rather than kept, since §401 closes it rather than merely
  demonstrating it.
- §252.8's sentence "a plane leaning through a bar's rim start" describes the
  turned block. It stays as written, and this section corrects it.

### 284.7 Why the turned face keeps the meet, under a tolerant policy — 2026-09-29

*Q363, what §330.7 left of Q210. Lane
`lane-q363-a-plane-leaning-through-a-bars-rim-start-tolerant`, off `34052a6d`.
No code changed: the fix needs `arrange.rs` and `mod.rs`, which were not this
lane's.*

**The turned face has no rim start to fold the meet into.** Each face is cut
by its own `Arrangement`, and an arrangement makes corner nodes only from its
own face's wire (`Arrangement::place_the_corners`). The rim start is a vertex
of the bar, so the cap and the wall have a node for it and the turned face,
which is the block's, does not. On the cap and on the wall the meet of the cap
plane, the wall and the turned face is folded into the rim start under the
tolerant policy (284.2). `Arrangement::fold_the_nodes` lets the input corner
win, and in doing so drops the meet's `made_of`, the three surfaces and which
of their meets it was; `Arrangement::meet_point` then hands the corner on as
`MeetPoint::Existing` with nothing else. On the turned face the same meet is a
node of its own and comes out as `MeetPoint::OfThreeSurfaces`. `Corners::of`
in `assemble.rs` names the one `Vertex` and the other `Meet`, so they are two
corners.

A print of every kept side's two ends, in `Assembly::from_the_pieces`, since
removed, showed it for the union with the bar first
(`PROBE_Q363=1 mise exec -- just t -p rcad-topo --test a_plane_leaning_through_a_bars_rim_start --no-capture under_a_tolerant_policy`).
The three surfaces are the turned face, the cap plane and the wall; their two
meets are `which 0`, far, at `(−5, 2.5e-15, 0)`, and `which 1`, near:

```
cap   (piece 1)      Vertex(1v1) at (5, 0, 0)  and  Meet(which 0)
wall  (piece 4)      Vertex(1v1) at (5, 0, 0)  and  Meet(which 0)
turned (pieces 10, 11)  Meet(which 1)          and  Meet(which 0)
```

**That is the whole gap.** A second probe, also removed, took the name
`Meet(which 1)` over to the rim start's corner in `Corners::taken_over` before
any piece was walked, and changed nothing else. It picked the pair by
coordinates, within `1e-9`, which is why it is a probe and not a fix
(`CornerName` says why a coordinate never names a corner). With it, all six
fuses answer, `Model::validate` passes, and each is one solid inside
`measure`'s own `volume_error` of 284.4's volume by hand, at a deflection of
`0.01`:

| fuse | measured | by hand |
|---|---|---|
| union, either way round | `64701.787 ± 2.641` | `64702.065` |
| intersection, either way round | `83.286 ± 0.496` | `83.333` |
| bar less block | `701.787 ± 2.641` | `702.065` |
| block less bar | `63916.714 ± 0.496` | `63916.667` |

Run as `PROBE_Q363_JOIN=1 mise exec -- just t -p rcad-topo --test
a_plane_leaning_through_a_bars_rim_start --no-capture probe_q363_measure`, a
test since removed. The take-over fired 48 times over the six fuses, every
time `Meet(which 1)` to vertex `1v1`.

**The fix is to carry the fold, not to guess it in `assemble`.** Families
considered:

- **Carry the names a node stands for.** `fold_the_nodes` keeps every
  absorbed `(surfaces, which)` on the node that survives, `meet_point` hands
  them on in `MeetPoint`, and `assemble` puts each one into
  `Corners::taken_over` before any piece is walked, so the order of the faces
  cannot matter. This records a decision the arrangement already made, under
  the policy it was asked under. It is the one to build.
- **Infer the join in `assemble` from the names alone.** The cap's wire puts
  the rim start between a side on the wall and a side on the turned face, so
  it is at one of the three surfaces' meets, and the cap names the other meet
  itself, which leaves `which 1`. That elimination is right for this shape.
  It is a guess wherever a face with the vertex does not reach every meet, and
  a wrong guess joins two points ten apart, which is a corrupt solid.
- **Infer it from coordinates.** The probe's rule. `CornerName` rules it out.
- **Ask the policy again in `assemble`.** The fold was decided on the order
  round the cap circle, whose enclosure straddles zero. A distance between
  the two points is proved non-zero, since the meet's `y` side is above zero,
  so any other question refuses the join the arrangement made.

Carrying the names has more to do than this shape needs. A meet that folds
into another meet loses its name the same way, and
`Arrangement::place_the_meets` drops a place outright when
`a_corner_is_this_place` says a corner already is it, so that corner must
record the place too. One meet taken over by two input vertices, each on a
face of a different solid, would join those two, so `taken_over` wants a
union of names rather than a map from one name to one corner. None of that
was built or measured.

`under_a_tolerant_policy_the_turned_block_is_ordered_and_its_faces_disagree_on_a_corner`
stays the check, and it trips once the names agree: under the second probe
its first fuse, the union with the bar first, answered where the pin wants
`ResultIsNotManifold { count: 1 }`.

### 284.8 A node carries every name it folds — 2026-10-04

*Queue row Q656, from 284.7. Lane `lane-q656-a-node-carries-every-name-it-folds`,
off `f4775743`, which holds Q209 (`164b8f73`) and Q711's corner naming
(§351.6).*

**The turned block no longer reaches the fold.** On `f4775743` the six
tolerant fuses of the turned block all answer, each one solid within
`volume_error` of 284.4's volume by hand, with nothing in the log. 284.7's
check had already become
`under_a_tolerant_policy_the_turned_block_answers_all_six_fuses_with_one_solid_each_too`
in Q209's commit. Nothing joined the two names. Since Q209 the order round the
cap circle is proved, so the policy is never asked, the meet is never folded
into the rim start, and it is a node of its own on the cap, on the wall and on
the turned face alike. A temporary print in `Corners`, since removed, of every
corner within `1e-6` of `(5, 0, 0)` showed one in each of the turned block's
twelve fuses, under both policies: `Meet(which 1)` at
`(5, 2.512e-15, 0)`. The rim start is no corner of the answer at all, since
it only starts the cap's closed edge. Q711 played no part: the rim start has a
plane and a cylinder round it, not three planes.

**Other shapes still reach it.** A second probe, also removed, recorded every
meet name an arrangement dropped, by a fold in `fold_the_nodes` or because
`a_corner_is_this_place` found a corner already was the place, and asked
`Corners`, once every kept piece was walked, whether the dropped name and the
name of the node that stood for it had become two different corners. Over
`mise exec -- just t -p rcad-topo --no-fail-fast` on `f4775743` that happened
in 36 fuses:

| test | fuses | refused `ResultIsNotManifold { count: 1 }` |
|---|---|---|
| `a_fused_draw_hands_its_provenance_back_through_the_door` | 20 | 18 |
| `every_seed_in_the_checked_in_corpus_is_fused_without_panicking` | 10 | 9 |
| `a_frustum_and_a_bar_of_the_same_size_refuse_the_same_corner_up_cubes_alike` | 6 | 6 |

The two corners were a few ulps apart in every one. The drops were of three
kinds: a meet of three planes folded into another meet of three planes, a meet
taken in by an input corner that §351.6 names by its own three planes, and a
meet of a cube's face, the top cap plane and the wall of a bar taken in by
the bar's top rim start, which is 284.7's shape under `Policy::certified`.

**What was built**, the family 284.7 chose:

- `Node::also` keeps every other meet a node stands for, as
  `(surfaces, which)` the way `made_of` names its own. `fold_the_nodes` puts
  the loser's name and everything it stood for on the survivor, for a meet
  into a corner and a meet into a meet alike, and
  `the_corner_that_is_this_place`, which `a_corner_is_this_place` now asks,
  hands back the corner node so `place_the_meets` and
  `place_where_two_rulings_cross` can record the place there.
- `split_a_face` hands back, beside the pieces, every node that stands for
  more than one name, as `NamesOfOnePoint`: the end the node is handed on as,
  and the meets it took in. `Scratch` gathers them from every face of both
  groups.
- `Corners::join_the_names_of_each_point` makes them a union of names before
  any piece is walked. Every name of one point goes by the least of them, so
  neither which face folded nor the order the solids came in can matter. A
  point with an input corner among its names goes where that corner alone
  would go, the lowest one if there are several, as the fold lets the corner
  win.

**Not a `MeetPoint` field**, though the row asked for one field on
`MeetPoint::Existing` and `OfThreeSurfaces`. `select.rs`'s own tests build
both variants and was outside this row's files. And a field on an end reaches
`assemble` only where a kept piece ends at that node, where the face that
folded may keep no piece there while the face that did not keeps one. Beside
the pieces, from every node, has neither limit.

**Q431's ask** was that an input corner solved from three planes also answer
to their `Meet` name. §351.6 did that in `assemble`, from
`BakedCorner::around`, and this row did not move it. Nothing is left of it.

**What moved.** The six fuses of the bar against the fifteenth corner-up cube
of `a_frustum_and_a_bar_of_the_same_size_refuse_the_same_corner_up_cubes_alike`
answer: the bar goes from 48 answered and 6 not a solid to 54 answered, and
its other 42 still refuse an undecided order, as the frustum's 48 do. That
pair is pinned by
`a_cube_on_its_corner_leaning_through_a_bars_top_rim_start_answers_all_six_fuses`,
a bar of radius 4 from `z = 3` to `z = 8` and a cube of side 8 stood on its
corner at `(0, 0, 4)`, under `Policy::certified`: each fuse one solid that
validates, with nothing logged, and union and intersection, and each
difference with the intersection, adding up to the solids within the measured
errors. With the join switched off all six refuse
`ResultIsNotManifold { count: 1 }`. Every test of `rcad-topo` passes with the
change, `fuses_that_must_agree.rs` included, so no pin moved under either
policy.

### 284.9 What this leaves

- Which of the 30 draws of `a_fused_draw_hands_its_provenance_back_through_the_door`
  and `every_seed_in_the_checked_in_corpus_is_fused_without_panicking` answer
  now, and with what volume. Neither test pins its draws' outcomes, and the
  probe was not run again after the change.
- In 44 fuses on `f4775743` a dropped name was a corner of the answer while
  the node that took it in ended no kept side; nearly all of them answered.
  Since this change, where that node was an input corner, the corner goes
  where the input corner goes and not where the meet arrived. Whether any of
  them moved by a bit was not compared, beyond every test passing.

### 284.10 How far an input corner moves a dropped name — 2026-10-15

*Queue row Q942, continuing 284.9. Two lanes ran on
`lane-q942-replay-the-30-draws-with-names-carried`: the first read the code
and reverted every probe it tried without measuring anything; this one
measured 284.9's second bullet and found why its first cannot be answered by
re-running 284.8's probe on this base.*

**284.9's second bullet: yes, by a few ulps, never more.** `Corners::of`
(`crates/rcad-topo/src/fuse/assemble.rs:7277`) hands a name the place
`join_the_names_of_each_point` (`:7203`) already settled on, whenever the two
were found to be one point; that handoff is the `Some(&(input_corner,
vertex))` arm at `:7290`. A temporary probe there (removed before this
commit, kept nowhere), run whenever `end` carried a name other than the
input corner's own vertex, printed the distance from `middle_of(input_corner)`
to `middle_of(end.point())`. Run as `PROBE_Q942_MOVED=1 mise exec -- just t -p
rcad-topo --no-fail-fast --no-capture` (a cold run, cancelled after 997 of
2188 tests — the suite's slow property tests, not this row's two, were what
was left ahead of it): 21 210 redirects fired, 19 178 of them (90%) by
exactly zero and the rest between `1.1e-16` and `5.0276450535801215e-14`
(the largest about 226 times a `f64`'s own epsilon), nine orders of magnitude
under the smallest tolerance named anywhere in this file (284.8's own
`1e-6` probe). Inside `a_fused_draw_hands_its_provenance_back_through_the_door`
alone — the row's own first test, lines 16135–25970 of that run's log — 9 824
of those redirects fired, 8 788 of them zero and the rest in the same tiny
range, topping out at the same `5.0276450535801215e-14`. That is 284.8's own
"a few ulps apart", read off the current code: an input corner does pull a
dropped name's corner a hair, never by anything a mesh or a `validate`
tolerance would call a move.

**284.9's first bullet is still open, and re-running 284.8's probe on this
base cannot close it.** `join_the_names_of_each_point` runs before any
corner is made, and every lookup after it goes through `goes_by`
(`:7284`) first, so two names it has joined can never again resolve to two
different corners — read off `Corners::of`'s control flow, not from trying it
and finding nothing. Asking 284.8's own question — "did the dropped name and
the name of the node that stood for it become two different corners" —
against this base therefore answers no for every fuse, whatever the 30
original draws individually do now: the question itself has no case left to
find, not because those 30 are confirmed fixed. Answering 284.9's first
bullet needs the identity of the original 30 (which corpus file, which half,
which test), and that can only come from 284.8's probe as it ran on
`f4775743`, the commit it was measured on.

Found this lane, so the next one does not have to re-find it:
`f4775743` is reachable (`git worktree add <path> f4775743 --detach`
succeeds) and still has the shapes 284.8's probe hooked. The drop happens in
`Arrangement::fold_the_nodes`
(`crates/rcad-topo/src/fuse/arrange.rs:1839`); its `Some(into)` arm (`:1857`)
is where one side of a fold's identity is always dropped — `mine`'s when
`takes_over` is false, the survivor's own `made_of` when it is true, so a
*blanket* "a name was dropped" probe fires on every fold and overcounts
284.8's 20/10/6, exactly as this row's first lane found. The dropped name
surfaces as a second corner, a few ulps from the one the node should have
shared, in `Corners::of` on that commit
(`crates/rcad-topo/src/fuse/assemble.rs:3694`), where it is named afresh
because nothing before Q656 told the two spellings to agree. §284.8 does not
say how its probe told seeds and halves apart; that still needs adding, in
whatever test harness hook is used (`crates/rcad-topo/tests/fuzzing/fuse.rs`).
Tag it, rebuild the probe at `f4775743` (not attempted this lane — only the
worktree and the two hook points were confirmed, not a build or a run),
replay the flagged seeds on the current base, and where one now answers,
check it the way
`a_cube_on_its_corner_leaning_through_a_bars_top_rim_start_answers_all_six_fuses`
(`crates/rcad-topo/tests/a_plane_leaning_through_a_bars_rim_start.rs:181`)
does.

### 284.11 A distance proxy for the dropped-name corner, and its limits — 2026-10-15

*Queue row Q942, continuing 284.9 and 284.10. Third lane on
`lane-q942-replay-the-30-draws-with-names-carried`, relayed from the one that
wrote 284.10. This closes no part of 284.9's first bullet; it narrows what the
next lane needs to build.*

**A cheaper probe than rebuilding `fold_the_nodes`'s own tracking, tried at
`f4775743` in a sibling `git worktree add ... f4775743 --detach`.** Rather than
following a dropped `made_of` through repeated folds, a temporary print in
`Assembly::new` (`crates/rcad-topo/src/fuse/assemble.rs`, right before the
`corners.made` that becomes `Self.corners`, so after every piece is walked and
no more corners are made) scanned every pair of the assembly's own corners and
printed any pair closer than `1e-10` and further than `0.0` — tight enough to
sit under every genuine "two features of a fuzz draw happen to be close"
distance and over `f64`'s own noise floor, and three orders of magnitude
tighter than the `1e-6` 284.8 names for its own targeted print. Two more
temporary prints, in `crates/rcad-topo/tests/fuzzing/fuse.rs`'s
`two_models_split_and_fused_noting_everything` (tagging `first`/`second`
immediately round each half's call) and in each of the two tests' own
`every_seed` (tagging the corpus filename before it is read), let the dupes be
read back against which draw and which half made them, since the test loop is
single-threaded and the three prints interleave in call order. None of the
three edits were committed; this subsection gives the exact lines so they can
be rebuilt in a few minutes, the way 284.10 did for its own probe.

**This proxy's count for
`a_fused_draw_hands_its_provenance_back_through_the_door` matches 284.8's
exactly: 20.** Over `PROBE_Q942_DUPES=1 mise exec -- cargo nextest run -p
rcad-topo --test a_fused_draw_hands_its_provenance_back_through_the_door
--no-capture` on `f4775743`, 368 pairs printed, but every one was either a
duplicate of another (the "both ways round" check runs the same pair of
groups twice) or sat at a distance from `1.1e-16` up to `8.8e-7` — most of
them far under 284.10's own `5.03e-14` ceiling, a few, at the top of that
range, probably genuine near-coincidences in a fuzz draw rather than the
bug — and resolved to exactly **20 distinct (seed, half) pairs**, the same
count 284.8's table gives this test. That match is the evidence the proxy is
finding the right thing on this one test, not a coincidence of the threshold.

**It does not reproduce 284.8's 10 for
`every_seed_in_the_checked_in_corpus_is_fused_without_panicking`.** Both tests
read the same corpus directory
(`fuzz/corpus/two_solids_fused_by_a_stranger`) and both end up inside the same
shared `two_models_split_and_fused`, so the same bytes meet the same fuse
code either way. Run the same way, this test's probe found the *same* 20
(seed, half) pairs as the first test, not a different 10. Whatever told
284.8's own probe the two tests' counts apart is not reproduced here, and
this lane did not find it — most likely 284.8 counted at the fold/drop level
(`fold_the_nodes`), where an event belongs to whichever call stack reached
it, and something about that attribution is test-specific in a way a
pure end-state distance scan on the finished `Corners` cannot see. The next
lane wanting the true 20/10/6 split needs the fold-level probe 284.10 already
sketched (`Arrangement::fold_the_nodes`,
`crates/rcad-topo/src/fuse/arrange.rs:1839`), not this one.

**Replaying the 20 at the current base mostly hits a different thing the row
did not ask about.** A temporary test
(`crates/rcad-topo/tests/q942_scratch_probe.rs`, removed before this commit,
not kept anywhere) decoded each of the 20 (seed, half) pairs exactly as
`how_one_model_is_fused_noting_which_way_the_second_call_went` does — same
header, same `rest.len() / 2` split, same `usize::from(header) %
(solids.len() + 1)` argument/tool split — on the **current** base (`6b5a987f`
already in it). Of the 20, **17 split one-sided**: every one of that draw's
solids landed on one side and none on the other, so there is no second group
for an inclusion-exclusion check to pair against. One further draw splits
`0`-`2`, also one-sided. Only **2 of the 20** split two-sided
(`26f1462c484dbaa264d92e26cf2402faa77b47da` half `first`, split 1 of 2; and
`af0513a2a8b8be4e4688ea514569a426c452145b` half `first`, also split 1 of 2 —
the same split on what look like two different corpus files, each 4763 bytes,
which the filenames alone do not explain and which this lane did not chase
down). Both of those two refuse **every** one of union, intersection and both
differences, each with the same `Undecidable` on `"where two planes cross"`
at a point with a subnormal coordinate
(`3.4766779039175e-310` and `1.54307e-319` respectively against `0.0`) — a
different refusal from `ResultIsNotManifold { count: 1 }`, and from
representation wobble near the limit of `f64`, not from a dropped name. So of
the 20 this proxy found, **none give this lane an inclusion-exclusion check
to run**: the 17 one-sided ones are not the shape the row's check applies to,
and the 2 two-sided ones refuse for an unrelated reason before any volume is
measured. This does not mean the fix did nothing for these 20 — a one-sided
fuse (one group against no tools) still walks the arrangement and could still
have hit the drop-and-diverge pattern 284.8 found at `f4775743`, and whether
it does **has not been checked**, since the inclusion-exclusion the row names
only makes sense for a two-sided pair.

**What this leaves, for the next lane:** 284.9's first bullet is still open.
The three probe edits above are the cheapest path back to a trustworthy
"which of the 30 draws" split by test, but a trustworthy "did the dropped
name become a corner" check still wants the fold-level tracking
(`fold_the_nodes`, `Corners::of` on `f4775743`) 284.10 already pointed at,
not this subsection's end-state distance scan — that scan is good evidence
for *where* to look, not a replacement for the real criterion. Separately,
for a one-sided draw (17 of the 20 found here), the question to ask on
replay is not inclusion-exclusion but whether the single answer's volume
equals the sum of its own input solids' volumes (union and difference
against no tools should both preserve it; intersection against no tools
should answer zero) — a different, simpler check than
`a_cube_on_its_corner_leaning_through_a_bars_top_rim_start_answers_all_six_fuses`'s,
not yet written.
