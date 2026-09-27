## 58. The floor a mesh publishes is one the walk will take, recorded 2026-09-06

### 58.1 The unit, and why its record is late

Ten commits, `c9ff754..8358620`, in three lanes — floor, tolerant and import —
merged at `86b72c3` and closed with three documentation commits. It changed
`SolidView::room_a_mesh_has_to_work_in`, added two functions to `mesh.rs`, moved
what `Error::DeflectionBelowWhatAMeshCanReach` hands a caller, added five tests
across three files, and discharged `docs/consumers.md`'s announcement bullet for
the first time. **It left no section here at all.** This is that section, written
from the commits and from the tree rather than from anyone's report.

### 58.2 One number doing two jobs, and only one of them was its job

`room_a_mesh_has_to_work_in` is asked two different questions and had one answer
for both.

- **Let a caller's own deflection through, or refuse it.** The floor that belongs
  here is `how_far_a_mesh_point_of_this_face_rounds`: how close to the surface a
  mesh point can be **placed**, once the evaluator has rounded against how far
  the surface reaches and the placement has rounded again against how far the
  face reaches where the solid sits.
- **Pick a deflection for a tolerant caller who asked for finer than the
  arithmetic has.** Here there is nobody left to hand a refusal to. Whatever
  comes out of that line goes to `rcad_geom::walk` with no-one else looking at
  it.

The placing floor says nothing about **how many points** the walk will put along
one crossing, and the walk will not put more than
`MOST_POINTS_ALONG_ONE_CROSSING` of them. So the fallback handed the walk twice a
number the walk refuses, and a caller got
`rcad_geom::Error::TooManyPointsAlongOneCrossing { most: 1048576 }` for a
deflection this crate had just worked out itself. A round bar of radius one at
the origin, `Deflection::of(1e-300)`, `Policy::tolerant(100.0)`: four public
calls, no fuse, no import, no placement, nothing leaning.

`whether_the_floor_predicts_the_step_cap.rs`'s header carries how far short it
was: `4.8706` times from the step cap on its own, then divided by the three
sixteenths of the deflection the walk is actually handed, which is `25.98`, and
`walk`'s own deviation puts the crossover a little past twenty-six. The measured
bracket either side of it is between sixteen and thirty-two.

### 58.3 The fix: a second floor, and which one answers whom

`the_finest_deflection_the_walk_will_take` is the placing floor with one term
added — `how_far_a_step_at_the_cap_stands_off_its_chord`, what the arc of one
step stands off its own chord when a whole turn is walked in as many steps as the
walk allows. The added term goes in exactly where the evaluator's roundoff goes
in, so it pays the same two cuts on the way down: the movement into place comes
off the deflection before the walk is called, and the walk is handed
`WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO` of what is left.

**Which floor is read depends on who chose the number, and that is the whole
design.** A caller who names a deflection is let through against the *placing*
floor and gets whatever the walk or the triangulator then says about the number
that caller chose. A deflection this crate picks is measured against the *walk*
floor, so the policy is asked about the number it will actually get. Asking a
policy about the first and then handing back the second is telling a caller its
budget was kept and then straying seventy-nine times further than it allowed.

`ROOM_ABOVE_THE_FLOOR` stays at `2.0` and its reason changed. It is no longer a
multiple of the placing floor, and no multiple of the placing floor would have
served: at the floor the walk asks `acos` for an angle three millionths of a
radian wide, where the subtraction in `1 − room/reach` has few digits left, and a
step count one over the cap is a refusal.

### 58.4 What `closest` carries now, and the separation, measured here

`DeflectionBelowWhatAMeshCanReach`'s `closest` used to be the placing floor and
is now the walk floor. That is a change to a published number and
`docs/consumers.md` records the announcement of it.

Re-measured for this section rather than copied, through a probe outside the tree
that calls only the public API: `Model::add_round_bar` / `Model::add_ball`, then
`SolidView::mesh` under `Policy::certified()`. The walk floor is read straight
out of the refusal's own `closest`. The placing floor is **hunted for** — bisect
for the deflection at which `DeflectionBelowWhatAMeshCanReach` stops being the
answer — which is the same hunt
`the_floor_under_a_deflection_stays_put_while_the_crossing_it_stands_for_runs_away`
makes, and neither number is worked out from the arithmetic that produces it.

| solid, asked `1e-300` | placing floor, hunted | walk floor, from `closest` | apart by |
|---|---|---|---|
| round bar, radius 1, height 2, at the origin | `9.214851104388813e-13` | `7.273230840146953e-11` | `78.92944506377196` |
| ball, radius 1, at the origin | `6.143234069592546e-13` | `4.848820560097969e-11` | `78.92944506377192` |

The ball's row is as measured for this section and is **superseded**: the
round-reach unit, merged as `fdf5e582`, halves both of its floors to
`3.0716170347962663e-13` and `2.4244102800489844e-11` — that unit left no
decisions section of its own, and `docs/consumers.md` carries the figures now.
The bar's row and both ratios stand.

**The seventy-nine is a fact about the added term and not about either shape.**
Two solids that share no face agree on it to fifteen digits, because at the
origin `how_far_moving_this_face_rounds` is zero and the ratio collapses to
`(POINT_ALONG_A_CROSSING_ROUNDOFF + the step's stand-off) /
POINT_ALONG_A_CROSSING_ROUNDOFF`. It is not a factor to carry about: away from
the origin the movement term dominates both floors and they converge.

Two honesties about that table. The hunted column **converges from above** — a
hundred geometric halvings from `1e-300` stop at the first deflection not ruled
out, so it can sit a few `ulp` high, and the ball's reading is thirteen `ulp`
above the `6.143234069592533e-13` `mesh.rs` publishes while agreeing with it to
thirteen digits. And **the same hunt does not work far from the origin**: there
the first deflection above the placing floor is one the walk accepts, so the
probe meshes for real and does not finish — the wall
`whether_the_floor_predicts_the_step_cap.rs` names in *why there is no test here
that meshes at the new floor*. What this section did measure out there is the
walk floor alone, `3.9968513728593615e-6` for a unit ball at `x = 1e9`, which is
the figure `docs/consumers.md` announced. That one is **superseded by the
round-reach unit** (`fdf5e582`, which left no decisions section of its own)
too, at `3.996827132753364e-6` — and it moved in the fifth figure where the pair
at the origin halved, because out there the placing term is eighty thousand times
the added one. It now has a test,
`a_unit_ball_a_thousand_million_units_out_is_cut_to_what_the_docs_announce`,
which it did not when this section was written.

### 58.5 Every bound this unit publishes, against the test that carries it

§15.6 says a published bound needs a test. Row by row, checked in the tree at
`99d6458`, where the three `rcad-topo` binaries this unit touches run 31 tests
and the `rcad-step` one runs 2, and all 33 pass.

| what is published, and where | carried by | holds? |
|---|---|---|
| `closest` names a deflection the walk will take (`error.rs`, `mesh.rs`) | `a_tolerant_caller_is_refused_by_the_floor_and_not_by_the_walk` | **in part** — see below |
| the mesher settles a fallback against its own floor instead of handing the walk a deflection it refuses | the same test, by the refusal's *name* | **yes** |
| a bar a billion units out still gets its old fallback, 5134 points and `5.99402529715387e-6` of deviation | `the_fallback_a_bar_a_billion_units_out_already_had_says_the_same_thing`, asserting both exactly | **yes** |
| a tolerant caller is refused as if its solid were broken where `room_left_over` gives up, over twelve cells | `a_tolerant_caller_is_refused_as_if_its_solid_were_broken_and_a_certified_one_is_not` | **yes** |
| those bars are not the thing that is wrong: built certified, empty provenance log, `validate` passes certified | `every_bar_in_the_table_is_built_and_checked_with_no_tolerance_asked_for` | **yes** |
| the mesher's floor stays put over four leans while the crossing's rounding runs away from it | `the_floor_under_a_deflection_stays_put_while_the_crossing_it_stands_for_runs_away` | **yes, as a shape** — no value asserted |
| styling is never a key in `shapes_this_reader_did_not_take`, and a styling-only file gets an `Err` rather than an empty map | `styling_is_never_among_the_shapes_walked_past`, over the real corpus | **yes** |
| `9.2148511043888e-13` and `7.273230840146953e-11`, the two floors of a bar of radius one at the origin (`mesh.rs`) | `a_round_bar_at_the_origin_places_a_point_a_thousand_times_closer_than_it_can_be_cut` | **yes**, since this section was written |
| `6.143234069592533e-13`, `4.848820560097969e-11` and *78.93 times apart* for a ball at the origin (`error.rs`) | `a_unit_ball_at_the_origin_places_a_point_a_thousand_times_closer_than_it_can_be_cut`, and the ratio on both shapes at once | **yes**, and the two floors are **superseded** by the round-reach unit (`fdf5e582`, no decisions section of its own) — `3.0716170347962663e-13` and `2.4244102800489844e-11`. The ratio stands |
| `741 455` steps at twice the floor against `1 048 576` at it (`ROOM_ABOVE_THE_FLOOR`) | `what_twice_the_room_buys_a_walk_at_its_step_cap.rs`, which also pins the one-step margin and the design room the count belongs to | **yes**, since this section was written |
| the leaning-cap figures: a conic reach of `9.999999999999999e8` at `lean = 1e9`, a crossing rounding of `5.6843418889229724e-5`, a floor of `9.697641120195938e-11` (`mesh.rs`) | nothing, and nothing is owed — `f9d54e0` deleted the doc block that held all three | **withdrawn** |

**The first row is the one worth reading twice.** The test asserts
`closest > 1 − cos(π / MOST_POINTS_ALONG_ONE_CROSSING)` on a bar of radius one at
the origin, which is a **necessary** condition: below that the walk certainly
refuses. It is not the promise the unit is named for, which is that the
deflection the mesher then picks — `closest` through `ROOM_ABOVE_THE_FLOOR` and
two more cuts — is one the walk **accepts**. The tree says why not, and the
reason is honest: at twice the floor a bar's two cap circles come to about one
and a half million boundary points and the triangulator was still running after
two minutes. So the promise is checked from the refusal side and not from the
mesh side, and that is a gap this section names rather than one it hides.

**Three rows say *no test*, and they are one species**: a floor's value written
into a doc comment because somebody measured it once. §56.5 made exactly this
repair for the floor at a distance, and the same repair is available here — a
test that hunts for both floors of one shape at the origin and asserts the pair,
which is cheap because both refusals are immediate. **This section does not make
it**: this lane owns `docs/` only. It is recorded as owed.

**The fourth row is not owed, because the bound it asks for a test on no longer
exists.** It was written as a fourth *no test* and it is a **withdrawn** bound.
`f9d54e0` rewrote `the_finest_deflection_the_walk_will_take`'s doc block and
deleted all three numbers with it — the conic reach, the crossing rounding and
the floor — and that commit sits inside the reach under-charge unit, merged as
`74c14c7` immediately before this section was written. So the census read a tree
that had already moved under it and asked for a test on a number nothing
publishes any more. Deleting the row would hide that; a row that says *withdrawn*
and names the commit does not.

What the block says now is a different claim about a different tree. The wall's
reach is no longer `2` at every lean: `mesh.rs` records `2000001.5` at
`lean = 1e6` and `4000000001.5` at `lean = 2e9`, which is `2 · lean + 1.5`, the
way the ellipse itself climbs. Those two are a *shape* under test rather than two
asserted values — `what_a_leaning_cut_costs_the_floor_under_a_mesh.rs` divides
the floor by that oracle at four leans and asserts the four shares agree — and
the floors that come out of it are asserted to the last digit by
`what_floor_a_bar_with_a_leaning_cap_is_given.rs`.

### 58.6 The new floor is too low on a leaning cut, and this unit did not close it

`walk` charges its room against `Conic::how_far_it_reaches`, the crossing's own
longer semi-axis. Both floors charge `how_far_the_points_of_reach`, the face's
corners plus its surface's radius. **They are not the same number, and the second
is not always the bigger.** A bar of radius one cut by a leaning plane has its
ellipse walked as an edge of the *wall*, whose reach is `2` at every lean and
knows nothing about the plane; the conic's own reach grows with the lean without
bound.

So the fix's promise does not hold on that family, and `mesh.rs` says so in the
same breath as making it. It holds where a crossing is a circle no wider than the
surface it is cut on — every cap circle of a bar, every circle a plane cuts in a
ball — which is what the defect was about. On a leaning cut the change moved
**which refusal arrives** and did not earn an answer:
`TooManyPointsAlongOneCrossing` before, `AWireDoesNotFanIntoALoopToMesh` with
`TooFewCorners { count: 1 }` after, and neither is a mesh.

By §16.10 this is a **gap and not a limit**, and the family of fixes is named
rather than waved at: charge the crossings instead of the corners, and every
conic on the face has to be worked out before there is a deflection to walk one
with — the walk's own job, done twice. Bound the ellipse from the pair of
surfaces that cut it, and the floor stops being a question about one surface,
which is the thing that makes it cheap. Or keep the stand-in away from the
numbers it is wrong about, which is what is claimed today and is a claim about
**who chose the deflection**, not about a face's reach covering its crossings'.
The third is free and the tree takes it. Extending this floor to a caller-named
deflection would turn a floor that does not bind into a wrong answer.

**Everything above this line is the tree as it was, and the gap has since been
closed by the second of those three and not the third.** The reach under-charge
unit, merged as `74c14c7`, made `how_far_the_points_of_reach` bound a pipe's or a
cone's face along its own axis from the pair of surfaces either side of each of
its edges. `f9d54e0` inside it rewrote the doc block this section is reading, so
"whose reach is `2` at every lean" is no longer true of anything: the wall now
reaches `2 · lean + 1.5`. What survives is the narrow claim — a face's reach
still does not cover its crossings' where nothing along an edge bounds where that
edge runs, a cone met by a pipe on its own axis being such an edge. Only the
`Cylinder` and `Cone` arms moved: a ball and a ring are bounded surfaces, so
`Sphere` and `Torus` are still charged from their corners.

### 58.7 The step cap is a gap; the rounding under it is a limit

`DeflectionBelowWhatAMeshCanReach` now has two reasons behind one name and
`error.rs` keeps them apart. The rounding half is a **limit**: no `f64` point
sits closer to the surface, and §18's scaling reaches it exactly. The step-cap
half is a **gap**: the answer exists, `f64` reaches it, and a bigger cap hands it
back. It is deferred because a mesh at the cap is already a million points along
one circle and nobody has asked for two — §13, and the deferral is of a limit's
*neighbour*, so it is written down as owed rather than as impossible.

### 58.8 What a tolerant caller is told where the walk runs out of room

The tolerant lane measured a second thing and fixed none of it, which is the
right order. Where `rcad_geom`'s `room_left_over` runs out, a certified caller is
refused with `CannotPlaceAPointThatCloseToACrossing` carrying both numbers — how
much of the deflection was left to walk the crossing in, and how much the
crossing rounds by — and a tolerant caller is waved through with `Ok(None)`. The
walk then hands back a crossing with no points between its ends, a closed edge's
single corner becomes the whole of its wire, and the mesher answers
`AWireDoesNotFanIntoALoopToMesh { TooFewCorners { count: 1 } }`, which is the
name for a malformed solid.

**Neither caller gets a mesh and the tolerant one is told something untrue about
why.** Twelve cells, twelve of each, and the solids are built certified with an
empty provenance log and pass `validate` certified. `docs/consumers.md` records
a consumer that keeps an unrecognised refusal's debug text verbatim in its bug
reports, so this wording travels: what that consumer files for this is a corner
count and a face key, and not one of the two numbers that say what happened.

That is the third way to reach the four *malformed shape* refusals, and `mesh.rs`
now names all three: a solid that arrived from a file, a fuse that was wrong
about a block corner on a ball's pole, and this — a tolerant policy on a solid
nothing is wrong with.

### 58.9 The import lane, and where it is recorded

The third lane's finding — `Import`'s census reading as complete when styling is
read past uncounted — is §56.6, written by a different session that found the
same defect independently and dropped its own fix on finding this lane's. It is
not restated here. What belongs here is only that the lane is part of this unit
and that its test is in the table in §58.5.

### 58.10 What this does not close

1. **The leaning cut**, §58.6. A gap when this was written, with its family of
   fixes named and none of them taken. The reach under-charge unit, `74c14c7`,
   has since taken the second of them; what is left open is the fallback the
   closing paragraph of §58.6 names.
2. **The tolerant wave-through**, §58.8. Measured over twelve cells and not
   fixed. Any fix closes the window those twelve cells live in, and the file says
   so, so a red there would be the fix and not a regression.
3. **Three published numbers with no test**, §58.5. Owed, and cheap. A fourth row
   was counted here and is withdrawn rather than owed.
4. **The promise checked only from the refusal side**, §58.5. Whether the
   fallback should be held to a mesh that *finishes* as well as to one the walk
   accepts is a separate question, and no section has answered it.
5. **A caller-named deflection is still held to the placing floor and no other**,
   deliberately. Holding it to the step cap as well would make
   `TooManyPointsAlongOneCrossing` unreachable through `mesh`, which is a claim
   about the public API and wants its own demonstration.

