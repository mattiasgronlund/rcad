## 431. Two flat faces walking one ring both ways are left out — 2026-10-11

*Queue rows Q1528 and Q1658. The rule was built under Q1528 by lane
`lane-q1528-the-tilts-a-cube-tipped-on-its-edge-refuses`, off `2a15f65d`: the
failing test in `6c027e78` ("pin the union at five tilts it is refused at,
count 4") and the fix in `76d65df9` ("leave out two flat faces that walk one
ring both ways"). That lane stopped because the fix moved the comb pins, which
lay outside its write set. Lane `lane-q1528-q1658-leave-out-a-ring-walked-both-ways`,
off `984e8603`, merged it in `73d0ef2e`, re-pinned the combs in `b492681c`,
measured the notch in `3a88e295`, rewrote the doc comments in `5e387879`, and
wrote this section in three relays. The judgement in 431.3 is the second
relay's. `leave_out_each_pair_of_flat_faces_walking_one_ring_both_ways` and
`walk_one_ring_both_ways`, both in `fuse/assemble.rs`
(`mise exec -- just src leave_out_each_pair_of_flat_faces_walking_one_ring_both_ways`).*

### 431.1 The rule

`Assembly::from_the_pieces` calls
`leave_out_each_pair_of_flat_faces_walking_one_ring_both_ways` once every
piece is placed, just before `leave_out_what_no_wire_walks` and before any
face is joined. It looks for two flat faces with one ring each, every side of
it straight, that walk the same edges, each edge once, and every edge the
other way round (`walk_one_ring_both_ways`). When the two sit on planes that
`agreement_of_two_planes` says are `Different`, both faces are left out. The
edges only they walked then have no face, and `leave_out_what_no_wire_walks`
leaves those out too, with every corner no edge left ends at (§417.4, Q1442).

**Why it is right.** As its corners are written, such a pair is one polygon
twice, facing both ways, and bounds no space between the two. Only the planes
the two are stored on differ, and a plane carries no corner. Leaving the pair
out takes one use off each side of every edge the pair walks, as a single wire
that `encloses_something` leaves out does, so every edge left still has its
faces paired. The rule folds no corner, since the two rings' corners are
already one by name, so it runs under every policy, as §365.7's two rules,
§417.4's third and §423's fourth do. It is the fifth of the family.

**Why §423's rule does not reach it.** §423's
`leave_out_each_flat_ring_another_plane_walks_back` would leave both faces out
too, but it asks once the faces are joined, and joining needs exactly two
faces along each edge to know which face is across it. An edge one such pair
shares with two other faces has four, and the joining refuses
`ResultIsNotManifold { count: 4 }` before §423's rule is asked. This rule asks
before, and only of whole rings, which no edge with more faces along it can
confuse.

**The case that needed it.** A cube tipped onto one edge along another cube's
floor (§423.1, `a_cube_tipped_onto_one_edge_along_another_cubes_floor.rs`).
At `t = 0.002` the tipped cube's corner at `x = 5` rounds onto the floor, at
`(4.999999999999999, 5, 0)`, and the union kept a sliver of the cube's bottom
face and a sliver of its side `x = 5`, each the triangle of that corner and
the floor's `(0, 5, 0)` and `(5, 5, 0)`. The edge from `(0, 5, 0)` to
`(5, 5, 0)` had four faces along it, the two slivers and the floor's pieces
either side of where the cube's edge rests, and the union was refused at 115
of Q1506's 400 tilts, under both policies and both ways round (the first
Q1528 lane's sweep, with a probe since taken out).
`the_union_is_both_cubes_less_what_they_share_where_it_was_refused` pins five
of those tilts, `t = 0.002`, `0.003`, `0.008`, `0.01` and `0.015`: each union
now answers the two cubes less what they share, under both policies, both
ways round.

### 431.2 The two combs, measured

The rule also reaches the two combs of §365.8's last note (Q1633, Q1634): a
comb of three teeth leaning forward two, and a comb of two teeth leaning back
two, one high at `(3, 0, 1)`, from `how_often_a_certified_fuse_of_a_skew_pair_refuses`.
`the_union_leaves_out_a_notch_whose_three_corners_round_to_two_points`, in
`a_sliver_along_an_edge_of_a_face_it_lies_on.rs`, measures what it does
there, exactly, from the planes the model stores:

- The first comb's tooth wall is stored `a·x − 2a·y − a = 0`, which is
  `x − 2y = 1` exactly; its gap floor is `y = 1` exactly; and the second
  comb's wall is stored `−a·x − 2a·y + d = 0`, which is `x + 2y = 5 + δ` with
  `δ = (d − 5a)/a` strictly between `2^-52` and `2^-51`, about `3.7e-16`.
- So three corners lie near `(3, 1)` from `z = 1` to `2`: the first comb's
  corner A = `(3, 1)`, the crossing B of the second comb's wall with the floor,
  `(3 + δ, 1)`, and the meet C of the two walls, `(3 + δ/2, 1 + δ/4)`.
- The notch between them, the triangle ABC from `z = 1` to `2`, which neither
  comb fills, holds `δ²/8`, `1.7333369499485123e-32`.
- `Corners::of` writes each corner at the middle of its box. A's box is the
  point itself, so its middle is A. B's and C's boxes have the one middle
  W = `(3.0000000000000004, 1)`, one rounding (`2^-51`) past `x = 3`, so the
  two are written as one corner. Both of those boxes hold A, at `z = 1`.
- The first comb has corners `(3, 1, 0)` and `(3, 1, 2)` on that line, and the
  second `(3, 1, 1)` and `(3, 1, 2)`. The certified union's one solid has only
  `(W, 1, 0)` and `(W, 1, 1)` near it.

As written, with B and C one corner at W, the floor and the tooth wall each
have a strip from A to W in two places: from `z = 1` to `2`, the notch's two
walls, and from `z = 0` to `1`, where the plane of the second comb's wall cuts
them below that wall. Each pair of strips walks one ring of four edges both
ways. The notch's third wall, on `x + 2y = 5 + δ`, has two sides of no length,
walks out and back along one edge, and is left out by §365.7's rule. Before
this rule each strip pair had four faces along one of its edges and the union
was refused `ResultIsNotManifold { count: 4 }`. The second relay found, with
a probe since taken out, that the rule leaves out those two pairs in each
union, one pair (`z = 0` to `1`) in each difference, and none in the
intersections. With the call switched off, the notch test and the two combs
tests fail on that refusal (`3a88e295`'s lane: 11 run, 8 passed, 3 failed).

### 431.3 The combs' union is right, as §417.4's answers are

Decided: the certified union of the two combs, one solid of 26 faces holding
`22 + 7 − 3`, is right, the same case as §417.4's, and no row is filed
against it.

**What the written answer leaves out.** A volume of `δ²/8`, about `1.7e-32`,
which holds no `f64` point. And one input corner: A, at `z = 0`, `1` and `2`.
Every corner dropped lies inside the boxes of B and C, which are the points
written at W; the notch test asserts that at `z = 1`, and the second relay
measured the same boxes at `z = 0` and `2` with a probe since taken out. Q1442
already left out an input corner under a certified policy, the turned cube's
tip, and §417.4's second paragraph took that as right.

**What differs from §417.4.** There, every corner the answer left out rounded
to the same `f64` triple as the corner written in its place. Here A's box is
the point itself and its middle is A, not W: two different doubles, joined
because the boxes of B and C hold A. And from `z = 0` to `1`, below the second
comb, the exact union needs no short edge at all, yet the first comb's corner
`(3, 1, 0)` is written at `(W, 1, 0)`, one rounding off.

**What the exact answer says that the written one does not.** Worked out from
the planes and not run: near `(3, 1)` from `z = 1` to `2` the exact union is
`{y ≤ 1} ∪ {x − 2y ≤ 1} ∪ {x + 2y ≥ 5 + δ}`. Its outside there is the notch,
which opens below `z = 1` into the first comb's gap and above `z = 2` onto the
top, so the exact union has a channel through it that the written answer
closes. That is the analogue of the solid count §417.4 names for the prism.

**How to disagree.** A reviewer who holds that a certified answer may not
write an input corner whose box is one point at a different triple, or may
not close a channel no `f64` point lies in, should file it as `wrong/C`
against this section. The alternative to weigh is to write every corner whose
box holds one triple at that triple, here A; nobody has built or run it.

### 431.4 Why the first comb less the second is refused

The certified first comb less the second is refused
`Invalid::DynamicRangeTooWide { ratio }` with `ratio` exactly `11 · 2^51`,
which `two_combs_leaning_opposite_ways_meet_in_two_parts_a_rounding_apart`
pins. Exactly, the difference's side of the tooth from A to C is the top of
the tooth's wall, which runs on below `z = 1`. As written, the strip of that
wall from `z = 0` to `1` is left out with its strip of the floor, as in the
union, so from `z = 1` to `2` the side is a face of its own. The floor strip
beside it there is the top of the second comb's base and is not in the
difference, so no face walks the side's ring back, and it stays. Its edges at
`z = 1` and `2` are written `2^-51` long beside the first comb's `11` across.
That is a length over a length, so no scaling brings the two together (§3.5),
and `check_dynamic_range` refuses it by the limit's own name. A tolerant
policy writes those edges and answers one solid of 25 faces holding `22 − 3`.

So the union answers where the difference refuses because in the union the
side's strip has a partner walking it back and in the difference it does not.

### 431.5 What it moved

- `a_sliver_along_an_edge_of_a_face_it_lies_on.rs` (`b492681c`):
  `two_combs_leaning_opposite_ways_meet_in_two_parts_a_rounding_apart` and
  `two_combs_leaning_opposite_ways_the_other_way_round_meet_in_two_parts_a_rounding_apart`.
  Both unions, under each policy, were refused `ResultIsNotManifold { count: 4 }`
  and are one solid of 26 faces holding `26`. The certified first comb less
  the second was refused the same way and is refused `DynamicRangeTooWide`
  at `11 · 2^51`; the tolerant one answers 25 faces holding `19`. The
  intersections, two parts holding `3`, and the second comb less the first,
  four parts holding `4`, did not move.
- `a_cube_tipped_onto_one_edge_along_another_cubes_floor.rs` (`6c027e78`):
  `the_union_is_both_cubes_less_what_they_share_where_it_was_refused`, above.
- `fuses_that_must_agree.rs`, rule off then on (the second relay with the
  call switched off, and the branch;
  `mise exec -- just t -p rcad-topo --test merged --no-capture -E 'test(=fuses_that_must_agree::how_often_a_certified_fuse_of_a_skew_pair_refuses) | test(=fuses_that_must_agree::swapping_the_two_sides_of_a_skew_pair_changes_not_one_bit) | test(=fuses_that_must_agree::how_often_a_certified_fuse_of_a_turned_pair_refuses)'`):
  - `how_often_a_certified_fuse_of_a_skew_pair_refuses`: 2 refused and 7 not
    a solid of 450, then 3 and 5. The union with the first comb first now
    answers, and the first comb less the second is refused for its edge.
  - `swapping_the_two_sides_of_a_skew_pair_changes_not_one_bit`: 12 refused,
    6 not a solid and 131 of 140 comparisons reached, then 12, 2 and 133.
  - `how_often_a_certified_fuse_of_a_turned_pair_refuses`: 25 refused and 6
    not a solid, both times.

  The three tests pass either way. Three doc comments there still give the
  counts from before: `SKEW_REFUSALS_MUST_STAY_BELOW`'s "2 of 450, 0.44%",
  the Q1608/Q1609 paragraph of `how_often_a_certified_fuse_of_a_skew_pair_refuses`,
  which has the combs' union and first less second not a solid, and the last
  paragraph of `swapping_the_two_sides_of_a_skew_pair_changes_not_one_bit`.
  That file lay outside this lane's write set, so the fix is a row of its
  own.

### 431.6 What this leaves

Q1658 is done. It asked for the combs to be refused by the limit's name, or
for a reason why not. The unions now answer (431.3), and the one fuse still
refused, the certified first comb less the second, is refused
`DynamicRangeTooWide` (431.4). Its other ask, that §365.8 record the
intersection answering with the second comb first and no swap comparison
disagreeing, the Q1633 and Q1634 note already does.

The first Q1528 lane's sweep of 400 tilts left three rows, filed when it
stopped: Q1686 (`OuterWireWindsAgainstTheNormal` and `VolumeIsNotPositive`
where the cube's corner is stored a rounding below the floor), Q1687 (a
tolerant difference answering a second solid of `8e-32`), and Q1688 (the
certified `DynamicRangeTooWide` refusals, whose cause nobody has run).
