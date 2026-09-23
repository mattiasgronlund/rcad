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

**No rev is written down here, on purpose.** One was — `22a163a`, described as
*`main` as it stood when rscene took the pin, which is this file's own last
commit*. That sentence pinned this file to itself, and it stopped being true the
next time anybody edited the file, which was immediately. It is **deleted rather
than corrected**: rscene's own `main` is unpushed, so a rev written down today
would name a commit nobody else can fetch, and a pin that cannot be resolved is
worse than none — it reads as a fact and behaves as a dead end.

The arrangement is unchanged and needs no ceremony: **rscene pins, and rcad never
bumps it.** Which rev rscene is on is rscene's to know and rcad's to be told when
it matters, which is what the announcements below are for.

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

### Withdrawal notice: per-face colour out of STEP import — **reinstated 2026-09-23, see `just decision 207`**

**The withdrawal below is spent.** rscene filed the demand on 2026-09-23, its
owner having overridden the rscene-side reasoning that produced the withdrawal:
its requirement 2.1.4 is to be met rather than reported around. The consumer is
named, so §13 no longer closes this, and the costing is `docs/decisions.md`
§207 — no new parsing, the face identity already exists inside the reader, and
the one structural change is in `rcad-topo` rather than in `rcad-step`.
**Accepted, shape settled with rscene the same day (§207.7), not built**: §0's
unit is elsewhere, a new unit is Mattias's call, and rscene asked for no rush.
Landing it carries a retirement obligation — rscene cites `import.rs:195`'s doc
comment verbatim and that sentence moves (§207.9). The census that ships with
the map is cut by what rcad can do about each case, and measured (§207.8).

The rest of this entry is kept as it stood, because the reciprocal rule below
only works if a withdrawal that gets reversed is visibly reversed rather than
quietly deleted. Read it as history from here down.

`docs/decisions.md` §56.7 recorded this as *undecided, not refused* — a
well-formed demand acknowledged twice in channel and written into no rcad file
until that line. **It is now withdrawn, and withdrawn is not refused.** rcad did
not decide against it. rscene searched its own history for the demand, found
nothing raised, and confirmed it took the refused path long ago: it draws flat
and reports colour unavailable, and it has no decision resting on the colour
arriving. There is no consumer, so `docs/decisions.md` §13 closes it — a part
with no named consumer is not built.

**The four design inputs are kept here so that a later builder does not have to
re-derive them.** They are the whole of what the demand ever specified, and they
cost nothing to carry:

1. `STYLED_ITEM` — what binds a style to a shape.
2. `COLOUR_RGB` — the colour itself.
3. `PRESENTATION_STYLE_ASSIGNMENT` — what gathers styles onto one item.
4. `SURFACE_STYLE_*` — the surface side of that assignment.

The shape it would take is a colour per `FaceKey` on `Import`, beside the solids
rather than inside them. rscene's sink for it is built and tested, so adopting it
later is source-side only and the withdrawal is cheap to reverse — which is
exactly why it is recorded rather than forgotten.

**Only the feature half is withdrawn.** §13 governs features and never
correctness gaps, and there was a correctness gap next to this one: `Import`'s
census read as complete while styling was walked past uncounted, so a consumer
was about to publish a denominator saying the importer had dropped nothing while
it had dropped every colour in the corpus. That is fixed and stays fixed —
`Import::shapes_this_reader_did_not_take` now says out loud that it does not
count styling, a file of styling and no solid gets an `Err` rather than an empty
map, and `styling_is_never_among_the_shapes_walked_past` pins both halves over
the real corpus. Withdrawing the colour does not reopen any of that.

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

**Correction, 2026-09-06: *"the guard holds"* is in circulation and is no longer
true.** The sentence said of rscene's near-apex guard that no defect is
demonstrated and the guard holds, on a worst case of `14.177` against a ball of
`16`. rscene has since measured a plain `translation((3,3,3)) ∘ rotation` at
`18.245` against that same `16`, which is over budget. Their account of the cause
is that the guard spends a budget written for three coordinates on
`max_element`, and `(3,3,3)` hands it the full √3 between the largest coordinate
and the true magnitude.

**This is rscene's bug with an rscene fix, and this file does not claim
otherwise.** Both numbers and the cause are rscene's own; **this side has not
re-taken the measurement and none of it is verified here.** It is written down
for two reasons and no third: the *guard holds* sentence travelled and is now
false wherever it landed, and it bears on when rscene's apex work is safe to lean
on. Nothing in rcad changes on it.

### How a demand is raised, decided and landed

- A demand arrives as three things: the exact signature proposed, the consumer
  code that cannot be written without it, and what the consumer does if refused.
  It is a request, not a specification.
- **rcad decides on rcad's own terms.** Being the named consumer is an argument,
  not a trump card, and *"same capability, different shape"* is a normal answer —
  `Model::which_model_this_is()` above is one, asked for as `Model::id()`.
- **The pin moves one way.** The consumer bumps its rev when it wants what
  landed. rcad never bumps it.
- **No consumer in rcad's CI gate.** rcad's gate is `just gate`. Coupling it
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

- **A second number under the same arrangement: the KiCad import rate.** rscene
  quotes rcad's **`97 of 323`** — how many of the KiCad corpus's files rcad
  imports — and asks to hear when it moves rather than meet the move in a sweep.
  Same shape as the bullet above and the same rules: the old number, the new
  number and which way it went, before the change reaches `main`; no threshold,
  for the reason given above; and if the figure is being cited, rcad says at the
  same time that the citation has stopped describing anything.

  Four things this number has that `Mesh::deviation` does not, which is why it is
  written out rather than folded into the bullet above.

  **It is a ceiling and not a forecast.** It says what one corpus did at one
  commit. It does not say that thirty per cent of the next batch of files will
  import, and quoted about any other corpus it is a number about a different
  question. A consumer sizing a retry ladder or a support burden off it is asking
  it to carry something it cannot.

  **It moves far more often than the deviation does.** Every import fix moves it,
  and a lane is in flight as this is written that may move it, so this figure may
  be stale before it is read. That is an argument for the announcement, not
  against the number.

  **rcad cannot move it quietly, and that was not always true.**
  `how_often_a_real_file_imports` pins the **exact set** of files that import —
  `KICAD_FILES_THAT_IMPORT`, ninety-seven names, pinned since `d23c700` — and
  fails both ways: a pinned file that stops importing, and an unpinned one that
  starts. So a fall is a red test rather than a silence. The caveat is the
  corpus, which lives outside the tree: a run that cannot find it runs this test
  short. `docs/decisions.md` §43.2's criterion 10 — *nothing asserts the
  real-file rate*, and *if 46/323 fell to 1/323 the test would still pass* — was
  true when it was written and is not true now.

  **It is unverified by this lane.** `97 / 323 = 30.0%` is quoted from
  `docs/decisions.md` §45.1, which measured it. What was checked here is that the
  pinned list holds ninety-seven names — the numerator, not the rate. Said
  plainly rather than left to look like a measurement, because a number this file
  lends its confidence to is exactly the kind that gets cited.

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
| unit ball | `3.0716170347962663e-13` | `2.4244102800489844e-11` | **up**, ×78.93 |
| four bars whose cap leans `4e9` to `1e11` | not re-measured — see below | `0.19395282244028492`, `0.484882056046163`, `1.4546461680657568`, `4.848820560134335` | **up**, and **four different answers**, the widest lean's `24.9999999955` times the narrowest's |

**That last row used to carry the unit ball's two numbers and the words *one
answer for all four*, and both halves were wrong.** The four bars give four
floors spanning a factor of twenty-five, pinned to the last digit by
`crates/rcad-topo/tests/what_floor_a_bar_with_a_leaning_cap_is_given.rs`, because
the floor follows the ellipse the cap cuts and the four ellipses are four
different sizes.

`4.848820560097969e-11` is not a stray, though. Divide each of the four floors by
its own lean and the same constant comes back to ten digits —
`4.848820561007123e-11`, `4.8488205604616304e-11`, `4.848820560219189e-11`,
`4.848820560134335e-11`. So it is a floor **per unit of what the shape is
charged**, and the row stated it as though it were a length. The old column is
left unfilled because rcad has not re-measured these four bars on the pre-move
tree; what stood there was the row above's figure, copied.

**The unit that constant is per was itself wrong, and the ball row above is what
moved.** The figure was first written down as *per unit of the shape's reach*,
which read `4.848820560097969e-11` off a unit ball — and a unit ball was being
charged `2` for a face that reaches `1`. Charged from its centre it is now cut to
`2.4244102800489844e-11`, and that is the constant: the bar of radius one and
height two is exactly `3` times it, a leaning bar is `2 · lean` times it, and the
ball is `1` times it, all three exact in `f64`. The bars did not move; only the
ball, and only because what it was charged did. A figure per unit of a quantity
is wrong in two independent ways, and this one had the second: the digits were
right and the unit was not.

**Far from the origin it barely moves**, which is the other half of the same
measurement: at `x = 1e9` the placing term is eighty thousand times the added one
and `closest` shifts in the eleventh decimal — a ball of radius one goes
`3.996803498977168e-6` → `3.996827132753364e-6`. The seventy-nine is what a
solid at the origin sees, not a factor to carry about.

That second figure is the ball charged from its centre, measured and pinned by
`a_unit_ball_a_thousand_million_units_out_is_cut_to_what_the_docs_announce`. It
was announced as `3.9968513728593615e-6` before that change. Note how little it
moved where the pair at the origin halved: out here the placing term is eighty
thousand times the added one, so what the ball is charged reaches only the fifth
figure.

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
  where a point can be *placed*; neither is `closest` any more. The ball's has
  since halved again, to `3.0716170347962663e-13`, for the separate reason in the
  row above — so that one is withdrawn twice over.
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

### Four ways a downstream sentence stops meaning what it meant

The announcement machinery above watches **numbers**: a bound or a rate rcad
publishes moves, and the move gets announced whether or not anyone thinks it
matters. That catches the case it was built for and misses four others. The
first three turned up on one day, 2026-09-06; the fourth was written the next
day and comes at it from the other side. They are **not** the same failure —
which is why they are four names here rather than one category with a
definition stretched far enough to cover all of them:

> **A downstream sentence whose truth rests on an rcad defect existing.** Fixing
> the defect makes it false. No number moves, no line of code changes on either
> side, and nothing in either tree says so.

> **A demonstration that quietly stops demonstrating.** The defect it was built
> to reach is still there, and the construction no longer reaches it. The test
> goes on passing, and passing has stopped being an answer to anything.

> **An ask that gets answered by something no one connected to it.** The thing
> asked for is already there. It landed for a reason that had nothing to do with
> the ask, nobody joined the two, and the way you find out is by going to send
> the demand.

> **A sweep whose sample set could only ever return one of the two answers.** The
> run completes, the samples are real, every assertion passes — and the finding
> was settled before the first sample was drawn, because the sample set could not
> have produced the other one.

What holds the four together is one sentence, and it is worth having in front of
the four:

> The artefact stayed internally consistent while the world it described moved,
> or was never fully sampled — and review checks consistency.

The first three are the *moved* half. The fourth is the *never fully sampled*
half, and it is the one that explains why a reading pass catches none of them.

**The first** is the one this section was written for, and its worked instance
is §56's apex fix. rscene's near-apex report tells callers, in a public doc
comment, that a face appearing in it says something about the kernel — that the
apex a placed cone's surface names and the one its mesher emits arrived by two
routes. That was true, and it was rcad's defect that made it true. After the fix
a placed cone face cannot separate the two spellings at all, so the sentence is
not merely stale: the same report now means the opposite thing, that rscene's
own tolerance swallowed a good point. A caller acting on the old wording files a
bug against rcad for something rcad did not do. rscene owns that text and is
rewriting it; the wording is theirs and is not restated here.

**What rcad owes is the trigger, not the rewrite.** The side that fixes a defect
is the only side that knows the moment it stopped existing, and a consumer cannot
watch for it — there is nothing to watch. So a fix that closes a defect a
consumer has *described* is an announcement here, in the same way a moved number
is, and the bullet the announcements sit under should be read as covering both.

The reason the first is worth a category rather than a note: it is the failure
mode where both trees are green, both documents are internally consistent, and
the disagreement lives only in what a sentence means. `rcad` cannot find it by
running anything.

**The second was found by nothing, and that is the difference that matters.**
rscene had a test that reached a live defect up here: a cone placed so that its
apex reads exactly zero, where the nearest mesh point then sat outside a ball of
zero width. **The ball is zero wide because there is nothing to size it with.** A
cone has no normal at its apex, so a consumer shading one has to find the apex
among the mesh points in order to know not to ask, and its only handle is exact
equality against `Cone::apex()` with a small ball behind it for the case where
rcad's arithmetic moved the point. The one length available to size that ball is
the apex's own coordinates, so it comes out as some multiple of `ε · |apex|` —
and an apex on the origin makes it exactly zero, for every multiple, while rcad's
floor under the same solid does not go to zero at all. Measured in this tree at
`b3a21da` under a certified policy with the apex reading `[0, 0, 0]`: a cone
reaching `1` is floored at `3.6366154200734765e-11` and one reaching `1e9` at
`3.636615420073477e-2`. Nine decades of floor against a backstop that stays
exactly zero, so the backstop vanishes precisely where it was asked to work.

§56's fix moved the mesh point onto the apex, which closed **the route from that
defect to a wrong answer and not the defect** — it is still there and still one
builder call away. That is what makes this the second arm and not the first.
rscene's account of what its own test used to see is rscene's; this side has not
re-taken those figures and nothing in rcad changes on them.

The first arm is caught by somebody reading the sentence, and somebody did read
the near-apex doc comment. This one has no sentence to read. The test file says
what it always said, the run is green, and it would have stayed green for ever
while proving nothing. **A test that has lost its subject looks exactly like a
test that still has it.**

There is no rcad-side trigger available for it, and saying so is worth more than
proposing one. The bullet above fires when rcad **closes** a defect a consumer
has described; §56 closed no defect here, only the route to it, so nothing was
owed and an announcement would not have helped. What rcad would need in order to
see it is which of a consumer's tests reaches which of rcad's defects, and that
is a fact about rscene's tree that rcad does not hold and has no way to acquire.
The control that does exist is downstream and rcad has already argued for it in
a neighbouring case: `docs/decisions.md` §56.7 says an absence that could not
have been a presence proves nothing, and asks a sweep to carry a positive
control. **A
demonstration wants the same thing** — an assertion that it still reaches the
thing it demonstrates — and it wants it for exactly the reason this instance
shows, that the day it stops reaching is the day nothing says so.

**The third was found by going to paste the text.** rscene wrote a demand asking
rcad to re-export a provenance name past `rcad-geom`, so that a consumer reading
its own log has the name rather than a copy of the string. The demand was never
sent. At `3a69d79` — the pin rscene took for an unrelated reason — `rcad`'s
facade already re-exported `ROOM_LEFT_TO_REFINE_A_WALK`, and the doc comment over
that list already gives the reason the demand was about to give: *a caller
reading its own log needs the name, not a copy of the string*. Checked here
rather than taken on trust: the re-export is at `crates/rcad/src/lib.rs:369` at
`b3a21da`, its doc comment ends on line 368 with that sentence, and the same name
is in the same list at `3a69d79`.

The wasted demand cost a paste, and that is not the expensive half. **A demand is
how §13 learns that a part has a consumer**, and a demand that never gets sent
because it was already granted leaves the part looking unconsumed — the
withdrawal notice above records that same constant as having a named consumer,
and nobody knew it had a second one waiting. Granting an ask in advance is not
the same as answering it, because answering it would have left a record.

**Watching numbers fails twice over as well, and this is a separate count from
the four arms.** The leaning-bars row above is the sharper half of the
contrast, because it is a case the machinery *could* see and missed anyway:
`4.848820560097969e-11` was present, announced, and correct to the digit — and
wrong in its **dimension**, a floor per unit of the shape's reach written as
though it were a length. So watching numbers fails twice over. It misses the
sentence with no number in it, which is the first arm above, and it passes a
number that is right about its digits and wrong about what it is a number *of*.
A figure is not checked by being announced.

**And that figure was itself dated, which is the third way past — and the date
earned itself within the hour.** The reading held as of `a8d4467`; the change to
how a ball is charged landed immediately after, and the constant is now
`2.4244102800489844e-11` per unit of what a shape is charged. The ball's reach
was being read as `2` where the shape reaches `1`, so the figure absorbed the
factor. Nothing about the two failures above changes; the number naming them
did.

A published figure offered as a fact about the tree wants the sha it was
measured on, in the same way §15.6 makes a published bound want a test — and a
sentence in a commit message wants it most of all, because that is the one
artefact nobody can go back and amend. `251c27a` is the standing example: it
publishes `4.848820560097969e-11` as a floor per unit of reach, undated, and it
is now wrong by a factor of two in a place that cannot be edited. §59.2 is the
sibling case, and the difference between them is the useful part — that one was
never right, this one was right and was moved.

**The fourth was found by somebody sampling the axis a sweep had held still**,
and it is the arm where nothing was wrong except the sample set.
`crates/rcad-geom/tests/what_the_dynamic_range_check_is_a_margin_against.rs`
asks how small a wedge's feature can get against its coordinates before the apex
rounds onto the base, and it swept the coordinate at `1`, `2^10`, `2^20` and
`2^40`. Checked here at `b4e698f`: every one of those four has a mantissa of
exactly zero. A feature of half an ulp puts the apex exactly halfway between two
doubles, a zero mantissa is the even side of every such tie, and so all four
coordinates round the same way. One cliff was the only shape of answer the
sample set could return. It returned one, and it looked complete returning it.

The truth is not a cliff. Another lane sampled every representable placement
across a whole binade and found the law is `radius / ulp(centre)`: above a half
a solid meshes wherever it stands, below a half it refuses wherever it stands,
and at exactly a half round-half-to-even decides on the last bit of the centre's
mantissa — so the answer alternates from one representable placement to the
next. Their census, 32 meshed and 32 refused over 64 consecutive placements, is
**their** measurement and lives in
`crates/rcad-topo/tests/how_far_out_a_ball_can_sit_and_still_mesh.rs`. What was
re-run here is that file: 13 tests, 13 passed at `b4e698f`.

**Nothing the sweep asserted was false and the code under it was not buggy**,
which is worth saying plainly because it is what makes the arm hard to see. Both
measurements are right. The sweep's own verdict — that the spread the kernel
publishes as supported sits far below where a written feature dies — is
untouched; re-measured here at `b4e698f` the feature survives to a spread of
`2^52` at worst and `2^53` at best, and one extra binade of head-room on half
the coordinates does not move that verdict at all. The failure is that the run
*looked complete*.

**What makes this a category and not carelessness is rcad-51's sentence, and the
wording is theirs.** The `COORDINATES` doc comment reasoned explicitly about its
sampling and still missed the axis, **because powers of two are simultaneously
the natural thing to sweep and the one family with no mantissa to be odd**. The
obvious choice and the blind choice were the same choice. A more thoughtful
comment was not the missing ingredient — the thoughtful comment had been
written, and was read, and did not help.

**The rule to apply to a sweep before writing it**: a sweep is blind exactly
when its sample set has fewer degrees of freedom than the answer does. So name
what the answer can turn on, and for each of those name two samples in the set
that differ on it and agree elsewhere. Anything with no such pair is a
*constant* of the run rather than a variable, and whatever the run reports is a
statement about that constant. The trap is that the family reached for by
convention is usually the family with the fewest degrees of freedom — powers of
two being the canonical case, since they are what anyone would pick to span a
range and they are constant in the one bit this answer turned on. The cheap
standing form is a **balance assertion**: make the sweep insist that both
answers actually occurred, so a sample set that goes one-sided again goes red
instead of quiet. `docs/decisions.md` §62 has the full argument and what the
rule costs.

**Each of the four was found by acting on the sentence, and not one by reviewing
it.** The stale doc comment turned up when somebody read it in order to use it;
the demonstration turned up when somebody went to reach the defect it names; the
granted ask turned up when somebody went to paste the demand and send it; the
blind sweep turned up when somebody went and sampled, one representable
placement at a time, the axis it had held still. That is an argument against
review as the safeguard for this family, and it is not a comfortable thing to
write down in a repository that keeps a `claim-audit` skill. **That skill would
have caught none of the four, and the fourth is the proof.** A pass of that kind
reads a sentence and asks whether it is consistent and whether it names
something real. All four were both — and the fourth had already had exactly that
pass, in writing, in its own doc comment, and came out of it blind. The
comfortable conclusion — *review harder next time* — is the one that would have
changed nothing, so it is not the conclusion recorded here. **Send the demand.
Run the demonstration against the thing it demonstrates. Use the sentence. Make
the sweep prove it saw both answers.** What is never acted on is never checked,
whoever reads it.

**Two of the first three are rscene's, and so is the third arm's name.** rscene
found the demonstration and the granted ask in the last hour of its own session,
and *an ask that gets answered by something no one connected to it* is rscene's
wording, kept because it is better than the one this side had. Written
2026-09-07 against `b3a21da`, which is the tree the `crates/rcad/src/lib.rs`
lines above were read on. rscene's own state as it reported it at that handoff —
`main` at `79083c6` on `origin`, green at 172 tests and 5 goldens, pinned at
`3a69d79` — is rscene's report and is **not** verified here, by the dating rule
three paragraphs up, which this section is subject to like anything else.

**The fourth arm is rcad-51's finding and rcad-51's framing**, and the sentence
about the obvious choice being the blind choice is quoted from them rather than
rewritten. It was added 2026-09-07 against `b4e698f`, which is the tree the
mantissas and the two test runs above were measured on; the binade census it
rests on is another lane's and is cited as theirs. The one-sentence statement
holding all four together is rcad's, at rscene's request — rscene is carrying
the same sentence in its own record.

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
