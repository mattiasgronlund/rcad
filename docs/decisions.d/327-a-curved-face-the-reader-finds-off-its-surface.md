## 327. A curved face the reader finds off its surface: a curved face's room follows where the solid sits too — 2026-09-28

*Q169, from the writer fuzzer's 2026-09-26 night (§265). Lane
`lane-q169-curved-face-off-its-surface`, off `0a7142ff`. The lane diagnosed the
file and pinned it, and stopped at its budget before writing this section; its
merger wrote it from the lane's report. **Neither the writer nor the reader is
wrong.** This is §326.3's gap, the room a corner gets depending on where its
solid sits, reached through a curved face's room,
`what_writing_a_point_down_costs`, and not through `allowed_wobble`. So the
second of §326.3's two fixes would not reach this file.*

*What was checked by whom: the arithmetic (every room, the miss and 11.375) was
worked again by the lane's verifier and matches. That `allowed_wobble` never
reaches a curved face the verifier read in `validate.rs`. The table's `validate`
and `validate_solid` rows, the scan in §327.3 and the two mutations in §327.4
come from the lane's throwaway probe and scratch copy and were not run again.
The committed pins re-check the miss to the bit and the two sides of 11.375.*

### 327.1 The file

`crash-4fa1eb2d42f7b22c3ff6455d6ae0de9bb736a93e`, 1 067 bytes, holds two solids.

- **Solid 0 is a round bar 15 tall**, with its corners at `(2, 0, 0)` and
  `(2, 0, 15)`. Its side is a pipe round `z` of radius `2 + 91·2⁻⁵¹ =
  2.0000000000000404` (bits `0x400000000000005b`), so both corners sit
  `91·2⁻⁵¹ ≈ 4.041212e-14` inside the wall. Its placement moves it by
  `(0, −1, −10)`.
- **Solid 1 plays no part.** It is a bar of radius 3, 16 tall, turned by a
  quaternion whose `x` is `5.4e-311`. The bar built alone reproduces the
  refusal with the same entity numbers.

The reader refuses corner `#15`, at `(2, −1, −10)`, on the curved face `#51`:
`ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn { off_by: 4.04121180963557e-14 }`.
The writer harness then panics with "a sound model was written as a file the
reader refuses".

### 327.2 The writer is not at fault

The move is exact. The file's `#6` to `#53`, which are the bar, are the same 48
lines as the text of the bar built alone through the public API and moved the
same way. The reader's `off_by` is `91·2⁻⁵¹` to the bit, and
`the_bar_moved_as_the_fuzzer_moved_it_passes_validate_and_the_reader_refuses_it`
asserts that.

### 327.3 The room

The reader builds each solid through `SolidBuilder::build`, which judges it in a
model of its own (§326.1). There `check_faces` hands `check_curved_face` a room
of `what_writing_a_point_down_costs(L)`, which is `MEASURING_SLACK ·
NEAREST_F64_POINT_ROUNDOFF · L = 16·ε·L`. The corner is widened into a box of
that half-width, and it is refused when the box is off the surface for certain
(`not_provably_off`). `L` is `largest_coordinate_in`, read where the corners are
stored. **`allowed_wobble` never reaches a curved face**: `check_face` hands
`check_curved_face` the writing-down room alone, and the doc comment of
`what_writing_a_point_down_costs` says it leaves `allowed_wobble` out on
purpose, because a corner may sit off a curved surface only by the width of the
number.

| check | `L` | room `16·ε·L` | result |
|---|---|---|---|
| `validate`, the whole model | 16, solid 1's top corner | `5.684342e-14` | Ok |
| `validate_solid`, solid 0 where it is stored | 15 | `5.329071e-14` | Ok |
| the reader, solid 0 where the file puts it | 10 | `3.552714e-14` | refused |

The miss of `4.041212e-14` fits once `L ≥ 91·2⁻⁵¹ / (16·2⁻⁵²) = 11.375`. Moved
down `z` by 0 to 15 in steps of 0.5, the reader takes the bar when its top
corner is at 11.5 or more and refuses it at 11 or less, while `validate` and
`validate_solid` pass all 31. `the_reader_takes_the_bar_once_its_top_corner_is_past_11_375`
pins the two sides of that edge, moved down by 3.5 and by 4.

### 327.4 Which room it is, shown by mutation

§326.3's row rested on two legs: (a) the reader and the kernel's checks judge the
same bar, and (b) the room follows the stored `L` through `allowed_wobble`.
**(a) holds. (b) holds in that the room follows the stored `L`, and not in that
it goes through `allowed_wobble`.** Two mutations of `check_faces`, each in a
scratch copy of the base:

- sizing only the curved room, `writing_a_point_down`, from how far apart the
  corners are rather than how far they are from the origin makes 3 of the 4
  pins fail, and the unmoved control still passes;
- sizing only `allowed_wobble` that way moves none of them.

### 327.5 The fix, and which pins turn

§326.3 named two fixes. Its second, sizing `allowed_wobble`'s tolerance half
from the solid's own size while the writing-down part keeps growing with the
coordinates, **does not reach this file**: a curved face's room
is nothing but a writing-down part. Two fixes do reach it.

- **The harness judges each solid where its placement puts it**, §326.3's first.
- **The curved room also stops depending on where the solid sits.** That is
  every caller of `what_writing_a_point_down_costs` in `validate.rs`, among
  them `check_curved_face`, `not_provably_off` in `check_edge` and the pole
  check. `check_curved_face` is the one that refuses this file.

This is a gap, not a limit: the answer exists, and the kernel gives it for the
same bar where it is stored. Under the second fix,
`the_bar_moved_as_the_fuzzer_moved_it_passes_validate_and_the_reader_refuses_it`
and `the_reader_takes_the_bar_once_its_top_corner_is_past_11_375` fail.
`the_writer_harness_panics_on_the_bar_moved_as_the_fuzzer_moved_it` fails under
either. `the_bar_where_it_was_built_comes_back_the_same` is the control and
passes under both.

### 327.6 What this leaves

- **No row of its own.** The lane asked that Q169 be folded into Q334, §326.3's
  row, and no queue verb changes a row's text or write set (Q293). So the merge
  dropped Q334 and added Q338 in its place: Q334's text, with this file as a
  second sample, the curved room as a part the fix must reach, and
  `a_curved_face_the_reader_finds_off_its_surface.rs` in its write set, since
  the harness fix turns one of its pins.
