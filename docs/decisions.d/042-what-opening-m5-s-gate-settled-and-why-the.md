## 42. What opening M5's gate settled, and why the reader is the small half, recorded 2026-09-02

*Written by M5's opening gate. §27 did this for M3 and §31 for M4. This one is
late — §41's fan of five lanes ran before this section existed — and that is
§42.1's subject rather than a footnote at the end.*

**Every load-bearing sentence below was checked against the tree at `460bf15`,
and four of them were checked by running something rather than by reading.**
§41 established that a gate which copies a report has audited nothing; this gate
had a scout's notes to work from and treated them the same way. Two claims in
those notes are corrected in §42.16, one finding of this gate's own was wrong and
is recorded in §42.15, and the headline below is in neither document.

**M5's first deliverable is not a STEP reader. It is a public way to build a
solid out of faces, and this kernel does not have one.** `Model::add_vertex`,
`add_edge`, `add_coedge`, `add_wire`, `add_face`, `add_shell` and `add_solid` are
all `pub(crate)`. The whole public builder surface is `ProfileBuilder` and five
named shapes — cube, extrusion, round bar, ball, bar cut off by a plane. An
importer's job is to turn `advanced_face` and `closed_shell` into a `Model`, and
there is no route from outside `rcad-topo` that does it. §42.5 is the
demonstration.

### 42.1 The unfreeze, ratified, and a freeze whose only exit had not been written

§39.7 item 7 said the `rcad-predicates` change was **"the first item on M5's
opening gate's desk", not a lane's**, on §31.5 and §15.9 — *"a change reopens
this gate and stops the fan"*. A lane took it (`939914e`, the only commit to that
crate after `f8165e6`, which is the commit that wrote the freeze). §41.3 records
the irregularity and hands the decision here.

**Ratified.** The change stands. Three reasons, in the order of how much they
carry.

**One: it cannot move an answer, and that is structural rather than measured.**
`meet_of_three_planes` now runs the old computation first and returns its `Ok`
untouched; the rescaled retry runs only on the `Err` branch.

```rust
let refused = match meet_of_three_planes_at_this_scale(first, second, third) {
    Ok(point) => return Ok(point),
    Err(refused) => refused,
};
meet_after_rescaling_each_plane(first, second, third)
    .map(Ok)
    .unwrap_or(Err(refused))
```

No input that was decided before is decided differently, and no input that was
decided before is decided at a different width. What §31.5's freeze exists to
stop is a change to that crate silently moving every rate in the tree; this one
cannot, by the shape of nine lines.

**Two: it was demonstrated at this gate, not argued.** On a copy of the tree in a
scratch directory, with the retry made to never answer and nothing else touched:

| | tests | outcome |
|---|---|---|
| the retry, removed | 2 | `three_mutually_perpendicular_planes_written_at_1e_200_still_answer`, `three_different_per_plane_exponents_give_the_same_box_as_one_shared_exponent` |
| everything else | **1139** | pass, unchanged |

**1139 of 1141 tests cannot tell whether the retry is there** — including all
seven `fuses_that_must_agree` families, all four §39.5 rate families and every
curved suite. That is a stronger statement than §41.3's re-measurement, because a
rate that did not move is consistent with a change that moves two things in
opposite directions and this is not.

**Three: the reason no real geometry reaches the new path was checked here.** The
one production route into `meet_of_three_planes` is `Plane::meet_of_three`
(`plane.rs:450`), reached from `three_surfaces.rs:174`; the four call sites in
`sphere.rs` and `cylinder.rs` are inside `#[cfg(test)]`. `Plane::meet_of_three`
passes `Plane::coefficients()`, and every `Plane` constructor normalizes the
normal to length one —
`coefficients_are_scaled_so_the_first_three_have_length_one` is the test. A
uniformly tiny coefficient array cannot arrive that way. It can arrive through
`rcad_predicates::meet_of_three_planes`, which is `pub` and takes raw `[f64; 4]`,
and that is the caller the fix serves.

**And the rule the lane broke is a rule that could not be obeyed.** §31.5 says a
lane that finds it needs a predicates change "comes back here rather than taking
it" — to a section that had already been written and closed. §39.7 forwarded that
to "M5's opening gate", which did not exist and would not exist for five lanes.
A freeze whose only release valve is a document nobody has written yet leaves a
lane two choices: leave a demonstrated gap open for a milestone, or break the
rule. **The rule is amended here rather than the lane blamed.**

> **§31.5 and §15.9, amended.** A frozen crate may be unfrozen mid-fan by a
> change that (a) is strictly additive — no input previously answered is
> answered differently — and (b) is demonstrated to be so by removing it and
> showing which tests fail, and (c) is recorded in the *closing* gate's section
> the way §41.3 recorded this one. Anything that is not strictly additive still
> stops the fan and waits for a gate. The freeze was never about who signs; it
> was about two fans contending for one crate (§15.9's table) and about a rate
> moving under everyone's feet. Neither happened here: §41's fan was lanes in
> sequence on one branch, and the rate is bit for bit.

**`rcad-predicates` is not re-frozen for M5.** §39.7 item 5 — `Exact::dot_of`
does not slide and `Exact::square_minus` does — is in that crate and §41.11 item
5 notes the §31.5 obstacle is gone. Freezing it again would recreate the same
trap. What replaces the freeze is the amendment above.

### 42.2 The same fix in two crates, written twice, with two different contracts

§39.7 predicted the rescale would be the right shape and §41.7 is the evidence:
the rotation defect wanted the same power-of-two rescale one type over. It is
worth being exact about what that shows, because it is not that the predicates
change enabled the rotation fix.

`rcad-geom` **could not** use `rcad-predicates`' version. `power_of_two.rs` is
`mod power_of_two;` — private to `rcad-predicates` — and its two helpers are
`pub(crate)`. `rcad-geom` has its own copies in `exact.rs`, `pub(crate)` there,
written a day later (`6a00ade`, 2026-09-01) than the predicates ones
(`a23d73d`, 2026-08-31). `isometry.rs:6` imports `binary_exponent` and
`multiplied_by_a_power_of_two` from `crate::exact`.

So the technique now has two implementations, and **their contracts differ on
purpose**:

| | on a scaled copy that is not a normal number |
|---|---|
| `rcad-predicates::power_of_two::scaled_by_a_power_of_two` | `None` — sinks the attempt, keeps the refusal |
| `rcad-geom::isometry::brought_into_the_range_its_squares_fit_in` | accepts it, and argues the slip is below `2^-1022` and therefore below any answer |

Both are right where they sit: a predicate's sign has to be about the input
exactly, and a unit quaternion's *direction* tolerates a slip under the smallest
normal. But two answers to one question in two crates is §1.5's shape, and the
next person to want the technique in `rcad-topo` will write a third. **Named as
an M5 item** (§42.13), not built here: the consumer that calls it due is the
third use.

**What the shape question actually settles.** The change was the right shape and
is now the fourth use of it — `orient3d`'s homogeneity retry, `exact.rs`'s slide,
`planes.rs`'s per-plane rescale, `isometry.rs`'s quaternion rescale — with
§41.2's Sterbenz shift a fifth member of the family that reconditions rather than
widens. §41's opening sentence, *when a bound looks too wide, ask what the
arithmetic was handed*, has five demonstrations and no counter-example.

### 42.3 What the crate survey establishes, and what it only reports

Roadmap §12 says M5 is STEP import. The survey behind this gate says the mature
stack is unavailable. **The survey's licence half is a fact and its maturity half
is a set of self-descriptions, and this section keeps them apart** — CLAUDE.md
asks for a demonstration where a paragraph is offered, and a README is a
paragraph.

**Verified on the tree.** `deny.toml` allows eleven licences — Apache-2.0,
Apache-2.0 WITH LLVM-exception, MIT, MIT-0, BSD-2-Clause, BSD-3-Clause, ISC,
Unicode-3.0, Zlib, CC0-1.0, MPL-2.0 — and no LGPL variant. `[[licenses.exceptions]]`
forbids `rcad` itself from claiming one, and `rug`, `malachite` and
`gmp-mpfr-sys` are named in `[[bans.deny]]`. `cargo deny check` runs in `just
default`.

**Verified today, against the crates.io API and the projects' own pages**, not
taken from the notes:

| | licence | latest | published | downloads | what it says about itself |
|---|---|---|---|---|---|
| `opencascade-sys` | **LGPL-2.1** | 0.3.0 | 2026-08-24 | 6 202 | bindings to OCCT, the industry kernel |
| `ruststep` | Apache-2.0 | 0.4.0 | 2024-09-20 | 62 820 | *"This project is still in experimental stage. DO NOT USE FOR PRODUCT."* |
| `truck-stepio` | Apache-2.0 | 0.3.0 | 2024-09-20 | 52 692 | *"Input will come further down the road."* — reading is not implemented |
| `step-io` | MIT OR Apache-2.0 | 0.2.4 | 2026-07-09 | **367** | *"⚠️ Experimental — early stage; expect breaking API changes at any time."* |

**What is established.** `opencascade-sys` fails `deny.toml`, so OCCT is out
*under this policy*. That is worth saying precisely: the wall is a **decision**
recorded in §7.1 (LGPL puts static-linking obligations on anyone embedding the
kernel), not a fact about the world, and the family of fixes the notes did not
name is *amend `deny.toml`*. This gate does not amend it and does not recommend
amending it — but "OCCT is impossible" is the wrong sentence and "we chose not
to" is the right one.

**What is only reported.** Nothing in that table was run. No file was handed to
any of those crates by the scout or by this gate. Two of them say they are
experimental and one documents that the feature we want is absent; that is three
self-descriptions, and a self-description is evidence about the author's
confidence and not a measurement of the code. **"No licence-clean crate is
production-grade" is not demonstrated and this section does not assert it.**

**And the notes understated `step-io` in the half that matters.** Its own entity
coverage matrix, read today, claims *read* support for `plane`,
`cylindrical_surface`, `conical_surface`, `spherical_surface`,
`toroidal_surface`, `b_spline_surface_with_knots`, `bezier_surface`,
`surface_of_revolution`, `surface_of_linear_extrusion` and `offset_surface`, and
for `manifold_solid_brep`, `advanced_face`, `closed_shell`, `edge_curve` and
`next_assembly_usage_occurrence`. Its README says the entity set was *"judged
against 50,000+ real-world files"*. Two GitHub stars is not evidence about any of
that; 367 downloads is a real number and says the code has few users, which is a
different claim.

**The one thing about it that decides anything is not its maturity.** Its README:
*"Heals non-standard input — what can be fixed is normalized; what cannot is
dropped with a reason, never silently."* Depending on it puts the healing
decision in a crate this kernel does not own, before a `Model` exists, where no
`&Policy` reaches it and no `ProvenanceLog` records it. That is §42.9's fork
being settled by a `Cargo.toml` line, which is the thing CLAUDE.md's strictness
rule exists to prevent.

### 42.4 The kernel can hold three of the eleven surfaces a file may carry

`Surface` has exactly three variants — `Plane`, `Sphere`, `Cylinder` — confirmed
by reading `surface.rs:25`. No `Cone` and no `Torus` exists anywhere; the only
mentions of either word in the whole of `crates/` are two doc comments saying
they are not there.

**§24.6's estimate of the blast radius is stale and the current figure is 22, not
five.** Files that pattern-match on the three variants:

| | files |
|---|---|
| `crates/*/src` | **22** |
| tests and benches as well | **38** |

A fourth variant is a compile break at every one of them, which is §40 working
rather than failing. **§39.7 item 15 is not "wire up an existing type": it is two
new surface kinds from zero — evaluation, a sign predicate, a distance, a
crossing with each of the other kinds, a mesher chart — plus the wiring.** A
torus is not a quadric and its sign predicate is a quartic, one step past
everything M3 and M4 built.

Set that beside `step-io`'s coverage matrix and the arithmetic is the whole of
this milestone's scoping problem: **a reader that reads eleven surface kinds
hands ten of them to a kernel that holds three.** The parser is not the
bottleneck. Neither is the licence. What decides how much of a real file this
kernel can hold is `Surface`, and no reader changes that.

### 42.5 There is no public way to build a solid out of faces

Demonstrated rather than argued, from a crate outside the workspace that depends
on `rcad-topo` by path:

```
error[E0624]: method `add_vertex` is private
```

`add_vertex`, `add_edge`, `add_closed_edge`, `add_edge_running`, `add_coedge`,
`add_wire`, `add_wire_of_steps`, `add_face`, `add_shell` and `add_solid` are all
`pub(crate)` in `model.rs`. `ModelData` is `pub(crate)` too. The public builder
surface is `ProfileBuilder`, `add_cube`, `add_extrusion`, `add_round_bar`,
`add_ball` and `add_bar_cut_off_by`.

**So the only public route from a stranger's file to a `Model` today is to forge
the kernel's own serialized bytes**, which is precisely the route §39.7 items 3
and 4 spent a lane hardening. An importer cannot use it: `FORMAT_VERSION` is
ours, the arenas' shape is ours, and writing an importer against `Model`'s serde
representation makes every future arena change a breaking change to the file
format.

**M5's first lane is a public B-rep builder.** Two things follow from CLAUDE.md
and neither is optional:

- It hands back keys and takes geometry. It never exposes `&FaceData` or any
  other arena payload; CLAUDE.md's rule about `FaceView`/`EdgeView`/`VertexView`
  is what the reading side already does and the writing side has to match.
- **Every arena invariant becomes caller input.** Today `add_coedge` may assume
  its edge exists because only this crate calls it. Made public, "the edge exists"
  is a bad parameter and not a bug, so it is an `Err`. That is §39.7 item 4's
  sweep one level up, applied before the code is written instead of after.

This is why the milestone's shape is what §42.10 says it is. It is also why the
lane cannot be a new crate on its own: a crate outside `rcad-topo` cannot reach
the arenas at all, so the builder is `rcad-topo`'s and the reader is somebody
else's.

### 42.6 `Model::validate` sees nothing that takes two solids to see

The scout's notes claim this and this gate demonstrated it. Five models built
through the public API, `Policy::certified()`, `Model::validate` on each:

| the model | `validate` |
|---|---|
| two unit cubes at the very same place | `Ok(())` |
| two unit cubes overlapping by half | `Ok(())` |
| a ball of radius 1 buried inside a cube of side 10 | `Ok(())` |
| two unit cubes stacked exactly face on face | `Ok(())` |
| two unit cubes with a `1e-9` crack between them | `Ok(())` |

Every one of those is something a real assembly contains: a duplicated part, an
interference, a part inside a housing, two parts that should have been sewn, a
hairline gap. **The sweep passes all five and is right to** — each solid is
internally sound, and that is all `validate` claims. `coincident_face` is a
private function in `fuse/select.rs`; `validate.rs` imports exactly one name from
`crate::fuse`, `name_of_surface_bits`. Nothing in the sweep compares one solid's
geometry against another's.

**The consequence for M5 is the sentence to take away: you cannot refuse by name
what you cannot detect.** §42.11's honest posture — refuse dirty input loudly —
is not free. It is a piece of work: a pairwise geometric check that does not
exist, taking `&Policy`, producing a named refusal. §42.12 criterion 8 is it.

It does not belong inside `validate`. §4.3's rule is that the full sweep runs at
test and import boundaries and per-entity checks run after mutations; a pairwise
check over solids is quadratic in a quantity `validate` is already linear in, and
an assembly is where solid counts get large. It is a separate public pass with
its own name.

**One correction to the notes while this is in view.** Their list of what the
sweep checks is missing two checks the fan added —
`check_which_side_of_its_trimming_each_curved_face_lies_on` and
`check_closed_edges_are_walked_the_way_their_surfaces_wind`. Neither compares two
solids, so the conclusion is unchanged.

### 42.7 Instancing has nowhere to go

Roadmap §9.1 puts this first among its STEP requirements:
*"`next_assembly_usage_occurrence` referencing one shape representation with
per-instance placements must come through as one node with N refs, not N
copies."*

`check_shells_and_solids` counts how many solids use each shell and refuses
anything but one:

```rust
0 => return Err(Invalid::Orphan { entity: "shell" }.into()),
1 => {}
count => return Err(Invalid::UsedMoreThanOnce { entity: "shell", count }.into()),
```

**A shell belongs to exactly one solid, and `Model::validate` says so by name.**
There is no sharing in the data model, so "one node with N refs" is not a thing
this kernel can hold. Preserving instancing the way roadmap §9.1 means it is a
change to what a solid *is*, in the same family as §31.8 item 11's "a `Model`
whose vertices are boxes".

**M5 does not make that change.** §13 rule 3 is satisfied the other way round:
the consumer is named — 500 bolts — and the *figure* that calls it due is not.
§24.16's rule is that a figure with no target is not an optimisation. So M5
imports N occurrences as N solids at N placements, **measures what that costs in
memory and in mesh time**, and publishes the number. If the number is bad, the
milestone that fixes it has a target to beat. §42.12 criterion 14.

### 42.8 Where an imported solid's numbers should live, and both costs are measured

An importer has to choose, for every solid it reads, between baking the file's
world coordinates into the stored geometry and factoring an `AXIS2_PLACEMENT_3D`
out into a location. §41.4 item 8 established that the second is possible and
that "STEP solids arrive with no placement at all" was a paragraph. It did not
say which to do. **Both sides now have a measurement against them and the gate
picks one.**

| choice | what it costs, measured |
|---|---|
| **bake** — world coordinates, every solid at the identity | the mesh floor depends on §41.2's shift, and a face whose numbers straddle the origin or spread over more than a factor of two gets no shift at all: `DeflectionBelowWhatAMeshCanReach`, §41.7's sixth row, **a gap** |
| **factor out** — stored numbers small, a location on the shell | §41.13 exactly: a general rotation of a pair at two placements breaks an exact face contact in **51.5% of 21 546 turns**, and of those the fuse refuses 98.2% and answers 0.95% with the pair unglued |

**Bake.** Three reasons.

1. §41.13's cost is a §19 violation and §41.7 row 6's is a refusal on a mesh.
   §4.1's ladder ranks them, and it is not close.
2. Baking puts every imported solid at the identity, so
   `two_faces_that_touch_stay_touching_when_one_movement_moves_both` applies to
   any pair of them turned together: exact, bit for bit, for the same reason
   §41.13 gives.
3. §41.2's shift was built for exactly this input — §41.4 item 8's own closing
   sentence, *"what the shift is built for is precisely a face at no placement at
   all"*.

**And it makes §41.7's sixth row M5's**, because baking is what walks into it. A
file whose part sits at `z ∈ [1e9, 3e9]` is the row's own example. §42.13 keeps
it deferred with the trigger named rather than promising it here, because the
spike is what says how often a real file lands there.

**This does not settle instancing**, which needs placements by definition. §42.7
takes the copies; a copy is baked like any other solid.

### 42.9 The healing fork, written up and left to the user

**This gate does not decide this one.** It sets out the options with their costs
and recommends one. The decision is the user's, and the recommendation below is
marked as awaiting it.

**The tension, stated exactly.** `Policy::tolerant` reinterprets an
already-computed predicate enclosure: `Strictness::TolerantFallback` lets
`resolve_under_policy` read a narrow `Decision::Undecidable` as `Zero` and tags
the entity with an `Approximation` saying how much slop that took. It never
merges a vertex, never sews a gap, never decides which entities exist. Roadmap
§9.2's healing list — sew shells within tolerance, remove slivers, fix wire
ordering — is topology-mutating and runs **before** any predicate is asked about
the result. It cannot travel as a `&Policy` argument to an operation that has not
started on geometry that does not yet exist.

Four options, and only the fourth is new:

| | where the repair happens | what it costs |
|---|---|---|
| **A** | nowhere: M5 refuses unhealed input by name | honest, and **not free** — §42.6 shows the detection does not exist, so the refusals have to be built before they can be named. An importer that refuses a large fraction of real files, and roadmap §12's "the corpora are usable" is only half true |
| **B** | inside a third-party reader | the tolerance is chosen outside this kernel, invisible to `Policy` and `ProvenanceLog`. `step-io` already does this and says so. The kernel's whole claim — strictness is explicit and provenance says what was approximated — is false at the one boundary a user cares about most |
| **C** | inside `rcad`, an explicit pass taking `&Policy` | wants a record of a *mutation*, which the notes read as forbidden by CLAUDE.md's "provenance is per decision, not per coordinate" |
| **D** | a separate crate above the kernel, producing a `Model` and a report | the kernel stays certified and refuses; the repair tool is a different product with a different promise. Costs a second thing to build and a second thing to test |

**The objection to C is weaker than it reads, and the user should know that
before choosing.** CLAUDE.md's rule is *per decision, not per coordinate*. "These
two vertices are one point" **is a decision** — one judgement, one record, one
`Cause`, one measured distance. It becomes per-coordinate only if the pass logs
every number it touched, which nothing forces it to do. What C actually needs is
a new `Cause` variant and a `Measures` arm, which is the shape §15.3 and §17.2
already used twice. C is not architecturally excluded; it is unbuilt.

**Recommendation, awaiting the user's decision: A for M5, with C written down as
the shape the milestone after it takes if healing is wanted at all, and B refused
outright.**

- **A, because the roadmap already chose it.** §13's risk register: *"Healing is
  unbounded | Fix only what your own operations produce; refuse dirty imports
  loudly."* That is this project's own answer to this question, written before
  any of the code was.
- **B refused outright**, for §42.3's reason: it settles the fork by a
  `Cargo.toml` line and makes the kernel's strictness claim untrue at the import
  boundary. This is the one part of the crate-choice question that is not about
  maturity.
- **C, not now.** Its consumer is a user who wants a broken file repaired rather
  than refused, and M5 has not met one yet. §13 rule 3.
- **D is the fallback if A refuses too much to be usable**, which is roadmap
  §13's second row and is exactly the risk the register names.

**What the user is being asked to decide**: whether "M5 refuses and does not
heal" is acceptable given that §42.12 criterion 10's measured import rate may
come back low, and whether C's per-decision reading above is the right reading of
the provenance rule. If the answer to the first is no, the milestone's shape
changes and this section is what gets rewritten.

### 42.10 What M5 is

Roadmap §12: *"M5 — STEP import. Instancing preserved. Now the corpora are
usable and the fuzzing loop closes."* Read against §42.4, §42.5 and §42.7, two
thirds of that sentence is a different milestone. §39.9 already scored the same
failure one level up: M4's list was built and the coverage was not.

**M5 is one sentence: a stranger's file becomes a `Model`, or it is refused by a
name that says which entity and why.** Everything else in roadmap §12's line is
either downstream of that or is a milestone of its own.

**The route for the reader: write it, in-crate, against the fixed AP203/AP214/AP242
structural subset — and measure before committing.** The argument, in order:

1. **The mapping is the work and no crate does it.** Every candidate stops at a
   parsed instance graph or at its own scene graph. Turning `advanced_face` into
   an rcad `Face` with a trimmed surface, wires that close and coedges that run
   opposite ways is rcad's job under every route.
2. **The parse is bounded.** Part-21's lexical wrapper plus roughly thirty
   entity kinds for solids. The EXPRESS *language* does not need a general
   parser; `step-io` reached the same simplification.
3. **A dependency that heals is the one thing §42.9 refuses.** This is the
   decisive difference between the routes and it is about behaviour, not
   maturity.
4. **`#![forbid(unsafe_code)]` removes the C route** (`stepcode`) without
   argument.

**And the first task in that lane is a measurement, not code.** CLAUDE.md says a
claim that something cannot be done needs a demonstration, and the same standard
applies to a claim that something *should* be done. So:

> **The spike.** Take a corpus of real files. Count, and publish: how many parse;
> which entity kinds actually appear and how often; what fraction of solids use
> only `plane`, `spherical_surface` and `cylindrical_surface`; what units and
> declared uncertainties appear; how many files carry
> `next_assembly_usage_occurrence`. It is legitimate to run `step-io` inside the
> spike as a dev-dependency to get those counts faster — measuring with a tool is
> not depending on it. **The spike's numbers are what §45 records and what
> decides whether the in-crate reader is small or is the milestone.**

**Lanes.** §13 rule 2's gate–fan–gate, §15.9's one owner per crate.

| Lane | What it is | Crate it owns | Opens |
|---|---|---|---|
| **0. This section** | §42 | `docs/decisions.md` | done |
| **1. The spike** | §42.10's measurement, no production code | none | now, **alone** |
| **2. The public B-rep builder** | §42.5, and every invariant it turns into caller input | `rcad-topo` | now, beside 1 |
| **3. The reader** | Part-21 lexing, entity resolution, AP-to-`Model` mapping, refusal names | a new `rcad-step` | when 2 lands |
| **4. Two solids in one model** | §42.6's pairwise check, and its refusal names | `rcad-topo`, **same owner as 2** | when 2 lands |
| **5. Units and uncertainty** | §42.12 criteria 6 and 7 | `rcad-geom` (`scale.rs`), then `rcad-step` | beside 3 |
| **6. The corpus harness and the rates** | test-only | `rcad-step/tests/`, `rcad/tests/` | when 3 lands |
| **7. The three inherited defects** | §42.12 criteria 11, 12, 13 | `rcad-geom`, `rcad-topo`, `rcad-predicates` | any time; independent |

**Lanes 2 and 4 are one owner** for §31.6's reason: they are the same crate and
the second is written against the first's arenas.

**Section numbers, reserved now.** §43 is the builder, §44 is the reader, §45 is
M5's closing gate.

### 42.11 What M5 refuses by name, up front

§39.7's own closing paragraph said an importer inherits every geometric gap the
kernel has. **An importer that loads a file and then refuses to fuse it is honest
and has to say so before the file is loaded.** This table is that sentence, made
into a list the milestone is judged against.

| The gap | Where it bites | M5 |
|---|---|---|
| two coincident curved faces (§39.7 item 10) | `fuse` only; §42.6 shows `validate` never reaches it | **refuses by name**, and does not touch. The importer says up front that fusing imported solids is where it lives |
| a corner exactly on a curved face (item 11) | `fuse` only | **refuses by name**, does not touch |
| a ball with a pipe through it (item 9) | `fuse` only | **refuses by name**, does not touch |
| a turned notched bar, a bar cut off its axis (item 13) | `fuse` only | **refuses by name**, does not touch |
| the piece round an off-axis cut, holding both poles (§41.7 row 2) | `fuse` only, and it is a **§19 violation** | **refuses by name**, does not touch. §41.12 says why: it wants §38.3's machinery and a rotatable chart, which is a change to `Sphere` or to `TrimmedDomain`, and `rcad-mesh` is at 0 files changed since M3 |
| a ball cut by a plane through its polar axis (item 12) | `fuse` only | **refuses by name**, does not touch |
| the residual mesh floor at **1.4444×** (§41.7 row 4) | every fused *and every imported* curved solid | **does not touch**, and the number is published in the importer's own documentation. Closing it means measuring the slip rather than bounding it, which stops being a property of one face |
| §19 for a pair whose members carry different placements (§41.13) | four ordinary public calls | **closes the half it can**: §42.12 criterion 11 bakes `add_extrusion`'s lift. The representation change stays deferred (§42.13) |
| a face straddling the origin or spanning more than 2× gets no shift (§41.7 row 6) | every baked imported solid far from the origin | **does not touch**; the spike says how often it is reached (§42.13) |
| cone, torus, B-spline surface, surface of revolution or extrusion | **the file, immediately** — eight of the eleven surface kinds a mainstream reader handles | **refuses by name, and this is the commonest refusal M5 will produce.** §42.12 criterion 4 |

**The sentence M5's documentation opens with, and this is a deliverable rather
than a nicety:** *this reader maps solids bounded by planes, spheres and
cylinders. Every other surface kind is refused by the name the file gives it. A
model that imports is internally sound and is not checked for interference with
anything else in the file unless you ask.*

### 42.12 M5 exit criteria

Fourteen. §31.7's shape, and §30.3's warning applies to all of them: **every name
below is a test that has to be written**, and a criterion that cannot fail is not
a criterion. Each names its consumer, per §13 rule 3.

1. A solid can be built through the public API face by face, and one built that
   way is the same solid `add_cube` builds — `a_cube_built_face_by_face_is_the_cube_add_cube_builds`.
   **Consumer: the reader. Nothing outside `rcad-topo` can reach an arena today
   (§42.5).**
2. No sequence of builder calls panics; every misuse is a named `Err` —
   `no_sequence_of_builder_calls_panics`, as a fuzz target over a script of
   calls. **Consumer: CLAUDE.md's tier table and §4.1. This is §39.7 item 4's
   rule applied to an API before it exists rather than to fifty-eight clauses
   after.**
3. A hand-written AP203 file holding one block reads in and measures its closed
   form — `a_block_written_by_hand_in_a_step_file_measures_what_its_numbers_say`.
   **Consumer: the first file anybody imports.**
4. A surface kind the kernel cannot hold is refused by the name the file uses,
   and the error carries that keyword —
   `a_file_holding_a_cone_is_refused_by_the_name_the_file_gives_it`.
   **Consumer: eight of eleven surface kinds (§42.4). This is the common path,
   not the rare one, and it is what §42.11's opening sentence is checked
   against.**
5. No sequence of bytes makes the reader panic — `no_step_file_makes_the_reader_panic`,
   a second fuzz target seeded from the corpus. **Consumer: roadmap §12's "the
   fuzzing loop closes", and CLAUDE.md's rule that bad caller input is an `Err`
   and never a panic — §4.1's tiers, with §4.3 for where the sweep runs.**
   **Not §16.11**, which is about the dynamic-range check being a ratio and says
   nothing about bytes; §41.4 item 6 records that fifteen doc comments still
   miscite it and this criterion does not add a sixteenth.
6. The file's declared unit reaches `Scale::unit`, and a unit the kernel does not
   have is refused rather than silently converted —
   `a_file_written_in_inches_comes_back_in_inches`,
   `a_unit_the_kernel_does_not_have_is_refused_rather_than_scaled`.
   **Consumer: roadmap §9.1, and §3.5's rule that two models in different units
   are two different models.**
7. The file's declared uncertainty is read, and a file asking for more precision
   than its own numbers carry says so —
   `a_file_declaring_more_precision_than_its_numbers_carry_says_so`.
   **Consumer: roadmap §9.1's own sentence.** `Scale` carries a unit and a
   nominal size and has nowhere to put a declared uncertainty; that is the change.
   **No threshold is offered here**, per §31.7's rule — the lane measures it and
   §45 records it.
8. Two solids that occupy the same space are not called a valid model, and two
   that merely touch and two that are far apart still are —
   `two_solids_that_occupy_the_same_space_are_not_called_valid`,
   `two_solids_that_only_touch_are_still_a_valid_model`.
   **Consumer: every assembly with a duplicated or interfering part.** §42.6 is
   the demonstration that nothing catches it today, and it is a new public pass
   taking `&Policy`, not a check inside `validate`.
9. Every solid the corpus yields validates, or the refusal names which solid and
   which invariant — `every_solid_in_the_corpus_validates_or_the_refusal_names_one`.
   **Consumer: anyone debugging a file that will not load.**
10. The import rate, family by family and never blended (§17.7): a control of
    files this kernel wrote itself at **100%**, and a real-file family measured
    and asserted below a threshold recorded in §45 — `how_often_a_real_file_imports`.
    **No number is offered, on purpose** (§31.7). The control is what makes the
    headline mean anything.
11. `add_extrusion` bakes its lift, and a pair built at a lift and turned
    together keeps its contact —
    `a_pair_built_at_a_lift_and_turned_together_keeps_its_contact`.
    **Consumer: every solid built anywhere but the origin, reachable in four
    public calls (§41.13).** It closes the demonstrated case and nothing wider,
    and the row that stays open is named in §42.13.
12. The mesher reads a placed face's stored surface instead of round-tripping it
    through its own placement — §41.11 item 19, `256·ε` charged at `18·ε` —
    `a_turned_curved_face_is_not_charged_for_a_round_trip_it_does_not_make`.
    **Consumer: every imported turned part**, which is most of them.
13. `Approximation` gets the boundary guard `Scale` has, and the ordering check
    stops being silently false on `NaN` — §41.11 item 20 —
    `an_approximation_that_is_not_a_number_is_refused_off_the_wire`.
    **Consumer: any `Model` that has been through a file, which is what M5 is.**
14. N occurrences of one shape come through as N solids at N placements, and what
    that costs is measured and published —
    `five_hundred_bolts_come_through_as_five_hundred_solids_at_five_hundred_placements`.
    **Consumer: roadmap §9.1.** Sharing is deferred and §42.13 names the figure
    that calls it due.

**Criteria 1, 8 and 10 are the load-bearing three.** One is the only new
construction the whole milestone rests on — nothing else can be built until it
exists. Eight is the question a file makes urgent and no code answers. Ten is the
control that makes the headline mean anything, and §17.7's precedent is that the
control is what tells a timid filter from real undecidability.

The claim audit runs before §45, per §16.10, and every refusal M5 can produce is
classified limit or gap by name, as §39.4 did for M4.

### 42.13 What stays deferred, and what calls it due

| | why it is deferred | what calls it due |
|---|---|---|
| **cone and torus** | 22 files in `src`, two new surface kinds from zero, a quartic sign predicate for the torus (§42.4). Refused by name meanwhile | **the spike's count**: what fraction of corpus solids need one. It is the largest single item M5 could contain and the number decides whether it is M5's or its own milestone |
| **B-spline surfaces as faces** | `BSplineSurface` exists in `rcad-geom` and reaches no arena; a B-spline crossing has no closed form and needs the free-form marcher §24.1 sent away | the same count |
| **shared shells for instancing** | a change to what a solid is; `validate` refuses a shell used twice by name (§42.7) | criterion 14's published memory and mesh-time figure, against a target somebody states. §24.16 |
| **one home for the power-of-two rescale** | two implementations with two contracts, in two crates that cannot import each other's (§42.2) | **the third use.** Two is a coincidence; three is a design |
| **§39.7 item 5** — `Exact::dot_of` does not slide and `Exact::square_minus` does | a change to `Exact` and therefore a re-measurement of every rate in §41.9. The §31.5 obstacle is gone (§42.1) but the consumer is still hypothetical | **a corpus file that reaches it.** "A STEP file with an awkward unit scale" is a named consumer only once a file has been named |
| **§39.7 item 6** — two authorities on *is this plane along this pipe's axis*, `intersect` exact and `== 0.0` elsewhere | moves `walk`'s dispatch and therefore the mesher; its own lane with its own re-measurement | the same. It is §1.5's shape and it is a **gap**, so it is due the moment a file reaches it |
| **§41.7 row 6** — a face straddling the origin or spanning over 2× gets no shift | §42.8 makes baking the choice, which walks into it | **the spike again**: how many corpus solids sit far from the origin and straddle |
| **§41.7 row 4** — the residual `1.4444` mesh floor | closing it means measuring the slip instead of bounding it, which stops the floor being a property of one face | a caller who needs a mesh finer than `1.4444×` the floor and says so |
| **§41.11 item 21** — `BSplineSurface` lacks the demonstration `BSplineCurve` has | its consumer is a file holding a B-spline surface, which M5 refuses by name | the row above it |
| **§39.7 items 14, 16, 17** | unchanged from §31.9 and §39.7; item 17 is settled as a design consequence and should stop being handed on | unchanged |

### 42.14 What this gate does not know

§30.3's lesson and §31.10's shape: the places this section plans on top of a
guess, worst first.

1. **How much of a real file this kernel can hold.** Every number in §42.10 and
   §42.13 waits on the spike, and this gate has run no file through anything.
   §42.4's arithmetic — three surface kinds against eleven — could mean 5% of
   solids import or 60%, and the two answers are different milestones. **This is
   the schedule risk and the spike is deliberately the first lane, alone.**
2. **What a public B-rep builder should look like.** Face-at-a-time with keys, a
   transactional batch that validates at commit, or a builder that takes a whole
   shell description. Each turns a different set of invariants into caller input.
   The lane's first day should be spent choosing, not building — the same
   instruction §31.10 gave lane 3 of M4, which was the one that came in on time.
3. **Whether criterion 8's pairwise check is affordable.** Quadratic in solids
   with a certified predicate at each pair. An assembly with a thousand parts may
   need a bounding-volume pass first, which is §39.7 item 16's BVH acquiring a
   consumer for the first time. This gate does not know and does not pre-build
   it.
4. **Whether "refuse and do not heal" survives contact with a corpus.** §42.9's
   recommendation is the roadmap's own and it has never been tested against a
   real file by anybody on this project.
5. **Whether baking is still right after the spike.** §42.8 picks it on two
   measurements that are both about kernel-built solids. A corpus of parts at
   large offsets could make §41.7 row 6 the common case and change the answer.

**The prediction, recorded before the work and to be scored at §43 rather than at
the milestone's end** (§30.2's cheaper half). §12.1's lesson is that the money
goes to code that was already written and looked finished, and §30.2 found the
precondition: it holds for milestones that mostly rewire. **M5 mostly writes**,
so this gate predicts the lesson does *not* hold and the money goes instead to
**the public builder's error surface** — not to parsing, not to the AP mapping.
Every `pub(crate)` builder in `model.rs` today is written on the assumption that
its caller is this crate; making them public means auditing each for what a
stranger can hand it, which is fifty-eight `# Panics` clauses' worth of work
(§41.3) on functions that do not carry the clauses yet. The second guess is
criterion 8. The third, and the one this gate would most like to be wrong about,
is that the reader itself is two weeks and is the part everybody talks about.

### 42.15 The claim audit at this gate, including the finding that was wrong

Run per CLAUDE.md before this section was written, pointed at the scout's notes,
at §41's own citations, and at the two claims this charter rests on.

**Inventory.** Four impossibility claims (the licence wall, "no licence-clean
crate is production-grade", "the frame-change fix cannot be expressed" §41.1,
"no member of the constant-widening half reaches 1" §41.2), two unreachability
claims (`validate` cannot see coincidence; a uniformly tiny coefficient array
cannot reach `meet_of_three_planes` from real geometry), one sufficiency claim
(§41.7's and §41.12's *"`a_rotation_written_at_any_scale_is_the_same_rotation` is
the demonstration"*).

**The finding this gate had, and it was wrong.** A repo-wide `grep` for
`a_rotation_written_at_any_scale_is_the_same_rotation` found it in five places —
§41.7's table, §41.12, and three doc comments at `plane.rs:279`,
`isometry.rs:311` and `cylinder.rs:110` — and in no test function. The draft
finding was that §41 and three doc comments cite a demonstration that does not
exist. **It does exist. It is a test *file*,
`crates/rcad-geom/tests/a_rotation_written_at_any_scale_is_the_same_rotation.rs`,
holding five tests, and `grep` over file contents does not match a filename.**
The mutation is what settled it: with `brought_into_the_range_its_squares_fit_in`
made the identity on a scratch copy, four tests fail and all four are in that
file — `a_quarter_turn_whose_squares_overflow_is_still_a_quarter_turn`,
`..._underflow_...`, `a_quarter_turn_written_at_every_power_of_two_scale_is_the_same_quarter_turn`
and `a_third_of_a_turn_about_a_leaning_axis_is_the_same_at_every_exact_scale`.
**§41's citation is sound.** Recorded because a gate that only prints the
findings that survived is not showing its method, and because "grep found nothing"
is the weakest possible evidence for an absence.

**Independently re-derived here**, from a crate outside the workspace:
`[2^k, 2^k, 0, 0]` names the same quarter turn about `x` at **all 2098** scales
an `f64` exponent field holds — 2098 right, 0 identity, 0 refused. §41.12's
figure, from a second direction.

**Findings that stand.**

1. **"No licence-clean crate is production-grade" is not demonstrated.** §42.3.
   Nothing was run. Three self-descriptions and one missing feature are not a
   measurement, and the notes' own bottom line converts them into a verdict. The
   sentence that survives is in §42.3's table.
2. **"OCCT is excluded" is a decision, not an impossibility.** §42.3. The family
   the notes did not name is amending `deny.toml`. This gate does not amend it;
   it corrects the sentence.
3. **The notes' account of `step-io` is understated where it matters and
   overstated where it does not.** Its coverage matrix claims read support for
   the full mainstream surface and topology set; its README claims judgement
   against 50 000+ files. Two GitHub stars is not evidence about either. 367
   downloads is.
4. **The notes' list of `validate`'s checks is two checks stale**, and the
   conclusion is unchanged. §42.6.

**Checked hardest and sound.** §41.2's *"no member of the constant-widening half
reaches 1"* — the four-row table in §41.2 is an enumeration of the family and the
`18·ε·L` floor is arithmetic; the family it does not close is named in the same
paragraph and deferred as §41.7's fourth row, which is the vocabulary working.
§41.1's *"the frame-change fix cannot be expressed"* — `Sphere::transformed`
moves only the centre, checked, and the fix named needs a different crate; it is
a **limit of today's types** and §41.12 files it as a milestone rather than as a
refusal, which is right.

**Was it worth running?** Yes, and for the reason the skill gives rather than the
one expected: the finding it broke was this gate's own, one paragraph before it
was written down.

### 42.16 Corrections to §41 and earlier, and the numbers

- **§24.6's list of what a new `Surface` variant touches names five places** —
  `validate`, the mesher, `is_wholly_flat`, the fuse, serialization — and §39.7
  item 15 carries it forward. The tree has **22 files in `src`** that match on
  the three variants, and 38 with tests and benches. §42.4. The list is right
  about kinds and wrong about size, which is what makes item 15 read like
  wiring.
- **§39.7's framing of items 3, 4 and 8 as "the import boundary itself" is right
  and incomplete.** The import boundary also includes an API that does not exist:
  there is no public way to build a solid out of faces. §42.5. Nothing in §39.7
  or §39.9 mentions it, and it is the first thing M5 needs.
- **Roadmap §9.1's instancing requirement has no home in the data model.**
  `validate` refuses a shell used by two solids, by name. §42.7. The roadmap is
  the long-range plan (§1) and this is one of the places it and the tree
  disagree deliberately.
- **§41.4 item 8's closing observation is now a decision.** It noted that the
  shift is built for a face at no placement at all. §42.8 makes that the
  importer's rule and gives the two costs measured against each other.
- **§41.7's second row and §41.13's gap are both §19 violations and only one of
  them says so** — §41.12 corrected the first. Both are in §42.11's table as
  §19 violations, which is the vocabulary §39.4 sorts thirty-three rows by.
- **§31.5 and §15.9's freeze rule is amended**, §42.1, because as written it
  could not be obeyed by a lane in the middle of a fan.
- **`rcad-predicates` is not frozen for M5**, and this section is where that is
  recorded rather than left as an absence.
- **The scout's notes at
  `scratchpad/m5-gate/m5-scope.md` are superseded by this section** wherever the
  two differ. Their three load-bearing claims held — the licence list, the three
  `Surface` variants, and `validate`'s blindness to coincidence — and §42.15
  lists where their wording did not.

**The numbers at the gate.**

| | at §41.12 | now |
|---|---|---|
| tests (`cargo nextest`, whole workspace) | 1136 | **1141** |
| suite duration | 2.76 s | **2.759 s** |
| doc tests | 19 | 19 |
| `FORMAT_VERSION` | 3 | **3** |
| fuzz corpus | 824 | 824 |
| fuzz targets | 1 | **1**, and criterion 5 makes it two |
| wildcard arms over our own enums | 76 | **76** |
| `Surface` variants | 3 | **3** |
| public ways to build a solid out of faces | — | **0** (§42.5) |

A gate that writes one file moves nothing, which is what it should look like.

