## 356. A meet of three planes `bake` asked about a corner folds into it — 2026-09-28

*Queue row Q272, from §310.3 and §301.7. Lane
`lane-q272-a-pyramid-folding-by-a-sixty-fourth`, off `9612b25f`. A turned
pyramid whose base is bent by 1/64 or less was refused at every certified
union with a cube clear of it: 0 of 108 at folds 1/64, 1/256 and 1/4096.
**It was a gap, and it is closed.** The refusals came from the fold in
`a_corner_is_this_place` measuring the corner's room in space, when the room
is a room across each plane: `bake` asks every plane round a corner whether it
passes through the corner's box grown by the room, and a meet of three of
those planes, two of them nearly one plane, lands up to 1/fold rooms away in
space. **Tightening the meet's box does not close it**: the point the three
stored planes meet at, worked out exactly, still sticks out 1.6 rooms at 1/64
and 108 at 1/4096 (356.2). The fold now also takes a meet whose three surfaces
are planes the corner's own `around` list holds, which `bake` asked. Three
planes meet at most once, so that meet is the corner written down by a worse
triple. The table goes from 0, 0, 0 to 107, 105, 97, the counts the room
×1000 used to give, and the room no longer moves any of them (356.3). In the
whole workspace the rule adds 7 folds, all in the two tests this row changed
(356.4). Code: `crates/rcad-topo/src/fuse/arrange.rs`, commit `63f75211`.*

  356.1   The gap, re-measured
  356.2   The families of fix, and why the room in space is the wrong one
  356.3   The rule, and the table after it
  356.4   What else it folds, across the workspace
  356.5   The tests, and what fails without the rule
  356.6   What this leaves, and what Q275 should measure first

### 356.1 The gap, re-measured

§310.2's table was a citation; this row re-measured it. The probe was
§301.6's, a scratch edit to `a_corner_is_this_place` that logs, for every
corner the fold is asked about past its first gate, how far the meet's box
sticks out past the corner's box in rooms, and scales the room by an
environment variable. The scratch test, an `#[ignore]`d function in
`a_corner_with_four_planes_round_it.rs` taken back out before the commit,
fused the 108 unions per fold of that file's module table and counted the
answers. Each run was `mise exec -- just t -p rcad-topo --test
a_corner_with_four_planes_round_it --run-ignored all --no-capture
q272_scratch_table`, with `Q272_PROBE` and `Q272_SCALE` set, on `9612b25f`
plus the probe.

| Fold | Meet's box sticks out, rooms | Answered | Room ×0.5 | Room ×1000 |
|---|---|---|---|---|
| 1 | 0.123 | 108 | 108 | 108 |
| 1/4 | 0.242 | 103 | 103 | 103 |
| 1/16 | 1.171 | 99 | 31 | 103 |
| 1/64 | 4.731 | 0 | 0 | 107 |
| 1/256 | 20.71 | 0 | 0 | 105 |
| 1/4096 | 290.5 | 0 | 0 | 97 |

Every figure matches §310.2's. The "sticks out" column here is the largest
over every corner the fold was asked about, not over the apex alone; that it
matches the apex figure says the apex is where it peaks.

`mise exec -- just t -p rcad-topo` on the base: 1139 run, 1139 passed, 11
skipped (log kept at `target/q272/before-rcad-topo.log` in the lane's tree).

### 356.2 The families of fix, and why the room in space is the wrong one

The fold takes a meet as a corner only when every point of the meet's box lies
inside the corner's box grown by `what_writing_the_corner_down_cost` (§253.4).
That fails here for one of two reasons: the meet's box is loose, or the point
it holds really is that far away. The fixes fall into three families by which
of the three things they change: the room, the meet, or the corner.

**The room in space, made wider.** §310.2's ×1000 column is this family, and
it closes the gap. It is also a tolerance: it grows the room for every corner
of every face, including the cuts and curved triples §253 held to it, to
answer a question about one kind of triple. The brief ruled it out and so does
§253.4. Not built.

**The meet, worked out tighter.** §253.7 expected this: "when the meet's box is
tightened the test's pin moves". The probe was extended with an exact test: the
meet of the three stored planes as Cramer's rule writes it before it divides,
each numerator and the volume an exact sum of `f64` through
`rcad_predicates::expansion`, compared with the corner's box one side at a
time. It measures how far the true meet of the stored planes sticks out, with
no box at all. Same command and runs as 356.1:

| Fold | Meet's box sticks out | The exact meet sticks out |
|---|---|---|
| 1 | 0.123 | 0.041 |
| 1/4 | 0.242 | 0.117 |
| 1/16 | 1.171 | 0.457 |
| 1/64 | 4.731 | 1.557 |
| 1/256 | 20.71 | 6.862 |
| 1/4096 | 290.5 | 107.7 |

**So no enclosure of the meet, however tight, folds every one of them at 1/64
or below.** The box sticks out about three times as far as the point inside
it, and the point is still past the room: at 1/64 at least one meet, worked out
exactly, lies 0.56 rooms beyond the corner's grown box. A meet the fold leaves
standing overlaps the apex and refuses its union under `Policy::certified`
(§310.2), so this family leaves at least one union per fold refused. How many
unions an exact fold would answer was not measured: the exact test ran as a
probe only, and never decided a fold. The reason is the planes, not the
arithmetic: each side plane is rounded where `Plane::through_three_points`
wrote it, and two planes that fold by `f` cross in a line that moves by about
the rounding over `f` when either one moves. So the four stored planes do not
meet at one point, and the meet of a nearly-flat triple lands up to about
1/fold times the rounding away from where a well-placed triple puts it; the
exact column grows about fourfold per fourfold smaller fold, which is that. The
exact test is not in the commit. By §253.7's exact figures it would also fold
§253.3's tolerant draw, whose meet is `4.1e-16` from its corner against a room
of `7.1e-14`, and so move that row's pin; that was not run.

**The corner, made wider.** `bake` could hand on a box round every triple's
meet instead of the best triple's. Every meet would then be inside the
corner, and the fold would take it. That spends the corner's precision in
every predicate that reads it — a cut through the apex, the order of nodes
along its edges, the stored vertex — to answer a question about which of two
nodes is the corner. Not built.

**Which leaves the room itself, measured where it was granted.** `bake`'s
`derive_a_corner` solves a corner with three or more surfaces round it from
its best-conditioned triple, then asks *every* surface round it about the
corner grown by the room, and refuses if one proves it off. So each plane in
the corner's `around` passes through that grown box: the room is a room
across each plane — how far a plane through one corner of a face misses the
next, which is the argument `what_writing_the_corner_down_cost` gives. The
fold measured the same number in space, round the meet, which is the same
thing for a well-placed triple and 1/fold times too tight for a nearly-flat
one. These are the only three things the fold's question reads — the room, the
meet's place and the corner's place — so a fix is in one of these families or
is not a fix of the fold.

### 356.3 The rule, and the table after it

`a_corner_is_this_place` now folds a meet into a corner node on both its
crossings, whose box the meet's box touches, when either:

- every point of the meet's box lies inside the corner's box grown by the room
  (§253.4, unchanged); or
- **the face's surface and the two crossings' partners are three planes, and
  all three are in the corner's `around` list, which has three or more
  surfaces** (`Arrangement::bake_placed_the_corner_on_the_three_planes`).

The count condition — no second place of the meet reaching the grown box —
applies to both, and for three planes there is only one place.

`around` with three or more surfaces is exactly the corners `derive_a_corner`
placed and asked about; with fewer it was read out of the model and nothing
asked (§286), and the rule does not apply. `BakedCorner::around`'s doc comment
now says so, because the rule leans on it.

**Why it is not a tolerance.** Nothing is widened. The rule reuses the answer
`bake` already gave with the room it already used, and adds one fact: three
planes meet at most once, so two straight crossings on a flat face that both
run through the corner cross nowhere else. The meet is not a second point that
might be the corner. It is the corner, written down by a worse triple. A node
left beside it hands the policy "are these one point?", and under
`Policy::certified` that question is unanswerable for a reason that is not
about the shape.

**What it does not reach.** A triple with a curved surface in it, where two
crossings can meet twice. A triple with a cut in it, which the corner was never
asked about — that includes the other solid's copy of a shared plane written
reversed, since `around` is compared as stored numbers. A corner with fewer
than three surfaces. All of these keep §253's rule alone. The one tolerant
fold §253.3 found past the room is the second kind: its corner reached one
crossing through a tolerant `passes_through`, so it stays refused, and
`a_meet_further_from_a_corner_than_rounding_reaches_is_not_folded_into_it`
still passes.

The table after, on the probe build with the rule switched in by an
environment variable, same command as 356.1 plus `Q272_RULE=1`:

| Fold | Answered, room ×1 | ×0 | ×0.5 | ×1000 |
|---|---|---|---|---|
| 1 | 108 | 108 | 108 | 108 |
| 1/4 | 103 | 103 | 103 | 103 |
| 1/16 | 103 | 103 | 103 | 103 |
| 1/64 | 107 | 107 | 107 | 107 |
| 1/256 | 105 | 105 | 105 | 105 |
| 1/4096 | 97 | 97 | 97 | 97 |

The ×1 column was measured again on the committed code: `63f75211` with a
scratch test counting the same 108 unions per fold put back in and taken out
again, no probe and no environment variable, same command: 108, 103, 103, 107,
105, 97. In every run with the rule, every meet the fold was asked about past
its first gate was folded, at every fold.

The room no longer moves any count in this family, and the counts are the
room ×1000 column of 356.1. The refusals left, listed by the scratch test at
×1, are "could not settle 1 decisions the answer depends on" or "two regions
of one face could not be told apart": 5 at a fold of 1/4, 5 at 1/16, 1 at
1/64, 3 at 1/256 and 11 at 1/4096, all at heights 1/4 or 4 except one at
1/4096 and height 1 (planes through the apex). They are §310.5's second bullet
and Q273's, and were not traced here.

### 356.4 What else it folds, across the workspace

The probe logged, for every fold call past the first gate, whether the box
rule held and whether the new rule held, with the test's thread name, written
one line per call in a single write so that parallel test processes do not
interleave. `Q272_PROBE=<file> Q272_RULE=1 mise exec -- just t --workspace
--no-fail-fast`, on the probe build:

```
Summary [  89.829s] 2705 tests run: 2703 passed (3 slow), 2 failed, 12 skipped
new folds (box fails, rule holds): 7
neither: 4
  6  a_pyramid_whose_two_faces_fold_by_a_sixty_fourth_is_refused_because_its_apex_meets_stick_out_past_the_room
  1  fuse::arrange::tests::a_place_as_far_off_a_corner_as_its_room_is_folded_into_it_and_one_a_step_further_is_not
```

The two failures are those two tests, and they are the only places the rule
folds a meet the room did not. The four calls neither rule folds are §253.3's
one draw, three times over: `a_fused_draw_hands_its_provenance_back_through_the_door`
(2), `a_meet_further_from_a_corner_than_rounding_reaches_is_not_folded_into_it`
and `every_seed_in_the_checked_in_corpus_is_fused_without_panicking`. The
property suites draw at random, so this is one run's count and not a bound.

### 356.5 The tests, and what fails without the rule

- **`a_pyramid_whose_two_faces_fold_by_a_sixty_fourth_is_united_though_its_apex_meets_stick_out_past_the_room`**,
  renamed from `..._is_refused_because_...` and turned round as its panic
  message asked: at folds 1/64 and 1/256, height 1, all 36 turns and spellings
  of each unite into the two solids, volumes to a billionth, empty log. The
  sixteenth's test and this one now share `every_turn_and_spelling_is_united`.
- **`a_meet_of_three_planes_bake_solved_the_corner_among_is_the_corner_however_far_its_box_reaches`**,
  new, in `arrange.rs`'s `tests` module: at the unit cube's corner at the
  origin, a place reaching a thousand rooms off folds on the bottom and the two
  walls, and does not on a wall and the cut `x = y`.
- **`a_place_as_far_off_a_corner_as_its_room_is_folded_into_it_and_one_a_step_further_is_not`**
  (§301.5) moved from the two walls to a wall and the cut `x = y`: on the walls
  the new rule folds a place however far it reaches, so the room no longer
  decides there, and on a cut it still does. Its places and its two asserts
  are as they were. The room ×0.5, ×0.1 and ×1.25 mutations of §301.5 were not
  run again; the question the test asks goes through the same box rule as
  before.

With the rule's call site switched off (`inside_the_room || (false && …)`) and
the tests kept, `mise exec -- just t -p rcad-topo --no-fail-fast -E
'test(/a_pyramid_whose|a_place_as_far_off_a_corner|a_meet_of_three_planes_bake_solved/)'`:
4 run, 2 failed — the sixty-fourth pyramid and the new unit test — and the
sixteenth pyramid and the room test pass.

`mise exec -- just t -p rcad-topo` on `63f75211`: 1140 run, 1140 passed, 11
skipped. Against the base's 1139, the answers that moved are the three tests
above: one renamed and turned round, one new, one moved to a cut. The
sixteenth's test still passes and no longer pins the room from below: with the
rule, a room ×0.5 answers all its 36 (356.3's ×0.5 column is 103 of 108 at
1/16, and the five refused are at heights 1/4 and 4). So §310.4's "only two
tests fail at ×0.5" should now be one, the hand-written test; that is a
forecast, not re-run across rcad-topo. §301.10 (Q495) re-ran it: the
sixteenth's test does pass at ×0.5, but four tests fail, not one — the
hand-written test and three more unit tests beside it in `arrange.rs`.

### 356.6 What this leaves, and what Q275 should measure first

- **The refusals left in the table** (356.3's last paragraph) are Q273's. The
  rule changed none of them: each had every apex meet folded before it.
- **The base corners sticking out 1.25 rooms** (§310.5, Q274) are three-plane
  corners, so a meet there of the corner's own three planes now folds by the
  rule whatever it sticks out; whether Q274 still has a question is Q274's to
  say.
- **§253.3's tolerant draw** is still refused, for a reason this rule does not
  reach: its corner is on one crossing by a tolerant `passes_through`, not by
  a plane `bake` asked.
- **The exact meet test** of 356.2 is not in the code. If a row finds a meet
  that the room in space should fold and a loose box stops, it is the fix, and
  its measurement here says it closes nothing in this family.

**Q275 should first measure where the apex refusals now come from, not the
fold.** At a corner with five or more planes, every face triple is three of the
corner's own planes, so the rule folds all of them however they stick out; the
probe's "neither" count there should be zero, and that is the first thing to
confirm. After that, in this order:

1. **An apex far from the origin.** The rule leans on `derive_a_corner`'s
   check, and that check grows the corner by a room read off the corner's own
   size, not the model's (§136.4). An apex far out, on faces whose other
   corners are near the origin, is where `bake` itself may refuse
   (`VertexOffItsFacePlane` or `where three surfaces meet`) before any fold is
   asked. Count `bake` refusals separately from fuse refusals.
2. **A sliver angle at the apex.** There the best triple `bake` can find is
   itself nearly flat, so the apex's own box is wide, and what refuses is
   likely a predicate reading that box — the order of two nodes along an edge
   into the apex, or the turn at it — rather than a fold. Log the apex box's
   width in rooms beside each refusal's predicate.
