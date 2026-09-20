## 104. The hollow — the unit after a plane, a cone and a coaxial pipe, recorded 2026-09-15

*Planning section. Written before any lane runs, against `main` at `6939147`.
Its numbers are re-derived here and name the command that produced them,
per §51.6; where a claim has no command behind it yet, it says so in place of
citing one.*

### 104.1 The name, and what the unit is

**The unit is called *the hollow*, and the roadmap's M6 number is left alone**,
which is §44.1's rule and the reason every unit since has had a name. Roadmap
§12 keeps M6 for local operations as a whole; this unit is the first of them.

The name is the operation: shelling turns a solid into a hollow one. Roadmap §8
orders the local operations by difficulty and puts it first — *"Shelling /
thickening — offset all faces, retrim. Easiest."* — and that ordering is the
decision of 2026-09-15, taken over starting at fillets.

**What exists today is nothing.** `grep -ril 'fillet\|chamfer\|shelling\|
draft_angle' crates --include=*.rs` reaches one test file, whose name is about
crossings. There is no local operation in this kernel, so this unit does not
extend one.

### 104.2 §13 is not satisfied yet, and that is the first gate

**§13 says a part with no named consumer is not built.** Shelling is a feature
and not a correctness gap, so §13 governs it, and CLAUDE.md says so in those
words.

`grep -inE 'shell|thicken|fillet|chamfer|offset|draft' docs/consumers.md`
returns no demand for any local operation. `rscene` has asked for a great many
things and this is not among them.

So the unit's first obligation is not code. Either a consumer raises the demand
the way `docs/consumers.md` records demands being raised, decided and landed, or
§13 is argued with and the record changed — CLAUDE.md's principle 6 permits the
second and forbids quietly doing neither. **No lane runs until one of those two
has happened.** This section names it rather than assuming the roadmap's
ordering is itself a consumer, because it is not: the roadmap is the north star
and §1 says it is not a description of this codebase.

### 104.3 The sentence the unit earns

> **A solid whose faces are planes can be hollowed to a wall of one thickness,
> and the result passes `Model::validate()`.**

Planes only, and one thickness, because that is the case whose offset is exact.
Offsetting a NURBS face has no exact NURBS answer in general — roadmap §8 says
so and §6 says why — and a unit that promised the general case would be
promising a number before its measurement, which §53.3 is the fourth unit in a
row to have been caught doing.

Falsifiable two ways, and **neither test exists yet**; writing them is the
unit's first lane, and naming them here rather than citing a line number is
§53.9's rule applied honestly:

- a box hollowed at thickness `t` has the volume of the box less the volume of
  the box inset by `t` on all six sides, to the tolerance the policy certifies;
  and
- the hollowed solid's outer shell is the original solid's, face for face, so
  the provenance of every outer face survives the operation.

### 104.4 What stays out

- **Fillets, chamfers, draft, sweeps, lofts and offsets.** Roadmap §8's order,
  and each is its own unit.
- **Non-planar faces.** The reason is §104.3's, and lifting it is the unit
  after this one if this one lands.
- **Variable thickness, and leaving faces open.** Both are what a shelling API
  grows next; neither has a consumer either, and §13 applies to them exactly as
  it applies to the whole.

### 104.5 What is not decided here

**The waves.** §53.4 and its four predecessors each named the lanes, their one
deliverable and their write-set before any lane ran, and that is the shape this
section is missing. It is missing deliberately: §104.2 is unanswered, and lanes
briefed against an unsettled scope are lanes that get re-briefed.

### 104.6 The predictions

**Inherited from §53.10, unchanged**, because §53 never ran and the bet was
always about whichever unit came next:

> **The next unit's largest error will be in a claim its spike made that no
> merged test carries.** Score it by taking every sentence in the planning
> section sourced to the spike and naming the merged test that would fail if it
> were false. A sentence with no such test is the specimen.

This section has no spike, so the bet's surface today is §104.3's two
falsifiers, both of which are declared as not-yet-written rather than cited.

**Carried from §52.13 by way of §103.3**, still unscored: that a load-bearing
test would turn out to be blind — its number coming from precisely the test that
was named. This unit's closing gate scores it, by mutating what each such test
claims to watch and confirming it fails. That is §53.7 charge 3, inherited whole
because it was never spent.

**This unit's own prediction, for the one after it:**

> **The largest error will be in what the hollowed solid's provenance claims.**
> §104.3's second falsifier says every outer face keeps its identity through the
> operation, and that is the kind of claim a test can assert about the faces it
> happens to build while being false about the faces a retrim splits. Score it
> by hollowing a solid whose faces the operation must divide, not one whose
> faces survive whole.

