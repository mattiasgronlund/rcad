## 321. The triangulator takes a corner under its floor as zero, where that changes nothing — 2026-09-28

*Queue row Q79, from §246.5 and §246.7. Lane `lane-q79-flush-tiny-corners`, off
`466634ac`. **`triangulate` now hands spade each scaled corner through
`spade::mitigate_underflow`**, which sets a coordinate under `2⁻¹⁴²` to zero,
and then checks that the triangles spade drew are still a tiling of the patch
that was asked for. Where they are not, it gives the refusal it gave before
the flush. On the writer fuzzer's 2026-09-26 directory, Q167's row goes from
1 057 to 0 and Q172's from 43 to 0, which is §320.4's bare-flush column
exactly. Q167 and Q172 close with this.*

### 321.1 What changed

In `crates/rcad-mesh/src/triangulate.rs`:

- **The flush.** Every corner goes to spade through `mitigate_underflow`.
  Where no corner has a scaled coordinate in `(0, 2⁻¹⁴²)`, spade gets the
  numbers it always got, so every patch that meshed before meshes the same,
  triangle for triangle.
- **The check, only where the flush moved a corner.** The patch is read
  again with every corner where the caller put it. It is refused unless:
  - no two corners in different places were given one vertex;
  - every side of every loop is one side of a triangle;
  - every triangle turns anticlockwise;
  - the hull's outline does not cross or touch itself.

  Only triangles and hull sides with a moved corner are asked.
- **The refusal when it fails.** It is `CoordinateOutOfRange { position, why:
  TooSmall }` at the first corner the flush moved, which is what spade said
  before. So the flush only ever turns a refusal into a mesh. It never turns
  a mesh into a refusal, and it leaves a refusal's words alone.

Why the checks are enough is in the doc comment on
`the_flush_moved_nothing_the_patch_rests_on`. In short: with every triangle
anticlockwise, the number of triangles covering a point equals the number of
times the hull's outline winds round it. An outline that does not cross itself
winds once or not at all, so the triangles tile the hull, and the loops are
sides of that tiling.

### 321.2 The four things a flushed corner can do, each shown

§246.5 asked for a demonstration, not a paragraph, that a flushed corner
cannot turn a triangle over or leave two patches drawing an edge differently.
It can do both. Each test below builds one case and fails without the check
that catches it. All are in `crates/rcad-mesh/tests/cutting_a_patch_into_triangles.rs`.

| what the flush does | test | caught by |
|---|---|---|
| carries a corner across the line of a thin triangle, turning it over | `a_triangle_the_flush_turns_over_is_refused` | a triangle turning the wrong way |
| turns a thin hole inside out, so the kept triangles either side of it overlap | `a_hole_the_flush_turns_inside_out_is_refused` | the triangle inside the hole, which is not kept, turning the wrong way |
| puts a corner of one hole on a side of another, which spade then splits: a crack against the next face | `a_corner_flushed_onto_another_loops_side_is_refused` | a side missing |
| puts two corners `2⁻¹⁵⁰` apart in one place, tying two holes to one point | `two_corners_the_flush_puts_in_one_place_are_refused` | two places on one vertex |

In the hole and the split side, every kept triangle still turns the right way.
The hole needs its own non-kept triangle checked, and the split side needs the
side check. In the split side, the corner is not on the hull either.

**What stays unshown:** no test here was built to fail only the hull-outline
check. It is there because the argument in 321.1 needs it. That it cannot be
reached is not claimed.

`finish` in `crates/rcad-topo/src/mesh.rs` is not touched. The triangulator
refuses all four before a patch reaches it.

### 321.3 The first version of the check refused 95 files it should not have

The first check asked for the hull to stay **convex**. A corner flushed onto a
straight side of the hull dents the hull inwards by less than `2⁻¹⁴²`, read
where the corner really is, while the triangles still tile the patch. On the
directory, that version (`b9610a3c`) left 96 files on Q172's row (`TooSmall`)
and 20 859 with no panic. The final version leaves 0 and 20 954, and §320.4
measured 0 for the bare flush. One of them, `crash-00ae3e003b173e587744bf72c9b0a09193a85988`,
is a bar of radius 5 turned `5.6e-309` radians about `x`. A temporary print in
the check showed where it failed: a hull side from `(15.39, 0)` to
`(15.60, 0)`, and the point `(8.01, 5e-324)`. Two things failed at that point:

- **`orient2d` left that turn undecided.** Its determinant is about
  `0.2 · 2⁻¹⁰⁷⁴`, under the smallest `f64`. `orient2d` does retry at a power of
  two, but it picks the power from the largest difference between the points,
  which here is about one. So it retries at the scale it started at. A single
  scaling that lifts every product into range does exist, and it is what
  `which_way_three_points_turn` in `triangulate.rs` now does: it moves every
  coordinate up by one exact power of two until the largest is near `2⁵⁰⁰`.
  `a_corner_a_subnormal_inside_a_straight_side_of_the_hull_is_meshed` uses
  those three numbers, and asserts that `orient2d` alone still leaves them
  undecided.
- **The dent itself.** A convexity check refuses it. A check that asks only
  that the outline not cross itself lets it through, and that is all the
  argument needs. `a_corner_flushed_onto_a_straight_side_of_the_hull_is_meshed`
  pins it.

With both changes, the directory replays as the bare flush did (321.4).

### 321.4 The replay

`§313.1`'s replay test with `§320`'s additions, copied into
`crates/rcad-step/tests/` uncommitted, over all 22 528 files of
`~/rcad-fuzz-logs/2026-09-26/artifacts/no_model_makes_the_writer_panic`. The
base is `466634ac`, exported with `git archive` into the scratchpad and built
there. The tip is `8a0f3068`. First panic line, numbers as `N`:

| first panic line | `466634ac` | `8a0f3068` |
|---|---|---|
| no panic | 19 905 | 20 954 |
| a different surface: the normal only (Q299) | 1 509 | 1 560 |
| the reader refuses: `#N, in the solid #N, is not one this kernel can build` (Q167) | 1 057 | 0 |
| does not measure after its own file: `TooSmall` (Q172) | 43 | 0 |
| the reader refuses: `what #N describes is not a solid this kernel can hold` (Q168) | 13 | 13 |
| the reader refuses: `#N, on the curved face #N, is N off the surface it sits on` (Q169) | 1 | 1 |

The base column is §320.4's `b5325a75` column, row for row. File by file, the
only files that change row are these:

```text
   1007  Q167's row  ->  no panic
     50  Q167's row  ->  a different surface (Q299)
     42  Q172's row  ->  no panic
      1  Q172's row  ->  a different surface (Q299)
```

No file got worse. The 50 + 1 that move on to the normal row belong to Q299
(§313.3).

### 321.5 What is pinned, and the seeds

- `a_disc_moved_by_two_to_the_minus_1007_is_cut_as_the_disc_that_was_not_moved`
  (rcad-mesh) is §246's cap. Its triangles are the unmoved disc's, index for
  index.
- `a_round_bar_moved_by_less_than_two_to_the_minus_142_comes_back_the_same`
  (rcad-step) moves a bar of radius 2 along `y` by `±2⁻ᵏ` for
  `k = 100, 141, 142, 143, 200, 1007, 1022`, and by the subnormals `2⁻¹⁰²³`
  and `2⁻¹⁰⁷⁴`. It writes each bar, reads it back and validates it. The corners
  come back the same bit for bit, and so do volume, area and centroid. It does
  not use `the_same_solid_comes_back`, because that one also fuses, and a fuse
  with this moved bar refuses for reasons of its own (§246.6).
- Three seeds in `fuzz/corpus/no_model_makes_the_writer_panic/`:
  - `a_round_bar_moved_by_two_to_the_minus_1007` is §246's seed
    `1bf987c1…` with byte 506 (counting from zero) set to `0x01`.
  - `a_round_bar_moved_by_a_subnormal_y` is §320.6's
    `crash-00f8c3b4…`: a bar of radius 4 placed at `(7.7e-304, 8.3e-317, 0)`.
  - `a_round_bar_turned_6_4e_minus_30_radians_about_x` is §320.6's
    `crash-0cd61ccf…`.

Run on the base with these files copied in, seven tests fail: the disc, the
two hull-dent tests, the moved bar, and three tests in
`every_model_the_writer_fuzzer_kept_is_written_again.rs`. The three are
`every_seed_in_the_checked_in_corpus_is_written_without_panicking`,
`a_curved_solid_in_the_corpus_reaches_the_comparison` and
`at_least_ten_seeds_decode_into_a_model_with_a_face`. The four tests of 321.2
pass on the base, and should. They assert the refusal the base already gives,
so they guard the checks and not the flush.

### 321.6 What this leaves

- **Q167 and Q172 close** with Q79: both rows are at 0 on the tip.
- **A fuse pinned as refused now answers, and answers right.**
  `a_wedge_cut_by_one_prism_whose_edge_is_the_polar_axis_is_answered_or_refused_by_name`
  in `crates/rcad-topo/tests/a_narrow_lune_from_a_pole_corner.rs` pinned
  `(90°, radius 1000, Intersection)` as refused, and with the flush it
  answers. It was measured in a scratch copy of this branch, with that case
  moved into the test's `answered` list. The answer is one solid, the model
  validates, and it measures as a quarter of the ball, so the test passes.
  That file was not this lane's, so a second lane,
  `lane-q79-flush-tiny-corners-2`, moved the pin in `e1194c80`. Its
  verifier ran the gate on `e1194c80`: green, 2 576 of 2 576. With the base
  `triangulate.rs` restored the moved pin fails, refused `TooSmall`.
- **`orient2d`'s retry in `crates/rcad-predicates/src/orient2d.rs`** picks
  its power of two from the largest difference, so it misses inputs whose
  products span from a subnormal to about one. One power fits them all. That
  is a §16.10 gap in the predicate. It is worked round here, inside the
  triangulator's own check, because `rcad-predicates` was not this lane's to
  change. The fix belongs in `exponent_that_brings_into_range`.
- **The hull-outline check has no test that needs it alone** (321.2).
- §246.6's fuse refusals on the moved bar are untouched.
