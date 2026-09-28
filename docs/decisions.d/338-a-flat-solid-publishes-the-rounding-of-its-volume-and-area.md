## 338. A flat solid publishes the rounding of its volume and area — 2026-09-28

*Queue row Q83, from §242.6. Lane `lane-q83-a-flat-solid-publishes-its-rounding`,
off `cc7499c3`.*

§25.4 said a wholly flat solid is "summed exactly", and that all three of its
errors are exactly `0.0`. The sums were `f64` sums. `volume_and_moment` redid
the volume exactly only when the sum was near zero, to settle its sign.
Anywhere else it was plain addition, and so was the area. A published error of
`0.0` on a value that is off is a wrong bound.

### 338.1 Both cases, measured against the exact truth

The test works out the truth of the *written* polyhedron in expansions: every
triangle `FaceView::for_each_triangle` hands out, taken from the world origin
and not from the kernel's hub. So it is a different route to the same number.
On the base, both of the row's cases fail it.
`a_flat_solid_publishes_the_rounding_of_its_measure.rs`, run with `mise exec
-- just t -p rcad-topo --test a_flat_solid_publishes_the_rounding_of_its_measure
--no-capture`:

- **Q81's two turned cubes.** `a` less `b` measures `8.000000000000002` with an
  error of `0`. The polyhedron the fuse wrote encloses about
  `8.000000000000002` too, but not exactly that value. The turn rounded the
  corners, so most of the distance from `8` is the solid's own. Only the last
  rounding is the sum's.
- **The cube of side two turned a radian and put a million units out.** A face
  measures `4.000000000015661` with an error of `0`, and the face it holds has
  an area of about `4.000000000015661`. §242.6's "`1.25e-10` off `4`" is where
  the rounded corners are. It is not the fan's arithmetic.

### 338.2 Publish the rounding, and keep the values

**Summing exactly cannot give a zero error in general, so the error has to be
published either way.** Six times the volume is a sum of products of `f64`, a
dyadic fraction. A sixth of it is a dyadic fraction only when three divides the
numerator. An area is half the square root of a dyadic fraction. So no `f64`
can hold either one in general, and "all three errors exactly `0.0`" is a
limit, not a gap.

That leaves two choices: round the exact sum and publish that rounding, or keep
the `f64` sum and publish how far it landed. **The lane kept the sum.** §25.4
gives its reason for the whole arrangement as "what keeps M0's numbers bit for
bit", and `a_flat_solids_measurement_is_bit_for_bit_what_it_was` pins those
bits. They did not move: that test still passes its bit comparison and fails
only at its `volume_error == 0.0` line (338.4). An exact sum would move the
bits of both turned shapes in that pin, and it would buy an error no smaller
than one rounding.

**The error is worked out, not bounded in advance.** A forward bound would be
nonzero on a cube with whole-number corners, where nothing rounds. The exact
distance is `0.0` there, so every pin on an unturned polyhedron stays as it
was.

- **Volume.** `how_far_the_volume_rounded` takes the fan's exact offsets, which
  `volume_and_moment` already built, and sums their determinants exactly,
  packing with `compress` after each triangle. It subtracts `6 × sixth`
  (exact, by `two_product`), divides the size of what is left by six rounding
  up, and shifts it back with `shift_up`. If shifting the volume itself landed
  among the subnormals, the spacing there is charged as well.
- **One face's area.** `how_far_the_area_rounded` takes each side exactly with
  `two_diff`, and the cross products and `S = |twice the vector area|²` exactly
  too. The distance is at most `|S − 4·length²| / (4·length)`, which needs no
  square root. `OwnFan` is the one copy of this, used by the solid and by
  `measure_one_face`, so the faces still add up to the solid bit for bit.
- **The solid's area.** The faces' errors added, each addition rounded up,
  plus what the running `f64` total dropped. That last term is found by adding
  the faces' areas again exactly.

After the fix, from the same test: `a` less `b` publishes `1.32e-15` and `b`
less `a` publishes `1.42e-15`, about one spacing of `f64` at 8. The far cube's
faces publish `1.7e-16` to `1.29e-15`, its volume `1.26e-15`, and its skin
`4.02e-15`. Each one holds the exact truth, and the skin is checked against a
bracket of two `f64` per face, because one `f64` per face was wider than the
bound. `a_cube_on_whole_numbers_still_owes_nothing` pins the zeros.

### 338.3 What is not done

- **The centroid wobble is still `0.0` for a flat solid and for its faces**,
  and the centroid is rounded like everything else. A face's centroid also has
  a question the area did not: a turned face's corners are not exactly in one
  plane, and the fan weighs its triangles by their shadow on the rounded
  vector area, so "the exact centroid" first needs a definition. The doc
  comments now say this. It is a queue row, not part of this one.
- **The exact sums assume nothing underflows.** An offset's lost part scaled by
  its axis's shift, or a product of three of them, could fall below the
  subnormals for a solid whose corners mix coordinates near `1e-300` with ones
  near one. `Model::validate`'s dynamic-range check refuses such a solid
  (§19.6). `measure` does not validate, so a deserialised one could reach this.
  This is argued, not shown.

### 338.4 Pins outside the write set that move

`mise exec -- just t -p rcad-topo --no-fail-fast`: 1 109 run, 1 107 passed,
2 failed, 11 skipped. The two that fail are both assertions that a turned flat
solid owes `0.0`, which is the claim this section withdraws:

- `measuring.rs:437`, `a_flat_solids_measurement_is_bit_for_bit_what_it_was`:
  "a cube of side three, turned a third of a turn" now has a `volume_error` of
  `6.66e-16`. The bits above it held. The loop stops at the first shape, so the
  turned plate's errors were not seen.
- `measuring_one_face.rs:190`, `a_cube_turned_and_far_away_measures_within_a_few_roundings_of_its_side_squared`:
  `area_error` is now `2.89e-16` where it asserts `0.0`. Its doc comment's
  "So the bound contains the truth does not hold" is what this section fixes.

`rcad` and `rcad-step` (`just t -p rcad -p rcad-step`) pass, 258 of 258, and so
do the doc tests of `rcad-topo` and `rcad`, 4 and 2. Every pin they hold is on
an unturned cube.

**Both were re-pinned at the merge, by the dispatcher.** The frozen-shape test
now carries each shape's two errors beside its five bit patterns: `0.0` for the
four whose corners are whole numbers, and for the two turned shapes what the
fix publishes — the cube `6.66e-16` and `1.30e-14`, the plate with a hole
(never seen before, since the loop stopped on the cube) `1.75e-14` and
`6.53e-14`, each under two spacings of `f64` at its value. The far cube's faces
now say their wobble is `0.0` and their area error is above zero and at most
two spacings at `4`; the doc comment says the error covers the adding up and
not the placing, which still puts every face `1.25e-10` off `side²`.
