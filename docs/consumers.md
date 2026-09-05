# Consumers

`docs/decisions.md` §13 says a part with no named consumer is not built. That
rule only works if the consumers are written down, and until now the only
external one existed as oral history in another repository. This file is the
list, and it is short on purpose.

It records **facts rcad owns**: what depends on rcad, what rcad has agreed to
build for it, and what rcad asks in return. It does **not** restate the
consumer's own reasoning — where a consumer keeps that text, this file points at
it, because the side that changes more often should own it and two copies drift.

## rscene

A scene layer over rcad: takes a `Model`, draws it, and answers *what did I
click* with a topological entity. Lives at `~/git/mattiasgronlund/rscene`.

It depends on the `rcad` facade only, at a pinned git rev, and concentrates every
mention of rcad in one module (`kernel.rs`) with a test enforcing that. Its
design doc is its own `docs/roadmap.md`; **§8 there owns the demand text** — the
signatures, the code that needs them, and what rscene does if refused. Read that
rather than trusting a summary here.

**No rev is pinned yet**, and pinning needs no ceremony: rscene pins a sha once
what it needs is merged here.

### What rcad has agreed to build for it

Neither is built. Both are agreed in shape, and both are §13-justified by rscene
being the named consumer.

- **`Model::id()`** — an opaque `Copy + Eq + Hash` id, one `u64` on `Model`,
  present in all builds, for a downstream cache-safety assert. Four properties:
  distinct per `Model`, **stable across mutation**, **fresh on clone**, **fresh
  on load**.

  It is deliberately **not** the per-key `ModelId` (`crates/rcad-topo/src/key.rs`),
  and the two must not be merged. `ModelId` is zero-sized in release so keys pay
  nothing for it, and it exists to catch a key from one model reaching a live
  slot in another (§1.3). Making it release-present would grow every one of the
  seven key types by eight bytes, in every arena and every map, to buy something
  no caller asked for. The two also need **opposite** behaviour under `Clone`: a
  clone's keys address the same slots and rightly survive, while the model's
  identity as a cache subject rightly does not — so `Model` needs a hand-written
  `Clone` that mints a fresh id. Fresh-on-clone makes a cache built for `A`
  invalid for `A.clone()`, which is a false negative, and that is the direction a
  safety assert should fail in.

  Fresh-on-load is already paid for and needs no attribute: `Model` has a
  hand-written `Serialize` (`serialize.rs:133`), the way back in is
  `Model::from_parts` (`model.rs:411`), and that mints a fresh id. The id is not
  a field of the serialized shape, so putting it on the wire would take a
  deliberate edit rather than the quiet deletion of a `#[serde(skip)]`.

  Stable-across-mutation costs nothing: `Model::fuse` takes `&mut self` and
  returns `Fused`, so a fuse mutates in place and the id survives it by
  construction. Invalidation downstream is driven by `Fused`, not by the id.

- **The mesh-to-topology correspondence**, forward maps only:
  `Mesh::points_along_edge(EdgeKey) -> &[u32]` and
  `Mesh::point_at_vertex(VertexKey) -> Option<u32>`. The mesher already builds
  both (`mesh.rs:360-361`) and `finish` drops them. The reverse index is
  display-side and stays downstream.

  **A shared edge is polylined once, and that was measured before the getter was
  agreed to.** `mesh.rs:406` skips an edge already walked, so an edge between two
  faces gets one point list, built by whichever face reached it first in the
  solid's stored order, and both faces then use it. The worry was that a tightly
  bending face could inherit a slack neighbour's boundary and exceed what
  `Mesh::deviation()` publishes — which would have been an rcad bug, since the
  consumer draws those points and republishes that number as the bound on what is
  on screen.

  **It cannot, and the worry rested on a misreading.** `room = deflection −
  how_far_moving_this_face_rounds(face)` (`mesh.rs:402`) looks like a per-face
  budget; `how_far_moving_this_face_rounds` (`mesh.rs:2046`) is what *placing*
  that face rounds and says nothing about how tightly it bends. There is no
  curvature-adaptive per-face budget in this mesher, so both faces hand
  `rcad_geom::walk` the same number and there is no slack budget to inherit.
  Measured over three lenses between overlapping balls at curvature ratios 20x,
  200x and 100x, meshed at three deflections: the published bound holds in all
  nine, closest approach 50.5% of it, and reversing the stored face order gives
  the same points, the same triangles and the same deviation **bit for bit**.
  `crates/rcad-topo/tests/whether_a_fine_face_is_handed_a_coarse_neighbours_edge.rs`
  is the demonstration.

  Do not read that as a general guarantee. It is safe for sphere against sphere
  because a circle drawn on a ball never has a radius larger than the ball's, so
  the boundary chord stays inside the half-budget reserved for it. Where that
  fails the triangulator **refuses** —
  `CannotTriangulate::BoundarySideStraysFurtherThanTheDeflection`
  (`rcad-mesh/src/triangulate.rs:175-186`) checks every boundary side before
  refining — so the failure mode is loud, not a quiet coarse patch.

  When it ships, the published order is the **edge's** stored direction. Today
  that convention is private knowledge at `mesh.rs:1102-1112`, where the caller
  reads `edge.orientation().is_reversed()` and walks backwards; a public getter
  has to say it out loud.

A third demand — a display-edge polyline API — was **withdrawn by rscene** on its
own inspection, because display edges should be built from mesh points and a more
accurate kernel curve would be worse. It is recorded because the withdrawal is
the behaviour this arrangement depends on.

### How a demand is raised, decided and landed

- A demand arrives as three things: the exact signature proposed, the consumer
  code that cannot be written without it, and what the consumer does if refused.
  It is a request, not a specification.
- **rcad decides on rcad's own terms.** Being the named consumer is an argument,
  not a trump card, and *"same capability, different shape"* is a normal answer —
  `Model::id()` above is one.
- **The pin moves one way.** The consumer bumps its rev when it wants what
  landed. rcad never bumps it.
- **No consumer in rcad's CI gate.** rcad's gate is `just default`. Coupling it
  to a downstream repository means a downstream bug can turn rcad red, and a red
  that is not rcad's trains people to ignore red.
- **Acceptance is rcad-side tests passing**, not the consumer compiling. A gap
  found after the bump is a new demand, not a reopened one.
- **The reciprocal of a demand is a withdrawal notice.** If a consumer stops
  needing a part, rcad wants to hear: §13 would rather delete an unconsumed part
  than carry it, and it cannot do that if the only record of the need is
  somewhere else.

### What rcad asks in return

- **A visual oracle for the mesher, with the `Model` attached.** A triangle that
  looks wrong on screen is worth a bug report carrying the model that produced
  it, not a screenshot. rcad's mesher publishes bounds and has open rows against
  them.
- **A real consumer of key lifetimes across a fuse.** `fuse/history.rs` remaps
  keys and rcad has no downstream that holds keys across an operation and then
  uses them. Picking is that consumer.
- **Never work around a refusal.** `Undecidable` and the named refusals are
  data. Hiding one behind a tolerance or a fallback so the picture looks right
  destroys rcad's best signal that something is wrong, and ships a wrong answer
  where the kernel offered a refusal. If a refusal blocks the consumer, it comes
  back as a demand.

  **One declared exception**, accepted: rscene retries a failed mesh at
  successively coarser deflections and, if the ladder is exhausted, draws a
  warning-coloured box. That is a *different question honestly answered* rather
  than one question fudged — it is bounded, every rung's refusal is reported, the
  deflection actually used is reported for every solid with its provenance, a
  climbed rung is distinguishable from a low deflection asked for deliberately,
  and exhaustion announces itself in a colour. Any further fallback of that kind
  amends the exception list rather than joining it silently.
