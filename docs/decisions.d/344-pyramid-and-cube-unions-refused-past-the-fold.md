## 344. The pyramid unions refused past the fold are two gaps: the fold's touch test and bake's corner picker — 2026-09-28

*Queue row Q273, from §310.5's second bullet and §301.7. Lane
`lane-q273-pyramid-unions-refused-past-the-fold`, off `893b2b94`. No kernel
code changed: every probe ran in a scratch copy of the tree. §310 left
unexplained two sets of refused unions from
`a_corner_with_four_planes_round_it.rs`'s table. The first set is the 5 of 108
refused at a fold of 1/4. The second is the 25 still refused with the fold's
room x1000. The module comment says these "had every apex meet folded, so they
are refused somewhere else". **That is wrong for 15 of the 25.** Those 15 are
refused at the apex, because meets that lie inside the room are never folded:
their boxes do not touch the apex's box, and the fold asks for a touch before
it looks at the room (344.2). All 5 at a fold of 1/4 are among them. The other
10 are refused in `bake`, at the base corner where the two folding faces meet
the base. That corner's only triple solves to a box 2.8 to 47 rooms past the
stored corner, and the picker allows 1 (344.3). **Both are gaps** (§16.10).
The answer is two solids that do not touch. With the touch test dropped and
both rooms x1000, all 648 unions give that answer: the right volume and an
empty log.*

  344.1   The probe, and the table re-measured
  344.2   Fifteen refused at the apex: a meet inside the room that does not touch the corner
  344.3   Ten refused in bake: the base corner where the fold meets the base
  344.4   What this corrects
  344.5   The proposed rows

### 344.1 The probe, and the table re-measured

The probe is a copy of the tree at `893b2b94`, made with `git archive`, in the
lane's scratchpad. It holds one test, `q273_probe.rs`, which builds every
pyramid in the module's table: 6 folds, heights 1/4, 1 and 4, both spellings
of the side planes, and the 18 turns. That is 108 unions per fold. The test
unites each pyramid with the cube under `Policy::certified()`. For each answer
it checks the same things the module's first test checks: two solids, an
empty log, and a volume within `1e-9`. For each refusal it writes out the
error and every record, with the record's input boxes. Environment variables
read inside the copy's source switch on these changes, one at a time:

- `Q273_SCALE` multiplies the room in `a_corner_is_this_place` (§301.6's
  scratch edit).
- `Q273_NO_TOUCH` skips the fold's `boxes_overlap(node.point, place)` test
  (344.2).
- `Q273_BAKE_SCALE` multiplies the room in `the_place_the_model_points_at`
  (344.3).
- `Q273_TRACE` prints each face's crossings. It prints each meet
  `place_the_meets` makes, with every corner that holds both of its crossings,
  whether the two boxes touch, and how far the meet sticks out in rooms. It
  prints any undecided order along a crossing, with both nodes. It prints any
  corner `derive_a_corner` gives up on, with each triple's box: its width and
  how far it misses the stored corner past touching, both in rooms.

The binary was run straight, for example `Q273_SCALE=1000
Q273_OUT=<scratchpad>/scale1000.log <scratchpad>/probe/target/debug/deps/q273_probe-38b615e8e376594d`,
after `mise exec -- cargo test --manifest-path <scratchpad>/probe/Cargo.toml -p
rcad-topo --test q273_probe --no-run`. `run_all.sh` in the scratchpad runs
each configuration. **This re-measures the table's "Answered" columns,** and
does not cite them:

| Fold | Room x1 | x0.5 | x1000 | x1, no touch | x1000, no touch | x1000, no touch, picker x1000 |
|---|---|---|---|---|---|---|
| 1 | 108 | 108 | 108 | 108 | 108 | 108 |
| 1/4 | 103 | 103 | 103 | **108** | 108 | 108 |
| 1/16 | 99 | 31 | 103 | 102 | 108 | 108 |
| 1/64 | 0 | 0 | 107 | 0 | 108 | 108 |
| 1/256 | 0 | 0 | 105 | 0 | 106 | 108 |
| 1/4096 | 0 | 0 | 97 | 0 | 100 | 108 |

The first three columns match the module's table. No run gave a wrong answer.
Every union that answered had two solids, the right volume and an empty log.

### 344.2 Fifteen refused at the apex: a meet inside the room that does not touch the corner

**Where it refuses.** `a_corner_is_this_place` (`fuse/arrange.rs:1343` on
`893b2b94`). It begins by returning `false` unless the corner holds both
crossings **and** `boxes_overlap(node.point, place)` (line 1354). Only after
that does it ask whether the meet's box lies inside the corner's box grown by
the room. A meet that fails the touch test stays a node beside the apex. Then
one of two things happens:

- **9 cases:** two such meets overlap each other on one crossing, and
  `order_along` (line 1752) cannot order them. The error is
  `Error::Undecidable` with one record: `"order along a crossing"`, level
  `Interval`, both inputs boxes at the apex. The enclosure is `8.2e-16` to
  `5.4e-14` down to a fold of 1/16, and `2.9e-12` at 1/4096.
- **6 cases:** the meets can be ordered, but the tiny cells they cut off
  round the apex give `count_the_windings`/`spread_the_winding` (lines
  3058–3113) a winding outside 0 and 1. The error is
  `Error::CannotTellTheCellsOfAFaceApart` on the first side face. The trace
  did not say which of that function's three returns fired.

**What it saw.** In every one of the 15, the face that refuses is the first
side face, `(apex, b1, b2)`, one of the two faces that fold. It is always
written `Through::ABaseCorner`. The trace lists four crossings on that face,
each marked as a cut: all three other side planes and the base. So the apex
has three meets on it, one per pair of the side-plane crossings:

- its own triple: the face, and its neighbours either side;
- the face, its folding neighbour, and the side face opposite;
- the face, its other neighbour, and the side face opposite.

`bake` solved the apex from the best-conditioned of the four triples (§310.1).
The trace shows the meet boxes missing the apex's box, so the triples meet at
points that are really different. In the first case, at a fold of 1/4, the
apex's box is 0.015 rooms wide and the meet boxes 0.02 to 0.2. Each meet box
misses the apex's box by 0.001 to 0.03 rooms, on one axis.

How far each meet that failed the touch test sticks out past the apex's box,
in rooms of the apex (64 ε × max(size, 1)):

| Fold | Refused (x1000) | Furthest out, per case, in rooms |
|---|---|---|
| 1/4 | 5 | 0.21, 0.26, 0.28, 0.32, 0.16 |
| 1/16 | 5 | 0.60, 0.94, 0.87, 1.09, 0.67 |
| 1/64 | 1 | 2.54 |
| 1/256 | 1 | 5.9 |
| 1/4096 | 3 | 197, 104, 204 |

The 1/4096 row counts only the cases that `bake` does not stop first. **At a
fold of 1/4, every meet lies within a third of the room** and still is not
folded. So the room is never consulted, and x1000 of it changes nothing.

**Why §310's table did not see these.** §301.6's probe replaced the *last*
expression of the fold's closure. The closure returns early at the touch
test, so the probe logged only meets that touched their corner. Its column
"Furthest an apex meet sticks out" is the furthest among touching meets, and
the reading "every apex meet folded" came from that column.

**Why the answer exists.** The answer is the two solids as they are.
Dropping the touch test in the scratch copy and changing nothing else gives
that answer for all 5 unions at a fold of 1/4 at the room as it is. At x1000
it gives it for all 15, with the right volume and an empty log (344.1's
table). The touch test is not part of §253's argument for the fold. That
argument is that a meet whose every point lies inside the corner's box grown
by the room is the corner written down another way. Whether the two boxes
share a point does not enter it. A meet a third of a room out passes the room
test either way. **So this is a gap, in the order of the tests** (§16.10): a
representable answer is refused before the test that would take it is asked.

**Why it may not simply be dropped.** This lane did not check that dropping
the touch test is safe. That belongs to the row that closes the gap (344.5).
What the row has to show:

- **§253.3's corpus fold, still refused for the right reason.** A meet box
  reaching `1.4e-10` from its corner, 2 012 times the room, is refused by the
  room test and not by the touch test. The row should confirm that from its
  own run, not from this sentence.
- **§244.3/§248's two places of one meet.** They are kept apart by the
  second test, which counts places inside the grown box, not by the touch
  test.
- The unit tests round `a_place_as_far_off_a_corner_as_its_room_is_folded_into_it_and_one_a_step_further_is_not`
  (§301.5).

### 344.3 Ten refused in bake: the base corner where the fold meets the base

**Where it refuses.** `derive_a_corner` (`fuse/bake.rs:654` on `893b2b94`)
gives up at line 686 with `Error::Undecidable`. Its one record is `"where
three surfaces meet"`, level `Interval`, enclosure `inf`, measures `Volume`,
and its input is `middle_of_the_places` of the corner's surfaces, not the
corner itself. It gives up because `the_place_the_model_points_at` (line 671,
defined at 768) turns down the only place of the only triple. Its test at
line 776, `boxes_overlap(place, stored, room)`, fails. This is **the site
Q426 already names** for §301.3's thin wedge.

**What it saw.** In all 10, the corner is `b2`, `(0, -1 - fold, 0)` turned.
That is where the two folding side faces meet the base. It has three planes
round it: the base and the two folding side faces. Those two cross at an angle
of about `fold`. The pyramid is built already turned, so its placement is the
identity. The stored corner is an exact `f64` triple, and the room is
`1.42e-14`. The triple's box, in rooms:

| Fold | Height, spelling, turn, axis | Box width x, y, z | Past touching, furthest axis |
|---|---|---|---|
| 1/256 | 4, apex, 0.3, (0.3, 1, -0.7) | 2.4, 1.4, 1.9 | 2.81 |
| 1/256 | 4, apex, 0.7, (3, -1, 2) | 4.2, 2.6, 4.6 | 2.92 |
| 1/4096 | 1/4, apex, 0.1, (1, 2, 3) | 87, 93, 8.8 | 10.3 |
| 1/4096 | 1/4, base corner, 1.1, (0.3, 1, -0.7) | 524, 721, 167 | 21.1 |
| 1/4096 | 1, apex, 0.1, (1, 2, 3) | 69, 33, 9.5 | 7.2 |
| 1/4096 | 4, apex, 0.1, (0.3, 1, -0.7) | 35, 11, 15 | 16.1 |
| 1/4096 | 4, apex, 0.3, (3, -1, 2) | 35, 16, 17 | 2.8 |
| 1/4096 | 4, apex, 1.9, (1, 2, 3) | 46, 103, 47 | 8.9 |
| 1/4096 | 4, apex, 2.5, (1, 2, 3) | 91, 110, 52 | 46.6 |
| 1/4096 | 4, base corner, 0.3, (3, -1, 2) | 35, 16, 15 | 3.6 |

The picker allows 1. The base plane is written through `b1`, `b4` and `b3`,
not through `b2`. Each folding side plane misses `b2` by its own rounding.
Two planes an angle `fold` apart carry that miss about `1/fold` times further
along the line where they cross. So the exact point where the three stored
planes meet lies several rooms from the stored corner at 1/256, and tens of
rooms from it at 1/4096. At x1000 these 10 are exactly the ones refused with
the touch test dropped.

**Why the answer exists.** `SolidBuilder::build` accepted each pyramid under
`Policy::certified()`, and §75.6's rule is that a solid `Model::validate`
accepts is one the fuse has to be able to bake. With the picker's room x1000
as well, all 10 answer with two solids, the right volume and an empty log
(344.1's last column). So the meet of the three planes is a corner the fuse
can use, and the refusal is a gap (§16.10).

**What this adds to Q426.** Q426 names two fixes. The first is to pick in the
stored frame and take the moved meet as the corner. That fits the wedge,
whose planes meet exactly at its stored corner before the turn. **It cannot
reach these 10.** Their placement is the identity, so the stored frame is the
world frame, and in it the three stored planes already miss the stored corner
by 2.8 to 47 rooms. The second fix is to size the picker's room by how badly
the triple is conditioned. That one would reach them, since the miss grows as
`1/fold`, the way the wedge's grows as 1/slope. So these pyramids are a
second shape for Q426's test, and they rule its first fix out as a whole
answer.

### 344.4 What this corrects

- **The module comment of `a_corner_with_four_planes_round_it.rs`** says
  "The few refused at ×1000, and at a fold of `1/4`, had every apex meet
  folded, so they are refused somewhere else". 15 of those 25, all 5 at 1/4
  among them, are refused at the apex, with meets left unfolded. The other 10
  are refused in `bake`, at `b2`, not at the apex. This lane leaves the
  comment alone, because a comment-only edit has no test that fails without
  it. The rows in 344.5 own that file and should rewrite the sentence when
  they turn the table round.
- **§310.2's "Furthest an apex meet sticks out" column** is the furthest among
  meets that touched the apex, not among all of them (344.2).
- **Q272's gap** (§310.3, the room at folds of 1/64 and below) is a third
  site, and it stays Q272's. With the touch test dropped and the room as it
  is, 1/64, 1/256 and 1/4096 still answer 0 of 108. Closing Q272 alone
  leaves at least 15 + 10 refusals across the table, going by the x1000
  column.

### 344.5 The proposed rows

This lane adds no row to `docs/queue.md`. That file is outside its write set.
One row per gap site, for the dispatcher to add:

- **The fold's touch test.** Tier 1, opus. Write set
  `crates/rcad-topo/src/fuse/arrange.rs`,
  `crates/rcad-topo/tests/a_corner_with_four_planes_round_it.rs`. Text:
  "At a fold of 1/4, 5 of the 108 pyramid-and-cube unions in
  a_corner_with_four_planes_round_it.rs are refused under
  Policy::certified(), all with the pyramid's planes written through a base
  corner. a_corner_is_this_place (fuse/arrange.rs:1354 on 893b2b94) tests
  that a meet's box touches its corner's box before it tests the room. The
  apex has four planes round it, so three meets on the first side face come
  from triples bake did not solve from. They lie 0.16 to 0.32 rooms out but
  miss the apex's box by a few hundredths of a room, so they are never
  folded. Then
  order_along cannot order two of them, or the windings go outside 0 and 1.
  Dropping the touch test answers all 5 at the room as it is, and 15 of the
  25 still refused at x1000, with the right volume and an empty log. Add a
  test of the 5 at a fold of 1/4 that fails today. Then fold on the room
  alone, or show why the touch test is needed. Keep §253.3's corpus meet and
  §248's two places refused. Rewrite the module comment's 'had every apex
  meet folded'. §344.2 §253 §16.10"
- **Bake's corner picker, at the folding base corner.** Fold it into Q426 if
  that row has not started. If it has, add as its own row after Q426. Tier 1,
  opus. Write set `crates/rcad-topo/src/fuse/bake.rs`,
  `crates/rcad-topo/tests/a_corner_with_four_planes_round_it.rs`. Text:
  "10 pyramid-and-cube unions in a_corner_with_four_planes_round_it.rs, at
  folds of 1/256 and 1/4096, are refused under Policy::certified() in
  derive_a_corner (fuse/bake.rs:686 on 893b2b94).
  the_place_the_model_points_at turns down the only place of the only triple
  at b2, where the two folding faces meet the base. That place is 2.8 to 47
  rooms past the stored corner, and the picker allows 1. The pyramid's
  placement is the identity, so picking in the stored frame (Q426's first
  fix) cannot reach it, and the miss grows as 1/fold. With the picker's room
  x1000, all 10 answer with the right volume and an empty log. Add these 10
  to the file's tests, failing today. Size the picker's room by the triple's
  conditioning, and keep §136.3's grazing corner refused. §344.3 §301.3
  §16.10"
