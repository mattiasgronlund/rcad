## 318. Two meets of one call can share one box on a closed crossing — 2026-09-28

*Queue row Q111, from §248.5. Lane `lane-q111-two-points-one-box`, off
`7de56c47`. `round_a_closed_crossing` answers `Zero` for `one == other`
without asking a policy, and a box with width can hold two points. **Found:** a
ball grazing a leaning block edge far from the origin puts the two meets of one
`meet_of_three_surfaces` call on the ball's two closed crossings with boxes
that are the same bits, and the arm folds them. **No wrong answer came of it:**
every such fuse refuses by name, on the turn at that box, and would refuse there
with the two meets kept apart. No `src/` changed.*

  318.1   What the suites bring to the arm
  318.2   The family that reaches it, and why this one
  318.3   The fuse, the two meets and what the kernel answers
  318.4   What this leaves

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
