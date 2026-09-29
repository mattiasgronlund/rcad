## 313. A plane written to STEP comes back an ulp off: the point is the writer's to fix, the normal is `rcad-geom`'s — 2026-09-27

*Q166, from the writer fuzzer's 2026-09-26 night (§265). Amends §249.2 and
§249.6 for one surface: "the writer writes every one of these numbers unchanged
and the reader parses them back exactly" is not true of a plane. Takes back the
claim in `write.rs` that "no point choice closes" a plane's rounding.*

### 313.1 What the fuzzer found

Of the 22 528 files in
`~/rcad-fuzz-logs/2026-09-26/artifacts/no_model_makes_the_writer_panic`, 3 107
stop at `assert_located_numbers_the_same`'s check that a face comes back on the
same surface, and every one of them is a plane:

| what differs | files |
|---|---|
| the offset only | 1 790 |
| the offset and the normal | 897 |
| the normal only | 420 |

Measured by a replay that was not committed: a test file that calls
`one_model_decoded_and_written` (`crates/rcad-step/tests/fuzzing/writer.rs`) on
every file in the directory under `catch_unwind` and counts the first line of
each panic with every number written as `N`, run as
`REPLAY_DIR=<that directory> mise exec -- just t -p rcad-step --test
zz_throwaway_replay --release replay_a_directory`. The split into the three rows
compares the `normal:` and `offset:` text of the two sides.

### 313.2 The offset was the writer's, and is closed

Q166's brief named four legs of a guess. Each was checked:

1. **A `Plane` is a unit normal and an offset, and STEP's `PLANE` is an
   `AXIS2_PLACEMENT_3D`, a point and a direction.** True
   (`crates/rcad-geom/src/plane.rs`, `Writing::surface`).
2. **The writer made a point from the two and the reader made the offset back
   from it.** True. The writer wrote `Plane::origin`, `normal · offset`, which
   rounds all three coordinates; the reader's `Plane::from_point_and_normal`
   dots it with the normal and rounds once. On 20 000 planes that face no axis,
   that point misses the offset on 7 777
   (`the_foot_of_the_perpendicular_misses_on_more_than_a_third_of_these_planes`).
3. **The reader scales a direction to length one again.** True, and it is the
   normal half, §313.3.
4. **The text is exact.** True: `real` writes Rust's shortest text that parses
   back to the same `f64`, which `every_number_this_writer_writes_reads_back_bit_for_bit`
   checks, and once only the point changes no file of the 22 528 stops at an
   offset any more (§313.4), which could not happen if the text rounded.

**What `write.rs` said, and why it was wrong.** It tried two points against 5 000
planes — the foot of the perpendicular, and one coordinate along the normal's
widest component — found both missed about as often, and concluded that
"nothing in this family of fixes is" an improvement and that this was "a limit
of STEP's own encoding of a plane". Both points are the same kind: a point
worked out from the normal and the offset in ordinary arithmetic, with no
thought for how the reader rounds. The kind it did not try is a point built
against the reader's own rounding. A point with two coordinates that are not
zero does it: `offset / nᵢ` along the widest component, which lands within about
an ulp, and what that left over, worked out exactly with one fused
multiply-add, divided by the next widest component. The second coordinate is
tiny, so its own rounding moves the dot product by about `2⁻⁵³` of an ulp, far
inside the interval that rounds to the offset. `a_point_this_reader_reads_back_as`
in `write.rs` has the argument, including why the next widest component is
never smaller than `2⁻²⁷`.

The writer checks the point through `plane_through` in `geometry.rs` — the
function the reader calls for every `PLANE`, so the two cannot drift apart —
and refuses by name, `CannotWrite::APlaneWhoseOffsetNoWrittenPointCarries`,
where the point does not carry the offset. The division-overflow case — a
plane `1.41e308` out — is now written: `the_offset_itself_along_the_widest`
builds it a point the reader reads the offset back from, checked on six
planes up to `f64::MAX`
(`a_plane_too_far_out_to_divide_is_written_at_a_point_it_reads_back_from`). It
remains a gap, and is reached only for offsets below `2⁻⁹⁶⁹` that are not
subnormal, where the reader rounds each product of the dot and drops what it
lost and `divided_along_the_widest` still counts that loss
(`some_planes_just_above_the_subnormals_are_still_refused_by_name`, 164 of
20 000); none of the 22 528 files is refused by it.

On the same 20 000 planes the new point misses none
(`every_plane_is_written_at_a_point_its_offset_reads_back_from`); half of them are
turned after they are built, so their normals sit off length one the way a
placed face's do. The reader did not change, so no file it reads from anyone
else changes either.

### 313.3 The normal is the reader's, and needs `rcad-geom`

A placed face's plane is `Plane::transformed` of the stored one, and a turn
leaves its normal a little off length one: over the 1 317 files on the base
where the normal came back different, `|n|² − 1` runs from −3.7 to 5.4 units of
`2⁻⁵²`. The reader builds every plane with `Plane::from_point_and_normal`, which
scales the direction to length one with every component the nearest `f64` to
the exact ratio. So it gives back `n` only if some vector of length exactly one
lies within half an ulp of each component of `n`.

**For most of these normals no such vector exists**, so no `DIRECTION` a file
could hold reads back as them, and no choice the writer makes can help. Checked
in exact rational arithmetic over each normal's half-ulp box: 1 257 of the
1 317 on the base, and 1 429 of the 1 509 on this branch, have no vector of
length one inside it. `no_direction_reads_back_as_a_normal_a_turn_left_short_of_length_one`
pins one down in whole numbers: `(0.7071067811865475, 0, 0.7071067811865475)`,
both components the `f64` just below `1/√2`, from
`crash-0ac3e625f698733b9b143d41eb65edbf7087b463`, which the reader turns into
the `f64` just above.

**It is a gap, not a limit** (§16.10): the file carries the bits exactly
(§313.2 leg 4), and it is the reader that cannot keep them. It cannot because
every constructor `Plane` has scales the normal again and its fields are
private, so `rcad-step` has no way to build a plane that keeps the normal the
file wrote. What closes it is a constructor in `rcad-geom` that keeps a normal
already of length one to within the rounding a turn leaves — the same promise
`Plane::transformed` makes about the planes it builds — and a reader that calls
it. The reader reads other people's files too, so whoever builds that has to
run `just t -p rcad-step` before and after: a direction another writer wrote to
17 digits, a few ulps from length one, would then keep its own bits and its
import answer would move by those bits.

Considered and not built:

- **The writer searching for a direction the reader turns into `n`.** It exists
  for the other 60 of 1 317 (80 of 1 509). The reader-side constructor closes
  all of them and makes the search pointless.
- **Writing the shell's placement and having the reader rebuild it**, so that
  the turn happens the same way on both sides. That is a feature, and §249.5
  chose to bake the placement in.
- **Scaling a turned normal back to length one when a face is read.**
  `Plane::transformed`'s doc comment and §17.9 are why it does not: the second
  scaling moves the last bit on every read.

### 313.4 The crash directory, before and after

The same replay, on the base `4ac75a74` and on this branch:

| first panic line, numbers as `N` | base | branch |
|---|---|---|
| no panic | 18 309 | 19 905 |
| a different surface: the offset only | 1 790 | 0 |
| a different surface: the offset and the normal | 897 | 0 |
| a different surface: the normal only | 420 | 1 509 |
| the reader refuses: `#N, in the solid #N, is not one this kernel can build` (Q167) | 1 057 | 1 057 |
| does not measure after its own file: `TooSmall` | 41 | 43 |
| the reader refuses: `what #N describes is not a solid this kernel can hold` (Q168) | 13 | 13 |
| the reader refuses: `#N, on the curved face #N, is N off the surface it sits on` (Q169) | 1 | 1 |

The check stops at a solid's first face that differs, so a file whose first
plane is now right can stop at a later plane's normal, which is why the
normal-only row grows past 420 + 897. 1 598 files pass the surface check that
did not; 1 596 of them now pass everything and 2 go on to the `TooSmall`
refusal, which the surface check stopped them short of before. The 2026-09-27
directory, 51 066 files, was not replayed.

The crash file `crash-001e0849a7f44ed74619db24f364721fa6dd90a5` — a round bar
turned about `x`, whose cap came back at `18.06094107536703` for
`18.060941075367033` — is checked in as the writer corpus seed
`a_turned_round_bar_whose_cap_the_foot_of_its_perpendicular_misses`, and a cube
turned a tenth of a radian (`a_cube_turned_a_tenth_of_a_radian_comes_back_on_the_same_planes`)
is the one built by hand beside it. Both fail with the writer's call site put
back to `flat.origin()`.
