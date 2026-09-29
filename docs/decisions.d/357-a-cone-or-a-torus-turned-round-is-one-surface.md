## 357. A cone or a torus turned round is one surface — 2026-09-29

*Queue row Q355, from Q44 and §231.5. Lane
`lane-q355-a-cone-turned-round-is-one-surface`, off `9eb17cf7`. `intersect`
called two cones, or two tori, the same surface only when they held the same
numbers, so a cone or a torus named with its axis turned round was refused by
name where a pipe turned round answers `SameSurface`. **It is decided exactly
now**: two cones with one apex, one slope and their axes along one line are
`SameSurface` whichever way each axis points, and two tori with one centre,
one pair of radii and their axes along one line the same. Two frustums stacked
on one cone, the upper one naming it about `−z`, unite in both orders into one
solid that validates.*

### 357.1 A cone here is both nappes

`Cone`'s own doc says so, `crates/rcad-geom/src/cone.rs` line 32: "this is the
**whole double cone**, both halves, meeting at the apex". Its sign is
`‖(p − apex) × w‖² − slope²·((p − apex)·w)²`, degree two in `w` on both terms.
So the cone about `w` and the cone about `−w` with the same apex are one point
set, and so are the cones about any two directions along one line — the sign is
multiplied by `λ²`, so every point keeps its side, not only the points on the
skin. Had a cone been one nappe, the cone with its axis reversed about the same
apex would have been the *other* nappe and the frustum pair would have been two
surfaces meeting at the apex only; it is not.

A torus is the same one degree up: its sign
`(w·w)·(x·x + R² − r²)² − 4R²·‖x × w‖²` is degree two in `w` on both terms.

### 357.2 What decides it, and why nothing else could

`one_cone_either_way_round` compares the apex and the slope as stored and asks
whether the two directions lie along one line; `one_ring_either_way_round`
compares the centre and both radii and asks the same of the directions. That is
the "if" of 357.1 and all of it.

**The "only if" is what makes a `false` safe to hand to the refusal.** A cone's
apex is its one point with no normal, its axis its one line of symmetry, and a
slope above zero (the only kind `Cone::new` builds) how fast it widens along
it; a torus with its tube narrower than its ring (the only kind `Torus::new`
builds) has one centre, one axis, and its widest and narrowest circles at
`R + r` and `R − r`. So a pair that differs in any of these differs somewhere,
and falls through to `ACrossingBetweenAConeAndAnotherCurvedSurface` or
`ACrossingBetweenATorusAndAnotherCurvedSurface` exactly as every pair did
before. A pair one ulp apart in any of them is two surfaces:
`a_cone_or_ring_a_hair_off_its_own_reverse_is_another_surface`.

**No `Policy` is asked.** Nothing here rounds, so a tolerant policy has nothing
to read as zero; a near-equal pair being `SameSurface` is exactly what this
must never say. The pipe's `SameSurface` goes through `ask` and a policy, and
for pipes that is §118.11's business, not this section's.

**The directions are decided in whole numbers and not by the cascade.** The
house question for "along one line" is `squared_length_of_a_cross_product`, and
its exact level refuses a product whose low half would be subnormal. A unit
direction reaches that with two components below about `2⁻⁴⁸⁵`, and
`(2⁻⁶⁰⁰, 2⁻⁶⁰⁰, 1)` against its reverse is one line the cascade answers
`Undecidable` for — measured in
`a_cone_about_a_direction_the_cascade_cannot_square_is_still_one_cone`, which
asserts the cascade's `Undecidable` before it asserts `SameSurface`. Taking only
the cascade's `Zero` would have refused that pair by name: a **gap** in
§16.10's sense, since the answer exists. Each component of `u × v` is
`a·b − c·d`, and whether two products of `f64` are equal needs no range at all:
each is a whole number below `2¹⁰⁶` times a power of two, which `i128` holds.
`two_directions_lie_along_one_line` does that, and
`two_directions_along_one_line_are_told_in_whole_numbers_at_any_size` covers
the smallest subnormal, `2¹⁰⁰⁰`, and a pair whose rounded `f64` cross product
is exactly zero and which is **not** along one line.

### 357.3 What the frustum union gives now, and a measure finding

`crates/rcad-topo/tests/two_frustums_stacked_on_one_cone.rs`: frustums on the
cone of apex `0`, slope `0.5`, over `z ∈ [2, 4]` about `+z` and `z ∈ [4, 6]`
about `−z`. Both orders of the union answer **one solid of four faces** — the
caps at `z = 2` and `z = 6` and the wall in its two spellings, one face each —
and `Model::validate` passes. The joiner leaves the two spellings apart, as
§231.5 said it would; Q354 joins them for pipes only.

**Its volume is wrong, and the fuse did not make it so.** A throwaway probe in
that test file, taken out before committing, measured each frustum alone at
deflection `0.01`, certified
(`mise exec -- just t -p rcad-topo --test two_frustums_stacked_on_one_cone --no-capture`):

| frustum | wall about `+z` | wall about `−z` | true |
|---|---|---|---|
| `z ∈ [2, 4]` | 14.642664 ± 0.208457 | **12.090699 ± 0.781295** | 14π/3 = 14.660766 |
| `z ∈ [4, 6]` | 39.761317 ± 0.352550 | **35.460664 ± 0.968026** | 38π/3 = 39.793507 |

The two spellings describe the same corners, caps and seam; only the wall's
cone and the circles' coedge directions differ. Named about the axis pointing
away from it, a frustum measures short by more than the bound `measure`
publishes for it. The union measured 48.473055 ± 1.595028 lower-first and
48.481776 ± 1.598270 upper-first, against 52π/3 = 54.454273. The test does not
assert a volume for that reason. This is not in this lane's files; it is a new
row (357.5).

### 357.4 The torus: reachable, and refused further on

A torus turned round is reachable through the public API: `SolidBuilder` takes
any `Torus`, and a quarter bend about `−z` builds and measures right —
19.706455 ± 0.398161 against 19.706528 ± 0.398162 for the same bend about `+z`,
from the same throwaway probe, where turning the axis round turns round the
cap circles' coedges and leaves the wall's shell orientation alone.
`two_quarter_bends_on_one_ring_turned_round_get_past_the_crossing_either_way_round`
unites a bend from `+x` to `+y` about `+z` with one from `+y` to `−x` about
`−z`. Before this section both orders were refused
`ACrossingBetweenATorusAndAnotherCurvedSurface` (checked by putting the
`one == other` guard back); now both are refused
`WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, at the corner where the two
spellings and a cap meet — everything past `intersect` still keys a surface by
the numbers it holds, so the corner sees two tori. **The same pair written in
one spelling does not answer either**: `ResultIsNotManifold { count: 4 }`. So
there is no answer for the turned pair to be compared against yet, and the test
pins where it stops rather than an answer.

### 357.5 What this leaves

- **One pin outside Q355's files moved with it, in Q502.**
  `two_cones_holding_the_same_numbers_are_one_surface`
  (`crates/rcad-geom/tests/the_seven_shapes_a_plane_cuts_a_cone_in.rs`)
  asserted the cone about `−z` is refused against the cone about `+z`, and
  with 357.2 gets `Ok(SameSurface)`; Q355's lane found it the only failure in
  a `--no-fail-fast` run of the workspace (2725 of 2726 pass). Its doc comment
  said calling the pair `SameSurface` "would need a canonical spelling that
  `transformed` could not round away". It does not: 357.2 compares the two
  spellings exactly without choosing either. Q502, lane
  `lane-q502-a-cone-turned-round-is-one-surface`, merged Q355's branch at
  `17c1764e`, flipped the assertion to `Ok(SameSurface)` and replaced that
  paragraph with one saying what the test pins now.
- **The measure finding of 357.3**: a cone face named about the axis pointing
  away from it measures short beyond its own bound. A trace row for
  `measure`/the mesher, with the four numbers above as its reproducer.
- `walk`'s cone arm is still guarded `one == other`
  (`crates/rcad-geom/src/walk.rs` line 727), so a closed run a caller names
  between two spellings of one cone is refused by name there. The frustum
  union answered, so no fuse here reached that refusal; it should take the
  same test when something does.
- The joiner, for cones and tori: Q354 is pipes only.
- The one-spelling stacked bends refusing `ResultIsNotManifold { count: 4 }`,
  which is not about spellings at all.
