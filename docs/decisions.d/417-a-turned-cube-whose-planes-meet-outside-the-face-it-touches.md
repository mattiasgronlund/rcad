## 417. A turned cube whose planes meet outside the face it touches — 2026-10-08

*Queue row Q1353. Lane `lane-q1353-a-turned-cube-whose-planes-meet-outside-the-face`,
off `7d8ed30e`, in three relays (§416): `5c91b4ed` built the rule, `852579c8`
moved the pins inside its write set, and `b2a7cceb` pinned the turned cube.
Each count says which relay ran it and with what. `probe_q1353_split` is in
`a_turned_cube_with_its_corner_on_a_face.rs` at `852579c8` and was taken out
after; the certified count was a scratch copy of it, never committed.*

### 417.1 The shape

A block `[0, 10]³` and a second cube of side 10 turned so all three edges at
one corner lean to `+x`, that corner put on the block's face `x = 0`. Where the
turned cube's three planes at that corner meet a rounding **outside** the face
(`x = −1.64e−16` for the turn by 1.5 about `(1, 1, 0)` with the corner at
`(0, 3, 3)`), its three edges cross `x = 0` at three points proved apart, and
the arrangement cuts the triangle between them out of the block's face. That
triangle is the cube's tip cut off by the face: each of its corners is on two
of the cube's planes and strictly behind the third, so it is **inside** the
turned cube. §365.8 placed the same triangle where the planes meet inside the
face, where it is outside the cube.

Of Q1208's 400 random turns (`q1208s_random_turns` in that file), 199 meet
outside, where `side_of_a_plane_where_three_planes_meet` against `x = 0` gives
`Neg`, and 201 inside (`Pos`). On the base the union with the block first under
`Policy::tolerant(100.0)` answered 51 of the 199 and refused 148: 74
`Undecidable` *a point inside a piece of a face* on the triangle, 58
`HoleWindsWithTheOuterWire` and 16 `OuterWireWindsAgainstTheNormal` (relay 0,
`probe_q1353_split` on the base with the probe added). The row's 74 written
broken are those 58 and 16.

### 417.2 The rule: a flat piece inside a convex flat solid

`by_a_plane` in `fuse/select.rs` already placed a flat piece with straight
sides **outside** a solid with only flat faces by one face plane `σ`, turned to
face out, with every corner of the solid on or behind it, every corner of the
piece on or in front of it and one strictly in front (§365.8). It now also
places the piece **inside** that solid when **every** face plane has every
corner of the solid on or behind it, every corner of the piece on or behind it
and, for each plane, one corner of the piece strictly behind.
`placed_by_the_planes_of` asks both of each solid; inside one solid of the
group is inside, as `by_a_corner` reads it. Each corner of the piece is asked
exactly, as the meet of its three planes, as before.

**Why it holds.** Every corner of the solid behind every face plane makes the
solid convex. Take `p` inside the piece, open in the piece's plane, and a face
plane `σ`. The piece lies in the hull of its corners, so all of it is on or
behind `σ`. Were `p` on `σ`, the signed distance to `σ`, which is linear on
the piece's plane, would be at most zero on a disk round `p` and zero at its
middle, so zero on all of that plane, and every corner of the piece on `σ`,
which one corner strictly behind rules out. So `p` is strictly behind every
face plane. So is a point `k` inside the solid, since a ball round it is in the
solid. Every point between `k` and `p` is then strictly behind every face
plane, so on no face, and the segment never crosses the solid's skin: `p` is
inside. The proof is also `by_a_plane`'s doc comment. A solid with a void has a
face plane with the outer corners in front of it, so it is left out.

The rule is exact, so it runs under both policies, and it moved pins outside
the turned cube's file (417.3).

### 417.3 What it moved

**The turned cube's 199, tolerant.** The union with the block first now
answers 62, and in each the union and the intersection hold 2000 between them
within `1e−9`. None is refused `Undecidable`. The 137 left: 65
`HoleWindsWithTheOuterWire`, 16 `OuterWireWindsAgainstTheNormal`, 34
`Invalid::VolumeIsNotPositive { volume: 0.0 }`, 16 `ResultIsNotManifold {
count: 4 }` and 6 `ResultIsNotManifold { count: 0 }` (relay 0 on `5c91b4ed`,
and this relay again on `852579c8`, the same split; `mise exec -- just t -p
rcad-topo --test a_turned_cube_with_its_corner_on_a_face --no-capture
probe_q1353_split`). So the 74 `Undecidable` went to 11 answers, 7 holes
winding with the outline, 34 shells enclosing nothing and 22 edges with four
faces or none. `random_turns_whose_planes_meet_outside_the_block_answer_or_are_refused_by_name_tolerant`
pins the first 60, of which 31 meet outside: 15 answer, turns 47 and 48 among
them, which relay 0 found `Undecidable` on the base, and 16 are pinned by name.

**The turned cube's 199, certified** (this relay, a scratch copy of the probe
on `852579c8`). The union answers 10 either way round, the intersection 8, the
block less the cube 8 and the cube less the block 2. Every one validates and
measures within `1e−6` of what the same fuse gives with the turned cube moved
`2⁻³⁰` further in, where nothing is a rounding from anything else (the union
`2000` less that intersection, a difference `1000` less it). The rest are
refused: the union 98 `DynamicRangeTooWide`, 35 holes winding with the
outline, 34 shells enclosing nothing, 16 and 6 edges with four faces or none;
the intersection 191 `DynamicRangeTooWide`. How the tip is written in the ones
that answer was not traced; 417.4 says why they are taken as right.

**The twelve fuses at `(0, 3, 3)`** (`ON_THE_FACE`) give what they gave on the
base: each pin there passed unchanged on `852579c8` and `b2a7cceb`. None of the
six tolerant ones answers, so none moved to volumes: that is 417.5's half.

**`two_turned_cubes_a_rounding_apart.rs`**: the pair overlapping by `2^-52`.
The last stop, the strip of the first cube's top from `y = 0.5` to `2`, inside
the second cube, is now placed, and all five certified fuses are refused
`DynamicRangeTooWide` instead of `Undecidable`: `a ∪ b`, `b ∪ a` and `b − a`
at `7.453e16`, `a ∩ b` at `3.924e16`, `a − b` at `4.166e16`. The limit
§365.8 named, now reached by name (relay 1, `mise exec -- just t -p rcad-topo
--no-fail-fast` with a probe since taken out).

**`a_prism_lying_on_one_edge.rs`** at `x = −1.5`, the prism's edge `3.1e−17`
into the block's top, certified. All six fuses were `Undecidable`. The union
either way round now answers two solids, 240 and 24, the block less the prism
the block and the prism less the block the prism, each validating; the
intersection is refused `ResultIsNotManifold { count: 0 }` both ways round
(relay 1). 417.4 is the judgment.

**Outside the lane's write set, two fail on the branch** and are not repinned
(relay 1, `mise exec -- just t -p rcad-topo --no-fail-fast`):

- `a_piece_of_a_face_bounded_by_two_straight_sides.rs`,
  `two_walls_a_hair_apart_are_caught_before_the_walk_gets_a_two_sided_piece`:
  the union under `Policy::tolerant(1e6)` and under `Policy::certified()` now
  answers one solid that validates, where both were `Undecidable`; the
  tolerant one measures `2450.0`, which is `3000 − 125 − 200 − 300 + 75`, the
  right volume.
- `fusing_solids.rs`,
  `what_two_turned_cubes_a_rounding_deep_in_each_other_share_is_too_thin_to_hold_and_is_refused`:
  now `Invalid(DynamicRangeTooWide { ratio: 3.924096673605416e16 })`, where
  it was `Undecidable`. The limit, now by name.

`fuses_that_must_agree.rs` and `three_cut_lines_through_one_corner_of_a_skew_pair.rs`
did not move.

### 417.4 An answer written as §365.7 writes a strip

The prism at `x = −1.5` and the 10 certified turned-cube unions answer under
`Policy::certified()` where the exact answer needs a side shorter than the gap
between two doubles. They are taken as right, superseding §365.8's "still
refused, a limit" for the prism. §365.7 settled that a side whose ends round to
one `f64` triple is left out of the answer (`sides_of` in `fuse/assemble.rs`),
and that a wire that then walks out and back along one edge encloses nothing
and is left out (`encloses_something`); both run under every policy. With
the inside rule, the prism's sliver and its strips are placed, and what is
written is what those two rules make of them: for the prism, the sliver cut out
of the block's top closes up and the two solids are written touching along the
prism's edge, as they are at `x = 0`. The exact union is one solid, the two
joined through a sliver `3.1e−17` deep that holds no double; the solid count
is the one thing the written answer says that the exact one does not. The
sliver's corners round to `(−1.5, 0, 2)` and `(−1.5, 0, 8)` (relay 1), and
rounding a constructed corner is what writing any answer does. A reviewer who holds that a certified answer may not change how
many solids there are should file it as `wrong/C`, against §365.7's rule and
not against this one.

### 417.5 The other half: the three crossings as one, not built

At `(0, 3, 3)`, and in the 137 of the 199 still refused under the tolerant
policy, the answer meant is the block with its `x = 0` face whole, with the
tip a rounding tall left out. It exists, so under `Policy::tolerant(100.0)` the
refusal is a gap. Under `Policy::certified()` it is a limit:
`a_turned_cube_with_its_corner_on_a_face.rs`'s doc shows every rounding of the
tip's corners that winds right has an edge of at most `4.7e−16` beside a
corner at 10, a range of `2.1e16`, against `1e11`.

Families weighed, none built:

- **(a) Fold the apex and the three crossings into one corner** under a
  tolerant policy, with an `Approximation` of how far each moved. The
  crossings are proved apart exactly, so no undecided sign is left for the
  policy to read as zero; `Strictness::TolerantFallback` promises only that,
  and `Arrangement::fold_the_nodes` folds only where the policy resolved a sign
  (§284.7, §284.8). Folding points proved apart, each within
  `Policy::max_wobble` of the others, is a new power for the policy, and
  takes a decision of its own written here before it is built. The fold must
  name the one corner the same way in every face that has one of the four
  (the block's face and the cube's three faces at the corner), or `Corners::of`
  joins some and not others.
- **(b) Write the exact answer with the tip a rounding proud.**
  `check_dynamic_range` is `Severity::Warn` under a tolerant policy, so a tip
  whose rounded corners wind right would be let through. It answers with a
  spike `1e−16` tall that the designer did not mean, and the 78 of 256
  roundings that wind right are a search, not a construction.
- **(c) Rely on the roundings meeting.** The 62 tolerant and 10 certified
  answers suggest that where the four tip points round to one `f64` triple,
  §365.7's rules close the face; where only some do, a shell enclosing nothing
  (34) or an edge with four faces (16) is left. **Unverified**: no print in
  `Corners::of` has shown which of the four were joined in any turn. A print
  there for turn 8 (`VolumeIsNotPositive { volume: 0.0 }`) is the next step,
  and if it holds, (a) reduces to joining the four under the policy where
  rounding joined only some.

(a) is the family the row names and the only one that reaches the answer meant
without a search; it is left to a row of its own because it changes
`Policy`'s contract, and that wants its own argument and its own lane.

### 417.6 What this leaves

- The fold of 417.5 (a), with the print of (c) first: a row.
- The two tests of 417.3 outside the lane's write set, failing on the branch:
  a lane whose write set holds them repins both before this branch can land.
- `ResultIsNotManifold { count: 0 }`, the prism's intersection and 6 of the
  turned-cube unions, names an edge with no face along it, which says nothing
  of the sliver it was refused for. Not traced.
