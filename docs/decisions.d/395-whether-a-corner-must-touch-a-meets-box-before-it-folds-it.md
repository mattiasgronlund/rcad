## 395. Whether a corner must touch a meet's box before it folds it — 2026-10-03

*Queue row Q435, from §344.2. Lane
`lane-q435-a-pyramid-apex-meet-folded-by-the-room`, off `25860052`.
`a_corner_is_this_place` asked whether a meet's box touches the corner's own
box before it looked at anything else. **It does not need to, and no longer
asks.** Neither of the two arguments for the fold reads the touch. §253's is
that every point of the meet's box lies within the room of the corner, and
§356's is that three planes meet once. Across the whole workspace the touch
decided 114 fold calls, all inside the room, and dropping it changes no
answer there. In `a_corner_with_four_planes_round_it.rs`'s table it decided
11 refused unions, 5 of them at a fold of 1/4. All 11 now answer with the two
solids, their volumes and an empty log. §248's two places stay apart by the
count of places in the room, which is what kept the second one apart before
as well. §253.3's corpus meet is still refused, by the room. The touch had
one more effect: on a meet of three of the corner's own planes it was the
only bound on distance. The count now takes that job, with the room
added. That leaves 5 unions at folds of 1/256 and 1/4096 refused, a gap left
for its own row (395.4).*

  395.1   The probe
  395.2   The pyramid table, re-measured
  395.3   What the touch test guarded, and what guards it now
  395.4   What the room still leaves: apex meets that miss it
  395.5   The tests, and what fails without the change
  395.6   What this leaves

### 395.1 The probe

A scratch edit to `a_corner_is_this_place`, taken back out before the change
was written, read three environment variables:

- `Q435_NO_TOUCH` skips the touch test, the rule this row commits;
- `Q435_RULE_B` folds on `(inside the room and one place reaching it) or the
  three planes bake solved the corner among`, with no bound on distance for
  the second (395.4);
- `Q435_LOG=<file>` appends a line for every call whose corner is on both
  crossings and where the rules disagree, or the touch holds and the fold
  does not. Each line has the touch, whether the box is inside the room,
  §356's rule, how many places reach the grown box, how far the meet's box
  sticks out past the corner's box and how far it misses it, both in rooms,
  both boxes, and the test's thread name.

A scratch `#[ignore]`d test in `a_corner_with_four_planes_round_it.rs`, also
taken out, fused the module table's 108 unions for each of its six folds and
printed every refusal with its parameters. It also printed every answer
without two solids, an empty log and a volume within `1e-9`, and there were
none. The runs were `mise exec -- just t -p rcad-topo --test
a_corner_with_four_planes_round_it --run-ignored all --no-capture
q435_scratch_table`, with the variables set, and `mise exec -- just t
--workspace --no-fail-fast` for the workspace runs.

### 395.2 The pyramid table, re-measured

The row's figures were from `893b2b94`, before §356 and before `bake`
stopped refusing the base corner where the two folding faces meet the base
(§344.3). On `25860052`:

| Fold | Base | No touch | No touch, room x1000 | No touch, rule B | This commit |
|---|---|---|---|---|---|
| 1 | 108 | 108 | 108 | 108 | 108 |
| 1/4 | 103 | 108 | 108 | 108 | 108 |
| 1/16 | 103 | 108 | 108 | 108 | 108 |
| 1/64 | 107 | 108 | 108 | 108 | 108 |
| 1/256 | 107 | 107 | 108 | 108 | 107 |
| 1/4096 | 104 | 104 | 108 | 108 | 104 |

No run gave a wrong answer. The 5 refused at 1/4 on the base are all at height
1/4 with the side planes written through a base corner: turned 0.1 about
`(3, -1, 2)`, 1.1 about `(0.3, 1, -0.7)`, and 1.9 about each of the three
axes. Four refuse on "could not settle 1 decisions" and one on "two regions of
one face could not be told apart". The 5 at 1/16 are also all written through
a base corner, four at height 1/4 and one at height 4. **None of the 10 that
`bake` refused in §344.3 is refused there on this base**: the rule-B column
answers all 648, and rule B changes only the fold, which comes after `bake`.

Over the whole table, the touch test turned away 72 fold calls that this
commit folds. 57 are inside the room: their boxes stick out 0.03 to 0.94
rooms past the apex's box and miss it by up to 0.38 of a room. The other 15
are meets of three of the apex's own planes that reach the grown box: they
stick out 1.09 to 10 rooms and miss the apex's own box by up to 0.22 of one.
§344.2 measured the 1/4 meets on the older base at 0.16 to 0.32 rooms out and
0.001 to 0.03 rooms off the apex's box; this run did not split its figures by
fold.

### 395.3 What the touch test guarded, and what guards it now

The touch test was the whole fold before §248 and §253, and those two narrowed
it. §248 showed a touch is no proof of one point: two meets of one call at
`x = 16 ∓ 2⁻⁵¹` touched and are different points. So the question is not
whether the touch was a good test, but whether anything the fold relies on
now came from it alone. There are three candidates, and each was run.

**A meet far from the corner, on the room's branch.** The room's branch asks
that every point of the meet's box lie inside the corner's box grown by
`what_writing_the_corner_down_cost`. Such a box is within the room of the
corner on every axis, so the room bounds the distance with or without the
touch. §253.4's argument for folding there is that distance, and nothing else.
`a_place_inside_a_corners_room_is_folded_into_it_though_its_box_misses_the_corners`
holds that.

**§253.3's corpus meet.** The workspace log has one line for
`a_meet_further_from_a_corner_than_rounding_reaches_is_not_folded_into_it`:
touch `true`, inside the room `false`, §356's rule `false`, sticking out
2012.67 rooms. The touch let it through and the room refused it. That test
passes with the touch gone.

**§248's two places.** The grazing block's corner `(−1, 2⁻²⁵, 2⁻⁵¹)` and the
ball's other place `(−1, 2⁻²⁵, −2⁻⁵¹)` are both exact, so the other place
misses the corner's box, and the touch turned it away. It also lies inside the
room, 0.0625 of it away, so the count of places reaching the grown box is two,
and the count turns it away too. The log has the corner's own place, which
touches, refused with `count=2` in every call on §288's two shapes: one in
`two_places_of_one_meet_inside_a_corners_room_are_neither_folded_into_it`, 36
in the grazing-block fuse test and 60 in the small-ball fuse test. The unit
test asserts both places stay nodes, and it passes with the touch gone.

**What the touch decided across the workspace.** With the touch still in, the
log recorded every call where dropping it would fold. There were 114: 76 in
`a_fused_draw_hands_its_provenance_back_through_the_door` and 38 in
`every_seed_in_the_checked_in_corpus_is_fused_without_panicking`, all on the
room's branch. They stick out at most 0.075 of a room and miss the corner's
box by at most 0.0047 of one. 12 of them have a curved surface, and in 6 of
those the meet has two places, with only the one asked about reaching the
room. The workspace run
with the touch skipped gave `2925 tests run: 2925 passed`, the base's own
count. The corpus test prints the tallies of its replay. They are identical
with the touch and without it, line for line: `ok: 346, refused: 344,
solids_made: 353`, the same both ways round 36 times.

**The one thing only the touch did.** On §356's branch the fold takes a meet
of three planes `bake` solved the corner among, however far its box sticks
out. With the touch gone, the only bound left on that branch is the count. A
meet of three planes has one place, so a count of one means that place reaches
the grown box. That widens the touch by exactly one room on every side. The
code now says so in so many words: `reaches_the_room(&place)` comes first.
`a_meet_of_three_planes_bake_solved_the_corner_among_is_folded_only_while_its_box_reaches_the_corners_room`
holds it from both sides, at the room's edge and one step past it.

### 395.4 What the room still leaves: apex meets that miss it

With the touch gone, the log of the table has 13 calls that the fold leaves,
all of the same kind. Each is a meet of three of the apex's own planes, with
§356's rule `true` and no place reaching the grown box. They stick out 5.9 to
204 rooms past the apex's own box and miss it by 1.3 to 60 rooms, so they
miss the grown box by at least a third of a room. Among the figures that stick
out are §344.2's for 1/256 and 1/4096: 5.9, 104, 197 and 204 rooms. These are
the 5 unions still refused, which
`a_pyramid_whose_apex_meets_miss_even_its_room_is_still_refused` pins.

**They are a gap.** Rule B drops the bound, and it answers all 648 unions of
the table with the two solids, their volumes and an empty log. §356.3's
argument does not use the bound either. Three planes meet once, and two
straight crossings on a flat face that both run through the corner cross
nowhere else. A meet left beside the corner on both of those crossings is a
second crossing of two straight lines, so keeping it as a node keeps a
topology that cannot be, at any distance.

**Why this row does not take it.** Rule B passes the whole workspace
(`2925 tests run: 2925 passed`) and it also changes one fold in a real input.
`every_kicad_file_that_imports_round_trips_to_the_same_solid` has a corner at
`(0.285985, −1.6e-15, 3.2e-16)` and a meet of three of its own planes at
`(0.285985, 3.24e-13, −1.78e-14)`. That meet is 22.9 rooms off, with a narrow
box, so the gap is not the box: it is the room `bake` gives a corner of an
imported solid (`Model::room_a_corner_of_this_solid_gets`, §368), which is
wider than the rounding. Today it stays a node and the round trip passes.
Under rule B it is folded and the round trip passes too. Whether that is the
same solid for the right reason is a question about the imported room, not
about the touch, and it gets its own row with this measurement.

### 395.5 The tests, and what fails without the change

- `a_pyramid_whose_apex_meets_lie_inside_its_room_but_miss_its_box_is_united_at_every_height`,
  in `a_corner_with_four_planes_round_it.rs`: folds 1/4, 1/16 and 1/64 at
  heights 1/4, 1 and 4, every turn and spelling, 324 unions. It is the
  row's test of the 5 at 1/4, and also holds the 6 at 1/16 and 1/64.
- `a_pyramid_whose_apex_meets_miss_even_its_room_is_still_refused`: the 5 of
  395.4, each refused. Its message says to move them when they unite.
- `a_place_inside_a_corners_room_is_folded_into_it_though_its_box_misses_the_corners`
  and `a_meet_of_three_planes_bake_solved_the_corner_among_is_folded_only_while_its_box_reaches_the_corners_room`,
  in `arrange.rs`'s `tests` module.

With `&& boxes_overlap(node.point, place)` put back into the first gate and
the tests kept, `mise exec -- just t -p rcad-topo --no-fail-fast -E
'test(/a_place_inside_a_corners_room_is_folded|is_folded_only_while_its_box_reaches|apex_meets_lie_inside_its_room|apex_meets_miss_even_its_room/)'`
gave `4 tests run: 1 passed, 3 failed`. The pyramid table test and both unit
tests failed, and the still-refused pin passed, as it should. `mise exec --
just t --workspace --no-fail-fast` on the commit: `2929 tests run: 2929
passed`. That is the base's 2925 plus these four, and no pin outside them
moved, in this file or in `two_meets_reach_one_corner.rs`,
`a_meet_that_fills_a_corners_room.rs` or
`a_meet_folds_into_a_corner_only_as_far_as_rounding_reaches.rs`.

The module comment of `a_corner_with_four_planes_round_it.rs` no longer says
the refusals left with §356 had every apex meet folded (§344.4). Its table
gains a column for this commit, and its "with §356" column is re-measured on
this base.

### 395.6 What this leaves

- **Apex meets of a corner's own three planes that miss its grown box**
  (395.4): 5 pyramid unions at 1/256 and 1/4096, refuses/B. Rule B closes
  them. Its row has to say whether the KiCad fold it adds is right, and that
  is about §368's room for imported solids.
- §253.6's `place_a_corner` question, and §288.5's ball shapes, are as they
  were. The fold's count is now the only thing between the grazing block's
  second place and the corner, and the unit test that holds it still holds.
