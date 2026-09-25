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
that did it, and nothing here reaches it.*

  244.1   Which proved zeros reach the arm
  244.2   A block edge grazing a ball puts the start between two nodes
  244.3   Where the boxes touch, `place_the_meets` folds the pair first
  244.4   What is not shown

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
it is, by `2⁻¹⁰³`, and the refusal is the right answer.

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
