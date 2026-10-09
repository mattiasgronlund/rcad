## 421. Two pipes crossing in two loops round the thinner are ordered — 2026-10-09

*Queue row Q1483. Lane `lane-q1483-two-pipes-crossing-in-a-quartic-loop`, off
`72d85401`, in four relays (§416): relay 0 wrote `904918e9` and `7b54d183`,
relay 1 `9132bc54`, relay 2 `f27b9e5b`, `96541407` and `e1c3c72a`, and relay 3
this section. **The row is not closed.** The fuse gets past the arrangement
now and is refused later, by the mesh it checks its answer with (421.4), in
files outside the row's.*

### 421.1 The shape

A cross-drilled shaft: a bar of radius 4 about `z` from `z = 0` to `z = 10`
(`160π`), and a bar of radius 1 along `x` from `x = −10` to `x = 10` at
`z = 5` (`20π`), through the fat bar's axis or `0.5` off it. The fat pipe
swallows the thin pipe's whole cross-section, so the two cross in two closed
loops, one each side of `x = 0`. Each loop is a quartic and lies in no plane.
`intersect` names each loop as part of the crossing, picked out by a half-space
whose plane holds the fat axis and is square to the thin one, here `x = 0`
(`two_loops_either_side_of_a_pipe` in `rcad-geom`'s `intersect.rs`). That plane
is ten from either end of the thin bar, so each cap is on one side of it.

On the base, every rule both ways round was refused
`ACrossingThisCannotBeOrderedAlong` before selection (Q1248's pin). This is the
row §39.4 lists as `TheseTwoSurfacesDoNotCrossInAConic` for "a ball against a
pipe, or two pipes", a gap, and §36.8 as `ACrossingThisCannotBeOrderedAlong`
for "the two surfaces cross in a quartic".

### 421.2 The order: round the thin pipe's axis

§39.4 said ordering points along such a curve "takes the marcher". For this
pair it does not. Look down the fat pipe's axis: it is a circle of radius `R`.
Each line of the thin pipe's wall, along its own axis, is a line across that
circle at a distance between `δ − r` and `δ + r` from its centre, for `δ` the
distance between the two axes and `r` the thin radius. Where `R > δ + r` every
such line crosses the circle twice, once going in and once coming out. So each
loop meets each line of the thin wall once, and going round a loop goes once
round the thin pipe's axis `w`. Three points of a loop, looked at down `w`, are
then three points of the thin pipe's circle in the same order, and the
module's determinant `((one − from) × (other − from)) · w` orders them: the
circle is convex even though the loop is not flat. This holds for axes at any
angle, not only square; the shapes pinned are all square.

Which way a loop runs: with `ρ_t` and `ρ_f` the steps out from each axis to a
point of the loop, the loop runs along `ρ_t × ρ_f`, and
`(ρ_t × ρ_f) · (w × ρ_t) = −r² (ρ_f · w)`. So the loop where the thin pipe
comes out of the fat one winds about `−w`, and the loop where it goes in winds
about `w`. Which loop a run is on is read off its start point.

In `rcad-geom`'s `along_a_crossing.rs` (`7b54d183`):

- `the_way_a_loop_round_a_thin_pipe_winds` is the two-pipe arm of
  `the_way_this_crossing_winds`.
- `whether_a_fat_pipe_swallows_a_thin_one` asks exactly whether
  `(R − r)² K − T² > 0`, with `K = ‖w × w_f‖²` and
  `T = (o_f − o_t) · (w × w_f)`. That is `K (R − r − δ)(R − r + δ)`, so its
  sign is the sign of `R − r − δ`.
- `whether_a_thin_pipe_comes_out_of_a_fat_one_at` gives the sign of
  `ρ_f · w`, over the point's box first and exactly where the box straddles.

In `fuse/arrange.rs`, `one_of_two_loops_round_a_thin_pipe` lets two pipes
named as part of their crossing through `how_a_crossing_runs` to the closed
curve's order.

**Still refused, each a gap** (the curve is there): two pipes of one radius,
two running the same way (they cross in lines, which other code answers),
and a thin pipe the fat one does not swallow whole, whose one loop runs back and
forth round both axes. A ball against a pipe off its centre is unchanged.

Pinned in `along_a_crossing.rs`'s tests (`96541407`):
`three_points_on_a_loop_where_two_pipes_cross_come_back_in_the_order_they_run_round_it`,
through the axis and half off it, both loops, eight starts, both orders, both
pairings. It checks each step against `∇S × ∇T` first, so it pins the geometry
and not just the code. `two_pipes_that_do_not_cross_in_two_loops_round_the_thinner_are_refused`
pins the three refused pairs.

### 421.3 The cells: past the plane between the two loops

With the order in place, every rule was refused
`CannotTellTheCellsOfAFaceApart` at `the_walk_of_each_other_lump_each_walk_faces`
on the fat bar's wall. Relay 1 traced it with a print it did not commit:
walk 4 of the fat wall (lumps `[0,0,1,1,2,2,3,3]`) had sides
`[Neg,Neg,Neg,None]` and agreed with both walks of the other loop, so
`which_of_them_holds` could not choose (`9132bc54`'s message). The cause:
`whether_past_the_picker` answered `PastThePicker::No` for a picker whose plane
is not square to the face's axis. Every node of the fat wall is on the thin
pipe, so a walk along one loop learned nothing about which side of the other
loop it was on.

`whether_past_the_plane_between_two_loops`, called first in
`whether_past_the_picker` for two pipes, answers it. Past the plane through the
fat axis:

- **on the fat wall**, each loop bounds a patch inside the thin pipe that lies
  wholly in front of the plane, so every point past the plane is outside that
  patch: the partner's positive side, even inside the other loop's patch;
- **on the thin wall**, the plane cuts one closed curve round the thin axis,
  inside the fat pipe, and everything past it is one tube the loop in front
  never meets: the partner's negative side, even beyond the other loop.

A box straddling the plane is answered where the two readings agree. Relay 1
measured the pin's first rule refused `CannotTellTheCellsOfAFaceApart` without
this at `7b54d183`. Relay 2 could not re-run that (the auto-mode classifier
refused checking out the old `arrange.rs`), and nor did relay 3. There is no
unit test of this function on its own; the pin in 421.4 is what reaches it.

### 421.4 Where the thin bar stops now

Every rule gets past the arrangement and assembly, and the mesh the fuse
checks its answer with refuses it. Each refusal names an edge or a face of the
answer, so assembly ran. Pinned by
`a_thin_bar_through_a_fat_one_is_refused_by_the_check_after_the_arrangement`
in `tests/a_cone_hollowed_along_its_axis.rs`, with the fat bar's seam up `+x`
and up `+y`:

- **Through the axis**, all six rules:
  `ACrossingAlongThisEdgeCouldNotBeFollowed`, from
  `CannotFollowACrossingWhereItsSurfacesAreTangent` at
  `SettlingAPointOntoBothSurfaces`. Relay 1 traced it with prints in
  `march.rs`'s `one_step`. From the seed `(4, 0, 6)`, the first trial,
  `1.7e307` long, cannot be settled. Every shorter trial from `4.1e153` down
  settles back onto the seed exactly, a chord of `0`. `one_step` only halves
  such a trial, so its fifty halvings never come near the loop, and it hands
  back the first trial's settling error.
- **Half off the axis**, in `every_rule_both_ways` order: the union with the
  fat bar first `CurvedFaceWithAHoleGoesTooFarRound`; the union with the thin
  bar first `CannotMeshAFace` `BoundaryCrossesItself`; the intersection both
  ways `CannotMeshAFace` `CornersCoincide`; the fat bar less the thin
  `CurvedFaceWithAHoleGoesTooFarRound`; the thin bar less the fat
  `BoundaryCrossesItself`. The two that were `AWireWillNotLieFlatOnItsFace`
  moved when Q1494 taught `mesh.rs`'s `cut_a_ring_across` to cut a ring with
  islands in it. Every one is now traced to `walk.rs` (Q1262, Q1508), as the
  pin's doc comment says.

So the refusal has moved from the arrangement, inside the row's files, to
`march.rs` and the mesher, outside them. It is still `refuses/B`, now in two
rows (421.7).

The row asked for answers checked against volumes. The answer pin turned into
this refusal pin; it could not stay as an `#[ignore]`d test, because the
justfile pins `ignored_tests` at 5. Once past, each rule is to answer, with `S`
what the two share (421.5): the union `180π − S`, the intersection `S`, the fat
bar less the thin `160π − S`, and the thin less the fat its two ends,
`(20π − S)/2` each.

### 421.5 What the two share, and how it was worked out

At `y`, the fat bar is `2√(16 − y²)` across along `x` and the thin bar
`2√(1 − (y − off)²)` along `z`. So with `s = y − off` the two share
`S = ∫ 4 √(16 − (s + off)²) √(1 − s²) ds` over `−1 ≤ s ≤ 1`. Through the axis
that is `8 ∫₀¹ √((1 − y²)(16 − y²)) dy`.

Put `s = sin t`, and `S = 2 ∫ cos² t √(16 − (sin t + off)²) dt` over a whole
turn, since the half turn from `π/2` to `3π/2` covers the same `s` again. That
integrand is smooth and periodic, so the trapezium rule over a whole turn
closes in on it geometrically. `what_a_thin_bar_through_a_fat_one_shares(off,
points)` is that rule, and
`what_a_thin_bar_through_a_fat_one_shares_closes_in_by_thirty_two_points` pins
that 32 points and 256 agree to `4e-15` of `S`:

| `off` | 32 points | 256 points |
|---|---|---|
| 0 | `24.9348268993723` | `24.934826899372304` |
| 0.5 | `24.732813206549718` | `24.732813206549725` |

(`mise exec -- just t -E 'test(/thin_bar_through_a_fat_one/)' --no-capture`.)

**Checked a second way.** A midpoint rule straight in `s`, with no change of
variable, has the square root's end to deal with, so its error falls as
`n^−1.5`: by 8 for each four times the points. At a million and four million
points it gives `24.934826906918538` and `24.934826900315578` through the axis,
and `24.7328132140285` and `24.732813207484565` half off. Those fall by 8 as
they should, and taking that out gives `24.934826899372297` and
`24.732813206549714`, within `4e-15` of the trapezium rule. The command, run
by relay 3 with the system `python3`:

```text
python3 -c "
import math
def shared(off, n):
    h = 2.0/n
    return sum(4*math.sqrt(16-(s+off)**2)*math.sqrt(1-s*s) for s in (-1+h*(i+0.5) for i in range(n)))*h
for off in (0.0, 0.5):
    a, b = shared(off, 1000000), shared(off, 4000000)
    print(off, 'midpoint', repr(a), repr(b), 'extrapolated', repr((8*b - a)/7))
"
```

### 421.6 Q1394

Open row Q1394 builds a thin bar through a fat one, fuses it, then fuses the
answer again with a far cube, to see whether `name_of_the_closed_crossing_through`
in `fuse/assemble.rs` names the two loops apart. Q1248's lane noted that leg met
this row's refusal before it reached naming. It now gets through assembly,
naming included, on the first fuse, but that fuse is still refused by its check
(421.4). So there is no answer to fuse with the cube yet, and Q1394's thin-bar
leg cannot be built until rows @1 and @2 of 421.7 land. Q1394 is unchanged.

### 421.7 What this leaves

- **Row: the march through the axis**, `refuses/B`, in `march.rs`'s
  `one_step`: shrink a trial that settles back onto its start (chord `0`) the
  way a trial that fails to settle is shrunk, pin it in `march.rs`'s tests,
  and turn the through-the-axis half of the pin into answers checked against
  421.5.
- **Row: the mesh half off the axis**, `refuses/B`: trace whether the answer's
  wires on the two pipe walls are wrong or `mesh.rs` cannot cut a pipe face
  holding a quartic loop, fix the side at fault, and turn the half-off half of
  the pin into answers. Q1494 fixed the mesher's side; the rest waits on Q1262
  and Q1508 in `walk.rs`.
- Not rows: `whether_past_the_plane_between_two_loops` has no unit test of its
  own (421.3), and leaning pipes are argued in 421.2 but not built.
- §39.4's and §36.8's rows for two pipes, and §34.10's, say so with a pointer
  here.
