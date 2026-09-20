## 176. Fused arcs are in, the 18 LED files are not a coarse walk, and the 23 are a limit — 2026-09-20

*Four lanes off `7243d894`, the first batch dispatched by §175's skill. §169.4's
fused arcs are **done**, and the sketch that named the work was wrong in exactly
the way §168.4's was: the fuse lowers a picker's **offset**, nothing downstream
reads an offset, and the fix is one acceptance test rather than two rebuilt
steps. §174.5's diagnosis of the 18 LED files is **falsified** — the walk it
calls too coarse never runs at all, because the circle is imprinted on a flat
face and both faces along it are the same plane; the mesh misses by a whole
radius and no change in `mesh.rs` can reach it. The 23 files hidden behind the
coedge fix are a **limit**, each one measured against its own bytes rather than
inherited from §174.4. And a soak is clean — while three of the five suites it
named do not implement the soak mechanism at all.*

### 176.1 Fused arcs: the offset was the only thing in the way

§169.4 said admitting fused arcs "needs steps 1 and 2 rebuilt around ends *in
front of* a picker rather than on it". It does not. Of the three steps that read
an arc's picker, **none reads its offset**:
`which_side_of_its_neighbour_a_side_keeps`, `where_a_side_lies`'s two-ends-on-
the-cut arm, and `where_an_arc_with_one_end_on_a_cut_lies` each read
`picker.normal()` and nothing else. The offset is the only thing
`the_half_space_both_ends_are_on` lowers. So what stood in the way was the
acceptance test in `the_sides_of`, which asked `half.side_of_point(at) ==
Decision::Zero` of **both** ends — a question about the offset, asked on behalf
of three readers that never wanted it.

**The fix does not trust the picker's normal as a direction.** It builds the
direction itself, exactly, from the two corners as they are stored:
`square_to_the_chord = flat.normal × chord`, and then reads **one sign** off the
picker's normal to turn it the right way round. How nearly square to the chord
that normal is never comes into it. `SideRun::Arc` now carries
`into_the_arc: [Interval; 3]` instead of a `HalfSpace`, which is the whole of
what the three steps below ever wanted.

A picker is accepted two ways, and `a_picker_that_tells_the_two_arcs_apart` is
both of them:

- **Its plane holds both ends** — §167's test, kept, and what a face built by
  hand gives.
- **Both ends are on or in front of it and the circle's centre is on or behind
  it** — what a fused arc gives. Then `{h ≥ 0}` on the circle is an arc of at
  most a half, because `h(x) − h(c) = n·(x − c)` and `−h(c) ≥ 0`; so the arc
  between the ends that lies inside it is the **minor** one, its midpoint `M` is
  in there with it, and `h(M) = h(c) + r(n·ŝ)` with `h(c) ≤ 0` and `h(M) ≥ 0`
  forces `n·ŝ ≥ 0`. The sign the code reads is therefore the arc's own side.

`where_the_circles_centre_sits` answers the second without dividing: the centre
of the circle is `c + ((e − n·c)/(n·n))n`, and since `n·n` is positive the sign
wanted is the sign of `(h·c − d)(n·n) + (e − n·c)(h·n)`. The ball's radius never
enters it.

**The test.** `a_fused_eighth_of_a_ball_says_which_points_are_inside_it`, in
`crates/rcad-topo/tests/point_in_a_curved_solid.rs`: 8 points inside and 12
outside. Against the base tree's `holds.rs` it fails with
`CannotFollowAFacesTrimming { face: FaceKey { slot: FaceSlot(11v1), model:
ModelId(4) } }` where it wants `Inside` — reproduced by the verifier
independently, not taken from the lane.

**Still refused, each checked rather than assumed:** a sphere face whose cutting
planes miss the ball's centre (`is_a_family_this_follows`, untouched);
cylinders and cones (`gradient_at` still takes only planes and balls); an arc
with neither end on the cut; and grazes. One refusal is new and of the same
kind: a picker whose normal reads `Zero` or undecided against the
square-to-chord direction tells the two arcs apart not at all, so the face is
refused by name rather than guessed at.

### 176.2 The 18 LED files: §174.5's diagnosis, falsified

§174.5 read three corners at 180°, 270° and 0° on a circle of radius 1.5 and
called it "the edge walk being far too coarse, in `walk_a_wire_round`". Measured,
**the walk never runs at all.**

`walk_the_edges_this_face_can_place` returns early for a plane — rightly; a plane
cannot say where a curve drawn on it bends. On
`kicad/LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step` both faces along all three
arcs of that circle are the **same plane**, `y = −1.27`: the two surfaces read
`Plane{normal:(0,1,−1.158067919543e-15),offset:−1.2700000000000116}` and
`Plane{normal:(5.551115123126e-17,1,0),offset:−1.27}`, the same plane to within
1.2e-15. So no face places the edge, `along_edge` has no entry for it, and
`walk_a_wire_round`'s `.unwrap_or_default()` reads a missing entry as *"this edge
needs no points"*. The circle comes out as the triangle on its three vertices,
one side of which is a diameter, and the two pin holes straddle it. That is the
whole of `BoundaryCrossesItself { at: 3 }`.

**The silent hedge is the defect worth naming.** `Mesh::deviation()` publishes a
bound this mesh misses by a whole radius — 1.5 mm — because the side from
`(2.77, −10.0)` to `(−0.23, −9.999999956464)` is a diameter of a circle of radius
1.5. A mesh that is *wrong*, where a refusal was available.

**No change confined to `mesh.rs` can reach it**, which is why the lane landed
nothing rather than landing machinery that does nothing. The circle's centre and
radius are not in the model: `rcad-step`'s `the_arc_between` names the arc with
`and: Surface::Plane(the circle's own plane)`, which for a circle imprinted on a
flat face *is* the face's own plane, so the run says "where this plane crosses
itself". `rcad_geom::walk` then answers `WhatARunLiesOn::StraightLine` for any
plane pair, unconditionally. The one in-write-set route — letting the flat face
walk the edge when no curved face will — is a no-op for exactly that reason; it
was tried, measured, and reverted.

**This is a gap and not a limit** (§16.10): the arc exists and is representable,
and what cannot reach it is the vocabulary the reader wrote it in. Priced at 18
files, all one KiCad generator's output — `LED_D3.0mm_Horizontal_*` (7, radius
1.5) and `LED_D5.0mm_Horizontal_*` (11, radius 2.5). §45.2 still governs: 18 is a
ceiling, and none of these files has been past this face, so what stands behind
it is unmeasured.

### 176.3 The 23 are a limit, and each one was measured

§174.5 warned the next reader not to let the 23 inherit §174.4's verdict. They
were measured instead, and they reach the same verdict by their own evidence.

**Naming them.** The depth probe was run twice: once on the base, once on an
untracked scratchpad copy with only `92e5e1c3`'s `brep.rs` hunk reverted. The
copy reproduces §174.2's "then" column exactly — 75 / 26 / 1 — the base gives
98 / 1 / 3, and the name-diff is **23 added, 0 removed**, all KiCad. That the
reverted copy reproduces a published column is what makes the difference a
measurement rather than a guess.

**The misses.** Every one of the 23 misses its curved face by between
1.975e-13 mm and 7.523e-13 mm, which is **132 to 7851 coordinate spacings**. The
smallest is 132, so **none** falls in §174.6's below-one-spacing band and that
caveat does not bite this row — which had to be checked, because it is the one
thing that would have made the verdict worth less.

**The bytes agree with the kernel.** Exact decimal arithmetic on each file's own
coordinates reproduces the kernel's `off_by` to 3–5 significant digits in all ten
distinct cases: `SW_PUSH_6mm` computes 1.972693e-13 against the kernel's
1.975374e-13; `SW_TH_Tactile_Omron_B3F-1110` 7.522911e-13 against 7.522999e-13.
Every corner is written to exactly twelve decimal places and the surface's answer
continues past the twelfth digit. The one cone in the set,
`Fuseholder_TR5_Littelfuse_No560_No460.step`, writes a half-angle of `π/4` as
`0.785398163397` — an irrational no decimal file can ever hold.

So **the row is a limit, unanimously**: a modelling tolerance somebody chose,
which §44.6's table rules a limit and which no arithmetic fixes.

**A correction to the brief that dispatched this lane**, recorded because the
lane was right to override it: the brief glossed a short-written decimal as a
*gap*. §44.6 and §174.4 rule the opposite — a miss **inside** one coordinate
spacing is the gap (the file wrote the nearest `f64` and the arithmetic could not
reach it), and a miss far outside it is the limit. The lane followed the record
rather than the brief and said so. A brief is not a source; `docs/decisions.md`
is.

### 176.4 A clean soak, and three suites that do not soak

§174.9 asked for a soak, because triangulation changed and the property suites
search randomly. All five named suites passed clean, ~14.6 minutes of test
wall-clock: `fuses_that_must_agree` at `RCAD_SOAK_FACTOR=500` (54 tests, 691.7s),
`turning_a_fuse` at 500 (2 tests, 164.9s), `random_extrusions` at
`PROPTEST_CASES=128000` (3 tests, 8.9s), and the two mesh suites in release
(6 tests, 8.9s; 24 tests, 0.1s). No `.proptest-regressions` file changed
anywhere, so no new failing seed was ever persisted.

**What the soak does not cover is the thing it was asked for.**
`whether_a_coarser_deflection_always_meshes` and `meshing_a_curved_solid` — the
two named as the priority, because triangulation is what moved — are fixed,
enumerated `#[test]` cases with no randomisation at all: no `proptest`, no
`RCAD_SOAK_FACTOR` hook, nothing that scales. A soak factor is a no-op there, and
one release run is the whole of what they can ever say. `random_extrusions` uses
a bare `proptest!` block at default config and does not read `RCAD_SOAK_FACTOR`
either; it soaks only through `PROPTEST_CASES`. Only two of the five wire into
the mechanism the `testing` skill documents.

So the triangulation change got a wider search only indirectly, through the
meshing paths `fuses_that_must_agree`, `turning_a_fuse` and `random_extrusions`
exercise. **Saying "the suites were soaked" of this set would have been true and
misleading**, and it is written down here so the next reader does not have to
find it again.

### 176.5 What is left

- **The 18 LED files**, now diagnosed and two crates away from where §174.5
  pointed. `crates/rcad-step/src/brep.rs`'s `the_arc_between` should name the
  arc with a surface that really crosses the face in that circle — a cylinder of
  the circle's radius about its axis — so the geometry survives into the model;
  an `ELLIPSE` needs its own answer and does not get one from a cylinder.
  `crates/rcad-geom/src/walk.rs`'s `the_two_surfaces_it_runs_between` reads the
  run's `and` only when the two face surfaces are equal **by value**, and here
  they are two planes 1.2e-15 apart, so it does not. This is the next unit.
- **A mesh that is wrong rather than refused.** Whatever is done about the arc,
  an edge whose run is not `Straight` and whose two faces are both planes can
  never be placed, and `Model::validate` passes such a solid today. Either the
  sweep says so or `mesh.rs` refuses by name — there is no existing variant that
  says "this edge bends and every face along it is flat", and
  `AFaceMeetsItsNeighbourOnItsOwnSurface` claims more than the mesher can prove.
- **S2 of the unheld-geometry family**, 37 files, two geometry types. §174.7,
  untouched by this batch.
- **A randomised generator for the two mesh suites**, or the explicit decision
  that the meshing paths inside the fuse suites are the coverage. §176.4.
- **The 23** are closed: a limit, measured. §174.9's first item needs no lane.

Write set: this section, `docs/decisions.md` §0, the pointer at the end of
§169.4, `crates/rcad-topo/src/holds.rs` and
`crates/rcad-topo/tests/point_in_a_curved_solid.rs`.

