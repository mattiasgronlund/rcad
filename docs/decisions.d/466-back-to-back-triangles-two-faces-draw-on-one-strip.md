## 466. Back-to-back triangles two faces draw on one strip — 2026-10-25

*Q2774 asked whether the volume sum validation reads may take two faces'
triangles drawn back to back on the same three points, which cancel, or how
the mesh keeps them apart against §187.4's wrong mesh. The sum may keep them,
and the mesh a caller is handed may not. The thick bar less the thin one turned
a radian about z now answers and validates, and so do two other solids that
the same fold stopped.*

### 466.1 What a pair is

A pair is a triangle of one face on the same three mesh points as a triangle of
another face, turned the other way, where the two faces lie on different
surfaces as they sit. `back_to_back_with_another_faces` in `mesh.rs` finds
them. Each triangle is paired at most once, with the first triangle after it in
the mesh that is not paired yet, so one mesh is paired the same way every time.

Pairs come where two faces bound one thin strip from the same two loops and
the mesh puts no point inside the strip. Three shapes are known:

- **The thick bar less the thin one.** The bar of radius 2 up z from
  `(0, 0, −2)` less the bar of radius 1 along x from `(−4, 1, 0)`, 8 long,
  turned 1 about z (Q2774's row, from Q2768). The thick wall's strip between
  its two windows and the thin wall inside it bound the same strip.
- **The round bar less the tilted block** (`an_ellipse_start_between_two_places.rs`),
  with radius `1 + 2⁻⁵¹` (Q1972, Q2262) and `1 + 2⁻⁴⁰` (Q1869). The block's
  flat strip of side between the fin's two corners and the bar's wall over it
  bound the same strip.
- **The plate with a round tab united with the block** `x 7.6 to 12.4` by
  `y 8.2 to 11.8` under its floor, under `tolerant(100)`
  (`a_block_cornered_on_a_round_tabs_rim.rs`, Q2478). One pair: a sliver of
  the block's roof one rounding wide (plane `z = 0`) and the tab's wall (the
  cylinder of radius 3 about `(10, 10)`) each draw a triangle on the points
  `(7.6, 11.8, 0)`, `(7.6000000000000005, 11.8, 0)` and `(7.6, 11.8, 0)`. Two
  of those are two mesh points at one place, so the triangle has no area.

Each side of a pair is used four times, twice by the pair and twice by the rest
of the mesh. Until Q2774, `BuildingAMesh::finish` refused that as any other
fold: `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }` for the first
two shapes, and `{ drawn: 2, used: 4 }` for the third. A finer mesh does not
clear it, since the strip is drawn that way at every mesh: from room `1.19`
down to `0.037` for the thick bar, and down to the fold ladder's last rung at
`6.1e-4` for the fin.

### 466.2 The mesh a caller is handed still refuses a pair

`SolidView::mesh` asks with `TrianglesBackToBack::Refused`. A mesh with a pair
in it is not one closed surface: four triangles meet along each side of the
pair. That is §187.4's wrong mesh. In the bar whose top leans five across for
one up, the cap and the wall cut off the same ear, and that gave two triangles
on the same three points facing opposite ways, a side used four times, and χ 3.
`mesh()` returned that as `Ok`, which was a wrong answer. A caller handed such
a mesh, to write out or to draw, gets the same wrong surface. So `mesh()`
refuses a pair as it refuses any other fold.

### 466.3 The sum validation reads may keep a pair

`SolidView::a_mesh_to_sum_over`, which is private to the crate, asks with
`TrianglesBackToBack::Kept`. Every mesh `validate.rs` asks for is asked that
way: in `how_much_space_a_curved_solid_encloses` and in
`the_coarsest_mesh_whose_sides_do_not_cross`. Under `Kept`, `finish` looks for
pairs only once it has found a fold, so a mesh that does not fold costs nothing
more. It counts the pairs out and asks whether the rest folds. It holds the
rest to what it holds any mesh to: each side drawn inside a patch is used by
exactly two triangles. Both triangles of each pair stay in the mesh, so each
face's area is still its own triangles'.

A pair does not move the sum:

- **Its volumes cancel.** The signed volume is a sum over the triangles of
  `det(a − H, b − H, c − H) / 6`, from one point `H`. The pair's two terms are
  one determinant with two columns swapped, so in exact arithmetic each is the
  other's negative. Each term's rounding is charged as any triangle's is.
- **Its moves cancel.** In `what_it_encloses_moved_onto_its_planes`
  (`validate.rs`), each point's move onto its plane belongs to the point
  (`the_moves_onto_the_planes`), so both triangles of a pair see the same three
  moves. Their first-order terms, `Dₐ · (B × C) + D_b · (C × A) + D_c · (A × B)`,
  are each other's negatives too.
- **The sum is still the same from any one point.** That function's doc rests
  its one `H` for every triangle on this: a closed mesh's sum is the same from
  any point. The rest of the mesh is closed, by the check above. The pair
  alone is closed too, each of its sides used once each way, and it encloses
  nothing from any point.
- **Its charges only add.** `first_order_size`, `higher_orders` and `hidden`
  are sums of sizes, and each triangle of a pair is charged as a triangle of
  its own face. So a pair can widen the error bar, and cannot move the answer.

### 466.4 Two faces of one surface back to back stay a fold

Two faces of one surface drawn back to back on the same three points are the
two sides of one sheet: a fin with nothing on either side of it, a solid folded
onto itself. §463.1 builds solids of that kind by hand. That stays a fold under
`Kept` as well.

"Another surface" is tested with an exact `Surface ==` on the surfaces as they
sit. So one plane written the other way round, normal `−z` at offset `0`
against normal `+z` at offset `0`, counts as another surface. Two faces of one
plane facing opposite ways, with their triangles on the same three points,
would then be paired and kept, where this section means them to stay a fold.
This has not been run, and no shape is known that does it: the round tab's pair
is a plane against a cylinder. Q2620, a check in `Model::validate` for two
faces of one solid on one patch (§463), would refuse such a solid whatever the
mesh does.

### 466.5 A side of a pair is held to two uses

`BuildingAMesh::the_sides_that_fold` holds each side that a triangle outside
the pairs drew inside its patch to being used by exactly two triangles. It also
holds it to being drawn inside by both of them, unless it is a side of a pair.
One face may draw a side inside its patch where the other face of its pair runs
along a loop there. Counted out, the pair takes one of the two triangles the
first face drew on that side. What is left there is the first face's other
triangle and the triangle across the second face's loop, drawn inside once and
used twice, which is how two faces meet along an edge. Without that, the first
lane's fix (`8248a3378`) refused both tilted-block tests in
`an_ellipse_start_between_two_places.rs` `{ drawn: 1, used: 2 }`. With it
(`c815e2862`) they answer.

### 466.6 What the probes measured

`SolidView::measure` still meshes with `SolidView::mesh`, so each of these
answers validates, and its `measure` is refused
`TwoPatchesOfTheMeshDrewTheSameSide` (466.8). Probes on Q2774's branch made
`measure` call `a_mesh_to_sum_over` instead, and were then removed:

- **The thick bar less the thin one**: one solid of four faces, which validates
  under `certified`. It measured within its error bar of `16π` less what the
  two bars share.
- **The bar of radius `1 + 2⁻⁴⁰` less the tilted block**, under `tolerant(25)`:
  `23.338721079017855 ± 0.47206064312105267`, against `23.379808839548105`
  worked out as the file's closed forms work it. Under `certified` the fuse is
  refused `DynamicRangeTooWide { ratio: 18691697672192.004 }` before any mesh.
- **The bar of radius `1 + 2⁻⁵¹` less the tilted block**, under `tolerant(25)`:
  within its error bar of `23.380`.
- **The tab's union under the floor**, under `tolerant(100)`, either way round:
  volume `259.6650066077878 ± 0.13200870781844362` against
  `259.6915008234622`. Area `355.44894103826437 ± 1.2791889237486007` against
  `338.64737793344216 + 51.36 − 2 × 17.28 = 355.44737793344217`, the plate's
  area and the block's, less the block's roof twice. So no face is doubled. It
  has 15 faces: the plate's 9, and the block's 6, one of them the sliver of its
  roof. One pair was counted out, the triangle of no area in 466.1. The plate
  alone measured `242.37853869946048 ± 0.132` against `242.4115`.

### 466.7 Q2262 needs no name of its own

Q2262 asked whether, under `tolerant`, a fin whose points round onto one plane
is refused by a name of its own rather than as a fold. Under `tolerant(25)` the
bar of radius `1 + 2⁻⁵¹` less the tilted block now answers one solid that
validates under `tolerant`. The same model is refused `DynamicRangeTooWide`
under `certified`, as before. Nothing refuses it as a fold now, so it needs no
name. Its `measure` refusal is 466.8's row.
`every_fuse_with_the_tilted_block_answers_its_closed_form_but_the_bar_less_the_block`
pins it.

### 466.8 What this leaves

- **`SolidView::measure` meshes with `SolidView::mesh`**, so the four answers
  of 466.6 validate and do not measure. A row (`refuses/B`, from Q2774) makes
  `measure` sum over `a_mesh_to_sum_over` and moves those pins to their closed
  forms.
- **The round tab's pin.** In `a_block_cornered_on_a_round_tabs_rim.rs` the
  tolerant union under the floor was pinned refused
  `TwoPatchesOfTheMeshDrewTheSameSide`. Since Q2774 it answers one solid that
  validates, and its `measure` is refused. That file is outside Q2774's write
  set, so the pin moved at Q2774's merge, to that answer; the `measure` row
  moves it to its closed form.
- **Q2478 keeps its other half**: the same union with the block on the plate's
  top is still refused `AWireDoesNotFanIntoALoopToMesh`.
- **One plane written two ways** (466.4) is Q2620's to catch.
