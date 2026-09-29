## 363. The room a corner gets is the smaller of where it is stored and where its solid sits — 2026-09-29

*Q646, which replaced Q338 and before it Q334 (§326.3), with Q169 (§327)
folded in. Lane `lane-q646-a-corners-room-wherever-the-solid-sits`, off
`34052a6d`. Every figure below was run in this lane, at that base, with the
command named beside it. The arithmetic in §363.1 against sizing the rooms
from the solid's own size was worked and not run.*

`Model::validate` and `Model::validate_solid` used to measure every room a
corner gets against the largest coordinate among the corners **where they are
stored**. The writer writes each solid **where its placement puts it**, and the
reader sizes the room from there. So a solid moved toward the origin got more
room from the kernel than from the reader, and a model the kernel passed was
written as a file the reader refuses. It happened through both rooms:
`allowed_wobble` on a flat face (`crash-7407c848…`, §326.3) and
`what_writing_a_point_down_costs` on a curved one (`crash-4fa1eb2d…`, §327).

**Now both are measured against the smaller of the two sizes**, where the
corners are stored and where the placement puts them
(`largest_coordinate_in` in `crates/rcad-topo/src/validate.rs`). Every room in
the sweep reads that one number, so the flat room, the curved room,
`not_provably_off` in `check_edge` and the pole check all follow it.

### 363.1 The ways out

The row named two.

- **The writer harness judges each solid where its placement puts it**, through
  a new `rcad-topo` call that copies a solid with its placement worked into its
  corners. `validate` would not change. The harness would agree with the reader
  exactly. But a model `validate` passes would still be written as a file the
  reader refuses. Only the harness would know it.
- **Both rooms stop depending on where the solid sits**, sized from the
  solid's own size instead. That works for the tolerance half of
  `allowed_wobble`. It cannot work for the half that pays for writing a number
  down, and a curved face's room is nothing else (§327.5). A corner written
  down in `f64` is off by the width of its own coordinates, not by the width of
  the solid's size. A bar of radius 2 and height 15 at `x = 10⁶` has corners
  whose `x` rounds by up to half of `2⁻³³`, about `5.8e-11`. A room of
  `16·ε·15 ≈ 5.3e-14` would refuse them, though nothing is wrong with them.
  This is arithmetic, not a run.

The fix taken is the first way's question asked inside `validate`, and only
in the direction that refuses. A corner has two places: where it is stored,
which is where the sweep measures it and where its numbers were written down,
and where its placement puts it, which is where every view, the writer and the
reader find it. How far it is off its surface is the same in both, up to the
rounding in moving it. The room is not the same, because the room grows with
the coordinates. **So a corner now has to fit the room of both places.**

### 363.2 Why this is sound

**The room is never wider than before.** `largest_coordinate_in` returns the
old size or a smaller one, and every room grows with it. So `validate` and
`validate_solid` pass nothing they did not pass before. What changed is only
what they refuse.

**What they now refuse, the reader refuses too.** The reader sizes the room
from where the file puts the solid, which is where the placement puts it. That
size is at least the one the sweep now uses. So a solid the sweep passes gets
at least as much room from the reader, up to the rounding in moving it
(§363.5).

**The placed size alone was tried, and refused.** Measured with only where the
placement puts the corners (`just t -p rcad-topo -p rcad-step --no-fail-fast`,
with `largest_coordinate_in` returning the placed size), two tests fail.

- `a_pole_off_the_pinch_is_refused::the_seed_that_prompted_this_is_refused_by_the_sweep`
  comes back `VolumeIsNotPositive` in place of `VertexOffItsFaceSurface`.
  That fuse seed's ball is placed by a translation of `−5.486e303`. Placed
  there, its room is about `16·ε·5.5e303 ≈ 2e289`, and its corner stored
  `4.7e226` from the ball's centre lands on the same `x` as the centre. A
  placement read from a file could widen the room without limit.
  `check_dynamic_range` reads the largest coordinate where the placement puts
  it (read in the code, not run on this seed), but it is asked last, and under
  a warning policy it only logs (`Model::validate`'s doc).
- `a_solid_moved_away_from_the_origin_gets_no_more_room_than_where_it_is_stored`,
  new here: a cube moved `10⁶` along `x` keeps the size it had at rest.

### 363.3 The fuzzer's night, before and after

All 22 528 files in
`~/rcad-fuzz-logs/2026-09-26/artifacts/no_model_makes_the_writer_panic/` went
through `one_model_decoded_and_written`, in a throwaway test (not committed).
The run marked "before" had `largest_coordinate_in` return the stored size.
The run marked "after" is this change.

| outcome | before | after |
|---|---|---|
| no panic, nothing refused | 22 314 | 22 314 |
| a face came back with a different surface (Q299) | 190 | 190 |
| no panic, counted `written_but_a_solid_unsound_alone` | 22 | 24 |
| `#N, on the curved face #N, is N off the surface it sits on` | 1 | 0 |
| `what #N describes is not a solid this kernel can hold` | 1 | 0 |

The per-file lists differ in two lines, and they are the row's two samples.
The three samples, one by one:

| file | `validate` | refused alone | harness, after |
|---|---|---|---|
| `crash-7407c848…` | Ok | solid 0, `VertexOffItsFacePlane { distance: 3.725290298461914e-9 }` | counted alone, no panic |
| `crash-4fa1eb2d…` | Ok | solid 0, `VertexOffItsFaceSurface` | counted alone, no panic |
| `crash-c8db92e4…` | Ok | solid 1, `VertexOffItsFacePlane { distance: 6.83940015733242e-9 }` | counted alone, no panic, as before |

`crash-c8db92e4…` does not move. Its solid 1 is refused where it is stored,
and moving it away from the origin cannot widen its room now. So the sweep
is stricter than the reader there, and the harness still does not reach the
Q299 difference in that file (§326.5). That costs a round trip nothing. It
is the price of never letting a placement widen a room.

### 363.4 The pins

With the change taken out (`largest_coordinate_in` returning the stored size,
`just t -p rcad-topo -p rcad-step --no-fail-fast`), five tests fail:

- `validate::tests::a_solid_moved_toward_the_origin_gets_the_room_of_where_it_now_sits`,
  new: a block stored at `x` from 1000 to 1002 and moved back by 1000 is sized
  at 2.
- `the_same_bar_moved_toward_the_origin_is_refused_by_validate_as_by_the_reader`,
  §326.3's pin turned round: `validate` and `validate_solid` both refuse the
  bar for `2⁻²⁸`, the reader for the same distance, and the harness counts the
  model as `written_but_unsound`.
- `the_bar_moved_as_the_fuzzer_moved_it_is_refused_by_validate_as_by_the_reader`
  and `the_writer_harness_counts_the_bar_moved_as_the_fuzzer_moved_it_rather_than_panicking`,
  §327.5's two pins turned round.
- `validate_and_the_reader_both_take_the_bar_once_its_top_corner_is_past_11_375`,
  §327.3's pin with `validate` asked beside the reader. Both take the bar moved
  down by 3.5 and both refuse it moved down by 4.

The two controls, `the_bar_where_it_was_built_comes_back_the_same` and
`a_bar_whose_corner_is_a_hair_above_its_cap_comes_back_where_it_was_built`,
pass both ways.

### 363.5 What this leaves

- **The rounding in moving a solid.** The sweep measures a corner off its
  surface where it is stored, and the reader measures it where the writer
  moved it. A turn rounds the corner and the surface both. So a corner within
  a few ulps of the edge of its room can still pass one and not the other.
  Nobody has built one, so it is `refuses/C` at most.
- **The reader's scale.** The model the reader builds carries the scale of its
  own vertices, and `allowed_wobble` takes the smaller of the model's declared
  tolerance and `1e-9·L`. Whether the two scales can differ by enough to split
  a flat corner was not looked at. `refuses/C` at most.
- **A solid built far out and moved toward the origin.** If its corners carry
  rounding from where they were built, and no slop is recorded on them, the
  sweep now refuses it where it sits. The reader already refused it out of the
  writer's own file. Not run, so `refuses/C` at most. Keeping such a solid
  through a file would need the writer to write each solid where it is stored,
  with its placement as a STEP placement, so that the reader judges the
  numbers the sweep judged. That is `rcad-step`'s writer and reader, outside
  this lane.
