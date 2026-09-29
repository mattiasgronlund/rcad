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

  *2026-09-29, Q539: done. 338.5 defines the exact middle of a face, and both
  wobbles are published against it.*
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

### 338.5 The centroid wobble, and which middle is exact

*2026-09-29. Queue row Q539, re-filed from Q401. Lane
`lane-q539-a-flat-solids-centroid-publishes-its-rounding`, off `a738fa55`.*

338.3's first bullet, done. A flat solid's `centroid_wobble`, and each flat
face's, is now worked out the way 338.2 works out the volume and area errors:
the sums behind the middle are taken exactly, and the published number is how
far the rounded middle landed from their ratio. So it is `0.0` exactly where
the middle came out on the nose, and every pin on a cube with whole-number
corners stays as it was.

**The solid needs no new definition.** Its exact middle is where the space its
triangles enclose balances: every tetrahedron the triangles make with the hub,
weighed by its signed volume and standing at a quarter of the way from the hub
to the sum of its corners. The triangles form a closed surface, so that point
is the same whatever the hub. It is the same triangles 338.1's volume is the
truth of. `volume_and_moment_as_expansions` sums both exactly from the exact
offsets `volume_and_moment` already builds.

**A face does need one, and it is the middle of the fan's triangles, each
weighed by its signed shadow on the fan's exact vector area.** Write `N` for
twice that vector area, the sum of `(b − a) × (c − a)` over the triangles with
nothing lost. A triangle weighs `((b − a) × (c − a)) · N` and stands at
`(a + b + c) / 3`. The face's middle is the weighted average.

Where the corners are in one plane, every sensible choice gives the same point.
Each cross product is parallel to `N`, so each weight is the triangle's area
times one common factor, with its sign, and the average is the textbook middle
of the face, holes and notches taken back. A turned face's corners are rounded
one by one and are not in one plane, and there the choices part:

- **The shadow on the rounded vector area.** This is what the `f64` code weighs
  by. A truth built from a rounded number moves whenever the arithmetic that
  rounds it moves, so it is a fact about the code and not about the face.
- **Each triangle's own area, with the sign it faces.** That is a square root
  per triangle, so there is no exact sum to hold the rounding against, and the
  published number could only be a forward bound. 338.2 turned that down,
  because it is not zero where nothing rounds.
- **The shadow on the exact vector area.** This is the one taken. It is built
  from the corners alone, by `+`, `−` and `×` and one division, so it can be
  worked out exactly. Its weights add up to `|N|²`, which is `|N|` times the
  doubled area `area_error` is measured against, so a face's area and its
  middle are about the same shadow.

These three weigh a triangle differently only through how far it leans off the
others, times a second small angle: the cosine's own step from one, or the tilt
of the rounded vector area. So they differ by a product of two small angles,
far below one rounding of the answer. **What does move the middle by about a
rounding is the choice of triangles.** Fanned from a different corner, the same
corners off one plane make a different surface. So the fan
`FaceView::for_each_triangle` hands out is part of the definition, as it is
part of the area's and the volume's.

**How it is worked out.** Along each axis the exact middle is `from +
moment / weight`, with `from` the hub or the fan's apex. `how_far_the_middle_is_from`
takes `middle − from` exactly with `two_diff`, forms `offset · weight − moment`
exactly, packs it, and divides its size by a lower bound on `|weight|` (a new
`at_least`), rounding up. Then it takes the length of the three distances,
rounding each step up. The packing matters. Without it the entries of the
difference partly cancel, and summing their sizes charged up to `1.6` times
the distance: `a less b` published `4.09e-16` for a middle `0.624` of that
away.

**Held against the truth.** `the_middle_of_a_flat_solid_and_of_each_face_is_within_its_wobble`,
in `a_flat_solid_publishes_the_rounding_of_its_measure.rs`, works out both
middles exactly by another route. For the solid, every tetrahedron is taken
from the world origin. For a face, each triangle's cross product is `a × b + b
× c + c × a` from the world origin, and the weights are summed one by one
rather than taken as `|N|²`. It then compares the squared distance with the
squared wobble exactly. It runs on Q81's two cubes, the cube of side two turned
a radian and a million units out, an L with uneven sides at the origin and
turned far out, and a plate with a hole turned far out. From `mise exec -- just t -p rcad-topo --test
a_flat_solid_publishes_the_rounding_of_its_measure --no-capture`: the solids'
wobbles are `2.55e-16` and `1.42e-16` for `a less b` and `b less a`,
`5.94e-11` for the far cube, `4.66e-16` for the L at the origin, `5.80e-11` for
it far out, and `2.05e-11` for the plate. The closest a solid's middle came to
its wobble, and the closest a face's came, both print as `1.000` of it. The
test floors both at `0.99`. With `measure.rs` put back to `a738fa55` and the
tests kept, it fails on `a less b`: measured `2.55e-16` from where the
triangles balance, with `0` owned up to.

**Do the exact sums assume nothing underflows?** Yes, as the volume's do (the
next bullet of 338.3), and they go one factor deeper. The solid's moment
multiplies a determinant, which is three offsets, by a fourth. A face's weight
multiplies two cross products, four sides, and its moment multiplies by a
fifth. The measured middle's offset from `from` is also scaled down by the
axis's shift, which is one more place where a lost part can fall below the
subnormals. So a milder mix of coordinate sizes than the volume's could reach
the subnormals and make a sum inexact. This is argued, not shown, exactly as
it is for the volume.

**Pins that moved.** All are in this row's write set.
`a_flat_solids_measurement_is_bit_for_bit_what_it_was` now carries each
frozen shape's wobble as a third error, and its bits did not move: the turned
cube `3.43e-16`, the turned plate `1.74e-15`, and the notched extrusion
`1.05e-16`. The notched extrusion's corners are whole numbers, but its middle
is five thirds along `x` and `y`, which no `f64` holds: a third of a spacing
on each axis, √2 times that together. The other three are `0.0`.
`a_cube_turned_and_far_away_measures_within_a_few_roundings_of_its_side_squared`
now asks each face's wobble to be at most one spacing of `f64` at a million
along each axis, and the largest to be above zero. Five faces print `5.8e-11`
to `8.2e-11`. The sixth face's middle came out exactly on an `f64` and owes
`0.0`, and the exact test above agrees. The volume and area errors did not
move a bit. `how_far_the_volume_rounded` reads the same expansion as before,
now handed in, and the face's exact vector area is the same sum, split out as
`twice_its_vector_area_exactly`.
