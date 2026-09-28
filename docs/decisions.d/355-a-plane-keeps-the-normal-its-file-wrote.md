## 355. A plane keeps the normal its file wrote, to within what a turn leaves — 2026-09-28

*Q299. Closes the gap §313.3 named: the STEP reader scaled every `DIRECTION`
to length one again, so a normal a turn left an ulp off length one came back
moved. Adds `Plane::from_point_and_normal_as_written` and
`Plane::NORMAL_LENGTH_ROUNDOFF` to `rcad-geom`, and `plane_through` in
`rcad-step` calls the first.*

### 355.1 What was built

- **`Plane::from_point_and_normal_as_written(point, normal)`** stores
  `normal`'s own bits when `n·n` is within `Plane::NORMAL_LENGTH_ROUNDOFF` of
  one, and otherwise scales it exactly as `Plane::from_point_and_normal` does.
  The check is exact: every square is an `Exact` product, and a square too
  small to form exactly (a component below about `2⁻⁴⁸⁴`) is charged at the
  most it could be, `2⁻⁹⁶⁸`, against the top of the bound, so a normal is kept
  only when it is proved inside. The two constructors share one private
  `through`, which checks the point before the normal as
  `from_point_and_normal` always did.
- **`plane_through`** (`crates/rcad-step/src/geometry.rs`) calls it. It is the
  reader's function for every `PLANE` and the writer's check on the point it
  picks, so the two still cannot drift apart.
- **The writer's point** (`a_point_this_reader_reads_back_as`, `write.rs`)
  gained one case. A normal the reader now keeps can face along an axis and
  be an ulp longer than one — `(0, 0, −1.0000000000000002)` is one a turn
  left — and then there is no second component to carry what `offset / nᵢ`
  left over. `stepped_until_it_reads_back` walks that one coordinate an `f64`
  at a time until it reads back. Without it three of the crash files below
  were refused by name, `APlaneWhoseOffsetNoWrittenPointCarries`; for the two
  offsets pinned in
  `a_normal_along_an_axis_an_ulp_too_long_is_written_at_a_point_it_reads_back_from`
  a coordinate two ulps from the quotient reads back, so that refusal was a
  gap and not a limit.

### 355.2 The bound is `64ε` on `|n·n − 1|`

Its doc comment has the argument. A constructor leaves `n·n` within `ε` of one;
`Isometry::POINT_ROUNDOFF` puts a turned component within `18ε` of where the
turn's own exact arithmetic puts it, so the turned vector is off by at most
`√3·18ε` and its squared length by at most `2√3·18ε ≈ 62.35ε`. Together `63.4ε`.

**One share is measured and not derived**: exact Rodrigues arithmetic keeps a
length only when the quaternion's is exactly one, and a stored quaternion comes
from `sin` and `cos`, which Rust does not bound. So
`every_normal_a_turn_leaves_sits_well_inside_the_bound` turns 20 000 normals
and finds the worst `|n·n − 1|` at **10.8ε turned once, 17.7ε through four
composed turns, 42.9ε through sixteen**, and asserts the first two are under
half the bound. Composing quaternions rounds and nothing scales the product
back, so a placement composed of more than about twenty turns can leave the
bound; that normal is then scaled, which is what every normal was before this.

`a_normal_is_kept_up_to_the_published_bound_and_scaled_past_it` is the edge,
on both sides of one, between neighbouring `f64`: `1 + 31ε` is kept and
`1 + 32ε` scaled, `1 − 32ε` kept and `1 − 32.5ε` scaled.

### 355.3 Past the bound it scales rather than refuses

The brief left this open. Scaled, for three reasons:

- **STEP does not ask for a `DIRECTION` of length one.** `(0, 0, 2)` is a
  correct way to write "up", and so is a direction written to twelve digits.
  A refusal would refuse correct files.
- **The only thing a caller could do with the refusal is call
  `from_point_and_normal`.** A refusal every caller turns straight back into
  the other constructor is a branch, not an error.
- **Both answers are the plane the caller named.** Which bits are stored is a
  choice about rounding, not a mistake in the input, so it is not the caller
  input `Err` is for. `Err` stays where it was: a point or a normal that is not
  made of finite numbers, a normal with no length, an offset that overflows.

### 355.4 What moved

**The writer fuzzer's 2026-09-26 crash directory**, 22 528 files, replayed
through the uncommitted replay test §313.1 describes, extended to write one
line per file. "Before" is this branch with `plane_through` put back to
`from_point_and_normal`, uncommitted and restored with `git checkout` after the
run, which is the reader on the base `9612b25f`:

| first panic line, numbers as `N` | before | after |
|---|---|---|
| no panic | 20 967 | 22 336 |
| a different surface: a plane's normal only | 1 559 | 0 |
| a different surface: a cylinder, `Cylinder vs Cylinder` | 0 | 190 |
| the reader refuses: `#N, on the curved face #N, is N off the surface it sits on` | 1 | 1 |
| the reader refuses: `what #N describes is not a solid this kernel can hold` | 1 | 1 |

**The base has 1 559 of these files where §313.4 counted 1 509**: `main` has
moved since, and more files now reach the surface check. Joined file by file,
every one of the 1 559 gets past the plane that stopped it: 1 369 pass
everything, and 190 go on to stop at a pipe's axis (§355.5). No file that
passed before fails now. The first commit alone left 5 of them stopped at a
normal with a component of `4.1e-304` or `6.5e-319` and refused 3 by name;
the second commit closes both (§355.1).

**`just t -p rcad-step`**, run with `--no-capture` before, after the first
commit and after the second, and the printed output compared with the run
bookkeeping taken out: 207 tests passed before and 209 after (two new), and
the same two answers moved both times.

- `the_foot_of_the_perpendicular_misses_on_more_than_a_third_of_these_planes`
  prints **9 271** of 20 000 where it printed 7 777. It reads the planes back
  through `plane_through`, and half of them are turned, so their normals are
  now kept and a different point is dotted with them. It still asserts more
  than a third; the doc comment in `Writing::surface` quotes both numbers.
- `a_turned_pair_read_back_from_rounded_text` prints, for a fuse of the pair
  read back from exact text, **109 decided and `ResultIsNotManifold` 2** where
  it printed 108 and 3. The row for the pair as built is 109 and 2, so the
  read-back pair now fuses the way the pair it was written from does, which is
  what the file's module doc already said it did.

No file in the STEP corpus is named by a moved answer: the corpus tests printed
the same lines both times.

### 355.5 What is left

- **A pipe's axis has the same gap.** The 190 files above stop at a
  `CYLINDRICAL_SURFACE` whose axis direction came back an ulp lower in both
  components it has, `(-0.6888549178837908, 0, -0.7248992358302746)` read as
  `(…907, 0, …745)` on `crash-021605a2eb25c792b85e614ede491bb3c580d599`. The
  reader builds it with `Axis::new`, which scales a direction to length one
  (§120), so the likely cause is the one §313.3 found for planes; it was not
  confirmed by a fix. Closing it needs an `Axis` constructor that keeps a
  direction within the bound, in `crates/rcad-geom/src/point.rs`, outside this
  row's write set, and the reader's cylinder, cone and torus arms in
  `geometry.rs` calling it. **A new row.**
- **Two spellings of one wall.** `from_point_and_normal` makes `(1, 0, 1)`
  and `(s, 0, s)` one plane (§22.6); the new constructor keeps `(s, 0, s)`, so
  a file that writes one wall's normal two ways, both near length one, now
  gets two planes an ulp apart. Nothing in the corpus or the suite showed it.
- **`Model::what_writing_a_plane_down_costs` still needs the normal's share of
  its slack.** It is applied to every model, not only to one read back from
  this writer: every plane the kernel builds still goes through
  `from_point_and_normal` and scales, a turn still leaves its normal up to the
  bound off length one, and another writer's twelve-digit direction is still
  scaled on read. Keeping the normal takes away one rounding on this writer's
  own round trip, and that rounding was never what the slack was sized for. No
  new row.
- `no_direction_reads_back_as_a_normal_a_turn_left_short_of_length_one` in
  `write.rs` stays. Its claim is about `from_point_and_normal` and is still
  true; it is now the demonstration of why the reader does not call it, and it
  asks that constructor by name rather than through `plane_through`.
  `the_reader_keeps_a_normal_a_turn_left_short_of_length_one` beside it is the
  reader keeping the same normal, and the writer's point for it reading back.

Commands: `git log --oneline -1`; `git switch -c
lane-q299-a-plane-that-keeps-its-written-normal 9612b25f`; `mise exec -- just
decision 313`, `22.6`, `121`, `132.2`, `132.6`, `114.3`; `mise exec -- just t
-p rcad-step --no-fail-fast` and the same with `--no-capture`, before and
after; `mise exec -- just t -p rcad-geom --lib plane::tests --no-capture`;
`REPLAY_DIR=… REPLAY_OUT=… mise exec -- just t -p rcad-step --test
zz_throwaway_replay --release replay_a_directory`, three times over the whole
directory (before, after the first commit, after the second) and once over
nine of its files with `REPLAY_DETAIL=1`; `sort` and `join` over the per-file
lines; one `python3` check that a coordinate reads back for the two offsets
above. The replay test was deleted before the commit.
