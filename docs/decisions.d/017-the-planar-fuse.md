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
| 7. `Policy::max_wobble` has a caller, judged against `ProvenanceLog::worst_wobble` | `Model::fuse` judges it after assembly and before the commit; `a_turned_pair_is_certified_because_a_box_is_not_a_decision_anybody_failed_to_prove` is the answering half, and `a_certified_fuse_over_a_tagged_input_succeeds_and_the_answer_keeps_the_tag` is §17.2's guard against reading the wrong quantity. **The refusing half has no test.** No fuse is known that the judgment refuses and whose answer does not exist. The one refusal by the judgment anybody has found was a gap, closed by Q1467 and pinned answering as `a_tolerant_policy_bigger_than_the_model_fuses_two_turned_blocks_meeting_at_a_corner_as_a_certified_one_does`. The refusing half was `a_fuse_that_needs_a_decision_nothing_can_prove_refuses` until 2026-10-05 and `what_two_turned_cubes_a_rounding_deep_in_each_other_share_is_too_thin_to_hold_and_is_refused` until Q1353, and the two paragraphs after these tables say why each was dropped and what was tried since |
| 8. A measured refusal rate under `Policy::certified()`, recorded in this file | §17.7, asserted by `how_often_a_certified_fuse_of_a_turned_pair_refuses` with `every_certified_fuse_of_a_lattice_pair_is_decided_and_records_nothing` and `every_certified_fuse_of_a_pair_sharing_coordinates_is_decided` as its controls |

| §14.8 oracle | The test that discharges it |
|---|---|
| Commutativity of union and intersection | `swapping_the_two_sides_of_a_lattice_pair_changes_not_one_bit`, and the same for turned pairs and for pairs sharing coordinates, all comparing the answer's faces bit for bit; `uniting_two_ells_flush_against_each_other_gives_the_same_faces_either_way_round` is the case that first failed |
| Idempotence and absorption | `a_solid_united_with_a_copy_of_itself_is_itself` for `A ∪ A` and `A ∩ A`, `a_solid_with_a_copy_of_itself_taken_away_is_nothing_at_all` for `A \ A` over both a lattice solid and a turned one, `a_solid_united_with_what_it_shares_with_another_is_itself` for `A ∪ (A ∩ B)` |
| `fuse([A,B,C]) ≡ fuse([fuse([A,B]), C])` | `uniting_three_at_once_is_uniting_two_and_then_the_third`, with `uniting_the_first_two_and_then_the_third_is_uniting_the_last_two_and_then_the_first` beside it for the order the three arrive in |

**Criterion 7's refusing half, amended 2026-10-05 (Q490).** The table named
`a_fuse_that_needs_a_decision_nothing_can_prove_refuses`, two towers `1e308`
tall, and three times its refusal turned out to be a gap: `InPlane::of` at
`1e200` (§19.5), `orient3d` at `1e308` (§324.6), and `midway_between` at
`1e308` (§353). The towers now unite, as
`two_towers_1e308_tall_are_united_by_proof_alone`. A refusal shows this half
only where the answer does not exist (§15.8, §16.10). So the new one was looked
for among fuses whose answer no `Model` can hold, and three kinds were tried:

- **A corner past the largest double.** No fuse has one. Its answer lies
  inside its inputs, whose corners are doubles, so no coordinate of the answer
  is bigger than the inputs' biggest. The towers are this kind, and they
  answer.
- **A corner between two doubles.** The model holds it as a box. That is
  representation wobble, which is never refused on (§17.2):
  `a_turned_pair_is_certified_because_a_box_is_not_a_decision_anybody_failed_to_prove`
  answers with every corner it solved a box.
- **Two corners of the answer closer than a double can tell apart.** Found.
  The two cubes `two_turned_cubes_a_rounding_apart.rs` turns by `0.3` overlap
  by a slab about `2^-52` thick. The new test finds the slab's eight corners
  and twelve edges from the planes alone, exactly, and shows that however an
  answer places its corners inside their boxes, its largest coordinate over its
  shortest edge is past `7.9e14`. `Model::validate` refuses every such answer
  (§3.5). The certified fuse stops before that, in selection, on the slab's
  face on the second cube's bottom, a strip `2^-52` wide. If selection ever
  places that strip, the range check refuses the answer instead, as
  `Invalid::DynamicRangeTooWide`, and the test fails so this half is looked at
  again. That happened: Q1353 placed the strip, and the test was repinned to
  `DynamicRangeTooWide` instead of failing (§417.3). The next paragraph says
  what that left.

The refusals of `a_block_edge_grazing_a_ball.rs` do not qualify: the answers
of those fuses exist (§346.3).

**Criterion 7's refusing half, amended 2026-10-09 (Q1457): none is known.**
The turned cubes are refused by the range check, and the fuse asks that only
after the judgment has passed. So their refusal says nothing about the
judgment, and this half lost its test a fourth time (§353.5). Q1457 looked
for a new one and found none.

*Where the judgment sits.* `Model::fused_under` asks it after
`split_both_groups`, `choose` and `assemble` have built an answer, and before
`check_dynamic_range` and the commit. In `fuse/mod.rs`, a decision the policy
could not prove goes through `must_decide` or `gave_up`, and both return
`Error::Undecidable` on the spot, before any answer is built. Records can also
stay in the log from a decision nothing was built on:
`classify_uncertain_point` keeps whatever a ray that grazed something left in
the log and tries the next ray, and selection drains that log into the fuse's
once the point is placed. Nobody has checked whether a grazed ray leaves a
record there under a certified policy. So the judgment sees records the policy
accepted one at a time, and may see records like those. Whatever it refuses,
it refuses after an answer was built. That is the order the code runs in. It
is not a claim that this half cannot be met.

*What was tried.*

- **The two towers `1e308` tall.** Refused three times, and each time a gap
  (§19.5, §324.6, §353). They now unite, as
  `two_towers_1e308_tall_are_united_by_proof_alone`.
- **The turned pair.**
  `a_turned_pair_is_certified_because_a_box_is_not_a_decision_anybody_failed_to_prove`
  answers.
- **The two turned cubes a rounding deep in each other.** No model holds their
  answer, and the fuse refuses it as `Invalid::DynamicRangeTooWide`, past the
  judgment (§417.3). That is §3.5's limit, not this half.
- **`a_block_edge_grazing_a_ball.rs`.** Refused, but the answers exist, so
  gaps (§346.3).
- **Two probes, not kept.** Q1457's first relay ran 8190 fuses and saw no
  refusal by the judgment. Its second ran 1920 with the judgment's line
  printing when it refused: two cubes of each side `2`, `2e3`, `2e4`, `2e5` and
  `2e6`, each pair turned eight ways at random about an axis through the
  middle of an edge, the second cube moved by `(S, S/4, S/8)`, `(S/2, S/4,
  S/8)`, `(S, S, S/8)` or `(S, S, S)` for side `S`, under all three rules and
  `Policy::tolerant` at `100`, `1e4`, `1e6` and `1e9`. The judgment refused
  six of them, and all six were one pair: two blocks of side `2000` meeting
  corner to corner, under `tolerant(1e4)` and `tolerant(1e6)`, under every
  rule. Every other fuse answered or was refused earlier, by name, and every
  `Undecidable` among those was refused on *a point inside a piece of a face*.

*Why the six are a gap.* The same pair answers under `Policy::certified()`,
`tolerant(100)` and `tolerant(1e9)`, so its answer exists.
`resolve_under_policy` reads each record as a wobble at the policy's working
size, and the judgment read the same log at the model's
`Scale::nominal_size`, which is `100` here. The pair's two `orient3d` records
are each about `2.1e-4` wide. A volume is divided by the working size squared
(§15.2), so at `100` each comes out `1e4` times what it is at `1e4`, about
`2.1e-8` against a budget of about `1.4e-10`. The policy accepted both, and
the judgment then refused the solid built from them, which is what
`Policy::wobble_is_acceptable`'s doc says its one budget is there to stop.
§15.2 named this seam: the model's nominal size is a second copy of the
policy's working size.

*Closed 2026-10-09 (Q1467).* The judgment reads the log at the policy's
working size now, and at the model's only under a certified policy, which
carries none (§15.2's note of the same date). The pair answers under all five
policies, every rule, both ways round: two blocks for a union, nothing for an
intersection, the first block for a difference, each validated and measured.
Under `tolerant(1e4)` and `tolerant(1e6)` each of those fuses still records
decisions the policy accepted and that read past the budget at the model's
size, so the pin still reaches the judgment. It is
`a_tolerant_policy_bigger_than_the_model_fuses_two_turned_blocks_meeting_at_a_corner_as_a_certified_one_does`.
Q1457's probe was not kept and has not been run again. If each of its fuses
was on a 100 mm model, as the pair was, its other fuses cannot have moved:
every working size it tried was at least the model's, dividing by a bigger
size gives a wobble no bigger, so the judgment passes every log it passed
before, and nothing before the judgment changed.

*What this leaves.* Criterion 7's refusing half has no demonstration. No fuse
is known that the judgment refuses and whose answer does not exist. Whether a
certified fuse can be refused over the record of a ray it threw away is filed
as a row too, and nobody has run it; if one is found, its answer was built, so
it is a gap as well.

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

