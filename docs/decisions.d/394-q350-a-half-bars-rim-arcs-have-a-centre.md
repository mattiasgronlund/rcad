## 394. Q350: a half bar's rim arcs have a centre, and `holds.rs` follows its faces — 2026-09-29

*Q350, `8f9750e2`, on `crates/rcad-topo/src/holds.rs`. Closes §325.5 row 1 and
the first item of §331.5.*

### 394.1 A pipe crossing a plane is centred on the axis origin

`where_a_closed_crossing_is_centred` (`crates/rcad-topo/src/holds.rs`) had an
arm for a ball and a cone, and none for a pipe, which is what left §325.5 row 1
and §331.5's first item open. It now has one: the axis origin as the curve's
centre, and the stored axis direction — turned to read positive against the
plane's normal — as the arm's `along`. The sign that turn needs is
`SquareToTheAxis` (`crates/rcad-topo/src/holds.rs`), which reads the sign of
the dot of a plane's normal and a ring's or a pipe's axis as a level-3 sum of
three products.

### 394.2 `holds.rs` follows a half bar's faces

Behind the centre, `holds.rs` reads:

- `gradient_at` (`crates/rcad-topo/src/holds.rs`) on a pipe, for a straight
  side on a curved face.
- A pipe cutting a flat face in a closed curve.
- `where_a_side_lies` (`crates/rcad-topo/src/holds.rs`) for a straight side
  against a pipe.
- `leaves_a_pipe_in_one_piece` (`crates/rcad-topo/src/holds.rs`), for a pipe
  face cut by planes, shown one piece.

All four exist in `holds.rs` at this tip (`mise exec -- just src <name>`).

### 394.3 What answers now

`mise exec -- just t -p rcad-topo --test merged -E 'test(/^two_half_bars_stacked_on_one_pipe::/) or test(/^fusing_two_bars_side_by_side::/)' --no-capture`
on this tip: 14 and 9 tests, all pass. Among them:

- **`two_half_bars_fuse_directly_to_what_cutting_the_block_off_afterwards_gives`**
  (`two_half_bars_stacked_on_one_pipe.rs`) is the twelve direct fuses: two
  wall spellings × three rules × both ways round, every one of which used to
  refuse `AFaceTrimmedByACurveThisCannotFollow`. Now each answers `8π`
  (union), nothing (intersection) or `4π` (difference), a half bar of four
  faces, within the mesh's bound.
- **`a_half_bar_places_a_point_inside_as_the_bar_and_the_block_say`**
  (same file) is the `classify_point((1, 0.5, 0.5))` claim: the point is
  inside the half bar, as it is inside the whole bar and outside the block
  cut off it.
- **`a_union_or_a_lens_fused_with_a_block_it_does_not_touch_comes_back_as_it_was`**
  (`fusing_two_bars_side_by_side.rs`) is §331's renamed pin: a union or a
  lens of two side-by-side bars, fused again with a block fifty above it that
  touches nothing, comes back with its first fuse's four corners, four faces
  and volume, by every rule and both ways round. It used to refuse
  `AFaceTrimmedByACurveThisCannotFollow` on the answer's face at `z = 0`, the
  same missing pipe arm. Its doc names what still refuses after this fix: the
  two crescents fused the same way, which refuse
  `ACurvedFaceGoesRightRoundWithNoSeam` — §331.5's last item, still open.

### 394.4 What this closes

- **§325.5 row 1** closes: the first step it named, a pipe arm in
  `where_a_closed_crossing_is_centred`, is in, and the two `holds.rs` items of
  §325.2 are the faces 394.2 lists.
- **§331.5's first item** closes: the arm landed, and the pin it broke is
  rewritten under its new name, 394.3's third test.

§331.5's second item (the top half, after the arm) and its last item (the
crescents against a block that touches nothing) are Q596's and are unaffected
here; neither is this row's to close.
