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
