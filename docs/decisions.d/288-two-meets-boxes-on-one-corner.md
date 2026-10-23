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
thing either policy is asked, with the fold or without it. So no fuse catches
the three mutations the row names. The shapes' refusals are pinned in
`crates/rcad-topo/tests/two_meets_reach_one_corner.rs`. **The fold itself is
pinned in `crates/rcad-topo/src/fuse/arrange.rs`'s `tests` module**, by two
tests that call `a_corner_is_this_place` directly and catch all three
mutations. They came from a second run, `lane-q110-two-meets-one-corner-2`
(288.4, 288.5). No kernel code changed. The ball's piece is Q112's refusal,
and it is a gap (288.5).*

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

**The second run caught all three from inside `arrange.rs`** (288.5). Each
edit was made twice. The first time was on `ff78f99f` itself, with the
worktree detached there, and the command was `mise exec -- just t -p
rcad-topo`. The second was on the branch at `1f525393`, which adds the two
unit tests, and the command was `mise exec -- just t -p rcad-topo --lib -E
'test(/two_places_of_one_meet_inside|a_corner_is_its_own_place/)'`. Each was
undone with `git checkout -- crates/rcad-topo/src/fuse/arrange.rs`. (b) was
the first run's edit, and (c) was `* 1000.0` on the same argument.

| Edit | `ff78f99f`, all of `rcad-topo` | The branch, the two new tests |
|---|---|---|
| (a) `== 1` to `>= 1` | 1037 run, 1037 passed | the grazing one fails |
| (b) uniqueness dropped | 1037 run, 1037 passed | the grazing one fails |
| (c) room × 1000 | 1037 run, 1037 passed | the small-ball one fails |

Under (a) and (b) the grazing test's fold answers `true` for the corner's own
place, `(−1, 2⁻²⁵, 2⁻⁵¹)`. Under (c) the small-ball test's fold answers
`false` for the corner itself. Two more edits bound the room from above.
`* 1.5` makes the small-ball test fail, because the second place on the edge
along `x`, `(6s, 0, 0)`, then touches the room, and a touch counts. `* 1.25`
passes.

**Reverting `arrange.rs` to the base proves nothing here.** The new tests
live in that file, so the revert takes them out with it. The three mutations
are the proof.

### 288.5 What this leaves

- **The branch is pinned from inside `arrange.rs`.** The first run named the
  two tests below and could not write them, because `arrange.rs` was outside
  its write set. The second run wrote them in that file's `tests` module. Both
  bake the two solids, cut the face by the ball, and draw the arrangement as
  far as `place_the_corners`. That is the state `place_the_meets` calls
  `a_corner_is_this_place` in. They do not use `Arrangement::of`, because the
  steps after `place_the_meets` fold nodes together. Each works out the places
  the way `place_the_meets` does, and asks about every one of them.
  - `two_places_of_one_meet_inside_a_corners_room_are_neither_folded_into_it`
    takes the grazing block's face `x = −1`, with the block above the corner
    `(−1, 2⁻²⁵, 2⁻⁵¹)`. It asks about the block's side `y = 2⁻²⁵` and the
    ball. The fold answers `false` for both places, so both stay nodes and
    the policy is left to say whether they are one point. That catches (a)
    and (b).
  - `a_corner_is_its_own_place_when_the_second_place_is_outside_its_room`
    takes the unit cube's face `z = 0` and the small ball at `s = 2⁻⁴⁸`. It
    asks about the ball with the wall `x = 0`, and then with the wall
    `y = 0`. The fold answers `true` for the corner `(0, 0, 0)` and `false`
    for the second place, `(0, 8s, 0)` and then `(6s, 0, 0)`. That catches
    (c), and it pins the room from above to less than `1.5` times its size.

  Both shapes did what the first run said they would. Drawing the arrangement
  asked no policy anything, and both tests check that the log stays empty, so
  the answers hold under any policy.
- **`select` refuses a piece narrower than its own round-trip box.** Every
  shape here has a right answer, so the refusal is a gap and not a limit. How
  to close it was not looked at. Until it is closed, no fuse of these shapes
  can show which place `a_corner_is_this_place` folds. When it is, the two
  tests in `two_meets_reach_one_corner.rs` panic and say so.
- **That answers Q112.** Q112 asks whether §248.3's refusal is a limit or a
  gap: `a point inside a piece of a face`, with an infinite enclosure, in
  `a_block_edge_grazing_a_ball.rs` at `h = 0`. That record has the same two
  inputs as the grazing fuses here, `(−1, 2⁻²⁵, ∓2⁻⁵¹)`, so it is the same
  piece of the ball, the one between the two places. By the bullet above it
  is a gap. The fix belongs in `select`, and Q112 holds it.

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
was `Q110_PROBE=1 mise exec -- just t -p rcad-topo --test merged -E 'test(/^two_meets_reach_one_corner::/)' --no-capture`.
For (a) and (c) the probe build also read `Q110_MUTANT_A` and `Q110_MUTANT_C`
and made those edits. The mutation (a) run was diffed against the base run for
the grazing shapes and the small ball at `2⁻⁴⁸` to `2⁻⁵²`, every record
included, and no line differed. The mutation (c) run was diffed against the
base run for those, the small ball at `2⁻⁴⁴` to `2⁻⁴⁰` and the leaning block,
outcomes and last records only, and no line differed.
