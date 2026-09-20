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

**1e11 is a margin, and until 2026-09-07 nothing said so or said what it is a
margin against.** This paragraph named the number and deferred to the roadmap for
why, and the roadmap is a plan rather than a description of this codebase (§1),
so the number was published and unheld — §15.6 with the test missing.
`what_the_dynamic_range_check_is_a_margin_against.rs` now measures it. A feature
written straight into the coordinates is gone somewhere between a spread of
`2^53` and `2^54`, around `1e16`; a constructed point, which is the first thing
to actually fail, stops being decidable at `2^49`. Both are far above 1e11, so
nothing goes wrong at the number itself.

**Which of `2^53` and `2^54` is one bit of the coordinate**, and the first
version of that file got this wrong in a way worth recording. At `2^53` the
feature is exactly half an ulp, so `c + feature` is a tie and rounds to even: a
coordinate with its last mantissa bit clear swallows the feature, one with it set
keeps it a binade longer. The file swept `1`, `2^10`, `2^20` and `2^40` — all
powers of two, all mantissa zero, **all four on the even side of the tie** — so
it could not reach the answer that survives. It reported a single cliff and
looked complete doing it. That is a sampling choice making one of the two
outcomes unreachable, which is a different failure from a wrong measurement or a
wrong claim, and powers of two are both the natural thing to sweep and the one
family with no mantissa to be odd. Found by `rcad [a57470]`'s lane reading the
file rather than by the file failing.

What the margin buys is **conditioning, which a ratio cannot see** — §16.11's
blindness with a quantity on it. A construction of conditioning `t` divides the
feature by `t` inside its own intermediates, so the spread that decides the
answer is `spread / t`, and the feature dies once that reaches `2^52`, exactly,
at every spread measured. At 1e11 that covers constructions conditioned down to
about `2.2e-5` and no further.

The margin is wanted rather than merely tidy because **past the cliff the kernel
does not refuse.** The offset rounds away, the points really are coplanar, and
the predicate answers about them correctly and about a model that is not the one
asked for. A refusal would be safe; a confident wrong sign is not.

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

