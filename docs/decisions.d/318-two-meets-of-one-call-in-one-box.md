## 318. Two meets of one call can share one box on a closed crossing — 2026-09-28

*Queue row Q111, from §248.5. Lane `lane-q111-two-points-one-box`, off
`7de56c47`. `round_a_closed_crossing` answers `Zero` for `one == other`
without asking a policy, and a box with width can hold two points. **Found:** a
ball grazing a leaning block edge far from the origin puts the two meets of one
`meet_of_three_surfaces` call on the ball's two closed crossings with boxes
that are the same bits, and the arm folds them. **No wrong answer came of it:**
every such fuse refuses by name, on the turn at that box, and would refuse there
with the two meets kept apart. No `src/` changed.* **Q663 closed the arm
(318.5).**

  318.1   What the suites bring to the arm
  318.2   The family that reaches it, and why this one
  318.3   The fuse, the two meets and what the kernel answers
  318.4   What this leaves
  318.5   The arm asks the policy

### 318.1 What the suites bring to the arm

A probe, kept out of the commit, logged every call `order_along` and
`which_comes_first` made to `round_a_closed_crossing`, and for each pair of
equal points what kind of node each was. `RCAD_Q111_PROBE=… just t -p rcad-topo
-p rcad` on `7de56c47` plus the probe:

```
     Summary [  81.264s] 1113 tests run: 1113 passed (2 slow), 12 skipped
 381543 comparisons of two different points
    488 comparisons of two equal points, every one exact (width 0)
```

The 488 are meets and corners that land on an `f64` triple, from six tests,
252 of them from `a_ball_smaller_than_the_room_on_a_cube_corner_refuses_on_its_own_face`.
An exact point is one point, so each of those `Zero`s is proved. **No
constructed box reached the arm equal to another in the suites**, which is
§248.2's finding measured at the arm rather than at the drop.

### 318.2 The family that reaches it, and why this one

Two equal boxes on one closed crossing are two nodes whose points were worked
out to the same bits. The nodes a closed crossing holds are corners, meets, and
the start and the point across it. Two meets of **one call** are the one pair
that is certainly two different points, since two places come only from a
discriminant proved positive (§248.1), and the one pair whose boxes come out of
the same arithmetic, so they are where equal boxes were looked for first.

Their boxes are equal when both roots of the line's quadratic land in one
rounding cell of every coordinate. That takes a line that nearly touches the
quadric, and a chord whose middle is off the `f64` grid:

- **An edge along an axis cannot do it.** Its chord's middle is the ball's
  centre along that axis, an `f64`, so the two roots straddle it. That is
  §248.3's shape: boxes `[15.999999999999998, 16]` and `[16, 16.000000000000004]`,
  touching, never equal.
- **Near the origin a leaning edge cannot either.** One `f64` step of the radius
  past touching leaves a chord of about `1e-8` (measured at `16`: the two `x`
  boxes are `1.85e-8` apart), far wider than a cell there.
- **Far out it can.** A cell at `1e8` is `1.5e-8` wide and at `1e10` it is
  `1.9e-6`.

`meet_of_three_surfaces` on a top `z = F + 0.25`, a wall with normal `(nx, ny,
0)` and a ball whose radius is found by halving on its bits (the smallest `f64`
at which the three share two points), 25 centres for each of six wall normals
and six distances `F`:

| `F` | configurations | two places with equal boxes |
|---|---|---|
| `16` | 150 | 0 |
| `1e6` | 150 | 0 |
| `1e8` | 150 | 1 |
| `1e9` | 150 | 0 |
| `1e10` | 150 | 7 |
| `1e12` | 150 | 0 |

The probe printed five of the eight, and all five had the wall normal `(1, 1,
0)`. The same halving through a
ball and a block built by the model (the wall `(1, 1, 0)`, 40 centres each at
`1e8` and `1e10`) found 12 of 80: 3 at `1e8` and 9 at `1e10`.

### 318.3 The fuse, the two meets and what the kernel answers

`crates/rcad-topo/tests/two_meets_in_one_box_on_a_closed_crossing.rs` holds one
of them: a block one unit tall with its top at `z = 10¹⁰ + 0.25` and a wall
leaning at 45°, and a ball at `(10000000000.809, 9999999999.85,
10000000000.9679)` of radius `0.7739638708673935`. One `f64` less and the three
share nothing. At that radius both places are

```
Constructed([[10000000000.604492, 10000000000.604506],
             [9999999999.645494, 9999999999.645508],
             [10000000000.249996, 10000000000.250002]])
```

bit for bit.

On the ball's face both lie on the circle the top cuts and the circle the wall
cuts, both closed. For the 12 shapes, fused under three rules and two policies,
the probe logged 288 comparisons at the arm, four a fuse, every one `meet0`
against `meet1` of the same three surfaces, with box widths `1.04e-7` and
`1.19e-7` at `1e8` and `1.34e-5` at `1e10`. Each was answered `Zero`, and the
pair folded into one node without a policy being asked.

**What the kernel answers:** all 72 fuses (`Policy::certified` and
`Policy::tolerant(25.0)`, three rules each) refuse with `Error::Undecidable`,
one record: `orient3d` at `Interval` on the ball's gradient box at the node and
the normals of the top and the wall. That is `turns_anticlockwise`, the turn
between the two circles at the node.

**What is right:** a refusal. The turn is `(p − c)·(n_top × n_wall)`, which is
linear in `p` and zero on the plane through the centre square to the edge line.
That plane holds the chord's middle, so the two meets sit either side of it and
the turn has opposite signs at them. Any box that holds both holds a point
where it is zero, and an enclosure of it straddles zero. Kept as two nodes with
one box each, both turns would be asked on that same box and refused the same
way. So in this family the unasked fold changes which record a certified fuse
refuses on, and nothing else. The test pins that refusal, and says so.

### 318.4 What this leaves

- **The arm is a gap and wants a fix row.** Two equal boxes with width are
  folded without the policy. §17.4 merges only a decided coincidence, and
  §244.4 reads it as leaving overlapping boxes to the policy to merge or
  refuse; this fold is neither decided nor the policy's. 318.3
  shows the pair is reachable through the public API. The one family seen is
  masked by the turn. A fix answers `Zero` only for exact points and hands equal
  boxes with width to the policy, for example as `Undecidable` as wide as the
  box. Under `Policy::certified` the test's refusal would then move to `order
  along a crossing` on the two meets, so the fix lane owns this file too.
- **Not searched:** two meets of **different** calls in one box (another third
  surface cutting the same circle a cell away), where the turn argument above
  does not apply and a fold could go on to be decided; a start and the point
  across a circle smaller than its box; and a tolerant policy whose budget
  would refuse the fold while the turn resolves. Those are the routes to a
  wrong answer this row did not close.
- The probe was not run over `rcad-step`'s tests or the nightly fuzz corpus
  beyond what `every_model_the_fuse_fuzzer_kept_is_fused_again` replays.

*The first bullet is done in 318.5, and the routes in the second no longer
pass an unasked fold.*

### 318.5 The arm asks the policy

*Queue row Q663, with §244.5. Lane
`lane-q663-a-closed-crossing-zero-only-when-exact`, off `7fc73410`.*

`round_a_closed_crossing` now answers two equal boxes through
`one_point_if_both_are_exact`: `Zero` when the box has no width, which is one
point and a proof, and `Undecidable` as wide as the box otherwise. The overlap
arm of §244 answers the same way. `merge_points_that_are_one_point` reaches
them through `order_along`, so the policy merges the pair or the fuse refuses
(§17.4).

**What 318.3's fuse answers now.** Every rule under both policies, with the
ball first, refuses with one record: `order along a crossing` at `Interval`,
the one box twice, as wide as the box, `1.33514404296875e-5`. With the block
first a probe saw the same predicate, one record each. A tolerant policy at a 25 mm working size
refuses too, so its budget is under that box. The test is
`every_fuse_refuses_on_the_order_of_the_two_places_in_one_box`, and against the
base `arrange.rs` it fails on the `orient3d` record 318.3 pinned.

**The routes 318.4 left open.** Two meets of different calls in one box, and a
start and the point across a circle smaller than its box, reach the same arm
as any two equal boxes, so neither is folded without the policy now. A
tolerant policy whose budget refuses the fold now refuses, as above.

**One thing this does not settle.** The enclosure the two arms hand back is a
box's width, a length, and `order_along` files every refusal on a closed
crossing as `Measures::Volume`, as `the_one_the_start_is` already did. A
tolerant policy reads a volume's enclosure divided by its working size twice,
so it merges boxes up to `s²` times wider than its budget for a length: 625
times at 25 mm. Not changed here, and no fuse was searched for that it misleads.

**2026-10-07, Q706: filed as a length, and searched for a fuse it misleads.**
`one_point_if_both_are_exact`, `the_one_the_start_is` and
`the_one_ahead_of_the_start`, and the box arm along a bend, now hand back
`Measures::Length`: each encloses the wider of two boxes' longest sides, or
`det(apart, n, w)` with `n` and `w` of length one, not a volume. A search of
168 tolerant fuses — the shape above lifted by `0` to `1048560` under working
sizes `1`, `25`, `100` and `1000`, and §318's block and ball at `1e10` under
working sizes `25` to `10000`, three rules, either solid first — found 21
merges a length budget refuses, and every one of the 21 still refused on the
next thing it met: no answer moved, and none was wrong. The sweep arm
(`which_comes_first_round_a_crossing`) stays `Measures::Volume`; what it
should be filed as is its own row (Q706's follow-up).

**2026-10-14, Q1273: the sweep filed by what it measures.** The sweep arm now
files its refusal as `rcad-geom` says
(`Surface::which_comes_first_round_a_crossing_and_what_it_measures`): an area
round a plane's conic, a circle round a pipe's or a cone's axis, and two
pipes' one loop; a volume round two balls' circle, two cones of one slope, and
a ball's one loop off a pipe's axis; a length where the way round cannot be
told. Probes of 7488 and 13984 tolerant fuses found no wrong answer. The
corner-up cube of side 6 on the steep cone, answered right at working sizes
`2` to `2.6` under the volume reading, now refuses at `2`
(`a_tolerant_fuse_of_a_cube_on_a_cones_apex_refuses_a_sweep_past_an_area_budget`),
and `the_sweep_round_a_planes_conic_is_an_area_and_round_two_balls_circle_a_volume`
pins the measure by growing six shapes to twice their size. Below a working
size of `1` the area reading is the more lenient one, by `1/s`; no fuse has
been searched for one it misleads (Q2002).
