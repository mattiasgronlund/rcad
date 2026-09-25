## 242. `rcad_testing`, a harness that runs OpenCASCADE's test corpus, and its seven asks — 2026-09-25

*A new named consumer arrived with seven things it cannot ask rcad. This says
who it is, which asks §13 now lets through, how each was triaged, what the first
batch landed, and why the first dispatch of that batch landed nothing at all.*

  242.1   Who is asking
  242.2   What §13 lets through now, and what it still does not
  242.3   The seven, triaged
  242.4   What the consumer has wrong, to tell it
  242.5   The first dispatch stopped at the budget, before any code
  242.6   What landed
  242.7   What is next

### 242.1 Who is asking

`rcad_testing` runs OpenCASCADE's DRAW test corpus against rcad's facade: 17 879
cases, 4 055 run so far, 133 of them saying anything about rcad. It is anonymous
and not on this machine. It counts "the kernel said it could not" apart from "the
kernel got it wrong", reports counts per functional area, never names a failing
case in its default report, and holds a rotating share of the corpus out of every
report.

Its demand text is an artifact Mattias owns, *Seven things rcad cannot be asked*
(<https://claude.ai/code/artifact/57a3cdca-8131-4889-a7f4-df24f52512d0>). As
`docs/consumers.md` says, it is pointed at and not copied. The case counts below
are the consumer's and were not measured here.

Two things about how it asks shape every answer below. **It asks for
capabilities, not signatures**, so the names are rcad's to pick. And **a refusal
is an acceptable answer to every one of the seven**. What it cannot count is a
question with no way to ask it. So an ask is met when there is a way to ask it
and an honest answer comes back, and a refusal is an honest answer.

### 242.2 What §13 lets through now, and what it still does not

§13 rule 3 says a part with no named consumer is not built. `rcad_testing` is
the named consumer for R1 to R6, and each has a case count behind it and a use
that fits in one sentence.

R7, faces on B-spline surfaces, is a different kind of ask. §24.1 set free-form
surfaces aside because nothing needed them, and that reason is now gone twice
over. This consumer needs them, and so does STEP import: its
`ASurfaceThisKernelDoesNotHold` (`crates/rcad-step/src/refusal.rs:47`) is "the
commonest refusal this reader makes" (§42.11). But a consumer arriving makes a
milestone *allowed*. It does not make it *next*, and §0 says a new unit is
Mattias's call. So R7 was asked, not taken, and he answered that it is not next
(§242.7).

### 242.3 The seven, triaged

| | ask | cases | what happens |
|---|---|---|---|
| R1 | the length of an edge or a wire, with an error bound | 1 473 | **batch 1**: straight edges and circles; everything else refuses by a name that says it is a gap |
| R2 | whether *one* solid is valid | 2 767 | **batch 1**: `Model::validate_solid` |
| R3 | the measure of a part, and a part as an operand | 1 007 | **batch 2**, landed for a face's area and centroid (§242.6); **declined for now** for a face or shell as an operand |
| R4 | an independent copy of a solid | 560 | **batch 1**: `Model::copy_solid` |
| R5 | a general fuse: keep every piece, and where each came from | 1 039 | **batch 2, as a design lane first** |
| R6 | the crossing itself, as a shape | 513 | **deferred** until R1 and R3 land |
| R7 | faces on B-spline surfaces | 144 | **not next**, by Mattias's answer; the next surface kind is picked by a STEP corpus count |

Here are the reasons for every row that is not simply "build it".

**R3's operand half waits for R6.** Operands are solids, and `validate` refuses
a face or shell that nothing owns (`check_everything_has_an_owner`, model.rs).
So a face cannot be held on its own today. Allowing that is the change R6 needs
too: somewhere in the model for a shape that is not a solid. It should be made
once, for both.

**R5 is closer than it looks, and still needs design first.** The assembler
already makes several solids, and `Fused::solids()` returns a slice. Three
things are missing. `choose` applies one `Rule`, where a general fuse sorts every
piece into the cell of the operands it lies inside. A face belongs to exactly one
shell (`no_owner_claims_the_same_entity_twice`), so the wall between two cells
needs one copy per cell, and the history has to say the two copies are one wall.
And `Fate::KeepIfItComesFirst` (fuse/select.rs) keeps one face where two pieces
coincide, which is right for one answer and wrong for two neighbouring cells.
The history also maps each input face to what it became. It does not map an
output solid to the operands it came from, so that has to be added. For two
operands the harness can do it today with three fuses: A − B, B − A and A ∩ B.

**R6 needs a place for edges that belong to no face.** The model cannot hold an
edge or a wire outside a face (the same orphan rule). There are two possible
shapes. One is a `Section` value that lives outside the model, with views and a
length, which is cheaper. The other is free wires or a compound in the model
(plan 5.1). What the corpus checks of a section is its total length and how many
parts it has, so neither shape is worth building before R1 gives it a length and
R3 gives it parts.

**R7 costs a milestone.** §42.4 counts what a new `Surface` variant costs:
evaluation, a sign predicate, a distance, a crossing with every other kind, a
mesher chart, and a compile break at every match. `Surface::Torus`'s match arms
are a fair stand-in, and there are about 190 of them: geom 99, topo 63, step 19,
mesh 6, facade 3. `BSplineSurface` and `BSplineCurve` exist in `rcad-geom` and
the facade re-exports them, but nothing in topo, mesh or step uses either. An
edge cannot come first either: an edge is named by the surfaces either side of
it (§24.3), so STEP's `ACurveThisKernelDoesNotHold` says a B-spline edge has
nothing to be named by until a B-spline is a `Surface`. `docs/plan.md` item 6.3
already records the free-form path as deferred on §24.1. Nothing here changes
that state: a consumer arriving is not work done. §42.11's table is partly stale
on this point, since cone and torus have landed since, and B-spline, revolution
and extrusion are the three still refused. R7
has the fewest cases of the seven behind it, and STEP import is the larger
consumer. So the recommendation is to pick the next surface kind by which one
blocks the most STEP corpus files, and that need not be B-spline.

**What the consumer says it is not asking for stands as written.** That is cone
and torus solids through `SolidBuilder`, the `bop*` protocol, a tolerance stored
on a shape, and `checkfreebounds`. rcad has no stored tolerance by design,
because strictness is an argument. All four are harness work.

### 242.4 What the consumer has wrong, to tell it

- **`EdgeView::run` is `pub`** (`crates/rcad-topo/src/view.rs:1357`), not
  `pub(crate)`, and `EdgeRun` is in the facade (`crates/rcad/src/lib.rs:347`).
  So a copy through the views never had to re-derive what an edge runs along.
  R4 makes that route unnecessary anyway.
- **"Not the length of the edge" is on `EdgeView::distance_between_its_ends`**
  (`view.rs:1320-1330`), not on `segment`. The point it was making, that
  nothing gave a length, is right.
- **"Same key, both move" is right**, and it is why R4 was needed.
  `Model::place_solid` composes the movement onto each shell's placement, so
  every name bound to that key moves.

### 242.5 The first dispatch stopped at the budget, before any code

R1, R2 and R4 went out as three Opus lanes, run `wf_07325529-8ad` on base
`c9244ac9`. **All three stopped at the 150k lane warning while they were still
reading, and none of them wrote a line.** They were warned at 153 431, 153 512
and 157 145 input tokens, per their own reports. The whole run took 319 seconds
and 61 tool calls, as the workflow's own usage line reports.

The warning was true. These are three different figures, where §235's defect
gave several lanes one shared figure, so this is §235's fix working. **The
briefs were what was wrong.** §235.4 says a lane expected to run long is told so
in its brief, and none of these three was. A lane that writes code in
`validate.rs` (4 548 lines) or `model.rs` (1 990) is a long lane by
construction: reading enough to change them safely comes to more than 150k.
The script's own "Budget ~45 minutes" line says nothing about context, so it
could not have caught this.

Each lane did return a design, and that is what saved the batch. The re-run,
`wf_1abdffde-f37` on `05193b51`, gated green here with 2 402 passed, carries
three things:

- each lane's design, so the new lane starts at the code;
- the sentence that the lane runs long, with an upper stop at about 400k;
- line ranges to read in place of whole files.

It also carries one decision the validate lane had left open, recorded in §242.6.

### 242.6 What landed

**R2: `Model::validate_solid(solid, policy, log) -> Result<()>`.** It runs the
checks `validate` runs, over the solid and what it reaches, and nothing else.
It is not a second validator. Every check in `validate.rs` now takes a
`Scope`, which is one map per arena. `validate` passes a scope holding every
slot, and a scope walks in slot order just as the arena does, so `validate`'s
answer and the order it finds things in are unchanged. `validate_solid` passes
what the solid reaches. `check_keys_resolve` is asked of the scope too, in the
same order. The walk steps over a dangling key rather than panicking on it,
and the key check then refuses it by name.

**Decided by the dispatcher: every model-wide number is taken from the solid.**
That means the rounding room (`largest_coordinate_in`, used by `check_edges` and
`check_faces`) as well as the dynamic range. So `validate_solid(m, s)` answers
what `validate` answers for a model holding `s` alone. The reason is that this
is how rcad already judges a solid when it is made. `SolidBuilder::build`
(builder.rs:336-340) validates a scratch model holding only the new solid, and
`Assembly::commit` (fuse/assemble.rs:344-347) does the same for a fuse's answer.
The other reading would let a far-away neighbour widen a small solid's room,
and that is a false accept. Sharing is still counted over the whole model: a
coedge, wire, face or shell the solid reaches that another owner also names is
`UsedMoreThanOnce`. Orphans are not asked about, because nothing reached
through its owner can be one.

**The cost, stated in the doc and demonstrated: neither answer implies the
other.**
- `a_corner_the_far_solid_makes_room_for_is_refused_on_its_own`: a far solid
  widens the room `validate` gives, so `validate` passes a corner that
  `validate_solid` refuses.
- `two_solids_each_fine_on_its_own_can_be_too_far_apart_together`: a tiny step
  beside a far block is too wide a range as one model, while each solid passes
  alone. This is the consumer's complaint made concrete, and it is why
  `validate_solid` is the one that answers `checkshape`.

The lane broke its own work four ways, one at a time, to show the tests catch
it: room taken model-wide, range taken model-wide, `validate_solid` sweeping
everything, and the walk indexing where it should ask. Each failed the test
aimed at it. Gate on the lane tip `1853fe3c`: 2 409 passed, 12 skipped.

One cost is not measured. A scope is seven `SecondaryMap`s, and a
`SecondaryMap` is as long as the highest slot put in it. So asking about one
small solid in a large model can still cost a pass as long as the arena. A DRAW
script's model is small, so this waits for a measurement (§13.3).

**R4: `Model::copy_solid(key) -> Result<SolidKey>`.** It is a deep copy, because
a shell belongs to one solid, a face to one shell and an edge to two coedges,
and a shared corner would be tagged in both solids by one setter. Every level is
gathered before anything is written, then written in the original's slot order
through the insert helpers, with a sorted `Renaming` in place of a map as long
as the arena (§4.3).

- **Slot order is what makes the copy measure bit for bit.** With no empty slot,
  new entities sort the way their originals do, so every walk visits the copy in
  the original's order. This was measured over nine shapes: cube, ball, round
  bar, frustum, pointed cone, quarter bend, two cubes united, a cube with a void,
  and a turned cube. For each one, `Debug` of every view field and of
  `SolidView::measure` is the same text for the unmoved copy as for the
  original. **A model read from a file can have empty slots**, because slotmap's
  serde keeps them and fills the highest first. The copy is then still whole,
  but no bit-for-bit promise is made for it.
  `a_copy_written_into_empty_slots_does_not_sort_the_way_its_original_does`
  shows this.
- **One refusal beyond the design.** A file can hold two solids whose skins
  close only together, such as a cube's faces split three and three. Copying one
  half would write edges walked once. So `copy_solid` asks first whether every
  edge of the solid is walked only from inside it, and refuses with
  `Invalid::EdgeCoedgeCount`. The lane turned the check off and the test then
  panicked in `check_touched_solid`.
- **A fuse never writes its inputs.** The lane checked every stage from
  `Model::fuse` down to `worst_tag_of_the_inputs`, and the only `&mut Model` is
  `Assembly::commit`, which inserts and tags only what it wrote. So a copy stays
  independent across a fuse as well as a move.

Gate on the lane tip `05b33bcf`: 2 414 passed, 12 skipped.

**R1: `EdgeView::length` and `WireView::length`, returning `Length`.**
`Length` carries a length and an error, the way `Measurement` does. It reaches
straight edges, whole circles where a plane is square to a pipe or cuts a ball,
and arcs of those circles, with the arc picked by the pickers and a picker that
cannot decide being a refusal. Every step is a `rcad_predicates::Interval`, and
the error is half the width of the interval the answer lands in, so it is not a
hand count of roundings. `atan2` is the one input that is allowed for rather
than proved: `ROOM_FOR_ATAN2` is 8 ε.

The lane measured the worst error over length against textbook values worked to
about 100 bits, and published and pinned a bound above each:

| edge | worst measured | published |
|---|---|---|
| straight, along an axis | exactly 0 | exactly 0 |
| straight, off an axis | 1.24 ε | 4 ε |
| a whole circle on a pipe | 1.59 ε | 2 ε |
| half a circle round a ball | 3.98 ε | 6 ε |

The lane measured 4.77 ε for the ball before a great circle's radius was taken
as the ball's own rather than squared and rooted back, and that is why that
special case exists. Its verifier reproduced the other three figures to every
digit, but not this one. An arc's error is a fixed number of radians, so a short arc's error
is larger as a share of its length. Only half circles are pinned as a ratio;
quarter and three-quarter arcs are checked for containment. Lengths are measured
in the frame the geometry is stored in, and a prism, a bar and a ball placed a
million units away and turned measure the same to the bit.

Every other curve refuses with **`CannotYetMeasureHowLongThisCurveIs { edge }`**.
It is a gap, and its doc names what would reach each family: Archimedes'
bracket over a walk for anything a plane cuts from a quadric, the
arithmetic–geometric mean for a whole ellipse, and an inscribed polygon plus a
bend bound for a degree-four crossing. An edge or wire too long for an `f64` is
**`LengthTooLargeToMeasure`**, a limit. The facade exports `Length` and
`ROOM_FOR_ATAN2`. The planar conics were not reached and are a queue row.

One edge case is left to `validate`: a plane that misses a ball gives a radius
of zero, so an edge along a circle that is not there measures zero. That is how
every view treats a model nobody validated, and `Deserialize` does not validate
(`serialize.rs`'s module doc).

The lane's own gate hit the 600-second tool limit on a cold release build, so
its verifier ran the whole gate on the lane tip `6c83a2ab`: 2 412 passed, 12
skipped.

**R3's measuring half: `SolidView::measure_one_face(face, deflection, policy,
log) -> Result<FaceMeasurement>`**, with an area, an `area_error` in the unit
squared, a centroid and a `centroid_wobble` as a length. It hangs off the solid
rather than `FaceView`, for three reasons. A `FaceView` does not know its solid.
A curved face cannot be meshed alone, because a cap reads the circle points its
wall walked (§25.7). And §3.2 already puts questions that need a face's
neighbours on the solid, as `face_holds_the_point` does. A face the solid does
not use is `NoSuchFaceInThisSolid`.

- **It takes the solid's own path, with the solid's own charges**, so no second
  bound was written. A wholly flat solid measures the face's fan; any other
  solid is meshed, and the face's triangles are picked out of the mesh.
- **The faces add up to the solid bit for bit.** Their areas, added from 0.0 in
  `faces()` order, equal `Measurement::surface_area` to the bit. That was
  checked on a cube, on five curved solids both at the origin and turned and a
  million units out, and on a fused union. The errors add up to the solid's
  within 8 ε of it, not to the bit, because the solid adds its flat rims before
  it multiplies and a face multiplies its own.
- **Every bound held against the textbook**: 96 curved faces at four
  deflections. The closest an area came to its error was a quarter of it. The
  centroid wobble is sound but loose: the closest a centroid came was 0.0022 of
  its wobble, about 460 times inside. Tightening it needs a bound on how
  lopsided a face's triangles can be, and nothing in the kernel has one.
- Measuring every face of a curved solid meshes the solid once per face. No
  call that measures them all at once was built, because nobody has asked for
  one (§13).
- **One refusal is argued rather than shown.** A face that comes out with no
  area returns what `Model::validate` refuses for that model, and panics if
  `validate` accepts it. The doc argues that `validate` refuses every such
  model. Showing it needs bytes built through `model.data`, which is private to
  `model.rs`, so the test is a queue row, Q97.

Gate on the lane tip `d7736e6e`: 2 408 passed, 12 skipped.

**What the lanes turned up beside their asks.** None of it came from the new
code, and each item is a queue row now:

- **A quarter bend fused with its copy is refused `Undecidable`**, at "a point
  in a piece that every solid could place", (3, 0, 0). A second bend built from
  the same numbers is refused the same way, so the copy is not the cause.
  `a_quarter_bend_and_its_copy_are_refused_as_two_bends_built_alike_are` pins
  it and moves when the gap closes.
- **`fuse([a], [a], Union)` gives no solids.** `a` has volume 8, and
  Intersection and `fuse([a, a], [], Union)` also give none. That is a wrong
  answer. The R5 design lane measured it. The cause was read in the code and not
  instrumented: `which_solid_owns_each_face` (select.rs:1021-1031) credits each
  face slot to the last solid holding it.
- **A face between two tools facing opposite ways depends on the tools'
  order.** Take s0 = [0,2]³, s1 = [2,4]×[0,2]×[0,2] and s2 = [0,2]×[1,3]×[0,2].
  `fuse([s0], [s1, s2], Difference)` is refused `ResultIsNotManifold { 3 }`,
  while `[s2, s1]` gives volume 4. The intersection is refused
  `ResultIsNotManifold { 1 }`. `settle_against_each` keeps only the first `On`
  it finds (read, not instrumented).
- **A turned pair of cubes touching face to face** gives A − B and B − A, while
  A ∪ B is refused `ResultIsNotManifold { 1 }`. It was not investigated.
- **A flat solid's errors are `0.0` while its sum rounds.** §25.4 says a wholly
  flat solid is "summed exactly" and all three errors are exactly `0.0`.
  `volume_and_moment_over` redoes the sum exactly only when the sum is near zero,
  to settle its sign. Anywhere else it is f64 addition. A turned cube united
  with its copy measures 8.000000000000002 with `volume_error` 0.0. The exact
  volume of the polyhedron the fuse wrote was not computed, so whether that
  published bound excludes its truth is still open. A face shows the same
  thing: a flat side-2 cube turned one radian and moved a million units out has
  a face that `measure_one_face` puts 1.25e-10 off 4, with `area_error` 0.0. The
  measure lane pinned that rather than claim the truth is inside, because the fix
  is the solid's and every face's together. `Length` counts its own arithmetic
  in its error, which is the other way to do it.
- **Deserialize lets in solids that share an edge**, because
  `check_shape_is_possible` asks for two opposite coedges per edge across the
  whole model. `copy_solid` refuses such a solid. Other operations were not
  checked.

### 242.7 What is next

**Batch 2 ran beside batch 1**, as run `wf_1f28011a-1f6` on the same base,
with the two items that could go first: R3's measuring half (§242.6) and a
read-only design lane for R5.

**R5 is designed as `Model::cut_into_cells(&[SolidKey], policy, log) ->
Result<Cells>`, a new method and not a new `Rule`.** It takes any number of
solids in one list, with no split into arguments and tools. A cell is named by
the ascending positions, in that list, of the operands it lies inside. It is
never empty. Two separate lumps under one name are two solids. `Cells` wraps a
`Fused`, so every history accessor carries over, and it adds `cells()`,
`cell(&[usize])`, `inside(solid)` and, in lane G4, `across(face)`. A `Rule`
cannot carry this for three reasons:
- `choose` gives a piece one fate, and a wall uses one piece twice, facing
  opposite ways, in two solids;
- `Fused` has nowhere to say which operands a solid lies inside;
- `row_for` has no row for it.

- **How a piece gets its cell.** The piece is placed against every other
  operand separately, with the per-piece setup computed once. The cell behind it
  is its own operand plus every operand it is inside or on facing the same way.
  The cell in front is every operand it is inside or on facing opposite ways.
- **What replaces `KeepIfItComesFirst`: the lowest position writes.** A piece
  lying on another operand's skin is kept only when no operand with a lower
  position has it on its skin. The kept piece is written twice: into the cell
  behind as it stands, and into the cell in front turned round. Every other
  piece in the stack is dropped as its twin, and history gives the twin's input
  face the kept piece's images, so a flush face whose area survives does not read
  as deleted.
- **Cells share no entity.** Each cell is its own `Assembly`. One assembly over
  every cell would put four to six faces on each cut edge. All the cells are
  committed together: one throwaway model, one sweep, one range check across
  all of them, one write. `across(face)` records which faces of other cells were
  built from the same pieces. It means "shares part of its area", because the
  joiner runs per cell and a face can be partly wall.
- **The oracle that comes free.** For two operands, cells [0] and [0, 1] come
  out of the same scratch, the same placements, the same chosen pieces in the
  same order and the same assembly as Difference and Intersection. So they are
  equal to those fuses bit for bit, by construction. Cell [1] against
  `fuse([B], [A], Difference)` holds bit for bit wherever commutativity already
  does. Volumes: the cells sum to the union. For every set T of operands, the
  chained intersection of T equals the sum over the cells containing T, which
  for three operands is seven equalities. Inclusion–exclusion then follows and
  is not an independent check. At every half-integer lattice point, the operands
  whose `classify_point` says Inside must be the `inside()` of the one cell
  holding that point. That proves the cells do not overlap, which
  `shares_space_with`, being a search, cannot.
- **No new refusal.** A cell whose region is not manifold is refused the way a
  fuse is: a limit, by §17.4. The placement refusals are gaps. With three or
  more operands, each piece is placed n − 1 times with no stop at the first
  Inside, so the general fuse can refuse a case the grouped Boolean answers.
  That rate is measured once lane G3 exists. mkvolume and volumemaker take faces
  as operands, which §242.3 declines, so they are not served.

**It is five lanes in a chain: Q80 → Q86 → Q87, then Q88 beside Q89.** All
five write in `fuse/`, which does not split (§13.6).

- **Q80** fixes the two wrong answers in §242.6's list, because the general
  fuse would inherit both. Its tests are the design lane's probes: `a` fused
  with itself either way equals `a`, and s0 − (s1 ∪ s2) equals s0 − (s2 ∪ s1)
  bit for bit, at volume 4.
- **Q80 goes before Q5, although both write `fuse/select.rs`.** The dispatcher
  who owns Q5 put it that way round, and the reason holds: two wrong answers
  outrank a refusal moving. Q5 had also come back with a red gate.
- **Q86** is the cell choice, in a new `fuse/cells.rs`.
- **Q87** is `commit_all`, the public method and `Cells`, and it writes the
  facade too.
- **Q88** is the property tests, and **Q89** is the wall relation and the twins'
  history.

Every one of them runs long.

**These wait on what batch 1 landed:**

- **R1 on every crossing that lies in a plane.** Each one is a convex conic, so
  an inscribed polygon bounds its length from below and the polygon of tangents
  from above. That is Archimedes' bracket over a walk, and
  `CannotYetMeasureHowLongThisCurveIs` names it.
- **R6, and R3's operand half with it,** once R1 and R3 have given a section a
  length and parts to count.

**R7 was asked of Mattias, and he took the recommendation in §242.3** (in this
session's terminal, 2026-09-25). B-spline faces are not the next unit. The next
surface kind is picked by counting which of the three the STEP reader still
refuses blocks the most corpus files: B-spline, surface of revolution, or
extrusion. That count is a queue row. Plan item 6.3 keeps the state it had, and
R7's row stays parked on this section until the count names a kind.

Every open item here, and every finding in §242.6's list, is a row of
`docs/queue.md` citing this section, landed in the same commit.
