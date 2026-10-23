## 282. The frustum's refused piece is two cells glued together, and the pointed cone's eight were a missing point — 2026-09-26

*Queue row Q123, from §251.6. Lane `lane-q123-frustum-piece` off `689fe3a3`.
The leaning block of `a_plane_across_a_cone.rs` and the 44 of the steeper-plane
family refuse on a piece that is **two pieces glued into one**, so no point can
be in it and selection is right to refuse. The glue is `arrange.rs`'s
`group_the_cycles_into_cells`, outside this lane. It is a gap. The 8 of the
pointed-cone family are something else: a triangle with the apex for a corner,
with no candidate inside it. That one was in this lane's files and is fixed.*

### 282.1 The piece, reproduced

The union of the leaning block with the frustum (slope 1/2 about `+z`, from
`z = 2` to `z = 8`) refuses on a piece of the **frustum's cone face**. The
block's low-`x` face is the plane `x = 3/8 (z + 1)`. It cuts the cone's upper
nappe in an arc of a hyperbola, from `(3.375, −2.147, 8)` on the top rim down to
`(1.5, 0, 3)` and back up to `(3.375, 2.147, 8)`. So the face should come apart
into two pieces:

- the **patch inside the block**, between the arc and the top rim through
  `(4, 0, 8)`;
- the **rest**, between the bottom rim and the arc together with the far side of
  the top rim.

Printed with a temporary `eprintln!` in `place_the_piece`, since removed, the
piece selection was handed has **three wires**:

| wire | sides (partner, material side) |
|---|---|
| 0 | bottom cap plane, `Neg` (above `z = 2`): the whole bottom rim |
| 1 | top cap plane `Neg` twice (rim through `(4, 0, 8)`), then the block's face `Neg` (inside the block) |
| 2 | top cap plane `Neg` (the far rim), then the block's face `Pos` (outside the block) |

Wires 1 and 2 run along the **same arc** in opposite directions, one with the
block's material on its material side and one with the outside.
`candidate_points` gave 27 candidates, and `the_piece_holds` said `false` to
every one:

- the 17 on a rim or above the top one failed that rim's sign, which came back
  undecided or the wrong way;
- each of the 10 strictly between the rims failed **exactly one** of the two
  sides along the arc. For example, `(2.5, 0, 5)` is inside the block and fails
  wire 2, and `(−3.5, 0, 7)` is outside it and fails wire 1.

No point is on both sides of one plane.

### 282.2 Which case it is: (c), and where the glue comes from

This is not (a), because candidates do lie in each of the two real pieces. It is
not (b), because the side rule answers `false` correctly for every candidate:
the piece really holds none of them. It is **(c)**: the arrangement handed
selection a piece that is not one region.

Printed with a temporary `eprintln!` in `group_the_cycles_into_cells`, since
removed:

- cycle 0 is the bottom rim's upper walk (lump 0, winding 1, sides
  `[Neg, Neg]` about the two rims);
- cycle 2 is the patch inside the block (lump 1, winding 1, `[Neg, Neg]`);
- cycle 4 is the rest's upper walk (lump 1, winding 1, `[Neg, Neg]`);
- the joins made by signs were `0–2` and `0–4`, and no others.

§36.3's rule joins two walks in **different** lumps when they agree about every
closed curve. Cycles 2 and 4 are in one lump, so they are never compared with
each other. What tells them apart is the arc, and the arc runs on past the rim,
so it is never drawn in full and has no sign. Cycle 0 agrees with both about
both rims, so it is joined to both, and they are joined to each other through
it. §36.3's sentence "two walks bound one cell exactly when they agree about
every closed curve" is false here. It holds only when every curve that separates
the walks of a lump closes.

**Selection's refusal is what stops this from being a wrong answer.** If a
point had been held for the glued piece, the patch and the rest would share one
fate, and in the union the patch is dropped (it is inside the block) while the
rest is kept. So the obvious fix of (b), letting `the_piece_holds` wind a curved
piece or ask its sides less strictly, would turn this refusal into a wrong
answer. The side rule is right. Do not change it for this.

### 282.3 A gap, and the fix is outside this lane

§16.10: the answer exists. The union of the two shapes is well defined, and so
are the two pieces the face should be cut into. So this is a **gap**, and it is
in `crates/rcad-topo/src/fuse/arrange.rs`, `group_the_cycles_into_cells`, which
this lane does not own. What the fix has to do, without claiming how:

- **Never let the sign rule put two walks of one lump into one cell** on a
  surface where the walks of one connected drawing bound different regions:
  the plane, the pipe, the cone and the ball. A torus is the exception, because
  a meridian circle's two sides are one region there, so this cannot simply be
  asserted everywhere.
- **When a walk's signs match more than one walk of another lump, the signs
  cannot place it**, and something else must. `piece_around` is the other route
  there is. Reading `first_arc_met`, its ray along the arc only looks at arcs of
  the home lump, and here the arc never reaches the bottom rim. So with only a
  guard added, this face would likely stop in `piece_around` instead. That is
  read from the code, not measured.

### 282.4 The same glue on a flat face

The rule does not care what the surface is, so it was probed on the most
ordinary shape that has the same drawing. The probe was a scratch test, run and
then deleted:

- a plate 10 × 10 × 2;
- a bar of radius 1 at `(3, 5)` through the plate, whose circle is a lump of its
  own on the plate's top and bottom;
- a block over `x ∈ [6, 16]`, `y ∈ [2, 8]` through the plate, whose three walls
  cut straight lines that join the plate's own boundary.

**Measured:** with each tool alone, `Difference`, `Union` and `Intersection`
each answer one solid. With both tools, all three refuse `ResultIsNotManifold`,
with count 3, 3 and 1. The same temporary print showed that on both the top and
the bottom face, the five walks of the boundary's lump (all winding 1) were each
joined to the circle's outer walk: five cells glued into one. On a flat face
`the_piece_holds` winds, and the winding finds a point in one of the five cells,
so the glued piece gets one fate. Here that ended as a non-manifold refusal.
Whether some shape turns it into a wrong answer that still validates has not
been measured, and the arrange.rs row should settle that first.

### 282.5 The pointed cone's eight: case (a), fixed

The 8 point-in-piece refusals of
`a_pointed_cone_with_its_apex_near_a_steeper_face_answers_right_or_refuses`
are **flat** pieces. Each is the block's steep face, where it passes through the
apex (crossing 0), bounded by one wire of two sides: a chord along the cap plane
or the block's bottom, and one "arc" along the cone. The plane through the apex
cuts the cone in two rulings, so the piece is a **triangle whose third corner is
the apex**, and nothing names that corner. Its two named corners wind round
nothing.

It had five candidates:

- three are the chord's middle, on the boundary;
- two come from `half_way_into_either_arc`, which aims past the apex as if it
  were a circle's centre, so one candidate lands beyond the chord and one past
  the apex. Both were worked by hand and match the printed points.

`half_way_across_the_chord_to_a_cone` gave none. It can return nothing only
when `across` does not normalise, the side is not a cone, the steps are not
finite, or the discriminant is below zero. The first three do not apply to
these pieces, so it was the discriminant. That fits the geometry: both ends of
the chord are at one height, so the triangle has two equal sides and the line
square to the chord through its middle runs through the apex. It touches the
cone there twice over, a double root, and rounding sends the discriminant to
either side of zero.

The fix reads a negative discriminant as that one root:
`discriminant.max(0.0).sqrt()`. The candidate half way to the apex is inside the
triangle, and `the_piece_holds` proves it through `in_the_bulge_of`'s nappe
rule. Where the line really misses the cone, the step is just one more point
that gets proved or dropped. **This gap was inside the lane's files and is
closed.** It is in `half_way_across_the_chord_to_a_cone`, the helper that only
`candidate_points` calls. It does not give a curved piece an exact point, so it
does nothing for Q102.

### 282.6 What moved

Commands: `mise exec -- just t -p rcad-topo --test merged -E 'test(/^a_plane_cut_along_a_cones_other_nappe::/)'`
before and after, and `mise exec -- just t -p rcad-topo --no-fail-fast` after
(1028 run, 1028 pass).

- **Pointed cone near a steeper face**: 60 right / 12 `CannotMeshAFace` / 16
  naming / 8 point-in-piece → **64 right** / 16 `CannotMeshAFace` / 16 naming.
  Of the 8, four answer, each held to that file's four checks, and four give up
  meshing. Nothing answered wrong: `fuse_and_check` panics on a wrong answer,
  and none did.
- **Steeper plane across a frustum**: unchanged at 40 / 44 / 12. All 44 refuse
  on a cone piece whose wires have 1, 3 and 2 sides, with the block's steep face
  as partner once with each material side. That is §282.1's glued piece, counted
  by a temporary signature print.
- **Block below a frustum**: unchanged at 56 / 20 / 20. Its 20 point-in-piece
  refusals have the same glued signature, with wires of 1, 3, 2 (8 cases),
  1, 2, 2 (4), 3, 2, 1 (4) and 3, 3, 2 (4).
- `a_plane_across_a_cone.rs`'s leaning block still refuses where it did. Its pin
  now holds the six corners **in wire order**, which is the three-wire piece,
  rather than sorted and deduplicated.
- `how_often_a_fuse_with_a_cone_refuses.rs`: nothing moved.

### 282.7 What this leaves

- **`arrange.rs`, `group_the_cycles_into_cells`**: §282.3. Its cases are the
  leaning block's union, the 44, the 20 of the block below, and §282.4's plate
  with a hole beside a cut.
- §36.3's heading states the rule as exact. §282.2 is where it is not, and the
  row that fixes it should correct that section too.
- The curved arm of `the_piece_holds` did not ask which nappe a point is on.
  A pointed cone's face is held by `z < 8` alone, and the whole lower nappe
  passes that. §350 measured whether a candidate ever reaches the other
  nappe: one does, on every cone whose radius is more than its height, and
  `the_piece_holds` now turns such a point away (§350.5 first, then §350.3).
