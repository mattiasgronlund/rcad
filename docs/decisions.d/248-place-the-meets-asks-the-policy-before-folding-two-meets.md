## 248. `place_the_meets` asks the policy before folding two meets — 2026-09-25

*Queue row Q72, from §244.3 and §244.4. Lane `lane-q72-meets-ask-the-policy`,
off `c0b29166`. `place_the_meets` threw a new meet away whenever a node on both
its crossings had a box touching its own, and the other meet of the same call
is such a node. **Fixed.** A meet is now dropped only against a corner that no
other meet of the call reaches. Every other meet stays a node, and
`merge_points_that_are_one_point` asks the policy whether two touching nodes
are one point, as §17.4 says it should. No suite fuse reaches the changed
drop. No fuse was found that carried the old drop through to a wrong answer.*

  248.1   What the drop did, and the one drop that is a proof
  248.2   What the suites drop, measured
  248.3   The test, and why its edge runs along `z`
  248.4   Whether the old drop gave a wrong answer
  248.5   What this leaves

### 248.1 What the drop did, and the one drop that is a proof

The brief gave three legs. Each was checked.

**Leg 1: the overlap test answers "same node" for boxes that touch.** By
reading: `boxes_overlap` compares `lo <= hi` on every axis, so boxes that share
only an end point overlap. By running: §244.3's two meets at `x = 16 ∓ 2⁻⁵¹` have
`x` boxes `[15.999999999999998, 16.0]` and `[16.0, 16.000000000000004]`. They
share `16.0`, and the second was dropped (244.3's probe, three lines, one per
rule).

The only nodes on both of a pair's crossings are corners and the earlier meets
of the same `meet_of_three_surfaces` call, because a meet node lists exactly its
own two crossings. The meets of one call are different points: two come from a
discriminant proved `Pos`, and a proved touch gives one. So a drop against a
meet of the same call always folds two different points.

**Leg 2: the fold that asks the policy is `merge_points_that_are_one_point`.**
It walks every crossing and proves the order of each neighbouring pair through
`order_along`. That goes through `must_decide`, which hands an undecided pair
to `resolve_under_policy` under the name `order along a crossing`. So the fix
adds no new question to ask the policy. It stops dropping the meet, and the
pair takes the route every other overlapping pair already takes.

**Leg 3: the drops the suites make are against corners.** A corner on both
crossings is on all three surfaces, so it is one of their meets, and its true
point lies inside the box of one of the call's meets. If only one meet's box
reaches the corner's box, that meet is the corner. That is a proof about boxes,
and it is what `a_corner_is_this_place` checks now. It counts every meet of the
call, including meets a picker rules out, since those are still meets of the
three surfaces. If a second meet's box also reaches the corner, nothing says
which of the two is the corner, and both stay nodes for the policy.

### 248.2 What the suites drop, measured

A probe patch, reverted before the fix was written, logged one line for every
node that the old drop threw a meet away against. Each line was sorted by two
things: whether that node is a corner, and whether its box is bit for bit the
meet's own. For a corner it also logged how many of the call's meets reach the
corner's box. `RCAD_Q72_PROBE=… just t -p rcad-topo -p rcad` on `c0b29166`
plus the probe and this row's test file:

```
     Summary [ 137.189s] 1002 tests run: 1002 passed (2 slow), 12 skipped
 990835 corner-identical places=1 overlapping-this-corner=1
   1592 corner-identical places=2 overlapping-this-corner=1
  75916 corner-overlap places=1 overlapping-this-corner=1
    344 corner-overlap places=2 overlapping-this-corner=1
     12 place-overlap exact=false probe
```

- **Every corner drop had exactly one meet reaching the corner**, including the
  1 936 where the call gave two meets. The new rule drops all of them, so no
  suite result can move.
- **No meet was dropped against a bit-identical meet** (`place-identical`
  never appears), so §244.3's other kind of drop never happens in the suites.
- **The only drops against a meet were this row's own repro**, twelve lines from
  the probe test holding §244.3's shape: two for each of its six fuses at
  `cx = 16`.

### 248.3 The test, and why its edge runs along `z`

§244.3's shape does not show the fix. With the fix in, all six of its fuses at
`cx = 16` refuse exactly as they did before, on the same record:

```
cx=16 certified Union: Undecidable
    orient3d Interval 1.7763568394002505e-15 [Constructed([[-1.7763568394002505e-15, 0.0], [1.0, 1.0], [-2.9802322387695313e-8, -2.9802322387695313e-8]]), Exact([0.0, 0.0, 1.0]), Exact([0.0, 1.0, 0.0])]
```

The box is the kept meet in the ball's frame, with 16 taken off its `x`. Both
exact inputs, `(0, 0, 1)` and `(0, 1, 0)`, lie in that frame's plane `x = 0`.
`add_ball` picks out its seam, the half of `y = 0` with `x ≥ 0`, with a
half-space on that same plane. An edge along `x` grazes the ball at `x = 0`, so
the kept meet's box, `x ∈ [−1.78e-15, 0]`, touches that plane, and a predicate
refuses on it whether the fold is made or not. An edge along `y` grazes it on the seam plane `y = 0`.
An edge along `z` at `x = −1` grazes it at the equator, a unit from both.

So `crates/rcad-topo/tests/a_block_edge_grazing_a_ball.rs` uses §244.2's
numbers turned onto `z`. It has a ball of radius `1 + 2⁻⁵¹` at `(0, 0, h)` and a
block whose edge is `x = −1, y = 2⁻²⁵`, which meet at `z = h ± 2⁻⁵¹`. At
`h = 16` their boxes are `[15.999999999999998, 16]` and
`[16, 16.000000000000004]`. It holds three tests:

- `two_meets_whose_boxes_touch_are_what_a_certified_fuse_refuses_on`: under
  `Policy::certified`, every rule refuses with exactly one record, `order along
  a crossing` at `Interval`, whose two inputs are the two meets.
- `a_tolerant_policy_asked_about_the_same_two_meets_folds_them`: under
  `Policy::tolerant(25.0)`, the same pair is recorded and resolved to `Zero`.
- `where_the_two_meets_are_exact_nothing_asks_whether_they_are_one`: the control.
  At `h = 0` both meets are exact and apart, so no `order along a crossing`
  record appears under either policy.

On the base `arrange.rs` (the fix commit's parent's file checked out, the test
kept), two of the three fail:

```
     Summary [   0.060s] 3 tests run: 1 passed, 2 failed, 0 skipped
        FAIL [   0.018s] (1/3) rcad-topo::a_block_edge_grazing_a_ball two_meets_whose_boxes_touch_are_what_a_certified_fuse_refuses_on
        FAIL [   0.053s] (2/3) rcad-topo::a_block_edge_grazing_a_ball a_tolerant_policy_asked_about_the_same_two_meets_folds_them
```

The certified one fails on the first rule. The base refused on an `orient3d`
whose only box is the kept meet, moved into the ball's frame:

```
    assertion `left == right` failed: Union: Record {
        predicate: "orient3d",
        inputs: [
            Constructed(
                [
                    [-1.0, -1.0],
                    [2.9802322387695313e-8, 2.9802322387695313e-8],
                    [-1.7763568394002505e-15, 0.0],
                ],
            ),
```

With the fix, the refusal moves to the pair itself:

```
cz=16 certified Union: Undecidable
    order along a crossing Interval 5.329070518200751e-15 [Constructed([[-1.0, -1.0], [2.9802322387695313e-8, 2.9802322387695313e-8], [16.0, 16.000000000000004]]), Constructed([[-1.0, -1.0], [2.9802322387695313e-8, 2.9802322387695313e-8], [15.999999999999998, 16.0]])]
```

Under the tolerant policy the pair is folded, and the fuse still refuses later,
on `a point inside a piece of a face` with an infinite enclosure. That refusal
is not about the fold. At `h = 0`, where there is no pair to fold, both
policies refuse on the same predicate:

```
cz=0 certified Union: Undecidable
    a point inside a piece of a face Interval inf [Exact([-1.0, 2.9802322387695313e-8, -4.440892098500626e-16]), Exact([-1.0, 2.9802322387695313e-8, 4.440892098500626e-16])]
```

The plane `x = −1` cuts the ball in a circle of radius about `2⁻²⁵`, and the
piece inside that circle is where it gives up.

### 248.4 Whether the old drop gave a wrong answer

**None was found.** On the base, every fuse tried refused:

- §244.3's edge along `x`, at `cx = 0` and `cx = 16`, three rules, under
  certified and `tolerant(25.0)`. At `cx = 16` all six refused on the
  `orient3d` quoted in 248.3. At `cx = 0` the tolerant three refused with
  `CannotTellTheCellsOfAFaceApart`, and the certified three on `order along a
  crossing` between the two exact meets (244.2).
- This row's edge along `z` at `h = 16`, three rules, both policies. Certified
  refused on the `orient3d` quoted in 248.3. The tolerant result on the base
  was not read beyond its test failing.
- The suites: 248.2 shows no suite fuse ever made the drop this row changes.

Under a tolerant policy the old drop did what the policy does anyway: 248.3's
tolerant test resolves the same pair to `Zero`. So a wrong answer from the old
drop would need a certified fuse in which the kept meet goes on to be decided.
That means its box would have to clear every later predicate while the
dropped meet, `8.9e-16` away, did not exist. None of the shapes above got that
far, and no other family was tried.

### 248.5 What this leaves

- `crates/rcad-topo/src/fuse/arrange.rs`: `round_a_closed_crossing` answers
  `Zero` for `one == other` without asking a policy, and two bit-identical boxes
  can hold two points. 248.2 saw no two meets with one box, so nothing in the
  suites puts such a pair on one closed crossing. That was not measured past the
  drop.
- `crates/rcad-topo/src/fuse/arrange.rs`: `a_corner_is_this_place` relies on the
  corner being on all three surfaces. Where that came from a tolerant
  `passes_through`, it rests on the policy's earlier acceptance, not on a proof.
  Not measured.
- `crates/rcad-topo/src/fuse/select.rs`: `a point inside a piece of a face`
  refuses with an infinite enclosure on the piece a plane cuts from a ball in a
  circle of radius `2⁻²⁵` (248.3). Whether that is a limit or a gap was not
  looked at.
- `crates/rcad-topo/tests/a_block_edge_grazing_a_ball.rs`: §244.3's edge along
  `x` refuses on an `orient3d` against the plane of the ball's seam picker, with
  the fix or without it. The call site was not found, and whether that refusal
  is a limit or a gap was not looked at.
