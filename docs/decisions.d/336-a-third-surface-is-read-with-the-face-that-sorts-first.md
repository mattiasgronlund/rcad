## 336. A third surface is read with the curved face that sorts first — 2026-09-28

*Queue row Q384, from Q15's answer and §222.1 item 3. Lane
`lane-q384-a-third-surface-read-in-the-surfaces-order`, off `96da3252`.
`rcad_geom::which_two_surfaces_a_run_is_between` read a run's third surface
with the first face listed (or with the ball, for a plane) where both faces are
curved and differ. It now reads it with the face `Surface::how_it_sorts_against`
puts first, whichever face is listed first. **A gap** (§16.10): the answer
existed and the order the caller happened to list the faces in chose another.*

### 336.1 What the order changed

§222.1 item 3 left this open with "nothing has shown it matters". Q15's
read-only lane showed three public answers that moved with the order, and this
lane measured each again on `96da3252`, as a test that fails there:

| shape | listed one way | listed the other way |
|---|---|---|
| a bar flaring into a cone, pipe axis written `-z`, read from a file | refused `AClosedEdgeIsWalkedAgainstItsSurface` | the same, both shell orders |
| the same bar, pipe axis `+z`: its pipe-to-cone rim's `EdgeView::length` | pipe first: `6π`, error `3.55e-15` | cone first: `CannotYetMeasureHowLongThisCurveIs` |
| a half ball on a pipe of its own radius, built through `SolidBuilder` | ball first: measures `20.9777 ± 1.680`, `21.1435 ± 0.455`, `21.2013 ± 0.0330` at `0.05`, `0.0137`, `0.001` | pipe first: `CannotMeshAFace { why: CornersCoincide }` on the ball |

The closed form for the half ball is `πr²(2r/3 + h)` with `r = 1.5`, `h = 2`,
which is `6.75π`, `21.2057504117311`; each measurement holds it within the
error it publishes.

The first row is the one worth reading slowly. `Model::validate` puts first the
face whose coedge it is checking, so the pipe's coedge of the rim was checked
along the pipe's circle, winding by the pipe's axis, and the cone's along the
cone's. Written with the pipe's axis down and the cone's up, the two cannot both
pass, and no shell order helps, because the order validate uses is its own.

### 336.2 Why that order

The order is plane, ball, pipe, cone, torus, then the stored numbers. It is the
order that already answers the neighbouring question, and it is the right one
for each consumer of this pair:

1. **The ball first gets its poles into the rim.** The mesher reads this pair
   for `put_in_the_poles_a_closed_crossing_passes`; a pair that names the pipe
   has no poles, which is the half ball's `CornersCoincide`.
2. **A pipe before a cone or a torus** is the pair `rcad-topo`'s edge length has
   a circle for (`Circle::where_they_cross`); a cone's is `None`.
3. **It is the surface the direction rule picks.** `rcad-topo`'s
   `validate.rs` and `rcad-step`'s `brep.rs` each have a `the_curved_one_of`,
   and `mesh.rs` has `the_curved_surface_that_names_a_closed_edge`; all three
   take the curved surface that sorts first. With this change the curve a
   closed edge is walked along and the surface that says which way it runs are
   one surface.

The helper is `walk.rs`'s `the_face_that_sorts_first`, and one arm now covers
both run kinds between two curved faces, where there were three.

### 336.3 What still depends on the order

**Where a face is flat, `along[0]` still stands.** Q15's lane reported that
changing the both-flat arm the same way broke two pinned answers,
`a_closed_run_between_two_flat_faces_names_a_face_and_its_curved_second_surface`
and the `AClosedRunCarriesItsSecondSurface` arm of
`every_spelling_of_one_surface_names_the_same_crossing`; this lane did not
re-run that. Both pin the first of two planes a hair apart. The mixed arm, a
flat face and a curved face with a third surface, was not tried; nothing in the
kernel or the reader writes that run. Neither has a shape measured against it.

**Two different balls on one rim** was not probed. The order picks one of them,
and only that ball's poles go in.

### 336.4 What the every-spelling property now says

`every_spelling_of_one_surface_names_the_same_crossing.rs` pinned `along[0]`:
the pair `[once, twice]` had to read as `[once, once]` does. A curved pair is
now read with whichever spelling sorts first, which is sometimes `twice`. The
property is restated: the pair is read with one of the two spellings, a curved
pair reads the same with its spellings listed either way round, and the kernel's
answer is compared against that spelling written once. Every draw is the same
as before; the controls that named a start moved from 899, 250, 128 to 915,
256, 125 in the three arms a curved pair reaches, all over their floors.

### 336.5 The guard Q41 needs

`a_flared_bar_reads_whichever_way_its_pipe_points.rs` also pins the union of two
round bars of radius 3 on one axis, `z` 0 to 4 and 2 to 6, read from one file
with their pipe axes written opposite ways: one solid, validates, and measures
`54π` within its error. It passes on the base and with this change. It is there
because Q15 reported Q41's change alone breaking it. The same pair's
intersection and difference refuse `TheFuseWroteABrokenAnswer
{ CoedgesRunTheSameWay }` on the base and with this change, measured by Q15's
probe run on this lane's tree both ways; that is row Q385.
