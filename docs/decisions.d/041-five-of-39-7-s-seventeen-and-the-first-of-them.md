## 41. Five of §39.7's seventeen, and the first of them is still open two units deeper, recorded 2026-09-02

*Written by the gate that closed M5's first fan. §38 did this for §37's four
defects. Three lanes ran in sequence on one branch and closed §39.7's items 1, 2,
3, 4 and 7 — the five §39.7 ranked first. This section is the record, and the
audit that had to happen before the record could be written.*

**Every load-bearing number below was re-derived at this gate, not copied from a
lane's report.** §39.3 records that the claim audit has found a real defect on
five consecutive milestones; §16.10 records why. What is new here is that the
same discipline was pointed at the lanes' own reports, and it caught three
things: a numeral that was wrong, an impossibility argument that ruled out the
half of a family that does not work and read as ruling out the half that does,
and **the headline item, which is closed at the one depth its two new tests pin
and open at another**. §41.6 is that one. A gate that finds nothing has usually
not looked.

**A fourth lane then took the two open items this section left and closed both.**
§41.12 is the record of that, and it corrects two numerals of §41.6's own — a
gate that audits three lanes' reports and not its own has still only audited
three reports.

| §39.7 | what it was | now |
|---|---|---|
| 1. a ball cut anywhere but through a pole is refused, and a rigid motion of both operands changes the answer | one wire made of one closed edge was **assumed** to run right round the surface, and on a ball a closed edge is any circle at all | **answered** at the depth §39.3 measured — all three axes, all 21 angles of the sweep. ~~**Not closed in general**: §41.6~~ **corrected by §41.12: the cap now answers at every depth on all three axes.** What is left is the piece on the *other* side of the cut, which is §41.7's second row and a different mechanism |
| 2. the fuse bakes a placement in and the mesh floor goes up 77-fold | the crossing term is charged against how far the face reaches in the frame it is *stored* in, and after a fuse "stored" is "world" | **answered** — the mesher shifts a far-off face into a frame where its numbers are small, exactly, and the 76.85 is 1.4444 |
| 3. three panics on a caller's bytes, demonstrated | `validate` on a closed edge between two flat faces, `mesh`/`measure` on a closed edge with two different ends, `mesh` on a wire fanned onto the wrong surface | **all three refuse by name** |
| 4. the rule that generalises them: a `# Panics` clause on a `pub` item needs a demonstration that the arenas cannot hold the shape | fifty-eight such clauses stood unswept | **swept, all fifty-eight.** Five more real ones found and fixed; two suspects the brief named were doc-only and are now cited to a demonstration |
| 7. `meet_of_three_planes` refuses three perpendicular planes written at `1e-200` | the determinant's one surviving term underflows past the smallest subnormal | **answered** — a retry with each plane scaled by its own exact power of two |

**Two of the five were closed by reconditioning the arithmetic rather than by
widening a bound, and that is now a pattern with a name.** Item 7 divides each
plane's four coefficients by a power of two before the determinants; item 2
subtracts an exact offset off a face's coordinates before the mesh is worked out.
Neither moved a published constant. §18 said scaling is a technique and not only
an oracle; these are the third and fourth times it has been the whole of a fix,
and §41.5 finds a fifth candidate sitting unfixed in `Isometry`. **When a bound
looks too wide, the question to ask first is what the arithmetic was handed, not
what the constant says.**

### 41.1 A ball cut off its polar axis, and the confident wrong answer underneath it

§39.7 item 1 was first on the list because it is a **§19 violation** — turning
ball and slab together flipped a working answer into a refusal at
`acos(1/4) = 1.3181` radians — and §19 is the property this kernel sells.

**The root cause is one line, and it is not the one §39.3 was looking at.**
`WalkedLoop::goes_right_round` was set from `wire.closes_on_one_edge()`, a
*structural* fact, on the argument that *"one coedge whose two ends are one corner
runs right round the curve its two faces cross in, which on a periodic surface is
right round the surface."* The second half is false on a sphere: a closed edge on
a ball is **any circle at all**, and most circles encircle no pole. So a cap that
does not go round the ball was sent to `cut_a_cap_up_to_its_pole`, which found no
pole to fan to along `y` and the **wrong** pole along `x`. Measured at this gate
on `main`, with the two raise sites of the name instrumented apart:

| axis | on `main` | where the refusal is raised |
|---|---|---|
| `z` | `Ok`, one solid | — |
| `y` | `ACurvedFaceGoesRightRoundWithNoSeam` | `mesh.rs`, **not** `arrange.rs` |
| `x` | `CannotMeshAFace { BoundaryCrossesItself { at: 4 } }` | the triangulator, handed four patches fanned to a pole the cap does not reach |

`goes_right_round` is now **measured** rather than asserted — the walk has to gain
at least half a turn of longitude all the way round — which can only turn `true`
into `false`, so no wire that meshed before takes a new path.

**And that uncovered the deeper defect.** With both caps down the generic path
they came back with the right volume and were walked **opposite ways round** in
the surface's own two coordinates: signed chart area `+3.4748599566989133` at
`y = 1` and `−3.4748599566989133` at `x = 1`, same magnitude, opposite sign, for
two caps that are one cap turned a quarter turn. The sign of that area *is* which
side of the loop the material is on, and it disagreed with itself.

The cause is §23.8's own failure mode one function over: `mesh.rs` re-derived
"which way is this closed edge stored running" by **summing the longitude all the
way round the edge** and leaving the evaluator's order alone when the sum came out
zero, while the fuse chose the coedge orientation from the **analytic** rule
(`a_closed_crossing_is_walked_the_way_it_is_stored`). The sum is exactly zero and
not merely small: on a ball of radius `r` at the origin cut by `x = d`, the
crossing tangent is `p × x̂` and the longitude direction is `ẑ × p`, and

```
(p × x̂)·(ẑ × p) = (p·ẑ)(x̂·p) − (p·p)(x̂·ẑ) = z·d
```

so the sign flips at `z = 0` and the two halves cancel. For a `z = d` cut the same
expression is `−‖p × ẑ‖²`, negative everywhere — **which is why exactly one axis
ever worked**. The summing rule is deleted; both sides now ask
`the_way_a_closed_edge_is_stored_running`, which is the authority the fuse writes
with and `Model::validate` reads back with.

**Under that sat a confident wrong answer, and §4.1 ranks one below both crashing
and refusing.** A loop on a ball bounds two pieces and the triangulator fills the
chart-interior of whichever it is handed; when the material is on the *other*
side — the piece holding both poles — it silently filled the complement.
Reproduced at this gate, with the one guard that stops it disabled and nothing
else changed:

```
two balls of radius 2, centres 2 apart, Rule::Union
  Ok, 1 solid, volume 10.396094933134792
  the union is 18π  = 56.548667764616276
  the lens   is 10π/3 = 10.471975511965978
```

It built the **lens** and called it the union. That is the same species as
§38.1's dropped ball and §39.3's `Ok`-with-an-invalid-solid — the third success
return in two milestones that the audit caught and nothing else did. It is now
refused `ACurvedFaceGoesRightRoundWithNoSeam` by name, gated on a
one-closed-curve wire on a surface that pinches so that a legitimately inside-out
whole-surface piece — §38.1's bubble — still passes.

**Why not the frame-change fix**, which would re-chart the ball so its pole is the
cut circle's axis and close the both-poles row as well: it cannot be expressed.
`Sphere::transformed` moves only the centre — a ball has no frame of its own by
design — so a rotated chart cannot be handed to `TrimmedDomain::new(*stored, …)`.
Doing it means changing `rcad-geom`'s `Sphere` or `rcad-mesh`'s `TrimmedDomain`,
and `rcad-mesh` is at **0 files changed through M4 and this fan**.

### 41.2 The 77-fold mesh floor, closed by neither of the two fixes §39.7 offered

§39.7 item 2 offered two families: keep the placement on the solid through a fuse,
or widen `rcad-mesh`'s bound. **Neither landed.** What landed is item 7's move one
layer up: recondition the arithmetic before doing it.

Re-measured at this gate on a clone at `2f6ee28`, before the fix, and on the
branch tip after it — the same four public calls both times:

| | floor | asked `1e-5` |
|---|---|---|
| a bar of radius 1, five tall, at `z = 1e9` | `3.996804731620784e-6` | meshes |
| after a `Difference` against a cube fifty units away that cuts nothing, before | `3.0716170532259685e-4` (**76.85181687573629×**) | refused |
| the same, now | `5.773161581679176e-6` (**1.4444442421729329×**) | meshes |

against the closed forms

```
(POINT_ALONG_A_CROSSING_ROUNDOFF / WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO + POINT_ROUNDOFF) / POINT_ROUNDOFF
    = (256·ε/0.1875 + 18·ε) / (18·ε) = 76.85185185185185
(POINT_ROUNDOFF + SURFACE_AFTER_SHIFTING_ROUNDOFF) / POINT_ROUNDOFF
    = (18 + 8)/18 = 26/18 = 1.4444444444444444
```

Six figures either way; the residual is the crossing term charged against the
bar's own five units, `1e-13`-sized against a `4e-6` floor. Every constant in
those two lines was read off the tree at this gate: `POINT_ALONG_A_CROSSING_ROUNDOFF`
`= 256·ε`, `Isometry::POINT_ROUNDOFF` `= 18·ε`, `WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO`
`= 0.1875`, `SURFACE_AFTER_SHIFTING_ROUNDOFF` `= 8·ε`.

**`rcad-mesh` produces none of it**, and that is checked rather than argued: the
refusal is `rcad_topo::Error::DeflectionBelowWhatAMeshCanReach`, raised in
`room_a_mesh_has_to_work_in` before a patch is handed over, and this gate read it
straight out of the error. `rcad-mesh`'s own
`CannotTriangulate::NoMeshCanGetThatCloseToTheSurface` is never reached.

**What landed.** `the_frame_this_face_is_meshed_in(face)` is usually the frame the
face is stored in, and for a face at **no placement at all** whose numbers are far
from the origin it is that frame shifted until they are small.
`a_shift_that_loses_nothing` moves a coordinate only when every number the face
holds in it has one sign and the largest is no more than twice the smallest, and
then by the smallest — **Sterbenz's condition**, `a − b` exact whenever
`b/2 ≤ a ≤ 2b` — so the face worked out in the shifted frame is the very same
face. A flat face is never shifted. Instrumented at this gate on the fused bar,
the surface `rcad-topo` hands `rcad-mesh` for its wall:

```
world       Cylinder { axis origin (0, 0, 1000000000), direction (0,0,1), radius 1 }
handed over Cylinder { axis origin (0, 0, 0),          direction (0,0,1), radius 1 }
frame       translation (0, 0, -1000000000)
```

An `EdgeRun` had to learn to be shifted with it: a ball's seam is *the plane its
meridian lies in* plus *the half-space picking one half of it*, and shifting the
ball while leaving the seam behind picks a piece of a crossing that is not there.
`HalfSpace::shifted`, `Picker::shifted` and `EdgeRun::shifted` take a `Vector` and
not an `Isometry` on purpose — a half-space stores its normal unscaled so an
exactly-square picking normal stays exactly square (§32.3), and a turn scales that
away.

**One published bound is new and it makes the floor higher, not lower.**
`rcad_geom::SURFACE_AFTER_SHIFTING_ROUNDOFF = 8 · f64::EPSILON`. §15.6's search
for it, re-run at this gate over 20 000 draws sixty octaves wide, worst slip as a
multiple of `ε` times the size:

| plane | half-space | ball | pipe | the bound |
|---|---|---|---|---|
| 1.5928602657890887 | 1.3661359484458944 | 0.4990673137264969 | 0.4990673137264969 | 8 |

Nothing existing was widened: `POINT_ALONG_A_CROSSING_ROUNDOFF`,
`Isometry::POINT_ROUNDOFF`, `CORNER_AFTER_MOVING_ROUNDOFF`,
`Surface::ROUND_TRIP_ROUNDOFF`, `WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO` and
`WHAT_THE_BOUNDARY_TAKES_OF_THE_BUDGET` are bit for bit what they were.

**Why the two families §39.7 named are not what closed it.** Family (A) — keep the
placement on the solid — is blocked in the tree twice over, and both blocks are
demonstrations rather than paragraphs. `fuse/bake.rs`'s `place_a_corner` asserts
outright that the faces round one corner are at one placement, so a fuse over two
solids at two placements must pick a frame that is in general neither input's; and
picking the first argument's is closed by
`swapping_the_two_sides_of_a_curved_pair_changes_not_one_bit`, which compares the
two orders **bit for bit**. Storing an answer's vertices through `F⁻¹` and reading
them back through `F` is a round trip that moves them, and `fuses_that_must_agree`
compares face planes and wire rings bit for bit across commutativity,
associativity and transform invariance. Family (A) buys mesh accuracy by spending
answer accuracy, which is backwards for a kernel whose §19 property is bit-for-bit
agreement. **A third blocker was offered and called decisive, and this gate does
not accept it** — §41.4 item 8.

Family (B) splits in two and only one half is closed. Take the bound as tight as
its own derivation allows and hand the boundary the whole deflection:

| what is assumed | ratio left |
|---|---|
| as shipped, `256·ε` and an 18.75% share | 76.85 |
| `POINT_ALONG_A_CROSSING_ROUNDOFF` cut to `34·ε`, its own derived figure | 11.07 |
| that, and the whole budget to the boundary (`0.375`) | 6.04 |
| that, and the derived figure halved again | 3.52 |

The floor of the input is `18·ε·L` and a conic worked out at `L` cannot go under a
few `ε·L` however honest the constant, so **no member of the constant-widening
half reaches 1** — arithmetic, not opinion. The other half of §39.7's own wording,
*"charges the roundings the surface actually has rather than the largest number in
it"*, is a different fix and it **does** reach 1: see §41.7's fourth row.

### 41.3 Fifty-eight `# Panics` clauses, swept, and a frozen crate unfrozen

§39.7 item 4's rule: *a `# Panics` clause on a `pub` item needs a demonstration
that the arenas cannot hold the shape, not an argument that geometry forbids it.*
`Model` is `Deserialize` and the arenas hold whatever the bytes say.

**Counted at this gate rather than taken on trust.** On `main`, `# Panics`
immediately above a strictly-`pub` item:

| crate | sites | verdict |
|---|---|---|
| `rcad-geom` | 21 | three real, all fixed; two doc-only, cited to a demonstration |
| `rcad-topo` | 27 | five real, all fixed |
| `rcad-predicates` | 10 | clean; one boundary gap flagged, not a panic (§41.8) |
| `rcad-mesh` | 0 | — |
| `rcad` | 0 | — |
| | **58** | |

Fifty-three remain on the branch. **The count is 58 only if `pub(crate)` is
excluded; counting those too gives 76, so eighteen `pub(crate)` clauses were not
swept.** That is defensible — a `pub(crate)` panic is the internal invariant
CLAUDE.md says to keep — but it is the sweep's boundary and is written down here
rather than left implicit.

The three §39.7 item 3 named, all now named `Err`s: `Invalid::ClosedEdgeBetweenTwoFlatFaces`
out of `validate` (the old `.expect("two planes cross in a straight line, which
never closes")`), `Error::AClosedRunDoesNotComeBackToWhereItStarted` out of
`rcad-geom`'s `NamedRun::new`, and `Error::AWireDoesNotFanIntoALoopToMesh` plus a
reused `Invalid::FaceHasNoWires` out of `mesh`'s `hand_over`. The five the sweep
found on top: `NamedRun::the_whole_of`, `Segment::transformed`,
`EdgeView::segment`, `Model::set_edge_approximation`, `Model::set_face_approximation`.

**`EdgeView::segment` is the one worth reading twice.** Its clause said
*"`Model::validate` rejects [same-point ends], so reaching it means the model was
already broken"* — and that is **false**. `check_edges` refuses same-point ends
only for `EdgeRun::Straight`; a closed edge is *supposed* to have `start == end`,
every round bar's two cap circles are exactly that, and they are ordinary and
`validate()`-sound. `segment()` never looked at the `EdgeRun` and panicked on the
commonest closed edge in the kernel. It returns `Result<Option<Segment>>` now.
This is not the `Deserialize` species at all — it is a `# Panics` clause that was
simply wrong about a shape the builder makes in two lines — and it is the argument
for sweeping the clauses rather than only the byte-reachable ones.

Four public signatures changed with them: `NamedRun::new`, `NamedRun::the_whole_of`
and `Segment::transformed` return `Result`, and `EdgeView::segment` returns
`Result<Option<Segment>>`. Four new public variants landed —
`rcad_geom::Error::{AClosedRunDoesNotComeBackToWhereItStarted,
ACrossingPieceHasNoEndsAndNothingPickingIt}` and
`rcad_topo::{Error::AWireDoesNotFanIntoALoopToMesh, Invalid::ClosedEdgeBetweenTwoFlatFaces}`
— and `just check-wildcards` did **not** move off 76. That is §40 working as §40.6
said it would, not §40 failing: no wildcard site enumerates every variant of the
two enums that grew, because cross-crate matches on `rcad_geom::Error` name one
variant and forward the rest.

**`rcad-predicates` is no longer frozen.** §39.7 item 7's fix is one file,
`planes.rs`: on refusal, retry with each plane scaled by the exact power of two
that brings its own largest coefficient into range. A plane names the same points
after any nonzero scaling of its four coefficients, and Cramer's numerator and
denominator determinants are each **linear in every row**, so scaling row `i` by
`sᵢ` multiplies both by `s₁s₂s₃` and the ratio cancels exactly — row by row, which
is why three unrelated exponents work where `orient3d`'s degree-three homogeneity
retry needs one shared factor. A plane with no exact scaled copy sinks the retry
and the original refusal stands, correctly: that is a limit.

**§39.7 item 7 said this was the opening gate's item and not a lane's**, on §31.5
and §15.9 — *"a change reopens this gate and stops the fan"*. It was taken by a
lane, and M5 has no opening-gate section. The change itself is audited and green;
the process rule was not followed, and this sentence is the record of that rather
than a quiet gap between §40 and §41.

**The measured rate did not move by one fuse**, which is the number to read.
`every_point_where_three_surfaces_meet_is_on_all_three_of_them`, re-run here:

| family | triples | corners | certified `Zero` on all three | refused | widest box |
|---|---|---|---|---|---|
| control, lattice | 2050 | 571 | 315 | **0.00%** | 2.1568020616793115e-16 |
| leaning | 2050 | 1281 | 0 | **0.00%** | 1.7763568394002505e-14 |
| written at `2^k` | 2050 | 646 | 240 | **10.83%** (222) | 3.0246243428348286e-16 |
| turned | 2050 | 682 | 0 | **0.00%** | 2.410769996328908e-15 |

Every figure is §39.5's, to every digit §39.5 printed. The reason is structural
and was checked rather than asserted: every production call site of
`meet_of_three_planes` goes through `Plane::coefficients()`, and every `Plane`
constructor normalizes the normal to length one, so a uniformly-tiny coefficient
array cannot reach it from real geometry. The fix moves nothing a generator
draws, exactly as §39.7 predicted, and that is now measured.

### 41.4 Every claim the lanes made that this gate could not verify, or found false

§16.10's rule applied to the reports rather than to the tree. The lanes' own
briefs record that lane claims in earlier milestones were repeatedly slightly
wrong in ways that misdirected the next lane, so nothing below was copied.

**Verified by re-derivation** — the mesh floors and both ratios, on a clone at
`2f6ee28` and on the tip; the three cap volumes and the 21-angle sweep; the
`10.83%` and its three sibling rows to full precision; all seven
`fuses_that_must_agree` families and their named breakdowns against §38.7; the
58 `# Panics` sites; the two-ball wrong answer, reproduced with the guard
disabled; the `SURFACE_AFTER_SHIFTING_ROUNDOFF` search; and the two refusal sites
of §39.7 item 1 on `main`.

**Found false or overstated:**

1. **The shift in this milestone's headline case is `(0, 0, −1e9)`, not
   `(0, 0, 1e9 − 2.5)`.** `add_round_bar(1.0, 5.0, …)` runs from the placement to
   five above it rather than centring on it, so the bar's stored numbers are
   `z ∈ [1e9, 1e9 + 5]` and the smallest is the billion exactly. Instrumented
   above. The conclusion that rests on it — that the slip of the neighbouring cap
   planes is *exactly* zero here, so 1.00 is reachable — survives unchanged and
   is in fact easier: `1e9 ⊖ 1e9 = 0` and `1000000005 ⊖ 1e9 = 5`, both exact.
2. **"Family (B) cannot close it" is true of half of family (B).** §39.7 item 2
   words (B) as *"widen `rcad-mesh`'s bound so it charges the roundings the
   surface actually has rather than the largest number in it"*, which is two
   fixes in one sentence. Widening the *constants* is exhausted at 3.52 and the
   table above is the demonstration. Charging *the roundings the surface actually
   has* is measuring the slip instead of bounding it — and that is precisely the
   fix that reaches 1.00 and is deferred as §41.7's fourth row. The
   impossibility is real and it is narrower than it reads.
3. **"A face spreading over more than a factor of two gets no shift" is a gap,
   not "a limit for the shift and a gap for the floor".** A limit is a property of
   an *answer*, not of a technique. The answer — a finer mesh of a fused solid
   spanning `z ∈ [1e9, 3e9]` — exists. What is true is that *this* shift cannot
   reach it exactly, which is a sentence about the fix. The vocabulary is worth
   keeping clean because §39.4 has thirty-three rows sorted by it.
4. **§39.7 item 2 does not name the wrong file; it names a real §16.10 gap that
   is not this one.** `triangulate.rs:860` does call its own bound a gap and does
   carry the escape sentence. What is wrong is the inference: closing that bound
   would have moved the measured figure by exactly zero, because `rcad-topo`
   refuses first under its own name. The correction is to the causal claim, not
   to the citation.
5. **The `# Panics` sweep counted 58 and the tree holds 76**, the difference
   being `pub(crate)` items. Written down above rather than left as a number
   nobody can reproduce.
6. **The §16.11 miscitation is fixed in two places and stands in fifteen.**
   §16.11 is about the dynamic-range check being a ratio; it says nothing about a
   caller's bytes, nothing about panics, and nothing about `validate` being the
   import boundary. The rule being cited is CLAUDE.md's own tier table and §4.1,
   with §4.3 for where the sweep runs. Fifteen doc comments in `rcad-topo` still
   cite §16.11 for it (`fuse/arrange.rs`, `mesh.rs` ×5, `classify.rs`,
   `validate.rs` ×2, `error.rs` ×3, `holds.rs`, and two test files), and so do
   §38's opening paragraph, §38.6's fourth row, §39.4's closing note and §39.7
   item 3.
7. **§38.3's table of what a handed-in direction buys is not current.** Lane 2
   tried to reproduce it and could not get a consistent pair out of the fallback:
   `unwrap_or(true)` makes both faces walk the edge the same way and trips the
   arena's own rule that two coedges run opposite ways, and falling back on
   `material_side` alone does the same because the two faces measure it against
   different partners. **This gate did not re-measure it either.** The three rows
   of §38.3's table should be treated as unverified until someone reproduces
   them; what *is* re-measured is that the refusal itself still stands
   (§41.7).
8. **"STEP solids arrive with no placement at all" is a paragraph where CLAUDE.md
   asks for a demonstration, and it was offered as the decisive blocker.** Family
   (A)'s third block is that a solid read out of a STEP file arrives in world
   coordinates with nothing left on it to say where it was put, so a fix that only
   helps kernel-built-then-fused solids leaves the importer with the whole 77.
   **That is a claim about a milestone that has not started, resting on an importer
   nobody has written.** CLAUDE.md asks for the family of fixes considered and why
   that family is the only one, and for "no caller hits this" to be checked against
   the public API rather than today's callers. The family it does not name is *an
   importer that factors a placement out of what it reads*: the format carries an
   `AXIS2_PLACEMENT_3D` on every surface and an `ITEM_DEFINED_TRANSFORMATION` on
   every assembly step, `Model::place_solid` is public, and nothing in this kernel
   stops an importer using either. **The verdict: the conclusion stands and this
   reason does not.** Blocks 1 and 2 are demonstrations in the tree — an assert in
   `bake.rs` and a bit-for-bit test — and either alone closes family (A). The third
   should be struck rather than called decisive, and it is worth noticing that it
   argues *for* the fix that landed rather than against family (A): what the shift
   is built for is precisely a face at no placement at all.

**And one claim that is true for a reason its own argument does not name.**
Lane 1 checked `Cylinder::basis` and `Plane::basis` — both said *"the constructors
do not allow that"* with no demonstration, §39.7 item 4's discredited shape — and
found the claim held: `Axis` and `Plane` both carry `#[serde(try_from = …)]`, so
`Deserialize` routes through the validating constructor. The argument names two
routes in and **the tree has three**. `Axis::transformed` and `Plane::transformed`
deliberately do *not* re-normalize, and both are reached with an `Isometry` that
came off the wire. The claim survives because `Isometry`'s own `try_from`
normalizes its quaternion — a fact two types away from where the sentence sits,
which is the exact shape §39.3's rule was written about. It is checked here rather
than left implicit, and checking it turned up §41.5.

### 41.5 A rotation off the wire that is not the rotation the bytes name

**Found at this gate, not fixed, and it is a wrong answer rather than a refusal.**
**Fixed by the fourth lane — §41.12, which also found the other half of it: the
same one line refuses a rotation whose squares *underflow*.**

`Isometry`'s `TryFrom<([f64; 3], [f64; 4])>` — which `#[serde(try_from = …)]`
makes the only route in on the wire, and `FaceUse.loc` makes reachable at every
placed face of a deserialized `Model` — checks that the four quaternion
coefficients are finite and that `norm()` is not zero, and then hands them to
nalgebra's `UnitQuaternion::from_quaternion`. When the **sum of their squares
overflows**, `norm()` is infinite, the zero test passes, and what comes back is
the **identity rotation**. Measured here, a quarter turn about `x` written at
growing scales:

```
[1e153, 1e153, 0, 0] → ẑ moves to (0, −1.0000000000000002, −2.2e-16)   correct
[1e154, 1e154, 0, 0] → ẑ moves to (0, 0, 1)                            the identity
[1e308, 1e308, 1e308, 1e308] → ẑ moves to (0, 0, 1)                    the identity
```

`2 · (1e154)² > f64::MAX` and `2 · (1e153)² < f64::MAX`, which is the whole of the
boundary. A caller's bytes name one rotation and the kernel silently uses another
— §4.1's lowest tier, below both refusing and crashing, and the second wrong
answer this section records.

**A gap, and its fix is item 7's fix one type over.** Dividing all four
coefficients by the exact power of two that brings the largest into range costs
not one bit and reaches the answer, exactly as `meet_of_three_planes` now does for
a plane's four coefficients. Recorded and not built: this gate writes
`docs/decisions.md` and nothing else, and the fix wants its own test and its own
lane. **Consumer: any `Model` that has been through a file**, which is what M5 is.
**Built, and it is that fix. §41.12.**

### 41.6 The §19 violation is closed at one depth and open at another

**This is the finding that should be read before the rest of this section.**
**And it is closed at every depth now — §41.12, which also corrects two numerals
below.**

§39.7 item 1's two new tests pin the same ball of radius four and the same
twenty-cube slab with its face **one unit** from the centre, on three axes and at
21 angles. Both pass, and at that depth the property holds. **At a deeper cut it
does not.** Measured at this gate — ball of radius four, a twenty-cube whose face
sits at `d`, `Rule::Intersection`, every volume read at a deflection of `0.002`,
against the cap's closed form `πh²(3r − h)/3` with `h = 4 − d`:

| `d` | square to `z` | square to `x` | closed form |
|---|---|---|---|
| 1.00 | `Ok` 84.798930 | `Ok` 84.801572 | 84.823002 |
| 2.00 | `Ok` 41.872603 | `Ok` 41.871236 | 41.887902 |
| 2.80 | `Ok` 16.278100 | `Ok` 16.278045 | 16.286016 |
| 2.82 | `Ok` 15.769081 | `Ok` 15.769041 | 15.776835 |
| **2.83** | **`Ok` 15.517572** | **`Invalid::VolumeIsNotPositive { volume: 0.0 }`** | **15.524900** |
| 3.00 | `Ok` 11.513343 | `VolumeIsNotPositive` | 11.519173 |
| 3.90 | `Ok` 0.123970 | `VolumeIsNotPositive` | 0.124617 |
| 3.99 | `Ok` 0.001166 | `VolumeIsNotPositive` | 0.001256 |

**The two columns agree to five figures right up to the row where one of them stops
answering.** Same two solids, same relative position, one quarter turn apart, and
one answers and one refuses. That is §19, and it is the sentence §39.7 item 1 was
put first for.

**It is not a regression.** On `main` the same `d = 3` cut square to `x` refuses
`VolumeIsNotPositive { volume: 0.0 }` too — it is the shallow cuts that used to
refuse `CannotMeshAFace` and now answer. The lanes made the branch strictly
better and the item is **not closed**.

**The cause is the sign-reading mesh and not the mesher this fan fixed.**
`nest_the_shells` reads a curved shell's sign by meshing it at
`as_coarse_as_a_mesh_gets()`, which is `Deflection::of(f64::MAX)`, and refuses
when that mesh encloses exactly nothing. A cap that holds a pole is cut into four
patches up to that pole whatever the deflection, so the `z` row survives to
`h = 0.01`; a cap that holds no pole goes down the generic path, and the coarsest
mesh of a cap of height `1.17` on a ball of radius four is empty. The `2.83`
boundary is `h/r ≈ 0.29` — not a sliver, not a degeneracy, and 5.8% of the ball.

**A gap.** The answer exists, is `15.5249`, and the very same shape turned a
quarter turn answers it at `15.517572`, within `0.05%`. The family of fixes is
named in §41.7's first row.

**And it means the catalogue row underneath it is understated by twenty times.**
That refusal was filed this fan as *"the lens between two balls of radius 2 three
apart… the lens is `π/4 = 0.785`"*. The same refusal takes a spherical cap of
**15.52** off a ball of radius four in three public calls. A row that describes a
gap by the smallest shape that reaches it is a row the next lane will size wrong.

**Two numerals of that paragraph are wrong and §41.12 is where they are put
right.** The lens of two balls of radius `r` a distance `d` apart is
`π(4r + d)(2r − d)²/12`, which at `r = 2` and `d = 3` is `11π/12 = 2.8798` and
not `π/4 = 0.7854` — so the cap understates by **5.4 times**, not twenty. The
point the paragraph is making survives the correction and the arithmetic under it
did not, which is the shape §16.10 is about. Both shapes answer now.

### 41.7 Every refusal these lanes leave standing, classified

CLAUDE.md's rule, and each row was re-checked against it here rather than taken
from a report — §41.4 item 3 is one that changed on the way.

| Refusal | Shape | | Demonstration |
|---|---|---|---|
| ~~`Invalid::VolumeIsNotPositive { volume: 0.0 }`~~ **withdrawn, §41.12** | **the sign-reading mesh encloses nothing**: an off-axis spherical cap under `h/r ≈ 0.29`, and the lens between two balls of radius two three apart | ~~**gap, and a §19 violation**~~ **closed** | §41.6's table. ~~`what_two_balls_that_really_cross_share_is_the_lens_between_them` pins the lens at `π/4`~~ — it is `11π/12`, and it now pins the **answer**. Both shapes answer: the fix is the finer sign-reading mesh this row named. No fuse of ordinary solids reaches this name any more; `validate` still raises it on a hand-built model whose faces point inwards |
| `ACurvedFaceGoesRightRoundWithNoSeam` | a face on a ball bounded by one closed curve with the **material outside it** — the whole ball less a disc; either half of a two-ball union; `Union`/`Difference` of a ball and a knife off its polar axis | **gap, and a §19 violation** — this row did not say so and §41.12 measured it | `what_is_round_two_crossing_balls_is_refused_for_want_of_a_seam`, `a_ball_cut_off_its_polar_axis_keeps_the_cap_and_refuses_what_is_round_it`. The answer is `18π = 56.549` for the two balls, and the piece on the *other* side of the very same curve is answered. Wants §38.3's machinery: a cut closed face with a seam and two poles of its own |
| `ACurvedFaceGoesRightRoundWithNoSeam` | nothing bounding a face on a plane or a pipe; one curve right round a pipe | **limit**, unchanged | §38.6's rows. No answer exists: such a face is unbounded |
| `DeflectionBelowWhatAMeshCanReach` on a fused or imported curved solid, at **1.4444×** the floor of the same solid unfused | a boolean that removes no material still costs the mesh 44% of its floor: the neighbours' bill, `(18 + 8)/18` | **gap** | `a_boolean_that_cuts_nothing_off_a_bar_does_not_make_it_much_coarser_to_mesh`. The shift is chosen from one face's numbers and is exact on them; the plane on the other side of an edge is moved by the same shift without being asked, and a plane's offset moves by a dot product, which rounds. **No shift makes that exact in general** — a product of two 53-bit numbers is not a 53-bit number — so the *bounding* family is exhausted at `4.5·ε·L`. What reaches 1.00 is measuring the slip instead of bounding it, which is exactly computable for a pure translation and is **zero in this very case**; it needs the floor to see the surfaces along a face's edges, which stops it being a property of one face |
| `DeflectionBelowWhatAMeshCanReach` on a curved solid **far from the origin at all**, at `18·ε·L` | a ball a billion units out cannot be meshed as finely as one at the origin | **limit**, unchanged | `a_ball_a_long_way_out_cannot_be_meshed_as_finely_as_one_at_the_origin`. No `f64` point sits closer to that surface; §18's scaling reaches it exactly |
| `DeflectionBelowWhatAMeshCanReach` on a face whose numbers **straddle the origin or spread over more than a factor of two** in every axis | a fused solid spanning `z ∈ [1e9, 3e9]` gets no shift | **gap**, and **not** a limit (§41.4 item 3) | Not demonstrated by a test. Sterbenz does not apply, so *this* shift cannot be taken exactly; a coarser power-of-two-grid shift applies more widely and is not exact, and would need the measured-slip machinery above to be honest |
| `AClosedEdgeOfTheAnswerWouldStartAtAPole` | a ball cut by a plane containing its polar axis | **gap**, unchanged | `a_ball_cut_by_a_plane_down_its_polar_axis_is_refused_by_name`, which asserts it for `x = 0` **and** `y = 0` and for all three rules and is green on this branch. **§39.7 item 12 is explicitly not closed** by anything in this fan |
| ~~a rotation off the wire whose quaternion norm overflows~~ **withdrawn, §41.12** | silently the identity rather than the rotation or a refusal | ~~**gap**, new at this gate, **not a refusal at all**~~ **closed** | §41.5, fixed by the rescale it named. `a_rotation_written_at_any_scale_is_the_same_rotation` |
| `Error::ZeroLength { what: "rotation" }` and `Error::NotFinite { what: "rotation" }` off the wire | four zeros, or a coefficient that is not a number | **limit** | `a_rotation_with_no_direction_at_all_is_still_refused`. Those bytes name no rotation for any arithmetic to find, at any scale. The only two things `Isometry::try_from` refuses now |

**One refusal was withdrawn.** `two_balls_that_really_cross_are_refused_by_name`
pinned `ACurvedFaceGoesRightRoundWithNoSeam` at two apart and `VolumeIsNotPositive`
at three apart, both under `Union`. Both now give the first name under `Union`,
and the `VolumeIsNotPositive` row moved to `Intersection`. **And then out of the
table altogether, §41.12: at three apart under `Intersection` it is the lens, and
the lens is answered.**

**§39.4's seventh undemonstrated refusal is now demonstrated.** It was the one
that needed four public calls and had no test: `add_round_bar` → `mesh` → `fuse` →
`mesh`, read out of `DeflectionBelowWhatAMeshCanReach`'s own `closest` field,
which is the only way a caller can learn where the floor is. Six of §39.4's seven
remain, all the hand-built-broken-model kind. **The count does not fall to six**,
because §41.6's cap and §41.5's rotation are two new rows with no test between
them. **It falls to six now**: both of those are closed and both left a test
behind (§41.12).

**`closest` is no longer the floor these rows are about, recorded 2026-09-06.**
The two `DeflectionBelowWhatAMeshCanReach` rows above — `18·ε·L` far from the
origin, and `1.4444×` after a fuse — and the sentence just above them are all
about **how close a mesh point can be placed**, which is the floor
`SolidView::mesh` still lets a *caller's own* deflection through against, and all
three read it out of `closest`. §58's unit broke that tie: `closest` now carries
**how fine a mesh can be cut**, the floor under a deflection the mesher picks for
itself, which is the placing floor with the walk's step cap added to it.

Measured here, hunting for the placing floor through the public API and reading
the walk floor out of the refusal (§58.4): at the origin a round bar of radius
one gives `9.214851104388813e-13` against `7.273230840146953e-11` and a ball of
radius one gives `6.143234069592546e-13` against `4.848820560097969e-11` —
`78.929` times apart in both cases. **So `closest` at the origin over-reports the
floor these rows name by seventy-nine times.**

The ball's two readings here are as of `fdf5e58`'s parent. Both have since
halved, to `3.0716170347962663e-13` and `2.4244102800489844e-11`, because a ball
is now charged from its centre rather than from its poles — the round-reach
unit, merged as `fdf5e582`, which left no decisions section of its own. The
seventy-nine did not move, and neither did the bar's pair, which is the point
the sentence above is making.

The rows themselves stand, and the reason is the shape of the two formulas rather
than a second measurement: the added term is a constant share of how far the face
reaches and does not grow with the placement, while `how_far_moving_this_face_rounds`
is in both floors and does. So the separation is at its widest at the origin and
shrinks with distance, which is why §56.5's ratios did not move when this landed,
and why `a_ball_a_long_way_out_cannot_be_meshed_as_finely_as_one_at_the_origin` is
still green — it brackets `closest` between `1e-3` and `1.0` at `x = 1e12`, and
both floors are inside that bracket.

What has to be said differently is the *method*. `closest` is no longer **the only
way a caller can learn where the floor is**, and for the placing floor it is no
longer a way at all: the reading is now a hunt, bisecting for the deflection at
which the refusal stops being the answer, which is what
`the_floor_under_a_deflection_stays_put_while_the_crossing_it_stands_for_runs_away`
does. That hunt terminates at the origin and **does not terminate far out**,
where the first deflection above the floor is one the walk accepts and the mesh
does not finish (§58.4). Any future measurement of the `18·ε·L` row needs a way
round that, and §56.5's slope — the floor's rise per unit of placement — is one,
because it cancels everything that does not move with the placement.

### 41.8 What the lanes flagged and did not build

Five items, each with a demonstration and none of them a live crash.

1. **The mesher round-trips a placed face's surface through its own placement and
   charges nothing for it.** `mesh.rs` computes
   `face.surface().transformed(&the_frame_this_face_is_meshed_in(face))` where
   `face.surface()` is already `stored.transformed(&location)` and the frame is
   `location.inverse()`. `Surface::ROUND_TRIP_ROUNDOFF` is **256·ε** and the only
   term the floor charges for placement is `Isometry::POINT_ROUNDOFF · L`, which
   is **18·ε·L** — fourteen times short for a placement that turns. It is exact
   for an axis-aligned translation, which is why no test catches it.
   **Pre-existing**, the same species as §39.7 item 2, and the fix is cheap: read
   the stored surface instead of round-tripping it.
2. **A shifted picker can decide confidently what an unshifted one refused.** A
   half-space in an `EdgeRun` strays by up to `SURFACE_AFTER_SHIFTING_ROUNDOFF · L`
   when shifted. That moves no mesh point — a strayed offset slides a flat surface
   along its own normal without turning it (§29.8) — and it can only be wrong
   about a pair of pieces closer together than the stray, which for a shift of a
   billion is an arc under about `1e-3` radians. Before the shift the same
   decision was made against a point box `POINT_ALONG_A_CROSSING_ROUNDOFF` wide in
   the world's numbers, fifty-seven times the stray, so the pair was undecidable
   then too — **but it was refused rather than answered, and that difference is
   real.** Not demonstrated by a test.
3. **`Approximation` has no boundary guard where `Scale` has one.** Checked here:
   `Approximation` derives `Deserialize` plainly, with no `#[serde(try_from = …)]`,
   and `check_approximation_ordering` compares magnitudes with `<`, which is
   silently false on `NaN`. So a deserialized `Model` can carry
   `Approximation { magnitude: NaN }` on a vertex, edge or face and the sweep will
   not catch it. **Not a reachable panic** — no call site re-wraps a stored
   magnitude, checked over all twenty `Approximation::new` sites — but the
   invariant is not closed at the boundary the way its neighbours are.
4. **`BSplineSurface` is missing the demonstration `BSplineCurve` has.**
   `evaluate`, `partial_u`, `partial_v` and `evaluate_with_enclosure` share the
   curve's "weight came out non-positive" invariant and the same constructor-side
   proof it cannot happen. The curve side has
   `weights_small_enough_to_blend_away_to_nothing_are_refused_up_front`; nothing
   in `bspline_surface.rs` demonstrates it. **Safe, no live bug, missing
   demonstration only** — which is exactly what §39.7 item 4's rule asks for.
5. **Two doc comments on one mechanism say opposite things.**
   `cut_a_cap_up_to_its_pole` says *"which of the two poles the face reaches is
   read off the loop's own direction"*; `which_pole_this_cap_reaches` — the
   function it is describing — opens *"**Not read off which way the loop runs**,
   and that is the trap this walked into once"*. They are using "the loop's
   direction" for two different things, the code is right either way now that the
   loop direction is trustworthy, and the wording is a §23.8 trap waiting for the
   next reader. Left alone deliberately: fixing it means touching the
   pole-choosing rule.

   *2026-10-15, Q2051 (from Q891): since Q508 they agree.*
   `which_pole_this_cap_reaches` in `mesh.rs` now reads a cap's pole off the
   walk round its rim (`runs_up`), and `cut_a_cap_up_to_its_pole`'s doc says
   the same thing. Q891's lane read `mesh.rs` lines 1916-1917 and 2396-2399
   at `1c8854a6` and found no trap left to flag; its row named this item and
   left this section alone.

### 41.9 The numbers

**At the three lanes and this gate.** The fourth lane's column is §41.12's, and
where the two disagree the later one is current.

| | at §39 (M4's close) | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1112, and 1113 after §40 | **1129** |
| suite duration | 2.9 s | **2.87 s** |
| doc tests | 19 | **19** |
| `FORMAT_VERSION` | 3 | **3, unchanged** |
| fuzz corpus | 815 | **815** |
| wildcard arms over our own enums | 76 | **76, unchanged** |
| `rcad-predicates` files changed | 0 for the whole of M4 | **1** (§41.3) |
| `rcad-mesh` files changed | 0 | **0**, through M4 and this fan |

**Files changed, whole tree:** `rcad-geom` 15 (11 in `src`), `rcad-topo` 10 (6),
`rcad-predicates` 1, `rcad` 1 test file, `rcad-mesh` 0. The suite is sixteen
tests larger than `main`.

**No measured rate moved, family by family and never blended (§17.7).** All seven
`fuses_that_must_agree` families re-measured at this gate, against §38.7:

| family | fuses | refused | not a solid |
|---|---|---|---|
| §37 control, a bar through a block | 423 | 0 (0.0%) | 0 (0.0%) |
| §17.7 control, lattice pairs | 423 | 0 (0.0%) | 20 (4.7%) |
| §17.7 degenerate, sharing coordinates | 423 | 0 (0.0%) | 72 (17.0%) |
| §17.7 control, somewhere else in `f64` | 423 | 0 (0.0%) | 17 (4.0%) |
| §17.7 headline, turned pairs | 423 | 96 (22.7%) | 17 (4.0%) |
| §22 headline, skew pairs | 423 | 51 (12.1%) | 8 (1.9%) |
| §22 control, both leaning alike | 423 | 24 (5.7%) | 17 (4.0%) |

The named breakdowns are unchanged too: turned 96 all *side of plane*; skew 9 /
30 / 12 across *a point inside a piece of a face* / *order along a crossing* /
*side of plane*; leaning alike 3 / 6 / 15. §39.6's own 22.70% against a threshold
of 24% reads the same, so its 1.3 points and six fuses of headroom are where
§39.6 left them.

`how_often_a_curved_fuse_refuses.rs`, re-measured: turned curved pairs **0.0% of
384**; a block turned against a bar **354 of 384 answered**, refusing
`{"a point inside a piece of a face": 24, "an edge would have 1 faces along it": 6}`.

**The three cap volumes and the sweep**, against `27π = 84.82300164692441`:

| axis | volume | off the closed form |
|---|---|---|
| `z` | 84.59732446013571 | −0.266% |
| `y` | 84.62054762306524 | −0.239% |
| `x` | 84.62021161694274 | −0.239% |

and turning ball and slab together about `x` in 0.1 rad steps answers at **all 21
angles from 0.00 to 2.00**, one solid each, from 84.56433812106617 (−0.30495%) to
84.62417907760833 (−0.23440%). No flip anywhere. §39.3 measured this same sweep
on the M4 tree and found it answered 1.00–1.30 and refused 1.40–1.50.

**Soaked, not fuzzed.** `RCAD_SOAK_FACTOR=20` over `fuses_that_must_agree.rs`
(7 719 fuses a family) gives control 0.0%, lattice 4.9% not a solid, sharing
coordinates 15.8%, elsewhere 4.5%, leaning alike 5.2% refused, skew 14.5%,
turned 24.3% — every one inside §38.8's soak ranges. **Neither soak was re-run at
this gate**; what stands in its place is two lanes measuring the same seven
figures independently, which is weaker and is said rather than hidden.
**`just fuzz` was not run by any lane or by this
gate.** `rcad-geom` changed, so §30.7's rule does ask for it; what stands in its
place is that the change is three new methods and one new constant, no existing
expression moved, `where_two_surfaces_cross` — the fuzz target — reaches none of
them, and the shipped equivalent
`every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly` passes. That is a
weaker argument than a run and is written down as one. **The fourth lane ran it
and the argument can be struck: §41.12.**

### 41.10 Corrections to §39 and earlier

- **§39.7 item 1's description of the `x` face is wrong in the half that matters.**
  It says *"there the fuse gets past naming the faces and the triangulator is what
  gives up"*, put as if the `y` row does not. Both get past naming the faces; the
  fuse answers both; **both refusals come out of `mesh.rs`**, three lines apart in
  the same `match`, and this gate instrumented `main` to confirm it. Anyone
  scoping the work from §39.7 alone would have looked in the wrong crate for one
  of the two.
- **§39.7 item 1 is not closed by this fan.** §41.6. It is closed for a cut one
  unit from the centre and open from `h/r ≈ 0.29` down. **The fourth lane closed
  that half too** — every depth, all three axes, §41.12. What is left of item 1 is
  the piece on the *other* side of the cut, which is a different mechanism and
  §41.7's second row.
- **§39.7 item 2's causal claim is wrong and its citation is not.**
  `triangulate.rs:860` really does call its own bound a §16.10 gap; fixing that
  bound would have moved the measured 77 by exactly zero, because `rcad-topo`
  refuses first under its own name. §41.2.
- **§39.7 item 2's family (B) is two fixes in one sentence**, and only the
  constant-widening one is impossible. §41.4 item 2.
- **§39.7 item 3 cites §16.11 for a rule §16.11 does not contain**, and so do
  §38's opening paragraph, §38.6's fourth row, §39.4's closing note, and fifteen
  doc comments still in the tree. §16.11 is about the dynamic-range check being a
  ratio. The rule is CLAUDE.md's tier table and §4.1, with §4.3 for where the
  sweep runs. §41.4 item 6.
- **§39.7 item 7 said the `rcad-predicates` change belonged to M5's opening gate
  and a lane took it.** §41.3. There is no M5 opening-gate section; this is where
  that is recorded.
- **§39.3's *"only the first [of three kinds of piece] is handled"* is now two of
  three.** Only the both-poles piece is unhandled.
- **§38.6's `ACurvedFaceGoesRightRoundWithNoSeam` row is one row for three
  shapes**, two limits and a gap, and §39.8 had already half-corrected it. §41.7
  splits it.
- **§38.3's table of what a handed-in direction buys was not reproducible** and
  should not be quoted as current until someone reproduces it. §41.4 item 7. The
  refusal it describes is unchanged and still asserted by name.
- **`WalkingAWire::finish`'s comment** — *"one coedge whose two ends are one
  corner runs right round the curve its two faces cross in, which on a periodic
  surface is right round the surface"* — is false on a sphere. Fixed in the tree.
- **`how_far_round_this_face_it_goes`'s doc** — *"Zero for a closed edge that does
  not go round the face at all … and then the evaluator's own direction is left
  alone. **That is the right answer for the wire this is read for**"* — is not:
  it left the same cap walked one way at `y = 1` and the other at `x = 1`. The
  function is deleted.
- **`cut_a_cap_up_to_its_pole`'s doc** said *"a ball cut by a plane leaves one
  [cap]"*. Only a plane square to the polar axis does. Fixed.
- **`a_ball_cut_by_a_plane_that_crosses_its_seam_twice_is_refused_by_name`'s doc**
  said *"one crossing works and two do not, and the two cuts differ in nothing
  else"*. The seam-crossing count was never what decided it: `z = 2` leaves two
  one-pole pieces, `x = 2` leaves a no-pole cap and a both-poles remainder, the
  cap answers against `40π/3` and only the remainder refuses. Renamed and both
  halves asserted.
- **`how_far_a_mesh_point_of_this_face_rounds`'s doc** said the evaluator follows
  a crossing in the frame the face is **stored** in. It follows it in the frame the
  face is **meshed** in, which was the same thing only while nothing baked.
  Rewritten.
- **`rcad-mesh`'s `NoMeshCanGetThatCloseToTheSurface` escape is true again**, and
  for a reason two crates from where the sentence sits: the mesher now puts a
  bar's axis back at the origin itself. §39.8 filed it as falsified and §41.2's
  instrumentation is the demonstration that it holds. **It is still worth
  distrusting** — what makes it true is nothing the sentence names, which is the
  species §39.3's rule was written about.
- **`fuse/bake.rs`'s header** — *"[`Picker`] still needs no `transformed`"* — is
  still true of the fuse and is now misleading as a statement about the kernel:
  the mesher needs `Picker::shifted`. Not edited, because it is `bake.rs`'s
  sentence about `bake.rs`.

### 41.11 What is left of §39.7's seventeen, as a current list

Renumbered so the next lane reads a list rather than a diff. **Items 5, 6, and 9
through 17 were untouched by this fan and are unchanged except where a sentence
below says otherwise.**

| now | was | state |
|---|---|---|
| 1 | §39.7 item 1 | **closed for the cap, open for what is round it, §41.12.** `Intersection` answers on all three axes at every depth down to `h = 0.001`, and at 21 angles of the sweep. `Union` and `Difference` off the polar axis still refuse `ACurvedFaceGoesRightRoundWithNoSeam`, which is **the same §19 violation in the other piece** and is §41.7's second row rather than this one — §38.3's machinery, a milestone and not a lane. **Consumer unchanged: the first imported ball anybody cuts** |
| 2 | §39.7 item 2 | **closed to 1.4444**, and the residual is §41.7's fourth row. **Consumer: every fused curved solid, and every imported one** |
| 3 | §39.7 item 3 | **closed.** All three refuse by name |
| 4 | §39.7 item 4 | **closed for `pub` items**, 58 of them. Eighteen `pub(crate)` clauses were out of scope, §41.3 |
| 5 | §39.7 item 5 | unchanged. `Exact::dot_of` does not slide and `Exact::square_minus` does. A change to `Exact` and therefore a lane with its own re-measurement of every rate in §41.9. **`rcad-predicates` is no longer frozen, so the §31.5 obstacle to it is gone** |
| 6 | §39.7 item 6 | unchanged. Two authorities on *is this plane along this pipe's axis*, `intersect` exact and `along_a_crossing.rs`/`walk.rs` with `== 0.0`, on `(1, 0, 3)` against `(−12, −12, 4)` |
| 7 | §39.7 item 7 | **closed**, and it unfroze `rcad-predicates`. §41.3 |
| 8 | §39.7 item 8 | **six of seven remain.** ~~and there are two new rows with no test: §41.6's cap and §41.5's rotation. Net eight~~ — both of those are closed with a test each (§41.12), so the count is **six**, all the hand-built-broken-model kind |
| 9–13 | §39.7 items 9–13 | unchanged: a ball with a pipe through it; two coincident curved faces; a corner exactly on a curved face; **a ball cut by a plane through its polar axis, re-measured this fan and still refused for all three rules and both `x = 0` and `y = 0`**; a turned notched bar and a bar cut by one plane off its axis |
| 14–17 | §39.7 items 14–17 | unchanged: three curved surfaces and a crossing through itself; cone and torus; the BVH, the bump arena and `rkyv`; `Undecidable` out of `mesh` recorded as a design consequence |
| ~~**18**~~ | new | ~~**`Isometry` off the wire silently identity when its quaternion norm overflows.** §41.5. A wrong answer on a caller's bytes, and the fix is item 7's fix one type over~~ **Closed, §41.12**, by that fix. Nothing is left on this row |
| **19** | new | **The mesher's uncharged surface round trip**, 256·ε charged at 18·ε. §41.8 item 1 |
| **20** | new | **`Approximation` has no boundary guard**, and `NaN` sails past the ordering check. §41.8 item 3 |
| **21** | new | **`BSplineSurface` is missing the demonstration `BSplineCurve` has.** §41.8 item 4 |

**What the closing gate has to be told.** Two things, and neither is a lane's
regression.

- **The headline item is not closed and the record must not say it is.** §41.6.
  A gate that had trusted the reports would have written "the §19 violation is
  closed", and the two tests that back that sentence pin one depth. The test this
  is owed is the same three-axis comparison at two or three depths, which fails
  today and should. **Written, red, and then green — §41.12.** What is left of
  item 1 is the piece on the other side of the cut, and that is a milestone.
- **A wrong answer went into the tree unnoticed and came out under a
  demonstration.** §41.1's two-ball union, reproduced here at 10.396 where the
  union is 56.549, was refused by name *because a lane looked for it* — nothing in
  the suite would have caught it. §39.9's sentence stands unchanged: ten lanes of
  green tests measured the things the lanes thought of.

### 41.12 A fourth lane, §41's own two open items, and two numerals of §41.6

*§41 left exactly two things undone and named them both: the wrong answer §41.5
found and did not fix, and the §19 violation §41.6 found under a closed item.
Both are closed here. Every figure below was measured on the branch, and where
one contradicts a figure above, the one above is wrong and says so.*

**Both fixes are the same move, and it is item 7's.** §41's own opening paragraph
said that when a bound looks too wide the question to ask first is what the
arithmetic was handed. Item 18's fix scales a quaternion's four coefficients by a
power of two before squaring them; item 1's asks a mesh again with less room when
the first one came back flat. Neither moved a published constant, neither
loosened a comparison, and **not one measured rate moved by a single fuse** —
shipped or soaked.

#### Item 18: a rotation off the wire, and the half of it §41.5 did not see

§41.5 has the mechanism right. What it did not have is that **the same one line
refuses at the other end**. `[1e-200, 1e-200, 0, 0]` is a quarter turn about `x`;
the sum of its squares underflows to zero, `norm()` is `0.0`, and the
`ZeroLength` guard fires on a rotation that is perfectly well named. So the
defect was a wrong answer above and a spurious refusal below, and one fix closes
both.

Measured on `d603c05`, `[2^k, 2^k, 0, 0]` at every `k` an `f64` exponent field
holds — the scaling is exact at all 2098 of them, so every one names the identical
quarter turn:

| what came back | scales | which |
|---|---|---|
| the quarter turn the bytes name | 1049 | `k` from `−537` to `511` |
| **the identity — a wrong answer** | **512** | `k` from `512` to `1023` |
| refused `ZeroLength` | 537 | `k` from `−1074` to `−538` |

**Exactly half of the ways of writing one rotation gave something else.** A third
of a turn about `(1, 2, 3)`, same sweep: 1022 of 2044. Both are 0 of 2098 and
0 of 2044 now.

`brought_into_the_range_its_squares_fit_in` multiplies all four coefficients by
the one power of two that puts the largest in `[1, 2)`. Three things make it
right, and the third is the one worth reading:

1. **It names the same rotation.** A quaternion names a rotation by the
   *direction* of its four coefficients, so one shared nonzero factor is
   invisible.
2. **It costs nothing where the old code worked — and the first way of saying
   that was wrong.** The sentence written first was *"where the squares were
   already in range this hands back the identical unit quaternion, bit for bit"*,
   and running `claim-audit` on it broke it in four hundred thousand draws: 2033
   of them had the *sum* of the squares an ordinary number with **one of the four
   squares in the subnormals**, and there the two answers differ by an ulp. A
   square that has fallen into the subnormals has lost bits, so the norm built
   from it is not the norm, and it is the **unscaled** answer that was wrong.
   The claim that survives is *all four squares normal*: then each scales by
   exactly `2^2e`, so the sum does, so its square root scales by exactly `2^e`,
   and the division cancels the factor.
   `scaling_first_moves_no_answer_whose_four_squares_were_normal_to_begin_with`
   is the demonstration, and it counts all three outcomes with a floor under each
   — 31 323 bit for bit, 4 212 at the subnormal boundary and within one ulp,
   4 433 with the sum out of range altogether — because a run where the second
   bucket was empty would be testing the first on nothing. **Nothing that used to
   answer now refuses**, measured the same way: 0 of 400 000.
3. **Nothing new is refused, and that was a decision.** The largest coefficient
   always lands in `[1, 2)`, so there is no unscalable quaternion and no new
   `Err`. A coefficient more than 1022 binades under the largest goes subnormal
   and rounds, by at most half of `2^-1074`; it is that far under the *norm* too,
   so the number it stands for in the unit quaternion is itself under `2^-1022`,
   and half the smallest `f64` is as close as any answer gets. What stays refused
   is four zeros and a coefficient that is not a number, and both are **limits**:
   those bytes name no rotation for any arithmetic to find, at any scale.

**And §41.4's last paragraph is now a demonstration rather than a coincidence.**
`Cylinder::basis` and `Plane::basis` claim a zero-length direction cannot reach
them, name two routes in, and the tree has three — `transformed`, which does not
re-normalize. §41.4 found the claim survives because `Isometry::try_from`
normalizes, "a fact two types away from where the sentence sits". It survived
before this fix as well, because a zero quaternion still normalizes to the
*identity* and the identity is a unit quaternion; it survives now for the reason
the sentence would want. Both doc comments name the third route and cite
`a_rotation_written_at_any_scale_is_the_same_rotation`.

#### Item 1: the §19 violation, and what the quarter turn was really about

§41.6 names the cause and this lane confirmed it before building on it, with
`how_much_space_it_encloses` instrumented on `d603c05`:

| | points | triangles | volume |
|---|---|---|---|
| `d = 2.82`, square to `z` | 5 | 6 | 6.330779 |
| `d = 2.82`, square to `x` | 5 | 6 | 6.330779 |
| `d = 2.83`, square to `z` | 5 | 6 | 6.233058 |
| **`d = 2.83`, square to `x`** | **4** | **4** | **0** |

**The boundary is not approximately `h/r = 0.29`, it is exactly
`d = r/√2 = 4·cos(π/4) = 2.8284271`, and the quarter turn in it is the mesher's
own.** The cut circle `x = d` on a ball of radius `r` charted about `z` spans
`2·atan(√(r² − d²)/d)` of longitude, which is a quarter turn exactly when
`r² − d² = d²`. `cut_off_what_goes_too_far_round` sets `widest = whole_turn / 4`
and leaves any patch narrower than that in one piece, and
`as_coarse_as_a_mesh_gets()` — `Deflection::of(f64::MAX)` — gives the
triangulator no reason to put a point inside one. Its doc rests on the mesher
never stepping more than a quarter turn: *"a bar still comes out four ways round
and a ball still comes out an octahedron"*. **That is an argument about a whole
surface and it is silent about a patch.** A patch under a quarter turn comes back
as its own boundary and nothing else — flat — and closed off by the flat face
across that boundary it encloses zero to the last bit, so `nest_the_shells`
refused `VolumeIsNotPositive { volume: 0.0 }`. Square to `z` the cap holds a pole
and is fanned up to it whatever the deflection, **which is why exactly one axis
ever worked**.

The fix: a zero is asked again with less room, halving from the mesh's own width
until the mesh has a point off the flat or the mesher will go no finer. Halving
from the width and not down from `f64::MAX` is what keeps it to a handful of
meshes — the bulge a flat patch hides is at most that width — and the mesh floor
is what terminates it. `nest_the_shells` and `check_shells_and_solids` go through
the one function now, which matters: without that the fuse would write an answer
the sweep then rejects.

Re-measured, ball of radius four, a twenty-cube whose face sits at `d`,
`Rule::Intersection`, every volume at a deflection of `0.002`, against
`πh²(3r − h)/3` with `h = 4 − d`:

| `d` | square to `z` | square to `y` | square to `x` | closed form |
|---|---|---|---|---|
| 1.00 | 84.798930 | 84.801606 | 84.801572 | 84.823002 |
| 2.00 | 41.872603 | 41.871231 | 41.871236 | 41.887902 |
| 2.82 | 15.769081 | 15.769041 | 15.769041 | 15.776835 |
| **2.83** | **15.517572** | **15.514765** | **15.514767** | **15.524900** |
| 3.00 | 11.513343 | 11.511263 | 11.511278 | 11.519173 |
| 3.50 | 3.006789 | 3.006957 | 3.006956 | 3.010693 |
| 3.90 | 0.123970 | 0.123949 | 0.123939 | 0.124617 |
| 3.99 | 0.001166 | 0.001176 | 0.001178 | 0.001256 |
| 3.999 | 0.000008 | 0.000008 | 0.000008 | 0.000013 |

**Every row answers on every axis**, down to a cap a thousandth of a unit high.
Down to `d = 3` the three columns agree with each other to **0.018%** where the
worst of them is 0.069% off the closed form — they are one shape tessellated
three ways, and what separates them is the order a sum of triangles is added up
in. Below that a fixed deflection stops buying accuracy on a shape that thin,
which is §41.2's own point that a deflection is an absolute length: at `d = 3.99`
the three are 0.96% apart and 7.1% under a cap of `0.001256`, and at `d = 3.999`
they print the same six decimals and are a third under a cap of `1.256e-5`.
**They still
answer, and they still answer together**, which is what §19 asks; how close a
`0.002` mesh gets to a cap that thin is a different question and not one this
table is about.

`the_same_ball_cut_at_three_depths_along_three_axes_gives_the_same_cap_each_time`
therefore pins `d = 1`, `2` and `3`, where the closed form is an oracle worth
asserting against.

It is the test §41.11 said was owed. It was written first, watched fail at
`d = 3` on `y` and `x`, and **it compares the three axes with each other** and not
only against the closed form: a depth where all three answered the same wrong
volume would sail through a test that only asked each of them to answer.

**The smaller shape of the same row closed with it.** The lens between two balls
of radius two, `Intersection`, at a deflection of `0.004`:

| apart | found | `π(4r + d)(2r − d)²/12` |
|---|---|---|
| 2.0 | 10.455170 | 10.471976 |
| 3.0 | 2.872094 | 2.879793 |
| 3.5 | 0.749062 | 0.752673 |
| 3.9 | 0.030459 | 0.031154 |

**No fuse of ordinary solids reaches `VolumeIsNotPositive` any more**, and
`degenerate_curved_fuses.rs` no longer imports `Invalid` at all. It is still
raised — by `validate` on a hand-built model whose faces point inwards, and by
`nest_the_shells` on a void with no shell around it — and its two unit tests in
`validate.rs` are green. What leaves §41.7's table is the *fuse* refusal.

#### What is *not* closed of item 1, measured rather than assumed

`Union` and `Difference` of the same ball and the same slab, `d = 1`, `2` and
`3`, at the same deflection:

| `d` | square to `z` | square to `y` and `x` | ball less the cap |
|---|---|---|---|
| 1 | `Ok` 183.219755 | `ACurvedFaceGoesRightRoundWithNoSeam` | 183.260 |
| 2 | `Ok` 226.145340 | the same | 226.195 |
| 3 | `Ok` 256.504610 | the same | 256.564 |

**That is a §19 violation too, and §41.7's row for it does not say so.** Same two
solids, same relative position, a quarter turn apart, one answering and one
refusing. It is a different mechanism and it was already classified: the piece
left round an off-axis cut holds *both* poles, and writing it down wants §38.3's
machinery — a cut closed face with a seam and two poles of its own. §41.1 says
why the frame-change fix cannot be expressed today: `Sphere::transformed` moves
only the centre, so a rotated chart cannot be handed to `TrimmedDomain::new`, and
reaching it means changing `rcad-geom`'s `Sphere` or `rcad-mesh`'s
`TrimmedDomain`. **A milestone and not a lane**, and half-fixing it to make item 1
read "closed" is the thing §4.1 ranks below refusing.

#### Two numerals of §41.6, put right

1. **"the lens is `π/4 = 0.785`" is wrong, and so is the "twentyfold" that rests
   on it.** The lens of two balls of radius `r` a distance `d` apart is
   `π(4r + d)(2r − d)²/12`; at `r = 2` and `d = 3` that is `11π/12 = 2.8798`. The
   cap at `d = 2.83` is `15.5249`, so the catalogue row understated the gap by
   **5.4 times** and not by twenty. §41.6's point — that a row describing a gap by
   the smallest shape reaching it is a row the next lane will size wrong — is
   exactly right and the arithmetic under it was not, which is §16.10's own shape.
   The suite carried the same wrong figure in
   `what_two_balls_that_really_cross_share_is_the_lens_between_them`'s doc, where
   it was never asserted against anything; it asserts `11π/12` now.
2. **`h/r ≈ 0.29` is exactly `1 − cos(π/4) = 0.2928932`,** and saying it exactly
   is what points at the quarter-turn rule rather than at a coincidence.

#### The numbers, and every rate re-measured

| | at §41.9 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1129 | **1136** |
| suite duration | 2.87 s | **2.76 s** |
| doc tests | 19 | **19** |
| `FORMAT_VERSION` | 3 | **3, unchanged** |
| fuzz corpus | 815 | **824** |
| wildcard arms over our own enums | 76 | **76, unchanged** |
| `rcad-mesh` files changed | 0 | **0**, through M4 and the whole fan |

**Files changed, whole tree:** `rcad-geom` 17 (12 in `src`), `rcad-topo` 10 (6),
`rcad-predicates` 1, `rcad` 1 test file, `rcad-mesh` 0, and 9 new fuzz corpus
entries. No public signature changed and no public variant was added: item 18
removes a wrong answer without adding a name, and item 1 removes a refusal
without adding one.

**Not one measured rate moved.** All seven `fuses_that_must_agree` families, the
curved suites, and the arrangement's own rows, re-run after both fixes and
identical to §41.9 to every digit §41.9 printed:

| family | fuses | refused | not a solid |
|---|---|---|---|
| §37 control, a bar through a block | 423 | 0 (0.0%) | 0 (0.0%) |
| §17.7 control, lattice pairs | 423 | 0 (0.0%) | 20 (4.7%) |
| §17.7 degenerate, sharing coordinates | 423 | 0 (0.0%) | 72 (17.0%) |
| §17.7 control, somewhere else in `f64` | 423 | 0 (0.0%) | 17 (4.0%) |
| §17.7 headline, turned pairs | 423 | 96 (22.7%) | 17 (4.0%) |
| §22 headline, skew pairs | 423 | 51 (12.1%) | 8 (1.9%) |
| §22 control, both leaning alike | 423 | 24 (5.7%) | 17 (4.0%) |

`how_often_a_curved_fuse_refuses`: turned curved pairs **0.0% of 384**; a block
turned against a bar **354 of 384 answered**, refusing
`{"a point inside a piece of a face": 24, "an edge would have 1 faces along it": 6}`.
Cutting a curved face: 0.00% of 2304 faces on all three families.

**Soaked as well as shipped**, `RCAD_SOAK_FACTOR=20` over
`fuses_that_must_agree.rs` (7 719 fuses a family): control 0.0%, lattice 4.9% not
a solid, sharing coordinates 15.8%, elsewhere 4.5%, leaning alike 5.2% refused,
skew 14.5%, turned 24.3%. **Every one of the seven is the figure §41.9's soak
printed**, which is the answer to §41.9's own note that neither soak was re-run
at that gate.

**`just fuzz` ran.** 120 seconds, 3 773 100 runs of `where_two_surfaces_cross`,
**no crash and no artifact**, coverage settling at 2589 edges and 6769 features.
Nine new corpus entries, 815 to 824, committed. §30.7's rule asked for this
because `rcad-geom` changed, and §41.9's replacement argument for not running it
can be struck rather than carried forward.

#### What the next lane should read out of this

- **Item 1 is closed for the cap and open for what is round it**, and the open
  half is a milestone. Item 18 is closed outright. Items 19, 20 and 21 of §41.11
  are untouched.
- **A claim audit broke this lane's own sentence too**, and it is the same
  species as the two above: *"where the squares were already in range"* was true
  of the sum and silent about the four. §16.10's rule caught its own author, one
  section after §41.4 pointed it at three lanes' reports. **Run it on what you
  wrote, not only on what you inherited.**
- **Both of §41's own open items were the same fix twice**, and both were found
  by a claim audit rather than by a failing test. §41.5's rotation came out of
  checking a `# Panics` clause two types away from it; item 1's real boundary came
  out of doubting a doc comment's *"a ball still comes out an octahedron"*, which
  is true and is about a different thing than the sentence it justifies. **The
  species is a claim that is true of the case its author had and silent about the
  case that breaks** — §16.10 for the third milestone running.
- **A gate that audits three reports has audited three reports.** §41 caught a
  numeral in a lane's report and printed one of its own; the correction is above
  and the pattern is worth naming, because the next gate will write a section too.

### 41.13 A §19 violation four lanes ran past, and the gate that could not see it

*The branch tip was red: `turning_a_fuse` failed on a pair of blocks. It is not a
regression from any of the twenty commits, the fuse is not at fault, and the
number this closes is not the one it was reported as. Every figure below was
measured on the branch.*

**The case.** A `2 x 1 x 1` block on a `3 x 3 x 3` one, meeting exactly on
`z = 0` over a rectangle of area 2. Turned by `TAU·44/64` about `(2, 3, -2)`:

| | solids | faces | volume | area |
|---|---|---|---|---|
| fuse then turn | 1 | 10 | 29 | 60 |
| turn then fuse | 2 | 12 | 28.99999999999999 | 63.999999999999964 |

`60 = 64 − 2·2`. The lattice side glues the pair and drops both faces of the
contact; the turned side never glues it. The property allowed `2.1e-10`.

#### What actually happened, and it is upstream of every suspect

Dumped in `bake::surface_of_every_face` and `cuts::cuts_between`:

```
face 1 (block 1, bottom): stored [0,0,-1,-0.0]  placed at t = [0,0,0]
                       -> [ 0.9975583778312507, ..., -0.0]
face 8 (block 2, top):    stored [0,0, 1,-3.0]  placed at t = [2.9926751334937522, ...]
                       -> [-0.9975583778312507, ...,  8.881784197001252e-16]
```

The two normals are **exactly** opposite. Only the offsets moved, by `8.88e-16`,
so `where_two_planes_cross` answers `Nothing`, `cuts::gather` makes no cut line,
no face is split, and `assemble` never sees an interface at all.

`place_solid` composes, so block 2's placement becomes `(R, fl(R·(0,0,-3)))` —
and `fl(R·(0,0,-3))` is not `-3·fl(R·(0,0,1))`, because the quaternion sandwich
is not exactly homogeneous in the vector it turns. `Plane::transformed` then
works the offset out as `d + n·t`, whose terms are both about 3 and nearly
cancel. **Summed exactly through `Exact` the residue is
`-5.195804564346705e-16`, so it is not an artefact of the `f64` summation
order**: the composed placement genuinely puts block 2 that far off block 1. That
is `0.78·EPSILON·L` at `L = 3`, inside `Isometry::POINT_ROUNDOFF`. The pair
handed to the turned fuse is not in contact, and the fuse answered it correctly.

**`d8b903e` is cleared with a demonstration and not an argument.** It changes
shell nesting; the failure is two stages upstream of nesting, in `cuts`, where no
cut line is ever made. The ball-cut-off-pole item it closed stays closed and
nothing here goes near it. `git diff main..m5` over every file on this path —
`isometry.rs`, `plane.rs`, `view.rs`, `intersect.rs`, `model.rs` — is doc
comments, `?` propagation and `ad43ccf`'s `TryFrom`, which is the wire route
only. `Isometry::rotation`, `apply_to_vector`, `compose`, `Plane::transformed`,
`place_solid` and `where_two_planes_cross`'s arithmetic are byte-identical to
`main`.

#### How broad it is

Every turn `pairs::a_turn` can draw, 21546 of them, asking only whether the two
contact planes come out coincident (`isometry_algebra.rs`):

```
same 10446, offsets differ 11100, normals differ 0, worst gap 4.88e-15
```

**51.5% of general rotations break an exact face contact.** Cross-tabulated
against what `Rule::Union` of this pair then does:

| | glued | undecidable | other error | apart |
|---|---|---|---|---|
| the planes still agree | 5631 | 2566 | 2249 | 0 |
| the planes disagree | 1 | 10896 | 98 | **105** |

So of the turns that break the contact, 98.2% refuse and **0.95% answer
confidently with the pair unglued**. Those 105 are what an area comparison
catches; the other 10896 are the number §17.7 has been publishing for two
milestones.

#### The turned family's refusal rate is this, and not much else

Over 4000 fresh draws of `a_turned_pair`, three rules each, under
`Policy::certified()`:

| | turned fuses | refused |
|---|---|---|
| the pair kept every contact | 5817 | 96 (**1.65%**) |
| the pair lost a contact | 6183 | 2832 (**45.80%**) |
| all together | 12000 | 2967 (24.73%), of which **2865 sit on a lost contact** |

24.73% is `fuses_that_must_agree`'s §17.7 turned headline — 22.70% shipped,
24.3% soaked — arriving from a second direction. **96.6% of that family's
refusals are pairs whose contact the placement rounding broke**, and a turned
pair that keeps its contacts refuses at 1.65%.

That number has been a headline of this tree since M3: §17.7 published it at
**105 of 414 = 25.4%, every one of them *side of plane***, every gate since has
re-measured it, and none has said what it was. §39.6 attributed two points of it
to one commit and put the rest down to the corpus growing. **The rest is this.**
A turned pair refuses because two of its faces were meant to be touching and are
a rounding apart, and *side of plane* is the name the arrangement gives to being
unable to say which side of one of them a corner of the other is on.

#### The seed, and what four green lanes were really worth

`turning_a_fuse` ran `TestRunner::new`, and `Config::default().rng_seed` is
`RngSeed::Random`, which proptest 1.11 turns into `ChaChaRng::from_os_rng` — OS
entropy on every run. **But the interesting number is not how lucky the greens
were.**

- Measured: **0 draws in 43000** break the area bound, and six soaked seeds at a
  hundred times the corpus — between 5099 and 5314 pairings each — find none
  either. A run is 81 pairings, so the property failed well under one run in
  five hundred.
- Measured: **17643 of 20000 drawn pairs share a plane, and 10129 of those lose
  a contact when the pair is turned.** Half of every corpus this suite has ever
  drawn carried the defect.

So the four lanes' greens were almost certainly not luck, and that is the worse
finding rather than the better one: **the gate had no power against the defect it
eventually reported.** It saw it once, in a hundredth of a percent of the
configurations that had it, and only because a rare shape let a broken contact
through to an answer instead of to a refusal. A property that reports the defect
once in 40000 draws is not a gate against it; the *refusal rate beside it* was
the measurement that had been describing it all along, and nobody had attributed
it.

The seed is fixed now, with `RCAD_SOAK_FACTOR` and `RCAD_SOAK_SEED` doing the
sweeping — the same two names `fuses_that_must_agree.rs` uses. That is worth less
than it looks for finding defects and worth a great deal for the numbers the file
prints, which changed every run and which §39.6's method — attribute a moved
number to a commit, never widen it — cannot be applied to otherwise.

#### What the property claims now

Surface area is **discontinuous** where two faces meet: lose a contact of area
`a` and both of its faces stay, so the total goes up by `2a` however small the
slip. Volume and the centroid do not move — 29 against 28.99999999999999 here.
A bound derived from `Isometry::POINT_ROUNDOFF` covers the two continuous
quantities and cannot cover the third, because what it is looking at is not a
rounding in the answer but a different answer.

`how_much_a_lost_contact_can_add` is **derived from the two inputs and is not a
slack**: for every pair of faces on one surface in the lattice frame and not in
the turned one, twice the smaller face. Where two faces coincide the two answers
can differ over that region and nowhere else, and each face is either kept or
dropped there, so twice the region bounds it and the region is inside both
faces. It is zero for every pairing whose contacts survived. The window widens
**upwards only** — a contact can be lost and never gained, because a lattice
pair's faces are a whole unit apart when they are not exactly together — so a
turned answer with *less* area has dropped a face and nothing excuses it. On the
pinned case the room is exactly 4 and the areas differ by 3.999999999999964.

#### Three fixes to the arithmetic, measured and refused

Over the same 21546 turns, how often the two contact planes come out coincident:

| how the moved offset is worked out | agree |
|---|---|
| `d + n_moved·t`, what the code does | 10446 |
| the same, summed exactly through `Exact` and rounded once | **378** |
| `d + n·(R⁻¹t)` | 12086 |

**Exactness makes it worse**, and that is the finding under the family: what
keeps a contact alive is not accuracy but two routes agreeing, and the `f64`
sum's own rounding lands on zero more often than the true value does. The fourth
member of the family, `n_moved · movement(origin())`, is what §19.1 took out and
is not a candidate.

**The residue is not the summation order**, which is the claim the whole
paragraph rests on and the one §16.10 says to demonstrate rather than assert.
Summed through the crate's own `Exact` the leftover on the pinned turn is
`-5.195804564346705e-16` against the `f64` sum's `-8.881784197001252e-16` — two
thirds as big, and not zero.
`a_placement_that_moves_a_plane_off_its_neighbour_does_it_in_the_arithmetic` is
where that is asserted. The composed placement really does put the second solid
that far off; no way of adding four numbers up recovers a contact that is not
there.

#### A gap, its consumer, and what would close it

**A gap.** The answer exists — a plane with offset `0` is an `f64` — and one
arrangement reaches it every time: **two faces that touch stay touching, exactly,
when one movement moves both.** The two faces of a contact are one plane and its
`reversed()`, `(n, d)` and `(−n, −d)`; `Plane::transformed` is a pure function of
the plane and the movement, and negating a vector negates each product and each
sum of an `f64` dot product exactly, so `(Rn, d + Rn·t)` and its partner come out
exact negatives whatever `t` is.
`two_faces_that_touch_stay_touching_when_one_movement_moves_both` is the
demonstration rather than that sentence.

So §19 holds bit-for-bit for a pair moved **as a unit** and fails by a rounding
for a pair whose members carry different placements — which is what
`add_extrusion` leaves behind for every solid built anywhere but the origin, and
is reachable from the public API in four ordinary calls.

Two families, neither built here, both with a named cost:

- **Keep a pair at one placement.** `add_extrusion` could bake its lift into the
  stored geometry rather than keep it as a location. That closes this case and
  nothing wider: two solids a user placed at two different translations and then
  turned together hit the same rounding.
- **Store a movement as `p → R(p + t)` rather than `Rp + t`.** Composing a
  rotation about the origin onto a translation is then exact — `t` is carried
  over untouched — and `Plane::transformed`'s offset becomes `d + n·t` on the
  *stored* normal and translation, which is exactly `0` for the case above. It
  is not exact when the outer movement has a translation of its own, and it is a
  representation change across four crates. Untested; the sentence above is a
  derivation and not a measurement.

Until one of them, the honest description of the kernel is the one this section
measures: **a general rotation applied to a pair of solids at different
placements moves them relative to each other by up to a rounding of the
placement, a contact does not survive that half the time, and the kernel refuses
45.8% of the pairs where it happens rather than answering them wrong.** That is
§4.1's ladder working — refusing beats a confident wrong answer — and it is the
whole of the turned family's refusal rate.

---
