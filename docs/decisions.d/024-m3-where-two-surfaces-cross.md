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

