## 252. A vertex that only starts a closed edge is not a corner of the answer — 2026-09-26

*Q76, which is §245.3's hole A. Lane `lane-q76-hole-a`, off `50a3165e`.
`Assembly::from_the_pieces` in `crates/rcad-topo/src/fuse/assemble.rs` now
drops an input vertex where the kept pieces do nothing but walk along one
closed input edge, and reads the two sides through it as one before any edge
is named. The three fuses of two bars side by side that met hole A now answer
and measure right, so all six fuses of that pair answer. The one that answered
already lost its corners at `(5, 0, 0)` and `(5, 0, 10)`. No pin moved.*

  252.1   The fix
  252.2   Why the input edge and not the crossing's name
  252.3   The test, and how it failed on the base
  252.4   A real corner stays one
  252.5   What the six fuses do now
  252.6   Where else the rule fires
  252.7   What this leaves

### 252.1 The fix

`vertices_that_only_start_a_closed_edge` looks at every side of every kept
piece before any corner is named. An input vertex qualifies when every kept
side that names it as an end passes three tests:

- it runs along exactly one input edge, the same one for every side, and that
  edge starts and ends at this vertex;
- it does not close on itself;
- it is one of two sides of one wire walked straight through the vertex. One
  side arrives and the next leaves. They have the same two surfaces, the same
  material side and the same input edge, and the run they make does not come
  back to where it started.

`read_through` then gives back each wire with the two sides either side of such
a vertex joined into one `FaceMeet`, from the first side's start to the second
side's end. A wire that passes through no such vertex is borrowed as it is,
which is every wire of almost every fuse. `Corners::with_each_closed_curve_started_once`
and `sides_of` both read the joined wires, so the vertex is never named and no
edge ends at it.

The rule reads only the input's topology and the pieces' own sides. No
coordinate is compared and no sign is asked, which is §245.5's requirement:
the rule depends only on which sides are kept.

### 252.2 Why the input edge and not the crossing's name

§245.5 wrote the rule as "every kept side that ends there runs along one
crossing (the same two surfaces, named either way round)". That is a weaker
test, because one crossing can turn. A plane through a pointed cone's apex cuts
the cone in two rulings that meet at the apex. Every side there then runs along
the same plane and the same cone, and the apex is a real corner. The rulings
are cuts, or two input edges, and never one closed edge, so the test used here
keeps the apex. That case was argued and not built. The verifier found it
cannot arise at all today: a pointed cone's apex is not a vertex of the model
(`crates/rcad-topo/tests/cones/mod.rs` builds none there), and the rule only
ever drops an `Existing` vertex (252.8).

A closed input edge runs right round one closed crossing. Where it starts is
the builder's choice and is no bend in the curve, which is what makes it safe
to join the two sides there.

### 252.3 The test, and how it failed on the base

`where_a_bar_started_its_cap_circle_is_not_a_corner_of_the_answer`, in
`crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs`, runs four fuses. It
runs the three that met hole A, which are the union with `here` as the
argument and the intersection and difference with `there` as the argument. It
also runs the intersection with `here` as the argument, which answered already
with `(5, 0, ·)` as corners. For each one it validates the model, asks for one
solid, checks that the corners are exactly `(3, ±4, 0)` and `(3, ±4, 10)`, and
checks the volume against the one worked out by hand.

With `git checkout 50a3165e -- crates/rcad-topo/src/fuse/assemble.rs` and
`mise exec -- just t -p rcad-topo --test merged -E 'test(~fusing_two_bars_side_by_side::where_a_bar_started)'`,
it failed at the first fuse:

```
    thread 'where_a_bar_started_its_cap_circle_is_not_a_corner_of_the_answer' (1273080) panicked at crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs:299:39:
    Union of two side-by-side bars: ResultIsNotManifold { count: 1 }
```

**The volumes by hand.** A bar is `π·5²·10 = 785.398…`. The two bars share a
lens made of two discs of radius 5 whose centres are 6 apart. Its half-angle
is `acos(3/5)`, and its chord is 8 long and 3 from each centre. The lens is
two circular segments, each a sector less a triangle, so its area is
`2·5²·acos(3/5) − 3·8 = 22.3648…` and the shared volume is `223.648…`. The
union is two bars less that, `1347.149…`, the intersection is `223.648…`, and
the difference is one bar less that, `561.751…`. Keeping or dropping the lens
wrongly moves any of them by 223.6, far more than the mesh's bound, so the
check can fail.

### 252.4 A real corner stays one

A vertex that only starts a closed edge fails the first test as soon as a
third surface meets the curve there. The cut along that surface's crossing
ends at the vertex, and a cut runs along no input edge.
`where_a_bar_started_its_cap_circle_stays_a_corner_where_the_other_circle_crosses_it`,
in the same file, builds that case. A second bar is centred at `(5, 5, 0)`,
so its circle runs through `(5, 0, 0)`, where the first bar started its own.
Subtracting one circle from the other leaves `x + y = 5`, so the circles cross
at `(5, 0, ·)` and `(0, 5, ·)`. The centres are `5·√2` apart, so the lens's
half-angle is `acos(√2/2) = π/4` and its area is `2·5²·π/4 − (5√2/2)·5√2 =
12.5·π − 25`. All six fuses, three rules each way round, answer and validate.
Each has exactly the corners `(0, 5, 0)`, `(0, 5, 10)`, `(5, 0, 0)` and
`(5, 0, 10)`, and each measures within the mesh's bound of the volume by hand.

The whole of `rcad-topo` is the wider check. `fuses_that_must_agree.rs` and
`turning_a_fuse.rs` compare answers bit for bit, and both passed unchanged
(252.6).

### 252.5 What the six fuses do now

A temporary test ran all six fuses of the side-by-side pair, each on a fresh
copy of the model, and printed each answer. It was
`mise exec -- just t -p rcad-topo --test merged -E 'test(~fusing_two_bars_side_by_side::temp_probe_all_six)' --no-capture`,
and the test is deleted again:

```
TEMP Union here on there: valid true faces 4 volume 1346.5972099829235 ± 2.207736708953189 corners [[3.0, -4.0, 0.0], [3.0, 4.0, 0.0], [3.0, 4.0, 10.0], [3.0, -4.0, 10.0]]
TEMP Union there on here: valid true faces 4 volume 1346.5972099829253 ± 2.2077367089531883 corners [[3.0, 4.0, 0.0], [3.0, -4.0, 0.0], [3.0, -4.0, 10.0], [3.0, 4.0, 10.0]]
TEMP Intersection here on there: valid true faces 4 volume 223.41772414475454 ± 0.9202470039325203 corners [[3.0, 4.0, 0.0], [3.0, -4.0, 0.0], [3.0, -4.0, 10.0], [3.0, 4.0, 10.0]]
TEMP Intersection there on here: valid true faces 4 volume 223.41772414475437 ± 0.9202470039325203 corners [[3.0, -4.0, 0.0], [3.0, 4.0, 0.0], [3.0, 4.0, 10.0], [3.0, -4.0, 10.0]]
TEMP Difference here on there: valid true faces 4 volume 561.5897429190844 ± 1.5661424928259258 corners [[3.0, -4.0, 0.0], [3.0, 4.0, 0.0], [3.0, 4.0, 10.0], [3.0, -4.0, 10.0]]
TEMP Difference there on here: valid true faces 4 volume 561.5897429190838 ± 1.5661424928469605 corners [[3.0, 4.0, 0.0], [3.0, -4.0, 0.0], [3.0, -4.0, 10.0], [3.0, 4.0, 10.0]]
```

All six answer, validate, and are one solid of four faces with four corners.
Each is within its mesh's bound of the volume by hand: `0.55 ≤ 2.21` for both
unions, `0.23 ≤ 0.92` for both intersections and `0.16 ≤ 1.57` for both
differences. The two unions, the two intersections and the two differences
agree to within a few ulps.

Neither bar's rim start is a corner of any of them. `(11, 0, ·)`, where
`there` started its circle, is absent from both unions now. It lies on
`there`'s outer arc, which is one edge of about 254 degrees. On the base the
union with `there` as the argument already answered with those four corners
and no `(11, 0, ·)`, and the union with `here` as the argument refused, so the
fix moved one union and not two (252.8).

`two_overlapping_parallel_bars_stop_in_select_whichever_way_round`, still
ignored for Q77, now stops at its first fuse because the union answers:

```
Union on two side-by-side bars answered Ok(Fused { model: ModelId(1), solids: [SolidKey { slot: SolidSlot(3v1), model: ModelId(1) }], ... vertices: {VertexSlot(1v1): [], VertexSlot(2v1): [], VertexSlot(3v1): [], VertexSlot(4v1): []} })
```

The line is cut short where it says `...`. In the history, all four input
vertices now map to nothing, and each bar's cap circle maps to one edge.

### 252.6 Where else the rule fires

A temporary print in `from_the_pieces` appended a line to a scratch file
whenever the set of vertices was not empty. The whole crate then ran with
`mise exec -- just t -p rcad-topo --no-fail-fast`:

```
     Summary [ 125.404s] 1009 tests run: 1009 passed (3 slow), 12 skipped
```

The file had 815 lines from 42 tests. Three of the lines were torn by two test
processes writing at once. The rule fired in 334 fuses of
`every_answer_over_a_block_turned_against_a_bar_is_a_solid_that_validates`,
and in 71 of `how_often_a_certified_fuse_of_a_bar_leaning_through_a_block_refuses`,
60 of `swapping_the_two_sides_of_a_bar_leaning_through_a_block_changes_not_one_bit`
and 54 of `how_often_a_certified_fuse_with_a_cone_in_it_refuses`. It also fired
in the frustum and pointed-cone files, such as
`a_block_and_frustum_united_can_be_united_again` and
`a_near_semicircle_keeps_its_centre_behind_its_picker`. Every one of those
tests passed unchanged, including the two refusal-rate pins and the
bit-for-bit swap. So the rule changes the shape of many answers, by one corner
fewer on a rim, and moves no count that anything pins.

### 252.7 What this leaves

- Rewrite `two_overlapping_parallel_bars_stop_in_select_whichever_way_round`
  as six fuses that answer and take off its `#[ignore]` (Q77). Its first fuse
  answers now, as 252.5 shows. `crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs`.
- Fuse the side-by-side answers once more. Both unions now carry an arc of
  about 254 degrees round `(11, 0, ·)`, the shape §224.1 found the next fuse
  may not follow. **252.8 found this is not what stops it**: every answer
  refuses the second fuse, the lens included, and did on the base too.
  `crates/rcad-topo/tests/fusing_two_bars_side_by_side.rs`.
- A side of another solid that ends at the same point as the vertex under
  another name is not counted (252.1). That would need a surface through the
  vertex that cuts no face of the vertex's own solid there, which is a
  tangency, and no test builds one. `crates/rcad-topo/src/fuse/assemble.rs`.
- Build the cone case of 252.2, a plane through a pointed cone's apex, to show
  that the apex stays a corner. `crates/rcad-topo/src/fuse/assemble.rs`.
  252.8 built it, and all six fuses refuse, so it shows nothing about the rule
  yet.

### 252.8 What the verifier found

An independent verifier read the fix and the record, ran mutations, and
merged the lane onto `732a849a` and with Q5's `fcf02a14`. Its verdict was to
merge, with the correction to 252.5 made above.

- **Confirmed.** `read_through` is exact: the two sides it joins keep their
  direction, and the other fields are equal because the third test
  (the pairing) already asked. Only an `Existing` vertex can be dropped. Two
  closed edges meeting at one vertex keep it. The plane `y = 0` through a
  bar's rim start keeps `(5, 0, ·)`, and all six of those fuses are within
  bound. Both new tests fail on the base, and the second one fails with extra
  corners `(10, 5, ·)` as well, which 252.4 did not mention. Every volume was
  worked again by hand, and each test asserts `|measured − by_hand| ≤`
  the mesh's bound.
- **Mutations.** Dropping the pairing test, or its "does not come back"
  check, fails five tests in `a_rim_a_fuse_cut_round_one_cone.rs` and
  `fuse_laws_over_a_cone.rs`. **Dropping test 1, or test 2, passes every
  suite**, because no fuse in the suites reaches the shape test 1 guards, and
  test 3's "does not come back" check compares the two ends as whole
  `MeetPoint`s, points included, while test 2 compares only the vertex, so
  test 3 covers test 2 only while the arrangement gives both ends of one
  closed side the same point and no kept wire runs along one closed edge
  twice. Neither test is asserted anywhere.
  `a_side_that_closes_on_itself_keeps_its_one_corner` (Q125, `a91dc003`) now
  stands in for both. Test 1 is pinned by
  `a_vertex_passed_along_anything_but_the_closed_edge_that_starts_there_stays_a_corner`,
  and the drop case by
  `where_a_bar_started_its_cap_circle_is_dropped_when_the_kept_arc_only_passes_it`.
- **Fusing an answer again** refuses `AFaceTrimmedByACurveThisCannotFollow`
  (§35) for every one of the six answers, the lens included, and did on the
  base. So the 254° arc is not the cause.
- **A pointed cone and a block whose face `x = 0` passes through the apex**:
  all six fuses refuse `CannotTellTheCellsOfAFaceApart { face: 8v1 }`.
- **A plane leaning through a bar's rim start** (`z = y` through
  `(5, 0, 0)`): all six refuse `Undecidable` on "order along a crossing".
- **The tangency of 252.7's third item** could not be built: all nine shapes
  tried refuse "which way a crossing runs" first.
- The two tests collect corners from each edge's start only. Taking both ends
  would be a stronger check, and it is not needed for this merge.

### 252.9 One pin moved when this met §251

Neither this lane nor Q5's ran with the other's change, and together they
move one pin. On the merged tree `a_plane_cut_along_a_cones_other_nappe.rs`'s
`a_pointed_cone_with_its_apex_near_a_steeper_face_answers_right_or_refuses`
went from 58 right and 14 `CannotMeshAFace` to **60 right and 12**. Its other
refusals did not move. With `assemble.rs` put back to `732a849a`, on the same
merged tree, the test passes at 58 and 14, so this section's rule is what
moves it. Before §251 those fuses never got past selection, so the rule could
not reach them. The file's `fuse_and_check` panics on a wrong answer, and a
case counts as right only after it validates, has the right number of solids,
measures within the bound of the volume by hand, and holds exactly the right
points on a grid. So both new answers are right. The pin was moved at merge.
