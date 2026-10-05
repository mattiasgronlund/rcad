## 200. The leaned half ball answers, `runs_up` reads the walk, and a pointed cone gives a wrong answer — 2026-09-22

*One batch, dispatched by §175's skill off `f7e9332c`: workflow run
`wf_e993188b-e4b`, three Opus lanes. The two lanes that changed `src/` were each
checked by a Sonnet verifier. The cone lane was test-only, so it had no verifier.
`f7e9332c` was green by the gate's own record before the batch. `just gate quiet`
run here on the merge `7067c4b6`: PASS, 2251 run, 2251 passed, 6 skipped. A
number below is a lane's unless it names a command run here. Numbers that came
from a probe since removed are marked as the lane's word.*

### 200.1 The leaned half ball was never a hole: the cut crosses the flat face

`lane-leaned-half-ball` (`05206e64`) closes §199.4's first item. It was a gap,
not a limit. The pin's placement is not the hole §199.1 described. With the
lean 20° and the turn −0.4, the cut plane's normal has `n_x = cos 20°·cos 0.4 =
0.8655`, under `√0.75`. So the cut circle dips 0.00102 past the flat face
`x = 0` (checked by the verifier with the same `python3` line). The skin piece
is one piece with four corners: both poles, and the two places where the cut
crosses `x = 0`. All four lie on `x = 0`. Every blend of them lands back on
`x = 0`, where the side test is undecided (a box 1.1e-13 wide, the lane's word).
A step along an axis from their middle lands in the cut-off cap or outside the
half ball. So every candidate point failed, and the fuse refused *a point inside
a piece of a face*. `in_the_bulge_of` and `half_way_into_either_arc` were not
involved; they run on flat pieces only.

The fix adds one more candidate point in `candidate_points`' curved branch. It
steps from the corners' middle along the sum of each cutting plane's material
direction, counting each plane once. It is tried after all the old candidates.

The pin is now an answer test. A sweep of 21 turns from −0.5 to 0.5 at 20° lean
answers, validates and measures. The expected volume now takes off only the
part of the cap that lies inside the half ball, computed by a slice integral.
The verifier put `fuse/` back to the base: 2 of 4 tests failed, the two new ones.

**It moved two pins in `a_narrow_lune_from_a_pole_corner.rs`, re-pinned here**
(`7067c4b6`). 14 wedges now answer, validate and measure against their closed
form: 5 cut by two knives (170° at every radius, 10° at radius 3) and 9 by one
prism (10° at every radius, 45° at radii 1, 3 and 1000, and 90° and 170° at
radius 1), all of them intersections. Each test now lists its answered cases,
and every other case must be refused by name. `ResultIsNotManifold` and
`CannotMeshAFace` join the accepted names, because most of the other cases now
get past the curved pieces and refuse later. The lane probed one of them: two
knives, 90°, radius 1. There a flat half disk of `x = 0` is settled inside the
knife at `(0, 1.1e-16, −0.5)`. That point lies in the 4.4e-16 sliver between
the polar axis and the turned knife's stored plane, which misses the poles by a
rounding. **That is a flat piece placed on the wrong side.** Here it ends in
`ResultIsNotManifold`, not in a wrong answer. It is an older fault in how flat
pieces are placed, reached for the first time.

**2026-10-05, Q990: what Q901 moved.** Q901
(`lane-q901-land-two-meets-of-three-planes`, tip `75adb1f8`) has `order_along`
ask the planes (`as_the_planes_say`) where the boxes cannot say, and
`passes_through` ask a corner's three planes. Its commits record six pins
moved. In `a_narrow_lune_from_a_pole_corner.rs` the two pins above now see the
45° wedges stop at *a point inside a piece of a face* and not at *order along a
crossing*; no case newly answers (`fdef5ce3`). In
`a_plane_cut_along_a_cones_other_nappe.rs` two tallies grow: three eighths
from 36 right to 48, and the block below the frustum from 68 to 72
(`45fbcfaf`). Two walls a hair apart now stop, under the tolerant policy, at
the sliver between `y = 5` and the wall leaned by `1e-100`, about `1e-99`
wide, where no `f64` point lies: a limit (`44f231d6`). The skew family's 39
refusals in 441 now all name *a point inside a piece of a face*, where they
named *order along a crossing* (`6e70b96a`). Two turned cubes a rounding
apart now refuse in selection; the cubes share no point, so that refusal is a
gap, shown by
`every_corner_of_each_cube_is_in_front_of_the_others_touching_face_only_when_apart`
(`75adb1f8`).

**Q901 has not landed, and Q990 did not land it either.** On `main` the cut through
`PinHeader_1x39_P2.54mm_Vertical` refuses early. With Q901 it runs through
every face of the header's body. `cuts.rs` cuts each face by every plane of the
body, 332 to 409 of them, and `place_the_meets` places a meet for every pair:
38 116 nodes on a face with four corners, of which 54 arcs are left. Q990
removed four costs that grew faster than that: the corner fold looked through
every node for every meet, the tighter box was worked out for every meet of
three planes, `lumps_with_a_curve_that_runs_on` looked each arc up by address,
and every piece worked out every walk's lump again. The 1x39 cut still did
not finish in 300 s. A probe, not kept, left out every cut whose partner keeps
the face's whole box on one side of it, before any meet is placed. With that,
the cut finished in 91.67 s (on a machine with a load average over 30), and
refused *a point inside a piece of a face*, as the 1x11 cut does. That rule
needs the face's box, which `bake.rs` keeps private to `Bounds`. The next cost
after it is `crossing_like` in `draw_the_crossings`, which compares every new
crossing with every one already drawn by exact determinants: 7 of 15 samples
of the probed cut.

**2026-10-05, Q1005: the cut through the 1x39 header in 5.81 s.** Q1005
merges Q990's branch and makes four changes in `arrange.rs`. Every time
below is one run, with the load average beside it. On Q990's tip the cut
took 270.40 s (load 16); after the four changes it takes 5.81 s (load 29),
and it refuses with the same record, *a point inside a piece of a face*.

1. `leave_out_the_cuts_clear_of_the_face` runs before any meet is placed.
   It leaves out every cut that runs along no boundary edge, is on no
   corner, and whose surface `side_of_surface` proves on one side of the
   face's whole box (`Bounds::as_a_point_anywhere_inside`, new in
   `bake.rs`). Such a crossing is on that surface, so it has no point in
   the box, and the box holds the face as far as `Bounds` says. So it draws
   nothing on the face. `place_the_meets` then skips every pair with such a
   cut in it, not only pairs of two as Q990's probe did: those meets are
   points outside the box, and skipping them took the time spent placing
   meets in the cut from 9.9 s (load 3) to 2.5 s (load 35). Its pin is
   `planes_clear_of_the_box_round_a_face_place_no_meet_and_cut_nothing`.
2. `crossing_like` compares a new line only with lines that may be the same
   one. `where_the_line_two_planes_cross_in_sorts`, new in `planes.rs`,
   gives each plane's line across a flat face a number that is the same for
   every plane through that line, boxed. Lines whose boxes share no point
   are different lines. With 1 and 2, the cut took 18.45 s (load 7). The
   pins are `a_new_line_is_compared_only_with_the_lines_it_may_be` and the
   two tests of the new function.
3. `the_walk_of_each_other_lump_each_walk_faces` compares only walks in two
   different lumps. It used to go through every pair of the 61 583 walks of
   the body's top face and pass over pairs in one lump. With 1 to 3, the cut
   took 6.03 s (load 29).
4. `the_nodes_on_each_crossing` collects each crossing's nodes in one pass.
   Before, `nodes_along` searched all the nodes once per crossing, and both
   `merge_points_that_are_one_point` and `cut_the_crossings_into_arcs` call
   it. Over the whole corpus that took merging from 12.3 s to 4.5 s, and
   cutting arcs from 7.4 s to 2.8 s (load 35 to 50).

Nothing moved in `rcad-topo` or `rcad-predicates`: their 1 593 tests pass,
the four new ones among them, with no pin changed. `every_kicad_file_that_imports_round_trips_to_the_same_solid`
took 35.618 s on this branch (load 17 to 29), against 21.600 s on `main` at
`eccf76b7` (load 22 to 31).

**What is still slow.** Measured over the corpus at a load of 35 to 50, 60
faces with more than 5 000 nodes spent 13.5 s placing 1.68 million meets of
three planes, about 8 µs each, and the other 10 486 faces spent 6.1 s.
`gather` in `cuts.rs` meets every plane with every plane, and in an early
sample it was 4 of 11 samples of the cut. Q1005 did not own that file.

### 200.2 A cap whose rim runs through the other pole is cut the way its walk ran

`lane-runs-up-other-pole` (`d1f7eab9`, `76204dff`) closes the `runs_up` item
from §198.1. `add_ball` plus a fuse cannot reach that cap: the ball's seam ends
at the pole the rim runs through, and every such cut was refused
`CannotTellTheCellsOfAFaceApart` (or `AWireWillNotLieFlatOnItsFace` at lean
1.1) before `runs_up` was called. That is a separate fuse gap, and it is still
open. `SolidBuilder` does reach the cap, as a sphere skin bounded by one circle
whose only corner is the south pole, plus a flat lid.

The lane built 63 caps holding the north pole: radii 0.5, 1 and 2, leans 0.3,
0.7 and 1.1, and seven turns. On the base all 63 were refused (45
`CornersCoincide`, 12 `FacesDoNotAgreeWhichWayTheyFace`, 6
`BoundaryCrossesItself`; the split is the lane's word). The verifier saw the
first one fail on base `mesh.rs` with `CornersCoincide`. The fix has three
parts:

1. `WalkedLoop` gets a `climbed` field, set in `WalkingAWire::finish`, and
   `runs_up` reads it. `how_far_the_longitude_climbed` is gone.
2. A cap is walked from its first corner that is not on a pole, and it gets a
   corner on a pole side wherever a quarter turn falls inside that side.
3. When a quarter turn is equally near two corners, the one off the pole wins.

All 63 now mesh closed, with V − E + F = 2, and measure within `3δ/r` of the
closed form. No other pin moved.

The lane found a second change and did not keep it. At the far pole the pole
side ends a quarter turn back, not at the meridian the next cut leaves by.
Ending it at the leaving meridian makes §191's pinned refusal
`a_turned_ball_united_with_a_block_will_not_mesh_where_the_upright_one_does`
answer: 126.82722 turned against 126.82571 upright (the lane's word, from an
`eprintln` since removed). That would move a pin, so it was left for a lane of
its own.

### 200.3 A pointed cone fused with a corner-up cube gives a wrong answer

`lane-cone-other-nappe` (`9a4ab1f0`) was test-only. It asked whether §199.3
point 1 can end in a wrong answer. `a_plane_cut_along_a_cones_other_nappe.rs`
runs 408 fuses in six families. All 72 answers are right, and the other 336 are
refused by name. A plane steeper than the cone that also crosses the frustum
refused 224 of 224 times, so no wrong answer came from that path.

**But a wrong answer exists, and it is left out of the file**, because nothing
can pin it without an `#[ignore]`. A pointed cone (slope 1/2, height 8) fused
with a cube of side 4 standing on its corner, centred on the apex: every rule
answers `Ok` and the model validates, but both solids come back untouched. The
intersection gives 0 solids where the shared volume is about 4.3964 (a column
sum, not a closed form). The union gives two solids of 198.0246 in all, where
the answer is 193.6449. It happens at slopes 1/4 and 1/2 and cube sides 3 to 5.
The lane's reproducer, not run here, using the new file's `a_pointed_cone`:

```rust
let policy = Policy::certified();
let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
let cone = a_pointed_cone(&mut model, 0.5, 8.0);
let diagonal = Axis::new(Point::origin(), Vector::new(1.0, -1.0, 0.0)).expect("a direction");
let stood = Isometry::rotation(&diagonal, (1.0f64 / 3.0).sqrt().acos()).expect("a finite turn");
let placed = Isometry::translation(Vector::new(0.0, 0.0, -2.0 * 3.0f64.sqrt()))
    .expect("a finite move")
    .compose(&stood);
let square = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
    .build(&policy, &mut ProvenanceLog::new())
    .expect("a square is a profile");
let cube = model.add_extrusion(&square, 4.0, &placed).expect("a square extrudes");
let answer = model
    .fuse(&[cone], &[cube], Rule::Intersection, &policy, &mut ProvenanceLog::new())
    .expect("the fuse answers");
model.validate(&policy, &mut ProvenanceLog::new()).expect("and validates");
assert_eq!(answer.solids().len(), 1); // fails: 0
```

There is a second wrong answer, with no fuse at all. That pointed cone's own
`classify_point` answers `Outside` at `(0, 0, 1)`, `(0.1, 0.07, 1.03)` and
`(−1.4, 0.07, 3.03)`, which are all inside it. Over a half-unit grid it answers
`Outside` at 106 points inside the cone, and `CannotFollowAFacesTrimming` at
3385 more. The frustum and the blocks agree with their closed forms everywhere.

Which one causes the fuse's wrong answer is **not shown**. It could be the
stray cut along the other nappe, or the cone's wrong `classify_point`. What
points away from the nappe: the frustum against the same cubes never answers
wrong, and the cone's classification is wrong before any fuse runs.

The merge renamed the file's catch-all `Containment` arm to list each variant,
because `check-wildcards` counted it as a new wildcard.

### 200.4 What is left

- **The pointed cone's wrong answers, 200.3.** This is first: a wrong answer
  comes before any refusal. Start with `classify_point` on a pointed cone
  (`(0, 0, 1)` at slope 1/2), then see whether the fuse's wrong answer goes
  away with it. The reproducer is in 200.3.
- The flat half disk placed on the wrong side of a knife through the sliver
  (200.1). It ends in `ResultIsNotManifold` today.
- The far pole's pole side ending a quarter turn back (200.2), which would
  answer §191's pin.
- `add_ball` plus a cut through a pole refused `CannotTellTheCellsOfAFaceApart`
  (200.2).
- Still open from §199.4: the fuse cutting along a cone's other nappe (now
  known to end in a refusal in 224 of 224 cases), `mesh.rs:839`, and the named
  refusal for a closed edge that neither direction rule answers.

Commands: `just decision 199.1`, `199.3`, `198.1`, `191`, `16.10`.
