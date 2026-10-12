## 323. A ray leaves a box beside a torus, and a ring's face counts the right way round — 2026-09-28

*Row Q102 (`lane-q102-a-point-a-ray-can-leave`, off `e7b1894a`, merged as it
was onto `0a7142ff` in `lane-q102-a-ray-leaves-a-box-2`, 323.8). §243.5 left
a bend less a copy of itself turned an eighth of a turn refusing on "a point in
a piece that every solid could place", and §243.6 named two ways out: give the
piece an exact point, or let a ray leave a box. The second is taken, in
`classify.rs`, and firing from exact points turned up a ring's face counting
its crossings backwards. The fuse now places every piece rightly and stops
later, writing them back out.*

  323.1   What refused
  323.2   Which way out
  323.3   Why one point can answer for a box
  323.4   A ring's face counted its crossings backwards
  323.5   Where the bend stops now
  323.6   The pins
  323.7   What this leaves
  323.8   Sent again onto `0a7142ff`, and the rebuilt bend's pin

### 323.1 What refused

Reproduced on `e7b1894a` in `tests/a_bend_less_a_turned_copy_of_itself.rs`:
`a_quarter_bend(4, 1)` and a second one placed an eighth of a turn round `z`.
Difference, union and intersection all came back `Err(Undecidable)` with one
record, "a point in a piece that every solid could place", whose inputs are
`(3, 0, 0)` and a box round `(5/√2, 5/√2, 0)`: the first bend's wall between
`y = 0` and the turned bend's first cap, as §243.5 says.

Every candidate in a curved piece is a box — `candidate_points` blends the
corners and `back_onto_the_surface` puts each back through the surface's two
coordinates. A box has no exact line, `where_a_line_meets_a_surface` refuses a
ring anything but an exact line (`WhereALineMeetsARingNeedsTheLineExactly`),
and the classifier answers `CannotFireARayAtAFacesSurface`.

### 323.2 Which way out

**Giving the piece an exact point is not a way out on its own**, and that was
measured before it was argued. With the box route below in and nothing else,
the next piece refused under the same name was flat: the turned bend's first
cap, whose candidate in the middle of the tube is where the ring crosses the
cap's plane — `the_middles_of_a_round_wire` hands that over as an *exact*
point, `(2.82842712474619, 2.8284271247461903, 0)`. The piece was refused
all the same. The new test's second case asks the exact point an ulp from it,
`4·(1/√2)` rounded on both axes, and on `e7b1894a` that is
`CannotFireARayAtAFacesSurface`. A ray's line is exact only where the point is
exact *and* its step to the target is, and that step was not. So an exact point
in the piece is not enough; its step has to be exact too.

And a curved piece has no exact point on it in general. A point with `f64`
coordinates is a fraction over a power of two, and a point like that on a
circle whose radius is a power of two is one of its four points on the axes:
`a² + b² = 4ᵏ` has no other whole solutions, because two odd squares add to
two more than a multiple of four and an odd and an even add to an odd number.
The bend of ring four has `(3, 4, 0)` on its outer equator only because five is
not a power of two.

So the ray leaves from beside the box, and it does it in `classify.rs`, where
the refusal is made — every caller of `classify_uncertain_point` gets it and
`fuse/select.rs` is not touched. `what_a_point_beside_it_says` runs only
once a ray from the point itself has been refused
`CannotFireARayAtAFacesSurface`, so no answer that came back before moves. For
each of the three targets in turn:

1. An exact point beside the middle `m` of the box, with an exact step to the
   target `t`: round the step `d = fl(t − m)` first and take the point as
   `t − d`. That subtraction is exact where `|t| ≥ |m|` (Dekker's), so the step
   from the point to the target is `d`. The step is checked with an interval
   subtraction rather than trusted.
2. The box grown to hold that point.
3. No face of the solid meets the grown box (`no_face_meets`, 323.3).
4. The exact point is fired from with the ordinary ray. A pass with no graze
   is the answer. A graze, a refusal, or a grown box a face meets hands over to
   the next target, and when none is left the first refusal stands.

### 323.3 Why one point can answer for a box

The brief's condition was that a ray from a box be shown to classify every
point of the box the same way. A box is convex, so the segment between any two
of its points stays inside it. A segment that meets no face of the solid does
not cross its skin, so its two ends are both inside or both outside. So when no
face meets the grown box by its corners, the exact point's answer is the
answer for every point of it, and the box the caller handed over is inside the
grown one — **that held only for the corners, not yet for the stored planes**
(queue row Q1068, below).

*2026-10-06, Q1068: the gap between the corners and the stored planes is
real. `no_face_meets` reads a straight-sided face by its fan triangles, built
from the face's corners; on a turned face the fan is a rounding off the
face's stored plane, so the grown box round a point `P` and the exact point
`B` beside it can miss every triangle while the stored plane still passes
between them. The ray from `B` then parts at that face, and the old winding
left the crossing out, counting one off, so `P` could come out on the wrong
side of the plane. The fix asks the face's own stored plane to place `P`
directly where `B`'s ray parts there (`what_its_plane_says_beside_it`, already
used for a ray from `P` itself), since that function's proof is about `P`'s
own box stretched to the plane and does not lean on `B` or on
`no_face_meets`. A turned bend-and-block probe over 270,000 points found 271
wrong before the fix and 3 after — each a point whose own line is exact,
Q1156's shape, not this path — and 76 refuse
`CannotFireARayAtAFacesSurface` (`4bb0e7b2`, pinned by
`a_point_inside_whose_ray_from_beside_parts_at_the_floor_is_not_put_outside`
and
`a_point_outside_whose_ray_from_beside_parts_at_the_top_is_not_put_inside` in
`tests/a_point_beside_a_turned_flat_face.rs`).*

`no_face_meets` asks each face by the description the ray counts it by. A flat
face with straight sides is its fanned triangles, so the box has to miss every
one of them: apart from it along a coordinate axis or across one of its sides,
or off the plane through it by a decided `orient3d` (Q342, Q523, below) — or,
where the fan of a face that is not convex reaches outside it, the face's
outline has to say `NotHeld` of the box. Every other face
is the part of its surface it holds, so either the surface's sign is decided
over the box, or `what_this_face_holds` says `NotHeld` of the box.

Both rest on **a sign decided about a box holds for every point in the box**.
That is not a new assumption. `count_where_the_ray_meets_it` already asks a
root's *box* whether the face holds it and drops the crossing on `NotHeld`, and
that is sound only under the same contract.

A ray from the exact point that says `OnBoundary` would contradict the proof,
and is asserted against rather than answered.

What is not answered, and stays refused: a box a face meets — the new test
pinned a box on the first bend's own wall refused against that bend, and
since Q1764 pins it `Undecidable` — and a box
with an infinite end, which has no finite middle. **A box only the ring's own
wall meets is not refused any more but `Undecidable`** (queue row Q1763,
below).

*2026-10-11, Q1763: a box on a ring's wall that no other face grazes is
`Undecidable`, as a box on a ball's wall is.* The case is a box the ring's own
sign cannot place, which the ring's face holds some of, and which no other face
of the solid grazes. The example is the box round `(3, 4, 0)` on the outer
equator of the glued bend's wall, which no face of the block meets; the block
alone says `Outside`. At `b8c4892c` the bend refused that box
`CannotFireARayAtAFacesSurface` under the certified policy, and a ball of
radius 5 about the origin, whose wall `(3, 4, 0)` is on too, said
`Undecidable`. The tolerant policy was not measured at that base. Q1764
measured it at `7b257c32`, the `main` it merged this row's branch onto, with
a probe test it did not keep: under `Policy::certified()` and under
`Policy::tolerant(100.0)` alike the bend said
`CannotFireARayAtAFacesSurface`, the ball `Undecidable` and the block alone
`Outside`. So the box was refused whatever the policy.

The decision is `Undecidable`. `classify.rs` keeps the two answers apart,
citing §4.2: `Undecidable` says the predicates were asked and none could
prove an answer, and `CannotFireARayAtAFacesSurface` says no ray was put to
the ring's surface. Here the ring's sign was asked, could not place the box,
and the face could not show it holds none of it, so the ring grazed every ray
from the box. That graze comes before any ray is fired at the ring, since
`count_where_the_ray_meets_it` stops at the refusal, so a ray fired at the
ring could not have undone it. The refusal told a caller that the missing ray
was why there was no answer, and it was not; a smaller box round the point is
what can answer. No policy tells the two apart here either: under
`Policy::tolerant(100.0)` the bend and the ball both say `Undecidable` now, and
`Undecidable` already covers a box no policy settles. Under §16.10 this is a
limit, not a gap. The bend's ring is 4 and its tube 1, and
`(9 + 16 + 0 + 15)² = 1600 = 4 · 16 · 25`, so `(3, 4, 0)` is on the wall
exactly, the box holds points inside the solid and points outside it, and no
one answer is right for all of it.

The code: `Crossings::add` now counts a face's graze as one that throws the
ray away (`a_graze_that_stands`) wherever that face's trimming was followed,
and no longer asks whether a ray was fired at the face. A graze from a face
whose trimming nothing follows still throws no ray away (Q1740), since
following that trimming might have put the box off the face.

The pins: `a_box_on_a_bends_wall_that_no_face_of_the_block_meets_is_undecidable_as_on_a_ball`,
in `tests/a_ray_from_beside_a_box.rs`, asks the bend and the ball under both
policies, and the block alone. The row asked which fuse pins move: none. Every
test in the workspace was run with `--no-fail-fast`, slow tier included, and
one pin moved, not a fuse's:
`a_bend_less_a_turned_copy_of_itself.rs`'s
`a_box_no_face_meets_is_answered_and_one_on_the_wall_is_not`, the test the
paragraph above says pins a box on the first bend's own wall refused, and
323.6 names. That box now gets `Undecidable { enclosure: 9.84755388344638e-10 }`,
and the pin moves with this row. It moved in Q1764, which merged this row's
branch at `24274fa5` rather than rebasing it, so the shas above still name what they
named: the pin asserts `Undecidable` of the box now, where it asserted the
refusal naming the wall. Read from the code and not measured:
`fuse/select.rs`'s `settle_against_each` gives up on the point for either
answer. `fuse/assemble.rs`'s `one_shell_is_inside_another` asks exact corners
through `classify_point`, and `Torus::side_of` decides an exact point unless a product overflows or
falls into the subnormals;
only a corner exactly on the ring's wall that the face's trimming cannot place
would now be `Undecidable` there rather than refused, and no pin moved for it.
One thing a fuse does see: `settle_against_each` keeps the ray's records only when the
answer is not `Undecidable`, so such a point now adds no records to the fuse's
log.*

*2026-10-12, read from the code at `7f18451b` and not measured: a third
caller, where the two answers do not end the same way.* `overlap.rs`'s
`a_corner_of_one_is_inside_the_other` asks exact corners through
`classify_point` as `one_shell_is_inside_another` does, so it reaches only the
same corner, exactly on a ring's wall that the face's trimming cannot place.
But it skips the refusal and widens on `Undecidable`. For that corner,
`shares_space_with` now says `Undecidable` where it said `NothingFound`, and
`Model::check_no_two_solids_share_space` refuses
`CannotTellWhetherTwoSolidsShareSpace` where it passed. That is a refusal where
there was a search that came up empty, never a wrong answer, and no shape is
known to reach it.

A box on the plane of a straight-sided face's fan triangle and off the
triangle is answered now (row Q342). It was refused, because only the plane
was asked; `misses_the_triangle` first holds the box and the triangle up
against each coordinate axis, and corners all below the box's lower end, or
all above its upper end, is a sign decided about the whole box with nothing
rounded. `classify.rs`'s tests `a_box_beside_a_blocks_side` pin seven boxes on
the planes of a block's sides, beside a quarter bend: on the base,
`(6, 1, 0)` was `CannotFireARayAtAFacesSurface`. Two more boxes off such a face
are answered since row Q523, and the same tests pin both. A box inside a
triangle's own bounding box and off it, on a turned block's top, is settled by
the nine directions across the triangle's sides (`apart_across_a_side`). A box
in the notch of an L, which a fan triangle covers though the face does not, is
settled by the face's outline where every corner is on the stored plane
(`its_fan_adds_up_to_nothing_over`). The same L turned about a slanted axis is
still refused, since its corners are then a rounding off the plane, and
`a_box_in_the_notch_of_a_turned_l_is_still_refused` pins it: every point of
that box is outside the L, so it is a gap (§16.10).

**No public call builds a solid that asks it yet**, and that is demonstrated,
not argued: `tests/a_ray_from_beside_a_box.rs` pins the three ways there are.
A fuse never asks such a box, since it asks a piece of one solid against the
other alone. A bend glued onto a block's side through `SolidBuilder` is
refused `AClosedEdgeIsWalkedAgainstItsSurface` whichever way the cap circle
is walked, though one of the two is right — so the pins above use one solid
holding a bend's shell and a block's, written in-crate the way a fuse writes
two pieces that do not touch. A block with a hollow cannot be described face
by face (`ASolidBuiltFaceByFaceHasOneShell`), and an L-shaped block less a
bend wholly inside it is refused as a quartic before any piece is placed. The
glued bend's refusal is a gap of its own, not traced: its likely cause is
323.7's open question, `validate.rs` multiplying the torus's out-of-the-tube
gradient by the wall's `Reversed`, which reads the wall's material as lying
outside its tube.

*2026-10-11, Q721: the pins above moved off `classify.rs` and onto a public
call.* `classify.rs`'s `mod a_box_beside_a_blocks_side`, which the paragraph
above named as the home of the seven box pins, is gone. They run now as four
tests in `tests/a_ray_from_beside_a_box.rs` —
`a_box_beside_a_side_of_the_block_a_bend_grows_out_of_is_answered`,
`a_box_beside_the_top_of_a_turned_block_a_bend_grows_out_of_is_answered_outside`,
`a_box_in_the_notch_of_an_l_a_bend_grows_out_of_is_answered_outside` and
`a_box_in_the_notch_of_a_turned_l_a_bend_grows_out_of_is_still_refused` — which
ask the same boxes through the public `classify_uncertain_point` of the glued
bend (two shells, a torus face and a flat face with straight sides) rather
than through `no_face_meets` on a solid built by the crate-internal
`Model::add_solid`. **No public call builds a solid that asks it yet** no
longer holds: the glued bend is one, which is what moved the pins onto it.

### 323.4 A ring's face counted its crossings backwards

With the box route in and nothing else, the fuse got past selection and came
back `ResultIsNotManifold` (count 1 for difference and union, 3 for
intersection). A temporary `eprintln!` in `select::choose` showed why: the
turned bend's wall from `90°` to `135°` was placed **inside** the first bend,
which ends at `90°`. A second `eprintln!` of each ray's tally gave `−1` for the
middle of the tube at `45°` against the first bend, and `−2` for the lattice
point `(−1, 3.5, 0)`, past its cap on `x = 0`, through the route that was
already there. One ray against a solid comes to nought or one whichever way
its faces are oriented, if they are oriented consistently; `−2` is not a count
any solid gives.

The cause is the torus arm of `gradient_at`. It is the quartic's own gradient,
which points out of the tube, and the count turns it round for a face used
reversed. But a face's orientation is read against the surface's own normal,
and a ring's points *into* its tube: `Torus::point_at` takes the tube angle
first, and at the outer equator `∂/∂tube × ∂/∂ring` is `r·w × (R + r)·u`,
which points towards the axis. That is why a bend's wall is stored `Reversed`
(`tests/rings/mod.rs`) — and why it was counted the wrong way round. The arm is
turned round now, and the two rays came to `+1` and `0`.

**That was a wrong answer on `main` from the public API.** `classify_point` of
`(−1, 3.5, 0)` against a quarter bend said `Inside`; the new test's first case
fails on `e7b1894a` with `left: Inside, right: Outside`. What hid it is that a
ray through a bend's wall alone comes to `−1`, which is not zero and was read
as inside, rightly by accident.

**Not changed, and possibly the same sign:** `validate.rs`'s `gradient_of` is
the same quartic gradient, and it is multiplied by the face's orientation in
the closed-edge rule that `Model::validate` and `holds.rs`'s
`what_a_ring_face_between_two_planes_holds` both read. §243.6 records
`Model::validate` refusing a valid three-quarter bend and passing one walked the
wrong way. Whether that is this sign was not checked; neither file is in this
row's write set. `mesh.rs` has a `gradient_of` of its own.

### 323.5 Where the bend stops now

With both changes in, the same `eprintln!` printed each piece's placement
against the other bend, and every one is what the geometry says:

| piece | placed against the other bend |
|---|---|
| first bend's cap on `y = 0` | outside |
| first bend's cap on `x = 0` | inside |
| first bend's wall, `0°` to `45°` | outside |
| first bend's wall, `45°` to `90°` | on it, facing the same way |
| turned bend's cap at `45°` | inside |
| turned bend's cap at `135°` | outside |
| turned bend's wall, `45°` to `90°` | on it, facing the same way |
| turned bend's wall, `90°` to `135°` | outside |

So the difference keeps the cap on `y = 0`, the wall to `45°` and the cap at
`45°` turned round; the union keeps both end caps, the first bend's wall and
the turned bend's wall past `90°`; the intersection keeps the cap at `45°`, the
wall from `45°` to `90°` once, and the cap on `x = 0`.

All three then refuse `TheFuseWroteABrokenAnswer { broken:
CoedgesRunTheSameWay }` — the refusal §243.5 recorded for a bend and its
bit-for-bit copy — writing those pieces back out. That is in `assemble.rs`, out
of this row's write set, and it was not traced.

### 323.6 The pins

`tests/a_bend_less_a_turned_copy_of_itself.rs`, three tests, each failing on
`e7b1894a`'s `classify.rs` with the tests kept:

- `a_point_past_a_bends_cap_is_outside_it_and_one_in_its_tube_is_inside` —
  eight exact points against a quarter bend. On the base, `(−1, 3.5, 0)` is
  `Inside`.
- `a_box_no_face_meets_is_answered_and_one_on_the_wall_is_not` — on the base,
  the exact point in the middle of the tube at `45°` is
  `CannotFireARayAtAFacesSurface`.
- `a_bend_less_a_copy_turned_an_eighth_of_a_turn_places_every_piece` — on the
  base, `Undecidable`.

One pin outside the first lane's write set moved with it:
`building_a_solid_out_of_faces.rs`'s
`a_bend_built_face_by_face_is_the_bend_a_quarter_bend_builds` expected the
refusal of 323.1 and failed with "expected the piece of the wall not to be
placed, got TheFuseWroteABrokenAnswer { broken: CoedgesRunTheSameWay }". It
asserted twice more what this row changed — the corners that refusal named,
and that a ray from the blend of those corners against the turned bend is
`CannotFireARayAtAFacesSurface`. The row was sent again with that file in its
write set, and 323.8 says what the pin says now.

### 323.7 What this leaves

- A bend fused with a copy of itself turned an eighth of a turn keeps the right
  pieces and fails writing them back, `CoedgesRunTheSameWay`, for all three
  rules (323.5). Not traced. The same refusal as a bend and its bit-for-bit
  copy (§243.5), so plausibly one cause.
- Whether `validate.rs`'s torus gradient, multiplied by a face's orientation,
  is the sign 323.4 fixed here, and whether it is §243.6's three-quarter bend.

### 323.8 Sent again onto `0a7142ff`, and the rebuilt bend's pin

The first lane's verifier found the pin 323.6 names broken outside its write
set, so the row was sent again as `lane-q102-a-ray-leaves-a-box-2`, with
`building_a_solid_out_of_faces.rs` in the write set. The first lane's branch,
tip `fef89d7e`, was merged onto `0a7142ff` with no conflicts and not rebased,
so every sha above still names what it named. That base is past `e7b1894a` by
Q79 (§321), Q100 (§322), Q116 (§325) and Q168 (§326).

The pin now says three things.

- **Both descriptions of the bend, less the turned bend, refuse
  `TheFuseWroteABrokenAnswer { broken: CoedgesRunTheSameWay }`**, and refuse
  it word for word alike. That is still the refusal the fuse half of that
  oracle compares, one stop later than before. The right answer is the eighth
  of the bend from `y = 0` to the turned bend's first cap, `π²` by Pappus, so
  when 323.7's first stop is closed the pin moves again and that half becomes
  `the_same_numbers`, as the cube's, the bar's and the ball's are.
- **The wall piece's corners are written from the shape**, `(3, 0, 0)` and
  `(5/√2, 5/√2, 0)`, because the refusal whose record named them is gone.
  `5/√2` goes in as a range, checked to hold it by squaring its two ends.
- **A ray from the blend of those corners against the turned bend is
  `Outside`**, and that answer is worked out from the shape rather than read
  off `classify.rs`: the blend is `3.72` from the axis, in the tube, but only
  `28°` round the ring, and the turned bend runs from `45°` to `135°`. On
  `e7b1894a` it was `CannotFireARayAtAFacesSurface`.

The table in 323.5 was printed again on the merged tree, with the same
temporary `eprintln!` in `select::choose`, and every row holds for all three
rules. The wall piece the old refusal stopped on has the corners the pin
writes: `(3, 0, 0)` and `(5/√2, 5/√2, 0)`.
