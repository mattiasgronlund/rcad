## 326. The writer harness judges each solid alone, as the reader does, and the room a corner gets depends on where its solid sits — 2026-09-28

*Q168, from the writer fuzzer's 2026-09-26 night (§265). Lane
`lane-q168-judge-each-solid-alone`, off `cdd0cb52`, the second pass at the row;
the first diagnosed it and stopped at its budget with no commit, and every
finding of it below was run again on this base. **Neither the writer nor the
reader is wrong.** The harness called a model sound when `Model::validate`
passed it, and the reader asks each solid a different question. Twelve of the 13
files are that and nothing else, and the harness now asks the reader's question
too. The thirteenth is a second cause the harness cannot answer from its own
file, and it is a queue row.*

### 326.1 Two causes, one refusal

Under the reader's refusal, "what #N describes is not a solid this kernel can
hold", all 13 files carry the same `rcad_topo::Error`:
`Invalid(VertexOffItsFacePlane { distance })`, with the distance between
`1.6e-9` and `1.5e-8`. The reader builds each solid it reads through
`SolidBuilder::build`, and that judges the new solid in a model of its own,
which is what `Model::validate_solid` answers (§242.6). `validate` judges the
whole model instead, and §242.6 already records that it can pass a model whose
solid `validate_solid` refuses, because a far neighbour widens the room
(`a_corner_the_far_solid_makes_room_for_is_refused_on_its_own`).

The room is `allowed_wobble` in `crates/rcad-topo/src/validate.rs`: the smaller
of the model's own tolerance and `1e-9 · L`, plus `16 · 4ε · L` for writing the
plane down, where `L` is the largest coordinate of any stored corner the check
looks at. For a model of 100 mm that is about `1.00001e-9 · L`.

A throwaway test (not committed) ran each file through `validate`, through
`validate_solid` on every solid, and through the writer and the reader:

- **12 files: `validate_solid` refuses exactly the solid the reader refuses**,
  by its place in the file, and with the same distance to the last digit. Every
  one passes `validate`. Sample `crash-9018206a0ba623e51da13398cf5d9e8eb08424b5`,
  1 724 bytes: a unit cube whose `+y` face sits on the plane at `1 + 2⁻²⁹`,
  beside a round bar of radius 2 and height 15. The cube's corners are `2⁻²⁹ ≈
  1.862645e-9` off that plane. The model's `L` of 15 gives about `1.5e-8` of
  room and the cube's own `L` of 1 gives `1.000014e-9`.
- **1 file: every solid passes `validate_solid`.**
  `crash-7407c84648fd0b490eed86a4b495cdd76d2e5ec2`, whose solid 0 is §326.3's.

### 326.2 The fix, and what fails without it

`read_back_and_compared` in `crates/rcad-step/tests/fuzzing/writer.rs` now asks
`validate_solid` of every solid once `validate` has passed, and counts a model
with a solid refused alone as `written_but_a_solid_unsound_alone`. It does not
panic on such a model. It stops there, as it does for `written_but_unsound`: a
file of it owes the reader nothing.

`crash-9018206a…` is checked in as
`fuzz/corpus/no_model_makes_the_writer_panic/a_unit_cube_a_hair_off_its_plane_beside_a_tall_bar`,
and `crates/rcad-step/tests/a_solid_the_writer_harness_judges_alone.rs` asks of
it directly that `validate` passes, that `validate_solid` refuses the cube with
`VertexOffItsFacePlane` and passes the bar, and that the harness counts it
`(written, written_but_a_solid_unsound_alone, read_back) = (1, 1, 0)`. With the
new check taken out and the seed and tests kept, four tests fail, each at the
harness's "a sound model was written as a file the reader refuses":
`every_seed_in_the_checked_in_corpus_is_written_without_panicking`,
`a_curved_solid_in_the_corpus_reaches_the_comparison`,
`at_least_ten_seeds_decode_into_a_model_with_a_face` and
`a_cube_a_hair_off_its_plane_is_counted_rather_than_failed`.

### 326.3 The room depends on where the solid sits

`L` is read off the corners where they are stored, before any placement
(`largest_coordinate_in`, whose doc says so). The writer writes each solid where
its placement puts it, and the reader builds it there. So the same solid gets
one room from `validate_solid` and another from the reader, and **moving a solid
toward the origin shrinks the room the reader gives it**.

In `crash-7407c848…`, solid 0 is a round bar of radius 4, 4 tall, whose top
corner sits `2⁻²⁸ ≈ 3.725290e-9` above its cap at 4. Measured per solid, at rest
and placed:

| | largest coordinate | room | corner off its cap |
|---|---|---|---|
| stored | `4.0000000037252903` | `4.000057e-9` | `3.725290e-9` |
| placed by `(−4, −3, −2)` | `3` | `3.000043e-9` | `3.725290e-9` |

The corner is inside the room at rest and outside it where the file puts it, so
both of the kernel's checks pass the solid and the reader refuses it. The
harness still panics on this file.
`the_same_bar_moved_toward_the_origin_is_refused_by_the_reader` builds that bar
from nothing, alone in its model. It checks that `validate` and
`validate_solid` pass it, and that the reader refuses it with
`VertexOffItsFacePlane { distance: 2⁻²⁸ }`. Beside it,
`a_bar_whose_corner_is_a_hair_above_its_cap_comes_back_where_it_was_built`
writes the same bar unmoved, and it comes back the same. So no neighbour is
involved. The only difference between the two is where the bar sits.

**It goes the other way too, and the new check is then stricter than the
reader.** In `crash-c8db92e4b4c5e65c0d47ab92ed0bf57d56b08fad`, solid 1 is a round
bar whose bottom corner is `6.8394e-9` off its cap. Its stored `L` of 4 gives
`4.000057e-9` of room, so `validate_solid` refuses it. Placed by
`(−11, 4, 14)`, its `L` is 16 and its room `1.600023e-8`, so the reader accepts
it. Before this change the harness went on to the comparison and panicked on
Q299's normal-only difference, in another solid. Now it stops at the new check.

**This is a gap, not a limit.** The answer the harness wants exists: whether
the reader will accept each solid where the file puts it. Two fixes reach it,
and neither is in this lane's write set:

- **The harness judges each solid where the file puts it.** That needs a model
  holding the solid with its placement worked into its corners, and
  `rcad-topo` has no public call that makes one. The only public copy,
  `Model::copy_solid`, keeps the placement as a placement. So the harness would
  have to rebuild the solid by hand, which is a second reader.
- **`rcad-topo` makes the room independent of the frame**, by sizing the
  tolerance half of `allowed_wobble` from the solid's own size rather than from
  its distance to the origin. The half that pays for writing the plane down
  does grow with the coordinates, and would stay that way.

The second changes what `validate` accepts, so it is a decision with an argument
still to have. It goes back to the queue as a row, with `crash-7407c848…` as its
sample.

**Closed by §363 (Q646), by neither of the two.** Every room is now measured
against the smaller of where the corners are stored and where the placement
puts them, so a placement can narrow a room and never widen it. `validate` and
`validate_solid` refuse this bar moved by `(−4, −3, −2)` for the same `2⁻²⁸`
the reader does, and the harness counts `crash-7407c848…` as a solid unsound
alone. The pin above was turned round and renamed
`the_same_bar_moved_toward_the_origin_is_refused_by_validate_as_by_the_reader`.
`crash-c8db92e4…` does not move: its solid is still refused where it is stored
(§363.3).

### 326.4 The whole directory, before and after

All 22 528 files in `~/rcad-fuzz-logs/2026-09-26/artifacts/no_model_makes_the_writer_panic/`
were replayed through the harness twice, with the throwaway test's
`replay_a_directory`. The first run had the new check taken out and the second
had it in. The runs took 107 s and 95 s. Rows with numbers written as `N`:

| row | before | after |
|---|---|---|
| no panic | 19 905 | 19 918 |
| a face came back with a different surface, plane, normal only (Q299) | 1 509 | 1 508 |
| `#N, in the solid #N, is not one this kernel can build` (Q79, §320) | 1 057 | 1 057 |
| a solid that measured does not measure after its own file, `TooSmall` | 43 | 43 |
| what #N describes is not a solid this kernel can hold (this row) | 13 | 1 |
| `#N, on the curved face #N, is N off the surface it sits on` (Q169) | 1 | 1 |

The per-file lists match on all 22 528 paths, and 13 files changed outcome. The
new check takes 12 from this row, the 12 of §326.1. It takes 1 from Q299's row,
`crash-c8db92e4…` of §326.3. It takes none from any other row.

### 326.5 What this leaves

- **A queue row for §326.3**: the reader refuses a solid that `validate_solid`
  passes, because the room is sized from where the corners are stored and the
  file writes them where the placement puts them. Sample `crash-7407c848…`.
  `the_same_bar_moved_toward_the_origin_is_refused_by_the_reader` fails the
  day it closes.
- Q299's row is one sample lighter. `crash-c8db92e4…` still has its
  normal-only difference, but the harness no longer reaches it.
