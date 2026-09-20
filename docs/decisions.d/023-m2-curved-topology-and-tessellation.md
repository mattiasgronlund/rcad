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

