## 417. A turned cube whose planes meet outside the face it touches — 2026-10-08

*Queue row Q1353. Lane `lane-q1353-a-turned-cube-whose-planes-meet-outside-the-face`,
off `7d8ed30e`, in three relays (§416): `5c91b4ed` built the rule, `852579c8`
moved the pins inside its write set, and `b2a7cceb` pinned the turned cube.
Each count says which relay ran it and with what. `probe_q1353_split` is in
`a_turned_cube_with_its_corner_on_a_face.rs` at `852579c8` and was taken out
after; the certified count was a scratch copy of it, never committed.
Queue row Q1442 (lane `lane-q1442-edges-only-dropped-wires-walked`, off
`7fddc415`, in two relays) added to 417.3, 417.4 and 417.6 and corrected
417.5 (c); its counts are `q1442_the_count`, a probe likewise taken out.
Queue row Q1506 (its second lane, `lane-q1506-the-turn-13-pin-repinned`, off
`602ee000`) added to 417.3 and 417.5; its counts are `probe_q1506_split` and
`probe_q1506_certified`, probes likewise taken out.*

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
Since queue row Q1506, 18 answer both rules, 1 answers its union and not its
intersection, and 12 are pinned by their union's refusal. Turns 8, 21 and 49,
which were shells enclosing nothing, answer once Q1506's rule leaves out the
flat ring another plane walks back. Turn 13, which was a hole winding with its
outline, answers its union, `1383.97`, and its intersection is still refused
`OuterWireWindsAgainstTheNormal`; with no intersection to hold it to, the pin
holds its union to `1000` more than the turned cube less the block, which
answers `383.97`, within the rounding the other checks allow. The 199 are
counted again below.

**Since queue row Q1442 the 6 `count: 0` answer too**, 68 in all, and the 131
left are the 65, 16, 34 and 16 above (Q1442's lane on `4a7dec01`, `mise exec
-- just t -p rcad-topo --test a_turned_cube_with_its_corner_on_a_face
--no-capture q1442_the_count`, a probe pasted into that file and taken out
after). Each of the 6 was an edge only left-out wires walked: the three
crossings round to one triple and the tip, where the three planes meet, to
another a rounding off the face, each of the cube's three faces at the corner
keeps a wire that walks out from that triple to the tip and back, and every
such wire is left out (417.4). The edge between the two had been made and had
no face along it. `Assembly::leave_out_what_no_wire_walks` in
`fuse/assemble.rs` now leaves out every edge no kept wire walks, and every
corner no edge left ends at, so the tip goes and the answer is written as if
it were on the face. `six_random_turns_whose_crossings_round_to_one_corner_answer_every_fuse`
pins turns 190, 228, 273, 283, 294 and 329, every rule both ways round under
both policies, against volumes.

**The turned cube's 199, certified** (this relay, a scratch copy of the probe
on `852579c8`). The union answers 10 either way round, the intersection 8, the
block less the cube 8 and the cube less the block 2. Every one validates and
measures within `1e−6` of what the same fuse gives with the turned cube moved
`2⁻³⁰` further in, where nothing is a rounding from anything else (the union
`2000` less that intersection, a difference `1000` less it). The rest are
refused: the union 98 `DynamicRangeTooWide`, 35 holes winding with the
outline, 34 shells enclosing nothing, 16 and 6 edges with four faces or none;
the intersection 191 `DynamicRangeTooWide`. How the tip is written in the ones
that answer was not traced; 417.4 says why they are taken as right. Since
Q1442 the 6 edges with no face are gone: the union answers 16 either way round,
the 6 of 417.3's tolerant paragraph among them, and is refused 98
`DynamicRangeTooWide`, 35 holes winding with the outline, 34 shells enclosing
nothing and 16 edges with four faces; the cube less the block answers 16 (14
one solid, one two and one three) and is refused 65 `DynamicRangeTooWide` and
118 `VolumeIsNotPositive` (Q1442's lane on `4a7dec01`, the same probe). None
of the fuses that probe asked, these three and the tolerant union with the
block first, is refused `count: 0` for any of the 199.

**Since queue row Q1506 the 34 shells enclosing nothing are gone**, under
both policies (Q1506's second lane, on `5bb2301f`, `mise exec -- just t
probe_q1506_split --no-capture` and `probe_q1506_certified`, probes pasted
into the turned cube's file and taken out after). Tolerant, the union with the
block first answers 104 of the 199 and is refused 63
`HoleWindsWithTheOuterWire`, 16 `OuterWireWindsAgainstTheNormal` and 16
`ResultIsNotManifold { count: 4 }`. The intersection of 102 of the 104
answers, and each pair validates and holds 2000 between them; the
intersection of turns 13 and 314 is refused `OuterWireWindsAgainstTheNormal`,
and their union validates and is `1000` more than the turned cube less the
block. The worst of those 104 checks is `1.14e−12`. Certified, the union
answers 53 either way round and is refused 95 `DynamicRangeTooWide`, 35
`HoleWindsWithTheOuterWire` and 16 `count: 4`; with the block first each of
the 53 validates and measures exactly what the tolerant union does. The cube
less the block answers 74 (66 one solid, 6 two and 2 three) and is refused 65
`DynamicRangeTooWide` and 60 `VolumeIsNotPositive`; the intersection still
answers 8 and is refused 191 `DynamicRangeTooWide`. The cube less the block's
74 were counted, not measured.

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
intersection was refused `ResultIsNotManifold { count: 0 }` both ways round
(relay 1), and since Q1442 answers no solid both ways round, as at `x = 0`:
every wire of the sliver walks out and back along the prism's edge and is left
out, and the edge and its two corners go with them
(`the_prism_with_its_edge_a_rounding_into_the_blocks_top_answers_as_if_on_it`).
417.4 is the judgment.

**Outside the first lane's write set, two failed on its branch** (relay 1,
`mise exec -- just t -p rcad-topo --no-fail-fast`); a second lane repinned
both to these answers in 8228a212:

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

Queue row Q1442 adds a third rule that follows from the two: an edge only
left-out wires walked has no face along it and is left out, and so is every
corner no edge left ends at (`Assembly::leave_out_what_no_wire_walks` in
`fuse/assemble.rs`). It writes the prism's intersection as no solid, and 6 more
certified turned-cube unions with the tip left out, as if it were on the face.
The same judgment covers them. It changes no answer: a fuse that answered had
no edge with no face, and a corner with no edge was dropped by the joining or
refused as an orphan.

### 417.5 The other half: the three crossings as one, not built

At `(0, 3, 3)`, and in the 95 of the 199 still refused under the tolerant
policy (137 before Q1442, 131 before Q1506), the answer meant is the block with its `x = 0` face whole, with the
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
- **(c) Rely on the roundings meeting.** This first read the 62 tolerant and
  10 certified answers as saying that where the four tip points round to one
  `f64` triple, §365.7's rules close the face, and where only some do, a shell
  enclosing nothing (34) or an edge with four faces (16) is left. **The count
  says otherwise.** Queue rows Q1439 and Q1442 rounded each of the four points
  to the middle of the box `Plane::meet_of_three` gives for its three planes
  and sorted the 199 by how many triples the three crossings make and whether
  the tip is on one, against the tolerant union with the block first (Q1442's
  lane, `q1442_the_count`, on `7fddc415` and again on `4a7dec01`):

  | crossings     | tip    | turns | union                                     |
  |---------------|--------|------:|-------------------------------------------|
  | one triple    | on it  |     2 | answers                                   |
  | one triple    | apart  |     6 | `count: 0`, answers since Q1442           |
  | two triples   | on one |     8 | answers                                   |
  | two triples   | apart  |    34 | `VolumeIsNotPositive { volume: 0.0 }`     |
  | two triples   | apart  |    16 | `ResultIsNotManifold { count: 4 }`        |
  | three triples | on one |     1 | answers                                   |
  | three triples | on one |     7 | `HoleWindsWithTheOuterWire`               |
  | three triples | apart  |    51 | answers                                   |
  | three triples | apart  |    58 | `HoleWindsWithTheOuterWire`               |
  | three triples | apart  |    16 | `OuterWireWindsAgainstTheNormal`          |

  So of the 62 that answered, only 2 have all four points on one triple and 51
  have all four apart, and every one of the 34 shells enclosing nothing has
  the crossings on two triples with the tip apart. The same rounding answers
  in some turns and is refused in others, so it is not what decides, and the
  reduction of (a) this hoped for is not shown. These are the probe's
  roundings, not a print in `Corners::of`; a print there for turn 8
  (`VolumeIsNotPositive { volume: 0.0 }`) is still what would show which
  points it joined.

(a) is the family the row names and the only one that reaches the answer meant
without a search; it is left to a row of its own because it changes
`Policy`'s contract, and that wants its own argument and its own lane.

### 417.6 What this leaves

- The fold of 417.5 (a), with the print of (c) first: a row.
- The two tests of 417.3 outside the first lane's write set: repinned by a
  second lane in 8228a212, before this branch landed. Nothing left.
- `ResultIsNotManifold { count: 0 }`, the prism's intersection and 6 of the
  turned-cube unions: done by queue row Q1442. It was an edge only left-out
  wires walked, and it is now left out with the corners no edge left ends at
  (417.3, 417.4).
