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
places puts the one ahead of it first (244.6).**

  244.1   Which proved zeros reach the arm
  244.2   A block edge grazing a ball puts the start between two nodes
  244.3   Where the boxes touch, `place_the_meets` folds the pair first
  244.4   What is not shown
  244.5   The premise lapsed with §248, and the arm now asks the policy
  244.6   A start between the two places puts the one ahead of it first

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
`crates/rcad-topo/tests/a_tilted_wall_grazing_a_ball_sums_to_zero.rs` checks
each of those. Q74 also reported a circle where two balls meet reaching the
arm, two balls `2⁻²⁴` apart along `x` cut by `z = 1`; that was not re-run here.

**A circle plane whose normal has three non-zero parts cannot reach the arm.**
This is Q74's argument, not a search. `inari` rounds outward, so the sweep
comes out `[0, 0]` only if every coordinate of a box with width is multiplied
by an exact zero. With no zero in the normal, either both nodes lie on one line
along an axis through the start, and such a line crosses the plane once, so
the two are one place; or one box is exactly the start, a box of no width.
Every box of no width is made an exact point (`as_exact_as_it_is` for meets and
corners), and `one == from` answers that before the sum is asked.

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
| 244.3's edge, `cx = 16` | refuses: `order along a crossing` on the two places, `2⁻⁴⁸` | intersection answers `2.0933739 ± 0.0062779`; union and difference `CannotMeshAFace { CornersCoincide }` |
| the prism above, ball first | refuses: `order along a crossing` on the two places, `2.3e-14`, `Volume` | intersection answers `0.43528013 ± 0.0025147`, the cap past the first wall; union and difference `CannotMeshAFace { CornersCoincide }` |
| the prism above, prism first | refuses: `order along a crossing` on the two places along the walls' shared edge, `4.4e-14`, `Length` | as above |

Before Q663 a certified fuse of either shape with the ball first refused on
`orient3d`, the turn at the folded node (§317, Q74); run against the base
`arrange.rs`, the pins below fail on exactly that record. The prism's
tolerant intersection is within its error of the exact cap, which its test
checks; the edge's was not compared with an exact volume here. The
tolerant union and difference of the prism are queue row Q659's, which found
the fuse right and the mesher at fault, and are not pinned here.
`a_block_edge_grazing_a_ball_at_cx_16.rs`'s pin that §317.4 names as the test
to flip is now `a_certified_fuse_refuses_on_the_order_of_the_two_places`: the
order refuses before the turn is asked, so §317's gap no longer shows through
that fuse under a certified policy, only in the file's measurement of the turn.

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
