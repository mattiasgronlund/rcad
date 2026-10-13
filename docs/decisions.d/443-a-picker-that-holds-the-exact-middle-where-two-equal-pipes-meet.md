## 443. A picker that holds the exact middle where two equal pipes meet — 2026-10-13

*Q1892, filed again from Q1662, which replaced Q1657. §31.9's Q1645 note
showed that bores on `(3, 4, 0)` and `(4, 0, 3)` through `(11, −3.25, 0.5)`
have no flat surface written in `f64` that picks their halves, and named the
third of Q1625's three families of fix as the one that reaches them: a picker
that holds the exact middle and a facing, rather than a rounded offset. This
section says what that picker is, where it is used, and how it is written
out.*

In short: a new arm, `Picker::InFrontOfAFlatThrough`, holds a point and a
facing as the caller wrote them, and picks the points `p` with
`facing · (p − through) ≥ 0`, asked exactly. The namer of four half-ellipses
takes it only for a surface where every facing it tries misses the middle, so
every pair it named before keeps its pickers bit for bit. A model writes it out
as a fifth tag and six numbers, and `FORMAT_VERSION` stays 3.

  443.1   What the picker is
  443.2   Why a new arm and not a change to `HalfSpace`
  443.3   Where the namer takes it
  443.4   How it is written out
  443.5   Who else reads it

### 443.1 What the picker is

`HalfSpaceThroughAPoint` holds `through` and `facing`, each three `f64`s,
untouched. Its side of a point `p` is the sign of
`f · p − f · c`, six products, through the same three levels every predicate
here runs: a float filter with its error bound, intervals, and an exact
expansion. Nothing about it rounds before the sign is asked, so its flat
surface passes exactly through `through`. That is the property a `HalfSpace`
built by `through_a_point_facing` has only when `f64` holds `f · c`.

Reversing it negates `facing`, which rounds nothing, so the two corners on
either side of one surface are exact opposites, as two reversed half-spaces
are. Moving it moves `through` as a point and turns `facing` as a vector. A
turn rounds both, as it rounds a half-space's normal, and reversing still
commutes with moving because negating a vector negates every product of the
turn exactly.

### 443.2 Why a new arm and not a change to `HalfSpace`

A `HalfSpace` is its four numbers (§32.3). It is written out as four, and the
note on the type says those four are the caller's, bit for bit. Giving it a
fifth number, or a point in place of the offset, changes every picker every
model holds and the bytes of every file written. A rounded offset is also the
right thing for the half-spaces that have one: §150's pair at `1e16` is why it
is rounded from the exact dot. The gap is narrow, a flat surface that must hold
one point exactly where `f64` cannot hold its offset, so the fix is a picker
for that case and nothing else.

### 443.3 Where the namer takes it

`four_half_ellipses_where_two_pipes_of_one_width_meet` asks
`a_flat_surface_through_where_the_walls_touch` for each surface, as before.
Where that finds a half-space, it is kept. Where it finds none, the surface
becomes the new picker through the middle `c`, facing the pipe's own stored
direction. That direction lies in the flat of the two directions, since it is
one of them, and it leans towards its own axis. Both are asked exactly, by the
same two checks every other facing passes. So the two ellipses' meeting points
`c ± r·n̂` lie exactly on both surfaces, which is all §31.9's Q1625 note asks of
them, and the four corners each hold one half.

The slide, the sums and the lowest terms stay. They are not needed for
correctness any more, but a pair they name keeps a `HalfSpace`, which every
reader of a picker already knows, and the pins on them do not move.

### 443.4 How it is written out

**`serialize.rs`.** `Picker` is written by serde's derive through postcard: a
tag for the arm, then the arm's numbers. The new arm is declared last, so it is
tag 4 and the four arms before it keep tags 0 to 3. Its payload is
`(through, facing)`, six numbers, read back through the same checks `new`
makes, so a file holding a zero or non-finite facing is refused rather than
read. `FORMAT_VERSION` stays 3. The version moved twice (§24.5, §33) because
old bytes would have parsed at the new shape into a different solid. Here they
cannot: every file written before parses exactly as it did, and a file holding
the new arm, read by code from before it, fails on an unknown tag, which is an
error and not a different solid.

**The STEP writer** writes no picker. A STEP file holds a curve and the points
that trim it, and `write.rs` asks a run's pickers only which side a point is on,
through `Picker::side_of_point` (`what_the_pickers_make_of`), which the new arm
answers exactly. The one place it reads a picker's own numbers is
`the_half_circle_from_pole_to_pole`, for a ball cut by a flat face with one
`InFrontOf` picker, which this picker never is; anything else there is `None`
and the circle is written another way. A file read back names the crossing
again through `intersect`, so it gets the new picker from the namer itself.

### 443.5 Who else reads it

- **The walk** (`walk.rs`). `which_side_of` chooses how a side reading is
  measured, and the new arm is measured as a length, as a half-space is.
  Everything else in the walk asks `Picker::side_of_point`.
- **The arrangement** (`fuse/arrange.rs`). `a_half_of_an_ellipse`, and the
  places that find where two halves meet and the middle of each, read a flat
  picker's facing and where an axis crosses it. For the new arm the facing is
  `facing` and the axes cross it at `through`, exactly.
- **`validate.rs`** asks every picker of a crossing which side each end of
  the edge is on, through `Picker::side_of_point`, and refuses an end behind
  one as `EdgeOutsideWhatPickedIt`; the new arm answers that as a half-space
  does (`an_edge_outside_a_flat_surface_holding_its_point_that_picked_it_is_caught`).
  `what_a_corner_past_the_picker_says` keeps its `let … else` on `InFrontOf`
  alone (§190.1). It reads the one picker that names a whole circle, through
  `the_picker_naming_the_circle_through`, which takes a run picked by exactly
  one picker, and `intersect` builds the new arm in one place only, as one of
  the pair that picks a half ellipse.
- **Matches outside `rcad-geom`** that sent the new arm to a wildcard
  (`holds.rs`, `fuse/assemble.rs`) name it in the arm the wildcard gave it, so
  nothing they answer moves (§40): `which_side_of_a_rings_circle` refuses,
  `how_a_side_runs_round` and `the_way_into_an_arc` find nothing, and
  `what_the_disk_of_a_rings_circle_holds` and `which_curve_of_the_pair` panic.
  Each reads the one picker of a closed curve or of an arc, which `intersect`
  never makes the new arm. That is a reading of the code and not a
  demonstration: a model built by hand, or read from a file, can put the new
  arm there, and what it gets then is untried.
- **Two test files did the same.** `the_one_flat_picker` in
  `where_two_leaning_pipes_far_out_cross.rs` refuses the new arm as it refuses
  every picker but `InFrontOf`. `Picking::of` in `tests/fuzzing/comparison.rs`
  panics on it, as its wildcard did: a `Picking` of its own for it needs an arm
  in `Picking::moved_by` in `fuses_that_must_agree.rs` too, since that match
  names every `Picking`, and that file was outside this row. So a fuse whose
  answer holds the new arm cannot yet be compared. None that answers is known:
  Q1896 and Q1897, the two pairs of bars crossing at `(11, −3.25, 0.5)`, say
  each of their fuses is refused.
