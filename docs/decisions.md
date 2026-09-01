# Design decisions

Settled decisions for `rcad`, recorded 2026-08-29 before any code was written.

`docs/roadmap.md` (formerly `Initial_Specification.md`) is the north star: it
describes what a tolerant B-Rep kernel needs and roughly in what order. It is a
roadmap, not a specification of this codebase. This file records what we actually
decided, and — more importantly — **where we chose to depart from the roadmap and
why**. Without that, a later reader sees the code disagree with the roadmap and
"fixes" it back.

Each decision cites the roadmap section it comes from or contradicts.

---

## 1. Departures from the roadmap

Read this section first. Everything else is elaboration.

### 1.1 No lazy exact number type (roadmap §2.1 vs §2.3)

The roadmap suggests CGAL's lazy exact number type: every derived coordinate
carries an interval plus an `Arc` to the DAG of operations that produced it. It
also claims (§2.3, §10.1) that serialization is near-memcpy, snapshots are cheap,
and the model is flat arrays.

**Both cannot be true.** A DAG-carrying coordinate turns every array into a graph
of `Arc`s and kills flat serialization, O(1) snapshots and cheap teardown.

We keep the flat arrays. Coordinates are plain `f64`. Provenance is recorded at
**decision** granularity, not value granularity — see §5 below. The roadmap's own
justification for provenance is *"downstream code knows which regions were
approximated and by how much"*, and that is a statement about decisions, not about
individual coordinates.

### 1.2 No per-entity tolerance field (roadmap §2.3, §5.4)

The roadmap's `FaceData` carries `tol: f64`, and §5.4 asserts
`tol(vertex) ≥ tol(edge) ≥ tol(face)`. That is the tolerance-based model we chose
not to build.

Entities instead carry an optional `Approximation` tag that exists only because a
recorded decision put slop there. §5.4's ordering invariant still holds across the
entities that have a value. This makes the tolerance-growth assertion of §11.2
mean something sharper than "a number went up": it means "a decision was
approximated, and here is which one."

### 1.3 `slotmap` instead of a hand-rolled arena (roadmap §2.3)

The roadmap describes `Slab<FaceData>` plus a hand-written
`FaceId { idx: u32, gen: u32 }`. That is a re-implementation of `slotmap`, which
additionally gives distinct key types per entity (so mixing a `FaceKey` and an
`EdgeKey` is a compile error) and `SecondaryMap` for adjacency and history maps
without hashing.

Cross-model key reuse is still ours to catch; `slotmap` cannot see it.

### 1.4 No bump arena yet (roadmap §10.1)

The roadmap argues a per-operation `Bump` should be in the signatures from the
start, by analogy with §7.2's multi-argument booleans. Nothing in M0 allocates
enough to measure a difference, and a `'bump` lifetime is viral.

Deferred to M4, when booleans exist and the win can be measured. §10.1 itself says
"measure rather than assume", so `criterion` and a `mimalloc` feature flag go in
now to make that measurement possible later.

### 1.5 No symbolic perturbation (roadmap §2.1)

Simulation of Simplicity always produces a coherent answer. That competes directly
with returning `Undecidable`. Shipping both before a single caller exists to say
which one they want means shipping two answers to the same question.

`Policy` carries the knob. It is unimplemented.

### 1.6 No scalar genericity (roadmap §2.2)

The roadmap says "consider `f64x2` as an opt-in working precision." Taken
seriously that is a `Scalar` type parameter threaded through geometry and
topology. `robust` and `inari` are `f64`-specific, precisions cannot be mixed
inside one model anyway, and the whole kernel would be monomorphized to be
instantiated twice.

`f64` only. If double-double ever happens it is a separate build of the numeric
layer, not a type parameter dragged through topology.

### 1.7 A planar boolean before intersection (roadmap §12, §13)

*Added 2026-08-30, after M0.*

The roadmap orders M3 intersection, then M4 booleans. That builds surface–surface
intersection before anything exists to consume it, and it puts the first
measurement of whether this kernel refuses too much at eighteen to thirty months.

**Insert M2.5: a general fuse over planar solids only.** M3 stays intersection.
M4 becomes *extend the fuse to curved intersections* rather than *build the fuse*.

Everything layer 4 needs — the decomposition, face and edge splitting,
IN/OUT/ON classification, shell reassembly, history maps, the multi-argument
signature roadmap §7.2 requires — does not depend on what kind of curve two faces
meet along. On planes that curve is a segment we can compute exactly, so all of
it sits on layers 0–2.

What that buys:

- **A refusal rate we can read off.** Roadmap §13's second risk is that the
  certified approach refuses too much to be usable. Nothing measures that today
  and nothing will until an operation exists to do the refusing. §12.6 already
  shows the symptom: `Policy::max_enclosure` and `enclosure_is_acceptable` have
  no caller anywhere in the workspace. A refusal rate is a property of an
  operation, not of a predicate.
- **Most of roadmap §7.4's degeneracy catalogue.** Coplanar faces, coincident
  edges, vertex-on-face, vertex-on-edge, zero-length edges, sliver faces, fully
  contained solids, touching-but-not-overlapping — all reachable with planes.
  Roadmap §1.3 puts booleans at 30% of the hard part and says that share is
  mostly degeneracy handling. This is that share, unblocked without touching
  intersection.
- **The metamorphic suite two milestones early.** `vol(A ∪ B) + vol(A ∩ B) =
  vol(A) + vol(B)`, associativity, `A \ (A \ B) ≡ A ∩ B`. §12.2 is the argument
  for wanting these as early as possible: where an oracle shares a lineage with
  the code under test, only a property the answer must satisfy can see the bug.

**This is a departure, not a schedule note.** Roadmap §13 files analytic-only
booleans as what to ship *if* M3 overruns. We are making the contingency the
plan. Without this recorded, a later reader who finds booleans arriving before an
intersector reads it as the schedule having slipped and "fixes" the order back.

**What it costs.** An interface designed against exact segments may not survive
marched, approximate curves carrying an `Undecidable`. Two guards, both cheap now
and expensive later:

- The fuse consumes an intersection *result* type that carries an enclosure and
  an `Approximation` from its first line, even though the planar producer never
  populates them. The curved case is then a new producer, not a new contract.
- The splitter is seeded with §2.5's three-planes-meeting cases, so the path
  where a split vertex is known only as an interval is live in the planar world.
  §12.3 is the warning: a generator of one shape family finds one family of bugs.

Rejected: keep the roadmap order and measure the refusal rate at M4. The answer
changes the design, and at M4 it arrives too late to act on.

Rejected: a throwaway boolean prototype instead. Throwaway code does not carry
the local invariant checks of §4.3, and a finding from code without invariants
does not transfer to code with them.

**Amended 2026-08-30.** M2.5 does not wait for M2 either, and runs alongside M1
rather than after it. §15.1 has the dependency argument and §15.9 has the terms
it runs under.

**M2.5 exit criteria.**

1. One decomposition and three selection rules — union, difference,
   intersection — over `Vec<Solid>` arguments and tools per roadmap §7.2. Not
   three algorithms.
2. Face and edge splitting at intersections, with image maps.
3. Point-in-solid classification reused from §9.2's benchmark, not a second
   implementation of it.
4. History maps returned per operation: generated, modified, deleted.
5. Roadmap §7.4's catalogue as explicit hand-written cases, planar entries only,
   each either passing or refusing with a recorded reason.
6. The §11.2 metamorphic properties as a `proptest` suite: inclusion–exclusion on
   volume, associativity, `A \ (A \ B) ≡ A ∩ B`, transform invariance.
7. `Policy::max_enclosure` has a caller — the fuse judges its worst enclosure
   against it and refuses. Closes the first gap in §12.6. *(Amended
   2026-08-30: §15.3 renames these. The criterion is now that
   `Policy::max_wobble` has a caller, judged against
   `ProvenanceLog::worst_wobble`.)*
8. A measured refusal rate under `Policy::certified()` over that corpus,
   recorded in this file.

**Schedule risk.** Item 5. The catalogue is where the roadmap says booleans
actually cost, and its size is unknown until entries start failing. §12.1's
lesson also applies here: the risk is not the part that looks hard, it is the
part that looks done — which here is classification, already shipped and
benchmarked in M0.

### 1.8 A crossing is never fitted to a B-spline (roadmap §6.3)

*Added 2026-08-31, while planning M3.*

Roadmap §6.3's fourth marching step is "approximate the resulting polyline as a
B-spline to tolerance". **We do not build it.**

§23.2 already settled that an edge is named by the surfaces it runs along and
never stores a curve, because a stored curve is a second authority beside those
surfaces and the first ulp of disagreement between them has no tiebreak. A
fitted B-spline is exactly that second authority, arriving with a tolerance
attached. §17.1 makes the same argument for planes.

What the fit was for — points along the crossing, at a chosen fineness — is
supplied instead by an evaluator that produces certified points on demand and
reports how far each may stray (§24.7). The approximation is then measured where
it is made rather than frozen into an object that outlives the measurement.

Recorded here because a later reader finds a marcher with no curve fitting after
it and reads it as a missing step.

---

## 2. Numeric model

### 2.1 Predicates return a decision, not a sign

```rust
pub enum Decision { Neg, Zero, Pos, Undecidable { enclosure: f64 } }
```

One type, so every call site must handle the fourth case. `Undecidable` carries
the enclosure width the roadmap's §6.3 asks callers to report.

A tolerant entry point exists alongside the certified one and is *separately
named*, so nobody reaches it by accident.

### 2.2 The cascade is built, not just wrapped

`robust` gives adaptive-exact `orient2d`/`orient3d`/`incircle`/`insphere`. It
fuses levels 1 and 3 internally and is always exact — so it never returns
`Undecidable`. Wrapping it would leave the third state as dead code from day one.

We build a small generic cascade — `f64` with a forward error bound, then
interval, then exact — and implement `orient3d` on it, **differentially tested
against `robust`**. That is the highest-value test available in M0: a known-correct
implementation validating the framework before anything depends on it.

### 2.3 Level 3 is expansion arithmetic, not rationals

Level 3 only needs the exact sign of a polynomial in `f64` inputs. Homogenize and
division never appears, so exact `+`, `−`, `×` on dyadic rationals is enough.

We write Shewchuk's expansion primitives (`two_sum`, `two_product`,
`expansion_sum`, `scale_expansion`) ourselves — roughly 200 lines, allocation-free
for small expansions, no dependency, no licence question.

Rejected: `rug` and `malachite` are LGPL (see §7.1). `dashu` is licence-clean but
heap-allocating, and buys division we do not need. It stays noted as an escape
hatch if some predicate ever genuinely requires division.

### 2.4 `inari` sits behind a newtype

`inari` is the certified-enclosure backbone, and it is a single-maintainer crate
whose last release was August 2024 with roughly 59k downloads. Its core is pure
Rust; the `gmp-mpfr-sys`/`rug` features stay off, which also keeps §7.1 clean.

`rcad_predicates::Interval` wraps it and exposes only what we use, with the
version pinned exactly. If `inari` goes unmaintained, that is a bounded porting
job instead of a rewrite.

Rejected: writing our own directed-rounding interval type. Rust has no stable
directed-rounding intrinsics; it means hardware rounding-mode manipulation, and it
is the kind of work that eats a month and produces something less correct than
what it replaced.

**Amended 2026-08-30, during M0 implementation.** `inari` does not build on
baseline x86-64: its directed-rounding primitives exist only for AVX+FMA and
AVX-512, and the crate emits `compile_error!` otherwise. Both 1.1.0 and 2.0.0
behave this way. `.cargo/config.toml` therefore sets
`-C target-cpu=x86-64-v3` for `x86_64`, making Haswell (2013) the floor. AArch64
needs nothing. The version pinned is `=2.0.0`, not the August-2024 release this
section was written against.

This is a real constraint on where the kernel runs, and it is worth knowing that
the §2.4 newtype is now also the seam behind which a baseline-x86-64 fallback
would go, should one ever be needed.

### 2.5 `Undecidable` is reached through constructed points

M0's geometry is planar (§3.1), so there are no quadrics to make a predicate
struggle. Instead: run `orient3d` where one argument is the intersection of three
planes — a *constructed* point, known only as an interval. Level 1 fails, level 2
straddles zero, and level 3 cannot help because the input itself is inexact.

This is exactly what the roadmap §2.1 calls "an exact answer to the wrong
question", it needs no geometry beyond planes, and it is the case real kernels get
wrong. `Sphere` and `Cylinder` also ship as types with sign predicates, since those
predicates are cheap to write against the cascade.

### 2.6 Strictness travels as an explicit value

```rust
pub struct Policy {
    strictness: Strictness,          // Certified | TolerantFallback { max_slop }
    max_enclosure: f64,
    dynamic_range_check: Severity,
    symbolic_perturbation: bool,     // always false; see §1.5
}
```

**Amended 2026-08-30.** The budget is now a single distance, `max_wobble`, and
the working size lives on `Strictness::TolerantFallback` rather than a
precomputed slop being stored. §12.4 made the first half of that change and
§15.3 the second; this sketch is left as written so the two amendments have
something to be amendments to.

Passed as `&Policy` to every operation that can make a decision. **Not** a field on
`Model` — model state that silently changes results is the footgun this design
exists to avoid. **Not** thread-local.

Constructors: `Policy::certified()`, `Policy::tolerant(scale)`.

---

## 3. Data model

### 3.1 M0 geometry is planar

M0's stated exit criterion in the roadmap is "prove `&Model: Sync` with a parallel
measurement benchmark", but measuring anything needs surfaces (M1) and wires (M2).
As written, M0 cannot reach its own exit criterion.

Resolution: `Surface::Plane` and straight-line edges only. A real cube, real
volume/area/centroid by the divergence theorem over triangles. No NURBS, no
quadrature. Planar solids are also where exact predicates decide everything, which
is where §2.2's work belongs anyway.

Rejected: a generic `Model<G: Geometry>`. The roadmap's concrete `Surface` enum is
the right design, and generics would infect every signature to buy a milestone
boundary.

### 3.2 Locations compose from day one

`FaceRef { face: FaceKey, loc: Isometry, orient: Orientation }` with real
composition, not an identity placeholder. Every accessor written without location
composition is one that has to be found and fixed later, and the ones missed are
precisely the bug class this design exists to avoid.

Power-of-two translation snapping (roadmap §2.2) is offered as a named
constructor, not enforced. Some placements are genuinely arbitrary, and silently
moving the user's part is worse than a slightly worse-conditioned intersection.

### 3.3 Views are the enforcement point, never `&FaceData`

`FaceView<'_>`, `EdgeView<'_>`, `VertexView<'_>` from the first accessor, carrying
the borrow *and* the composed transform. Raw `&FaceData` is never public.

The roadmap frames views as an ergonomics investment (§13). They are, but that is
not why they ship in M0: handing out `&FaceData` hands out un-located geometry, so
the view is the mechanism that makes §3.2 impossible to forget.

### 3.4 Points are our own types

`Point`, `Vector`, `Isometry` are newtypes in the public API with `From`/`Into`
both ways; `nalgebra` is used freely inside. `nalgebra` is at 0.35 and ships
breaking releases regularly — if `nalgebra::Point3<f64>` appeared in the public
API, every `nalgebra` bump would be a breaking release for every downstream user,
which directly damages the roadmap's §13 "ship M1 standalone" mitigation.

### 3.5 Units and working scale

```rust
Model::new(Scale { unit: Unit::Millimetre, nominal_size: 100.0 })
```

Coordinates are plain `f64` in the declared unit. Default tolerances derive from
`nominal_size`. The unit is carried for interop (STEP, roadmap §9.1) and never
participates in arithmetic.

A validity check computes `max coordinate magnitude / smallest feature size` and
**fails loudly** above roughly 1e11. The roadmap §2.2 names this as the case to
detect and report rather than silently degrade on, and it costs almost nothing.

Rejected: typed quantities (`uom`) — heavy, poor `nalgebra` interop, infects every
signature. Rejected: unit-agnostic `f64` plus a doc comment — OCCT's mistake.

### 3.6 Public surface

Public in M0: `Model` construction, typed keys, the views, measurement,
`validate()`, serialization, `Policy`, `Decision`, `Approximation`. Everything
else `pub(crate)`.

`#[non_exhaustive]` on every public enum — `Surface`, `Error`, `Cause`,
`Decision`. All of them grow at every milestone; without it each milestone is a
semver break. `Decision` especially, since §2.1 makes every call site match on it
exhaustively on purpose.

`publish = false`, version `0.0.0` until M1. M0 has nothing a stranger would want.

### 3.7 `Orientation` is exhaustive

**Added 2026-08-30, during M0 implementation.** `Orientation` carries `Forward`
and `Reversed` and will never carry anything else — there is no third way round
a face. §3.6's rule is about enums that grow, and this is not one of them, so
marking it `#[non_exhaustive]` would only force a `_` arm at every match in
`rcad-topo`: noise that hides nothing and that would silently swallow a variant
if one ever did appear. It stays exhaustive. `Surface`, `Error`, `Cause`,
`Decision` and `Unit` keep the attribute; all of those grow.

### 3.8 The default tolerance is a billionth of the nominal size

**Added 2026-08-30, during M0 implementation.** §3.5 says default tolerances
derive from `nominal_size` without saying by how much. The factor is `1e-9`
(`Scale::RELATIVE_LINEAR_TOLERANCE`), so a 100 mm part gets `1e-7` mm — the same
figure OCCT uses for confusion at that working size.

The reasoning: `f64` resolves about one part in `2.2e-16`, and `1e-9` leaves
roughly seven decimal digits above that for rounding to accumulate over a long
chain of operations without eating the tolerance. It is one named constant, so
changing it later is a one-line change and a re-run of the suite.

---

## 4. Errors and invariants

### 4.1 Three tiers

The roadmap says "never panic, return `EvalError`" (§4.5). `CLAUDE.md` says
"crash > corrupt" and "assert invariants". One rule:

| Situation | Response | Why |
|---|---|---|
| Bad caller input — parameter outside the domain, zero weight, empty knot vector | `Err` | It is data, not a bug |
| Broken internal invariant — edge with three coedges, key resolving to a freed slot, approximation grew | **panic** | It is a bug; emitting a corrupt solid is worse than dying |
| Cannot decide | `Decision::Undecidable` | It is a result, not a failure |

### 4.2 Error types

`thiserror` enums per crate. No `anyhow` in library code. A `#[non_exhaustive]
rcad::Error` at the facade wraps the crate errors.

`Undecidable` is a variant at the *operation* level (`Error::Undecidable { records }`,
carrying the §5 records that caused it) even though it is a `Decision` variant at
the *predicate* level. A predicate that cannot decide is a result; an operation
that cannot decide is a failure.

### 4.3 Validation is local by default

The roadmap §5.4 says assert invariants "after every operation, not at the end".
Taken literally with a full-model sweep, a test performing *n* operations becomes
O(n²) and the suite stops being runnable long before M4.

- **After every mutation, in debug**: cheap checks on *touched entities only* —
  keys resolve, coedge count, orientation agrees, approximation did not grow.
- **At test and import boundaries**: full `Model::validate()`, public API, called
  by `proptest` and by anyone importing untrusted data.
- **Always, including release**: the approximation-growth check. One comparison,
  and §11.2's whole argument is that catching inflation at the source turns a
  bisection session into a line number.

---

## 5. Provenance

Split into two pieces, because a pointer into a log dangles the moment the model
is serialized without it.

**On the entity** — small, self-contained, serializable:

```rust
pub struct Approximation { magnitude: f64, cause: Cause }
pub enum Cause { TolerantFallback, ImportedUncertainty, UndecidableResolved, /* … */ }
```

**In the returned log** — full records with the predicate, its inputs and the
enclosure width. Returned by value in the operation result. Not stored in `Model`,
not serialized.

Detailed records are emitted **only on non-certified outcomes**. A fully certified
operation produces an empty log, which is cheap and is itself a meaningful signal.
Recording every predicate call in a boolean would be millions of records.

A `&mut dyn ProvenanceSink` for streaming is deferred until a caller needs it.

---

## 6. Serialization

`serde` + `postcard`, explicitly **unstable and version-tagged**. Revisit `rkyv`
at M2 when the data model has settled.

The roadmap §11.3 wants the fuzz corpus and the file format to be the same thing,
which argues for stabilizing early. But the model will churn violently through M1
and M2, and `rkyv`'s archived-type derives fight every structural change. Storing
a corpus in a format you intend to break is how you end up unable to break it.

**Fuzz corpus is generated from constructor code, not stored as files**, until
M2. `cargo-fuzz` is deferred to M2 for the same reason.

Going through `serde` also means `slotmap`'s internal layout stops mattering,
which resolves the tension with §1.3.

---

## 7. Workspace and dependencies

### 7.1 Licence: Apache-2.0

`nalgebra` is Apache-2.0 only, so an MIT-only workspace was never available unless
we dropped it. Apache-2.0 also carries an explicit patent grant, which is worth
having in a CAD kernel.

**LGPL is a hard exclusion for runtime dependencies** — it puts static-linking
obligations on anyone embedding the kernel. This rules out `rug` (LGPL-3.0+, and a
GMP C dependency) and `malachite` (LGPL-3.0-only). `cargo-deny` enforces this in
CI so a violation fails the build rather than being noticed in review.

OCCT's test corpus (roadmap §11.1) is LGPL-2.1-with-exception. It stays **out of
the repo entirely**, referenced by a fetch script, so its licence never touches
our tree.

### 7.2 Crates

| Crate | Contains | Depends on |
|---|---|---|
| `rcad-predicates` | Cascade, expansion arithmetic, `Interval`, `Decision`, `Policy` | `inari`, `robust` (test only) |
| `rcad-geom` | Analytic primitives, later NURBS. **The M1 standalone crate** | `rcad-predicates`, `nalgebra` |
| `rcad-topo` | Arena `Model`, keys, views, serialization, validity | `rcad-geom`, `slotmap` |
| `rcad` | Facade | all |

Predicates split by what they need: `orient3d`/`incircle`/`insphere` take only
points and live in `rcad-predicates`; quadric-sign predicates need quadric types
and live in `rcad-geom`, implementing a trait from `rcad-predicates`. This keeps
`rcad-geom` free of any topology dependency, which is what makes the M1 standalone
story possible.

### 7.3 `rayon` is a dev-dependency in M0

The roadmap §10's central claim is that parallelism is *free* because
`&Model: Sync`. If that is true, the M0 library needs no `rayon` at all — the
caller parallelizes, and the benchmark proves it from outside.

This is the more honest test. If the benchmark hits its scaling target using only
the public API and no cooperation from inside the crate, §10's claim is
demonstrated rather than asserted. If it cannot, we learn that in M0 instead of at
M2.

A default-on `parallel` feature arrives at M2, when tessellation needs internal
parallelism.

### 7.4 Lints

`#![forbid(unsafe_code)]` in every crate we write. Dependencies contain plenty
internally; that is their business and it is audited by far more users than ours
will be.

`#![deny(missing_docs)]` on public API. Clippy at `deny(warnings)` in CI with a
documented allow-list, rather than blanket `pedantic` — which produces more noise
than signal on numeric code.

---

## 8. Tooling and repo

### 8.1 mise owns the tools, rustup owns the compiler

`rust-toolchain.toml` pins the compiler and its components (`clippy`, `rustfmt`,
`rust-src`). It is rustup's native mechanism and the thing rust-analyzer, IDE
plugins, `docs.rs` and CI actions already read.

`mise.toml` pins everything rustup cannot: `cargo-nextest`, `cargo-deny`,
`cargo-fuzz` (M2), `taplo`, `just`. With `settings.lockfile = true` and `mise.lock`
committed — confirm the lockfile covers the `cargo:` backend and not only
`aqua:`/`ubi:` before relying on it.

Nothing is unpinned. This just declines to make mise fight rustup over the one
thing rustup already does natively. CI runs `mise install` then `mise exec`, never
installing tools ad hoc.

### 8.2 `just` recipes, shared with CI

`just check` / `test` / `lint` / `deny` / `bench`. CI invokes the same recipes, so
local and CI cannot drift.

### 8.3 Git

Conventional Commits, scoped by crate name — makes per-milestone changelogs free.
Branch per change, squash merge. The branching earns its keep specifically because
"every change needs a test, and it must fail without the change" is a claim
somebody has to check, and a pull request is where that check has somewhere to
live.

---

## 9. Testing

### 9.1 What M0 ships

`proptest`, `cargo-nextest`, `criterion`, `cargo-deny`. `cargo-fuzz` deferred to
M2 (§6).

**Generators: extruded random simple polygons, with holes.** Not random convex
hulls — those are always valid and always convex, so they never exercise a
non-convex wire or an inner loop, which is where wire orientation invariants
actually bite. Convex hulls are tempting because generation is easy, and that ease
is exactly why they test nothing interesting.

Shrinking matters more than generation (roadmap §11.3). Budget for it.

The two tests that matter most in M0:

- `orient3d` differentially against `robust` on adversarial inputs — near-
  degenerate, collinear, wide dynamic range.
- **Transform invariance**: an arbitrary rotation must not change volume. §11.2
  says start here, and it costs nothing.

### 9.2 The parallel benchmark measures point classification

Planar volume by the divergence theorem is a handful of flops per face. Fanning
that across rayon measures the scheduler, then invites a false conclusion about the
data model.

The workload is **point-in-solid classification over a sampled grid**: real work
per query, exercises the predicate cascade under concurrency, and it is the
decomposition-invariant oracle §11.2 needs anyway — so none of it is throwaway
benchmark code.

Two pass criteria:

1. ≥0.8× linear scaling to physical core count.
2. **Bit-identical results to the serial run.** This is the one that will actually
   find a bug — it catches any accidental order-dependence in the cascade or in
   provenance recording.

---

## 10. M0 exit criteria

1. Workspace per §7.2; mise + `rust-toolchain.toml` per §8.1; `just` + CI per
   §8.2; `cargo-deny` enforcing §7.1.
2. Shewchuk expansion primitives; own `orient3d`, differentially tested against
   `robust`.
3. Tiered cascade with `Decision`; `Undecidable` reachable via the constructed-
   point case (§2.5); `Interval` newtype over `inari`.
4. `Policy`, `Approximation` tags, provenance records returned per operation.
5. `Model` on `slotmap` with typed keys; `FaceRef` with real location
   composition; views only, never `&FaceData`.
6. `Plane` participating in topology; `Sphere`/`Cylinder` as types with sign
   predicates only.
7. Planar solid construction, measurement, `Model::validate()`, local invariant
   checks after every mutation.
8. `serde` + `postcard` round-trip, explicitly unstable and version-tagged.
9. `proptest` over extruded polygons with holes; transform-invariance test;
   dynamic-range check.
10. Parallel point-classification benchmark: ≥0.8× linear scaling, bit-identical
    to serial.
11. `docs/roadmap.md`, `docs/decisions.md`, rewritten `CLAUDE.md`.

**Schedule risk.** Item 3's constructed-point path is the only one not confidently
scoped. It is the first place the interval machinery meets a real geometric
construction, and that is where this kind of estimate is usually wrong. If M0
slips, it slips there.

---

## 11. Deliberately deferred

| Thing | Until | Why |
|---|---|---|
| Bump arena in signatures | M2.5 | §1.4 — nothing to measure yet |
| Symbolic perturbation | Never, unless a caller asks | §1.5 — competes with `Undecidable` |
| `rkyv` zero-copy format | M2 | §6 — model still churning |
| `cargo-fuzz` | M2 | §6 — needs a format worth fuzzing |
| `rayon` inside the library | M2 | §7.3 — tessellation is the first real need |
| `ProvenanceSink` streaming | On demand | §5 — no caller yet |
| Branded lifetimes for cross-model keys | Indefinitely | Debug-checked model id is the pragmatic fix |

**Amended 2026-08-30.** §1.7 inserts M2.5, and the planar fuse is where scratch
allocation first becomes measurable — candidate pair lists, split pieces, image
maps, per-face working sets, all short-lived. The bump arena row therefore moves
from M4 to M2.5; §1.4's reason for deferring it, "nothing to measure yet",
expires there rather than at M4. `rkyv`, `cargo-fuzz` and `rayon` stay at M2:
all three are gated on tessellation and a settled format, and M2.5 changes
neither.

**Amended 2026-08-30, while planning M2.5.** The bump-arena row moves back out
to M4, and the paragraph above is wrong. It reasoned from the allocating end —
the fuse allocates, therefore the arena has arrived — and never named anything
inside M2.5 that would consume a `Bump` or any figure the arena would have to
beat. §1.7's eight exit criteria are what M2.5 is judged on and not one of them
mentions allocation, so the criteria win over the amended table. §13 rule 3 is
the general form: no named consumer inside the current milestone, no build, and
a viral `'bump` lifetime threaded through every fuse signature for a win nobody
has a target for is the largest possible version of the mistake that rule
exists to stop. M4 is where the consumer appears — a fuse over curved
intersections, run over the many-tool batches roadmap §7.2 is written for, is
the first scratch big enough for §1.4's "measure rather than assume" to come
back with a number instead of noise.

---

## 12. What M0 actually cost, recorded 2026-08-30

M0 is complete. Every criterion in §10 is met. This section records what the
work taught us that §1–§11 could not have known, because those were written
before any code existed.

### 12.1 The schedule risk was in the wrong place

§10 said that if M0 slipped it would slip on the constructed-point path, item 3,
"the first place the interval machinery meets a real geometric construction."

It did not slip there at all. `meet_of_three_planes` produces a genuine
`Point::Constructed` box, the float level declines because an input is inexact,
the interval level straddles, and the exact level cannot run — `Undecidable`
falls out of the design rather than having to be arranged. It was among the
easier pieces.

The cost went instead to **things that were already written and looked
finished**. Three separate bugs shipped through a passing test suite and were
found only by review:

- the level-1 filters certifying a wrong sign under underflow (§12.2),
- `ProfileBuilder` refusing a right triangle (§12.3),
- the face-winding check refusing a valid sliver face (§12.3).

A prediction to carry into M1: the risk is not the part that is obviously hard.
Attention goes there. The risk is the part that looks done.

### 12.2 A relative error bound has a precondition, and it is not optional

The level-1 filters compared `|value|` against `ERROR_BOUND × permanent`.
Shewchuk's bounds are *relative*, derived under `fl(x∘y) = (x∘y)(1+δ)`. That
model fails the moment a product lands in the subnormals: the error becomes
*absolute*, and `ERROR_BOUND × permanent` itself flushes to zero, so the
comparison passes unconditionally. `orient3d` certified `Pos` at `Level::Float`
on an input whose exact determinant is `−2⁻⁷⁶`.

Two things make this worth a section rather than a line in a changelog.

**We already knew.** `expansion::product_is_exact` exists precisely because
`robust` "returns a confidently wrong sign on coordinates near the bottom of the
`f64` range. A CAD kernel cannot afford that." The guard was applied at level 3.
Level 1 runs first and never reached it. Knowing about a hazard and guarding one
of the three places it appears is the shape this mistake takes.

**The differential test could not see it.** `robust` is wrong the same way, so
`orient3d` matching `robust` on every input was consistent with both being
wrong. §2.2 chose that test because a known-correct implementation validates the
framework — which holds only where the oracle is actually correct. What found it
was a *metamorphic* test: scale every coordinate by a power of two, which
multiplies the determinant by a positive power of two and so must preserve its
sign. That needs no oracle. Where an oracle and the code under test share a
lineage, prefer a property the answer must satisfy over an authority to compare
against.

The fix restores Shewchuk's own hypotheses rather than inventing a new bound:
every product in the float evaluation must be normal, and the bound must itself
be normal. Both are required — `2⁻⁵³ × 2¹⁰⁰⁰` amplifies a damaged minor back
into the normal range, so a bound-only check leaves the flagship case live.
Addition needs no guard: by Hauser's lemma a sum landing in the subnormals is
exact.

`orient2d` is the one filter that was always safe, and now says so in a comment.
Round-to-nearest is monotone, so `fl(f) − fl(b)` has the true sign or is zero,
never the opposite. The comment matters more than the code: without it the next
reader adds a guard for symmetry and pays for it.

Level 1 costs about 20% more on ordinary `orient3d` input. That is the price of
the certification meaning what it says.

### 12.3 A generator that makes one family of shapes finds one family of bugs

§9.1 rejected random convex hulls because they are always valid and always
convex, so they never exercise a non-convex wire or an inner loop. That
reasoning was right and did not go far enough. What shipped was a generator of
notched stars with strictly increasing angles about the origin — non-convex,
with holes, and still a monoculture.

`ProfileBuilder` refused a plain right triangle as self-crossing, because
`check_the_corner_does_not_double_back` called a bounding-box helper whose
precondition is "the point is already known to be on the line" without ever
establishing that. Measured afterwards: the broken rule refused 92% of generated
triangles and **0%** of star outlines. A 335-test suite, including a property
suite, could not see the most basic polygon there is.

The same review found `random_extrusions` using `prop_filter_map(… .ok())`,
silently discarding every profile the builder refused — the exact mechanism by
which a wrongly-refused input stays invisible. **A property suite must assert
that valid input is accepted, not only that invalid input is rejected.** One
direction alone is satisfied by a function that refuses everything.

The generator now also makes triangles and sheared combs, and `build` is
`expect`ed rather than filtered, with the by-construction soundness argument
written down. The stronger evidence is an exhaustive one: every 3-, 4-, 5- and
6-point ring on a small integer grid — 160,179 rings — checked against an
independent integer oracle, zero wrongly refused and zero wrongly accepted.

Routing the face-winding check through the cascade (it had been deciding on a
bare `f64` sign, the only place in the kernel that did) then turned up a second
wrongly-refused input of the same kind: a sliver face whose two cross products
differ by one, the larger not representable, so the sum is exactly `0.0` and a
good face is called inside-out.

### 12.4 An enclosure carries the units of the predicate that produced it

`Policy::tolerant(scale)` set `max_slop = 64·ε·scale³`. That is a volume, right
for `orient3d` and wrong for every other degree. Translated into permitted
positional wobble, `orient3d` gives a scale-invariant `64ε` while `orient2d`
gives `64·ε·scale²` — off by a factor of the working size, and growing with it.

`Policy` now stores the working size rather than a precomputed budget, and
derives it from what a predicate measures (`Measures::Length | Area | Volume`).
Storing the raw size means there is no second number to fall out of step.

No end-to-end outcome flip was ever built, and the reason is worth recording:
`orient2d` takes exact `[f64; 2]`, so the only enclosures it can produce are
subnormal or infinite, never the window the bug opens. Reaching that window
needs constructed points, which only `orient3d` accepts, and its degree was
already right. The bug was real, latent, and would have become reachable the
moment a second predicate took constructed input.

### 12.5 Corrections to earlier sections

- **§2.4**: `inari` does not build on baseline x86-64. Amended in place.
- **§3.6**: `Orientation` is deliberately exhaustive; see §3.7.
- **§4.3**: the always-on approximation-growth check is a **guard on future
  code, not an active detector**. Nothing in M0 can inflate slop
  unaccounted-for — the only writers are the `set_*_approximation` setters, and
  a caller reaching those has already chosen the budget. Debug builds
  additionally check that the cached figure still matches the arena, which
  catches slop written into an entity directly rather than through a setter.
  Recorded here so the check is not read as protection it does not provide.
- **§5**: two operations were dropping their records on the success path
  (`Model::validate`, `ProfileBuilder::build`). Both now take a
  `&mut ProvenanceLog`. The convention is now explicit: an operation that can
  make a non-certified decision takes `&Policy` and `&mut ProvenanceLog`; one
  that cannot takes neither, because a log that is always empty is noise rather
  than accounting.
- **§9.2**: criterion 10 passes at 0.89–0.93 of linear to 12 physical cores,
  with `rayon` a dev-dependency and the library unchanged — §7.3's claim
  demonstrated rather than asserted. Two honest caveats. The **≥0.8× half is not
  machine-checked**; it is a `criterion` run a human reads. The bit-identity
  half *is* a test the normal suite runs. And the comparison's `enclosure` and
  provenance fields **carry no signal today**: with exact vertices under a
  certified policy every query certifies, so the log is empty and nothing is
  `Undecidable`. §9.2's "or in provenance recording" clause is not yet exercised
  and cannot be until classification meets constructed points.

### 12.6 Known gaps carried into M1

- `Policy::max_enclosure` and `enclosure_is_acceptable` have no caller anywhere
  in the workspace. The natural one judges an operation's worst enclosure and
  belongs in `rcad-topo` or the facade; `ProvenanceLog::worst_enclosure()` is
  the obvious input. §2.6 lists the field, so this is a missing caller, not a
  field to delete. **Amended 2026-08-30:** §1.7 names the caller — the M2.5
  fuse, exit criterion 7.
- `Approximation` magnitudes from a tolerant `orient2d` are areas, while
  §5.4's `tol(vertex) ≥ tol(edge) ≥ tol(face)` ordering wants lengths.
  Consistent within one predicate degree, wrong across degrees. Same root cause
  as §12.4 and not yet fixed at the tagging end.
- A tolerant `add_extrusion` tags every vertex, edge and face it builds with one
  approximation for the whole profile. Mapping a record back to the corner it
  came from is the real job and has not been started.
- `Model::validate`'s `allowed_wobble` is `64·ε·largest_coordinate`. A rigorous
  bound for `signed_distance_to` is about `3·ε·largest`, so the magnitude is
  sane but the factor is a guess. It should come from a bound published next to
  `Plane::signed_distance_to` in `rcad-geom`, the way `ORIENT3D_ERROR_BOUND`
  sits beside the expression it bounds.
- `PredicatePoint::enclosure()` returns `[Interval; 3]`, so the facade's rustdoc
  names a type §3.6 deliberately keeps private. Cosmetic today; the seam to
  watch if it bites.

The second and fourth of these are the same kind of mistake — a meaning settled
in one place and not carried to the other. §13 rule 4 is the general form.

**Amended 2026-08-30.** The first, second and fourth are settled in §15 —
§15.3, §15.2 and §15.6 respectively — because M1 and M2.5 both push on them and
two heads settling them separately is how §12.4 happened.

---

## 13. How work is decomposed

*Recorded 2026-08-30, alongside §12.*

§1–§12 say what to build. This says how the work is cut into pieces, because
M0's cost landed somewhere the milestone list cannot show. §12.1: the hard part
was easy, and the money went to code that was already written and looked
finished.

Every gap in §12.6 is a **seam**. A field with no caller. An `Approximation`
magnitude in the units of the wrong predicate degree. An error bound living in a
different crate from the expression it bounds. A private type leaking through the
facade. None of them is a bug inside a unit; all of them are bugs between two.
That is the failure this section exists to bound, and running more than one piece
of work at a time makes exactly that failure more likely.

**1. Split by oracle, not by milestone subpart.** A piece can be worked on
independently when the property that decides it is correct can be written down
*before* the work starts: a textbook formula, a differential test against an
implementation sharing no lineage (§2.2), or a metamorphic relation (§12.2). A
piece whose specification is discovered by writing it is design work, and design
work does not split.

**2. A milestone is shaped gate, fan, gate.** Not a uniform graph. M1
concretely: settle the `Curve` and `Surface` extension, the knot vector type and
its invariants, the homogeneous-internal / affine-external boundary of roadmap
§4.1 and the `EvalError` cases of roadmap §4.5 — one pass, one head. Then the
Piegl & Tiller algorithms fan wide, each with its own textbook oracle. Then
rejoin for surface poles, continuity classification, and the seam where a NURBS
point becomes a `PredicatePoint` carrying an enclosure.

**3. No named consumer, no build.** §12.6's first gap is a field specified in
§2.6, implemented, tested, and called by nothing. §11 already defers
`ProvenanceSink` for exactly this reason — "no caller yet". Generalize it: a
piece with no consumer inside the current milestone is deferred to the milestone
where its consumer appears.

**4. Settle shared meanings before the fan-out, in writing.** §12.4: an enclosure
carries the units of the predicate that produced it. That is one decision, and
two people making it independently make it twice, differently. Before M1 fans
out, settle here what an `Approximation` magnitude measures across predicate
degrees, and that an error bound is published beside the expression it bounds —
the way `ORIENT3D_ERROR_BOUND` sits beside its expression and
`Model::validate`'s `allowed_wobble` does not.

**5. Vary the generators across pieces.** §12.3: one family of shapes finds one
family of bugs. Two pieces sharing a generator report green together, and the
agreement means nothing.

**6. Review is the constraint, not authoring.** §12.1: three bugs shipped through
a passing suite and were found only by review. More work in flight produces more
code per unit of review, which pushes on the constraint from the wrong side. Cap
work in flight inside a single crate at two or three pieces. Go wider only across
crate boundaries that are already stable contracts.

| Splits | Does not split |
|---|---|
| Roadmap §4.2–4.4's algorithm list — each one is a textbook formula | What a shared type means: enclosure units, `Approximation` semantics, the `Decision` contract |
| Cone and torus sign predicates, once the cascade is fixed | Surface–surface marching (roadmap §6.3) — the branches interact |
| Roadmap §7.4's catalogue written as test cases | Boolean classification and shell assembly — graph traversal |
| `cargo-fuzz` harness, `criterion` benches, `ruststep` plumbing | Review |

Rule 3's operational half is also in `CLAUDE.md`, because that is what is read
every session. The reasoning is here.

---

## 14. The M1 oracle plan

*Recorded 2026-08-30, before M1.*

§13 rule 1 makes the oracle list and the work breakdown the same document: a
piece of M1 can be worked on independently exactly when the property deciding it
is correct is written down before the work starts. So this list is not a test
plan appended to a design — it is what says which parts of M1 fan out and which
stay in the gate. A candidate algorithm with nothing here is gate work.

### 14.1 What a candidate has to answer

`rcad-predicates/tests/scaling_a_predicate_must_not_change_its_answer.rs` is the
worked example. Its module comment answers all four of these, and that is why it
found a bug the differential suite next door could not see.

1. **Is the relation exactly true, or does it need a bound?** Exactly-true
   relations cost nothing to trust. A bounded one is worth nothing until the
   bound is published beside the expression it bounds (§13 rule 4); without that
   the tolerance gets tuned until the test passes, and then the test is
   measuring the tolerance.
2. **Do the two runs take different paths through the dangerous code?** The
   discriminating question. Scaling by a power of two works because one run
   decides at level 1 and the other does not. Two runs down the same branch
   prove nothing, however pretty the algebra.
3. **What family of inputs does the generator make?** §12.3. Different per
   piece, deliberately, or two pieces report green together and the agreement
   means nothing.
4. **Which gap in §12 would this have caught?** If none, say why it is being
   written anyway.

### 14.2 What M0 already covers

| Kind | Stock |
|---|---|
| Metamorphic | scaling by 2ᵏ on predicates and surfaces; rigid-motion algebra — round trip, associativity, identity, distance preservation, homomorphism; volume under rotation; serialization round trip |
| Differential | `orient2d` and `orient3d` against `robust`, with the shared-lineage blind spot §12.2 records |
| Refusal | dynamic range, error shapes, undecidable through constructed points |

Nearly all of it is **invariance**. There is no closed-form oracle anywhere in
the tree, because planar M0 had nothing whose answer is known in advance. M1 is
the first milestone where that axis exists, and it is the cheapest new coverage
available.

### 14.3 Exactly true, so free to trust

Write these first.

- **Partition of unity.** `Σ Nᵢ,ₚ(u) = 1`. The sum has p+1 terms, so the bound
  is derivable rather than guessed. A direct test of Cox–de Boor.
- **Basis functions are never negative and never exceed one.** Together with
  partition of unity this gives the convex hull property, far cheaper than
  testing hull containment. Roadmap §6.2's Bézier clipping rests on that
  property, so this is M1 protecting M3.
- **Knot insertion is bit-exact outside the affected span.** Boehm's algorithm
  touches p+1 control points and copies the rest, so evaluation in an untouched
  span must agree to the last bit. This is the sharp form. "Insert a knot and
  the curve is unchanged" everywhere is the same idea needing a bound, and
  belongs in §14.5.
- **The split seam.** At a knot of multiplicity p the curve passes exactly
  through a control point, so both halves evaluated at the seam must agree bit
  for bit.
- **Scaling by 2ᵏ, extended to evaluation and derivatives.** A curve scaled by s
  has every derivative order scaled by exactly s. Reuses M0's generator whole
  and aims §12.2's bug family at the new arithmetic.

### 14.4 Closed form — the axis M0 could not have

- A **rational quadratic Bézier circle** is exact by construction, so evaluated
  points must satisfy M0's sphere sign predicate. No shared lineage at all.
- **Curvature of a circle is `1/r`, of a line `0`. Gaussian curvature of a
  sphere is `1/r²`, mean curvature `1/r`.** Tests roadmap §4.3's fundamental
  forms against known answers. Fix the mean-curvature sign convention in the
  gate — it is a coin flip, and two pieces of a fan-out will call it
  differently.
- **Arc length of a circular arc is `rθ`.** Tests roadmap §4.4.

### 14.5 Bounded, and therefore gated

Degree elevation preserves the curve; splitting reproduces the original on each
subdomain; all weights equal reproduces the non-rational B-spline. All three are
good relations and none can be written until §13 rule 4's bounds exist. They are
gate work, not fan-out work, and listing them here is how that stays visible.

### 14.6 Differential is weak here

de Boor against Cox–de Boor is tempting and close to worthless: one author, one
book, one set of conventions, so a misread convention appears in both. That is
§12.2's failure exactly. `curvo` and `truck-geometry` are genuinely independent
lineage and worth a low-weight check if their licences allow it. Neither may be
the only check on anything.

### 14.7 The acceptance oracle, and why it goes first

Every refusal-shaped test in the tree checks that bad input is refused. Nothing
checks that a refusal was **deserved** — that `Undecidable` came back because the
case is genuinely undecidable and not because a filter was timid.

§12.3 already stated the general form: a property suite must assert that valid
input is accepted, not only that invalid input is refused. M1 needs the two-sided
version — cases whose answer is exactly computable, which `Policy::certified()`
must decide. Build it first, not last. Without it a cascade that refuses far more
often than it must passes every other oracle on this list, and §1.7's exit
criterion 8 records a number about our filters rather than about the geometry.

### 14.8 Carried to M2.5

Not M1, recorded here because it came out of the same pass. §1.7's exit criteria
5–8 name the boolean oracles; three more belong with them. **Commutativity** of
union and intersection, which catches argument-order dependence — the defect
§9.2's bit-identical criterion exists to find. **Idempotence and absorption**:
`A ∪ A ≡ A`, `A ∩ A ≡ A`, `A \ A ≡ ∅`, `A ∪ (A ∩ B) ≡ A`. And
`fuse([A,B,C]) ≡ fuse([fuse([A,B]), C])`, which takes genuinely different paths
and tests the whole reason roadmap §7.2 exists.

---

## 15. Shared meanings, settled before M1 and M2.5 fan out

*Recorded 2026-08-30, before either milestone starts.*

§13 rule 2 says a milestone is shaped gate, fan, gate. §13 rule 4 says the
shared meanings get settled in the gate, in writing, because two people making
the same decision independently make it twice, differently — §12.4 is what that
looks like afterwards.

M1 and M2.5 are going to run side by side (§15.1). That makes one gate serve two
milestones, so this section is it. Everything here is a meaning that both would
otherwise have to decide, and every one of them is decided here rather than in
either milestone.

### 15.1 M1 and M2.5 do not depend on each other, and M2.5 does not need M2

`rcad-topo` depends on `rcad-geom` and never the other way round (§7.2). M1 is
work inside `rcad-geom`; the M2.5 fuse is work inside `rcad-topo`. Neither
consumes anything the other produces:

- The planar fuse needs `Plane` — including `basis`, `plane_coordinates_of`,
  `point_at` and `meet_of_three` — plus `Segment`, `Point`, `Isometry` and
  `Surface::as_plane`. All of that shipped in M0. Nothing in a knot vector or a
  control net reaches it. Edges do not even store a curve (§3.1), so M1's
  `Curve` never appears in the fuse's path.
- M1 consumes nothing from topology. That is the whole point of §7.2, and it is
  what makes "ship M1 standalone" possible at all.

**M2.5 also does not need M2.** M2 is tessellation and rendering on curved
geometry, and curved geometry is M1's output. Building a planar solid and
validating it landed in M0 instead (§3.1), and classification already
triangulates faces to cast rays through. So the fuse's inputs are all present.

This is recorded as a departure in the §1 sense: the numbering says M1, M2, M2.5
in that order and the dependencies do not. Running M2.5 alongside M1 serves
§1.7's own argument, which is that the refusal rate is worth having early — it
arrives earlier still. A later reader who finds the two milestones landing
together should read this, not the numbering.

**What that costs, and the honest counter-argument.** §13 rule 6: review is the
constraint, not authoring, and two milestones in flight is more code per unit of
review. §12.1 is the evidence — three bugs shipped through a green 335-test
suite and were found only by review. Rule 6 permits going wider across a crate
boundary that is already a stable contract, and this one is, for planar work.
Permits, not recommends. If review starts slipping, the milestone to pause is
M2.5, because M1 is on the critical path to M2, M3 and everything after.

### 15.2 An `Approximation` magnitude is a length

**Settled: the magnitude on an entity is how far a point could be out of place,
in the model's own unit. Never the raw value of a predicate.**

§12.4 fixed the budget end of this and left the tagging end alone; §12.6's
second gap is the leftover. Today `resolve_under_policy` is handed the
`Measures` of the predicate that gave up, uses it to pick a budget, and then
stores the raw enclosure in the tag and throws the units away. So an
`Approximation` off `orient2d` is an area, off `orient3d` a volume, and off the
dynamic-range check a length — three different quantities in one `f64` field
with one name.

The conversion is already written down, in `Measures`' own doc comment: move a
corner by `w` at working size `s` and a predicate of that degree moves by about
`w · s^(degree−1)`. Turn it round:

```
wobble = enclosure ÷ working_size^(degree − 1)
```

Length divides by one and passes through unchanged, which is why the
dynamic-range check was already right. Area divides by `s`, volume by `s²`.

**What was already right, and what this adds.** Since
`max_slop = 64·ε·s^degree`, the wobble a tolerant policy permits already comes
to `64·ε·s` whatever the predicate degree — §12.4 got that far when it made the
budget carry units. What it did not do is put the *tag* in those terms, so the
number stored on an entity is still in the units of whichever predicate happened
to produce it. This rule finishes the job: budget and tag measure the same
thing, which is what lets §15.3 compare them and §5.4 order them.

The uniform `64·ε·s` is worth a test either way, because it is the property that
breaks first if a degree is ever added to `Measures` without its conversion.

**Which working size.** The one the tolerant policy already carries, since the
conversion happens inside `resolve_under_policy`'s tolerant branch and nothing
outside it produces a tag from an enclosure. No new plumbing. That the model's
`Scale::nominal_size` is a second copy of the same number is a seam worth
watching, not one to fix here.

**This is an accounting figure, not a bound, and that has to be said out loud.**
The true sensitivity of a predicate to a moved corner is its gradient at the
configuration in hand, and a sliver triangle has a small one — the same
enclosure there means a larger real wobble than `s^(degree−1)` suggests. What
the rule buys is a number that can be compared between two entities and summed
across an operation, which is the only thing a stored magnitude is for.

Rejected: use the actual gradient. More faithful, and it gives up the one
property that makes the number worth storing — the tag would depend on where the
part sits and what shape the triangle is, so two tags could no longer be
compared, and §5.4's ordering check would be comparing noise. It also costs a
gradient evaluation on every non-certified decision. Noted as the escape hatch
if the stand-in ever proves too loose to act on.

**The named consumer**, per §13 rule 3, already exists:
`check_approximation_ordering` in `validate.rs` compares vertex against edge
against face, and §5.4's `tol(vertex) ≥ tol(edge) ≥ tol(face)` is only a
statement about anything once all three are lengths.

### 15.3 An operation judges a wobble, and the field is named for it

`Policy::max_enclosure` and `enclosure_is_acceptable` still have no caller
(§12.6, first gap); §1.7's exit criterion 7 gives them one. Before that caller
is written, two things get settled.

**The operation judges in lengths.** Once §15.2 makes every tag a wobble, the
operation-level question — is this solid's worst slop small enough to hand
back — is a question about a length. So `max_enclosure`'s `Measures` argument is
not a choice the caller gets to make; there is exactly one right answer to pass,
and an argument with one right answer is a way to get it wrong.

**So the names change**, while there is still no caller to break:

| Was | Is | Means |
|---|---|---|
| `Policy::max_enclosure(Measures)` | `Policy::max_wobble()` | the widest a point may be out of place, `64·ε·working_size` |
| `Policy::enclosure_is_acceptable(f64, Measures)` | `Policy::wobble_is_acceptable(f64)` | judged against the above |
| `ProvenanceLog::worst_enclosure()` | `ProvenanceLog::worst_wobble(working_size)` | the worst wobble in the log |
| `Policy::max_slop(Measures)` | *gone* | see below |

**Amended 2026-08-30, while implementing.** `max_slop` is removed rather than
kept alongside. It was the same budget expressed in a predicate's own units, and
`resolve_under_policy` was its only caller. Leaving it would mean the predicate
judging an enclosure against one number while the operation judges a wobble
against another — equal in exact arithmetic, not equal in `f64`, so a decision
could be accepted at the predicate and then push a solid over the operation's
budget by an ulp. Instead the conversion happens once, before the judgement, and
the number that is judged is the number that gets stored. `Measures::wobble_of`
carries the units knowledge and has a real caller in `Record::wobble`, so §13
rule 3 is satisfied where it was not before.

`Record::enclosure()` stays as it is and keeps returning the raw value in the
predicate's own units — it is the honest record of what the cascade saw, and
that is what a record is for. `Record` grows a `measures()` so the units are no
longer lost at the moment the record is made.

`ProvenanceLog::worst_enclosure` is the one that has to go rather than merely
being renamed. It folds a maximum over records whose values need not share a
degree. No operation in the tree mixes degrees into one log yet — the profile
builder records only `orient2d` areas, classification only `orient3d` volumes,
`Model::validate` only the dynamic-range check's length, and the swept-area
check builds a fresh log inside the error it returns. Nothing stops the mixing
though: a caller may hand the same log to two operations, and `drain_into`
exists precisely so one operation's log can be folded into another's. The fuse
is the first thing that will mix degrees on purpose. That is §12.4 one level up,
waiting for a caller.

§2.6 lists these as fields of `Policy` and it still does. This renames them, and
records that the old names were the invitation: a field called `max_enclosure`
asks to be compared against a raw enclosure, which is the units bug it exists to
prevent.

### 15.4 `Decision` does not grow during the parallel window

§3.6 says `Decision` is `#[non_exhaustive]` because it grows at every milestone,
and §2.1 says every call site matches all its cases on purpose. Both are still
true. Together they mean a new variant landing in one worktree breaks every
match in the other — and the matches that do not break are worse, because
`classify.rs` and `validate.rs` handle an unknown variant by panicking, which
turns a merge into a runtime failure somewhere else.

**Neither M1 nor M2.5 adds a variant.** Nothing either needs one for: a NURBS
point that cannot be placed is `Undecidable`, which is what `Undecidable` is.
If one of them turns out to need one anyway, that is not a change to make inside
a fan piece — it stops the fan, reopens this gate, and gets written down here.

### 15.5 No decision is taken on projected coordinates

**Settled: the fuse decides in 3D, on stored coordinates, through `orient3d`.
`orient2d` keeps taking exact `[f64; 2]` and nothing else.**

Projecting into a face's plane is tempting, because splitting a planar face is a
2D problem and `plane_coordinates_of` is right there. It is also a way to throw
away exactness for free: `Plane::basis` normalizes, so projecting a vertex whose
3D coordinates are exact produces 2D coordinates that are not. Every projected
point is a constructed point, level 3 of the cascade can no longer run on it
(§2.5), and the fuse would be manufacturing the undecidability §1.7 exists to
measure. Exit criterion 8 would then report a number about our projection, not
about the geometry — §14.7's warning, arriving by a different road.

`orient2d`'s only caller today is the profile builder, whose input is `[f64; 2]`
from the caller and genuinely exact. That stays true.

If the fuse turns out to need a 2D predicate on constructed input, growing
`orient2d` to take a point that may be exact or constructed is gate work, not
fan work: it is a change to `rcad-predicates`, which §15.9 freezes.

### 15.6 A bound lives beside the expression it bounds; a safety factor lives beside its consumer

§12.6's fourth gap is `Model::validate`'s `allowed_wobble`, whose `64·ε` is a
guess where a rigorous `3·ε` exists. It is half fixed: `MEASURING_ROUNDOFF` now
derives the `3·ε` in its doc comment and says plainly that the twentyfold on top
is not derived. What it does not do is live in the same crate as
`Plane::signed_distance_to`, the expression it is a bound on.

M1 is about to write a great many bounds — partition of unity over p+1 terms,
evaluation, derivatives — so the rule matters more than the one case:

1. **A bound on an expression is a named public constant in the same module as
   the expression**, with its derivation in its doc comment. `ORIENT3D_ERROR_BOUND`
   is the pattern.
2. **A safety factor is a separate named constant at the consumer**, with its
   own reason. Multiplying a derived bound by twenty is a judgement about how
   much slack a caller wants, and it is not part of the bound.

Applied to the open case: the bound moves to `rcad-geom` next to
`signed_distance_to`; the slack factor stays in `validate.rs` under its own
name. This is why the fix lands before the fork — it touches `rcad-geom`, which
M1 owns from then on.

**Amended 2026-08-30, while implementing. The `3·ε` was not a bound.** §12.6's
fourth gap, and the paragraph above as first written, both called `3·ε` the
rigorous figure that `64·ε` was a guess at. Writing the derivation out properly
showed it is not: it counts half an ulp per *term*, but a rounding costs half an
ulp of the **running sum**, which is larger than any one term. Charging each
term for every operation it passes through, and using `|n₁| + |n₂| + |n₃| ≤ √3`
for a unit normal, gives `u·(4√3 + 1)·L` — just under `4·ε·L`.

So `Plane::SIGNED_DISTANCE_ROUNDOFF` is `4 * f64::EPSILON`, and the slack at the
consumer is 16 rather than the "little over twenty" the old comment described.
The product is `64·ε` either way, so nothing the kernel does changes; what
changes is that the figure is now derived rather than asserted.

Worth noting how it was caught, because it is §14.1 question 1 exactly: the
number had a derivation written next to it and was still wrong, because nothing
executed the derivation. The bound only became true when a test evaluated the
same expression in exact arithmetic and subtracted. **A published bound needs a
test that could fail, not a paragraph.** That test cannot show a bound is
*tight* — rounding errors cancel, and it passes at `2·ε` and `3·ε` too — so the
derivation is still what makes it a bound. The test is what stops the expression
drifting out from under it.

### 15.7 The intersection result type lives in `rcad-topo`

§1.7 requires the fuse to consume an intersection *result* that carries an
enclosure and an `Approximation` from its first line, so that M3's curved
intersector is a new producer rather than a new contract.

**It goes in `rcad-topo`.** §13 rule 3: its only consumer is the fuse, and the
fuse is there. Putting it in `rcad-geom` where M3's producer will eventually
live builds a thing with no consumer in the milestone that builds it — the
mistake §12.6's first gap records — and it would put the one type the two
milestones share into the crate M1 is rewriting.

Its `Approximation` is a length, per §15.2. The planar producer leaves it at
zero and fills in nothing else; that it is never populated in M2.5 is the point.

When M3 gives it a second producer, moving it down to `rcad-geom` is a
relocation and nothing more, since `rcad-topo` already depends on `rcad-geom`.
Recorded so that move reads as planned rather than as a layering mistake being
corrected.

**Amended 2026-08-30, while planning M2.5.** "Fills in nothing else" is half
wrong, and as written it would have someone wire the endpoint enclosure to
zero. The `Approximation` — how far the computed curve strays from the true
one — really is zero for two planes and stays zero forever: the meet of two
planes is a straight line and the plane pair *is* that line, with nothing
approximated about it. M3's marcher is its first producer, exactly as this
section says. **The endpoint enclosure is a different field and it is not zero
and never will be**, because `Plane::meet_of_three` solves by Cramer's rule and
Cramer divides. Every end of a planar face meet is therefore a constructed
point with a real box around it — §2.5's case, which §17.1 turns from the
exotic path into the only path. So the planar producer populates the enclosure
on every meet it returns, and it is only the fidelity that stays at zero.

### 15.8 "Refusal rate" and "deserved refusal" are two different numbers

§1.7's exit criterion 8 wants a refusal rate. §14.7 wants an acceptance oracle —
proof that a refusal was deserved rather than a timid filter. They are the same
worry from two sides, and left alone the two milestones would produce two
numbers with one name.

- **Refusal rate (M2.5, criterion 8)** is counted **per operation**: the share
  of fuse operations over the §1.7 corpus that come back `Error::Undecidable`
  under `Policy::certified()`. Per operation, because that is the unit a user
  experiences a refusal in, and because §1.7's risk is about whether the kernel
  is usable.
- **The acceptance oracle (M1, §14.7)** is **per case, pass or fail, not a
  rate**: cases whose answer is exactly computable, which `Policy::certified()`
  must decide. A single one failing is a bug, not a percentage going up.

Both get written down; neither gets called "the refusal rate" except the first.

### 15.9 What lands before the fork, and who owns what after

**Before either worktree opens.** Each of these needs a test that fails without
it, per `CLAUDE.md`:

1. `Record` carries its `Measures` and can report a wobble (§15.2, §15.3).
2. `resolve_under_policy` tags a wobble rather than a raw enclosure (§15.2).
   The test worth having is the falls-out property: one tolerant policy permits
   `64·ε·working_size` of wobble whatever the predicate degree.
3. `ProvenanceLog::worst_enclosure` becomes `worst_wobble(working_size)`; the
   `Policy` pair is renamed (§15.3).
4. The `3·ε` bound moves next to `Plane::signed_distance_to`, and the safety
   factor stays at its consumer under its own name (§15.6).

Items 1–3 are in `rcad-predicates` and item 4 is in `rcad-geom` — both crates
the fork would otherwise contend for, which is why all four land first. None is
large. Doing them afterwards means doing them twice.

**All four landed 2026-08-30.** Two things came out of the work that the plan
above could not have known, both recorded where they belong: `Policy::max_slop`
is removed rather than kept (§15.3), and `3·ε` was never a bound (§15.6). The
change is covered by four tests that fail without it — three on what a tag holds
and how a log is summed, one on the plane bound — and the suite is 404 green.

**After the fork.**

| Crate | Owner during the window | Rule |
|---|---|---|
| `rcad-geom` | M1 | M2.5 may **add** items — a two-plane meet beside `meet_of_three` is the expected one. It may not change what anything already there means. |
| `rcad-topo` | M2.5 | M1 has no business here. |
| `rcad-predicates` | nobody | Frozen. A change reopens this gate and stops both fans. |
| `rcad` facade | both | Re-export lists only. A textual conflict, never a semantic one. |
| `docs/decisions.md` | both | Append only. M1 writes §16, M2.5 writes §17; those numbers are reserved now so the two never collide. |

Two worktrees, and each gets its own `target/`. Sharing `CARGO_TARGET_DIR`
between them only makes the builds wait on each other's lock.

§13 rule 5 is satisfied without anyone arranging it: M1 generates knot vectors
and control nets, M2.5 generates extruded polygons and the §7.4 catalogue. The
two suites cannot go green together for the same wrong reason.

**Amended 2026-08-30, while planning M2.5.** The table's `rcad-geom` row
predicts "a two-plane meet beside `meet_of_three`". **That is a deliberate
non-addition: M2.5 will not write one.** The plane *pair* is the line already,
exactly and for free — `{A, B}` names it with no arithmetic at all — and every
question the splitter actually asks is a meet of three planes, which exists.
Turning the pair into a point plus a direction would normalize a cross product
and divide, replacing an exact object with a rounded one before anything used
it, and that is §15.5's mistake arriving by a different road: the fuse
manufacturing the undecidability §1.7 exists to measure.

What `rcad-geom` gets instead is one added predicate,
`Plane::agreement_with`, which answers whether two planes describe the same
plane by exact proportionality of their coefficient 4-vectors, and which way
each faces (§17.4). And `rcad-predicates`, which this table freezes, gets
exactly one additive change: `Cause::ConstructedPoint` (§17.2). It is taken
*before* M1's worktree opens rather than during the window, so the freeze in
the table is real rather than aspirational — the row means nobody may change
that crate while two fans are running, and running M2.5 alone until the change
has landed costs a day and satisfies it literally.

---

## 16. What M1 actually cost, recorded 2026-08-30

*Written by M1, per §15.9's reservation of this number.*

`rcad-geom` can now represent, evaluate and subdivide NURBS curves and surfaces,
and can hand a NURBS point to the predicate cascade **carrying an enclosure** —
the seam §13 rule 2 named as M1's closing gate. The suite went from 404 to 549.

`rcad-predicates` was not touched, so §15.9's freeze held. No `Decision` variant
was added, so §15.4 held. §12.6's fifth gap is closed: `Interval` is re-exported
from the facade, because callers now take `PredicatePoint::enclosure()` apart and
a type the rustdoc names has to be one the facade exports.

### 16.1 The schedule risk was named twice and landed in neither place

§12.1's lesson was that the risk is not the part that looks hard, so the M1 plan
named two candidates: the enclosure width, and — because it is textbook and
finished-looking — the de Boor evaluator. Neither failed.

On shapes of a stated working size the enclosure came in at **2 to 20 `f64`
steps of that size**, flat across working sizes from 1 to a million and across
degrees 1 to 5.

| | curve, well conditioned | curve, wide range | patch, well conditioned | patch, wide range |
|---|---|---|---|---|
| steps of the working size | 2–11 | 2.5–14 | 5–14 | 4–20 |

**The answer for M3 is conditional, and the condition is the interesting half.**
For well-conditioned input this settles it: tens of steps, not thousands, and
certified marching works. But the first version of this figure was measured with
every knot vector drawn from the same six small whole numbers — the monoculture
§16.9 records — and once the generators put a span of a million steps next to a
span of one, the tail moved. Over a million draws of the six-decades-of-weight
family: **median 15 steps, 99.9th percentile 986, worst 66,120.** That is a
parameter landing a hair below a knot in a span a million times wider than its
neighbour, which is the curve being ill-conditioned rather than the arithmetic
being sloppy — the box is honest about it, which is the whole point of having
one. **M3 will need centred forms for that case**, and it now knows which case,
eighteen months before it would otherwise have found out.

That family is therefore held to a scale-free ceiling — the enclosure's
half-width against the published `f64` bound, worst measured 0.145 — because a
ceiling in steps of the working size would have to sit near a million to stop
flickering, and a ceiling that loose asserts nothing.

Where the cost actually went was neither algorithm. It went to **defining the
numbers before measuring them** — §16.2 and §16.3 below, and the ceiling above.
All three are the same mistake in different clothes: a quantity that is
meaningless until you say what it is measured against.

### 16.2 An enclosure width has to be measured against the right thing

Reported in steps *of the coordinate*, degree-5 patches came back at 2·10⁷ and
looked exactly like the schedule risk landing. They were not. The worst case is a
coordinate of −1.2·10⁻⁷ with an absolute width of 5.6·10⁻¹⁶, which is five steps
of the control net that produced it. Dividing a scale-free width by a coordinate
that crosses zero says nothing about the arithmetic and everything about where
the curve happens to pass near an axis.

This is the same trap `EVALUATION_ROUNDOFF`'s own derivation opens with, and it
was walked into anyway, one file apart. The bound is stated in terms of the
largest control coordinate the evaluation touches *because* nothing relative to
the answer survives a de Boor result near zero — and then the width was reported
relative to the answer.

Both readings are now reported. The ceiling is on the second, and it is
machine-checked in the normal suite rather than read off a `criterion` run,
because §12.5 was honest that nobody notices one of those regressing.

### 16.3 A bound's operation count is not its rounding count

`EVALUATION_ROUNDOFF` charges **nine roundings per de Boor stage.** Counting the
operations gives seven, and seven is wrong. Two of the roundings form the
coefficient `a`, and their error is not carried down one chain — it is charged at
full size against the running values, because `|δa|·|d[j]|` does not shrink when
`a` is small.

§15.6 said a published bound needs a test that could fail, not a paragraph, and
M1 is the second run of that experiment. The test — the same recursion in exact
rational arithmetic, subtracted — is in the tree and it works. It is also **not
sufficient, measurably**: of about fifteen deliberate mutations of the evaluator,
every algorithmic one was caught, and one survived. Dropping the `w'` term from
the rational divide is worth about a factor of two, and the exact-arithmetic test
carries roughly 300× slack, so it only fails below about a five-hundredth of the
published constant.

That is not a defect in the test; it is §15.6's own claim arriving with a number
on it. **The derivation is what makes it a bound. The test is what stops the
expression drifting out from under the derivation.** Neither substitutes for the
other, and now we know the size of the gap between them.

### 16.4 A frozen crate is a design constraint, not only a process one

The M1 plan claimed no change to `rcad-predicates` was required, because
`Point::Constructed` and `Interval` are already public. That was very nearly
right and missed one thing: **`Interval` has no public division.** Interval de
Boor divides — at every knot difference and at the perspective divide — so the
enclosure could not be built from the frozen API as it stood.

It was built locally in `rcad-geom` instead, from `Interval::new`, `lo` and `hi`:
IEEE division rounds to nearest, so the true quotient lies between `q.next_down()`
and `q.next_up()`, and the four corner quotients widened by a step each way
enclose. It costs a couple of steps of width over a directed-rounding divide.

Recorded because the outcome was better than the shortcut would have been — the
soundness argument is local, written down and tested — but the freeze was **not
free**, and a future gate should not assume a frozen crate's public surface is
sufficient just because the types are public. Check the operations.

When `rcad-predicates` next opens, moving the divide down to `Interval` is a
relocation and nothing more.

### 16.5 The acceptance oracle could not be built first, and that is worth saying

§14.7 says build it first, not last, and §15.8 says it is per case, pass or fail,
never a rate. Both held. "First" did not, and could not have.

The oracle's subject is the *enclosure*, and the enclosure is the last thing M1
builds. What was built first is its other half: the exhaustive check that
`KnotVector::new` accepts exactly what an independent transcription of the six
rules accepts — every non-decreasing sequence over the whole numbers 0 to 7 for
degrees 1 to 3, **182,595 of them, enumerated rather than sampled**, zero wrongly
refused and zero wrongly accepted.

So §14.7's instruction resolves into two: the *acceptance half of every
constructor* goes first and can, and the *acceptance half of the cascade* goes
with the thing it judges. Only the second was what §14.7 had in mind, and only
the first is schedulable early. A milestone that reads §14.7 as one instruction
will either build it late or build the wrong one.

What it bought, when it did land: the quarter circle with weights `(1, 1, 2)` is
the unit circle exactly and reaches `(3/5, 4/5)` at `u = 1/2`, which is not an
`f64`. Handed to `Sphere::side_of` as an exact triple, that point is **certified
`Pos`** — off a circle it is exactly on. Handed over as a box it is
`Undecidable`. Neither assertion needs a tolerance, and the contrast is the whole
argument for the seam.

The acceptance half is asserted too, and it is the half §14.7 exists for: the
same points *are* certified against spheres of half and twice the radius, at both
clamped ends, on the extruded pipe and on a flat patch's plane. A timid filter
passes every other oracle on the list by refusing everything.

The quarter-sphere patch the plan called a stretch goal worked: the dyadic
weights do survive the revolution, giving `1,1,2,1,1,2,2,2,4`, and the octant
point at `u = v = ½` is `(9/25, 12/25, 4/5)`, whose squares sum to exactly one.

### 16.6 Rule 3 removes exactly the pieces rule 2 promised would fan

§13 rule 2 says a milestone is shaped gate, fan, gate, and named M1's algorithms
as the fan. §13 rule 3 says a part with no named consumer is deferred. Applied to
roadmap §4, rule 3 cut about half of it — and **it cut the fan.**

M1 did not fan wide. What survived is evaluation for curve and surface,
derivatives, the interval path, insertion, splitting and conics, and four of
those are the same body of de Boor code reached through different doors. The work
ran as a gate and one long sequenced line, with two small pieces at the end.

This is a general finding, not an M1 detail. Rule 3 removes the independent,
textbook-oracled pieces — degree elevation, the Oslo algorithm, curvature and the
fundamental forms, arc-length parametrization — precisely because their consumers
are in later milestones. Those are exactly the pieces that were going to fan.
**Rule 2 currently reads as though this will not happen. It will happen at every
milestone.** Expect gate-plus-line, and treat a genuine wide fan as the exception
that needs arguing for.

### 16.7 The departures this milestone made

1. **No enum arm, and the `Copy` cost with it.** `BSplineCurve` and
   `BSplineSurface` are concrete types. There is no `Curve` enum and no
   `Surface::BSpline`. The arm is the seam to topology, its consumer is M2, and
   it costs `Surface: Copy` (`surface.rs`), which `FaceView::surface` consumes —
   rippling into the crate M2.5 holds for the duration. Deferred to M2.
2. **Clamped and non-periodic only, until M5.** Invariants 3 and 4 enforce it.
   Every periodic surface mechanical CAD cares about is analytic and stays
   analytic (roadmap §4.1); periodic NURBS first arrive with a STEP file, and the
   refusal message says so.
3. **One always-rational type, which vacates one of §14.5's oracles.** Dividing
   by exactly `1.0` is exact, so the non-rational case costs nothing and there is
   no second variant. That makes "all weights equal reproduces the non-rational
   B-spline" a statement about nothing. A deliberate loss, recorded rather than
   quietly dropped.
4. **§4.1 beats roadmap §4.5 on panics.** `evaluate` has exactly one failure
   mode — a parameter outside the domain or not finite — and returns `Err` for
   it. A zero denominator during evaluation means a validated curve carried a bad
   weight, which is a broken invariant, and it panics. Roadmap §4.5 says never
   panic; §4.1's three tiers win, because a corrupt point is worse than a crash.
5. **`publish = false` past M1.** §3.6 says "until M1", which now reads as
   optimistic rather than as a forgotten TODO. Nothing here is shaped for a
   stranger yet: `MAX_DEGREE`, the two roundoff constants and the conic
   constructors are all still finding their consumers.
6. **Rule 3 shrinks the fan rule 2 promises** — §16.6.

### 16.8 Known gaps carried into M2 and M3

- **Surface refinement does not exist.** Insertion, splitting and Bézier
  extraction are curves only. Nothing in M1 consumes the surface versions, and
  rule 3 says wait for the consumer. Roadmap §6.3's surface–surface marching is
  it.
- **General-angle arcs are not exact and were not built.** `cos θ` is an `f64`
  for almost no angle, so a general arc constructor cannot be exact, and an
  inexact one had no consumer this milestone. `circle` and `quarter_arc` are what
  shipped.
- **Conic exactness stops in two places, and the tests claim it only where it
  holds.** The in-plane frame is a cross product and a normalize, exact only when
  the axis is a coordinate direction; and a control point is `centre ± radius`,
  which rounds away from the origin. Both are documented at the constructor.
- **An enclosure costs 12× the `f64` evaluation for a degree-1 curve and 83× for
  a degree-5 surface** (332 ns against 27.7 µs). No consumer has asked for speed,
  so nothing was optimised. The first saving available is that the local interval
  divide computes four corner quotients where both operands are usually single
  numbers; it was left alone deliberately, because it is a correctness-critical
  divide and M3 is the first caller that will care.
- **The published `f64` bound is much wider than the enclosure it is tested
  against.** Across every family the enclosure's half-width never passed 0.145 of
  the bound, so the bound is at worst about seven times looser and usually far
  more. That is the honest behaviour of a bound stated over the homogeneous net
  when the net can be five hundred times the affine one, and it is another reason
  the bound is not a second enclosure source. It also means the ratio makes a
  usable scale-free ceiling where a count of `f64` steps does not — §16.1.

### 16.9 Review found what the suite did not, again

*Added after the milestone's adversarial review, which ran before the merge.*

§12.1 recorded that M0 shipped three bugs through a green 335-test suite and
that every one was found by review. **M1 did the same thing.** The suite was 549
green and `just` passed when the review started. It found seven defects, two of
them serious, and this section exists so the pattern is recorded twice rather
than rediscovered a third time.

**A certified wrong answer.** A de Boor coefficient's denominator overflows when
the knot range passes `f64::MAX`, every share collapses to zero, and evaluation
returns control point 0. On a degree-3 curve spanning `-9e307` to `9e307` that is
`0.0` where the truth is `1.5` — and `evaluation_roundoff_at` certified it with a
bound of `4.5e-14`.

This is **§12.2 in its third location**, and the shape is now unmistakable.
`EVALUATION_ROUNDOFF` is relative throughout, so it holds only while its
hypothesis does, and the hypothesis was enforced on the control coordinates and
on the weight but not on the knots. §12.2's own words were that we already knew
and guarded one of the three places the hazard appears; M1's plan called itself
the second; this was the third, in the same milestone, in a bound written by
someone who had read §12.2. **Enumerate every input a relative bound depends on,
and guard each one, or the count will keep going up.** The fix refuses such a
knot vector at construction, and the bound's doc comment now numbers all three
places rather than two.

The interval path was sound throughout and returned a box that does contain the
true point. That is the seam working exactly as designed, and it is the strongest
evidence in the milestone that making the enclosure the only enclosure — never
intersecting it with the `f64` bound — was the right call. Had they been
intersected, this bug would have produced a confidently wrong box.

**A silent NaN.** `tangent` returned `NaN` where the true derivative was exactly
`1e8`, because the derived net scaled before dividing. §4.1 ranks a silent
corrupt value below a crash, and this was one.

**Amended, after the fix was questioned.** Dividing before scaling left a
narrower case still refused — control coordinates more than the whole `f64`
range apart, whose true tangent may be an ordinary number — and it was written
down as a limit that *no ordering of the arithmetic could fix*. That was true,
and it was the wrong frame. Reordering cannot help, because every route to a
derived point needs the same subtraction; **scaling can**, and one halving
always suffices, since two coordinates each at most `f64::MAX` have halves at
most `f64::MAX` apart. The factor of two returns in the final scale. A tangent
of exactly `5e307` that had been refused now comes back exact.

The general lesson is worth more than the case: **a proof that no member of a
family of fixes works is not a proof that the thing cannot be fixed.** The
correct reading of "no ordering helps" was that ordering was the wrong family to
be searching. The gap survived a review that had already found two real bugs in
the same function, because it arrived carrying its own impossibility argument —
and an argument for why something cannot be done is exactly the kind of claim a
reviewer is least likely to re-derive.

**And once more, one layer down.** The same amendment left a second case
written down as a limit: `rational_derivative` forms `(A' − w'·C) / w`, and
`w'·C` can leave the range while the quotient would fit. Reaching it needs a net
built near the ends of `f64` on purpose, so it was recorded rather than fixed.
That was the same misjudgement a second time — "no consumer will hit this" is a
statement about today's callers, while a representable answer being refused is a
statement about the code. It is fixed the same way: when the product overflows,
`w` is by construction large enough to have brought it back, so the expression is
formed again dividing first, where `A'/w` and `w'/w` are both *smaller* than the
terms that overflowed. The two spellings are equal in exact arithmetic and fail
on opposite inputs, so the code tries rather than chooses — choosing would mean a
tuned threshold on `w` standing where a plain question about the answer does.

A curve whose true tangent is `-4.76e300` was being refused and now comes back,
matching exact rational arithmetic to every digit `f64` carries.

**What separates the two cases is worth stating, because it is the rule.** A
refusal is a *limit* when the answer does not exist in `f64` and a *gap* when it
does and the arithmetic could not reach it. A knot span one `f64` step wide under
ordinary coordinates has a true derivative past `f64::MAX` — that is a limit, and
`KnotVector::new` accepts such a span on purpose, so §4.1 makes it an `Err` about
the numbers asked for. Everything else was a gap. Neither fix claims totality and
the doc comments say so; what they claim is that the set of inputs answered is
strictly larger, and that the remainder refuses rather than handing back a `NaN`.

Both gaps were found by being asked about, not by the review and not by the
suite — which is the third time in this milestone that the thing carrying an
explanation of why it was fine turned out not to be.

**Three oracles that could not fail.** The knot-insertion oracle — §14.3's *first*
exactly-true relation — asserted nothing on about half of every family's draws,
because the helper choosing untouched control points returns an empty list on
short curves and only one of four tests checked. The split oracle's "two routes
to one answer" was one route: the by-hand raising loop was character for
character `split_at`'s own. Both are §14.1 question 2 answered "yes" by tests
that could not have said "no".

**And the cause of all of it: one generator family.** Every strategy drew its
knots from the same six small whole numbers, and the scaling oracle cloned the
knot vector unchanged while scaling only the control net — so the de Boor
coefficients, which are the dangerous code, received bit-identical inputs in both
runs of the only metamorphic test aimed at them. **No test in the milestone ever
varied a knot value.** That is why a bug in the coefficient denominator survived
549 tests.

§12.3 already said one family of shapes finds one family of bugs. What M1 adds is
that a generator can be varied in the dimension you are thinking about and
constant in the one that matters: the nets were random, wide-ranging and
carefully conditioned, and the knots were `0, 1, 2, 3` every single time. **Ask
of each generator not "is this varied?" but "which argument of the dangerous
function does this never change?"**

The repair is measurable. Knots now span 48 orders of magnitude, and scaling the
*knot vector* by `2^k` is a second oracle down a genuinely different path — the
point may not move one bit, the tangent must come back divided by exactly `2^k`.
Mutation confirms it: a de Boor nudge that only bites above `1e6` passes the
entire suite as it stood at review time, and fails all four of the new scaling
tests.

**What this says about §13 rule 6.** Rule 6 caps work in flight because review is
the constraint, not authoring. Both milestones now have the same evidence, and it
is stronger than the rule states: review did not merely find *more* than the
suite, it found things the suite was structurally incapable of finding. Budget it
as a phase with its own findings, not as a pass over finished work.

### 16.10 A claim that justifies an absence is not reviewable, and needs its own check

*Added after §16.9's second amendment, because the same thing happened twice and
the second time it was noticed only by being asked about.*

Two of M1's defects were found by neither the suite nor the adversarial review.
They were found by a reader asking what a sentence meant. Both sentences were
doing the same job, and it is a job no reviewer was looking at.

**The distinction that matters.** A code review examines **what is there**. Both
of these were arguments for **what is not there**:

- *"no ordering of the arithmetic gets that back"* — an argument that a fix does
  not exist.
- *"reachable only by handing `BSplineCurve::new` a net near the ends of the
  `f64` range on purpose"* — an argument that a fix is not needed.

An absence has no lines in a diff. A reviewer hunting defects reads the code that
exists, finds it consistent with the doc comment beside it, and moves on — which
is the correct thing to do with code. The review that ran on this milestone did
check both `assert!` unreachability arguments in `de_boor.rs` and found them
sound, because those are claims about **code that is there**. It walked past the
two claims about code that is not, and so would any reviewer, because there was
nothing to look at.

Worse, the reasoning attached reads as *evidence of care*. A bare `TODO` invites
scrutiny. A paragraph explaining why something cannot be done deflects it, and
deflects it in proportion to how well written it is. Both of these were well
written. Both were wrong.

**This generalizes §15.6, which the project already believes.** §15.6 says a
published bound needs **a test that could fail, not a paragraph** — prose
standing in for evidence, caught for one kind of claim. It is the same failure
here, in two more:

> **A claim that something cannot be done, or cannot be reached, needs a
> demonstration and not a paragraph.**

For an impossibility claim the demonstration is an enumeration: name the family
of fixes considered and say why that family is the only one. "No ordering helps"
was true of orderings and silent about scalings, and the silence is where the
bug lived. For an unreachability claim the demonstration is a reachability
argument through the **public API**, not through today's callers.

**Three species, and the question each needs answered:**

| Species | Sounds like | The question |
|---|---|---|
| Impossibility | "no ordering helps", "there is no way to reach it" | Is the named family of fixes the only family? |
| Unreachability | "nothing in the kernel builds that", "only on purpose" | Is it reachable through the public API at all? |
| Sufficiency | "one guard is enough", "by construction", "covers every case" | Enumerate what it ranges over; which member is uncovered? |

The third is not hypothetical either — it is the milestone's most serious bug, a
relative bound whose precondition was enforced in two of the three places it was
needed.

**The uncomfortable part: §13 rule 3 is also the camouflage.** "A part with no
named consumer is not built" is a real rule, it is in `CLAUDE.md`, and it is
right. It is also exactly the sentence the second defect hid behind. The
refinement, which §16.9 already names: **rule 3 governs features, not correctness
gaps.** Deferring a part nobody needs is rule 3 working. Deferring a case where a
representable answer is refused is rule 3 being used as cover, because the
question rule 3 asks — *who consumes this?* — is the wrong question to ask about
a defect. The test is §16.9's: a refusal is a **limit** when the answer does not
exist, and a **gap** when it does and the arithmetic could not reach it. Rule 3
may defer a missing feature. It may not defer a gap.

**Operationally**, and the half of this that lives in `CLAUDE.md`: the markers are
few and grepping for them is cheap — `cannot`, `never`, `always`, `no ordering`,
`nothing`, `only`, `impossible`, `unreachable`, `by construction`, `suffices`,
`covers`, `reachable only`. A whole crate holds a dozen or two. Each is a work
item with a definite question, and the answer is a demonstration or a rewritten
sentence. `.claude/skills/claim-audit` is the pass, and it is deliberately **not**
a code review: pointing a reviewer at defects finds defects, and these are not
defects until someone asks what the sentence rules out.

### 16.11 The dynamic-range check is a ratio, and cannot see absolute magnitude

*Found by the §16.10 claim audit. The question it opened is answered in §18;
what stays here is what the audit actually established.*

`DynamicRange::is_within_supported_range` is `self.ratio <= LARGEST_SUPPORTED_RATIO`
and nothing else. So the check answers *"is this model's smallest feature
resolvable against its largest?"* — the question §3.5 set out to ask, and a good
one.

It is not the question three doc comments were citing it for. `build.rs` and
`validate.rs` both said `Undecidable` on exact `f64` input happens only when a
product overflows or falls subnormal, *"which is what the dynamic-range check
exists to catch"*. It cannot catch that, because the models where it happens are
**uniformly** tiny or **uniformly** enormous, and those have a ratio of exactly
one. Measured: the `1e-170` square that used to panic the profile builder
reports `ratio = 1, accepted = true`. Two further copies of the same sentence
were later found in `orient2d.rs` and `orient3d.rs`, which the freeze had put
out of reach of the first correction — see §18.

**Correction, and it is this section's own mistake.** As first written, the
paragraph above said `largest_coordinate_magnitude` is "computed, stored, and
never consulted". That is false. It has no accessor, but `coordinate_spacing()`
reads it inside the same module to build the `Warn`-severity slop tag. It is
neither dead nor a §13 rule 3 violation, and the sentence asserting otherwise
was written in the same session as §16.10 and never checked. Recorded rather
than quietly edited, because a section about unchecked claims making an
unchecked claim is the most useful example it could have.

**The design question this opened — should the kernel refuse a model whose
coordinates are absolutely too small or too large, independently of their
ratio? — is answered "no" in §18**, and answered by removing the reason to ask
it rather than by weighing the trade. Uniformly small or large coordinates turn
out to be a §16.9 *gap*: the sign exists and an exact power-of-two rescale
reaches it. A predicate that can decide them has nothing to refuse.

**The inversion worth carrying forward.** The ratio check catches the genuinely
wide-spread model, which is a real *limit* — no single scaling brings the
differences and the raw coordinates into range together. It is blind to the
uniform case, which is not a limit at all. So the check was right about what it
caught, and every sentence citing it was reasoning about the other half.

**The large end is a different bug, and §18 does not fix it.** Measured:
`orient2d` and `orient3d` never fail on the large end at all, out to `1e200`.
The `1e140` cube's `classify_point` failure comes from the ray-cast target,
derived from the model's bounding box, overflowing — not from predicate degree.
It lives in `rcad-topo`, it has its own cause and its own oracle, and §13 rule 6
says it is a separate piece of work.

Recorded with a hypothesis attached, so the next pass starts somewhere: it is
probably a gap too, and a ray target chosen relative to the bounding box rather
than in absolute coordinates would likely just work.

**Both of those paragraphs are wrong, and §20.7 has the measurements.** The
`1e140` cube classifies `Inside` with an empty log and has since `a23d73d`, so
there is no large-end `classify_point` failure left to explain. The cause named
above was never the cause: the ray target does not overflow until side
`7.7e307`, and at `1e140` what overflowed was inside `orient3d`. The hypothesis
attached for the next pass therefore proposed a fix for a problem that did not
exist — a reminder that a cause recorded without a demonstration is a guess
wearing a measurement's clothes, which is the whole of §16.10.

One thing that made this urgent has also gone quiet in a way worth noting.
`Model::validate` does refuse the `1e140` cube — but by an unrelated
`VolumeIsNotPositive { volume: inf }`, not by its dynamic-range step, which
passes with a ratio of one. The protection was always accidental. **§20.4 acted
on that sentence:** an infinite volume is no longer reported as an invalid
model, because a `1e300` cube is not broken, and `Invalid::VolumeIsNotPositive`
now means only that the faces point inwards.

## 17. The planar fuse

*Recorded 2026-08-30, before M2.5 starts.*

§15.9's last table hands M1 §16 and M2.5 §17 precisely so the two milestones
can append to this file at the same time without colliding. They did, and the
reservation did its job: the two sections were written in separate worktrees
that never saw each other, and putting them in the same file cost one ordering
decision and nothing else.

**Amended 2026-08-31.** This section was drafted before M2.5 started and says
so below; it was then amended in place as the work landed, and §17.7 and §17.8
were written afterwards from measurements. Where a subsection reads as a plan
and a later amendment corrects it, the amendment is what shipped.

§1.7 inserts M2.5 and lists its eight exit criteria, §14.8 adds three more
oracles, and §15 settles the meanings M1 and M2.5 would otherwise settle twice.
This section is what a planning pass over all of that concluded. Everything
here is settled; the fuse gets built to it.

### 17.1 A face *is* its plane

A face carries a `Surface::Plane` and a ring of corner vertices, and those are
two different geometries that agree only to roundoff.
`Plane::from_point_and_normal` normalizes, so the normal of a side face of an
extruded polygon is irrational in general and what we store is the nearest
`f64` triple to it; `Model::validate` correspondingly asks only that a corner
sit within `allowed_wobble` of its own face's plane, not that it sit on it. A
fuse has to pick one of the two as the truth, because it is going to emit
coordinates that are not in the input and it has to be able to say what they
are coordinates *of*.

**Settled: `Surface::Plane` is authoritative, and the stored corners are cached
samples of it.** At fuse entry every input vertex is re-derived as the meet of
the planes of the faces around it — a constructed point with its enclosure, not
an `f64` triple read out of the arena. A vertex where four or more faces meet
is over-determined: we derive it from three well-conditioned planes and then
check that each remaining plane agrees with the result inside its enclosure,
refusing when one does not. That check is worth having on its own, because a
solid whose fourth plane misses the corner by more than the corner's own box is
not the solid the user thinks they built.

The deciding argument is drift across chained fuses, and it only shows up in
the third or fourth boolean. A split face inherits its parent's plane bit for
bit, so however long the chain the planes in the model stay the planes the user
built, undegraded. Vertices are re-derived from those planes on every fuse, so
the rounding in a vertex is the rounding of one Cramer solve — never a sum over
the operation history.

Rejected: vertex-authoritative geometry, with each face's plane fitted to its
corners. It is the obvious choice, since the corners are what the user typed
and what gets serialized, and it is a slow leak: every fuse re-rounds every
vertex, every re-rounded vertex moves the next fitted plane, and the error
compounds along the chain. It is also the version that passes the tests you
would think to write first — one fuse against a hand-checked answer shows
nothing at all. §14.8's associativity oracle,
`fuse([A,B,C]) ≡ fuse([fuse([A,B]),C])`, is exactly what catches it, by
failing, some weeks after the choice has become load-bearing.

One of §1.7's own guards then falls out instead of having to be arranged. It
asks that "the path where a split vertex is known only as an interval" be live
in the planar world, and worries that a generator will have to be coaxed into
producing it. With the plane authoritative that path is live for *every*
vertex, including the ones that came straight from the user's profile, so it is
the ordinary case and not a case anyone has to remember to test.

### 17.2 Two kinds of wobble, and only one of them refuses

We have two quantities, both lengths in the model's own unit per §15.2, and
they were one careless sentence away from sharing a name.

| | Where it comes from | Where it lives | Judged by |
|---|---|---|---|
| Unproved-decision wobble | a predicate could not decide and a tolerant policy accepted the enclosure anyway | written by `resolve_under_policy` into the `ProvenanceLog`, folded by `ProvenanceLog::worst_wobble(working_size)` | exit criterion 7, against `Policy::max_wobble()` |
| Representation wobble | the distance from a true meet to the nearest `f64` — present even when every decision was *proved* | tagged on the entity as an `Approximation`, visible in `Model::worst_approximation` | nothing. It is reported, never refused on |

**Settled: exit criterion 7 is a statement about unproved-decision wobble
alone.**

`Policy::certified().max_wobble()` is exactly `0.0`, and that is what makes the
distinction load-bearing rather than pedantic. Reading criterion 7 as covering
both quantities would make a certified fuse refuse every input whose corners
are not exactly representable — which is every part that is not axis-aligned on
a lattice, so very nearly every part — and exit criterion 8 would then be
reporting how well `f64` represents a rotated cube rather than how often our
filters give up. That is §14.7's warning one milestone on, and §15.5 is the same
warning about projection. The log makes the split easy to keep hold of: §5
emits detailed records only on non-certified outcomes, so a `ProvenanceLog`
never contains a proved decision at all, and `worst_wobble` therefore already
computes the first quantity and nothing else. Criterion 7 is self-consistent as
a statement about it alone.

Rejected: wiring criterion 7 to `Model::worst_approximation()` instead of
`ProvenanceLog::worst_wobble()`. This is not a straw man — it is the shorter
line of code, it reads as the more thorough choice, both are an `f64` in the
same unit since §15.2, and the wrong one compiles. Naming it here is most of
the guard; the test that a certified fuse over an already-tagged input succeeds
is the rest.

That last is the same answer one level up, and it needs saying because it looks
like a contradiction: **a certified fuse over an already-tagged input succeeds,
and carries the input's tag through to its outputs.** A tag on an imported
vertex is a statement about how that vertex got into the model, not about
anything this operation did, and a certified policy is a promise about this
operation's decisions. Refusing on an inherited tag would make
`Policy::certified()` useless on exactly the models roadmap §9.1's STEP
importer will produce, since an exporter's rounding arrives as
`Cause::ImportedUncertainty` on the first vertex.

Representation wobble gets its own cause, additively, in `rcad-predicates`:

```rust
pub enum Cause {
    UndecidableResolved,
    TolerantFallback,
    ConstructedPoint,
    ImportedUncertainty,
}
```

Rejected: reusing `TolerantFallback` and saving the variant. Representation
wobble happens under a *certified* policy, where by definition no fallback
took place, so the tag would claim a tolerant policy accepted something about a
decision that was proved exactly. The causes exist to be diagnostic, and that
reuse blinds them precisely where the diagnosis matters — a model full of
`TolerantFallback` tags that a certified run produced.

### 17.3 Placement is baked in at fuse entry

§3.2 composes locations from day one, so the same face can appear at two
placements and a `FaceRef` carries the `Isometry` that says which. The fuse
cannot decide anything about two solids until their coordinates are in one
frame, so one of them has to move.

**Settled: the fuse bakes placement into world coordinates at entry, and the
transform's rounding is representation wobble, tagged as such.** Rotating a
point is multiplications and additions in `f64`, so an exact vertex under a
general rotation lands near, not at, its true image. Nobody failed to prove
anything there; it is §17.2's second kind, and it does not refuse. §3.2 already
offers power-of-two translation snapping as a named constructor for callers who
want the rounding to be zero, and declines to enforce it, which is the same
judgement made earlier.

This is a debt being paid rather than a detail being noted. `classify.rs`
already hands rotation-rounded coordinates to `orient3d` wrapped as
`Point::Exact` and tags nothing. The wrapper is not lying about the arithmetic
that follows — it really is exact on those coordinates — but the coordinates
handed in have already lost something and nothing in the model says so. §12.5
records the visible half of the same gap, that classification's enclosure and
provenance fields "carry no signal today ... and cannot until classification
meets constructed points". M2.5 is where they meet, so M2.5 is where the
tagging becomes honest.

### 17.4 The rest, settled

The signature first, since several rows below are about what it means:

```rust
pub fn fuse(
    &mut self,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Fused>
```

`A` is the union of the arguments and `B` the union of the tools, per roadmap
§7.2's insistence that both sides are sets from the start. `Rule::Union` is
`A ∪ B`, `Rule::Intersection` is `A ∩ B`, `Rule::Difference` is `A \ B`. An
empty result is a success, not an error — two solids that do not touch have an
empty intersection and the caller asked a well-formed question.

| Question | Settled | Why |
|---|---|---|
| A non-manifold result | Refuse with `Error::ResultIsNotManifold` | `check_touched_edge` asserts exactly two coedges on an edge; relaxing that changes what a `Model` *is*, everywhere, and that is not work a boolean gets to do on its way past |
| Atomicity | Build the whole result in scratch and commit once. Any `Err` leaves the model byte-identical | A half-applied boolean is the corrupt solid §4.1 would rather crash than emit, and here we can have neither |
| `Rule` | `#[non_exhaustive]` | Imprint and symmetric difference are the obvious next two, so it is an enum that grows — §3.6's rule, and the contrast with §3.7's deliberately exhaustive `Orientation` is the whole test: there is no fourth way round a face, there are plenty more selection rules |
| History maps | One image map per entity kind. Generated, modified and deleted are **derived** from it, not stored | Three stored lists can disagree with each other and with the model; one map cannot. "Deleted" means an input key with no image, so input keys stay valid for asking |
| BVH broad phase | Out | Roadmap §7.1 opens with it, but a broad phase changes no answer — it only skips pairs that would have produced nothing. §1.7's criteria are about answers, M2.5 has no performance target to beat, and it goes to M4 with the bump arena (§11) |
| ON classification | `Plane::agreement_with`: exact proportionality of the two planes' coefficient 4-vectors, six 2×2 minors of degree 2, each decided through the cascade, and it reports which way each plane faces | A deliberate improvement on roadmap §7.3's "normal comparison": comparing normals needs a tolerance, which is the model §1.2 declines to build, and it also cannot see the offset — two parallel planes a millimetre apart have identical normals and are not the same plane. Same way against opposite way is what the ON columns of the selection table need |
| Self-intersecting input | Falls out of the decomposition as `Error::ArgumentIntersectsItself`, and is deliberately **not** added to `Model::validate` | The decomposition already computes every face-pair meet, so a meet between two faces of the same solid that is not along their shared edge is free to notice. Putting it in `validate` would make validation quadratic in face count, which §4.3 spent a section refusing |
| Reassembly | Group surviving pieces into shells by edge adjacency, nest shells into solids with `classify_point`, orient by signed volume — outer positive, voids negative | The manifold check of the first row lands in the adjacency walk, where an edge with three claimants is discovered rather than searched for. An undecidable sign on the volume refuses: an inside-out solid is worse than no solid |
| Validating the output | Full `Model::validate()` on the committed result, in debug builds | §4.3 puts full sweeps at test and import boundaries and local checks after mutations, and a fuse is not really a mutation — it invents topology that was in no input, which puts it nearer an import |

**Selection is one table read as data, and one `match` on `Rule` that picks the
row.** §1.7's first criterion says one decomposition and three selection rules,
not three algorithms, and three `match` arms each walking the pieces their own
way is three algorithms wearing one signature.

| `Rule` | Argument piece IN `B` | Argument piece OUT of `B` | Tool piece IN `A` | Tool piece OUT of `A` | ON, same direction | ON, opposite |
|---|---|---|---|---|---|---|
| `Union` | drop | keep | drop | keep | keep once | drop |
| `Intersection` | keep | drop | keep | drop | keep once | drop |
| `Difference` | drop | keep | keep, reversed | drop | drop | keep once |

**Splitting is one arrangement per face against all of that face's cut lines at
once, never a sequence of pairwise splits.** Splitting by one line, then
splitting the pieces by the next, gives output that depends on the order the
lines arrived in, and the order the lines arrived in depends on the order of the
arguments. §14.8's commutativity oracle and its associativity oracle would then
fail by construction — not because of a bug we could find and fix, but because
the algorithm genuinely computes a different partition for `fuse(A, B)` than for
`fuse(B, A)`. Rejected: pairwise-sequential splitting, which is easier to write
and easier to debug one line at a time, and which cannot be made to satisfy the
two oracles we most want.

A cut line is always a *pair of planes*, and never a point plus a direction
(§15.9). Two faces whose planes are not parallel contribute the pair
`{plane(A), plane(B)}`. Two coplanar faces contribute one line per edge of `B`,
each as `{plane(A), plane(B')}` where `B'` is the face on the other side of that
edge — the boundary of `B` is what cuts `A`, and each boundary edge already lies
on two planes. The difference between the two cases lives entirely in the
gathering step, so the arrangement itself sees one uniform kind of input and has
no coplanarity special case in it.

Ordering points along a cut line is an interval comparison on the line
direction's dominant axis, and needs no new predicate — which matters, because
§15.9 freezes `rcad-predicates` and a new predicate would reopen that gate and
stop both fans. Two points whose intervals on that axis overlap are
undecidable, and that is not a gap: an overlap is exactly the coincidence case,
and the coincidence case is the one that gets merged rather than ordered.

Degenerate pieces are prevented, not filtered out afterwards. A decided
coincidence merges its two points into one rather than creating an edge of no
length, so the zero-length edge never exists to be swept up. A zero-area face
surviving to the commit is therefore a broken internal invariant and **panics**,
per §4.1 — filtering it would be hiding the evidence that the splitter is
wrong. Slivers that are certifiably non-zero are **kept**: §12.3 is what happens
when a valid sliver gets refused, and it took an exhaustive search over 160,179
rings to find out.

Finally, determinism. **No `HashMap` or `HashSet` anywhere in the fuse's working
state** — `BTreeMap` and `BTreeSet` keyed by slot, or a `Vec` with an explicit
sort. Iteration order over a hashed container varies per process, that order
decides the order pieces are inserted into the arena, and arena order is
visible in every serialized byte. §9.2's bit-identical criterion, §14.8's
oracles and criterion 8's reruns all compare one run against another, and every
one of them would start failing intermittently. The test is cheap and nothing
else in the suite would notice: run the same fuse twice in one process and
assert the serialized results are equal.

**Amended 2026-08-30, while implementing.** Splitting all-at-once is not on its
own enough to make the answer a function of the two *sets*, and the metamorphic
suite found the hole on its first run. Two coplanar faces that overlap are cut
by *different* sets of lines — a face never sees the lines that only its own
solid's corners put on the other one — so the region they share comes out
partitioned two different ways, and "keep once" keeps whichever came first.
Swapping the two sides of the fuse therefore changed the answer: two ells flush
against each other united into twenty-three faces one way round and twenty-six
the other, the same solid both times. **Settled: at the commit, two faces of the
result that lie on one plane facing one way with an edge between them are joined
into one face, and any corner that stops being a corner when they are — a point
with two faces round it, sitting in the middle of a straight edge — is dropped,
its two edges becoming one.** `Plane::agreement_with` decides "one plane facing
one way" exactly, the rings that come out are sorted into an outline and holes by
the same swept-area sign `Model::validate` judges a face by, and the joined face
takes the smallest of its pieces' planes read as four numbers, so nothing about
the answer depends on the order the pieces arrived in. The two ells now come out
as the eight faces the shape has, whichever way round they are asked for.

This is not only about commutativity, and that is what makes it the fix rather
than a tidy-up. §17.1 re-derives every corner as the meet of three planes at the
next fuse's entry, and a corner with two coplanar faces on each side of it has
**two** planes round it, not three — Cramer's determinant is exactly zero and
there is no other triple to try. So the unjoined answer was a solid this kernel
could write and then could not read: uniting two boxes that overlap along one
axis and uniting the result with a third refused outright. Joining also drops
the cuts a plane leaves where the solid it belongs to is nowhere near, which is
what a cut by a *plane* costs and what §17.4's splitter had no way to undo: a
cube in the middle of another cube united into fifty-four faces, and now unites
into six.

Rejected: letting `bake` tolerate a coplanar neighbour by solving the corner from
some other triple of planes. There is no other triple — the planes round such a
corner are two planes, however many faces carry them — so the only version of
that fix is to stop re-deriving the corner and read its stored coordinates
instead, which gives up §17.1 for exactly the corners a chain of fuses passes
through most often. It also leaves the partition order-dependent, so §14.8's
commutativity oracle would still fail.

One consequence for the error tiers, settled here because it is the same bug
seen from the other end. `Plane::meet_of_three` divides by the determinant of
three normals and has only an interval filter to bound it — there is no exact
level to fall back on, because the answer is a point and not a sign. A
determinant it cannot tell from zero is therefore a decision the cascade did not
settle, and the operation is left with nothing to return: that is
`Error::Undecidable` with a record naming the predicate, not
`Error::Geometry(PlanesDoNotMeetInAPoint)`. The old tier put a refusal somewhere
§17.7's breakdown of what gave up could not see it.

**Amended 2026-08-31.** The ON-classification row above says
`Plane::agreement_with` decides exact proportionality through the cascade, which
reads as though it always answers. It does not, and `PlaneAgreement` has a
fourth variant, `CannotTell`. Each of the six minors is one `orient2d`, and
`orient2d` can come back `Undecidable` when a product underflows out of the
range where `f64` multiplication can be undone — a normal tilted by roughly
`1e-300` reaches it. Answering `Different` there would be a confident lie about
a plane and its own reverse, which is what the test that found it asks. §3.5's
dynamic-range check is what keeps models like that out; a caller who has one
anyway gets told that nothing here can tell. This is the ordinary shape of the
rule in `CLAUDE.md`: a predicate returns a `Decision` and `Undecidable` is one
of the answers, so anything built out of predicates inherits the case.

**Amended 2026-08-31, and the amendment of 2026-08-30 was half of it.** That
paragraph joins two coplanar faces of the *answer*, and it is written as though
the answer were the only place such a pair could come from. It is not. A
`ProfileBuilder` outline may have three points in a row — nothing refuses one,
and nothing should, since `[0,0], [1,0], [2,0]` is a perfectly good way to
describe one wall of a part — and the extrusion of such a profile has two side
faces on one plane with an edge between them. §17.1 then cannot re-derive the
corner on that edge: the planes round it are two planes and not three, however
many faces carry them, Cramer's determinant is exactly zero, and the fuse refuses
with a record naming `WHERE_THREE_PLANES_MEET`. It refused whatever the other
solid was and whatever the rule was, including for a pair five units apart that
never touched, because the refusal is about one input's own corners and not about
the pair at all.

**Settled: the same joining runs on the way in.** `bake` builds an `Assembly` out
of each input solid, calls the same `join_up_what_is_one_face` the commit calls,
and re-derives the corners of what comes back — so §17.1 still holds and every
corner the arrangement sees still has three planes round it. It is the one
joiner and not a second one that has to agree with the first, which is what
§1.7's third criterion means by *reused, not reimplemented*, and two joiners is a
seam of exactly the kind §13's preamble is about. A joined input face keeps
naming every input face it was made of, and a joined input edge every input edge,
so nothing that survived comes back through the history as deleted; that is why
`BakedFace`, `Piece` and `FaceMeet` now name a list of ancestors rather than one.

Nothing already decided moves, and the shape of the fix is what says so rather
than a hopeful test. The joining is skipped whole for an input with no two
coplanar faces sharing an edge — which is every input the fuse itself built, and
every one the catalogue and the three generated families produce — and an input
that does have such a pair refused outright before, so there is no answer to
change. §17.7's three measured rows came back with the same counts to the fuse,
and every bit-identical property still passes.

Rejected: refusing three profile points in a row in `ProfileBuilder`, or refusing
two coplanar faces sharing an edge in `Model::validate`. Both narrow what counts
as a valid model to suit one operation, and that is a larger decision than this
bug is entitled to make. What a `Model` is has to be settled for the kernel, not
for whichever operation found the shape inconvenient first.

### 17.5 What this milestone owes to earlier sections

Four corrections, all amended in place where they belong so a reader of that
section is not left with the old claim, and listed here so a reader of this one
knows the ledger is settled.

§15.7 is half wrong: its `Approximation` is genuinely zero for planes and stays
so, but the endpoint enclosure is not zero and never will be, because Cramer
divides. §15.9's predicted two-plane meet in `rcad-geom` is a deliberate
non-addition, since the plane pair is already the line and constructing a point
on it would round. §11's bump-arena row moves back out of M2.5 to M4, where a
consumer for it exists. And §12.6's first gap closes: `Policy::max_wobble` and
`wobble_is_acceptable` get their caller, which is the fuse, judging
`ProvenanceLog::worst_wobble` under exit criterion 7 (§17.2) — a field
specified in §2.6, implemented in M0 and called by nothing since.

### 17.6 The work breakdown and its oracles

§13 rule 1 makes the oracle list and the work breakdown the same document: a
piece can be worked on independently exactly when the property that decides it
is correct can be written down before the work starts. §13 rule 2 shapes a
milestone gate, fan, gate. So this is both.

M2.5 runs **alone** until the opening gate has landed, because the gate changes
`rcad-predicates`, which §15.9 freezes for the parallel window. Opening M1's
worktree afterwards costs nothing.

| Piece | Shape | What decides it is correct |
|---|---|---|
| 1. `Cause::ConstructedPoint` | opening gate | Additive to a `#[non_exhaustive]` enum; a certified fuse over a rotated input tags its vertices and names this cause (§17.2) |
| 2. `Plane::agreement_with` in `rcad-geom` | opening gate | Exactly true: proportional 4-vectors agree, non-proportional ones do not, and the cascade decides both. M2.5's only point of contact with the crate M1 owns, closed on day one rather than mid-window |
| 3. This section | opening gate | — |
| 4. Scratch types in `crates/rcad-topo/src/fuse/` | opening gate | The shapes below, plus the commit-once structure §17.4 requires |
| 5. The splitter | core, one head | Exactly true and needing no bound (§14.1 question 1): the areas of a face's pieces sum to the parent's area, and every piece's boundary closes |
| 6. Classification and selection | core, one head | Roadmap §11.2's decomposition-invariant list against dense point sampling |
| 7. The catalogue, nine planar entries | alongside, test-only | Each named case passes or refuses with a recorded reason (§1.7 criterion 5) |
| 8. The metamorphic suite | alongside, test-only | Roadmap §11.2's properties plus §14.8's three, and bit-exact invariance under *exactly representable* isometries |
| 9. Shell reassembly | closing gate | Euler characteristic and shell counts on the assembled result; the manifold refusal of §17.4 |
| 10. Bounded transform invariance under general rotations | closing gate | The bound, published beside the expression it bounds (§15.6) |
| 11. Measure and record criterion 8 | closing gate | §17.7 |

Pieces 5 and 6 stay in one head because §13's own table files "boolean
classification and shell assembly — graph traversal" under *does not split*, and
rule 6 caps work in flight inside a single crate. Pieces 7 and 8 run alongside
because they touch no shared code at all.

The scratch types being decided are these, and they are §15.7's intersection
result type made concrete:

```rust
struct FaceMeet {
    carried_by: [Plane; 2],
    ends: [MeetPoint; 2],
    approximation: Approximation,
}

enum MeetPoint {
    Existing(VertexKey),
    OfThreePlanes([Plane; 3]),
}
```

`carried_by` is §17.4's pair-of-planes cut line, and `MeetPoint` is why no
two-plane meet is needed: an end is either a vertex that already exists or three
planes that have not been solved yet, and solving is `Plane::meet_of_three`.

Piece 6 generalizes `SolidView::classify_point` to accept a `PredicatePoint`,
keeping today's `Point` entry as a thin wrapper over it. §1.7's third criterion
says classification is *reused, not reimplemented*, and this is the strongest
reading available of that word: one body, two entry points, rather than a second
ray caster that agrees with the first until it does not.

**The schedule risk is piece 6, not piece 7.** §1.7 names the catalogue, and
adds in the same breath that §12.1's lesson is that cost lands on the part that
looks done — which here is classification. Both halves are in §1.7; this
sharpens which one to believe. Point classification is shipped, benchmarked,
and about to be handed constructed points for the first time in its life, and
§12.5 already records that its enclosure and provenance fields carry no signal
today. The catalogue is visible work that attracts attention. Classification
looks finished.

Four of roadmap §7.4's thirteen catalogue entries are deferred, and the reason
is recorded here so a reader can tell "out of scope" from "forgotten":

| Deferred entry | Until | Why |
|---|---|---|
| Tangent spheres | M4 | Needs two curved surfaces to be tangent along something; M2.5 has planes |
| Tangent cylinder–plane | M4 | Same — the tangency is a curve on a curved surface, and no such surface participates in a planar fuse |
| Seam-crossing intersections | M4 | A seam is where a periodic surface's parameter domain joins up. A plane has no seam |
| Closed periodic surfaces | M4 | A plane is neither closed nor periodic |

The other nine — coplanar faces, coincident edges, vertex-on-face,
vertex-on-edge, zero-length edges, sliver faces, fully-contained solids,
touching-but-not-overlapping, and self-intersecting inputs — are all reachable
with planes and are all in M2.5, which is most of §1.7's argument for building
the fuse this early.

**Amended 2026-08-31, with what two of the pieces turned into.** Piece 6 shipped
as the table says: `SolidView::classify_uncertain_point` is the one body, and
`classify_point` is that function called with an exact point and nothing else.
What the table could not know is that the generalization was not where the cost
was — §17.8 has that — and that the bug it uncovered was in the old body rather
than the new entry. `FaceView::for_each_triangle` fans a face from one corner of
its outline, so the triangles overlap and a hole is covered twice, once each
way. Asking one triangle at a time whether it holds the point therefore put
every point over a hole on the solid's skin. The boundary question is now asked
of the whole outline: a half-line from the point, edges counted as they cross
it, corners on the line read as a half-open range so there is no tie to break.

Piece 10's bound is `Isometry::POINT_ROUNDOFF`, `18·ε` per unit of the largest
coordinate, published beside `apply_to_point` as §15.6 rule 1 requires and
checked by evaluating the same quaternion sandwich a second time in exact
expansion arithmetic and subtracting. `turning_a_fuse.rs` propagates it into
bounds on volume, area and centroid, with the slack factor a separate named
constant at the consumer per §15.6 rule 2. Nothing about that was a surprise,
and it is recorded because it is the only place in M2.5 where a test had to
wait on a derivation.

### 17.7 Criterion 8 — the refusal rate

§1.7's eighth exit criterion wants a measured refusal rate under
`Policy::certified()`, and §15.8 already settles what it counts: the share of
*fuse operations* over the corpus that come back `Error::Undecidable`, per
operation, because that is the unit a user experiences a refusal in.

The shape of the measurement is a decision and not a result — a single averaged
percentage would be a worse number than no number, since it would blend a
control that must never refuse with a family built to be refused on.

**Three families, reported separately, never as one number.**

| Family | What it is | What its number means |
|---|---|---|
| The §7.4 catalogue | the nine planar entries, hand-written | a pass/refuse **table**, not a rate. Nine cases is not a sample, and each one refusing is either right or a bug on its own terms |
| Lattice-aligned pairs | integer grid, axis-aligned | the **control**. Every meet is exactly representable, so the expected refusal rate is zero and any refusal at all is a bug in our filters, not a fact about geometry |
| Rotated pairs | the same shapes under general rotations | the **headline rate**, asserted below a threshold recorded here |

The control is the row that earns its keep. §14.7's worry is that a refusal
might be undeserved — a timid filter rather than genuine undecidability — and
on the rotated family there is no way to tell the two apart from the rate
alone. On the lattice-aligned family there is: the answer is exactly
computable, so a certified policy must decide it, and one refusal is one bug.

All three are machine-checked and asserted, in the ordinary test suite. §12.5
had to record that criterion 10's ≥0.8× half "is not machine-checked; it is a
`criterion` run a human reads", and that caveat should not need writing twice.

**Measured 2026-08-31.** The generated families run from a fixed seed, so the
figures below reproduce; the properties beside them run from a fresh seed every
time, because their job is to find inputs nobody has thought of and a frozen
seed would stop them finding any.

| Family | Corpus | Refused | Rate | Where |
|---|---|---|---|---|
| The §7.4 catalogue | 13 cases over nine entries | 3: two `ResultIsNotManifold`, one `ArgumentIntersectsItself` | table, not a rate | `the_nine_planar_entries_as_a_table` |
| Lattice-aligned pairs | 138 configurations × 3 rules = 414 fuses | **0** | 0% | `every_certified_fuse_of_a_lattice_pair_is_decided_and_records_nothing` |
| The same, built to share coordinates | 138 × 3 = 414 fuses | **0** | 0% | `every_certified_fuse_of_a_pair_sharing_coordinates_is_decided` |
| Rotated pairs | 138 × 3 = 414 fuses | 105 | **25.4%** | `how_often_a_certified_fuse_of_a_turned_pair_refuses` |

**These are the figures M2.5 closed on, and the headline one has since moved.**
§19.3 records 87 of 414 — 21.0% — over the same corpus, after the reader defect
§19.1 describes was fixed; the shipped corpus is 420 fuses now, because two
proptest regression seeds went in with that fix. The rest of this section reads
the same either way: what it is about is the *shape* of the measurement, and
nothing about the shape changed.

A corpus is 128 generated configurations plus the ten seeds `proptest` has kept
from past failures, each asked all three rules. The same shapes, the same
shifts and the same count appear in all three generated rows, so the only thing
between the control and the headline is the rotation — which is what makes the
comparison say anything. The control is measured twice because there are two
ways for an exactly representable pair to be hard: ordinary overlaps, and the
deliberately degenerate configurations of the last paragraph of this section,
where the same corner value appears in both solids. Neither refuses once.

None of the catalogue's three refusals is an `Error::Undecidable`, so none of
them is a refusal in criterion 8's sense at all. Two are what §17.4's first row
settles is right: two cubes meeting along a whole edge, and a small cube
touching a big one along part of one. Four faces would run along that edge and a
`Model` means an edge is used by exactly two, so the union of those solids is
not something this kernel can hold. The third is the ninth entry, an input that
runs through itself, which is a bad argument and refuses as one.

| §7.4 entry | the pair | rule | outcome |
|---|---|---|---|
| 1. coplanar faces | two cubes sharing a whole face | Union | answered |
| 1. coplanar faces | two slabs whose faces half overlap | Union | answered |
| 2. coincident edges | two cubes catty-corner, sharing one whole edge | Union | refused, `ResultIsNotManifold` |
| 2. coincident edges | the same two, asked what they share | Intersection | answered |
| 3. vertex on face | two cubes offset by a corner | Union | answered |
| 4. vertex on edge | a box poking out of the top of a cube | Union | answered |
| 5. zero-length edges | a cube and two bars whose walls are on the same planes | Union | answered |
| 6. sliver faces | a cube and a knife a millionth of the way in | Intersection | answered |
| 7. fully contained solids | a cube in the middle of another, hollowed out | Difference | answered |
| 8. touching, not overlapping | two cubes touching on a face | Union | answered |
| 8. touching, not overlapping | two cubes touching along part of an edge | Union | refused, `ResultIsNotManifold` |
| 8. touching, not overlapping | two cubes touching at a corner | Union | answered |
| 9. self-intersecting input | two cubes wired into one solid that runs through itself | Union | refused, `ArgumentIntersectsItself` |

**The headline rate broken down by which predicate could not decide is the half
of criterion 8 anybody can act on.** The rate alone says how often the kernel
gives up; only the breakdown says on what, and §14.7's whole worry is that a
refusal might be undeserved.

| What could not be decided | Refusals | Share of the 414 |
|---|---|---|
| `"side of plane"` | 105 | 25.4% |
| `"a point inside a piece of a face"` | 0 | 0% |
| everything else | 0 | 0% |

Every one of the 105 comes from one question: whether a corner the fuse
re-derived lies exactly on a cut line's plane. Under §17.1 a corner is the meet
of three planes and so a box, and a box sitting on a plane it was solved from
straddles it however much precision is spent, so the splitter cannot tell
whether that line runs through that corner. Fourteen fresh seeds over the same
corpus put the rate between 19.6% and 28.3%, and no run of them has ever named a
third cause; `"a point inside a piece of a face"` still turns up on some seeds,
at most twelve out of about a hundred.

That second cause has already earned its keep. Before the two fixes below it was
a much larger share, and having it named separately is what showed that those
refusals were **undeserved** rather than genuinely undecidable. A face was read
one triangle of its own fan at a time, and the fan is a signed overlapping one,
so a hole is covered twice and a point over it came back `OnBoundary`; and ON
was being settled by casting a ray, when §17.1 makes it a question about two
planes' coefficients, which no box widens. Fixing both took the rate from 32.8%
to 25.8% and took `A \ A` over a turned solid from every case refused to none.
Neither would have been visible in the total on its own.

The threshold is **24%**, asserted by
`how_often_a_certified_fuse_of_a_turned_pair_refuses` in
`crates/rcad-topo/tests/fuses_that_must_agree.rs`. It is chosen from measurement
rather than from taste, and it has moved down twice as undeserved refusals were
found and fixed — 32%, then 28%, then this. What it has to do each time is sit
close enough under the current rate that any one of the fixed defects coming
back on its own trips it, and far enough over it to leave room for ordinary
drift; the constant's own doc carries the arithmetic. A threshold is not a
target — what is recorded is the measurement, and the assertion is only there so
the measurement cannot drift without somebody noticing.

**The headline family and the degenerate family have to be read together, and a
reader who takes 25.4% on its own will misread it.** The turned family is the
lattice family rotated, and the lattice family is full of configurations that
are *exactly* degenerate: faces flush against each other, corners landing on
faces, solids offset by a whole extent. Turning those leaves them geometrically
degenerate while their two stored planes stop agreeing in the last bits, which
is precisely the question the 105 refusals could not answer. So a large share of
the headline number is a statement about how degenerate the corpus is, not only
about how timid the filters are — and the fact that the very same
configurations refuse zero times when they are not turned is what shows the
filters are not the whole story.

What the number does not say is anything about real models. It is measured over
*this* corpus: four rectilinear outlines, small integer coordinates, two solids
placed within a few units of each other and deliberately steered into
coincidence. Roadmap §13's second risk — "certified approach refuses too much to
be usable" — is about parts people draw, which are mostly not degenerate and
mostly do not have two faces a rounding apart. A quarter of *this* corpus
refusing is therefore evidence and not a verdict: it bounds how bad the answer
can be on a corpus chosen to be hard, and the honest reading is that the risk is
still open and now has a number attached to it instead of nothing.

The generators are three configuration families and not M0's extruded-polygon
configurations reused wholesale. M0's generator supplies *shapes*, and we take
it whole for that; what we do not take is its configurations, because §12.3 is
the section where one family of shapes found one family of bugs and a 335-test
suite could not see a right triangle. Two of the three families above are new
configurations of old shapes, which is the cheapest form §13 rule 5 takes.

The degenerate configurations are built by **sharing coordinates between the two
solids**, so the degeneracy is exact: the same corner value appears in both
inputs, and the coplanarity or the vertex-on-face is true rather than nearly
true. Rejected: nudging a valid pair by 1e-9 to walk into the dangerous band,
which is what roadmap §11.3 recommends and which is right for fuzzing. It
generates the *near* case and never the *on* case, and the on case is what the
catalogue is about — coplanar faces means coplanar, not coplanar to within a
nanometre. We take §11.3's instruction and note that our dangerous band has a
boundary we can land on exactly, so we construct it instead of approaching it.

**Amended 2026-08-31, after a soak at scale.** Everything above was measured over
128 generated configurations, and three of the claims made from that sample are
artefacts of its size. A soak over twelve fresh seeds at 12,800 configurations a
seed — 38,400 fuses each — says:

- The turned family's rate is **26.0% to 27.7%, mean 26.66%, standard deviation
  0.46 points**. That is a tenth of the ±4 points the same corpus spread over at
  414 fuses. The 25.4% above is inside the range and was never wrong; what was
  wrong is how much of it looked like signal.
- The two controls refused **0 of 921,600 fuses** between them, which is the
  control claim holding over a corpus two thousand times the one above.
- **A third cause exists.** "No run of them has ever named a third cause" is
  false. `"order along a line"` turns up at 0.13% on all twelve seeds, and
  `"orient3d"` on two of the twelve. Neither can be seen at 414 fuses, where the
  expected count of the commoner of the two is one half.

So `MEASURED_CASES` is **1,000** rather than 128, and each row is now 1,000
generated configurations plus the ten seeds `proptest` has kept, each asked all
three rules: 3,030 fuses. The ceiling on it is the clock and not the statistics
— two thousand cases takes the whole workspace suite from ten seconds to two
minutes, a thousand takes it to one, and one minute is what this measurement is
worth. The by-cause breakdown prints two decimal places, because one place
rounds a cause at 0.13% to the same figure as a cause ten times commoner.

| Family | Corpus | Refused | Rate |
|---|---|---|---|
| Lattice-aligned pairs | 3,030 fuses | **0** | 0% |
| The same, built to share coordinates | 3,030 fuses | **0** | 0% |
| Rotated pairs | 3,030 fuses | 825 | **27.2%** |

| What could not be decided | Refusals | Share of the 3,030 |
|---|---|---|
| `"side of plane"` | 807 | 26.63% |
| `"a point inside a piece of a face"` | 12 | 0.40% |
| `"order along a line"` | 6 | 0.20% |
| everything else | 0 | 0% |

The third cause is no surprise once it has been seen. Two points on a cut line
whose intervals on that line's dominant axis overlap cannot be put in order, and
§17.4 settles that an overlap there is the coincidence case rather than a gap;
under a general rotation, two corners a rounding apart produce exactly that
overlap. It is one fuse in five hundred. Naming it separately is the entire
reason the breakdown exists, and it took a corpus seven times the shipped one
before it was named at all.

**The threshold moves from 32% to 30%**, and the tightening is what the precision
buys. Thirty-two sat above every fresh seed by so much that it was guarding
sampling noise rather than drift: neither of the two undeserved refusals recorded
above would have tripped it on its own, since each was worth about 3.5 points and
only the two together reached 32.8%. The fixed-seed rate is 27.2%, the fresh-seed
spread at this corpus size is about ±1.6 points, so the widest seed lands near
29.5% — and 30% is above that while still tripping on either bug coming back. It
is asserted in the same place by the same test.

**Amended 2026-08-31, and `MEASURED_CASES` goes back to 128.**

The raise to a thousand took the workspace suite from ten seconds to sixty, and a
minute on every run is more than this measurement is worth — these numbers change
about once a milestone and the suite runs about once a minute. The corpus is 128
generated configurations plus the ten `proptest` has kept from past failures
again: 138 configurations, **414 fuses per family**, and the suite back to about
ten seconds.

Nothing measured at scale is withdrawn. What changes is how much of it the suite
re-checks, and that split is the point of this amendment.

**Checked on every run**, at 414 fuses per family, from a fixed seed:

| Family | Corpus | Refused | Rate |
|---|---|---|---|
| Lattice-aligned pairs | 414 fuses | **0** | 0% |
| The same, built to share coordinates | 414 fuses | **0** | 0% |
| The same, written somewhere else in `f64` | 414 fuses | **0** | 0% |
| Rotated pairs | 414 fuses | 105 | **25.4%** |

| What could not be decided | Refusals | Share of the 414 |
|---|---|---|
| `"side of plane"` | 105 | 25.36% |
| everything else | 0 | 0% |

**Not checked on any run.** The 26.66% ± 0.46 over twelve seeds and 38,400 fuses
each, the zero refusals in 921,600 control fuses, and the third refusal cause are
all evidence from a **soak the suite does not re-run**. Three consequences, said
plainly rather than glossed:

- The fresh-seed spread is back to about **±4 points**. The asserted number is
  still deterministic — the measurement runs from a fixed seed — so nothing goes
  flaky. What drops is *power*: 414 fuses is a far noisier estimate of the true
  rate than 38,400, and a regression has to be larger before the threshold can
  see it.
- The third cause, `"order along a line"` at 0.13%, is **below the resolution of
  the default run**. Its expected count over 414 fuses is one half, and the run
  above names it zero times. It turned up on twelve seeds out of twelve at scale.
  **A default run that does not name it is not evidence that it has gone.**
- `"a point inside a piece of a face"` at 0.40% is in the same position: at 414
  fuses some seeds see it and most do not, and the fixed seed does not.

This puts §12.5's complaint back, in this section's own voice. §12.5 had to
record that criterion 10's ≥0.8× half "is not machine-checked; it is a `criterion`
run a human reads", and this section says above that "that caveat should not need
writing twice". It is being written twice. The trade is deliberate, the reason is
suite wall clock, and a reader is owed the split rather than an average of the
two.

**The soak is one command and not a code edit**, which is what makes the split
tolerable. `MEASURED_CASES` is multiplied by `RCAD_SOAK_FACTOR` and the fixed seed
is replaced by `RCAD_SOAK_SEED`, both read at run time:

```sh
for seed in $(seq 1 12); do
  RCAD_SOAK_FACTOR=100 RCAD_SOAK_SEED=$seed cargo nextest run -p rcad-topo \
    --no-capture how_often_a_certified_fuse_of_a_turned_pair_refuses
done
```

Both variables are documented at the constants they drive, in
`crates/rcad-topo/tests/fuses_that_must_agree.rs`.

**The threshold moves from 30% to 28%.** The fixed-seed rate over 414 fuses is
25.36% — 105 refusals — and it is the same number on every machine, so the
threshold has no sampling noise to clear and only ordinary drift. Two and a half
points is eleven fuses. Thirty would no longer do the job the amendment above gave
it: each of the two undeserved refusals recorded there is worth about 3.5 points
at this corpus, which puts the rate at 28.9%, so 30% would catch neither of them
alone where at 3,030 fuses it caught either. Twenty-eight catches both.

**And it is judged on the shipped corpus only.** A soak is not the same
measurement: six fresh seeds at ten times the corpus came out at 27.0%, 27.1%,
27.2%, 27.2%, 28.1% and 28.4% over 3,870 fuses each — a different mean and a
different spread, with two of the six above 28% and nothing wrong. Judging those
by the shipped corpus's threshold would be treating two numbers as one, which is
the mistake the top of this section exists to avoid. A soak prints its rate,
prints its by-cause breakdown, and judges nothing; the fixed 414-fuse run is what
carries the assertion. That is `this_is_the_shipped_corpus` in
`fuses_that_must_agree.rs`, and it is the same trade as everything else in this
amendment: what the machine checks is the small number, and the big one is read
by a person.

Those six seeds are also what says the soak mechanism works. Each named
`"order along a line"` on four of the six, at 0.08% to 0.23%, and
`"a point inside a piece of a face"` on all six at 0.62% to 1.01% — both causes
that the shipped 414-fuse run sees zero times.

**A fourth measured family, and it is a control.** The lattice pairs with every
coordinate multiplied by `2^k` and moved by `2^m` along each axis, both exact.
Zero refusals of 414. It is a sharper control than the lattice family alone,
because it is the same fuse asked at sizes from `2^-280` to `2^30`, and measurement
says the size changes what the arithmetic can settle: the same overlapping pair
answers down to `k = -356` unshifted and only to `k = -332` shifted by `2^30`. Its
generator is `pairs::a_pair_somewhere_else`, it answers §14.1's four questions
there, and it found two defects — §17.9.

**Two vacuity guards, because three of these four rows could have been green over
nothing.** `every_certified_fuse_of_a_lattice_pair_is_decided_and_records_nothing`
asserted `refused == 0` and nothing else, so a regression that made *every* fuse
come back `ResultIsNotManifold` would have kept that assertion true and left every
metamorphic property beside it vacuously passing — each of them returns `Ok(())`
the moment a fuse has no answer. Measured on this tree, 20 of the 414 control
fuses and 72 of the 414 degenerate ones are legitimately not solids, so the guard
is a plausible band and not a zero. Each measured row now judges that share
against one; each property reports how many of the comparisons it set out to make
it reached, and fails below three tenths. Demonstrated by making `Model::fuse`
return `ResultIsNotManifold` unconditionally: the refusal assertions stay
satisfied, and all eighteen tests in the file fail anyway — the measured rows on
the band and the properties on the floor. `turning_a_fuse.rs` had already solved
this for itself, and the guard here is deliberately the same shape rather than a
second one.

### 17.8 What M2.5 actually cost, recorded 2026-08-31

M2.5 is complete. §12 did this for M0; this is the same for the fuse, and it is
here rather than in a new top-level section because the milestone's record
should read in one piece.

**The schedule risk was where §17.6 put it and not where §1.7 did.** §1.7 named
item 5, the catalogue, "where the roadmap says booleans actually cost". §17.6
disagreed in the same file and named piece 6, classification — "shipped,
benchmarked, and about to be handed constructed points for the first time in its
life". §17.6 was right, and the evidence is three separate things.

The catalogue cost about what it looked like. Writing it turned up two
coincidences the fuse could not see, and both were the same function giving up
on something it could have decided: `coincident_face` asked for a winding of
exactly `+1` about the piece's own normal, so every `OppositeWay` coincidence —
which is most of them — came back as no coincidence at all; and the winding
count fanned from one fixed corner and gave up whenever the point sat on a chord
of that fan, which every candidate a rectangular piece can offer does. Both fixes
are inside one function. Four entries later had to have their asserted counts
restated, and that was fallout from a fix elsewhere rather than catalogue work.

Classification cost more, and it cost it twice, both times in exactly the part
§12.5 had already recorded as carrying no signal. `FaceView::for_each_triangle`
fans a face from one corner of its outline, so its triangles overlap and a hole
is covered twice, once each way; reading one triangle at a time therefore put
every point over a hole on the solid's skin, and a frame united with a plug
filling one corner of its hole refused. That pair is lattice-aligned, so it is
§17.7's control family, where a refusal is a bug by definition. Separately, ON
was being settled by casting a ray, which cannot work when the point is a box
sitting on the very plane it was solved from — §17.1 makes ON a question about
two planes' coefficients, which no box widens. Together those two took the
headline rate from 32.8% to 25.8% and took `A \ A` over a turned solid from
every case refused to none.

Neither of those was found by the catalogue, and neither was found by review.
**Every defect that mattered in M2.5 was found by a metamorphic property, not by
a hand-written case.** Commutativity found the coplanar-partition hole on its
first run: two ells flush against each other united into twenty-three faces one
way round and twenty-six the other, the same solid both times. Associativity and
`A \ (A \ B)` found the other half of the same hole, that a corner between two
coplanar faces has two planes round it and not three, so the *next* fuse over
such an answer divides by a determinant of exactly zero. §17.4's amendment is the
one fix for both. §12.2 preferred "a property the answer must satisfy over an
authority to compare against" on the strength of one bug; it now has a second and
much larger data point, and the two together are the general form: the risk is
the part that looks done, and the oracle that finds it is a property rather than
a case. The catalogue is the part everyone looks at, which is why the bugs were
not in it.

One place the cost went that neither prediction named is the closing gate's
bound. §15.6 settles that a published bound needs a test that could fail, and a
bounded relation is worth nothing until the bound exists, so `turning_a_fuse.rs`
could not be written at all until `Isometry::POINT_ROUNDOFF` had been derived —
a page of arithmetic charging each term of the quaternion sandwich for every
rounding it passes through. That is not much work, but it cannot be done
alongside the test that needs it, which is §13 rule 1 seen from an angle that
section does not cover: a piece whose oracle has to be *derived* first is one
piece and not two, however separable the code looks.

**All eight of §1.7's exit criteria are met, and so are §14.8's three extra
oracles.** One at a time, against the shipped code:

| §1.7 criterion | The test that discharges it |
|---|---|
| 1. One decomposition and three selection rules over sets of arguments and tools, not three algorithms | `fuse/select.rs` is one table read as data and one `match` on `Rule` in `row_for`; `every_certified_fuse_of_a_lattice_pair_is_decided_and_records_nothing` asks all three rules of every configuration, `two_cubes_in_one_group_cut_each_other_the_same_as_two_in_opposite_groups` pins the set semantics, `two_tools_cutting_along_the_same_planes_merge_instead_of_making_an_edge_of_no_length` hands it two tools and `uniting_three_at_once_is_uniting_two_and_then_the_third` three arguments |
| 2. Face and edge splitting at intersections, with image maps | `the_pieces_of_a_face_add_up_to_the_area_it_started_with`, `every_side_of_a_piece_is_walked_once_and_a_shared_one_once_each_way`, `every_face_of_an_answer_names_the_input_face_it_was_cut_from` |
| 3. Classification reused, not reimplemented | `classify_point` is `classify_uncertain_point` called with an exact point and nothing else; `the_exact_way_in_and_the_general_one_give_the_same_answers` |
| 4. History maps returned per operation | `the_input_keys_still_work_and_a_face_nothing_reaches_comes_through_once`, `a_face_that_is_swallowed_whole_has_no_image`, `every_face_of_the_answer_has_exactly_one_input_face_behind_it` |
| 5. The catalogue as hand-written cases, each passing or refusing with a reason | `the_nine_planar_entries_as_a_table` and the eighteen per-entry tests beside it in `degenerate_fuses.rs`; `a_solid_that_runs_through_itself_cannot_be_fused` for the ninth entry, which no public API can build |
| 6. The §11.2 metamorphic properties as a `proptest` suite | `what_the_union_and_the_overlap_take_up_is_what_the_two_solids_take_up`, `uniting_the_first_two_and_then_the_third_is_uniting_the_last_two_and_then_the_first`, `taking_away_what_is_left_after_taking_the_other_away_leaves_what_the_two_share`, and transform invariance in both halves — `turning_a_pair_a_quarter_turn_and_moving_it_a_whole_number_changes_not_one_bit` exactly, `turning_the_answer_to_a_fuse_agrees_with_fusing_the_turned_inputs` within the bound |
| 7. `Policy::max_wobble` has a caller, judged against `ProvenanceLog::worst_wobble` | `Model::fuse` judges it after assembly and before the commit; `a_turned_pair_is_certified_because_a_box_is_not_a_decision_anybody_failed_to_prove` and `a_fuse_that_needs_a_decision_nothing_can_prove_refuses` are the two halves, and `a_certified_fuse_over_a_tagged_input_succeeds_and_the_answer_keeps_the_tag` is §17.2's guard against reading the wrong quantity |
| 8. A measured refusal rate under `Policy::certified()`, recorded in this file | §17.7, asserted by `how_often_a_certified_fuse_of_a_turned_pair_refuses` with `every_certified_fuse_of_a_lattice_pair_is_decided_and_records_nothing` and `every_certified_fuse_of_a_pair_sharing_coordinates_is_decided` as its controls |

| §14.8 oracle | The test that discharges it |
|---|---|
| Commutativity of union and intersection | `swapping_the_two_sides_of_a_lattice_pair_changes_not_one_bit`, and the same for turned pairs and for pairs sharing coordinates, all comparing the answer's faces bit for bit; `uniting_two_ells_flush_against_each_other_gives_the_same_faces_either_way_round` is the case that first failed |
| Idempotence and absorption | `a_solid_united_with_a_copy_of_itself_is_itself` for `A ∪ A` and `A ∩ A`, `a_solid_with_a_copy_of_itself_taken_away_is_nothing_at_all` for `A \ A` over both a lattice solid and a turned one, `a_solid_united_with_what_it_shares_with_another_is_itself` for `A ∪ (A ∩ B)` |
| `fuse([A,B,C]) ≡ fuse([fuse([A,B]), C])` | `uniting_three_at_once_is_uniting_two_and_then_the_third`, with `uniting_the_first_two_and_then_the_third_is_uniting_the_last_two_and_then_the_first` beside it for the order the three arrive in |

Two things §17.6 promised that did **not** ship, recorded rather than claimed.
Its ninth piece names "Euler characteristic and shell counts on the assembled
result" as what decides shell reassembly is correct. Shell counts are checked, by
name, in `a_cube_wholly_inside_another_under_all_three_rules` and
`a_cube_with_a_smaller_one_taken_out_of_the_middle_is_one_solid_with_a_void`, and
compared between two runs by every property that goes through `invariants_of`.
The Euler characteristic is checked nowhere. What stands in for it is
`Model::validate` over the whole committed result in debug builds, which asks
that every edge is used by exactly two coedges and every wire closes — from which
χ follows rather than being an independent check on it. That is less than the
piece promised.

And inclusion and exclusion is asserted on lattice pairs only. On turned pairs
each of its four volumes is a rounded sum over a solid whose corners are already
a rounding away from the planes they were solved from, and bounding the sum needs
a bound on how badly conditioned those plane triples are, which nothing in this
tree has. `Isometry::POINT_ROUNDOFF` does not reach it: that bound covers turning
an answer that has already been computed, not the fuse's own arithmetic in the
turned frame. Guessing a tolerance instead would make the test measure the
tolerance (§14.1 question 1), so the property is not run there and the gap is
written down at the top of `fuses_that_must_agree.rs` as well as here.

**Amended 2026-08-31, and three things above need correcting.**

The paragraph on the metamorphic properties says §17.4's amendment "is the one
fix for both" halves of the coplanar hole. Joining coplanar faces of the
*answer* was half of it. A public builder can hand the fuse an input that already
has two faces on one plane sharing an edge, and §17.1 cannot read that either:
`ProfileBuilder` accepts three profile points in a row, the extrusion of one has
two side faces on a single plane, and every fuse over such a solid refused —
under all three rules, in debug and release, and whether the pair overlapped,
touched or was nowhere near. §17.4's amendment of today records the rest of the
fix. Criterion 8's row still points at §17.7, which now records the same three
families over ten times the corpus, a third refusal cause, and a threshold of
30% rather than 32%.

The method is worth recording, because it is §12.3's lesson with a cost ratio
attached. Three million fuses drawn from the three shipped generators found
nothing, in 934 seconds. One new shape family — a profile with a point halfway
along a wall — found this in 0.02 seconds and under forty fuses. §12.3 is the
section where one family of shapes found one family of bugs and a 335-test suite
could not see a right triangle; this is the same sentence with numbers on it, and
the numbers are about fifty thousand to one in favour of a new shape over more of
the old ones. **The corpus that has already run is worth almost nothing; the
shape nobody has drawn yet is worth almost everything.**

The last of it is about comments. `bake.rs` argued, in the doc comment on the
very branch this bug reached, that "the corner of a solid this fuse itself built
always has three different planes round it ... so reaching here means the solid
came from somewhere else". Every word of that is true, and the second half names
the way in while calling it a place nobody comes from. A comment explaining why
something is unreachable is a claim, and a claim that spells out its own
counterexample is worth reading twice.

### 17.9 Three defects found after M2.5 closed, recorded 2026-08-31

*After M2.5 closed. §17.8 says the corpus that has already run is worth almost
nothing and the shape nobody has drawn yet is worth almost everything; this is
the same sentence about **where** a shape is drawn rather than **what** shape it
is.*

**All three are fixed; §19 is the record, and it is worth reading beside this
one.** The first turned out to have a second half this section could not see,
the second took criterion 7's own demonstration down with it, and two further
claims fell out while they were being fixed. Every test named below now says
what it checks instead of what it could not; §19 gives the new names.

Two new generators and one new oracle went in after §17.8 was written. Between
them they found three defects, and all three are **gaps** rather than limits
(§16.10): the answer exists, is exactly representable, and the arithmetic we
chose does not reach it. None is fixed here. Each has an `#[ignore]`d minimal
reproduction naming what it is, so the suite stays green and nothing is quietly
patched.

All three are also §16.11's open question with teeth. `DynamicRange` is a
*ratio*, and every one of these models has a perfectly ordinary ratio. It is the
absolute magnitude — of a coordinate, of a plane offset, of a normal's squared
length — that breaks them.

**1. A plane read out of a model is not the plane that was put in.**
`FaceView::plane` is `surface.transformed(&location)`, and `Plane::transformed`
rebuilds the offset as `normal · movement(origin())` where `origin()` is
`normal * offset` — so the round trip computes `offset · (nx² + ny² + nz²)`. For
a normal only as close to unit length as `f64` allows, that squared length is not
exactly one, and the offset comes back one ulp further out. **Even for
`Isometry::identity()`.** Applied twenty times a measured plane walks from
`-1.0000000000000002` to `-1.0000000000000047`, monotonically, with nothing to
stop it.

This is §17.1's deciding argument failing at the accessor. That section chose
plane-authoritative geometry over vertex-authoritative geometry because "a split
face inherits its parent's plane bit for bit, so however long the chain the
planes in the model stay the planes the user built, undegraded", and the fuse
reads its inputs' planes through exactly this accessor — so a chain of fuses
moves every non-axis-aligned plane one more ulp per fuse. The slow leak that
argument rejected arrives through the getter instead of through the geometry.

What is **not** wrong is the fuse. The plane the answer *stores* is the input's
plane bit for bit: reading the answer back gives one ulp more and reading it back
twice gives two, which is only consistent with the arena holding the undegraded
value. Axis-aligned planes are unaffected, because a signed unit axis has a
squared length of exactly one — which is why the whole existing suite is green
and why it took a turned chain to see this.

Found by the oracle §17.1 always deserved and never had: **every face of every
answer has to carry some input face's four coefficients, bit for bit, or their
exact negation.** It is folded into the fuse helper in
`fuses_that_must_agree.rs`, so all of that file's fuses over exactly
representable families check it at no extra wall clock. It is not run on turned
pairs, and the reason is this defect: there the two sides have been through
different numbers of reads and a mismatch would be a fact about the accessor
rather than about the fuse. Reproductions:
`moving_a_plane_by_the_identity_walks_its_offset_one_ulp_further_every_time` in
`rcad-geom`, and
`uniting_a_turned_solid_with_copies_of_itself_never_moves_a_plane` next to the
oracle.

**2. A solid whose faces are more than about `2^72` from the origin cannot be
fused at all.** `fuse/select.rs`'s `InPlane::of` needs a point it can *prove* is
off a face's plane, and looks for one by stepping along the normal from the
plane's nearest point to the origin by `1.0`, then `1024.0`, then `1048576.0` —
three absolute distances. Past an offset of about `2^72` the `f64` spacing there
is wider than the largest of the three, every step lands back on the plane, and
the fuse returns `Error::Undecidable` naming `"a point off a face's plane"`. The
doc comment on that function says "a bigger reach only helps when the plane's own
offset is large enough to swallow a smaller one" and then hard-codes three
reaches — §17.8's last paragraph again, a claim that spells out its own
counterexample. Reproduction:
`a_cube_too_far_from_the_origin_cannot_be_fused_at_all`.

**3. A solid more than about `2^28` from the origin is said to enclose negative
space.** `SolidView::measure` and `signed_volume` sum `a · (b × c)` over the
answer's triangles with `a`, `b` and `c` measured **from the world origin**, so a
solid `2^m` away adds terms of size `2^{3m}` to reach a volume of order one. Past
`m = 28` the spacing of those terms is larger than the answer, what comes back has
no correct digits and can be negative, and `Model::validate` — which the fuse runs
over its own answer in debug builds — refuses a perfectly good solid.
`DynamicRange` sees a ratio of `2.7e8` here and is satisfied. Summing about the
solid's own first corner instead gives the exact answer, because the differences
are exact by Sterbenz's lemma when the coordinates share an exponent.
Reproduction:
`a_frame_a_long_way_from_the_origin_is_said_to_enclose_negative_space`. The frame
is the shape that goes first, at eight units of volume; a cube survives further,
because a box's terms cancel in opposite pairs and a frame's do not.

**What the deeper chains found: nothing, and that is worth recording too.** §17.8
notes that associativity found the coplanar-corner hole at depth two and that
§17.1's argument is about depth three and four, so `a_chain_of_fuses` builds four
solids and runs `fuse([A,B,C,D])` against uniting them one at a time, and
`((A ∪ B) \ B) ∪ B` against `A ∪ B`. Both pass. A fourth fuse's inputs do have
properties no generated input has — corners with four or more planes round them,
faces joined out of coplanar pieces, shells built by `nest_the_shells` — and the
fix §17.4's amendment made for depth two covers them. The depth-four property is
also the file's least productive: a sixth of its fuses have no answer to give,
because four solids in a row often leave an edge with four faces along it, and it
is what sets the vacuity floor §17.7 records.

**Where the three came from is one sentence.** Not a new shape. The *same* four
outlines, on the same integer grid, with every coordinate multiplied by a power of
two and moved by a power of two — two knobs, both exact, both leaving the geometry
identical. §12.3 is where one family of shapes found one family of bugs; this is
where one family of *sizes* did, and the two knobs cost about eighty lines.

---

## 18. Scaling is a technique, not only an oracle

*Recorded 2026-08-31, after the §16.10 claim audit reached `rcad-predicates`.*

M0 wrote a metamorphic oracle: scale a predicate's inputs by `2ᵏ` and the answer
must not change (§14.2). It has been one of the sharpest tests in the tree ever
since, and §14.1 explains why — the two runs take different paths through the
dangerous code.

Nobody noticed that it is also **a way to compute**.

### 18.1 What was being refused, and why it was a gap

```
orient2d([1e-200,0], [0,1e-200], [0,0])   → Undecidable
   all six coordinates × 2^600 (exact)     → Pos, at Level::Float
orient3d at 1e300                          → Undecidable
   all twelve coordinates × 2^-700 (exact) → Pos
```

By §16.9's test these are **gaps, not limits**: the sign exists and exact
arithmetic reaches it. `orient2d` is homogeneous of degree 2 in its coordinates
and `orient3d` of degree 3, so scaling every input by `s > 0` multiplies the
determinant by `s^d` and cannot change its sign. A power of two rounds nothing.

The refusal was excused by the sentence §16.11 corrected in three files and
could not reach in two more — `orient2d.rs` and `orient3d.rs` were **frozen**
under §15.9 at the time. The freeze that made this crate trustworthy is exactly
what let a retired sentence go on doing load-bearing work inside it. That is a
finding about process, not about arithmetic, and it is the argument for auditing
a crate *when its freeze lifts* rather than treating the freeze as a warranty.

The crate's own test asserted *"there is no honest sign to report"*, and the
property refuting it was three files away being used as an oracle.

### 18.2 What was built

A shared helper in `rcad-predicates`, called by `orient2d` and `orient3d` at the
entry point both the plain and tolerant routes funnel through.

- **Lazy.** First pass exactly as before; the retry runs only on `Undecidable`.
  Zero cost on the path that already worked. Retrying at the entry point re-runs
  the whole cascade, which is necessary rather than tidy: the `1e-200` case fails
  at *all three* levels — subnormal products, collapsed intervals, overflowing
  expansions — so a fix inside `on_floats` would not have been one.
- **Normalised on the largest difference**, not the largest coordinate, because
  these predicates subtract before they multiply.
- **A self-check that decides limit from gap.** Every scaled coordinate must
  still be normal, or the retry is discarded and the original refusal stands.
  Coordinates near `1e300` with differences near `1e-300` cannot both be brought
  into range, and that model is a genuine limit that must keep being refused.
  The guard makes the boundary a property of the code rather than of a comment —
  which is the whole of §16.10 applied forwards.
- **Nothing is marked.** No `Level` variant, nothing in provenance. A rescued
  answer is certified in exactly the sense an ordinary one is, and §5's empty log
  has to keep meaning what it says.
- **Two predicates only.** The surfaces would each need their own parameters
  scaled — a radius, a plane offset — and their gap is plausible rather than
  demonstrated. §13 rule 3.

**The bounds survive untouched**, and this is the part that had to be checked
rather than asserted (§16.3). `value` and the permanent are sums of the *same*
degree-`d` products, so an exact scaling moves both by exactly `s^d` and the
comparison `value > BOUND × permanent` is scale-invariant. Asserted bit for bit
in tests over full-mantissa coordinates, not argued in a paragraph.

### 18.3 The oracle nearly went quiet, and two probes already had

Self-normalisation is precisely the change that could make both runs of the
scaling oracle take the same path, turning §14.1 question 2's answer from "yes"
into a tautology that still passes. It was checked *before* the change landed.

It was not blinded — the retry only follows a refusal, and a wrongly certified
level-1 answer is not a refusal. But making that **measurable rather than
argued** found two probes that were already dead: `orient3d`'s overflow search
looked only at the minors, when it overflows in the height products; and
`orient2d`'s never overflowed anything at all, because shifts of 300–400 on
coordinates below `2²³` cannot. That run had been reporting 197,000 healthy
certified samples and testing nothing.

The suite now counts how many samples actually reach the dangerous path and
fails if that count collapses. **An oracle that cannot say how much it is
testing is one nobody will notice going quiet** — §16.9, arriving for the third
time in two milestones, and the first time it was caught before it cost anything.

### 18.4 A point that is not a point

`Interval::exactly(NaN)` maps to `ENTIRE`, justified by *"an enclosure of
everything can never certify a sign"*. False: a zero coefficient annihilates it,
and `orient3d` on a point with a `NaN` coordinate returned a certified `Neg`
under `Policy::certified()` with an empty log.

Not *unsound* — certification needs the value constant over every real in that
coordinate, and it is — but a kernel whose premise is "prove it or admit you
cannot" must not answer confidently about a point that does not exist.

Guarded at the point of use in all six production predicates, not at
construction: enforcing finiteness inside `Point` means private variants, `From`
becoming `TryFrom`, and 80 call sites growing error handling for a case that
cannot arise in them. Refusing to *answer* costs one check where the predicate
already reads its inputs, and matches `product_is_normal`, which is a
point-of-use guard too.

A constructed point holding an unbounded interval is deliberately left alone.
Every interval the public API can build denotes a non-empty set of reals, so a
sign certified across it is a true statement about every point in the box. `NaN`
differs because there is no point at all.

Surfaces turned out to be able to hold non-finite parameters — `transformed` is
infallible and overflows — so a cylinder whose axis origin had overflowed
certified `Pos` about an ordinary finite point. Also guarded; the infallible
`transformed` is not fixed.

### 18.5 The net under the bounds had 7× of slack

§15.6 says a published bound needs a test that could fail. Mutation testing
showed `ORIENT3D_ERROR_BOUND` — `7ε` — could be cut to **`1ε`** with the entire
suite green. §16.3's bound was wrong by nine roundings to seven; **this net would
not have caught it.**

Searching rather than guessing — random exponents, full mantissas, then hill
climbing over `|value − exact| / permanent`, about 10⁸ candidates — found `6.286ε`,
cross-checked against exact rational arithmetic. The net now fails below `6.25ε`.
The measured figure is in the doc comment, so the next reader knows how much
slack remains instead of re-measuring.

`ORIENT2D_ERROR_BOUND` produced a scare worth recording: a search found `3.360ε`
against a `3ε` constant. It is not a forward error bound on `value` — it bounds
the two products against the permanent, and the final subtraction's rounding is
absorbed because `|value|` never exceeds the permanent. Shewchuk's filter working
as designed and as its own doc comment says. The test now asserts the quantity
the bound is actually about.

### 18.6 What this does not fix

- **The large end.** `orient2d` and `orient3d` never fail there. The `1e140`
  cube's `classify_point` failure is a ray target derived from the bounding box,
  in `rcad-topo` — §16.11 carries it with a hypothesis attached. *(Both halves
  of that sentence are wrong; §20.7 measured it. There is no failure left, and
  the ray target was never the cause.)*
- **`product_is_exact` reads like an overflow gate and is not one.** It is true
  for `1e200` squared and true for anything times zero. It is safe today only
  because `expansion_sum` and `scale_expansion` drop an entry only when it
  compares equal to zero, and no infinity or `NaN` ever does — so `expansion_sign`
  is the real backstop. That fact had never been written down; it is now, on
  `two_diff`, with a test. Two call sites bypass the gate entirely and lean on
  `expansion_sign` alone. The doc is correct; the call sites are what mislead.
  *(§21 closed the gate. The last two sentences are wrong and §21.3 measured
  them: no call site bypasses it.)*
- **`Approximation` magnitudes and `two_diff`'s "always exact"** were both stated
  more strongly than they held and are fixed, but neither was the load-bearing
  kind of wrong.

### 18.7 The refusal rate does not move, and I had written that it would

*Corrected 2026-08-31, after measuring.*

As first written this section was headed *"The refusal rate moves, and it moves
the right way"*, and said the change "certifies cases that previously came back
`Undecidable`, so that number improves". **Measured across `a23d73d`, it does
not move at all.**

| | before | after |
|---|---|---|
| Turned pairs, the headline family | 105 of 414 = **25.4%** | 105 of 414 = **25.4%** |
| Lattice, sharing coordinates, elsewhere (controls) | 0 | 0 |

Every refusal is `"side of plane"`, before and after; the catalogue table is
identical row for row; no operation that succeeded now refuses; no answer
changed value. The harness is `Tally` in
`crates/rcad-topo/tests/fuses_that_must_agree.rs`, and it counts §15.8's number
— per operation, `Err(Error::Undecidable)` only, under `Policy::certified()`.

**The change is real; the corpus cannot see it.** Verified rather than assumed:
copying `main`'s `when_it_cannot_tell.rs` into a worktree at the previous commit
fails four tests, so `a23d73d` genuinely certifies cases its predecessor
refused. None of them lie inside §1.7's corpus, which contains no coordinates
anywhere near `1e-170` or `1e200`. The corpus is built from shapes at ordinary
working sizes, which is what makes it a good corpus for its own purpose and
blind to this one.

**Two things worth carrying.** A delta of `0/414` bounds the corpus-level
improvement near zero; it does **not** show the change is inert, and those are
different statements. And §1.7's criterion 8 measures refusals over one corpus
at one range of magnitudes — a kernel-wide "refusal rate" it is not, and nothing
should quote it as one.

**Why this correction is in the record rather than a quiet edit.** The claim
that the number improves was written because it was obviously true — more cases
certify, so fewer refuse. It is obviously true and it is not what the number
does, because the number is over a corpus that never contained the cases. That
is §16.10's shape exactly: a locally valid argument answering a subtly different
question than the measurement asks. Written by the same head that wrote §16.10,
in the same session, one commit after using it to find two panics.

---

## 19. A read is a read, and a solid is where it is

*Recorded 2026-08-31, fixing the three defects §17.9 recorded and not fixed.*

All three were gaps (§16.10) and all three are fixed. Between them they took
three more claims down with them, which is the part worth reading: each of the
three was load-bearing for a sentence somewhere else that turned out to be
false.

### 19.1 Reading a plane out of a model moved it, twice over

`FaceView::plane` is `surface.transformed(&location)` whatever the location is,
so **every read of every face pays whatever that function does to a plane**, and
a chain of fuses pays it once per fuse. `Plane::transformed` was doing two
things to a plane that `Isometry::identity()` should have left alone:

1. It rebuilt the offset as `moved_normal · movement(origin())`. `origin()` is
   `normal * offset`, so the round trip computed `offset · (nx² + ny² + nz²)`,
   and a squared length of `1.0000000000000002` — an ordinary one — walked the
   offset an ulp further out on every read. §17.9 found this.
2. It scaled the moved normal back to unit length. **Scaling a vector to unit
   length does not always give a vector whose length reads back as exactly
   one**: the direction `(6, 3, 7)` normalizes to something whose computed
   length is `0.9999999999999999`, so a second scaling grows all three
   components. This is the half §17.9 did not see, because the reproduction it
   had used a normal whose squared length rounded to exactly `1.0`.

The offset is carried over now — a turn leaves `normal · point` alone, so all a
shift can do is move the plane along its own moved normal — and the moved normal
is not scaled again. It was scaled once, where the plane was built.
`Axis::transformed` had the second defect too, so a pipe read out of a model
turned a little; same fix.

**What this cost, measured rather than argued.** `Plane`'s own documentation
already said that scaling an almost-unit normal "can move the last bit or two,
so a plane that has been written and read back is the same plane to within
rounding, not bit for bit" — about serialization. Doing it on every read made
that sentence true of reading, which is the thing §17.1 chose plane-authoritative
geometry to prevent.

### 19.2 The oracle that could not run, and the property that was too strong

`every_plane_of_the_answer_is_a_plane_of_an_input` — §17.1 stated as an
assertion — ran on about two thirds of this tree's fuses and was switched off for
the turned families, because there it would have measured the reader rather than
the fuse. It runs on all of them now, and **the turned families are where it
bites**: a turned normal is not a signed unit axis and nothing about it is exact
by construction. Widening it is what caught defect 1's second half, within a
second of wall clock.

The other half went the other way. `uniting_a_turned_solid_with_copies_of_itself_never_moves_a_plane`
asserted that `A ∪ A` gives back `A` bit for bit, corners included.
(It is `uniting_a_turned_solid_with_more_copies_of_itself_settles_after_the_first_union`
now, for the reason below.) **That is
not what §17.1 asks**, and the plane defect had been hiding the difference. §17.1
says the plane is authoritative and a stored corner is a *cached sample* of it,
and that the first fuse re-derives every corner as the meet of the planes around
it. So the first union is entitled to move a corner. Measured on a unit block
turned a sixty-fourth of a turn: the input cached the far corner's `y` as
`1.0932018670017576`, the exact meet of that corner's three stored planes is
`1.09320186700175740…`, and the fuse stores the `f64` nearest it,
`1.0932018670017574`. **The cached corner is the ulp that was wrong**, and
correcting it is the whole point of re-deriving.

What §17.1 forbids is that correction *repeating* — "the rounding in a vertex is
the rounding of one Cramer solve, never a sum over the operation history" — so
the property now measures from the first union, and every union after it has to
change nothing at all. The claim it used to make was strictly stronger than the
design and would have forced the fuse to prefer a stale corner over a correct
one.

### 19.3 The refusal rate does move, and by how much

§18.7 records a change that was obviously going to improve criterion 8's number
and measurably did not. This is the other outcome, on the same harness and the
same corpus of 414 turned fuses:

| | before | after |
|---|---|---|
| Turned pairs, the headline family | 105 refused = **25.4%** | 87 refused = **21.0%** |
| Not a solid, same family | 16 = 3.9% | 21 = 5.1% |
| The three control rows | 0 refused | 0 refused |

The corpus was held fixed for that measurement: two proptest regression seeds
were added by the failures the widened oracle found, and both figures above are
from before they went in. It is the same sentence §17.7 already wrote about why
that family refuses at all — this corpus is the lattice family *turned*, and the
lattice family is full of configurations that are exactly degenerate, so turning
them leaves two coincident faces whose stored planes disagree in the last bits.
The reader was manufacturing that disagreement. Eighteen of the 105 refusals
were it.

Five more answers are now not-a-solid rather than refusals, which is a real
change of outcome and not a change of shape: those fuses now get far enough to
find out that what they were asked for is not something a `Model` can hold.

### 19.4 A distance is not a length that scales

`InPlane::of` needs a point it can prove is off a face's plane, and stepped
along the normal by `1.0`, then `1024.0`, then `1048576.0`. Past an offset of
about `2^72` the `f64` spacing is wider than the largest of those, every
candidate lands back exactly on the plane, and the fuse refuses a fuse it could
plainly answer. The doc comment on that function spelled out its own
counterexample — "a bigger reach only helps when the plane's own offset is large
enough to swallow a smaller one" — and then hard-coded three reaches anyway.

The reach is a fraction of the plane's own offset now — the only length in sight
that scales with the plane — with a floor of one. **Both halves of that sentence
were measured rather than reasoned, and one of them was a surprise.** Dropping
the floor and stepping off a plane `0.05` from the origin by `0.05` leaves the
reference point so close to the face that the in-plane winding fan cannot be
decided, and the headline refusal rate goes from 20.7% to 37.9%: 72 more fuses
that could not place a point inside a piece of a face. The reference point has
to be off the plane *and* far enough off it to be worth measuring turns against,
and only the first of those is obvious.

The second try is the same distance the other way, and it is what a plane out at
`1e308` needs, where stepping further from the origin overflows and stepping
towards it cannot. Stepping towards the origin *first* was measured as well: it
is the same 37.9%, because a face far from the origin then gets a reference point
a whole offset away from it and the fan degenerates again. **This is why there is
no limit claim here.** The obvious sentence to write was that past about
`f64::MAX` both steps overflow and the point being looked for does not exist —
and the fix family that sentence would have hidden is the one now used as the
second try, which CLAUDE.md's rule about impossibility claims exists to make
somebody go and look for.

`signed_volume` and `measure` hung their tetrahedra from the world origin, so a
solid `2^m` out summed terms of size `2^{3m}` to reach a volume of order one and
lost every digit past `m = 28`. They hang from a corner of the solid now, which
makes the three subtractions exact by Sterbenz's lemma whenever the coordinates
share an exponent — which is exactly the case that was worst. **The corner is
the smallest one and not the first the arena hands over**, and that distinction
is not fussiness: two routes through a fuse reach the same solid with its faces
in different orders, and the first version of this fix made the centroid depend
on which came first. The associativity property caught it inside one run.

**Where the ranges end now, all measured.** A cube fuses with a copy of itself
out to `2^340`; at `2^341` six times its volume is past the largest `f64`. A
`3 × 3` frame encloses exactly eight out to `2^50`; past `2^53` a wall one unit
thick has both its corners on the same number and there is no frame to fuse.
Both ends are limits — the answer stops existing in the number type — and both
have a test that says which. `a_pair_somewhere_else` reached `2^30` because of
these two gaps and now reaches `2^280` against a measured edge of `2^339`, with
its shift raised from `2^24` to the `2^30` its own doc always said the dynamic
range allowed. §17.7's four measured rows do not move when it is widened.

### 19.5 Three claims that fell out with the fixes

Each was found by fixing something else, and each is the §16.10 shape: an
argument that was locally valid about a subtly different question.

**Criterion 7's demonstration was demonstrating the bug.**
`a_fuse_that_needs_a_decision_nothing_can_prove_refuses` built two towers `1e200`
tall and said "nothing in `f64` can multiply coordinates that far apart without
overflowing, so a decision the answer depends on cannot be proved". The fuse
never reached a predicate: `InPlane::of` could not step off the towers' top
plane. They fuse now, and enclose `1.75e200` as they should. The demonstration
moves to `1e308`, where `orient3d` genuinely overflows and the records say so —
and the test now also asserts that no record names `"a point off a face's
plane"`, so the demonstration cannot quietly become a bug again. §15.8's
distinction between a refusal and a *deserved* refusal, applied to a test rather
than to a rate.

**`check_dynamic_range` could not see where a solid is.** It read
`vertex.point` straight out of the arena, and a solid placed a long way out
keeps ordinary small numbers there and carries the distance in its location. So
a frame at `2^39` passed the check, and the same geometry failed it after a
fuse, because a fuse bakes placement in (§17.3) — the same model, two answers,
decided by where the placement happened to be written down. What the check is
about is the numbers the predicates see, and those are located. This is §16.11
again and worse than §16.11 thought: the ratio could not see absolute magnitude,
and it could not see the magnitude either.

**A debug sweep was blaming the fuse for what it was handed.**
`Assembly::commit` runs `Model::validate` over the whole model after writing and
panics on failure. The sweep covers parts of the model no fuse looked at, and
§4.3 is explicit that a caller may hold a model that does not pass it, since the
sweep runs at test and import boundaries and not after every mutation. So a
caller with one out-of-range solid got a panic from an unrelated fuse. It now
validates before writing too and asserts only when the model arrived sound,
which is what the panic message always claimed: *a fuse writes a valid model or
does not write at all*.

### 19.6 What this does not fix

- **The lower end of the scale range is still a limit and still there.** Below
  about `2^-359` unshifted, `orient3d`'s products fall subnormal, level 1
  declines by design and expansion arithmetic will not certify a product it
  cannot represent. §12.2's precondition, arriving end to end, and the honest
  answer.
- **`DynamicRange` is still a ratio.** It now measures the right coordinates,
  but §16.11's question — that a ratio cannot see absolute magnitude — is
  answered only in the sense that the two things it broke are fixed. A model at
  `2^300` with features at `2^290` still reports a comfortable ratio.
- **Nothing here widens the shape vocabulary.** §17.8 measured that three
  million fuses over the existing generators found nothing and one new shape
  family found a defect in forty; that is still the best-value work available,
  and generator families 3 to 6 are still unbuilt.

---

## 20. One assert, three answers, and two of them were never checked

*Recorded 2026-08-31.*

`SolidView::measure` returns volume, surface area and centroid. It had one
check:

```rust
assert!(six_volumes.is_finite() && six_volumes > 0.0, ...);
```

Two things are wrong with that line and they are unrelated to each other.

### 20.1 The assert was doing two jobs and only one was its own

The justification written above it says a closed solid with its faces pointing
outwards cannot enclose non-positive space, "which makes this a broken model
rather than bad input". That is true, and it is about **geometry**. The check is
about **arithmetic**. A cube of side `5e102` points its faces outwards perfectly
well; what it did was overflow. So:

- **not finite** — the sum ran out of room. Bad input, and bad input is data
  rather than a bug (§4.1). Return `Err`.
- **finite and not positive** — the faces point inwards. A corrupt solid, and a
  crash beats a corruption. Still panics.

Measured before the fix, all three of these built and then panicked:
`add_cube(5e102)`, `add_cube(1e140)`, `add_cube(1e200)`. Same shape as the
`unreachable!` in `ring_runs_anticlockwise` that §16.11 found: a claim about
what geometry can do, standing guard over what `f64` can hold.

### 20.2 The worse half: two answers nobody was checking at all

`six_volumes` is the **last** of the three quantities to overflow, and it was
the only one guarded. Measured across the range, on cubes:

| side | what came back |
|---|---|
| `2^256` (`1.16e77`) and up | volume fine, `surface_area` **infinity**, centroid **infinity on all three axes** — returned, no complaint |
| `2^341` (`4.5e102`) and up | `six_volumes` overflows, panic |

That is 85 binades wide. In all of it, `measure` handed back a `Measurement`
with a real volume and an infinite centroid and said nothing. The assert existed
to stop exactly that, and it was watching the one field that survived longest.

Both crossovers are exactly powers of two and both mechanisms are plain:
`Vector::length` is `sqrt(x² + y² + z²)`, so it squares components of size
`side²` and dies at `side⁴ = 2^1024`; the moment sum accumulates
`six_volume × offset ≈ side⁴` and dies at the same place.

**And both are §16.10 gaps, not limits.** A `1e100` cube's surface area is
`6e200` and its centroid is `5e99`. Both are ordinary `f64` numbers. Only the
arithmetic on the way to them overflowed — which is the definition.

### 20.3 The fix: scale by what each quantity is invariant under

§18's technique, applied three times with the scale chosen by what is being
measured. Every factor is an exact power of two, so every sum comes back bit for
bit what it would have been, and the whole change is invisible to models that
were already in range — a claim the existing exact-equality and
`changes_not_one_bit` tests check on every run.

- **Volume and centroid: one power of two per axis.** A 3×3 determinant has one
  entry from each column in every term, so scaling a column scales the
  determinant by exactly that factor. The moment inherits it.
- **Surface area: one power of two per face.** A length does *not* come apart
  that way — `x² + y² + z²` only factors if all three were scaled alike — so
  each face is measured whole. **Per face and not per solid**, because a tower
  one unit across and `1e200` tall scaled to its own height would push its top
  and bottom faces to `1e-400`, and the two units of area they carry would
  silently vanish.
- **Undo the scaling in steps, never in one multiplication.** `2^exponent` need
  not be representable when the answer is. Every step moves the same way, so the
  running value walks towards the answer and never past it, and an overflow
  means what it should.

Refusals are now `Error::VolumeTooLargeToMeasure` and
`Error::SurfaceAreaTooLargeToMeasure`. They are separate because area and volume
grow at different rates: a wide thin slab can run out of room for its area while
its volume still fits.

### 20.4 `validate` was calling a sound model broken

`Model::validate` and `fuse/assemble.rs` shared the same conflated test and so
reported `Invalid::VolumeIsNotPositive { volume: inf }` for a `1e300` cube.
§16.11's closing paragraph had already noticed and filed it as *"the protection
was always accidental"*. It is not a broken model — the same cube classifies
points `Inside` with an empty log, which is the kernel certifying that it works.
`Invalid` now means only what its name says, and too-large is its own error.

### 20.5 A published bound that was the wrong side of §16.10

`a_solid_a_long_way_from_the_origin.rs` said the cube range ends at `2^341`, and
gave the reason:

> a cube that big encloses `2^1023` of space, six times which is past the
> largest `f64`

That sentence fails §16.10's test on its own evidence. `2^1023` is a perfectly
ordinary `f64`. Only the `6 ×` on the way to it was not, because the sum was
scaled back in one multiplication. The answer existed and the arithmetic could
not reach it — a **gap**, written down two lines beneath a sentence calling it a
limit, and pinned by a test asserting the refusal so it could not drift back.
§15.6 says a published bound needs a test; this is the case where the test held
the wrong bound in place.

The range is now one binade wider. `2^341` measures — volume `2^1023`, area
`6 × 2^682`, centroid at half the side, all exact — and `2^342` refuses, because
`2^1026` genuinely is not a number `f64` has. That end is real.

### 20.6 The regression this caused, and what it taught

The first fix used **one** scale for the whole solid, taken from its largest
offset. `just` went from one failure to a different one: two towers a unit
across and `1e200` tall now fused to a volume of exactly zero. Scaling by the
tallest axis put the two short ones at `2^-664`, and their product with the
third is `2^-1328` — nothing.

Worth keeping because the fix and the bug come from the same place. A uniform
scale is right for a *shape*, and a solid is not obliged to be one size in all
three directions. The dynamic-range check would not have warned: the tower's
ratio is `1e200`, wildly past `LARGEST_SUPPORTED_RATIO` of `1e11`, and the test
asserting it works has been passing all along. §16.11's inversion again — the
ratio check is blind to the case that actually breaks the arithmetic.

### 20.7 Corrections owed to earlier sections

Measured at `898ac58` and recorded rather than quietly edited, which is what
§16.10 is for.

1. **§16.11's "the large end is a different bug" is stale.** `1e140`, `1e200`
   and `1e300` cubes all classify `Inside` with an **empty log**. Fixed by
   `a23d73d`'s rescaling, before this work started.
2. **The cause §16.11 recorded for it was wrong.** It blames the ray target
   from the bounding box overflowing. That does not overflow until side
   `7.7e307`; at `1e140` the overflow was inside `orient3d`. The attached
   hypothesis — that a ray target relative to the bounding box "would likely
   just work" — proposed a fix for a problem that was not there.
3. **The same wrong cause is repeated in `classify.rs`**, claiming a `1e140`
   cube returns `Undecidable` with thirty-six records. It returns `Inside` with
   none.

---

## 21. A gate named for a check it only half did

*Recorded 2026-08-31.*

`product_is_exact(a, b)` answers whether [`two_product`] on `a` and `b` is
exact. It was `true` for `1e200` squared.

Its doc was accurate — it described the subnormal precondition and nothing else,
and that is genuinely what it tested. The **name** is the problem. At
twenty-seven call sites it reads as *the* gate that keeps unrepresentable
products out of the expansion arithmetic, and it only ever guarded the bottom of
the range. It now rejects overflow too.

### 21.1 Nothing downstream changes, and that was checked rather than hoped

The prediction going in was that no outcome moves, because an overflowed product
reaches `expansion_sign`, which refuses to name a sign for a non-finite
expansion. A prediction is not a measurement, so each site was checked:

Every guarded call site sits in an `exactly()` — level 3 of the cascade — and
every one of them maps `Decision::Undecidable` to `None`: `orient2d`,
`orient3d`, `plane`, `sphere`, `cylinder`, `surface`, and
`validate.rs`'s `AreaSweptRoundTheNormal`. The cascade turns that `None` into
`Undecidable { enclosure: straddle }` at `Level::Interval`. So both routes to
the refusal — the gate declining early, and the arithmetic running and the sign
refusing — converge on the same `None` and produce the same `Decision`.

**The cascade would have caught it if they did not.** `decide` treats
`Some(Decision::Undecidable { .. })` from level 3 as `unreachable!`, because
exact arithmetic must certify a sign or say it cannot run. A site that mapped an
overflow to `Undecidable` instead of `None` would panic, not drift.

`de_boor`'s `quotient_is_exact` is the one gate that is not an `exactly()`. It
returns `false` on a failed gate, and with an overflowed product it returned
`false` anyway, because `two_product` hands back an infinity and the `high ==
top` comparison against a finite `top` fails. Same answer, one step sooner.

740 tests pass, including §17.7's refusal-rate bound, which is the oracle that
would notice a rate moving the wrong way.

### 21.2 The zero test is on the arguments, and has to be

The obvious tightening — read the zero off the product rather than off the
arguments — is wrong, and quietly. `1e-200` squared underflows to zero with
neither argument zero, and calling that exact would open the **subnormal** end of
the gate while closing the overflow end. That end is the one `robust` gets wrong
and returns a confident wrong sign for. The two ends are separate tests and stay
separate.

### 21.3 Two claims carried into this work that did not reproduce

Both came from §18.6 and were checked at `898ac58` before being acted on.

1. **"Two call sites bypass the gate entirely and lean on `expansion_sign`
   alone."** They do not. Every use of `two_product` or `scale_expansion`
   outside `expansion.rs` itself is preceded by a `product_is_exact` in the same
   function — `validate.rs`'s path, named as one of the two, gates both its
   minor and its scaling. Nothing was left to add, so the "legibility, not
   correctness" change this predicted has no subject.
2. **"The zero-elimination invariant is documented on `two_diff` and needs tests
   at the two functions that provide it."** It has them.
   `a_non_finite_entry_cannot_be_dropped_on_its_way_to_a_sign` already runs an
   overflowed pair through `expansion_sum` *and* through `scale_expansion`, and
   through the scale-by-zero case specifically — which is the subtle one, since
   a short-circuit on a zero factor is what would break the backstop. That work
   was already done.

Recorded because a plan is a claim like any other, and §16.10 does not exempt
the ones written a day earlier.
---

## 22. A pair that is not square to itself

*Recorded 2026-08-31.* §17.8 priced a new shape family at about fifty thousand
to one against more of the old corpus, and §19.6 carried "nothing here widens
the shape vocabulary" forward as the best-value work available. This widens it by
one family and one control. The family is the cheapest thing that changes the
**arrangement** rather than the numbers: one whole-number shear per solid, no new
outline, no new arithmetic on the way in.

### 22.1 What the corpus structurally could not draw

Every configuration family that existed before this one turns **both** solids by
the same rotation. `a_turned_pair` rotates the pair; `a_pair_somewhere_else`
scales and shifts it; `a_flush_pair`, `a_chain_of_fuses` and the rest leave it on
the axes. So however irrational the world coordinates get, the pair stays exactly
as square to *itself* as the lattice configuration it came from, and both solids'
wall planes have normals drawn from one orthonormal triple.

That has a consequence nobody had written down. Take a face whose normal `n` is
from that triple. Every line cut into it — where another plane meets it, and its
own boundary — runs along `n × m` for some other `m` in the triple, and there are
only **two** such directions. Three distinct lines through one point need three
distinct directions, because two distinct lines sharing a direction are parallel
and meet nowhere. So:

**No configuration in the corpus has ever put three concurrent cut lines on a
face.** Not on any seed, not at any scale, not under any rotation. It was not
rare; the generators could not express it.

A face's arrangement has to put the intersections it finds in order along a
line. Where two of them are the same point, that order is a **tie** rather than a
comparison, and no amount of precision settles a tie. That is the branch the
suite had never reached.

### 22.2 The family, and what stays exact

`a_skew_pair` leans each solid over by its own whole number: a corner at
`(x, y)` moves to `(x + a·y, y)`, with `a` drawn from `-2..=2` per solid and
shrinking toward `0`, which is the lattice configuration the case came from.

As a matrix that is `[[1, a], [0, 1]]`, determinant `1`, and the determinant does
three jobs at once. An affine map takes a simple polygon to a simple polygon and
a positive determinant keeps it anticlockwise, so a leaning `Shape` never needs
checking and never gets filtered — every draw builds. Whole numbers go to whole
numbers, so every corner reaching `ProfileBuilder` is exact. And the volume does
not change, so a leaning pair encloses what its lattice original enclosed.

What is **not** exact, and provably cannot be, is the tilted walls.
`build_side_face` takes a wall normal as `(along_y, -along_x, 0)` and
`Plane::from_point_and_normal` normalizes it. The only exactly representable unit
normals are the signed axis directions, and that is a proof rather than an
observation: an `f64` is a dyadic rational, so a unit normal with `f64`
components clears to whole numbers with `p₁² + p₂² + p₃² = 4^k`; squares are `0`
or `1` mod `4`, so all three must be even; so every solution descends to one of
`4^{k-1}`, and the descent bottoms out at `1 = 1 + 0 + 0`. Only `(±2^k, 0, 0)`
and its permutations survive.

So this family sits between the two kinds the suite already had. **Corners exact,
caps exact, tilted walls rounded.** `a_lattice_pair` has everything exact;
`a_turned_pair` has nothing exact. Nothing had ever separated the two effects.

Because of that it runs only what is exactly true whatever the coefficients are:
the two ways round a question agreeing bit for bit, and every plane of an answer
being a plane of an input. Inclusion and exclusion is not asserted, for the
reason §17.8 gives for turned pairs. Neither is transform invariance, and that
one is specific here — moving a plane slides it along its own normal by `n · t`,
exact when `n` is a signed unit axis and a rounding otherwise, so the property
would be comparing the generator's arithmetic against the kernel's rather than
the kernel against itself.

### 22.3 The control, and the ordering that nearly made it a lie

§17.7 settles that families are reported separately and never as one number, and
that the control is the row that earns its keep. The control here is
`a_pair_leaning_the_same_way`: **both** solids leaned by the same amount, which
is one affine map `S` of the whole lattice pair — same shapes, same overlaps,
same touchings, the same cut lines on every face in the same order. The lattice
family's claim is that a certified policy must decide all of that, and it refuses
zero. The one thing left between the two rows is that the walls are tilted.

**That claim was false when it was first written, and the code was what was
wrong.** The shear was being applied to the outline's own corners and the
placement added afterwards, which is `S(ring) + at` — the image of nothing. An
affine image of the pair is `S(ring + at) = S(ring) + S(at)`, so the shear has to
go on *after* the placement. Leaning the solid where it stands, not leaning it
and then standing it somewhere.

It was caught by a number that did not add up, not by reading the code. An affine
map preserves incidence exactly, so a control that is an affine image of the
lattice family must produce not-a-solid answers at about the lattice family's
rate — an answer is not a solid here almost always because two solids meet along
a whole edge and four faces would run along it. The control was reporting 1.9%
against the lattice family's 4.8%. With the ordering corrected it reports 3.1%,
and the remaining gap is the two caveats the file already carries for
`A_PAIR_ELSEWHERE_IS_NOT_A_SOLID`: this family draws one more number per case, so
it is not the same 128 configurations, and a fuse that refuses never gets far
enough to find out whether its answer was a solid.

Worth recording as its own paragraph because of what the wrong version did. It
built, it ran, it shrank properly, it reported a plausible number, and every
sentence written about it was wrong. A control that does not control is worse
than no control, because the subtraction still produces a figure.

### 22.4 The measurement

| Family | Corpus | Refused | Rate | Where |
|---|---|---|---|---|
| Lattice pairs, for reference | 420 fuses | 0 | 0% | `every_certified_fuse_of_a_lattice_pair_is_decided_and_records_nothing` |
| Both solids leaning **alike** | 420 fuses | 30 | **7.1%** | `how_often_a_certified_fuse_of_a_pair_leaning_the_same_way_refuses` |
| The two leaning **differently** | 420 fuses | 60 | **14.3%** | `how_often_a_certified_fuse_of_a_skew_pair_refuses` |

So the subtraction the control exists for: **tilting the walls costs 7.1 points
with the arrangement held fixed, and leaning the two solids differently costs
7.2 more.** Neither number was available before, and the headline alone would
have been the two added together with no way to tell which was which.

The threshold on the headline is **18%**, asserted by the test named above and
judged on the shipped corpus only, for the reason §17.7 gives about applying one
corpus's threshold to another. Fresh seeds at ten times the corpus come out at
15.5% and 15.6% for the headline and 7.7%, 7.8% and 8.0% for the control.

The breakdown is the half anybody can act on, and here it is the claim rather
than a footnote.

| What could not be decided | Leaning alike | Leaning differently |
|---|---|---|
| `"order along a line"` | 9 | **30** |
| `"side of plane"` | 18 | 21 |
| `"a point inside a piece of a face"` | 3 | 9 |

§17.7 records that every refusal the turned family ever produced named
`"side of plane"`, over fourteen seeds, with `"a point inside a piece of a face"`
turning up rarely and **no run ever naming a third cause**. Leaning the two
solids differently does not merely refuse more; it refuses for a *different
reason*, and that reason becomes the leading one. `"side of plane"` barely moves
between the two rows — 18 against 21 — while `"order along a line"` more than
triples. At soak scale the same thing is sharper and steadier: `"side of plane"`
runs at 5.1–6.4% of fuses in both rows, and `"order along a line"` goes from
1.2–2.2% in the control to 9.0–9.4% in the headline.

That is the arrangement, isolated. It is asserted rather than only printed:
`Tally::the_cause_named_most_often_is` fails the headline row if
`"order along a line"` ever stops being what these fuses mostly refuse on, since
at that point the family has quietly become its own control.

Two cautions, because the mechanism is tempting and is not proved here. The
control names `"order along a line"` nine times too, so concurrency is not the
only way to reach it and this section does not claim it is. And what the rate
says about real models is what §17.7 already says about its own: measured over a
corpus steered into coincidence, it bounds how bad the answer can be on
purpose-built input, and is evidence rather than a verdict.

### 22.5 The second slide was written, measured and taken out

The shear started as two slides — `x` by so much per step of `y`, then `y` by so
much per step of `x`, composing to `[[1, a], [b, 1 + ab]]`, still determinant
one. The argument for it was structural and sounded right: one slide leaves every
solid a wall along `(1, 0)`, so two solids always keep one pair of mutually
parallel wall directions, and only the second slide lets a pair share no wall
direction at all.

Measured, it does not do that. It squeezes. A grid of refusal rates over every
pair of leans settles which effect the number reports:

| first \ second | `[0,0]` | `[1,0]` | `[2,0]` | `[0,1]` | `[1,1]` | `[2,2]` |
|---|---|---|---|---|---|---|
| `[0,0]` | **0.0%** | 8.8% | 14.4% | 11.2% | 25.0% | 45.6% |
| `[1,0]` | 6.9% | 4.4% | 18.1% | 16.9% | 24.4% | 44.4% |
| `[2,0]` | 13.1% | 16.9% | 10.6% | 23.8% | 25.0% | 43.8% |
| `[0,1]` | 13.8% | 15.0% | 18.8% | 16.9% | 30.6% | 48.8% |
| `[1,1]` | 38.1% | 36.2% | 34.4% | 43.8% | **36.2%** | 56.9% |
| `[2,2]` | 61.2% | 61.2% | 58.1% | 66.9% | 64.4% | **76.9%** |

The diagonal is the control — both solids leaning alike, an affine image of a
lattice pair, no new arrangement anywhere. A solid leaned `[1, 1]` refuses 38.1%
against a flat partner and **36.2% against a partner leaning identically**, which
is the same number. The rate follows the lean of the single solid and not the
relation between the two. At `a = b = 2` the composition squeezes 34 to 1 against
5.83 to 1 for one slide of two, and the headline the two slides bought was 46.4%
with `"where three planes meet"` leading it — which is what a sliver looks like,
not what an arrangement looks like.

With one slide that cause disappears from the breakdown entirely.

Recorded because the argument for the second slide was a good one and it was
wrong, and only the grid could tell. §16.10's shape again: a locally valid
argument answering a subtly different question than the measurement asks. Two
of those in one section, counting §22.3.

### 22.6 The family found a defect on its first full run

`swapping_the_two_sides_of_a_skew_pair` failed the second time the whole suite
ran, and the cause is one line of arithmetic that every family before this one
was safe from by luck.

`Plane::from_point_and_normal` normalizes the vector it is handed, so what it
stores depends on that vector's **length** and not only on its direction:

```text
normal (1, 1, 0)  ->  0.7071067811865475
normal (2, 2, 0)  ->  0.7071067811865475
normal (3, 3, 0)  ->  0.7071067811865476
normal (6, 6, 0)  ->  0.7071067811865476
```

`build_side_face` takes a wall normal as `(along_y, -along_x, 0)` straight off
the profile, so the length is whatever that outline's edge happened to be. **Two
walls that are the same plane get coefficients an ulp apart when they stand on
edges of different lengths.** §17.1 makes a face *be* its plane, so those are two
different planes to this kernel, and a fuse over such a pair keeps whichever
belonged to the argument — which makes the answer depend on which side is which.
An Ell and a Comb, both leaned by `-1`, both at the origin, intersected: same
solid, same twelve faces, same corners to the bit, and one face on a plane
differing in the last place.

Every rectilinear family is safe from this and none of them is safe *by design*.
An axis-aligned normal of any length normalizes exactly — dividing `(0, 3, 0)` by
`3` is exact — and the turned families rotate both solids by one isometry, so
coplanar walls stay bit-identical however long their edges. Tilting a wall is
what breaks it, and until now nothing here could tilt one.

**A gap and not a limit** (§16.10). The answer exists and the arithmetic can
reach it: dividing the normal by its largest-magnitude component before
normalizing makes the result depend on the ratios alone, and the ratios of two
scalar multiples are the same real number and so round the same way. That is a
change to every plane this kernel stores, which is why it is recorded here and
not made here.

Two things stand in for the fix. `the_same_wall_built_from_a_longer_edge_is_a_different_plane`
pins the arithmetic and is written the way §19 writes them — it asserts what the
code does, so fixing the defect fails it loudly with the fix in hand. And the
commutativity property over skew pairs compares **the corners only**, which is
not much of a retreat: the partition is what §17.4's argument is about, a
splitter that ordered its cuts differently would move a corner, and what is given
up is the last bit of a coefficient nobody disagrees about geometrically. When
the plane is canonical the comparison goes back to every last bit, and the diff
is one word.

Worth saying plainly, because §17.8 predicted it and put a price on it: **the
family found this on its first full run of the suite, after three million fuses
over the old generators had found nothing.**

### 22.7 What this does not settle

- **§22.6's plane defect is recorded and not fixed.** It is a gap, the fix is
  named, and it touches every plane in the kernel. *(Fixed at the opening of
  M2, by exactly the fix named there — §23.8 item 1. Item 2 beside it is the
  second defect on the same line that this section could not see: a normal near
  `f64::MAX` overflowed its own length and came back as a direction of length
  zero. `swapping_the_two_sides_of_a_skew_pair` compares every last bit now and
  `HowMuchMustAgree` went with it, which is the one-word diff §22.6 predicted.)*
- **Whether the `"order along a line"` refusals are deserved.** §14.7's worry
  applies here exactly as it does to the turned family, and a rate cannot tell a
  timid filter from a genuine tie. What would tell is a hand-written pair with
  three cut lines provably through one point, asked whether the two orderings
  give the same solid. That is the next piece of work this family opens, and it
  is not done here. The family's headline is 14.3% against the turned family's
  21.0%, so if any of it is undeserved it is worth about as much as the defects
  §19 fixed.
- **The concurrency claim is an argument about the generators, not a test.**
  §22.1 proves the old families cannot reach three concurrent lines and the proof
  is short, but nothing asserts that the new family *does* reach it. What stands
  in for it is the refusal breakdown, which is a consequence and not the thing
  itself. §15.6's rule about published bounds needing tests points the same way
  here.
- **Nothing here leans a cap.** `add_extrusion` extrudes along `z`, so the top
  and bottom faces of both solids stay parallel to each other whatever the
  outlines do. An oblique prism needs a different builder. §17.8's generator
  families 4 to 6 — `a_hollow_pair`, `a_perforated_plate`,
  `a_pair_that_must_be_refused` — are still unbuilt.

---

## 23. M2 — curved topology and tessellation

*Recorded 2026-08-31, as the milestone's gate closed. §15 did this for M1 and
M2.5; this is the same for M2, and it is written before either lane fans out
because §13 rule 4 says a meaning two people settle separately gets settled
twice, differently.*

### 23.1 What M2 is, now that the roadmap's version of it is three-quarters done

Roadmap §12 says *"Topology + tessellation. Build a solid by hand, validate it,
mesh it, render it."* That sentence predates §1.7's insertion of M2.5. For
planar solids, build, validate and a triangle fan all shipped in M0, and a
general fuse shipped over them — so M2 as literally written is an exporter and
a week.

What is genuinely unbuilt is the **seam between M1's geometry and M0's
topology**. `FaceData` held a `Surface` and only `as_plane` was ever read;
`EdgeData` was two vertex slots; there was no curved face, no seam and no pole
anywhere. M3 cannot intersect surfaces that topology cannot hold, so that seam
is M2's content and tessellation is the consumer that makes it buildable under
§13 rule 3.

Curved surfaces in topology are **cylinder and sphere only** — the two whose
sign predicates shipped in M0 §10 criterion 6. A cone brings an apex, which is a
pole and a curvature singularity in one entity, and a torus brings a second
periodic direction; neither has a predicate and neither has a consumer.

### 23.2 An edge is named by the surfaces it runs along, and never by a curve

The same argument §15.9 makes for a pair of planes naming their line, one layer
up. An edge stores its two ends and an `EdgeRun` saying *which* thing the
surfaces around it describe. It does not store a curve, because a curve would
be a second authority sitting beside the faces' own surfaces, and the first ulp
of disagreement between them would have no tiebreak — which is §17.1's argument
about planes, arriving for curves.

Three cases, and the third is the one that forced the design:

- **Straight.** The two ends are the whole of it. A cylinder's seam ruling is
  this, which is why a round bar needs nothing cleverer.
- **Right round where its faces cross.** A cap circle of a bar: the cylinder
  crosses the cap's plane in one circle and the edge is all of it. Nothing to
  disambiguate.
- **Part of where it crosses something else.** A ball's meridian seam. Both its
  sides are the *same* sphere, so the faces name nothing between them; and a
  plane crosses a sphere in one circle, which two points cut into **two** arcs
  with the same two ends. So naming the crossing and naming the ends still names
  two things. What settles it is a list of half-spaces the edge lies in — each
  exact, each answering with a `Decision`, and no constructed witness point
  anywhere, which would have been §15.5's mistake arriving by a new road.

`validate` reads all of it. An edge named by a crossing has to sit on both
surfaces of that crossing and inside every half-space that picked it, or the
name describes a curve the edge is not on and nothing else in the kernel would
ever notice.

### 23.3 A pole is its own kind of wire step

A wire is a chain of steps, and a step is either along a coedge or standing
still at a pole. A ball's one wire is four steps of which two go nowhere.

OCCT's answer is a degenerate edge — zero-long in space, full-length in
`(u, v)`. **Rejected**, because roadmap §7.4 lists zero-length edges in the
degeneracy catalogue as something to detect and refuse, and overloading that
shape to also mean "fine by design" makes the catalogue entry unaskable: the
check can no longer tell the good kind from the broken kind. Here a zero-length
edge still means broken and nothing else.

### 23.4 Watertightness is a claim about 3D points and nothing else

Edge points are worked out **once** and the same 3D points are handed to both
faces along that edge. Each face is free to disagree about where those points
sit in *its* `(u, v)`, and for a cylinder it must, because the inverse
parameterization is an `atan2`.

That reads like a defect until you see that `(u, v)` is a working space and
never an answer: the triangulation uses it to decide connectivity, and the
coordinate that reaches the mesh is the shared one, bit for bit. The oracle is
combinatorial rather than metric — every mesh edge used by exactly two triangles
with opposite orientation, and χ of the mesh equal to χ of the solid — which is
a sharper claim than "the gaps are small".

### 23.5 Three knobs meet at a mesh and they are not the same knob

- **`Deflection`** decides how fine the mesh is. A coarse mesh is a request,
  not a failure, and nothing judges it. It travels as an argument and is never a
  field on a model, for §2.6's reason.
- **`Policy`** decides what happens when the deviation cannot be *proven* under
  that deflection: certified refuses, tolerant tags.
- **The model's linear tolerance** is about the geometry and has nothing to say
  about triangles. `Deflection`'s default is a thousandth of the nominal size,
  six orders coarser, and a test states the gap so it cannot quietly close.

Conflating the first two is the tempting mistake, and it would mean a caller who
asked for a rougher picture had also silently asked the kernel to certify less.

**The units, which is where §12.4 says this kernel loses money.** A mesh's
deviation is a **length** and goes in an `Approximation`, per §15.2. A volume
error is not a length and must not be crammed into one; when `measure` moves
onto the mesh it reports volume and area errors as separate named quantities in
their own units, and what gets judged against `Policy::max_wobble` is the
length. `Measures::wobble_of` already exists to make numbers from different
predicates comparable, and this is the same discipline one layer up.

### 23.6 The crate boundary caught a requirement that could not be met as written

`rcad-mesh` sits **between** `rcad-geom` and `rcad-topo` and cannot see
topology. That is what gives it an oracle needing no solid — a polygon in,
triangles out — which is §13 rule 1's condition for a piece that can be worked
on beside the rest instead of after it. The dependency list in its `Cargo.toml`
makes it a fact the compiler checks.

Planning this milestone also settled that a mesh triangle carries the `FaceKey`
it came from, for provenance and for grouping in an exported file. **Those two
cannot both hold as stated**: `FaceKey` lives in `rcad-topo`, above this crate,
so a triangle here naming one is a dependency cycle.

The shape that meets both is two levels. `rcad-mesh` triangulates one trimmed
patch and returns topology-free triangles; `rcad-topo` pairs them with the key
it asked about. Recorded because it is the second time in this milestone that
two separately-settled decisions turned out to be jointly impossible, and both
times the thing that found it was writing the contract down rather than
reasoning about it.

### 23.7 Where the schedule risk is

Not the surface-pair edge representation, which is the part that looks hard.
**`measure`** — shipped in M0, benchmarked, exact, bit-for-bit reproducible, and
already fixed twice in §19 and §20. Rebuilding it on an approximate mesh
produced by a third-party triangulator puts every metamorphic property in the
fuse suite downstream of new arithmetic: inclusion–exclusion, idempotence,
absorption, all of them read its output.

§12.1 and §17.8 both say the risk is the part that looks done, and §17.6 was
right against §1.7 for exactly this reason. This is the third time the
prediction has been made and the first two were both correct.

### 23.8 What the gate found while closing

Four defects, none predicted, all of them silent wrong answers rather than
crashes or refusals. They are recorded together because the pattern is one
thing: **each was a helper that meant one thing until a new kind of input
arrived, and then quietly meant another.**

1. **A plane was the vector that named it, not the direction it faced.** §22.6
   recorded this and left it. Normalizing depends on the length of the vector
   handed over, so `(1, 1, 0)` and `(3, 3, 0)` — the same plane — came out an
   ulp apart, and a kernel where a face *is* its plane then had two planes.
   Fixed by dividing by the largest-magnitude component first, so what is stored
   depends on the ratios alone.
2. **And the same line had a second defect nobody had recorded.** A normal near
   `f64::MAX` overflowed its own length, and `normalized` returned a direction
   of length **zero** — not a refusal. Dividing first closes it, and closes a
   subnormal refusal at the other end. Both are §16.10 gaps.
3. **An edge's length was the distance between its two ends.** True until a
   closed edge existed, whose two ends are one corner: every round bar then
   reported a smallest feature of exactly zero and the dynamic-range check
   refused it. Renamed to `distance_between_its_ends`, which cannot be misread.
4. **`FaceView::plane` claimed `validate` made it unreachable.** That stopped
   being true the moment a curved face was legal, and the fuse called it in
   three places, so `Model::fuse` on a round bar panicked. Worse, the classifier
   did not panic: it silently answered as though the bar were the handful of
   chords its corners span, which §4.1 ranks below the crash. Both now refuse
   through one shared `SolidView::is_wholly_flat`, asked over every input before
   anything is walked — so which answer comes back cannot depend on the order
   the faces were reached, which is what the commutativity properties compare
   across.

Item 4 is §17.8's last paragraph a third time: a comment explaining why
something is unreachable is a claim, and this one named the check that had
stopped doing the job.

### 23.9 Gaps carried into the lanes, named rather than skipped

- **Nothing says a round cap faces outwards.** A one-corner wire has no polygon
  to wind, and which way a circle goes is a fact about the circle rather than
  about its corners. The oracle is the mesh, where a tessellated cap does have a
  polygon.
- **Nothing measures a curved solid's volume.** `signed_volume` sums over
  triangles and a curved solid is not a polyhedron, so the check is skipped
  rather than run against a number about a shape the solid is not. Same oracle,
  same lane.
- **Arcs in a profile are their own piece.** Planning assumed they were the
  cheap route to a cylindrical face. They are not: every 2D self-intersection
  check in `build.rs` would have to become arc-aware. A round bar reaches the
  seam, the curved face and the closed edge with none of that, which is why it
  went first.

---

## 24. M3 — where two surfaces cross

*Recorded 2026-08-31, before either stage opens. §15 settled the shared meanings
before M1 and M2.5 fanned out and §23 did it for M2; this is the same for M3,
for §13 rule 4's reason.*

### 24.1 What M3 is, and where the free-form half went

Roadmap §12 says *"Intersection. Analytic quadric cases, then marching with
certification."* Both halves, one milestone. The second half has no consumer
here: `Surface` holds a plane, a sphere and a cylinder, because §23.1 kept it to
the curved kinds that have sign predicates. A marcher over B-spline surfaces
would cross surfaces that topology cannot hold and the fuse cannot classify —
§13 rule 3 in its largest form.

**M3 is the analytic pairs, plus a marcher run over those same analytic
surfaces.** The marcher is built where the answer is already known in closed
form, because that is the only setting where its tangency and near-tangency
branches have a ground truth to be judged against. Roadmap §6.3 says
near-tangential contact is where kernels produce silent garbage, and a marcher
whose only oracle is itself cannot watch that happen. This is §2.2's "built, not
wrapped" argument one layer up.

One objection deserves an answer rather than silence. §15.7 says the
`Approximation` field — how far a computed curve strays from the true one — is
zero for planes and that "M3's marcher is its first real producer". Analytic
pairs also cross in exactly named curves, so on scope alone that field would sit
unproduced through a third milestone, which is §12.6's first gap in new clothes.
The answer is §24.3: the fidelity does not belong to the crossing. It belongs to
*evaluating points along* the crossing, and that has a producer here.

### 24.2 The consumer is the mesher, and it is not the fuse

§13 rule 3 decides whether M3 is a milestone at all. The intersector's headline
consumer is a fuse over curved solids, and that is M4; read strictly, the rule
folds M3 into M4 and leaves one large milestone with no gate in the middle —
the wrong shape under rule 6, where review and not authoring is the constraint.

**The consumer inside M3 is the mesher, with `validate` behind it.** M2 builds a
round bar by declaring its edges; M3 computes the same bar's cap circle from the
cylinder and the cap's plane, and then reaches a shape M2 could not declare at
all — a cylinder cut by a tilted plane, whose edge is an ellipse. The mesh is
the oracle, and it is one that already exists rather than one this milestone has
to invent, which is rule 1's condition.

### 24.3 A crossing is named, and never carried

§23.2 settled that an edge holds its two ends and an `EdgeRun` saying what the
surfaces around it describe, and never a curve, because a curve is a second
authority beside those surfaces with no tiebreak at the first ulp of
disagreement. §17.1 is the same argument for planes.

M3's output obeys that rule: two surfaces, the half-spaces that pick one piece,
and two ends. Roadmap §6.3's fourth step, fitting the marched polyline to a
B-spline, is a deliberate non-build recorded in §1.8.

What that pushes out is real and has to land somewhere — a mesher still needs
points along an ellipse. It lands in the **evaluator** (§24.7), which makes
certified points on a named crossing and says how far each may stray. The
approximation is then measured where it is made, instead of being frozen into an
object that outlives the measurement and then disagrees with the surfaces that
named it.

### 24.4 M2 closes first, and what lands at the gate

No overlap with M2's lanes, for two reasons, and the second is load-bearing.

`is_wholly_flat` gates `measure`, `classify` and `fuse`. §23.9 left curved
volume to an M2 lane and §23.7 called rebuilding `measure` on a mesh *the*
schedule risk of M2. §24.2 then makes the mesher M3's consumer — so if that gate
is still on `measure` when M3 opens, M3's consumer produces triangles nothing
can weigh and the intersector has no oracle at all. **M3 does not open until
curved `measure` has landed.** `classify` and `fuse` stay gated until M4.

At the gate, before either stage opens:

1. The meet type splits and the topology-free half relocates to `rcad-geom`
   (§24.9).
2. `EdgeRun::PartOfWhereItCrosses::picked_by` widens from `Plane` to `Surface`
   (§24.5).
3. `rcad-geom::Error` gains `Undecidable` (§24.10).
4. Every `rcad-predicates` addition M3 needs is taken **here**, and the crate is
   then frozen for the milestone.

§15.9's last paragraph is the precedent: taking the predicate change before the
window rather than during it cost a day and made the freeze real instead of
aspirational.

### 24.5 How far the pair table goes

Three surface kinds make six pairs, and the useful split is not by pair but by
whether the crossing **lies in a plane**.

| Pair | What they cross in | M3 |
|---|---|---|
| plane ∩ plane | a line | shipped in M2.5 |
| plane ∩ sphere | a circle, in the given plane | named exactly |
| plane ∩ cylinder | an ellipse; a circle, one line or two lines in special position | named exactly |
| sphere ∩ sphere | a circle, in the radical plane | named exactly, by reduction to the row above |
| sphere ∩ cylinder | a degree-4 space curve, planar in special position | special cases named; the rest marched |
| cylinder ∩ cylinder | a degree-4 space curve, planar in special position | special cases named; the rest marched |

The split is not arbitrary. The planar special cases of the last two rows —
coaxial, parallel axes, equal radii with crossing axes — are exactly the
configurations where the marcher has an exact answer to be differenced against,
so §24.1's oracle exists by construction rather than by anyone arranging it.

**M2's edge vocabulary stops being sufficient here, in two places, and both are
gate work.** `EdgeRun::RightRoundWhereItsFacesCross` carries nothing and its
doc says there is nothing to disambiguate; that is true of a circle and false of
a quartic, which can be two disjoint closed loops. And `picked_by` holds
`SmallVec<[Plane; 1]>`, so a piece that can only be picked out by the side of a
sphere or a cylinder cannot be named — `SideOfSurface` already answers that
exactly, and widening the field to `Surface` is one line now and a migration
later.

### 24.6 The surface set does not grow

No cone, no torus. A cone is a quadric and cone ∩ plane is the textbook conic
family, which makes it the most tempting addition on the list, and roadmap §13's
table even files cone and torus sign predicates as work that splits.

M3 changes one half of §23.1's reason — an intersector *is* a consumer for a
sign predicate. It does not change the other half. A new `Surface` variant
touches `validate`, the mesher, `is_wholly_flat`, the fuse and serialization,
which is M2's kind of work arriving inside M3's window, in the same milestone
that already reopens the frozen predicate crate. The cone's consumer is draft
(M6) or a STEP file that actually holds one (M5), and it should arrive there
with the pole machinery M2 built already proven on the sphere.

### 24.7 One evaluator, two arms, and one acceptance test

The evaluator has a closed-form arm where the crossing is planar — parametrize
in the crossing plane, with the bound stated beside the expression per §15.6 —
and the marcher where it is a quartic. Step size comes from `Deflection` and
`Surface::furthest_a_unit_step_goes`, which M2 built for this and which has one
caller today.

Three rules already bind it and none is new. Edge points are worked out **once**
and the same 3D points go to both faces bit for bit (§23.4), so the evaluator is
the single authority and neither face re-derives. What it reports is a length,
in an `Approximation` (§15.2). How fine is `Deflection` and what happens when
the deviation cannot be *proven* is `Policy` — two knobs, never one (§23.5).

**The acceptance test is one sentence: every point the evaluator produces must
be certified ON both surfaces by `SideOfSurface`.** That oracle already exists,
costs nothing to state, and is precisely the check a marched point that drifted
off one surface cannot pass.

### 24.8 Two located surfaces in, and no frame change

`intersect` takes two `Surface`s with their locations already composed, and the
caller in `rcad-topo` does the composing once. Not two `FaceView`s. §19.1 is the
reason: `FaceView::plane` is `surface.transformed(&location)` on **every read**,
and that function was walking a plane's offset an ulp per read and rescaling an
almost-unit normal. An intersector that reads its own faces pays that per pair,
per fuse.

That also settles the crate. A function over two `Surface`s lives in `rcad-geom`
beside them, which is where §15.7 planned to relocate the meet type anyway.

**No working frame is introduced.** Roadmap §2.2 says to centre on the
interaction, snap the shift to a power of two and never rotate, and
`power_of_two.rs` already exists. It is still a conditioning technique with no
figure it has to beat, which is the shape of mistake §1.4 and §11's bump-arena
row exist to stop. Add it when a refusal breakdown names ill-conditioning as the
leading cause — a measurement §24.11 produces anyway.

### 24.9 What the answer can be

`FaceMeet` cannot move to `rcad-geom` whole: its `along: SmallVec<[EdgeKey; 1]>`
names input edges, and `EdgeKey` lives in `rcad-topo`, above that crate. This is
**§23.6 arriving a second time** — two separately settled decisions that turn
out to be jointly impossible — and the same two-level shape resolves it.
`rcad-geom` returns a topology-free `SurfaceMeet`; `rcad-topo` wraps it with the
`along` bookkeeping and the history it owes.

Four arms, covering five outcomes:

- `Nothing` — they do not meet.
- `TouchAtOnePoint` — tangent contact.
- `Curves(SmallVec<[NamedRun; 2]>)` — one crossing curve, or two.
- `SameSurface` — the crossing is two-dimensional.

Point contact and surface coincidence are **answers about the geometry**, not
errors: roadmap §7.4 lists both in the degeneracy catalogue as cases to handle,
and what a fuse does with them is the fuse's decision at M4. Each `NamedRun`
carries an `EdgeRun` and two ends and nothing else, so the fuse drops it into
topology without translating it — a result that needed translating to become an
edge would be the second authority §24.3 refuses.

### 24.10 Tangency decides; what refuses, and what it is called

For two quadrics whose coefficients are exactly representable, "are they
tangent" is a sign question on a discriminant — a polynomial in the inputs —
so level 3 decides it exactly, the same way `orient3d` does. **There is no
near-tangency band for exact inputs.** The band exists only for constructed
inputs (§2.5) and inside the marcher. So exact tangency is an *answer*: a plane
tangent to a sphere gives `TouchAtOnePoint`, a plane tangent to a cylinder gives
one line.

That is a falsifiable prediction and it is written before the work on purpose:
**the refusal rate on a lattice-aligned control must be 0%**, exactly as
§17.7's control row is, and every refusal M3 reports must trace to a constructed
input or to a marcher step whose enclosure was too wide. A refusal anywhere else
is a timid filter, which is the thing §14.7 wanted to be able to tell apart from
genuine undecidability and which the rotated rate alone cannot.

**Naming a crossing is an operation; evaluating points along it is a separate
one.** Naming is a chain of exact sign decisions, so `intersect` returns
`Result` and `rcad-geom` gains an `Undecidable` variant carrying its records —
that crate holds twenty-odd variants today and every one is bad caller input,
because until now it had no operation that could give up. Evaluation refuses
separately, under `Policy`.

**A partly undecidable answer refuses whole.** If one of two curve arms cannot
be named, the call fails. A caller handed one curve has no way to know a second
existed, and a fuse classifying against half a crossing builds a shell that does
not close — §4.1 ranks the crash above the silent wrong answer.

Provenance names identify **the question asked**, not the configuration:
`"where a plane and a sphere cross"`, in the style of the existing
`"side of sphere"` and `WHERE_THREE_PLANES_MEET`. §17.7's breakdown is only
readable if the names partition by what gave up rather than by what was being
modelled.

### 24.11 The numbers, and what generates them

§15.8 says a refusal rate and an acceptance oracle are two different numbers,
and §17.7 fixed the shape: families reported separately, never blended into one
percentage. Four numbers here.

| Number | What it is | Kind |
|---|---|---|
| The curved half of roadmap §7.4 | tangent spheres, tangent cylinder–plane, seam-crossing intersections, closed periodic surfaces | a pass/refuse **table**, not a rate |
| Lattice-aligned pairs | exactly representable quadrics | the **control**, expected 0% (§24.10) |
| Rotated pairs | the same shapes under general rotations | the **headline**, asserted below a threshold recorded here |
| Marcher against closed form | on planar crossings, where both answer | a **ratio to the published bound** |

The fourth exists only because §24.1 chose a redundant marcher, and it could not
have been stated without it. It is a ratio and not a length for §16.8's reason:
the ratio stays comparable across scales where a count of steps does not.

Generators are bars, balls and pipes crossing pipes — nothing extruded. §13 rule
5: M2.5's suite and M3's must not be able to go green together for the same
wrong reason, and sharing a generator is how that happens.

### 24.12 The shape of the work, and the prediction

**Gate, two stages, gate.** Not a fan. The four planar-crossing pairs look like
four pieces with four textbook oracles, but they all reduce to the same
operation — find the plane the crossing lies in, then name a conic in it — so
four owners would be four people editing one reduction. §13's table already says
marching does not split.

The two stages run **in sequence, in one worktree**, closed form first. §15.9's
table gives each crate one owner during a fan, and §24.8 puts both stages inside
`rcad-geom`, so a fork here would need file-level ownership inside a single
crate — the seam-between-two-units failure §13 opens by naming. The dependency
is real anyway: the closed form is the marcher's oracle, so it has to exist
first. Keeping one stage in flight also holds review, the actual constraint, at
one stage's worth.

**The prediction, recorded before the work rather than after it.** §12.1, §17.8
and §23.7 all said the money goes to code that was already written and looked
finished, and the first two were right. Here it is `SideOfSurface` for sphere
and cylinder — shipped in M0, benchmarked, and never once asked a question near
a tangency — and the local interval divide beneath it, which §16.8 left alone on
the explicit grounds that M3 is the first caller that will care. Three
predictions so far, two confirmed; this is the first made in advance, so scoring
it is arithmetic rather than memory.

### 24.13 What gets logged, and how often

`intersect` takes `&Policy` and `&mut ProvenanceLog`, per §12.5's convention:
an operation that can make a non-certified decision takes both, one that cannot
takes neither. `rcad-geom` already depends on `rcad-predicates`, so this adds no
edge to the graph.

**The evaluator's per-point bound is judged where it is made, and only the worst
reaches the edge.** A mesh of a curved face is thousands of points, and §5 says
provenance is recorded per decision, not per coordinate — a record per evaluated
point would be that rule exactly inverted, arriving dressed as thoroughness. One
record per edge, holding the largest deviation any of its points could carry.
`ProvenanceLog::worst_wobble` already sums that shape.

### 24.14 Fuzzing comes due, and the format bumps

§11 deferred `cargo-fuzz` "until M2 — needs a format worth fuzzing", and there
is no `fuzz/` directory in the tree. The reason has been overtaken: the better
target is not the format but **a pair of surfaces in**, with §24.7's acceptance
test as the assertion. That makes it a property fuzzer rather than a crash hunt,
and roadmap §11.3 says coverage-guided fuzzing works unusually well here because
the branches worth reaching *are* the degeneracy handlers — which is all M3 is.

**Roadmap §11.3's "mutate, do not generate" does not apply to this target**, and
that is worth writing down before someone applies it anyway. The warning is
about NURBS, where random control points make garbage no kernel handles. A
sphere is four numbers and a cylinder is seven, and every draw from that space
is a valid surface.

`FORMAT_VERSION` goes to 2 with §24.5's widened `EdgeRun`, and gets a round-trip
test. `rkyv` stays deferred: still no consumer, and §13 rule 3 has not changed.

### 24.15 The intersector is public

The entry point has no topology in its signature once §24.9 splits the type, so
exporting it costs nothing. Roadmap §13's last risk row is the argument —
*ship M1 standalone; NURBS libraries have users who are not kernel users* — and
quadric intersection with certified results and an explicit `Undecidable` has
users who are not kernel users either.

The facade exports the entry point, `SurfaceMeet` and `NamedRun`. `FaceMeet` and
its `along` bookkeeping stay `pub(crate)` in `rcad-topo`. A public consumer is
also the strongest answer §13 rule 3 can be given for the milestone after this
one.

### 24.16 The cost is measured and not tuned

§16.8 measured an enclosure at 12× the `f64` evaluation for a degree-1 curve and
83× for a degree-5 surface, and §24.12 predicts the interval divide beneath it
as the risk. So M3 has to produce a figure for it, or the prediction cannot be
scored.

A figure is not a target. §1.4 and §11's bump-arena row both say an optimisation
with no number it must beat is the mistake, and the number the marcher needs
cannot be known until the marcher exists. **One benchmark: cost per certified
point along a marched crossing, with the share spent inside the interval divide
broken out. Human-read, no target, and labelled as human-read** — §12.5 had to
record that caveat for criterion 10 and it should not need writing a third time.
Optimising the divide defers to the milestone that has a figure to beat.

### 24.17 Every refusal is a limit or a gap

§16.10 was written after "no ordering of the arithmetic helps" turned out to be
true of orderings and silent about scalings, and a representable answer was
refused for it. `CLAUDE.md` carries the operational half: name the family of
fixes considered and say why that family is the only one, and call a refusal a
**limit** when the answer does not exist and a **gap** when it does and the
arithmetic could not reach it.

M3 makes more of these claims than any milestone so far, and the largest is
structural: *the general quartic has no exact naming*. That is a claim about a
whole family of configurations, made by the milestone that also decides not to
build the alternative — §16.10's shape almost exactly. It has to name what was
weighed and put down: resultants, the Steiner special cases, an exact algebraic
representation of the curve. Not because any of them should be built, but
because "cannot" and "we chose not to" are different sentences and only one of
them is true here.

The claim audit runs before the closing gate, and every refusal is classified.

### 24.18 M3 exit criteria

1. The four planar-crossing pairs of §24.5 name their crossing exactly, as an
   `EdgeRun` plus two ends and nothing else.
2. Every named run passes `validate`'s existing check — it sits on both surfaces
   and inside every half-space that picked it.
3. Every point the evaluator produces is certified **ON** both surfaces by
   `SideOfSurface` under a certified policy. This is §14.7's acceptance oracle
   for M3.
4. `TouchAtOnePoint` and `SameSurface` come back as answers, and the curved
   entries of roadmap §7.4 are a pass/refuse table.
5. Lattice-aligned control: **0%** `Error::Undecidable` under
   `Policy::certified()`. One refusal is one bug.
6. Rotated pairs: rate measured, asserted below a threshold recorded in this
   file.
7. Marcher against closed form on planar crossings: deviation as a ratio to the
   published bound, asserted below a ceiling recorded in this file.
8. A curved solid whose edges are **computed** rather than declared meshes, and
   the mesh passes §23.4's oracle — every mesh edge used by exactly two
   triangles with opposite orientation, and χ of the mesh equal to χ of the
   solid.
9. Transform invariance: the same pair under a general rotation names the same
   kind of crossing. Roadmap §11.2 says start here, and it needs no oracle.
10. A `cargo-fuzz` target over `(Surface, Surface)` asserting criterion 3, with
    its corpus checked in and its run duration recorded.
11. Cost per certified marched point benchmarked, with the interval divide's
    share broken out. **Human-read, no target** (§24.16).
12. The claim audit passes, and every refusal is classified limit or gap by name.
13. `FORMAT_VERSION` at 2, with a round-trip test over the widened `EdgeRun`.
14. The facade exports the intersector, `SurfaceMeet` and `NamedRun`;
    `FaceMeet` stays `pub(crate)`.

Criteria 5, 3 and 7 are the load-bearing three. Five is the control that makes
six mean anything; three is the only criterion that separates a correct marcher
from a plausible one; seven exists solely because §24.1 chose a redundant
marcher and could not have been written under any other scope.

### 24.19 What M3 hands to M4

Written as a list because §16.8 and §23.9 both were, and both times the next
gate started from it.

1. `classify` and `fuse` stay gated on `is_wholly_flat`. M4 is where a fuse over
   curved crossings is the first consumer that can remove the gate.
2. The interval divide is measured and not optimised (§24.16). M4 is the first
   caller with a figure it has to beat.
3. The general quartic is named but only ever evaluated by the marcher, so a
   fuse over two crossing pipes is M4's first real test of criterion 7's ratio.
4. `rkyv`, the bump arena, cone and torus, the B-spline fit (§1.8) and the
   working frame (§24.8) all stay deferred, each with its consumer named.
5. §24.12's risk prediction was recorded before the work. M4's gate scores it.

---

## 25. Where the mesher meets the topology

*Recorded 2026-09-01, before the lanes fan out. §15 settled the shared meanings
before M1 and M2.5, §23 before M2's lanes and §24 before M3's stages; this is
the same, for §13 rule 4's reason. Three quarters of M2 has landed. What is left
is the seam between `rcad-mesh` and `rcad-topo`, plus curved `measure`, and it
is about to be four people.*

### 25.1 The boundary is refined by whoever can see both sides of it

**Settled: `rcad-topo` refines every boundary. `rcad-mesh` refines only the
inside of a patch and never adds a point to a loop it was handed.**

§23.4: edge points are worked out once and the same 3D points go to both faces
along that edge, bit for bit. `rcad-mesh` is shown one face at a time, and its
`Cargo.toml` makes that a fact the compiler checks (§23.6), so it cannot know a
loop is shared. If it subdivided one, the two faces along an edge would each
refine in their own `(u, v)` and land on different 3D points — and not by a
rounding. A cylinder's inverse and a plane's are different functions of the same
edge, so the two sides need not even produce the same *number* of points, and
§23.4's oracle would fail by a count rather than by a distance.

Rejected: hand the mesher both faces at once. That is topology in the one crate
whose whole value is not having any, and it costs the oracle that needs no
solid — §13 rule 1's condition, and the reason `rcad-mesh` could be built beside
the rest instead of after it.

Rejected: let the mesher subdivide and have the caller weld coincident points
afterwards. Welding is a tolerance decision on constructed points, which is a
place to manufacture the undecidability §1.7 exists to measure — §15.5's mistake
arriving by a new road. An index cannot drift and costs nothing.

So the mesher may assume **a loop it is given is final**: every corner is a point
the caller means to see in the mesh, it hands every one of them back at the index
it came in at, and it adds nothing to any loop. `Patch`'s own doc already
promises the index half. Points it adds go strictly inside the patch and on the
end of the corner list.

One consequence, because it looks wrong otherwise. A refined boundary point is
**not** certified onto its surface and does not have to be: it comes out of an
evaluator and carries a deviation. `validate`'s "every corner of a curved face is
certified onto its surface" is about vertices stored in the model, not about
points in a mesh, and running the two together would be asking a picture to be as
true as the geometry it is a picture of.

### 25.2 Which of a surface's own two coordinates a point sits at

`Surface::point_at` has no inverse, and every lane below needs one.

```rust
impl Surface  { pub fn coordinates_of(&self, point: Point) -> (f64, f64) }
impl Sphere   { pub fn coordinates_of(&self, point: Point) -> (f64, f64) }
impl Cylinder { pub fn coordinates_of(&self, point: Point) -> (f64, f64) }
```

`Plane::plane_coordinates_of` already exists and keeps its name; the `Surface`
arm calls it.

**Bare numbers. No `Decision` and no `Result`.** It decides nothing and cannot
fail. §23.4 says `(u, v)` is a working space and never an answer, and a predicate
is for answers; handing one back here would invite a caller to take a decision on
projected coordinates, which is exactly what §15.5 forbids.

**It answers about the nearest point on the surface.** A point off the surface
gets its projection's coordinates, which is what `Plane::plane_coordinates_of`
already does, so this is one rule rather than two. The axis of a cylinder and the
centre of a sphere have no nearest point, and there the answer is `(0, 0)`,
stated rather than refused: a caller that reaches it is already broken, and a
number is cheaper than a branch nobody takes.

**What a round-trip test may assert:** for a point *on* the surface,
`point_at(coordinates_of(p))` is within a stated multiple of `radius · ε` of
`p` — the bound a named public constant beside the expression, with a test that
could fail, per §15.6 and its amendment. What it may **not** assert: bit equality
in either direction, or `coordinates_of(point_at(u, v)) == (u, v)`, which is
false for any `u` outside the branch below and undefined at a pole.

**The seam.** `u` comes back on one `atan2` branch, `(-π, π]`, chosen once and
said out loud. A point exactly on a cylinder's seam — `y = 0`, `x > 0` in
`Cylinder::basis` — comes back `u = 0` and never `2π`, because
`atan2(+0.0, positive)` is `+0.0`. So a point on the seam has one answer here and
not two.

**A wire that goes all the way round is the caller's problem and not the
surface's.** Whoever walks a loop is the one that knows it is a loop, so the
unwrap lives in `rcad-topo`: after taking each corner's `u`, add or subtract a
turn so that no two corners in a row jump by more than half a turn. A ball's face
then comes out as the rectangle §25.6 needs, with its two seam sides at `u = 0`
and `u = 2π` instead of both at zero.

Rejected: have `coordinates_of` take the previous `u` and carry on from it. That
makes a surface's answer depend on the order it was asked in, which is the shape
of bug §19.1 and §23.8 keep finding.

### 25.3 The mesher reports a length and never judges it

```rust
pub fn triangulate(domain: &TrimmedDomain) -> Result<Patch, CannotTriangulate>
impl Patch { pub fn deviation(&self) -> Approximation }
```

The signature does not change. What changes is that a `Patch` carries a number.

**A length, in the model's own unit, in an `Approximation`** — §15.2 and §23.5 —
with `Cause::FlatStandIn`, which §25.8 takes here. Never in `(u, v)`: a `u` on a
pipe of radius 1000 and a `u` on one of radius 0.001 are not the same length, and
§23.4 says `(u, v)` is never an answer.

**One number per patch**, the furthest any point of any triangle can be from the
surface. Not one per triangle: §5 records provenance per decision and not per
coordinate, and §24.13 says the same about the evaluator for the same reason.

**`triangulate` does not take a `&Policy`, and that is the decision rather than
an omission.** §12.5's convention gives a policy and a log to an operation that
can make a non-certified *decision*, and this one makes none — it refines until
its own bound is under the `Deflection` it was handed. Whether that is good
enough is a judgement, it belongs to the caller, and the caller is in `rcad-topo`
where the policy already is. A mesher that judged would mean someone who asked
for a rougher picture had silently asked the kernel to certify less, which is the
conflation §23.5 exists to stop.

**There is no best-effort answer.** `triangulate` meets the deflection or
refuses. A patch that quietly missed it and reported the miss would need every
caller to compare two numbers, and none would. The refusal is about the request —
this deflection needs more triangles than this domain can hold — and never about
the surface; naming it is the lane's.

Then what is the number for, if a successful call always comes in under the
deflection? Because it is usually far under, and because the caller has to put a
real figure on the face it built (§15.2) rather than the ceiling it asked for.

### 25.4 A volume error is not a length, and `Measurement` says so in three units

```rust
pub struct Measurement { /* volume, volume_error, surface_area, area_error,
                            centroid, centroid_wobble */ }

impl Measurement {
    pub fn volume_error(self) -> f64;     // cubed unit
    pub fn area_error(self) -> f64;       // squared unit
    pub fn centroid_wobble(self) -> f64;  // length
}
```

**Only `centroid_wobble` is judged against `Policy::max_wobble`.** It is the one
length, and §15.2 says a magnitude is how far a *point* could be out of place —
the centroid is a point and this is how far it could be out of place. The other
two are reported and never judged: a budget in lengths says nothing true about a
cubic unit, and cramming one into an `Approximation` is §12.4's mistake, which
§23.5 refused in advance.

The signature the lane lands:

```rust
pub fn measure(
    self,
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Measurement>
```

A `Deflection` because the answer is only as good as the mesh it is summed over,
and a deflection read from anywhere but the call site is §2.6's footgun. A
`Policy` and a log because it can now make a non-certified decision, which is
§12.5's rule.

**For a wholly flat solid the deflection is ignored**, the exact fan is summed
exactly as it is today, and all three errors come back exactly `0.0`. That is a
claim with a test behind it, and it is what keeps M0's numbers bit for bit.

Rejected: keep `measure()` and add a second `measure_meshed(&Mesh)`. Two
functions answering "how big is this" is two answers to one question, and §23.8
is four instances of what happens when one of them stops being right.

**Not taken at this gate**, which is a change from the plan that opened it.
Adding the fields now would put three of them and one argument into a public type
with nothing producing any of them, tested only by asserting the zero they were
initialised with — §12.6's first gap exactly, the mistake §13 was written after.
`Measurement` and `measure` live in one file that one lane owns; the shape above
is the contract, and that lane lands it in one go.

**Amended 2026-09-01, while implementing. "Judged" cannot mean "refused on", and
the arithmetic is what says so.** `Policy::max_wobble` is `64·ε·working_size` —
`1.4e-12` for a 100 mm model — and `0.0` under `Policy::certified`. A mesh of a
ball of radius three at a deflection of `0.1` leaves a centroid wobble of about
`3e-1`, and the wobble falls roughly in step with the deflection, so meeting that
budget would need a deflection near `5e-13`. A ball asked for one comes back
`MoreMeshPointsThanASolidWillHold`. So
`policy.wobble_is_acceptable(centroid_wobble)` is `false` for **every** curved
solid at **every** deflection that can be met, and a `measure` that refused on it
would have replaced §25.5's blanket refusal with another wearing a hat — the lane
would have delivered nothing.

It is also the conflation §23.5 forbids, arriving from the other side: the
*deflection* would be deciding what the *policy* certifies, so a caller who asked
for a rougher picture would be refused an answer rather than given a rougher one.
§23.5's own division of labour is the one that holds — the deflection says how
fine, and the policy decides what happens when the deviation cannot be *proven*
under it, which is the floor `SolidView::mesh` already asks it about.

So what the sentence above means is that `centroid_wobble` is the one of the
three that is *commensurable* with `Policy::max_wobble`, and comparing them is
the caller's to do on a number handed back for exactly that. `measure` passes
`policy` and the log through to `mesh` and refuses nothing of its own.
`judging_a_meshed_centroid_against_the_policy_would_refuse_every_ball` is the
demonstration rather than the paragraph, per `CLAUDE.md`.

### 25.5 Until then, `measure` refuses a curved solid

§24.4 states as a premise that `is_wholly_flat` gates `measure`. **It does not,
and never did.** `classify` and `fuse` ask it; `measure` was left out when §23.8
item 4 added the shared check, and nothing in the suite asked.

What happened on a round bar was neither a refusal nor an answer. Its caps fan
into no triangles at all — a one-corner wire has no polygon — and its wall into
two triangles with no area, so `six_volumes` comes out a clean `0.0` and
`measure` dies on its own `assert!(six_volumes > 0.0)`. A ball does the same. The
assert is right that a solid enclosing nothing is a broken model; it is wrong
about which of the two it was looking at.

**Settled: `measure` asks `is_wholly_flat` first and returns
`Error::MeasureNeedsFlatFaces`**, alongside `Error::FuseNeedsFlatFaces` and for
the same reason — the solid is sound and the operation is what is not ready for
it. Deliberately not `Error::Undecidable`: nothing here is in doubt. Asked over
every face before anything is summed, so which answer comes back cannot depend on
the order the faces were reached.

Landed at this gate, with a test. It comes back out when §25.4's lane lands.

### 25.6 A pole is a side of the rectangle, not a repeated corner

`Loop::through` refuses fewer than three corners and `triangulate` refuses two
corners in the same place, so a ball's face — a rectangle in `(u, v)` whose top
and bottom edges are each one point in 3D — has to be handed over in some shape
that survives both.

**Settled: the caller hands over the rectangle, and a pole is a *side* of it
whose corners share one 3D point.**

A ball's one face is four wire steps: down the seam, across the south pole, up
the seam, back across the north pole. In `(u, v)` that is four corners —
`(0, π/2)`, `(0, −π/2)`, `(2π, −π/2)`, `(2π, π/2)` — three or more of them, all
different, so `Loop::through` and `triangulate` take it as it stands. The two
`u`s differ because §25.2's unwrap made them differ.

**Identity is the caller's table and never the mesher's index.** Two corners at
different `(u, v)` may be one 3D point, and both pole corners of that rectangle
are. The topo layer maps each `Patch` corner to a point index of its own and maps
every pole corner of a face to the same one, so §23.4's oracle — every mesh edge
used by exactly two triangles, opposite ways round — is a statement about indices
and comes out right. A triangle whose three corners do not map to three different
points is **dropped**: it has no area and covers nothing, and dropping it is what
turns the rectangle's top row into the fan a sphere's pole actually has.

What the mesher may refuse is what it refuses today and nothing new: fewer than
three corners, two corners in the same place, a boundary that crosses itself, a
corner of one loop sitting on another loop's side. It is **not** entitled to
notice that two different `(u, v)` are one point, because it cannot see 3D at all.

Rejected: a pole as one repeated corner, with the mesher taught to allow it. That
asks a crate with no topology to tell which repeats are a pole from which are the
broken input `CornersCoincide` exists to catch — the same overloading §23.3
turned down for zero-length edges, and it makes that check unaskable the same way.

### 25.7 What a mesh of a solid is

§23.6 settled the two levels: `rcad-mesh` returns topology-free triangles and
`rcad-topo` pairs them with the key it asked about. This names the upper one.

`crates/rcad-topo/src/mesh.rs`:

```rust
pub struct Mesh { /* … */ }
impl Mesh {
    pub fn points(&self) -> &[Point];
    pub fn triangles(&self) -> &[MeshTriangle];
    pub fn deviation(&self) -> Approximation;
}

pub struct MeshTriangle { /* … */ }
impl MeshTriangle {
    pub fn corners(self) -> [u32; 3];
    pub fn face(self) -> FaceKey;
}

impl SolidView<'_> {
    pub fn mesh(
        self,
        deflection: Deflection,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Mesh>;
}
```

**One point table for the whole solid, and that is what watertightness *is*
here.** An edge's points appear in it once, and both faces along that edge name
the same index, so §23.4's oracle is a check on indices with no distance in it
anywhere. §23.4 called the claim combinatorial rather than metric; this is the
type that makes it so.

`mesh` takes a `&Policy` and a log where `triangulate` takes neither (§25.3). The
judging happens once, here, where the policy is.

**The order is fixed** — faces in stored order, and within a face whatever
`triangulate` gives back for a domain built the same way — because §9.2's
benchmark compares a serial run against a parallel one bit for bit, and every sum
built on a shuffled mesh would come out different.

The facade exports `Mesh` and `MeshTriangle`. §24.15's argument for the
intersector applies here for less work: the only topology in the signature is a
`FaceKey`, which is public already.

### 25.8 What landed at this gate

Each has a test that fails without it.

1. **`Cause::FlatStandIn`** (§25.3), taken here rather than in a lane. §15.9's
   precedent is exact: `Cause::ConstructedPoint` was taken before M1's worktree
   opened so that the freeze on `rcad-predicates` would be real rather than
   aspirational, and it cost a day. **`rcad-predicates` is frozen from here until
   the lanes close.** The variant goes **last** in the enum, because `postcard`
   writes an enum as its number: one inserted in the middle makes every
   `Approximation` already on disk read back blaming something else, silently,
   because the bytes still parse and the magnitude is still right. There is a
   test on the numbering.
2. **`measure` refuses a curved solid** (§25.5).
3. **`SolidView::bounding_box` is now `box_around_its_corners`** (§25.9).
4. **`Deflection` is exported from the facade.** Nothing out there takes one yet,
   and it goes out anyway: every signature this section writes down takes one,
   and three lanes would otherwise each add the same line to the one file they
   share. §15.9's table makes the facade's re-export list the shared, append-only
   thing precisely so that a fan does not contend over it, and pre-loading the
   one name all of them need is what a gate is for.

Not taken, and named here so that nobody takes them inside a lane: the
`Measurement` fields and `measure`'s new arguments (§25.4), and `Mesh` (§25.7).
Both would be public shapes with nothing producing them.

### 25.9 What the gate found that the plan did not

**`bounding_box` was not one, and had not been since a curved face became
legal.** §23.8 recorded four defects that were each a helper meaning one thing
until a new kind of input arrived; this is the fifth, and it was still in the
tree because §23.8 stopped at the ones that had already fired.

A round bar's two corners are both on its seam, so the box round them is a
**line**, while the bar bulges a whole radius out of it on every side. Nothing
crashed and nothing refused: a caller asking how big a bar is got a confident
wrong answer, which §4.1 ranks below the crash. A ball is worse — its one face
has two corners, two corners make no triangle, and the answer is `None`, which
the doc said meant "no faces at all". A ball has one.

Renamed `box_around_its_corners`, which cannot be misread, exactly as
`distance_between_its_ends` was (§23.8 item 3). A real box round a curved solid
needs the mesh and belongs to the lane that builds one. `classify` is the only
caller inside the kernel and it asks `is_wholly_flat` before it reads a box, so
nothing changes for it.

Also corrected: `rcad-mesh`'s crate doc still said the triangulator was not there
yet. It shipped in the same milestone that wrote the sentence.

### 25.10 Who owns what, and in what order

| Lane | What it is | Files it may touch | Opens |
|---|---|---|---|
| **A. Which coordinates a point sits at** | §25.2 | `rcad-geom/src/surface.rs`, `sphere.rs`, `cylinder.rs`, `lib.rs`, its own new test file | now |
| **B. Refinement, and the number for how far it strays** | §25.3 | everything under `crates/rcad-mesh/` | now |
| **C. The mesh of a solid** | §25.1, §25.6, §25.7 | `rcad-topo/src/mesh.rs` (new), `view.rs`, `lib.rs`, `rcad-topo/Cargo.toml`, `rcad-topo/tests/meshing_a_curved_solid.rs` (new) | when A lands |
| **D. Curved `measure`, and the two gaps behind it** | §25.4, §25.5, §23.9's first two | `rcad-topo/src/measure.rs`, `validate.rs`, `error.rs`, `rcad-topo/tests/measuring.rs`, `a_solid_with_a_curved_face.rs`, `rcad/src/lib.rs` | when C lands |
| `rcad-predicates` | nobody | frozen (§25.8) | — |

**Not a four-way fan, and the ordering is a dependency rather than a preference.**
A and B are in different crates and neither consumes the other: B refines inside
a patch, which needs `point_at` and a curvature bound and never the inverse. C
needs A's inverse to put a shared edge point into each face's own `(u, v)`, and
it is written against B's contract rather than B's code, so it needs B landed only
to run green. D needs C's `Mesh`.

That puts two lanes inside `rcad-topo`, which §13 rule 6 permits — it caps work
in flight in one crate at two or three — and they are in sequence rather than
side by side because the dependency is real. §24.12 made the same call for the
same reason.

`crates/rcad/src/lib.rs` is shared, re-export lists only, as §15.9's table has
it, **with one exception written down here so it is not a surprise**: the worked
example in it calls `measure()`, and §25.4 changes that signature. That edit
belongs to D. C hands its two re-export lines to D rather than editing the file.

§13 rule 5 wants the lanes' generators to differ so that two suites cannot go
green together for the same wrong reason. A draws points on spheres and pipes, B
draws polygons in a flat domain, and those two are clear of everything. **C and D
share the bar and the ball**, because those are the only two curved solids that
exist, and that is worth naming rather than glossing. What separates them is the
oracle: C's is combinatorial — every mesh edge used by exactly two triangles, and
χ of the mesh equal to χ of the solid — and D's is arithmetic, a bar's volume
against `πr²h`. No one wrong mesh satisfies both.

M2's closing gate writes **§26**; the number is reserved here so the lanes do not
collide over it.

---

## 26. What M2 actually cost, recorded 2026-09-01

*Written by M2's closing gate, per §25.10's reservation of this number. §12 did
this for M0, §16 for M1 and §17.8 for the fuse.*

M2 is complete. Topology holds curved faces, seams and poles; `rcad-mesh` cuts a
trimmed patch into triangles to a stated deflection; `SolidView::mesh` puts a
whole solid into one point table; `measure` weighs a solid that is not a
polyhedron and says in three separate units how much of the answer the triangles
could be hiding. `validate`'s two named gaps (§23.9) are closed. The suite went
from 796 tests to 860, and the claim audit that closes every milestone ran last
and found two defects and two overstated sentences.

`rcad-predicates` was touched once, at the gate, to add `Cause::FlatStandIn`.
§25.8's freeze held for the whole of the fan.

### 26.1 §23.8's pattern held, and there are seven of them now

§23.8 recorded four defects with one shape: **a helper that meant one thing
until a new kind of input arrived, and then quietly meant another.** It predicted
nothing about how many more there were. Three more turned up:

5. **`bounding_box` was not one** (§25.9). A round bar's two corners are both on
   its seam, so the box round them is a line while the bar bulges a radius out of
   it on every side, and a ball's answer was `None` — which the doc said meant a
   solid with no faces. Renamed `box_around_its_corners`.
6. **`measure` asserted its way to a panic** (§25.5). A round bar's caps fan into
   no triangles at all, so `six_volumes` came out a clean `0.0` and `measure`
   died on its own `assert!(six_volumes > 0.0)`. The assert was right that a
   solid enclosing nothing is broken and wrong about which of the two it was
   looking at.
7. **A ball asked for a rough picture meshed into nothing at all** — found by the
   claim audit, after all four lanes had landed and the suite was green. The
   mesher sized a step by how far it went round in *longitude*, and a ball's seam
   runs pole to pole: half a turn of latitude and no longitude whatever. Past a
   deflection of about `6.6 · radius` the seam came back one step long, the face
   was a rectangle whose only corners were its two poles, every triangle stood on
   one point twice and was dropped, and `mesh` handed back two points, no
   triangles, and a deviation that read perfectly honestly. `measure` then died
   on the *same* assert as item 6, down a road item 6's fix did not cover.

So the pattern held, and it sharpened. In items 1 to 5 the new input was a new
kind of **solid** — the first curved face. In items 6 and 7 it was a new kind of
**number**: a deflection, which is a knob a caller turns and no shape at all.
That is the harder half to see coming, because a new solid arrives with a new
builder and a review, and a new number arrives inside an existing signature.

The audit also found a fourth instance of the §16.10 species proper — three
places where the mesher **asserted** on a shape `Model`'s `Deserialize` can
deliver, one of them inside `validate`, which is the one function in the kernel
whose job is to hand back a refusal rather than crash. §26.5 has them.

### 26.2 §23.7 predicted the risk and thereby defused it

§23.7 named `measure` — shipped in M0, exact, bit-for-bit reproducible, and read
by every metamorphic property in the fuse suite — as M2's schedule risk, on the
grounds that rebuilding it on an approximate mesh puts inclusion–exclusion,
idempotence and absorption downstream of new arithmetic.

**Half right, and the half that was wrong is wrong for a good reason.** It did
not slip there, because the prediction was acted on: lane D left the exact path
untouched, byte for byte, and wrote the mesh path beside it behind
`is_wholly_flat`. `a_flat_solids_measurement_is_bit_for_bit_what_it_was` freezes
six shapes' volume, area and centroid as bit patterns read off the kernel before
the change, and it catches a turned cube routed through the mesh — its volume
moves by an ulp and its centroid much further. A prediction that causes the risk
to be designed around is a prediction working, not a prediction failing, and it
should be scored that way.

The unbudgeted time went to three things nobody had listed:

- **No patch may span more than a quarter turn.** A triangle is measured by its
  longest chord, and a chord stops growing with the turn it spans once that turn
  passes a half — right the way round it is zero again. A ball's patch therefore
  filled with triangles that each stood on one point twice, and the mesh came
  back empty. Half a turn is not enough either. Lane C found this with its own
  oracle, and the claim audit later found the same rule missing in the other
  direction (§26.1 item 7), which makes it the most expensive single idea in the
  milestone.
- **χ of a solid is not `V − E + F`**, twice over: a face with a hole is worth
  `2 − wires` rather than one, and a pole is not an edge. The oracle found the
  first of those on a shape the suite already trusted.
- **§25.4's `max_wobble` reading had to be amended mid-flight**, because taken
  literally it refused every curved solid at every reachable deflection.

**The running score, since §24.12 asks for one.** §12.1 stated the lesson after
the fact rather than predicting anything. §17.6 predicted, against §1.7 in the
same file, and was right. §23.7 predicted and was half right in the sense above.
§24.12's is made and not yet scored. So: **three predictions made in advance, one
right, one half right, one open.** §24.12 also says it is "the first made in
advance", which is not so — §23.7 was written before M2's lanes fanned out, and
this section is what scores it.

### 26.3 §25 was wrong about arithmetic and right about ownership, and that is why it paid

§25 exists because §13 rule 4 says a meaning two people settle separately gets
settled twice, differently. Four lanes ran off it. Three of them reported places
it was silent, incomplete or wrong:

- **§25.6's "`Loop::through` and `triangulate` take it as it stands" was false,
  and is now false three times over.** It was written about a ball's rectangle
  and it is true of the two refusals it names — but the rectangle a ball hands
  over cannot be triangulated *usefully*, because every triangle in it spans the
  seam. Lane C found the quarter-turn cut. The audit found the same rule missing
  in latitude.
- **§25.2's suggested bound shape was false.** "Within a stated multiple of
  `radius · ε`" reads perfectly and is wrong: the angle a pipe reads off a point
  is a difference of terms the size of the whole model, so a pipe of radius one a
  long way out loses the model's worth of bits and not one radius' worth. The
  constant landed as `256 · ε` of *the largest number involved*, and the worst a
  search over forty orders of magnitude can find is `2.4 · ε` of it.
- **§25.4's `max_wobble` reading was amended while the lane was open**, as above.
- **§25.10's file table under-counted a signature change.** Lane D was given six
  files and touched twenty, because `measure`'s new arguments ripple into every
  test that calls it. Nothing collided, because D ran last and alone; had it run
  beside another `rcad-topo` lane it would have.

**Was writing it first still worth it? Yes, and the split says why.** Everything
§25 got wrong was about *arithmetic* — a bound's shape, how coarse is too coarse,
what a budget compares against. Code finds those, cheaply, in the lane that hits
them. Everything §25 got right was about *meaning and ownership* — who refines a
boundary, what a `(u, v)` is for, which of three knobs decides what, where
identity lives, what a mesh of a solid is. Two people settle those differently
and neither is wrong, which is the expensive kind. Four lanes wrote across five
crates with one shared re-export file and collided over nothing; `Cause::FlatStandIn`
went in at the gate and the `rcad-predicates` freeze was real rather than
aspirational. A gate that is wrong about numbers and right about names has done
its job. It should stop suggesting numbers.

### 26.4 The numbers

| | at §25's gate | now |
|---|---|---|
| tests (`cargo nextest`) | 796 | 860 |
| doc tests | 23 | 24 |
| suite, wall clock | 16.0 s | 17.0 s |

What a curved solid costs, release build, 100 mm working size:

| | `validate` | `measure` at the model's own deflection | triangles |
|---|---|---|---|
| cube of side ten | 9.9 µs | 5.4 µs | 12 |
| bar, radius 3, height 5 | 30 µs | 272 µs | 112 |
| ball, radius 3 | 23 µs | 1.31 ms | 840 |

`validate` meshes every curved solid now and did not before, and it costs two to
three times a polyhedron's sweep to do it — because it asks for the coarsest mesh
there is, which is a sign and a count and neither gets truer with more triangles.
`measure` is two orders above `validate` on the same solid for the opposite
reason: it is summed over a mesh at the deflection the caller asked for.

Two published bounds and how much room the drawn cases leave:
`Surface::ROUND_TRIP_ROUNDOFF` is `256 · ε` of the largest number involved and
the worst drawn case uses `2.4`; `rcad-mesh`'s evaluator roundoff is `8 · ε` of
how far the surface reaches and the worst drawn case is inside ten times it, with
a test that fails if a constant ten times too big would also pass.

### 26.5 What the claim audit found, and what it could not break

The audit (`claim-audit`, per §16.10) ran after review, over every "cannot",
"nothing reaches this" and "by construction" the four lanes wrote. Two defects
and two sentences.

**Defects.**

1. **The collapsing ball** (§26.1 item 7). A gap, not a limit: the mesh exists,
   the arithmetic simply did not ask for enough of it. Fixed in the mesher, where
   the floor belongs. `validate` had already met this and worked around it by
   meshing a ball at its own radius instead of as coarsely as the mesher goes —
   and a workaround in one caller is a floor only that caller gets, which is
   exactly why `measure` panicked. The workaround is gone and the sweep is a
   constant again.
2. **Three shapes a file can hold that the mesher asserted on.** Lane C wrote
   *"a curved face with a hole is asserted against; nothing public builds one"*,
   which is §16.10's mistake verbatim: no *builder* makes one, and `Model`
   implements `Deserialize`. §16.11 already records that `validate` is the
   untrusted-import boundary, and since §25 that sweep meshes. The three are a
   curved face with a hole, a wire that comes back to a different longitude than
   it left, and a pole step on a flat face. All three are refusals now —
   `Error::CurvedFaceWithAHoleGoesTooFarRound` and
   `Error::AWireWillNotLieFlatOnItsFace`. The second one panicked *inside*
   `validate`: a ball missing one of its two pole steps is three wire steps that
   chain up and close, so nothing earlier in the sweep has anything to say about
   it.

**Sentences, corrected rather than fixed.**

3. **Is the deflection floor a limit, or does a scaling reach it?** §16.10 exists
   because "no ordering of the arithmetic helps" was true of orderings and silent
   about scalings, so both floors were asked the question directly.
   `Error::DeflectionBelowWhatAMeshCanReach` is a **limit**: it is a fixed
   multiple of the solid's own coordinates, so shrinking the whole model shrinks
   the floor in exact step and nothing *relative* is ever refused, and no
   ordering, rescaling or compensated sum puts a point between two neighbouring
   `f64`s. `rcad-mesh`'s `NoMeshCanGetThatCloseToTheSurface` said "no mesh" and
   meant "no mesh built from points bounded the way this crate bounds them" — a
   pipe is invariant along its own axis, so one whose axis origin sits a billion
   units up it is refused a deflection it could comfortably meet. That is a
   **gap in a bound rather than in an answer**; closing it needs the axis
   direction, because an error along `z` moves a point off a `z` pipe by nothing
   and off a slanted one by all of it. `rcad-topo` never builds one —
   `add_round_bar` puts the axis at the origin and carries the placement on the
   solid — so it is named and left, which is what §16.10 asks for when the family
   of fixes is known and the consumer is not.
4. **`validate` claims §23.4's oracle and checks something weaker.**
   `every_side_of_the_skin_is_used_both_ways` counts a *balance* and not a count,
   so a side used four times, twice each way, would pass. There is no input the
   kernel can build that fails the count without also failing the balance, so
   there is no test that could fail if it were strengthened — and a change with
   no failing test is not a change (`CLAUDE.md` rule 2). The sentence says so
   now, in both the sweep and the suite's own copy of the oracle.

**Two of the four flagged claims survived, and here is how hard they were
pushed.** Lane C's ball-seam comment says a ball whose seam went elsewhere would
get "a watertight mesh of the right ball cut in the wrong place", and that
nothing today could tell the difference. Built: a ball whose `EdgeRun` records
its seam a quarter turn from where `add_ball` puts it meshes into 276 triangles,
passes the whole combinatorial oracle, validates clean and measures the volume it
should. Nothing can tell, and the reason is that a sphere looks the same from
every meridian and only that one face uses that edge. It is a **gap** all the
same — the longitude is in the model, on `EdgeRun::PartOfWhereItCrosses`, and the
mesher does not read it — and it becomes a wrong answer the moment a second face
meets a ball along that seam. That is M3's, and §26.6 hands it over.

Lane D's `Undecidable` arm out of a triangulation is a **gap** and could not be
reached. §2.5 says `Undecidable` arrives through *constructed* points, and every
corner handed to the mesher today is the projection of an exact stored corner; the
only other road is a coordinate product overflowing or falling subnormal, and
`Profile::build` runs the same exact predicate on the same corners first, with the
triangulation's own `2^-142`–`2^201` window closing the rest. Probed across forty
orders of magnitude of radius and height, through the public builders, in every
combination: **not one panic and not one `Undecidable` — every extreme came back
as a named refusal.** So the arm is correct, correctly classified and currently
unreachable, and the first thing that reaches it is a mesh domain holding a
constructed corner, which is M3's crossings.

### 26.6 What M2 hands to M3

§24.4 says M3 does not open until curved `measure` has landed. **It has landed**,
with error terms in three units and its exact path frozen bit for bit, so that
precondition is met and M3's gate may open on §24.4's four items. Six things M3's
gate must now also deal with, which §24 did not know:

1. **The mesher can only walk an edge that is a straight line in the surface's
   own two coordinates.** That is exactly right for the three edges this kernel
   builds — a pipe's ruling, a cap circle, a ball's meridian seam — and false in
   general: a circle cut out of a ball by a plane that misses the centre is not
   one. §24.2 makes the mesher M3's consumer, so **M3's first crossing is also
   the first edge the mesher cannot walk.** This is the largest single item on
   this list and it is not in §24.
2. **A ball's seam longitude is readable off `EdgeRun::PartOfWhereItCrosses`, and
   §24.4 item 2 widens that very field from `Plane` to `Surface`.** The two are
   the same edit arriving from two directions. Whoever widens the field should
   also make the pole-to-pole walk read the meridian out of it instead of
   assuming longitude zero; doing it at the gate costs a few lines and doing it
   after M3's first sphere–plane crossing costs a debugging session.
3. **`validate` meshes now, so a new `Surface` variant is nine stops and not
   one.** Three in `rcad-topo/src/mesh.rs`, one in its `measure.rs` and five in
   `rcad-mesh/src/triangulate.rs` all match on `Surface` and panic on an arm they
   have not been taught — which is the right answer to a broken workspace
   invariant and a long list to walk. §24.6 says the surface set does not grow in M3, so
   nothing is due — but the cost of growing it has gone up and should be counted
   before, not during.
4. **`Undecidable` out of `mesh` becomes reachable when a mesh domain first holds
   a constructed corner** (§26.5). M3's crossing points are the first. The arm
   and its provenance record already exist; what does not exist is a test, and
   M3 is the milestone that can write one.
5. **`rcad-mesh`'s evaluator-roundoff bound over-charges a pipe placed along its
   own axis** (§26.5 item 3). Unreachable from `rcad-topo`. If M3 ever hands the
   mesher a surface it did not get from a `Model`, this becomes reachable, and
   the fix needs the axis direction.
6. **`measure` over a mesh still asserts `six_volumes > 0.0`.** After §26.5's fix
   nothing found can reach it — the coarseness sweep now runs to a hundred
   thousand times the solid, and `validate` meshes every curved solid as coarsely
   as the mesher goes and checks the sign. It is an assert on a number derived
   from a caller's deflection, and that is the shape §26.1 items 6 and 7 both
   had. Worth a second look the next time a new kind of surface arrives.

Carried forward unchanged from §23.9's third item: **arcs in a profile are still
their own piece**, and every 2D self-intersection check in `build.rs` would have
to become arc-aware. Nothing in M3 needs one.

### 26.7 Corrections to earlier sections

- **§24.12**: "this is the first made in advance" is not so. §17.6 named the
  fuse's risk against §1.7 before the work, and §23.7 was recorded before M2's
  lanes fanned out. §24.12's is the third made in advance and the only one still
  open; §26.2 keeps the score.
- **§25.5**: `measure`'s blanket refusal of a curved solid came back out when
  lane D landed, as that section said it would.
- **§25.6**: "`Loop::through` and `triangulate` take it as it stands" is true of
  those two refusals and not of the mesh that results. See §26.3.
- **§25.10**: the file table is a statement about the lane's *own* files and not
  about the ripple of a signature it changes. See §26.3.
- **§23.9**: all three gaps are addressed. A round cap facing outwards and a
  curved solid's volume are both checked over the mesh in
  `check_shells_and_solids`; arcs in a profile are deferred again, at item 3
  above.

---

## 27. What opening M3's gate settled

*Recorded 2026-09-01, before either stage opens. §24 is the specification and it
stands; this is what its four gate items turned out to be once somebody read the
discriminants, plus the six things §26.6 handed over that §24 did not know about.
A gate and not a milestone, so it is short.*

### 27.1 The two stages on paper: every sign question, and what it costs

§24.4 item 4 cannot be done without knowing what the stages will ask, so here is
the whole list. Every question is a **sign**, every one is a polynomial in the
numbers a `Surface` stores, and none of them is a distance compared against a
tolerance.

Write `N = n·n` for a plane's normal, `W = w·w` for a pipe's axis direction —
both about one and exactly one only for a coordinate direction.

| The question | The polynomial | Degree | Dimension | Who decides it |
|---|---|---|---|---|
| are two directions parallel | three 2×2 minors of `w₁, w₂` | 2 | ratio | exact, `orient2d`'s shape |
| is a plane parallel to an axis | `n·w` | 2 | ratio | exact, two products |
| do two axes meet | `(o₂−o₁)·(w₁×w₂)` | 3 | volume | `orient3d`'s determinant, over stored numbers |
| is a centre on an axis | three components of `(c−o)×w` | 2 | length | exact, `orient2d`'s shape |
| plane against ball | `N·R² − (n·c−d)²` | 6 | area | new, in `rcad-geom` |
| plane against pipe, parallel to it | `N·r² − W·(n·o−d)²` | 6 | area | new |
| ball against ball | `(R₁±R₂)² − ‖c₂−c₁‖²` | 2 | area | new |
| ball against pipe, centre on the axis | `W·R² − r²` | 4 | area | new |
| ball against pipe, in general | `V² − 4r²A`, with `A = ‖(c−o)×w‖²` and `V = A + r² − W·R²` | 8 | **four lengths** | new |
| pipe against pipe, parallel axes | `(A·W₂ − r₁²W₂ − r₂²W₁)² − 4r₁²r₂²W₁W₂` | 12 | **four lengths** | new |
| pipe against pipe, one surface | `A = 0` and `r₁²W₂ = r₂²W₁` | 4 | area | new |

Four things fall out of that table, and three of them are corrections to §24.

One row needs a note. "Do two axes meet" is `orient3d`'s three-by-three
determinant exactly, but not `orient3d`'s *signature*: feeding it four points
means adding a direction to an origin, and that sum rounds. The determinant is
written over the stored numbers instead, which is what every other row does too.

**The discriminant has to carry `n·n` and `w·w`, and the textbook one does not.**
A plane's normal is scaled to length one on the way in and that division rounds,
so `Plane::side_of_point` is certified against the plane those *stored* numbers
name; the same is true of a pipe, where `Cylinder`'s own doc says the certified
radius is `r / ‖w‖`. Write the discriminant as `R² − (n·c−d)²` and it answers
about a ball of radius `R/‖n‖` — a different ball, by about `1e-16·R`. Then
criterion 3, which wants every evaluated point certified **ON** both surfaces by
`SideOfSurface`, fails by exactly that much on any surface that is not
axis-aligned. Multiplying through by `N` costs three products and two sums.
**The rule, and it is the load-bearing one: an intersector's discriminants are
polynomials in the same stored numbers `SideOfSurface` reads, with no
normalization anywhere.** Otherwise the two halves of M3 are certified about two
different solids.

**There is no near-tangency band because there is no constructed input.** §24.10
says the band exists only for constructed inputs and inside the marcher. Sharper:
`intersect` takes two `Surface`s, a `Surface` is made of `f64`, and level 3
decides any polynomial in exact `f64`. The naming half of M3 has **no**
constructed input at all. Its only way to refuse is exact arithmetic declining
because a product would overflow or fall into the subnormals — which §16.10 calls
a **gap** wherever a rescaling reaches the answer, and
`the_same_pipe_and_point_scaled_up_to_fit_put_the_point_outside` already
demonstrates that shape one degree down. So criterion 5's 0% is not a hope about
a narrow band; it is structural, and a refusal from the closed-form pairs at
ordinary magnitudes is a bug rather than a close call.

**The two quartic pairs do not fit the expansion buffers this workspace uses, and
that is the schedule risk nobody had listed.** `square_of_expansion` refuses more
than eight entries; a pipe's `‖(c−o)×w‖²` alone runs to 384. Squaring a
400-entry expansion needs `2·400²` — about 320 000 `f64`, 2.6 MB — and that is
not a stack array. The family of fixes considered: **(i)** a heap scratch used
only at level 3, which runs rarely, costs one allocation and refuses nothing;
**(ii)** decline past some length and report `Undecidable`, which trades
criterion 5's 0% away for a buffer size; **(iii)** drop the degree, which needs
`√A` and so leaves polynomial arithmetic altogether. **(i) is the answer**, and
it belongs to stage 2 — but `expansion`'s crate doc says *"every function writes
into a caller-supplied slice, so a predicate that knows its own sizes never
allocates"*, and a predicate that cannot know its own size within a stack budget
is a case that sentence did not have. Zero elimination keeps the drawn lengths
tiny; it is the worst case a fixed buffer would have to hold.

**None of the four planar pairs constructs a plane.** §24.5 files sphere ∩ sphere
as *"named exactly, by reduction to the row above"*, through the radical plane.
The reduction is needed to *evaluate points along* the circle and not to *name*
it: two spheres name their circle between them, so the run is
`RightRoundWhereItsFacesCross` and nothing is built. Plane ∩ sphere and the
ellipse of plane ∩ cylinder are the same. `picked_by` is only ever needed where a
crossing comes in **two pieces** — two lines from a plane parallel to a pipe's
axis, two circles from a ball centred on a pipe's axis, two loops of a quartic —
and in the first two the picking plane is exactly representable.

### 27.2 What went into `rcad-predicates`, and what was weighed and put down

Two things, and the crate is frozen from here until M3 closes.

1. **`Measures::ManyLengths { how_many }`.** The table above has two entries at
   the fourth power of a length, and `Measures` stopped at a volume. An
   enclosure from one of them filed as a `Volume` reads a whole working size too
   wide — so a decision the policy would have taken comes back undecided
   instead, which is the timid filter §24.10 says a 0% control cannot tell from
   genuine undecidability. §15.3's units bug, one degree above where §15.3 could
   see it.
2. **`Cause::PointAlongACrossing`.** §15.7 said the `Approximation` field for how
   far a computed curve strays has no producer yet and §24.3 answers that the
   fidelity belongs to *evaluating points along* a crossing. That is a point put
   where a curve is, and `Cause::FlatStandIn` is a triangle put where a surface
   is — one dimension apart, in the same mesh, and only the cause tells them
   apart. Last in the enum, per §25.8.

Weighed and **not** taken:

- **Multiplying one expansion by another.** Every discriminant of degree six and
  up needs it. It is not a `rcad-predicates` change: `rcad-geom` already owns
  `square_of_expansion` privately, §7.2 puts the quadric predicates in that crate
  because the quadric types are there, and the freeze does not reach it.
- **A public `Interval` divide.** The marcher will divide ranges.
  `Interval::divided_by` is `pub(crate)` and `rcad-geom` has `quotient_enclosing`
  of its own; both stages live in `rcad-geom`, so moving that function inside
  that crate is stage work and costs the freeze nothing.
- **`Measures` for a dimensionless value.** "Are these two axes parallel" and "is
  this plane parallel to that axis" have no units. Their inputs are direction
  components of order one, every product is exact, level 3 always decides, so no
  record is ever written and no arm would ever be read. §13 rule 3.
- **A fifth `Decision` for "tangent".** Tangency is a certified `Zero` on a
  discriminant. A variant for it would be a second answer to a settled question,
  which is §1.5's shape.
- **`Policy` knobs for the marcher.** §24.7 asks for two, and they exist: how
  fine is `Deflection`, which is not a policy, and what happens when the
  deviation cannot be proven, which is `Policy::wobble_is_acceptable`.
- **Provenance name constants for the new questions.** `&'static str` at the call
  site in `rcad-geom`, exactly as `SIDE_OF_SPHERE` is.

### 27.3 The type split, and three places §24.9 needs amending

`EdgeRun` **moves to `rcad-geom` whole**, which §24.9 left open. Nothing in it
names a key: it holds a second `Surface` and the half-spaces that pick one piece,
and both are geometry. `EdgeData` — which holds vertex slots — stays where it is.
`EdgeRun` therefore becomes public, so §24.15's export list is three names and
not two.

Three amendments:

- **`Curves` carries a `Vec`, not a `SmallVec<[NamedRun; 2]>`.** Two `NamedRun`s
  inline come to over five hundred bytes, every arm of an enum is as wide as its
  widest, and `SurfaceMeet::Nothing` is what most pairs of faces in a fuse
  answer. Widening the common answer to save an allocation on the rare one is
  backwards.
- **`RightRoundWhereItsFacesCross` still carries nothing.** §24.5 calls this gate
  work, and what landed is the corrected *claim* rather than a field: the variant
  says "one closed curve", which is true of every crossing the four planar pairs
  produce and false of a quartic that leaves two loops. The families of fixes are
  half-spaces on the closed variant, or a variant of its own. Neither is built,
  because the only consumer is stage 2, stage 2 is one owner in one worktree
  (§24.12), and §13 rule 4's settle-it-twice hazard needs two people. The format
  is versioned and explicitly unstable, so a third version is cheap.
- **The fuse's `FaceMeet` did not move and did not change.** It is a plane–plane
  meet whose ends are `MeetPoint`s naming input vertices, and it belongs to
  M2.5's planar fuse. §24.9's "`rcad-topo` wraps it with the `along` bookkeeping"
  describes M4's shape, and M4 is where a fuse over curved crossings is the first
  consumer that has to merge the two.

`NamedRun`'s ends are `rcad_predicates::Point`, so they may be a box rather than
a point. That is what makes §26.6 item 4 reachable, and it is deliberate: an end
handed over as an exact triple would buy a certified answer to the wrong question
(§2.5).

### 27.4 §24.12's prediction is aimed at a divide M3 does not call

§24.12 predicts the money goes to `SideOfSurface` for sphere and cylinder and to
the local interval divide beneath it. Both were read at the gate.

`SideOfSurface` needs no work. What needs the care is its *caller*: §27.1's first
finding is exactly the way to get a sphere predicate right and a sphere
intersector wrong.

**There is no interval divide beneath `SideOfSurface`.** Neither `Sphere::side_of`
nor `Cylinder::side_of` divides at any of the three levels — they are products,
sums and differences all the way down. `quotient_enclosing` sits beneath
`BSplineCurve::evaluate_with_enclosure` and its surface twin, and §24.1 dropped
the free-form half of M3, so **nothing in this milestone calls it**. §16.8's "M3
is the first caller that will care" is right that a marcher divides ranges and
wrong that this is the divide it will use: the marcher's is a new one, over a
different expression. So half of §24.12's prediction is misaimed before the work
starts, and the honest scoring at M3's close is against the marcher's own divide
and against `SideOfSurface`'s callers.

### 27.5 Who walks an edge, and what crosses that seam

§26.6 item 1: the mesher walks an edge by stepping in a straight line through the
surface's own two coordinates, which is right for a pipe's ruling, a cap circle
and a ball's meridian and wrong for the ellipse a tilted plane cuts out of a
pipe. Criterion 8 needs a solid whose edges were *computed* to mesh, so this is a
seam between M3's two stages and M2's mesher, and §13 rule 4 says settle it now.

**Settled: the evaluator in `rcad-geom` walks every edge, and `rcad-topo` walks
none.** Two authorities on where the points along an edge are is the shape §23.8
kept finding, and the mesher's straight line is one of them.

What crosses the seam:

- The mesher hands over a **`NamedRun`** — the edge's own run and its two ends,
  which §27.3's test shows every edge the kernel builds already is — together
  with **the surfaces of the two faces either side** (the run names the second
  surface only when the faces do not), and a **`Deflection`**, a `&Policy` and a
  log.
- It gets back **3D points**, in the edge's stored direction, ends not included,
  and one `Approximation` blaming `Cause::PointAlongACrossing`.
- It does **not** get a `(u, v)`. A shared edge lands in two different coordinate
  systems and the mesher already reads each with `Surface::coordinates_of`. That
  is what keeps the seam narrow: everything downstream of `walk` works from 3D
  points and needs no change at all.
- §23.4 is unchanged: the points are worked out **once** and the identical values
  go to both faces. The evaluator is the single authority; neither face
  re-derives.

Nothing of this is built at the gate. What the gate checked is that the types can
say it: `every_edge_the_kernel_builds_is_already_a_named_run`.

### 27.6 §26.6's six items, sorted

| | | |
|---|---|---|
| 1 | the mesher cannot walk a curved edge | **gate** for the seam, stage for the code — §27.5 |
| 2 | a ball's seam longitude is not read | **gate**, done — §27.7 |
| 3 | a new `Surface` variant is nine stops | **stage**, and nothing is due: §24.6 keeps the surface set fixed |
| 4 | `Undecidable` out of `mesh` becomes reachable | **stage**; the gate only had to make the ends of a run able to be a box, and they are |
| 5 | the evaluator bound over-charges a pipe on its own axis | **stage**, and only if a stage hands the mesher a surface that did not come from a `Model` |
| 6 | `measure` still asserts `six_volumes > 0.0` | **stage**, watched: §26.6 says to look again when a new kind of surface arrives, and M3 brings none |

### 27.7 What the gate found that the plan did not

**A pole set the walk's starting longitude, and a sound ball could not be
meshed.** Reading the meridian out of `EdgeRun` (§26.6 item 2) turned up a second
defect the first one was hiding. A ball's wire opens at a pole; a pole names
every longitude, so `coordinates_of` answers `atan2(0, 0)`; the walk took that
zero as where it started and unwrapped every corner after it towards longitude
zero. Half a turn away that is exactly the boundary the wire check refuses at, so
a ball seamed down the far half of its own meridian — a perfectly sound solid —
came back `AWireWillNotLieFlatOnItsFace`. A pole no longer starts a walk, and the
turns counted between poles are shifted rather than dropped. That is §23.8's
pattern for the eighth time, and the new kind of input was neither a solid nor a
number: it was **a field that had never been read**.

**The witness on the equator had to be handed over as a box.** Working out which
half of a meridian a seam is on means asking a half-space about a point on the
equator, and that point comes out of trigonometry. Given to the predicate as an
exact triple, `sin(π)` coming back `2.4e-16` instead of zero is enough to certify
a confident sign about noise and pick a half on the strength of it — §15.5's
mistake by a new road. Given the box it is really known to be inside, the
predicate answers `Undecidable`, neither half is ruled out and the mesher refuses.
`Error::ASeamBetweenTwoPolesNamesNoMeridian` is that refusal, and it has to be a
refusal rather than an assert because `validate` cannot catch it: both ends of a
pole-to-pole seam satisfy every half-space its true meridian satisfies, so
`check_edges` has nothing to look at.

**`Measures` had three arms and the pair table needs four.** §27.2 item 1. Worth
naming separately because it is the one gate item that would otherwise have been
taken *inside* the frozen window, which is what §15.9's precedent is about.

### 27.8 The numbers, and corrections to earlier sections

| | at §26 | now |
|---|---|---|
| tests (`cargo nextest`) | 860 | 878 |
| `FORMAT_VERSION` | 1 | 2 |

Corrections:

- **§24.4 item 1**: `EdgeRun` had to move to `rcad-geom` for `NamedRun` to hold
  one, and it could. §24.9 did not say where it would land.
- **§24.9**: the curve list is a `Vec`; `RightRoundWhereItsFacesCross` keeps
  carrying nothing and the gap is named instead; `FaceMeet` is M4's to merge.
  See §27.3.
- **§24.10**: "there is no near-tangency band for exact inputs" is true and
  understated — the naming half has no constructed input at all, so the only
  refusal it has is exact arithmetic running out of range. See §27.1.
- **§24.10 again**: the discriminants are not the textbook ones. They carry `n·n`
  and `w·w`, or the intersector and `SideOfSurface` are certified about two
  different solids.
- **§24.12**: `SideOfSurface` has no interval divide beneath it and M3 never
  calls the one §16.8 measured. See §27.4.
- **§26.4**: the doc-test count is 19 by `cargo test --workspace --doc`, not 24.
  It has not moved at this gate.

---

## 28. What M3's first stage cost, recorded 2026-09-01

*The four pairs whose crossing lies in a plane, named exactly, plus the
evaluator that puts certified points along one. §27 is the model for the shape:
a stage and not a milestone, so it is short. The marcher, the fuzz target, the
cost benchmark and the claim audit are the second stage's and the closing
gate's.*

### 28.1 §27.1's prediction was right, and it moves the headline to zero

§24.10 called the rotated refusal rate "the headline" and expected it above
zero. §27.1 argued, **before any of this was written**, that there is nothing
for a turned pair to be undecided about: `intersect` takes two `Surface`s, a
`Surface` is made of `f64`, every discriminant is a polynomial in those, and
level 3 decides. Measured, on 2048 pairs per family:

| Family | `Error::Undecidable` | Second refusal | Threshold recorded |
|---|---|---|---|
| lattice — the control | **0.00%** | 0.00% | zero, asserted |
| turned — the headline | **0.00%** | 0.54% | zero, asserted |
| leaning whole-number directions | 0.00% | 0.63% | zero, asserted |
| written at `2^k`, `k` from −620 to 620 | 10.50% | 0.00% | 25% |

**Criterion 6's threshold is zero**, recorded here and asserted in
`how_often_naming_a_crossing_refuses.rs`. That is a strong assertion on purpose:
if a change ever makes a turned pair refuse, the answer is to find which product
left the range and not to raise the number.

Three predictions were scored before this stage and two were right; this is the
fourth, the second made in advance, and it holds.

### 28.2 There are two refusals and §24 predicted one of them

The first is the one §27.1 named: exact arithmetic declining a product that
would round its low half onto the subnormal grid, or overflow. Measured, one
pair of each kind: every family answers from `2^-538` to `2^509`, and two planes
answer everywhere, because their discriminants multiply two direction components
or two offsets rather than a radius by itself. A **gap**, demonstrated and not
argued — all 215 refusals in the shipped corpus are named the moment the same two
surfaces are written back on the lattice.

The second was not on anyone's list. **A plane exactly along a pipe's axis and
inside it cuts two lines, and telling them apart needs a `Plane` whose unit
normal is exactly square to both the axis and the cutting plane's normal.**
`Plane` scales every normal to length one and that division rounds, so the
normal we can write down is exactly square to both only sometimes — and a *line*
is endless, so a half-space it only stays inside near the origin has not picked
it at all. `Error::CannotNameWhichPieceOfACrossingThisIs`, 0.5% of turned pairs
and none at all on the lattice, where that direction is a coordinate axis.

Also a **gap**, and the families of fixes weighed: a `Plane` that stores its
normal unscaled, which is a second kind of plane for every predicate in the
kernel to be certified about and so M4's business; a curved half-space, which
buys nothing because `Cylinder` normalizes its axis the same way; and testing
the separation instead of arranging it, which compares the picking plane's own
offset rounding against the half-distance between the two lines and is a sign
question of about the fourteenth degree. **The third is the fix and it needs the
second stage's expansion buffer**, so it lands there.

### 28.3 Two ends is the wrong shape for a crossing of two surfaces

§24.3 says a crossing comes back as "two surfaces, the half-spaces that pick one
piece, and two ends", and §24.9 gives every `NamedRun` a pair of ends. Writing
`intersect` found that **a pair of surfaces names a curve and nothing on it**.
The circle where a plane cuts a ball has no start; the line where two planes
cross has no end at either side. Ends are what *topology* adds — the vertices
that cut a curve into edges — and an intersector handed two surfaces has none to
give.

Two of the four pairs answer with an endless line, and one of those is criterion
4's own row ("a plane tangent to a cylinder gives one line"), so this is not a
corner case the type could be left silent about.

`NamedRun::the_whole_of` is the answer and it widened nothing: both ends come
back as the box that says *nothing at all is known about where this is*. That
can never be confused with an end a predicate gave up on, because §24.10 refuses
such an answer whole — an undecided end comes back as `Err` and never inside a
run. `EdgeRun` did not grow a variant and the format did not move.

### 28.4 "Certified ON" has to mean "never certified OFF"

§24.7 calls criterion 3 one sentence: *every point the evaluator produces must be
certified ON both surfaces by `SideOfSurface`*. Taken literally it is
unreachable, and not because the evaluator is weak. A point on the circle where
a plane cuts a ball is at coordinates no `f64` can name, so `Decision::Zero`
about it is not something any amount of precision reaches; handed over as an
exact triple it gets a certified `Pos` or `Neg`, which is §2.5's exact answer to
the wrong question.

The reachable form is the one
`a_point_exactly_on_a_circle_is_never_certified_off_it.rs` already argues one
milestone down, and it has two halves rather than one: handed over as **the box
it is really known to be inside**, no point may be certified *off* either
surface, and separately every point must be certified *off* a surface it is
plainly not on. The second half is what stops the first passing for an evaluator
whose bound is a mile wide.

Measured over about thirty thousand points on three families: the furthest any
point sits off its own curve is **1.2% of the bound published for it**.

### 28.5 The published bound needed two steps that are not roundings

`POINT_ALONG_A_CROSSING_ROUNDOFF` is `256 · f64::EPSILON` of the largest number
involved, against a derivation of about `34 · f64::EPSILON` — the same seven-fold
room `Surface::ROUND_TRIP_ROUNDOFF` leaves next door, and for the same reason,
that Rust does not say how accurate `sin`, `cos` and `atan2` are.

Two steps of the chain are things that had to be *put there* rather than counted,
and without either the constant is not a bound at all:

1. **A nearly-tangent circle's radius comes from a difference of two large
   squares.** `n·n · R² − (n·c − d)²` in `f64` cancels to noise, and the radius
   that comes out is wrong by about `ε·L²/radius`, which grows without bound as
   the circle shrinks. Summed through the exact expansion instead — the very
   discriminant that named the crossing — it is right to a rounding or two and
   the bound stays *relative*.
2. **`axis × normal` for a plane nearly square to a pipe is tilted out of the
   plane** by about `ε / sin`, so every point built on it misses the plane by the
   radius times that. What gets used is the part of it lying in the plane, which
   leaves a tilt of order `ε²`.

Neither would have been caught by the drawn corpus. The first needs a tangency
approached rather than hit; the second needs two directions nearly but not
exactly parallel, which the lattice family cannot draw — turning a plane and a
pipe that were exactly parallel turns them by the *same* rotation and leaves them
exactly parallel, so the corpus has the two cases and not the band between them.
That is worth writing down as a shape rather than as an incident: **a generator
built out of exact configurations and one rotation reaches the on-case and the
far-off case and skips the near-case**, and §17.7 already says a nudge of `1e-9`
is not the answer.

### 28.6 Corrections to §24 and §27

- **§24.10, criterion 9.** "The same pair under a general rotation names the
  same kind of crossing" is not true of a pair on a knife edge, and should not
  be: every predicate here is exact about the numbers *as stored* (§17.1), and
  turning rounds them, so a plane that rested exactly on a ball genuinely no
  longer does. The knife edges are named, skipped and counted. The surprising
  one is not the tangency: **a plane not parallel to an endless pipe cuts an
  ellipse however far away it is**, so "parallel and well clear of it" answers
  `Nothing`, and the smallest tilt turns that into an enormous ellipse. There the
  knife edge is the whole of the `Nothing` answer rather than a near miss. 1485
  of 2048 pairs are compared and 564 skipped, which is high because the control's
  directions are all coordinate axes.
- **§27.2's fourth "weighed and not taken".** It declines a `Measures` arm for a
  dimensionless value on the grounds that "level 3 always decides such a
  question, so no record is ever written". It can be written: a direction with a
  component down among the subnormals underflows its own product, and "is this
  plane parallel to this axis" then gives up.
  `a_direction_with_a_subnormal_component_cannot_be_told_from_square_to_a_plane`
  is the demonstration. It is filed as `Measures::Length`, whose conversion is
  the identity, which is the *right* conversion for a value with no units; the
  **name** is wrong, and `Measures::Dimensionless` is one line at M4's gate.
  The freeze held: nothing in `rcad-predicates` moved.
- **§24.9's four arms are enough.** `Nothing`, `TouchAtOnePoint`, `Curves` and
  `SameSurface` cover every answer the four planar pairs give, including one
  ball swallowed by another and a plane clear of a pipe it runs alongside. The
  `Vec` in `Curves` is never longer than two.
- **§24.5's row for plane ∩ cylinder** lists "an ellipse; a circle, one line or
  two lines in special position". The circle needs no naming of its own — a
  circle and an ellipse are both one closed curve and neither is drawn, so they
  are the same answer. What does need its own path is the *evaluator*, where a
  plane square to the axis has no `axis × normal` to take a direction from at
  all, and the pipe's own `basis` is what stands in.

### 28.7 What went into the code, and what it cost

Four error variants in `rcad-geom` beyond the gate's `Undecidable`:
`CrossingRunsToAQuartic`, `CannotNameWhichPieceOfACrossingThisIs`,
`CannotPlaceAPointThatCloseToACrossing` and `CannotWorkOutHowBigACrossingIs`.
The first is a deferral said out loud, which is not the same thing as a wrong
answer; the middle two are the two refusals of §28.2 and §28.4's policy
question; the last is the gap again, met while working out how big a conic is.

**Level 1 of the cascade is deliberately not implemented for any of the new
questions.** A forward error bound is a derived claim about a polynomial and
§15.6 says a published bound needs a test; deriving and testing one for each of
seven questions buys speed on a path nothing has measured, which is the shape
§1.4 and §24.16 both turn down. Level 2 decides everything level 1 would have,
by construction rather than by derivation. If M4 measures the intersector and
wants the filter, the derivations are the work and they are not large.

**`Exact` is a heap-backed expansion**, which is §27.1's fix (i) landing one
degree below where §27.1 assigned it. `n·n · r² − w·w · (n·o − d)²` multiplies a
six-entry expansion by a ninety-eight-entry one, and a fixed buffer for that is
over a thousand `f64` on a level that runs on a small minority of calls. The
existing `square_of_expansion` is untouched: the two predicates that call it know
their own sizes, which is exactly the case `expansion`'s crate doc describes.

### 28.8 The numbers

| | at §27 | now |
|---|---|---|
| tests (`cargo nextest`) | 878 | 931 |
| suite duration | 16.6 s | 16.9 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | 2 |

The suite grew by fifty-three tests and by three tenths of a second, because a
crossing is a handful of sign decisions rather than a fuse — which is why the
measured rows here run 2048 configurations where `rcad-topo`'s run 128.

### 28.9 What the second stage inherits

1. **`Exact` is already there.** The eighth- and twelfth-degree discriminants of
   §27.1 can be written straight into it with no buffer arithmetic anywhere.
2. **The arms are in place.** `intersect` and the evaluator's
   `the_conic_between` each have a `CrossingRunsToAQuartic` arm holding the two
   quartic pairs, so the second stage fills them in rather than restructuring
   what the first shipped.
3. **`EdgeRun::RightRoundWhereItsFacesCross` still says "one closed curve"**,
   which is true of every crossing the four planar pairs produce and false of a
   quartic that leaves two loops. §27.3 named the two families of fixes and
   built neither; the second stage is the consumer that decides.
4. **The two-lines gap of §28.2** is fixed by a fourteenth-degree separation
   test, which needs the buffer the second stage is building anyway.
5. **The mesher does not call `walk` yet.** §27.5 settled who walks an edge and
   this stage built the evaluator, but swapping the mesher over changes how many
   points every curved mesh gets and moves the pole and seam handling down a
   layer. `every_curved_edge_the_kernel_builds_is_walked_into_points_on_both_its_faces`
   is the evidence that what is left is wiring: every curved edge the kernel
   builds already crosses that seam and comes back with points on both faces.
   Criterion 8 needs the wiring and a solid whose edges are computed, and both
   are the closing gate's.
6. **The marcher's own interval divide** is still the thing to measure, per
   §27.4, and `Interval::divided_by` is still `pub(crate)` in a frozen crate
   while `quotient_enclosing` in `rcad-geom` is now `pub(crate)` to that whole
   crate rather than to one module. That was the move §27.2 said would cost the
   freeze nothing, and it did not.

---

## 29. What M3's second stage cost, recorded 2026-09-01

*The two rows whose crossing is a degree-four space curve, plus the marcher that
draws what no conic stands in for. §28 is the model for the shape. Longer than
§28 is, because §24.17 asks this stage to make the milestone's largest claim
stand up and a claim of that size does not fit in a paragraph.*

### 29.1 The pair table is finished, and one of the six rows is finished twice

| Pair | What it answers | Where |
|---|---|---|
| plane ∩ plane, plane ∩ ball, plane ∩ pipe, ball ∩ ball | named exactly | stage one |
| ball ∩ pipe | **named exactly, in all five shapes**; the quartic ones marched | here |
| pipe ∩ pipe, axes along one direction | **named exactly** | here |
| pipe ∩ pipe, axes leaning | **named as far as an enclosure reaches** | here |

The ball-and-pipe row came out stronger than §24.5 wrote it. That section files
it as *"special cases named; the rest marched"*, and the special case it had in
mind is the ball threaded on the axis, which cuts two circles. What is actually
true is that **every** ball-and-pipe crossing is named exactly, quartic or not:
§27.1's eighth-degree discriminant settles which of five shapes it is, and the
marcher is needed only to *draw* the two that do not lie in a plane. Naming and
drawing came apart exactly where §24.3 said they would.

The five shapes, and the one that has no name:

```text
A = ‖(c−o)×w‖²    V = A + r² − W·R²    Δ = V² − 4r²A
```

`W(R² − (ρ∓a)²) = −V ± 2r√A`, so the pair `(sign V, sign Δ)` settles both
tangency discriminants without ever taking the root that stands between them.
Nothing, one touching point, one closed loop, two closed loops — and, where the
two loops touch, **one loop that runs through itself**, which
`EdgeRun` has no variant for and this stage refuses by name.

### 29.2 Two leaning pipes, and where the exact level stops

Two pipes whose axes lean against each other come down to one clean statement.
Walk the thinner pipe's cross-section round by an angle `θ` and ask how far
along its own axis the other pipe reaches; that is a quadratic, and its
discriminant is

```text
sin²α · [ ρ₂² − (δ + ρ₁·cos θ)² ]
```

with `α` the angle between the axes, `δ` the distance between them and `ρᵢ` the
certified radii. **The whole shape of the crossing is where `ρ₂` sits against
`δ ± ρ₁`** — above both and the fat pipe swallows the thin one's whole
cross-section and leaves two loops; between them, one; below both, two again
with the pipes' roles the other way round; past `δ − ρ₁` on the far side,
nothing at all. Three comparisons, and each is a **length against a length**.

That is why the exact level runs out here and nowhere else in the file. Those
lengths are `rᵢ/√Wᵢ` and `|T|/√K` for polynomials `T` and `K` in the stored
numbers — three square roots in three different metrics. Clearing them costs
eight degrees, and §29.6 measures what happens then. So the comparison is
**enclosed** instead, which decides it wherever the two sides are not on a knife
edge, and the exact level runs where the two stored directions have exactly the
same squared length — which is every pair whose axes are coordinate directions,
and so the whole of criterion 5's control.

### 29.3 The marcher, and why its bound is measured rather than derived

The closed form knows its per-point bound before it starts: a count of roundings
along a chain, published as `POINT_ALONG_A_CROSSING_ROUNDOFF` and multiplied by
the size of the shapes. The marcher cannot work that way, because how far one of
its points sits off the curve is something it *finds out* by settling the point.

So it measures. After each point is settled onto both surfaces by Newton in the
plane the two normals span, an enclosure of how far it really sits from each of
them is worked out, and the two are divided by the sine of the angle between the
surfaces. That is the radius of the region a point that close to both surfaces
can be in, and the largest of them along a run is what `walk` publishes.

**Which is why near-tangency refuses rather than drifting.** As two surfaces come
parallel that sine goes to nothing and the bound goes up without limit, so the
marcher does not quietly hand back points nobody can judge — it says how far out
they could be and lets the policy decide, and past the point where the sine
cannot be told from zero it refuses by name. Roadmap §6.3 calls near-tangential
contact the place kernels produce silent garbage; that is the branch §24.1 built
this to be able to watch, and it is the branch criterion 7 measures.

The step rule is the same shape. **How far it bends** is measured and not
derived: the middle of each straight piece is settled onto the curve and how far
it moved is that piece's sagitta, and a step that spends more than three
quarters of the deflection on it is halved and tried again. **How far it turns**
is capped at a quarter turn, because past a half turn a step cannot be told from
one the other way. Termination: every accepted step moves the point at least one
step along a curve of finite length, so a run takes finitely many — and because
that argument is about the curve rather than about the arithmetic, it is not
left to stand alone. `MOST_POINTS_ALONG_ONE_CROSSING` caps the count outright
and reaching it is a refusal rather than a truncated answer (§24.10).

### 29.4 Criterion 7, the load-bearing one: 0.577, and the same at every scale

§24.5 promised the oracle would exist by construction, and it does. A ball
threaded on a pipe's axis **is a ball and a pipe**, so the evaluator marches it
like any other degree-four crossing and nothing in it knows the answer is
special — and the answer is two circles of the pipe's own certified radius at
`±√(R² − ρ²)` along the axis, which is a formula anybody can write down. The
marcher is therefore differenced against a written-down answer and not against
another of our own implementations.

| | worst share of its own published bound |
|---|---|
| marched, drawn where they stand | **0.577** |
| marched, written at `2^k` for `k` from −200 to 200 | **0.577** |
| the same circles parametrized instead | 0.008 |

**Criterion 7's ceiling is three quarters**, recorded here and asserted in
`how_far_the_marcher_strays_from_a_crossing_that_is_a_circle.rs`. One would be
the point at which the published bound stops being a bound.

The gap between the two arms is not a weakness and it is worth saying why, or
the next person will try to close it. The parametrized arm publishes `256·ε`
against a derivation of about `34·ε`, so a drawn case uses under a hundredth of
it. The marcher publishes what it measured, and what it measured is
`(d₁ + d₂)/sin ψ` where the true distance to the crossing is
`√(d₁² + d₂² − 2d₁d₂cos ψ)/sin ψ`. For two surfaces meeting square on with equal
residuals those differ by exactly `1/√2 = 0.707`, and the measured 0.577 sits
just under it. **A ratio near 0.58 is the bound being nearly tight. A ratio near
zero would have meant the marcher was publishing a number it had not earned.**

That the figure is *identical* across four hundred binary orders of magnitude is
§16.8's argument for a ratio arriving as a measurement.

### 29.5 The numbers, family by family

Criterion 5's control and criterion 6's headline, on 2048 pairs per family.
Reported apart from stage one's four planar pairs for §17.7's reason: the two
kinds of pair refuse for reasons the other does not have.

| Family | `Undecidable` | no half-space | runs through itself | threshold |
|---|---|---|---|---|
| lattice, planar pairs — the control | 0.00% | 0.00% | 0.00% | zero, asserted |
| lattice, quartic pairs — **the control** | **0.00%** | **0.00%** | 4.25% | zero, asserted |
| turned, planar pairs | 0.00% | 0.44% | 0.00% | zero / 2% |
| turned, quartic pairs | **4.64%** | **10.93%** | 0.00% | 8% / 15% |
| leaning | 0.00% | 0.24% | 0.00% | zero / 2% |
| **the band** — drawn tangent, moved a few last bits off it | **0.00%** | 10.79% | 1.07% | 5% |
| written at `2^k` | 10.49% | 3.95% | 0.00% | 25% |

Criterion 3, over about a hundred and thirty thousand points:

| Family | furthest a point sat off its own curve |
|---|---|
| lattice / leaning / turned, parametrized | 0.006 / 0.008 / 0.012 of the bound |
| lattice / turned, quartic pairs, **marched** | 0.738 / 0.355 of the bound |
| the band | 0.410 of the bound |

Three things in those tables are worth reading twice.

**The quartic control is 0% and it did not have to be.** Two leaning pipes are
the one pair whose comparison is not a polynomial, and on the lattice the exact
level still reaches every one of them — because there both stored directions
have the same squared length and the awkward root is rational.

**4.64% of turned quartic pairs refuse, and every one of them is a knife edge
the turn made.** The generator draws radii up to five and places up to six apart
in whole numbers, so a lattice pipe pair is exactly tangent surprisingly often —
five apart with radii of two and three is one — and §28.6 already recorded that
turning such a pair leaves it genuinely no longer tangent, by a few last bits.
`every_turned_quartic_crossing_that_refuses_was_exactly_tangent_before_the_turn`
is the demonstration and not the paragraph: all 95 of them are settled by the
same code before the turn.

**The band refuses nothing for want of a sign, which was not the expectation.**
§29.7 says what closed it.

### 29.6 §24.17's structural claim, and what it turns out to be

§24.17 asks for the largest of M3's limit-or-gap claims to be made to stand up:
*"the general quartic has no exact naming"*. It is not one claim. It is three,
they have three different answers, and only one of them is a limit.

**One: naming which of the five shapes a degree-four crossing takes.** Not a
limit at all — §29.1 does it exactly for a ball and a pipe, at the eighth degree,
and §29.2 does it for two pipes wherever the arithmetic reaches. §24.17 expected
this to be the hard part and it was the easy one.

**Two: reaching that answer for two leaning pipes at any scale.** A **gap**, and
this is the one with a measurement behind it. The comparison needs
`M = r₁²W₂K + r₂²W₁K − T²W₁W₂` squared. Measured on two perfectly ordinary
turned directions and an offset of order one: `M` comes to **22 entries spanning
505 binary orders**, and squaring it asks for the product of its smallest entry
with itself, which lands a thousand orders down — past the bottom of `f64`, so
`Exact::times` declines and level 3 cannot run.

That is a different kind of refusal from every other one in this milestone, and
the difference is the point. §28.2's range refusals are cured by a rescale
because the *whole number* is too small; here the number is fine and its own
internal spread is too wide, and a spread is invariant under scaling. What a
scaling can still do is slide the doubled spread into the window `f64` leaves,
and `the_question_two_leaning_pipes_ask_is_too_wide_to_square_where_it_stands`
shows it doing exactly that: the same expansion written `2^300` higher squares
without complaint. So the answer exists, this arithmetic at this scale could not
reach it, and the fix is the working frame §24.8 defers for want of a figure to
beat. **A gap, and now the first thing with a figure asking for that frame.**

Weighed and put down, with the reason each was put down:

- **The pencil's characteristic polynomial and the Segre classification.**
  `det(λM₁ + μM₂)` for two quadric matrices is a degree-four binary form whose
  real root structure classifies the intersection, and its coefficients *are*
  polynomials in the stored numbers. Put down because reading a real
  intersection type off it is the Dupont–Lazard–Lazard–Petitjean machinery, and
  because the determinant of a 4×4 whose entries are already degree four is
  degree sixteen before anything is compared — the same wall as above, reached
  by a longer road.
- **Sturm chains on the trigonometric discriminant.** §29.2's `Dsc(θ)` is a
  trigonometric polynomial of degree two, so counting its sign changes is
  counting the real roots of a quartic in `tan(θ/2)`, which is textbook. Put
  down because writing that quartic down needs an orthonormal frame across the
  pipe, and `Cylinder::basis` normalizes — so its coefficients are not
  polynomials in the stored numbers and §27.1's load-bearing rule would be
  broken to get them. This one is worth remembering: it is not that the algebra
  is hard, it is that the frame is not exact.
- **An exact algebraic representation of the curve.** Would make the naming
  question trivial and the *evaluation* question no easier, because a mesher
  still needs points and points still round. §24.3 already refuses to carry a
  curve beside the surfaces that name it, and this would be that with more
  degrees.
- **The Steiner special cases.** Two of the three are built — coaxial and
  parallel axes are §29.1's row. The third is not, and see below.

**Three: a crossing that runs through itself.** Two equal pipes whose axes meet
leave two ellipses that cross each other twice; a ball reaching exactly across a
pipe's whole width leaves two loops that pinch together at one point. Both come
back `Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo`, and classifying
that refusal took two goes, because the obvious answer is wrong.

The obvious answer is that `EdgeRun` cannot say it. That is **false**, and saying
so and moving on would have been the paragraph §24.17 warns against. The two
loops of a pinch are exactly `{n̂·(p − c) ≥ 0}` and its opposite for the plane
through the ball's centre square to the axis, which is one `picked_by` each; the
two ellipses are exactly the parts lying *in* a bisector plane, which is a pair
of opposite half-spaces, and `picked_by` takes as many as it is given. The
vocabulary is sufficient. What is not reachable is writing either plane down —
the same `Plane`-stores-a-unit-normal gap §29.8 is about, and on the lattice,
where those directions are coordinate axes, even that is reachable.

**The real reason to refuse is the evaluator, and it is demonstrable.** At the
point where a crossing runs through itself the two surfaces are *tangent*, so the
crossing has no direction there and the marcher has nothing to follow:
`marching_from_where_a_crossing_runs_through_itself_has_no_direction_to_follow`
is that, in four lines. §24.10 says a partly undecidable answer refuses whole,
and naming two pieces the evaluator cannot draw is that shape exactly — so the
refusal happens where it can still be one refusal instead of two half-answers.

So: a **limit** of the naming half taken together with the evaluator, and a
**gap** of the naming half on its own. The family of fixes is a marcher that can
be told to start away from a singular point and stop at it, plus a third
`EdgeRun` variant that hands a fuse the crossing point rather than making it
notice. Both have one consumer and it is M4's fuse.

### 29.7 The expansion buffer cost almost nothing, and the useful thing was elsewhere

§27.1 filed a schedule risk here: the eighth- and twelfth-degree discriminants
need an expansion squared, `square_of_expansion` caps at eight entries, and a
pipe's `‖(c−o)×w‖²` runs to 384 with a worst case around 2.6 MB. §28.7 then
landed the fix a degree early — `Exact` is heap-backed — and left this stage
with nothing to build.

Measured, so that the risk is closed with a number rather than a shrug. For a
ball against a pipe at ordinary sizes: `A` is **8 entries**, `V` is **5**, `V²`
is **16**, `r²A` is **4**. The worst case a fixed buffer would have had to hold
is thousands; the drawn case is tens, because zero elimination keeps expansions
short whenever the terms are of similar size — which §27.1 predicted and nobody
had checked. **The buffer cost one allocation on a level that runs on a minority
of calls, and no refusals at all.**

What did turn out to matter was the opposite move: `Exact::enclosure`, which
sums an expansion's entries in interval arithmetic instead of taking the nearest
`f64`. An expansion's entries do not overlap, so enclosing it costs a rounding
or two **of the answer**, where the same expression built out of intervals from
the start carries a rounding of the *inputs*. Two axes a last bit from parallel
cross into something whose every component cancelled away: enclosed from the
expansion, `‖w₁ × w₂‖²` is known to full relative accuracy; built from
intervals, it reaches down to zero, and the distance between the axes — which
divides by its root — comes back as everything. That one change took the band
family from refusing 0.2% to refusing none.

### 29.8 The half-space gap: half of it closed, and the other half is `Plane`'s

§28.2 left `CannotNameWhichPieceOfACrossingThisIs` with a named fix: *"testing
the separation instead of arranging it"*. Instrumented before the work, stage
one's refusals broke down as: 13 of 24 failed because the picking normal was not
exactly square to the endless direction, and 11 of 24 failed only on the two
conditions that did not have to be exact.

So the fix was applied where it works and it closed that 11. A picking plane no
longer has to be square to the cutting plane's normal, nor to pass through the
axis with nothing rounded; where the two lines sit either side of it is now a
comparison between their own separation `√(ρ² − g²)` and how far the plane
strayed. The same test, in its bounded form, is what tells two loops of a
ball-and-pipe crossing apart — there the pieces sit on a sphere of known radius,
so what could move the reading is bounded by `‖n̂ × ŵ‖·R` and the comparison is
ordinary.

**The other half stays, and it is a gap.** For a piece that runs endlessly along
a direction — two lines down a pipe, two loops running the length of a fat pipe —
the picking normal must be exactly square to that direction or it reads a
different number at either end, and testing cannot stand in for arranging when
the thing being tested is unbounded. Every `Plane` constructor scales its normal
to length one and that division rounds, so **which normals come out exactly
square to a given direction is not something a caller chooses**; on the lattice
it is a coordinate axis and comes out right, and off it, sometimes.
`two_lines_no_f64_half_space_can_tell_apart_are_refused_rather_than_guessed`
is the demonstration that it is a gap and not a limit: the same configuration
written on the lattice is named without complaint.

§28.2's first family is the fix — a plane that stores its normal unscaled — and
it is a change to a type every predicate in the kernel is certified about, so it
is M4's. Measured cost of leaving it: 0.44% of turned planar pairs, 10.9% of
turned quartic ones.

### 29.9 Criterion 10: what runs in CI and what does not, said plainly

`cargo-fuzz` builds with a sanitizer that needs a nightly compiler and
`rust-toolchain.toml` pins a stable one, so **`just default` does not run the
fuzz target**. `just fuzz` does, on a compiler the repo does not pin, and
`mise.toml` now pins `cargo-fuzz` itself. Two things had to be worked around and
both are written into the recipe: `cargo-fuzz` sets `RUSTFLAGS` as an
environment variable, which shadows the `.cargo/config.toml` entry `inari` needs
to build at all (§2.4), and the shared property module is reached with `#[path]`
rather than `include!` because it opens with a module doc comment.

What runs on every commit is the same property over the same decoder:
`every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs` replays the
checked-in corpus and soaks two hundred thousand pseudo-random byte strings,
about seven hundred thousand points, in five seconds.

**Run duration: 901 seconds, 23 037 787 executions at about 25 600 a second**,
corpus grown and then minimised to 562 inputs — 530 the fuzzer kept and 32
written out by hand and named for the configuration each one is. It found two
things in its first five minutes, both real, and nothing in the fourteen minutes
after them:

1. **A conic drawn near the top of `f64`'s range published a bound of infinity**,
   and `Approximation::new` asserts a magnitude is a real amount. Now
   `Error::CannotWorkOutHowBigACrossingIs`, which is the same refusal the naming
   half has one step earlier.
2. **`where_two_balls_cut` put the circle's centre nowhere near its own
   published bound**, for two balls of almost the same size whose centres are
   almost in the same place. `D² + R₁² − R₂²` cancels to a millionth of either
   square and is then divided by a `D²` smaller still, so a relative rounding of
   the difference comes out of the division magnified by however many times `D`
   goes into the radius. Summed through `Exact` like the radius beside it, which
   is what §28.5's second finding had already done for the radius and nobody had
   done for the centre. `two_balls_of_almost_one_size_almost_in_one_place` is
   the seed, checked in.

Both are stage one's, both are in the band §28.5 named, and neither was
reachable by any generator in the suite. That is the argument for criterion 10
having been worth the trouble, and it is a measurement rather than a hope.

A third came out of the soak before the fuzzer ran: `quotient_enclosing`
asserts its divisor is away from zero, which is right for the spline evaluator
where every caller has proved it and wrong for an intersector whose divisors are
lengths a caller handed over. A direction whose squared length underflows is bad
input and not a broken invariant, so it now divides into a box that says nothing
is known and turns into a refusal (§4.1).

### 29.10 Criterion 11: the divide is 4% of a marched point, and §24.12 was wrong

Human-read, no target, and labelled as such — §24.16 said this should not need
writing a third time and this is the third time.

| | |
|---|---|
| one certified marched point | **2.58 µs** (643 µs for a 249-point run) |
| one parametrized point | 33 ns (2.3 µs for a 70-point run) |
| naming the whole crossing, drawing none of it | 1.22 µs |
| one interval divide | 11.3 ns |
| **the interval divide's share of a marched point** | **9 divides × 11.3 ns = 4%** |

So **§24.12's prediction is scored and it is wrong in both halves.** §27.4
already found the first half misaimed — neither sign predicate divides at any
level. The second half said the money would go to the divide beneath it, and
measured against the marcher's own divide, which is the one §27.4 said to score
it against, it is 4%. The money goes to settling: two Newton corrections per
trial step, each of them six dot products and a two-by-two solve, and a certified
enclosure of the deviation on top.

Marching costs about **eighty times** what parametrizing the same curve costs.
That is the price of criterion 3 holding for a curve nobody can write down, and
it is a number M4 can decide whether to spend.

### 29.11 Corrections to §24, §27 and §28

- **§24.5's ball-and-pipe row** says "special cases named; the rest marched".
  Every ball-and-pipe crossing is *named*; only two of the five shapes need
  marching to be *drawn*. See §29.1.
- **§24.5's pipe-and-pipe row**, same sentence, and here the correction goes the
  other way: the coaxial and parallel-axis special cases are named exactly, and
  the third one §24.5 lists — equal radii with crossing axes — **cannot be
  named at all**, because what it produces is one curve that runs through
  itself. See §29.6.
- **§24.5's claim that the oracle exists by construction** holds, but not for
  the reason it gives. It is the ball threaded on the axis that provides it, not
  the equal-radii pipes.
- **§24.12's prediction is wrong in both halves.** §29.10.
- **§24.17's structural claim splits three ways**, and only one third of it is a
  limit. §29.6.
- **§27.1's schedule risk did not materialise.** The expansions that had to be
  squared came to tens of entries and not thousands. §29.7.
- **§27.2's "a public `Interval` divide" was weighed and not taken**, on the
  grounds that `rcad-geom` has `quotient_enclosing` of its own. True, and it
  turned out that function's *contract* was the thing that needed changing
  rather than its visibility. §29.9.
- **§28.2's fix closes half the gap** and the other half is a limit of `Plane`
  rather than of the arithmetic. §29.8.
- **§28.5's finding is now a generator.** `a_crossing_a_hair_from_the_knife_edge`
  draws seven ways two of these surfaces can rest against each other, exactly
  tangent in whole numbers, then moves the second one a handful of its own last
  bits off it and optionally leans it by `2^-k`. Both halves are relative, for
  §17.7's reason.
- **§28.7's `Exact` is the right shape and was missing one method.** §29.7.

### 29.12 The numbers

| | at §28 | now |
|---|---|---|
| tests (`cargo nextest`) | 931 | 973 |
| suite duration | 16.9 s | 18.1 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | 2 |
| fuzz corpus | — | 562 inputs |

Forty-two tests and a second and a fifth, and most of that second is the
marcher: a marched point costs eighty times a parametrized one, and three of the
measured rows walk tens of thousands of them.

### 29.13 What the closing gate inherits

1. **Criterion 8 is not done**, and §28.9 item 5 already said why: the mesher
   still walks its own edges, and swapping it over to `walk` changes how many
   points every curved mesh gets. A curved solid whose edges are *computed*
   needs that wiring plus a builder that takes two surfaces instead of an edge.
2. **Criterion 12, the claim audit**, has not run. §29.6 is written to be
   audited rather than to pass one.
3. **The two `EdgeRun` gaps are now one.** §27.3 left
   `RightRoundWhereItsFacesCross` saying "one closed curve" with a quartic's two
   loops as the counterexample; those are named as two `PartOfWhereItCrosses`
   runs, so the variant's claim is true again. What is left is the curve that
   runs through itself, and §29.6 says its consumer is M4's fuse.
4. **A working frame now has a figure asking for it**, which §24.8 said was the
   condition. It is the 505-order spread of §29.6, and the ask is specific:
   squaring the two-pipe discriminant needs the expansion slid up the range, not
   the inputs centred.
5. **`Plane` storing an unscaled normal has a measured cost of not doing it**:
   10.9% of turned quartic pairs. That is §28.2's first family with a number
   against it for the first time.
6. **`Measures::Dimensionless` is still one line**, per §28.6, and the leaning
   pipes' three comparisons are lengths, which is the right arm and the right
   name.

---

## 30. What M3 actually cost, recorded 2026-09-01

*Written by M3's closing gate. §12 did this for M0, §16 for M1, §17.8 for the
fuse and §26 for M2. §24.12's shape was "gate, two stages, gate"; this is the
second gate.*

M3 is complete. `rcad-geom` names where any two of a plane, a ball and a pipe
cross, refuses by name where it cannot, walks certified points along the answer,
and marches the two rows no conic stands in for. The mesher no longer walks its
own edges. A solid whose edges were **computed** meshes and passes §23.4's
oracle, and one of them is a shape M2 could not declare at all. The suite went
from 860 tests at M2's close to 985.

`rcad-predicates` was touched once, at the gate, for `Measures::ManyLengths` and
`Cause::PointAlongACrossing`. §24.4 item 4's freeze held through both stages and
through this gate.

### 30.1 The fourteen criteria, and the test that discharges each

§17's table of "criterion → the test that discharges it" is the precedent, and
it is the most useful thing these sections contain. Every row names a test that
fails if the claim stops holding.

| | Criterion | Discharged by |
|---|---|---|
| 1 | the four planar pairs name their crossing exactly | `two_planes_leaning_against_each_other_cross_in_one_endless_line`, `a_plane_through_a_ball_cuts_one_circle`, `a_plane_that_leans_across_a_pipe_cuts_one_closed_curve`, `two_balls_that_overlap_cross_in_one_circle` |
| 2 | every named run passes `validate`'s check | `every_point_of_a_picked_run_is_inside_every_half_space_that_picked_it` — the whole run and not only its two ends — with `an_edge_named_by_a_crossing_it_is_not_on_is_caught` and `an_edge_outside_a_curved_half_space_that_picked_it_is_caught` for the check itself |
| 3 | every evaluated point certified ON both surfaces | `every_point_along_a_crossing_is_on_both_surfaces.rs`, six families, about 130 000 points |
| 4 | `TouchAtOnePoint` and `SameSurface` are answers; roadmap §7.4 is a table | `the_degeneracies_a_crossing_has_to_answer.rs`, fourteen rows, one test each |
| 5 | lattice control: **0%** | `a_certified_naming_of_a_lattice_crossing_never_refuses` and `a_certified_naming_of_a_quartic_lattice_crossing_never_refuses`, both asserted at zero |
| 6 | rotated rate measured and under a recorded threshold | `how_often_naming_a_crossing_refuses.rs`, five families, thresholds in §28.1 and §29.5 |
| 7 | marcher against closed form, as a ratio | `a_marched_circle_stays_inside_the_bound_the_marcher_published_for_it` and `..._written_anywhere_in_f64_stays_inside_the_same_share_of_its_bound` |
| 8 | a computed-edge solid meshes and passes §23.4's oracle | `a_bar_whose_cap_circles_were_computed_meshes_exactly_as_the_declared_one_does` and `a_bar_cut_off_at_an_angle_meshes_into_a_closed_surface`, both through `the_mesh_closes_up`; `a_bar_cut_off_at_an_angle_is_built_and_meshed_from_the_front_door` out at the facade |
| 9 | transform invariance | `turning_a_pair_of_surfaces_names_the_same_kind_of_crossing` **and** `turning_a_quartic_pair_of_surfaces_names_the_same_kind_of_crossing` — the second added at this gate, see §30.4 |
| 10 | a `cargo-fuzz` target over `(Surface, Surface)`, corpus checked in | `fuzz/fuzz_targets/where_two_surfaces_cross.rs`; on every commit, `every_seed_in_the_checked_in_corpus_names_its_crossing_honestly` and `a_soak_over_pseudo_random_bytes_names_every_crossing_honestly` |
| 11 | cost per marched point, divide broken out | `what_a_marched_point_costs.rs`. Human-read, no target (§24.16) |
| 12 | the claim audit passes; every refusal classified | §30.4 and §30.5 |
| 13 | `FORMAT_VERSION` at 2 with a round trip | `a_seam_picked_out_by_a_pipe_comes_back_picked_out_by_the_same_pipe`, `a_run_the_intersector_hands_back_writes_out_and_reads_in_unchanged`, `a_plane_and_a_surface_holding_that_plane_are_different_bytes` |
| 14 | the facade exports the intersector, `SurfaceMeet`, `NamedRun`, `EdgeRun`; `FaceMeet` stays private | `naming_a_crossing_from_the_front_door.rs`: `the_hardest_edge_the_kernel_builds_can_be_said_in_this_vocabulary`, `every_way_two_surfaces_can_meet_is_something_a_caller_can_match_on`, `a_caller_out_here_can_cross_two_surfaces_and_walk_the_answer` |

**All fourteen are met.** Two of them were not when this gate opened: criterion 8
had no builder and no wiring (§29.13 item 1), and criterion 9 was discharged for
four of the six pairs and said it was discharged for all of them.

One thing handed to M3 and **not** done, which is not a criterion and is said
here rather than buried: §26.6 item 4 predicted that `Undecidable` out of `mesh`
would become reachable when a mesh domain first held a constructed corner, and
named M3 as the milestone that could write the test. It cannot. `walk` hands the
mesher 3D points and the mesher reads them back through `Surface::coordinates_of`,
so every corner the triangulator sees is still an ordinary `f64` pair. The first
constructed corner in a mesh domain is a fuse's, and that is M4's.

### 30.2 §24.12's prediction, scored: wrong in both halves

§24.12 named `SideOfSurface` for sphere and cylinder, and the local interval
divide beneath it, as where M3's money would go. It was recorded before the work
on purpose.

- **First half, misaimed before a line was written.** §27.4 read both functions
  at the gate. `SideOfSurface` needed no work at all; what needed care was its
  *callers*, because a discriminant written the textbook way is certified about a
  different ball than the predicate is (§27.1).
- **Second half, wrong by measurement.** There is no interval divide beneath
  `SideOfSurface` — neither sphere nor cylinder divides at any of the three
  levels. §16.8's divide is the spline evaluator's and M3 never calls it. Scored
  against the divide the marcher actually built, which is what §27.4 said to
  score it against: **9 divides at 11.2 ns is 4% of a 2.66 µs marched point**
  (§29.10, re-measured at this gate).

The money went to **settling**: two Newton corrections per trial step, six dot
products and a two-by-two solve each, and a certified enclosure of the deviation
on top.

**The running score.** §26.2 corrected this once already and it stands: §12.1
stated its lesson after the fact and predicted nothing; §17.6 predicted and was
right; §23.7 predicted and was half right, in the sense that acting on the
prediction is what stopped it coming true. §24.12 is the fourth and it is wrong.
**Three predictions made in advance, one right, one half right, one wrong.**

**What the project should conclude, and it is not "stop predicting".** §12.1's
lesson is that the money goes to code that was already written and looked
finished. That held for M0 and M1 and half held for M2, and every one of those
milestones spent most of its time *rewiring* code that existed. M3 wrote
`intersect.rs`, `walk.rs` and `march.rs` from nothing, and the money went to the
new code — to a settling loop that did not exist when the prediction was made.
So §12.1's lesson has a precondition nobody had written down: **it is a claim
about milestones that mostly rewire, and a milestone that mostly writes will not
obey it.** The prediction is what exposed that, which is the second-best thing a
prediction can do.

The other half of the conclusion is cheaper and more immediately useful. Half of
§24.12 was scored *at the gate*, by reading two functions for an hour, before any
of the work it was about. A prediction made in advance is worth what the next
gate can spend checking it against the code, and that checking is far cheaper
than the prediction is accurate. Keep making them; check them at the next gate
rather than at the milestone's end.

### 30.3 What §24 got wrong, and what a settling document is for

Gathered rather than re-derived. §24 was written before anybody had read the
discriminants, and it shows in a particular way.

- **The discriminants are not the textbook ones** (§27.1). They have to carry
  `n·n` and `w·w`, or the intersector and `SideOfSurface` are certified about two
  different solids.
- **There is no near-tangency band, because there is no constructed input**
  (§27.1). §24.10 said the band exists only for constructed inputs; sharper, the
  naming half has none at all, so criterion 5's 0% is structural rather than
  hopeful.
- **"Two ends" is the wrong shape for a run** (§28.3). A pair of surfaces names a
  curve and nothing on it; ends are what topology adds.
- **"Certified ON" is unreachable as written** (§28.4). It has to mean "never
  certified off, and certified off a surface it is plainly not on", which is two
  halves rather than one.
- **§24.5's ball-and-pipe row is entirely nameable** and its pipe-and-pipe row
  less so (§29.11). Every ball-and-pipe crossing is named exactly; the
  equal-radii pipe pair §24.5 lists as a named special case cannot be named at
  all, because it produces one curve that runs through itself.
- **Criterion 7's oracle came from a threaded ball, not from equal pipes**
  (§29.11). §24.5's claim that the oracle exists by construction holds, for a
  different reason than it gives.
- **§24.17's structural claim splits three ways** (§29.6), and only one third of
  it is a limit.
- **Criterion 9's wording outran its generator**, and that one is this gate's.
  §30.4.

**Every one of those except the last is about arithmetic, and every one of them
cost almost nothing.** Four were found in a day at the gate before any code, two
while writing the code, and two by the answer coming out better than promised.
That is §26.3's split arriving a second time: a settling document is wrong about
numbers and right about meanings, and the numbers are cheap to be wrong about
because the first person to need one does the arithmetic. §26.3 ended with *"It
should stop suggesting numbers"*, and §24 suggested numbers anyway. Two
milestones is enough: the convention should be a rule and not an observation.

**The one that cost something is the one that reads like a test.** Criterion 9
says *"the same pair under a general rotation names the same kind of crossing"*.
That is not a claim about a number, so nobody did arithmetic on it; it is a
specification of a property, a property with that name existed, and the property
was green. It survived a gate, two stages and a claim audit's worth of review,
and what found it was enumerating what the generator draws. **The dangerous
sentences in a settling document are the ones that specify a test rather than
claim a number**, because a number gets checked by the first person who needs it
and a test that already exists does not.

### 30.4 What the claim audit found

The audit (`claim-audit`, per §16.10) ran over M3's seven new files in
`rcad-geom` and the mesher wiring in `rcad-topo`. §24.17 said M3 makes more of
these claims than any milestone so far, and it does. Three defects and four
sentences.

**Defects.**

1. **Criterion 9 covered four of the six pairs.** `a_lattice_crossing`'s own doc
   says a ball crossing a pipe and a pipe crossing a pipe are drawn by
   `a_quartic_pair` instead, and nothing was handing that strategy to the
   invariance property. A **sufficiency** claim, broken by enumerating what the
   generator ranges over. The quartic pairs have one knife edge the planar ones
   do not — a crossing that runs through itself is a tangency, so a turn moves it
   off — and it is skipped alongside the others. Measured: **1594 of 2048
   compared, 455 skipped**, against 1485 and 564 for the planar family.
2. **`walk` panicked on a caller's argument.** A run's ends are `Point`s, so an
   end may be a box; the box whose every side runs the whole number line is how a
   pair of surfaces says it names a curve and nothing on it, and **every other
   end has to have a middle**. `NamedRun::new` takes an exact triple with a `NaN`
   in it, and a box with one infinite side, and both make the angle round the
   conic not a number — down to `Interval::new(NaN, NaN)` and a panic. An
   **unreachability** claim by silence: `walk` is public, `NamedRun::new` is
   public, and nothing between them narrows what an end may be, which is §16.10's
   rule about checking the public API rather than today's callers.
   `Error::AnEndOfARunNamesNoPlace`.
3. **A refusal the second stage made unreachable and left behind.**
   `CrossingRunsToAQuartic`'s doc said the evaluator still raised it. It does
   not: `walk` matches the three quartic pairs and hands every one of them to the
   marcher before the closed form is reached, so the arm that produced the
   variant covers exactly the set that never arrives. Removed; the arm is a panic
   naming the invariant.

**Sentences, corrected rather than fixed.**

4. **"Fifty halvings puts any step below the smallest normal number"** is out by
   twenty binary orders: `2^-50` of a step of one is `9e-16` and the smallest
   normal number is `2e-308`. Fifty is the right number for a different reason —
   the sagitta falls with the *square* of the step, so fifty halvings ask a piece
   to bend `1e-30` of what the first one did — and the doc says that now.
5. **"Every accepted step moves the point at least `step` along the curve"** is
   not what the code checks. The trial point is settled onto both surfaces
   afterwards, so what a step covers is whatever the settling left, and all that
   is asked is that it is not nothing. The termination argument leans on the
   count cap, which the same paragraph already said.
6. **"Level 3 decides the rest"** is false for the one question that is not a
   polynomial. Two leaning pipes compare a length against a length in three
   different metrics; clearing the roots costs eight degrees and runs past what
   `f64` can square, so the exact arm answers `None` and the enclosure is the
   whole of the answer (§29.2).
7. **"Nothing builds a quartic yet"**, on `EdgeRun::RightRoundWhereItsFacesCross`,
   stopped being true when the second stage built them. §29.13 item 3 had
   noticed; the doc comment had not.

**What it could not break.** The hardest push went at §29.6's structural claim —
*an expansion whose entries span more than about half of `f64`'s range cannot be
squared at any scale*. Squaring needs every pairwise product exact, so the
smallest entry must be above `2^-484` and the largest below `2^512`, which is a
spread of about 996 binary orders; scaling by a power of two slides both together
and cannot change the spread. The claim is right, "about half of `f64`'s range"
is a fair reading of 996 against 2098, and §29.6's measured 505-order spread is
correctly a **gap** rather than a limit. The second-hardest went at
`how_many_steps`'s bound `reach · (1 − cos(step/2))` for an *ellipse* rather than
a circle: it holds, and it is tight at a half turn and at two thirds of one for a
degenerate ellipse, because the two semi-axes are square to each other and the
second derivative of the parametrization is never longer than the larger of them.

### 30.5 Every refusal M3 can produce, limit or gap

§24.17: *"The claim audit runs before the closing gate, and every refusal is
classified."* Fourteen rows, because four refusals have two causes each and
§17.7 says families are reported separately. **Six are gaps and seven are
limits, and one is filed as both**, because §29.6 says the naming half and the
evaluator classify it differently.

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `Undecidable` | a product leaves `f64`'s range while naming | **gap** | `a_ball_too_small_for_its_own_squares_cannot_be_crossed_and_the_same_one_scaled_up_can`; all 215 refusals in the shipped corpus are named on the lattice by `every_crossing_a_scaling_refuses_is_one_the_same_shapes_on_the_lattice_can_name` |
| `Undecidable` | a direction with a subnormal component underflows its own product | **gap** | `a_direction_with_a_subnormal_component_cannot_be_told_from_square_to_a_plane` |
| `Undecidable` | two leaning pipes' comparison is too wide to square where it stands | **gap** | `the_question_two_leaning_pipes_ask_is_too_wide_to_square_where_it_stands` — the same expansion written `2^300` higher squares |
| `CannotNameWhichPieceOfACrossingThisIs` | a picking normal must be exactly square to an endless direction and `Plane` scales every normal | **gap** | `two_lines_no_f64_half_space_can_tell_apart_are_refused_rather_than_guessed`; the same pair on the lattice is named. §29.8, and M4's to close |
| `CannotNameWhichPieceOfACrossingThisIs` | the half-spaces on a run pick both arcs between its ends, or neither | **limit** | `a_seam_that_picks_neither_half_of_its_meridian_or_both_is_refused` — the two halves of that meridian are genuinely alike |
| `ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo` | the two surfaces are tangent along the curve | **limit** of the naming half taken with the evaluator; **gap** of the naming half alone | `marching_from_where_a_crossing_runs_through_itself_has_no_direction_to_follow`; §29.6 |
| `CannotWorkOutHowBigACrossingIs` | the conic's own size runs past the top of `f64` | **gap** | `a_crossing_too_big_to_put_a_bound_on_is_refused_and_the_same_one_smaller_is_walked` |
| `CannotFollowACrossingWhereItsSurfacesAreTangent` | the two surfaces really are tangent where the marcher stands | **limit** | `marching_along_a_crossing_where_the_two_surfaces_are_tangent_refuses_by_name` |
| `CannotFollowACrossingWhereItsSurfacesAreTangent` | the sine of the angle between them cannot be told from zero, though they do cross | **gap**, and **no generator reaches it** | Named and not demonstrated. §29.7's `Exact::enclosure` took the band family from refusing 0.2% to refusing none, and nothing in the suite has reached this arm since |
| `CannotPlaceAPointThatCloseToACrossing` | the deflection asks for closer than a point can be placed | **limit** | `a_tolerant_caller_gets_the_finest_mesh_the_arithmetic_reaches` — a certified caller is refused and a tolerant one is told what it got |
| `TooManyPointsAlongOneCrossing` | the request needs more points than one run holds | **limit** of the request | `a_crossing_that_takes_more_points_than_it_is_allowed_refuses_rather_than_stopping_short`, `a_deflection_needing_more_points_than_can_be_held_is_refused` |
| `AnEndOfARunNamesNoPlace` | an end that is neither "nowhere in particular" nor a box with a middle | **limit** | `an_end_that_does_not_say_where_it_is_is_refused_rather_than_walked` |
| `ACapDoesNotCutRightRoundABar`, `ACapOfABarFacesIntoIt`, `TheCapsOfABarMeetInsideIt` | two planes that do not cut a bar | **limit** | `a_bar_whose_caps_are_not_caps_is_refused` |
| `AnEdgeWithoutTwoFacesAlongItCannotBeWalked` | three or more faces meet along one edge, so no pair says where it runs | **limit** | `an_edge_with_three_faces_along_it_is_refused_rather_than_asserted_against` |

`DeflectionBelowWhatAMeshCanReach` is M2's and §26.5 item 3 already classified it
a **limit**. M3 moved its floor and §30.6 says by how much.

**The one row worth reading twice is the ninth**, because it is the only refusal
in the milestone that is named, classified and undemonstrated. It is a gap by
construction — two surfaces that cross have a direction to follow, and only the
arithmetic loses it — and the generator built to reach exactly that band
(§29.11's `a_crossing_a_hair_from_the_knife_edge`) does not, because
`Exact::enclosure` made the sine decidable at scales nobody expected. That is a
claim about unreachability with a measurement behind it rather than a paragraph,
which is what §16.10 asks for, but it is not a demonstration and it should not be
written up as one.

### 30.6 What wiring the mesher to `walk` moved

§28.9 item 5 left this to the closing gate and predicted that swapping the mesher
over "changes how many points every curved mesh gets". Measured, before and
after, at a 100 mm working size:

| | before | after |
|---|---|---|
| bar, radius 3, height 5, deflection 0.1 | 58 points, 112 triangles | **58, 112** |
| the same bar at 0.01 | 178, 352 | **178, 352** |
| ball, radius 3, deflection 0.1 | 422 points, 840 triangles | **424, 844** |
| the same ball at 0.01 | 4463, 8922 | **4429, 8854** |

**The bar did not move at all and the ball did**, and the reason is worth having.
The two layers measure the same arc differently: the evaluator sizes a step by
the true sagitta of the curve, `reach · (1 − cos(step/2))`, and `rcad-mesh` judges
what comes out by `chord² / (3 · radius)`, the bound it derives for a flat piece
with its corners on any surface. Those differ by exactly eight thirds, because
`sin²x = (1 − cos x)(1 + cos x) ≤ 2(1 − cos x)` — so a boundary is refined to
three eighths of what the triangulator will allow, and that factor is an
inequality turned round rather than a fudge. With it in place a bar's cap circle
comes out at the same count either way. A ball's seam does not, because it is
half a great circle and the old walk sized it off the ball's radius through the
chord formula while the new one sizes it off the conic's own reach. Both meshes
pass the oracle at both deflections, and the ball's **measured volume moved
towards the exact `4πr³/3` in both directions**: 111.27610 → 111.29357 and
112.89652 → 112.89801, against 113.09734.

Three things came out of the wiring that were not wiring.

1. **A conic carried one number where it needed two.** The size everything was
   worked out from — a plane's offset, a ball's centre, an axis origin — is the
   right scale for a *rounding* and the wrong one for a *bend*. Charged against
   it, the cap circle ten up a bar was walked into more pieces than the one at the
   bottom, and the middle of a meshed bar was no longer half way up it. §19 for
   the evaluator, and it is `how_finely_a_crossing_is_walked_is_about_its_size_
   not_its_place.rs` now.
2. **The floor under a deflection is two chains in two frames.** The evaluator
   follows a crossing in the frame the face is *stored* in and the placement
   rounds against where the solid *sits*, and charging both against the far one
   would have put the floor fifty times higher than the arithmetic asks for.
3. **Which meridian a pole-to-pole seam runs down is no longer read in
   `rcad-topo`.** It is the same question `walk` already asks — which of the two
   arcs between the same two ends is this — and asking it twice is the shape §23.8
   kept finding. `Error::ASeamBetweenTwoPolesNamesNoMeridian` is gone and the
   refusal comes from the evaluator.

So §28.9 item 5's "what is left is wiring" was right about the seam and the
types, and wrong that nothing would be found: a defect in `walk`, a derivation
that had to be done, and a floor that had to be split.

### 30.7 The numbers

Reported family by family and never blended, per §17.7.

**Criterion 5, the control.** 2048 pairs per family, `Policy::certified()`.

| Family | `Undecidable` | Threshold |
|---|---|---|
| lattice, planar pairs | **0.00%** | zero, asserted |
| lattice, quartic pairs | **0.00%** | zero, asserted |

**Criterion 6, the headline.** Same 2048 per family. "No half-space" is
`CannotNameWhichPieceOfACrossingThisIs` and is counted apart because it refuses
for a reason `Undecidable` does not have.

| Family | `Undecidable` | no half-space | runs through itself | thresholds |
|---|---|---|---|---|
| turned, planar | 0.00% | 0.44% | 0.00% | zero / 2% |
| turned, quartic | **4.64%** | **10.93%** | 0.00% | 8% / 15% |
| leaning | 0.00% | 0.24% | 0.00% | zero / 2% |
| the band | 0.00% | 10.79% | 1.07% | 5% |
| written at `2^k` | 10.49% | 3.95% | 0.00% | 25% |

**Criterion 3**, over about 130 000 points: the furthest any point sat off its
own curve, as a share of the bound published for it.

| | |
|---|---|
| lattice / leaning / turned, parametrized | 0.006 / 0.008 / 0.012 |
| lattice / turned quartic, marched | 0.738 / 0.355 |
| the band | 0.410 |

**Criterion 7**, the load-bearing one: **0.577** of the marcher's own published
bound, and **the same figure at every scale** from `2^-200` to `2^200`. The
ceiling is three quarters. The parametrized arm of the same circles is 0.008,
and §29.4 says why the gap between them is the bound being nearly tight rather
than a weakness.

**Criterion 10**: **901 seconds, 23 037 787 executions at about 25 600 a
second**, corpus minimised to **562 inputs — 529 the fuzzer kept and 33 written
by hand**. On every commit the same property runs over the same decoder: the
whole corpus, plus 200 000 pseudo-random byte strings and about 700 000 points,
in five seconds.

**Criterion 11**, human-read and no target (§24.16), re-measured at this gate:

| | |
|---|---|
| one certified marched point | **2.66 µs** (662 µs for a 249-point run) |
| one parametrized point | 34 ns (2.35 µs for a 69-point run) |
| naming the whole crossing, drawing none of it | 1.48 µs |
| one interval divide | 11.2 ns |
| **the interval divide's share of a marched point** | **9 × 11.2 ns = 4%** |

Within a few percent of §29.10 on every row. Marching still costs about eighty
times parametrizing the same curve.

**The suite.**

| | at §29 | now |
|---|---|---|
| tests (`cargo nextest`) | 973 | **985** |
| suite duration | 18.1 s | 18.0 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | 2 |
| fuzz corpus | 562 | 562 |

**`just fuzz` was re-run because this lane touches `rcad-geom`**: 5 105 002
executions in 181 seconds at about 28 200 a second, and **no defect at all**.
Coverage moved by one edge and the working corpus by four inputs, none of which
is worth checking in unminimised, so the shipped corpus stays at 562.

Twelve tests and no time at all, which is what a gate that fixes things rather
than building them looks like. Across the milestone: **860 → 985 tests, 17.0 s →
18.0 s.**

### 30.8 What M3 hands to M4

§24.19 wrote five items before the work. Corrected and extended.

1. **`classify` and `fuse` stay gated on `is_wholly_flat`** (§24.19 item 1,
   unchanged). M4 is where a fuse over curved crossings is the first consumer
   that can remove the gate.
2. **The interval divide is measured and it is not where the money is.** §24.19
   item 2 said M4 would be the first caller with a figure to beat. The figure
   exists and it says 4%; what to optimise, if anything, is **settling** — two
   Newton corrections per trial step, each six dot products and a two-by-two
   solve, with a certified enclosure of the deviation on top.
3. **The general quartic is named and only ever marched**, so a fuse over two
   crossing pipes is where criterion 7's 0.577 is first spent in anger (§24.19
   item 3, unchanged).
4. **A working frame now has a figure asking for it**, and §24.8 deferred it on
   the explicit grounds that it had none. The figure is §29.6's **505-order
   spread**, and the ask is specific: the two-pipe discriminant needs its
   *expansion slid up the range*, not its inputs centred. That deferral has come
   due. `rkyv`, the bump arena, cone and torus and the B-spline fit stay deferred
   with their consumers named.
5. **`Plane` storing an unscaled normal** has a measured cost of not doing it:
   0.44% of turned planar pairs and 10.9% of turned quartic ones. It is a change
   to a type every predicate in the kernel is certified about, which is why it is
   M4's.
6. **A crossing that runs through itself** needs a third `EdgeRun` variant and a
   marcher that can be told to start away from a singular point and stop at it.
   One consumer, and it is M4's fuse (§29.6).
7. **`Measures::Dimensionless` is still one line** (§28.6), and the leaning
   pipes' three comparisons are lengths, which is the right arm and the wrong
   name.
8. **The mesher's floor moved by a factor of about fifty.** A boundary point is
   placed by the evaluator now, which publishes `256 · ε` where the mesher
   charged `8 · ε`, so the finest mesh of a solid at the origin is that much
   coarser than it was. Nothing needs it finer today. The way back is a tighter
   derivation of `POINT_ALONG_A_CROSSING_ROUNDOFF`, which has seven times the
   room its own chain needs (§28.5) and is wide for the trigonometry.
9. **`MOST_POINTS_A_MESH_WILL_HOLD` is shadowed by the per-crossing cap.** A ball
   has one curved edge and `MOST_POINTS_ALONG_ONE_CROSSING` is four times
   smaller, so the whole-mesh cap is now only reachable through the cutting of a
   face. The first solid with five curved edges is a fused one.
10. **A closed edge that is a *hole* in a curved face has no direction rule.**
    The mesher picks a closed edge's direction by the sign of its turn round the
    face, which is what makes a bar's wall close; a hole that does not wind round
    the face gets the evaluator's own direction, and nothing says that is the one
    which reads as a hole. No builder makes such a face. M4's ball with a pipe
    through it is the first.
11. **`Undecidable` out of `mesh` is still unreachable**, and §26.6 item 4 named
    M3 as the milestone that would reach it. §30.1 says why it did not, and the
    first constructed corner in a mesh domain is a fuse's.

### 30.9 Corrections to earlier sections

- **§24.19 item 2** said M4 would be the first caller with a figure the interval
  divide has to beat. The figure arrived in M3 and it says the divide is 4%. See
  §30.8 item 2.
- **§24.18 criterion 9** reads "the same pair", and the property that discharged
  it drew four of the six pairs. Both are drawn now. §30.4.
- **§26.6 item 4** is not closed and M3 could not close it. §30.1.
- **§28.9 item 5**'s "what is left is wiring" was right about the seam and the
  types and wrong that the wiring would find nothing. §30.6.
- **§28.7** lists `CrossingRunsToAQuartic` among the four error variants stage
  one added. The second stage made it unreachable; it is gone. §30.4 item 3.
- **§29.9** says the corpus is "530 the fuzzer kept and 32 written out by hand".
  It is **529 and 33**; the total of 562 is right.
- **§29.10**'s benchmark figures are confirmed within a few percent. The
  parametrized run is 69 points and not 70, because §30.6 item 1 changed which
  number the step rule reads.
- **§29.13 item 1** says criterion 8 is not done and **item 2** says the claim
  audit has not run. Both are done. **Item 3** stands: the curve that runs
  through itself is still outside `EdgeRun`'s vocabulary, and §30.8 item 6 hands
  it on. **Items 4 and 5** are carried into §30.8 items 4 and 5.

---

## 31. What opening M4's gate settled

*Recorded 2026-09-01, before any lane opens. §15 settled the shared meanings
before M1 and M2.5 fanned out, §23 before M2's lanes, §24 before M3's stages,
§25 before M2's last four and §27 at M3's inner gate. This is the same, for §13
rule 4's reason.*

**§30.3 is the instruction manual for this section and it is followed
literally.** A settling document is right about meanings and ownership and wrong
about arithmetic, the numbers are cheap to be wrong about because the first
person who needs one does the sums, and the dangerous sentences are the ones
that specify a *test* rather than claim a number. So: every number below is
marked confident or guessed in the row it sits in, every criterion in §31.7
names a test that does not exist yet rather than one that already passes, and
§31.10 is a list of the places this gate is planning on top of a guess.

### 31.1 What M4 is, now that the roadmap's version of it already ships

Roadmap §12 says M4 is *"General fuse, multi-argument, history maps. Run the
metamorphic suite."* Read that against the tree before believing it describes
work to be done:

- `Model::fuse` takes `&[SolidKey]` for both the arguments and the tools, and
  has since M2.5. Roadmap §7.2's "multi-argument from the start" is done.
- `Fused` carries `images_of_face`, `images_of_edge` and `images_of_vertex`,
  with generated, modified and deleted derived from them. Roadmap §7.1 step 6 is
  done.
- `Rule` holds Union, Intersection and Difference, read as one table and one
  `match` (§17.4). Roadmap §7.1 step 5 is done.
- `crates/rcad-topo/tests/fuses_that_must_agree.rs` is the metamorphic suite —
  commutativity, associativity, idempotence, absorption, `A \ A`, batched
  against chained, bit-identical under exact isometries. Roadmap §11.2 is done.
- `crates/rcad-topo/tests/degenerate_fuses.rs` is roadmap §7.4's catalogue,
  nine of thirteen entries, as a pass/refuse table.

All of it works, and all of it works **for planes only**. §1.7 already records
why: the planar boolean was built before intersection, on purpose, so that the
decomposition, the selection table, the history maps and the metamorphic
oracles would all be shipped and measured before a curved crossing arrived to
be classified against them. Roadmap §12 is the long-range plan and §1 says it is
not a description of this codebase.

**So M4 is one sentence: the same fuse, over curved faces.** Its one deliverable
is that the five lines at the top of `Model::fuse` come out —

```rust
for &solid in arguments.iter().chain(tools) {
    if !self.solid(solid).is_wholly_flat() {
        return Err(Error::FuseNeedsFlatFaces);
    }
}
```

— and everything below them keeps working. `classify` loses its
`Containment::NeedsFlatFaces` arm for the same reason and at the same time,
because §17.4 makes classification the thing selection is built on and a fuse
that cut curved faces and could not place them would have got half way.

**What M4 is not.** It is not a new surface kind (§24.6's argument is unchanged
and cone and torus still have no consumer), it is not the free-form marcher
(§24.1), and it is not a performance milestone — §31.9 keeps the BVH and the
bump arena deferred and says what figure would call each due.

### 31.2 Every question a curved fuse asks, and who decides it

§27.1's descendant, and the same discipline: every row is a **sign** or a
**construction**, every one is about the numbers a `Surface` stores, and none of
them is a distance compared against a tolerance.

Write `n` for a plane's stored normal and `d` for its offset, `c` and `R` for a
ball's centre and radius, `o`, `w` and `r` for a pipe's axis origin, axis
direction and radius, and `∇S(x)` for a surface's gradient at `x` — which is `n`
for a plane, `x − c` for a ball, and `|w|²(x − o) − (w·(x − o))w` for a pipe,
each of them a polynomial in the stored numbers with no normalization in it.

| | The question | Where the fuse asks it | The polynomial | Degree | Who decides it | Today |
|---|---|---|---|---|---|---|
| 1 | where two faces cross | `cuts::gather` | §27.1's eleven rows | 2–12 | `intersect` | **exists** |
| 2 | are two faces on one surface | `cuts::cuts_between`, the ON columns | as row 1 | 2–12 | `intersect`, `SurfaceMeet::SameSurface` | **exists, and it does not say which way round** — §31.3 |
| 3 | do two faces touch at one point | `cuts::gather` | as row 1 | 2–12 | `intersect` | **exists** |
| 4 | where three surfaces meet | every corner of every arrangement | three planes: one 3×3 determinant of stored coefficients. Two planes and a quadric: the discriminant of the quadratic the pair-of-planes line cuts out of it, cleared of `\|n₁ × n₂\|²` | 3 / about 14 | `Plane::meet_of_three` / **new**, `rcad-geom` | **a third of it** |
| 5 | how many points three surfaces meet in | the same | the sign of row 4's discriminant | as row 4 | **new** | **no** |
| 6 | is this corner on that cut curve | `place_the_corners` | `SideOfSurface` certified `Zero` on the partner surface | 1–2 | `side_of_surface` | **exists** |
| 7 | are two cut curves one curve | `line_for` | all planes: four 3×3 minors of coefficients. Otherwise: are the two partners one surface, which is row 2 | 3 / 2–12 | `relation_of_two_lines`, `intersect` | **half** |
| 8 | which of two points on an arc comes first | `nodes_along`, `order_along` | straight: a comparison on the direction's dominant axis. A conic: `orient3d(A, B, C, a point off its plane)`, because a conic is convex and three points on a convex curve are in cyclic order exactly when they turn that way | 1 / 3 | the dominant axis rule / **new caller of `orient3d`** | **half** |
| 9 | which of two arcs leaving a corner is anticlockwise | `turns_anticlockwise` | `orient3d(∇S(x), ∇T₁(x), ∇T₂(x), origin)` — §17.4's determinant of three normals with gradients in their place | 3 in the gradients | `orient3d`, **new caller** | **the predicate exists** |
| 10 | does this trimmed curved face hold this point | selection, ray casting, `validate` | **not settled here** — §31.10 item 1 | ? | **new** | **no** |
| 11 | where a ray meets a curved face's surface | `classify_uncertain_point` | ball: `(v·(c−b))² − \|v\|²(\|c−b\|² − R²)`. Pipe: the same with the axis projected out | 4 / 6 | **new** | **no** |
| 12 | which way a ray crosses a curved face | the same | `v · ∇S(x)` | 2 | **new** | **no** |
| 13 | is a piece on a face of the other group, and which way | the ON columns | row 2, **and** the two faces' stored orientations | — | **new**, and it is two questions — §31.3 | **half** |
| 14 | are two corners one corner | `Corners::of` | never a coordinate: the names of the three surfaces, sorted, plus which of their meets this is | — | **new naming, no arithmetic** | **half** |
| 15 | does a wire keep the material on its left | `check_curved_face` | **not settled here** — §31.10 item 2 | ? | **new** | **no** |

**Which of those numbers I am confident in.** Rows 1, 2, 3, 6, 7 and 14 are
statements about code that exists and I am confident in them. Row 9 I am
confident in and it is the nicest thing in the table: §17.4's derivation of
`turns_anticlockwise` uses only that an arc of `S ∩ T` runs along
`∇S × ∇T` and that `(a×b)×(a×c) = a·((a×b)·c)`, and neither step used flatness
anywhere, so the planar formula is the curved formula with a constant gradient
substituted in. Row 8's conic half I am confident in for the same reason a
convex outline can be wound: an ellipse is convex, so the cyclic order of three
points on it is the sign of the area they sweep, and no centre and no angle has
to be constructed. **Row 4's "about 14" is a guess**, and so is row 11's 4 and 6
— I have not written the expansions out, and §30.3 says the first person who
needs one will do that in an afternoon and be right where this is wrong.

**Three rows are the milestone, and they are 4, 10 and 11.** Everything else is
either shipped or a rename.

**The limit under row 4, stated as a prediction and classified.** Two planes and
a quadric is a quadratic in one unknown and its discriminant is a polynomial in
stored numbers. One plane and two quadrics is a quartic, three quadrics is a
degree-eight system, and neither has a discriminant this milestone is going to
write. **M4 answers row 4 when at least two of the three surfaces are planes and
refuses by name otherwise.** That refusal is a **gap** and not a limit — the
points exist, and a real algebraic solver reaches them — so §16.10 says fix it
rather than defer it, and M4 defers it anyway. The reason is the one §29.6
already found for the two-pipe discriminant, one degree up: the family of fixes
is the Dupont–Lazard–Lazard–Petitjean machinery or a certified numerical solver
with its own separation bound, and both are a milestone rather than a lane. The
consumer that will call it due is named: **two capped bars crossing near one of
their caps**, where the quartic where the two pipes meet runs into a cap circle,
so the corner is a plane and two pipes. Two bars crossing away from their caps
have no such corner and are M4's.

Nothing else in the milestone is blocked by it. A block with a round hole
through it, a round bar cut by a block, a ball with a pipe through it and two
bars crossing in the middle all have corners where at least two of the three
surfaces are planes, or no corners at all.

### 31.3 What a fuse over curved faces means

§25.7 settled "what a mesh of a solid is" and that was the most useful thing in
its section. This is the same job for a curved fuse. Every item below is a
meaning, not an algorithm.

**A face *is* its surface, and §17.1 is the same sentence with one word
changed.** A piece cut out of a curved face inherits the parent's `Surface` bit
for bit, so chaining fuses never degrades a sphere, and the stored corners are
cached samples of it.

**§17.1's re-derivation does not generalize, and the reason is worth having.**
A corner of a planar solid has three planes round it and Cramer's rule places
it. A corner of a *round bar* has two surfaces round it — the cap's plane and
the wall's cylinder — and the whole cap circle satisfies both. There is nothing
to solve. **Settled: a corner with three or more surfaces round it is
re-derived, and a corner with two is read out of the model.** That is not a
retreat, because `check_curved_face` has already certified that the stored
corner is `Decision::Zero` on its surface — a corner of a curved face is proved
to be exactly on the curve it sits on, which is a stronger promise than the
box a Cramer solve gives.

**A corner read out of the model and then moved is a box.** `bake` composes
placement, and `Isometry::apply_to_point` rounds. A transformed corner is
therefore *near* the transformed surface and not certified on it, so a
`side_of_surface` asked about it as an exact triple would certify a confident
sign about a rounding — §15.5's mistake by the road §27.7 already found once.
**Settled: a baked corner with two surfaces round it is a `PredicatePoint`
whose box is the placement's own rounding, tagged `Cause::ConstructedPoint`,
and it is representation wobble that refuses nothing (§17.2).** How wide the box
is, is arithmetic and §31.10 item 6 says it is a guess.

**A cut is by the second *surface*, whole, and never by the curve.** §17.4's
"everything that matters to a face is a pair of planes" becomes **everything
that matters to a face is a pair of surfaces**. `BakedFace::cuts` holds
`Surface` rather than `Plane`, the arrangement's `Line` holds a partner
`Surface`, and cuts.rs's argument for why no face pair may be skipped —
a plane reaches past the face carrying it, so skipping the pair leaves an edge
split on one side and not the other — reads the same with "surface" in it.
Nothing constructs a curve anywhere, which is §24.3 kept.

**A pair of surfaces can name two curves, and that is what `picked_by` is
for.** Where the planar arrangement has one line per pair, the curved one has
one *arc family* per pair, told apart exactly as `EdgeRun::PartOfWhereItCrosses`
tells them apart. So the arrangement's line list is a list of `EdgeRun`s, which
is a type that already exists and that the fuse can hand to `walk` without
translating — §24.3's requirement that nothing needs translating to become an
edge.

**A seam is an ordinary boundary edge, and a crossing that runs across one is an
ordinary node.** A seam's two faces are the *same* face, so `BakedEdge::across`
cannot be "the plane of the face on the other side": there is no other side.
It is read out of the edge's own `EdgeRun::PartOfWhereItCrosses::and`, which is
the field that exists for exactly this. After that the arrangement has nothing
to say about seams at all — a cut curve crossing a ball's meridian is a corner
where three surfaces meet, like any other. **This is where §15.5 pays: because
no decision in the arrangement is taken in `(u, v)`, the periodic-surface
degeneracy that roadmap §7.4 lists is not a case here.** It is a case one layer
down, in the mesher, where M2 already solved it.

**A split curved face may have no outer wire.** A ball with a pipe through it
leaves the ball's face as a sphere with two disks taken out, and neither of its
two closed boundary curves is more "outer" than the other. On a closed surface
"outline and holes" is not a property of the geometry; it is an order the face
stores, and `FaceData`'s doc already says the first wire is the outer one by
fiat. **Settled: the invariant a curved face has to satisfy is that every wire
keeps the face's material on its left, about the surface's outward normal —
not that the first winds one way and the rest the other.** The flat case is the
special case where those two coincide. That gives `check_curved_face` the
winding check it has never had; how the check is written is §31.10 item 2.

**A closed crossing with no corner on it still needs one vertex.** A wire needs
a coedge, an edge needs two ends, and `wire_closes_on_one_edge` needs both ends
to be one vertex. So the fuse has to place a vertex somewhere on a closed
crossing, and where it places it must depend on the two surfaces and on nothing
else, or `fuse(A, B)` and `fuse(B, A)` disagree. **Settled: the vertex goes
where `walk` starts, because `walk` already starts a whole closed run at a place
that is a function of the pair alone, and asking the question twice is the shape
§23.8 kept finding.** `rcad-geom` grows one small entry for it, and that is the
whole of the change.

**Which way a closed edge runs is decided by row 9, and that closes §30.8 item
10.** The mesher currently picks a closed edge's direction by the sign of its
turn round the face, which works for a bar's cap circle and says nothing about
a hole. The direction is the crossing's own stored one; which of the two faces
walks it forward is the face whose material lies on its left, and that is
`orient3d(∇S, ∇T, the arc's direction)`. One rule, both cases, and the mesher
stops having a rule of its own.

**`TouchAtOnePoint` cuts nothing.** Two tangent balls contribute no cut curve
and no vertex, so neither face is split and neither solid learns about the
other. Union leaves two shells that touch at a point and share no vertex, so
they come back as two solids; intersection is a single point, which is not a
solid, so the answer is empty and that is a success (§17.4). Roadmap §7.4's
tangent-spheres and tangent-cylinder–plane entries are that, plus the same
question asked of a tangency along a whole curve, which `intersect` answers as
one curve and the fuse cuts along like any other.

**`SameSurface` is §17.4's coplanar case with one word changed.** Two coplanar
faces contribute one cut line per edge of the other, each named by the pair
`{S, U}` for the face `U` across that edge. Two faces on one *cylinder* do
exactly the same thing, and `boundary_of` needs no change beyond its type. That
is the whole of the coincident-curved-face case, and it is free.

**But `SurfaceMeet::SameSurface` does not say which way round, and
`Plane::agreement_with` does.** The ON columns of §17.4's selection table need
same-facing against opposite-facing, and for a plane that is carried in the
stored coefficients because a reversed plane is a different four numbers. A
sphere has no reversed form; a face on the outside of a ball and a face on the
inside of a cavity are the same `Sphere` used with opposite `Orientation`.
**Settled: the ON question splits in two — "is it one surface", which is
geometry and `intersect` answers it, and "does it face the same way", which is
topology and the faces' own orientations answer it.** For a plane the two are
welded together today, and unwelding them is what makes one rule cover all
three surface kinds. §17.4's ON row is corrected in §31.11.

**The merged meet type, which §27.3 handed here.** `FaceMeet` keeps its shape
and widens:

```rust
pub(crate) struct FaceMeet {
    carried_by: [Surface; 2],
    picked_by: SmallVec<[Surface; 1]>,
    ends: [MeetPoint; 2],
    along: SmallVec<[EdgeKey; 1]>,
    approximation: Option<Approximation>,
}

pub(crate) enum MeetPoint {
    Existing { vertex: VertexKey, point: PredicatePoint },
    OfThreeSurfaces { surfaces: [Surface; 3], which: u8, point: PredicatePoint },
}
```

`which` is the new field and it is load-bearing: three surfaces can meet at more
than one point, so the three sorted surfaces no longer name a corner on their
own, and `Corners::of` would join two different corners into one. It counts
along the arc's own order, which row 8 decides exactly, so it depends on the
three surfaces and not on which face asked first. `approximation` finally gets
a producer that is not zero, which is what §15.7 predicted and §17.5 corrected
twice.

At the commit each `FaceMeet` becomes an `EdgeRun`, by a table and not by a
judgement:

| The two surfaces either side | The ends | Written as |
|---|---|---|
| two planes | two different corners | `Straight` |
| any pair naming one closed curve | one corner twice | `RightRoundWhereItsFacesCross` |
| a pair naming two curves, or an arc of one | anything | `PartOfWhereItCrosses { and, picked_by }` |
| one surface, twice — a seam | two corners | `PartOfWhereItCrosses { and, picked_by }` |

### 31.4 The two type fixes that go first, and the one that is not a fix

The sequencing is settled elsewhere and this section plans inside it: §30.8
items 5 and 4 land as one lane against a still-flat fuse, where the existing 985
tests certify them, before `Error::FuseNeedsFlatFaces` comes off. Both change
types every predicate in the kernel is certified about, and re-certifying a
moving type underneath fanned-out lanes is the expensive order.

**Item 4 first, and it is small.** §29.6 measured a 22-entry expansion spanning
505 binary orders that cannot be squared where it stands, and showed the same
expansion written `2^300` higher squaring without complaint. The fix is not a
working frame and not centred inputs: **what the caller wants is a sign, a sign
is unchanged by multiplying by a positive power of two, and multiplying an
expansion by a power of two is exact.** So `Exact` slides the expansion into the
window before squaring and slides nothing back, and there is no accuracy
question to argue about anywhere in it. §30.4 already published the window —
every pairwise product exact needs the smallest entry above `2^-484` and the
largest below `2^512`, a spread of about 996 binary orders — and §15.6 says a
published bound needs a test, which that one does not have yet.

What has to be true afterwards: the two-pipe comparison decides wherever the
spread fits, `Error::Undecidable` out of the quartic pairs falls, and §30.7's
two moving rows — **4.64% `Undecidable` on turned quartic pairs and 10.49% on
pairs written at `2^k`** — are re-measured and the thresholds in
`how_often_naming_a_crossing_refuses.rs` moved down to match. A rate that does
not move is the lane's own regression, not a surprise about the world.

**Item 5 is `Plane`, and this gate found that the fix as written is not one.**

§29.8 diagnoses the half-space gap as the *normalization*: "every `Plane`
constructor scales its normal to length one and that division rounds, so which
normals come out exactly square to a given direction is not something a caller
chooses". §30.8 item 5 turns that into "`Plane` storing an unscaled normal".
Both sentences are about `normalized_as`. **They are both silent about the step
before it, and that step has the same effect.**

`Plane::reduced_by_largest_component` divides all three components by the
largest, and that division is correctly rounded rather than exact. Worked
through, for a pipe whose stored axis direction is `w = (1, 3, 5)` — the three
natural normals square to it, each of them exactly square before the plane
stores it:

| the caller writes | reduced to | the exact dot with `w` |
|---|---|---|
| `(3, −1, 0)` | `(1, −1/3, 0)` | `2⁻⁵⁴` |
| `(0, 5, −3)` | `(0, 1, −3/5)` | `2⁻⁵³` |
| `(5, 0, −1)` | `(1, 0, −1/5)` | `−2⁻⁵⁴` |

Not one of them survives, and level 3 of the cascade is exact so it reports
every one of them as genuinely not square. For `w = (1, 2, 3)` the normal
`(2, −1, 0)` *does* survive, because 2 is a power of two and dividing by it is
exact. **That is the real rule, and it explains §29.8's own observation better
than §29.8 does: reduction preserves exact squareness exactly when the largest
component is a power of two, which on the lattice it always is — "on the lattice
it is a coordinate axis and comes out right, and off it, sometimes."**

So the property to aim at is not "an unscaled normal". It is this, and it is
what the lane's test has to say:

> **A normal a caller wrote to be exactly square to a direction it already holds
> is still exactly square to that direction after a `Plane` has stored it.**

The family of fixes, with what each keeps and what each gives up:

1. **Store the four numbers the caller handed over, unchanged.** Gets the
   property outright. Gives up §22.6 — two faces built from the same plane by
   routes that differ by a scale factor store different numbers, and
   `CornerName::Meet`, `canonical_cut_planes`'s dedup and
   `plane_of_the_joined_face` all compare bit patterns.
2. **Keep the reduction, drop only the normalization.** This is §30.8 item 5 as
   written and the table above says it is **not a fix**.
3. **Reduce by the power of two nearest the largest component.** Exact, so it
   keeps the property; keeps §22.6 for scale factors that are powers of two and
   loses it for the rest, which is the case §22.6 was written about.
4. **Separate the stored numbers from the canonical name**: store what the
   caller handed over, and give `Plane` a canonical key that the three
   bit-comparing call sites use instead. Keeps both properties and adds a second
   answer to "are these the same plane" beside `agreement_with`, which is
   §23.8's pattern for the ninth time.
5. **Leave it**, and keep paying 0.44% of turned planar pairs and 10.9% of
   turned quartic ones.

**Not settled here, and deliberately.** Which of 1, 3 and 4 to land is a
question about what §22.6 actually needs, and §22.6's own test —
`the_same_wall_built_from_a_longer_edge_is_the_same_plane` — is the thing that
answers it. What is settled is that **the lane must land one of them, must say
in §32 which of the two properties it gave up, and must leave a test behind for
each.** Choosing 2 because §30.8 named it is the one outcome this gate exists to
prevent.

**And `Axis` has §22.6's bug, unfixed, and M4 is its first consumer.**
`Axis::new` normalizes and does not reduce. Drawing 200 000 integer directions
and scaling each by an integer between 2 and 97, **115 834 of them — 58% — store
different bits from the direction they are proportional to**. Reducing first and
then normalizing brings that to zero out of 200 000. Nothing has noticed because
nothing builds the same pipe two ways; a curved fuse does, the moment two solids
are flush against each other on one cylinder, and then `intersect` answers
`Curves` where it should answer `SameSurface` and the ON columns never fire.
`the_same_pipe_built_from_a_longer_axis_direction_is_the_same_pipe` is the test
that fails today, and it belongs in this lane because it is the same change to
the same kind of type.

**What certifies the whole lane.** The 985 tests that pass today, unchanged in
count and in what they assert, plus:

- the squareness property above;
- `Plane::SIGNED_DISTANCE_ROUNDOFF` re-derived, because its derivation uses
  `|n₁| + |n₂| + |n₃| ≤ √3`, which is a statement about a unit normal — §15.6;
- every reader of `offset()` **as a length** found and given something that is
  one: `InPlane::of`'s reach in `select.rs`, `Surface::ROUND_TRIP_ROUNDOFF`'s
  "largest number involved", `walk`'s conic size, `Plane::origin`, `project`,
  `basis`, `plane_coordinates_of`, `point_at`, `signed_distance_to` and
  `validate`'s `allowed_wobble`;
- `FORMAT_VERSION` to **3**, with a round trip, because the four numbers a plane
  writes out change;
- the moved figures, measured and recorded: the 0.44% and 10.9% of §29.8.

### 31.5 `rcad-predicates` takes nothing at this gate, and is frozen

The first milestone that needs no change to it. §15.9's rule stands: a change
reopens this gate and stops the fan, so a lane that finds it needs one comes
back here rather than taking it.

Weighed and put down:

- **A `Cause` for a corner moved by a placement.** §31.3 tags a baked curved
  corner `Cause::ConstructedPoint`, whose doc says "a point that had to be built
  rather than read ... landed between two `f64`". A read corner that had to be
  moved is the same quantity told the same way, and a second variant would be a
  second answer to a settled question (§1.5).
- **A `Measures` arm.** `ManyLengths { how_many }` is parametric and §31.2's
  highest row is a fourteenth-degree question, which it already spells.
- **`Measures::Dimensionless`.** §28.6 and §30.8 item 7 keep it one line away.
  It is still one line away and M4 is still not its consumer: nothing in
  §31.2 asks a question without units.
- **A fifth `Decision`.** Same answer as §27.2.

### 31.6 Who owns what, and in what order

§13 rule 2 shapes a milestone gate, fan, gate; §15.9 gives each crate one owner
during a fan; §13 rule 6 caps work in flight inside one crate at two or three.

| Lane | What it is | Crate it owns | Opens |
|---|---|---|---|
| **0. This section** | §31 | `docs/decisions.md` | done |
| **1. The two type fixes** | §31.4, both of them, expansion first | `rcad-geom`, and it ripples everywhere | now, **alone** |
| **2. Where three surfaces meet** | §31.2 rows 4, 5, 8, and one point on a closed crossing | `rcad-geom` | when 1 lands |
| **3. Does a curved face hold this point** | §31.2 rows 10, 11, 12, 15 | `rcad-topo` (`classify.rs`, a new file, `validate.rs`) | when 1 lands, **beside 2** |
| **4. The curved arrangement** | §31.3, `fuse/bake.rs`, `fuse/cuts.rs`, `fuse/arrange.rs` | `rcad-topo` | when 2 and 3 land |
| **5. Curved selection and the commit** | `fuse/select.rs`, `fuse/assemble.rs`, `fuse/mod.rs`, `fuse/history.rs` | `rcad-topo`, **same owner as 4** | when 4 lands |
| **6. The four curved catalogue entries** | roadmap §7.4's remainder, test-only | `rcad-topo/tests/` | alongside, from when 5 lands |
| **7. The metamorphic suite over curved solids** | test-only | `rcad-topo/tests/` | alongside, from when 5 lands |
| `rcad-predicates` | nobody | frozen (§31.5) | — |
| `crates/rcad/src/lib.rs` | everybody | re-export lists only | — |

**Lane 1 runs alone and nothing else is open.** That is the sequencing decision
this gate was told to plan inside, and reading the code agrees with it: `Plane`
is in the signature of `orient3d`'s callers, `validate`, the mesher, the
splitter, the intersector and the serialized format, and a lane holding any of
those open while the type moves underneath it would be debugging someone else's
regressions, which CLAUDE.md rule 4 forbids by name.

**Inside lane 1, the expansion goes first.** It is the smaller change, it is in
a different file, and it moves two of §30.7's measured rows. Doing it after
`Plane` would mean measuring those rows against a `Plane` that had just moved,
with no way to say which change moved them.

**Lanes 2 and 3 run side by side, and that is the only concurrency in the
milestone.** They are in different crates, §15.9's rule is satisfied literally,
and neither consumes the other: lane 2 answers questions about three surfaces
and never looks at a face; lane 3 answers a question about one face and its
wires and never needs three surfaces at once. Lane 3 is written against lane 2's
absence, not against its contract.

**Lanes 4 and 5 are one owner, in sequence.** §13's table files "boolean
classification and shell assembly — graph traversal" under *does not split*, and
§17.6 kept the same two pieces in one head for the planar fuse. This is that,
again, and §24.12's argument against file-level ownership inside one crate
applies unchanged.

**§24.12's prediction was scored wrong in both halves and this section does not
repeat its mistake.** §30.2's conclusion was not "stop predicting" — it was that
half of §24.12 was scored at the gate, by reading two functions for an hour, and
that checking a prediction is far cheaper than the prediction is accurate. So
§31.10 makes one, and M4's first closing record scores it.

**Generators, per §13 rule 5.** Lane 2 draws triples of planes, balls and pipes.
Lane 3 draws point grids against a bar and a ball. Lane 4 draws blocks with
round holes. Lane 5 draws the same shapes under the three rules. Lanes 6 and 7
draw the catalogue and turned pairs of curved solids. **Lanes 4, 5, 6 and 7 all
want a bar and a block, and that is worth naming rather than glossing, exactly
as §25.10 named C and D sharing the bar and the ball.** What separates them is
the oracle: lane 4's is that the pieces of a face cover it and overlap nowhere,
lane 5's is a dense point sampling against a mesh, lane 6's is a pass/refuse
table, and lane 7's is metamorphic. No one wrong answer satisfies all four.

**Section numbers, reserved now so no two lanes collide over them.** §32 is the
two type fixes, §33 is the curved fuse, §34 is M4's closing gate.

### 31.7 M4 exit criteria

Sixteen, each naming a test that does not exist yet. §30.3 warns that the
dangerous sentences in a settling document are the ones that specify a test
rather than claim a number, because a number gets checked by the first person
who needs it and a test that already exists does not — so **every name below is
a test that has to be written, and criterion 3 is the one that reuses an
existing file and therefore the one to read twice.**

1. A normal a caller wrote square to a direction is still square to it once a
   plane has stored it —
   `a_half_space_written_square_to_a_direction_is_still_square_to_it_once_stored`.
2. The same pipe described by two proportional axis directions is one pipe —
   `the_same_pipe_built_from_a_longer_axis_direction_is_the_same_pipe`.
3. The half-space gap is closed and the figures moved: the shipped thresholds in
   `how_often_naming_a_crossing_refuses.rs` come down, and
   `two_lines_no_f64_half_space_can_tell_apart_are_refused_rather_than_guessed`
   is replaced by one asserting the same pair off the lattice is now named.
4. A comparison too wide to square where it stands is squared somewhere else —
   `a_comparison_too_wide_to_square_is_decided_after_it_is_slid_up_the_range` —
   and the published window has a test —
   `an_expansion_wider_than_the_window_cannot_be_squared_at_any_scale`.
5. Where three surfaces meet is answered as boxes, every one certified `Zero`
   on all three under a certified policy —
   `every_point_where_three_surfaces_meet_is_on_all_three_of_them`. This is
   M3's criterion 3 one dimension down and it is the load-bearing one.
6. The count of meeting points is a sign, and a lattice control refuses none —
   `a_certified_meeting_of_three_lattice_surfaces_never_refuses`.
7. Three surfaces the milestone cannot solve refuse by name rather than
   guessing —
   `three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved`.
8. Points along a planar crossing are ordered with no angle constructed —
   `three_points_on_a_conic_come_back_in_the_order_they_run_round_it`.
9. One authority answers whether a trimmed curved face holds a point, and it
   agrees with a fine mesh of that face wherever both can tell —
   `whether_a_curved_face_holds_a_point_agrees_with_a_mesh_of_the_same_face`.
10. Point-in-solid works on a curved solid, with the ON case —
    `the_middle_of_a_bars_wall_is_on_its_skin`,
    `a_point_a_hair_inside_a_balls_skin_is_inside_it`.
11. `Error::FuseNeedsFlatFaces` and `Containment::NeedsFlatFaces` are gone, and
    a block with a round hole through it is one solid —
    `a_block_with_a_round_hole_through_it_is_one_solid`.
12. The pieces a curved face is cut into cover it and overlap nowhere —
    `the_pieces_a_curved_face_is_cut_into_cover_it_and_overlap_nowhere`.
13. A fused curved solid validates, meshes, and passes §23.4's oracle —
    `a_block_with_a_round_hole_meshes_into_a_closed_surface` and
    `a_ball_with_a_pipe_through_it_meshes_into_a_closed_surface`, both through
    `the_mesh_closes_up`.
14. Roadmap §7.4's four remaining entries are a pass/refuse table —
    `the_four_curved_entries_as_a_table`: tangent spheres, tangent
    cylinder–plane, seam-crossing intersections, closed periodic surfaces.
15. The metamorphic properties hold over curved solids —
    `swapping_the_two_sides_of_a_curved_pair_changes_not_one_bit`,
    `a_bar_with_a_copy_of_itself_taken_away_is_nothing_at_all`,
    `uniting_a_block_and_a_bar_the_other_way_round_gives_the_same_solid`.
16. The two rates, reported family by family and never blended (§17.7): a
    lattice-aligned curved control at **0%** under `Policy::certified()` —
    `every_certified_fuse_of_a_lattice_curved_pair_is_decided_and_records_nothing`
    — and a turned curved family measured and asserted below a threshold
    recorded in §34 — `how_often_a_certified_fuse_of_a_turned_curved_pair_refuses`.

**No number is offered for criterion 16's threshold, on purpose.** §26.3 ended
with "it should stop suggesting numbers" and §30.3 said two milestones is enough
and the convention should be a rule. This is the rule being kept: the lane
measures it, §34 records it, and this section does not guess it.

Criteria 5, 9 and 16 are the load-bearing three. Five is the only new
construction the whole milestone rests on; nine is the question three separate
call sites ask and the one place two answers would silently disagree; sixteen is
the control that makes the headline mean anything, and §17.7's precedent is that
the control is what tells a timid filter from real undecidability.

The claim audit runs before §34, per §16.10, and every refusal M4 can produce is
classified limit or gap by name, as §30.5 did for M3.

### 31.8 §30.8's eleven items, sorted

The way §27.6 sorted §26.6's six.

| | | |
|---|---|---|
| 1 | `classify` and `fuse` gated on `is_wholly_flat` | **M4's, and it is the whole milestone** — §31.1, criterion 11 |
| 2 | the interval divide is 4% and the money is in settling | **not work.** A figure with no target is not an optimisation (§24.16), and M4 has no performance criterion. §34 may re-measure it beside a curved fuse |
| 3 | the general quartic is only ever marched, so a fuse over two crossing pipes spends criterion 7's 0.577 in anger | **M4's**, and it lands as lane 5's two-bars case rather than as a criterion of its own |
| 4 | the two-pipe discriminant needs its expansion slid up the range | **lane 1**, first half — §31.4 |
| 5 | `Plane` storing an unscaled normal | **lane 1**, second half, and §31.4 says the fix as written is not one |
| 6 | a crossing that runs through itself needs a third `EdgeRun` variant and a marcher that can start away from a singular point | **deferred, and the consumer is named but is not M4's** — §31.9 |
| 7 | `Measures::Dimensionless` | **not M4's** — §31.5 |
| 8 | the mesher's floor moved by about fifty | **not work.** Nothing needs it finer, and M4 does not change that. §34 watches it |
| 9 | `MOST_POINTS_A_MESH_WILL_HOLD` is shadowed by the per-crossing cap, and the first solid with five curved edges is a fused one | **M4's, and it is a watch rather than a task** — a block with a round hole has three curved edges and a ball with a bar through it has two, so it may still not fire |
| 10 | a closed edge that is a hole in a curved face has no direction rule | **M4's, and §31.3 settles it** with row 9's determinant. Lane 4 |
| 11 | `Undecidable` out of `mesh` is still unreachable | **not work, and this gate predicts M4 does not reach it either** — see below |

**Item 11 deserves the paragraph, because it has now been handed on twice.**
§26.6 item 4 named M3 as the milestone that would reach it and §30.1 recorded
that M3 could not, because `walk` hands the mesher 3D points and the mesher
reads them back through `coordinates_of`, so every corner a triangulator sees is
an ordinary `f64` pair. §30.8 item 11 says the first constructed corner in a
mesh domain is a fuse's. **It is not**, and the reason is one line of
`assemble.rs`: `write_into` stores `corner.point`, which is `middle_of(point)` —
a `Point` and not a box. A committed vertex has always been an exact triple and
a curved fuse does not change that; the width goes on the vertex as an
`Approximation` and never into the coordinate. So a mesh domain will still hold
no constructed corner after M4. **This is a prediction and §34 scores it.** What
would make it reachable is a `Model` whose vertices are boxes, which is a change
to what a model *is* and is nobody's milestone yet.

### 31.9 What stays deferred, and what would call it due

§13 rule 3: a part with no named consumer is not built. Each row names the
consumer and, where there is one, the figure that would call it due.

| Deferred | Consumer | What would call it due |
|---|---|---|
| The BVH broad phase (roadmap §7.1, sent to M4 by §17.4) | a fuse over hundreds of tools | A measured pair count. §17.4's argument that a *face*-level box test changes an answer still holds, and the solid-level one is already there. Nothing in M4 has a target |
| The bump arena (§11, sent to M4 by §17.5) | the same fuse | A measured allocation share of a curved fuse. §30.8 item 2 says the money in curved geometry is in settling, not in allocation, and §1.4 says an optimisation with no number it must beat is the mistake. §34 can produce the number |
| `rkyv` (§6, §24.14) | still nobody | Unchanged |
| Cone and torus (§24.6) | a STEP file that holds one (M5), or draft (M6) | Unchanged. §24.6's argument — a new `Surface` variant touches `validate`, the mesher, `is_wholly_flat`, the fuse and serialization — is *more* true during M4, not less |
| The B-spline fit of a marched crossing (§1.8) | unchanged | Unchanged |
| A crossing that runs through itself (§29.6, §30.8 item 6) | **named as M4's fuse, and this gate moves it out** | Two equal pipes whose axes meet, or a ball reaching exactly across a pipe. Both are knife edges a fuse reaches only when a caller asks for them exactly, they need a third `EdgeRun` variant *and* a marcher that can start away from a singular point and stop at it, and M4 already has one refusal of that shape (§31.2's row 4 limit). It is deferred to the milestone that builds the algebraic solver, and until then it refuses by name and `the_four_curved_entries_as_a_table` records the refusal |
| Solving three curved surfaces (§31.2) | two capped bars crossing near a cap | A real algebraic solver, and it is a milestone rather than a lane |
| A working frame that centres its inputs (§24.8) | — | **Not deferred and not built.** §30.8 item 4 said the deferral came due; §31.4 says what came due was sliding an expansion by a power of two, which is a different and much smaller thing. Centring inputs still has no figure asking for it |

**Two of those rows are gaps deferred rather than fixed, and CLAUDE.md says fix
a gap and defer a limit.** They are named here rather than buried for exactly
that reason: a crossing that runs through itself and a meet of three curved
surfaces are both gaps, both have answers, and both are deferred because the
family of fixes is a milestone's worth of algebra. Saying so is the difference
between "cannot" and "we chose not to", which §24.17 is about.

### 31.10 What this gate does not know

§30.3's whole lesson is that a settling document written before anyone has read
the arithmetic gets things wrong, and that the cheap ones are the numbers. These
are the places this section is planning on top of a guess, worst first.

1. **How to decide whether a trimmed curved face holds a point.** §31.2 row 10,
   and it is the schedule risk of the milestone. The planar answer fans the
   outline from a corner and asks `orient3d` against a reference point off the
   plane, so nothing is projected (§15.5). There is no such reference point off
   a sphere that works for the whole face, and unrolling a cylinder goes through
   `atan2`, which makes any decision on the result a decision on projected
   coordinates. Three routes I can see and cannot choose between: a ray *along
   the surface* from the point, crossing-counted against the wires, which turns
   the question into more three-surface meets; the sign of the gradient
   determinant at each arc, which is exact but only decides for a face whose
   arcs all curve the same way; and cutting the face into pieces the flat
   argument does reach. **I do not know which, and the lane's first day should be
   spent finding out rather than building.**
2. **How `validate` checks that a wire keeps the material on its left.**
   §31.3 settles what the invariant *is* and not how to ask it. Today
   `check_curved_face` checks corners and nothing else, and the flat winding
   check has no curved twin. It may turn out to be row 9's determinant at each
   corner, in which case it is cheap; it may need row 10, in which case it is
   row 10 again.
3. **Whether `Plane`'s fix can keep §22.6.** §31.4 lists five candidates and
   demonstrates that the one §30.8 named is not a fix. Which of the other three
   to land, and what it costs the three bit-comparing call sites, is the lane's
   and this gate does not know it.
4. **The degrees in §31.2 rows 4, 5, 11 and 12.** Marked as guesses in the row.
5. **Whether the mesher can mesh what the fuse hands it.** A fused curved face
   is the first with a hole in it, and `mesh.rs` refuses one that goes more than
   a quarter turn round with `CurvedFaceWithAHoleGoesTooFarRound`, on a doc
   comment whose reason is *"the fuse refuses a curved solid outright"*. That
   sentence stops being true in lane 5. `MOST_PIECES_A_FACE_IS_CUT_INTO` rests
   on "four pieces is the answer for every face this kernel builds", which is
   also about today's builders. **Neither is a bug yet and both are claims that
   the milestone falsifies**, which is §16.10's shape, and the claim audit
   before §34 is where they get re-argued.
6. **How wide a moved curved corner's box has to be.** §31.3 says it is the
   placement's own rounding; whether that is one `Isometry::POINT_ROUNDOFF` or
   two — because `Surface::transformed` rounds the surface as well as the point
   — is a derivation nobody has done.
7. **Whether `TouchAtOnePoint` cutting nothing survives the manifold check.**
   §31.3 settles that two tangent balls united come back as two solids sharing
   no vertex. `Model::validate` has no way to notice that two shells touch, so
   the answer is a model the kernel calls valid and a machinist would call one
   part. That may be right and it may be the thing §34 has to correct.

**The prediction, recorded before the work and to be scored at §32 rather than
at the milestone's end** (§30.2's cheaper half). §12.1's lesson — the money goes
to code that was already written and looked finished — has the precondition
§30.2 discovered: it holds for milestones that mostly *rewire* and not for ones
that mostly write. M4 mostly rewires. So the lesson should hold, and the
specific place is **`fuse/arrange.rs`, and not the parts about lines.** The
cycle walk, the winding count and `piece_around`'s ray are all written in the
vocabulary of `Line::axis` and `forward_climbs_the_axis` — a dominant axis and a
direction that either climbs it or does not — and every one of those is a
straight-line idea sitting inside a function whose name says nothing about
straight lines. `first_arc_met` compares positions along one axis at every turn
and has no other way of asking where anything is.
The second guess is `mesh.rs`, for item 5's reason. The third, and the one I
would most like to be wrong about, is that `classify_uncertain_point` costs
nothing again: §17.6 predicted it as M2.5's schedule risk and §17.8 recorded
that the cost was elsewhere, and predicting the same function twice in a row is
how a prediction becomes a habit.

### 31.11 The numbers at the gate, and corrections to earlier sections

| | at §30 | now |
|---|---|---|
| tests (`cargo nextest`) | 985 | 985 |
| suite duration | 18.0 s | 18.3 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | 2 |
| fuzz corpus | 562 | 562 |

A gate that writes one file moves nothing, which is what it should look like.
`just default` is green, `cargo-deny` included.

Corrections:

- **§29.8** blames the half-space gap on `Plane` scaling its normal to length
  one. The reduction before it has the same effect and §31.4 demonstrates it on
  three normals square to `(1, 3, 5)`. §29.8's own observation — right on the
  lattice, sometimes off it — is explained by the reduction and not by the
  normalization: reduction is exact exactly when the largest component is a
  power of two.
- **§30.8 item 5** says the fix is "`Plane` storing an unscaled normal". As
  written that is item 2 of §31.4's five, and it is not a fix.
- **§30.8 item 11** predicts M4 reaches `Undecidable` out of `mesh`. §31.8 says
  it does not, and why, and files that as a prediction of its own.
- **§17.1** says every corner is re-derived as the meet of the planes around it.
  That is true of a corner with three surfaces round it and false of one with
  two — a round bar's cap corner — which is read out of the model instead and is
  certified onto both its surfaces by `check_curved_face`. §31.3.
- **§17.4's ON row** treats "is it the same surface" and "does it face the same
  way" as one question that `Plane::agreement_with` answers. For a curved face
  they are two questions and the second one is topology. §31.3.
- **§24.9** leaves what a fuse does with `TouchAtOnePoint` and `SameSurface` to
  M4. Settled in §31.3: a touch cuts nothing, and one surface is §17.4's
  coplanar case with one word changed.
- **§27.3** leaves `FaceMeet` for M4 to merge. Settled in §31.3, and
  `MeetPoint::OfThreePlanes` gains a field rather than only widening — three
  surfaces can meet at more than one point, so the three names no longer name a
  corner.
- **`crates/rcad-topo/tests/fusing_solids.rs`** says of
  `every_face_of_the_answer_has_exactly_one_input_face_behind_it` that it is
  *"deliberately removed at M3: a curved intersection can produce a face that is
  no input face's child"*. This gate believes that is wrong — every piece is
  still cut out of an input face, and what a curved fuse invents is edges rather
  than faces. Lane 5 keeps that property alive over curved inputs, and if it
  fires, the property is what says so and `Fused::generated_faces` stops being
  empty for the first time.
- **`Axis::new`** normalizes without reducing, so it is not canonical under
  scaling. 58% of drawn proportional pairs store different bits. §31.4, and it
  is lane 1's.

## 32. What M4's two type fixes cost, recorded 2026-09-01

Lane 1 of §31.6: the expansion slide and the two types, against a still-flat
fuse, before `Error::FuseNeedsFlatFaces` comes off.

Two of the four criteria are met, one is met by a different route than the gate
named, and **the fourth cannot be met at all in the shape §31.4 gives it** —
which is the finding this section exists for.

### 32.1 The expansion slide, and the row it moved is not the row §31.4 named

§31.4's first half went in as written. `Exact::square_minus` slides a number
into the window where squaring it is exact, slides the other side of the
comparison by twice as much so the two stay comparable, and slides nothing
back: a sign is unchanged by multiplying by a positive power of two, and
multiplying an expansion by one is exact. The slide is **zero wherever the
square already fits**, so every comparison that was decided yesterday is decided
through exactly the same arithmetic today.

The window §30.4 published now has the test §15.6 asks for, and it is a
demonstration rather than a paragraph:
`an_expansion_wider_than_the_window_cannot_be_squared_at_any_scale` writes an
expansion one binary order wider than the window at **every** placement in the
normal range — 1050 of them — and is refused at all of them, and one exactly as
wide as the window is squared at every placement. The two ends are `2^-484` and
`2^512`, so the widest spread that fits is 995 binades.

**The turned quartic family's `Undecidable` rate went from 4.64% to 0.00%.** All
95 refusals were the 505-order expansion, and the slide reaches every one of
them. Over twenty times the draws the rate is 0.03% — fourteen of 40 961 — and
those are expansions whose own spread is past the window, which no scale
narrows.

**§31.4's other named row does not move, and this is a correction rather than a
regression.** §31.4 says the slide also brings down §30.7's 10.49% on pairs
written at `2^k`. It does not, and cannot: those 215 refusals are named
`where a plane and a ball cross`, `where a plane and a pipe cross`,
`where two balls cross` and `where two planes cross`, and every one of them is
`Exact::product_of(radius, radius)` refusing two raw `f64` whose product would
land among the subnormals. There is no expansion yet to slide. What reaches that
row is scaling the *inputs* — the working frame §24.8 defers and §31.9 says
still has no figure asking for it. It has one now, and it is 10.49%.

**One test had its number re-derived and one had its draws re-derived.**
`A_TURNED_QUARTIC_CROSSING_MAY_REFUSE` came down from 8% to 0.2%, which is six
times the 0.03% measured over twenty corpora and four draws of the shipped 2048.
And `every_turned_quartic_crossing_that_refuses_was_exactly_tangent_before_the_turn`
now draws twenty times what the rows beside it draw, because the refusal it is
about fell by the same factor and 2048 draws reach none of it — a property with
nothing to measure passes for the wrong reason, which is what §14.7 is about.
All fourteen refusals it does find are still knife edges a rotation made.

### 32.2 `Axis` had §22.6's defect and now does not

`Axis::new` normalized the direction it was handed without dividing by the
largest component first, so what it stored depended on that vector's length and
not only on its direction. Re-measured over every integer direction in `-6..=6`
with a length, against every whole-number multiple of it from two to
ninety-seven: **80 396 of 210 816 pairs — 38.1% — stored a different direction
for the same line**, and reducing first brings that to none of them.

§31.11's last correction records 58% from a different draw; the two are
different families and both are honest. This one is exhaustive over a lattice
and reproduces exactly.

`the_same_pipe_built_from_a_longer_axis_direction_is_the_same_pipe` is the
consumer: two solids flush against each other on one cylinder describe that
cylinder twice, off two builders, and `intersect` answered `Curves` where the
truth is `SameSurface`. It answers `SameSurface` now, for multiples of two,
three, five, seven, eleven and ninety-seven, and for the same pipe pointed
backwards.

`Vector::scaled_to_length_one_from_anywhere`'s doc claimed the first division is
exact. It is not — `a/|m|` is one correctly rounded division — and what is exact
is that the largest component comes out as `±1`. Corrected in place.

### 32.3 A plane can be canonical under scaling or it can keep a normal square, and not both

**This is the section §31.4 asks for and the answer is not one of its five.**

§31.4 got the diagnosis right and §31.11's correction of §29.8 stands:
`Plane::reduced_by_largest_component` loses exact squareness on its own, before
the normalization anyone had blamed. `a_normal_written_square_to_a_direction_stops_being_square_once_a_plane_stores_it`
is that, on the three normals a caller can write exactly square to `(1, 3, 5)` —
and two of the three read exactly `0.0` in plain `f64` afterwards while the
exact level sees a sign, which is why it hid.

§31.4's family of fixes turns on reducing by a **power of two**, which rounds
nothing. Half of that is right, and
`a_power_of_two_reduction_keeps_a_normal_square_and_stops_one_wall_being_one_plane`
shows both halves:

1. All three normals written square to `(1, 3, 5)` are **still exactly square**
   after a power-of-two reduction.
2. Two multiples of one wall — `(0, −1, 0)` and `(0, −3, 0)` — reduce to
   `(0, −1, 0)` and `(0, −1.5, 0)`, which differ by a factor that is **not** a
   power of two. One rotation later they are not exactly proportional at all,
   because every component of each is rounded on its own, and
   `Plane::agreement_with` answers `Different`, correctly, about two faces that
   are one wall.

**A canonical name beside the stored numbers does not reach the second half,
which is what §31.4's candidate 4 assumes.** §31.4 names three call sites that
compare bits. There are more than three, and the ones that matter are not
comparisons of bits at all: `Plane::agreement_with`, `planes_are_parallel` and
`relation_of_two_lines` are *exact predicates over the coefficients*, and the
two coefficient vectors above are genuinely not proportional, so no key
computed from them can say otherwise without inventing a tolerance.

Measured, with the whole of candidate 4 in place — power-of-two storage, a
`Plane::canonical_coefficients` key, the three named call sites moved onto it,
and a fourth found on the way (§17.4's "keep once" cell picks one of two
coincident pieces by argument index, which stops being order-free the moment two
coincident faces can hold different numbers):
`swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit` **refused 112 of
its 136 fuses, 106 of them naming `where three planes meet`, and could compare
12 of the 124 pairs it set out to compare against a floor of three tenths.**
Two walls that are one wall each cut the other, and three planes that share a
line have nowhere to meet.

Before that it compared all of them. The lattice control was worse still: it did
not refuse, it **panicked** — `the two coedges of an edge must run opposite
ways`, which is a broken solid rather than an honest refusal, because a "keep
once" that picks by the plane drops both of a coincident pair in the one cell
where the two are not interchangeable.

The general statement, and it is short. Canonical under scaling means dividing
by something the direction alone decides. Exact means dividing by a power of
two. Two multiples of one vector have the same power of two only when the
multiple is itself a power of two. **So the two properties cannot both hold**,
and §31.4's "you cannot have both" is right about more than it knew.

**What lands is candidate 5, and the half-space gap stays.** Its rates are
re-measured below and its classification is corrected: it is still a **gap**,
because a separating half-space exists, but it is not a gap any change to
`Plane` reaches. The family of fixes, and why each is the only one left:

| | |
|---|---|
| Change what `Plane` stores | Demonstrated above not to be one |
| Pick a different square normal | A caller *can* write one — swap two components of the direction and negate one, and the dot product is two products that are exact negatives. Measured over 7201 turned pipe axes, **10.5% of them have at least one of the three such normals survive being stored**, against 1.7% for the cross product the code writes today. Not a closure, and a tenth of a percentage point on the planar row is not worth the branch |
| Test the separation rather than arrange it | §29.8: a piece that runs on for ever leaves any half-space whose normal leans into its direction, however far along, so there is nothing bounded to test against |
| A half-space that is not a `Plane` | **The one that reaches it**, and it is a new `Surface` variant — which §24.6 says touches `validate`, the mesher, `is_wholly_flat`, the fuse and serialization. A milestone's decision and not a lane's, so this lane reports rather than takes it, the way §31.5 has a lane report rather than take a change to a frozen crate |

`FORMAT_VERSION` stays at **2**. §31.4 expected it at 3 because a plane's four
numbers would change; they do not.

### 32.4 The numbers

Family by family and never blended (§17.7). 2048 pairs per family,
`Policy::certified()`.

| Family | `Undecidable`, at §30.7 | now | no half-space, at §30.7 | now |
|---|---|---|---|---|
| lattice, planar | 0.00% | 0.00% | 0.00% | 0.00% |
| lattice, quartic | 0.00% | 0.00% | 0.00% | 0.00% |
| turned, planar | 0.00% | 0.00% | 0.44% | **0.44%** |
| turned, quartic | 4.64% | **0.00%** | 10.93% | **11.86%** |
| leaning | 0.00% | 0.00% | 0.24% | **0.20%** |
| the band | 0.00% | 0.00% | 10.79% | 10.79% |
| written at `2^k` | 10.49% | 10.49% | 3.95% | 3.95% |

**The turned quartic row's half-space count went up and its total went down.**
Nineteen pairs that used to refuse for want of a sign now get as far as being
told they are two loops, and refuse there instead. Taken together that family
refused 15.57% and now refuses 11.86%.

The leaning row moved from 0.24% to 0.20% because §32.2 changed what a leaning
pipe's stored axis direction is.

| | at §31 | now |
|---|---|---|
| tests (`cargo nextest`) | 985 | **994** |
| suite duration | 18.3 s | 18.6 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | 2 |
| fuzz corpus | 562 | 562 |

### 32.5 Corrections to §31, and what lanes 2 and 3 have to be told

- **§31.4** says the expansion slide brings down §30.7's 10.49% on pairs written
  at `2^k`. It does not; §32.1 says why, and that row's fix is the centred frame
  §31.9 files as having no figure asking for it. It has one.
- **§31.4's five candidates for `Plane`.** Candidate 2 the gate already showed is
  not a fix. Candidates 1, 3 and 4 are not fixes either, and §32.3 is the
  demonstration. What lands is candidate 5.
- **§31.4** lists three call sites that compare a plane's bits. The count is at
  least four — `select.rs`'s "keep once" cell is the fourth — and counting them
  is the wrong question anyway, because the ones that decide are exact
  predicates rather than bit comparisons.
- **§31.7 criterion 1** names
  `a_half_space_written_square_to_a_direction_is_still_square_to_it_once_stored`.
  That test cannot pass, and what stands in its place is the pair of
  demonstrations in §32.3, written §19's way round: they assert what the code
  does, so a fix would fail them loudly.
- **§31.7 criterion 3** asks for the half-space figures to move. They have not,
  and §32.3 says what would move them.
- **§31.11's last correction** puts `Axis::new`'s cost at 58% of drawn
  proportional pairs. Re-measured exhaustively over a small lattice it is 38.1%;
  both are honest measurements of different families.
- **The trap for lanes 2 and 3**, and it is one sentence: **two faces that are
  the same plane are guaranteed to hold the same four numbers, and everything in
  the fuse rests on that.** It survives because `Plane` still scales its normal
  to length one. Any lane tempted to change what a plane stores — to make a
  normal exactly square to something, to widen a range, to save a division —
  should read §32.3 first, because the failure is not local and does not look
  like the change that caused it.

## 33. The half-space gap, and a type that is not a surface, recorded 2026-09-01

Lane 1b of §31.6, taken out of §32.3's last row: **the half-space gap is closed,
and closing it moved every family that had one.** §31.6 reserved §33 for the
curved fuse; this lane takes the number and the curved fuse becomes §34's
neighbour, so the closing gate is not surprised by the shift.

### 33.1 §24.6 stands, and the argument is that a picker is not a surface

§24.6 says the surface set does not grow, and CLAUDE.md core principle 6 says a
decision is argued and amended rather than routed around. **It is neither
amended nor routed around, because what landed is not a `Surface`.**

§24.6's argument is that a new `Surface` variant touches `validate`, the mesher,
`is_wholly_flat`, the fuse and serialization. Every one of those is a place that
asks *what is this face sitting on*. A half-space is never a face's geometry: it
picks one piece of a crossing out of the rest, and §24.5 already widened that
field once without touching any of them, because the field is not `Surface`'s
job.

So the type split follows the question. `EdgeRun::PartOfWhereItCrosses` holds a
`SmallVec<[Picker; 1]>`, and

```rust
pub enum Picker {
    InFrontOf(HalfSpace),
    OutsideABall(Sphere),
    OutsideAPipe(Cylinder),
}
```

has no arm that is a plane, so there is no second way to say "flat picker" and
nothing has to decide which of two spellings it is holding. `validate`, the
mesher and `is_wholly_flat` never see a `HalfSpace`, because the type system
will not let one reach them — which is the point. **A `Surface::HalfSpace`
variant would have been a lie the type system was telling**: every `match` on
`Surface` in `rcad-topo` would have grown an arm for a case that cannot occur,
and each of those arms would have been an unreachable claim of exactly the kind
§16.10 says needs a demonstration and never gets one.

The cost §24.6 names is therefore not paid, and the cost that *is* paid is one
file in `rcad-geom`, three call sites in the intersector, one dispatcher in
`walk`, four constructions in `rcad-topo`'s tests and the format version.

### 33.2 A normal a caller wrote square to a direction stays square

`HalfSpace` stores `normal` and `offset` exactly as written. No reduction, no
normalization, no rescaling on the way back in from the format.

That is safe only because of an invariant the type system does not state, and
the doc comment says so: **a half-space is never a face's geometry and is never
compared for face identity.** §17.1 rests on two faces that are the same plane
holding the same four numbers, and `Plane` earns that by scaling its normal to
length one (§32.5's trap). Two half-spaces describing one flat surface may hold
quite different numbers, and nothing anywhere asks whether two of them are the
same.

`a_normal_written_square_to_a_direction_is_still_square_to_it_once_stored` is
§31.7's first criterion, which §32.5 recorded as impossible. It is the same three
normals §32.3 used, and it asserts both halves: all three are still exactly
square after a `HalfSpace` stores them, and not one of them is after a `Plane`
does. The two demonstrations §32.3 left in `plane.rs` are untouched and still
pass, because nothing about `Plane` changed.

### 33.3 The separating direction is not the one you can write down, and does not have to be

This is the part §32.3 did not reach, and it is why "pick a different square
normal" looked like a tenth of a percentage point rather than a closure.

A caller can write exactly three normals square to a direction `w`: `w × x̂`,
`w × ŷ` and `w × ẑ`, each two of `w`'s own components with one negated and a
zero, so the dot product is two products of the same pair of numbers with
opposite signs. None of them is the direction that separates two pieces of a
crossing. **It does not have to be.**

What separates two endless lines is any half-space whose normal is square to
their common direction and has a non-zero component along the direction they are
apart in. What separates two loops of a leaning-pipe crossing is the same, and
the price of a normal that is not the ideal direction is a real number rather
than a refusal. Writing `n̂ = α·â + β·b̂` in the two directions across the fat
pipe's axis, a loop point sits `a` along `â` and `b` along `b̂` with
`a² + b² = ρ₂²` and `|a| ≥ √(ρ₂² − (δ+ρ₁)²)`, so `|b| ≤ δ + ρ₁` and the reading
is at least `α·√(ρ₂² − (δ+ρ₁)²) − |β|·(δ + ρ₁)`.

The published floor on `α` is **`1/√3`**: it is `max |b̂ᵢ| / √(1 − ŵᵢ²)`, which
is at least `max |b̂ᵢ|`, which is at least `1/√3` because `b̂` has length one.
§15.6 asks a published bound for a test and
`the_best_exactly_square_normal_is_never_less_than_a_third_aligned_with_what_was_asked`
searches every whole-number direction in a nine-cube against every other and
finds **`1/√2`** and never less — the worst case is a direction along a
coordinate axis, where one of the three normals is nothing at all and the other
two are a right angle apart.

**The asked-for direction goes in first**, and that is not an optimisation. On
the lattice the separating direction is *already* exactly square, and taking a
coordinate crossing instead put the lattice quartic control at 1.12% — a
regression the control did not catch, because nothing had ever asserted the
control's half-space count. It does now.

### 33.4 A cross product that cancels, where a difference of squares does not

A ball crossing a pipe off-axis leaves two loops either side of the plane
through the ball's centre square to the pipe's axis, and the picker's normal is
now that axis direction, bit for bit. How much of it lies *across* the axis is
therefore exactly nothing — but only if the arithmetic can see it.

`√(‖n‖² − (n·ŵ)²)` cannot. The two squares are equal to a couple of last bits,
and the square root of a couple of last bits is about `1e−8`, which then
multiplies the ball's radius. `‖n × ŵ‖` can, and summed through `Exact` it is
exactly zero. **Eight orders of magnitude, from reading the same fact a
different way**, and it is the same lesson §29's exact sums taught about a
nearly-parallel cross product.

### 33.5 The numbers

Family by family and never blended (§17.7). 2048 pairs per family,
`Policy::certified()`, share refusing
`CannotNameWhichPieceOfACrossingThisIs`.

| Family | at §32.4 | now |
|---|---|---|
| lattice, planar | 0.00% | 0.00% |
| lattice, quartic | 0.00% | 0.00% |
| turned, planar | 0.44% | **0.10%** |
| turned, quartic | 11.86% | **2.24%** |
| leaning | 0.20% | **0.00%** |
| the band | 10.79% | **3.76%** |
| written at `2^k` | 3.95% | 3.95% |

`Undecidable` does not move in any family, which is expected: nothing here
changes a sign question. The `2^k` row does not move either, and that is the same
finding §32.1 recorded — those refusals are products landing among the
subnormals, and what reaches them is the centred frame §31.9 files as deferred.

**Every refusal that remains is a different one from the one this lane closed.**
Instrumented site by site over all seven families, the arm that says *this
picking normal is not exactly square to the direction the piece runs along*
fires **zero times**, in every family. That arm can now only be reached when
`Exact::dot_of` declines because a product of two of the direction's own
components lands among the subnormals, and no generator here draws one.

What is left, classified:

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `CannotNameWhichPieceOfACrossingThisIs` | the two lines a plane cuts from a pipe are not provably either side of the half-space between them | **gap** | the bounded half §29.8 closed; what is left is a plane so close to the pipe's wall that the two lines nearly meet, and the same pair drawn away from the wall is named |
| `CannotNameWhichPieceOfACrossingThisIs` | the best exactly-square normal is not aligned enough with the direction separating two loops | **gap** | the loops are there and are tellable apart; what runs out is `f64`'s supply of vectors exactly square to a given direction. §33.3's floor is `1/√3` and a fat pipe barely wider than what is inside it needs better than that |
| `CannotNameWhichPieceOfACrossingThisIs` | two loops of a ball-and-pipe crossing nearly touch | **gap** | the reach between them is smaller than the picker's own offset rounding; the same pair drawn further apart is named |
| `CannotNameWhichPieceOfACrossingThisIs` | a picking normal is not exactly square and cannot be made so | **gap, and no generator reaches it** | named and not demonstrated. It needs `Exact::dot_of` to decline on two components of a stored direction, which is the subnormal-product family §30.5's second row is about |

§30.5's fourth row — *"a picking normal must be exactly square to an endless
direction and `Plane` scales every normal"* — is **discharged**, and its
demonstration `two_lines_no_f64_half_space_can_tell_apart_are_refused_rather_than_guessed`
is replaced by
`two_lines_off_the_lattice_are_told_apart_by_a_half_space_written_square_to_them`,
which asserts the opposite three ways: the pair is named, the picking normal is
exactly square to the pipe's stored axis, and a point a million million steps
along one of the two lines is still on the side that picked it.

| | at §32 | now |
|---|---|---|
| tests (`cargo nextest`) | 994 | **1001** |
| suite duration | 18.6 s | **94.4 s** |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 2 | **3** |
| fuzz corpus | 562 | **699** |

**The suite duration is the one number that got worse, and it is worth reading
rather than glossing.** Ninety of the ninety-four seconds are one test:
`every_point_along_a_crossing_a_hair_from_tangency_is_on_both_the_surfaces_that_named_it`.
Measured by forcing every picking site to refuse, that row walks 33 runs and
44 212 points and takes 1.5 s; with the pickers working it walks 43 runs and
45 442 points and takes 90 s. **The extra thousand points cost eighty-eight
seconds because they are *marched*** — nearly-tangent quartic loops that this
crate used to refuse to name at all, whose per-point cost is the marcher's, and
whose per-point cost in a debug build is about thirty-five times what it is in a
release one, where the whole row takes 2.7 s.

The draws stay at 128. Those thousand points are the most valuable in the file —
they are §24.7's acceptance test applied to exactly the pieces the milestone just
learned to name — and cutting them to buy back CI time would be §14.7's failure
on purpose. What would buy it back honestly is an optimised test profile, and
that is a repo-wide tooling decision rather than a lane's.

### 33.6 Corrections to §30, §31 and §32

- **§32.3's last row** says the half-space "is a new `Surface` variant". It is
  not, and §33.1 is the argument. The row's conclusion — that a change to `Plane`
  cannot reach the gap — stands untouched and is why this lane exists.
- **§32.3's third row** files "test the separation rather than arrange it" as
  unreachable, quoting §29.8: *"a piece that runs on for ever leaves any
  half-space whose normal leans into its direction, however far along, so there
  is nothing bounded to test against."* That is exactly right about two straight
  lines down a pipe, which are endless. **It is not right about two loops of a
  leaning-pipe crossing, which are closed curves and therefore bounded.** Nothing
  turns on it — arranging the squareness is cheaper than bounding the extent of a
  loop that runs a long way up a nearly-parallel pipe, and it is what landed —
  but the sentence covers one case and was applied to two.
- **§32.4's band row**, the 10.79%, is §30.5's **fourth** row reaching the band
  family, not its fifth. §30.5's fifth row — *the half-spaces on a run pick both
  arcs between its ends, or neither* — is a refusal `walk` produces about a
  meridian, and this table only calls `intersect`. It is still a limit, it is
  still `a_seam_that_picks_neither_half_of_its_meridian_or_both_is_refused`, and
  it did not move. The band's intersector rate did, from 10.79% to 3.76%.
- **§31.7 criterion 1** is met, and §32.5's correction of it is withdrawn. The
  test cannot pass about a `Plane` and does pass about a `HalfSpace`.
- **§31.7 criterion 3** is met in both halves.
- **§31.4's** list of what certifies the lane includes `FORMAT_VERSION` to 3.
  It gets there, for a different reason than §31.4 gave: a plane's four numbers
  did not change, and the field that holds a picker did.
- **§31.5 held.** `rcad-predicates` took nothing. `SideOfSurface` is a trait with
  one method over a `Point`, and a new implementor in `rcad-geom` is not a change
  to it — see §33.7.

### 33.7 `SideOfSurface` needed nothing, and the reason generalises

The one thing that could have made this design more expensive than a `Surface`
variant was the side question: if answering *which side of this picker* had
needed a predicate that only exists for `Surface`, the split would have cost
what it saved.

It does not, and the reason is that a half-space is bounded by a plane, so the
question reduces to the sign of `normal · p − offset` over raw coefficients.
`SideOfPlane` was already that expression and nothing in it wanted a unit
normal: its level-1 error bound is a fraction of the permanent, which carries the
normal's own size, and levels 2 and 3 are an interval evaluation and an exact
one. So the predicate was renamed `SideOfAFlatSurface`, given the four numbers
instead of a `Plane`, and shared. `Plane::side_of_point` and
`HalfSpace::side_of_point` are now one line each over the same arithmetic, and
they differ only in the name they log.

`SIDE_OF_A_HALF_SPACE` is a separate provenance name, so a record still says
which of the two gave up.

### 33.8 The trap for lanes 2 and 3

§32.5's trap stands word for word and is the more dangerous of the two: **two
faces that are the same plane are guaranteed to hold the same four numbers, and
everything in the fuse rests on that.** Nothing this lane did weakens it, because
nothing this lane did touched `Plane`.

The new one is its mirror image, and it is one sentence: **a `HalfSpace` has no
canonical form and two of them describing one flat surface may hold quite
different numbers, so nothing may ever compare two of them for equality, dedupe
them, or use one as a key.** `PartialEq` is derived because a test wants to say
"the picker that came back out of the format is the one that went in", which is a
question about *bits* and not about geometry. A lane that finds itself asking
whether two half-spaces are the same flat surface has the wrong type in its hand;
what it wants is `Plane::agreement_with`, and what it should do first is ask why
a picker escaped into a place that decides face identity.

Three smaller ones:

- **`Picker` has no `transformed`.** Nothing composes a location onto an
  `EdgeRun` today — `validate` compares a picker against raw vertex points in the
  solid's own frame — and giving one to a `HalfSpace` would be the second
  authority §17.1 rejected, arriving through a different door. Lane 4 is where
  this comes due, when a fused solid holds edges from two placed arguments.
- **A picker's `Undecidable` is a real answer and rules out nothing.** Both
  `walk` and `validate` already treat it that way, so both pieces come back
  picked and the caller refuses rather than guessing. A new consumer that treats
  `Undecidable` as "not held" silently picks the wrong piece.
- **`FORMAT_VERSION` is 3 and the old bytes still parse.** A version 2 file's
  `Surface::Plane` reads as a `Picker::InFrontOf` without complaint and describes
  the same flat surface with different numbers. The version check is the only
  thing between them, and
  `a_plane_and_a_half_space_facing_the_same_way_are_the_same_bytes_saying_different_numbers`
  is why it has to be.

## 34. Where three surfaces meet, recorded 2026-09-01

Lane 2 of §31.6: §31.2 rows 4, 5 and 8, one point on a closed crossing, and the
box §31.3 says a moved curved corner has to become. Written beside lane 3 and
touching only `crates/rcad-geom`, which is §15.9's rule in force.

Three of the five things this lane was told to build were as §31 describes them.
The other two were not, and both are recorded below rather than glossed: **the
degree §31.2 flagged as a guess is right for one of the two quadrics and four too
high for the other**, and **§31.3's "walk already starts a whole closed run at a
place that is a function of the pair alone" was false for two balls** — measured,
forty of forty swapped pairs walked into different points.

### 34.1 The degrees, measured against the guesses

§31.2 row 4 gives "3 / about 14" and marks the second as a guess. Both halves,
worked out term by term rather than estimated:

| The triple | The polynomial | Degree | §31.2 said |
|---|---|---|---|
| three planes | one 3×3 determinant of stored coefficients | **3** | 3, and it is right |
| two planes and a ball | `B² − A·C` for `A = D·D`, `B = V·D`, `C = V·V − m²R²` | **10** | about 14 |
| two planes and a pipe | the same with the axis crossed out of both: `A = ‖D × w‖²`, `B = (D × w)·(V × w)`, `C = ‖V × w‖² − m²r²` | **14** | about 14 |

`D` is `n₁ × n₂` at degree 2, `m` is the two-by-two minor of the normals that is
largest, and `V` is the line's own point offset to the quadric's centre or axis
origin, at degree 3.

**Four of the degrees the guess was carrying come from the point on the line, and
picking a different one is what spends them.** The obvious point on the line of
two planes is the foot of the perpendicular from the origin, and it is the one
§31.2's parenthetical is written about — it needs `‖n₁ × n₂‖²` underneath it and
carries four more degrees into everything built on it, which is exactly the
`|n₁ × n₂|²` the row says the discriminant has to be "cleared of". The point with
a **zero coordinate** — set `x_k = 0` for the axis whose minor is largest and
solve the remaining two-by-two by Cramer — needs no clearing at all, because the
factor was never introduced. Ball: 10 rather than 14. Pipe: 14 rather than 18.

So §31.2's number is right about the pipe and its parenthetical is a description
of the more expensive of two routes. §30.3's rule held again: the guess was
cheap to be wrong about and the first person who needed it did the sums.

**A degree is not what a caller pays, and the second number is measured too.**
`how_many_f64_the_discriminant_of_a_line_and_a_quadric_takes_to_hold` records how
many `f64` the exact value needs:

| | A | B | C | `B² − A·C` | spread |
|---|---|---|---|---|---|
| a ball, on the lattice | 3 | 3 | 4 | **5** | 166 |
| a pipe, on the lattice | 7 | 9 | 7 | **19** | 393 |
| a ball, turned | 9 | 11 | 16 | **36** | 516 |
| a pipe, turned | 14 | 19 | 20 | **57** | 761 |

`Exact::times` bounds a product at twice the two lengths multiplied, which for
the last row would be five figures; what keeps it at 57 is `expansion_sum`
dropping every roundoff that comes out exactly zero. **761 binary orders against
the 995 §32.1's window leaves** is the number to watch: the turned pipe row is
what runs out first if this degree ever goes up again.

### 34.2 `Measures` needed nothing, and `rcad-predicates` is still frozen

§31.5 weighed a `Measures` arm and said "`ManyLengths { how_many }` is parametric
and §31.2's highest row is a fourteenth-degree question, which it already
spells". **The fourteenth-degree question is an area.** A degree is a count of
stored numbers multiplied together and `Measures` is about *units*, and the two
part company here: the parameter `s` running along the line is dimensionless, so
the quadratic's `A` is dimensionless, `B` is a length and `C` is an area, and
`B² − A·C` is an area like every other discriminant in the intersector.

So the highest-degree question in the milestone files as `Measures::Area`, the
same arm `where_a_plane_and_a_ball_cross` has used since M3. §31.5 held, and it
held for a reason it did not have: **not because `ManyLengths` already spells
fourteen, but because fourteen was never the number that mattered.**

### 34.3 Two ways the boxes came out wrong, both caught by the oracle

Criterion 5 is `every_point_where_three_surfaces_meet_is_on_all_three_of_them`,
and writing it turned up two defects that reading the code did not. Both are the
same shape and it is §29's: **a difference of two large numbers, evaluated where
it cancels.**

**The discriminant's size was being read in intervals.** `B²` and `A·C` for a
line nearly tangent to a quadric are two large numbers that cancel to nothing, so
an interval evaluation of the difference reaches out to the rounding of the
larger, `√Δ` carries that, and so does every point built on it. Measured: the
somewhere-else row's widest box was **2.1e-8 of the shape**. Summed through the
expansion instead — the same difference, `B.times(&B).minus(&A.times(&C))`, with
no slide in it — it is **3.0e-16**.

That is §33.4's finding one question up, and it needs the distinction §32.1 drew
kept sharp: **`Exact::square_minus` is a sign and never a value**, because it
multiplies both sides by a positive power of two. It is what settles the sign
where the un-slid difference runs out of range, and it is never what the roots
are worked out from. The two are separate calls on separate lines.

**And `(−B ± √Δ)/A` is not good enough on its own.** A line a hair off parallel
to a pipe's axis has `A = ‖D × w‖²` down at `1e-32`, and then one of the two
roots is a difference of two nearly equal numbers divided by that. Measured on a
bar's own cap plane turned by a sixty-fourth: **a box fifteen hundred times the
shape**, for a corner that is perfectly well determined. The fix is the pair a
quadratic solver has used since Forsythe — `q = −(B + sign(B)·√Δ)` cancels
nothing, one root is `q/A` and the other is `C/q` — and because both forms are
valid enclosures of the same number, the narrower of the two is taken per root
and there is no branch to get wrong when `q` runs through zero.

Neither would have been found by reading. The first hid because the sign was
always right; the second hid because the case that shows it is a bar's own cap
plane, which is the most ordinary shape in the kernel.

### 34.4 A closed crossing that started in two places

§31.3 settles where the one vertex on a closed crossing goes: *"the vertex goes
where `walk` starts, because `walk` already starts a whole closed run at a place
that is a function of the pair alone"*. **That was true of three of the four
conic pairs and false of the fourth.**

`where_two_balls_cut` builds the radical plane through the gap between the two
centres, so the pair the other way round gets that plane reversed, a different
`Plane::basis` out of it, and a start a quarter of the way round the circle.
Measured over forty pairs: **forty of forty walked into different points**, and
not by a rounding — the first point was `(4.21, −2.44, −1.14)` one way and
`(1.90, 4.48, −1.14)` the other. The other three arms are order-free only by
accident of shape: both halves of their `match` call the same function with the
same arguments.

So `Surface` grows **one** fixed order over the numbers each kind stores, and the
two questions that need one share it rather than each inventing its own — §23.8's
pattern avoided rather than repeated. Sorting on stored numbers is sound for
exactly the reason §32.5's trap gives, that two faces which are the same plane
hold the same four numbers, and it is **not** sound for a `HalfSpace`, which
§33.8 says has no canonical form at all. Two pipes of exactly the same certified
radius were a tie in the same shape and are broken the same way.

`where_a_closed_crossing_starts` is the entry §31.3 asks for, and it is one small
function as that section predicted.

**The marched half still runs the other way round when the pair is swapped**, and
that is left alone. A conic is parametrized from a frame the unordered pair
fixes; a degree-four loop is followed along `∇S × ∇T`, which flips. §31.3 already
says which of the two faces walks a closed edge forward is decided by the
material on its left, so the direction flipping is the geometry rather than a
defect — but two halves of one function behaving differently is what a caller
assumes away, so
`a_conic_walks_the_same_way_round_whichever_face_asks_and_a_marched_loop_does_not`
says it out loud.

### 34.5 Row 8's other half, and why a conic needs no angle

The arrangement has ordered points along a straight crossing by its direction's
dominant axis since M1, which is the half §31.2 records as done. A closed
crossing has no axis to climb and no end to measure from.

**A conic is convex**, so three points on it run round it in the order they wind,
and which way they wind is the sign of the area they sweep. One determinant; no
angle constructed, no `atan2`, nothing projected into a surface's own two
coordinates — §15.5 kept, and §31.2 was right to be confident about this row.

Which way is forward is the crossing's own direction `∇S × ∇T`, so the answer
flips when the two surfaces are swapped. Working out which plane normal that
winds anticlockwise about is a line of algebra for each pair and it is written
down in the module rather than asserted. The pipe is the one worth having: the
ellipse's tangent is `n × ρ` for the outward radial direction `ρ`, and whether
that runs anticlockwise about `n` is the sign of `u · ρ` for `u` the step from
the ellipse's centre — which works out to `‖u × ŵ‖²`, above zero for every `u` in
a plane the axis is not parallel to.

### 34.6 How wide a moved curved corner's box is: five, not one or two

§31.10 item 6: *"whether that is one `Isometry::POINT_ROUNDOFF` or two — because
`Surface::transformed` rounds the surface as well as the point — is a derivation
nobody has done."* Done, and it is **five**, with `CORNER_AFTER_MOVING_ROUNDOFF`
published at `128 · f64::EPSILON`, about seven.

Two slips add. The point costs one `POINT_ROUNDOFF`. The surface costs one for a
ball — the centre moves and the radius is carried over exactly — and **four** for
a plane or a pipe, each derived separately in the constant's own doc: a plane
carries the turned normal's slip twice, once against the point and once through
the shift, and a pipe carries the moved origin, the tilt of the axis under a
point up to `2L` away, and the certified radius `r/‖w‖` moving with `‖w‖`.

**The half that matters is not the number.** `a_corner_read_out_of_a_model_and_moved_is_a_box_around_where_it_lands`
measures that **2265 of 3040** moved corners are certified `Pos` or `Neg` off a
surface they are a rounding from, when handed over as an exact triple. That is
§15.5's mistake arriving by §27.7's road, on the most ordinary shape in the
kernel, and if it had come out near zero there would have been nothing here worth
building. Handed over as the box, none of the 3040 is certified either way, and
the same box against the same surface moved comfortably clear is decided every
time. A search across sixteen orders of magnitude of size uses **a twenty-fifth**
of the published bound, which is the usual gap between a worst case and a drawn
one.

### 34.7 The numbers

Family by family and never blended (§17.7). 2048 triples per family,
`Policy::certified()`, two flat cuts and a third surface drawn flat as often as
round.

| Family | corners found | certified `Zero` on all three | refused | widest box, as a share of the size |
|---|---|---|---|---|
| lattice | 571 | 315 (55%) | **0.00%** | 2.2e-16 |
| leaning | 1281 | 0 | **0.00%** | 1.8e-14 |
| turned | 682 | 0 | **0.00%** | 2.4e-15 |
| written at `2^k`, moved by `2^m` | 646 | 240 (37%) | **10.83%** | 3.0e-16 |

**Criterion 6 is met and the lattice control is stronger than it was asked to
be.** §31.7 asks that a certified meeting of three lattice surfaces never refuse.
It never does — and 315 of its 571 corners come back as boxes of *no width at
all*, certified `Decision::Zero` on all three surfaces, which is criterion 5 met
to the letter rather than in the "never certified off" reading the other rows
need. Where a corner is a whole number the arithmetic rounds nothing and the box
is a point.

**The `2^k` row's 10.83% is the same refusal §32.1 and §33.5 both recorded and
neither moved**, arriving one question up: a product whose low half would land
among the subnormals. It is a **gap** and the same centred working frame §31.9
files as having no consumer is what reaches it. It now has a second one.

The intersector's own rates are unchanged by this lane, which is what they should
be — nothing here changes a sign question about a pair:

| Family | at §33.5 | now |
|---|---|---|
| lattice, planar / quartic | 0.00% | 0.00% |
| turned, planar | 0.10% | 0.10% |
| turned, quartic | 2.24% | 2.24% |
| leaning | 0.00% | 0.00% |
| the band | 3.76% | 3.76% |
| written at `2^k` | 3.95% | 3.95% |

| | at §33 | now |
|---|---|---|
| tests in `rcad-geom`, which is all this lane owns | 481 | **510** |
| tests (`cargo nextest`, whole workspace) | 1001 | **1045**, and still moving — lane 3 is committing beside this |
| suite duration | 94.4 s | 96.7 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 699 | **746** |

`just default` is green, `cargo-deny` included.

`just fuzz 180` ran 4 348 402 executions in 181 seconds, found **47 new seeds**
and **no crash**. Coverage went from 2505 to 2506 edges, so the new arithmetic is
reached by the corpus rather than sitting outside it.

### 34.8 The suite's ninety seconds are one draw, and splitting it does not help

§33.5 recorded the suite at 94.4 s with ninety of it in one test, and said the
honest fix was an optimised test profile rather than cutting draws. This lane was
asked to split that test's draws instead, so that the same 128 draws run on more
than one core. **It was built, measured, and reverted, because it buys nothing.**

The split was written so that the runner still samples the strategy all 128 times
in every one of the eight tests — the shard picks which draws are *walked* and
never which are *made*, so the union of the shards is the family draw for draw
and nothing is cut or moved. Measured: **draw 73 alone takes 88.6 of the 90
seconds** and every other shard finishes in under 0.4.

Draw 73 is two pipes down one direction, radii 5 and 4 with their axes 9 apart —
exactly tangent — with the second radius moved down one last bit and its axis
leaned by `2^-29`. Nearly parallel and nearly tangent leaves a very long thin
quartic loop, and the marcher walks 18 365 points round it at the deflection this
file asks for. That is the row working.

So no arrangement of the draws across tests can go below 88 s, and eight tests
that make the suite no faster are eight tests. **What does reach it is measured:
`[profile.test] opt-level = 2` in the workspace manifest takes `rcad-geom`'s
suite from 91 s to 2.7 s**, which is the same factor §33.5 measured for a release
build. That is one line, it changes no semantics — `debug-assertions` and
`overflow-checks` are separate settings and stay on — and it is **not taken
here**, because the workspace manifest is not this lane's and §33.5 already
called it a repo-wide tooling decision. Reported rather than taken, the way
§31.5 has a lane report rather than take a change to a frozen crate.

### 34.9 Corrections to §31, and what lanes 4 and 5 have to be told

- **§31.2 row 4's degree** is 10 for two planes and a ball and 14 for two planes
  and a pipe. The guess of "about 14" is right about the pipe; four of the
  degrees it carries for the ball are the foot-of-the-perpendicular point on the
  line, and the point with a zero coordinate never introduces them. §34.1.
- **§31.2 row 4's parenthetical** — "cleared of `‖n₁ × n₂‖²`" — describes the
  more expensive of two routes. Nothing is cleared, because nothing is
  introduced.
- **§31.5** keeps `Measures` unchanged for a reason it did not have. The
  fourteenth-degree question is an **area**: a degree counts stored numbers and
  `Measures` counts units, and the parameter running along the line is
  dimensionless. §34.2.
- **§31.3** says `walk` starts a whole closed run at a place that is a function
  of the pair alone. It did not, for two balls, and forty of forty swapped pairs
  proved it. Fixed rather than worked around. §34.4.
- **§31.7 criterion 5** asks for every meeting point to be "certified `Zero` on
  all three". Read literally that is unreachable for a box with width, for the
  reason `every_point_along_a_crossing_is_on_both_surfaces.rs` already argues one
  dimension up, so the oracle is M3's: never certified *off*, plus a control
  moved comfortably clear that must be decided. **On the lattice the literal
  reading is met as well**, 315 corners of 571.
- **`meet_of_three_planes` in `rcad-predicates` hands back a box that is not a
  place, and says `Ok`.** Cramer's rule there divides one interval determinant by
  another and checks neither for overflow, so three planes whose offsets are near
  the top of the range and whose normals lean at a small angle come back as
  `Ok(Constructed([…, […, −inf]]))` — a box a caller reads as a corner. Measured
  on `x = 1e300`, `y = 1e300` and a plane leaning by `2^-50`: the `z` side is
  `[−inf, −1.8e308]`. **This lane does not fix it, because §31.5 freezes that
  crate**, and reports it the way that section says a lane should. What lands in
  `rcad-geom` instead is that `meet_of_three_surfaces` reads such a box as the
  refusal it is;
  `three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere`
  is the demonstration, and the same three planes written lower down are placed,
  so it is a **gap**.
- **§31.7 criterion 8** names `three_points_on_a_conic_come_back_in_the_order_they_run_round_it`.
  It is `three_points_on_a_circle_come_back_in_the_order_they_run_round_it`, with
  one beside it per remaining conic pair — a bar's cap circle and the circle
  where two balls cross — because which normal a crossing winds anticlockwise
  about is a different line of algebra for each.

**The traps for lanes 4 and 5**, which build the curved arrangement and then
selection and commit on top of this:

- **`meet_of_three_surfaces` puts its three surfaces in a fixed order before it
  works anything out, and the points come back running along `n₁ × n₂` for the
  two planes in *that* order.** So `MeetPoint::OfThreeSurfaces`'s `which` counts
  the same way whichever face asked, which is what §31.3 needs it to do — but
  only while the caller hands over the same `Surface` values. Two faces on one
  plane hold the same four numbers (§32.5) and that is what the order rests on.
- **`Plane::meet_of_three` is still the one authority on where three planes
  meet**, and `meet_of_three_surfaces` calls it rather than answering a second
  time. What is new is that its refusal is *read*: a determinant worked out in
  intervals encloses zero whether it is zero or not, and the same determinant
  summed exactly says which. So three planes that share a line now come back
  `MoreThanAPoint`, three that share nothing come back `Nowhere`, and a corner
  Cramer's rule could not bound is placed off the line instead. **Lane 4 can
  switch `bake.rs` onto this and lose nothing**, because the planar path through
  it is bit-for-bit `Plane::meet_of_three`.
- **`MeetOfThree::MoreThanAPoint` is an answer and not a refusal.** Two faces on
  one plane with a curved surface across them share a whole curve, and so does a
  plane pair whose line runs down a pipe's own wall. A consumer that treats it as
  a failure will refuse a fuse that has no corner there rather than one that
  cannot be done.
- **A marched closed loop runs the other way round when its pair is swapped, and
  a conic does not.** §34.4. Whichever of the two a lane is holding, the
  direction to trust is the one §31.3 settles by the material on the left.
- **`which_comes_first_round_a_crossing` refuses a quartic**, and that refusal is
  a **gap**: the two loops are there and ordering points along one takes the
  marcher rather than a determinant. Lane 4 meets it the first time a corner
  lands on a ball-and-pipe crossing, which is a ball with a bar through it.
- **`where_a_corner_lands_after_moving` hands back a box and an `Approximation`,
  and the box is not what a vertex stores.** §31.8's prediction — that a mesh
  domain still holds no constructed corner after M4 — rests on `write_into`
  storing `corner.point`. The box is for asking predicates; the middle of it is
  for storing. A lane that stores the box has changed what a model is.
- **`Picker` still has no `transformed`**, unchanged from §33.8, and this lane
  did not need one. It comes due at lane 4 exactly as that section says.

### 34.10 What this lane leaves, classified

Every refusal `meet_of_three_surfaces` and `which_comes_first_round_a_crossing`
can produce, limit or gap by name (§16.10):

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` | fewer than two of the three are planes | **gap** | `three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved` exhibits `(3, 0, 4)` exactly on a pipe, two balls and a plane in whole numbers, and refuses two triples through it. §31.2 defers it and names the consumer |
| `Undecidable` out of `meet_of_three_surfaces` | a product the exact arithmetic needs falls out of `f64`'s range | **gap** | the `2^k` family's 10.83%; the same triples on the lattice refuse none of it |
| `Predicate(PlanesDoNotMeetInAPoint)` | three planes meet at a point neither Cramer's rule nor the line route can bound | **gap** | inherited from `meet_of_three_planes`, and now reached far less often, because the line route divides by a determinant known to full relative accuracy rather than by an interval one |
| `Undecidable` out of `meet_of_three_planes`'s own `Ok` | three planes meet past the top of `f64`'s range, and Cramer's rule says `Ok` holding a box with an infinite side | **gap** | `three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere`; the same three written lower down are placed |
| `TheseTwoSurfacesDoNotCrossInAConic` | two planes, or a plane down a pipe's own axis | **limit** | the question is a different one and the arrangement's dominant-axis rule answers it |
| `TheseTwoSurfacesDoNotCrossInAConic` | a ball against a pipe, or two pipes | **gap** | the curve is there and ordering points along it takes the marcher |
| `TheCrossingOfTheseTwoSurfacesIsNotClosed` | a crossing that runs on for ever | **limit** | the vertex does not exist: such an edge is cut by corners at both ends rather than closing on itself |

## 35. Does a curved face hold this point, recorded 2026-09-01

Lane 3 of §31.6: §31.2's rows 10, 11, 12 and 15, written beside lane 2 and
against its absence rather than against its contract.

§31.10 calls row 10 the schedule risk of the whole milestone, says it has no
obvious route that avoids deciding on `(u, v)`, names three candidates, and says
**the lane's first day should be spent choosing rather than building**. It was.
§35.1 is the choice and the demonstration behind it, and the rest of this
section is what the choice cost and the four things nobody had noticed that fell
out of it.

### 35.1 The three routes, and the one that needed no curve followed

**Route 1, a ray *along the surface* from the point, crossing-counted against
the wires.** §31.10 already says it "turns the question into more three-surface
meets". It does, and the meets it turns into are the ones this milestone
**refuses**.

A ray along a ball's skin is the ball crossed with a plane through the point. A
boundary of a ball with a pipe through it — §31.7 criterion 13's own shape —
runs along where the ball crosses the pipe. So every crossing that ray has to
count is a point of ball, plane and pipe together, and
`meet_of_three_surfaces` refuses fewer than two planes by name (§34.10, first
row). `a_ray_along_a_balls_skin_needs_a_meet_of_three_that_this_milestone_refuses`
is that, with the same call and two planes in it as the control, because what is
refused has to be shown to be the degree and not the call.

That is a demonstration and not a paragraph, which is what CLAUDE.md asks of a
claim that something cannot be done. **The family of fixes is named and it is
one**: an algebraic solver for a plane and two quadrics, which §31.2 already
defers to the milestone that builds one. Nothing about ordering the arithmetic
differently reaches it, because what is missing is not accuracy but a root of a
quartic.

Route 1 is also wrong for a second and independent reason, and it is worth
having because it would still hold if the algebra were free: §31.6 says lane 3
is written against lane 2's *absence*. A row 10 built on `meet_of_three_surfaces`
would have made the two lanes sequential and turned the milestone's only
concurrency into a queue.

**Route 3, cutting the face into pieces the flat argument does reach.** The cuts
have to be *placed*, and placing one on a curved face is where that cut's surface
meets the two the face's boundary already runs along — the same meet, arrived at
from the other side. It is route 1 with an extra step.

**Route 2 is what landed, and the precondition §31.10 gives it is not the
right one.** §31.10 says it "only decides for a face whose arcs all curve the
same way", which is a convexity condition and would rule out a ball with a pipe
through it. The condition that actually holds is topological:

> Every surface that trims a face meets that face's surface in **whole closed
> curves, all of which are edges of this face**.

Then that crossing cuts the surface into exactly **two** pieces, the face is one
of them, and which one is a sign — `side_of_surface`, the predicate that has
existed since M1. So the answer to row 10 is: for each surface trimming the
face, is the point on the side the face lies on. Nothing is constructed, no
curve is followed, nothing is ordered along one, no three surfaces are met at
once, and no `(u, v)` is computed anywhere.

**Two things are checked rather than assumed**, and each is one of the two ways
the answer would otherwise be silently wrong:

- Every trimming edge is a **whole closed curve** — its two ends are one corner.
  An edge with two different ends runs along a *piece* of a crossing, and which
  piece a point is on is row 8's question about where a curve runs.
  `Error::AFaceTrimmedByACurveThisCannotFollow`.
- The edges running along one surface are **every** curve the two surfaces cross
  in, counted by asking `intersect`, which is the one authority on that (§24.3).
  One of the two disks a pipe leaves in a ball's skin is bounded by a whole
  closed curve and would pass the first check; "the piece of the ball inside the
  pipe" names *both* disks, so a face holding one of two loops is refused.
  `Error::AFaceMissesPartOfWhereItsSurfacesCross`.

Both refusals are **gaps** and §35.11 names what closes each.

### 35.2 Which side a face lies on is a fact only its corners hold, and only topology can pick the corner

The route needs one number per trimming surface: which of its two sides the face
is on. **There is exactly one place in the model that says**, and finding that
out is the most useful thing in this section.

It is not the wire. §31.3 settles that a curved face's invariant is "every wire
keeps the material on its left", and reading a side off that needs the wire's
direction at a corner — which for a **closed** edge the model does not have.
§30.8 item 10 records that a closed edge that is a hole in a curved face has no
direction rule; §31.3 settles what the rule should be and hands it to lane 4.
Until lane 4 lands it, a closed edge's own sense is whatever `walk` produced for
the pair in the order it happened to be handed them.

**It is not the neighbouring face's own orientation either**, and that one is
worth writing out because it is the first thing to reach for and it is nearly
right. At an edge, the face on `S` and the face on `T` bound the same wedge of
material; write `a` and `b` for their two outward normals. Where the edge is
**convex** the material is `{d·a < 0} ∩ {d·b < 0}` and the `S` face carries on
into `d·b < 0`; where it is **reflex** the material is the union of the two and
the `S` face carries on into `d·b > 0`. So the answer is the neighbour's
orientation, turned round exactly when the edge is reflex — and `a` and `b`
alone cannot say which, because a dihedral angle and its explement leave the
same pair of normals. What tells them apart is the sign of the edge's own
direction against `a × b`, which is the direction the model does not have. The
two routes are therefore one route, and both wait on lane 4.

**And it is not the bounded-side rule that the flat path uses either**, which
is the third thing to reach for and the one worth being explicit about, because
the flat path really does use it. A closed curve cut on a *plane* by a ball or a
pipe bounds the part of the plane inside that ball or pipe — one of the two
sides is bounded and the other runs on for ever, so which is which is settled
without asking anything. On a **closed** surface both sides are bounded: a
circle on a sphere leaves two caps and neither is the outer one. So the rule
that rescues a round cap is silent about a disk cut out of a ball, which is
exactly the face `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` refuses.

So it is read off a **corner of the face that is off that surface**. Every
corner is either on the trimming surface — the corners of the very edges running
along it are — or off it on the side the face lies on. One corner off it
answers, every other corner off it has to agree, and a face with corners off it
on both sides is refused rather than guessed.

**And which corners are "on it" is topology and not arithmetic, which is the
regression this lane owns.** Asked with a predicate it works on a solid stored
where it sits and fails on one that has been *placed*:
`Isometry::apply_to_point` rounds, so the corner that is exactly on a cap circle
lands a hair off the cap's plane, on whichever side the rounding chose, and a
placed bar's wall reads as standing on both sides of its own cap.
`a_bar_and_a_ball_moved_and_turned_still_mesh_into_closed_surfaces` caught it
within a minute of the sweep's new check going in. The fix is to skip the
corners of the edges running along that surface **because they are on that
edge**, which no rounding takes away. §19.1 again, arriving through a door
nobody had opened.

### 35.3 Rows 11 and 12: the only thing in classification that is built rather than read

Where a straight ray meets a plane, a ball or a pipe is one equation in one
unknown of degree at most two. Put `x = from + t·along` into the very expression
`side_of_surface` decides the sign of — `|x − centre|² − radius²`, or
`|(x − origin) × direction|² − radius²` — and what comes out is `a·t² + b·t + k`
either way, the pipe being the ball with the axis crossed out of every vector
first. So the surface a root is worked out *from* and the surface it is later
tested *against* are the same one written the same way, which is §23.8's pattern
kept rather than found again.

Row 12 is `v · ∇S` at the root, times the face's own orientation. A polynomial
in the stored numbers with no normalization in it, exactly as §31.2 says.

Three things about the arithmetic, none of them guessed in advance:

**A point already on the surface is a root, and the quadratic formula loses it.**
With `k` exactly zero the two roots are `0` and `−b/a`. Read off
`(−b ± √(b²))/2a` instead, the first one comes back as a box straddling zero,
because enclosing a square root steps outwards — and then every point on a bar's
wall grazes its own face and `the_middle_of_a_bars_wall_is_on_its_skin` is
`Undecidable`. Factoring is not an optimisation here; it is the difference
between an answer and a refusal. This is `rcad-geom`'s own warning about
`root_enclosing` — "widening every root by an ulp would cost the exact cases
their exactness" (§29) — arriving one level up in a different shape.

**A division has to keep its exact cases too**, for the same reason: a ray that
starts exactly on a cap plane divides a numerator of nothing, and a widened
nothing is a box straddling zero, which reads as a ray that might be about to
cross the very face it is standing on. So each end of a quotient is multiplied
back out through `two_product` and stepped outwards only when that proves it was
inexact.

**`root_enclosing`, `quotient_or_everything`, `dot_enclosing`, `cross_enclosing`
and `gap_enclosing` are `pub(crate)` in `rcad-geom`, and this lane wrote all five
again in `rcad-topo`.** `rcad_predicates::Interval::divided_by` would have saved
one of them and is `pub(crate)` too — deliberately, and its doc says why:
dividing by a range that touches zero leaves an empty or unbounded interval and
would break `Interval::sign`'s no-empty invariant for anybody who could reach
it. So that one is not a missing export but a decision, and the copy here checks
its divisor first exactly as every caller inside that crate does. That is five spellings of five questions and §23.8 says
what that leads to. It is not fixed here because `rcad-geom` has one owner during
the fan (§15.9) and this lane is not it. **The fix is one line of
`rcad-geom/src/lib.rs`** — the module is already a module — and §35.10 hands it
to whoever opens that crate next.

### 35.4 A round cap fans into no triangles, and a round hole fans into a lie

`FaceView::for_each_triangle` fans each wire's corners from an apex. It is what
ray casting counted crossings with, what `box_around_its_corners` measured, and
what the planar volume sums are built on, and **it reads every wire as a
polygon**.

A round cap's wire is one closed edge and one corner. It fans into **no triangles
at all**, so before this lane a ray fired at a bar walked straight past both of
its ends and the count came back as though the bar were an endless pipe. Nothing
had noticed because `classify` refused a curved solid at the door.

A round **hole** is worse than invisible. Its one corner fans into a triangle
with no area, so the outer polygon's own fan covers the hole and a ray is counted
through material that is not there. Nothing builds such a face yet; a block with
a round hole through it is §31.7 criterion 11's headline shape and lane 5 makes
one.

**A flat face keeps the triangle route only while every side of it is straight**,
and that is what keeps a polyhedron's answer exactly what it was: every corner is
a coordinate the model holds, every test is an `orient3d` over four of them, and
nothing is built. Everything else — every curved face, and every flat face with a
curved side — goes by where the ray meets the surface, which is a division and
hands the predicate after it a box. The two routes are not two answers to one
question: the second is the general one and the first is the special case that
costs nothing.

**`for_each_triangle`'s other callers are still on the polygon reading**, and
§35.10 says which.

### 35.5 §25.9's box came due, and it is not the mesh that pays for it

`box_around_its_corners` says in its own doc that it is not a box round a curved
solid, and §25.9 hands the real one to "the lane that builds a mesh". A ray
caster needs it before that: the early "outside the box" test is a *proof* that a
point is outside the solid, and a box that misses the bulge turns it into a
confident wrong answer.

`SolidView::box_around_it` is that box and it needs no mesh. A ball's skin is
inside the box round its centre a radius each way, whatever the face is trimmed
down to. A pipe's is a radius from its axis, and how far *along* the axis a face
reaches is settled by its corners — the distance along an axis grows steadily
everywhere on a pipe, so it has no high or low spot in the middle of a face and
reaches its own furthest on the boundary. A flat face is the box round its own
corners, as before, so a polyhedron gets the very same box and the very same
three ray targets it got yesterday.

### 35.6 What `Containment::NeedsFlatFaces` became, and what the two `unreachable!()` are

`NeedsFlatFaces` said "this solid has a curved face and casting a ray at one is
not something this does yet". It is gone, and `classify` fires rays at curved
faces. What replaces it is narrower and it is not nothing:
`Containment::CannotFollowAFacesTrimming`, which says one face of the solid is
trimmed in a way §35.1's two checks will not follow.

**It is deliberately not `Undecidable`, and the reason is the one the old
variant's doc already gave**: `Undecidable` means every predicate was asked and
none could prove an answer, and this means the question was never put. Folding
the second into the first would let a caller loosen a policy and expect it to
help.

`select.rs:420` and `assemble.rs:943` justified an `unreachable!()` on "the fuse
refuses curved solids at entry", and lane 4 takes that gate off.
**Neither is a broken internal invariant, and both are refusals**, because what
reaches them is a face of a *caller's* solid:

- **`select.rs`'s `settle_against_each`** returns `Ok(None)`. A face nobody can
  follow is not a piece that could be placed somewhere else, and reading it as
  `Outside` would drop material. `Ok(None)` is what the function already means by
  "this piece could not be settled", so the fuse refuses rather than commits.
- **`assemble.rs`'s `one_shell_is_inside_another`** joins the arm that abandons a
  corner and tries the next. Saying anything stronger there would be a second
  opinion about which corners are worth trying; running out of corners is already
  `Error::Undecidable` and that is the honest end of it.

Both are edited here rather than left broken, because the branch has to compile;
they are lane 5's files and lane 5 should read them.

### 35.7 §31.10 item 5, scored: nothing this lane builds reaches `mesh.rs`'s two claims

§31.10 item 5 says a fused curved face is the first with a hole in it and that
`CurvedFaceWithAHoleGoesTooFarRound` and `MOST_PIECES_A_FACE_IS_CUT_INTO` both
rest on statements about today's builders. **Neither is reached by anything here,
and one of them is now known not to be reachable by the shape everyone expected
to reach it.**

`a_washer_passes_the_full_sweep` builds the first solid in this kernel with a
round hole in a **flat** face — a bar with a round hole straight through it,
which is §31.7 criterion 11's shape with a round outline instead of a square one
— and it passes `Model::validate`, mesher and all. `CurvedFaceWithAHoleGoesTooFarRound`
is about a hole in a **curved** face, and a washer has none: its two walls have
one wire each and its two caps are flat. So the claim at `error.rs:239` — "nothing
this kernel builds is such a face" — is untouched, and what falsifies it is a
curved face with a hole, which is a ball with a pipe through it and is lane 5's.

`MOST_PIECES_A_FACE_IS_CUT_INTO` is not reached either, for the same reason: a
washer's curved faces go right round and are cut into four pieces each, which is
what the constant's own doc predicts.

**The washer is the useful part of that test rather than the refusal it did not
provoke.** It is the shape lane 5 commits, built two milestones early, and it
says the sweep and the mesher are ready for it.

### 35.8 The numbers

Sixteen tests, and what each of §31.7's criteria in this lane's reach cost:

| | | |
|---|---|---|
| **9** | one authority on whether a curved face holds a point, agreeing with a fine mesh | met. `whether_a_curved_face_holds_a_point_agrees_with_a_mesh_of_the_same_face`, over a bar's wall and a ball's skin, 400 samples all compared and 304 of them held — the split derived rather than eyeballed |
| **10** | point in a curved solid, with the ON case | met. `the_middle_of_a_bars_wall_is_on_its_skin` and `a_point_a_hair_inside_a_balls_skin_is_inside_it`, both named in §31.7 word for word, plus five more that pin the cases they do not reach |
| **11** | `Containment::NeedsFlatFaces` gone | met for its half. `Error::FuseNeedsFlatFaces` is lane 4's and stands |
| **15 (row)** | a wire keeps the material on its left | **half met, and §35.10 says which half.** The sweep checks that a curved face does not stand on both sides of something trimming it; that it winds the right way round is not asked, because a closed edge has no direction to wind |
| **16** | a lattice curved control at 0% | met for classification. `a_grid_of_points_against_a_bar_and_a_ball_is_decided_and_records_nothing`: 9826 points, none undecided, nothing recorded |

Family by family and never blended (§17.7), over `Policy::certified()`:

| Family | points | undecided | could not follow a face | recorded |
|---|---|---|---|---|
| lattice bar | 4913 | 0.00% | 0.00% | none |
| lattice ball | 4913 | 0.00% | 0.00% | none |
| turned and moved bar | 2197 | 0.00% | 0.00% | none |
| turned and moved ball | 2197 | 0.00% | 0.00% | none |

**The turned rows recording nothing is structural rather than lucky**, and worth
saying because a zero that is luck is a zero that moves. The grid is drawn on the
lattice and the solid is not, so every ray meets a turned surface in a box; a
certified policy accepts nothing, so a predicate handed a box either proves its
sign and says nothing or gives up. A record there would mean a decision had been
taken on something nobody proved.

| | when this lane opened | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1009 | **1045** |
| of those, this lane's | — | **16** |
| suite duration | 96.5 s | 98.1 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 699 | 746 |

**Lanes 2 and 3 share a working tree, and the workspace counts cannot be split
by reading them.** §34.7 records 1045 as well, and says so in its own row: the
two lanes commit into the same branch from the same directory, so each has
swept the other's files into a commit at least once. Nothing was lost — the
later commit carries the later content — but a lane in this position should
stage by path rather than by `--all`, and this one learned that after two
commits rather than before.

The sixteen tests come to about nine seconds of processor time, seven and a half
of which is criterion 9's mesh oracle. The suite's own ninety-seven seconds are
still the one `rcad-geom` row §33.5 and §34.8 describe, and nothing here touches
it.

**One cost is not in that table and lane 5 should know it.** Asking whether a
face holds a point calls `intersect` once per surface trimming it, to count the
loops, and every one of those answers depends on the face alone rather than on
the point. A washer classified once is 2 walls × 2 roots × 3 rays = 12 curved
queries, each over 2 trimming planes, so **24 `intersect` calls for 4 distinct
answers**. It comes to about 90 µs per classification of a bar, which is nothing
for a ray caster and is not nothing for a fuse that classifies a witness point
per piece per pair. Nothing here has a target (§24.16), so it is a figure and not a
task; what would make it one is a curved fuse that spends measurable time in
`intersect` naming the same pair over and over, and the fix would be to ask once
per face rather than once per query.

### 35.9 Corrections to §31, §32 and §33

- **§31.10 item 1** gives route 2 the precondition "a face whose arcs all curve
  the same way". That is convexity, and it is neither necessary nor sufficient.
  What the route needs is that every surface trimming the face meets it in whole
  closed curves, all of which are edges of the face — §35.1. A ball with a pipe
  through it satisfies it and is not convex.
- **§31.10 item 1** also says there is "no such reference point off a sphere that
  works for the whole face". That is right and it is the whole reason the route
  that landed is not a crossing count: a crossing count needs a place whose
  answer is known, and a closed surface has none. §31.3's "a split curved face
  may have no outer wire" is the same fact seen from the topology side.
- **§31.10 item 2** guesses that the winding check "may turn out to be row 9's
  determinant at each corner, in which case it is cheap". It is not row 9's
  determinant, because row 9 needs the arc's direction and a closed edge has
  none. What is checkable without one is §35.2's side agreement, and it is cheap.
- **§31.2 row 10** leaves its polynomial and its degree unsettled with a "?".
  **There is no polynomial**, and that is the shape of the answer rather than a
  gap in it: the row is decided by other rows' predicates — `side_of_surface`,
  and `intersect` to count loops — and adds no arithmetic of its own. That is
  the whole reason it costs what it costs.
- **§31.2 row 11** marks its degrees 4 for a ball and 6 for a pipe as guesses.
  They are guesses about the *discriminant in the stored numbers* and this lane
  did not need one: what it solves is the quadratic in the ray's own parameter,
  which is degree **2** for both curved kinds and degree 1 for a plane, and the
  discriminant's sign is read as an interval and never expanded. So the row's
  number is neither confirmed nor corrected here, and the caller does not pay it.
- **§31.8 item 9** watches `MOST_POINTS_A_MESH_WILL_HOLD` for the first solid
  with five edges on a curved surface. A washer has six of those — four circles
  and two seams — and it meshes without coming near the cap.
- **§33.8's third trap** — a picker's `Undecidable` rules out nothing — did not
  fire, because nothing in this lane reads `picked_by`. It is still true and lane
  4 still meets it.

### 35.10 The traps for lanes 4 and 5

- **A closed edge still has no direction rule, and row 10 wants one.** §35.2 reads
  which side of a trimming surface a face lies on off a corner of the face that
  is off that surface, and a face whose *every* corner sits on its trimming has
  none — one of the two disks a pipe leaves in a ball's skin is exactly that.
  Once §31.3's rule for which of the two faces walks a closed edge forward is in
  the model, the side can be read off the wire instead and that refusal goes.
  **That is the single change that widens row 10 most**, and it is lane 4's.
- **`for_each_triangle` reads every wire as a polygon**, and the fan of a face
  with a curved side is wrong in two different ways (§35.4). Ray casting no longer
  uses it for such a face. **`measure`'s planar sums, `box_around_its_corners`
  and `bake.rs`'s box round a face still do**, and each is sound only while the
  faces it is asked about are polygons — which `measure` enforces with
  `is_wholly_flat` and `box_around_its_corners` says in its own name.
  **`bake.rs`'s `bake_one_face` has no such guard**, and lane 4 is what first
  hands it a curved face. Measured against a bar: the box it builds round the
  wall is the box round the seam, a line the wall bulges a whole radius out of,
  and the box round a round cap is `Bounds::nothing()`, because one corner fans
  into no triangles at all. A broad phase reading either will skip a pair that
  crosses. `SolidView::box_around_it` (§35.5) is the box to read instead.
- **The five enclosure helpers are written twice**, once `pub(crate)` in
  `rcad-geom` and once in `rcad-topo/src/classify.rs` (§35.3). Whoever opens
  `rcad-geom` next should export them and delete the copy; until then, a change to
  one is a change owed to the other.
- **A face trimmed by a *piece* of a crossing is refused, and lane 5 makes them.**
  A bar with a notch cut in its wall by a block is the first: its wall's boundary
  runs along the block's planes for a stretch and then stops. Row 10 refuses it by
  name, `classify` hands back `CannotFollowAFacesTrimming`, and the fuse refuses
  rather than guesses. §35.11 says what closes it.
- **The sweep's new check is only asked of a face whose every side is a whole
  closed crossing, and that is not timidity.** A face trimmed by a piece of a
  crossing straddles that piece's surface quite properly. A lane that widens the
  check without widening the condition will refuse sound solids.
- **`Containment` has a fifth variant and every `match` on it has to grow an
  arm.** §35.6 says what the two in the fuse became and why neither is a panic.
- **A round wire on a flat face takes its outer-or-hole sense from stored order,
  and nothing checks it.** `what_a_flat_face_holds` counts the first wire `+1`
  and the rest `−1`, which is `FaceData`'s "the first wire is the outer one by
  fiat" read literally — and `check_flat_face` deliberately walks past a wire
  that closes on one edge, because a circle has no polygon to wind. So a washer's
  cap arriving from a *file* with its two wires the other way round inverts
  silently, where the same mistake on a polygon is
  `Invalid::OuterWireWindsAgainstTheNormal`. It is a gap in the sweep and not in
  the answer, and closing it wants the bounded-side rule §35.2 rejects for curved
  faces and keeps for flat ones.
- **`SolidView::box_around_its_corners` now has no caller inside the kernel** —
  only two tests and a benchmark, all three of which want a bounding box and
  would be righter on `box_around_it`. Two public methods with "box" in the name
  is §23.8's shape reproduced rather than closed, and whoever is next in `view.rs`
  should move those three and delete it.
- **The fuse still winds a point round a piece's outline itself.**
  `fuse/select.rs`'s `winding_around` is a second answer to the question
  `holds.rs` exists to be the only one of, and it cannot simply call this one:
  its outlines are made of the `PredicatePoint`s the arrangement solved for, and
  the walk here takes corners the model holds. Widening the walk to boxes is what
  would close it, and it is lane 5's to weigh.

### 35.11 Every refusal this lane can produce, classified

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `AFaceTrimmedByACurveThisCannotFollow` | an edge of the face has two different ends, so it runs along part of a crossing | **gap** | `a_face_trimmed_by_part_of_a_curve_is_refused_by_name`. The point is on the piece or it is not; what says which is ordering points along a curve (row 8) after meeting three surfaces (row 4), and §34.10 files both. A closed-edge direction rule does **not** close this one |
| `AFaceMissesPartOfWhereItsSurfacesCross` | the face runs along some but not all of the curves its surface and a neighbour's cross in | **gap** | `a_face_along_one_of_two_loops_is_refused_rather_than_named_as_both`. What closes it is a way to tell one loop from another, which `Picker` already does for an edge (§33.1) — the same idea applied to a piece of a face |
| `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` | every corner of the face sits on the surface trimming it | **gap** | the side exists and the model does not record it; §35.10's first trap says what would |
| `Containment::CannotFollowAFacesTrimming` | any of the three above, met while counting a ray | **gap** | the three above, one layer up |
| `Invalid::AFaceStraddlesSomethingTrimmingIt` | a curved face stands on both sides of a whole crossing that bounds it | **not a refusal at all** — a model whose topology and geometry disagree, which arrives from a file. `a_curved_face_standing_on_both_sides_of_something_that_trims_it_is_caught` |
| `Undecidable` out of `face_holds_the_point` | a sign about a box could not be settled | **limit** where the box straddles the surface and a **gap** where the arithmetic ran out; no generator here reaches either, and §35.8's four families refuse none of it |

## 36. The curved arrangement, recorded 2026-09-01

Lane 4 of §31.6: `fuse/bake.rs`, `fuse/cuts.rs` and `fuse/arrange.rs`, splitting
curved faces at their crossings, built under `Error::FuseNeedsFlatFaces` and
leaving it on for lane 5.

The headline is one sentence and it is §31.7 criterion 12: **a block with a bar
through it comes apart into a square with a round hole in it, the disk that came
out, and a wall in three rings**, and the same pair turned about an awkward axis
does the same with ellipses everywhere and nothing approximated.

§31.10's prediction was that M4's money goes to `fuse/arrange.rs`, *and not the
parts about lines*. **It is right, and §36.2 scores it line by line.** The
straight-line ideas were not where the names said they were.

### 36.1 What a face is now, and the two things §31.3 got wrong

§17.4's "everything that matters to a face is a pair of planes" is now the same
sentence with one word changed, all the way through: `BakedFace` carries a
`Surface`, `BakedFace::cuts` is a list of `Surface`, `BakedEdge::across` is the
surface of the face on the other side, `Piece` carries a `Surface`, and
`FaceMeet::carried_by` is a pair of them. `cuts.rs` no longer asks
`Plane::agreement_with` and then `planes_are_parallel`; it asks `intersect`,
which is the one authority on how two surfaces meet (§24.3) and settles the same
three cases in one call. Two faces on one **cylinder** are the coincident case
for free, which is what §31.3 predicted and it cost nothing.

**§31.3 is wrong about the seam, and the demonstration is `seams_are_left_out`.**
That section says a seam is an ordinary boundary edge whose second surface is
read out of its own `EdgeRun::PartOfWhereItCrosses::and`. It cannot be one: a
seam is used **twice by one face**, so both uses run along one arc of one
crossing in opposite directions, and an arrangement that took it as boundary
would have one stretch of one curve claimed by two sides of one wire at once — a
winding count reads that as `+1` and `−1` about the same region. What a seam is,
is where the surface's own two coordinates were cut open and not where the face
stops, which is the sentence `holds.rs` already says on the other side of the
crate. So it is left out, and a bar's wall comes back bounded by its two cap
circles and a ball's skin by nothing at all.

**§31.3 is also wrong that a corner has two routes and not three.** It settles
"three or more surfaces round it is re-derived, two is read out of the model".
A **pole of a ball** has *one* face round it and one surface, and the old
`planes_around_a_vertex` asserted at least three. What landed is "three or more
is re-derived, fewer is read and moved", with a pole handing the same surface
over twice — which is honest rather than a fudge, because the box round a moved
corner is derived over two chains and a pole's two chains are the same one.

One correction inside that: **a solid that has not been moved is not moved**.
`where_a_corner_lands_after_moving` publishes a worst-case reach and applies it
whatever the movement is, so a bar standing where it was built would have come
back as a box — and §17.7's whole lattice control is that a lattice model decides
exactly. Baking checks for the identity first.

### 36.2 §31.10's prediction, scored line by line

The prediction names three functions and says the vocabulary of `Line::axis`,
`forward_climbs_the_axis` and `first_arc_met` is "a straight-line idea sitting
inside a function whose name says nothing about it". Every one of those is a
**real** assumption. So are three the prediction does not name, and they are the
expensive ones. And two things that look like straight-line ideas turned out to
be nothing of the kind.

| What it was | Real assumption, or only a name | What it cost |
|---|---|---|
| `Line::axis`, `forward_climbs_the_axis` | **real** | `Crossing::order` is now `AlongAnAxis` or `RoundAClosedCurve`, and a closed one is ordered by `which_comes_first_round_a_crossing` from the place §34.4 made a function of the pair |
| `nodes_along`, `order_along` | **real**, and they are the axis rule again | a cyclic order is not a total one until somewhere is called first, so a closed crossing sorts by insertion against its own start |
| `first_arc_met`, `piece_around` | **real** | a ray needs a direction to fire along and a bounded face to hit, and a closed curve has neither. Kept for lumps that hold an endless curve, and a second route added beside it |
| `drop_the_lumps_that_are_only_cuts` | **real, and the name says nothing at all about it** | its argument is *"a cut line that runs through the face crosses its boundary to get there"*, which is true of a line and false of a circle. The circle a bar cuts in a block's face lies wholly inside it and touches no boundary, and dropping it lost the hole. Now "lumps of **straight** cuts alone" |
| `pieces`, `the_cycle_of_the_lump_that_is_wound_once`, `outer_lump` | **real, and the deepest of them** | a piece was one wire and its holes, read off the lump the face's first wire is in. A bar's wall between two cap circles has two boundary curves in two lumps and neither is outer (§31.3). A piece is now a **cell of the surface**: a group of walks, joined across lumps by which side of every closed curve each lies on |
| `cut_the_lines_into_arcs` | **real, in one word** | a closed crossing wraps, so `n` nodes leave `n` arcs and not `n − 1`, and one node on its own leaves one arc that starts and ends there. That one word is what a cap circle and a round hole are made of |
| `turns_anticlockwise` | **only a name**, exactly as §31.2 row 9 predicted | `orient3d` of three stored normals became `orient3d` of three gradients, and for three planes the gradients *are* the stored normals, so the arithmetic is bit for bit what it was |
| the cycle walk — `walk_the_cycles`, `next_around_the_cell`, `sorted_anticlockwise`, `half_of_the_turn` | **only a name**, and it is the largest single piece of the file | not one line changed for curvature. Keeping the region on the left and taking the next half-edge clockwise from the way back is a statement about a graph drawn on a surface and never about which surface |
| `count_the_windings`, `spread_the_winding` | **only a name** | crossing a boundary arc from the right of its wire to the left adds one, on a sphere as on a plane. What changed is what it counts *over* — cells instead of walks — and that is the row above, not this one |
| `relation_of_two_lines` | **only a name, and it is still the right answer** | §31.2 row 7: all planes and it is four exact minors, otherwise it is "are the two partners one surface" and `intersect` answers. Both are there and the planar path is untouched |

**So the money went where §31.10 said and not to what it named.** The two
functions it calls out by name — `piece_around` and `first_arc_met` — are still
in the file and still work; what had to be rewritten around them is what decides
*which walks bound one region*, which the prediction does not mention and which
is a third of `arrange.rs`.

### 36.3 Two walks bound one cell exactly when they agree about every closed curve

This is the new idea and it is worth stating on its own, because it is what
replaces the ray for everything a ray cannot reach.

A curve drawn on a face is **drawn in full** when it closes and none of its arcs
was thrown away. Such a curve cuts the surface into exactly two, so *which side
of it* is a sign — read off the half-edge where a walk uses it, and off any of
the walk's own nodes where it does not. Two walks in different lumps of the
drawing bound the same cell exactly when they agree about every curve drawn in
full.

A curve that runs on for ever is never drawn in full, and that is the load-
bearing word rather than a caveat: the stretch of a line past its outermost node
is not drawn, so the sign of it says the same thing on both sides of a face's own
rim and cannot tell the inside from the outside. Two things stand in for it. A
lump holding an endless curve is placed by `piece_around`'s ray, unchanged. And a
lump of closed cuts on a **flat** face is placed by counting how often the face's
own rim winds round one of its nodes.

Both of the ways that count can fail turned out to matter, and both are the same
shape as §35.2's regression — *a fact about the drawing that arithmetic cannot
see*:

- **A fan cannot answer about a point on one of its own chords, and every chord
  of a square runs through its centre.** `select.rs`'s `winding_around` fans from
  each corner in turn, and for a rectangle the chords are the two diagonals, so
  no apex helps. The circle a bar cuts in the middle of a block's face starts at
  a point that is often exactly there. What stands in its place is a crossing
  count along a half-line, which is `holds.rs`'s route over boxes — **a third
  spelling of one question**, and §36.7 hands the unification on.
- **The corner a half-line is aimed at is on it by construction**, and saying so
  is not an optimisation. The determinant that would work it out has the same box
  in two of its rows, and an interval evaluation of one like that straddles zero
  however far away everything is. Every corner of a placed solid is such a box,
  so before this was written down the count refused on every turned pair.

### 36.4 Four families, and none of them refuses

Family by family and never blended (§17.7). 256 drawn pairs per family, every
face of both solids cut, `Policy::certified()`.

| Family | faces cut | refused | what they named |
|---|---|---|---|
| lattice | 2304 | **0.00%** | — |
| turned | 2304 | **0.00%** | — |
| leaning | 2304 | **0.00%** | — |
| written at `2^k` | 2304 | **0.00%** | — |

**A zero that is luck is a zero that moves, so here is why this one does not.**
The turned family started at **22.09%** and every one of those refusals was about
a curve that reaches no part of the face it was hung on. A block wall a hair off
parallel to a bar's axis cuts that bar's cylinder in an ellipse **ten
quadrillion** across — the measured node was at `−4.2e16` — and three points on
one are so nearly in a line that ordering them is an honest refusal. The same
wall *exactly* parallel to it cuts two straight lines whose direction has to be
read off a gradient at a box that far out, and that is the other half:
`which way a crossing runs` at 235 of 2304, `order along a crossing` at 274.

A **cut** all of whose nodes are provably clear of the box round the face cuts
nothing off it, and is left undrawn. It is a proof and not a shortcut: a curve
that crossed the face's boundary would share a node with one of the crossings the
boundary runs along, and every meet of every pair has been worked out by then.

**The word "cut" in that sentence is load-bearing and was measured to be.**
Written as "a crossing", it drops curves the face's own rim runs along — and the
rim of a face is not optional however far its nodes are. Eight tests of the
planar suite said so, and they are the demonstration.

Two families were re-measured rather than drawn fresh, because the generator's
first version drew knife edges: a bar exactly tangent to a block's wall is
roadmap §7.4's own catalogue entry and not what a *control* is for (§17.7). The
bar is kept a whole unit clear, and the entry stays in the catalogue where it
belongs.

The intersector's own rates are untouched by this lane, which is what they should
be — `rcad-geom` has **0 files changed** — and so are §35.8's classification
rates.

### 36.5 `Picker` did not come due, and that is a finding rather than a reprieve

§33.8 and §34.9 both hand lane 4 the same trap: *"`Picker` has no `transformed`,
and this comes due at you, when a fused solid holds edges from two placed
arguments."* **It did not, and the reason generalises.**

Nothing in the arrangement ever moves a picker. Where two surfaces cross in more
than one curve, the arrangement asks `intersect` about the two surfaces *already
placed* and is handed the pickers of the moved crossing — so the only stored
picker a fuse could have read is the one saying which half of a meridian a seam
is, and §36.1 leaves seams out. The rule to carry forward is one sentence:
**ask the intersector about the surfaces where they are, rather than moving what
it said about them where they were.**

That leaves §33.8's warning intact rather than discharged. A `HalfSpace` still has
no canonical form, still must never be compared or deduped or keyed on, and
`Picker::side_of_point`'s `Undecidable` still rules nothing out — which is why a
boundary edge whose corner is held by two curves of one pair, or by none, is
`CannotTellWhichCurveOfACrossingAFaceRunsAlong` and never a guess.

### 36.6 What each criterion in this lane's reach cost

| | | |
|---|---|---|
| **12** | the pieces a curved face is cut into cover it and overlap nowhere | met. `the_pieces_a_curved_face_is_cut_into_cover_it_and_overlap_nowhere`, over every face of a block with a bar through it. The oracle needs no area and so reads the same on a curved face as on a flat one: every side is walked once, a side two pieces share is walked once each way, and a side nobody walks back along is on the rim |
| **16** | a lattice curved control at 0% | met for this lane, and the other three families are at 0% too. §36.4 |
| **11** | `Error::FuseNeedsFlatFaces` gone | **not this lane's**, and it stands. Everything under it is built |
| **13** | a fused curved solid meshes | not this lane's; nothing is committed yet |
| §30.8 item 10 | a closed edge that is a hole in a curved face has no direction rule | **closed**, and it cost one line of algebra rather than a predicate. `(∇S × ∇T)` crossed with `∇S` and dotted with `∇T` is `−‖∇S × ∇T‖²` by Cauchy–Schwarz, so **left of forward is always the negative side of the partner**, everywhere, with no case analysis — and a wire walks a closed edge forward exactly when the face's material is inside the partner. `a_bar_wall_is_bounded_by_two_cap_circles_that_run_opposite_ways` |
| §30.8 item 11 | is `Undecidable` out of `mesh` reachable | **still not**, and §31.8 keeps its lead. Nothing here changes what a committed vertex is |

### 36.7 What lane 5 has to be told

Written for a stranger, because the same head reading it a day later is one.

- **The gate is still on.** `Model::fuse` refuses a curved solid at
  `fuse/mod.rs`, and `Piece::plane()`, `select.rs`'s `outward_plane_of_face` and
  `assemble.rs`'s whole `Face` are still written in `Plane` behind it. Each of
  those `expect`s says so in its own doc. Taking the gate off is taking those
  three with it.
- **A piece may have no outer wire, and `Piece::wires` says so.** On a flat piece
  the first is outer and the rest are holes, which is what `Model::validate`
  reads. On a closed surface the order is the order the walks were found in and
  nothing claims more (§31.3). `check_curved_face` has no winding check and
  §35.8 records that half of criterion 15 as open; this lane did not close it.
- **The seam is gone from every piece and has to be put back.** A piece of a
  bar's wall that goes right round the cylinder needs one to be meshable, and
  `mesh.rs` is where that bites. Nothing in the arrangement can invent it,
  because a seam is a *choice* of where to cut the surface open, and the commit
  is the first place that choice belongs.
- **Three refusals are the shape of what is left, and all three are gaps.**
  `ACrossingThisCannotBeOrderedAlong` for a quartic — a ball with a pipe through
  it, and §34.9 named this lane as where it arrives. `CannotTellTheCellsOfAFaceApart`
  for two regions the sign rule joined that the surface keeps apart.
  `CannotTellWhichCurveOfACrossingAFaceRunsAlong` for a corner two curves of one
  pair both hold. §36.8 classifies every one.
- **A quartic closes, and finding a place to start on one is not being able to
  order it.** Reading those two as the same question gave a ball with a pipe
  through it **four** pieces where it has three, in silence, until they were told
  apart. Whatever else changes, `how_a_crossing_runs` must keep dispatching on
  the *pair* and not on whether a start could be found.
- **There are now three spellings of "how often does this wire wind round this
  point".** `holds.rs`'s crossing count over model corners, `select.rs`'s fan over
  the arrangement's boxes, and `arrange.rs`'s crossing count over the same boxes.
  §35.10's last trap asked for one and there are three. The fan is the one that
  cannot answer at a rectangle's centre; the two counts are the same algorithm
  over two point types. Whoever unifies them should keep the count.
- **`Bounds::provably_misses` is a proof and the box behind it has to stay one.**
  `SolidView::box_around_one_face` bounds a curved face by an argument per surface
  kind, and a flat face by its corners *plus the box of every face across a side
  of it that is not straight*. Tighten either and the drop in §36.4 stops being
  sound.
- **`stretches_of_a_line_inside` says nothing about a curved pair.** It measures
  stretches along an axis and a closed crossing has none, so it comes back empty
  and the self-crossing check `cuts.rs` runs is silent about a solid whose curved
  faces run through each other. A **gap**, and what closes it is a way to say
  where two arcs of one closed curve overlap.
- **The coplanar joiner is only asked about a wholly flat solid.** It is written
  in planes, and nothing in this milestone builds a curved solid with two faces on
  one surface. A curved *answer* of this fuse could have one, and it arrives as
  `AFaceMeetsItsNeighbourOnItsOwnSurface` rather than as a wrong arrangement.
- **`rcad-predicates` is still frozen**, at 0 files changed for five lanes now,
  and `rcad-geom` is at 0 for this one.

### 36.8 Every refusal this lane can produce, classified

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `ACrossingThisCannotBeOrderedAlong` | the two surfaces cross in a quartic — a ball against a pipe, or two pipes | **gap** | `a_ball_with_a_pipe_through_it_is_refused_rather_than_cut_wrongly`. The curve is there and closes; what is missing is an order along it, which takes the marcher rather than a determinant. §34.10 files the same one dimension down |
| `CannotTellTheCellsOfAFaceApart` | two regions of one face were joined that the surface keeps apart | **gap** | telling them apart takes a point inside each, which is a construction this arrangement does not make. Not a broken invariant, because a face that really did overlap itself looks the same from here and only one of the two is a bug in this crate |
| `CannotTellWhichCurveOfACrossingAFaceRunsAlong` | a boundary edge's corner is held by two of the curves its pair names, or by none | **gap** | a picker's `Undecidable` rules nothing out (§33.8), so both come back held. What closes it is a picker that decides, which is §33.5's own four rows |
| `AFaceMeetsItsNeighbourOnItsOwnSurface` | two faces on one surface with an edge between them | **gap** | they carry no curve for that edge to run along. The joiner takes such a pair apart for a flat solid and is not asked about a curved one; what closes it is a joiner written in surfaces |
| `AFaceMeetsItsNeighbourNowhere` | a face has an edge along a curve its two surfaces do not have | **not a refusal at all** — a model whose topology and geometry disagree |
| `TwoCurvesTouchWithoutCrossing` | two curves on one face are tangent at a node | **gap** | there is no anticlockwise order round such a node and so no way round the cell it bounds. Following one of the curves past the touch closes it |
| `Undecidable` naming `where three surfaces meet` | no triple of surfaces round a corner could be shown to meet at a point | **gap** | inherited from `meet_of_three_surfaces`, and §34.10 classifies it there |
| `Undecidable` naming `order along a crossing` | two points on one crossing could not be told apart or put in order | **limit** where the two are one point and a **gap** where the arithmetic ran out; §36.4's four families reach neither |
| `Undecidable` naming `which way a crossing runs` | the direction of a crossing worked out at a box straddles every axis | **gap** | it was the turned family's whole budget and is now unreached, because the curves that provoked it reach no face |
| `Undecidable` naming `which piece of a face a hole is in` | no ray through a lump of endless curves gave an answer | **gap**, unchanged from §17 |

### 36.9 Corrections to §31, §33 and §34

- **§31.3's seam.** A seam is *not* an ordinary boundary edge and its second
  surface is not read out of its own run. §36.1, with the demonstration.
- **§31.3's two routes for a corner.** There are three cases and not two: three or
  more surfaces round it, exactly two, and **one** — a pole of a ball, which the
  old code asserted could not happen.
- **§31.3's "a corner read out of the model and then moved is a box".** True of a
  corner that was moved. A solid standing where it was built is not moved, and
  giving it a box anyway costs §17.7's lattice control its exactness.
- **§31.10's prediction** is right about the file and wrong about the parts.
  §36.2 scores it: `turns_anticlockwise` and the whole cycle walk are only names,
  and what had to be rewritten is which walks bound one region, which the
  prediction does not mention.
- **§33.8's first trap and §34.9's last** say `Picker::transformed` comes due at
  lane 4. It does not, and §36.5 says why the reason generalises.
- **§34.9** says `Plane::meet_of_three` can be switched for
  `meet_of_three_surfaces` and nothing lost. True, and one thing gained that the
  trap does not mention: **three surfaces meet at two points and the old check
  kept the first.** `place_the_crossings` skipped a place if *any* node lay on
  both curves rather than if one lay *here*, which was right while three planes
  met once. A cap plane cutting a bar's own cap circle is the shape it shows on,
  and what it left behind was a line that entered a closed curve and never came
  out.
- **§35.10's second trap** — `bake.rs`'s box round a face has no guard — is
  closed, and the box now hangs off the solid because the neighbour across a
  curved side has to be found already placed. `SolidView::box_around_its_corners`
  still has no kernel caller and this lane did not delete it.

### 36.10 The numbers

| | at §35 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1046 | **1062** |
| of those, in `rcad-topo` | 312 | **324** |
| suite duration | 2.9 s | 2.9 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 746 | **775** |

`just default` is green, `cargo-deny` included.

`just fuzz 180` ran 5 274 327 executions in 181 seconds, found **29 new seeds**
and **no crash**. Coverage stayed at 2506 edges, which is what it should be:
`rcad-geom` has 0 files changed, so there is no new arithmetic for the corpus to
reach.

**One test had its number re-derived.** `how_often_a_certified_fuse_of_a_skew_pair_refuses`
asserts which predicate the skew family's refusals mostly name, and the doc
beside it records "30 naming it against 21 naming `side of plane`". The 30 has
not moved — it is the same question about the same straight crossings, under the
name `"order along a crossing"` rather than `"order along a line"`. The 21 beside
it came down to **12**, because the gathering step stopped asking
`Plane::agreement_with` and then `planes_are_parallel` about every pair of faces
and started asking `intersect`, which settles the same three cases in one call
and gives up in fewer places on the way. The assertion is on the leading cause
and the margin widened rather than narrowed.

### 36.11 The gate is off, and what that cost

Lane 5 of §31.6: `select.rs`, `assemble.rs`, `mod.rs` and `history.rs`, and the
five lines at the top of `Model::fuse` that §31.1 calls M4's one deliverable.
They are gone, `Error::FuseNeedsFlatFaces` is deleted, and **a block with a round
hole taken out of it is one solid that validates and meshes** under all three
rules, both ways round.

| | | |
|---|---|---|
| **11** | the gate gone, and a block with a round hole is one solid | met. `the_fuse_takes_a_curved_solid_now`, and `Error::FuseNeedsFlatFaces` has no variant left to assert on — what stands in its place is the operation working |
| **13** | a fused curved solid meshes and passes §23.4's oracle | met for the first of its two shapes. `a_block_with_a_round_hole_meshes_into_a_closed_surface`, through `the_mesh_closes_up`, for Union, Intersection and Difference — **and the volume of each**, which is the strongest thing there is to say about a curved answer and is what says the tube is not inside out. `a_ball_with_a_pipe_through_it` is refused by §36.9's first row and stays refused |
| **16** | the two rates, family by family | met, and stronger than asked. Four families, three rules, both ways round, **0.00% refused and nothing recorded** on all four — including the turned one. §36.14 |
| **12** | the pieces cover the face and overlap nowhere | met by lane 4 |

Three things a curved answer needed that a flat one never did.

**Which way a face walks a closed edge is not something its two ends can say**,
because they are one corner. It is read off which side of the partner the
material is on — §36.6's identity, already in the arrangement — and *turned round
for whichever of the two faces did not create the edge*, because "forward" is
`∇S × ∇T` for that face's own surface and the two faces name `S` and `T` the
other way round from each other. Without that, both coedges of every cap circle
came out forward.

**Which surface a face is on is geometry and which way round it is used is
topology, and only a plane welds them together** (§31.3's ON split, arriving in
the commit). A plane turned round is four negated numbers and the answer stores
it that way; a sphere has no reversed form, so a curved face that a rule turns
round carries the turn on its *shell* instead. The same split settles the ON
columns: for a flat pair `Plane::agreement_with` answers "one surface" and "same
way" together, and for a curved pair `intersect` answers the first and the two
faces' own orientations answer the second.

**A candidate point inside a curved piece has to be put back onto the surface**,
because a blend of corners on a sphere is not on it. That goes through the
surface's own two coordinates, which is `atan2` and is allowed for the reason §5
gives — *choosing* which point to look at decides nothing, and what follows
proves it is inside the piece with signs that never see a coordinate. What comes
back is a **box** [`Surface::ROUND_TRIP_ROUNDOFF`] wide, and the width is not
slop anybody accepted: a candidate a rounding off a curved face would be
classified confidently against a solid whose own face it is lying on, and `ON` is
the one column a ray was never what settled.

And a piece bounded by **one** closed curve has one corner, so every blend of its
corners is that corner and every one of them is on the boundary. Two families
were added for it: the middle of a round wire, which for a flat piece is where
the ball or pipe that cut it has its axis, and six points spread round a curved
surface from the middle of what corners there are.

### 36.12 The seam a fuse cannot invent, and where it went instead

A fuse's answer contains a **tube**: a ring of a bar's wall between two planes,
bounded by two curves that each go right round the cylinder and by nothing else.
It is the first face in this kernel with a hole in it that is not flat, and
§35.7 predicted lane 5 would make one.

**It has no seam and cannot be given one.** A seam is an edge between two corners
at a single longitude, and the two curves bounding a tube start wherever the pair
that cut each of them starts — which is `where_a_closed_crossing_starts`'s answer
for each pair separately and is not the same longitude twice. Inventing a corner
at a chosen longitude does not help either: the plane the seam would run in has
to hold both corners *provably*, and a plane built through two rounded points
holds neither.

So the seam went into the mesher, where it is a cut in the surface's own two
coordinates and not topology anybody has to hold. `cut_a_ring_across` cuts a ring
at four longitudes at once and hands back four patches with one loop each, which
is the shape everything below it already expected. **The rule §35.7 and §31.10
item 5 said lane 5 would falsify is falsified in exactly one place**: the
sentence *"nothing this kernel builds is such a face"* is gone, and what is still
refused is a face with a hole that is **not** a ring — one curve right round and
another that is not — because then there is nowhere to cut across to.

Three things that had to be right for the ring cut, each of them a fact about a
chain that goes right round rather than an arithmetic bound:

- **A corner's distance to a longitude is not measured modulo a whole turn.** A
  chain that goes right round has a first corner and a last a whole turn apart,
  and modulo a turn they are equally near everything — so which came back decided
  whether the next stretch went a quarter of the way round or three quarters.
- **The two chains are walked in longitude frames of their own**, a whole number
  of turns apart, and one is put into the other's frame before anything is
  measured. A patch that took them as they came stepped a whole turn crossing
  from one to the other, which in the surface's two coordinates is a boundary
  folded across itself.
- **Every stretch is put on the end unwrapped against what is already there**,
  which is what `arrive` does one level up and for the same reason.

**What is still refused, and it is the shape M4 does not reach:** a cap a plane
cuts off a **ball**. Its whole boundary is one circle of latitude, and its patch
is a rectangle whose top edge is the pole and whose two sides are one meridian
used twice — and the model says neither of those about the face. A ring has
somewhere to cut *to* and a cap does not.
`a_ball_cut_by_a_plane_is_refused_by_name_rather_than_meshed_wrongly` asserts it
of the whole family and not of one shape, because a refusal that only fires
sometimes is one somebody has to characterise (§14.7).

### 36.13 A confident wrong answer, and two corners that could not be exact

**The worst thing found in this milestone, and the fuse is what asked the
question in anger.** `classify_point` said a point outside a bar of radius two
was inside it. §4.1 ranks that below a crash.

`gap_enclosing(a, b)` is the step *from* `a` *to* `b`, and the two callers in
`classify.rs`'s `gradient_at` never turned it round — so a ball's and a pipe's
gradient pointed at the middle instead of away from it, and *which way a ray
crosses a curved face* came out backwards every time.

**It cancels for a solid whose faces are all curved**, which is why lane 3's
grids of 9826 points against a bar and a ball never showed it: reversing every
crossing flips the total, and `winding == 0` is still `winding == 0`. What shows
it is a ray that crosses **one curved face and one flat one** — a ray through a
bar's wall and out of its cap — and there the two contributions that should
cancel add instead. Nothing in the kernel fired such a ray until selection asked
where a piece of a block's face sits relative to a bar.

Two more, and both are corners a fuse builds that a builder never did:

- **A corner of a curved face is asked for a sign and never for a distance**
  (§35), so a *constructed* corner — which every corner a fuse solves for is —
  could not pass `check_curved_face` at all: it is a rounding off its own
  surface and the exact level says so. What it can be asked is whether the box
  it already carries straddles that surface, which is §17.2 read literally. An
  exactly placed corner is still asked exactly what it always was, because the
  box is only consulted after the exact point has failed.
- **Where a closed crossing starts is a walked point and not an exact one.** §34
  says so in as many words — *"it is a point and not a box, and that is not a
  promise it is exact"* — and reading it as exact made it a corner of a curved
  face that the sweep could certify was off that face's own surface. It is a box
  [`POINT_ALONG_A_CROSSING_ROUNDOFF`] wide, unless **both** surfaces certify the
  triple `Zero`, which on the lattice they do — and there the box would have
  turned every sign the arrangement asks about it into a refusal.

One thing changed shape rather than being fixed. `Assembly::commit` re-validates
what it wrote, but only in a debug build and only as a *panic*: it is there to
catch a bug in this crate, and a release build writes the answer out regardless.
A curved answer whose two faces along one side disagree which way they face is
exactly that, so **the skin is checked for balance before anything is written**
and comes back as a refusal. Refusing beats corrupting, and both beat crashing on
a caller's solid.

### 36.14 The numbers

Family by family and never blended (§17.7). 64 drawn pairs per family, three
rules, both ways round — 384 fuses each — under `Policy::certified()`.

| Family | fuses | refused | recorded anything |
|---|---|---|---|
| lattice, a bar through a block | 384 | **0.00%** | 0 |
| turned about an awkward axis | 384 | **0.00%** | 0 |
| written at `2^k` | 384 | **0.00%** | 0 |
| a ball resting on a block | 384 | **100%**, one named reason | — |

**The turned row is the one to read twice.** A turned curved pair refusing
nothing is stronger than §31.7 asked for, and it is not luck: lane 4 took the
arrangement's own turned rate to zero by dropping cuts that reach no part of the
face (§36.4), and everything above it is signs about boxes that either prove or
give up — and none of these draws reaches one that gives up. The families here
are deliberately clear of knife edges, which is what a *control* is for; the
catalogue entries live in `degenerate_fuses.rs` and are lane 6's.

**The ball row is a refusal and not a rate.** Every fuse of it refuses, for the
one reason §36.12 gives, and the test asserts both — that the reason is the only
one and that it fires every time.

The intersector's and the classifier's own rates are unchanged by this lane
except where §36.13's defect moved an *answer* rather than a rate.

| | at §36.10 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1062 | **1067** |
| of those, in `rcad-topo` | 324 | **329** |
| suite duration | 2.9 s | 2.9 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 775 | **793** |

`just default` is green, `cargo-deny` included. **`rcad-predicates` is still at 0
files changed for the whole milestone**, across six lanes.

`just fuzz 180` ran 5 242 652 executions in 181 seconds, found **18 new seeds**
and **no crash**. Coverage went from 2506 to 2507 edges, which is what it should
be: `rcad-geom` is at 0 files changed for lanes 4 and 5 together, so there is no
new arithmetic down there for the corpus to reach.

**One prediction settled.** §30.8 item 11 said M4 would make `Undecidable` out of
`mesh` reachable through a fuse's constructed corner; §31.8 predicted it would
not, because `write_into` stores `middle_of(point)` and a committed vertex has
always been an exact triple. **§31.8 is right.** `Assembly::Corner::point` is a
`Point`, `write_into` hands it to `add_vertex` unchanged, and the mesher reads
vertices and walked points that are ordinary `f64` triples — so a mesh domain
still holds no constructed corner after M4. The width goes on the vertex as an
`Approximation`, which is where §17.2 says it goes and where §36.13 now reads it
back from.

**§31.8 item 9 is still not reached.** `MOST_POINTS_A_MESH_WILL_HOLD` is four
million and the largest thing built here is a block with a round hole meshed at a
deflection of a hundredth. §35.9 already corrected the watch and nothing here
moves it.

**§30.8 item 6 did not come due.** A crossing that runs through itself needs two
equal pipes whose axes meet, or a ball reaching exactly across a pipe; the first
is refused by `intersect` and the second by the quartic ordering, both before
anything could reach the third `EdgeRun` variant. §31.9's deferral stands and its
consumer is still not this fuse.

**§35.6's two refusals hold with the gate off.** `select.rs`'s
`CannotFollowAFacesTrimming → Ok(None)` and `assemble.rs`'s abandon-a-corner are
both about a face of a *caller's* solid, and taking the gate off does not make
either of them a broken invariant — what changed is that they are now reachable,
by a fused curved solid handed back to the fuse. Neither fires in §36.14's four
families.

### 36.15 What lanes 6 and 7 have to be told

Both are test-only, in `rcad-topo/tests/`, and both build on this.

- **Three shapes work and two refuse, and the line between them is one
  sentence:** an answer whose every curved edge is a **whole closed curve** is
  built, and one with an *arc* of a curved crossing in it is refused. A block
  with a round hole through it, a bar cut square across by a block, and a blind
  hole are the first; a bar notched by a block and a ball cut by a plane are the
  second. Write the catalogue against that line rather than around it.
- **A ball cut by a plane refuses**, so roadmap §7.4's tangent-sphere entries
  reach `ACurvedFaceGoesRightRoundWithNoSeam` before they reach anything about
  tangency. That is a real answer for the table and not a hole in it, but it
  means the entry is measuring the seam and not the tangency.
- **The metamorphic suite's generators are flat** (`tests/pairs/mod.rs` builds
  profiles and extrusions), and a curved family needs a `Shape` arm of its own. A
  bar through a block, with the bar a whole unit clear of the block's walls, is
  the family §36.14 measured and the one that refuses nothing.
- **A turned curved pair refuses nothing at all**, so the properties that hand
  back `Ok(())` when there is no answer will really run over it. That makes the
  *comparison count* guard (`SOME_COMPARISONS_MUST_HAPPEN`) worth setting high
  for a curved family rather than low.
- **Two coincident curved faces are kept as two.** The joiner only ever joins two
  flat faces, because the walk that rebuilds a joined face's rings winds corners
  about a plane's normal and has no curved twin. A curved answer is therefore
  sound but cut into more faces than it has, and a property that counts faces
  will see it. What used to make joining urgent no longer does: §17.1 could not
  re-derive a corner with two surfaces round it and §31.3's second route reads
  one out of the model.
- **`Model::fuse` is no longer gated on anything**, so a test that asserted a
  refusal by name has nothing to assert. `the_fuse_takes_a_curved_solid_now`
  replaced the last one.

### 36.16 Every refusal this lane can produce, classified

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` | an edge of the answer runs along part of a curved crossing | **gap** | `a_bar_notched_by_a_block_is_refused_by_name`. Two corners on a circle leave two arcs with the same two ends, and what tells them apart is a half-space through the chord square to the crossing's own plane — which exists, and which cannot be *built* here, because both ends have to be provably on it once its offset is rounded and nothing rounds it in a direction it can prove |
| `ACurvedFaceGoesRightRoundWithNoSeam` | a curved face is bounded by one curve right round its surface and nothing else | **closed at §36.18 where the surface pinches**, and what is left of it is not a gap: a face bounded that way on a surface with *no* pole is half an endless tube and is not a face |
| `ACurvedFaceCannotBeCutSmallEnoughToMesh` | cutting a curved face is not making the pieces narrower | **gap, and no generator reaches it** | it was an `assert!` until this lane, on the grounds that no builder makes such a face; `validate` walks it and `Model` is `Deserialize`, so what reaches it is a caller's bytes |
| `Invalid::FacesDoNotAgreeWhichWayTheyFace` out of the fuse | the skin the answer would have does not balance | **not a refusal about arithmetic** — it is this crate saying it cannot assemble what it cut, before writing anything. A release build would otherwise have committed it |
| `Invalid::VolumeIsNotPositive` | a shell of the answer encloses nothing | unchanged from §17, and now summed over a **mesh** for a curved shell, because a bar's wall fans into two triangles on its seam and a round cap into none at all |
| `Undecidable` naming `a point inside a piece of a face` | no candidate could be proved inside a piece and placed | **gap**, unchanged from §17.4, and now with four families of candidate rather than two |
| everything lane 4 can produce | | | §36.8 |

### 36.17 Corrections to §31 and §35

- **§35.6** says `select.rs:420` and `assemble.rs:943` are refusals rather than
  panics because what reaches them is a face of a caller's solid. That reasoning
  was written against a gate that was still on; with the gate off it holds
  unchanged, and §36.14 records that neither fires in the measured families.
- **§35.7** says `CurvedFaceWithAHoleGoesTooFarRound`'s claim is untouched by a
  washer and that what falsifies it is a ball with a pipe through it. What
  falsifies it is a **tube**, which is what a block with a round hole leaves of
  the bar's wall — a simpler shape than the one predicted, and one this lane
  builds on its first try.
- **§31.10 item 5** watches two claims in `mesh.rs`. Both are now false as
  written: the face-with-a-hole one because a ring is cut across (§36.12), and
  `MOST_PIECES_A_FACE_IS_CUT_INTO`'s because it was an `assert!` on a path
  `validate` walks — a panic on a caller's bytes, which §16.11 says it may not
  be. It is a named refusal now.
- **§35's `check_curved_face`** asks a corner for a sign and never for a
  distance, "and that is the point". It still does, and what §36.13 adds is that
  a corner carrying recorded slop is asked about the *box* it carries when the
  exact point fails. Nothing is loosened for a corner a builder placed.
- **§30.8 item 11** predicted M4 reaches `Undecidable` out of `mesh`. It does
  not, and §31.8's competing prediction is the right one. §36.14.
- **§31.3's ON row** is settled in the commit as well as in selection: two faces
  on one surface facing the same way is `intersect` plus the two orientations,
  and for a plane it stays `Plane::agreement_with` because a reversed plane is a
  different four numbers.

### 36.18 A ball cut by a plane, and the direction rule that took

Lane 5b of §31.6, and it exists because CLAUDE.md says to fix a gap rather than
defer it. §36.16 left two, both classified as gaps by the lane that wrote them,
and between them they stopped the kernel cutting a ball with a plane — which is
about the simplest curved boolean there is.

**One of the two is closed.** `Error::ACurvedFaceGoesRightRoundWithNoSeam` is
gone and §31.7 criterion 13 is met in full: `a_ball_cut_by_a_plane_meshes_into_a_closed_surface`
takes the cap above and the cap below two different cuts, meshes each through
§23.4's oracle, and checks the volume against `πh²(3R − h)/3`.

**Both poles are in that table and that is the point of it rather than
tidiness.** The lane before this one shipped an arm that never turned a step
round, and it cancelled in every test that used only curved faces; the same trap
is here, one shape along. An arm that read the pole backwards would still mesh a
*north* cap into a closed surface of exactly the right volume — it would simply
never be asked about a south one. The south rows are where getting it wrong does
not cancel, and getting it wrong is what happened first.

#### What a cap needed, and what it turned out to need

A cap's patch in the sphere's own two coordinates is a rectangle whose top edge
is the pole and whose two sides are one meridian used twice, and the model says
neither: a [`WireStep::Pole`] and a seam are what say them and a fuse can invent
neither. So they go into the mesher, four times, where a pole is a corner and a
meridian is a cut — the same move §36.12 made for a ring, with the second loop
replaced by a point.

**Which of the two poles the face reaches costs no arithmetic.** A face keeps its
material on the left of its wire, and a surface's own two coordinates are
right-handed about its outward normal — `∂u × ∂v` is `R² cos v` times it, and
`cos v` is positive everywhere but the poles — so the left of "longitude
increasing" is "latitude increasing", and a loop running that way bounds the
piece **above** it.

**And that read something arbitrary, which is the finding.** Which of a closed
edge's two coedges is the forward one says only which of the two faces along it
came first: the two have to disagree, so within one solid it is fixed, and
against anything outside it is a coin toss. §30.8 item 10 has recorded that a
closed edge has no direction rule since M3 and §35.10 called giving it one *"the
single change that widens row 10 most"*.

There already was a rule and it was half of one: *"a closed edge is stored
running the way this face's longitude increases"*, where **this face** is
whichever of the two walked it first. A bar never showed the hole in it — its two
cap circles are both between a plane and one cylinder, so both were stored the
cylinder's way and the wall's rectangle closed, which is the case the rule was
written for. A ball cut by a plane leaves two caps in **two different fuses**,
and each was stored the way whichever face got there first ran.

The rule is now: **a closed edge runs the way the longitude of the curved surface
either side of it increases, and where both are curved `Surface`'s own fixed
order picks one.** It depends on the edge and not on the walk, it keeps the bar
for the reason the old rule kept it, and the fuse sets its coedge orientations
against the same rule — `which_way_a_closed_edge_is_walked` in `assemble.rs`,
which is §36.6's identity for the wire's direction in space and one dot product
for the stored one.

**What it does not yet do is get checked.** `Model::validate` does not ask a
closed edge whether it is stored the way the rule says, so a solid arriving from
a file may disagree with it — and the builders are not asserted against it
either, because nothing they build reaches the path that reads it. That is the
half of §30.8 item 10 still open, and it is now a *check* rather than a rule.

### 36.19 The arc, and how far it got

`Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` is **still there**, and
this section is the demonstration §16.10 asks for rather than the paragraph it
forbids. It was built, it works as far as the naming goes, and it was taken out
again — so what follows is measured rather than argued, and the next lane starts
from the far end of it.

**§36.16's own row is wrong and this corrects it.** It says the half-space that
tells two arcs apart *"cannot be built here, because both ends have to be
provably on that plane once its offset is rounded and nothing rounds it in a
direction it can prove"*. All three clauses are wrong:

- **The plane is nameable without constructing anything.** Two corners on a
  closed crossing leave two arcs, and the plane through the chord *square to the
  plane the crossing lies in* meets that plane in the chord line — which a conic
  meets at exactly its two ends, so the two arcs are strictly either side of it
  whichever is the longer. The crossing's own plane never has to be built: the
  tangent `∇S × ∇T` and the chord both lie in it, so `tangent × chord` is its
  normal and `chord × that` is the one wanted.
- **Which side this arc is on is a sign and not a search.** The wire leaves its
  first corner along `∇S × ∇T` or against it, by the material side (§36.6), and
  the reading of the plane grows or shrinks along that direction. No point on the
  arc has to be built.
- **The offset can be proved rather than rounded.** `Model::validate` asks that
  both ends read *on or in front*, so the offset comes **down** — and how far
  down is not a number anybody needs, because the loop asks the predicate about
  the ends as the boxes `walk` will ask about them as, doubles the step, and
  stops when the predicate says so.

**And `Model::validate` was never the blocker either.** Lane 3 already has the
sweep skip a face trimmed by part of a curve, by name, so an answer with arcs in
it passes.

**What actually stops it**, measured: with the arcs named, a bar notched by a
block builds an answer and refuses inside `rcad_geom::walk`, which is picking
which of the two arcs to march. The refusal carries two `side of a half-space`
records about boxes round the arc's own ends — and it refuses under
`Policy::tolerant` as well as under a certified one, so it is not a budget
question. The records are swept into the refusal by a later `ask` rather than
being the decision that failed, so they name where the reading was uncertain and
not what gave up, and **this lane did not localise it further**.

So the classification is unchanged and the family of fixes is narrower by three
of its four members: it is a **gap**, the naming is done, the sweep is done, and
what is left is one function in `rcad-geom` deciding which arc a picker picks.
The two obvious members of the remaining family are that `walk`'s midpoint test
is asked about a point too near the chord for a short arc, and that
`every_half_space_holds` records a reading that *rules nothing out* — which §5
says is a route not taken rather than a decision anybody failed to prove, the
same argument `select.rs` already makes about a ray it abandoned.

**Nothing half-built was left behind.** The picker code is not in the tree: it
had no test that could pass, and CLAUDE.md's rule about that is not a preference.

### 36.20 The quartic, weighed and left

`which_comes_first_round_a_crossing` refuses a quartic, so a **ball with a pipe
through it** is still refused and §31.7 criterion 13 is met for two of its three
shapes rather than all three.

Left, and classified a **gap** exactly as §34.10 already classifies it: the two
loops are there and are tellable apart, and what is missing is an *order along*
one. That is not a sign anybody can add — a degree-four loop has no conic frame
to sweep an area in, and ordering points along one is the marcher's job, which
makes it a change to `rcad-geom`'s `walk` and `along_a_crossing` rather than to
anything this lane owns. §31.2's own table puts it at the top of the degree
column and §31.9 defers the algebra it rests on.

**It is also not what blocked the shape this lane was asked for.** A ball cut by
a plane never asks it: a plane and a ball cross in a circle, which §34.5 orders
with one determinant. Taking the quartic here would have been a second
milestone's worth of marcher work bolted to a lane whose job was a cap.

### 36.21 The numbers

Family by family and never blended (§17.7), the same four families at 384 fuses
each so the rows are comparable with §36.14's.

| Family | fuses | refused | recorded anything |
|---|---|---|---|
| lattice, a bar through a block | 384 | **0.00%** | 0 |
| turned about an awkward axis | 384 | **0.00%** | 0 |
| **a ball resting on a block** | 384 | **0.00%** | 0 |
| written at `2^k` | 384 | **0.00%** | 0 |

The ball row was **100% refused** at §36.14 and is now a control like the other
three. That is the whole of what this lane is.

| | at §36.14 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1067 | **1067** |
| suite duration | 2.9 s | 2.9 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 793 | **805** |

`just fuzz 180` ran 4 836 184 executions in 181 seconds, found **12 new seeds**
and **no crash**. Coverage stayed at 2507 edges, which is what it should be:
`rcad-geom` is untouched, so there is no new arithmetic for the corpus to reach.

**The test count did not move and that is worth a line.** A test that asserted a
refusal of the whole ball family came out and a test that meshes four caps and
checks four volumes went in. A milestone that measures itself by test count would
have scored this lane at nothing.

`just default` is green, `cargo-deny` included. **`rcad-predicates` is still at 0
files changed for the whole milestone**, and `rcad-geom` is at 0 for lanes 4, 5
and 5b together — the closed-edge direction rule landed entirely in `rcad-topo`,
because what needed settling was which of two surfaces names the direction and
not how a crossing is walked.

**No test had its number re-derived.** Nothing measured moved except the ball
family's refusal share, which went from every fuse to none.

**Did widening anything break what lane 3 proved?** Nothing in `holds.rs`
changed in this lane, so nothing lane 3 proved was widened at all. What changed
underneath it is which way round a closed edge's walked points are stored, and
§35.8's four families of point classification, `point_in_a_curved_solid.rs` and
`meshing_a_curved_solid.rs` all pass unchanged — which is the answer that
matters, because a mesh whose two faces along an edge read the same points in the
same order is exactly what §23.4's promise is made of.

### 36.22 What lanes 6 and 7 have to be told, corrected

§36.15 stands except where this lane moved it.

- **A ball cut by a plane works**, so roadmap §7.4's entries about balls reach
  what they are about rather than reaching a refusal on the way in. §36.15's
  second bullet is withdrawn.
- **The line between what is built and what is refused has moved by one word**:
  an answer whose every curved edge is a **whole closed curve** is built, and
  that now includes a face bounded by one such curve with a pole behind it. An
  *arc* is still refused, so a bar notched by a block still is, and so is any
  entry that cuts a curved face twice with crossing curves.
- **A ball with a pipe through it is refused for a different reason from a
  notched bar**, and a catalogue that runs them together will read one refusal
  where there are two: the first is the quartic ordering (§36.20) and the second
  is the arc (§36.19).
- **The closed-edge direction rule is a rule and not yet a check.** A metamorphic
  property that builds a curved solid, writes it out and reads it back is fine;
  one that hand-builds a closed edge with the coedges the other way round will
  not be caught by `validate` and may mesh a cap to the wrong pole. Say so rather
  than assume it.

### 36.23 The arc, closed, and what was really in the way

§36.19 measured how far the last lane got and named two candidates for the rest.
**Neither was the cause**, and the localisation it asked for is one sentence: the
naming was right, the walk was right, and what threw it away was the
**coplanar joiner**.

An arc built by `Edges::of` carried the half-space that named it. Then
`join_up_what_is_one_face` ran, joined runs of arcs into one edge at every corner
only two edges leave, and handed the joined edge ends the half-space had never
been built for. §36.19's two candidates — a midpoint too near the chord, and a
picker's `Undecidable` ruling nothing out — are both about *reading* the
half-space, and the half-space that reached `walk` was for a different pair of
ends. Measured: a bar notched by a block joins exactly two arcs into one, twice,
and both of them are on the bar's cap circles where the block's planes cut them
and nothing else does.

**Both of §36.19's candidates are still true statements and neither fires.** With
the joined arcs named, the four families are at 0.00% and the notched bar records
nothing at all.

#### What names an arc, and why nothing on it is built

The half-space is the one through the chord **square to the plane the crossing
lies in**: that plane meets the crossing's plane in the chord line, a line meets
a conic at exactly two points, so the two arcs are strictly either side of it
whichever is longer.

Neither plane is ever constructed. For `v` a direction from one end **into** the
arc and `c` the chord, both lie in the crossing's plane, so `v × c` is that
plane's normal and `c × (v × c)` is the one wanted — which written out is

    facing = v(c·c) − c(c·v)

and whose dot with `v` is `|c|²|v|² − (c·v)²`, never negative by Cauchy–Schwarz.
**So the normal already faces the way the arc goes and there is no sign to case
out.** §36.19 wrote this with `v` the tangent `∇S × ∇T`; the generalisation to any
`v` into the arc is what makes joining work, and it costs nothing.

Which way the wire leaves its first corner is a sign and not a search: left of
`∇S × ∇T` is always the partner's negative side (§36.6), so it leaves along that
direction exactly when its material is inside the partner — which
`FaceMeet::material_side` already holds, written from which way round the
arrangement walked the arc and costing no arithmetic.

The offset is proved and not rounded, exactly as §36.19 said: it comes down from
the smaller of the two `f64` readings by a step that starts at one rounding of
the numbers involved and doubles, until the predicate itself says both ends read
*on or in front*. It stops once the total is past twice `|facing| · max|p|`,
which Cauchy–Schwarz puts past any reading there is, so a half-space not found by
then does not exist.

#### The joined arc, and the corner that is a point on it

A run of arcs joined into one is named the same way any one of its pieces is —
and **a corner the joining dissolved is a point on the joined arc**. So `v` is the
direction from a surviving end towards one of them, and still nothing is built:
that corner is one the arrangement placed.

An unjoined arc uses its own stored direction, which is the tangent. Two arms,
and both are walked by the shapes below: every arc of the notched bar's own
notch is unjoined and every arc of its cap circles is joined.

#### What is left of the refusal

`Error::ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` is still there and is now
one case: `facing` with no length, which means the chord lies along the
crossing's own direction. For a conic that is an arc so short the chord rounds
onto its own tangent — two corners a hair apart on the unit circle, where
`cos δ` is exactly one and `v(c·c) − c(c·v)` cancels to exactly nothing. A
**gap** (§16.10): the half-space exists and `f64` cannot reach it.
`an_arc_whose_chord_rounds_onto_its_own_tangent_is_refused_by_name`.

#### Two things §36.19 did not see

**A straight piece of a crossing is not an arc and must not be named as one.** A
block's `y = 1` plane cuts a bar's wall in two straight lines, and those are edges
between a plane and a cylinder with two different ends — which the old table sent
to the refusal. They are `EdgeRun::Straight`, whose own doc has always said
"straight between its two ends, which are the whole of it", and `walk` has always
had nothing to put between them. **The test is read off the surfaces and never off
the chord**, in the same three cases and the same spelling `walk` uses, because a
short enough arc has a chord that rounds onto its own tangent and this question
has to stay one about the pair.

That is also worth more than it costs: `holds.rs` refuses a wire with any
non-straight coedge, `classify.rs` fans a face whose every side is straight, and
`view.rs` widens a flat face's box for every side that is not. Calling a straight
side straight is right in all three.

**Which surface goes in `and` is decided by a check and not by taste.** The field
names the second surface for a run whose two faces are the *same* surface — a
ball's seam — and an arc's two faces already name both, so either is true. What
picks between them is that `Model::validate` asks both ends to read exactly `Zero`
against `and`, and a corner this fuse solved for is exactly on a plane it was
solved from and only within its own box on a quadric. The flat one goes in, and
`name_of_surface_bits` gives that for free by sorting a plane ahead of a ball or a
pipe.

A crossing with **two** curved surfaces and two ends is a pair of balls cut twice,
and nothing builds one: measured, two balls united refuse at
`ACurvedFaceGoesRightRoundWithNoSeam` long before an arc of them is named. When
that stops being true what has to give is the *check* — the same slop
`check_curved_face` already reads off a solved corner (§36.13), read in
`check_edges` too — and not the choice. Lanes 6 and 7 are told.

### 36.24 The direction rule stops being a rule, and the two things it was wrong about

§36.22 said the closed-edge direction rule was a rule and that `Model::validate`
did not ask a closed edge whether it obeyed it. It does now, with the very
function the fuse chooses by — `a_closed_crossing_is_walked_the_way_it_is_stored`
— because two spellings of one question is how those two come to disagree
(§23.8). `a_closed_edge_walked_the_wrong_way_round_its_surface_is_caught`, with a
control that flips *one* coedge and is caught by the older check instead.

Reading it back found two things the rule as written in §36.6 got wrong.

**The face's own orientation does come into it.** §36.6 says it does not: "a wire
winds about the surface's stored normal whichever way round its solid uses the
face". A sphere and a pipe have no reversed form, so a curved face the rule turned
round is *the same surface* with its wire stored walked the other way and
`FaceUse::orient` saying so. The rule has to be asked about the gradients the
**solid uses**, and compared against the coedge's turn **as stored** — the turn a
view composes back is the un-turned one. Both halves are load-bearing: the
block-with-a-round-hole answer fails the check on either one alone.

**`a_washer` had its hole's circles walked backwards**, both coedges of each, and
the comment in the fixture argued that it should. It survives every other check
because a wire that is one closed curve has no polygon to wind, so the outline and
the hole ran the same way round the cap and nothing looked. A hole runs the other
way round a face than its outline does, and this is the first thing in the tree
that can say so. §36.22's own warning — "may mesh a cap to the wrong pole" — was
about a property nobody had written, and the thing it warned about was already in
the tree.

**A ball turned inside out is still caught by the volume and not by this.** A ball
has no closed edge at all: its one seam runs pole to pole. So
`a_curved_solid_turned_inside_out_is_caught` is a two-row table now, and §23.9's
own claim — a negative meshed volume — is measured in it rather than waited for.

### 36.25 What each criterion in these two lanes cost

| | | |
|---|---|---|
| **13** | a fused curved solid validates, meshes and passes §23.4's oracle | **met for all three shapes in this lane's reach**. `a_bar_notched_by_a_block_meshes_into_a_closed_surface`, beside §36.18's ball cap and §36.12's block with a round hole. A ball with a pipe through it is still the quartic (§36.20) |
| **16** | the four families at 0.00% | met, unmoved, and the notched bar joins them as a fifth shape that decides everything and records nothing |
| §30.8 item 10 | a closed edge has no direction rule | **the rule became a check** (§36.24), and it found a fixture that had been wrong since it was written |

### 36.26 Every refusal these two lanes can produce, classified

§36.16's first row is **withdrawn**: it says the half-space that tells two arcs
apart cannot be built here, in all three of its clauses, and §36.19 already
corrected the clauses. What it refuses now is one case and not the shape.

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` | the chord of an arc lies along the crossing's own direction | **gap** | `an_arc_whose_chord_rounds_onto_its_own_tangent_is_refused_by_name`. The half-space exists; `f64` cannot reach it at that separation |
| `Invalid::AClosedEdgeIsWalkedAgainstItsSurface` | a face walks a closed edge against the way its surface winds | **not a refusal about arithmetic** — it is the sweep saying the model's topology and its geometry disagree, which §16.11 says arrives from a file |
| everything §36.16 lists apart from its first row | | | unchanged |

### 36.27 The numbers

Family by family and never blended (§17.7), the same four families at 384 fuses
each so the rows are comparable with §36.21's.

| Family | fuses | refused | recorded anything |
|---|---|---|---|
| lattice, a bar through a block | 384 | **0.00%** | 0 |
| turned about an awkward axis | 384 | **0.00%** | 0 |
| a ball resting on a block | 384 | **0.00%** | 0 |
| written at `2^k` | 384 | **0.00%** | 0 |

Not one of them moved, which is the point: an arc that used to refuse by name
appears in none of the four, and naming it broke none of them.

| | at §36.21 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1067 | **1070** |
| suite duration | 2.9 s | 2.9 s |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 805 | **815** |

Three tests net: the notched bar's mesh, the direction-rule check, and the arc
refusal's own demonstration. A fourth changed hands — the test that asserted the
notched bar was refused by name now asserts it is decided and records nothing.

`just fuzz 180` ran 5 210 761 executions in 181 seconds, found **10 new seeds**
and **no crash**. Coverage moved by one edge, 2507 to 2508, and that is a longer
run rather than new arithmetic: `rcad-geom` is untouched by these lanes.

`just default` is green, `cargo-deny` included. **`rcad-predicates` is still at 0
files changed for the whole milestone**, seven lanes now, and **`rcad-geom` is at
0 for lanes 4, 5, 5b and 5c together**. That is worth a line of its own: §36.19
said what was left was "one function in `rcad-geom` deciding which arc a picker
picks", and the answer was in `rcad-topo` — the picker was right and the joiner
threw it away.

**The notched bar, measured.** A bar of radius 2 and height 8 with a 2-cube
notched out of its wall meshes into 436 triangles at a deflection of 0.02, closes
up under §23.4's oracle in full, and measures **96.8665** against the
**96.8781** the closed form gives — 0.012% under, and under is where an inscribed
mesh belongs. Refined ten times it comes to 96.8769, which is 0.0012%.

**And the shape where the error would not cancel.** Every arc a block leaves on a
bar of that size is shorter than half a turn, so a picker that always took the
shorter of two would pass the notched bar and never be asked — the same shape of
hole as the north cap that meshed correctly with the pole read backwards
(§36.18). A wedge with its point at `(−1, 0)` opening in `+x` leaves an arc of
**228 degrees**, joined out of three stretches, longer than its own sibling. It
measures **87.6998** against **87.7111**, 0.013% out. Both rows are in one test
and the second is why the first is worth anything: with the direction into the
arc read backwards, **both** shapes stop being solids at all, refused by the
triangulator for a boundary that crosses itself.

### 36.28 What lanes 6 and 7 have to be told, corrected again

§36.22 stands except where these two lanes moved it.

- **A bar notched by a block works**, so the line §36.22 drew at "a whole closed
  curve" has moved: an edge along *part* of a curved crossing is built and named.
  Its third bullet is withdrawn.
- **The closed-edge direction rule is a check now**, so §36.22's last bullet is
  withdrawn too — and the thing it warned about was in the tree, in `a_washer`,
  and had been since that fixture was written. A hand-built curved solid is worth
  running `Model::validate` over before it is trusted, and two of the ones in the
  tree were not.
- **A ball with a pipe through it is still the quartic** (§36.20), and it is now
  the *only* shape in roadmap §7.4's reach that a curved fuse refuses for a reason
  of its own.
- **`check_edges` asks an arc's ends for an exact `Zero` against `and`.** That is
  what makes the flat surface of the pair the one stored there (§36.23), and it is
  a check a pair of *two* curved surfaces would fail on a corner that is only
  within its box. Nothing builds such a pair today. Whoever makes two balls
  fuseable closes `ACurvedFaceGoesRightRoundWithNoSeam` first and inherits this on
  the same day.
- **The coplanar joiner reaches curved edges and is written in planes.** It joined
  arcs of a cap circle correctly here only because this lane taught the join to
  re-name what it made. Anything else added to `Edge` that depends on its two ends
  has the same problem and will not announce itself.
- **A notched bar works on the lattice and a turned one does not**, and the
  reason is not the arc. Measured over twelve rotations about `(1, 2, 3)`, every
  one of them refuses inside the *arrangement* with `Undecidable` naming **order
  along a crossing** — which runs before anything names an edge, and refuses
  under `Policy::tolerant` as well as a certified one, so it is not a budget
  question. §34.5's determinant orders three points round a conic and a turned
  cap circle cut at seven places is where it stops. The turned family in the
  table above is at 0.00% because its bar passes *through* the block a whole unit
  clear of every wall and its cap circles are cut nowhere.
- **`select.rs` cannot place a point inside a flat piece bounded by one chord and
  one arc.** Two corners leave no polygon to wind and every candidate is rejected,
  so a bar cut by a single plane not through its axis refuses with `Undecidable`
  naming *a point inside a piece of a face*. §36.16 already lists that refusal;
  what is new is that arcs make it reachable from a shape as plain as that. A
  **gap**, and what closes it is a piece that is read by which side of each of its
  bounding surfaces it lies on rather than by winding — which is the rule
  `the_piece_holds` already uses for a *curved* piece, one dimension across.

## 37. Roadmap §7.4 over curved solids, recorded 2026-09-01

Lanes 6 and 7, test-only. `rcad-predicates` is at **0 files changed for the
whole milestone**, nine lanes now, and `rcad-geom` and `rcad-topo`'s `src/` are
untouched by these two.

**The headline is not the table.** Four defects came out of writing it, three of
them panics on solids a caller can build through the public API and one of them a
confident wrong answer, and §4.1 ranks a wrong answer below both. They are pinned
by tests that say in their names that they are wrong, and nothing here blesses
any of them.

### 37.1 The catalogue, as a table

`degenerate_curved_fuses.rs`, and both tables are printed by the tests that
assert them, so this section cannot drift away from what the kernel does without
something going red. Criterion 14's four first.

| §7.4 entry | the pair | rule | outcome |
|---|---|---|---|
| tangent spheres | two balls of radius two, centres four apart | Union | **answered with nothing, which is wrong** |
| tangent spheres, the control | the same two balls, three apart, really crossing | Union | refused by name |
| tangent cylinder–plane | a bar of radius two and a wall on `y = -2` | Difference | refused, `TwoCurvesTouchWithoutCrossing` |
| seam-crossing intersections | a bar notched across its seam | Difference | **answered** |
| seam-crossing intersections | a ball cut by a plane meeting its seam once | Difference | **answered** |
| seam-crossing intersections | a ball cut by a plane meeting its seam twice | Union | refused, `ACurvedFaceGoesRightRoundWithNoSeam` |
| seam-crossing intersections | a ball cut by the very plane its seam lies in | Union | **panics** |
| closed periodic surfaces | a block with a round hole right through it | Difference | **answered** |

And §7.4's other entries, asked again with a ball or a bar in them.

| §7.4 entry | the pair | rule | outcome |
|---|---|---|---|
| coincident curved faces | one bar built twice, in the same place | Union | refused, `Undecidable` on *a point inside a piece of a face* |
| vertex on a curved face | a `3 × 4` block corner on a bar of radius five | Union | **panics** |
| vertex on a curved face | the same corner on a ball of radius five | Union | **panics** |
| zero-length edges | two tools meeting on a plane that cuts a round wall | Difference | refused, `Undecidable` on *a point inside a piece of a face* |
| sliver faces | a bar cut by a slab a millionth thick | Intersection | **answered** |
| fully contained solids | a bar in the middle of a block, united | Union | **answered** |
| fully contained solids | the same, hollowed out | Difference | **panics** |
| fully contained solids | a ball in the middle of a block | Intersection | **answered with nothing, which is wrong** |
| touching, not overlapping | a ball resting on a block's top at one point | Union | **the ball is dropped, which is wrong** |
| touching, not overlapping | a block sitting on a bar's cap | Union | refused, `TwoCurvesTouchWithoutCrossing` |
| self-intersecting inputs | nothing in the public API builds a curved one | Union | discharged by the planar row |

### 37.2 How each entry is known to reach what it is about

§14.7's vacuity rule, and lane 5c's finding that the turned family reads 0.00%
because its bar passes clear of every wall. Every entry above is built so the
degeneracy is **exact** — shared whole numbers, a Pythagorean triple, or a plane
through a stored radius — and every one of them is paired with a control a tenth
of a millimetre off it. Where the two come back the same way the entry is not
reaching its subject and the row below says so rather than claiming a pass.

| entry | how it is known to reach it |
|---|---|
| tangent spheres | **it does not.** The two balls touch and neither cuts the other, so both are dropped, and the same nothing comes back when they are a mile apart. §37.4's first defect is the whole of this row |
| tangent cylinder–plane | moving the tangent wall from `y = -2` to `y = -1.9` changes the refusal to a different one **by name**. Two named outcomes either side of one number |
| seam-crossing | the notch across the seam and its exact mirror image on the far side agree to seven parts in `10^17`. A reflection carries the bar to itself and the notch to the other notch, and carries the seam nowhere — so any difference between the two would be the seam and there is none |
| seam-crossing, the ball | the crossing circle meets a ball's seam once for a cut by `z = c` and twice for a cut by `x = c`. One is answered and the other refused, and the two cuts differ in the axis and in nothing else |
| closed periodic | the answer is asserted to contain closed edges — edges that come back to the corner they left because they go right round the cylinder |
| coincident curved faces | the refusal names `(2, 0, 0)`, which is the bar's own seam corner, with an infinite enclosure: nothing was nearly decided |
| vertex on a curved face | `3² + 4² = 5²` puts the corner *provably* on the wall, and the same corner at `3.9` and `4.1` is answered either side on a bar and refused either side on a ball. Only the exact incidence panics |
| zero-length edges | the refusal names `(0, ±2, 0)`, which are the two rulings where the shared plane touches the cylinder |
| sliver faces | the slab is `2^-20` thick, exactly representable, so this is a sliver and not a rounding; the disc it leaves measures `4π · 2^-20` to a quarter of a per cent |
| fully contained | the bar in the same position gets two of the three rules right, so the ball's two wrong answers are the ball's shape and not containment |

### 37.3 The metamorphic results, family by family

`fuses_that_must_agree.rs`, and never blended (§17.7). The curved family is a
block with a round bar through it, the bar a whole unit clear of every wall,
generated by `a_curved_pair` — **a family of its own and not an arm of
`Shape`**, because an arm would have changed the corpus every measured family
draws from and every number §17.7 and §22 record with it.

| property | fuses | refused | not a solid | compared |
|---|---|---|---|---|
| a bar through a block, every rule, fixed seed | 423 | **0** | **0** | — |
| commutativity, a bar through a block | 244 | 0 | 0 | **122 of 122, 100%** |

**One hundred per cent is the number to read**, not the pass. It is the highest
reach of any property in the file — on the run recorded here the flat families
came in between 62% and 100%, with the depth-four chain the floor-setter, and
those run from a fresh seed so they move — and it says the property really
compared a curved answer bit for bit on every case it drew. What makes that
possible is the row above it: nothing in this family refuses, so both sides of
every comparison arrive.

Three of §31.7 criterion 15's names, scored:

| | |
|---|---|
| `swapping_the_two_sides_of_a_curved_pair_changes_not_one_bit` | **met**, at 100% reach |
| `uniting_a_block_and_a_bar_the_other_way_round_gives_the_same_solid` | **met**, at one fixed pair so a failure names a shape rather than a seed |
| `a_bar_with_a_copy_of_itself_taken_away_is_nothing_at_all` | **cannot be met today.** `A \ A` over a bar is refused — `Undecidable` naming a point inside a piece of a face, at the bar's seam corner. The test asserts the refusal by name, and the control is the same question over a block, which answers and leaves nothing |

**Union and intersection only, never difference.** `A \ B` and `B \ A` are
different questions; the first draft of the commutativity property asked all
three and the suite caught it on the second case it drew.

#### The properties that are *not* run over the curved family, and why

Not an oversight in any of the three cases, and each is the same shape of
argument §17.7 uses to keep inclusion and exclusion off the turned family.

- **Inclusion and exclusion, absorption, associativity, the chains.** All of them
  compare `invariants_of`, which is a volume, an area and a centroid read off a
  **mesh** for a curved solid. Two decompositions of one solid mesh differently,
  so the comparison is not exactly true and a bound on it needs a bound on how
  far two meshes of one shape can be from each other — which nothing here has.
  A tolerance guessed instead would make the test measure the tolerance (§14.1
  question 1).
- **Transform invariance.** A `Movement` is a quarter turn and a whole-number
  shift, exact on a plane's four coefficients and on a corner. Nobody has worked
  out what it is on a pipe's axis, and `Answer::moved_by` panics rather than
  guess.
- **`A ∪ A ∪ A ∪ …`.** Two bit-identical bars refuse, so it would compare
  nothing. That is the row above rather than a second finding.

Three solids at once was tried and is not here: a block with **two** bars through
it refuses with `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` before anything is
selected, so a curved trio property would assert nothing at all. Named here so
the next lane does not spend an afternoon finding out.

### 37.4 Four defects this lane found and did not fix

A metamorphic suite that edits the kernel until it goes green is worth nothing,
so none of these is fixed. Each is pinned by a test whose name says it is wrong,
and each has a control.

**1. A ball that nothing cuts is dropped by the fuse.** `fuse([ball], [], Union)`
comes back `Ok` with **zero solids**, no refusal, an empty log, and a model that
validates. A bar in the same position comes back whole. It is not classification:
the ball answers `Inside`, `OnBoundary` and `Outside` correctly for points at its
middle, on its skin and outside it. A ball is one face, its boundary is a seam
with a pole at each end, and the arrangement of that face against no cuts yields
no piece its own boundary winds once round.

Everything about a ball that does not cross its partner inherits it: two tangent
balls, a ball inside a ball, a ball inside a block asked what they share or what
is left, a ball resting on a block, a ball tangent to a bar. **This is a wrong
answer and not a refusal**, so §16.10's two words do not fit it — it is neither
a limit nor a gap but the thing both of them exist to avoid.
`a_ball_that_nothing_cuts_is_dropped_by_the_fuse`.

**2. A corner exactly on a curved face panics.** `arrange.rs`, *"two arcs leave
one node the same way, so they lie on top of each other"*. A `3 × 4` block corner
on a bar of radius five, and the same corner on a ball of radius five. The
controls are the same corner at `3.9` and `4.1`: on the bar both are answered,
on the ball both are refused by name, and only the exact incidence panics. A
panic on public input, which §16.11 says may not happen.
`a_block_corner_exactly_on_a_bars_wall_panics`,
`a_block_corner_exactly_on_a_balls_skin_panics`.

**3. A bar with a cap inside another solid panics on the way out.**
`assemble.rs`, *"a run of edges that became one edge has two ends left"*. It
reaches a bar wholly inside a block asked for the difference, a bar with one cap
inside a block under union and difference, and a bar cut a sliver in from its own
cap. **It is §36.15's own blind hole**, which that section lists among the shapes
that work — and it does work, in the arrangement, which is all
`a_blind_hole_cuts_the_face_it_stops_short_of_as_well` ever asked it.
`hollowing_a_block_out_with_a_bar_inside_it_panics`,
`a_blind_hole_panics_when_it_is_committed_rather_than_only_arranged`.

**4. A ball cut by the very plane its seam lies in panics.** *"the two coedges of
an edge must run opposite ways"* — `Invalid::CoedgesRunTheSameWay` arriving as an
assertion rather than as the refusal it is elsewhere. The same ball cut by any
other plane parallel to that one is refused by name, and cut square to it is
answered. `a_ball_cut_by_the_very_plane_its_seam_lies_in_panics`.

### 37.5 Every refusal these two lanes found, classified

The rows §36.16 and §36.26 already carry are unchanged. These are the ones the
catalogue reached that no section had placed.

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `TwoCurvesTouchWithoutCrossing` | a cut plane is tangent to a bar's wall, so two curves on that wall touch and do not cross | **limit** | `a_wall_that_touches_a_bar_without_crossing_it_is_refused_by_name`. There is no curve to walk: the two arcs meet and there is nothing between them, and the same wall a tenth off tangent refuses for a different reason by name |
| `TwoCurvesTouchWithoutCrossing` | a block sits exactly on a bar's cap, so its wall touches the cap circle | **limit**, the same one | `a_block_sitting_on_a_bars_cap_is_refused_by_name` |
| `ACurvedFaceGoesRightRoundWithNoSeam` | a cut meets a ball's seam twice, leaving a piece bounded by one closed curve and no seam | **gap** | `a_ball_cut_by_a_plane_that_crosses_its_seam_twice_is_refused_by_name`. The answer exists — it is two caps of a ball — and §36.18 closed the pinching case without closing this one |
| `Undecidable`, *a point inside a piece of a face* | two coincident curved faces | **gap** | `a_bar_and_a_copy_of_it_in_the_same_place_are_refused_for_want_of_a_point_inside_a_piece`. The answer is the bar. Under `Policy::tolerant` as well |
| `Undecidable`, *a point inside a piece of a face* | two tools meeting on one plane that cuts a round wall | **gap** | `two_tools_meeting_on_a_plane_that_cuts_a_round_wall_are_refused_by_name`. One tool making the same cut is answered, so the answer exists and the arithmetic could not reach it |
| `Invalid::VolumeIsNotPositive` | two balls crossing far enough apart that the answer's shell measures nothing | **not a refusal about arithmetic** — this crate saying it cannot assemble what it cut, before writing anything | `two_balls_that_really_cross_are_refused_by_name` |
| `ACrossingThisCannotBeOrderedAlong` | a ball with a pipe through it, §36.20's quartic | **gap**, unchanged from §36.20 | `a_ball_with_a_pipe_through_it_is_refused_by_name`, and **it is not a budget question**: `Policy::tolerant` refuses it too, which §36.20 did not measure |
| `Undecidable`, *order along a crossing* | a turned notched bar | **gap**, unchanged from §36.28 | `a_turned_notched_bar_is_refused_for_want_of_an_order_along_a_crossing`, tolerant too, and the same shape unturned is decided and records nothing |
| `Undecidable`, *a point inside a piece of a face* | a bar cut by one plane off its axis | **gap**, unchanged from §36.28 | `a_bar_cut_by_one_plane_off_its_axis_is_refused_for_want_of_a_point_inside_a_piece` |

`ResultIsNotManifold { count: 1 }` was reached once, by a notch whose wall cuts
right across a bar, and it is **an edge used once rather than four times** — a
hole in the skin rather than §17.4's four-faces-along-an-edge. It is not in the
catalogue because nothing here has worked out whether the answer exists, and
guessing which of the two words it is would be the paragraph §16.10 forbids.
Named so the closing gate can decide.

### 37.6 The numbers

| | at §36.27 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1070 | **1102** |
| suite duration | 2.9 s | **2.9 s** |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | 3 |
| fuzz corpus | 815 | 815, untouched |

Thirty-two tests: twenty-seven in the catalogue and five metamorphic.
`just default` is green, `cargo-deny` included. `just fuzz` was not run — neither
lane touches `rcad-geom`, so a longer soak would measure the same arithmetic
§36.27 already soaked.

**No measured number moved.** Lane 7 changed how an answer is written down for
comparison, which touches every family in `fuses_that_must_agree.rs`, so the six
fixed-seed rows were re-measured against a clone of the tree at `3b0e1ef` and
come back identical fuse for fuse: 423 fuses each; the three control rows at 0
refused; both leaning alike at 24; skew at 51; turned at 96; and the
not-a-solid shares at 4.7%, 17.0%, 4.0%, 4.0%, 1.9% and 4.0%. The rows that do
differ between the two runs are exactly the ones drawn from a **fresh seed every
run** by design, and they differ from one run of the same tree to the next.

**Writing an answer down by its surface rather than by its plane loses nothing
for a flat family.** A face's outward normal is its stored plane's normal times
the orientation's sign, so two faces agreeing about the plane bits agree about
the normal exactly when they agree about the orientation — and under a
`Movement`, which is a quarter turn and a whole-number shift, multiplying by `±1`
commutes with a swap and a negation. The comparison is the same one, one field
narrower and three surfaces wider.

### 37.7 Corrections to §31 through §36

- **§36.15's third shape does not work.** "A block with a round hole through it,
  a bar cut square across by a block, and a blind hole are the first" — the blind
  hole is arranged and never committed, and committing it panics (§37.4's third
  defect). The two others do work, end to end, and are in the catalogue.
- **§36.23 and §36.28 on two balls are right about one sub-family and not the
  shape.** "Measured, two balls united refuse at
  `ACurvedFaceGoesRightRoundWithNoSeam`" holds for two balls of radius two up to
  two apart. At three apart they refuse `Invalid::VolumeIsNotPositive`, and once
  they stop crossing at all — tangent, nested, or far apart — they come back
  `Ok` with nothing. So the sentence §36.28 hangs on it, that whoever makes two
  balls fuseable closes that refusal first, is only the first thing they close.
- **§36.20 left the quartic unmeasured under a tolerant policy.** It refuses
  there too, so it is not a budget question and no amount of slack reaches it.
- **§31.7 criterion 15's second name cannot be met**, and §37.3 says what stands
  in the way. The criterion is not withdrawn; the test is written to fail the day
  it becomes meetable.
- **§36.22's fourth bullet is withdrawn by §36.24 and by this lane both.** It
  warned that a hand-built curved solid might mesh a cap to the wrong pole
  without `validate` catching it; §36.24 made the rule a check, and every curved
  solid this lane builds goes through `Model::validate` after every fuse.
- **The doc comments on `fuses_that_must_agree.rs`'s six thresholds are stale**,
  and were before this lane: they quote 420 fuses, 20.71% turned, 14.29% skew,
  4.8%, 17.4%, 4.1%, 3.9%, 1.2% and 3.1%, against a tree that gives 423, 22.7%,
  12.1%, 4.7%, 17.0%, 4.0%, 4.0%, 1.9% and 4.0%. Every threshold still holds with
  room. `SKEW_REFUSALS_MUST_STAY_BELOW`'s doc also still carries the literal
  placeholder `SKEW_SOAK_PROSE`. Not touched here, because a number in a doc
  comment is somebody's derivation and re-deriving six of them is not this lane's
  work — but the closing gate inherits them.

### 37.8 What the closing gate has to be told

- **Three panics and a wrong answer are the milestone's exit risk**, not the
  refusals. §16.11 says a caller's bytes may not panic, and all four defects in
  §37.4 arrive from `Model::fuse` on solids the public API builds in two lines.
- **The order to close them in is the order they cost.** The dropped ball is a
  wrong answer and is worth more than the other three together; the corner on a
  curved face is the one an ordinary part reaches by accident; the blind hole is
  the one a user will hit first.
- **Two entries of roadmap §7.4 are not measuring what they are named.** Tangent
  spheres measures the dropped ball, and vertex-on-a-curved-face measures a
  panic. Neither is a hole in the catalogue — both are recorded — and only five of the nineteen rows are
  answered at all, so a closing gate counting rows will get the wrong number
  either way.
- **The curved metamorphic family is the strongest row in the file and the
  narrowest.** One shape, at 100% reach, and every degenerate arrangement of it
  is hand-written in the catalogue instead. Widening the generator to reach
  tangency or coincidence would take the reach to nothing, which is what §37.2's
  first row is about.

## 38. The four defects §37 found, fixed, recorded 2026-09-01

Lane 8. `rcad-predicates` is at **0 files changed for the whole milestone**, ten
lanes now, and `rcad-geom` is at 0 files changed for this lane — everything here
is `rcad-topo`, six source files and two test files.

§37.4 found four defects and reported them rather than fixing them, which was
right: a metamorphic suite that edits the kernel until it goes green is worth
nothing. All four are closed. Three of them were panics on solids the public API
builds in two lines, which §16.11 says may not happen, and one was a confident
wrong answer, which §4.1 ranks below both crashing and refusing.

**Two are answered and two are refused by name**, and which of the two is right
differs per defect. The rule this lane read them by is CLAUDE.md's: a refusal is
a *limit* when the answer does not exist and a *gap* when it does and the
arithmetic could not reach it. A gap gets fixed where the fix is the arithmetic;
where the answer exists and this kernel's *representation* cannot hold it, the
gap gets an honest refusal and a demonstration of what it would take.

| §37.4 | what it was | now |
|---|---|---|
| 1. a ball nothing cuts is dropped | the arrangement of a face with no boundary and no cut has no cell to read a piece off | **answered** — the whole surface, cut open along a seam of its own |
| 2. a corner on a curved face panics | two crossings that are one curve, so no order round the node exists | **refused**, `TwoCurvesTouchWithoutCrossing`, a gap |
| 3. a blind hole panics on the way out | a closed edge's one corner read as a corner to dissolve | **answered** — the blind hole and the bubble both |
| 4. a ball cut on its polar axis panics | a closed edge starting at a pole, where no longitude says which way it runs | **refused**, `AClosedEdgeOfTheAnswerWouldStartAtAPole`, a gap |

And §37.5's one unclassified refusal, `ResultIsNotManifold { count: 1 }`, is a
**gap** and is closed with it (§38.5).

### 38.1 A ball that nothing cuts comes back as a ball

`fuse([ball], [], Union)` came back `Ok` with zero solids, no refusal, an empty
log, and a model that validated. §37.4 called it *"a wrong answer, so neither
limit nor gap — the thing both words exist to avoid"*, and it swallowed four
catalogue rows: tangent spheres, a ball in a block under intersection and
difference, and a ball resting on a block.

**The cause is one sentence.** A ball's skin is one face whose one wire is a
seam, `bake` leaves seams out at the door — and it is right to, for the reason
`seams_are_left_out` demonstrates — so the arrangement of that face against no
cuts has no curve drawn on it, no node, no arc, no cell, and therefore no piece.
Nothing was wrong with any step; the empty arrangement simply had no meaning
attached to it.

It has one now: **an arrangement with no arcs at all is the whole of its
surface**, handed back as one piece with `Piece::is_the_whole_surface` set. Three
places read that word and no fourth needs to:

- `select.rs` says such a piece holds every point of its surface. Its wire is a
  seam, and a seam is where the surface's own two coordinates were cut open and
  not where the face stops.
- `assemble.rs` writes it out with a `WireStep::Pole` between the two walks of
  the seam, which is the only thing in the model a fuse had never built and the
  only reason the fuse could not hold a closed face before.
- the commit does nothing special: an uncut closed face shares no edge with
  anything, so it is always a shell on its own, and nesting, volume and meshing
  are the paths a ball already went down.

**The seam the answer carries is not the input's, and that is the load-bearing
choice.** The answer's face is written with no placement of its own, so its two
coordinates are read off the surface standing where it is — and a ball has no
frame of its own (`Sphere::point_at`'s own doc says so), so its poles are
`centre ± (0, 0, r)` in the world's axes and its longitude runs from `+x`
whatever the input was turned by. Carrying the input's seam through a rotation
would name a meridian of some other sphere, which is §17.1's mistake — re-using a
cached sample instead of re-deriving — one surface up. The poles are built by
interval arithmetic and collapse onto an exact triple when the sum does not
round, because handing back a rounded triple as an exact one is §34.6's mistake.

#### What is not load-bearing, measured rather than argued

Two things looked like they were and are not, and saying so is §16.10's rule
about "no ordering helps" applied to its own opposite.

- **Which half of the meridian the seam is.** Both halves run pole to pole, so
  both cut the surface open along a meridian and the patch is the same rectangle
  shifted. Flipping `material_side` on the two seam sides leaves all 368 tests of
  `rcad-topo` green.
- **Where the poles are** needs no shape of its own. Putting them on the equator
  instead — still on the ball, still on the meridian plane, so every check about
  *where* they are passes — is caught by the plainest ball there is: the
  triangulator refuses a patch whose corner stands on an earlier one, and every
  ball test in the file goes red at once.

#### The shape where the error does not cancel

A ball fused **on its own** exercises none of the three things that can go wrong,
and that is worth writing down because it is the obvious test to reach for.
`place_the_piece` returns `Outside` without asking anything when there is no
other solid, so selection never looks at the piece; nothing turns it round; and
its shell is never nested. Two shapes were found by perturbing the fix and
measuring which rows went red:

- **A ball wholly inside a block, hollowed out.** Making a whole-surface piece
  never turn round leaves every other test in the crate green and fails exactly
  this one, on two solids where there should be one: the ball comes back as a
  separate positive solid instead of a void, so the block gains its volume
  instead of losing it. `a_ball_wholly_inside_a_block_answers_all_three_rules`.
  This is also the shape §38.4's bubble is measured against, and the only row in
  the crate where a closed shell's winding is told apart by a volume rather than
  by an assert.
- **A ball touching a wall at `centre + (0, r, 0)`.** A whole sphere has one
  corner pair, so selection can offer nine candidates — three blends of the two
  poles, all of them the centre, and six spread a radius along each axis — and
  eight of the nine sit on the seam's own meridian plane or the wrong side of it.
  The ninth is that one. Put a wall exactly there and it comes back `OnBoundary`, which is a
  candidate that says nothing and is abandoned — so reading the seam as a
  trimming leaves selection with no candidate at all and the fuse refuses
  `Undecidable` on *a point inside a piece of a face* on a pair it can plainly
  answer. Removing the whole-surface rule from `the_piece_holds` fails this row
  and only this row.
  `a_ball_touching_a_wall_at_the_one_point_that_can_place_it_keeps_both`.

### 38.2 Two arcs that lie on top of each other

A `3 × 4` block corner on a bar of radius five panicked in `arrange.rs` on *"two
arcs leave one node the same way, so they lie on top of each other"*, and so did
the same corner on a ball of radius five. `3.9` and `4.1` were answered or
refused cleanly either side; only the exact `4.0` panicked.

**They do lie on top of each other.** The plane `x = 3` cuts a bar of radius five
in the two rulings `(3, ±4, z)` and the plane `y = 4` cuts it in `(±3, 4, z)`, so
both hold `(3, 4, z)` and the arrangement is carrying one line as two crossings.
There is no anticlockwise order round that node to be found, because the two
directions out of it along the two crossings are the same direction.

That is exactly what `Error::TwoCurvesTouchWithoutCrossing` already says, and the
same geometry one function away already refused with it: `half_of_the_turn`
returns it when the turn against the reference crossing is zero. The walk
asserted instead, on the belief that the first comparison had ruled the case out.
It had not — **a pair both anticlockwise of the reference is never compared
against it**, and that is the whole of the hole. The assert becomes the same
refusal, three lines, and the two sites now give one answer to one question
(§23.8).

**A gap and not a limit.** The answer exists and is the bar: a `3 × 4` rectangle
measured from the axis is inside a circle of radius five, so the block is wholly
inside the bar and the union is the bar unchanged.

#### The family of fixes, and why it is the only one (§16.10)

The arrangement's contract is **one crossing per curve**. Two objects for one
curve breaks the anticlockwise order at every node on that curve, so the only
family that closes this is *folding two crossings that are the same curve when
their partner surfaces differ*. `crossing_like` already folds a pair — for three
planes, by the rank of a three-by-four matrix of stored coefficients, and for
anything else by asking whether the two **partners** are one surface. Neither
reaches here: the two partners are two different planes, and what coincides is
one branch of each pair's crossing with the cylinder. Closing it needs a
predicate that says whether one branch of `S ∩ Q₁` is the same curve as one
branch of `S ∩ Q₂` for a cylinder and two planes, which is new arithmetic in
`rcad-geom` and is not this lane's to write in the same afternoon as four panics.

Nothing short of folding helps, and that is a statement about the shape rather
than about the arithmetic: the tie is exact — the determinant of the three
gradients is `det((3,4,0), (1,0,0), (0,1,0))`, which is zero in whole numbers,
not nearly zero — so no ordering, no scaling and no budget reaches it. Measured:
`Policy::tolerant(100.0)` refuses it the same way.

### 38.3 A closed edge that starts at a pole

A ball cut by `x = 0` panicked in the arena on *"the two coedges of an edge must
run opposite ways"*.

**§37.4's description of it is wrong and the correction matters.** It calls
`x = 0` "the plane the ball's seam lies in"; the seam lies in `y = 0`. What the
two planes have in common is the thing that does the damage: **both contain the
ball's polar axis**, so the great circle each of them cuts runs through both
poles, and the one corner the arrangement puts on that closed crossing is a pole.

A closed edge starts and ends at the one corner it has, so its ends say nothing
about which way it runs and only the surface's own parametrisation does — the way
its longitude takes the points along it, which is
`a_closed_crossing_is_walked_the_way_it_is_stored` and is the rule the fuse writes
with and `Model::validate` reads back with. At a pole every longitude runs
through the one point, so `ẑ × ∇C` is **exactly** the zero vector and the sign of
a dot against it is not a fact about the shape. Both faces along the edge came
back walking it the same way.

The rule now says it has nothing to read. The fuse turns that into
`Error::AClosedEdgeOfTheAnswerWouldStartAtAPole` and `Model::validate` skips it,
the same way it already skips a face with no corner off its partner. Only a
sphere reaches it: a cylinder's gradient on its own surface is radial and never
parallel to its axis, so its longitude never vanishes.

#### A gap, and the demonstration that the direction rule is not what makes it one

Handed a direction the rule cannot derive — patched in for the measurement and
taken straight back out — the same fuse gets two steps further and refuses
anyway:

| rule | with a direction handed to it |
|---|---|
| `Union` | `ACurvedFaceGoesRightRoundWithNoSeam` |
| `Intersection` | `CannotMeshAFace`, `CornersCoincide` |
| `Difference` | `ACurvedFaceGoesRightRoundWithNoSeam` |

The piece is **half a ball bounded by one curve right round its skin with the
poles on that curve rather than inside it**. `cut_a_cap_up_to_its_pole` cuts a cap
into four patches up to the pole *inside* it, and there is no pole inside this
one. So the answer exists as a shape and no face of this kernel holds it, which
makes this a gap of the same kind as §37.5's `ACurvedFaceGoesRightRoundWithNoSeam`
row. Closing it takes the other half of what §38.1 built: a *cut* closed face
written out with a seam and poles of its own, where §38.1 only had to do it for an
uncut one.

#### The controls, and they are what say the entry reaches its subject

The same ball cut at its **equator** is answered — the great circle runs through
no pole, so each half is a cap with a pole inside it — and so is the same ball cut
by a plane through its middle leaning `0.4` of a radian. Two answers either side
of one incidence, with the same ball and the same knife moved, says this is about
the polar axis and not about cutting through the centre.

### 38.4 A closed edge has one corner, and it is not a corner to dissolve

§36.15's own blind hole panicked in `assemble.rs` on *"a run of edges that became
one edge has two ends left"*, and so did a bar wholly inside a block asked for the
difference. §36.15 lists the blind hole among the shapes that work; it works in
the arrangement and had never been committed, and §37.7 already corrected that
claim.

**A closed edge starts and ends at its one corner**, so it appears *twice* in the
list of edges leaving that corner. The joiner read a list of two as "two edges
meet here and nothing else does", dissolved the corner, and left the joined edge
with no ends at all. The word the code was missing is **different**: two
*different* edges meeting at a corner and nothing else is a run to fold into one;
one edge written down twice is a curve that closes.

Three more assertions downstream said the same straight-line thing in three more
places, and each is one word out in the same direction:

- *an edge whose two ends are one corner has no length* — true of a straight side,
  and a cap circle and the rim of a round hole are the counter-example;
- *a ring of a face has at least three sides* — `Model::validate`'s own rule is
  three **or one that closes**;
- *which way a face walks an edge is which of its two corners the walk starts at*
  — which says nothing at all when the two corners are one.

All four are fixed and none of them is a tolerance. This is a **correct answer**
and not a refusal: the blind hole and the bubble are ordinary shapes, the
arrangement already had them right, and the only thing wrong was the way they
were written down.

#### Where an error here would not cancel, measured

**Both ways of getting a closed edge wrong are caught before a volume is read**,
and that is worth recording because a volume is the obvious thing to lean on.
Dissolving its corner leaves the joined edge with no ends, which is the panic
§37.4 pinned; walking it the same way from both faces trips the arena's own rule
that two coedges run opposite ways. Measured: putting the corner test back leaves
every other test in the crate green and panics on the bubble and on the blind
hole, without ever reaching a mesh.

So the closed form is not what tells a right winding from a wrong one here — it
is what says the answer is the **right** solid rather than merely a valid one.
The blind hole is `512 − 3π` and a hole all the way through would be `512 − 8π`,
and both are solids. The shape where a winding *does* have to be told apart by a
volume is §38.1's bubble, and §38.1 records that measurement.

### 38.5 A chord is not the arc it spans

§37.5 reached `ResultIsNotManifold { count: 1 }` — an edge used **once** rather
than four times, which is a hole in the skin and not §17.4's four faces along an
edge — and left it unclassified because nothing had worked out whether the answer
exists.

It exists. The shape is a bar of radius two with a block on `1 ≤ x ≤ 3`,
`−1 ≤ y ≤ 7`, `3 ≤ z ≤ 5`, so the block's two inner walls cut the bar and the rest
of it sticks out: a bar with a rectangular tab. All three rules now measure it
against a closed form, and the shared piece has a tidy one — the disc of radius
two with `x ≥ 1` has area `4π/3 − √3`, the corner of it with `y ≤ −1` has area
`1 − √3 + π/3`, and the difference is exactly `π − 1`.

**It is a gap, and it is §36.28's chord-and-arc gap arriving as a wrong
classification instead of a refusal.** The block's top at `z = 5` comes apart into
the bit inside the bar and the bit outside it, and the outside bit has one *arc*
among its sides. A piece is kept by classifying a point provably inside it, and
the winding count that proves it reads the corners as a **polygon** — so the
middle of the chord across that arc counted as inside the piece, when it is on the
far side of the arc and inside the bar. The piece was classified `Inside`,
dropped, and the arc left with one face along it. The manifold check caught the
hole, so what came out was a refusal and not a corrupt solid; what it was refusing
was a wrong classification.

What rules the chord's middle out is a sign the arrangement had already written
down and this branch was not reading: `FaceMeet::material_side`, which the
**curved** branch of the very same function has always read. A flat piece now
reads it too, for every side of it whose partner is curved, before winding
anything.

**Candidates are only ever ruled out, never in**, so nothing is now proved inside
a piece that was not before — which is what makes this safe to do late. Measured:
no number in `fuses_that_must_agree.rs` moved by one fuse, and
`a_bar_cut_by_one_plane_off_its_axis_is_refused_for_want_of_a_point_inside_a_piece`
still refuses.

### 38.6 Every refusal this lane leaves, classified

§37.5's rows stand except where this lane moved them. These are the changes.

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `TwoCurvesTouchWithoutCrossing` | a `3 × 4` block corner on a bar or a ball of radius five, so two crossings are one curve | **gap** | `a_block_corner_exactly_on_a_bars_wall_is_refused_by_name`. The answer is the bar. Tolerant refuses it too, and the determinant is exactly zero, so it is not a budget question. §38.2 has the family of fixes |
| `AClosedEdgeOfTheAnswerWouldStartAtAPole` | a ball cut by a plane containing its polar axis | **gap** | `a_ball_cut_by_a_plane_down_its_polar_axis_is_refused_by_name`, with the equator and a leaning plane answered as controls. §38.3 measures what a direction handed to it buys, which is a different refusal two steps on |
| `ResultIsNotManifold { count: 1 }` | **withdrawn.** It was a chord standing in for an arc, and the pair it was reached by is answered | — | `a_bar_with_a_tab_on_it_is_answered_rather_than_left_with_a_hole_in_its_skin` |
| `ACurvedFaceGoesRightRoundWithNoSeam` | now also a face with **no** boundary at all on a surface that does not pinch — a plane or a pipe with nothing trimming it | **limit** | unreachable from the public API and reachable from `Deserialize`, which is why it is a refusal and not an assert (§16.11). No answer exists: such a face is unbounded |

The four rows §37.5 marked `gap` for coincident bars, two tools on one plane, the
quartic and the turned notched bar are all unchanged and all still open.

### 38.7 The numbers

| | at §37.6 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1102 | **1106** |
| suite duration | 2.9 s | **2.9 s** |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | **3, unchanged** |
| `rcad-predicates` files changed, whole milestone | 0 | **0** |
| `rcad-geom` files changed, this lane | — | **0** |
| fuzz corpus | 815 | 815 |

**No measured number moved**, family by family and never blended (§17.7), and
these are the same numbers §37.6 recorded against a tree four fixes ago:

| family | fuses | refused | not a solid |
|---|---|---|---|
| §17.7 control, lattice pairs | 423 | 0 (0.0%) | 20 (4.7%) |
| §17.7 degenerate, sharing coordinates | 423 | 0 (0.0%) | 72 (17.0%) |
| §17.7 control, somewhere else in `f64` | 423 | 0 (0.0%) | 17 (4.0%) |
| §17.7 headline, turned pairs | 423 | 96 (22.7%) | 17 (4.0%) |
| §22 headline, skew pairs | 423 | 51 (12.1%) | 8 (1.9%) |
| §22 control, both leaning alike | 423 | 24 (5.7%) | 17 (4.0%) |
| §37 control, a bar through a block | 423 | 0 (0.0%) | 0 (0.0%) |

**`just fuzz 180` was re-run because this lane touches kernel code**: 5 282 767
executions in 181 seconds at about 29 200 a second, and **no defect at all**. The
working corpus grew by fifteen inputs and coverage by seven edges. None of it is
checked in — this lane changed **no `rcad-geom` file**, so the arithmetic the
fuzzer draws over is the one §36.27 already soaked, and §30.7's rule holds — an
unminimised handful is not worth checking in. The shipped corpus stays at 815.

That every measured rate is unchanged is the number to read. Four fixes went into
the arrangement, the assembly and selection, and the one that could plausibly have
moved a rate — §38.5, which makes classification *stricter* — moved none of them,
because it only ever rules a candidate out and another candidate answers.

### 38.8 The six thresholds, re-derived

§37.7's last bullet: the doc comments on `fuses_that_must_agree.rs`'s thresholds
quoted numbers the tree no longer gives. Every threshold still held with room, so
these were stale prose and not failures — but a doc comment is a claim.

| quoted | measured |
|---|---|
| 420 fuses a family | **423** |
| turned refusing 20.71% | **22.70%** |
| skew refusing 14.29% | **12.06%** |
| lattice not a solid 4.8% | **4.7%** |
| flush not a solid 17.4% | **17.0%** |
| elsewhere not a solid 4.1% | **4.0%** |
| turned not a solid 3.9% | **4.0%** |
| skew not a solid 1.2% | **1.9%** |
| leaning alike not a solid 3.1% | **4.0%** |

Two derived claims went with them: `MEASURED_CASES`'s doc said each family is 414
fuses and the suite runs in about ten seconds, and both are now what they measure.

`SKEW_REFUSALS_MUST_STAY_BELOW` also still carried the literal placeholder
`SKEW_SOAK_PROSE`. It is filled in with a soak measured here — six fresh seeds at
ten times the corpus, giving **12.6%, 13.8%, 14.5%, 15.3%, 16.2% and 16.3%**, none
of them over the 18% threshold and the worst within 1.7 points of it. The turned
family's own soak was re-measured while the numbers were out: **23.4%, 23.4%,
23.9%, 24.0%, 24.0% and 24.7%**, so four of six now sit above its threshold where
the old prose said two of six did.

**One number is worth reading twice.** The turned family's headroom is now 1.3
points, which is six fuses, where the prose said 3.3 points and fourteen. That is
the measurement having moved and not the threshold slipping — it read 20.71% of
420 when it was written and reads 22.70% of 423 now — but six fuses is thin, and
the doc says so where it used to say fourteen.

### 38.9 Corrections to §31 through §37

- **§36.15's third shape works now.** §37.7 corrected it to "does not work"; the
  blind hole is committed as well as arranged, and so are the two other shapes
  §36.15 names. Both corrections stand in order: §36.15 was wrong when written,
  §37.7 was right when written, and §38.4 is what makes §36.15 true.
- **§37.4's fourth defect is misnamed.** *"A ball cut by the very plane its seam
  lies in"* — the seam lies in `y = 0` and the defect is reached by `x = 0` too.
  What the two have in common is the polar axis, not the seam, and the difference
  is not pedantry: a plane through the ball's centre that does **not** contain the
  polar axis is answered.
- **§37.4's third defect no longer reaches its third shape.** It lists "a bar cut
  a sliver in from its own cap" alongside the blind hole and the bubble. A knife
  whose face is exactly the bar's own cap plane refuses `Undecidable` on *a point
  inside a piece of a face* — §36.28's coincident-face gap, and it does so with
  §38.5's change taken back out too, so it is not this lane's doing. A knife that
  slices a sliver *in from* the cap without sharing its plane is answered, at
  `π r² h` to a part in ten thousand.
- **§37.4's first defect is understated in one direction and overstated in
  another.** It says a ball's classification is correct while its fuse is wrong,
  which is right; it does not say that `fuse([ball], [], Union)` — the smallest
  case — exercises neither selection nor nesting, so the obvious test for it
  proves the least (§38.1).
- **§37.5's `ResultIsNotManifold { count: 1 }` row is withdrawn.** It is §36.28's
  chord-and-arc gap, and the pair that reached it is answered (§38.5).
- **§37.8's ordering was right.** "The dropped ball is a wrong answer and is worth
  more than the other three together" is what this lane found: §38.1 is the only
  one of the four that took a new concept — a seam the fuse writes rather than
  reads — and the other three are one word each.
- **`Case::Panics` is gone from `degenerate_curved_fuses.rs`**, along with
  `Outcome::Nothing`. An outcome no row reaches is a row nobody can read, and both
  are §37 artefacts of the state this lane closed.

### 38.10 What the closing gate has to be told

- **The milestone's exit risk is closed.** §37.8 named three panics and a wrong
  answer as it. There is no panic left on public-API input in the curved
  catalogue, and no `Ok` with the wrong answer in it — the table the tests print
  has two outcomes again, answered and refused, and `Outcome::Nothing` no longer
  compiles.
- **Two of the four are refusals and both are gaps**, so neither is finished
  business. §38.2 wants a predicate that says whether one branch of `S ∩ Q₁` is
  the same curve as one branch of `S ∩ Q₂`; §38.3 wants the other half of what
  §38.1 built, a cut closed face written out with its own seam and poles. Both
  are `rcad-geom` and `rcad-topo` work of a size no lane should start after
  lunch.
- **§38.1 is the first time this fuse writes a `WireStep::Pole`.** Everything
  about a closed face the fuse could not hold before now has half a road built to
  it, and §38.3 is the shape waiting at the other end of it.
- **The turned family's threshold has six fuses of headroom** (§38.8). It is not
  wrong and it is not a target, but it is the number most likely to go red next
  for a reason that is nobody's regression.

## 39. What M4 actually cost, recorded 2026-09-01

*Written by M4's closing gate. §12 did this for M0, §16 for M1, §17.8 for the
fuse, §26 for M2 and §30 for M3.*

M4 is complete in the only sense §31.1 gave it: `Model::fuse` no longer refuses a
curved solid, `Containment::NeedsFlatFaces` is gone, and a block with a round
hole through it, a bar notched by a block, a bar notched by a wedge and a ball
cut by a plane are fused, validated, meshed and measured against their closed
forms. The suite went from 985 tests at M3's close to **1111**.

`rcad-predicates` is at **0 files changed for the whole milestone**, across ten
lanes and this gate. So is `rcad-mesh`.

**And it does not do what a reader of §31.1 would expect it to do.** A ball with
a pipe through it — one of the two shapes §31.7 criterion 13 names — is refused,
and so are a bar coincident with itself, a ball cut down its polar axis, a block
corner exactly on a bar, and a bar cut by one plane off its axis. §39.4 is the
table of all of it, and §39.9 answers the question roadmap §12 asks.

### 39.1 The sixteen criteria, and the test that discharges each

§30.1's shape. Every row names a test that fails if the claim stops holding.
**Fourteen are met, two are met in part, and none is unmet.**

The two partial rows are 13 and 15, and the reason is the same in both: the
criterion named a *shape*, and the shape refuses. Four of the fourteen are met
with something worth reading in the row — **1** is met by a type the gate did not
know would exist and cannot be met as written, **5** is met in §28.4's form
rather than to the letter, **8** is met with a defect underneath it the audit
found, and **16** is met only because this gate re-derived the number it is
judged against.

| | Criterion | | Discharged by |
|---|---|---|---|
| 1 | a normal written square to a direction survives storage | **met, by a type the gate did not know would exist** | `a_normal_written_square_to_a_direction_is_still_square_to_it_once_stored` on `HalfSpace`. The criterion says *"once a plane has stored it"*, and §32.3 demonstrates that a `Plane` cannot: canonical under scaling and exactly square are incompatible, and `a_normal_written_square_to_a_direction_stops_being_square_once_a_plane_stores_it` pins that too |
| 2 | one pipe described twice is one pipe | **met** | `the_same_pipe_built_from_a_longer_axis_direction_is_the_same_pipe` |
| 3 | the half-space gap closed, the figures moved | **met** | `two_lines_off_the_lattice_are_told_apart_by_a_half_space_written_square_to_them` replaced the demonstration that asserted the opposite, and the shipped thresholds came down: a planar pair lacking a half-space from 2% to 0.5%, a quartic one from 15% to 4%, and `Undecidable` out of a turned quartic pair from 8% to 0.2% |
| 4 | a comparison too wide to square is squared elsewhere, and the window has a test | **met** | `a_comparison_too_wide_to_square_is_decided_after_it_is_slid_up_the_range` and `an_expansion_wider_than_the_window_cannot_be_squared_at_any_scale` |
| 5 | where three surfaces meet, certified on all three | **met, in §28.4's form** | `every_point_where_three_surfaces_meet_is_on_all_three_of_them.rs`, three families and a lattice control. "Certified ON" is "never certified OFF" one dimension down, because a box round a corner no `f64` names has real width; on the lattice, where nothing rounds, it is met to the letter |
| 6 | the count of meeting points is a sign, and the lattice refuses none | **met** | `a_certified_meeting_of_three_lattice_surfaces_never_refuses`; `MeetOfThree::At` carries one point or two and the discriminant's sign picks |
| 7 | three surfaces this cannot solve refuse by name | **met** | `three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved` |
| 8 | points along a planar crossing ordered with no angle built | **met, and one authority short of what §23.8 asks** | `three_points_on_a_circle_come_back_in_the_order_they_run_round_it` and `three_points_on_a_bars_cap_circle_come_back_in_the_order_they_run_round_it` — a circle and an ellipse, no `atan2` anywhere. What the audit found under it is §39.3's fourth finding: `intersect` and `which_comes_first_round_a_crossing` decide "is this plane along the pipe's axis" two different ways and disagree |
| 9 | one authority on whether a curved face holds a point | **met** | `whether_a_curved_face_holds_a_point_agrees_with_a_mesh_of_the_same_face`, and `SolidView::face_holds_the_point` really is the one: `classify.rs`, `fuse/select.rs` and `validate.rs` all ask it and none has a second answer |
| 10 | point-in-solid over a curved solid, with the ON case | **met** | `the_middle_of_a_bars_wall_is_on_its_skin`, `a_point_a_hair_inside_a_balls_skin_is_inside_it` |
| 11 | the flat-face gates are gone and a block with a round hole is one solid | **met** | `a_block_with_a_round_hole_through_it_is_bounded_by_whole_circles`, which asserts one solid and the volume. `Error::FuseNeedsFlatFaces` and `Containment::NeedsFlatFaces` do not exist |
| 12 | the pieces of a cut curved face cover it and overlap nowhere | **met** | `the_pieces_a_curved_face_is_cut_into_cover_it_and_overlap_nowhere` |
| 13 | a fused curved solid validates, meshes and passes §23.4's oracle | **met in part** | Met for four shapes: `a_block_with_a_round_hole_meshes_into_a_closed_surface` under all three rules, `a_ball_cut_by_a_plane_meshes_into_a_closed_surface`, and `a_bar_notched_by_a_block_meshes_into_a_closed_surface` for a block and for a wedge. **The second shape the criterion names by test cannot be written**: `a_ball_with_a_pipe_through_it_is_refused_by_name` is what stands there, and §36.20's quartic ordering is why |
| 14 | roadmap §7.4's four remaining entries as a pass/refuse table | **met** | `the_four_curved_entries_as_a_table`, eight rows. Four are answered and four refuse, and every refusal is in §39.4 |
| 15 | the metamorphic properties over curved solids | **met in part** | `swapping_the_two_sides_of_a_curved_pair_changes_not_one_bit` and `uniting_a_block_and_a_bar_the_other_way_round_gives_the_same_solid` are met. `a_bar_with_a_copy_of_itself_taken_away_is_nothing_at_all` **cannot be**: two coincident cylinders refuse for want of a point inside a piece of a face, tolerant too, and the test asserts today's refusal by name with a block as the control that answers. It fails the day the refusal is closed, which is §19's shape and the right way to leave it |
| 16 | the two rates, family by family and never blended | **met, and re-derived at this gate** | `every_certified_fuse_of_a_lattice_curved_pair_is_decided_and_records_nothing` over three control families at 0%, and `how_often_a_certified_fuse_of_a_turned_curved_pair_refuses`. The turned rate is **0.00% of 384 fuses** and was judged against a ceiling of three fifths, written at §36.14 when the ball family beside it refused everything and never moved when §36.21 took that family to zero. It is asserted at zero now — see §39.6 |

**Criteria 5, 9 and 16 were named the load-bearing three and all three are met.**
The two that are not fully met are 13 and 15, and neither is a hole in the work:
both name a shape that refuses, both refusals are in §39.4 with a demonstration,
and in each case the test that stands there asserts the refusal rather than
quietly not existing.

### 39.2 What §31 got wrong, and the two predictions scored

§30.3 is the instruction manual and §31 followed it: mark every number confident
or guessed, name a test rather than claim a number, and list what the gate is
planning on top of a guess. That worked. **Every substantial thing §31 got wrong
is a number or an arithmetic route, and nothing it settled about meaning or
ownership had to be undone.** The list, because a settling document is worth what
its errors teach:

- **§31.2 row 4's degree was four too high** for two planes and a quadric —
  about 14 guessed, 10 measured — and §34.1 has the table.
- **§31.4's five candidates for `Plane` contained no fix**, and the fifth,
  "leave it", is what landed. The property the gate wanted went onto a new type
  that is not a surface (§33.1), which is the answer nobody at the gate saw.
- **§31.10 item 1's route 2 had the wrong precondition.** The gate said the
  gradient-determinant route decides "for a face whose arcs all curve the same
  way", which is convexity. §35.2 found the precondition is topological — a
  corner off the trimming surface — and has nothing to do with curvature.
- **`Picker::transformed` never came due.** §33.8 and §34.9 both said lane 4
  would need it. §36.5 says why it did not, and it is a finding rather than a
  reprieve: a seam is left out of the arrangement altogether.
- **§36.19 named the arc's blocker as one function in `rcad-geom`.** It was in
  `rcad-topo`, and the two candidate causes §36.19 offered were both innocent
  (§36.23). The picker was right and the joiner threw it away.
- **§31.3's seam is not an ordinary boundary edge**, and §31.3's "two routes for
  a corner" are three, because a pole of a ball has one surface round it.
- **§31.10 item 7 has not been answered.** Whether two tangent balls united
  coming back as two shells survives the manifold check is still a model this
  kernel calls valid and a machinist calls one part. The catalogue records the
  answer; nobody has said it is right.

**The prediction §31.10 made about where the money would go was right about the
file and wrong about the parts**, and §36.2 already scored it line by line:
`fuse/arrange.rs` was the place, and the cycle walk and `turns_anticlockwise`
were only names, while what had to be rewritten — which walks bound one region —
is not mentioned in the prediction at all.

**§30.8 item 11 against §31.8, settled.** §30.8 item 11 predicted M4 would make
`Undecidable` out of `mesh` reachable, because the first constructed corner in a
mesh domain would be a fuse's. §31.8 predicted it would not, on one line of
`assemble.rs`. **§31.8 is right, and this gate confirms it independently.**
`Assembly::write_into` calls `model.add_vertex(corner.point)` with a `Point` —
an exact triple — and the width goes onto the vertex as an `Approximation`
instead, which is where §17.2 says it goes. The only route to `Undecidable` out
of `mesh` is `refusal_for`'s `CannotTellIfLoopsTouch`, an exact `orient2d` over
two `f64` coordinates, and the exact level decides those. §26.6 item 4 named M3,
§30.8 item 11 named M4, and both were wrong for the same reason: what has to
change is what a `Model`'s vertices *are*, and that is still nobody's milestone.

### 39.3 What the claim audit found

Run before this section, per §16.10, in depth over `rcad-geom`,
`rcad-predicates` and `rcad-topo` — about sixty absence-arguing claims in the
first two and forty-seven in the third — and swept over `rcad-mesh` and `rcad`,
which changed **no files this milestone** and whose claims were last audited at
§26.5 and §30.4. Every `panic!` left in `rcad-mesh` is a `Surface`- or
`Decision`-grew-a-variant arm and all of them are dead against today's three and
four; the facade has none outside its doc examples.

It has now found real defects on five consecutive milestones. This time: **two
behaviour defects, one of them a wrong answer handed back as a success, three
panics on a caller's bytes, and seven sentences that read as care and answer the
wrong question.**

**The worst one, and it would have merged.** `fuse/assemble.rs:1397` said, and
§36.23 repeated: *"`Model::validate` asks that both ends read exactly `Zero`
against `and`, and a corner this fuse solved for is exactly on a plane it was
solved from."* It is not. A corner of a curved arc is `meet_of_three_surfaces`
of two planes and a quadric, which is `through + t·direction` over a divisor, and
it lands on the plane exactly only when the line's direction is axis-aligned. A
unit block turned a fifth of a radian and notched into a bar of radius one is
enough: `Model::fuse` came back **`Ok` with a solid that fails
`Model::validate`** in a release build, and panicked in a debug one. A corrupt
answer returned as a success is what §4.1 ranks below a crash.

The fix is the one that same doc comment names as future work — the slop
`check_curved_face` already reads off a solved corner (§36.13), read in
`check_edges` too — and it went in with `Undecidable` there ceasing to be
reported as a broken model.

**Why nothing caught it is the finding under the finding.** Every family in
`how_often_a_curved_fuse_refuses.rs` turns *both* solids by one rotation. That is
an affine map of the pair: it preserves incidence exactly, so a cut plane stays
square to the bar's own frame in every draw and every corner keeps a coordinate
`f64` holds. **No test in this repository had ever turned one solid relative to a
curved one**, in the whole of M4. The new family does, sweeps every answer
through `Model::validate`, and fails on the tree before the fix.

**Four more, each with a breaking input, none fixed here.** Three are panics
reachable only through `Model`'s `Deserialize`, and §39.7 hands them to M5:

1. `Model::validate` panics — *"two planes cross in a straight line, which never
   closes"* — on a cube with one extra closed edge spliced into two of its wires.
   `validate` is the untrusted import boundary and `validate.rs:26` says so.
2. `SolidView::mesh` and `SolidView::measure` panic in `rcad-geom`'s
   `NamedRun::new` on a bar whose cap circle was given two different ends.
   Nothing checks that a `RightRoundWhereItsFacesCross` edge starts and ends at
   one vertex, and `SolidView::mesh` is public in its own right.
3. `SolidView::mesh` panics `TooFewCorners` on a bar whose wall face carries a
   `Plane` instead of its `Cylinder` — a four-step wire that fans into one corner
   in the plane's two coordinates.
4. And one that is a refusal rather than a panic: `Exact::dot_of` gives up on a
   direction with a subnormal component, and the sentence beside it said *"no
   scaling of a **unit** direction reaches the answer"*. True, and about a family
   nobody proposed. Multiplying both triples by `2^300` rounds nothing, cannot
   change the sign of a dot product, and decides it. The fix is a slide inside
   `Exact::dot_of`, which `Exact::square_minus` has had since §32.1.

**Two behaviour defects fixed here besides the first.** `Exact::product_of`
stored `[0.0, high]` for every product that came out exact, and
`exponent_that_makes_its_square_exact` read that zero as the smallest entry and
answered "already as low as it goes" — so a number with a good square somewhere
else was refused where it stood, at any magnitude. No call site hands it a raw
product today, which is why a green suite never saw it; the doc that said an
expansion's entries are non-zero is what kept anyone from looking. And
`HalfSpace::through_a_point_square_to` handed a zero `towards` refused for want
of a normal with three good ones already in hand, because a zero triple is
exactly square to everything, vacuously, and won the ranking.

**One disagreement between two authorities**, which is §23.8's own failure mode
and no code review sees it because neither site is wrong alone. `intersect`
decides "is this plane along this pipe's axis" with the certified cascade over
the exact dot; `along_a_crossing.rs` and `walk.rs` decide it with `== 0.0` on the
rounded `f64`. An axis along `(1, 0, 3)` against a plane facing `(−12, −12, 4)`
is a pair of small whole numbers where the two disagree completely: `intersect`
names one closed curve and `which_comes_first_round_a_crossing` refuses it as
"not a conic". Pinned by a test that fails the day it is closed.

**Seven sentences rewritten and none of them a defect**, which is the other half
of what this audit is for. `MOST_PIECES_A_FACE_IS_CUT_INTO` named the one
condition that provably cannot reach it. `FaceView::plane`'s "a bug in this crate
rather than bad input" is three lines from outside. `a_point_off_the_plane` said
`None` needs a normal that is not a real number, when what it needs is a corner
at the top of the range — the step *starts* at the corner's magnitude and
overflows on the first try, with a good answer at a smaller one. `MeetOfThree`'s
"the two can never be confused" is true of every route through `intersect` and
not of the type. And `Plane::meet_of_three` can hand back `Ok` holding a box with
a side that runs the whole number line, which its `# Errors` did not say.

**The rule this run adds**, and it is stronger than "check the public API":
**a `# Panics` clause on a `pub` item needs a demonstration that the arenas
cannot hold the shape, not an argument that geometry forbids it.** `Model` is
`Deserialize` and the arenas hold whatever the bytes say. That is the fourth
milestone running in which this exact species has been the audit's main catch.

### 39.4 Every refusal M4 can produce, limit or gap

§30.5 is the precedent and §16.10 the rule: a **limit** where the answer does not
exist, a **gap** where it does and the arithmetic could not reach it. Thirty
rows. **Every inherited row was re-checked rather than copied, and three moved.**

Rows M3 could already produce are §30.5's and are not repeated except where M4
moved them.

**Where two or three surfaces meet — `rcad-geom`**

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` | fewer than two of the three are planes | **gap** | `three_curved_surfaces_meeting_at_a_point_are_refused_rather_than_solved`. §31.2 defers it and names the consumer: two capped bars crossing near a cap |
| `Undecidable` out of `meet_of_three_surfaces` | a product the exact arithmetic needs leaves `f64`'s range | **gap** | the `2^k` family's **10.83%** against 0.00% for the same triples on the lattice |
| `Predicate(PlanesDoNotMeetInAPoint)` | three planes meet at a point neither Cramer's rule nor the line route can bound | **gap** | inherited from `meet_of_three_planes` and now reached far less often |
| `Undecidable` read out of `meet_of_three_planes`'s own `Ok` | three planes meet **past the top of the range**, and Cramer's rule hands back a box with an infinite side | **limit — §34.10 said gap and was wrong** | `three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere`. §34.10 argued it a gap because the same three planes lower down are placed; that is a different triple with a different answer, and *this* triple's answer is a point at about `1e315` that no `f64` triple names. Whose it is: **`rcad-predicates`'**, which returns the box, and `rcad-geom`'s `meet_of_three_surfaces` reads it as the refusal it is. The frozen crate keeps it and §39.7 hands it on |
| `TheseTwoSurfacesDoNotCrossInAConic` | two planes, or a plane down a pipe's own axis | **limit** | the question is a different one and the arrangement's dominant-axis rule answers it |
| `TheseTwoSurfacesDoNotCrossInAConic` | a plane whose dot with the axis cancels in `f64` and not exactly | **gap, new at this gate** | `a_conic_the_intersector_names_is_refused_here_because_the_two_ask_it_differently`. The pair is a conic, `intersect` says so, and this refuses it |
| `TheseTwoSurfacesDoNotCrossInAConic` | a ball against a pipe, or two pipes | **gap** | the curve is there and ordering points along it takes the marcher |
| `TheCrossingOfTheseTwoSurfacesIsNotClosed` | a crossing that runs on for ever | **limit** | the vertex does not exist: such an edge is cut by corners at both ends rather than closing on itself |
| `Undecidable` out of `intersect` | a direction with a subnormal component underflows its own product | **gap** | `a_direction_with_a_subnormal_component_cannot_be_told_from_square_to_a_plane`, whose second half slides both triples by `2^300` and reads the sign |

**Whether a curved face holds a point — `rcad-topo`**

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `AFaceTrimmedByACurveThisCannotFollow` | an edge of the face runs along *part* of a crossing | **gap** | `a_face_trimmed_by_part_of_a_curve_is_refused_by_name` |
| `AFaceMissesPartOfWhereItsSurfacesCross` | the face runs along some but not all of the curves its pair crosses in | **gap** | `a_face_along_one_of_two_loops_is_refused_rather_than_named_as_both` |
| `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` | every corner of a curved face sits on the surface trimming it | **gap, undemonstrated** | the side exists and the model does not record it |
| `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn` | a **flat** face with a corner at the top of the range and a normal off the axes | **gap, new at this gate, undemonstrated** | `a_point_off_the_plane` grows its step from the corner's own magnitude, so the first try overflows and every one after it does; a smaller step reaches an answer. Reachable from `Deserialize` only |
| `Containment::CannotFollowAFacesTrimming` | any of the above, met while counting a ray | **gap** | the above, one layer up |
| `Undecidable` out of `face_holds_the_point` | a sign about a box could not be settled | **limit** where the box straddles the surface, **gap** where the arithmetic ran out | no generator reaches either |

**The curved arrangement and the commit — `rcad-topo`**

| Refusal | Cause | | Demonstration |
|---|---|---|---|
| `ACrossingThisCannotBeOrderedAlong` | a ball with a pipe through it — the quartic | **gap** | `a_ball_with_a_pipe_through_it_is_refused_by_name`. **Not a budget question**: `Policy::tolerant` refuses it too (§37.7). §36.20 and §34.10 |
| `Undecidable`, *order along a crossing* | a **turned notched bar** — §34.5's determinant, in the arrangement | **gap** | `a_turned_notched_bar_is_refused_for_want_of_an_order_along_a_crossing`, tolerant too, and the same shape unturned is decided and records nothing |
| `Undecidable`, *a point inside a piece of a face* | a **bar cut by a single plane not through its axis** | **gap** | `a_bar_cut_by_one_plane_off_its_axis_is_refused_for_want_of_a_point_inside_a_piece`. §36.28, and §38.9 says the coincident-face gap below is entangled with it: a knife whose face is exactly the bar's own cap plane refuses here too |
| `Undecidable`, *a point inside a piece of a face* | two coincident curved faces | **gap** | `a_bar_and_a_copy_of_it_in_the_same_place_are_refused_for_want_of_a_point_inside_a_piece`. The answer is the bar. Tolerant too. **This is what criterion 15's second name runs into** |
| `Undecidable`, *a point inside a piece of a face* | two tools meeting on one plane that cuts a round wall | **gap** | `two_tools_meeting_on_a_plane_that_cuts_a_round_wall_are_refused_by_name`; one tool making the same cut is answered |
| `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` | **an arc whose chord rounds onto its own tangent** | **gap** | `an_arc_whose_chord_rounds_onto_its_own_tangent_is_refused_by_name`. The half-space exists; `f64` cannot reach it at that separation. §36.24, §36.26 |
| `TwoCurvesTouchWithoutCrossing` | a plane tangent to a bar's wall, or a block sitting exactly on a bar's cap | **limit** | `a_wall_that_touches_a_bar_without_crossing_it_is_refused_by_name`, `a_block_sitting_on_a_bars_cap_is_refused_by_name`. The two arcs meet and there is nothing between them |
| `TwoCurvesTouchWithoutCrossing` | **a corner exactly on a curved face** — a `3 × 4` block corner on a bar or a ball of radius five | **gap** | `a_block_corner_exactly_on_a_bars_wall_is_refused_by_name`, `..._on_a_balls_skin_...`. The answer is the bar; the determinant is exactly zero and tolerant refuses it too. §38.2 |
| `AClosedEdgeOfTheAnswerWouldStartAtAPole` | **a ball cut by a plane through its polar axis** | **gap** | `a_ball_cut_by_a_plane_down_its_polar_axis_is_refused_by_name`, with the equator and a leaning plane answered as controls. §38.3 |
| `ACurvedFaceGoesRightRoundWithNoSeam` | a cut meets a ball's seam twice | **gap** | `a_ball_cut_by_a_plane_that_crosses_its_seam_twice_is_refused_by_name`. The answer is two caps of a ball |
| `ACurvedFaceGoesRightRoundWithNoSeam` | a face with no boundary at all on a surface that does not pinch | **limit** | such a face is unbounded and there is no answer. Reachable from `Deserialize` only |
| `CannotTellWhichCurveOfACrossingAFaceRunsAlong` | a boundary edge's corner is held by two of the curves its pair names, or by none | **gap, undemonstrated** | a picker's `Undecidable` rules nothing out (§33.8) |
| `CannotTellTheCellsOfAFaceApart` | two regions of one face were joined that the surface keeps apart | **gap, undemonstrated** | telling them apart takes a point inside each |
| `AFaceMeetsItsNeighbourOnItsOwnSurface` | two faces on one cylinder with an edge between them | **gap, undemonstrated** | what closes it is a joiner written in surfaces |
| `ACurvedFaceCannotBeCutSmallEnoughToMesh` | sixteen cuts that each narrow both halves | **gap, undemonstrated, no generator reaches it** | measured at one or two pieces over every curved fuse, at deflections from `1e-4` to 3 |
| `CurvedFaceWithAHoleGoesTooFarRound` | a curved face with a hole that is not a ring, going more than a quarter turn round | **gap, from a file only** | the fuse's ring is cut across into four patches and a hole that does not go right round has nothing to cut across to — but such a face has a triangulation and the cutting rule cannot find it. §31.10 item 5 filed the old doc as a claim M4 would falsify, and it did: *"the fuse refuses a curved solid outright"* stopped being true. What still keeps a fuse away from it is the row above — the only fused face with a hole in it is a ball with a pipe through it, and that refuses first |
| `Undecidable`, *which piece of a face a hole is in* | no ray through a lump of endless curves gave an answer | **gap**, unchanged from §17 | |
| `Undecidable`, *which way a crossing runs* | the direction worked out at a box straddles every axis | **gap**, and unreached since §36.8 | |

**Not refusals about arithmetic.** `Invalid::FacesDoNotAgreeWhichWayTheyFace` and
`Invalid::VolumeIsNotPositive` out of the fuse are this crate saying it cannot
assemble what it cut, before writing anything — `two_balls_that_really_cross_are_refused_by_name`
is the second. `Invalid::AClosedEdgeIsWalkedAgainstItsSurface`,
`Invalid::AFaceStraddlesSomethingTrimmingIt`, `AFaceMeetsItsNeighbourNowhere` and
`AnEdgeWithoutTwoFacesAlongItCannotBeWalked` are models whose topology and
geometry disagree, which §16.11 says arrive from a file.
`ResultIsNotManifold { count: 1 }` is **withdrawn** (§38.6).

**Six rows are named, classified and undemonstrated**, marked above. §30.5 had
one such row and called it the one worth reading twice; six is a different
quantity of the same thing, and §39.7 hands it on. Each is reachable in principle
from `Deserialize` and each needs a hand-built broken model to reach, which is
the reason none has one.

### 39.5 The numbers

| | at §30 (M3's close) | at §38 | now |
|---|---|---|---|
| tests (`cargo nextest`, whole workspace) | 985 | 1106 | **1111** |
| suite duration | 18.0 s | 2.9 s | **2.9 s** |
| doc tests | 19 | 19 | **19** |
| `FORMAT_VERSION` | 2 | 3 | **3** |
| fuzz corpus | 562 | 815 | **815** |
| `rcad-predicates` files changed | — | 0 | **0** |
| `rcad-mesh` files changed | — | 0 | **0** |

**The suite went from 18.0 s to 2.9 s and that is not a speedup.** §34.8 set
`[profile.test] opt-level = 2` because one draw of the band family marched a
nearly-tangent quartic loop in 18 365 points and was 90 of the suite's 99
seconds unoptimised. `debug-assertions` and `overflow-checks` are separate
settings and stay on, so every `debug_assert!` still fires and no test verifies
less than it did. Nothing in the kernel got faster.

**Files changed over the milestone**, whole tree: `rcad-geom` 20 files,
`rcad-topo` 24, `rcad` 2, `rcad-mesh` 0, `rcad-predicates` 0.

**Family by family, never blended (§17.7). Naming a crossing**, 2049 pairs each:

| Family | `Undecidable` | no half-space | runs through itself |
|---|---|---|---|
| control, whole numbers | 0.00% | 0.00% | 0.00% |
| quartic control, whole numbers | 0.00% | 0.00% | 4.25% |
| the band, a few bits off tangent | 0.00% | 3.76% | 1.07% |
| written at `2^k`, moved by `2^m` | 10.49% | 3.95% | 0.00% |
| leaning | 0.00% | 0.00% | 0.00% |
| **headline, turned** | 0.00% | **0.10%** | 0.00% |
| **quartic headline, turned** | 0.00% | **2.24%** | 0.00% |

The last two rows are criterion 3's, and they read 0.44% and 10.9% at §30.7.

**Where three surfaces meet**, 2050 triples each:

| Family | corners | certified `Zero` on all three | refused | widest box |
|---|---|---|---|---|
| control, lattice | 571 | 315 | **0.00%** | 2.2e-16 of the size |
| leaning | 1281 | 0 | **0.00%** | 1.8e-14 |
| written at `2^k` | 646 | 240 | **10.83%** | 3.0e-16 |
| turned | 682 | 0 | **0.00%** | 2.4e-15 |

**A curved fuse**, 384 fuses a family, all three rules and both ways round:

| Family | refused | recorded anything |
|---|---|---|
| lattice, a bar through a block | **0.00%** | 0 |
| turned about an awkward axis | **0.00%** | 0 |
| a ball resting on a block | **0.00%** | 0 |
| written at `2^k` | **0.00%** | 0 |
| **a block turned *against* a bar**, new at this gate | 7.8% | — |

That last row is §39.3's family and it is the only one here that turns one solid
relative to the other. 354 of its 384 fuses answer, every answer passes
`Model::validate`, and the thirty that do not are 24 `Undecidable` on *a point
inside a piece of a face* and six that are not solids.

**A planar fuse**, 423 fuses a family, unchanged from §38.7 and re-measured here:
lattice 0.0% refused and 4.7% not a solid; sharing coordinates 0.0% and 17.0%;
elsewhere 0.0% and 4.0%; **turned 22.7% and 4.0%**; skew 12.1% and 1.9%; leaning
alike 5.7% and 4.0%; a bar through a block 0.0% and 0.0%.

**`just fuzz 900`, run because this gate touches `rcad-geom`**: **24 486 343
executions in 901 seconds** at about 27 200 a second, coverage 2508 → **2515**
edges, and **no defect at all**. The seven new edges are the two branches this
gate added and the arithmetic behind them. Twenty-two working inputs were kept
and none is checked in: they are unminimised, and §30.7's rule is that an
unminimised handful is not worth the corpus. The shipped corpus stays at 815.

`just default` is green, `cargo-deny` included.

### 39.6 The turned family's threshold, and where its headroom went

§38.10 handed this on as *"the number most likely to go red next for a reason
that is nobody's regression"*: `TURNED_REFUSALS_MUST_STAY_BELOW` is 24%, the
measurement is 22.70%, and 1.3 points is six fuses. §38.8 put the shrinkage down
to the corpus growing — *"it read 20.71% of 420 when it was written and reads
22.70% of 423 now, and the corpus grew because the generator did"*.

**Measured, that is not what happened.** At `67a6b7f`, M3's close, the same test
on the same seed gives **423 fuses and 87 refusals, 20.6%**. The corpus was
already 423 and the 420 in the prose was stale from before M3. So the two points
were spent *inside* M4, and they were spent at one commit: `6a00ade`, the
expansion slide, leaves it at 87, and `c1e713e` takes it to 96.

`c1e713e` is the commit that made `Axis::new` reduce before it normalises, and
**every configuration in this family is turned about an `Axis`**. It changed the
rotation each pair is built with. So this is a **different corpus** and not the
same corpus refusing more — which is why nothing got worse anywhere else: the
family's own *order along a line* refusals went to zero over the milestone and
its *not a solid* count fell from 24 to 17.

**Decision: the threshold is not re-derived, and the risk is not handed on
vaguely either.** Three reasons, in the order they matter.

1. **Twenty-four per cent is doing its job and doing it sharply.** It fires on
   anything 1.3 points above the measurement. The smallest undeserved refusal on
   record was worth 3.5 points and §19.1's was worth 4.4, so the threshold has
   three times the sensitivity it needs. Widening it to buy drift room would
   trade a real property for a false comfort.
2. **A soak cannot re-derive it.** The threshold is judged on the shipped corpus
   only, deterministically, and a soak is a different corpus with a distribution
   of its own — six fresh seeds at ten times the corpus run 23.4% to 24.7%, four
   of them above this number with nothing wrong. Setting a shipped-corpus
   threshold from soak numbers is exactly the blending §17.7 exists to forbid.
   The soak the gate asked for would produce a number that must not be used.
3. **The thing that was actually wrong was the accounting, and it is fixed.**
   §38.8 could not say where the two points went because nobody had measured the
   ancestor. Now the doc comment names the commit. **When this test goes red, the
   answer is to attribute the move the way this section did and re-derive from
   the new corpus — never to raise the number.**

**The one threshold that was re-derived is criterion 16's**, and it went the
other way. `how_often_a_certified_fuse_of_a_turned_curved_pair_refuses` judged a
measurement of 0.00% against a ceiling of three fifths. That ceiling was written
at §36.14, when the ball family beside it refused every fuse; §36.21 took that
family to none and left the ceiling where it was. Sixty points of headroom over a
deterministic zero catches nothing. It is asserted at zero now, and the doc says
what that costs: the headline of this file now says exactly what its control
says, because **no draw of this generator reaches a shape that refuses**. The
shapes that refuse are all hand-written in `degenerate_curved_fuses.rs`. Widening
the generator until it reaches one is what would make a *rate* over this family
mean anything, and it is the day the number gets re-derived again.

### 39.7 What M4 hands to M5

§30.8's shape: numbered, each with its evidence and its named consumer, and §13
rule 3 applies — a part with no named consumer is not built. Roadmap §12 says M5
is **STEP import**, which changes what "named consumer" means for half of this
list: an importer's whole job is to turn a stranger's bytes into a `Model`.

1. **Three panics on a caller's bytes, demonstrated.** §39.3's list: `validate`
   on a closed edge between two flat faces, `mesh` and `measure` on a closed edge
   with two different ends, `mesh` on a wire fanned onto the wrong surface.
   **Consumer: M5, immediately and unavoidably.** §16.11 says a caller's bytes
   may not panic, and a STEP file is a caller's bytes. This is the first item
   because it is the only one M5 cannot route around.
2. **The rule that generalises them.** A `# Panics` clause on a `pub` item needs
   a demonstration that the arenas cannot hold the shape, not an argument that
   geometry forbids it. `Model` is `Deserialize`. Four milestones running, the
   claim audit's main catch has been this species; M5 should sweep every
   `# Panics` in the tree against it once, rather than meeting them one at a
   time.
3. **`Exact::dot_of` does not slide and `Exact::square_minus` does.** §39.3's
   fourth item, with the input. Closing it is a change to `Exact` and therefore
   to every predicate that reaches level three, so it is a lane with its own
   re-measurement of every rate in §39.5. **Consumer: any caller wanting a
   direction with a subnormal component decided** — which a STEP file with an
   awkward unit scale can produce.
4. **Two authorities on "is this plane along this pipe's axis".** `intersect`
   decides it exactly; `along_a_crossing.rs` and `walk.rs` decide it with
   `== 0.0`. Demonstrated on `(1, 0, 3)` against `(−12, −12, 4)`. It moves
   `walk`'s dispatch and therefore the mesher, so it needs its own lane and its
   own re-measurement. **Consumer: any imported solid whose axis and cut plane
   are not both lattice-friendly**, which is most of them.
5. **`meet_of_three_planes` hands back `Ok` holding an unbounded box**, and
   refuses three mutually perpendicular planes written at `1e-200`. It is
   `rcad-predicates`', the crate has been frozen for ten lanes, and unfreezing it
   reopens the gate (§31.5) — so **this is the first item on M5's opening gate's
   desk**, not a lane's. The `-inf` case is a **limit** and correctly refused
   upstream; the scaled case is a **gap** whose fix is dividing each plane's four
   coefficients by a power of two before the determinants, which is exact and
   leaves Cramer's ratio unchanged.
6. **Six refusals are named, classified and undemonstrated** (§39.4). Each needs
   a hand-built broken model, which is what M5 will be building anyway.
7. **A ball with a pipe through it is still refused**, and it is criterion 13's
   second shape. The quartic ordering gap, §36.20 and §34.10, and it is not a
   budget question: tolerant refuses it too. **Consumer: any imported solid with
   a hole through a sphere**, and there is no shortage.
8. **Two coincident curved faces are refused**, which is criterion 15's second
   name and roadmap §7.4's first entry. **Consumer: STEP files are full of
   coincident faces** — this is the item most likely to be the first thing an
   importer hits, because duplicated and coincident geometry is what §9.2's
   healing list is about.
9. **A corner exactly on a curved face is refused**, and §38.2 wants a predicate
   that says whether one branch of `S ∩ Q₁` is the same curve as one branch of
   `S ∩ Q₂`. **Consumer: an ordinary part reaches it by accident**, which is
   §37.8's own ranking.
10. **A ball cut by a plane through its polar axis is refused**, and §38.3 wants
    the other half of what §38.1 built: a cut closed face written out with a seam
    and poles of its own. **Consumer: the same importer**, and the road is half
    built.
11. **A turned notched bar, and a bar cut by a single plane off its axis.** Both
    §36.28's, both refused under `Policy::tolerant` too, and §38.9 says the
    second is entangled with item 8. **Consumer: the first turned part anybody
    imports.**
12. **Solving three curved surfaces, and a crossing that runs through itself.**
    Unchanged from §31.9 and still deferred with their consumers named. Both are
    gaps whose family of fixes is a milestone of algebra.
13. **Cone and torus have a consumer for the first time.** §24.6's argument —
    a new `Surface` variant touches `validate`, the mesher, `is_wholly_flat`, the
    fuse and serialization — was true through M4 and stays true, but roadmap
    §12's own line for M5 is *"a STEP file that holds one"*. This is the
    milestone where that stops being hypothetical.
14. **The BVH, the bump arena and `rkyv` stay deferred**, consumers unchanged and
    no figure asking for any of them. §30.8 item 2's interval divide at 4% is
    still not where the money is, and M4 had no performance criterion.
15. **`Undecidable` out of `mesh` is still unreachable**, handed on for the third
    time. §39.2 settles that it needs a `Model` whose vertices are boxes, which
    is a change to what a model *is*. **It should stop being handed on as a
    milestone item and be recorded as a design consequence**, which this sentence
    does.

**What a STEP file walks straight into, said plainly.** Items 1, 6 and 2 are the
import boundary itself and are not optional. Items 8, 9 and 11 are geometry that
real files contain in quantity: coincident faces, a corner landing exactly on a
cylinder, a part that is not axis-aligned. Item 13 is a surface kind the format
has and the kernel does not. **A STEP importer built on this kernel today would
load a wide class of files and refuse a wide class of ordinary parts**, and the
refusals would be honest and named, which is the thing this kernel was built to
do — but they would be frequent.

### 39.8 Corrections to earlier sections

- **§34.10's fourth row** classifies `meet_of_three_planes` handing back an
  infinite box as a **gap**. It is a **limit**: the meet is at about `1e315` and
  no `f64` triple names it. §39.4.
- **§38.8** attributes the turned family's 2.1 points to the corpus growing from
  420 to 423. The corpus was already 423 at M3's close and the rate was 20.6%;
  the points were spent at `c1e713e`. §39.6.
- **§36.23's paragraph on `the_surface_that_names_it_twice`** says a corner this
  fuse solved for is exactly on a plane it was solved from. It is not, and §39.3
  is the demonstration.
- **§36.16's row for `ACurvedFaceCannotBeCutSmallEnoughToMesh`** says reaching it
  means a piece is being cut without getting any smaller. `cut_in_two` hands back
  `None` for exactly that case and the piece never comes round again; reaching it
  needs sixteen cuts that each *do* narrow both halves.
- **§35.11's row for `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`** records the
  curved cause only. There is a flat one, and §39.4 has it.
- **§31.7 criterion 1** cannot be met by a `Plane` and §32.3 proves it. What
  meets it is `HalfSpace`, which did not exist when the criterion was written.
- **§31.7 criterion 16's threshold** was never derived from its measurement.
  §39.6.
- **§30.8 item 11 is wrong and §31.8 is right**, and it is settled rather than
  handed on again. §39.2.

### 39.9 Is this a kernel

Roadmap §12 says of M4: *"This is the point at which you have a kernel."*

**Not yet, and the gap is narrower than the refusal table makes it look.**

What is true: a caller can declare or compute a solid with curved faces, fuse it
with another under any of three rules in any order, get history maps back, have
the answer validated, meshed to a stated deflection and measured against its
closed form to a part in ten thousand, and write it out and read it back. Every
decision in that chain is a certified sign or a named refusal, the metamorphic
properties hold over curved pairs, and the whole of roadmap §7.4's catalogue is a
table with nineteen rows and no unanswered questions in it — nine answered, nine
refused by name, one that nothing in the public API can build. That is more than
"a boolean that works on the shapes it was tested on", which is what most of this
gets called.

What is not true is the sentence a user would need. **Nine of the nineteen
catalogue rows refuse**, and they are not exotic: two coincident faces, a corner
landing on a cylinder, a ball with a hole through it, a bar cut by one plane that
misses its axis. A kernel is a thing you can build a part on without knowing
which arrangements it dislikes, and this one still has arrangements it dislikes
that an ordinary part reaches by accident — §37.8 said so of one of them and it
is still true of four.

And the audit is the sharper answer. **This gate found `Model::fuse` returning
`Ok` with a solid that fails its own sweep**, on a block turned against a bar,
which is not a degenerate arrangement but the ordinary case of two parts that are
not aligned with each other. It survived ten lanes, a green suite of 1106 tests
and a code review, because no test in the milestone had ever turned one solid
relative to a curved one. A kernel is not a thing whose worst defect is found by
the last person to look.

So: **the architecture is a kernel's and the coverage is not.** Everything
roadmap §12 lists for M4 is built and measured. What M4 shows is that the list
was the wrong measure — the fuse working over curved faces was three lanes, and
the four milestones' worth of degeneracy that a real part contains is what M5 and
M6 are actually for. Roadmap §12's sentence should read *"this is the point at
which the kernel's shape stops changing"*, which is true, and which is worth
more than what it says.
