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
(`mise exec -- just t -p rcad-topo --test merged -E 'test(/^two_frustums_stacked_on_one_cone::/)' --no-capture`):

| frustum | wall about `+z` | wall about `−z` | true |
|---|---|---|---|
| `z ∈ [2, 4]` | 14.642664 ± 0.208457 | **12.090699 ± 0.781295** | 14π/3 = 14.660766 |
| `z ∈ [4, 6]` | 39.761317 ± 0.352550 | **35.460664 ± 0.968026** | 38π/3 = 39.793507 |

The two spellings describe the same corners, caps and seam; only the wall's
cone and the circles' coedge directions differ. Named about the axis pointing
away from it, a frustum measures short by more than the bound `measure`
publishes for it. The union measured 48.473055 ± 1.595028 lower-first and
48.481776 ± 1.598270 upper-first, against 52π/3 = 54.454273. The test did not
assert a volume for that reason. This was not in this lane's files; it became
a new row (357.5), and 357.6 is its fix.

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
  away from it measures short beyond its own bound. Traced in Q510 and fixed
  in Q560: 357.6.
- `walk`'s cone arm is still guarded `one == other`
  (`crates/rcad-geom/src/walk.rs` line 727), so a closed run a caller names
  between two spellings of one cone is refused by name there. The frustum
  union answered, so no fuse here reached that refusal; it should take the
  same test when something does.
- The joiner, for cones and tori: Q354 is pipes only.
- The one-spelling stacked bends refusing `ResultIsNotManifold { count: 4 }`,
  which is not about spellings at all.

### 357.6 The measure finding was `Cone::point_at`, on the nappe with negative `along`

*Queue row Q560, from Q510's trace. Lane
`lane-q560-a-cone-point-read-back-on-the-other-side`, off `d578df1b`.*

`Cone::coordinates_of` reads the angle off the direction across the axis,
whichever nappe the point is on, and `along` straight off the axis. The cone
about `−z` has `along < 0` everywhere above its apex, so the frustum walls of
357.3 named about `−z` sit wholly on that side. `Cone::point_at` scaled the
step across by `slope · along`, which is negative there, so it put every such
point **half a turn round** from where it had been read: `(1, 0, 2)` on the
cone about `−z` read `(0, −2)` and came back `(−1, 0, 2)`. The mesher reads a
wall's boundary with `coordinates_of` and puts every point it adds back through
`point_at`, so the wall came out folded. **The fix is to step across by
`Cone::radius_at(along)`**, which is never negative; `round` then names the
same direction across on both nappes and `point_at` undoes `coordinates_of`
on both.

Rejected: flipping the angle `coordinates_of` reads when `along < 0`. That
puts the seam of that nappe at `round = π`, and §25.2 fixes it at zero.

**Pinned.** `a_point_on_either_half_of_a_cone_comes_back_to_where_it_was`
(`crates/rcad-geom/tests/a_round_trip_through_a_surfaces_own_two_coordinates.rs`)
places points on both nappes **by hand**, never through `point_at`, for the
cones about `+z` and `−z` and the same two turned and shifted. It has to: a
round trip that starts in `point_at` passes the old code, because
`coordinates_of` reads the turned angle and `point_at` turns it back. On the
base the first point it tried with a negative `along`, `(0.375, 0, −0.75)` on
the cone about `+z`, moved `2⁴⁴` times the bound.
`two_frustums_stacked_on_one_cone.rs` now asserts each frustum's volume inside
its bound in both spellings, and the union's in both orders, at deflection
`0.01`; on the base the frustum `z ∈ [2, 4]` about `−z` measured
12.090699 ± 0.781295 against 14π/3 and the union lower-first
48.473055 ± 1.595028 against 52π/3, both failing.

**What the frustums measure now**, from a throwaway probe in that test file,
taken out before committing
(`mise exec -- just t -p rcad-topo --test merged -E 'test(~two_frustums_stacked_on_one_cone::q560_probe)' --no-capture`):

| | `d = 0.01` | `d = 0.001` | true |
|---|---|---|---|
| `z ∈ [2, 4]` about `+z` | 14.642664 ± 0.208457 | 14.658959 ± 0.021050 | 14.660766 |
| `z ∈ [2, 4]` about `−z` | 14.643758 ± 0.208463 | 14.658955 ± 0.021055 | 14.660766 |
| `z ∈ [4, 6]` about `+z` | 39.761317 ± 0.352550 | 39.790319 ± 0.034920 | 39.793507 |
| `z ∈ [4, 6]` about `−z` | 39.761238 ± 0.348103 | 39.790283 ± 0.034920 | 39.793507 |
| union, lower first | 54.403210 ± 0.558633 | 54.449088 ± 0.056224 | 54.454273 |
| union, upper first | 54.403215 ± 0.558633 | 54.449086 ± 0.056224 | 54.454273 |

Every row is inside its bound. The `+z` rows are the numbers 357.3 gave,
unchanged, as they should be: nothing about `+z` has a negative `along` here.

**What the gate shows moving: nothing pinned.** `just gate quiet` at
`50326260` passed, 2761 tests. The same `point_at` is reached by three more
callers, each through `coordinates_of` and back, and each was wrong the same
way for a point with a negative `along` until now:

- **the mesher's cut points** (`crates/rcad-topo/src/mesh.rs`, through
  `Surface::point_at`) — the folded frustum above;
- **`back_onto_the_surface`** (`crates/rcad-topo/src/fuse/select.rs`), which
  put every candidate for a piece of such a wall half a turn round before
  classifying it. §350.1's family 1 rests on "every corner of a piece is on its
  face's nappe or at the apex, so `along` is at least zero"; a cone named
  about the axis pointing away from its face breaks the premise, and with this
  fix the round trip no longer needs it — the landing point keeps its
  direction across on either nappe. §350.1 is not in Q560's files.
- **`how_far_off`** (`crates/rcad-step/src/brep.rs`), which reports how far a
  refused corner is off its face and would have said about a diameter for a
  corner at negative `along`. It is a report and not a decision.

None of the three has a test here that fails on the base; the union above is
the one fuse in Q560's files through `back_onto_the_surface` on such a wall,
and it answered the same faces before the fix. A shape meant to show the
select path — the frustum `z ∈ [4, 6]` about `−z` cut in half by the block
`[−10, 0] × [−10, 10] × [2, 12]`, as `a_frustum_cut_in_half_down_its_axis.rs`
cuts one — **refuses `TwoCurvesTouchWithoutCrossing` in all six fuses, before
and after the fix**, where the same frustum about `+z` answers all six inside
their bounds. A throwaway probe in the same test file showed it; it is a new
row and not this one.

`∂u × ∂v` of the new chart still points out of the material on both nappes:
`every_surface_but_a_torus_crosses_its_two_coordinates_out_of_the_material`
samples a cone at `along = ±0.75, ±2` and passes.
