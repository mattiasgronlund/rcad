# Building a Tolerant B-Rep Kernel over NURBS

A requirements and technology roadmap. Written as a starting point for someone
who has decided to build one, not as an argument that they should.

Everything here is a judgement call presented as a recommendation. Where I think
the choice is genuinely contested I say so. Verify library versions, licences and
maintenance status yourself — this reflects the ecosystem as I understand it and
that moves.

---

## 1. Scope

### 1.1 What a kernel must do

A boundary-representation modeling kernel is four capabilities stacked on a
geometry library:

1. **Represent** solids as a graph of trimmed parametric surface patches.
2. **Intersect** those patches robustly, including in degenerate configurations.
3. **Combine** solids by point-set operations (union, difference, intersection).
4. **Modify** solids locally (fillets, chamfers, shells, drafts, offsets).

Everything else — meshing, measurement, import/export, healing — is downstream of
these and dramatically easier.

### 1.2 Non-goals for a first version

Cut all of these. Each is a project in itself and none is needed to prove the core
works:

- Assembly management, constraint solving, mates
- Feature history / parametric rebuild (but see §2.4 — leave the hooks in)
- Visualization beyond a triangle dump
- Direct modeling / push-pull editing
- Sheet metal, mould design, or any vertical application
- IGES, and initially STEP export

### 1.3 The honest difficulty distribution

By source volume, most of a kernel is geometry and numerics. By effort, it is not:

| Component | Share of the hard part |
|---|---|
| Linear algebra, transforms, quadrature, root-finding | ~0% — use libraries |
| NURBS evaluation, knot algorithms, approximation | ~10% — textbook, well-specified |
| Curve/surface intersection | ~35% — the central problem |
| Boolean operations | ~30% — mostly degeneracy handling |
| Fillets, offsets, sweeps | ~15% |
| Healing, import robustness | ~10% — never finished |

Plan the schedule around the middle two rows.

---

## 2. Foundational decisions

These four are effectively irreversible. Everything in §3 onward is replaceable;
these are not.

### 2.1 The numeric model

**Recommendation: tiered — exact predicates on analytic geometry, certified
enclosures on free-form, tolerance as an explicit opt-in.**

The core distinction: *constructions* produce coordinates, *predicates* produce
discrete decisions. Only predicates determine the combinatorial structure of the
result. If every predicate is decided correctly, the topology is valid even when
the coordinates are approximate. One wrong predicate produces a shell that does
not close.

Spend your accuracy budget entirely on predicates:

```
orient3d(a, b, c, d):
  level 1  f64 determinant + forward error bound     ~99% of calls stop here
  level 2  interval arithmetic, directed rounding    5-20x cost
  level 3  exact rational / expansion arithmetic     100-1000x, near-never reached
```

Because inputs are `f64`, they are exact dyadic rationals, and low-degree
predicates are polynomials in them — so level 3 is genuinely computable for
planes, cylinders, cones, spheres and their intersections. Mechanical CAD is
overwhelmingly analytic by face count, so this covers most real operations, and it
covers exactly the cases (coplanar faces, coincident edges) where tolerance-based
kernels cause the most avoidable damage.

For NURBS the exact route degrades. Cascaded constructions produce algebraic
objects of rapidly growing degree, and an exact predicate on an approximate
constructed point is an exact answer to the wrong question. Use **certified
enclosures** instead: interval Newton / Krawczyk operators prove that a solution
exists and is unique within a box. When the box is small enough to decide the
predicate, decide it. When it is not, return `Undecidable` rather than guessing.

Consider CGAL's lazy exact number type as the implementation pattern: every
derived value carries an interval plus a pointer to the DAG of operations that
produced it; exact re-evaluation walks the DAG only for the subgraph in question.

**Contested.** Parasolid, ACIS and OCCT are all tolerance-based. Thirty years of
commercial pressure pushed every shipping kernel the same way, because a slightly
wrong model beats a correct refusal for most users. Mitigate with:

- **Symbolic perturbation** (Simulation of Simplicity): resolve degeneracies by
  perturbing infinitesimally and *consistently*. You always get a topologically
  coherent answer, possibly a surprising one, never an incoherent one.
- **A tiered API**: attempt certified, fall back to tolerant on explicit request,
  and tag the result with provenance so downstream code knows which regions were
  approximated and by how much.

That provenance is the thing OCCT structurally cannot offer. Its tolerances record
how much slop a shape currently carries, not where it came from or which decision
was guessed. It has to be designed in from the first line.

### 2.2 Dynamic range and units

`f64` has ~2.2e-16 relative spacing. At coordinate magnitude X, adjacent
representables are `X * 2.2e-16` apart. Algorithms burn headroom to rounding, so
budget a few hundred to a few thousand ULPs of margin.

**Usable dynamic range is ~1e11 to 1e12**, not the 4.5e15 the format nominally
offers.

Consequences:

- Make the kernel **unit-aware**, or at minimum require the caller to declare a
  working scale at model creation. OCCT's unit-agnosticism means its 1e-7
  confusion constant silently means 0.1 nm in mm models and 0.1 µm in metre
  models. That is a footgun, not a feature.
- Derive tolerance defaults from the declared scale rather than hardcoding a
  literal.
- **Author near the origin; place by transform.** Geometry is stored in a local
  frame and positioned by the `loc` on its reference (§2.3), never with the offset
  baked into coordinates. A 1 µm feature on a part positioned 800 m from origin is
  a much harder problem than the same feature at origin.
- Consider `f64x2` (double-double) as an opt-in working precision for the rare
  model that genuinely needs 1e15 of range.

**Recentering is a conditioning technique, not a repair.** Translating a shape
that is *already stored* at 800 m does not recover precision — those coordinates
were quantized to the ULP at 800 m when they were written, and shifting them moves
a number that is already rounded. What recentering does buy is well-conditioned
intermediates: determinants, cross products and Newton residuals involve
differences of similar large quantities and lose relative precision on every
operation, so keeping the arithmetic small matters even when the inputs are not.

```
naive:      store at 800 m → shift to origin → operate → shift back
                      ↑ precision already lost here, unrecoverably

correct:    store at origin, placement = 800 m on the reference
            → operate in A's local frame using the relative transform B⁻¹A
```

When two instances do interact, choose the working frame deliberately:

- Center on the **interaction**, not on either argument — the centroid of the
  overlap of the two bounding boxes, or the corner of the smaller one. A bracket
  meeting a chassis wants the bracket's frame.
- **Snap translations to a power of two.** A shift of exactly `2^k` alters only the
  exponent field, so transform-then-inverse is bit-exact. An arbitrary shift
  rounds, making results depend on where the object happens to sit — which is
  precisely the transform-invariance property §11.2 tests for.
- **Translate, do not rotate.** Rotation into a face-local frame multiplies by
  non-representable sines and cosines and always loses precision. Leave
  orientation alone.

None of this helps when a *single connected object* spans the dynamic range
internally — a 10 m machine carrying 100 nm features has no origin at which all
its coordinates are small. That is a hard failure requiring extended precision, and
it is the case to detect and report rather than silently degrade on.

### 2.3 The data model

**Recommendation: immutable geometry behind `Arc`, mutable topology in a typed
arena.**

The asymmetry is the point, and it is easy to misread. *Geometry* is immutable —
never edited in place, so it is `Sync` for free. *Topology* is mutable, guarded by
`&mut Model`. A boolean does not build a new arena; it mutates the existing one,
inserting new faces and freeing consumed ones. Safety comes from the unique borrow,
not from immutability.

OCCT conflates two kinds of sharing under one mechanism (intrusive refcounting).
They want opposite solutions.

**Geometry — read-mostly, acyclic, high value to share:**

```rust
#[derive(Clone)]
pub enum Surface {
    Plane(Plane),                    // small, by value
    Cylinder(Cylinder),
    Sphere(Sphere),
    Cone(Cone),
    Torus(Torus),
    BSpline(Arc<BSplineSurface>),    // heavy, shared
    Offset { basis: Arc<Surface>, distance: f64 },
    Revolution { basis: Arc<Curve>, axis: Axis },
    Extrusion { basis: Arc<Curve>, dir: Vector },
}
```

Never mutated in place. Edits produce new values; `Arc::make_mut` gives
copy-on-write during construction. This is `Send + Sync` with zero effort because
it is genuinely immutable — no locks, no interior mutability, no lazy caches
hiding in `const` methods.

**Topology — mutable during algorithms, cyclic, needs stable identity:**

```rust
pub struct Model {
    faces: Slab<FaceData>,
    edges: Slab<EdgeData>,
    verts: Slab<VertexData>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct FaceId { idx: u32, gen: u32 }   // generational

struct FaceData {
    surface: Surface,
    wires: SmallVec<[WireRef; 1]>,   // most faces have exactly one
    tol: f64,
}

#[derive(Copy, Clone)]
struct FaceRef { face: FaceId, loc: Isometry, orient: Orientation }
```

Exclusivity moves up a level: `&mut Model` is unique, so there is no second
mutable path to any node. Ids are `Copy`, hashable, and cost nothing to pass.

Use `SmallVec` for the common cardinalities — one wire per face, two coedges per
edge. Without it a "flat" arena still performs one heap allocation per node and the
cache-locality claim is only half true.

**On the generation field.** It is a per-slot reuse counter guarding against ABA,
not a model version. Slot 5 is freed, refilled by an unrelated face, and a stale
`FaceId { idx: 5, .. }` then resolves to the wrong geometry with no crash. Slots
recycle constantly *inside a single boolean*, so narrow generations are genuinely
exhaustible; `u32` is simply the width at which you stop having to analyse it.

The alternative — tombstone dead slots, never reuse, compact when no external ids
are outstanding — permits a bare `u32` id at 4 bytes. Rejected here because
compaction renumbers everything, and callers of a kernel legitimately hold face ids
across operations. Eight bytes is the honest price for ids that stay valid.

Note that generations do not catch **cross-model** use: an id from model A may
resolve in model B by coincidence. That needs a separate model id checked in debug
builds, or an invariant lifetime brand.

Note this makes OCCT's confusing `TopoDS_Shape` / `TopoDS_TShape` split *visible*:
`FaceId` is the shared node, `FaceRef` is the reference carrying location and
orientation. Same design, expressed in the types, and now impossible to forget to
compose the location because geometry is unreachable except through a ref.

This is also convergent — OCCT 8.0's `BRepGraph` is the same shape (flat entity
vectors, typed ids, O(1) reverse indices, refs carrying orientation and location).
They arrived there from the C++ side after thirty years.

**What falls out for free:**

- Adjacency as indices (`HashMap<EdgeId, SmallVec<[FaceId; 2]>>`), O(1) instead of
  traversal-plus-map on every query
- No reference cycles — store downward links, derive upward as an index
- Serialization is near-memcpy; the model *is* flat arrays. Your fuzz corpus and
  your file format become the same thing
- **Snapshots are memcpy too** — a few MB for a large model. Cheap undo, and the
  before/after topology diff that no CAD system currently offers, without needing
  persistent data structures. Version by snapshot plus history maps, not by making
  the arena immutable
- `&Model` is `Sync`, so all read-only work (measurement, validation, projection,
  evaluation, tessellation) parallelizes with `rayon` and no per-thread setup
- Operation history is data — return `HashMap<FaceId, SmallVec<[FaceId; 2]>>`
  alongside the result rather than needing a builder object to outlive the call

**What you pay:**

- The arena must be in scope: `model.face(id).area()`. Mitigate with a view type
  bundling the borrow: `FaceView<'_>`.
- Ids from one model can be used against another. Generational indices plus a
  debug-checked model id is the pragmatic fix; branded lifetimes are the rigorous
  one and are unpleasant enough to defer.
- No automatic freeing — explicit compaction, or accept garbage until drop.

**Rejected alternatives.** All-`Arc` persistent structures work until you need
back-references, then you get cycles or a side index (which is the arena with
worse cache behaviour). `GhostCell` with branded tokens is sound and zero-cost but
ergonomically brutal and rests on subtle `unsafe`.

### 2.4 Provenance and naming, from day one

Retrofitting this is what makes topological naming unsolvable in existing systems.
Design it in:

- Every entity carries a **derivation**, not a position. Not `Face12` but "the face
  generated by extruding profile edge e3 of sketch S". This is the topology-based
  naming of the literature (CATIA, NX) as opposed to geometry-based
  (SolidWorks, and OCCT's coordinate-driven selection).
- Every operation returns complete **history maps**: generated, modified, deleted.
  Not optional, not a separate query on a builder object.
- Expose selection as **queries that re-evaluate**, not stored ids. Onshape's model:
  `qCreatedBy(op, EntityType.FACE)` filtered by orientation/adjacency, re-run at
  rebuild time. Deterministic ids exist underneath for when a stable handle is
  needed, derived from the operation graph rather than position.

**The unsolved residue:** splits and merges. When one face is divided in two, both
halves have identical provenance. No system resolves this symbolically; all fall
back to geometric heuristics (centroid proximity, area ratio, normal agreement).
Budget for heuristics and make them inspectable.

---

## 3. Layer 0 — Foundations

Nearly all of this exists. Do not write it.

| Need | Use | Notes |
|---|---|---|
| Vectors, points, transforms | `nalgebra` | `Isometry3`/`Similarity3`/`Affine3` as distinct types is `gp_Trsf`'s form enum promoted to the type system |
| Dense linear algebra | `nalgebra`, or `faer` for performance | LU, QR, SVD, Cholesky, eigen |
| Optimization | `argmin` | BFGS, L-BFGS, Gauss-Newton, trust region, Nelder-Mead |
| Quadrature | `gauss-quad` | Gauss-Legendre for length/area/volume/inertia |
| Interval arithmetic | `inari` | IEEE 1788 conformant; the certified-enclosure backbone |
| Exact predicates | `robust` | Shewchuk adaptive orient2d/orient3d/incircle/insphere |
| Automatic differentiation | `num-dual` | Removes a whole class of hand-coded derivative bugs |
| Delaunay triangulation | `spade` | Constrained, robust predicates — this *is* the meshing core |
| Spatial indexing / BVH | `parry3d`, `rstar` | |
| Parallelism | `rayon` | |

**Write yourself:** multivariate Newton on square systems (the `math_FunctionSetRoot`
equivalent, ~50 lines on `nalgebra`), and the tiered predicate cascade from §2.1.

---

## 4. Layer 1 — Geometry

### 4.1 Representation

Keep analytic primitives **exact**, never converted to NURBS. A cylinder that
stays a cylinder never accumulates approximation error, and quadric-quadric
intersection can be solved symbolically by classifying the quadratic form.

NURBS as universal fallback:

```
C(u) = Σ Nᵢ,ₚ(u) wᵢ Pᵢ  /  Σ Nᵢ,ₚ(u) wᵢ
```

Work in **homogeneous coordinates internally, expose affine results only** — this
is the design rule `truck-geometry` states explicitly and it is correct.

### 4.2 Algorithms required

- **Cox–de Boor** recursion for basis functions
- **de Boor** evaluation (not Cox–de Boor — numerically better and faster)
- **Boehm knot insertion**; **Oslo algorithm** for multiple insertion
- **Degree elevation**, **splitting**, **reparametrization**
- Derivatives to arbitrary order (`D0`..`DN`); surface normal `∂S/∂u × ∂S/∂v`
- Conics as **rational quadratic Béziers** (exact circle representation)
- **Bézier ↔ B-spline** conversion, and Bernstein-basis extraction (needed for
  §6 root isolation, which relies on the convex hull property)

### 4.3 Differential geometry

First and second fundamental forms; principal, Gaussian and mean curvature;
Frenet frames; continuity classification (C⁰..Cⁿ, G¹/G²). Needed by fillets,
surface joining and adaptive tessellation.

### 4.4 Approximation

Least-squares B-spline fitting under constraints; interpolation through given
points; arc-length parametrization. `curvo` and `truck-geometry` both implement
substantial parts of §4.2–4.4 and are worth reading even if you do not depend on
them.

### 4.5 Degeneracy handling

Return `EvalError` for invalid parameters and degenerate weights. Never panic.
Surface poles (sphere, cone apex) where the normal vanishes need explicit
representation — these become the "degenerate edges" that confuse every user of
every kernel.

---

## 5. Layer 2 — Topology

### 5.1 Entity graph

`Vertex → Edge → Wire → Face → Shell → Solid → Compound`

### 5.2 Orientation algebra

`Forward | Reversed | Internal | External`, composed as a ±1 group operation.
Orientation is a property of *how a shell uses a face*, not of the face — the same
face is Forward in one solid and Reversed in the adjacent one. It must live on the
reference, not the node.

### 5.3 Parametric curves (pcurves)

Every edge carries its 3D curve **and** a 2D curve in the `(u,v)` domain of each
adjacent face. A seam edge on a cylinder carries two different pcurves on the same
face.

This dual representation is mandatory and is the source of most subtle bugs.
Consider making it a first-class entity rather than a list to search: a **CoEdge**
owning the pcurve, UV endpoints and seam pairing directly, one per (edge, face)
use. This is what `BRepGraph` does and it removes the "iterate curve
representations matching by surface handle and location" pattern entirely.

### 5.4 Validity invariants

Assert these after every operation, not at the end:

- `tol(vertex) ≥ tol(edge) ≥ tol(face)`
- Every edge in a manifold solid has exactly two coedges
- Wire closure and orientation consistency
- Pcurve endpoints agree with 3D curve endpoints within tolerance
- **Maximum tolerance across the shape has not grown** (see §11.2)

---

## 6. Layer 3 — Core algorithms

### 6.1 Point/curve/surface distance (`Extrema`)

Minimize `‖C(u) − P‖²` via derivative root-finding. Underpins projection,
snapping, and the correction step of every marching algorithm. Build this first —
everything else calls it.

### 6.2 Curve–surface intersection

Subdivide to isolate, then Newton. Bézier clipping (Sederberg) or the projected-
polyhedron method for certified isolation of all roots using the convex hull
property.

### 6.3 Surface–surface intersection — the central problem

`S₁(u₁,v₁) − S₂(u₂,v₂) = 0` is three equations in four unknowns; the solution set
is generically a curve.

```
1. If both surfaces are quadrics → solve analytically by classifying the
   quadratic form. Do not skip this; it is most of real CAD.
2. Otherwise: find starting points (subdivision + sampling + Bézier clipping)
3. March: tangent direction is n₁ × n₂; adaptive step; Newton correction back
   onto both surfaces; terminate on loop closure or boundary hit
4. Approximate the resulting polyline as a B-spline to tolerance
```

**Where it breaks, and what you must handle explicitly:**

- **Tangential contact** — `n₁ × n₂` vanishes, marching direction undefined
- **Near-tangential** — direction is defined but ill-conditioned; this is where
  kernels produce silent garbage
- Intersection curves that are **closed loops** with no boundary entry point
- **Coincident surface patches** (intersection is 2-dimensional)
- **Isolated point** intersections
- Self-intersecting results

Use interval Newton to *certify* each marched point. When the enclosure is too
wide to decide, report `Undecidable` with the enclosure width — do not guess.

Marching is inherently sequential (each step depends on the last). Parallelize
across distinct intersection curves and across starting-point search, not within a
march.

### 6.4 Tessellation

Constrained Delaunay in each face's `(u,v)` domain (`spade`), refined until 3D
deviation from the true surface is under a deflection tolerance, then mapped
through `S(u,v)`.

Edges must be discretized **first and consistently**, since a shared edge must
produce identical points for both adjacent faces. This phase is serial and can
dominate on models with many small faces — worth optimizing early.

---

## 7. Layer 4 — Boolean operations

### 7.1 The general-fuse architecture

Do not implement cut, union and intersection separately. Implement one
decomposition and three selection rules:

```
1. Broad phase: BVH cull to candidate face pairs
2. Intersect all candidate pairs (parallel)
3. Split faces and edges at intersections; build image maps
4. Classify each piece IN / OUT / ON relative to the other argument
5. Select pieces per operation; reassemble shells and solids
6. Emit history maps
```

### 7.2 Multi-argument from the start

Accept `Vec<Solid>` as arguments and tools, not one of each. `base.cut([t1..t500])`
must run one decomposition over the whole set, not 500 chained booleans. The
chained form recomputes the growing base's face set every time and scales
superlinearly; the batched form does one BVH pass and culls most pairs without any
intersection work.

This is not an optimization to add later — it changes the API signature, and every
downstream feature (patterns, arrays, threads) depends on it.

### 7.3 Classification

Point-in-solid by ray casting with exact predicates; normal comparison for
ON-classification. This is where wrong predicates produce non-closing shells, so
it is the highest-value place for the tiered cascade of §2.1.

### 7.4 Degeneracy catalogue

Enumerate and test deliberately. The classical list:

coplanar faces · coincident edges · vertex-on-face · vertex-on-edge · tangent
spheres · tangent cylinder-plane · zero-length edges · sliver faces ·
seam-crossing intersections · closed periodic surfaces · fully-contained solids ·
touching-but-not-overlapping solids · self-intersecting inputs

---

## 8. Layer 5 — Local operations

Sequenced by difficulty. Each is optional for a first release.

- **Shelling / thickening** — offset all faces, retrim. Easiest.
- **Fillets / chamfers** — rolling-ball surface: solve for a ball staying tangent
  to both faces, trajectory computed by marching like §6.3. Variable radius,
  vertex blends and fillet chains each multiply difficulty.
- **Draft** — rotate faces about a neutral plane, retrim.
- **Sweeps / lofts** — surface construction is straightforward; self-intersection
  detection is not.
- **Offsets** — hardest. Offset surfaces have no exact NURBS equivalent in general,
  and self-intersection removal is an open problem in practice. Every kernel is
  bad at this; you will be too.

---

## 9. Layer 6 — Interop and healing

### 9.1 STEP

`ruststep` handles EXPRESS schema parsing and code generation, which is the honest
foundation. Mapping AP203/AP214 geometry onto your kernel is a separate large
project.

Preserve **instancing**: `next_assembly_usage_occurrence` referencing one shape
representation with per-instance placements must come through as one node with N
refs, not N copies. If you get this right, meshing 500 identical bolts costs one
bolt (store triangulation on the node, apply the transform at draw time).

Honour the file's declared uncertainty, and surface it. A file declaring 1e-3
uncertainty in mm cannot be recovered to micron precision downstream.

### 9.2 Healing

`ShapeFix`-equivalent functionality has no counterpart outside commercial kernels.
It is thirty years of accumulated knowledge about how real files are broken and
cannot be shortcut. Minimum viable set:

- Sew shells from disconnected faces within tolerance
- Rebuild missing or inconsistent pcurves
- Remove sliver faces and zero-length edges
- Fix wire ordering and orientation
- **Cap maximum tolerance** — allow repair, forbid smearing

---

## 10. Parallelism

The ownership model of §2.3 makes this mostly free, which is the largest
structural advantage over OCCT.

**Free (read-only, `&Model` is `Sync`):** measurement and mass properties,
validity checking, projection and extrema of many points, surface evaluation,
per-face tessellation, per-entity import transfer.

**Parallel with care:** boolean intersection phase (per face pair), starting-point
search for SSI.

**Inherently sequential:** marching along one intersection curve, Newton iteration,
boolean classification and shell assembly (graph traversal), fillet chain
propagation.

Structure mutating algorithms as parallel-read → collect → serial-apply against
`&mut Model`.

### 10.1 Scratch allocation is the real ceiling

Allocator contention, not algorithm structure, is what usually caps parallel
scaling — and it presents in profiles as poor algorithmic scaling, which sends
people optimising the wrong thing. OCCT's parallel performance is limited by its
global-freelist memory manager more than by Amdahl.

Booleans allocate enormously and briefly: candidate pair lists, split pieces, image
maps, marched polylines, per-face working sets. All short-lived, all discarded at
the end of the operation, and all being allocated by every worker simultaneously.

Rather than tuning the global allocator, avoid it:

```rust
fn boolean(model: &mut Model, args: &[SolidId], tools: &[SolidId]) -> Result<SolidId> {
    let scratch = Bump::new();
    // every intermediate allocates in `scratch`
    // ...
    // scratch drops in O(1); only the committed result touches the model arena
}
```

A per-operation bump arena makes scratch allocation a pointer increment and frees
the whole set in O(1). If you do this, the choice of global allocator stops
mattering much — which is itself the argument for doing it. Keep `mimalloc` or
`jemalloc` as the fallback for what remains, and measure rather than assume.

One asymmetry to know: a bump arena drops in O(1), but the *model* arena cannot,
because `FaceData` contains `Vec`s and `Arc`s each needing a destructor. Discarding
a model with hundreds of thousands of faces is O(n) drops. If that shows up,
pooling control nets and wire lists into side arenas turns teardown into a few
buffer frees — a second reason to consider pooling beyond allocation count.

---

## 11. Testing

This is where a modern project can genuinely compress OCCT's thirty years —
perhaps to three to five for a competent small team. Not eliminate it.

### 11.1 Corpora

- **OCCT's own `tests/` directory** — thousands of DRAW Tcl scripts, a large
  fraction distilled directly from bug reports. This is the accumulated thirty
  years already reduced to minimal reproductions. Highest-leverage single asset
  available. *Check the licence before building a product on it.*
- **ABC dataset** — ~1M CAD models as STEP with B-rep intact
- **Fusion 360 Gallery** — models with construction sequences
- **Bias warning:** these are modern-kernel output and clean by construction. The
  cases that made OCCT's boolean code enormous are 1997 IGES files with
  mismatched tolerances. A clean corpus exercises the paths that already work.

### 11.2 Oracles

Direct comparison against another kernel fails — two correct kernels produce
different but equally valid face partitions. Topological equality is the wrong
predicate.

**Decomposition-invariant quantities:** volume, area, centre of mass, inertia
tensor, Euler characteristic, genus, shell/cavity counts, symmetric Hausdorff
distance between densely sampled boundaries, and point-classification agreement
(sample through the bounding box, compare IN/OUT/ON).

**Metamorphic properties — no reference implementation needed:**

```
vol(A ∪ B) + vol(A ∩ B) = vol(A) + vol(B)
(A ∪ B) ∪ C ≡ A ∪ (B ∪ C)          associativity
A \ (A \ B) ≡ A ∩ B
T(op(A, B)) ≡ op(T(A), T(B))        rigid-transform invariance
```

**Start with transform invariance.** Kernels overfit badly to axis-aligned and
origin-centred configurations; an arbitrary rotation finds that immediately, with
no oracle at all.

**Tolerance-growth assertion:** track maximum tolerance across the shape after
every operation. Tolerance inflation is the pathology that displaces errors in
both space and time — the operation that caused it succeeds, and something four
steps later fails. Catching it at the source turns a bisection session into a line
number.

*Licence note:* commercial kernel EULAs commonly prohibit benchmarking and
competitive use. "Only as a test oracle" is not obviously outside that.

### 11.3 Finding hard cases

- **Coverage-guided fuzzing** works unusually well here because the branches you
  want to reach *are* the degeneracy handlers.
- **Mutate, do not generate.** Random control points produce garbage no kernel
  handles, giving a flood of meaningless mutual failures. Take a real model and
  perturb: jitter control points by 1e-9, rotate by 1e-7 rad, nudge two faces to
  near-coplanar. That walks deliberately into the dangerous band from a valid
  start.
- **`proptest` shrinking** matters more than generation. The gap between "this
  800-face assembly fails" and "these two tangent cylinders fail" is most of the
  debugging cost.
- **Interval arithmetic as self-oracle** — `inari` tells you when a result is not
  provably correct, with no reference needed.
- Hand-write the §7.4 degeneracy catalogue as explicit cases.

### 11.4 What testing does not compress

Tooling finds bugs; it does not fix them. In a marcher, a tangency failure is
typically fixed by adding a branch, and branches interact — constrained by the
hundreds of cases that already pass. The bottleneck moves from discovery to
repair, and repair needs differential-geometry judgement no harness supplies.

---

## 12. Milestones

Each produces something demonstrable and de-risks the next.

**M0 — Foundations.** Predicate cascade, interval arithmetic wiring, arena data
model, generational ids, serialization. Prove `&Model: Sync` with a parallel
measurement benchmark.

**M1 — Geometry.** NURBS curves and surfaces, analytic primitives, evaluation,
derivatives, knot algorithms. Ship as a standalone crate; it is useful alone.

**M2 — Topology + tessellation.** Build a solid by hand, validate it, mesh it,
render it. First end-to-end artifact.

**M3 — Intersection.** Analytic quadric cases, then marching with certification.
This is the schedule risk; timebox generously and expect to be wrong.

**M4 — Booleans.** General fuse, multi-argument, history maps. Run the
metamorphic suite. *This is the point at which you have a kernel.*

**M5 — STEP import.** Instancing preserved. Now the corpora are usable and the
fuzzing loop closes.

**M6 — Local operations.** Fillets first, offsets last, offsets possibly never.

Getting to M4 is realistically 18–30 months for a small competent team. M0–M2 will
feel fast and mislead you about M3.

---

## 13. Risk register

| Risk | Mitigation |
|---|---|
| SSI degeneracy handling consumes the schedule | Timebox; ship analytic-only booleans first; they cover most mechanical CAD |
| Certified approach refuses too much to be usable | Tiered API with tolerant fallback and provenance tagging from the start |
| Offsets never become reliable | Accept it. So has everyone. Scope them out of v1 |
| Healing is unbounded | Fix only what your own operations produce; refuse dirty imports loudly |
| Arena ergonomics drive contributors away | Invest early in `FaceView`-style borrows and good iterators |
| No adoption because no ecosystem | Ship M1 standalone; NURBS libraries have users who are not kernel users |

---

## 14. Reading

Cited from memory — verify editions and availability.

**Core texts**

- Piegl & Tiller, *The NURBS Book* — the reference for everything in §4
- Patrikalakis & Maekawa, *Shape Interrogation for Computer Aided Design and
  Manufacturing* — intersection, extrema, interval methods; the closest thing to a
  textbook for §6
- Hoschek & Lasser, *Fundamentals of Computer Aided Geometric Design*
- Farin, *Curves and Surfaces for CAGD*

**Robustness**

- Shewchuk, "Adaptive Precision Floating-Point Arithmetic and Fast Robust
  Geometric Predicates" (1997)
- Edelsbrunner & Mücke, "Simulation of Simplicity" (1990)
- CGAL manual, Exact Geometric Computation and the lazy kernel

**Intersection**

- Sederberg & Nishita, "Curve intersection using Bézier clipping" (1990)
- Grandine & Klein, "A new approach to the surface intersection problem" (1997)

**Naming**

- Kripac, "A mechanism for persistently naming topological entities in
  history-based parametric solid models" (1997)
- Capoyleas, Chen & Hoffmann, on generative naming
- Farjana, Han & Mun, "Implementation of persistent identification of topological
  entities based on macro-parametrics approach", *JCDE* 3(2), 2016 — includes the
  survey of which commercial systems use topology- vs geometry-based naming

**Code to read**

- OCCT `src/ModelingAlgorithms/TKBO` (BOPAlgo) — the general fuse in practice
- OCCT `BRepGraph` (8.0) — flat-arena topology, arrived at independently
- `truck` (ricosjp) — pure-Rust kernel; read `truck-geometry`'s design guide
- `curvo` (mattatz) — standalone NURBS on `nalgebra`
- CGAL Nef polyhedra — the exact-arithmetic boolean approach

---

## 15. The one-paragraph summary

Use existing crates for everything below NURBS; they are excellent and often
better than OCCT's equivalents. Write the NURBS layer from the textbooks. Spend
your actual engineering on surface–surface intersection and boolean degeneracy,
which is where every kernel's real cost lives. Make three decisions differently
from OCCT and make them first: certified predicates with explicit `Undecidable`
rather than silent tolerance inflation, an arena data model that makes read
parallelism free and serialization trivial, and derivation-based provenance on
every entity so naming and history are properties of the design rather than a
retrofit. Then budget three to five years and a real test infrastructure, because
the remaining difficulty is not mathematical depth — it is accumulated special
cases, and nothing compresses that to zero.
