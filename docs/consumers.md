# Consumers

`docs/decisions.md` §13 says a part with no named consumer is not built. That
rule only works if the consumers are written down, and until now the only
external one existed as oral history in another repository. This file is the
list, and it is short on purpose.

It records **facts rcad owns**: what depends on rcad, what rcad has agreed to
build for it, and what rcad asks in return. It does **not** restate the
consumer's own reasoning — where a consumer keeps that text, this file points at
it, because the side that changes more often should own it and two copies drift.

### One exception to *a fact rcad measures gets no second answer downstream*

The rule is about **numbers**. A bound or a rate copied downstream is a second
thing to keep right, and a copy restated in the copier's own words can be wrong
and internally consistent, which is the case you never find out about.

A downstream assertion that carries **no number** is not that. rscene asserts
that rcad's five surfaces do not all cross their two coordinates the same way —
never which surface is which — so a change to any one chart's winding moves
rcad's table test and leaves rscene's untouched, and the two cannot drift apart
in the way this rule guards against.

What makes it worth keeping rather than merely harmless is **what its failure
would mean**, and it is a category worth having a name for:

> **A test whose failure is news that you can delete something.** If rcad's five
> charts ever became uniform, every normal rscene draws would still be correct
> and rscene would simply be carrying gradient machinery it no longer needs. Its
> assertion failing does not say *rscene is broken*; it says *rscene is
> over-engineered*.

A design that survives only because of a fact about somebody else's code should
hear when that fact changes, and nothing in either tree would otherwise say it.
Both sides keeping a test is right here for the same reason it is usually wrong:
these two answer different questions, and only one of them has a number in it.

## rscene

A scene layer over rcad: takes a `Model`, draws it, and answers *what did I
click* with a topological entity. Lives at `~/git/mattiasgronlund/rscene`.

It depends on the `rcad` facade only, at a pinned git rev, and concentrates every
mention of rcad in one module (`kernel.rs`) with a test enforcing that. Its
design doc is its own `docs/roadmap.md`; **§8 there owns the demand text** — the
signatures, the code that needs them, and what rscene does if refused. Read that
rather than trusting a summary here.

**Pinned at `22a163a`** — `main` as it stood when rscene took the pin, which is
this file's own last commit. Not the first sha carrying the two demands below;
those landed earlier, and rscene had no reason to reach back past them. Pinning
needed no ceremony, as arranged: rscene pinned it, and rcad does not bump it.

### What rcad built for it

Both are built and merged, and both are §13-justified by rscene being the named
consumer. If rscene stops needing either, rcad wants to hear — see the
withdrawal notice below.

- **`Model::which_model_this_is() -> WhichModelThisIs`** — an opaque
  `Copy + Eq + Hash` mark, one `u64` on `Model`, present in all builds, for a
  downstream cache-safety assert. Four properties: distinct per `Model`,
  **stable across mutation**, **fresh on clone**, **fresh on load**.

  rscene asked for `Model::id()`. That name was taken by a `pub(crate) fn id()`
  with 41 call sites across ten files, and freeing it was the wrong trade —
  worse, two methods called `id` on one type, one per-key and one per-model,
  with **opposite** behaviour under `Clone`, would restate the trap rather than
  avoid it. `identity` was rejected for the same reason at one remove: `Id` is
  the abbreviation of *identity*, and `Isometry::identity()` already means "no
  movement" here. The cost is 8 bytes per model, 376 → 384 in release.

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

  Three things the getters say out loud that were private knowledge before, and
  a caller meets all three. **The points run the way the edge is stored**, not
  the way any coedge uses it — a caller holding a coedge flips on
  `coedge.edge().orientation().is_reversed()`. **The two end vertices are not in
  the list**; they are `point_at_vertex` of the edge's two vertices, and a closed
  edge's single corner sits at both ends. **A flat face contributes nothing**, so
  a cube's `along_edge` is entirely empty, and absent and empty are deliberately
  the same answer — `points_along_edge` returns `&[]` for a seam that walked to
  nothing, for a never-walked edge, and for an edge of another solid, because all
  three mean *nothing to draw between the ends*. `point_at_vertex` keeps its
  `Option`, where `None` means something different: not a corner of this solid.

A third demand — a display-edge polyline API — was **withdrawn by rscene** on its
own inspection, because display edges should be built from mesh points and a more
accurate kernel curve would be worse. It is recorded because the withdrawal is
the behaviour this arrangement depends on.

### What rscene now leans on, said before it leaned on it

The rule two sections down — *say so before you depend on a claim* — as it
applies to the pin above. None of this is a request. It is a list of sentences
that have just become expensive for rcad to be wrong about, published so that
somebody with a reason to be adversarial reads them.

Each is now a test in rscene's `crates/rscene-core/src/kernel.rs`, in
`mod facts_this_design_rests_on`, so a pin bump that falsifies one fails
rscene's build rather than rotting quietly in its prose. rscene's own
`docs/roadmap.md` §3 and §7 carry the reasoning; this is the list.

- **A cube's `points_along_edge` is empty for all twelve edges.** A flat face
  contributes nothing, so rscene's edge path meets the empty case on the
  commonest solid there is.
- **`points_along_edge` of another solid's edge is `&[]`, not an error**, and
  `point_at_vertex` of another solid's vertex is `None`. rscene depends on the
  asymmetry: the slice cannot tell a foreign edge from a straight one, which is
  what makes `Model::which_model_this_is()` a guard rather than a nicety.
- **A closed edge's two ends are one vertex**, so an assembled polyline starts
  and ends at the same point and rscene closes it explicitly.
- **A full circle is cut into at least four segments however coarse the
  deflection.** Measured at rscene's end: exactly four at a deflection a hundred
  times the bar's diameter, against twenty-three at the deflection a viewport
  asks for. rscene asserts the segment count rather than the point count,
  because four segments is what the quarter-turn cap promises.
- **`Mesh::triangles()` hands back faces in the solid's stored order**, so each
  face's triangles arrive together. rscene lays its shaded buffer out one
  contiguous span a face — that is how a per-face colour override becomes a draw
  range rather than a face id on every vertex — and it now takes that
  contiguity from this sentence rather than manufacturing it. It had a stable
  sort for it, which was the identity permutation on every mesh it ever saw;
  code that cannot be told from its absence went, and the promise it was
  guarding against is asserted instead. Checked on a fused solid as well as on
  primitives, because a boolean is where a face list is built rather than
  declared.
- **The model identity is distinct per model, stable across mutation and fresh
  on clone.** The fourth property, fresh after a serde round trip, is *not*
  asserted at rscene's end: rcad measures it, rscene never deserializes a
  `Model`, and two answers that can disagree is worse than one.

**One correction, which is rscene's own and not rcad's.** rscene's §4.5 used to
say that `Surface::coordinates_of` yields the true normal along with `(u, v)`.
It does not, and rcad never said it did — only `Plane` has a `normal()`. This is
not a demand: every surface publishes the parameters its normal follows from, so
rscene differentiates the implicit form itself.

Worth one sentence back, because it is a fact about rcad that a consumer nearly
got wrong: **which way `∂u × ∂v` of `point_at` faces is not the same on all five
surfaces.** That was rcad's to check and rcad had not checked it, so the table
that stood here is gone and the measurement is in
`crates/rcad-geom/tests/where_a_surfaces_own_two_coordinates_land.rs` instead.
`every_surface_but_a_torus_crosses_its_two_coordinates_out_of_the_material` is
the table, surface by surface, taken against `SideOfSurface::side_of` and
`Plane::side_of_point` rather than against the parameterization itself, at
ninety-four samples clear of a ball's poles and a cone's apex.
`the_five_surfaces_do_not_all_cross_their_two_coordinates_the_same_way` asserts
only that they disagree — which is the one bit rscene's design rests on, and the
one whose failure is news rather than a regression. Read the table there. A copy
of it here could be wrong and internally consistent, and then it would pass —
this file points at facts for the reason its opening gives, and does not keep
second copies of them.

What is rcad's to own and to say here is the **ordering** that makes one surface
the odd one out, because that is a decision and not a measurement. A torus's two
coordinates run tube-first so that a bend of pipe charts like a straight bar: one
coordinate that wraps, one that runs cap to cap, one seam. `Torus::point_at`
argues it in full under *Round the tube comes first, and that is not arbitrary*.
The hand falls out of the ordering, and the ordering is worth more than the hand.

rscene takes gradients instead of the parameterization's handedness, and checks
them the same way — a step along a normal must land outside and a step back
inside — which caught an inverted torus on the first sample. That is what
prompted the two tests above.

### Withdrawal notice: rscene's §8.3 exception for the unlogged walk

`59f3e15` met the condition rscene's §8.3 had written against this site — *when
it records like its siblings, this exception is deleted* — so it is deleted.
rscene's retry ladder is the only exception left on that list. §8.3 owns the
text; this is the notice §13 asks for, and the withdrawal is unconditional.

Four things rcad may want from it:

- **Nothing should be deleted on rscene's account.** `ROOM_LEFT_TO_REFINE_A_WALK`
  has a named consumer as of now, and decision 44's report entry — an empty
  point list where a polyline was expected — is explicitly *not* withdrawn. It
  was written to outlive this fix and it does.
- **rscene keys on the record's name, not its outcome variant.** The
  `Undecidable`-with-no-`Approximation` filing is noted as rcad's open gap and
  is not a demand; rscene simply does not depend on the half that is wrong.
- **rscene keeps the starved-closed-edge branch**, unreachable or not, because
  §52.11 is open. If that row closes as *no wire can hold a closed edge
  alongside other coedges*, rscene would like to hear: that is the point at
  which deleting the branch becomes right, and rscene will not delete it on the
  strength of "no route rcad's suite can build".
- **The wording will reach bug reports.** rscene's §4.8 report keeps an
  unrecognised refusal's own debug text verbatim, so
  `AWireDoesNotFanIntoALoopToMesh { TooFewCorners { count: 1 } }` will appear in
  reports whose cause was a starved walk. That is already recorded as rcad's; it
  is repeated here only as evidence of how far the wording travels.

### The cone, and what rscene did with it

rscene's §3 carried the pointed cone as *the* measurement that killed **coarser
is likelier to succeed**. That sentence is withdrawn from §3 and replaced with
the honest position — one family swept, no other, nothing known either way —
rather than with the opposite claim.

**The retry ladder did not move.** When the original reason failed, §4.8 was
re-argued from *cost* rather than patched, so the evidence being withdrawn a
second time changes nothing about the design. rcad may find that worth knowing
before spending effort on whether the general claim holds: rscene has no
decision resting on it in either direction.

### How a demand is raised, decided and landed

- A demand arrives as three things: the exact signature proposed, the consumer
  code that cannot be written without it, and what the consumer does if refused.
  It is a request, not a specification.
- **rcad decides on rcad's own terms.** Being the named consumer is an argument,
  not a trump card, and *"same capability, different shape"* is a normal answer —
  `Model::which_model_this_is()` above is one, asked for as `Model::id()`.
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
- **Say so before you depend on a claim, and this one runs both ways.** Before
  pinning a claim about the other side — in a document, in an assertion, or in a
  design that cannot survive its being false — name it to whoever owns it. It
  asks for no work and it is not a demand: the owner may say nothing. It works
  because a claim about to become load-bearing acquires a reader with a reason to
  be adversarial about it, which is most of a closing gate at none of the cost
  (`docs/decisions.md` §51.10). rcad owes it too — before rcad depends on
  something a consumer publishes, the same sentence is available.

  It has already paid twice, both times catching rcad's own error rather than a
  consumer's: *at least four steps* restated as *at least four points*, where the
  emit line excludes both ends and the floor is three; and a starved point list
  blamed on a coarse deflection when the trigger is a tolerant policy, which no
  choice of deflection can dodge.

- **A bound the consumer republishes and deliberately does not pin gets its move
  announced before it lands.** `Mesh::deviation` is the one rcad number rscene
  restates to its own readers as a statement about how wrong the picture may be,
  and rscene keeps **no test asserting its value** — deliberately, by this file's
  opening rule, because a fact rcad measures gets no second answer downstream. It
  is not the exception recorded there either: that one is for a downstream
  assertion carrying **no number**, and this is nothing but a number.
  The cost of that choice is that a change here fails nothing downstream and
  quietly changes what somebody else's documentation means, which is the worse of
  the two failure modes and the one nothing in either tree would otherwise catch.

  So this is a control and not a courtesy: **before a change to what
  `Mesh::deviation` computes reaches `main`, rcad says the old number, the new
  number, and which way it moved.** Downwards counts. A bound that got better still
  leaves a published sentence overstating how wrong the picture may be, and that
  sentence stopped being true the same as any other.

  **No threshold, and the asymmetry is the reason.** The announcement fires on any
  change to what the bound computes, not on one large enough for the consumer to
  act on. A consumer that only wants to hear about numbers big enough to matter
  trains the producer to filter, and **the producer's filter is invisible to the
  consumer by construction** — the consumer cannot see what it was not told, so it
  never finds out the threshold exists. **A silence cannot be audited**, and that is
  the whole argument. rscene argued this after being told about
  an understatement of `2.4e-5` that changed nothing it runs, and it is right: the
  useful half of that message was not the size but that the term is charged against
  corners a face can outrun, which is what will matter when somebody meshes
  something stranger than a bar.

  **A citation is not a second answer — but it does go stale.** Where a consumer
  wants rcad's figure in its own documentation, citing rcad's measurement is the
  right shape and does not breach the no-second-answer rule: a citation cannot
  drift the way a paraphrase can, because there is still only one measurement. What
  it can do is stop describing anything, the moment rcad fixes what it measures. So
  a fix here carries a **second** obligation beyond announcing that the number
  moved: any figure a consumer is citing has to be retired, and rcad says which,
  at the same time. A stale citation is worse than a paraphrase, because its
  provenance is exactly what makes a reader trust it.

  The half that is not rcad's: a raised floor arrives downstream as solids that
  meshed before and now refuse, which is rscene's retry ladder climbing rungs, and
  rscene's §4.8 owns telling a climb across many solids at once from a climb on
  one. It is named here only because it is what the announcement above is *for* —
  without it the first symptom of an rcad fix reads downstream as a difficult
  model, and the afternoon goes on the wrong end.

  Recorded at both ends on purpose. An arrangement written down on one side only
  is indistinguishable from nobody having thought about it, and the next reader of
  either function has no way to tell which it was.

  **And the condition for ending it, promised in advance rather than remembered.**
  If rscene stops republishing the number — the report grows its own bound, or the
  doc stops making the claim — rcad hears, by the withdrawal rule two bullets up.
  At that point the announcement is machinery with no named consumer and §13 would
  rather delete it than carry it. It is written here beside the arrangement rather
  than filed under withdrawals, because an arrangement whose ending condition lives
  somewhere else is the same failure as one recorded at a single end.

### Announcement: `Mesh::deviation` moved on a ball, and the floor under it moved 79×

The bullet above, discharged for the first time. Two numbers a consumer reads
change on the branch that makes the mesher pick a deflection `rcad_geom::walk`
will accept, and this says what each was, what it is, and which way it went. No
threshold was applied to either, because the bullet above says there is none.

**Measured against the two trees, not read off a report.** `c9ff754` and the
branch tip were extracted side by side, one probe binary built against each, and
both run. It is worth saying that way round: the lane that made the change
reported `Mesh::deviation` **bit-identical**, and that was true of the one shape
it measured — a bar — and false on a ball, where the deviation follows the
deflection almost exactly, so a move in the floor goes straight into the
published number. A figure nobody re-ran is how the wrong half of that gets
announced.

**One. `Mesh::deviation` on a ball, upwards.** `add_ball(radius, translation(1e9,
0, 0))`, `Deflection::of(1e-300)`, `Policy::tolerant(1e12)`:

| ball | old (`c9ff754`) | new | which way |
|---|---|---|---|
| r = 0.25 | `7.993599754617626e-6` | `7.993623476198684e-6` | **up** `2.3721581e-11`, which is `2.97e-6` of itself |
| r = 0.5 | `7.99360180936651e-6` | `7.99360180936651e-6` | unchanged, bit for bit |
| r = 1 | `7.993605375215217e-6` | `7.993700080924048e-6` | **up** `9.4705709e-11`, which is `1.18e-5` of itself |

Upwards is the direction that turns a citation into an understatement of how
wrong the picture may be, which is the worse of the two ways for it to go stale.

The point counts move where the number does not, and not all the same way:
872 666 → 872 658 at r = 0.25, 3 640 342 → 3 640 278 at r = 1, and 2 235 566 →
2 235 596 at r = 0.5, whose deviation does not move at all. A bar of radius one
and height two, and balls of radius `0.05` and `0.1`, are identical in both
trees — same deviation, same point count. That is the measurement the lane made,
and it was right about it.

**Where the number cannot have moved, run rather than argued.** A deflection at
or above the floor takes an early return this branch does not touch, so nothing
there should change — and the way to say so is a run. Two radii × four
deflections from `1e-2` to `1e-5` × certified and tolerant × at the origin and at
`x = 1e9`: twenty-four cells, all identical in both trees. So the move is
confined to a deflection asked for **under** the floor and let through by a
tolerant policy, which is exactly the case where `Mesh::deviation` is the only
bound there is, because the deflection asked for bounds nothing.

**Two. The floor itself, up seventy-nine times at the origin.**
`Error::DeflectionBelowWhatAMeshCanReach` carries `closest` — *the closest a mesh
of this solid can be* — and it now reports how fine a mesh can be **cut** where
it used to report how close a point can be **placed**. Asked `1e-300` at the
origin:

| solid | old `closest` | new `closest` | which way |
|---|---|---|---|
| bar r = 1, h = 2 | `9.2148511043888e-13` | `7.273230840146953e-11` | **up**, ×78.93 |
| unit ball | `6.143234069592533e-13` | `4.848820560097969e-11` | **up**, ×78.93 |
| four bars whose cap leans `4e9` to `1e11` | `6.143234069592533e-13` | `4.848820560097969e-11` | **up**, ×78.93, and one answer for all four |

**Far from the origin it barely moves**, which is the other half of the same
measurement: at `x = 1e9` the placing term is eighty thousand times the added one
and `closest` shifts in the eleventh decimal — a ball of radius one goes
`3.996803498977168e-6` → `3.9968513728593615e-6`. The seventy-nine is what a
solid at the origin sees, not a factor to carry about.

It is arguably the better number now — the old one was a floor the mesher then
did not work to. That is not the test this bullet sets. It is a number a consumer
reads, it moved, and this is where that gets said rather than nowhere.

**A refusal also changed its name where the floor bites.** At the origin, asked
`1e-300` under `Policy::tolerant(100.0)` and `Policy::tolerant(1000.0)`, both the
bar and the unit ball used to come back
`TooManyPointsAlongOneCrossing { most: 1048576 }` and now come back
`DeflectionBelowWhatAMeshCanReach`. Said here because the withdrawal notice above
already establishes how far rcad's refusal wording travels: it reaches bug
reports verbatim.

**What rcad's own figures have stopped describing.** The second obligation in the
bullet above, at the same time as the move. This is a list of rcad measurements
that no longer describe what they described — **not** a list of what anybody
should retire. Which of these is cited anywhere is not rcad's to know, and this
file's opening rule is why rcad does not keep that list.

- `9.2148511043888e-13` for a bar of radius one, and `6.143234069592533e-13` for
  a unit ball, **as what the refusal carries** at the origin. Both still describe
  where a point can be *placed*; neither is `closest` any more.
- Any `Mesh::deviation` measured on a curved solid cut **under** the floor before
  this branch. Three ball radii are tabled above; a radius rcad did not measure
  is in the same position and rcad cannot say which way it went without running
  it.
- Any account of what a tolerant caller is refused with at the origin under a
  deflection below the floor, which was `TooManyPointsAlongOneCrossing` and is
  not now.

Nothing else is withdrawn. Every figure at or above the floor stands, the bar and
the two small balls stand, and the far-from-the-origin floor stands to the digits
anyone was likely to quote.

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
