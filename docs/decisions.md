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
| Bump arena in signatures | M4 | §1.4 — nothing to measure yet |
| Symbolic perturbation | Never, unless a caller asks | §1.5 — competes with `Undecidable` |
| `rkyv` zero-copy format | M2 | §6 — model still churning |
| `cargo-fuzz` | M2 | §6 — needs a format worth fuzzing |
| `rayon` inside the library | M2 | §7.3 — tessellation is the first real need |
| `ProvenanceSink` streaming | On demand | §5 — no caller yet |
| Branded lifetimes for cross-model keys | Indefinitely | Debug-checked model id is the pragmatic fix |
