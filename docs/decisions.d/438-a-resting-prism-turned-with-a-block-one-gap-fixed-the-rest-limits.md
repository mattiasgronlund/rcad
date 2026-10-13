## 438. A resting prism turned with a block: one gap fixed, the rest limits — 2026-10-12

*Queue row Q1582, by lane `lane-q1582-a-resting-prism-turned-with-a-block`,
off `14f67981`, in four relays. Relay 0 wrote the builder
`a_turned_block_and_resting_prism` and the failing union at the turn `0.625`
in `ec97cdca`. Relay 1 wrote the fix in `0915c33e` and ran the sweep on to 60
turns in `6c3b4735`. Relay 2 pinned the fix's box check in `b6e138f7` and
measured how deep the ends lie in `6ffd3986`. Relay 3 wrote this section and
ran 438.5's probe. The tests are in
`crates/rcad-topo/tests/a_prism_lying_on_one_edge.rs`; the fix is
`Assembly::side_of_a_corner_of` and
`Assembly::the_three_planes_meeting_at_a_corner_of` in `fuse/assemble.rs`
(`mise exec -- just src side_of_a_corner_of`).*

> **2026-10-13 note (Q1901): the fix is deleted, with the check it fixed.**
> Queue row Q1901 dropped `Assembly::rest_on_the_outer_side_of`, the check
> 438.2 describes, since a solid touching itself along a line is an answer
> (§39.4's note of the same day). `side_of_a_corner_of` and
> `the_three_planes_meeting_at_a_corner_of` had no other caller and went with
> it, and so did 438.2's unit test
> `a_corner_whose_planes_meet_far_outside_its_box_is_read_where_it_was_written`,
> which called the check. `git show 0915c33e b6e138f7` has them. No corner of
> a wall is asked any more, so 438.2's gap has nothing left to come back
> through: the union at `0.625` and the sweep's counts in 438.1 pass unchanged
> (`a_resting_prism_with_one_end_a_rounding_inside_a_turned_top_unites_as_they_stand`,
> `a_resting_prism_on_a_turned_blocks_top_answers_as_they_stand_or_is_refused_by_name`).

### 438.1 The shape, and how its 360 fuses end

The block of `a_prism_lying_on_one_edge.rs`, `−4 ≤ x ≤ 4`, `−3 ≤ y ≤ 0`,
`0 ≤ z ≤ 10`, and the resting prism, the triangle `(0, 0)`, `(3, 2)`,
`(−1, 2)` from `z = 2` to `z = 8`, its edge along the block's top `y = 0`.
Both are turned about `x` by `Isometry::rotation` at `k` fortieths of a
radian, `k = 1` to `60`: Q1281's `a_turned_block_and_wedge` with the resting
triangle in place of the wedge. Turning rounds the prism's walls, and each end
of its edge, where the two walls and that end's plane meet, asked exactly
(`sides_of_the_edge`), lands a rounding above the turned top or a rounding
inside it. At no turn is an end on the top.

Every rule both ways round is 6 fuses a turn, 360 in all, under
`Policy::certified()`
(`a_resting_prism_on_a_turned_blocks_top_answers_as_they_stand_or_is_refused_by_name`):

| ends        | turns | as they stand | `DynamicRangeTooWide` | `HoleWindsWithTheOuterWire` |
|-------------|------:|--------------:|----------------------:|----------------------------:|
| both above  |    17 |           102 |                     0 |                           0 |
| one inside  |    21 |             6 |                    93 |                          27 |
| both inside |    22 |             6 |                   120 |                           6 |

"As they stand" is the union written as the two solids as they stood before
the fuse, the intersection as no solid, and each difference as its first
solid whole, each solid checked against the turned solid it is
(`differs_from_as_they_stand`). With an end inside, that is §417.4's answer,
the edge taken to lie on the top.

The 12 answers with an end inside are at fortieths 9, 25 and 47. At 9 and 47,
both ends inside, the intersection both ways round and the block less the
prism answer. At 25, one end inside, every rule answers.
`TheFuseWroteABrokenAnswer { broken: HoleWindsWithTheOuterWire }` comes for
the union both ways round and the block less the prism, at fortieths 3, 4, 14,
31, 32, 33, 39, 42, 43, 50 and 57: 33 fuses. No other refusal and no wrong
answer comes at any turn.

### 438.2 The gap: a corner read by the point it was written as

At the turn `0.625` (`k = 25`) the end at `z = 2` is a rounding inside the
top and the end at `z = 8` a rounding above it. The intersection and both
differences answered as they stand, and the union was refused
`ResultIsNotManifold { count: 4 }` both ways round. Its answer exists: it is
§417.4's answer, as the other four fuses' are at that turn. So it was a gap.

The four darts along the edge, from where it crosses the top to the end at
`z = 8`, were the top's two pieces and the prism's two walls.
`Assembly::rest_on_the_outer_side_of` parts such a line when each wall rests on
the outer side of the top. It asked each corner of a wall which side of the top
it is on, by the point the corner is written down as: the middle of the box
round where three planes meet. Q1281 had already stopped asking the corners of
the top's own pieces. The wall's far corner, where the two walls and the end at
`z = 8` meet, was written `(3.36e-16, −4.680778183523698, 6.487704956041742)`,
a rounding inside the top, while its three planes meet strictly above it. The
wall read as reaching inside, the line was not parted, and four faces met along
one edge.

`0915c33e` asks each corner through `side_of_a_corner_of`. Where the edges of
the face at the corner run along exactly three planes, a plane and the same
plane turned round counted once, and the three are proved exactly to meet
inside the box the corner was solved as, give or take its wobble, it asks
`side_of_a_plane_where_three_planes_meet` of them, which answers exactly.
Otherwise, and for a corner a tolerant policy moved (§417.7), it reads the
written point, as the base did. It also asks each corner once; the answer does
not depend on the order, since any corner on the inner side ends the parting.
The tests that fail without it are
`a_resting_prism_with_one_end_a_rounding_inside_a_turned_top_unites_as_they_stand`
and the sweep's pinned counts.

**Why the box is asked.** Three planes can meet far from the corner. Where a
face's side runs on nearly straight past a corner, from one wall to another
lying in nearly the same plane, the three meet far along it, at a point the
face does not have. `b6e138f7` builds that: a triangle on `z = 0` resting on
`y = 0`, whose corner `(1, 1)` sits between the wall `x + y = 2` and a wall
turned `2^−41` from it, in a box of `2^−30`. Its three planes meet near
`(257, −255, 0)`. With the box asked, the corner is read where it was written
and the face rests on the outer side;
`a_corner_whose_planes_meet_far_outside_its_box_is_read_where_it_was_written`
fails with `Some(three)` in place of `inside.then_some(three)`. No fuse in
rcad-topo's tests meets the case: relay 2 turned the check into a panic, and
`mise exec -- just t -p rcad-topo --no-fail-fast` ran 2000 tests without
reaching it.

### 438.3 The limits: every exact answer with an end inside needs the sliver

Where an end is inside the top, the prism and the block overlap in a sliver
along that part of the edge. Every exact answer then has a side across the
sliver along the top, from where one wall meets the top to where the other
does.

Relay 2 measured how deep each of the 65 ends inside lies, with a probe that
moved the top in until the end read above it, bisecting. The deepest is the
end at `z = 8` at the turn `1.475` (`k = 59`), `9.1e-16` deep, 1.03 of the gap
between doubles there. Every other end is under 0.58 of a gap, every end at
`z = 2` under 0.37, and the shallowest, at `z = 2` turned `0.025`, is
`3.8e-19`. The probe was not kept.
`every_end_of_the_resting_prisms_edge_inside_the_turned_top_is_under_two_roundings_deep`
pins what it found: all 65 under two gaps, and only that one past one gap.

The walls lean out 3 across for 2 up and 1 across for 2 up, so the side across
the sliver is twice its depth: under `3.6e-15`, beside a block reaching 10
from the origin. That is a range above `2.7e15`, against the `1e11` a model
supports (§3.5). So under `Policy::certified()` the 213 `DynamicRangeTooWide`
and the 33 `HoleWindsWithTheOuterWire` refusals are limits by range, class
`refuses/D`, as §417.5 rules for the turned cube's tip.

The 33 are refused under the wrong name, though: the fuse wrote an answer and
its own check caught it, where in the other 213 the range check came first.
A row asks why.

Not shown: that no double lies in the sliver. At a small turn the end's
smaller coordinate has finer doubles than its larger one, so the argument is
by range alone, not by what each axis can hold.

### 438.4 The wedge to 60 turns

Q1281 pinned the wedge at `k = 1` to `24`. `TURNS` is now `1..=60`, and at the
branch's tip the wedge answers every rule both ways round at every turn
(`a_wedge_lying_on_its_edge_on_a_turned_blocks_top_answers_every_rule_both_ways_round`),
its edge on the top exactly
(`the_turned_wedges_edge_is_on_the_turned_blocks_top_at_every_turn`).

### 438.5 `VolumeIsNotPositive` was gone before the fix

Q1281's lane, after its fix `96b9b052`, swept this shape and met the
intersection refused `VolumeIsNotPositive` at 15 turns: `k` = 1, 2, 3, 13, 19,
21, 30, 39, 40, 42, 49, 51, 54, 58 and 60. This sweep meets it at none.

Relay 3 switched the fix's reading off, with `if false &&` in front of the
test in `side_of_a_corner_of`, so that every corner is read by its written
point as at the base. That edit was run and taken out, never committed. The
sweep then failed on the two unions at `0.625`, `ResultIsNotManifold { count:
4 }`, and on nothing else, and the test lists every fuse that is neither as
they stand nor refused by one of the two names. So `VolumeIsNotPositive` was
already gone at `14f67981`, removed by a commit between `96b9b052` and it.
Which one is not traced. `95414ee2` and `7b425551` (Q1614 and Q1664), which
ask the answer's range earlier in the fuse, are the likeliest, and that is a
guess nobody has run.

### 438.6 Not run: the tolerant policy

`Policy::tolerant(100.0)` has not been run on this sweep. §417.5 makes the
turned cube's tip a gap under it, since the answer meant exists, and the same
may hold here; nobody has run it, so it has no class (§360.5).

### 438.7 What this leaves

- A row, tier 2: run the sweep under `Policy::tolerant(100.0)` and say for
  each refusal whether §417.4's answer exists and is reached.
- A row, `refuses/D`, tier 5: the 33 `HoleWindsWithTheOuterWire` fuses refused
  by the limit's name, or a reason why not.
- A row, tier 2.2, small: `just claim-section --help` claimed the number
  after this one for the title `--help` during this row. That number is a gap,
  which costs nothing (§159).
- No row: which commit took `VolumeIsNotPositive` away (438.5). A refusal
  that is gone is not a defect.
