## 288. Two places of one meet within one corner's room: reachable, and hidden behind the piece between them — 2026-09-27

*Queue row Q110, from §248.6 and §253.7. Lane `lane-q110-two-meets-one-corner`,
off `ff78f99f`. `a_corner_is_this_place` folds a meet into a corner only when
no other place of the same meet reaches the corner's room, and nothing tested
that `count == 1`, the uniqueness condition, or the size of the room. **Shapes
that put two places of one meet within one corner's room are easy to build**:
a block corner on a ball where its edge grazes it, and a ball smaller than the
room on a cube corner. **But no fuse of them shows which place is folded.**
Both places lie on the ball, so the ball's other two crossings fence off a
piece between them no wider than the room. A point on a curved face is a box
four times wider than that, so `select` refuses on that piece. It is the only
thing either policy is asked, with the fold or without it. So the three
mutations the row names still pass every test, the two new ones included. The
branch can be pinned from inside `arrange.rs` (288.5). No code changed. The
shapes are pinned in `crates/rcad-topo/tests/two_meets_reach_one_corner.rs`.*

  288.1   Two shapes that reach the branch
  288.2   What the fuse does with them
  288.3   Why no fuse of these shapes can show the fold
  288.4   The three mutations
  288.5   What this leaves
  288.6   The probe

### 288.1 Two shapes that reach the branch

The room is `what_writing_the_corner_down_cost`: `16 × 4 × ε` times the
corner's largest coordinate, floored at one. At a corner no bigger than one it
is `2⁻⁴⁶`, `1.42e-14`. For both places of one meet to lie within it, the two
must be closer together than that, and three planes meet only once. So one
surface is curved, and either a line nearly touches it or the curved surface
is itself smaller than the room.

- **A block corner on a ball where its edge grazes it.** §244.2's numbers: a
  ball of radius `1 + 2⁻⁵¹` at the origin and the edge `x = −1, y = 2⁻²⁵`
  meet at `z = ±2⁻⁵¹`. Those are `2⁻⁵⁰` apart, which is `0.0625` of the room.
  A block whose corner is one of those two places has that corner exactly on
  the ball, so `passes_through` proves it `Zero` under either policy. Three
  placements are pinned: the corner at `+2⁻⁵¹` with the block above it, so the
  other place is past the corner; the same corner with the block below it, so
  the other place is on the edge; and the corner at `−2⁻⁵¹` with the block
  above.
- **A ball smaller than the room on a cube's corner.** A ball of radius `13s`
  at `(3s, 4s, 12s)` passes through the origin, since `3² + 4² + 12² = 13²`.
  The cube's three edges there meet it a second time at `6s`, `8s` and `24s`.
  At `s = 2⁻⁵²` all three are within the room (`0.094`, `0.125` and `0.375`
  of it). At `2⁻⁵⁰` two are (`0.375`, `0.5`), and at `2⁻⁴⁸` none is (`1.5`,
  `2`, `6`). That last one is the control.

The probe of 288.6 logged every call that got past the touch test. It logged
calls with two places and `count=2` for every one of the 18 grazing fuses and
for every fuse at `s = 2⁻⁵²` and `2⁻⁵⁰`. The corner and the place were exact
in every one of those lines.

### 288.2 What the fuse does with them

**Every fuse refuses, and asks one thing: `a point inside a piece of a face`,
at `Interval`, with an infinite enclosure.** That covers the three rules,
`Policy::certified()` and `Policy::tolerant(25.0)`, all three grazing
placements and all three ball sizes. That record is the whole log, so the
tolerant policy is never asked anything either.

- For the grazing shapes, the piece's inputs are the two places,
  `(−1, 2⁻²⁵, −2⁻⁵¹)` then `(−1, 2⁻²⁵, 2⁻⁵¹)`.
- For the small ball, they are the three second places, `(0, 8s, 0)`,
  `(0, 0, 24s)` and `(6s, 0, 0)`. They lie on three different planes of the
  cube, so the piece is on the ball: no face of the cube holds all three.

`a_block_corner_where_its_edge_grazes_a_ball_refuses_on_the_piece_between_the_two_places`
and `a_ball_smaller_than_the_room_on_a_cube_corner_refuses_on_its_own_face`
pin exactly that. Both panic if the fuse answers, and the message says these
shapes can then show the fold.

### 288.3 Why no fuse of these shapes can show the fold

**This is argued for the shapes tried and measured on them. It is not a proof
for every shape.** Both places lie on all three surfaces. On the curved one,
the other two surfaces cross in two curves, and both curves run through both
places. Between the places they fence off a piece no wider than the distance
between the places, so no wider than the room's diagonal. A point `select`
puts on a curved face comes back through `back_onto_the_surface` as a box that
reaches `Surface::ROUND_TRIP_ROUNDOFF`, `256 ε` times its size floored at one,
from its middle. That is four times the room. Such a box does not fit inside
the piece, and `select` refused on it under both policies.

A ball has one face over the whole sphere, so the piece is always a piece of
that face, and every fuse tried refuses on it. The fold only changes the nodes
on the block's faces. With the fold made, the block faces lose the second
place. Without it, they keep it. Either way the ball's piece is still there.
The probe run with mutation (a) logged the same outcome and the same records,
line for line, as the base.

**Where the argument stops.** The piece is only a piece of the curved face if
it lies inside that face's rim. A ball's face has no rim near it. A bar's or
a cone's side face, or a face an earlier fuse cut, can have one there. One
such case was thought through and not built: a bar's side face whose rim start
is the corner, cut by a plane nearly along its cap circle. That puts the piece
on the side face below the cap, off the face. But the same two places then
fence off a piece of the cap, between the circle and the plane's line, and
that piece is inside the cap. No shape was found where the piece misses every
face of all three surfaces. So "no fuse can show it" is measured for balls and
argued no further.

**The room cannot be shown with these shapes either.** The room decides two
things. First, it decides whether a meet's box is close enough to the corner
to fold. §253's test pins that at 2 012 times the room, and nothing pins it
lower. Second, it decides whether a second place counts. A second place
between one and a thousand rooms from the corner is what would tell the two
rooms apart, and the small ball puts one there at `s = 2⁻⁴⁸` to `2⁻⁴⁰`: `1.5`,
`2`, `6` and so on up to `1536` rooms. With the room grown, the count becomes
two, so the corner's own meet is not folded. But that meet is exact and equal
to the corner. The probe run with mutation (c) gave the same outcome as the
base for every fuse at `s = 2⁻⁴⁸`, `2⁻⁴⁴`, `2⁻⁴²` and `2⁻⁴⁰`; `2⁻⁴⁶` was not
in that run. For the first use, a meet box between
one and a thousand rooms wide at an exact corner was tried with a block whose
side leans off the grazing edge by a slope of `−2⁻⁴⁸`, `−2⁻⁵²` or `2⁻⁵²`. Its
plane is rounded when its normal is scaled to length one, and the corner comes
out as a box. Under `Policy::certified` every such fuse refused on `side of
sphere` about that box. No call in any of these shapes reached the branch with
a meet box past `0.031` of the room.

### 288.4 The three mutations

Each was a scratch edit to `crates/rcad-topo/src/fuse/arrange.rs` on this
branch, which is `ff78f99f` plus the new test file. It was undone with
`git checkout -- crates/rcad-topo/src/fuse/arrange.rs` before the next. The
command was `mise exec -- just t --workspace`, so each run holds every test of
the base and the two new ones.

- **(a) `count() == 1` to `count() >= 1`.** 2537 tests run, 2537 passed. Not
  caught.
- **(b) The uniqueness condition dropped**: `&& places.iter().filter(…).count()
  == 1` deleted, with `let _ = places;` above it. 2537 tests run, 2537
  passed. Not caught. (a) and (b) are the same function. When the first
  condition holds, `place`'s box lies inside the grown box, so it overlaps it,
  so the count is at least one.
- **(c) The room grown a thousand times**:
  `what_writing_the_corner_down_cost(node.point.enclosure()) * 1000.0`. 2537
  tests run, 2537 passed. Not caught.

**Why the new tests cannot catch them** is 288.3. The only thing either shape
asks is the ball's piece, and the fold moves nodes on the block's faces.

### 288.5 What this leaves

- **The branch can be pinned from inside `arrange.rs`.** Its `tests` module
  already builds a `BakedFace` with `bake` and an `Arrangement` from it. One
  test there can take the grazing block's face `x = −1`, with the ball as a
  cut, and call `a_corner_is_this_place` for each of the two places. It
  should return `false` for both, which catches (a) and (b). A second test can
  take the small ball at `s = 2⁻⁴⁸`, whose second places are `1.5` to `6` rooms
  from the corner. There the call should return `true` for the corner's own
  place, which catches (c). `crates/rcad-topo/src/fuse/arrange.rs` was outside
  this lane's write set.
- **`select` refuses a piece narrower than its own round-trip box.** Every
  shape here has a right answer, so the refusal is a gap and not a limit. How
  to close it was not looked at. Until it is closed, no fuse of these shapes
  can show which place `a_corner_is_this_place` folds. When it is, the two new
  tests panic and say so.

### 288.6 The probe

This patch was applied to `a_corner_is_this_place` for the counts in 288.1
and 288.3, and taken back out. It replaces the last expression of the closure:

```rust
let within = (0..3).all(|axis| {
    room[axis].lo() <= meet[axis].lo() && meet[axis].hi() <= room[axis].hi()
});
let count = places.iter().filter(|&&other| boxes_overlap(allowed, other)).count();
if std::env::var_os("Q110_PROBE").is_some() {
    let corner = node.point.enclosure();
    let allowance = super::bake::what_writing_the_corner_down_cost(corner);
    let far = |b: [Interval; 3]| {
        (0..3)
            .map(|axis| (b[axis].hi() - corner[axis].lo()).abs()
                .max((corner[axis].hi() - b[axis].lo()).abs()))
            .fold(0.0, f64::max)
    };
    let others: Vec<f64> =
        places.iter().map(|other| far(other.enclosure()) / allowance).collect();
    eprintln!(
        "Q110 within={within} count={count} places={} reach/room={} others/room={others:?} corner={:?} place={:?}",
        places.len(), far(meet) / allowance, node.point, place
    );
}
within && count == 1
```

The probe test file fused the shapes of the test file, plus the small ball at
`s = 2⁻⁴⁶` to `2⁻⁴⁰` and the leaning block of 288.3. Each ran under all three
rules and both policies, and printed the outcome and every record. The command
was `Q110_PROBE=1 mise exec -- just t -p rcad-topo --test two_meets_reach_one_corner --no-capture`.
For (a) and (c) the probe build also read `Q110_MUTANT_A` and `Q110_MUTANT_C`
and made those edits. The mutation (a) run was diffed against the base run for
the grazing shapes and the small ball at `2⁻⁴⁸` to `2⁻⁵²`, every record
included, and no line differed. The mutation (c) run was diffed against the
base run for those, the small ball at `2⁻⁴⁴` to `2⁻⁴⁰` and the leaning block,
outcomes and last records only, and no line differed.
