## 244. Whether a closed crossing folds two nodes that are apart — 2026-09-25

*Queue row Q57, from §232.3. `round_a_closed_crossing` answers `Zero` when
the sweep from the walked start is proved zero and the two nodes' boxes
overlap, and `merge_points_that_are_one_point` folds on that `Zero`. Lane
`lane-q57-closed-crossing-folds`, off `60668c15`, read-only on `crates/`.
**Half an answer.** The start really does miss the circle and really does land
between two different nodes, in a fuse anyone can call. Where those two nodes'
boxes are apart the fuse refuses, as the code says it should. Where the boxes
touch, a different line folds the pair first — the check in `place_the_meets`
that throws away a meet already on the list. So this arm was never the one
that did it, and nothing here reaches it.* **Since §248 it does, and since
Q663 the arm asks the policy (244.5). Since Q449 a start between the two
places puts the one ahead of it first (244.6). Since Q699 (`a1d7c237`) two
places whose boxes each end at a start off the crossing, as at `cx = 16`, are
ordered by which end of its box the start is at.**

  244.1   Which proved zeros reach the arm
  244.2   A block edge grazing a ball puts the start between two nodes
  244.3   Where the boxes touch, `place_the_meets` folds the pair first
  244.4   What is not shown
  244.5   The premise lapsed with §248, and the arm now asks the policy
  244.6   A start between the two places puts the one ahead of it first
  244.7   Two boxes that end at the start are put in order by which end it is at

### 244.1 Which proved zeros reach the arm

`which_comes_first_round_a_crossing` has no level 1, so a `Zero` comes from
one of two places in `decide`:

- **The exact level**, which runs only when `from`, `one` and `other` are all
  `PredicatePoint::Exact`. Two exact points overlap only when every coordinate
  is equal, and then `one == other` has already answered at the top of
  `round_a_closed_crossing`. So an exact-level zero never reaches the
  overlap arm with two different nodes.
- **The interval level**, when `n · ((one − from) × (other − from))` comes out
  as exactly `[0, 0]`. `inari` rounds outward, so that needs every product in
  the sum to be a point interval. In practice that means coordinates of no
  width that cancel, with `one`, `other` and `from` sharing a coordinate
  exactly. The half ball in `round_a_closed_crossing`'s own doc comment
  (`x = 0, z = 0.5`) is one.

`from` is always the walked triple as an exact point — `how_a_crossing_runs`
builds `PredicatePoint::from(from.coordinates())` whatever the surfaces
certify. So a wrong fold needs two nodes on the crossing, a coordinate they
share exactly with `from`, and boxes that touch. On a circle in a plane
`z = c` the start sits at angle zero, which `Plane::basis` puts on the `±y`
side for a `±z` normal. Two nodes share the start's `y` exactly only if both
lie on the line `y = from.y` in that plane. That line runs close to the circle
at the start, so the two nodes are the two places where it cuts the circle.
They are distinct, and close together, only when the line nearly touches the
circle.

### 244.2 A block edge grazing a ball puts the start between two nodes

The shape of 244.1 can be built exactly. The numbers are chosen so that
`R² − a² − b²` is one unit in the last place: `(2m)² + (2m²)² + 1 =
(2m² + 1)²`, with `m = 2²⁵`, scaled by `2⁻⁵¹`.

- A ball of radius `R = 1 + 2⁻⁵¹` at `(cx, 0, 0)`.
- A block `x ∈ [cx − 2, cx + 2]`, `y ∈ [−2, 1]`, `z ∈ [a − 2, a]` with
  `a = −2⁻²⁵`, made by `ProfileBuilder` and `add_extrusion` shifted down.

Its edge `y = 1, z = a` misses the ball's centre by `√(R² − 2⁻¹⁰²)`, so it
cuts the ball at `x = cx ± 2⁻⁵¹`, **`8.9e-16` apart**. Both block faces land on
that edge. The top face `z = a` (normal `+z`) crosses the ball in a circle
whose walked start is `(cx, 1, a)`. The side face `y = 1` (normal `+y`)
crosses it in a circle of radius `2⁻²⁵` whose start is `(cx, 1, a)` as well.
Each start is on the edge, `2⁻¹⁰³` inside the ball: exactly between the two
places the edge cuts it.

With `cx = 0` both cuts are exact points, `(±4.44e-16, 1, −2.98e-8)`. Every
rule refuses under `Policy::certified`, and the one record is `order along a
crossing`, level `Interval`, enclosure `0.0`, on those two points. That is
`the_one_the_start_is` answering `(false, false)`: two boxes of no width,
neither holding the start. Its doc comment says of that case that it "needs a
start that is off the conic, which a walked start can be by a rounding". Here
it is, by `2⁻¹⁰³`. **The refusal was not the right answer** (244.6): the
start lies between the two places along the circle, so which comes first has
an answer, and since Q449 the arm gives it.

### 244.3 Where the boxes touch, `place_the_meets` folds the pair first

With `cx = 16` the two cuts are no longer exact. Their `x` boxes are
`[15.999999999999998, 16.0]` and `[16.0, 16.000000000000004]`, and they share
the point `16.0`. They come out of one `meet_of_three_surfaces` call — the
ball and the two block planes — so the second place checks `place_the_meets`'
dedupe: *is there a node on both these crossings whose box overlaps mine?*
The first place is, and **the second place is dropped** — two points
`8.9e-16` apart become one node without a policy being asked. Every rule then
refuses under `Policy::certified` on a later `orient3d`, whose input is the
kept box moved into a local frame: `x ∈ [−1.78e-15, 0]`, `y = 1`, `z = a`.

So `round_a_closed_crossing` never sees the pair. With two nodes gone to one,
there is no second node to order. The dedupe asks nothing about the policy,
and it folds before the arm this row is about could.

Measured with a probe patch that logged both lines and was reverted before
this commit. One log line was written whenever the overlap arm returned
`Zero` with `one != other`. Another was written whenever `place_the_meets`
dropped a place against a node that is not a corner (`vertex: None`) and whose
box is not bit-for-bit the same:

- `RCAD_Q57_PROBE=… just t -p rcad-topo -p rcad`: 1001 passed, 12 skipped,
  **no line of either kind**. Across both crates' suites, the overlap arm
  never folded two different nodes, and the dedupe never dropped one meet
  against another.
- The block and ball above, under all three rules at `cx = 16`: three dedupe
  lines, one per rule, for the surfaces `[ball, z = a, y = 1]`. No overlap-arm
  line.

The probe is not needed to rebuild the cases. The ball and block are
`rcad-topo` public API, and the test below reproduces what each fuse
returns:

```rust
fn fuse_a_block_edge_grazing_a_ball(centre_x: f64, rule: Rule) -> rcad_topo::Result<Fused> {
    let mut model = Model::new(Scale::millimetres(25.0).unwrap());
    let ball = model
        .add_ball(1.0 + f64::powi(2.0, -51),
                  &Isometry::translation(Vector::new(centre_x, 0.0, 0.0)).unwrap())
        .unwrap();
    let square = ProfileBuilder::new(vec![
        [centre_x - 2.0, -2.0], [centre_x + 2.0, -2.0],
        [centre_x + 2.0, 1.0], [centre_x - 2.0, 1.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .unwrap();
    let top = -f64::powi(2.0, -25);
    let block = model
        .add_extrusion(&square, 2.0,
                       &Isometry::translation(Vector::new(0.0, 0.0, top - 2.0)).unwrap())
        .unwrap();
    model.fuse(&[ball], &[block], rule, &Policy::certified(), &mut ProvenanceLog::new())
}
```

### 244.4 What is not shown

*The first two bullets are answered in 244.5. The third went with the dedupe
in §248.*

- **A fuse that reaches the overlap arm with two different nodes.** Its two
  nodes must be on the closed crossing and on *different* second crossings,
  or the dedupe takes them first (244.3). In the family of 244.1 that means
  two different surfaces whose meets both land on the line `y = from.y`. A
  corner will not do. A corner exactly on that line is also on the plane that
  makes the line, so `place_the_corners` lists that plane and the dedupe fires
  again. What is left is a curved face cut by the plane `y = b` and by a tilted
  plane holding the same edge line exactly. The sphere is cut by three circles
  through the same two points. Whether each circle's own meet call keeps a
  different one of the two, and so hands the overlap arm the pair, is not
  built. The tilted plane has to hold the line bit for bit, and a unit normal
  of two non-zero parts rarely allows that.
- **Families off the axes.** 244.1 names the one way to a point interval that
  was found by reading. A tilted circle plane or a ball–ball circle (normal
  `other.centre − one.centre`, rounded) would need coordinates cancelling
  exactly in a sum of three products. That is not proved impossible.
- **Whether the dedupe's fold ever gives a wrong answer.** In 244.3 a later
  predicate refuses. The dedupe is outside this row's arm, but it is the same
  unproved fold one step earlier — **reported as a defect in the certified
  sense** (§17.4: overlapping boxes are for the policy to merge or refuse) —
  and whether any fuse carries it through to an answer is not measured.

### 244.5 The premise lapsed with §248, and the arm now asks the policy

*Queue row Q663, from Q73 and Q74, which were read-only. Lane
`lane-q663-a-closed-crossing-zero-only-when-exact`, off `7fc73410`.*

**The premise of 244.4 lapsed with §248.** Its first bullet asked for meets
of *different* calls only because `place_the_meets` dropped the second meet of
one call. §248 removed that drop, so both meets of one call are kept, and
244.3's own shape at `cx = 16` hands the overlap arm its two places: the start
`(16, 1, −2⁻²⁵)` and both places lie on the edge line, the sweep is exactly
`[0, 0]`, and the boxes touch at `16`. Q73 found it with a temporary print in
the arm.

**The second bullet is answered: a tilted plane reaches the arm too.** Q74's
prism on the triangle `(0, 0)`, `(4, 3)`, `(−3, 3)`, 32 tall, has two walls
through the `z` axis whose stored normals are `(0.6, −0.8, 0)` and
`(−0.7071067811865476, −0.7071067811865476, 0)`, offsets exactly zero. A ball
of radius `1 + 2⁻⁵¹` at `(1, 2⁻²⁵, 16)` grazes that axis. Both walls' circles
start at exactly `(0, 0, 16)`; the meet of the two walls and the ball is two
boxes with `x` and `y` exactly zero and `z` overlapping round `16`; and
`which_comes_first_round_a_crossing` answers `Zero` on both walls.
`round_either_walls_circle_the_two_places_sweep_exactly_zero`, in
`crates/rcad-topo/tests/a_tilted_wall_grazing_a_ball_sums_to_zero.rs`, checks
each of those. Q74 also reported a circle where two balls meet reaching the
arm, two balls `2⁻²⁴` apart along `x` cut by `z = 1`. Q663 did not re-run it;
Q701 did, and the note at the end of this part says what it answers.

**A crossing in a plane whose normal has three non-zero parts reaches the arm
only with two nodes that are one place.** *Rewritten by Q701, 2026-10-11.*
Until then this paragraph was Q74's argument with no search or test behind
it. It argued from one sum, where the sweep is now two (step 1). It named only
meets and corners as the places a box of no width is made exact (step 3). And
it put the two nodes on a line through the start, which nothing showed they
must be on (step 4). Each step below says whether a test or a reading of the
code is behind it.

The **family** is every pair `the_way_this_crossing_winds` accepts whose
crossing lies in a plane:

- a plane against a ball, a pipe, a cone or a torus (both circle families),
- two balls,
- and the circles square to an axis that a ball and a pipe, a cone and a pipe,
  a cone and a ball, or two cones share.

In each, the plane the crossing lies in has a normal that is zero in the same
parts as the normal `n` the sweep is read against. For a plane pair `n` is that
plane's own normal, both torus rows included. For two balls it is
`c₂ − c₁` rounded, and a difference of two `f64`s rounds to zero only where it
is zero. For the circles on an axis it is the axis direction, which
`HalfSpace::through_a_point_facing` stores as handed over. For two cones of
one slope it is the step between the apexes, which lies along the axis. That
is a reading of `the_way_this_crossing_winds` and the functions it hands each
pair to, in `along_a_crossing.rs` on `959d1d64`, and not a test. Left out are
the two loops round a thin pipe, a pipe swallowing a thinner one and a ball
swallowing a pipe off its axis. They do not lie in a plane, and whether they
reach the arm with two places is not shown.

Write `f` for the start, `P` and `Q` for the two boxes, `p` and `q` for points
in them, `u = p − f`, `w = q − f`, and `V = n · (u × w)`.

1. **A zero means `V` is zero over the whole of both boxes.** The interval
   level sweeps two ways, `n · (u × w)` and `n · (u × (q − p))`, which are one
   number, and answers where the two enclosures overlap (§406.1,
   `along_a_crossing.rs`'s `on_intervals`). Each holds `V` for every `p` in
   `P` and `q` in `Q`, so an overlap of exactly `[0, 0]` means `V` is zero for
   all of them. It does not mean either sum is a point: `[0, h]` against
   `[l, 0]` overlaps in `[0, 0]`. The exact level runs only where all three are
   `Exact`, and two `Exact` nodes overlap only where they are equal, which
   `one == other` answers first (244.1). This step is a reading of
   `HowThreeOnACurveWind` and of `round_a_closed_crossing_and_what_it_measures`,
   not a test.
2. **Where one box has width along an axis, the other has width along that
   axis at most.** `V = (n × u) · w` is linear in `w`. So where `Q` has width
   along axis `i`, `(n × u)ᵢ = n_j u_k − n_k u_j` is zero over `P`, and with
   `n_j` and `n_k` not zero `P` has no width along `j` or `k`. The same holds
   with the two boxes swapped. So a box along `x` against one along `y` never
   sweeps zero. The step is algebra, and a test checks its last sentence on
   one pair of boxes (below).
3. **Case A: one box wide in two axes or three.** By step 2 the other box,
   say `P`, is a point, and two parts of `n × u` are zero. The third is too,
   since `(n × u) · n = 0` and that part of `n` is not zero. So `u = μn`.
   - **`μ ≠ 0`**: `V` is truly zero, and the interval still straddles,
     because interval arithmetic takes each place a variable occurs on its
     own. Each wide coordinate of `w`, and of `q − p` in the second sum,
     occurs twice, once with `+k` and once with `−k`, where
     `k = μ n_a n_b ≠ 0` for the other two axes `a` and `b`. Taking those two
     at the two ends of the box and everything else at one value gives `±k`
     times the box's width. So each sum straddles zero strictly, their overlap
     does too, and the answer is not `Zero`. That each wide coordinate occurs
     twice is a reading of `on_intervals`, which forms `u × w` and then dots
     it with `n`; a test checks the straddle on a few `μ` (below).
   - **`μ = 0`**: `P` is the start, a box of no width. Every node of no width
     is an `Exact` point. That is a reading of every place
     `fuse/arrange.rs` makes a node, on `959d1d64`, and not a test. Meets,
     touches, and the places `place_where_two_halves_meet` and
     `place_the_middle_of_each_half` make go through `as_exact_as_it_is`.
     Corners come from `bake.rs`'s
     `where_the_model_puts_a_corner`, which is `Exact` for a solid that has not
     moved and `as_exact_as_it_is` for one that has, or from `derive_a_corner`,
     which uses `as_exact_as_it_is`. A cone's apex and the one pole a circle
     runs through are `Exact`. The start's own node and the point across a
     circle from a pole are `Exact` or a box of positive reach. Every other
     node copies one of these. `PredicatePoint` compares by value, so
     `one == from` answers before the sum is asked.
4. **Case B: both boxes wide along one axis `i` at most.** Overlapping, they
   agree exactly on the other two coordinates, so both lie on one line `L`
   along `i`. Their true places lie on `L` and on the crossing's plane, whose
   normal is not zero along `i`. So `L` crosses that plane once, and the two
   are one place. The step does not need `L` to pass through `f`. It is
   algebra, and a test checks it on one line (below).
5. **Case C: two points.** This is case B with no width at all. Two points
   overlap only where they are the same triple, both are `Exact` by step 3's
   reading, and `one == other` answers first. No test of its own.

Three tests in
`crates/rcad-topo/tests/a_tilted_wall_grazing_a_ball_sums_to_zero.rs` check
steps 2 to 4 on the plane through the origin with normal `(1, 2, 2)`. It cuts
a ball of radius 9 at `(6, −6, 3)` in a circle through the origin, and every
sweep starts there:

- `a_point_along_a_tilted_normal_against_a_wide_box_does_not_sweep_zero`
  checks step 3's `μ ≠ 0` case for `μ` of `±1`, `2`, `0.5` and `2⁻⁶⁰`, boxes
  wide in two axes and in three, both orders. At `μ = 0` the sweep is `Zero`,
  which is why `one == from` has to answer first. That the fuse's `P` is then
  the `Exact` start rests on step 3's reading, not on this test.
- `two_boxes_on_an_axis_through_the_start_sweep_zero_and_that_axis_crosses_the_plane_once`
  checks step 4 on two boxes along `x` that meet at the start: they sweep
  zero, and the `x` axis crosses the plane once, at the start. The same two
  boxes moved off the start straddle.
- `boxes_along_two_different_axes_from_the_start_do_not_sweep_zero` checks
  step 2's last sentence on a box along `x` and one along `y`, both ending at
  the start.

They check each step on one plane and a few boxes, and prove none of them.
No search over tilted planes was run.

**Why the answer no longer carries weight.** Reaching the arm mattered because
the arm answered `Zero`, and `merge_points_that_are_one_point` folded on it
without a policy. Q663 makes the arm, and the arm for two equal boxes (§318.4),
answer `Zero` only when both boxes have no width, which is a proof, and
`Undecidable` as wide as the wider box otherwise. So any family that reaches
it now meets the policy, whether or not the argument above has a hole.

**What the fuses answer now**, measured with a probe in this row's test file,
each rule, each argument order:

| shape | `Policy::certified` | `Policy::tolerant(25.0)` |
|---|---|---|
| 244.3's edge, `cx = 16` | until Q699: refuses `order along a crossing` on the two places, `2⁻⁴⁸`. Since Q699, ball first: refuses `orient3d`, the turn at the place below `16`, `2⁻⁴⁹` (pinned); block first: `order along a crossing` along the block edge, `5.33e-15` (Q699's probe, not pinned) | until Q699: intersection answers `2.0933739 ± 0.0062779`; union and difference `CannotMeshAFace { CornersCoincide }`. Since Q699: every rule, either solid first, `ResultIsNotManifold { count: 3 }` (Q699's probe, not pinned) |
| the prism above, ball first | refuses: `order along a crossing` on the two places, `2.3e-14`, `Length` | intersection answers `0.43528013 ± 0.0025147`, the cap past the first wall; union and difference `CannotMeshAFace { CornersCoincide }` |
| the prism above, prism first | refuses: `order along a crossing` on the two places along the walls' shared edge, `4.4e-14`, `Length` | as above |

Before Q663 a certified fuse of either shape with the ball first refused on
`orient3d`, the turn at the folded node (§317, Q74); run against the base
`arrange.rs`, the two pins named
`a_certified_fuse_refuses_on_the_order_of_the_two_places`, in
`a_tilted_wall_grazing_a_ball_sums_to_zero.rs` and, at the time,
`a_block_edge_grazing_a_ball_at_cx_16.rs`, failed on exactly that record in
Q663's run. The prism's tolerant intersection is within its error
of the exact cap, which `a_tolerant_intersection_is_the_cap_past_the_first_wall`
checks; the edge's was not compared with an exact volume here. The
tolerant union and difference of the prism are queue row Q659's, which found
the fuse right and the mesher at fault, and are not pinned here.
`a_block_edge_grazing_a_ball_at_cx_16.rs`'s pin that §317.4 names as the test
to flip was `a_certified_fuse_refuses_on_the_order_of_the_two_places` from
Q663 until Q699, while the order refused before the turn was asked. Q699
decides that order, so the pin is
`a_certified_fuse_refuses_on_the_turn_at_the_place_below_16` again, and §317's
gap shows through that fuse under a certified policy.

**2026-10-11, Q701: the two-ball circle, run again.** Two balls of radius
`1 + 2⁻⁵¹` at `(0, y, 0)` and `(2⁻²⁴, y, 0)` cross in a circle in the plane
`x = 2⁻²⁵`, and `z = 1` cuts that circle at `(2⁻²⁵, y ± 2⁻⁵¹, 1)`. The fuses
are the whole first ball against the second with its cap above `z = 1` cut
off, each rule, either solid first. The normal the two balls' circle is swept
against is `(2⁻²⁴, 0, 0)`, with two zero parts, so this is 244.1's family
along another axis and not a tilted plane.

The pins named below are in
`crates/rcad-topo/tests/a_tilted_wall_grazing_a_ball_sums_to_zero.rs`.

- **At `y = 16`** the two places are not `f64`s. They come back as boxes with
  `x` and `z` exact that touch at `y = 16`. Round the two balls' circle from
  its walked start `(2⁻²⁵, 16, 1)` they sweep exactly zero, which is the
  overlap arm's input. Round either ball's circle in `z = 1` from its walked
  start the sweep straddles, by `1.5881867761018131e-22`.
  `two_balls_at_y_16_cut_z_1_in_two_boxes_that_sweep_zero_round_their_circle`
  checks the boxes and both sweeps.
  - Under `Policy::certified` every fuse refuses before it gets there, on the
    order of the two places round a ball's circle in `z = 1`. The one record
    is `order along a crossing`, `Interval`, `Volume`,
    `1.5881867761018131e-22`, which
    `two_balls_at_y_16_refuse_under_a_certified_policy_round_a_circle_in_z_1`
    pins. With the arm's order turned into an infinite refusal by a scratch
    edit, that pin still passes, so no certified fuse takes the arm's order
    before it refuses; a temporary print in the arm, never committed, printed
    nothing for these fuses either. The places are two and the start is
    neither, so that order exists: a gap, queue row Q1698.
  - Under `Policy::tolerant(25.0)` every fuse reaches the overlap arm, where
    Q699's `the_one_ahead_of_the_start` orders the pair, and then refuses on
    §346's limit. The one record past the budget is `a point in a piece that
    every solid could place`, `Length`, enclosure infinite, which
    `two_balls_at_y_16_under_a_tolerant_policy_refuse_on_a_piece_no_solid_can_place`
    pins. The same scratch edit makes that pin fail on its first fuse, the
    union with the whole ball first, whose record past the budget is then the
    arm's `order along a crossing`. That shows one fuse goes through the arm
    and takes its order there. That all six do rests on the print, which
    counted 48 arm hits, all in these fuses and all ordered, and not on a
    test.
- **At `y = 0`** the two places are the exact `(2⁻²⁵, ±2⁻⁵¹, 1)`, apart, so
  their boxes do not overlap and they never reach the overlap arm, though they
  sweep zero round the two balls' circle, as
  `two_balls_at_y_0_cut_z_1_in_two_exact_places_that_are_apart` checks. Under
  both policies every fuse refuses on `a point inside a piece of a face`,
  `Length`, enclosure infinite, on the two places, which
  `two_balls_at_y_0_refuse_on_a_point_inside_a_piece_of_a_face` pins. That is
  §346's limit, as at 244.6's `cx = 0`.

### 244.6 A start between the two places puts the one ahead of it first

*Queue row Q449, from §317.6. Lane `lane-q449-a-start-between-two-nodes-by-angle`,
off `bfc39286`.*

**What 244.2 found, run again on the base.** At `cx = 0` both circles start at
the exact `(0, 1, −2⁻²⁵)`, the two places are the exact `(±2⁻⁵¹, 1, −2⁻²⁵)`,
the sweep is a proved zero, and neither place's box holds the start. Under
`Policy::certified` every rule, with either solid first, refused on `order
along a crossing` with enclosure `0.0`. That record is the `(false, false)`
arm of `the_one_the_start_is`, not a sweep that stopped at `Interval`. Under
`Policy::tolerant(25.0)` every rule refused with
`CannotTellTheCellsOfAFaceApart`. A temporary print in the arm showed this,
and was taken out before the commit.

**The answer exists.** The start is inside the ball and on the chord between
the two places, so it lies between them along the circle. Going round from
there, the place ahead of the start comes first. The crossing of `S` and `T`
runs along `∇S × ∇T`, the fact `which_comes_first_round_a_crossing`'s table is
built on. So the arm now asks, for each place, the sign of the step from the
start dotted with `∇S × ∇T` at the start. It reads only a `Pos` for one place
against a `Neg` for the other. Every other pair of signs stays `Undecidable`,
as before, and so does the arm where both boxes hold the start.

**Why not the centre, as the row asked.** The row asked for
`det(w, from − c, P − c)`, with the centre `c` taken from
`where_a_closed_crossing_is_centred`. That function's answer keeps its fields
private to `holds.rs`, which was outside this row's write set. The run at the
start needs no centre. For a ball's circle the two are one polynomial: `∇S × ∇T`
at the start is `w × (from − c)`, for a ball and a plane either way round and
for two balls, and `(P − from) · (w × (from − c))` is `det(w, from − c, P − c)`.
On a pipe's or a cone's ellipse they are not the same polynomial. This lane did
not build an ellipse that reaches the arm.

**What says the sign is the right one.** In `fuse/arrange.rs`,
`a_start_inside_the_ball_between_two_places_puts_the_one_ahead_of_it_first`
runs both circles, with each surface first. The sweep from the same start to a
point across the circle is decided. It puts that point after the place the arm
now calls first, and before the other. Any other answer would leave the
circle's order with no consistent place to start.

**What the fuse answers now.** Under both policies, every rule, either solid
first, the one record is `a point inside a piece of a face` on the two places.
The sliver between them holds no point with `f64` coordinates. That is §346's
limit, the one the edge along `z` meets at `h = 0` in
`a_block_edge_grazing_a_ball.rs`. `a_block_edge_grazing_a_ball_at_cx_0.rs` pins
both facts.

**The broken bar's pin does not move.** The round bar whose top corner a file
moves to `(3, 0, 11)`, cut along its seam, reaches the same arm. There the two
points are the moved corner and a new one at `(3, 0, 10)`, one straight above
the other. The start `(0, 3, 10)` is not between them, so both steps point the
same way and the arm still refuses. `OFF_THE_CAP` is still `Undecidable` on
`order along a crossing`.

### 244.7 Two boxes that end at the start are put in order by which end it is at

*Queue row Q699, from §244.5's open table row. Lane
`lane-q699-two-places-on-a-grazed-ball-ordered`, merged at `a6450f53`; its own
commit is `a1d7c237`.*

**The overlap arm now orders a pair whose boxes end at an off-crossing start,
one at each end, instead of leaving it to the policy.** §244.5's `cx = 16`
shape cuts the ball at `16 ± 2⁻⁵¹`, not an `f64`, so the two places are the
boxes `[15.999999999999998, 16]` and `[16, 16.000000000000004]` along `x`,
with `y` and `z` exactly the walked start's. Both boxes hold the start, one at
each end, so they touch and the sweep from the start reads exactly zero — the
case `boxes_overlap` hands to `one_point_if_both_are_exact` since §248.

Read as a box, the step from the start to either place could be nothing, so
its sign was untold there and the pair fell to the policy. But where a box
agrees with the start exactly on two axes, every point of it but one is the
start moved along the third, and that one point is the start itself. A place
is on both surfaces of the crossing by construction, so where the start is
proved off either surface — `side_of_surface` answering `Pos` or `Neg` — the
place cannot be the start. A box `[s, h]` along that third axis, with the
start at `s`, then holds a place past the start; `[l, s]` holds one short of
it. `ahead_by_the_end_of_its_box_the_start_is_at` reads which end the start is
at and combines it with the sign of the crossing's run along that axis to
answer which way the place lies, and `the_one_ahead_of_the_start` asks it only
where the step read off the box itself left the sign untold. The overlap arm
takes an order only when this reads `Pos` for one place and `Neg` for the
other — **which also proves the two are two places**, since two places either
side of one point cannot be that one point twice.

**What each fuse answers now**, from §244.5's table: Q699's own probe except
where marked pinned. `cx = 16` under `Policy::certified`: ball first, every
rule refuses `orient3d`, the turn at the place below `16` (§317's gap, pinned
as `a_certified_fuse_refuses_on_the_turn_at_the_place_below_16`); block first,
every rule refuses `order along a crossing` along the block edge, `5.33e-15`.
Under `Policy::tolerant(25.0)`, either solid first, every rule now answers
`ResultIsNotManifold { count: 3 }`, in all six combinations of rule and
argument order.

**The tilted wall (Q74) and §318.3's far block are not reached, and why.** On
the tilted wall both `z` boxes hold `16` inside them rather than at an end,
and each holds both of the crossing's two true places, so no box says which
place the start is nearest to; `ahead_by_the_end_of_its_box_the_start_is_at`
answers `Undecidable` there the same as before. On the far block the
crossing's two places are one box used twice, with the start outside it on
more than one axis, so no single axis is left moved by exactly one and the
function answers `Undecidable` again. Queue row Q908 takes up ordering those
two by another method.
